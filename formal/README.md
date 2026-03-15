# Formal Artifact Layout

This directory contains the first OxCalc-local assurance artifacts that move W006-W008 beyond planning-only status.

## Layout
- `formal/lean/`
  - Lean-facing state vocabularies and theorem-oriented skeletons.
- `formal/tla/`
  - TLA+ models and model-check configuration for coordinator, publication, and later concurrency safety.
- `formal/replay/`
  - reserved for later replay schemas and artifact-family bindings.

## Current Floor
1. `formal/lean/OxCalc/CoreEngine/Stage1State.lean`
   - first Lean-facing state-object skeleton for the implemented Stage 1 structural, coordinator, and recalc vocabulary.
   - exercised with a local `lean formal/lean/OxCalc/CoreEngine/Stage1State.lean` typecheck run.
2. `formal/tla/CoreEngineStage1.tla`
   - first TLA+ skeleton for Stage 1 coordinator, publication, reject, pin, and recalc actions.
3. `formal/tla/CoreEngineStage1.cfg`
   - first model-check configuration skeleton for the Stage 1 TLA+ module.

## Status
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - the Lean skeleton has been typechecked once locally, but no theorem authoring exists yet
  - no TLC or other TLA+ tool run has been executed in this repo yet
  - replay and pack artifacts do not yet consume these formal artifacts
  - the artifact set is still a first skeleton floor rather than a proof- or model-check-complete lane
