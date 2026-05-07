# W041 Operated Assurance Retained-History And Alert-Dispatch Service Tranche

Status: `spec_drafted_with_checked_replay_evidence`

Bead: `calc-sui.6`

Run id: `w041-operated-assurance-retained-history-alert-dispatch-001`

## Purpose

This packet records the W041 operated-assurance tranche after the W041 Stage 2 analyzer and pack-equivalence evidence.

The narrow target is to replace the previous file-backed-only service disposition with a stronger service-envelope evidence packet where feasible:

1. a runnable operated-assurance runner branch,
2. a service-readable envelope and run-queue manifest contract,
3. a retained-history query API contract,
4. replay-correlation rows,
5. retained-witness lifecycle rows,
6. local alert/quarantine dispatch evaluation,
7. exact service blockers.

This packet does not promote an operated continuous-assurance service, retained-history service, retained-witness lifecycle service, external alert/quarantine dispatcher, operated cross-engine differential service, pack-grade replay, C5, Stage 2 production policy, or release-grade verification.

## OxFml Formatting Intake

The latest OxFml formatting update was reviewed before this packet was emitted.

Current W041.6 consequence:

1. W073 remains a typed-only input contract for aggregate and visualization conditional-formatting metadata.
2. `VerificationConditionalFormattingRule.typed_rule` is the accepted metadata source for `colorScale`, `dataBar`, `iconSet`, `top`, `bottom`, `aboveAverage`, and `belowAverage`.
3. W072 bounded `thresholds` strings are intentionally ignored for those families.
4. OxFml now has old-string non-interpretation evidence for visualization strings and aggregate option strings.
5. This packet does not construct a conditional-formatting request payload, so no OxFml handoff is filed.
6. The W073 guard is carried through the source index, alert-dispatch rows, service-readiness rows, and no-promotion decision.

## Artifact Surface

| Artifact | Purpose |
| --- | --- |
| `archive/test-runs-core-engine-w038-w045/operated-assurance/w041-operated-assurance-retained-history-alert-dispatch-001/run_summary.json` | W041 summary, row counts, and no-promotion flags |
| `archive/test-runs-core-engine-w038-w045/operated-assurance/w041-operated-assurance-retained-history-alert-dispatch-001/source_evidence_index.json` | 12 source rows binding W041 obligations, W073 intake, W040 service artifacts, W041 Stage 2 blockers, W040 pack decision, and retained-witness lifecycles |
| `archive/test-runs-core-engine-w038-w045/operated-assurance/w041-operated-assurance-retained-history-alert-dispatch-001/w041_operated_service_envelope.json` | 8 service-envelope rows and service/run-queue boundary facts |
| `archive/test-runs-core-engine-w038-w045/operated-assurance/w041-operated-assurance-retained-history-alert-dispatch-001/w041_retained_history_service_query.json` | 25 retained-history rows, 7 query rows, and 5 replay-correlation rows |
| `archive/test-runs-core-engine-w038-w045/operated-assurance/w041-operated-assurance-retained-history-alert-dispatch-001/w041_retained_witness_lifecycle_register.json` | 4 retained-witness lifecycle rows |
| `archive/test-runs-core-engine-w038-w045/operated-assurance/w041-operated-assurance-retained-history-alert-dispatch-001/w041_alert_dispatch_service_register.json` | 18 local alert/quarantine dispatch rows and clean decisions |
| `archive/test-runs-core-engine-w038-w045/operated-assurance/w041-operated-assurance-retained-history-alert-dispatch-001/w041_cross_engine_service_register.json` | file-backed cross-engine substrate and W041 Stage 2 service dependency |
| `archive/test-runs-core-engine-w038-w045/operated-assurance/w041-operated-assurance-retained-history-alert-dispatch-001/w041_service_readiness_register.json` | 17 readiness criteria, 12 satisfied or boundary-satisfied, 5 blocked |
| `archive/test-runs-core-engine-w038-w045/operated-assurance/w041-operated-assurance-retained-history-alert-dispatch-001/w041_exact_service_blocker_register.json` | 5 exact remaining service blockers |
| `archive/test-runs-core-engine-w038-w045/operated-assurance/w041-operated-assurance-retained-history-alert-dispatch-001/promotion_decision.json` | service-envelope evidence accepted; service and pack promotions remain false |
| `archive/test-runs-core-engine-w038-w045/operated-assurance/w041-operated-assurance-retained-history-alert-dispatch-001/validation.json` | validation status and counts |

## Evidence Result

The W041.6 runner emits:

1. 12 source evidence rows.
2. 8 service-envelope rows.
3. 25 retained-history rows.
4. 7 retained-history query-register rows.
5. 5 replay-correlation rows.
6. 4 retained-witness lifecycle rows.
7. 18 evaluated alert/quarantine rows.
8. 0 quarantine decisions.
9. 0 alert decisions.
10. 17 service-readiness criteria.
11. 5 exact service blockers.
12. 0 failed rows.

New or sharpened evidence facts:

1. file-backed service envelope is present,
2. service-readable run queue manifest is present,
3. retained-history query API contract is present,
4. replay-correlation index is present,
5. retained-witness lifecycle register is present,
6. retention SLO policy is declared,
7. retention SLO enforcement is absent,
8. local alert-dispatch service contract is evaluated,
9. external alert dispatcher contract is represented but not operated.

Exact remaining blockers:

1. `service.operated_scheduler_service_endpoint_absent`
2. `service.retained_history_service_endpoint_absent`
3. `service.external_alert_dispatcher_absent`
4. `service.operated_cross_engine_differential_absent`
5. `service.retention_slo_retained_witness_pack_governance_absent`

## Promotion Consequence

This evidence narrows W040 service-artifact gaps by adding a service envelope, query API contract, retained-witness lifecycle register, and alert-dispatch service register.

The following remain unpromoted:

1. operated continuous-assurance service,
2. retained-history service,
3. retained-witness lifecycle service,
4. external alert/quarantine dispatcher,
5. operated cross-engine differential service,
6. Stage 2 production policy,
7. pack-grade replay,
8. `cap.C5.pack_valid`,
9. release-grade verification.

## Semantic Equivalence Statement

This packet adds W041 operated-assurance runner logic, emitted service-envelope artifacts, retained-history query evidence, replay-correlation rows, retained-witness lifecycle rows, local alert-dispatch rows, readiness rows, exact blocker rows, tests, and status text only.

It does not change coordinator scheduling, invalidation strategy, dependency graph construction, soft-reference update behavior, recalc behavior, publication semantics, reject policy, overlay lifecycle behavior, TraceCalc execution semantics, TreeCalc/CoreEngine runtime behavior, Lean/TLA model semantics, pack runtime behavior, OxFml/OxFunc evaluator behavior, or external service behavior.

Observable runtime behavior is invariant under this packet. The changed observable artifacts are W041.6 service-envelope evidence files and the operated-assurance runner test/CLI path.

## Validation

| Command | Result |
| --- | --- |
| `cargo fmt --all -- --check` | passed |
| `cargo test -p oxcalc-tracecalc operated_assurance -- --nocapture` | passed; 4 tests |
| `cargo run -p oxcalc-tracecalc-cli -- operated-assurance w041-operated-assurance-retained-history-alert-dispatch-001` | passed; emitted 12 source rows, 25 history rows, 18 alert rules, 5 exact blockers, and 0 failed rows |
| `cargo test -p oxcalc-tracecalc` | passed; 49 tests |
| `cargo test -p oxcalc-tracecalc-cli` | passed |
| JSON parse for `archive/test-runs-core-engine-w038-w045/operated-assurance/w041-operated-assurance-retained-history-alert-dispatch-001/*.json` | passed |
| `scripts/check-worksets.ps1` | passed |
| `br ready --json` | passed after bead closure; next ready bead is `calc-sui.7` |
| `br dep cycles --json` | passed; 0 cycles |
| `git diff --check` | passed; CRLF normalization warnings only |

## Status Report

- execution_state: `calc-sui.6_operated_assurance_service_envelope_validated_no_promotion`
- scope_completeness: `scope_partial`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- open_lanes:
  - `calc-sui.7` independent evaluator breadth and operated differential service tranche
  - operated recurring scheduler/service endpoint remains open
  - retained-history service endpoint remains open
  - external alert/quarantine dispatcher remains open
  - operated cross-engine differential service remains open
  - retention SLO enforcement, retained-witness lifecycle service, and pack-grade replay governance remain open
  - broad OxFml display/publication, public migration, callable metadata, callable carrier sufficiency, pack/C5, Stage 2 policy, and release-grade verification remain unpromoted

## Pre-Closure Verification Checklist

| Check | Result |
| --- | --- |
| Workset and bead ids are explicit | yes: `W041`, `calc-sui.6` |
| Required artifacts exist | yes: W041.6 operated-assurance packet artifacts are present |
| Checked/replay evidence exists for changed classification | yes: runner test and generated operated-assurance packet |
| Service boundaries are explicit | yes: service envelope and query contract are separated from operated service promotion |
| No declared gap is match-promoted | yes: service endpoint, retained-history service, external dispatcher, operated cross-engine service, and retention/pack governance blockers remain exact |
| Semantic-equivalence statement is present | yes |
| Cross-repo impact assessed | yes; W073 formatting update is a carried input-contract guard and no handoff is triggered |

## Completion Claim Self-Audit

| Audit Item | Result |
| --- | --- |
| Claim is limited to `calc-sui.6` target | yes |
| Scaffolding is not reported as implementation | yes |
| Spec text has checked/replay evidence | yes: runner test and generated operated-assurance packet |
| Cross-repo handoff is not treated as closure | yes; W073 remains an input-contract guard and broad OxFml closure remains under `calc-sui.8` |
| Uncertain lanes default to in-progress | yes; operated service, external dispatcher, cross-engine service, retention SLO enforcement, pack/C5, and release-grade blockers are retained |
| Strategy-change equivalence statement is present | yes |
