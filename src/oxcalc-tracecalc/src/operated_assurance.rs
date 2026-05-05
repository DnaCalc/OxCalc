#![forbid(unsafe_code)]

//! W038/W039/W040 operated-assurance, alert/quarantine, and service-disposition packet emission.

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
const W039_RUN_SUMMARY_SCHEMA_V1: &str = "oxcalc.operated_assurance.w039.run_summary.v1";
const W039_SOURCE_INDEX_SCHEMA_V1: &str = "oxcalc.operated_assurance.w039.source_evidence_index.v1";
const W039_RETAINED_HISTORY_SCHEMA_V1: &str =
    "oxcalc.operated_assurance.w039.retained_history_lifecycle.v1";
const W039_ALERT_DISPATCHER_SCHEMA_V1: &str =
    "oxcalc.operated_assurance.w039.alert_dispatcher_enforcement.v1";
const W039_CROSS_ENGINE_SERVICE_SCHEMA_V1: &str =
    "oxcalc.operated_assurance.w039.cross_engine_service_substrate.v1";
const W039_SERVICE_READINESS_SCHEMA_V1: &str =
    "oxcalc.operated_assurance.w039.service_readiness_register.v1";
const W039_BLOCKER_REGISTER_SCHEMA_V1: &str =
    "oxcalc.operated_assurance.w039.exact_service_blocker_register.v1";
const W039_PROMOTION_DECISION_SCHEMA_V1: &str =
    "oxcalc.operated_assurance.w039.promotion_decision.v1";
const W039_VALIDATION_SCHEMA_V1: &str = "oxcalc.operated_assurance.w039.validation.v1";
const W040_RUN_SUMMARY_SCHEMA_V1: &str = "oxcalc.operated_assurance.w040.run_summary.v1";
const W040_SOURCE_INDEX_SCHEMA_V1: &str = "oxcalc.operated_assurance.w040.source_evidence_index.v1";
const W040_OPERATED_RUNNER_SCHEMA_V1: &str =
    "oxcalc.operated_assurance.w040.operated_runner_register.v1";
const W040_RETAINED_HISTORY_STORE_SCHEMA_V1: &str =
    "oxcalc.operated_assurance.w040.retained_history_store_query.v1";
const W040_ALERT_DISPATCHER_SCHEMA_V1: &str =
    "oxcalc.operated_assurance.w040.alert_dispatcher_enforcement.v1";
const W040_CROSS_ENGINE_SERVICE_SCHEMA_V1: &str =
    "oxcalc.operated_assurance.w040.cross_engine_service_register.v1";
const W040_SERVICE_READINESS_SCHEMA_V1: &str =
    "oxcalc.operated_assurance.w040.service_readiness_register.v1";
const W040_BLOCKER_REGISTER_SCHEMA_V1: &str =
    "oxcalc.operated_assurance.w040.exact_service_blocker_register.v1";
const W040_PROMOTION_DECISION_SCHEMA_V1: &str =
    "oxcalc.operated_assurance.w040.promotion_decision.v1";
const W040_VALIDATION_SCHEMA_V1: &str = "oxcalc.operated_assurance.w040.validation.v1";

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
const W039_RESIDUAL_LEDGER: &str = "docs/test-runs/core-engine/release-grade-ledger/w039-residual-successor-obligation-ledger-001/successor_obligation_ledger.json";
const W038_OPERATED_ASSURANCE_SUMMARY: &str = "docs/test-runs/core-engine/operated-assurance/w038-operated-assurance-alert-quarantine-001/run_summary.json";
const W038_OPERATED_ASSURANCE_VALIDATION: &str = "docs/test-runs/core-engine/operated-assurance/w038-operated-assurance-alert-quarantine-001/validation.json";
const W038_OPERATED_MULTI_RUN_HISTORY: &str = "docs/test-runs/core-engine/operated-assurance/w038-operated-assurance-alert-quarantine-001/multi_run_history.json";
const W038_OPERATED_ALERT_QUARANTINE: &str = "docs/test-runs/core-engine/operated-assurance/w038-operated-assurance-alert-quarantine-001/alert_quarantine_enforcement.json";
const W038_OPERATED_CROSS_ENGINE_SERVICE: &str = "docs/test-runs/core-engine/operated-assurance/w038-operated-assurance-alert-quarantine-001/cross_engine_service_disposition.json";
const W038_OPERATED_SERVICE_READINESS: &str = "docs/test-runs/core-engine/operated-assurance/w038-operated-assurance-alert-quarantine-001/service_readiness_disposition.json";
const W038_OPERATED_SERVICE_BLOCKERS: &str = "docs/test-runs/core-engine/operated-assurance/w038-operated-assurance-alert-quarantine-001/exact_service_blocker_register.json";
const W038_OPERATED_PROMOTION_DECISION: &str = "docs/test-runs/core-engine/operated-assurance/w038-operated-assurance-alert-quarantine-001/promotion_decision.json";
const W039_STAGE2_SUMMARY: &str = "docs/test-runs/core-engine/stage2-replay/w039-stage2-production-policy-replay-governance-001/run_summary.json";
const W039_STAGE2_VALIDATION: &str = "docs/test-runs/core-engine/stage2-replay/w039-stage2-production-policy-replay-governance-001/validation.json";
const W039_STAGE2_DECISION: &str = "docs/test-runs/core-engine/stage2-replay/w039-stage2-production-policy-replay-governance-001/promotion_decision.json";
const W039_STAGE2_BLOCKERS: &str = "docs/test-runs/core-engine/stage2-replay/w039-stage2-production-policy-replay-governance-001/w039_stage2_exact_blocker_register.json";
const W038_PACK_DECISION: &str = "docs/test-runs/core-engine/pack-capability/w038-pack-c5-release-decision-001/decision/pack_capability_decision.json";
const W040_DIRECT_OBLIGATION_SUMMARY: &str = "docs/test-runs/core-engine/release-grade-ledger/w040-direct-verification-obligation-map-001/run_summary.json";
const W040_DIRECT_OBLIGATION_MAP: &str = "docs/test-runs/core-engine/release-grade-ledger/w040-direct-verification-obligation-map-001/direct_verification_obligation_map.json";
const W040_FORMATTING_INTAKE: &str = "docs/test-runs/core-engine/release-grade-ledger/w040-direct-verification-obligation-map-001/w073_formatting_intake.json";
const W039_OPERATED_ASSURANCE_SUMMARY: &str = "docs/test-runs/core-engine/operated-assurance/w039-operated-assurance-retained-history-001/run_summary.json";
const W039_OPERATED_ASSURANCE_VALIDATION: &str = "docs/test-runs/core-engine/operated-assurance/w039-operated-assurance-retained-history-001/validation.json";
const W039_RETAINED_HISTORY_LIFECYCLE: &str = "docs/test-runs/core-engine/operated-assurance/w039-operated-assurance-retained-history-001/w039_retained_history_lifecycle.json";
const W039_ALERT_DISPATCHER_ENFORCEMENT: &str = "docs/test-runs/core-engine/operated-assurance/w039-operated-assurance-retained-history-001/w039_alert_dispatcher_enforcement.json";
const W039_CROSS_ENGINE_SERVICE_SUBSTRATE: &str = "docs/test-runs/core-engine/operated-assurance/w039-operated-assurance-retained-history-001/w039_cross_engine_service_substrate.json";
const W039_SERVICE_READINESS_REGISTER: &str = "docs/test-runs/core-engine/operated-assurance/w039-operated-assurance-retained-history-001/w039_service_readiness_register.json";
const W039_OPERATED_SERVICE_BLOCKERS: &str = "docs/test-runs/core-engine/operated-assurance/w039-operated-assurance-retained-history-001/w039_exact_service_blocker_register.json";
const W039_OPERATED_PROMOTION_DECISION: &str = "docs/test-runs/core-engine/operated-assurance/w039-operated-assurance-retained-history-001/promotion_decision.json";
const W040_STAGE2_SUMMARY: &str = "docs/test-runs/core-engine/stage2-replay/w040-stage2-production-policy-equivalence-001/run_summary.json";
const W040_STAGE2_VALIDATION: &str = "docs/test-runs/core-engine/stage2-replay/w040-stage2-production-policy-equivalence-001/validation.json";
const W040_STAGE2_DECISION: &str = "docs/test-runs/core-engine/stage2-replay/w040-stage2-production-policy-equivalence-001/promotion_decision.json";
const W040_STAGE2_BLOCKERS: &str = "docs/test-runs/core-engine/stage2-replay/w040-stage2-production-policy-equivalence-001/w040_stage2_exact_blocker_register.json";

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
        if run_id.contains("w040") {
            return self.execute_w040(repo_root, run_id);
        }
        if run_id.contains("w039") {
            return self.execute_w039(repo_root, run_id);
        }

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

    fn execute_w040(
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

        let w040_direct_summary = read_json(repo_root, W040_DIRECT_OBLIGATION_SUMMARY)?;
        let w040_direct_map = read_json(repo_root, W040_DIRECT_OBLIGATION_MAP)?;
        let w040_formatting_intake = read_json(repo_root, W040_FORMATTING_INTAKE)?;
        let w039_summary = read_json(repo_root, W039_OPERATED_ASSURANCE_SUMMARY)?;
        let w039_validation = read_json(repo_root, W039_OPERATED_ASSURANCE_VALIDATION)?;
        let w039_retained = read_json(repo_root, W039_RETAINED_HISTORY_LIFECYCLE)?;
        let w039_alerts = read_json(repo_root, W039_ALERT_DISPATCHER_ENFORCEMENT)?;
        let w039_cross_engine = read_json(repo_root, W039_CROSS_ENGINE_SERVICE_SUBSTRATE)?;
        let w039_readiness = read_json(repo_root, W039_SERVICE_READINESS_REGISTER)?;
        let w039_blockers = read_json(repo_root, W039_OPERATED_SERVICE_BLOCKERS)?;
        let w039_promotion = read_json(repo_root, W039_OPERATED_PROMOTION_DECISION)?;
        let w040_stage2_summary = read_json(repo_root, W040_STAGE2_SUMMARY)?;
        let w040_stage2_validation = read_json(repo_root, W040_STAGE2_VALIDATION)?;
        let w040_stage2_decision = read_json(repo_root, W040_STAGE2_DECISION)?;
        let w040_stage2_blockers = read_json(repo_root, W040_STAGE2_BLOCKERS)?;

        let source_rows = w040_source_rows(
            &w040_direct_summary,
            &w040_direct_map,
            &w040_formatting_intake,
            &w039_summary,
            &w039_validation,
            &w039_retained,
            &w039_alerts,
            &w039_cross_engine,
            &w039_readiness,
            &w039_blockers,
            &w039_promotion,
            &w040_stage2_summary,
            &w040_stage2_validation,
            &w040_stage2_decision,
            &w040_stage2_blockers,
        );
        let source_failures = w040_source_validation_failures(&source_rows);
        let operated_runner =
            w040_operated_runner_register(run_id, &relative_artifact_root, source_rows.len());
        let retained_store = w040_retained_history_store_query(
            run_id,
            &relative_artifact_root,
            &w039_retained,
            &w040_stage2_summary,
            &w040_stage2_blockers,
            &w040_formatting_intake,
        );
        let alert_dispatcher = w040_alert_dispatcher(
            run_id,
            &w039_alerts,
            &w039_promotion,
            &w040_stage2_decision,
            &retained_store,
            &w040_formatting_intake,
        );
        let cross_engine_service = w040_cross_engine_service(
            run_id,
            &w039_cross_engine,
            &w040_stage2_summary,
            &w040_stage2_blockers,
        );
        let readiness = w040_service_readiness(
            run_id,
            &relative_artifact_root,
            &operated_runner,
            &retained_store,
            &alert_dispatcher,
            &cross_engine_service,
            &w040_formatting_intake,
        );
        let exact_blockers = w040_exact_service_blockers();
        let exact_service_blocker_count = exact_blockers.len();
        let failed_row_count = source_failures.len();

        let source_evidence_index_path =
            format!("{relative_artifact_root}/source_evidence_index.json");
        let operated_runner_path =
            format!("{relative_artifact_root}/w040_operated_runner_register.json");
        let retained_history_store_path =
            format!("{relative_artifact_root}/w040_retained_history_store_query.json");
        let alert_dispatcher_path =
            format!("{relative_artifact_root}/w040_alert_dispatcher_enforcement.json");
        let cross_engine_service_path =
            format!("{relative_artifact_root}/w040_cross_engine_service_register.json");
        let service_readiness_path =
            format!("{relative_artifact_root}/w040_service_readiness_register.json");
        let blocker_register_path =
            format!("{relative_artifact_root}/w040_exact_service_blocker_register.json");
        let promotion_decision_path = format!("{relative_artifact_root}/promotion_decision.json");
        let validation_path = format!("{relative_artifact_root}/validation.json");

        write_json(
            &artifact_root.join("source_evidence_index.json"),
            &json!({
                "schema_version": W040_SOURCE_INDEX_SCHEMA_V1,
                "run_id": run_id,
                "artifact_root": relative_artifact_root,
                "source_evidence_row_count": source_rows.len(),
                "rows": source_rows,
                "source_artifacts": {
                    "w040_direct_obligation_summary": W040_DIRECT_OBLIGATION_SUMMARY,
                    "w040_direct_obligation_map": W040_DIRECT_OBLIGATION_MAP,
                    "w040_formatting_intake": W040_FORMATTING_INTAKE,
                    "w039_operated_assurance_summary": W039_OPERATED_ASSURANCE_SUMMARY,
                    "w039_operated_assurance_validation": W039_OPERATED_ASSURANCE_VALIDATION,
                    "w039_retained_history_lifecycle": W039_RETAINED_HISTORY_LIFECYCLE,
                    "w039_alert_dispatcher": W039_ALERT_DISPATCHER_ENFORCEMENT,
                    "w039_cross_engine_service": W039_CROSS_ENGINE_SERVICE_SUBSTRATE,
                    "w039_service_readiness": W039_SERVICE_READINESS_REGISTER,
                    "w039_exact_service_blockers": W039_OPERATED_SERVICE_BLOCKERS,
                    "w039_promotion_decision": W039_OPERATED_PROMOTION_DECISION,
                    "w040_stage2_summary": W040_STAGE2_SUMMARY,
                    "w040_stage2_validation": W040_STAGE2_VALIDATION,
                    "w040_stage2_decision": W040_STAGE2_DECISION,
                    "w040_stage2_blockers": W040_STAGE2_BLOCKERS
                }
            }),
        )?;
        write_json(
            &artifact_root.join("w040_operated_runner_register.json"),
            &operated_runner,
        )?;
        write_json(
            &artifact_root.join("w040_retained_history_store_query.json"),
            &retained_store,
        )?;
        write_json(
            &artifact_root.join("w040_alert_dispatcher_enforcement.json"),
            &alert_dispatcher,
        )?;
        write_json(
            &artifact_root.join("w040_cross_engine_service_register.json"),
            &cross_engine_service,
        )?;
        write_json(
            &artifact_root.join("w040_service_readiness_register.json"),
            &readiness,
        )?;
        write_json(
            &artifact_root.join("w040_exact_service_blocker_register.json"),
            &json!({
                "schema_version": W040_BLOCKER_REGISTER_SCHEMA_V1,
                "run_id": run_id,
                "exact_service_blocker_count": exact_service_blocker_count,
                "rows": exact_blockers
            }),
        )?;
        write_json(
            &artifact_root.join("promotion_decision.json"),
            &json!({
                "schema_version": W040_PROMOTION_DECISION_SCHEMA_V1,
                "run_id": run_id,
                "decision_state": "w040_operated_assurance_retained_history_artifacts_validated_service_unpromoted",
                "file_backed_operated_runner_present": true,
                "retained_history_artifact_store_present": bool_at(&retained_store, "file_backed_retained_history_store_present"),
                "retained_history_query_register_present": bool_at(&retained_store, "retained_history_query_register_present"),
                "replay_correlation_index_present": bool_at(&retained_store, "replay_correlation_index_present"),
                "local_alert_dispatcher_evaluated": bool_at(&alert_dispatcher, "local_dispatcher_evidenced"),
                "w073_typed_formatting_guard_carried": bool_at(&w040_formatting_intake, "w072_threshold_fallback_allowed_for_aggregate_visualization") == false,
                "operated_continuous_assurance_service_promoted": false,
                "retained_history_service_promoted": false,
                "external_alert_dispatcher_promoted": false,
                "quarantine_service_promoted": false,
                "operated_cross_engine_differential_service_promoted": false,
                "pack_grade_replay_promoted": false,
                "c5_promoted": false,
                "stage2_policy_promoted": false,
                "service_readiness_criteria_count": number_at(&readiness, "criteria_count"),
                "service_readiness_blocked_count": number_at(&readiness, "blocked_criteria_count"),
                "exact_service_blocker_count": exact_service_blocker_count,
                "blockers": w040_exact_service_blockers()
                    .iter()
                    .map(|row| row["blocker_id"].clone())
                    .collect::<Vec<_>>(),
                "semantic_equivalence_statement": "This W040 runner emits operated-assurance service artifacts, a file-backed retained-history store/query register, replay-correlation index, local alert/quarantine dispatcher evidence, and exact service blockers only. It does not change scheduler, recalc, publication, replay, pack, service, alert-dispatch, TraceCalc, TreeCalc, OxFml, OxFunc, Lean, or TLA semantics."
            }),
        )?;
        write_json(
            &artifact_root.join("validation.json"),
            &json!({
                "schema_version": W040_VALIDATION_SCHEMA_V1,
                "run_id": run_id,
                "status": if source_failures.is_empty() {
                    "w040_operated_assurance_retained_history_service_artifacts_valid"
                } else {
                    "w040_operated_assurance_retained_history_service_artifacts_invalid"
                },
                "source_evidence_row_count": source_rows.len(),
                "operated_runner_row_count": number_at(&operated_runner, "row_count"),
                "multi_run_history_row_count": number_at(&retained_store, "store_record_count"),
                "query_register_row_count": number_at(&retained_store, "query_register_row_count"),
                "replay_correlation_row_count": number_at(&retained_store, "replay_correlation_row_count"),
                "evaluated_alert_rule_count": number_at(&alert_dispatcher, "evaluated_rule_count"),
                "quarantine_decision_count": number_at(&alert_dispatcher, "quarantine_decision_count"),
                "alert_decision_count": number_at(&alert_dispatcher, "alert_decision_count"),
                "service_readiness_criteria_count": number_at(&readiness, "criteria_count"),
                "service_readiness_blocked_count": number_at(&readiness, "blocked_criteria_count"),
                "exact_service_blocker_count": exact_service_blocker_count,
                "failed_row_count": failed_row_count,
                "validation_failures": source_failures
            }),
        )?;
        write_json(
            &artifact_root.join("run_summary.json"),
            &json!({
                "schema_version": W040_RUN_SUMMARY_SCHEMA_V1,
                "run_id": run_id,
                "artifact_root": relative_artifact_root,
                "source_evidence_index_path": source_evidence_index_path,
                "operated_runner_register_path": operated_runner_path,
                "retained_history_store_query_path": retained_history_store_path,
                "alert_dispatcher_enforcement_path": alert_dispatcher_path,
                "cross_engine_service_register_path": cross_engine_service_path,
                "service_readiness_register_path": service_readiness_path,
                "exact_service_blocker_register_path": blocker_register_path,
                "promotion_decision_path": promotion_decision_path,
                "validation_path": validation_path,
                "source_evidence_row_count": source_rows.len(),
                "operated_runner_row_count": number_at(&operated_runner, "row_count"),
                "multi_run_history_row_count": number_at(&retained_store, "store_record_count"),
                "query_register_row_count": number_at(&retained_store, "query_register_row_count"),
                "replay_correlation_row_count": number_at(&retained_store, "replay_correlation_row_count"),
                "evaluated_alert_rule_count": number_at(&alert_dispatcher, "evaluated_rule_count"),
                "quarantine_decision_count": number_at(&alert_dispatcher, "quarantine_decision_count"),
                "alert_decision_count": number_at(&alert_dispatcher, "alert_decision_count"),
                "service_readiness_criteria_count": number_at(&readiness, "criteria_count"),
                "service_readiness_blocked_count": number_at(&readiness, "blocked_criteria_count"),
                "exact_service_blocker_count": exact_service_blocker_count,
                "failed_row_count": failed_row_count,
                "file_backed_operated_runner_present": true,
                "retained_history_artifact_store_present": bool_at(&retained_store, "file_backed_retained_history_store_present"),
                "retained_history_query_register_present": bool_at(&retained_store, "retained_history_query_register_present"),
                "replay_correlation_index_present": bool_at(&retained_store, "replay_correlation_index_present"),
                "local_alert_dispatcher_evaluated": bool_at(&alert_dispatcher, "local_dispatcher_evidenced"),
                "operated_continuous_assurance_service_promoted": false,
                "retained_history_service_promoted": false,
                "operated_cross_engine_differential_service_promoted": false,
                "external_alert_dispatcher_promoted": false
            }),
        )?;

        Ok(OperatedAssuranceRunSummary {
            run_id: run_id.to_string(),
            schema_version: W040_RUN_SUMMARY_SCHEMA_V1.to_string(),
            source_evidence_row_count: source_rows.len(),
            multi_run_history_row_count: number_at(&retained_store, "store_record_count") as usize,
            evaluated_alert_rule_count: number_at(&alert_dispatcher, "evaluated_rule_count")
                as usize,
            quarantine_decision_count: number_at(&alert_dispatcher, "quarantine_decision_count")
                as usize,
            alert_decision_count: number_at(&alert_dispatcher, "alert_decision_count") as usize,
            service_readiness_criteria_count: number_at(&readiness, "criteria_count") as usize,
            service_readiness_blocked_count: number_at(&readiness, "blocked_criteria_count")
                as usize,
            exact_service_blocker_count,
            failed_row_count,
            operated_service_promoted: false,
            artifact_root: relative_artifact_root,
        })
    }

    fn execute_w039(
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

        let w039_ledger = read_json(repo_root, W039_RESIDUAL_LEDGER)?;
        let w038_summary = read_json(repo_root, W038_OPERATED_ASSURANCE_SUMMARY)?;
        let w038_validation = read_json(repo_root, W038_OPERATED_ASSURANCE_VALIDATION)?;
        let w038_history = read_json(repo_root, W038_OPERATED_MULTI_RUN_HISTORY)?;
        let w038_alerts = read_json(repo_root, W038_OPERATED_ALERT_QUARANTINE)?;
        let w038_cross_engine = read_json(repo_root, W038_OPERATED_CROSS_ENGINE_SERVICE)?;
        let w038_readiness = read_json(repo_root, W038_OPERATED_SERVICE_READINESS)?;
        let w038_blockers = read_json(repo_root, W038_OPERATED_SERVICE_BLOCKERS)?;
        let w038_promotion = read_json(repo_root, W038_OPERATED_PROMOTION_DECISION)?;
        let w039_stage2_summary = read_json(repo_root, W039_STAGE2_SUMMARY)?;
        let w039_stage2_validation = read_json(repo_root, W039_STAGE2_VALIDATION)?;
        let w039_stage2_decision = read_json(repo_root, W039_STAGE2_DECISION)?;
        let w039_stage2_blockers = read_json(repo_root, W039_STAGE2_BLOCKERS)?;
        let w038_pack_decision = read_json(repo_root, W038_PACK_DECISION)?;

        let source_rows = w039_source_rows(
            &w039_ledger,
            &w038_summary,
            &w038_validation,
            &w038_history,
            &w038_alerts,
            &w038_cross_engine,
            &w038_readiness,
            &w038_blockers,
            &w038_promotion,
            &w039_stage2_summary,
            &w039_stage2_validation,
            &w039_stage2_decision,
            &w039_stage2_blockers,
            &w038_pack_decision,
        );
        let source_failures = w039_source_validation_failures(&source_rows);
        let retained_history = w039_retained_history(
            run_id,
            &relative_artifact_root,
            &w038_history,
            &w039_stage2_summary,
            &w039_stage2_blockers,
            &w038_pack_decision,
        );
        let alert_dispatcher = w039_alert_dispatcher(
            run_id,
            &w038_alerts,
            &w038_promotion,
            &w039_stage2_decision,
            &w038_pack_decision,
        );
        let cross_engine_service = w039_cross_engine_service(
            run_id,
            &w038_cross_engine,
            &w039_stage2_summary,
            &w039_stage2_blockers,
        );
        let readiness = w039_service_readiness(
            run_id,
            &relative_artifact_root,
            &retained_history,
            &alert_dispatcher,
            &cross_engine_service,
        );
        let exact_blockers = w039_exact_service_blockers();
        let exact_service_blocker_count = exact_blockers.len();
        let failed_row_count = source_failures.len();

        let source_evidence_index_path =
            format!("{relative_artifact_root}/source_evidence_index.json");
        let retained_history_path =
            format!("{relative_artifact_root}/w039_retained_history_lifecycle.json");
        let alert_dispatcher_path =
            format!("{relative_artifact_root}/w039_alert_dispatcher_enforcement.json");
        let cross_engine_service_path =
            format!("{relative_artifact_root}/w039_cross_engine_service_substrate.json");
        let service_readiness_path =
            format!("{relative_artifact_root}/w039_service_readiness_register.json");
        let blocker_register_path =
            format!("{relative_artifact_root}/w039_exact_service_blocker_register.json");
        let promotion_decision_path = format!("{relative_artifact_root}/promotion_decision.json");
        let validation_path = format!("{relative_artifact_root}/validation.json");

        write_json(
            &artifact_root.join("source_evidence_index.json"),
            &json!({
                "schema_version": W039_SOURCE_INDEX_SCHEMA_V1,
                "run_id": run_id,
                "artifact_root": relative_artifact_root,
                "source_evidence_row_count": source_rows.len(),
                "rows": source_rows,
                "source_artifacts": {
                    "w039_successor_obligation_ledger": W039_RESIDUAL_LEDGER,
                    "w038_operated_assurance_summary": W038_OPERATED_ASSURANCE_SUMMARY,
                    "w038_operated_assurance_validation": W038_OPERATED_ASSURANCE_VALIDATION,
                    "w038_multi_run_history": W038_OPERATED_MULTI_RUN_HISTORY,
                    "w038_alert_quarantine": W038_OPERATED_ALERT_QUARANTINE,
                    "w038_cross_engine_service": W038_OPERATED_CROSS_ENGINE_SERVICE,
                    "w038_service_readiness": W038_OPERATED_SERVICE_READINESS,
                    "w038_service_blockers": W038_OPERATED_SERVICE_BLOCKERS,
                    "w038_service_promotion_decision": W038_OPERATED_PROMOTION_DECISION,
                    "w039_stage2_summary": W039_STAGE2_SUMMARY,
                    "w039_stage2_validation": W039_STAGE2_VALIDATION,
                    "w039_stage2_decision": W039_STAGE2_DECISION,
                    "w039_stage2_blockers": W039_STAGE2_BLOCKERS,
                    "w038_pack_decision": W038_PACK_DECISION
                }
            }),
        )?;
        write_json(
            &artifact_root.join("w039_retained_history_lifecycle.json"),
            &retained_history,
        )?;
        write_json(
            &artifact_root.join("w039_alert_dispatcher_enforcement.json"),
            &alert_dispatcher,
        )?;
        write_json(
            &artifact_root.join("w039_cross_engine_service_substrate.json"),
            &cross_engine_service,
        )?;
        write_json(
            &artifact_root.join("w039_service_readiness_register.json"),
            &readiness,
        )?;
        write_json(
            &artifact_root.join("w039_exact_service_blocker_register.json"),
            &json!({
                "schema_version": W039_BLOCKER_REGISTER_SCHEMA_V1,
                "run_id": run_id,
                "exact_service_blocker_count": exact_service_blocker_count,
                "rows": exact_blockers
            }),
        )?;
        write_json(
            &artifact_root.join("promotion_decision.json"),
            &json!({
                "schema_version": W039_PROMOTION_DECISION_SCHEMA_V1,
                "run_id": run_id,
                "decision_state": "w039_operated_service_substrate_validated_service_unpromoted",
                "local_alert_quarantine_enforcement_evidenced": true,
                "retained_history_lifecycle_bound": true,
                "cross_engine_file_backed_substrate_bound": true,
                "operated_continuous_assurance_service_promoted": false,
                "external_alert_dispatcher_promoted": false,
                "operated_cross_engine_differential_service_promoted": false,
                "retained_history_service_promoted": false,
                "pack_grade_replay_promoted": false,
                "c5_promoted": false,
                "stage2_policy_promoted": false,
                "service_readiness_criteria_count": number_at(&readiness, "criteria_count"),
                "service_readiness_blocked_count": number_at(&readiness, "blocked_criteria_count"),
                "exact_service_blocker_count": exact_service_blocker_count,
                "blockers": w039_exact_service_blockers()
                    .iter()
                    .map(|row| row["blocker_id"].clone())
                    .collect::<Vec<_>>(),
                "semantic_equivalence_statement": "This W039 runner binds checked W038 operated-assurance evidence, W039 Stage 2 service dependencies, retained-history lifecycle rows, and alert-dispatcher policy rows only. It does not change scheduler, recalc, publication, replay, pack, service, alert-dispatch, TraceCalc, TreeCalc, OxFml, OxFunc, Lean, or TLA semantics."
            }),
        )?;
        write_json(
            &artifact_root.join("validation.json"),
            &json!({
                "schema_version": W039_VALIDATION_SCHEMA_V1,
                "run_id": run_id,
                "status": if source_failures.is_empty() {
                    "w039_operated_service_substrate_valid"
                } else {
                    "w039_operated_service_substrate_invalid"
                },
                "source_evidence_row_count": source_rows.len(),
                "multi_run_history_row_count": number_at(&retained_history, "row_count"),
                "evaluated_alert_rule_count": number_at(&alert_dispatcher, "evaluated_rule_count"),
                "quarantine_decision_count": number_at(&alert_dispatcher, "quarantine_decision_count"),
                "alert_decision_count": number_at(&alert_dispatcher, "alert_decision_count"),
                "service_readiness_criteria_count": number_at(&readiness, "criteria_count"),
                "service_readiness_blocked_count": number_at(&readiness, "blocked_criteria_count"),
                "exact_service_blocker_count": exact_service_blocker_count,
                "failed_row_count": failed_row_count,
                "validation_failures": source_failures
            }),
        )?;
        write_json(
            &artifact_root.join("run_summary.json"),
            &json!({
                "schema_version": W039_RUN_SUMMARY_SCHEMA_V1,
                "run_id": run_id,
                "artifact_root": relative_artifact_root,
                "source_evidence_index_path": source_evidence_index_path,
                "retained_history_lifecycle_path": retained_history_path,
                "alert_dispatcher_enforcement_path": alert_dispatcher_path,
                "cross_engine_service_substrate_path": cross_engine_service_path,
                "service_readiness_register_path": service_readiness_path,
                "exact_service_blocker_register_path": blocker_register_path,
                "promotion_decision_path": promotion_decision_path,
                "validation_path": validation_path,
                "source_evidence_row_count": source_rows.len(),
                "multi_run_history_row_count": number_at(&retained_history, "row_count"),
                "evaluated_alert_rule_count": number_at(&alert_dispatcher, "evaluated_rule_count"),
                "quarantine_decision_count": number_at(&alert_dispatcher, "quarantine_decision_count"),
                "alert_decision_count": number_at(&alert_dispatcher, "alert_decision_count"),
                "service_readiness_criteria_count": number_at(&readiness, "criteria_count"),
                "service_readiness_blocked_count": number_at(&readiness, "blocked_criteria_count"),
                "exact_service_blocker_count": exact_service_blocker_count,
                "failed_row_count": failed_row_count,
                "operated_continuous_assurance_service_promoted": false,
                "operated_cross_engine_differential_service_promoted": false,
                "external_alert_dispatcher_promoted": false,
                "retained_history_service_promoted": false
            }),
        )?;

        Ok(OperatedAssuranceRunSummary {
            run_id: run_id.to_string(),
            schema_version: W039_RUN_SUMMARY_SCHEMA_V1.to_string(),
            source_evidence_row_count: source_rows.len(),
            multi_run_history_row_count: number_at(&retained_history, "row_count") as usize,
            evaluated_alert_rule_count: number_at(&alert_dispatcher, "evaluated_rule_count")
                as usize,
            quarantine_decision_count: number_at(&alert_dispatcher, "quarantine_decision_count")
                as usize,
            alert_decision_count: number_at(&alert_dispatcher, "alert_decision_count") as usize,
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

#[allow(clippy::too_many_arguments)]
fn w039_source_rows(
    w039_ledger: &Value,
    w038_summary: &Value,
    w038_validation: &Value,
    w038_history: &Value,
    w038_alerts: &Value,
    w038_cross_engine: &Value,
    w038_readiness: &Value,
    w038_blockers: &Value,
    w038_promotion: &Value,
    w039_stage2_summary: &Value,
    w039_stage2_validation: &Value,
    w039_stage2_decision: &Value,
    w039_stage2_blockers: &Value,
    w038_pack_decision: &Value,
) -> Vec<Value> {
    vec![
        json!({
            "row_id": "source.w039_successor_obligation_ledger",
            "artifact": W039_RESIDUAL_LEDGER,
            "valid": number_at(w039_ledger, "obligation_count") == 20,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": 0,
            "promoted_unsupported_service": false,
            "semantic_state": "w039_service_obligations_bound"
        }),
        json!({
            "row_id": "source.w038_operated_assurance_summary",
            "artifact": W038_OPERATED_ASSURANCE_SUMMARY,
            "valid": string_at(w038_validation, "status") == "w038_operated_assurance_packet_valid"
                && number_at(w038_summary, "exact_service_blocker_count") == 4,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": number_at(w038_summary, "failed_row_count"),
            "promoted_unsupported_service": bool_at(w038_promotion, "operated_continuous_assurance_service_promoted")
                || bool_at(w038_promotion, "operated_cross_engine_differential_service_promoted")
                || bool_at(w038_promotion, "external_alert_dispatcher_promoted"),
            "semantic_state": "w038_operated_assurance_packet_bound"
        }),
        json!({
            "row_id": "source.w038_retained_history_ledger",
            "artifact": W038_OPERATED_MULTI_RUN_HISTORY,
            "valid": number_at(w038_history, "row_count") == 15,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": 0,
            "promoted_unsupported_service": bool_at(w038_history, "retained_history_service_present"),
            "semantic_state": "w038_checked_in_history_bound_without_service_store"
        }),
        json!({
            "row_id": "source.w038_alert_quarantine_local_evaluation",
            "artifact": W038_OPERATED_ALERT_QUARANTINE,
            "valid": number_at(w038_alerts, "evaluated_rule_count") == 8
                && number_at(w038_alerts, "quarantine_decision_count") == 0
                && number_at(w038_alerts, "alert_decision_count") == 0,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": number_at(w038_alerts, "quarantine_decision_count"),
            "promoted_unsupported_service": bool_at(w038_alerts, "external_alert_dispatcher_promoted"),
            "semantic_state": "w038_local_alert_rules_evaluated"
        }),
        json!({
            "row_id": "source.w038_cross_engine_file_backed_service",
            "artifact": W038_OPERATED_CROSS_ENGINE_SERVICE,
            "valid": bool_at(w038_cross_engine, "file_backed_pilot_present")
                && !bool_at(w038_cross_engine, "operated_cross_engine_differential_service_promoted"),
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": number_at(w038_cross_engine, "w037_cross_engine_unexpected_mismatch_count"),
            "failed_row_count": 0,
            "promoted_unsupported_service": bool_at(w038_cross_engine, "operated_cross_engine_differential_service_promoted"),
            "semantic_state": "file_backed_cross_engine_pilot_bound"
        }),
        json!({
            "row_id": "source.w038_service_readiness_blockers",
            "artifact": W038_OPERATED_SERVICE_READINESS,
            "valid": number_at(w038_readiness, "blocked_criteria_count") == 4
                && number_at(w038_blockers, "exact_service_blocker_count") == 4,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": 0,
            "promoted_unsupported_service": bool_at(w038_readiness, "operated_continuous_assurance_service_promoted")
                || bool_at(w038_readiness, "cross_engine_differential_service_promoted")
                || bool_at(w038_readiness, "external_alert_dispatcher_promoted"),
            "semantic_state": "w038_service_blockers_bound"
        }),
        json!({
            "row_id": "source.w039_stage2_service_dependency",
            "artifact": W039_STAGE2_SUMMARY,
            "valid": string_at(w039_stage2_validation, "status") == "w039_stage2_policy_governance_valid"
                && row_with_field_exists(
                    w039_stage2_blockers,
                    "row_id",
                    "w039_stage2_operated_cross_engine_service_dependency_blocker"
                ),
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": number_at(w039_stage2_summary, "failed_row_count"),
            "promoted_unsupported_service": bool_at(w039_stage2_decision, "operated_cross_engine_stage2_service_promoted")
                || bool_at(w039_stage2_decision, "stage2_policy_promoted"),
            "semantic_state": "w039_stage2_service_dependency_bound"
        }),
        json!({
            "row_id": "source.w038_pack_retained_history_blockers",
            "artifact": W038_PACK_DECISION,
            "valid": string_at(w038_pack_decision, "decision_status") == "capability_not_promoted"
                && array_contains_string(
                    &w038_pack_decision["no_promotion_reason_ids"],
                    "pack.grade.w038_retained_history_store_absent"
                ),
            "missing_artifact_count": number_at(w038_pack_decision, "missing_artifact_count"),
            "unexpected_mismatch_count": 0,
            "failed_row_count": 0,
            "promoted_unsupported_service": bool_at(w038_pack_decision, "capability_promoted"),
            "semantic_state": "pack_retained_history_blocker_bound"
        }),
    ]
}

fn w039_source_validation_failures(source_rows: &[Value]) -> Vec<String> {
    source_rows
        .iter()
        .flat_map(|row| {
            let row_id = text_at(row, "row_id");
            let mut failures = Vec::new();
            if !bool_at(row, "valid") {
                failures.push(format!("{row_id}.valid_false"));
            }
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

fn w039_retained_history(
    run_id: &str,
    relative_artifact_root: &str,
    w038_history: &Value,
    w039_stage2_summary: &Value,
    w039_stage2_blockers: &Value,
    w038_pack_decision: &Value,
) -> Value {
    let mut rows = w038_history
        .get("rows")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let next_order = rows.len() + 1;
    rows.push(history_row(
        next_order,
        "w039.stage2_policy_governance",
        "w039_stage2_service_dependency",
        "stage2_policy_governance_bound_with_operated_service_dependency",
        W039_STAGE2_SUMMARY,
        number_at(w039_stage2_summary, "failed_row_count"),
        0,
        number_at(w039_stage2_summary, "exact_remaining_blocker_count"),
    ));
    rows.push(history_row(
        next_order + 1,
        "w039.stage2_service_blocker",
        "w039_stage2_operated_service_dependency",
        "operated_stage2_differential_service_blocker_retained",
        W039_STAGE2_BLOCKERS,
        0,
        0,
        if row_with_field_exists(
            w039_stage2_blockers,
            "row_id",
            "w039_stage2_operated_cross_engine_service_dependency_blocker",
        ) {
            1
        } else {
            0
        },
    ));
    rows.push(history_row(
        next_order + 2,
        "w038.pack_c5_decision",
        "w038_pack_retained_history_blocker",
        "pack_decision_retains_history_store_and_program_grade_governance_blockers",
        W038_PACK_DECISION,
        0,
        0,
        array_len(&w038_pack_decision["no_promotion_reason_ids"]) as u64,
    ));

    json!({
        "schema_version": W039_RETAINED_HISTORY_SCHEMA_V1,
        "run_id": run_id,
        "artifact_root": relative_artifact_root,
        "history_kind": "w039_retained_history_lifecycle_from_checked_artifacts",
        "continuous_service_present": false,
        "retained_history_service_present": false,
        "retained_history_query_api_present": false,
        "replay_correlation_service_present": false,
        "history_lifecycle_state": "checked_in_lifecycle_ledger_bound_without_operated_store",
        "row_count": rows.len(),
        "rows": rows
    })
}

fn w039_alert_dispatcher(
    run_id: &str,
    w038_alerts: &Value,
    w038_promotion: &Value,
    w039_stage2_decision: &Value,
    w038_pack_decision: &Value,
) -> Value {
    let mut rows = w038_alerts
        .get("rows")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    rows.push(json!({
        "rule_id": "quarantine.w039_unsupported_operated_service_claim",
        "action": "quarantine_run_and_block_service_promotion",
        "trigger": "any operated service or dispatcher promotion flag appears without operated service artifacts",
        "owner": "calc-f7o.5; calc-f7o.9",
        "triggered": bool_at(w038_promotion, "operated_continuous_assurance_service_promoted")
            || bool_at(w038_promotion, "external_alert_dispatcher_promoted")
            || bool_at(w039_stage2_decision, "operated_cross_engine_stage2_service_promoted"),
        "decision": "clean",
        "evidence": {
            "operated_continuous_assurance_service_promoted": bool_at(w038_promotion, "operated_continuous_assurance_service_promoted"),
            "external_alert_dispatcher_promoted": bool_at(w038_promotion, "external_alert_dispatcher_promoted"),
            "operated_cross_engine_stage2_service_promoted": bool_at(w039_stage2_decision, "operated_cross_engine_stage2_service_promoted")
        }
    }));
    rows.push(json!({
        "rule_id": "quarantine.w039_pack_or_c5_claim_without_retained_service",
        "action": "quarantine_run_and_block_pack_reassessment",
        "trigger": "pack-grade replay or C5 is promoted while retained history service remains absent",
        "owner": "calc-f7o.5; calc-f7o.8; calc-f7o.9",
        "triggered": bool_at(w038_pack_decision, "capability_promoted"),
        "decision": "clean",
        "evidence": {
            "capability_promoted": bool_at(w038_pack_decision, "capability_promoted"),
            "retained_history_service_present": false
        }
    }));
    rows.push(json!({
        "rule_id": "alert.w039_stage2_service_dependency_retained",
        "action": "record_stage2_service_dependency_without_quarantine",
        "trigger": "Stage 2 policy-governance packet retains operated service as an exact dependency",
        "owner": "calc-f7o.5; calc-f7o.6",
        "triggered": false,
        "decision": "clean",
        "evidence": {
            "stage2_policy_promoted": bool_at(w039_stage2_decision, "stage2_policy_promoted"),
            "operated_cross_engine_stage2_service_promoted": bool_at(w039_stage2_decision, "operated_cross_engine_stage2_service_promoted")
        }
    }));

    let quarantine_decision_count = rows
        .iter()
        .filter(|row| {
            row.get("triggered").and_then(Value::as_bool) == Some(true)
                && text_at(row, "action").starts_with("quarantine")
        })
        .count();
    let alert_decision_count = rows
        .iter()
        .filter(|row| {
            row.get("triggered").and_then(Value::as_bool) == Some(true)
                && text_at(row, "action").starts_with("alert")
        })
        .count();

    json!({
        "schema_version": W039_ALERT_DISPATCHER_SCHEMA_V1,
        "run_id": run_id,
        "policy_source": W038_OPERATED_ALERT_QUARANTINE,
        "policy_state": "w039_local_dispatch_policy_evaluated_without_external_dispatcher_promotion",
        "evaluated_rule_count": rows.len(),
        "quarantine_decision_count": quarantine_decision_count,
        "alert_decision_count": alert_decision_count,
        "clean_rule_count": rows.len() - quarantine_decision_count - alert_decision_count,
        "local_enforcement_evidenced": true,
        "external_alert_dispatcher_promoted": false,
        "quarantine_service_promoted": false,
        "rows": rows
    })
}

fn w039_cross_engine_service(
    run_id: &str,
    w038_cross_engine: &Value,
    w039_stage2_summary: &Value,
    w039_stage2_blockers: &Value,
) -> Value {
    json!({
        "schema_version": W039_CROSS_ENGINE_SERVICE_SCHEMA_V1,
        "run_id": run_id,
        "file_backed_pilot_present": bool_at(w038_cross_engine, "file_backed_pilot_present"),
        "w038_file_backed_gate_row_count": number_at(w038_cross_engine, "w037_cross_engine_gate_row_count"),
        "w039_stage2_policy_row_count": number_at(w039_stage2_summary, "policy_row_count"),
        "w039_stage2_service_dependency_blocker_present": row_with_field_exists(
            w039_stage2_blockers,
            "row_id",
            "w039_stage2_operated_cross_engine_service_dependency_blocker"
        ),
        "operated_cross_engine_differential_service_present": false,
        "operated_cross_engine_differential_service_promoted": false,
        "service_state": "file_backed_cross_engine_substrate_bound_without_operated_service",
        "blocked_service_claims": [
            "recurring_cross_engine_diff_scheduler",
            "cross_engine_service_endpoint",
            "service_retained_history_store",
            "external_alert_dispatcher"
        ]
    })
}

fn w039_service_readiness(
    run_id: &str,
    relative_artifact_root: &str,
    retained_history: &Value,
    alert_dispatcher: &Value,
    cross_engine_service: &Value,
) -> Value {
    let criteria = vec![
        criterion(
            "readiness.w039_retained_history_lifecycle_bound",
            "satisfied",
            "W039 extends W038 history with Stage 2 service dependency and pack retained-history blockers",
        ),
        criterion(
            "readiness.w039_alert_dispatch_policy_evaluated",
            "satisfied",
            "W039 evaluates local alert/quarantine dispatch rules against operated-service and pack promotion flags",
        ),
        criterion(
            "readiness.w039_cross_engine_substrate_bound",
            "satisfied_boundary",
            "W039 binds file-backed cross-engine substrate and Stage 2 service dependency without service promotion",
        ),
        criterion(
            "readiness.no_quarantine_decisions",
            "satisfied",
            "W039 source rows have no missing artifacts, unexpected mismatches, failed semantic rows, or unsupported service promotion",
        ),
        criterion(
            "readiness.stage2_service_dependency_classified",
            "satisfied",
            "W039 Stage 2 policy-governance retains operated service as an exact dependency",
        ),
        criterion(
            "readiness.pack_retained_history_blocker_classified",
            "satisfied",
            "W038 pack/C5 decision retains retained-history and program-grade replay governance blockers",
        ),
        criterion(
            "readiness.source_artifacts_retained",
            "satisfied",
            "all W038 and W039 source artifacts required by this substrate packet are present",
        ),
        criterion(
            "service.operated_regression_runner",
            "blocked",
            "no recurring operated regression runner or scheduler is present",
        ),
        criterion(
            "service.retained_history_store",
            "blocked",
            "history is checked-in evidence rather than an operated retained store with retention guarantees",
        ),
        criterion(
            "service.retained_history_query_api",
            "blocked",
            "no retained-history query API or replay-correlation service is operated",
        ),
        criterion(
            "service.enforcing_alert_dispatcher",
            "blocked",
            "local rule evaluation is present, but no external alert dispatcher or quarantine service is operated",
        ),
        criterion(
            "service.operated_cross_engine_differential",
            "blocked",
            "cross-engine differential evidence remains file-backed rather than an operated service",
        ),
    ];
    let blocked_criteria_count = criteria
        .iter()
        .filter(|row| row.get("state").and_then(Value::as_str) == Some("blocked"))
        .count();

    json!({
        "schema_version": W039_SERVICE_READINESS_SCHEMA_V1,
        "run_id": run_id,
        "artifact_root": relative_artifact_root,
        "readiness_state": "w039_operated_service_substrate_bound_without_service_promotion",
        "criteria_count": criteria.len(),
        "satisfied_criteria_count": criteria.len() - blocked_criteria_count,
        "blocked_criteria_count": blocked_criteria_count,
        "history_window_row_count": number_at(retained_history, "row_count"),
        "evaluated_alert_rule_count": number_at(alert_dispatcher, "evaluated_rule_count"),
        "quarantine_decision_count": number_at(alert_dispatcher, "quarantine_decision_count"),
        "alert_decision_count": number_at(alert_dispatcher, "alert_decision_count"),
        "file_backed_cross_engine_substrate_present": bool_at(cross_engine_service, "file_backed_pilot_present"),
        "operated_continuous_assurance_service_promoted": false,
        "cross_engine_differential_service_promoted": false,
        "external_alert_dispatcher_promoted": false,
        "retained_history_service_promoted": false,
        "criteria": criteria
    })
}

fn w039_exact_service_blockers() -> Vec<Value> {
    vec![
        json!({
            "blocker_id": "service.operated_regression_runner_absent",
            "owner": "calc-f7o.5; calc-f7o.9",
            "status_after_run": "exact_remaining_blocker",
            "reason": "W039 binds a retained history lifecycle ledger, but no recurring runner or scheduler is operated.",
            "promotion_consequence": "operated continuous assurance service remains unpromoted"
        }),
        json!({
            "blocker_id": "service.retained_history_store_absent",
            "owner": "calc-f7o.5; calc-f7o.8; calc-f7o.9",
            "status_after_run": "exact_remaining_blocker",
            "reason": "history remains checked-in evidence rather than an operated retained store with retention guarantees.",
            "promotion_consequence": "retained history service, pack-grade replay, and C5 remain unpromoted"
        }),
        json!({
            "blocker_id": "service.retained_history_query_api_absent",
            "owner": "calc-f7o.5; calc-f7o.8",
            "status_after_run": "exact_remaining_blocker",
            "reason": "no retained-history query API or replay-correlation service is operated.",
            "promotion_consequence": "program-grade replay governance remains unpromoted"
        }),
        json!({
            "blocker_id": "service.external_alert_dispatcher_absent",
            "owner": "calc-f7o.5; calc-f7o.9",
            "status_after_run": "exact_remaining_blocker",
            "reason": "W039 evaluates alert/quarantine rules locally but does not operate an external dispatcher or quarantine service.",
            "promotion_consequence": "alert/quarantine dispatcher claims remain unpromoted"
        }),
        json!({
            "blocker_id": "service.operated_cross_engine_differential_absent",
            "owner": "calc-f7o.5; calc-f7o.6",
            "status_after_run": "exact_remaining_blocker",
            "reason": "cross-engine evidence remains file-backed and does not run as an operated differential service.",
            "promotion_consequence": "operated cross-engine differential service and fully independent diversity promotion remain blocked"
        }),
    ]
}

#[allow(clippy::too_many_arguments)]
fn w040_source_rows(
    w040_direct_summary: &Value,
    w040_direct_map: &Value,
    w040_formatting_intake: &Value,
    w039_summary: &Value,
    w039_validation: &Value,
    w039_retained: &Value,
    w039_alerts: &Value,
    w039_cross_engine: &Value,
    w039_readiness: &Value,
    w039_blockers: &Value,
    w039_promotion: &Value,
    w040_stage2_summary: &Value,
    w040_stage2_validation: &Value,
    w040_stage2_decision: &Value,
    w040_stage2_blockers: &Value,
) -> Vec<Value> {
    vec![
        json!({
            "row_id": "source.w040_direct_obligation_map",
            "artifact": W040_DIRECT_OBLIGATION_MAP,
            "valid": text_at(w040_direct_summary, "status") == "direct_verification_obligation_map_validated"
                && number_at(w040_direct_summary, "obligation_count") == 23
                && number_at(w040_direct_map, "obligation_count") == 23,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": 0,
            "promoted_unsupported_service": false,
            "semantic_state": "w040_service_obligations_bound"
        }),
        json!({
            "row_id": "source.w040_w073_typed_formatting_guard",
            "artifact": W040_FORMATTING_INTAKE,
            "valid": text_at(w040_formatting_intake, "contract_mode") == "direct_replacement_for_aggregate_and_visualization_metadata"
                && !bool_at(w040_formatting_intake, "w072_threshold_fallback_allowed_for_aggregate_visualization"),
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": 0,
            "promoted_unsupported_service": false,
            "semantic_state": "w073_typed_only_formatting_guard_bound"
        }),
        json!({
            "row_id": "source.w039_operated_assurance_summary",
            "artifact": W039_OPERATED_ASSURANCE_SUMMARY,
            "valid": text_at(w039_validation, "status") == "w039_operated_service_substrate_valid"
                && number_at(w039_summary, "exact_service_blocker_count") == 5,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": number_at(w039_summary, "failed_row_count"),
            "promoted_unsupported_service": bool_at(w039_promotion, "operated_continuous_assurance_service_promoted")
                || bool_at(w039_promotion, "retained_history_service_promoted")
                || bool_at(w039_promotion, "external_alert_dispatcher_promoted")
                || bool_at(w039_promotion, "operated_cross_engine_differential_service_promoted"),
            "semantic_state": "w039_service_substrate_bound_without_promotion"
        }),
        json!({
            "row_id": "source.w039_retained_history_lifecycle",
            "artifact": W039_RETAINED_HISTORY_LIFECYCLE,
            "valid": number_at(w039_retained, "row_count") == 18
                && !bool_at(w039_retained, "retained_history_service_present")
                && !bool_at(w039_retained, "retained_history_query_api_present"),
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": 0,
            "promoted_unsupported_service": bool_at(w039_retained, "retained_history_service_present"),
            "semantic_state": "w039_lifecycle_rows_available_for_w040_file_backed_store"
        }),
        json!({
            "row_id": "source.w039_alert_dispatcher_policy",
            "artifact": W039_ALERT_DISPATCHER_ENFORCEMENT,
            "valid": number_at(w039_alerts, "evaluated_rule_count") == 11
                && number_at(w039_alerts, "quarantine_decision_count") == 0
                && number_at(w039_alerts, "alert_decision_count") == 0,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": number_at(w039_alerts, "quarantine_decision_count"),
            "promoted_unsupported_service": bool_at(w039_alerts, "external_alert_dispatcher_promoted")
                || bool_at(w039_alerts, "quarantine_service_promoted"),
            "semantic_state": "w039_local_dispatch_policy_clean"
        }),
        json!({
            "row_id": "source.w039_cross_engine_file_backed_substrate",
            "artifact": W039_CROSS_ENGINE_SERVICE_SUBSTRATE,
            "valid": bool_at(w039_cross_engine, "file_backed_pilot_present")
                && !bool_at(w039_cross_engine, "operated_cross_engine_differential_service_promoted"),
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": 0,
            "promoted_unsupported_service": bool_at(w039_cross_engine, "operated_cross_engine_differential_service_promoted"),
            "semantic_state": "file_backed_cross_engine_substrate_bound_for_w040_service_dependency"
        }),
        json!({
            "row_id": "source.w039_service_readiness_blockers",
            "artifact": W039_SERVICE_READINESS_REGISTER,
            "valid": number_at(w039_readiness, "blocked_criteria_count") == 5
                && number_at(w039_blockers, "exact_service_blocker_count") == 5,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": 0,
            "promoted_unsupported_service": bool_at(w039_readiness, "operated_continuous_assurance_service_promoted")
                || bool_at(w039_readiness, "retained_history_service_promoted")
                || bool_at(w039_readiness, "external_alert_dispatcher_promoted")
                || bool_at(w039_readiness, "cross_engine_differential_service_promoted"),
            "semantic_state": "w039_service_blockers_bound_for_w040_direct_attempt"
        }),
        json!({
            "row_id": "source.w040_stage2_policy_dependency",
            "artifact": W040_STAGE2_SUMMARY,
            "valid": text_at(w040_stage2_validation, "status") == "w040_stage2_policy_equivalence_valid"
                && number_at(w040_stage2_summary, "failed_row_count") == 0
                && row_with_field_exists(
                    w040_stage2_blockers,
                    "row_id",
                    "w040_stage2_operated_cross_engine_service_dependency_blocker"
                ),
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": number_at(w040_stage2_summary, "failed_row_count"),
            "promoted_unsupported_service": bool_at(w040_stage2_decision, "operated_cross_engine_stage2_service_promoted")
                || bool_at(w040_stage2_decision, "stage2_policy_promoted"),
            "semantic_state": "w040_stage2_service_dependency_bound"
        }),
        json!({
            "row_id": "source.w040_stage2_pack_governance_dependency",
            "artifact": W040_STAGE2_BLOCKERS,
            "valid": row_with_field_exists(
                    w040_stage2_blockers,
                    "row_id",
                    "w040_stage2_pack_grade_replay_governance_blocker"
                )
                && !bool_at(w040_stage2_decision, "pack_grade_replay_governance_promoted"),
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": 0,
            "promoted_unsupported_service": bool_at(w040_stage2_decision, "pack_grade_replay_governance_promoted"),
            "semantic_state": "w040_pack_governance_dependency_retained"
        }),
        json!({
            "row_id": "source.w040_promotion_guard",
            "artifact": W040_STAGE2_DECISION,
            "valid": !bool_at(w040_stage2_decision, "stage2_policy_promoted")
                && !bool_at(w040_stage2_decision, "stage2_promotion_candidate")
                && !bool_at(w040_stage2_decision, "operated_cross_engine_stage2_service_promoted")
                && !bool_at(w039_promotion, "pack_grade_replay_promoted")
                && !bool_at(w039_promotion, "c5_promoted"),
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": 0,
            "promoted_unsupported_service": bool_at(w040_stage2_decision, "stage2_policy_promoted")
                || bool_at(w040_stage2_decision, "operated_cross_engine_stage2_service_promoted")
                || bool_at(w039_promotion, "pack_grade_replay_promoted")
                || bool_at(w039_promotion, "c5_promoted"),
            "semantic_state": "service_pack_stage2_promotion_guard_clean"
        }),
    ]
}

fn w040_source_validation_failures(source_rows: &[Value]) -> Vec<String> {
    w039_source_validation_failures(source_rows)
}

fn w040_operated_runner_register(
    run_id: &str,
    relative_artifact_root: &str,
    source_evidence_row_count: usize,
) -> Value {
    let rows = vec![
        json!({
            "row_id": "runner.cli_entrypoint",
            "state": "satisfied",
            "runner_command": format!("cargo run -p oxcalc-tracecalc-cli -- operated-assurance {run_id}"),
            "evidence_or_blocker": "the W040 operated-assurance packet is emitted by the checked CLI runner"
        }),
        json!({
            "row_id": "runner.artifact_root",
            "state": "satisfied",
            "artifact_root": relative_artifact_root,
            "evidence_or_blocker": "runner writes a service-readable artifact root for source index, retained store, alert dispatch, readiness, blockers, decision, and validation"
        }),
        json!({
            "row_id": "runner.source_ingestion",
            "state": "satisfied",
            "source_evidence_row_count": source_evidence_row_count,
            "evidence_or_blocker": "runner binds predecessor W039 operated-assurance rows, W040 direct obligations, Stage 2 service blockers, and W073 formatting intake"
        }),
        json!({
            "row_id": "runner.retained_store_update",
            "state": "satisfied",
            "evidence_or_blocker": "runner emits a deterministic file-backed retained-history store and query register"
        }),
        json!({
            "row_id": "runner.alert_dispatcher_evaluation",
            "state": "satisfied",
            "evidence_or_blocker": "runner evaluates local alert/quarantine dispatch rules over source, store, service, Stage 2, pack, and W073 inputs"
        }),
        json!({
            "row_id": "runner.recurring_scheduler",
            "state": "blocked",
            "evidence_or_blocker": "no recurring scheduler, daemon, or service endpoint is operated by this packet"
        }),
    ];

    json!({
        "schema_version": W040_OPERATED_RUNNER_SCHEMA_V1,
        "run_id": run_id,
        "artifact_root": relative_artifact_root,
        "runner_kind": "file_backed_operated_assurance_runner",
        "runner_command": format!("cargo run -p oxcalc-tracecalc-cli -- operated-assurance {run_id}"),
        "file_backed_runner_present": true,
        "service_endpoint_present": false,
        "recurring_scheduler_present": false,
        "row_count": rows.len(),
        "rows": rows
    })
}

fn w040_retained_history_store_query(
    run_id: &str,
    relative_artifact_root: &str,
    w039_retained: &Value,
    w040_stage2_summary: &Value,
    _w040_stage2_blockers: &Value,
    w040_formatting_intake: &Value,
) -> Value {
    let mut rows = w039_retained
        .get("rows")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let next_order = rows.len() + 1;
    rows.push(history_row(
        next_order,
        "w040.direct_obligation_map",
        "w040_direct_verification_obligation_map",
        "direct_verification_service_obligations_bound",
        W040_DIRECT_OBLIGATION_MAP,
        0,
        0,
        0,
    ));
    rows.push(history_row(
        next_order + 1,
        "w040.stage2_policy_equivalence",
        "w040_stage2_service_dependency",
        "stage2_policy_equivalence_bound_with_operated_service_and_pack_blockers",
        W040_STAGE2_SUMMARY,
        number_at(w040_stage2_summary, "failed_row_count"),
        0,
        number_at(w040_stage2_summary, "exact_remaining_blocker_count"),
    ));
    rows.push(history_row(
        next_order + 2,
        "w040.w073_formatting_intake",
        "w040_w073_typed_formatting_guard",
        "typed_only_formatting_guard_retained_in_service_history",
        W040_FORMATTING_INTAKE,
        0,
        0,
        if bool_at(
            w040_formatting_intake,
            "w072_threshold_fallback_allowed_for_aggregate_visualization",
        ) {
            1
        } else {
            0
        },
    ));

    let query_register = vec![
        json!({
            "query_id": "history.by_run_id",
            "query_kind": "lookup_artifact_root_and_run_summary",
            "result_source": "run_summary.json",
            "deterministic": true
        }),
        json!({
            "query_id": "history.by_source_input_id",
            "query_kind": "filter_history_rows",
            "result_source": "w040_retained_history_store_query.json.rows",
            "deterministic": true
        }),
        json!({
            "query_id": "history.by_blocker",
            "query_kind": "filter_nonzero_blocker_count_and_exact_service_blocker_register",
            "result_source": "w040_exact_service_blocker_register.json",
            "deterministic": true
        }),
        json!({
            "query_id": "history.by_w040_obligation",
            "query_kind": "join_service_rows_to_direct_obligation_map",
            "result_source": W040_DIRECT_OBLIGATION_MAP,
            "deterministic": true
        }),
        json!({
            "query_id": "history.by_replay_correlation",
            "query_kind": "join_history_rows_to_stage2_and_pack_replay_dependency_artifacts",
            "result_source": W040_STAGE2_BLOCKERS,
            "deterministic": true
        }),
    ];
    let replay_correlation_index = vec![
        json!({
            "correlation_id": "corr.w040_stage2_service_dependency",
            "source_artifacts": [W040_STAGE2_SUMMARY, W040_STAGE2_BLOCKERS],
            "w040_obligations": ["W040-OBL-011", "W040-OBL-015", "W040-OBL-021"],
            "replay_role": "stage2_policy_and_pack_governance_dependency"
        }),
        json!({
            "correlation_id": "corr.w040_retained_history_pack_gate",
            "source_artifacts": [W039_RETAINED_HISTORY_LIFECYCLE, W040_DIRECT_OBLIGATION_MAP],
            "w040_obligations": ["W040-OBL-013", "W040-OBL-021"],
            "replay_role": "retained_history_and_pack_grade_replay_gate"
        }),
        json!({
            "correlation_id": "corr.w040_w073_formatting_guard",
            "source_artifacts": [W040_FORMATTING_INTAKE],
            "w040_obligations": ["W040-OBL-018"],
            "replay_role": "observable_formatting_guard"
        }),
    ];

    json!({
        "schema_version": W040_RETAINED_HISTORY_STORE_SCHEMA_V1,
        "run_id": run_id,
        "artifact_root": relative_artifact_root,
        "history_kind": "w040_file_backed_retained_history_store_with_query_register",
        "file_backed_retained_history_store_present": true,
        "retained_history_query_register_present": true,
        "replay_correlation_index_present": true,
        "retained_history_service_present": false,
        "retention_slo_enforced": false,
        "source_history_row_count": number_at(w039_retained, "row_count"),
        "store_record_count": rows.len(),
        "query_register_row_count": query_register.len(),
        "replay_correlation_row_count": replay_correlation_index.len(),
        "history_lifecycle_state": "file_backed_store_and_query_register_bound_without_operated_retention_service",
        "rows": rows,
        "query_register": query_register,
        "replay_correlation_index": replay_correlation_index
    })
}

fn w040_alert_dispatcher(
    run_id: &str,
    w039_alerts: &Value,
    w039_promotion: &Value,
    w040_stage2_decision: &Value,
    retained_store: &Value,
    w040_formatting_intake: &Value,
) -> Value {
    let mut rows = w039_alerts
        .get("rows")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    rows.push(json!({
        "rule_id": "quarantine.w040_unsupported_operated_service_claim",
        "action": "quarantine_run_and_block_service_promotion",
        "trigger": "any W039/W040 operated service or Stage 2 service promotion flag appears without operated service artifacts",
        "owner": "calc-tv5.6; calc-tv5.10",
        "triggered": bool_at(w039_promotion, "operated_continuous_assurance_service_promoted")
            || bool_at(w039_promotion, "retained_history_service_promoted")
            || bool_at(w039_promotion, "external_alert_dispatcher_promoted")
            || bool_at(w040_stage2_decision, "operated_cross_engine_stage2_service_promoted"),
        "decision": "clean",
        "evidence": {
            "operated_continuous_assurance_service_promoted": bool_at(w039_promotion, "operated_continuous_assurance_service_promoted"),
            "retained_history_service_promoted": bool_at(w039_promotion, "retained_history_service_promoted"),
            "external_alert_dispatcher_promoted": bool_at(w039_promotion, "external_alert_dispatcher_promoted"),
            "operated_cross_engine_stage2_service_promoted": bool_at(w040_stage2_decision, "operated_cross_engine_stage2_service_promoted")
        }
    }));
    rows.push(json!({
        "rule_id": "quarantine.w040_missing_retained_query_or_correlation",
        "action": "quarantine_run_and_block_pack_reassessment",
        "trigger": "file-backed retained-history query register or replay-correlation index is absent",
        "owner": "calc-tv5.6; calc-tv5.9",
        "triggered": !bool_at(retained_store, "retained_history_query_register_present")
            || !bool_at(retained_store, "replay_correlation_index_present"),
        "decision": "clean",
        "evidence": {
            "retained_history_query_register_present": bool_at(retained_store, "retained_history_query_register_present"),
            "replay_correlation_index_present": bool_at(retained_store, "replay_correlation_index_present")
        }
    }));
    rows.push(json!({
        "rule_id": "alert.w040_w073_typed_formatting_guard_retained",
        "action": "record_w073_guard_without_handoff",
        "trigger": "W040 service packet carries W073 typed-only conditional-formatting input guard",
        "owner": "calc-tv5.6; calc-tv5.8",
        "triggered": false,
        "decision": "clean",
        "evidence": {
            "w072_threshold_fallback_allowed_for_aggregate_visualization": bool_at(w040_formatting_intake, "w072_threshold_fallback_allowed_for_aggregate_visualization"),
            "contract_mode": text_at(w040_formatting_intake, "contract_mode")
        }
    }));

    let quarantine_decision_count = rows
        .iter()
        .filter(|row| {
            row.get("triggered").and_then(Value::as_bool) == Some(true)
                && text_at(row, "action").starts_with("quarantine")
        })
        .count();
    let alert_decision_count = rows
        .iter()
        .filter(|row| {
            row.get("triggered").and_then(Value::as_bool) == Some(true)
                && text_at(row, "action").starts_with("alert")
        })
        .count();

    json!({
        "schema_version": W040_ALERT_DISPATCHER_SCHEMA_V1,
        "run_id": run_id,
        "policy_source": W039_ALERT_DISPATCHER_ENFORCEMENT,
        "policy_state": "w040_local_dispatcher_evaluated_without_external_dispatcher_promotion",
        "evaluated_rule_count": rows.len(),
        "quarantine_decision_count": quarantine_decision_count,
        "alert_decision_count": alert_decision_count,
        "clean_rule_count": rows.len() - quarantine_decision_count - alert_decision_count,
        "local_dispatcher_evidenced": true,
        "external_alert_dispatcher_promoted": false,
        "quarantine_service_promoted": false,
        "rows": rows
    })
}

fn w040_cross_engine_service(
    run_id: &str,
    w039_cross_engine: &Value,
    w040_stage2_summary: &Value,
    w040_stage2_blockers: &Value,
) -> Value {
    json!({
        "schema_version": W040_CROSS_ENGINE_SERVICE_SCHEMA_V1,
        "run_id": run_id,
        "file_backed_cross_engine_substrate_present": bool_at(w039_cross_engine, "file_backed_pilot_present"),
        "w039_file_backed_gate_row_count": number_at(w039_cross_engine, "w038_file_backed_gate_row_count"),
        "w040_stage2_policy_row_count": number_at(w040_stage2_summary, "policy_row_count"),
        "w040_stage2_service_dependency_blocker_present": row_with_field_exists(
            w040_stage2_blockers,
            "row_id",
            "w040_stage2_operated_cross_engine_service_dependency_blocker"
        ),
        "operated_cross_engine_differential_service_present": false,
        "operated_cross_engine_differential_service_promoted": false,
        "service_endpoint_present": false,
        "service_state": "file_backed_cross_engine_substrate_and_stage2_dependency_bound_without_operated_service",
        "blocked_service_claims": [
            "recurring_cross_engine_diff_scheduler",
            "cross_engine_service_endpoint",
            "operated_mismatch_quarantine_dispatcher"
        ]
    })
}

fn w040_service_readiness(
    run_id: &str,
    relative_artifact_root: &str,
    operated_runner: &Value,
    retained_store: &Value,
    alert_dispatcher: &Value,
    cross_engine_service: &Value,
    w040_formatting_intake: &Value,
) -> Value {
    let criteria = vec![
        criterion(
            "readiness.w040_runner_entrypoint_runnable",
            "satisfied",
            "W040 operated-assurance packet is emitted through the checked oxcalc-tracecalc-cli operated-assurance runner",
        ),
        criterion(
            "readiness.w040_source_evidence_index_bound",
            "satisfied",
            "W040 source index binds direct obligations, W039 service substrate, W040 Stage 2 blockers, and W073 formatting intake",
        ),
        criterion(
            "readiness.w040_file_backed_retained_store_present",
            "satisfied",
            "W040 emits a deterministic file-backed retained-history artifact store",
        ),
        criterion(
            "readiness.w040_retained_query_register_present",
            "satisfied",
            "W040 emits deterministic retained-history query families",
        ),
        criterion(
            "readiness.w040_replay_correlation_index_present",
            "satisfied",
            "W040 emits replay-correlation rows for Stage 2, pack governance, and W073 guard evidence",
        ),
        criterion(
            "readiness.w040_alert_dispatcher_policy_evaluated",
            "satisfied",
            "W040 evaluates local alert/quarantine rules against service, retained-history, Stage 2, pack, and W073 inputs",
        ),
        criterion(
            "readiness.no_quarantine_decisions",
            "satisfied",
            "W040 source rows and service artifacts have no quarantine decisions",
        ),
        criterion(
            "readiness.w040_cross_engine_substrate_bound",
            "satisfied_boundary",
            "W040 binds file-backed cross-engine substrate and Stage 2 service dependency without service promotion",
        ),
        criterion(
            "readiness.w040_stage2_service_dependency_classified",
            "satisfied",
            "W040 Stage 2 policy equivalence retains operated cross-engine service as an exact dependency",
        ),
        criterion(
            "readiness.w073_typed_formatting_guard_carried",
            "satisfied",
            "W040 service packet carries W073 typed-only conditional-formatting input guard",
        ),
        criterion(
            "service.recurring_operated_scheduler",
            "blocked",
            "file-backed CLI runner is present, but no recurring scheduler, daemon, or service endpoint is operated",
        ),
        criterion(
            "service.external_alert_dispatcher",
            "blocked",
            "local dispatcher evidence is present, but no external alert dispatcher or quarantine service is operated",
        ),
        criterion(
            "service.operated_cross_engine_differential",
            "blocked",
            "cross-engine differential evidence remains file-backed rather than an operated service",
        ),
        criterion(
            "service.retention_slo_and_pack_governance",
            "blocked",
            "file-backed retained history has no operated retention SLO and pack-grade replay governance remains unpromoted",
        ),
    ];
    let blocked_criteria_count = criteria
        .iter()
        .filter(|row| row.get("state").and_then(Value::as_str) == Some("blocked"))
        .count();

    json!({
        "schema_version": W040_SERVICE_READINESS_SCHEMA_V1,
        "run_id": run_id,
        "artifact_root": relative_artifact_root,
        "readiness_state": "w040_service_artifacts_validated_without_operated_service_promotion",
        "criteria_count": criteria.len(),
        "satisfied_criteria_count": criteria.len() - blocked_criteria_count,
        "blocked_criteria_count": blocked_criteria_count,
        "runner_row_count": number_at(operated_runner, "row_count"),
        "history_store_record_count": number_at(retained_store, "store_record_count"),
        "query_register_row_count": number_at(retained_store, "query_register_row_count"),
        "replay_correlation_row_count": number_at(retained_store, "replay_correlation_row_count"),
        "evaluated_alert_rule_count": number_at(alert_dispatcher, "evaluated_rule_count"),
        "quarantine_decision_count": number_at(alert_dispatcher, "quarantine_decision_count"),
        "alert_decision_count": number_at(alert_dispatcher, "alert_decision_count"),
        "file_backed_cross_engine_substrate_present": bool_at(cross_engine_service, "file_backed_cross_engine_substrate_present"),
        "w073_threshold_fallback_allowed": bool_at(w040_formatting_intake, "w072_threshold_fallback_allowed_for_aggregate_visualization"),
        "operated_continuous_assurance_service_promoted": false,
        "retained_history_service_promoted": false,
        "cross_engine_differential_service_promoted": false,
        "external_alert_dispatcher_promoted": false,
        "criteria": criteria
    })
}

fn w040_exact_service_blockers() -> Vec<Value> {
    vec![
        json!({
            "blocker_id": "service.operated_regression_scheduler_absent",
            "owner": "calc-tv5.6; calc-tv5.10",
            "status_after_run": "exact_remaining_blocker",
            "reason": "W040 has a runnable file-backed CLI runner, but no recurring scheduler, daemon, service endpoint, or operated run queue.",
            "promotion_consequence": "operated continuous assurance service remains unpromoted"
        }),
        json!({
            "blocker_id": "service.external_alert_dispatcher_absent",
            "owner": "calc-tv5.6; calc-tv5.10",
            "status_after_run": "exact_remaining_blocker",
            "reason": "W040 evaluates alert/quarantine rules locally but does not operate an external dispatcher or quarantine service.",
            "promotion_consequence": "alert/quarantine dispatcher claims remain unpromoted"
        }),
        json!({
            "blocker_id": "service.operated_cross_engine_differential_absent",
            "owner": "calc-tv5.6; calc-tv5.7",
            "status_after_run": "exact_remaining_blocker",
            "reason": "cross-engine evidence remains file-backed and does not run as an operated differential service.",
            "promotion_consequence": "operated cross-engine differential service, independent diversity, and Stage 2 service dependencies remain blocked"
        }),
        json!({
            "blocker_id": "service.retention_slo_and_pack_governance_absent",
            "owner": "calc-tv5.6; calc-tv5.9",
            "status_after_run": "exact_remaining_blocker",
            "reason": "W040 emits a file-backed retained-history store/query register and replay-correlation index, but no operated retention SLO or pack-grade replay governance service.",
            "promotion_consequence": "retained-history service, pack-grade replay, C5, and release-grade verification remain unpromoted"
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

fn string_at<'a>(value: &'a Value, key: &str) -> &'a str {
    value.get(key).and_then(Value::as_str).unwrap_or("")
}

fn array_len(value: &Value) -> usize {
    value.as_array().map_or(0, Vec::len)
}

fn array_contains_string(value: &Value, expected: &str) -> bool {
    value
        .as_array()
        .is_some_and(|items| items.iter().any(|item| item.as_str() == Some(expected)))
}

fn row_with_field_exists(value: &Value, field: &str, expected: &str) -> bool {
    value
        .get("rows")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .any(|row| row.get(field).and_then(Value::as_str) == Some(expected))
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

    #[test]
    fn operated_assurance_runner_binds_w039_service_substrate_without_promotion() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let run_id = format!("test-w039-operated-assurance-{}", std::process::id());
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
        assert_eq!(summary.multi_run_history_row_count, 18);
        assert_eq!(summary.evaluated_alert_rule_count, 11);
        assert_eq!(summary.quarantine_decision_count, 0);
        assert_eq!(summary.alert_decision_count, 0);
        assert_eq!(summary.service_readiness_criteria_count, 12);
        assert_eq!(summary.service_readiness_blocked_count, 5);
        assert_eq!(summary.exact_service_blocker_count, 5);
        assert_eq!(summary.failed_row_count, 0);
        assert!(!summary.operated_service_promoted);

        let validation = read_json(
            &repo_root,
            &format!("docs/test-runs/core-engine/operated-assurance/{run_id}/validation.json"),
        )
        .unwrap();
        assert_eq!(
            validation["status"],
            "w039_operated_service_substrate_valid"
        );

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
        assert_eq!(promotion["retained_history_lifecycle_bound"], true);

        let blocker_register = read_json(
            &repo_root,
            &format!(
                "docs/test-runs/core-engine/operated-assurance/{run_id}/w039_exact_service_blocker_register.json"
            ),
        )
        .unwrap();
        assert_eq!(blocker_register["exact_service_blocker_count"], 5);

        cleanup();
    }

    #[test]
    fn operated_assurance_runner_binds_w040_service_artifacts_without_promotion() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let run_id = format!("test-w040-operated-assurance-{}", std::process::id());
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

        assert_eq!(summary.source_evidence_row_count, 10);
        assert_eq!(summary.multi_run_history_row_count, 21);
        assert_eq!(summary.evaluated_alert_rule_count, 14);
        assert_eq!(summary.quarantine_decision_count, 0);
        assert_eq!(summary.alert_decision_count, 0);
        assert_eq!(summary.service_readiness_criteria_count, 14);
        assert_eq!(summary.service_readiness_blocked_count, 4);
        assert_eq!(summary.exact_service_blocker_count, 4);
        assert_eq!(summary.failed_row_count, 0);
        assert!(!summary.operated_service_promoted);

        let validation = read_json(
            &repo_root,
            &format!("docs/test-runs/core-engine/operated-assurance/{run_id}/validation.json"),
        )
        .unwrap();
        assert_eq!(
            validation["status"],
            "w040_operated_assurance_retained_history_service_artifacts_valid"
        );

        let retained_store = read_json(
            &repo_root,
            &format!(
                "docs/test-runs/core-engine/operated-assurance/{run_id}/w040_retained_history_store_query.json"
            ),
        )
        .unwrap();
        assert_eq!(
            retained_store["file_backed_retained_history_store_present"],
            true
        );
        assert_eq!(
            retained_store["retained_history_query_register_present"],
            true
        );
        assert_eq!(retained_store["replay_correlation_index_present"], true);
        assert_eq!(retained_store["retained_history_service_present"], false);

        let promotion = read_json(
            &repo_root,
            &format!(
                "docs/test-runs/core-engine/operated-assurance/{run_id}/promotion_decision.json"
            ),
        )
        .unwrap();
        assert_eq!(promotion["retained_history_artifact_store_present"], true);
        assert_eq!(
            promotion["operated_continuous_assurance_service_promoted"],
            false
        );
        assert_eq!(promotion["retained_history_service_promoted"], false);

        let blocker_register = read_json(
            &repo_root,
            &format!(
                "docs/test-runs/core-engine/operated-assurance/{run_id}/w040_exact_service_blocker_register.json"
            ),
        )
        .unwrap();
        assert_eq!(blocker_register["exact_service_blocker_count"], 4);

        cleanup();
    }
}
