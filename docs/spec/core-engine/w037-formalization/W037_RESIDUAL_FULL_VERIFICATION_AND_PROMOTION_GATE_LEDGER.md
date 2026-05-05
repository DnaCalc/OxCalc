# W037 Residual Full-Verification And Promotion-Gate Ledger

Status: `calc-ubd.2_residual_full_verification_ledger_validated`
Workset: `W037`
Parent epic: `calc-ubd`
Bead: `calc-ubd.2`

## 1. Purpose

This ledger converts W036 closure residuals and no-promotion blockers into W037 obligations.

W037 begins from a strong W036 verification tranche, but not from full verification. W036 deepened coverage, conformance, Lean, TLA, differential, continuous-assurance, and pack/capability evidence while deliberately keeping full TraceCalc oracle coverage, full optimized/core-engine verification, full Lean/TLA verification, direct OxFml evaluator re-execution, operated services, C5, pack-grade replay, and Stage 2 policy unpromoted.

This ledger prevents those remaining lanes from staying only in prose. It makes each W037 obligation traceable to an owner bead, evidence root, and promotion consequence.

## 2. Authority Inputs Reviewed

| Input | Role in W037 |
|---|---|
| `docs/spec/core-engine/w036-formalization/W036_CLOSURE_AUDIT_AND_SUCCESSOR_PACKET.md` | canonical W036 closure audit and successor packet |
| `docs/worksets/W037_CORE_FORMALIZATION_FULL_VERIFICATION_PROMOTION_GATES.md` | W037 workset scope and gate |
| `docs/spec/core-engine/w036-formalization/W036_RESIDUAL_COVERAGE_AND_PROMOTION_BLOCKER_LEDGER.md` | W036 obligation predecessor ledger |
| `docs/spec/core-engine/w036-formalization/W036_TRACECALC_COVERAGE_CLOSURE_CRITERIA_AND_MATRIX_EXPANSION.md` | W036 TraceCalc coverage-criteria source |
| `docs/spec/core-engine/w036-formalization/W036_OPTIMIZED_CORE_ENGINE_CONFORMANCE_CLOSURE_PLAN_AND_FIRST_FIXES.md` | W036 optimized/core-engine conformance source |
| `docs/spec/core-engine/w036-formalization/W036_LEAN_THEOREM_COVERAGE_EXPANSION.md` | W036 Lean/callable-boundary source |
| `docs/spec/core-engine/w036-formalization/W036_TLA_STAGE2_PARTITION_AND_SCHEDULER_EQUIVALENCE_MODEL.md` | W036 TLA/Stage 2 source |
| `docs/spec/core-engine/w036-formalization/W036_INDEPENDENT_EVALUATOR_DIVERSITY_AND_CROSS_ENGINE_DIFFERENTIAL_HARNESS.md` | W036 diversity/differential source |
| `docs/spec/core-engine/w036-formalization/W036_CONTINUOUS_ASSURANCE_OPERATION_AND_HISTORY_WINDOW.md` | W036 continuous-assurance source |
| `docs/spec/core-engine/w036-formalization/W036_PACK_GRADE_REPLAY_AND_CAPABILITY_PROMOTION_GATE_REASSESSMENT.md` | W036 pack/capability no-promotion source |
| `docs/test-runs/core-engine/tracecalc-reference-machine/w036-tracecalc-coverage-closure-001/` | deterministic W036 TraceCalc evidence |
| `docs/test-runs/core-engine/implementation-conformance/w036-implementation-conformance-closure-001/` | deterministic W036 conformance evidence |
| `docs/test-runs/core-engine/tla/w036-stage2-partition-001/` | deterministic W036 TLC evidence |
| `docs/test-runs/core-engine/independent-conformance/w036-independent-diversity-differential-001/` | deterministic W036 diversity evidence |
| `docs/test-runs/core-engine/cross-engine-differential/w036-independent-diversity-differential-001/` | deterministic W036 differential evidence |
| `docs/test-runs/core-engine/continuous-assurance/w036-continuous-assurance-operation-001/` | deterministic W036 continuous-assurance evidence |
| `docs/test-runs/core-engine/pack-capability/w036-pack-capability-reassessment-001/` | deterministic W036 pack/capability decision |
| `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md` | current inbound OxFml observation ledger |

## 3. Evidence Roots Declared

W037 may emit artifacts under these roots:

1. `docs/spec/core-engine/w037-formalization/`
2. `docs/test-runs/core-engine/tracecalc-reference-machine/w037-*`
3. `docs/test-runs/core-engine/implementation-conformance/w037-*`
4. `docs/test-runs/core-engine/treecalc-local/w037-*`
5. `docs/test-runs/core-engine/independent-conformance/w037-*`
6. `docs/test-runs/core-engine/cross-engine-differential/w037-*`
7. `docs/test-runs/core-engine/continuous-assurance/w037-*`
8. `docs/test-runs/core-engine/pack-capability/w037-*`
9. `formal/lean/OxCalc/CoreEngine/W037*.lean`
10. `formal/tla/CoreEngineW037*.tla`
11. `formal/tla/CoreEngineW037*.cfg`
12. `docs/handoffs/` only if a concrete OxFml-owned seam insufficiency is exposed and registered through the handoff process.

Checked-in evidence must use repo-relative paths. Validation runs must not mutate prior W033, W034, W035, or W036 baselines unless a later bead explicitly regenerates and supersedes them.

## 4. Promotion Limits

W037 starts with these limits:

1. `cap.C5.pack_valid` is not promoted.
2. pack-grade replay governance is not promoted.
3. Stage 2 scheduler policy is not promoted.
4. full TraceCalc oracle coverage is not claimed.
5. full optimized/core-engine verification is not claimed.
6. fully independent evaluator implementation diversity is not claimed.
7. direct OxFml evaluator re-execution is exercised for the `calc-ubd.4` upstream-host fixture slice only; pack-grade replay remains unpromoted.
8. full Lean verification is not claimed.
9. full TLA+ verification is not claimed.
10. operated continuous assurance is not promoted.
11. operated continuous cross-engine differential service is not promoted.
12. alert/quarantine enforcement service is not promoted.
13. timing or scaling evidence is not semantic correctness proof.
14. W073 conditional-formatting typed metadata remains watch/input-contract evidence until an exercised OxCalc artifact constructs that payload family.
15. the semantic-format versus display-facing boundary remains watch/input-contract evidence: `format_delta` and `display_delta` must stay distinct, and broader display closure is not assumed.

Any later W037 promotion candidate must include direct artifacts, a semantic-equivalence statement, and an updated pack/capability decision.

## 5. W036 No-Promotion Disposition

| W036 reason id | W037 disposition |
|---|---|
| `pack.grade.program_scope.unproven` | carry to `W037-OBL-013` pack/program governance |
| `pack.grade.direct_oxfml_evaluator_reexecution_absent` | carry to `W037-OBL-005` direct OxFml evaluator re-execution |
| `pack.grade.independent_conformance_declared_gaps` | carry to `W037-OBL-003` and `W037-OBL-004` implementation/conformance closure |
| `pack.grade.continuous_diff_suite_absent` | carry to `W037-OBL-011` operated cross-engine differential service |
| `pack.grade.fully_independent_evaluator_absent` | carry to `W037-OBL-010` independent evaluator diversity |
| `pack.grade.treecalc_c4_c5_unproven` | carry to `W037-OBL-013` pack/C5 decision |
| `pack.grade.w036_formal_slices_bounded_not_full_verification` | carry to `W037-OBL-007` and `W037-OBL-008` proof/model closure |
| `pack.grade.w036_stage2_scheduler_policy_unpromoted` | carry to `W037-OBL-009` Stage 2 deterministic replay and promotion criteria |
| `pack.grade.w036_tracecalc_oracle_not_full_coverage` | carry to `W037-OBL-001` and `W037-OBL-002` TraceCalc observable closure |
| `pack.grade.w036_declared_gap_blockers_remain` | carry to `W037-OBL-003` and `W037-OBL-004` |
| `pack.grade.w036_optimized_core_engine_conformance_not_full` | carry to `W037-OBL-003` and `W037-OBL-004` |
| `pack.grade.w036_stage2_replay_equivalence_not_pack_grade` | carry to `W037-OBL-009` |
| `pack.grade.w036_fully_independent_evaluator_absent` | carry to `W037-OBL-010` |
| `pack.grade.w036_continuous_cross_engine_diff_service_absent` | carry to `W037-OBL-011` and `W037-OBL-012` |
| `pack.grade.w036_continuous_assurance_simulated_not_operated` | carry to `W037-OBL-012` |
| `pack.grade.w036_quarantine_policy_not_enforced_by_service` | carry to `W037-OBL-012` |
| `pack.grade.w036_timing_not_correctness_proof` | retained as guardrail for `W037-OBL-012` and `W037-OBL-013` |
| `pack.grade.program_grade_replay_governance_not_reached` | carry to `W037-OBL-013` |
| `pack.grade.retained_witness_promotion_not_shared_program_grade` | carry to `W037-OBL-013` |
| `pack.grade.w036_program_grade_replay_governance_not_reached` | carry to `W037-OBL-013` |
| `pack.grade.w036_direct_oxfml_evaluator_reexecution_absent` | carry to `W037-OBL-005` |
| `pack.grade.w036_pack_c5_no_promotion_after_reassessment` | carry to `W037-OBL-013` and `W037-OBL-014` |

## 6. Residual Obligation Matrix

| Obligation id | Area | W036 floor | W037 owner | Required W037 disposition |
|---|---|---|---|---|
| `W037-OBL-001` | TraceCalc observable closure | 32 matrix rows, 30 covered, 1 uncovered, 1 excluded, no full oracle claim | `calc-ubd.1` | produce current observable-semantics closure criteria with deterministic replay or explicit authority-exclusion for every in-scope row |
| `W037-OBL-002` | multi-reader overlay release ordering | W036 has bounded TLA model evidence, not TraceCalc replay | `calc-ubd.1`, `calc-ubd.6` | add direct replay, justify authority exclusion, or retain exact blocker and promotion consequence |
| `W037-OBL-003` | optimized/core-engine declared gap closure | W036 has 2 harness first-fix rows, 4 blocker-routed rows, and 0 match-promoted rows | `calc-ubd.3` | convert gaps into implementation fixes, direct differential matches, spec evolution, or residual blockers |
| `W037-OBL-004` | match-promotion guard and conformance proof | W036 forbids counting declared gaps as matches | `calc-ubd.3` | preserve the guard and define what evidence is sufficient to promote each gap row |
| `W037-OBL-005` | direct OxFml evaluator re-execution | W036 has projection evidence and direct evaluator absence as pack blocker | `calc-ubd.4` | exercised for the upstream-host fixture slice by `w037-direct-oxfml-evaluator-001`; pack-grade replay still awaits later gates |
| `W037-OBL-006` | `LET`/`LAMBDA` OxFml/OxFunc carrier seam | W036 checked callable-carrier boundary inventory and kept full OxFunc kernel opaque | `calc-ubd.4`, `calc-ubd.5` | `calc-ubd.4` exercises lexical capture and returned-lambda invocation through OxFml; `calc-ubd.5` still owns proof/model inventory and general OxFunc kernel exclusion |
| `W037-OBL-007` | Lean proof closure inventory | W036 Lean artifacts have zero explicit axioms but remain inventory slices | `calc-ubd.5` | `calc-ubd.5` exercises a checked W037 closure inventory over 12 Lean files with 0 explicit axioms and 0 `sorry`/`admit` placeholders; full Lean verification remains open |
| `W037-OBL-008` | TLA model closure inventory | W036 checks bounded Stage 2 partition configs and no policy promotion | `calc-ubd.5`, `calc-ubd.6` | `calc-ubd.5` exercises 11 routine TLC configs and classifies full TLA, Stage 2 replay, and policy-promotion gaps; `calc-ubd.6` still owns deterministic replay and partition criteria |
| `W037-OBL-009` | Stage 2 deterministic replay and partition promotion criteria | W036 has bounded partition model and scheduler-readiness criteria, but no deterministic replay equivalence | `calc-ubd.6`, `calc-ubd.8` | define replayable promotion criteria and semantic-equivalence obligations before any Stage 2 claim |
| `W037-OBL-010` | independent evaluator diversity | W036 classifies 0 fully independent evaluator rows | `calc-ubd.7` | define fully independent evaluator criteria and either add qualifying rows or record blockers |
| `W037-OBL-011` | operated cross-engine differential service | W036 emits deterministic differential rows without an operated service | `calc-ubd.7` | pilot, simulate with stricter limits, or explicitly block service claims |
| `W037-OBL-012` | operated continuous assurance and quarantine | W036 emits simulated multi-run history, thresholds, and policy only | `calc-ubd.7` | establish operated evidence or preserve non-promotion with retention, alert, and quarantine blockers |
| `W037-OBL-013` | pack-grade replay governance and C5 | W036 highest honest capability is `cap.C4.distill_valid` with 22 blockers | `calc-ubd.8` | reassess C5 only after direct evaluator, conformance, proof/model, service, and Stage 2 gates are bound |
| `W037-OBL-014` | closure audit and release decision | W036 keeps broader objective `in_progress` | `calc-ubd.9` | map the active objective to artifacts and decide release-grade verification, successor scope, or spec/scope revision |
| `W037-OBL-015` | OxFml inbound seam/watch updates | current OxFml notes include W073 formatting guard, semantic-format/display boundary, consumer public surface, host/runtime, table, immutable-edit, stand-in, and registered-external packet convergence | `calc-ubd.4`, all beads where exercised | consume current OxFml surfaces without direct sibling edits; file handoff only on concrete mismatch |
| `W037-OBL-016` | spec evolution discipline | W037 is not a fixed-spec test pass | every W037 bead | patch specs, create implementation work, file handoffs, or record deferrals when evidence changes understanding |
| `W037-OBL-017` | evidence non-mutation and lineage | W033-W036 checked baselines exist | every W037 bead | declare new run ids and avoid accidental mutation of older baselines |

## 7. Bead Mapping

| Bead | Primary obligations |
|---|---|
| `calc-ubd.1` | `W037-OBL-001`, `W037-OBL-002` |
| `calc-ubd.3` | `W037-OBL-003`, `W037-OBL-004`, `W037-OBL-016`, `W037-OBL-017` |
| `calc-ubd.4` | `W037-OBL-005`, `W037-OBL-006`, `W037-OBL-015`, `W037-OBL-016`, `W037-OBL-017` |
| `calc-ubd.5` | `W037-OBL-006`, `W037-OBL-007`, `W037-OBL-008`, `W037-OBL-016`, `W037-OBL-017` |
| `calc-ubd.6` | `W037-OBL-002`, `W037-OBL-008`, `W037-OBL-009`, `W037-OBL-016`, `W037-OBL-017` |
| `calc-ubd.7` | `W037-OBL-010`, `W037-OBL-011`, `W037-OBL-012`, `W037-OBL-016`, `W037-OBL-017` |
| `calc-ubd.8` | `W037-OBL-009`, `W037-OBL-013`, `W037-OBL-015`, `W037-OBL-016`, `W037-OBL-017` |
| `calc-ubd.9` | `W037-OBL-014`, `W037-OBL-016`, `W037-OBL-017`, all open-lane audit rows |

The dependency graph makes `calc-ubd.1` the next ready bead after this ledger closes, because the TraceCalc observable-closure lane is the first direct-evidence gate for the W037 sequence.

## 8. OxFml Watch And Handoff Rules

Current watch and consumed-now rows:

1. W073 aggregate and visualization conditional-formatting metadata remains `typed_rule`-only for `colorScale`, `dataBar`, `iconSet`, `top`, `bottom`, `aboveAverage`, and `belowAverage`.
2. bounded `thresholds` strings remain intentionally ignored for those W073 families.
3. `thresholds` remains an OxFml input only for scalar/operator/expression rule families where threshold text is the rule input.
4. `format_delta` and `display_delta` remain distinct consequence categories; broader display-facing closure is not assumed from semantic-format evidence.
5. format dependency tokens, locale/date-system inputs, and replayable format-sensitive outcomes are formalization hooks only until an exercised OxCalc artifact requires them.
6. ordinary downstream use should target `consumer::runtime`, `consumer::editor`, and `consumer::replay`; public `substrate::...` access is not an ordinary downstream integration contract.
7. host/runtime, table-context, immutable-edit, fixture stand-in, and registered-external packet lanes are converged enough for current planning, but not treated as broad production coordinator API freeze.
8. direct OxFml evaluator re-execution is now exercised for `calc-ubd.4` upstream-host fixtures; this removes direct-evaluator absence for that slice but does not promote pack-grade replay.
9. `LET`/`LAMBDA` remains a narrow OxCalc/OxFml/OxFunc carrier fragment; `calc-ubd.4` exercises two direct rows, while general OxFunc semantic kernels remain out of OxCalc scope.

No OxFml handoff is filed by this bead. A W037 handoff is required only if evidence shows an OxFml-owned evaluator, FEC/F3E, runtime facade, formatting, or fixture/host packet clause is insufficient for an exercised OxCalc artifact.

Post-ledger intake note: the `format_delta`/`display_delta` and formalization-hook rows above were sharpened during the later `calc-ubd.3` intake of OxFml formatting updates. They update the W037 watch/input-contract surface and do not change the original `calc-ubd.2` runtime, replay, proof, model, or promotion evidence.

Post-`calc-ubd.4` note: `w037-direct-oxfml-evaluator-001` exercises the direct OxFml runtime facade, two narrow `LET`/`LAMBDA` carrier rows, and one W073 typed conditional-formatting guard row. The run has 12 upstream-host cases and 0 expectation mismatches. It does not promote pack-grade replay, C5, full optimized/core-engine verification, full Lean/TLA verification, or broad display-facing closure.

Post-`calc-ubd.5` note: `w037-proof-model-closure-001` records the W037 Lean/TLA proof/model closure inventory. It checks 12 Lean files, checks 11 routine TLC configs, scans 0 explicit Lean axioms and 0 Lean `sorry`/`admit` placeholders, and records no full Lean/TLA verification, no Stage 2 policy promotion, no pack-grade replay, no C5 promotion, and no general OxFunc callable-kernel promotion.

## 9. Semantic-Equivalence Statement

This bead adds a W037 residual ledger and updates planning/status surfaces only. It does not change coordinator scheduling, invalidation strategy, dependency graph construction, soft-reference update behavior, recalc behavior, publication semantics, reject policy, overlay lifecycle behavior, TraceCalc semantics, TreeCalc/CoreEngine runtime behavior, Lean/TLA model semantics, pack-decision logic, continuous-assurance runner semantics, or OxFml/OxFunc evaluator behavior.

Observable runtime behavior is invariant under this bead because it introduces no runtime producer, evaluator, coordinator transition, formal theorem, TLA action, fixture expectation, replay expectation, service operation, or pack-promotion change.

## 10. Verification

| Command | Result |
|---|---|
| `scripts/check-worksets.ps1` | passed; `worksets=15`, `beads total=99`, `open=9`, `in_progress=0`, `ready=1`, `blocked=7`, `closed=90` |
| `br dep cycles --json` | passed with `count: 0` |
| `git diff --check` | passed; CRLF normalization warnings only |
| W037 ledger consistency check | passed; 17 distinct `W037-OBL-*` ids detected |

Cargo, Lean, and TLC validation are not required for this bead because it changes planning/spec/status surfaces only and introduces no Rust, Lean, TLA+, fixture, replay, or service semantics.

## 11. Pre-Closure Verification Checklist

| # | Check | Result |
|---|---|---|
| 1 | Spec text and realization notes updated for all in-scope items? | yes; this ledger and W037 status/index surfaces record W037 obligations |
| 2 | Pack expectations updated for affected packs? | yes; pack remains unpromoted and `calc-ubd.8` owns reassessment after direct W037 evidence |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes for carried W036 inputs; this bead emits no new behavior and declares W037 evidence roots |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes; Section 9 states no runtime strategy changed |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes; no exercised OxFml-owned seam insufficiency or handoff trigger is exposed by this ledger |
| 6 | All required tests pass? | yes for this documentation/bead-graph audit scope; see Section 10 |
| 7 | No known semantic gaps remain in declared scope? | yes for this ledger target; all known W036 blockers are mapped to W037 owners and promotion consequences |
| 8 | Completion language audit passed? | yes; broader formalization and promotion objectives remain partial |
| 9 | `WORKSET_REGISTER.md` updated when ordered workset truth changed? | not applicable; ordered workset truth is unchanged |
| 10 | `IN_PROGRESS_FEATURE_WORKLIST.md` updated when feature-map truth changed? | yes; IP-04 carries W037 residual-ledger status |
| 11 | execution-state blocker surface updated? | yes; `.beads/` records `calc-ubd.2` in progress and later closure evidence |

## 12. Completion Claim Self-Audit

| Step | Audit result |
|---|---|
| Scope re-read | pass; `calc-ubd.2` asks for W036 closure residuals and no-promotion blockers to become W037 obligations |
| Gate criteria re-read | pass; every W036 open lane is mapped to W037 evidence, implementation, handoff/watch, deferral, or successor ownership |
| Silent scope reduction check | pass; no full formalization, full verification, pack, continuous, direct OxFml, or Stage 2 promotion is claimed |
| "Looks done but is not" pattern check | pass; this is a ledger/planning bead, not implementation, proof, model, replay, service, or promotion closure |
| Result | pass for the `calc-ubd.2` ledger target |

## 13. Three-Axis Report

- execution_state: `calc-ubd.2_residual_full_verification_ledger_validated`
- scope_completeness: `scope_partial`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- open_lanes:
  - `calc-ubd.1` is the next ready W037 bead
  - `calc-ubd.3` through `calc-ubd.9` remain blocked by the sequential W037 path
  - full Lean/TLA verification remains open
  - full TraceCalc oracle coverage remains open
  - full optimized/core-engine verification and fully independent evaluator diversity remain open
  - direct OxFml evaluator re-execution and `LET`/`LAMBDA` seam evidence remain open
  - Stage 2 deterministic replay and partition promotion criteria remain open
  - pack-grade replay, C5, operated continuous-assurance service, operated continuous cross-engine differential service, and enforcing alert/quarantine service remain unpromoted
