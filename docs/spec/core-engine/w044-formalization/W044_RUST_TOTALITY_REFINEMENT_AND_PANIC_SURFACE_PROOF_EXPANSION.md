# W044 Rust Totality Refinement And Panic-Surface Proof Expansion

Status: `calc-b1t.3_rust_totality_refinement_panic_surface_expansion_validated`
Workset: `W044`
Parent epic: `calc-b1t`
Bead: `calc-b1t.3`

## 1. Purpose

This packet widens the W044 Rust totality/refinement and panic-surface proof frontier after `calc-b1t.2`.

The narrow result is a checked W044 Lean row model plus a formal-assurance packet that binds the W044 mixed dynamic add/release evidence, publication no-publish fence evidence, W043 Rust predecessor evidence, W044 optimized/core conformance evidence, current W073 typed-only formatting intake, ordinary `LET`/`LAMBDA` value-carrier evidence, and retained exact blockers.

The new W044 refinement evidence proves the exercised mixed dynamic transition derives all three dependency transition reasons:

1. `DependencyAdded`,
2. `DependencyRemoved`,
3. `DependencyReclassified`.

The exercised post-edit path rejects as `HostInjectedFailure` and publishes no value. This narrows the Rust refinement frontier, but it does not promote Rust-engine totality, Rust refinement, panic-free core-domain proof, full optimized/core verification, full Lean/TLA verification, Stage 2 policy, pack-grade replay, C5, callable metadata projection, callable carrier sufficiency, release-grade verification, broad OxFml closure, or general OxFunc kernels.

## 2. Authority Inputs Reviewed

| Input | Role |
|---|---|
| `docs/worksets/W044_CORE_FORMALIZATION_RELEASE_GRADE_BLOCKER_BURN_DOWN_AND_SERVICE_PROOF_CLOSURE.md` | W044 scope and `calc-b1t.3` gate |
| `docs/spec/core-engine/w044-formalization/W044_RESIDUAL_RELEASE_GRADE_BLOCKER_RECLASSIFICATION_AND_PROMOTION_CONTRACT_MAP.md` | W044 obligations, promotion contracts, no-promotion guard, W073 intake |
| `docs/spec/core-engine/w044-formalization/W044_OPTIMIZED_CORE_DYNAMIC_TRANSITION_AND_CALLABLE_METADATA_IMPLEMENTATION.md` | W044.2 optimized/core evidence and exact blockers |
| `docs/test-runs/core-engine/implementation-conformance/w044-optimized-core-dynamic-transition-callable-metadata-001/` | dynamic transition, callable metadata, blocker, and validation artifacts |
| `docs/test-runs/core-engine/treecalc-local/w044-optimized-core-dynamic-transition-treecalc-001/` | 28-case TreeCalc replay and mixed dynamic post-edit artifacts |
| `docs/test-runs/core-engine/formal-assurance/w043-rust-totality-refinement-panic-free-frontier-001/` | predecessor Rust totality/refinement packet |
| `../OxFml/docs/worksets/W073_conditional_formatting_typed_visualization_payload.md` | current W073 typed-only formatting guard |
| `../OxFml/docs/handoffs/HANDOFF-DNAONECALC-012_W073_TYPED_CF_PAYLOAD_FIRST_SLICE.md` | downstream typed-rule request-construction handoff |

## 3. Artifact Surface

Run id: `w044-rust-totality-refinement-panic-surface-expansion-001`

| Artifact | Result |
|---|---|
| `formal/lean/OxCalc/CoreEngine/W044RustTotalityAndRefinement.lean` | checked Lean row model for W044 Rust totality/refinement classification |
| `docs/test-runs/core-engine/formal-assurance/w044-rust-totality-refinement-panic-surface-expansion-001/run_summary.json` | 16 rows, 11 local checked-proof classification rows, 0 bounded-model rows, 1 accepted external seam, 3 accepted boundaries, 4 totality boundaries, 9 refinement rows, 6 exact blockers, 0 failed rows |
| `w044_rust_totality_refinement_ledger.json` | machine-readable 16-row Rust totality/refinement ledger |
| `w044_rust_totality_boundary_register.json` | 4 totality-boundary rows |
| `w044_rust_refinement_register.json` | 9 refinement rows, including 1 automatic mixed dynamic transition row |
| `w044_rust_exact_blocker_register.json` | 6 exact remaining blockers |
| `source_evidence_index.json` | source evidence index for W044.1, W044.2, W043 Rust, TreeCalc, W073, and Lean artifacts |
| `validation.json` | validation status `formal_assurance_w044_rust_totality_refinement_valid` |

## 4. Implementation Delta

Changed files:

1. `formal/lean/OxCalc/CoreEngine/W044RustTotalityAndRefinement.lean`
2. `src/oxcalc-tracecalc/src/formal_assurance.rs`

The formal-assurance runner now has a W044 Rust profile that:

1. reads W044 residual-blocker and promotion-contract artifacts,
2. reads W043 Rust predecessor formal-assurance artifacts,
3. reads W044 optimized/core conformance, dynamic-transition, callable, blocker, and validation artifacts,
4. reads the W044 TreeCalc mixed dynamic post-edit and `LET`/`LAMBDA` artifacts,
5. counts current audited panic-family markers across the core Rust audit surface,
6. emits W044 Rust totality/refinement ledger, totality-boundary register, refinement register, blocker register, source evidence index, validation, and run summary artifacts,
7. validates 16 rows, 1 automatic mixed dynamic-transition refinement row, 6 exact blockers, and 0 failed rows.

## 5. Row Disposition

| Row | Disposition |
|---|---|
| result/error carrier | direct totality evidence |
| W044 TreeCalc packet | direct totality evidence |
| W043 Rust predecessor packet | carried refinement regression evidence |
| mixed dynamic add/release transition | direct refinement evidence for the exercised mixed pattern |
| publication no-publish fence | direct refinement evidence for the exercised reject/no-publication path |
| W043 dynamic-transition predecessor rows | carried refinement regression evidence |
| snapshot-fence counterpart breadth | exact refinement blocker |
| capability-view counterpart breadth | exact refinement blocker |
| `LET`/`LAMBDA` ordinary value carrier | direct callable value-carrier totality evidence |
| runtime panic surface | exact totality boundary and blocker |
| broader dynamic transition coverage | exact refinement blocker |
| callable metadata projection | exact totality/refinement blocker |
| full optimized/core release-grade conformance | exact release-grade boundary |
| `LET`/`LAMBDA` carrier seam | accepted external seam boundary |
| spec-evolution refinement guard | accepted boundary |
| W073 typed formatting guard | accepted formatting boundary |

Observed counts:

1. 16 Rust/proof rows.
2. 11 local checked-proof classification rows.
3. 0 bounded-model rows.
4. 1 accepted external seam.
5. 3 accepted boundaries.
6. 4 totality-boundary rows.
7. 9 refinement rows.
8. 1 automatic mixed dynamic-transition refinement row.
9. 6 exact remaining blockers.
10. 0 failed rows.

## 6. Dynamic And Callable Classification

The W044.3 formal-assurance runner checks:

1. TreeCalc emitted 28 cases with 0 expectation mismatches.
2. The mixed dynamic transition derives `DependencyAdded`, `DependencyRemoved`, and `DependencyReclassified`.
3. The mixed dynamic post-edit result rejects as `HostInjectedFailure`.
4. The mixed dynamic post-edit result exposes an empty `published_values` object.
5. The W043 predecessor Rust packet remains valid and has 0 failed rows.
6. The higher-order `LET`/`LAMBDA` case publishes ordinary value `17` through current candidate/publication value carriers.

This packet strengthens Rust refinement evidence for the exercised mixed add/release dynamic transition and no-publication fence. It does not prove all dynamic reference families, all snapshot/capability counterpart breadth, all callable metadata projection, structural-plus-formula edit breadth, or production host-resolution behavior.

## 7. Remaining Exact Blockers

| Blocker | Owner |
|---|---|
| `w044_runtime_panic_surface_totality_boundary` | `calc-b1t.3`; successor audit in `calc-b1t.10` |
| `w044_broader_dynamic_transition_coverage_refinement_boundary` | `calc-b1t.3`; `calc-b1t.4`; `calc-b1t.5`; `calc-b1t.10` |
| `w044_snapshot_fence_breadth_refinement_boundary` | `calc-b1t.3`; `calc-b1t.5` |
| `w044_capability_view_breadth_refinement_boundary` | `calc-b1t.3`; `calc-b1t.5` |
| `w044_callable_metadata_projection_totality_boundary` | `calc-b1t.3`; `calc-b1t.8`; external OxFunc semantic authority remains outside OxCalc |
| `w044_full_optimized_core_release_grade_conformance_boundary` | `calc-b1t.10`; `calc-b1t.11` |

The Rust panic-surface audit currently observes 158 panic-family markers across 12 audited core files. That count is a guardrail, not a semantic proof. Panic-free whole-engine totality remains unpromoted.

## 8. OxFml Formatting Intake

The latest W073 formatting intake remains aligned with W044:

1. `VerificationConditionalFormattingRule.typed_rule` is the only accepted metadata source for `colorScale`, `dataBar`, `iconSet`, `top`, `bottom`, `aboveAverage`, and `belowAverage`.
2. W072 bounded `thresholds` strings are intentionally ignored for those aggregate and visualization families.
3. `thresholds` remains meaningful only for scalar/operator/expression rules.
4. DNA OneCalc downstream typed-rule request construction is required but remains unverified by OxCalc.
5. W044.3 does not construct conditional-formatting requests and does not require an OxCalc core-engine code change.
6. No OxFml handoff is required by this bead.

## 9. Semantic-Equivalence Statement

This packet adds a checked Lean classification file, a W044 formal-assurance runner profile, emitted formal-assurance artifacts, and documentation.

It does not change coordinator scheduling, invalidation strategy, dependency graph construction, soft-reference update behavior, recalc behavior, publication semantics, reject policy, overlay lifecycle behavior, TraceCalc semantics, TreeCalc/CoreEngine runtime behavior, OxFml runtime behavior, OxFunc callable kernels, Stage 2 scheduler policy, pack/C5 capability policy, operated-service behavior, alert dispatch behavior, retained-history behavior, or retained-witness behavior.

Observable engine results are invariant under this packet. The only changed observable artifacts are W044.3 formal-assurance evidence files and the checked Lean classification file.

## 10. Verification

| Command | Result |
|---|---|
| `lean formal/lean/OxCalc/CoreEngine/W044RustTotalityAndRefinement.lean` | passed |
| `cargo fmt --all -- --check` | passed |
| `cargo test -p oxcalc-tracecalc formal_assurance_runner_classifies_w044_rust_totality_and_refinement -- --nocapture` | passed |
| `cargo run -p oxcalc-tracecalc-cli -- formal-assurance w044-rust-totality-refinement-panic-surface-expansion-001` | passed; emitted 16 rows with 0 failed rows |
| JSON parse for W044.3 formal-assurance artifacts | passed |
| `cargo test -p oxcalc-tracecalc formal_assurance -- --nocapture` | passed; 11 tests |
| `scripts/check-worksets.ps1` | passed after bead closure |
| `br ready --json` | passed after bead closure; next ready bead is `calc-b1t.4` |
| `br dep cycles --json` | passed with `count: 0` |
| `git diff --check` | passed after bead closure; CRLF normalization warnings only |

## 11. Pre-Closure Verification Checklist

| # | Check | Result |
|---|---|---|
| 1 | Spec text and realization notes updated for all in-scope items? | yes; this packet, W044 README/status surfaces, feature map, Lean file, runner profile, and machine-readable artifacts record the slice |
| 2 | Pack expectations updated for affected packs? | yes; pack/C5 remains unpromoted and `calc-b1t.10` owns reassessment |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes; W044.2 TreeCalc replay and W044.3 formal-assurance artifacts are bound |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes; no runtime strategy changed |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes; current W073 typed-only formatting intake is carried and no handoff trigger exists |
| 6 | All required tests pass? | yes for this target; see Section 10 |
| 7 | No known semantic gaps remain in declared scope? | yes for this W044.3 classification target; broader exact blockers are explicit |
| 8 | Completion language audit passed? | yes; no Rust-engine totality, panic-free core-domain proof, Rust refinement, optimized/core, Lean/TLA, Stage 2, pack/C5, service, independent-diversity, broad OxFml, callable metadata, callable carrier, registered-external, provider-publication, or general OxFunc promotion is claimed |
| 9 | `WORKSET_REGISTER.md` updated when ordered workset truth changed? | not applicable; ordered workset truth is unchanged |
| 10 | `IN_PROGRESS_FEATURE_WORKLIST.md` updated when feature-map truth changed? | yes |
| 11 | execution-state blocker surface updated? | yes; `.beads/` records `calc-b1t.3` closure and queues `calc-b1t.4` readiness |

## 12. Completion Claim Self-Audit

| Step | Audit result |
|---|---|
| Scope re-read | pass; `calc-b1t.3` asks for Rust totality/refinement and panic-surface proof expansion |
| Gate criteria re-read | pass; each totality/refinement claim has direct evidence, checked proof/model classification, or exact blockers |
| Silent scope reduction check | pass; broader Rust totality/refinement, panic-free proof, full optimized/core, release-grade, Stage 2, pack/C5, service, OxFml breadth, callable metadata, callable carrier sufficiency, and general OxFunc lanes remain explicit open lanes |
| "Looks done but is not" pattern check | pass; checked Lean classification, marker census, and one mixed dynamic-transition row are not reported as whole-engine proof or runtime implementation |
| Result | pass for the `calc-b1t.3` target |

## 13. Three-Axis Report

- execution_state: `calc-b1t.3_rust_totality_refinement_panic_surface_expansion_validated`
- scope_completeness: `scope_partial`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- open_lanes:
  - `calc-b1t.4` Lean/TLA full-verification and unbounded fairness expansion is next
  - broader dynamic dependency-transition coverage remains partial beyond fixture-scale mixed add/release and predecessor addition/release patterns
  - snapshot-fence counterpart breadth remains an exact blocker
  - capability-view counterpart breadth remains an exact blocker
  - callable metadata projection remains an exact blocker
  - callable carrier sufficiency proof remains blocked
  - runtime panic-surface proof remains an exact blocker
  - full optimized/core verification, Rust-engine totality, Rust refinement, release-grade verification, C5, pack-grade replay, Stage 2 policy, operated services, independent evaluator breadth, mismatch quarantine, broad OxFml display/publication, public migration, W073 downstream typed-rule uptake, registered-external callable projection, provider-failure/callable-publication semantics, and general OxFunc kernels remain unpromoted
