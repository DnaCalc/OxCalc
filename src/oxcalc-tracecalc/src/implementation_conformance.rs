#![forbid(unsafe_code)]

//! W035/W036/W037 implementation-conformance packet emission.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde_json::{Value, json};
use thiserror::Error;

const IMPLEMENTATION_CONFORMANCE_RUN_SUMMARY_SCHEMA_V1: &str =
    "oxcalc.implementation_conformance.run_summary.v1";
const IMPLEMENTATION_CONFORMANCE_GAP_REGISTER_SCHEMA_V1: &str =
    "oxcalc.implementation_conformance.gap_disposition_register.v1";
const IMPLEMENTATION_CONFORMANCE_EVIDENCE_SCHEMA_V1: &str =
    "oxcalc.implementation_conformance.evidence_summary.v1";
const IMPLEMENTATION_CONFORMANCE_VALIDATION_SCHEMA_V1: &str =
    "oxcalc.implementation_conformance.validation.v1";
const IMPLEMENTATION_CONFORMANCE_W036_ACTION_REGISTER_SCHEMA_V1: &str =
    "oxcalc.implementation_conformance.w036_action_register.v1";
const IMPLEMENTATION_CONFORMANCE_W036_BLOCKER_REGISTER_SCHEMA_V1: &str =
    "oxcalc.implementation_conformance.w036_blocker_register.v1";
const IMPLEMENTATION_CONFORMANCE_W036_MATCH_GUARD_SCHEMA_V1: &str =
    "oxcalc.implementation_conformance.w036_match_guard.v1";
const IMPLEMENTATION_CONFORMANCE_W037_DECISION_REGISTER_SCHEMA_V1: &str =
    "oxcalc.implementation_conformance.w037_decision_register.v1";
const IMPLEMENTATION_CONFORMANCE_W037_BLOCKER_REGISTER_SCHEMA_V1: &str =
    "oxcalc.implementation_conformance.w037_blocker_register.v1";
const IMPLEMENTATION_CONFORMANCE_W037_MATCH_GUARD_SCHEMA_V1: &str =
    "oxcalc.implementation_conformance.w037_match_guard.v1";

const W034_INDEPENDENT_CONFORMANCE_RUN_ID: &str = "w034-independent-conformance-001";
const W034_TREECALC_RUN_ID: &str = "w034-independent-conformance-treecalc-001";
const W035_ORACLE_MATRIX_RUN_ID: &str = "w035-tracecalc-oracle-matrix-001";
const W035_IMPLEMENTATION_CONFORMANCE_RUN_ID: &str =
    "w035-implementation-conformance-hardening-001";
const W036_ORACLE_MATRIX_RUN_ID: &str = "w036-tracecalc-coverage-closure-001";
const W036_IMPLEMENTATION_CONFORMANCE_RUN_ID: &str = "w036-implementation-conformance-closure-001";
const W037_ORACLE_MATRIX_RUN_ID: &str = "w037-tracecalc-observable-closure-001";
const W037_TREECALC_RUN_ID: &str = "w037-optimized-core-conformance-treecalc-001";

#[derive(Debug, Error)]
pub enum ImplementationConformanceError {
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
pub struct ImplementationConformanceRunSummary {
    pub run_id: String,
    pub schema_version: String,
    pub gap_disposition_row_count: usize,
    pub implementation_work_count: usize,
    pub spec_evolution_deferral_count: usize,
    pub validated_row_count: usize,
    pub failed_row_count: usize,
    pub w036_action_row_count: usize,
    pub w036_first_fix_row_count: usize,
    pub w036_blocker_routed_row_count: usize,
    pub w036_match_promoted_count: usize,
    pub w037_decision_row_count: usize,
    pub w037_fixed_or_promoted_count: usize,
    pub w037_residual_blocker_count: usize,
    pub w037_match_promoted_count: usize,
    pub artifact_root: String,
}

#[derive(Debug, Clone, Default)]
pub struct ImplementationConformanceRunner;

#[derive(Debug, Clone)]
struct GapDispositionSpec {
    row_id: &'static str,
    source_gap_classification: &'static str,
    disposition_kind: &'static str,
    disposition: &'static str,
    authority_owner: &'static str,
    carry_forward_lane: &'static str,
    reason: &'static str,
    w035_matrix_row_ids: &'static [&'static str],
}

#[derive(Debug, Clone)]
struct EvaluatedDispositionRow {
    row: Value,
    disposition_kind: &'static str,
    valid: bool,
}

#[derive(Debug, Clone)]
struct W036ClosureSpec {
    row_id: &'static str,
    source_w034_gap_row_id: &'static str,
    source_w035_disposition_kind: &'static str,
    w036_obligation_id: &'static str,
    w036_disposition_kind: &'static str,
    w036_disposition: &'static str,
    conformance_match_state: &'static str,
    first_fix_state: &'static str,
    implementation_evidence_state: &'static str,
    blocker_bead: Option<&'static str>,
    authority_owner: &'static str,
    promotion_consequence: &'static str,
    reason: &'static str,
    implementation_evidence_sources: &'static [&'static str],
    w036_matrix_row_ids: &'static [&'static str],
}

#[derive(Debug, Clone)]
struct EvaluatedW036ClosureRow {
    row: Value,
    first_fix: bool,
    blocker_routed: bool,
    match_promoted: bool,
    valid: bool,
}

#[derive(Debug, Clone)]
struct W037DecisionSpec {
    row_id: &'static str,
    source_w036_action_row_id: &'static str,
    source_w036_disposition_kind: &'static str,
    w037_obligation_id: &'static str,
    w037_decision_kind: &'static str,
    w037_decision: &'static str,
    conformance_match_state: &'static str,
    implementation_evidence_state: &'static str,
    residual_blocker_bead: Option<&'static str>,
    authority_owner: &'static str,
    promotion_consequence: &'static str,
    reason: &'static str,
    implementation_evidence_sources: &'static [&'static str],
    w037_matrix_row_ids: &'static [&'static str],
    required_treecalc_case_id: Option<&'static str>,
}

#[derive(Debug, Clone)]
struct EvaluatedW037DecisionRow {
    row: Value,
    fixed_or_promoted: bool,
    residual_blocker: bool,
    match_promoted: bool,
    valid: bool,
}

impl ImplementationConformanceRunner {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    pub fn execute(
        &self,
        repo_root: &Path,
        run_id: &str,
    ) -> Result<ImplementationConformanceRunSummary, ImplementationConformanceError> {
        if run_id.contains("w037") {
            return self.execute_w037(repo_root, run_id);
        }
        if run_id.contains("w036") {
            return self.execute_w036(repo_root, run_id);
        }

        self.execute_w035(repo_root, run_id)
    }

    fn execute_w035(
        &self,
        repo_root: &Path,
        run_id: &str,
    ) -> Result<ImplementationConformanceRunSummary, ImplementationConformanceError> {
        let artifact_root = repo_root.join(format!(
            "docs/test-runs/core-engine/implementation-conformance/{run_id}"
        ));
        let relative_artifact_root = relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "implementation-conformance",
            run_id,
        ]);
        if artifact_root.exists() {
            fs::remove_dir_all(&artifact_root).map_err(|source| {
                ImplementationConformanceError::RemoveDirectory {
                    path: artifact_root.display().to_string(),
                    source,
                }
            })?;
        }
        create_directory(&artifact_root)?;

        let w034_summary_path = relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "independent-conformance",
            W034_INDEPENDENT_CONFORMANCE_RUN_ID,
            "run_summary.json",
        ]);
        let w034_diff_path = relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "independent-conformance",
            W034_INDEPENDENT_CONFORMANCE_RUN_ID,
            "comparisons",
            "treecalc_tracecalc_differential.json",
        ]);
        let w034_core_projection_path = relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "independent-conformance",
            W034_INDEPENDENT_CONFORMANCE_RUN_ID,
            "comparisons",
            "core_engine_projection_differential.json",
        ]);
        let treecalc_summary_path = relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "treecalc-local",
            W034_TREECALC_RUN_ID,
            "run_summary.json",
        ]);
        let matrix_summary_path = relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "tracecalc-reference-machine",
            W035_ORACLE_MATRIX_RUN_ID,
            "oracle-matrix",
            "run_summary.json",
        ]);
        let matrix_path = relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "tracecalc-reference-machine",
            W035_ORACLE_MATRIX_RUN_ID,
            "oracle-matrix",
            "coverage_matrix.json",
        ]);

        let w034_summary = read_json(repo_root, &w034_summary_path)?;
        let w034_diff = read_json(repo_root, &w034_diff_path)?;
        let w034_core_projection = read_json(repo_root, &w034_core_projection_path)?;
        let treecalc_summary = read_json(repo_root, &treecalc_summary_path)?;
        let matrix_summary = read_json(repo_root, &matrix_summary_path)?;
        let matrix = read_json(repo_root, &matrix_path)?;

        let gap_rows = rows_by_id(&w034_diff, "row_id");
        let matrix_rows = rows_by_id(&matrix, "row_id");
        let evaluated_rows = GAP_DISPOSITION_SPECS
            .iter()
            .map(|spec| evaluate_disposition_row(spec, &gap_rows, &matrix_rows))
            .collect::<Vec<_>>();

        let implementation_work_count = evaluated_rows
            .iter()
            .filter(|row| row.disposition_kind == "implementation_work_deferred")
            .count();
        let spec_evolution_deferral_count = evaluated_rows
            .iter()
            .filter(|row| row.disposition_kind == "spec_evolution_deferral")
            .count();
        let validated_row_count = evaluated_rows.iter().filter(|row| row.valid).count();
        let failed_row_count = evaluated_rows.iter().filter(|row| !row.valid).count();

        let validation_failures = validation_failures(
            &w034_summary,
            &treecalc_summary,
            &matrix_summary,
            failed_row_count,
        );
        let validation_status = if validation_failures.is_empty() {
            "implementation_conformance_hardening_valid"
        } else {
            "implementation_conformance_hardening_failed"
        };

        write_json(
            &artifact_root.join("gap_disposition_register.json"),
            &json!({
                "schema_version": IMPLEMENTATION_CONFORMANCE_GAP_REGISTER_SCHEMA_V1,
                "run_id": run_id,
                "source_independent_conformance_run_id": W034_INDEPENDENT_CONFORMANCE_RUN_ID,
                "treecalc_reference_run_id": W034_TREECALC_RUN_ID,
                "w035_tracecalc_oracle_matrix_run_id": W035_ORACLE_MATRIX_RUN_ID,
                "row_count": evaluated_rows.len(),
                "implementation_work_count": implementation_work_count,
                "spec_evolution_deferral_count": spec_evolution_deferral_count,
                "validated_row_count": validated_row_count,
                "failed_row_count": failed_row_count,
                "rows": evaluated_rows.iter().map(|row| row.row.clone()).collect::<Vec<_>>(),
            }),
        )?;

        write_json(
            &artifact_root.join("evidence_summary.json"),
            &json!({
                "schema_version": IMPLEMENTATION_CONFORMANCE_EVIDENCE_SCHEMA_V1,
                "run_id": run_id,
                "source_paths": {
                    "w034_independent_conformance_summary": w034_summary_path,
                    "w034_treecalc_tracecalc_differential": w034_diff_path,
                    "w034_core_projection_differential": w034_core_projection_path,
                    "w034_treecalc_summary": treecalc_summary_path,
                    "w035_oracle_matrix_summary": matrix_summary_path,
                    "w035_oracle_matrix": matrix_path,
                },
                "w034_independent_conformance": {
                    "comparison_row_count": number_at(&w034_summary, "comparison_row_count"),
                    "declared_gap_count": number_at(&w034_summary, "declared_gap_count"),
                    "missing_artifact_count": number_at(&w034_summary, "missing_artifact_count"),
                    "unexpected_mismatch_count": number_at(&w034_summary, "unexpected_mismatch_count"),
                },
                "w034_treecalc_local": {
                    "case_count": number_at(&treecalc_summary, "case_count"),
                    "expectation_mismatch_count": number_at(&treecalc_summary, "expectation_mismatch_count"),
                    "result_counts": treecalc_summary.get("result_counts").cloned().unwrap_or_else(|| json!({})),
                },
                "w034_core_projection": {
                    "projection_row_count": number_at(&w034_core_projection, "projection_row_count"),
                    "projection_present_count": array_at(&w034_core_projection, "rows")
                        .iter()
                        .filter(|row| string_at(row, "projection_state") == "projection_present")
                        .count(),
                },
                "w035_oracle_matrix": {
                    "tracecalc_scenario_count": number_at(&matrix_summary, "tracecalc_scenario_count"),
                    "matrix_row_count": number_at(&matrix_summary, "matrix_row_count"),
                    "covered_row_count": number_at(&matrix_summary, "covered_row_count"),
                    "classified_uncovered_row_count": number_at(&matrix_summary, "classified_uncovered_row_count"),
                    "missing_or_failed_row_count": number_at(&matrix_summary, "missing_or_failed_row_count"),
                },
                "conformance_hardening": {
                    "gap_disposition_row_count": evaluated_rows.len(),
                    "implementation_work_count": implementation_work_count,
                    "spec_evolution_deferral_count": spec_evolution_deferral_count,
                    "validated_row_count": validated_row_count,
                    "failed_row_count": failed_row_count,
                },
            }),
        )?;

        write_json(
            &artifact_root.join("validation.json"),
            &json!({
                "schema_version": IMPLEMENTATION_CONFORMANCE_VALIDATION_SCHEMA_V1,
                "run_id": run_id,
                "status": validation_status,
                "validation_failures": validation_failures,
                "gap_disposition_row_count": evaluated_rows.len(),
                "validated_row_count": validated_row_count,
                "failed_row_count": failed_row_count,
            }),
        )?;

        let summary = ImplementationConformanceRunSummary {
            run_id: run_id.to_string(),
            schema_version: IMPLEMENTATION_CONFORMANCE_RUN_SUMMARY_SCHEMA_V1.to_string(),
            gap_disposition_row_count: evaluated_rows.len(),
            implementation_work_count,
            spec_evolution_deferral_count,
            validated_row_count,
            failed_row_count,
            w036_action_row_count: 0,
            w036_first_fix_row_count: 0,
            w036_blocker_routed_row_count: 0,
            w036_match_promoted_count: 0,
            w037_decision_row_count: 0,
            w037_fixed_or_promoted_count: 0,
            w037_residual_blocker_count: 0,
            w037_match_promoted_count: 0,
            artifact_root: relative_artifact_root.clone(),
        };
        write_json(
            &artifact_root.join("run_summary.json"),
            &json!({
                "schema_version": summary.schema_version,
                "run_id": summary.run_id,
                "gap_disposition_row_count": summary.gap_disposition_row_count,
                "implementation_work_count": summary.implementation_work_count,
                "spec_evolution_deferral_count": summary.spec_evolution_deferral_count,
                "validated_row_count": summary.validated_row_count,
                "failed_row_count": summary.failed_row_count,
                "w036_action_row_count": summary.w036_action_row_count,
                "w036_first_fix_row_count": summary.w036_first_fix_row_count,
                "w036_blocker_routed_row_count": summary.w036_blocker_routed_row_count,
                "w036_match_promoted_count": summary.w036_match_promoted_count,
                "w037_decision_row_count": summary.w037_decision_row_count,
                "w037_fixed_or_promoted_count": summary.w037_fixed_or_promoted_count,
                "w037_residual_blocker_count": summary.w037_residual_blocker_count,
                "w037_match_promoted_count": summary.w037_match_promoted_count,
                "artifact_root": summary.artifact_root,
                "gap_disposition_register_path": format!("{relative_artifact_root}/gap_disposition_register.json"),
                "evidence_summary_path": format!("{relative_artifact_root}/evidence_summary.json"),
                "validation_path": format!("{relative_artifact_root}/validation.json"),
            }),
        )?;

        Ok(summary)
    }

    fn execute_w036(
        &self,
        repo_root: &Path,
        run_id: &str,
    ) -> Result<ImplementationConformanceRunSummary, ImplementationConformanceError> {
        let artifact_root = repo_root.join(format!(
            "docs/test-runs/core-engine/implementation-conformance/{run_id}"
        ));
        let relative_artifact_root = relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "implementation-conformance",
            run_id,
        ]);
        if artifact_root.exists() {
            fs::remove_dir_all(&artifact_root).map_err(|source| {
                ImplementationConformanceError::RemoveDirectory {
                    path: artifact_root.display().to_string(),
                    source,
                }
            })?;
        }
        create_directory(&artifact_root)?;

        let w035_gap_register_path = relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "implementation-conformance",
            W035_IMPLEMENTATION_CONFORMANCE_RUN_ID,
            "gap_disposition_register.json",
        ]);
        let w034_core_projection_path = relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "independent-conformance",
            W034_INDEPENDENT_CONFORMANCE_RUN_ID,
            "comparisons",
            "core_engine_projection_differential.json",
        ]);
        let w036_matrix_summary_path = relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "tracecalc-reference-machine",
            W036_ORACLE_MATRIX_RUN_ID,
            "oracle-matrix",
            "run_summary.json",
        ]);
        let w036_matrix_path = relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "tracecalc-reference-machine",
            W036_ORACLE_MATRIX_RUN_ID,
            "oracle-matrix",
            "coverage_matrix.json",
        ]);

        let w035_gap_register = read_json(repo_root, &w035_gap_register_path)?;
        let w034_core_projection = read_json(repo_root, &w034_core_projection_path)?;
        let w036_matrix_summary = read_json(repo_root, &w036_matrix_summary_path)?;
        let w036_matrix = read_json(repo_root, &w036_matrix_path)?;

        let source_rows = rows_by_id(&w035_gap_register, "row_id");
        let matrix_rows = rows_by_id(&w036_matrix, "row_id");
        let evaluated_rows = W036_CLOSURE_SPECS
            .iter()
            .map(|spec| evaluate_w036_closure_row(spec, &source_rows, &matrix_rows))
            .collect::<Vec<_>>();

        let first_fix_row_count = evaluated_rows.iter().filter(|row| row.first_fix).count();
        let blocker_routed_row_count = evaluated_rows
            .iter()
            .filter(|row| row.blocker_routed)
            .count();
        let match_promoted_count = evaluated_rows
            .iter()
            .filter(|row| row.match_promoted)
            .count();
        let validated_row_count = evaluated_rows.iter().filter(|row| row.valid).count();
        let failed_row_count = evaluated_rows.iter().filter(|row| !row.valid).count();
        let implementation_work_count = number_at(&w035_gap_register, "implementation_work_count")
            .try_into()
            .unwrap_or_default();
        let spec_evolution_deferral_count =
            number_at(&w035_gap_register, "spec_evolution_deferral_count")
                .try_into()
                .unwrap_or_default();

        let validation_failures = w036_validation_failures(
            &w035_gap_register,
            &w036_matrix_summary,
            failed_row_count,
            match_promoted_count,
        );
        let validation_status = if validation_failures.is_empty() {
            "implementation_conformance_w036_closure_plan_valid"
        } else {
            "implementation_conformance_w036_closure_plan_failed"
        };

        let action_rows = evaluated_rows
            .iter()
            .map(|row| row.row.clone())
            .collect::<Vec<_>>();
        let blocker_rows = action_rows
            .iter()
            .filter(|row| !row["blocker_bead"].is_null())
            .cloned()
            .collect::<Vec<_>>();

        write_json(
            &artifact_root.join("w036_closure_action_register.json"),
            &json!({
                "schema_version": IMPLEMENTATION_CONFORMANCE_W036_ACTION_REGISTER_SCHEMA_V1,
                "run_id": run_id,
                "source_implementation_conformance_run_id": W035_IMPLEMENTATION_CONFORMANCE_RUN_ID,
                "w036_tracecalc_oracle_matrix_run_id": W036_ORACLE_MATRIX_RUN_ID,
                "source_gap_row_count": number_at(&w035_gap_register, "row_count"),
                "action_row_count": action_rows.len(),
                "first_fix_row_count": first_fix_row_count,
                "blocker_routed_row_count": blocker_routed_row_count,
                "match_promoted_count": match_promoted_count,
                "validated_row_count": validated_row_count,
                "failed_row_count": failed_row_count,
                "rows": action_rows,
            }),
        )?;

        write_json(
            &artifact_root.join("w036_blocker_register.json"),
            &json!({
                "schema_version": IMPLEMENTATION_CONFORMANCE_W036_BLOCKER_REGISTER_SCHEMA_V1,
                "run_id": run_id,
                "blocker_row_count": blocker_rows.len(),
                "rows": blocker_rows,
            }),
        )?;

        write_json(
            &artifact_root.join("w036_match_promotion_guard.json"),
            &json!({
                "schema_version": IMPLEMENTATION_CONFORMANCE_W036_MATCH_GUARD_SCHEMA_V1,
                "run_id": run_id,
                "source_declared_gap_row_count": number_at(&w035_gap_register, "row_count"),
                "promoted_match_count": match_promoted_count,
                "non_promoted_row_count": evaluated_rows.len().saturating_sub(match_promoted_count),
                "guard_status": if match_promoted_count == 0 {
                    "no_declared_gap_promoted_as_match"
                } else {
                    "declared_gap_match_promotion_present"
                },
                "policy": "W036 calc-rqq.3 forbids counting a W035 declared gap as an optimized/core-engine match without replay/diff evidence; this run promotes no declared gaps as matches.",
            }),
        )?;

        write_json(
            &artifact_root.join("evidence_summary.json"),
            &json!({
                "schema_version": IMPLEMENTATION_CONFORMANCE_EVIDENCE_SCHEMA_V1,
                "run_id": run_id,
                "source_paths": {
                    "w035_gap_disposition_register": w035_gap_register_path,
                    "w034_core_projection_differential": w034_core_projection_path,
                    "w036_oracle_matrix_summary": w036_matrix_summary_path,
                    "w036_oracle_matrix": w036_matrix_path,
                },
                "w035_gap_dispositions": {
                    "row_count": number_at(&w035_gap_register, "row_count"),
                    "implementation_work_count": number_at(&w035_gap_register, "implementation_work_count"),
                    "spec_evolution_deferral_count": number_at(&w035_gap_register, "spec_evolution_deferral_count"),
                    "failed_row_count": number_at(&w035_gap_register, "failed_row_count"),
                },
                "w034_core_projection": {
                    "projection_row_count": number_at(&w034_core_projection, "projection_row_count"),
                    "projection_present_count": array_at(&w034_core_projection, "rows")
                        .iter()
                        .filter(|row| string_at(row, "projection_state") == "projection_present")
                        .count(),
                },
                "w036_oracle_matrix": {
                    "matrix_row_count": number_at(&w036_matrix_summary, "matrix_row_count"),
                    "covered_row_count": number_at(&w036_matrix_summary, "covered_row_count"),
                    "classified_uncovered_row_count": number_at(&w036_matrix_summary, "classified_uncovered_row_count"),
                    "excluded_row_count": number_at(&w036_matrix_summary, "excluded_row_count"),
                    "missing_or_failed_row_count": number_at(&w036_matrix_summary, "missing_or_failed_row_count"),
                },
                "w036_closure_actions": {
                    "action_row_count": evaluated_rows.len(),
                    "first_fix_row_count": first_fix_row_count,
                    "blocker_routed_row_count": blocker_routed_row_count,
                    "match_promoted_count": match_promoted_count,
                    "validated_row_count": validated_row_count,
                    "failed_row_count": failed_row_count,
                },
            }),
        )?;

        write_json(
            &artifact_root.join("validation.json"),
            &json!({
                "schema_version": IMPLEMENTATION_CONFORMANCE_VALIDATION_SCHEMA_V1,
                "run_id": run_id,
                "status": validation_status,
                "validation_failures": validation_failures,
                "source_gap_row_count": number_at(&w035_gap_register, "row_count"),
                "w036_action_row_count": evaluated_rows.len(),
                "validated_row_count": validated_row_count,
                "failed_row_count": failed_row_count,
                "match_promoted_count": match_promoted_count,
            }),
        )?;

        let summary = ImplementationConformanceRunSummary {
            run_id: run_id.to_string(),
            schema_version: IMPLEMENTATION_CONFORMANCE_RUN_SUMMARY_SCHEMA_V1.to_string(),
            gap_disposition_row_count: evaluated_rows.len(),
            implementation_work_count,
            spec_evolution_deferral_count,
            validated_row_count,
            failed_row_count,
            w036_action_row_count: evaluated_rows.len(),
            w036_first_fix_row_count: first_fix_row_count,
            w036_blocker_routed_row_count: blocker_routed_row_count,
            w036_match_promoted_count: match_promoted_count,
            w037_decision_row_count: 0,
            w037_fixed_or_promoted_count: 0,
            w037_residual_blocker_count: 0,
            w037_match_promoted_count: 0,
            artifact_root: relative_artifact_root.clone(),
        };
        write_json(
            &artifact_root.join("run_summary.json"),
            &json!({
                "schema_version": summary.schema_version,
                "run_id": summary.run_id,
                "gap_disposition_row_count": summary.gap_disposition_row_count,
                "implementation_work_count": summary.implementation_work_count,
                "spec_evolution_deferral_count": summary.spec_evolution_deferral_count,
                "validated_row_count": summary.validated_row_count,
                "failed_row_count": summary.failed_row_count,
                "w036_action_row_count": summary.w036_action_row_count,
                "w036_first_fix_row_count": summary.w036_first_fix_row_count,
                "w036_blocker_routed_row_count": summary.w036_blocker_routed_row_count,
                "w036_match_promoted_count": summary.w036_match_promoted_count,
                "w037_decision_row_count": summary.w037_decision_row_count,
                "w037_fixed_or_promoted_count": summary.w037_fixed_or_promoted_count,
                "w037_residual_blocker_count": summary.w037_residual_blocker_count,
                "w037_match_promoted_count": summary.w037_match_promoted_count,
                "artifact_root": summary.artifact_root,
                "w036_closure_action_register_path": format!("{relative_artifact_root}/w036_closure_action_register.json"),
                "w036_blocker_register_path": format!("{relative_artifact_root}/w036_blocker_register.json"),
                "w036_match_promotion_guard_path": format!("{relative_artifact_root}/w036_match_promotion_guard.json"),
                "evidence_summary_path": format!("{relative_artifact_root}/evidence_summary.json"),
                "validation_path": format!("{relative_artifact_root}/validation.json"),
            }),
        )?;

        Ok(summary)
    }

    fn execute_w037(
        &self,
        repo_root: &Path,
        run_id: &str,
    ) -> Result<ImplementationConformanceRunSummary, ImplementationConformanceError> {
        let artifact_root = repo_root.join(format!(
            "docs/test-runs/core-engine/implementation-conformance/{run_id}"
        ));
        let relative_artifact_root = relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "implementation-conformance",
            run_id,
        ]);
        if artifact_root.exists() {
            fs::remove_dir_all(&artifact_root).map_err(|source| {
                ImplementationConformanceError::RemoveDirectory {
                    path: artifact_root.display().to_string(),
                    source,
                }
            })?;
        }
        create_directory(&artifact_root)?;

        let w036_action_register_path = relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "implementation-conformance",
            W036_IMPLEMENTATION_CONFORMANCE_RUN_ID,
            "w036_closure_action_register.json",
        ]);
        let w037_matrix_summary_path = relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "tracecalc-reference-machine",
            W037_ORACLE_MATRIX_RUN_ID,
            "oracle-matrix",
            "run_summary.json",
        ]);
        let w037_matrix_path = relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "tracecalc-reference-machine",
            W037_ORACLE_MATRIX_RUN_ID,
            "oracle-matrix",
            "coverage_matrix.json",
        ]);
        let w037_treecalc_summary_path = relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "treecalc-local",
            W037_TREECALC_RUN_ID,
            "run_summary.json",
        ]);

        let w036_action_register = read_json(repo_root, &w036_action_register_path)?;
        let w037_matrix_summary = read_json(repo_root, &w037_matrix_summary_path)?;
        let w037_matrix = read_json(repo_root, &w037_matrix_path)?;
        let w037_treecalc_summary = read_json(repo_root, &w037_treecalc_summary_path)?;

        let source_rows = rows_by_id(&w036_action_register, "row_id");
        let matrix_rows = rows_by_id(&w037_matrix, "row_id");
        let evaluated_rows = W037_DECISION_SPECS
            .iter()
            .map(|spec| {
                evaluate_w037_decision_row(
                    repo_root,
                    spec,
                    &source_rows,
                    &matrix_rows,
                    &w037_treecalc_summary,
                )
            })
            .collect::<Result<Vec<_>, _>>()?;

        let fixed_or_promoted_count = evaluated_rows
            .iter()
            .filter(|row| row.fixed_or_promoted)
            .count();
        let residual_blocker_count = evaluated_rows
            .iter()
            .filter(|row| row.residual_blocker)
            .count();
        let match_promoted_count = evaluated_rows
            .iter()
            .filter(|row| row.match_promoted)
            .count();
        let validated_row_count = evaluated_rows.iter().filter(|row| row.valid).count();
        let failed_row_count = evaluated_rows.iter().filter(|row| !row.valid).count();
        let implementation_work_count = number_at(&w036_action_register, "first_fix_row_count")
            .try_into()
            .unwrap_or_default();
        let spec_evolution_deferral_count = array_at(&w036_action_register, "rows")
            .iter()
            .filter(|row| string_at(row, "w036_disposition_kind").contains("deferral"))
            .count();

        let decision_rows = evaluated_rows
            .iter()
            .map(|row| row.row.clone())
            .collect::<Vec<_>>();
        let residual_rows = decision_rows
            .iter()
            .filter(|row| !row["residual_blocker_bead"].is_null())
            .cloned()
            .collect::<Vec<_>>();

        let validation_failures = w037_validation_failures(
            &w036_action_register,
            &w037_matrix_summary,
            &w037_treecalc_summary,
            failed_row_count,
            match_promoted_count,
            residual_blocker_count,
        );
        let validation_status = if validation_failures.is_empty() {
            "implementation_conformance_w037_decisions_valid"
        } else {
            "implementation_conformance_w037_decisions_failed"
        };

        write_json(
            &artifact_root.join("w037_conformance_decision_register.json"),
            &json!({
                "schema_version": IMPLEMENTATION_CONFORMANCE_W037_DECISION_REGISTER_SCHEMA_V1,
                "run_id": run_id,
                "source_w036_implementation_conformance_run_id": W036_IMPLEMENTATION_CONFORMANCE_RUN_ID,
                "w037_tracecalc_oracle_matrix_run_id": W037_ORACLE_MATRIX_RUN_ID,
                "w037_treecalc_reference_run_id": W037_TREECALC_RUN_ID,
                "decision_row_count": decision_rows.len(),
                "fixed_or_promoted_count": fixed_or_promoted_count,
                "residual_blocker_count": residual_blocker_count,
                "match_promoted_count": match_promoted_count,
                "validated_row_count": validated_row_count,
                "failed_row_count": failed_row_count,
                "rows": decision_rows,
            }),
        )?;

        write_json(
            &artifact_root.join("w037_residual_blocker_register.json"),
            &json!({
                "schema_version": IMPLEMENTATION_CONFORMANCE_W037_BLOCKER_REGISTER_SCHEMA_V1,
                "run_id": run_id,
                "residual_blocker_count": residual_rows.len(),
                "rows": residual_rows,
            }),
        )?;

        write_json(
            &artifact_root.join("w037_match_promotion_guard.json"),
            &json!({
                "schema_version": IMPLEMENTATION_CONFORMANCE_W037_MATCH_GUARD_SCHEMA_V1,
                "run_id": run_id,
                "source_declared_gap_row_count": number_at(&w036_action_register, "action_row_count"),
                "promoted_match_count": match_promoted_count,
                "allowed_promoted_rows": ["w037_decision_dynamic_dependency_bind_projection_fixed"],
                "non_promoted_row_count": evaluated_rows.len().saturating_sub(match_promoted_count),
                "guard_status": if match_promoted_count == 1 {
                    "w037_declared_gap_promotion_guard_holds"
                } else {
                    "w037_declared_gap_promotion_guard_failed"
                },
                "policy": "W037 calc-ubd.3 permits a declared-gap row to become a conformance match only when new direct TreeCalc differential evidence and TraceCalc replay evidence are both bound. All other declared gaps remain non-promoted residual blockers.",
            }),
        )?;

        write_json(
            &artifact_root.join("evidence_summary.json"),
            &json!({
                "schema_version": IMPLEMENTATION_CONFORMANCE_EVIDENCE_SCHEMA_V1,
                "run_id": run_id,
                "source_paths": {
                    "w036_action_register": w036_action_register_path,
                    "w037_oracle_matrix_summary": w037_matrix_summary_path,
                    "w037_oracle_matrix": w037_matrix_path,
                    "w037_treecalc_summary": w037_treecalc_summary_path,
                    "w037_dynamic_resolved_treecalc_result": relative_artifact_path([
                        "docs",
                        "test-runs",
                        "core-engine",
                        "treecalc-local",
                        W037_TREECALC_RUN_ID,
                        "cases",
                        "tc_local_dynamic_resolved_publish_001",
                        "result.json",
                    ]),
                },
                "w036_actions": {
                    "action_row_count": number_at(&w036_action_register, "action_row_count"),
                    "first_fix_row_count": number_at(&w036_action_register, "first_fix_row_count"),
                    "blocker_routed_row_count": number_at(&w036_action_register, "blocker_routed_row_count"),
                    "match_promoted_count": number_at(&w036_action_register, "match_promoted_count"),
                    "failed_row_count": number_at(&w036_action_register, "failed_row_count"),
                },
                "w037_oracle_matrix": {
                    "matrix_row_count": number_at(&w037_matrix_summary, "matrix_row_count"),
                    "covered_row_count": number_at(&w037_matrix_summary, "covered_row_count"),
                    "classified_uncovered_row_count": number_at(&w037_matrix_summary, "classified_uncovered_row_count"),
                    "excluded_row_count": number_at(&w037_matrix_summary, "excluded_row_count"),
                    "missing_or_failed_row_count": number_at(&w037_matrix_summary, "missing_or_failed_row_count"),
                },
                "w037_treecalc_local": {
                    "case_count": number_at(&w037_treecalc_summary, "case_count"),
                    "expectation_mismatch_count": number_at(&w037_treecalc_summary, "expectation_mismatch_count"),
                    "result_counts": w037_treecalc_summary.get("result_counts").cloned().unwrap_or_else(|| json!({})),
                },
                "w037_decisions": {
                    "decision_row_count": evaluated_rows.len(),
                    "fixed_or_promoted_count": fixed_or_promoted_count,
                    "residual_blocker_count": residual_blocker_count,
                    "match_promoted_count": match_promoted_count,
                    "validated_row_count": validated_row_count,
                    "failed_row_count": failed_row_count,
                },
            }),
        )?;

        write_json(
            &artifact_root.join("validation.json"),
            &json!({
                "schema_version": IMPLEMENTATION_CONFORMANCE_VALIDATION_SCHEMA_V1,
                "run_id": run_id,
                "status": validation_status,
                "validation_failures": validation_failures,
                "source_w036_action_row_count": number_at(&w036_action_register, "action_row_count"),
                "w037_decision_row_count": evaluated_rows.len(),
                "validated_row_count": validated_row_count,
                "failed_row_count": failed_row_count,
                "fixed_or_promoted_count": fixed_or_promoted_count,
                "residual_blocker_count": residual_blocker_count,
                "match_promoted_count": match_promoted_count,
            }),
        )?;

        let summary = ImplementationConformanceRunSummary {
            run_id: run_id.to_string(),
            schema_version: IMPLEMENTATION_CONFORMANCE_RUN_SUMMARY_SCHEMA_V1.to_string(),
            gap_disposition_row_count: evaluated_rows.len(),
            implementation_work_count,
            spec_evolution_deferral_count,
            validated_row_count,
            failed_row_count,
            w036_action_row_count: 0,
            w036_first_fix_row_count: 0,
            w036_blocker_routed_row_count: 0,
            w036_match_promoted_count: 0,
            w037_decision_row_count: evaluated_rows.len(),
            w037_fixed_or_promoted_count: fixed_or_promoted_count,
            w037_residual_blocker_count: residual_blocker_count,
            w037_match_promoted_count: match_promoted_count,
            artifact_root: relative_artifact_root.clone(),
        };
        write_json(
            &artifact_root.join("run_summary.json"),
            &json!({
                "schema_version": summary.schema_version,
                "run_id": summary.run_id,
                "gap_disposition_row_count": summary.gap_disposition_row_count,
                "implementation_work_count": summary.implementation_work_count,
                "spec_evolution_deferral_count": summary.spec_evolution_deferral_count,
                "validated_row_count": summary.validated_row_count,
                "failed_row_count": summary.failed_row_count,
                "w036_action_row_count": summary.w036_action_row_count,
                "w036_first_fix_row_count": summary.w036_first_fix_row_count,
                "w036_blocker_routed_row_count": summary.w036_blocker_routed_row_count,
                "w036_match_promoted_count": summary.w036_match_promoted_count,
                "w037_decision_row_count": summary.w037_decision_row_count,
                "w037_fixed_or_promoted_count": summary.w037_fixed_or_promoted_count,
                "w037_residual_blocker_count": summary.w037_residual_blocker_count,
                "w037_match_promoted_count": summary.w037_match_promoted_count,
                "artifact_root": summary.artifact_root,
                "w037_conformance_decision_register_path": format!("{relative_artifact_root}/w037_conformance_decision_register.json"),
                "w037_residual_blocker_register_path": format!("{relative_artifact_root}/w037_residual_blocker_register.json"),
                "w037_match_promotion_guard_path": format!("{relative_artifact_root}/w037_match_promotion_guard.json"),
                "evidence_summary_path": format!("{relative_artifact_root}/evidence_summary.json"),
                "validation_path": format!("{relative_artifact_root}/validation.json"),
            }),
        )?;

        Ok(summary)
    }
}

fn evaluate_disposition_row(
    spec: &GapDispositionSpec,
    gap_rows: &BTreeMap<String, Value>,
    matrix_rows: &BTreeMap<String, Value>,
) -> EvaluatedDispositionRow {
    let mut failures = Vec::new();
    let gap_row = gap_rows.get(spec.row_id);
    if let Some(row) = gap_row {
        if string_at(row, "comparison_state") != "declared_capability_gap" {
            failures.push("source_row_not_declared_capability_gap".to_string());
        }
        if string_pointer(row, "/details/gap_classification") != spec.source_gap_classification {
            failures.push("source_gap_classification_mismatch".to_string());
        }
        if !array_at(row, "failures").is_empty() {
            failures.push("source_gap_row_has_failures".to_string());
        }
        if !array_at(row, "missing_artifacts").is_empty() {
            failures.push("source_gap_row_has_missing_artifacts".to_string());
        }
    } else {
        failures.push("source_gap_row_missing".to_string());
    }

    let mut matrix_evidence = Vec::new();
    for matrix_row_id in spec.w035_matrix_row_ids {
        if let Some(matrix_row) = matrix_rows.get(*matrix_row_id) {
            let evidence_state = string_at(matrix_row, "evidence_state");
            if !matches!(
                evidence_state.as_str(),
                "covered_passed" | "classified_uncovered_deferred"
            ) {
                failures.push(format!("matrix_row_invalid_state:{matrix_row_id}"));
            }
            matrix_evidence.push(json!({
                "row_id": matrix_row_id,
                "obligation_id": matrix_row["obligation_id"],
                "evidence_state": evidence_state,
                "classification": matrix_row["classification"],
                "scenario_id": matrix_row["scenario_id"],
            }));
        } else {
            failures.push(format!("matrix_row_missing:{matrix_row_id}"));
        }
    }

    EvaluatedDispositionRow {
        row: json!({
            "row_id": spec.row_id,
            "source_gap_classification": spec.source_gap_classification,
            "disposition_kind": spec.disposition_kind,
            "disposition": spec.disposition,
            "authority_owner": spec.authority_owner,
            "carry_forward_lane": spec.carry_forward_lane,
            "reason": spec.reason,
            "source_comparison_state": gap_row.map(|row| row["comparison_state"].clone()).unwrap_or(Value::Null),
            "source_tracecalc_scenario_id": gap_row.map(|row| row["tracecalc_scenario_id"].clone()).unwrap_or(Value::Null),
            "source_treecalc_case_id": gap_row.map(|row| row["treecalc_case_id"].clone()).unwrap_or(Value::Null),
            "w035_matrix_evidence": matrix_evidence,
            "validation_state": if failures.is_empty() { "disposition_validated" } else { "disposition_failed" },
            "failures": failures,
        }),
        disposition_kind: spec.disposition_kind,
        valid: failures.is_empty(),
    }
}

fn evaluate_w036_closure_row(
    spec: &W036ClosureSpec,
    source_rows: &BTreeMap<String, Value>,
    matrix_rows: &BTreeMap<String, Value>,
) -> EvaluatedW036ClosureRow {
    let mut failures = Vec::new();
    let source_row = source_rows.get(spec.source_w034_gap_row_id);
    if let Some(row) = source_row {
        if string_at(row, "validation_state") != "disposition_validated" {
            failures.push("source_w035_disposition_not_validated".to_string());
        }
        if string_at(row, "disposition_kind") != spec.source_w035_disposition_kind {
            failures.push("source_w035_disposition_kind_mismatch".to_string());
        }
        if !array_at(row, "failures").is_empty() {
            failures.push("source_w035_disposition_has_failures".to_string());
        }
    } else {
        failures.push("source_w035_gap_disposition_missing".to_string());
    }

    let mut matrix_evidence = Vec::new();
    for matrix_row_id in spec.w036_matrix_row_ids {
        if let Some(matrix_row) = matrix_rows.get(*matrix_row_id) {
            let evidence_state = string_at(matrix_row, "evidence_state");
            if !matches!(
                evidence_state.as_str(),
                "covered_passed" | "excluded_by_authority" | "classified_uncovered_deferred"
            ) {
                failures.push(format!("w036_matrix_row_invalid_state:{matrix_row_id}"));
            }
            matrix_evidence.push(json!({
                "row_id": matrix_row_id,
                "obligation_id": matrix_row["obligation_id"],
                "coverage_class": matrix_row["coverage_class"],
                "evidence_state": evidence_state,
                "classification": matrix_row["classification"],
                "scenario_id": matrix_row["scenario_id"],
                "owner": matrix_row["owner"],
            }));
        } else {
            failures.push(format!("w036_matrix_row_missing:{matrix_row_id}"));
        }
    }

    if spec.conformance_match_state == "promoted_match" && matrix_evidence.is_empty() {
        failures.push("promoted_match_without_replay_or_diff_evidence".to_string());
    }
    if spec.implementation_evidence_state == "no_implementation_evidence"
        && spec.blocker_bead.is_none()
    {
        failures.push("row_has_neither_implementation_evidence_nor_blocker_bead".to_string());
    }

    let blocker_bead = spec
        .blocker_bead
        .map_or(Value::Null, |blocker| json!(blocker));
    let source_w035_disposition = source_row
        .map(|row| {
            json!({
                "row_id": row["row_id"],
                "disposition_kind": row["disposition_kind"],
                "disposition": row["disposition"],
                "authority_owner": row["authority_owner"],
                "carry_forward_lane": row["carry_forward_lane"],
                "source_tracecalc_scenario_id": row["source_tracecalc_scenario_id"],
                "source_treecalc_case_id": row["source_treecalc_case_id"],
            })
        })
        .unwrap_or(Value::Null);

    EvaluatedW036ClosureRow {
        row: json!({
            "row_id": spec.row_id,
            "source_w034_gap_row_id": spec.source_w034_gap_row_id,
            "w036_obligation_id": spec.w036_obligation_id,
            "source_w035_disposition": source_w035_disposition,
            "w036_disposition_kind": spec.w036_disposition_kind,
            "w036_disposition": spec.w036_disposition,
            "conformance_match_state": spec.conformance_match_state,
            "first_fix_state": spec.first_fix_state,
            "implementation_evidence_state": spec.implementation_evidence_state,
            "implementation_evidence_sources": spec.implementation_evidence_sources,
            "blocker_bead": blocker_bead,
            "authority_owner": spec.authority_owner,
            "promotion_consequence": spec.promotion_consequence,
            "reason": spec.reason,
            "w036_matrix_evidence": matrix_evidence,
            "validation_state": if failures.is_empty() { "w036_disposition_validated" } else { "w036_disposition_failed" },
            "failures": failures,
        }),
        first_fix: spec.first_fix_state != "no_first_fix",
        blocker_routed: spec.blocker_bead.is_some(),
        match_promoted: spec.conformance_match_state == "promoted_match",
        valid: failures.is_empty(),
    }
}

fn evaluate_w037_decision_row(
    repo_root: &Path,
    spec: &W037DecisionSpec,
    source_rows: &BTreeMap<String, Value>,
    matrix_rows: &BTreeMap<String, Value>,
    treecalc_summary: &Value,
) -> Result<EvaluatedW037DecisionRow, ImplementationConformanceError> {
    let mut failures = Vec::new();
    let source_row = source_rows.get(spec.source_w036_action_row_id);
    if let Some(row) = source_row {
        if string_at(row, "validation_state") != "w036_disposition_validated" {
            failures.push("source_w036_action_not_validated".to_string());
        }
        if string_at(row, "w036_disposition_kind") != spec.source_w036_disposition_kind {
            failures.push("source_w036_disposition_kind_mismatch".to_string());
        }
        if !array_at(row, "failures").is_empty() {
            failures.push("source_w036_action_has_failures".to_string());
        }
    } else {
        failures.push("source_w036_action_missing".to_string());
    }

    let mut matrix_evidence = Vec::new();
    for matrix_row_id in spec.w037_matrix_row_ids {
        if let Some(matrix_row) = matrix_rows.get(*matrix_row_id) {
            let evidence_state = string_at(matrix_row, "evidence_state");
            if !matches!(
                evidence_state.as_str(),
                "covered_passed" | "excluded_by_authority" | "classified_uncovered_deferred"
            ) {
                failures.push(format!("w037_matrix_row_invalid_state:{matrix_row_id}"));
            }
            matrix_evidence.push(json!({
                "row_id": matrix_row_id,
                "obligation_id": matrix_row["obligation_id"],
                "coverage_class": matrix_row["coverage_class"],
                "evidence_state": evidence_state,
                "classification": matrix_row["classification"],
                "scenario_id": matrix_row["scenario_id"],
                "owner": matrix_row["owner"],
            }));
        } else {
            failures.push(format!("w037_matrix_row_missing:{matrix_row_id}"));
        }
    }

    let treecalc_evidence = if let Some(case_id) = spec.required_treecalc_case_id {
        let result_path = relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "treecalc-local",
            W037_TREECALC_RUN_ID,
            "cases",
            case_id,
            "result.json",
        ]);
        let result = read_json(repo_root, &result_path)?;
        let dependency_shape_update_count = result
            .pointer("/candidate_result/dependency_shape_updates")
            .and_then(Value::as_array)
            .map_or(0, Vec::len);
        let dynamic_runtime_effect_count = result
            .pointer("/publication_bundle/published_runtime_effects")
            .and_then(Value::as_array)
            .map_or(0, |effects| {
                effects
                    .iter()
                    .filter(|effect| string_at(effect, "family") == "DynamicDependency")
                    .count()
            });
        let carriage_shape_update_count = result
            .pointer("/publication_bundle/carriage_classification/dependency_shape_update_count")
            .and_then(Value::as_u64)
            .unwrap_or_default();

        if string_at(&result, "result_state") != "published" {
            failures.push("treecalc_dynamic_resolved_case_not_published".to_string());
        }
        if dependency_shape_update_count == 0 {
            failures.push("treecalc_dynamic_resolved_shape_update_missing".to_string());
        }
        if dynamic_runtime_effect_count == 0 {
            failures.push("treecalc_dynamic_resolved_runtime_effect_missing".to_string());
        }
        if carriage_shape_update_count == 0 {
            failures.push("treecalc_dynamic_resolved_carriage_count_missing".to_string());
        }
        if number_at(treecalc_summary, "expectation_mismatch_count") != 0 {
            failures.push("w037_treecalc_expectation_mismatch_count_nonzero".to_string());
        }

        json!({
            "treecalc_run_id": W037_TREECALC_RUN_ID,
            "case_id": case_id,
            "result_path": result_path,
            "result_state": string_at(&result, "result_state"),
            "dependency_shape_update_count": dependency_shape_update_count,
            "dynamic_runtime_effect_count": dynamic_runtime_effect_count,
            "carriage_dependency_shape_update_count": carriage_shape_update_count,
        })
    } else {
        Value::Null
    };

    if spec.conformance_match_state == "promoted_match"
        && (matrix_evidence.is_empty() || treecalc_evidence.is_null())
    {
        failures.push("promoted_match_without_tracecalc_and_treecalc_evidence".to_string());
    }
    if spec.w037_decision_kind == "residual_blocker" && spec.residual_blocker_bead.is_none() {
        failures.push("residual_blocker_without_owner_bead".to_string());
    }

    let residual_blocker_bead = spec
        .residual_blocker_bead
        .map_or(Value::Null, |blocker| json!(blocker));
    let source_w036_action = source_row
        .map(|row| {
            json!({
                "row_id": row["row_id"],
                "w036_disposition_kind": row["w036_disposition_kind"],
                "w036_disposition": row["w036_disposition"],
                "conformance_match_state": row["conformance_match_state"],
                "first_fix_state": row["first_fix_state"],
                "implementation_evidence_state": row["implementation_evidence_state"],
                "blocker_bead": row["blocker_bead"],
            })
        })
        .unwrap_or(Value::Null);

    Ok(EvaluatedW037DecisionRow {
        row: json!({
            "row_id": spec.row_id,
            "source_w036_action_row_id": spec.source_w036_action_row_id,
            "w037_obligation_id": spec.w037_obligation_id,
            "source_w036_action": source_w036_action,
            "w037_decision_kind": spec.w037_decision_kind,
            "w037_decision": spec.w037_decision,
            "conformance_match_state": spec.conformance_match_state,
            "implementation_evidence_state": spec.implementation_evidence_state,
            "implementation_evidence_sources": spec.implementation_evidence_sources,
            "residual_blocker_bead": residual_blocker_bead,
            "authority_owner": spec.authority_owner,
            "promotion_consequence": spec.promotion_consequence,
            "reason": spec.reason,
            "w037_matrix_evidence": matrix_evidence,
            "treecalc_evidence": treecalc_evidence,
            "validation_state": if failures.is_empty() { "w037_decision_validated" } else { "w037_decision_failed" },
            "failures": failures,
        }),
        fixed_or_promoted: spec.w037_decision_kind != "residual_blocker",
        residual_blocker: spec.residual_blocker_bead.is_some(),
        match_promoted: spec.conformance_match_state == "promoted_match",
        valid: failures.is_empty(),
    })
}

fn validation_failures(
    w034_summary: &Value,
    treecalc_summary: &Value,
    matrix_summary: &Value,
    failed_row_count: usize,
) -> Vec<String> {
    let mut failures = Vec::new();
    if number_at(w034_summary, "declared_gap_count") != 6 {
        failures.push("w034_declared_gap_count_changed".to_string());
    }
    if number_at(w034_summary, "unexpected_mismatch_count") != 0 {
        failures.push("w034_unexpected_mismatch_count_nonzero".to_string());
    }
    if number_at(w034_summary, "missing_artifact_count") != 0 {
        failures.push("w034_missing_artifact_count_nonzero".to_string());
    }
    if number_at(treecalc_summary, "expectation_mismatch_count") != 0 {
        failures.push("treecalc_expectation_mismatch_count_nonzero".to_string());
    }
    if number_at(matrix_summary, "missing_or_failed_row_count") != 0 {
        failures.push("w035_matrix_missing_or_failed_row_count_nonzero".to_string());
    }
    if failed_row_count != 0 {
        failures.push("gap_disposition_row_failures_present".to_string());
    }
    failures
}

fn w036_validation_failures(
    w035_gap_register: &Value,
    w036_matrix_summary: &Value,
    failed_row_count: usize,
    match_promoted_count: usize,
) -> Vec<String> {
    let mut failures = Vec::new();
    if number_at(w035_gap_register, "row_count") != 6 {
        failures.push("w035_gap_disposition_row_count_changed".to_string());
    }
    if number_at(w035_gap_register, "failed_row_count") != 0 {
        failures.push("w035_gap_disposition_failed_row_count_nonzero".to_string());
    }
    if number_at(w036_matrix_summary, "matrix_row_count") != 32 {
        failures.push("w036_matrix_row_count_changed".to_string());
    }
    if number_at(w036_matrix_summary, "missing_or_failed_row_count") != 0 {
        failures.push("w036_matrix_missing_or_failed_row_count_nonzero".to_string());
    }
    if failed_row_count != 0 {
        failures.push("w036_closure_action_row_failures_present".to_string());
    }
    if match_promoted_count != 0 {
        failures.push("w036_declared_gap_match_promotion_present".to_string());
    }
    failures
}

fn w037_validation_failures(
    w036_action_register: &Value,
    w037_matrix_summary: &Value,
    w037_treecalc_summary: &Value,
    failed_row_count: usize,
    match_promoted_count: usize,
    residual_blocker_count: usize,
) -> Vec<String> {
    let mut failures = Vec::new();
    if number_at(w036_action_register, "action_row_count") != 6 {
        failures.push("w036_action_row_count_changed".to_string());
    }
    if number_at(w036_action_register, "failed_row_count") != 0 {
        failures.push("w036_action_failed_row_count_nonzero".to_string());
    }
    if number_at(w037_matrix_summary, "matrix_row_count") != 32 {
        failures.push("w037_matrix_row_count_changed".to_string());
    }
    if number_at(w037_matrix_summary, "missing_or_failed_row_count") != 0 {
        failures.push("w037_matrix_missing_or_failed_row_count_nonzero".to_string());
    }
    if number_at(w037_treecalc_summary, "expectation_mismatch_count") != 0 {
        failures.push("w037_treecalc_expectation_mismatch_count_nonzero".to_string());
    }
    if number_at(w037_treecalc_summary, "case_count") < 24 {
        failures.push("w037_treecalc_case_count_too_low".to_string());
    }
    if failed_row_count != 0 {
        failures.push("w037_decision_row_failures_present".to_string());
    }
    if match_promoted_count != 1 {
        failures.push("w037_expected_one_dynamic_match_promotion".to_string());
    }
    if residual_blocker_count != 5 {
        failures.push("w037_expected_five_residual_blockers".to_string());
    }
    failures
}

fn rows_by_id(document: &Value, key: &str) -> BTreeMap<String, Value> {
    array_at(document, "rows")
        .iter()
        .filter_map(|row| Some((row.get(key)?.as_str()?.to_string(), row.clone())))
        .collect()
}

fn read_json(
    repo_root: &Path,
    relative_path: &str,
) -> Result<Value, ImplementationConformanceError> {
    let path = repo_root.join(relative_path);
    let content = fs::read_to_string(&path).map_err(|source| {
        ImplementationConformanceError::ReadArtifact {
            path: path.display().to_string(),
            source,
        }
    })?;
    serde_json::from_str(&content).map_err(|source| ImplementationConformanceError::ParseJson {
        path: path.display().to_string(),
        source,
    })
}

fn write_json(path: &Path, value: &Value) -> Result<(), ImplementationConformanceError> {
    let content = serde_json::to_string_pretty(value).expect("JSON serialization should succeed");
    fs::write(path, format!("{content}\n")).map_err(|source| {
        ImplementationConformanceError::WriteFile {
            path: path.display().to_string(),
            source,
        }
    })
}

fn create_directory(path: &Path) -> Result<(), ImplementationConformanceError> {
    fs::create_dir_all(path).map_err(|source| ImplementationConformanceError::CreateDirectory {
        path: path.display().to_string(),
        source,
    })
}

fn array_at<'a>(value: &'a Value, key: &str) -> &'a [Value] {
    value
        .get(key)
        .and_then(Value::as_array)
        .map_or(&[], Vec::as_slice)
}

fn string_at(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn string_pointer(value: &Value, pointer: &str) -> String {
    value
        .pointer(pointer)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn number_at(value: &Value, key: &str) -> u64 {
    value.get(key).and_then(Value::as_u64).unwrap_or_default()
}

fn relative_artifact_path<'a>(segments: impl IntoIterator<Item = &'a str>) -> String {
    segments
        .into_iter()
        .filter(|segment| !segment.trim().is_empty())
        .map(|segment| segment.replace('\\', "/").trim_matches('/').to_string())
        .collect::<Vec<_>>()
        .join("/")
}

const GAP_DISPOSITION_SPECS: &[GapDispositionSpec] = &[
    GapDispositionSpec {
        row_id: "ic_gap_dynamic_dependency_001",
        source_gap_classification: "treecalc_local_dynamic_dependency_projection_gap",
        disposition_kind: "implementation_work_deferred",
        disposition: "open_dynamic_dependency_bind_projection_work",
        authority_owner: "calc-tkq.3",
        carry_forward_lane: "calc-tkq.8_next_tranche_packetization",
        reason: "W035 TraceCalc covers dynamic dependency switch publication, but TreeCalc-local still represents this surface as a rejected residual carrier instead of a published dynamic-bind update.",
        w035_matrix_row_ids: &["w035_dependency_dynamic_switch_publish"],
    },
    GapDispositionSpec {
        row_id: "ic_gap_lambda_host_effect_001",
        source_gap_classification: "treecalc_local_host_sensitive_lambda_projection_gap",
        disposition_kind: "spec_evolution_deferral",
        disposition: "defer_host_sensitive_lambda_effect_to_callable_seam_map",
        authority_owner: "calc-tkq.4",
        carry_forward_lane: "lean_assumption_discharge_and_seam_proof_map",
        reason: "W035 includes the OxCalc/OxFml callable-carrier fragment, but host-sensitive lambda execution effects cross into OxFunc/OxFml-owned semantics beyond this implementation-conformance target.",
        w035_matrix_row_ids: &["w035_callable_full_oxfunc_semantics"],
    },
    GapDispositionSpec {
        row_id: "ic_gap_w034_dynamic_dependency_negative_001",
        source_gap_classification: "treecalc_local_dynamic_dependency_shape_update_projection_gap",
        disposition_kind: "implementation_work_deferred",
        disposition: "open_dynamic_dependency_negative_projection_work",
        authority_owner: "calc-tkq.3",
        carry_forward_lane: "calc-tkq.8_next_tranche_packetization",
        reason: "TraceCalc covers unresolved dynamic dependency rejection and W035 positive dynamic update rows, but TreeCalc-local does not yet project the same dynamic shape-update evidence.",
        w035_matrix_row_ids: &[
            "w035_dependency_dynamic_negative",
            "w035_dependency_dynamic_release_publish",
        ],
    },
    GapDispositionSpec {
        row_id: "ic_gap_w034_snapshot_fence_projection_001",
        source_gap_classification: "treecalc_local_snapshot_fence_projection_gap",
        disposition_kind: "implementation_work_deferred",
        disposition: "open_coordinator_snapshot_fence_projection_work",
        authority_owner: "calc-tkq.5",
        carry_forward_lane: "tla_non_routine_and_future_optimized_coordinator_conformance",
        reason: "W035 TraceCalc covers snapshot-fence rejection, but TreeCalc-local is a single-run local fixture surface and still lacks a stale candidate admission fence counterpart.",
        w035_matrix_row_ids: &["w035_stale_snapshot_fence_reject"],
    },
    GapDispositionSpec {
        row_id: "ic_gap_w034_capability_view_fence_projection_001",
        source_gap_classification: "treecalc_local_capability_view_fence_projection_gap",
        disposition_kind: "implementation_work_deferred",
        disposition: "open_coordinator_capability_view_fence_projection_work",
        authority_owner: "calc-tkq.5",
        carry_forward_lane: "tla_non_routine_and_future_optimized_coordinator_conformance",
        reason: "TreeCalc-local has capability-sensitive reject evidence, but W035 TraceCalc's compatibility-fenced capability-view mismatch remains a coordinator fence surface without a local TreeCalc fixture counterpart.",
        w035_matrix_row_ids: &["w035_stale_capability_view_fence_reject"],
    },
    GapDispositionSpec {
        row_id: "ic_gap_w034_higher_order_callable_metadata_001",
        source_gap_classification: "treecalc_local_higher_order_callable_identity_projection_gap",
        disposition_kind: "implementation_work_deferred",
        disposition: "open_callable_metadata_projection_work",
        authority_owner: "calc-tkq.4",
        carry_forward_lane: "lean_assumption_discharge_and_callable_seam_proof_map",
        reason: "TreeCalc-local matches the ordinary value for the W034 higher-order row, but it still does not project callable identity metadata as a conformance surface.",
        w035_matrix_row_ids: &["w035_callable_higher_order_publish"],
    },
];

const W036_CLOSURE_SPECS: &[W036ClosureSpec] = &[
    W036ClosureSpec {
        row_id: "w036_action_dynamic_dependency_bind_projection",
        source_w034_gap_row_id: "ic_gap_dynamic_dependency_001",
        source_w035_disposition_kind: "implementation_work_deferred",
        w036_obligation_id: "W036-OBL-003",
        w036_disposition_kind: "harness_first_fix",
        w036_disposition: "bind_dynamic_projection_to_positive_replay_and_current_treecalc_runtime_effect_boundary",
        conformance_match_state: "not_promoted",
        first_fix_state: "w036_harness_evidence_bound",
        implementation_evidence_state: "tracecalc_replay_plus_treecalc_projection_evidence",
        blocker_bead: None,
        authority_owner: "calc-rqq.3",
        promotion_consequence: "optimized/core-engine conformance and pack/C5 remain blocked until the optimized lane publishes a dynamic-bind update differential rather than only a dynamic runtime-effect reject boundary.",
        reason: "W036 has deterministic TraceCalc positive dynamic-switch replay and current TreeCalc projection evidence for the rejected runtime-effect boundary. This closes the prose-only handoff into a harness action, but it is not a conformance match.",
        implementation_evidence_sources: &[
            "docs/test-runs/core-engine/independent-conformance/w034-independent-conformance-001/comparisons/core_engine_projection_differential.json",
            "docs/test-runs/core-engine/tracecalc-reference-machine/w036-tracecalc-coverage-closure-001/oracle-matrix/coverage_matrix.json",
        ],
        w036_matrix_row_ids: &[
            "w035_dependency_dynamic_switch_publish",
            "w036_dynamic_dependency_switch_seed",
        ],
    },
    W036ClosureSpec {
        row_id: "w036_action_lambda_host_effect_boundary",
        source_w034_gap_row_id: "ic_gap_lambda_host_effect_001",
        source_w035_disposition_kind: "spec_evolution_deferral",
        w036_obligation_id: "W036-OBL-004",
        w036_disposition_kind: "formal_deferral_with_blocker",
        w036_disposition: "route_host_sensitive_lambda_effect_to_let_lambda_boundary_inventory",
        conformance_match_state: "not_promoted",
        first_fix_state: "no_first_fix",
        implementation_evidence_state: "tracecalc_carrier_replay_plus_oxfunc_kernel_exclusion",
        blocker_bead: Some("calc-rqq.4"),
        authority_owner: "calc-rqq.3; calc-rqq.4",
        promotion_consequence: "LET/LAMBDA carrier conformance remains bounded to OxCalc/OxFml carrier evidence until calc-rqq.4 records the Lean/OxFunc-opaque boundary inventory.",
        reason: "W036 keeps callable runtime-effect visibility covered in TraceCalc while excluding the general OxFunc LAMBDA semantic kernel from OxCalc-local TraceCalc oracle scope.",
        implementation_evidence_sources: &[
            "docs/test-runs/core-engine/tracecalc-reference-machine/w036-tracecalc-coverage-closure-001/oracle-matrix/coverage_matrix.json",
            "docs/test-runs/core-engine/implementation-conformance/w035-implementation-conformance-hardening-001/gap_disposition_register.json",
        ],
        w036_matrix_row_ids: &[
            "w036_callable_runtime_effect_visibility",
            "w035_callable_full_oxfunc_semantics",
        ],
    },
    W036ClosureSpec {
        row_id: "w036_action_dynamic_dependency_negative_shape_update",
        source_w034_gap_row_id: "ic_gap_w034_dynamic_dependency_negative_001",
        source_w035_disposition_kind: "implementation_work_deferred",
        w036_obligation_id: "W036-OBL-005",
        w036_disposition_kind: "harness_first_fix",
        w036_disposition: "bind_dynamic_negative_and_release_rows_to_shape_update_harness_requirements",
        conformance_match_state: "not_promoted",
        first_fix_state: "w036_harness_evidence_bound",
        implementation_evidence_state: "tracecalc_negative_release_replay_plus_treecalc_boundary_evidence",
        blocker_bead: None,
        authority_owner: "calc-rqq.3",
        promotion_consequence: "optimized/core-engine conformance remains blocked until dynamic negative and release/reclassification surfaces have optimized-lane differential evidence.",
        reason: "W036 binds the negative and release/reclassification replay rows to one action, keeping the current TreeCalc residual reject boundary visible rather than counting it as a match.",
        implementation_evidence_sources: &[
            "docs/test-runs/core-engine/independent-conformance/w034-independent-conformance-001/comparisons/treecalc_tracecalc_differential.json",
            "docs/test-runs/core-engine/tracecalc-reference-machine/w036-tracecalc-coverage-closure-001/oracle-matrix/coverage_matrix.json",
        ],
        w036_matrix_row_ids: &[
            "w035_dependency_dynamic_negative",
            "w035_dependency_dynamic_release_publish",
        ],
    },
    W036ClosureSpec {
        row_id: "w036_action_snapshot_fence_projection",
        source_w034_gap_row_id: "ic_gap_w034_snapshot_fence_projection_001",
        source_w035_disposition_kind: "implementation_work_deferred",
        w036_obligation_id: "W036-OBL-006",
        w036_disposition_kind: "coordinator_harness_blocker",
        w036_disposition: "route_snapshot_fence_projection_to_tla_and_coordinator_replay_lane",
        conformance_match_state: "not_promoted",
        first_fix_state: "no_first_fix",
        implementation_evidence_state: "tracecalc_replay_evidence_with_missing_treecalc_counterpart",
        blocker_bead: Some("calc-rqq.5"),
        authority_owner: "calc-rqq.3; calc-rqq.5",
        promotion_consequence: "snapshot-fence conformance remains blocked until W036 TLA/coordinator work supplies a local counterpart or proves the boundary external to optimized conformance.",
        reason: "W036 TraceCalc covers snapshot-fence rejection, but the local TreeCalc fixture lane still has no stale-candidate admission fence counterpart.",
        implementation_evidence_sources: &[
            "docs/test-runs/core-engine/tracecalc-reference-machine/w036-tracecalc-coverage-closure-001/oracle-matrix/coverage_matrix.json",
        ],
        w036_matrix_row_ids: &["w035_stale_snapshot_fence_reject"],
    },
    W036ClosureSpec {
        row_id: "w036_action_capability_view_fence_projection",
        source_w034_gap_row_id: "ic_gap_w034_capability_view_fence_projection_001",
        source_w035_disposition_kind: "implementation_work_deferred",
        w036_obligation_id: "W036-OBL-007",
        w036_disposition_kind: "coordinator_harness_blocker",
        w036_disposition: "route_capability_view_fence_projection_to_tla_and_coordinator_replay_lane",
        conformance_match_state: "not_promoted",
        first_fix_state: "no_first_fix",
        implementation_evidence_state: "tracecalc_replay_evidence_with_missing_treecalc_counterpart",
        blocker_bead: Some("calc-rqq.5"),
        authority_owner: "calc-rqq.3; calc-rqq.5",
        promotion_consequence: "capability-view fence conformance remains blocked until W036 TLA/coordinator work supplies a local counterpart or proves the boundary external to optimized conformance.",
        reason: "W036 TraceCalc covers compatibility-fenced capability-view rejection, while TreeCalc-local only has broader capability-sensitive reject evidence.",
        implementation_evidence_sources: &[
            "docs/test-runs/core-engine/tracecalc-reference-machine/w036-tracecalc-coverage-closure-001/oracle-matrix/coverage_matrix.json",
        ],
        w036_matrix_row_ids: &["w035_stale_capability_view_fence_reject"],
    },
    W036ClosureSpec {
        row_id: "w036_action_callable_metadata_projection",
        source_w034_gap_row_id: "ic_gap_w034_higher_order_callable_metadata_001",
        source_w035_disposition_kind: "implementation_work_deferred",
        w036_obligation_id: "W036-OBL-008",
        w036_disposition_kind: "formal_deferral_with_blocker",
        w036_disposition: "route_callable_metadata_projection_to_lean_callable_boundary_inventory",
        conformance_match_state: "not_promoted",
        first_fix_state: "no_first_fix",
        implementation_evidence_state: "tracecalc_callable_carrier_replay_with_metadata_gap",
        blocker_bead: Some("calc-rqq.4"),
        authority_owner: "calc-rqq.3; calc-rqq.4",
        promotion_consequence: "callable metadata projection remains blocked until callable carrier sufficiency is proven or a concrete metadata projection fixture is added.",
        reason: "W036 TraceCalc covers higher-order callable carrier publication, but TreeCalc-local still compares ordinary value only and does not project callable identity metadata.",
        implementation_evidence_sources: &[
            "docs/test-runs/core-engine/independent-conformance/w034-independent-conformance-001/comparisons/treecalc_tracecalc_differential.json",
            "docs/test-runs/core-engine/tracecalc-reference-machine/w036-tracecalc-coverage-closure-001/oracle-matrix/coverage_matrix.json",
        ],
        w036_matrix_row_ids: &["w035_callable_higher_order_publish"],
    },
];

const W037_DECISION_SPECS: &[W037DecisionSpec] = &[
    W037DecisionSpec {
        row_id: "w037_decision_dynamic_dependency_bind_projection_fixed",
        source_w036_action_row_id: "w036_action_dynamic_dependency_bind_projection",
        source_w036_disposition_kind: "harness_first_fix",
        w037_obligation_id: "W037-OBL-003",
        w037_decision_kind: "implementation_fixed_with_treecalc_differential",
        w037_decision: "promote_positive_dynamic_bind_projection_from_w036_first_fix_to_direct_treecalc_differential",
        conformance_match_state: "promoted_match",
        implementation_evidence_state: "tracecalc_replay_plus_treecalc_dynamic_resolved_publication_evidence",
        residual_blocker_bead: None,
        authority_owner: "calc-ubd.3",
        promotion_consequence: "the positive dynamic-bind projection gap no longer blocks optimized/core-engine conformance, but broader dynamic negative, release, and reclassification surfaces remain residual blockers.",
        reason: "W037 adds a resolved dynamic-potential TreeCalc carrier that publishes through the OxFml-backed value path, emits a DynamicDependency runtime effect, and records a dependency_shape_updates sidecar on the accepted candidate.",
        implementation_evidence_sources: &[
            "docs/test-runs/core-engine/treecalc-local/w037-optimized-core-conformance-treecalc-001/cases/tc_local_dynamic_resolved_publish_001/result.json",
            "docs/test-runs/core-engine/tracecalc-reference-machine/w037-tracecalc-observable-closure-001/oracle-matrix/coverage_matrix.json",
        ],
        w037_matrix_row_ids: &[
            "w035_dependency_dynamic_switch_publish",
            "w036_dynamic_dependency_switch_seed",
        ],
        required_treecalc_case_id: Some("tc_local_dynamic_resolved_publish_001"),
    },
    W037DecisionSpec {
        row_id: "w037_decision_dynamic_negative_release_residual_blocker",
        source_w036_action_row_id: "w036_action_dynamic_dependency_negative_shape_update",
        source_w036_disposition_kind: "harness_first_fix",
        w037_obligation_id: "W037-OBL-003",
        w037_decision_kind: "residual_blocker",
        w037_decision: "carry_dynamic_negative_release_and_reclassification_surfaces_as_residual_blockers",
        conformance_match_state: "not_promoted",
        implementation_evidence_state: "tracecalc_negative_release_replay_without_treecalc_release_reclassification_differential",
        residual_blocker_bead: Some("calc-ubd.9"),
        authority_owner: "calc-ubd.3; calc-ubd.9",
        promotion_consequence: "full optimized/core-engine conformance remains blocked until dynamic negative, release, and reclassification surfaces receive direct optimized-lane differential evidence or successor scope.",
        reason: "The W037 resolved dynamic carrier proves the positive bind projection only. It does not exercise unresolved dynamic negative handling, release, or dependency reclassification as an optimized TreeCalc differential.",
        implementation_evidence_sources: &[
            "docs/test-runs/core-engine/tracecalc-reference-machine/w037-tracecalc-observable-closure-001/oracle-matrix/coverage_matrix.json",
            "docs/test-runs/core-engine/implementation-conformance/w036-implementation-conformance-closure-001/w036_closure_action_register.json",
        ],
        w037_matrix_row_ids: &[
            "w035_dependency_dynamic_negative",
            "w035_dependency_dynamic_release_publish",
        ],
        required_treecalc_case_id: None,
    },
    W037DecisionSpec {
        row_id: "w037_decision_lambda_host_effect_residual_blocker",
        source_w036_action_row_id: "w036_action_lambda_host_effect_boundary",
        source_w036_disposition_kind: "formal_deferral_with_blocker",
        w037_obligation_id: "W037-OBL-006",
        w037_decision_kind: "residual_blocker",
        w037_decision: "carry_lambda_host_effect_to_direct_oxfml_and_let_lambda_seam_beads",
        conformance_match_state: "not_promoted",
        implementation_evidence_state: "tracecalc_carrier_replay_with_direct_oxfml_evaluator_absent",
        residual_blocker_bead: Some("calc-ubd.4"),
        authority_owner: "calc-ubd.3; calc-ubd.4",
        promotion_consequence: "LET/LAMBDA carrier conformance remains blocked until direct OxFml evaluator evidence and callable carrier boundary inventory are bound.",
        reason: "The host-sensitive callable effect crosses the narrow OxCalc/OxFml/OxFunc carrier fragment and cannot be promoted from TreeCalc-local value evidence alone.",
        implementation_evidence_sources: &[
            "docs/test-runs/core-engine/tracecalc-reference-machine/w037-tracecalc-observable-closure-001/oracle-matrix/coverage_matrix.json",
            "docs/test-runs/core-engine/implementation-conformance/w036-implementation-conformance-closure-001/w036_closure_action_register.json",
        ],
        w037_matrix_row_ids: &[
            "w036_callable_runtime_effect_visibility",
            "w035_callable_full_oxfunc_semantics",
        ],
        required_treecalc_case_id: None,
    },
    W037DecisionSpec {
        row_id: "w037_decision_snapshot_fence_projection_residual_blocker",
        source_w036_action_row_id: "w036_action_snapshot_fence_projection",
        source_w036_disposition_kind: "coordinator_harness_blocker",
        w037_obligation_id: "W037-OBL-009",
        w037_decision_kind: "residual_blocker",
        w037_decision: "carry_snapshot_fence_projection_to_stage2_and_coordinator_replay",
        conformance_match_state: "not_promoted",
        implementation_evidence_state: "tracecalc_replay_without_treecalc_stale_candidate_counterpart",
        residual_blocker_bead: Some("calc-ubd.6"),
        authority_owner: "calc-ubd.3; calc-ubd.6",
        promotion_consequence: "snapshot-fence conformance remains blocked until deterministic coordinator replay or Stage 2 partition evidence supplies an optimized counterpart.",
        reason: "TreeCalc-local remains a single-run local fixture lane and does not exercise stale accepted-candidate admission against an older structural snapshot.",
        implementation_evidence_sources: &[
            "docs/test-runs/core-engine/tracecalc-reference-machine/w037-tracecalc-observable-closure-001/oracle-matrix/coverage_matrix.json",
        ],
        w037_matrix_row_ids: &["w035_stale_snapshot_fence_reject"],
        required_treecalc_case_id: None,
    },
    W037DecisionSpec {
        row_id: "w037_decision_capability_view_fence_projection_residual_blocker",
        source_w036_action_row_id: "w036_action_capability_view_fence_projection",
        source_w036_disposition_kind: "coordinator_harness_blocker",
        w037_obligation_id: "W037-OBL-009",
        w037_decision_kind: "residual_blocker",
        w037_decision: "carry_capability_view_fence_projection_to_stage2_and_coordinator_replay",
        conformance_match_state: "not_promoted",
        implementation_evidence_state: "tracecalc_replay_without_treecalc_capability_view_fence_counterpart",
        residual_blocker_bead: Some("calc-ubd.6"),
        authority_owner: "calc-ubd.3; calc-ubd.6",
        promotion_consequence: "capability-view fence conformance remains blocked until deterministic coordinator replay or Stage 2 partition evidence supplies an optimized counterpart.",
        reason: "TreeCalc-local has capability-sensitive reject evidence, but not compatibility-fenced capability-view mismatch evidence.",
        implementation_evidence_sources: &[
            "docs/test-runs/core-engine/tracecalc-reference-machine/w037-tracecalc-observable-closure-001/oracle-matrix/coverage_matrix.json",
        ],
        w037_matrix_row_ids: &["w035_stale_capability_view_fence_reject"],
        required_treecalc_case_id: None,
    },
    W037DecisionSpec {
        row_id: "w037_decision_callable_metadata_projection_residual_blocker",
        source_w036_action_row_id: "w036_action_callable_metadata_projection",
        source_w036_disposition_kind: "formal_deferral_with_blocker",
        w037_obligation_id: "W037-OBL-006",
        w037_decision_kind: "residual_blocker",
        w037_decision: "carry_callable_metadata_projection_to_let_lambda_and_proof_inventory",
        conformance_match_state: "not_promoted",
        implementation_evidence_state: "tracecalc_callable_carrier_replay_with_treecalc_value_only_counterpart",
        residual_blocker_bead: Some("calc-ubd.5"),
        authority_owner: "calc-ubd.3; calc-ubd.5",
        promotion_consequence: "callable metadata projection remains blocked until callable carrier sufficiency is proven or a concrete metadata projection fixture is added.",
        reason: "TreeCalc-local still compares the ordinary value for the higher-order row; callable identity metadata is not yet projected as an optimized conformance surface.",
        implementation_evidence_sources: &[
            "docs/test-runs/core-engine/tracecalc-reference-machine/w037-tracecalc-observable-closure-001/oracle-matrix/coverage_matrix.json",
            "docs/test-runs/core-engine/implementation-conformance/w036-implementation-conformance-closure-001/w036_closure_action_register.json",
        ],
        w037_matrix_row_ids: &["w035_callable_higher_order_publish"],
        required_treecalc_case_id: None,
    },
];

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn implementation_conformance_runner_classifies_w034_gaps() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let run_id = format!(
            "test-w035-implementation-conformance-{}",
            std::process::id()
        );
        let artifact_root = repo_root.join(format!(
            "docs/test-runs/core-engine/implementation-conformance/{run_id}"
        ));
        let cleanup = || {
            if artifact_root.exists() {
                let _ = fs::remove_dir_all(&artifact_root);
            }
        };

        cleanup();
        let summary = ImplementationConformanceRunner::new()
            .execute(&repo_root, &run_id)
            .unwrap();

        assert_eq!(summary.gap_disposition_row_count, 6);
        assert_eq!(summary.implementation_work_count, 5);
        assert_eq!(summary.spec_evolution_deferral_count, 1);
        assert_eq!(summary.validated_row_count, 6);
        assert_eq!(summary.failed_row_count, 0);

        let validation = read_json(
            &repo_root,
            &format!(
                "docs/test-runs/core-engine/implementation-conformance/{run_id}/validation.json"
            ),
        )
        .unwrap();
        assert_eq!(
            validation["status"],
            "implementation_conformance_hardening_valid"
        );

        cleanup();
    }

    #[test]
    fn implementation_conformance_runner_classifies_w036_closure_actions() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let run_id = format!(
            "test-w036-implementation-conformance-{}",
            std::process::id()
        );
        let artifact_root = repo_root.join(format!(
            "docs/test-runs/core-engine/implementation-conformance/{run_id}"
        ));
        let cleanup = || {
            if artifact_root.exists() {
                let _ = fs::remove_dir_all(&artifact_root);
            }
        };

        cleanup();
        let summary = ImplementationConformanceRunner::new()
            .execute(&repo_root, &run_id)
            .unwrap();

        assert_eq!(summary.gap_disposition_row_count, 6);
        assert_eq!(summary.w036_action_row_count, 6);
        assert_eq!(summary.w036_first_fix_row_count, 2);
        assert_eq!(summary.w036_blocker_routed_row_count, 4);
        assert_eq!(summary.w036_match_promoted_count, 0);
        assert_eq!(summary.validated_row_count, 6);
        assert_eq!(summary.failed_row_count, 0);

        let validation = read_json(
            &repo_root,
            &format!(
                "docs/test-runs/core-engine/implementation-conformance/{run_id}/validation.json"
            ),
        )
        .unwrap();
        assert_eq!(
            validation["status"],
            "implementation_conformance_w036_closure_plan_valid"
        );

        let guard = read_json(
            &repo_root,
            &format!(
                "docs/test-runs/core-engine/implementation-conformance/{run_id}/w036_match_promotion_guard.json"
            ),
        )
        .unwrap();
        assert_eq!(guard["promoted_match_count"], 0);

        cleanup();
    }

    #[test]
    fn implementation_conformance_runner_classifies_w037_decisions() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let run_id = format!(
            "test-w037-implementation-conformance-{}",
            std::process::id()
        );
        let artifact_root = repo_root.join(format!(
            "docs/test-runs/core-engine/implementation-conformance/{run_id}"
        ));
        let cleanup = || {
            if artifact_root.exists() {
                let _ = fs::remove_dir_all(&artifact_root);
            }
        };

        cleanup();
        let summary = ImplementationConformanceRunner::new()
            .execute(&repo_root, &run_id)
            .unwrap();

        assert_eq!(summary.gap_disposition_row_count, 6);
        assert_eq!(summary.w037_decision_row_count, 6);
        assert_eq!(summary.w037_fixed_or_promoted_count, 1);
        assert_eq!(summary.w037_residual_blocker_count, 5);
        assert_eq!(summary.w037_match_promoted_count, 1);
        assert_eq!(summary.validated_row_count, 6);
        assert_eq!(summary.failed_row_count, 0);

        let validation = read_json(
            &repo_root,
            &format!(
                "docs/test-runs/core-engine/implementation-conformance/{run_id}/validation.json"
            ),
        )
        .unwrap();
        assert_eq!(
            validation["status"],
            "implementation_conformance_w037_decisions_valid"
        );

        let guard = read_json(
            &repo_root,
            &format!(
                "docs/test-runs/core-engine/implementation-conformance/{run_id}/w037_match_promotion_guard.json"
            ),
        )
        .unwrap();
        assert_eq!(guard["promoted_match_count"], 1);
        assert_eq!(
            guard["guard_status"],
            "w037_declared_gap_promotion_guard_holds"
        );

        cleanup();
    }
}
