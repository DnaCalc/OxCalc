# W038 TraceCalc Oracle Authority And Authority-Exclusion Discharge

Status: `calc-zsr.2_tracecalc_authority_discharge_validated`
Workset: `W038`
Parent epic: `calc-zsr`
Bead: `calc-zsr.2`

## 1. Purpose

This packet discharges the W037 TraceCalc authority-exclusion row for the current OxCalc-owned observable profile.

The result is narrow and non-promoting:

1. W037 has 32 TraceCalc matrix rows, 31 covered rows, 0 uncovered rows, 0 missing/failed rows, and 1 authority-excluded row.
2. The excluded row is `w035_callable_full_oxfunc_semantics`, the general OxFunc LAMBDA semantic kernel.
3. That row is accepted as an external-owner exclusion for OxCalc W038 release-grade accounting.
4. TraceCalc authority is therefore clean for the current OxCalc-owned observable reference profile.
5. Release-grade core verification is still not promoted because optimized/core-engine conformance, proof/model totality, Stage 2 replay, operated service, independent-diversity, and pack/C5 gates remain open.

## 2. Authority Inputs Reviewed

| Input | Role |
|---|---|
| `docs/spec/core-engine/w038-formalization/W038_RESIDUAL_RELEASE_GRADE_OBLIGATION_LEDGER_AND_OBJECTIVE_MAP.md` | W038 obligations `W038-OBL-001` and `W038-OBL-002` |
| `docs/test-runs/core-engine/tracecalc-reference-machine/w037-tracecalc-observable-closure-001/oracle-matrix/run_summary.json` | W037 TraceCalc row counts |
| `docs/test-runs/core-engine/tracecalc-reference-machine/w037-tracecalc-observable-closure-001/oracle-matrix/coverage_closure_criteria.json` | W037 coverage criteria and prior promotion blockers |
| `docs/test-runs/core-engine/tracecalc-reference-machine/w037-tracecalc-observable-closure-001/oracle-matrix/excluded_surface_register.json` | authority-exclusion row to discharge |
| `docs/test-runs/core-engine/tracecalc-reference-machine/w037-tracecalc-observable-closure-001/oracle-matrix/uncovered_surface_register.json` | confirms no uncovered W037 rows |
| `docs/test-runs/core-engine/tracecalc-reference-machine/w037-tracecalc-observable-closure-001/oracle-matrix/no_loss_crosswalk.json` | no-loss map from W033-W035 rows and scenarios |
| `docs/test-runs/core-engine/upstream-host/w037-direct-oxfml-evaluator-001/run_summary.json` | confirms the W037 direct OxFml slice exists and supersedes the older direct-evaluator absence blocker |
| `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md` | reviewed inbound observations; no new TraceCalc authority handoff trigger |

## 3. Artifact Surface

Run id: `w038-tracecalc-authority-discharge-001`

| Artifact | Result |
|---|---|
| `archive/test-runs-core-engine-w038-w045/tracecalc-authority/w038-tracecalc-authority-discharge-001/run_summary.json` | records scoped TraceCalc authority disposition |
| `archive/test-runs-core-engine-w038-w045/tracecalc-authority/w038-tracecalc-authority-discharge-001/authority_discharge_ledger.json` | discharges the general OxFunc kernel exclusion as external authority |
| `archive/test-runs-core-engine-w038-w045/tracecalc-authority/w038-tracecalc-authority-discharge-001/oracle_authority_map.json` | maps covered, excluded, superseded, and non-TraceCalc gates |
| `archive/test-runs-core-engine-w038-w045/tracecalc-authority/w038-tracecalc-authority-discharge-001/validation.json` | records validation for this packet |

## 4. Authority Decision

| Surface | W037 state | W038 disposition |
|---|---|---|
| OxCalc-owned observable reference rows | 31 covered rows, 0 failed/missing rows | accepted as covered reference behavior for the current W038 TraceCalc profile |
| uncovered rows | 0 uncovered rows | no W038 uncovered TraceCalc row remains |
| general OxFunc LAMBDA semantic kernel | 1 excluded row: `w035_callable_full_oxfunc_semantics` | accepted external-owner exclusion; not part of OxCalc release-grade claim |
| direct OxFml evaluator absence | W037 coverage criteria listed it as an older promotion blocker | superseded by `w037-direct-oxfml-evaluator-001` for the exercised upstream-host slice |
| optimized/core-engine conformance | not a TraceCalc authority row | remains owned by `calc-zsr.3` |
| proof/model totality | not a TraceCalc authority row | remains owned by `calc-zsr.4` and `calc-zsr.5` |
| Stage 2 partition replay | not a TraceCalc authority row | remains owned by `calc-zsr.5` |
| operated service and independent diversity | not TraceCalc authority rows | remain owned by `calc-zsr.6` and `calc-zsr.7` |
| pack/C5 release decision | not a TraceCalc authority row | remains owned by `calc-zsr.8` and `calc-zsr.9` |

## 5. Promotion Consequence

This packet removes the W038 TraceCalc-specific authority-exclusion uncertainty for the current OxCalc-owned observable profile.

It does not promote:

1. release-grade full verification,
2. full optimized/core-engine verification,
3. full Lean/TLA verification,
4. general OxFunc kernel semantics,
5. Stage 2 policy,
6. operated assurance service,
7. fully independent evaluator diversity,
8. pack-grade replay,
9. C5.

The next direct evidence bead is `calc-zsr.3` for optimized/core-engine conformance blocker closure.

## 6. Reviewed Inbound Observations

Reviewed `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md`.

Current TraceCalc-authority intake:

1. The latest OxFml formatting update remains compatible with the W037 direct upstream-host guard.
2. General OxFunc callable kernel semantics remain external to OxCalc; this packet does not claim them.
3. The narrow `LET`/`LAMBDA` carrier stays in OxCalc/OxFml scope where exercised.
4. No OxFml handoff is filed because the accepted external-authority row is an OxFunc-owned semantic-kernel boundary, not an OxFml contract defect.

## 7. Semantic-Equivalence Statement

This bead adds TraceCalc authority-discharge documentation and machine-readable authority artifacts only.

It does not change coordinator scheduling, invalidation strategy, dependency graph construction, soft-reference update behavior, recalc behavior, publication semantics, reject policy, overlay lifecycle behavior, TraceCalc execution semantics, TreeCalc/CoreEngine runtime behavior, Lean/TLA model semantics, continuous-assurance runner semantics, pack runtime behavior, service behavior, alert dispatch behavior, or OxFml/OxFunc evaluator behavior.

Observable runtime behavior is invariant under this bead.

## 8. Verification

| Command | Result |
|---|---|
| JSON parse for `archive/test-runs-core-engine-w038-w045/tracecalc-authority/w038-tracecalc-authority-discharge-001/*.json` | passed |
| `scripts/check-worksets.ps1` | passed after bead closure |
| `br ready --json` | passed after bead closure; next ready bead is `calc-zsr.3` |
| `br dep cycles --json` | passed with `count: 0` |
| `git diff --check` | passed; CRLF normalization warnings only |

No cargo, Lean, or TLC command is required for this authority-discharge bead because it emits no code, formal model, fixture, runner, replay behavior, or runtime semantics.

## 9. Pre-Closure Verification Checklist

| # | Check | Result |
|---|---|---|
| 1 | Spec text and realization notes updated for all in-scope items? | yes; this packet, W038 README/status surfaces, and machine-readable authority artifacts record the slice |
| 2 | Pack expectations updated for affected packs? | yes; pack/C5 remains unpromoted and `calc-zsr.8` owns reassessment |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes for TraceCalc-covered predecessor behavior; W037 replay artifacts remain the cited authority |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes; Section 7 states no runtime strategy changed |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes; no OxFml or FEC/F3E handoff trigger exists |
| 6 | All required tests pass? | yes for this documentation/bead-graph audit scope; see Section 8 |
| 7 | No known semantic gaps remain in declared scope? | yes for the TraceCalc authority-discharge target; non-TraceCalc gates remain explicit open lanes |
| 8 | Completion language audit passed? | yes; no release-grade or full verification promotion is claimed |
| 9 | `WORKSET_REGISTER.md` updated when ordered workset truth changed? | not applicable; ordered workset truth is unchanged |
| 10 | `IN_PROGRESS_FEATURE_WORKLIST.md` updated when feature-map truth changed? | yes; IP-04 records the W038 TraceCalc authority-discharge state |
| 11 | execution-state blocker surface updated? | yes; `.beads/` records `calc-zsr.2` closure and `calc-zsr.3` readiness |

## 10. Completion Claim Self-Audit

| Step | Audit result |
|---|---|
| Scope re-read | pass; `calc-zsr.2` asks to discharge or justify W037 TraceCalc authority exclusions |
| Gate criteria re-read | pass; covered reference behavior, accepted external exclusion, and remaining non-TraceCalc gates are separated |
| Silent scope reduction check | pass; general OxFunc kernels and non-TraceCalc release gates remain explicitly outside this target |
| "Looks done but is not" pattern check | pass; a TraceCalc authority discharge is not represented as release-grade full verification |
| Result | pass for the `calc-zsr.2` target |

## 11. Three-Axis Report

- execution_state: `calc-zsr.2_tracecalc_authority_discharge_validated`
- scope_completeness: `scope_partial`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- open_lanes:
  - `calc-zsr.3` optimized core-engine conformance blocker closure and fixes is next
  - proof/model assumption discharge remains open
  - Stage 2 partition replay and semantic-equivalence execution remain open
  - operated assurance, alert/quarantine, and cross-engine service remain open
  - independent evaluator diversity and OxFml seam watch closure remain open
  - pack-grade replay governance, C5, and W038 release decision remain open
