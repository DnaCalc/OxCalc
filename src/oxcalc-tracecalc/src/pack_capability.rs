#![forbid(unsafe_code)]

//! Post-W033 through W043 pack capability decision packet emission.

use std::fs;
use std::path::Path;

use serde_json::{Value, json};
use thiserror::Error;

const PACK_CAPABILITY_RUN_SUMMARY_SCHEMA_V1: &str = "oxcalc.pack_capability.run_summary.v1";
const PACK_CAPABILITY_EVIDENCE_INDEX_SCHEMA_V1: &str = "oxcalc.pack_capability.evidence_index.v1";
const PACK_CAPABILITY_DECISION_SCHEMA_V1: &str = "oxcalc.pack_capability.decision.v1";
const PACK_CAPABILITY_BUNDLE_SCHEMA_V1: &str = "oxcalc.pack_capability.bundle_manifest.v1";
const PACK_CAPABILITY_VALIDATION_SCHEMA_V1: &str = "oxcalc.pack_capability.validation.v1";

const TRACE_RETAINED_W023_RUN_ID: &str = "w023-sequence3-program-decision";
const POST_W033_OXFML_BRIDGE_RUN_ID: &str = "post-w033-direct-oxfml-fixture-bridge-001";
const POST_W033_LET_LAMBDA_TRACECALC_RUN_ID: &str = "post-w033-let-lambda-carrier-witness-001";
const POST_W033_LET_LAMBDA_TREECALC_RUN_ID: &str = "post-w033-let-lambda-treecalc-witness-001";
const POST_W033_INDEPENDENT_TREECALC_RUN_ID: &str =
    "post-w033-independent-conformance-treecalc-001";
const POST_W033_INDEPENDENT_CONFORMANCE_RUN_ID: &str = "post-w033-independent-conformance-001";
const W034_TRACECALC_RUN_ID: &str = "w034-tracecalc-oracle-deepening-001";
const W034_TREECALC_RUN_ID: &str = "w034-independent-conformance-treecalc-001";
const W034_INDEPENDENT_CONFORMANCE_RUN_ID: &str = "w034-independent-conformance-001";
const W034_FORMAL_ARTIFACTS: &[&str] = &[
    "docs/spec/core-engine/w034-formalization/W034_LEAN_PROOF_FAMILY_DEEPENING.md",
    "docs/spec/core-engine/w034-formalization/W034_TLA_MODEL_FAMILY_AND_CONTENTION_PRECONDITIONS.md",
    "formal/lean/OxCalc/CoreEngine/W034PublicationFences.lean",
    "formal/lean/OxCalc/CoreEngine/W034DependenciesOverlays.lean",
    "formal/lean/OxCalc/CoreEngine/W034LetLambdaReplay.lean",
    "formal/lean/OxCalc/CoreEngine/W034RefinementObligations.lean",
    "formal/tla/CoreEngineW034Interleavings.tla",
    "formal/tla/CoreEngineW034Interleavings.smoke.cfg",
];
const W034_FORMATTING_WATCH_ARTIFACTS: &[&str] = &[
    "docs/spec/core-engine/w034-formalization/W034_RESIDUAL_OBLIGATION_AND_AUTHORITY_LEDGER.md",
    "docs/spec/core-engine/w034-formalization/W034_FORMALIZATION_DEEPENING_PLAN.md",
];
const W035_TRACECALC_ORACLE_MATRIX_RUN_ID: &str = "w035-tracecalc-oracle-matrix-001";
#[cfg(test)]
const W035_IMPLEMENTATION_CONFORMANCE_RUN_ID: &str =
    "w035-implementation-conformance-hardening-001";
#[cfg(test)]
const W035_CONTINUOUS_ASSURANCE_RUN_ID: &str = "w035-continuous-assurance-gate-001";
const W035_FORMAL_ARTIFACTS: &[&str] = &[
    "docs/spec/core-engine/w035-formalization/W035_LEAN_ASSUMPTION_DISCHARGE_AND_SEAM_PROOF_MAP.md",
    "docs/spec/core-engine/w035-formalization/W035_TLA_NON_ROUTINE_EXPLORATION_AND_SCHEDULER_PRECONDITIONS.md",
    "formal/lean/OxCalc/CoreEngine/W035AssumptionDischarge.lean",
    "formal/lean/OxCalc/CoreEngine/W035SeamProofMap.lean",
    "formal/tla/CoreEngineW035NonRoutineInterleavings.tla",
    "formal/tla/CoreEngineW035NonRoutineInterleavings.multi_reader.cfg",
    "formal/tla/CoreEngineW035NonRoutineInterleavings.scheduler_gate.cfg",
    "formal/tla/CoreEngineW035NonRoutineInterleavings.partition_gap.cfg",
];
const W035_FORMATTING_WATCH_ARTIFACTS: &[&str] = &[
    "docs/spec/core-engine/w035-formalization/W035_RESIDUAL_PROOF_OBLIGATION_AND_SPEC_EVOLUTION_LEDGER.md",
    "docs/spec/core-engine/w035-formalization/W035_LEAN_ASSUMPTION_DISCHARGE_AND_SEAM_PROOF_MAP.md",
];
const W035_TRACECALC_ORACLE_MATRIX_ARTIFACTS: &[&str] = &[
    "docs/test-runs/core-engine/tracecalc-reference-machine/w035-tracecalc-oracle-matrix-001/run_summary.json",
    "docs/test-runs/core-engine/tracecalc-reference-machine/w035-tracecalc-oracle-matrix-001/oracle-matrix/run_summary.json",
    "docs/test-runs/core-engine/tracecalc-reference-machine/w035-tracecalc-oracle-matrix-001/oracle-matrix/coverage_matrix.json",
    "docs/test-runs/core-engine/tracecalc-reference-machine/w035-tracecalc-oracle-matrix-001/oracle-matrix/uncovered_surface_register.json",
    "docs/test-runs/core-engine/tracecalc-reference-machine/w035-tracecalc-oracle-matrix-001/oracle-matrix/validation.json",
    "docs/test-runs/core-engine/tracecalc-reference-machine/w035-tracecalc-oracle-matrix-001/replay-appliance/validation/bundle_validation.json",
];
const W035_IMPLEMENTATION_CONFORMANCE_ARTIFACTS: &[&str] = &[
    "docs/test-runs/core-engine/implementation-conformance/w035-implementation-conformance-hardening-001/run_summary.json",
    "docs/test-runs/core-engine/implementation-conformance/w035-implementation-conformance-hardening-001/evidence_summary.json",
    "docs/test-runs/core-engine/implementation-conformance/w035-implementation-conformance-hardening-001/gap_disposition_register.json",
    "docs/test-runs/core-engine/implementation-conformance/w035-implementation-conformance-hardening-001/validation.json",
];
const W035_CONTINUOUS_ASSURANCE_ARTIFACTS: &[&str] = &[
    "docs/test-runs/core-engine/continuous-assurance/w035-continuous-assurance-gate-001/run_summary.json",
    "docs/test-runs/core-engine/continuous-assurance/w035-continuous-assurance-gate-001/evidence/source_evidence_index.json",
    "docs/test-runs/core-engine/continuous-assurance/w035-continuous-assurance-gate-001/differentials/cross_engine_differential_gate.json",
    "docs/test-runs/core-engine/continuous-assurance/w035-continuous-assurance-gate-001/decision/continuous_assurance_decision.json",
    "docs/test-runs/core-engine/continuous-assurance/w035-continuous-assurance-gate-001/replay-appliance/validation/bundle_validation.json",
];
const W035_SUPPLEMENTAL_EVIDENCE: &[SupplementalEvidenceSpec] = &[
    SupplementalEvidenceSpec {
        input_id: "w035_tracecalc_oracle_matrix",
        artifact_paths: W035_TRACECALC_ORACLE_MATRIX_ARTIFACTS,
        satisfied_input_id: "w035_tracecalc_oracle_matrix_valid",
        evidence_state_present: "oracle_matrix_present_with_classified_uncovered_rows",
        observations: &[
            "W035 TraceCalc oracle matrix has 30 passing scenarios, 17 matrix rows, 15 covered rows, and 2 classified uncovered rows.",
            "The matrix widens the oracle surface but is not full TraceCalc coverage of the engine universe.",
        ],
        reason_ids: &["pack.grade.tracecalc_oracle_matrix_not_full_coverage"],
    },
    SupplementalEvidenceSpec {
        input_id: "w035_implementation_conformance_hardening",
        artifact_paths: W035_IMPLEMENTATION_CONFORMANCE_ARTIFACTS,
        satisfied_input_id: "w035_gap_dispositions_valid",
        evidence_state_present: "gap_dispositions_valid_without_match_promotion",
        observations: &[
            "W035 implementation conformance records 6 gap dispositions, 5 implementation-work deferrals, 1 spec-evolution deferral, and 0 failed rows.",
            "The dispositions prevent false matches; they do not promote full optimized/core-engine conformance.",
        ],
        reason_ids: &[
            "pack.grade.implementation_gap_dispositions_remain",
            "pack.grade.optimized_core_engine_conformance_not_full",
        ],
    },
    SupplementalEvidenceSpec {
        input_id: "w035_continuous_assurance_gate",
        artifact_paths: W035_CONTINUOUS_ASSURANCE_ARTIFACTS,
        satisfied_input_id: "w035_continuous_assurance_gate_present",
        evidence_state_present: "continuous_gate_defined_without_service_promotion",
        observations: &[
            "W035 continuous assurance defines scheduled lanes and differential criteria with 0 missing artifacts and 0 unexpected mismatches.",
            "The packet explicitly keeps continuous scale assurance, cross-engine differential service, pack C5, Stage 2, and performance-correctness claims unpromoted.",
        ],
        reason_ids: &[
            "pack.grade.continuous_assurance_gate_not_running_service",
            "pack.grade.continuous_cross_engine_diff_service_absent",
        ],
    },
];
const W036_TRACECALC_COVERAGE_RUN_ID: &str = "w036-tracecalc-coverage-closure-001";
const W036_INDEPENDENT_DIFFERENTIAL_RUN_ID: &str = "w036-independent-diversity-differential-001";
const W036_FORMAL_ARTIFACTS: &[&str] = &[
    "docs/spec/core-engine/w036-formalization/W036_LEAN_THEOREM_COVERAGE_EXPANSION.md",
    "docs/spec/core-engine/w036-formalization/W036_TLA_STAGE2_PARTITION_AND_SCHEDULER_EQUIVALENCE_MODEL.md",
    "formal/lean/OxCalc/CoreEngine/W036LeanCoverageExpansion.lean",
    "formal/lean/OxCalc/CoreEngine/W036CallableBoundaryInventory.lean",
    "formal/tla/CoreEngineW036Stage2Partition.tla",
    "formal/tla/CoreEngineW036Stage2Partition.scheduler_blocked.cfg",
    "formal/tla/CoreEngineW036Stage2Partition.partition_cross_dep.cfg",
    "formal/tla/CoreEngineW036Stage2Partition.bounded_ready.cfg",
    "formal/tla/CoreEngineW036Stage2Partition.fence_reject.cfg",
    "formal/tla/CoreEngineW036Stage2Partition.multi_reader.cfg",
];
const W036_FORMATTING_WATCH_ARTIFACTS: &[&str] = &[
    "docs/spec/core-engine/w036-formalization/W036_RESIDUAL_COVERAGE_AND_PROMOTION_BLOCKER_LEDGER.md",
    "docs/spec/core-engine/w036-formalization/W036_CONTINUOUS_ASSURANCE_OPERATION_AND_HISTORY_WINDOW.md",
];
const W036_TRACECALC_COVERAGE_ARTIFACTS: &[&str] = &[
    "docs/test-runs/core-engine/tracecalc-reference-machine/w036-tracecalc-coverage-closure-001/run_summary.json",
    "docs/test-runs/core-engine/tracecalc-reference-machine/w036-tracecalc-coverage-closure-001/oracle-matrix/run_summary.json",
    "docs/test-runs/core-engine/tracecalc-reference-machine/w036-tracecalc-coverage-closure-001/oracle-matrix/coverage_closure_criteria.json",
    "docs/test-runs/core-engine/tracecalc-reference-machine/w036-tracecalc-coverage-closure-001/oracle-matrix/coverage_matrix.json",
    "docs/test-runs/core-engine/tracecalc-reference-machine/w036-tracecalc-coverage-closure-001/oracle-matrix/no_loss_crosswalk.json",
    "docs/test-runs/core-engine/tracecalc-reference-machine/w036-tracecalc-coverage-closure-001/oracle-matrix/validation.json",
    "docs/test-runs/core-engine/tracecalc-reference-machine/w036-tracecalc-coverage-closure-001/replay-appliance/validation/bundle_validation.json",
];
const W036_IMPLEMENTATION_CONFORMANCE_ARTIFACTS: &[&str] = &[
    "docs/test-runs/core-engine/implementation-conformance/w036-implementation-conformance-closure-001/run_summary.json",
    "docs/test-runs/core-engine/implementation-conformance/w036-implementation-conformance-closure-001/evidence_summary.json",
    "docs/test-runs/core-engine/implementation-conformance/w036-implementation-conformance-closure-001/w036_closure_action_register.json",
    "docs/test-runs/core-engine/implementation-conformance/w036-implementation-conformance-closure-001/w036_blocker_register.json",
    "docs/test-runs/core-engine/implementation-conformance/w036-implementation-conformance-closure-001/w036_match_promotion_guard.json",
    "docs/test-runs/core-engine/implementation-conformance/w036-implementation-conformance-closure-001/validation.json",
];
const W036_TLA_STAGE2_ARTIFACTS: &[&str] = &[
    "docs/test-runs/core-engine/tla/w036-stage2-partition-001/run_summary.json",
    "docs/test-runs/core-engine/tla/w036-stage2-partition-001/promotion_blockers.json",
    "docs/test-runs/core-engine/tla/w036-stage2-partition-001/validation.json",
];
const W036_DIFFERENTIAL_ARTIFACTS: &[&str] = &[
    "docs/test-runs/core-engine/independent-conformance/w036-independent-diversity-differential-001/run_summary.json",
    "docs/test-runs/core-engine/independent-conformance/w036-independent-diversity-differential-001/diversity/evaluator_diversity_register.json",
    "docs/test-runs/core-engine/independent-conformance/w036-independent-diversity-differential-001/differentials/cross_engine_differential_harness.json",
    "docs/test-runs/core-engine/independent-conformance/w036-independent-diversity-differential-001/decision/promotion_guard.json",
    "docs/test-runs/core-engine/independent-conformance/w036-independent-diversity-differential-001/replay-appliance/validation/bundle_validation.json",
    "docs/test-runs/core-engine/cross-engine-differential/w036-independent-diversity-differential-001/run_summary.json",
    "docs/test-runs/core-engine/cross-engine-differential/w036-independent-diversity-differential-001/differentials/cross_engine_differential_harness.json",
    "docs/test-runs/core-engine/cross-engine-differential/w036-independent-diversity-differential-001/decision/promotion_guard.json",
    "docs/test-runs/core-engine/cross-engine-differential/w036-independent-diversity-differential-001/validation.json",
];
const W036_CONTINUOUS_ASSURANCE_ARTIFACTS: &[&str] = &[
    "docs/test-runs/core-engine/continuous-assurance/w036-continuous-assurance-operation-001/run_summary.json",
    "docs/test-runs/core-engine/continuous-assurance/w036-continuous-assurance-operation-001/evidence/source_evidence_index.json",
    "docs/test-runs/core-engine/continuous-assurance/w036-continuous-assurance-operation-001/schedule/continuous_assurance_schedule.json",
    "docs/test-runs/core-engine/continuous-assurance/w036-continuous-assurance-operation-001/history/assurance_history_window.json",
    "docs/test-runs/core-engine/continuous-assurance/w036-continuous-assurance-operation-001/thresholds/regression_thresholds.json",
    "docs/test-runs/core-engine/continuous-assurance/w036-continuous-assurance-operation-001/alerts/quarantine_policy.json",
    "docs/test-runs/core-engine/continuous-assurance/w036-continuous-assurance-operation-001/operation/simulated_multi_run_evidence.json",
    "docs/test-runs/core-engine/continuous-assurance/w036-continuous-assurance-operation-001/decision/continuous_assurance_decision.json",
    "docs/test-runs/core-engine/continuous-assurance/w036-continuous-assurance-operation-001/replay-appliance/validation/bundle_validation.json",
];
const W036_SUPPLEMENTAL_EVIDENCE: &[SupplementalEvidenceSpec] = &[
    SupplementalEvidenceSpec {
        input_id: "w036_tracecalc_coverage_closure",
        artifact_paths: W036_TRACECALC_COVERAGE_ARTIFACTS,
        satisfied_input_id: "w036_tracecalc_coverage_closure_valid",
        evidence_state_present: "coverage_closure_criteria_present_no_full_oracle_claim",
        observations: &[
            "W036 TraceCalc coverage has 32 matrix rows, 30 covered rows, 1 classified uncovered row, 1 excluded row, and 0 failed/missing rows.",
            "The coverage criteria are stronger than W035 but still do not promote a full TraceCalc oracle claim.",
        ],
        reason_ids: &["pack.grade.w036_tracecalc_oracle_not_full_coverage"],
    },
    SupplementalEvidenceSpec {
        input_id: "w036_implementation_conformance_closure",
        artifact_paths: W036_IMPLEMENTATION_CONFORMANCE_ARTIFACTS,
        satisfied_input_id: "w036_implementation_conformance_closure_valid",
        evidence_state_present: "closure_actions_present_no_match_promotion",
        observations: &[
            "W036 implementation conformance emits 6 action rows, 2 harness first-fix rows, 4 blocker-routed rows, and 0 failed rows.",
            "No declared W035 gap is promoted as an optimized/core-engine match.",
        ],
        reason_ids: &[
            "pack.grade.w036_declared_gap_blockers_remain",
            "pack.grade.w036_optimized_core_engine_conformance_not_full",
        ],
    },
    SupplementalEvidenceSpec {
        input_id: "w036_tla_stage2_partition",
        artifact_paths: W036_TLA_STAGE2_ARTIFACTS,
        satisfied_input_id: "w036_tla_stage2_partition_valid",
        evidence_state_present: "bounded_stage2_partition_model_present_no_policy_promotion",
        observations: &[
            "W036 TLA Stage 2 partition checks 5 configs, 0 failed configs, and explicit scheduler-readiness criteria.",
            "The model is bounded evidence and does not promote Stage 2 policy or pack-grade replay.",
        ],
        reason_ids: &[
            "pack.grade.w036_stage2_scheduler_policy_unpromoted",
            "pack.grade.w036_stage2_replay_equivalence_not_pack_grade",
        ],
    },
    SupplementalEvidenceSpec {
        input_id: "w036_independent_differential_harness",
        artifact_paths: W036_DIFFERENTIAL_ARTIFACTS,
        satisfied_input_id: "w036_independent_differential_harness_valid",
        evidence_state_present: "differential_harness_present_no_service_promotion",
        observations: &[
            "W036 independent diversity records 0 fully independent evaluator rows and 6 promotion blockers.",
            "W036 cross-engine differentials record 0 unexpected mismatches and no continuous service promotion.",
        ],
        reason_ids: &[
            "pack.grade.w036_fully_independent_evaluator_absent",
            "pack.grade.w036_continuous_cross_engine_diff_service_absent",
        ],
    },
    SupplementalEvidenceSpec {
        input_id: "w036_continuous_assurance_history",
        artifact_paths: W036_CONTINUOUS_ASSURANCE_ARTIFACTS,
        satisfied_input_id: "w036_continuous_assurance_history_valid",
        evidence_state_present: "simulated_history_thresholds_present_no_service_promotion",
        observations: &[
            "W036 continuous assurance emits 6 simulated history rows, 7 threshold rules, 7 quarantine/alert rules, and 0 unexpected mismatches.",
            "The packet is simulated multi-run evidence, not an operated assurance service or alert dispatcher.",
        ],
        reason_ids: &[
            "pack.grade.w036_continuous_assurance_simulated_not_operated",
            "pack.grade.w036_quarantine_policy_not_enforced_by_service",
            "pack.grade.w036_timing_not_correctness_proof",
        ],
    },
];
const W037_TRACECALC_OBSERVABLE_RUN_ID: &str = "w037-tracecalc-observable-closure-001";
const W037_TREECALC_CONFORMANCE_RUN_ID: &str = "w037-optimized-core-conformance-treecalc-001";
const W037_FORMAL_ARTIFACTS: &[&str] = &[
    "docs/spec/core-engine/w037-formalization/W037_LEAN_TLA_PROOF_MODEL_CLOSURE_INVENTORY.md",
    "docs/spec/core-engine/w037-formalization/W037_STAGE2_DETERMINISTIC_REPLAY_AND_PARTITION_PROMOTION_CRITERIA.md",
    "formal/lean/OxCalc/CoreEngine/W037ProofModelClosureInventory.lean",
    "formal/lean/OxCalc/CoreEngine/W037Stage2PromotionCriteria.lean",
];
const W037_FORMATTING_WATCH_ARTIFACTS: &[&str] = &[
    "docs/spec/core-engine/w037-formalization/W037_DIRECT_OXFML_EVALUATOR_AND_LET_LAMBDA_SEAM_EVIDENCE.md",
    "docs/spec/core-engine/w037-formalization/W037_OPERATED_CONTINUOUS_ASSURANCE_AND_CROSS_ENGINE_SERVICE_PILOT.md",
];
const W037_TRACECALC_OBSERVABLE_ARTIFACTS: &[&str] = &[
    "docs/test-runs/core-engine/tracecalc-reference-machine/w037-tracecalc-observable-closure-001/oracle-matrix/run_summary.json",
    "docs/test-runs/core-engine/tracecalc-reference-machine/w037-tracecalc-observable-closure-001/oracle-matrix/coverage_closure_criteria.json",
    "docs/test-runs/core-engine/tracecalc-reference-machine/w037-tracecalc-observable-closure-001/oracle-matrix/no_loss_crosswalk.json",
    "docs/test-runs/core-engine/tracecalc-reference-machine/w037-tracecalc-observable-closure-001/replay-appliance/validation/bundle_validation.json",
];
const W037_IMPLEMENTATION_CONFORMANCE_ARTIFACTS: &[&str] = &[
    "docs/test-runs/core-engine/implementation-conformance/w037-implementation-conformance-closure-001/run_summary.json",
    "docs/test-runs/core-engine/implementation-conformance/w037-implementation-conformance-closure-001/w037_conformance_decision_register.json",
    "docs/test-runs/core-engine/implementation-conformance/w037-implementation-conformance-closure-001/w037_residual_blocker_register.json",
    "docs/test-runs/core-engine/implementation-conformance/w037-implementation-conformance-closure-001/w037_match_promotion_guard.json",
];
const W037_DIRECT_OXFML_ARTIFACTS: &[&str] = &[
    "docs/test-runs/core-engine/upstream-host/w037-direct-oxfml-evaluator-001/run_summary.json",
    "docs/test-runs/core-engine/upstream-host/w037-direct-oxfml-evaluator-001/case_index.json",
];
const W037_FORMAL_INVENTORY_ARTIFACTS: &[&str] = &[
    "docs/test-runs/core-engine/formal-inventory/w037-proof-model-closure-001/run_summary.json",
    "docs/test-runs/core-engine/formal-inventory/w037-proof-model-closure-001/promotion_blockers.json",
    "docs/test-runs/core-engine/formal-inventory/w037-proof-model-closure-001/validation.json",
];
const W037_STAGE2_CRITERIA_ARTIFACTS: &[&str] = &[
    "docs/test-runs/core-engine/stage2-criteria/w037-stage2-deterministic-replay-criteria-001/run_summary.json",
    "docs/test-runs/core-engine/stage2-criteria/w037-stage2-deterministic-replay-criteria-001/promotion_decision.json",
    "docs/test-runs/core-engine/stage2-criteria/w037-stage2-deterministic-replay-criteria-001/validation.json",
];
const W037_CONTINUOUS_ASSURANCE_ARTIFACTS: &[&str] = &[
    "docs/test-runs/core-engine/continuous-assurance/w037-operated-assurance-service-pilot-001/run_summary.json",
    "docs/test-runs/core-engine/continuous-assurance/w037-operated-assurance-service-pilot-001/evidence/source_evidence_index.json",
    "docs/test-runs/core-engine/continuous-assurance/w037-operated-assurance-service-pilot-001/service/service_readiness.json",
    "docs/test-runs/core-engine/continuous-assurance/w037-operated-assurance-service-pilot-001/service/cross_engine_service_pilot.json",
    "docs/test-runs/core-engine/continuous-assurance/w037-operated-assurance-service-pilot-001/replay-appliance/validation/bundle_validation.json",
];
const W037_SUPPLEMENTAL_EVIDENCE: &[SupplementalEvidenceSpec] = &[
    SupplementalEvidenceSpec {
        input_id: "w037_tracecalc_observable_closure",
        artifact_paths: W037_TRACECALC_OBSERVABLE_ARTIFACTS,
        satisfied_input_id: "w037_tracecalc_observable_closure_valid",
        evidence_state_present: "observable_closure_present_with_authority_exclusion_no_full_oracle_claim",
        observations: &[
            "W037 TraceCalc observable closure has 32 matrix rows, 31 covered rows, 0 uncovered rows, 1 authority-excluded row, and 0 failed/missing rows.",
            "The observable closure improves the oracle surface but still does not promote full TraceCalc oracle authority for pack C5.",
        ],
        reason_ids: &["pack.grade.w037_tracecalc_oracle_not_full_coverage"],
    },
    SupplementalEvidenceSpec {
        input_id: "w037_implementation_conformance_closure",
        artifact_paths: W037_IMPLEMENTATION_CONFORMANCE_ARTIFACTS,
        satisfied_input_id: "w037_implementation_conformance_closure_valid",
        evidence_state_present: "one_declared_gap_promoted_with_residual_blockers",
        observations: &[
            "W037 implementation conformance records 6 decision rows, 1 fixed/promoted row, 5 residual blockers, and 0 failed rows.",
            "One declared gap is match-promoted under guard, but full optimized/core-engine verification remains blocked.",
        ],
        reason_ids: &["pack.grade.w037_optimized_core_engine_conformance_not_full"],
    },
    SupplementalEvidenceSpec {
        input_id: "w037_direct_oxfml_evaluator",
        artifact_paths: W037_DIRECT_OXFML_ARTIFACTS,
        satisfied_input_id: "w037_direct_oxfml_evaluator_valid",
        evidence_state_present: "direct_oxfml_slice_present_no_pack_promotion",
        observations: &[
            "W037 direct OxFml evaluator evidence has 12 upstream-host fixture rows, 3 direct-OxFml rows, 2 LET/LAMBDA rows, 1 W073 typed formatting guard row, and 0 expectation mismatches.",
            "Direct evaluator absence is removed for this slice, but pack-grade replay governance and other C5 blockers remain.",
        ],
        reason_ids: &[],
    },
    SupplementalEvidenceSpec {
        input_id: "w037_proof_model_inventory",
        artifact_paths: W037_FORMAL_INVENTORY_ARTIFACTS,
        satisfied_input_id: "w037_proof_model_inventory_valid",
        evidence_state_present: "proof_model_inventory_checked_no_full_verification_promotion",
        observations: &[
            "W037 proof/model inventory checks 12 Lean files and 11 routine TLC configs with zero explicit Lean axioms, zero sorry/admit placeholders, and zero failed TLC configs.",
            "The inventory is checked bounded evidence, not total Lean/TLA verification or general OxFunc kernel promotion.",
        ],
        reason_ids: &[
            "pack.grade.w037_full_lean_tla_verification_not_promoted",
            "pack.grade.w037_general_oxfunc_kernel_not_promoted",
        ],
    },
    SupplementalEvidenceSpec {
        input_id: "w037_stage2_criteria",
        artifact_paths: W037_STAGE2_CRITERIA_ARTIFACTS,
        satisfied_input_id: "w037_stage2_criteria_valid",
        evidence_state_present: "stage2_criteria_present_no_policy_promotion",
        observations: &[
            "W037 Stage 2 criteria record 7 criteria rows, 3 satisfied rows, 4 blocked rows, and no promotion candidate.",
            "Deterministic partition replay, production partition soundness, operated differential service, and pack governance remain absent.",
        ],
        reason_ids: &[
            "pack.grade.w037_stage2_policy_unpromoted",
            "pack.grade.w037_stage2_replay_equivalence_not_pack_grade",
        ],
    },
    SupplementalEvidenceSpec {
        input_id: "w037_operated_assurance_service_pilot",
        artifact_paths: W037_CONTINUOUS_ASSURANCE_ARTIFACTS,
        satisfied_input_id: "w037_operated_assurance_service_pilot_valid",
        evidence_state_present: "file_backed_service_readiness_present_no_operated_service_promotion",
        observations: &[
            "W037 continuous assurance records 16 source rows, 9 differential rows, 11 history rows, 10 readiness criteria, 4 blocked service criteria, 0 missing artifacts, and 0 unexpected mismatches.",
            "The service pilot is file-backed readiness evidence, not an operated assurance service, alert dispatcher, or continuous cross-engine differential service.",
        ],
        reason_ids: &[
            "pack.grade.w037_operated_continuous_assurance_service_absent",
            "pack.grade.w037_quarantine_policy_not_enforced_by_service",
            "pack.grade.w037_continuous_cross_engine_diff_service_absent",
            "pack.grade.w037_fully_independent_evaluator_absent",
            "pack.grade.w037_timing_not_correctness_proof",
        ],
    },
];
const W038_FORMAL_ARTIFACTS: &[&str] = &[
    "docs/spec/core-engine/w038-formalization/W038_PROOF_MODEL_ASSUMPTION_DISCHARGE_AND_TOTALITY_BOUNDARY_HARDENING.md",
    "docs/spec/core-engine/w038-formalization/W038_STAGE2_PARTITION_REPLAY_AND_SEMANTIC_EQUIVALENCE_EXECUTION.md",
    "formal/lean/OxCalc/CoreEngine/W038AssumptionDischargeAndTotality.lean",
];
const W038_FORMATTING_WATCH_ARTIFACTS: &[&str] = &[
    "docs/spec/core-engine/w038-formalization/W038_STAGE2_PARTITION_REPLAY_AND_SEMANTIC_EQUIVALENCE_EXECUTION.md",
    "docs/spec/core-engine/w038-formalization/W038_INDEPENDENT_EVALUATOR_DIVERSITY_AND_OXFML_SEAM_WATCH_CLOSURE.md",
];
const W038_TRACECALC_AUTHORITY_ARTIFACTS: &[&str] = &[
    "docs/test-runs/core-engine/tracecalc-authority/w038-tracecalc-authority-discharge-001/run_summary.json",
    "docs/test-runs/core-engine/tracecalc-authority/w038-tracecalc-authority-discharge-001/authority_discharge_ledger.json",
    "docs/test-runs/core-engine/tracecalc-authority/w038-tracecalc-authority-discharge-001/oracle_authority_map.json",
    "docs/test-runs/core-engine/tracecalc-authority/w038-tracecalc-authority-discharge-001/validation.json",
];
const W038_IMPLEMENTATION_CONFORMANCE_ARTIFACTS: &[&str] = &[
    "docs/test-runs/core-engine/implementation-conformance/w038-optimized-core-conformance-disposition-001/run_summary.json",
    "docs/test-runs/core-engine/implementation-conformance/w038-optimized-core-conformance-disposition-001/evidence_summary.json",
    "docs/test-runs/core-engine/implementation-conformance/w038-optimized-core-conformance-disposition-001/w038_conformance_disposition_register.json",
    "docs/test-runs/core-engine/implementation-conformance/w038-optimized-core-conformance-disposition-001/w038_exact_remaining_blocker_register.json",
    "docs/test-runs/core-engine/implementation-conformance/w038-optimized-core-conformance-disposition-001/w038_match_promotion_guard.json",
    "docs/test-runs/core-engine/implementation-conformance/w038-optimized-core-conformance-disposition-001/validation.json",
];
const W038_FORMAL_ASSURANCE_ARTIFACTS: &[&str] = &[
    "docs/test-runs/core-engine/formal-assurance/w038-proof-model-assumption-discharge-001/run_summary.json",
    "docs/test-runs/core-engine/formal-assurance/w038-proof-model-assumption-discharge-001/source_evidence_index.json",
    "docs/test-runs/core-engine/formal-assurance/w038-proof-model-assumption-discharge-001/w038_assumption_discharge_ledger.json",
    "docs/test-runs/core-engine/formal-assurance/w038-proof-model-assumption-discharge-001/w038_exact_proof_model_blocker_register.json",
    "docs/test-runs/core-engine/formal-assurance/w038-proof-model-assumption-discharge-001/w038_model_bound_register.json",
    "docs/test-runs/core-engine/formal-assurance/w038-proof-model-assumption-discharge-001/w038_totality_boundary_register.json",
    "docs/test-runs/core-engine/formal-assurance/w038-proof-model-assumption-discharge-001/validation.json",
];
const W038_STAGE2_REPLAY_ARTIFACTS: &[&str] = &[
    "docs/test-runs/core-engine/stage2-replay/w038-stage2-partition-replay-001/run_summary.json",
    "docs/test-runs/core-engine/stage2-replay/w038-stage2-partition-replay-001/partition_replay_matrix.json",
    "docs/test-runs/core-engine/stage2-replay/w038-stage2-partition-replay-001/partition_order_permutation_replay.json",
    "docs/test-runs/core-engine/stage2-replay/w038-stage2-partition-replay-001/semantic_equivalence_report.json",
    "docs/test-runs/core-engine/stage2-replay/w038-stage2-partition-replay-001/stage2_exact_blocker_register.json",
    "docs/test-runs/core-engine/stage2-replay/w038-stage2-partition-replay-001/promotion_decision.json",
    "docs/test-runs/core-engine/stage2-replay/w038-stage2-partition-replay-001/validation.json",
];
const W038_OPERATED_ASSURANCE_ARTIFACTS: &[&str] = &[
    "docs/test-runs/core-engine/operated-assurance/w038-operated-assurance-alert-quarantine-001/run_summary.json",
    "docs/test-runs/core-engine/operated-assurance/w038-operated-assurance-alert-quarantine-001/multi_run_history.json",
    "docs/test-runs/core-engine/operated-assurance/w038-operated-assurance-alert-quarantine-001/alert_quarantine_enforcement.json",
    "docs/test-runs/core-engine/operated-assurance/w038-operated-assurance-alert-quarantine-001/cross_engine_service_disposition.json",
    "docs/test-runs/core-engine/operated-assurance/w038-operated-assurance-alert-quarantine-001/service_readiness_disposition.json",
    "docs/test-runs/core-engine/operated-assurance/w038-operated-assurance-alert-quarantine-001/exact_service_blocker_register.json",
    "docs/test-runs/core-engine/operated-assurance/w038-operated-assurance-alert-quarantine-001/promotion_decision.json",
    "docs/test-runs/core-engine/operated-assurance/w038-operated-assurance-alert-quarantine-001/validation.json",
];
const W038_DIVERSITY_SEAM_ARTIFACTS: &[&str] = &[
    "docs/test-runs/core-engine/diversity-seam/w038-diversity-seam-watch-001/run_summary.json",
    "docs/test-runs/core-engine/diversity-seam/w038-diversity-seam-watch-001/implementation_diversity_disposition.json",
    "docs/test-runs/core-engine/diversity-seam/w038-diversity-seam-watch-001/oxfml_seam_watch_packet.json",
    "docs/test-runs/core-engine/diversity-seam/w038-diversity-seam-watch-001/exact_diversity_seam_blocker_register.json",
    "docs/test-runs/core-engine/diversity-seam/w038-diversity-seam-watch-001/promotion_decision.json",
    "docs/test-runs/core-engine/diversity-seam/w038-diversity-seam-watch-001/validation.json",
];
const W038_SUPPLEMENTAL_EVIDENCE: &[SupplementalEvidenceSpec] = &[
    SupplementalEvidenceSpec {
        input_id: "w038_tracecalc_authority_discharge",
        artifact_paths: W038_TRACECALC_AUTHORITY_ARTIFACTS,
        satisfied_input_id: "w038_tracecalc_authority_discharge_valid",
        evidence_state_present: "tracecalc_authority_discharged_for_current_profile_no_release_grade_promotion",
        observations: &[
            "W038 TraceCalc authority records 32 source rows, 31 covered rows, 0 uncovered rows, 1 accepted external authority exclusion, and 0 remaining TraceCalc authority blockers.",
            "TraceCalc authority is discharged for the current OxCalc-owned observable profile but does not promote release-grade verification alone.",
        ],
        reason_ids: &["pack.grade.w038_release_grade_verification_not_promoted"],
    },
    SupplementalEvidenceSpec {
        input_id: "w038_optimized_core_conformance_disposition",
        artifact_paths: W038_IMPLEMENTATION_CONFORMANCE_ARTIFACTS,
        satisfied_input_id: "w038_optimized_core_conformance_disposition_valid",
        evidence_state_present: "w038_conformance_disposition_present_with_exact_blockers",
        observations: &[
            "W038 optimized/core conformance records 5 disposition rows, 3 direct-evidence rows, 1 accepted boundary row, 4 exact remaining blockers, and 0 failed rows.",
            "No declared gap is promoted as a match in W038; optimized/core release-grade verification remains blocked.",
        ],
        reason_ids: &[
            "pack.grade.w038_optimized_core_engine_conformance_not_full",
            "pack.grade.w038_callable_metadata_projection_absent",
        ],
    },
    SupplementalEvidenceSpec {
        input_id: "w038_formal_assurance_disposition",
        artifact_paths: W038_FORMAL_ASSURANCE_ARTIFACTS,
        satisfied_input_id: "w038_formal_assurance_disposition_valid",
        evidence_state_present: "w038_formal_assurance_present_with_totality_boundaries",
        observations: &[
            "W038 formal assurance records 8 assumption rows, 3 local-proof rows, 2 bounded-model rows, 3 totality boundaries, 6 exact blockers, and 0 failed rows.",
            "Full Lean/TLA verification, pack-grade replay, C5, Stage 2 policy, and general OxFunc kernels remain unpromoted.",
        ],
        reason_ids: &[
            "pack.grade.w038_full_lean_tla_verification_not_promoted",
            "pack.grade.w038_general_oxfunc_kernel_not_promoted",
        ],
    },
    SupplementalEvidenceSpec {
        input_id: "w038_stage2_partition_replay",
        artifact_paths: W038_STAGE2_REPLAY_ARTIFACTS,
        satisfied_input_id: "w038_stage2_partition_replay_valid",
        evidence_state_present: "bounded_stage2_replay_present_no_policy_promotion",
        observations: &[
            "W038 Stage 2 replay records 5 bounded partition rows, 6 permutation rows, 5 observable-invariance rows, 1 W073 formatting watch row, 3 exact blockers, and 0 failed rows.",
            "Production Stage 2 policy remains unpromoted because partition soundness, operated cross-engine service, and pack-grade replay governance remain absent.",
        ],
        reason_ids: &[
            "pack.grade.w038_stage2_policy_unpromoted",
            "pack.grade.w038_stage2_replay_equivalence_not_pack_grade",
            "pack.grade.w038_production_partition_soundness_absent",
            "pack.grade.w038_pack_grade_replay_governance_absent",
        ],
    },
    SupplementalEvidenceSpec {
        input_id: "w038_operated_assurance_disposition",
        artifact_paths: W038_OPERATED_ASSURANCE_ARTIFACTS,
        satisfied_input_id: "w038_operated_assurance_disposition_valid",
        evidence_state_present: "local_alert_quarantine_and_history_bound_no_operated_service_promotion",
        observations: &[
            "W038 operated assurance records 8 source rows, 15 multi-run history rows, 8 alert rules, 4 exact service blockers, and 0 failed rows.",
            "Local alert/quarantine evaluation and file-backed service disposition are not an operated assurance service, external dispatcher, or operated cross-engine differential.",
        ],
        reason_ids: &[
            "pack.grade.w038_operated_continuous_assurance_service_absent",
            "pack.grade.w038_external_alert_dispatcher_absent",
            "pack.grade.w038_operated_cross_engine_diff_service_absent",
            "pack.grade.w038_retained_history_store_absent",
        ],
    },
    SupplementalEvidenceSpec {
        input_id: "w038_independent_diversity_seam_watch",
        artifact_paths: W038_DIVERSITY_SEAM_ARTIFACTS,
        satisfied_input_id: "w038_independent_diversity_seam_watch_valid",
        evidence_state_present: "diversity_and_oxfml_seam_watch_bound_without_full_independence",
        observations: &[
            "W038 diversity/seam watch records 5 diversity rows, 8 OxFml seam-watch rows, 4 exact blockers, current W073 typed-formatting alignment, and 0 failed rows.",
            "Fully independent evaluator diversity, callable metadata projection, and broad OxFml display/publication closure remain unpromoted.",
        ],
        reason_ids: &[
            "pack.grade.w038_fully_independent_evaluator_absent",
            "pack.grade.w038_callable_metadata_projection_absent",
            "pack.grade.w038_broad_oxfml_display_publication_unpromoted",
        ],
    },
];
const W039_FORMAL_ARTIFACTS: &[&str] = &[
    "docs/spec/core-engine/w039-formalization/W039_LEAN_TLA_TOTALITY_AND_PROOF_MODEL_CLOSURE_TRANCHE.md",
    "docs/spec/core-engine/w039-formalization/W039_STAGE2_PRODUCTION_PARTITION_POLICY_AND_REPLAY_GOVERNANCE.md",
    "formal/lean/OxCalc/CoreEngine/W039ProofModelTotalityClosure.lean",
    "formal/lean/OxCalc/CoreEngine/W039Stage2ProductionPolicy.lean",
];
const W039_FORMATTING_WATCH_ARTIFACTS: &[&str] = &[
    "docs/spec/core-engine/w039-formalization/W039_OXFML_SEAM_BREADTH_AND_CALLABLE_METADATA_CLOSURE.md",
    "docs/test-runs/core-engine/release-grade-ledger/w039-residual-successor-obligation-ledger-001/w073_formatting_intake.json",
];
const W039_RELEASE_LEDGER_ARTIFACTS: &[&str] = &[
    "docs/test-runs/core-engine/release-grade-ledger/w039-residual-successor-obligation-ledger-001/run_summary.json",
    "docs/test-runs/core-engine/release-grade-ledger/w039-residual-successor-obligation-ledger-001/source_evidence_index.json",
    "docs/test-runs/core-engine/release-grade-ledger/w039-residual-successor-obligation-ledger-001/successor_obligation_ledger.json",
    "docs/test-runs/core-engine/release-grade-ledger/w039-residual-successor-obligation-ledger-001/promotion_readiness_map.json",
    "docs/test-runs/core-engine/release-grade-ledger/w039-residual-successor-obligation-ledger-001/w073_formatting_intake.json",
    "docs/test-runs/core-engine/release-grade-ledger/w039-residual-successor-obligation-ledger-001/validation.json",
];
const W039_IMPLEMENTATION_CONFORMANCE_ARTIFACTS: &[&str] = &[
    "docs/test-runs/core-engine/implementation-conformance/w039-optimized-core-exact-blocker-disposition-001/run_summary.json",
    "docs/test-runs/core-engine/implementation-conformance/w039-optimized-core-exact-blocker-disposition-001/evidence_summary.json",
    "docs/test-runs/core-engine/implementation-conformance/w039-optimized-core-exact-blocker-disposition-001/w039_exact_blocker_disposition_register.json",
    "docs/test-runs/core-engine/implementation-conformance/w039-optimized-core-exact-blocker-disposition-001/w039_exact_remaining_blocker_register.json",
    "docs/test-runs/core-engine/implementation-conformance/w039-optimized-core-exact-blocker-disposition-001/w039_match_promotion_guard.json",
    "docs/test-runs/core-engine/implementation-conformance/w039-optimized-core-exact-blocker-disposition-001/validation.json",
];
const W039_FORMAL_ASSURANCE_ARTIFACTS: &[&str] = &[
    "docs/test-runs/core-engine/formal-assurance/w039-proof-model-totality-closure-001/run_summary.json",
    "docs/test-runs/core-engine/formal-assurance/w039-proof-model-totality-closure-001/source_evidence_index.json",
    "docs/test-runs/core-engine/formal-assurance/w039-proof-model-totality-closure-001/w039_proof_model_totality_ledger.json",
    "docs/test-runs/core-engine/formal-assurance/w039-proof-model-totality-closure-001/w039_exact_proof_model_blocker_register.json",
    "docs/test-runs/core-engine/formal-assurance/w039-proof-model-totality-closure-001/w039_model_bound_register.json",
    "docs/test-runs/core-engine/formal-assurance/w039-proof-model-totality-closure-001/w039_totality_boundary_register.json",
    "docs/test-runs/core-engine/formal-assurance/w039-proof-model-totality-closure-001/validation.json",
];
const W039_STAGE2_REPLAY_ARTIFACTS: &[&str] = &[
    "docs/test-runs/core-engine/stage2-replay/w039-stage2-production-policy-replay-governance-001/run_summary.json",
    "docs/test-runs/core-engine/stage2-replay/w039-stage2-production-policy-replay-governance-001/source_evidence_index.json",
    "docs/test-runs/core-engine/stage2-replay/w039-stage2-production-policy-replay-governance-001/w039_partition_soundness_register.json",
    "docs/test-runs/core-engine/stage2-replay/w039-stage2-production-policy-replay-governance-001/w039_replay_governance_register.json",
    "docs/test-runs/core-engine/stage2-replay/w039-stage2-production-policy-replay-governance-001/w039_stage2_policy_gate_register.json",
    "docs/test-runs/core-engine/stage2-replay/w039-stage2-production-policy-replay-governance-001/w039_stage2_exact_blocker_register.json",
    "docs/test-runs/core-engine/stage2-replay/w039-stage2-production-policy-replay-governance-001/promotion_decision.json",
    "docs/test-runs/core-engine/stage2-replay/w039-stage2-production-policy-replay-governance-001/validation.json",
];
const W039_OPERATED_ASSURANCE_ARTIFACTS: &[&str] = &[
    "docs/test-runs/core-engine/operated-assurance/w039-operated-assurance-retained-history-001/run_summary.json",
    "docs/test-runs/core-engine/operated-assurance/w039-operated-assurance-retained-history-001/source_evidence_index.json",
    "docs/test-runs/core-engine/operated-assurance/w039-operated-assurance-retained-history-001/w039_retained_history_lifecycle.json",
    "docs/test-runs/core-engine/operated-assurance/w039-operated-assurance-retained-history-001/w039_alert_dispatcher_enforcement.json",
    "docs/test-runs/core-engine/operated-assurance/w039-operated-assurance-retained-history-001/w039_cross_engine_service_substrate.json",
    "docs/test-runs/core-engine/operated-assurance/w039-operated-assurance-retained-history-001/w039_service_readiness_register.json",
    "docs/test-runs/core-engine/operated-assurance/w039-operated-assurance-retained-history-001/w039_exact_service_blocker_register.json",
    "docs/test-runs/core-engine/operated-assurance/w039-operated-assurance-retained-history-001/promotion_decision.json",
    "docs/test-runs/core-engine/operated-assurance/w039-operated-assurance-retained-history-001/validation.json",
];
const W039_DIVERSITY_SEAM_ARTIFACTS: &[&str] = &[
    "docs/test-runs/core-engine/diversity-seam/w039-independent-evaluator-cross-engine-diversity-001/run_summary.json",
    "docs/test-runs/core-engine/diversity-seam/w039-independent-evaluator-cross-engine-diversity-001/source_evidence_index.json",
    "docs/test-runs/core-engine/diversity-seam/w039-independent-evaluator-cross-engine-diversity-001/w039_independent_evaluator_row_set.json",
    "docs/test-runs/core-engine/diversity-seam/w039-independent-evaluator-cross-engine-diversity-001/w039_cross_engine_diversity_register.json",
    "docs/test-runs/core-engine/diversity-seam/w039-independent-evaluator-cross-engine-diversity-001/w039_differential_service_authority_register.json",
    "docs/test-runs/core-engine/diversity-seam/w039-independent-evaluator-cross-engine-diversity-001/w039_exact_diversity_blocker_register.json",
    "docs/test-runs/core-engine/diversity-seam/w039-independent-evaluator-cross-engine-diversity-001/promotion_decision.json",
    "docs/test-runs/core-engine/diversity-seam/w039-independent-evaluator-cross-engine-diversity-001/validation.json",
];
const W039_OXFML_SEAM_ARTIFACTS: &[&str] = &[
    "docs/test-runs/core-engine/oxfml-seam/w039-oxfml-seam-breadth-callable-metadata-001/run_summary.json",
    "docs/test-runs/core-engine/oxfml-seam/w039-oxfml-seam-breadth-callable-metadata-001/source_evidence_index.json",
    "docs/test-runs/core-engine/oxfml-seam/w039-oxfml-seam-breadth-callable-metadata-001/w039_oxfml_surface_register.json",
    "docs/test-runs/core-engine/oxfml-seam/w039-oxfml-seam-breadth-callable-metadata-001/w039_publication_display_boundary_register.json",
    "docs/test-runs/core-engine/oxfml-seam/w039-oxfml-seam-breadth-callable-metadata-001/w039_callable_metadata_register.json",
    "docs/test-runs/core-engine/oxfml-seam/w039-oxfml-seam-breadth-callable-metadata-001/w039_exact_oxfml_seam_blocker_register.json",
    "docs/test-runs/core-engine/oxfml-seam/w039-oxfml-seam-breadth-callable-metadata-001/promotion_decision.json",
    "docs/test-runs/core-engine/oxfml-seam/w039-oxfml-seam-breadth-callable-metadata-001/validation.json",
];
const W039_UPSTREAM_HOST_ARTIFACTS: &[&str] = &[
    "docs/test-runs/core-engine/upstream-host/w039-oxfml-seam-breadth-callable-metadata-001/run_summary.json",
    "docs/test-runs/core-engine/upstream-host/w039-oxfml-seam-breadth-callable-metadata-001/case_index.json",
];
const W039_SUPPLEMENTAL_EVIDENCE: &[SupplementalEvidenceSpec] = &[
    SupplementalEvidenceSpec {
        input_id: "w039_residual_successor_obligation_ledger",
        artifact_paths: W039_RELEASE_LEDGER_ARTIFACTS,
        satisfied_input_id: "w039_residual_successor_obligation_ledger_valid",
        evidence_state_present: "promotion_readiness_map_present_with_pack_c5_blocked",
        observations: &[
            "W039 residual ledger records 10 residual lanes, 20 W039 obligations, 9 promotion targets, current W073 typed-only formatting intake, and cap.C4.distill_valid as highest honest capability.",
            "The promotion map keeps release-grade verification, pack-grade replay, C5, Stage 2, operated service, independent evaluator, broad OxFml, and general OxFunc claims unpromoted.",
        ],
        reason_ids: &[
            "pack.grade.w039_release_grade_verification_not_promoted",
            "pack.grade.w039_pack_c5_blocked_by_promotion_map",
        ],
    },
    SupplementalEvidenceSpec {
        input_id: "w039_optimized_core_exact_blocker_disposition",
        artifact_paths: W039_IMPLEMENTATION_CONFORMANCE_ARTIFACTS,
        satisfied_input_id: "w039_optimized_core_exact_blocker_disposition_valid",
        evidence_state_present: "w039_conformance_disposition_present_with_exact_blockers",
        observations: &[
            "W039 optimized/core conformance records 5 disposition rows, 2 direct-evidence rows, 4 exact remaining blockers, 0 match-promoted rows, and 0 failed rows.",
            "Dynamic release/reclassification, snapshot fence, capability-view fence, and callable metadata projection remain exact optimized/core blockers.",
        ],
        reason_ids: &[
            "pack.grade.w039_optimized_core_engine_conformance_not_full",
            "pack.grade.w039_callable_metadata_projection_absent",
        ],
    },
    SupplementalEvidenceSpec {
        input_id: "w039_formal_assurance_totality_closure",
        artifact_paths: W039_FORMAL_ASSURANCE_ARTIFACTS,
        satisfied_input_id: "w039_formal_assurance_totality_closure_valid",
        evidence_state_present: "w039_formal_assurance_present_with_totality_boundaries",
        observations: &[
            "W039 formal assurance records 7 proof/model rows, 3 local-proof rows, 2 bounded-model rows, 4 totality boundaries, 6 exact blockers, and 0 failed rows.",
            "Full Lean/TLA verification, Rust-engine totality, pack-grade replay, C5, Stage 2 policy, release-grade verification, broad OxFml, and general OxFunc remain unpromoted.",
        ],
        reason_ids: &[
            "pack.grade.w039_full_lean_tla_verification_not_promoted",
            "pack.grade.w039_rust_engine_totality_not_promoted",
            "pack.grade.w039_general_oxfunc_kernel_not_promoted",
        ],
    },
    SupplementalEvidenceSpec {
        input_id: "w039_stage2_production_policy_replay_governance",
        artifact_paths: W039_STAGE2_REPLAY_ARTIFACTS,
        satisfied_input_id: "w039_stage2_production_policy_replay_governance_valid",
        evidence_state_present: "w039_stage2_policy_packet_present_no_policy_promotion",
        observations: &[
            "W039 Stage 2 records 10 policy rows, 5 satisfied bounded-profile rows, 5 partition replay rows, 6 permutation rows, 5 observable-invariance rows, 5 exact blockers, and 0 failed rows.",
            "Stage 2 production policy, operated Stage 2 differential service, pack-grade replay, C5, retained witness lifecycle, and release-grade verification remain unpromoted.",
        ],
        reason_ids: &[
            "pack.grade.w039_stage2_policy_unpromoted",
            "pack.grade.w039_stage2_replay_equivalence_not_pack_grade",
            "pack.grade.w039_production_partition_soundness_absent",
            "pack.grade.w039_pack_grade_replay_governance_absent",
        ],
    },
    SupplementalEvidenceSpec {
        input_id: "w039_operated_assurance_retained_history",
        artifact_paths: W039_OPERATED_ASSURANCE_ARTIFACTS,
        satisfied_input_id: "w039_operated_assurance_retained_history_valid",
        evidence_state_present: "w039_service_substrate_present_no_operated_service_promotion",
        observations: &[
            "W039 operated assurance records 8 source rows, 18 retained-history rows, 11 evaluated alert rules, 12 readiness criteria, 5 blocked criteria, 5 exact service blockers, and 0 failed rows.",
            "Operated continuous assurance, retained-history service, alert dispatcher, operated cross-engine differential, pack-grade replay, C5, Stage 2 policy, and release-grade verification remain unpromoted.",
        ],
        reason_ids: &[
            "pack.grade.w039_operated_continuous_assurance_service_absent",
            "pack.grade.w039_retained_history_store_absent",
            "pack.grade.w039_external_alert_dispatcher_absent",
            "pack.grade.w039_operated_cross_engine_diff_service_absent",
        ],
    },
    SupplementalEvidenceSpec {
        input_id: "w039_independent_evaluator_cross_engine_diversity",
        artifact_paths: W039_DIVERSITY_SEAM_ARTIFACTS,
        satisfied_input_id: "w039_independent_evaluator_cross_engine_diversity_valid",
        evidence_state_present: "w039_diversity_packet_present_no_independent_evaluator_promotion",
        observations: &[
            "W039 diversity records 10 source rows, 7 independent-evaluator rows, 7 cross-engine diversity rows, 7 differential-authority rows, 6 exact diversity blockers, and 0 failed rows.",
            "Fully independent evaluator, operated cross-engine differential service, broad OxFml seam, callable metadata projection, pack-grade replay, C5, Stage 2 policy, and release-grade verification remain unpromoted.",
        ],
        reason_ids: &[
            "pack.grade.w039_fully_independent_evaluator_absent",
            "pack.grade.w039_operated_cross_engine_diff_service_absent",
        ],
    },
    SupplementalEvidenceSpec {
        input_id: "w039_oxfml_seam_breadth_callable_metadata",
        artifact_paths: W039_OXFML_SEAM_ARTIFACTS,
        satisfied_input_id: "w039_oxfml_seam_breadth_callable_metadata_valid",
        evidence_state_present: "w039_oxfml_seam_packet_present_with_callable_blockers",
        observations: &[
            "W039 OxFml seam records 6 source rows, 5 surface rows, 4 publication/display rows, 5 callable rows, 5 exact seam blockers, and 0 failed rows.",
            "The current W073 typed-only formatting guard, format/display boundary, public consumer notes, LET/LAMBDA carrier rows, and callable metadata blockers are bound without broad OxFml or callable metadata promotion.",
        ],
        reason_ids: &[
            "pack.grade.w039_broad_oxfml_display_publication_unpromoted",
            "pack.grade.w039_callable_metadata_projection_absent",
            "pack.grade.w039_general_oxfunc_kernel_not_promoted",
        ],
    },
    SupplementalEvidenceSpec {
        input_id: "w039_fresh_direct_oxfml_upstream_host",
        artifact_paths: W039_UPSTREAM_HOST_ARTIFACTS,
        satisfied_input_id: "w039_fresh_direct_oxfml_upstream_host_valid",
        evidence_state_present: "current_direct_oxfml_run_present_no_pack_promotion",
        observations: &[
            "W039 upstream-host records 12 cases, 3 direct OxFml cases, 2 LET/LAMBDA cases, 1 W073 typed-rule formatting guard, and 0 expectation mismatches.",
            "The direct OxFml slice removes re-execution absence for the exercised surface but does not promote pack-grade replay, C5, broad OxFml, callable metadata, or general OxFunc kernels.",
        ],
        reason_ids: &[],
    },
];
const W040_TREECALC_CONFORMANCE_RUN_ID: &str =
    "w040-optimized-core-dynamic-release-reclassification-001";
const W040_FORMAL_ARTIFACTS: &[&str] = &[
    "docs/spec/core-engine/w040-formalization/W040_RUST_TOTALITY_AND_REFINEMENT_PROOF_TRANCHE.md",
    "docs/spec/core-engine/w040-formalization/W040_LEAN_TLA_FULL_VERIFICATION_DISCHARGE_TRANCHE.md",
    "docs/spec/core-engine/w040-formalization/W040_STAGE2_PRODUCTION_POLICY_AND_EQUIVALENCE_IMPLEMENTATION.md",
];
const W040_FORMATTING_WATCH_ARTIFACTS: &[&str] = &[
    "docs/spec/core-engine/w040-formalization/W040_OXFML_SEAM_BREADTH_AND_CALLABLE_METADATA_IMPLEMENTATION.md",
    "docs/test-runs/core-engine/release-grade-ledger/w040-direct-verification-obligation-map-001/w073_formatting_intake.json",
];
const W040_RELEASE_LEDGER_ARTIFACTS: &[&str] = &[
    "docs/test-runs/core-engine/release-grade-ledger/w040-direct-verification-obligation-map-001/run_summary.json",
    "docs/test-runs/core-engine/release-grade-ledger/w040-direct-verification-obligation-map-001/source_evidence_index.json",
    "docs/test-runs/core-engine/release-grade-ledger/w040-direct-verification-obligation-map-001/direct_verification_obligation_map.json",
    "docs/test-runs/core-engine/release-grade-ledger/w040-direct-verification-obligation-map-001/promotion_target_gate_map.json",
    "docs/test-runs/core-engine/release-grade-ledger/w040-direct-verification-obligation-map-001/w073_formatting_intake.json",
    "docs/test-runs/core-engine/release-grade-ledger/w040-direct-verification-obligation-map-001/validation.json",
];
const W040_IMPLEMENTATION_CONFORMANCE_ARTIFACTS: &[&str] = &[
    "docs/test-runs/core-engine/implementation-conformance/w040-optimized-core-exact-blocker-fixes-differentials-001/run_summary.json",
    "docs/test-runs/core-engine/implementation-conformance/w040-optimized-core-exact-blocker-fixes-differentials-001/source_evidence_index.json",
    "docs/test-runs/core-engine/implementation-conformance/w040-optimized-core-exact-blocker-fixes-differentials-001/dynamic_release_reclassification_evidence.json",
    "docs/test-runs/core-engine/implementation-conformance/w040-optimized-core-exact-blocker-fixes-differentials-001/w040_exact_blocker_disposition_register.json",
    "docs/test-runs/core-engine/implementation-conformance/w040-optimized-core-exact-blocker-fixes-differentials-001/w040_exact_remaining_blocker_register.json",
    "docs/test-runs/core-engine/implementation-conformance/w040-optimized-core-exact-blocker-fixes-differentials-001/w040_match_promotion_guard.json",
    "docs/test-runs/core-engine/implementation-conformance/w040-optimized-core-exact-blocker-fixes-differentials-001/validation.json",
];
const W040_RUST_TOTALITY_ARTIFACTS: &[&str] = &[
    "docs/test-runs/core-engine/formal-assurance/w040-rust-totality-refinement-proof-tranche-001/run_summary.json",
    "docs/test-runs/core-engine/formal-assurance/w040-rust-totality-refinement-proof-tranche-001/source_evidence_index.json",
    "docs/test-runs/core-engine/formal-assurance/w040-rust-totality-refinement-proof-tranche-001/w040_rust_totality_refinement_ledger.json",
    "docs/test-runs/core-engine/formal-assurance/w040-rust-totality-refinement-proof-tranche-001/w040_rust_totality_boundary_register.json",
    "docs/test-runs/core-engine/formal-assurance/w040-rust-totality-refinement-proof-tranche-001/w040_rust_refinement_register.json",
    "docs/test-runs/core-engine/formal-assurance/w040-rust-totality-refinement-proof-tranche-001/w040_rust_exact_blocker_register.json",
    "docs/test-runs/core-engine/formal-assurance/w040-rust-totality-refinement-proof-tranche-001/validation.json",
];
const W040_LEAN_TLA_ARTIFACTS: &[&str] = &[
    "docs/test-runs/core-engine/formal-assurance/w040-lean-tla-full-verification-discharge-001/run_summary.json",
    "docs/test-runs/core-engine/formal-assurance/w040-lean-tla-full-verification-discharge-001/source_evidence_index.json",
    "docs/test-runs/core-engine/formal-assurance/w040-lean-tla-full-verification-discharge-001/w040_lean_tla_discharge_ledger.json",
    "docs/test-runs/core-engine/formal-assurance/w040-lean-tla-full-verification-discharge-001/w040_lean_proof_register.json",
    "docs/test-runs/core-engine/formal-assurance/w040-lean-tla-full-verification-discharge-001/w040_tla_model_bound_register.json",
    "docs/test-runs/core-engine/formal-assurance/w040-lean-tla-full-verification-discharge-001/w040_lean_tla_exact_blocker_register.json",
    "docs/test-runs/core-engine/formal-assurance/w040-lean-tla-full-verification-discharge-001/validation.json",
];
const W040_STAGE2_REPLAY_ARTIFACTS: &[&str] = &[
    "docs/test-runs/core-engine/stage2-replay/w040-stage2-production-policy-equivalence-001/run_summary.json",
    "docs/test-runs/core-engine/stage2-replay/w040-stage2-production-policy-equivalence-001/source_evidence_index.json",
    "docs/test-runs/core-engine/stage2-replay/w040-stage2-production-policy-equivalence-001/w040_partition_analyzer_soundness_register.json",
    "docs/test-runs/core-engine/stage2-replay/w040-stage2-production-policy-equivalence-001/w040_observable_equivalence_register.json",
    "docs/test-runs/core-engine/stage2-replay/w040-stage2-production-policy-equivalence-001/w040_stage2_policy_gate_register.json",
    "docs/test-runs/core-engine/stage2-replay/w040-stage2-production-policy-equivalence-001/w040_stage2_exact_blocker_register.json",
    "docs/test-runs/core-engine/stage2-replay/w040-stage2-production-policy-equivalence-001/promotion_decision.json",
    "docs/test-runs/core-engine/stage2-replay/w040-stage2-production-policy-equivalence-001/validation.json",
];
const W040_OPERATED_ASSURANCE_ARTIFACTS: &[&str] = &[
    "docs/test-runs/core-engine/operated-assurance/w040-operated-assurance-retained-history-service-001/run_summary.json",
    "docs/test-runs/core-engine/operated-assurance/w040-operated-assurance-retained-history-service-001/source_evidence_index.json",
    "docs/test-runs/core-engine/operated-assurance/w040-operated-assurance-retained-history-service-001/w040_operated_runner_register.json",
    "docs/test-runs/core-engine/operated-assurance/w040-operated-assurance-retained-history-service-001/w040_retained_history_store_query.json",
    "docs/test-runs/core-engine/operated-assurance/w040-operated-assurance-retained-history-service-001/w040_cross_engine_service_register.json",
    "docs/test-runs/core-engine/operated-assurance/w040-operated-assurance-retained-history-service-001/w040_alert_dispatcher_enforcement.json",
    "docs/test-runs/core-engine/operated-assurance/w040-operated-assurance-retained-history-service-001/w040_service_readiness_register.json",
    "docs/test-runs/core-engine/operated-assurance/w040-operated-assurance-retained-history-service-001/w040_exact_service_blocker_register.json",
    "docs/test-runs/core-engine/operated-assurance/w040-operated-assurance-retained-history-service-001/promotion_decision.json",
    "docs/test-runs/core-engine/operated-assurance/w040-operated-assurance-retained-history-service-001/validation.json",
];
const W040_DIVERSITY_SEAM_ARTIFACTS: &[&str] = &[
    "docs/test-runs/core-engine/diversity-seam/w040-independent-evaluator-operated-differential-001/run_summary.json",
    "docs/test-runs/core-engine/diversity-seam/w040-independent-evaluator-operated-differential-001/source_evidence_index.json",
    "docs/test-runs/core-engine/diversity-seam/w040-independent-evaluator-operated-differential-001/w040_independent_scalar_evaluator_implementation.json",
    "docs/test-runs/core-engine/diversity-seam/w040-independent-evaluator-operated-differential-001/w040_independent_evaluator_row_set.json",
    "docs/test-runs/core-engine/diversity-seam/w040-independent-evaluator-operated-differential-001/w040_cross_engine_differential_register.json",
    "docs/test-runs/core-engine/diversity-seam/w040-independent-evaluator-operated-differential-001/w040_differential_authority_register.json",
    "docs/test-runs/core-engine/diversity-seam/w040-independent-evaluator-operated-differential-001/w040_exact_diversity_blocker_register.json",
    "docs/test-runs/core-engine/diversity-seam/w040-independent-evaluator-operated-differential-001/promotion_decision.json",
    "docs/test-runs/core-engine/diversity-seam/w040-independent-evaluator-operated-differential-001/validation.json",
];
const W040_OXFML_SEAM_ARTIFACTS: &[&str] = &[
    "docs/test-runs/core-engine/oxfml-seam/w040-oxfml-seam-breadth-callable-metadata-001/run_summary.json",
    "docs/test-runs/core-engine/oxfml-seam/w040-oxfml-seam-breadth-callable-metadata-001/source_evidence_index.json",
    "docs/test-runs/core-engine/oxfml-seam/w040-oxfml-seam-breadth-callable-metadata-001/w040_oxfml_consumed_surface_register.json",
    "docs/test-runs/core-engine/oxfml-seam/w040-oxfml-seam-breadth-callable-metadata-001/w040_publication_display_boundary_register.json",
    "docs/test-runs/core-engine/oxfml-seam/w040-oxfml-seam-breadth-callable-metadata-001/w040_callable_metadata_implementation_register.json",
    "docs/test-runs/core-engine/oxfml-seam/w040-oxfml-seam-breadth-callable-metadata-001/w040_exact_oxfml_seam_blocker_register.json",
    "docs/test-runs/core-engine/oxfml-seam/w040-oxfml-seam-breadth-callable-metadata-001/promotion_decision.json",
    "docs/test-runs/core-engine/oxfml-seam/w040-oxfml-seam-breadth-callable-metadata-001/validation.json",
];
const W040_UPSTREAM_HOST_ARTIFACTS: &[&str] = &[
    "docs/test-runs/core-engine/upstream-host/w040-oxfml-seam-breadth-callable-metadata-001/run_summary.json",
    "docs/test-runs/core-engine/upstream-host/w040-oxfml-seam-breadth-callable-metadata-001/case_index.json",
];
const W040_SUPPLEMENTAL_EVIDENCE: &[SupplementalEvidenceSpec] = &[
    SupplementalEvidenceSpec {
        input_id: "w040_direct_verification_obligation_map",
        artifact_paths: W040_RELEASE_LEDGER_ARTIFACTS,
        satisfied_input_id: "w040_direct_verification_obligation_map_valid",
        evidence_state_present: "direct_verification_map_present_with_pack_c5_blocked",
        observations: &[
            "W040 direct-verification map records 23 obligations, 11 promotion-target gates, and current W073 typed-only formatting intake.",
            "The map keeps release-grade verification, pack-grade replay, C5, Stage 2, operated service, independent evaluator, broad OxFml, callable metadata, and general OxFunc claims unpromoted until direct gates are satisfied.",
        ],
        reason_ids: &[
            "pack.grade.w040_release_grade_verification_not_promoted",
            "pack.grade.w040_pack_c5_blocked_by_direct_verification_map",
        ],
    },
    SupplementalEvidenceSpec {
        input_id: "w040_optimized_core_exact_blocker_fixes",
        artifact_paths: W040_IMPLEMENTATION_CONFORMANCE_ARTIFACTS,
        satisfied_input_id: "w040_optimized_core_exact_blocker_fixes_valid",
        evidence_state_present: "w040_optimized_core_packet_present_with_exact_blockers",
        observations: &[
            "W040 optimized/core conformance binds dynamic dependency-change evidence and retains exact blockers without declared-gap match promotion.",
            "Automatic dynamic transition detection, snapshot/capability counterparts, and callable metadata projection remain blockers for broader pack claims.",
        ],
        reason_ids: &[
            "pack.grade.w040_optimized_core_engine_conformance_not_full",
            "pack.grade.w040_callable_metadata_projection_absent",
        ],
    },
    SupplementalEvidenceSpec {
        input_id: "w040_rust_totality_refinement",
        artifact_paths: W040_RUST_TOTALITY_ARTIFACTS,
        satisfied_input_id: "w040_rust_totality_refinement_valid",
        evidence_state_present: "w040_rust_totality_refinement_packet_present",
        observations: &[
            "W040 Rust totality/refinement records local checked-proof classification, totality boundaries, refinement rows, and exact blockers.",
            "Whole-engine Rust totality, panic-free core-domain totality, and broad refinement remain unpromoted.",
        ],
        reason_ids: &[
            "pack.grade.w040_rust_totality_not_full",
            "pack.grade.w040_refinement_not_full",
        ],
    },
    SupplementalEvidenceSpec {
        input_id: "w040_lean_tla_full_verification_discharge",
        artifact_paths: W040_LEAN_TLA_ARTIFACTS,
        satisfied_input_id: "w040_lean_tla_full_verification_discharge_valid",
        evidence_state_present: "w040_lean_tla_packet_present_with_exact_blockers",
        observations: &[
            "W040 Lean/TLA records checked proof/model rows, bounded model rows, accepted external seams, totality boundaries, and exact blockers.",
            "Full Lean verification, full TLA verification, fairness and unbounded scheduler coverage, and general OxFunc kernels remain unpromoted.",
        ],
        reason_ids: &[
            "pack.grade.w040_full_lean_tla_verification_not_promoted",
            "pack.grade.w040_general_oxfunc_kernel_not_promoted",
        ],
    },
    SupplementalEvidenceSpec {
        input_id: "w040_stage2_production_policy_equivalence",
        artifact_paths: W040_STAGE2_REPLAY_ARTIFACTS,
        satisfied_input_id: "w040_stage2_production_policy_equivalence_valid",
        evidence_state_present: "w040_stage2_policy_packet_present_no_policy_promotion",
        observations: &[
            "W040 Stage 2 records policy, partition replay, permutation, and observable-invariance rows for bounded profiles.",
            "Stage 2 policy remains unpromoted until full production partition analyzer soundness, fairness/unbounded scheduler coverage, operated cross-engine service evidence, and pack-grade replay governance are present.",
        ],
        reason_ids: &[
            "pack.grade.w040_stage2_policy_unpromoted",
            "pack.grade.w040_stage2_replay_equivalence_not_pack_grade",
            "pack.grade.w040_production_partition_soundness_absent",
            "pack.grade.w040_pack_grade_replay_governance_absent",
        ],
    },
    SupplementalEvidenceSpec {
        input_id: "w040_operated_assurance_retained_history_service",
        artifact_paths: W040_OPERATED_ASSURANCE_ARTIFACTS,
        satisfied_input_id: "w040_operated_assurance_retained_history_service_valid",
        evidence_state_present: "w040_service_artifacts_present_no_service_promotion",
        observations: &[
            "W040 operated assurance adds file-backed runner, retained-history store/query, replay-correlation, and local dispatcher artifacts.",
            "Operated continuous assurance, retained-history service, external alert/quarantine dispatcher, and operated cross-engine differential service remain unpromoted.",
        ],
        reason_ids: &[
            "pack.grade.w040_operated_continuous_assurance_service_absent",
            "pack.grade.w040_retained_history_service_absent",
            "pack.grade.w040_external_alert_dispatcher_absent",
            "pack.grade.w040_operated_cross_engine_diff_service_absent",
        ],
    },
    SupplementalEvidenceSpec {
        input_id: "w040_independent_evaluator_operated_differential",
        artifact_paths: W040_DIVERSITY_SEAM_ARTIFACTS,
        satisfied_input_id: "w040_independent_evaluator_operated_differential_valid",
        evidence_state_present: "w040_bounded_independent_evaluator_present_no_diversity_promotion",
        observations: &[
            "W040 diversity adds a bounded independent scalar arithmetic evaluator and cross-engine differential rows.",
            "Full independent-evaluator breadth, operated cross-engine differential service, mismatch triage/quarantine service, and release-grade authority remain unpromoted.",
        ],
        reason_ids: &[
            "pack.grade.w040_fully_independent_evaluator_absent",
            "pack.grade.w040_independent_evaluator_breadth_absent",
            "pack.grade.w040_operated_cross_engine_diff_service_absent",
        ],
    },
    SupplementalEvidenceSpec {
        input_id: "w040_oxfml_seam_breadth_callable_metadata",
        artifact_paths: W040_OXFML_SEAM_ARTIFACTS,
        satisfied_input_id: "w040_oxfml_seam_breadth_callable_metadata_valid",
        evidence_state_present: "w040_oxfml_seam_packet_present_with_callable_blockers",
        observations: &[
            "W040 OxFml seam records 8 source rows, 6 consumed-surface rows, 5 publication/display rows, 6 callable rows, 6 exact blockers, and 0 failed rows.",
            "The current W073 typed-only formatting guard, public consumer notes, format/display boundary, LET/LAMBDA carrier rows, fixture-host and registered-external watch rows, and callable metadata blockers are bound without broad OxFml or callable metadata promotion.",
        ],
        reason_ids: &[
            "pack.grade.w040_broad_oxfml_display_publication_unpromoted",
            "pack.grade.w040_public_consumer_migration_not_verified",
            "pack.grade.w040_callable_metadata_projection_absent",
            "pack.grade.w040_callable_carrier_sufficiency_proof_absent",
            "pack.grade.w040_general_oxfunc_kernel_not_promoted",
        ],
    },
    SupplementalEvidenceSpec {
        input_id: "w040_fresh_direct_oxfml_upstream_host",
        artifact_paths: W040_UPSTREAM_HOST_ARTIFACTS,
        satisfied_input_id: "w040_fresh_direct_oxfml_upstream_host_valid",
        evidence_state_present: "current_w040_direct_oxfml_run_present_no_pack_promotion",
        observations: &[
            "W040 upstream-host records 12 cases, 3 direct OxFml cases, 2 LET/LAMBDA cases, 1 W073 typed-rule formatting guard, and 0 expectation mismatches.",
            "The direct OxFml slice preserves current W073 and LET/LAMBDA evidence but does not promote pack-grade replay, C5, broad OxFml, callable metadata, or general OxFunc kernels.",
        ],
        reason_ids: &[],
    },
];
const W041_TREECALC_CONFORMANCE_RUN_ID: &str =
    "w041-optimized-core-automatic-dynamic-transition-001";
const W041_FORMAL_ARTIFACTS: &[&str] = &[
    "docs/spec/core-engine/w041-formalization/W041_RUST_TOTALITY_REFINEMENT_AND_PANIC_BOUNDARY_DISCHARGE.md",
    "docs/spec/core-engine/w041-formalization/W041_LEAN_TLA_FULL_VERIFICATION_AND_FAIRNESS_DISCHARGE.md",
    "docs/spec/core-engine/w041-formalization/W041_STAGE2_PRODUCTION_ANALYZER_AND_PACK_EQUIVALENCE_PROOF_TRANCHE.md",
    "formal/lean/OxCalc/CoreEngine/W041RustTotalityAndRefinement.lean",
    "formal/lean/OxCalc/CoreEngine/W041LeanTlaFullVerificationAndFairnessDischarge.lean",
    "formal/lean/OxCalc/CoreEngine/W041Stage2ProductionAnalyzerAndPackEquivalence.lean",
];
const W041_FORMATTING_WATCH_ARTIFACTS: &[&str] = &[
    "docs/spec/core-engine/w041-formalization/W041_OXFML_BROAD_DISPLAY_PUBLICATION_AND_CALLABLE_CARRIER_CLOSURE.md",
    "docs/test-runs/core-engine/release-grade-ledger/w041-residual-release-grade-successor-obligation-map-001/w073_formatting_intake.json",
];
const W041_RELEASE_LEDGER_ARTIFACTS: &[&str] = &[
    "docs/test-runs/core-engine/release-grade-ledger/w041-residual-release-grade-successor-obligation-map-001/run_summary.json",
    "docs/test-runs/core-engine/release-grade-ledger/w041-residual-release-grade-successor-obligation-map-001/source_evidence_index.json",
    "docs/test-runs/core-engine/release-grade-ledger/w041-residual-release-grade-successor-obligation-map-001/successor_obligation_map.json",
    "docs/test-runs/core-engine/release-grade-ledger/w041-residual-release-grade-successor-obligation-map-001/promotion_target_gate_map.json",
    "docs/test-runs/core-engine/release-grade-ledger/w041-residual-release-grade-successor-obligation-map-001/w073_formatting_intake.json",
    "docs/test-runs/core-engine/release-grade-ledger/w041-residual-release-grade-successor-obligation-map-001/validation.json",
];
const W041_IMPLEMENTATION_CONFORMANCE_ARTIFACTS: &[&str] = &[
    "docs/test-runs/core-engine/implementation-conformance/w041-optimized-core-residual-blocker-differentials-001/run_summary.json",
    "docs/test-runs/core-engine/implementation-conformance/w041-optimized-core-residual-blocker-differentials-001/source_evidence_index.json",
    "docs/test-runs/core-engine/implementation-conformance/w041-optimized-core-residual-blocker-differentials-001/dynamic_release_reclassification_auto_transition_evidence.json",
    "docs/test-runs/core-engine/implementation-conformance/w041-optimized-core-residual-blocker-differentials-001/w041_exact_blocker_disposition_register.json",
    "docs/test-runs/core-engine/implementation-conformance/w041-optimized-core-residual-blocker-differentials-001/w041_exact_remaining_blocker_register.json",
    "docs/test-runs/core-engine/implementation-conformance/w041-optimized-core-residual-blocker-differentials-001/w041_match_promotion_guard.json",
    "docs/test-runs/core-engine/implementation-conformance/w041-optimized-core-residual-blocker-differentials-001/validation.json",
];
const W041_RUST_TOTALITY_ARTIFACTS: &[&str] = &[
    "docs/test-runs/core-engine/formal-assurance/w041-rust-totality-refinement-proof-tranche-001/run_summary.json",
    "docs/test-runs/core-engine/formal-assurance/w041-rust-totality-refinement-proof-tranche-001/source_evidence_index.json",
    "docs/test-runs/core-engine/formal-assurance/w041-rust-totality-refinement-proof-tranche-001/w041_rust_totality_refinement_ledger.json",
    "docs/test-runs/core-engine/formal-assurance/w041-rust-totality-refinement-proof-tranche-001/w041_rust_totality_boundary_register.json",
    "docs/test-runs/core-engine/formal-assurance/w041-rust-totality-refinement-proof-tranche-001/w041_rust_refinement_register.json",
    "docs/test-runs/core-engine/formal-assurance/w041-rust-totality-refinement-proof-tranche-001/w041_rust_exact_blocker_register.json",
    "docs/test-runs/core-engine/formal-assurance/w041-rust-totality-refinement-proof-tranche-001/validation.json",
];
const W041_LEAN_TLA_ARTIFACTS: &[&str] = &[
    "docs/test-runs/core-engine/formal-assurance/w041-lean-tla-full-verification-fairness-discharge-001/run_summary.json",
    "docs/test-runs/core-engine/formal-assurance/w041-lean-tla-full-verification-fairness-discharge-001/source_evidence_index.json",
    "docs/test-runs/core-engine/formal-assurance/w041-lean-tla-full-verification-fairness-discharge-001/w041_lean_tla_discharge_ledger.json",
    "docs/test-runs/core-engine/formal-assurance/w041-lean-tla-full-verification-fairness-discharge-001/w041_lean_proof_register.json",
    "docs/test-runs/core-engine/formal-assurance/w041-lean-tla-full-verification-fairness-discharge-001/w041_tla_model_bound_register.json",
    "docs/test-runs/core-engine/formal-assurance/w041-lean-tla-full-verification-fairness-discharge-001/w041_lean_tla_exact_blocker_register.json",
    "docs/test-runs/core-engine/formal-assurance/w041-lean-tla-full-verification-fairness-discharge-001/validation.json",
];
const W041_STAGE2_REPLAY_ARTIFACTS: &[&str] = &[
    "docs/test-runs/core-engine/stage2-replay/w041-stage2-production-analyzer-pack-equivalence-001/run_summary.json",
    "docs/test-runs/core-engine/stage2-replay/w041-stage2-production-analyzer-pack-equivalence-001/source_evidence_index.json",
    "docs/test-runs/core-engine/stage2-replay/w041-stage2-production-analyzer-pack-equivalence-001/w041_stage2_policy_gate_register.json",
    "docs/test-runs/core-engine/stage2-replay/w041-stage2-production-analyzer-pack-equivalence-001/w041_production_analyzer_soundness_register.json",
    "docs/test-runs/core-engine/stage2-replay/w041-stage2-production-analyzer-pack-equivalence-001/w041_pack_equivalence_register.json",
    "docs/test-runs/core-engine/stage2-replay/w041-stage2-production-analyzer-pack-equivalence-001/w041_stage2_exact_blocker_register.json",
    "docs/test-runs/core-engine/stage2-replay/w041-stage2-production-analyzer-pack-equivalence-001/promotion_decision.json",
    "docs/test-runs/core-engine/stage2-replay/w041-stage2-production-analyzer-pack-equivalence-001/validation.json",
];
const W041_OPERATED_ASSURANCE_ARTIFACTS: &[&str] = &[
    "docs/test-runs/core-engine/operated-assurance/w041-operated-assurance-retained-history-alert-dispatch-001/run_summary.json",
    "docs/test-runs/core-engine/operated-assurance/w041-operated-assurance-retained-history-alert-dispatch-001/source_evidence_index.json",
    "docs/test-runs/core-engine/operated-assurance/w041-operated-assurance-retained-history-alert-dispatch-001/w041_operated_service_envelope.json",
    "docs/test-runs/core-engine/operated-assurance/w041-operated-assurance-retained-history-alert-dispatch-001/w041_retained_history_service_query.json",
    "docs/test-runs/core-engine/operated-assurance/w041-operated-assurance-retained-history-alert-dispatch-001/w041_cross_engine_service_register.json",
    "docs/test-runs/core-engine/operated-assurance/w041-operated-assurance-retained-history-alert-dispatch-001/w041_retained_witness_lifecycle_register.json",
    "docs/test-runs/core-engine/operated-assurance/w041-operated-assurance-retained-history-alert-dispatch-001/w041_alert_dispatch_service_register.json",
    "docs/test-runs/core-engine/operated-assurance/w041-operated-assurance-retained-history-alert-dispatch-001/w041_service_readiness_register.json",
    "docs/test-runs/core-engine/operated-assurance/w041-operated-assurance-retained-history-alert-dispatch-001/w041_exact_service_blocker_register.json",
    "docs/test-runs/core-engine/operated-assurance/w041-operated-assurance-retained-history-alert-dispatch-001/promotion_decision.json",
    "docs/test-runs/core-engine/operated-assurance/w041-operated-assurance-retained-history-alert-dispatch-001/validation.json",
];
const W041_DIVERSITY_SEAM_ARTIFACTS: &[&str] = &[
    "docs/test-runs/core-engine/diversity-seam/w041-independent-evaluator-breadth-operated-differential-001/run_summary.json",
    "docs/test-runs/core-engine/diversity-seam/w041-independent-evaluator-breadth-operated-differential-001/source_evidence_index.json",
    "docs/test-runs/core-engine/diversity-seam/w041-independent-evaluator-breadth-operated-differential-001/w041_independent_formula_evaluator_implementation.json",
    "docs/test-runs/core-engine/diversity-seam/w041-independent-evaluator-breadth-operated-differential-001/w041_independent_evaluator_breadth_register.json",
    "docs/test-runs/core-engine/diversity-seam/w041-independent-evaluator-breadth-operated-differential-001/w041_cross_engine_differential_service_register.json",
    "docs/test-runs/core-engine/diversity-seam/w041-independent-evaluator-breadth-operated-differential-001/w041_mismatch_authority_router.json",
    "docs/test-runs/core-engine/diversity-seam/w041-independent-evaluator-breadth-operated-differential-001/w041_exact_diversity_blocker_register.json",
    "docs/test-runs/core-engine/diversity-seam/w041-independent-evaluator-breadth-operated-differential-001/promotion_decision.json",
    "docs/test-runs/core-engine/diversity-seam/w041-independent-evaluator-breadth-operated-differential-001/validation.json",
];
const W041_OXFML_SEAM_ARTIFACTS: &[&str] = &[
    "docs/test-runs/core-engine/oxfml-seam/w041-oxfml-broad-display-publication-callable-carrier-001/run_summary.json",
    "docs/test-runs/core-engine/oxfml-seam/w041-oxfml-broad-display-publication-callable-carrier-001/source_evidence_index.json",
    "docs/test-runs/core-engine/oxfml-seam/w041-oxfml-broad-display-publication-callable-carrier-001/w041_oxfml_consumed_surface_register.json",
    "docs/test-runs/core-engine/oxfml-seam/w041-oxfml-broad-display-publication-callable-carrier-001/w041_publication_display_boundary_register.json",
    "docs/test-runs/core-engine/oxfml-seam/w041-oxfml-broad-display-publication-callable-carrier-001/w041_callable_carrier_and_metadata_register.json",
    "docs/test-runs/core-engine/oxfml-seam/w041-oxfml-broad-display-publication-callable-carrier-001/w041_registered_external_provider_register.json",
    "docs/test-runs/core-engine/oxfml-seam/w041-oxfml-broad-display-publication-callable-carrier-001/w041_exact_oxfml_seam_blocker_register.json",
    "docs/test-runs/core-engine/oxfml-seam/w041-oxfml-broad-display-publication-callable-carrier-001/promotion_decision.json",
    "docs/test-runs/core-engine/oxfml-seam/w041-oxfml-broad-display-publication-callable-carrier-001/validation.json",
];
const W041_UPSTREAM_HOST_ARTIFACTS: &[&str] = &[
    "docs/test-runs/core-engine/upstream-host/w041-oxfml-broad-display-callable-carrier-001/run_summary.json",
    "docs/test-runs/core-engine/upstream-host/w041-oxfml-broad-display-callable-carrier-001/case_index.json",
];
const W041_SUPPLEMENTAL_EVIDENCE: &[SupplementalEvidenceSpec] = &[
    SupplementalEvidenceSpec {
        input_id: "w041_successor_obligation_map",
        artifact_paths: W041_RELEASE_LEDGER_ARTIFACTS,
        satisfied_input_id: "w041_successor_obligation_map_valid",
        evidence_state_present: "w041_successor_map_present_with_pack_c5_blocked",
        observations: &[
            "W041 successor map records 28 obligations, 12 promotion-target gates, and current W073 typed-only formatting intake.",
            "The map keeps release-grade verification, pack-grade replay, C5, Stage 2, operated service, independent evaluator, broad OxFml, callable metadata, callable carrier, and general OxFunc claims unpromoted unless direct gates are satisfied.",
        ],
        reason_ids: &[
            "pack.grade.w041_release_grade_verification_not_promoted",
            "pack.grade.w041_pack_c5_blocked_by_successor_obligation_map",
        ],
    },
    SupplementalEvidenceSpec {
        input_id: "w041_optimized_core_residual_blocker_differentials",
        artifact_paths: W041_IMPLEMENTATION_CONFORMANCE_ARTIFACTS,
        satisfied_input_id: "w041_optimized_core_residual_blocker_differentials_valid",
        evidence_state_present: "w041_optimized_core_packet_present_with_exact_blockers",
        observations: &[
            "W041 optimized/core conformance binds automatic dynamic transition evidence and retains exact blockers without declared-gap match promotion.",
            "Full optimized/core verification and callable metadata projection remain blockers for broader pack claims.",
        ],
        reason_ids: &[
            "pack.grade.w041_optimized_core_engine_conformance_not_full",
            "pack.grade.w041_callable_metadata_projection_absent",
        ],
    },
    SupplementalEvidenceSpec {
        input_id: "w041_rust_totality_refinement",
        artifact_paths: W041_RUST_TOTALITY_ARTIFACTS,
        satisfied_input_id: "w041_rust_totality_refinement_valid",
        evidence_state_present: "w041_rust_totality_refinement_packet_present",
        observations: &[
            "W041 Rust totality/refinement records local checked-proof classification, totality boundaries, refinement rows, automatic dynamic transition refinement, and exact blockers.",
            "Whole-engine Rust totality, panic-free core-domain totality, and broad refinement remain unpromoted.",
        ],
        reason_ids: &[
            "pack.grade.w041_rust_totality_not_full",
            "pack.grade.w041_refinement_not_full",
        ],
    },
    SupplementalEvidenceSpec {
        input_id: "w041_lean_tla_full_verification_fairness",
        artifact_paths: W041_LEAN_TLA_ARTIFACTS,
        satisfied_input_id: "w041_lean_tla_full_verification_fairness_valid",
        evidence_state_present: "w041_lean_tla_packet_present_with_exact_blockers",
        observations: &[
            "W041 Lean/TLA records checked proof/model rows, bounded model rows, accepted external seams, totality boundaries, and exact blockers.",
            "Full Lean verification, full TLA verification, fairness and unbounded scheduler coverage, and general OxFunc kernels remain unpromoted.",
        ],
        reason_ids: &[
            "pack.grade.w041_full_lean_tla_verification_not_promoted",
            "pack.grade.w041_fairness_unbounded_scheduler_coverage_absent",
            "pack.grade.w041_general_oxfunc_kernel_not_promoted",
        ],
    },
    SupplementalEvidenceSpec {
        input_id: "w041_stage2_production_analyzer_pack_equivalence",
        artifact_paths: W041_STAGE2_REPLAY_ARTIFACTS,
        satisfied_input_id: "w041_stage2_production_analyzer_pack_equivalence_valid",
        evidence_state_present: "w041_stage2_packet_present_no_policy_or_pack_promotion",
        observations: &[
            "W041 Stage 2 records policy, partition replay, permutation, observable-invariance, and declared pack-equivalence rows for bounded profiles.",
            "Stage 2 policy remains unpromoted until full production partition-analyzer soundness, fairness/unbounded scheduler coverage, operated cross-engine service evidence, and pack-grade replay governance are present.",
        ],
        reason_ids: &[
            "pack.grade.w041_stage2_policy_unpromoted",
            "pack.grade.w041_stage2_replay_equivalence_not_pack_grade",
            "pack.grade.w041_production_partition_soundness_absent",
            "pack.grade.w041_pack_grade_replay_governance_absent",
        ],
    },
    SupplementalEvidenceSpec {
        input_id: "w041_operated_assurance_retained_history_alert_dispatch",
        artifact_paths: W041_OPERATED_ASSURANCE_ARTIFACTS,
        satisfied_input_id: "w041_operated_assurance_retained_history_alert_dispatch_valid",
        evidence_state_present: "w041_service_envelope_present_no_service_promotion",
        observations: &[
            "W041 operated assurance adds file-backed service envelope, retained-history query contract, retained-witness lifecycle register, replay-correlation rows, and alert/quarantine service-contract rows.",
            "Operated continuous assurance, retained-history service, retained-witness lifecycle service, external alert/quarantine dispatcher, and operated cross-engine differential service remain unpromoted.",
        ],
        reason_ids: &[
            "pack.grade.w041_operated_continuous_assurance_service_absent",
            "pack.grade.w041_retained_history_service_absent",
            "pack.grade.w041_retained_witness_lifecycle_service_absent",
            "pack.grade.w041_external_alert_dispatcher_absent",
            "pack.grade.w041_operated_cross_engine_diff_service_absent",
        ],
    },
    SupplementalEvidenceSpec {
        input_id: "w041_independent_evaluator_breadth_operated_differential",
        artifact_paths: W041_DIVERSITY_SEAM_ARTIFACTS,
        satisfied_input_id: "w041_independent_evaluator_breadth_operated_differential_valid",
        evidence_state_present: "w041_broadened_independent_formula_fragment_present_no_diversity_promotion",
        observations: &[
            "W041 diversity adds a broadened independent formula-fragment evaluator and cross-engine differential/service rows.",
            "Full independent-evaluator breadth, operated cross-engine differential service, mismatch triage/quarantine service, and release-grade authority remain unpromoted.",
        ],
        reason_ids: &[
            "pack.grade.w041_fully_independent_evaluator_absent",
            "pack.grade.w041_independent_evaluator_breadth_absent",
            "pack.grade.w041_operated_cross_engine_diff_service_absent",
            "pack.grade.w041_mismatch_quarantine_service_absent",
        ],
    },
    SupplementalEvidenceSpec {
        input_id: "w041_oxfml_broad_display_publication_callable_carrier",
        artifact_paths: W041_OXFML_SEAM_ARTIFACTS,
        satisfied_input_id: "w041_oxfml_broad_display_publication_callable_carrier_valid",
        evidence_state_present: "w041_oxfml_seam_packet_present_with_callable_provider_blockers",
        observations: &[
            "W041 OxFml seam records 12 source rows, 10 consumed-surface rows, 8 publication/display rows, 8 callable rows, 6 registered-external/provider rows, 8 exact blockers, and 0 failed rows.",
            "The current W073 typed-only formatting guard, public consumer notes, distinct format/display boundary, LET/LAMBDA carrier rows, registered-external/provider watch rows, and callable blockers are bound without broad OxFml or callable promotion.",
        ],
        reason_ids: &[
            "pack.grade.w041_broad_oxfml_display_publication_unpromoted",
            "pack.grade.w041_public_consumer_migration_not_verified",
            "pack.grade.w041_callable_metadata_projection_absent",
            "pack.grade.w041_callable_carrier_sufficiency_proof_absent",
            "pack.grade.w041_registered_external_projection_absent",
            "pack.grade.w041_provider_failure_callable_publication_watch_lane",
            "pack.grade.w041_general_oxfunc_kernel_not_promoted",
        ],
    },
    SupplementalEvidenceSpec {
        input_id: "w041_fresh_direct_oxfml_upstream_host",
        artifact_paths: W041_UPSTREAM_HOST_ARTIFACTS,
        satisfied_input_id: "w041_fresh_direct_oxfml_upstream_host_valid",
        evidence_state_present: "current_w041_direct_oxfml_run_present_no_pack_promotion",
        observations: &[
            "W041 upstream-host records 12 cases, 3 direct OxFml cases, 2 LET/LAMBDA cases, 1 W073 typed-rule formatting guard, and 0 expectation mismatches.",
            "The direct OxFml slice preserves current W073 and LET/LAMBDA evidence but does not promote pack-grade replay, C5, broad OxFml, callable metadata, or general OxFunc kernels.",
        ],
        reason_ids: &[],
    },
];
const W042_TREECALC_CONFORMANCE_RUN_ID: &str =
    "w042-optimized-core-counterpart-conformance-treecalc-001";
const W042_FORMAL_ARTIFACTS: &[&str] = &[
    "docs/spec/core-engine/w042-formalization/W042_RUST_TOTALITY_REFINEMENT_AND_CORE_PANIC_BOUNDARY_CLOSURE.md",
    "docs/spec/core-engine/w042-formalization/W042_LEAN_TLA_FAIRNESS_AND_FULL_VERIFICATION_EXPANSION.md",
    "docs/spec/core-engine/w042-formalization/W042_STAGE2_PRODUCTION_ANALYZER_AND_PACK_GRADE_EQUIVALENCE_CLOSURE.md",
    "formal/lean/OxCalc/CoreEngine/W042RustTotalityAndRefinement.lean",
    "formal/lean/OxCalc/CoreEngine/W042LeanTlaFairnessFullVerificationExpansion.lean",
    "formal/lean/OxCalc/CoreEngine/W042Stage2ProductionAnalyzerAndPackGradeEquivalence.lean",
];
const W042_FORMATTING_WATCH_ARTIFACTS: &[&str] = &[
    "docs/spec/core-engine/w042-formalization/W042_OXFML_PUBLIC_MIGRATION_CALLABLE_CARRIER_AND_REGISTERED_EXTERNAL_CLOSURE.md",
    "docs/test-runs/core-engine/release-grade-ledger/w042-residual-release-grade-closure-obligation-ledger-001/w073_formatting_intake.json",
];
const W042_RELEASE_LEDGER_ARTIFACTS: &[&str] = &[
    "docs/test-runs/core-engine/release-grade-ledger/w042-residual-release-grade-closure-obligation-ledger-001/run_summary.json",
    "docs/test-runs/core-engine/release-grade-ledger/w042-residual-release-grade-closure-obligation-ledger-001/source_evidence_index.json",
    "docs/test-runs/core-engine/release-grade-ledger/w042-residual-release-grade-closure-obligation-ledger-001/closure_obligation_map.json",
    "docs/test-runs/core-engine/release-grade-ledger/w042-residual-release-grade-closure-obligation-ledger-001/promotion_target_gate_map.json",
    "docs/test-runs/core-engine/release-grade-ledger/w042-residual-release-grade-closure-obligation-ledger-001/w073_formatting_intake.json",
    "docs/test-runs/core-engine/release-grade-ledger/w042-residual-release-grade-closure-obligation-ledger-001/validation.json",
];
const W042_IMPLEMENTATION_CONFORMANCE_ARTIFACTS: &[&str] = &[
    "docs/test-runs/core-engine/implementation-conformance/w042-optimized-core-counterpart-conformance-callable-metadata-001/run_summary.json",
    "docs/test-runs/core-engine/implementation-conformance/w042-optimized-core-counterpart-conformance-callable-metadata-001/source_evidence_index.json",
    "docs/test-runs/core-engine/implementation-conformance/w042-optimized-core-counterpart-conformance-callable-metadata-001/w042_counterpart_conformance_register.json",
    "docs/test-runs/core-engine/implementation-conformance/w042-optimized-core-counterpart-conformance-callable-metadata-001/w042_callable_metadata_projection_register.json",
    "docs/test-runs/core-engine/implementation-conformance/w042-optimized-core-counterpart-conformance-callable-metadata-001/w042_exact_remaining_blocker_register.json",
    "docs/test-runs/core-engine/implementation-conformance/w042-optimized-core-counterpart-conformance-callable-metadata-001/w042_match_promotion_guard.json",
    "docs/test-runs/core-engine/implementation-conformance/w042-optimized-core-counterpart-conformance-callable-metadata-001/validation.json",
];
const W042_RUST_TOTALITY_ARTIFACTS: &[&str] = &[
    "docs/test-runs/core-engine/formal-assurance/w042-rust-totality-refinement-core-panic-boundary-001/run_summary.json",
    "docs/test-runs/core-engine/formal-assurance/w042-rust-totality-refinement-core-panic-boundary-001/source_evidence_index.json",
    "docs/test-runs/core-engine/formal-assurance/w042-rust-totality-refinement-core-panic-boundary-001/w042_rust_totality_refinement_ledger.json",
    "docs/test-runs/core-engine/formal-assurance/w042-rust-totality-refinement-core-panic-boundary-001/w042_rust_totality_boundary_register.json",
    "docs/test-runs/core-engine/formal-assurance/w042-rust-totality-refinement-core-panic-boundary-001/w042_rust_refinement_register.json",
    "docs/test-runs/core-engine/formal-assurance/w042-rust-totality-refinement-core-panic-boundary-001/w042_rust_exact_blocker_register.json",
    "docs/test-runs/core-engine/formal-assurance/w042-rust-totality-refinement-core-panic-boundary-001/validation.json",
];
const W042_LEAN_TLA_ARTIFACTS: &[&str] = &[
    "docs/test-runs/core-engine/formal-assurance/w042-lean-tla-fairness-full-verification-expansion-001/run_summary.json",
    "docs/test-runs/core-engine/formal-assurance/w042-lean-tla-fairness-full-verification-expansion-001/source_evidence_index.json",
    "docs/test-runs/core-engine/formal-assurance/w042-lean-tla-fairness-full-verification-expansion-001/w042_lean_tla_discharge_ledger.json",
    "docs/test-runs/core-engine/formal-assurance/w042-lean-tla-fairness-full-verification-expansion-001/w042_lean_proof_register.json",
    "docs/test-runs/core-engine/formal-assurance/w042-lean-tla-fairness-full-verification-expansion-001/w042_tla_model_bound_register.json",
    "docs/test-runs/core-engine/formal-assurance/w042-lean-tla-fairness-full-verification-expansion-001/w042_lean_tla_exact_blocker_register.json",
    "docs/test-runs/core-engine/formal-assurance/w042-lean-tla-fairness-full-verification-expansion-001/validation.json",
];
const W042_STAGE2_REPLAY_ARTIFACTS: &[&str] = &[
    "docs/test-runs/core-engine/stage2-replay/w042-stage2-production-analyzer-pack-grade-equivalence-closure-001/run_summary.json",
    "docs/test-runs/core-engine/stage2-replay/w042-stage2-production-analyzer-pack-grade-equivalence-closure-001/source_evidence_index.json",
    "docs/test-runs/core-engine/stage2-replay/w042-stage2-production-analyzer-pack-grade-equivalence-closure-001/w042_stage2_policy_gate_register.json",
    "docs/test-runs/core-engine/stage2-replay/w042-stage2-production-analyzer-pack-grade-equivalence-closure-001/w042_production_analyzer_soundness_register.json",
    "docs/test-runs/core-engine/stage2-replay/w042-stage2-production-analyzer-pack-grade-equivalence-closure-001/w042_pack_grade_equivalence_register.json",
    "docs/test-runs/core-engine/stage2-replay/w042-stage2-production-analyzer-pack-grade-equivalence-closure-001/w042_stage2_exact_blocker_register.json",
    "docs/test-runs/core-engine/stage2-replay/w042-stage2-production-analyzer-pack-grade-equivalence-closure-001/promotion_decision.json",
    "docs/test-runs/core-engine/stage2-replay/w042-stage2-production-analyzer-pack-grade-equivalence-closure-001/validation.json",
];
const W042_OPERATED_ASSURANCE_ARTIFACTS: &[&str] = &[
    "docs/test-runs/core-engine/operated-assurance/w042-operated-assurance-retained-history-retained-witness-alert-service-001/run_summary.json",
    "docs/test-runs/core-engine/operated-assurance/w042-operated-assurance-retained-history-retained-witness-alert-service-001/source_evidence_index.json",
    "docs/test-runs/core-engine/operated-assurance/w042-operated-assurance-retained-history-retained-witness-alert-service-001/w042_operated_service_envelope.json",
    "docs/test-runs/core-engine/operated-assurance/w042-operated-assurance-retained-history-retained-witness-alert-service-001/w042_retained_history_service_query.json",
    "docs/test-runs/core-engine/operated-assurance/w042-operated-assurance-retained-history-retained-witness-alert-service-001/w042_cross_engine_service_register.json",
    "docs/test-runs/core-engine/operated-assurance/w042-operated-assurance-retained-history-retained-witness-alert-service-001/w042_retained_witness_lifecycle_register.json",
    "docs/test-runs/core-engine/operated-assurance/w042-operated-assurance-retained-history-retained-witness-alert-service-001/w042_alert_dispatch_service_register.json",
    "docs/test-runs/core-engine/operated-assurance/w042-operated-assurance-retained-history-retained-witness-alert-service-001/w042_service_readiness_register.json",
    "docs/test-runs/core-engine/operated-assurance/w042-operated-assurance-retained-history-retained-witness-alert-service-001/w042_exact_service_blocker_register.json",
    "docs/test-runs/core-engine/operated-assurance/w042-operated-assurance-retained-history-retained-witness-alert-service-001/promotion_decision.json",
    "docs/test-runs/core-engine/operated-assurance/w042-operated-assurance-retained-history-retained-witness-alert-service-001/validation.json",
];
const W042_DIVERSITY_SEAM_ARTIFACTS: &[&str] = &[
    "docs/test-runs/core-engine/diversity-seam/w042-independent-evaluator-breadth-mismatch-quarantine-operated-differential-001/run_summary.json",
    "docs/test-runs/core-engine/diversity-seam/w042-independent-evaluator-breadth-mismatch-quarantine-operated-differential-001/source_evidence_index.json",
    "docs/test-runs/core-engine/diversity-seam/w042-independent-evaluator-breadth-mismatch-quarantine-operated-differential-001/w042_independent_reference_model_implementation.json",
    "docs/test-runs/core-engine/diversity-seam/w042-independent-evaluator-breadth-mismatch-quarantine-operated-differential-001/w042_independent_evaluator_breadth_register.json",
    "docs/test-runs/core-engine/diversity-seam/w042-independent-evaluator-breadth-mismatch-quarantine-operated-differential-001/w042_cross_engine_differential_service_register.json",
    "docs/test-runs/core-engine/diversity-seam/w042-independent-evaluator-breadth-mismatch-quarantine-operated-differential-001/w042_mismatch_quarantine_authority_router.json",
    "docs/test-runs/core-engine/diversity-seam/w042-independent-evaluator-breadth-mismatch-quarantine-operated-differential-001/w042_exact_diversity_blocker_register.json",
    "docs/test-runs/core-engine/diversity-seam/w042-independent-evaluator-breadth-mismatch-quarantine-operated-differential-001/promotion_decision.json",
    "docs/test-runs/core-engine/diversity-seam/w042-independent-evaluator-breadth-mismatch-quarantine-operated-differential-001/validation.json",
];
const W042_OXFML_SEAM_ARTIFACTS: &[&str] = &[
    "docs/test-runs/core-engine/oxfml-seam/w042-oxfml-public-migration-callable-carrier-registered-external-001/run_summary.json",
    "docs/test-runs/core-engine/oxfml-seam/w042-oxfml-public-migration-callable-carrier-registered-external-001/source_evidence_index.json",
    "docs/test-runs/core-engine/oxfml-seam/w042-oxfml-public-migration-callable-carrier-registered-external-001/w042_oxfml_consumed_surface_register.json",
    "docs/test-runs/core-engine/oxfml-seam/w042-oxfml-public-migration-callable-carrier-registered-external-001/w042_publication_display_boundary_register.json",
    "docs/test-runs/core-engine/oxfml-seam/w042-oxfml-public-migration-callable-carrier-registered-external-001/w042_callable_carrier_and_metadata_register.json",
    "docs/test-runs/core-engine/oxfml-seam/w042-oxfml-public-migration-callable-carrier-registered-external-001/w042_registered_external_provider_register.json",
    "docs/test-runs/core-engine/oxfml-seam/w042-oxfml-public-migration-callable-carrier-registered-external-001/w042_exact_oxfml_seam_blocker_register.json",
    "docs/test-runs/core-engine/oxfml-seam/w042-oxfml-public-migration-callable-carrier-registered-external-001/promotion_decision.json",
    "docs/test-runs/core-engine/oxfml-seam/w042-oxfml-public-migration-callable-carrier-registered-external-001/validation.json",
];
const W042_UPSTREAM_HOST_ARTIFACTS: &[&str] = &[
    "docs/test-runs/core-engine/upstream-host/w042-oxfml-public-migration-callable-carrier-registered-external-001/run_summary.json",
    "docs/test-runs/core-engine/upstream-host/w042-oxfml-public-migration-callable-carrier-registered-external-001/case_index.json",
];
const W042_SUPPLEMENTAL_EVIDENCE: &[SupplementalEvidenceSpec] = &[
    SupplementalEvidenceSpec {
        input_id: "w042_closure_obligation_map",
        artifact_paths: W042_RELEASE_LEDGER_ARTIFACTS,
        satisfied_input_id: "w042_closure_obligation_map_valid",
        evidence_state_present: "w042_closure_map_present_with_pack_c5_blocked",
        observations: &[
            "W042 closure map records 33 obligations, 14 promotion-target gates, and current W073 typed-only formatting intake.",
            "The map keeps release-grade verification, pack-grade replay, C5, Stage 2, operated services, retained history/witness services, independent evaluator, broad OxFml, callable, registered-external, provider publication, and general OxFunc claims unpromoted unless direct gates are satisfied.",
        ],
        reason_ids: &[
            "pack.grade.w042_release_grade_verification_not_promoted",
            "pack.grade.w042_pack_c5_blocked_by_closure_obligation_map",
        ],
    },
    SupplementalEvidenceSpec {
        input_id: "w042_optimized_core_counterpart_conformance",
        artifact_paths: W042_IMPLEMENTATION_CONFORMANCE_ARTIFACTS,
        satisfied_input_id: "w042_optimized_core_counterpart_conformance_valid",
        evidence_state_present: "w042_counterpart_packet_present_with_exact_blockers",
        observations: &[
            "W042 optimized/core counterpart conformance binds a 26-case TreeCalc replay, 5 direct-evidence rows, 3 exact remaining blockers, 0 match promotions, and 0 failed rows.",
            "Declared-profile counterparts and callable value carriers are evidenced, but full optimized/core verification and callable metadata projection remain blocked.",
        ],
        reason_ids: &[
            "pack.grade.w042_optimized_core_engine_conformance_not_full",
            "pack.grade.w042_broader_dynamic_transition_coverage_absent",
            "pack.grade.w042_callable_metadata_projection_absent",
        ],
    },
    SupplementalEvidenceSpec {
        input_id: "w042_rust_totality_refinement_core_panic_boundary",
        artifact_paths: W042_RUST_TOTALITY_ARTIFACTS,
        satisfied_input_id: "w042_rust_totality_refinement_valid",
        evidence_state_present: "w042_rust_packet_present_with_totality_boundaries",
        observations: &[
            "W042 Rust totality/refinement records 13 rows, 10 local checked-proof rows, 4 totality boundaries, 7 refinement rows, 4 exact blockers, and 0 failed rows.",
            "Whole-engine Rust totality, broad refinement, and panic-free core-domain proof remain unpromoted.",
        ],
        reason_ids: &[
            "pack.grade.w042_rust_totality_not_full",
            "pack.grade.w042_refinement_not_full",
            "pack.grade.w042_runtime_panic_surface_proof_absent",
        ],
    },
    SupplementalEvidenceSpec {
        input_id: "w042_lean_tla_fairness_full_verification_expansion",
        artifact_paths: W042_LEAN_TLA_ARTIFACTS,
        satisfied_input_id: "w042_lean_tla_fairness_full_verification_valid",
        evidence_state_present: "w042_lean_tla_packet_present_with_exact_blockers",
        observations: &[
            "W042 Lean/TLA records 14 proof/model rows, 8 local checked-proof rows, 4 bounded-model rows, 5 totality boundaries, 5 exact blockers, and 0 failed rows.",
            "Full Lean verification, full TLA verification, scheduler fairness, unbounded model coverage, and general OxFunc kernels remain unpromoted.",
        ],
        reason_ids: &[
            "pack.grade.w042_full_lean_tla_verification_not_promoted",
            "pack.grade.w042_fairness_unbounded_scheduler_coverage_absent",
            "pack.grade.w042_general_oxfunc_kernel_not_promoted",
        ],
    },
    SupplementalEvidenceSpec {
        input_id: "w042_stage2_production_analyzer_pack_grade_equivalence",
        artifact_paths: W042_STAGE2_REPLAY_ARTIFACTS,
        satisfied_input_id: "w042_stage2_pack_grade_equivalence_valid",
        evidence_state_present: "w042_stage2_packet_present_no_policy_or_pack_promotion",
        observations: &[
            "W042 Stage 2 records 18 policy rows, 12 satisfied declared-profile rows, 5 partition replay rows, 6 permutation rows, 5 observable-invariance rows, 6 exact blockers, and 0 failed rows.",
            "Declared-profile pack equivalence is bound, but production Stage 2 policy and pack-grade replay remain blocked by production partition soundness, scheduler/fairness coverage, operated service evidence, retained-witness lifecycle, and pack governance.",
        ],
        reason_ids: &[
            "pack.grade.w042_stage2_policy_unpromoted",
            "pack.grade.w042_stage2_replay_equivalence_not_pack_grade",
            "pack.grade.w042_production_partition_soundness_absent",
            "pack.grade.w042_pack_grade_replay_governance_absent",
        ],
    },
    SupplementalEvidenceSpec {
        input_id: "w042_operated_assurance_retained_history_witness_alert",
        artifact_paths: W042_OPERATED_ASSURANCE_ARTIFACTS,
        satisfied_input_id: "w042_operated_assurance_retained_history_witness_alert_valid",
        evidence_state_present: "w042_service_envelope_present_no_service_promotion",
        observations: &[
            "W042 operated assurance records 9 service-envelope rows, 29 retained-history rows, 10 query rows, 6 retained-witness lifecycle rows, 23 alert/quarantine rows, 21 readiness criteria, 6 exact service blockers, and 0 failed rows.",
            "Operated continuous assurance, retained-history service, retained-witness lifecycle service, retention SLO enforcement, external alert dispatcher, and operated cross-engine differential service remain unpromoted.",
        ],
        reason_ids: &[
            "pack.grade.w042_operated_continuous_assurance_service_absent",
            "pack.grade.w042_retained_history_service_absent",
            "pack.grade.w042_retained_witness_lifecycle_service_absent",
            "pack.grade.w042_retention_slo_not_enforced",
            "pack.grade.w042_external_alert_dispatcher_absent",
            "pack.grade.w042_operated_cross_engine_diff_service_absent",
        ],
    },
    SupplementalEvidenceSpec {
        input_id: "w042_independent_evaluator_mismatch_quarantine_differential",
        artifact_paths: W042_DIVERSITY_SEAM_ARTIFACTS,
        satisfied_input_id: "w042_independent_evaluator_mismatch_quarantine_valid",
        evidence_state_present: "w042_diversity_packet_present_no_service_or_full_independence_promotion",
        observations: &[
            "W042 diversity records a 4-case independent named-reference model with 4 matches, 11 independent-evaluator rows, 10 cross-engine rows, 10 mismatch-authority rows, 17 accepted boundaries, 7 exact blockers, and 0 failed rows.",
            "Full independent evaluator breadth, operated cross-engine differential service, mismatch quarantine service, and release-grade authority remain unpromoted.",
        ],
        reason_ids: &[
            "pack.grade.w042_fully_independent_evaluator_absent",
            "pack.grade.w042_independent_evaluator_breadth_absent",
            "pack.grade.w042_operated_cross_engine_diff_service_absent",
            "pack.grade.w042_mismatch_quarantine_service_absent",
        ],
    },
    SupplementalEvidenceSpec {
        input_id: "w042_oxfml_public_migration_callable_registered_external",
        artifact_paths: W042_OXFML_SEAM_ARTIFACTS,
        satisfied_input_id: "w042_oxfml_public_migration_callable_registered_external_valid",
        evidence_state_present: "w042_oxfml_seam_packet_present_with_callable_provider_blockers",
        observations: &[
            "W042 OxFml seam records 17 source rows, 12 consumed-surface rows, 10 publication/display rows, 10 callable rows, 8 registered-external/provider rows, 10 exact blockers, and 0 failed rows.",
            "The current W073 typed-only formatting guard, public consumer notes, distinct format/display boundary, LET/LAMBDA carrier rows, registered-external/provider watch rows, and callable blockers are bound without broad OxFml or callable promotion.",
        ],
        reason_ids: &[
            "pack.grade.w042_broad_oxfml_display_publication_unpromoted",
            "pack.grade.w042_public_consumer_migration_not_verified",
            "pack.grade.w042_callable_metadata_projection_absent",
            "pack.grade.w042_callable_carrier_sufficiency_proof_absent",
            "pack.grade.w042_registered_external_projection_absent",
            "pack.grade.w042_provider_failure_callable_publication_watch_lane",
            "pack.grade.w042_general_oxfunc_kernel_not_promoted",
        ],
    },
    SupplementalEvidenceSpec {
        input_id: "w042_fresh_direct_oxfml_upstream_host",
        artifact_paths: W042_UPSTREAM_HOST_ARTIFACTS,
        satisfied_input_id: "w042_fresh_direct_oxfml_upstream_host_valid",
        evidence_state_present: "current_w042_direct_oxfml_run_present_no_pack_promotion",
        observations: &[
            "W042 upstream-host records 12 cases, 3 direct OxFml cases, 2 LET/LAMBDA cases, 1 W073 typed-rule formatting guard, and 0 expectation mismatches.",
            "The direct OxFml slice preserves current W073 and LET/LAMBDA evidence but does not promote pack-grade replay, C5, broad OxFml, callable metadata, registered-external projection, provider publication, or general OxFunc kernels.",
        ],
        reason_ids: &[],
    },
];
const W043_TREECALC_CONFORMANCE_RUN_ID: &str = "w043-optimized-core-broad-conformance-treecalc-001";
const W043_FORMAL_ARTIFACTS: &[&str] = &[
    "docs/spec/core-engine/w043-formalization/W043_RUST_TOTALITY_REFINEMENT_AND_PANIC_FREE_CORE_PROOF_FRONTIER.md",
    "docs/spec/core-engine/w043-formalization/W043_LEAN_TLA_FULL_VERIFICATION_AND_UNBOUNDED_FAIRNESS_DISCHARGE.md",
    "docs/spec/core-engine/w043-formalization/W043_STAGE2_PRODUCTION_PARTITION_ANALYZER_AND_SCHEDULER_EQUIVALENCE.md",
    "formal/lean/OxCalc/CoreEngine/W043RustTotalityAndRefinement.lean",
    "formal/lean/OxCalc/CoreEngine/W043LeanTlaFullVerificationAndFairness.lean",
    "formal/lean/OxCalc/CoreEngine/W043Stage2ProductionPartitionAnalyzerAndSchedulerEquivalence.lean",
];
const W043_FORMATTING_WATCH_ARTIFACTS: &[&str] = &[
    "docs/spec/core-engine/w043-formalization/W043_OXFML_PUBLIC_MIGRATION_FORMATTING_CALLABLE_AND_REGISTERED_EXTERNAL_SEAM.md",
    "docs/test-runs/core-engine/release-grade-ledger/w043-residual-release-grade-proof-service-obligation-map-001/oxfml_inbound_observation_intake.json",
];
const W043_RELEASE_LEDGER_ARTIFACTS: &[&str] = &[
    "docs/test-runs/core-engine/release-grade-ledger/w043-residual-release-grade-proof-service-obligation-map-001/run_summary.json",
    "docs/test-runs/core-engine/release-grade-ledger/w043-residual-release-grade-proof-service-obligation-map-001/source_evidence_index.json",
    "docs/test-runs/core-engine/release-grade-ledger/w043-residual-release-grade-proof-service-obligation-map-001/proof_service_obligation_map.json",
    "docs/test-runs/core-engine/release-grade-ledger/w043-residual-release-grade-proof-service-obligation-map-001/promotion_target_gate_map.json",
    "docs/test-runs/core-engine/release-grade-ledger/w043-residual-release-grade-proof-service-obligation-map-001/oxfml_inbound_observation_intake.json",
    "docs/test-runs/core-engine/release-grade-ledger/w043-residual-release-grade-proof-service-obligation-map-001/validation.json",
];
const W043_IMPLEMENTATION_CONFORMANCE_ARTIFACTS: &[&str] = &[
    "docs/test-runs/core-engine/implementation-conformance/w043-optimized-core-broad-conformance-callable-metadata-closure-001/run_summary.json",
    "docs/test-runs/core-engine/implementation-conformance/w043-optimized-core-broad-conformance-callable-metadata-closure-001/source_evidence_index.json",
    "docs/test-runs/core-engine/implementation-conformance/w043-optimized-core-broad-conformance-callable-metadata-closure-001/w043_counterpart_conformance_register.json",
    "docs/test-runs/core-engine/implementation-conformance/w043-optimized-core-broad-conformance-callable-metadata-closure-001/w043_dynamic_transition_evidence.json",
    "docs/test-runs/core-engine/implementation-conformance/w043-optimized-core-broad-conformance-callable-metadata-closure-001/w043_callable_metadata_projection_register.json",
    "docs/test-runs/core-engine/implementation-conformance/w043-optimized-core-broad-conformance-callable-metadata-closure-001/w043_exact_remaining_blocker_register.json",
    "docs/test-runs/core-engine/implementation-conformance/w043-optimized-core-broad-conformance-callable-metadata-closure-001/w043_match_promotion_guard.json",
    "docs/test-runs/core-engine/implementation-conformance/w043-optimized-core-broad-conformance-callable-metadata-closure-001/w073_formatting_intake.json",
    "docs/test-runs/core-engine/implementation-conformance/w043-optimized-core-broad-conformance-callable-metadata-closure-001/validation.json",
];
const W043_RUST_TOTALITY_ARTIFACTS: &[&str] = &[
    "docs/test-runs/core-engine/formal-assurance/w043-rust-totality-refinement-panic-free-frontier-001/run_summary.json",
    "docs/test-runs/core-engine/formal-assurance/w043-rust-totality-refinement-panic-free-frontier-001/source_evidence_index.json",
    "docs/test-runs/core-engine/formal-assurance/w043-rust-totality-refinement-panic-free-frontier-001/w043_rust_totality_refinement_ledger.json",
    "docs/test-runs/core-engine/formal-assurance/w043-rust-totality-refinement-panic-free-frontier-001/w043_rust_totality_boundary_register.json",
    "docs/test-runs/core-engine/formal-assurance/w043-rust-totality-refinement-panic-free-frontier-001/w043_rust_refinement_register.json",
    "docs/test-runs/core-engine/formal-assurance/w043-rust-totality-refinement-panic-free-frontier-001/w043_rust_exact_blocker_register.json",
    "docs/test-runs/core-engine/formal-assurance/w043-rust-totality-refinement-panic-free-frontier-001/validation.json",
];
const W043_LEAN_TLA_ARTIFACTS: &[&str] = &[
    "docs/test-runs/core-engine/formal-assurance/w043-lean-tla-full-verification-unbounded-fairness-001/run_summary.json",
    "docs/test-runs/core-engine/formal-assurance/w043-lean-tla-full-verification-unbounded-fairness-001/source_evidence_index.json",
    "docs/test-runs/core-engine/formal-assurance/w043-lean-tla-full-verification-unbounded-fairness-001/w043_lean_tla_discharge_ledger.json",
    "docs/test-runs/core-engine/formal-assurance/w043-lean-tla-full-verification-unbounded-fairness-001/w043_lean_proof_register.json",
    "docs/test-runs/core-engine/formal-assurance/w043-lean-tla-full-verification-unbounded-fairness-001/w043_tla_model_bound_register.json",
    "docs/test-runs/core-engine/formal-assurance/w043-lean-tla-full-verification-unbounded-fairness-001/w043_lean_tla_exact_blocker_register.json",
    "docs/test-runs/core-engine/formal-assurance/w043-lean-tla-full-verification-unbounded-fairness-001/validation.json",
];
const W043_STAGE2_REPLAY_ARTIFACTS: &[&str] = &[
    "docs/test-runs/core-engine/stage2-replay/w043-stage2-production-partition-analyzer-scheduler-equivalence-001/run_summary.json",
    "docs/test-runs/core-engine/stage2-replay/w043-stage2-production-partition-analyzer-scheduler-equivalence-001/source_evidence_index.json",
    "docs/test-runs/core-engine/stage2-replay/w043-stage2-production-partition-analyzer-scheduler-equivalence-001/w043_stage2_policy_gate_register.json",
    "docs/test-runs/core-engine/stage2-replay/w043-stage2-production-partition-analyzer-scheduler-equivalence-001/w043_production_partition_analyzer_register.json",
    "docs/test-runs/core-engine/stage2-replay/w043-stage2-production-partition-analyzer-scheduler-equivalence-001/w043_scheduler_equivalence_register.json",
    "docs/test-runs/core-engine/stage2-replay/w043-stage2-production-partition-analyzer-scheduler-equivalence-001/w043_pack_grade_equivalence_register.json",
    "docs/test-runs/core-engine/stage2-replay/w043-stage2-production-partition-analyzer-scheduler-equivalence-001/w043_stage2_exact_blocker_register.json",
    "docs/test-runs/core-engine/stage2-replay/w043-stage2-production-partition-analyzer-scheduler-equivalence-001/promotion_decision.json",
    "docs/test-runs/core-engine/stage2-replay/w043-stage2-production-partition-analyzer-scheduler-equivalence-001/validation.json",
];
const W043_OPERATED_ASSURANCE_ARTIFACTS: &[&str] = &[
    "docs/test-runs/core-engine/operated-assurance/w043-operated-assurance-retained-history-witness-slo-alert-service-001/run_summary.json",
    "docs/test-runs/core-engine/operated-assurance/w043-operated-assurance-retained-history-witness-slo-alert-service-001/source_evidence_index.json",
    "docs/test-runs/core-engine/operated-assurance/w043-operated-assurance-retained-history-witness-slo-alert-service-001/w043_operated_service_envelope.json",
    "docs/test-runs/core-engine/operated-assurance/w043-operated-assurance-retained-history-witness-slo-alert-service-001/w043_retained_history_service_query.json",
    "docs/test-runs/core-engine/operated-assurance/w043-operated-assurance-retained-history-witness-slo-alert-service-001/w043_cross_engine_service_register.json",
    "docs/test-runs/core-engine/operated-assurance/w043-operated-assurance-retained-history-witness-slo-alert-service-001/w043_retained_witness_lifecycle_register.json",
    "docs/test-runs/core-engine/operated-assurance/w043-operated-assurance-retained-history-witness-slo-alert-service-001/w043_alert_dispatch_service_register.json",
    "docs/test-runs/core-engine/operated-assurance/w043-operated-assurance-retained-history-witness-slo-alert-service-001/w043_service_readiness_register.json",
    "docs/test-runs/core-engine/operated-assurance/w043-operated-assurance-retained-history-witness-slo-alert-service-001/w043_exact_service_blocker_register.json",
    "docs/test-runs/core-engine/operated-assurance/w043-operated-assurance-retained-history-witness-slo-alert-service-001/promotion_decision.json",
    "docs/test-runs/core-engine/operated-assurance/w043-operated-assurance-retained-history-witness-slo-alert-service-001/validation.json",
];
const W043_DIVERSITY_SEAM_ARTIFACTS: &[&str] = &[
    "docs/test-runs/core-engine/diversity-seam/w043-independent-evaluator-breadth-mismatch-quarantine-differential-service-001/run_summary.json",
    "docs/test-runs/core-engine/diversity-seam/w043-independent-evaluator-breadth-mismatch-quarantine-differential-service-001/source_evidence_index.json",
    "docs/test-runs/core-engine/diversity-seam/w043-independent-evaluator-breadth-mismatch-quarantine-differential-service-001/w043_independent_reference_model_implementation.json",
    "docs/test-runs/core-engine/diversity-seam/w043-independent-evaluator-breadth-mismatch-quarantine-differential-service-001/w043_independent_evaluator_breadth_register.json",
    "docs/test-runs/core-engine/diversity-seam/w043-independent-evaluator-breadth-mismatch-quarantine-differential-service-001/w043_cross_engine_differential_service_register.json",
    "docs/test-runs/core-engine/diversity-seam/w043-independent-evaluator-breadth-mismatch-quarantine-differential-service-001/w043_mismatch_quarantine_authority_router.json",
    "docs/test-runs/core-engine/diversity-seam/w043-independent-evaluator-breadth-mismatch-quarantine-differential-service-001/w043_exact_diversity_blocker_register.json",
    "docs/test-runs/core-engine/diversity-seam/w043-independent-evaluator-breadth-mismatch-quarantine-differential-service-001/promotion_decision.json",
    "docs/test-runs/core-engine/diversity-seam/w043-independent-evaluator-breadth-mismatch-quarantine-differential-service-001/validation.json",
];
const W043_OXFML_SEAM_ARTIFACTS: &[&str] = &[
    "docs/test-runs/core-engine/oxfml-seam/w043-oxfml-public-migration-formatting-callable-registered-external-001/run_summary.json",
    "docs/test-runs/core-engine/oxfml-seam/w043-oxfml-public-migration-formatting-callable-registered-external-001/source_evidence_index.json",
    "docs/test-runs/core-engine/oxfml-seam/w043-oxfml-public-migration-formatting-callable-registered-external-001/w043_oxfml_consumed_surface_register.json",
    "docs/test-runs/core-engine/oxfml-seam/w043-oxfml-public-migration-formatting-callable-registered-external-001/w043_publication_display_boundary_register.json",
    "docs/test-runs/core-engine/oxfml-seam/w043-oxfml-public-migration-formatting-callable-registered-external-001/w043_callable_carrier_and_metadata_register.json",
    "docs/test-runs/core-engine/oxfml-seam/w043-oxfml-public-migration-formatting-callable-registered-external-001/w043_registered_external_provider_register.json",
    "docs/test-runs/core-engine/oxfml-seam/w043-oxfml-public-migration-formatting-callable-registered-external-001/w043_exact_oxfml_seam_blocker_register.json",
    "docs/test-runs/core-engine/oxfml-seam/w043-oxfml-public-migration-formatting-callable-registered-external-001/promotion_decision.json",
    "docs/test-runs/core-engine/oxfml-seam/w043-oxfml-public-migration-formatting-callable-registered-external-001/validation.json",
];
const W043_UPSTREAM_HOST_ARTIFACTS: &[&str] = &[
    "docs/test-runs/core-engine/upstream-host/w043-oxfml-public-migration-formatting-callable-registered-external-001/run_summary.json",
    "docs/test-runs/core-engine/upstream-host/w043-oxfml-public-migration-formatting-callable-registered-external-001/case_index.json",
];
const W043_SUPPLEMENTAL_EVIDENCE: &[SupplementalEvidenceSpec] = &[
    SupplementalEvidenceSpec {
        input_id: "w043_proof_service_obligation_map",
        artifact_paths: W043_RELEASE_LEDGER_ARTIFACTS,
        satisfied_input_id: "w043_proof_service_obligation_map_valid",
        evidence_state_present: "w043_proof_service_map_present_with_pack_c5_blocked",
        observations: &[
            "W043 proof-service map records 36 obligations, 16 promotion-target gates, and current W073 typed-only formatting intake.",
            "The map keeps release-grade verification, pack-grade replay, C5, Stage 2, operated services, retained history/witness services, independent evaluator, broad OxFml, callable, registered-external, provider publication, and general OxFunc claims unpromoted unless direct gates are satisfied.",
        ],
        reason_ids: &[
            "pack.grade.w043_release_grade_verification_not_promoted",
            "pack.grade.w043_pack_c5_blocked_by_proof_service_obligation_map",
        ],
    },
    SupplementalEvidenceSpec {
        input_id: "w043_optimized_core_broad_conformance",
        artifact_paths: W043_IMPLEMENTATION_CONFORMANCE_ARTIFACTS,
        satisfied_input_id: "w043_optimized_core_broad_conformance_valid",
        evidence_state_present: "w043_optimized_core_packet_present_with_exact_blockers",
        observations: &[
            "W043 optimized/core conformance binds a 27-case TreeCalc replay, dynamic addition/reclassification evidence, carried declared-profile counterparts, 3 exact blockers, 0 match promotions, and 0 failed rows.",
            "Broader dynamic transition coverage, full optimized/core verification, and callable metadata projection remain blocked.",
        ],
        reason_ids: &[
            "pack.grade.w043_optimized_core_engine_conformance_not_full",
            "pack.grade.w043_broader_dynamic_transition_coverage_absent",
            "pack.grade.w043_callable_metadata_projection_absent",
        ],
    },
    SupplementalEvidenceSpec {
        input_id: "w043_rust_totality_refinement_panic_free_frontier",
        artifact_paths: W043_RUST_TOTALITY_ARTIFACTS,
        satisfied_input_id: "w043_rust_totality_refinement_valid",
        evidence_state_present: "w043_rust_packet_present_with_totality_boundaries",
        observations: &[
            "W043 Rust totality/refinement records 14 rows, 11 local checked-proof rows, 4 totality boundaries, 4 exact blockers, and 0 failed rows.",
            "Whole-engine Rust totality, broad refinement, and panic-free core-domain proof remain unpromoted.",
        ],
        reason_ids: &[
            "pack.grade.w043_rust_totality_not_full",
            "pack.grade.w043_refinement_not_full",
            "pack.grade.w043_runtime_panic_surface_proof_absent",
        ],
    },
    SupplementalEvidenceSpec {
        input_id: "w043_lean_tla_full_verification_unbounded_fairness",
        artifact_paths: W043_LEAN_TLA_ARTIFACTS,
        satisfied_input_id: "w043_lean_tla_full_verification_unbounded_fairness_valid",
        evidence_state_present: "w043_lean_tla_packet_present_with_exact_blockers",
        observations: &[
            "W043 Lean/TLA records 15 proof/model rows, 9 local checked-proof rows, 4 bounded-model rows, 5 exact blockers, and 0 failed rows.",
            "Full Lean verification, full TLA verification, scheduler fairness, unbounded model coverage, and general OxFunc kernels remain unpromoted.",
        ],
        reason_ids: &[
            "pack.grade.w043_full_lean_tla_verification_not_promoted",
            "pack.grade.w043_fairness_unbounded_scheduler_coverage_absent",
            "pack.grade.w043_general_oxfunc_kernel_not_promoted",
        ],
    },
    SupplementalEvidenceSpec {
        input_id: "w043_stage2_partition_analyzer_scheduler_equivalence",
        artifact_paths: W043_STAGE2_REPLAY_ARTIFACTS,
        satisfied_input_id: "w043_stage2_scheduler_equivalence_valid",
        evidence_state_present: "w043_stage2_packet_present_no_policy_or_pack_promotion",
        observations: &[
            "W043 Stage 2 records 20 policy rows, 14 satisfied declared-profile rows, 5 partition replay rows, 6 permutation rows, 5 observable-invariance rows, 6 exact blockers, and 0 failed rows.",
            "Declared-profile scheduler equivalence and pack-equivalence inputs are bound, but production Stage 2 policy and pack-grade replay remain blocked by production partition soundness, scheduler/fairness coverage, operated service evidence, retained-witness lifecycle, and pack governance.",
        ],
        reason_ids: &[
            "pack.grade.w043_stage2_policy_unpromoted",
            "pack.grade.w043_stage2_replay_equivalence_not_pack_grade",
            "pack.grade.w043_production_partition_soundness_absent",
            "pack.grade.w043_pack_grade_replay_governance_absent",
        ],
    },
    SupplementalEvidenceSpec {
        input_id: "w043_operated_assurance_retained_history_witness_slo_alert",
        artifact_paths: W043_OPERATED_ASSURANCE_ARTIFACTS,
        satisfied_input_id: "w043_operated_assurance_retained_history_witness_slo_alert_valid",
        evidence_state_present: "w043_service_envelope_present_no_service_promotion",
        observations: &[
            "W043 operated assurance records 10 service-envelope rows, 33 retained-history rows, 13 query rows, 8 retained-witness lifecycle rows, 29 alert/quarantine rows, 22 readiness criteria, 6 exact service blockers, and 0 failed rows.",
            "Operated continuous assurance, retained-history service, retained-witness lifecycle service, retention SLO enforcement, external alert dispatcher, and operated cross-engine differential service remain unpromoted.",
        ],
        reason_ids: &[
            "pack.grade.w043_operated_continuous_assurance_service_absent",
            "pack.grade.w043_retained_history_service_absent",
            "pack.grade.w043_retained_witness_lifecycle_service_absent",
            "pack.grade.w043_retention_slo_not_enforced",
            "pack.grade.w043_external_alert_dispatcher_absent",
            "pack.grade.w043_operated_cross_engine_diff_service_absent",
        ],
    },
    SupplementalEvidenceSpec {
        input_id: "w043_independent_evaluator_mismatch_quarantine_differential",
        artifact_paths: W043_DIVERSITY_SEAM_ARTIFACTS,
        satisfied_input_id: "w043_independent_evaluator_mismatch_quarantine_valid",
        evidence_state_present: "w043_diversity_packet_present_no_service_or_full_independence_promotion",
        observations: &[
            "W043 diversity records a 6-case independent named-reference model with 6 matches, 13 independent-evaluator rows, 12 cross-engine rows, 12 mismatch-authority rows, 21 accepted boundaries, 8 exact blockers, and 0 failed rows.",
            "Full independent evaluator breadth, operated cross-engine differential service, mismatch quarantine service, and release-grade authority remain unpromoted.",
        ],
        reason_ids: &[
            "pack.grade.w043_fully_independent_evaluator_absent",
            "pack.grade.w043_independent_evaluator_breadth_absent",
            "pack.grade.w043_operated_cross_engine_diff_service_absent",
            "pack.grade.w043_mismatch_quarantine_service_absent",
        ],
    },
    SupplementalEvidenceSpec {
        input_id: "w043_oxfml_public_migration_formatting_callable_registered_external",
        artifact_paths: W043_OXFML_SEAM_ARTIFACTS,
        satisfied_input_id: "w043_oxfml_public_migration_formatting_callable_registered_external_valid",
        evidence_state_present: "w043_oxfml_seam_packet_present_with_callable_provider_blockers",
        observations: &[
            "W043 OxFml seam records 19 source rows, 13 consumed-surface rows, 12 publication/display rows, 11 callable rows, 9 registered-external/provider rows, 11 exact blockers, and 0 failed rows.",
            "The current W073 typed-only formatting guard, downstream typed-rule request-construction watch, public consumer notes, distinct format/display boundary, LET/LAMBDA carrier rows, registered-external/provider watch rows, and callable blockers are bound without broad OxFml or callable promotion.",
        ],
        reason_ids: &[
            "pack.grade.w043_broad_oxfml_display_publication_unpromoted",
            "pack.grade.w043_public_consumer_migration_not_verified",
            "pack.grade.w043_w073_public_request_construction_uptake_not_verified",
            "pack.grade.w043_callable_metadata_projection_absent",
            "pack.grade.w043_callable_carrier_sufficiency_proof_absent",
            "pack.grade.w043_registered_external_projection_absent",
            "pack.grade.w043_provider_failure_callable_publication_watch_lane",
            "pack.grade.w043_general_oxfunc_kernel_not_promoted",
        ],
    },
    SupplementalEvidenceSpec {
        input_id: "w043_fresh_direct_oxfml_upstream_host",
        artifact_paths: W043_UPSTREAM_HOST_ARTIFACTS,
        satisfied_input_id: "w043_fresh_direct_oxfml_upstream_host_valid",
        evidence_state_present: "current_w043_direct_oxfml_run_present_no_pack_promotion",
        observations: &[
            "W043 upstream-host records 12 cases, 3 direct OxFml cases, 2 LET/LAMBDA cases, 1 W073 typed-rule formatting guard, and 0 expectation mismatches.",
            "The direct OxFml slice preserves current W073 and LET/LAMBDA evidence but does not promote pack-grade replay, C5, broad OxFml, callable metadata, registered-external projection, provider publication, downstream typed-rule request construction, or general OxFunc kernels.",
        ],
        reason_ids: &[],
    },
];

#[derive(Debug, Error)]
pub enum PackCapabilityError {
    #[error("failed to create artifact directory {path}: {source}")]
    CreateDirectory {
        path: String,
        source: std::io::Error,
    },
    #[error("failed to remove existing artifact root {path}: {source}")]
    RemoveDirectory {
        path: String,
        source: std::io::Error,
    },
    #[error("failed to read artifact {path}: {source}")]
    ReadArtifact {
        path: String,
        source: std::io::Error,
    },
    #[error("failed to parse JSON artifact {path}: {source}")]
    ParseJson {
        path: String,
        source: serde_json::Error,
    },
    #[error("failed to write artifact {path}: {source}")]
    WriteFile {
        path: String,
        source: std::io::Error,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackCapabilityRunSummary {
    pub run_id: String,
    pub schema_version: String,
    pub decision_status: String,
    pub highest_honest_capability: String,
    pub satisfied_input_count: usize,
    pub blocker_count: usize,
    pub missing_artifact_count: usize,
    pub artifact_root: String,
}

#[derive(Debug, Clone, Default)]
pub struct PackCapabilityRunner;

#[derive(Debug, Clone)]
struct EvidenceRow {
    input_id: &'static str,
    artifact_path: String,
    evidence_state: String,
    observations: Vec<String>,
    reason_ids: Vec<String>,
}

#[derive(Debug, Clone, Default)]
struct EvidenceEvaluation {
    rows: Vec<EvidenceRow>,
    missing_paths: Vec<String>,
    blockers: Vec<String>,
    satisfied_inputs: Vec<String>,
}

#[derive(Debug, Clone)]
struct PackCapabilityProfile {
    profile_id: &'static str,
    oxfml_bridge_run_id: &'static str,
    let_lambda_tracecalc_run_id: &'static str,
    let_lambda_treecalc_run_id: &'static str,
    independent_treecalc_run_id: &'static str,
    independent_conformance_run_id: &'static str,
    program_governance_artifact: &'static str,
    formal_artifacts: &'static [&'static str],
    formal_input_id: &'static str,
    formal_satisfied_input_id: &'static str,
    formal_evidence_state_present: &'static str,
    formal_observations: &'static [&'static str],
    formal_reason_ids: &'static [&'static str],
    formatting_watch_artifacts: &'static [&'static str],
    additional_static_blockers: &'static [&'static str],
    supplemental_evidence: &'static [SupplementalEvidenceSpec],
    successor_lanes: &'static [&'static str],
}

#[derive(Debug, Clone)]
struct SupplementalEvidenceSpec {
    input_id: &'static str,
    artifact_paths: &'static [&'static str],
    satisfied_input_id: &'static str,
    evidence_state_present: &'static str,
    observations: &'static [&'static str],
    reason_ids: &'static [&'static str],
}

impl PackCapabilityRunner {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    pub fn execute(
        &self,
        repo_root: &Path,
        run_id: &str,
    ) -> Result<PackCapabilityRunSummary, PackCapabilityError> {
        let artifact_root = repo_root.join(format!(
            "docs/test-runs/core-engine/pack-capability/{run_id}"
        ));
        let relative_artifact_root = relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "pack-capability",
            run_id,
        ]);

        if artifact_root.exists() {
            fs::remove_dir_all(&artifact_root).map_err(|source| {
                PackCapabilityError::RemoveDirectory {
                    path: artifact_root.display().to_string(),
                    source,
                }
            })?;
        }

        create_directory(&artifact_root)?;
        create_directory(&artifact_root.join("decision"))?;
        create_directory(&artifact_root.join("evidence"))?;
        create_directory(&artifact_root.join("replay-appliance"))?;
        create_directory(&artifact_root.join("replay-appliance/validation"))?;

        let profile = pack_capability_profile(run_id);
        let evaluation = evaluate_evidence(repo_root, &profile)?;
        let decision_status =
            if evaluation.missing_paths.is_empty() && !evaluation.blockers.is_empty() {
                "capability_not_promoted"
            } else if evaluation.missing_paths.is_empty() {
                "capability_promotion_candidate"
            } else {
                "evidence_incomplete"
            };
        let highest_honest_capability = "cap.C4.distill_valid";

        write_json(
            &artifact_root.join("evidence/evidence_index.json"),
            &json!({
                "schema_version": PACK_CAPABILITY_EVIDENCE_INDEX_SCHEMA_V1,
                "run_id": run_id,
                "evidence_profile": profile.profile_id,
                "target_capability": "cap.C5.pack_valid",
                "highest_honest_capability": highest_honest_capability,
                "missing_artifact_count": evaluation.missing_paths.len(),
                "blocker_count": evaluation.blockers.len(),
                "satisfied_input_count": evaluation.satisfied_inputs.len(),
                "rows": evaluation.rows.iter().map(evidence_row_json).collect::<Vec<_>>(),
            }),
        )?;

        write_json(
            &artifact_root.join("decision/pack_capability_decision.json"),
            &json!({
                "schema_version": PACK_CAPABILITY_DECISION_SCHEMA_V1,
                "run_id": run_id,
                "evidence_profile": profile.profile_id,
                "target_capability": "cap.C5.pack_valid",
                "decision_status": decision_status,
                "highest_honest_capability": highest_honest_capability,
                "capability_promoted": decision_status == "capability_promotion_candidate",
                "missing_artifact_count": evaluation.missing_paths.len(),
                "missing_paths": evaluation.missing_paths,
                "satisfied_inputs": evaluation.satisfied_inputs,
                "no_promotion_reason_ids": evaluation.blockers,
                "evidence_index_path": format!("{relative_artifact_root}/evidence/evidence_index.json"),
                "successor_lanes": profile.successor_lanes,
                "stage2_readiness": {
                    "stage2_scheduler_promoted": false,
                    "decision_state": "not_ready_for_stage2_promotion",
                    "required_before_promotion": [
                        "concrete_partition_coverage_model",
                        "scheduler_semantic_equivalence_replay",
                        "continuous_cross_engine_differential_service",
                        "pack_grade_replay_governance"
                    ]
                },
                "semantic_equivalence_statement": "This runner reads existing evidence and emits pack/Stage 2 readiness decisions only. It does not change coordinator scheduling, invalidation, dependency graph construction, soft-reference resolution, recalc semantics, publication semantics, reject policy, TraceCalc, TreeCalc, Lean/TLA, or OxFml evaluator behavior.",
                "handoff_decision": {
                    "status": "handoff_not_required",
                    "reason_ids": [
                        "oxfml.seam.no_new_trigger_from_pack_governance_packet",
                        "oxfml.seam.watch_lanes_only"
                    ]
                }
            }),
        )?;

        let required_artifacts = required_artifacts(run_id, &profile);
        write_json(
            &artifact_root.join("replay-appliance/bundle_manifest.json"),
            &json!({
                "schema_version": PACK_CAPABILITY_BUNDLE_SCHEMA_V1,
                "run_id": run_id,
                "evidence_profile": profile.profile_id,
                "artifact_root": relative_artifact_root,
                "target_capability": "cap.C5.pack_valid",
                "highest_honest_capability": highest_honest_capability,
                "claimed_capability": "pack_governance_decision_packet",
                "excluded_capabilities": [
                    "cap.C5.pack_valid",
                    "continuous_cross_engine_diff_suite",
                    "fully_independent_evaluator_implementation"
                ],
                "required_artifacts": required_artifacts,
            }),
        )?;

        let summary = PackCapabilityRunSummary {
            run_id: run_id.to_string(),
            schema_version: PACK_CAPABILITY_RUN_SUMMARY_SCHEMA_V1.to_string(),
            decision_status: decision_status.to_string(),
            highest_honest_capability: highest_honest_capability.to_string(),
            satisfied_input_count: evaluation.satisfied_inputs.len(),
            blocker_count: evaluation.blockers.len(),
            missing_artifact_count: evaluation.missing_paths.len(),
            artifact_root: relative_artifact_root.clone(),
        };
        write_json(
            &artifact_root.join("run_summary.json"),
            &json!({
                "schema_version": summary.schema_version,
                "run_id": summary.run_id,
                "evidence_profile": profile.profile_id,
                "target_capability": "cap.C5.pack_valid",
                "decision_status": summary.decision_status,
                "highest_honest_capability": summary.highest_honest_capability,
                "satisfied_input_count": summary.satisfied_input_count,
                "blocker_count": summary.blocker_count,
                "missing_artifact_count": summary.missing_artifact_count,
                "artifact_root": summary.artifact_root,
                "decision_path": format!("{relative_artifact_root}/decision/pack_capability_decision.json"),
                "evidence_index_path": format!("{relative_artifact_root}/evidence/evidence_index.json"),
                "bundle_validation_path": format!("{relative_artifact_root}/replay-appliance/validation/bundle_validation.json"),
            }),
        )?;

        let validation_path =
            artifact_root.join("replay-appliance/validation/bundle_validation.json");
        write_json(
            &validation_path,
            &json!({
                "schema_version": PACK_CAPABILITY_VALIDATION_SCHEMA_V1,
                "run_id": run_id,
                "status": "pending_final_validation_write",
            }),
        )?;
        let missing_required_paths = required_artifacts
            .iter()
            .filter(|relative_path| !repo_root.join(relative_path.as_str()).exists())
            .cloned()
            .collect::<Vec<_>>();
        write_json(
            &validation_path,
            &json!({
                "schema_version": PACK_CAPABILITY_VALIDATION_SCHEMA_V1,
                "run_id": run_id,
                "status": if missing_required_paths.is_empty() { "bundle_valid" } else { "missing_required_artifacts" },
                "missing_paths": missing_required_paths,
                "validated_required_artifact_count": required_artifacts.len(),
                "decision_status": decision_status,
                "highest_honest_capability": highest_honest_capability,
            }),
        )?;

        Ok(summary)
    }
}

fn evaluate_evidence(
    repo_root: &Path,
    profile: &PackCapabilityProfile,
) -> Result<EvidenceEvaluation, PackCapabilityError> {
    let mut evaluation = EvidenceEvaluation::default();
    evaluate_retained_decision(repo_root, &mut evaluation)?;
    evaluate_bridge(repo_root, profile, &mut evaluation)?;
    evaluate_let_lambda(repo_root, profile, &mut evaluation)?;
    evaluate_independent_conformance(repo_root, profile, &mut evaluation)?;
    evaluate_treecalc_capability(repo_root, profile, &mut evaluation)?;
    evaluate_formal_artifacts(repo_root, profile, &mut evaluation);
    evaluate_formatting_watch_artifacts(repo_root, profile, &mut evaluation);
    evaluate_supplemental_evidence(repo_root, profile, &mut evaluation);
    add_static_program_blockers(profile, &mut evaluation);
    dedup_strings(&mut evaluation.blockers);
    dedup_strings(&mut evaluation.satisfied_inputs);
    Ok(evaluation)
}

fn evaluate_retained_decision(
    repo_root: &Path,
    evaluation: &mut EvidenceEvaluation,
) -> Result<(), PackCapabilityError> {
    let pack_path = retained_validation_path("pack_grade_decision.json");
    let program_path = retained_validation_path("program_grade_decision.json");
    let pack = read_json(repo_root, &pack_path)?;
    let program = read_json(repo_root, &program_path)?;
    let mut observations = Vec::new();
    let mut reason_ids = Vec::new();

    if let Some(pack) = &pack {
        observations.push(format!(
            "pack_decision_status:{}",
            text_at(pack, "decision_status")
        ));
        observations.push(format!(
            "pack_highest_honest_capability:{}",
            text_at(pack, "highest_honest_capability")
        ));
        if text_at(pack, "decision_status") == "capability_not_promoted" {
            evaluation
                .satisfied_inputs
                .push("retained_semantic_pack_decision_present".to_string());
        }
    }
    if let Some(program) = &program {
        observations.push(format!(
            "program_decision_status:{}",
            text_at(program, "decision_status")
        ));
        if text_at(program, "decision_status") == "capability_not_promoted" {
            reason_ids.push("pack.grade.program_scope.unproven".to_string());
        }
    }

    add_missing_if_absent(evaluation, &pack_path, &pack);
    add_missing_if_absent(evaluation, &program_path, &program);
    evaluation.blockers.extend(reason_ids.clone());
    evaluation.rows.push(EvidenceRow {
        input_id: "retained_pack_program_decisions",
        artifact_path: format!("{pack_path};{program_path}"),
        evidence_state: if pack.is_some() && program.is_some() {
            "evidence_present".to_string()
        } else {
            "missing_artifact".to_string()
        },
        observations,
        reason_ids,
    });
    Ok(())
}

fn evaluate_bridge(
    repo_root: &Path,
    profile: &PackCapabilityProfile,
    evaluation: &mut EvidenceEvaluation,
) -> Result<(), PackCapabilityError> {
    let summary_path = relative_artifact_path([
        "docs",
        "test-runs",
        "core-engine",
        "oxfml-fixture-bridge",
        profile.oxfml_bridge_run_id,
        "run_summary.json",
    ]);
    let validation_path = relative_artifact_path([
        "docs",
        "test-runs",
        "core-engine",
        "oxfml-fixture-bridge",
        profile.oxfml_bridge_run_id,
        "replay-appliance",
        "validation",
        "bundle_validation.json",
    ]);
    let summary = read_json(repo_root, &summary_path)?;
    let validation = read_json(repo_root, &validation_path)?;
    add_missing_if_absent(evaluation, &summary_path, &summary);
    add_missing_if_absent(evaluation, &validation_path, &validation);

    let mut observations = Vec::new();
    let mut reason_ids = Vec::new();
    if let Some(summary) = &summary {
        observations.push(format!(
            "fixture_case_count:{}",
            number_at(summary, "fixture_case_count")
        ));
        observations.push(format!(
            "mismatch_count:{}",
            number_at(summary, "mismatch_count")
        ));
        observations.push(format!(
            "handoff_triggered:{}",
            bool_at(summary, "handoff_triggered")
        ));
        if number_at(summary, "mismatch_count") == 0 && !bool_at(summary, "handoff_triggered") {
            evaluation
                .satisfied_inputs
                .push("direct_oxfml_fixture_projection_has_no_mismatch".to_string());
        }
        if !profile.profile_id.starts_with("w037_")
            && !profile.profile_id.starts_with("w038_")
            && !profile.profile_id.starts_with("w039_")
            && !profile.profile_id.starts_with("w040_")
            && !profile.profile_id.starts_with("w041_")
            && !profile.profile_id.starts_with("w042_")
            && !profile.profile_id.starts_with("w043_")
        {
            reason_ids.push("pack.grade.direct_oxfml_evaluator_reexecution_absent".to_string());
        }
    }
    if let Some(validation) = &validation {
        observations.push(format!("bundle_status:{}", text_at(validation, "status")));
    }
    evaluation.blockers.extend(reason_ids.clone());
    evaluation.rows.push(EvidenceRow {
        input_id: "direct_oxfml_fixture_bridge",
        artifact_path: format!("{summary_path};{validation_path}"),
        evidence_state: if summary.is_some() && validation.is_some() {
            "projection_valid_no_pack_promotion".to_string()
        } else {
            "missing_artifact".to_string()
        },
        observations,
        reason_ids,
    });
    Ok(())
}

fn evaluate_let_lambda(
    repo_root: &Path,
    profile: &PackCapabilityProfile,
    evaluation: &mut EvidenceEvaluation,
) -> Result<(), PackCapabilityError> {
    let trace_validation_path = bundle_validation_path(
        "tracecalc-reference-machine",
        profile.let_lambda_tracecalc_run_id,
    );
    let tree_validation_path =
        bundle_validation_path("treecalc-local", profile.let_lambda_treecalc_run_id);
    let trace_validation = read_json(repo_root, &trace_validation_path)?;
    let tree_validation = read_json(repo_root, &tree_validation_path)?;
    add_missing_if_absent(evaluation, &trace_validation_path, &trace_validation);
    add_missing_if_absent(evaluation, &tree_validation_path, &tree_validation);

    let mut observations = Vec::new();
    if let Some(trace_validation) = &trace_validation {
        observations.push(format!(
            "tracecalc_bundle_status:{}",
            text_at(trace_validation, "status")
        ));
    }
    if let Some(tree_validation) = &tree_validation {
        observations.push(format!(
            "treecalc_bundle_status:{}",
            text_at(tree_validation, "status")
        ));
    }
    if trace_validation
        .as_ref()
        .is_some_and(|value| text_at(value, "status") == "bundle_valid")
        && tree_validation
            .as_ref()
            .is_some_and(|value| text_at(value, "status") == "bundle_valid")
    {
        evaluation
            .satisfied_inputs
            .push("let_lambda_carrier_witness_bundles_valid".to_string());
    }
    evaluation.rows.push(EvidenceRow {
        input_id: "let_lambda_carrier_witnesses",
        artifact_path: format!("{trace_validation_path};{tree_validation_path}"),
        evidence_state: if trace_validation.is_some() && tree_validation.is_some() {
            "witness_bundles_validated".to_string()
        } else {
            "missing_artifact".to_string()
        },
        observations,
        reason_ids: Vec::new(),
    });
    Ok(())
}

fn evaluate_independent_conformance(
    repo_root: &Path,
    profile: &PackCapabilityProfile,
    evaluation: &mut EvidenceEvaluation,
) -> Result<(), PackCapabilityError> {
    let summary_path = relative_artifact_path([
        "docs",
        "test-runs",
        "core-engine",
        "independent-conformance",
        profile.independent_conformance_run_id,
        "run_summary.json",
    ]);
    let validation_path = relative_artifact_path([
        "docs",
        "test-runs",
        "core-engine",
        "independent-conformance",
        profile.independent_conformance_run_id,
        "replay-appliance",
        "validation",
        "bundle_validation.json",
    ]);
    let summary = read_json(repo_root, &summary_path)?;
    let validation = read_json(repo_root, &validation_path)?;
    add_missing_if_absent(evaluation, &summary_path, &summary);
    add_missing_if_absent(evaluation, &validation_path, &validation);

    let mut observations = Vec::new();
    let mut reason_ids = Vec::new();
    if let Some(summary) = &summary {
        observations.push(format!(
            "unexpected_mismatch_count:{}",
            number_at(summary, "unexpected_mismatch_count")
        ));
        observations.push(format!(
            "declared_gap_count:{}",
            number_at(summary, "declared_gap_count")
        ));
        if number_at(summary, "unexpected_mismatch_count") == 0 {
            evaluation
                .satisfied_inputs
                .push("independent_conformance_has_no_unexpected_mismatch".to_string());
        }
        if number_at(summary, "declared_gap_count") > 0 {
            reason_ids.push("pack.grade.independent_conformance_declared_gaps".to_string());
        }
    }
    if let Some(validation) = &validation {
        observations.push(format!("bundle_status:{}", text_at(validation, "status")));
    }
    reason_ids.push("pack.grade.continuous_diff_suite_absent".to_string());
    reason_ids.push("pack.grade.fully_independent_evaluator_absent".to_string());
    evaluation.blockers.extend(reason_ids.clone());
    evaluation.rows.push(EvidenceRow {
        input_id: "independent_conformance_widening",
        artifact_path: format!("{summary_path};{validation_path}"),
        evidence_state: if summary.is_some() && validation.is_some() {
            "widened_conformance_present_with_gaps".to_string()
        } else {
            "missing_artifact".to_string()
        },
        observations,
        reason_ids,
    });
    Ok(())
}

fn evaluate_treecalc_capability(
    repo_root: &Path,
    profile: &PackCapabilityProfile,
    evaluation: &mut EvidenceEvaluation,
) -> Result<(), PackCapabilityError> {
    let capability_path = relative_artifact_path([
        "docs",
        "test-runs",
        "core-engine",
        "treecalc-local",
        profile.independent_treecalc_run_id,
        "replay-appliance",
        "adapter_capabilities",
        "oxcalc_treecalc.json",
    ]);
    let capability = read_json(repo_root, &capability_path)?;
    add_missing_if_absent(evaluation, &capability_path, &capability);

    let mut observations = Vec::new();
    let mut reason_ids = Vec::new();
    if let Some(capability) = &capability {
        observations.push(format!(
            "claimed_capability_levels:{}",
            capability
                .get("claimed_capability_levels")
                .and_then(Value::as_array)
                .map_or(0, std::vec::Vec::len)
        ));
        observations.push(format!(
            "target_capability_levels:{}",
            capability
                .get("target_capability_levels")
                .and_then(Value::as_array)
                .map_or(0, std::vec::Vec::len)
        ));
        evaluation
            .satisfied_inputs
            .push("treecalc_capability_snapshot_present".to_string());
        reason_ids.push("pack.grade.treecalc_c4_c5_unproven".to_string());
    }
    evaluation.blockers.extend(reason_ids.clone());
    evaluation.rows.push(EvidenceRow {
        input_id: "treecalc_capability_snapshot",
        artifact_path: capability_path,
        evidence_state: if capability.is_some() {
            "capability_ceiling_snapshot_present".to_string()
        } else {
            "missing_artifact".to_string()
        },
        observations,
        reason_ids,
    });
    Ok(())
}

fn evaluate_formal_artifacts(
    repo_root: &Path,
    profile: &PackCapabilityProfile,
    evaluation: &mut EvidenceEvaluation,
) {
    if profile.formal_artifacts.is_empty() {
        return;
    }

    let missing_paths = profile
        .formal_artifacts
        .iter()
        .filter(|relative_path| !repo_root.join(relative_path).exists())
        .map(|relative_path| (*relative_path).to_string())
        .collect::<Vec<_>>();
    evaluation.missing_paths.extend(missing_paths.clone());

    let reason_ids = profile
        .formal_reason_ids
        .iter()
        .map(|reason| (*reason).to_string())
        .collect::<Vec<_>>();
    if missing_paths.is_empty() {
        evaluation
            .satisfied_inputs
            .push(profile.formal_satisfied_input_id.to_string());
        evaluation.blockers.extend(reason_ids.clone());
    }

    evaluation.rows.push(EvidenceRow {
        input_id: profile.formal_input_id,
        artifact_path: profile.formal_artifacts.join(";"),
        evidence_state: if missing_paths.is_empty() {
            profile.formal_evidence_state_present.to_string()
        } else {
            "missing_artifact".to_string()
        },
        observations: profile
            .formal_observations
            .iter()
            .map(|observation| (*observation).to_string())
            .collect(),
        reason_ids,
    });
}

fn evaluate_formatting_watch_artifacts(
    repo_root: &Path,
    profile: &PackCapabilityProfile,
    evaluation: &mut EvidenceEvaluation,
) {
    if profile.formatting_watch_artifacts.is_empty() {
        return;
    }

    let missing_paths = profile
        .formatting_watch_artifacts
        .iter()
        .filter(|relative_path| !repo_root.join(relative_path).exists())
        .map(|relative_path| (*relative_path).to_string())
        .collect::<Vec<_>>();
    evaluation.missing_paths.extend(missing_paths.clone());
    if missing_paths.is_empty() {
        evaluation
            .satisfied_inputs
            .push("oxfml_w073_formatting_watch_classified".to_string());
    }

    evaluation.rows.push(EvidenceRow {
        input_id: "oxfml_w073_typed_conditional_formatting_watch",
        artifact_path: profile.formatting_watch_artifacts.join(";"),
        evidence_state: if missing_paths.is_empty() {
            "watch_classified_no_current_oxcalc_request_path".to_string()
        } else {
            "missing_artifact".to_string()
        },
        observations: vec![
            "OxFml W073 aggregate and visualization conditional-formatting metadata is typed_rule-only."
                .to_string(),
            "OxCalc artifacts in this gate do not construct those payloads; no local code patch or handoff is required here."
                .to_string(),
        ],
        reason_ids: Vec::new(),
    });
}

fn evaluate_supplemental_evidence(
    repo_root: &Path,
    profile: &PackCapabilityProfile,
    evaluation: &mut EvidenceEvaluation,
) {
    for source in profile.supplemental_evidence {
        let missing_paths = source
            .artifact_paths
            .iter()
            .filter(|relative_path| !repo_root.join(relative_path).exists())
            .map(|relative_path| (*relative_path).to_string())
            .collect::<Vec<_>>();
        evaluation.missing_paths.extend(missing_paths.clone());

        let reason_ids = source
            .reason_ids
            .iter()
            .map(|reason| (*reason).to_string())
            .collect::<Vec<_>>();
        if missing_paths.is_empty() {
            evaluation
                .satisfied_inputs
                .push(source.satisfied_input_id.to_string());
            evaluation.blockers.extend(reason_ids.clone());
        }

        evaluation.rows.push(EvidenceRow {
            input_id: source.input_id,
            artifact_path: source.artifact_paths.join(";"),
            evidence_state: if missing_paths.is_empty() {
                source.evidence_state_present.to_string()
            } else {
                "missing_artifact".to_string()
            },
            observations: source
                .observations
                .iter()
                .map(|observation| (*observation).to_string())
                .collect(),
            reason_ids,
        });
    }
}

fn add_static_program_blockers(
    profile: &PackCapabilityProfile,
    evaluation: &mut EvidenceEvaluation,
) {
    let mut reason_ids = vec![
        "pack.grade.program_grade_replay_governance_not_reached".to_string(),
        "pack.grade.retained_witness_promotion_not_shared_program_grade".to_string(),
    ];
    reason_ids.extend(
        profile
            .additional_static_blockers
            .iter()
            .map(|reason| (*reason).to_string()),
    );
    evaluation.blockers.extend(reason_ids.clone());
    evaluation.rows.push(EvidenceRow {
        input_id: "program_grade_governance",
        artifact_path: profile.program_governance_artifact.to_string(),
        evidence_state: "policy_blocker_retained".to_string(),
        observations: vec![
            "Successor evidence widens local proof/replay/conformance but does not establish cap.C5.pack_valid without direct pack-grade replay governance.".to_string(),
        ],
        reason_ids,
    });
}

fn pack_capability_profile(run_id: &str) -> PackCapabilityProfile {
    if run_id.starts_with("w043-") {
        PackCapabilityProfile {
            profile_id: "w043_pack_grade_replay_governance_c5_reassessment",
            oxfml_bridge_run_id: POST_W033_OXFML_BRIDGE_RUN_ID,
            let_lambda_tracecalc_run_id: W037_TRACECALC_OBSERVABLE_RUN_ID,
            let_lambda_treecalc_run_id: W043_TREECALC_CONFORMANCE_RUN_ID,
            independent_treecalc_run_id: W043_TREECALC_CONFORMANCE_RUN_ID,
            independent_conformance_run_id: W036_INDEPENDENT_DIFFERENTIAL_RUN_ID,
            program_governance_artifact: "docs/spec/core-engine/w043-formalization/W043_PACK_GRADE_REPLAY_GOVERNANCE_AND_C5_RELEASE_REASSESSMENT.md",
            formal_artifacts: W043_FORMAL_ARTIFACTS,
            formal_input_id: "w043_proof_model_stage2_formal_packets",
            formal_satisfied_input_id: "w043_proof_model_stage2_packets_present",
            formal_evidence_state_present: "w043_formal_packets_present_with_totality_fairness_stage2_boundaries",
            formal_observations: &[
                "W043 proof/model and Stage 2 artifacts bind Rust totality/refinement boundaries, Lean/TLA bounded fairness evidence, scheduler-equivalence predicates, observable invariance, declared pack-equivalence inputs, and replay-governance blockers.",
                "The artifacts are direct W043 evidence but do not promote full Lean/TLA verification, Stage 2 policy, pack-grade replay, or C5.",
            ],
            formal_reason_ids: &[
                "pack.grade.w043_formal_slices_bounded_not_full_verification",
                "pack.grade.w043_stage2_policy_unpromoted",
                "pack.grade.w043_pack_grade_replay_governance_absent",
            ],
            formatting_watch_artifacts: W043_FORMATTING_WATCH_ARTIFACTS,
            additional_static_blockers: &[
                "pack.grade.w043_program_grade_replay_governance_not_reached",
                "pack.grade.w043_pack_grade_replay_governance_absent",
                "pack.grade.w043_pack_c5_no_promotion_after_reassessment",
                "pack.grade.w043_release_grade_decision_deferred_to_calc_2p3_10",
            ],
            supplemental_evidence: W043_SUPPLEMENTAL_EVIDENCE,
            successor_lanes: &[
                "calc-2p3.10",
                "future_pack_grade_replay_governance_service",
                "future_retained_witness_lifecycle_service",
                "future_retained_history_service_endpoint",
                "future_operated_continuous_assurance_service",
                "future_operated_cross_engine_diff_service",
                "future_stage2_partition_equivalence_packet",
                "future_independent_evaluator_implementation",
                "future_mismatch_quarantine_service",
                "future_callable_metadata_projection_fixture",
                "future_callable_carrier_sufficiency_proof",
                "future_registered_external_projection_fixture",
                "future_w073_downstream_typed_rule_request_construction_uptake",
            ],
        }
    } else if run_id.starts_with("w042-") {
        PackCapabilityProfile {
            profile_id: "w042_pack_grade_replay_governance_c5_reassessment",
            oxfml_bridge_run_id: POST_W033_OXFML_BRIDGE_RUN_ID,
            let_lambda_tracecalc_run_id: W037_TRACECALC_OBSERVABLE_RUN_ID,
            let_lambda_treecalc_run_id: W042_TREECALC_CONFORMANCE_RUN_ID,
            independent_treecalc_run_id: W042_TREECALC_CONFORMANCE_RUN_ID,
            independent_conformance_run_id: W036_INDEPENDENT_DIFFERENTIAL_RUN_ID,
            program_governance_artifact: "docs/spec/core-engine/w042-formalization/W042_PACK_GRADE_REPLAY_GOVERNANCE_AND_C5_REASSESSMENT.md",
            formal_artifacts: W042_FORMAL_ARTIFACTS,
            formal_input_id: "w042_proof_model_stage2_formal_packets",
            formal_satisfied_input_id: "w042_proof_model_stage2_packets_present",
            formal_evidence_state_present: "w042_formal_packets_present_with_totality_fairness_stage2_boundaries",
            formal_observations: &[
                "W042 proof/model and Stage 2 artifacts bind Rust totality/refinement boundaries, Lean/TLA bounded fairness evidence, production-analyzer predicates, observable invariance, declared pack-equivalence inputs, and replay-governance blockers.",
                "The artifacts are direct W042 evidence but do not promote full Lean/TLA verification, Stage 2 policy, pack-grade replay, or C5.",
            ],
            formal_reason_ids: &[
                "pack.grade.w042_formal_slices_bounded_not_full_verification",
                "pack.grade.w042_stage2_policy_unpromoted",
                "pack.grade.w042_pack_grade_replay_governance_absent",
            ],
            formatting_watch_artifacts: W042_FORMATTING_WATCH_ARTIFACTS,
            additional_static_blockers: &[
                "pack.grade.w042_program_grade_replay_governance_not_reached",
                "pack.grade.w042_pack_grade_replay_governance_absent",
                "pack.grade.w042_pack_c5_no_promotion_after_reassessment",
                "pack.grade.w042_release_grade_decision_deferred_to_calc_czd10",
            ],
            supplemental_evidence: W042_SUPPLEMENTAL_EVIDENCE,
            successor_lanes: &[
                "calc-czd.10",
                "future_pack_grade_replay_governance_service",
                "future_retained_witness_lifecycle_service",
                "future_retained_history_service_endpoint",
                "future_operated_continuous_assurance_service",
                "future_operated_cross_engine_diff_service",
                "future_stage2_partition_equivalence_packet",
                "future_independent_evaluator_implementation",
                "future_mismatch_quarantine_service",
                "future_callable_metadata_projection_fixture",
                "future_callable_carrier_sufficiency_proof",
                "future_registered_external_projection_fixture",
            ],
        }
    } else if run_id.starts_with("w041-") {
        PackCapabilityProfile {
            profile_id: "w041_pack_grade_replay_governance_c5_reassessment",
            oxfml_bridge_run_id: POST_W033_OXFML_BRIDGE_RUN_ID,
            let_lambda_tracecalc_run_id: W037_TRACECALC_OBSERVABLE_RUN_ID,
            let_lambda_treecalc_run_id: W041_TREECALC_CONFORMANCE_RUN_ID,
            independent_treecalc_run_id: W041_TREECALC_CONFORMANCE_RUN_ID,
            independent_conformance_run_id: W036_INDEPENDENT_DIFFERENTIAL_RUN_ID,
            program_governance_artifact: "docs/spec/core-engine/w041-formalization/W041_PACK_GRADE_REPLAY_GOVERNANCE_AND_C5_REASSESSMENT.md",
            formal_artifacts: W041_FORMAL_ARTIFACTS,
            formal_input_id: "w041_proof_model_stage2_formal_packets",
            formal_satisfied_input_id: "w041_proof_model_stage2_packets_present",
            formal_evidence_state_present: "w041_formal_packets_present_with_totality_fairness_stage2_boundaries",
            formal_observations: &[
                "W041 proof/model and Stage 2 artifacts bind Rust totality/refinement, Lean/TLA bounded fairness evidence, bounded production-analyzer predicates, observable equivalence, and replay-governance blockers.",
                "The artifacts are direct W041 evidence but do not promote full Lean/TLA verification, Stage 2 policy, pack-grade replay, or C5.",
            ],
            formal_reason_ids: &[
                "pack.grade.w041_formal_slices_bounded_not_full_verification",
                "pack.grade.w041_stage2_policy_unpromoted",
                "pack.grade.w041_pack_grade_replay_governance_absent",
            ],
            formatting_watch_artifacts: W041_FORMATTING_WATCH_ARTIFACTS,
            additional_static_blockers: &[
                "pack.grade.w041_program_grade_replay_governance_not_reached",
                "pack.grade.w041_pack_grade_replay_governance_absent",
                "pack.grade.w041_pack_c5_no_promotion_after_reassessment",
                "pack.grade.w041_release_grade_decision_deferred_to_calc_sui10",
            ],
            supplemental_evidence: W041_SUPPLEMENTAL_EVIDENCE,
            successor_lanes: &[
                "calc-sui.10",
                "future_pack_grade_replay_governance_service",
                "future_retained_witness_lifecycle_service",
                "future_retained_history_service_endpoint",
                "future_operated_continuous_assurance_service",
                "future_operated_cross_engine_diff_service",
                "future_stage2_partition_equivalence_packet",
                "future_independent_evaluator_implementation",
                "future_callable_metadata_projection_fixture",
            ],
        }
    } else if run_id.starts_with("w040-") {
        PackCapabilityProfile {
            profile_id: "w040_pack_grade_replay_governance_c5_promotion_decision",
            oxfml_bridge_run_id: POST_W033_OXFML_BRIDGE_RUN_ID,
            let_lambda_tracecalc_run_id: W037_TRACECALC_OBSERVABLE_RUN_ID,
            let_lambda_treecalc_run_id: W040_TREECALC_CONFORMANCE_RUN_ID,
            independent_treecalc_run_id: W040_TREECALC_CONFORMANCE_RUN_ID,
            independent_conformance_run_id: W036_INDEPENDENT_DIFFERENTIAL_RUN_ID,
            program_governance_artifact: "docs/spec/core-engine/w040-formalization/W040_PACK_GRADE_REPLAY_GOVERNANCE_AND_C5_PROMOTION_DECISION.md",
            formal_artifacts: W040_FORMAL_ARTIFACTS,
            formal_input_id: "w040_proof_model_stage2_formal_packets",
            formal_satisfied_input_id: "w040_proof_model_stage2_packets_present",
            formal_evidence_state_present: "w040_formal_packets_present_with_totality_stage2_boundaries",
            formal_observations: &[
                "W040 proof/model and Stage 2 artifacts bind Rust totality/refinement, Lean/TLA discharge, bounded production-policy predicates, observable equivalence, and replay-governance blockers.",
                "The artifacts are direct W040 evidence but do not promote full Lean/TLA verification, Stage 2 policy, pack-grade replay, or C5.",
            ],
            formal_reason_ids: &[
                "pack.grade.w040_formal_slices_bounded_not_full_verification",
                "pack.grade.w040_stage2_policy_unpromoted",
                "pack.grade.w040_pack_grade_replay_governance_absent",
            ],
            formatting_watch_artifacts: W040_FORMATTING_WATCH_ARTIFACTS,
            additional_static_blockers: &[
                "pack.grade.w040_program_grade_replay_governance_not_reached",
                "pack.grade.w040_pack_grade_replay_governance_absent",
                "pack.grade.w040_pack_c5_no_promotion_after_reassessment",
                "pack.grade.w040_release_grade_decision_deferred_to_calc_tv510",
            ],
            supplemental_evidence: W040_SUPPLEMENTAL_EVIDENCE,
            successor_lanes: &[
                "calc-tv5.10",
                "future_pack_grade_replay_governance_service",
                "future_retained_witness_lifecycle_service",
                "future_operated_continuous_assurance_service",
                "future_operated_cross_engine_diff_service",
                "future_stage2_partition_equivalence_packet",
                "future_independent_evaluator_implementation",
                "future_callable_metadata_projection_fixture",
            ],
        }
    } else if run_id.starts_with("w039-") {
        PackCapabilityProfile {
            profile_id: "w039_pack_grade_replay_governance_c5_reassessment",
            oxfml_bridge_run_id: POST_W033_OXFML_BRIDGE_RUN_ID,
            let_lambda_tracecalc_run_id: W037_TRACECALC_OBSERVABLE_RUN_ID,
            let_lambda_treecalc_run_id: W037_TREECALC_CONFORMANCE_RUN_ID,
            independent_treecalc_run_id: W037_TREECALC_CONFORMANCE_RUN_ID,
            independent_conformance_run_id: W036_INDEPENDENT_DIFFERENTIAL_RUN_ID,
            program_governance_artifact: "docs/spec/core-engine/w039-formalization/W039_PACK_GRADE_REPLAY_GOVERNANCE_AND_C5_REASSESSMENT.md",
            formal_artifacts: W039_FORMAL_ARTIFACTS,
            formal_input_id: "w039_proof_model_stage2_formal_packets",
            formal_satisfied_input_id: "w039_proof_model_stage2_packets_present",
            formal_evidence_state_present: "w039_formal_packets_present_with_totality_and_stage2_boundaries",
            formal_observations: &[
                "W039 proof/model and Stage 2 artifacts bind totality classification, bounded model evidence, production-policy predicates, and replay-governance blockers.",
                "The artifacts are direct W039 evidence but do not promote full Lean/TLA verification, Stage 2 policy, pack-grade replay, or C5.",
            ],
            formal_reason_ids: &[
                "pack.grade.w039_formal_slices_bounded_not_full_verification",
                "pack.grade.w039_stage2_policy_unpromoted",
                "pack.grade.w039_pack_grade_replay_governance_absent",
            ],
            formatting_watch_artifacts: W039_FORMATTING_WATCH_ARTIFACTS,
            additional_static_blockers: &[
                "pack.grade.w039_program_grade_replay_governance_not_reached",
                "pack.grade.w039_pack_grade_replay_governance_absent",
                "pack.grade.w039_pack_c5_no_promotion_after_reassessment",
                "pack.grade.w039_release_grade_decision_deferred_to_calc_f7o9",
            ],
            supplemental_evidence: W039_SUPPLEMENTAL_EVIDENCE,
            successor_lanes: &[
                "calc-f7o.9",
                "future_pack_grade_replay_governance_service",
                "future_retained_witness_lifecycle_service",
                "future_operated_continuous_assurance_service",
                "future_operated_cross_engine_diff_service",
                "future_stage2_partition_equivalence_packet",
                "future_independent_evaluator_implementation",
                "future_callable_metadata_projection_fixture",
            ],
        }
    } else if run_id.starts_with("w038-") {
        PackCapabilityProfile {
            profile_id: "w038_pack_grade_replay_governance_c5_release_decision",
            oxfml_bridge_run_id: POST_W033_OXFML_BRIDGE_RUN_ID,
            let_lambda_tracecalc_run_id: W037_TRACECALC_OBSERVABLE_RUN_ID,
            let_lambda_treecalc_run_id: W037_TREECALC_CONFORMANCE_RUN_ID,
            independent_treecalc_run_id: W037_TREECALC_CONFORMANCE_RUN_ID,
            independent_conformance_run_id: W036_INDEPENDENT_DIFFERENTIAL_RUN_ID,
            program_governance_artifact: "docs/spec/core-engine/w038-formalization/W038_PACK_GRADE_REPLAY_GOVERNANCE_AND_C5_RELEASE_DECISION.md",
            formal_artifacts: W038_FORMAL_ARTIFACTS,
            formal_input_id: "w038_proof_model_stage2_formal_packets",
            formal_satisfied_input_id: "w038_proof_model_stage2_packets_present",
            formal_evidence_state_present: "w038_formal_packets_present_with_totality_and_stage2_boundaries",
            formal_observations: &[
                "W038 proof/model and Stage 2 artifacts bind assumption-discharge, totality, bounded replay, and semantic-equivalence evidence.",
                "The artifacts are direct W038 evidence but do not promote full Lean/TLA verification, Stage 2 policy, pack-grade replay, or C5.",
            ],
            formal_reason_ids: &[
                "pack.grade.w038_formal_slices_bounded_not_full_verification",
                "pack.grade.w038_stage2_policy_unpromoted",
            ],
            formatting_watch_artifacts: W038_FORMATTING_WATCH_ARTIFACTS,
            additional_static_blockers: &[
                "pack.grade.w038_program_grade_replay_governance_not_reached",
                "pack.grade.w038_pack_c5_no_promotion_after_reassessment",
            ],
            supplemental_evidence: W038_SUPPLEMENTAL_EVIDENCE,
            successor_lanes: &[
                "calc-zsr.9",
                "future_pack_grade_replay_governance_service",
                "future_operated_continuous_assurance_service",
                "future_operated_cross_engine_diff_service",
                "future_stage2_partition_equivalence_packet",
                "future_independent_evaluator_row_set",
            ],
        }
    } else if run_id.starts_with("w037-") {
        PackCapabilityProfile {
            profile_id: "w037_pack_grade_replay_governance_c5_candidate",
            oxfml_bridge_run_id: POST_W033_OXFML_BRIDGE_RUN_ID,
            let_lambda_tracecalc_run_id: W037_TRACECALC_OBSERVABLE_RUN_ID,
            let_lambda_treecalc_run_id: W037_TREECALC_CONFORMANCE_RUN_ID,
            independent_treecalc_run_id: W037_TREECALC_CONFORMANCE_RUN_ID,
            independent_conformance_run_id: W036_INDEPENDENT_DIFFERENTIAL_RUN_ID,
            program_governance_artifact: "docs/spec/core-engine/w037-formalization/W037_PACK_GRADE_REPLAY_GOVERNANCE_AND_C5_CANDIDATE_DECISION.md",
            formal_artifacts: W037_FORMAL_ARTIFACTS,
            formal_input_id: "w037_proof_model_and_stage2_formal_packets",
            formal_satisfied_input_id: "w037_proof_model_and_stage2_packets_present",
            formal_evidence_state_present: "w037_formal_packets_present_bounded_no_c5_promotion",
            formal_observations: &[
                "W037 Lean/TLA artifacts are checked proof/model and Stage 2 criteria slices, not total verification.",
                "Stage 2 deterministic replay criteria are explicit, but Stage 2 policy remains unpromoted.",
            ],
            formal_reason_ids: &[
                "pack.grade.w037_formal_slices_bounded_not_full_verification",
                "pack.grade.w037_stage2_scheduler_policy_unpromoted",
            ],
            formatting_watch_artifacts: W037_FORMATTING_WATCH_ARTIFACTS,
            additional_static_blockers: &[
                "pack.grade.w037_program_grade_replay_governance_not_reached",
                "pack.grade.w037_pack_c5_no_promotion_after_reassessment",
            ],
            supplemental_evidence: W037_SUPPLEMENTAL_EVIDENCE,
            successor_lanes: &[
                "calc-ubd.9",
                "future_pack_grade_replay_governance_service",
                "future_operated_continuous_assurance_service",
                "future_continuous_cross_engine_diff_service",
                "future_stage2_partition_equivalence_packet",
            ],
        }
    } else if run_id.starts_with("w036-") {
        PackCapabilityProfile {
            profile_id: "w036_pack_grade_replay_capability_reassessment",
            oxfml_bridge_run_id: POST_W033_OXFML_BRIDGE_RUN_ID,
            let_lambda_tracecalc_run_id: W036_TRACECALC_COVERAGE_RUN_ID,
            let_lambda_treecalc_run_id: W034_TREECALC_RUN_ID,
            independent_treecalc_run_id: W034_TREECALC_RUN_ID,
            independent_conformance_run_id: W036_INDEPENDENT_DIFFERENTIAL_RUN_ID,
            program_governance_artifact: "docs/spec/core-engine/w036-formalization/W036_PACK_GRADE_REPLAY_AND_CAPABILITY_PROMOTION_GATE_REASSESSMENT.md",
            formal_artifacts: W036_FORMAL_ARTIFACTS,
            formal_input_id: "w036_lean_tla_formal_gate_packets",
            formal_satisfied_input_id: "w036_lean_tla_packets_present",
            formal_evidence_state_present: "w036_formal_packets_present_bounded_no_promotion",
            formal_observations: &[
                "W036 Lean artifacts expand theorem coverage inventory and callable boundary classification without full Lean verification.",
                "W036 TLA artifacts add bounded Stage 2 partition evidence but keep Stage 2 policy unpromoted.",
            ],
            formal_reason_ids: &[
                "pack.grade.w036_formal_slices_bounded_not_full_verification",
                "pack.grade.w036_stage2_scheduler_policy_unpromoted",
            ],
            formatting_watch_artifacts: W036_FORMATTING_WATCH_ARTIFACTS,
            additional_static_blockers: &[
                "pack.grade.w036_program_grade_replay_governance_not_reached",
                "pack.grade.w036_direct_oxfml_evaluator_reexecution_absent",
                "pack.grade.w036_pack_c5_no_promotion_after_reassessment",
            ],
            supplemental_evidence: W036_SUPPLEMENTAL_EVIDENCE,
            successor_lanes: &[
                "calc-rqq.9",
                "future_pack_grade_replay_governance",
                "future_operated_continuous_assurance_service",
                "future_continuous_cross_engine_diff_service",
                "future_stage2_partition_equivalence_packet",
            ],
        }
    } else if run_id.starts_with("w035-") {
        PackCapabilityProfile {
            profile_id: "w035_pack_stage2_readiness_reassessment",
            oxfml_bridge_run_id: POST_W033_OXFML_BRIDGE_RUN_ID,
            let_lambda_tracecalc_run_id: W035_TRACECALC_ORACLE_MATRIX_RUN_ID,
            let_lambda_treecalc_run_id: W034_TREECALC_RUN_ID,
            independent_treecalc_run_id: W034_TREECALC_RUN_ID,
            independent_conformance_run_id: W034_INDEPENDENT_CONFORMANCE_RUN_ID,
            program_governance_artifact: "docs/spec/core-engine/w035-formalization/W035_PACK_CAPABILITY_AND_STAGE2_READINESS_REASSESSMENT.md",
            formal_artifacts: W035_FORMAL_ARTIFACTS,
            formal_input_id: "w035_lean_tla_formal_gate_packets",
            formal_satisfied_input_id: "w035_lean_tla_packets_present",
            formal_evidence_state_present: "formal_gate_packets_present_bounded_no_promotion",
            formal_observations: &[
                "W035 Lean artifacts classify local proof rows, external seam assumptions, and opaque OxFunc kernel boundaries.",
                "W035 TLA artifacts check bounded non-routine scheduler/overlay models and keep Stage 2 policy unpromoted.",
            ],
            formal_reason_ids: &[
                "pack.grade.w035_formal_slices_bounded_not_full_verification",
                "pack.grade.stage2_scheduler_preconditions_not_satisfied",
            ],
            formatting_watch_artifacts: W035_FORMATTING_WATCH_ARTIFACTS,
            additional_static_blockers: &[
                "pack.grade.continuous_scale_assurance_unpromoted",
                "pack.grade.stage2_scheduler_policy_unpromoted",
                "pack.grade.pack_c5_no_promotion_after_w035_reassessment",
                "pack.grade.w035_closure_audit_not_yet_recorded",
            ],
            supplemental_evidence: W035_SUPPLEMENTAL_EVIDENCE,
            successor_lanes: &[
                "calc-tkq.8",
                "future_pack_grade_replay_governance",
                "future_continuous_cross_engine_diff_service",
                "future_stage2_partition_equivalence_packet",
            ],
        }
    } else if run_id.starts_with("w034-") {
        PackCapabilityProfile {
            profile_id: "w034_formalization_gate_binding",
            oxfml_bridge_run_id: POST_W033_OXFML_BRIDGE_RUN_ID,
            let_lambda_tracecalc_run_id: W034_TRACECALC_RUN_ID,
            let_lambda_treecalc_run_id: W034_TREECALC_RUN_ID,
            independent_treecalc_run_id: W034_TREECALC_RUN_ID,
            independent_conformance_run_id: W034_INDEPENDENT_CONFORMANCE_RUN_ID,
            program_governance_artifact: "docs/spec/core-engine/w034-formalization/W034_PACK_CAPABILITY_AND_CONTINUOUS_SCALE_GATE_BINDING.md",
            formal_artifacts: W034_FORMAL_ARTIFACTS,
            formal_input_id: "w034_lean_tla_formal_gate_packets",
            formal_satisfied_input_id: "w034_lean_tla_packets_present",
            formal_evidence_state_present: "formal_gate_packets_present_bounded_no_promotion",
            formal_observations: &[
                "W034 Lean/TLA artifacts are checked bounded proof/model slices, not full Lean/TLA verification.",
                "Stage 2 contention is modeled as blocked precondition evidence, not promoted scheduler policy.",
            ],
            formal_reason_ids: &[
                "pack.grade.w034_formal_slices_bounded_not_full_verification",
                "pack.grade.stage2_contention_preconditions_unpromoted",
            ],
            formatting_watch_artifacts: W034_FORMATTING_WATCH_ARTIFACTS,
            additional_static_blockers: &[
                "pack.grade.continuous_scale_assurance_unpromoted",
                "pack.grade.w034_closure_audit_not_yet_recorded",
            ],
            supplemental_evidence: &[],
            successor_lanes: &[
                "calc-rcr",
                "calc-8lg",
                "future_program_grade_pack_scope",
                "future_continuous_cross_engine_diff_suite",
                "w034_closure_audit",
            ],
        }
    } else {
        PackCapabilityProfile {
            profile_id: "post_w033_pack_capability_decision",
            oxfml_bridge_run_id: POST_W033_OXFML_BRIDGE_RUN_ID,
            let_lambda_tracecalc_run_id: POST_W033_LET_LAMBDA_TRACECALC_RUN_ID,
            let_lambda_treecalc_run_id: POST_W033_LET_LAMBDA_TREECALC_RUN_ID,
            independent_treecalc_run_id: POST_W033_INDEPENDENT_TREECALC_RUN_ID,
            independent_conformance_run_id: POST_W033_INDEPENDENT_CONFORMANCE_RUN_ID,
            program_governance_artifact: "docs/spec/core-engine/w033-formalization/W033_PACK_CAPABILITY_BINDING.md",
            formal_artifacts: &[],
            formal_input_id: "",
            formal_satisfied_input_id: "",
            formal_evidence_state_present: "",
            formal_observations: &[],
            formal_reason_ids: &[],
            formatting_watch_artifacts: &[],
            additional_static_blockers: &[],
            supplemental_evidence: &[],
            successor_lanes: &[
                "calc-rcr",
                "calc-8lg",
                "future_program_grade_pack_scope",
                "future_continuous_cross_engine_diff_suite",
                "w034_closure_audit",
            ],
        }
    }
}

fn evidence_row_json(row: &EvidenceRow) -> Value {
    json!({
        "input_id": row.input_id,
        "artifact_path": row.artifact_path,
        "evidence_state": row.evidence_state,
        "observations": row.observations,
        "reason_ids": row.reason_ids,
    })
}

fn retained_validation_path(file_name: &str) -> String {
    relative_artifact_path([
        "docs",
        "test-runs",
        "core-engine",
        "tracecalc-retained-failures",
        TRACE_RETAINED_W023_RUN_ID,
        "replay-appliance",
        "validation",
        file_name,
    ])
}

fn bundle_validation_path(lane: &str, run_id: &str) -> String {
    relative_artifact_path([
        "docs",
        "test-runs",
        "core-engine",
        lane,
        run_id,
        "replay-appliance",
        "validation",
        "bundle_validation.json",
    ])
}

fn add_missing_if_absent(evaluation: &mut EvidenceEvaluation, path: &str, value: &Option<Value>) {
    if value.is_none() {
        evaluation.missing_paths.push(path.to_string());
    }
}

fn dedup_strings(values: &mut Vec<String>) {
    let mut unique = Vec::new();
    for value in values.drain(..) {
        if !unique.contains(&value) {
            unique.push(value);
        }
    }
    *values = unique;
}

fn text_at(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or("<missing>")
        .to_string()
}

fn number_at(value: &Value, key: &str) -> u64 {
    value.get(key).and_then(Value::as_u64).unwrap_or(0)
}

fn bool_at(value: &Value, key: &str) -> bool {
    value.get(key).and_then(Value::as_bool).unwrap_or(false)
}

fn read_json(repo_root: &Path, relative_path: &str) -> Result<Option<Value>, PackCapabilityError> {
    let path = repo_root.join(relative_path);
    if !path.exists() {
        return Ok(None);
    }
    let content =
        fs::read_to_string(&path).map_err(|source| PackCapabilityError::ReadArtifact {
            path: path.display().to_string(),
            source,
        })?;
    serde_json::from_str(&content)
        .map(Some)
        .map_err(|source| PackCapabilityError::ParseJson {
            path: path.display().to_string(),
            source,
        })
}

fn create_directory(path: &Path) -> Result<(), PackCapabilityError> {
    fs::create_dir_all(path).map_err(|source| PackCapabilityError::CreateDirectory {
        path: path.display().to_string(),
        source,
    })
}

fn write_json(path: &Path, value: &Value) -> Result<(), PackCapabilityError> {
    let content = serde_json::to_string_pretty(value).expect("JSON serialization should succeed");
    fs::write(path, format!("{content}\n")).map_err(|source| PackCapabilityError::WriteFile {
        path: path.display().to_string(),
        source,
    })
}

fn required_artifacts(run_id: &str, profile: &PackCapabilityProfile) -> Vec<String> {
    [
        "run_summary.json",
        "decision/pack_capability_decision.json",
        "evidence/evidence_index.json",
        "replay-appliance/bundle_manifest.json",
        "replay-appliance/validation/bundle_validation.json",
    ]
    .iter()
    .map(|artifact| {
        relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "pack-capability",
            run_id,
            artifact,
        ])
    })
    .chain([
        retained_validation_path("pack_grade_decision.json"),
        retained_validation_path("program_grade_decision.json"),
        relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "oxfml-fixture-bridge",
            profile.oxfml_bridge_run_id,
            "run_summary.json",
        ]),
        relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "oxfml-fixture-bridge",
            profile.oxfml_bridge_run_id,
            "replay-appliance",
            "validation",
            "bundle_validation.json",
        ]),
        bundle_validation_path(
            "tracecalc-reference-machine",
            profile.let_lambda_tracecalc_run_id,
        ),
        bundle_validation_path("treecalc-local", profile.let_lambda_treecalc_run_id),
        relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "independent-conformance",
            profile.independent_conformance_run_id,
            "run_summary.json",
        ]),
        relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "independent-conformance",
            profile.independent_conformance_run_id,
            "replay-appliance",
            "validation",
            "bundle_validation.json",
        ]),
        relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "treecalc-local",
            profile.independent_treecalc_run_id,
            "replay-appliance",
            "adapter_capabilities",
            "oxcalc_treecalc.json",
        ]),
        profile.program_governance_artifact.to_string(),
    ])
    .chain(
        profile
            .formal_artifacts
            .iter()
            .map(|artifact| (*artifact).to_string()),
    )
    .chain(
        profile
            .formatting_watch_artifacts
            .iter()
            .map(|artifact| (*artifact).to_string()),
    )
    .chain(
        profile
            .supplemental_evidence
            .iter()
            .flat_map(|source| source.artifact_paths.iter())
            .map(|artifact| (*artifact).to_string()),
    )
    .collect()
}

fn relative_artifact_path<'a>(segments: impl IntoIterator<Item = &'a str>) -> String {
    segments.into_iter().collect::<Vec<_>>().join("/")
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    static TEST_REPO_COUNTER: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn pack_capability_runner_keeps_c5_unpromoted_when_blockers_remain() {
        let repo_root = unique_temp_repo();
        create_source_artifacts(&repo_root);

        let summary = PackCapabilityRunner::new()
            .execute(&repo_root, "pack-test")
            .expect("pack capability packet should write");

        assert_eq!(summary.decision_status, "capability_not_promoted");
        assert_eq!(summary.highest_honest_capability, "cap.C4.distill_valid");
        assert_eq!(summary.missing_artifact_count, 0);
        assert!(summary.blocker_count > 0);

        let decision = read_required_json(
            &repo_root,
            "docs/test-runs/core-engine/pack-capability/pack-test/decision/pack_capability_decision.json",
        );
        assert_eq!(decision["capability_promoted"], false);

        let validation = read_required_json(
            &repo_root,
            "docs/test-runs/core-engine/pack-capability/pack-test/replay-appliance/validation/bundle_validation.json",
        );
        assert_eq!(validation["status"], "bundle_valid");

        fs::remove_dir_all(repo_root.parent().unwrap()).unwrap();
    }

    #[test]
    fn pack_capability_runner_binds_w034_formal_gate_inputs() {
        let repo_root = unique_temp_repo();
        create_w034_source_artifacts(&repo_root);

        let summary = PackCapabilityRunner::new()
            .execute(&repo_root, "w034-pack-test")
            .expect("W034 pack capability packet should write");

        assert_eq!(summary.decision_status, "capability_not_promoted");
        assert_eq!(summary.highest_honest_capability, "cap.C4.distill_valid");
        assert_eq!(summary.missing_artifact_count, 0);

        let decision = read_required_json(
            &repo_root,
            "docs/test-runs/core-engine/pack-capability/w034-pack-test/decision/pack_capability_decision.json",
        );
        assert_eq!(
            decision["evidence_profile"],
            "w034_formalization_gate_binding"
        );
        assert!(
            decision["no_promotion_reason_ids"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason.as_str()
                    == Some("pack.grade.w034_formal_slices_bounded_not_full_verification"))
        );

        let validation = read_required_json(
            &repo_root,
            "docs/test-runs/core-engine/pack-capability/w034-pack-test/replay-appliance/validation/bundle_validation.json",
        );
        assert_eq!(validation["status"], "bundle_valid");

        fs::remove_dir_all(repo_root.parent().unwrap()).unwrap();
    }

    #[test]
    fn pack_capability_runner_binds_w035_reassessment_inputs() {
        let repo_root = unique_temp_repo();
        create_w035_source_artifacts(&repo_root);

        let summary = PackCapabilityRunner::new()
            .execute(&repo_root, "w035-pack-stage2-test")
            .expect("W035 pack capability packet should write");

        assert_eq!(summary.decision_status, "capability_not_promoted");
        assert_eq!(summary.highest_honest_capability, "cap.C4.distill_valid");
        assert_eq!(summary.missing_artifact_count, 0);

        let decision = read_required_json(
            &repo_root,
            "docs/test-runs/core-engine/pack-capability/w035-pack-stage2-test/decision/pack_capability_decision.json",
        );
        assert_eq!(
            decision["evidence_profile"],
            "w035_pack_stage2_readiness_reassessment"
        );
        assert_eq!(
            decision["stage2_readiness"]["stage2_scheduler_promoted"],
            false
        );
        assert!(
            decision["no_promotion_reason_ids"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason.as_str()
                    == Some("pack.grade.stage2_scheduler_preconditions_not_satisfied"))
        );
        assert!(
            decision["no_promotion_reason_ids"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason.as_str()
                    == Some("pack.grade.continuous_assurance_gate_not_running_service"))
        );

        let validation = read_required_json(
            &repo_root,
            "docs/test-runs/core-engine/pack-capability/w035-pack-stage2-test/replay-appliance/validation/bundle_validation.json",
        );
        assert_eq!(validation["status"], "bundle_valid");

        fs::remove_dir_all(repo_root.parent().unwrap()).unwrap();
    }

    #[test]
    fn pack_capability_runner_binds_w036_reassessment_inputs() {
        let repo_root = unique_temp_repo();
        create_w036_source_artifacts(&repo_root);

        let summary = PackCapabilityRunner::new()
            .execute(&repo_root, "w036-pack-capability-test")
            .expect("W036 pack capability packet should write");

        assert_eq!(summary.decision_status, "capability_not_promoted");
        assert_eq!(summary.highest_honest_capability, "cap.C4.distill_valid");
        assert_eq!(summary.missing_artifact_count, 0);
        assert_eq!(summary.satisfied_input_count, 12);
        assert_eq!(summary.blocker_count, 22);

        let decision = read_required_json(
            &repo_root,
            "docs/test-runs/core-engine/pack-capability/w036-pack-capability-test/decision/pack_capability_decision.json",
        );
        assert_eq!(
            decision["evidence_profile"],
            "w036_pack_grade_replay_capability_reassessment"
        );
        assert_eq!(decision["capability_promoted"], false);
        assert_eq!(
            decision["stage2_readiness"]["stage2_scheduler_promoted"],
            false
        );
        assert!(
            decision["no_promotion_reason_ids"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason.as_str()
                    == Some("pack.grade.w036_continuous_assurance_simulated_not_operated"))
        );
        assert!(
            decision["no_promotion_reason_ids"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason.as_str()
                    == Some("pack.grade.w036_pack_c5_no_promotion_after_reassessment"))
        );

        let validation = read_required_json(
            &repo_root,
            "docs/test-runs/core-engine/pack-capability/w036-pack-capability-test/replay-appliance/validation/bundle_validation.json",
        );
        assert_eq!(validation["status"], "bundle_valid");

        fs::remove_dir_all(repo_root.parent().unwrap()).unwrap();
    }

    #[test]
    fn pack_capability_runner_binds_w037_c5_candidate_inputs() {
        let repo_root = unique_temp_repo();
        create_w037_source_artifacts(&repo_root);

        let summary = PackCapabilityRunner::new()
            .execute(&repo_root, "w037-pack-capability-test")
            .expect("W037 pack capability packet should write");

        assert_eq!(summary.decision_status, "capability_not_promoted");
        assert_eq!(summary.highest_honest_capability, "cap.C4.distill_valid");
        assert_eq!(summary.missing_artifact_count, 0);
        assert_eq!(summary.satisfied_input_count, 13);
        assert_eq!(summary.blocker_count, 22);

        let decision = read_required_json(
            &repo_root,
            "docs/test-runs/core-engine/pack-capability/w037-pack-capability-test/decision/pack_capability_decision.json",
        );
        assert_eq!(
            decision["evidence_profile"],
            "w037_pack_grade_replay_governance_c5_candidate"
        );
        assert_eq!(decision["capability_promoted"], false);
        assert_eq!(
            decision["stage2_readiness"]["stage2_scheduler_promoted"],
            false
        );
        assert!(
            decision["satisfied_inputs"]
                .as_array()
                .unwrap()
                .iter()
                .any(|input| input.as_str() == Some("w037_direct_oxfml_evaluator_valid"))
        );
        assert!(
            !decision["no_promotion_reason_ids"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason.as_str()
                    == Some("pack.grade.direct_oxfml_evaluator_reexecution_absent"))
        );
        assert!(
            decision["no_promotion_reason_ids"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason.as_str()
                    == Some("pack.grade.w037_pack_c5_no_promotion_after_reassessment"))
        );
        assert!(
            decision["no_promotion_reason_ids"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason.as_str()
                    == Some("pack.grade.w037_operated_continuous_assurance_service_absent"))
        );

        let validation = read_required_json(
            &repo_root,
            "docs/test-runs/core-engine/pack-capability/w037-pack-capability-test/replay-appliance/validation/bundle_validation.json",
        );
        assert_eq!(validation["status"], "bundle_valid");

        fs::remove_dir_all(repo_root.parent().unwrap()).unwrap();
    }

    #[test]
    fn pack_capability_runner_binds_w038_release_decision_inputs() {
        let repo_root = unique_temp_repo();
        create_w038_source_artifacts(&repo_root);

        let summary = PackCapabilityRunner::new()
            .execute(&repo_root, "w038-pack-capability-test")
            .expect("W038 pack capability packet should write");

        assert_eq!(summary.decision_status, "capability_not_promoted");
        assert_eq!(summary.highest_honest_capability, "cap.C4.distill_valid");
        assert_eq!(summary.missing_artifact_count, 0);
        assert_eq!(summary.satisfied_input_count, 13);
        assert_eq!(summary.blocker_count, 25);

        let decision = read_required_json(
            &repo_root,
            "docs/test-runs/core-engine/pack-capability/w038-pack-capability-test/decision/pack_capability_decision.json",
        );
        assert_eq!(
            decision["evidence_profile"],
            "w038_pack_grade_replay_governance_c5_release_decision"
        );
        assert_eq!(decision["capability_promoted"], false);
        assert_eq!(
            decision["stage2_readiness"]["stage2_scheduler_promoted"],
            false
        );
        assert!(
            decision["satisfied_inputs"]
                .as_array()
                .unwrap()
                .iter()
                .any(|input| input.as_str() == Some("w038_stage2_partition_replay_valid"))
        );
        assert!(
            decision["satisfied_inputs"]
                .as_array()
                .unwrap()
                .iter()
                .any(|input| input.as_str() == Some("w038_independent_diversity_seam_watch_valid"))
        );
        assert!(
            !decision["no_promotion_reason_ids"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason.as_str()
                    == Some("pack.grade.direct_oxfml_evaluator_reexecution_absent"))
        );
        assert!(
            decision["no_promotion_reason_ids"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason.as_str()
                    == Some("pack.grade.w038_pack_c5_no_promotion_after_reassessment"))
        );
        assert!(
            decision["no_promotion_reason_ids"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason.as_str()
                    == Some("pack.grade.w038_operated_continuous_assurance_service_absent"))
        );
        assert!(
            decision["no_promotion_reason_ids"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason.as_str()
                    == Some("pack.grade.w038_fully_independent_evaluator_absent"))
        );

        let validation = read_required_json(
            &repo_root,
            "docs/test-runs/core-engine/pack-capability/w038-pack-capability-test/replay-appliance/validation/bundle_validation.json",
        );
        assert_eq!(validation["status"], "bundle_valid");

        fs::remove_dir_all(repo_root.parent().unwrap()).unwrap();
    }

    #[test]
    fn pack_capability_runner_binds_w039_c5_reassessment_inputs() {
        let repo_root = unique_temp_repo();
        create_w039_source_artifacts(&repo_root);

        let summary = PackCapabilityRunner::new()
            .execute(&repo_root, "w039-pack-capability-test")
            .expect("W039 pack capability packet should write");

        assert_eq!(summary.decision_status, "capability_not_promoted");
        assert_eq!(summary.highest_honest_capability, "cap.C4.distill_valid");
        assert_eq!(summary.missing_artifact_count, 0);
        assert!(summary.satisfied_input_count >= 15);
        assert!(summary.blocker_count > 25);

        let decision = read_required_json(
            &repo_root,
            "docs/test-runs/core-engine/pack-capability/w039-pack-capability-test/decision/pack_capability_decision.json",
        );
        assert_eq!(
            decision["evidence_profile"],
            "w039_pack_grade_replay_governance_c5_reassessment"
        );
        assert_eq!(decision["capability_promoted"], false);
        assert_eq!(
            decision["stage2_readiness"]["stage2_scheduler_promoted"],
            false
        );
        assert!(
            decision["satisfied_inputs"]
                .as_array()
                .unwrap()
                .iter()
                .any(|input| input.as_str() == Some("w039_fresh_direct_oxfml_upstream_host_valid"))
        );
        assert!(
            decision["satisfied_inputs"]
                .as_array()
                .unwrap()
                .iter()
                .any(|input| input.as_str()
                    == Some("w039_operated_assurance_retained_history_valid"))
        );
        assert!(
            !decision["no_promotion_reason_ids"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason.as_str()
                    == Some("pack.grade.direct_oxfml_evaluator_reexecution_absent"))
        );
        assert!(
            decision["no_promotion_reason_ids"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason.as_str()
                    == Some("pack.grade.w039_pack_c5_no_promotion_after_reassessment"))
        );
        assert!(
            decision["no_promotion_reason_ids"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason.as_str()
                    == Some("pack.grade.w039_pack_grade_replay_governance_absent"))
        );
        assert!(
            decision["no_promotion_reason_ids"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason.as_str()
                    == Some("pack.grade.w039_retained_history_store_absent"))
        );
        assert!(
            decision["no_promotion_reason_ids"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason.as_str()
                    == Some("pack.grade.w039_callable_metadata_projection_absent"))
        );

        let validation = read_required_json(
            &repo_root,
            "docs/test-runs/core-engine/pack-capability/w039-pack-capability-test/replay-appliance/validation/bundle_validation.json",
        );
        assert_eq!(validation["status"], "bundle_valid");

        fs::remove_dir_all(repo_root.parent().unwrap()).unwrap();
    }

    #[test]
    fn pack_capability_runner_binds_w040_c5_promotion_decision_inputs() {
        let repo_root = unique_temp_repo();
        create_w040_source_artifacts(&repo_root);

        let summary = PackCapabilityRunner::new()
            .execute(&repo_root, "w040-pack-capability-test")
            .unwrap();

        assert_eq!(summary.decision_status, "capability_not_promoted");
        assert_eq!(summary.highest_honest_capability, "cap.C4.distill_valid");
        assert_eq!(summary.missing_artifact_count, 0);
        assert!(summary.satisfied_input_count >= 16);
        assert!(summary.blocker_count > 30);

        let decision = read_required_json(
            &repo_root,
            "docs/test-runs/core-engine/pack-capability/w040-pack-capability-test/decision/pack_capability_decision.json",
        );
        assert_eq!(
            decision["evidence_profile"],
            "w040_pack_grade_replay_governance_c5_promotion_decision"
        );
        assert_eq!(decision["capability_promoted"], false);
        assert_eq!(
            decision["stage2_readiness"]["stage2_scheduler_promoted"],
            false
        );
        assert!(
            decision["satisfied_inputs"]
                .as_array()
                .unwrap()
                .iter()
                .any(|input| input.as_str() == Some("w040_fresh_direct_oxfml_upstream_host_valid"))
        );
        assert!(
            decision["satisfied_inputs"]
                .as_array()
                .unwrap()
                .iter()
                .any(|input| input.as_str()
                    == Some("w040_operated_assurance_retained_history_service_valid"))
        );
        assert!(
            decision["satisfied_inputs"]
                .as_array()
                .unwrap()
                .iter()
                .any(|input| input.as_str()
                    == Some("w040_oxfml_seam_breadth_callable_metadata_valid"))
        );
        assert!(
            !decision["no_promotion_reason_ids"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason.as_str()
                    == Some("pack.grade.direct_oxfml_evaluator_reexecution_absent"))
        );
        assert!(
            decision["no_promotion_reason_ids"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason.as_str()
                    == Some("pack.grade.w040_pack_c5_no_promotion_after_reassessment"))
        );
        assert!(
            decision["no_promotion_reason_ids"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason.as_str()
                    == Some("pack.grade.w040_pack_grade_replay_governance_absent"))
        );
        assert!(
            decision["no_promotion_reason_ids"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason.as_str()
                    == Some("pack.grade.w040_retained_history_service_absent"))
        );
        assert!(
            decision["no_promotion_reason_ids"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason.as_str()
                    == Some("pack.grade.w040_callable_metadata_projection_absent"))
        );

        let validation = read_required_json(
            &repo_root,
            "docs/test-runs/core-engine/pack-capability/w040-pack-capability-test/replay-appliance/validation/bundle_validation.json",
        );
        assert_eq!(validation["status"], "bundle_valid");

        fs::remove_dir_all(repo_root.parent().unwrap()).unwrap();
    }

    #[test]
    fn pack_capability_runner_binds_w041_c5_reassessment_inputs() {
        let repo_root = unique_temp_repo();
        create_w041_source_artifacts(&repo_root);

        let summary = PackCapabilityRunner::new()
            .execute(&repo_root, "w041-pack-capability-test")
            .unwrap();

        assert_eq!(summary.decision_status, "capability_not_promoted");
        assert_eq!(summary.highest_honest_capability, "cap.C4.distill_valid");
        assert_eq!(summary.missing_artifact_count, 0);
        assert!(summary.satisfied_input_count >= 16);
        assert!(summary.blocker_count > 30);

        let decision = read_required_json(
            &repo_root,
            "docs/test-runs/core-engine/pack-capability/w041-pack-capability-test/decision/pack_capability_decision.json",
        );
        assert_eq!(
            decision["evidence_profile"],
            "w041_pack_grade_replay_governance_c5_reassessment"
        );
        assert_eq!(decision["capability_promoted"], false);
        assert_eq!(
            decision["stage2_readiness"]["stage2_scheduler_promoted"],
            false
        );
        assert!(
            decision["satisfied_inputs"]
                .as_array()
                .unwrap()
                .iter()
                .any(|input| input.as_str() == Some("w041_fresh_direct_oxfml_upstream_host_valid"))
        );
        assert!(
            decision["satisfied_inputs"]
                .as_array()
                .unwrap()
                .iter()
                .any(|input| input.as_str()
                    == Some("w041_operated_assurance_retained_history_alert_dispatch_valid"))
        );
        assert!(
            decision["satisfied_inputs"]
                .as_array()
                .unwrap()
                .iter()
                .any(|input| input.as_str()
                    == Some("w041_oxfml_broad_display_publication_callable_carrier_valid"))
        );
        assert!(
            !decision["no_promotion_reason_ids"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason.as_str()
                    == Some("pack.grade.direct_oxfml_evaluator_reexecution_absent"))
        );
        assert!(
            decision["no_promotion_reason_ids"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason.as_str()
                    == Some("pack.grade.w041_pack_c5_no_promotion_after_reassessment"))
        );
        assert!(
            decision["no_promotion_reason_ids"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason.as_str()
                    == Some("pack.grade.w041_pack_grade_replay_governance_absent"))
        );
        assert!(
            decision["no_promotion_reason_ids"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason.as_str()
                    == Some("pack.grade.w041_retained_witness_lifecycle_service_absent"))
        );
        assert!(
            decision["no_promotion_reason_ids"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason.as_str()
                    == Some("pack.grade.w041_callable_carrier_sufficiency_proof_absent"))
        );

        let validation = read_required_json(
            &repo_root,
            "docs/test-runs/core-engine/pack-capability/w041-pack-capability-test/replay-appliance/validation/bundle_validation.json",
        );
        assert_eq!(validation["status"], "bundle_valid");

        fs::remove_dir_all(repo_root.parent().unwrap()).unwrap();
    }

    #[test]
    fn pack_capability_runner_binds_w042_pack_c5_reassessment_inputs() {
        let repo_root = unique_temp_repo();
        create_w042_source_artifacts(&repo_root);

        let summary = PackCapabilityRunner::new()
            .execute(&repo_root, "w042-pack-capability-test")
            .unwrap();

        assert_eq!(summary.decision_status, "capability_not_promoted");
        assert_eq!(summary.highest_honest_capability, "cap.C4.distill_valid");
        assert_eq!(summary.missing_artifact_count, 0);
        assert!(summary.satisfied_input_count >= 16);
        assert!(summary.blocker_count > 35);

        let decision = read_required_json(
            &repo_root,
            "docs/test-runs/core-engine/pack-capability/w042-pack-capability-test/decision/pack_capability_decision.json",
        );
        assert_eq!(
            decision["evidence_profile"],
            "w042_pack_grade_replay_governance_c5_reassessment"
        );
        assert_eq!(decision["capability_promoted"], false);
        assert_eq!(
            decision["stage2_readiness"]["stage2_scheduler_promoted"],
            false
        );
        assert!(
            decision["satisfied_inputs"]
                .as_array()
                .unwrap()
                .iter()
                .any(|input| input.as_str() == Some("w042_fresh_direct_oxfml_upstream_host_valid"))
        );
        assert!(
            decision["satisfied_inputs"]
                .as_array()
                .unwrap()
                .iter()
                .any(|input| input.as_str()
                    == Some("w042_operated_assurance_retained_history_witness_alert_valid"))
        );
        assert!(
            decision["satisfied_inputs"]
                .as_array()
                .unwrap()
                .iter()
                .any(|input| input.as_str()
                    == Some("w042_oxfml_public_migration_callable_registered_external_valid"))
        );
        assert!(
            !decision["no_promotion_reason_ids"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason.as_str()
                    == Some("pack.grade.direct_oxfml_evaluator_reexecution_absent"))
        );
        assert!(
            decision["no_promotion_reason_ids"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason.as_str()
                    == Some("pack.grade.w042_pack_c5_no_promotion_after_reassessment"))
        );
        assert!(
            decision["no_promotion_reason_ids"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason.as_str()
                    == Some("pack.grade.w042_pack_grade_replay_governance_absent"))
        );
        assert!(
            decision["no_promotion_reason_ids"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason.as_str()
                    == Some("pack.grade.w042_retained_witness_lifecycle_service_absent"))
        );
        assert!(
            decision["no_promotion_reason_ids"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason.as_str()
                    == Some("pack.grade.w042_callable_carrier_sufficiency_proof_absent"))
        );
        assert!(
            decision["no_promotion_reason_ids"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason.as_str()
                    == Some("pack.grade.w042_registered_external_projection_absent"))
        );

        let validation = read_required_json(
            &repo_root,
            "docs/test-runs/core-engine/pack-capability/w042-pack-capability-test/replay-appliance/validation/bundle_validation.json",
        );
        assert_eq!(validation["status"], "bundle_valid");

        fs::remove_dir_all(repo_root.parent().unwrap()).unwrap();
    }

    #[test]
    fn pack_capability_runner_binds_w043_pack_c5_reassessment_inputs() {
        let repo_root = unique_temp_repo();
        create_w043_source_artifacts(&repo_root);

        let summary = PackCapabilityRunner::new()
            .execute(&repo_root, "w043-pack-capability-test")
            .unwrap();

        assert_eq!(summary.decision_status, "capability_not_promoted");
        assert_eq!(summary.highest_honest_capability, "cap.C4.distill_valid");
        assert_eq!(summary.missing_artifact_count, 0);
        assert!(summary.satisfied_input_count >= 16);
        assert!(summary.blocker_count > 40);

        let decision = read_required_json(
            &repo_root,
            "docs/test-runs/core-engine/pack-capability/w043-pack-capability-test/decision/pack_capability_decision.json",
        );
        assert_eq!(
            decision["evidence_profile"],
            "w043_pack_grade_replay_governance_c5_reassessment"
        );
        assert_eq!(decision["capability_promoted"], false);
        assert_eq!(
            decision["stage2_readiness"]["stage2_scheduler_promoted"],
            false
        );
        assert!(
            decision["satisfied_inputs"]
                .as_array()
                .unwrap()
                .iter()
                .any(|input| input.as_str() == Some("w043_fresh_direct_oxfml_upstream_host_valid"))
        );
        assert!(
            decision["satisfied_inputs"]
                .as_array()
                .unwrap()
                .iter()
                .any(|input| input.as_str()
                    == Some("w043_operated_assurance_retained_history_witness_slo_alert_valid"))
        );
        assert!(decision["satisfied_inputs"].as_array().unwrap().iter().any(
            |input| input.as_str()
                == Some(
                    "w043_oxfml_public_migration_formatting_callable_registered_external_valid"
                )
        ));
        assert!(
            !decision["no_promotion_reason_ids"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason.as_str()
                    == Some("pack.grade.direct_oxfml_evaluator_reexecution_absent"))
        );
        assert!(
            decision["no_promotion_reason_ids"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason.as_str()
                    == Some("pack.grade.w043_pack_c5_no_promotion_after_reassessment"))
        );
        assert!(
            decision["no_promotion_reason_ids"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason.as_str()
                    == Some("pack.grade.w043_pack_grade_replay_governance_absent"))
        );
        assert!(
            decision["no_promotion_reason_ids"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason.as_str()
                    == Some("pack.grade.w043_retained_witness_lifecycle_service_absent"))
        );
        assert!(
            decision["no_promotion_reason_ids"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason.as_str()
                    == Some("pack.grade.w043_callable_carrier_sufficiency_proof_absent"))
        );
        assert!(
            decision["no_promotion_reason_ids"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason.as_str()
                    == Some("pack.grade.w043_registered_external_projection_absent"))
        );
        assert!(
            decision["no_promotion_reason_ids"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason.as_str()
                    == Some(
                        "pack.grade.w043_w073_public_request_construction_uptake_not_verified"
                    ))
        );

        let validation = read_required_json(
            &repo_root,
            "docs/test-runs/core-engine/pack-capability/w043-pack-capability-test/replay-appliance/validation/bundle_validation.json",
        );
        assert_eq!(validation["status"], "bundle_valid");

        fs::remove_dir_all(repo_root.parent().unwrap()).unwrap();
    }

    fn unique_temp_repo() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let counter = TEST_REPO_COUNTER.fetch_add(1, Ordering::Relaxed);
        let base = std::env::temp_dir().join(format!(
            "oxcalc-pack-capability-test-{}-{nanos}-{counter}",
            std::process::id()
        ));
        let repo_root = base.join("OxCalc");
        fs::create_dir_all(&repo_root).unwrap();
        repo_root
    }

    fn create_source_artifacts(repo_root: &Path) {
        write_json_test(
            repo_root,
            &retained_validation_path("pack_grade_decision.json"),
            json!({
                "decision_status": "capability_not_promoted",
                "highest_honest_capability": "cap.C4.distill_valid",
            }),
        );
        write_json_test(
            repo_root,
            &retained_validation_path("program_grade_decision.json"),
            json!({
                "decision_status": "capability_not_promoted",
            }),
        );
        write_json_test(
            repo_root,
            "docs/test-runs/core-engine/oxfml-fixture-bridge/post-w033-direct-oxfml-fixture-bridge-001/run_summary.json",
            json!({
                "fixture_case_count": 45,
                "mismatch_count": 0,
                "handoff_triggered": false,
            }),
        );
        write_json_test(
            repo_root,
            "docs/test-runs/core-engine/oxfml-fixture-bridge/post-w033-direct-oxfml-fixture-bridge-001/replay-appliance/validation/bundle_validation.json",
            json!({
                "status": "bundle_valid",
            }),
        );
        write_json_test(
            repo_root,
            "docs/test-runs/core-engine/tracecalc-reference-machine/post-w033-let-lambda-carrier-witness-001/replay-appliance/validation/bundle_validation.json",
            json!({
                "status": "bundle_valid",
            }),
        );
        write_json_test(
            repo_root,
            "docs/test-runs/core-engine/treecalc-local/post-w033-let-lambda-treecalc-witness-001/replay-appliance/validation/bundle_validation.json",
            json!({
                "status": "bundle_valid",
            }),
        );
        write_json_test(
            repo_root,
            "docs/test-runs/core-engine/independent-conformance/post-w033-independent-conformance-001/run_summary.json",
            json!({
                "unexpected_mismatch_count": 0,
                "declared_gap_count": 2,
            }),
        );
        write_json_test(
            repo_root,
            "docs/test-runs/core-engine/independent-conformance/post-w033-independent-conformance-001/replay-appliance/validation/bundle_validation.json",
            json!({
                "status": "bundle_valid",
            }),
        );
        write_json_test(
            repo_root,
            "docs/test-runs/core-engine/treecalc-local/post-w033-independent-conformance-treecalc-001/replay-appliance/adapter_capabilities/oxcalc_treecalc.json",
            json!({
                "claimed_capability_levels": [
                    "cap.C0.ingest_valid",
                    "cap.C1.replay_valid",
                    "cap.C2.diff_valid",
                    "cap.C3.explain_valid"
                ],
                "target_capability_levels": [
                    "cap.C4.distill_valid",
                    "cap.C5.pack_valid"
                ]
            }),
        );
        write_text_test(
            repo_root,
            "docs/spec/core-engine/w033-formalization/W033_PACK_CAPABILITY_BINDING.md",
            "post-W033 pack capability binding\n",
        );
    }

    fn create_w034_source_artifacts(repo_root: &Path) {
        create_source_artifacts(repo_root);
        write_json_test(
            repo_root,
            "docs/test-runs/core-engine/tracecalc-reference-machine/w034-tracecalc-oracle-deepening-001/replay-appliance/validation/bundle_validation.json",
            json!({
                "status": "bundle_valid",
            }),
        );
        write_json_test(
            repo_root,
            "docs/test-runs/core-engine/treecalc-local/w034-independent-conformance-treecalc-001/replay-appliance/validation/bundle_validation.json",
            json!({
                "status": "bundle_valid",
            }),
        );
        write_json_test(
            repo_root,
            "docs/test-runs/core-engine/independent-conformance/w034-independent-conformance-001/run_summary.json",
            json!({
                "unexpected_mismatch_count": 0,
                "declared_gap_count": 6,
            }),
        );
        write_json_test(
            repo_root,
            "docs/test-runs/core-engine/independent-conformance/w034-independent-conformance-001/replay-appliance/validation/bundle_validation.json",
            json!({
                "status": "bundle_valid",
            }),
        );
        write_json_test(
            repo_root,
            "docs/test-runs/core-engine/treecalc-local/w034-independent-conformance-treecalc-001/replay-appliance/adapter_capabilities/oxcalc_treecalc.json",
            json!({
                "claimed_capability_levels": [
                    "cap.C0.ingest_valid",
                    "cap.C1.replay_valid",
                    "cap.C2.diff_valid",
                    "cap.C3.explain_valid"
                ],
                "target_capability_levels": [
                    "cap.C4.distill_valid",
                    "cap.C5.pack_valid"
                ]
            }),
        );
        write_text_test(
            repo_root,
            "docs/spec/core-engine/w034-formalization/W034_PACK_CAPABILITY_AND_CONTINUOUS_SCALE_GATE_BINDING.md",
            "W034 pack capability and continuous scale gate binding\n",
        );
        for artifact in W034_FORMAL_ARTIFACTS
            .iter()
            .chain(W034_FORMATTING_WATCH_ARTIFACTS.iter())
        {
            write_text_test(repo_root, artifact, "W034 gate artifact\n");
        }
    }

    fn create_w035_source_artifacts(repo_root: &Path) {
        create_w034_source_artifacts(repo_root);
        write_json_test(
            repo_root,
            "docs/test-runs/core-engine/tracecalc-reference-machine/w035-tracecalc-oracle-matrix-001/replay-appliance/validation/bundle_validation.json",
            json!({
                "status": "bundle_valid",
            }),
        );
        for artifact in W035_TRACECALC_ORACLE_MATRIX_ARTIFACTS {
            if artifact.ends_with("bundle_validation.json") {
                continue;
            }
            write_json_test(
                repo_root,
                artifact,
                json!({
                    "run_id": W035_TRACECALC_ORACLE_MATRIX_RUN_ID,
                    "status": "valid"
                }),
            );
        }
        for artifact in W035_IMPLEMENTATION_CONFORMANCE_ARTIFACTS {
            write_json_test(
                repo_root,
                artifact,
                json!({
                    "run_id": W035_IMPLEMENTATION_CONFORMANCE_RUN_ID,
                    "status": "implementation_conformance_hardening_valid"
                }),
            );
        }
        for artifact in W035_CONTINUOUS_ASSURANCE_ARTIFACTS {
            write_json_test(
                repo_root,
                artifact,
                json!({
                    "run_id": W035_CONTINUOUS_ASSURANCE_RUN_ID,
                    "decision_status": "continuous_assurance_gate_defined_without_promotion",
                    "status": "bundle_valid"
                }),
            );
        }
        write_text_test(
            repo_root,
            "docs/spec/core-engine/w035-formalization/W035_PACK_CAPABILITY_AND_STAGE2_READINESS_REASSESSMENT.md",
            "W035 pack capability and Stage 2 readiness reassessment\n",
        );
        for artifact in W035_FORMAL_ARTIFACTS
            .iter()
            .chain(W035_FORMATTING_WATCH_ARTIFACTS.iter())
        {
            write_text_test(repo_root, artifact, "W035 gate artifact\n");
        }
    }

    fn create_w036_source_artifacts(repo_root: &Path) {
        create_w035_source_artifacts(repo_root);
        write_json_test(
            repo_root,
            "docs/test-runs/core-engine/tracecalc-reference-machine/w036-tracecalc-coverage-closure-001/replay-appliance/validation/bundle_validation.json",
            json!({
                "status": "bundle_valid",
            }),
        );
        write_json_test(
            repo_root,
            "docs/test-runs/core-engine/independent-conformance/w036-independent-diversity-differential-001/run_summary.json",
            json!({
                "unexpected_mismatch_count": 0,
                "declared_gap_count": 6,
            }),
        );
        write_json_test(
            repo_root,
            "docs/test-runs/core-engine/independent-conformance/w036-independent-diversity-differential-001/replay-appliance/validation/bundle_validation.json",
            json!({
                "status": "bundle_valid",
            }),
        );
        for artifact in W036_TRACECALC_COVERAGE_ARTIFACTS
            .iter()
            .chain(W036_IMPLEMENTATION_CONFORMANCE_ARTIFACTS.iter())
            .chain(W036_TLA_STAGE2_ARTIFACTS.iter())
            .chain(W036_DIFFERENTIAL_ARTIFACTS.iter())
            .chain(W036_CONTINUOUS_ASSURANCE_ARTIFACTS.iter())
        {
            write_json_test(
                repo_root,
                artifact,
                json!({
                    "run_id": "w036-pack-test-source",
                    "status": "bundle_valid"
                }),
            );
        }
        write_json_test(
            repo_root,
            "docs/test-runs/core-engine/independent-conformance/w036-independent-diversity-differential-001/run_summary.json",
            json!({
                "unexpected_mismatch_count": 0,
                "declared_gap_count": 6,
            }),
        );
        write_json_test(
            repo_root,
            "docs/test-runs/core-engine/independent-conformance/w036-independent-diversity-differential-001/replay-appliance/validation/bundle_validation.json",
            json!({
                "status": "bundle_valid",
            }),
        );
        for artifact in W036_FORMAL_ARTIFACTS
            .iter()
            .chain(W036_FORMATTING_WATCH_ARTIFACTS.iter())
        {
            write_text_test(repo_root, artifact, "W036 gate artifact\n");
        }
        write_text_test(
            repo_root,
            "docs/spec/core-engine/w036-formalization/W036_PACK_GRADE_REPLAY_AND_CAPABILITY_PROMOTION_GATE_REASSESSMENT.md",
            "W036 pack-grade replay and capability promotion gate reassessment\n",
        );
    }

    fn create_w037_source_artifacts(repo_root: &Path) {
        create_w036_source_artifacts(repo_root);
        write_json_test(
            repo_root,
            "docs/test-runs/core-engine/tracecalc-reference-machine/w037-tracecalc-observable-closure-001/replay-appliance/validation/bundle_validation.json",
            json!({
                "status": "bundle_valid",
            }),
        );
        write_json_test(
            repo_root,
            "docs/test-runs/core-engine/treecalc-local/w037-optimized-core-conformance-treecalc-001/replay-appliance/validation/bundle_validation.json",
            json!({
                "status": "bundle_valid",
            }),
        );
        write_json_test(
            repo_root,
            "docs/test-runs/core-engine/treecalc-local/w037-optimized-core-conformance-treecalc-001/replay-appliance/adapter_capabilities/oxcalc_treecalc.json",
            json!({
                "claimed_capability_levels": [
                    "cap.C0.ingest_valid",
                    "cap.C1.replay_valid",
                    "cap.C2.diff_valid",
                    "cap.C3.explain_valid",
                    "cap.C4.distill_valid"
                ],
                "target_capability_levels": [
                    "cap.C5.pack_valid"
                ]
            }),
        );
        for artifact in W037_TRACECALC_OBSERVABLE_ARTIFACTS
            .iter()
            .chain(W037_IMPLEMENTATION_CONFORMANCE_ARTIFACTS.iter())
            .chain(W037_DIRECT_OXFML_ARTIFACTS.iter())
            .chain(W037_FORMAL_INVENTORY_ARTIFACTS.iter())
            .chain(W037_STAGE2_CRITERIA_ARTIFACTS.iter())
            .chain(W037_CONTINUOUS_ASSURANCE_ARTIFACTS.iter())
        {
            write_json_test(
                repo_root,
                artifact,
                json!({
                    "run_id": "w037-pack-test-source",
                    "status": "bundle_valid"
                }),
            );
        }
        for artifact in W037_FORMAL_ARTIFACTS
            .iter()
            .chain(W037_FORMATTING_WATCH_ARTIFACTS.iter())
        {
            write_text_test(repo_root, artifact, "W037 gate artifact\n");
        }
        write_text_test(
            repo_root,
            "docs/spec/core-engine/w037-formalization/W037_PACK_GRADE_REPLAY_GOVERNANCE_AND_C5_CANDIDATE_DECISION.md",
            "W037 pack-grade replay governance and C5 candidate decision\n",
        );
    }

    fn create_w038_source_artifacts(repo_root: &Path) {
        create_w037_source_artifacts(repo_root);
        for artifact in W038_TRACECALC_AUTHORITY_ARTIFACTS
            .iter()
            .chain(W038_IMPLEMENTATION_CONFORMANCE_ARTIFACTS.iter())
            .chain(W038_FORMAL_ASSURANCE_ARTIFACTS.iter())
            .chain(W038_STAGE2_REPLAY_ARTIFACTS.iter())
            .chain(W038_OPERATED_ASSURANCE_ARTIFACTS.iter())
            .chain(W038_DIVERSITY_SEAM_ARTIFACTS.iter())
        {
            write_json_test(
                repo_root,
                artifact,
                json!({
                    "run_id": "w038-pack-test-source",
                    "status": "bundle_valid"
                }),
            );
        }
        for artifact in W038_FORMAL_ARTIFACTS
            .iter()
            .chain(W038_FORMATTING_WATCH_ARTIFACTS.iter())
        {
            write_text_test(repo_root, artifact, "W038 gate artifact\n");
        }
        write_text_test(
            repo_root,
            "docs/spec/core-engine/w038-formalization/W038_PACK_GRADE_REPLAY_GOVERNANCE_AND_C5_RELEASE_DECISION.md",
            "W038 pack-grade replay governance and C5 release decision\n",
        );
    }

    fn create_w039_source_artifacts(repo_root: &Path) {
        create_w038_source_artifacts(repo_root);
        for artifact in W039_RELEASE_LEDGER_ARTIFACTS
            .iter()
            .chain(W039_IMPLEMENTATION_CONFORMANCE_ARTIFACTS.iter())
            .chain(W039_FORMAL_ASSURANCE_ARTIFACTS.iter())
            .chain(W039_STAGE2_REPLAY_ARTIFACTS.iter())
            .chain(W039_OPERATED_ASSURANCE_ARTIFACTS.iter())
            .chain(W039_DIVERSITY_SEAM_ARTIFACTS.iter())
            .chain(W039_OXFML_SEAM_ARTIFACTS.iter())
            .chain(W039_UPSTREAM_HOST_ARTIFACTS.iter())
        {
            write_json_test(
                repo_root,
                artifact,
                json!({
                    "run_id": "w039-pack-test-source",
                    "status": "bundle_valid"
                }),
            );
        }
        for artifact in W039_FORMAL_ARTIFACTS
            .iter()
            .chain(W039_FORMATTING_WATCH_ARTIFACTS.iter())
        {
            if artifact.ends_with(".json") {
                write_json_test(
                    repo_root,
                    artifact,
                    json!({
                        "run_id": "w039-pack-test-source",
                        "status": "bundle_valid"
                    }),
                );
            } else {
                write_text_test(repo_root, artifact, "W039 gate artifact\n");
            }
        }
        write_text_test(
            repo_root,
            "docs/spec/core-engine/w039-formalization/W039_PACK_GRADE_REPLAY_GOVERNANCE_AND_C5_REASSESSMENT.md",
            "W039 pack-grade replay governance and C5 reassessment\n",
        );
    }

    fn create_w040_source_artifacts(repo_root: &Path) {
        create_w039_source_artifacts(repo_root);
        write_json_test(
            repo_root,
            "docs/test-runs/core-engine/treecalc-local/w040-optimized-core-dynamic-release-reclassification-001/replay-appliance/validation/bundle_validation.json",
            json!({
                "status": "bundle_valid",
            }),
        );
        write_json_test(
            repo_root,
            "docs/test-runs/core-engine/treecalc-local/w040-optimized-core-dynamic-release-reclassification-001/replay-appliance/adapter_capabilities/oxcalc_treecalc.json",
            json!({
                "claimed_capability_levels": [
                    "cap.C0.ingest_valid",
                    "cap.C1.replay_valid",
                    "cap.C2.diff_valid",
                    "cap.C3.explain_valid"
                ],
                "target_capability_levels": [
                    "cap.C4.distill_valid",
                    "cap.C5.pack_valid"
                ]
            }),
        );
        for artifact in W040_RELEASE_LEDGER_ARTIFACTS
            .iter()
            .chain(W040_IMPLEMENTATION_CONFORMANCE_ARTIFACTS.iter())
            .chain(W040_RUST_TOTALITY_ARTIFACTS.iter())
            .chain(W040_LEAN_TLA_ARTIFACTS.iter())
            .chain(W040_STAGE2_REPLAY_ARTIFACTS.iter())
            .chain(W040_OPERATED_ASSURANCE_ARTIFACTS.iter())
            .chain(W040_DIVERSITY_SEAM_ARTIFACTS.iter())
            .chain(W040_OXFML_SEAM_ARTIFACTS.iter())
            .chain(W040_UPSTREAM_HOST_ARTIFACTS.iter())
        {
            write_json_test(
                repo_root,
                artifact,
                json!({
                    "run_id": "w040-pack-test-source",
                    "status": "bundle_valid"
                }),
            );
        }
        for artifact in W040_FORMAL_ARTIFACTS
            .iter()
            .chain(W040_FORMATTING_WATCH_ARTIFACTS.iter())
        {
            if artifact.ends_with(".json") {
                write_json_test(
                    repo_root,
                    artifact,
                    json!({
                        "run_id": "w040-pack-test-source",
                        "status": "bundle_valid"
                    }),
                );
            } else {
                write_text_test(repo_root, artifact, "W040 gate artifact\n");
            }
        }
        write_text_test(
            repo_root,
            "docs/spec/core-engine/w040-formalization/W040_PACK_GRADE_REPLAY_GOVERNANCE_AND_C5_PROMOTION_DECISION.md",
            "W040 pack-grade replay governance and C5 promotion decision\n",
        );
    }

    fn create_w041_source_artifacts(repo_root: &Path) {
        create_w040_source_artifacts(repo_root);
        write_json_test(
            repo_root,
            "docs/test-runs/core-engine/treecalc-local/w041-optimized-core-automatic-dynamic-transition-001/replay-appliance/validation/bundle_validation.json",
            json!({
                "status": "bundle_valid",
            }),
        );
        write_json_test(
            repo_root,
            "docs/test-runs/core-engine/treecalc-local/w041-optimized-core-automatic-dynamic-transition-001/replay-appliance/adapter_capabilities/oxcalc_treecalc.json",
            json!({
                "claimed_capability_levels": [
                    "cap.C0.ingest_valid",
                    "cap.C1.replay_valid",
                    "cap.C2.diff_valid",
                    "cap.C3.explain_valid"
                ],
                "target_capability_levels": [
                    "cap.C4.distill_valid",
                    "cap.C5.pack_valid"
                ]
            }),
        );
        for artifact in W041_RELEASE_LEDGER_ARTIFACTS
            .iter()
            .chain(W041_IMPLEMENTATION_CONFORMANCE_ARTIFACTS.iter())
            .chain(W041_RUST_TOTALITY_ARTIFACTS.iter())
            .chain(W041_LEAN_TLA_ARTIFACTS.iter())
            .chain(W041_STAGE2_REPLAY_ARTIFACTS.iter())
            .chain(W041_OPERATED_ASSURANCE_ARTIFACTS.iter())
            .chain(W041_DIVERSITY_SEAM_ARTIFACTS.iter())
            .chain(W041_OXFML_SEAM_ARTIFACTS.iter())
            .chain(W041_UPSTREAM_HOST_ARTIFACTS.iter())
        {
            write_json_test(
                repo_root,
                artifact,
                json!({
                    "run_id": "w041-pack-test-source",
                    "status": "bundle_valid"
                }),
            );
        }
        for artifact in W041_FORMAL_ARTIFACTS
            .iter()
            .chain(W041_FORMATTING_WATCH_ARTIFACTS.iter())
        {
            if artifact.ends_with(".json") {
                write_json_test(
                    repo_root,
                    artifact,
                    json!({
                        "run_id": "w041-pack-test-source",
                        "status": "bundle_valid"
                    }),
                );
            } else {
                write_text_test(repo_root, artifact, "W041 gate artifact\n");
            }
        }
        write_text_test(
            repo_root,
            "docs/spec/core-engine/w041-formalization/W041_PACK_GRADE_REPLAY_GOVERNANCE_AND_C5_REASSESSMENT.md",
            "W041 pack-grade replay governance and C5 reassessment\n",
        );
    }

    fn create_w042_source_artifacts(repo_root: &Path) {
        create_w041_source_artifacts(repo_root);
        write_json_test(
            repo_root,
            "docs/test-runs/core-engine/treecalc-local/w042-optimized-core-counterpart-conformance-treecalc-001/replay-appliance/validation/bundle_validation.json",
            json!({
                "status": "bundle_valid",
            }),
        );
        write_json_test(
            repo_root,
            "docs/test-runs/core-engine/treecalc-local/w042-optimized-core-counterpart-conformance-treecalc-001/replay-appliance/adapter_capabilities/oxcalc_treecalc.json",
            json!({
                "claimed_capability_levels": [
                    "cap.C0.ingest_valid",
                    "cap.C1.replay_valid",
                    "cap.C2.diff_valid",
                    "cap.C3.explain_valid"
                ],
                "target_capability_levels": [
                    "cap.C4.distill_valid",
                    "cap.C5.pack_valid"
                ]
            }),
        );
        for artifact in W042_RELEASE_LEDGER_ARTIFACTS
            .iter()
            .chain(W042_IMPLEMENTATION_CONFORMANCE_ARTIFACTS.iter())
            .chain(W042_RUST_TOTALITY_ARTIFACTS.iter())
            .chain(W042_LEAN_TLA_ARTIFACTS.iter())
            .chain(W042_STAGE2_REPLAY_ARTIFACTS.iter())
            .chain(W042_OPERATED_ASSURANCE_ARTIFACTS.iter())
            .chain(W042_DIVERSITY_SEAM_ARTIFACTS.iter())
            .chain(W042_OXFML_SEAM_ARTIFACTS.iter())
            .chain(W042_UPSTREAM_HOST_ARTIFACTS.iter())
        {
            write_json_test(
                repo_root,
                artifact,
                json!({
                    "run_id": "w042-pack-test-source",
                    "status": "bundle_valid"
                }),
            );
        }
        for artifact in W042_FORMAL_ARTIFACTS
            .iter()
            .chain(W042_FORMATTING_WATCH_ARTIFACTS.iter())
        {
            if artifact.ends_with(".json") {
                write_json_test(
                    repo_root,
                    artifact,
                    json!({
                        "run_id": "w042-pack-test-source",
                        "status": "bundle_valid"
                    }),
                );
            } else {
                write_text_test(repo_root, artifact, "W042 gate artifact\n");
            }
        }
        write_text_test(
            repo_root,
            "docs/spec/core-engine/w042-formalization/W042_PACK_GRADE_REPLAY_GOVERNANCE_AND_C5_REASSESSMENT.md",
            "W042 pack-grade replay governance and C5 reassessment\n",
        );
    }

    fn create_w043_source_artifacts(repo_root: &Path) {
        create_w042_source_artifacts(repo_root);
        write_json_test(
            repo_root,
            "docs/test-runs/core-engine/treecalc-local/w043-optimized-core-broad-conformance-treecalc-001/replay-appliance/validation/bundle_validation.json",
            json!({
                "status": "bundle_valid",
            }),
        );
        write_json_test(
            repo_root,
            "docs/test-runs/core-engine/treecalc-local/w043-optimized-core-broad-conformance-treecalc-001/replay-appliance/adapter_capabilities/oxcalc_treecalc.json",
            json!({
                "claimed_capability_levels": [
                    "cap.C0.ingest_valid",
                    "cap.C1.replay_valid",
                    "cap.C2.diff_valid",
                    "cap.C3.explain_valid"
                ],
                "target_capability_levels": [
                    "cap.C4.distill_valid",
                    "cap.C5.pack_valid"
                ]
            }),
        );
        for artifact in W043_RELEASE_LEDGER_ARTIFACTS
            .iter()
            .chain(W043_IMPLEMENTATION_CONFORMANCE_ARTIFACTS.iter())
            .chain(W043_RUST_TOTALITY_ARTIFACTS.iter())
            .chain(W043_LEAN_TLA_ARTIFACTS.iter())
            .chain(W043_STAGE2_REPLAY_ARTIFACTS.iter())
            .chain(W043_OPERATED_ASSURANCE_ARTIFACTS.iter())
            .chain(W043_DIVERSITY_SEAM_ARTIFACTS.iter())
            .chain(W043_OXFML_SEAM_ARTIFACTS.iter())
            .chain(W043_UPSTREAM_HOST_ARTIFACTS.iter())
        {
            write_json_test(
                repo_root,
                artifact,
                json!({
                    "run_id": "w043-pack-test-source",
                    "status": "bundle_valid"
                }),
            );
        }
        for artifact in W043_FORMAL_ARTIFACTS
            .iter()
            .chain(W043_FORMATTING_WATCH_ARTIFACTS.iter())
        {
            if artifact.ends_with(".json") {
                write_json_test(
                    repo_root,
                    artifact,
                    json!({
                        "run_id": "w043-pack-test-source",
                        "status": "bundle_valid"
                    }),
                );
            } else {
                write_text_test(repo_root, artifact, "W043 gate artifact\n");
            }
        }
        write_text_test(
            repo_root,
            "docs/spec/core-engine/w043-formalization/W043_PACK_GRADE_REPLAY_GOVERNANCE_AND_C5_RELEASE_REASSESSMENT.md",
            "W043 pack-grade replay governance and C5 release reassessment\n",
        );
    }

    fn write_json_test(repo_root: &Path, relative_path: &str, value: Value) {
        let path = repo_root.join(relative_path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, serde_json::to_string_pretty(&value).unwrap() + "\n").unwrap();
    }

    fn write_text_test(repo_root: &Path, relative_path: &str, value: &str) {
        let path = repo_root.join(relative_path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, value).unwrap();
    }

    fn read_required_json(repo_root: &Path, relative_path: &str) -> Value {
        serde_json::from_str(&fs::read_to_string(repo_root.join(relative_path)).unwrap()).unwrap()
    }
}
