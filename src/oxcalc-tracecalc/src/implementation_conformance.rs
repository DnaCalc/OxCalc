#![forbid(unsafe_code)]

//! W035/W036/W037/W038/W039/W044/W045 implementation-conformance packet emission.

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
const IMPLEMENTATION_CONFORMANCE_W038_DISPOSITION_REGISTER_SCHEMA_V1: &str =
    "oxcalc.implementation_conformance.w038_disposition_register.v1";
const IMPLEMENTATION_CONFORMANCE_W038_BLOCKER_REGISTER_SCHEMA_V1: &str =
    "oxcalc.implementation_conformance.w038_blocker_register.v1";
const IMPLEMENTATION_CONFORMANCE_W038_MATCH_GUARD_SCHEMA_V1: &str =
    "oxcalc.implementation_conformance.w038_match_guard.v1";
const IMPLEMENTATION_CONFORMANCE_W039_DISPOSITION_REGISTER_SCHEMA_V1: &str =
    "oxcalc.implementation_conformance.w039_disposition_register.v1";
const IMPLEMENTATION_CONFORMANCE_W039_BLOCKER_REGISTER_SCHEMA_V1: &str =
    "oxcalc.implementation_conformance.w039_blocker_register.v1";
const IMPLEMENTATION_CONFORMANCE_W039_MATCH_GUARD_SCHEMA_V1: &str =
    "oxcalc.implementation_conformance.w039_match_guard.v1";
const IMPLEMENTATION_CONFORMANCE_W044_DISPOSITION_REGISTER_SCHEMA_V1: &str =
    "oxcalc.implementation_conformance.w044_disposition_register.v1";
const IMPLEMENTATION_CONFORMANCE_W044_BLOCKER_REGISTER_SCHEMA_V1: &str =
    "oxcalc.implementation_conformance.w044_blocker_register.v1";
const IMPLEMENTATION_CONFORMANCE_W044_MATCH_GUARD_SCHEMA_V1: &str =
    "oxcalc.implementation_conformance.w044_match_guard.v1";
const IMPLEMENTATION_CONFORMANCE_W044_DYNAMIC_TRANSITION_SCHEMA_V1: &str =
    "oxcalc.implementation_conformance.w044_dynamic_transition_evidence.v1";
const IMPLEMENTATION_CONFORMANCE_W044_CALLABLE_METADATA_SCHEMA_V1: &str =
    "oxcalc.implementation_conformance.w044_callable_metadata_projection_register.v1";
const IMPLEMENTATION_CONFORMANCE_W045_DISPOSITION_REGISTER_SCHEMA_V1: &str =
    "oxcalc.implementation_conformance.w045_disposition_register.v1";
const IMPLEMENTATION_CONFORMANCE_W045_BLOCKER_REGISTER_SCHEMA_V1: &str =
    "oxcalc.implementation_conformance.w045_blocker_register.v1";
const IMPLEMENTATION_CONFORMANCE_W045_MATCH_GUARD_SCHEMA_V1: &str =
    "oxcalc.implementation_conformance.w045_match_guard.v1";
const IMPLEMENTATION_CONFORMANCE_W045_DYNAMIC_TRANSITION_SCHEMA_V1: &str =
    "oxcalc.implementation_conformance.w045_dynamic_transition_coverage_register.v1";
const IMPLEMENTATION_CONFORMANCE_W045_COUNTERPART_SCHEMA_V1: &str =
    "oxcalc.implementation_conformance.w045_counterpart_coverage_register.v1";
const IMPLEMENTATION_CONFORMANCE_W045_CALLABLE_METADATA_SCHEMA_V1: &str =
    "oxcalc.implementation_conformance.w045_callable_metadata_projection_register.v1";

const W034_INDEPENDENT_CONFORMANCE_RUN_ID: &str = "w034-independent-conformance-001";
const W034_TREECALC_RUN_ID: &str = "w034-independent-conformance-treecalc-001";
const W035_ORACLE_MATRIX_RUN_ID: &str = "w035-tracecalc-oracle-matrix-001";
const W035_IMPLEMENTATION_CONFORMANCE_RUN_ID: &str =
    "w035-implementation-conformance-hardening-001";
const W036_ORACLE_MATRIX_RUN_ID: &str = "w036-tracecalc-coverage-closure-001";
const W036_IMPLEMENTATION_CONFORMANCE_RUN_ID: &str = "w036-implementation-conformance-closure-001";
const W037_IMPLEMENTATION_CONFORMANCE_RUN_ID: &str = "w037-implementation-conformance-closure-001";
const W037_ORACLE_MATRIX_RUN_ID: &str = "w037-tracecalc-observable-closure-001";
const W037_TREECALC_RUN_ID: &str = "w037-optimized-core-conformance-treecalc-001";
const W037_UPSTREAM_HOST_RUN_ID: &str = "w037-direct-oxfml-evaluator-001";
const W037_STAGE2_CRITERIA_RUN_ID: &str = "w037-stage2-deterministic-replay-criteria-001";
const W037_FORMAL_INVENTORY_RUN_ID: &str = "w037-proof-model-closure-001";
const W038_TRACECALC_AUTHORITY_RUN_ID: &str = "w038-tracecalc-authority-discharge-001";
const W038_IMPLEMENTATION_CONFORMANCE_RUN_ID: &str =
    "w038-optimized-core-conformance-disposition-001";
const W039_RESIDUAL_LEDGER_RUN_ID: &str = "w039-residual-successor-obligation-ledger-001";
const W043_IMPLEMENTATION_CONFORMANCE_RUN_ID: &str =
    "w043-optimized-core-broad-conformance-callable-metadata-closure-001";
const W044_RESIDUAL_LEDGER_RUN_ID: &str =
    "w044-residual-release-grade-blocker-reclassification-map-001";
const W044_IMPLEMENTATION_CONFORMANCE_RUN_ID: &str =
    "w044-optimized-core-dynamic-transition-callable-metadata-001";
const W044_TREECALC_RUN_ID: &str = "w044-optimized-core-dynamic-transition-treecalc-001";
const W044_MIXED_DYNAMIC_CASE_ID: &str = "tc_local_dynamic_mixed_add_release_auto_post_edit_001";
const W045_RESIDUAL_LEDGER_RUN_ID: &str =
    "w045-residual-release-grade-successor-obligation-current-oxfml-intake-map-001";

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
    pub w038_disposition_row_count: usize,
    pub w038_direct_evidence_bound_count: usize,
    pub w038_accepted_boundary_count: usize,
    pub w038_exact_remaining_blocker_count: usize,
    pub w038_match_promoted_count: usize,
    pub w039_disposition_row_count: usize,
    pub w039_direct_evidence_bound_count: usize,
    pub w039_exact_remaining_blocker_count: usize,
    pub w039_match_promoted_count: usize,
    pub w044_disposition_row_count: usize,
    pub w044_direct_evidence_bound_count: usize,
    pub w044_exact_remaining_blocker_count: usize,
    pub w044_match_promoted_count: usize,
    pub w045_disposition_row_count: usize,
    pub w045_direct_evidence_bound_count: usize,
    pub w045_exact_remaining_blocker_count: usize,
    pub w045_match_promoted_count: usize,
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

#[derive(Debug, Clone)]
struct W038DispositionSpec {
    row_id: &'static str,
    source_w037_residual_row_id: &'static str,
    w038_obligation_id: &'static str,
    w038_disposition_kind: &'static str,
    w038_disposition: &'static str,
    conformance_match_state: &'static str,
    implementation_evidence_state: &'static str,
    direct_evidence_bound: bool,
    accepted_boundary: bool,
    exact_remaining_blocker_bead: Option<&'static str>,
    authority_owner: &'static str,
    promotion_consequence: &'static str,
    reason: &'static str,
    implementation_evidence_sources: &'static [&'static str],
    required_evidence_checks: &'static [&'static str],
}

#[derive(Debug, Clone)]
struct EvaluatedW038DispositionRow {
    row: Value,
    direct_evidence_bound: bool,
    accepted_boundary: bool,
    exact_remaining_blocker: bool,
    match_promoted: bool,
    valid: bool,
}

#[derive(Debug, Clone)]
struct EvaluatedW039DispositionRow {
    row: Value,
    direct_evidence_bound: bool,
    exact_remaining_blocker: bool,
    match_promoted: bool,
    valid: bool,
}

#[derive(Debug, Clone)]
struct EvaluatedW044DispositionRow {
    row: Value,
    direct_evidence_bound: bool,
    exact_remaining_blocker: bool,
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
        if run_id.contains("w045") {
            return self.execute_w045(repo_root, run_id);
        }
        if run_id.contains("w044") {
            return self.execute_w044(repo_root, run_id);
        }
        if run_id.contains("w039") {
            return self.execute_w039(repo_root, run_id);
        }
        if run_id.contains("w038") {
            return self.execute_w038(repo_root, run_id);
        }
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
            w038_disposition_row_count: 0,
            w038_direct_evidence_bound_count: 0,
            w038_accepted_boundary_count: 0,
            w038_exact_remaining_blocker_count: 0,
            w038_match_promoted_count: 0,
            w039_disposition_row_count: 0,
            w039_direct_evidence_bound_count: 0,
            w039_exact_remaining_blocker_count: 0,
            w039_match_promoted_count: 0,
            w044_disposition_row_count: 0,
            w044_direct_evidence_bound_count: 0,
            w044_exact_remaining_blocker_count: 0,
            w044_match_promoted_count: 0,
            w045_disposition_row_count: 0,
            w045_direct_evidence_bound_count: 0,
            w045_exact_remaining_blocker_count: 0,
            w045_match_promoted_count: 0,
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
                "w038_disposition_row_count": summary.w038_disposition_row_count,
                "w038_direct_evidence_bound_count": summary.w038_direct_evidence_bound_count,
                "w038_accepted_boundary_count": summary.w038_accepted_boundary_count,
                "w038_exact_remaining_blocker_count": summary.w038_exact_remaining_blocker_count,
                "w038_match_promoted_count": summary.w038_match_promoted_count,
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
            w038_disposition_row_count: 0,
            w038_direct_evidence_bound_count: 0,
            w038_accepted_boundary_count: 0,
            w038_exact_remaining_blocker_count: 0,
            w038_match_promoted_count: 0,
            w039_disposition_row_count: 0,
            w039_direct_evidence_bound_count: 0,
            w039_exact_remaining_blocker_count: 0,
            w039_match_promoted_count: 0,
            w044_disposition_row_count: 0,
            w044_direct_evidence_bound_count: 0,
            w044_exact_remaining_blocker_count: 0,
            w044_match_promoted_count: 0,
            w045_disposition_row_count: 0,
            w045_direct_evidence_bound_count: 0,
            w045_exact_remaining_blocker_count: 0,
            w045_match_promoted_count: 0,
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
                "w038_disposition_row_count": summary.w038_disposition_row_count,
                "w038_direct_evidence_bound_count": summary.w038_direct_evidence_bound_count,
                "w038_accepted_boundary_count": summary.w038_accepted_boundary_count,
                "w038_exact_remaining_blocker_count": summary.w038_exact_remaining_blocker_count,
                "w038_match_promoted_count": summary.w038_match_promoted_count,
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
            w038_disposition_row_count: 0,
            w038_direct_evidence_bound_count: 0,
            w038_accepted_boundary_count: 0,
            w038_exact_remaining_blocker_count: 0,
            w038_match_promoted_count: 0,
            w039_disposition_row_count: 0,
            w039_direct_evidence_bound_count: 0,
            w039_exact_remaining_blocker_count: 0,
            w039_match_promoted_count: 0,
            w044_disposition_row_count: 0,
            w044_direct_evidence_bound_count: 0,
            w044_exact_remaining_blocker_count: 0,
            w044_match_promoted_count: 0,
            w045_disposition_row_count: 0,
            w045_direct_evidence_bound_count: 0,
            w045_exact_remaining_blocker_count: 0,
            w045_match_promoted_count: 0,
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
                "w038_disposition_row_count": summary.w038_disposition_row_count,
                "w038_direct_evidence_bound_count": summary.w038_direct_evidence_bound_count,
                "w038_accepted_boundary_count": summary.w038_accepted_boundary_count,
                "w038_exact_remaining_blocker_count": summary.w038_exact_remaining_blocker_count,
                "w038_match_promoted_count": summary.w038_match_promoted_count,
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

    fn execute_w038(
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

        let w037_residual_register_path = relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "implementation-conformance",
            W037_IMPLEMENTATION_CONFORMANCE_RUN_ID,
            "w037_residual_blocker_register.json",
        ]);
        let w037_match_guard_path = relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "implementation-conformance",
            W037_IMPLEMENTATION_CONFORMANCE_RUN_ID,
            "w037_match_promotion_guard.json",
        ]);
        let w037_treecalc_summary_path = relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "treecalc-local",
            W037_TREECALC_RUN_ID,
            "run_summary.json",
        ]);
        let w037_upstream_host_summary_path = relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "upstream-host",
            W037_UPSTREAM_HOST_RUN_ID,
            "run_summary.json",
        ]);
        let w038_tracecalc_authority_summary_path = relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "tracecalc-authority",
            W038_TRACECALC_AUTHORITY_RUN_ID,
            "run_summary.json",
        ]);
        let w037_stage2_decision_path = relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "stage2-criteria",
            W037_STAGE2_CRITERIA_RUN_ID,
            "promotion_decision.json",
        ]);
        let w037_formal_blockers_path = relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "formal-inventory",
            W037_FORMAL_INVENTORY_RUN_ID,
            "promotion_blockers.json",
        ]);

        let w037_residual_register = read_json(repo_root, &w037_residual_register_path)?;
        let w037_match_guard = read_json(repo_root, &w037_match_guard_path)?;
        let w037_treecalc_summary = read_json(repo_root, &w037_treecalc_summary_path)?;
        let w037_upstream_host_summary = read_json(repo_root, &w037_upstream_host_summary_path)?;
        let w038_tracecalc_authority_summary =
            read_json(repo_root, &w038_tracecalc_authority_summary_path)?;
        let w037_stage2_decision = read_json(repo_root, &w037_stage2_decision_path)?;
        let w037_formal_blockers = read_json(repo_root, &w037_formal_blockers_path)?;

        let source_rows = rows_by_id(&w037_residual_register, "row_id");
        let evaluated_rows = W038_DISPOSITION_SPECS
            .iter()
            .map(|spec| {
                evaluate_w038_disposition_row(
                    repo_root,
                    spec,
                    &source_rows,
                    &w037_treecalc_summary,
                    &w037_upstream_host_summary,
                    &w038_tracecalc_authority_summary,
                    &w037_stage2_decision,
                    &w037_formal_blockers,
                )
            })
            .collect::<Result<Vec<_>, _>>()?;

        let direct_evidence_bound_count = evaluated_rows
            .iter()
            .filter(|row| row.direct_evidence_bound)
            .count();
        let accepted_boundary_count = evaluated_rows
            .iter()
            .filter(|row| row.accepted_boundary)
            .count();
        let exact_remaining_blocker_count = evaluated_rows
            .iter()
            .filter(|row| row.exact_remaining_blocker)
            .count();
        let match_promoted_count = evaluated_rows
            .iter()
            .filter(|row| row.match_promoted)
            .count();
        let validated_row_count = evaluated_rows.iter().filter(|row| row.valid).count();
        let failed_row_count = evaluated_rows.iter().filter(|row| !row.valid).count();

        let disposition_rows = evaluated_rows
            .iter()
            .map(|row| row.row.clone())
            .collect::<Vec<_>>();
        let exact_blocker_rows = disposition_rows
            .iter()
            .filter(|row| !row["exact_remaining_blocker_bead"].is_null())
            .cloned()
            .collect::<Vec<_>>();

        let validation_failures = w038_validation_failures(
            &w037_residual_register,
            &w037_match_guard,
            &w037_treecalc_summary,
            &w037_upstream_host_summary,
            &w038_tracecalc_authority_summary,
            failed_row_count,
            match_promoted_count,
            accepted_boundary_count,
            exact_remaining_blocker_count,
        );
        let validation_status = if validation_failures.is_empty() {
            "implementation_conformance_w038_dispositions_valid"
        } else {
            "implementation_conformance_w038_dispositions_failed"
        };

        write_json(
            &artifact_root.join("w038_conformance_disposition_register.json"),
            &json!({
                "schema_version": IMPLEMENTATION_CONFORMANCE_W038_DISPOSITION_REGISTER_SCHEMA_V1,
                "run_id": run_id,
                "source_w037_implementation_conformance_run_id": W037_IMPLEMENTATION_CONFORMANCE_RUN_ID,
                "source_w037_residual_blocker_count": number_at(&w037_residual_register, "residual_blocker_count"),
                "disposition_row_count": disposition_rows.len(),
                "direct_evidence_bound_count": direct_evidence_bound_count,
                "accepted_boundary_count": accepted_boundary_count,
                "exact_remaining_blocker_count": exact_remaining_blocker_count,
                "match_promoted_count": match_promoted_count,
                "validated_row_count": validated_row_count,
                "failed_row_count": failed_row_count,
                "rows": disposition_rows,
            }),
        )?;

        write_json(
            &artifact_root.join("w038_exact_remaining_blocker_register.json"),
            &json!({
                "schema_version": IMPLEMENTATION_CONFORMANCE_W038_BLOCKER_REGISTER_SCHEMA_V1,
                "run_id": run_id,
                "exact_remaining_blocker_count": exact_blocker_rows.len(),
                "rows": exact_blocker_rows,
            }),
        )?;

        write_json(
            &artifact_root.join("w038_match_promotion_guard.json"),
            &json!({
                "schema_version": IMPLEMENTATION_CONFORMANCE_W038_MATCH_GUARD_SCHEMA_V1,
                "run_id": run_id,
                "source_w037_residual_blocker_count": number_at(&w037_residual_register, "residual_blocker_count"),
                "source_w037_promoted_match_count": number_at(&w037_match_guard, "promoted_match_count"),
                "promoted_match_count": match_promoted_count,
                "allowed_promoted_rows": [],
                "non_promoted_row_count": evaluated_rows.len().saturating_sub(match_promoted_count),
                "guard_status": if match_promoted_count == 0 {
                    "w038_declared_gap_promotion_guard_holds"
                } else {
                    "w038_declared_gap_promotion_guard_failed"
                },
                "policy": "W038 calc-zsr.3 does not count any W037 residual blocker or declared gap as an optimized/core-engine match. Direct evidence may narrow or reclassify a blocker, but match promotion remains zero in this slice.",
            }),
        )?;

        write_json(
            &artifact_root.join("evidence_summary.json"),
            &json!({
                "schema_version": IMPLEMENTATION_CONFORMANCE_EVIDENCE_SCHEMA_V1,
                "run_id": run_id,
                "source_paths": {
                    "w037_residual_blocker_register": w037_residual_register_path,
                    "w037_match_promotion_guard": w037_match_guard_path,
                    "w037_treecalc_summary": w037_treecalc_summary_path,
                    "w037_upstream_host_summary": w037_upstream_host_summary_path,
                    "w038_tracecalc_authority_summary": w038_tracecalc_authority_summary_path,
                    "w037_stage2_promotion_decision": w037_stage2_decision_path,
                    "w037_formal_promotion_blockers": w037_formal_blockers_path,
                },
                "w037_residuals": {
                    "residual_blocker_count": number_at(&w037_residual_register, "residual_blocker_count"),
                },
                "w037_treecalc_local": {
                    "case_count": number_at(&w037_treecalc_summary, "case_count"),
                    "expectation_mismatch_count": number_at(&w037_treecalc_summary, "expectation_mismatch_count"),
                    "result_counts": w037_treecalc_summary.get("result_counts").cloned().unwrap_or_else(|| json!({})),
                },
                "w037_direct_oxfml": {
                    "fixture_case_count": number_at(&w037_upstream_host_summary, "fixture_case_count"),
                    "let_lambda_case_count": number_at(&w037_upstream_host_summary, "let_lambda_case_count"),
                    "w073_typed_rule_case_count": number_at(&w037_upstream_host_summary, "w073_typed_rule_case_count"),
                    "expectation_mismatch_count": number_at(&w037_upstream_host_summary, "expectation_mismatch_count"),
                },
                "w038_tracecalc_authority": {
                    "remaining_tracecalc_authority_blocker_count": number_at(&w038_tracecalc_authority_summary, "remaining_tracecalc_authority_blocker_count"),
                    "accepted_external_exclusion_count": number_at(&w038_tracecalc_authority_summary, "accepted_external_exclusion_count"),
                    "general_oxfunc_kernel_promoted": w038_tracecalc_authority_summary["general_oxfunc_kernel_promoted"].clone(),
                },
                "w038_dispositions": {
                    "disposition_row_count": evaluated_rows.len(),
                    "direct_evidence_bound_count": direct_evidence_bound_count,
                    "accepted_boundary_count": accepted_boundary_count,
                    "exact_remaining_blocker_count": exact_remaining_blocker_count,
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
                "source_w037_residual_blocker_count": number_at(&w037_residual_register, "residual_blocker_count"),
                "w038_disposition_row_count": evaluated_rows.len(),
                "validated_row_count": validated_row_count,
                "failed_row_count": failed_row_count,
                "direct_evidence_bound_count": direct_evidence_bound_count,
                "accepted_boundary_count": accepted_boundary_count,
                "exact_remaining_blocker_count": exact_remaining_blocker_count,
                "match_promoted_count": match_promoted_count,
            }),
        )?;

        let summary = ImplementationConformanceRunSummary {
            run_id: run_id.to_string(),
            schema_version: IMPLEMENTATION_CONFORMANCE_RUN_SUMMARY_SCHEMA_V1.to_string(),
            gap_disposition_row_count: evaluated_rows.len(),
            implementation_work_count: direct_evidence_bound_count,
            spec_evolution_deferral_count: accepted_boundary_count,
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
            w038_disposition_row_count: evaluated_rows.len(),
            w038_direct_evidence_bound_count: direct_evidence_bound_count,
            w038_accepted_boundary_count: accepted_boundary_count,
            w038_exact_remaining_blocker_count: exact_remaining_blocker_count,
            w038_match_promoted_count: match_promoted_count,
            w039_disposition_row_count: 0,
            w039_direct_evidence_bound_count: 0,
            w039_exact_remaining_blocker_count: 0,
            w039_match_promoted_count: 0,
            w044_disposition_row_count: 0,
            w044_direct_evidence_bound_count: 0,
            w044_exact_remaining_blocker_count: 0,
            w044_match_promoted_count: 0,
            w045_disposition_row_count: 0,
            w045_direct_evidence_bound_count: 0,
            w045_exact_remaining_blocker_count: 0,
            w045_match_promoted_count: 0,
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
                "w038_disposition_row_count": summary.w038_disposition_row_count,
                "w038_direct_evidence_bound_count": summary.w038_direct_evidence_bound_count,
                "w038_accepted_boundary_count": summary.w038_accepted_boundary_count,
                "w038_exact_remaining_blocker_count": summary.w038_exact_remaining_blocker_count,
                "w038_match_promoted_count": summary.w038_match_promoted_count,
                "artifact_root": summary.artifact_root,
                "w038_conformance_disposition_register_path": format!("{relative_artifact_root}/w038_conformance_disposition_register.json"),
                "w038_exact_remaining_blocker_register_path": format!("{relative_artifact_root}/w038_exact_remaining_blocker_register.json"),
                "w038_match_promotion_guard_path": format!("{relative_artifact_root}/w038_match_promotion_guard.json"),
                "evidence_summary_path": format!("{relative_artifact_root}/evidence_summary.json"),
                "validation_path": format!("{relative_artifact_root}/validation.json"),
            }),
        )?;

        Ok(summary)
    }

    fn execute_w039(
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

        let w039_ledger_path = relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "release-grade-ledger",
            W039_RESIDUAL_LEDGER_RUN_ID,
            "successor_obligation_ledger.json",
        ]);
        let w039_formatting_intake_path = relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "release-grade-ledger",
            W039_RESIDUAL_LEDGER_RUN_ID,
            "w073_formatting_intake.json",
        ]);
        let w038_summary_path = relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "implementation-conformance",
            W038_IMPLEMENTATION_CONFORMANCE_RUN_ID,
            "run_summary.json",
        ]);
        let w038_disposition_path = relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "implementation-conformance",
            W038_IMPLEMENTATION_CONFORMANCE_RUN_ID,
            "w038_conformance_disposition_register.json",
        ]);
        let w038_blocker_path = relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "implementation-conformance",
            W038_IMPLEMENTATION_CONFORMANCE_RUN_ID,
            "w038_exact_remaining_blocker_register.json",
        ]);
        let w038_match_guard_path = relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "implementation-conformance",
            W038_IMPLEMENTATION_CONFORMANCE_RUN_ID,
            "w038_match_promotion_guard.json",
        ]);

        let w039_ledger = read_json(repo_root, &w039_ledger_path)?;
        let w039_formatting_intake = read_json(repo_root, &w039_formatting_intake_path)?;
        let w038_summary = read_json(repo_root, &w038_summary_path)?;
        let w038_disposition_register = read_json(repo_root, &w038_disposition_path)?;
        let w038_blocker_register = read_json(repo_root, &w038_blocker_path)?;
        let w038_match_guard = read_json(repo_root, &w038_match_guard_path)?;

        let source_blocker_rows = rows_by_id(&w038_blocker_register, "row_id");
        let obligation_rows = rows_by_id_from_array(&w039_ledger, "obligations", "obligation_id");

        let mut evaluated_rows = vec![
            evaluate_w039_exact_blocker_row(
                &source_blocker_rows,
                &obligation_rows,
                "w039_dynamic_release_reclassification_exact_blocker",
                "w038_disposition_dynamic_negative_release_reclassification",
                "W039-OBL-002",
                "retained_exact_blocker_after_w038_direct_evidence",
                "retain dynamic release/reclassification as an exact optimized/core blocker until a direct release/reclassification differential exists",
                "calc-f7o.2",
                "full optimized/core verification remains blocked",
                "W038 bound dynamic rejection, resolved dynamic publication, and retention-release guardrail evidence, but not the missing dependency release/reclassification differential.",
            ),
            evaluate_w039_exact_blocker_row(
                &source_blocker_rows,
                &obligation_rows,
                "w039_snapshot_fence_counterpart_exact_blocker",
                "w038_disposition_snapshot_fence_projection_exact_blocker",
                "W039-OBL-003",
                "retained_exact_stage2_coordinator_blocker",
                "retain snapshot-fence optimized counterpart as an exact blocker for Stage 2/coordinator replay",
                "calc-f7o.2; calc-f7o.4",
                "snapshot-fence conformance and Stage 2 production policy remain blocked",
                "W038 still lacks a deterministic stale accepted-candidate counterpart in the optimized/coordinator lane.",
            ),
            evaluate_w039_exact_blocker_row(
                &source_blocker_rows,
                &obligation_rows,
                "w039_capability_view_fence_counterpart_exact_blocker",
                "w038_disposition_capability_view_fence_projection_exact_blocker",
                "W039-OBL-003",
                "retained_exact_stage2_coordinator_blocker",
                "retain capability-view fence optimized counterpart as an exact blocker for Stage 2/coordinator replay",
                "calc-f7o.2; calc-f7o.4",
                "capability-view fence conformance and Stage 2 production policy remain blocked",
                "W038 has broader capability-sensitive reject evidence, but not the compatibility-fenced capability-view mismatch counterpart.",
            ),
            evaluate_w039_exact_blocker_row(
                &source_blocker_rows,
                &obligation_rows,
                "w039_callable_metadata_projection_exact_blocker",
                "w038_disposition_callable_metadata_projection_exact_blocker",
                "W039-OBL-004",
                "retained_exact_callable_metadata_blocker",
                "retain callable metadata projection as an exact proof/seam blocker until a projection fixture or carrier sufficiency proof exists",
                "calc-f7o.2; calc-f7o.7",
                "callable metadata projection and broad callable conformance remain blocked",
                "W038 preserves value-only TreeCalc and direct OxFml carrier evidence, but no metadata projection surface exists.",
            ),
        ];
        evaluated_rows.push(evaluate_w039_match_guard_row(
            &obligation_rows,
            &w038_match_guard,
            "W039-OBL-005",
        ));

        let direct_evidence_bound_count = evaluated_rows
            .iter()
            .filter(|row| row.direct_evidence_bound)
            .count();
        let exact_remaining_blocker_count = evaluated_rows
            .iter()
            .filter(|row| row.exact_remaining_blocker)
            .count();
        let match_promoted_count = evaluated_rows
            .iter()
            .filter(|row| row.match_promoted)
            .count();
        let validated_row_count = evaluated_rows.iter().filter(|row| row.valid).count();
        let failed_row_count = evaluated_rows.iter().filter(|row| !row.valid).count();
        let disposition_rows = evaluated_rows
            .iter()
            .map(|row| row.row.clone())
            .collect::<Vec<_>>();
        let exact_blocker_rows = disposition_rows
            .iter()
            .filter(|row| bool_at(row, "exact_remaining_blocker"))
            .cloned()
            .collect::<Vec<_>>();

        let validation_failures = w039_validation_failures(
            &w039_ledger,
            &w039_formatting_intake,
            &w038_summary,
            &w038_disposition_register,
            &w038_blocker_register,
            &w038_match_guard,
            failed_row_count,
            match_promoted_count,
            exact_remaining_blocker_count,
        );
        let validation_status = if validation_failures.is_empty() {
            "implementation_conformance_w039_exact_blockers_valid"
        } else {
            "implementation_conformance_w039_exact_blockers_failed"
        };

        write_json(
            &artifact_root.join("w039_exact_blocker_disposition_register.json"),
            &json!({
                "schema_version": IMPLEMENTATION_CONFORMANCE_W039_DISPOSITION_REGISTER_SCHEMA_V1,
                "run_id": run_id,
                "source_w039_residual_ledger_run_id": W039_RESIDUAL_LEDGER_RUN_ID,
                "source_w038_implementation_conformance_run_id": W038_IMPLEMENTATION_CONFORMANCE_RUN_ID,
                "disposition_row_count": disposition_rows.len(),
                "direct_evidence_bound_count": direct_evidence_bound_count,
                "exact_remaining_blocker_count": exact_remaining_blocker_count,
                "match_promoted_count": match_promoted_count,
                "validated_row_count": validated_row_count,
                "failed_row_count": failed_row_count,
                "rows": disposition_rows,
            }),
        )?;

        write_json(
            &artifact_root.join("w039_exact_remaining_blocker_register.json"),
            &json!({
                "schema_version": IMPLEMENTATION_CONFORMANCE_W039_BLOCKER_REGISTER_SCHEMA_V1,
                "run_id": run_id,
                "exact_remaining_blocker_count": exact_blocker_rows.len(),
                "rows": exact_blocker_rows,
            }),
        )?;

        write_json(
            &artifact_root.join("w039_match_promotion_guard.json"),
            &json!({
                "schema_version": IMPLEMENTATION_CONFORMANCE_W039_MATCH_GUARD_SCHEMA_V1,
                "run_id": run_id,
                "source_w038_promoted_match_count": number_at(&w038_match_guard, "promoted_match_count"),
                "promoted_match_count": match_promoted_count,
                "non_promoted_row_count": evaluated_rows.len().saturating_sub(match_promoted_count),
                "guard_status": if match_promoted_count == 0 {
                    "w039_declared_gap_promotion_guard_holds"
                } else {
                    "w039_declared_gap_promotion_guard_failed"
                },
                "policy": "W039 calc-f7o.2 does not count any W038 exact blocker, declared gap, accepted boundary, or retained blocker as an optimized/core-engine match. Direct evidence may narrow a blocker, but match promotion remains zero in this slice.",
                "allowed_promoted_rows": Vec::<String>::new(),
            }),
        )?;

        write_json(
            &artifact_root.join("evidence_summary.json"),
            &json!({
                "schema_version": IMPLEMENTATION_CONFORMANCE_EVIDENCE_SCHEMA_V1,
                "run_id": run_id,
                "source_paths": {
                    "w039_successor_obligation_ledger": w039_ledger_path,
                    "w039_w073_formatting_intake": w039_formatting_intake_path,
                    "w038_run_summary": w038_summary_path,
                    "w038_conformance_disposition_register": w038_disposition_path,
                    "w038_exact_remaining_blocker_register": w038_blocker_path,
                    "w038_match_promotion_guard": w038_match_guard_path,
                },
                "w039_ledger": {
                    "obligation_count": number_at(&w039_ledger, "obligation_count"),
                    "optimized_core_obligations": ["W039-OBL-002", "W039-OBL-003", "W039-OBL-004", "W039-OBL-005", "W039-OBL-008"],
                },
                "w038_disposition_inputs": {
                    "w038_disposition_row_count": number_at(&w038_summary, "w038_disposition_row_count"),
                    "w038_direct_evidence_bound_count": number_at(&w038_summary, "w038_direct_evidence_bound_count"),
                    "w038_accepted_boundary_count": number_at(&w038_summary, "w038_accepted_boundary_count"),
                    "w038_exact_remaining_blocker_count": number_at(&w038_summary, "w038_exact_remaining_blocker_count"),
                    "w038_match_promoted_count": number_at(&w038_summary, "w038_match_promoted_count"),
                },
                "w039_dispositions": {
                    "disposition_row_count": evaluated_rows.len(),
                    "direct_evidence_bound_count": direct_evidence_bound_count,
                    "exact_remaining_blocker_count": exact_remaining_blocker_count,
                    "match_promoted_count": match_promoted_count,
                    "validated_row_count": validated_row_count,
                    "failed_row_count": failed_row_count,
                },
                "w073_formatting_intake": {
                    "typed_only_family_count": array_at(&w039_formatting_intake, "typed_only_families").len(),
                    "rule": w039_formatting_intake["current_rule"].clone(),
                    "thresholds_rule": w039_formatting_intake["thresholds_rule"].clone(),
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
                "source_w038_exact_remaining_blocker_count": number_at(&w038_blocker_register, "exact_remaining_blocker_count"),
                "w039_disposition_row_count": evaluated_rows.len(),
                "validated_row_count": validated_row_count,
                "failed_row_count": failed_row_count,
                "direct_evidence_bound_count": direct_evidence_bound_count,
                "exact_remaining_blocker_count": exact_remaining_blocker_count,
                "match_promoted_count": match_promoted_count,
            }),
        )?;

        let summary = ImplementationConformanceRunSummary {
            run_id: run_id.to_string(),
            schema_version: IMPLEMENTATION_CONFORMANCE_RUN_SUMMARY_SCHEMA_V1.to_string(),
            gap_disposition_row_count: evaluated_rows.len(),
            implementation_work_count: direct_evidence_bound_count,
            spec_evolution_deferral_count: 0,
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
            w038_disposition_row_count: 0,
            w038_direct_evidence_bound_count: 0,
            w038_accepted_boundary_count: 0,
            w038_exact_remaining_blocker_count: 0,
            w038_match_promoted_count: 0,
            w039_disposition_row_count: evaluated_rows.len(),
            w039_direct_evidence_bound_count: direct_evidence_bound_count,
            w039_exact_remaining_blocker_count: exact_remaining_blocker_count,
            w039_match_promoted_count: match_promoted_count,
            w044_disposition_row_count: 0,
            w044_direct_evidence_bound_count: 0,
            w044_exact_remaining_blocker_count: 0,
            w044_match_promoted_count: 0,
            w045_disposition_row_count: 0,
            w045_direct_evidence_bound_count: 0,
            w045_exact_remaining_blocker_count: 0,
            w045_match_promoted_count: 0,
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
                "w038_disposition_row_count": summary.w038_disposition_row_count,
                "w038_direct_evidence_bound_count": summary.w038_direct_evidence_bound_count,
                "w038_accepted_boundary_count": summary.w038_accepted_boundary_count,
                "w038_exact_remaining_blocker_count": summary.w038_exact_remaining_blocker_count,
                "w038_match_promoted_count": summary.w038_match_promoted_count,
                "w039_disposition_row_count": summary.w039_disposition_row_count,
                "w039_direct_evidence_bound_count": summary.w039_direct_evidence_bound_count,
                "w039_exact_remaining_blocker_count": summary.w039_exact_remaining_blocker_count,
                "w039_match_promoted_count": summary.w039_match_promoted_count,
                "artifact_root": summary.artifact_root,
                "w039_exact_blocker_disposition_register_path": format!("{relative_artifact_root}/w039_exact_blocker_disposition_register.json"),
                "w039_exact_remaining_blocker_register_path": format!("{relative_artifact_root}/w039_exact_remaining_blocker_register.json"),
                "w039_match_promotion_guard_path": format!("{relative_artifact_root}/w039_match_promotion_guard.json"),
                "evidence_summary_path": format!("{relative_artifact_root}/evidence_summary.json"),
                "validation_path": format!("{relative_artifact_root}/validation.json"),
            }),
        )?;

        Ok(summary)
    }

    fn execute_w044(
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

        let w044_summary_path = relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "release-grade-ledger",
            W044_RESIDUAL_LEDGER_RUN_ID,
            "run_summary.json",
        ]);
        let w044_blocker_map_path = relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "release-grade-ledger",
            W044_RESIDUAL_LEDGER_RUN_ID,
            "blocker_reclassification_map.json",
        ]);
        let w043_summary_path = relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "implementation-conformance",
            W043_IMPLEMENTATION_CONFORMANCE_RUN_ID,
            "run_summary.json",
        ]);
        let w043_counterpart_path = relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "implementation-conformance",
            W043_IMPLEMENTATION_CONFORMANCE_RUN_ID,
            "w043_counterpart_conformance_register.json",
        ]);
        let w043_blocker_path = relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "implementation-conformance",
            W043_IMPLEMENTATION_CONFORMANCE_RUN_ID,
            "w043_exact_remaining_blocker_register.json",
        ]);
        let w043_callable_path = relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "implementation-conformance",
            W043_IMPLEMENTATION_CONFORMANCE_RUN_ID,
            "w043_callable_metadata_projection_register.json",
        ]);
        let w043_match_guard_path = relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "implementation-conformance",
            W043_IMPLEMENTATION_CONFORMANCE_RUN_ID,
            "w043_match_promotion_guard.json",
        ]);
        let treecalc_summary_path = relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "treecalc-local",
            W044_TREECALC_RUN_ID,
            "run_summary.json",
        ]);
        let mixed_seed_path = w044_treecalc_case_path(
            W044_MIXED_DYNAMIC_CASE_ID,
            "post_edit/invalidation_seeds.json",
        );
        let mixed_result_path =
            w044_treecalc_case_path(W044_MIXED_DYNAMIC_CASE_ID, "post_edit/result.json");
        let mixed_closure_path = w044_treecalc_case_path(
            W044_MIXED_DYNAMIC_CASE_ID,
            "post_edit/invalidation_closure.json",
        );

        let w044_summary = read_json(repo_root, &w044_summary_path)?;
        let w044_blocker_map = read_json(repo_root, &w044_blocker_map_path)?;
        let w043_summary = read_json(repo_root, &w043_summary_path)?;
        let w043_counterpart = read_json(repo_root, &w043_counterpart_path)?;
        let w043_blocker_register = read_json(repo_root, &w043_blocker_path)?;
        let w043_callable_register = read_json(repo_root, &w043_callable_path)?;
        let w043_match_guard = read_json(repo_root, &w043_match_guard_path)?;
        let treecalc_summary = read_json(repo_root, &treecalc_summary_path)?;
        let mixed_seeds = read_json(repo_root, &mixed_seed_path)?;
        let mixed_result = read_json(repo_root, &mixed_result_path)?;
        let mixed_closure = read_json(repo_root, &mixed_closure_path)?;

        let w044_lanes = rows_by_id_from_array(&w044_blocker_map, "rows", "source_lane");
        let w043_counterpart_rows = rows_by_id(&w043_counterpart, "row_id");
        let w043_blocker_rows = rows_by_id(&w043_blocker_register, "row_id");
        let w043_callable_rows = rows_by_id(&w043_callable_register, "row_id");

        let dynamic_evidence_failures = w044_dynamic_transition_failures(
            &mixed_seeds,
            &mixed_result,
            &mixed_closure,
            &treecalc_summary,
        );
        let seed_reasons = array_at_top(&mixed_seeds)
            .iter()
            .map(|seed| string_at(seed, "reason"))
            .collect::<Vec<_>>();
        let closure_reasons = array_at_top(&mixed_closure)
            .iter()
            .find(|record| number_at(record, "node_id") == 3)
            .map(|record| {
                array_at(record, "reasons")
                    .iter()
                    .filter_map(Value::as_str)
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let dynamic_evidence = json!({
            "schema_version": IMPLEMENTATION_CONFORMANCE_W044_DYNAMIC_TRANSITION_SCHEMA_V1,
            "run_id": run_id,
            "treecalc_run_id": W044_TREECALC_RUN_ID,
            "case_id": W044_MIXED_DYNAMIC_CASE_ID,
            "source_paths": {
                "treecalc_run_summary": treecalc_summary_path,
                "mixed_dynamic_invalidation_seeds": mixed_seed_path,
                "mixed_dynamic_result": mixed_result_path,
                "mixed_dynamic_invalidation_closure": mixed_closure_path,
            },
            "required_seed_reasons": [
                "DependencyAdded",
                "DependencyRemoved",
                "DependencyReclassified"
            ],
            "observed_seed_reasons": seed_reasons,
            "observed_closure_reasons": closure_reasons,
            "post_edit_result_state": string_at(&mixed_result, "result_state"),
            "post_edit_reject_kind": string_pointer(&mixed_result, "/reject_detail/kind"),
            "direct_evidence_bound": dynamic_evidence_failures.is_empty(),
            "failures": dynamic_evidence_failures,
        });
        write_json(
            &artifact_root.join("w044_dynamic_transition_evidence.json"),
            &dynamic_evidence,
        )?;

        let evaluated_rows = vec![
            evaluate_w044_dynamic_direct_row(
                &w044_lanes,
                &w043_counterpart_rows,
                &dynamic_evidence,
                &relative_artifact_root,
            ),
            evaluate_w044_dynamic_exact_blocker_row(
                &w044_lanes,
                &w043_blocker_rows,
                &dynamic_evidence,
                &relative_artifact_root,
            ),
            evaluate_w044_declared_counterpart_exact_blocker_row(
                &w044_lanes,
                &w043_counterpart_rows,
                "w044_snapshot_fence_counterpart_breadth_exact_blocker",
                "w043_snapshot_fence_counterpart_declared_profile_evidence",
                "snapshot_fence_counterpart_breadth",
                "snapshot-fence counterpart evidence remains declared-profile only; broad Stage 2 and production scheduler equivalence still need direct counterpart coverage",
                "stage2_production_policy_and_pack_equivalence_remain_unpromoted",
            ),
            evaluate_w044_declared_counterpart_exact_blocker_row(
                &w044_lanes,
                &w043_counterpart_rows,
                "w044_capability_view_counterpart_breadth_exact_blocker",
                "w043_capability_view_counterpart_declared_profile_evidence",
                "capability_view_counterpart_breadth",
                "capability-view counterpart evidence remains declared-profile only; broad capability-fence production coverage still needs direct counterpart evidence",
                "capability_view_counterpart_and_stage2_policy_remain_unpromoted",
            ),
            evaluate_w044_callable_metadata_exact_blocker_row(
                &w044_lanes,
                &w043_blocker_rows,
                &w043_callable_rows,
            ),
            evaluate_w044_match_guard_row(&w043_match_guard),
        ];

        let direct_evidence_bound_count = evaluated_rows
            .iter()
            .filter(|row| row.direct_evidence_bound)
            .count();
        let exact_remaining_blocker_count = evaluated_rows
            .iter()
            .filter(|row| row.exact_remaining_blocker)
            .count();
        let match_promoted_count = evaluated_rows
            .iter()
            .filter(|row| row.match_promoted)
            .count();
        let validated_row_count = evaluated_rows.iter().filter(|row| row.valid).count();
        let failed_row_count = evaluated_rows.iter().filter(|row| !row.valid).count();
        let disposition_rows = evaluated_rows
            .iter()
            .map(|row| row.row.clone())
            .collect::<Vec<_>>();
        let exact_blocker_rows = disposition_rows
            .iter()
            .filter(|row| bool_at(row, "exact_remaining_blocker"))
            .cloned()
            .collect::<Vec<_>>();

        let validation_failures = w044_validation_failures(
            &w044_summary,
            &w044_blocker_map,
            &w043_summary,
            &w043_counterpart,
            &w043_blocker_register,
            &w043_callable_register,
            &w043_match_guard,
            &treecalc_summary,
            &dynamic_evidence,
            failed_row_count,
            match_promoted_count,
            exact_remaining_blocker_count,
        );
        let validation_status = if validation_failures.is_empty() {
            "implementation_conformance_w044_dynamic_transition_callable_metadata_valid"
        } else {
            "implementation_conformance_w044_dynamic_transition_callable_metadata_failed"
        };

        write_json(
            &artifact_root.join("w044_optimized_core_disposition_register.json"),
            &json!({
                "schema_version": IMPLEMENTATION_CONFORMANCE_W044_DISPOSITION_REGISTER_SCHEMA_V1,
                "run_id": run_id,
                "source_w044_residual_ledger_run_id": W044_RESIDUAL_LEDGER_RUN_ID,
                "source_w043_implementation_conformance_run_id": W043_IMPLEMENTATION_CONFORMANCE_RUN_ID,
                "disposition_row_count": disposition_rows.len(),
                "direct_evidence_bound_count": direct_evidence_bound_count,
                "exact_remaining_blocker_count": exact_remaining_blocker_count,
                "match_promoted_count": match_promoted_count,
                "validated_row_count": validated_row_count,
                "failed_row_count": failed_row_count,
                "rows": disposition_rows,
            }),
        )?;

        write_json(
            &artifact_root.join("w044_exact_remaining_blocker_register.json"),
            &json!({
                "schema_version": IMPLEMENTATION_CONFORMANCE_W044_BLOCKER_REGISTER_SCHEMA_V1,
                "run_id": run_id,
                "exact_remaining_blocker_count": exact_blocker_rows.len(),
                "rows": exact_blocker_rows,
            }),
        )?;

        write_json(
            &artifact_root.join("w044_match_promotion_guard.json"),
            &json!({
                "schema_version": IMPLEMENTATION_CONFORMANCE_W044_MATCH_GUARD_SCHEMA_V1,
                "run_id": run_id,
                "source_w043_match_promoted_count": number_at(&w043_match_guard, "match_promoted_count"),
                "promoted_match_count": match_promoted_count,
                "non_promoted_row_count": evaluated_rows.len().saturating_sub(match_promoted_count),
                "guard_status": if match_promoted_count == 0 {
                    "w044_declared_gap_promotion_guard_holds"
                } else {
                    "w044_declared_gap_promotion_guard_failed"
                },
                "policy": "W044 calc-b1t.2 binds direct mixed dynamic transition evidence and retains exact blockers without counting declared profiles, note-level evidence, external formatting intake, callable value carriers, or retained blockers as optimized/core matches.",
                "allowed_promoted_rows": Vec::<String>::new(),
            }),
        )?;

        write_json(
            &artifact_root.join("w044_callable_metadata_projection_register.json"),
            &json!({
                "schema_version": IMPLEMENTATION_CONFORMANCE_W044_CALLABLE_METADATA_SCHEMA_V1,
                "run_id": run_id,
                "source_w043_callable_metadata_projection_register": w043_callable_path,
                "callable_metadata_projection_promoted": false,
                "row_count": 3,
                "rows": [
                    {
                        "row_id": "w044_treecalc_callable_value_carrier_carried",
                        "source_w043_row_id": "w043_treecalc_callable_value_only_evidence",
                        "evidence_state": "value_carrier_evidenced",
                        "metadata_projection_evidenced": false,
                        "consequence": "TreeCalc LET/LAMBDA value-carrier evidence is retained as value evidence only."
                    },
                    {
                        "row_id": "w044_upstream_host_callable_carrier_carried",
                        "source_w043_row_id": "w043_upstream_host_let_lambda_carrier_evidence_carried",
                        "evidence_state": "carrier_evidenced",
                        "metadata_projection_evidenced": false,
                        "consequence": "Direct OxFml LET/LAMBDA carrier evidence is retained as carrier evidence only."
                    },
                    {
                        "row_id": "w044_callable_metadata_projection_exact_blocker",
                        "source_w043_row_id": "w043_callable_metadata_projection_exact_blocker",
                        "evidence_state": "exact_blocker_retained",
                        "metadata_projection_evidenced": false,
                        "consequence": "Callable metadata projection remains an exact blocker in W044.2."
                    }
                ],
            }),
        )?;

        write_json(
            &artifact_root.join("evidence_summary.json"),
            &json!({
                "schema_version": IMPLEMENTATION_CONFORMANCE_EVIDENCE_SCHEMA_V1,
                "run_id": run_id,
                "source_paths": {
                    "w044_release_grade_run_summary": w044_summary_path,
                    "w044_blocker_reclassification_map": w044_blocker_map_path,
                    "w043_run_summary": w043_summary_path,
                    "w043_counterpart_conformance_register": w043_counterpart_path,
                    "w043_exact_remaining_blocker_register": w043_blocker_path,
                    "w043_callable_metadata_projection_register": w043_callable_path,
                    "w043_match_promotion_guard": w043_match_guard_path,
                    "w044_treecalc_run_summary": treecalc_summary_path,
                    "w044_dynamic_transition_evidence": format!("{relative_artifact_root}/w044_dynamic_transition_evidence.json"),
                },
                "w044_release_grade_inputs": {
                    "source_residual_lane_count": number_at(&w044_summary, "source_residual_lane_count"),
                    "obligation_count": number_at(&w044_summary, "obligation_count"),
                    "promotion_contract_count": number_at(&w044_summary, "promotion_contract_count"),
                    "oxfml_formatting_update_incorporated": bool_at(&w044_summary, "oxfml_formatting_update_incorporated"),
                    "w073_downstream_request_construction_uptake_verified": bool_at(&w044_summary, "w073_downstream_request_construction_uptake_verified"),
                },
                "w043_inputs": {
                    "disposition_row_count": number_at(&w043_summary, "disposition_row_count"),
                    "direct_evidence_bound_count": number_at(&w043_summary, "direct_evidence_bound_count"),
                    "exact_remaining_blocker_count": number_at(&w043_summary, "exact_remaining_blocker_count"),
                    "match_promoted_count": number_at(&w043_summary, "match_promoted_count"),
                },
                "w044_treecalc_inputs": {
                    "treecalc_run_id": W044_TREECALC_RUN_ID,
                    "case_count": number_at(&treecalc_summary, "case_count"),
                    "expectation_mismatch_count": number_at(&treecalc_summary, "expectation_mismatch_count"),
                    "mixed_dynamic_case_id": W044_MIXED_DYNAMIC_CASE_ID,
                },
                "w044_dispositions": {
                    "disposition_row_count": evaluated_rows.len(),
                    "direct_evidence_bound_count": direct_evidence_bound_count,
                    "exact_remaining_blocker_count": exact_remaining_blocker_count,
                    "match_promoted_count": match_promoted_count,
                    "validated_row_count": validated_row_count,
                    "failed_row_count": failed_row_count,
                },
                "no_promotion_claims": [
                    "full_optimized_core_verification",
                    "release_grade_full_verification",
                    "stage2_production_policy",
                    "callable_metadata_projection",
                    "callable_carrier_sufficiency",
                    "pack_grade_replay",
                    "cap.C5.pack_valid",
                    "general_oxfunc_kernels"
                ],
            }),
        )?;

        write_json(
            &artifact_root.join("validation.json"),
            &json!({
                "schema_version": IMPLEMENTATION_CONFORMANCE_VALIDATION_SCHEMA_V1,
                "run_id": run_id,
                "status": validation_status,
                "validation_failures": validation_failures,
                "w044_disposition_row_count": evaluated_rows.len(),
                "validated_row_count": validated_row_count,
                "failed_row_count": failed_row_count,
                "direct_evidence_bound_count": direct_evidence_bound_count,
                "exact_remaining_blocker_count": exact_remaining_blocker_count,
                "match_promoted_count": match_promoted_count,
                "treecalc_case_count": number_at(&treecalc_summary, "case_count"),
                "treecalc_expectation_mismatch_count": number_at(&treecalc_summary, "expectation_mismatch_count"),
            }),
        )?;

        let summary = ImplementationConformanceRunSummary {
            run_id: run_id.to_string(),
            schema_version: IMPLEMENTATION_CONFORMANCE_RUN_SUMMARY_SCHEMA_V1.to_string(),
            gap_disposition_row_count: evaluated_rows.len(),
            implementation_work_count: direct_evidence_bound_count,
            spec_evolution_deferral_count: 0,
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
            w038_disposition_row_count: 0,
            w038_direct_evidence_bound_count: 0,
            w038_accepted_boundary_count: 0,
            w038_exact_remaining_blocker_count: 0,
            w038_match_promoted_count: 0,
            w039_disposition_row_count: 0,
            w039_direct_evidence_bound_count: 0,
            w039_exact_remaining_blocker_count: 0,
            w039_match_promoted_count: 0,
            w044_disposition_row_count: evaluated_rows.len(),
            w044_direct_evidence_bound_count: direct_evidence_bound_count,
            w044_exact_remaining_blocker_count: exact_remaining_blocker_count,
            w044_match_promoted_count: match_promoted_count,
            w045_disposition_row_count: 0,
            w045_direct_evidence_bound_count: 0,
            w045_exact_remaining_blocker_count: 0,
            w045_match_promoted_count: 0,
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
                "w038_disposition_row_count": summary.w038_disposition_row_count,
                "w038_direct_evidence_bound_count": summary.w038_direct_evidence_bound_count,
                "w038_accepted_boundary_count": summary.w038_accepted_boundary_count,
                "w038_exact_remaining_blocker_count": summary.w038_exact_remaining_blocker_count,
                "w038_match_promoted_count": summary.w038_match_promoted_count,
                "w039_disposition_row_count": summary.w039_disposition_row_count,
                "w039_direct_evidence_bound_count": summary.w039_direct_evidence_bound_count,
                "w039_exact_remaining_blocker_count": summary.w039_exact_remaining_blocker_count,
                "w039_match_promoted_count": summary.w039_match_promoted_count,
                "w044_disposition_row_count": summary.w044_disposition_row_count,
                "w044_direct_evidence_bound_count": summary.w044_direct_evidence_bound_count,
                "w044_exact_remaining_blocker_count": summary.w044_exact_remaining_blocker_count,
                "w044_match_promoted_count": summary.w044_match_promoted_count,
                "w045_disposition_row_count": summary.w045_disposition_row_count,
                "w045_direct_evidence_bound_count": summary.w045_direct_evidence_bound_count,
                "w045_exact_remaining_blocker_count": summary.w045_exact_remaining_blocker_count,
                "w045_match_promoted_count": summary.w045_match_promoted_count,
                "artifact_root": summary.artifact_root,
                "w044_optimized_core_disposition_register_path": format!("{relative_artifact_root}/w044_optimized_core_disposition_register.json"),
                "w044_exact_remaining_blocker_register_path": format!("{relative_artifact_root}/w044_exact_remaining_blocker_register.json"),
                "w044_match_promotion_guard_path": format!("{relative_artifact_root}/w044_match_promotion_guard.json"),
                "w044_dynamic_transition_evidence_path": format!("{relative_artifact_root}/w044_dynamic_transition_evidence.json"),
                "w044_callable_metadata_projection_register_path": format!("{relative_artifact_root}/w044_callable_metadata_projection_register.json"),
                "evidence_summary_path": format!("{relative_artifact_root}/evidence_summary.json"),
                "validation_path": format!("{relative_artifact_root}/validation.json"),
            }),
        )?;

        Ok(summary)
    }

    fn execute_w045(
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

        let w045_summary_path = relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "release-grade-ledger",
            W045_RESIDUAL_LEDGER_RUN_ID,
            "run_summary.json",
        ]);
        let w045_obligation_map_path = relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "release-grade-ledger",
            W045_RESIDUAL_LEDGER_RUN_ID,
            "successor_obligation_map.json",
        ]);
        let w045_oxfml_intake_path = relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "release-grade-ledger",
            W045_RESIDUAL_LEDGER_RUN_ID,
            "oxfml_inbound_observation_intake.json",
        ]);
        let w044_summary_path = relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "implementation-conformance",
            W044_IMPLEMENTATION_CONFORMANCE_RUN_ID,
            "run_summary.json",
        ]);
        let w044_disposition_path = relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "implementation-conformance",
            W044_IMPLEMENTATION_CONFORMANCE_RUN_ID,
            "w044_optimized_core_disposition_register.json",
        ]);
        let w044_blocker_path = relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "implementation-conformance",
            W044_IMPLEMENTATION_CONFORMANCE_RUN_ID,
            "w044_exact_remaining_blocker_register.json",
        ]);
        let w044_dynamic_path = relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "implementation-conformance",
            W044_IMPLEMENTATION_CONFORMANCE_RUN_ID,
            "w044_dynamic_transition_evidence.json",
        ]);
        let w044_callable_path = relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "implementation-conformance",
            W044_IMPLEMENTATION_CONFORMANCE_RUN_ID,
            "w044_callable_metadata_projection_register.json",
        ]);
        let w044_match_guard_path = relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "implementation-conformance",
            W044_IMPLEMENTATION_CONFORMANCE_RUN_ID,
            "w044_match_promotion_guard.json",
        ]);
        let treecalc_summary_path = relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "treecalc-local",
            W044_TREECALC_RUN_ID,
            "run_summary.json",
        ]);

        let w045_summary = read_json(repo_root, &w045_summary_path)?;
        let w045_obligation_map = read_json(repo_root, &w045_obligation_map_path)?;
        let w045_oxfml_intake = read_json(repo_root, &w045_oxfml_intake_path)?;
        let w044_summary = read_json(repo_root, &w044_summary_path)?;
        let w044_disposition = read_json(repo_root, &w044_disposition_path)?;
        let w044_blocker_register = read_json(repo_root, &w044_blocker_path)?;
        let w044_dynamic = read_json(repo_root, &w044_dynamic_path)?;
        let w044_callable = read_json(repo_root, &w044_callable_path)?;
        let w044_match_guard = read_json(repo_root, &w044_match_guard_path)?;
        let treecalc_summary = read_json(repo_root, &treecalc_summary_path)?;

        let obligation_rows = rows_by_id_from_array(&w045_obligation_map, "obligations", "id");
        let w044_disposition_rows = rows_by_id(&w044_disposition, "row_id");
        let w044_blocker_rows = rows_by_id(&w044_blocker_register, "row_id");

        let dynamic_register_rows = vec![
            json!({
                "row_id": "w045_dynamic_mixed_transition_carried_direct_evidence",
                "source_w044_row_id": "w044_dynamic_mixed_add_release_direct_evidence",
                "source_w044_dynamic_evidence": w044_dynamic_path,
                "direct_evidence_bound": bool_at(&w044_dynamic, "direct_evidence_bound"),
                "evidence_state": "carried_direct_evidence",
                "promotion_consequence": "mixed add/remove/reclassify evidence is retained as direct evidence but not broad optimized/core promotion"
            }),
            json!({
                "row_id": "w045_broader_dynamic_transition_exact_blocker",
                "source_w044_row_id": "w044_broader_dynamic_transition_remaining_exact_blocker",
                "direct_evidence_bound": true,
                "exact_remaining_blocker": true,
                "required_expansion": [
                    "additional descriptor transitions",
                    "structural-plus-formula edits",
                    "host-resolution surfaces",
                    "sufficiency proof"
                ],
                "promotion_consequence": "full optimized/core dynamic transition coverage remains unpromoted"
            }),
            json!({
                "row_id": "w045_soft_reference_indirect_resolution_exact_blocker",
                "source_w045_obligation_id": "W045-OBL-006",
                "direct_evidence_bound": false,
                "exact_remaining_blocker": true,
                "required_expansion": [
                    "INDIRECT selector churn",
                    "late reference resolution",
                    "soft-reference update breadth beyond the W044 mixed case"
                ],
                "promotion_consequence": "soft-reference and INDIRECT coverage remains an exact optimized/core blocker"
            }),
        ];
        write_json(
            &artifact_root.join("w045_dynamic_transition_coverage_register.json"),
            &json!({
                "schema_version": IMPLEMENTATION_CONFORMANCE_W045_DYNAMIC_TRANSITION_SCHEMA_V1,
                "run_id": run_id,
                "source_w044_implementation_conformance_run_id": W044_IMPLEMENTATION_CONFORMANCE_RUN_ID,
                "source_w044_treecalc_run_id": W044_TREECALC_RUN_ID,
                "dynamic_transition_row_count": dynamic_register_rows.len(),
                "direct_evidence_bound_count": dynamic_register_rows
                    .iter()
                    .filter(|row| bool_at(row, "direct_evidence_bound"))
                    .count(),
                "exact_remaining_blocker_count": dynamic_register_rows
                    .iter()
                    .filter(|row| bool_at(row, "exact_remaining_blocker"))
                    .count(),
                "rows": dynamic_register_rows,
            }),
        )?;

        let counterpart_rows = vec![
            json!({
                "row_id": "w045_snapshot_fence_counterpart_breadth_exact_blocker",
                "source_w044_row_id": "w044_snapshot_fence_counterpart_breadth_exact_blocker",
                "source_w045_obligation_ids": ["W045-OBL-007", "W045-OBL-020"],
                "direct_evidence_bound": false,
                "exact_remaining_blocker": true,
                "promotion_consequence": "snapshot-fence counterpart breadth and Stage 2 policy remain unpromoted"
            }),
            json!({
                "row_id": "w045_capability_view_counterpart_breadth_exact_blocker",
                "source_w044_row_id": "w044_capability_view_counterpart_breadth_exact_blocker",
                "source_w045_obligation_ids": ["W045-OBL-008", "W045-OBL-020"],
                "direct_evidence_bound": false,
                "exact_remaining_blocker": true,
                "promotion_consequence": "capability-view counterpart breadth and Stage 2 policy remain unpromoted"
            }),
        ];
        write_json(
            &artifact_root.join("w045_counterpart_coverage_register.json"),
            &json!({
                "schema_version": IMPLEMENTATION_CONFORMANCE_W045_COUNTERPART_SCHEMA_V1,
                "run_id": run_id,
                "source_w044_exact_remaining_blocker_register": w044_blocker_path,
                "counterpart_row_count": counterpart_rows.len(),
                "exact_remaining_blocker_count": counterpart_rows.len(),
                "rows": counterpart_rows,
            }),
        )?;

        write_json(
            &artifact_root.join("w045_callable_metadata_projection_register.json"),
            &json!({
                "schema_version": IMPLEMENTATION_CONFORMANCE_W045_CALLABLE_METADATA_SCHEMA_V1,
                "run_id": run_id,
                "source_w044_callable_metadata_projection_register": w044_callable_path,
                "callable_metadata_projection_promoted": false,
                "row_count": 4,
                "rows": [
                    {
                        "row_id": "w045_treecalc_callable_value_carrier_carried",
                        "source_w044_row_id": "w044_treecalc_callable_value_carrier_carried",
                        "evidence_state": "value_carrier_evidenced",
                        "metadata_projection_evidenced": false,
                        "consequence": "TreeCalc LET/LAMBDA value-carrier evidence remains value evidence only."
                    },
                    {
                        "row_id": "w045_upstream_host_callable_carrier_carried",
                        "source_w044_row_id": "w044_upstream_host_callable_carrier_carried",
                        "evidence_state": "carrier_evidenced",
                        "metadata_projection_evidenced": false,
                        "consequence": "Direct OxFml LET/LAMBDA carrier evidence remains carrier evidence only."
                    },
                    {
                        "row_id": "w045_callable_metadata_projection_exact_blocker",
                        "source_w044_row_id": "w044_callable_metadata_projection_exact_blocker",
                        "evidence_state": "exact_blocker_retained",
                        "metadata_projection_evidenced": false,
                        "consequence": "Callable metadata projection remains an exact blocker in W045.2."
                    },
                    {
                        "row_id": "w045_registered_external_provider_publication_watch",
                        "source_w045_obligation_id": "W045-OBL-034",
                        "evidence_state": "watch_lane_retained",
                        "metadata_projection_evidenced": false,
                        "consequence": "Registered-external callable projection and provider/callable publication remain W045.8 seam obligations."
                    }
                ],
            }),
        )?;

        let evaluated_rows = vec![
            evaluate_w045_dynamic_carried_direct_row(
                &obligation_rows,
                &w044_disposition_rows,
                &w044_dynamic,
                &relative_artifact_root,
            ),
            evaluate_w045_dynamic_exact_blocker_row(
                &obligation_rows,
                &w044_blocker_rows,
                &relative_artifact_root,
            ),
            evaluate_w045_soft_reference_indirect_exact_blocker_row(&obligation_rows),
            evaluate_w045_counterpart_exact_blocker_row(
                &obligation_rows,
                &w044_blocker_rows,
                "w045_snapshot_fence_counterpart_breadth_exact_blocker",
                "w044_snapshot_fence_counterpart_breadth_exact_blocker",
                "W045-OBL-007",
                "snapshot_fence_counterpart_breadth",
                "snapshot-fence counterpart evidence remains declared-profile only; broad production scheduler equivalence still needs direct counterpart coverage",
                "stage2_production_policy_and_pack_equivalence_remain_unpromoted",
            ),
            evaluate_w045_counterpart_exact_blocker_row(
                &obligation_rows,
                &w044_blocker_rows,
                "w045_capability_view_counterpart_breadth_exact_blocker",
                "w044_capability_view_counterpart_breadth_exact_blocker",
                "W045-OBL-008",
                "capability_view_counterpart_breadth",
                "capability-view counterpart evidence remains declared-profile only; broad capability-fence production coverage still needs direct counterpart evidence",
                "capability_view_counterpart_and_stage2_policy_remain_unpromoted",
            ),
            evaluate_w045_callable_metadata_exact_blocker_row(
                &obligation_rows,
                &w044_blocker_rows,
                &w044_callable,
            ),
            evaluate_w045_match_guard_row(&obligation_rows, &w044_match_guard),
        ];

        let direct_evidence_bound_count = evaluated_rows
            .iter()
            .filter(|row| row.direct_evidence_bound)
            .count();
        let exact_remaining_blocker_count = evaluated_rows
            .iter()
            .filter(|row| row.exact_remaining_blocker)
            .count();
        let match_promoted_count = evaluated_rows
            .iter()
            .filter(|row| row.match_promoted)
            .count();
        let validated_row_count = evaluated_rows.iter().filter(|row| row.valid).count();
        let failed_row_count = evaluated_rows.iter().filter(|row| !row.valid).count();
        let disposition_rows = evaluated_rows
            .iter()
            .map(|row| row.row.clone())
            .collect::<Vec<_>>();
        let exact_blocker_rows = disposition_rows
            .iter()
            .filter(|row| bool_at(row, "exact_remaining_blocker"))
            .cloned()
            .collect::<Vec<_>>();

        let validation_failures = w045_validation_failures(
            &w045_summary,
            &w045_obligation_map,
            &w045_oxfml_intake,
            &w044_summary,
            &w044_disposition,
            &w044_blocker_register,
            &w044_dynamic,
            &w044_callable,
            &w044_match_guard,
            &treecalc_summary,
            failed_row_count,
            match_promoted_count,
            exact_remaining_blocker_count,
        );
        let validation_status = if validation_failures.is_empty() {
            "implementation_conformance_w045_optimized_core_counterpart_callable_metadata_valid"
        } else {
            "implementation_conformance_w045_optimized_core_counterpart_callable_metadata_failed"
        };

        write_json(
            &artifact_root.join("w045_optimized_core_disposition_register.json"),
            &json!({
                "schema_version": IMPLEMENTATION_CONFORMANCE_W045_DISPOSITION_REGISTER_SCHEMA_V1,
                "run_id": run_id,
                "source_w045_residual_ledger_run_id": W045_RESIDUAL_LEDGER_RUN_ID,
                "source_w044_implementation_conformance_run_id": W044_IMPLEMENTATION_CONFORMANCE_RUN_ID,
                "disposition_row_count": disposition_rows.len(),
                "direct_evidence_bound_count": direct_evidence_bound_count,
                "exact_remaining_blocker_count": exact_remaining_blocker_count,
                "match_promoted_count": match_promoted_count,
                "validated_row_count": validated_row_count,
                "failed_row_count": failed_row_count,
                "rows": disposition_rows,
            }),
        )?;

        write_json(
            &artifact_root.join("w045_exact_remaining_blocker_register.json"),
            &json!({
                "schema_version": IMPLEMENTATION_CONFORMANCE_W045_BLOCKER_REGISTER_SCHEMA_V1,
                "run_id": run_id,
                "exact_remaining_blocker_count": exact_blocker_rows.len(),
                "rows": exact_blocker_rows,
            }),
        )?;

        write_json(
            &artifact_root.join("w045_match_promotion_guard.json"),
            &json!({
                "schema_version": IMPLEMENTATION_CONFORMANCE_W045_MATCH_GUARD_SCHEMA_V1,
                "run_id": run_id,
                "source_w044_match_promoted_count": number_at(&w044_match_guard, "promoted_match_count"),
                "promoted_match_count": match_promoted_count,
                "non_promoted_row_count": evaluated_rows.len().saturating_sub(match_promoted_count),
                "guard_status": if match_promoted_count == 0 {
                    "w045_declared_gap_promotion_guard_holds"
                } else {
                    "w045_declared_gap_promotion_guard_failed"
                },
                "policy": "W045 calc-zkio.2 carries direct mixed dynamic transition evidence and retains exact blockers without counting W044 declared-profile counterparts, W073 formatting intake, callable value carriers, registered-external watch rows, or retained blockers as optimized/core matches.",
                "allowed_promoted_rows": Vec::<String>::new(),
            }),
        )?;

        write_json(
            &artifact_root.join("evidence_summary.json"),
            &json!({
                "schema_version": IMPLEMENTATION_CONFORMANCE_EVIDENCE_SCHEMA_V1,
                "run_id": run_id,
                "source_paths": {
                    "w045_release_grade_run_summary": w045_summary_path,
                    "w045_successor_obligation_map": w045_obligation_map_path,
                    "w045_oxfml_inbound_observation_intake": w045_oxfml_intake_path,
                    "w044_run_summary": w044_summary_path,
                    "w044_optimized_core_disposition_register": w044_disposition_path,
                    "w044_exact_remaining_blocker_register": w044_blocker_path,
                    "w044_dynamic_transition_evidence": w044_dynamic_path,
                    "w044_callable_metadata_projection_register": w044_callable_path,
                    "w044_match_promotion_guard": w044_match_guard_path,
                    "w044_treecalc_run_summary": treecalc_summary_path,
                },
                "w045_release_grade_inputs": {
                    "source_residual_lane_count": number_at(&w045_summary, "source_residual_lane_count"),
                    "successor_obligation_count": number_at(&w045_summary, "successor_obligation_count"),
                    "promotion_contract_count": number_at(&w045_summary, "promotion_contract_count"),
                    "oxfml_formatting_update_incorporated": bool_at(&w045_summary, "oxfml_formatting_update_incorporated"),
                    "w073_downstream_request_construction_uptake_verified_by_oxcalc": bool_at(&w045_summary, "w073_downstream_request_construction_uptake_verified_by_oxcalc"),
                },
                "w044_inputs": {
                    "disposition_row_count": number_at(&w044_summary, "w044_disposition_row_count"),
                    "direct_evidence_bound_count": number_at(&w044_summary, "w044_direct_evidence_bound_count"),
                    "exact_remaining_blocker_count": number_at(&w044_summary, "w044_exact_remaining_blocker_count"),
                    "match_promoted_count": number_at(&w044_summary, "w044_match_promoted_count"),
                },
                "w044_treecalc_inputs": {
                    "treecalc_run_id": W044_TREECALC_RUN_ID,
                    "case_count": number_at(&treecalc_summary, "case_count"),
                    "expectation_mismatch_count": number_at(&treecalc_summary, "expectation_mismatch_count"),
                    "mixed_dynamic_case_id": W044_MIXED_DYNAMIC_CASE_ID,
                },
                "w045_dispositions": {
                    "disposition_row_count": evaluated_rows.len(),
                    "direct_evidence_bound_count": direct_evidence_bound_count,
                    "exact_remaining_blocker_count": exact_remaining_blocker_count,
                    "match_promoted_count": match_promoted_count,
                    "validated_row_count": validated_row_count,
                    "failed_row_count": failed_row_count,
                },
                "no_promotion_claims": [
                    "full_optimized_core_verification",
                    "release_grade_full_verification",
                    "stage2_production_policy",
                    "callable_metadata_projection",
                    "callable_carrier_sufficiency",
                    "pack_grade_replay",
                    "cap.C5.pack_valid",
                    "registered_external_callable_projection",
                    "provider_failure_callable_publication_semantics",
                    "general_oxfunc_kernels"
                ],
            }),
        )?;

        write_json(
            &artifact_root.join("validation.json"),
            &json!({
                "schema_version": IMPLEMENTATION_CONFORMANCE_VALIDATION_SCHEMA_V1,
                "run_id": run_id,
                "status": validation_status,
                "validation_failures": validation_failures,
                "w045_disposition_row_count": evaluated_rows.len(),
                "validated_row_count": validated_row_count,
                "failed_row_count": failed_row_count,
                "direct_evidence_bound_count": direct_evidence_bound_count,
                "exact_remaining_blocker_count": exact_remaining_blocker_count,
                "match_promoted_count": match_promoted_count,
                "treecalc_case_count": number_at(&treecalc_summary, "case_count"),
                "treecalc_expectation_mismatch_count": number_at(&treecalc_summary, "expectation_mismatch_count"),
            }),
        )?;

        let summary = ImplementationConformanceRunSummary {
            run_id: run_id.to_string(),
            schema_version: IMPLEMENTATION_CONFORMANCE_RUN_SUMMARY_SCHEMA_V1.to_string(),
            gap_disposition_row_count: evaluated_rows.len(),
            implementation_work_count: direct_evidence_bound_count,
            spec_evolution_deferral_count: 0,
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
            w038_disposition_row_count: 0,
            w038_direct_evidence_bound_count: 0,
            w038_accepted_boundary_count: 0,
            w038_exact_remaining_blocker_count: 0,
            w038_match_promoted_count: 0,
            w039_disposition_row_count: 0,
            w039_direct_evidence_bound_count: 0,
            w039_exact_remaining_blocker_count: 0,
            w039_match_promoted_count: 0,
            w044_disposition_row_count: 0,
            w044_direct_evidence_bound_count: 0,
            w044_exact_remaining_blocker_count: 0,
            w044_match_promoted_count: 0,
            w045_disposition_row_count: evaluated_rows.len(),
            w045_direct_evidence_bound_count: direct_evidence_bound_count,
            w045_exact_remaining_blocker_count: exact_remaining_blocker_count,
            w045_match_promoted_count: match_promoted_count,
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
                "w038_disposition_row_count": summary.w038_disposition_row_count,
                "w038_direct_evidence_bound_count": summary.w038_direct_evidence_bound_count,
                "w038_accepted_boundary_count": summary.w038_accepted_boundary_count,
                "w038_exact_remaining_blocker_count": summary.w038_exact_remaining_blocker_count,
                "w038_match_promoted_count": summary.w038_match_promoted_count,
                "w039_disposition_row_count": summary.w039_disposition_row_count,
                "w039_direct_evidence_bound_count": summary.w039_direct_evidence_bound_count,
                "w039_exact_remaining_blocker_count": summary.w039_exact_remaining_blocker_count,
                "w039_match_promoted_count": summary.w039_match_promoted_count,
                "w044_disposition_row_count": summary.w044_disposition_row_count,
                "w044_direct_evidence_bound_count": summary.w044_direct_evidence_bound_count,
                "w044_exact_remaining_blocker_count": summary.w044_exact_remaining_blocker_count,
                "w044_match_promoted_count": summary.w044_match_promoted_count,
                "w045_disposition_row_count": summary.w045_disposition_row_count,
                "w045_direct_evidence_bound_count": summary.w045_direct_evidence_bound_count,
                "w045_exact_remaining_blocker_count": summary.w045_exact_remaining_blocker_count,
                "w045_match_promoted_count": summary.w045_match_promoted_count,
                "artifact_root": summary.artifact_root,
                "w045_optimized_core_disposition_register_path": format!("{relative_artifact_root}/w045_optimized_core_disposition_register.json"),
                "w045_exact_remaining_blocker_register_path": format!("{relative_artifact_root}/w045_exact_remaining_blocker_register.json"),
                "w045_match_promotion_guard_path": format!("{relative_artifact_root}/w045_match_promotion_guard.json"),
                "w045_dynamic_transition_coverage_register_path": format!("{relative_artifact_root}/w045_dynamic_transition_coverage_register.json"),
                "w045_counterpart_coverage_register_path": format!("{relative_artifact_root}/w045_counterpart_coverage_register.json"),
                "w045_callable_metadata_projection_register_path": format!("{relative_artifact_root}/w045_callable_metadata_projection_register.json"),
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

#[allow(clippy::too_many_arguments)]
fn evaluate_w039_exact_blocker_row(
    source_blocker_rows: &BTreeMap<String, Value>,
    obligation_rows: &BTreeMap<String, Value>,
    row_id: &'static str,
    source_w038_row_id: &'static str,
    w039_obligation_id: &'static str,
    w039_disposition_kind: &'static str,
    w039_disposition: &'static str,
    authority_owner: &'static str,
    promotion_consequence: &'static str,
    reason: &'static str,
) -> EvaluatedW039DispositionRow {
    let mut failures = Vec::new();
    let source_row = source_blocker_rows.get(source_w038_row_id);
    if let Some(row) = source_row {
        if string_at(row, "validation_state") != "w038_disposition_validated" {
            failures.push("source_w038_blocker_not_validated".to_string());
        }
        if string_at(row, "conformance_match_state") != "not_promoted" {
            failures.push("source_w038_blocker_unexpectedly_promoted".to_string());
        }
        if !array_at(row, "failures").is_empty() {
            failures.push("source_w038_blocker_has_failures".to_string());
        }
    } else {
        failures.push("source_w038_exact_blocker_missing".to_string());
    }
    let obligation = obligation_rows.get(w039_obligation_id);
    if obligation.is_none() {
        failures.push("w039_obligation_missing".to_string());
    }

    let source_direct_evidence_bound =
        source_row.is_some_and(|row| bool_at(row, "direct_evidence_bound"));
    let source_disposition_kind = source_row
        .map(|row| string_at(row, "w038_disposition_kind"))
        .unwrap_or_default();
    let source_is_exact = source_disposition_kind.contains("exact_remaining_blocker");
    if !source_is_exact {
        failures.push("source_w038_row_is_not_exact_blocker".to_string());
    }

    EvaluatedW039DispositionRow {
        row: json!({
            "row_id": row_id,
            "source_w038_row_id": source_w038_row_id,
            "w039_obligation_id": w039_obligation_id,
            "source_w038_disposition": source_row.map(|row| json!({
                "row_id": row["row_id"],
                "w038_obligation_id": row["w038_obligation_id"],
                "w038_disposition_kind": row["w038_disposition_kind"],
                "w038_disposition": row["w038_disposition"],
                "direct_evidence_bound": row["direct_evidence_bound"],
                "accepted_boundary": row["accepted_boundary"],
                "exact_remaining_blocker_bead": row["exact_remaining_blocker_bead"],
                "implementation_evidence_state": row["implementation_evidence_state"],
                "promotion_consequence": row["promotion_consequence"],
            })).unwrap_or(Value::Null),
            "w039_obligation": obligation.map(|row| json!({
                "obligation_id": row["obligation_id"],
                "area": row["area"],
                "owner_beads": row["owner_beads"],
                "required_evidence": row["required_evidence"],
                "promotion_consequence": row["promotion_consequence"],
            })).unwrap_or(Value::Null),
            "w039_disposition_kind": w039_disposition_kind,
            "w039_disposition": w039_disposition,
            "conformance_match_state": "not_promoted",
            "direct_evidence_bound": source_direct_evidence_bound,
            "exact_remaining_blocker": true,
            "match_promoted": false,
            "authority_owner": authority_owner,
            "promotion_consequence": promotion_consequence,
            "reason": reason,
            "validation_state": if failures.is_empty() { "w039_exact_blocker_validated" } else { "w039_exact_blocker_failed" },
            "failures": failures,
        }),
        direct_evidence_bound: source_direct_evidence_bound,
        exact_remaining_blocker: true,
        match_promoted: false,
        valid: failures.is_empty(),
    }
}

fn evaluate_w039_match_guard_row(
    obligation_rows: &BTreeMap<String, Value>,
    w038_match_guard: &Value,
    w039_obligation_id: &'static str,
) -> EvaluatedW039DispositionRow {
    let mut failures = Vec::new();
    if number_at(w038_match_guard, "promoted_match_count") != 0 {
        failures.push("source_w038_match_guard_has_promoted_matches".to_string());
    }
    if obligation_rows.get(w039_obligation_id).is_none() {
        failures.push("w039_match_guard_obligation_missing".to_string());
    }

    EvaluatedW039DispositionRow {
        row: json!({
            "row_id": "w039_declared_gap_match_promotion_guard",
            "source_w038_guard_status": w038_match_guard["guard_status"],
            "w039_obligation_id": w039_obligation_id,
            "w039_disposition_kind": "match_promotion_guard",
            "w039_disposition": "retain zero match promotion for W038 exact blockers and declared gaps",
            "conformance_match_state": "not_promoted",
            "direct_evidence_bound": false,
            "exact_remaining_blocker": false,
            "match_promoted": false,
            "authority_owner": "calc-f7o.2; calc-f7o.8; calc-f7o.9",
            "promotion_consequence": "full optimized/core verification, pack-grade replay, C5, and release-grade claims remain blocked if any declared gap or exact blocker is counted as a match",
            "reason": "W039 preserves the W038 declared-gap guard and records no promoted optimized/core match rows in this slice.",
            "validation_state": if failures.is_empty() { "w039_match_guard_validated" } else { "w039_match_guard_failed" },
            "failures": failures,
        }),
        direct_evidence_bound: false,
        exact_remaining_blocker: false,
        match_promoted: false,
        valid: failures.is_empty(),
    }
}

fn evaluate_w044_dynamic_direct_row(
    w044_lanes: &BTreeMap<String, Value>,
    w043_counterpart_rows: &BTreeMap<String, Value>,
    dynamic_evidence: &Value,
    relative_artifact_root: &str,
) -> EvaluatedW044DispositionRow {
    let mut failures = Vec::new();
    if w044_lanes
        .get("w043_residual.optimized_core_dynamic_dependency_transitions")
        .is_none()
    {
        failures.push("w044_dynamic_transition_lane_missing".to_string());
    }
    for source_row_id in [
        "w043_dynamic_addition_reclassification_direct_evidence",
        "w043_dynamic_release_reclassification_carried_evidence",
    ] {
        if let Some(source_row) = w043_counterpart_rows.get(source_row_id) {
            if !bool_at(source_row, "direct_evidence_bound") {
                failures.push(format!(
                    "source_w043_dynamic_row_not_direct:{source_row_id}"
                ));
            }
            if bool_at(source_row, "match_promoted") {
                failures.push(format!("source_w043_dynamic_row_promoted:{source_row_id}"));
            }
        } else {
            failures.push(format!("source_w043_dynamic_row_missing:{source_row_id}"));
        }
    }
    if !bool_at(dynamic_evidence, "direct_evidence_bound") {
        failures.push("w044_mixed_dynamic_direct_evidence_failed".to_string());
    }
    if !array_at(dynamic_evidence, "failures").is_empty() {
        failures.push("w044_mixed_dynamic_evidence_has_failures".to_string());
    }

    EvaluatedW044DispositionRow {
        row: json!({
            "row_id": "w044_dynamic_mixed_add_release_direct_evidence",
            "source_w044_lane": "w043_residual.optimized_core_dynamic_dependency_transitions",
            "source_w043_row_ids": [
                "w043_dynamic_addition_reclassification_direct_evidence",
                "w043_dynamic_release_reclassification_carried_evidence"
            ],
            "w044_obligation_ids": ["W044-OBL-004", "W044-OBL-005", "W044-OBL-006", "W044-OBL-011"],
            "policy_area": "mixed_dynamic_soft_reference_transition",
            "disposition_kind": "direct_evidence_bound_narrowing",
            "disposition": "bind mixed automatic DependencyAdded, DependencyRemoved, and DependencyReclassified evidence for one soft-reference owner after a formula-catalog transition",
            "conformance_match_state": "not_promoted",
            "direct_evidence_bound": true,
            "exact_remaining_blocker": false,
            "match_promoted": false,
            "authority_owner": "calc-b1t.2",
            "promotion_consequence": "mixed dynamic transition evidence narrows the blocker but does not promote broad dynamic transition coverage",
            "source_artifacts": [
                format!("{relative_artifact_root}/w044_dynamic_transition_evidence.json")
            ],
            "validation_state": if failures.is_empty() { "w044_disposition_validated" } else { "w044_disposition_failed" },
            "failures": failures,
        }),
        direct_evidence_bound: true,
        exact_remaining_blocker: false,
        match_promoted: false,
        valid: failures.is_empty(),
    }
}

fn evaluate_w044_dynamic_exact_blocker_row(
    w044_lanes: &BTreeMap<String, Value>,
    w043_blocker_rows: &BTreeMap<String, Value>,
    dynamic_evidence: &Value,
    relative_artifact_root: &str,
) -> EvaluatedW044DispositionRow {
    let mut failures = Vec::new();
    if w044_lanes
        .get("w043_residual.optimized_core_dynamic_dependency_transitions")
        .is_none()
    {
        failures.push("w044_dynamic_transition_lane_missing".to_string());
    }
    if w043_blocker_rows
        .get("w043_broader_dynamic_transition_coverage_remaining_exact_blocker")
        .is_none()
    {
        failures.push("source_w043_broader_dynamic_blocker_missing".to_string());
    }
    if !bool_at(dynamic_evidence, "direct_evidence_bound") {
        failures.push("w044_mixed_dynamic_direct_evidence_failed".to_string());
    }

    EvaluatedW044DispositionRow {
        row: json!({
            "row_id": "w044_broader_dynamic_transition_remaining_exact_blocker",
            "source_w044_lane": "w043_residual.optimized_core_dynamic_dependency_transitions",
            "source_w043_row_id": "w043_broader_dynamic_transition_coverage_remaining_exact_blocker",
            "w044_obligation_ids": ["W044-OBL-004", "W044-OBL-005", "W044-OBL-006", "W044-OBL-011"],
            "policy_area": "broader_automatic_dynamic_dependency_transition_coverage",
            "disposition_kind": "retained_exact_blocker_after_direct_evidence",
            "disposition": "retain broader dynamic transition coverage as an exact optimized/core blocker after adding mixed add+release+reclassification evidence",
            "conformance_match_state": "not_promoted",
            "direct_evidence_bound": true,
            "exact_remaining_blocker": true,
            "match_promoted": false,
            "authority_owner": "calc-b1t.2",
            "promotion_consequence": "full optimized/core verification remains blocked until dynamic coverage spans additional descriptor transitions, structural-plus-formula edits, host-resolution surfaces, or a sufficiency proof",
            "source_artifacts": [
                format!("{relative_artifact_root}/w044_dynamic_transition_evidence.json")
            ],
            "validation_state": if failures.is_empty() { "w044_disposition_validated" } else { "w044_disposition_failed" },
            "failures": failures,
        }),
        direct_evidence_bound: true,
        exact_remaining_blocker: true,
        match_promoted: false,
        valid: failures.is_empty(),
    }
}

fn evaluate_w044_declared_counterpart_exact_blocker_row(
    w044_lanes: &BTreeMap<String, Value>,
    w043_counterpart_rows: &BTreeMap<String, Value>,
    row_id: &'static str,
    source_w043_row_id: &'static str,
    policy_area: &'static str,
    disposition: &'static str,
    promotion_consequence: &'static str,
) -> EvaluatedW044DispositionRow {
    let mut failures = Vec::new();
    if w044_lanes
        .get("w043_residual.stage2_production_policy_and_pack_equivalence")
        .is_none()
    {
        failures.push("w044_stage2_policy_lane_missing".to_string());
    }
    if let Some(source_row) = w043_counterpart_rows.get(source_w043_row_id) {
        if !bool_at(source_row, "direct_evidence_bound") {
            failures.push(format!(
                "source_w043_counterpart_row_not_direct:{source_w043_row_id}"
            ));
        }
        if bool_at(source_row, "match_promoted") {
            failures.push(format!(
                "source_w043_counterpart_row_promoted:{source_w043_row_id}"
            ));
        }
    } else {
        failures.push(format!(
            "source_w043_counterpart_row_missing:{source_w043_row_id}"
        ));
    }

    EvaluatedW044DispositionRow {
        row: json!({
            "row_id": row_id,
            "source_w044_lane": "w043_residual.stage2_production_policy_and_pack_equivalence",
            "source_w043_row_id": source_w043_row_id,
            "w044_obligation_ids": ["W044-OBL-019", "W044-OBL-020", "W044-OBL-021", "W044-OBL-022"],
            "policy_area": policy_area,
            "disposition_kind": "retained_exact_blocker_declared_profile_only",
            "disposition": disposition,
            "conformance_match_state": "not_promoted",
            "direct_evidence_bound": false,
            "exact_remaining_blocker": true,
            "match_promoted": false,
            "authority_owner": "calc-b1t.2; calc-b1t.5",
            "promotion_consequence": promotion_consequence,
            "source_artifacts": [
                "docs/test-runs/core-engine/implementation-conformance/w043-optimized-core-broad-conformance-callable-metadata-closure-001/w043_counterpart_conformance_register.json"
            ],
            "validation_state": if failures.is_empty() { "w044_disposition_validated" } else { "w044_disposition_failed" },
            "failures": failures,
        }),
        direct_evidence_bound: false,
        exact_remaining_blocker: true,
        match_promoted: false,
        valid: failures.is_empty(),
    }
}

fn evaluate_w044_callable_metadata_exact_blocker_row(
    w044_lanes: &BTreeMap<String, Value>,
    w043_blocker_rows: &BTreeMap<String, Value>,
    w043_callable_rows: &BTreeMap<String, Value>,
) -> EvaluatedW044DispositionRow {
    let mut failures = Vec::new();
    if w044_lanes
        .get("w043_residual.callable_metadata_projection")
        .is_none()
    {
        failures.push("w044_callable_metadata_lane_missing".to_string());
    }
    if w043_blocker_rows
        .get("w043_callable_metadata_projection_exact_blocker")
        .is_none()
    {
        failures.push("source_w043_callable_metadata_blocker_missing".to_string());
    }
    if let Some(source_row) =
        w043_callable_rows.get("w043_callable_metadata_projection_exact_blocker")
    {
        if bool_at(source_row, "metadata_projection_evidenced") {
            failures.push("source_w043_callable_metadata_projection_already_evidenced".to_string());
        }
    } else {
        failures.push("source_w043_callable_metadata_projection_row_missing".to_string());
    }

    EvaluatedW044DispositionRow {
        row: json!({
            "row_id": "w044_callable_metadata_projection_exact_blocker",
            "source_w044_lane": "w043_residual.callable_metadata_projection",
            "source_w043_row_id": "w043_callable_metadata_projection_exact_blocker",
            "w044_obligation_ids": ["W044-OBL-009", "W044-OBL-010", "W044-OBL-038", "W044-OBL-039"],
            "policy_area": "callable_metadata_projection",
            "disposition_kind": "retained_exact_callable_metadata_blocker",
            "disposition": "retain callable metadata projection as an exact blocker; LET/LAMBDA value-carrier and direct OxFml carrier evidence are not metadata projection evidence",
            "conformance_match_state": "not_promoted",
            "direct_evidence_bound": false,
            "exact_remaining_blocker": true,
            "match_promoted": false,
            "authority_owner": "calc-b1t.2; calc-b1t.8",
            "promotion_consequence": "callable metadata projection, registered external callable projection, and provider-failure callable publication semantics remain unpromoted",
            "source_artifacts": [
                "docs/test-runs/core-engine/implementation-conformance/w043-optimized-core-broad-conformance-callable-metadata-closure-001/w043_callable_metadata_projection_register.json",
                "docs/test-runs/core-engine/implementation-conformance/w043-optimized-core-broad-conformance-callable-metadata-closure-001/w043_exact_remaining_blocker_register.json"
            ],
            "validation_state": if failures.is_empty() { "w044_disposition_validated" } else { "w044_disposition_failed" },
            "failures": failures,
        }),
        direct_evidence_bound: false,
        exact_remaining_blocker: true,
        match_promoted: false,
        valid: failures.is_empty(),
    }
}

fn evaluate_w044_match_guard_row(w043_match_guard: &Value) -> EvaluatedW044DispositionRow {
    let mut failures = Vec::new();
    if number_at(w043_match_guard, "match_promoted_count") != 0 {
        failures.push("source_w043_match_guard_has_promoted_matches".to_string());
    }

    EvaluatedW044DispositionRow {
        row: json!({
            "row_id": "w044_declared_gap_match_promotion_guard",
            "source_w043_guard_rows": w043_match_guard["guard_rows"].clone(),
            "w044_obligation_ids": ["W044-OBL-001", "W044-OBL-002", "W044-OBL-003", "W044-OBL-044"],
            "policy_area": "no_proxy_promotion_guard",
            "disposition_kind": "match_promotion_guard",
            "disposition": "retain zero match promotion for W044 exact blockers, declared-profile counterparts, W073 formatting intake, callable value carriers, and release-grade residuals",
            "conformance_match_state": "not_promoted",
            "direct_evidence_bound": false,
            "exact_remaining_blocker": false,
            "match_promoted": false,
            "authority_owner": "calc-b1t.2; calc-b1t.11",
            "promotion_consequence": "full optimized/core, release-grade, C5, pack-grade, Stage 2, callable metadata, and general OxFunc claims remain blocked if any proxy row is counted as a match",
            "source_artifacts": [
                "docs/test-runs/core-engine/implementation-conformance/w043-optimized-core-broad-conformance-callable-metadata-closure-001/w043_match_promotion_guard.json"
            ],
            "validation_state": if failures.is_empty() { "w044_disposition_validated" } else { "w044_disposition_failed" },
            "failures": failures,
        }),
        direct_evidence_bound: false,
        exact_remaining_blocker: false,
        match_promoted: false,
        valid: failures.is_empty(),
    }
}

fn require_w045_obligations(
    obligation_rows: &BTreeMap<String, Value>,
    failures: &mut Vec<String>,
    obligation_ids: &[&str],
) {
    for obligation_id in obligation_ids {
        if obligation_rows.get(*obligation_id).is_none() {
            failures.push(format!("w045_obligation_missing:{obligation_id}"));
        }
    }
}

fn evaluate_w045_dynamic_carried_direct_row(
    obligation_rows: &BTreeMap<String, Value>,
    w044_disposition_rows: &BTreeMap<String, Value>,
    w044_dynamic: &Value,
    relative_artifact_root: &str,
) -> EvaluatedW044DispositionRow {
    let mut failures = Vec::new();
    require_w045_obligations(
        obligation_rows,
        &mut failures,
        &["W045-OBL-005", "W045-OBL-006", "W045-OBL-011"],
    );
    if let Some(source_row) =
        w044_disposition_rows.get("w044_dynamic_mixed_add_release_direct_evidence")
    {
        if string_at(source_row, "validation_state") != "w044_disposition_validated" {
            failures.push("source_w044_dynamic_direct_row_not_validated".to_string());
        }
        if !bool_at(source_row, "direct_evidence_bound") {
            failures.push("source_w044_dynamic_direct_row_not_direct".to_string());
        }
        if bool_at(source_row, "match_promoted") {
            failures.push("source_w044_dynamic_direct_row_promoted".to_string());
        }
    } else {
        failures.push("source_w044_dynamic_direct_row_missing".to_string());
    }
    if !bool_at(w044_dynamic, "direct_evidence_bound") {
        failures.push("w044_dynamic_transition_evidence_not_direct".to_string());
    }
    if !array_at(w044_dynamic, "failures").is_empty() {
        failures.push("w044_dynamic_transition_evidence_has_failures".to_string());
    }

    EvaluatedW044DispositionRow {
        row: json!({
            "row_id": "w045_dynamic_mixed_transition_carried_direct_evidence",
            "source_w044_row_id": "w044_dynamic_mixed_add_release_direct_evidence",
            "w045_obligation_ids": ["W045-OBL-005", "W045-OBL-006", "W045-OBL-011"],
            "policy_area": "mixed_dynamic_soft_reference_transition",
            "disposition_kind": "carried_direct_evidence_narrowing",
            "disposition": "carry W044 mixed automatic DependencyAdded, DependencyRemoved, and DependencyReclassified evidence as direct dynamic-transition evidence",
            "conformance_match_state": "not_promoted",
            "direct_evidence_bound": true,
            "exact_remaining_blocker": false,
            "match_promoted": false,
            "authority_owner": "calc-zkio.2",
            "promotion_consequence": "mixed dynamic transition evidence narrows the blocker but does not promote broad optimized/core coverage",
            "source_artifacts": [
                format!("{relative_artifact_root}/w045_dynamic_transition_coverage_register.json")
            ],
            "validation_state": if failures.is_empty() { "w045_disposition_validated" } else { "w045_disposition_failed" },
            "failures": failures,
        }),
        direct_evidence_bound: true,
        exact_remaining_blocker: false,
        match_promoted: false,
        valid: failures.is_empty(),
    }
}

fn evaluate_w045_dynamic_exact_blocker_row(
    obligation_rows: &BTreeMap<String, Value>,
    w044_blocker_rows: &BTreeMap<String, Value>,
    relative_artifact_root: &str,
) -> EvaluatedW044DispositionRow {
    let mut failures = Vec::new();
    require_w045_obligations(
        obligation_rows,
        &mut failures,
        &["W045-OBL-005", "W045-OBL-006", "W045-OBL-011"],
    );
    if let Some(source_row) =
        w044_blocker_rows.get("w044_broader_dynamic_transition_remaining_exact_blocker")
    {
        if string_at(source_row, "validation_state") != "w044_disposition_validated" {
            failures.push("source_w044_dynamic_blocker_not_validated".to_string());
        }
        if !bool_at(source_row, "exact_remaining_blocker") {
            failures.push("source_w044_dynamic_blocker_not_exact".to_string());
        }
        if bool_at(source_row, "match_promoted") {
            failures.push("source_w044_dynamic_blocker_promoted".to_string());
        }
    } else {
        failures.push("source_w044_dynamic_blocker_missing".to_string());
    }

    EvaluatedW044DispositionRow {
        row: json!({
            "row_id": "w045_broader_dynamic_transition_remaining_exact_blocker",
            "source_w044_row_id": "w044_broader_dynamic_transition_remaining_exact_blocker",
            "w045_obligation_ids": ["W045-OBL-005", "W045-OBL-006", "W045-OBL-011"],
            "policy_area": "broader_automatic_dynamic_dependency_transition_coverage",
            "disposition_kind": "retained_exact_blocker_after_carried_direct_evidence",
            "disposition": "retain broader dynamic transition coverage as an exact optimized/core blocker after carrying W044 mixed add/remove/reclassify evidence",
            "conformance_match_state": "not_promoted",
            "direct_evidence_bound": true,
            "exact_remaining_blocker": true,
            "match_promoted": false,
            "authority_owner": "calc-zkio.2",
            "promotion_consequence": "full optimized/core verification remains blocked until dynamic coverage spans additional descriptor transitions, structural-plus-formula edits, host-resolution surfaces, or a sufficiency proof",
            "source_artifacts": [
                format!("{relative_artifact_root}/w045_dynamic_transition_coverage_register.json")
            ],
            "validation_state": if failures.is_empty() { "w045_disposition_validated" } else { "w045_disposition_failed" },
            "failures": failures,
        }),
        direct_evidence_bound: true,
        exact_remaining_blocker: true,
        match_promoted: false,
        valid: failures.is_empty(),
    }
}

fn evaluate_w045_soft_reference_indirect_exact_blocker_row(
    obligation_rows: &BTreeMap<String, Value>,
) -> EvaluatedW044DispositionRow {
    let mut failures = Vec::new();
    require_w045_obligations(obligation_rows, &mut failures, &["W045-OBL-006"]);

    EvaluatedW044DispositionRow {
        row: json!({
            "row_id": "w045_soft_reference_indirect_resolution_exact_blocker",
            "source_w045_obligation_id": "W045-OBL-006",
            "policy_area": "soft_reference_indirect_late_resolution_breadth",
            "disposition_kind": "retained_exact_blocker_needs_new_direct_evidence",
            "disposition": "retain soft-reference, INDIRECT, and late reference-resolution update breadth as an exact optimized/core blocker beyond the W044 mixed dynamic case",
            "conformance_match_state": "not_promoted",
            "direct_evidence_bound": false,
            "exact_remaining_blocker": true,
            "match_promoted": false,
            "authority_owner": "calc-zkio.2",
            "promotion_consequence": "soft-reference and INDIRECT coverage remains unpromoted until direct broader fixture evidence or a sufficiency proof exists",
            "source_artifacts": [
                "docs/test-runs/core-engine/release-grade-ledger/w045-residual-release-grade-successor-obligation-current-oxfml-intake-map-001/successor_obligation_map.json"
            ],
            "validation_state": if failures.is_empty() { "w045_disposition_validated" } else { "w045_disposition_failed" },
            "failures": failures,
        }),
        direct_evidence_bound: false,
        exact_remaining_blocker: true,
        match_promoted: false,
        valid: failures.is_empty(),
    }
}

#[allow(clippy::too_many_arguments)]
fn evaluate_w045_counterpart_exact_blocker_row(
    obligation_rows: &BTreeMap<String, Value>,
    w044_blocker_rows: &BTreeMap<String, Value>,
    row_id: &'static str,
    source_w044_row_id: &'static str,
    w045_obligation_id: &'static str,
    policy_area: &'static str,
    disposition: &'static str,
    promotion_consequence: &'static str,
) -> EvaluatedW044DispositionRow {
    let mut failures = Vec::new();
    require_w045_obligations(obligation_rows, &mut failures, &[w045_obligation_id]);
    if let Some(source_row) = w044_blocker_rows.get(source_w044_row_id) {
        if string_at(source_row, "validation_state") != "w044_disposition_validated" {
            failures.push(format!(
                "source_w044_counterpart_not_validated:{source_w044_row_id}"
            ));
        }
        if !bool_at(source_row, "exact_remaining_blocker") {
            failures.push(format!(
                "source_w044_counterpart_not_exact:{source_w044_row_id}"
            ));
        }
        if bool_at(source_row, "match_promoted") {
            failures.push(format!(
                "source_w044_counterpart_promoted:{source_w044_row_id}"
            ));
        }
    } else {
        failures.push(format!(
            "source_w044_counterpart_missing:{source_w044_row_id}"
        ));
    }

    EvaluatedW044DispositionRow {
        row: json!({
            "row_id": row_id,
            "source_w044_row_id": source_w044_row_id,
            "w045_obligation_ids": [w045_obligation_id],
            "policy_area": policy_area,
            "disposition_kind": "retained_exact_blocker_declared_profile_only",
            "disposition": disposition,
            "conformance_match_state": "not_promoted",
            "direct_evidence_bound": false,
            "exact_remaining_blocker": true,
            "match_promoted": false,
            "authority_owner": "calc-zkio.2; calc-zkio.5",
            "promotion_consequence": promotion_consequence,
            "source_artifacts": [
                "docs/test-runs/core-engine/implementation-conformance/w044-optimized-core-dynamic-transition-callable-metadata-001/w044_exact_remaining_blocker_register.json"
            ],
            "validation_state": if failures.is_empty() { "w045_disposition_validated" } else { "w045_disposition_failed" },
            "failures": failures,
        }),
        direct_evidence_bound: false,
        exact_remaining_blocker: true,
        match_promoted: false,
        valid: failures.is_empty(),
    }
}

fn evaluate_w045_callable_metadata_exact_blocker_row(
    obligation_rows: &BTreeMap<String, Value>,
    w044_blocker_rows: &BTreeMap<String, Value>,
    w044_callable: &Value,
) -> EvaluatedW044DispositionRow {
    let mut failures = Vec::new();
    require_w045_obligations(
        obligation_rows,
        &mut failures,
        &["W045-OBL-009", "W045-OBL-010", "W045-OBL-034"],
    );
    if w044_blocker_rows
        .get("w044_callable_metadata_projection_exact_blocker")
        .is_none()
    {
        failures.push("source_w044_callable_metadata_blocker_missing".to_string());
    }
    if bool_at(w044_callable, "callable_metadata_projection_promoted") {
        failures.push("source_w044_callable_metadata_projection_promoted".to_string());
    }

    EvaluatedW044DispositionRow {
        row: json!({
            "row_id": "w045_callable_metadata_projection_exact_blocker",
            "source_w044_row_id": "w044_callable_metadata_projection_exact_blocker",
            "w045_obligation_ids": ["W045-OBL-009", "W045-OBL-010", "W045-OBL-034"],
            "policy_area": "callable_metadata_projection",
            "disposition_kind": "retained_exact_callable_metadata_blocker",
            "disposition": "retain callable metadata projection as an exact blocker; LET/LAMBDA value-carrier evidence, registered-external watch rows, and direct OxFml carrier evidence are not metadata projection evidence",
            "conformance_match_state": "not_promoted",
            "direct_evidence_bound": false,
            "exact_remaining_blocker": true,
            "match_promoted": false,
            "authority_owner": "calc-zkio.2; calc-zkio.8",
            "promotion_consequence": "callable metadata projection, registered external callable projection, and provider-failure callable publication semantics remain unpromoted",
            "source_artifacts": [
                "docs/test-runs/core-engine/implementation-conformance/w044-optimized-core-dynamic-transition-callable-metadata-001/w044_callable_metadata_projection_register.json",
                "docs/test-runs/core-engine/implementation-conformance/w044-optimized-core-dynamic-transition-callable-metadata-001/w044_exact_remaining_blocker_register.json"
            ],
            "validation_state": if failures.is_empty() { "w045_disposition_validated" } else { "w045_disposition_failed" },
            "failures": failures,
        }),
        direct_evidence_bound: false,
        exact_remaining_blocker: true,
        match_promoted: false,
        valid: failures.is_empty(),
    }
}

fn evaluate_w045_match_guard_row(
    obligation_rows: &BTreeMap<String, Value>,
    w044_match_guard: &Value,
) -> EvaluatedW044DispositionRow {
    let mut failures = Vec::new();
    require_w045_obligations(
        obligation_rows,
        &mut failures,
        &["W045-OBL-001", "W045-OBL-002", "W045-OBL-011"],
    );
    if number_at(w044_match_guard, "promoted_match_count") != 0 {
        failures.push("source_w044_match_guard_has_promoted_matches".to_string());
    }

    EvaluatedW044DispositionRow {
        row: json!({
            "row_id": "w045_declared_gap_match_promotion_guard",
            "source_w044_guard_status": w044_match_guard["guard_status"],
            "w045_obligation_ids": ["W045-OBL-001", "W045-OBL-002", "W045-OBL-011"],
            "policy_area": "no_proxy_promotion_guard",
            "disposition_kind": "match_promotion_guard",
            "disposition": "retain zero match promotion for W044 exact blockers, declared-profile counterparts, W073 formatting intake, callable value carriers, registered-external watch rows, and release-grade residuals",
            "conformance_match_state": "not_promoted",
            "direct_evidence_bound": false,
            "exact_remaining_blocker": false,
            "match_promoted": false,
            "authority_owner": "calc-zkio.2; calc-zkio.11",
            "promotion_consequence": "full optimized/core, release-grade, C5, pack-grade, Stage 2, callable metadata, registered-external, and general OxFunc claims remain blocked if any proxy row is counted as a match",
            "source_artifacts": [
                "docs/test-runs/core-engine/implementation-conformance/w044-optimized-core-dynamic-transition-callable-metadata-001/w044_match_promotion_guard.json"
            ],
            "validation_state": if failures.is_empty() { "w045_disposition_validated" } else { "w045_disposition_failed" },
            "failures": failures,
        }),
        direct_evidence_bound: false,
        exact_remaining_blocker: false,
        match_promoted: false,
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

fn evaluate_w038_disposition_row(
    repo_root: &Path,
    spec: &W038DispositionSpec,
    source_rows: &BTreeMap<String, Value>,
    treecalc_summary: &Value,
    upstream_host_summary: &Value,
    tracecalc_authority_summary: &Value,
    stage2_decision: &Value,
    formal_blockers: &Value,
) -> Result<EvaluatedW038DispositionRow, ImplementationConformanceError> {
    let mut failures = Vec::new();
    let source_row = source_rows.get(spec.source_w037_residual_row_id);
    if let Some(row) = source_row {
        if string_at(row, "validation_state") != "w037_decision_validated" {
            failures.push("source_w037_residual_not_validated".to_string());
        }
        if string_at(row, "w037_decision_kind") != "residual_blocker" {
            failures.push("source_w037_row_not_residual_blocker".to_string());
        }
        if string_at(row, "conformance_match_state") != "not_promoted" {
            failures.push("source_w037_residual_was_promoted".to_string());
        }
        if !array_at(row, "failures").is_empty() {
            failures.push("source_w037_residual_has_failures".to_string());
        }
    } else {
        failures.push("source_w037_residual_missing".to_string());
    }

    let mut evidence_checks = Vec::new();
    for check_id in spec.required_evidence_checks {
        let check = evaluate_w038_evidence_check(
            repo_root,
            check_id,
            treecalc_summary,
            upstream_host_summary,
            tracecalc_authority_summary,
            stage2_decision,
            formal_blockers,
        )?;
        if string_at(&check, "status") != "passed" {
            failures.push(format!("w038_evidence_check_failed:{check_id}"));
        }
        evidence_checks.push(check);
    }

    if spec.conformance_match_state == "promoted_match" {
        failures.push("w038_residual_match_promotion_forbidden".to_string());
    }
    if spec.w038_disposition_kind == "exact_remaining_blocker"
        && spec.exact_remaining_blocker_bead.is_none()
    {
        failures.push("exact_remaining_blocker_without_owner_bead".to_string());
    }
    if spec.accepted_boundary && spec.exact_remaining_blocker_bead.is_some() {
        failures.push("accepted_boundary_also_marked_exact_remaining_blocker".to_string());
    }

    let exact_remaining_blocker_bead = spec
        .exact_remaining_blocker_bead
        .map_or(Value::Null, |blocker| json!(blocker));
    let source_w037_residual = source_row
        .map(|row| {
            json!({
                "row_id": row["row_id"],
                "source_w036_action_row_id": row["source_w036_action_row_id"],
                "w037_obligation_id": row["w037_obligation_id"],
                "w037_decision_kind": row["w037_decision_kind"],
                "w037_decision": row["w037_decision"],
                "conformance_match_state": row["conformance_match_state"],
                "implementation_evidence_state": row["implementation_evidence_state"],
                "residual_blocker_bead": row["residual_blocker_bead"],
            })
        })
        .unwrap_or(Value::Null);

    Ok(EvaluatedW038DispositionRow {
        row: json!({
            "row_id": spec.row_id,
            "source_w037_residual_row_id": spec.source_w037_residual_row_id,
            "source_w037_residual": source_w037_residual,
            "w038_obligation_id": spec.w038_obligation_id,
            "w038_disposition_kind": spec.w038_disposition_kind,
            "w038_disposition": spec.w038_disposition,
            "conformance_match_state": spec.conformance_match_state,
            "implementation_evidence_state": spec.implementation_evidence_state,
            "direct_evidence_bound": spec.direct_evidence_bound,
            "accepted_boundary": spec.accepted_boundary,
            "exact_remaining_blocker_bead": exact_remaining_blocker_bead,
            "authority_owner": spec.authority_owner,
            "promotion_consequence": spec.promotion_consequence,
            "reason": spec.reason,
            "implementation_evidence_sources": spec.implementation_evidence_sources,
            "evidence_checks": evidence_checks,
            "validation_state": if failures.is_empty() { "w038_disposition_validated" } else { "w038_disposition_failed" },
            "failures": failures,
        }),
        direct_evidence_bound: spec.direct_evidence_bound,
        accepted_boundary: spec.accepted_boundary,
        exact_remaining_blocker: spec.exact_remaining_blocker_bead.is_some(),
        match_promoted: spec.conformance_match_state == "promoted_match",
        valid: failures.is_empty(),
    })
}

fn evaluate_w038_evidence_check(
    repo_root: &Path,
    check_id: &str,
    treecalc_summary: &Value,
    upstream_host_summary: &Value,
    tracecalc_authority_summary: &Value,
    stage2_decision: &Value,
    formal_blockers: &Value,
) -> Result<Value, ImplementationConformanceError> {
    let check = match check_id {
        "w037_treecalc_dynamic_negative_reject" => {
            let result_path = w037_treecalc_case_path("tc_local_dynamic_reject_001", "result.json");
            let overlay_path = w037_treecalc_case_path(
                "tc_local_dynamic_reject_001",
                "runtime_effect_overlays.json",
            );
            let result = read_json(repo_root, &result_path)?;
            let overlays = read_json(repo_root, &overlay_path)?;
            let dynamic_overlay_count = array_at_top(&overlays)
                .iter()
                .filter(|overlay| string_at(overlay, "overlay_kind") == "DynamicDependency")
                .count();
            let passed = string_at(&result, "result_state") == "rejected"
                && string_pointer(&result, "/reject_detail/kind") == "DynamicDependencyFailure"
                && dynamic_overlay_count > 0;
            json!({
                "check_id": check_id,
                "status": if passed { "passed" } else { "failed" },
                "artifact_paths": [result_path, overlay_path],
                "observed": {
                    "result_state": string_at(&result, "result_state"),
                    "reject_kind": string_pointer(&result, "/reject_detail/kind"),
                    "dynamic_overlay_count": dynamic_overlay_count,
                },
            })
        }
        "w037_treecalc_dynamic_resolved_publish" => {
            let result_path =
                w037_treecalc_case_path("tc_local_dynamic_resolved_publish_001", "result.json");
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
            let passed = string_at(&result, "result_state") == "published"
                && dependency_shape_update_count > 0
                && dynamic_runtime_effect_count > 0;
            json!({
                "check_id": check_id,
                "status": if passed { "passed" } else { "failed" },
                "artifact_paths": [result_path],
                "observed": {
                    "result_state": string_at(&result, "result_state"),
                    "dependency_shape_update_count": dependency_shape_update_count,
                    "dynamic_runtime_effect_count": dynamic_runtime_effect_count,
                },
            })
        }
        "w037_treecalc_dynamic_retention_release" => {
            let retention_path = relative_artifact_path([
                "docs",
                "test-runs",
                "core-engine",
                "treecalc-local",
                W037_TREECALC_RUN_ID,
                "retention_guardrail.json",
            ]);
            let retention = read_json(repo_root, &retention_path)?;
            let passed = array_contains_string(
                &retention,
                "claims_exercised",
                "R5.overlay_retention_release",
            ) && retention
                .pointer("/retention/evicted_overlay_count_after_release")
                .and_then(Value::as_u64)
                .unwrap_or_default()
                > 0
                && counter_value(&retention, "release_events") > 0;
            json!({
                "check_id": check_id,
                "status": if passed { "passed" } else { "failed" },
                "artifact_paths": [retention_path],
                "observed": {
                    "claims_exercised": retention["claims_exercised"].clone(),
                    "evicted_overlay_count_after_release": retention.pointer("/retention/evicted_overlay_count_after_release").cloned().unwrap_or(Value::Null),
                    "release_events": counter_value(&retention, "release_events"),
                },
            })
        }
        "w037_direct_oxfml_let_lambda" => {
            let lexical_path = w037_upstream_host_case_path(
                "uh_let_lambda_lexical_capture_eval_001",
                "result.json",
            );
            let returned_path = w037_upstream_host_case_path(
                "uh_returned_lambda_invocation_eval_001",
                "result.json",
            );
            let lexical = read_json(repo_root, &lexical_path)?;
            let returned = read_json(repo_root, &returned_path)?;
            let passed = number_at(upstream_host_summary, "let_lambda_case_count") >= 2
                && number_at(upstream_host_summary, "expectation_mismatch_count") == 0
                && string_at(&lexical, "status") == "matched"
                && string_at(&returned, "status") == "matched"
                && lexical
                    .pointer("/w037_interpretation/narrow_let_lambda_carrier")
                    .and_then(Value::as_bool)
                    .unwrap_or(false)
                && returned
                    .pointer("/w037_interpretation/narrow_let_lambda_carrier")
                    .and_then(Value::as_bool)
                    .unwrap_or(false);
            json!({
                "check_id": check_id,
                "status": if passed { "passed" } else { "failed" },
                "artifact_paths": [lexical_path, returned_path],
                "observed": {
                    "let_lambda_case_count": number_at(upstream_host_summary, "let_lambda_case_count"),
                    "expectation_mismatch_count": number_at(upstream_host_summary, "expectation_mismatch_count"),
                    "lexical_status": string_at(&lexical, "status"),
                    "returned_status": string_at(&returned, "status"),
                },
            })
        }
        "w037_direct_oxfml_returned_lambda" => {
            let returned_path = w037_upstream_host_case_path(
                "uh_returned_lambda_invocation_eval_001",
                "result.json",
            );
            let returned = read_json(repo_root, &returned_path)?;
            let passed = string_at(&returned, "status") == "matched"
                && returned
                    .pointer("/w037_interpretation/direct_oxfml_evaluator_reexecution")
                    .and_then(Value::as_bool)
                    .unwrap_or(false)
                && !returned
                    .pointer("/w037_interpretation/general_oxfunc_kernel_claimed")
                    .and_then(Value::as_bool)
                    .unwrap_or(true);
            json!({
                "check_id": check_id,
                "status": if passed { "passed" } else { "failed" },
                "artifact_paths": [returned_path],
                "observed": {
                    "status": string_at(&returned, "status"),
                    "value_payload": string_pointer(&returned, "/candidate_result/value_payload"),
                    "general_oxfunc_kernel_claimed": returned.pointer("/w037_interpretation/general_oxfunc_kernel_claimed").cloned().unwrap_or(Value::Null),
                },
            })
        }
        "w038_tracecalc_authority_discharge" => {
            let passed = number_at(
                tracecalc_authority_summary,
                "remaining_tracecalc_authority_blocker_count",
            ) == 0
                && number_at(
                    tracecalc_authority_summary,
                    "accepted_external_exclusion_count",
                ) == 1
                && !tracecalc_authority_summary
                    .get("general_oxfunc_kernel_promoted")
                    .and_then(Value::as_bool)
                    .unwrap_or(true);
            json!({
                "check_id": check_id,
                "status": if passed { "passed" } else { "failed" },
                "artifact_paths": [relative_artifact_path([
                    "docs",
                    "test-runs",
                    "core-engine",
                    "tracecalc-authority",
                    W038_TRACECALC_AUTHORITY_RUN_ID,
                    "run_summary.json",
                ])],
                "observed": {
                    "remaining_tracecalc_authority_blocker_count": number_at(tracecalc_authority_summary, "remaining_tracecalc_authority_blocker_count"),
                    "accepted_external_exclusion_count": number_at(tracecalc_authority_summary, "accepted_external_exclusion_count"),
                    "general_oxfunc_kernel_promoted": tracecalc_authority_summary["general_oxfunc_kernel_promoted"].clone(),
                },
            })
        }
        "w037_stage2_replay_blocker_named" => {
            let passed = !stage2_decision
                .get("stage2_policy_promoted")
                .and_then(Value::as_bool)
                .unwrap_or(true)
                && array_contains_string(
                    stage2_decision,
                    "blockers",
                    "stage2.deterministic_partition_replay_absent",
                );
            json!({
                "check_id": check_id,
                "status": if passed { "passed" } else { "failed" },
                "artifact_paths": [relative_artifact_path([
                    "docs",
                    "test-runs",
                    "core-engine",
                    "stage2-criteria",
                    W037_STAGE2_CRITERIA_RUN_ID,
                    "promotion_decision.json",
                ])],
                "observed": {
                    "stage2_policy_promoted": stage2_decision["stage2_policy_promoted"].clone(),
                    "blockers": stage2_decision["blockers"].clone(),
                },
            })
        }
        "w037_treecalc_capability_reject" => {
            let result_path =
                w037_treecalc_case_path("tc_local_capability_sensitive_reject_001", "result.json");
            let result = read_json(repo_root, &result_path)?;
            let passed = string_at(&result, "result_state") == "rejected"
                && string_pointer(&result, "/reject_detail/kind") == "HostInjectedFailure"
                && array_contains_string(
                    &result,
                    "diagnostics",
                    "dependency_diagnostic:residual:carrier:capability:host_function_availability",
                );
            json!({
                "check_id": check_id,
                "status": if passed { "passed" } else { "failed" },
                "artifact_paths": [result_path],
                "observed": {
                    "result_state": string_at(&result, "result_state"),
                    "reject_kind": string_pointer(&result, "/reject_detail/kind"),
                    "diagnostic_count": array_at(&result, "diagnostics").len(),
                },
            })
        }
        "w037_callable_treecalc_value_only" => {
            let result_path = w037_treecalc_case_path(
                "tc_local_w034_higher_order_let_lambda_publish_001",
                "result.json",
            );
            let result = read_json(repo_root, &result_path)?;
            let dependency_shape_update_count = result
                .pointer("/candidate_result/dependency_shape_updates")
                .and_then(Value::as_array)
                .map_or(0, Vec::len);
            let published_runtime_effect_count = result
                .pointer("/publication_bundle/published_runtime_effects")
                .and_then(Value::as_array)
                .map_or(0, Vec::len);
            let passed = string_at(&result, "result_state") == "published"
                && dependency_shape_update_count == 0
                && published_runtime_effect_count == 0;
            json!({
                "check_id": check_id,
                "status": if passed { "passed" } else { "failed" },
                "artifact_paths": [result_path],
                "observed": {
                    "result_state": string_at(&result, "result_state"),
                    "dependency_shape_update_count": dependency_shape_update_count,
                    "published_runtime_effect_count": published_runtime_effect_count,
                    "published_value_delta": result.pointer("/publication_bundle/published_view_delta").cloned().unwrap_or(Value::Null),
                },
            })
        }
        "w037_proof_callable_boundary" => {
            let row = array_at(formal_blockers, "rows").iter().find(|row| {
                string_at(row, "blocker_id") == "callable.general_oxfunc_kernel_not_promoted"
            });
            let passed = row
                .map(|row| string_at(row, "state") == "opaque_external_boundary")
                .unwrap_or(false);
            json!({
                "check_id": check_id,
                "status": if passed { "passed" } else { "failed" },
                "artifact_paths": [relative_artifact_path([
                    "docs",
                    "test-runs",
                    "core-engine",
                    "formal-inventory",
                    W037_FORMAL_INVENTORY_RUN_ID,
                    "promotion_blockers.json",
                ])],
                "observed": row.cloned().unwrap_or(Value::Null),
            })
        }
        _ => json!({
            "check_id": check_id,
            "status": "failed",
            "artifact_paths": [],
            "observed": {
                "failure": "unknown_w038_evidence_check",
            },
        }),
    };

    if number_at(treecalc_summary, "expectation_mismatch_count") != 0 {
        Ok(json!({
            "check_id": check_id,
            "status": "failed",
            "artifact_paths": check["artifact_paths"].clone(),
            "observed": check["observed"].clone(),
            "run_level_failure": "w037_treecalc_expectation_mismatch_count_nonzero",
        }))
    } else {
        Ok(check)
    }
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

fn w038_validation_failures(
    w037_residual_register: &Value,
    w037_match_guard: &Value,
    w037_treecalc_summary: &Value,
    upstream_host_summary: &Value,
    tracecalc_authority_summary: &Value,
    failed_row_count: usize,
    match_promoted_count: usize,
    accepted_boundary_count: usize,
    exact_remaining_blocker_count: usize,
) -> Vec<String> {
    let mut failures = Vec::new();
    if number_at(w037_residual_register, "residual_blocker_count") != 5 {
        failures.push("w037_residual_blocker_count_changed".to_string());
    }
    if number_at(w037_match_guard, "promoted_match_count") != 1 {
        failures.push("w037_prior_match_guard_changed".to_string());
    }
    if number_at(w037_treecalc_summary, "expectation_mismatch_count") != 0 {
        failures.push("w037_treecalc_expectation_mismatch_count_nonzero".to_string());
    }
    if number_at(upstream_host_summary, "let_lambda_case_count") < 2 {
        failures.push("w037_upstream_host_let_lambda_case_count_too_low".to_string());
    }
    if number_at(upstream_host_summary, "expectation_mismatch_count") != 0 {
        failures.push("w037_upstream_host_expectation_mismatch_count_nonzero".to_string());
    }
    if number_at(
        tracecalc_authority_summary,
        "remaining_tracecalc_authority_blocker_count",
    ) != 0
    {
        failures.push("w038_tracecalc_authority_blockers_remaining".to_string());
    }
    if failed_row_count != 0 {
        failures.push("w038_disposition_row_failures_present".to_string());
    }
    if match_promoted_count != 0 {
        failures.push("w038_declared_gap_match_promotion_present".to_string());
    }
    if accepted_boundary_count != 1 {
        failures.push("w038_expected_one_accepted_boundary".to_string());
    }
    if exact_remaining_blocker_count != 4 {
        failures.push("w038_expected_four_exact_remaining_blockers".to_string());
    }
    failures
}

fn w039_validation_failures(
    w039_ledger: &Value,
    w039_formatting_intake: &Value,
    w038_summary: &Value,
    w038_disposition_register: &Value,
    w038_blocker_register: &Value,
    w038_match_guard: &Value,
    failed_row_count: usize,
    match_promoted_count: usize,
    exact_remaining_blocker_count: usize,
) -> Vec<String> {
    let mut failures = Vec::new();
    if number_at(w039_ledger, "obligation_count") != 20 {
        failures.push("w039_successor_obligation_count_changed".to_string());
    }
    if array_at(w039_formatting_intake, "typed_only_families").len() != 7 {
        failures.push("w039_w073_typed_only_family_count_changed".to_string());
    }
    if !string_at(w039_formatting_intake, "thresholds_rule").contains("intentionally ignored") {
        failures.push("w039_w073_threshold_fallback_guard_missing".to_string());
    }
    if number_at(w038_summary, "w038_disposition_row_count") != 5 {
        failures.push("w038_disposition_row_count_changed".to_string());
    }
    if number_at(w038_summary, "w038_exact_remaining_blocker_count") != 4 {
        failures.push("w038_summary_exact_blocker_count_changed".to_string());
    }
    if number_at(w038_disposition_register, "disposition_row_count") != 5 {
        failures.push("w038_disposition_register_row_count_changed".to_string());
    }
    if number_at(w038_blocker_register, "exact_remaining_blocker_count") != 4 {
        failures.push("w038_blocker_register_count_changed".to_string());
    }
    if number_at(w038_match_guard, "promoted_match_count") != 0 {
        failures.push("w038_match_guard_promoted_matches_present".to_string());
    }
    if failed_row_count != 0 {
        failures.push("w039_disposition_row_failures_present".to_string());
    }
    if match_promoted_count != 0 {
        failures.push("w039_declared_gap_match_promotion_present".to_string());
    }
    if exact_remaining_blocker_count != 4 {
        failures.push("w039_expected_four_exact_remaining_blockers".to_string());
    }
    failures
}

fn w044_dynamic_transition_failures(
    mixed_seeds: &Value,
    mixed_result: &Value,
    mixed_closure: &Value,
    treecalc_summary: &Value,
) -> Vec<String> {
    let mut failures = Vec::new();
    let seed_reasons = array_at_top(mixed_seeds)
        .iter()
        .map(|seed| string_at(seed, "reason"))
        .collect::<Vec<_>>();
    for expected_reason in [
        "DependencyAdded",
        "DependencyRemoved",
        "DependencyReclassified",
    ] {
        if !seed_reasons.iter().any(|reason| reason == expected_reason) {
            failures.push(format!(
                "mixed_dynamic_seed_reason_missing:{expected_reason}"
            ));
        }
    }
    if seed_reasons.len() != 3 {
        failures.push("mixed_dynamic_seed_reason_count_changed".to_string());
    }
    if string_at(mixed_result, "result_state") != "rejected" {
        failures.push("mixed_dynamic_post_edit_not_rejected".to_string());
    }
    if string_pointer(mixed_result, "/reject_detail/kind") != "HostInjectedFailure" {
        failures.push("mixed_dynamic_post_edit_reject_kind_changed".to_string());
    }
    if number_at(treecalc_summary, "case_count") != 28 {
        failures.push("w044_treecalc_case_count_changed".to_string());
    }
    if number_at(treecalc_summary, "expectation_mismatch_count") != 0 {
        failures.push("w044_treecalc_expectation_mismatch_count_nonzero".to_string());
    }

    let closure_record = array_at_top(mixed_closure)
        .iter()
        .find(|record| number_at(record, "node_id") == 3);
    if let Some(record) = closure_record {
        if !bool_at(record, "requires_rebind") {
            failures.push("mixed_dynamic_closure_rebind_not_required".to_string());
        }
        let closure_reasons = array_at(record, "reasons")
            .iter()
            .filter_map(Value::as_str)
            .collect::<Vec<_>>();
        for expected_reason in [
            "DependencyAdded",
            "DependencyRemoved",
            "DependencyReclassified",
        ] {
            if !closure_reasons.contains(&expected_reason) {
                failures.push(format!(
                    "mixed_dynamic_closure_reason_missing:{expected_reason}"
                ));
            }
        }
    } else {
        failures.push("mixed_dynamic_closure_record_for_owner_missing".to_string());
    }

    failures
}

fn w044_validation_failures(
    w044_summary: &Value,
    w044_blocker_map: &Value,
    w043_summary: &Value,
    w043_counterpart: &Value,
    w043_blocker_register: &Value,
    w043_callable_register: &Value,
    w043_match_guard: &Value,
    treecalc_summary: &Value,
    dynamic_evidence: &Value,
    failed_row_count: usize,
    match_promoted_count: usize,
    exact_remaining_blocker_count: usize,
) -> Vec<String> {
    let mut failures = Vec::new();
    if number_at(w044_summary, "source_residual_lane_count") != 20 {
        failures.push("w044_source_residual_lane_count_changed".to_string());
    }
    if number_at(w044_summary, "obligation_count") != 45 {
        failures.push("w044_obligation_count_changed".to_string());
    }
    if number_at(w044_blocker_map, "source_residual_lane_count") != 20 {
        failures.push("w044_blocker_map_lane_count_changed".to_string());
    }
    if array_at(w044_blocker_map, "rows").len() != 20 {
        failures.push("w044_blocker_map_row_count_changed".to_string());
    }
    if !bool_at(w044_summary, "oxfml_formatting_update_incorporated") {
        failures.push("w044_oxfml_formatting_update_not_incorporated".to_string());
    }
    if bool_at(
        w044_summary,
        "w073_downstream_request_construction_uptake_verified",
    ) {
        failures.push("w044_w073_downstream_request_unexpectedly_verified".to_string());
    }
    if number_at(w043_summary, "exact_remaining_blocker_count") != 3 {
        failures.push("w043_summary_exact_blocker_count_changed".to_string());
    }
    if number_at(w043_summary, "match_promoted_count") != 0 {
        failures.push("w043_summary_match_promoted_count_changed".to_string());
    }
    if number_at(w043_summary, "failed_row_count") != 0 {
        failures.push("w043_summary_failed_rows_present".to_string());
    }
    if number_at(w043_counterpart, "row_count") != 8 {
        failures.push("w043_counterpart_row_count_changed".to_string());
    }
    if number_at(w043_blocker_register, "exact_remaining_blocker_count") != 3 {
        failures.push("w043_blocker_register_count_changed".to_string());
    }
    if bool_at(
        w043_callable_register,
        "callable_metadata_projection_promoted",
    ) {
        failures.push("w043_callable_metadata_projection_promoted".to_string());
    }
    if number_at(w043_match_guard, "match_promoted_count") != 0 {
        failures.push("w043_match_guard_promoted_matches_present".to_string());
    }
    if number_at(treecalc_summary, "case_count") != 28 {
        failures.push("w044_treecalc_case_count_changed".to_string());
    }
    if number_at(treecalc_summary, "expectation_mismatch_count") != 0 {
        failures.push("w044_treecalc_expectation_mismatch_count_nonzero".to_string());
    }
    if !bool_at(dynamic_evidence, "direct_evidence_bound") {
        failures.push("w044_dynamic_transition_direct_evidence_failed".to_string());
    }
    if !array_at(dynamic_evidence, "failures").is_empty() {
        failures.push("w044_dynamic_transition_evidence_failures_present".to_string());
    }
    if failed_row_count != 0 {
        failures.push("w044_disposition_row_failures_present".to_string());
    }
    if match_promoted_count != 0 {
        failures.push("w044_declared_gap_match_promotion_present".to_string());
    }
    if exact_remaining_blocker_count != 4 {
        failures.push("w044_expected_four_exact_remaining_blockers".to_string());
    }
    failures
}

#[allow(clippy::too_many_arguments)]
fn w045_validation_failures(
    w045_summary: &Value,
    w045_obligation_map: &Value,
    w045_oxfml_intake: &Value,
    w044_summary: &Value,
    w044_disposition: &Value,
    w044_blocker_register: &Value,
    w044_dynamic: &Value,
    w044_callable: &Value,
    w044_match_guard: &Value,
    treecalc_summary: &Value,
    failed_row_count: usize,
    match_promoted_count: usize,
    exact_remaining_blocker_count: usize,
) -> Vec<String> {
    let mut failures = Vec::new();
    if number_at(w045_summary, "source_residual_lane_count") != 22 {
        failures.push("w045_source_residual_lane_count_changed".to_string());
    }
    if number_at(w045_summary, "successor_obligation_count") != 36 {
        failures.push("w045_successor_obligation_count_changed".to_string());
    }
    if number_at(w045_summary, "promotion_contract_count") != 18 {
        failures.push("w045_promotion_contract_count_changed".to_string());
    }
    if number_at(w045_obligation_map, "source_residual_lane_count") != 22 {
        failures.push("w045_obligation_map_lane_count_changed".to_string());
    }
    if number_at(w045_obligation_map, "successor_obligation_count") != 36 {
        failures.push("w045_obligation_map_obligation_count_changed".to_string());
    }
    if array_at(w045_obligation_map, "obligations").len() != 36 {
        failures.push("w045_obligation_map_obligation_rows_changed".to_string());
    }
    if !bool_at(w045_summary, "oxfml_formatting_update_incorporated") {
        failures.push("w045_oxfml_formatting_update_not_incorporated".to_string());
    }
    if bool_at(
        w045_summary,
        "w073_downstream_request_construction_uptake_verified_by_oxcalc",
    ) {
        failures.push("w045_w073_downstream_request_unexpectedly_verified".to_string());
    }
    if bool_at(
        w045_oxfml_intake,
        "w073_downstream_typed_rule_request_construction_verified",
    ) {
        failures.push("w045_oxfml_intake_unexpected_downstream_verification".to_string());
    }
    if number_at(w044_summary, "w044_disposition_row_count") != 6 {
        failures.push("w044_summary_disposition_row_count_changed".to_string());
    }
    if number_at(w044_summary, "w044_direct_evidence_bound_count") != 2 {
        failures.push("w044_summary_direct_evidence_count_changed".to_string());
    }
    if number_at(w044_summary, "w044_exact_remaining_blocker_count") != 4 {
        failures.push("w044_summary_exact_blocker_count_changed".to_string());
    }
    if number_at(w044_summary, "w044_match_promoted_count") != 0 {
        failures.push("w044_summary_match_promoted_count_changed".to_string());
    }
    if number_at(w044_summary, "failed_row_count") != 0 {
        failures.push("w044_summary_failed_rows_present".to_string());
    }
    if number_at(w044_disposition, "disposition_row_count") != 6 {
        failures.push("w044_disposition_register_row_count_changed".to_string());
    }
    if number_at(w044_blocker_register, "exact_remaining_blocker_count") != 4 {
        failures.push("w044_blocker_register_count_changed".to_string());
    }
    if !bool_at(w044_dynamic, "direct_evidence_bound") {
        failures.push("w044_dynamic_transition_direct_evidence_not_bound".to_string());
    }
    if !array_at(w044_dynamic, "failures").is_empty() {
        failures.push("w044_dynamic_transition_evidence_failures_present".to_string());
    }
    if bool_at(w044_callable, "callable_metadata_projection_promoted") {
        failures.push("w044_callable_metadata_projection_promoted".to_string());
    }
    if number_at(w044_match_guard, "promoted_match_count") != 0 {
        failures.push("w044_match_guard_promoted_matches_present".to_string());
    }
    if number_at(treecalc_summary, "case_count") != 28 {
        failures.push("w044_treecalc_case_count_changed".to_string());
    }
    if number_at(treecalc_summary, "expectation_mismatch_count") != 0 {
        failures.push("w044_treecalc_expectation_mismatch_count_nonzero".to_string());
    }
    if failed_row_count != 0 {
        failures.push("w045_disposition_row_failures_present".to_string());
    }
    if match_promoted_count != 0 {
        failures.push("w045_declared_gap_match_promotion_present".to_string());
    }
    if exact_remaining_blocker_count != 5 {
        failures.push("w045_expected_five_exact_remaining_blockers".to_string());
    }
    failures
}

fn rows_by_id(document: &Value, key: &str) -> BTreeMap<String, Value> {
    array_at(document, "rows")
        .iter()
        .filter_map(|row| Some((row.get(key)?.as_str()?.to_string(), row.clone())))
        .collect()
}

fn rows_by_id_from_array(document: &Value, array_key: &str, key: &str) -> BTreeMap<String, Value> {
    array_at(document, array_key)
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

fn array_at_top(value: &Value) -> &[Value] {
    value.as_array().map_or(&[], Vec::as_slice)
}

fn array_contains_string(value: &Value, key: &str, expected: &str) -> bool {
    array_at(value, key)
        .iter()
        .any(|item| item.as_str() == Some(expected))
}

fn counter_value(value: &Value, counter_id: &str) -> i64 {
    array_at(value, "counters")
        .iter()
        .find(|entry| string_at(entry, "counter") == counter_id)
        .and_then(|entry| entry.get("value"))
        .and_then(Value::as_i64)
        .unwrap_or_default()
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

fn bool_at(value: &Value, key: &str) -> bool {
    value.get(key).and_then(Value::as_bool).unwrap_or(false)
}

fn relative_artifact_path<'a>(segments: impl IntoIterator<Item = &'a str>) -> String {
    segments
        .into_iter()
        .filter(|segment| !segment.trim().is_empty())
        .map(|segment| segment.replace('\\', "/").trim_matches('/').to_string())
        .collect::<Vec<_>>()
        .join("/")
}

fn w037_treecalc_case_path(case_id: &str, leaf: &str) -> String {
    relative_artifact_path([
        "docs",
        "test-runs",
        "core-engine",
        "treecalc-local",
        W037_TREECALC_RUN_ID,
        "cases",
        case_id,
        leaf,
    ])
}

fn w037_upstream_host_case_path(case_id: &str, leaf: &str) -> String {
    relative_artifact_path([
        "docs",
        "test-runs",
        "core-engine",
        "upstream-host",
        W037_UPSTREAM_HOST_RUN_ID,
        "cases",
        case_id,
        leaf,
    ])
}

fn w044_treecalc_case_path(case_id: &str, leaf: &str) -> String {
    relative_artifact_path([
        "docs",
        "test-runs",
        "core-engine",
        "treecalc-local",
        W044_TREECALC_RUN_ID,
        "cases",
        case_id,
        leaf,
    ])
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

const W038_DISPOSITION_SPECS: &[W038DispositionSpec] = &[
    W038DispositionSpec {
        row_id: "w038_disposition_dynamic_negative_release_reclassification",
        source_w037_residual_row_id: "w037_decision_dynamic_negative_release_residual_blocker",
        w038_obligation_id: "W038-OBL-003",
        w038_disposition_kind: "partial_direct_evidence_exact_remaining_blocker",
        w038_disposition: "bind dynamic negative reject, resolved dynamic publish, and retention-release guardrail evidence while carrying release/reclassification differential as an exact remaining blocker",
        conformance_match_state: "not_promoted",
        implementation_evidence_state: "dynamic_negative_and_retention_guardrails_bound_without_release_reclassification_match",
        direct_evidence_bound: true,
        accepted_boundary: false,
        exact_remaining_blocker_bead: Some("calc-zsr.9"),
        authority_owner: "calc-zsr.3; calc-zsr.9",
        promotion_consequence: "the dynamic negative/release row is narrowed but not promoted; W038 still lacks an optimized differential that proves dependency release and reclassification against the TraceCalc release row.",
        reason: "TreeCalc-local now supplies direct evidence for dynamic-potential rejection, resolved dynamic publication, and retained dynamic overlay release, but those artifacts do not constitute the missing release/reclassification differential.",
        implementation_evidence_sources: &[
            "docs/test-runs/core-engine/treecalc-local/w037-optimized-core-conformance-treecalc-001/cases/tc_local_dynamic_reject_001/result.json",
            "docs/test-runs/core-engine/treecalc-local/w037-optimized-core-conformance-treecalc-001/cases/tc_local_dynamic_resolved_publish_001/result.json",
            "docs/test-runs/core-engine/treecalc-local/w037-optimized-core-conformance-treecalc-001/retention_guardrail.json",
        ],
        required_evidence_checks: &[
            "w037_treecalc_dynamic_negative_reject",
            "w037_treecalc_dynamic_resolved_publish",
            "w037_treecalc_dynamic_retention_release",
        ],
    },
    W038DispositionSpec {
        row_id: "w038_disposition_lambda_host_effect_direct_oxfml_boundary_accepted",
        source_w037_residual_row_id: "w037_decision_lambda_host_effect_residual_blocker",
        w038_obligation_id: "W038-OBL-004",
        w038_disposition_kind: "accepted_boundary_after_direct_oxfml_evidence",
        w038_disposition: "replace the stale direct-OxFml-absence blocker with an accepted narrow LET/LAMBDA carrier boundary and external OxFunc-kernel exclusion",
        conformance_match_state: "not_promoted",
        implementation_evidence_state: "direct_oxfml_let_lambda_evidence_bound_with_general_oxfunc_kernel_excluded",
        direct_evidence_bound: true,
        accepted_boundary: true,
        exact_remaining_blocker_bead: None,
        authority_owner: "calc-zsr.3; calc-zsr.7; external:OxFunc",
        promotion_consequence: "the direct OxFml absence no longer blocks the exercised LET/LAMBDA carrier slice, but OxCalc still makes no general OxFunc LAMBDA-kernel or optimized TreeCalc value-only match claim.",
        reason: "W037 direct upstream-host evidence exercises lexical LET/LAMBDA capture and returned-lambda invocation through OxFml; W038 TraceCalc authority accepts the general OxFunc kernel row as external owner scope.",
        implementation_evidence_sources: &[
            "docs/test-runs/core-engine/upstream-host/w037-direct-oxfml-evaluator-001/run_summary.json",
            "docs/test-runs/core-engine/upstream-host/w037-direct-oxfml-evaluator-001/cases/uh_let_lambda_lexical_capture_eval_001/result.json",
            "docs/test-runs/core-engine/upstream-host/w037-direct-oxfml-evaluator-001/cases/uh_returned_lambda_invocation_eval_001/result.json",
            "docs/test-runs/core-engine/tracecalc-authority/w038-tracecalc-authority-discharge-001/run_summary.json",
        ],
        required_evidence_checks: &[
            "w037_direct_oxfml_let_lambda",
            "w038_tracecalc_authority_discharge",
        ],
    },
    W038DispositionSpec {
        row_id: "w038_disposition_snapshot_fence_projection_exact_blocker",
        source_w037_residual_row_id: "w037_decision_snapshot_fence_projection_residual_blocker",
        w038_obligation_id: "W038-OBL-005",
        w038_disposition_kind: "exact_remaining_blocker",
        w038_disposition: "carry snapshot-fence projection as an exact Stage 2/coordinator replay blocker",
        conformance_match_state: "not_promoted",
        implementation_evidence_state: "tracecalc_replay_present_without_optimized_stale_candidate_counterpart",
        direct_evidence_bound: false,
        accepted_boundary: false,
        exact_remaining_blocker_bead: Some("calc-zsr.5"),
        authority_owner: "calc-zsr.3; calc-zsr.5",
        promotion_consequence: "snapshot-fence conformance and Stage 2 promotion remain blocked until deterministic coordinator or partition replay supplies an optimized counterpart.",
        reason: "W037 Stage 2 criteria explicitly names deterministic partition replay absence; TreeCalc-local still has no stale accepted-candidate admission fence counterpart.",
        implementation_evidence_sources: &[
            "docs/test-runs/core-engine/stage2-criteria/w037-stage2-deterministic-replay-criteria-001/promotion_decision.json",
        ],
        required_evidence_checks: &["w037_stage2_replay_blocker_named"],
    },
    W038DispositionSpec {
        row_id: "w038_disposition_capability_view_fence_projection_exact_blocker",
        source_w037_residual_row_id: "w037_decision_capability_view_fence_projection_residual_blocker",
        w038_obligation_id: "W038-OBL-006",
        w038_disposition_kind: "exact_remaining_blocker",
        w038_disposition: "carry compatibility-fenced capability-view projection as an exact Stage 2/coordinator replay blocker",
        conformance_match_state: "not_promoted",
        implementation_evidence_state: "broader_treecalc_capability_reject_present_without_compatibility_fence_counterpart",
        direct_evidence_bound: false,
        accepted_boundary: false,
        exact_remaining_blocker_bead: Some("calc-zsr.5"),
        authority_owner: "calc-zsr.3; calc-zsr.5",
        promotion_consequence: "capability-view fence conformance remains blocked until deterministic coordinator or partition replay exercises the compatibility-view fence mismatch directly.",
        reason: "TreeCalc-local retains broader capability-sensitive reject evidence, but W038 still cannot count that as the TraceCalc compatibility-fenced capability-view mismatch.",
        implementation_evidence_sources: &[
            "docs/test-runs/core-engine/treecalc-local/w037-optimized-core-conformance-treecalc-001/cases/tc_local_capability_sensitive_reject_001/result.json",
            "docs/test-runs/core-engine/stage2-criteria/w037-stage2-deterministic-replay-criteria-001/promotion_decision.json",
        ],
        required_evidence_checks: &[
            "w037_treecalc_capability_reject",
            "w037_stage2_replay_blocker_named",
        ],
    },
    W038DispositionSpec {
        row_id: "w038_disposition_callable_metadata_projection_exact_blocker",
        source_w037_residual_row_id: "w037_decision_callable_metadata_projection_residual_blocker",
        w038_obligation_id: "W038-OBL-007",
        w038_disposition_kind: "exact_remaining_blocker",
        w038_disposition: "carry callable metadata projection as an exact proof/seam blocker while preserving value-only TreeCalc and direct OxFml callable-carrier evidence",
        conformance_match_state: "not_promoted",
        implementation_evidence_state: "treecalc_value_only_counterpart_plus_direct_oxfml_returned_lambda_without_metadata_projection",
        direct_evidence_bound: true,
        accepted_boundary: false,
        exact_remaining_blocker_bead: Some("calc-zsr.4; calc-zsr.7"),
        authority_owner: "calc-zsr.3; calc-zsr.4; calc-zsr.7; external:OxFunc",
        promotion_consequence: "callable metadata projection remains blocked until callable carrier sufficiency is proven or a concrete metadata projection fixture exists.",
        reason: "The optimized TreeCalc counterpart publishes the ordinary higher-order value only; direct OxFml exercises the narrow returned-lambda carrier, and W037 proof inventory keeps the general OxFunc callable kernel external.",
        implementation_evidence_sources: &[
            "docs/test-runs/core-engine/treecalc-local/w037-optimized-core-conformance-treecalc-001/cases/tc_local_w034_higher_order_let_lambda_publish_001/result.json",
            "docs/test-runs/core-engine/upstream-host/w037-direct-oxfml-evaluator-001/cases/uh_returned_lambda_invocation_eval_001/result.json",
            "docs/test-runs/core-engine/formal-inventory/w037-proof-model-closure-001/promotion_blockers.json",
            "docs/test-runs/core-engine/tracecalc-authority/w038-tracecalc-authority-discharge-001/run_summary.json",
        ],
        required_evidence_checks: &[
            "w037_callable_treecalc_value_only",
            "w037_direct_oxfml_returned_lambda",
            "w037_proof_callable_boundary",
            "w038_tracecalc_authority_discharge",
        ],
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

    #[test]
    fn implementation_conformance_runner_classifies_w038_dispositions() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let run_id = format!(
            "test-w038-implementation-conformance-{}",
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

        assert_eq!(summary.gap_disposition_row_count, 5);
        assert_eq!(summary.w038_disposition_row_count, 5);
        assert_eq!(summary.w038_direct_evidence_bound_count, 3);
        assert_eq!(summary.w038_accepted_boundary_count, 1);
        assert_eq!(summary.w038_exact_remaining_blocker_count, 4);
        assert_eq!(summary.w038_match_promoted_count, 0);
        assert_eq!(summary.validated_row_count, 5);
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
            "implementation_conformance_w038_dispositions_valid"
        );

        let guard = read_json(
            &repo_root,
            &format!(
                "docs/test-runs/core-engine/implementation-conformance/{run_id}/w038_match_promotion_guard.json"
            ),
        )
        .unwrap();
        assert_eq!(guard["promoted_match_count"], 0);
        assert_eq!(
            guard["guard_status"],
            "w038_declared_gap_promotion_guard_holds"
        );

        cleanup();
    }

    #[test]
    fn implementation_conformance_runner_classifies_w039_exact_blockers() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let run_id = format!(
            "test-w039-implementation-conformance-{}",
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

        assert_eq!(summary.gap_disposition_row_count, 5);
        assert_eq!(summary.w039_disposition_row_count, 5);
        assert_eq!(summary.w039_direct_evidence_bound_count, 2);
        assert_eq!(summary.w039_exact_remaining_blocker_count, 4);
        assert_eq!(summary.w039_match_promoted_count, 0);
        assert_eq!(summary.validated_row_count, 5);
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
            "implementation_conformance_w039_exact_blockers_valid"
        );

        let guard = read_json(
            &repo_root,
            &format!(
                "docs/test-runs/core-engine/implementation-conformance/{run_id}/w039_match_promotion_guard.json"
            ),
        )
        .unwrap();
        assert_eq!(guard["promoted_match_count"], 0);
        assert_eq!(
            guard["guard_status"],
            "w039_declared_gap_promotion_guard_holds"
        );

        cleanup();
    }

    #[test]
    fn implementation_conformance_runner_classifies_w044_dynamic_callable_tranche() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let run_id = format!(
            "test-w044-implementation-conformance-{}",
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
        assert_eq!(summary.w044_disposition_row_count, 6);
        assert_eq!(summary.w044_direct_evidence_bound_count, 2);
        assert_eq!(summary.w044_exact_remaining_blocker_count, 4);
        assert_eq!(summary.w044_match_promoted_count, 0);
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
            "implementation_conformance_w044_dynamic_transition_callable_metadata_valid"
        );

        let guard = read_json(
            &repo_root,
            &format!(
                "docs/test-runs/core-engine/implementation-conformance/{run_id}/w044_match_promotion_guard.json"
            ),
        )
        .unwrap();
        assert_eq!(guard["promoted_match_count"], 0);
        assert_eq!(
            guard["guard_status"],
            "w044_declared_gap_promotion_guard_holds"
        );

        cleanup();
    }

    #[test]
    fn implementation_conformance_runner_classifies_w045_optimized_core_tranche() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let run_id = format!(
            "test-w045-implementation-conformance-{}",
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

        assert_eq!(summary.gap_disposition_row_count, 7);
        assert_eq!(summary.w045_disposition_row_count, 7);
        assert_eq!(summary.w045_direct_evidence_bound_count, 2);
        assert_eq!(summary.w045_exact_remaining_blocker_count, 5);
        assert_eq!(summary.w045_match_promoted_count, 0);
        assert_eq!(summary.validated_row_count, 7);
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
            "implementation_conformance_w045_optimized_core_counterpart_callable_metadata_valid"
        );

        let guard = read_json(
            &repo_root,
            &format!(
                "docs/test-runs/core-engine/implementation-conformance/{run_id}/w045_match_promotion_guard.json"
            ),
        )
        .unwrap();
        assert_eq!(guard["promoted_match_count"], 0);
        assert_eq!(
            guard["guard_status"],
            "w045_declared_gap_promotion_guard_holds"
        );

        cleanup();
    }
}
