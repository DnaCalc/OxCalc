#![forbid(unsafe_code)]

//! Local TreeCalc fixture runner and artifact emission.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde_json::json;
use thiserror::Error;

use crate::coordinator::{RejectDetail, RuntimeEffect, RuntimeEffectFamily};
use crate::dependency::{
    DependencyDiagnostic, DependencyEdge, InvalidationClosure, InvalidationSeed,
};
use crate::recalc::OverlayEntry;
use crate::treecalc::{LocalTreeCalcRunArtifacts, LocalTreeCalcRunState};
use crate::treecalc_fixture::{
    TreeCalcFixtureError, TreeCalcFixtureExecution, TreeCalcFixtureExpected,
    TreeCalcFixturePostEditExecution, execute_fixture_case, load_case, load_manifest,
};

const TREECALC_RUN_MANIFEST_SCHEMA_V1: &str = "oxcalc.treecalc.local_run_manifest.v1";
const TREECALC_RUN_SUMMARY_SCHEMA_V1: &str = "oxcalc.treecalc.local_run_summary.v1";
const TREECALC_LOCAL_TRACE_SCHEMA_V1: &str = "oxcalc.treecalc.local_trace.v1";
const TREECALC_LOCAL_EXPLAIN_SCHEMA_V1: &str = "oxcalc.treecalc.local_explain.v1";

#[derive(Debug, Error)]
pub enum TreeCalcRunnerError {
    #[error(transparent)]
    Fixture(#[from] TreeCalcFixtureError),
    #[error("failed to create artifact root {path}: {source}")]
    CreateDirectory {
        path: String,
        source: std::io::Error,
    },
    #[error("failed to remove existing artifact root {path}: {source}")]
    RemoveDirectory {
        path: String,
        source: std::io::Error,
    },
    #[error("failed to write artifact {path}: {source}")]
    WriteFile {
        path: String,
        source: std::io::Error,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeCalcRunSummary {
    pub run_id: String,
    pub schema_version: String,
    pub case_count: usize,
    pub result_counts: Vec<(String, usize)>,
    pub expectation_mismatch_count: usize,
    pub artifact_root: String,
}

#[derive(Debug, Clone, Default)]
pub struct TreeCalcRunner;

impl TreeCalcRunner {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    pub fn execute_manifest(
        &self,
        repo_root: &Path,
        run_id: &str,
    ) -> Result<TreeCalcRunSummary, TreeCalcRunnerError> {
        let manifest_path = repo_root.join("docs/test-fixtures/core-engine/treecalc/MANIFEST.json");
        let manifest = load_manifest(&manifest_path)?;

        let artifact_root = repo_root.join(format!(
            "docs/test-runs/core-engine/treecalc-local/{run_id}"
        ));
        let relative_artifact_root =
            relative_artifact_path(["docs", "test-runs", "core-engine", "treecalc-local", run_id]);

        if artifact_root.exists() {
            fs::remove_dir_all(&artifact_root).map_err(|source| {
                TreeCalcRunnerError::RemoveDirectory {
                    path: artifact_root.display().to_string(),
                    source,
                }
            })?;
        }

        create_directory(&artifact_root)?;
        create_directory(&artifact_root.join("cases"))?;
        create_directory(&artifact_root.join("conformance"))?;

        write_json(
            &artifact_root.join("manifest_selection.json"),
            &json!({
                "schema_version": TREECALC_RUN_MANIFEST_SCHEMA_V1,
                "corpus_id": manifest.corpus_id,
                "base_path": manifest.base_path,
                "cases": manifest
                    .cases
                    .iter()
                    .map(|entry| {
                        json!({
                            "case_id": entry.case_id,
                            "path": entry.path,
                            "tags": entry.tags,
                        })
                    })
                    .collect::<Vec<_>>(),
            }),
        )?;

        let engine = crate::treecalc::LocalTreeCalcEngine;
        let mut case_results = Vec::new();
        let mut oracle_baseline = Vec::new();
        let mut engine_diff = Vec::new();
        let mut explain_index = Vec::new();

        for entry in &manifest.cases {
            let case_path = repo_root
                .join("docs/test-fixtures/core-engine/treecalc")
                .join(entry.path.replace('/', "\\"));
            let case = load_case(&case_path)?;
            let execution = execute_fixture_case(&engine, &case)?;
            let artifacts = &execution.initial_artifacts;
            let case_directory = artifact_root.join("cases").join(&entry.case_id);
            create_directory(&case_directory)?;
            let case_artifact_paths =
                write_case_artifacts(&case_directory, &relative_artifact_root, &case, &execution)?;
            let expectation_mismatches = compare_expected(&case.expected, artifacts);
            let conformance_artifacts = write_case_conformance_artifacts(
                &case_directory,
                &relative_artifact_root,
                &case,
                artifacts,
                &expectation_mismatches,
            )?;
            let support_artifacts = write_case_trace_and_explain_artifacts(
                &case_directory,
                &relative_artifact_root,
                &case,
                artifacts,
                &expectation_mismatches,
            )?;
            oracle_baseline.push(case_oracle_baseline_object(&case));
            engine_diff.push(case_engine_diff_object(
                &case,
                artifacts,
                &expectation_mismatches,
            ));
            explain_index.push(case_explain_index_object(
                &case,
                artifacts,
                &relative_artifact_root,
                &expectation_mismatches,
            ));
            case_results.push(json!({
                "case_id": case.case_id,
                "description": case.description,
                "result_state": result_state_name(&artifacts.result_state),
                "tags": entry.tags,
                "expectation_mismatches": expectation_mismatches,
                "conformance_state": conformance_state_name(&expectation_mismatches),
                "artifact_paths": case_artifact_paths,
                "conformance_artifact_paths": conformance_artifacts,
                "supporting_artifact_paths": support_artifacts,
            }));
        }

        let mut result_counts = BTreeMap::new();
        let mut expectation_mismatch_count = 0usize;
        let mut mismatch_case_count = 0usize;
        for case_result in &case_results {
            let result_state = case_result["result_state"]
                .as_str()
                .expect("result_state should be present");
            *result_counts
                .entry(result_state.to_string())
                .or_insert(0usize) += 1;
            let mismatch_count = case_result["expectation_mismatches"]
                .as_array()
                .map_or(0, std::vec::Vec::len);
            expectation_mismatch_count += mismatch_count;
            if mismatch_count > 0 {
                mismatch_case_count += 1;
            }
        }

        write_json(&artifact_root.join("case_index.json"), &json!(case_results))?;
        write_json(
            &artifact_root.join("conformance/oracle_baseline.json"),
            &json!(oracle_baseline),
        )?;
        write_json(
            &artifact_root.join("conformance/engine_diff.json"),
            &json!(engine_diff),
        )?;
        write_json(
            &artifact_root.join("conformance/conformance_summary.json"),
            &json!({
                "case_count": manifest.cases.len(),
                "mismatch_case_count": mismatch_case_count,
                "expectation_mismatch_count": expectation_mismatch_count,
                "conformance_pass_count": manifest.cases.len() - mismatch_case_count,
                "oracle_baseline_path": format!("{relative_artifact_root}/conformance/oracle_baseline.json"),
                "engine_diff_path": format!("{relative_artifact_root}/conformance/engine_diff.json"),
            }),
        )?;
        write_json(
            &artifact_root.join("conformance/explain_index.json"),
            &json!(explain_index),
        )?;

        let summary = TreeCalcRunSummary {
            run_id: run_id.to_string(),
            schema_version: TREECALC_RUN_SUMMARY_SCHEMA_V1.to_string(),
            case_count: manifest.cases.len(),
            result_counts: result_counts.into_iter().collect(),
            expectation_mismatch_count,
            artifact_root: artifact_root.display().to_string(),
        };

        write_json(
            &artifact_root.join("run_summary.json"),
            &json!({
                "schema_version": TREECALC_RUN_SUMMARY_SCHEMA_V1,
                "run_id": summary.run_id,
                "case_count": summary.case_count,
                "result_counts": BTreeMap::from_iter(summary.result_counts.clone()),
                "expectation_mismatch_count": summary.expectation_mismatch_count,
                "artifact_root": relative_artifact_root,
            }),
        )?;

        Ok(summary)
    }
}

fn compare_expected(
    expected: &TreeCalcFixtureExpected,
    artifacts: &LocalTreeCalcRunArtifacts,
) -> Vec<String> {
    let mut mismatches = Vec::new();

    if expected.result_state != result_state_name(&artifacts.result_state) {
        mismatches.push(format!(
            "result_state: expected {} observed {}",
            expected.result_state,
            result_state_name(&artifacts.result_state)
        ));
    }

    if let Some(expected_values) = &expected.published_values {
        let actual_values = artifacts
            .published_values
            .iter()
            .map(|(node_id, value)| (node_id.0, value.clone()))
            .collect::<BTreeMap<_, _>>();
        for (node_id, expected_value) in expected_values {
            match actual_values.get(node_id) {
                Some(actual_value) if actual_value == expected_value => {}
                Some(actual_value) => mismatches.push(format!(
                    "published_value:{node_id}: expected {expected_value} observed {actual_value}"
                )),
                None => mismatches.push(format!(
                    "published_value:{node_id}: expected {expected_value} observed <missing>"
                )),
            }
        }
    }

    if let Some(expected_order) = &expected.evaluation_order {
        let actual_order = artifacts
            .evaluation_order
            .iter()
            .map(|node_id| node_id.0)
            .collect::<Vec<_>>();
        if actual_order != *expected_order {
            mismatches.push(format!(
                "evaluation_order: expected {:?} observed {:?}",
                expected_order, actual_order
            ));
        }
    }

    if let Some(expected_reject_kind) = &expected.reject_kind {
        let observed_reject_kind = artifacts
            .reject_detail
            .as_ref()
            .map(|detail| format!("{:?}", detail.kind));
        if observed_reject_kind.as_ref() != Some(expected_reject_kind) {
            mismatches.push(format!(
                "reject_kind: expected {} observed {}",
                expected_reject_kind,
                observed_reject_kind.unwrap_or_else(|| "<none>".to_string())
            ));
        }
    }

    if let Some(expected_runtime_effect_kinds) = &expected.runtime_effect_kinds {
        let observed_runtime_effect_kinds = artifacts
            .runtime_effects
            .iter()
            .map(|runtime_effect| runtime_effect.kind.clone())
            .collect::<Vec<_>>();
        if observed_runtime_effect_kinds != *expected_runtime_effect_kinds {
            mismatches.push(format!(
                "runtime_effect_kinds: expected {:?} observed {:?}",
                expected_runtime_effect_kinds, observed_runtime_effect_kinds
            ));
        }
    }

    mismatches
}

fn result_state_name(result_state: &LocalTreeCalcRunState) -> &'static str {
    match result_state {
        LocalTreeCalcRunState::Published => "published",
        LocalTreeCalcRunState::VerifiedClean => "verified_clean",
        LocalTreeCalcRunState::Rejected => "rejected",
    }
}

fn conformance_state_name(mismatches: &[String]) -> &'static str {
    if mismatches.is_empty() {
        "matches_expected"
    } else {
        "mismatch_against_expected"
    }
}

fn write_case_artifacts(
    case_directory: &Path,
    relative_artifact_root: &str,
    case: &crate::treecalc_fixture::TreeCalcFixtureCase,
    execution: &TreeCalcFixtureExecution,
) -> Result<serde_json::Value, TreeCalcRunnerError> {
    let artifacts = &execution.initial_artifacts;
    write_json(
        case_directory.join("input_case.json").as_path(),
        &json!(case),
    )?;
    write_json(
        case_directory.join("published_values.json").as_path(),
        &json!(
            artifacts
                .published_values
                .iter()
                .map(|(node_id, value)| json!({
                    "node_id": node_id.0,
                    "value": value,
                }))
                .collect::<Vec<_>>()
        ),
    )?;
    write_json(
        case_directory.join("runtime_effects.json").as_path(),
        &json!(
            artifacts
                .runtime_effects
                .iter()
                .map(runtime_effect_json)
                .collect::<Vec<_>>()
        ),
    )?;
    write_json(
        case_directory
            .join("runtime_effect_overlays.json")
            .as_path(),
        &json!(
            artifacts
                .runtime_effect_overlays
                .iter()
                .map(overlay_json)
                .collect::<Vec<_>>()
        ),
    )?;
    write_json(
        case_directory.join("dependency_graph.json").as_path(),
        &json!({
            "cycle_groups": artifacts.dependency_graph.cycle_groups.iter().map(|group| {
                group.iter().map(|node_id| node_id.0).collect::<Vec<_>>()
            }).collect::<Vec<_>>(),
            "descriptors": artifacts.dependency_graph.descriptors_by_owner.values().flat_map(|descriptors| descriptors.iter()).map(dependency_descriptor_json).collect::<Vec<_>>(),
            "diagnostics": artifacts.dependency_graph.diagnostics.iter().map(dependency_diagnostic_json).collect::<Vec<_>>(),
            "edges": artifacts.dependency_graph.edges_by_owner.values().flat_map(|edges| edges.iter()).map(dependency_edge_json).collect::<Vec<_>>(),
        }),
    )?;
    write_json(
        case_directory.join("invalidation_closure.json").as_path(),
        &invalidation_closure_json(&artifacts.invalidation_closure),
    )?;
    write_json(
        case_directory.join("node_states.json").as_path(),
        &json!(
            artifacts
                .node_states
                .iter()
                .map(|(node_id, state)| json!({
                    "node_id": node_id.0,
                    "state": format!("{state:?}"),
                }))
                .collect::<Vec<_>>()
        ),
    )?;
    write_json(
        case_directory.join("result.json").as_path(),
        &json!({
            "case_id": case.case_id,
            "result_state": result_state_name(&artifacts.result_state),
            "evaluation_order": artifacts.evaluation_order.iter().map(|node_id| node_id.0).collect::<Vec<_>>(),
            "diagnostics": artifacts.diagnostics,
            "published_values_path": relative_case_artifact_path(relative_artifact_root, &case.case_id, "published_values.json"),
            "runtime_effects_path": relative_case_artifact_path(relative_artifact_root, &case.case_id, "runtime_effects.json"),
            "runtime_effect_overlays_path": relative_case_artifact_path(relative_artifact_root, &case.case_id, "runtime_effect_overlays.json"),
            "dependency_graph_path": relative_case_artifact_path(relative_artifact_root, &case.case_id, "dependency_graph.json"),
            "invalidation_closure_path": relative_case_artifact_path(relative_artifact_root, &case.case_id, "invalidation_closure.json"),
            "node_states_path": relative_case_artifact_path(relative_artifact_root, &case.case_id, "node_states.json"),
            "reject_detail": artifacts.reject_detail.as_ref().map(reject_detail_json),
            "candidate_result": artifacts.candidate_result.as_ref().map(|candidate_result| json!({
                "aligned_canonical_family": "AcceptedCandidateResult",
                "projection_owner": "oxcalc_local",
                "candidate_result_id": candidate_result.candidate_result_id,
                "target_set": candidate_result.target_set.iter().map(|node_id| node_id.0).collect::<Vec<_>>(),
                "value_updates": candidate_result.value_updates.iter().map(|(node_id, value)| (node_id.0.to_string(), value.clone())).collect::<BTreeMap<_, _>>(),
                "runtime_effects": candidate_result.runtime_effects.iter().map(runtime_effect_json).collect::<Vec<_>>(),
            })),
            "publication_bundle": artifacts.publication_bundle.as_ref().map(|publication_bundle| json!({
                "aligned_canonical_family": "CommitBundle",
                "projection_owner": "oxcalc_local",
                "publication_id": publication_bundle.publication_id,
                "candidate_result_id": publication_bundle.candidate_result_id,
                "published_view_delta": publication_bundle.published_view_delta.iter().map(|(node_id, value)| (node_id.0.to_string(), value.clone())).collect::<BTreeMap<_, _>>(),
                "published_runtime_effects": publication_bundle.published_runtime_effects.iter().map(runtime_effect_json).collect::<Vec<_>>(),
                "trace_markers": publication_bundle.trace_markers,
                "carriage_classification": publication_carriage_classification_json(artifacts),
            })),
            "execution_restriction_interaction": execution_restriction_interaction_json(artifacts),
        }),
    )?;

    let mut artifact_paths = json!({
        "input_case": relative_case_artifact_path(relative_artifact_root, &case.case_id, "input_case.json"),
        "result": relative_case_artifact_path(relative_artifact_root, &case.case_id, "result.json"),
        "published_values": relative_case_artifact_path(relative_artifact_root, &case.case_id, "published_values.json"),
        "runtime_effects": relative_case_artifact_path(relative_artifact_root, &case.case_id, "runtime_effects.json"),
        "runtime_effect_overlays": relative_case_artifact_path(relative_artifact_root, &case.case_id, "runtime_effect_overlays.json"),
        "dependency_graph": relative_case_artifact_path(relative_artifact_root, &case.case_id, "dependency_graph.json"),
        "invalidation_closure": relative_case_artifact_path(relative_artifact_root, &case.case_id, "invalidation_closure.json"),
        "node_states": relative_case_artifact_path(relative_artifact_root, &case.case_id, "node_states.json"),
    });

    if let Some(post_edit_execution) = &execution.post_edit {
        let post_edit_artifact_paths = write_post_edit_artifacts(
            case_directory,
            relative_artifact_root,
            case,
            post_edit_execution,
        )?;
        artifact_paths
            .as_object_mut()
            .expect("artifact paths should be object")
            .insert("post_edit".to_string(), post_edit_artifact_paths);
    }

    Ok(artifact_paths)
}

fn write_post_edit_artifacts(
    case_directory: &Path,
    relative_artifact_root: &str,
    case: &crate::treecalc_fixture::TreeCalcFixtureCase,
    execution: &TreeCalcFixturePostEditExecution,
) -> Result<serde_json::Value, TreeCalcRunnerError> {
    let post_edit_directory = case_directory.join("post_edit");
    create_directory(&post_edit_directory)?;

    write_json(
        post_edit_directory.join("edit_outcomes.json").as_path(),
        &json!(execution
            .edit_outcomes
            .iter()
            .map(|outcome| json!({
                "snapshot_id": outcome.snapshot.snapshot_id().0,
                "impact": format!("{:?}", outcome.impact),
                "affected_node_ids": outcome.affected_node_ids.iter().map(|node_id| node_id.0).collect::<Vec<_>>(),
                "diagnostic_events": outcome.diagnostic_events,
            }))
            .collect::<Vec<_>>()),
    )?;
    write_json(
        post_edit_directory.join("runtime_effects.json").as_path(),
        &json!(
            execution
                .rerun_artifacts
                .runtime_effects
                .iter()
                .map(runtime_effect_json)
                .collect::<Vec<_>>()
        ),
    )?;
    write_json(
        post_edit_directory
            .join("runtime_effect_overlays.json")
            .as_path(),
        &json!(
            execution
                .rerun_artifacts
                .runtime_effect_overlays
                .iter()
                .map(overlay_json)
                .collect::<Vec<_>>()
        ),
    )?;
    write_json(
        post_edit_directory
            .join("invalidation_seeds.json")
            .as_path(),
        &json!(
            execution
                .invalidation_seeds
                .iter()
                .map(invalidation_seed_json)
                .collect::<Vec<_>>()
        ),
    )?;
    write_json(
        post_edit_directory.join("result.json").as_path(),
        &json!({
            "case_id": case.case_id,
            "result_state": result_state_name(&execution.rerun_artifacts.result_state),
            "evaluation_order": execution.rerun_artifacts.evaluation_order.iter().map(|node_id| node_id.0).collect::<Vec<_>>(),
            "reject_detail": execution.rerun_artifacts.reject_detail.as_ref().map(reject_detail_json),
            "invalidation_seeds": execution.invalidation_seeds.iter().map(invalidation_seed_json).collect::<Vec<_>>(),
            "runtime_effects": execution.rerun_artifacts.runtime_effects.iter().map(runtime_effect_json).collect::<Vec<_>>(),
            "runtime_effect_overlays": execution.rerun_artifacts.runtime_effect_overlays.iter().map(overlay_json).collect::<Vec<_>>(),
            "published_values": execution.rerun_artifacts.published_values.iter().map(|(node_id, value)| (node_id.0.to_string(), value.clone())).collect::<BTreeMap<_, _>>(),
        }),
    )?;
    write_json(
        post_edit_directory.join("trace.json").as_path(),
        &json!({
            "schema_version": TREECALC_LOCAL_TRACE_SCHEMA_V1,
            "case_id": case.case_id,
            "phase": "post_edit",
            "result_state": result_state_name(&execution.rerun_artifacts.result_state),
            "events": build_trace_events(case, &execution.rerun_artifacts),
        }),
    )?;
    write_json(
        post_edit_directory.join("explain.json").as_path(),
        &json!({
            "schema_version": TREECALC_LOCAL_EXPLAIN_SCHEMA_V1,
            "case_id": case.case_id,
            "phase": "post_edit",
            "result_state": result_state_name(&execution.rerun_artifacts.result_state),
            "edit_impacts": execution.edit_outcomes.iter().map(|outcome| format!("{:?}", outcome.impact)).collect::<Vec<_>>(),
            "diagnostic_events": execution.edit_outcomes.iter().flat_map(|outcome| outcome.diagnostic_events.iter().cloned()).collect::<Vec<_>>(),
            "reject_detail": execution.rerun_artifacts.reject_detail.as_ref().map(reject_detail_json),
            "invalidation_seeds": execution.invalidation_seeds.iter().map(invalidation_seed_json).collect::<Vec<_>>(),
            "runtime_effects": execution.rerun_artifacts.runtime_effects.iter().map(runtime_effect_json).collect::<Vec<_>>(),
            "runtime_effect_overlays": execution.rerun_artifacts.runtime_effect_overlays.iter().map(overlay_json).collect::<Vec<_>>(),
        }),
    )?;

    Ok(json!({
        "edit_outcomes": relative_case_artifact_path(relative_artifact_root, &case.case_id, "post_edit/edit_outcomes.json"),
        "invalidation_seeds": relative_case_artifact_path(relative_artifact_root, &case.case_id, "post_edit/invalidation_seeds.json"),
        "runtime_effects": relative_case_artifact_path(relative_artifact_root, &case.case_id, "post_edit/runtime_effects.json"),
        "runtime_effect_overlays": relative_case_artifact_path(relative_artifact_root, &case.case_id, "post_edit/runtime_effect_overlays.json"),
        "result": relative_case_artifact_path(relative_artifact_root, &case.case_id, "post_edit/result.json"),
        "trace": relative_case_artifact_path(relative_artifact_root, &case.case_id, "post_edit/trace.json"),
        "explain": relative_case_artifact_path(relative_artifact_root, &case.case_id, "post_edit/explain.json"),
    }))
}

fn write_case_conformance_artifacts(
    case_directory: &Path,
    relative_artifact_root: &str,
    case: &crate::treecalc_fixture::TreeCalcFixtureCase,
    artifacts: &LocalTreeCalcRunArtifacts,
    expectation_mismatches: &[String],
) -> Result<serde_json::Value, TreeCalcRunnerError> {
    write_json(
        case_directory.join("oracle.json").as_path(),
        &json!({
            "case_id": case.case_id,
            "expected_result_state": case.expected.result_state,
            "expected_published_values": case.expected.published_values,
            "expected_evaluation_order": case.expected.evaluation_order,
            "expected_reject_kind": case.expected.reject_kind,
            "expected_runtime_effect_kinds": case.expected.runtime_effect_kinds,
        }),
    )?;
    write_json(
        case_directory.join("engine_diff.json").as_path(),
        &json!({
            "case_id": case.case_id,
            "observed_result_state": result_state_name(&artifacts.result_state),
            "conformance_state": conformance_state_name(expectation_mismatches),
            "mismatches": expectation_mismatches,
        }),
    )?;

    Ok(json!({
        "oracle": relative_case_artifact_path(relative_artifact_root, &case.case_id, "oracle.json"),
        "engine_diff": relative_case_artifact_path(relative_artifact_root, &case.case_id, "engine_diff.json"),
    }))
}

fn write_case_trace_and_explain_artifacts(
    case_directory: &Path,
    relative_artifact_root: &str,
    case: &crate::treecalc_fixture::TreeCalcFixtureCase,
    artifacts: &LocalTreeCalcRunArtifacts,
    expectation_mismatches: &[String],
) -> Result<serde_json::Value, TreeCalcRunnerError> {
    write_json(
        case_directory.join("trace.json").as_path(),
        &json!({
            "schema_version": TREECALC_LOCAL_TRACE_SCHEMA_V1,
            "case_id": case.case_id,
            "result_state": result_state_name(&artifacts.result_state),
            "events": build_trace_events(case, artifacts),
        }),
    )?;
    write_json(
        case_directory.join("explain.json").as_path(),
        &json!({
            "schema_version": TREECALC_LOCAL_EXPLAIN_SCHEMA_V1,
            "case_id": case.case_id,
            "conformance_state": conformance_state_name(expectation_mismatches),
            "mismatch_count": expectation_mismatches.len(),
            "mismatches": expectation_mismatches,
            "reject_detail": artifacts.reject_detail.as_ref().map(reject_detail_json),
            "runtime_effects": artifacts.runtime_effects.iter().map(runtime_effect_json).collect::<Vec<_>>(),
            "runtime_effect_overlays": artifacts.runtime_effect_overlays.iter().map(overlay_json).collect::<Vec<_>>(),
            "publication_bundle": artifacts.publication_bundle.as_ref().map(|publication_bundle| json!({
                "aligned_canonical_family": "CommitBundle",
                "projection_owner": "oxcalc_local",
                "publication_id": publication_bundle.publication_id,
                "candidate_result_id": publication_bundle.candidate_result_id,
                "published_value_delta_node_count": publication_bundle.published_view_delta.len(),
                "published_runtime_effect_count": publication_bundle.published_runtime_effects.len(),
                "trace_marker_count": publication_bundle.trace_markers.len(),
                "carriage_classification": publication_carriage_classification_json(artifacts),
            })),
            "execution_restriction_interaction": execution_restriction_interaction_json(artifacts),
            "notes": build_explain_notes(artifacts, expectation_mismatches),
        }),
    )?;

    Ok(json!({
        "trace": relative_case_artifact_path(relative_artifact_root, &case.case_id, "trace.json"),
        "explain": relative_case_artifact_path(relative_artifact_root, &case.case_id, "explain.json"),
    }))
}

fn build_trace_events(
    case: &crate::treecalc_fixture::TreeCalcFixtureCase,
    artifacts: &LocalTreeCalcRunArtifacts,
) -> Vec<serde_json::Value> {
    let mut events = Vec::new();
    let mut step_id = 1usize;

    events.push(json!({
        "step_id": step_id,
        "label": "case_loaded",
        "case_id": case.case_id,
        "formula_count": case.formulas.len(),
        "node_count": case.nodes.len(),
    }));
    step_id += 1;

    for node_id in &artifacts.evaluation_order {
        events.push(json!({
            "step_id": step_id,
            "label": "evaluate_node",
            "node_id": node_id.0,
        }));
        step_id += 1;
    }

    for runtime_effect in &artifacts.runtime_effects {
        events.push(json!({
            "step_id": step_id,
            "label": "runtime_effect_observed",
            "kind": runtime_effect.kind,
            "kind_owner": "oxcalc_local_projection",
            "family": format!("{:?}", runtime_effect.family),
            "family_owner": "oxcalc_local_projection",
            "detail": runtime_effect.detail,
        }));
        step_id += 1;
    }

    for overlay in &artifacts.runtime_effect_overlays {
        events.push(json!({
            "step_id": step_id,
            "label": "runtime_effect_overlay_emitted",
            "owner_node_id": overlay.key.owner_node_id.0,
            "overlay_kind": format!("{:?}", overlay.key.overlay_kind),
            "payload_identity": overlay.key.payload_identity,
        }));
        step_id += 1;
    }

    if let Some(candidate_result) = &artifacts.candidate_result {
        events.push(json!({
            "step_id": step_id,
            "label": "candidate_adapted",
            "aligned_canonical_family": "AcceptedCandidateResult",
            "projection_owner": "oxcalc_local",
            "candidate_result_id": candidate_result.candidate_result_id,
            "target_set": candidate_result.target_set.iter().map(|node_id| node_id.0).collect::<Vec<_>>(),
        }));
        step_id += 1;
    }

    if let Some(publication_bundle) = &artifacts.publication_bundle {
        events.push(json!({
            "step_id": step_id,
            "label": "publication_committed",
            "aligned_canonical_family": "CommitBundle",
            "projection_owner": "oxcalc_local",
            "publication_id": publication_bundle.publication_id,
            "candidate_result_id": publication_bundle.candidate_result_id,
        }));
        step_id += 1;
    }

    if let Some(reject_detail) = &artifacts.reject_detail {
        events.push(json!({
            "step_id": step_id,
            "label": "candidate_rejected",
            "aligned_canonical_family": "RejectRecord",
            "projection_owner": "oxcalc_local",
            "candidate_result_id": reject_detail.candidate_result_id,
            "kind": format!("{:?}", reject_detail.kind),
            "kind_owner": "oxcalc_local_projection",
            "detail": reject_detail.detail,
        }));
        step_id += 1;
    }

    events.push(json!({
        "step_id": step_id,
        "label": "run_finished",
        "result_state": result_state_name(&artifacts.result_state),
    }));

    events
}

fn build_explain_notes(
    artifacts: &LocalTreeCalcRunArtifacts,
    expectation_mismatches: &[String],
) -> Vec<String> {
    let mut notes = Vec::new();

    if expectation_mismatches.is_empty() {
        notes.push("local fixture expectation floor matched observed artifacts".to_string());
    } else {
        notes.push("local fixture expectation floor diverged from observed artifacts".to_string());
    }

    if !artifacts.runtime_effects.is_empty() {
        notes.push(
            "runtime-derived effects were emitted in the local sequential runtime".to_string(),
        );
    }

    if artifacts.reject_detail.is_some() {
        notes.push("result ended in conservative local rejection".to_string());
    }

    if artifacts.publication_bundle.is_some() {
        notes.push("result reached candidate adaptation and publication".to_string());
    }

    notes
}

fn runtime_effect_json(runtime_effect: &RuntimeEffect) -> serde_json::Value {
    json!({
        "kind": runtime_effect.kind,
        "kind_owner": "oxcalc_local_projection",
        "family": format!("{:?}", runtime_effect.family),
        "family_owner": "oxcalc_local_projection",
        "detail": runtime_effect.detail,
    })
}

fn publication_carriage_classification_json(
    artifacts: &LocalTreeCalcRunArtifacts,
) -> serde_json::Value {
    let dependency_shape_update_count = artifacts
        .candidate_result
        .as_ref()
        .map(|candidate_result| candidate_result.dependency_shape_updates.len())
        .unwrap_or(0);

    json!({
        "publish_critical_categories": ["value_delta"],
        "replay_visible_non_publish_critical_categories": [
            "published_runtime_effects",
            "trace_markers",
        ],
        "local_floor_only_categories": ["dependency_shape_updates"],
        "explicit_current_absence_categories": [
            "shape_delta",
            "topology_delta",
            "format_delta",
            "display_delta",
        ],
        "dependency_shape_update_count": dependency_shape_update_count,
    })
}

fn execution_restriction_interaction_json(
    artifacts: &LocalTreeCalcRunArtifacts,
) -> serde_json::Value {
    let execution_restriction_count = artifacts
        .runtime_effects
        .iter()
        .filter(|runtime_effect| {
            matches!(
                runtime_effect.family,
                RuntimeEffectFamily::ExecutionRestriction
            )
        })
        .count();

    let publication_outcome = if execution_restriction_count == 0 {
        "none_observed"
    } else if artifacts.publication_bundle.is_some() {
        "published_sidecar_only"
    } else {
        "rejected_no_publication"
    };

    json!({
        "execution_restriction_observed": execution_restriction_count > 0,
        "execution_restriction_count": execution_restriction_count,
        "publication_outcome": publication_outcome,
        "publication_sensitive_consequence": false,
        "topology_sensitive_consequence": false,
    })
}

fn overlay_json(overlay: &OverlayEntry) -> serde_json::Value {
    json!({
        "owner_node_id": overlay.key.owner_node_id.0,
        "overlay_kind": format!("{:?}", overlay.key.overlay_kind),
        "structural_snapshot_id": overlay.key.structural_snapshot_id.0,
        "compatibility_basis": overlay.key.compatibility_basis,
        "payload_identity": overlay.key.payload_identity,
        "is_protected": overlay.is_protected,
        "is_eviction_eligible": overlay.is_eviction_eligible,
        "detail": overlay.detail,
    })
}

fn dependency_edge_json(edge: &DependencyEdge) -> serde_json::Value {
    json!({
        "edge_id": edge.edge_id,
        "descriptor_id": edge.descriptor_id,
        "owner_node_id": edge.owner_node_id.0,
        "target_node_id": edge.target_node_id.0,
        "kind": format!("{:?}", edge.kind),
    })
}

fn dependency_descriptor_json(
    descriptor: &crate::dependency::DependencyDescriptor,
) -> serde_json::Value {
    json!({
        "descriptor_id": descriptor.descriptor_id,
        "owner_node_id": descriptor.owner_node_id.0,
        "target_node_id": descriptor.target_node_id.map(|node_id| node_id.0),
        "kind": format!("{:?}", descriptor.kind),
        "carrier_detail": descriptor.carrier_detail,
        "requires_rebind_on_structural_change": descriptor.requires_rebind_on_structural_change,
    })
}

fn dependency_diagnostic_json(diagnostic: &DependencyDiagnostic) -> serde_json::Value {
    json!({
        "descriptor_id": diagnostic.descriptor_id,
        "kind": format!("{:?}", diagnostic.kind),
        "detail": diagnostic.detail,
    })
}

fn invalidation_closure_json(closure: &InvalidationClosure) -> serde_json::Value {
    json!(closure
        .records
        .values()
        .map(|record| json!({
            "node_id": record.node_id.0,
            "calc_state": format!("{:?}", record.calc_state),
            "requires_rebind": record.requires_rebind,
            "reasons": record.reasons.iter().map(|reason| format!("{reason:?}")).collect::<Vec<_>>(),
        }))
        .collect::<Vec<_>>())
}

fn invalidation_seed_json(seed: &InvalidationSeed) -> serde_json::Value {
    json!({
        "node_id": seed.node_id.0,
        "reason": format!("{:?}", seed.reason),
    })
}

fn reject_detail_json(reject_detail: &RejectDetail) -> serde_json::Value {
    json!({
        "aligned_canonical_family": "RejectRecord",
        "projection_owner": "oxcalc_local",
        "candidate_result_id": reject_detail.candidate_result_id,
        "kind": format!("{:?}", reject_detail.kind),
        "kind_owner": "oxcalc_local_projection",
        "detail": reject_detail.detail,
    })
}

fn case_oracle_baseline_object(
    case: &crate::treecalc_fixture::TreeCalcFixtureCase,
) -> serde_json::Value {
    json!({
        "case_id": case.case_id,
        "expected_result_state": case.expected.result_state,
        "expected_published_values": case.expected.published_values,
        "expected_evaluation_order": case.expected.evaluation_order,
        "expected_reject_kind": case.expected.reject_kind,
        "expected_runtime_effect_kinds": case.expected.runtime_effect_kinds,
    })
}

fn case_engine_diff_object(
    case: &crate::treecalc_fixture::TreeCalcFixtureCase,
    artifacts: &LocalTreeCalcRunArtifacts,
    expectation_mismatches: &[String],
) -> serde_json::Value {
    json!({
        "case_id": case.case_id,
        "observed_result_state": result_state_name(&artifacts.result_state),
        "conformance_state": conformance_state_name(expectation_mismatches),
        "mismatches": expectation_mismatches,
    })
}

fn case_explain_index_object(
    case: &crate::treecalc_fixture::TreeCalcFixtureCase,
    artifacts: &LocalTreeCalcRunArtifacts,
    relative_artifact_root: &str,
    expectation_mismatches: &[String],
) -> serde_json::Value {
    json!({
        "case_id": case.case_id,
        "conformance_state": conformance_state_name(expectation_mismatches),
        "result_state": result_state_name(&artifacts.result_state),
        "explain": relative_case_artifact_path(relative_artifact_root, &case.case_id, "explain.json"),
        "trace": relative_case_artifact_path(relative_artifact_root, &case.case_id, "trace.json"),
    })
}

fn create_directory(path: &Path) -> Result<(), TreeCalcRunnerError> {
    fs::create_dir_all(path).map_err(|source| TreeCalcRunnerError::CreateDirectory {
        path: path.display().to_string(),
        source,
    })
}

fn write_json(path: &Path, value: &serde_json::Value) -> Result<(), TreeCalcRunnerError> {
    let text = serde_json::to_string_pretty(value).expect("json serialization should succeed");
    fs::write(path, text).map_err(|source| TreeCalcRunnerError::WriteFile {
        path: path.display().to_string(),
        source,
    })
}

fn relative_artifact_path<'a>(segments: impl IntoIterator<Item = &'a str>) -> String {
    segments.into_iter().collect::<Vec<_>>().join("/")
}

fn relative_case_artifact_path(
    relative_artifact_root: &str,
    case_id: &str,
    file_name: &str,
) -> String {
    format!("{relative_artifact_root}/cases/{case_id}/{file_name}")
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    use super::*;
    use crate::dependency::{InvalidationReasonKind, NodeInvalidationRecord};
    use crate::recalc::NodeCalcState;
    use crate::structural::TreeNodeId;

    #[test]
    fn invalidation_closure_json_preserves_non_structural_reasons() {
        let closure = InvalidationClosure {
            impacted_order: vec![TreeNodeId(2), TreeNodeId(4)],
            records: BTreeMap::from([
                (
                    TreeNodeId(2),
                    NodeInvalidationRecord {
                        node_id: TreeNodeId(2),
                        calc_state: NodeCalcState::Needed,
                        requires_rebind: false,
                        reasons: vec![InvalidationReasonKind::UpstreamPublication],
                    },
                ),
                (
                    TreeNodeId(4),
                    NodeInvalidationRecord {
                        node_id: TreeNodeId(4),
                        calc_state: NodeCalcState::DirtyPending,
                        requires_rebind: true,
                        reasons: vec![
                            InvalidationReasonKind::DependencyAdded,
                            InvalidationReasonKind::DependencyRemoved,
                            InvalidationReasonKind::DependencyReclassified,
                        ],
                    },
                ),
            ]),
        };

        let json = invalidation_closure_json(&closure);
        let records = json.as_array().expect("closure json should be an array");

        assert_eq!(records[0]["reasons"][0], "UpstreamPublication");
        assert_eq!(records[1]["requires_rebind"], true);
        assert_eq!(records[1]["reasons"][0], "DependencyAdded");
        assert_eq!(records[1]["reasons"][1], "DependencyRemoved");
        assert_eq!(records[1]["reasons"][2], "DependencyReclassified");
    }

    #[test]
    fn invalidation_seed_json_preserves_non_structural_reason() {
        let json = invalidation_seed_json(&InvalidationSeed {
            node_id: TreeNodeId(4),
            reason: InvalidationReasonKind::DependencyReclassified,
        });

        assert_eq!(json["node_id"], 4);
        assert_eq!(json["reason"], "DependencyReclassified");
    }

    #[test]
    fn treecalc_runner_emits_local_run_artifacts() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let run_id = "test-treecalc-local-run";
        let artifact_root = repo_root.join(format!(
            "docs/test-runs/core-engine/treecalc-local/{run_id}"
        ));
        if artifact_root.exists() {
            fs::remove_dir_all(&artifact_root).unwrap();
        }

        let runner = TreeCalcRunner::new();
        let summary = runner.execute_manifest(&repo_root, run_id).unwrap();

        assert_eq!(summary.case_count, 13);
        assert_eq!(summary.expectation_mismatch_count, 0);
        assert!(artifact_root.join("run_summary.json").exists());
        assert!(artifact_root.join("case_index.json").exists());
        assert!(
            artifact_root
                .join("conformance/oracle_baseline.json")
                .exists()
        );
        assert!(artifact_root.join("conformance/engine_diff.json").exists());
        assert!(
            artifact_root
                .join("cases/tc_local_publish_001/result.json")
                .exists()
        );
        assert!(
            artifact_root
                .join("cases/tc_local_publish_001/oracle.json")
                .exists()
        );
        assert!(
            artifact_root
                .join("cases/tc_local_publish_001/engine_diff.json")
                .exists()
        );
        assert!(
            artifact_root
                .join("cases/tc_local_publish_001/trace.json")
                .exists()
        );
        assert!(
            artifact_root
                .join("cases/tc_local_publish_001/explain.json")
                .exists()
        );
        assert!(
            artifact_root
                .join("cases/tc_local_rebind_after_rename_001/post_edit/result.json")
                .exists()
        );
        assert!(
            artifact_root
                .join("cases/tc_local_rebind_after_rename_001/post_edit/invalidation_seeds.json")
                .exists()
        );
        assert!(
            artifact_root
                .join("cases/tc_local_recalc_after_constant_edit_001/post_edit/result.json")
                .exists()
        );
        assert!(
            artifact_root
                .join("cases/tc_local_move_direct_target_rebind_001/post_edit/result.json")
                .exists()
        );
        assert!(
            artifact_root
                .join("cases/tc_local_remove_direct_target_001/post_edit/result.json")
                .exists()
        );
        assert!(
            artifact_root
                .join("cases/tc_local_recalc_chain_after_constant_edit_001/post_edit/result.json")
                .exists()
        );
        assert!(
            artifact_root
                .join("cases/tc_local_post_edit_host_sensitive_overlay_001/post_edit/runtime_effect_overlays.json")
                .exists()
        );
        assert!(
            artifact_root
                .join("cases/tc_local_mixed_publish_then_post_edit_overlay_001/post_edit/runtime_effect_overlays.json")
                .exists()
        );
        assert!(
            artifact_root
                .join("conformance/explain_index.json")
                .exists()
        );

        let rename_post_edit_result = serde_json::from_str::<serde_json::Value>(
            &fs::read_to_string(
                artifact_root.join("cases/tc_local_rebind_after_rename_001/post_edit/result.json"),
            )
            .unwrap(),
        )
        .unwrap();
        assert!(
            rename_post_edit_result["invalidation_seeds"]
                .as_array()
                .is_some_and(|seeds| seeds
                    .iter()
                    .any(|seed| { seed["reason"] == "StructuralRebindRequired" }))
        );

        let rename_post_edit_explain = serde_json::from_str::<serde_json::Value>(
            &fs::read_to_string(
                artifact_root.join("cases/tc_local_rebind_after_rename_001/post_edit/explain.json"),
            )
            .unwrap(),
        )
        .unwrap();
        assert!(
            rename_post_edit_explain["invalidation_seeds"]
                .as_array()
                .is_some_and(|seeds| seeds
                    .iter()
                    .any(|seed| { seed["reason"] == "StructuralRebindRequired" }))
        );

        let direct_move_post_edit_result = serde_json::from_str::<serde_json::Value>(
            &fs::read_to_string(
                artifact_root
                    .join("cases/tc_local_move_direct_target_rebind_001/post_edit/result.json"),
            )
            .unwrap(),
        )
        .unwrap();
        assert!(
            direct_move_post_edit_result["invalidation_seeds"]
                .as_array()
                .is_some_and(|seeds| {
                    !seeds.is_empty()
                        && seeds
                            .iter()
                            .all(|seed| seed["reason"] == "StructuralRecalcOnly")
                })
        );

        let direct_move_post_edit_explain = serde_json::from_str::<serde_json::Value>(
            &fs::read_to_string(
                artifact_root
                    .join("cases/tc_local_move_direct_target_rebind_001/post_edit/explain.json"),
            )
            .unwrap(),
        )
        .unwrap();
        assert!(
            direct_move_post_edit_explain["invalidation_seeds"]
                .as_array()
                .is_some_and(|seeds| {
                    !seeds.is_empty()
                        && seeds
                            .iter()
                            .all(|seed| seed["reason"] == "StructuralRecalcOnly")
                })
        );

        let published_result = serde_json::from_str::<serde_json::Value>(
            &fs::read_to_string(artifact_root.join("cases/tc_local_publish_001/result.json"))
                .unwrap(),
        )
        .unwrap();
        let published_dependency_graph = serde_json::from_str::<serde_json::Value>(
            &fs::read_to_string(
                artifact_root.join("cases/tc_local_publish_001/dependency_graph.json"),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(
            published_dependency_graph["descriptors"][0]["descriptor_id"],
            "bind:formula:b:oxfml_ref:0"
        );
        assert_eq!(
            published_dependency_graph["descriptors"][0]["owner_node_id"],
            3
        );
        assert_eq!(
            published_dependency_graph["descriptors"][0]["target_node_id"],
            2
        );
        assert_eq!(
            published_dependency_graph["descriptors"][0]["kind"],
            "StaticDirect"
        );
        assert_eq!(
            published_dependency_graph["descriptors"][0]["carrier_detail"],
            "direct_node:node:2"
        );
        assert_eq!(
            published_dependency_graph["descriptors"][0]["requires_rebind_on_structural_change"],
            false
        );
        assert_eq!(
            published_result["candidate_result"]["aligned_canonical_family"],
            "AcceptedCandidateResult"
        );
        assert_eq!(
            published_result["candidate_result"]["projection_owner"],
            "oxcalc_local"
        );
        assert_eq!(
            published_result["publication_bundle"]["aligned_canonical_family"],
            "CommitBundle"
        );
        assert_eq!(
            published_result["publication_bundle"]["projection_owner"],
            "oxcalc_local"
        );
        assert_eq!(
            published_result["candidate_result"]["candidate_result_id"],
            published_result["publication_bundle"]["candidate_result_id"]
        );
        assert!(
            published_result["publication_bundle"]["publication_id"]
                .as_str()
                .is_some_and(|value| !value.is_empty())
        );
        assert_eq!(
            published_result["candidate_result"]["candidate_result_id"]
                .as_str()
                .is_some_and(|value| !value.is_empty()),
            true
        );
        assert_eq!(
            published_result["publication_bundle"]["published_view_delta"]
                .as_object()
                .map(|entries| entries.len()),
            Some(1)
        );
        assert_eq!(
            published_result["publication_bundle"]["trace_markers"][0],
            "publication_committed"
        );
        assert!(
            published_result["publication_bundle"]
                .get("commit_attempt_id")
                .is_none()
        );
        assert!(published_result["reject_detail"].is_null());
        assert_eq!(
            published_result["publication_bundle"]["carriage_classification"]["publish_critical_categories"]
                [0],
            "value_delta"
        );
        assert_eq!(
            published_result["publication_bundle"]["carriage_classification"]["replay_visible_non_publish_critical_categories"]
                [0],
            "published_runtime_effects"
        );
        assert_eq!(
            published_result["publication_bundle"]["carriage_classification"]["replay_visible_non_publish_critical_categories"]
                [1],
            "trace_markers"
        );
        assert_eq!(
            published_result["publication_bundle"]["carriage_classification"]["local_floor_only_categories"]
                [0],
            "dependency_shape_updates"
        );
        assert_eq!(
            published_result["publication_bundle"]["carriage_classification"]["explicit_current_absence_categories"]
                [0],
            "shape_delta"
        );
        assert_eq!(
            published_result["publication_bundle"]["carriage_classification"]["explicit_current_absence_categories"]
                [1],
            "topology_delta"
        );
        assert_eq!(
            published_result["publication_bundle"]["carriage_classification"]["explicit_current_absence_categories"]
                [2],
            "format_delta"
        );
        assert_eq!(
            published_result["publication_bundle"]["carriage_classification"]["explicit_current_absence_categories"]
                [3],
            "display_delta"
        );
        assert_eq!(
            published_result["publication_bundle"]["carriage_classification"]["dependency_shape_update_count"],
            0
        );
        assert_eq!(
            published_result["execution_restriction_interaction"]["publication_sensitive_consequence"],
            false
        );

        let published_explain = serde_json::from_str::<serde_json::Value>(
            &fs::read_to_string(artifact_root.join("cases/tc_local_publish_001/explain.json"))
                .unwrap(),
        )
        .unwrap();
        assert_eq!(
            published_explain["publication_bundle"]["published_value_delta_node_count"],
            1
        );
        assert_eq!(
            published_explain["publication_bundle"]["published_runtime_effect_count"],
            0
        );
        assert_eq!(
            published_explain["publication_bundle"]["trace_marker_count"],
            1
        );
        assert_eq!(
            published_explain["publication_bundle"]["carriage_classification"]["publish_critical_categories"]
                [0],
            "value_delta"
        );
        assert_eq!(
            published_explain["publication_bundle"]["carriage_classification"]["replay_visible_non_publish_critical_categories"]
                [0],
            "published_runtime_effects"
        );
        assert_eq!(
            published_explain["publication_bundle"]["carriage_classification"]["local_floor_only_categories"]
                [0],
            "dependency_shape_updates"
        );
        assert_eq!(
            published_explain["publication_bundle"]["carriage_classification"]["explicit_current_absence_categories"]
                [0],
            "shape_delta"
        );
        assert_eq!(
            published_explain["publication_bundle"]["carriage_classification"]["explicit_current_absence_categories"]
                [1],
            "topology_delta"
        );
        assert_eq!(
            published_explain["publication_bundle"]["carriage_classification"]["explicit_current_absence_categories"]
                [2],
            "format_delta"
        );
        assert_eq!(
            published_explain["publication_bundle"]["carriage_classification"]["explicit_current_absence_categories"]
                [3],
            "display_delta"
        );
        assert_eq!(
            published_explain["execution_restriction_interaction"]["publication_outcome"],
            "none_observed"
        );

        let rejected_result = serde_json::from_str::<serde_json::Value>(
            &fs::read_to_string(
                artifact_root.join("cases/tc_local_host_sensitive_reject_001/result.json"),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(
            rejected_result["reject_detail"]["aligned_canonical_family"],
            "RejectRecord"
        );
        assert_eq!(
            rejected_result["reject_detail"]["projection_owner"],
            "oxcalc_local"
        );
        assert_eq!(
            rejected_result["reject_detail"]["kind_owner"],
            "oxcalc_local_projection"
        );
        assert!(
            rejected_result["reject_detail"]["candidate_result_id"]
                .as_str()
                .is_some_and(|value| !value.is_empty())
        );
        assert!(rejected_result.get("publication_id").is_none());
        assert!(
            rejected_result["reject_detail"]
                .get("reject_record_id")
                .is_none()
        );
        assert!(rejected_result["publication_bundle"].is_null());
        assert_eq!(
            rejected_result["execution_restriction_interaction"]["publication_outcome"],
            "rejected_no_publication"
        );
        assert_eq!(
            rejected_result["execution_restriction_interaction"]["publication_sensitive_consequence"],
            false
        );
        assert_eq!(
            rejected_result["execution_restriction_interaction"]["topology_sensitive_consequence"],
            false
        );
        assert!(
            rejected_result["runtime_effects_path"]
                .as_str()
                .unwrap()
                .ends_with("cases/tc_local_host_sensitive_reject_001/runtime_effects.json")
        );
        assert!(
            rejected_result["runtime_effect_overlays_path"]
                .as_str()
                .unwrap()
                .ends_with("cases/tc_local_host_sensitive_reject_001/runtime_effect_overlays.json")
        );

        let runtime_effects = serde_json::from_str::<serde_json::Value>(
            &fs::read_to_string(
                artifact_root.join("cases/tc_local_host_sensitive_reject_001/runtime_effects.json"),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(runtime_effects[0]["kind_owner"], "oxcalc_local_projection");
        assert_eq!(
            runtime_effects[0]["family_owner"],
            "oxcalc_local_projection"
        );
        assert_eq!(runtime_effects[0]["family"], "ExecutionRestriction");

        let dynamic_runtime_effects = serde_json::from_str::<serde_json::Value>(
            &fs::read_to_string(
                artifact_root.join("cases/tc_local_dynamic_reject_001/runtime_effects.json"),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(
            dynamic_runtime_effects[0]["kind_owner"],
            "oxcalc_local_projection"
        );
        assert_eq!(
            dynamic_runtime_effects[0]["family_owner"],
            "oxcalc_local_projection"
        );
        assert_eq!(dynamic_runtime_effects[0]["family"], "DynamicDependency");

        let host_sensitive_explain = serde_json::from_str::<serde_json::Value>(
            &fs::read_to_string(
                artifact_root.join("cases/tc_local_host_sensitive_reject_001/explain.json"),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(
            host_sensitive_explain["runtime_effects"][0]["family"],
            "ExecutionRestriction"
        );
        assert_eq!(
            host_sensitive_explain["runtime_effect_overlays"][0]["overlay_kind"],
            "ExecutionRestriction"
        );
        assert_eq!(
            host_sensitive_explain["execution_restriction_interaction"]["publication_outcome"],
            "rejected_no_publication"
        );
        assert_eq!(
            host_sensitive_explain["execution_restriction_interaction"]["publication_sensitive_consequence"],
            false
        );
        assert_eq!(
            host_sensitive_explain["execution_restriction_interaction"]["topology_sensitive_consequence"],
            false
        );
        assert!(host_sensitive_explain["publication_bundle"].is_null());

        let dynamic_explain = serde_json::from_str::<serde_json::Value>(
            &fs::read_to_string(
                artifact_root.join("cases/tc_local_dynamic_reject_001/explain.json"),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(
            dynamic_explain["runtime_effects"][0]["family"],
            "DynamicDependency"
        );
        assert_eq!(
            dynamic_explain["runtime_effect_overlays"][0]["overlay_kind"],
            "DynamicDependency"
        );
        assert_eq!(
            dynamic_explain["execution_restriction_interaction"]["publication_outcome"],
            "none_observed"
        );

        fs::remove_dir_all(&artifact_root).unwrap();
    }
}
