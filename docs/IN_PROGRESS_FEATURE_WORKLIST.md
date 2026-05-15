# OxCalc In-Progress Feature Worklist

This file is a compact orientation surface. It is not live execution state; `.beads/` owns that. Older detailed W033-W045 status material was archived during the W046 cleanup prelude at `archive/IN_PROGRESS_FEATURE_WORKLIST.pre-w046-cleanup.md`.

## Current Focus
1. `W051_SPARSE_RANGE_READERS_AND_DEFINED_ENTRY_SEMANTICS.md`
2. Current route: keep W050 as the reached-gate formula-authority substrate, then plan Excel-scope sparse range readers before bounded-memory and formalization work.
3. Predecessors: W050 formula-authority/session/prepared-package rework, W047 CTRO implementation-first work, and W048 circular-reference closure under single-host Excel scope.
4. W049 receives post-W050 formalization intake; non-Excel rich/virtual data ideas are parked in `docs/spec/core-engine/CORE_ENGINE_FUTURE_IDEAS_RICH_AND_VIRTUAL_DATA.md`.

## Active Aim
The active calculation-model line has moved from W050 execution to successor planning. Current active aim:
1. keep W050 closure/status surfaces aligned with the landed production seam,
2. plan W051 as Excel-compatible sparse range readers and defined-entry semantics, not generic rich data,
3. keep W054 bounded-memory and pinned-epoch GC sequenced after the W051 artifact set,
4. route post-W050 formalization to W049,
5. route sensitivity/derivative behavior to W052 only where it maps to recognizable spreadsheet behavior,
6. keep non-Excel virtual/rich data ideas out of the workset queue until a product decision promotes them.

Current next engineering move:

1. Use the W051 activation review to lock the sparse reader API, first function group, and replay artifact shape across OxCalc/OxFml/OxFunc.
2. Create the W051 epic/bead path only after those intake points are explicit.
3. Keep fixture-only reader scaffolding clearly separated from production sparse range authority.

## Active Truth Surfaces
1. `docs/worksets/W047_CALC_TIME_REBINDING_OVERLAY_DESIGN_SWEEP.md`
2. `docs/worksets/W048_CIRCULAR_DEPENDENCY_CALCULATION_PROCESSING.md`
3. `docs/worksets/W050_OXCALC_OXFML_FORMULA_AUTHORITY_REWORK.md`
4. `docs/worksets/W051_SPARSE_RANGE_READERS_AND_DEFINED_ENTRY_SEMANTICS.md`
5. `docs/worksets/W054_BOUNDED_MEMORY_AND_PINNED_EPOCH_GC.md`
6. `docs/worksets/W049_CORE_ENGINE_FORMALIZATION_RESTART_AFTER_CTRO_AND_CYCLES.md`
7. `docs/spec/core-engine/CORE_ENGINE_FUTURE_IDEAS_RICH_AND_VIRTUAL_DATA.md`
8. `docs/spec/core-engine/CORE_ENGINE_RECALC_AND_INCREMENTAL_MODEL.md`
9. `docs/spec/core-engine/CORE_ENGINE_FORMALIZATION_AND_ASSURANCE.md`
10. `docs/spec/core-engine/CORE_ENGINE_TRACECALC_REFERENCE_MACHINE.md`
11. `docs/spec/core-engine/CORE_ENGINE_OXFML_SEAM.md`
12. `formal/tla/` for current engine-state models
13. `formal/lean/` for active proof/model files
14. `src/oxcalc-core/` and `src/oxcalc-tracecalc/` for implementation and reference machinery

## Archived Predecessor Material
W038-W045 packet docs, workset docs, late row-classifier Lean files, generated evidence roots, and superseded showcase material now live under top-level `archive/`.

The archive is retained as historical evidence and predecessor context. It is not the active direction for W046.

## Open Cleanup Lanes
1. Distill any useful W033-W045 semantic fragments into W046 specs before further archiving.
