# W061: Strict Excel Grid Planning And Reference Floor

## Purpose

Promote the grid-extension planning set into OxCalc-owned implementation surfaces and create the first executable reference floor for `strict-excel-grid`.

## Depends on

- W047 CTRO design and positive-publication lessons.
- W050 plan-template/session direction.
- W060 host reference system and `ReferenceSystemProvider` shape.
- OxFml W077 for the generic `BindProfile`/reference-profile ABI, source-preserving formula
  syntax/bind lifecycle, symbolic references, and caller-independent normal-form identity.
- OxDoc bootstrap for `.xlsx` event-stream contracts.

## Seam guardrail

OxFml owns formula syntax, binding lifecycle, reference-expression plumbing, normal-form key
envelopes, source span/text preservation, and profile dispatch. OxCalc owns the
`strict-excel-grid` reference profile semantics and runtime provider: grid coordinates,
bounds/`#REF!`, A1/R1C1 coordinate interpretation, dependency emission, dereference,
spill-ledger lookup, hidden-row host context, and structural edit transforms. DnaTreeCalc
remains the non-grid proving profile for the same OxFml reference machinery.

The generalization target is **reference binding**, not grid behavior in OxFml. OxFml may ask
an active profile to recognize, bind, normalize, emit dependencies, prepare reference-visible
function arguments, transform under host edits, and render references. OxFml must not answer
what A1 means, how row deletion rewrites a range, how `A1#` dereferences, or how hidden rows
affect aggregate context.

## Spec surfaces

- `docs/spec/core-engine/CORE_ENGINE_REFERENCE_PROFILE_CONTRACT.md`
- `docs/spec/core-engine/CORE_ENGINE_GRID_MODEL.md`
- `docs/spec/core-engine/CORE_ENGINE_GRID_REFINEMENT_AND_EQUIVALENCE.md`
- `docs/spec/core-engine/CORE_ENGINE_GRID_REFERENCE_MACHINE.md`
- `docs/spec/core-engine/CORE_ENGINE_GRID_PERF_REGISTER.md`

## Closure condition

W061 closes when the grid semantic docs are indexed, GridCalc-Ref has a first BTreeMap
value/effects reference-machine implementation for bounds/R1C1/materialization cases,
GridInvalidation-Ref has a first scalar dirty-closure oracle for small dependency cases, the
differential harness can run `--engine reference|optimized|both` for a small grid corpus, the
perf register emits counter assertions for the first touched rows, and all `[verify-COM]`
spill/hidden-row claims have explicit OxXlPlay capture beads or blockers.

## Current reality and ordering correction

The current executable floor is ahead of the original "minimal A1" planning slice: it already
has GridCalc-Ref/GridInvalidation-Ref, an optimized storage candidate, seed and scale runners,
primary-pass template counters, same-sheet names, structured table overlays, spill-ledger
facts, hidden-row host context, whole-axis references, and a first row/column structural-edit
slice. That progress is OxCalc-owned evidence for the profile boundary.

It does **not** remove the seam gate. The next broadening step is to harden the generic
reference-profile ABI with OxFml W077, then prove it through DnaTreeCalc tree references and a
tiny fake profile. New grid breadth should either consume that acknowledged seam or be
explicitly labeled OxCalc-local evidence until the non-grid canary passes.

## Initial lanes

1. Contract hardening: keep `CORE_ENGINE_REFERENCE_PROFILE_CONTRACT.md`, the public grid
   packet/type glossary, source-text-vs-normal-form doctrine, value-oracle-vs-invalidation
   doctrine, and the first structural edit matrix aligned before broad grid behavior expands.
2. Tree/non-grid proof lane: W077 must keep DnaTreeCalc tree references flowing through the
   same reference-profile seam, with a fake second profile test before the grid profile becomes
   broad enough to bias the ABI.
3. GridCalc-Ref BTreeMap state and sampled readout.
4. GridInvalidation-Ref scalar dependency oracle for dirty-closure checks.
   Current executable floor also applies row/column structural edits to scalar cell/range and
   dynamic dependent descriptors, retaining semantic range intent so inserted cells inside a
   range participate in post-edit dirty closure.
5. GridOptimizedSheet authored-storage candidate: sparse points, dense value regions, repeated
   formula regions, and bounded projection to GridCalc-Ref.
6. Grid corpus seed: bounds, normal-form translation, materialization, insert/delete. The first
   engine-mode adapter is in-memory (`Reference`, `Optimized`, `Both`) over explicit probes, and
   `GridCorpusRunner` now parses the `reference|optimized|both` engine values for sparse,
   dense-values-only, dense-values-plus-aggregate, repeated-R1C1, hidden-row visibility,
   defined-name value/text resolution, table-overlay structured-reference value/text
   resolution, table omitted/current-row formula context, committed spill-ledger `A1#`,
   dynamic `SEQUENCE` spill, out-of-order spill-repair,
   mutual neighboring `SEQUENCE` spill blockage, table-overlay and merged-region spill blockage,
   and dynamic-invalidation seed cases, plus a post-structural-edit optimized dense/repeated-R1C1
   case. Its
   `execute_seed_corpus` path emits `docs/test-runs/core-engine/grid-seed/<run_id>` artifacts;
   `cargo run -p oxcalc-tracecalc-cli -- grid-seed <run-id> --engine reference|optimized|both`
   is the current CLI wrapper. Evidence run `w061-grid-seed-cli-009` passed 18 cases in `both`
   mode with zero value, differential, invalidation, and P-20 mismatches, including structural
   invalidation checks for shifted cell dependencies and inserted-row range expansion, hidden-row
   visibility dirty closure for a `SUBTOTAL(109,...)` aggregate chain, whole-row/whole-column
   formula evaluation with axis-indexed dirty closure plus optimized occupied-slot provider
   enumeration (`1:1` declared 32 cells and visited 2 occupied slots; `A:B` declared 256 cells
   and visited 2 occupied slots in the seed fixture), defined-name `SUM(InputRange)` and
   `SUM(INDIRECT("InputRange"))` value resolution through the grid provider plus finite
   named-range dirty closure, `namespace_lifecycle` artifacts for defined-name rename/delete,
   table-overlay `SUM(Table1[Amount])` and
   `SUM(INDIRECT("Table1[Amount]"))` value resolution through the grid provider plus finite
   table dirty closure plus `namespace_lifecycle` artifacts for table rename/resize/delete,
   repeated in-table `=[@Amount]*2` current-row formulas through
   OxFml's generic `TableCallerRegion` packet plus same-row scalar dirty closure, and
   repeated in-table `=SUM([Amount])` formulas through caller-local table-column resolution
   plus finite table-column dirty closure,
   section-qualified, escaped-column, and non-contiguous header/totals section-union
   structured-reference value/text resolution through the provider, spill-fact dirty closure for
   an `A1#` consumer chain,
   dynamic `SEQUENCE(3)` spill publication counters,
   bounded spill-repair counters for a `SUM(B1#)` consumer evaluated before its `B1` anchor,
   blocked-spill counters for table-overlay and merged-region extent conflicts, and a
   mutual-anchor blocked-spill seed for neighboring `SEQUENCE` formulas plus a merged-region
   `SpillBlocker(extent)` dirty-closure check for blocked-anchor repair. The repeated-R1C1
   optimized case also emits a warm no-op cache-hit report with `cells_visited == 0`,
   `formula_evaluations == 0`, `p19_warm_noop_holds == true`, and readout equal to the
   optimized baseline. A strict-bound unit test covers the same P-20 provider path for
   `SUM(A:B)` with 2,097,152 declared cells and 2 occupied slots visited.
7. OxXlPlay COM capture prerequisites for spill and hidden rows.
8. Perf counters and register assertions (`P-00`, `P-10`, `P-11`, `P-14`, `P-15`, `P-16`, `P-17`, `P-18`, `P-19`, `P-20`, `P-21`).
   Current executable floor: the seed runner emits P-19/P-20 JSON sections where applicable,
   and the first grid-scale runner writes `counter_summary.json` and `register_assertions.json`
   for `sparse-whole-column`, `full-column-1m`, `sparse-singletons`, `zig-zag-1m`,
   `dense-values`, `repeated-r1c1`, `fill-down-r1c1`, `pascal-r1c1-1m`, `boring-1mx10`,
   `direct-r1c1-1m`, `unary-r1c1-1m`, `argument-aggregate-r1c1-1m`, `math-function-r1c1-1m`, `mod-function-r1c1-1m`, `rounding-function-r1c1-1m`, `integer-function-r1c1-1m`, `log-function-r1c1-1m`, `trig-function-r1c1-1m`, `angle-function-r1c1-1m`, `reference-function-r1c1-1m`, `logical-function-r1c1-1m`, `if-logical-r1c1-1m`, `two-left-r1c1-1m`, `absolute-r1c1-1m`, `division-r1c1-1m`, `decimal-r1c1-1m`, `recursive-binary-r1c1-1m`, `if-r1c1-1m`, `if-branch-r1c1-1m`, `nested-if-r1c1-1m`, `iferror-r1c1-1m`, `comparison-r1c1-1m`, `comparison-expression-r1c1-1m`, `comparison-iferror-r1c1-1m`, `sum-row-r1c1-1m`, `sumsq-row-r1c1-1m`,
   `count-row-r1c1-1m`, `product-row-r1c1-1m`, `average-row-r1c1-1m`, `min-max-row-r1c1-1m`, `sum-window-r1c1-1m`,
   `division-error-r1c1-1m`, `division-error-propagation-r1c1-1m`, `aggregate-error-r1c1-1m`, `text-function-r1c1-1m`, `index-function-r1c1-1m`, `match-function-r1c1-1m`, and
   `insert-storm-1m`, `publication-delta-1m`, `tile-stream-64k`, `viewport-64k-of-1m`, `cow-retention-1m`, `plan-cache-rounds-1m`, `range-invalidation-1m`, `range-query-1m`, `sum-pyramid-1m`,
   `dirty-rect-1m`, `hide-storm-1m`, `spill-anchor-1m`, `filter-spill-1m`, `sequence-spill-1m`, `spill-blockage-1m`,
   `aggregate-context-1m`, and `spill-epoch-1m` profiles. Evidence runs
   `w061-grid-scale-sparse-whole-column-001`,
   `w061-grid-scale-full-column-1m-001`, `w061-grid-scale-sparse-singletons-001`,
   `w061-grid-scale-zig-zag-1m-001`, `w061-grid-scale-dense-values-001`, and
   `w061-grid-scale-repeated-r1c1-001`, plus `w061-grid-scale-fill-down-r1c1-001`,
   `w061-grid-scale-pascal-r1c1-1m-001`, and
   `w061-grid-scale-boring-1mx10-001`, plus `w061-grid-scale-insert-storm-1m-001`,
   `w061-grid-scale-direct-r1c1-1m-001`,
   `w061-grid-scale-unary-r1c1-1m-001`,
   `w061-grid-scale-argument-aggregate-r1c1-1m-001`,
   `w061-grid-scale-math-function-r1c1-1m-001`,
   `w061-grid-scale-mod-function-r1c1-1m-001`,
   `w061-grid-scale-rounding-function-r1c1-1m-001`,
   `w061-grid-scale-integer-function-r1c1-1m-001`,
   `w061-grid-scale-log-function-r1c1-1m-001`,
   `w061-grid-scale-trig-function-r1c1-1m-001`,
   `w061-grid-scale-angle-function-r1c1-1m-001`,
   `w061-grid-scale-reference-function-r1c1-1m-001`,
   `w061-grid-scale-logical-function-r1c1-1m-001`,
   `w061-grid-scale-if-logical-r1c1-1m-001`,
   `w061-grid-scale-two-left-r1c1-1m-001`,
   `w061-grid-scale-absolute-r1c1-1m-001`,
   `w061-grid-scale-division-r1c1-1m-001`,
   `w061-grid-scale-decimal-r1c1-1m-001`,
   `w061-grid-scale-recursive-binary-r1c1-1m-001`,
   `w061-grid-scale-if-r1c1-1m-001`,
   `w061-grid-scale-if-branch-r1c1-1m-001`,
   `w061-grid-scale-nested-if-r1c1-1m-001`,
   `w061-grid-scale-iferror-r1c1-1m-001`,
   `w061-grid-scale-comparison-r1c1-1m-001`,
   `w061-grid-scale-comparison-expression-r1c1-1m-001`,
   `w061-grid-scale-comparison-iferror-r1c1-1m-001`,
   `w061-grid-scale-sum-row-r1c1-1m-001`,
   `w061-grid-scale-sumsq-row-r1c1-1m-001`,
   `w061-grid-scale-count-row-r1c1-1m-001`,
   `w061-grid-scale-product-row-r1c1-1m-001`,
   `w061-grid-scale-average-row-r1c1-1m-001`,
   `w061-grid-scale-min-max-row-r1c1-1m-001`,
   `w061-grid-scale-sum-window-r1c1-1m-001`,
   `w061-grid-scale-division-error-r1c1-1m-001`,
   `w061-grid-scale-division-error-propagation-r1c1-1m-001`,
   `w061-grid-scale-aggregate-error-r1c1-1m-001`,
   `w061-grid-scale-text-function-r1c1-1m-001`,
   `w061-grid-scale-index-function-r1c1-1m-001`,
   `w061-grid-scale-match-function-r1c1-1m-001`,
   `w061-grid-scale-publication-delta-1m-001`, and
   `w061-grid-scale-range-invalidation-1m-001`,
   `w061-grid-scale-range-query-1m-001`, and
   `w061-grid-scale-sum-pyramid-1m-001`, and
   `w061-grid-scale-dirty-rect-1m-001`, and
   `w061-grid-scale-hide-storm-1m-001`, `w061-grid-scale-spill-anchor-1m-001`, and
   `w061-grid-scale-filter-spill-1m-006`, and
   `w061-grid-scale-sequence-spill-1m-002`, and
   `w061-grid-scale-spill-blockage-1m-001`, and
   `w061-grid-scale-aggregate-context-1m-001`, and
   `w061-grid-scale-spill-epoch-1m-003`, and
   `w061-grid-scale-tile-stream-64k-001`, and
   `w061-grid-scale-viewport-64k-of-1m-001`, and
   `w061-grid-scale-cow-retention-1m-001`, and
   `w061-grid-scale-plan-cache-rounds-1m-001`, all
   passed with zero failed register assertions. These
   cover strict `A:A` occupied-slot enumeration plus zero blank-cell bytes, 1,000,000 compact
   sparse numeric singleton cells at 24 B/cell, 1,000,000 full-width zig-zag sparse numeric
   singleton cells across 16,384 columns at 24 B/cell with 16,383,000,000 blank cells costing
   zero bytes and a 1,000,000-point P-18 partition bound, the fully occupied 1,048,576-row dense
   `full-column-1M` `SUM(A:A)` lane with 1,048,576 dense slots visited and zero sparse slots,
   10,000 packed dense numeric values with zero
   computed sparse cells and about 8.014 authored bytes/cell, 1,000 repeated R1C1 formulas with
   one prepared template, one formula-plan miss plus 999 hits, and a zero-visit warm no-op pass,
   the full 1,000,000-row `=R[-1]C+1`
   fill-down lane with one prepared template, expected first/middle/last values, and zero-visit
   warm no-op plus one formula-plan miss and 999,998 hits, the 1,000,000-row x 8-column
   `pascal-r1c1-1M` lane with dense/sparse boundaries, a 6,999,993-cell 2D repeated R1C1
   recurrence `=RC[-1]+R[-1]C`, one prepared template, one formula-plan miss plus 6,999,992
   hits, two dense computed regions, seven sparse computed seed cells, expected sampled
   recurrence values, and zero-visit warm no-op, and the full 1,000,000-row x
   10-column `boring-1Mx10` lane with 8,000,000 packed
   dense numeric cells, a 2,000,000-cell repeated R1C1 formula block, one prepared template,
   one formula-plan miss plus 1,999,999 hits, about 6.400 authored bytes per occupied cell,
   expected sampled values, a valid P-18 partition witness over one dense region and one
   repeated-formula region, and zero-visit warm no-op, plus the `direct-r1c1-1M` lane with one
   dense input column and two repeated direct scalar R1C1 formula-output columns, `=RC[-1]`
   and `=(RC[-2])`, two prepared templates and compiled plans, two formula-plan misses plus
   1,999,998 hits, sampled direct and parenthesized values 10/5000000/10000000, 3,000,000
   dense numeric-packed computed cells, and zero-visit warm no-op, plus the `unary-r1c1-1M`
   lane with one dense input column and three repeated unary scalar R1C1 formula-output
   columns, `=-RC[-1]`, `=-(RC[-2]+5)`, and `=-RC[-3]*2+1`, three prepared templates and
   compiled plans, three formula-plan misses plus 2,999,997 hits, sampled direct values
   -10/-5000000/-10000000, parenthesized values -15/-5000005/-10000005, arithmetic values
   -19/-9999999/-19999999, 4,000,000 dense numeric-packed computed cells, and zero-visit warm
   no-op, plus the `argument-aggregate-r1c1-1M` lane with two dense input columns and six
   repeated aggregate argument-list R1C1 formula-output columns,
   `=SUM(RC[-2],RC[-1],5)`, `=COUNT(RC[-3],RC[-2],5)`,
   `=PRODUCT(RC[-4],RC[-3],2)`, `=AVERAGE(RC[-5],RC[-4],5)`,
   `=MIN(RC[-6],RC[-5],5)`, and `=MAX(RC[-7],RC[-6],5)`, six prepared templates and compiled
   plans, six formula-plan misses plus 5,999,994 hits, sampled first values
   16/3/20/5.333333333333333/1/10, sampled middle SUM/PRODUCT/AVERAGE values
   5500005/5000000000000/1833335, sampled last SUM/PRODUCT/AVERAGE/MIN/MAX values
   11000005/20000000000000/3666668.3333333335/5/10000000, 8,000,000 dense numeric-packed
   computed cells, and zero-visit warm no-op, plus the `math-function-r1c1-1M` lane with two
   dense input columns and three repeated scalar math-function R1C1 formula-output columns,
   `=ABS(RC[-2])`, `=SQRT(RC[-2])`, and `=POWER(ABS(RC[-4]),2)`, three prepared templates and
   compiled plans, three formula-plan misses plus 2,999,997 hits, sampled first ABS/SQRT/POWER
   values 1/1/1, sampled middle values 500000/500000/250000000000, sampled last values
   1000000/1000000/1000000000000, 5,000,000 dense numeric-packed computed cells, and
   zero-visit warm no-op, plus the `mod-function-r1c1-1M` lane with two dense input columns
   and three repeated MOD scalar-function R1C1 formula-output columns,
   `=MOD(RC[-2],RC[-1])`, `=IF(MOD(RC[-3],2)=0,RC[-3]/2,RC[-3]*3)`, and
   `=MOD(POWER(RC[-4],2),RC[-3])`, three prepared templates and compiled plans, three
   formula-plan misses plus 2,999,997 hits, sampled first MOD/IF/POWER-MOD values 1/3/1,
   sampled middle values 4/250000/2, sampled last values 1/500000/1, 5,000,000 dense
   numeric-packed computed cells, and
   zero-visit warm no-op, plus the `rounding-function-r1c1-1M` lane with two dense input
   columns and three repeated ROUND-family R1C1 formula-output columns,
   `=ROUND(RC[-2],RC[-1])`, `=ROUNDUP(RC[-3],RC[-2])`, and
   `=ROUNDDOWN(RC[-4],RC[-3])`, three prepared templates and compiled plans, three
   formula-plan misses plus 2,999,997 hits, sampled first ROUND/ROUNDUP/ROUNDDOWN values
   2/2/1, sampled middle values 500001/500001/500000, sampled last values
   1000001/1000001/1000000, 5,000,000 dense numeric-packed computed cells, and
   zero-visit warm no-op, plus the `integer-function-r1c1-1M` lane with two dense input
   columns and three repeated INT/TRUNC R1C1 formula-output columns,
   `=INT(RC[-2])`, `=TRUNC(RC[-3])`, and `=TRUNC(RC[-4],RC[-3])`, three prepared templates
   and compiled plans, three formula-plan misses plus 2,999,997 hits, sampled first
   INT/TRUNC/TRUNC-tens values 1/1/0, sampled middle values 500000/500000/500000, sampled
   last values 1000000/1000000/1000000, 5,000,000 dense numeric-packed computed cells, and
   zero-visit warm no-op, plus the `log-function-r1c1-1M` lane with two dense numeric input
   columns and four repeated EXP/LN/LOG R1C1 formula-output columns,
   `=EXP(RC[-1])`, `=LN(RC[-3])`, `=LOG10(RC[-4]*100)`, and
   `=LOG(RC[-5]*100,10)`, four prepared templates and compiled plans, four formula-plan
   misses plus 3,999,996 hits, sampled first/middle/last EXP/LN/LOG10/LOG values 1/0/2/2,
   6,000,000 dense numeric-packed computed cells, zero sparse cells, and
   zero-visit warm no-op, plus the `trig-function-r1c1-1M` lane with one dense numeric input
   column and three repeated SIN/COS/TAN R1C1 formula-output columns, `=SIN(RC[-1])`,
   `=COS(RC[-2])`, and `=TAN(RC[-3])`, three prepared templates and compiled plans, three
   formula-plan misses plus 2,999,997 hits, sampled first/middle/last SIN/COS/TAN values
   0/1/0, 4,000,000 dense numeric-packed computed cells, zero sparse cells, and
   zero-visit warm no-op, plus the `angle-function-r1c1-1M` lane with two dense numeric input
   columns and four repeated RADIANS/DEGREES/PI R1C1 formula-output columns,
   `=RADIANS(RC[-2])`, `=DEGREES(RC[-2])`, `=SIN(RADIANS(RC[-4]))`, and `=PI()`, four
   prepared templates and compiled plans, four formula-plan misses plus 3,999,996 hits,
   sampled first/middle/last RADIANS/DEGREES/SIN/PI values 0/0/0/3.141592653589793,
   6,000,000 dense numeric-packed computed cells, zero sparse cells, and
   zero-visit warm no-op, plus the `reference-function-r1c1-1M` formula-only lane with six
   repeated ROW/COLUMN reference-identity formula-output columns,
   `=ROW()`, `=COLUMN()`, `=ROW(RC[-2])`, `=COLUMN(RC[-3])`, `=ROWS(R1C1:R3C1)`, and
   `=COLUMNS(RC[-5]:RC[-3])`, six prepared templates and compiled plans, six formula-plan
   misses plus 5,999,994 hits, sampled ROW values 1/500000/1000000 and COLUMN/ROWS/COLUMNS
   values 2/1/3/3, 6,000,000 dense numeric-packed computed cells, zero literal cells, zero
   sparse cells, and
   zero-visit warm no-op, plus the `logical-function-r1c1-1M` lane with two dense signed input
   columns and three repeated logical-function R1C1 formula-output columns,
   `=AND(RC[-2]>0,RC[-1]>0)`, `=OR(RC[-3]>0,RC[-2]>0)`, and
   `=NOT(AND(RC[-4]>0,RC[-3]>0))`, three prepared templates and compiled plans, three
   formula-plan misses plus 2,999,997 hits, sampled first AND/OR/NOT values false/true/true,
   row-3 values true/true/false, sampled middle values false/false/true, sampled last values
   false/false/true, 5,000,000 dense computed cells, 2,000,000 numeric-packed input cells,
   3,000,000 logical-packed formula-output cells, zero sparse cells, and
   zero-visit warm no-op, plus the `if-logical-r1c1-1M` lane with two dense signed input
   columns and three repeated numeric IF formula-output columns with logical-function
   conditions,
   `=IF(AND(RC[-2]>0,RC[-1]>0),RC[-2]+RC[-1],0)`,
   `=IF(OR(RC[-3]>0,RC[-2]>0),RC[-3]-RC[-2],0)`, and
   `=IF(NOT(AND(RC[-4]>0,RC[-3]>0)),ABS(RC[-4]),0)`, three prepared templates and compiled
   plans, three formula-plan misses plus 2,999,997 hits, sampled first values 0/2/1,
   row-3 values 6/0/0, sampled middle values 0/0/500000, sampled last values 0/0/1000000,
   5,000,000 dense numeric-packed computed cells, zero sparse cells, and
   zero-visit warm no-op, plus the `two-left-r1c1-1M` lane with
   the same dense/repeated shape, `=RC[-2]+RC[-1]`, one prepared template and compiled plan,
   one formula-plan miss plus 1,999,999 hits, sampled formula values
   2015/1500000023/3000000023, dense formula output, and zero-visit warm no-op, plus the
   `absolute-r1c1-1M` lane with two dense input columns, a mixed absolute/relative repeated
   R1C1 formula block `=RC[-1]+R1C1`, one prepared template and compiled plan, one formula-plan
   miss plus 999,999 hits, sampled formula values 3/1000001/2000001, dense formula output, and
   zero-visit warm no-op, plus the
   `division-r1c1-1M` lane with one dense input column, a 1,000,000-cell repeated R1C1
   `=RC[-1]/2` formula block, one prepared template and compiled plan, one formula-plan miss
   plus 999,999 hits, sampled values 1/500000/1000000, dense formula output, and zero-visit
   warm no-op, plus the `decimal-r1c1-1M` lane with the same dense/repeated shape,
   `=RC[-1]*0.5`, one prepared template and compiled plan, one formula-plan miss plus 999,999
   hits, sampled values 1/500000/1000000, dense formula output, and zero-visit warm no-op, plus
   the `recursive-binary-r1c1-1M` lane with three dense input columns and two repeated R1C1
   recursive-binary formula-output columns `=RC[-3]+RC[-2]*RC[-1]` and
   `=(RC[-4]+RC[-3])*RC[-2]`, two prepared templates and compiled plans, two formula-plan
   misses plus 1,999,998 hits, sampled precedence values 21/10500000/21000000 and
   parenthesized values 22/11000000/22000000, 5,000,000 dense numeric-packed computed cells,
   zero sparse computed cells, and zero-visit warm no-op, plus
   the `if-r1c1-1M` lane with one dense signed input column, a repeated R1C1 numeric IF block
   `=IF(RC[-1]>0,RC[-1],0)`, one prepared template and compiled plan, one formula-plan miss
   plus 999,999 hits, sampled values 1/0/999999/0, dense formula output, and zero-visit warm
   no-op, plus
   the `if-branch-r1c1-1M` lane with one dense signed input column, a repeated R1C1 numeric IF
   branch-expression block `=IF(RC[-1]>0,RC[-1]*2,RC[-1]/2)`, one prepared template and
   compiled plan, one formula-plan miss plus 999,999 hits, sampled values
   2/-250000/1999998/-500000, dense numeric-packed formula output, and zero-visit warm no-op, plus
   the `nested-if-r1c1-1M` lane with two dense input columns, a repeated R1C1 nested numeric IF
   block `=IF(RC[-2]>500000,IF(RC[-1]>0,RC[-2]*2,RC[-2]*3),IF(RC[-1]>0,RC[-2]+1,RC[-2]-1))`,
   one prepared template and compiled plan, one formula-plan miss plus 999,999 hits, sampled
   values 0/500001/1500003/2000000, 3,000,000 dense numeric-packed computed cells, and zero-visit
   warm no-op, plus
   the `iferror-r1c1-1M` lane with two dense input columns, a repeated R1C1 worksheet-error
   recovery block `=IFERROR(RC[-2]/RC[-1],0)`, one prepared template and compiled plan, one
   formula-plan miss plus 999,999 hits, sampled values 1/0/999999/0, dense formula output, and
   zero-visit warm no-op, plus
   the `comparison-r1c1-1M` lane with one dense signed input column, a repeated R1C1 logical
   output block `=RC[-1]>0`, one prepared template and compiled plan, one formula-plan miss
   plus 999,999 hits, sampled values true/false/true/false, dense logical formula output with
   2,000,000 dense computed cells, 1,000,000 numeric-packed input cells,
   1,000,000 logical-packed formula output cells, zero sparse cells, and zero-visit warm no-op, plus
   the `comparison-expression-r1c1-1M` lane with two dense signed input columns, a repeated R1C1
   scalar-expression comparison block `=RC[-2]*2>RC[-1]+1`, one prepared template and compiled
   plan, one formula-plan miss plus 999,999 hits, sampled values false/false/true/false,
   3,000,000 dense computed cells, 2,000,000 numeric-packed input cells,
   1,000,000 logical-packed formula output cells, zero sparse cells, and zero-visit warm no-op, plus
   the `comparison-iferror-r1c1-1M` lane with two dense input columns, a repeated R1C1 nested
   IFERROR comparison block `=IFERROR(RC[-2]/RC[-1],0)>0`, one prepared template and compiled
   plan, one formula-plan miss plus 999,999 hits, sampled values true/false/true/false,
   3,000,000 dense computed cells, 2,000,000 numeric-packed input cells,
   1,000,000 logical-packed formula output cells, zero sparse cells, and zero-visit warm no-op, plus
   the `sum-row-r1c1-1M` lane with three dense input columns, a repeated R1C1 range aggregate
   `=SUM(RC[-3]:RC[-1])`, one prepared template and compiled plan, one formula-plan miss plus
   999,999 hits, sampled values 6/3000000/6000000, dense formula output, and zero-visit warm no-op, plus
   the `sumsq-row-r1c1-1M` lane with three dense input columns, a repeated R1C1 range
   aggregate `=SUMSQ(RC[-3]:RC[-1])`, one prepared template and compiled plan, one formula-plan
   miss plus 999,999 hits, sampled values 14/3500000000000/14000000000000, 4,000,000 dense
   numeric-packed computed cells, zero sparse computed cells, and zero-visit warm no-op, plus
   the `count-row-r1c1-1M` lane with three dense input columns, a repeated R1C1 range
   aggregate `=COUNT(RC[-3]:RC[-1])`, one prepared template and compiled plan, one
   formula-plan miss plus 999,999 hits, sampled values 3/3/3, dense formula output, and
   zero-visit warm no-op, plus
   the `product-row-r1c1-1M` lane with three dense input columns, a repeated R1C1 range
   aggregate `=PRODUCT(RC[-3]:RC[-1])`, one prepared template and compiled plan, one
   formula-plan miss plus 999,999 hits, sampled values 6/6/6, dense formula output, and
   zero-visit warm no-op, plus
   the `average-row-r1c1-1M` lane with three dense input columns, a repeated R1C1 range
   aggregate `=AVERAGE(RC[-3]:RC[-1])`, one prepared template and compiled plan, one
   formula-plan miss plus 999,999 hits, sampled values 2/1000000/2000000, dense formula
   output, and zero-visit warm no-op, plus
   the `min-max-row-r1c1-1M` lane with three dense input columns, repeated R1C1 range
   aggregates `=MIN(RC[-3]:RC[-1])` and `=MAX(RC[-4]:RC[-2])`, two prepared templates and
   compiled plans, two formula-plan misses plus 1,999,998 hits, sampled MIN values
   1/500000/1000000, sampled MAX values 3/1500000/3000000, dense formula output, and
   zero-visit warm no-op, plus
   the `sum-window-r1c1-1M` lane with one dense input column, a vertical repeated R1C1 range
   aggregate `=SUM(R[-2]C[-1]:RC[-1])`, one prepared template and compiled plan, one
   formula-plan miss plus 999,997 hits, sampled values 6/1499997/2999997, dense formula output,
   and zero-visit warm no-op, plus
   the `division-error-r1c1-1M` lane with the same dense/repeated shape,
   `=RC[-1]/0`, one prepared template and compiled plan, one formula-plan miss plus 999,999
   hits, sampled `Div0` values, dense formula output, and zero-visit warm no-op, plus the
   `division-error-propagation-r1c1-1M` lane with one dense input column, a direct-error
   repeated R1C1 block `=RC[-1]/0`, a dependent repeated R1C1 block `=RC[-1]+1`, two prepared
   templates and compiled plans, two formula-plan misses plus 1,999,998 hits, dense direct and
   propagated `Div0` output, and zero-visit warm no-op, plus the
   `aggregate-error-r1c1-1M` lane with one dense input column, a direct-error block
   `=RC[-1]/0`, a range aggregate over the erroring lane `=SUM(RC[-2]:RC[-1])`, and an
   IFERROR recovery block `=IFERROR(SUM(RC[-3]:RC[-2]),RC[-3])`; it prepares three templates
   and compiled plans, records three formula-plan misses plus 2,999,997 hits, publishes
   4,000,000 dense computed cells with 2,000,000 numeric-packed cells, zero sparse cells, and
   sampled `Div0` aggregate errors plus recovered values 2/1000000/2000000, plus the
   `text-function-r1c1-1M` lane with one uniform dense text input column, repeated text-function
   blocks `=LEN(RC[-1])`, `=LEFT(RC[-2],3)`, `=RIGHT(RC[-3],4)`, and
   `=CONCAT(RC[-2],RC[-1])`, four prepared templates and compiled plans, four formula-plan
   misses plus 3,999,996 hits, 5,000,000 dense computed cells with 1,000,000 numeric-packed
   cells, zero sparse cells, sampled `RowGrid`/`7`/`Row`/`Grid`/`RowGrid` values, P-10 uniform
   text authored storage, and zero-visit warm no-op, plus the `index-function-r1c1-1M` lane with
   two dense input columns and repeated INDEX blocks
   `=INDEX(RC[-2]:RC[-1],1,1)`, `=INDEX(RC[-3]:RC[-2],1,2)`,
   `=INDEX(R1C1:RC1,ROW(),1)`, and `=INDEX(RC[-5]:RC[-4],2,1)`, four prepared templates and
   compiled plans, four formula-plan misses plus 3,999,996 hits, 6,000,000 dense computed
   cells with 3,000,000 numeric-packed cells, zero sparse cells, sampled numeric/text/dynamic
   lookup values 10/Index/5000000/10000000, sampled `Ref` output for the out-of-range index,
   and zero-visit warm no-op, plus the `match-function-r1c1-1M` lane with three dense numeric
   input columns and repeated exact MATCH / nested INDEX-MATCH blocks
   `=MATCH(RC[-2],RC[-3]:RC[-1],0)`,
   `=INDEX(RC[-4]:RC[-2],1,MATCH(RC[-3],RC[-4]:RC[-2],0))`, and
   `=MATCH(999999999,RC[-5]:RC[-3],0)`, three prepared templates and compiled plans, three
   formula-plan misses plus 2,999,997 hits, 6,000,000 dense computed cells with 5,000,000
   numeric-packed cells, zero sparse cells, sampled exact-match position `2`, nested
   INDEX/MATCH value `5000001`, no-match `NA` output, and zero-visit warm no-op. Supported
   retained repeated-R1C1 templates
   (`=RC[-1]`, `=(RC[-2])`, `=-RC[-1]`, `=-(RC[-2]+5)`, `=-RC[-3]*2+1`, `=RC[-1]*2`, `=R[-1]C+1`, `=RC[-1]+R[-1]C`, `=RC[-2]+RC[-1]`, `=RC[-1]+R1C1`,
   `=RC[-1]/2`,
   `=RC[-1]*0.5`, `=RC[-3]+RC[-2]*RC[-1]`, `=(RC[-4]+RC[-3])*RC[-2]`,
   `=IF(RC[-1]>0,RC[-1],0)`,
   `=IF(RC[-1]>0,RC[-1]*2,RC[-1]/2)`,
   `=IF(RC[-2]>500000,IF(RC[-1]>0,RC[-2]*2,RC[-2]*3),IF(RC[-1]>0,RC[-2]+1,RC[-2]-1))`,
   `=IFERROR(RC[-2]/RC[-1],0)`, `=RC[-1]>0`, `=RC[-2]*2>RC[-1]+1`,
   `=IFERROR(RC[-2]/RC[-1],0)>0`,
   `=SUM(RC[-3]:RC[-1])`, `=SUMSQ(RC[-3]:RC[-1])`,
   `=COUNT(RC[-3]:RC[-1])`,
   `=PRODUCT(RC[-3]:RC[-1])`,
   `=AVERAGE(RC[-3]:RC[-1])`,
   `=MIN(RC[-3]:RC[-1])`, `=MAX(RC[-4]:RC[-2])`,
   `=SUM(RC[-2]:RC[-1])`, `=IFERROR(SUM(RC[-3]:RC[-2]),RC[-3])`,
   `=SUM(R[-2]C[-1]:RC[-1])`, `=SUM(RC[-2],RC[-1],5)`,
   `=COUNT(RC[-3],RC[-2],5)`, `=PRODUCT(RC[-4],RC[-3],2)`,
   `=AVERAGE(RC[-5],RC[-4],5)`, `=MIN(RC[-6],RC[-5],5)`,
   `=MAX(RC[-7],RC[-6],5)`, `=ABS(RC[-2])`, `=SQRT(RC[-2])`,
   `=POWER(ABS(RC[-4]),2)`, `=MOD(RC[-2],RC[-1])`,
   `=IF(MOD(RC[-3],2)=0,RC[-3]/2,RC[-3]*3)`,
   `=MOD(POWER(RC[-4],2),RC[-3])`, `=ROUND(RC[-2],RC[-1])`,
   `=ROUNDUP(RC[-3],RC[-2])`, `=ROUNDDOWN(RC[-4],RC[-3])`,
   `=INT(RC[-2])`, `=TRUNC(RC[-3])`, `=TRUNC(RC[-4],RC[-3])`,
   `=EXP(RC[-1])`, `=LN(RC[-3])`, `=LOG10(RC[-4]*100)`,
   `=LOG(RC[-5]*100,10)`,
   `=SIN(RC[-1])`, `=COS(RC[-2])`, `=TAN(RC[-3])`,
   `=RADIANS(RC[-2])`, `=DEGREES(RC[-2])`, `=SIN(RADIANS(RC[-4]))`, `=PI()`,
   `=ROW()`, `=COLUMN()`, `=ROW(RC[-2])`, `=COLUMN(RC[-3])`,
   `=ROWS(R1C1:R3C1)`, `=COLUMNS(RC[-5]:RC[-3])`,
   `=LEN(RC[-1])`, `=LEFT(RC[-2],3)`, `=RIGHT(RC[-3],4)`,
   `=CONCAT(RC[-2],RC[-1])`,
   `=INDEX(RC[-2]:RC[-1],1,1)`, `=INDEX(RC[-3]:RC[-2],1,2)`,
   `=INDEX(R1C1:RC1,ROW(),1)`, `=INDEX(RC[-5]:RC[-4],2,1)`,
   `=MATCH(RC[-2],RC[-3]:RC[-1],0)`,
   `=INDEX(RC[-4]:RC[-2],1,MATCH(RC[-3],RC[-4]:RC[-2],0))`,
   `=MATCH(999999999,RC[-5]:RC[-3],0)`,
   `=AND(RC[-2]>0,RC[-1]>0)`,
   `=OR(RC[-3]>0,RC[-2]>0)`, `=NOT(AND(RC[-4]>0,RC[-3]>0))`,
   `=IF(AND(RC[-2]>0,RC[-1]>0),RC[-2]+RC[-1],0)`,
   `=IF(OR(RC[-3]>0,RC[-2]>0),RC[-3]-RC[-2],0)`,
   `=IF(NOT(AND(RC[-4]>0,RC[-3]>0)),ABS(RC[-4]),0)`,
   `=RC[-1]/0`, plus dependent `=RC[-1]+1` over an upstream error) now publish
   dense computed formula-output regions in these scale lanes. Unit test
   `optimized_grid_compact_oxfml_recalc_evaluates_direct_scalar_r1c1_templates` proves direct
   scalar and parenthesized direct scalar R1C1 references as dense formula-output templates,
   unit test `optimized_grid_compact_oxfml_recalc_evaluates_unary_minus_r1c1_templates`
   proves unary-minus scalar R1C1 expressions as dense formula-output templates,
   unit test
   `optimized_grid_compact_oxfml_recalc_evaluates_r1c1_argument_aggregate_templates`
   proves SUM/SUMSQ/COUNT/PRODUCT/AVERAGE/MIN/MAX argument-list R1C1 calls as dense formula-output
   templates,
   unit test `optimized_grid_compact_oxfml_recalc_evaluates_r1c1_scalar_function_templates`
   proves ABS/SQRT/POWER scalar math-function R1C1 calls, nested scalar-function arguments,
   and `SQRT` domain errors as dense formula-output templates,
   unit test `optimized_grid_compact_oxfml_recalc_evaluates_r1c1_mod_function_templates`
   proves MOD scalar-function R1C1 calls, MOD-driven IF conditions, nested scalar-function
   arguments, negative-modulo semantics, and divisor-zero errors,
   unit test `optimized_grid_compact_oxfml_recalc_evaluates_r1c1_rounding_function_templates`
   proves ROUND/ROUNDUP/ROUNDDOWN scalar-function R1C1 calls with positive, negative, and
   negative-digit rounding cases,
   unit test `optimized_grid_compact_oxfml_recalc_evaluates_r1c1_integer_function_templates`
   proves INT floor semantics, one-arg TRUNC default digits, and two-arg TRUNC negative-digit
   cases as dense formula-output templates,
   unit test `optimized_grid_compact_oxfml_recalc_evaluates_r1c1_log_function_templates`
   proves EXP, LN, LOG10, and LOG scalar-function R1C1 calls, default LOG base behavior, and
   logarithm domain errors as dense formula-output templates,
   unit test `optimized_grid_compact_oxfml_recalc_evaluates_r1c1_trig_function_templates`
   proves SIN, COS, and TAN scalar-function R1C1 calls over radians as dense formula-output
   templates,
   unit test `optimized_grid_compact_oxfml_recalc_evaluates_r1c1_angle_function_templates`
   proves RADIANS, DEGREES, nested degree-to-radian trig, and zero-arg PI calls as dense
   formula-output templates,
   unit test `optimized_grid_compact_oxfml_recalc_evaluates_r1c1_reference_function_templates`
   proves ROW/COLUMN/ROWS/COLUMNS over current cell, R1C1 references, finite R1C1 ranges, and
   arithmetic composition without value dereference,
   unit test `optimized_grid_compact_oxfml_recalc_r1c1_text_functions_stay_dense` proves
   LEN/LEFT/RIGHT/CONCAT over R1C1 reference arguments as dense `CalcValue` text/numeric output
   and chained formula-output references without sparse fallback,
   unit test `optimized_grid_compact_oxfml_recalc_r1c1_index_function_stays_dense` proves
   bounded positive-index `INDEX` over R1C1 ranges as dense numeric/text/`#REF!` `CalcValue`
   output, including a dynamic `ROW()` selector without range enumeration,
   unit test `optimized_grid_compact_oxfml_recalc_r1c1_match_function_stays_dense` proves
   exact `MATCH(...,0)` over finite R1C1 ranges, nested `INDEX(...,MATCH(...),...)`, and
   first-class `#N/A` output as dense formula-output cells,
   unit test
   `optimized_grid_compact_oxfml_recalc_evaluates_r1c1_logical_function_templates` proves
   AND/OR/NOT over comparison/logical R1C1 expressions as dense logical formula-output
   templates,
   unit test
   `optimized_grid_compact_oxfml_recalc_evaluates_r1c1_if_logical_condition_templates` proves
   logical-function R1C1 expressions as IF conditions with dense numeric formula-output arms,
   and unit test
   `optimized_grid_compact_oxfml_recalc_evaluates_general_binary_r1c1_templates` proves the
   underlying optimized compiler accepts the bounded recursive binary R1C1 class over relative
   and absolute refs, finite numeric literals, arithmetic precedence, parenthesized scalar
   subexpressions, `+`, `-`, `*`, and `/` rather than only those exact strings.
   Unit tests `optimized_grid_compact_oxfml_recalc_evaluates_r1c1_if_templates`,
   `optimized_grid_compact_oxfml_recalc_evaluates_nested_r1c1_if_templates`, and
   `optimized_grid_compact_oxfml_recalc_r1c1_if_propagates_condition_and_branch_errors`
   prove the numeric IF R1C1 conditional class, nested scalar branch expressions, and
   selected-branch/error propagation.
   Unit test `optimized_grid_compact_oxfml_recalc_evaluates_r1c1_iferror_templates` proves
   IFERROR over bounded R1C1 scalar expressions, fallback expressions, and non-evaluation of an
   erroring fallback when the first expression is clean.
   Unit tests `optimized_grid_compact_oxfml_recalc_evaluates_r1c1_comparison_templates` and
   `optimized_grid_compact_oxfml_recalc_r1c1_comparison_propagates_operand_errors` prove direct
   comparison formulas as dense logical output, scalar-expression comparison operands, nested
   IFERROR operands, plus operand-error propagation.
   `optimized_grid_compact_oxfml_recalc_evaluates_absolute_r1c1_templates` proves absolute
   R1C1 axes for point and range references,
   `optimized_grid_compact_oxfml_recalc_evaluates_r1c1_sum_range_templates`,
   `optimized_grid_compact_oxfml_recalc_evaluates_r1c1_sumsq_range_templates`,
   `optimized_grid_compact_oxfml_recalc_evaluates_r1c1_count_range_templates`,
   `optimized_grid_compact_oxfml_recalc_evaluates_r1c1_product_range_templates`,
   `optimized_grid_compact_oxfml_recalc_evaluates_r1c1_average_range_templates`, and
   `optimized_grid_compact_oxfml_recalc_evaluates_r1c1_min_max_range_templates` prove the
   narrow R1C1 SUM/SUMSQ/COUNT/PRODUCT/AVERAGE/MIN/MAX-range classes and `w061-grid-scale-sum-window-r1c1-1m-001` retains
   the vertical sliding-window leg: the fill-down run reports one
   sparse seed plus one dense formula-output region, while the `boring-1Mx10` run reports
   `computed_sparse_cells == 0` and the two-left, absolute-r1c1, numeric-division,
   decimal-literal, recursive-binary, numeric-IF, IF branch-expression, IF logical-condition, IFERROR, scalar-MOD, scalar-rounding, scalar-integer, scalar-log, scalar-trig, scalar-angle, reference-function, logical-function, comparison, comparison-expression, comparison-iferror, sum-row, sumsq-row, count-row, product-row,
   average-row, min-max-row, sum-window, division-error, division-error-propagation, aggregate-error, text-function, index-function, and match-function lanes also report
   `computed_sparse_cells == 0`. Broader multi-error precedence beyond the retained aggregate-error lane and broader non-arithmetic
   worksheet-error algebra inside dense formula-output regions remain outside the current
   optimized error-algebra claim.
   The P-10 sparse/dense/repeated authored-storage byte legs,
   including the full-width `zig-zag-1M` adversarial sparse leg, the P-20 fully occupied
   `full-column-1M` lane, the R1C1 `fill-down-1M` and `pascal-r1c1-1M` lanes, and the
   `boring-1Mx10` authored-storage/template/warm-no-op lane are now executable. The
   `insert-storm-1M` lane applies six row insert/delete edits over 9,999,920 compact authored
   cells, touches 42 region metadata records versus a 59,999,520-cell naive rewrite floor,
   keeps zero sparse point materialization, and passes P-17 for the compact structural-edit
   floor. The `publication-delta-1M` lane compares two 1,000,000-row optimized valuations after
   one dense input cell and its dependent repeated-R1C1 output change; it reports two changed
   dense region publication entries, zero sparse/spill delta entries, and
   `publication_entry_ratio_micros == 1` against a 2,000,000-cell publication floor, passing
   P-22 for the compact publication-entry floor. P-18 partition-witness counters are now executable for the sparse singleton and
   dense/repeated-formula scale layouts; parallel execution remains deferred. The
   `range-invalidation-1M` lane installs a 1,000,000-row finite range dependency as one
   compressed reverse edge with zero range scalarization, keeps only one downstream scalar edge,
   and proves a row-500,000 seed dirties the range formula and downstream dependent, passing
   P-13 for the finite-range compressed reverse-edge floor. The `range-query-1M` lane installs
   1,000 compressed finite range dependencies over 1,000,000 rows and checks only two indexed
   candidates for a row-500,501 seed instead of scanning all 1,000 range edges, passing P-12 for
   the compressed-range block-index query floor. The `sum-pyramid-1M` lane installs a
   1,000,000-row aggregation pyramid with 1,111 compressed range edges covering a
   3,700,111-cell expanded support floor, zero range scalarization, one downstream scalar edge,
   `indexed_candidate_sum == 20`, and a 6-cell dirty closure, passing P-12/P-13 plus
   `GRID-SUM-PYRAMID-1M`. The `dirty-rect-1M` lane installs 1,000 compressed range consumers,
   1,000 scalar consumers, and one downstream chain over 1,000,000 rows; an 11-cell dirty
   rectangle checks three indexed candidates across 2,002 total edges and closes exactly the
   selected range consumer, scalar consumer, and downstream cell, passing P-12 plus
   `GRID-DIRTY-RECT-1M`. The `hide-storm-1M` lane installs 1,000
   hidden-sensitive AxisState row-band dependencies over 1,000,000 rows and checks only two
   indexed candidates for a row-500,501 hidden-row seed instead of scanning all 1,000
   visibility edges, passing P-24 for the visibility block-index query floor. The
   `spill-anchor-1M` lane resolves
   a 1,000,000-row `A1#` extent with one spill-ledger probe, zero extent cells scanned for the
   ledger lookup, and three provider value entries scanned/returned, passing P-25 for the
   ledger-probe floor. The `filter-spill-1M` lane clears a 1,000,000-row old spill extent
   through the optimized sparse-value index: three indexed clear candidates are removed from
   1003 sparse values, 1000 unrelated sparse values remain, and the smaller re-spill sample
   touches six sparse cells total versus 5,000,000 grid cells. The same lane evaluates
   `FILTER(A1:B1000000,C1:C1000000)` over dense value/include regions, publishes a 500,000-row
   by 2-column spill with 999,999 ghosts, keeps `computed_sparse_cells == 0`, samples FILTER
   row pairs 101/102, 25000001/25000002, and 50000001/50000002, leaves the first vacated
   row empty, and commits one spill fact plus one epoch anchor back to optimized sheet state
   (`filter_formula_spill_commit_committed_fact_entries == 1`,
   `filter_formula_spill_commit_anchors_added == 1`), passing P-23 for the old-spill
   indexed-clear plus dense value-dependent two-column FILTER committed-publication floor. The
   same run now covers a column-mask FILTER over a 999,999-row by 3-column source with a
   horizontal include row, publishing a 999,999-row by 2-column dense output with zero sparse
   computed cells and committed sheet spill state, passing `GRID-FILTER-COLUMN-SPILL-1M`.
   The same run applies one sparse include-mask override, runs a second committed optimized
   FILTER recalc, shrinks the dense output to 499,999 rows by 2 columns, clears the vacated
   ghost row, and advances the committed extent/value epoch to 2, passing
   `GRID-FILTER-LIFECYCLE-1M`. The `sequence-spill-1M` lane publishes `SEQUENCE(1000000)` as one dense
   optimized computed region with 1,000,000 packed numeric cells, zero sparse computed cells,
   999,999 ghost cells, and sampled values 1/500000/1000000, then commits that optimized
   publication back to sheet state (`spill_commit_committed_fact_entries == 1`,
   `spill_commit_anchors_added == 1`, `spill_commit_current_epoch_anchors == 1`,
   `sheet_committed_spill_fact_entries == 1`), passing P-23 for the dense dynamic-array
   publication plus committed spill-state floor. The `spill-blockage-1M` lane checks a 1,000,000-row intended spill
   extent through compact blocker candidates: the empty leg performs zero compact blocker
   probes, while a sparse blocker at row 1,000,000 is found with one compact probe instead of a
   1,000,000-cell scan, passing P-26 for the optimized compact blocker-probe floor. The
   `spill-epoch-1M` lane compares spill anchor snapshots for a 1,000,000-row `A1#` extent:
   unchanged snapshots preserve two ledger epochs and dirty zero consumers, unrelated
   value-epoch churn dirties zero consumers, and A1 value/extent changes dirty the `A1#`
   consumer plus its downstream cell, passing P-27 for the first spill-epoch ledger plus
   dirty-closure floor. The
   `aggregate-context-1M` lane reports the grid host-info provider plan for a 1,000,000-row
   `SUBTOTAL`-style context query: two explicit AxisState rows plus three default row runs
   produce five row-context probes, while the current OxFunc host-info seam still expands the
   returned packet to 1,000,000 cells, passing P-28 for the provider-side row-run floor. The
   `tile-stream-64K` lane streams a 320 x 200 subscribed tile over a 1,000,000-row x
   320-column model, visits 64,000 dense tile cells, visits zero unrelated sparse cells despite
   1,000 sparse values outside the tile, estimates a 1,536,159-byte frame at about 24.003 bytes
   per subscribed cell under the 64-byte P-15 cap, and passes `GRID-TILE-STREAM-64K`. The
   `viewport-64K-of-1M` lane evaluates a 64,000-cell visible formula column over a
   1,000,000-row x 10-column dense/repeated-R1C1 model by projecting only the visible rows'
   same-row upstream cone: 64,000 dense input cells plus 128,000 repeated formula cells,
   192,000 evaluated cells before visible completion, two dense computed regions, zero sparse
   computed cells, and a 1.92% evaluated-to-full-occupied ratio, passing P-16 plus
   `GRID-VIEWPORT-64K`. The `cow-retention-1M` lane retains seven compact roots across the
   same insert/delete storm, shares one dense payload across all roots, records
   64,012,633 retained COW bytes versus a 448,009,561-byte full-snapshot retention floor,
   keeps zero sparse materialization and zero blank-cell bytes, and passes P-21 plus
   `GRID-COW-RETENTION-1M`. The `plan-cache-rounds-1M` lane runs the dense/repeated-R1C1
   shape through three dirty compact OxFml recalc rounds sharing one
   `GridOptimizedFormulaPlanCache`: 6,000,000 total formula lookups, 5,999,999 hits, one first-round
   miss, zero later-round misses, one cached template, one cached compiled R1C1 plan object,
   one first-round compiled-plan miss, zero later compiled-plan misses, and two later
   compiled-plan hits, passing P-14 plus `GRID-PLAN-CACHE-ROUNDS-1M`. Unit test
   `optimized_grid_formula_plan_cache_recompiles_stale_fingerprint_and_prunes_unused_plans`
   covers the first compiled-plan lifecycle floor: changed source/channel fingerprints recompile
   even when the normal-form key is stale, and unused templates/compiled plans are pruned after
   formula regions are replaced by values. Formula-output functions outside direct/unary scalar R1C1, aggregate argument-list R1C1, scalar math-function R1C1, MOD scalar-function R1C1, ROUND-family scalar-function R1C1, INT/TRUNC scalar-function R1C1, EXP/LN/LOG scalar-function R1C1, SIN/COS/TAN scalar-function R1C1, RADIANS/DEGREES/PI scalar-function R1C1, reference-function R1C1, text-function R1C1 over reference arguments, bounded positive-index INDEX over R1C1 ranges, exact MATCH over finite one-dimensional R1C1 ranges, logical-function R1C1, IF logical-condition R1C1, bounded recursive binary R1C1, comparison R1C1 with scalar operands and nested IFERROR, numeric IF with nested scalar branch expressions, IFERROR R1C1, narrow SUM/SUMSQ/COUNT/PRODUCT/AVERAGE/MIN/MAX-range classes, and the retained aggregate-error range/IFERROR lane,
   broader multi-error precedence beyond the retained aggregate-error lane and broader non-arithmetic worksheet-error algebra inside dense formula-output regions,
   broader compiled-plan eviction policy/version lifecycle,
   production COW retention GC/lifecycle, the broader publication lifecycle and spill-storm
   arbitration matrix, the broader FILTER value-dependent spill matrix,
   production host tile protocol, production viewport scheduling, and the broader workload matrix remain open.
9. Spill reference floor: ledger, `SpillBlocker(extent)` watches, `A1#` provider path.
   Current executable floor: GridCalc-Ref and GridOptimizedSheet both carry committed
   anchor-keyed spill facts into the grid `ReferenceSystemProvider`, and formula results that
   are arrays now publish spill extents as anchor/ghost computed cells in GridCalc-Ref and
   dense optimized computed regions in GridOptimizedSheet for successful array payloads. `A1#` evaluates through
   OxFml/OxFunc in reference and optimized modes. Formula-owned spill-ledger changes run a
   bounded repair phase, counted separately from primary `P-00`, so earlier consumers such as
   `SUM(B1#)` converge after a later `B1` anchor publishes. GridInvalidation-Ref records
   `SpillFact(anchor)` dependencies and closes over spill-shape changes separately from
   current member-cell edits. It also records finite `SpillBlocker(extent)` dependencies for
   blocked anchors, with a merged-region blocker-watch seed proving that blocker clearance or
   movement dirties the anchor chain separately from ordinary value edits. Authored-cell,
   merged-region, table-overlay, and boundary blockage produce anchor `#SPILL!` plus a blocked
   spill fact in the executable floor. The first mutual-anchor floor also blocks a later
   dynamic-array anchor that lies inside an earlier blocked formula-owned spill extent; broad
   mutual/body-overlap arbitration, table structural invalidation, circular-spill cap
   policy, and Excel-order evidence remain open lanes. The optimized path now shares a compact
   spill-blockage probe with its runtime blocked decision: the retained
   `w061-grid-scale-spill-blockage-1m-001` artifact proves an empty 1,000,000-row intended
   extent performs zero compact blocker probes, and a far sparse blocker is found with one
   compact probe rather than an empty-cell scan. `w061-grid-scale-spill-epoch-1m-003` adds the
   first spill-epoch ledger floor: `GridSpillEpochLedger` preserves epochs for unchanged
   formula-owned spills and advances them for value, extent, and blocked-state changes in
   GridCalc-Ref and optimized valuations, and `GridOptimizedSheet` now commits optimized
   valuation spill facts/fingerprints back into sheet state with same-grid validation and ledger
   update counters (`optimized_spill_commit_first_added == 1`,
   `optimized_spill_commit_second_preserved == 1`,
   `optimized_spill_commit_extent_changed == 1`; covered by
   `grid_calc_ref_spill_epoch_ledger_preserves_and_advances_anchor_epochs`,
   `optimized_grid_spill_epoch_ledger_preserves_and_advances_anchor_epochs`, and
   `optimized_grid_rejects_spill_publication_commit_from_different_grid`); unchanged and
   unrelated spill-anchor snapshots do not dirty `A1#` consumers, while value/extent epoch
   changes for the referenced anchor dirty only that consumer chain.
   `w061-grid-scale-filter-spill-1m-006` adds the first optimized
   old-spill clear floor for re-spill: old output lookup is index bounded and unrelated sparse
   values survive, and it exercises a dense-backed 1,000,000-row two-column FILTER input whose
   500,000-row by 2-column result publishes as dense output with zero sparse computed cells and
   commits back to sheet spill facts plus epoch ledger state. It also adds the first
   column-mask FILTER floor with a horizontal include row over a 999,999-row by 3-column dense
   source, publishing a committed 999,999-row by 2-column dense output.
   The same run proves a later optimized committed recalc can shrink a value-dependent FILTER
   spill to 499,999 rows by 2 columns and advance the committed extent/value epoch to 2.
   `w061-grid-scale-sequence-spill-1m-002` adds the first dense optimized dynamic-array
   publication plus committed spill-state floor: successful numeric `SEQUENCE` output is
   region-backed rather than sparse-cell backed, and the optimized publication is committed
   back to sheet spill facts plus epoch ledger state. The broader FILTER matrix, broader
   publication lifecycle cases, and broad spill-storm arbitration remain open.
10. Hidden-row floor: AxisState provenance, `GridHostInfoProvider`, visibility invalidation.
    Current executable floor: GridCalc-Ref and GridOptimizedSheet both feed
    `GridHostInfoProvider` to OxFml/OxFunc evaluation, and GridInvalidation-Ref records
    row/column visibility ranges as first-class dependencies with closure queries for hide,
    filter, and outline-like AxisState changes. The provider now exposes a
    `GridAggregateContextQueryReport` that plans aggregate row context over AxisState row runs;
    retained evidence `w061-grid-scale-aggregate-context-1m-001` proves five row-context probes
    for a 1,000,000-row aggregate reference with two hidden rows, while explicitly recording the
    current 1,000,000-cell OxFunc packet expansion as the remaining run-compressed seam gap.
11. Defined-name value/invalidation floor: GridCalc-Ref and GridOptimizedSheet both carry a same-sheet
    defined-name map into `GridReferenceSystemProvider`. OxFml binds name syntax as symbolic
    `excel.grid.v1` payloads, while OxCalc resolves only the provider-owned runtime namespace.
    `SUM(InputRange)` and `SUM(INDIRECT("InputRange"))` now evaluate in both engines and in the
    seed corpus. `GridInvalidation-Ref` records finite `Name(name, extent)` dependencies,
    scalarizes the current extent for ordinary value edits, exposes name-key dirty closure for
    namespace changes, and transforms the finite extent under row/column insert/delete. The
    grid machines also transform same-sheet defined-name rects under row/column edits.
    Defined-name lifecycle APIs now cover same-sheet rename and delete in GridCalc-Ref and
    GridOptimizedSheet: direct formulas are rewritten on rename (`InputRange` -> `DataRange`)
    and delete (`DataRange` -> `#NAME?`), while `INDIRECT("InputRange")` text is preserved and
    resolves through runtime text routing. GridInvalidation-Ref retargets/drops finite name
    dependencies for rename/delete lifecycle operations and returns the namespace-key dirty
    closure; the seed corpus emits these as `namespace_lifecycle` reports. Namespace versioning
    and structured/table name interactions remain open.
12. Table-overlay structured-reference floor: GridCalc-Ref and GridOptimizedSheet now carry
   same-sheet `GridTableOverlay` records with bounded table range, optional header/totals
   ranges, column descriptors, and a feature-rendered-region claim. OxFml still owns parsing
   native structured-reference syntax and calls the generic `bind_structured_reference` hook;
   OxCalc's strict grid profile binds the formula token as a symbolic `excel.grid.v1`
   structured-reference payload, while `GridReferenceSystemProvider` resolves explicit
   `Table1`, `Table1[#All]`, `Table1[#Data]`, `Table1[#Headers]`, `Table1[#Totals]`,
   `Table1[Column]`, section-qualified column references, contiguous column ranges,
   non-contiguous section unions over table columns, escaped explicit column names, caller-local
   `[Column]` references, and caller-local `[@Column]` references to provider-owned ranges at
   runtime.
   `SUM(Table1[Amount])`, repeated in-table `=SUM([Amount])` and `=[@Amount]*2` formulas,
   `SUM(Table1[[#Data],[Amount]:[Tax]])`,
   `SUM(Table1[[#Headers],[#Totals],[Amount]:[Tax]])`, escaped-column formulas, and
   corresponding `INDIRECT(...)` text forms now evaluate in both engines, and
   `GridInvalidation-Ref` records finite
   `Table(table, extent)` dependencies and current-row scalar cell dependencies with scalar
   dirty closure, namespace-key dirty closure, and row/column insert/delete extent transforms.
   First table-overlay lifecycle APIs now cover same-sheet resize, rename, and delete in
   GridCalc-Ref and GridOptimizedSheet: stale table feature-rendered-region claims are removed,
   explicit structured-reference formulas are rewritten on table rename (`Table1[Column]` ->
   `Sales[Column]`) and table delete (`Sales[Column]` -> `#REF!`), renamed tables resolve
   through the provider under the new name, and deleted overlays stop contributing table
   metadata. GridInvalidation-Ref retargets/drops finite table dependencies for rename/delete
   lifecycle operations, rebuilds scalar edges for table resize extents, and returns the
   namespace-key dirty closure; the seed corpus emits these as `namespace_lifecycle` reports.
   Current exclusions: table/name collision precedence for omitted references,
   `INDIRECT("Table1[...]")` text rewrites, resize-driven formula source
   expansion/shrink semantics, full table namespace versioning, and OxDoc table ingest/export
   fidelity.
13. Structural edit algebra over formulas, tables, merged regions, spill anchors/extents,
   whole-row/column references, geometry-coupled opaque parts, and future feature-rendered
   regions. Current executable floor: `StrictExcelGridReferenceProfile::transform_reference`
   accepts an `excel-grid-structural-edit.v1` payload with old/new formula-anchor context and
   transforms point references, finite area references, whole-row references, and whole-column
   references over row/column insert/delete. It returns first-class `#REF!` payloads for deleted
   point/range targets and keeps spill, name-reference payloads, and structured references
   host-sensitive; finite same-sheet defined-name rects transform in OxCalc's provider-owned
   namespace map rather than in OxFml core.
   `GridCalcRefSheet::apply_axis_edit` now uses this profile result to rewrite moved authored
   formula cells, preserve R1C1 source shape where the relative template survives, rebind fresh
   normal-form keys, and emit formula-transform counters in the structural edit report.
   `GridOptimizedSheet::apply_axis_edit` now applies the same row/column insert/delete lane to
   sparse points, dense value regions, and repeated formula regions without materializing the
   whole sheet; inserted rows/columns inside compact regions become gaps by splitting the
   compact regions.
   Feature-rendered regions now have a first class-sensitive edit policy in both GridCalc-Ref
   and GridOptimizedSheet: table overlays keep transforming, while pivot-like regions refuse
   inside row/column structural edits and mark `needs_refresh` when a before-region edit shifts
   their geometry.
   `GridInvalidationRef::apply_axis_edit` covers the matching small dirty-closure lane for
   scalarized cell/range dependencies, axis-indexed whole-row/whole-column value dependencies,
   finite same-sheet name and table dependencies, committed spill-fact anchors, and finite
   spill-blocker extents; opaque-part structural invalidation, full namespace versioning, and
   full spill placement/blockage arbitration remain outside this first executable floor.

## Implementation sequence from current reality

1. **ABI hardening gate:** align this contract target with OxFml W077. Required shape:
   default-preserving `BindProfile`, profile id/version, `GridBounds`, source-span/surface-text
   preservation, symbolic R1C1 and caller-relative A1 bound reference records, `$` fidelity,
   caller-independent normal-form/cache identity, dependency envelopes, argument-preparation
   hooks, edit-transform envelopes with deletion holes/`#REF!`, and rendering separate from
   normal form.
2. **Non-grid proof gate:** run existing DnaTreeCalc tree references through the same OxFml
   lifecycle and add a fake minimal profile test. This proves the seam is reference-profile
   shaped, not grid shaped.
3. **Stabilize current OxCalc grid floor:** keep GridCalc-Ref as the BTreeMap value/effects
   oracle and GridInvalidation-Ref as the scalar dirty-closure oracle; preserve the current
   `reference|optimized|both` harness and mark optimized support non-claiming where no
   production candidate exists.
4. **Constrain new grid breadth:** add new syntax/runtime families in narrow slices only after
   the profile seam owns recognition, binding, normal form, dependency emission, argument
   preparation, transform, and render policy for the family.
5. **Structural edits:** continue from the existing row/column slice through the structural-edit
   matrix: formulas, names, tables, merged regions, spill anchors/extents, whole-row/column
   references, geometry-coupled opaque parts, and feature-rendered regions. Any unsupported
   family returns a host-sensitive/unsupported transform result rather than an ad hoc rewrite.
6. **Excel/OxDoc evidence:** route OxDoc follow-ups for `PartStore`/opaque-byte preservation and
   hidden-provenance reconciliation; route OxXlPlay scenarios for spill order, hidden-row
   behavior, `SUBTOTAL`/`AGGREGATE`, row-height-zero, AutoFilter, outline collapse, and
   structural-edit comparison.

## Rollout mode

`planning_promoted_reference_floor_next`
