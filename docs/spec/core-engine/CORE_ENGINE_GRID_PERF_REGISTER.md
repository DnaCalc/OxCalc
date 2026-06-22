# CORE_ENGINE_GRID_PERF_REGISTER

Status: **Promoted active planning register** (2026-06-13). Canonical OxCalc register for grid-lane performance claims and retained counter gates. Companion to the Invariant Register in CORE_ENGINE_GRID_REFINEMENT_AND_EQUIVALENCE.md.

## 1. Doctrine

1. **Every optimization ships as a permanent pair.** The reference twin (GridCalc-Ref, the
   scalarizer lowering, eager renumbering, per-cell reverse edges, per-cell prepare, map-based
   storage) is product code selected at runtime (`--engine reference|optimized|both`), never
   scaffolding. A differential run is one process.
2. **Merge rule:** no optimization lands without (a) an Invariant Register row (equivalence
   oracle, expand-and-compare against its twin) and (b) a row here with a falsifiable claim.
3. **Gates are deterministic counters, never wall-clock.** Bytes/cell, prepare count, cells
   evaluated, edges visited, slots visited, bytes/frame. This reconciles per-PR gating with
   DnaTreeCalc TECHNICAL.md §7.6 ("no clock-time success gates") and the documented ±25%
   wall variance. Wall-clock is recorded evidence in retained baselines only, always with
   ref/optimized ratios alongside absolutes.
4. **Counter fidelity:** every counter is asserted against a closed-form expectation on at
   least one workload, so a silently broken counter fails validation, not just gating.
5. **The reference twin is gated too:** naive in structure, not constants — it shares the leaf
   evaluation stack (OxFml/OxFunc) with the optimized engine and differs only in machinery.
   Its own row (P-00) keeps it usable as an oracle.

## 2. Row schema

| Field | Content |
|---|---|
| id / invariant | `P-xx` ↔ `I-xx` cross-link (mandatory both ways) |
| claim | complexity + constant, falsifiable (e.g. "≤17 B/cell on boring-1Mx10") |
| ref twin | which paired implementation expands/checks it |
| workload(s) | named ids from §5 |
| measured | ref number, optimized number, ratio, retained run id |
| budget | counted regression bound (never wall-clock) |
| status | claimed → measured → **bound** (counter gate live) → retired |

## 3. Seeded rows — calc-perf round 1 (tree lane, merged to main)

> To absorb verbatim from `CORE_ENGINE_HOST_WORKER_PASSIVITY_SPIKE.md` (fix table and
> measured numbers) on owner confirmation of the one-register decision. Skeleton:

| id | claim (summary) | evidence | status |
|---|---|---|---|
| P-01 | edge-cache warm-path cost fix (round-1 fix 1, commit `1955c8d`) | spike doc fix table | measured |
| P-02 | diagnostic-seed scaling fix (round-1 fix 2, commit `6a2cca0`) | spike doc fix table | measured |
| P-03 | consumer clone/retention fix (round-1 fix 3, commit `aa8eb26`) | spike doc fix table | measured |
| P-04 | layer-snapshot digest fix — warm-recalc OOM root cause (commit `64e144f`) | spike doc + OOM analysis | measured |

Round-2 residuals (incl. `CandidatePublication` growth and w056 O(n²) diagnostics) enter as
open rows when round 2 is scoped; P-22 below sidesteps the publication residual structurally
for the grid lane.

## 4. Open rows — grid lane catalogue

Invariant cross-links (I-xx) pend the equivalence doc; the proposed sets for spill
(I-SP1..I-SP5: bodies never authored; ledger ⇔ computed support; arbitration determinism;
quiesced no-edit extent stability; scalarizer equivalence) and visibility (I-VIS-1..3: exact
toggle closure; toggle storms change no insensitive value; manual/filter provenance
separation) were distilled from the archived DnaTreeCalc grid recon notes and are now owned by `CORE_ENGINE_GRID_REFINEMENT_AND_EQUIVALENCE.md`.

| id | claim | workload(s) | counter (vs ref twin) | status |
|---|---|---|---|---|
| P-00 | GridCalc-Ref evaluates each occupied cell exactly once in the primary recalc phase; spill-repair revisits are separate phase counters and excluded from the primary exactly-once assertion | all | primary_cells_evaluated == occupied; spill_repair_formula_evaluations reported by pass | measured (seed smoke) |
| P-10 | blocks: ≤17 B/cell dense, ≤85 B/cell adversarial singletons, blank cells = 0 bytes | boring-1Mx10, zig-zag-1M, full-column-1M, text-function-r1c1-1M, index-function-r1c1-1M, match-function-r1c1-1M, vlookup-function-r1c1-1M | bytes-by-layer vs map-of-CalcValue (>200 B/cell) | measured (boring-1Mx10 + full-column-1M + zig-zag-1M + dense/repeated smoke + uniform text-function lane + INDEX/MATCH/VLOOKUP lookup lanes + 1M sparse singleton byte leg) |
| P-11 | template prepare-once: prepare_count == distinct templates, not cells | fill-down-1M, pascal-r1c1-1M, direct-r1c1-1M, unary-r1c1-1M, argument-aggregate-r1c1-1M, math-function-r1c1-1M, mod-function-r1c1-1M, rounding-function-r1c1-1M, integer-function-r1c1-1M, log-function-r1c1-1M, trig-function-r1c1-1M, angle-function-r1c1-1M, reference-function-r1c1-1M, logical-function-r1c1-1M, if-logical-r1c1-1M, two-left-r1c1-1M, absolute-r1c1-1M, division-r1c1-1M, decimal-r1c1-1M, recursive-binary-r1c1-1M, if-r1c1-1M, if-branch-r1c1-1M, nested-if-r1c1-1M, iferror-r1c1-1M, comparison-r1c1-1M, comparison-expression-r1c1-1M, comparison-iferror-r1c1-1M, sum-row-r1c1-1M, sumsq-row-r1c1-1M, count-row-r1c1-1M, product-row-r1c1-1M, average-row-r1c1-1M, min-max-row-r1c1-1M, sum-window-r1c1-1M, division-error-r1c1-1M, division-error-propagation-r1c1-1M, aggregate-error-r1c1-1M, text-function-r1c1-1M, index-function-r1c1-1M, match-function-r1c1-1M, vlookup-function-r1c1-1M, enron-mix | prepare counter vs per-cell prepare | measured (seed + scale smoke + fill-down-1M R1C1 + pascal-r1c1-1M + boring-1Mx10 + direct-r1c1-1M + unary-r1c1-1M + argument-aggregate-r1c1-1M + math-function-r1c1-1M + mod-function-r1c1-1M + rounding-function-r1c1-1M + integer-function-r1c1-1M + log-function-r1c1-1M + trig-function-r1c1-1M + angle-function-r1c1-1M + reference-function-r1c1-1M + logical-function-r1c1-1M + if-logical-r1c1-1M + two-left-r1c1-1M + absolute-r1c1-1M + division-r1c1-1M + decimal-r1c1-1M + recursive-binary-r1c1-1M + if-r1c1-1M + if-branch-r1c1-1M + nested-if-r1c1-1M + iferror-r1c1-1M + comparison-r1c1-1M + comparison-expression-r1c1-1M + comparison-iferror-r1c1-1M + sum-row-r1c1-1M + sumsq-row-r1c1-1M + count-row-r1c1-1M + product-row-r1c1-1M + average-row-r1c1-1M + min-max-row-r1c1-1M + sum-window-r1c1-1M + division-error-r1c1-1M + division-error-propagation-r1c1-1M + aggregate-error-r1c1-1M + text-function-r1c1-1M + index-function-r1c1-1M + match-function-r1c1-1M + vlookup-function-r1c1-1M) |
| P-12 | rect propagation + interval index: invalidation O(dirty-rects·log n + consumers) | range-query-1M, dirty-rect-1M, edit-storm, sum-pyramid-N | indexed candidates + seeds vs full compressed-edge scan | measured (compressed range, sum-pyramid, and dirty-rectangle block-index floors; production invalidation equivalence still open) |
| P-13 | compressed reverse edges (FAP/TACO-style): support bytes O(regions); queries ≡ expanded graph | range-invalidation-1M, fill-down-1M, sum-pyramid-N | graph bytes + expand-and-compare | measured (finite-range and sum-pyramid compressed reverse-edge floors; broad interval-query asymptotics still open) |
| P-14 | primary-pass formula plan cache hit rate >= (formula cells - templates)/formula cells; persistent dirty-round cache misses only on first encounter | repeated-r1c1, fill-down-1M, pascal-r1c1-1M, boring-1Mx10, direct-r1c1-1M, unary-r1c1-1M, argument-aggregate-r1c1-1M, math-function-r1c1-1M, mod-function-r1c1-1M, rounding-function-r1c1-1M, integer-function-r1c1-1M, log-function-r1c1-1M, trig-function-r1c1-1M, angle-function-r1c1-1M, reference-function-r1c1-1M, logical-function-r1c1-1M, if-logical-r1c1-1M, two-left-r1c1-1M, absolute-r1c1-1M, division-r1c1-1M, decimal-r1c1-1M, recursive-binary-r1c1-1M, if-r1c1-1M, if-branch-r1c1-1M, nested-if-r1c1-1M, iferror-r1c1-1M, comparison-r1c1-1M, comparison-expression-r1c1-1M, comparison-iferror-r1c1-1M, sum-row-r1c1-1M, sumsq-row-r1c1-1M, count-row-r1c1-1M, product-row-r1c1-1M, average-row-r1c1-1M, min-max-row-r1c1-1M, sum-window-r1c1-1M, division-error-r1c1-1M, division-error-propagation-r1c1-1M, aggregate-error-r1c1-1M, text-function-r1c1-1M, index-function-r1c1-1M, match-function-r1c1-1M, vlookup-function-r1c1-1M, plan-cache-rounds-1M | hit/miss counters plus compiled-plan cache counters | measured (template lookup floor, direct scalar R1C1, unary-minus R1C1, aggregate argument-list R1C1, scalar math-function R1C1, MOD scalar-function R1C1, ROUND-family scalar-function R1C1, INT/TRUNC scalar-function R1C1, EXP/LN/LOG scalar-function R1C1, SIN/COS/TAN scalar-function R1C1, RADIANS/DEGREES/PI scalar-function R1C1, ROW/COLUMN reference-function R1C1, text-function R1C1 over reference arguments, bounded INDEX, exact MATCH, and exact VLOOKUP over R1C1 ranges, logical-function R1C1, IF logical-condition R1C1, bounded recursive binary R1C1 with precedence/parentheses, comparison R1C1 with operands, scalar expressions, and nested IFERROR, numeric IF/IFERROR R1C1, IF scalar branch expressions including nested scalar IF, R1C1 SUM/SUMSQ/COUNT/PRODUCT/AVERAGE/MIN/MAX-range compiled-plan floors, aggregate-error/IFERROR range floor, stale-source fingerprint recompile, and prune-to-active-template floor; broader production eviction policy/version lifecycle still open) |
| P-15 | tile streaming: frame bytes ≤64·subscribed-cells for numeric tile readout, independent of model size and unrelated sparse changes | tile-stream-64K, doom-320x200 | estimated frame bytes + dense/sparse visited | measured (64K tile-stream floor; production host tile protocol still open) |
| P-16 | visible-first: cells evaluated before P0-complete ≤ upstream closure of visible rects | viewport-64k-of-1M | evaluation counter vs full occupied floor | measured (64K same-row R1C1 viewport floor; production viewport scheduler still open) |
| P-17 | insert/delete: compact region metadata touched ≤ boundary split count, not authored cells rewritten (guards the 337 s rebind-churn pathology) | insert-storm-1M | compact-region metadata touches vs eager full-cell rewrite floor | measured (compact row insert/delete scale floor) |
| P-18 | partition witness: same-level regions have disjoint read/write rects (witness validity only; parallel execution deferred) | boring-1Mx10, pascal-r1c1-1M, zig-zag-1M | witness validity + max-parallelism bound recorded | measured (compact partition witness; parallel execution still deferred) |
| P-19 | warm no-op visits 0 cells on non-volatile sheets (permanent tripwire for the 10–80× warm pathology) | all, warm pass | cells-visited == 0 | measured (seed + scale smoke) |
| P-20 | occupancy-proportional aggregates: `SUM(A:A)` slots visited == occupied, never 2^20 | full-column-1M, zig-zag-1M | reader slots-visited | measured (seed + strict unit + sparse and full-column scale artifacts) |
| P-21 | COW retention bytes ∝ touched blocks per revision | cow-retention-1M, edit-storm + retention | retained roots vs full snapshot retention floor | measured (shared dense-payload retained-root floor; production retention GC/lifecycle still open) |
| P-22 | grid publication entries ∝ delta, never full-N | publication-delta-1M, edit-storm | publication-entries | measured (compact dense publication-delta floor; production publication lifecycle still open) |
| P-23 | re-spill cost ∝ \|old ∪ new extent\| (cells written + rects propagated), never sheet size | filter-spill-1M, sequence-spill-1M | cells written + rects propagated | measured (old-spill indexed clear + dense SEQUENCE/FILTER committed-publication and FILTER lifecycle floors; broader FILTER matrix still open) |
| P-24 | hidden-toggle: cells_evaluated == \|affected visibility consumers\|, edges visited O(log intervals + k), independent of sheet size | hide-storm | seeds + cells-evaluated | measured (AxisState visibility block-index floor; broader hide/filter/outline provenance still open) |
| P-25 | `A1#` dereference = O(1) spill-ledger probes, no extent scan | spill-anchor-1M, spill-storm | ledger-probe counter | measured (1M anchor-provider floor; broad spill-storm still open) |
| P-26 | spill blockage check ∝ occupied blocks ∩ intended extent (never empty-slot iteration) | spill intent over empty 1M-row column | blocks-probed counter | measured (optimized compact blocker-probe floor; broad spill arbitration still open) |
| P-27 | spill-extent epoch precision: `A1#` consumers re-evaluate only on extent-epoch or value change, never on unrelated anchor-sheet churn | spill-storm | consumer-evaluations counter | measured (reference dirty-closure floor plus first GridCalc-Ref/optimized spill-epoch ledger and optimized spill-state commit hook; broad publication lifecycle still open) |
| P-28 | aggregate-context query slots visited ∝ axis runs ∩ referenced span (provider side); per-cell seam expansion bounded by range extent (run-compressed seam = registered deferral) | hide-storm | provider slots-visited | measured (provider row-run report floor; run-compressed OxFunc packet still open) |

## 5. Workloads

Current P-19 evidence is deliberately narrow: `w061-grid-seed-cli-009` proves the optimized
repeated-R1C1 seed can hit an unchanged-sheet warm no-op cache with zero cells visited and zero
formula evaluations, and that the warm readout equals the optimized baseline. This is a smoke
counter gate for compact authored state, not a full production invalidation or volatility
claim.

Current P-20 evidence has both sparse and fully occupied retained legs: `w061-grid-seed-cli-009`
emits P-20 occupied-slot reports for `1:1` and `A:B` in the whole-axis seed case,
`optimized_grid_whole_column_enumeration_visits_occupied_slots_not_extent` covers strict Excel
`A:B` bounds with 2,097,152 declared cells and 2 occupied slots visited, and
`w061-grid-scale-full-column-1m-001` covers the fully occupied `full-column-1M` workload with
1,048,576 declared/defined dense slots visited.

Current grid-scale smoke evidence:

- `w061-grid-scale-sparse-whole-column-001`: strict `A:A` declares 1,048,576 cells, defines 3
  occupied cells, visits 3 slots, records `blank_cell_bytes == 0`, and passes P-10 blank-cell
  and P-20 assertions.
- `w061-grid-scale-full-column-1m-001`: strict `A:A` over one fully occupied dense numeric
  column declares 1,048,576 cells, defines and visits exactly 1,048,576 dense value slots,
  visits zero sparse slots, intersects one compact region, computes `SUM(A:A) == 549756338176`,
  records 8,388,743 dense-value-region authored bytes
  (`dense_bytes_per_cell_micros == 8000129`, about 8.000 B/cell), warms with zero cells
  visited, and passes P-00/P-10/P-19/P-20 plus `GRID-FULL-COLUMN-1M`.
- `w061-grid-scale-sparse-singletons-001`: 1,000,000 isolated numeric authored cells over a
  1,000,000 x 16 grid remain sparse points, report 24,000,000 sparse-point authored bytes
  (`sparse_point_bytes_per_cell_micros == 24000000`, exactly 24 B/cell), sample first/middle/last
  values as 1/500000/1000000, record `blank_cell_bytes == 0`, and pass P-10.
- `w061-grid-scale-zig-zag-1m-001`: 1,000,000 isolated numeric authored cells zig-zag across
  the full strict Excel width (16,384 columns), remain sparse points, report 24,000,000
  sparse-point authored bytes (`sparse_point_bytes_per_cell_micros == 24000000`, exactly
  24 B/cell), span 16,384 configured columns, record 16,383,000,000 blank cells at
  `blank_cell_bytes == 0`, sample first/middle/last values as 1/500000/1000000 at columns
  1/8480/576, record a P-18 partition bound of 1,000,000 unique sparse points with zero compact
  region overlaps, and pass P-10/P-18 plus `GRID-ZIG-ZAG-1M`.
- `w061-grid-scale-dense-values-001`: 1,000 x 10 dense values remain one dense region, compute
  zero sparse cells, store 10,000 packed numeric cells, report 80,135 dense-region authored
  bytes (`dense_bytes_per_cell_micros == 8013500`, about 8.014 B/cell), and pass P-00/P-10.
- `w061-grid-scale-repeated-r1c1-001`: 1,000 repeated R1C1 formula cells prepare one template,
  produce last value 2000, warm with zero cells visited, report 260 repeated-formula authored
  bytes (`repeated_formula_bytes_per_cell_micros == 260000`, about 0.260 B/formula cell) plus
  packed numeric inputs, publish dense computed formula output (`computed_sparse_cells == 0`),
  report 1 formula-plan miss plus 999 hits (`formula_plan_cache_hit_rate_micros == 999000`),
  and pass P-00/P-10/P-11/P-14/P-19.
- `w061-grid-scale-fill-down-r1c1-001`: 1,000,000-row fill-down with `A1 = 1` and a single
  repeated R1C1 region `A2:A1000000 = R[-1]C+1` prepares one template, evaluates 999,999 formula
  cells, produces first/middle/last values 1/500000/1000000, reports 260 shared authored
  formula bytes (`repeated_formula_bytes_per_cell_micros == 261` after rounding over 999,999
  cells), publishes one dense computed formula-output region plus the one sparse seed value,
  reports 1 formula-plan miss plus 999,998 hits (`formula_plan_cache_hit_rate_micros == 999999`),
  warms with zero cells visited, and passes P-00/P-10/P-11/P-14/P-19 plus the
  `GRID-FILL-DOWN-R1C1` value assertion.
- `w061-grid-scale-pascal-r1c1-1m-001`: 1,000,000 rows x 8 columns with a dense first-column
  boundary, seven sparse top-row seeds, and a 6,999,993-cell repeated R1C1 formula region
  `=RC[-1]+R[-1]C`. It prepares one template, evaluates 8,000,000 occupied cells exactly once,
  reports 8,000,980 authored bytes (`authored_bytes_per_cell_micros == 1000123`, about
  1.000 B/occupied cell), 270 repeated-formula authored bytes
  (`repeated_formula_bytes_per_cell_micros == 39` after rounding over 6,999,993 formula cells),
  one formula-plan miss plus 6,999,992 hits (`formula_plan_cache_hit_rate_micros == 1000000`),
  two dense computed regions, seven sparse computed seed cells, sampled first/middle/last
  recurrence values matching the expected row-major recurrence, a zero-visit warm no-op, and
  passes P-00/P-10/P-11/P-14/P-18/P-19 plus `GRID-PASCAL-R1C1-1M`.
- `w061-grid-scale-boring-1mx10-001`: 1,000,000 rows x 10 columns with 8,000,000 packed
  dense numeric value cells and a 2,000,000-cell repeated R1C1 formula block `=RC[-1]*2`.
  It prepares one template, evaluates 10,000,000 occupied cells exactly once, reports
  64,000,802 authored bytes (`authored_bytes_per_cell_micros == 6400081`, about
  6.400 B/occupied cell), 260 repeated-formula authored bytes, first/middle/last sampled
  formula values 2016/2000000032/4000000032, zero blank-cell bytes, and a zero-visit warm
  no-op. It reports 1 formula-plan miss plus 1,999,999 hits
  (`formula_plan_cache_hit_rate_micros == 1000000`) and passes P-00/P-10/P-11/P-14/P-19 plus
  the `GRID-BORING-1MX10` value/storage assertion.
  The run also reports `computed_dense_value_regions == 2` and `computed_sparse_cells == 0`,
  covering dense computed output for the supported repeated-R1C1 formula block. It also passes
  P-18 with one dense-value region, one repeated-formula region, zero same-level overlaps, and
  `partition_max_parallelism_bound == 2`.
- `w061-grid-scale-direct-r1c1-1m-001`: 1,000,000 rows x 3 columns with one packed dense
  numeric input column and two repeated R1C1 direct scalar formula blocks, `=RC[-1]` and
  `=(RC[-2])`. It prepares two templates and two compiled R1C1 scalar plans, evaluates
  3,000,000 occupied cells exactly once, reports 8,001,098 authored bytes
  (`authored_bytes_per_cell_micros == 2667033`, about 2.667 B/occupied cell), two formula-plan
  misses plus 1,999,998 hits, two compiled-plan misses, 3,000,000 packed dense numeric computed
  cells, zero sparse computed cells, sampled direct and parenthesized values
  10/5000000/10000000, a zero-visit warm no-op, and a valid P-18 partition witness. It passes
  P-00/P-10/P-11/P-14/P-18/P-19 plus `GRID-DIRECT-R1C1-1M`.
- `w061-grid-scale-unary-r1c1-1m-001`: 1,000,000 rows x 4 columns with one packed dense
  numeric input column and three repeated R1C1 unary formula blocks, `=-RC[-1]`,
  `=-(RC[-2]+5)`, and `=-RC[-3]*2+1`. It prepares three templates and three compiled R1C1
  plans, evaluates 4,000,000 occupied cells exactly once, reports 8,001,372 authored bytes
  (`authored_bytes_per_cell_micros == 2000343`, about 2.000 B/occupied cell), three
  formula-plan misses plus 2,999,997 hits, three compiled-plan misses, 4,000,000 packed dense
  numeric computed cells, zero sparse computed cells, sampled direct values
  -10/-5000000/-10000000, parenthesized values -15/-5000005/-10000005, arithmetic values
  -19/-9999999/-19999999, a zero-visit warm no-op, and a valid P-18 partition witness. It
  passes P-00/P-10/P-11/P-14/P-18/P-19 plus `GRID-UNARY-R1C1-1M`.
- `w061-grid-scale-argument-aggregate-r1c1-1m-001`: 1,000,000 rows x 8 columns with two
  packed dense numeric input columns and six repeated R1C1 aggregate argument-list formula
  blocks, `=SUM(RC[-2],RC[-1],5)`, `=COUNT(RC[-3],RC[-2],5)`,
  `=PRODUCT(RC[-4],RC[-3],2)`, `=AVERAGE(RC[-5],RC[-4],5)`,
  `=MIN(RC[-6],RC[-5],5)`, and `=MAX(RC[-7],RC[-6],5)`. It prepares six templates and six
  compiled R1C1 plans, evaluates 8,000,000 occupied cells exactly once, reports 16,002,306
  authored bytes (`authored_bytes_per_cell_micros == 2000289`, about 2.000 B/occupied cell),
  six formula-plan misses plus 5,999,994 hits, six compiled-plan misses, 8,000,000 packed
  dense numeric computed cells, zero sparse computed cells, sampled first SUM/COUNT/PRODUCT/
  AVERAGE/MIN/MAX values 16/3/20/5.333333333333333/1/10, sampled middle SUM/PRODUCT/AVERAGE
  values 5500005/5000000000000/1833335, sampled last SUM/PRODUCT/AVERAGE/MIN/MAX values
  11000005/20000000000000/3666668.3333333335/5/10000000, a zero-visit warm no-op, and a valid
  P-18 partition witness. It passes P-00/P-10/P-11/P-14/P-18/P-19 plus
  `GRID-ARGUMENT-AGGREGATE-R1C1-1M`.
- `w061-grid-scale-math-function-r1c1-1m-001`: 1,000,000 rows x 5 columns with two packed
  dense numeric input columns and three repeated R1C1 scalar math-function formula blocks,
  `=ABS(RC[-2])`, `=SQRT(RC[-2])`, and `=POWER(ABS(RC[-4]),2)`. It prepares three templates
  and three compiled R1C1 plans, evaluates 5,000,000 occupied cells exactly once, reports
  16,001,400 authored bytes (`authored_bytes_per_cell_micros == 3200280`, about
  3.200 B/occupied cell), three formula-plan misses plus 2,999,997 hits, three compiled-plan
  misses, 5,000,000 packed dense numeric computed cells, zero sparse computed cells, sampled
  first ABS/SQRT/POWER values 1/1/1, sampled middle values 500000/500000/250000000000, sampled
  last values 1000000/1000000/1000000000000, a zero-visit warm no-op, and a valid P-18
  partition witness. It passes P-00/P-10/P-11/P-14/P-18/P-19 plus
  `GRID-MATH-FUNCTION-R1C1-1M`.
- `w061-grid-scale-mod-function-r1c1-1m-001`: 1,000,000 rows x 5 columns with two packed
  dense numeric input columns and three repeated R1C1 MOD formula blocks,
  `=MOD(RC[-2],RC[-1])`, `=IF(MOD(RC[-3],2)=0,RC[-3]/2,RC[-3]*3)`, and
  `=MOD(POWER(RC[-4],2),RC[-3])`. It prepares three templates and three compiled R1C1 plans,
  evaluates 5,000,000 occupied cells exactly once, reports 16,001,478 authored bytes
  (`authored_bytes_per_cell_micros == 3200296`, about 3.200 B/occupied cell), three
  formula-plan misses plus 2,999,997 hits, three compiled-plan misses, 5,000,000 dense
  numeric-packed computed cells, zero logical-packed output cells, zero sparse computed cells,
  sampled first MOD/IF/POWER-MOD values 1/3/1, sampled middle values 4/250000/2, sampled last
  values 1/500000/1, a zero-visit warm no-op, and a valid P-18 partition witness. It passes
  P-00/P-10/P-11/P-14/P-18/P-19 plus `GRID-MOD-FUNCTION-R1C1-1M`.
- `w061-grid-scale-rounding-function-r1c1-1m-001`: 1,000,000 rows x 5 columns with two packed
  dense numeric input columns and three repeated R1C1 ROUND-family formula blocks,
  `=ROUND(RC[-2],RC[-1])`, `=ROUNDUP(RC[-3],RC[-2])`, and
  `=ROUNDDOWN(RC[-4],RC[-3])`. It prepares three templates and three compiled R1C1 plans,
  evaluates 5,000,000 occupied cells exactly once, reports 16,001,446 authored bytes
  (`authored_bytes_per_cell_micros == 3200290`, about 3.200 B/occupied cell), three
  formula-plan misses plus 2,999,997 hits, three compiled-plan misses, 5,000,000 dense
  numeric-packed computed cells, zero logical-packed output cells, zero sparse computed cells,
  sampled first ROUND/ROUNDUP/ROUNDDOWN values 2/2/1, sampled middle values
  500001/500001/500000, sampled last values 1000001/1000001/1000000, a zero-visit warm no-op,
  and a valid P-18 partition witness. It passes P-00/P-10/P-11/P-14/P-18/P-19 plus
  `GRID-ROUNDING-FUNCTION-R1C1-1M`.
- `w061-grid-scale-integer-function-r1c1-1m-001`: 1,000,000 rows x 5 columns with two packed
  dense numeric input columns and three repeated R1C1 INT/TRUNC formula blocks,
  `=INT(RC[-2])`, `=TRUNC(RC[-3])`, and `=TRUNC(RC[-4],RC[-3])`. It prepares three templates
  and three compiled R1C1 plans, evaluates 5,000,000 occupied cells exactly once, reports
  16,001,402 authored bytes (`authored_bytes_per_cell_micros == 3200281`, about
  3.200 B/occupied cell), three formula-plan misses plus 2,999,997 hits, three compiled-plan
  misses, 5,000,000 dense numeric-packed computed cells, zero logical-packed output cells, zero
  sparse computed cells, sampled first INT/TRUNC/TRUNC-tens values 1/1/0, sampled middle values
  500000/500000/500000, sampled last values 1000000/1000000/1000000, a zero-visit warm no-op,
  and a valid P-18 partition witness. It passes P-00/P-10/P-11/P-14/P-18/P-19 plus
  `GRID-INTEGER-FUNCTION-R1C1-1M`.
- `w061-grid-scale-log-function-r1c1-1m-001`: 1,000,000 rows x 6 columns with two packed
  dense numeric input columns and four repeated R1C1 EXP/LN/LOG formula blocks,
  `=EXP(RC[-1])`, `=LN(RC[-3])`, `=LOG10(RC[-4]*100)`, and
  `=LOG(RC[-5]*100,10)`. It prepares four templates and four compiled R1C1 plans,
  evaluates 6,000,000 occupied cells exactly once, reports 16,001,670 authored bytes
  (`authored_bytes_per_cell_micros == 2666945`, about 2.667 B/occupied cell), four
  formula-plan misses plus 3,999,996 hits, four compiled-plan misses, 6,000,000 dense
  numeric-packed computed cells, zero logical-packed output cells, zero sparse computed cells,
  sampled first/middle/last EXP/LN/LOG10/LOG values 1/0/2/2, a zero-visit warm no-op, and a
  valid P-18 partition witness. It passes P-00/P-10/P-11/P-14/P-18/P-19 plus
  `GRID-LOG-FUNCTION-R1C1-1M`.
- `w061-grid-scale-trig-function-r1c1-1m-001`: 1,000,000 rows x 4 columns with one packed
  dense numeric input column and three repeated R1C1 SIN/COS/TAN formula blocks,
  `=SIN(RC[-1])`, `=COS(RC[-2])`, and `=TAN(RC[-3])`. It prepares three templates and three
  compiled R1C1 plans, evaluates 4,000,000 occupied cells exactly once, reports 8,001,380
  authored bytes (`authored_bytes_per_cell_micros == 2000345`, about 2.000 B/occupied cell),
  three formula-plan misses plus 2,999,997 hits, three compiled-plan misses, 4,000,000 dense
  numeric-packed computed cells, zero logical-packed output cells, zero sparse computed cells,
  sampled first/middle/last SIN/COS/TAN values 0/1/0, a zero-visit warm no-op, and a valid P-18
  partition witness. It passes P-00/P-10/P-11/P-14/P-18/P-19 plus
  `GRID-TRIG-FUNCTION-R1C1-1M`.
- `w061-grid-scale-angle-function-r1c1-1m-001`: 1,000,000 rows x 6 columns with two packed
  dense numeric input columns and four repeated R1C1 RADIANS/DEGREES/PI formula blocks,
  `=RADIANS(RC[-2])`, `=DEGREES(RC[-2])`, `=SIN(RADIANS(RC[-4]))`, and `=PI()`. It prepares
  four templates and four compiled R1C1 plans, evaluates 6,000,000 occupied cells exactly once,
  reports 16,001,666 authored bytes (`authored_bytes_per_cell_micros == 2666945`, about
  2.667 B/occupied cell), four formula-plan misses plus 3,999,996 hits, four compiled-plan
  misses, 6,000,000 dense numeric-packed computed cells, zero logical-packed output cells, zero
  sparse computed cells, sampled first/middle/last RADIANS/DEGREES/SIN/PI values
  0/0/0/3.141592653589793, a zero-visit warm no-op, and a valid P-18 partition witness. It
  passes P-00/P-10/P-11/P-14/P-18/P-19 plus `GRID-ANGLE-FUNCTION-R1C1-1M`.
- `w061-grid-scale-reference-function-r1c1-1m-001`: 1,000,000 rows x 6 columns with six
  repeated R1C1 reference-function formula blocks, `=ROW()`, `=COLUMN()`, `=ROW(RC[-2])`,
  `=COLUMN(RC[-3])`, `=ROWS(R1C1:R3C1)`, and `=COLUMNS(RC[-5]:RC[-3])`. It prepares six
  templates and six compiled R1C1 plans, evaluates 6,000,000 occupied formula cells exactly
  once, reports 2,069 authored bytes (`authored_bytes_per_cell_micros == 345`), six
  formula-plan misses plus 5,999,994 hits, six compiled-plan misses, 6,000,000 dense
  numeric-packed computed cells, zero literal cells, zero sparse computed cells, sampled ROW
  values 1/500000/1000000 and sampled COLUMN/ROWS/COLUMNS values 2/1/3/3, a zero-visit warm
  no-op, and a valid P-18 partition witness. It passes P-00/P-10/P-11/P-14/P-18/P-19 plus
  `GRID-REFERENCE-FUNCTION-R1C1-1M`.
- `w061-grid-scale-logical-function-r1c1-1m-001`: 1,000,000 rows x 5 columns with two
  packed dense signed numeric input columns and three repeated R1C1 logical-function formula
  blocks, `=AND(RC[-2]>0,RC[-1]>0)`, `=OR(RC[-3]>0,RC[-2]>0)`, and
  `=NOT(AND(RC[-4]>0,RC[-3]>0))`. It prepares three templates and three compiled R1C1 plans,
  evaluates 5,000,000 occupied cells exactly once, reports 16,001,454 authored bytes
  (`authored_bytes_per_cell_micros == 3200291`, about 3.200 B/occupied cell), three
  formula-plan misses plus 2,999,997 hits, three compiled-plan misses, 5,000,000 dense
  computed cells with 2,000,000 numeric-packed input cells, 3,000,000 logical-packed formula
  output cells, zero sparse computed cells, sampled first AND/OR/NOT values false/true/true,
  sampled row-3 values true/true/false, sampled middle values false/false/true, sampled last
  values false/false/true, a zero-visit warm no-op, and a valid P-18 partition witness. It
  passes P-00/P-10/P-11/P-14/P-18/P-19 plus `GRID-LOGICAL-FUNCTION-R1C1-1M`.
- `w061-grid-scale-if-logical-r1c1-1m-001`: 1,000,000 rows x 5 columns with two packed
  dense signed numeric input columns and three repeated R1C1 IF formula blocks with logical
  function conditions:
  `=IF(AND(RC[-2]>0,RC[-1]>0),RC[-2]+RC[-1],0)`,
  `=IF(OR(RC[-3]>0,RC[-2]>0),RC[-3]-RC[-2],0)`, and
  `=IF(NOT(AND(RC[-4]>0,RC[-3]>0)),ABS(RC[-4]),0)`. It prepares three templates and three
  compiled R1C1 plans, evaluates 5,000,000 occupied cells exactly once, reports 16,001,570
  authored bytes (`authored_bytes_per_cell_micros == 3200314`, about 3.200 B/occupied cell),
  three formula-plan misses plus 2,999,997 hits, three compiled-plan misses, 5,000,000 dense
  numeric-packed computed cells, zero logical-packed output cells, zero sparse computed cells,
  sampled first AND/OR/NOT-condition IF values 0/2/1, sampled row-3 values 6/0/0, sampled
  middle values 0/0/500000, sampled last values 0/0/1000000, a zero-visit warm no-op, and a
  valid P-18 partition witness. It passes P-00/P-10/P-11/P-14/P-18/P-19 plus
  `GRID-IF-LOGICAL-R1C1-1M`.
- `w061-grid-scale-two-left-r1c1-1m-001`: 1,000,000 rows x 10 columns with 8,000,000 packed
  dense numeric input cells plus a 2,000,000-cell repeated R1C1 formula block
  `=RC[-2]+RC[-1]`. It prepares one template and one compiled R1C1 plan, evaluates
  10,000,000 occupied cells exactly once, reports 64,000,804 authored bytes
  (`authored_bytes_per_cell_micros == 6400081`, about 6.400 B/occupied cell), 270 repeated
  formula bytes (`repeated_formula_bytes_per_cell_micros == 135` after rounding over
  2,000,000 formula cells), one formula-plan miss plus 1,999,999 hits, one compiled-plan miss,
  dense computed formula output (`computed_sparse_cells == 0`), sampled formula values
  2015/1500000023/3000000023, a zero-visit warm no-op, and a valid P-18 partition witness.
  It passes P-00/P-10/P-11/P-14/P-18/P-19 plus `GRID-TWO-LEFT-R1C1-1M`.
- `w061-grid-scale-absolute-r1c1-1m-001`: 1,000,000 rows x 3 columns with two packed dense
  numeric input columns and a 1,000,000-cell mixed absolute/relative repeated R1C1 formula block
  `=RC[-1]+R1C1`. It prepares one template and one compiled R1C1 plan, evaluates 3,000,000
  occupied cells exactly once, reports 16,000,800 authored bytes
  (`authored_bytes_per_cell_micros == 5333600`, about 5.334 B/occupied cell), 266 shared
  repeated-formula authored bytes, one formula-plan miss plus 999,999 hits, one compiled-plan
  miss, dense computed formula output (`computed_sparse_cells == 0`), sampled formula values
  3/1000001/2000001, a zero-visit warm no-op, and a valid P-18 partition witness. It passes
  P-00/P-10/P-11/P-14/P-18/P-19 plus `GRID-ABSOLUTE-R1C1-1M`.
- `w061-grid-scale-division-r1c1-1m-001`: 1,000,000 rows x 2 columns with one packed dense
  numeric input column and a 1,000,000-cell repeated R1C1 formula block `=RC[-1]/2`. It
  prepares one template and one compiled R1C1 plan, evaluates 2,000,000 occupied cells exactly
  once, reports 8,000,794 authored bytes (`authored_bytes_per_cell_micros == 4000397`, about
  4.000 B/occupied cell), 260 repeated-formula authored bytes, one formula-plan miss plus
  999,999 hits, one compiled-plan miss, dense computed formula output
  (`computed_sparse_cells == 0`), sampled formula values 1/500000/1000000, a zero-visit warm
  no-op, and a valid P-18 partition witness. It passes P-00/P-10/P-11/P-14/P-18/P-19 plus
  `GRID-DIVISION-R1C1-1M`.
- `w061-grid-scale-decimal-r1c1-1m-001`: 1,000,000 rows x 2 columns with one packed dense
  numeric input column and a 1,000,000-cell repeated R1C1 formula block `=RC[-1]*0.5`. It
  prepares one template and one compiled R1C1 plan, evaluates 2,000,000 occupied cells exactly
  once, reports 8,000,798 authored bytes (`authored_bytes_per_cell_micros == 4000399`, about
  4.000 B/occupied cell), 264 repeated-formula authored bytes, one formula-plan miss plus
  999,999 hits, one compiled-plan miss, dense computed formula output
  (`computed_sparse_cells == 0`), sampled formula values 1/500000/1000000, a zero-visit warm
  no-op, and a valid P-18 partition witness. It passes P-00/P-10/P-11/P-14/P-18/P-19 plus
  `GRID-DECIMAL-R1C1-1M`.
- `w061-grid-scale-recursive-binary-r1c1-1m-001`: 1,000,000 rows x 5 columns with three
  packed dense numeric input columns and two repeated R1C1 recursive binary formula blocks:
  `=RC[-3]+RC[-2]*RC[-1]` and `=(RC[-4]+RC[-3])*RC[-2]`. It prepares two templates and two
  compiled R1C1 plans, evaluates 5,000,000 occupied cells exactly once, reports 24,001,154
  authored bytes (`authored_bytes_per_cell_micros == 4800231`, about 4.800 B/occupied cell),
  two formula-plan misses plus 1,999,998 hits, two compiled-plan misses, 5,000,000 packed dense
  numeric computed cells, zero sparse computed cells, sampled precedence values
  21/10500000/21000000 and parenthesized values 22/11000000/22000000, a zero-visit warm
  no-op, and a valid P-18 partition witness. It passes P-00/P-10/P-11/P-14/P-18/P-19 plus
  `GRID-RECURSIVE-BINARY-R1C1-1M`.
- `w061-grid-scale-if-r1c1-1m-001`: 1,000,000 rows x 2 columns with one packed dense signed
  numeric input column and a 1,000,000-cell repeated R1C1 formula block
  `=IF(RC[-1]>0,RC[-1],0)`. It prepares one template and one compiled R1C1 IF plan, evaluates
  2,000,000 occupied cells exactly once, reports 8,000,868 authored bytes
  (`authored_bytes_per_cell_micros == 4000434`, about 4.000 B/occupied cell), one formula-plan
  miss plus 999,999 hits, one compiled-plan miss, dense computed formula output
  (`computed_sparse_cells == 0`), sampled formula values 1/0/999999/0, a zero-visit warm no-op,
  and a valid P-18 partition witness. It passes P-00/P-10/P-11/P-14/P-18/P-19 plus
  `GRID-IF-R1C1-1M`.
- `w061-grid-scale-if-branch-r1c1-1m-001`: 1,000,000 rows x 2 columns with one packed dense
  signed numeric input column and a 1,000,000-cell repeated R1C1 formula block
  `=IF(RC[-1]>0,RC[-1]*2,RC[-1]/2)`. It prepares one template and one compiled R1C1 IF plan
  with scalar branch expressions, evaluates 2,000,000 occupied cells exactly once, reports
  8,000,886 authored bytes (`authored_bytes_per_cell_micros == 4000443`, about 4.000
  B/occupied cell), one formula-plan miss plus 999,999 hits, one compiled-plan miss, dense
  computed formula output (`computed_sparse_cells == 0`, `computed_dense_cells == 2000000`,
  and `computed_dense_numeric_packed_cells == 2000000`), sampled formula values
  2/-250000/1999998/-500000, a zero-visit warm no-op, and a valid P-18 partition witness. It
  passes P-00/P-10/P-11/P-14/P-18/P-19 plus `GRID-IF-BRANCH-R1C1-1M`.
- `w061-grid-scale-nested-if-r1c1-1m-001`: 1,000,000 rows x 3 columns with two packed dense
  input columns and a 1,000,000-cell repeated R1C1 formula block
  `=IF(RC[-2]>500000,IF(RC[-1]>0,RC[-2]*2,RC[-2]*3),IF(RC[-1]>0,RC[-2]+1,RC[-2]-1))`.
  It prepares one template and one compiled R1C1 IF plan with nested scalar IF branch
  expressions, evaluates 3,000,000 occupied cells exactly once, reports 16,000,984 authored
  bytes (`authored_bytes_per_cell_micros == 5333662`, about 5.334 B/occupied cell), one
  formula-plan miss plus 999,999 hits, one compiled-plan miss, 3,000,000 packed dense numeric
  computed cells, zero sparse computed cells, sampled formula values 0/500001/1500003/2000000,
  a zero-visit warm no-op, and a valid P-18 partition witness. It passes
  P-00/P-10/P-11/P-14/P-18/P-19 plus `GRID-NESTED-IF-R1C1-1M`.
- `w061-grid-scale-iferror-r1c1-1m-001`: 1,000,000 rows x 3 columns with one packed dense
  input region containing 2,000,000 numeric cells and a 1,000,000-cell repeated R1C1 formula
  block `=IFERROR(RC[-2]/RC[-1],0)`. It prepares one template and one compiled R1C1 IFERROR
  plan, evaluates 3,000,000 occupied cells exactly once, reports 16,000,874 authored bytes
  (`authored_bytes_per_cell_micros == 5333625`, about 5.334 B/occupied cell), one formula-plan
  miss plus 999,999 hits, one compiled-plan miss, dense computed formula output
  (`computed_sparse_cells == 0`), sampled formula values 1/0/999999/0, a zero-visit warm no-op,
  and a valid P-18 partition witness. It passes P-00/P-10/P-11/P-14/P-18/P-19 plus
  `GRID-IFERROR-R1C1-1M`.
- `w061-grid-scale-comparison-r1c1-1m-001`: 1,000,000 rows x 2 columns with one packed dense
  signed numeric input column and a 1,000,000-cell repeated R1C1 formula block `=RC[-1]>0`.
  It prepares one template and one compiled R1C1 comparison plan, evaluates 2,000,000 occupied
  cells exactly once, reports 8,000,842 authored bytes
  (`authored_bytes_per_cell_micros == 4000421`, about 4.000 B/occupied cell), one formula-plan
  miss plus 999,999 hits, one compiled-plan miss, dense computed logical formula output
  (`computed_sparse_cells == 0`, `computed_dense_cells == 2000000`,
  `computed_dense_numeric_packed_cells == 1000000`, and
  `computed_dense_logical_packed_cells == 1000000`), sampled formula values
  true/false/true/false, a zero-visit warm no-op, and a valid P-18 partition witness. It passes
  P-00/P-10/P-11/P-14/P-18/P-19 plus `GRID-COMPARISON-R1C1-1M`.
- `w061-grid-scale-comparison-expression-r1c1-1m-001`: 1,000,000 rows x 3 columns with one
  packed dense signed numeric input region containing 2,000,000 cells and a 1,000,000-cell
  repeated R1C1 formula block `=RC[-2]*2>RC[-1]+1`. It prepares one template and one compiled
  R1C1 comparison plan over scalar-expression operands, evaluates 3,000,000 occupied cells
  exactly once, reports 16,000,860 authored bytes
  (`authored_bytes_per_cell_micros == 5333620`, about 5.334 B/occupied cell), one formula-plan
  miss plus 999,999 hits, one compiled-plan miss, dense computed logical formula output
  (`computed_sparse_cells == 0`, `computed_dense_cells == 3000000`,
  `computed_dense_numeric_packed_cells == 2000000`, and
  `computed_dense_logical_packed_cells == 1000000`), sampled formula values
  false/false/true/false, a zero-visit warm no-op, and a valid P-18 partition witness. It passes
  P-00/P-10/P-11/P-14/P-18/P-19 plus `GRID-COMPARISON-EXPRESSION-R1C1-1M`.
- `w061-grid-scale-comparison-iferror-r1c1-1m-001`: 1,000,000 rows x 3 columns with one
  packed dense numeric input region containing 2,000,000 cells and a 1,000,000-cell repeated
  R1C1 formula block `=IFERROR(RC[-2]/RC[-1],0)>0`. It prepares one template and one compiled
  R1C1 comparison plan with a nested IFERROR scalar operand, evaluates 3,000,000 occupied cells
  exactly once, reports 16,000,878 authored bytes
  (`authored_bytes_per_cell_micros == 5333626`, about 5.334 B/occupied cell), one formula-plan
  miss plus 999,999 hits, one compiled-plan miss, dense computed logical formula output
  (`computed_sparse_cells == 0`, `computed_dense_cells == 3000000`,
  `computed_dense_numeric_packed_cells == 2000000`, and
  `computed_dense_logical_packed_cells == 1000000`), sampled formula values
  true/false/true/false, a zero-visit warm no-op, and a valid P-18 partition witness. It passes
  P-00/P-10/P-11/P-14/P-18/P-19 plus `GRID-COMPARISON-IFERROR-R1C1-1M`.
- `w061-grid-scale-sum-row-r1c1-1m-001`: 1,000,000 rows x 4 columns with one packed dense
  input region containing 3,000,000 numeric cells and a 1,000,000-cell repeated R1C1 formula
  block `=SUM(RC[-3]:RC[-1])`. It prepares one template and one compiled R1C1 SUM-range plan,
  evaluates 4,000,000 occupied cells exactly once, reports 24,000,814 authored bytes
  (`authored_bytes_per_cell_micros == 6000204`, about 6.000 B/occupied cell), 280
  repeated-formula authored bytes, one formula-plan miss plus 999,999 hits, one compiled-plan
  miss, dense computed formula output (`computed_sparse_cells == 0`), sampled formula values
  6/3000000/6000000, a zero-visit warm no-op, and a valid P-18 partition witness. It passes
  P-00/P-10/P-11/P-14/P-18/P-19 plus `GRID-SUM-ROW-R1C1-1M`.
- `w061-grid-scale-sumsq-row-r1c1-1m-001`: 1,000,000 rows x 4 columns with one packed dense
  input region containing 3,000,000 numeric cells and a 1,000,000-cell repeated R1C1 formula
  block `=SUMSQ(RC[-3]:RC[-1])`. It prepares one template and one compiled R1C1 SUMSQ-range
  plan, evaluates 4,000,000 occupied cells exactly once, reports 24,000,866 authored bytes
  (`authored_bytes_per_cell_micros == 6000217`, about 6.000 B/occupied cell), 284
  repeated-formula authored bytes, one formula-plan miss plus 999,999 hits, one compiled-plan
  miss, dense numeric computed formula output (`computed_dense_cells == 4000000`,
  `computed_dense_numeric_packed_cells == 4000000`, and `computed_sparse_cells == 0`), sampled
  formula values 14/3500000000000/14000000000000, a zero-visit warm no-op, and a valid P-18
  partition witness. It passes P-00/P-10/P-11/P-14/P-18/P-19 plus
  `GRID-SUMSQ-ROW-R1C1-1M`.
- `w061-grid-scale-count-row-r1c1-1m-001`: 1,000,000 rows x 4 columns with one packed dense
  input region containing 3,000,000 numeric cells and a 1,000,000-cell repeated R1C1 formula
  block `=COUNT(RC[-3]:RC[-1])`. It prepares one template and one compiled R1C1 COUNT-range
  plan, evaluates 4,000,000 occupied cells exactly once, reports 24,000,818 authored bytes
  (`authored_bytes_per_cell_micros == 6000205`, about 6.000 B/occupied cell), 284
  repeated-formula authored bytes, one formula-plan miss plus 999,999 hits, one compiled-plan
  miss, dense computed formula output (`computed_sparse_cells == 0`), sampled formula values
  3/3/3, a zero-visit warm no-op, and a valid P-18 partition witness. It passes
  P-00/P-10/P-11/P-14/P-18/P-19 plus `GRID-COUNT-ROW-R1C1-1M`.
- `w061-grid-scale-product-row-r1c1-1m-001`: 1,000,000 rows x 4 columns with one packed dense
  input region containing 3,000,000 numeric cells and a 1,000,000-cell repeated R1C1 formula
  block `=PRODUCT(RC[-3]:RC[-1])`. It prepares one template and one compiled R1C1 PRODUCT-range
  plan, evaluates 4,000,000 occupied cells exactly once, reports 24,000,822 authored bytes
  (`authored_bytes_per_cell_micros == 6000206`, about 6.000 B/occupied cell), 288
  repeated-formula authored bytes, one formula-plan miss plus 999,999 hits, one compiled-plan
  miss, dense computed formula output (`computed_sparse_cells == 0`), sampled formula values
  6/6/6, a zero-visit warm no-op, and a valid P-18 partition witness. It passes
  P-00/P-10/P-11/P-14/P-18/P-19 plus `GRID-PRODUCT-ROW-R1C1-1M`.
- `w061-grid-scale-average-row-r1c1-1m-001`: 1,000,000 rows x 4 columns with one packed dense
  input region containing 3,000,000 numeric cells and a 1,000,000-cell repeated R1C1 formula
  block `=AVERAGE(RC[-3]:RC[-1])`. It prepares one template and one compiled R1C1
  AVERAGE-range plan, evaluates 4,000,000 occupied cells exactly once, reports 24,000,822
  authored bytes (`authored_bytes_per_cell_micros == 6000206`, about 6.000 B/occupied cell),
  288 repeated-formula authored bytes, one formula-plan miss plus 999,999 hits, one
  compiled-plan miss, dense computed formula output (`computed_sparse_cells == 0`), sampled
  formula values 2/1000000/2000000, a zero-visit warm no-op, and a valid P-18 partition
  witness. It passes P-00/P-10/P-11/P-14/P-18/P-19 plus `GRID-AVERAGE-ROW-R1C1-1M`.
- `w061-grid-scale-min-max-row-r1c1-1m-001`: 1,000,000 rows x 5 columns with one packed dense
  input region containing 3,000,000 numeric cells, a 1,000,000-cell repeated R1C1 MIN formula
  block `=MIN(RC[-3]:RC[-1])`, and a 1,000,000-cell repeated R1C1 MAX formula block
  `=MAX(RC[-4]:RC[-2])`. It prepares two templates and two compiled R1C1 aggregate plans,
  evaluates 5,000,000 occupied cells exactly once, reports 24,001,094 authored bytes
  (`authored_bytes_per_cell_micros == 4800219`, about 4.800 B/occupied cell), 560
  repeated-formula authored bytes, two formula-plan misses plus 1,999,998 hits, two
  compiled-plan misses, dense computed formula output (`computed_sparse_cells == 0`), sampled
  MIN values 1/500000/1000000 and MAX values 3/1500000/3000000, a zero-visit warm no-op, and a
  valid P-18 partition witness. It passes P-00/P-10/P-11/P-14/P-18/P-19 plus
  `GRID-MIN-MAX-ROW-R1C1-1M`.
- `w061-grid-scale-sum-window-r1c1-1m-001`: 1,000,000 rows x 2 columns with one packed dense
  input column and a 999,998-cell repeated R1C1 formula block `=SUM(R[-2]C[-1]:RC[-1])`
  starting at row 3. It prepares one template and one compiled R1C1 SUM-range plan, evaluates
  1,999,998 occupied cells exactly once, reports 8,000,822 authored bytes
  (`authored_bytes_per_cell_micros == 4000416`, about 4.000 B/occupied cell), 289
  repeated-formula authored bytes, one formula-plan miss plus 999,997 hits, one compiled-plan
  miss, dense computed formula output (`computed_sparse_cells == 0`), sampled formula values
  6/1499997/2999997, a zero-visit warm no-op, and a valid P-18 partition witness. It passes
  P-00/P-10/P-11/P-14/P-18/P-19 plus `GRID-SUM-WINDOW-R1C1-1M`.
- `w061-grid-scale-division-error-r1c1-1m-001`: 1,000,000 rows x 2 columns with one packed
  dense numeric input column and a 1,000,000-cell repeated R1C1 formula block `=RC[-1]/0`. It
  prepares one template and one compiled R1C1 plan, evaluates 2,000,000 occupied cells exactly
  once, reports 8,000,794 authored bytes (`authored_bytes_per_cell_micros == 4000397`, about
  4.000 B/occupied cell), 260 repeated-formula authored bytes, one formula-plan miss plus
  999,999 hits, one compiled-plan miss, dense computed formula output
  (`computed_sparse_cells == 0`), sampled first/middle/last formula values `Div0`, a zero-visit
  warm no-op, and a valid P-18 partition witness. It passes P-00/P-10/P-11/P-14/P-18/P-19 plus
  `GRID-DIVISION-ERROR-R1C1-1M`.
  Unit tests `optimized_grid_compact_oxfml_recalc_evaluates_general_binary_r1c1_templates`,
  `optimized_grid_compact_oxfml_recalc_evaluates_absolute_r1c1_templates`,
  `optimized_grid_compact_oxfml_recalc_evaluates_r1c1_sum_range_templates`,
  `optimized_grid_compact_oxfml_recalc_evaluates_r1c1_sumsq_range_templates`,
  `optimized_grid_compact_oxfml_recalc_evaluates_r1c1_count_range_templates`,
  `optimized_grid_compact_oxfml_recalc_evaluates_r1c1_product_range_templates`,
  `optimized_grid_compact_oxfml_recalc_evaluates_r1c1_average_range_templates`, and
  `optimized_grid_compact_oxfml_recalc_evaluates_r1c1_min_max_range_templates` cover the
  generalized compiler floors behind these retained shapes: binary R1C1 formulas with relative
  and absolute reference operands and finite numeric literal operands over `+`, `-`, `*`, or
  `/`, plus the narrow `SUM(<R1C1 ref>:<R1C1 ref>)`,
  `SUMSQ(<R1C1 ref>:<R1C1 ref>)`,
  `COUNT(<R1C1 ref>:<R1C1 ref>)`,
  `PRODUCT(<R1C1 ref>:<R1C1 ref>)`,
  `AVERAGE(<R1C1 ref>:<R1C1 ref>)`,
  `MIN(<R1C1 ref>:<R1C1 ref>)`, and
  `MAX(<R1C1 ref>:<R1C1 ref>)` aggregate range forms, can publish dense
  numeric or direct `Div0` formula output without adding per-template engine variants.
- `w061-grid-scale-division-error-propagation-r1c1-1m-001`: 1,000,000 rows x 3 columns with
  one packed dense input column, a 1,000,000-cell repeated R1C1 direct-error block
  `=RC[-1]/0`, and a 1,000,000-cell repeated R1C1 dependent block `=RC[-1]+1`. It prepares two
  templates and two compiled R1C1 plans, evaluates 3,000,000 occupied cells exactly once,
  reports 8,001,054 authored bytes (`authored_bytes_per_cell_micros == 2667018`, about
  2.667 B/occupied cell), 520 repeated-formula authored bytes, two formula-plan misses plus
  1,999,998 hits, two compiled-plan misses, dense direct and propagated `Div0` formula output
  (`computed_sparse_cells == 0`), sampled first/middle/last propagated values `Div0`, a
  zero-visit warm no-op, and a valid P-18 partition witness. It passes
  P-00/P-10/P-11/P-14/P-18/P-19 plus `GRID-DIVISION-ERROR-PROPAGATION-R1C1-1M`.
  The same unit test covers the compiler floor for the dependent `=RC[-1]+1` propagation shape:
  a single upstream worksheet error operand is propagated into dense `CalcValue` output inside
  the bounded recursive binary R1C1 class.
- `w061-grid-scale-aggregate-error-r1c1-1m-001`: 1,000,000 rows x 4 columns with one packed
  dense input column, direct-error block `=RC[-1]/0`, range aggregate
  `=SUM(RC[-2]:RC[-1])`, and recovery block `=IFERROR(SUM(RC[-3]:RC[-2]),RC[-3])`. It prepares
  three templates and three compiled R1C1 plans, evaluates 4,000,000 occupied cells exactly
  once, reports 8,001,434 authored bytes (`authored_bytes_per_cell_micros == 2000359`, about
  2.000 B/occupied cell), 3,000,000 formula cells, three formula-plan misses plus 2,999,997
  hits, three compiled-plan misses, four dense computed regions, 2,000,000 numeric-packed
  computed cells, zero logical-packed cells, zero sparse cells, sampled aggregate `Div0`
  values, recovered values 2/1000000/2000000, a zero-visit warm no-op, and a valid P-18
  partition witness. It passes P-00/P-10/P-11/P-14/P-18/P-19 plus
  `GRID-AGGREGATE-ERROR-R1C1-1M`.
  Unit test `optimized_grid_compact_oxfml_recalc_r1c1_range_aggregate_propagates_errors`
  covers the compact compiler floor for a dense formula-produced worksheet error flowing into
  a range aggregate and then through IFERROR recovery without sparse output.
- `w061-grid-scale-text-function-r1c1-1m-001`: 1,000,000 rows x 5 columns with one uniform
  dense text input column (`RowGrid`) and four repeated text-function R1C1 formula blocks:
  `=LEN(RC[-1])`, `=LEFT(RC[-2],3)`, `=RIGHT(RC[-3],4)`, and
  `=CONCAT(RC[-2],RC[-1])`. It prepares four templates and four compiled R1C1 plans, evaluates
  5,000,000 occupied cells exactly once, reports 1,830 authored bytes
  (`authored_bytes_per_cell_micros == 366`), 4,000,000 formula cells, four formula-plan misses
  plus 3,999,996 hits, four compiled-plan misses, five dense computed regions, 1,000,000
  numeric-packed computed cells, zero logical-packed cells, zero sparse cells, sampled
  `RowGrid`/`7`/`Row`/`Grid`/`RowGrid` values, a zero-visit warm no-op, and a valid P-18
  partition witness. It passes P-00/P-10/P-11/P-14/P-18/P-19 plus
  `GRID-TEXT-FUNCTION-R1C1-1M`.
  Unit test `optimized_grid_compact_oxfml_recalc_r1c1_text_functions_stay_dense` covers the
  compact compiler floor for LEN/LEFT/RIGHT/CONCAT over R1C1 reference arguments and chained
  formula-output references without sparse fallback.
- `w061-grid-scale-index-function-r1c1-1m-001`: 1,000,000 rows x 6 columns with two dense
  input columns and four repeated INDEX R1C1 formula blocks:
  `=INDEX(RC[-2]:RC[-1],1,1)`, `=INDEX(RC[-3]:RC[-2],1,2)`,
  `=INDEX(R1C1:RC1,ROW(),1)`, and `=INDEX(RC[-5]:RC[-4],2,1)`. It prepares four templates
  and four compiled R1C1 plans, evaluates 6,000,000 occupied cells exactly once, reports
  8,002,021 authored bytes (`authored_bytes_per_cell_micros == 1333671`, about 1.334
  B/occupied cell), 4,000,000 formula cells, four formula-plan misses plus 3,999,996 hits,
  four compiled-plan misses, six dense computed regions, 3,000,000 numeric-packed computed
  cells, zero logical-packed cells, zero sparse cells, sampled numeric/text/dynamic lookup
  values 10/Index/5000000/10000000, sampled `Ref` output for the out-of-range index, a
  zero-visit warm no-op, and a valid P-18 partition witness. It passes
  P-00/P-10/P-11/P-14/P-18/P-19 plus `GRID-INDEX-FUNCTION-R1C1-1M`.
  Unit test `optimized_grid_compact_oxfml_recalc_r1c1_index_function_stays_dense` covers the
  compact compiler floor for bounded positive-index INDEX over R1C1 ranges, including
  numeric/text lookup, dynamic `ROW()` selector lookup, and first-class `#REF!` output.
- `w061-grid-scale-match-function-r1c1-1m-001`: 1,000,000 rows x 6 columns with three dense
  numeric input columns and three repeated exact MATCH / nested INDEX-MATCH R1C1 formula blocks:
  `=MATCH(RC[-2],RC[-3]:RC[-1],0)`,
  `=INDEX(RC[-4]:RC[-2],1,MATCH(RC[-3],RC[-4]:RC[-2],0))`, and
  `=MATCH(999999999,RC[-5]:RC[-3],0)`. It prepares three templates and three compiled R1C1
  plans, evaluates 6,000,000 occupied cells exactly once, reports 24,001,540 authored bytes
  (`authored_bytes_per_cell_micros == 4000257`, about 4.000 B/occupied cell), 3,000,000
  formula cells, three formula-plan misses plus 2,999,997 hits, three compiled-plan misses,
  four dense computed regions, 5,000,000 numeric-packed computed cells, zero logical-packed
  cells, zero sparse cells, sampled exact-match position `2`, sampled nested INDEX/MATCH value
  `5000001`, sampled no-match `NA`, a zero-visit warm no-op, and a valid P-18 partition
  witness. It passes P-00/P-10/P-11/P-14/P-18/P-19 plus
  `GRID-MATCH-FUNCTION-R1C1-1M`.
  Unit test `optimized_grid_compact_oxfml_recalc_r1c1_match_function_stays_dense` covers the
  compact compiler floor for exact `MATCH(...,0)` over finite R1C1 ranges, nested
  `INDEX(...,MATCH(...),...)`, and first-class `#N/A` output.
- `w061-grid-scale-vlookup-function-r1c1-1m-001`: 1,000,000 rows x 7 columns with three dense
  input columns and four repeated exact VLOOKUP R1C1 formula blocks:
  `=VLOOKUP(RC[-3],RC[-3]:RC[-1],2,FALSE)`,
  `=VLOOKUP(RC[-4],RC[-4]:RC[-2],3,0)`,
  `=VLOOKUP(999999999,RC[-5]:RC[-3],2,FALSE)`, and
  `=VLOOKUP(RC[-6],RC[-6]:RC[-4],4,FALSE)`. It prepares four templates and four compiled
  R1C1 plans, evaluates 7,000,000 occupied cells exactly once, reports 16,002,254 authored
  bytes (`authored_bytes_per_cell_micros == 2286037`, about 2.286 B/occupied cell),
  4,000,000 formula cells, four formula-plan misses plus 3,999,996 hits, four compiled-plan
  misses, seven dense computed regions, 3,000,000 numeric-packed computed cells, zero
  logical-packed cells, zero sparse cells, sampled text/numeric/no-match/out-of-range values
  `Lookup`/`50000000`/`NA`/`Ref`, a zero-visit warm no-op, and a valid P-18 partition witness.
  It passes P-00/P-10/P-11/P-14/P-18/P-19 plus `GRID-VLOOKUP-FUNCTION-R1C1-1M`.
  Unit test `optimized_grid_compact_oxfml_recalc_r1c1_vlookup_function_stays_dense` covers the
  compact compiler floor for exact `VLOOKUP(...,FALSE|0)` over finite R1C1 ranges, including
  dense text/numeric output plus first-class `#N/A` and `#REF!` output.
- `w061-grid-scale-plan-cache-rounds-1m-001`: the same 1,000,000-row x 10-column dense plus
  repeated-R1C1 shape runs three dirty compact OxFml recalc rounds against one persistent
  `GridOptimizedFormulaPlanCache`. Each round sees 2,000,000 formula lookups; round 1 records
  one miss and 1,999,999 hits, rounds 2 and 3 record zero misses and 2,000,000 hits each,
  for 6,000,000 total lookups, 5,999,999 hits, one total miss, and one cached template. The same
  run records one cached compiled R1C1 plan object, one compiled-plan miss in round 1, zero
  later compiled-plan misses, and two later compiled-plan hits. It passes P-14 plus
  `GRID-PLAN-CACHE-ROUNDS-1M`.
  Unit test `optimized_grid_formula_plan_cache_recompiles_stale_fingerprint_and_prunes_unused_plans`
  adds the first compiled-plan lifecycle floor: a stale cached normal-form key with changed
  formula source/channel fingerprint recompiles instead of reusing the old plan, and replacing
  the formula region with literals prunes the cached template and compiled plan.
- `w061-grid-scale-insert-storm-1m-001`: six row insert/delete edits over a compact
  1,000,000-row x 10-column dense/repeated-formula model. The run starts with 9,999,920
  authored cells, visits 42 compact region metadata records, compares that against a
  59,999,520-cell naive rewrite floor (`compact_metadata_touch_ratio_micros == 0` after
  integer rounding), drops only the 30 cells in the three deleted rows, keeps final dense and
  repeated formula storage as seven compact regions each, materializes zero sparse points,
  records `blank_cell_bytes == 0`, and passes P-10/P-17 plus `GRID-INSERT-STORM-1M`.
- `w061-grid-scale-publication-delta-1m-001`: two 1,000,000-row optimized valuations compare
  a one-cell dense input change plus its dependent repeated-R1C1 output. The delta report sees
  two changed dense region entries, zero sparse entries, zero spill fact entries,
  `publication_entries_total == 2`, and `publication_entry_ratio_micros == 1` against
  `naive_current_computed_cell_publication_floor == 2000000`. The run passes P-22 plus
  `GRID-PUBLICATION-DELTA-1M`.
- `w061-grid-scale-range-invalidation-1m-001`: a 1,000,000-row finite range dependency is
  installed into GridInvalidation-Ref with `installed_range_scalar_edges == 0`,
  `compressed_range_edges == 1`, `expanded_scalar_edge_floor == 1000000`, and only one ordinary
  downstream scalar edge. A seed at row 500,000 produces a dirty closure of size 3
  (seed + range formula + downstream dependent), and the run passes P-13 plus
  `GRID-RANGE-INVALIDATION-1M`.
- `w061-grid-scale-range-query-1m-001`: 1,000 compressed finite range dependencies cover
  1,000,000 rows. A seed at row 500,501 checks `indexed_candidate_count == 2` instead of the
  `naive_candidate_floor == 1000`, matches one dependent, and produces a dirty closure of size
  3. The run passes P-12 plus `GRID-RANGE-QUERY-1M`.
- `w061-grid-scale-sum-pyramid-1m-001`: a 1,000,000-row aggregation pyramid installs 1,111
  compressed range edges over 1,000 level-1 ranges, 100 level-2 ranges, 10 level-3 ranges, and
  one final range. Those edges stand in for `expanded_range_edge_floor == 3700111` with
  `installed_range_scalar_edges == 0`, one downstream scalar edge, `indexed_candidate_sum == 20`
  across the selected leaf-to-root chain, and `dirty_closure_size == 6`. The run passes P-12,
  P-13, and `GRID-SUM-PYRAMID-1M`.
- `w061-grid-scale-dirty-rect-1m-001`: a 1,000,000-row dirty-rectangle invalidation lane
  installs 1,000 compressed range consumers plus 1,000 scalar consumers and one downstream
  scalar chain. An 11-cell dirty rectangle checks `indexed_candidate_sum == 3` across
  `total_edge_count == 2002`, with `indexed_compressed_range_candidate_count == 2`,
  `indexed_scalar_candidate_count == 1`, `matched_compressed_range_dependent_count == 1`,
  `matched_scalar_dependent_count == 1`, and `dirty_closure_size == 3`. The run passes P-12
  plus `GRID-DIRTY-RECT-1M`.
- `w061-grid-scale-hide-storm-1m-001`: 1,000 hidden-sensitive AxisState row-band dependencies
  cover 1,000,000 rows. A hidden-row seed at row 500,501 checks
  `indexed_candidate_count == 2` instead of the `naive_candidate_floor == 1000`, matches one
  aggregate formula, and produces a dirty closure of size 2 (aggregate + downstream). The run
  passes P-24 plus `GRID-HIDE-STORM-1M`.
- `w061-grid-scale-spill-anchor-1m-001`: a 1,000,000-row `A1#` spill extent is resolved by the
  grid provider with `spill_ledger_probe_count == 1`,
  `spill_extent_cells_scanned_for_ledger == 0`, `provider_value_entries_scanned == 3`, and
  `defined_cells_returned == 3`, passing P-25 plus `GRID-SPILL-ANCHOR-1M`.
- `w061-grid-scale-filter-spill-1m-006`: a 1,000,000-row old spill extent is cleared through
  the optimized sparse-value index before a smaller re-spill sample is published, and a real
  `FILTER(A1:B1000000,C1:C1000000)` formula publishes a 500,000-row by 2-column dense output.
  The run
  reports `naive_sparse_value_scan_floor == 1003`, `indexed_clear_candidate_count == 3`,
  `sparse_values_removed == 3`, `sparse_values_after_clear == 1000`,
  `respill_sparse_cells_touched == 6` against `grid_cell_capacity == 5000000`,
  `filter_formula_spill_extent_declared_cells == 1000000`,
  `filter_formula_spill_extent_declared_rows == 500000`,
  `filter_formula_spill_extent_declared_cols == 2`,
  `filter_formula_spill_ghost_cells_published == 999999`,
  `filter_formula_computed_dense_value_regions == 3`,
  `filter_formula_computed_dense_cells == 4000000`,
  `filter_formula_computed_dense_numeric_packed_cells == 3000000`,
  `filter_formula_computed_sparse_cells == 0`,
  `filter_formula_spill_commit_committed_fact_entries == 1`,
  `filter_formula_spill_commit_anchors_added == 1`,
  `filter_formula_spill_commit_current_epoch_anchors == 1`, sampled FILTER row pairs
  101/102, 25000001/25000002, and 50000001/50000002, and empty vacated values after the
  contracted extent. The same run covers a column-mask FILTER over a 999,999-row by 3-column
  source with horizontal include row: `column_filter_spill_extent_declared_cells == 1999998`,
  `column_filter_spill_extent_declared_rows == 999999`,
  `column_filter_spill_extent_declared_cols == 2`,
  `column_filter_computed_sparse_cells == 0`,
  `column_filter_spill_commit_committed_fact_entries == 1`, and sampled selected-column values
  101/103, 49999901/49999903, and 99999901/99999903, passing P-23 plus
  `GRID-FILTER-SPILL-1M` and `GRID-FILTER-COLUMN-SPILL-1M`. The same run applies one sparse
  mask override, performs a second committed optimized FILTER recalc, shrinks the output to
  499,999 rows by 2 columns (`filter_lifecycle_spill_extent_declared_cells == 999998`), records
  `filter_lifecycle_spill_commit_extent_changed_anchors == 1`,
  `filter_lifecycle_spill_commit_value_changed_anchors == 1`,
  `filter_lifecycle_committed_value_epoch == 2`, and passes `GRID-FILTER-LIFECYCLE-1M`.
- `w061-grid-scale-sequence-spill-1m-002`: a `SEQUENCE(1000000)` dynamic-array formula
  publishes one dense computed region with `computed_dense_cells == 1000000`,
  `computed_dense_numeric_packed_cells == 1000000`, `computed_sparse_cells == 0`,
  `spill_ghost_cells_published == 999999`, and sampled values 1/500000/1000000. The same run
  commits the optimized publication back to sheet state with
  `spill_commit_committed_fact_entries == 1`, `spill_commit_anchors_added == 1`,
  `spill_commit_current_epoch_anchors == 1`, and
  `sheet_committed_spill_fact_entries == 1`, passing P-23 plus `GRID-SEQUENCE-SPILL-1M`.
- `w061-grid-scale-spill-blockage-1m-001`: a 1,000,000-row intended spill extent is checked by
  compact blocker probes. The empty extent reports `empty_compact_blocker_probe_count == 0`
  versus `empty_naive_extent_cell_probe_floor == 1000000`; the far-blocker leg finds a sparse
  blocker at row 1,000,000 with `far_blocker_compact_blocker_probe_count == 1`,
  `far_blocker_probe_ratio_micros == 1`, and `far_blocker_blocked == true`, passing P-26 plus
  `GRID-SPILL-BLOCKAGE-1M`.
- `w061-grid-scale-spill-epoch-1m-003`: a 1,000,000-row `A1#` spill extent is compared through
  `GridSpillEpochLedger` snapshots. Unchanged snapshots compare two anchors, preserve both
  epochs, and dirty zero consumers; unrelated value-epoch churn changes one non-dependent
  anchor and still dirties zero consumers; A1 value-epoch and extent-epoch changes each dirty
  exactly the `A1#` consumer plus downstream cell (`value_dirty_closure_size == 2`,
  `extent_dirty_closure_size == 2`). The same run now records optimized spill-state commit
  counters (`optimized_spill_commit_first_added == 1`,
  `optimized_spill_commit_second_preserved == 1`, `optimized_spill_commit_extent_changed == 1`),
  passing P-27 plus `GRID-SPILL-EPOCH-1M`.
- `w061-grid-scale-aggregate-context-1m-001`: a 1,000,000-row `SUBTOTAL`-style aggregate context
  report visits `axis_run_probe_count == 5` row-context runs (two explicit AxisState rows and
  three default runs) instead of doing one AxisState row lookup per referenced cell. The current
  OxFunc host-info seam still requires `per_cell_context_expansion_count == 1000000`, which is
  recorded explicitly, and the run passes P-28 plus `GRID-AGGREGATE-CONTEXT-1M`.
- `w061-grid-scale-tile-stream-64k-001`: a 320 x 200 visible tile over a 1,000,000-row x
  320-column model streams only the subscribed 64,000 cells. The run reports
  `estimated_frame_bytes == 1536159`, `estimated_value_payload_bytes == 512000`,
  `frame_bytes_per_subscribed_cell_micros == 24002485` (about 24.003 bytes/cell),
  `dense_value_cells_visited == 64000`, `sparse_value_cells_visited == 0`,
  `compact_regions_intersected == 1`, `unrelated_sparse_cells == 1000`,
  `full_grid_cell_floor == 320000000`, and `max_frame_bytes_per_subscribed_cell == 64`,
  passing P-15 plus `GRID-TILE-STREAM-64K`.
- `w061-grid-scale-viewport-64k-of-1m-001`: a 64,000-cell visible formula column over a
  1,000,000-row x 10-column dense/repeated-R1C1 model is evaluated through the optimized
  visible-first path. The visible upstream cone is rows 468,000..531,999 and columns 8..10:
  64,000 dense input cells plus 128,000 repeated formula cells. The run reports
  `visible_cell_count == 64000`, `visible_upstream_cell_count == 192000`,
  `cells_evaluated_before_visible_complete == 192000`,
  `formula_evaluations_before_visible_complete == 128000`,
  `full_recalc_occupied_cell_floor == 10000000`,
  `visible_eval_to_full_occupied_ratio_micros == 19200`, `computed_dense_value_regions == 2`,
  `computed_sparse_cells == 0`, `snapshot_defined_cells == 64000`,
  `snapshot_sparse_value_cells_visited == 0`, and bottom visible value `2127996032`, passing
  P-16 plus `GRID-VIEWPORT-64K`.
- `w061-grid-scale-cow-retention-1m-001`: seven retained compact roots (initial plus six
  row insert/delete edits) share one dense numeric payload across the edited history. The run
  reports `retained_revision_count == 7`, `unique_dense_payloads == 1`,
  `unique_dense_payload_bytes == 63999488`, `cow_retained_bytes == 64012633`,
  `naive_full_snapshot_retention_bytes_floor == 448009561`,
  `retained_to_naive_ratio_micros == 142883` (about 14.288%),
  `compact_region_metadata_touches == 42`, `retained_compact_regions == 56`,
  `sparse_point_cells_final == 0`, and `blank_cell_bytes_final == 0`, passing P-21 plus
  `GRID-COW-RETENTION-1M`.

This is still partial scale evidence. P-10's dense/repeated authored-storage floor, the
1M sparse-singleton and full-width `zig-zag-1M` byte legs, the named `fill-down-1M` and
`pascal-r1c1-1M` R1C1 lanes, the full `boring-1Mx10` authored-storage/template/warm-no-op lane,
the `direct-r1c1-1M` direct scalar formula-output lane, the `unary-r1c1-1M` unary scalar
formula-output lane, the `argument-aggregate-r1c1-1M` aggregate argument-list formula-output lane,
the `math-function-r1c1-1M` scalar math-function formula-output lane,
the `mod-function-r1c1-1M` MOD scalar-function formula-output lane,
the `rounding-function-r1c1-1M` ROUND-family formula-output lane,
the `integer-function-r1c1-1M` INT/TRUNC formula-output lane,
the `log-function-r1c1-1M` EXP/LN/LOG formula-output lane,
the `trig-function-r1c1-1M` SIN/COS/TAN formula-output lane,
the `angle-function-r1c1-1M` RADIANS/DEGREES/PI formula-output lane,
the `reference-function-r1c1-1M` ROW/COLUMN reference-identity formula-output lane,
the `logical-function-r1c1-1M` logical-function formula-output lane,
the `if-logical-r1c1-1M` IF logical-condition formula-output lane,
the `two-left-r1c1-1M` same-row two-input formula-output lane, the `absolute-r1c1-1M`
mixed absolute/relative R1C1 lane, the `division-r1c1-1M`
numeric division lane, the `decimal-r1c1-1M` decimal-literal multiplication lane, the
`recursive-binary-r1c1-1M` precedence/parenthesized arithmetic lane, the
`if-r1c1-1M` numeric conditional lane, the `if-branch-r1c1-1M` arithmetic branch-expression
lane, the `nested-if-r1c1-1M` nested scalar conditional lane, the `iferror-r1c1-1M` worksheet-error recovery lane,
the `comparison-r1c1-1M` logical formula-output lane, the
`comparison-expression-r1c1-1M` scalar-expression comparison lane, the
`comparison-iferror-r1c1-1M` nested IFERROR comparison lane, the `sum-row-r1c1-1M` narrow row-range SUM lane, the `sumsq-row-r1c1-1M` narrow row-range
SUMSQ lane, the `count-row-r1c1-1M` narrow row-range
COUNT lane, the `product-row-r1c1-1M` narrow row-range PRODUCT lane, the `average-row-r1c1-1M` narrow row-range
AVERAGE lane, the `min-max-row-r1c1-1M` paired row-range MIN/MAX lane, the
`sum-window-r1c1-1M` vertical SUM-window lane, the `division-error-r1c1-1M` direct `Div0` lane, the
`division-error-propagation-r1c1-1M` direct-plus-propagated `Div0` lane, and the
`aggregate-error-r1c1-1M` range-aggregate error/IFERROR recovery lane, the
`text-function-r1c1-1M` text-function lane over R1C1 reference arguments, the
`index-function-r1c1-1M` bounded positive-index INDEX lane over R1C1 ranges, and the
`match-function-r1c1-1M` exact MATCH / nested INDEX-MATCH lane over finite R1C1 ranges, and the
`vlookup-function-r1c1-1M` exact VLOOKUP lane over finite R1C1 ranges are now bound by
retained scale artifacts. Dense computed formula-output storage is bound for the retained repeated-R1C1
templates, and the optimized compiled-plan path now supports direct scalar R1C1 references,
parenthesized direct scalar R1C1 references, unary-minus scalar R1C1 expressions, a bounded recursive binary R1C1
expression class over relative and absolute refs, finite numeric literals, arithmetic
precedence, parenthesized subexpressions, `+`, `-`, `*`, and `/`, a
comparison R1C1 class over operands, scalar expressions, and nested IFERROR, a numeric IF R1C1 conditional class with comparison/logical-function conditions and nested scalar branch
expressions, an IFERROR R1C1 error-recovery class, aggregate argument-list R1C1 calls over
bounded scalar expressions and finite R1C1 ranges, scalar math functions `ABS`, `SQRT`,
`POWER`, and `MOD`, plus `ROUND`, `ROUNDUP`, `ROUNDDOWN`, `INT`, `TRUNC`, `EXP`, `LN`,
`LOG10`, `LOG`, `SIN`, `COS`, `TAN`, `RADIANS`, `DEGREES`, and `PI`, over bounded scalar expressions,
reference-identity functions `ROW`, `COLUMN`, `ROWS`, and `COLUMNS` over current cell, R1C1
references, and finite R1C1 ranges,
bounded logical functions `AND`, `OR`, and `NOT` over
comparison/logical expressions, plus
text functions `LEN`, `LEFT`, `RIGHT`, and `CONCAT` over R1C1 reference arguments, plus
bounded positive-index `INDEX` over finite R1C1 ranges, plus
exact `MATCH(...,0)` over finite one-dimensional R1C1 ranges, plus
exact `VLOOKUP(...,FALSE|0)` over finite R1C1 ranges, plus
narrow
`SUM(<R1C1 ref>:<R1C1 ref>)`,
`SUMSQ(<R1C1 ref>:<R1C1 ref>)`,
`COUNT(<R1C1 ref>:<R1C1 ref>)`,
`PRODUCT(<R1C1 ref>:<R1C1 ref>)`,
`AVERAGE(<R1C1 ref>:<R1C1 ref>)`,
`MIN(<R1C1 ref>:<R1C1 ref>)`, and
`MAX(<R1C1 ref>:<R1C1 ref>)`, including direct `Div0` output for division by
zero, single upstream worksheet-error propagation, and the retained range-aggregate
worksheet-error/IFERROR recovery slice. The fully occupied `full-column-1M`
P-20 lane is now bound,
P-17 has a compact structural-edit scale floor for row insert/delete over dense and
repeated-formula regions, and P-18 has compact partition-witness evidence for sparse singleton
and dense/repeated-formula layouts. P-12 now has retained compressed-range indexed-query,
dirty-rectangle, and sum-pyramid indexed-candidate floors, P-13 has retained finite-range and sum-pyramid compressed
reverse-edge floors, P-14 has primary-pass
template lookup hit/miss counters for the retained repeated-R1C1 lanes plus a persistent
compiled R1C1 formula-plan object round floor and stale-fingerprint/prune lifecycle unit floor,
P-23 has retained
old-spill indexed clear and dense SEQUENCE/FILTER dynamic-array committed-publication floors.
P-24 has a retained
AxisState visibility block-index floor, P-25 has a retained 1M spill-ledger probe floor, and
P-26 has a retained optimized compact blocker-probe floor, P-27 has a retained reference
dirty-closure plus first in-engine spill-epoch ledger and optimized spill-state commit floor, and P-28 has a retained provider-side row-run context report
floor, P-15 has a retained 64K tile-stream frame/readout floor, P-16 has a retained 64K
visible-first same-row R1C1 upstream-cone floor, and P-21 has a retained shared-payload
COW-root floor. The broad retained workload matrix, broader
formula-output functions outside direct/unary scalar R1C1, aggregate argument-list R1C1,
scalar math-function R1C1, MOD scalar-function R1C1, ROUND-family scalar-function R1C1, INT/TRUNC scalar-function R1C1, EXP/LN/LOG scalar-function R1C1, SIN/COS/TAN scalar-function R1C1, RADIANS/DEGREES/PI scalar-function R1C1, reference-function R1C1, logical-function R1C1, IF logical-condition R1C1,
bounded recursive binary R1C1, comparison R1C1 with scalar operands and nested IFERROR, numeric IF with
nested bounded scalar branch expressions, IFERROR R1C1, text-function R1C1 over reference arguments,
bounded positive-index INDEX over finite R1C1 ranges, exact MATCH over finite one-dimensional R1C1
ranges, exact VLOOKUP over finite R1C1 ranges, and narrow
SUM/SUMSQ/COUNT/PRODUCT/AVERAGE/MIN/MAX-range classes plus the retained aggregate-error range/IFERROR lane, broader multi-error precedence beyond that lane and broader
non-arithmetic worksheet-error algebra inside dense formula-output regions, broader compiled-plan eviction policy/version lifecycle, production COW retention GC/lifecycle,
production publication lifecycle, production invalidation equivalence,
broader hide/filter/outline visibility provenance, run-compressed OxFunc host-info packets,
the broader FILTER value-dependent spill matrix, broad spill arbitration, production
broad publication lifecycle, production host tile protocol, production viewport scheduling, parallel execution, and Excel comparison legs are not yet bound.

Descriptors retained, fixtures generated (treecalc-scale precedent — no million-cell fixtures
in git). Engine workloads under `OxCalc/docs/test-corpus/grid-perf/`; host/session workloads
(doom, viewport, intent replays) under `DnaTreeCalc/docs/test-corpus/perf/` (directory named
in doctrine; to be created).

| id | shape | external comparison |
|---|---|---|
| boring-1Mx10 | 1M rows × 10 cols dense values+formulas | Excel ~37 B/cell, 0.57 s full recalc (candid-startup benchmark) |
| zig-zag-1M | 1M isolated diagonal singletons (worst block fragmentation) | — |
| full-column-1M | one column fully occupied, `SUM(A:A)` consumers | — |
| sum-pyramid-1M | 1000/100/10/1 aggregation pyramid (Sestoft §3.3 O(N²) support blowup case) | — |
| deep-chain-N | sequential dependency chain (same shape as the calc-ekq3 spike chain — keeps rounds comparable) | spike baselines |
| fill-down-1M | one template region, `=R[-1]C+1` style | — |
| pascal-r1c1-1M | one 2D repeated R1C1 recurrence region, `=RC[-1]+R[-1]C`, with dense/sparse boundaries | — |
| direct-r1c1-1M | two repeated direct scalar R1C1 regions, `=RC[-1]` and `=(RC[-2])`, with dense input and formula-output columns | retained dense formula-output floor |
| unary-r1c1-1M | three repeated unary scalar R1C1 regions, `=-RC[-1]`, `=-(RC[-2]+5)`, and `=-RC[-3]*2+1`, with dense input and formula-output columns | retained dense formula-output floor |
| argument-aggregate-r1c1-1M | six repeated aggregate argument-list R1C1 regions, `=SUM(RC[-2],RC[-1],5)`, `=COUNT(RC[-3],RC[-2],5)`, `=PRODUCT(RC[-4],RC[-3],2)`, `=AVERAGE(RC[-5],RC[-4],5)`, `=MIN(RC[-6],RC[-5],5)`, and `=MAX(RC[-7],RC[-6],5)`, with two dense input columns and dense formula-output columns | retained dense formula-output floor |
| math-function-r1c1-1M | three repeated scalar math-function R1C1 regions, `=ABS(RC[-2])`, `=SQRT(RC[-2])`, and `=POWER(ABS(RC[-4]),2)`, with two dense input columns and dense formula-output columns | retained dense formula-output floor |
| mod-function-r1c1-1M | three repeated MOD scalar-function R1C1 regions, `=MOD(RC[-2],RC[-1])`, `=IF(MOD(RC[-3],2)=0,RC[-3]/2,RC[-3]*3)`, and `=MOD(POWER(RC[-4],2),RC[-3])`, with two dense input columns and dense formula-output columns | retained dense formula-output floor |
| rounding-function-r1c1-1M | three repeated ROUND-family R1C1 regions, `=ROUND(RC[-2],RC[-1])`, `=ROUNDUP(RC[-3],RC[-2])`, and `=ROUNDDOWN(RC[-4],RC[-3])`, with two dense input columns and dense formula-output columns | retained dense formula-output floor |
| integer-function-r1c1-1M | three repeated INT/TRUNC R1C1 regions, `=INT(RC[-2])`, `=TRUNC(RC[-3])`, and `=TRUNC(RC[-4],RC[-3])`, with two dense input columns and dense formula-output columns | retained dense formula-output floor |
| log-function-r1c1-1M | four repeated EXP/LN/LOG R1C1 regions, `=EXP(RC[-1])`, `=LN(RC[-3])`, `=LOG10(RC[-4]*100)`, and `=LOG(RC[-5]*100,10)`, with two dense input columns and dense formula-output columns | retained dense formula-output floor |
| trig-function-r1c1-1M | three repeated SIN/COS/TAN R1C1 regions, `=SIN(RC[-1])`, `=COS(RC[-2])`, and `=TAN(RC[-3])`, with one dense input column and dense formula-output columns | retained dense formula-output floor |
| angle-function-r1c1-1M | four repeated angle-conversion R1C1 regions, `=RADIANS(RC[-2])`, `=DEGREES(RC[-2])`, `=SIN(RADIANS(RC[-4]))`, and `=PI()`, with two dense input columns and dense formula-output columns | retained dense formula-output floor |
| reference-function-r1c1-1M | six repeated reference-identity R1C1 regions, `=ROW()`, `=COLUMN()`, `=ROW(RC[-2])`, `=COLUMN(RC[-3])`, `=ROWS(R1C1:R3C1)`, and `=COLUMNS(RC[-5]:RC[-3])`, with formula-only dense output | retained dense formula-output floor |
| logical-function-r1c1-1M | three repeated logical-function R1C1 regions, `=AND(RC[-2]>0,RC[-1]>0)`, `=OR(RC[-3]>0,RC[-2]>0)`, and `=NOT(AND(RC[-4]>0,RC[-3]>0))`, with two dense signed input columns and dense logical formula-output columns | retained dense formula-output floor |
| if-logical-r1c1-1M | three repeated numeric IF regions with logical-function conditions, `=IF(AND(RC[-2]>0,RC[-1]>0),RC[-2]+RC[-1],0)`, `=IF(OR(RC[-3]>0,RC[-2]>0),RC[-3]-RC[-2],0)`, and `=IF(NOT(AND(RC[-4]>0,RC[-3]>0)),ABS(RC[-4]),0)`, with two dense signed input columns and dense numeric formula-output columns | retained dense formula-output floor |
| two-left-r1c1-1M | one repeated R1C1 same-row two-input region, `=RC[-2]+RC[-1]`, with dense inputs and formula-output columns | retained dense formula-output floor |
| absolute-r1c1-1M | one repeated mixed absolute/relative R1C1 region, `=RC[-1]+R1C1`, with dense inputs and formula-output column | retained dense formula-output floor |
| division-r1c1-1M | one repeated R1C1 numeric-safe division region, `=RC[-1]/2`, with dense inputs and formula-output column | retained dense formula-output floor |
| decimal-r1c1-1M | one repeated R1C1 decimal-literal multiplication region, `=RC[-1]*0.5`, with dense inputs and formula-output column | retained dense formula-output floor |
| recursive-binary-r1c1-1M | two repeated R1C1 recursive binary arithmetic regions, `=RC[-3]+RC[-2]*RC[-1]` and `=(RC[-4]+RC[-3])*RC[-2]`, with dense inputs and formula-output columns | retained dense formula-output floor |
| if-r1c1-1M | one repeated R1C1 numeric IF region, `=IF(RC[-1]>0,RC[-1],0)`, with dense signed inputs and dense formula-output column | retained dense formula-output floor |
| if-branch-r1c1-1M | one repeated R1C1 numeric IF branch-expression region, `=IF(RC[-1]>0,RC[-1]*2,RC[-1]/2)`, with dense signed inputs and dense formula-output column | retained dense formula-output floor |
| nested-if-r1c1-1M | one repeated R1C1 nested IF region, `=IF(RC[-2]>500000,IF(RC[-1]>0,RC[-2]*2,RC[-2]*3),IF(RC[-1]>0,RC[-2]+1,RC[-2]-1))`, with dense selector/input columns and dense formula-output column | retained dense formula-output floor |
| iferror-r1c1-1M | one repeated R1C1 IFERROR region, `=IFERROR(RC[-2]/RC[-1],0)`, with dense numerator/denominator inputs and dense formula-output column | retained dense formula-output floor |
| comparison-r1c1-1M | one repeated R1C1 comparison region, `=RC[-1]>0`, with dense signed inputs and dense logical formula-output column | retained dense formula-output floor |
| comparison-expression-r1c1-1M | one repeated R1C1 scalar-expression comparison region, `=RC[-2]*2>RC[-1]+1`, with dense signed inputs and dense logical formula-output column | retained dense formula-output floor |
| comparison-iferror-r1c1-1M | one repeated R1C1 nested IFERROR comparison region, `=IFERROR(RC[-2]/RC[-1],0)>0`, with dense signed inputs and dense logical formula-output column | retained dense formula-output floor |
| sum-row-r1c1-1M | one repeated R1C1 row-range aggregate region, `=SUM(RC[-3]:RC[-1])`, with three dense input columns and dense formula-output column | retained dense formula-output floor |
| sumsq-row-r1c1-1M | one repeated R1C1 row-range aggregate region, `=SUMSQ(RC[-3]:RC[-1])`, with three dense input columns and dense formula-output column | retained dense formula-output floor |
| count-row-r1c1-1M | one repeated R1C1 row-range aggregate region, `=COUNT(RC[-3]:RC[-1])`, with three dense input columns and dense formula-output column | retained dense formula-output floor |
| product-row-r1c1-1M | one repeated R1C1 row-range aggregate region, `=PRODUCT(RC[-3]:RC[-1])`, with three dense input columns and dense formula-output column | retained dense formula-output floor |
| average-row-r1c1-1M | one repeated R1C1 row-range aggregate region, `=AVERAGE(RC[-3]:RC[-1])`, with three dense input columns and dense formula-output column | retained dense formula-output floor |
| min-max-row-r1c1-1M | two repeated R1C1 row-range aggregate regions, `=MIN(RC[-3]:RC[-1])` and `=MAX(RC[-4]:RC[-2])`, with three dense input columns and dense formula-output columns | retained dense formula-output floor |
| sum-window-r1c1-1M | one repeated R1C1 vertical sliding-window aggregate region, `=SUM(R[-2]C[-1]:RC[-1])`, with dense input column and dense formula-output column | retained dense formula-output floor |
| division-error-r1c1-1M | one repeated R1C1 direct division-error region, `=RC[-1]/0`, with dense inputs and dense `Div0` formula-output column | retained dense formula-output floor |
| division-error-propagation-r1c1-1M | two repeated R1C1 regions, `=RC[-1]/0` then `=RC[-1]+1`, with dense direct and propagated `Div0` formula-output columns | retained dense formula-output floor |
| aggregate-error-r1c1-1M | three repeated R1C1 regions, `=RC[-1]/0`, `=SUM(RC[-2]:RC[-1])`, and `=IFERROR(SUM(RC[-3]:RC[-2]),RC[-3])`, with dense aggregate-error and recovered formula-output columns | retained dense formula-output floor |
| text-function-r1c1-1M | four repeated text-function R1C1 regions, `=LEN(RC[-1])`, `=LEFT(RC[-2],3)`, `=RIGHT(RC[-3],4)`, and `=CONCAT(RC[-2],RC[-1])`, with dense text input and dense text/numeric formula-output columns | retained dense formula-output floor |
| index-function-r1c1-1M | four repeated INDEX R1C1 regions over finite ranges, including dynamic `ROW()` selector and out-of-range `#REF!`, with dense input and dense formula-output columns | retained dense formula-output floor |
| match-function-r1c1-1M | three repeated exact MATCH / nested INDEX-MATCH R1C1 regions, `=MATCH(RC[-2],RC[-3]:RC[-1],0)`, `=INDEX(RC[-4]:RC[-2],1,MATCH(RC[-3],RC[-4]:RC[-2],0))`, and `=MATCH(999999999,RC[-5]:RC[-3],0)`, with dense input and dense numeric/`#N/A` formula-output columns | retained dense formula-output floor |
| vlookup-function-r1c1-1M | four repeated exact VLOOKUP R1C1 regions, `=VLOOKUP(RC[-3],RC[-3]:RC[-1],2,FALSE)`, `=VLOOKUP(RC[-4],RC[-4]:RC[-2],3,0)`, `=VLOOKUP(999999999,RC[-5]:RC[-3],2,FALSE)`, and `=VLOOKUP(RC[-6],RC[-6]:RC[-4],4,FALSE)`, with dense input and dense text/numeric/`#N/A`/`#REF!` formula-output columns | retained dense formula-output floor |
| plan-cache-rounds-1M | boring-1Mx10 shape over three dirty recalc rounds sharing one formula-plan cache and cached compiled R1C1 plan object | retained P-14 persistent compiled-plan floor |
| flash-fill-region | host-declared never-materialized region (owner decision: second authorship mode) | — |
| enron-mix | generator parameterized to ~4.5% unique-formula ratio | corpus statistics |
| insert-storm-1M | repeated row insert/delete across regions and block boundaries | — |
| publication-delta-1M | one dense input edit plus dependent repeated-formula dense output publication delta | — |
| cow-retention-1M | retained roots across insert/delete storm share dense payloads instead of retaining full snapshots | retained COW-root floor |
| range-invalidation-1M | one 1M-row finite range dependency plus downstream scalar dependent | — |
| range-query-1M | 1000 compressed finite ranges over 1M rows plus one downstream scalar dependent | — |
| dirty-rect-1M | 11-cell dirty rectangle over scalar and compressed range consumers in a 1M-row sheet | — |
| spill-anchor-1M | one 1M-row `A1#` extent with sparse stored values | — |
| spill-blockage-1M | one 1M-row intended spill extent, empty leg plus far sparse blocker leg | — |
| spill-epoch-1M | one 1M-row `A1#` extent with unchanged, unrelated, value, and extent epoch comparisons | — |
| aggregate-context-1M | one 1M-row aggregate context reference over two hidden AxisState rows | — |
| sequence-spill-1M | one 1M-row `SEQUENCE()` dynamic-array output published as computed storage | — |
| viewport-64k-of-1M | 64K visible formula cells plus same-row R1C1 upstream cone over a 1M-row dirty model | retained visible-first floor |
| doom-320x200 / tile-stream-64K | cell=pixel, 64k subscribed tile cells changing per tick at 30–60 Hz | retained 64K tile frame/readout floor |
| filter-spill-1M | FILTER() anchor whose result size changes per edit (re-spill old∪new cost) | — |
| spill-storm | many anchors + `A1#` consumers; extent churn + blockage/clearance cycles | — |
| hide-storm | boring-1Mx10 + k SUBTOTAL footers + running `SUBTOTAL(103,…)` probes; random hide/unhide/filter toggles | — |

## 6. Cadence and gates

- **Per PR:** counter gates at smoke scale for rows the PR touches, plus the ≤1e5-cell
  differential (`--engine both`). Engine rows gate in OxCalc CI; host rows in DnaTreeCalc CI.
- **Per round** (calc-ekq3 model): retained full-scale acceptance run →
  `docs/test-runs/core-engine/grid-scale/BASELINE_<date>.md` in the established format
  (commit, machine, phase split, variance caveat, ref/optimized ratios).
- **Run artifacts:** the scale runner gains `counter_summary.json` and
  `register_assertions.json` (per touched row: bound, measured, pass/fail) beside the existing
  summaries; phase vocabulary extends with `block_build`, `template_prepare`,
  `rect_propagation`, `tile_publication`.

## 7. Open questions (owner)

1. One register spanning lanes (this file) vs per-lane registers — confirm; this moves
   calc-perf record-keeping here.
2. Named baseline machine + pinned toolchain for retained wall-clock runs, or ratio-only?
3. `CalcRunCounters` compiled into release permanently (recommended) vs feature-gated?
4. Host-row gates (tiles, replay) in DnaTreeCalc CI even when the code is in OxCalc?
5. Hand-author the boring-1Mx10 Excel workbook now for the §7.6 Excel-timing leg, or wait for
   the OxDoc export path?
