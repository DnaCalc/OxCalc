# W045 Rust Totality Refinement And Panic-Surface Hardening

Status: `calc-zkio.3_rust_totality_refinement_panic_surface_hardening_validated`
Workset: `W045`
Parent epic: `calc-zkio`
Bead: `calc-zkio.3`

## 1. Purpose

This packet deepens the W045 Rust totality/refinement and panic-surface frontier after `calc-zkio.2`.

The W045.3 runner consumes the W045 residual obligation map, W045 optimized/core conformance packet, W044 Rust formal-assurance packet, and current OxFml formatting intake. It emits a fresh W045 Rust row model and formal-assurance packet that carries valid predecessor evidence, binds W045.2 optimized/core classifications into the Rust proof surface, and makes the remaining panic-surface, dynamic, soft-reference/`INDIRECT`, counterpart, callable-metadata, and release-grade blockers exact.

This narrows the assurance surface. It does not promote Rust-engine totality, Rust refinement, panic-free core-domain proof, full optimized/core verification, full Lean/TLA verification, Stage 2 production policy, pack-grade replay, C5, callable metadata projection, callable carrier sufficiency, release-grade verification, broad OxFml closure, W073 downstream uptake, registered-external/provider callable publication, or general OxFunc kernels.

## 2. Authority Inputs Reviewed

| Input | Role |
|---|---|
| `docs/worksets/W045_CORE_FORMALIZATION_RELEASE_GRADE_SERVICE_AND_CROSS_REPO_UPTAKE_VERIFICATION.md` | W045 scope and `calc-zkio.3` gate |
| `docs/spec/core-engine/w045-formalization/W045_RESIDUAL_RELEASE_GRADE_SUCCESSOR_OBLIGATION_AND_CURRENT_OXFML_INTAKE_MAP.md` | W045 obligations, promotion contracts, W073/public-surface intake, and no-promotion claims |
| `docs/test-runs/core-engine/release-grade-ledger/w045-residual-release-grade-successor-obligation-current-oxfml-intake-map-001/` | machine-readable W045.1 successor map, contracts, and OxFml intake |
| `docs/spec/core-engine/w045-formalization/W045_OPTIMIZED_CORE_COUNTERPART_COVERAGE_AND_CALLABLE_METADATA_PROJECTION_CLOSURE.md` | W045.2 optimized/core and callable metadata tranche |
| `docs/test-runs/core-engine/implementation-conformance/w045-optimized-core-counterpart-callable-metadata-001/` | W045.2 disposition, dynamic, counterpart, callable, blocker, match-guard, and validation artifacts |
| `docs/test-runs/core-engine/formal-assurance/w044-rust-totality-refinement-panic-surface-expansion-001/` | predecessor W044 Rust totality/refinement packet |
| `formal/lean/OxCalc/CoreEngine/W044RustTotalityAndRefinement.lean` | predecessor checked Lean row model |
| `../OxFml/docs/worksets/W073_conditional_formatting_typed_visualization_payload.md` | current W073 typed-only formatting guard |
| `../OxFml/docs/handoffs/HANDOFF-DNAONECALC-012_W073_TYPED_CF_PAYLOAD_FIRST_SLICE.md` | downstream typed-rule request-construction handoff |

## 3. Artifact Surface

Run id: `w045-rust-totality-refinement-panic-surface-hardening-001`

| Artifact | Result |
|---|---|
| `formal/lean/OxCalc/CoreEngine/W045RustTotalityAndRefinement.lean` | checked Lean row model for W045 Rust totality/refinement classification |
| `docs/test-runs/core-engine/formal-assurance/w045-rust-totality-refinement-panic-surface-hardening-001/run_summary.json` | 17 rows, 11 local checked-proof classification rows, 0 bounded-model rows, 1 accepted external seam, 4 accepted boundaries, 5 totality boundaries, 9 refinement rows, 7 exact blockers, 1 panic-surface row, 0 failed rows |
| `w045_rust_totality_refinement_ledger.json` | machine-readable 17-row Rust totality/refinement ledger |
| `w045_rust_totality_boundary_register.json` | 5 totality-boundary rows |
| `w045_rust_refinement_register.json` | 9 refinement rows, including 1 automatic mixed dynamic-transition bridge row |
| `w045_rust_exact_blocker_register.json` | 7 exact remaining blockers |
| `w045_panic_surface_register.json` | 1 panic-surface row and current panic-family marker census |
| `source_evidence_index.json` | W045.1, W045.2, W044 Rust, OxFml-intake, match-guard, and Lean source index |
| `validation.json` | validation status `formal_assurance_w045_rust_totality_refinement_valid` |

## 4. Implementation Delta

Changed files:

1. `formal/lean/OxCalc/CoreEngine/W045RustTotalityAndRefinement.lean`
2. `src/oxcalc-tracecalc/src/formal_assurance.rs`

The formal-assurance runner now has a W045 Rust profile that:

1. reads W045 residual successor obligations, promotion contracts, and OxFml intake,
2. reads W045.2 implementation-conformance, dynamic-transition, counterpart, callable, exact-blocker, and match-guard artifacts,
3. reads W044 Rust formal-assurance ledger, validation, refinement, and blocker artifacts,
4. counts current audited panic-family markers across the core Rust audit surface,
5. emits W045 Rust totality/refinement ledger, totality-boundary register, refinement register, blocker register, panic-surface register, source evidence index, validation, and run summary artifacts,
6. validates 17 rows, 1 automatic mixed dynamic-transition bridge row, 1 panic-surface row, 7 exact blockers, and 0 failed rows.

No runtime coordinator, dependency, invalidation, recalc, publication, OxFml, OxFunc, Stage 2, pack, or service behavior is changed.

## 5. Row Disposition

| Row | Disposition |
|---|---|
| result/error carrier | direct totality evidence |
| W044 Rust predecessor packet | carried Rust regression evidence |
| W045 optimized/core packet | direct typed-artifact totality bridge for current exercised paths |
| mixed dynamic transition | carried direct refinement evidence for the exercised mixed add/remove/reclassify pattern |
| publication no-publish fence | carried refinement evidence for the exercised reject/no-publication path |
| `LET`/`LAMBDA` ordinary value carrier | carried callable value-carrier totality evidence |
| runtime panic surface | exact totality boundary and panic-surface blocker |
| broader dynamic transition coverage | exact refinement blocker |
| soft-reference/`INDIRECT` late reference-resolution breadth | exact refinement blocker |
| snapshot-fence counterpart breadth | exact refinement blocker |
| capability-view counterpart breadth | exact refinement blocker |
| callable metadata projection | exact totality/refinement blocker |
| full optimized/core release-grade conformance | exact release-grade boundary |
| `LET`/`LAMBDA` carrier seam | accepted external seam boundary |
| spec-evolution refinement guard | accepted boundary |
| W073 typed formatting guard | accepted formatting boundary |
| no-proxy match-promotion guard | accepted boundary |

Observed counts:

1. 17 Rust/proof rows.
2. 11 local checked-proof classification rows.
3. 0 bounded-model rows.
4. 1 accepted external seam.
5. 4 accepted boundaries.
6. 5 totality-boundary rows.
7. 9 refinement rows.
8. 1 automatic mixed dynamic-transition bridge row.
9. 1 panic-surface row.
10. 7 exact remaining blockers.
11. 0 failed rows.

## 6. Remaining Exact Blockers

| Blocker | Owner |
|---|---|
| `w045_runtime_panic_surface_totality_boundary` | `calc-zkio.3`; successor reassessment in `calc-zkio.10` and `calc-zkio.11` |
| `w045_broader_dynamic_transition_refinement_boundary` | `calc-zkio.3`; `calc-zkio.4`; `calc-zkio.5`; `calc-zkio.10` |
| `w045_soft_reference_indirect_resolution_refinement_boundary` | `calc-zkio.2`; `calc-zkio.3`; successor fixture/proof work |
| `w045_snapshot_fence_counterpart_refinement_boundary` | `calc-zkio.3`; `calc-zkio.5` |
| `w045_capability_view_counterpart_refinement_boundary` | `calc-zkio.3`; `calc-zkio.5` |
| `w045_callable_metadata_projection_totality_boundary` | `calc-zkio.3`; `calc-zkio.8`; external OxFunc semantic authority remains outside OxCalc |
| `w045_full_optimized_core_release_grade_conformance_boundary` | `calc-zkio.10`; `calc-zkio.11` |

The Rust panic-surface audit observes 158 panic-family markers across 12 audited core files. That count is a guardrail, not a semantic proof. Panic-free whole-engine totality remains unpromoted.

## 7. OxFml Formatting Intake

The W045.3 packet carries the current OxFml formatting intake from W045.1 and W045.2:

1. `VerificationConditionalFormattingRule.typed_rule` is the only accepted metadata source for `colorScale`, `dataBar`, `iconSet`, `top`, `bottom`, `aboveAverage`, and `belowAverage`.
2. W072 bounded `thresholds` strings are intentionally ignored for those aggregate and visualization families.
3. `thresholds` remains meaningful only for scalar/operator/expression rules.
4. DNA OneCalc downstream typed-rule request construction is required but remains unverified by OxCalc.
5. W045.3 does not construct conditional-formatting requests and does not require an OxCalc core-engine code change.
6. No OxFml handoff is required by this bead.

## 8. Semantic-Equivalence Statement

This packet adds a checked Lean classification file, a W045 formal-assurance runner profile, emitted formal-assurance artifacts, and documentation.

It does not change coordinator scheduling, invalidation strategy, dependency graph construction, soft-reference update behavior, recalc behavior, publication semantics, reject policy, overlay lifecycle behavior, TraceCalc semantics, TreeCalc/CoreEngine runtime behavior, OxFml runtime behavior, OxFunc callable kernels, Stage 2 scheduler policy, pack/C5 capability policy, operated-service behavior, alert dispatch behavior, retained-history behavior, retained-witness behavior, or release-scale behavior.

Observable engine results are invariant under this packet. The only changed observable artifacts are W045.3 formal-assurance evidence files and the checked Lean classification file.

## 9. Verification

| Command | Result |
|---|---|
| `lean formal/lean/OxCalc/CoreEngine/W045RustTotalityAndRefinement.lean` | passed |
| `cargo test -p oxcalc-tracecalc formal_assurance_runner_classifies_w045_rust_totality_and_refinement -- --nocapture` | passed |
| `cargo run -p oxcalc-tracecalc-cli -- formal-assurance w045-rust-totality-refinement-panic-surface-hardening-001` | passed; emitted 17 rows with 0 failed rows |
| JSON parse for W045.3 formal-assurance artifacts | passed |
| `cargo test -p oxcalc-tracecalc formal_assurance -- --nocapture` | passed; 13 tests |
| `cargo fmt --all -- --check` | passed |
| `scripts/check-worksets.ps1` | passed pre-close; `worksets=23; beads total=187; open=9; in_progress=1; ready=0; blocked=8; deferred=0; closed=177` |
| `br ready --json` | passed pre-close; no ready beads while `calc-zkio.3` is in progress |
| `br dep cycles --json` | passed pre-close; `count=0` |
| `git diff --check` | passed pre-close; CRLF normalization warnings only |
| JSON parse for W045.3 formal-assurance artifacts | passed post-close |
| `scripts/check-worksets.ps1` | passed post-close; `worksets=23; beads total=187; open=9; in_progress=0; ready=1; blocked=7; deferred=0; closed=178` |
| `br ready --json` | passed post-close; next ready bead is `calc-zkio.4` |
| `br dep cycles --json` | passed post-close; `count=0` |
| `git diff --check` | passed post-close; CRLF normalization warnings only |

## 10. Pre-Closure Verification Checklist

| # | Check | Result |
|---|---|---|
| 1 | Spec text and realization notes updated for all in-scope items? | yes; this packet, W045 README/status surfaces, feature map, Lean file, runner profile, and machine-readable artifacts record the slice |
| 2 | Pack expectations updated for affected packs? | yes; pack/C5 remains unpromoted and `calc-zkio.10` owns reassessment |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes; W045.2 implementation-conformance evidence and W045.3 formal-assurance artifacts are bound |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes; no runtime strategy changed |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes; current W073 typed-only formatting intake is carried and no handoff trigger exists |
| 6 | All required tests pass? | yes for this target; see Section 9 |
| 7 | No known semantic gaps remain in declared scope? | yes for this W045.3 classification target; broader exact blockers are explicit |
| 8 | Completion language audit passed? | yes; no Rust-engine totality, panic-free core-domain proof, Rust refinement, optimized/core, Lean/TLA, Stage 2, pack/C5, service, independent-diversity, broad OxFml, callable metadata, callable carrier, registered-external, provider-publication, or general OxFunc promotion is claimed |
| 9 | `WORKSET_REGISTER.md` updated when ordered workset truth changed? | not applicable; ordered workset truth is unchanged |
| 10 | `IN_PROGRESS_FEATURE_WORKLIST.md` updated when feature-map truth changed? | yes |
| 11 | execution-state blocker surface updated? | yes; `.beads/` records `calc-zkio.3` closed and `calc-zkio.4` ready |

## 11. Completion Claim Self-Audit

| Step | Audit result |
|---|---|
| Scope re-read | pass; `calc-zkio.3` asks for Rust totality/refinement and panic-surface hardening |
| Gate criteria re-read | pass; each totality/refinement claim has direct evidence, checked proof/model classification, or exact blockers |
| Silent scope reduction check | pass; broader Rust totality/refinement, panic-free proof, soft-reference/`INDIRECT`, full optimized/core, release-grade, Stage 2, pack/C5, service, OxFml breadth, callable metadata, callable carrier sufficiency, and general OxFunc lanes remain explicit open lanes |
| "Looks done but is not" pattern check | pass; checked Lean classification, marker census, W045.2 conformance bridge, and one mixed dynamic-transition bridge row are not reported as whole-engine proof or runtime implementation |
| Result | pass for the `calc-zkio.3` target after final post-close validation |

## 12. Three-Axis Report

- execution_state: `calc-zkio.3_rust_totality_refinement_panic_surface_hardening_validated`
- scope_completeness: `scope_partial`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- open_lanes:
  - `calc-zkio.4` Lean/TLA verification, fairness, and totality discharge is next
  - broader dynamic dependency-transition coverage remains partial beyond fixture-scale mixed add/remove/reclassify and predecessor addition/release patterns
  - soft-reference/`INDIRECT` and late reference-resolution breadth remains an exact blocker
  - snapshot-fence counterpart breadth remains an exact blocker
  - capability-view counterpart breadth remains an exact blocker
  - callable metadata projection remains an exact blocker
  - callable carrier sufficiency proof remains blocked
  - runtime panic-surface proof remains an exact blocker
  - full optimized/core verification, Rust-engine totality, Rust refinement, release-grade verification, C5, pack-grade replay, Stage 2 policy, operated services, independent evaluator breadth, mismatch quarantine, broad OxFml display/publication, public migration, W073 downstream typed-rule uptake, registered-external callable projection, provider-failure/callable-publication semantics, continuous scale assurance, and general OxFunc kernels remain unpromoted
