# W036 TraceCalc Coverage Closure Criteria And Matrix Expansion

Status: `calc-rqq.2_tracecalc_coverage_closure_matrix_validated`
Workset: `W036`
Parent epic: `calc-rqq`
Bead: `calc-rqq.2`

## 1. Purpose

This packet expands the W035 bounded TraceCalc oracle matrix into a W036 coverage-criteria artifact for the current hand-auditable TraceCalc corpus.

The result is still not a full core-engine oracle claim. The W036 matrix states which current rows are covered, which rows remain uncovered, which rows are excluded by authority, and how W033-W035 scenarios and W035 matrix rows carry forward without loss.

## 2. Authority Inputs Reviewed

| Input | Role in this bead |
|---|---|
| `docs/spec/core-engine/w036-formalization/W036_RESIDUAL_COVERAGE_AND_PROMOTION_BLOCKER_LEDGER.md` | W036 obligations `W036-OBL-001` and `W036-OBL-002` |
| `docs/worksets/W036_CORE_FORMALIZATION_VERIFICATION_CLOSURE_EXPANSION.md` | W036 scope, gate model, and guardrails |
| `docs/spec/core-engine/w035-formalization/W035_TRACECALC_ORACLE_MATRIX_EXPANSION.md` | W035 matrix baseline and uncovered rows |
| `docs/test-runs/core-engine/tracecalc-reference-machine/w035-tracecalc-oracle-matrix-001/` | W035 deterministic source evidence |
| `docs/test-corpus/core-engine/tracecalc/MANIFEST.json` | current 30-scenario hand-auditable TraceCalc corpus |
| `../OxFml/docs/worksets/W073_conditional_formatting_typed_visualization_payload.md` | current W073 watch/input-contract surface |

## 3. Implementation Surface

| Surface | Change |
|---|---|
| `src/oxcalc-tracecalc/src/oracle_matrix.rs` | keeps the W035 matrix profile and adds a W036 coverage-closure profile when the run id contains `w036` |
| `src/oxcalc-tracecalc-cli/src/main.rs` | reports excluded-row counts for oracle-matrix runs |
| `docs/test-runs/core-engine/tracecalc-reference-machine/w036-tracecalc-coverage-closure-001/` | checked-in deterministic W036 run artifact root |

The W036 profile retains all W035 matrix row identities, adds the 15 previously non-matrix corpus scenarios as covered rows, separates one authority exclusion from the uncovered register, and emits closure/crosswalk artifacts.

## 4. Matrix Summary

Run id: `w036-tracecalc-coverage-closure-001`

| Metric | Value |
|---|---:|
| TraceCalc scenarios | 30 |
| Matrix rows | 32 |
| Covered rows | 30 |
| Classified uncovered rows | 1 |
| Excluded rows | 1 |
| Failed or missing rows | 0 |
| Missing no-loss crosswalk rows | 0 |
| Full oracle claim | `false` |
| Matrix validation | `matrix_valid` |

Primary artifacts:

1. `docs/test-runs/core-engine/tracecalc-reference-machine/w036-tracecalc-coverage-closure-001/run_summary.json`
2. `docs/test-runs/core-engine/tracecalc-reference-machine/w036-tracecalc-coverage-closure-001/oracle-matrix/run_summary.json`
3. `docs/test-runs/core-engine/tracecalc-reference-machine/w036-tracecalc-coverage-closure-001/oracle-matrix/coverage_matrix.json`
4. `docs/test-runs/core-engine/tracecalc-reference-machine/w036-tracecalc-coverage-closure-001/oracle-matrix/coverage_closure_criteria.json`
5. `docs/test-runs/core-engine/tracecalc-reference-machine/w036-tracecalc-coverage-closure-001/oracle-matrix/no_loss_crosswalk.json`
6. `docs/test-runs/core-engine/tracecalc-reference-machine/w036-tracecalc-coverage-closure-001/oracle-matrix/uncovered_surface_register.json`
7. `docs/test-runs/core-engine/tracecalc-reference-machine/w036-tracecalc-coverage-closure-001/oracle-matrix/excluded_surface_register.json`
8. `docs/test-runs/core-engine/tracecalc-reference-machine/w036-tracecalc-coverage-closure-001/oracle-matrix/validation.json`

## 5. Coverage Criteria

The machine-readable criterion is emitted in `coverage_closure_criteria.json`.

| Class | Meaning |
|---|---|
| `covered` | scenario result passed, validation/assertion/conformance mismatch arrays are empty, and every required trace label is present |
| `uncovered` | relevant to future core-engine verification but no deterministic TraceCalc scenario exists in this profile |
| `excluded` | outside the TraceCalc profile by authority, not silently omitted |
| `missing_or_failed` | scenario failed, required label missing, or a source artifact could not be read |

No full TraceCalc oracle claim is made. Full promotion would require zero missing/failed rows, zero uncovered rows, no core-engine-owned excluded rows, optimized/core-engine conformance closure, and discharge or explicit non-oracle ownership for Lean/TLA obligations.

## 6. No-Loss Crosswalk

`no_loss_crosswalk.json` records two no-loss relations:

1. all 17 W035 matrix row identities are retained in the W036 matrix,
2. all 18 W033-W035 tagged TraceCalc scenarios map to at least one W036 row.

The crosswalk reports `missing_crosswalk_count = 0`.

## 7. Residual Rows

| Row | Class | Owner | Promotion consequence |
|---|---|---|---|
| `w035_overlay_multi_reader_release_order` | `uncovered` | `calc-rqq.5` | TraceCalc remains single-threaded; W036 TLA Stage 2 partition and scheduler-equivalence work must own this before any Stage 2/full-oracle promotion |
| `w035_callable_full_oxfunc_semantics` | `excluded` | `external:OxFunc; calc-rqq.4 records boundary` | W036 keeps the OxCalc/OxFml `LET`/`LAMBDA` carrier fragment in scope but excludes the general OxFunc LAMBDA kernel from TraceCalc oracle coverage |

## 8. OxFml W073 Intake

The current OxFml W073 formatting update remains watch/input-contract evidence only for this bead. The W036 TraceCalc matrix does not construct conditional-formatting aggregate or visualization payloads, and no OxCalc request-construction path exists in this repo for `VerificationConditionalFormattingRule`.

If a later W036 artifact exercises W073 aggregate or visualization payloads, `typed_rule` remains the only accepted metadata source for `colorScale`, `dataBar`, `iconSet`, `top`, `bottom`, `aboveAverage`, and `belowAverage`. W072 bounded `thresholds` strings remain intentionally ignored for those families.

No OxFml handoff is filed by this bead.

## 9. Semantic-Equivalence Statement

This bead adds a W036 TraceCalc oracle-matrix profile, generated evidence artifacts, and documentation. It does not change coordinator scheduling, dirty marking, dependency graph construction, soft-reference resolution, recalc semantics, publication fences, reject policy, overlay lifecycle semantics, TraceCalc execution semantics, TreeCalc/CoreEngine behavior, formal theorem statements, TLA actions, pack-decision logic, continuous-assurance runner semantics, or OxFml/OxFunc evaluator behavior.

Observable core-engine runtime behavior is invariant under this bead. The new artifacts classify and crosswalk evidence; they do not alter production or reference-machine evaluation semantics.

## 10. Verification

| Command | Result |
|---|---|
| `cargo test -p oxcalc-tracecalc oracle_matrix` | passed; 2 tests |
| `cargo test -p oxcalc-tracecalc-cli --no-run` | passed |
| `cargo run -p oxcalc-tracecalc-cli -- tracecalc-oracle-matrix w036-tracecalc-coverage-closure-001` | passed; emitted 32 matrix rows, 30 covered rows, 1 classified uncovered row, 1 excluded row, and 0 failed/missing rows |

Final workset/bead validation is recorded in the bead closure note after status surfaces are updated.

## 11. Pre-Closure Verification Checklist

| # | Check | Result |
|---|---|---|
| 1 | Spec text and realization notes updated for all in-scope items? | yes; this packet, W036 status, spec index, and feature-map surfaces record the W036 coverage-criteria evidence |
| 2 | Pack expectations updated for affected packs? | yes; pack remains unpromoted and `calc-rqq.8` owns reassessment after W036 evidence is bound |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes; the checked-in W036 run root carries per-scenario artifacts, matrix validation, closure criteria, crosswalk, uncovered, and excluded registers |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes; Section 9 states no runtime policy or strategy changed |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes; W073 remains watch/input-contract evidence with no exercised payload mismatch or handoff trigger |
| 6 | All required tests pass? | yes for focused validation; final validation is recorded in the bead closure note |
| 7 | No known semantic gaps remain in declared scope? | yes for this target; covered, uncovered, and excluded rows are machine-readable and no full oracle claim is made |
| 8 | Completion language audit passed? | yes; broader formalization and full oracle/implementation verification remain partial |
| 9 | `WORKSET_REGISTER.md` updated when ordered workset truth changed? | not applicable; ordered workset truth is unchanged |
| 10 | `IN_PROGRESS_FEATURE_WORKLIST.md` updated when feature-map truth changed? | yes; IP-04 carries the W036 matrix evidence |
| 11 | execution-state blocker surface updated? | yes; `.beads/` records `calc-rqq.2` execution state and will carry closure evidence |

## 12. Completion Claim Self-Audit

| Step | Audit result |
|---|---|
| Scope re-read | pass; `calc-rqq.2` asks for W036 TraceCalc coverage criteria and matrix expansion |
| Gate criteria re-read | pass; machine-readable matrix states covered, uncovered, and excluded rows, plus no-loss relation to W033-W035 scenarios |
| Silent scope reduction check | pass; multi-reader overlay and full OxFunc kernel surfaces remain explicit and are not silently omitted |
| "Looks done but is not" pattern check | pass; no full oracle, optimized/core-engine conformance, Stage 2, pack-grade, Lean, or TLA promotion claim is made |
| Result | pass for the `calc-rqq.2` coverage-criteria target |

## 13. Three-Axis Report

- execution_state: `calc-rqq.2_tracecalc_coverage_closure_matrix_validated`
- scope_completeness: `scope_partial`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- open_lanes:
  - `calc-rqq.3` through `calc-rqq.9` remain open
  - optimized/core-engine conformance closure remains open
  - full Lean/TLA verification remains open
  - concrete Stage 2 partition modeling and replay equivalence remain open
  - independent evaluator diversity, cross-engine differential service, continuous assurance operation/history, pack-grade replay, and Stage 2 policy remain unpromoted
