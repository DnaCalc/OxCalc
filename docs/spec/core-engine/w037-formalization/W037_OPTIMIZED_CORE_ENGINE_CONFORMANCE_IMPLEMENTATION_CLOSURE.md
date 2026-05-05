# W037 Optimized/Core-Engine Conformance Implementation Closure

Status: `calc-ubd.3_optimized_core_conformance_decisions_validated`
Workset: `W037`
Parent epic: `calc-ubd`
Bead: `calc-ubd.3`

## 1. Purpose

This packet records the W037 pass over the six W036 optimized/core-engine conformance action rows.

The target is not full optimized/core-engine verification. The target is to convert each W036 declared gap action into one of:

1. implementation fix with direct TreeCalc/CoreEngine evidence,
2. replay/differential match promotion with a guard,
3. spec-evolution or authority deferral,
4. explicit residual blocker with owner lane.

W037 promotes one row and keeps five residual blockers exact.

## 2. Authority Inputs Reviewed

| Input | Role |
|---|---|
| `docs/spec/core-engine/w036-formalization/W036_OPTIMIZED_CORE_ENGINE_CONFORMANCE_CLOSURE_PLAN_AND_FIRST_FIXES.md` | W036 source action plan |
| `docs/test-runs/core-engine/implementation-conformance/w036-implementation-conformance-closure-001/w036_closure_action_register.json` | source action register with six W036 rows |
| `docs/test-runs/core-engine/tracecalc-reference-machine/w037-tracecalc-observable-closure-001/oracle-matrix/coverage_matrix.json` | TraceCalc replay/oracle evidence for the W037 rows |
| `docs/test-runs/core-engine/treecalc-local/w037-optimized-core-conformance-treecalc-001/` | new TreeCalc/CoreEngine evidence run |
| `docs/test-runs/core-engine/implementation-conformance/w037-implementation-conformance-closure-001/` | W037 decision, blocker, guard, evidence, and validation artifacts |
| `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md` | current OxFml seam-watch and formatting/display intake |

## 3. Implementation Changes

W037 adds a narrow positive dynamic-dependency carrier to the local TreeCalc formula model:

1. `TreeReference::DynamicResolved` carries a resolved dynamic-potential target, carrier id, and detail.
2. The TreeCalc/OxFml translation lowers it through the ordinary reference value path while preserving `DependencyDescriptorKind::DynamicPotential`.
3. Publication now carries direct evidence for the dynamic bind update:
   - `runtime_effect.dynamic_reference` with `DynamicDependency` family,
   - `AcceptedCandidateResult.dependency_shape_updates`,
   - publication carriage classification with `dependency_shape_update_count: 1`.

The fixture `tc_local_dynamic_resolved_publish_001` proves the positive dynamic bind row without expanding the unresolved dynamic negative, release, or dependency-reclassification surfaces.

## 4. Deterministic Evidence

| Artifact | Result |
|---|---|
| `docs/test-runs/core-engine/treecalc-local/w037-optimized-core-conformance-treecalc-001/run_summary.json` | 24 cases, 0 expectation mismatches |
| `docs/test-runs/core-engine/treecalc-local/w037-optimized-core-conformance-treecalc-001/cases/tc_local_dynamic_resolved_publish_001/result.json` | published, value `7`, 1 dynamic runtime effect, 1 dependency shape update |
| `docs/test-runs/core-engine/implementation-conformance/w037-implementation-conformance-closure-001/run_summary.json` | 6 decision rows, 1 fixed/promoted, 5 residual blockers, 1 match-promoted, 0 failed |
| `docs/test-runs/core-engine/implementation-conformance/w037-implementation-conformance-closure-001/validation.json` | `implementation_conformance_w037_decisions_valid`, no validation failures |
| `docs/test-runs/core-engine/implementation-conformance/w037-implementation-conformance-closure-001/w037_match_promotion_guard.json` | one allowed promoted row and five non-promoted rows |

## 5. Decision Summary

| Row | W037 decision |
|---|---|
| `w037_decision_dynamic_dependency_bind_projection_fixed` | promoted from W036 first-fix state to direct TreeCalc differential evidence |
| `w037_decision_dynamic_negative_release_residual_blocker` | residual blocker; positive dynamic bind evidence does not prove negative/release/reclassification |
| `w037_decision_lambda_host_effect_residual_blocker` | residual blocker for direct OxFml evaluator and `LET`/`LAMBDA` seam evidence |
| `w037_decision_snapshot_fence_projection_residual_blocker` | residual blocker for Stage 2/coordinator replay evidence |
| `w037_decision_capability_view_fence_projection_residual_blocker` | residual blocker for Stage 2/coordinator replay evidence |
| `w037_decision_callable_metadata_projection_residual_blocker` | residual blocker for callable carrier proof/model inventory |

## 6. OxFml Formatting Intake

The current OxFml formatting update is incorporated as watch/input-contract evidence:

1. W073 aggregate and visualization conditional-formatting metadata remains `typed_rule`-only.
2. bounded `thresholds` strings remain intentionally ignored for W073 aggregate and visualization families.
3. `format_delta` and `display_delta` remain distinct categories.
4. broader display-facing closure is not inferred from semantic-format evidence.
5. no artifact in this bead constructs conditional-formatting request payloads or display-facing publication payloads, so no OxFml handoff is triggered.

## 7. Semantic-Equivalence Statement

The new `DynamicResolved` path affects only formulas that explicitly use that new carrier. Existing TreeCalc formulas, static references, relative references, residual dynamic-potential rejects, host-sensitive rejects, capability-sensitive rejects, shape/topology rejects, verified-clean paths, and publication paths without resolved dynamic descriptors retain their observable results.

For formulas with `DynamicResolved`, observable value evaluation is equivalent to the same resolved reference evaluated through the existing OxFml-backed value path, with additional dynamic-dependency metadata sidecars. The sidecars do not alter `value_delta`; they make the dependency-shape and dynamic-effect consequences replay-visible.

## 8. Verification

| Command | Result |
|---|---|
| `cargo fmt --all` | passed |
| `cargo test -p oxcalc-core treecalc_fixture::tests::checked_in_treecalc_fixtures_execute_against_local_runtime` | passed |
| `cargo test -p oxcalc-core treecalc::tests::local_treecalc_engine_publishes_resolved_dynamic_reference_shape_update` | passed |
| `cargo test -p oxcalc-core formula::tests::formula_catalog_lowers_resolved_dynamic_reference_as_dynamic_edge` | passed |
| `cargo run -p oxcalc-tracecalc-cli -- treecalc w037-optimized-core-conformance-treecalc-001` | passed; 24 TreeCalc cases emitted |
| `cargo run -p oxcalc-tracecalc-cli -- implementation-conformance w037-implementation-conformance-closure-001` | passed; 6 W037 decisions emitted |
| `cargo test -p oxcalc-tracecalc implementation_conformance` | passed |
| `cargo test -p oxcalc-core treecalc_runner::tests::treecalc_runner_emits_local_run_artifacts` | passed on sequential rerun after an initial parallel-test artifact-root race |
| `cargo test -p oxcalc-core treecalc` | passed |
| `cargo test -p oxcalc-core` | passed; 51 unit tests, 5 integration tests |
| `cargo test -p oxcalc-tracecalc-cli` | passed; 0 tests |
| `cargo fmt --all -- --check` | passed |
| `scripts/check-worksets.ps1` | passed after bead closure; `worksets=15`, `beads total=99`, `open=7`, `in_progress=0`, `ready=1`, `blocked=5`, `closed=92` |
| `br dep cycles --json` | passed with `count: 0` |
| `git diff --check` | passed; CRLF normalization warnings only |

## 9. Pre-Closure Verification Checklist

| # | Check | Result |
|---|---|---|
| 1 | Spec text and realization notes updated for all in-scope items? | yes; this packet, W037 status surfaces, and the formatting watch rows record the change |
| 2 | Pack expectations updated for affected packs? | yes; no pack/C5 promotion is made, and residual blockers remain inputs to later W037 gates |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes for the positive dynamic bind row; five residual rows retain exact blocker evidence rather than promotion |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes; Section 7 covers the new carrier path and unchanged paths |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes; no exercised OxFml-owned formatting or FEC/F3E insufficiency is exposed |
| 6 | All required tests pass? | yes for the focused W037 conformance target; see Section 8 |
| 7 | No known semantic gaps remain in declared scope? | yes for the declared `calc-ubd.3` disposition target; five broader conformance gaps remain explicit residual blockers |
| 8 | Completion language audit passed? | yes; this packet does not claim full optimized/core-engine verification |
| 9 | `WORKSET_REGISTER.md` updated when ordered workset truth changed? | not applicable; ordered workset truth is unchanged |
| 10 | `IN_PROGRESS_FEATURE_WORKLIST.md` updated when feature-map truth changed? | yes; IP-04 records the W037 conformance decision evidence |
| 11 | execution-state blocker surface updated? | yes; `.beads/` records `calc-ubd.3` closure evidence and `calc-ubd.4` is the next ready W037 bead |

## 10. Completion Claim Self-Audit

| Step | Audit result |
|---|---|
| Scope re-read | pass; `calc-ubd.3` asks for W036 conformance rows to become implementation fixes, direct matches, or residual blockers |
| Gate criteria re-read | pass; all six rows have a validated W037 decision |
| Silent scope reduction check | pass; the positive dynamic row is promoted, but dynamic negative/release, callable, and coordinator-fence rows remain non-promoted |
| "Looks done but is not" pattern check | pass; W037 does not claim full optimized/core-engine verification, pack-grade replay, C5, or Stage 2 policy |
| Result | pass for the `calc-ubd.3` conformance-decision target |

## 11. Three-Axis Report

- execution_state: `calc-ubd.3_optimized_core_conformance_decisions_validated`
- scope_completeness: `scope_partial`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- open_lanes:
  - `calc-ubd.4` is the next ready W037 bead
  - five W037 implementation-conformance rows remain residual blockers
  - direct OxFml evaluator re-execution and `LET`/`LAMBDA` seam evidence remain open
  - Lean/TLA proof and model closure inventory remains open
  - Stage 2 deterministic replay and partition promotion criteria remain open
  - operated continuous assurance, operated cross-engine differential service, pack-grade replay governance, and C5 candidate decision remain unpromoted
