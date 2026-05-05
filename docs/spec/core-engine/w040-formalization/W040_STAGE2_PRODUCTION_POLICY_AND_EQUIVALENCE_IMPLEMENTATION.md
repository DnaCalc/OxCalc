# W040 Stage 2 Production Policy And Equivalence Implementation

Status: `calc-tv5.5_stage2_policy_equivalence_validated_no_promotion`
Workset: `W040`
Parent epic: `calc-tv5`
Bead: `calc-tv5.5`

## 1. Purpose

This packet attacks the W040 Stage 2 production-policy and equivalence target.

The result is deliberately non-promoting. It adds a W040 Stage 2 runner profile, a checked Lean predicate, generated policy/equivalence registers, and direct counterpart evidence for the Stage 2 snapshot-fence and capability-view fence rows.

The target is not to claim Stage 2 production policy, full production partition-analyzer soundness, full TLA verification, operated Stage 2 differential service, pack-grade replay, C5, release-grade verification, broad OxFml display/publication closure, callable metadata projection, or general OxFunc ownership. The target is to move the W039 Stage 2 state beyond carried bounded replay by binding W040 dynamic/soft-reference evidence, fence counterpart evidence, bounded analyzer evidence, observable-equivalence evidence, and exact remaining blockers before `calc-tv5.6` starts.

## 2. Authority Inputs Reviewed

| Input | Role |
|---|---|
| `docs/test-runs/core-engine/release-grade-ledger/w040-direct-verification-obligation-map-001/direct_verification_obligation_map.json` | W040 obligations `W040-OBL-003`, `W040-OBL-009`, `W040-OBL-010`, `W040-OBL-011`, `W040-OBL-015`, and `W040-OBL-021` |
| `docs/test-runs/core-engine/stage2-replay/w039-stage2-production-policy-replay-governance-001/` | predecessor Stage 2 policy-governance evidence |
| `docs/test-runs/core-engine/implementation-conformance/w040-optimized-core-exact-blocker-fixes-differentials-001/` | W040 dynamic/soft-reference seed evidence and remaining optimized/core blockers |
| `docs/test-runs/core-engine/treecalc-local/w040-optimized-core-dynamic-release-reclassification-001/run_summary.json` | 25-case TreeCalc run with 0 expectation mismatches |
| `docs/test-runs/core-engine/formal-assurance/w040-lean-tla-full-verification-discharge-001/` | W040 Lean/TLA bounded model rows and fairness/scheduler blockers |
| `docs/test-runs/core-engine/tla/w036-stage2-partition-001/` | bounded TLA partition model, fence-reject model, and promotion blockers |
| `docs/test-runs/core-engine/tracecalc-reference-machine/w037-tracecalc-observable-closure-001/scenarios/tc_w034_snapshot_fence_reject_001/` | TraceCalc stale snapshot reject/no-publish evidence |
| `docs/test-runs/core-engine/tracecalc-reference-machine/w037-tracecalc-observable-closure-001/scenarios/tc_w034_capability_fence_reject_001/` | TraceCalc capability-view mismatch reject/no-publish evidence |
| `../OxFml/docs/worksets/W073_conditional_formatting_typed_visualization_payload.md` and local W040 intake notes | W073 typed-only formatting guard retained for Stage 2 observable surfaces |

## 3. Artifact Surface

Run id: `w040-stage2-production-policy-equivalence-001`

| Artifact | Result |
|---|---|
| `formal/lean/OxCalc/CoreEngine/W040Stage2ProductionPolicyAndEquivalence.lean` | checked Lean predicate for W040 Stage 2 policy requirements and current no-promotion state |
| `docs/test-runs/core-engine/stage2-replay/w040-stage2-production-policy-equivalence-001/run_summary.json` | records 12 policy rows, 8 satisfied policy rows, 5 partition replay rows, 6 permutation rows, 5 observable-invariance rows, snapshot and capability counterparts evidenced, bounded analyzer evidenced, 4 exact blockers, and 0 failed rows |
| `docs/test-runs/core-engine/stage2-replay/w040-stage2-production-policy-equivalence-001/w040_stage2_policy_gate_register.json` | W040 policy-gate row ledger |
| `docs/test-runs/core-engine/stage2-replay/w040-stage2-production-policy-equivalence-001/w040_partition_analyzer_soundness_register.json` | dynamic/soft-reference, fence counterpart, bounded analyzer, full analyzer, and fairness/scheduler rows |
| `docs/test-runs/core-engine/stage2-replay/w040-stage2-production-policy-equivalence-001/w040_observable_equivalence_register.json` | bounded replay, permutation, observable invariance, fence no-publish, W073, operated-service, and pack-governance rows |
| `docs/test-runs/core-engine/stage2-replay/w040-stage2-production-policy-equivalence-001/w040_stage2_exact_blocker_register.json` | four exact remaining Stage 2 blockers |
| `docs/test-runs/core-engine/stage2-replay/w040-stage2-production-policy-equivalence-001/promotion_decision.json` | Stage 2 policy remains unpromoted |
| `docs/test-runs/core-engine/stage2-replay/w040-stage2-production-policy-equivalence-001/source_evidence_index.json` | source artifact index binding W039, W040 optimized/core, W040 formal, TLA, and TraceCalc inputs |
| `docs/test-runs/core-engine/stage2-replay/w040-stage2-production-policy-equivalence-001/validation.json` | validation status `w040_stage2_policy_equivalence_valid` |

## 4. Policy Disposition

| Row family | W040 disposition | Evidence consequence |
|---|---|---|
| bounded baseline-versus-Stage-2 replay | carried forward from W039 | five declared profiles retain deterministic bounded replay evidence |
| partition-order permutation replay | carried forward from W039 | six permutation rows remain valid, including one nontrivial independent-order swap |
| observable-result invariance | carried forward from W039 | five declared profiles preserve observable projections |
| dynamic/soft-reference replay | bound to W040 optimized/core and TreeCalc evidence | explicit dependency release/reclassification seed behavior is direct evidence for declared Stage 2 profiles |
| snapshot-fence counterpart | direct evidence | TraceCalc stale snapshot reject/no-publish plus bounded TLA fence-reject evidence discharges the Stage 2 counterpart row for the declared profile |
| capability-view fence counterpart | direct evidence | TraceCalc capability-view mismatch reject/no-publish plus bounded TLA fence-reject evidence discharges the Stage 2 counterpart row for the declared profile |
| bounded partition analyzer predicate | direct bounded evidence | partition ownership, cross-partition blocking, completed-partition readiness, replay-evidence preconditions, and fence rejection are evidenced for bounded declared profiles |
| W073 typed formatting guard | carried as observable watch | typed-only conditional-formatting metadata remains a Stage 2 observable-surface constraint |
| full production partition analyzer soundness | exact blocker | bounded declared-profile evidence is not full production analyzer soundness |
| fairness and unbounded scheduler coverage | exact blocker | full TLA verification and unbounded scheduler coverage remain blocked |
| operated cross-engine Stage 2 differential service | exact blocker | operated service evidence remains under `calc-tv5.6` and `calc-tv5.7` |
| pack-grade replay governance | exact blocker | pack-grade replay, C5, retained witness lifecycle, and release decision remain under `calc-tv5.9` and `calc-tv5.10` |

## 5. Lean Policy Predicate

`formal/lean/OxCalc/CoreEngine/W040Stage2ProductionPolicyAndEquivalence.lean` defines `Stage2ProductionEvidence` and `CanPromoteStage2ProductionPolicy`.

The current W040 evidence has:

1. bounded partition replay,
2. partition-order permutation replay,
3. observable-result invariance,
4. dynamic/soft-reference replay,
5. snapshot-fence counterpart evidence,
6. capability-view fence counterpart evidence,
7. bounded partition analyzer model evidence,
8. no full production partition analyzer soundness,
9. no fairness/unbounded scheduler coverage discharge,
10. no operated cross-engine differential service,
11. no pack-grade replay governance.

The checked predicate proves the current W040 evidence does not promote Stage 2 policy, and that any future promotion requires declared-profile replay, fence counterparts, full production analyzer soundness, and fairness/scheduler coverage.

## 6. Semantic-Equivalence Statement

For the declared W040 Stage 2 profiles, baseline replay, partition replay, partition-order permutations, dynamic/soft-reference release/reclassification evidence, snapshot-fence reject/no-publish evidence, capability-view fence reject/no-publish evidence, and W073 typed-formatting observable guards preserve the observable result surface.

This bead does not change coordinator scheduling, invalidation strategy, dependency graph construction, soft-reference update behavior, recalc behavior, publication semantics, reject policy, overlay lifecycle behavior, TraceCalc semantics, TreeCalc runtime behavior, optimized/core runtime behavior, OxFml evaluator behavior, OxFunc kernels, pack/C5 capability policy, operated service behavior, alert/quarantine policy, retained-history behavior, or the actual Stage 2 scheduler policy.

Stage 2 production scheduler and partition policy remain unpromoted. A future promotion still needs full production partition-analyzer soundness, fairness and unbounded scheduler coverage, operated cross-engine Stage 2 differential service evidence, and pack-grade replay governance for the claimed scope.

## 7. OxFml Formatting Intake

This bead does not construct a new OxFml formatting request payload and does not file an OxFml handoff.

It carries the W073 guard as a Stage 2 observable-surface constraint:

1. `VerificationConditionalFormattingRule.typed_rule` remains the only accepted metadata source for `colorScale`, `dataBar`, `iconSet`, `top`, `bottom`, `aboveAverage`, and `belowAverage`.
2. W072 bounded `thresholds` strings are not a fallback for those families.
3. Broad OxFml seam closure remains under `calc-tv5.8`.

## 8. Verification

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | passed |
| `lean formal/lean/OxCalc/CoreEngine/W040Stage2ProductionPolicyAndEquivalence.lean` | passed |
| `rg -n "^\\s*(axiom|sorry|admit)\\b" formal/lean` | passed; no placeholders found |
| `cargo test -p oxcalc-tracecalc stage2_replay -- --nocapture` | passed; 3 focused tests |
| `cargo test -p oxcalc-tracecalc` | passed; 41 tests |
| `cargo test -p oxcalc-tracecalc-cli` | passed; 0 tests |
| `cargo run -p oxcalc-tracecalc-cli -- stage2-replay w040-stage2-production-policy-equivalence-001` | passed; emitted W040 Stage 2 artifacts |
| W040-local TLC rerun over five `formal/tla/CoreEngineW036Stage2Partition.tla` configs | passed; no TLC errors found |
| JSON parse for `docs/test-runs/core-engine/stage2-replay/w040-stage2-production-policy-equivalence-001/*.json` | passed |
| `scripts/check-worksets.ps1` | passed after bead closure |
| `br ready --json` | passed after bead closure; next ready bead is `calc-tv5.6` |
| `br dep cycles --json` | passed with `count: 0` |
| `git diff --check` | passed; CRLF normalization warnings only |

## 9. Pre-Closure Verification Checklist

| # | Check | Result |
|---|---|---|
| 1 | Spec text and realization notes updated for all in-scope items? | yes; this packet, W040 workset/status surfaces, feature map, formal README, Lean file, runner profile, and machine-readable artifacts record the slice |
| 2 | Pack expectations updated for affected packs? | yes; pack-grade Stage 2 replay governance remains unpromoted and `calc-tv5.9` owns reassessment |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes; W040 emits deterministic Stage 2 policy/equivalence artifacts and binds deterministic TraceCalc, TreeCalc, and TLA evidence for the declared profiles |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes; Section 6 states declared-profile observable invariance and no production Stage 2 policy promotion |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes; W073 typed-only formatting is retained as a guard and no handoff trigger exists |
| 6 | All required tests pass? | yes for the current target; see Section 8 |
| 7 | No known semantic gaps remain in declared scope? | yes for this policy/equivalence target; production Stage 2 promotion blockers remain exact with owner lanes |
| 8 | Completion language audit passed? | yes; no Stage 2 production policy, full production analyzer, operated service, pack-grade replay, C5, broad OxFml, callable metadata, general OxFunc, or release-grade promotion is claimed |
| 9 | `WORKSET_REGISTER.md` updated when ordered workset truth changed? | not applicable; ordered workset truth is unchanged |
| 10 | `IN_PROGRESS_FEATURE_WORKLIST.md` updated when feature-map truth changed? | yes; IP-04 records the W040 Stage 2 state |
| 11 | execution-state blocker surface updated? | yes; `.beads/` records `calc-tv5.5` closure and `calc-tv5.6` readiness |

## 10. Completion Claim Self-Audit

| Step | Audit result |
|---|---|
| Scope re-read | pass; `calc-tv5.5` asks for Stage 2 production policy and equivalence evidence |
| Gate criteria re-read | pass; scheduler/partition strategy changes have semantic-equivalence statements and direct replay/model evidence before any policy promotion |
| Silent scope reduction check | pass; the packet explicitly separates direct declared-profile evidence from exact blockers and no production Stage 2 policy is promoted |
| "Looks done but is not" pattern check | pass; bounded replay, TraceCalc rejects, TLA checks, TreeCalc seed evidence, and checked predicates are not represented as full production analyzer soundness, fairness coverage, operated service evidence, pack-grade replay, C5, or release-grade verification |
| Result | pass for the `calc-tv5.5` target after final bead closure validation |

## 11. Three-Axis Report

- execution_state: `calc-tv5.5_stage2_policy_equivalence_validated_no_promotion`
- scope_completeness: `scope_partial`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- open_lanes:
  - `calc-tv5.6` operated assurance and retained history service evidence is next
  - full production partition analyzer soundness remains an exact Stage 2 blocker
  - fairness and unbounded scheduler coverage remain exact Stage 2/TLA blockers
  - operated cross-engine Stage 2 differential service remains open
  - pack-grade replay governance, retained witness lifecycle, C5, and release-grade decision remain open
  - independent evaluator row set and diversity evidence remain open
  - OxFml seam breadth, W073 typed-only conditional-formatting metadata, public consumer surfaces, and callable metadata closure remain open
