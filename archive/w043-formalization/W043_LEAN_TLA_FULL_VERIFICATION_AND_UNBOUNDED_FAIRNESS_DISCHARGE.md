# W043 Lean/TLA Full Verification And Unbounded Fairness Discharge

Status: `calc-2p3.4_lean_tla_full_verification_unbounded_fairness_validated`
Workset: `W043`
Parent epic: `calc-2p3`
Bead: `calc-2p3.4`

## 1. Purpose

This packet deepens the W043 Lean/TLA proof/model tranche after `calc-2p3.3`.

The narrow result is a checked W043 Lean row model plus a formal-assurance packet that binds the W043 proof-service obligation map, the W042 Lean/TLA predecessor packet, the W043 Rust totality/refinement frontier, the W042 Stage 2 analyzer/pack-grade packet, the bounded W037 TLA inventory, current W073 typed-only formatting intake, and explicit exact blockers.

It strengthens the proof/model evidence frontier, but it does not promote full Lean verification, full TLA verification, scheduler fairness, unbounded model coverage, Rust-engine totality, Rust refinement, Stage 2 production policy, pack-grade replay, C5, callable carrier sufficiency, release-grade verification, or general OxFunc kernels.

## 2. Authority Inputs Reviewed

| Input | Role |
|---|---|
| `docs/worksets/W043_CORE_FORMALIZATION_RELEASE_GRADE_PROOF_AND_OPERATED_SERVICE_INTEGRATION.md` | W043 scope and `calc-2p3.4` gate |
| `docs/spec/core-engine/w043-formalization/W043_RESIDUAL_RELEASE_GRADE_PROOF_SERVICE_OBLIGATION_MAP.md` | W043 obligations `W043-OBL-012` through `W043-OBL-014` and no-promotion guard |
| `docs/spec/core-engine/w043-formalization/W043_RUST_TOTALITY_REFINEMENT_AND_PANIC_FREE_CORE_PROOF_FRONTIER.md` | W043.3 Rust totality/refinement and callable-carrier evidence |
| `archive/test-runs-core-engine-w038-w045/formal-assurance/w042-lean-tla-fairness-full-verification-expansion-001/` | predecessor Lean/TLA proof/model packet |
| `archive/test-runs-core-engine-w038-w045/stage2-replay/w042-stage2-production-analyzer-pack-grade-equivalence-closure-001/` | bounded Stage 2 analyzer and pack-grade equivalence input |
| `docs/test-runs/core-engine/formal-inventory/w037-proof-model-closure-001/` | checked Lean and bounded TLA inventory floor |
| `../OxFml/docs/worksets/W073_conditional_formatting_typed_visualization_payload.md` | current W073 typed-only formatting guard |

## 3. Artifact Surface

Run id: `w043-lean-tla-full-verification-unbounded-fairness-001`

| Artifact | Result |
|---|---|
| `formal/lean/OxCalc/CoreEngine/W043LeanTlaFullVerificationAndFairness.lean` | checked Lean row model for W043 Lean/TLA proof/model classification |
| `archive/test-runs-core-engine-w038-w045/formal-assurance/w043-lean-tla-full-verification-unbounded-fairness-001/run_summary.json` | 15 rows, 9 local checked-proof rows, 4 bounded-model rows, 1 accepted external seam, 2 accepted boundaries, 5 totality boundaries, 5 exact blockers, 0 failed rows |
| `w043_lean_tla_discharge_ledger.json` | machine-readable 15-row Lean/TLA discharge ledger |
| `w043_lean_proof_register.json` | 9 local checked-proof classification rows |
| `w043_tla_model_bound_register.json` | 4 bounded-model rows |
| `w043_lean_tla_exact_blocker_register.json` | 5 exact remaining proof/model blockers |
| `source_evidence_index.json` | source evidence index for W043.1, W043.3, W042 Lean/TLA, W042 Stage 2, W037 inventory, W073, and Lean artifacts |
| `validation.json` | validation status `formal_assurance_w043_lean_tla_fairness_valid` |

## 4. Implementation Delta

Changed files:

1. `formal/lean/OxCalc/CoreEngine/W043LeanTlaFullVerificationAndFairness.lean`
2. `src/oxcalc-tracecalc/src/formal_assurance.rs`

The formal-assurance runner now has a W043 Lean/TLA profile that:

1. reads W043 proof-service obligation artifacts,
2. reads W037 formal inventory and TLA inventory artifacts,
3. reads the W042 Lean/TLA predecessor packet and exact blockers,
4. reads the W043 Rust totality/refinement ledger, refinement rows, and blockers,
5. reads the W042 Stage 2 analyzer/pack-grade equivalence packet,
6. reads the W043 W073 typed-only formatting intake,
7. emits W043 Lean/TLA discharge ledger, Lean proof register, TLA model-bound register, exact blocker register, source evidence index, validation, and run summary artifacts,
8. validates 15 rows, 9 checked-proof rows, 4 bounded-model rows, 5 exact blockers, and 0 failed rows.

## 5. Row Disposition

| Row | Disposition |
|---|---|
| Lean inventory and placeholder audit | checked Lean evidence |
| W042 Lean/TLA predecessor bridge | checked Lean bridge evidence |
| W043 Rust dynamic addition bridge | checked Lean refinement bridge |
| W043 Rust dynamic release bridge | checked Lean refinement bridge |
| W043 callable carrier bridge | checked Lean callable-carrier bridge |
| W042 Stage 2 analyzer/pack predicate | checked Lean policy predicate |
| routine TLC config set | bounded model with exact totality boundary |
| Stage 2 partition bounded configs | bounded Stage 2 model evidence |
| W042 Stage 2 equivalence packet | bounded Stage 2 equivalence input |
| scheduler fairness and unbounded interleaving | exact model-assumption boundary |
| full Lean verification | exact proof blocker |
| full TLA verification | exact model blocker |
| Rust totality/refinement dependency | exact proof/model blocker |
| `LET`/`LAMBDA` carrier seam | accepted external seam boundary |
| spec-evolution guard | accepted boundary |

Observed counts:

1. 15 proof/model rows.
2. 9 local checked-proof classification rows.
3. 4 bounded-model rows.
4. 1 accepted external seam.
5. 2 accepted boundaries.
6. 5 totality-boundary rows.
7. 5 exact remaining blockers.
8. 2 dynamic-refinement bridge rows.
9. 0 failed rows.

## 6. Fairness And Model-Bound Position

The W043.4 packet keeps the TLA floor explicit:

1. The routine TLC inventory still has 11 passed configs and 0 failed configs.
2. The W042 Stage 2 packet has bounded partition replay, permutation replay, observable-invariance, bounded analyzer evidence, declared pack-grade equivalence input, and 6 exact Stage 2 blockers.
3. Those rows are bounded or declared-profile evidence only.
4. Full TLA verification remains blocked by unbounded model coverage, scheduler fairness, production partition-analyzer soundness, and downstream operated/pack evidence.
5. Stage 2 production partition-analyzer and scheduler equivalence remain owned by `calc-2p3.5`.

This is a proof/model strengthening packet, not a promotion packet.

## 7. Rust And Callable Bridges

W043.4 consumes the W043.3 Rust rows:

1. `w043_automatic_dynamic_addition_refinement_evidence`,
2. `w043_automatic_dynamic_release_refinement_evidence`,
3. `w043_callable_value_carrier_totality_evidence`.

Those rows are checked proof inputs for the formal model. They do not discharge retained Rust or callable blockers:

1. runtime panic-surface totality boundary,
2. broader dynamic transition coverage,
3. callable metadata projection,
4. full optimized/core release-grade conformance,
5. callable carrier sufficiency beyond the current value-carrier row,
6. general OxFunc kernels.

## 8. OxFml W073 Formatting Intake

The latest OxFml formatting update was reviewed against this packet.

The W073 contract remains typed-only for aggregate and visualization conditional-formatting metadata:

1. `VerificationConditionalFormattingRule.typed_rule` is required for `colorScale`, `dataBar`, `iconSet`, `top`, `bottom`, `aboveAverage`, and `belowAverage`.
2. W072 bounded `thresholds` strings are intentionally ignored for those families.
3. `thresholds` remains meaningful only for scalar/operator/expression rule families where threshold text is the real rule input.

W043.4 does not construct conditional-formatting requests and does not change OxFml evaluator behavior. The source index therefore carries the W043 W073 intake as a seam-watch source; broader request construction and public migration remain owned by `calc-2p3.8`.

No OxFml handoff is required by this bead.

## 9. Semantic-Equivalence Statement

This packet adds a checked Lean classification file, a W043 formal-assurance runner profile, emitted formal-assurance artifacts, and documentation.

It does not change coordinator scheduling, invalidation strategy, dependency graph construction, soft-reference update behavior, recalc behavior, publication semantics, reject policy, overlay lifecycle behavior, TraceCalc semantics, TreeCalc/CoreEngine runtime behavior, OxFml runtime behavior, OxFunc callable kernels, Stage 2 scheduler policy, pack/C5 capability policy, operated-service behavior, alert dispatch behavior, retained-history behavior, or retained-witness behavior.

Observable engine results are invariant under this packet. The only changed observable artifacts are W043.4 formal-assurance evidence files and the checked Lean classification file.

## 10. Verification

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | passed |
| `lean formal/lean/OxCalc/CoreEngine/W043LeanTlaFullVerificationAndFairness.lean` | passed |
| `cargo test -p oxcalc-tracecalc formal_assurance_runner_classifies_w043_lean_tla_fairness_expansion -- --nocapture` | passed |
| `cargo run -p oxcalc-tracecalc-cli -- formal-assurance w043-lean-tla-full-verification-unbounded-fairness-001` | passed; emitted 15 rows with 0 failed rows |
| JSON parse for W043.4 formal-assurance artifacts | passed |
| `lean formal/lean/OxCalc/CoreEngine/W043RustTotalityAndRefinement.lean` | passed |
| `lean formal/lean/OxCalc/CoreEngine/W042Stage2ProductionAnalyzerAndPackGradeEquivalence.lean` | passed |
| `cargo test -p oxcalc-tracecalc formal_assurance -- --nocapture` | passed |
| `cargo test -p oxcalc-tracecalc` | passed; 61 tests |
| `scripts/check-worksets.ps1` | passed after bead closure |
| `br ready --json` | passed after bead closure; next ready bead is `calc-2p3.5` |
| `br dep cycles --json` | passed with `count: 0` |
| `git diff --check` | passed after bead closure; CRLF normalization warnings only |

## 11. Pre-Closure Verification Checklist

| # | Check | Result |
|---|---|---|
| 1 | Spec text and realization notes updated for all in-scope items? | yes; this packet, W043 README/status surfaces, feature map, Lean file, runner profile, and machine-readable artifacts record the slice |
| 2 | Pack expectations updated for affected packs? | yes; pack/C5 remains unpromoted and `calc-2p3.9` owns reassessment |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes; W043.3, W042 Lean/TLA, W042 Stage 2, W037 TLA inventory, and W043.4 formal-assurance artifacts are bound |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes; no runtime strategy changed |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes; current W073 typed-only formatting intake is carried and no handoff trigger exists |
| 6 | All required tests pass? | yes; see Section 10 |
| 7 | No known semantic gaps remain in declared scope? | yes for this W043.4 classification target; broader exact blockers are explicit |
| 8 | Completion language audit passed? | yes; no full Lean/TLA, scheduler fairness, unbounded model coverage, Rust totality/refinement, optimized/core, Stage 2, pack/C5, service, independent-diversity, broad OxFml, callable metadata, callable carrier, registered-external, provider-publication, or general OxFunc promotion is claimed |
| 9 | `WORKSET_REGISTER.md` updated when ordered workset truth changed? | not applicable; ordered workset truth is unchanged |
| 10 | `IN_PROGRESS_FEATURE_WORKLIST.md` updated when feature-map truth changed? | yes |
| 11 | execution-state blocker surface updated? | yes; `.beads/` records `calc-2p3.4` closure and `calc-2p3.5` readiness |

## 12. Completion Claim Self-Audit

| Step | Audit result |
|---|---|
| Scope re-read | pass; `calc-2p3.4` asks for Lean/TLA verification deepening with fairness and unbounded-model boundaries explicit |
| Gate criteria re-read | pass; discharged proof rows, bounded model rows, exact blockers, accepted boundaries, and no-promotion claims are separated |
| Silent scope reduction check | pass; full Lean/TLA, fairness, unbounded model coverage, Rust dependency, Stage 2 policy, pack/C5, service, OxFml breadth, callable metadata, callable carrier sufficiency, and general OxFunc lanes remain explicit open lanes |
| "Looks done but is not" pattern check | pass; checked classification and bounded model evidence are not reported as full verification |
| Result | pass for the `calc-2p3.4` target |

## 13. Three-Axis Report

- execution_state: `calc-2p3.4_lean_tla_full_verification_unbounded_fairness_validated`
- scope_completeness: `scope_partial`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- open_lanes:
  - `calc-2p3.5` Stage 2 production partition analyzer and scheduler equivalence is next
  - full Lean verification remains blocked
  - full TLA verification remains blocked
  - scheduler fairness and unbounded model coverage remain exact blockers
  - runtime panic-surface proof, broader dynamic transition coverage, callable metadata projection, callable carrier sufficiency, full optimized/core verification, Rust totality/refinement, release-grade verification, C5, pack-grade replay, Stage 2 policy, operated services, independent evaluator breadth, mismatch quarantine, broad OxFml display/publication, public migration, registered-external callable projection, provider-failure/callable-publication semantics, and general OxFunc kernels remain unpromoted
