# W044 Stage 2 Production Partition Analyzer And Scheduler Equivalence Implementation

Status: `calc-b1t.5_stage2_scheduler_equivalence_validated`
Workset: `W044`
Parent epic: `calc-b1t`
Bead: `calc-b1t.5`

## 1. Purpose

This packet deepens the W044 Stage 2 lane after the W044 Lean/TLA proof/model packet.

The narrow result is a checked W044 Stage 2 Lean predicate plus a Stage 2 replay packet that binds the W044 residual blocker map, W043 Stage 2 predecessor evidence, W044 optimized/core mixed dynamic-transition evidence, W044 Rust mixed dynamic and no-publication refinement rows, W044 Lean/TLA bounded model rows, current W073 typed-only formatting intake, declared-profile scheduler and pack equivalence, and explicit exact blockers.

It strengthens the production partition-analyzer and scheduler-equivalence evidence frontier, but it does not promote Stage 2 production policy, pack-grade replay, C5, release-grade verification, production partition-analyzer soundness, scheduler fairness, unbounded model coverage, operated cross-engine Stage 2 differential service, retained-witness lifecycle, pack-grade replay governance, broad OxFml closure, or general OxFunc kernels.

## 2. Authority Inputs Reviewed

| Input | Role |
|---|---|
| `docs/worksets/W044_CORE_FORMALIZATION_RELEASE_GRADE_BLOCKER_BURN_DOWN_AND_SERVICE_PROOF_CLOSURE.md` | W044 scope and `calc-b1t.5` gate |
| `docs/spec/core-engine/w044-formalization/W044_RESIDUAL_RELEASE_GRADE_BLOCKER_RECLASSIFICATION_AND_PROMOTION_CONTRACT_MAP.md` | W044 residual obligations, promotion contracts, no-proxy guards, and W073 intake |
| `docs/spec/core-engine/w044-formalization/W044_OPTIMIZED_CORE_DYNAMIC_TRANSITION_AND_CALLABLE_METADATA_IMPLEMENTATION.md` | mixed dynamic add/remove/reclassify evidence, no-publication behavior, and retained optimized/core blockers |
| `docs/spec/core-engine/w044-formalization/W044_RUST_TOTALITY_REFINEMENT_AND_PANIC_SURFACE_PROOF_EXPANSION.md` | W044 Rust refinement bridge and no-publication proof input |
| `docs/spec/core-engine/w044-formalization/W044_LEAN_TLA_UNBOUNDED_FAIRNESS_AND_FULL_VERIFICATION_PROOF_EXPANSION.md` | W044 Lean/TLA bounded model input and retained fairness/unbounded blockers |
| `docs/test-runs/core-engine/stage2-replay/w043-stage2-production-partition-analyzer-scheduler-equivalence-001/` | predecessor Stage 2 declared-profile scheduler, pack, partition, permutation, and observable-invariance packet |
| `docs/test-runs/core-engine/implementation-conformance/w044-optimized-core-dynamic-transition-callable-metadata-001/` | W044 optimized/core dynamic-transition and exact blocker packet |
| `docs/test-runs/core-engine/formal-assurance/w044-rust-totality-refinement-panic-surface-expansion-001/` | W044 Rust refinement and exact blocker packet |
| `docs/test-runs/core-engine/formal-assurance/w044-lean-tla-unbounded-fairness-full-verification-expansion-001/` | W044 Lean/TLA proof/model and exact blocker packet |
| `../OxFml/docs/worksets/W073_conditional_formatting_typed_visualization_payload.md` | current W073 typed-only formatting guard |

## 3. Artifact Surface

Run id: `w044-stage2-production-partition-analyzer-scheduler-equivalence-001`

| Artifact | Result |
|---|---|
| `formal/lean/OxCalc/CoreEngine/W044Stage2ProductionPartitionAnalyzerAndSchedulerEquivalence.lean` | checked Lean predicate for W044 Stage 2 production partition-analyzer, scheduler-equivalence, pack-equivalence, and no-promotion guards |
| `docs/test-runs/core-engine/stage2-replay/w044-stage2-production-partition-analyzer-scheduler-equivalence-001/run_summary.json` | 25 policy rows, 17 satisfied rows, 5 partition replay rows, 6 permutation rows, 5 observable-invariance rows, 1 formatting watch row, 8 exact blockers, 0 failed rows |
| `w044_stage2_policy_gate_register.json` | machine-readable 25-row W044 Stage 2 policy gate ledger |
| `w044_production_partition_analyzer_register.json` | 18 production analyzer rows, 14 satisfied rows, 4 exact production blockers |
| `w044_scheduler_equivalence_register.json` | 9 scheduler rows, 7 satisfied rows, 2 exact scheduler blockers |
| `w044_pack_grade_equivalence_register.json` | 9 pack-equivalence rows, 7 satisfied rows, 2 exact pack blockers |
| `w044_stage2_exact_blocker_register.json` | 8 exact remaining Stage 2 and pack blockers |
| `promotion_decision.json` | no Stage 2 policy promotion and no pack-grade replay promotion |
| `source_evidence_index.json` | source evidence index for W044 residual, W044 optimized/core, W044 Rust, W044 Lean/TLA, W043 Stage 2, W073, and Lean artifacts |
| `validation.json` | validation status `w044_stage2_scheduler_equivalence_valid` and zero validation failures |

## 4. Implementation Delta

Changed files:

1. `formal/lean/OxCalc/CoreEngine/W044Stage2ProductionPartitionAnalyzerAndSchedulerEquivalence.lean`
2. `src/oxcalc-tracecalc/src/stage2_replay.rs`

The Stage 2 replay runner now has a W044 profile that:

1. reads the W044 residual blocker map and current W073 intake,
2. reads the W044 optimized/core mixed dynamic-transition and exact blocker packet,
3. reads W044 Rust refinement and no-publication rows,
4. reads W044 Lean/TLA bounded model and exact blocker rows,
5. reads W043 Stage 2 predecessor policy, analyzer, scheduler, pack, blocker, promotion, and validation artifacts,
6. emits W044 source index, policy gate, production analyzer, scheduler, pack, blocker, promotion, validation, and summary artifacts,
7. validates 25 policy rows, 17 satisfied rows, 5 partition replay rows, 6 permutation rows, 5 observable-invariance rows, 1 formatting watch row, 8 exact blockers, and 0 failed rows.

## 5. Row Disposition

Observed counts:

1. 25 policy rows.
2. 17 satisfied policy rows.
3. 5 bounded partition replay rows carried.
4. 6 partition-order permutation rows carried.
5. 1 nontrivial partition-order permutation row carried.
6. 5 observable-invariance rows carried.
7. 1 W073 typed-formatting watch row carried.
8. 12 production-relevant analyzer input rows.
9. 7 scheduler-equivalence satisfied rows.
10. 7 pack-equivalence satisfied rows.
11. 8 exact remaining blockers.
12. 0 failed rows.

Satisfied declared-profile and direct-input rows:

1. W043 Stage 2 predecessor policy packet.
2. bounded baseline-versus-Stage-2 replay.
3. bounded partition-order permutation replay.
4. observable-result invariance for declared profiles.
5. W043 dynamic addition/release regression input.
6. W044 mixed dynamic add/remove/reclassify transition input.
7. W044 mixed dynamic rebind reject no-publication input.
8. declared-profile snapshot-fence counterpart input.
9. declared-profile capability-view counterpart input.
10. W044 Rust mixed dynamic and no-publication refinement bridge.
11. W044 Lean/TLA bounded model bridge.
12. W073 typed-only formatting guard.
13. no-proxy promotion guard.
14. production-relevant analyzer input bundle.
15. declared-profile scheduler equivalence.
16. semantic-equivalence statement.
17. declared-profile pack equivalence.

Exact blockers:

1. broader dynamic transition coverage,
2. snapshot-fence breadth,
3. capability-view breadth,
4. full production partition-analyzer soundness,
5. fairness scheduler unbounded coverage,
6. operated cross-engine Stage 2 differential service,
7. retained-witness lifecycle pack dependency,
8. pack-grade replay governance.

## 6. Production Analyzer, Scheduler, And Pack Position

W044.5 narrows the Stage 2 evidence gap by binding production-relevant analyzer inputs from W044 optimized/core, W044 Rust, W044 Lean/TLA, and W043 Stage 2 predecessor artifacts.

The production analyzer register has 18 rows, 14 satisfied rows, and 4 exact production blockers. The scheduler register has 9 rows, 7 satisfied rows, and 2 exact scheduler blockers. The pack-equivalence register has 9 rows, 7 satisfied rows, and 2 exact pack blockers.

Those rows are declared-profile and bounded evidence. Stage 2 production policy remains unpromoted because full production partition-analyzer soundness, broader dynamic transition coverage, snapshot-fence breadth, capability-view breadth, scheduler fairness, unbounded model coverage, operated Stage 2 differential service, retained-witness lifecycle evidence, and pack governance are still required.

Pack-grade replay remains unpromoted because declared-profile pack equivalence is not pack-grade replay governance and does not provide retained-witness lifecycle evidence.

## 7. OxFml W073 Formatting Intake

The latest OxFml formatting update was reviewed against this packet.

The W073 contract remains typed-only for aggregate and visualization conditional-formatting metadata:

1. `VerificationConditionalFormattingRule.typed_rule` is required for `colorScale`, `dataBar`, `iconSet`, `top`, `bottom`, `aboveAverage`, and `belowAverage`.
2. W072 bounded `thresholds` strings are intentionally ignored for those families.
3. `thresholds` remains meaningful only for scalar/operator/expression rule families where threshold text is the real rule input.

W044.5 does not construct conditional-formatting requests and does not change OxFml evaluator behavior. The Stage 2 packet carries W073 as a formatting watch row and no-proxy guard input only. Broader downstream typed-rule request construction and public migration remain owned by `calc-b1t.8`.

No OxFml handoff is required by this bead.

## 8. Semantic-Equivalence Statement

This packet adds a checked Lean classification file, a W044 Stage 2 replay runner profile, emitted Stage 2 replay artifacts, and documentation.

It does not change coordinator scheduling, dependency graph construction, invalidation strategy, soft-reference update behavior, recalc behavior, publication semantics, reject policy, overlay lifecycle behavior, TraceCalc semantics, TreeCalc/CoreEngine runtime behavior, OxFml runtime behavior, OxFunc callable kernels, pack/C5 capability policy, operated-service behavior, alert dispatch behavior, retained-history behavior, or retained-witness behavior.

Observable engine results are invariant under this packet. The packet records that observable-result, scheduler, and pack equivalence are evidenced for declared W044 Stage 2 profiles only; production scheduling, Stage 2 policy, and pack-grade replay remain unpromoted until the retained blockers are discharged.

## 9. Verification

| Command | Result |
|---|---|
| `lean formal/lean/OxCalc/CoreEngine/W044Stage2ProductionPartitionAnalyzerAndSchedulerEquivalence.lean` | passed |
| `lean formal/lean/OxCalc/CoreEngine/W043Stage2ProductionPartitionAnalyzerAndSchedulerEquivalence.lean` | passed |
| `lean formal/lean/OxCalc/CoreEngine/W044LeanTlaFullVerificationAndFairness.lean` | passed |
| `cargo fmt --all -- --check` | passed |
| `cargo test -p oxcalc-tracecalc stage2_replay_runner_writes_w044_scheduler_equivalence_without_promotion -- --nocapture` | passed |
| `cargo test -p oxcalc-tracecalc stage2_replay -- --nocapture` | passed; 7 tests |
| `cargo test -p oxcalc-tracecalc` | passed; 70 tests and doc-tests |
| `cargo run -p oxcalc-tracecalc-cli -- stage2-replay w044-stage2-production-partition-analyzer-scheduler-equivalence-001` | passed; emitted 25 policy rows, 17 satisfied rows, 8 exact blockers, and 0 failed rows |
| JSON parse for W044.5 Stage 2 artifacts | passed |
| `scripts/check-worksets.ps1` | passed; worksets=22, ready queue has `calc-b1t.6` |
| `br ready --json` | passed; next ready bead is `calc-b1t.6` |
| `br dep cycles --json` | passed with `count: 0` |
| `git diff --check` | passed; CRLF normalization warnings only |

## 10. Pre-Closure Verification Checklist

| # | Check | Result |
|---|---|---|
| 1 | Spec text and realization notes updated for all in-scope items? | yes; this packet, W044 README/status surfaces, feature map, Lean predicate, runner profile, and Stage 2 artifacts record the slice |
| 2 | Pack expectations updated for affected packs? | yes; pack-grade replay and C5 remain unpromoted and `calc-b1t.10` owns reassessment |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes; W043 Stage 2, W044 optimized/core, W044 Rust, W044 Lean/TLA, and W044.5 Stage 2 artifacts are bound |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes; no runtime strategy changed and declared-profile observable-result, scheduler, and pack equivalence remain bounded |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes; W073 is carried as typed-only guard and no OxCalc handoff trigger exists |
| 6 | All required tests pass? | yes for this target; see Section 9 |
| 7 | No known semantic gaps remain in declared scope? | yes for this W044.5 classification target; broader exact blockers are explicit |
| 8 | Completion language audit passed? | yes; no Stage 2, pack-grade replay, C5, release-grade, production analyzer, fairness, service, retained-witness, OxFml breadth, callable, or general OxFunc promotion is claimed |
| 9 | `WORKSET_REGISTER.md` updated when ordered workset truth changed? | not applicable; ordered workset truth is unchanged |
| 10 | `IN_PROGRESS_FEATURE_WORKLIST.md` updated when feature-map truth changed? | yes |
| 11 | execution-state blocker surface updated? | yes; `.beads/` records `calc-b1t.5` closure and queues `calc-b1t.6` readiness |

## 11. Completion Claim Self-Audit

| Step | Audit result |
|---|---|
| Scope re-read | pass; `calc-b1t.5` asks for Stage 2 production partition-analyzer and scheduler-equivalence evidence without promotion from bounded or declared-profile evidence alone |
| Gate criteria re-read | pass; declared-profile rows, exact blockers, production analyzer rows, scheduler rows, pack rows, and promotion guards are separated |
| Silent scope reduction check | pass; broader dynamic coverage, snapshot-fence breadth, capability-view breadth, full production analyzer soundness, scheduler fairness, unbounded coverage, operated Stage 2 differential service, retained-witness lifecycle, pack governance, Stage 2 policy, C5, and release-grade verification remain explicit open lanes |
| "Looks done but is not" pattern check | pass; bounded replay, declared-profile scheduler equivalence, declared-profile pack equivalence, and production-relevant analyzer inputs are not reported as production Stage 2 policy or pack-grade replay |
| Result | pass for the `calc-b1t.5` target |

## 12. Three-Axis Report

- execution_state: `calc-b1t.5_stage2_scheduler_equivalence_validated`
- scope_completeness: `scope_partial`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- open_lanes:
  - `calc-b1t.6` operated continuous assurance, retained-history, witness, SLO, and alert service is next
  - broader dynamic transition coverage remains blocked
  - snapshot-fence breadth remains blocked
  - capability-view breadth remains blocked
  - full production partition-analyzer soundness remains blocked
  - scheduler fairness and unbounded model coverage remain exact blockers
  - operated cross-engine Stage 2 differential service remains blocked
  - retained-witness lifecycle and retention SLO remain blocked
  - pack-grade replay governance remains blocked
  - Stage 2 production policy, pack-grade replay, C5, release-grade verification, operated services, independent evaluator breadth, mismatch quarantine, broad OxFml display/publication, public migration, W073 downstream typed-rule uptake, callable metadata projection, callable carrier sufficiency, registered-external callable projection, provider-failure/callable-publication semantics, and general OxFunc kernels remain unpromoted
