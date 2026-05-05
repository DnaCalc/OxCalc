# W042 Rust Totality Refinement And Core Panic-Boundary Closure

Status: `spec_drafted_with_checked_evidence`

Bead: `calc-czd.3`

Run id: `w042-rust-totality-refinement-core-panic-boundary-001`

## Purpose

This packet strengthens the W042 Rust totality/refinement evidence after `calc-czd.2`.

The narrow result is a checked W042 row model plus a formal-assurance packet that binds W042 optimized/core counterpart evidence, W042 TreeCalc replay artifacts, W041 Rust predecessor evidence, callable value-carrier evidence, and the current closure-obligation map. It records direct evidence for exercised dependency, publication, snapshot/capability, and ordinary `LET`/`LAMBDA` value-carrier paths while preserving exact boundaries for whole-engine panic-free proof, broader dynamic-transition coverage, callable metadata projection, and full optimized/core release-grade conformance.

This packet does not promote Rust-engine totality, Rust refinement, full optimized/core verification, full Lean/TLA verification, Stage 2 policy, pack-grade replay, C5, callable metadata projection, callable carrier sufficiency, release-grade verification, or general OxFunc kernels.

## Evidence Surfaces

| Artifact | Purpose |
|---|---|
| `formal/lean/OxCalc/CoreEngine/W042RustTotalityAndRefinement.lean` | checked Lean row model for W042 Rust totality/refinement classification |
| `docs/test-runs/core-engine/formal-assurance/w042-rust-totality-refinement-core-panic-boundary-001/run_summary.json` | records row counts, register paths, promotion guards, and no-failure summary |
| `docs/test-runs/core-engine/formal-assurance/w042-rust-totality-refinement-core-panic-boundary-001/w042_rust_totality_refinement_ledger.json` | records the 13 W042 Rust rows |
| `docs/test-runs/core-engine/formal-assurance/w042-rust-totality-refinement-core-panic-boundary-001/w042_rust_totality_boundary_register.json` | records 4 totality-boundary rows |
| `docs/test-runs/core-engine/formal-assurance/w042-rust-totality-refinement-core-panic-boundary-001/w042_rust_refinement_register.json` | records 7 refinement rows, including the automatic dynamic transition row |
| `docs/test-runs/core-engine/formal-assurance/w042-rust-totality-refinement-core-panic-boundary-001/w042_rust_exact_blocker_register.json` | records 4 exact remaining blockers |
| `docs/test-runs/core-engine/formal-assurance/w042-rust-totality-refinement-core-panic-boundary-001/source_evidence_index.json` | binds W041, W042.1, W042.2, TreeCalc, W073 intake, and Lean evidence sources |
| `docs/test-runs/core-engine/formal-assurance/w042-rust-totality-refinement-core-panic-boundary-001/validation.json` | records validation status and zero validation failures |

## Row Disposition

| Row | Disposition |
|---|---|
| result/error carrier | direct totality evidence |
| W042 TreeCalc counterpart packet | direct totality evidence |
| explicit dependency seed rebind regression | direct refinement evidence |
| automatic dynamic resolved-to-potential transition | direct refinement evidence for the exercised pattern |
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

1. 13 Rust/proof rows.
2. 10 local checked-proof classification rows.
3. 0 bounded-model rows.
4. 1 accepted external seam.
5. 2 accepted boundaries.
6. 4 totality-boundary rows.
7. 7 refinement rows.
8. 1 automatic dynamic transition refinement row.
9. 4 exact remaining blockers.
10. 0 failed rows.

## Dynamic And Callable Classification

The W042.3 formal-assurance runner binds the W042.2 TreeCalc run and checks:

1. TreeCalc emitted 26 cases with 0 expectation mismatches.
2. The automatic transition row derives both `DependencyRemoved` and `DependencyReclassified`.
3. The post-edit invalidation closure marks node 3 as `requires_rebind: true`.
4. The post-edit rerun rejects with `HostInjectedFailure` before a new publication.
5. The higher-order `LET`/`LAMBDA` case publishes ordinary value `17` through the current candidate/publication value carriers.

The runner now accepts the current W042 artifact shapes:

1. W042 closure obligations may use `id` rather than the older `obligation_id` field name.
2. TreeCalc publication evidence may be observed through `candidate_result.value_updates` and `publication_bundle.published_view_delta`, not only an inline `published_values` array.

Those reader changes are artifact-schema compatibility corrections in the formal-assurance tooling. They do not change TreeCalc or optimized/core runtime behavior.

## Remaining Exact Blockers

| Blocker | Owner |
|---|---|
| `w042_runtime_panic_surface_totality_boundary` | `calc-czd.3`; successor audit in `calc-czd.10` |
| `w042_broader_dynamic_transition_coverage_refinement_boundary` | `calc-czd.4`; `calc-czd.5` |
| `w042_callable_metadata_projection_totality_boundary` | `calc-czd.4`; `calc-czd.8`; external OxFunc semantic authority remains outside OxCalc |
| `w042_full_optimized_core_release_grade_conformance_boundary` | `calc-czd.10` |

The Rust panic-surface audit currently observes 146 panic-family markers across 12 audited core files. That count is a guardrail, not a semantic proof. Panic-free whole-engine totality remains unpromoted.

## OxFml Formatting Intake

The latest OxFml W073 formatting change was reviewed for this packet.

Current OxCalc impact:

1. W042.3 does not construct conditional-formatting requests and does not need a core Rust code change for W073.
2. W042.3 source evidence indexes the W042.2 `w073_formatting_intake.json` guard.
3. W073 remains typed-only for `colorScale`, `dataBar`, `iconSet`, `top`, `bottom`, `aboveAverage`, and `belowAverage`.
4. W072 bounded `thresholds` strings are not fallback metadata for those aggregate and visualization families.
5. The fuller W073 breadth, public migration, and request-construction uptake remain owned by `calc-czd.8`.

## Semantic-Equivalence Statement

This packet changes formal-assurance artifact reading, proof/model classification, formal-assurance runner output, a checked Lean classification file, and documentation only. It does not change coordinator scheduling, invalidation strategy, dependency graph construction, soft-reference update behavior, recalc behavior, publication semantics, reject policy, overlay lifecycle behavior, TraceCalc semantics, TreeCalc/CoreEngine runtime behavior, OxFml runtime behavior, or OxFunc callable kernels.

Observable engine results are invariant under this packet. The only changed observable artifacts are W042.3 formal-assurance evidence files and the checked Lean classification file.

## Validation

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | passed |
| `lean formal/lean/OxCalc/CoreEngine/W042RustTotalityAndRefinement.lean` | passed |
| `cargo test -p oxcalc-tracecalc formal_assurance_runner_classifies_w042_rust_totality_and_refinement -- --nocapture` | passed |
| `cargo run -p oxcalc-tracecalc-cli -- formal-assurance w042-rust-totality-refinement-core-panic-boundary-001` | passed; emitted 13 rows with 0 failed rows |

## Status Report

- execution_state: `calc-czd.3_rust_totality_refinement_core_panic_boundary_validated`
- scope_completeness: `scope_partial`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- open_lanes:
  - `calc-czd.4` Lean/TLA fairness and full-verification expansion
  - broader dynamic dependency-transition coverage remains partial
  - callable metadata projection remains an exact blocker
  - runtime panic-surface proof remains an exact blocker
  - W073 typed-only conditional-formatting request-construction uptake remains a later OxFml/public migration lane
  - full optimized/core verification, Rust-engine totality, Rust refinement, release-grade verification, C5, pack-grade replay, Stage 2 policy, operated services, independent evaluator breadth, mismatch quarantine, broad OxFml display/publication, public migration, callable carrier sufficiency, registered-external callable projection, provider-failure/callable-publication semantics, and general OxFunc kernels remain unpromoted

## Pre-Closure Verification Checklist

| # | Check | Result |
|---|---|---|
| 1 | Spec text and realization notes updated for all in-scope items? | yes; this packet, W042 README/status surfaces, feature map, Lean row model, and formal-assurance artifacts record the slice |
| 2 | Pack expectations updated for affected packs? | yes; pack/C5 remains unpromoted and `calc-czd.9` owns reassessment |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes; W042.2 TreeCalc replay and W042.3 formal-assurance artifacts are bound |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes; no runtime strategy changed |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes; current OxFml W073 update is carried as a typed-only guard and no OxCalc handoff trigger exists |
| 6 | All required tests pass? | yes; see Validation |
| 7 | No known semantic gaps remain in declared scope? | yes for this W042.3 classification target; broader exact blockers are explicit |
| 8 | Completion language audit passed? | yes; no release-grade, Rust totality, Rust refinement, optimized/core, Lean/TLA, Stage 2, pack/C5, service, independent-diversity, broad OxFml, callable metadata, callable carrier, registered-external, provider-publication, or general OxFunc promotion is claimed |
| 9 | `WORKSET_REGISTER.md` updated when ordered workset truth changed? | yes; no ordered workset-truth change in this bead |
| 10 | `IN_PROGRESS_FEATURE_WORKLIST.md` updated when feature-map truth changed? | yes; IP-04 records the W042 Rust update |
| 11 | execution-state blocker surface updated? | yes; `.beads/` records `calc-czd.3` state |

## Completion Claim Self-Audit

| Step | Audit result |
|---|---|
| Scope re-read | pass; `calc-czd.3` asks for Rust totality/refinement and core panic-boundary evidence |
| Gate criteria re-read | pass; each totality/refinement claim has direct evidence, checked proof/model classification, or exact blockers |
| Silent scope reduction check | pass; broader Rust totality/refinement, full optimized/core, release-grade, Stage 2, pack/C5, service, OxFml breadth, callable metadata, and general OxFunc lanes remain explicit open lanes |
| "Looks done but is not" pattern check | pass; artifact-reader fixes and checked classification are not reported as whole-engine proof or runtime implementation |
| Result | pass for the `calc-czd.3` target |
