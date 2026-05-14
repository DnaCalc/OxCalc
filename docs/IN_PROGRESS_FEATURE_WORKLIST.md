# OxCalc In-Progress Feature Worklist

This file is a compact orientation surface. It is not live execution state; `.beads/` owns that. Older detailed W033-W045 status material was archived during the W046 cleanup prelude at `archive/IN_PROGRESS_FEATURE_WORKLIST.pre-w046-cleanup.md`.

## Current Focus
1. `W050_OXCALC_OXFML_FORMULA_AUTHORITY_REWORK.md`
2. Current route: re-establish the OxCalc/OxFml formula authority boundary before further TreeCalc showcase or formula-surface expansion.
3. Predecessors: W047 CTRO implementation-first work through `calc-aylq`; W048 circular-reference closure under single-host Excel scope.
4. W049 route remains available for successor formalization/deepening, but W050 should clarify formula authority first where formula semantics are in question.

## Active Aim
W048 now owns the circular dependency calculation target end to end. The scope was reopened because the prior run closed on a conservative non-iterative slice and deferred Excel-match iterative behavior. Corrected active aim:
1. repair W048 scope/status surfaces and mark the prior closure audit superseded,
2. replace W048 Python tooling with PowerShell, Rust, or C#,
3. collect reproducible, bit-exact Excel circular-reference observations including iterative calculation,
4. materialize dependency graphs explicitly across structural, published-effective, and candidate-effective layers,
5. make cycle-root/order, initial-value, terminal, publication, release/re-entry, and calculation-chain choices explicit,
6. implement Excel-match iterative circular-reference behavior in TraceCalc,
7. implement the same behavior in TreeCalc optimized/core,
8. develop and execute the circular-reference conformance corpus against Excel, TraceCalc, and TreeCalc,
9. introduce W048-owned formal definitions, proof/model artifacts, and checker targets using non-Python tooling,
10. capture innovation opportunities only as explicit opt-in profiles after the Excel-match default is separated.

Current next engineering move:

1. Activate W050 child beads for code inventory and docs/showcase wording repair.
2. Audit `TreeFormula`, `TreeReference`, and the retired pre-W050 `evaluate_via_oxfml` bridge to distinguish source carriage from formula semantics.
3. Remove or quarantine tests/docs that imply OxCalc locally computes dynamic arrays or spreadsheet functions.

## Active Truth Surfaces
1. `docs/worksets/W047_CALC_TIME_REBINDING_OVERLAY_DESIGN_SWEEP.md`
2. `docs/worksets/W048_CIRCULAR_DEPENDENCY_CALCULATION_PROCESSING.md`
3. `docs/worksets/W050_OXCALC_OXFML_FORMULA_AUTHORITY_REWORK.md`
4. `docs/spec/core-engine/w048-cycles/`
5. `docs/spec/core-engine/w047-ctro/`
6. `docs/spec/core-engine/CORE_ENGINE_RECALC_AND_INCREMENTAL_MODEL.md`
7. `docs/spec/core-engine/CORE_ENGINE_FORMALIZATION_AND_ASSURANCE.md`
8. `docs/spec/core-engine/CORE_ENGINE_TRACECALC_REFERENCE_MACHINE.md`
9. `docs/spec/core-engine/CORE_ENGINE_OXFML_SEAM.md`
10. `formal/tla/` for current engine-state models
11. `formal/lean/` for active proof/model files
12. `src/oxcalc-core/` and `src/oxcalc-tracecalc/` for implementation and reference machinery

## Archived Predecessor Material
W038-W045 packet docs, workset docs, late row-classifier Lean files, generated evidence roots, and superseded showcase material now live under top-level `archive/`.

The archive is retained as historical evidence and predecessor context. It is not the active direction for W046.

## Open Cleanup Lanes
1. Distill any useful W033-W045 semantic fragments into W046 specs before further archiving.
