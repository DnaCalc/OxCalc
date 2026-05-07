# W041 Rust Totality Refinement And Panic Boundary Discharge

Status: `spec_drafted_with_checked_evidence`

Bead: `calc-sui.3`

Run id: `w041-rust-totality-refinement-proof-tranche-001`

## Purpose

This packet strengthens the W041 Rust totality/refinement evidence after `calc-sui.2`.

The narrow new result is that the automatic dynamic dependency transition evidenced by W041.2 is now classified as direct Rust refinement evidence for the exercised resolved-to-potential pattern. W040 carried the automatic transition as an exact blocker. W041.3 moves that one exercised pattern into direct evidence while retaining whole-engine Rust totality, panic-free core-domain proof, snapshot/capability refinement, and callable metadata projection as explicit boundaries or blockers.

This packet does not promote Rust-engine totality, Rust refinement, full optimized/core verification, full Lean/TLA verification, Stage 2 policy, pack-grade replay, C5, callable metadata projection, callable carrier sufficiency, or general OxFunc kernels.

## Evidence Surfaces

| Artifact | Purpose |
| --- | --- |
| `formal/lean/OxCalc/CoreEngine/W041RustTotalityAndRefinement.lean` | checked Lean row model for W041 Rust totality/refinement classification |
| `archive/test-runs-core-engine-w038-w045/formal-assurance/w041-rust-totality-refinement-proof-tranche-001/run_summary.json` | records row counts, register paths, promotion guards, and no-failure summary |
| `archive/test-runs-core-engine-w038-w045/formal-assurance/w041-rust-totality-refinement-proof-tranche-001/w041_rust_totality_refinement_ledger.json` | records the 10 W041 Rust rows |
| `archive/test-runs-core-engine-w038-w045/formal-assurance/w041-rust-totality-refinement-proof-tranche-001/w041_rust_totality_boundary_register.json` | records 4 totality-boundary rows |
| `archive/test-runs-core-engine-w038-w045/formal-assurance/w041-rust-totality-refinement-proof-tranche-001/w041_rust_refinement_register.json` | records 5 refinement rows, including the automatic dynamic transition row |
| `archive/test-runs-core-engine-w038-w045/formal-assurance/w041-rust-totality-refinement-proof-tranche-001/w041_rust_exact_blocker_register.json` | records 4 exact remaining blockers |
| `archive/test-runs-core-engine-w038-w045/formal-assurance/w041-rust-totality-refinement-proof-tranche-001/source_evidence_index.json` | binds predecessor W040, W041.2, TreeCalc, and Lean evidence sources |
| `archive/test-runs-core-engine-w038-w045/formal-assurance/w041-rust-totality-refinement-proof-tranche-001/validation.json` | records validation status and zero validation failures |

## Row Disposition

| Row | Disposition |
| --- | --- |
| result/error carrier | direct totality evidence |
| successor formula-catalog fixture path | direct totality evidence |
| explicit dependency seed rebind regression | direct refinement evidence |
| automatic dynamic resolved-to-potential transition | direct refinement evidence for the exercised pattern |
| runtime panic surface | exact totality boundary and blocker |
| snapshot-fence counterpart | exact refinement blocker |
| capability-view fence counterpart | exact refinement blocker |
| callable metadata projection | exact totality/refinement blocker |
| LET/LAMBDA carrier seam | accepted external seam boundary |
| spec-evolution refinement guard | accepted boundary |

Observed counts:

1. 10 Rust/proof rows.
2. 7 local checked-proof classification rows.
3. 0 bounded-model rows.
4. 1 accepted external seam.
5. 2 accepted boundaries.
6. 4 totality-boundary rows.
7. 5 refinement rows.
8. 1 automatic dynamic transition refinement row.
9. 4 exact remaining blockers.
10. 0 failed rows.

## Dynamic Transition Classification

W041.2 generated direct TreeCalc evidence for the successor formula-catalog path:

1. `archive/test-runs-core-engine-w038-w045/treecalc-local/w041-optimized-core-automatic-dynamic-transition-001/run_summary.json`
2. `archive/test-runs-core-engine-w038-w045/treecalc-local/w041-optimized-core-automatic-dynamic-transition-001/cases/tc_local_dynamic_release_reclassification_auto_post_edit_001/post_edit/invalidation_seeds.json`
3. `archive/test-runs-core-engine-w038-w045/treecalc-local/w041-optimized-core-automatic-dynamic-transition-001/cases/tc_local_dynamic_release_reclassification_auto_post_edit_001/post_edit/invalidation_closure.json`
4. `archive/test-runs-core-engine-w038-w045/treecalc-local/w041-optimized-core-automatic-dynamic-transition-001/cases/tc_local_dynamic_release_reclassification_auto_post_edit_001/post_edit/result.json`

The W041.3 formal-assurance runner binds those artifacts and checks:

1. TreeCalc emitted 26 cases with 0 expectation mismatches.
2. The automatic transition row derives both `DependencyRemoved` and `DependencyReclassified`.
3. The post-edit invalidation closure marks node 3 as `requires_rebind: true`.
4. The post-edit rerun rejects with `HostInjectedFailure` before a new publication.

That moves the automatic dynamic transition from the W040 exact-blocker position into W041 direct refinement evidence for this exercised pattern only.

## Remaining Exact Blockers

| Blocker | Owner |
| --- | --- |
| `w041_runtime_panic_surface_totality_boundary` | `calc-sui.3`; successor audit in `calc-sui.10` |
| `w041_snapshot_fence_refinement_boundary` | `calc-sui.5` |
| `w041_capability_view_fence_refinement_boundary` | `calc-sui.5` |
| `w041_callable_metadata_projection_totality_boundary` | `calc-sui.8`; external OxFunc semantic authority remains outside OxCalc |

The Rust panic-surface audit currently observes 146 panic-family markers across 12 audited core files. That count is a guardrail, not a semantic proof. Panic-free whole-engine totality remains unpromoted.

## OxFml Formatting Intake

The latest OxFml W073 formatting change was inspected during this packet.

Current OxCalc impact:

1. The current upstream-host W073 guard already emits `VerificationConditionalFormattingRule.typed_rule` for the exercised top-rank row.
2. `cargo test -p oxcalc-core upstream_host -- --nocapture` passes against the updated OxFml worktree.
3. No W041.3 Rust totality/refinement code change is required for W073.
4. The fuller W073 breadth remains owned by `calc-sui.8`: aggregate and visualization families must use `typed_rule` for `colorScale`, `dataBar`, `iconSet`, `top`, `bottom`, `aboveAverage`, and `belowAverage`, and W072 bounded `thresholds` strings are not fallback metadata for those families.

## Semantic Equivalence Statement

This packet changes proof/model classification, formal-assurance runner output, and documentation only. It does not change optimized/core recalc behavior, TreeCalc invalidation behavior, publication policy, scheduling policy, OxFml runtime behavior, or OxFunc callable kernels.

Observable engine results are therefore invariant under this packet. The only changed observable artifacts are W041.3 formal-assurance evidence files and the checked Lean classification file.

## Validation

| Command | Result |
| --- | --- |
| `lean formal/lean/OxCalc/CoreEngine/W041RustTotalityAndRefinement.lean` | passed |
| `cargo test -p oxcalc-tracecalc formal_assurance_runner_classifies_w041_rust_totality_and_refinement -- --nocapture` | passed |
| `cargo run -p oxcalc-tracecalc-cli -- formal-assurance w041-rust-totality-refinement-proof-tranche-001` | passed; emitted 10 rows with 0 failed rows |
| `cargo test -p oxcalc-core upstream_host -- --nocapture` | passed against current OxFml formatting worktree |

## Status Report

- execution_state: `calc-sui.3_rust_totality_refinement_classified_no_promotion`
- scope_completeness: `scope_partial`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- open_lanes:
  - `calc-sui.4` Lean/TLA full-verification and fairness discharge
  - `calc-sui.5` snapshot-fence and capability-view fence counterparts
  - `calc-sui.8` W073 breadth, broad OxFml display/publication, callable metadata projection, and callable carrier sufficiency
  - runtime panic-surface proof remains a W041 exact blocker
  - full optimized/core verification, Rust-engine totality, Rust refinement, release-grade verification, C5, pack-grade replay, Stage 2 policy, operated services, independent evaluator breadth, public migration, and general OxFunc kernels remain unpromoted

## Pre-Closure Verification Checklist

| Check | Result |
| --- | --- |
| Workset and bead ids are explicit | yes: `W041`, `calc-sui.3` |
| Required artifacts exist | yes: Lean file and W041.3 formal-assurance packet artifacts are present |
| Direct replay evidence exists for changed behavior | yes: W041.2 TreeCalc replay is bound into the automatic transition row |
| No declared gap is match-promoted | yes: promotion claims remain false and exact blockers are retained |
| Residual blockers are explicit | yes: four exact remaining Rust totality/refinement blockers |
| Semantic equivalence statement is present | yes |

## Completion Claim Self-Audit

| Audit Item | Result |
| --- | --- |
| Claim is limited to `calc-sui.3` target | yes |
| Scaffolding is not reported as implementation | yes |
| Spec text has checked/replay evidence | yes: Lean check, formal-assurance run, and TreeCalc replay links |
| Cross-repo handoff is not treated as closure | yes; W073 breadth remains `calc-sui.8` and OxFml-owned |
| Uncertain lanes default to in-progress | yes; exact blockers and open lanes are retained |
| Strategy-change equivalence statement is present | yes |
