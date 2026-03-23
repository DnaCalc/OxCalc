# W025: TreeCalc Structural and Formula Substrate Widening

## Purpose
Create the first real TreeCalc structural substrate beyond the proving-floor shape so OxCalc can carry formula-bearing nodes, relative-reference context, and immutable structural edits without relying on `TraceCalc`-only semantics.

## Position and Dependencies
- **Depends on**: W002, W017, W024
- **Blocks**: W026, W027, W028, W029, W030, W031
- **Cross-repo**: none

## Scope
### In scope
1. stable TreeCalc node identity beyond the current proving-floor root-with-children shape
2. node taxonomy for container, constant, and formula-bearing nodes
3. formula-artifact and bind-artifact attachment points on nodes or explicit structural payloads
4. structural context needed for direct and tree-relative reference interpretation
5. immutable structural edit operations and snapshot-respin rules for the first TreeCalc-ready phase
6. structural diagnostics or artifacts sufficient to support later dependency build and replay identity

### Out of scope
1. OxFml bind-package consumption itself
2. real dependency graph derivation
3. evaluator-backed candidate intake
4. runtime-derived dependency overlay closure
5. first TreeCalc-ready baseline run

## Deliverables
1. a widened TreeCalc structural model in Rust that can represent formula-bearing nodes and relative-reference context
2. explicit structural edit and snapshot-respin rules for rename, move, formula replacement, add, and remove
3. deterministic structural diagnostics or artifact surfaces suitable for later dependency and replay binding
4. workset-level mapping from the widened structural model into later bind/dependency/candidate lanes

## Gate Model
### Entry gate
- W024 remains only a replay-facing residual lane and is not a blocker to TreeCalc semantic-completion packetization
- the first TreeCalc semantic-completion target remains defined in `CORE_ENGINE_TREECALC_SEMANTIC_COMPLETION_PLAN.md`

### Exit gate
- immutable snapshots can represent the first real TreeCalc node/formula substrate
- structural edit and identity rules are explicit enough to implement rebind/recalc consequences
- at least one deterministic structural artifact or diagnostic exists for the widened substrate

## Pre-Closure Verification Checklist
1. Spec text and realization notes updated for all in-scope items: no
2. Pack expectations updated for affected packs: no
3. At least one deterministic replay artifact exists per in-scope behavior: no
4. Semantic-equivalence statement provided for policy or strategy changes: no
5. FEC/F3E cross-repo impact assessed and handoff filed if needed: no
6. All required tests pass: no
7. No known semantic gaps remain in declared scope: no
8. Completion language audit passed: no
9. `IN_PROGRESS_FEATURE_WORKLIST.md` updated: no
10. `CURRENT_BLOCKERS.md` updated if needed: no

## Status
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - OxFml bind-package consumption is still deferred to W026
  - local TreeCalc formula ownership and reference lowering now exist, but real OxFml-produced bind artifacts are not consumed yet
  - structural and formula artifacts now include a checked-in local TreeCalc fixture corpus covering direct publish, verified-clean, ancestor-relative, sibling-offset, host-sensitive reject, dynamic reject, rename-triggered rebind, recalc-only constant-edit, recalc-only dependency-chain, recalc-only post-edit runtime-effect and overlay, mixed publication-then-post-edit overlay, move-triggered rebind, and removal families, but replay-visible TreeCalc artifacts are still open
- claim_confidence: moderate
- reviewed_inbound_observations: latest OxFml downstream notes consumed as seam baseline; no new immediate handoff trigger exists yet
