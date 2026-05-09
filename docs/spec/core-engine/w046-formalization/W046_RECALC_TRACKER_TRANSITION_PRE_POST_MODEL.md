# W046 Recalc Tracker Transition Pre/Post Model

Status: `calc-gucd.4_recalc_tracker_transition_model_validated`

Workset: `W046`

Parent epic: `calc-gucd`

Bead: `calc-gucd.4`

## 1. Purpose

This packet models the recalc tracker and adjacent coordinator transition slice that follows invalidation/rebind and precedes evaluation-order/read-discipline proof work.

It covers:

1. `Stage1RecalcTracker` node-state transition preconditions and postconditions;
2. demand-set behavior for needed, verified-clean, rejected-pending-repair, and publication-clear paths;
3. candidate production as distinct from stable publication;
4. verified-clean no-publication behavior;
5. rejection as no-publication;
6. coordinator publication only from an accepted candidate;
7. exact limits that remain for later evaluation-order, working-value, and TraceCalc refinement beads.

The packet is scoped to W046 semantic proof-spine targets. It does not claim a full mechanized semantic proof of the Rust engine, full TLA verification, or release-grade readiness.

## 2. Implementation Crosswalk

| Semantic object or transition | Rust surface | Model surface | Evidence root |
| --- | --- | --- | --- |
| node calc states | `NodeCalcState` in `src/oxcalc-core/src/recalc.rs:12` | Lean `NodeCalcState`; TLA `NodeStates` | TreeCalc `node_states` result artifacts |
| dirty marking | `Stage1RecalcTracker::mark_dirty` in `src/oxcalc-core/src/recalc.rs:122` | Lean/TLA `markDirty` | invalidation closure and TreeCalc result states |
| demand entry | `mark_needed` in `src/oxcalc-core/src/recalc.rs:128` | Lean/TLA `markNeeded` | dirty/needed tracker tests and local TreeCalc artifacts |
| cycle-blocked closure record | `DependencyGraph::derive_invalidation_closure` cycle member state in `src/oxcalc-core/src/dependency.rs:270`; seed-state derivation in `src/oxcalc-core/src/dependency.rs:292` | Lean/TLA `recordCycleBlockedFromClosure` | `invalidation_closure_distinguishes_rebind_and_cycle_blocked_records` |
| evaluation entry | `begin_evaluate` in `src/oxcalc-core/src/recalc.rs:136` | Lean/TLA `beginEvaluate` | capability-fence overlay and evaluation-loop artifacts |
| verified clean | `verify_clean` in `src/oxcalc-core/src/recalc.rs:159`; TreeCalc verified-clean branch in `src/oxcalc-core/src/treecalc.rs:447` | Lean/TLA `verifyClean` | `tc_local_verified_clean_001` and `LocalTreeCalcRunState::VerifiedClean` |
| candidate-ready value | `produce_candidate_result` in `src/oxcalc-core/src/recalc.rs:168`; TreeCalc candidate branch in `src/oxcalc-core/src/treecalc.rs:452` | Lean/TLA `produceCandidateResult` | local candidate and candidate-result artifacts |
| dependency-shape candidate | `produce_dependency_shape_update` in `src/oxcalc-core/src/recalc.rs:193` | Lean/TLA `produceDependencyShapeUpdate` | dynamic resolved reference and dependency-shape artifacts |
| reject/fallback tracker state | `reject_or_fallback` in `src/oxcalc-core/src/recalc.rs:218`; reject runner in `src/oxcalc-core/src/treecalc.rs:944` | Lean/TLA `rejectOrFallback` | cycle, rebind, dynamic, and host-failure reject artifacts |
| rejected reentry | `reenter_rejected_pending_repair` in `src/oxcalc-core/src/recalc.rs:238` | Lean/TLA `reenterRejectedPendingRepair` | model target; not a broad repair workflow proof |
| tracker publication clear | `publish_and_clear` in `src/oxcalc-core/src/recalc.rs:248`; TreeCalc publish-clear loop in `src/oxcalc-core/src/treecalc.rs:510` | Lean/TLA `trackerPublishAndClear` | publication result artifacts |
| release/evict eligibility | `release_and_evict_eligible` in `src/oxcalc-core/src/recalc.rs:256` | Lean/TLA `releaseAndEvictEligible` | overlay eviction tracker test |
| candidate admission | `TreeCalcCoordinator::admit_candidate_work` in `src/oxcalc-core/src/coordinator.rs:230` | Lean/TLA `admitCandidateWork` | TreeCalc candidate path and rejection placeholder candidate path |
| accepted candidate record | `record_accepted_candidate_result` in `src/oxcalc-core/src/coordinator.rs:245` | Lean/TLA `recordAcceptedCandidateResult` | candidate/publication artifacts |
| coordinator reject | `reject_candidate_work` in `src/oxcalc-core/src/coordinator.rs:292` | Lean/TLA `rejectCandidateWork` | reject artifacts and reject counters |
| coordinator publish | `accept_and_publish` in `src/oxcalc-core/src/coordinator.rs:260`; TreeCalc publish path in `src/oxcalc-core/src/treecalc.rs:507` | Lean/TLA `acceptAndPublish` | publication bundle artifacts |

## 3. Phase Contracts

`T08.MarkDirtyNeeded`:

1. `mark_dirty` is a permissive local tracker write in Rust: it overwrites the node state with `DirtyPending` and protects the execution overlay without checking the prior state.
2. The phase-level precondition is external to `mark_dirty`: invalidation must choose formula owners from the current snapshot before evaluation is scheduled.
3. `mark_needed` has the local precondition `DirtyPending`.
4. `mark_needed` sets `Needed`, inserts the node into the demand set, and protects the execution overlay.

`T11.BeginEvaluate`:

1. local precondition: node state is `Needed`;
2. postcondition: node state is `Evaluating`;
3. postcondition: an execution overlay is protected;
4. postcondition: a capability/fence attachment is present for the evaluation basis.

Cycle-blocked closure record:

1. closure precondition: SCC/cycle classification has identified the node as a cycle member;
2. implementation location: `CycleBlocked` is currently assigned by invalidation closure records, not by a `Stage1RecalcTracker` mutator;
3. model postcondition: the recalc envelope records `CycleBlocked`, keeps demand present, and does not publish.

`T14.VerifyClean`:

1. local precondition: node state is `Evaluating`;
2. postcondition: node state is `VerifiedClean`;
3. postcondition: demand is cleared for the node;
4. postcondition: no candidate or publication is emitted by the verified-clean path.

`T15.ProduceCandidate`:

1. local precondition: node state is `Evaluating`;
2. postcondition: node state is `PublishReady`;
3. postcondition: candidate payload or dependency-shape overlay is attached;
4. postcondition: the candidate remains distinct from publication until coordinator acceptance and publication.

`T16.RejectCandidate`:

1. tracker precondition: node state is `Evaluating` or `PublishReady`;
2. coordinator precondition: matching in-flight or accepted candidate exists;
3. postcondition: tracker state becomes `RejectedPendingRepair` and demand remains present;
4. postcondition: dynamic dependency overlays for that node are removed by the fallback path;
5. postcondition: published view and publication counters do not advance on rejection.

`T17.PublishCandidate`:

1. coordinator precondition: accepted candidate exists;
2. postcondition: coordinator writes one publication bundle and advances publication counters;
3. postcondition: in-flight and accepted candidate state is cleared;
4. tracker clear precondition: node state is `PublishReady`;
5. tracker clear postcondition: node returns to `Clean`, demand is cleared, and execution overlay becomes eviction-eligible.

## 4. Checked Lean Artifact

Artifact: `formal/lean/OxCalc/CoreEngine/W046RecalcTrackerTransitions.lean`

Checked command:

```powershell
lean formal\lean\OxCalc\CoreEngine\W046RecalcTrackerTransitions.lean
```

Result: passed.

Lean definitions:

1. `NodeCalcState`: dirty/needed/evaluating/verified-clean/publish-ready/rejected/cycle-blocked vocabulary matching the Rust enum.
2. `RecalcAction`: tracker and coordinator action vocabulary.
3. `EngineRecalcState`: bounded state envelope for node state, demand, overlays, candidate state, publication version, and counters.
4. `ActionPrecondition`: local transition guards.
5. `ActionPostcondition`: local post-state relation.
6. `LegalTransition`: pre/post transition relation.
7. `RecalcTransitionSemanticModel`: proof-carrier envelope for a checked transition.

Checked theorem targets:

| Theorem | Contract |
| --- | --- |
| `markNeeded_requires_dirty` | needed transition requires dirty-pending input |
| `beginEvaluate_requires_needed` | evaluation starts only from needed state |
| `recordCycleBlockedFromClosure_is_no_publish` | cycle-blocked closure records retain demand and preserve publication state |
| `verifyClean_clears_demand` | verified-clean clears demand |
| `verifyClean_is_no_publish` | verified-clean preserves publication state |
| `produceCandidate_is_not_publication` | candidate production attaches payload and preserves publication state |
| `produceDependencyShapeUpdate_is_not_publication` | dependency-shape candidate attaches dynamic/candidate signal and preserves publication state |
| `rejectOrFallback_requires_evaluating_or_publish_ready` | fallback rejection starts only from evaluating or publish-ready tracker state |
| `rejectOrFallback_is_no_publish` | fallback rejection enters rejected-pending-repair, keeps demand, removes dynamic overlay, and preserves publication state |
| `trackerPublishAndClear_requires_publish_ready` | tracker publication clear requires publish-ready |
| `trackerPublishAndClear_clears_demand` | tracker publication clear returns to clean and clears demand |
| `rejectCandidateWork_requires_candidate` | coordinator rejection requires an in-flight or accepted candidate |
| `rejectCandidateWork_is_no_publish` | coordinator rejection clears candidate state without publication |
| `recordAcceptedCandidate_is_not_publication` | accepted-candidate record does not publish |
| `acceptAndPublish_requires_accepted_candidate` | coordinator publication requires an accepted candidate |
| `acceptAndPublish_advances_publication` | coordinator publication advances publication state and clears candidate state |

## 5. Checked TLA Artifact

Artifacts:

1. `formal/tla/CoreEngineW046RecalcTracker.tla`
2. `formal/tla/CoreEngineW046RecalcTracker.smoke.cfg`
3. `docs/test-runs/core-engine/tla/w046-recalc-tracker-001/`

Checked command:

```powershell
powershell -ExecutionPolicy Bypass -File scripts\run-tlc.ps1 formal\tla\CoreEngineW046RecalcTracker.tla formal\tla\CoreEngineW046RecalcTracker.smoke.cfg
```

Result: passed.

TLC summary:

| Field | Value |
| --- | --- |
| TLC version | `2.19 of 08 August 2024` |
| states generated | `77,096` |
| distinct states | `13,671` |
| queue left | `0` |
| complete-state depth | `7` |
| result | no error found |

Checked invariants:

1. `TypeInvariant`
2. `DemandClearedForVerifiedClean`
3. `RejectedPendingRepairRetainsDemand`
4. `CycleBlockedRetainsDemand`
5. `PublishReadyHasCandidateSignal`
6. `RejectIsNoPublish`
7. `VerifiedCleanIsNoPublish`
8. `CycleBlockedIsNoPublish`
9. `CandidateStepsAreNoPublish`
10. `PublishRequiresAcceptedCandidate`
11. `PublicationOnlyFromAcceptedCandidate`

Smoke model shape:

1. one node with the full tracker state vocabulary;
2. bounded transition depth of six actions;
3. explicit tracker/envelope actions for dirty, needed, cycle-blocked, evaluating, verified-clean, candidate, dependency-shape update, rejection, repair reentry, publication clear, and release/evict;
4. explicit coordinator actions for candidate admission, accepted-candidate record, rejection, and publication;
5. decision-history records capture before/after publication versions so no-publish properties are checkable without trusting prose.

## 6. Replay Roots

| Root | Use in this bead |
| --- | --- |
| `docs/test-runs/core-engine/treecalc-local/w037-optimized-core-conformance-treecalc-001/cases/tc_local_verified_clean_001/result.json` | verified-clean path emits no candidate or publication bundle and leaves published values unchanged |
| `docs/test-runs/core-engine/treecalc-local/w037-optimized-core-conformance-treecalc-001/cases/tc_local_dynamic_resolved_publish_001/result.json` | dynamic resolved path produces candidate result and publication bundle |
| `docs/test-runs/core-engine/treecalc-local/w037-optimized-core-conformance-treecalc-001/cases/tc_local_dynamic_resolved_publish_001/dependency_graph.json` | dependency-shape update pressure that leads to publish-ready candidate state |
| `docs/test-runs/core-engine/treecalc-local/w037-optimized-core-conformance-treecalc-001/cases/tc_local_rebind_after_rename_001/post_edit/result.json` | rebind gate rejection preserves published values |
| `docs/test-runs/core-engine/treecalc-local/w034-independent-conformance-treecalc-001/cases/tc_local_dynamic_reject_001/result.json` | dynamic unresolved rejection route |
| `docs/test-corpus/core-engine/tracecalc/hand-auditable/tc_w034_dynamic_dependency_negative_001.json` | TraceCalc dynamic negative reference scenario carried into later refinement |
| `docs/test-corpus/core-engine/tracecalc/hand-auditable/tc_w035_dirty_seed_closure_no_under_invalidation_001.json` | dirty seed/closure evidence feeding tracker dirty/needed transition expectations |

## 7. Assumptions And Limits

1. The Lean artifact models local pre/post relations; it does not prove the Rust functions line-by-line.
2. The TLA artifact is a bounded model, not full TLA verification.
3. `mark_dirty` is intentionally modeled as a permissive local Rust method. The stronger phase-level guard lives in invalidation/scheduling, not inside the method.
4. `CycleBlocked` is modeled as an invalidation-closure record state. Current Rust does not expose a `Stage1RecalcTracker` mutator for this state.
5. `record_accepted_candidate_result` is modeled as accepting the in-flight candidate without publishing; the current Rust coordinator keeps the in-flight candidate until publish or reject.
6. This bead does not prove full cycle policy. Cycle rejection and evaluation-order consequences remain successor work.
7. Working-value read discipline is not proven here; it is the next bead.
8. TraceCalc refinement and TreeCalc/CoreEngine observable equivalence are not claimed here.

## 8. Successor Obligations

| Successor bead | Consumes from this packet |
| --- | --- |
| `calc-gucd.5` | tracker preconditions for evaluating nodes, verified-clean no-candidate path, reject short-circuit, and publish-ready candidate state |
| `calc-gucd.6` | candidate/reject/publication event vocabulary for TraceCalc refinement |
| `calc-gucd.7` | OxFml effect handler law that formula evaluation may produce candidates/rejections but cannot publish directly |
| `calc-gucd.8` | proof-service evidence rows recast over concrete tracker/coordinator transition artifacts |
| `calc-gucd.9` | phase timing signatures for evaluation, rejection recording, candidate publication, verified-clean finalize, and tracker transition counts |

## 9. Validation

| Command | Result |
| --- | --- |
| `lean formal\lean\OxCalc\CoreEngine\W046RecalcTrackerTransitions.lean` | passed |
| `powershell -ExecutionPolicy Bypass -File scripts\run-tlc.ps1 formal\tla\CoreEngineW046RecalcTracker.tla formal\tla\CoreEngineW046RecalcTracker.smoke.cfg` | passed |
| `cargo test -p oxcalc-core recalc` | passed |
| `cargo test -p oxcalc-core cycle_blocked` | passed |
| `cargo test -p oxcalc-core verified_clean` | passed |
| `cargo test -p oxcalc-core reject` | passed |
| `powershell -ExecutionPolicy Bypass -File scripts\check-worksets.ps1` | passed |
| `br dep cycles --json` | passed with `count: 0` |
| JSON parse check for `w046-recalc-tracker-001` summary/validation | passed |
| TLC log non-empty check for `w046-recalc-tracker-001` | passed |
| `git diff --check` | passed; emitted line-ending normalization warnings only |

## 10. Semantic-Equivalence Statement

This bead changes formal artifacts, evidence metadata, spec/status documents, and bead state only.

Observable OxCalc behavior is invariant under this bead. It does not change recalc tracker behavior, coordinator behavior, dependency graph construction, invalidation closure, dynamic-reference resolution, soft-reference rebind derivation, evaluation order, working-value reads, TraceCalc execution, TreeCalc/CoreEngine execution, OxFml evaluation, rejection, publication, pack policy, or service readiness.

## 11. Reviewed Inbound Observations

Reviewed `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md`.

Relevant intake for this bead:

1. OxFml continues to require candidate/commit separation and reject-is-no-publish semantics.
2. OxFml remains authoritative for evaluator artifact meaning, typed reject outcomes, fence meaning, and replay-safe identity.
3. W046 recalc tracker modeling therefore treats candidate and accepted-candidate state as not publication, and models coordinator rejection as no-publication.
4. No new OxFml handoff is needed because this bead does not change shared FEC/F3E text or evaluator-facing clauses.

## 12. Pre-Closure Verification Checklist

| # | Check | Result |
|---|-------|--------|
| 1 | Spec text and realization notes updated for all in-scope items? | yes; this packet, W046 status surfaces, transition catalog, fragment ledger, and formal layout note the recalc tracker model |
| 2 | Pack expectations updated for affected packs? | yes; no pack expectation changed by this model bead |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes; replay roots are listed in Section 6 and the TLA run artifact is checked in |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes; no policy/strategy change, invariant statement in Section 10 |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes; no normative FEC/F3E seam change and no OxFml handoff needed |
| 6 | All required tests pass? | yes; see Section 9 |
| 7 | No known semantic gaps remain in declared scope? | yes for declared `calc-gucd.4` target; broader cycle policy, evaluation-order/read discipline, TraceCalc refinement, full Rust proof, and unbounded TLA remain explicit successor or limit lanes |
| 8 | Completion language audit passed? | yes; no full Rust, full recalc-engine, full TLA, or release-grade proof is claimed |
| 9 | `WORKSET_REGISTER.md` updated when ordered workset truth changed? | yes; no ordered workset register change required |
| 10 | `IN_PROGRESS_FEATURE_WORKLIST.md` updated when feature-map truth changed? | yes; current semantic bead moved to `calc-gucd.5` |
| 11 | execution-state blocker surface updated? | yes; `.beads/` owns `calc-gucd.4` state |

## 13. Completion Claim Self-Audit

| Step | Result |
| --- | --- |
| Scope re-read | pass; bead asks for dirty, needed, evaluating, verified-clean, publish-ready, rejected-pending-repair, cycle-blocked vocabulary, allowed transitions, demand clearing, no-publish rejection, verified-clean no-publish, and publish-only-from-accepted-candidate targets |
| Gate criteria re-read | pass; model artifacts, checked commands, replay roots, and explicit limits are recorded |
| Silent scope reduction check | pass; full cycle policy, working-value reads, TraceCalc refinement, Rust implementation proof, and unbounded TLA verification are explicitly not hidden |
| "Looks done but is not" pattern check | pass; the packet says the model is checked and scoped, not a full mechanized semantic proof of all recalc behavior |
| Include result | pass; checklist, self-audit, semantic-equivalence statement, and three-axis report are included |

## 14. Current Status

- execution_state: `calc-gucd.4_closed`
- scope_completeness: `scope_complete`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
