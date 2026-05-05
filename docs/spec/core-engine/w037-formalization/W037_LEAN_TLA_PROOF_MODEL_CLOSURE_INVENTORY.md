# W037 Lean/TLA Proof Model Closure Inventory

Status: `calc-ubd.5_proof_model_inventory_validated`
Workset: `W037`
Parent epic: `calc-ubd`
Bead: `calc-ubd.5`

## 1. Purpose

This packet records the W037 Lean/TLA proof and model closure inventory slice.

The target is to convert the W036 Lean and TLA inventories into explicit W037 closure classifications where runnable artifacts support the claim, and to keep assumptions, external seams, bounds, proof gaps, Stage 2 replay gaps, pack gates, and C5 gates explicit. It is not a full Lean verification claim, a full TLA verification claim, a general OxFunc callable-kernel claim, a Stage 2 policy promotion, a pack-grade replay promotion, or a C5 promotion.

## 2. Authority Inputs Reviewed

| Input | Role |
|---|---|
| `docs/worksets/W037_CORE_FORMALIZATION_FULL_VERIFICATION_PROMOTION_GATES.md` | W037 scope and exit-gate authority |
| `docs/spec/core-engine/w037-formalization/W037_RESIDUAL_FULL_VERIFICATION_AND_PROMOTION_GATE_LEDGER.md` | W037 obligation map, including `W037-OBL-006`, `W037-OBL-007`, and `W037-OBL-008` |
| `docs/spec/core-engine/w036-formalization/W036_LEAN_THEOREM_COVERAGE_EXPANSION.md` | W036 Lean proof-inventory predecessor |
| `docs/spec/core-engine/w036-formalization/W036_TLA_STAGE2_PARTITION_AND_SCHEDULER_EQUIVALENCE_MODEL.md` | W036 TLA model predecessor |
| `formal/lean/OxCalc/CoreEngine/` | checked Lean proof and inventory surface from Stage 1 through W037 |
| `formal/tla/` | checked routine TLC model/config surface from Stage 1 through W036 |
| `docs/test-runs/core-engine/upstream-host/w037-direct-oxfml-evaluator-001/` | direct OxFml and narrow `LET`/`LAMBDA` runtime evidence consumed by the proof/model inventory |

## 3. Artifact Surface

| Artifact | Result |
|---|---|
| `formal/lean/OxCalc/CoreEngine/W037ProofModelClosureInventory.lean` | checked W037 closure-inventory rows and non-promotion theorems |
| `docs/test-runs/core-engine/formal-inventory/w037-proof-model-closure-001/run_summary.json` | 12 Lean files checked, 11 routine TLC configs checked, 0 Lean explicit axioms, 0 Lean `sorry` placeholders, 0 TLC failed configs |
| `docs/test-runs/core-engine/formal-inventory/w037-proof-model-closure-001/lean_inventory.json` | Lean file list, closure rows, and claim limits |
| `docs/test-runs/core-engine/formal-inventory/w037-proof-model-closure-001/tla_inventory.json` | TLC config list, state counts, bounded-model rows, and claim limits |
| `docs/test-runs/core-engine/formal-inventory/w037-proof-model-closure-001/promotion_blockers.json` | full Lean, full TLA, general OxFunc, Stage 2, pack, C5, and spec-evolution blockers |
| `docs/test-runs/core-engine/formal-inventory/w037-proof-model-closure-001/validation.json` | validation command ledger |

## 4. Closure Classification

| Obligation | W037 decision |
|---|---|
| `W037-OBL-006` narrow `LET`/`LAMBDA` carrier seam | closed for the `calc-ubd.5` proof/model inventory target by combining W036 checked callable-boundary inventory with `calc-ubd.4` direct runtime evidence; general OxFunc callable kernels remain opaque and unpromoted |
| `W037-OBL-007` Lean proof closure inventory | closed for the `calc-ubd.5` target: 12 Lean files typecheck, the W037 closure inventory typechecks, and the Lean surface has 0 explicit axioms and 0 `sorry`/`admit` placeholders by scan; full Lean verification remains open |
| `W037-OBL-008` TLA model closure inventory | closed for the `calc-ubd.5` target: 11 routine TLC configs from Stage 1 through W036 pass; the evidence is bounded model checking, so full TLA verification remains open |
| `W037-OBL-009` Stage 2 replay/equivalence | explicitly deferred to `calc-ubd.6`; proof/model inventory alone does not promote Stage 2 policy |
| `W037-OBL-013` pack/C5 | explicitly deferred to `calc-ubd.8`; proof/model inventory alone does not promote pack-grade replay or C5 |
| `W037-OBL-016` spec evolution | current proof/model claims are recorded without freezing future spec evolution |
| `W037-OBL-017` cross-repo seam authority | OxFml/OxFunc-owned semantics remain external where not exercised by OxCalc artifacts |

## 5. Lean Surface

The checked Lean surface is:

1. `Stage1State.lean`
2. `W033FirstSlice.lean`
3. `W033PostSlice.lean`
4. `W034PublicationFences.lean`
5. `W034DependenciesOverlays.lean`
6. `W034LetLambdaReplay.lean`
7. `W034RefinementObligations.lean`
8. `W035AssumptionDischarge.lean`
9. `W035SeamProofMap.lean`
10. `W036LeanCoverageExpansion.lean`
11. `W036CallableBoundaryInventory.lean`
12. `W037ProofModelClosureInventory.lean`

The W037 file proves that the closure rows are target-limited, that callable-carrier evidence does not promote the full OxFunc kernel, and that full Lean verification, full TLA verification, Stage 2 policy, pack-grade replay, C5, and general OxFunc kernels remain unpromoted.

## 6. TLA Surface

The checked routine TLA surface is:

1. `CoreEngineStage1.smoke.cfg`
2. `CoreEnginePostW033.smoke.cfg`
3. `CoreEngineW034Interleavings.smoke.cfg`
4. `CoreEngineW035NonRoutineInterleavings.scheduler_gate.cfg`
5. `CoreEngineW035NonRoutineInterleavings.partition_gap.cfg`
6. `CoreEngineW035NonRoutineInterleavings.multi_reader.cfg`
7. `CoreEngineW036Stage2Partition.scheduler_blocked.cfg`
8. `CoreEngineW036Stage2Partition.partition_cross_dep.cfg`
9. `CoreEngineW036Stage2Partition.bounded_ready.cfg`
10. `CoreEngineW036Stage2Partition.fence_reject.cfg`
11. `CoreEngineW036Stage2Partition.multi_reader.cfg`

This is the routine bounded TLC floor. `formal/tla/CoreEngineStage1.cfg` remains a deeper exploration config and is not promoted here as a routine terminating baseline.

## 7. Semantic-Equivalence Statement

This bead adds a checked Lean inventory file, machine-readable proof/model inventory artifacts, and spec/status text only.

It does not change coordinator scheduling, invalidation strategy, dependency graph construction, soft-reference update behavior, recalc behavior, publication semantics, reject policy, overlay lifecycle behavior, TraceCalc execution semantics, TreeCalc/CoreEngine runtime behavior, TLA model semantics, pack-decision logic, continuous-assurance runner semantics, or OxFml/OxFunc evaluator behavior.

Observable runtime behavior is invariant under this bead. The new Lean artifact classifies proof/model closure and non-promotion boundaries; it does not alter executable calculator behavior.

## 8. Verification

| Command | Result |
|---|---|
| `lean formal\lean\OxCalc\CoreEngine\Stage1State.lean` through `lean formal\lean\OxCalc\CoreEngine\W037ProofModelClosureInventory.lean` | passed; 12 Lean files |
| `rg -n "^\s*(axiom|sorry|admit)\b" formal/lean` | passed with no matches |
| `scripts\run-tlc.ps1` routine config set listed in Section 6 | passed; 11 configs, 0 failed configs |
| JSON parse for `docs/test-runs/core-engine/formal-inventory/w037-proof-model-closure-001/*.json` | passed |
| `cargo fmt --all -- --check` | passed |
| `scripts/check-worksets.ps1` | passed after bead closure; worksets=15, beads total=99, open=5, in_progress=0, ready=1, blocked=3, closed=94 |
| `br ready --json` | passed; next ready bead is `calc-ubd.6` |
| `br dep cycles --json` | passed; `count: 0` |
| `git diff --check` | passed; line-ending warnings only |

No Rust tests are required for this bead because it introduces no Rust source or runtime behavior.

## 9. Pre-Closure Verification Checklist

| # | Check | Result |
|---|---|---|
| 1 | Spec text and realization notes updated for all in-scope items? | yes; this packet, formal artifact layout, W037 workset, residual ledger, spec index, and feature worklist record the inventory |
| 2 | Pack expectations updated for affected packs? | yes; pack/C5 remain unpromoted and `calc-ubd.8` owns reassessment |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes for carried runtime behavior; this bead adds checked Lean inventory and bounded TLC artifacts rather than runtime replay |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes; Section 7 states no runtime strategy changed |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes; no OxFml/FEC/F3E change or handoff trigger exists for this bead |
| 6 | All required tests pass? | yes; see Section 8 |
| 7 | No known semantic gaps remain in declared scope? | yes for the `calc-ubd.5` target; broader full Lean/TLA, Stage 2, pack, C5, operated-service, and full-verification gaps remain open |
| 8 | Completion language audit passed? | yes; the packet limits claims to `calc-ubd.5` proof/model inventory and keeps broader proof/model/promotion lanes open |
| 9 | `WORKSET_REGISTER.md` updated when ordered workset truth changed? | not applicable; ordered workset truth is unchanged |
| 10 | `IN_PROGRESS_FEATURE_WORKLIST.md` updated when feature-map truth changed? | yes; IP-04 records the W037 proof/model inventory slice and non-promotion limits |
| 11 | execution-state blocker surface updated? | yes; the W037 workset and bead closure notes carry this target status |

## 10. Completion Claim Self-Audit

| Step | Audit result |
|---|---|
| Scope re-read | pass; `calc-ubd.5` asks for Lean/TLA proof/model closure inventory with assumptions, bounds, and proof gaps stated |
| Gate criteria re-read | pass; `W037-OBL-006`, `W037-OBL-007`, `W037-OBL-008`, `W037-OBL-016`, and `W037-OBL-017` are classified with runnable Lean/TLA artifacts where closure is claimed |
| Silent scope reduction check | pass; full Lean/TLA verification, general OxFunc kernels, Stage 2 replay/policy, pack-grade replay, C5, operated services, and full verification remain explicit open lanes |
| "Looks done but is not" pattern check | pass; checked inventory and bounded TLC runs are not represented as total proof, full TLA verification, pack-grade replay, C5, or Stage 2 policy promotion |
| Result | pass for the `calc-ubd.5` target only; W037 remains scope-partial |

## 11. Three-Axis Report

- execution_state: `calc-ubd.5_proof_model_inventory_validated`
- scope_completeness: `scope_partial`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- open_lanes:
  - `calc-ubd.6` Stage 2 deterministic replay and partition promotion criteria is the next ready W037 target
  - full Lean verification remains open
  - full TLA verification remains open
  - Stage 2 deterministic replay/equivalence remains open
  - operated continuous assurance, operated cross-engine differential service, pack-grade replay governance, C5 candidate decision, and W037 closure audit remain open
