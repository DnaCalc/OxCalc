# W044 Lean/TLA Unbounded Fairness And Full-Verification Proof Expansion

Status: `calc-b1t.4_lean_tla_unbounded_fairness_full_verification_expansion_validated`
Workset: `W044`
Parent epic: `calc-b1t`
Bead: `calc-b1t.4`

## 1. Purpose

This packet deepens the W044 Lean/TLA proof/model tranche after `calc-b1t.3`.

The narrow result is a checked W044 Lean row model plus a formal-assurance packet that binds the W044 residual blocker map, the W043 Lean/TLA predecessor packet, the W044 Rust totality/refinement and panic-surface packet, the W043 Stage 2 scheduler-equivalence packet, the bounded W037 TLA inventory, current W073 typed-only formatting intake, and explicit exact blockers.

It strengthens the proof/model evidence frontier, but it does not promote full Lean verification, full TLA verification, scheduler fairness, unbounded model coverage, Rust-engine totality, Rust refinement, Stage 2 production policy, pack-grade replay, C5, callable carrier sufficiency, release-grade verification, broad OxFml closure, or general OxFunc kernels.

## 2. Authority Inputs Reviewed

| Input | Role |
|---|---|
| `docs/worksets/W044_CORE_FORMALIZATION_RELEASE_GRADE_BLOCKER_BURN_DOWN_AND_SERVICE_PROOF_CLOSURE.md` | W044 scope and `calc-b1t.4` gate |
| `docs/spec/core-engine/w044-formalization/W044_RESIDUAL_RELEASE_GRADE_BLOCKER_RECLASSIFICATION_AND_PROMOTION_CONTRACT_MAP.md` | W044 obligations `W044-OBL-016` through `W044-OBL-018` and no-promotion guard |
| `docs/spec/core-engine/w044-formalization/W044_RUST_TOTALITY_REFINEMENT_AND_PANIC_SURFACE_PROOF_EXPANSION.md` | W044.3 Rust totality/refinement and callable-carrier evidence |
| `archive/test-runs-core-engine-w038-w045/formal-assurance/w043-lean-tla-full-verification-unbounded-fairness-001/` | predecessor Lean/TLA proof/model packet |
| `archive/test-runs-core-engine-w038-w045/formal-assurance/w044-rust-totality-refinement-panic-surface-expansion-001/` | W044 Rust mixed dynamic, no-publication, and exact blocker input |
| `archive/test-runs-core-engine-w038-w045/stage2-replay/w043-stage2-production-partition-analyzer-scheduler-equivalence-001/` | bounded Stage 2 scheduler-equivalence and pack-grade input |
| `docs/test-runs/core-engine/formal-inventory/w037-proof-model-closure-001/` | checked Lean and bounded TLA inventory floor |
| `../OxFml/docs/worksets/W073_conditional_formatting_typed_visualization_payload.md` | current W073 typed-only formatting guard |

## 3. Artifact Surface

Run id: `w044-lean-tla-unbounded-fairness-full-verification-expansion-001`

| Artifact | Result |
|---|---|
| `formal/lean/OxCalc/CoreEngine/W044LeanTlaFullVerificationAndFairness.lean` | checked Lean row model for W044 Lean/TLA proof/model classification |
| `archive/test-runs-core-engine-w038-w045/formal-assurance/w044-lean-tla-unbounded-fairness-full-verification-expansion-001/run_summary.json` | 16 rows, 10 local checked-proof rows, 4 bounded-model rows, 1 accepted external seam, 3 accepted boundaries, 5 totality boundaries, 5 exact blockers, 0 failed rows |
| `w044_lean_tla_discharge_ledger.json` | machine-readable 16-row Lean/TLA discharge ledger |
| `w044_lean_proof_register.json` | 10 local checked-proof classification rows |
| `w044_tla_model_bound_register.json` | 4 bounded-model rows |
| `w044_lean_tla_exact_blocker_register.json` | 5 exact remaining proof/model blockers |
| `source_evidence_index.json` | source evidence index for W044.1, W044.3, W043 Lean/TLA, W043 Stage 2, W037 inventory, W073, and Lean artifacts |
| `validation.json` | validation status `formal_assurance_w044_lean_tla_fairness_valid` |

## 4. Implementation Delta

Changed files:

1. `formal/lean/OxCalc/CoreEngine/W044LeanTlaFullVerificationAndFairness.lean`
2. `src/oxcalc-tracecalc/src/formal_assurance.rs`

The formal-assurance runner now has a W044 Lean/TLA profile that:

1. reads W044 residual blocker and W073 intake artifacts,
2. reads W037 formal inventory and TLA inventory artifacts,
3. reads the W043 Lean/TLA predecessor packet and exact blockers,
4. reads the W044 Rust totality/refinement ledger, refinement rows, and blockers,
5. reads the W043 Stage 2 scheduler-equivalence and pack-governance packet,
6. emits W044 Lean/TLA discharge ledger, Lean proof register, TLA model-bound register, exact blocker register, source evidence index, validation, and run summary artifacts,
7. validates 16 rows, 10 checked-proof rows, 4 bounded-model rows, 5 exact blockers, and 0 failed rows.

## 5. Row Disposition

| Row | Disposition |
|---|---|
| Lean inventory and placeholder audit | checked Lean evidence |
| W043 Lean/TLA predecessor bridge | checked Lean bridge evidence |
| W044 mixed dynamic refinement bridge | checked Lean refinement bridge |
| W044 publication no-publish fence bridge | checked Lean publication-fence bridge |
| W044 callable carrier bridge | checked Lean callable-carrier bridge |
| W043 Stage 2 scheduler/pack predicate | checked Lean policy predicate |
| routine TLC config set | bounded model with exact totality boundary |
| Stage 2 partition bounded configs | bounded Stage 2 model evidence |
| W043 Stage 2 equivalence packet | bounded Stage 2 equivalence input |
| scheduler fairness and unbounded interleaving | exact model-assumption boundary |
| full Lean verification | exact proof blocker |
| full TLA verification | exact model blocker |
| Rust totality/refinement dependency | exact proof/model blocker |
| `LET`/`LAMBDA` carrier seam | accepted external seam boundary |
| spec-evolution guard | accepted boundary |
| W073 typed formatting guard | accepted formatting boundary |

Observed counts:

1. 16 proof/model rows.
2. 10 local checked-proof classification rows.
3. 4 bounded-model rows.
4. 1 accepted external seam.
5. 3 accepted boundaries.
6. 5 totality-boundary rows.
7. 5 exact remaining blockers.
8. 1 mixed dynamic-refinement bridge row.
9. 1 publication-fence bridge row.
10. 0 failed rows.

## 6. Fairness And Model-Bound Position

The W044.4 packet keeps the TLA floor explicit:

1. The routine TLC inventory still has 11 passed configs and 0 failed configs.
2. The W043 Stage 2 packet has bounded partition replay, permutation replay, observable-invariance, bounded analyzer evidence, declared scheduler-equivalence input, and 6 exact Stage 2 blockers.
3. Those rows are bounded or declared-profile evidence only.
4. Full TLA verification remains blocked by unbounded model coverage, scheduler fairness, production partition-analyzer soundness, and downstream operated/pack evidence.
5. Stage 2 production partition-analyzer and scheduler equivalence remain owned by `calc-b1t.5`.

This is a proof/model strengthening packet, not a promotion packet.

## 7. Rust And Callable Bridges

W044.4 consumes the W044.3 Rust rows:

1. `w044_mixed_dynamic_add_release_refinement_evidence`,
2. `w044_publication_fence_no_publish_refinement_evidence`,
3. `w044_callable_value_carrier_totality_evidence`.

Those rows are checked proof inputs for the formal model. They do not discharge retained Rust or callable blockers:

1. runtime panic-surface totality boundary,
2. broader dynamic transition coverage,
3. snapshot-fence counterpart breadth,
4. capability-view counterpart breadth,
5. callable metadata projection,
6. full optimized/core release-grade conformance,
7. callable carrier sufficiency beyond the current value-carrier row,
8. general OxFunc kernels.

## 8. OxFml W073 Formatting Intake

The latest OxFml formatting update was reviewed against this packet.

The W073 contract remains typed-only for aggregate and visualization conditional-formatting metadata:

1. `VerificationConditionalFormattingRule.typed_rule` is required for `colorScale`, `dataBar`, `iconSet`, `top`, `bottom`, `aboveAverage`, and `belowAverage`.
2. W072 bounded `thresholds` strings are intentionally ignored for those families.
3. `thresholds` remains meaningful only for scalar/operator/expression rule families where threshold text is the real rule input.

W044.4 does not construct conditional-formatting requests and does not change OxFml evaluator behavior. The source index therefore carries the W044 W073 intake as a seam-watch source; broader request construction and public migration remain owned by `calc-b1t.8`.

No OxFml handoff is required by this bead.

## 9. Semantic-Equivalence Statement

This packet adds a checked Lean classification file, a W044 formal-assurance runner profile, emitted formal-assurance artifacts, and documentation.

It does not change coordinator scheduling, invalidation strategy, dependency graph construction, soft-reference update behavior, recalc behavior, publication semantics, reject policy, overlay lifecycle behavior, TraceCalc semantics, TreeCalc/CoreEngine runtime behavior, OxFml runtime behavior, OxFunc callable kernels, Stage 2 scheduler policy, pack/C5 capability policy, operated-service behavior, alert dispatch behavior, retained-history behavior, or retained-witness behavior.

Observable engine results are invariant under this packet. The only changed observable artifacts are W044.4 formal-assurance evidence files and the checked Lean classification file.

## 10. Verification

| Command | Result |
|---|---|
| `lean formal/lean/OxCalc/CoreEngine/W044LeanTlaFullVerificationAndFairness.lean` | passed |
| `lean formal/lean/OxCalc/CoreEngine/W044RustTotalityAndRefinement.lean` | passed |
| `lean formal/lean/OxCalc/CoreEngine/W043Stage2ProductionPartitionAnalyzerAndSchedulerEquivalence.lean` | passed |
| `cargo fmt --all -- --check` | passed |
| `cargo test -p oxcalc-tracecalc formal_assurance_runner_classifies_w044_lean_tla_fairness_expansion -- --nocapture` | passed |
| `cargo test -p oxcalc-tracecalc formal_assurance -- --nocapture` | passed; 12 tests |
| `cargo run -p oxcalc-tracecalc-cli -- formal-assurance w044-lean-tla-unbounded-fairness-full-verification-expansion-001` | passed; emitted 16 rows with 0 failed rows |
| JSON parse for W044.4 formal-assurance artifacts | passed |
| `scripts/check-worksets.ps1` | passed; ready queue has `calc-b1t.5` |
| `br ready --json` | passed; `calc-b1t.5` is ready |
| `br dep cycles --json` | passed; 0 cycles |
| `git diff --check` | passed; CRLF normalization warnings only |

## 11. Pre-Closure Verification Checklist

| # | Check | Result |
|---|---|---|
| 1 | Spec text and realization notes updated for all in-scope items? | yes; this packet, W044 README/status surfaces, feature map, Lean file, runner profile, and machine-readable artifacts record the slice |
| 2 | Pack expectations updated for affected packs? | yes; pack/C5 remains unpromoted and `calc-b1t.10` owns reassessment |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes; W044.3, W043 Lean/TLA, W043 Stage 2, W037 TLA inventory, and W044.4 formal-assurance artifacts are bound |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes; no runtime strategy changed |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes; current W073 typed-only formatting intake is carried and no handoff trigger exists |
| 6 | All required tests pass? | yes for this target; see Section 10 |
| 7 | No known semantic gaps remain in declared scope? | yes for this W044.4 classification target; broader exact blockers are explicit |
| 8 | Completion language audit passed? | yes; no full Lean/TLA, scheduler fairness, unbounded model coverage, Rust totality/refinement, optimized/core, Stage 2, pack/C5, service, independent-diversity, broad OxFml, callable metadata, callable carrier, registered-external, provider-publication, or general OxFunc promotion is claimed |
| 9 | `WORKSET_REGISTER.md` updated when ordered workset truth changed? | not applicable; ordered workset truth is unchanged |
| 10 | `IN_PROGRESS_FEATURE_WORKLIST.md` updated when feature-map truth changed? | yes |
| 11 | execution-state blocker surface updated? | yes; `.beads/` records `calc-b1t.4` closure and queues `calc-b1t.5` readiness |

## 12. Completion Claim Self-Audit

| Step | Audit result |
|---|---|
| Scope re-read | pass; `calc-b1t.4` asks for Lean/TLA verification deepening with fairness and unbounded-model boundaries explicit |
| Gate criteria re-read | pass; discharged proof rows, bounded model rows, exact blockers, accepted boundaries, and no-promotion claims are separated |
| Silent scope reduction check | pass; full Lean/TLA, fairness, unbounded model coverage, Rust dependency, Stage 2 policy, pack/C5, service, OxFml breadth, callable metadata, callable carrier sufficiency, and general OxFunc lanes remain explicit open lanes |
| "Looks done but is not" pattern check | pass; checked classification and bounded model evidence are not reported as full verification |
| Result | pass for the `calc-b1t.4` target |

## 13. Three-Axis Report

- execution_state: `calc-b1t.4_lean_tla_unbounded_fairness_full_verification_expansion_validated`
- scope_completeness: `scope_partial`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- open_lanes:
  - `calc-b1t.5` Stage 2 production partition analyzer and scheduler equivalence implementation is next
  - full Lean verification remains blocked
  - full TLA verification remains blocked
  - scheduler fairness and unbounded model coverage remain exact blockers
  - runtime panic-surface proof, broader dynamic transition coverage, snapshot/capability counterpart breadth, callable metadata projection, callable carrier sufficiency, full optimized/core verification, Rust totality/refinement, release-grade verification, C5, pack-grade replay, Stage 2 policy, operated services, independent evaluator breadth, mismatch quarantine, broad OxFml display/publication, public migration, W073 downstream typed-rule uptake, registered-external callable projection, provider-failure/callable-publication semantics, and general OxFunc kernels remain unpromoted
