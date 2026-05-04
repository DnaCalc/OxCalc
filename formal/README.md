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
   - first Lean-facing state-object skeleton for the current Stage 1 structural, coordinator, and recalc vocabulary.
   - exercised with a local `lean formal/lean/OxCalc/CoreEngine/Stage1State.lean` typecheck run.
2. `formal/lean/OxCalc/CoreEngine/W033FirstSlice.lean`
   - W033 first-slice Lean vocabulary for abstract OxFml seam facts, LET/LAMBDA carrier facts, coordinator state, overlays, conservative invalidation, and small transition invariants.
   - exercised with a local `lean formal/lean/OxCalc/CoreEngine/W033FirstSlice.lean` typecheck run.
3. `formal/lean/OxCalc/CoreEngine/W033PostSlice.lean`
   - post-W033 Lean widening for FEC bridge/fence compatibility, dependency closure, overlay retention, callable carrier visibility, replay equivalence, and no Stage 2 contention promotion.
   - exercised with a local `lean formal/lean/OxCalc/CoreEngine/W033PostSlice.lean` typecheck run.
4. `formal/lean/OxCalc/CoreEngine/W034PublicationFences.lean`
   - W034 publication-fence proof slice for snapshot, compatibility, capability-view, reject/no-publish, and atomic-publication facts.
   - exercised with a local `lean formal/lean/OxCalc/CoreEngine/W034PublicationFences.lean` typecheck run.
5. `formal/lean/OxCalc/CoreEngine/W034DependenciesOverlays.lean`
   - W034 dependency-closure and protected-overlay proof slice for static, runtime, dynamic-shape update, retention, and safety facts.
   - exercised with a local `lean formal/lean/OxCalc/CoreEngine/W034DependenciesOverlays.lean` typecheck run.
6. `formal/lean/OxCalc/CoreEngine/W034LetLambdaReplay.lean`
   - W034 `LET`/`LAMBDA` callable-carrier and replay-equivalent independent-order proof slice.
   - exercised with a local `lean formal/lean/OxCalc/CoreEngine/W034LetLambdaReplay.lean` typecheck run.
7. `formal/lean/OxCalc/CoreEngine/W034RefinementObligations.lean`
   - W034 conformance-classification and refinement-obligation proof slice.
   - exercised with a local `lean formal/lean/OxCalc/CoreEngine/W034RefinementObligations.lean` typecheck run.
8. `formal/tla/CoreEngineStage1.tla`
   - first TLA+ skeleton for Stage 1 coordinator, publication, reject, pin, and recalc actions.
9. `formal/tla/CoreEngineStage1.cfg`
   - deeper exploration configuration for the Stage 1 TLA+ module.
10. `formal/tla/CoreEngineStage1.smoke.cfg`
   - bounded smoke-model configuration for quick local TLC verification.
11. `formal/tla/CoreEnginePostW033.tla`
   - post-W033 smoke model for imported candidate facts, dependency closure, callable carrier visibility, protected overlays, compatible-fence publication, reject/no-publish decisions, and no Stage 2 contention promotion.
12. `formal/tla/CoreEnginePostW033.smoke.cfg`
   - bounded smoke-model configuration for `CoreEnginePostW033`.
13. `formal/tla/CoreEngineW034Interleavings.tla`
   - W034 interleaving model for stale fences, dependency update interleavings, pinned overlay retention/release, and Stage 2 contention-precondition blocking.
14. `formal/tla/CoreEngineW034Interleavings.smoke.cfg`
   - bounded routine smoke-model configuration for `CoreEngineW034Interleavings`.
15. `formal/replay/stage1-hand-authored/`
   - first hand-authored replay artifact slice for `R1`, `R2`, and `R7`.
16. `docs/test-runs/core-engine/tracecalc-reference-machine/w013-sequence-a-baseline/`
   - first emitted harness and oracle baseline run covering `R1`, `R2`, `R7`, `R4`, and `R5` through the `TraceCalc` corpus.
17. `formal/measurement/stage1_counter_schema.json`
   - first machine-readable Stage 1 counter schema.
18. `formal/measurement/stage1_experiment_register.json`
   - first machine-readable Stage 1 experiment register.

## Status
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - the Stage 1 Lean skeleton has been typechecked once locally, W033 adds a checked first-slice Lean artifact, the post-W033 successor slice widens checked theorem coverage, and W034 adds checked adjacent proof-family slices for fences, dependencies, overlays, LET/LAMBDA carriers, replay equivalence, and refinement classification
  - the broader Lean theorem backlog remains open
  - repo-local TLC tooling now exists via `scripts/bootstrap-tla-tools.ps1` and `scripts/run-tlc.ps1`
  - `formal/tla/CoreEngineStage1.tla`, `formal/tla/CoreEnginePostW033.tla`, and `formal/tla/CoreEngineW034Interleavings.tla` have bounded smoke configs for routine TLC checks
  - `formal/tla/CoreEngineStage1.cfg` remains a deeper exploration config and is not yet declared as a routine terminating baseline
  - wider model-check coverage and stronger TLA+ property inventory remain open
  - replay artifacts now include a first emitted harness and oracle baseline run, but replay-pack export and richer replay families remain open
  - measurement artifacts remain schema/register definitions; running code emits scenario counters, but not the later full measurement surface
  - the artifact set is still a first assurance floor rather than a fully matured proof, model-check, replay-pack, or instrumentation lane
