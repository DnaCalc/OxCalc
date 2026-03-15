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
The current checked-in slice is intentionally small and explicit.
It is suitable for future validator, runner, and replay-pack work, but it is not yet an exercised replay corpus.

Current scenarios:
1. `tc_accept_publish_001.json`
2. `tc_reject_no_publish_001.json`
3. `tc_pinned_view_stability_001.json`
4. `tc_dynamic_dep_switch_001.json`
5. `tc_overlay_retention_001.json`
6. `tc_scale_chain_seed_001.json`

## Relationship To Spec Docs
This corpus is authored against:
1. `docs/spec/core-engine/CORE_ENGINE_TEST_HARNESS_AND_FIXTURES.md`
2. `docs/spec/core-engine/CORE_ENGINE_TEST_SCENARIO_SCHEMA_AND_TRACECALC.md`

## Status
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - no validator or fixture runner consumes this corpus yet
  - no replay-pack artifact binds to these scenarios yet
  - generated large-graph corpus lanes are still unauthored
