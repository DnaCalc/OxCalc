# W037 Stage 2 Deterministic Replay And Partition Promotion Criteria

Status: `calc-ubd.6_stage2_criteria_validated`
Workset: `W037`
Parent epic: `calc-ubd`
Bead: `calc-ubd.6`

## 1. Purpose

This packet records the W037 Stage 2 deterministic replay and partition promotion criteria slice.

The target is to decide whether Stage 2 scheduler/partition promotion can be supported by deterministic replay and semantic-equivalence evidence beyond the W036 bounded configs. The answer for this bead is no: promotion criteria are now runnable and explicit, but deterministic partition replay, production partition soundness, operated cross-engine service evidence, and pack-grade replay governance are still absent.

## 2. Authority Inputs Reviewed

| Input | Role |
|---|---|
| `docs/worksets/W037_CORE_FORMALIZATION_FULL_VERIFICATION_PROMOTION_GATES.md` | W037 scope and Stage 2 promotion guardrails |
| `docs/spec/core-engine/w037-formalization/W037_RESIDUAL_FULL_VERIFICATION_AND_PROMOTION_GATE_LEDGER.md` | W037 obligations `W037-OBL-002`, `W037-OBL-008`, `W037-OBL-009`, `W037-OBL-016`, and `W037-OBL-017` |
| `docs/spec/core-engine/w037-formalization/W037_LEAN_TLA_PROOF_MODEL_CLOSURE_INVENTORY.md` | predecessor proof/model classification showing Stage 2 replay remains open |
| `docs/spec/core-engine/w036-formalization/W036_TLA_STAGE2_PARTITION_AND_SCHEDULER_EQUIVALENCE_MODEL.md` | W036 bounded Stage 2 model source |
| `docs/test-runs/core-engine/tla/w036-stage2-partition-001/` | deterministic W036 TLC evidence |
| `docs/test-runs/core-engine/pack-capability/w036-pack-capability-reassessment-001/decision/pack_capability_decision.json` | prior pack/Stage 2 no-promotion decision |
| `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md` | reviewed inbound observations, including the current formatting boundary and W073 typed conditional-formatting input-contract direction |
| `../OxFml/docs/worksets/W073_conditional_formatting_typed_visualization_payload.md` | reviewed latest W073 typed-only aggregate/visualization metadata update; no Stage 2 criteria impact or new OxFml handoff trigger |

## 3. Artifact Surface

| Artifact | Result |
|---|---|
| `formal/lean/OxCalc/CoreEngine/W037Stage2PromotionCriteria.lean` | checked Stage 2 promotion predicate and current no-promotion theorem |
| `docs/test-runs/core-engine/stage2-criteria/w037-stage2-deterministic-replay-criteria-001/run_summary.json` | 7 criteria rows, 3 satisfied criteria rows, 4 blocked criteria rows, no Stage 2 promotion candidate |
| `docs/test-runs/core-engine/stage2-criteria/w037-stage2-deterministic-replay-criteria-001/promotion_criteria.json` | criteria dispositions and promotion consequences |
| `docs/test-runs/core-engine/stage2-criteria/w037-stage2-deterministic-replay-criteria-001/semantic_equivalence_requirements.json` | observable-result invariance obligations required before any future Stage 2 scheduler/partition claim |
| `docs/test-runs/core-engine/stage2-criteria/w037-stage2-deterministic-replay-criteria-001/replay_evidence_map.json` | map from existing replay/model/conformance/pack evidence to Stage 2 criteria |
| `docs/test-runs/core-engine/stage2-criteria/w037-stage2-deterministic-replay-criteria-001/promotion_decision.json` | no-promotion decision and successor lanes |
| `docs/test-runs/core-engine/stage2-criteria/w037-stage2-deterministic-replay-criteria-001/validation.json` | validation command ledger |

## 4. Criteria Decision

| Criteria | Current disposition |
|---|---|
| concrete partition model | satisfied as bounded TLA evidence only |
| scheduler equivalence criteria | satisfied as criteria definition only |
| observable-result invariance statement | satisfied as promotion requirement definition only |
| deterministic partition replay | blocked |
| production cross-partition dependency proof | blocked |
| operated cross-engine Stage 2 differential service | blocked; `calc-ubd.7` owns the next service lane |
| pack-grade replay governance | blocked; `calc-ubd.8` owns reassessment |

The criteria packet therefore records `stage2_promotion_blocked`, not a promotion candidate.

## 5. Semantic-Equivalence Requirement

A Stage 2 scheduler or partition strategy is promotable only if observable results are invariant between the unpartitioned baseline execution and the Stage 2 execution for every promoted profile.

The required observable surface includes published value payloads and extents, candidate outcomes, reject codes, dependency consequences, topology/spill consequences, overlay observations visible to pinned readers, exercised format/display consequences, runtime effects, and replay-bundle validation status.

The current packet defines that obligation. It does not demonstrate the obligation with partitioned replay.

## 6. Stage 2 No-Promotion Decision

Stage 2 policy remains unpromoted because:

1. no baseline-versus-Stage-2 partitioned replay exists,
2. no partition-order permutation replay exists,
3. no production partition analyzer soundness proof exists,
4. no operated cross-engine Stage 2 differential service exists,
5. no pack-grade Stage 2 replay bundle validation exists.

## 7. Semantic-Equivalence Statement

This bead adds a checked Lean criteria file, machine-readable Stage 2 criteria artifacts, and spec/status text only.

It does not change coordinator scheduling, partition policy, invalidation strategy, dependency graph construction, soft-reference update behavior, recalc behavior, publication semantics, reject policy, overlay lifecycle behavior, TraceCalc execution semantics, TreeCalc/CoreEngine runtime behavior, TLA model semantics, pack-decision logic, continuous-assurance runner semantics, or OxFml/OxFunc evaluator behavior.

Observable runtime behavior is invariant under this bead. The packet defines evidence required before any future Stage 2 scheduler or partition strategy can claim semantic equivalence.

## 8. Verification

| Command | Result |
|---|---|
| `lean formal\lean\OxCalc\CoreEngine\W037Stage2PromotionCriteria.lean` | passed |
| `scripts\run-tlc.ps1 formal\tla\CoreEngineW036Stage2Partition.tla formal\tla\CoreEngineW036Stage2Partition.scheduler_blocked.cfg` | passed |
| `scripts\run-tlc.ps1 formal\tla\CoreEngineW036Stage2Partition.tla formal\tla\CoreEngineW036Stage2Partition.partition_cross_dep.cfg` | passed |
| `scripts\run-tlc.ps1 formal\tla\CoreEngineW036Stage2Partition.tla formal\tla\CoreEngineW036Stage2Partition.bounded_ready.cfg` | passed |
| JSON parse for `docs/test-runs/core-engine/stage2-criteria/w037-stage2-deterministic-replay-criteria-001/*.json` | passed |
| `cargo test -p oxcalc-core upstream_host -- --nocapture` | passed; current local OxFml W073 formatting update remains compatible with the checked-in W037 upstream-host guard |
| `cargo fmt --all -- --check` | passed |
| `scripts/check-worksets.ps1` | passed after bead closure; worksets=15, beads total=99, open=4, in_progress=0, ready=1, blocked=2, closed=95 |
| `br ready --json` | passed; next ready bead is `calc-ubd.7` |
| `br dep cycles --json` | passed; `count: 0` |
| `git diff --check` | passed; line-ending warnings only |

No Rust tests are required for this bead because it introduces no Rust source or runtime behavior.

## 9. Pre-Closure Verification Checklist

| # | Check | Result |
|---|---|---|
| 1 | Spec text and realization notes updated for all in-scope items? | yes; this packet, the W037 workset, the residual ledger, the feature worklist, the formal README, and machine-readable criteria artifacts record the slice |
| 2 | Pack expectations updated for affected packs? | yes; pack/C5 remain unpromoted and `calc-ubd.8` owns reassessment |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes for carried baseline/model evidence; this bead records criteria and explicit absence of partitioned Stage 2 replay |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes; Sections 5 and 7 state the required observable-result invariance and no strategy change |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes; no OxFml/FEC/F3E change or handoff trigger exists for this criteria bead |
| 6 | All required tests pass? | yes; commands in Section 8 passed |
| 7 | No known semantic gaps remain in declared scope? | yes for the `calc-ubd.6` target; the missing partition replay, production partition proof, operated service, pack governance, C5, and Stage 2 policy lanes are explicit blockers |
| 8 | Completion language audit passed? | yes; the packet limits claims to criteria/no-promotion evidence and keeps Stage 2 policy, pack/C5, operated-service, and full-verification lanes open |
| 9 | `WORKSET_REGISTER.md` updated when ordered workset truth changed? | not applicable; ordered workset truth is unchanged |
| 10 | `IN_PROGRESS_FEATURE_WORKLIST.md` updated when feature-map truth changed? | yes; IP-04 records the Stage 2 criteria packet and no-promotion consequence |
| 11 | execution-state blocker surface updated? | yes; `.beads/` records `calc-ubd.6` closed and `calc-ubd.7` ready |

## 10. Completion Claim Self-Audit

| Step | Audit result |
|---|---|
| Scope re-read | pass; the target is Stage 2 deterministic replay and partition promotion criteria, not Stage 2 policy promotion |
| Gate criteria re-read | pass; runnable criteria artifacts exist, the Lean predicate is checked, three Stage 2 TLC configs are checked, and absent replay/proof/service/pack governance are explicit blockers |
| Silent scope reduction check | pass; deterministic partition replay, production partition analyzer soundness, operated cross-engine service, pack governance, C5, and full-verification lanes remain explicit open lanes |
| "Looks done but is not" pattern check | pass; bounded TLA evidence and criteria text are not over-read as semantic-equivalence proof or policy promotion |
| Result | pass for the `calc-ubd.6` target only; W037 remains scope-partial |

## 11. Three-Axis Report

- execution_state: `calc-ubd.6_stage2_criteria_validated`
- scope_completeness: `scope_partial`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- open_lanes:
  - `calc-ubd.7` operated continuous assurance and cross-engine service pilot is the next ready W037 bead
  - deterministic partition replay remains absent
  - production partition analyzer soundness remains absent
  - pack-grade replay governance and C5 remain open
  - W037 closure audit and full-verification release decision remain open
