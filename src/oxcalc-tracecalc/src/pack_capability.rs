#![forbid(unsafe_code)]

//! Post-W033 pack capability decision packet emission.

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
const OXFML_BRIDGE_RUN_ID: &str = "post-w033-direct-oxfml-fixture-bridge-001";
const LET_LAMBDA_TRACECALC_RUN_ID: &str = "post-w033-let-lambda-carrier-witness-001";
const LET_LAMBDA_TREECALC_RUN_ID: &str = "post-w033-let-lambda-treecalc-witness-001";
const INDEPENDENT_TREECALC_RUN_ID: &str = "post-w033-independent-conformance-treecalc-001";
const INDEPENDENT_CONFORMANCE_RUN_ID: &str = "post-w033-independent-conformance-001";

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

        let evaluation = evaluate_evidence(repo_root)?;
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
                "target_capability": "cap.C5.pack_valid",
                "decision_status": decision_status,
                "highest_honest_capability": highest_honest_capability,
                "capability_promoted": decision_status == "capability_promotion_candidate",
                "missing_artifact_count": evaluation.missing_paths.len(),
                "missing_paths": evaluation.missing_paths,
                "satisfied_inputs": evaluation.satisfied_inputs,
                "no_promotion_reason_ids": evaluation.blockers,
                "evidence_index_path": format!("{relative_artifact_root}/evidence/evidence_index.json"),
                "successor_lanes": [
                    "calc-rcr",
                    "calc-8lg",
                    "future_program_grade_pack_scope",
                    "future_continuous_cross_engine_diff_suite"
                ],
                "handoff_decision": {
                    "status": "handoff_not_required",
                    "reason_ids": [
                        "oxfml.seam.no_new_trigger_from_pack_governance_packet",
                        "oxfml.seam.watch_lanes_only"
                    ]
                }
            }),
        )?;

        let required_artifacts = required_artifacts(run_id);
        write_json(
            &artifact_root.join("replay-appliance/bundle_manifest.json"),
            &json!({
                "schema_version": PACK_CAPABILITY_BUNDLE_SCHEMA_V1,
                "run_id": run_id,
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

fn evaluate_evidence(repo_root: &Path) -> Result<EvidenceEvaluation, PackCapabilityError> {
    let mut evaluation = EvidenceEvaluation::default();
    evaluate_retained_decision(repo_root, &mut evaluation)?;
    evaluate_bridge(repo_root, &mut evaluation)?;
    evaluate_let_lambda(repo_root, &mut evaluation)?;
    evaluate_independent_conformance(repo_root, &mut evaluation)?;
    evaluate_treecalc_capability(repo_root, &mut evaluation)?;
    add_static_program_blockers(&mut evaluation);
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
    evaluation: &mut EvidenceEvaluation,
) -> Result<(), PackCapabilityError> {
    let summary_path = relative_artifact_path([
        "docs",
        "test-runs",
        "core-engine",
        "oxfml-fixture-bridge",
        OXFML_BRIDGE_RUN_ID,
        "run_summary.json",
    ]);
    let validation_path = relative_artifact_path([
        "docs",
        "test-runs",
        "core-engine",
        "oxfml-fixture-bridge",
        OXFML_BRIDGE_RUN_ID,
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
        reason_ids.push("pack.grade.direct_oxfml_evaluator_reexecution_absent".to_string());
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
    evaluation: &mut EvidenceEvaluation,
) -> Result<(), PackCapabilityError> {
    let trace_validation_path =
        bundle_validation_path("tracecalc-reference-machine", LET_LAMBDA_TRACECALC_RUN_ID);
    let tree_validation_path = bundle_validation_path("treecalc-local", LET_LAMBDA_TREECALC_RUN_ID);
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
    evaluation: &mut EvidenceEvaluation,
) -> Result<(), PackCapabilityError> {
    let summary_path = relative_artifact_path([
        "docs",
        "test-runs",
        "core-engine",
        "independent-conformance",
        INDEPENDENT_CONFORMANCE_RUN_ID,
        "run_summary.json",
    ]);
    let validation_path = relative_artifact_path([
        "docs",
        "test-runs",
        "core-engine",
        "independent-conformance",
        INDEPENDENT_CONFORMANCE_RUN_ID,
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
    evaluation: &mut EvidenceEvaluation,
) -> Result<(), PackCapabilityError> {
    let capability_path = relative_artifact_path([
        "docs",
        "test-runs",
        "core-engine",
        "treecalc-local",
        INDEPENDENT_TREECALC_RUN_ID,
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

fn add_static_program_blockers(evaluation: &mut EvidenceEvaluation) {
    let reason_ids = vec![
        "pack.grade.program_grade_replay_governance_not_reached".to_string(),
        "pack.grade.retained_witness_promotion_not_shared_program_grade".to_string(),
    ];
    evaluation.blockers.extend(reason_ids.clone());
    evaluation.rows.push(EvidenceRow {
        input_id: "post_w033_program_grade_governance",
        artifact_path: "docs/spec/core-engine/w033-formalization/W033_PACK_CAPABILITY_BINDING.md"
            .to_string(),
        evidence_state: "policy_blocker_retained".to_string(),
        observations: vec![
            "W033 and successor evidence widen local proof/replay/conformance but do not establish cap.C5.pack_valid.".to_string(),
        ],
        reason_ids,
    });
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

fn required_artifacts(run_id: &str) -> Vec<String> {
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
            OXFML_BRIDGE_RUN_ID,
            "run_summary.json",
        ]),
        relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "independent-conformance",
            INDEPENDENT_CONFORMANCE_RUN_ID,
            "run_summary.json",
        ]),
    ])
    .collect()
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

    fn unique_temp_repo() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let base = std::env::temp_dir().join(format!(
            "oxcalc-pack-capability-test-{}-{nanos}",
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
    }

    fn write_json_test(repo_root: &Path, relative_path: &str, value: Value) {
        let path = repo_root.join(relative_path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, serde_json::to_string_pretty(&value).unwrap() + "\n").unwrap();
    }

    fn read_required_json(repo_root: &Path, relative_path: &str) -> Value {
        serde_json::from_str(&fs::read_to_string(repo_root.join(relative_path)).unwrap()).unwrap()
    }
}
