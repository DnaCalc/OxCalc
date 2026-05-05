#![forbid(unsafe_code)]

//! W035 implementation-conformance hardening packet emission.

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

const W034_INDEPENDENT_CONFORMANCE_RUN_ID: &str = "w034-independent-conformance-001";
const W034_TREECALC_RUN_ID: &str = "w034-independent-conformance-treecalc-001";
const W035_ORACLE_MATRIX_RUN_ID: &str = "w035-tracecalc-oracle-matrix-001";

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
                "artifact_root": summary.artifact_root,
                "gap_disposition_register_path": format!("{relative_artifact_root}/gap_disposition_register.json"),
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
}
