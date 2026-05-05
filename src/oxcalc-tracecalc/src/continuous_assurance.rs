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

const W034_SCALE_BINDING_RUN_ID: &str = "w034-continuous-scale-gate-binding-001";
const W034_PACK_RUN_ID: &str = "w034-pack-capability-gate-binding-001";
const W035_ORACLE_MATRIX_RUN_ID: &str = "w035-tracecalc-oracle-matrix-001";
const W035_IMPLEMENTATION_CONFORMANCE_RUN_ID: &str =
    "w035-implementation-conformance-hardening-001";

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
        create_directory(&artifact_root.join("schedule"))?;
        create_directory(&artifact_root.join("replay-appliance"))?;
        create_directory(&artifact_root.join("replay-appliance/validation"))?;

        let evaluation = evaluate(repo_root)?;
        let schedule = continuous_assurance_schedule(run_id, &relative_artifact_root);
        let decision = decision_packet(run_id, &evaluation);

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

        let required_artifacts = required_artifacts(run_id);
        write_json(
            &artifact_root.join("replay-appliance/bundle_manifest.json"),
            &json!({
                "schema_version": BUNDLE_MANIFEST_SCHEMA_V1,
                "run_id": run_id,
                "artifact_root": relative_artifact_root,
                "claimed_capability": "continuous_assurance_gate_packet",
                "excluded_capabilities": [
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
                "artifact_root": summary.artifact_root,
                "source_evidence_index_path": format!("{relative_artifact_root}/evidence/source_evidence_index.json"),
                "schedule_path": format!("{relative_artifact_root}/schedule/continuous_assurance_schedule.json"),
                "cross_engine_differential_gate_path": format!("{relative_artifact_root}/differentials/cross_engine_differential_gate.json"),
                "decision_path": format!("{relative_artifact_root}/decision/continuous_assurance_decision.json"),
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
                "decision_status": summary.decision_status,
            }),
        )?;

        Ok(summary)
    }
}

fn evaluate(repo_root: &Path) -> Result<Evaluation, ContinuousAssuranceError> {
    let mut evaluation = Evaluation::default();

    evaluate_scale_binding(repo_root, &mut evaluation)?;
    evaluate_pack_binding(repo_root, &mut evaluation)?;
    evaluate_oracle_matrix(repo_root, &mut evaluation)?;
    evaluate_implementation_conformance(repo_root, &mut evaluation)?;
    evaluate_formal_artifacts(repo_root, &mut evaluation);

    evaluation.cross_engine_rows = cross_engine_rows();
    for row in &evaluation.cross_engine_rows {
        collect_failures(row, &mut evaluation.unexpected_mismatches);
    }

    evaluation.no_promotion_reasons = vec![
        "continuous.no_scheduled_regression_runner".to_string(),
        "continuous.no_cross_engine_diff_service".to_string(),
        "continuous.no_history_window_for_regression_thresholds".to_string(),
        "continuous.no_alerting_or_quarantine_policy".to_string(),
        "continuous.performance_not_correctness_proof".to_string(),
        "continuous.independent_evaluator_diversity_not_full".to_string(),
        "continuous.pack_c5_not_promoted".to_string(),
        "continuous.stage2_scheduler_not_promoted".to_string(),
        "continuous.formal_evidence_bounded_not_full_verification".to_string(),
    ];

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

fn continuous_assurance_schedule(run_id: &str, artifact_root: &str) -> Value {
    json!({
        "schema_version": SCHEDULE_SCHEMA_V1,
        "run_id": run_id,
        "artifact_root": artifact_root,
        "lanes": [
            {
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
            },
            {
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
            },
            {
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
            }
        ],
    })
}

fn cross_engine_rows() -> Vec<Value> {
    vec![
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
    ]
}

fn decision_packet(run_id: &str, evaluation: &Evaluation) -> Value {
    json!({
        "schema_version": DECISION_SCHEMA_V1,
        "run_id": run_id,
        "decision_status": if evaluation.missing_artifacts.is_empty() && evaluation.unexpected_mismatches.is_empty() {
            "continuous_assurance_gate_defined_without_promotion"
        } else {
            "continuous_assurance_gate_has_source_gaps"
        },
        "continuous_scale_assurance_promoted": false,
        "cross_engine_differential_service_promoted": false,
        "pack_capability_promoted": false,
        "stage2_scheduler_promoted": false,
        "performance_claim_promoted": false,
        "source_missing_artifact_count": evaluation.missing_artifacts.len(),
        "unexpected_mismatch_count": evaluation.unexpected_mismatches.len(),
        "no_promotion_reason_ids": evaluation.no_promotion_reasons,
        "semantic_equivalence_statement": "This runner reads existing W034/W035 evidence and emits continuous-assurance gate criteria only. It does not change coordinator scheduling, invalidation, recalc, publication, reject, TraceCalc, TreeCalc, or evaluator behavior.",
    })
}

fn required_artifacts(run_id: &str) -> Vec<String> {
    [
        "run_summary.json",
        "evidence/source_evidence_index.json",
        "schedule/continuous_assurance_schedule.json",
        "differentials/cross_engine_differential_gate.json",
        "decision/continuous_assurance_decision.json",
        "replay-appliance/bundle_manifest.json",
        "replay-appliance/validation/bundle_validation.json",
    ]
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
    .chain(source_artifacts())
    .collect()
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
    segments.into_iter().collect::<Vec<_>>().join("/")
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
