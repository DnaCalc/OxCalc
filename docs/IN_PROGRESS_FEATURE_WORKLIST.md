# OxCalc In-Progress Feature Worklist

This file is a compact feature-orientation surface. It is not live execution state; `.beads/` owns that. It is also not the source for broad product-feature claims; use the active spec, implementation, and evidence surfaces for that.

Older detailed W033-W045 status material was archived during the W046 cleanup prelude at `archive/IN_PROGRESS_FEATURE_WORKLIST.pre-w046-cleanup.md`.

## Current Focus
1. `W056_TREECALC_FULL_REFERENCE_AND_TABLE_LOWERING.md`
2. Current route: keep W050 as the reached-gate formula-authority substrate and W051 as the closed first sparse/reference-reader scope, then complete the W056 full TreeCalc reference/table-lowering successor before bounded-memory and formalization work.
3. Predecessors: W050 formula-authority/session/prepared-package rework, W047 CTRO implementation-first work, and W048 circular-reference closure under single-host Excel scope.
4. W049 receives post-W050 formalization intake; non-Excel rich/virtual data ideas are parked in `docs/spec/core-engine/CORE_ENGINE_FUTURE_IDEAS_RICH_AND_VIRTUAL_DATA.md`.

## Active Aim
The active calculation-model line has moved from W050 execution to successor planning. Current active aim:
1. keep W050 closure/status surfaces aligned with the landed production seam,
2. plan W051 as Excel-compatible sparse range readers, defined-entry semantics, and DNA TreeCalc `@CHILDREN` / `.*` reference-collection compatibility with OxFml-owned generic parsing, an OxCalc-supplied host context, and OxCalc-owned resolution, not generic rich data,
3. keep W054 bounded-memory and pinned-epoch GC sequenced after the W051 artifact set,
4. route post-W050 formalization to W049,
5. route sensitivity/derivative behavior to W052 only where it maps to recognizable spreadsheet behavior,
6. keep non-Excel virtual/rich data ideas out of the workset queue until a product decision promotes them.

Current next engineering move:

1. Continue W056 from the closed W051 first scope: keep the promoted node-associated table topic stable while widening the implemented `ChildrenV1` carrier and sparse/reference-reader path into the remaining non-table TreeCalc reference families, and retain bridge/replay evidence.
2. Keep built-in/UDF/defined-name/defined-name-`LAMBDA` name/call precedence blocked on OxFml `W074-CALC005` oracle evidence; map TreeCalc host names and lambda-valued nodes to the closest Excel defined-name lane until that evidence justifies an explicit extension.
3. Treat the active DnaTreeCalc free-standing and qualified children corpus slice as first bridge evidence only; full W004/W005 non-table reference activation remains W056 scope, while the declared node-associated table topic is promoted through `calc-4vs8.34` through `calc-4vs8.38` and full intended table support is tracked by the third-pass `calc-4vs8.39` through `calc-4vs8.43` completion spine.
4. Keep fixture-only reader scaffolding and any `treecalc_eager_values_fallback.v1` materialization clearly separated from production sparse range and reference-preserving reference-collection authority.

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
