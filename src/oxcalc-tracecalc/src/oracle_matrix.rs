#![forbid(unsafe_code)]

//! W035 TraceCalc oracle matrix packet emission.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use serde_json::{Value, json};
use thiserror::Error;

use crate::runner::{TraceCalcRunner, TraceCalcRunnerError};

const ORACLE_MATRIX_RUN_SUMMARY_SCHEMA_V1: &str = "oxcalc.tracecalc.oracle_matrix.summary.v1";
const ORACLE_MATRIX_COVERAGE_SCHEMA_V1: &str = "oxcalc.tracecalc.oracle_matrix.coverage.v1";
const ORACLE_MATRIX_UNCOVERED_SCHEMA_V1: &str = "oxcalc.tracecalc.oracle_matrix.uncovered.v1";
const ORACLE_MATRIX_VALIDATION_SCHEMA_V1: &str = "oxcalc.tracecalc.oracle_matrix.validation.v1";

#[derive(Debug, Error)]
pub enum TraceCalcOracleMatrixError {
    #[error(transparent)]
    TraceCalcRun(#[from] TraceCalcRunnerError),
    #[error("failed to create artifact directory {path}: {source}")]
    CreateDirectory {
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
pub struct TraceCalcOracleMatrixRunSummary {
    pub run_id: String,
    pub schema_version: String,
    pub tracecalc_scenario_count: usize,
    pub matrix_row_count: usize,
    pub covered_row_count: usize,
    pub uncovered_row_count: usize,
    pub missing_or_failed_row_count: usize,
    pub artifact_root: String,
}

#[derive(Debug, Clone, Default)]
pub struct TraceCalcOracleMatrixRunner;

#[derive(Debug, Clone)]
struct MatrixRowSpec {
    row_id: &'static str,
    obligation_id: &'static str,
    family: &'static str,
    surface: &'static str,
    scenario_id: Option<&'static str>,
    required_labels: &'static [&'static str],
    classification: &'static str,
    owner: &'static str,
    reason: &'static str,
}

#[derive(Debug, Clone)]
struct MatrixRowEvaluation {
    row: Value,
    covered: bool,
    uncovered: bool,
    missing_or_failed: bool,
}

impl TraceCalcOracleMatrixRunner {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    pub fn execute(
        &self,
        repo_root: &Path,
        run_id: &str,
    ) -> Result<TraceCalcOracleMatrixRunSummary, TraceCalcOracleMatrixError> {
        let trace_summary =
            TraceCalcRunner::new().execute_manifest(repo_root, run_id, None, None)?;
        let artifact_root = repo_root.join(format!(
            "docs/test-runs/core-engine/tracecalc-reference-machine/{run_id}"
        ));
        let relative_artifact_root = relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "tracecalc-reference-machine",
            run_id,
        ]);
        let matrix_root = artifact_root.join("oracle-matrix");
        create_directory(&matrix_root)?;

        let rows = MATRIX_ROWS
            .iter()
            .map(|spec| evaluate_row(repo_root, &relative_artifact_root, spec))
            .collect::<Result<Vec<_>, _>>()?;
        let covered_row_count = rows.iter().filter(|row| row.covered).count();
        let uncovered_row_count = rows.iter().filter(|row| row.uncovered).count();
        let missing_or_failed_row_count = rows.iter().filter(|row| row.missing_or_failed).count();
        let row_values = rows.iter().map(|row| row.row.clone()).collect::<Vec<_>>();
        let uncovered_rows = rows
            .iter()
            .filter(|row| row.uncovered || row.missing_or_failed)
            .map(|row| row.row.clone())
            .collect::<Vec<_>>();

        write_json(
            &matrix_root.join("coverage_matrix.json"),
            &json!({
                "schema_version": ORACLE_MATRIX_COVERAGE_SCHEMA_V1,
                "run_id": run_id,
                "tracecalc_run_summary_path": format!("{relative_artifact_root}/run_summary.json"),
                "matrix_row_count": rows.len(),
                "covered_row_count": covered_row_count,
                "classified_uncovered_row_count": uncovered_row_count,
                "missing_or_failed_row_count": missing_or_failed_row_count,
                "rows": row_values,
            }),
        )?;

        write_json(
            &matrix_root.join("uncovered_surface_register.json"),
            &json!({
                "schema_version": ORACLE_MATRIX_UNCOVERED_SCHEMA_V1,
                "run_id": run_id,
                "classified_uncovered_row_count": uncovered_row_count,
                "missing_or_failed_row_count": missing_or_failed_row_count,
                "rows": uncovered_rows,
            }),
        )?;

        write_json(
            &matrix_root.join("validation.json"),
            &json!({
                "schema_version": ORACLE_MATRIX_VALIDATION_SCHEMA_V1,
                "run_id": run_id,
                "status": if missing_or_failed_row_count == 0 {
                    "matrix_valid"
                } else {
                    "matrix_has_failed_or_missing_rows"
                },
                "tracecalc_scenario_count": trace_summary.scenario_count,
                "matrix_row_count": rows.len(),
                "covered_row_count": covered_row_count,
                "classified_uncovered_row_count": uncovered_row_count,
                "missing_or_failed_row_count": missing_or_failed_row_count,
            }),
        )?;

        let summary = TraceCalcOracleMatrixRunSummary {
            run_id: run_id.to_string(),
            schema_version: ORACLE_MATRIX_RUN_SUMMARY_SCHEMA_V1.to_string(),
            tracecalc_scenario_count: trace_summary.scenario_count,
            matrix_row_count: rows.len(),
            covered_row_count,
            uncovered_row_count,
            missing_or_failed_row_count,
            artifact_root: relative_artifact_root.clone(),
        };
        write_json(
            &matrix_root.join("run_summary.json"),
            &json!({
                "schema_version": summary.schema_version,
                "run_id": summary.run_id,
                "tracecalc_scenario_count": summary.tracecalc_scenario_count,
                "matrix_row_count": summary.matrix_row_count,
                "covered_row_count": summary.covered_row_count,
                "classified_uncovered_row_count": summary.uncovered_row_count,
                "missing_or_failed_row_count": summary.missing_or_failed_row_count,
                "artifact_root": summary.artifact_root,
                "coverage_matrix_path": format!("{relative_artifact_root}/oracle-matrix/coverage_matrix.json"),
                "uncovered_surface_register_path": format!("{relative_artifact_root}/oracle-matrix/uncovered_surface_register.json"),
                "validation_path": format!("{relative_artifact_root}/oracle-matrix/validation.json"),
            }),
        )?;

        Ok(summary)
    }
}

fn evaluate_row(
    repo_root: &Path,
    relative_artifact_root: &str,
    spec: &MatrixRowSpec,
) -> Result<MatrixRowEvaluation, TraceCalcOracleMatrixError> {
    let Some(scenario_id) = spec.scenario_id else {
        return Ok(MatrixRowEvaluation {
            row: row_json(spec, "classified_uncovered_deferred", &[], &[], None, None),
            covered: false,
            uncovered: true,
            missing_or_failed: false,
        });
    };

    let result_path = relative_artifact_path([
        relative_artifact_root,
        "scenarios",
        scenario_id,
        "result.json",
    ]);
    let trace_path = relative_artifact_path([
        relative_artifact_root,
        "scenarios",
        scenario_id,
        "trace.json",
    ]);
    let result = read_json(repo_root, &result_path)?;
    let trace = read_json(repo_root, &trace_path)?;

    let mut failures = Vec::new();
    if string_at(&result, "result_state") != "passed" {
        failures.push("scenario_result_not_passed".to_string());
    }
    for array_name in [
        "validation_failures",
        "assertion_failures",
        "conformance_mismatches",
    ] {
        if !array_at(&result, array_name).is_empty() {
            failures.push(format!("{array_name}_not_empty"));
        }
    }

    let labels = trace_labels(&trace);
    for label in spec.required_labels {
        if !labels.contains(*label) {
            failures.push(format!("missing_required_trace_label:{label}"));
        }
    }

    let evidence_state = if failures.is_empty() {
        "covered_passed"
    } else {
        "covered_failed"
    };
    Ok(MatrixRowEvaluation {
        row: row_json(
            spec,
            evidence_state,
            &[result_path, trace_path],
            &failures,
            Some(scenario_id),
            Some(labels.into_iter().collect::<Vec<_>>()),
        ),
        covered: failures.is_empty(),
        uncovered: false,
        missing_or_failed: !failures.is_empty(),
    })
}

fn row_json(
    spec: &MatrixRowSpec,
    evidence_state: &str,
    artifact_paths: &[String],
    failures: &[String],
    scenario_id: Option<&str>,
    observed_trace_labels: Option<Vec<String>>,
) -> Value {
    json!({
        "row_id": spec.row_id,
        "obligation_id": spec.obligation_id,
        "matrix_family": spec.family,
        "surface": spec.surface,
        "scenario_id": scenario_id,
        "evidence_state": evidence_state,
        "classification": spec.classification,
        "owner": spec.owner,
        "reason": spec.reason,
        "required_trace_labels": spec.required_labels,
        "observed_trace_labels": observed_trace_labels.unwrap_or_default(),
        "artifact_paths": artifact_paths,
        "failures": failures,
    })
}

fn read_json(repo_root: &Path, relative_path: &str) -> Result<Value, TraceCalcOracleMatrixError> {
    let path = repo_root.join(relative_path);
    let content =
        fs::read_to_string(&path).map_err(|source| TraceCalcOracleMatrixError::ReadArtifact {
            path: path.display().to_string(),
            source,
        })?;
    serde_json::from_str(&content).map_err(|source| TraceCalcOracleMatrixError::ParseJson {
        path: path.display().to_string(),
        source,
    })
}

fn trace_labels(trace: &Value) -> BTreeSet<String> {
    trace
        .get("events")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|event| event.get("label").and_then(Value::as_str))
        .map(str::to_string)
        .collect()
}

fn string_at(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn array_at<'a>(value: &'a Value, key: &str) -> &'a [Value] {
    value
        .get(key)
        .and_then(Value::as_array)
        .map_or(&[], Vec::as_slice)
}

fn write_json(path: &Path, value: &Value) -> Result<(), TraceCalcOracleMatrixError> {
    let content = serde_json::to_string_pretty(value).expect("JSON serialization should succeed");
    fs::write(path, format!("{content}\n")).map_err(|source| {
        TraceCalcOracleMatrixError::WriteFile {
            path: path.display().to_string(),
            source,
        }
    })
}

fn create_directory(path: &Path) -> Result<(), TraceCalcOracleMatrixError> {
    fs::create_dir_all(path).map_err(|source| TraceCalcOracleMatrixError::CreateDirectory {
        path: path.display().to_string(),
        source,
    })
}

fn relative_artifact_path<'a>(segments: impl IntoIterator<Item = &'a str>) -> String {
    segments
        .into_iter()
        .filter(|segment| !segment.trim().is_empty())
        .map(|segment| segment.replace('\\', "/").trim_matches('/').to_string())
        .collect::<Vec<_>>()
        .join("/")
}

const MATRIX_ROWS: &[MatrixRowSpec] = &[
    MatrixRowSpec {
        row_id: "w035_stale_snapshot_fence_reject",
        obligation_id: "W035-OBL-001",
        family: "stale_fence",
        surface: "snapshot epoch mismatch rejects without publication",
        scenario_id: Some("tc_w034_snapshot_fence_reject_001"),
        required_labels: &["candidate_admitted", "candidate_rejected"],
        classification: "covered_by_w034_seed",
        owner: "calc-tkq.2",
        reason: "W034 snapshot-fence reject remains part of the W035 matrix floor.",
    },
    MatrixRowSpec {
        row_id: "w035_stale_capability_view_fence_reject",
        obligation_id: "W035-OBL-001",
        family: "stale_fence",
        surface: "capability-view mismatch rejects without publication",
        scenario_id: Some("tc_w034_capability_fence_reject_001"),
        required_labels: &["candidate_admitted", "candidate_rejected"],
        classification: "covered_by_w034_seed",
        owner: "calc-tkq.2",
        reason: "W034 capability-fence reject remains part of the W035 matrix floor.",
    },
    MatrixRowSpec {
        row_id: "w035_stale_profile_version_fence_reject",
        obligation_id: "W035-OBL-001",
        family: "stale_fence",
        surface: "profile-version mismatch rejects without publication",
        scenario_id: Some("tc_w035_profile_fence_reject_001"),
        required_labels: &["candidate_admitted", "candidate_rejected"],
        classification: "covered_by_w035_matrix",
        owner: "calc-tkq.2",
        reason: "W035 adds a profile-version fence row that was not in the W034 TraceCalc seed.",
    },
    MatrixRowSpec {
        row_id: "w035_stale_post_candidate_fence_reject",
        obligation_id: "W035-OBL-001",
        family: "stale_fence",
        surface: "accepted candidate can still be rejected before publication",
        scenario_id: Some("tc_w035_candidate_after_emit_fence_reject_001"),
        required_labels: &["candidate_emitted", "candidate_rejected"],
        classification: "covered_by_w035_matrix",
        owner: "calc-tkq.2",
        reason: "W035 adds a post-candidate no-publish reject row for candidate/publication separation.",
    },
    MatrixRowSpec {
        row_id: "w035_dependency_dynamic_negative",
        obligation_id: "W035-OBL-002",
        family: "dependency_update",
        surface: "unresolved dynamic dependency rejects without publication",
        scenario_id: Some("tc_w034_dynamic_dependency_negative_001"),
        required_labels: &["candidate_admitted", "candidate_rejected"],
        classification: "covered_by_w034_seed",
        owner: "calc-tkq.2",
        reason: "W034 dynamic negative row remains part of the W035 dependency-update floor.",
    },
    MatrixRowSpec {
        row_id: "w035_dependency_static_add_publish",
        obligation_id: "W035-OBL-002",
        family: "dependency_update",
        surface: "static dependency addition publishes with dependency-shape evidence",
        scenario_id: Some("tc_w035_static_dependency_add_publish_001"),
        required_labels: &["candidate_shape_update_produced", "candidate_published"],
        classification: "covered_by_w035_matrix",
        owner: "calc-tkq.2",
        reason: "W035 adds a positive static dependency-shape update row.",
    },
    MatrixRowSpec {
        row_id: "w035_dependency_dynamic_switch_publish",
        obligation_id: "W035-OBL-002",
        family: "dependency_update",
        surface: "dynamic dependency switch publishes with runtime-effect evidence",
        scenario_id: Some("tc_w035_dynamic_dependency_switch_publish_001"),
        required_labels: &["candidate_shape_update_produced", "candidate_published"],
        classification: "covered_by_w035_matrix",
        owner: "calc-tkq.2",
        reason: "W035 adds a positive dynamic dependency switch row.",
    },
    MatrixRowSpec {
        row_id: "w035_dependency_dynamic_release_publish",
        obligation_id: "W035-OBL-002",
        family: "dependency_update",
        surface: "dynamic dependency release publishes with runtime-effect evidence",
        scenario_id: Some("tc_w035_dynamic_dependency_release_publish_001"),
        required_labels: &["candidate_shape_update_produced", "candidate_published"],
        classification: "covered_by_w035_matrix",
        owner: "calc-tkq.2",
        reason: "W035 adds a dynamic dependency release/reclassification row.",
    },
    MatrixRowSpec {
        row_id: "w035_dependency_dirty_seed_closure",
        obligation_id: "W035-OBL-002",
        family: "dependency_update",
        surface: "dirty seed invalidates downstream closure without under-invalidation",
        scenario_id: Some("tc_w035_dirty_seed_closure_no_under_invalidation_001"),
        required_labels: &["node_marked_needed", "candidate_published"],
        classification: "covered_by_w035_matrix",
        owner: "calc-tkq.2",
        reason: "W035 adds a no-under-invalidation closure row for downstream dependents.",
    },
    MatrixRowSpec {
        row_id: "w035_overlay_retain_release",
        obligation_id: "W035-OBL-003",
        family: "overlay_retention",
        surface: "protected dynamic overlay retained while reader is pinned and released after unpin",
        scenario_id: Some("tc_w034_overlay_eviction_after_unpin_001"),
        required_labels: &[
            "overlay_retained",
            "eviction_eligibility_opened",
            "overlay_released",
        ],
        classification: "covered_by_w034_seed",
        owner: "calc-tkq.2",
        reason: "W034 overlay retain/release row remains part of the W035 matrix floor.",
    },
    MatrixRowSpec {
        row_id: "w035_overlay_reuse_protected_dynamic",
        obligation_id: "W035-OBL-003",
        family: "overlay_retention",
        surface: "protected dynamic overlay is reused on later compatible candidate emission",
        scenario_id: Some("tc_w035_overlay_reuse_protected_dynamic_001"),
        required_labels: &["overlay_retained", "candidate_shape_update_produced"],
        classification: "covered_by_w035_matrix",
        owner: "calc-tkq.2",
        reason: "W035 adds a retained-overlay reuse row.",
    },
    MatrixRowSpec {
        row_id: "w035_overlay_multi_reader_release_order",
        obligation_id: "W035-OBL-003",
        family: "overlay_retention",
        surface: "multi-reader overlay release ordering across independent pins",
        scenario_id: None,
        required_labels: &[],
        classification: "classified_uncovered_tla_owner",
        owner: "calc-tkq.5",
        reason: "TraceCalc remains single-threaded and does not model full multi-reader interleavings; W035 TLA non-routine exploration owns this row.",
    },
    MatrixRowSpec {
        row_id: "w035_callable_direct_capture_publish",
        obligation_id: "W035-OBL-004",
        family: "let_lambda_callable",
        surface: "LET/LAMBDA direct capture publishes ordinary value with callable identity",
        scenario_id: Some("tc_let_lambda_carrier_publish_001"),
        required_labels: &["candidate_shape_update_produced", "candidate_published"],
        classification: "covered_by_w033_seed",
        owner: "calc-tkq.2",
        reason: "W033 direct carrier row remains part of the W035 callable floor.",
    },
    MatrixRowSpec {
        row_id: "w035_callable_higher_order_publish",
        obligation_id: "W035-OBL-004",
        family: "let_lambda_callable",
        surface: "higher-order callable publishes ordinary value with replay-visible carrier identity",
        scenario_id: Some("tc_w034_let_lambda_higher_order_replay_001"),
        required_labels: &["candidate_shape_update_produced", "candidate_published"],
        classification: "covered_by_w034_seed",
        owner: "calc-tkq.2",
        reason: "W034 higher-order carrier row remains part of the W035 callable floor.",
    },
    MatrixRowSpec {
        row_id: "w035_callable_defined_name_publish",
        obligation_id: "W035-OBL-004",
        family: "let_lambda_callable",
        surface: "defined-name callable carrier publishes ordinary value with origin preservation",
        scenario_id: Some("tc_w035_let_lambda_defined_name_callable_001"),
        required_labels: &["candidate_shape_update_produced", "candidate_published"],
        classification: "covered_by_w035_matrix",
        owner: "calc-tkq.2",
        reason: "W035 adds a defined-name callable carrier row.",
    },
    MatrixRowSpec {
        row_id: "w035_callable_publication_reject",
        obligation_id: "W035-OBL-004",
        family: "let_lambda_callable",
        surface: "callable-as-value publication remains a typed reject/no-publish row",
        scenario_id: Some("tc_w035_lambda_callable_publication_reject_001"),
        required_labels: &["candidate_admitted", "candidate_rejected"],
        classification: "covered_by_w035_matrix",
        owner: "calc-tkq.2",
        reason: "W035 adds a callable-publication-policy reject row.",
    },
    MatrixRowSpec {
        row_id: "w035_callable_full_oxfunc_semantics",
        obligation_id: "W035-OBL-004",
        family: "let_lambda_callable",
        surface: "full OxFunc LAMBDA semantic kernel",
        scenario_id: None,
        required_labels: &[],
        classification: "classified_out_of_scope_oxfunc_kernel",
        owner: "calc-tkq.4",
        reason: "W035 formalizes the OxCalc/OxFml callable carrier fragment, not the general OxFunc semantic kernel.",
    },
];

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn oracle_matrix_runner_emits_valid_matrix_for_repo_corpus() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let run_id = format!("test-w035-oracle-matrix-{}", std::process::id());
        let artifact_root = repo_root.join(format!(
            "docs/test-runs/core-engine/tracecalc-reference-machine/{run_id}"
        ));
        let cleanup = || {
            if artifact_root.exists() {
                let _ = fs::remove_dir_all(&artifact_root);
            }
        };

        cleanup();
        let summary = TraceCalcOracleMatrixRunner::new()
            .execute(&repo_root, &run_id)
            .unwrap();

        assert_eq!(summary.tracecalc_scenario_count, 30);
        assert_eq!(summary.matrix_row_count, MATRIX_ROWS.len());
        assert!(summary.covered_row_count >= 15);
        assert_eq!(summary.uncovered_row_count, 2);
        assert_eq!(summary.missing_or_failed_row_count, 0);

        let validation = read_json(
            &repo_root,
            &format!(
                "docs/test-runs/core-engine/tracecalc-reference-machine/{run_id}/oracle-matrix/validation.json"
            ),
        )
        .unwrap();
        assert_eq!(validation["status"], "matrix_valid");

        cleanup();
    }
}
