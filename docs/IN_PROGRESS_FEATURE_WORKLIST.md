# OxCalc In-Progress Feature Worklist

This file is a compact orientation surface. It is not live execution state; `.beads/` owns that. Older detailed W033-W045 status material was archived during the W046 cleanup prelude at `archive/IN_PROGRESS_FEATURE_WORKLIST.pre-w046-cleanup.md`.

## Current Focus
1. `W048_CIRCULAR_DEPENDENCY_CALCULATION_PROCESSING.md`
2. Current route: reopened W048 scope repair and full Excel-match circular-reference implementation before successor formalization/deepening
3. Predecessor: W047 CTRO implementation-first work through `calc-aylq`
4. W049 route: deferred until W048 no longer hides Excel-match circular-reference semantics as successor scope

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

1. cleared: `calc-zci1.16` / `BLK-W048-EXCEL-ROOT` has declared local root/report evidence from documented `Worksheet.CircularReference` in `w048-excel-root-report-002`;
2. blocked: `calc-zci1.19` / `BLK-W048-EXCEL-VERSION` needs a second Excel host/version packet, or explicit user acceptance of a single-host scoped claim;
3. no W048 ready beads remain until the version blocker is resolved or user-scoped.

## Active Truth Surfaces
1. `docs/worksets/W047_CALC_TIME_REBINDING_OVERLAY_DESIGN_SWEEP.md`
2. `docs/worksets/W048_CIRCULAR_DEPENDENCY_CALCULATION_PROCESSING.md`
3. `docs/spec/core-engine/w048-cycles/`
4. `docs/spec/core-engine/w047-ctro/`
5. `docs/spec/core-engine/CORE_ENGINE_RECALC_AND_INCREMENTAL_MODEL.md`
6. `docs/spec/core-engine/CORE_ENGINE_FORMALIZATION_AND_ASSURANCE.md`
7. `docs/spec/core-engine/CORE_ENGINE_TRACECALC_REFERENCE_MACHINE.md`
8. `docs/spec/core-engine/CORE_ENGINE_OXFML_SEAM.md`
9. `formal/tla/` for current engine-state models
10. `formal/lean/` for active proof/model files
11. `src/oxcalc-core/` and `src/oxcalc-tracecalc/` for implementation and reference machinery

## Archived Predecessor Material
W038-W045 packet docs, workset docs, late row-classifier Lean files, generated evidence roots, and superseded showcase material now live under top-level `archive/`.

The archive is retained as historical evidence and predecessor context. It is not the active direction for W046.

## Open Cleanup Lanes
1. Distill any useful W033-W045 semantic fragments into W046 specs before further archiving.
