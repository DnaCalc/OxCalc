# CORE_ENGINE_GRID_REFERENCE_MACHINE

Status: **Promoted planning spec** (2026-06-13). Companion to `CORE_ENGINE_GRID_MODEL.md`.

## 1. Purpose

GridCalc-Ref is the simple-correct reference implementation for the strict Excel grid profile. It is product test code, not disposable scaffolding. Optimized grid code proves itself by differential execution or by closed-form checks against this reference where full differential scale is too large.

## 2. State model

GridCalc-Ref represents each sheet as ordinary maps:

- authored cells: `BTreeMap<(row, col), CellState>`;
- axis props: per-row/per-column `BTreeMap` or run-expanded maps;
- tables, merged regions, and feature-rendered regions as explicit rect records;
- computed valuation as a separate `BTreeMap<(row, col), CalcValue>`;
- spill records as a plain anchor-keyed map recomputed during recalc.

It deliberately does not use block storage, template coalescing, persistent graph compression, interval indexes, tile streaming, or optimized publication structures.

The first optimized candidate is allowed to sit beside it as **GridOptimizedSheet**: a compact
authored-state representation with sparse point cells, dense row-major value regions, and
repeated formula regions. Its first valuation layer keeps dense literal results as computed
regions, keeps supported repeated-R1C1 numeric formula outputs as computed dense regions,
stores unsupported formula outputs sparsely, and reports `P-00`/`P-11` counters
(`cells_evaluated == occupied`, `formula_templates_prepared == distinct templates`). It must
also expose sampled readout, an optimized valuation-backed `ReferenceSystemProvider` for
OxFml/OxFunc formula evaluation, and a bounded expansion/projection into GridCalc-Ref for
differential checks before it is used as a production engine path. It also carries a seed-scale
warm no-op cache token over compact authored state, axis state, spill facts, names, tables,
merged regions, feature-rendered regions, and the materialization limit; an unchanged optimized
sheet can reuse the cached valuation and report `P-19` with zero cells visited and zero formula
evaluations, while any token mismatch falls back to normal recalc. The first executable harness
surface is in-memory: `GridEngineMode::Reference`, `GridEngineMode::Optimized`, and
`GridEngineMode::Both` run over explicit probe addresses with a caller-supplied
materialization limit. `grid_runner::GridCorpusRunner` is the first seed-corpus runner over that
surface: it parses the `reference`, `optimized`, and `both` engine argument values and runs
sparse, dense-values-only, dense-values-plus-aggregate, repeated-R1C1, defined-name,
table-overlay structured-reference, table omitted/current-row formula context, spill-anchor-ledger,
dynamic-SEQUENCE-spill, hidden-row visibility, structural-edit, and dynamic-invalidation
scenarios through the same engine-mode API.
`execute_seed_corpus(repo_root, run_id, engine)` writes
`docs/test-runs/core-engine/grid-seed/<run_id>/run_summary.json`, `case_index.json`, and
per-case result artifacts, including GridInvalidation-Ref scalar/dynamic closure sections where
the scenario declares them. The current CLI shell is
`cargo run -p oxcalc-tracecalc-cli -- grid-seed <run-id> --engine reference|optimized|both`; it
delegates to the same runner API rather than introducing a second semantic path.
The first retained grid scale runner is
`cargo run -p oxcalc-tracecalc-cli -- grid-scale <sparse-whole-column|full-column-1m|sparse-singletons|zig-zag-1m|dense-values|repeated-r1c1|fill-down-r1c1|pascal-r1c1-1m|boring-1mx10|direct-r1c1-1m|unary-r1c1-1m|argument-aggregate-r1c1-1m|math-function-r1c1-1m|mod-function-r1c1-1m|rounding-function-r1c1-1m|integer-function-r1c1-1m|log-function-r1c1-1m|trig-function-r1c1-1m|angle-function-r1c1-1m|reference-function-r1c1-1m|logical-function-r1c1-1m|if-logical-r1c1-1m|two-left-r1c1-1m|absolute-r1c1-1m|division-r1c1-1m|decimal-r1c1-1m|recursive-binary-r1c1-1m|if-r1c1-1m|if-branch-r1c1-1m|nested-if-r1c1-1m|iferror-r1c1-1m|comparison-r1c1-1m|comparison-expression-r1c1-1m|comparison-iferror-r1c1-1m|sum-row-r1c1-1m|sumsq-row-r1c1-1m|count-row-r1c1-1m|product-row-r1c1-1m|average-row-r1c1-1m|min-max-row-r1c1-1m|sum-window-r1c1-1m|division-error-r1c1-1m|division-error-propagation-r1c1-1m|aggregate-error-r1c1-1m|insert-storm-1m|publication-delta-1m|tile-stream-64k|viewport-64k-of-1m|cow-retention-1m|plan-cache-rounds-1m|range-invalidation-1m|range-query-1m|sum-pyramid-1m|dirty-rect-1m|hide-storm-1m|spill-anchor-1m|spill-blockage-1m|aggregate-context-1m|spill-epoch-1m|filter-spill-1m|sequence-spill-1m> <run-id> [--rows N] [--cols N]`.
It writes `run_summary.json`, `counter_summary.json`, and `register_assertions.json` under
`docs/test-runs/core-engine/grid-scale/<run_id>`. The counter summary includes the
`oxcalc.grid.optimized.authored_storage_bytes.v1` accounting model: packed numeric dense
payloads and compact sparse numeric points count as row/column/revision plus `f64` bytes,
edited dense subregions can share backing payload slices across retained compact roots,
repeated formulas count shared source/normal-form strings once per authored region, and blank
grid capacity contributes zero bytes.

The optimized evaluator has narrow repeated-R1C1 fast paths for direct scalar R1C1 references,
parenthesized direct scalar R1C1 references, unary-minus scalar R1C1 expressions, and a bounded recursive binary
expression class over relative and absolute R1C1 references, finite numeric literals,
parenthesized scalar subexpressions, and `+`, `-`, `*`, or `/` with ordinary arithmetic
precedence; direct comparison formulas over operands or scalar expressions, including nested
`IFERROR`, that publish logical values,
numeric `IF(...)` comparisons and logical-function conditions with scalar-expression arms, including nested scalar `IF`
branches, `IFERROR(...)` over bounded scalar R1C1 operands/binary
expressions/range aggregates, argument-list aggregates over bounded scalar R1C1 expressions and
finite R1C1 ranges, scalar math functions `ABS`, `SQRT`, `POWER`, and `MOD`, plus `ROUND`,
`ROUNDUP`, `ROUNDDOWN`, `INT`, `TRUNC`, `EXP`, `LN`, `LOG10`, `LOG`, `SIN`, `COS`, `TAN`,
`RADIANS`, `DEGREES`, and `PI` over bounded scalar R1C1
expressions, reference-identity functions `ROW`, `COLUMN`, `ROWS`, and `COLUMNS` over current
cell, R1C1 references, and finite R1C1 ranges, bounded logical functions `AND`, `OR`, and `NOT` over comparison/logical
expressions, plus
`SUM(<R1C1 ref>:<R1C1 ref>)`,
`SUMSQ(<R1C1 ref>:<R1C1 ref>)`,
`COUNT(<R1C1 ref>:<R1C1 ref>)`,
`PRODUCT(<R1C1 ref>:<R1C1 ref>)`,
`AVERAGE(<R1C1 ref>:<R1C1 ref>)`,
`MIN(<R1C1 ref>:<R1C1 ref>)`, and
`MAX(<R1C1 ref>:<R1C1 ref>)` over finite row/column ranges. When a repeated
region has no punch-through overrides, numeric results publish as packed dense numeric output;
direct division by zero, single upstream worksheet-error operands, and the retained aggregate-error
lane `SUM(<numeric ref>:<error ref>)` plus `IFERROR(SUM(...),...)` publish dense error/recovered
output through the `CalcValue` dense payload. These templates still live in the profile-bound formula
surface and retain normal-form/template counters. Unsupported formulas, non-numeric operands,
other range functions, broader multi-error precedence, and broader non-arithmetic worksheet-error
algebra continue through the ordinary OxFml/OxFunc/provider path.

## 3. Recalc algorithm

The first value/effects reference floor is mark-all-dirty. Its primary recalc phase is the
`P-00` exactly-once surface; any spill repair revisit runs in named repair phases with separate
counters and is excluded from that primary-phase assertion.

1. bind/evaluate each occupied formula independently through the ordinary OxFml/OxFunc stack;
2. recompute spill placement in deterministic spec order;
3. iterate bounded spill repair passes using the same cap as the semantic spec;
4. publish computed values and committed effects after the pass quiesces or reaches the spill cap.

The reference machine may share leaf formula/function evaluation with the optimized engine. It
may not share optimized storage, graph, invalidation, or publication machinery.

Because this floor is mark-all-dirty, GridCalc-Ref is **not** the oracle for exact invalidation
closure. Exact dirty-closure equivalence is checked by a separate **GridInvalidation-Ref**
scalarizer: a simple expanded dependency model that records semantic dependency edges over
cells, spill facts, axis-visibility ranges, tables, merged regions, and structural-edit shifts.
Optimized invalidation compares against that scalarizer, while GridCalc-Ref remains the oracle
for values and committed effects.

## 4. Readout and differential contract

The differential harness runs one scenario against GridCalc-Ref, GridInvalidation-Ref where
dirty closure is in scope, and the optimized engine, then compares:

- all occupied authored cells;
- all committed spill extents and blocked-anchor facts;
- boundary probes around row 1, row 1,048,576, col 1, col 16,384, block edges, and sampled
  blanks;
- invalidation closure for small and medium cases, using GridInvalidation-Ref rather than the
  mark-all-dirty value reference;
- declared feature-rendered-region flags once admitted.

Full differential is capped at the reference budget. Above that cap, optimized runs use
closed-form workload expectations and sampled readout cones.

The current W061 executable slice is the value/effects plus dirty-closure seed harness:

- `Reference` mode projects compact authored state into GridCalc-Ref and evaluates through
  OxFml/OxFunc using the grid provider.
- `Optimized` mode evaluates through `GridOptimizedValuation` and its valuation-backed
  `ReferenceSystemProvider`.
- `Both` mode executes both paths and returns probe-level `GridDifferentialMismatch` rows.
- `GridCorpusRunner` binds those modes to the first seed corpus and reports expectation
  mismatches separately from reference/optimized differential mismatches.
- `GridCorpusRunner::execute_seed_corpus` emits durable run artifacts with recalc counters,
  probe readouts, expected values, invalidation scalar/dynamic closure reports, namespace
  lifecycle reports, optimized warm no-op/P-19 reports, P-20 occupied-slot enumeration reports
  where available, and mismatch sections for the seed cases.
- The current checked CLI evidence run is
  `docs/test-runs/core-engine/grid-seed/w061-grid-seed-cli-009`, emitted with `--engine both`;
  it passed 18 seed cases with zero expectation, differential, invalidation, and P-20
  mismatches.
- In that run, `grid_seed_repeated_r1c1_formula_region_001` emits an optimized warm no-op
  cache-hit report with `cells_visited == 0`, `formula_evaluations == 0`,
  `p19_warm_noop_holds == true`, and readout equal to the optimized baseline.
- In the same run, `grid_seed_whole_axis_value_invalidation_001` emits a P-20 report for
  profile-bound whole-axis formulas: `1:1` declares 32 cells and visits 2 occupied slots, while
  `A:B` declares 256 cells and visits 2 occupied slots. The unit test
  `optimized_grid_whole_column_enumeration_visits_occupied_slots_not_extent` covers the
  strict-Excel bound for `A:B` with 2,097,152 declared cells and 2 occupied slots visited.
- First grid-scale smoke artifacts cover the optimized storage profiles called out by
  W061: `w061-grid-scale-sparse-whole-column-001` proves strict `A:A` declares 1,048,576 cells
  and visits 3 occupied slots while recording zero bytes for blank grid capacity;
  `w061-grid-scale-full-column-1m-001` proves the fully occupied `full-column-1M` aggregate lane:
  a 1,048,576-cell packed dense numeric column plus `SUM(A:A)` declares and visits exactly
  1,048,576 dense slots, visits zero sparse slots, intersects one compact region, computes
  549756338176, and warms with zero cells visited;
  `w061-grid-scale-sparse-singletons-001` stores 1,000,000 isolated numeric authored cells as
  compact sparse points at 24 B/cell with zero blank-cell bytes;
  `w061-grid-scale-zig-zag-1m-001` stores 1,000,000 isolated numeric authored cells as compact
  sparse points while wrapping diagonally across all 16,384 strict Excel columns, also at
  24 B/cell with 16,383,000,000 blank cells costing zero bytes, and records a valid P-18
  partition witness with a 1,000,000-point parallelism bound;
  `w061-grid-scale-dense-values-001` keeps 10,000 dense values in one packed numeric region
  with zero computed sparse cells and about 8.014 authored bytes per dense cell;
  `w061-grid-scale-repeated-r1c1-001` evaluates 1,000 repeated R1C1 formula cells with one
  prepared template, about 0.260 authored formula bytes per formula cell, packed numeric
  inputs, dense computed formula output (`computed_sparse_cells == 0`), and a warm no-op pass
  visiting zero cells, with P-14 reporting 1 formula-plan miss and 999 hits;
  `w061-grid-scale-fill-down-r1c1-001` evaluates a 1,000,000-row `=R[-1]C+1` fill-down with one
  prepared template, 999,999 formula cells, first/middle/last values 1/500000/1000000, shared
  formula authored bytes, one sparse seed value plus one dense computed formula-output region,
  a warm no-op pass visiting zero cells, and P-14 reporting 1 formula-plan miss and 999,998 hits;
  `w061-grid-scale-pascal-r1c1-1m-001` evaluates a 1,000,000-row x 8-column two-dimensional
  R1C1 recurrence `=RC[-1]+R[-1]C` with one dense boundary column, seven sparse top-row seeds,
  6,999,993 formula cells, one prepared template, one formula-plan miss plus 6,999,992 hits,
  two dense computed regions, seven sparse computed seed cells, sampled first/middle/last
  recurrence values matching the closed row-major recurrence, and a warm no-op pass visiting
  zero cells;
  `w061-grid-scale-boring-1mx10-001` evaluates the 1,000,000-row x 10-column boring workload
  with 8,000,000 packed dense numeric input cells plus a 2,000,000-cell repeated R1C1 formula
  block, one prepared template, about 6.400 authored bytes per occupied cell, sampled final
  formula value 4000000032, dense computed formula output (`computed_sparse_cells == 0`), a
  valid P-18 partition witness over one dense region plus one repeated-formula region, and a
  warm no-op pass visiting zero cells, with P-14 reporting 1 formula-plan miss and 1,999,999 hits;
  `w061-grid-scale-direct-r1c1-1m-001` evaluates a 1,000,000-row x 3-column direct scalar
  formula-output shape with one dense input column and two repeated R1C1 formula columns:
  `=RC[-1]` and `=(RC[-2])`. It prepares two templates and two compiled plans, records two
  formula-plan misses plus 1,999,998 hits, publishes 3,000,000 packed dense numeric computed
  cells with zero sparse computed cells, samples direct and parenthesized values
  10/5000000/10000000, warms with zero visits, and passes P-00/P-10/P-11/P-14/P-18/P-19 plus
  `GRID-DIRECT-R1C1-1M`;
  `w061-grid-scale-unary-r1c1-1m-001` evaluates a 1,000,000-row x 4-column unary scalar
  formula-output shape with one dense input column and three repeated R1C1 formula columns:
  `=-RC[-1]`, `=-(RC[-2]+5)`, and `=-RC[-3]*2+1`. It prepares three templates and three
  compiled plans, records three formula-plan misses plus 2,999,997 hits, publishes 4,000,000
  packed dense numeric computed cells with zero sparse computed cells, samples direct,
  parenthesized, and arithmetic values -10/-5000000/-10000000, -15/-5000005/-10000005, and
  -19/-9999999/-19999999, warms with zero visits, and passes P-00/P-10/P-11/P-14/P-18/P-19
  plus `GRID-UNARY-R1C1-1M`;
  `w061-grid-scale-argument-aggregate-r1c1-1m-001` evaluates a 1,000,000-row x 8-column
  aggregate-argument formula-output shape with two dense input columns and six repeated R1C1
  formula columns: `=SUM(RC[-2],RC[-1],5)`, `=COUNT(RC[-3],RC[-2],5)`,
  `=PRODUCT(RC[-4],RC[-3],2)`, `=AVERAGE(RC[-5],RC[-4],5)`,
  `=MIN(RC[-6],RC[-5],5)`, and `=MAX(RC[-7],RC[-6],5)`. It prepares six templates and six
  compiled plans, records six formula-plan misses plus 5,999,994 hits, publishes 8,000,000
  packed dense numeric computed cells with zero sparse computed cells, samples first SUM/COUNT/
  PRODUCT/AVERAGE/MIN/MAX values 16/3/20/5.333333333333333/1/10, samples middle SUM/PRODUCT/
  AVERAGE values 5500005/5000000000000/1833335, samples last SUM/PRODUCT/AVERAGE/MIN/MAX
  values 11000005/20000000000000/3666668.3333333335/5/10000000, warms with zero visits, and
  passes P-00/P-10/P-11/P-14/P-18/P-19 plus `GRID-ARGUMENT-AGGREGATE-R1C1-1M`;
  `w061-grid-scale-math-function-r1c1-1m-001` evaluates a 1,000,000-row x 5-column scalar
  math-function formula-output shape with two dense input columns and three repeated R1C1
  formula columns: `=ABS(RC[-2])`, `=SQRT(RC[-2])`, and `=POWER(ABS(RC[-4]),2)`. It prepares
  three templates and three compiled plans, records three formula-plan misses plus 2,999,997
  hits, publishes 5,000,000 packed dense numeric computed cells with zero sparse computed cells,
  samples first ABS/SQRT/POWER values 1/1/1, middle values 500000/500000/250000000000, and last
  values 1000000/1000000/1000000000000, warms with zero visits, and passes
  P-00/P-10/P-11/P-14/P-18/P-19 plus `GRID-MATH-FUNCTION-R1C1-1M`;
  `w061-grid-scale-mod-function-r1c1-1m-001` evaluates a 1,000,000-row x 5-column MOD
  function formula-output shape with two dense input columns and three repeated R1C1 formula
  columns: `=MOD(RC[-2],RC[-1])`, `=IF(MOD(RC[-3],2)=0,RC[-3]/2,RC[-3]*3)`, and
  `=MOD(POWER(RC[-4],2),RC[-3])`. It prepares three templates and three compiled plans,
  records three formula-plan misses plus 2,999,997 hits, publishes 5,000,000 packed dense
  numeric computed cells with zero sparse computed cells, samples first MOD/IF/POWER-MOD
  values 1/3/1, middle values 4/250000/2, and last values 1/500000/1, warms with zero visits,
  and passes P-00/P-10/P-11/P-14/P-18/P-19 plus `GRID-MOD-FUNCTION-R1C1-1M`;
  `w061-grid-scale-rounding-function-r1c1-1m-001` evaluates a 1,000,000-row x 5-column
  ROUND-family formula-output shape with two dense input columns and three repeated R1C1 formula
  columns: `=ROUND(RC[-2],RC[-1])`, `=ROUNDUP(RC[-3],RC[-2])`, and
  `=ROUNDDOWN(RC[-4],RC[-3])`. It prepares three templates and three compiled plans, records
  three formula-plan misses plus 2,999,997 hits, publishes 5,000,000 packed dense numeric
  computed cells with zero sparse computed cells, samples first ROUND/ROUNDUP/ROUNDDOWN values
  2/2/1, middle values 500001/500001/500000, and last values 1000001/1000001/1000000, warms
  with zero visits, and passes P-00/P-10/P-11/P-14/P-18/P-19 plus
  `GRID-ROUNDING-FUNCTION-R1C1-1M`;
  `w061-grid-scale-integer-function-r1c1-1m-001` evaluates a 1,000,000-row x 5-column
  INT/TRUNC formula-output shape with two dense input columns and three repeated R1C1 formula
  columns: `=INT(RC[-2])`, `=TRUNC(RC[-3])`, and `=TRUNC(RC[-4],RC[-3])`. It prepares three
  templates and three compiled plans, records three formula-plan misses plus 2,999,997 hits,
  publishes 5,000,000 packed dense numeric computed cells with zero sparse computed cells,
  samples first INT/TRUNC/TRUNC-tens values 1/1/0, middle values 500000/500000/500000, and
  last values 1000000/1000000/1000000, warms with zero visits, and passes
  P-00/P-10/P-11/P-14/P-18/P-19 plus `GRID-INTEGER-FUNCTION-R1C1-1M`;
  `w061-grid-scale-log-function-r1c1-1m-001` evaluates a 1,000,000-row x 6-column EXP/LN/LOG
  formula-output shape with two dense input columns and four repeated R1C1 formula columns:
  `=EXP(RC[-1])`, `=LN(RC[-3])`, `=LOG10(RC[-4]*100)`, and `=LOG(RC[-5]*100,10)`. It
  prepares four templates and four compiled plans, records four formula-plan misses plus
  3,999,996 hits, publishes 6,000,000 packed dense numeric computed cells with zero sparse
  computed cells, samples first/middle/last EXP/LN/LOG10/LOG values 1/0/2/2, warms with zero
  visits, and passes P-00/P-10/P-11/P-14/P-18/P-19 plus `GRID-LOG-FUNCTION-R1C1-1M`;
  `w061-grid-scale-trig-function-r1c1-1m-001` evaluates a 1,000,000-row x 4-column
  SIN/COS/TAN formula-output shape with one dense input column and three repeated R1C1 formula
  columns: `=SIN(RC[-1])`, `=COS(RC[-2])`, and `=TAN(RC[-3])`. It prepares three templates
  and three compiled plans, records three formula-plan misses plus 2,999,997 hits, publishes
  4,000,000 packed dense numeric computed cells with zero sparse computed cells, samples
  first/middle/last SIN/COS/TAN values 0/1/0, warms with zero visits, and passes
  P-00/P-10/P-11/P-14/P-18/P-19 plus `GRID-TRIG-FUNCTION-R1C1-1M`;
  `w061-grid-scale-angle-function-r1c1-1m-001` evaluates a 1,000,000-row x 6-column
  RADIANS/DEGREES/PI formula-output shape with two dense input columns and four repeated R1C1
  formula columns: `=RADIANS(RC[-2])`, `=DEGREES(RC[-2])`, `=SIN(RADIANS(RC[-4]))`, and
  `=PI()`. It prepares four templates and four compiled plans, records four formula-plan
  misses plus 3,999,996 hits, publishes 6,000,000 packed dense numeric computed cells with
  zero sparse computed cells, samples first/middle/last RADIANS/DEGREES/SIN/PI values
  0/0/0/3.141592653589793, warms with zero visits, and passes P-00/P-10/P-11/P-14/P-18/P-19
  plus `GRID-ANGLE-FUNCTION-R1C1-1M`;
  `w061-grid-scale-reference-function-r1c1-1m-001` evaluates a 1,000,000-row x 6-column
  formula-only reference-identity shape with six repeated R1C1 formula columns: `=ROW()`,
  `=COLUMN()`, `=ROW(RC[-2])`, `=COLUMN(RC[-3])`, `=ROWS(R1C1:R3C1)`, and
  `=COLUMNS(RC[-5]:RC[-3])`. It prepares six templates and six compiled plans, records six
  formula-plan misses plus 5,999,994 hits, publishes 6,000,000 packed dense numeric computed
  cells with zero sparse computed cells and zero literal cells, samples ROW values
  1/500000/1000000 plus COLUMN/ROWS/COLUMNS values 2/1/3/3, warms with zero visits, and passes
  P-00/P-10/P-11/P-14/P-18/P-19 plus `GRID-REFERENCE-FUNCTION-R1C1-1M`;
  `w061-grid-scale-logical-function-r1c1-1m-001` evaluates a 1,000,000-row x 5-column logical
  function formula-output shape with two dense signed input columns and three repeated R1C1
  formula columns: `=AND(RC[-2]>0,RC[-1]>0)`, `=OR(RC[-3]>0,RC[-2]>0)`, and
  `=NOT(AND(RC[-4]>0,RC[-3]>0))`. It prepares three templates and three compiled plans,
  records three formula-plan misses plus 2,999,997 hits, publishes 5,000,000 dense computed
  cells with 2,000,000 numeric-packed input cells, 3,000,000 logical-packed formula-output
  cells, and zero sparse computed cells, samples first AND/OR/NOT values false/true/true,
  row-3 values true/true/false, middle values false/false/true, and last values
  false/false/true, warms with zero visits, and passes P-00/P-10/P-11/P-14/P-18/P-19 plus
  `GRID-LOGICAL-FUNCTION-R1C1-1M`;
  `w061-grid-scale-if-logical-r1c1-1m-001` evaluates a 1,000,000-row x 5-column numeric IF
  formula-output shape with two dense signed input columns and three repeated R1C1 formula
  columns using `AND`, `OR`, and `NOT(AND(...))` conditions:
  `=IF(AND(RC[-2]>0,RC[-1]>0),RC[-2]+RC[-1],0)`,
  `=IF(OR(RC[-3]>0,RC[-2]>0),RC[-3]-RC[-2],0)`, and
  `=IF(NOT(AND(RC[-4]>0,RC[-3]>0)),ABS(RC[-4]),0)`. It prepares three templates and
  three compiled plans, records three formula-plan misses plus 2,999,997 hits, publishes
  5,000,000 dense numeric-packed computed cells with zero sparse computed cells, samples first
  AND/OR/NOT-condition IF values 0/2/1, row-3 values 6/0/0, middle values 0/0/500000, and
  last values 0/0/1000000, warms with zero visits, and passes P-00/P-10/P-11/P-14/P-18/P-19
  plus `GRID-IF-LOGICAL-R1C1-1M`;
  `w061-grid-scale-two-left-r1c1-1m-001` evaluates a second dense/repeated-R1C1 1M-row x
  10-column formula-output shape using `=RC[-2]+RC[-1]`: one prepared template, one compiled
  plan, 2,000,000 formula cells, one formula-plan miss plus 1,999,999 hits, one compiled-plan
  miss, dense computed formula output (`computed_sparse_cells == 0`), sampled values
  2015/1500000023/3000000023, zero-visit warm no-op, and P-00/P-10/P-11/P-14/P-18/P-19 plus
  `GRID-TWO-LEFT-R1C1-1M` passing;
  `w061-grid-scale-absolute-r1c1-1m-001` evaluates a 1,000,000-row x 3-column mixed
  absolute/relative R1C1 shape using `=RC[-1]+R1C1`: two packed dense input columns,
  1,000,000 formula cells, one prepared template, one compiled plan, one formula-plan miss plus
  999,999 hits, one compiled-plan miss, dense computed formula output (`computed_sparse_cells == 0`),
  sampled values 3/1000001/2000001, zero-visit warm no-op, and P-00/P-10/P-11/P-14/P-18/P-19
  plus `GRID-ABSOLUTE-R1C1-1M` passing;
  `w061-grid-scale-division-r1c1-1m-001` evaluates a 1,000,000-row x 2-column numeric-safe
  division shape using `=RC[-1]/2`: one packed dense input column, 1,000,000 formula cells, one
  prepared template, one compiled plan, one formula-plan miss plus 999,999 hits, one
  compiled-plan miss, dense computed formula output (`computed_sparse_cells == 0`), sampled
  values 1/500000/1000000, zero-visit warm no-op, and P-00/P-10/P-11/P-14/P-18/P-19 plus
  `GRID-DIVISION-R1C1-1M` passing;
  `w061-grid-scale-decimal-r1c1-1m-001` evaluates a 1,000,000-row x 2-column decimal-literal
  multiplication shape using `=RC[-1]*0.5`: one packed dense input column, 1,000,000 formula
  cells, one prepared template, one compiled plan, one formula-plan miss plus 999,999 hits,
  one compiled-plan miss, dense computed formula output (`computed_sparse_cells == 0`), sampled
  values 1/500000/1000000, zero-visit warm no-op, and P-00/P-10/P-11/P-14/P-18/P-19 plus
  `GRID-DECIMAL-R1C1-1M` passing;
  `w061-grid-scale-recursive-binary-r1c1-1m-001` evaluates a 1,000,000-row x 5-column
  recursive binary arithmetic shape with three dense input columns and two repeated R1C1 formula
  columns: `=RC[-3]+RC[-2]*RC[-1]` and `=(RC[-4]+RC[-3])*RC[-2]`. It prepares two templates
  and two compiled plans, records two formula-plan misses plus 1,999,998 hits, publishes
  5,000,000 packed dense numeric computed cells with zero sparse computed cells, samples
  precedence values 21/10500000/21000000 and parenthesized values 22/11000000/22000000, warms
  with zero visits, and passes P-00/P-10/P-11/P-14/P-18/P-19 plus
  `GRID-RECURSIVE-BINARY-R1C1-1M`;
  `w061-grid-scale-if-r1c1-1m-001` evaluates a 1,000,000-row x 2-column conditional clamp
  using `=IF(RC[-1]>0,RC[-1],0)`: one packed dense signed input column, 1,000,000 formula
  cells, one prepared template, one compiled IF plan, one formula-plan miss plus 999,999 hits,
  one compiled-plan miss, dense computed formula output (`computed_sparse_cells == 0`), sampled
  values 1/0/999999/0, zero-visit warm no-op, and P-00/P-10/P-11/P-14/P-18/P-19 plus
  `GRID-IF-R1C1-1M` passing;
  `w061-grid-scale-if-branch-r1c1-1m-001` evaluates a 1,000,000-row x 2-column conditional
  arithmetic-branch lane using `=IF(RC[-1]>0,RC[-1]*2,RC[-1]/2)`: one packed dense signed
  input column, 1,000,000 formula cells, one prepared template, one compiled IF plan with
  scalar branch expressions, one formula-plan miss plus 999,999 hits, one compiled-plan miss,
  dense computed formula output (`computed_dense_numeric_packed_cells == 2000000` and
  `computed_sparse_cells == 0`), sampled values 2/-250000/1999998/-500000, zero-visit warm
  no-op, and P-00/P-10/P-11/P-14/P-18/P-19 plus `GRID-IF-BRANCH-R1C1-1M` passing;
  `w061-grid-scale-nested-if-r1c1-1m-001` evaluates a 1,000,000-row x 3-column nested
  conditional lane using
  `=IF(RC[-2]>500000,IF(RC[-1]>0,RC[-2]*2,RC[-2]*3),IF(RC[-1]>0,RC[-2]+1,RC[-2]-1))`:
  one packed dense input region with 2,000,000 cells, 1,000,000 formula cells, one prepared
  template, one compiled IF plan with nested scalar IF branch expressions, one formula-plan
  miss plus 999,999 hits, one compiled-plan miss, 3,000,000 packed dense numeric computed
  cells, zero sparse computed cells, sampled values 0/500001/1500003/2000000, zero-visit warm
  no-op, and P-00/P-10/P-11/P-14/P-18/P-19 plus `GRID-NESTED-IF-R1C1-1M` passing;
  `w061-grid-scale-iferror-r1c1-1m-001` evaluates a 1,000,000-row x 3-column error-recovery
  lane using `=IFERROR(RC[-2]/RC[-1],0)`: one packed dense input region with 2,000,000 cells,
  1,000,000 formula cells, one prepared template, one compiled IFERROR plan, one formula-plan
  miss plus 999,999 hits, one compiled-plan miss, dense computed formula output
  (`computed_sparse_cells == 0`), sampled values 1/0/999999/0, zero-visit warm no-op, and
  P-00/P-10/P-11/P-14/P-18/P-19 plus `GRID-IFERROR-R1C1-1M` passing;
  `w061-grid-scale-comparison-r1c1-1m-001` evaluates a 1,000,000-row x 2-column logical output
  lane using `=RC[-1]>0`: one packed dense signed input column, 1,000,000 formula cells, one
  prepared template, one compiled comparison plan, one formula-plan miss plus 999,999 hits, one
  compiled-plan miss, dense computed formula output (`computed_sparse_cells == 0`), sampled
  logical values true/false/true/false, `computed_dense_cells == 2000000` with
  1,000,000 input cells numeric-packed and 1,000,000 formula output cells logical-packed,
  zero-visit warm no-op, and P-00/P-10/P-11/P-14/P-18/P-19 plus
  `GRID-COMPARISON-R1C1-1M` passing;
  `w061-grid-scale-comparison-expression-r1c1-1m-001` evaluates a 1,000,000-row x 3-column
  scalar-expression comparison lane using `=RC[-2]*2>RC[-1]+1`: one packed dense signed input
  region with 2,000,000 cells, 1,000,000 formula cells, one prepared template, one compiled
  comparison plan, one formula-plan miss plus 999,999 hits, one compiled-plan miss, dense
  logical formula output (`computed_dense_cells == 3000000`,
  `computed_dense_numeric_packed_cells == 2000000`,
  `computed_dense_logical_packed_cells == 1000000`, and `computed_sparse_cells == 0`),
  sampled logical values false/false/true/false, zero-visit warm no-op, and
  P-00/P-10/P-11/P-14/P-18/P-19 plus `GRID-COMPARISON-EXPRESSION-R1C1-1M` passing;
  `w061-grid-scale-comparison-iferror-r1c1-1m-001` evaluates a 1,000,000-row x 3-column
  nested-error-recovery comparison lane using `=IFERROR(RC[-2]/RC[-1],0)>0`: one packed
  dense input region with 2,000,000 cells, 1,000,000 formula cells, one prepared template,
  one compiled comparison plan with nested IFERROR scalar operand, one formula-plan miss plus
  999,999 hits, one compiled-plan miss, dense logical formula output
  (`computed_dense_cells == 3000000`, `computed_dense_numeric_packed_cells == 2000000`,
  `computed_dense_logical_packed_cells == 1000000`, and `computed_sparse_cells == 0`),
  sampled logical values true/false/true/false, zero-visit warm no-op, and
  P-00/P-10/P-11/P-14/P-18/P-19 plus `GRID-COMPARISON-IFERROR-R1C1-1M` passing;
  `w061-grid-scale-sum-row-r1c1-1m-001` evaluates a 1,000,000-row x 4-column row-range
  aggregate using `=SUM(RC[-3]:RC[-1])`: one packed dense input region with 3,000,000 cells,
  1,000,000 formula cells, one prepared template, one compiled plan, one formula-plan miss
  plus 999,999 hits, one compiled-plan miss, dense computed formula output
  (`computed_sparse_cells == 0`), sampled values 6/3000000/6000000, zero-visit warm no-op,
  and P-00/P-10/P-11/P-14/P-18/P-19 plus `GRID-SUM-ROW-R1C1-1M` passing;
  `w061-grid-scale-sumsq-row-r1c1-1m-001` evaluates a 1,000,000-row x 4-column row-range
  aggregate using `=SUMSQ(RC[-3]:RC[-1])`: one packed dense input region with 3,000,000 cells,
  1,000,000 formula cells, one prepared template, one compiled plan, one formula-plan miss
  plus 999,999 hits, one compiled-plan miss, dense numeric computed formula output
  (`computed_dense_cells == 4000000`, `computed_dense_numeric_packed_cells == 4000000`, and
  `computed_sparse_cells == 0`), sampled values 14/3500000000000/14000000000000, zero-visit
  warm no-op, and P-00/P-10/P-11/P-14/P-18/P-19 plus
  `GRID-SUMSQ-ROW-R1C1-1M` passing;
  `w061-grid-scale-count-row-r1c1-1m-001` evaluates a 1,000,000-row x 4-column row-range
  aggregate using `=COUNT(RC[-3]:RC[-1])`: one packed dense input region with 3,000,000 cells,
  1,000,000 formula cells, one prepared template, one compiled plan, one formula-plan miss
  plus 999,999 hits, one compiled-plan miss, dense computed formula output
  (`computed_sparse_cells == 0`), sampled values 3/3/3, zero-visit warm no-op, and
  P-00/P-10/P-11/P-14/P-18/P-19 plus `GRID-COUNT-ROW-R1C1-1M` passing;
  `w061-grid-scale-product-row-r1c1-1m-001` evaluates a 1,000,000-row x 4-column row-range
  aggregate using `=PRODUCT(RC[-3]:RC[-1])`: one packed dense input region with 3,000,000 cells,
  1,000,000 formula cells, one prepared template, one compiled plan, one formula-plan miss
  plus 999,999 hits, one compiled-plan miss, dense computed formula output
  (`computed_sparse_cells == 0`), sampled values 6/6/6, zero-visit warm no-op, and
  P-00/P-10/P-11/P-14/P-18/P-19 plus `GRID-PRODUCT-ROW-R1C1-1M` passing;
  `w061-grid-scale-average-row-r1c1-1m-001` evaluates a 1,000,000-row x 4-column row-range
  aggregate using `=AVERAGE(RC[-3]:RC[-1])`: one packed dense input region with 3,000,000
  cells, 1,000,000 formula cells, one prepared template, one compiled plan, one formula-plan
  miss plus 999,999 hits, one compiled-plan miss, dense computed formula output
  (`computed_sparse_cells == 0`), sampled values 2/1000000/2000000, zero-visit warm no-op,
  and P-00/P-10/P-11/P-14/P-18/P-19 plus `GRID-AVERAGE-ROW-R1C1-1M` passing;
  `w061-grid-scale-min-max-row-r1c1-1m-001` evaluates a 1,000,000-row x 5-column paired
  row-range aggregate using `=MIN(RC[-3]:RC[-1])` and `=MAX(RC[-4]:RC[-2])`: one packed
  dense input region with 3,000,000 cells, 2,000,000 formula cells, two prepared templates,
  two compiled plans, two formula-plan misses plus 1,999,998 hits, two compiled-plan misses,
  dense computed formula output (`computed_sparse_cells == 0`), sampled MIN values
  1/500000/1000000 and MAX values 3/1500000/3000000, zero-visit warm no-op, and
  P-00/P-10/P-11/P-14/P-18/P-19 plus `GRID-MIN-MAX-ROW-R1C1-1M` passing;
  `w061-grid-scale-sum-window-r1c1-1m-001` evaluates a 1,000,000-row x 2-column vertical
  sliding-window aggregate using `=SUM(R[-2]C[-1]:RC[-1])`: one packed dense input column,
  999,998 formula cells starting at row 3, one prepared template, one compiled plan, one
  formula-plan miss plus 999,997 hits, one compiled-plan miss, dense computed formula output
  (`computed_sparse_cells == 0`), sampled values 6/1499997/2999997, zero-visit warm no-op,
  and P-00/P-10/P-11/P-14/P-18/P-19 plus `GRID-SUM-WINDOW-R1C1-1M` passing;
  `w061-grid-scale-division-error-r1c1-1m-001` evaluates the same 1,000,000-row x 2-column
  shape with `=RC[-1]/0`: one packed dense input column, 1,000,000 formula cells, one prepared
  template, one compiled plan, one formula-plan miss plus 999,999 hits, one compiled-plan miss,
  dense computed formula output (`computed_sparse_cells == 0`), sampled first/middle/last
  formula values `Div0`, zero-visit warm no-op, and P-00/P-10/P-11/P-14/P-18/P-19 plus
  `GRID-DIVISION-ERROR-R1C1-1M` passing;
  `w061-grid-scale-division-error-propagation-r1c1-1m-001` evaluates a 1,000,000-row x
  3-column dense/repeated R1C1 error-propagation chain with col 2 `=RC[-1]/0` and col 3
  `=RC[-1]+1`: one packed dense input column, two repeated formula regions, 2,000,000 formula
  cells, two prepared templates, two compiled plans, two formula-plan misses plus 1,999,998
  hits, dense direct and propagated `Div0` outputs (`computed_sparse_cells == 0`), sampled
  first/middle/last propagated values `Div0`, zero-visit warm no-op, and
  P-00/P-10/P-11/P-14/P-18/P-19 plus `GRID-DIVISION-ERROR-PROPAGATION-R1C1-1M` passing;
  `w061-grid-scale-aggregate-error-r1c1-1m-001` evaluates a 1,000,000-row x 4-column
  dense/repeated R1C1 aggregate-error chain with col 2 `=RC[-1]/0`, col 3
  `=SUM(RC[-2]:RC[-1])`, and col 4 `=IFERROR(SUM(RC[-3]:RC[-2]),RC[-3])`: one packed dense
  input column, three repeated formula regions, 3,000,000 formula cells, three prepared
  templates, three compiled plans, three formula-plan misses plus 2,999,997 hits, 4,000,000
  dense computed cells, 2,000,000 numeric-packed cells, zero sparse cells, sampled aggregate
  `Div0` values plus recovered values 2/1000000/2000000, zero-visit warm no-op, and
  P-00/P-10/P-11/P-14/P-18/P-19 plus `GRID-AGGREGATE-ERROR-R1C1-1M` passing;
  `w061-grid-scale-text-function-r1c1-1m-001` evaluates a 1,000,000-row x 5-column
  dense/repeated R1C1 text-function lane with col 1 uniform dense text `RowGrid`, col 2
  `=LEN(RC[-1])`, col 3 `=LEFT(RC[-2],3)`, col 4 `=RIGHT(RC[-3],4)`, and col 5
  `=CONCAT(RC[-2],RC[-1])`: one compact dense text input column, four repeated formula regions,
  4,000,000 formula cells, four prepared templates, four compiled plans, four formula-plan
  misses plus 3,999,996 hits, 5,000,000 dense computed cells, 1,000,000 numeric-packed cells,
  zero logical-packed cells, zero sparse cells, sampled `RowGrid`/`7`/`Row`/`Grid`/`RowGrid`
  values, zero-visit warm no-op, and P-00/P-10/P-11/P-14/P-18/P-19 plus
  `GRID-TEXT-FUNCTION-R1C1-1M` passing;
  `w061-grid-scale-index-function-r1c1-1m-001` evaluates a 1,000,000-row x 6-column
  dense/repeated R1C1 INDEX lane with two dense input columns, col 3
  `=INDEX(RC[-2]:RC[-1],1,1)`, col 4 `=INDEX(RC[-3]:RC[-2],1,2)`, col 5
  `=INDEX(R1C1:RC1,ROW(),1)`, and col 6 `=INDEX(RC[-5]:RC[-4],2,1)`: two compact dense input
  regions, four repeated formula regions, 4,000,000 formula cells, four prepared templates,
  four compiled plans, four formula-plan misses plus 3,999,996 hits, 6,000,000 dense computed
  cells, 3,000,000 numeric-packed cells, zero logical-packed cells, zero sparse cells, sampled
  numeric/text/dynamic lookup values 10/Index/5000000/10000000, sampled out-of-range `Ref`
  output, zero-visit warm no-op, and P-00/P-10/P-11/P-14/P-18/P-19 plus
  `GRID-INDEX-FUNCTION-R1C1-1M` passing;
  `w061-grid-scale-match-function-r1c1-1m-001` evaluates a 1,000,000-row x 6-column
  dense/repeated exact MATCH / nested INDEX-MATCH lane with three dense numeric input columns,
  col 4 `=MATCH(RC[-2],RC[-3]:RC[-1],0)`, col 5
  `=INDEX(RC[-4]:RC[-2],1,MATCH(RC[-3],RC[-4]:RC[-2],0))`, and col 6
  `=MATCH(999999999,RC[-5]:RC[-3],0)`: one compact dense input region, three repeated
  formula regions, 3,000,000 formula cells, three prepared templates, three compiled plans,
  three formula-plan misses plus 2,999,997 hits, 6,000,000 dense computed cells, 5,000,000
  numeric-packed cells, zero sparse cells, sampled match/index/no-match values `2`/`5000001`/`NA`,
  zero-visit warm no-op, and P-00/P-10/P-11/P-14/P-18/P-19 plus
  `GRID-MATCH-FUNCTION-R1C1-1M` passing;
  `optimized_grid_compact_oxfml_recalc_evaluates_general_binary_r1c1_templates` proves the
  compiled plan is not just a list of exact variants: the optimized path compiles binary R1C1
  formulas with relative reference operands, finite numeric literal operands, recursive scalar
  arithmetic, ordinary precedence, and parenthesized subexpressions over `+`/`-`/`*` and `/`
  into dense numeric, direct `Div0`, or single-error propagated output;
  `optimized_grid_compact_oxfml_recalc_evaluates_r1c1_if_templates` proves numeric R1C1 IF
  templates with comparison conditions and scalar-expression result arms, and
  `optimized_grid_compact_oxfml_recalc_r1c1_if_propagates_condition_and_branch_errors` keeps
  condition errors and selected branch errors as dense `CalcValue` output;
  `optimized_grid_compact_oxfml_recalc_evaluates_r1c1_iferror_templates` proves IFERROR over a
  bounded binary expression, a fallback binary expression, and non-evaluation of an erroring
  fallback when the first expression is clean;
  `optimized_grid_compact_oxfml_recalc_evaluates_r1c1_comparison_templates` and
  `optimized_grid_compact_oxfml_recalc_r1c1_comparison_propagates_operand_errors` prove direct
  comparison formulas publish dense logical output, support scalar-expression operands including
  nested IFERROR, and propagate operand errors;
  `optimized_grid_compact_oxfml_recalc_r1c1_range_aggregate_propagates_errors` proves R1C1
  range aggregates propagate a dense formula-produced worksheet error and can be recovered by
  IFERROR without sparse formula output;
  `optimized_grid_compact_oxfml_recalc_r1c1_text_functions_stay_dense` proves LEN/LEFT/RIGHT/CONCAT
  over R1C1 reference arguments publish dense text/numeric `CalcValue` output and can chain
  references through prior repeated-formula output regions;
  `optimized_grid_compact_oxfml_recalc_r1c1_index_function_stays_dense` proves bounded
  positive-index INDEX over R1C1 ranges publishes dense numeric, text, and `#REF!` `CalcValue`
  output and can use a dynamic `ROW()` selector without enumerating the referenced range;
  `optimized_grid_compact_oxfml_recalc_r1c1_match_function_stays_dense` proves exact
  `MATCH(...,0)` over finite one-dimensional R1C1 ranges, nested `INDEX(...,MATCH(...),...)`,
  and first-class `#N/A` output as dense formula-output cells;
  `optimized_grid_compact_oxfml_recalc_evaluates_absolute_r1c1_templates` proves the compiled
  plan resolves absolute axes in point and range references through `R1C1` and
  `SUM(R1C1:R1C3)` while preserving repeated-template counters;
  `optimized_grid_compact_oxfml_recalc_evaluates_r1c1_reference_function_templates` proves
  `ROW`, `COLUMN`, `ROWS`, and `COLUMNS` over current cell, R1C1 references, finite R1C1
  ranges, and arithmetic composition without value dereference;
  `optimized_grid_compact_oxfml_recalc_evaluates_r1c1_sum_range_templates`,
  `optimized_grid_compact_oxfml_recalc_evaluates_r1c1_sumsq_range_templates`,
  `optimized_grid_compact_oxfml_recalc_evaluates_r1c1_count_range_templates`,
  `optimized_grid_compact_oxfml_recalc_evaluates_r1c1_product_range_templates`, and
  `optimized_grid_compact_oxfml_recalc_evaluates_r1c1_average_range_templates` prove the
  separate optimized range-function floor for `SUM(RC[-3]:RC[-1])`,
  `SUMSQ(RC[-3]:RC[-1])`,
  `COUNT(RC[-3]:RC[-1])`,
  `PRODUCT(RC[-3]:RC[-1])`, and
  `AVERAGE(RC[-3]:RC[-1])` over dense row inputs, while
  `optimized_grid_compact_oxfml_recalc_evaluates_r1c1_min_max_range_templates` proves
  `MIN(RC[-3]:RC[-1])` and `MAX(RC[-3]:RC[-1])` through the same aggregate-plan class, and
  `w061-grid-scale-sum-window-r1c1-1m-001` extends the retained evidence to a vertical
  multi-row relative window over dense inputs;
  `w061-grid-scale-plan-cache-rounds-1m-001` runs that dense/repeated-R1C1 shape through three
  dirty compact OxFml recalc rounds sharing one `GridOptimizedFormulaPlanCache`: the first
  2,000,000-formula round has one miss, the next two rounds have zero misses, and the run
  records 6,000,000 lookups, 5,999,999 hits, one miss, one cached template, one cached compiled
  R1C1 plan object, one first-round compiled-plan miss, zero later compiled-plan misses, two
  later compiled-plan hits, and P-14 plus `GRID-PLAN-CACHE-ROUNDS-1M` passing;
  `optimized_grid_formula_plan_cache_recompiles_stale_fingerprint_and_prunes_unused_plans`
  covers the first lifecycle floor for that cache by forcing a stale normal-form key with changed
  R1C1 source to recompile, then replacing the formula region with values and pruning the cached
  template/compiled plan;
  `w061-grid-scale-insert-storm-1m-001` applies six row insert/delete edits over the same
  compact dense/repeated-formula region shape with 9,999,920 authored cells before edits,
  touches 42 compact region metadata records versus a 59,999,520-cell naive rewrite floor,
  preserves zero sparse point materialization, and passes P-10/P-17 plus
  `GRID-INSERT-STORM-1M`.
- `w061-grid-scale-publication-delta-1m-001` compares two 1,000,000-row optimized valuations
  after one dense input cell changes and its repeated-R1C1 dependent output changes. The
  revision-insensitive publication delta report records two changed dense region entries,
  `publication_entries_total == 2`, zero sparse/spill delta entries, and
  `publication_entry_ratio_micros == 1` against a 2,000,000-cell current-computed publication
  floor. This is the first P-22 compact publication-entry floor; production publication
  lifecycle remains open.
- `w061-grid-scale-range-invalidation-1m-001` installs a 1,000,000-row finite range dependency
  as one compressed reverse edge with zero scalar edges for the range, keeps only the ordinary
  downstream cell edge in the scalar graph, and proves that an edit seed at row 500,000 dirties
  the range formula plus its downstream dependent. This is the first P-13 finite-range
  compressed reverse-edge floor.
- `w061-grid-scale-range-query-1m-001` installs 1,000 finite compressed range dependencies over
  1,000,000 rows and uses the compressed range block index to check only 2 indexed candidates
  for a seed at row 500,501, matching one dependent and its downstream chain. This is the first
  P-12 interval-index query floor; production invalidation equivalence remains open.
- `w061-grid-scale-sum-pyramid-1m-001` installs a 1,000,000-row compressed aggregation pyramid:
  1,111 compressed range edges cover 3,700,111 expanded support cells with zero range
  scalarization, one downstream scalar edge, `indexed_candidate_sum == 20` across the selected
  leaf-to-root chain, and an exact 6-cell dirty closure. This is the first P-12/P-13
  sum-pyramid floor.
- `w061-grid-scale-dirty-rect-1m-001` installs 1,000 compressed range consumers, 1,000 scalar
  consumers, and one downstream chain over a 1,000,000-row sheet. An 11-cell dirty rectangle
  checks 3 indexed candidates across 2,002 total edges, matches one range consumer and one
  scalar consumer, and closes exactly those two dependents plus the downstream cell. This is
  the first P-12 dirty-rectangle block-index floor.
- `w061-grid-scale-hide-storm-1m-001` installs 1,000 hidden-sensitive row-band dependencies over
  1,000,000 rows and uses the AxisState visibility block index to check only 2 indexed
  candidates for a hidden-row seed at row 500,501, matching one aggregate formula and its
  downstream chain. This is the first P-24 hidden-toggle dirty-closure query floor; broader
  hide/filter/outline provenance and production invalidation equivalence remain open.
- `w061-grid-scale-spill-anchor-1m-001` resolves a 1,000,000-row `A1#` extent through the grid
  provider with one spill-ledger probe, zero extent cells scanned for the ledger lookup, and
  three provider value entries scanned/returned. This is the first P-25 spill-ledger probe
  floor; broad spill-storm extent churn and epoch-precision gates remain open.
- `w061-grid-scale-filter-spill-1m-006` clears a 1,000,000-row old spill extent through the
  optimized sparse-value index: 3 indexed candidates are removed from 1003 sparse values, 1000
  unrelated sparse values survive the clear, and the smaller re-spill sample touches 6 sparse
  cells total versus 5,000,000 grid cells. The same run evaluates a real
  `FILTER(A1:B1000000,C1:C1000000)` formula over dense value/include regions, publishes a
  500,000-row by 2-column spill extent with 999,999 ghosts, keeps `computed_sparse_cells == 0`,
  samples FILTER row pairs 101/102, 25000001/25000002, and 50000001/50000002 with the first
  vacated row empty, and commits one spill fact plus one epoch anchor back to optimized sheet
  state (`filter_formula_spill_commit_committed_fact_entries == 1`,
  `filter_formula_spill_commit_anchors_added == 1`). It also evaluates a column-mask FILTER
  over a 999,999-row by 3-column source with a horizontal include row, publishing a 999,999-row
  by 2-column dense output with zero sparse computed cells and committing one spill fact plus
  one epoch anchor (`column_filter_spill_extent_declared_cells == 1999998`,
  `column_filter_spill_commit_committed_fact_entries == 1`). The row-mask FILTER leg then
  applies one sparse mask override and runs a second committed optimized recalc: the dense output
  shrinks to 499,999 rows by 2 columns (`filter_lifecycle_spill_extent_declared_cells == 999998`),
  clears the vacated ghost row, reports one extent-change anchor and one value-change anchor,
  and advances the committed spill epoch to 2. This is the first combined P-23 old-spill indexed
  clear plus dense row-mask/column-mask FILTER committed-publication and FILTER lifecycle floor;
  the broader FILTER matrix remains open.
- `w061-grid-scale-sequence-spill-1m-002` publishes `SEQUENCE(1000000)` as one optimized dense
  computed region with 1,000,000 packed numeric cells, zero sparse computed cells, and sampled
  values 1/500000/1000000. The same run uses
  `GridOptimizedSheet::recalculate_mark_all_dirty_compact_with_oxfml_and_commit_spill_publication`
  and records `spill_commit_committed_fact_entries == 1`,
  `spill_commit_anchors_added == 1`, `spill_commit_current_epoch_anchors == 1`, and
  `sheet_committed_spill_fact_entries == 1`. This is the first P-23 dense dynamic-array
  publication plus optimized committed spill-state floor.
- `w061-grid-scale-spill-blockage-1m-001` probes a 1,000,000-row intended spill extent through
  compact blocker candidates: an empty extent performs zero compact blocker probes, while a
  sparse blocker at row 1,000,000 is found with one compact probe instead of scanning the
  1,000,000-cell extent. This is the first P-26 spill-blockage probe floor; broad spill
  arbitration and occupied-block indexing across all blocker families remain open.
- `w061-grid-scale-spill-epoch-1m-003` compares spill anchor snapshots for a 1,000,000-row
  `A1#` extent. Unchanged snapshots and unrelated anchor value churn dirty zero consumers, while
  A1 value-epoch and extent-epoch changes dirty the `A1#` consumer plus its downstream chain.
  The current runner now builds those snapshots through `GridSpillEpochLedger`, whose compact
  unit coverage (`grid_calc_ref_spill_epoch_ledger_preserves_and_advances_anchor_epochs`,
  `optimized_grid_spill_epoch_ledger_preserves_and_advances_anchor_epochs`) preserves epochs
  for unchanged formula-owned spills and advances them for value, extent, and blocked-state
  changes in both GridCalc-Ref and the optimized valuation.
  `GridOptimizedSheet::commit_spill_publication_from_valuation` now commits optimized valuation
  spill facts/fingerprints back into sheet state with same-grid validation and ledger update
  counters (`optimized_spill_commit_first_added == 1`,
  `optimized_spill_commit_second_preserved == 1`,
  `optimized_spill_commit_extent_changed == 1`; unit
  `optimized_grid_rejects_spill_publication_commit_from_different_grid`). This is the first
  P-27 spill-epoch ledger plus optimized spill-state commit floor. The committed SEQUENCE and
  FILTER recalc paths above exercise the first end-to-end publication commit API for
  constant-shape arrays, one-shot value-dependent arrays, and a retained 1M FILTER shape-change
  lifecycle; broader publication lifecycle cases and spill-storm arbitration remain open.
- `w061-grid-scale-aggregate-context-1m-001` reports the grid host-info provider plan for a
  1,000,000-row `SUBTOTAL`-style aggregate context: two explicit AxisState rows plus three
  default row runs produce five row-context probes, while the current OxFunc host-info seam still
  requires a 1,000,000-cell `AggregateReferenceContext` packet. This is the first P-28
  provider-side row-run floor; a true run-compressed host-info packet remains a cross-repo seam
  follow-up.
- `w061-grid-scale-tile-stream-64k-001` reports a tile snapshot over a 320 x 200 visible rect
  inside a 1,000,000-row x 320-column model. The optimized valuation resolves only the
  subscribed 64,000 cells, visits 64,000 dense cells, visits zero unrelated sparse cells despite
  1,000 sparse values outside the tile, intersects one compact region, and estimates a
  1,536,159-byte frame (about 24.003 bytes per subscribed cell), passing P-15 and
  `GRID-TILE-STREAM-64K`. This is the first optimized tile readout floor; production host tile
  protocol and viewport scheduling remain open.
- `w061-grid-scale-viewport-64k-of-1m-001` reports the first optimized visible-first recalc
  floor. A 64,000-cell visible formula column in a 1,000,000-row x 10-column dense/repeated-R1C1
  model is cleaned by projecting only columns 8..10 for the visible rows: 64,000 dense input
  cells and 128,000 repeated formula cells. The run evaluates 192,000 cells before the visible
  rect is complete, records a 10,000,000-cell full occupied floor, publishes two dense computed
  regions with zero sparse computed cells, and reads the 64,000 visible cells with zero sparse
  visits, passing P-16 and `GRID-VIEWPORT-64K`. Production viewport scheduling remains open.
- `w061-grid-scale-cow-retention-1m-001` reports the first optimized retained-root COW floor.
  Initial plus six edited roots over the 1,000,000-row dense/repeated-R1C1 shape share one dense
  numeric payload across row insert/delete splits. The COW report records seven retained roots,
  one unique dense payload, 64,012,633 retained bytes versus a 448,009,561-byte full-snapshot
  floor, a 14.288% retained-to-naive ratio, 42 compact metadata touches, 56 retained compact
  regions, zero sparse materialization, and zero blank-cell bytes, passing P-21 and
  `GRID-COW-RETENTION-1M`. Production retention GC/lifecycle policy remains open.
- `StrictExcelGridReferenceProfile::transform_reference` is the first profile-level structural
  edit algebra slice. The `excel-grid-structural-edit.v1` payload carries row/column
  insert/delete plus formula-anchor context; the profile transforms point, finite area,
  whole-row, and whole-column reference payloads, including deleted-target `#REF!` outcomes and
  R1C1-relative normal-form preservation when the formula anchor moves with the edit.
- `GridCalcRefSheet::apply_axis_edit` now consumes that profile transform for authored formula
  cells: moved formulas rewrite profile-owned grid reference source spans, preserve surviving
  R1C1 shape, rebind at the new anchor for a fresh normal-form key, and report formula-cell and
  formula-reference transform counters on `GridStructuralEditReport`.
- `GridOptimizedSheet::apply_axis_edit` covers the matching compact authored-storage slice:
  sparse points move/drop with formula rewrites, dense value regions split around inserted
  blank rows/columns, and repeated formula regions split while preserving R1C1 template
  evaluation against dense regions.
- `GridInvalidationRef::apply_axis_edit` is the first structural dirty-closure oracle slice:
  it preserves semantic cell/range/dynamic dependency descriptors, transforms dependent cells
  and scalar dependencies under row/column insert/delete, expands/shrinks finite ranges before
  scalarization, and emits structural edit reports in the seed corpus.
- Whole-row/whole-column value dependencies now have an axis-indexed dirty-closure lane:
  `GridInvalidation-Ref` records `AxisValue(row|column, first..last)` dependencies without
  scalarizing all cells in the referenced rows/columns. Ordinary cell edit seeds consult those
  row/column indexes during closure, and structural edits shift/shrink/expand the axis ranges.
- AxisState visibility is now a first dirty-closure dependency family: hidden-sensitive
  formulas can install row/column visibility ranges, `dirty_closure_for_axis_visibility` closes
  through ordinary scalar dependents, and optimized formula evaluation supplies the same
  `GridHostInfoProvider` row context as the reference path. Visibility dependencies now use the
  same block-indexed query shape as compressed finite ranges, with the retained
  `w061-grid-scale-hide-storm-1m-001` floor proving 2 indexed candidates rather than scanning
  all 1,000 visibility edges for a 1M-row hidden-toggle seed. The provider also exposes
  `GridAggregateContextQueryReport`, which plans aggregate row context by AxisState row runs; the
  retained `w061-grid-scale-aggregate-context-1m-001` floor probes five row-context runs for a
  1M-row aggregate reference while separately recording the current per-cell OxFunc packet
  expansion.
- Optimized spill blockage now has a compact probe report over sparse points, dense-value
  regions, repeated-formula regions, merged regions, feature-rendered regions, and active spill
  facts. `GridOptimizedSheet::optimized_spill_blockage_probe_report` exposes the deterministic
  P-26 counters, and the optimized publication path uses the same probe for its runtime blocked
  decision instead of iterating empty cells in the intended extent.
- Optimized formula-output clearing now exposes `GridOptimizedSpillClearReport` and uses the
  valuation sparse row/column index to find old spill outputs inside the prior extent, and it
  removes old dense computed spill regions in the same clear step. The retained
  `w061-grid-scale-filter-spill-1m-006` floor proves old sparse-output cleanup does not scan
  unrelated sparse values or blank sheet capacity and commits the value-dependent FILTER
  publication to sheet spill state, while the same retained run proves a later committed FILTER
  recalc can shrink dense output and clear vacated ghost cells. Unit coverage also proves dense
  old spill regions are cleared. The optimized provider also has a budgeted large-dense
  dereference path: references fully covered by dense computed regions, including sparse edited
  overlays over the dense cover, can materialize within the caller's explicit budget for
  value-consuming functions such as FILTER, while sparse large references continue to reject
  eager dereference and stay on the sparse-enumeration path.
- Same-sheet defined names now have a first value/invalidation floor: GridCalc-Ref and
  GridOptimizedSheet both carry a defined-name map into `GridReferenceSystemProvider`.
  OxFml binds names as symbolic `excel.grid.v1` references, while the provider resolves
  the runtime namespace for both `SUM(InputRange)` and `SUM(INDIRECT("InputRange"))`.
  GridInvalidation-Ref records finite `Name(name, extent)` dependencies, scalarizes the
  current extent for ordinary value edits, exposes name-key dirty closure for namespace
  changes, and transforms the finite extent under row/column insert/delete. The grid
  machines transform same-sheet defined-name rects under the same row/column edit lane.
  Defined-name lifecycle APIs now cover same-sheet rename and delete in GridCalc-Ref and
  GridOptimizedSheet: direct formulas are rewritten on rename (`InputRange` -> `DataRange`)
  and delete (`DataRange` -> `#NAME?`), while `INDIRECT("InputRange")` text is preserved and
  resolves through runtime text routing. GridInvalidation-Ref retargets/drops finite name
  dependencies for rename/delete lifecycle operations and returns the namespace-key dirty
  closure; the seed corpus emits these checks in the `namespace_lifecycle` artifact lane.
  Namespace versioning and structured/table name interactions remain open.
- Same-sheet table overlays now have a first structured-reference value/invalidation floor:
  GridCalc-Ref and GridOptimizedSheet carry bounded `GridTableOverlay` records, expose them as
  feature-rendered regions, publish OxFml table-context descriptors, and register explicit
  `Table`, `Table[#All]`, `Table[#Data]`, `Table[#Headers]`, `Table[#Totals]`,
  `Table[Column]`, section-qualified column references, contiguous column ranges,
  non-contiguous section unions over table columns, escaped explicit column names, caller-local
  `[Column]` references, and caller-local `[@Column]` references with the grid
  `ReferenceSystemProvider`. OxFml parses native structured-reference syntax and dispatches
  through the generic `bind_structured_reference` profile hook; the strict grid profile emits a
  symbolic structured-reference payload, and OxCalc's provider owns dereference to concrete
  ranges. `SUM(Table1[Amount])`, repeated in-table `=SUM([Amount])` and `=[@Amount]*2` formulas,
  `SUM(Table1[[#Data],[Amount]:[Tax]])`,
  `SUM(Table1[[#Headers],[#Totals],[Amount]:[Tax]])`, escaped-column formulas, and corresponding
  `INDIRECT(...)` text forms now evaluate through the same reference/optimized engine paths.
  GridInvalidation-Ref records finite
  `Table(table, extent)` dependencies and current-row scalar cell dependencies, scalarizes the
  current extent for ordinary value edits, exposes table-key dirty closure for namespace changes,
  and transforms the finite extent under row/column insert/delete. First table-overlay lifecycle
  APIs now cover same-sheet resize, rename, and delete in GridCalc-Ref and GridOptimizedSheet:
  stale table feature-rendered-region claims are removed, explicit structured-reference formulas
  are rewritten on table rename (`Table1[Column]` -> `Sales[Column]`) and table delete
  (`Sales[Column]` -> `#REF!`), renamed tables resolve through the provider under the new name,
  and deleted overlays stop contributing table metadata. GridInvalidation-Ref retargets/drops
  finite table dependencies for rename/delete lifecycle operations, rebuilds scalar edges for
  table resize extents, and returns the namespace-key dirty closure; the seed corpus emits
  these checks in the `namespace_lifecycle` artifact lane. This floor intentionally excludes
  table/name collision precedence for omitted references, `INDIRECT("Table1[...]")` text
  rewrites, resize-driven formula source expansion/shrink semantics, and full table namespace
  versioning.
- Committed spill facts are now a first ledger-backed reference family: GridCalc-Ref and
  GridOptimizedSheet both expose anchor-keyed extents to the grid `ReferenceSystemProvider`,
  `A1#` consumers evaluate through the same OxFml/OxFunc path in both engines, and
  GridInvalidation-Ref records `SpillFact(anchor)` dependencies so extent changes dirty
  consumers separately from current member-cell edits.
- Blocked spill anchors now have a separate blocker-watch dependency family:
  GridInvalidation-Ref records `SpillBlocker(extent)` dependencies and scalarizes the finite
  watched extent into a distinct reverse index. A merged/table/feature blocker edit inside that
  extent dirties the anchor chain without being conflated with ordinary value edits or
  `A1#` spill-shape consumers.
- Dynamic spill placement has a bounded executable floor: array-valued formulas publish anchor
  and ghost computed cells in GridCalc-Ref and as dense optimized computed regions in
  GridOptimizedSheet for successful array payloads; authored-cell, merged-region, table-overlay,
  and boundary overflow yield anchor `#SPILL!` plus a blocked spill fact; a later dynamic-array
  anchor inside an earlier blocked formula-owned spill extent also yields `#SPILL!`, giving the
  first mutual-anchor blockage floor without claiming full spill arbitration. Recalc reports expose
  published/blocked spill-fact and ghost-cell counters. Formula-owned spill-ledger changes
  trigger bounded repair passes for spill-anchor consumers, with `spill_repair_passes`,
  `spill_repair_formula_evaluations`, and `spill_repair_converged` emitted separately from
  primary `P-00` evaluation counters.

This slice claims only the seed-corpus dirty-closure checks declared in those scenarios,
including the small structural edit checks, whole-row/whole-column axis-value closure, finite
defined-name/table closure, namespace lifecycle closure, and merged-region blocker-watch
closure emitted by the run artifact. It does not yet claim optimized production invalidation
equivalence, broad mutual/body-overlap spill arbitration, full namespace versioning,
Excel-pinned spill ordering, or geometry-coupled opaque-part handling.

## 5. Initial corpus families

- bounds and `#REF!` translation;
- R1C1 normal-form fill/copy equivalence;
- template materialization and punch-through overrides;
- insert/delete over formulas, ranges, tables, and merged regions;
- whole-row/whole-column formulas and axis-indexed dirty closure;
- defined-name value resolution and `INDIRECT("Name")` text routing;
- explicit table structured-reference value resolution, section-qualified/escaped/multi-area columns, and
  `INDIRECT("Table[Column]")` text routing;
- spill blockage, clearance, contraction, mutual blockage, `A1#`, hidden-row placement;
- hidden/manual/filter/outline visibility-sensitive aggregates;
- pivot-like feature-rendered-region edit refusals and needs-refresh flags for structural edits.

## 6. Implementation sequence

1. Build the BTreeMap sheet state and sampled readout.
2. Add GridInvalidation-Ref as an expanded scalar dependency oracle for dirty-closure checks.
3. Add formula evaluation through existing OxFml/OxFunc leaves.
4. Add bounds/reference adjustment and template materialization scenarios.
5. Add GridOptimizedSheet authored storage for sparse points, dense value regions, and repeated
   formula regions, with compact valuation/readout, optimized provider-backed formula evaluation,
   bounded projection to GridCalc-Ref, the in-memory `reference|optimized|both` harness, and the
   seed corpus runner over that harness.
6. Add spill fixpoint and visibility AxisState floors.
7. Wire `--engine reference|optimized|both` into the existing scale/differential runner.
