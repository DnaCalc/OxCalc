# TraceCalc Corpus

This directory contains the first checked-in self-contained `TraceCalc` scenario corpus for the OxCalc core-engine harness.

## Purpose
The corpus exists to provide hand-auditable scenario inputs for:
1. candidate-result versus publication separation,
2. reject-is-no-publish behavior,
3. pinned-view stability,
4. dynamic dependency switching,
5. overlay retention expectations,
6. early scale and generator-shape planning.

## Layout
- `MANIFEST.json`
  - corpus-level index for the currently authored scenarios.
- `hand-auditable/*.json`
  - small explicit scenarios intended for direct inspection.

## Current Corpus Slice
The current checked-in slice is intentionally explicit and is now exercised by the validator, runner, engine, and reference-machine paths.

Current scenarios:
1. `tc_accept_publish_001.json`
2. `tc_reject_no_publish_001.json`
3. `tc_pinned_view_stability_001.json`
4. `tc_dynamic_dep_switch_001.json`
5. `tc_overlay_retention_001.json`
6. `tc_scale_chain_seed_001.json`
7. `tc_verify_clean_no_publish_001.json`
8. `tc_multinode_dag_publish_001.json`
9. `tc_publication_fence_reject_001.json`
10. `tc_artifact_token_reject_001.json`
11. `tc_fallback_reentry_001.json`
12. `tc_cycle_region_reject_001.json`

The manifest currently indexes 38 hand-auditable scenarios, including the W033,
W034, W035, W037, and W048 extensions. The short list above is the original
seed slice, not the full active manifest.

## W057 Snapshot-Layer Surface
The Rust TraceCalc runner now emits explicit snapshot-layer artifacts:

1. `workspace_revision` names the structure, node-input, and namespace roots.
2. `snapshot_layers` names formula-binding, dependency-shape, publication, and
   runtime-overlay layer refs.
3. `snapshot_layers.json` records the final refs plus dependency-shape
   publication records per scenario.
4. `w057-snapshot-coverage/coverage.json` maps W056/W057 epoch-snapshot
   surfaces to TraceCalc oracle scenarios and optimized TreeCalc fixtures, or
   names exact TraceCalc language blockers.

The current exact blockers are formula text edit with unchanged dependency
shape, static formula-to-literal release, dynamic target value update through an
old CTRO effective graph, unresolved-to-resolved formula edit as a single
transition, and rename/delete/move structural edits as first-class TraceCalc
steps.

## Relationship To Spec Docs
This corpus is authored against:
1. `docs/spec/core-engine/CORE_ENGINE_TEST_HARNESS_AND_FIXTURES.md`
2. `docs/spec/core-engine/CORE_ENGINE_TEST_SCENARIO_SCHEMA_AND_TRACECALC.md`
3. `docs/spec/core-engine/CORE_ENGINE_TEST_VALIDATOR_AND_RUNNER_CONTRACT.md`

## Status
Product status: the checked corpus is exercised by the Rust TraceCalc
reference-machine runner, oracle/engine self-diff, oracle matrix, replay bundle
projection, and W057 snapshot-layer coverage packet.

Evidence: `cargo test -p oxcalc-tracecalc` executes the full manifest in the
runner test and asserts the W057 snapshot-layer artifact and coverage packet.

Still open: the TraceCalc scenario language does not yet model unchanged-shape
formula edits, static formula-to-literal release, dynamic target value updates
through prior CTRO overlays, unresolved-to-resolved formula text edits, or
rename/delete/move structural edits as first-class scenario steps.
