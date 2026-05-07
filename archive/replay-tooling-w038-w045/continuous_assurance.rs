#![forbid(unsafe_code)]

//! W035 continuous assurance and cross-engine differential gate packet emission.

use std::fs;
use std::path::Path;

use serde_json::{Value, json};
use thiserror::Error;

const RUN_SUMMARY_SCHEMA_V1: &str = "oxcalc.continuous_assurance.run_summary.v1";
const EVIDENCE_INDEX_SCHEMA_V1: &str = "oxcalc.continuous_assurance.evidence_index.v1";
const SCHEDULE_SCHEMA_V1: &str = "oxcalc.continuous_assurance.schedule.v1";
const DIFFERENTIAL_GATE_SCHEMA_V1: &str = "oxcalc.continuous_assurance.cross_engine_gate.v1";
const DECISION_SCHEMA_V1: &str = "oxcalc.continuous_assurance.decision.v1";
const BUNDLE_MANIFEST_SCHEMA_V1: &str = "oxcalc.continuous_assurance.bundle_manifest.v1";
const BUNDLE_VALIDATION_SCHEMA_V1: &str = "oxcalc.continuous_assurance.bundle_validation.v1";
const HISTORY_WINDOW_SCHEMA_V1: &str = "oxcalc.continuous_assurance.history_window.v1";
const REGRESSION_THRESHOLDS_SCHEMA_V1: &str =
    "oxcalc.continuous_assurance.regression_thresholds.v1";
const QUARANTINE_POLICY_SCHEMA_V1: &str = "oxcalc.continuous_assurance.quarantine_policy.v1";
const SIMULATED_MULTI_RUN_SCHEMA_V1: &str = "oxcalc.continuous_assurance.simulated_multi_run.v1";
const SERVICE_READINESS_SCHEMA_V1: &str = "oxcalc.continuous_assurance.service_readiness.v1";
const CROSS_ENGINE_SERVICE_PILOT_SCHEMA_V1: &str =
    "oxcalc.continuous_assurance.cross_engine_service_pilot.v1";

const W034_SCALE_BINDING_RUN_ID: &str = "w034-continuous-scale-gate-binding-001";
const W034_PACK_RUN_ID: &str = "w034-pack-capability-gate-binding-001";
const W035_ORACLE_MATRIX_RUN_ID: &str = "w035-tracecalc-oracle-matrix-001";
const W035_IMPLEMENTATION_CONFORMANCE_RUN_ID: &str =
    "w035-implementation-conformance-hardening-001";
const W035_CONTINUOUS_ASSURANCE_RUN_ID: &str = "w035-continuous-assurance-gate-001";
const W036_TRACECALC_COVERAGE_RUN_ID: &str = "w036-tracecalc-coverage-closure-001";
const W036_IMPLEMENTATION_CONFORMANCE_RUN_ID: &str = "w036-implementation-conformance-closure-001";
const W036_TLA_STAGE2_RUN_ID: &str = "w036-stage2-partition-001";
const W036_INDEPENDENT_DIFFERENTIAL_RUN_ID: &str = "w036-independent-diversity-differential-001";
const W037_TRACECALC_OBSERVABLE_RUN_ID: &str = "w037-tracecalc-observable-closure-001";
const W037_IMPLEMENTATION_CONFORMANCE_RUN_ID: &str = "w037-implementation-conformance-closure-001";
const W037_DIRECT_OXFML_RUN_ID: &str = "w037-direct-oxfml-evaluator-001";
const W037_FORMAL_INVENTORY_RUN_ID: &str = "w037-proof-model-closure-001";
const W037_STAGE2_CRITERIA_RUN_ID: &str = "w037-stage2-deterministic-replay-criteria-001";

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

#[derive(Debug, Error)]
pub enum ContinuousAssuranceError {
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
pub struct ContinuousAssuranceRunSummary {
    pub run_id: String,
    pub schema_version: String,
    pub decision_status: String,
    pub source_evidence_row_count: usize,
    pub scheduled_lane_count: usize,
    pub cross_engine_gate_row_count: usize,
    pub missing_artifact_count: usize,
    pub unexpected_mismatch_count: usize,
    pub no_promotion_reason_count: usize,
    pub history_window_row_count: usize,
    pub regression_threshold_count: usize,
    pub quarantine_rule_count: usize,
    pub simulated_multi_run_count: usize,
    pub service_readiness_criteria_count: usize,
    pub service_readiness_blocked_count: usize,
    pub cross_engine_service_pilot_present: bool,
    pub continuous_service_promoted: bool,
    pub artifact_root: String,
}

#[derive(Debug, Clone, Default)]
pub struct ContinuousAssuranceRunner;

#[derive(Debug, Clone, Default)]
struct Evaluation {
    source_rows: Vec<Value>,
    cross_engine_rows: Vec<Value>,
    missing_artifacts: Vec<String>,
    unexpected_mismatches: Vec<String>,
    no_promotion_reasons: Vec<String>,
}

#[derive(Debug, Clone)]
struct OperationArtifacts {
    history_window: Value,
    regression_thresholds: Value,
    quarantine_policy: Value,
    simulated_multi_run: Value,
    service_readiness: Option<Value>,
    cross_engine_service_pilot: Option<Value>,
}

impl ContinuousAssuranceRunner {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    pub fn execute(
        &self,
        repo_root: &Path,
        run_id: &str,
    ) -> Result<ContinuousAssuranceRunSummary, ContinuousAssuranceError> {
        let artifact_root = repo_root.join(format!(
            "docs/test-runs/core-engine/continuous-assurance/{run_id}"
        ));
        let relative_artifact_root = relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "continuous-assurance",
            run_id,
        ]);

        if artifact_root.exists() {
            fs::remove_dir_all(&artifact_root).map_err(|source| {
                ContinuousAssuranceError::RemoveDirectory {
                    path: artifact_root.display().to_string(),
                    source,
                }
            })?;
        }

        create_directory(&artifact_root)?;
        create_directory(&artifact_root.join("decision"))?;
        create_directory(&artifact_root.join("differentials"))?;
        create_directory(&artifact_root.join("evidence"))?;
        if is_operation_history_run(run_id) {
            create_directory(&artifact_root.join("alerts"))?;
            create_directory(&artifact_root.join("history"))?;
            create_directory(&artifact_root.join("operation"))?;
            create_directory(&artifact_root.join("thresholds"))?;
        }
        if is_w037_run(run_id) {
            create_directory(&artifact_root.join("service"))?;
        }
        create_directory(&artifact_root.join("schedule"))?;
        create_directory(&artifact_root.join("replay-appliance"))?;
        create_directory(&artifact_root.join("replay-appliance/validation"))?;

        let evaluation = evaluate(repo_root, run_id)?;
        let operation_artifacts = if is_operation_history_run(run_id) {
            Some(operation_artifacts(
                run_id,
                &relative_artifact_root,
                &evaluation,
            ))
        } else {
            None
        };
        let schedule = continuous_assurance_schedule(run_id, &relative_artifact_root);
        let decision = decision_packet(run_id, &evaluation, operation_artifacts.as_ref());

        write_json(
            &artifact_root.join("evidence/source_evidence_index.json"),
            &json!({
                "schema_version": EVIDENCE_INDEX_SCHEMA_V1,
                "run_id": run_id,
                "artifact_root": relative_artifact_root,
                "row_count": evaluation.source_rows.len(),
                "missing_artifact_count": evaluation.missing_artifacts.len(),
                "unexpected_mismatch_count": evaluation.unexpected_mismatches.len(),
                "rows": evaluation.source_rows,
            }),
        )?;
        write_json(
            &artifact_root.join("schedule/continuous_assurance_schedule.json"),
            &schedule,
        )?;
        write_json(
            &artifact_root.join("differentials/cross_engine_differential_gate.json"),
            &json!({
                "schema_version": DIFFERENTIAL_GATE_SCHEMA_V1,
                "run_id": run_id,
                "row_count": evaluation.cross_engine_rows.len(),
                "unexpected_mismatch_count": count_failure_rows(&evaluation.cross_engine_rows),
                "continuous_service_present": false,
                "rows": evaluation.cross_engine_rows,
            }),
        )?;
        write_json(
            &artifact_root.join("decision/continuous_assurance_decision.json"),
            &decision,
        )?;
        if let Some(operation_artifacts) = &operation_artifacts {
            write_json(
                &artifact_root.join("history/assurance_history_window.json"),
                &operation_artifacts.history_window,
            )?;
            write_json(
                &artifact_root.join("thresholds/regression_thresholds.json"),
                &operation_artifacts.regression_thresholds,
            )?;
            write_json(
                &artifact_root.join("alerts/quarantine_policy.json"),
                &operation_artifacts.quarantine_policy,
            )?;
            write_json(
                &artifact_root.join("operation/simulated_multi_run_evidence.json"),
                &operation_artifacts.simulated_multi_run,
            )?;
            if let Some(service_readiness) = &operation_artifacts.service_readiness {
                write_json(
                    &artifact_root.join("service/service_readiness.json"),
                    service_readiness,
                )?;
            }
            if let Some(cross_engine_service_pilot) =
                &operation_artifacts.cross_engine_service_pilot
            {
                write_json(
                    &artifact_root.join("service/cross_engine_service_pilot.json"),
                    cross_engine_service_pilot,
                )?;
            }
        }

        let required_artifacts = required_artifacts(run_id);
        write_json(
            &artifact_root.join("replay-appliance/bundle_manifest.json"),
            &json!({
                "schema_version": BUNDLE_MANIFEST_SCHEMA_V1,
                "run_id": run_id,
                "artifact_root": relative_artifact_root,
                "claimed_capability": if is_w037_run(run_id) {
                    "operated_assurance_service_readiness_packet"
                } else if is_w036_run(run_id) {
                    "simulated_continuous_assurance_history_packet"
                } else {
                    "continuous_assurance_gate_packet"
                },
                "excluded_capabilities": [
                    "operated_continuous_assurance_service",
                    "continuous_scale_assurance",
                    "continuous_cross_engine_diff_service",
                    "cap.C5.pack_valid",
                    "stage2_scheduler_promotion"
                ],
                "required_artifacts": required_artifacts,
            }),
        )?;

        let summary = ContinuousAssuranceRunSummary {
            run_id: run_id.to_string(),
            schema_version: RUN_SUMMARY_SCHEMA_V1.to_string(),
            decision_status: text_at(&decision, "decision_status"),
            source_evidence_row_count: evaluation.source_rows.len(),
            scheduled_lane_count: schedule
                .get("lanes")
                .and_then(Value::as_array)
                .map_or(0, Vec::len),
            cross_engine_gate_row_count: evaluation.cross_engine_rows.len(),
            missing_artifact_count: evaluation.missing_artifacts.len(),
            unexpected_mismatch_count: evaluation.unexpected_mismatches.len(),
            no_promotion_reason_count: evaluation.no_promotion_reasons.len(),
            history_window_row_count: operation_artifacts.as_ref().map_or(0, |artifacts| {
                number_at(&artifacts.history_window, "row_count") as usize
            }),
            regression_threshold_count: operation_artifacts.as_ref().map_or(0, |artifacts| {
                number_at(&artifacts.regression_thresholds, "rule_count") as usize
            }),
            quarantine_rule_count: operation_artifacts.as_ref().map_or(0, |artifacts| {
                number_at(&artifacts.quarantine_policy, "rule_count") as usize
            }),
            simulated_multi_run_count: operation_artifacts.as_ref().map_or(0, |artifacts| {
                number_at(&artifacts.simulated_multi_run, "row_count") as usize
            }),
            service_readiness_criteria_count: operation_artifacts
                .as_ref()
                .and_then(|artifacts| artifacts.service_readiness.as_ref())
                .map_or(0, |value| number_at(value, "criteria_count") as usize),
            service_readiness_blocked_count: operation_artifacts
                .as_ref()
                .and_then(|artifacts| artifacts.service_readiness.as_ref())
                .map_or(0, |value| {
                    number_at(value, "blocked_criteria_count") as usize
                }),
            cross_engine_service_pilot_present: operation_artifacts
                .as_ref()
                .is_some_and(|artifacts| artifacts.cross_engine_service_pilot.is_some()),
            continuous_service_promoted: false,
            artifact_root: relative_artifact_root.clone(),
        };

        write_json(
            &artifact_root.join("run_summary.json"),
            &json!({
                "schema_version": summary.schema_version,
                "run_id": summary.run_id,
                "decision_status": summary.decision_status,
                "source_evidence_row_count": summary.source_evidence_row_count,
                "scheduled_lane_count": summary.scheduled_lane_count,
                "cross_engine_gate_row_count": summary.cross_engine_gate_row_count,
                "missing_artifact_count": summary.missing_artifact_count,
                "unexpected_mismatch_count": summary.unexpected_mismatch_count,
                "no_promotion_reason_count": summary.no_promotion_reason_count,
                "history_window_row_count": summary.history_window_row_count,
                "regression_threshold_count": summary.regression_threshold_count,
                "quarantine_rule_count": summary.quarantine_rule_count,
                "simulated_multi_run_count": summary.simulated_multi_run_count,
                "service_readiness_criteria_count": summary.service_readiness_criteria_count,
                "service_readiness_blocked_count": summary.service_readiness_blocked_count,
                "cross_engine_service_pilot_present": summary.cross_engine_service_pilot_present,
                "continuous_service_promoted": summary.continuous_service_promoted,
                "artifact_root": summary.artifact_root,
                "source_evidence_index_path": format!("{relative_artifact_root}/evidence/source_evidence_index.json"),
                "schedule_path": format!("{relative_artifact_root}/schedule/continuous_assurance_schedule.json"),
                "cross_engine_differential_gate_path": format!("{relative_artifact_root}/differentials/cross_engine_differential_gate.json"),
                "decision_path": format!("{relative_artifact_root}/decision/continuous_assurance_decision.json"),
                "history_window_path": if is_operation_history_run(run_id) { Value::String(format!("{relative_artifact_root}/history/assurance_history_window.json")) } else { Value::Null },
                "regression_thresholds_path": if is_operation_history_run(run_id) { Value::String(format!("{relative_artifact_root}/thresholds/regression_thresholds.json")) } else { Value::Null },
                "quarantine_policy_path": if is_operation_history_run(run_id) { Value::String(format!("{relative_artifact_root}/alerts/quarantine_policy.json")) } else { Value::Null },
                "simulated_multi_run_path": if is_operation_history_run(run_id) { Value::String(format!("{relative_artifact_root}/operation/simulated_multi_run_evidence.json")) } else { Value::Null },
                "service_readiness_path": if is_w037_run(run_id) { Value::String(format!("{relative_artifact_root}/service/service_readiness.json")) } else { Value::Null },
                "cross_engine_service_pilot_path": if is_w037_run(run_id) { Value::String(format!("{relative_artifact_root}/service/cross_engine_service_pilot.json")) } else { Value::Null },
                "bundle_validation_path": format!("{relative_artifact_root}/replay-appliance/validation/bundle_validation.json"),
            }),
        )?;

        let validation_path =
            artifact_root.join("replay-appliance/validation/bundle_validation.json");
        write_json(
            &validation_path,
            &json!({
                "schema_version": BUNDLE_VALIDATION_SCHEMA_V1,
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
                "schema_version": BUNDLE_VALIDATION_SCHEMA_V1,
                "run_id": run_id,
                "status": if missing_required_paths.is_empty() { "bundle_valid" } else { "missing_required_artifacts" },
                "missing_paths": missing_required_paths,
                "validated_required_artifact_count": required_artifacts.len(),
                "source_missing_artifact_count": evaluation.missing_artifacts.len(),
                "unexpected_mismatch_count": evaluation.unexpected_mismatches.len(),
                "history_window_row_count": summary.history_window_row_count,
                "regression_threshold_count": summary.regression_threshold_count,
                "quarantine_rule_count": summary.quarantine_rule_count,
                "simulated_multi_run_count": summary.simulated_multi_run_count,
                "service_readiness_criteria_count": summary.service_readiness_criteria_count,
                "service_readiness_blocked_count": summary.service_readiness_blocked_count,
                "cross_engine_service_pilot_present": summary.cross_engine_service_pilot_present,
                "continuous_service_promoted": summary.continuous_service_promoted,
                "decision_status": summary.decision_status,
            }),
        )?;

        Ok(summary)
    }
}

fn evaluate(repo_root: &Path, run_id: &str) -> Result<Evaluation, ContinuousAssuranceError> {
    let mut evaluation = Evaluation::default();

    evaluate_scale_binding(repo_root, &mut evaluation)?;
    evaluate_pack_binding(repo_root, &mut evaluation)?;
    evaluate_oracle_matrix(repo_root, &mut evaluation)?;
    evaluate_implementation_conformance(repo_root, &mut evaluation)?;
    evaluate_formal_artifacts(repo_root, &mut evaluation);

    if is_operation_history_run(run_id) {
        evaluate_w035_continuous_gate(repo_root, &mut evaluation)?;
        evaluate_w036_tracecalc_coverage(repo_root, &mut evaluation)?;
        evaluate_w036_implementation_conformance(repo_root, &mut evaluation)?;
        evaluate_w036_formal_artifacts(repo_root, &mut evaluation);
        evaluate_w036_tla_stage2_partition(repo_root, &mut evaluation)?;
        evaluate_w036_independent_differential(repo_root, &mut evaluation)?;
    }
    if is_w037_run(run_id) {
        evaluate_w037_tracecalc_observable(repo_root, &mut evaluation)?;
        evaluate_w037_implementation_conformance(repo_root, &mut evaluation)?;
        evaluate_w037_direct_oxfml(repo_root, &mut evaluation)?;
        evaluate_w037_formal_inventory(repo_root, &mut evaluation)?;
        evaluate_w037_stage2_criteria(repo_root, &mut evaluation)?;
    }

    evaluation.cross_engine_rows = cross_engine_rows(run_id);
    for row in &evaluation.cross_engine_rows {
        collect_failures(row, &mut evaluation.unexpected_mismatches);
    }

    evaluation.no_promotion_reasons = no_promotion_reasons(run_id);

    Ok(evaluation)
}

fn evaluate_scale_binding(
    repo_root: &Path,
    evaluation: &mut Evaluation,
) -> Result<(), ContinuousAssuranceError> {
    let summary_path = relative_artifact_path([
        "docs",
        "test-runs",
        "core-engine",
        "metamorphic-scale-semantic-binding",
        W034_SCALE_BINDING_RUN_ID,
        "run_summary.json",
    ]);
    let criteria_path = relative_artifact_path([
        "docs",
        "test-runs",
        "core-engine",
        "metamorphic-scale-semantic-binding",
        W034_SCALE_BINDING_RUN_ID,
        "decision",
        "continuous_scale_assurance_criteria.json",
    ]);
    let summary = read_json(repo_root, &summary_path)?;
    let criteria = read_json(repo_root, &criteria_path)?;
    add_missing_if_absent(evaluation, &summary_path, &summary);
    add_missing_if_absent(evaluation, &criteria_path, &criteria);

    let mut failures = Vec::new();
    if let Some(summary) = &summary {
        if number_at(summary, "missing_artifact_count") != 0 {
            failures.push("w034_scale_binding_missing_artifacts".to_string());
        }
        if number_at(summary, "unexpected_mismatch_count") != 0 {
            failures.push("w034_scale_binding_unexpected_mismatch".to_string());
        }
        if number_at(summary, "validated_scale_run_count") != 7 {
            failures.push("w034_scale_binding_expected_seven_validated_scale_rows".to_string());
        }
    }
    evaluation.unexpected_mismatches.extend(failures.clone());
    evaluation.source_rows.push(json!({
        "input_id": "w034_scale_semantic_binding",
        "artifact_paths": [summary_path, criteria_path],
        "evidence_state": if failures.is_empty() && summary.is_some() && criteria.is_some() {
            "semantic_scale_binding_present_no_continuous_promotion"
        } else {
            "source_gap"
        },
        "validated_scale_run_count": summary.as_ref().map_or(0, |value| number_at(value, "validated_scale_run_count")),
        "scale_signature_row_count": summary.as_ref().map_or(0, |value| number_at(value, "scale_signature_row_count")),
        "continuous_scale_assurance_promoted": criteria.as_ref().is_some_and(|value| bool_at(value, "continuous_scale_assurance_promoted")),
        "failures": failures,
    }));
    Ok(())
}

fn evaluate_pack_binding(
    repo_root: &Path,
    evaluation: &mut Evaluation,
) -> Result<(), ContinuousAssuranceError> {
    let summary_path = relative_artifact_path([
        "docs",
        "test-runs",
        "core-engine",
        "pack-capability",
        W034_PACK_RUN_ID,
        "run_summary.json",
    ]);
    let decision_path = relative_artifact_path([
        "docs",
        "test-runs",
        "core-engine",
        "pack-capability",
        W034_PACK_RUN_ID,
        "decision",
        "pack_capability_decision.json",
    ]);
    let summary = read_json(repo_root, &summary_path)?;
    let decision = read_json(repo_root, &decision_path)?;
    add_missing_if_absent(evaluation, &summary_path, &summary);
    add_missing_if_absent(evaluation, &decision_path, &decision);

    let mut failures = Vec::new();
    if decision
        .as_ref()
        .is_some_and(|value| bool_at(value, "capability_promoted"))
    {
        failures.push("unexpected_pack_capability_promotion".to_string());
    }
    evaluation.unexpected_mismatches.extend(failures.clone());
    evaluation.source_rows.push(json!({
        "input_id": "w034_pack_capability_binding",
        "artifact_paths": [summary_path, decision_path],
        "evidence_state": if failures.is_empty() && summary.is_some() && decision.is_some() {
            "pack_binding_present_c5_not_promoted"
        } else {
            "source_gap"
        },
        "highest_honest_capability": summary.as_ref().map_or("<missing>".to_string(), |value| text_at(value, "highest_honest_capability")),
        "blocker_count": summary.as_ref().map_or(0, |value| number_at(value, "blocker_count")),
        "failures": failures,
    }));
    Ok(())
}

fn evaluate_oracle_matrix(
    repo_root: &Path,
    evaluation: &mut Evaluation,
) -> Result<(), ContinuousAssuranceError> {
    let summary_path = relative_artifact_path([
        "docs",
        "test-runs",
        "core-engine",
        "tracecalc-reference-machine",
        W035_ORACLE_MATRIX_RUN_ID,
        "oracle-matrix",
        "run_summary.json",
    ]);
    let summary = read_json(repo_root, &summary_path)?;
    add_missing_if_absent(evaluation, &summary_path, &summary);
    let mut failures = Vec::new();
    if let Some(summary) = &summary
        && number_at(summary, "missing_or_failed_row_count") != 0
    {
        failures.push("oracle_matrix_has_missing_or_failed_rows".to_string());
    }
    evaluation.unexpected_mismatches.extend(failures.clone());
    evaluation.source_rows.push(json!({
        "input_id": "w035_tracecalc_oracle_matrix",
        "artifact_paths": [summary_path],
        "evidence_state": if failures.is_empty() && summary.is_some() {
            "oracle_matrix_present_with_classified_uncovered_rows"
        } else {
            "source_gap"
        },
        "matrix_row_count": summary.as_ref().map_or(0, |value| number_at(value, "matrix_row_count")),
        "covered_row_count": summary.as_ref().map_or(0, |value| number_at(value, "covered_row_count")),
        "classified_uncovered_row_count": summary.as_ref().map_or(0, |value| number_at(value, "classified_uncovered_row_count")),
        "failures": failures,
    }));
    Ok(())
}

fn evaluate_implementation_conformance(
    repo_root: &Path,
    evaluation: &mut Evaluation,
) -> Result<(), ContinuousAssuranceError> {
    let summary_path = relative_artifact_path([
        "docs",
        "test-runs",
        "core-engine",
        "implementation-conformance",
        W035_IMPLEMENTATION_CONFORMANCE_RUN_ID,
        "run_summary.json",
    ]);
    let summary = read_json(repo_root, &summary_path)?;
    add_missing_if_absent(evaluation, &summary_path, &summary);
    let mut failures = Vec::new();
    if let Some(summary) = &summary
        && number_at(summary, "failed_row_count") != 0
    {
        failures.push("implementation_conformance_has_failed_rows".to_string());
    }
    evaluation.unexpected_mismatches.extend(failures.clone());
    evaluation.source_rows.push(json!({
        "input_id": "w035_implementation_conformance",
        "artifact_paths": [summary_path],
        "evidence_state": if failures.is_empty() && summary.is_some() {
            "gap_dispositions_present_without_promotion"
        } else {
            "source_gap"
        },
        "gap_disposition_row_count": summary.as_ref().map_or(0, |value| number_at(value, "gap_disposition_row_count")),
        "implementation_work_count": summary.as_ref().map_or(0, |value| number_at(value, "implementation_work_count")),
        "spec_evolution_deferral_count": summary.as_ref().map_or(0, |value| number_at(value, "spec_evolution_deferral_count")),
        "failures": failures,
    }));
    Ok(())
}

fn evaluate_formal_artifacts(repo_root: &Path, evaluation: &mut Evaluation) {
    let missing = W035_FORMAL_ARTIFACTS
        .iter()
        .filter(|path| !repo_root.join(path).exists())
        .map(|path| (*path).to_string())
        .collect::<Vec<_>>();
    evaluation.missing_artifacts.extend(missing.clone());
    evaluation.source_rows.push(json!({
        "input_id": "w035_lean_tla_formal_packets",
        "artifact_paths": W035_FORMAL_ARTIFACTS,
        "evidence_state": if missing.is_empty() {
            "bounded_formal_packets_present_no_full_verification_promotion"
        } else {
            "missing_artifact"
        },
        "missing_artifacts": missing,
    }));
}

fn evaluate_w035_continuous_gate(
    repo_root: &Path,
    evaluation: &mut Evaluation,
) -> Result<(), ContinuousAssuranceError> {
    let summary_path = relative_artifact_path([
        "docs",
        "test-runs",
        "core-engine",
        "continuous-assurance",
        W035_CONTINUOUS_ASSURANCE_RUN_ID,
        "run_summary.json",
    ]);
    let decision_path = relative_artifact_path([
        "docs",
        "test-runs",
        "core-engine",
        "continuous-assurance",
        W035_CONTINUOUS_ASSURANCE_RUN_ID,
        "decision",
        "continuous_assurance_decision.json",
    ]);
    let summary = read_json(repo_root, &summary_path)?;
    let decision = read_json(repo_root, &decision_path)?;
    add_missing_if_absent(evaluation, &summary_path, &summary);
    add_missing_if_absent(evaluation, &decision_path, &decision);

    let mut failures = Vec::new();
    if let Some(summary) = &summary {
        if number_at(summary, "missing_artifact_count") != 0 {
            failures.push("w035_continuous_gate_missing_artifacts".to_string());
        }
        if number_at(summary, "unexpected_mismatch_count") != 0 {
            failures.push("w035_continuous_gate_unexpected_mismatch".to_string());
        }
    }
    if decision
        .as_ref()
        .is_some_and(|value| bool_at(value, "cross_engine_differential_service_promoted"))
    {
        failures.push("unexpected_w035_continuous_service_promotion".to_string());
    }
    evaluation.unexpected_mismatches.extend(failures.clone());
    evaluation.source_rows.push(json!({
        "input_id": "w035_continuous_assurance_gate",
        "artifact_paths": [summary_path, decision_path],
        "evidence_state": if failures.is_empty() && summary.is_some() && decision.is_some() {
            "w035_gate_present_without_service_promotion"
        } else {
            "source_gap"
        },
        "source_evidence_row_count": summary.as_ref().map_or(0, |value| number_at(value, "source_evidence_row_count")),
        "scheduled_lane_count": summary.as_ref().map_or(0, |value| number_at(value, "scheduled_lane_count")),
        "cross_engine_gate_row_count": summary.as_ref().map_or(0, |value| number_at(value, "cross_engine_gate_row_count")),
        "missing_artifact_count": summary.as_ref().map_or(0, |value| number_at(value, "missing_artifact_count")),
        "unexpected_mismatch_count": summary.as_ref().map_or(0, |value| number_at(value, "unexpected_mismatch_count")),
        "continuous_service_promoted": decision.as_ref().is_some_and(|value| bool_at(value, "cross_engine_differential_service_promoted")),
        "failures": failures,
    }));
    Ok(())
}

fn evaluate_w036_tracecalc_coverage(
    repo_root: &Path,
    evaluation: &mut Evaluation,
) -> Result<(), ContinuousAssuranceError> {
    let summary_path = relative_artifact_path([
        "docs",
        "test-runs",
        "core-engine",
        "tracecalc-reference-machine",
        W036_TRACECALC_COVERAGE_RUN_ID,
        "oracle-matrix",
        "run_summary.json",
    ]);
    let criteria_path = relative_artifact_path([
        "docs",
        "test-runs",
        "core-engine",
        "tracecalc-reference-machine",
        W036_TRACECALC_COVERAGE_RUN_ID,
        "oracle-matrix",
        "coverage_closure_criteria.json",
    ]);
    let summary = read_json(repo_root, &summary_path)?;
    let criteria = read_json(repo_root, &criteria_path)?;
    add_missing_if_absent(evaluation, &summary_path, &summary);
    add_missing_if_absent(evaluation, &criteria_path, &criteria);

    let mut failures = Vec::new();
    if let Some(summary) = &summary
        && number_at(summary, "missing_or_failed_row_count") != 0
    {
        failures.push("w036_tracecalc_has_missing_or_failed_rows".to_string());
    }
    if criteria
        .as_ref()
        .is_some_and(|value| bool_at(value, "full_oracle_claim"))
    {
        failures.push("unexpected_w036_full_tracecalc_oracle_claim".to_string());
    }
    evaluation.unexpected_mismatches.extend(failures.clone());
    evaluation.source_rows.push(json!({
        "input_id": "w036_tracecalc_coverage_closure",
        "artifact_paths": [summary_path, criteria_path],
        "evidence_state": if failures.is_empty() && summary.is_some() && criteria.is_some() {
            "coverage_closure_criteria_present_no_full_oracle_claim"
        } else {
            "source_gap"
        },
        "matrix_row_count": summary.as_ref().map_or(0, |value| number_at(value, "matrix_row_count")),
        "covered_row_count": summary.as_ref().map_or(0, |value| number_at(value, "covered_row_count")),
        "classified_uncovered_row_count": summary.as_ref().map_or(0, |value| number_at(value, "classified_uncovered_row_count")),
        "excluded_row_count": summary.as_ref().map_or(0, |value| number_at(value, "excluded_row_count")),
        "missing_or_failed_row_count": summary.as_ref().map_or(0, |value| number_at(value, "missing_or_failed_row_count")),
        "full_oracle_claim": criteria.as_ref().is_some_and(|value| bool_at(value, "full_oracle_claim")),
        "failures": failures,
    }));
    Ok(())
}

fn evaluate_w036_implementation_conformance(
    repo_root: &Path,
    evaluation: &mut Evaluation,
) -> Result<(), ContinuousAssuranceError> {
    let summary_path = relative_artifact_path([
        "docs",
        "test-runs",
        "core-engine",
        "implementation-conformance",
        W036_IMPLEMENTATION_CONFORMANCE_RUN_ID,
        "run_summary.json",
    ]);
    let guard_path = relative_artifact_path([
        "docs",
        "test-runs",
        "core-engine",
        "implementation-conformance",
        W036_IMPLEMENTATION_CONFORMANCE_RUN_ID,
        "w036_match_promotion_guard.json",
    ]);
    let summary = read_json(repo_root, &summary_path)?;
    let guard = read_json(repo_root, &guard_path)?;
    add_missing_if_absent(evaluation, &summary_path, &summary);
    add_missing_if_absent(evaluation, &guard_path, &guard);

    let mut failures = Vec::new();
    if let Some(summary) = &summary
        && number_at(summary, "failed_row_count") != 0
    {
        failures.push("w036_implementation_conformance_has_failed_rows".to_string());
    }
    if let Some(guard) = &guard
        && number_at(guard, "promoted_match_count") != 0
    {
        failures.push("w036_declared_gap_promoted_as_match".to_string());
    }
    evaluation.unexpected_mismatches.extend(failures.clone());
    evaluation.source_rows.push(json!({
        "input_id": "w036_implementation_conformance_closure",
        "artifact_paths": [summary_path, guard_path],
        "evidence_state": if failures.is_empty() && summary.is_some() && guard.is_some() {
            "conformance_closure_actions_present_without_match_promotion"
        } else {
            "source_gap"
        },
        "w036_action_row_count": summary.as_ref().map_or(0, |value| number_at(value, "w036_action_row_count")),
        "w036_first_fix_row_count": summary.as_ref().map_or(0, |value| number_at(value, "w036_first_fix_row_count")),
        "w036_blocker_routed_row_count": summary.as_ref().map_or(0, |value| number_at(value, "w036_blocker_routed_row_count")),
        "w036_match_promoted_count": summary.as_ref().map_or(0, |value| number_at(value, "w036_match_promoted_count")),
        "failed_row_count": summary.as_ref().map_or(0, |value| number_at(value, "failed_row_count")),
        "failures": failures,
    }));
    Ok(())
}

fn evaluate_w036_formal_artifacts(repo_root: &Path, evaluation: &mut Evaluation) {
    let missing = W036_FORMAL_ARTIFACTS
        .iter()
        .filter(|path| !repo_root.join(path).exists())
        .map(|path| (*path).to_string())
        .collect::<Vec<_>>();
    evaluation.missing_artifacts.extend(missing.clone());
    evaluation.source_rows.push(json!({
        "input_id": "w036_lean_tla_formal_packets",
        "artifact_paths": W036_FORMAL_ARTIFACTS,
        "evidence_state": if missing.is_empty() {
            "w036_bounded_formal_packets_present_no_full_verification_promotion"
        } else {
            "missing_artifact"
        },
        "missing_artifacts": missing,
    }));
}

fn evaluate_w036_tla_stage2_partition(
    repo_root: &Path,
    evaluation: &mut Evaluation,
) -> Result<(), ContinuousAssuranceError> {
    let summary_path = relative_artifact_path([
        "docs",
        "test-runs",
        "core-engine",
        "tla",
        W036_TLA_STAGE2_RUN_ID,
        "run_summary.json",
    ]);
    let blockers_path = relative_artifact_path([
        "docs",
        "test-runs",
        "core-engine",
        "tla",
        W036_TLA_STAGE2_RUN_ID,
        "promotion_blockers.json",
    ]);
    let summary = read_json(repo_root, &summary_path)?;
    let blockers = read_json(repo_root, &blockers_path)?;
    add_missing_if_absent(evaluation, &summary_path, &summary);
    add_missing_if_absent(evaluation, &blockers_path, &blockers);

    let mut failures = Vec::new();
    if let Some(summary) = &summary
        && number_at(summary, "failed_config_count") != 0
    {
        failures.push("w036_tla_stage2_has_failed_configs".to_string());
    }
    evaluation.unexpected_mismatches.extend(failures.clone());
    evaluation.source_rows.push(json!({
        "input_id": "w036_tla_stage2_partition",
        "artifact_paths": [summary_path, blockers_path],
        "evidence_state": if failures.is_empty() && summary.is_some() && blockers.is_some() {
            "bounded_tla_partition_evidence_present_no_stage2_promotion"
        } else {
            "source_gap"
        },
        "config_count": summary.as_ref().map_or(0, |value| number_at(value, "config_count")),
        "passed_config_count": summary.as_ref().map_or(0, |value| number_at(value, "passed_config_count")),
        "failed_config_count": summary.as_ref().map_or(0, |value| number_at(value, "failed_config_count")),
        "blocker_count": blockers.as_ref().map_or(0, |value| number_at(value, "blocker_count")),
        "failures": failures,
    }));
    Ok(())
}

fn evaluate_w036_independent_differential(
    repo_root: &Path,
    evaluation: &mut Evaluation,
) -> Result<(), ContinuousAssuranceError> {
    let independent_summary_path = relative_artifact_path([
        "docs",
        "test-runs",
        "core-engine",
        "independent-conformance",
        W036_INDEPENDENT_DIFFERENTIAL_RUN_ID,
        "run_summary.json",
    ]);
    let cross_summary_path = relative_artifact_path([
        "docs",
        "test-runs",
        "core-engine",
        "cross-engine-differential",
        W036_INDEPENDENT_DIFFERENTIAL_RUN_ID,
        "run_summary.json",
    ]);
    let guard_path = relative_artifact_path([
        "docs",
        "test-runs",
        "core-engine",
        "cross-engine-differential",
        W036_INDEPENDENT_DIFFERENTIAL_RUN_ID,
        "decision",
        "promotion_guard.json",
    ]);
    let independent_summary = read_json(repo_root, &independent_summary_path)?;
    let cross_summary = read_json(repo_root, &cross_summary_path)?;
    let guard = read_json(repo_root, &guard_path)?;
    add_missing_if_absent(evaluation, &independent_summary_path, &independent_summary);
    add_missing_if_absent(evaluation, &cross_summary_path, &cross_summary);
    add_missing_if_absent(evaluation, &guard_path, &guard);

    let mut failures = Vec::new();
    for summary in [&independent_summary, &cross_summary].into_iter().flatten() {
        if number_at(summary, "missing_artifact_count") != 0 {
            failures.push("w036_independent_or_cross_engine_missing_artifacts".to_string());
        }
        if number_at(summary, "unexpected_mismatch_count") != 0 {
            failures.push("w036_independent_or_cross_engine_unexpected_mismatch".to_string());
        }
    }
    if guard.as_ref().is_some_and(|value| {
        bool_at(value, "continuous_cross_engine_service_promoted")
            || bool_at(value, "full_independent_evaluator_promoted")
            || bool_at(value, "pack_grade_promoted")
            || bool_at(value, "stage2_policy_promoted")
    }) {
        failures.push("unexpected_w036_differential_or_pack_promotion".to_string());
    }
    evaluation.unexpected_mismatches.extend(failures.clone());
    evaluation.source_rows.push(json!({
        "input_id": "w036_independent_differential_harness",
        "artifact_paths": [independent_summary_path, cross_summary_path, guard_path],
        "evidence_state": if failures.is_empty() && independent_summary.is_some() && cross_summary.is_some() && guard.is_some() {
            "independent_differential_harness_present_without_service_promotion"
        } else {
            "source_gap"
        },
        "comparison_row_count": independent_summary.as_ref().map_or(0, |value| number_at(value, "comparison_row_count")),
        "w036_diversity_row_count": independent_summary.as_ref().map_or(0, |value| number_at(value, "w036_diversity_row_count")),
        "w036_differential_row_count": cross_summary.as_ref().map_or(0, |value| number_at(value, "differential_row_count")),
        "w036_promotion_blocker_count": independent_summary.as_ref().map_or(0, |value| number_at(value, "w036_promotion_blocker_count")),
        "declared_gap_count": cross_summary.as_ref().map_or(0, |value| number_at(value, "declared_gap_count")),
        "continuous_cross_engine_service_promoted": cross_summary.as_ref().is_some_and(|value| bool_at(value, "continuous_cross_engine_service_promoted")),
        "full_independent_evaluator_promoted": cross_summary.as_ref().is_some_and(|value| bool_at(value, "full_independent_evaluator_promoted")),
        "failures": failures,
    }));
    Ok(())
}

fn evaluate_w037_tracecalc_observable(
    repo_root: &Path,
    evaluation: &mut Evaluation,
) -> Result<(), ContinuousAssuranceError> {
    let summary_path = relative_artifact_path([
        "docs",
        "test-runs",
        "core-engine",
        "tracecalc-reference-machine",
        W037_TRACECALC_OBSERVABLE_RUN_ID,
        "oracle-matrix",
        "run_summary.json",
    ]);
    let criteria_path = relative_artifact_path([
        "docs",
        "test-runs",
        "core-engine",
        "tracecalc-reference-machine",
        W037_TRACECALC_OBSERVABLE_RUN_ID,
        "oracle-matrix",
        "coverage_closure_criteria.json",
    ]);
    let summary = read_json(repo_root, &summary_path)?;
    let criteria = read_json(repo_root, &criteria_path)?;
    add_missing_if_absent(evaluation, &summary_path, &summary);
    add_missing_if_absent(evaluation, &criteria_path, &criteria);

    let mut failures = Vec::new();
    if let Some(summary) = &summary {
        if number_at(summary, "missing_or_failed_row_count") != 0 {
            failures.push("w037_tracecalc_has_missing_or_failed_rows".to_string());
        }
        if number_at(summary, "classified_uncovered_row_count") != 0 {
            failures.push("w037_tracecalc_has_uncovered_rows".to_string());
        }
    }
    if criteria
        .as_ref()
        .is_some_and(|value| bool_at(value, "full_oracle_claim"))
    {
        failures.push("unexpected_w037_full_tracecalc_oracle_claim".to_string());
    }
    evaluation.unexpected_mismatches.extend(failures.clone());
    evaluation.source_rows.push(json!({
        "input_id": "w037_tracecalc_observable_closure",
        "artifact_paths": [summary_path, criteria_path],
        "evidence_state": if failures.is_empty() && summary.is_some() && criteria.is_some() {
            "observable_closure_present_with_authority_exclusion_no_full_oracle_claim"
        } else {
            "source_gap"
        },
        "matrix_row_count": summary.as_ref().map_or(0, |value| number_at(value, "matrix_row_count")),
        "covered_row_count": summary.as_ref().map_or(0, |value| number_at(value, "covered_row_count")),
        "classified_uncovered_row_count": summary.as_ref().map_or(0, |value| number_at(value, "classified_uncovered_row_count")),
        "excluded_row_count": summary.as_ref().map_or(0, |value| number_at(value, "excluded_row_count")),
        "missing_or_failed_row_count": summary.as_ref().map_or(0, |value| number_at(value, "missing_or_failed_row_count")),
        "full_oracle_claim": criteria.as_ref().is_some_and(|value| bool_at(value, "full_oracle_claim")),
        "failures": failures,
    }));
    Ok(())
}

fn evaluate_w037_implementation_conformance(
    repo_root: &Path,
    evaluation: &mut Evaluation,
) -> Result<(), ContinuousAssuranceError> {
    let summary_path = relative_artifact_path([
        "docs",
        "test-runs",
        "core-engine",
        "implementation-conformance",
        W037_IMPLEMENTATION_CONFORMANCE_RUN_ID,
        "run_summary.json",
    ]);
    let guard_path = relative_artifact_path([
        "docs",
        "test-runs",
        "core-engine",
        "implementation-conformance",
        W037_IMPLEMENTATION_CONFORMANCE_RUN_ID,
        "w037_match_promotion_guard.json",
    ]);
    let summary = read_json(repo_root, &summary_path)?;
    let guard = read_json(repo_root, &guard_path)?;
    add_missing_if_absent(evaluation, &summary_path, &summary);
    add_missing_if_absent(evaluation, &guard_path, &guard);

    let mut failures = Vec::new();
    if let Some(summary) = &summary {
        if number_at(summary, "failed_row_count") != 0 {
            failures.push("w037_implementation_conformance_has_failed_rows".to_string());
        }
        if number_at(summary, "w037_match_promoted_count") != 1 {
            failures.push("w037_expected_exactly_one_match_promoted_row".to_string());
        }
    }
    if let Some(guard) = &guard
        && number_at(guard, "promoted_match_count") != 1
    {
        failures.push("w037_declared_gap_promotion_guard_unexpected_count".to_string());
    }
    evaluation.unexpected_mismatches.extend(failures.clone());
    evaluation.source_rows.push(json!({
        "input_id": "w037_implementation_conformance_closure",
        "artifact_paths": [summary_path, guard_path],
        "evidence_state": if failures.is_empty() && summary.is_some() && guard.is_some() {
            "one_declared_gap_promoted_with_guard_and_residual_blockers_preserved"
        } else {
            "source_gap"
        },
        "w037_decision_row_count": summary.as_ref().map_or(0, |value| number_at(value, "w037_decision_row_count")),
        "w037_fixed_or_promoted_count": summary.as_ref().map_or(0, |value| number_at(value, "w037_fixed_or_promoted_count")),
        "w037_residual_blocker_count": summary.as_ref().map_or(0, |value| number_at(value, "w037_residual_blocker_count")),
        "w037_match_promoted_count": summary.as_ref().map_or(0, |value| number_at(value, "w037_match_promoted_count")),
        "failed_row_count": summary.as_ref().map_or(0, |value| number_at(value, "failed_row_count")),
        "failures": failures,
    }));
    Ok(())
}

fn evaluate_w037_direct_oxfml(
    repo_root: &Path,
    evaluation: &mut Evaluation,
) -> Result<(), ContinuousAssuranceError> {
    let summary_path = relative_artifact_path([
        "docs",
        "test-runs",
        "core-engine",
        "upstream-host",
        W037_DIRECT_OXFML_RUN_ID,
        "run_summary.json",
    ]);
    let summary = read_json(repo_root, &summary_path)?;
    add_missing_if_absent(evaluation, &summary_path, &summary);

    let mut failures = Vec::new();
    if let Some(summary) = &summary {
        if number_at(summary, "expectation_mismatch_count") != 0 {
            failures.push("w037_direct_oxfml_expectation_mismatch".to_string());
        }
        if number_at(summary, "w073_typed_rule_case_count") == 0 {
            failures.push("w037_direct_oxfml_missing_w073_typed_rule_guard".to_string());
        }
        if summary.get("promotion_limits").is_some_and(|limits| {
            bool_at(limits, "pack_grade_replay_promoted")
                || bool_at(limits, "c5_promoted")
                || bool_at(limits, "general_oxfunc_kernel_claimed")
        }) {
            failures.push("unexpected_w037_direct_oxfml_promotion_claim".to_string());
        }
    }
    evaluation.unexpected_mismatches.extend(failures.clone());
    evaluation.source_rows.push(json!({
        "input_id": "w037_direct_oxfml_evaluator",
        "artifact_paths": [summary_path],
        "evidence_state": if failures.is_empty() && summary.is_some() {
            "direct_oxfml_slice_present_with_let_lambda_and_w073_guard"
        } else {
            "source_gap"
        },
        "fixture_case_count": summary.as_ref().map_or(0, |value| number_at(value, "fixture_case_count")),
        "direct_oxfml_case_count": summary.as_ref().map_or(0, |value| number_at(value, "direct_oxfml_case_count")),
        "let_lambda_case_count": summary.as_ref().map_or(0, |value| number_at(value, "let_lambda_case_count")),
        "w073_typed_rule_case_count": summary.as_ref().map_or(0, |value| number_at(value, "w073_typed_rule_case_count")),
        "expectation_mismatch_count": summary.as_ref().map_or(0, |value| number_at(value, "expectation_mismatch_count")),
        "failures": failures,
    }));
    Ok(())
}

fn evaluate_w037_formal_inventory(
    repo_root: &Path,
    evaluation: &mut Evaluation,
) -> Result<(), ContinuousAssuranceError> {
    let summary_path = relative_artifact_path([
        "docs",
        "test-runs",
        "core-engine",
        "formal-inventory",
        W037_FORMAL_INVENTORY_RUN_ID,
        "run_summary.json",
    ]);
    let validation_path = relative_artifact_path([
        "docs",
        "test-runs",
        "core-engine",
        "formal-inventory",
        W037_FORMAL_INVENTORY_RUN_ID,
        "validation.json",
    ]);
    let summary = read_json(repo_root, &summary_path)?;
    let validation = read_json(repo_root, &validation_path)?;
    add_missing_if_absent(evaluation, &summary_path, &summary);
    add_missing_if_absent(evaluation, &validation_path, &validation);

    let mut failures = Vec::new();
    if let Some(summary) = &summary {
        if !bool_at(summary, "all_checked_artifacts_passed") {
            failures.push("w037_formal_inventory_checked_artifacts_not_all_passed".to_string());
        }
        if number_at(summary, "lean_explicit_axiom_count") != 0 {
            failures.push("w037_formal_inventory_has_explicit_axioms".to_string());
        }
        if number_at(summary, "lean_sorry_placeholder_count") != 0 {
            failures.push("w037_formal_inventory_has_sorry_placeholders".to_string());
        }
        if number_at(summary, "tla_failed_config_count") != 0 {
            failures.push("w037_formal_inventory_has_failed_tla_configs".to_string());
        }
    }
    evaluation.unexpected_mismatches.extend(failures.clone());
    evaluation.source_rows.push(json!({
        "input_id": "w037_proof_model_inventory",
        "artifact_paths": [summary_path, validation_path],
        "evidence_state": if failures.is_empty() && summary.is_some() && validation.is_some() {
            "proof_model_inventory_checked_without_full_verification_promotion"
        } else {
            "source_gap"
        },
        "lean_file_count": summary.as_ref().map_or(0, |value| number_at(value, "lean_file_count")),
        "lean_files_checked": summary.as_ref().map_or(0, |value| number_at(value, "lean_files_checked")),
        "lean_explicit_axiom_count": summary.as_ref().map_or(0, |value| number_at(value, "lean_explicit_axiom_count")),
        "lean_sorry_placeholder_count": summary.as_ref().map_or(0, |value| number_at(value, "lean_sorry_placeholder_count")),
        "tla_routine_config_count": summary.as_ref().map_or(0, |value| number_at(value, "tla_routine_config_count")),
        "tla_failed_config_count": summary.as_ref().map_or(0, |value| number_at(value, "tla_failed_config_count")),
        "failures": failures,
    }));
    Ok(())
}

fn evaluate_w037_stage2_criteria(
    repo_root: &Path,
    evaluation: &mut Evaluation,
) -> Result<(), ContinuousAssuranceError> {
    let summary_path = relative_artifact_path([
        "docs",
        "test-runs",
        "core-engine",
        "stage2-criteria",
        W037_STAGE2_CRITERIA_RUN_ID,
        "run_summary.json",
    ]);
    let decision_path = relative_artifact_path([
        "docs",
        "test-runs",
        "core-engine",
        "stage2-criteria",
        W037_STAGE2_CRITERIA_RUN_ID,
        "promotion_decision.json",
    ]);
    let summary = read_json(repo_root, &summary_path)?;
    let decision = read_json(repo_root, &decision_path)?;
    add_missing_if_absent(evaluation, &summary_path, &summary);
    add_missing_if_absent(evaluation, &decision_path, &decision);

    let mut failures = Vec::new();
    if let Some(summary) = &summary {
        if bool_at(summary, "stage2_policy_promoted") {
            failures.push("unexpected_w037_stage2_policy_promotion".to_string());
        }
        if bool_at(summary, "stage2_promotion_candidate") {
            failures.push("unexpected_w037_stage2_promotion_candidate".to_string());
        }
        if number_at(summary, "blocked_criteria_row_count") == 0 {
            failures.push("w037_stage2_criteria_expected_blocked_rows".to_string());
        }
    }
    if decision
        .as_ref()
        .is_some_and(|value| bool_at(value, "stage2_policy_promoted"))
    {
        failures.push("unexpected_w037_stage2_decision_policy_promotion".to_string());
    }
    evaluation.unexpected_mismatches.extend(failures.clone());
    evaluation.source_rows.push(json!({
        "input_id": "w037_stage2_criteria",
        "artifact_paths": [summary_path, decision_path],
        "evidence_state": if failures.is_empty() && summary.is_some() && decision.is_some() {
            "stage2_criteria_present_with_deterministic_replay_blockers"
        } else {
            "source_gap"
        },
        "criteria_row_count": summary.as_ref().map_or(0, |value| number_at(value, "criteria_row_count")),
        "satisfied_criteria_row_count": summary.as_ref().map_or(0, |value| number_at(value, "satisfied_criteria_row_count")),
        "blocked_criteria_row_count": summary.as_ref().map_or(0, |value| number_at(value, "blocked_criteria_row_count")),
        "stage2_policy_promoted": summary.as_ref().is_some_and(|value| bool_at(value, "stage2_policy_promoted")),
        "stage2_promotion_candidate": summary.as_ref().is_some_and(|value| bool_at(value, "stage2_promotion_candidate")),
        "failures": failures,
    }));
    Ok(())
}

fn continuous_assurance_schedule(run_id: &str, artifact_root: &str) -> Value {
    let mut lanes = vec![
        json!({
            "lane_id": "continuous.semantic.smoke",
            "cadence": "per_change_or_nightly",
            "required_commands": [
                "cargo test -p oxcalc-tracecalc oracle_matrix",
                "cargo test -p oxcalc-tracecalc implementation_conformance",
                "cargo test -p oxcalc-tracecalc scale_semantic_binding"
            ],
            "acceptance": [
                "zero failed oracle-matrix rows",
                "zero failed implementation-conformance rows",
                "zero source missing artifacts"
            ]
        }),
        json!({
            "lane_id": "continuous.scale.regression",
            "cadence": "scheduled_weekly_or_release_candidate",
            "required_profiles": [
                "grid-cross-sum",
                "fanout-bands",
                "dynamic-indirect-stripes",
                "relative-rebind-churn"
            ],
            "acceptance": [
                "closed-form semantic validation passes",
                "metamorphic signature rows remain matched",
                "timing changes are reported as measurement, not correctness proof"
            ]
        }),
        json!({
            "lane_id": "continuous.cross_engine.diff",
            "cadence": "release_candidate_and_before_pack_promotion",
            "required_comparisons": [
                "TraceCalc oracle matrix",
                "TreeCalc/CoreEngine implementation conformance",
                "million-node scale semantic signatures",
                "pack/capability decision packet"
            ],
            "acceptance": [
                "no unexpected mismatches",
                "declared gaps are not counted as matches",
                "pack C5 promotion requires all blockers cleared"
            ]
        }),
    ];

    if is_operation_history_run(run_id) {
        lanes.push(json!({
            "lane_id": "continuous.history.thresholds",
            "cadence": if is_w037_run(run_id) {
                "file_backed_service_readiness_from_checked_in_successor_evidence_until_runner_exists"
            } else {
                "simulated_from_checked_in_successor_evidence_until_runner_exists"
            },
            "required_artifacts": [
                "history/assurance_history_window.json",
                "thresholds/regression_thresholds.json",
                "alerts/quarantine_policy.json",
                "operation/simulated_multi_run_evidence.json"
            ],
            "acceptance": [
                "history window is machine-readable and points at checked-in evidence roots",
                "semantic regressions quarantine before any performance interpretation",
                "timing rows are alert-only measurements and never correctness proof",
                "no operated continuous service promotion is made from simulated history"
            ]
        }));
    }
    if is_w037_run(run_id) {
        lanes.push(json!({
            "lane_id": "continuous.service.readiness",
            "cadence": "release_candidate_and_before_pack_or_service_promotion",
            "required_artifacts": [
                "service/service_readiness.json",
                "service/cross_engine_service_pilot.json",
                "differentials/cross_engine_differential_gate.json"
            ],
            "acceptance": [
                "service-readiness criteria distinguish satisfied file-backed inputs from blocked operated-service claims",
                "cross-engine service pilot remains file-backed until an operated runner and alert dispatcher exist",
                "full independent evaluator diversity remains a blocker until a qualifying evaluator implementation is present"
            ]
        }));
    }

    json!({
        "schema_version": SCHEDULE_SCHEMA_V1,
        "run_id": run_id,
        "artifact_root": artifact_root,
        "lanes": lanes,
    })
}

fn cross_engine_rows(run_id: &str) -> Vec<Value> {
    let mut rows = vec![
        json!({
            "row_id": "diff.tracecalc_oracle_matrix_to_engine_projection",
            "comparison_state": "bounded_semantic_match_with_classified_uncovered_rows",
            "required_for_promotion": true,
            "current_limit": "TraceCalc covers W035 matrix rows but not full engine universe",
            "failures": []
        }),
        json!({
            "row_id": "diff.implementation_conformance_gap_dispositions",
            "comparison_state": "gap_dispositions_valid_without_match_promotion",
            "required_for_promotion": true,
            "current_limit": "implementation-work and spec-evolution deferrals remain",
            "failures": []
        }),
        json!({
            "row_id": "diff.scale_semantic_signatures",
            "comparison_state": "scale_signatures_matched_without_continuous_service",
            "required_for_promotion": true,
            "current_limit": "single checked-in W034 binding is not recurring assurance",
            "failures": []
        }),
        json!({
            "row_id": "diff.independent_evaluator_diversity",
            "comparison_state": "not_fully_independent",
            "required_for_promotion": true,
            "current_limit": "TreeCalc/CoreEngine projection is useful but not fully independent evaluator implementation diversity",
            "failures": []
        }),
    ];

    if is_operation_history_run(run_id) {
        rows.push(json!({
            "row_id": "diff.w036_cross_engine_differential_harness",
            "comparison_state": "deterministic_harness_present_without_operated_service",
            "required_for_promotion": true,
            "current_limit": "W036 emits machine-readable differential rows and promotion blockers, not a continuous cross-engine service",
            "failures": []
        }));
        rows.push(json!({
            "row_id": "diff.w036_history_threshold_quarantine",
            "comparison_state": "simulated_history_thresholds_present_without_service_promotion",
            "required_for_promotion": true,
            "current_limit": "W036 history, thresholds, and quarantine policy are checked-in simulated evidence, not an enforcing scheduler or alert service",
            "failures": []
        }));
    }
    if is_w037_run(run_id) {
        rows.push(json!({
            "row_id": "diff.w037_direct_oxfml_guard",
            "comparison_state": "direct_oxfml_fixture_slice_present_without_pack_grade_replay",
            "required_for_promotion": true,
            "current_limit": "W037 exercises the direct OxFml runtime facade, narrow LET/LAMBDA rows, and W073 typed formatting guard, but does not promote pack-grade replay",
            "failures": []
        }));
        rows.push(json!({
            "row_id": "diff.w037_stage2_partition_service_blocker",
            "comparison_state": "stage2_criteria_present_without_operated_differential_service",
            "required_for_promotion": true,
            "current_limit": "Stage 2 criteria state deterministic replay and operated differential requirements, but deterministic partition replay and operated service remain absent",
            "failures": []
        }));
        rows.push(json!({
            "row_id": "diff.w037_operated_service_pilot",
            "comparison_state": "file_backed_pilot_present_without_operated_service_promotion",
            "required_for_promotion": true,
            "current_limit": "W037 records readiness inputs and blockers, not a scheduled runner, enforcing alert dispatcher, or continuous cross-engine service",
            "failures": []
        }));
    }

    rows
}

fn decision_packet(
    run_id: &str,
    evaluation: &Evaluation,
    operation_artifacts: Option<&OperationArtifacts>,
) -> Value {
    json!({
        "schema_version": DECISION_SCHEMA_V1,
        "run_id": run_id,
        "decision_status": if evaluation.missing_artifacts.is_empty() && evaluation.unexpected_mismatches.is_empty() {
            if is_w037_run(run_id) {
                "w037_operated_assurance_service_pilot_recorded_without_service_promotion"
            } else if is_w036_run(run_id) {
                "w036_simulated_continuous_assurance_history_defined_without_service_promotion"
            } else {
                "continuous_assurance_gate_defined_without_promotion"
            }
        } else {
            if is_w037_run(run_id) {
                "w037_operated_assurance_service_pilot_has_source_gaps"
            } else if is_w036_run(run_id) {
                "w036_continuous_assurance_operation_has_source_gaps"
            } else {
                "continuous_assurance_gate_has_source_gaps"
            }
        },
        "continuous_scale_assurance_promoted": false,
        "cross_engine_differential_service_promoted": false,
        "operated_continuous_assurance_service_promoted": false,
        "fully_independent_evaluator_promoted": false,
        "simulated_history_window_present": operation_artifacts.is_some(),
        "regression_thresholds_present": operation_artifacts.is_some(),
        "quarantine_alert_policy_present": operation_artifacts.is_some(),
        "service_readiness_present": operation_artifacts.is_some_and(|artifacts| artifacts.service_readiness.is_some()),
        "cross_engine_service_pilot_present": operation_artifacts.is_some_and(|artifacts| artifacts.cross_engine_service_pilot.is_some()),
        "history_window_row_count": operation_artifacts.map_or(0, |artifacts| number_at(&artifacts.history_window, "row_count")),
        "regression_threshold_count": operation_artifacts.map_or(0, |artifacts| number_at(&artifacts.regression_thresholds, "rule_count")),
        "quarantine_rule_count": operation_artifacts.map_or(0, |artifacts| number_at(&artifacts.quarantine_policy, "rule_count")),
        "service_readiness_criteria_count": operation_artifacts
            .and_then(|artifacts| artifacts.service_readiness.as_ref())
            .map_or(0, |value| number_at(value, "criteria_count")),
        "service_readiness_blocked_count": operation_artifacts
            .and_then(|artifacts| artifacts.service_readiness.as_ref())
            .map_or(0, |value| number_at(value, "blocked_criteria_count")),
        "pack_capability_promoted": false,
        "stage2_scheduler_promoted": false,
        "performance_claim_promoted": false,
        "timing_is_correctness_evidence": false,
        "source_missing_artifact_count": evaluation.missing_artifacts.len(),
        "unexpected_mismatch_count": evaluation.unexpected_mismatches.len(),
        "no_promotion_reason_ids": evaluation.no_promotion_reasons,
        "semantic_equivalence_statement": if is_w037_run(run_id) {
            "This runner reads existing W034/W035/W036/W037 evidence and emits file-backed service-readiness and cross-engine pilot artifacts only. It does not change coordinator scheduling, invalidation, dependency graph construction, soft-reference resolution, recalc, publication, reject, TraceCalc, TreeCalc, Lean/TLA model, pack-decision, service operation, alert dispatch, or OxFml/OxFunc evaluator behavior."
        } else if is_w036_run(run_id) {
            "This runner reads existing W034/W035/W036 evidence and emits simulated continuous-assurance operation, history-window, threshold, and quarantine-policy artifacts only. It does not change coordinator scheduling, invalidation, dependency graph construction, soft-reference resolution, recalc, publication, reject, TraceCalc, TreeCalc, Lean/TLA model, pack-decision, or OxFml/OxFunc evaluator behavior."
        } else {
            "This runner reads existing W034/W035 evidence and emits continuous-assurance gate criteria only. It does not change coordinator scheduling, invalidation, recalc, publication, reject, TraceCalc, TreeCalc, or evaluator behavior."
        },
    })
}

fn required_artifacts(run_id: &str) -> Vec<String> {
    let mut artifacts = vec![
        "run_summary.json",
        "evidence/source_evidence_index.json",
        "schedule/continuous_assurance_schedule.json",
        "differentials/cross_engine_differential_gate.json",
        "decision/continuous_assurance_decision.json",
        "replay-appliance/bundle_manifest.json",
        "replay-appliance/validation/bundle_validation.json",
    ];

    if is_operation_history_run(run_id) {
        artifacts.extend([
            "history/assurance_history_window.json",
            "thresholds/regression_thresholds.json",
            "alerts/quarantine_policy.json",
            "operation/simulated_multi_run_evidence.json",
        ]);
    }
    if is_w037_run(run_id) {
        artifacts.extend([
            "service/service_readiness.json",
            "service/cross_engine_service_pilot.json",
        ]);
    }

    artifacts
        .iter()
        .map(|artifact| {
            relative_artifact_path([
                "docs",
                "test-runs",
                "core-engine",
                "continuous-assurance",
                run_id,
                artifact,
            ])
        })
        .chain(source_artifacts_for_run(run_id))
        .collect()
}

fn source_artifacts_for_run(run_id: &str) -> Vec<String> {
    let mut artifacts = source_artifacts();
    if is_operation_history_run(run_id) {
        artifacts.extend(w036_source_artifacts());
    }
    if is_w037_run(run_id) {
        artifacts.extend(w037_source_artifacts());
    }
    artifacts
}

fn source_artifacts() -> Vec<String> {
    let mut artifacts = vec![
        relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "metamorphic-scale-semantic-binding",
            W034_SCALE_BINDING_RUN_ID,
            "run_summary.json",
        ]),
        relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "metamorphic-scale-semantic-binding",
            W034_SCALE_BINDING_RUN_ID,
            "decision",
            "continuous_scale_assurance_criteria.json",
        ]),
        relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "pack-capability",
            W034_PACK_RUN_ID,
            "run_summary.json",
        ]),
        relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "pack-capability",
            W034_PACK_RUN_ID,
            "decision",
            "pack_capability_decision.json",
        ]),
        relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "tracecalc-reference-machine",
            W035_ORACLE_MATRIX_RUN_ID,
            "oracle-matrix",
            "run_summary.json",
        ]),
        relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "implementation-conformance",
            W035_IMPLEMENTATION_CONFORMANCE_RUN_ID,
            "run_summary.json",
        ]),
    ];
    artifacts.extend(W035_FORMAL_ARTIFACTS.iter().map(|path| (*path).to_string()));
    artifacts
}

fn w036_source_artifacts() -> Vec<String> {
    let mut artifacts = vec![
        relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "continuous-assurance",
            W035_CONTINUOUS_ASSURANCE_RUN_ID,
            "run_summary.json",
        ]),
        relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "continuous-assurance",
            W035_CONTINUOUS_ASSURANCE_RUN_ID,
            "decision",
            "continuous_assurance_decision.json",
        ]),
        relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "tracecalc-reference-machine",
            W036_TRACECALC_COVERAGE_RUN_ID,
            "oracle-matrix",
            "run_summary.json",
        ]),
        relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "tracecalc-reference-machine",
            W036_TRACECALC_COVERAGE_RUN_ID,
            "oracle-matrix",
            "coverage_closure_criteria.json",
        ]),
        relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "implementation-conformance",
            W036_IMPLEMENTATION_CONFORMANCE_RUN_ID,
            "run_summary.json",
        ]),
        relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "implementation-conformance",
            W036_IMPLEMENTATION_CONFORMANCE_RUN_ID,
            "w036_match_promotion_guard.json",
        ]),
        relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "tla",
            W036_TLA_STAGE2_RUN_ID,
            "run_summary.json",
        ]),
        relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "tla",
            W036_TLA_STAGE2_RUN_ID,
            "promotion_blockers.json",
        ]),
        relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "independent-conformance",
            W036_INDEPENDENT_DIFFERENTIAL_RUN_ID,
            "run_summary.json",
        ]),
        relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "cross-engine-differential",
            W036_INDEPENDENT_DIFFERENTIAL_RUN_ID,
            "run_summary.json",
        ]),
        relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "cross-engine-differential",
            W036_INDEPENDENT_DIFFERENTIAL_RUN_ID,
            "decision",
            "promotion_guard.json",
        ]),
    ];
    artifacts.extend(W036_FORMAL_ARTIFACTS.iter().map(|path| (*path).to_string()));
    artifacts
}

fn w037_source_artifacts() -> Vec<String> {
    vec![
        relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "tracecalc-reference-machine",
            W037_TRACECALC_OBSERVABLE_RUN_ID,
            "oracle-matrix",
            "run_summary.json",
        ]),
        relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "tracecalc-reference-machine",
            W037_TRACECALC_OBSERVABLE_RUN_ID,
            "oracle-matrix",
            "coverage_closure_criteria.json",
        ]),
        relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "implementation-conformance",
            W037_IMPLEMENTATION_CONFORMANCE_RUN_ID,
            "run_summary.json",
        ]),
        relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "implementation-conformance",
            W037_IMPLEMENTATION_CONFORMANCE_RUN_ID,
            "w037_match_promotion_guard.json",
        ]),
        relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "upstream-host",
            W037_DIRECT_OXFML_RUN_ID,
            "run_summary.json",
        ]),
        relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "formal-inventory",
            W037_FORMAL_INVENTORY_RUN_ID,
            "run_summary.json",
        ]),
        relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "formal-inventory",
            W037_FORMAL_INVENTORY_RUN_ID,
            "validation.json",
        ]),
        relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "stage2-criteria",
            W037_STAGE2_CRITERIA_RUN_ID,
            "run_summary.json",
        ]),
        relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "stage2-criteria",
            W037_STAGE2_CRITERIA_RUN_ID,
            "promotion_decision.json",
        ]),
    ]
}

fn operation_artifacts(
    run_id: &str,
    artifact_root: &str,
    evaluation: &Evaluation,
) -> OperationArtifacts {
    let history_rows = operation_history_rows(run_id, evaluation);
    let semantic_acceptance_state =
        if evaluation.missing_artifacts.is_empty() && evaluation.unexpected_mismatches.is_empty() {
            if is_w037_run(run_id) {
                "w037_service_readiness_inputs_valid_with_known_promotion_blockers"
            } else {
                "history_window_semantic_inputs_valid_with_known_promotion_blockers"
            }
        } else {
            "history_window_has_source_gaps"
        };
    let history_window = json!({
        "schema_version": HISTORY_WINDOW_SCHEMA_V1,
        "run_id": run_id,
        "artifact_root": artifact_root,
        "window_kind": if is_w037_run(run_id) {
            "file_backed_service_readiness_history_from_checked_in_evidence"
        } else {
            "simulated_multi_run_history_from_checked_in_evidence"
        },
        "continuous_service_present": false,
        "semantic_acceptance_state": semantic_acceptance_state,
        "timing_correctness_role": "measurement_only_not_correctness_evidence",
        "row_count": history_rows.len(),
        "rows": history_rows,
    });

    let regression_rules = regression_threshold_rules(run_id);
    let regression_thresholds = json!({
        "schema_version": REGRESSION_THRESHOLDS_SCHEMA_V1,
        "run_id": run_id,
        "artifact_root": artifact_root,
        "rule_count": regression_rules.len(),
        "semantic_first": true,
        "timing_can_quarantine": false,
        "rules": regression_rules,
    });

    let quarantine_rules = quarantine_policy_rules(run_id);
    let quarantine_policy = json!({
        "schema_version": QUARANTINE_POLICY_SCHEMA_V1,
        "run_id": run_id,
        "artifact_root": artifact_root,
        "rule_count": quarantine_rules.len(),
        "policy_state": "policy_defined_no_alert_executor_promotion",
        "continuous_service_present": false,
        "rules": quarantine_rules,
    });

    let simulated_multi_run = json!({
        "schema_version": SIMULATED_MULTI_RUN_SCHEMA_V1,
        "run_id": run_id,
        "artifact_root": artifact_root,
        "operation_mode": if is_w037_run(run_id) {
            "file_backed_service_pilot_from_checked_in_evidence_epochs"
        } else {
            "simulated_from_checked_in_evidence_epochs"
        },
        "continuous_service_present": false,
        "continuous_service_promoted": false,
        "row_count": history_window.get("rows").and_then(Value::as_array).map_or(0, Vec::len),
        "rows": history_window.get("rows").cloned().unwrap_or_else(|| json!([])),
        "simulation_limits": [
            "not a scheduled runner",
            "not an alert dispatcher",
            "not a continuous cross-engine differential service",
            "not timing-based correctness evidence"
        ],
    });

    let service_readiness = is_w037_run(run_id).then(|| {
        w037_service_readiness(
            run_id,
            artifact_root,
            evaluation,
            &history_window,
            &regression_thresholds,
            &quarantine_policy,
        )
    });
    let cross_engine_service_pilot = is_w037_run(run_id).then(|| {
        w037_cross_engine_service_pilot(
            run_id,
            artifact_root,
            evaluation,
            service_readiness.as_ref().expect("W037 readiness exists"),
            &history_window,
        )
    });

    OperationArtifacts {
        history_window,
        regression_thresholds,
        quarantine_policy,
        simulated_multi_run,
        service_readiness,
        cross_engine_service_pilot,
    }
}

fn operation_history_rows(run_id: &str, evaluation: &Evaluation) -> Vec<Value> {
    let mut row_inputs = vec![
        ("w034.scale_binding", "w034_scale_semantic_binding"),
        ("w035.continuous_gate", "w035_continuous_assurance_gate"),
        ("w036.tracecalc_coverage", "w036_tracecalc_coverage_closure"),
        (
            "w036.implementation_conformance",
            "w036_implementation_conformance_closure",
        ),
        ("w036.tla_stage2_partition", "w036_tla_stage2_partition"),
        (
            "w036.independent_differential",
            "w036_independent_differential_harness",
        ),
    ];
    if is_w037_run(run_id) {
        row_inputs.extend([
            (
                "w037.tracecalc_observable",
                "w037_tracecalc_observable_closure",
            ),
            (
                "w037.implementation_conformance",
                "w037_implementation_conformance_closure",
            ),
            ("w037.direct_oxfml", "w037_direct_oxfml_evaluator"),
            ("w037.proof_model_inventory", "w037_proof_model_inventory"),
            ("w037.stage2_criteria", "w037_stage2_criteria"),
        ]);
    }
    row_inputs
        .into_iter()
    .enumerate()
    .map(|(index, (evidence_epoch, input_id))| {
        let source = source_row(evaluation, input_id);
        json!({
            "window_order": index + 1,
            "evidence_epoch": evidence_epoch,
            "source_input_id": input_id,
            "source_artifact_paths": source
                .and_then(|row| row.get("artifact_paths"))
                .cloned()
                .unwrap_or_else(|| json!([])),
            "semantic_state": source.map_or("missing_source_row".to_string(), |row| text_at(row, "evidence_state")),
            "missing_artifact_count": source.map_or(1, |row| number_at(row, "missing_artifact_count")),
            "unexpected_mismatch_count": source.map_or(1, failure_count),
            "failed_row_count": source.map_or(0, |row| number_at(row, "failed_row_count") + number_at(row, "missing_or_failed_row_count") + number_at(row, "failed_config_count")),
            "declared_gap_count": source.map_or(0, |row| number_at(row, "declared_gap_count")),
            "blocker_count": source.map_or(0, |row| number_at(row, "blocker_count") + number_at(row, "w036_promotion_blocker_count")),
            "timing_role": "measurement_only",
            "promotion_consequence": "source may feed later pack-grade decisions only when semantic thresholds pass and declared gaps are not counted as matches"
        })
    })
    .collect()
}

fn regression_threshold_rules(run_id: &str) -> Vec<Value> {
    let minimum_window_rows = if is_w037_run(run_id) { 11 } else { 6 };
    let minimum_window_reason = if is_w037_run(run_id) {
        "W037 requires a machine-readable history window across predecessor and W037 service-readiness evidence"
    } else {
        "W036 requires a machine-readable history window across predecessor and successor evidence"
    };
    vec![
        json!({
            "rule_id": "threshold.semantic.missing_artifacts_zero",
            "severity": "quarantine",
            "metric": "missing_artifact_count",
            "operator": "eq",
            "value": 0,
            "reason": "missing evidence invalidates the assurance window"
        }),
        json!({
            "rule_id": "threshold.semantic.unexpected_mismatches_zero",
            "severity": "quarantine",
            "metric": "unexpected_mismatch_count",
            "operator": "eq",
            "value": 0,
            "reason": "unexpected cross-engine or oracle mismatches block assurance"
        }),
        json!({
            "rule_id": "threshold.semantic.failed_rows_zero",
            "severity": "quarantine",
            "metric": "failed_row_count",
            "operator": "eq",
            "value": 0,
            "reason": "failed oracle, conformance, or TLC rows block progression"
        }),
        json!({
            "rule_id": "threshold.semantic.declared_gaps_not_matches",
            "severity": "quarantine",
            "metric": "declared_gap_promoted_as_match_count",
            "operator": "eq",
            "value": 0,
            "reason": "declared gaps remain blockers until replay or differential evidence promotes them"
        }),
        json!({
            "rule_id": "threshold.history.minimum_window_rows",
            "severity": "quarantine",
            "metric": "history_window_row_count",
            "operator": "gte",
            "value": minimum_window_rows,
            "reason": minimum_window_reason
        }),
        json!({
            "rule_id": "threshold.service.no_simulated_service_promotion",
            "severity": "quarantine",
            "metric": "continuous_service_promoted",
            "operator": "eq",
            "value": false,
            "reason": "simulated multi-run evidence does not promote an operated assurance service"
        }),
        json!({
            "rule_id": "threshold.timing.measurement_only",
            "severity": "alert",
            "metric": "timing_regression",
            "operator": "report_only",
            "value": "no_correctness_consequence_without_semantic_failure",
            "reason": "timing can trigger investigation but cannot prove or disprove semantic correctness by itself"
        }),
    ]
}

fn quarantine_policy_rules(run_id: &str) -> Vec<Value> {
    let owner = if is_w037_run(run_id) {
        "calc-ubd.7"
    } else {
        "calc-rqq.7"
    };
    let mut rules = vec![
        json!({
            "rule_id": "quarantine.source_missing_artifact",
            "trigger": "any source evidence row has missing_artifact_count > 0",
            "action": "quarantine_run",
            "owner": owner
        }),
        json!({
            "rule_id": "quarantine.unexpected_mismatch",
            "trigger": "any TraceCalc, implementation-conformance, TLA, independent, or cross-engine row reports an unexpected mismatch",
            "action": "quarantine_run_and_open_blocker",
            "owner": owner
        }),
        json!({
            "rule_id": "quarantine.failed_semantic_row",
            "trigger": "any oracle, conformance, or TLC failed-row/config count is non-zero",
            "action": "quarantine_run_and_block_pack_reassessment",
            "owner": if is_w037_run(run_id) { "calc-ubd.7; calc-ubd.8" } else { "calc-rqq.7; calc-rqq.8" }
        }),
        json!({
            "rule_id": "quarantine.declared_gap_promoted_as_match",
            "trigger": "a declared gap is counted as a match without replay/diff evidence",
            "action": "quarantine_run_and_reopen_conformance_lane",
            "owner": if is_w037_run(run_id) { "calc-ubd.3; calc-ubd.7" } else { "calc-rqq.3; calc-rqq.7" }
        }),
        json!({
            "rule_id": "quarantine.unsupported_promotion_flag",
            "trigger": "full oracle, operated continuous service, pack C5, or Stage 2 policy is promoted without its required evidence bundle",
            "action": "quarantine_run_and_block_decision",
            "owner": if is_w037_run(run_id) { "calc-ubd.7; calc-ubd.8; calc-ubd.9" } else { "calc-rqq.7; calc-rqq.8; calc-rqq.9" }
        }),
        json!({
            "rule_id": "alert.oxfml_w073_formatting_payload_mismatch",
            "trigger": "an exercised conditional-formatting payload uses thresholds for W073 aggregate or visualization rule families instead of typed_rule",
            "action": "file_or_update_oxfml_handoff",
            "owner": if is_w037_run(run_id) { "calc-ubd.7; OxFml watch lane" } else { "calc-rqq.7; OxFml watch lane" }
        }),
        json!({
            "rule_id": "alert.timing_regression_only",
            "trigger": "timing changes while semantic thresholds pass",
            "action": "record_performance_alert_without_correctness_failure",
            "owner": owner
        }),
    ];
    if is_w037_run(run_id) {
        rules.push(json!({
            "rule_id": "quarantine.operated_service_claim_without_artifacts",
            "trigger": "an operated assurance or cross-engine service claim is made without recurring runner, retention, and enforcing alert artifacts",
            "action": "quarantine_run_and_block_service_promotion",
            "owner": "calc-ubd.7; calc-ubd.9"
        }));
    }
    rules
}

fn w037_service_readiness(
    run_id: &str,
    artifact_root: &str,
    evaluation: &Evaluation,
    history_window: &Value,
    regression_thresholds: &Value,
    quarantine_policy: &Value,
) -> Value {
    let direct_oxfml_ready =
        source_row(evaluation, "w037_direct_oxfml_evaluator").is_some_and(|row| {
            text_at(row, "evidence_state")
                == "direct_oxfml_slice_present_with_let_lambda_and_w073_guard"
        });
    let criteria = vec![
        service_readiness_criterion(
            "readiness.history_window_present",
            "satisfied",
            "checked-in W034-W037 history window is retained and machine-readable",
        ),
        service_readiness_criterion(
            "readiness.regression_thresholds_present",
            "satisfied",
            "semantic-first regression thresholds are retained",
        ),
        service_readiness_criterion(
            "readiness.quarantine_policy_defined",
            "satisfied",
            "quarantine and alert policy is machine-readable",
        ),
        service_readiness_criterion(
            "readiness.source_artifacts_retained",
            if evaluation.missing_artifacts.is_empty() {
                "satisfied"
            } else {
                "blocked"
            },
            "all required predecessor and W037 source artifacts are present",
        ),
        service_readiness_criterion(
            "readiness.unexpected_mismatches_zero",
            if evaluation.unexpected_mismatches.is_empty() {
                "satisfied"
            } else {
                "blocked"
            },
            "TraceCalc, TreeCalc/CoreEngine, OxFml, proof/model, and Stage 2 source rows report no unexpected semantic mismatches",
        ),
        service_readiness_criterion(
            "readiness.direct_oxfml_guard_present",
            if direct_oxfml_ready {
                "satisfied"
            } else {
                "blocked"
            },
            "direct OxFml evaluator fixture slice includes LET/LAMBDA and W073 typed conditional-formatting guard evidence",
        ),
        service_readiness_criterion(
            "service.operated_regression_runner",
            "blocked",
            "no recurring operated regression runner, retention service, or run scheduler is present",
        ),
        service_readiness_criterion(
            "service.enforcing_alert_dispatcher",
            "blocked",
            "quarantine policy is not enforced by an alert dispatcher",
        ),
        service_readiness_criterion(
            "service.operated_cross_engine_differential",
            "blocked",
            "cross-engine differential evidence is file-backed, not an operated service",
        ),
        service_readiness_criterion(
            "service.fully_independent_evaluator",
            "blocked",
            "W037 still has no qualifying fully independent evaluator implementation",
        ),
    ];
    let blocked_criteria_count = count_criteria_state(&criteria, "blocked");
    let satisfied_criteria_count = count_criteria_state(&criteria, "satisfied");

    json!({
        "schema_version": SERVICE_READINESS_SCHEMA_V1,
        "run_id": run_id,
        "artifact_root": artifact_root,
        "readiness_state": if evaluation.missing_artifacts.is_empty() && evaluation.unexpected_mismatches.is_empty() {
            "w037_service_readiness_inputs_present_without_operated_service_promotion"
        } else {
            "w037_service_readiness_has_source_gaps"
        },
        "criteria_count": criteria.len(),
        "satisfied_criteria_count": satisfied_criteria_count,
        "blocked_criteria_count": blocked_criteria_count,
        "history_window_row_count": number_at(history_window, "row_count"),
        "regression_threshold_count": number_at(regression_thresholds, "rule_count"),
        "quarantine_rule_count": number_at(quarantine_policy, "rule_count"),
        "missing_artifact_count": evaluation.missing_artifacts.len(),
        "unexpected_mismatch_count": evaluation.unexpected_mismatches.len(),
        "operated_continuous_assurance_service_promoted": false,
        "cross_engine_differential_service_promoted": false,
        "fully_independent_evaluator_promoted": false,
        "criteria": criteria,
    })
}

fn w037_cross_engine_service_pilot(
    run_id: &str,
    artifact_root: &str,
    evaluation: &Evaluation,
    service_readiness: &Value,
    history_window: &Value,
) -> Value {
    json!({
        "schema_version": CROSS_ENGINE_SERVICE_PILOT_SCHEMA_V1,
        "run_id": run_id,
        "artifact_root": artifact_root,
        "pilot_mode": "file_backed_cross_engine_service_readiness_packet",
        "operated_service_present": false,
        "operated_service_promoted": false,
        "alert_dispatcher_present": false,
        "continuous_cross_engine_service_promoted": false,
        "source_evidence_row_count": evaluation.source_rows.len(),
        "cross_engine_gate_row_count": evaluation.cross_engine_rows.len(),
        "history_window_row_count": number_at(history_window, "row_count"),
        "service_readiness_criteria_count": number_at(service_readiness, "criteria_count"),
        "service_readiness_blocked_count": number_at(service_readiness, "blocked_criteria_count"),
        "blocked_service_claims": [
            "operated_continuous_assurance_service",
            "enforcing_alert_dispatcher",
            "operated_cross_engine_differential_service",
            "fully_independent_evaluator_implementation"
        ],
        "pilot_inputs": [
            "history/assurance_history_window.json",
            "thresholds/regression_thresholds.json",
            "alerts/quarantine_policy.json",
            "differentials/cross_engine_differential_gate.json",
            "service/service_readiness.json"
        ],
        "promotion_consequence": "The pilot records readiness inputs and service blockers only. It does not promote operated assurance, cross-engine service, C5, pack-grade replay, or Stage 2 policy."
    })
}

fn service_readiness_criterion(
    criterion_id: &str,
    state: &str,
    evidence_or_blocker: &str,
) -> Value {
    json!({
        "criterion_id": criterion_id,
        "state": state,
        "evidence_or_blocker": evidence_or_blocker,
    })
}

fn count_criteria_state(criteria: &[Value], state: &str) -> usize {
    criteria
        .iter()
        .filter(|criterion| criterion.get("state").and_then(Value::as_str) == Some(state))
        .count()
}

fn source_row<'a>(evaluation: &'a Evaluation, input_id: &str) -> Option<&'a Value> {
    evaluation
        .source_rows
        .iter()
        .find(|row| row.get("input_id").and_then(Value::as_str) == Some(input_id))
}

fn failure_count(row: &Value) -> u64 {
    row.get("failures")
        .and_then(Value::as_array)
        .map_or(0, |failures| failures.len() as u64)
}

fn no_promotion_reasons(run_id: &str) -> Vec<String> {
    if is_w037_run(run_id) {
        return [
            "continuous.no_operated_regression_runner".to_string(),
            "continuous.service_pilot_file_backed_not_daemon".to_string(),
            "continuous.quarantine_policy_not_enforced_by_alert_service".to_string(),
            "continuous.cross_engine_diff_service_not_operated".to_string(),
            "continuous.independent_evaluator_diversity_not_full".to_string(),
            "continuous.stage2_partition_replay_absent".to_string(),
            "continuous.stage2_differential_service_not_operated".to_string(),
            "continuous.pack_c5_not_promoted".to_string(),
            "continuous.performance_not_correctness_proof".to_string(),
            "continuous.formal_evidence_bounded_not_full_verification".to_string(),
            "continuous.full_verification_not_released".to_string(),
            "continuous.pack_grade_replay_governance_absent".to_string(),
        ]
        .into();
    }
    if is_w036_run(run_id) {
        return [
            "continuous.no_operated_regression_runner".to_string(),
            "continuous.simulated_history_window_not_running_service".to_string(),
            "continuous.quarantine_policy_not_enforced_by_alert_service".to_string(),
            "continuous.cross_engine_diff_service_not_operated".to_string(),
            "continuous.performance_not_correctness_proof".to_string(),
            "continuous.tracecalc_oracle_not_full_coverage".to_string(),
            "continuous.optimized_core_engine_conformance_not_full".to_string(),
            "continuous.independent_evaluator_diversity_not_full".to_string(),
            "continuous.pack_c5_not_promoted".to_string(),
            "continuous.stage2_scheduler_not_promoted".to_string(),
            "continuous.formal_evidence_bounded_not_full_verification".to_string(),
        ]
        .into();
    }

    [
        "continuous.no_scheduled_regression_runner".to_string(),
        "continuous.no_cross_engine_diff_service".to_string(),
        "continuous.no_history_window_for_regression_thresholds".to_string(),
        "continuous.no_alerting_or_quarantine_policy".to_string(),
        "continuous.performance_not_correctness_proof".to_string(),
        "continuous.independent_evaluator_diversity_not_full".to_string(),
        "continuous.pack_c5_not_promoted".to_string(),
        "continuous.stage2_scheduler_not_promoted".to_string(),
        "continuous.formal_evidence_bounded_not_full_verification".to_string(),
    ]
    .into()
}

fn is_w036_run(run_id: &str) -> bool {
    run_id.starts_with("w036-")
}

fn is_w037_run(run_id: &str) -> bool {
    run_id.starts_with("w037-")
}

fn is_operation_history_run(run_id: &str) -> bool {
    is_w036_run(run_id) || is_w037_run(run_id)
}

fn collect_failures(row: &Value, failures: &mut Vec<String>) {
    for failure in row
        .get("failures")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        if let Some(failure) = failure.as_str() {
            failures.push(format!("{}:{failure}", text_at(row, "row_id")));
        }
    }
}

fn count_failure_rows(rows: &[Value]) -> usize {
    rows.iter()
        .filter(|row| {
            row.get("failures")
                .and_then(Value::as_array)
                .is_some_and(|failures| !failures.is_empty())
        })
        .count()
}

fn add_missing_if_absent(evaluation: &mut Evaluation, path: &str, value: &Option<Value>) {
    if value.is_none() {
        evaluation.missing_artifacts.push(path.to_string());
    }
}

fn read_json(
    repo_root: &Path,
    relative_path: &str,
) -> Result<Option<Value>, ContinuousAssuranceError> {
    let path = repo_root.join(relative_path);
    if !path.exists() {
        return Ok(None);
    }
    let content =
        fs::read_to_string(&path).map_err(|source| ContinuousAssuranceError::ReadArtifact {
            path: path.display().to_string(),
            source,
        })?;
    serde_json::from_str(&content)
        .map(Some)
        .map_err(|source| ContinuousAssuranceError::ParseJson {
            path: path.display().to_string(),
            source,
        })
}

fn write_json(path: &Path, value: &Value) -> Result<(), ContinuousAssuranceError> {
    let content = serde_json::to_string_pretty(value).expect("JSON serialization should succeed");
    fs::write(path, format!("{content}\n")).map_err(|source| ContinuousAssuranceError::WriteFile {
        path: path.display().to_string(),
        source,
    })
}

fn create_directory(path: &Path) -> Result<(), ContinuousAssuranceError> {
    fs::create_dir_all(path).map_err(|source| ContinuousAssuranceError::CreateDirectory {
        path: path.display().to_string(),
        source,
    })
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

fn relative_artifact_path<'a>(segments: impl IntoIterator<Item = &'a str>) -> String {
    let parts = segments.into_iter().collect::<Vec<_>>();
    historical_w038_w045_artifact_path(&parts).unwrap_or_else(|| parts.join("/"))
}

fn historical_w038_w045_artifact_path(parts: &[&str]) -> Option<String> {
    if parts.len() >= 5
        && parts[0] == "docs"
        && parts[1] == "test-runs"
        && parts[2] == "core-engine"
        && matches!(
            parts[4].get(..4),
            Some("w038" | "w039" | "w040" | "w041" | "w042" | "w043" | "w044" | "w045")
        )
    {
        let mut archived = vec!["archive", "test-runs-core-engine-w038-w045"];
        archived.extend_from_slice(&parts[3..]);
        Some(archived.join("/"))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    #[test]
    fn continuous_assurance_runner_writes_gate_without_promotion() {
        let repo_root = unique_temp_repo();
        create_source_artifacts(&repo_root);

        let summary = ContinuousAssuranceRunner::new()
            .execute(&repo_root, "w035-continuous-test")
            .expect("continuous assurance packet should write");

        assert_eq!(
            summary.decision_status,
            "continuous_assurance_gate_defined_without_promotion"
        );
        assert_eq!(summary.source_evidence_row_count, 5);
        assert_eq!(summary.scheduled_lane_count, 3);
        assert_eq!(summary.cross_engine_gate_row_count, 4);
        assert_eq!(summary.missing_artifact_count, 0);
        assert_eq!(summary.unexpected_mismatch_count, 0);
        assert!(summary.no_promotion_reason_count > 0);
        assert_eq!(summary.history_window_row_count, 0);
        assert_eq!(summary.regression_threshold_count, 0);
        assert_eq!(summary.quarantine_rule_count, 0);
        assert_eq!(summary.simulated_multi_run_count, 0);
        assert_eq!(summary.service_readiness_criteria_count, 0);
        assert_eq!(summary.service_readiness_blocked_count, 0);
        assert!(!summary.cross_engine_service_pilot_present);
        assert!(!summary.continuous_service_promoted);

        let decision = read_required_json(
            &repo_root,
            "docs/test-runs/core-engine/continuous-assurance/w035-continuous-test/decision/continuous_assurance_decision.json",
        );
        assert_eq!(decision["continuous_scale_assurance_promoted"], false);
        assert_eq!(decision["pack_capability_promoted"], false);
        assert_eq!(decision["stage2_scheduler_promoted"], false);

        let validation = read_required_json(
            &repo_root,
            "docs/test-runs/core-engine/continuous-assurance/w035-continuous-test/replay-appliance/validation/bundle_validation.json",
        );
        assert_eq!(validation["status"], "bundle_valid");

        fs::remove_dir_all(repo_root.parent().unwrap()).unwrap();
    }

    #[test]
    fn continuous_assurance_runner_writes_w036_operation_history() {
        let repo_root = unique_temp_repo();
        create_source_artifacts(&repo_root);
        create_w036_source_artifacts(&repo_root);

        let summary = ContinuousAssuranceRunner::new()
            .execute(&repo_root, "w036-continuous-test")
            .expect("W036 continuous assurance packet should write");

        assert_eq!(
            summary.decision_status,
            "w036_simulated_continuous_assurance_history_defined_without_service_promotion"
        );
        assert_eq!(summary.source_evidence_row_count, 11);
        assert_eq!(summary.scheduled_lane_count, 4);
        assert_eq!(summary.cross_engine_gate_row_count, 6);
        assert_eq!(summary.missing_artifact_count, 0);
        assert_eq!(summary.unexpected_mismatch_count, 0);
        assert_eq!(summary.history_window_row_count, 6);
        assert_eq!(summary.regression_threshold_count, 7);
        assert_eq!(summary.quarantine_rule_count, 7);
        assert_eq!(summary.simulated_multi_run_count, 6);
        assert_eq!(summary.service_readiness_criteria_count, 0);
        assert_eq!(summary.service_readiness_blocked_count, 0);
        assert!(!summary.cross_engine_service_pilot_present);
        assert!(!summary.continuous_service_promoted);

        let decision = read_required_json(
            &repo_root,
            "docs/test-runs/core-engine/continuous-assurance/w036-continuous-test/decision/continuous_assurance_decision.json",
        );
        assert_eq!(decision["simulated_history_window_present"], true);
        assert_eq!(
            decision["operated_continuous_assurance_service_promoted"],
            false
        );
        assert_eq!(decision["timing_is_correctness_evidence"], false);

        let history = read_required_json(
            &repo_root,
            "docs/test-runs/core-engine/continuous-assurance/w036-continuous-test/history/assurance_history_window.json",
        );
        assert_eq!(history["continuous_service_present"], false);
        assert_eq!(history["row_count"], 6);

        let thresholds = read_required_json(
            &repo_root,
            "docs/test-runs/core-engine/continuous-assurance/w036-continuous-test/thresholds/regression_thresholds.json",
        );
        assert_eq!(thresholds["semantic_first"], true);
        assert_eq!(thresholds["timing_can_quarantine"], false);

        let validation = read_required_json(
            &repo_root,
            "docs/test-runs/core-engine/continuous-assurance/w036-continuous-test/replay-appliance/validation/bundle_validation.json",
        );
        assert_eq!(validation["status"], "bundle_valid");

        fs::remove_dir_all(repo_root.parent().unwrap()).unwrap();
    }

    #[test]
    fn continuous_assurance_runner_writes_w037_service_pilot_without_promotion() {
        let repo_root = unique_temp_repo();
        create_source_artifacts(&repo_root);
        create_w036_source_artifacts(&repo_root);
        create_w037_source_artifacts(&repo_root);

        let summary = ContinuousAssuranceRunner::new()
            .execute(&repo_root, "w037-continuous-test")
            .expect("W037 continuous assurance service pilot packet should write");

        assert_eq!(
            summary.decision_status,
            "w037_operated_assurance_service_pilot_recorded_without_service_promotion"
        );
        assert_eq!(summary.source_evidence_row_count, 16);
        assert_eq!(summary.scheduled_lane_count, 5);
        assert_eq!(summary.cross_engine_gate_row_count, 9);
        assert_eq!(summary.missing_artifact_count, 0);
        assert_eq!(summary.unexpected_mismatch_count, 0);
        assert_eq!(summary.no_promotion_reason_count, 12);
        assert_eq!(summary.history_window_row_count, 11);
        assert_eq!(summary.regression_threshold_count, 7);
        assert_eq!(summary.quarantine_rule_count, 8);
        assert_eq!(summary.simulated_multi_run_count, 11);
        assert_eq!(summary.service_readiness_criteria_count, 10);
        assert_eq!(summary.service_readiness_blocked_count, 4);
        assert!(summary.cross_engine_service_pilot_present);
        assert!(!summary.continuous_service_promoted);

        let readiness = read_required_json(
            &repo_root,
            "docs/test-runs/core-engine/continuous-assurance/w037-continuous-test/service/service_readiness.json",
        );
        assert_eq!(
            readiness["readiness_state"],
            "w037_service_readiness_inputs_present_without_operated_service_promotion"
        );
        assert_eq!(readiness["criteria_count"], 10);
        assert_eq!(readiness["blocked_criteria_count"], 4);
        assert_eq!(
            readiness["operated_continuous_assurance_service_promoted"],
            false
        );
        assert_eq!(
            readiness["cross_engine_differential_service_promoted"],
            false
        );

        let pilot = read_required_json(
            &repo_root,
            "docs/test-runs/core-engine/continuous-assurance/w037-continuous-test/service/cross_engine_service_pilot.json",
        );
        assert_eq!(pilot["operated_service_present"], false);
        assert_eq!(pilot["continuous_cross_engine_service_promoted"], false);
        assert_eq!(pilot["service_readiness_blocked_count"], 4);

        let validation = read_required_json(
            &repo_root,
            "docs/test-runs/core-engine/continuous-assurance/w037-continuous-test/replay-appliance/validation/bundle_validation.json",
        );
        assert_eq!(validation["status"], "bundle_valid");
        assert_eq!(validation["service_readiness_criteria_count"], 10);
        assert_eq!(validation["service_readiness_blocked_count"], 4);
        assert_eq!(validation["cross_engine_service_pilot_present"], true);

        fs::remove_dir_all(repo_root.parent().unwrap()).unwrap();
    }

    fn unique_temp_repo() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let base = std::env::temp_dir().join(format!(
            "oxcalc-continuous-assurance-test-{}-{nanos}",
            std::process::id()
        ));
        let repo_root = base.join("OxCalc");
        fs::create_dir_all(&repo_root).unwrap();
        repo_root
    }

    fn create_source_artifacts(repo_root: &Path) {
        write_json_test(
            repo_root,
            &source_artifacts()[0],
            json!({
                "validated_scale_run_count": 7,
                "scale_signature_row_count": 5,
                "missing_artifact_count": 0,
                "unexpected_mismatch_count": 0,
            }),
        );
        write_json_test(
            repo_root,
            &source_artifacts()[1],
            json!({
                "continuous_scale_assurance_promoted": false,
            }),
        );
        write_json_test(
            repo_root,
            &source_artifacts()[2],
            json!({
                "highest_honest_capability": "cap.C4.distill_valid",
                "blocker_count": 12,
            }),
        );
        write_json_test(
            repo_root,
            &source_artifacts()[3],
            json!({
                "capability_promoted": false,
            }),
        );
        write_json_test(
            repo_root,
            &source_artifacts()[4],
            json!({
                "matrix_row_count": 17,
                "covered_row_count": 15,
                "classified_uncovered_row_count": 2,
                "missing_or_failed_row_count": 0,
            }),
        );
        write_json_test(
            repo_root,
            &source_artifacts()[5],
            json!({
                "gap_disposition_row_count": 6,
                "implementation_work_count": 5,
                "spec_evolution_deferral_count": 1,
                "failed_row_count": 0,
            }),
        );
        for artifact in W035_FORMAL_ARTIFACTS {
            write_text_test(repo_root, artifact, "W035 formal artifact\n");
        }
    }

    fn create_w036_source_artifacts(repo_root: &Path) {
        write_json_test(
            repo_root,
            &w036_source_artifacts()[0],
            json!({
                "source_evidence_row_count": 5,
                "scheduled_lane_count": 3,
                "cross_engine_gate_row_count": 4,
                "missing_artifact_count": 0,
                "unexpected_mismatch_count": 0,
            }),
        );
        write_json_test(
            repo_root,
            &w036_source_artifacts()[1],
            json!({
                "cross_engine_differential_service_promoted": false,
            }),
        );
        write_json_test(
            repo_root,
            &w036_source_artifacts()[2],
            json!({
                "matrix_row_count": 32,
                "covered_row_count": 30,
                "classified_uncovered_row_count": 1,
                "excluded_row_count": 1,
                "missing_or_failed_row_count": 0,
            }),
        );
        write_json_test(
            repo_root,
            &w036_source_artifacts()[3],
            json!({
                "full_oracle_claim": false,
            }),
        );
        write_json_test(
            repo_root,
            &w036_source_artifacts()[4],
            json!({
                "w036_action_row_count": 6,
                "w036_first_fix_row_count": 2,
                "w036_blocker_routed_row_count": 4,
                "w036_match_promoted_count": 0,
                "failed_row_count": 0,
            }),
        );
        write_json_test(
            repo_root,
            &w036_source_artifacts()[5],
            json!({
                "promoted_match_count": 0,
            }),
        );
        write_json_test(
            repo_root,
            &w036_source_artifacts()[6],
            json!({
                "config_count": 5,
                "passed_config_count": 5,
                "failed_config_count": 0,
            }),
        );
        write_json_test(
            repo_root,
            &w036_source_artifacts()[7],
            json!({
                "blocker_count": 5,
            }),
        );
        write_json_test(
            repo_root,
            &w036_source_artifacts()[8],
            json!({
                "comparison_row_count": 15,
                "w036_diversity_row_count": 5,
                "w036_promotion_blocker_count": 6,
                "missing_artifact_count": 0,
                "unexpected_mismatch_count": 0,
            }),
        );
        write_json_test(
            repo_root,
            &w036_source_artifacts()[9],
            json!({
                "differential_row_count": 6,
                "declared_gap_count": 2,
                "continuous_cross_engine_service_promoted": false,
                "full_independent_evaluator_promoted": false,
                "missing_artifact_count": 0,
                "unexpected_mismatch_count": 0,
            }),
        );
        write_json_test(
            repo_root,
            &w036_source_artifacts()[10],
            json!({
                "continuous_cross_engine_service_promoted": false,
                "full_independent_evaluator_promoted": false,
                "pack_grade_promoted": false,
                "stage2_policy_promoted": false,
            }),
        );
        for artifact in W036_FORMAL_ARTIFACTS {
            write_text_test(repo_root, artifact, "W036 formal artifact\n");
        }
    }

    fn create_w037_source_artifacts(repo_root: &Path) {
        write_json_test(
            repo_root,
            &w037_source_artifacts()[0],
            json!({
                "matrix_row_count": 32,
                "covered_row_count": 31,
                "classified_uncovered_row_count": 0,
                "excluded_row_count": 1,
                "missing_or_failed_row_count": 0,
            }),
        );
        write_json_test(
            repo_root,
            &w037_source_artifacts()[1],
            json!({
                "full_oracle_claim": false,
            }),
        );
        write_json_test(
            repo_root,
            &w037_source_artifacts()[2],
            json!({
                "w037_decision_row_count": 6,
                "w037_fixed_or_promoted_count": 1,
                "w037_residual_blocker_count": 5,
                "w037_match_promoted_count": 1,
                "failed_row_count": 0,
            }),
        );
        write_json_test(
            repo_root,
            &w037_source_artifacts()[3],
            json!({
                "promoted_match_count": 1,
            }),
        );
        write_json_test(
            repo_root,
            &w037_source_artifacts()[4],
            json!({
                "fixture_case_count": 12,
                "direct_oxfml_case_count": 3,
                "let_lambda_case_count": 2,
                "w073_typed_rule_case_count": 1,
                "expectation_mismatch_count": 0,
                "promotion_limits": {
                    "pack_grade_replay_promoted": false,
                    "c5_promoted": false,
                    "general_oxfunc_kernel_claimed": false
                }
            }),
        );
        write_json_test(
            repo_root,
            &w037_source_artifacts()[5],
            json!({
                "all_checked_artifacts_passed": true,
                "lean_file_count": 12,
                "lean_files_checked": 12,
                "lean_explicit_axiom_count": 0,
                "lean_sorry_placeholder_count": 0,
                "tla_routine_config_count": 11,
                "tla_failed_config_count": 0,
            }),
        );
        write_json_test(
            repo_root,
            &w037_source_artifacts()[6],
            json!({
                "validation_state": "w037_proof_model_closure_inventory_validated",
            }),
        );
        write_json_test(
            repo_root,
            &w037_source_artifacts()[7],
            json!({
                "criteria_row_count": 7,
                "satisfied_criteria_row_count": 3,
                "blocked_criteria_row_count": 4,
                "stage2_policy_promoted": false,
                "stage2_promotion_candidate": false,
            }),
        );
        write_json_test(
            repo_root,
            &w037_source_artifacts()[8],
            json!({
                "stage2_policy_promoted": false,
            }),
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
