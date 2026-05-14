#![forbid(unsafe_code)]

//! Local TreeCalc fixture runner and artifact emission.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use serde_json::{Value, json};
use thiserror::Error;

use crate::coordinator::{
    AcceptedCandidateResult, DependencyShapeUpdate, RejectDetail, RuntimeEffect,
    RuntimeEffectFamily, TreeCalcCoordinator,
};
use crate::dependency::{
    DependencyDiagnostic, DependencyEdge, InvalidationClosure, InvalidationSeed,
};
use crate::recalc::{NodeCalcState, OverlayEntry, OverlayKind, Stage1RecalcTracker};
use crate::rich_value_capability::RichValueCapabilityTraceReplayColumns;
use crate::structural::{
    StructuralNode, StructuralNodeKind, StructuralSnapshot, StructuralSnapshotId, TreeNodeId,
};
use crate::treecalc::{
    LocalTreeCalcRunArtifacts, LocalTreeCalcRunState, PreparedFormulaIdentityTrace,
};
use crate::treecalc_fixture::{
    TreeCalcFixtureError, TreeCalcFixtureExecution, TreeCalcFixtureExpected,
    TreeCalcFixturePostEditExecution, execute_fixture_case, load_case, load_manifest,
};

const TREECALC_RUN_MANIFEST_SCHEMA_V1: &str = "oxcalc.treecalc.local_run_manifest.v1";
const TREECALC_RUN_SUMMARY_SCHEMA_V1: &str = "oxcalc.treecalc.local_run_summary.v1";
const TREECALC_LOCAL_TRACE_SCHEMA_V1: &str = "oxcalc.treecalc.local_trace.v1";
const TREECALC_LOCAL_EXPLAIN_SCHEMA_V1: &str = "oxcalc.treecalc.local_explain.v1";
const TREECALC_SESSION_PATH_EVIDENCE_SCHEMA_V1: &str = "oxcalc.treecalc.session_path_evidence.v1";
const TREECALC_REPLAY_ARTIFACT_MANIFEST_SCHEMA_V1: &str =
    "oxcalc.treecalc.replay_artifact_manifest.v1";
const TREECALC_MEASUREMENT_COUNTER_SUMMARY_SCHEMA_V1: &str =
    "oxcalc.treecalc.measurement_counter_summary.v1";
const TREECALC_RETENTION_GUARDRAIL_SCHEMA_V1: &str = "oxcalc.treecalc.retention_guardrail.v1";
const TREECALC_TYPED_REJECT_TAXONOMY_SCHEMA_V1: &str = "oxcalc.treecalc.typed_reject_taxonomy.v1";
const TREECALC_HOST_CONTEXT_WATCH_SCHEMA_V1: &str = "oxcalc.treecalc.host_context_watch.v1";
const TREECALC_OVERLAY_ECONOMICS_SCHEMA_V1: &str = "oxcalc.treecalc.overlay_economics.v1";
const TREECALC_REPLAY_APPLIANCE_BUNDLE_SCHEMA_V1: &str =
    "oxcalc.treecalc.replay_appliance_bundle.v1";
const TREECALC_REPLAY_APPLIANCE_RUN_SCHEMA_V1: &str = "oxcalc.treecalc.replay_appliance_run.v1";
const TREECALC_REPLAY_APPLIANCE_VALIDATION_SCHEMA_V1: &str =
    "oxcalc.treecalc.replay_appliance_validation.v1";
const TREECALC_REPLAY_ADAPTER_CAPABILITY_SCHEMA_V1: &str =
    "oxcalc.treecalc.replay_adapter_capability.v1";

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
    #[error("failed to build residual evidence: {0}")]
    ResidualEvidence(String),
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
        let mut case_counter_sets = Vec::new();
        let mut case_phase_timing_sets = Vec::new();
        let mut session_path_evidence_entries = Vec::new();

        for entry in &manifest.cases {
            let case_path = repo_root
                .join("docs/test-fixtures/core-engine/treecalc")
                .join(entry.path.replace('/', "\\"));
            let case = load_case(&case_path)?;
            let execution = execute_fixture_case(&engine, &case)?;
            let artifacts = &execution.initial_artifacts;
            session_path_evidence_entries.push(session_path_evidence_entry_json(
                &case.case_id,
                "initial",
                artifacts,
                &relative_artifact_root,
                "result.json",
                "trace.json",
            ));
            case_phase_timing_sets.push((
                format!("{}:initial", case.case_id),
                artifacts.phase_timings_micros.clone(),
            ));
            if let Some(post_edit_execution) = &execution.post_edit {
                session_path_evidence_entries.push(session_path_evidence_entry_json(
                    &case.case_id,
                    "post_edit",
                    &post_edit_execution.rerun_artifacts,
                    &relative_artifact_root,
                    "post_edit/result.json",
                    "post_edit/trace.json",
                ));
                case_phase_timing_sets.push((
                    format!("{}:post_edit", case.case_id),
                    post_edit_execution
                        .rerun_artifacts
                        .phase_timings_micros
                        .clone(),
                ));
            }
            let case_counters = treecalc_case_counters(artifacts);
            let case_directory = artifact_root.join("cases").join(&entry.case_id);
            create_directory(&case_directory)?;
            let case_artifact_paths = write_case_artifacts(
                &case_directory,
                &relative_artifact_root,
                &case,
                &execution,
                &case_counters,
            )?;
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
                &case_counters,
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
                "counters": counter_entries_json(&case_counters),
            }));
            case_counter_sets.push((case.case_id.clone(), case_counters));
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

        let (retention_guardrail, retention_counters) = retention_guardrail_evidence_json()?;
        write_json(
            &artifact_root.join("retention_guardrail.json"),
            &retention_guardrail,
        )?;
        write_json(
            &artifact_root.join("measurement_counter_summary.json"),
            &measurement_counter_summary_json(&case_counter_sets, &retention_counters),
        )?;
        write_json(
            &artifact_root.join("phase_timing_summary.json"),
            &phase_timing_summary_json(&case_phase_timing_sets),
        )?;
        write_json(
            &artifact_root.join("typed_reject_taxonomy.json"),
            &typed_reject_taxonomy_json(&case_counter_sets),
        )?;
        write_json(
            &artifact_root.join("host_context_watch.json"),
            &host_context_watch_json(),
        )?;
        write_json(
            &artifact_root.join("overlay_economics_summary.json"),
            &overlay_economics_summary_json(&case_counter_sets, &retention_counters),
        )?;
        write_json(
            &artifact_root.join("session_path_evidence.json"),
            &session_path_evidence_json(
                run_id,
                &relative_artifact_root,
                &session_path_evidence_entries,
            ),
        )?;
        write_replay_appliance_projection(
            repo_root,
            &artifact_root,
            run_id,
            &relative_artifact_root,
            &case_results,
        )?;

        write_json(
            &artifact_root.join("replay_artifact_manifest.json"),
            &replay_artifact_manifest_json(
                run_id,
                &relative_artifact_root,
                manifest.cases.len(),
                &case_results,
            ),
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
                "phase_timing_summary_path": format!("{relative_artifact_root}/phase_timing_summary.json"),
            }),
        )?;

        Ok(summary)
    }
}

fn replay_artifact_manifest_json(
    run_id: &str,
    relative_artifact_root: &str,
    case_count: usize,
    case_results: &[serde_json::Value],
) -> serde_json::Value {
    json!({
        "schema_version": TREECALC_REPLAY_ARTIFACT_MANIFEST_SCHEMA_V1,
        "run_id": run_id,
        "artifact_root": relative_artifact_root,
        "case_count": case_count,
        "required_root_artifacts": [
            "manifest_selection.json",
            "run_summary.json",
            "case_index.json",
            "replay_artifact_manifest.json",
            "measurement_counter_summary.json",
            "phase_timing_summary.json",
            "session_path_evidence.json",
            "retention_guardrail.json",
            "typed_reject_taxonomy.json",
            "host_context_watch.json",
            "overlay_economics_summary.json",
            "replay-appliance/bundle_manifest.json",
            "replay-appliance/adapter_capabilities/oxcalc_treecalc.json",
            "replay-appliance/validation/bundle_validation.json",
            format!("replay-appliance/runs/{run_id}/run_manifest.json"),
            "conformance/oracle_baseline.json",
            "conformance/engine_diff.json",
            "conformance/conformance_summary.json",
            "conformance/explain_index.json"
        ],
        "case_artifact_families": [
            "input_case",
            "result",
            "published_values",
            "dependency_graph",
            "invalidation_closure",
            "runtime_effects",
            "runtime_effect_overlays",
            "counters",
            "node_states",
            "phase_timings",
            "oracle",
            "engine_diff",
            "trace",
            "explain"
        ],
        "cases": case_results
            .iter()
            .map(|case_result| {
                json!({
                    "case_id": case_result["case_id"],
                    "result_state": case_result["result_state"],
                    "conformance_state": case_result["conformance_state"],
                    "artifact_paths": case_result["artifact_paths"],
                    "conformance_artifact_paths": case_result["conformance_artifact_paths"],
                    "supporting_artifact_paths": case_result["supporting_artifact_paths"],
                    "post_edit_artifact_paths": case_result["artifact_paths"]["post_edit"],
                })
            })
            .collect::<Vec<_>>(),
    })
}

fn session_path_evidence_json(
    run_id: &str,
    relative_artifact_root: &str,
    entries: &[Value],
) -> Value {
    json!({
        "schema_version": TREECALC_SESSION_PATH_EVIDENCE_SCHEMA_V1,
        "run_id": run_id,
        "artifact_root": relative_artifact_root,
        "artifact_root_declaration": {
            "declared_root": relative_artifact_root,
            "runner": "oxcalc-tracecalc-cli treecalc <run-id>",
            "root_policy": "checked_in_when_artifact_root_is_committed_else_ephemeral"
        },
        "evidence_policy": {
            "retention": "checked_in_or_explicit_ephemeral",
            "checked_in_root_required_for_b8": true,
            "ephemeral_policy": "allowed only when the run command, run_id, and non-mutation validation are recorded in the governing workset/spec note"
        },
        "commands": [
            format!("cargo run -p oxcalc-tracecalc-cli -- treecalc {run_id}"),
            "cargo test -p oxcalc-core treecalc_runner_emits_local_run_artifacts",
            "git diff --check"
        ],
        "entry_count": entries.len(),
        "entries": entries,
    })
}

fn session_path_evidence_entry_json(
    case_id: &str,
    phase: &str,
    artifacts: &LocalTreeCalcRunArtifacts,
    relative_artifact_root: &str,
    result_artifact: &str,
    trace_artifact: &str,
) -> Value {
    let diagnostics = &artifacts.diagnostics;
    let oxcalc_candidate_result_id = artifacts
        .candidate_result
        .as_ref()
        .map(|candidate| candidate.candidate_result_id.clone())
        .or_else(|| {
            artifacts
                .local_candidate
                .as_ref()
                .map(|candidate| candidate.candidate_result_id.clone())
        });
    let publication_candidate_result_id = artifacts
        .publication_bundle
        .as_ref()
        .map(|bundle| bundle.candidate_result_id.clone());
    let reject_candidate_result_id = artifacts
        .reject_detail
        .as_ref()
        .map(|reject| reject.candidate_result_id.clone());
    let candidate_publication_id_match = match (
        oxcalc_candidate_result_id.as_deref(),
        publication_candidate_result_id.as_deref(),
    ) {
        (Some(candidate_id), Some(publication_candidate_id)) => {
            Some(candidate_id == publication_candidate_id)
        }
        _ => None,
    };

    json!({
        "case_id": case_id,
        "phase": phase,
        "result_state": result_state_name(&artifacts.result_state),
        "artifact_paths": {
            "result": relative_case_artifact_path(relative_artifact_root, case_id, result_artifact),
            "trace": relative_case_artifact_path(relative_artifact_root, case_id, trace_artifact),
        },
        "candidate_result_keys": {
            "oxcalc_candidate_result_id": oxcalc_candidate_result_id,
            "oxfml_candidate_result_ids": diagnostic_values(diagnostics, "oxfml_candidate_result_id:"),
            "oxfml_candidate_trace_correlation_ids": diagnostic_values(diagnostics, "oxfml_candidate_trace_correlation_id:"),
            "oxfml_candidate_value_delta_candidate_result_ids": diagnostic_values(diagnostics, "oxfml_candidate_value_delta_candidate_result_id:"),
        },
        "commit_correlation_keys": {
            "oxcalc_publication_candidate_result_id": publication_candidate_result_id,
            "candidate_publication_id_match": candidate_publication_id_match,
            "oxfml_commit_candidate_result_ids": diagnostic_values(diagnostics, "oxfml_commit_candidate_result_id:"),
            "oxfml_commit_attempt_ids": diagnostic_values(diagnostics, "oxfml_commit_attempt_id:"),
            "oxfml_commit_value_delta_candidate_result_ids": diagnostic_values(diagnostics, "oxfml_commit_value_delta_candidate_result_id:"),
        },
        "reject_correlation_keys": {
            "oxcalc_reject_candidate_result_id": reject_candidate_result_id,
            "oxfml_reject_commit_attempt_ids": diagnostic_values(diagnostics, "oxfml_reject_commit_attempt_id:"),
            "oxfml_reject_trace_correlation_ids": diagnostic_values(diagnostics, "oxfml_reject_trace_correlation_id:"),
        },
        "returned_value_surface_diagnostics": diagnostics_with_prefix(diagnostics, "oxfml_returned_value_surface_"),
        "replay_facing_diagnostics": replay_facing_diagnostics(diagnostics),
        "non_mutation_validation": {
            "published_has_publication_bundle": artifacts.publication_bundle.is_some(),
            "rejected_has_no_publication_bundle": artifacts.reject_detail.is_some() && artifacts.publication_bundle.is_none(),
            "verified_clean_has_no_publication_bundle": artifacts.result_state == LocalTreeCalcRunState::VerifiedClean && artifacts.publication_bundle.is_none(),
        },
    })
}

fn diagnostic_values(diagnostics: &[String], prefix: &str) -> Vec<String> {
    diagnostics
        .iter()
        .filter_map(|diagnostic| diagnostic.strip_prefix(prefix).map(str::to_string))
        .collect()
}

fn diagnostics_with_prefix(diagnostics: &[String], prefix: &str) -> Vec<String> {
    diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.starts_with(prefix))
        .cloned()
        .collect()
}

fn replay_facing_diagnostics(diagnostics: &[String]) -> Vec<String> {
    diagnostics
        .iter()
        .filter(|diagnostic| {
            diagnostic.starts_with("oxfml_")
                || diagnostic.starts_with("candidate_rejected:")
                || diagnostic.starts_with("runtime_effect_environment_")
        })
        .cloned()
        .collect()
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
    case_counters: &[(String, i64)],
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
        case_directory.join("counters.json").as_path(),
        &json!({
            "case_id": case.case_id,
            "schema_ref": "formal/measurement/stage1_counter_schema.json",
            "counter_scope": "treecalc_local_case",
            "counters": counter_entries_json(case_counters),
        }),
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
        case_directory.join("phase_timings.json").as_path(),
        &phase_timings_json(artifacts),
    )?;
    write_json(
        case_directory.join("result.json").as_path(),
        &json!({
            "case_id": case.case_id,
            "result_state": result_state_name(&artifacts.result_state),
            "evaluation_order": artifacts.evaluation_order.iter().map(|node_id| node_id.0).collect::<Vec<_>>(),
            "diagnostics": artifacts.diagnostics,
            "prepared_formula_identities": artifacts.prepared_formula_identities.iter().map(prepared_formula_identity_json).collect::<Vec<_>>(),
            "derivation_traces": &artifacts.derivation_traces,
            "published_values_path": relative_case_artifact_path(relative_artifact_root, &case.case_id, "published_values.json"),
            "runtime_effects_path": relative_case_artifact_path(relative_artifact_root, &case.case_id, "runtime_effects.json"),
            "runtime_effect_overlays_path": relative_case_artifact_path(relative_artifact_root, &case.case_id, "runtime_effect_overlays.json"),
            "counters_path": relative_case_artifact_path(relative_artifact_root, &case.case_id, "counters.json"),
            "dependency_graph_path": relative_case_artifact_path(relative_artifact_root, &case.case_id, "dependency_graph.json"),
            "invalidation_closure_path": relative_case_artifact_path(relative_artifact_root, &case.case_id, "invalidation_closure.json"),
            "node_states_path": relative_case_artifact_path(relative_artifact_root, &case.case_id, "node_states.json"),
            "phase_timings_path": relative_case_artifact_path(relative_artifact_root, &case.case_id, "phase_timings.json"),
            "reject_detail": artifacts.reject_detail.as_ref().map(reject_detail_json),
            "candidate_result": artifacts.candidate_result.as_ref().map(|candidate_result| json!({
                "aligned_canonical_family": "AcceptedCandidateResult",
                "projection_owner": "oxcalc_local",
                "candidate_result_id": candidate_result.candidate_result_id,
                "target_set": candidate_result.target_set.iter().map(|node_id| node_id.0).collect::<Vec<_>>(),
                "value_updates": candidate_result.value_updates.iter().map(|(node_id, value)| (node_id.0.to_string(), value.clone())).collect::<BTreeMap<_, _>>(),
                "dependency_shape_updates": candidate_result.dependency_shape_updates.iter().map(dependency_shape_update_json).collect::<Vec<_>>(),
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
        "counters": relative_case_artifact_path(relative_artifact_root, &case.case_id, "counters.json"),
        "dependency_graph": relative_case_artifact_path(relative_artifact_root, &case.case_id, "dependency_graph.json"),
        "invalidation_closure": relative_case_artifact_path(relative_artifact_root, &case.case_id, "invalidation_closure.json"),
        "node_states": relative_case_artifact_path(relative_artifact_root, &case.case_id, "node_states.json"),
        "phase_timings": relative_case_artifact_path(relative_artifact_root, &case.case_id, "phase_timings.json"),
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
    let post_edit_counters = treecalc_case_counters(&execution.rerun_artifacts);

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
        post_edit_directory.join("counters.json").as_path(),
        &json!({
            "case_id": case.case_id,
            "phase": "post_edit",
            "schema_ref": "formal/measurement/stage1_counter_schema.json",
            "counter_scope": "treecalc_local_post_edit_case",
            "counters": counter_entries_json(&post_edit_counters),
        }),
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
        post_edit_directory
            .join("invalidation_closure.json")
            .as_path(),
        &invalidation_closure_json(&execution.rerun_artifacts.invalidation_closure),
    )?;
    write_json(
        post_edit_directory.join("phase_timings.json").as_path(),
        &phase_timings_json(&execution.rerun_artifacts),
    )?;
    write_json(
        post_edit_directory.join("result.json").as_path(),
        &json!({
            "case_id": case.case_id,
            "result_state": result_state_name(&execution.rerun_artifacts.result_state),
            "evaluation_order": execution.rerun_artifacts.evaluation_order.iter().map(|node_id| node_id.0).collect::<Vec<_>>(),
            "reject_detail": execution.rerun_artifacts.reject_detail.as_ref().map(reject_detail_json),
            "prepared_formula_identities": execution.rerun_artifacts.prepared_formula_identities.iter().map(prepared_formula_identity_json).collect::<Vec<_>>(),
            "invalidation_seeds": execution.invalidation_seeds.iter().map(invalidation_seed_json).collect::<Vec<_>>(),
            "invalidation_closure_path": relative_case_artifact_path(relative_artifact_root, &case.case_id, "post_edit/invalidation_closure.json"),
            "counters": counter_entries_json(&post_edit_counters),
            "runtime_effects": execution.rerun_artifacts.runtime_effects.iter().map(runtime_effect_json).collect::<Vec<_>>(),
            "runtime_effect_overlays": execution.rerun_artifacts.runtime_effect_overlays.iter().map(overlay_json).collect::<Vec<_>>(),
            "published_values": execution.rerun_artifacts.published_values.iter().map(|(node_id, value)| (node_id.0.to_string(), value.clone())).collect::<BTreeMap<_, _>>(),
            "candidate_result": execution.rerun_artifacts.candidate_result.as_ref().map(|candidate_result| json!({
                "aligned_canonical_family": "AcceptedCandidateResult",
                "projection_owner": "oxcalc_local",
                "candidate_result_id": candidate_result.candidate_result_id,
                "target_set": candidate_result.target_set.iter().map(|node_id| node_id.0).collect::<Vec<_>>(),
                "value_updates": candidate_result.value_updates.iter().map(|(node_id, value)| (node_id.0.to_string(), value.clone())).collect::<BTreeMap<_, _>>(),
                "dependency_shape_updates": candidate_result.dependency_shape_updates.iter().map(dependency_shape_update_json).collect::<Vec<_>>(),
                "runtime_effects": candidate_result.runtime_effects.iter().map(runtime_effect_json).collect::<Vec<_>>(),
            })),
            "publication_bundle": execution.rerun_artifacts.publication_bundle.as_ref().map(|publication_bundle| json!({
                "aligned_canonical_family": "CommitBundle",
                "projection_owner": "oxcalc_local",
                "publication_id": publication_bundle.publication_id,
                "candidate_result_id": publication_bundle.candidate_result_id,
                "published_view_delta": publication_bundle.published_view_delta.iter().map(|(node_id, value)| (node_id.0.to_string(), value.clone())).collect::<BTreeMap<_, _>>(),
                "published_runtime_effects": publication_bundle.published_runtime_effects.iter().map(runtime_effect_json).collect::<Vec<_>>(),
                "trace_markers": publication_bundle.trace_markers,
                "carriage_classification": publication_carriage_classification_json(&execution.rerun_artifacts),
            })),
            "phase_timings_path": relative_case_artifact_path(relative_artifact_root, &case.case_id, "post_edit/phase_timings.json"),
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
            "counters": counter_entries_json(&post_edit_counters),
            "runtime_effects": execution.rerun_artifacts.runtime_effects.iter().map(runtime_effect_json).collect::<Vec<_>>(),
            "runtime_effect_overlays": execution.rerun_artifacts.runtime_effect_overlays.iter().map(overlay_json).collect::<Vec<_>>(),
            "publication_bundle": execution.rerun_artifacts.publication_bundle.as_ref().map(|publication_bundle| json!({
                "aligned_canonical_family": "CommitBundle",
                "projection_owner": "oxcalc_local",
                "publication_id": publication_bundle.publication_id,
                "candidate_result_id": publication_bundle.candidate_result_id,
                "published_value_delta_node_count": publication_bundle.published_view_delta.len(),
                "published_runtime_effect_count": publication_bundle.published_runtime_effects.len(),
                "trace_marker_count": publication_bundle.trace_markers.len(),
                "carriage_classification": publication_carriage_classification_json(&execution.rerun_artifacts),
            })),
            "phase_timings": phase_timings_json(&execution.rerun_artifacts),
        }),
    )?;

    Ok(json!({
        "edit_outcomes": relative_case_artifact_path(relative_artifact_root, &case.case_id, "post_edit/edit_outcomes.json"),
        "invalidation_seeds": relative_case_artifact_path(relative_artifact_root, &case.case_id, "post_edit/invalidation_seeds.json"),
        "invalidation_closure": relative_case_artifact_path(relative_artifact_root, &case.case_id, "post_edit/invalidation_closure.json"),
        "counters": relative_case_artifact_path(relative_artifact_root, &case.case_id, "post_edit/counters.json"),
        "runtime_effects": relative_case_artifact_path(relative_artifact_root, &case.case_id, "post_edit/runtime_effects.json"),
        "runtime_effect_overlays": relative_case_artifact_path(relative_artifact_root, &case.case_id, "post_edit/runtime_effect_overlays.json"),
        "phase_timings": relative_case_artifact_path(relative_artifact_root, &case.case_id, "post_edit/phase_timings.json"),
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
    case_counters: &[(String, i64)],
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
            "counters": counter_entries_json(case_counters),
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

    for identity in &artifacts.prepared_formula_identities {
        let mut event = json!({
            "step_id": step_id,
            "label": "prepared_formula_identity",
            "owner_node_id": identity.owner_node_id.0,
            "formula_artifact_id": identity.formula_artifact_id,
            "bind_artifact_id": identity.bind_artifact_id,
            "formula_stable_id": identity.formula_stable_id,
            "prepared_callable_key": identity.prepared_callable_key,
            "shape_key": identity.shape_key,
            "dispatch_skeleton_key": identity.dispatch_skeleton_key,
            "plan_template_key": identity.plan_template_key,
            "hole_binding_fingerprint": identity.hole_binding_fingerprint,
            "template_hole_count": identity.template_hole_count,
        });
        add_rich_value_capability_columns(&mut event, &identity.rich_value_capability_columns);
        events.push(event);
        step_id += 1;
    }

    for node_id in &artifacts.evaluation_order {
        events.push(json!({
            "step_id": step_id,
            "label": "evaluate_node",
            "node_id": node_id.0,
        }));
        step_id += 1;
    }

    for trace in &artifacts.derivation_traces {
        let mut event = json!({
            "step_id": step_id,
            "label": "derivation_trace_recorded",
            "trace_schema_id": trace.trace_schema_id,
            "owner_node_id": trace.owner_node_id.0,
            "formula_stable_id": trace.formula_stable_id,
            "plan_template_key": trace.template_selection.plan_template_key,
            "hole_binding_count": trace.hole_bindings.len(),
            "root_invocation_count": trace.sub_invocation_tree.len(),
            "kernel_returned_value": trace.kernel_returned_value,
        });
        add_rich_value_capability_columns(&mut event, &trace.rich_value_capability_columns);
        events.push(event);
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

    let dependency_shape_updates = artifacts
        .candidate_result
        .as_ref()
        .map(|candidate| candidate.dependency_shape_updates.as_slice())
        .or_else(|| {
            artifacts
                .local_candidate
                .as_ref()
                .map(|candidate| candidate.dependency_shape_updates.as_slice())
        })
        .unwrap_or(&[]);
    for update in dependency_shape_updates {
        events.push(json!({
            "step_id": step_id,
            "label": "dependency_shape_update_observed",
            "kind": update.kind,
            "affected_node_ids": update.affected_node_ids.iter().map(|node_id| node_id.0).collect::<Vec<_>>(),
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

fn prepared_formula_identity_json(identity: &PreparedFormulaIdentityTrace) -> serde_json::Value {
    let mut identity_json = json!({
        "owner_node_id": identity.owner_node_id.0,
        "formula_artifact_id": identity.formula_artifact_id,
        "bind_artifact_id": identity.bind_artifact_id,
        "formula_stable_id": identity.formula_stable_id,
        "prepared_callable_key": identity.prepared_callable_key,
        "shape_key": identity.shape_key,
        "dispatch_skeleton_key": identity.dispatch_skeleton_key,
        "plan_template_key": identity.plan_template_key,
        "hole_binding_fingerprint": identity.hole_binding_fingerprint,
        "template_hole_count": identity.template_hole_count,
    });
    add_rich_value_capability_columns(&mut identity_json, &identity.rich_value_capability_columns);
    identity_json
}

fn add_rich_value_capability_columns(
    value: &mut serde_json::Value,
    columns: &RichValueCapabilityTraceReplayColumns,
) {
    if columns.is_empty() {
        return;
    }

    value["rich_value_capability_columns"] =
        serde_json::to_value(columns).expect("capability columns should serialize");
}

fn dependency_shape_update_json(update: &DependencyShapeUpdate) -> serde_json::Value {
    json!({
        "kind": update.kind,
        "affected_node_ids": update
            .affected_node_ids
            .iter()
            .map(|node_id| node_id.0)
            .collect::<Vec<_>>(),
    })
}

fn phase_timings_json(artifacts: &LocalTreeCalcRunArtifacts) -> serde_json::Value {
    json!({
        "unit": "microseconds",
        "timings_micros": &artifacts.phase_timings_micros,
        "timings_ms": artifacts
            .phase_timings_micros
            .iter()
            .map(|(phase_name, micros)| {
                (phase_name.clone(), (*micros as f64) / 1_000.0)
            })
            .collect::<BTreeMap<_, _>>(),
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

fn treecalc_case_counters(artifacts: &LocalTreeCalcRunArtifacts) -> Vec<(String, i64)> {
    let mut counters = BTreeMap::new();

    if artifacts.local_candidate.is_some() || artifacts.reject_detail.is_some() {
        increment_counter(&mut counters, "candidate_admissions");
    }
    if artifacts.candidate_result.is_some() {
        increment_counter(&mut counters, "accepted_candidate_results");
    }
    if artifacts.publication_bundle.is_some() {
        increment_counter(&mut counters, "publications_committed");
    }
    if let Some(reject_detail) = &artifacts.reject_detail {
        increment_counter(&mut counters, "abandoned_candidates");
        let reject_kind = to_snake_case(&format!("{:?}", reject_detail.kind));
        increment_counter(&mut counters, &format!("rejects_by_class.{reject_kind}"));
        increment_counter(&mut counters, &format!("fallback_by_reason.{reject_kind}"));
        let affected = artifacts
            .local_candidate
            .as_ref()
            .map(|candidate| candidate.target_set.len())
            .unwrap_or_else(|| artifacts.evaluation_order.len());
        add_to_counter(
            &mut counters,
            "fallback_affected_work_volume",
            i64::try_from(affected).unwrap_or(0),
        );
    }

    let work_count = i64::try_from(artifacts.evaluation_order.len()).unwrap_or(0);
    if work_count > 0 {
        add_to_counter(&mut counters, "nodes_marked_dirty", work_count);
        add_to_counter(&mut counters, "nodes_marked_needed", work_count);
    }

    let verified_clean_count = artifacts
        .node_states
        .values()
        .filter(|state| matches!(state, NodeCalcState::VerifiedClean))
        .count();
    if verified_clean_count > 0 {
        add_to_counter(
            &mut counters,
            "verified_clean_nodes",
            i64::try_from(verified_clean_count).unwrap_or(0),
        );
    }

    let overlay_count = i64::try_from(artifacts.runtime_effect_overlays.len()).unwrap_or(0);
    if overlay_count > 0 {
        add_to_counter(&mut counters, "overlay_lookups", overlay_count);
        add_to_counter(&mut counters, "overlay_misses", overlay_count);
        add_to_counter(&mut counters, "overlay_creations", overlay_count);
    }

    counters.into_iter().collect()
}

fn retention_guardrail_evidence_json() -> Result<(Value, Vec<(String, i64)>), TreeCalcRunnerError> {
    let snapshot = StructuralSnapshot::create(
        StructuralSnapshotId(9_031),
        TreeNodeId(1),
        [
            StructuralNode {
                node_id: TreeNodeId(1),
                kind: StructuralNodeKind::Root,
                symbol: "Root".to_string(),
                parent_id: None,
                child_ids: vec![TreeNodeId(2), TreeNodeId(3), TreeNodeId(4)],
                formula_artifact_id: None,
                bind_artifact_id: None,
                constant_value: None,
            },
            StructuralNode {
                node_id: TreeNodeId(2),
                kind: StructuralNodeKind::Constant,
                symbol: "X".to_string(),
                parent_id: Some(TreeNodeId(1)),
                child_ids: vec![],
                formula_artifact_id: None,
                bind_artifact_id: None,
                constant_value: Some("2".to_string()),
            },
            StructuralNode {
                node_id: TreeNodeId(3),
                kind: StructuralNodeKind::Calculation,
                symbol: "Y".to_string(),
                parent_id: Some(TreeNodeId(1)),
                child_ids: vec![],
                formula_artifact_id: None,
                bind_artifact_id: None,
                constant_value: None,
            },
            StructuralNode {
                node_id: TreeNodeId(4),
                kind: StructuralNodeKind::Calculation,
                symbol: "Z".to_string(),
                parent_id: Some(TreeNodeId(1)),
                child_ids: vec![],
                formula_artifact_id: None,
                bind_artifact_id: None,
                constant_value: None,
            },
        ],
    )
    .map_err(|source| TreeCalcRunnerError::ResidualEvidence(source.to_string()))?;

    let mut coordinator = TreeCalcCoordinator::new(snapshot.clone());
    let initial_values = BTreeMap::from([
        (TreeNodeId(2), "2".to_string()),
        (TreeNodeId(3), "40".to_string()),
        (TreeNodeId(4), "42".to_string()),
    ]);
    coordinator.seed_published_view(
        &initial_values,
        Some("treecalc_retention:publication:initial"),
        &[],
    );

    let mut counters = BTreeMap::new();
    let mut events = Vec::new();
    let pinned = coordinator.pin_reader("reader:treecalc-retention");
    increment_counter(&mut counters, "reader.pinned");
    set_counter(
        &mut counters,
        "pinned_reader_count",
        i64::try_from(coordinator.pinned_readers().len()).unwrap_or(0),
    );
    events.push(json!({
        "label": "reader_pinned",
        "reader_id": pinned.reader_id,
        "publication_id": pinned.publication_id,
    }));

    let mut tracker = Stage1RecalcTracker::new(snapshot.clone());
    let owner_node_id = TreeNodeId(4);
    tracker.mark_dirty(owner_node_id);
    increment_counter(&mut counters, "nodes_marked_dirty");
    tracker
        .mark_needed(owner_node_id)
        .map_err(|source| TreeCalcRunnerError::ResidualEvidence(source.to_string()))?;
    increment_counter(&mut counters, "nodes_marked_needed");
    tracker
        .begin_evaluate(owner_node_id, "snapshot:9031")
        .map_err(|source| TreeCalcRunnerError::ResidualEvidence(source.to_string()))?;
    increment_counter(&mut counters, "overlay_lookups");
    increment_counter(&mut counters, "overlay_misses");
    tracker
        .produce_dependency_shape_update(
            owner_node_id,
            "snapshot:9031",
            "treecalc_retention:candidate:updated-z",
        )
        .map_err(|source| TreeCalcRunnerError::ResidualEvidence(source.to_string()))?;
    increment_counter(&mut counters, "overlay_creations");

    let value_updates = BTreeMap::from([(owner_node_id, "63".to_string())]);
    coordinator
        .admit_candidate_work(AcceptedCandidateResult {
            candidate_result_id: "treecalc_retention:candidate:updated-z".to_string(),
            structural_snapshot_id: snapshot.snapshot_id(),
            artifact_token_basis: "snapshot:9031".to_string(),
            compatibility_basis: "snapshot:9031".to_string(),
            target_set: vec![owner_node_id],
            value_updates,
            dependency_shape_updates: vec![DependencyShapeUpdate {
                kind: "retained_dynamic_dependency_guardrail".to_string(),
                affected_node_ids: vec![owner_node_id],
            }],
            runtime_effects: vec![RuntimeEffect {
                kind: "dynamic_ref_activated".to_string(),
                family: RuntimeEffectFamily::DynamicDependency,
                detail: "retention_guardrail".to_string(),
            }],
            diagnostic_events: vec!["treecalc_retention_guardrail".to_string()],
        })
        .map_err(|source| TreeCalcRunnerError::ResidualEvidence(source.to_string()))?;
    coordinator
        .record_accepted_candidate_result("treecalc_retention:candidate:updated-z")
        .map_err(|source| TreeCalcRunnerError::ResidualEvidence(source.to_string()))?;
    increment_counter(&mut counters, "accepted_candidate_results");
    let publication = coordinator
        .accept_and_publish("treecalc_retention:publication:updated-z")
        .map_err(|source| TreeCalcRunnerError::ResidualEvidence(source.to_string()))?;
    increment_counter(&mut counters, "publications_committed");
    tracker
        .publish_and_clear(owner_node_id)
        .map_err(|source| TreeCalcRunnerError::ResidualEvidence(source.to_string()))?;
    events.push(json!({
        "label": "publication_committed",
        "publication_id": publication.publication_id,
        "candidate_result_id": publication.candidate_result_id,
    }));

    let pinned_after_publication = coordinator
        .pinned_readers()
        .into_iter()
        .find(|view| view.reader_id == "reader:treecalc-retention")
        .ok_or_else(|| {
            TreeCalcRunnerError::ResidualEvidence("retention pin disappeared before release".into())
        })?;
    let published_after = coordinator.published_view().values.clone();
    let pinned_stable = pinned_after_publication.values == initial_values
        && published_after.get(&owner_node_id) == Some(&"63".to_string());
    events.push(json!({
        "label": "pinned_view_stability_checked",
        "stable": pinned_stable,
    }));

    let retained_dynamic_overlays = tracker
        .overlays()
        .values()
        .filter(|entry| {
            entry.key.overlay_kind == OverlayKind::DynamicDependency
                && entry.is_protected
                && !entry.is_eviction_eligible
        })
        .count();
    if retained_dynamic_overlays > 0 && !coordinator.pinned_readers().is_empty() {
        increment_counter(&mut counters, "retention_blocked_cleanup");
        add_to_counter(
            &mut counters,
            "overlay.retained",
            i64::try_from(retained_dynamic_overlays).unwrap_or(0),
        );
        events.push(json!({
            "label": "retention_blocked_cleanup",
            "protected_dynamic_overlay_count": retained_dynamic_overlays,
        }));
    }

    if coordinator.unpin_reader("reader:treecalc-retention") {
        increment_counter(&mut counters, "reader.unpinned");
        increment_counter(&mut counters, "release_events");
        set_counter(
            &mut counters,
            "pinned_reader_count",
            i64::try_from(coordinator.pinned_readers().len()).unwrap_or(0),
        );
        events.push(json!({
            "label": "reader_unpinned",
            "reader_id": "reader:treecalc-retention",
        }));
    }

    tracker
        .release_and_evict_eligible(owner_node_id)
        .map_err(|source| TreeCalcRunnerError::ResidualEvidence(source.to_string()))?;
    increment_counter(&mut counters, "eviction_eligibility_opened");
    events.push(json!({
        "label": "eviction_eligibility_opened",
        "owner_node_id": owner_node_id.0,
    }));
    let evicted_count = tracker.evict_eligible_overlays();
    if evicted_count > 0 {
        add_to_counter(
            &mut counters,
            "overlay_evictions",
            i64::try_from(evicted_count).unwrap_or(0),
        );
        events.push(json!({
            "label": "overlay_released",
            "evicted_count": evicted_count,
        }));
    }

    let counter_entries = counters.into_iter().collect::<Vec<_>>();
    Ok((
        json!({
            "schema_version": TREECALC_RETENTION_GUARDRAIL_SCHEMA_V1,
            "evidence_id": "tc_local_pinned_reader_retention_001",
            "description": "TreeCalc-local guardrail over pinned-reader stability, retained dynamic overlays, release, and eviction eligibility.",
            "source_scope": "runner_generated_from_core_coordinator_and_recalc_apis",
            "pinned_reader_stability": {
                "reader_id": "reader:treecalc-retention",
                "stable": pinned_stable,
                "pinned_values_before_publication": value_map_json(&initial_values),
                "pinned_values_after_publication": value_map_json(&pinned_after_publication.values),
                "published_values_after_publication": value_map_json(&published_after),
            },
            "retention": {
                "protected_dynamic_overlay_count_before_release": retained_dynamic_overlays,
                "evicted_overlay_count_after_release": evicted_count,
                "cleanup_blocked_while_reader_pinned": retained_dynamic_overlays > 0,
            },
            "events": events,
            "counters": counter_entries_json(&counter_entries),
            "claims_exercised": [
                "R4.pinned_reader_stability",
                "R5.overlay_retention_release",
                "C2.pinned_reader_and_retention",
                "C4.overlay_economics_eviction"
            ],
        }),
        counter_entries,
    ))
}

fn measurement_counter_summary_json(
    case_counter_sets: &[(String, Vec<(String, i64)>)],
    retention_counters: &[(String, i64)],
) -> Value {
    let aggregate = aggregate_counters(
        case_counter_sets
            .iter()
            .map(|(_, counters)| counters.as_slice())
            .chain(std::iter::once(retention_counters)),
    );
    json!({
        "schema_version": TREECALC_MEASUREMENT_COUNTER_SUMMARY_SCHEMA_V1,
        "schema_ref": "formal/measurement/stage1_counter_schema.json",
        "counter_scope": "treecalc_local_run",
        "case_count": case_counter_sets.len(),
        "counter_families": [
            {
                "family_id": "C1",
                "name": "candidate_and_publication",
                "status": "exercised_by_case_artifacts",
                "counters": counters_with_prefixes(&aggregate, &["candidate_", "accepted_", "publications_", "rejects_by_class", "abandoned_"]),
            },
            {
                "family_id": "C2",
                "name": "pinned_reader_and_retention",
                "status": "exercised_by_retention_guardrail",
                "counters": counters_with_prefixes(&aggregate, &["pinned_reader_", "reader.", "release_events", "retention_blocked_cleanup", "eviction_eligibility_opened"]),
            },
            {
                "family_id": "C3",
                "name": "invalidation_and_fallback",
                "status": "exercised_by_case_artifacts",
                "counters": counters_with_prefixes(&aggregate, &["nodes_marked_", "verified_clean_nodes", "fallback_"]),
            },
            {
                "family_id": "C4",
                "name": "overlay_economics",
                "status": "exercised_by_runtime_effect_and_retention_artifacts",
                "counters": counters_with_prefixes(&aggregate, &["overlay_lookups", "overlay_hits", "overlay_misses", "overlay_creations", "overlay_evictions", "overlay.retained", "overlay_reuse_after_retention"]),
            },
            {
                "family_id": "C5",
                "name": "stage2_reserved",
                "status": "reserved_not_emitted",
                "counters": [],
            }
        ],
        "aggregate_counters": counter_entries_json(&aggregate),
        "case_counter_sets": case_counter_sets.iter().map(|(case_id, counters)| json!({
            "case_id": case_id,
            "counters": counter_entries_json(counters),
        })).collect::<Vec<_>>(),
        "retention_guardrail_counters": counter_entries_json(retention_counters),
    })
}

fn phase_timing_summary_json(case_phase_timing_sets: &[(String, BTreeMap<String, u128>)]) -> Value {
    let mut values_by_phase = BTreeMap::<String, Vec<(String, u128)>>::new();
    for (case_phase_id, timings) in case_phase_timing_sets {
        for (phase_name, micros) in timings {
            values_by_phase
                .entry(phase_name.clone())
                .or_default()
                .push((case_phase_id.clone(), *micros));
        }
    }

    let phases = values_by_phase
        .into_iter()
        .map(|(phase_name, values)| {
            let count = values.len();
            let total_micros = values.iter().map(|(_, micros)| *micros).sum::<u128>();
            let min = values
                .iter()
                .min_by_key(|(_, micros)| *micros)
                .expect("phase values are non-empty");
            let max = values
                .iter()
                .max_by_key(|(_, micros)| *micros)
                .expect("phase values are non-empty");
            json!({
                "phase_name": phase_name,
                "count": count,
                "total_micros": total_micros,
                "total_ms": total_micros as f64 / 1_000.0,
                "min_micros": min.1,
                "min_case_phase": min.0,
                "max_micros": max.1,
                "max_case_phase": max.0,
            })
        })
        .collect::<Vec<_>>();

    json!({
        "schema_version": "oxcalc.treecalc.phase_timing_summary.v1",
        "unit": "microseconds",
        "case_phase_count": case_phase_timing_sets.len(),
        "phases": phases,
        "case_phase_timings": case_phase_timing_sets.iter().map(|(case_phase_id, timings)| json!({
            "case_phase_id": case_phase_id,
            "timings_micros": timings,
        })).collect::<Vec<_>>(),
    })
}

fn typed_reject_taxonomy_json(case_counter_sets: &[(String, Vec<(String, i64)>)]) -> Value {
    let aggregate = aggregate_counters(
        case_counter_sets
            .iter()
            .map(|(_, counters)| counters.as_slice()),
    );
    let observed_reject_kinds = aggregate
        .iter()
        .map(|(counter, _)| counter)
        .filter_map(|counter| counter.strip_prefix("rejects_by_class."))
        .map(str::to_string)
        .collect::<BTreeSet<_>>();
    let declared_reject_kinds = [
        "snapshot_mismatch",
        "artifact_token_mismatch",
        "profile_version_mismatch",
        "capability_mismatch",
        "publication_fence_mismatch",
        "dynamic_dependency_failure",
        "synthetic_cycle_reject",
        "host_injected_failure",
    ];
    let unobserved_declared = declared_reject_kinds
        .iter()
        .filter(|kind| !observed_reject_kinds.contains(**kind))
        .copied()
        .collect::<Vec<_>>();

    json!({
        "schema_version": TREECALC_TYPED_REJECT_TAXONOMY_SCHEMA_V1,
        "source": "treecalc_local_run_counter_artifacts",
        "observed_reject_kinds": observed_reject_kinds.into_iter().collect::<Vec<_>>(),
        "unobserved_declared_reject_kinds": unobserved_declared,
        "watch_lanes": [
            {
                "lane_id": "provider_failure",
                "status": "watch_no_treecalc_local_evidence",
                "handoff_required": false,
            },
            {
                "lane_id": "callable_publication",
                "status": "watch_no_treecalc_local_evidence",
                "handoff_required": false,
            },
            {
                "lane_id": "future_oxfml_runtime_reject_family",
                "status": "watch_no_treecalc_local_evidence",
                "handoff_required": false,
            }
        ],
        "handoff_triggered": false,
        "handoff_basis": "No concrete coordinator-visible seam insufficiency is exposed by this TreeCalc-local run.",
    })
}

fn host_context_watch_json() -> Value {
    json!({
        "schema_version": TREECALC_HOST_CONTEXT_WATCH_SCHEMA_V1,
        "source": "treecalc_local_scope_review",
        "treecalc_local_current_context": {
            "address_modes": ["direct_node", "relative_sibling_offset"],
            "caller_context": "local_structural_parent_and_sibling_context_only",
            "table_context": "not_admitted_to_treecalc_local_runtime_scope",
            "host_sensitive_behavior": "conservative_reject_with_runtime_effect_overlay",
        },
        "related_existing_evidence": [
            "docs/test-fixtures/core-engine/upstream-host/README.md",
            "docs/test-fixtures/core-engine/upstream-host/cases/uh_table_context_bind_001.json",
            "docs/test-fixtures/core-engine/upstream-host/cases/uh_structured_reference_eval_001.json",
            "src/oxcalc-core/tests/upstream_host_scaffolding.rs"
        ],
        "watch_lanes": [
            {
                "lane_id": "caller_table_context",
                "status": "covered_on_upstream_host_surface_not_treecalc_local_formula_scope",
                "handoff_required": false,
            },
            {
                "lane_id": "structured_reference_breadth",
                "status": "watch_until_treecalc_scope_admits_table_semantics",
                "handoff_required": false,
            },
            {
                "lane_id": "host_sensitive_direct_binding",
                "status": "current_treecalc_floor_rejects_conservatively",
                "handoff_required": false,
            }
        ],
        "handoff_triggered": false,
    })
}

fn overlay_economics_summary_json(
    case_counter_sets: &[(String, Vec<(String, i64)>)],
    retention_counters: &[(String, i64)],
) -> Value {
    let aggregate = aggregate_counters(
        case_counter_sets
            .iter()
            .map(|(_, counters)| counters.as_slice())
            .chain(std::iter::once(retention_counters)),
    );
    json!({
        "schema_version": TREECALC_OVERLAY_ECONOMICS_SCHEMA_V1,
        "source": "treecalc_local_counter_and_retention_guardrail_artifacts",
        "overlay_counters": counters_with_prefixes(&aggregate, &[
            "overlay_lookups",
            "overlay_hits",
            "overlay_misses",
            "overlay_creations",
            "overlay_evictions",
            "overlay.retained",
            "overlay_reuse_after_retention",
        ]),
        "fallback_counters": counters_with_prefixes(&aggregate, &[
            "fallback_by_reason",
            "fallback_affected_work_volume",
        ]),
        "retention_counters": counters_with_prefixes(&aggregate, &[
            "retention_blocked_cleanup",
            "eviction_eligibility_opened",
            "release_events",
        ]),
        "economics_reading": {
            "overlay_hits_observed": counter_value(&aggregate, "overlay_hits") > 0,
            "overlay_misses_observed": counter_value(&aggregate, "overlay_misses") > 0,
            "overlay_evictions_observed": counter_value(&aggregate, "overlay_evictions") > 0,
            "fallback_observed": aggregate.iter().any(|(counter, _)| counter.starts_with("fallback_by_reason.")),
            "optimization_claim": "not_promoted",
        },
        "guardrail_artifacts": [
            "retention_guardrail.json",
            "measurement_counter_summary.json"
        ],
    })
}

fn write_replay_appliance_projection(
    repo_root: &Path,
    artifact_root: &Path,
    run_id: &str,
    relative_artifact_root: &str,
    case_results: &[Value],
) -> Result<(), TreeCalcRunnerError> {
    create_directory(&artifact_root.join("replay-appliance"))?;
    create_directory(&artifact_root.join("replay-appliance/adapter_capabilities"))?;
    create_directory(&artifact_root.join("replay-appliance/runs").join(run_id))?;
    create_directory(&artifact_root.join("replay-appliance/validation"))?;

    let adapter_path = relative_artifact_path([
        relative_artifact_root,
        "replay-appliance",
        "adapter_capabilities",
        "oxcalc_treecalc.json",
    ]);
    let run_manifest_path = relative_artifact_path([
        relative_artifact_root,
        "replay-appliance",
        "runs",
        run_id,
        "run_manifest.json",
    ]);
    let bundle_manifest_path = relative_artifact_path([
        relative_artifact_root,
        "replay-appliance",
        "bundle_manifest.json",
    ]);
    let session_path_evidence_path =
        relative_artifact_path([relative_artifact_root, "session_path_evidence.json"]);
    let validation_path = relative_artifact_path([
        relative_artifact_root,
        "replay-appliance",
        "validation",
        "bundle_validation.json",
    ]);

    write_json(
        &artifact_root.join("replay-appliance/adapter_capabilities/oxcalc_treecalc.json"),
        &json!({
            "schema_version": TREECALC_REPLAY_ADAPTER_CAPABILITY_SCHEMA_V1,
            "adapter_id": "oxcalc-treecalc-local-replay-adapter",
            "lane_id": "oxcalc_treecalc_local",
            "run_id": run_id,
            "projection_scope": "treecalc_local_run_snapshot",
            "claimed_capability_levels": [
                "cap.C0.ingest_valid",
                "cap.C1.replay_valid",
                "cap.C2.diff_valid",
                "cap.C3.explain_valid"
            ],
            "target_capability_levels": ["cap.C4.distill_valid", "cap.C5.pack_valid"],
            "known_limits": [
                "treecalc.local.limit.no_stage2_concurrency",
                "treecalc.local.limit.host_table_context_watch_only",
                "treecalc.local.limit.performance_not_promoted"
            ],
        }),
    )?;

    let case_projection = case_results
        .iter()
        .map(|case_result| {
            json!({
                "case_id": case_result["case_id"],
                "result_state": case_result["result_state"],
                "conformance_state": case_result["conformance_state"],
                "source_artifact_paths": {
                    "result": case_result["artifact_paths"]["result"],
                    "trace": case_result["supporting_artifact_paths"]["trace"],
                    "explain": case_result["supporting_artifact_paths"]["explain"],
                    "counters": case_result["artifact_paths"]["counters"],
                    "published_values": case_result["artifact_paths"]["published_values"],
                    "runtime_effect_overlays": case_result["artifact_paths"]["runtime_effect_overlays"],
                    "reject_detail": case_result["artifact_paths"]["result"],
                },
                "required_equality_surfaces": [
                    "published_values",
                    "result_state",
                    "reject_detail",
                    "counter_set",
                    "trace_labels",
                    "runtime_effect_overlays"
                ],
            })
        })
        .collect::<Vec<_>>();

    write_json(
        &artifact_root
            .join("replay-appliance/runs")
            .join(run_id)
            .join("run_manifest.json"),
        &json!({
            "schema_version": TREECALC_REPLAY_APPLIANCE_RUN_SCHEMA_V1,
            "run_kind": "treecalc_local_run",
            "run_id": run_id,
            "source_artifact_root": relative_artifact_root,
            "source_run_summary_path": relative_artifact_path([relative_artifact_root, "run_summary.json"]),
            "source_replay_artifact_manifest_path": relative_artifact_path([relative_artifact_root, "replay_artifact_manifest.json"]),
            "source_measurement_counter_summary_path": relative_artifact_path([relative_artifact_root, "measurement_counter_summary.json"]),
            "source_retention_guardrail_path": relative_artifact_path([relative_artifact_root, "retention_guardrail.json"]),
            "source_session_path_evidence_path": session_path_evidence_path.clone(),
            "cases": case_projection,
        }),
    )?;

    write_json(
        &artifact_root.join("replay-appliance/bundle_manifest.json"),
        &json!({
            "schema_version": TREECALC_REPLAY_APPLIANCE_BUNDLE_SCHEMA_V1,
            "bundle_kind": "treecalc_local_run",
            "lane_id": "oxcalc_treecalc_local",
            "run_id": run_id,
            "source_artifact_root": relative_artifact_root,
            "run_manifest_path": run_manifest_path.clone(),
            "session_path_evidence_path": session_path_evidence_path.clone(),
            "adapter_capabilities_path": adapter_path.clone(),
            "validation_path": validation_path,
            "preserved_view_families": [
                "published_values",
                "reject_set",
                "counter_set",
                "runtime_effect_overlay_set",
                "retention_guardrail",
                "session_correlation_keys",
                "replay_facing_diagnostics"
            ],
            "projection_status": "local_projection_validated",
        }),
    )?;

    let checked_paths = case_results
        .iter()
        .flat_map(|case_result| {
            [
                case_result["artifact_paths"]["result"].as_str(),
                case_result["artifact_paths"]["counters"].as_str(),
                case_result["supporting_artifact_paths"]["trace"].as_str(),
                case_result["supporting_artifact_paths"]["explain"].as_str(),
            ]
        })
        .flatten()
        .map(str::to_string)
        .chain([
            adapter_path,
            run_manifest_path,
            bundle_manifest_path,
            session_path_evidence_path,
            relative_artifact_path([relative_artifact_root, "measurement_counter_summary.json"]),
            relative_artifact_path([relative_artifact_root, "retention_guardrail.json"]),
            relative_artifact_path([relative_artifact_root, "typed_reject_taxonomy.json"]),
            relative_artifact_path([relative_artifact_root, "host_context_watch.json"]),
            relative_artifact_path([relative_artifact_root, "overlay_economics_summary.json"]),
        ])
        .collect::<Vec<_>>();
    let missing_paths = checked_paths
        .iter()
        .filter(|path| !repo_root.join(path).exists())
        .cloned()
        .collect::<Vec<_>>();

    write_json(
        &artifact_root.join("replay-appliance/validation/bundle_validation.json"),
        &json!({
            "schema_version": TREECALC_REPLAY_APPLIANCE_VALIDATION_SCHEMA_V1,
            "bundle_kind": "treecalc_local_run",
            "run_id": run_id,
            "status": if missing_paths.is_empty() { "bundle_valid" } else { "bundle_degraded" },
            "checked_paths": checked_paths,
            "missing_paths": missing_paths,
            "non_mutation_validation": {
                "session_path_evidence_checked": true,
                "checked_artifacts_only": true,
                "publication_mutation_source": "runner_source_artifacts",
            },
        }),
    )
}

fn aggregate_counters<'a>(
    counter_sets: impl IntoIterator<Item = &'a [(String, i64)]>,
) -> Vec<(String, i64)> {
    let mut aggregate = BTreeMap::new();
    for counters in counter_sets {
        for (counter, value) in counters {
            add_to_counter(&mut aggregate, counter, *value);
        }
    }
    aggregate.into_iter().collect()
}

fn counters_with_prefixes(counters: &[(String, i64)], prefixes: &[&str]) -> Vec<Value> {
    counter_entries_json(
        &counters
            .iter()
            .filter(|(counter, _)| prefixes.iter().any(|prefix| counter.starts_with(prefix)))
            .cloned()
            .collect::<Vec<_>>(),
    )
}

fn counter_value(counters: &[(String, i64)], counter: &str) -> i64 {
    counters
        .iter()
        .find_map(|(candidate, value)| (candidate == counter).then_some(*value))
        .unwrap_or(0)
}

fn counter_entries_json(entries: &[(String, i64)]) -> Vec<Value> {
    let mut ordered = entries.to_vec();
    ordered.sort_by(|left, right| left.0.cmp(&right.0));
    ordered
        .into_iter()
        .map(|(counter, value)| json!({ "counter": counter, "value": value }))
        .collect()
}

fn value_map_json(values: &BTreeMap<TreeNodeId, String>) -> Vec<Value> {
    values
        .iter()
        .map(|(node_id, value)| json!({ "node_id": node_id.0, "value": value }))
        .collect()
}

fn increment_counter(counters: &mut BTreeMap<String, i64>, counter: &str) {
    add_to_counter(counters, counter, 1);
}

fn add_to_counter(counters: &mut BTreeMap<String, i64>, counter: &str, value: i64) {
    *counters.entry(counter.to_string()).or_insert(0) += value;
}

fn set_counter(counters: &mut BTreeMap<String, i64>, counter: &str, value: i64) {
    counters.insert(counter.to_string(), value);
}

fn to_snake_case(input: &str) -> String {
    let mut result = String::new();
    for (index, character) in input.chars().enumerate() {
        if character.is_uppercase() {
            if index > 0 {
                result.push('_');
            }
            for lower in character.to_lowercase() {
                result.push(lower);
            }
        } else {
            result.push(character);
        }
    }
    result
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

        assert_eq!(summary.case_count, 37);
        assert_eq!(summary.expectation_mismatch_count, 0);
        assert!(artifact_root.join("run_summary.json").exists());
        assert!(artifact_root.join("case_index.json").exists());
        assert!(artifact_root.join("replay_artifact_manifest.json").exists());
        assert!(artifact_root.join("session_path_evidence.json").exists());
        assert!(
            artifact_root
                .join("measurement_counter_summary.json")
                .exists()
        );
        assert!(artifact_root.join("retention_guardrail.json").exists());
        assert!(artifact_root.join("typed_reject_taxonomy.json").exists());
        assert!(artifact_root.join("host_context_watch.json").exists());
        assert!(
            artifact_root
                .join("overlay_economics_summary.json")
                .exists()
        );
        assert!(
            artifact_root
                .join("replay-appliance/bundle_manifest.json")
                .exists()
        );
        assert!(
            artifact_root
                .join("replay-appliance/adapter_capabilities/oxcalc_treecalc.json")
                .exists()
        );
        assert!(
            artifact_root
                .join("replay-appliance/validation/bundle_validation.json")
                .exists()
        );
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
                .join("cases/tc_local_publish_001/counters.json")
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
                .join("cases/tc_local_post_edit_capability_sensitive_overlay_001/post_edit/runtime_effect_overlays.json")
                .exists()
        );
        assert!(
            artifact_root
                .join("cases/tc_local_post_edit_shape_topology_overlay_001/post_edit/runtime_effect_overlays.json")
                .exists()
        );
        assert!(
            artifact_root
                .join("conformance/explain_index.json")
                .exists()
        );

        let replay_manifest = serde_json::from_str::<serde_json::Value>(
            &fs::read_to_string(artifact_root.join("replay_artifact_manifest.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(
            replay_manifest["schema_version"],
            TREECALC_REPLAY_ARTIFACT_MANIFEST_SCHEMA_V1
        );
        assert_eq!(replay_manifest["case_count"], 37);
        assert!(
            replay_manifest["required_root_artifacts"]
                .as_array()
                .is_some_and(|artifacts| artifacts
                    .iter()
                    .any(|artifact| { artifact == "conformance/explain_index.json" }))
        );
        assert!(
            replay_manifest["required_root_artifacts"]
                .as_array()
                .is_some_and(|artifacts| artifacts
                    .iter()
                    .any(|artifact| { artifact == "measurement_counter_summary.json" }))
        );
        assert!(
            replay_manifest["required_root_artifacts"]
                .as_array()
                .is_some_and(|artifacts| artifacts
                    .iter()
                    .any(|artifact| { artifact == "retention_guardrail.json" }))
        );
        assert!(
            replay_manifest["required_root_artifacts"]
                .as_array()
                .is_some_and(|artifacts| artifacts
                    .iter()
                    .any(|artifact| { artifact == "session_path_evidence.json" }))
        );
        assert!(
            replay_manifest["case_artifact_families"]
                .as_array()
                .is_some_and(|families| families
                    .iter()
                    .any(|family| { family == "runtime_effect_overlays" }))
        );
        assert!(
            replay_manifest["case_artifact_families"]
                .as_array()
                .is_some_and(|families| families.iter().any(|family| { family == "counters" }))
        );

        let measurement_summary = serde_json::from_str::<serde_json::Value>(
            &fs::read_to_string(artifact_root.join("measurement_counter_summary.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(
            measurement_summary["schema_version"],
            TREECALC_MEASUREMENT_COUNTER_SUMMARY_SCHEMA_V1
        );
        assert!(
            measurement_summary["counter_families"]
                .as_array()
                .is_some_and(
                    |families| families.iter().any(|family| family["family_id"] == "C2"
                        && family["status"] == "exercised_by_retention_guardrail")
                )
        );

        let retention_guardrail = serde_json::from_str::<serde_json::Value>(
            &fs::read_to_string(artifact_root.join("retention_guardrail.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(
            retention_guardrail["schema_version"],
            TREECALC_RETENTION_GUARDRAIL_SCHEMA_V1
        );
        assert_eq!(
            retention_guardrail["pinned_reader_stability"]["stable"],
            true
        );
        assert!(
            retention_guardrail["retention"]["evicted_overlay_count_after_release"]
                .as_u64()
                .is_some_and(|count| count > 0)
        );

        let typed_reject_taxonomy = serde_json::from_str::<serde_json::Value>(
            &fs::read_to_string(artifact_root.join("typed_reject_taxonomy.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(
            typed_reject_taxonomy["schema_version"],
            TREECALC_TYPED_REJECT_TAXONOMY_SCHEMA_V1
        );
        assert_eq!(typed_reject_taxonomy["handoff_triggered"], false);

        let session_path_evidence = serde_json::from_str::<serde_json::Value>(
            &fs::read_to_string(artifact_root.join("session_path_evidence.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(
            session_path_evidence["schema_version"],
            TREECALC_SESSION_PATH_EVIDENCE_SCHEMA_V1
        );
        assert_eq!(
            session_path_evidence["artifact_root"],
            "docs/test-runs/core-engine/treecalc-local/test-treecalc-local-run"
        );
        assert_eq!(
            session_path_evidence["artifact_root_declaration"]["declared_root"],
            session_path_evidence["artifact_root"]
        );
        assert!(
            session_path_evidence["commands"]
                .as_array()
                .is_some_and(|commands| commands.iter().any(|command| {
                    command
                        == "cargo run -p oxcalc-tracecalc-cli -- treecalc test-treecalc-local-run"
                }))
        );
        let session_entries = session_path_evidence["entries"]
            .as_array()
            .expect("session path evidence entries should be an array");
        assert!(
            session_path_evidence["entry_count"]
                .as_u64()
                .is_some_and(|count| count as usize == session_entries.len())
        );
        assert!(session_entries.len() >= 37);
        let publish_session = session_entries
            .iter()
            .find(|entry| entry["case_id"] == "tc_local_publish_001" && entry["phase"] == "initial")
            .expect("publish session evidence entry should exist");
        assert!(
            publish_session["candidate_result_keys"]["oxcalc_candidate_result_id"]
                .as_str()
                .is_some_and(|value| !value.is_empty())
        );
        assert!(
            publish_session["commit_correlation_keys"]["oxcalc_publication_candidate_result_id"]
                .as_str()
                .is_some_and(|value| !value.is_empty())
        );
        assert_eq!(
            publish_session["commit_correlation_keys"]["candidate_publication_id_match"],
            true
        );
        assert!(
            publish_session["replay_facing_diagnostics"]
                .as_array()
                .is_some_and(|diagnostics| diagnostics.iter().any(|diagnostic| diagnostic
                    .as_str()
                    .is_some_and(|value| value.starts_with("oxfml_commit_attempt_id:"))))
        );
        let local_reject_session = session_entries
            .iter()
            .find(|entry| {
                entry["case_id"] == "tc_local_host_sensitive_reject_001"
                    && entry["phase"] == "initial"
            })
            .expect("local host-sensitive reject session evidence entry should exist");
        assert!(
            local_reject_session["reject_correlation_keys"]["oxcalc_reject_candidate_result_id"]
                .as_str()
                .is_some_and(|value| !value.is_empty())
        );
        assert_eq!(
            local_reject_session["non_mutation_validation"]["rejected_has_no_publication_bundle"],
            true
        );
        let oxfml_reject_session = session_entries
            .iter()
            .find(|entry| {
                entry["case_id"] == "tc_local_lambda_host_sensitive_reject_001"
                    && entry["phase"] == "initial"
            })
            .expect("OxFml host-sensitive reject session evidence entry should exist");
        assert!(
            oxfml_reject_session["returned_value_surface_diagnostics"]
                .as_array()
                .is_some_and(|diagnostics| diagnostics.iter().any(|diagnostic| diagnostic
                    .as_str()
                    .is_some_and(|value| value.starts_with("oxfml_returned_value_surface_"))))
        );
        assert!(
            oxfml_reject_session["replay_facing_diagnostics"]
                .as_array()
                .is_some_and(|diagnostics| diagnostics.iter().any(|diagnostic| diagnostic
                    .as_str()
                    .is_some_and(|value| value.starts_with("oxfml_returned_value_surface_"))))
        );

        let replay_bundle = serde_json::from_str::<serde_json::Value>(
            &fs::read_to_string(artifact_root.join("replay-appliance/bundle_manifest.json"))
                .unwrap(),
        )
        .unwrap();
        assert_eq!(
            replay_bundle["schema_version"],
            TREECALC_REPLAY_APPLIANCE_BUNDLE_SCHEMA_V1
        );
        assert!(
            replay_bundle["preserved_view_families"]
                .as_array()
                .is_some_and(|families| families
                    .iter()
                    .any(|family| { family == "session_correlation_keys" }))
        );

        let replay_validation = serde_json::from_str::<serde_json::Value>(
            &fs::read_to_string(
                artifact_root.join("replay-appliance/validation/bundle_validation.json"),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(replay_validation["status"], "bundle_valid");
        assert!(
            replay_validation["checked_paths"]
                .as_array()
                .is_some_and(|paths| paths.iter().any(|path| path
                    == "docs/test-runs/core-engine/treecalc-local/test-treecalc-local-run/session_path_evidence.json"))
        );
        assert_eq!(
            replay_validation["non_mutation_validation"]["session_path_evidence_checked"],
            true
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
        assert!(
            published_result["prepared_formula_identities"][0]["shape_key"]
                .as_str()
                .is_some_and(|value| value.starts_with("shape:v1:"))
        );
        assert!(
            published_result["prepared_formula_identities"][0]["dispatch_skeleton_key"]
                .as_str()
                .is_some_and(|value| value.starts_with("dispatch_skeleton:v1:"))
        );
        assert!(
            published_result["prepared_formula_identities"][0]["plan_template_key"]
                .as_str()
                .is_some_and(|value| value.starts_with("plan_template:v1:"))
        );
        assert!(
            published_result["prepared_formula_identities"][0]["prepared_callable_key"]
                .as_str()
                .is_some_and(|value| value.starts_with("prepared_callable:v1:"))
        );
        assert!(
            published_result["prepared_formula_identities"][0]["hole_binding_fingerprint"]
                .as_str()
                .is_some_and(|value| value.starts_with("hole_bindings:v1:"))
        );
        assert!(
            published_result["prepared_formula_identities"][0]["template_hole_count"]
                .as_u64()
                .is_some_and(|value| value > 0)
        );
        let published_trace = serde_json::from_str::<serde_json::Value>(
            &fs::read_to_string(artifact_root.join("cases/tc_local_publish_001/trace.json"))
                .unwrap(),
        )
        .unwrap();
        assert!(
            published_trace["events"]
                .as_array()
                .is_some_and(|events| events.iter().any(|event| {
                    event["label"] == "prepared_formula_identity"
                        && event["plan_template_key"]
                            .as_str()
                            .is_some_and(|value| value.starts_with("plan_template:v1:"))
                        && event["hole_binding_fingerprint"]
                            .as_str()
                            .is_some_and(|value| value.starts_with("hole_bindings:v1:"))
                }))
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
        assert!(
            published_result["candidate_result"]["candidate_result_id"]
                .as_str()
                .is_some_and(|value| !value.is_empty())
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

        let dynamic_resolved_result = serde_json::from_str::<serde_json::Value>(
            &fs::read_to_string(
                artifact_root.join("cases/tc_local_dynamic_resolved_publish_001/result.json"),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(dynamic_resolved_result["result_state"], "published");
        assert_eq!(
            dynamic_resolved_result["candidate_result"]["dependency_shape_updates"][0]["kind"],
            "activate_dynamic_dep"
        );
        assert_eq!(
            dynamic_resolved_result["publication_bundle"]["carriage_classification"]["dependency_shape_update_count"],
            1
        );
        assert_eq!(
            dynamic_resolved_result["publication_bundle"]["published_runtime_effects"][0]["family"],
            "DynamicDependency"
        );

        let dynamic_release_reclass_seeds = serde_json::from_str::<serde_json::Value>(
            &fs::read_to_string(
                artifact_root.join(
                    "cases/tc_local_dynamic_release_reclassification_post_edit_001/post_edit/invalidation_seeds.json",
                ),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(
            dynamic_release_reclass_seeds.as_array().map(Vec::len),
            Some(2)
        );
        assert!(
            dynamic_release_reclass_seeds
                .as_array()
                .is_some_and(|seeds| seeds
                    .iter()
                    .any(|seed| seed["reason"] == "DynamicDependencyReleased"))
        );
        assert!(
            dynamic_release_reclass_seeds
                .as_array()
                .is_some_and(|seeds| seeds
                    .iter()
                    .any(|seed| seed["reason"] == "DynamicDependencyReclassified"))
        );

        let dynamic_release_reclass_result = serde_json::from_str::<serde_json::Value>(
            &fs::read_to_string(
                artifact_root.join(
                    "cases/tc_local_dynamic_release_reclassification_post_edit_001/post_edit/result.json",
                ),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(dynamic_release_reclass_result["result_state"], "rejected");
        assert_eq!(
            dynamic_release_reclass_result["reject_detail"]["kind"],
            "DynamicDependencyFailure"
        );
        let dynamic_release_reclass_closure = serde_json::from_str::<serde_json::Value>(
            &fs::read_to_string(
                artifact_root.join(
                    "cases/tc_local_dynamic_release_reclassification_post_edit_001/post_edit/invalidation_closure.json",
                ),
            )
            .unwrap(),
        )
        .unwrap();
        assert!(
            dynamic_release_reclass_closure
                .as_array()
                .is_some_and(|records| records.iter().any(|record| {
                    record["node_id"] == 3
                        && record["requires_rebind"] == false
                        && record["reasons"].as_array().is_some_and(|reasons| {
                            reasons
                                .iter()
                                .any(|reason| reason == "DynamicDependencyReleased")
                                && reasons
                                    .iter()
                                    .any(|reason| reason == "DynamicDependencyReclassified")
                        })
                }))
        );

        let auto_dynamic_release_reclass_seeds = serde_json::from_str::<serde_json::Value>(
            &fs::read_to_string(
                artifact_root.join(
                    "cases/tc_local_dynamic_release_reclassification_auto_post_edit_001/post_edit/invalidation_seeds.json",
                ),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(
            auto_dynamic_release_reclass_seeds.as_array().map(Vec::len),
            Some(2)
        );
        assert!(
            auto_dynamic_release_reclass_seeds
                .as_array()
                .is_some_and(|seeds| seeds
                    .iter()
                    .any(|seed| seed["reason"] == "DynamicDependencyReleased"))
        );
        assert!(
            auto_dynamic_release_reclass_seeds
                .as_array()
                .is_some_and(|seeds| seeds
                    .iter()
                    .any(|seed| seed["reason"] == "DynamicDependencyReclassified"))
        );

        let auto_dynamic_addition_seeds = serde_json::from_str::<serde_json::Value>(
            &fs::read_to_string(
                artifact_root.join(
                    "cases/tc_local_dynamic_addition_auto_post_edit_001/post_edit/invalidation_seeds.json",
                ),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(
            auto_dynamic_addition_seeds.as_array().map(Vec::len),
            Some(2)
        );
        assert!(auto_dynamic_addition_seeds.as_array().is_some_and(|seeds| {
            seeds
                .iter()
                .any(|seed| seed["reason"] == "DynamicDependencyActivated")
        }));
        assert!(auto_dynamic_addition_seeds.as_array().is_some_and(|seeds| {
            seeds
                .iter()
                .any(|seed| seed["reason"] == "DynamicDependencyReclassified")
        }));
        let auto_dynamic_addition_result =
            serde_json::from_str::<serde_json::Value>(
                &fs::read_to_string(artifact_root.join(
                    "cases/tc_local_dynamic_addition_auto_post_edit_001/post_edit/result.json",
                ))
                .unwrap(),
            )
            .unwrap();
        assert_eq!(auto_dynamic_addition_result["result_state"], "published");
        assert_eq!(auto_dynamic_addition_result["published_values"]["3"], "13");
        let auto_dynamic_addition_closure =
            serde_json::from_str::<serde_json::Value>(
                &fs::read_to_string(artifact_root.join(
                    "cases/tc_local_dynamic_addition_auto_post_edit_001/post_edit/invalidation_closure.json",
                ))
                .unwrap(),
            )
            .unwrap();
        assert!(
            auto_dynamic_addition_closure
                .as_array()
                .is_some_and(|records| records.iter().any(|record| {
                    record["node_id"] == 3
                        && record["requires_rebind"] == false
                        && record["reasons"].as_array().is_some_and(|reasons| {
                            reasons
                                .iter()
                                .any(|reason| reason == "DynamicDependencyActivated")
                                && reasons
                                    .iter()
                                    .any(|reason| reason == "DynamicDependencyReclassified")
                        })
                }))
        );

        let mixed_dynamic_transition_seeds =
            serde_json::from_str::<serde_json::Value>(
                &fs::read_to_string(artifact_root.join(
                    "cases/tc_local_dynamic_mixed_add_release_auto_post_edit_001/post_edit/invalidation_seeds.json",
                ))
                .unwrap(),
            )
            .unwrap();
        assert_eq!(
            mixed_dynamic_transition_seeds.as_array().map(Vec::len),
            Some(3)
        );
        for expected_reason in [
            "DynamicDependencyActivated",
            "DynamicDependencyReleased",
            "DynamicDependencyReclassified",
        ] {
            assert!(
                mixed_dynamic_transition_seeds
                    .as_array()
                    .is_some_and(|seeds| seeds
                        .iter()
                        .any(|seed| seed["reason"] == expected_reason)),
                "missing mixed dynamic transition seed {expected_reason}"
            );
        }
        let mixed_dynamic_transition_result = serde_json::from_str::<serde_json::Value>(
            &fs::read_to_string(artifact_root.join(
                "cases/tc_local_dynamic_mixed_add_release_auto_post_edit_001/post_edit/result.json",
            ))
            .unwrap(),
        )
        .unwrap();
        assert_eq!(mixed_dynamic_transition_result["result_state"], "rejected");
        assert_eq!(
            mixed_dynamic_transition_result["reject_detail"]["kind"],
            "DynamicDependencyFailure"
        );
        let mixed_dynamic_transition_closure =
            serde_json::from_str::<serde_json::Value>(
                &fs::read_to_string(artifact_root.join(
                    "cases/tc_local_dynamic_mixed_add_release_auto_post_edit_001/post_edit/invalidation_closure.json",
                ))
                .unwrap(),
            )
            .unwrap();
        assert!(
            mixed_dynamic_transition_closure
                .as_array()
                .is_some_and(|records| records.iter().any(|record| {
                    record["node_id"] == 3
                        && record["requires_rebind"] == false
                        && record["reasons"].as_array().is_some_and(|reasons| {
                            reasons
                                .iter()
                                .any(|reason| reason == "DynamicDependencyActivated")
                                && reasons
                                    .iter()
                                    .any(|reason| reason == "DynamicDependencyReleased")
                                && reasons
                                    .iter()
                                    .any(|reason| reason == "DynamicDependencyReclassified")
                        })
                }))
        );

        let dynamic_switch_result = serde_json::from_str::<serde_json::Value>(
            &fs::read_to_string(artifact_root.join(
                "cases/tc_local_dynamic_target_switch_downstream_publish_001/post_edit/result.json",
            ))
            .unwrap(),
        )
        .unwrap();
        assert_eq!(dynamic_switch_result["result_state"], "published");
        assert_eq!(dynamic_switch_result["published_values"]["3"], "7");
        assert_eq!(dynamic_switch_result["published_values"]["5"], "8");
        assert!(
            dynamic_switch_result["candidate_result"]["dependency_shape_updates"]
                .as_array()
                .is_some_and(|updates| updates
                    .iter()
                    .any(|update| update["kind"] == "activate_dynamic_dep")
                    && updates
                        .iter()
                        .any(|update| update["kind"] == "release_dynamic_dep"))
        );
        assert_eq!(
            dynamic_switch_result["publication_bundle"]["published_view_delta"]["3"],
            "7"
        );
        assert_eq!(
            dynamic_switch_result["publication_bundle"]["published_view_delta"]["5"],
            "8"
        );
        let dynamic_switch_closure = serde_json::from_str::<serde_json::Value>(
            &fs::read_to_string(artifact_root.join(
                "cases/tc_local_dynamic_target_switch_downstream_publish_001/post_edit/invalidation_closure.json",
            ))
            .unwrap(),
        )
        .unwrap();
        assert!(dynamic_switch_closure.as_array().is_some_and(|records| {
            records.iter().any(|record| {
                record["node_id"] == 3
                    && record["requires_rebind"] == false
                    && record["reasons"].as_array().is_some_and(|reasons| {
                        reasons
                            .iter()
                            .any(|reason| reason == "DynamicDependencyActivated")
                            && reasons
                                .iter()
                                .any(|reason| reason == "DynamicDependencyReleased")
                            && reasons
                                .iter()
                                .any(|reason| reason == "DynamicDependencyReclassified")
                    })
            }) && records.iter().any(|record| {
                record["node_id"] == 5
                    && record["requires_rebind"] == false
                    && record["reasons"].as_array().is_some_and(|reasons| {
                        reasons.iter().any(|reason| reason == "UpstreamPublication")
                    })
            })
        }));

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

        let capability_explain = serde_json::from_str::<serde_json::Value>(
            &fs::read_to_string(
                artifact_root.join("cases/tc_local_capability_sensitive_reject_001/explain.json"),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(
            capability_explain["runtime_effects"][0]["family"],
            "CapabilitySensitive"
        );
        assert_eq!(
            capability_explain["runtime_effect_overlays"][0]["overlay_kind"],
            "ExecutionRestriction"
        );
        assert!(capability_explain["publication_bundle"].is_null());

        let shape_explain = serde_json::from_str::<serde_json::Value>(
            &fs::read_to_string(
                artifact_root.join("cases/tc_local_shape_topology_reject_001/explain.json"),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(
            shape_explain["runtime_effects"][0]["family"],
            "ShapeTopology"
        );
        assert_eq!(
            shape_explain["runtime_effect_overlays"][0]["overlay_kind"],
            "ShapeTopology"
        );
        assert!(shape_explain["publication_bundle"].is_null());

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
