#![forbid(unsafe_code)]

//! Post-W033/W034/W035 pack capability decision packet emission.

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
        if !profile.profile_id.starts_with("w037_") {
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
    if run_id.starts_with("w037-") {
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
