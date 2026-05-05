# W035 TLA Non-Routine Exploration And Scheduler Preconditions

Status: `calc-tkq.5_tla_non_routine_exploration_validated`
Workset: `W035`
Parent epic: `calc-tkq`
Bead: `calc-tkq.5`

## 1. Purpose

This packet extends the W034 routine TLA smoke work into a bounded non-routine exploration slice.

The target is to make two W035 pressures explicit:

1. multi-reader overlay release ordering must not allow one reader's release to evict another reader's protected overlay,
2. Stage 2 scheduler promotion requires visible equivalence preconditions rather than bounded smoke evidence alone.

This is not a full concurrency proof and does not promote Stage 2 policy. It records model limits, checked invariants, state counts, and exact no-promotion consequences.

## 2. Authority Inputs Reviewed

| Input | Role in this bead |
|---|---|
| `docs/worksets/W035_CORE_FORMALIZATION_PROOF_AND_ASSURANCE_HARDENING.md` | W035 scope and `calc-tkq.5` gate |
| `docs/spec/core-engine/w035-formalization/W035_RESIDUAL_PROOF_OBLIGATION_AND_SPEC_EVOLUTION_LEDGER.md` | W035 obligations for overlay interleavings and Stage 2 preconditions |
| `docs/spec/core-engine/w035-formalization/W035_TRACECALC_ORACLE_MATRIX_EXPANSION.md` | routes multi-reader overlay release ordering to `calc-tkq.5` |
| `docs/spec/core-engine/w035-formalization/W035_LEAN_ASSUMPTION_DISCHARGE_AND_SEAM_PROOF_MAP.md` | routes multi-reader overlay interleavings to the TLA lane |
| `docs/spec/core-engine/w034-formalization/W034_TLA_MODEL_FAMILY_AND_CONTENTION_PRECONDITIONS.md` | predecessor TLA smoke model and state counts |
| `formal/tla/CoreEngineStage1.tla` | Stage 1 baseline model |
| `formal/tla/CoreEnginePostW033.tla` | post-W033 baseline model |
| `formal/tla/CoreEngineW034Interleavings.tla` | predecessor interleaving model |

## 3. TLA Artifact Surface

| Artifact | Role |
|---|---|
| `formal/tla/CoreEngineW035NonRoutineInterleavings.tla` | W035 model for multi-reader overlay release ordering and Stage 2 precondition gates |
| `formal/tla/CoreEngineW035NonRoutineInterleavings.multi_reader.cfg` | two-reader/two-overlay exploration with missing Stage 2 evidence |
| `formal/tla/CoreEngineW035NonRoutineInterleavings.scheduler_gate.cfg` | sound partition abstraction with one missing scheduler-equivalence evidence item |
| `formal/tla/CoreEngineW035NonRoutineInterleavings.partition_gap.cfg` | all evidence present but partition-coverage soundness deliberately false |

The W035 model is intentionally smaller and sharper than the W034 interleaving model. It isolates the non-routine surfaces that W034 left open instead of increasing every W034 axis at once.

## 4. Checked Invariants

| Invariant | Meaning |
|---|---|
| `TypeInvariant` | model variables stay in declared state families |
| `ProtectedOverlayPinnedAndRetained` | protected overlays remain pinned, not eviction-eligible, and not evicted |
| `ReleasedReaderDoesNotReleaseOtherReaderProtectedOverlay` | releasing one reader cannot remove another reader's protection |
| `EvictedOverlayWasUnprotected` | eviction only applies after protection release |
| `BlockedStage2DecisionIsNoPromotion` | a blocked Stage 2 decision cannot publish promotion state |
| `Stage2PromotionRequiresPreconditions` | promotion state implies all evidence and partition preconditions |
| `Stage2BlockedWhenEvidenceMissing` | missing evidence keeps Stage 2 unpromoted |
| `PromotionReadyRequiresSoundPartitions` | promotion-ready state implies the partition-coverage gate |

## 5. Scheduler Preconditions

The W035 model records these Stage 2 preconditions as model gates:

1. `RequiredStage2Evidence` must be fully present in `availableEvidence`.
2. partition coverage must be sound.
3. missing evidence blocks promotion as `blocked_missing_preconditions`.
4. unsound partition coverage blocks promotion as `blocked_partition_gap`.
5. promotion-ready state is reachable only when both evidence and partition gates hold.

The partition gate is represented as the abstract constant `PartitionCoverageSoundInput`. This is deliberate: W035 tests the scheduler gate obligation, not the full future partitioning algorithm. A later Stage 2 promotion candidate would need a concrete partition/ownership model and replay evidence.

## 6. TLC Runs

Commands:

```powershell
scripts\run-tlc.ps1 formal\tla\CoreEngineStage1.tla formal\tla\CoreEngineStage1.smoke.cfg
scripts\run-tlc.ps1 formal\tla\CoreEnginePostW033.tla formal\tla\CoreEnginePostW033.smoke.cfg
scripts\run-tlc.ps1 formal\tla\CoreEngineW034Interleavings.tla formal\tla\CoreEngineW034Interleavings.smoke.cfg
scripts\run-tlc.ps1 formal\tla\CoreEngineW035NonRoutineInterleavings.tla formal\tla\CoreEngineW035NonRoutineInterleavings.scheduler_gate.cfg
scripts\run-tlc.ps1 formal\tla\CoreEngineW035NonRoutineInterleavings.tla formal\tla\CoreEngineW035NonRoutineInterleavings.partition_gap.cfg
scripts\run-tlc.ps1 formal\tla\CoreEngineW035NonRoutineInterleavings.tla formal\tla\CoreEngineW035NonRoutineInterleavings.multi_reader.cfg
```

Observed results:

| Model/config | TLC version | States generated | Distinct states | Queue left | Depth | Result |
|---|---|---:|---:|---:|---:|---|
| `CoreEngineStage1.smoke` | `2.19 of 08 August 2024` | 4,855 | 908 | 0 | 5 | no error found |
| `CoreEnginePostW033.smoke` | `2.19 of 08 August 2024` | 814,981 | 49,614 | 0 | 5 | no error found |
| `CoreEngineW034Interleavings.smoke` | `2.19 of 08 August 2024` | 247,984 | 19,373 | 0 | 5 | no error found |
| `CoreEngineW035NonRoutineInterleavings.scheduler_gate` | `2.19 of 08 August 2024` | 5,224 | 1,443 | 0 | 6 | no error found |
| `CoreEngineW035NonRoutineInterleavings.partition_gap` | `2.19 of 08 August 2024` | 8,205 | 1,924 | 0 | 6 | no error found |
| `CoreEngineW035NonRoutineInterleavings.multi_reader` | `2.19 of 08 August 2024` | 30,633 | 5,408 | 0 | 6 | no error found |

The W035 configs use `MaxTransitions = 5`. TLC reports complete state-graph depth 6 because it includes the initial state plus the bounded transition sequence.

## 7. Obligation Disposition

| Obligation | `calc-tkq.5` disposition |
|---|---|
| `W035-OBL-003` | multi-reader overlay release ordering is now checked in a two-reader/two-overlay TLA config; broader overlay economics and production GC policy remain later work |
| `W035-OBL-009` | W035 adds non-routine configs beyond W034 smoke and records exact state counts and model limits |
| `W035-OBL-010` | Stage 2 scheduler promotion remains blocked unless evidence and partition preconditions are visible |

## 8. Promotion Consequence

No Stage 2 policy is promoted by this bead.

The W035 TLA evidence strengthens the no-promotion rationale:

1. W035 has a checked abstraction for missing-evidence and partition-gap blocks.
2. W035 does not yet have a concrete Stage 2 partition model.
3. W035 does not yet have deterministic replay equivalence for Stage 2 schedules.
4. W035 does not yet have pack-grade replay or continuous cross-engine differential evidence.

Any future promotion candidate must replace the abstract partition-soundness input with concrete partition facts and must provide semantic-equivalence evidence showing that observable results are invariant under the scheduler strategy.

## 9. Semantic-Equivalence Statement

This bead adds TLA artifacts and spec text only. It does not change coordinator scheduling, dependency graph construction, soft-reference resolution, recalc semantics, publication semantics, reject policy, overlay lifecycle behavior, TraceCalc semantics, TreeCalc semantics, Lean artifacts, pack-decision logic, or OxFml evaluator behavior.

Observable runtime behavior is invariant under this bead. The TLA configs explore abstract interleavings and preconditions; they do not alter executable calculator behavior.

## 10. Verification

| Command | Result |
|---|---|
| `scripts\run-tlc.ps1 formal\tla\CoreEngineStage1.tla formal\tla\CoreEngineStage1.smoke.cfg` | passed |
| `scripts\run-tlc.ps1 formal\tla\CoreEnginePostW033.tla formal\tla\CoreEnginePostW033.smoke.cfg` | passed |
| `scripts\run-tlc.ps1 formal\tla\CoreEngineW034Interleavings.tla formal\tla\CoreEngineW034Interleavings.smoke.cfg` | passed |
| `scripts\run-tlc.ps1 formal\tla\CoreEngineW035NonRoutineInterleavings.tla formal\tla\CoreEngineW035NonRoutineInterleavings.scheduler_gate.cfg` | passed |
| `scripts\run-tlc.ps1 formal\tla\CoreEngineW035NonRoutineInterleavings.tla formal\tla\CoreEngineW035NonRoutineInterleavings.partition_gap.cfg` | passed |
| `scripts\run-tlc.ps1 formal\tla\CoreEngineW035NonRoutineInterleavings.tla formal\tla\CoreEngineW035NonRoutineInterleavings.multi_reader.cfg` | passed |
| `cargo fmt --all -- --check` | passed |
| `scripts/check-worksets.ps1` | passed |
| `br dep cycles --json` | passed; `count=0` |
| `git diff --check` | passed; CRLF normalization warnings only |

No Rust tests are required for this TLA/documentation-only bead because it introduces no Rust behavior, fixture runner, or replay emission code.

## 11. Pre-Closure Verification Checklist

| # | Check | Result |
|---|---|---|
| 1 | Spec text and realization notes updated for all in-scope items? | yes; this packet, W035 workset status, W035 ledger, spec index, and feature-map surfaces record the W035 TLA evidence |
| 2 | Pack expectations updated for affected packs? | yes; no pack promotion is claimed and `calc-tkq.7` still owns reassessment |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes for carried W035 behavior evidence; this bead adds checked TLA model evidence rather than new runtime replay artifacts |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes; Section 9 states no runtime policy or strategy changed and Section 8 records future scheduler-equivalence requirements |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes; no OxFml-owned seam contradiction or handoff trigger appeared |
| 6 | All required tests pass? | yes; TLC and hygiene validation commands passed |
| 7 | No known semantic gaps remain in declared scope? | yes for this target; broader Stage 2, partition, pack, and continuous-assurance gaps are explicitly carried forward |
| 8 | Completion language audit passed? | yes; no full TLA verification, full concurrency proof, pack-grade replay, or Stage 2 promotion is claimed |
| 9 | `WORKSET_REGISTER.md` updated when ordered workset truth changed? | not applicable; ordered workset truth is unchanged |
| 10 | `IN_PROGRESS_FEATURE_WORKLIST.md` updated when feature-map truth changed? | yes; IP-04 points at this TLA evidence |
| 11 | execution-state blocker surface updated? | yes; `.beads/` records `calc-tkq.5` execution state and later closure evidence |

## 12. Completion Claim Self-Audit

| Step | Audit result |
|---|---|
| Scope re-read | pass; `calc-tkq.5` asks for non-routine TLA exploration and scheduler/Stage 2 equivalence preconditions |
| Gate criteria re-read | pass; model limits, state counts, invariants, and no-promotion consequences are recorded |
| Silent scope reduction check | pass; the abstract partition-soundness input and bounded transition depth are explicitly documented |
| "Looks done but is not" pattern check | pass; bounded TLA checks are not represented as full formal verification or scheduler promotion |
| Result | pass for the `calc-tkq.5` TLA target |

## 13. Three-Axis Report

- execution_state: `calc-tkq.5_tla_non_routine_exploration_validated`
- scope_completeness: `scope_partial`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- open_lanes:
  - `calc-tkq.6` through `calc-tkq.8` remain open
  - full Lean/TLA verification remains open
  - full TraceCalc oracle coverage remains open
  - full optimized/core-engine verification and fully independent evaluator diversity remain open beyond W035 conformance-hardening dispositions
  - concrete Stage 2 partition modeling, pack-grade replay, continuous scale assurance, and Stage 2 policy remain unpromoted
