# W038 Optimized Core-Engine Conformance Blocker Closure And Fixes

Status: `calc-zsr.3_optimized_core_conformance_disposition_validated`
Workset: `W038`
Parent epic: `calc-zsr`
Bead: `calc-zsr.3`

## 1. Purpose

This packet rechecks the five W037 optimized/core-engine residual blockers and converts each one into one of the W038 allowed dispositions:

1. direct evidence bound with an exact remaining blocker,
2. accepted boundary/spec ownership decision,
3. exact remaining blocker.

No W037 residual blocker is counted as an optimized/core-engine match in this W038 slice.

## 2. Authority Inputs Reviewed

| Input | Role |
|---|---|
| `docs/worksets/W038_CORE_FORMALIZATION_RELEASE_GRADE_CLOSURE_HARDENING.md` | W038 gate model and `calc-zsr.3` target |
| `docs/spec/core-engine/w038-formalization/W038_RESIDUAL_RELEASE_GRADE_OBLIGATION_LEDGER_AND_OBJECTIVE_MAP.md` | W038 obligations `W038-OBL-003` through `W038-OBL-007` |
| `docs/test-runs/core-engine/implementation-conformance/w037-implementation-conformance-closure-001/w037_residual_blocker_register.json` | source W037 residual blockers |
| `docs/test-runs/core-engine/treecalc-local/w037-optimized-core-conformance-treecalc-001/` | optimized TreeCalc-local evidence |
| `docs/test-runs/core-engine/upstream-host/w037-direct-oxfml-evaluator-001/` | direct OxFml and narrow `LET`/`LAMBDA` evidence |
| `docs/test-runs/core-engine/tracecalc-authority/w038-tracecalc-authority-discharge-001/run_summary.json` | accepted TraceCalc authority exclusion for general OxFunc kernels |
| `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md` | reviewed inbound observations; latest W073 formatting change is already captured by the typed-only guardrail |

## 3. Artifact Surface

Run id: `w038-optimized-core-conformance-disposition-001`

| Artifact | Result |
|---|---|
| `docs/test-runs/core-engine/implementation-conformance/w038-optimized-core-conformance-disposition-001/run_summary.json` | 5 W038 disposition rows, 3 direct-evidence rows, 1 accepted boundary, 4 exact remaining blockers, 0 match-promoted rows, 0 failed rows |
| `docs/test-runs/core-engine/implementation-conformance/w038-optimized-core-conformance-disposition-001/w038_conformance_disposition_register.json` | machine-readable disposition rows and evidence checks |
| `docs/test-runs/core-engine/implementation-conformance/w038-optimized-core-conformance-disposition-001/w038_exact_remaining_blocker_register.json` | exact remaining blockers and owner lanes |
| `docs/test-runs/core-engine/implementation-conformance/w038-optimized-core-conformance-disposition-001/w038_match_promotion_guard.json` | zero W038 match promotions; declared-gap guard holds |
| `docs/test-runs/core-engine/implementation-conformance/w038-optimized-core-conformance-disposition-001/evidence_summary.json` | source evidence index and counts |
| `docs/test-runs/core-engine/implementation-conformance/w038-optimized-core-conformance-disposition-001/validation.json` | validation status `implementation_conformance_w038_dispositions_valid` |

## 4. Disposition Summary

| W037 residual blocker | W038 disposition | Evidence consequence |
|---|---|---|
| dynamic negative/release/reclassification | partial direct evidence plus exact remaining blocker | TreeCalc dynamic rejection, resolved dynamic publication, and retained-overlay release are bound; release/reclassification differential remains absent |
| lambda host effect | accepted boundary after direct OxFml evidence | direct OxFml `LET`/`LAMBDA` rows remove the stale direct-evaluator absence, while general OxFunc kernels remain external |
| snapshot fence projection | exact remaining blocker | Stage 2/coordinator replay must supply a stale-candidate counterpart before promotion |
| capability-view fence projection | exact remaining blocker | broader TreeCalc capability reject is not the compatibility-fenced capability-view mismatch |
| callable metadata projection | exact remaining blocker | TreeCalc remains value-only; direct OxFml returned-lambda evidence and proof inventory preserve the carrier boundary but do not add metadata projection |

## 5. Match-Promotion Guard

`w038_match_promotion_guard.json` records:

1. `source_w037_residual_blocker_count: 5`,
2. `promoted_match_count: 0`,
3. `allowed_promoted_rows: []`,
4. `guard_status: w038_declared_gap_promotion_guard_holds`.

This preserves the W038 rule that a declared gap is not a match. Direct evidence may narrow or reclassify a blocker, but this packet does not promote full optimized/core-engine conformance.

## 6. OxFml Formatting Intake

Reviewed the current OxFml W073 formatting update.

OxCalc already reflects the current rule in W038 and W037 surfaces:

1. W073 aggregate and visualization conditional-formatting metadata is `typed_rule`-only.
2. `thresholds` remains only for scalar/operator/expression rule families where threshold text is the real input.
3. `format_delta` and `display_delta` remain distinct consequence categories.

No OxCalc code patch or OxFml handoff is triggered by the latest formatting update in this bead, because `calc-zsr.3` constructs no new conditional-formatting request payload.

## 7. Semantic-Equivalence Statement

This bead adds an implementation-conformance runner path, CLI reporting, emitted artifacts, and status/spec text for W038 blocker disposition.

It does not change coordinator scheduling, invalidation strategy, dependency graph construction, soft-reference update behavior, recalc behavior, publication semantics, reject policy, overlay lifecycle behavior, TreeCalc runtime behavior, TraceCalc reference semantics, OxFml evaluator behavior, OxFunc kernels, Stage 2 scheduler policy, or pack/C5 capability policy.

Observable runtime behavior is invariant under this bead. The packet only classifies current evidence and exact remaining blockers.

## 8. Verification

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | passed |
| `cargo test -p oxcalc-tracecalc implementation_conformance -- --nocapture` | passed; 4 tests |
| `cargo test -p oxcalc-tracecalc-cli` | passed; 0 tests |
| `cargo run -p oxcalc-tracecalc-cli -- implementation-conformance w038-optimized-core-conformance-disposition-001` | passed; emitted W038 disposition artifacts |
| JSON parse for `docs/test-runs/core-engine/implementation-conformance/w038-optimized-core-conformance-disposition-001/*.json` | passed |
| `scripts/check-worksets.ps1` | passed after bead closure |
| `br ready --json` | passed after bead closure; next ready bead is `calc-zsr.4` |
| `br dep cycles --json` | passed with `count: 0` |
| `git diff --check` | passed; CRLF normalization warnings only |

## 9. Pre-Closure Verification Checklist

| # | Check | Result |
|---|---|---|
| 1 | Spec text and realization notes updated for all in-scope items? | yes; this packet, W038 README/status surfaces, and machine-readable artifacts record the slice |
| 2 | Pack expectations updated for affected packs? | yes; pack/C5 remains unpromoted and `calc-zsr.8` owns reassessment |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes; the W038 runner binds existing deterministic TreeCalc, direct OxFml, TraceCalc-authority, Stage 2, and proof-inventory artifacts |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes; Section 7 states no runtime strategy changed |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes; OxFml W073 is compatible with current OxCalc evidence and no handoff trigger exists |
| 6 | All required tests pass? | yes; see Section 8 and closure report |
| 7 | No known semantic gaps remain in declared scope? | yes for the `calc-zsr.3` disposition target; broader semantic gaps are exact remaining blockers or later W038 lanes |
| 8 | Completion language audit passed? | yes; no release-grade, C5, pack-grade replay, Stage 2 policy, operated-service, or full optimized/core-engine verification promotion is claimed |
| 9 | `WORKSET_REGISTER.md` updated when ordered workset truth changed? | not applicable; ordered workset truth is unchanged |
| 10 | `IN_PROGRESS_FEATURE_WORKLIST.md` updated when feature-map truth changed? | yes; IP-04 records the W038 conformance-disposition state |
| 11 | execution-state blocker surface updated? | yes; `.beads/` records `calc-zsr.3` closure and `calc-zsr.4` readiness |

## 10. Completion Claim Self-Audit

| Step | Audit result |
|---|---|
| Scope re-read | pass; `calc-zsr.3` asks to convert the five W037 optimized/core residual blockers into fixes, direct matches, accepted spec evolution, or exact remaining blockers |
| Gate criteria re-read | pass; all five rows have a W038 disposition, evidence checks, and promotion consequences |
| Silent scope reduction check | pass; no row is silently dropped and no declared gap is promoted as a match |
| "Looks done but is not" pattern check | pass; direct evidence is distinguished from exact remaining blockers and release-grade optimized verification is not promoted |
| Result | pass for the `calc-zsr.3` target |

## 11. Three-Axis Report

- execution_state: `calc-zsr.3_optimized_core_conformance_disposition_validated`
- scope_completeness: `scope_partial`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- open_lanes:
  - `calc-zsr.4` proof-model assumption discharge and totality boundary hardening is next
  - dynamic release/reclassification differential remains an exact remaining blocker for release decision/successor routing
  - snapshot and capability-view fence counterparts remain exact Stage 2/coordinator replay blockers under `calc-zsr.5`
  - callable metadata projection remains exact proof/seam work under `calc-zsr.4` and `calc-zsr.7`
  - operated assurance, alert/quarantine, and cross-engine service remain open
  - independent evaluator diversity and OxFml seam watch closure remain open
  - pack-grade replay governance, C5, and W038 release decision remain open
