# OxCalc In-Progress Feature Worklist

This file is a compact orientation surface. It is not live execution state; `.beads/` owns that. Older detailed W033-W045 status material was archived during the W046 cleanup prelude at `archive/IN_PROGRESS_FEATURE_WORKLIST.pre-w046-cleanup.md`.

## Current Focus
1. `W048_CIRCULAR_DEPENDENCY_CALCULATION_PROCESSING.md`
2. Current route: execute the full W048 circular dependency lane before successor formalization/deepening
3. Predecessor: W047 CTRO implementation-first work through `calc-aylq`
4. W049 route: formal/checker/sidecar/readiness successor work after W047 and W048

## Active Aim
W048 now owns the circular dependency calculation target end to end:
1. collect reproducible Excel circular-reference observations,
2. materialize dependency graphs explicitly across structural, published-effective, and candidate-effective layers,
3. make cycle-root/order, initial-value, terminal, publication, and release/re-entry choices explicit,
4. implement the reference behavior in TraceCalc,
5. implement the optimized/core behavior in TreeCalc,
6. develop and execute the circular-reference test corpus,
7. introduce W048-owned formal definitions, proof/model artifacts, and checker targets,
8. capture innovation opportunities in explicit opt-in profiles,
9. leave W049 as successor deepening/organization after W048 artifacts exist.

Current next engineering move:

1. execute `calc-zci1.1` Excel probes,
2. execute `calc-zci1.2` materialized graph sidecar widening,
3. execute `calc-zci1.3` TraceCalc reference cycle behavior,
4. execute `calc-zci1.6` TreeCalc optimized cycle behavior,
5. execute `calc-zci1.7` circular-reference corpus and conformance runs,
6. execute `calc-zci1.5` W048 formal cycle artifacts,
7. use the W048 result as the artifact base for successor formalization/deepening.

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
