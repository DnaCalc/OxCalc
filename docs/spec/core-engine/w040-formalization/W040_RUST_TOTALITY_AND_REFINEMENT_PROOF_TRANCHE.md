# W040 Rust Totality And Refinement Proof Tranche

Status: `calc-tv5.3_rust_totality_refinement_classified_no_promotion`
Workset: `W040`
Parent epic: `calc-tv5`
Bead: `calc-tv5.3`

## 1. Purpose

This packet attacks the W040 Rust totality and refinement target.

The result is deliberately non-promoting. It adds a checked Lean classification file and a W040 formal-assurance runner profile that bind Rust totality, panic-surface, refinement, optimized/core exact blockers, LET/LAMBDA carrier scope, and spec-evolution rows into one deterministic evidence packet.

The target is not to claim whole-engine Rust totality, panic-free core execution, full optimized/core verification, full Lean/TLA verification, Stage 2 production policy, callable metadata projection, pack-grade replay, C5, release-grade verification, broad OxFml display/publication closure, or general OxFunc kernel ownership. The target is to replace a broad Rust totality/refinement gap with direct evidence rows and exact blockers before `calc-tv5.4` starts.

## 2. Authority Inputs Reviewed

| Input | Role |
|---|---|
| `docs/test-runs/core-engine/release-grade-ledger/w040-direct-verification-obligation-map-001/run_summary.json` | W040 obligation-map source; confirms 23 obligations and explicit no-promotion guard for Rust totality/refinement |
| `docs/test-runs/core-engine/release-grade-ledger/w040-direct-verification-obligation-map-001/direct_verification_obligation_map.json` | W040 obligations `W040-OBL-006`, `W040-OBL-007`, and `W040-OBL-020` for Rust totality, refinement, and LET/LAMBDA carrier scope |
| `docs/test-runs/core-engine/formal-assurance/w039-proof-model-totality-closure-001/run_summary.json` | predecessor proof/model state; Rust-engine totality was not promoted |
| `docs/test-runs/core-engine/implementation-conformance/w040-optimized-core-exact-blocker-fixes-differentials-001/run_summary.json` | W040 optimized/core direct evidence and retained exact blocker counts |
| `docs/test-runs/core-engine/implementation-conformance/w040-optimized-core-exact-blocker-fixes-differentials-001/w040_exact_remaining_blocker_register.json` | dynamic transition, snapshot fence, capability-view fence, and callable metadata exact blockers |
| `docs/test-runs/core-engine/implementation-conformance/w040-optimized-core-exact-blocker-fixes-differentials-001/dynamic_release_reclassification_evidence.json` | explicit dependency release/reclassification seed evidence |
| `docs/test-runs/core-engine/treecalc-local/w040-optimized-core-dynamic-release-reclassification-001/run_summary.json` | 25-case TreeCalc run with zero expectation mismatches |
| `docs/test-runs/core-engine/treecalc-local/w040-optimized-core-dynamic-release-reclassification-001/cases/tc_local_dynamic_release_reclassification_post_edit_001/post_edit/result.json` | post-edit rejection evidence for dependency rebind |
| `docs/test-runs/core-engine/treecalc-local/w040-optimized-core-dynamic-release-reclassification-001/cases/tc_local_dynamic_release_reclassification_post_edit_001/post_edit/invalidation_closure.json` | post-edit invalidation closure requiring rebind |
| `../OxFml/docs/worksets/W073_conditional_formatting_typed_visualization_payload.md` and local W040 intake notes | W073 typed-only formatting guard retained; no new OxFml handoff trigger in this bead |

## 3. Artifact Surface

Run id: `w040-rust-totality-refinement-proof-tranche-001`

| Artifact | Result |
|---|---|
| `formal/lean/OxCalc/CoreEngine/W040RustTotalityAndRefinement.lean` | checked Lean classification surface for 10 W040 Rust rows |
| `docs/test-runs/core-engine/formal-assurance/w040-rust-totality-refinement-proof-tranche-001/run_summary.json` | records 10 Rust rows, 7 local checked-proof classification rows, 0 bounded-model rows, 1 external seam, 2 accepted boundaries, 5 totality boundaries, 5 refinement rows, 5 exact blockers, and 0 failed rows |
| `docs/test-runs/core-engine/formal-assurance/w040-rust-totality-refinement-proof-tranche-001/w040_rust_totality_refinement_ledger.json` | machine-readable W040 Rust totality/refinement row ledger |
| `docs/test-runs/core-engine/formal-assurance/w040-rust-totality-refinement-proof-tranche-001/w040_rust_totality_boundary_register.json` | five totality-boundary rows |
| `docs/test-runs/core-engine/formal-assurance/w040-rust-totality-refinement-proof-tranche-001/w040_rust_refinement_register.json` | five refinement rows |
| `docs/test-runs/core-engine/formal-assurance/w040-rust-totality-refinement-proof-tranche-001/w040_rust_exact_blocker_register.json` | five exact remaining Rust/proof blockers |
| `docs/test-runs/core-engine/formal-assurance/w040-rust-totality-refinement-proof-tranche-001/source_evidence_index.json` | source evidence index binding W039 proof/model state, W040 direct obligations, W040 optimized/core evidence, TreeCalc post-edit evidence, and Rust panic-marker audit |
| `docs/test-runs/core-engine/formal-assurance/w040-rust-totality-refinement-proof-tranche-001/validation.json` | validation status `formal_assurance_w040_rust_totality_refinement_valid` |

## 4. Row Disposition

| W040 row | Disposition | Evidence consequence |
|---|---|---|
| `w040_result_error_carrier_totality_evidence` | direct totality carrier evidence | promoted Rust paths use typed `Result`/error carriers, but whole-engine totality remains unpromoted |
| `w040_fixture_invalidation_seed_error_totality_evidence` | direct fixture error-carrier evidence | W040 explicit invalidation seed parsing reports unsupported reasons as typed errors |
| `w040_dependency_seed_rebind_refinement_evidence` | direct refinement evidence | explicit `DependencyRemoved` and `DependencyReclassified` seeds force rebind/no-publication behavior |
| `w040_dynamic_transition_refinement_exact_blocker` | exact refinement blocker | automatic dependency-set transition detection remains absent without manual seed injection |
| `w040_runtime_panic_surface_totality_boundary` | exact totality boundary | panic-free Rust engine claim remains blocked; the audit observed 144 panic-family markers across 12 core Rust files |
| `w040_snapshot_fence_refinement_boundary` | exact refinement blocker | stale accepted-candidate snapshot-fence counterpart remains owned by Stage 2/coordinator evidence |
| `w040_capability_view_fence_refinement_boundary` | exact refinement blocker | compatibility-fenced capability-view mismatch counterpart remains owned by Stage 2/coordinator evidence |
| `w040_callable_metadata_projection_totality_boundary` | exact totality/refinement blocker | callable metadata projection remains blocked; the LET/LAMBDA carrier seam is narrow and general OxFunc remains external |
| `w040_let_lambda_carrier_external_boundary` | accepted external seam boundary | LET/LAMBDA carrier interaction remains in OxCalc/OxFml formalization scope while general OxFunc kernels remain external-owner scope |
| `w040_spec_evolution_refinement_guard` | accepted spec-evolution guard | refinement evidence can correct specs or implementation before later promotion decisions |

## 5. Lean And Runner Surface

`formal/lean/OxCalc/CoreEngine/W040RustTotalityAndRefinement.lean` defines 10 W040 Rust rows and proves:

1. direct non-promoting classification for the Rust result/error carrier and fixture invalidation seed error rows,
2. direct non-promoting refinement classification for the explicit dependency seed rebind evidence row,
3. exact totality/refinement blocker classification for dynamic transition, panic surface, snapshot fence, capability-view fence, and callable metadata projection rows,
4. accepted external boundary classification for the LET/LAMBDA carrier seam,
5. accepted spec-evolution guard classification,
6. summary values of 10 rows, 5 totality boundaries, 5 refinement rows, 5 exact blockers, and no Rust totality/refinement, optimized/core, callable metadata, or general OxFunc kernel promotion.

`src/oxcalc-tracecalc/src/formal_assurance.rs` now has a W040 Rust totality/refinement profile.

The W040 profile:

1. reads W040 direct obligation-map artifacts,
2. reads W039 formal-assurance proof/model artifacts,
3. reads W040 optimized/core evidence, exact blockers, and validation artifacts,
4. reads W040 TreeCalc dynamic release/reclassification artifacts,
5. checks that the W040 Lean Rust classification file exists,
6. emits a Rust totality/refinement ledger, totality-boundary register, refinement register, exact-blocker register, source evidence index, validation, and run summary,
7. validates that W040 carries 10 Rust rows, 5 totality boundaries, 5 refinement rows, 5 exact blockers, and 0 failed rows.

## 6. OxFml And LET/LAMBDA Seam

This bead does not construct a new OxFml formatting payload and does not file an OxFml handoff.

It preserves two seam constraints for later W040 work:

1. W073 aggregate and visualization conditional-formatting metadata is `typed_rule`-only for the named families; W072 bounded `thresholds` strings are not a fallback there.
2. `LET`/`LAMBDA` remains a narrow carrier seam consumed by OxCalc/OxFml integration; general OxFunc kernels remain outside OxCalc formalization scope unless a specific carrier obligation requires a peek.

## 7. Semantic-Equivalence Statement

This bead adds a checked Lean classification file, W040 formal-assurance runner profile, emitted artifacts, and status/spec text.

It does not change coordinator scheduling, invalidation strategy, dependency graph construction, soft-reference update behavior, recalc behavior, publication semantics, reject policy, overlay lifecycle behavior, TreeCalc runtime behavior, TraceCalc reference semantics, optimized/core runtime semantics, OxFml evaluator behavior, OxFunc kernels, Stage 2 scheduler policy, pack/C5 capability policy, operated-service behavior, alert dispatch behavior, or retained-history behavior.

Observable runtime behavior is invariant under this bead. The W040 runner classifies Rust totality/refinement evidence, totality boundaries, accepted seams, exact blockers, and no-promotion predicates only.

## 8. Verification

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | passed before artifact emission |
| `lean formal/lean/OxCalc/CoreEngine/W040RustTotalityAndRefinement.lean` | passed |
| `rg -n "^\\s*(axiom|sorry|admit)\\b" formal/lean` | passed; no placeholders found |
| `cargo test -p oxcalc-tracecalc formal_assurance -- --nocapture` | passed; 3 focused tests |
| `cargo test -p oxcalc-tracecalc` | passed; 39 tests |
| `cargo test -p oxcalc-tracecalc-cli` | passed; 0 tests |
| `cargo run -p oxcalc-tracecalc-cli -- formal-assurance w040-rust-totality-refinement-proof-tranche-001` | passed; emitted W040 Rust totality/refinement artifacts |
| JSON parse for `docs/test-runs/core-engine/formal-assurance/w040-rust-totality-refinement-proof-tranche-001/*.json` | passed |
| `scripts/check-worksets.ps1` | passed after bead closure |
| `br ready --json` | passed after bead closure; next ready bead is `calc-tv5.4` |
| `br dep cycles --json` | passed with `count: 0` |
| `git diff --check` | passed; CRLF normalization warnings only |

## 9. Pre-Closure Verification Checklist

| # | Check | Result |
|---|---|---|
| 1 | Spec text and realization notes updated for all in-scope items? | yes; this packet, W040 workset/status surfaces, feature map, Lean file, runner profile, and machine-readable artifacts record the slice |
| 2 | Pack expectations updated for affected packs? | yes; pack/C5 remains unpromoted and `calc-tv5.9` owns reassessment |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes; this bead emits deterministic formal-assurance artifacts and cites the W040 TreeCalc post-edit evidence |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes; Section 7 states no runtime strategy changed |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes; W073 typed-only formatting is retained as a guard and no handoff trigger exists |
| 6 | All required tests pass? | yes for the current target; see Section 8 |
| 7 | No known semantic gaps remain in declared scope? | yes for this classification target; broader Rust totality/refinement gaps remain exact blockers with owner lanes |
| 8 | Completion language audit passed? | yes; no whole-engine Rust totality, optimized/core verification, Stage 2 policy, pack-grade replay, C5, release-grade verification, broad OxFml, callable metadata, or general OxFunc promotion is claimed |
| 9 | `WORKSET_REGISTER.md` updated when ordered workset truth changed? | not applicable; ordered workset truth is unchanged |
| 10 | `IN_PROGRESS_FEATURE_WORKLIST.md` updated when feature-map truth changed? | yes; IP-04 records the W040 Rust totality/refinement state |
| 11 | execution-state blocker surface updated? | yes; `.beads/` records `calc-tv5.3` closure and `calc-tv5.4` readiness |

## 10. Completion Claim Self-Audit

| Step | Audit result |
|---|---|
| Scope re-read | pass; `calc-tv5.3` asks for W040 Rust totality and refinement proof tranche |
| Gate criteria re-read | pass; Rust carrier evidence, TreeCalc refinement evidence, totality boundaries, accepted seams, exact blockers, and promotion predicates have deterministic evidence |
| Silent scope reduction check | pass; whole-engine Rust totality, panic-free core domain, automatic dynamic transition detection, snapshot/capability counterparts, callable metadata projection, Stage 2, pack/C5, release-grade verification, broad OxFml, and general OxFunc kernels remain unpromoted |
| "Looks done but is not" pattern check | pass; checked Lean classification, marker census, and explicit-seed TreeCalc evidence are not represented as full Rust-engine proof |
| Result | pass for the `calc-tv5.3` target after final bead closure validation |

## 11. Three-Axis Report

- execution_state: `calc-tv5.3_rust_totality_refinement_classified_no_promotion`
- scope_completeness: `scope_partial`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- open_lanes:
  - `calc-tv5.4` Lean/TLA full-verification discharge tranche is next
  - automatic dynamic dependency-set transition differential remains an exact optimized/core/refinement blocker
  - panic-free whole-engine Rust totality remains an exact totality blocker
  - snapshot-fence and capability-view counterparts remain exact Stage 2/coordinator replay blockers
  - callable metadata projection remains an exact proof/seam blocker
  - Lean/TLA full verification, Stage 2 production policy, operated services, retained history, independent evaluator diversity, OxFml seam breadth, pack/C5, and release-grade decision remain open
