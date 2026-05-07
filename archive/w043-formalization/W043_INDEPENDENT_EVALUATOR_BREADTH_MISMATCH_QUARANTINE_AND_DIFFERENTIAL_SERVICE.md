# W043 Independent Evaluator Breadth Mismatch Quarantine And Differential Service

Status: `spec_drafted_with_checked_evidence`

Bead: `calc-2p3.7`

Run id: `w043-independent-evaluator-breadth-mismatch-quarantine-differential-service-001`

## Purpose

This packet deepens the W043 diversity lane after the W043 operated-assurance, retained-history, retained-witness, retention-SLO, and alert/quarantine packet.

The narrow new evidence is an extended independent named-reference model evaluator slice that checks simple model recalculation without using TraceCalc, optimized/core, TreeCalc, OxFml, or OxFunc evaluator kernels. It extends the W042 four-case model with fan-in checksum and mismatch-routing control cases, then binds W043 operated-assurance service rows, W043 Stage 2 service blockers, mismatch-authority rows, current OxFml W073 typed-only formatting intake, and exact no-promotion blockers for operated differential and mismatch quarantine service behavior.

The packet does not promote full independent evaluator breadth, operated cross-engine differential service, mismatch quarantine service, broad OxFml display/publication, public migration, callable metadata projection, callable carrier sufficiency, registered-external callable projection, provider-failure/callable-publication semantics, pack-grade replay, C5, Stage 2 policy, release-grade verification, or general OxFunc kernels.

## Evidence Surfaces

| Artifact | Purpose |
|---|---|
| `archive/test-runs-core-engine-w038-w045/diversity-seam/w043-independent-evaluator-breadth-mismatch-quarantine-differential-service-001/run_summary.json` | records row counts, no-promotion flags, service blockers, and W073 guard retention |
| `archive/test-runs-core-engine-w038-w045/diversity-seam/w043-independent-evaluator-breadth-mismatch-quarantine-differential-service-001/source_evidence_index.json` | 17 source rows binding W043 obligations, W073 intake, W042 diversity evidence, W043 Stage 2 evidence, and W043 operated-assurance evidence |
| `archive/test-runs-core-engine-w038-w045/diversity-seam/w043-independent-evaluator-breadth-mismatch-quarantine-differential-service-001/w043_independent_reference_model_implementation.json` | 6 independent reference-model cases and 6 matches |
| `archive/test-runs-core-engine-w038-w045/diversity-seam/w043-independent-evaluator-breadth-mismatch-quarantine-differential-service-001/w043_independent_evaluator_breadth_register.json` | 13 independent-evaluator breadth rows |
| `archive/test-runs-core-engine-w038-w045/diversity-seam/w043-independent-evaluator-breadth-mismatch-quarantine-differential-service-001/w043_cross_engine_differential_service_register.json` | 12 cross-engine differential and service-classification rows |
| `archive/test-runs-core-engine-w038-w045/diversity-seam/w043-independent-evaluator-breadth-mismatch-quarantine-differential-service-001/w043_mismatch_quarantine_authority_router.json` | 12 mismatch-authority rows |
| `archive/test-runs-core-engine-w038-w045/diversity-seam/w043-independent-evaluator-breadth-mismatch-quarantine-differential-service-001/w043_exact_diversity_blocker_register.json` | 8 exact diversity, operated-service, mismatch, and release-grade blockers |
| `archive/test-runs-core-engine-w038-w045/diversity-seam/w043-independent-evaluator-breadth-mismatch-quarantine-differential-service-001/promotion_decision.json` | explicit no-promotion decision and semantic-equivalence statement |
| `archive/test-runs-core-engine-w038-w045/diversity-seam/w043-independent-evaluator-breadth-mismatch-quarantine-differential-service-001/validation.json` | validation counts and zero failures |

## Result

The run records:

1. 17 source evidence rows.
2. 6 independent reference-model cases.
3. 6 independent reference-model matches.
4. 13 independent-evaluator breadth rows.
5. 12 cross-engine differential and service-classification rows.
6. 12 mismatch-authority rows.
7. 21 accepted boundary rows.
8. 9 service-blocked rows.
9. 8 exact blockers.
10. 0 failed rows.

The independent reference model evaluates:

1. `X:=2`, `Y:=X*20`, `Z:=X+Y`, yielding `Z=42`.
2. `X:=3`, `Y:=X*20`, `Z:=X+Y`, yielding `Z=63`.
3. a row/column gate case with `IF(Sum>10,Sum*2,0)`, yielding `24`.
4. an edit-delta case that records the `Z` delta from `X=2` to `X=3`, yielding `21`.
5. a fan-in checksum case that combines top/left/bias rows and yields `37`.
6. a mismatch-routing control case that detects `Actual=13` versus `Oracle=14` and yields `Quarantine=1`.

## OxFml W073 Intake

The latest OxFml formatting update remains incorporated as a guard:

1. W073 remains a direct typed-only metadata contract for `colorScale`, `dataBar`, `iconSet`, `top`, `bottom`, `aboveAverage`, and `belowAverage`.
2. W072 bounded `thresholds` strings are intentionally ignored for those aggregate and visualization families.
3. `thresholds` remains meaningful for scalar/operator/expression families where threshold text is the actual rule input.
4. OxFml old-string non-interpretation evidence is carried as a no-fallback guard.
5. W043.7 does not construct conditional-formatting requests and does not require an OxCalc core-engine code change.
6. Public migration and typed request-construction uptake remain owned by `calc-2p3.8`.

## Exact Remaining Blockers

1. `w043_diversity.full_independent_evaluator_breadth_absent`
2. `w043_diversity.operated_cross_engine_differential_service_absent`
3. `w043_diversity.mismatch_quarantine_service_absent`
4. `w043_diversity.stage2_operated_differential_dependency_absent`
5. `w043_diversity.retained_witness_attachment_service_absent`
6. `w043_diversity.oxfml_callable_breadth_dependency_absent`
7. `w043_diversity.pack_grade_replay_governance_dependency_absent`
8. `w043_diversity.release_grade_promotion_authority_absent`

## Semantic-Equivalence Statement

This W043 diversity runner extends the bounded independent named-reference model, binds W043 operated-assurance and Stage 2 evidence, and classifies diversity, differential-service, mismatch-quarantine, and spec-evolution authority rows only.

It does not change evaluator kernels used by OxCalc, coordinator scheduling, recalc, publication, replay, pack, service operation, TraceCalc, TreeCalc, OxFml, OxFunc, Lean, or TLA semantics. Observable engine behavior is invariant under this packet. The independent reference-model rows are evidence for the declared W043.7 slice only; they are not a full evaluator, operated service, pack-grade, C5, Stage 2, or release-grade promotion.

## Validation

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | passed |
| `cargo test -p oxcalc-tracecalc diversity_seam_runner_binds_w043_reference_model_without_service_promotion -- --nocapture` | passed; 1 focused test |
| `cargo run -p oxcalc-tracecalc-cli -- diversity-seam w043-independent-evaluator-breadth-mismatch-quarantine-differential-service-001` | passed; emitted 17 source rows, 13 diversity rows, 12 OxFml seam-watch rows, 8 exact blockers, 0 failed rows |
| JSON parse for `archive/test-runs-core-engine-w038-w045/diversity-seam/w043-independent-evaluator-breadth-mismatch-quarantine-differential-service-001/*.json` | passed |
| `cargo test -p oxcalc-tracecalc diversity_seam -- --nocapture` | passed; 6 diversity-seam tests |
| `cargo test -p oxcalc-tracecalc` | passed; 64 tests plus doctests |
| `scripts/check-worksets.ps1` | passed; worksets=21, beads total=163, open=4, in_progress=0, ready=1, blocked=2, closed=159 |
| `br ready --json` | passed; `calc-2p3.8` ready |
| `br dep cycles --json` | passed; 0 cycles |
| `git diff --check` | passed with CRLF normalization warnings only |

## Status Report

- execution_state: `calc-2p3.7_independent_reference_model_diversity_validated`
- scope_completeness: `scope_partial`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- open_lanes:
  - full independent evaluator breadth remains blocked
  - operated cross-engine differential service remains blocked
  - mismatch quarantine service remains blocked
  - retained-witness attachment to mismatch quarantine and pack governance remains blocked
  - Stage 2 operated differential dependency remains blocked
  - broad OxFml display/publication, public migration, callable carrier sufficiency, callable metadata projection, registered-external callable projection, and provider-failure/callable-publication semantics remain owned by `calc-2p3.8`
  - pack-grade replay governance and C5 reassessment remain owned by `calc-2p3.9`
  - release-grade verification decision remains owned by `calc-2p3.10`
  - general OxFunc kernels remain outside OxCalc formalization scope except for the narrow `LET`/`LAMBDA` carrier seam

## Pre-Closure Verification Checklist

| # | Check | Result |
|---|---|---|
| 1 | Spec text and realization notes updated for all in-scope items? | yes; this packet, W043 README/status surfaces, feature map, runner, test, and generated diversity artifacts record the slice |
| 2 | Pack expectations updated for affected packs? | yes; pack-grade replay and C5 remain unpromoted and `calc-2p3.9` owns reassessment |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes; W043.7 emits independent reference-model, independent breadth, cross-engine, mismatch-authority, blocker, decision, validation, and summary artifacts |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes; no runtime strategy changed |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes; W073 is carried as typed-only guard and no OxCalc handoff trigger exists |
| 6 | All required tests pass? | yes; see Validation |
| 7 | No known semantic gaps remain in declared scope? | yes for the W043.7 evidence target; broader exact blockers are explicit |
| 8 | Completion language audit passed? | yes; no full independent evaluator, operated cross-engine differential service, mismatch quarantine service, pack/C5, Stage 2, release-grade, OxFml breadth, callable, or general OxFunc promotion is claimed |
| 9 | `WORKSET_REGISTER.md` updated when ordered workset truth changed? | yes; no ordered workset-truth change in this bead |
| 10 | `IN_PROGRESS_FEATURE_WORKLIST.md` updated when feature-map truth changed? | yes; IP-04 records the W043 diversity update |
| 11 | execution-state blocker surface updated? | yes; `.beads/` records `calc-2p3.7` closed and `calc-2p3.8` ready |

## Completion Claim Self-Audit

| Step | Audit result |
|---|---|
| Scope re-read | pass; `calc-2p3.7` asks for wider independent evaluator authority and operated differential/mismatch-quarantine evidence where feasible, not release-grade diversity promotion |
| Gate criteria re-read | pass; projections over TraceCalc, optimized/core, or shared fixtures are not promoted as fully independent breadth |
| Silent scope reduction check | pass; full evaluator breadth, operated differential service, mismatch quarantine, OxFml breadth, pack/C5, Stage 2, and release-grade blockers remain explicit |
| "Looks done but is not" pattern check | pass; the reference-model evaluator is a bounded evidence slice and not a full independent evaluator |
| Result | pass for the `calc-2p3.7` target |
