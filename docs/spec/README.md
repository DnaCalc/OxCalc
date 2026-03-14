# OxCalc Spec Index

This directory is the OxCalc-owned mutable spec library.

## Canonical OxCalc Set
The rewritten canonical core-engine set is:
- `docs/spec/core-engine/CORE_ENGINE_ARCHITECTURE.md`
- `docs/spec/core-engine/CORE_ENGINE_STATE_AND_SNAPSHOTS.md`
- `docs/spec/core-engine/CORE_ENGINE_RECALC_AND_INCREMENTAL_MODEL.md`
- `docs/spec/core-engine/CORE_ENGINE_OVERLAY_AND_DERIVED_RUNTIME.md`
- `docs/spec/core-engine/CORE_ENGINE_COORDINATOR_AND_PUBLICATION.md`
- `docs/spec/core-engine/CORE_ENGINE_OXFML_SEAM.md`
- `docs/spec/core-engine/CORE_ENGINE_FORMALIZATION_AND_ASSURANCE.md`
- `docs/spec/core-engine/CORE_ENGINE_REALIZATION_ROADMAP.md`

Rewrite-control artifacts for this reset:
- `docs/spec/core-engine/SPEC_REWRITE_PLAN.md`
- `docs/spec/core-engine/SPEC_REWRITE_DOCUMENT_MAP.md`
- `docs/spec/core-engine/REWRITE_PROMOTION_LEDGER.md`

## Bootstrap Archive and Reference-Only Material
The previous bootstrap set is preserved under:
- `docs/spec/core-engine/archive/bootstrap-2026-03/`

Bootstrap redirect/reference-only files remain in `docs/spec/core-engine/` for provenance and pointer stability.
Foundation snapshot files in `docs/spec/core-engine/` are local reference support, not OxCalc-owned canonical architecture.

## Visibility and Related Policy Docs
- `docs/spec/visibility/*`
  - retained for visibility-priority and formatting-boundary policy work.

## Consumed Mirror Set
- `docs/spec/fec-f3e/*`
  - copied from OxFml-owned canonical seam specs for local implementation reference.

## Mirror Policy
1. OxCalc owns its canonical core-engine spec set in this repo.
2. OxFml owns the canonical shared FEC/F3E seam specification.
3. Foundation retains doctrine and conformance-policy ownership and keeps read-only mirrors/snapshots for cross-program assurance.
