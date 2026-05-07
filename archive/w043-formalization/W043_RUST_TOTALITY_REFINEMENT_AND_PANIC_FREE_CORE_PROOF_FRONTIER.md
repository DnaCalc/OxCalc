# W043 Rust Totality Refinement And Panic-Free Core Proof Frontier

Status: `calc-2p3.3_rust_totality_refinement_panic_free_frontier_validated`
Workset: `W043`
Parent epic: `calc-2p3`
Bead: `calc-2p3.3`

## 1. Purpose

This packet strengthens the W043 Rust totality/refinement frontier after `calc-2p3.2`.

The narrow result is a checked W043 Lean row model plus a formal-assurance packet that binds the W043 optimized/core conformance evidence, the fresh 27-case TreeCalc replay, W042 Rust predecessor evidence, W073 typed-only formatting intake, ordinary `LET`/`LAMBDA` value-carrier evidence, and explicit exact blockers.

The new W043 refinement evidence distinguishes two automatic dynamic dependency transitions:

1. potential-to-resolved dynamic addition: `DependencyAdded` plus `DependencyReclassified`,
2. resolved-to-potential dynamic release: `DependencyRemoved` plus `DependencyReclassified`.

Both exercised transitions force rebind behavior before reevaluation. This narrows the Rust refinement frontier, but it does not promote Rust-engine totality, Rust refinement, panic-free core-domain proof, full optimized/core verification, full Lean/TLA verification, Stage 2 policy, pack-grade replay, C5, callable metadata projection, callable carrier sufficiency, release-grade verification, or general OxFunc kernels.

## 2. Authority Inputs Reviewed

| Input | Role |
|---|---|
| `docs/worksets/W043_CORE_FORMALIZATION_RELEASE_GRADE_PROOF_AND_OPERATED_SERVICE_INTEGRATION.md` | W043 scope and `calc-2p3.3` gate |
| `docs/spec/core-engine/w043-formalization/W043_RESIDUAL_RELEASE_GRADE_PROOF_SERVICE_OBLIGATION_MAP.md` | W043 obligations `W043-OBL-009` through `W043-OBL-011` and no-promotion guard |
| `docs/spec/core-engine/w043-formalization/W043_OPTIMIZED_CORE_BROAD_CONFORMANCE_AND_CALLABLE_METADATA_CLOSURE.md` | W043.2 optimized/core evidence and exact blockers |
| `archive/test-runs-core-engine-w038-w045/implementation-conformance/w043-optimized-core-broad-conformance-callable-metadata-closure-001/` | dynamic transition, counterpart, callable, blocker, W073, and validation artifacts |
| `archive/test-runs-core-engine-w038-w045/treecalc-local/w043-optimized-core-broad-conformance-treecalc-001/` | 27-case TreeCalc replay and post-edit transition artifacts |
| `archive/test-runs-core-engine-w038-w045/formal-assurance/w042-rust-totality-refinement-core-panic-boundary-001/` | predecessor Rust totality/refinement packet |
| `../OxFml/docs/worksets/W073_conditional_formatting_typed_visualization_payload.md` | current W073 typed-only formatting guard |

## 3. Artifact Surface

Run id: `w043-rust-totality-refinement-panic-free-frontier-001`

| Artifact | Result |
|---|---|
| `formal/lean/OxCalc/CoreEngine/W043RustTotalityAndRefinement.lean` | checked Lean row model for W043 Rust totality/refinement classification |
| `archive/test-runs-core-engine-w038-w045/formal-assurance/w043-rust-totality-refinement-panic-free-frontier-001/run_summary.json` | 14 rows, 11 local checked-proof classification rows, 0 bounded-model rows, 1 accepted external seam, 2 accepted boundaries, 4 totality boundaries, 8 refinement rows, 4 exact blockers, 0 failed rows |
| `w043_rust_totality_refinement_ledger.json` | machine-readable 14-row Rust totality/refinement ledger |
| `w043_rust_totality_boundary_register.json` | 4 totality-boundary rows |
| `w043_rust_refinement_register.json` | 8 refinement rows, including 2 automatic dynamic transition rows |
| `w043_rust_exact_blocker_register.json` | 4 exact remaining blockers |
| `source_evidence_index.json` | source evidence index for W043.1, W043.2, W042 Rust, TreeCalc, W073, and Lean artifacts |
| `validation.json` | validation status `formal_assurance_w043_rust_totality_refinement_valid` |

## 4. Implementation Delta

Changed files:

1. `formal/lean/OxCalc/CoreEngine/W043RustTotalityAndRefinement.lean`
2. `src/oxcalc-tracecalc/src/formal_assurance.rs`

The formal-assurance runner now has a W043 Rust profile that:

1. reads W043 proof-service obligation artifacts,
2. reads W042 Rust predecessor formal-assurance artifacts,
3. reads W043 optimized/core conformance, dynamic-transition, callable, blocker, W073, and validation artifacts,
4. reads the W043 TreeCalc post-edit artifacts for automatic addition and release transitions,
5. counts current audited panic-family markers across the core Rust audit surface,
6. emits W043 Rust totality/refinement ledger, totality-boundary register, refinement register, blocker register, source evidence index, validation, and run summary artifacts,
7. validates 14 rows, 2 automatic dynamic-transition refinement rows, 4 exact blockers, and 0 failed rows.

## 5. Row Disposition

| Row | Disposition |
|---|---|
| result/error carrier | direct totality evidence |
| W043 TreeCalc packet | direct totality evidence |
| W042 Rust predecessor packet | direct refinement regression evidence |
| automatic dependency addition | direct refinement evidence for the exercised potential-to-resolved pattern |
| automatic dependency release | direct refinement evidence for the exercised resolved-to-potential pattern |
| snapshot-fence declared-profile counterpart | direct declared-profile refinement evidence |
| capability-view declared-profile counterpart | direct declared-profile refinement evidence |
| `LET`/`LAMBDA` ordinary value carrier | direct callable value-carrier totality evidence |
| runtime panic surface | exact totality boundary and blocker |
| broader dynamic transition coverage | exact refinement blocker |
| callable metadata projection | exact totality/refinement blocker |
| full optimized/core release-grade conformance | exact release-grade boundary |
| `LET`/`LAMBDA` carrier seam | accepted external seam boundary |
| spec-evolution refinement guard | accepted boundary |

Observed counts:

1. 14 Rust/proof rows.
2. 11 local checked-proof classification rows.
3. 0 bounded-model rows.
4. 1 accepted external seam.
5. 2 accepted boundaries.
6. 4 totality-boundary rows.
7. 8 refinement rows.
8. 2 automatic dynamic-transition refinement rows.
9. 4 exact remaining blockers.
10. 0 failed rows.

## 6. Dynamic And Callable Classification

The W043.3 formal-assurance runner checks:

1. TreeCalc emitted 27 cases with 0 expectation mismatches.
2. The automatic addition transition derives both `DependencyAdded` and `DependencyReclassified`.
3. The automatic release transition derives both `DependencyRemoved` and `DependencyReclassified`.
4. Both post-edit invalidation closures mark node 3 as `requires_rebind: true`.
5. Both post-edit reruns reject as `HostInjectedFailure`.
6. The addition post-edit rejection commits no new publication.
7. The higher-order `LET`/`LAMBDA` case publishes ordinary value `17` through current candidate/publication value carriers.

This packet strengthens Rust refinement evidence for two exercised dynamic transition shapes. It does not prove all dynamic reference families, multi-descriptor mixtures, structural-plus-formula edits, or production host-resolution behavior.

## 7. Remaining Exact Blockers

| Blocker | Owner |
|---|---|
| `w043_runtime_panic_surface_totality_boundary` | `calc-2p3.3`; successor audit in `calc-2p3.10` |
| `w043_broader_dynamic_transition_coverage_refinement_boundary` | `calc-2p3.3`; `calc-2p3.4`; `calc-2p3.5`; `calc-2p3.10` |
| `w043_callable_metadata_projection_totality_boundary` | `calc-2p3.4`; `calc-2p3.8`; external OxFunc semantic authority remains outside OxCalc |
| `w043_full_optimized_core_release_grade_conformance_boundary` | `calc-2p3.10` |

The Rust panic-surface audit currently observes 152 panic-family markers across 12 audited core files. That count is a guardrail, not a semantic proof. Panic-free whole-engine totality remains unpromoted.

## 8. OxFml Formatting Intake

The latest W073 formatting intake remains aligned with W043:

1. `VerificationConditionalFormattingRule.typed_rule` is the only accepted metadata source for `colorScale`, `dataBar`, `iconSet`, `top`, `bottom`, `aboveAverage`, and `belowAverage`.
2. W072 bounded `thresholds` strings are intentionally ignored for those aggregate and visualization families.
3. `thresholds` remains meaningful only for scalar/operator/expression rules.
4. W043.3 does not construct conditional-formatting requests and does not require an OxCalc core-engine code change.
5. No OxFml handoff is required by this bead.

## 9. Semantic-Equivalence Statement

This packet adds a checked Lean classification file, a W043 formal-assurance runner profile, emitted formal-assurance artifacts, and documentation.

It does not change coordinator scheduling, invalidation strategy, dependency graph construction, soft-reference update behavior, recalc behavior, publication semantics, reject policy, overlay lifecycle behavior, TraceCalc semantics, TreeCalc/CoreEngine runtime behavior, OxFml runtime behavior, OxFunc callable kernels, Stage 2 scheduler policy, pack/C5 capability policy, operated-service behavior, alert dispatch behavior, retained-history behavior, or retained-witness behavior.

Observable engine results are invariant under this packet. The only changed observable artifacts are W043.3 formal-assurance evidence files and the checked Lean classification file.

## 10. Verification

| Command | Result |
|---|---|
| `lean formal/lean/OxCalc/CoreEngine/W043RustTotalityAndRefinement.lean` | passed |
| `cargo test -p oxcalc-tracecalc formal_assurance_runner_classifies_w043_rust_totality_and_refinement -- --nocapture` | passed |
| `cargo run -p oxcalc-tracecalc-cli -- formal-assurance w043-rust-totality-refinement-panic-free-frontier-001` | passed; emitted 14 rows with 0 failed rows |
| JSON parse for W043.3 formal-assurance artifacts | passed |
| `cargo fmt --all -- --check` | passed |
| `cargo test -p oxcalc-tracecalc formal_assurance -- --nocapture` | passed |
| `cargo test -p oxcalc-tracecalc` | passed; 60 tests |
| `scripts/check-worksets.ps1` | passed after bead closure |
| `br ready --json` | passed after bead closure; next ready bead is `calc-2p3.4` |
| `br dep cycles --json` | passed with `count: 0` |
| `git diff --check` | passed after bead closure; CRLF normalization warnings only |

## 11. Pre-Closure Verification Checklist

| # | Check | Result |
|---|---|---|
| 1 | Spec text and realization notes updated for all in-scope items? | yes; this packet, W043 README/status surfaces, feature map, Lean file, runner profile, and machine-readable artifacts record the slice |
| 2 | Pack expectations updated for affected packs? | yes; pack/C5 remains unpromoted and `calc-2p3.9` owns reassessment |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes; W043.2 TreeCalc replay and W043.3 formal-assurance artifacts are bound |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes; no runtime strategy changed |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes; current W073 typed-only formatting intake is carried and no handoff trigger exists |
| 6 | All required tests pass? | yes; see Section 10 |
| 7 | No known semantic gaps remain in declared scope? | yes for this W043.3 classification target; broader exact blockers are explicit |
| 8 | Completion language audit passed? | yes; no Rust-engine totality, panic-free core-domain proof, Rust refinement, optimized/core, Lean/TLA, Stage 2, pack/C5, service, independent-diversity, broad OxFml, callable metadata, callable carrier, registered-external, provider-publication, or general OxFunc promotion is claimed |
| 9 | `WORKSET_REGISTER.md` updated when ordered workset truth changed? | not applicable; ordered workset truth is unchanged |
| 10 | `IN_PROGRESS_FEATURE_WORKLIST.md` updated when feature-map truth changed? | yes |
| 11 | execution-state blocker surface updated? | yes; `.beads/` records `calc-2p3.3` closure and `calc-2p3.4` readiness |

## 12. Completion Claim Self-Audit

| Step | Audit result |
|---|---|
| Scope re-read | pass; `calc-2p3.3` asks for Rust totality/refinement and panic-free core proof frontier evidence |
| Gate criteria re-read | pass; each totality/refinement claim has direct evidence, checked proof/model classification, or exact blockers |
| Silent scope reduction check | pass; broader Rust totality/refinement, panic-free proof, full optimized/core, release-grade, Stage 2, pack/C5, service, OxFml breadth, callable metadata, callable carrier sufficiency, and general OxFunc lanes remain explicit open lanes |
| "Looks done but is not" pattern check | pass; checked Lean classification, marker census, and two automatic dynamic-transition rows are not reported as whole-engine proof or runtime implementation |
| Result | pass for the `calc-2p3.3` target |

## 13. Three-Axis Report

- execution_state: `calc-2p3.3_rust_totality_refinement_panic_free_frontier_validated`
- scope_completeness: `scope_partial`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- open_lanes:
  - `calc-2p3.4` Lean/TLA full-verification and unbounded fairness discharge is next
  - broader dynamic dependency-transition coverage remains partial beyond fixture-scale addition/release and reclassification patterns
  - callable metadata projection remains an exact blocker
  - callable carrier sufficiency proof remains blocked
  - runtime panic-surface proof remains an exact blocker
  - full optimized/core verification, Rust-engine totality, Rust refinement, release-grade verification, C5, pack-grade replay, Stage 2 policy, operated services, independent evaluator breadth, mismatch quarantine, broad OxFml display/publication, public migration, registered-external callable projection, provider-failure/callable-publication semantics, and general OxFunc kernels remain unpromoted
