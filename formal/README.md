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
2. `formal/lean/OxCalc/CoreEngine/W033FirstSlice.lean`
   - W033 first-slice Lean vocabulary for abstract OxFml seam facts, LET/LAMBDA carrier facts, coordinator state, overlays, conservative invalidation, and small transition invariants.
   - exercised with a local `lean formal/lean/OxCalc/CoreEngine/W033FirstSlice.lean` typecheck run.
3. `formal/tla/CoreEngineStage1.tla`
   - first TLA+ skeleton for Stage 1 coordinator, publication, reject, pin, and recalc actions.
4. `formal/tla/CoreEngineStage1.cfg`
   - deeper exploration configuration for the Stage 1 TLA+ module.
5. `formal/tla/CoreEngineStage1.smoke.cfg`
   - bounded smoke-model configuration for quick local TLC verification.
6. `formal/replay/stage1-hand-authored/`
   - first hand-authored replay artifact slice for `R1`, `R2`, and `R7`.
7. `docs/test-runs/core-engine/tracecalc-reference-machine/w013-sequence-a-baseline/`
   - first emitted harness and oracle baseline run covering `R1`, `R2`, `R7`, `R4`, and `R5` through the `TraceCalc` corpus.
8. `formal/measurement/stage1_counter_schema.json`
   - first machine-readable Stage 1 counter schema.
9. `formal/measurement/stage1_experiment_register.json`
   - first machine-readable Stage 1 experiment register.

## Status
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - the Stage 1 Lean skeleton has been typechecked once locally, and W033 now adds a checked first-slice Lean artifact with small transition invariants
  - the broader W033 Lean theorem backlog remains open
  - repo-local TLC tooling now exists via `scripts/bootstrap-tla-tools.ps1` and `scripts/run-tlc.ps1`
  - `formal/tla/CoreEngineStage1.tla` has been exercised locally for W033 with `.\scripts\run-tlc.ps1 formal\tla\CoreEngineStage1.tla formal\tla\CoreEngineStage1.smoke.cfg`
  - `formal/tla/CoreEngineStage1.cfg` remains a deeper exploration config and is not yet declared as a routine terminating baseline
  - wider model-check coverage and stronger TLA+ property inventory remain open
  - replay artifacts now include a first emitted harness and oracle baseline run, but replay-pack export and richer replay families remain open
  - measurement artifacts remain schema/register definitions; running code emits scenario counters, but not the later full measurement surface
  - the artifact set is still a first assurance floor rather than a proof-, model-check-, replay-pack-, or instrumentation-complete lane
