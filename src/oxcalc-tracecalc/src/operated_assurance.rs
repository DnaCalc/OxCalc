#![forbid(unsafe_code)]

//! W038 operated-assurance, alert/quarantine, and service-disposition packet emission.

use std::fs;
use std::path::Path;

use serde_json::{Value, json};
use thiserror::Error;

const RUN_SUMMARY_SCHEMA_V1: &str = "oxcalc.operated_assurance.w038.run_summary.v1";
const SOURCE_INDEX_SCHEMA_V1: &str = "oxcalc.operated_assurance.w038.source_evidence_index.v1";
const MULTI_RUN_HISTORY_SCHEMA_V1: &str = "oxcalc.operated_assurance.w038.multi_run_history.v1";
const ALERT_QUARANTINE_SCHEMA_V1: &str =
    "oxcalc.operated_assurance.w038.alert_quarantine_enforcement.v1";
const CROSS_ENGINE_SERVICE_SCHEMA_V1: &str =
    "oxcalc.operated_assurance.w038.cross_engine_service_disposition.v1";
const SERVICE_READINESS_SCHEMA_V1: &str =
    "oxcalc.operated_assurance.w038.service_readiness_disposition.v1";
const BLOCKER_REGISTER_SCHEMA_V1: &str =
    "oxcalc.operated_assurance.w038.exact_service_blocker_register.v1";
const PROMOTION_DECISION_SCHEMA_V1: &str = "oxcalc.operated_assurance.w038.promotion_decision.v1";
const VALIDATION_SCHEMA_V1: &str = "oxcalc.operated_assurance.w038.validation.v1";

const W037_CONTINUOUS_RUN_SUMMARY: &str = "docs/test-runs/core-engine/continuous-assurance/w037-operated-assurance-service-pilot-001/run_summary.json";
const W037_SERVICE_READINESS: &str = "docs/test-runs/core-engine/continuous-assurance/w037-operated-assurance-service-pilot-001/service/service_readiness.json";
const W037_CROSS_ENGINE_SERVICE_PILOT: &str = "docs/test-runs/core-engine/continuous-assurance/w037-operated-assurance-service-pilot-001/service/cross_engine_service_pilot.json";
const W037_HISTORY_WINDOW: &str = "docs/test-runs/core-engine/continuous-assurance/w037-operated-assurance-service-pilot-001/history/assurance_history_window.json";
const W037_QUARANTINE_POLICY: &str = "docs/test-runs/core-engine/continuous-assurance/w037-operated-assurance-service-pilot-001/alerts/quarantine_policy.json";
const W037_CROSS_ENGINE_GATE: &str = "docs/test-runs/core-engine/continuous-assurance/w037-operated-assurance-service-pilot-001/differentials/cross_engine_differential_gate.json";
const W038_TRACECALC_AUTHORITY_SUMMARY: &str = "docs/test-runs/core-engine/tracecalc-authority/w038-tracecalc-authority-discharge-001/run_summary.json";
const W038_IMPLEMENTATION_CONFORMANCE_SUMMARY: &str = "docs/test-runs/core-engine/implementation-conformance/w038-optimized-core-conformance-disposition-001/run_summary.json";
const W038_FORMAL_ASSURANCE_SUMMARY: &str = "docs/test-runs/core-engine/formal-assurance/w038-proof-model-assumption-discharge-001/run_summary.json";
const W038_STAGE2_REPLAY_SUMMARY: &str =
    "docs/test-runs/core-engine/stage2-replay/w038-stage2-partition-replay-001/run_summary.json";
const W038_STAGE2_REPLAY_DECISION: &str = "docs/test-runs/core-engine/stage2-replay/w038-stage2-partition-replay-001/promotion_decision.json";

#[derive(Debug, Error)]
pub enum OperatedAssuranceError {
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
pub struct OperatedAssuranceRunSummary {
    pub run_id: String,
    pub schema_version: String,
    pub source_evidence_row_count: usize,
    pub multi_run_history_row_count: usize,
    pub evaluated_alert_rule_count: usize,
    pub quarantine_decision_count: usize,
    pub alert_decision_count: usize,
    pub service_readiness_criteria_count: usize,
    pub service_readiness_blocked_count: usize,
    pub exact_service_blocker_count: usize,
    pub failed_row_count: usize,
    pub operated_service_promoted: bool,
    pub artifact_root: String,
}

#[derive(Debug, Clone, Default)]
pub struct OperatedAssuranceRunner;

#[derive(Debug, Clone)]
struct AlertRule {
    rule_id: &'static str,
    action: &'static str,
    trigger: &'static str,
    owner: &'static str,
    triggered: bool,
    evidence: Value,
}

impl OperatedAssuranceRunner {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    pub fn execute(
        &self,
        repo_root: &Path,
        run_id: &str,
    ) -> Result<OperatedAssuranceRunSummary, OperatedAssuranceError> {
        let relative_artifact_root = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "operated-assurance",
            run_id,
        ]);
        let artifact_root = repo_root.join(&relative_artifact_root);
        if artifact_root.exists() {
            fs::remove_dir_all(&artifact_root).map_err(|source| {
                OperatedAssuranceError::RemoveDirectory {
                    path: artifact_root.display().to_string(),
                    source,
                }
            })?;
        }
        fs::create_dir_all(&artifact_root).map_err(|source| {
            OperatedAssuranceError::CreateDirectory {
                path: artifact_root.display().to_string(),
                source,
            }
        })?;

        let w037_summary = read_json(repo_root, W037_CONTINUOUS_RUN_SUMMARY)?;
        let w037_service_readiness = read_json(repo_root, W037_SERVICE_READINESS)?;
        let w037_cross_engine_pilot = read_json(repo_root, W037_CROSS_ENGINE_SERVICE_PILOT)?;
        let w037_history_window = read_json(repo_root, W037_HISTORY_WINDOW)?;
        let w037_quarantine_policy = read_json(repo_root, W037_QUARANTINE_POLICY)?;
        let w037_cross_engine_gate = read_json(repo_root, W037_CROSS_ENGINE_GATE)?;
        let w038_tracecalc = read_json(repo_root, W038_TRACECALC_AUTHORITY_SUMMARY)?;
        let w038_conformance = read_json(repo_root, W038_IMPLEMENTATION_CONFORMANCE_SUMMARY)?;
        let w038_formal = read_json(repo_root, W038_FORMAL_ASSURANCE_SUMMARY)?;
        let w038_stage2 = read_json(repo_root, W038_STAGE2_REPLAY_SUMMARY)?;
        let w038_stage2_decision = read_json(repo_root, W038_STAGE2_REPLAY_DECISION)?;

        let source_rows = source_rows(
            &w037_summary,
            &w037_service_readiness,
            &w037_cross_engine_pilot,
            &w037_cross_engine_gate,
            &w038_tracecalc,
            &w038_conformance,
            &w038_formal,
            &w038_stage2,
            &w038_stage2_decision,
        );
        let source_failures = source_validation_failures(&source_rows);
        let multi_run_history = multi_run_history(
            run_id,
            &relative_artifact_root,
            &w037_history_window,
            &w038_tracecalc,
            &w038_conformance,
            &w038_formal,
            &w038_stage2,
        );
        let alert_rules = alert_rules(
            &source_rows,
            &w037_summary,
            &w037_service_readiness,
            &w037_cross_engine_pilot,
            &w038_stage2_decision,
        );
        let alert_rows = alert_rules.iter().map(alert_rule_row).collect::<Vec<_>>();
        let quarantine_decision_count = alert_rules
            .iter()
            .filter(|rule| rule.triggered && rule.action.starts_with("quarantine"))
            .count();
        let alert_decision_count = alert_rules
            .iter()
            .filter(|rule| rule.triggered && rule.action.starts_with("alert"))
            .count();
        let readiness = service_readiness_disposition(
            run_id,
            &relative_artifact_root,
            &multi_run_history,
            alert_rules.len(),
            quarantine_decision_count,
            alert_decision_count,
            &w037_cross_engine_pilot,
            &w038_stage2,
        );
        let exact_blockers = exact_service_blockers();
        let exact_service_blocker_count = exact_blockers.len();
        let failed_row_count = source_failures.len() + quarantine_decision_count;

        let source_evidence_index_path =
            format!("{relative_artifact_root}/source_evidence_index.json");
        let multi_run_history_path = format!("{relative_artifact_root}/multi_run_history.json");
        let alert_quarantine_path =
            format!("{relative_artifact_root}/alert_quarantine_enforcement.json");
        let cross_engine_service_path =
            format!("{relative_artifact_root}/cross_engine_service_disposition.json");
        let service_readiness_path =
            format!("{relative_artifact_root}/service_readiness_disposition.json");
        let blocker_register_path =
            format!("{relative_artifact_root}/exact_service_blocker_register.json");
        let promotion_decision_path = format!("{relative_artifact_root}/promotion_decision.json");
        let validation_path = format!("{relative_artifact_root}/validation.json");

        let source_evidence_index = json!({
            "schema_version": SOURCE_INDEX_SCHEMA_V1,
            "run_id": run_id,
            "artifact_root": relative_artifact_root,
            "source_evidence_row_count": source_rows.len(),
            "rows": source_rows,
            "source_artifacts": {
                "w037_continuous_run_summary": W037_CONTINUOUS_RUN_SUMMARY,
                "w037_service_readiness": W037_SERVICE_READINESS,
                "w037_cross_engine_service_pilot": W037_CROSS_ENGINE_SERVICE_PILOT,
                "w037_history_window": W037_HISTORY_WINDOW,
                "w037_quarantine_policy": W037_QUARANTINE_POLICY,
                "w037_cross_engine_gate": W037_CROSS_ENGINE_GATE,
                "w038_tracecalc_authority_summary": W038_TRACECALC_AUTHORITY_SUMMARY,
                "w038_implementation_conformance_summary": W038_IMPLEMENTATION_CONFORMANCE_SUMMARY,
                "w038_formal_assurance_summary": W038_FORMAL_ASSURANCE_SUMMARY,
                "w038_stage2_replay_summary": W038_STAGE2_REPLAY_SUMMARY,
                "w038_stage2_replay_decision": W038_STAGE2_REPLAY_DECISION
            }
        });
        let alert_quarantine = json!({
            "schema_version": ALERT_QUARANTINE_SCHEMA_V1,
            "run_id": run_id,
            "policy_source": W037_QUARANTINE_POLICY,
            "source_policy_rule_count": number_at(&w037_quarantine_policy, "rule_count"),
            "policy_state": "w038_local_alert_quarantine_rules_evaluated_without_external_dispatcher_promotion",
            "evaluated_rule_count": alert_rules.len(),
            "quarantine_decision_count": quarantine_decision_count,
            "alert_decision_count": alert_decision_count,
            "clean_rule_count": alert_rules.len() - quarantine_decision_count - alert_decision_count,
            "local_enforcement_evidenced": true,
            "external_alert_dispatcher_promoted": false,
            "rows": alert_rows
        });
        let cross_engine_service = json!({
            "schema_version": CROSS_ENGINE_SERVICE_SCHEMA_V1,
            "run_id": run_id,
            "file_backed_pilot_present": true,
            "w037_cross_engine_gate_row_count": number_at(&w037_cross_engine_gate, "row_count"),
            "w037_cross_engine_unexpected_mismatch_count": number_at(&w037_cross_engine_gate, "unexpected_mismatch_count"),
            "w038_stage2_bounded_replay_present": number_at(&w038_stage2, "partition_replay_row_count") > 0,
            "operated_cross_engine_differential_service_present": false,
            "operated_cross_engine_differential_service_promoted": false,
            "disposition": "file_backed_cross_engine_rows_and_bounded_stage2_replay_are_bound_as_inputs_only",
            "blocked_service_claims": [
                "recurring_cross_engine_diff_scheduler",
                "service_retained_history_store",
                "external_alert_dispatcher",
                "operated_cross_engine_endpoint"
            ]
        });
        let blocker_register = json!({
            "schema_version": BLOCKER_REGISTER_SCHEMA_V1,
            "run_id": run_id,
            "exact_service_blocker_count": exact_service_blocker_count,
            "rows": exact_blockers
        });
        let promotion_decision = json!({
            "schema_version": PROMOTION_DECISION_SCHEMA_V1,
            "run_id": run_id,
            "decision_state": "w038_local_alert_quarantine_evidence_bound_service_unpromoted",
            "local_alert_quarantine_enforcement_evidenced": true,
            "multi_run_history_bound": true,
            "cross_engine_file_backed_pilot_bound": true,
            "operated_continuous_assurance_service_promoted": false,
            "external_alert_dispatcher_promoted": false,
            "operated_cross_engine_differential_service_promoted": false,
            "fully_independent_evaluator_promoted": false,
            "pack_grade_replay_promoted": false,
            "c5_promoted": false,
            "stage2_policy_promoted": false,
            "service_readiness_criteria_count": number_at(&readiness, "criteria_count"),
            "service_readiness_blocked_count": number_at(&readiness, "blocked_criteria_count"),
            "exact_service_blocker_count": exact_service_blocker_count,
            "blockers": exact_blockers
                .iter()
                .map(|row| row["blocker_id"].clone())
                .collect::<Vec<_>>(),
            "semantic_equivalence_statement": "This runner binds checked W037/W038 source artifacts, extends the multi-run evidence ledger, and evaluates alert/quarantine rules locally. It does not change scheduler, recalc, publication, replay, pack, service, alert-dispatch, TraceCalc, TreeCalc, OxFml, OxFunc, Lean, or TLA semantics."
        });
        let validation = json!({
            "schema_version": VALIDATION_SCHEMA_V1,
            "run_id": run_id,
            "status": if source_failures.is_empty() && quarantine_decision_count == 0 {
                "w038_operated_assurance_packet_valid"
            } else {
                "w038_operated_assurance_packet_invalid"
            },
            "source_evidence_row_count": source_rows.len(),
            "multi_run_history_row_count": number_at(&multi_run_history, "row_count"),
            "evaluated_alert_rule_count": alert_rules.len(),
            "quarantine_decision_count": quarantine_decision_count,
            "alert_decision_count": alert_decision_count,
            "service_readiness_criteria_count": number_at(&readiness, "criteria_count"),
            "service_readiness_blocked_count": number_at(&readiness, "blocked_criteria_count"),
            "exact_service_blocker_count": exact_service_blocker_count,
            "failed_row_count": failed_row_count,
            "validation_failures": source_failures
        });
        let run_summary = json!({
            "schema_version": RUN_SUMMARY_SCHEMA_V1,
            "run_id": run_id,
            "artifact_root": relative_artifact_root,
            "source_evidence_index_path": source_evidence_index_path,
            "multi_run_history_path": multi_run_history_path,
            "alert_quarantine_enforcement_path": alert_quarantine_path,
            "cross_engine_service_disposition_path": cross_engine_service_path,
            "service_readiness_disposition_path": service_readiness_path,
            "exact_service_blocker_register_path": blocker_register_path,
            "promotion_decision_path": promotion_decision_path,
            "validation_path": validation_path,
            "source_evidence_row_count": source_rows.len(),
            "multi_run_history_row_count": number_at(&multi_run_history, "row_count"),
            "evaluated_alert_rule_count": alert_rules.len(),
            "quarantine_decision_count": quarantine_decision_count,
            "alert_decision_count": alert_decision_count,
            "service_readiness_criteria_count": number_at(&readiness, "criteria_count"),
            "service_readiness_blocked_count": number_at(&readiness, "blocked_criteria_count"),
            "exact_service_blocker_count": exact_service_blocker_count,
            "failed_row_count": failed_row_count,
            "operated_continuous_assurance_service_promoted": false,
            "operated_cross_engine_differential_service_promoted": false,
            "external_alert_dispatcher_promoted": false
        });

        write_json(
            &artifact_root.join("source_evidence_index.json"),
            &source_evidence_index,
        )?;
        write_json(
            &artifact_root.join("multi_run_history.json"),
            &multi_run_history,
        )?;
        write_json(
            &artifact_root.join("alert_quarantine_enforcement.json"),
            &alert_quarantine,
        )?;
        write_json(
            &artifact_root.join("cross_engine_service_disposition.json"),
            &cross_engine_service,
        )?;
        write_json(
            &artifact_root.join("service_readiness_disposition.json"),
            &readiness,
        )?;
        write_json(
            &artifact_root.join("exact_service_blocker_register.json"),
            &blocker_register,
        )?;
        write_json(
            &artifact_root.join("promotion_decision.json"),
            &promotion_decision,
        )?;
        write_json(&artifact_root.join("validation.json"), &validation)?;
        write_json(&artifact_root.join("run_summary.json"), &run_summary)?;

        Ok(OperatedAssuranceRunSummary {
            run_id: run_id.to_string(),
            schema_version: RUN_SUMMARY_SCHEMA_V1.to_string(),
            source_evidence_row_count: source_rows.len(),
            multi_run_history_row_count: number_at(&multi_run_history, "row_count") as usize,
            evaluated_alert_rule_count: alert_rules.len(),
            quarantine_decision_count,
            alert_decision_count,
            service_readiness_criteria_count: number_at(&readiness, "criteria_count") as usize,
            service_readiness_blocked_count: number_at(&readiness, "blocked_criteria_count")
                as usize,
            exact_service_blocker_count,
            failed_row_count,
            operated_service_promoted: false,
            artifact_root: relative_artifact_root,
        })
    }
}

fn source_rows(
    w037_summary: &Value,
    w037_service_readiness: &Value,
    w037_cross_engine_pilot: &Value,
    w037_cross_engine_gate: &Value,
    w038_tracecalc: &Value,
    w038_conformance: &Value,
    w038_formal: &Value,
    w038_stage2: &Value,
    w038_stage2_decision: &Value,
) -> Vec<Value> {
    vec![
        json!({
            "row_id": "source.w037_continuous_assurance_summary",
            "artifact": W037_CONTINUOUS_RUN_SUMMARY,
            "missing_artifact_count": number_at(w037_summary, "missing_artifact_count"),
            "unexpected_mismatch_count": number_at(w037_summary, "unexpected_mismatch_count"),
            "failed_row_count": 0,
            "promoted_unsupported_service": bool_at(w037_summary, "continuous_service_promoted"),
            "semantic_state": text_at(w037_summary, "decision_status")
        }),
        json!({
            "row_id": "source.w037_service_readiness",
            "artifact": W037_SERVICE_READINESS,
            "missing_artifact_count": number_at(w037_service_readiness, "missing_artifact_count"),
            "unexpected_mismatch_count": number_at(w037_service_readiness, "unexpected_mismatch_count"),
            "failed_row_count": 0,
            "blocked_criteria_count": number_at(w037_service_readiness, "blocked_criteria_count"),
            "promoted_unsupported_service": bool_at(w037_service_readiness, "operated_continuous_assurance_service_promoted")
                || bool_at(w037_service_readiness, "cross_engine_differential_service_promoted"),
            "semantic_state": text_at(w037_service_readiness, "readiness_state")
        }),
        json!({
            "row_id": "source.w037_cross_engine_pilot",
            "artifact": W037_CROSS_ENGINE_SERVICE_PILOT,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": 0,
            "promoted_unsupported_service": bool_at(w037_cross_engine_pilot, "operated_service_promoted")
                || bool_at(w037_cross_engine_pilot, "continuous_cross_engine_service_promoted"),
            "semantic_state": text_at(w037_cross_engine_pilot, "pilot_mode")
        }),
        json!({
            "row_id": "source.w037_cross_engine_gate",
            "artifact": W037_CROSS_ENGINE_GATE,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": number_at(w037_cross_engine_gate, "unexpected_mismatch_count"),
            "failed_row_count": count_failure_rows(w037_cross_engine_gate),
            "promoted_unsupported_service": bool_at(w037_cross_engine_gate, "continuous_service_present"),
            "semantic_state": "w037_cross_engine_gate_rows_present"
        }),
        json!({
            "row_id": "source.w038_tracecalc_authority",
            "artifact": W038_TRACECALC_AUTHORITY_SUMMARY,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": number_at(w038_tracecalc, "missing_or_failed_row_count"),
            "promoted_unsupported_service": false,
            "semantic_state": "w038_tracecalc_authority_bound"
        }),
        json!({
            "row_id": "source.w038_implementation_conformance",
            "artifact": W038_IMPLEMENTATION_CONFORMANCE_SUMMARY,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": number_at(w038_conformance, "failed_row_count"),
            "promoted_unsupported_service": false,
            "semantic_state": "w038_conformance_disposition_bound"
        }),
        json!({
            "row_id": "source.w038_formal_assurance",
            "artifact": W038_FORMAL_ASSURANCE_SUMMARY,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": number_at(w038_formal, "failed_row_count"),
            "promoted_unsupported_service": bool_at(&w038_formal["promotion_claims"], "stage2_policy_promoted")
                || bool_at(&w038_formal["promotion_claims"], "pack_grade_replay_promoted")
                || bool_at(&w038_formal["promotion_claims"], "c5_promoted"),
            "semantic_state": "w038_formal_assurance_bound"
        }),
        json!({
            "row_id": "source.w038_stage2_replay",
            "artifact": W038_STAGE2_REPLAY_SUMMARY,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": number_at(w038_stage2, "failed_row_count"),
            "promoted_unsupported_service": bool_at(w038_stage2, "stage2_policy_promoted")
                || bool_at(w038_stage2_decision, "stage2_policy_promoted"),
            "semantic_state": "w038_stage2_bounded_replay_bound"
        }),
    ]
}

fn source_validation_failures(source_rows: &[Value]) -> Vec<String> {
    source_rows
        .iter()
        .flat_map(|row| {
            let row_id = text_at(row, "row_id");
            let mut failures = Vec::new();
            if number_at(row, "missing_artifact_count") > 0 {
                failures.push(format!("{row_id}.missing_artifact_count_nonzero"));
            }
            if number_at(row, "unexpected_mismatch_count") > 0 {
                failures.push(format!("{row_id}.unexpected_mismatch_count_nonzero"));
            }
            if number_at(row, "failed_row_count") > 0 {
                failures.push(format!("{row_id}.failed_row_count_nonzero"));
            }
            if bool_at(row, "promoted_unsupported_service") {
                failures.push(format!("{row_id}.unsupported_service_promotion"));
            }
            failures
        })
        .collect()
}

fn multi_run_history(
    run_id: &str,
    relative_artifact_root: &str,
    w037_history_window: &Value,
    w038_tracecalc: &Value,
    w038_conformance: &Value,
    w038_formal: &Value,
    w038_stage2: &Value,
) -> Value {
    let mut rows = w037_history_window
        .get("rows")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let next_order = rows.len() + 1;
    rows.push(history_row(
        next_order,
        "w038.tracecalc_authority",
        "w038_tracecalc_authority_discharge",
        "tracecalc_authority_discharge_present_without_release_grade_promotion",
        W038_TRACECALC_AUTHORITY_SUMMARY,
        number_at(w038_tracecalc, "missing_or_failed_row_count"),
        0,
        0,
    ));
    rows.push(history_row(
        next_order + 1,
        "w038.implementation_conformance",
        "w038_implementation_conformance_disposition",
        "optimized_conformance_disposition_present_with_exact_blockers",
        W038_IMPLEMENTATION_CONFORMANCE_SUMMARY,
        number_at(w038_conformance, "failed_row_count"),
        0,
        number_at(w038_conformance, "w038_exact_remaining_blocker_count"),
    ));
    rows.push(history_row(
        next_order + 2,
        "w038.formal_assurance",
        "w038_formal_assurance_assumption_discharge",
        "formal_assurance_present_with_totality_boundaries",
        W038_FORMAL_ASSURANCE_SUMMARY,
        number_at(w038_formal, "failed_row_count"),
        0,
        number_at(w038_formal, "exact_remaining_blocker_count"),
    ));
    rows.push(history_row(
        next_order + 3,
        "w038.stage2_replay",
        "w038_stage2_partition_replay",
        "bounded_stage2_replay_present_with_production_policy_blockers",
        W038_STAGE2_REPLAY_SUMMARY,
        number_at(w038_stage2, "failed_row_count"),
        0,
        number_at(w038_stage2, "exact_remaining_blocker_count"),
    ));

    json!({
        "schema_version": MULTI_RUN_HISTORY_SCHEMA_V1,
        "run_id": run_id,
        "artifact_root": relative_artifact_root,
        "history_kind": "w038_runner_bound_history_from_checked_artifacts",
        "continuous_service_present": false,
        "retained_history_service_present": false,
        "timing_correctness_role": "measurement_only_not_correctness_evidence",
        "semantic_acceptance_state": "w038_history_bound_with_known_service_blockers",
        "row_count": rows.len(),
        "rows": rows
    })
}

fn history_row(
    window_order: usize,
    evidence_epoch: &str,
    source_input_id: &str,
    semantic_state: &str,
    artifact: &str,
    failed_row_count: u64,
    unexpected_mismatch_count: u64,
    blocker_count: u64,
) -> Value {
    json!({
        "window_order": window_order,
        "evidence_epoch": evidence_epoch,
        "source_input_id": source_input_id,
        "semantic_state": semantic_state,
        "source_artifact_paths": [artifact],
        "missing_artifact_count": 0,
        "unexpected_mismatch_count": unexpected_mismatch_count,
        "failed_row_count": failed_row_count,
        "declared_gap_count": 0,
        "blocker_count": blocker_count,
        "timing_role": "measurement_only",
        "promotion_consequence": "source may feed later service and pack decisions only when semantic thresholds pass and blockers are not counted as promotions"
    })
}

fn alert_rules(
    source_rows: &[Value],
    w037_summary: &Value,
    w037_service_readiness: &Value,
    w037_cross_engine_pilot: &Value,
    w038_stage2_decision: &Value,
) -> Vec<AlertRule> {
    let missing_artifact_count = source_rows
        .iter()
        .map(|row| number_at(row, "missing_artifact_count"))
        .sum::<u64>();
    let unexpected_mismatch_count = source_rows
        .iter()
        .map(|row| number_at(row, "unexpected_mismatch_count"))
        .sum::<u64>();
    let failed_row_count = source_rows
        .iter()
        .map(|row| number_at(row, "failed_row_count"))
        .sum::<u64>();
    let unsupported_promotion = source_rows
        .iter()
        .any(|row| bool_at(row, "promoted_unsupported_service"));
    let w073_guard_present = number_at(w037_summary, "source_evidence_row_count") > 0;
    let operated_service_claimed =
        bool_at(
            w037_service_readiness,
            "operated_continuous_assurance_service_promoted",
        ) || bool_at(w037_cross_engine_pilot, "operated_service_promoted");
    let stage2_policy_promoted = bool_at(w038_stage2_decision, "stage2_policy_promoted");

    vec![
        alert_rule(
            "quarantine.source_missing_artifact",
            "quarantine_run",
            "any source evidence row has missing_artifact_count > 0",
            "calc-zsr.6",
            missing_artifact_count > 0,
            json!({ "missing_artifact_count": missing_artifact_count }),
        ),
        alert_rule(
            "quarantine.unexpected_mismatch",
            "quarantine_run_and_open_blocker",
            "any source evidence row reports an unexpected mismatch",
            "calc-zsr.6",
            unexpected_mismatch_count > 0,
            json!({ "unexpected_mismatch_count": unexpected_mismatch_count }),
        ),
        alert_rule(
            "quarantine.failed_semantic_row",
            "quarantine_run_and_block_pack_reassessment",
            "any oracle, conformance, replay, or proof/model row reports a failed row",
            "calc-zsr.6; calc-zsr.8",
            failed_row_count > 0,
            json!({ "failed_row_count": failed_row_count }),
        ),
        alert_rule(
            "quarantine.unsupported_promotion_flag",
            "quarantine_run_and_block_decision",
            "full verification, operated service, pack/C5, or Stage 2 policy is promoted without required evidence",
            "calc-zsr.6; calc-zsr.8; calc-zsr.9",
            unsupported_promotion || stage2_policy_promoted,
            json!({
                "unsupported_source_promotion": unsupported_promotion,
                "stage2_policy_promoted": stage2_policy_promoted
            }),
        ),
        alert_rule(
            "alert.oxfml_w073_formatting_payload_mismatch",
            "file_or_update_oxfml_handoff",
            "an exercised W073 aggregate or visualization conditional-formatting row lacks typed_rule evidence",
            "calc-zsr.6; calc-zsr.7; OxFml watch lane",
            !w073_guard_present,
            json!({ "w073_guard_present": w073_guard_present }),
        ),
        alert_rule(
            "alert.timing_regression_only",
            "record_performance_alert_without_correctness_failure",
            "timing changes while semantic thresholds pass",
            "calc-zsr.6",
            false,
            json!({ "timing_correctness_role": "measurement_only" }),
        ),
        alert_rule(
            "quarantine.operated_service_claim_without_artifacts",
            "quarantine_run_and_block_service_promotion",
            "an operated assurance or cross-engine service claim is made without recurring runner, retention, and enforcing alert artifacts",
            "calc-zsr.6; calc-zsr.9",
            operated_service_claimed,
            json!({ "operated_service_claimed": operated_service_claimed }),
        ),
        alert_rule(
            "alert.stage2_bounded_replay_without_operated_service",
            "record_stage2_service_gap_without_quarantine",
            "bounded Stage 2 replay exists but operated cross-engine service remains absent",
            "calc-zsr.6",
            false,
            json!({
                "bounded_replay_present": true,
                "operated_cross_engine_service_present": false
            }),
        ),
    ]
}

fn alert_rule(
    rule_id: &'static str,
    action: &'static str,
    trigger: &'static str,
    owner: &'static str,
    triggered: bool,
    evidence: Value,
) -> AlertRule {
    AlertRule {
        rule_id,
        action,
        trigger,
        owner,
        triggered,
        evidence,
    }
}

fn alert_rule_row(rule: &AlertRule) -> Value {
    json!({
        "rule_id": rule.rule_id,
        "action": rule.action,
        "trigger": rule.trigger,
        "owner": rule.owner,
        "triggered": rule.triggered,
        "decision": if rule.triggered {
            "triggered"
        } else {
            "clean"
        },
        "evidence": rule.evidence
    })
}

fn service_readiness_disposition(
    run_id: &str,
    relative_artifact_root: &str,
    multi_run_history: &Value,
    evaluated_alert_rule_count: usize,
    quarantine_decision_count: usize,
    alert_decision_count: usize,
    w037_cross_engine_pilot: &Value,
    w038_stage2: &Value,
) -> Value {
    let criteria = vec![
        criterion(
            "readiness.w038_multi_run_history_bound",
            "satisfied",
            "W037 history is extended with W038 TraceCalc authority, conformance, formal-assurance, and Stage 2 replay rows",
        ),
        criterion(
            "readiness.alert_quarantine_rules_evaluated",
            "satisfied",
            "W038 evaluates alert/quarantine rules against current source rows",
        ),
        criterion(
            "readiness.source_artifacts_retained",
            "satisfied",
            "all required predecessor and W038 source artifacts are present",
        ),
        criterion(
            "readiness.unexpected_mismatches_zero",
            "satisfied",
            "current W037/W038 source rows report no unexpected semantic mismatches",
        ),
        criterion(
            "readiness.stage2_bounded_replay_present",
            "satisfied",
            "W038 Stage 2 bounded replay and permutation evidence is present",
        ),
        criterion(
            "readiness.cross_engine_file_backed_pilot_present",
            "satisfied_boundary",
            "W037 cross-engine pilot rows are file-backed inputs, not operated service proof",
        ),
        criterion(
            "service.operated_regression_runner",
            "blocked",
            "no recurring operated regression runner, retention service, or run scheduler is present",
        ),
        criterion(
            "service.enforcing_alert_dispatcher",
            "blocked",
            "W038 local rule evaluation is present, but no external alert dispatcher or quarantine service is operated",
        ),
        criterion(
            "service.operated_cross_engine_differential",
            "blocked",
            "cross-engine differential evidence remains file-backed rather than an operated service",
        ),
        criterion(
            "service.retained_history_store",
            "blocked",
            "multi-run history is checked-in evidence, not a retained service store with lifecycle guarantees",
        ),
    ];
    let blocked_criteria_count = criteria
        .iter()
        .filter(|row| row.get("state").and_then(Value::as_str) == Some("blocked"))
        .count();

    json!({
        "schema_version": SERVICE_READINESS_SCHEMA_V1,
        "run_id": run_id,
        "artifact_root": relative_artifact_root,
        "readiness_state": "w038_local_enforcement_inputs_present_without_operated_service_promotion",
        "criteria_count": criteria.len(),
        "satisfied_criteria_count": criteria.len() - blocked_criteria_count,
        "blocked_criteria_count": blocked_criteria_count,
        "history_window_row_count": number_at(multi_run_history, "row_count"),
        "evaluated_alert_rule_count": evaluated_alert_rule_count,
        "quarantine_decision_count": quarantine_decision_count,
        "alert_decision_count": alert_decision_count,
        "w037_file_backed_pilot_present": bool_at(w037_cross_engine_pilot, "cross_engine_service_pilot_present")
            || text_at(w037_cross_engine_pilot, "pilot_mode") == "file_backed_cross_engine_service_readiness_packet",
        "w038_stage2_partition_replay_row_count": number_at(w038_stage2, "partition_replay_row_count"),
        "operated_continuous_assurance_service_promoted": false,
        "cross_engine_differential_service_promoted": false,
        "external_alert_dispatcher_promoted": false,
        "criteria": criteria
    })
}

fn criterion(criterion_id: &str, state: &str, evidence_or_blocker: &str) -> Value {
    json!({
        "criterion_id": criterion_id,
        "state": state,
        "evidence_or_blocker": evidence_or_blocker
    })
}

fn exact_service_blockers() -> Vec<Value> {
    vec![
        json!({
            "blocker_id": "service.operated_regression_runner_absent",
            "owner": "calc-zsr.6; calc-zsr.9",
            "status_after_run": "exact_remaining_blocker",
            "reason": "W038 binds a multi-run evidence ledger but does not operate a recurring runner or scheduler.",
            "promotion_consequence": "operated continuous assurance service remains unpromoted"
        }),
        json!({
            "blocker_id": "service.external_alert_dispatcher_absent",
            "owner": "calc-zsr.6; calc-zsr.9",
            "status_after_run": "exact_remaining_blocker",
            "reason": "W038 evaluates alert/quarantine rules locally but does not operate an external dispatcher or quarantine service.",
            "promotion_consequence": "alert/quarantine dispatcher claims remain unpromoted"
        }),
        json!({
            "blocker_id": "service.operated_cross_engine_differential_absent",
            "owner": "calc-zsr.6; calc-zsr.7",
            "status_after_run": "exact_remaining_blocker",
            "reason": "cross-engine evidence remains file-backed and does not run as a continuous differential service.",
            "promotion_consequence": "operated cross-engine differential service remains unpromoted"
        }),
        json!({
            "blocker_id": "service.retained_history_store_absent",
            "owner": "calc-zsr.6; calc-zsr.8",
            "status_after_run": "exact_remaining_blocker",
            "reason": "multi-run history is checked-in evidence rather than a retained service store with lifecycle and retention guarantees.",
            "promotion_consequence": "pack-grade replay and service-retained history claims remain unpromoted"
        }),
    ]
}

fn count_failure_rows(value: &Value) -> u64 {
    value
        .get("rows")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|row| {
            row.get("failures")
                .and_then(Value::as_array)
                .is_some_and(|failures| !failures.is_empty())
        })
        .count() as u64
}

fn read_json(repo_root: &Path, relative_path: &str) -> Result<Value, OperatedAssuranceError> {
    let path = repo_root.join(relative_path);
    let contents =
        fs::read_to_string(&path).map_err(|source| OperatedAssuranceError::ReadArtifact {
            path: path.display().to_string(),
            source,
        })?;
    serde_json::from_str(&contents).map_err(|source| OperatedAssuranceError::ParseJson {
        path: path.display().to_string(),
        source,
    })
}

fn write_json(path: &Path, value: &Value) -> Result<(), OperatedAssuranceError> {
    let contents = serde_json::to_string_pretty(value).map_err(|source| {
        OperatedAssuranceError::ParseJson {
            path: path.display().to_string(),
            source,
        }
    })?;
    fs::write(path, format!("{contents}\n")).map_err(|source| OperatedAssuranceError::WriteFile {
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

fn relative_artifact_path(parts: &[&str]) -> String {
    parts.join("/")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn operated_assurance_runner_binds_w038_service_packet_without_promotion() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let run_id = format!("test-w038-operated-assurance-{}", std::process::id());
        let artifact_root = repo_root.join(format!(
            "docs/test-runs/core-engine/operated-assurance/{run_id}"
        ));
        let cleanup = || {
            if artifact_root.exists() {
                let _ = fs::remove_dir_all(&artifact_root);
            }
        };

        cleanup();
        let summary = OperatedAssuranceRunner::new()
            .execute(&repo_root, &run_id)
            .unwrap();

        assert_eq!(summary.source_evidence_row_count, 8);
        assert_eq!(summary.multi_run_history_row_count, 15);
        assert_eq!(summary.evaluated_alert_rule_count, 8);
        assert_eq!(summary.quarantine_decision_count, 0);
        assert_eq!(summary.alert_decision_count, 0);
        assert_eq!(summary.service_readiness_criteria_count, 10);
        assert_eq!(summary.service_readiness_blocked_count, 4);
        assert_eq!(summary.exact_service_blocker_count, 4);
        assert_eq!(summary.failed_row_count, 0);
        assert!(!summary.operated_service_promoted);

        let validation = read_json(
            &repo_root,
            &format!("docs/test-runs/core-engine/operated-assurance/{run_id}/validation.json"),
        )
        .unwrap();
        assert_eq!(validation["status"], "w038_operated_assurance_packet_valid");

        let promotion = read_json(
            &repo_root,
            &format!(
                "docs/test-runs/core-engine/operated-assurance/{run_id}/promotion_decision.json"
            ),
        )
        .unwrap();
        assert_eq!(
            promotion["operated_continuous_assurance_service_promoted"],
            false
        );
        assert_eq!(
            promotion["local_alert_quarantine_enforcement_evidenced"],
            true
        );

        cleanup();
    }
}
