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
4. `formal/replay/stage1-hand-authored/`
   - first hand-authored replay artifact slice for `R1`, `R2`, and `R7`.
5. `formal/measurement/stage1_counter_schema.json`
   - first machine-readable Stage 1 counter schema.
6. `formal/measurement/stage1_experiment_register.json`
   - first machine-readable Stage 1 experiment register.

## Status
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - the Lean skeleton has been typechecked once locally, but no theorem authoring exists yet
  - no TLC or other TLA+ tool run has been executed in this repo yet
  - replay artifacts now exist only as hand-authored seed artifacts, not as harness- or oracle-emitted outputs
  - measurement artifacts now exist only as schema/register definitions; no runtime instrumentation emits them yet
  - the artifact set is still a first skeleton floor rather than a proof-, model-check-, replay-pipeline-, or instrumentation-complete lane
