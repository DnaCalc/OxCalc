# W036 Residual Coverage And Promotion-Blocker Ledger

Status: `calc-rqq.1_residual_coverage_ledger_validated`
Workset: `W036`
Parent epic: `calc-rqq`
Bead: `calc-rqq.1`

## 1. Purpose

This ledger converts the W035 closure residuals, no-promotion blockers, and active-objective gaps into W036 obligations.

W036 begins from a stronger W035 floor, but not from full verification. W035 made the remaining gaps explicit: TraceCalc coverage is bounded, optimized/core-engine conformance still has declared gaps, Lean/TLA artifacts are checked slices rather than total proof, independent evaluator diversity is incomplete, continuous assurance is a gate rather than an operated lane, and pack/Stage 2 remain unpromoted.

This ledger prevents those residuals from staying only in prose.

## 2. Authority Inputs Reviewed

| Input | Role in W036 |
|---|---|
| `docs/spec/core-engine/w035-formalization/W035_CLOSURE_AUDIT_AND_SUCCESSOR_PACKET.md` | canonical W035 closure and successor packet |
| `docs/worksets/W036_CORE_FORMALIZATION_VERIFICATION_CLOSURE_EXPANSION.md` | W036 workset scope and gate |
| `docs/spec/core-engine/w035-formalization/W035_RESIDUAL_PROOF_OBLIGATION_AND_SPEC_EVOLUTION_LEDGER.md` | W035 residual disposition ledger |
| `docs/spec/core-engine/w035-formalization/W035_TRACECALC_ORACLE_MATRIX_EXPANSION.md` | W035 oracle matrix source and uncovered rows |
| `docs/spec/core-engine/w035-formalization/W035_IMPLEMENTATION_CONFORMANCE_HARDENING.md` | W035 implementation-conformance dispositions |
| `docs/spec/core-engine/w035-formalization/W035_LEAN_ASSUMPTION_DISCHARGE_AND_SEAM_PROOF_MAP.md` | W035 Lean proof-map source |
| `docs/spec/core-engine/w035-formalization/W035_TLA_NON_ROUTINE_EXPLORATION_AND_SCHEDULER_PRECONDITIONS.md` | W035 TLA/scheduler precondition source |
| `docs/spec/core-engine/w035-formalization/W035_CONTINUOUS_ASSURANCE_AND_CROSS_ENGINE_DIFFERENTIAL_GATE.md` | W035 continuous-assurance source |
| `docs/spec/core-engine/w035-formalization/W035_PACK_CAPABILITY_AND_STAGE2_READINESS_REASSESSMENT.md` | W035 pack/Stage 2 readiness source |
| `docs/test-runs/core-engine/tracecalc-reference-machine/w035-tracecalc-oracle-matrix-001/` | deterministic W035 TraceCalc matrix evidence |
| `docs/test-runs/core-engine/implementation-conformance/w035-implementation-conformance-hardening-001/` | deterministic W035 gap-disposition evidence |
| `docs/test-runs/core-engine/continuous-assurance/w035-continuous-assurance-gate-001/` | deterministic W035 continuous-gate evidence |
| `docs/test-runs/core-engine/pack-capability/w035-pack-stage2-readiness-001/` | deterministic W035 pack/Stage 2 no-promotion decision |
| `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md` | current inbound OxFml observation ledger |

## 3. Evidence Roots Declared

W036 may emit artifacts under these roots:

1. `docs/spec/core-engine/w036-formalization/`
2. `docs/test-runs/core-engine/tracecalc-reference-machine/w036-*`
3. `docs/test-runs/core-engine/treecalc-local/w036-*`
4. `docs/test-runs/core-engine/independent-conformance/w036-*`
5. `docs/test-runs/core-engine/cross-engine-differential/w036-*`
6. `docs/test-runs/core-engine/continuous-assurance/w036-*`
7. `docs/test-runs/core-engine/pack-capability/w036-*`
8. `formal/lean/OxCalc/CoreEngine/W036*.lean`
9. `formal/tla/CoreEngineW036*.tla`
10. `formal/tla/CoreEngineW036*.cfg`

Checked-in evidence must use repo-relative paths. Validation runs must not mutate prior W033, W034, or W035 baselines unless a later bead explicitly regenerates and supersedes them.

## 4. Promotion Limits

W036 starts with these limits:

1. `cap.C5.pack_valid` is not promoted.
2. continuous scale assurance is not promoted.
3. continuous cross-engine differential service is not promoted.
4. full TraceCalc oracle coverage is not claimed.
5. full optimized/core-engine verification is not claimed.
6. fully independent evaluator implementation diversity is not claimed.
7. full Lean verification is not claimed.
8. full TLA+ verification is not claimed.
9. Stage 2 scheduler policy is not promoted.
10. W073 conditional-formatting typed metadata remains watch/input-contract evidence until a W036 artifact constructs that payload family.

Any later W036 promotion candidate must include machine-readable evidence, a semantic-equivalence statement, and an updated pack/capability decision.

## 5. W035 Promotion-Blocker Disposition

| W035 blocker | W036 disposition |
|---|---|
| `pack.grade.program_scope.unproven` | carry to `W036-OBL-016` pack/program governance |
| `pack.grade.direct_oxfml_evaluator_reexecution_absent` | carry to `W036-OBL-017` OxFml seam watch/direct-evaluator evidence |
| `pack.grade.independent_conformance_declared_gaps` | carry to `W036-OBL-003` through `W036-OBL-008` implementation gap dispositions |
| `pack.grade.continuous_diff_suite_absent` | carry to `W036-OBL-013` cross-engine differential harness |
| `pack.grade.fully_independent_evaluator_absent` | carry to `W036-OBL-012` independent evaluator diversity |
| `pack.grade.treecalc_c4_c5_unproven` | carry to `W036-OBL-016` pack capability |
| `pack.grade.w035_formal_slices_bounded_not_full_verification` | carry to `W036-OBL-009` and `W036-OBL-010` Lean/TLA expansion |
| `pack.grade.stage2_scheduler_preconditions_not_satisfied` | carry to `W036-OBL-011` Stage 2 partition/replay equivalence |
| `pack.grade.tracecalc_oracle_matrix_not_full_coverage` | carry to `W036-OBL-001` and `W036-OBL-002` TraceCalc coverage closure |
| `pack.grade.implementation_gap_dispositions_remain` | carry to `W036-OBL-003` through `W036-OBL-008` |
| `pack.grade.optimized_core_engine_conformance_not_full` | carry to `W036-OBL-003` through `W036-OBL-008` |
| `pack.grade.continuous_assurance_gate_not_running_service` | carry to `W036-OBL-014` continuous assurance operation |
| `pack.grade.continuous_cross_engine_diff_service_absent` | carry to `W036-OBL-013` and `W036-OBL-014` |
| `pack.grade.program_grade_replay_governance_not_reached` | carry to `W036-OBL-016` |
| `pack.grade.retained_witness_promotion_not_shared_program_grade` | carry to `W036-OBL-016` |
| `pack.grade.continuous_scale_assurance_unpromoted` | carry to `W036-OBL-014` |
| `pack.grade.stage2_scheduler_policy_unpromoted` | carry to `W036-OBL-011` |
| `pack.grade.pack_c5_no_promotion_after_w035_reassessment` | carry to `W036-OBL-016` |
| `pack.grade.w035_closure_audit_not_yet_recorded` | retired by `W035_CLOSURE_AUDIT_AND_SUCCESSOR_PACKET.md`; W036 owns a new closure audit in `calc-rqq.9` |
| `continuous.no_scheduled_regression_runner` | carry to `W036-OBL-014` |
| `continuous.no_cross_engine_diff_service` | carry to `W036-OBL-013` |
| `continuous.no_history_window_for_regression_thresholds` | carry to `W036-OBL-014` |
| `continuous.no_alerting_or_quarantine_policy` | carry to `W036-OBL-014` |
| `continuous.performance_not_correctness_proof` | retained as W036 guardrail for `W036-OBL-014` and `W036-OBL-016` |
| `continuous.independent_evaluator_diversity_not_full` | carry to `W036-OBL-012` |
| `continuous.pack_c5_not_promoted` | carry to `W036-OBL-016` |
| `continuous.stage2_scheduler_not_promoted` | carry to `W036-OBL-011` |
| `continuous.formal_evidence_bounded_not_full_verification` | carry to `W036-OBL-009` and `W036-OBL-010` |

## 6. Residual Obligation Matrix

| Obligation id | Area | W035 floor | W036 owner | Required W036 disposition |
|---|---|---|---|---|
| `W036-OBL-001` | TraceCalc coverage closure criteria | 17 matrix rows, 15 covered, 2 classified uncovered | `calc-rqq.2` | define machine-readable current observable-semantics coverage criteria and no-loss crosswalk to W033-W035 rows |
| `W036-OBL-002` | TraceCalc uncovered rows | callable full OxFunc semantics and multi-reader/other non-core surfaces are classified, not covered | `calc-rqq.2`, `calc-rqq.4`, `calc-rqq.5` | cover, exclude with authority, or defer each uncovered row with owner and promotion consequence |
| `W036-OBL-003` | dynamic dependency bind projection | W035 TraceCalc covers dynamic switch publish, TreeCalc-local still rejects residual carrier | `calc-rqq.3` | implement, harness, or retain blocker for dynamic bind projection |
| `W036-OBL-004` | host-sensitive lambda effect | W035 classifies host-sensitive lambda effects as OxFunc/OxFml-owned beyond carrier scope | `calc-rqq.3`, `calc-rqq.4` | decide whether W036 needs OxCalc-local carrier proof, OxFml handoff/watch, or explicit OxFunc-opaque deferral |
| `W036-OBL-005` | dynamic dependency negative/shape-update projection | W035 validates dynamic negative/release evidence but keeps TreeCalc-local projection gap | `calc-rqq.3` | implement, harness, or retain blocker for dynamic negative/shape-update projection |
| `W036-OBL-006` | stale snapshot-fence projection | W035 TraceCalc covers snapshot-fence rejection; TreeCalc-local lacks counterpart | `calc-rqq.3`, `calc-rqq.5` | decide optimized/coordinator conformance surface and replay evidence |
| `W036-OBL-007` | capability-view fence projection | W035 TraceCalc covers capability-view fence rejection; local fixture counterpart remains absent | `calc-rqq.3`, `calc-rqq.5` | decide optimized/coordinator conformance surface and replay evidence |
| `W036-OBL-008` | callable metadata projection | W035 keeps callable identity metadata as implementation-work deferral | `calc-rqq.3`, `calc-rqq.4` | implement metadata projection, prove carrier sufficiency, or retain blocker |
| `W036-OBL-009` | Lean theorem coverage | W035 classifies local proof rows and assumptions | `calc-rqq.4` | expand checked theorem families and produce proof inventory showing proved, assumed, opaque, and deferred facts |
| `W036-OBL-010` | TLA model coverage | W035 checks bounded non-routine overlay/scheduler configs | `calc-rqq.5` | replace or refine abstract gates where practical and record TLC limits and promotion blockers |
| `W036-OBL-011` | Stage 2 partition/replay equivalence | W035 uses abstract partition-soundness input and no replay equivalence | `calc-rqq.5`, `calc-rqq.8` | provide concrete bounded partition model and semantic-equivalence replay criteria or retain no-promotion blockers |
| `W036-OBL-012` | independent evaluator diversity | W035 has TreeCalc/CoreEngine projection, not fully independent implementation diversity | `calc-rqq.6` | define diversity criteria and classify existing/new engines against them |
| `W036-OBL-013` | cross-engine differential harness | W035 defines differential gate but not service/harness breadth | `calc-rqq.6` | emit machine-readable cross-engine differentials with declared gaps and diversity limits |
| `W036-OBL-014` | continuous assurance operation | W035 defines schedule lanes but no operated service/history/alerting | `calc-rqq.7` | define or simulate multi-run history, thresholds, quarantine/alert policy, and semantic-first acceptance |
| `W036-OBL-015` | OxFml W073 formatting input contract | W035 watch row says typed-rule-only aggregate/visualization metadata | all W036 beads where exercised | use `typed_rule` for W073 families; file handoff only on concrete mismatch |
| `W036-OBL-016` | pack-grade replay and capability | W035 pack decision has 19 no-promotion blockers and `cap.C4.distill_valid` ceiling | `calc-rqq.8` | reassess C5 only after W036 evidence; promote only with direct pack-grade replay evidence |
| `W036-OBL-017` | OxFml direct evaluator/seam evidence | W035 direct fixture bridge is projection evidence only | `calc-rqq.6`, `calc-rqq.8` | decide whether direct OxFml evaluator re-execution is needed for pack-grade proof or stays watch/deferred |
| `W036-OBL-018` | spec evolution discipline | W035 treats specs and current implementation as evidence surfaces | every W036 bead | patch specs, create implementation beads, file handoffs, or record deferrals when evidence changes understanding |
| `W036-OBL-019` | evidence non-mutation | W033-W035 checked-in baselines exist | every W036 bead | declare new run ids and avoid accidental mutation of older baselines |
| `W036-OBL-020` | active-objective completion audit | W035 audit keeps broader objective in progress | `calc-rqq.9` | map every active objective requirement to direct artifacts before any full-completion claim |

## 7. Bead Mapping

| Bead | Primary obligations |
|---|---|
| `calc-rqq.2` | `W036-OBL-001`, `W036-OBL-002` |
| `calc-rqq.3` | `W036-OBL-003`, `W036-OBL-004`, `W036-OBL-005`, `W036-OBL-006`, `W036-OBL-007`, `W036-OBL-008` |
| `calc-rqq.4` | `W036-OBL-002`, `W036-OBL-004`, `W036-OBL-008`, `W036-OBL-009`, `W036-OBL-015` |
| `calc-rqq.5` | `W036-OBL-002`, `W036-OBL-006`, `W036-OBL-007`, `W036-OBL-010`, `W036-OBL-011` |
| `calc-rqq.6` | `W036-OBL-012`, `W036-OBL-013`, `W036-OBL-017` |
| `calc-rqq.7` | `W036-OBL-014` |
| `calc-rqq.8` | `W036-OBL-011`, `W036-OBL-016`, `W036-OBL-017` |
| `calc-rqq.9` | `W036-OBL-018`, `W036-OBL-019`, `W036-OBL-020`, all open-lane audit rows |

### calc-rqq.2 Disposition Update

`calc-rqq.2` now records the W036 TraceCalc coverage criteria and matrix expansion in `W036_TRACECALC_COVERAGE_CLOSURE_CRITERIA_AND_MATRIX_EXPANSION.md`.

Evidence run `w036-tracecalc-coverage-closure-001` emits 32 matrix rows, 30 covered rows, 1 classified uncovered row, 1 excluded row, 0 failed/missing rows, and 0 no-loss crosswalk gaps. The multi-reader overlay release-order row remains uncovered and routed to `calc-rqq.5`; the full OxFunc LAMBDA semantic-kernel row is excluded from the OxCalc TraceCalc profile and recorded for the `calc-rqq.4` boundary inventory. No full TraceCalc oracle claim is made.

### calc-rqq.3 Disposition Update

`calc-rqq.3` now records the W036 optimized/core-engine conformance closure plan and first-fix harness evidence in `W036_OPTIMIZED_CORE_ENGINE_CONFORMANCE_CLOSURE_PLAN_AND_FIRST_FIXES.md`.

Evidence run `w036-implementation-conformance-closure-001` emits 6 closure action rows, 2 harness first-fix rows, 4 blocker-routed rows, 0 match-promoted rows, and 0 failed rows. Dynamic dependency bind and dynamic negative/shape-update rows are bound to W036 harness evidence but remain non-matches. LET/LAMBDA host effect and callable metadata rows route to `calc-rqq.4`; snapshot-fence and capability-view fence rows route to `calc-rqq.5`. Full optimized/core-engine verification remains unpromoted.

## 8. OxFml Watch And Handoff Rules

Current watch rows:

1. W073 typed conditional-formatting metadata remains `typed_rule`-only for `colorScale`, `dataBar`, `iconSet`, `top`, `bottom`, `aboveAverage`, and `belowAverage`.
2. W072 bounded `thresholds` strings are intentionally ignored for those W073 families.
3. `thresholds` remains an OxFml input only for scalar/operator/expression rule families where threshold text is the rule input.
4. Runtime facade and replay comparison-view updates remain OxFml-owned inputs unless W036 exposes a concrete mismatch.
5. Direct OxFml evaluator re-execution remains a pack-grade evidence question, not a local OxCalc implementation claim.

No OxFml handoff is filed by this bead. A W036 handoff is required only if evidence shows an OxFml-owned evaluator, FEC/F3E, runtime facade, or formatting clause is insufficient for an exercised OxCalc artifact.

## 9. Semantic-Equivalence Statement

This bead adds a W036 residual ledger and updates planning/status surfaces only. It does not change coordinator scheduling, invalidation strategy, dependency graph construction, soft-reference update behavior, recalc behavior, publication semantics, reject policy, TraceCalc semantics, TreeCalc semantics, Lean/TLA model semantics, pack-decision logic, continuous-assurance runner semantics, or OxFml evaluator behavior.

Observable runtime behavior is invariant under this bead because it introduces no runtime producer, evaluator, coordinator transition, formal theorem, TLA action, or fixture expectation change.

## 10. Verification

| Command | Result |
|---|---|
| `scripts/check-worksets.ps1` | passed; `worksets=14`, `beads total=89`, `open=9`, `in_progress=1`, `ready=0`, `blocked=8`, `closed=79` |
| `br dep cycles --json` | passed; `count=0` |
| `git diff --check` | passed; CRLF normalization warnings only |
| `rg -n "VerificationConditionalFormattingRule\|typed_rule\|conditional-formatting\|conditional_formatting" src docs/spec/core-engine/w036-formalization docs/worksets/W036_CORE_FORMALIZATION_VERIFICATION_CLOSURE_EXPANSION.md docs/IN_PROGRESS_FEATURE_WORKLIST.md` | passed; matches are W036 watch/planning rows plus existing pack-capability watch input string, with no OxCalc request-construction path for W073 payloads |

Cargo, Lean, and TLC validation are not required for this bead because it changes planning/spec/status surfaces only and introduces no Rust, Lean, TLA+, fixture, or replay semantics.

## 11. Pre-Closure Verification Checklist

| # | Check | Result |
|---|---|---|
| 1 | Spec text and realization notes updated for all in-scope items? | yes; this ledger, W036 workset status, spec index, and feature-map surfaces record W036 obligations |
| 2 | Pack expectations updated for affected packs? | yes; pack remains unpromoted and W036 `calc-rqq.8` owns reassessment after successor evidence |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes for carried W035 inputs; this bead emits no new behavior and declares W036 evidence roots |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes; Section 9 states no runtime strategy changed |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes; W073 remains watch/input-contract evidence with no exercised payload mismatch or handoff trigger |
| 6 | All required tests pass? | yes; planning/spec validation commands passed, and no runtime/formal test lane is in scope for this bead |
| 7 | No known semantic gaps remain in declared scope? | yes for this ledger target; all known W035 blockers are mapped to W036 owners or retired when already satisfied |
| 8 | Completion language audit passed? | yes; broader formalization and promotion objectives remain partial |
| 9 | `WORKSET_REGISTER.md` updated when ordered workset truth changed? | not applicable unless W036 ordered truth changes |
| 10 | `IN_PROGRESS_FEATURE_WORKLIST.md` updated when feature-map truth changed? | yes; IP-04 carries W036 residual ledger status |
| 11 | execution-state blocker surface updated? | yes; `.beads/` records `calc-rqq.1` in progress and later closure evidence |

## 12. Completion Claim Self-Audit

| Step | Audit result |
|---|---|
| Scope re-read | pass; `calc-rqq.1` asks for W035 residuals, no-promotion blockers, and active objective gaps to become a W036 obligation ledger |
| Gate criteria re-read | pass; every W035 open lane is mapped to W036 proof, implementation, replay/evidence, watch/handoff, or explicit deferral ownership |
| Silent scope reduction check | pass; no full formalization, full verification, pack, continuous, or Stage 2 promotion is claimed |
| "Looks done but is not" pattern check | pass; this is a ledger/planning bead, not implementation, proof, model, replay, or promotion closure |
| Result | pass for the `calc-rqq.1` ledger target |

## 13. Three-Axis Report

- execution_state: `calc-rqq.1_residual_coverage_ledger_validated`
- scope_completeness: `scope_partial`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- open_lanes:
  - `calc-rqq.2` through `calc-rqq.9` remain open
  - full Lean/TLA verification remains open
  - full TraceCalc oracle coverage remains open
  - full optimized/core-engine verification and fully independent evaluator diversity remain open
  - concrete Stage 2 partition modeling and replay equivalence remain open
  - pack-grade replay, continuous-scale service operation, continuous cross-engine differential service, and Stage 2 policy remain unpromoted
