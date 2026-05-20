# OxCalc Spec

This is the entrypoint for OxCalc requirements, design, planning, and model truth.

Use this file the way newer DnaCalc repos use `docs/SPEC.md`: start here, then follow the indexed spec set. The old `docs/spec/README.md` remains the detailed index for the canonical core-engine library; this file is the shallow reading-order and ownership filter.

## Reading Order

1. Core spec index: [`docs/spec/README.md`](spec/README.md)
2. Recalc and incremental model: [`CORE_ENGINE_RECALC_AND_INCREMENTAL_MODEL.md`](spec/core-engine/CORE_ENGINE_RECALC_AND_INCREMENTAL_MODEL.md)
3. Coordinator and publication: [`CORE_ENGINE_COORDINATOR_AND_PUBLICATION.md`](spec/core-engine/CORE_ENGINE_COORDINATOR_AND_PUBLICATION.md)
4. State and snapshots: [`CORE_ENGINE_STATE_AND_SNAPSHOTS.md`](spec/core-engine/CORE_ENGINE_STATE_AND_SNAPSHOTS.md)
5. Profile selectors: [`CORE_ENGINE_PROFILE_SELECTORS.md`](spec/core-engine/CORE_ENGINE_PROFILE_SELECTORS.md)
6. Overlay and derived runtime: [`CORE_ENGINE_OVERLAY_AND_DERIVED_RUNTIME.md`](spec/core-engine/CORE_ENGINE_OVERLAY_AND_DERIVED_RUNTIME.md)
7. OxFml seam companion: [`CORE_ENGINE_OXFML_SEAM.md`](spec/core-engine/CORE_ENGINE_OXFML_SEAM.md)
8. TraceCalc reference machine: [`CORE_ENGINE_TRACECALC_REFERENCE_MACHINE.md`](spec/core-engine/CORE_ENGINE_TRACECALC_REFERENCE_MACHINE.md)
9. Formalization and assurance: [`CORE_ENGINE_FORMALIZATION_AND_ASSURANCE.md`](spec/core-engine/CORE_ENGINE_FORMALIZATION_AND_ASSURANCE.md)

## Active Planning And Execution Pointers

1. Workset roadmap: [`WORKSET_REGISTER.md`](WORKSET_REGISTER.md)
2. Current feature orientation: [`IN_PROGRESS_FEATURE_WORKLIST.md`](IN_PROGRESS_FEATURE_WORKLIST.md)
3. Bead execution truth: `.beads/` through `br`

## Ownership Rules

1. OxCalc owns coordinator, dependency graph, invalidation, overlay, epoch, scheduling, publication, TraceCalc, and OxCalcTree runtime contract semantics.
2. OxFml owns formula grammar, bind, evaluator, and canonical FEC/F3E shared seam specs.
3. OxFunc owns function and worksheet value semantics.
4. Foundation owns program doctrine, profiles, conformance policy, and mirror governance.

## Reporting Rule

Spec text should support implementation and verification. When a feature area is discussed, report product status, evidence, open gaps, and formal status separately.
