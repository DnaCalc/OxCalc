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

## Supporting Realization and Test Docs
- `docs/spec/core-engine/CORE_ENGINE_TEST_HARNESS_AND_FIXTURES.md`
  - supporting companion for self-contained fixture, scenario, and alternate calculation-space design.
- `docs/spec/core-engine/CORE_ENGINE_TEST_SCENARIO_SCHEMA_AND_TRACECALC.md`
  - supporting companion defining canonical JSON scenario schema and the first concrete `TraceCalc` surface.

## Seed Test Corpus
- `docs/test-corpus/core-engine/tracecalc/README.md`
  - first checked-in self-contained `TraceCalc` scenario corpus.

## Archived Rewrite-Control Material
The rewrite-control artifacts used to establish the canonical set are preserved for provenance under:
- `docs/spec/core-engine/archive/rewrite-control-2026-03/`

These files are historical planning and promotion-control artifacts, not active canonical guidance.

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
