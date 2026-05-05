# W035 TraceCalc Oracle Matrix Expansion

Status: `calc-tkq.2_tracecalc_oracle_matrix_validated`
Workset: `W035`
Parent epic: `calc-tkq`
Bead: `calc-tkq.2`

## 1. Purpose

This packet expands the TraceCalc reference-oracle surface into a generated W035 matrix for stale fences, dependency updates, overlay retention, and the narrow `LET`/`LAMBDA` callable-carrier fragment.

The intent is not to treat the initial spec as frozen. The matrix gives W035 an executable evidence surface that can expose missing assumptions, spec pressure, implementation gaps, and later proof obligations.

## 2. Authority Inputs Reviewed

| Input | Role in this bead |
|---|---|
| `docs/spec/core-engine/w035-formalization/W035_RESIDUAL_PROOF_OBLIGATION_AND_SPEC_EVOLUTION_LEDGER.md` | W035 residual obligation owner and promotion limits |
| `docs/worksets/W035_CORE_FORMALIZATION_PROOF_AND_ASSURANCE_HARDENING.md` | W035 scope, gate model, and open-lane contract |
| `docs/spec/core-engine/w034-formalization/W034_TRACECALC_ORACLE_DEEPENING.md` | W034 seed scenarios and oracle-limit baseline |
| `docs/spec/core-engine/CORE_ENGINE_TRACECALC_REFERENCE_MACHINE.md` | TraceCalc reference-machine role |
| `docs/spec/core-engine/CORE_ENGINE_TEST_SCENARIO_SCHEMA_AND_TRACECALC.md` | scenario schema and assertion shape |
| `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md` | current inbound OxFml observation ledger |
| `../OxFml/docs/worksets/W073_conditional_formatting_typed_visualization_payload.md` | W073 formatting watch input |
| `../OxFml/docs/handoffs/HANDOFF-DNAONECALC-012_W073_TYPED_CF_PAYLOAD_FIRST_SLICE.md` | typed conditional-formatting uptake note |

## 3. Implementation Surface

| Surface | Change |
|---|---|
| `src/oxcalc-tracecalc/src/oracle_matrix.rs` | adds `TraceCalcOracleMatrixRunner`, matrix row specs, row validation, and matrix artifact emission |
| `src/oxcalc-tracecalc-cli/src/main.rs` | adds `tracecalc-oracle-matrix <run-id>` command |
| `docs/test-corpus/core-engine/tracecalc/MANIFEST.json` | expands the TraceCalc corpus from 21 to 30 scenarios |
| `docs/test-corpus/core-engine/tracecalc/hand-auditable/tc_w035_*.json` | adds nine W035 hand-auditable scenarios |
| `docs/test-runs/core-engine/tracecalc-reference-machine/w035-tracecalc-oracle-matrix-001/` | checked-in deterministic run artifact root |

## 4. Matrix Summary

Run id: `w035-tracecalc-oracle-matrix-001`

| Metric | Value |
|---|---:|
| TraceCalc scenarios | 30 |
| Matrix rows | 17 |
| Covered rows | 15 |
| Classified uncovered rows | 2 |
| Failed or missing rows | 0 |
| Matrix validation | `matrix_valid` |
| Replay bundle validation | `bundle_valid` |
| Oracle/engine conformance mismatches | 0 |

Primary artifacts:

1. `docs/test-runs/core-engine/tracecalc-reference-machine/w035-tracecalc-oracle-matrix-001/run_summary.json`
2. `docs/test-runs/core-engine/tracecalc-reference-machine/w035-tracecalc-oracle-matrix-001/oracle-matrix/run_summary.json`
3. `docs/test-runs/core-engine/tracecalc-reference-machine/w035-tracecalc-oracle-matrix-001/oracle-matrix/coverage_matrix.json`
4. `docs/test-runs/core-engine/tracecalc-reference-machine/w035-tracecalc-oracle-matrix-001/oracle-matrix/uncovered_surface_register.json`
5. `docs/test-runs/core-engine/tracecalc-reference-machine/w035-tracecalc-oracle-matrix-001/oracle-matrix/validation.json`
6. `docs/test-runs/core-engine/tracecalc-reference-machine/w035-tracecalc-oracle-matrix-001/replay-appliance/validation/bundle_validation.json`
7. `docs/test-runs/core-engine/tracecalc-reference-machine/w035-tracecalc-oracle-matrix-001/conformance/engine_diff.json`

## 5. Obligation Coverage

| Obligation | Matrix family | Covered rows | Classified uncovered rows | Disposition |
|---|---|---:|---:|---|
| `W035-OBL-001` | stale fences | 4 | 0 | snapshot, capability-view, profile-version, and post-candidate rejection rows covered |
| `W035-OBL-002` | dependency update | 5 | 0 | dynamic negative, static add, dynamic switch, dynamic release, and downstream dirty-seed closure rows covered |
| `W035-OBL-003` | overlay retention | 2 | 1 | retain/release and protected-overlay reuse rows covered; multi-reader release ordering routed to `calc-tkq.5` |
| `W035-OBL-004` | `LET`/`LAMBDA` callable carrier | 4 | 1 | direct, higher-order, defined-name, and reject/no-publish callable rows covered; full OxFunc kernel routed to `calc-tkq.4` |

## 6. Added W035 Scenarios

| Scenario | Matrix role |
|---|---|
| `tc_w035_profile_fence_reject_001` | profile-version stale-fence reject/no-publish row |
| `tc_w035_candidate_after_emit_fence_reject_001` | post-candidate rejection before publication |
| `tc_w035_static_dependency_add_publish_001` | static dependency-shape publication row |
| `tc_w035_dynamic_dependency_switch_publish_001` | dynamic dependency switch publication row |
| `tc_w035_dynamic_dependency_release_publish_001` | dynamic dependency release/reclassification publication row |
| `tc_w035_dirty_seed_closure_no_under_invalidation_001` | downstream dirty-seed closure row |
| `tc_w035_overlay_reuse_protected_dynamic_001` | protected dynamic overlay reuse row |
| `tc_w035_let_lambda_defined_name_callable_001` | defined-name callable carrier publication row |
| `tc_w035_lambda_callable_publication_reject_001` | callable-as-value reject/no-publish policy row |

## 7. Classified Uncovered Rows

| Row | Owner | Reason |
|---|---|---|
| `w035_overlay_multi_reader_release_order` | `calc-tkq.5` | TraceCalc is single-threaded and does not model full multi-reader interleavings; W035 TLA non-routine exploration owns this surface |
| `w035_callable_full_oxfunc_semantics` | `calc-tkq.4` | W035 covers the OxCalc/OxFml callable-carrier fragment, not the general OxFunc semantic kernel |

These rows are explicit deferrals, not failures. They remain visible in `uncovered_surface_register.json`.

## 8. OxFml Formatting Watch

The OxFml W073 formatting update is incorporated as watch/input-contract evidence only. This bead does not construct `VerificationConditionalFormattingRule` payloads and found no OxCalc runtime request-construction path to patch.

If a later W035 artifact exercises W073 aggregate or visualization conditional-formatting payloads, `typed_rule` remains the only accepted metadata source for `colorScale`, `dataBar`, `iconSet`, `top`, `bottom`, `aboveAverage`, and `belowAverage`. W072 bounded `thresholds` strings remain intentionally ignored for those families.

No OxFml handoff is filed by this bead.

## 9. Semantic-Equivalence Statement

This bead adds a TraceCalc oracle-matrix runner, a CLI command, nine hand-auditable W035 scenarios, and deterministic evidence artifacts. It does not change coordinator scheduling, dirty marking, dependency graph construction, soft-reference resolution, recalc semantics, publication fences, reject policy, overlay lifecycle semantics, formal theorem statements, TLA actions, pack-decision logic, or OxFml evaluator behavior.

Observable core-engine runtime behavior is invariant under this bead. The new artifacts exercise and classify behavior through TraceCalc; they do not alter the production coordinator or evaluator surface.

## 10. Verification

| Command | Result |
|---|---|
| `cargo run -p oxcalc-tracecalc-cli -- tracecalc-oracle-matrix w035-tracecalc-oracle-matrix-001` | passed; emitted 17 matrix rows, 15 covered rows, 2 classified uncovered rows, 0 failed/missing rows |
| `cargo test -p oxcalc-tracecalc oracle_matrix` | passed; 1 test |
| `cargo test -p oxcalc-tracecalc runner::tests::execute_manifest_produces_passing_conformance_artifacts_for_seed_corpus` | passed; 1 test |
| `cargo test -p oxcalc-tracecalc` | passed; 13 tests |
| `cargo test -p oxcalc-tracecalc-cli` | passed; 0 tests |
| `cargo fmt --all -- --check` | passed |

Workset, bead-graph, and diff hygiene commands are recorded in the closure bead note after the final status surfaces are updated.

## 11. Pre-Closure Verification Checklist

| # | Check | Result |
|---|---|---|
| 1 | Spec text and realization notes updated for all in-scope items? | yes; this packet, W035 status, and feature-map surfaces identify the oracle-matrix expansion and limits |
| 2 | Pack expectations updated for affected packs? | yes; no pack promotion is claimed, and pack reassessment remains owned by `calc-tkq.7` |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes; the checked-in `w035-tracecalc-oracle-matrix-001` root carries per-scenario artifacts and matrix validation |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes; Section 9 states no runtime policy or strategy changed |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes; W073 formatting was assessed as watch/input-contract evidence, with no concrete OxCalc handoff trigger |
| 6 | All required tests pass? | yes; see Section 10 and the bead closure note |
| 7 | No known semantic gaps remain in declared scope? | yes for this target; uncovered surfaces are classified and routed to later W035 beads |
| 8 | Completion language audit passed? | yes; broader formalization and full oracle coverage remain partial |
| 9 | `WORKSET_REGISTER.md` updated when ordered workset truth changed? | not applicable; ordered workset truth is unchanged |
| 10 | `IN_PROGRESS_FEATURE_WORKLIST.md` updated when feature-map truth changed? | yes; IP-04 points at this W035 matrix evidence |
| 11 | execution-state blocker surface updated? | yes; `.beads/` records `calc-tkq.2` execution state and later closure evidence |

## 12. Completion Claim Self-Audit

| Step | Audit result |
|---|---|
| Scope re-read | pass; `calc-tkq.2` asks for generated stale-fence, dependency-update, overlay-retention, and `LET`/`LAMBDA` callable matrices |
| Gate criteria re-read | pass; deterministic matrix artifacts exist and any remaining uncovered oracle surfaces are classified |
| Silent scope reduction check | pass; multi-reader overlay and full OxFunc kernel surfaces are explicitly routed rather than silently omitted |
| "Looks done but is not" pattern check | pass; no production implementation promotion or full formalization claim is made |
| Result | pass for the `calc-tkq.2` oracle-matrix target |

## 13. Three-Axis Report

- execution_state: `calc-tkq.2_tracecalc_oracle_matrix_validated`
- scope_completeness: `scope_partial`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- open_lanes:
  - `calc-tkq.3` through `calc-tkq.8` remain open
  - full formalization, full Lean/TLA verification, full TraceCalc oracle coverage, optimized/core-engine verification, pack-grade replay, continuous scale assurance, and Stage 2 policy remain partial
