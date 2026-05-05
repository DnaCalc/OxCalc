#![forbid(unsafe_code)]

//! W035-W037 TraceCalc oracle matrix packet emission.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use serde_json::{Value, json};
use thiserror::Error;

use crate::runner::{TraceCalcRunner, TraceCalcRunnerError};

const ORACLE_MATRIX_RUN_SUMMARY_SCHEMA_V1: &str = "oxcalc.tracecalc.oracle_matrix.summary.v1";
const ORACLE_MATRIX_COVERAGE_SCHEMA_V1: &str = "oxcalc.tracecalc.oracle_matrix.coverage.v1";
const ORACLE_MATRIX_UNCOVERED_SCHEMA_V1: &str = "oxcalc.tracecalc.oracle_matrix.uncovered.v1";
const ORACLE_MATRIX_EXCLUDED_SCHEMA_V1: &str = "oxcalc.tracecalc.oracle_matrix.excluded.v1";
const ORACLE_MATRIX_VALIDATION_SCHEMA_V1: &str = "oxcalc.tracecalc.oracle_matrix.validation.v1";
const ORACLE_MATRIX_CLOSURE_CRITERIA_SCHEMA_V1: &str =
    "oxcalc.tracecalc.oracle_matrix.closure_criteria.v1";
const ORACLE_MATRIX_NO_LOSS_SCHEMA_V1: &str = "oxcalc.tracecalc.oracle_matrix.no_loss_crosswalk.v1";
const W035_ORACLE_MATRIX_RUN_ID: &str = "w035-tracecalc-oracle-matrix-001";

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
    pub excluded_row_count: usize,
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
    excluded: bool,
    missing_or_failed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MatrixProfile {
    W035,
    W036CoverageClosure,
    W037ObservableClosure,
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
        let profile = MatrixProfile::for_run_id(run_id);
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

        let row_specs = row_specs_for(profile);
        let rows = row_specs
            .iter()
            .map(|spec| evaluate_row(repo_root, &relative_artifact_root, spec, profile))
            .collect::<Result<Vec<_>, _>>()?;
        let covered_row_count = rows.iter().filter(|row| row.covered).count();
        let uncovered_row_count = rows.iter().filter(|row| row.uncovered).count();
        let excluded_row_count = rows.iter().filter(|row| row.excluded).count();
        let missing_or_failed_row_count = rows.iter().filter(|row| row.missing_or_failed).count();
        let row_values = rows.iter().map(|row| row.row.clone()).collect::<Vec<_>>();
        let uncovered_rows = rows
            .iter()
            .filter(|row| row.uncovered || row.missing_or_failed)
            .map(|row| row.row.clone())
            .collect::<Vec<_>>();
        let excluded_rows = rows
            .iter()
            .filter(|row| row.excluded)
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
                "excluded_row_count": excluded_row_count,
                "missing_or_failed_row_count": missing_or_failed_row_count,
                "full_oracle_claim": false,
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
            &matrix_root.join("excluded_surface_register.json"),
            &json!({
                "schema_version": ORACLE_MATRIX_EXCLUDED_SCHEMA_V1,
                "run_id": run_id,
                "excluded_row_count": excluded_row_count,
                "rows": excluded_rows,
            }),
        )?;

        let mut validation_status = if missing_or_failed_row_count == 0 {
            "matrix_valid"
        } else {
            "matrix_has_failed_or_missing_rows"
        };
        let no_loss_crosswalk = if matches!(
            profile,
            MatrixProfile::W036CoverageClosure | MatrixProfile::W037ObservableClosure
        ) {
            let crosswalk = no_loss_crosswalk_json(run_id, &rows, profile);
            if number_at(&crosswalk, "missing_crosswalk_count") != 0 {
                validation_status = "matrix_has_missing_no_loss_crosswalk_rows";
            }
            write_json(&matrix_root.join("no_loss_crosswalk.json"), &crosswalk)?;
            write_json(
                &matrix_root.join("coverage_closure_criteria.json"),
                &coverage_closure_criteria_json(
                    run_id,
                    profile,
                    trace_summary.scenario_count,
                    rows.len(),
                    covered_row_count,
                    uncovered_row_count,
                    excluded_row_count,
                    missing_or_failed_row_count,
                ),
            )?;
            Some(crosswalk)
        } else {
            None
        };

        write_json(
            &matrix_root.join("validation.json"),
            &json!({
                "schema_version": ORACLE_MATRIX_VALIDATION_SCHEMA_V1,
                "run_id": run_id,
                "status": validation_status,
                "tracecalc_scenario_count": trace_summary.scenario_count,
                "matrix_row_count": rows.len(),
                "covered_row_count": covered_row_count,
                "classified_uncovered_row_count": uncovered_row_count,
                "excluded_row_count": excluded_row_count,
                "missing_or_failed_row_count": missing_or_failed_row_count,
                "missing_crosswalk_count": no_loss_crosswalk
                    .as_ref()
                    .map_or(0, |crosswalk| number_at(crosswalk, "missing_crosswalk_count")),
            }),
        )?;

        let summary = TraceCalcOracleMatrixRunSummary {
            run_id: run_id.to_string(),
            schema_version: ORACLE_MATRIX_RUN_SUMMARY_SCHEMA_V1.to_string(),
            tracecalc_scenario_count: trace_summary.scenario_count,
            matrix_row_count: rows.len(),
            covered_row_count,
            uncovered_row_count,
            excluded_row_count,
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
                "excluded_row_count": summary.excluded_row_count,
                "missing_or_failed_row_count": summary.missing_or_failed_row_count,
                "artifact_root": summary.artifact_root,
                "coverage_matrix_path": format!("{relative_artifact_root}/oracle-matrix/coverage_matrix.json"),
                "uncovered_surface_register_path": format!("{relative_artifact_root}/oracle-matrix/uncovered_surface_register.json"),
                "excluded_surface_register_path": format!("{relative_artifact_root}/oracle-matrix/excluded_surface_register.json"),
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
    profile: MatrixProfile,
) -> Result<MatrixRowEvaluation, TraceCalcOracleMatrixError> {
    let Some(scenario_id) = spec.scenario_id else {
        let excluded = matches!(
            profile,
            MatrixProfile::W036CoverageClosure | MatrixProfile::W037ObservableClosure
        ) && spec.classification.starts_with("classified_out_of_scope");
        let evidence_state = if excluded {
            "excluded_by_authority"
        } else {
            "classified_uncovered_deferred"
        };
        return Ok(MatrixRowEvaluation {
            row: row_json(spec, evidence_state, &[], &[], None, None, profile),
            covered: false,
            uncovered: !excluded,
            excluded,
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
            profile,
        ),
        covered: failures.is_empty(),
        uncovered: false,
        excluded: false,
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
    profile: MatrixProfile,
) -> Value {
    let coverage_class = coverage_class(spec, evidence_state, profile);
    json!({
        "row_id": spec.row_id,
        "obligation_id": spec.obligation_id,
        "matrix_family": spec.family,
        "surface": spec.surface,
        "scenario_id": scenario_id,
        "evidence_state": evidence_state,
        "coverage_class": coverage_class,
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

fn number_at(value: &Value, key: &str) -> usize {
    value
        .get(key)
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
        .unwrap_or_default()
}

fn coverage_class(
    spec: &MatrixRowSpec,
    evidence_state: &str,
    profile: MatrixProfile,
) -> &'static str {
    if evidence_state == "covered_passed" {
        "covered"
    } else if matches!(
        profile,
        MatrixProfile::W036CoverageClosure | MatrixProfile::W037ObservableClosure
    ) && spec.classification.starts_with("classified_out_of_scope")
    {
        "excluded"
    } else if evidence_state == "classified_uncovered_deferred" {
        "uncovered"
    } else {
        "missing_or_failed"
    }
}

impl MatrixProfile {
    fn for_run_id(run_id: &str) -> Self {
        if run_id.contains("w037") {
            Self::W037ObservableClosure
        } else if run_id.contains("w036") {
            Self::W036CoverageClosure
        } else {
            Self::W035
        }
    }
}

fn row_specs_for(profile: MatrixProfile) -> Vec<MatrixRowSpec> {
    let mut rows = MATRIX_ROWS.to_vec();
    if matches!(
        profile,
        MatrixProfile::W036CoverageClosure | MatrixProfile::W037ObservableClosure
    ) {
        for row in &mut rows {
            row.obligation_id = "W036-OBL-001";
            row.owner = "calc-rqq.2";
        }
        for row in &mut rows {
            match row.row_id {
                "w035_overlay_multi_reader_release_order" => {
                    row.obligation_id = "W036-OBL-002";
                    row.classification = "classified_uncovered_tla_stage2_owner";
                    row.owner = "calc-rqq.5";
                    row.reason = "TraceCalc remains single-threaded and does not model full multi-reader interleavings; W036 TLA Stage 2 partition and scheduler-equivalence work owns this row.";
                }
                "w035_callable_full_oxfunc_semantics" => {
                    row.obligation_id = "W036-OBL-002";
                    row.owner = "external:OxFunc; calc-rqq.4 records boundary";
                    row.reason = "W036 keeps the OxCalc/OxFml LET/LAMBDA carrier fragment in scope and excludes the general OxFunc LAMBDA semantic kernel from TraceCalc oracle coverage.";
                }
                _ => {}
            }
        }
        rows.extend_from_slice(W036_EXTENSION_ROWS);
    }
    if profile == MatrixProfile::W037ObservableClosure {
        for row in &mut rows {
            row.obligation_id = "W037-OBL-001";
            row.owner = "calc-ubd.1";
        }
        for row in &mut rows {
            match row.row_id {
                "w035_overlay_multi_reader_release_order" => {
                    row.scenario_id = Some("tc_w037_overlay_multi_reader_release_order_001");
                    row.required_labels = &[
                        "reader_pinned",
                        "overlay_retained",
                        "overlay_release_deferred_for_remaining_readers",
                        "overlay_released",
                    ];
                    row.classification = "covered_by_w037_tracecalc_replay";
                    row.reason = "W037 adds deterministic TraceCalc replay evidence for two pinned readers: the first unpin defers overlay release while another reader remains pinned, and the final unpin opens eviction eligibility and releases the overlay.";
                }
                "w035_callable_full_oxfunc_semantics" => {
                    row.obligation_id = "W037-OBL-006";
                    row.owner = "external:OxFunc; calc-ubd.4/calc-ubd.5 record boundary";
                    row.reason = "W037 keeps the OxCalc/OxFml LET/LAMBDA carrier fragment in scope and excludes the general OxFunc LAMBDA semantic kernel from TraceCalc oracle coverage.";
                }
                _ => {}
            }
        }
    }
    rows
}

fn no_loss_crosswalk_json(
    run_id: &str,
    rows: &[MatrixRowEvaluation],
    profile: MatrixProfile,
) -> Value {
    let w035_rows = MATRIX_ROWS
        .iter()
        .map(|source| {
            let retained = rows.iter().find(|row| row.row["row_id"] == source.row_id);
            json!({
                "source_run_id": W035_ORACLE_MATRIX_RUN_ID,
                "source_row_id": source.row_id,
                "target_run_id": run_id,
                "target_row_id": retained.map_or(Value::Null, |row| row.row["row_id"].clone()),
                "relation": retained.map_or("missing", |_| "retained"),
                "target_coverage_class": retained
                    .map_or(Value::Null, |row| row.row["coverage_class"].clone()),
            })
        })
        .collect::<Vec<_>>();

    let scenario_rows = W033_W035_SCENARIO_CROSSWALK
        .iter()
        .map(|(scenario_id, source_workset)| {
            let targets = rows
                .iter()
                .filter(|row| row.row["scenario_id"] == *scenario_id)
                .map(|row| {
                    json!({
                        "target_row_id": row.row["row_id"],
                        "coverage_class": row.row["coverage_class"],
                        "evidence_state": row.row["evidence_state"],
                    })
                })
                .collect::<Vec<_>>();
            json!({
                "source_workset": source_workset,
                "source_scenario_id": scenario_id,
                "target_run_id": run_id,
                "relation": if targets.is_empty() {
                    "missing"
                } else {
                    "covered_or_classified"
                },
                "targets": targets,
            })
        })
        .collect::<Vec<_>>();

    let missing_w035_rows = w035_rows
        .iter()
        .filter(|row| row["relation"] == "missing")
        .count();
    let missing_scenarios = scenario_rows
        .iter()
        .filter(|row| row["relation"] == "missing")
        .count();

    let relation = match profile {
        MatrixProfile::W037ObservableClosure => {
            "w037 retains every W035 matrix row identity, maps every W033-W035 tagged corpus scenario, and adds direct TraceCalc replay for the W035 multi-reader overlay release-order row"
        }
        MatrixProfile::W036CoverageClosure => {
            "w036 retains every W035 matrix row identity and maps every W033-W035 tagged corpus scenario to at least one W036 row"
        }
        MatrixProfile::W035 => "w035 baseline matrix run",
    };

    json!({
        "schema_version": ORACLE_MATRIX_NO_LOSS_SCHEMA_V1,
        "run_id": run_id,
        "source_w035_matrix_run_id": W035_ORACLE_MATRIX_RUN_ID,
        "relation": relation,
        "w035_matrix_row_count": MATRIX_ROWS.len(),
        "w035_matrix_rows_retained_count": MATRIX_ROWS.len() - missing_w035_rows,
        "w033_w035_scenario_count": W033_W035_SCENARIO_CROSSWALK.len(),
        "w033_w035_scenarios_mapped_count": W033_W035_SCENARIO_CROSSWALK.len() - missing_scenarios,
        "missing_crosswalk_count": missing_w035_rows + missing_scenarios,
        "w035_matrix_rows": w035_rows,
        "w033_w035_scenarios": scenario_rows,
    })
}

fn coverage_closure_criteria_json(
    run_id: &str,
    profile: MatrixProfile,
    tracecalc_scenario_count: usize,
    matrix_row_count: usize,
    covered_row_count: usize,
    uncovered_row_count: usize,
    excluded_row_count: usize,
    missing_or_failed_row_count: usize,
) -> Value {
    let (closure_state, no_loss_criterion, promotion_blockers, semantic_equivalence_statement) =
        match profile {
            MatrixProfile::W037ObservableClosure => (
                "observable_rows_covered_no_full_oracle_claim",
                "Every W035 matrix row identity is retained, every W033-W035 tagged corpus scenario maps to a W037 row, and the prior multi-reader overlay release-order row has direct TraceCalc replay evidence.",
                vec![
                    "tracecalc.general_oxfunc_lambda_kernel_excluded_from_oxcalc_tracecalc_profile",
                    "tracecalc.optimized_core_engine_conformance_closure_not_yet_reached",
                    "tracecalc.direct_oxfml_evaluator_reexecution_not_yet_exercised",
                    "tracecalc.lean_tla_and_stage2_partition_work_remains_open",
                    "tracecalc.independent_evaluator_diversity_and_operated_assurance_lanes_remain_open",
                ],
                "W037 changes only the spec-purpose TraceCalc reference-machine replay profile for multi-reader overlay release ordering. It does not change production TreeCalc/CoreEngine runtime behavior, coordinator scheduling, invalidation, dependency graph construction, soft-reference resolution, recalc semantics, publication semantics, reject policy, pack decisions, continuous-assurance runners, or OxFml/OxFunc evaluator behavior.",
            ),
            MatrixProfile::W036CoverageClosure | MatrixProfile::W035 => (
                "criteria_defined_no_full_oracle_claim",
                "Every W035 matrix row identity is retained, and every W033-W035 tagged corpus scenario maps to a W036 row.",
                vec![
                    "tracecalc.multi_reader_overlay_release_order_deferred_to_tla_stage2",
                    "tracecalc.general_oxfunc_lambda_kernel_excluded_from_oxcalc_tracecalc_profile",
                    "tracecalc.optimized_core_engine_conformance_closure_not_yet_reached",
                    "tracecalc.lean_tla_assumption_and_stage2_partition_work_remains_open",
                    "tracecalc.independent_evaluator_diversity_and_continuous_assurance_lanes_remain_open",
                ],
                "This runner emits evidence classification artifacts only. It does not change coordinator scheduling, invalidation, dependency graph construction, soft-reference resolution, recalc semantics, publication semantics, reject policy, TraceCalc execution semantics, TreeCalc/CoreEngine behavior, Lean/TLA models, pack decisions, continuous-assurance runners, or OxFml/OxFunc evaluator behavior.",
            ),
        };

    json!({
        "schema_version": ORACLE_MATRIX_CLOSURE_CRITERIA_SCHEMA_V1,
        "run_id": run_id,
        "closure_state": closure_state,
        "full_oracle_claim": false,
        "matrix_row_count": matrix_row_count,
        "covered_row_count": covered_row_count,
        "classified_uncovered_row_count": uncovered_row_count,
        "excluded_row_count": excluded_row_count,
        "missing_or_failed_row_count": missing_or_failed_row_count,
        "current_tracecalc_corpus_scenario_count": tracecalc_scenario_count,
        "criteria": {
            "covered_row": "A row is covered only when its scenario result passed, validation/assertion/conformance mismatch arrays are empty, and every required trace label is present.",
            "uncovered_row": "A row is uncovered when the surface is relevant to future core-engine verification but has no deterministic TraceCalc scenario in this profile.",
            "excluded_row": "A row is excluded only when authority belongs outside this TraceCalc profile, such as the general OxFunc semantic kernel.",
            "no_loss": no_loss_criterion,
            "full_oracle_promotion": "A full TraceCalc oracle claim requires zero missing/failed rows, zero uncovered rows, no core-engine-owned excluded rows, optimized/core-engine conformance closure, and discharge or explicit non-oracle ownership for Lean/TLA obligations."
        },
        "promotion_blockers": promotion_blockers,
        "semantic_equivalence_statement": semantic_equivalence_statement
    })
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

const W036_EXTENSION_ROWS: &[MatrixRowSpec] = &[
    MatrixRowSpec {
        row_id: "w036_accept_publish_boundary",
        obligation_id: "W036-OBL-001",
        family: "candidate_publication_boundary",
        surface: "accepted candidate emits and publishes an ordinary value",
        scenario_id: Some("tc_accept_publish_001"),
        required_labels: &[
            "candidate_admitted",
            "candidate_emitted",
            "candidate_published",
        ],
        classification: "covered_by_w036_closure_matrix",
        owner: "calc-rqq.2",
        reason: "W036 closure criteria include the original accept/publish Stage 1 seed row.",
    },
    MatrixRowSpec {
        row_id: "w036_reject_no_publish_boundary",
        obligation_id: "W036-OBL-001",
        family: "reject_no_publish",
        surface: "typed reject leaves publication unchanged",
        scenario_id: Some("tc_reject_no_publish_001"),
        required_labels: &["candidate_admitted", "candidate_rejected"],
        classification: "covered_by_w036_closure_matrix",
        owner: "calc-rqq.2",
        reason: "W036 closure criteria retain the original reject/no-publish semantic seed row.",
    },
    MatrixRowSpec {
        row_id: "w036_pinned_view_stability",
        obligation_id: "W036-OBL-001",
        family: "pinned_view",
        surface: "pinned reader observes stable published view",
        scenario_id: Some("tc_pinned_view_stability_001"),
        required_labels: &["reader_pinned", "candidate_published"],
        classification: "covered_by_w036_closure_matrix",
        owner: "calc-rqq.2",
        reason: "Pinned-view stability remains part of the current observable TraceCalc corpus.",
    },
    MatrixRowSpec {
        row_id: "w036_dynamic_dependency_switch_seed",
        obligation_id: "W036-OBL-001",
        family: "dependency_update",
        surface: "dynamic dependency switch seed publishes with dependency-shape evidence",
        scenario_id: Some("tc_dynamic_dep_switch_001"),
        required_labels: &["candidate_shape_update_produced", "candidate_published"],
        classification: "covered_by_w036_closure_matrix",
        owner: "calc-rqq.2",
        reason: "The original dynamic dependency switch seed is retained beside the W035 widened rows.",
    },
    MatrixRowSpec {
        row_id: "w036_overlay_retention_seed",
        obligation_id: "W036-OBL-001",
        family: "overlay_retention",
        surface: "overlay retention seed publishes with retained overlay evidence",
        scenario_id: Some("tc_overlay_retention_001"),
        required_labels: &["overlay_retained", "candidate_published"],
        classification: "covered_by_w036_closure_matrix",
        owner: "calc-rqq.2",
        reason: "The original overlay retention seed remains a current observable TraceCalc row.",
    },
    MatrixRowSpec {
        row_id: "w036_scale_chain_seed",
        obligation_id: "W036-OBL-001",
        family: "scale_seed",
        surface: "scale-chain seed remains replayable as a semantic fixture",
        scenario_id: Some("tc_scale_chain_seed_001"),
        required_labels: &["candidate_published"],
        classification: "covered_by_w036_closure_matrix",
        owner: "calc-rqq.2",
        reason: "The scale seed is covered as a replayable semantic fixture, not as performance promotion evidence.",
    },
    MatrixRowSpec {
        row_id: "w036_verify_clean_no_publish",
        obligation_id: "W036-OBL-001",
        family: "verify_clean",
        surface: "clean verification produces no publication",
        scenario_id: Some("tc_verify_clean_no_publish_001"),
        required_labels: &["node_verified_clean"],
        classification: "covered_by_w036_closure_matrix",
        owner: "calc-rqq.2",
        reason: "Verify-clean/no-publish behavior remains part of the current observable TraceCalc corpus.",
    },
    MatrixRowSpec {
        row_id: "w036_multinode_dag_publish",
        obligation_id: "W036-OBL-001",
        family: "topological_scheduling",
        surface: "multi-node DAG publishes after topological scheduling",
        scenario_id: Some("tc_multinode_dag_publish_001"),
        required_labels: &["topo_group_scheduled", "candidate_published"],
        classification: "covered_by_w036_closure_matrix",
        owner: "calc-rqq.2",
        reason: "The W014 DAG scheduling seed is retained in the W036 current-corpus matrix.",
    },
    MatrixRowSpec {
        row_id: "w036_publication_fence_reject",
        obligation_id: "W036-OBL-001",
        family: "reject_no_publish",
        surface: "publication-fence reject remains no-publish",
        scenario_id: Some("tc_publication_fence_reject_001"),
        required_labels: &["candidate_admitted", "candidate_rejected"],
        classification: "covered_by_w036_closure_matrix",
        owner: "calc-rqq.2",
        reason: "The W014 publication-fence reject seed is retained by the W036 no-loss profile.",
    },
    MatrixRowSpec {
        row_id: "w036_artifact_token_reject",
        obligation_id: "W036-OBL-001",
        family: "reject_no_publish",
        surface: "artifact-token reject remains no-publish",
        scenario_id: Some("tc_artifact_token_reject_001"),
        required_labels: &["candidate_admitted", "candidate_rejected"],
        classification: "covered_by_w036_closure_matrix",
        owner: "calc-rqq.2",
        reason: "The artifact-token reject seed is retained by the W036 no-loss profile.",
    },
    MatrixRowSpec {
        row_id: "w036_fallback_reentry_overlay_reuse",
        obligation_id: "W036-OBL-001",
        family: "fallback_reentry",
        surface: "fallback re-entry can reject then publish through compatible overlay reuse",
        scenario_id: Some("tc_fallback_reentry_001"),
        required_labels: &[
            "fallback_reentered",
            "candidate_rejected",
            "candidate_published",
        ],
        classification: "covered_by_w036_closure_matrix",
        owner: "calc-rqq.2",
        reason: "Fallback re-entry remains covered as an observable transition family in TraceCalc.",
    },
    MatrixRowSpec {
        row_id: "w036_cycle_region_reject",
        obligation_id: "W036-OBL-001",
        family: "cycle_region",
        surface: "cycle region is detected and rejected",
        scenario_id: Some("tc_cycle_region_reject_001"),
        required_labels: &["cycle_region_detected", "candidate_rejected"],
        classification: "covered_by_w036_closure_matrix",
        owner: "calc-rqq.2",
        reason: "The cycle/SCC reject seed is retained in the current observable TraceCalc matrix.",
    },
    MatrixRowSpec {
        row_id: "w036_callable_invocation_reject",
        obligation_id: "W036-OBL-001",
        family: "let_lambda_callable",
        surface: "LET/LAMBDA invocation-contract mismatch rejects without publication",
        scenario_id: Some("tc_let_lambda_invocation_reject_001"),
        required_labels: &["candidate_admitted", "candidate_rejected"],
        classification: "covered_by_w033_seed",
        owner: "calc-rqq.2",
        reason: "The W033 invocation reject row remains part of the OxCalc/OxFml callable-carrier fragment.",
    },
    MatrixRowSpec {
        row_id: "w036_callable_runtime_effect_visibility",
        obligation_id: "W036-OBL-001",
        family: "let_lambda_callable",
        surface: "LET/LAMBDA callable invocation can carry runtime-effect visibility",
        scenario_id: Some("tc_let_lambda_runtime_effect_001"),
        required_labels: &["candidate_published"],
        classification: "covered_by_w033_seed",
        owner: "calc-rqq.2",
        reason: "The W033 runtime-effect row is retained without claiming the full OxFunc LAMBDA kernel.",
    },
    MatrixRowSpec {
        row_id: "w036_replay_equivalent_independent_order",
        obligation_id: "W036-OBL-001",
        family: "replay_equivalence",
        surface: "independent-order replay-equivalent history publishes deterministically",
        scenario_id: Some("tc_w034_replay_equivalent_independent_order_001"),
        required_labels: &["candidate_published"],
        classification: "covered_by_w034_seed",
        owner: "calc-rqq.2",
        reason: "The W034 replay-equivalent independent-order row is retained for W036 no-loss coverage.",
    },
];

const W033_W035_SCENARIO_CROSSWALK: &[(&str, &str)] = &[
    ("tc_let_lambda_carrier_publish_001", "W033"),
    ("tc_let_lambda_invocation_reject_001", "W033"),
    ("tc_let_lambda_runtime_effect_001", "W033"),
    ("tc_w034_snapshot_fence_reject_001", "W034"),
    ("tc_w034_capability_fence_reject_001", "W034"),
    ("tc_w034_dynamic_dependency_negative_001", "W034"),
    ("tc_w034_overlay_eviction_after_unpin_001", "W034"),
    ("tc_w034_let_lambda_higher_order_replay_001", "W034"),
    ("tc_w034_replay_equivalent_independent_order_001", "W034"),
    ("tc_w035_profile_fence_reject_001", "W035"),
    ("tc_w035_candidate_after_emit_fence_reject_001", "W035"),
    ("tc_w035_static_dependency_add_publish_001", "W035"),
    ("tc_w035_dynamic_dependency_switch_publish_001", "W035"),
    ("tc_w035_dynamic_dependency_release_publish_001", "W035"),
    (
        "tc_w035_dirty_seed_closure_no_under_invalidation_001",
        "W035",
    ),
    ("tc_w035_overlay_reuse_protected_dynamic_001", "W035"),
    ("tc_w035_let_lambda_defined_name_callable_001", "W035"),
    ("tc_w035_lambda_callable_publication_reject_001", "W035"),
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

        assert_eq!(summary.tracecalc_scenario_count, 31);
        assert_eq!(summary.matrix_row_count, MATRIX_ROWS.len());
        assert!(summary.covered_row_count >= 15);
        assert_eq!(summary.uncovered_row_count, 2);
        assert_eq!(summary.excluded_row_count, 0);
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

    #[test]
    fn oracle_matrix_runner_emits_w036_coverage_closure_artifacts() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let run_id = format!("test-w036-coverage-closure-{}", std::process::id());
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

        assert_eq!(summary.tracecalc_scenario_count, 31);
        assert_eq!(
            summary.matrix_row_count,
            MATRIX_ROWS.len() + W036_EXTENSION_ROWS.len()
        );
        assert_eq!(summary.covered_row_count, 30);
        assert_eq!(summary.uncovered_row_count, 1);
        assert_eq!(summary.excluded_row_count, 1);
        assert_eq!(summary.missing_or_failed_row_count, 0);

        let validation = read_json(
            &repo_root,
            &format!(
                "docs/test-runs/core-engine/tracecalc-reference-machine/{run_id}/oracle-matrix/validation.json"
            ),
        )
        .unwrap();
        assert_eq!(validation["status"], "matrix_valid");
        assert_eq!(validation["missing_crosswalk_count"], 0);

        let closure = read_json(
            &repo_root,
            &format!(
                "docs/test-runs/core-engine/tracecalc-reference-machine/{run_id}/oracle-matrix/coverage_closure_criteria.json"
            ),
        )
        .unwrap();
        assert_eq!(closure["full_oracle_claim"], false);
        assert_eq!(
            closure["closure_state"],
            "criteria_defined_no_full_oracle_claim"
        );
        assert!(
            closure["promotion_blockers"]
                .as_array()
                .unwrap()
                .iter()
                .any(|blocker| blocker
                    == "tracecalc.multi_reader_overlay_release_order_deferred_to_tla_stage2")
        );

        let crosswalk = read_json(
            &repo_root,
            &format!(
                "docs/test-runs/core-engine/tracecalc-reference-machine/{run_id}/oracle-matrix/no_loss_crosswalk.json"
            ),
        )
        .unwrap();
        assert_eq!(crosswalk["missing_crosswalk_count"], 0);
        assert_eq!(
            crosswalk["w035_matrix_rows_retained_count"],
            MATRIX_ROWS.len()
        );
        assert_eq!(
            crosswalk["w033_w035_scenarios_mapped_count"],
            W033_W035_SCENARIO_CROSSWALK.len()
        );
        assert!(
            crosswalk["relation"]
                .as_str()
                .unwrap()
                .starts_with("w036 retains")
        );

        cleanup();
    }

    #[test]
    fn oracle_matrix_runner_emits_w037_observable_closure_artifacts() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let run_id = format!("test-w037-observable-closure-{}", std::process::id());
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

        assert_eq!(summary.tracecalc_scenario_count, 31);
        assert_eq!(
            summary.matrix_row_count,
            MATRIX_ROWS.len() + W036_EXTENSION_ROWS.len()
        );
        assert_eq!(summary.covered_row_count, 31);
        assert_eq!(summary.uncovered_row_count, 0);
        assert_eq!(summary.excluded_row_count, 1);
        assert_eq!(summary.missing_or_failed_row_count, 0);

        let validation = read_json(
            &repo_root,
            &format!(
                "docs/test-runs/core-engine/tracecalc-reference-machine/{run_id}/oracle-matrix/validation.json"
            ),
        )
        .unwrap();
        assert_eq!(validation["status"], "matrix_valid");
        assert_eq!(validation["missing_crosswalk_count"], 0);

        let closure = read_json(
            &repo_root,
            &format!(
                "docs/test-runs/core-engine/tracecalc-reference-machine/{run_id}/oracle-matrix/coverage_closure_criteria.json"
            ),
        )
        .unwrap();
        assert_eq!(closure["full_oracle_claim"], false);
        assert_eq!(
            closure["closure_state"],
            "observable_rows_covered_no_full_oracle_claim"
        );
        assert!(
            closure["promotion_blockers"]
                .as_array()
                .unwrap()
                .iter()
                .all(|blocker| blocker
                    != "tracecalc.multi_reader_overlay_release_order_deferred_to_tla_stage2")
        );

        let matrix = read_json(
            &repo_root,
            &format!(
                "docs/test-runs/core-engine/tracecalc-reference-machine/{run_id}/oracle-matrix/coverage_matrix.json"
            ),
        )
        .unwrap();
        let rows = matrix["rows"].as_array().unwrap();
        let multi_reader = rows
            .iter()
            .find(|row| row["row_id"] == "w035_overlay_multi_reader_release_order")
            .unwrap();
        assert_eq!(multi_reader["coverage_class"], "covered");
        assert_eq!(
            multi_reader["scenario_id"],
            "tc_w037_overlay_multi_reader_release_order_001"
        );

        let crosswalk = read_json(
            &repo_root,
            &format!(
                "docs/test-runs/core-engine/tracecalc-reference-machine/{run_id}/oracle-matrix/no_loss_crosswalk.json"
            ),
        )
        .unwrap();
        assert_eq!(crosswalk["missing_crosswalk_count"], 0);
        assert!(
            crosswalk["relation"]
                .as_str()
                .unwrap()
                .starts_with("w037 retains")
        );

        cleanup();
    }
}
