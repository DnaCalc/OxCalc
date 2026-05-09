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

## Post-W034 Formalization Additions

1. `formal/lean/OxCalc/CoreEngine/W035AssumptionDischarge.lean`
   - W035 assumption discharge and residual seam classification.
2. `formal/lean/OxCalc/CoreEngine/W035SeamProofMap.lean`
   - W035 seam proof map.
3. `formal/lean/OxCalc/CoreEngine/W036LeanCoverageExpansion.lean`
   - W036 proof inventory for match guards, harness rows, external seams, opaque kernels, and TLA deferrals.
4. `formal/lean/OxCalc/CoreEngine/W036CallableBoundaryInventory.lean`
   - W036 callable carrier, metadata deferral, OxFml seam, and OxFunc opaque-kernel boundary inventory.
5. `formal/lean/OxCalc/CoreEngine/W037ProofModelClosureInventory.lean`
   - W037 checked closure inventory for proof/model rows, open boundaries, and non-promotion claims.
6. `formal/tla/CoreEngineW035NonRoutineInterleavings.tla`
   - W035 model for multi-reader overlay release ordering and Stage 2 precondition gates.
7. `formal/tla/CoreEngineW036Stage2Partition.tla`
   - W036 bounded model for concrete partition ownership, scheduler-readiness criteria, stale snapshot/capability-view fences, and multi-reader overlay release ordering.
8. `docs/test-runs/core-engine/formal-inventory/w037-proof-model-closure-001/`
   - W037 proof/model inventory packet: 12 Lean files checked, 11 routine TLC configs checked, 0 explicit Lean axioms, 0 Lean `sorry`/`admit` placeholders, and no full Lean/TLA or pack/Stage 2 promotion.
9. `formal/lean/OxCalc/CoreEngine/W037Stage2PromotionCriteria.lean`
   - W037 checked Stage 2 promotion predicate and current no-promotion theorem.
10. `docs/test-runs/core-engine/stage2-criteria/w037-stage2-deterministic-replay-criteria-001/`
   - W037 Stage 2 criteria packet: 7 criteria rows, 3 satisfied rows, 4 blocked rows, explicit observable-result invariance requirements, and no Stage 2 promotion candidate.
11. `archive/lean-w038-w045/W038AssumptionDischargeAndTotality.lean`
   - W038 proof/model assumption-discharge, totality-boundary, exact-blocker, external-seam, and non-promotion proof slice.
12. `archive/test-runs-core-engine-w038-w045/formal-assurance/w038-proof-model-assumption-discharge-001/`
   - W038 formal-assurance packet: 8 assumption rows, 3 local-proof rows, 2 bounded-model rows, 1 external-seam row, 3 totality boundaries, 6 exact blockers, 0 failed rows, and no full Lean/TLA, Stage 2, pack, C5, or general OxFunc promotion.
13. `archive/lean-w038-w045/W040RustTotalityAndRefinement.lean`
   - W040 Rust totality/refinement classification proof slice for Result/error carrier evidence, dependency rebind refinement, totality/refinement blockers, LET/LAMBDA carrier boundary, and spec-evolution guard.
14. `archive/test-runs-core-engine-w038-w045/formal-assurance/w040-rust-totality-refinement-proof-tranche-001/`
   - W040 Rust totality/refinement packet: 10 Rust/proof rows, 7 local checked-proof rows, 5 totality boundaries, 5 refinement rows, 5 exact blockers, 0 failed rows, and no Rust totality, optimized/core, Stage 2, pack, C5, broad OxFml, general OxFunc, or release-grade promotion.
15. `archive/lean-w038-w045/W040LeanTlaFullVerificationDischarge.lean`
   - W040 Lean/TLA proof-model classification proof slice for Lean inventory, Rust bridge, Stage 2 policy predicate, bounded TLA evidence, exact proof/model blockers, LET/LAMBDA external seam, and spec-evolution guard.
16. `archive/test-runs-core-engine-w038-w045/formal-assurance/w040-lean-tla-full-verification-discharge-001/`
   - W040 Lean/TLA packet: 11 proof/model rows, 6 local checked-proof rows, 3 bounded-model rows, 1 accepted external seam, 2 accepted boundaries, 5 totality boundaries, 5 exact blockers, 0 failed rows, and no full Lean, full TLA, Rust totality, Stage 2, pack, C5, general OxFunc, or release-grade promotion.
17. `archive/lean-w038-w045/W040Stage2ProductionPolicyAndEquivalence.lean`
   - W040 Stage 2 policy/equivalence predicate for declared-profile replay, fence counterparts, bounded analyzer evidence, full production analyzer soundness, fairness/scheduler coverage, operated service dependency, pack governance, and no-promotion claims.
18. `archive/test-runs-core-engine-w038-w045/stage2-replay/w040-stage2-production-policy-equivalence-001/`
   - W040 Stage 2 packet: 12 policy rows, 8 satisfied rows, snapshot/capability fence counterparts evidenced, bounded analyzer evidence, 4 exact blockers, 0 failed rows, and no Stage 2 production policy, operated service, pack, C5, or release-grade promotion.

## W046 Engine Semantic Spine Additions

1. `formal/lean/OxCalc/CoreEngine/W046DependencyGraph.lean`
   - W046 graph-build semantic model for descriptors, forward edges, reverse-edge converse, diagnostic preservation, and cycle-group classification shape.
   - exercised with a local `lean formal\lean\OxCalc\CoreEngine\W046DependencyGraph.lean` typecheck run.
2. `formal/tla/CoreEngineW046DependencyGraph.tla`
   - W046 bounded graph-build model for forward-edge valid targets, reverse-edge converse, untargeted dynamic diagnostics, and non-trivial SCC classification shape.
3. `formal/tla/CoreEngineW046DependencyGraph.smoke.cfg`
   - bounded smoke configuration for `CoreEngineW046DependencyGraph`.
4. `docs/test-runs/core-engine/tla/w046-dependency-graph-001/`
   - W046 TLC evidence packet: 1 config checked, 3 states generated, 2 distinct states, depth 2, no error, and no full TLA/Rust/SCC/release-grade promotion.
5. `formal/lean/OxCalc/CoreEngine/W046InvalidationRebind.lean`
   - W046 invalidation/rebind semantic model for reverse reachability, no-under-invalidation, rebind reason classification, dynamic dependency transition reasons, and stale-binding no-publish.
   - exercised with a local `lean formal\lean\OxCalc\CoreEngine\W046InvalidationRebind.lean` typecheck run.
6. `formal/tla/CoreEngineW046InvalidationRebind.tla`
   - W046 bounded invalidation/rebind model for reverse-reachability closure, dynamic transition seeds, upstream dependent propagation, rebind flag soundness, and rebind-gate rejection.
7. `formal/tla/CoreEngineW046InvalidationRebind.smoke.cfg`
   - bounded smoke configuration for `CoreEngineW046InvalidationRebind`.
8. `docs/test-runs/core-engine/tla/w046-invalidation-rebind-001/`
   - W046 TLC evidence packet: 1 config checked, 4 states generated, 3 distinct states, depth 3, no error, and no full TLA/Rust/dynamic-reference/release-grade promotion.
9. `formal/lean/OxCalc/CoreEngine/W046RecalcTrackerTransitions.lean`
   - W046 recalc tracker and coordinator pre/post model for dirty, needed, evaluating, verified-clean, publish-ready, rejected-pending-repair, candidate, reject, and accepted-candidate publication transitions.
   - exercised with a local `lean formal\lean\OxCalc\CoreEngine\W046RecalcTrackerTransitions.lean` typecheck run.
10. `formal/tla/CoreEngineW046RecalcTracker.tla`
   - W046 bounded recalc tracker model for demand clearing, cycle-blocked demand/no-publish, publish-ready candidate signals, verified-clean no-publish, reject no-publish, candidate-not-publication, and publish-only-from-accepted-candidate checks.
11. `formal/tla/CoreEngineW046RecalcTracker.smoke.cfg`
   - bounded smoke configuration for `CoreEngineW046RecalcTracker`.
12. `docs/test-runs/core-engine/tla/w046-recalc-tracker-001/`
   - W046 TLC evidence packet: 1 config checked, 77,096 states generated, 13,671 distinct states, depth 7, no error, and no full TLA/Rust/release-grade promotion.
13. `formal/lean/OxCalc/CoreEngine/W046EvaluationOrderReadDiscipline.lean`
   - W046 evaluation-order and working-value read-discipline semantic model for dependency-before-dependent order, stable/prior reads, diagnostic short-circuit, cycle reject no-evaluation, verified-clean no-publication, and no-torn candidate targets.
   - exercised with a local `lean formal\lean\OxCalc\CoreEngine\W046EvaluationOrderReadDiscipline.lean` typecheck run.
14. `formal/tla/CoreEngineW046EvaluationOrder.tla`
   - W046 bounded evaluation-order model for acyclic order selection, stable/prior read events, changed-value publication, verified-clean finalization, diagnostic rejection, cycle rejection, and terminal decision exclusivity.
15. `formal/tla/CoreEngineW046EvaluationOrder.smoke.cfg`
   - bounded smoke configuration for `CoreEngineW046EvaluationOrder`.
16. `docs/test-runs/core-engine/tla/w046-evaluation-order-001/`
   - W046 TLC evidence packet: 1 config checked, 24 states generated, 16 distinct states, depth 5, no error, and no full TLA/Rust/TraceCalc/OxFml/release-grade promotion.
17. `formal/lean/OxCalc/CoreEngine/W046TraceCalcRefinement.lean`
   - W046 TraceCalc refinement kernel for observable packets, exact value/diagnostic/reject/publication matching, dependency/invalidation/trace-family preservation, and exact-blocker classification.
   - exercised with a local `lean formal\lean\OxCalc\CoreEngine\W046TraceCalcRefinement.lean` typecheck run.
18. `formal/tla/CoreEngineW046TraceCalcRefinement.tla`
   - W046 bounded TraceCalc-to-engine observable-refinement model for accept/publish, verified-clean no-publication, reject no-publication, dynamic dependency, and invalidation-closure row shapes.
19. `formal/tla/CoreEngineW046TraceCalcRefinement.smoke.cfg`
   - bounded smoke configuration for `CoreEngineW046TraceCalcRefinement`.
20. `docs/test-runs/core-engine/tla/w046-tracecalc-refinement-001/`
   - W046 TLC evidence packet: 1 config checked, 11 states generated, 6 distinct states, depth 2, no error, and no full TLA/Rust/TraceCalc/TreeCalc/release-grade promotion.
21. `docs/test-runs/core-engine/refinement/w046-tracecalc-refinement-kernel-001/`
   - W046 selected-kernel binding packet: 12 rows, 8 matched refinement rows, 1 TraceCalc oracle self-check row, 3 exact blockers, 0 unexpected mismatches, and no full TraceCalc/TreeCalc/CoreEngine or release-grade promotion.

## Status
- execution_state: `calc-gucd.7_oxfml_seam_ready`
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: integrated
- open_lanes:
  - the Stage 1 Lean skeleton has been typechecked once locally, W033 adds a checked first-slice Lean artifact, the post-W033 successor slice widens checked theorem coverage, W034 adds checked adjacent proof-family slices, W035/W036 add proof-inventory slices, W037 adds a checked proof/model closure inventory, W038 adds checked assumption-discharge and totality-boundary classification, W040 adds checked Rust, Lean/TLA, and Stage 2 policy/equivalence classification slices, and W046 now adds checked dependency-graph, invalidation/rebind, recalc-tracker, evaluation-order/read-discipline, and TraceCalc-refinement model slices
  - full Lean verification remains open
  - repo-local TLC tooling now exists via `scripts/bootstrap-tla-tools.ps1` and `scripts/run-tlc.ps1`
  - `formal/tla/CoreEngineStage1.tla`, `formal/tla/CoreEnginePostW033.tla`, `formal/tla/CoreEngineW034Interleavings.tla`, `formal/tla/CoreEngineW035NonRoutineInterleavings.tla`, `formal/tla/CoreEngineW036Stage2Partition.tla`, `formal/tla/CoreEngineW046DependencyGraph.tla`, `formal/tla/CoreEngineW046InvalidationRebind.tla`, `formal/tla/CoreEngineW046RecalcTracker.tla`, `formal/tla/CoreEngineW046EvaluationOrder.tla`, and `formal/tla/CoreEngineW046TraceCalcRefinement.tla` have bounded configs for routine TLC checks
  - `formal/tla/CoreEngineStage1.cfg` remains a deeper exploration config and is not yet declared as a routine terminating baseline
  - full TLA verification remains open
  - W046 OxFml seam and scale semantic-regression models remain open; TraceCalc refinement is currently scoped to the selected-kernel relation and binding packet
  - W040 now binds declared-profile Stage 2 policy/equivalence evidence and snapshot/capability fence counterparts, but full production partition analyzer soundness, fairness/unbounded scheduler coverage, operated cross-engine service evidence, and pack-grade replay governance remain open; Stage 2 policy remains unpromoted
  - replay artifacts now include a first emitted harness and oracle baseline run, but replay-pack export and richer replay families remain open
  - measurement artifacts remain schema/register definitions; running code emits scenario counters, but not the later full measurement surface
  - the artifact set is still a first assurance floor rather than a fully matured proof, model-check, replay-pack, or instrumentation lane
