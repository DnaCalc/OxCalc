#![forbid(unsafe_code)]

//! Deterministic runner for checked-in direct OxFml upstream-host fixtures.

use std::fs;
use std::path::Path;

use serde_json::{Value, json};
use thiserror::Error;

use crate::upstream_host_fixture::{
    UpstreamHostFixtureCase, UpstreamHostFixtureError, array_cell_data_bars,
    array_cell_effective_fill_colors, array_cell_icons, conditional_formatting_typed_rule_families,
    execute_fixture_case, fixture_expectation_mismatches, load_case, load_manifest,
    trace_function_ids, value_payload_summary,
};

const UPSTREAM_HOST_RUN_SUMMARY_SCHEMA_V1: &str = "oxcalc.upstream_host.direct_run_summary.v1";
const UPSTREAM_HOST_CASE_RESULT_SCHEMA_V1: &str = "oxcalc.upstream_host.direct_case_result.v1";
const UPSTREAM_HOST_CASE_INDEX_SCHEMA_V1: &str = "oxcalc.upstream_host.direct_case_index.v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpstreamHostRunSummary {
    pub run_id: String,
    pub artifact_root: String,
    pub fixture_case_count: usize,
    pub expectation_mismatch_count: usize,
    pub direct_oxfml_case_count: usize,
    pub let_lambda_case_count: usize,
    pub formatting_guard_case_count: usize,
    pub w073_typed_rule_case_count: usize,
}

#[derive(Debug, Error)]
pub enum UpstreamHostRunnerError {
    #[error("fixture error: {0}")]
    Fixture(#[from] UpstreamHostFixtureError),
    #[error("failed to create directory {path}: {source}")]
    CreateDir {
        path: String,
        source: std::io::Error,
    },
    #[error("failed to remove directory {path}: {source}")]
    RemoveDir {
        path: String,
        source: std::io::Error,
    },
    #[error("failed to write {path}: {source}")]
    Write {
        path: String,
        source: std::io::Error,
    },
}

#[derive(Debug, Default, Clone, Copy)]
pub struct UpstreamHostRunner;

impl UpstreamHostRunner {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    pub fn execute(
        &self,
        repo_root: &Path,
        run_id: &str,
    ) -> Result<UpstreamHostRunSummary, UpstreamHostRunnerError> {
        let manifest_path =
            repo_root.join("docs/test-fixtures/core-engine/upstream-host/MANIFEST.json");
        let manifest = load_manifest(&manifest_path)?;
        let artifact_root =
            repo_root.join(format!("docs/test-runs/core-engine/upstream-host/{run_id}"));
        reset_artifact_root(&artifact_root)?;

        let mut case_index = Vec::new();
        let mut expectation_mismatch_count = 0usize;
        let mut direct_oxfml_case_count = 0usize;
        let mut let_lambda_case_count = 0usize;
        let mut formatting_guard_case_count = 0usize;
        let mut w073_typed_rule_case_count = 0usize;

        for manifest_case in &manifest.cases {
            let case_path = repo_root
                .join(&manifest.base_path)
                .join(manifest_case.path.replace('/', "\\"));
            let case = load_case(&case_path)?;
            let execution = execute_fixture_case(&case)?;
            let mismatches = fixture_expectation_mismatches(&case, &execution);
            expectation_mismatch_count += mismatches.len();

            let tags = &manifest_case.tags;
            if tags.iter().any(|tag| tag == "direct-oxfml") {
                direct_oxfml_case_count += 1;
            }
            if tags.iter().any(|tag| tag == "let-lambda") {
                let_lambda_case_count += 1;
            }
            if tags.iter().any(|tag| tag == "formatting") {
                formatting_guard_case_count += 1;
            }
            if tags.iter().any(|tag| tag == "w073") {
                w073_typed_rule_case_count += 1;
            }

            let case_dir = artifact_root.join("cases").join(&case.case_id);
            create_dir_all(&case_dir)?;
            write_json(
                &case_dir.join("input_case.json"),
                &serde_json::to_value(&case).expect("fixture case should serialize"),
            )?;
            write_json(
                &case_dir.join("result.json"),
                &case_result_json(&case, &manifest_case.tags, &execution, &mismatches),
            )?;
            case_index.push(json!({
                "case_id": case.case_id,
                "path": format!("cases/{}/result.json", case.case_id),
                "tags": manifest_case.tags,
                "status": if mismatches.is_empty() { "matched" } else { "mismatched" },
                "expectation_mismatch_count": mismatches.len()
            }));
        }

        let artifact_root_rel = format!("docs/test-runs/core-engine/upstream-host/{run_id}");
        write_json(
            &artifact_root.join("case_index.json"),
            &json!({
                "schema_version": UPSTREAM_HOST_CASE_INDEX_SCHEMA_V1,
                "run_id": run_id,
                "cases": case_index
            }),
        )?;

        let summary = UpstreamHostRunSummary {
            run_id: run_id.to_string(),
            artifact_root: artifact_root_rel,
            fixture_case_count: manifest.cases.len(),
            expectation_mismatch_count,
            direct_oxfml_case_count,
            let_lambda_case_count,
            formatting_guard_case_count,
            w073_typed_rule_case_count,
        };
        write_json(
            &artifact_root.join("run_summary.json"),
            &run_summary_json(&summary),
        )?;
        Ok(summary)
    }
}

fn case_result_json(
    case: &UpstreamHostFixtureCase,
    tags: &[String],
    execution: &crate::upstream_host_fixture::UpstreamHostFixtureExecution,
    mismatches: &[String],
) -> Value {
    let output = &execution.recalc_output;
    json!({
        "schema_version": UPSTREAM_HOST_CASE_RESULT_SCHEMA_V1,
        "case_id": case.case_id,
        "description": case.description,
        "status": if mismatches.is_empty() { "matched" } else { "mismatched" },
        "expectation_mismatches": mismatches,
        "tags": tags,
        "direct_oxfml_runtime_surface": {
            "crate": "oxfml_core",
            "entry": "oxfml_core::consumer::runtime::RuntimeEnvironment::execute",
            "request": "oxfml_core::consumer::runtime::RuntimeFormulaRequest",
            "backend": case.evaluation_backend
        },
        "formula": {
            "formula_stable_id": output.source.formula_stable_id.0,
            "formula_text_version": output.source.formula_text_version.0,
            "formula_channel_kind": format!("{:?}", output.source.formula_channel_kind),
            "entered_formula_text": output.source.entered_formula_text
        },
        "returned_value_surface": {
            "kind": format!("{:?}", output.returned_value_surface.kind),
            "payload_summary": output.returned_value_surface.payload_summary,
            "rich_value_type_name": output.returned_value_surface.rich_value_type_name,
            "host_provider_outcome_kind": output.returned_value_surface.host_provider_outcome.as_ref().map(|outcome| format!("{:?}", outcome.outcome_kind))
        },
        "candidate_result": {
            "candidate_result_id": output.candidate_result.candidate_result_id,
            "value_payload": value_payload_summary(&output.candidate_result.value_delta.published_payload),
            "published_value_class": format!("{:?}", output.candidate_result.value_delta.published_value_class),
            "result_extent": output.candidate_result.value_delta.result_extent.as_ref().map(|extent| json!({
                "rows": extent.rows,
                "cols": extent.cols
            })),
            "shape_delta_present": true,
            "topology_spill_fact_count": output.candidate_result.topology_delta.spill_facts.len(),
            "topology_dependency_fact_count": output.candidate_result.topology_delta.dependency_consequence_facts.len(),
            "format_delta_present": output.candidate_result.format_delta.is_some(),
            "display_delta_present": output.candidate_result.display_delta.is_some()
        },
        "commit_decision": commit_decision_json(&output.commit_decision),
        "evaluation_trace": {
            "function_ids": trace_function_ids(output),
            "prepared_call_count": output.evaluation.trace.prepared_calls.len()
        },
        "typed_query_bundle": {
            "families": output.typed_query_bundle_spec.families.iter().map(|family| format!("{:?}", family)).collect::<Vec<_>>()
        },
        "verification_publication_surface": {
            "has_publication_context": output.verification_publication_surface.has_publication_context,
            "format_profile": output.verification_publication_surface.format_profile,
            "visible_value_text": output.verification_publication_surface.visible_value_text,
            "effective_display_text": output.verification_publication_surface.effective_display_text,
            "conditional_formatting_rule_kind": output.verification_publication_surface.conditional_formatting_rule_kind,
            "conditional_formatting_thresholds": output.verification_publication_surface.conditional_formatting_thresholds,
            "conditional_formatting_typed_rule_families": conditional_formatting_typed_rule_families(output),
            "array_cell_effective_fill_colors": array_cell_effective_fill_colors(output),
            "array_cell_data_bars": array_cell_data_bars(output),
            "array_cell_icons": array_cell_icons(output),
            "format_delta_present": output.verification_publication_surface.format_delta.is_some(),
            "display_delta_present": output.verification_publication_surface.display_delta.is_some()
        },
        "w037_interpretation": {
            "direct_oxfml_evaluator_reexecution": tags.iter().any(|tag| tag == "direct-oxfml"),
            "narrow_let_lambda_carrier": tags.iter().any(|tag| tag == "let-lambda"),
            "w073_typed_formatting_guard": tags.iter().any(|tag| tag == "w073"),
            "general_oxfunc_kernel_claimed": false,
            "pack_grade_replay_promoted": false
        }
    })
}

fn commit_decision_json(decision: &oxfml_core::seam::AcceptDecision) -> Value {
    match decision {
        oxfml_core::seam::AcceptDecision::Accepted(bundle) => json!({
            "kind": "accepted",
            "commit_attempt_id": bundle.commit_attempt_id,
            "candidate_result_id": bundle.candidate_result_id,
            "formula_stable_id": bundle.formula_stable_id
        }),
        oxfml_core::seam::AcceptDecision::Rejected(reject) => json!({
            "kind": "rejected",
            "commit_attempt_id": reject.commit_attempt_id,
            "trace_correlation_id": reject.trace_correlation_id,
            "reject_code": format!("{:?}", reject.reject_code)
        }),
    }
}

fn run_summary_json(summary: &UpstreamHostRunSummary) -> Value {
    json!({
        "schema_version": UPSTREAM_HOST_RUN_SUMMARY_SCHEMA_V1,
        "run_id": summary.run_id,
        "artifact_root": summary.artifact_root,
        "fixture_case_count": summary.fixture_case_count,
        "expectation_mismatch_count": summary.expectation_mismatch_count,
        "direct_oxfml_case_count": summary.direct_oxfml_case_count,
        "let_lambda_case_count": summary.let_lambda_case_count,
        "formatting_guard_case_count": summary.formatting_guard_case_count,
        "w073_typed_rule_case_count": summary.w073_typed_rule_case_count,
        "all_expectations_matched": summary.expectation_mismatch_count == 0,
        "promotion_limits": {
            "pack_grade_replay_promoted": false,
            "c5_promoted": false,
            "general_oxfunc_kernel_claimed": false
        }
    })
}

fn reset_artifact_root(path: &Path) -> Result<(), UpstreamHostRunnerError> {
    if path.exists() {
        fs::remove_dir_all(path).map_err(|source| UpstreamHostRunnerError::RemoveDir {
            path: path.display().to_string(),
            source,
        })?;
    }
    create_dir_all(path)
}

fn create_dir_all(path: &Path) -> Result<(), UpstreamHostRunnerError> {
    fs::create_dir_all(path).map_err(|source| UpstreamHostRunnerError::CreateDir {
        path: path.display().to_string(),
        source,
    })
}

fn write_json(path: &Path, value: &Value) -> Result<(), UpstreamHostRunnerError> {
    if let Some(parent) = path.parent() {
        create_dir_all(parent)?;
    }
    let text = serde_json::to_string_pretty(value).expect("json value should serialize");
    fs::write(path, format!("{text}\n")).map_err(|source| UpstreamHostRunnerError::Write {
        path: path.display().to_string(),
        source,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn upstream_host_runner_emits_direct_oxfml_artifacts() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let run_id = "test-upstream-host-direct-run";
        let artifact_root =
            repo_root.join(format!("docs/test-runs/core-engine/upstream-host/{run_id}"));
        let _ = fs::remove_dir_all(&artifact_root);

        let summary = UpstreamHostRunner::new()
            .execute(&repo_root, run_id)
            .unwrap();

        assert_eq!(summary.fixture_case_count, 16);
        assert_eq!(summary.expectation_mismatch_count, 0);
        assert_eq!(summary.direct_oxfml_case_count, 7);
        assert_eq!(summary.let_lambda_case_count, 2);
        assert_eq!(summary.w073_typed_rule_case_count, 5);
        assert!(artifact_root.join("run_summary.json").exists());

        let _ = fs::remove_dir_all(&artifact_root);
    }
}
