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

## Relationship To Spec Docs
This corpus is authored against:
1. `docs/spec/core-engine/CORE_ENGINE_TEST_HARNESS_AND_FIXTURES.md`
2. `docs/spec/core-engine/CORE_ENGINE_TEST_SCENARIO_SCHEMA_AND_TRACECALC.md`
3. `docs/spec/core-engine/CORE_ENGINE_TEST_VALIDATOR_AND_RUNNER_CONTRACT.md`

## Status
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - the corpus is now exercised by the checked-in `w013-sequence-a-baseline` and `w014-stage1-widening-baseline` runs
  - replay-pack and replay-appliance bundle projections are still later lanes
  - generated large-graph corpus lanes remain narrower than the later economics and retained-failure lanes
