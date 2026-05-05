# W036 TLA Stage 2 Partition And Scheduler Equivalence Model

Status: `calc-rqq.5_tla_stage2_partition_model_validated`
Workset: `W036`
Parent epic: `calc-rqq`
Bead: `calc-rqq.5`

## 1. Purpose

This packet deepens the W035 TLA scheduler gate by adding a bounded Stage 2 partition/ownership model and explicit scheduler-readiness criteria.

The target is TLA model evidence over the Stage 2 precondition surface. It is not Stage 2 policy promotion, full TLA verification, pack-grade replay, or optimized/core-engine conformance promotion.

## 2. Authority Inputs Reviewed

| Input | Role in this bead |
|---|---|
| `docs/spec/core-engine/w035-formalization/W035_TLA_NON_ROUTINE_EXPLORATION_AND_SCHEDULER_PRECONDITIONS.md` | W035 abstract `PartitionCoverageSoundInput` and scheduler-gate source |
| `formal/tla/CoreEngineW035NonRoutineInterleavings.tla` | predecessor model for multi-reader overlay and Stage 2 preconditions |
| `docs/spec/core-engine/w036-formalization/W036_RESIDUAL_COVERAGE_AND_PROMOTION_BLOCKER_LEDGER.md` | W036 obligations `W036-OBL-002`, `W036-OBL-006`, `W036-OBL-007`, `W036-OBL-010`, and `W036-OBL-011` |
| `docs/spec/core-engine/w036-formalization/W036_TRACECALC_COVERAGE_CLOSURE_CRITERIA_AND_MATRIX_EXPANSION.md` | routes the multi-reader overlay row to this TLA lane |
| `docs/spec/core-engine/w036-formalization/W036_OPTIMIZED_CORE_ENGINE_CONFORMANCE_CLOSURE_PLAN_AND_FIRST_FIXES.md` | routes snapshot/capability fence projection rows to this TLA/coordinator lane |
| `docs/spec/core-engine/w036-formalization/W036_LEAN_THEOREM_COVERAGE_EXPANSION.md` | routes snapshot/capability fences and multi-reader overlays to this TLA lane |

## 3. TLA Artifact Surface

| Artifact | Role |
|---|---|
| `formal/tla/CoreEngineW036Stage2Partition.tla` | W036 bounded model for two-partition ownership, scheduler-readiness criteria, stale snapshot and capability-view fences, and multi-reader overlay release ordering |
| `formal/tla/CoreEngineW036Stage2Partition.scheduler_blocked.cfg` | sound ownership with missing semantic replay evidence |
| `formal/tla/CoreEngineW036Stage2Partition.partition_cross_dep.cfg` | sound ownership with a cross-partition dependency blocker |
| `formal/tla/CoreEngineW036Stage2Partition.bounded_ready.cfg` | bounded readiness criteria with all modeled evidence present |
| `formal/tla/CoreEngineW036Stage2Partition.fence_reject.cfg` | stale snapshot and capability-view candidate rejection |
| `formal/tla/CoreEngineW036Stage2Partition.multi_reader.cfg` | two-reader/two-overlay retention and release ordering under the same partition model |
| `docs/test-runs/core-engine/tla/w036-stage2-partition-001/` | raw TLC logs plus machine-readable run summary, validation, and promotion blockers |

The model makes partition ownership concrete in the bounded model through two explicit owned-node sets. The cross-partition dependency condition remains a bounded model input, so this is a scheduler-equivalence criteria model rather than a full partitioning algorithm proof.

## 4. Checked Invariants

| Invariant | Meaning |
|---|---|
| `TypeInvariant` | model variables stay in declared state families |
| `ConcretePartitionOwnershipSound` | each node has a modeled partition owner and scheduled/finished nodes remain owned |
| `FinishedNodesWereScheduled` | a node cannot be marked finished before it was scheduled |
| `BoundedReadyRequiresPreconditions` | bounded scheduler readiness implies sound ownership, no cross-partition dependency, finished partition work, scheduler criteria evidence, and semantic replay evidence |
| `BlockedStage2DecisionIsNoPromotion` | blocked Stage 2 decisions cannot promote policy |
| `Stage2PolicyPromotionRequiresReplayEvidence` | any policy promotion would require semantic replay evidence |
| `NoStage2PolicyPromotion` | the W036 model never promotes Stage 2 policy |
| `RejectIsNoPublish` | stale snapshot/capability-view reject decisions do not publish |
| `AcceptedCandidateRequiresFences` | accepted candidates satisfy both snapshot and capability-view fences |
| `NoStaleSnapshotAccepted` | stale snapshot candidates are not accepted |
| `NoCapabilityViewFenceAccepted` | stale capability-view candidates are not accepted |
| `ProtectedOverlayPinnedAndRetained` | protected overlays remain pinned and not eviction-eligible |
| `ReleasedReaderDoesNotReleaseOtherReaderProtectedOverlay` | releasing one reader does not release another reader's protected overlay |
| `EvictedOverlayWasUnprotected` | evicted overlays were unprotected before eviction |

## 5. Scheduler Criteria

The W036 model records bounded Stage 2 readiness as:

1. partition ownership model is sound,
2. every bounded partition owns at least one node,
3. no modeled cross-partition dependency blocks independent scheduling,
4. all owned partition nodes have finished,
5. `partition_ownership_model` evidence is present,
6. `scheduler_equivalence_criteria` evidence is present,
7. `semantic_replay_equivalence` evidence is present.

Even when those criteria are met, `stage2PolicyPromoted` remains false. Policy promotion stays owned by later replay, differential, and pack-grade gates.

## 6. TLC Runs

Commands:

```powershell
scripts\run-tlc.ps1 formal\tla\CoreEngineW036Stage2Partition.tla formal\tla\CoreEngineW036Stage2Partition.scheduler_blocked.cfg
scripts\run-tlc.ps1 formal\tla\CoreEngineW036Stage2Partition.tla formal\tla\CoreEngineW036Stage2Partition.partition_cross_dep.cfg
scripts\run-tlc.ps1 formal\tla\CoreEngineW036Stage2Partition.tla formal\tla\CoreEngineW036Stage2Partition.bounded_ready.cfg
scripts\run-tlc.ps1 formal\tla\CoreEngineW036Stage2Partition.tla formal\tla\CoreEngineW036Stage2Partition.fence_reject.cfg
scripts\run-tlc.ps1 formal\tla\CoreEngineW036Stage2Partition.tla formal\tla\CoreEngineW036Stage2Partition.multi_reader.cfg
```

The checked raw logs are under `docs/test-runs/core-engine/tla/w036-stage2-partition-001/`.

| Config | TLC version | States generated | Distinct states | Queue left | Depth | Result |
|---|---|---:|---:|---:|---:|---|
| `scheduler_blocked` | `2.19 of 08 August 2024` | 60,632 | 10,975 | 0 | 6 | no error found |
| `partition_cross_dep` | `2.19 of 08 August 2024` | 60,632 | 10,975 | 0 | 6 | no error found |
| `bounded_ready` | `2.19 of 08 August 2024` | 15,306 | 3,395 | 0 | 6 | no error found |
| `fence_reject` | `2.19 of 08 August 2024` | 54,690 | 6,490 | 0 | 5 | no error found |
| `multi_reader` | `2.19 of 08 August 2024` | 120,062 | 15,872 | 0 | 6 | no error found |

Machine-readable summary:

1. `docs/test-runs/core-engine/tla/w036-stage2-partition-001/run_summary.json`
2. `docs/test-runs/core-engine/tla/w036-stage2-partition-001/promotion_blockers.json`
3. `docs/test-runs/core-engine/tla/w036-stage2-partition-001/validation.json`

## 7. Obligation Disposition

| Obligation | Disposition |
|---|---|
| `W036-OBL-002` | the multi-reader overlay release-order row now has bounded TLA model evidence; it is still not TraceCalc replay evidence and does not create a full TraceCalc oracle claim |
| `W036-OBL-006` | stale snapshot-fence rejection has a bounded TLA/coordinator counterpart; optimized/core-engine conformance remains unpromoted until later differential or fixture evidence |
| `W036-OBL-007` | capability-view fence rejection has a bounded TLA/coordinator counterpart; optimized/core-engine conformance remains unpromoted until later differential or fixture evidence |
| `W036-OBL-010` | W036 records concrete bounded partition ownership, scheduler criteria, model limits, TLC state counts, and promotion blockers |
| `W036-OBL-011` | Stage 2 partition/replay equivalence remains blocked for policy promotion; W036 records criteria and no-promotion evidence rather than a promoted scheduler policy |

## 8. Promotion Consequence

No Stage 2 policy, pack capability, full TLA verification, full TraceCalc oracle, or optimized/core-engine conformance claim is promoted by this bead.

The W036 TLA evidence improves the model surface by replacing W035's single abstract partition-soundness boolean with bounded concrete ownership and explicit scheduler-readiness criteria. It still does not prove a production partitioning algorithm, does not provide deterministic Stage 2 replay equivalence, and does not provide pack-grade replay evidence.

## 9. OxFml Watch

No OxFml handoff is filed by this bead.

This bead constructs no W073 conditional-formatting request payloads and does not exercise OxFml formatting metadata. The existing W073 `typed_rule` watch remains unchanged.

## 10. Semantic-Equivalence Statement

This bead adds TLA model/config files, TLC evidence artifacts, and spec/status text only.

It does not change coordinator scheduling, invalidation strategy, dependency graph construction, soft-reference update behavior, recalc behavior, publication semantics, reject policy, overlay lifecycle behavior, TraceCalc execution semantics, TreeCalc/CoreEngine runtime behavior, Lean artifacts, pack-decision logic, continuous-assurance runner semantics, or OxFml/OxFunc evaluator behavior.

Observable runtime behavior is invariant under this bead. The TLA artifacts model bounded interleavings and criteria; they do not alter executable calculator behavior.

## 11. Verification

| Command | Result |
|---|---|
| `scripts\run-tlc.ps1 formal\tla\CoreEngineW036Stage2Partition.tla formal\tla\CoreEngineW036Stage2Partition.scheduler_blocked.cfg` | passed |
| `scripts\run-tlc.ps1 formal\tla\CoreEngineW036Stage2Partition.tla formal\tla\CoreEngineW036Stage2Partition.partition_cross_dep.cfg` | passed |
| `scripts\run-tlc.ps1 formal\tla\CoreEngineW036Stage2Partition.tla formal\tla\CoreEngineW036Stage2Partition.bounded_ready.cfg` | passed |
| `scripts\run-tlc.ps1 formal\tla\CoreEngineW036Stage2Partition.tla formal\tla\CoreEngineW036Stage2Partition.fence_reject.cfg` | passed |
| `scripts\run-tlc.ps1 formal\tla\CoreEngineW036Stage2Partition.tla formal\tla\CoreEngineW036Stage2Partition.multi_reader.cfg` | passed |
| `cargo fmt --all -- --check` | passed |
| `scripts/check-worksets.ps1` | passed |
| `br dep cycles --json` | passed; `count=0` |
| `git diff --check` | passed; CRLF normalization warnings only |

No Rust tests are required for this TLA/documentation-only bead because it introduces no Rust behavior, fixture runner, or replay emission code.

## 12. Pre-Closure Verification Checklist

| # | Check | Result |
|---|---|---|
| 1 | Spec text and realization notes updated for all in-scope items? | yes; this packet, W036 status surfaces, W036 residual ledger, spec index, and feature-map surfaces record the W036 TLA evidence |
| 2 | Pack expectations updated for affected packs? | yes; pack remains unpromoted and `calc-rqq.8` still owns reassessment |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes for carried replay-backed rows; this bead adds checked TLA model artifacts and raw TLC logs rather than runtime replay artifacts |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes; Section 10 states no runtime strategy changed and Section 8 records no Stage 2 policy promotion |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes; no OxFml-owned seam contradiction or handoff trigger appeared |
| 6 | All required tests pass? | yes; TLC and hygiene validation commands passed |
| 7 | No known semantic gaps remain in declared scope? | yes for this target; broader Stage 2 replay, pack, differential, independent evaluator, continuous-assurance, and full-verification gaps are explicitly carried forward |
| 8 | Completion language audit passed? | yes; no full TLA verification, full concurrency proof, pack-grade replay, optimized/core-engine conformance, or Stage 2 promotion is claimed |
| 9 | `WORKSET_REGISTER.md` updated when ordered workset truth changed? | not applicable; ordered workset truth is unchanged |
| 10 | `IN_PROGRESS_FEATURE_WORKLIST.md` updated when feature-map truth changed? | yes; IP-04 points at this W036 TLA evidence |
| 11 | execution-state blocker surface updated? | yes; `.beads/` records `calc-rqq.5` execution state and later closure evidence |

## 13. Completion Claim Self-Audit

| Step | Audit result |
|---|---|
| Scope re-read | pass; `calc-rqq.5` asks for concrete bounded partition/ownership modeling and scheduler equivalence criteria |
| Gate criteria re-read | pass; TLC artifacts record checked invariants, state counts, model limits, and promotion blockers |
| Silent scope reduction check | pass; the cross-partition dependency input, bounded transition depth, and no-promotion consequences are explicit |
| "Looks done but is not" pattern check | pass; bounded TLA checks are not represented as full verification, pack-grade replay, or scheduler promotion |
| Result | pass for the `calc-rqq.5` TLA Stage 2 partition/scheduler-equivalence target |

## 14. Three-Axis Report

- execution_state: `calc-rqq.5_tla_stage2_partition_model_validated`
- scope_completeness: `scope_partial`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- open_lanes:
  - `calc-rqq.6` through `calc-rqq.9` remain open
  - full TLA verification remains partial because W036 adds bounded model-check evidence, not total proof
  - full TraceCalc oracle coverage remains partial because the multi-reader row is model-checked rather than TraceCalc-replayed
  - optimized/core-engine conformance remains partial because snapshot/capability fence rows have TLA counterparts but no promoted differential/fixture match
  - concrete production Stage 2 partitioning, deterministic scheduler replay equivalence, pack-grade replay, continuous assurance operation, cross-engine differential service, independent evaluator diversity, and Stage 2 policy remain unpromoted
