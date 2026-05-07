# W039 Stage 2 Production Partition Policy And Replay Governance

Status: `calc-f7o.4_stage2_policy_replay_governance_validated`
Workset: `W039`
Parent epic: `calc-f7o`
Bead: `calc-f7o.4`

## 1. Purpose

This packet attacks the W039 Stage 2 production-policy target.

The result is deliberately non-promoting. W039 adds a Stage 2 policy-governance runner profile and a checked Lean policy predicate that bind W038 bounded replay evidence, W039 optimized/core exact blockers, and W039 proof/model exact blockers into one promotion decision.

The target is not to claim Stage 2 production policy, pack-grade replay, C5, operated Stage 2 differential service, or release-grade verification. The target is to carry forward bounded replay evidence, make production-policy promotion predicates explicit, and retain exact blockers for the evidence still absent.

## 2. Authority Inputs Reviewed

| Input | Role |
|---|---|
| `docs/spec/core-engine/w039-formalization/W039_RESIDUAL_SUCCESSOR_OBLIGATION_LEDGER_AND_PROMOTION_READINESS_MAP.md` | W039 Stage 2 obligations `W039-OBL-003`, `W039-OBL-007`, `W039-OBL-009`, `W039-OBL-010`, `W039-OBL-014`, `W039-OBL-017`, and `W039-OBL-020` |
| `archive/test-runs-core-engine-w038-w045/release-grade-ledger/w039-residual-successor-obligation-ledger-001/successor_obligation_ledger.json` | machine-readable W039 owner lanes and promotion consequences |
| `docs/spec/core-engine/w038-formalization/W038_STAGE2_PARTITION_REPLAY_AND_SEMANTIC_EQUIVALENCE_EXECUTION.md` | predecessor bounded Stage 2 replay packet |
| `archive/test-runs-core-engine-w038-w045/stage2-replay/w038-stage2-partition-replay-001/` | W038 partition replay, permutation replay, semantic-equivalence, blocker, promotion, and validation artifacts |
| `archive/test-runs-core-engine-w038-w045/implementation-conformance/w039-optimized-core-exact-blocker-disposition-001/` | W039 snapshot-fence, capability-view fence, dynamic, and callable exact blockers |
| `archive/test-runs-core-engine-w038-w045/formal-assurance/w039-proof-model-totality-closure-001/` | W039 proof/model exact blocker and Stage 2 proof-gate inputs |
| `../OxFml/docs/worksets/W073_conditional_formatting_typed_visualization_payload.md` and reviewed local W073 diffs | inbound W073 typed-only formatting seam context retained for Stage 2 observable surfaces |

## 3. Artifact Surface

Run id: `w039-stage2-production-policy-replay-governance-001`

| Artifact | Result |
|---|---|
| `formal/lean/OxCalc/CoreEngine/W039Stage2ProductionPolicy.lean` | checked Lean predicate for W039 Stage 2 production-policy promotion requirements |
| `archive/test-runs-core-engine-w038-w045/stage2-replay/w039-stage2-production-policy-replay-governance-001/run_summary.json` | records 10 policy rows, 5 satisfied policy rows, 5 partition replay rows, 6 permutation rows, 5 observable-invariance rows, 1 formatting watch row, 5 exact blockers, and 0 failed rows |
| `archive/test-runs-core-engine-w038-w045/stage2-replay/w039-stage2-production-policy-replay-governance-001/w039_stage2_policy_gate_register.json` | policy-gate row ledger |
| `archive/test-runs-core-engine-w038-w045/stage2-replay/w039-stage2-production-policy-replay-governance-001/w039_partition_soundness_register.json` | dynamic/soft-reference, snapshot-fence, capability-view fence, and production-analyzer soundness rows |
| `archive/test-runs-core-engine-w038-w045/stage2-replay/w039-stage2-production-policy-replay-governance-001/w039_replay_governance_register.json` | bounded replay, permutation, observable-invariance, operated-service, and pack-governance rows |
| `archive/test-runs-core-engine-w038-w045/stage2-replay/w039-stage2-production-policy-replay-governance-001/w039_stage2_exact_blocker_register.json` | five exact remaining Stage 2 blockers |
| `archive/test-runs-core-engine-w038-w045/stage2-replay/w039-stage2-production-policy-replay-governance-001/promotion_decision.json` | Stage 2 policy remains unpromoted |
| `archive/test-runs-core-engine-w038-w045/stage2-replay/w039-stage2-production-policy-replay-governance-001/source_evidence_index.json` | source artifact index binding W038 and W039 inputs |
| `archive/test-runs-core-engine-w038-w045/stage2-replay/w039-stage2-production-policy-replay-governance-001/validation.json` | validation status `w039_stage2_policy_governance_valid` |

## 4. Policy Disposition

| Row family | W039 disposition | Evidence consequence |
|---|---|---|
| bounded baseline-versus-Stage-2 replay | carried forward from W038 | five declared profiles retain deterministic bounded replay evidence |
| partition-order permutation replay | carried forward from W038 | six permutation rows remain valid, including one nontrivial independent-order swap |
| observable-result invariance | carried forward from W038 | five declared profiles preserve observable projections |
| dynamic/soft-reference replay | carried forward from W038 plus W039 conformance context | bounded dynamic reference evidence exists; dynamic release/reclassification remains an optimized/core blocker outside this Stage 2 policy target |
| W073 typed formatting guard | carried forward as observable watch | typed-only conditional-formatting metadata remains a Stage 2 observable-surface constraint |
| snapshot-fence counterpart | exact blocker | stale accepted-candidate counterpart remains absent |
| capability-view fence counterpart | exact blocker | compatibility-fenced capability-view mismatch counterpart remains absent |
| production partition analyzer soundness | exact blocker | no production analyzer proof or corpus-wide soundness corpus is claimed |
| operated cross-engine Stage 2 differential service | exact blocker | operated service evidence remains under `calc-f7o.5` and `calc-f7o.6` |
| pack-grade replay governance | exact blocker | pack-grade replay, C5, retained witness lifecycle, and release decision remain under `calc-f7o.8` and `calc-f7o.9` |

## 5. Lean Policy Predicate

`formal/lean/OxCalc/CoreEngine/W039Stage2ProductionPolicy.lean` defines `Stage2ProductionPolicyEvidence` and `CanPromoteStage2ProductionPolicy`.

The current W039 evidence has:

1. bounded partition replay,
2. partition-order permutation replay,
3. observable-result invariance,
4. dynamic/soft-reference replay,
5. no snapshot-fence counterpart,
6. no capability-view fence counterpart,
7. no production partition analyzer soundness proof,
8. no operated cross-engine differential service,
9. no pack-grade replay governance.

The checked predicate proves the current W039 evidence does not promote Stage 2 policy, and that any future promotion requires both declared-profile replay and fence counterparts.

## 6. Semantic-Equivalence Statement

For the bounded profiles carried into `w039-stage2-production-policy-replay-governance-001`, observable results remain invariant between the baseline schedule, the declared Stage 2 partition schedule, and every admissible partition-order permutation emitted by the predecessor W038 runner.

This bead does not change coordinator scheduling, invalidation strategy, dependency graph construction, soft-reference update behavior, recalc behavior, publication semantics, reject policy, overlay lifecycle behavior, TraceCalc semantics, TreeCalc runtime behavior, optimized/core runtime behavior, OxFml evaluator behavior, OxFunc kernels, pack/C5 capability policy, operated service behavior, alert/quarantine policy, or retained-history behavior.

Stage 2 production scheduler and partition policy remain unpromoted. A future promotion needs production partition-analyzer soundness, direct snapshot/capability fence counterpart evidence, operated cross-engine Stage 2 differential service evidence, and pack-grade replay governance for the claimed scope.

## 7. OxFml Formatting Intake

This bead does not construct a new OxFml formatting request payload and does not file an OxFml handoff.

It carries the W073 guard as a Stage 2 observable-surface constraint:

1. `VerificationConditionalFormattingRule.typed_rule` remains the only accepted metadata source for `colorScale`, `dataBar`, `iconSet`, `top`, `bottom`, `aboveAverage`, and `belowAverage`.
2. W072 bounded `thresholds` strings are not a fallback for those families.
3. Broad OxFml seam closure remains under `calc-f7o.7`.

## 8. Verification

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | passed |
| `cargo test -p oxcalc-tracecalc stage2_replay -- --nocapture` | passed; 2 tests |
| `cargo test -p oxcalc-tracecalc` | passed; 34 tests |
| `cargo test -p oxcalc-tracecalc-cli` | passed; 0 tests |
| `cargo run -p oxcalc-tracecalc-cli -- stage2-replay w039-stage2-production-policy-replay-governance-001` | passed; emitted W039 Stage 2 policy-governance artifacts |
| `lean formal/lean/OxCalc/CoreEngine/W039Stage2ProductionPolicy.lean` | passed |
| `rg -n "^\\s*(axiom|sorry|admit)\\b" formal/lean` | passed; no placeholders found |
| W036 Stage 2 bounded partition TLC profiles | passed for `bounded_ready`, `fence_reject`, `multi_reader`, `partition_cross_dep`, and `scheduler_blocked` |
| JSON parse for `archive/test-runs-core-engine-w038-w045/stage2-replay/w039-stage2-production-policy-replay-governance-001/*.json` | passed |
| `scripts/check-worksets.ps1` | passed after bead closure |
| `br ready --json` | passed after bead closure; next ready bead is `calc-f7o.5` |
| `br dep cycles --json` | passed with `count: 0` |
| `git diff --check` | passed; CRLF normalization warnings only |

## 9. Pre-Closure Verification Checklist

| # | Check | Result |
|---|---|---|
| 1 | Spec text and realization notes updated for all in-scope items? | yes; this packet, W039 workset/status surfaces, feature map, Lean file, runner profile, and machine-readable artifacts record the slice |
| 2 | Pack expectations updated for affected packs? | yes; pack-grade Stage 2 replay governance remains unpromoted and `calc-f7o.8` owns reassessment |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes; W038 deterministic replay artifacts are carried forward and W039 emits a deterministic policy-governance packet |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes; Section 6 states bounded observable invariance and no production Stage 2 policy promotion |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes; W073 typed-only formatting is retained as a guard and no handoff trigger exists |
| 6 | All required tests pass? | yes; see Section 8 |
| 7 | No known semantic gaps remain in declared scope? | yes for this policy-governance target; production Stage 2 promotion blockers remain exact with owner lanes |
| 8 | Completion language audit passed? | yes; no Stage 2 production policy, pack-grade replay, C5, operated service, broad OxFml, or release-grade promotion is claimed |
| 9 | `WORKSET_REGISTER.md` updated when ordered workset truth changed? | not applicable; ordered workset truth is unchanged |
| 10 | `IN_PROGRESS_FEATURE_WORKLIST.md` updated when feature-map truth changed? | yes; IP-04 records the W039 Stage 2 state |
| 11 | execution-state blocker surface updated? | yes; `.beads/` records `calc-f7o.4` closure and `calc-f7o.5` readiness |

## 10. Completion Claim Self-Audit

| Step | Audit result |
|---|---|
| Scope re-read | pass; `calc-f7o.4` asks for Stage 2 production partition policy and replay governance |
| Gate criteria re-read | pass; observable invariance, bounded partition replay, dynamic/soft references, fence counterparts, service dependencies, and pack governance are classified before any policy promotion |
| Silent scope reduction check | pass; the packet explicitly separates carried bounded evidence from exact blockers and no production Stage 2 policy is promoted |
| "Looks done but is not" pattern check | pass; bounded replay and checked predicates are not represented as production partition analyzer soundness, operated service evidence, pack-grade replay, C5, or release-grade verification |
| Result | pass for the `calc-f7o.4` target |

## 11. Three-Axis Report

- execution_state: `calc-f7o.4_stage2_policy_replay_governance_validated`
- scope_completeness: `scope_partial`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- open_lanes:
  - `calc-f7o.5` operated assurance service and retained history substrate is next
  - snapshot-fence and capability-view counterparts remain exact Stage 2/coordinator replay blockers
  - production partition analyzer soundness remains exact Stage 2 blocker
  - operated cross-engine Stage 2 differential service remains open
  - pack-grade replay governance, retained witness lifecycle, C5, and release-grade decision remain open
  - independent evaluator row set and diversity evidence remain open
  - OxFml seam breadth, W073 typed-only conditional-formatting metadata, public consumer surfaces, and callable metadata closure remain open
