#![forbid(unsafe_code)]

//! Independent TraceCalc-to-TreeCalc conformance packet emission.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use serde_json::{Value, json};
use thiserror::Error;

const INDEPENDENT_CONFORMANCE_RUN_SUMMARY_SCHEMA_V1: &str =
    "oxcalc.independent_conformance.run_summary.v1";
const INDEPENDENT_CONFORMANCE_SURFACE_MAPPING_SCHEMA_V1: &str =
    "oxcalc.independent_conformance.surface_mapping.v1";
const INDEPENDENT_CONFORMANCE_TREECALC_DIFF_SCHEMA_V1: &str =
    "oxcalc.independent_conformance.treecalc_tracecalc_diff.v1";
const INDEPENDENT_CONFORMANCE_CORE_PROJECTION_SCHEMA_V1: &str =
    "oxcalc.independent_conformance.core_projection_diff.v1";
const INDEPENDENT_CONFORMANCE_BUNDLE_SCHEMA_V1: &str =
    "oxcalc.independent_conformance.bundle_manifest.v1";
const INDEPENDENT_CONFORMANCE_VALIDATION_SCHEMA_V1: &str =
    "oxcalc.independent_conformance.bundle_validation.v1";

const TRACECALC_REFERENCE_RUN_ID: &str = "w034-tracecalc-oracle-deepening-001";
const TREECALC_REFERENCE_RUN_ID: &str = "w034-independent-conformance-treecalc-001";

#[derive(Debug, Error)]
pub enum IndependentConformanceError {
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
pub struct IndependentConformanceRunSummary {
    pub run_id: String,
    pub schema_version: String,
    pub comparison_row_count: usize,
    pub exact_value_match_count: usize,
    pub no_publication_match_count: usize,
    pub lifecycle_surface_match_count: usize,
    pub declared_gap_count: usize,
    pub missing_artifact_count: usize,
    pub unexpected_mismatch_count: usize,
    pub artifact_root: String,
}

#[derive(Debug, Clone, Default)]
pub struct IndependentConformanceRunner;

#[derive(Debug, Clone, Copy)]
struct NodeValueMapping {
    trace_node_id: &'static str,
    tree_node_id: u64,
}

#[derive(Debug, Clone, Copy)]
struct ValueRowSpec {
    row_id: &'static str,
    observable_class: &'static str,
    trace_scenario_id: &'static str,
    tree_case_id: &'static str,
    value_mappings: &'static [NodeValueMapping],
}

#[derive(Debug, Clone, Copy)]
struct NoPublicationRowSpec {
    row_id: &'static str,
    observable_class: &'static str,
    trace_scenario_id: &'static str,
    tree_case_id: &'static str,
    expected_tree_state: &'static str,
    required_trace_event_family: Option<&'static str>,
    require_trace_reject: bool,
}

#[derive(Debug, Clone, Copy)]
struct DeclaredGapRowSpec {
    row_id: &'static str,
    observable_class: &'static str,
    trace_scenario_id: &'static str,
    tree_case_id: &'static str,
    expected_tree_state: &'static str,
    required_tree_runtime_effect_family: &'static str,
    gap_classification: &'static str,
}

#[derive(Debug, Clone, Copy)]
struct TraceOnlyGapRowSpec {
    row_id: &'static str,
    observable_class: &'static str,
    trace_scenario_id: &'static str,
    required_trace_event_family: Option<&'static str>,
    require_trace_reject: bool,
    gap_classification: &'static str,
    gap_reason: &'static str,
}

#[derive(Debug, Clone, Copy)]
struct LifecycleRowSpec {
    row_id: &'static str,
    observable_class: &'static str,
    trace_scenario_id: &'static str,
    tree_artifact_name: &'static str,
    required_trace_event_families: &'static [&'static str],
    required_tree_event_labels: &'static [&'static str],
}

#[derive(Debug, Clone, Copy, Default)]
struct ComparisonCounts {
    exact_value_matches: usize,
    no_publication_matches: usize,
    lifecycle_surface_matches: usize,
    declared_gaps: usize,
    missing_artifacts: usize,
    unexpected_mismatches: usize,
}

struct BaseRow<'a> {
    row_id: &'a str,
    observable_class: &'a str,
    trace_scenario_id: &'a str,
    tree_case_id: Option<&'a str>,
    comparison_state: &'a str,
    missing_artifacts: Vec<String>,
    failures: Vec<String>,
    details: Value,
}

const ACCEPT_VALUE_MAPPINGS: &[NodeValueMapping] = &[NodeValueMapping {
    trace_node_id: "B",
    tree_node_id: 3,
}];
const DAG_VALUE_MAPPINGS: &[NodeValueMapping] = &[
    NodeValueMapping {
        trace_node_id: "B",
        tree_node_id: 3,
    },
    NodeValueMapping {
        trace_node_id: "C",
        tree_node_id: 4,
    },
    NodeValueMapping {
        trace_node_id: "D",
        tree_node_id: 5,
    },
];
const LET_LAMBDA_VALUE_MAPPINGS: &[NodeValueMapping] = &[NodeValueMapping {
    trace_node_id: "B",
    tree_node_id: 3,
}];
const W034_HIGHER_ORDER_VALUE_MAPPINGS: &[NodeValueMapping] = &[NodeValueMapping {
    trace_node_id: "Out",
    tree_node_id: 3,
}];
const W034_INDEPENDENT_ORDER_VALUE_MAPPINGS: &[NodeValueMapping] = &[
    NodeValueMapping {
        trace_node_id: "Left",
        tree_node_id: 4,
    },
    NodeValueMapping {
        trace_node_id: "Top",
        tree_node_id: 5,
    },
    NodeValueMapping {
        trace_node_id: "Check",
        tree_node_id: 6,
    },
];
const W034_OVERLAY_TRACE_FAMILIES: &[&str] = &[
    "session.reader_pinned",
    "overlay.retained",
    "session.reader_unpinned",
    "overlay.eviction_eligible",
    "overlay.released",
];
const W034_OVERLAY_TREE_EVENT_LABELS: &[&str] = &[
    "reader_pinned",
    "retention_blocked_cleanup",
    "reader_unpinned",
    "eviction_eligibility_opened",
    "overlay_released",
];

const VALUE_ROW_SPECS: &[ValueRowSpec] = &[
    ValueRowSpec {
        row_id: "ic_exact_accept_publish_001",
        observable_class: "accepted_publication_value_delta",
        trace_scenario_id: "tc_accept_publish_001",
        tree_case_id: "tc_local_tracecalc_accept_publish_equiv_001",
        value_mappings: ACCEPT_VALUE_MAPPINGS,
    },
    ValueRowSpec {
        row_id: "ic_exact_multinode_dag_001",
        observable_class: "multi_node_dag_value_delta",
        trace_scenario_id: "tc_multinode_dag_publish_001",
        tree_case_id: "tc_local_tracecalc_multinode_dag_equiv_001",
        value_mappings: DAG_VALUE_MAPPINGS,
    },
    ValueRowSpec {
        row_id: "ic_exact_let_lambda_capture_001",
        observable_class: "let_lambda_capture_value_delta",
        trace_scenario_id: "tc_let_lambda_carrier_publish_001",
        tree_case_id: "tc_local_let_lambda_capture_publish_001",
        value_mappings: LET_LAMBDA_VALUE_MAPPINGS,
    },
    ValueRowSpec {
        row_id: "ic_exact_w034_higher_order_let_lambda_value_001",
        observable_class: "w034_let_lambda_higher_order_value_delta",
        trace_scenario_id: "tc_w034_let_lambda_higher_order_replay_001",
        tree_case_id: "tc_local_w034_higher_order_let_lambda_publish_001",
        value_mappings: W034_HIGHER_ORDER_VALUE_MAPPINGS,
    },
    ValueRowSpec {
        row_id: "ic_exact_w034_independent_order_value_001",
        observable_class: "w034_replay_equivalent_independent_order_value_delta",
        trace_scenario_id: "tc_w034_replay_equivalent_independent_order_001",
        tree_case_id: "tc_local_w034_independent_order_equiv_001",
        value_mappings: W034_INDEPENDENT_ORDER_VALUE_MAPPINGS,
    },
];

const NO_PUBLICATION_ROW_SPECS: &[NoPublicationRowSpec] = &[
    NoPublicationRowSpec {
        row_id: "ic_no_publish_verified_clean_001",
        observable_class: "verified_clean_no_publication",
        trace_scenario_id: "tc_verify_clean_no_publish_001",
        tree_case_id: "tc_local_verified_clean_001",
        expected_tree_state: "verified_clean",
        required_trace_event_family: Some("candidate.verified_clean"),
        require_trace_reject: false,
    },
    NoPublicationRowSpec {
        row_id: "ic_no_publish_reject_001",
        observable_class: "reject_no_publication",
        trace_scenario_id: "tc_reject_no_publish_001",
        tree_case_id: "tc_local_capability_sensitive_reject_001",
        expected_tree_state: "rejected",
        required_trace_event_family: None,
        require_trace_reject: true,
    },
    NoPublicationRowSpec {
        row_id: "ic_no_publish_w034_capability_fence_reject_001",
        observable_class: "w034_capability_fence_reject_no_publication",
        trace_scenario_id: "tc_w034_capability_fence_reject_001",
        tree_case_id: "tc_local_capability_sensitive_reject_001",
        expected_tree_state: "rejected",
        required_trace_event_family: Some("reject.issued"),
        require_trace_reject: true,
    },
];

const LIFECYCLE_ROW_SPECS: &[LifecycleRowSpec] = &[LifecycleRowSpec {
    row_id: "ic_lifecycle_w034_overlay_release_001",
    observable_class: "w034_overlay_retention_release_lifecycle",
    trace_scenario_id: "tc_w034_overlay_eviction_after_unpin_001",
    tree_artifact_name: "retention_guardrail.json",
    required_trace_event_families: W034_OVERLAY_TRACE_FAMILIES,
    required_tree_event_labels: W034_OVERLAY_TREE_EVENT_LABELS,
}];

const TRACE_ONLY_GAP_ROW_SPECS: &[TraceOnlyGapRowSpec] = &[
    TraceOnlyGapRowSpec {
        row_id: "ic_gap_w034_snapshot_fence_projection_001",
        observable_class: "w034_snapshot_fence_reject_current_local_gap",
        trace_scenario_id: "tc_w034_snapshot_fence_reject_001",
        required_trace_event_family: Some("reject.issued"),
        require_trace_reject: true,
        gap_classification: "treecalc_local_snapshot_fence_projection_gap",
        gap_reason: "TraceCalc covers snapshot epoch fence mismatch; TreeCalc-local has no fixture-level snapshot fence admission counterpart yet.",
    },
    TraceOnlyGapRowSpec {
        row_id: "ic_gap_w034_capability_view_fence_projection_001",
        observable_class: "w034_capability_view_fence_reject_current_local_gap",
        trace_scenario_id: "tc_w034_capability_fence_reject_001",
        required_trace_event_family: Some("reject.issued"),
        require_trace_reject: true,
        gap_classification: "treecalc_local_capability_view_fence_projection_gap",
        gap_reason: "TreeCalc-local exercises capability-sensitive references, but not compatibility-fenced capability-view mismatch replay.",
    },
    TraceOnlyGapRowSpec {
        row_id: "ic_gap_w034_higher_order_callable_metadata_001",
        observable_class: "w034_let_lambda_callable_identity_current_local_gap",
        trace_scenario_id: "tc_w034_let_lambda_higher_order_replay_001",
        required_trace_event_family: None,
        require_trace_reject: false,
        gap_classification: "treecalc_local_higher_order_callable_identity_projection_gap",
        gap_reason: "TreeCalc-local compares the published value, but does not yet project returned callable identity or callable-carrier metadata.",
    },
];

const DECLARED_GAP_ROW_SPECS: &[DeclaredGapRowSpec] = &[
    DeclaredGapRowSpec {
        row_id: "ic_gap_dynamic_dependency_001",
        observable_class: "dynamic_dependency_current_local_gap",
        trace_scenario_id: "tc_dynamic_dep_switch_001",
        tree_case_id: "tc_local_dynamic_reject_001",
        expected_tree_state: "rejected",
        required_tree_runtime_effect_family: "DynamicDependency",
        gap_classification: "treecalc_local_dynamic_dependency_projection_gap",
    },
    DeclaredGapRowSpec {
        row_id: "ic_gap_lambda_host_effect_001",
        observable_class: "lambda_host_sensitive_current_local_gap",
        trace_scenario_id: "tc_let_lambda_runtime_effect_001",
        tree_case_id: "tc_local_lambda_host_sensitive_reject_001",
        expected_tree_state: "rejected",
        required_tree_runtime_effect_family: "ExecutionRestriction",
        gap_classification: "treecalc_local_host_sensitive_lambda_projection_gap",
    },
    DeclaredGapRowSpec {
        row_id: "ic_gap_w034_dynamic_dependency_negative_001",
        observable_class: "w034_dynamic_dependency_negative_current_local_gap",
        trace_scenario_id: "tc_w034_dynamic_dependency_negative_001",
        tree_case_id: "tc_local_dynamic_reject_001",
        expected_tree_state: "rejected",
        required_tree_runtime_effect_family: "DynamicDependency",
        gap_classification: "treecalc_local_dynamic_dependency_shape_update_projection_gap",
    },
];

impl IndependentConformanceRunner {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    pub fn execute(
        &self,
        repo_root: &Path,
        run_id: &str,
    ) -> Result<IndependentConformanceRunSummary, IndependentConformanceError> {
        let artifact_root = repo_root.join(format!(
            "docs/test-runs/core-engine/independent-conformance/{run_id}"
        ));
        let relative_artifact_root = relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "independent-conformance",
            run_id,
        ]);

        if artifact_root.exists() {
            fs::remove_dir_all(&artifact_root).map_err(|source| {
                IndependentConformanceError::RemoveDirectory {
                    path: artifact_root.display().to_string(),
                    source,
                }
            })?;
        }

        create_directory(&artifact_root)?;
        create_directory(&artifact_root.join("comparisons"))?;
        create_directory(&artifact_root.join("replay-appliance"))?;
        create_directory(&artifact_root.join("replay-appliance/validation"))?;

        write_json(
            &artifact_root.join("surface_mapping.json"),
            &surface_mapping_json(run_id, &relative_artifact_root),
        )?;

        let mut comparison_rows = Vec::new();
        for spec in VALUE_ROW_SPECS {
            comparison_rows.push(value_comparison_row(repo_root, spec)?);
        }
        for spec in NO_PUBLICATION_ROW_SPECS {
            comparison_rows.push(no_publication_comparison_row(repo_root, spec)?);
        }
        for spec in LIFECYCLE_ROW_SPECS {
            comparison_rows.push(lifecycle_comparison_row(repo_root, spec)?);
        }
        for spec in DECLARED_GAP_ROW_SPECS {
            comparison_rows.push(declared_gap_comparison_row(repo_root, spec)?);
        }
        for spec in TRACE_ONLY_GAP_ROW_SPECS {
            comparison_rows.push(trace_only_gap_comparison_row(repo_root, spec)?);
        }

        let counts = comparison_counts(&comparison_rows);
        write_json(
            &artifact_root.join("comparisons/treecalc_tracecalc_differential.json"),
            &json!({
                "schema_version": INDEPENDENT_CONFORMANCE_TREECALC_DIFF_SCHEMA_V1,
                "run_id": run_id,
                "tracecalc_reference_run_id": TRACECALC_REFERENCE_RUN_ID,
                "treecalc_reference_run_id": TREECALC_REFERENCE_RUN_ID,
                "comparison_row_count": comparison_rows.len(),
                "exact_value_match_count": counts.exact_value_matches,
                "no_publication_match_count": counts.no_publication_matches,
                "lifecycle_surface_match_count": counts.lifecycle_surface_matches,
                "declared_gap_count": counts.declared_gaps,
                "missing_artifact_count": counts.missing_artifacts,
                "unexpected_mismatch_count": counts.unexpected_mismatches,
                "rows": comparison_rows,
            }),
        )?;

        let core_rows = core_projection_rows(repo_root)?;
        write_json(
            &artifact_root.join("comparisons/core_engine_projection_differential.json"),
            &json!({
                "schema_version": INDEPENDENT_CONFORMANCE_CORE_PROJECTION_SCHEMA_V1,
                "run_id": run_id,
                "treecalc_reference_run_id": TREECALC_REFERENCE_RUN_ID,
                "projection_row_count": core_rows.len(),
                "rows": core_rows,
            }),
        )?;

        let required_artifacts = required_artifacts(run_id);
        write_json(
            &artifact_root.join("replay-appliance/bundle_manifest.json"),
            &json!({
                "schema_version": INDEPENDENT_CONFORMANCE_BUNDLE_SCHEMA_V1,
                "run_id": run_id,
                "artifact_root": relative_artifact_root,
                "tracecalc_reference_run_id": TRACECALC_REFERENCE_RUN_ID,
                "treecalc_reference_run_id": TREECALC_REFERENCE_RUN_ID,
                "claimed_capability": "independent_treecalc_tracecalc_observable_surface_comparison",
                "excluded_capabilities": [
                    "fully_independent_evaluator_implementation",
                    "pack_grade_replay",
                    "continuous_cross_engine_differential_suite"
                ],
                "required_artifacts": required_artifacts,
            }),
        )?;

        let summary = IndependentConformanceRunSummary {
            run_id: run_id.to_string(),
            schema_version: INDEPENDENT_CONFORMANCE_RUN_SUMMARY_SCHEMA_V1.to_string(),
            comparison_row_count: comparison_rows.len(),
            exact_value_match_count: counts.exact_value_matches,
            no_publication_match_count: counts.no_publication_matches,
            lifecycle_surface_match_count: counts.lifecycle_surface_matches,
            declared_gap_count: counts.declared_gaps,
            missing_artifact_count: counts.missing_artifacts,
            unexpected_mismatch_count: counts.unexpected_mismatches,
            artifact_root: relative_artifact_root.clone(),
        };
        write_json(
            &artifact_root.join("run_summary.json"),
            &json!({
                "schema_version": summary.schema_version,
                "run_id": summary.run_id,
                "tracecalc_reference_run_id": TRACECALC_REFERENCE_RUN_ID,
                "treecalc_reference_run_id": TREECALC_REFERENCE_RUN_ID,
                "comparison_row_count": summary.comparison_row_count,
                "exact_value_match_count": summary.exact_value_match_count,
                "no_publication_match_count": summary.no_publication_match_count,
                "lifecycle_surface_match_count": summary.lifecycle_surface_match_count,
                "declared_gap_count": summary.declared_gap_count,
                "missing_artifact_count": summary.missing_artifact_count,
                "unexpected_mismatch_count": summary.unexpected_mismatch_count,
                "handoff_triggered": summary.unexpected_mismatch_count > 0,
                "artifact_root": summary.artifact_root,
                "surface_mapping_path": format!("{relative_artifact_root}/surface_mapping.json"),
                "treecalc_tracecalc_differential_path": format!("{relative_artifact_root}/comparisons/treecalc_tracecalc_differential.json"),
                "core_engine_projection_differential_path": format!("{relative_artifact_root}/comparisons/core_engine_projection_differential.json"),
                "bundle_validation_path": format!("{relative_artifact_root}/replay-appliance/validation/bundle_validation.json"),
            }),
        )?;

        let validation_path =
            artifact_root.join("replay-appliance/validation/bundle_validation.json");
        write_json(
            &validation_path,
            &json!({
                "schema_version": INDEPENDENT_CONFORMANCE_VALIDATION_SCHEMA_V1,
                "run_id": run_id,
                "status": "pending_final_validation_write",
            }),
        )?;

        let missing_paths = required_artifacts
            .iter()
            .filter(|relative_path| !repo_root.join(relative_path.as_str()).exists())
            .cloned()
            .collect::<Vec<_>>();
        write_json(
            &validation_path,
            &json!({
                "schema_version": INDEPENDENT_CONFORMANCE_VALIDATION_SCHEMA_V1,
                "run_id": run_id,
                "status": if missing_paths.is_empty() { "bundle_valid" } else { "missing_required_artifacts" },
                "missing_paths": missing_paths,
                "validated_required_artifact_count": required_artifacts.len(),
                "unexpected_mismatch_count": summary.unexpected_mismatch_count,
                "declared_gap_count": summary.declared_gap_count,
            }),
        )?;

        Ok(summary)
    }
}

fn value_comparison_row(
    repo_root: &Path,
    spec: &ValueRowSpec,
) -> Result<Value, IndependentConformanceError> {
    let trace_result_path = trace_artifact_path(spec.trace_scenario_id, "result.json");
    let trace_published_path = trace_artifact_path(spec.trace_scenario_id, "published_view.json");
    let tree_result_path = tree_artifact_path(spec.tree_case_id, "result.json");
    let tree_published_path = tree_artifact_path(spec.tree_case_id, "published_values.json");

    let trace_result = read_json(repo_root, &trace_result_path)?;
    let trace_published = read_json(repo_root, &trace_published_path)?;
    let tree_result = read_json(repo_root, &tree_result_path)?;
    let tree_published = read_json(repo_root, &tree_published_path)?;
    let missing_artifacts = missing_artifacts([
        (&trace_result_path, &trace_result),
        (&trace_published_path, &trace_published),
        (&tree_result_path, &tree_result),
        (&tree_published_path, &tree_published),
    ]);

    if !missing_artifacts.is_empty() {
        return Ok(base_row_json(BaseRow {
            row_id: spec.row_id,
            observable_class: spec.observable_class,
            trace_scenario_id: spec.trace_scenario_id,
            tree_case_id: Some(spec.tree_case_id),
            comparison_state: "missing_artifact",
            missing_artifacts,
            failures: Vec::new(),
            details: json!({}),
        }));
    }

    let trace_result = trace_result.as_ref().expect("missing checked above");
    let trace_published = trace_published.as_ref().expect("missing checked above");
    let tree_result = tree_result.as_ref().expect("missing checked above");
    let tree_published = tree_published.as_ref().expect("missing checked above");

    let trace_values = trace_value_map(trace_published);
    let tree_values = tree_value_map(tree_published);
    let mut failures = Vec::new();
    if !trace_result_is_clean_pass(trace_result) {
        failures.push("tracecalc_result_not_clean_pass".to_string());
    }
    if tree_result.get("result_state").and_then(Value::as_str) != Some("published") {
        failures.push("treecalc_result_not_published".to_string());
    }

    let value_pairs = spec
        .value_mappings
        .iter()
        .map(|mapping| {
            let tree_key = mapping.tree_node_id.to_string();
            let trace_value = trace_values.get(mapping.trace_node_id);
            let tree_value = tree_values.get(&tree_key);
            if trace_value != tree_value {
                failures.push(format!(
                    "value_mismatch:{}:{}",
                    mapping.trace_node_id, mapping.tree_node_id
                ));
            }
            json!({
                "trace_node_id": mapping.trace_node_id,
                "tree_node_id": mapping.tree_node_id,
                "trace_value": trace_value,
                "tree_value": tree_value,
                "matched": trace_value == tree_value,
            })
        })
        .collect::<Vec<_>>();

    Ok(base_row_json(BaseRow {
        row_id: spec.row_id,
        observable_class: spec.observable_class,
        trace_scenario_id: spec.trace_scenario_id,
        tree_case_id: Some(spec.tree_case_id),
        comparison_state: if failures.is_empty() {
            "matched_exact_value_surface"
        } else {
            "mismatch"
        },
        missing_artifacts: Vec::new(),
        failures,
        details: json!({
            "asserted_surfaces": [
                "assertion_result_set",
                "published_value_delta",
                "candidate_publication_boundary"
            ],
            "value_pairs": value_pairs,
            "trace_artifacts": {
                "result": trace_result_path,
                "published_view": trace_published_path,
            },
            "tree_artifacts": {
                "result": tree_result_path,
                "published_values": tree_published_path,
            },
        }),
    }))
}

fn no_publication_comparison_row(
    repo_root: &Path,
    spec: &NoPublicationRowSpec,
) -> Result<Value, IndependentConformanceError> {
    let trace_result_path = trace_artifact_path(spec.trace_scenario_id, "result.json");
    let trace_trace_path = trace_artifact_path(spec.trace_scenario_id, "trace.json");
    let trace_rejects_path = trace_artifact_path(spec.trace_scenario_id, "rejects.json");
    let tree_result_path = tree_artifact_path(spec.tree_case_id, "result.json");

    let trace_result = read_json(repo_root, &trace_result_path)?;
    let trace_trace = read_json(repo_root, &trace_trace_path)?;
    let trace_rejects = read_json(repo_root, &trace_rejects_path)?;
    let tree_result = read_json(repo_root, &tree_result_path)?;
    let missing_artifacts = missing_artifacts([
        (&trace_result_path, &trace_result),
        (&trace_trace_path, &trace_trace),
        (&trace_rejects_path, &trace_rejects),
        (&tree_result_path, &tree_result),
    ]);

    if !missing_artifacts.is_empty() {
        return Ok(base_row_json(BaseRow {
            row_id: spec.row_id,
            observable_class: spec.observable_class,
            trace_scenario_id: spec.trace_scenario_id,
            tree_case_id: Some(spec.tree_case_id),
            comparison_state: "missing_artifact",
            missing_artifacts,
            failures: Vec::new(),
            details: json!({}),
        }));
    }

    let trace_result = trace_result.as_ref().expect("missing checked above");
    let trace_trace = trace_trace.as_ref().expect("missing checked above");
    let trace_rejects = trace_rejects.as_ref().expect("missing checked above");
    let tree_result = tree_result.as_ref().expect("missing checked above");
    let mut failures = Vec::new();

    if !trace_result_is_clean_pass(trace_result) {
        failures.push("tracecalc_result_not_clean_pass".to_string());
    }
    if tree_result.get("result_state").and_then(Value::as_str) != Some(spec.expected_tree_state) {
        failures.push("treecalc_result_state_mismatch".to_string());
    }
    if !tree_publication_bundle_is_absent(tree_result) {
        failures.push("treecalc_publication_bundle_present".to_string());
    }
    if let Some(required_family) = spec.required_trace_event_family
        && !trace_has_event_family(trace_trace, required_family)
    {
        failures.push(format!("missing_trace_event_family:{required_family}"));
    }
    if spec.require_trace_reject && trace_reject_count(trace_rejects) == 0 {
        failures.push("tracecalc_reject_missing".to_string());
    }

    Ok(base_row_json(BaseRow {
        row_id: spec.row_id,
        observable_class: spec.observable_class,
        trace_scenario_id: spec.trace_scenario_id,
        tree_case_id: Some(spec.tree_case_id),
        comparison_state: if failures.is_empty() {
            "matched_no_publication_surface"
        } else {
            "mismatch"
        },
        missing_artifacts: Vec::new(),
        failures,
        details: json!({
            "asserted_surfaces": [
                "assertion_result_set",
                "candidate_publication_boundary",
                "reject_is_no_publish_or_verified_clean_is_no_publish"
            ],
            "required_trace_event_family": spec.required_trace_event_family,
            "require_trace_reject": spec.require_trace_reject,
            "trace_reject_count": trace_reject_count(trace_rejects),
            "tree_publication_bundle_present": !tree_publication_bundle_is_absent(tree_result),
            "trace_artifacts": {
                "result": trace_result_path,
                "trace": trace_trace_path,
                "rejects": trace_rejects_path,
            },
            "tree_artifacts": {
                "result": tree_result_path,
            },
        }),
    }))
}

fn lifecycle_comparison_row(
    repo_root: &Path,
    spec: &LifecycleRowSpec,
) -> Result<Value, IndependentConformanceError> {
    let trace_result_path = trace_artifact_path(spec.trace_scenario_id, "result.json");
    let trace_trace_path = trace_artifact_path(spec.trace_scenario_id, "trace.json");
    let trace_counters_path = trace_artifact_path(spec.trace_scenario_id, "counters.json");
    let tree_guardrail_path = tree_root_artifact_path(spec.tree_artifact_name);

    let trace_result = read_json(repo_root, &trace_result_path)?;
    let trace_trace = read_json(repo_root, &trace_trace_path)?;
    let trace_counters = read_json(repo_root, &trace_counters_path)?;
    let tree_guardrail = read_json(repo_root, &tree_guardrail_path)?;
    let missing_artifacts = missing_artifacts([
        (&trace_result_path, &trace_result),
        (&trace_trace_path, &trace_trace),
        (&trace_counters_path, &trace_counters),
        (&tree_guardrail_path, &tree_guardrail),
    ]);

    if !missing_artifacts.is_empty() {
        return Ok(base_row_json(BaseRow {
            row_id: spec.row_id,
            observable_class: spec.observable_class,
            trace_scenario_id: spec.trace_scenario_id,
            tree_case_id: None,
            comparison_state: "missing_artifact",
            missing_artifacts,
            failures: Vec::new(),
            details: json!({}),
        }));
    }

    let trace_result = trace_result.as_ref().expect("missing checked above");
    let trace_trace = trace_trace.as_ref().expect("missing checked above");
    let trace_counters = trace_counters.as_ref().expect("missing checked above");
    let tree_guardrail = tree_guardrail.as_ref().expect("missing checked above");
    let mut failures = Vec::new();

    if !trace_result_is_clean_pass(trace_result) {
        failures.push("tracecalc_result_not_clean_pass".to_string());
    }

    let missing_trace_event_families = spec
        .required_trace_event_families
        .iter()
        .copied()
        .filter(|family| !trace_has_event_family(trace_trace, family))
        .collect::<Vec<_>>();
    for family in &missing_trace_event_families {
        failures.push(format!("missing_trace_event_family:{family}"));
    }

    let missing_tree_event_labels = spec
        .required_tree_event_labels
        .iter()
        .copied()
        .filter(|label| !tree_has_event_label(tree_guardrail, label))
        .collect::<Vec<_>>();
    for label in &missing_tree_event_labels {
        failures.push(format!("missing_tree_event_label:{label}"));
    }

    let trace_evicted_count = counter_value(trace_counters, "overlay_evictions");
    let tree_evicted_count = tree_guardrail
        .get("retention")
        .and_then(|retention| retention.get("evicted_overlay_count_after_release"))
        .and_then(Value::as_u64);
    if trace_evicted_count != tree_evicted_count {
        failures.push("overlay_eviction_count_mismatch".to_string());
    }
    if tree_guardrail
        .get("pinned_reader_stability")
        .and_then(|stability| stability.get("stable"))
        .and_then(Value::as_bool)
        != Some(true)
    {
        failures.push("tree_pinned_reader_stability_not_true".to_string());
    }
    if tree_guardrail
        .get("retention")
        .and_then(|retention| retention.get("cleanup_blocked_while_reader_pinned"))
        .and_then(Value::as_bool)
        != Some(true)
    {
        failures.push("tree_cleanup_not_blocked_while_reader_pinned".to_string());
    }

    Ok(base_row_json(BaseRow {
        row_id: spec.row_id,
        observable_class: spec.observable_class,
        trace_scenario_id: spec.trace_scenario_id,
        tree_case_id: None,
        comparison_state: if failures.is_empty() {
            "matched_lifecycle_surface"
        } else {
            "mismatch"
        },
        missing_artifacts: Vec::new(),
        failures,
        details: json!({
            "asserted_surfaces": [
                "overlay_lifecycle",
                "pinned_reader_stability",
                "retention_release",
                "eviction_count"
            ],
            "trace_evicted_overlay_count": trace_evicted_count,
            "tree_evicted_overlay_count": tree_evicted_count,
            "missing_trace_event_families": missing_trace_event_families,
            "missing_tree_event_labels": missing_tree_event_labels,
            "trace_artifacts": {
                "result": trace_result_path,
                "trace": trace_trace_path,
                "counters": trace_counters_path,
            },
            "tree_artifacts": {
                "retention_guardrail": tree_guardrail_path,
            },
        }),
    }))
}

fn declared_gap_comparison_row(
    repo_root: &Path,
    spec: &DeclaredGapRowSpec,
) -> Result<Value, IndependentConformanceError> {
    let trace_result_path = trace_artifact_path(spec.trace_scenario_id, "result.json");
    let tree_result_path = tree_artifact_path(spec.tree_case_id, "result.json");
    let tree_runtime_effects_path = tree_artifact_path(spec.tree_case_id, "runtime_effects.json");

    let trace_result = read_json(repo_root, &trace_result_path)?;
    let tree_result = read_json(repo_root, &tree_result_path)?;
    let tree_runtime_effects = read_json(repo_root, &tree_runtime_effects_path)?;
    let missing_artifacts = missing_artifacts([
        (&trace_result_path, &trace_result),
        (&tree_result_path, &tree_result),
        (&tree_runtime_effects_path, &tree_runtime_effects),
    ]);

    if !missing_artifacts.is_empty() {
        return Ok(base_row_json(BaseRow {
            row_id: spec.row_id,
            observable_class: spec.observable_class,
            trace_scenario_id: spec.trace_scenario_id,
            tree_case_id: Some(spec.tree_case_id),
            comparison_state: "missing_artifact",
            missing_artifacts,
            failures: Vec::new(),
            details: json!({}),
        }));
    }

    let trace_result = trace_result.as_ref().expect("missing checked above");
    let tree_result = tree_result.as_ref().expect("missing checked above");
    let tree_runtime_effects = tree_runtime_effects
        .as_ref()
        .expect("missing checked above");
    let families = runtime_effect_families(tree_runtime_effects);
    let mut failures = Vec::new();

    if !trace_result_is_clean_pass(trace_result) {
        failures.push("tracecalc_result_not_clean_pass".to_string());
    }
    if tree_result.get("result_state").and_then(Value::as_str) != Some(spec.expected_tree_state) {
        failures.push("treecalc_result_state_mismatch".to_string());
    }
    if !families
        .iter()
        .any(|family| family == spec.required_tree_runtime_effect_family)
    {
        failures.push(format!(
            "missing_tree_runtime_effect_family:{}",
            spec.required_tree_runtime_effect_family
        ));
    }

    Ok(base_row_json(BaseRow {
        row_id: spec.row_id,
        observable_class: spec.observable_class,
        trace_scenario_id: spec.trace_scenario_id,
        tree_case_id: Some(spec.tree_case_id),
        comparison_state: if failures.is_empty() {
            "declared_capability_gap"
        } else {
            "mismatch"
        },
        missing_artifacts: Vec::new(),
        failures,
        details: json!({
            "gap_classification": spec.gap_classification,
            "promotion_policy": "gap_row_is_not_a_conformance_match",
            "required_tree_runtime_effect_family": spec.required_tree_runtime_effect_family,
            "observed_tree_runtime_effect_families": families,
            "trace_artifacts": {
                "result": trace_result_path,
            },
            "tree_artifacts": {
                "result": tree_result_path,
                "runtime_effects": tree_runtime_effects_path,
            },
        }),
    }))
}

fn trace_only_gap_comparison_row(
    repo_root: &Path,
    spec: &TraceOnlyGapRowSpec,
) -> Result<Value, IndependentConformanceError> {
    let trace_result_path = trace_artifact_path(spec.trace_scenario_id, "result.json");
    let trace_trace_path = trace_artifact_path(spec.trace_scenario_id, "trace.json");
    let trace_rejects_path = trace_artifact_path(spec.trace_scenario_id, "rejects.json");

    let trace_result = read_json(repo_root, &trace_result_path)?;
    let trace_trace = read_json(repo_root, &trace_trace_path)?;
    let trace_rejects = read_json(repo_root, &trace_rejects_path)?;
    let missing_artifacts = missing_artifacts([
        (&trace_result_path, &trace_result),
        (&trace_trace_path, &trace_trace),
        (&trace_rejects_path, &trace_rejects),
    ]);

    if !missing_artifacts.is_empty() {
        return Ok(base_row_json(BaseRow {
            row_id: spec.row_id,
            observable_class: spec.observable_class,
            trace_scenario_id: spec.trace_scenario_id,
            tree_case_id: None,
            comparison_state: "missing_artifact",
            missing_artifacts,
            failures: Vec::new(),
            details: json!({}),
        }));
    }

    let trace_result = trace_result.as_ref().expect("missing checked above");
    let trace_trace = trace_trace.as_ref().expect("missing checked above");
    let trace_rejects = trace_rejects.as_ref().expect("missing checked above");
    let mut failures = Vec::new();

    if !trace_result_is_clean_pass(trace_result) {
        failures.push("tracecalc_result_not_clean_pass".to_string());
    }
    if let Some(required_family) = spec.required_trace_event_family
        && !trace_has_event_family(trace_trace, required_family)
    {
        failures.push(format!("missing_trace_event_family:{required_family}"));
    }
    if spec.require_trace_reject && trace_reject_count(trace_rejects) == 0 {
        failures.push("tracecalc_reject_missing".to_string());
    }

    Ok(base_row_json(BaseRow {
        row_id: spec.row_id,
        observable_class: spec.observable_class,
        trace_scenario_id: spec.trace_scenario_id,
        tree_case_id: None,
        comparison_state: if failures.is_empty() {
            "declared_capability_gap"
        } else {
            "mismatch"
        },
        missing_artifacts: Vec::new(),
        failures,
        details: json!({
            "gap_classification": spec.gap_classification,
            "gap_reason": spec.gap_reason,
            "promotion_policy": "trace_only_gap_row_is_not_a_conformance_match",
            "required_trace_event_family": spec.required_trace_event_family,
            "require_trace_reject": spec.require_trace_reject,
            "trace_reject_count": trace_reject_count(trace_rejects),
            "trace_artifacts": {
                "result": trace_result_path,
                "trace": trace_trace_path,
                "rejects": trace_rejects_path,
            },
        }),
    }))
}

fn core_projection_rows(repo_root: &Path) -> Result<Vec<Value>, IndependentConformanceError> {
    let mut case_ids = BTreeSet::new();
    for spec in VALUE_ROW_SPECS {
        case_ids.insert(spec.tree_case_id);
    }
    for spec in NO_PUBLICATION_ROW_SPECS {
        case_ids.insert(spec.tree_case_id);
    }
    for spec in DECLARED_GAP_ROW_SPECS {
        case_ids.insert(spec.tree_case_id);
    }

    case_ids
        .into_iter()
        .map(|case_id| core_projection_row(repo_root, case_id))
        .collect()
}

fn core_projection_row(
    repo_root: &Path,
    case_id: &str,
) -> Result<Value, IndependentConformanceError> {
    let result_path = tree_artifact_path(case_id, "result.json");
    let dependency_path = tree_artifact_path(case_id, "dependency_graph.json");
    let runtime_effects_path = tree_artifact_path(case_id, "runtime_effects.json");
    let result = read_json(repo_root, &result_path)?;
    let dependency_graph = read_json(repo_root, &dependency_path)?;
    let runtime_effects = read_json(repo_root, &runtime_effects_path)?;
    let missing_artifacts = missing_artifacts([
        (&result_path, &result),
        (&dependency_path, &dependency_graph),
        (&runtime_effects_path, &runtime_effects),
    ]);

    if !missing_artifacts.is_empty() {
        return Ok(json!({
            "tree_case_id": case_id,
            "projection_state": "missing_artifact",
            "missing_artifacts": missing_artifacts,
        }));
    }

    let result = result.as_ref().expect("missing checked above");
    let dependency_graph = dependency_graph.as_ref().expect("missing checked above");
    let runtime_effects = runtime_effects.as_ref().expect("missing checked above");

    Ok(json!({
        "tree_case_id": case_id,
        "projection_state": "projection_present",
        "result_state": result.get("result_state").and_then(Value::as_str),
        "publication_bundle_present": !tree_publication_bundle_is_absent(result),
        "reject_kind": result
            .get("reject_detail")
            .and_then(|detail| detail.get("kind"))
            .and_then(Value::as_str),
        "dependency_descriptors": dependency_graph
            .get("descriptors")
            .and_then(Value::as_array)
            .map(|descriptors| descriptors.iter().map(|descriptor| json!({
                "descriptor_id": descriptor.get("descriptor_id"),
                "kind": descriptor.get("kind"),
                "owner_node_id": descriptor.get("owner_node_id"),
                "target_node_id": descriptor.get("target_node_id"),
                "requires_rebind_on_structural_change": descriptor.get("requires_rebind_on_structural_change"),
            })).collect::<Vec<_>>())
            .unwrap_or_default(),
        "runtime_effect_families": runtime_effect_families(runtime_effects),
        "artifact_paths": {
            "result": result_path,
            "dependency_graph": dependency_path,
            "runtime_effects": runtime_effects_path,
        },
    }))
}

fn base_row_json(row: BaseRow<'_>) -> Value {
    json!({
        "row_id": row.row_id,
        "observable_class": row.observable_class,
        "tracecalc_reference_run_id": TRACECALC_REFERENCE_RUN_ID,
        "tracecalc_scenario_id": row.trace_scenario_id,
        "treecalc_reference_run_id": TREECALC_REFERENCE_RUN_ID,
        "treecalc_case_id": row.tree_case_id,
        "comparison_state": row.comparison_state,
        "missing_artifacts": row.missing_artifacts,
        "failures": row.failures,
        "independence_note": "TraceCalc and TreeCalc artifacts are produced by different runners and input schemata; this is not a fully independent evaluator implementation.",
        "details": row.details,
    })
}

fn surface_mapping_json(run_id: &str, relative_artifact_root: &str) -> Value {
    json!({
        "schema_version": INDEPENDENT_CONFORMANCE_SURFACE_MAPPING_SCHEMA_V1,
        "run_id": run_id,
        "artifact_root": relative_artifact_root,
        "tracecalc_reference_run_id": TRACECALC_REFERENCE_RUN_ID,
        "treecalc_reference_run_id": TREECALC_REFERENCE_RUN_ID,
        "observable_surface_authority": "docs/spec/core-engine/w034-formalization/W034_TRACECALC_ORACLE_DEEPENING.md",
        "value_rows": VALUE_ROW_SPECS.iter().map(|spec| json!({
            "row_id": spec.row_id,
            "observable_class": spec.observable_class,
            "tracecalc_scenario_id": spec.trace_scenario_id,
            "treecalc_case_id": spec.tree_case_id,
            "value_mappings": spec.value_mappings.iter().map(|mapping| json!({
                "trace_node_id": mapping.trace_node_id,
                "tree_node_id": mapping.tree_node_id,
            })).collect::<Vec<_>>(),
        })).collect::<Vec<_>>(),
        "no_publication_rows": NO_PUBLICATION_ROW_SPECS.iter().map(|spec| json!({
            "row_id": spec.row_id,
            "observable_class": spec.observable_class,
            "tracecalc_scenario_id": spec.trace_scenario_id,
            "treecalc_case_id": spec.tree_case_id,
            "expected_tree_state": spec.expected_tree_state,
            "required_trace_event_family": spec.required_trace_event_family,
            "require_trace_reject": spec.require_trace_reject,
        })).collect::<Vec<_>>(),
        "lifecycle_rows": LIFECYCLE_ROW_SPECS.iter().map(|spec| json!({
            "row_id": spec.row_id,
            "observable_class": spec.observable_class,
            "tracecalc_scenario_id": spec.trace_scenario_id,
            "treecalc_root_artifact": spec.tree_artifact_name,
            "required_trace_event_families": spec.required_trace_event_families,
            "required_tree_event_labels": spec.required_tree_event_labels,
        })).collect::<Vec<_>>(),
        "declared_gap_rows": DECLARED_GAP_ROW_SPECS.iter().map(|spec| json!({
            "row_id": spec.row_id,
            "observable_class": spec.observable_class,
            "tracecalc_scenario_id": spec.trace_scenario_id,
            "treecalc_case_id": spec.tree_case_id,
            "expected_tree_state": spec.expected_tree_state,
            "required_tree_runtime_effect_family": spec.required_tree_runtime_effect_family,
            "gap_classification": spec.gap_classification,
        })).collect::<Vec<_>>(),
        "trace_only_gap_rows": TRACE_ONLY_GAP_ROW_SPECS.iter().map(|spec| json!({
            "row_id": spec.row_id,
            "observable_class": spec.observable_class,
            "tracecalc_scenario_id": spec.trace_scenario_id,
            "required_trace_event_family": spec.required_trace_event_family,
            "require_trace_reject": spec.require_trace_reject,
            "gap_classification": spec.gap_classification,
            "gap_reason": spec.gap_reason,
        })).collect::<Vec<_>>(),
    })
}

fn comparison_counts(rows: &[Value]) -> ComparisonCounts {
    let mut counts = ComparisonCounts::default();
    for row in rows {
        match row.get("comparison_state").and_then(Value::as_str) {
            Some("matched_exact_value_surface") => counts.exact_value_matches += 1,
            Some("matched_no_publication_surface") => counts.no_publication_matches += 1,
            Some("matched_lifecycle_surface") => counts.lifecycle_surface_matches += 1,
            Some("declared_capability_gap") => counts.declared_gaps += 1,
            Some("missing_artifact") => counts.missing_artifacts += 1,
            Some("mismatch") => counts.unexpected_mismatches += 1,
            _ => {}
        }
    }
    counts
}

fn trace_result_is_clean_pass(result: &Value) -> bool {
    result.get("result_state").and_then(Value::as_str) == Some("passed")
        && empty_array(result.get("assertion_failures"))
        && empty_array(result.get("validation_failures"))
        && empty_array(result.get("conformance_mismatches"))
}

fn tree_publication_bundle_is_absent(result: &Value) -> bool {
    result
        .get("publication_bundle")
        .is_none_or(serde_json::Value::is_null)
}

fn trace_has_event_family(trace: &Value, required_family: &str) -> bool {
    trace
        .get("events")
        .and_then(Value::as_array)
        .is_some_and(|events| {
            events.iter().any(|event| {
                event.get("normalized_event_family").and_then(Value::as_str)
                    == Some(required_family)
            })
        })
}

fn tree_has_event_label(tree_guardrail: &Value, required_label: &str) -> bool {
    tree_guardrail
        .get("events")
        .and_then(Value::as_array)
        .is_some_and(|events| {
            events
                .iter()
                .any(|event| event.get("label").and_then(Value::as_str) == Some(required_label))
        })
}

fn trace_reject_count(rejects: &Value) -> usize {
    rejects
        .get("rejects")
        .and_then(Value::as_array)
        .map_or(0, std::vec::Vec::len)
}

fn counter_value(counters: &Value, counter_name: &str) -> Option<u64> {
    counters
        .get("counters")
        .and_then(Value::as_array)?
        .iter()
        .find(|entry| entry.get("counter").and_then(Value::as_str) == Some(counter_name))?
        .get("value")?
        .as_u64()
}

fn trace_value_map(published_view: &Value) -> BTreeMap<String, String> {
    published_view
        .get("node_values")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|entry| {
            Some((
                entry.get("node_id")?.as_str()?.to_string(),
                entry.get("value")?.as_str()?.to_string(),
            ))
        })
        .collect()
}

fn tree_value_map(published_values: &Value) -> BTreeMap<String, String> {
    published_values
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|entry| {
            Some((
                entry.get("node_id")?.as_u64()?.to_string(),
                entry.get("value")?.as_str()?.to_string(),
            ))
        })
        .collect()
}

fn runtime_effect_families(runtime_effects: &Value) -> Vec<String> {
    let mut families = runtime_effects
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|effect| effect.get("family").and_then(Value::as_str))
        .map(str::to_string)
        .collect::<Vec<_>>();
    families.sort();
    families.dedup();
    families
}

fn missing_artifacts<'a>(
    artifacts: impl IntoIterator<Item = (&'a String, &'a Option<Value>)>,
) -> Vec<String> {
    artifacts
        .into_iter()
        .filter(|(_, value)| value.is_none())
        .map(|(path, _)| path.clone())
        .collect()
}

fn empty_array(value: Option<&Value>) -> bool {
    value
        .and_then(Value::as_array)
        .is_none_or(std::vec::Vec::is_empty)
}

fn read_json(
    repo_root: &Path,
    relative_path: &str,
) -> Result<Option<Value>, IndependentConformanceError> {
    let path = repo_root.join(relative_path);
    if !path.exists() {
        return Ok(None);
    }
    let content =
        fs::read_to_string(&path).map_err(|source| IndependentConformanceError::ReadArtifact {
            path: path.display().to_string(),
            source,
        })?;
    serde_json::from_str(&content).map(Some).map_err(|source| {
        IndependentConformanceError::ParseJson {
            path: path.display().to_string(),
            source,
        }
    })
}

fn create_directory(path: &Path) -> Result<(), IndependentConformanceError> {
    fs::create_dir_all(path).map_err(|source| IndependentConformanceError::CreateDirectory {
        path: path.display().to_string(),
        source,
    })
}

fn write_json(path: &Path, value: &Value) -> Result<(), IndependentConformanceError> {
    let content = serde_json::to_string_pretty(value).expect("JSON serialization should succeed");
    fs::write(path, format!("{content}\n")).map_err(|source| {
        IndependentConformanceError::WriteFile {
            path: path.display().to_string(),
            source,
        }
    })
}

fn trace_artifact_path(scenario_id: &str, artifact_name: &str) -> String {
    relative_artifact_path([
        "docs",
        "test-runs",
        "core-engine",
        "tracecalc-reference-machine",
        TRACECALC_REFERENCE_RUN_ID,
        "scenarios",
        scenario_id,
        artifact_name,
    ])
}

fn tree_artifact_path(case_id: &str, artifact_name: &str) -> String {
    relative_artifact_path([
        "docs",
        "test-runs",
        "core-engine",
        "treecalc-local",
        TREECALC_REFERENCE_RUN_ID,
        "cases",
        case_id,
        artifact_name,
    ])
}

fn tree_root_artifact_path(artifact_name: &str) -> String {
    relative_artifact_path([
        "docs",
        "test-runs",
        "core-engine",
        "treecalc-local",
        TREECALC_REFERENCE_RUN_ID,
        artifact_name,
    ])
}

fn required_artifacts(run_id: &str) -> Vec<String> {
    [
        "run_summary.json",
        "surface_mapping.json",
        "comparisons/treecalc_tracecalc_differential.json",
        "comparisons/core_engine_projection_differential.json",
        "replay-appliance/bundle_manifest.json",
        "replay-appliance/validation/bundle_validation.json",
    ]
    .iter()
    .map(|artifact| {
        relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "independent-conformance",
            run_id,
            artifact,
        ])
    })
    .chain([
        relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "tracecalc-reference-machine",
            TRACECALC_REFERENCE_RUN_ID,
            "run_summary.json",
        ]),
        relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "treecalc-local",
            TREECALC_REFERENCE_RUN_ID,
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
    fn independent_conformance_runner_writes_clean_diff_packet() {
        let repo_root = unique_temp_repo();
        create_tracecalc_artifacts(&repo_root);
        create_treecalc_artifacts(&repo_root);

        let summary = IndependentConformanceRunner::new()
            .execute(&repo_root, "independent-test")
            .expect("independent conformance packet should write");

        assert_eq!(summary.comparison_row_count, 15);
        assert_eq!(summary.exact_value_match_count, 5);
        assert_eq!(summary.no_publication_match_count, 3);
        assert_eq!(summary.lifecycle_surface_match_count, 1);
        assert_eq!(summary.declared_gap_count, 6);
        assert_eq!(summary.missing_artifact_count, 0);
        assert_eq!(summary.unexpected_mismatch_count, 0);

        let validation = read_required_json(
            &repo_root,
            "docs/test-runs/core-engine/independent-conformance/independent-test/replay-appliance/validation/bundle_validation.json",
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
            "oxcalc-independent-conformance-test-{}-{nanos}",
            std::process::id()
        ));
        let repo_root = base.join("OxCalc");
        fs::create_dir_all(&repo_root).unwrap();
        repo_root
    }

    fn create_tracecalc_artifacts(repo_root: &Path) {
        write_source_run_summary(
            repo_root,
            &relative_artifact_path([
                "docs",
                "test-runs",
                "core-engine",
                "tracecalc-reference-machine",
                TRACECALC_REFERENCE_RUN_ID,
                "run_summary.json",
            ]),
        );
        trace_result(repo_root, "tc_accept_publish_001");
        trace_published(
            repo_root,
            "tc_accept_publish_001",
            &[("A", "2"), ("B", "2")],
        );
        trace_result(repo_root, "tc_multinode_dag_publish_001");
        trace_published(
            repo_root,
            "tc_multinode_dag_publish_001",
            &[("A", "3"), ("B", "3"), ("C", "3"), ("D", "6")],
        );
        trace_result(repo_root, "tc_let_lambda_carrier_publish_001");
        trace_published(
            repo_root,
            "tc_let_lambda_carrier_publish_001",
            &[("A", "10"), ("B", "15")],
        );
        trace_result(repo_root, "tc_verify_clean_no_publish_001");
        trace_trace(
            repo_root,
            "tc_verify_clean_no_publish_001",
            &["candidate.verified_clean"],
        );
        trace_rejects(repo_root, "tc_verify_clean_no_publish_001", &[]);
        trace_result(repo_root, "tc_reject_no_publish_001");
        trace_trace(repo_root, "tc_reject_no_publish_001", &[]);
        trace_rejects(repo_root, "tc_reject_no_publish_001", &["reject"]);
        trace_result(repo_root, "tc_dynamic_dep_switch_001");
        trace_result(repo_root, "tc_let_lambda_runtime_effect_001");
        trace_result(repo_root, "tc_w034_let_lambda_higher_order_replay_001");
        trace_published(
            repo_root,
            "tc_w034_let_lambda_higher_order_replay_001",
            &[("Base", "12"), ("Out", "17")],
        );
        trace_trace(repo_root, "tc_w034_let_lambda_higher_order_replay_001", &[]);
        trace_rejects(repo_root, "tc_w034_let_lambda_higher_order_replay_001", &[]);
        trace_result(repo_root, "tc_w034_replay_equivalent_independent_order_001");
        trace_published(
            repo_root,
            "tc_w034_replay_equivalent_independent_order_001",
            &[("Left", "4"), ("Top", "5"), ("Check", "9")],
        );
        trace_result(repo_root, "tc_w034_capability_fence_reject_001");
        trace_trace(
            repo_root,
            "tc_w034_capability_fence_reject_001",
            &["reject.issued"],
        );
        trace_rejects(
            repo_root,
            "tc_w034_capability_fence_reject_001",
            &["reject"],
        );
        trace_result(repo_root, "tc_w034_overlay_eviction_after_unpin_001");
        trace_trace(
            repo_root,
            "tc_w034_overlay_eviction_after_unpin_001",
            W034_OVERLAY_TRACE_FAMILIES,
        );
        trace_rejects(repo_root, "tc_w034_overlay_eviction_after_unpin_001", &[]);
        trace_counters(
            repo_root,
            "tc_w034_overlay_eviction_after_unpin_001",
            &[("overlay_evictions", 3)],
        );
        trace_result(repo_root, "tc_w034_dynamic_dependency_negative_001");
        trace_result(repo_root, "tc_w034_snapshot_fence_reject_001");
        trace_trace(
            repo_root,
            "tc_w034_snapshot_fence_reject_001",
            &["reject.issued"],
        );
        trace_rejects(repo_root, "tc_w034_snapshot_fence_reject_001", &["reject"]);
    }

    fn create_treecalc_artifacts(repo_root: &Path) {
        write_source_run_summary(
            repo_root,
            &relative_artifact_path([
                "docs",
                "test-runs",
                "core-engine",
                "treecalc-local",
                TREECALC_REFERENCE_RUN_ID,
                "run_summary.json",
            ]),
        );
        tree_case(
            repo_root,
            "tc_local_tracecalc_accept_publish_equiv_001",
            "published",
            true,
        );
        tree_published(
            repo_root,
            "tc_local_tracecalc_accept_publish_equiv_001",
            &[(3, "2")],
        );
        tree_case(
            repo_root,
            "tc_local_tracecalc_multinode_dag_equiv_001",
            "published",
            true,
        );
        tree_published(
            repo_root,
            "tc_local_tracecalc_multinode_dag_equiv_001",
            &[(3, "3"), (4, "3"), (5, "6")],
        );
        tree_case(
            repo_root,
            "tc_local_let_lambda_capture_publish_001",
            "published",
            true,
        );
        tree_published(
            repo_root,
            "tc_local_let_lambda_capture_publish_001",
            &[(3, "15")],
        );
        tree_case(
            repo_root,
            "tc_local_w034_higher_order_let_lambda_publish_001",
            "published",
            true,
        );
        tree_published(
            repo_root,
            "tc_local_w034_higher_order_let_lambda_publish_001",
            &[(3, "17")],
        );
        tree_case(
            repo_root,
            "tc_local_w034_independent_order_equiv_001",
            "published",
            true,
        );
        tree_published(
            repo_root,
            "tc_local_w034_independent_order_equiv_001",
            &[(4, "4"), (5, "5"), (6, "9")],
        );
        tree_case(
            repo_root,
            "tc_local_verified_clean_001",
            "verified_clean",
            false,
        );
        tree_case(
            repo_root,
            "tc_local_capability_sensitive_reject_001",
            "rejected",
            false,
        );
        tree_case(repo_root, "tc_local_dynamic_reject_001", "rejected", false);
        tree_runtime_effects(
            repo_root,
            "tc_local_dynamic_reject_001",
            &["DynamicDependency"],
        );
        tree_case(
            repo_root,
            "tc_local_lambda_host_sensitive_reject_001",
            "rejected",
            false,
        );
        tree_runtime_effects(
            repo_root,
            "tc_local_lambda_host_sensitive_reject_001",
            &["ExecutionRestriction"],
        );
        tree_retention_guardrail(repo_root);
    }

    fn trace_result(repo_root: &Path, scenario_id: &str) {
        write_json_test(
            repo_root,
            &trace_artifact_path(scenario_id, "result.json"),
            json!({
                "result_state": "passed",
                "assertion_failures": [],
                "validation_failures": [],
                "conformance_mismatches": [],
            }),
        );
    }

    fn trace_published(repo_root: &Path, scenario_id: &str, values: &[(&str, &str)]) {
        write_json_test(
            repo_root,
            &trace_artifact_path(scenario_id, "published_view.json"),
            json!({
                "scenario_id": scenario_id,
                "node_values": values.iter().map(|(node_id, value)| json!({
                    "node_id": node_id,
                    "value": value,
                })).collect::<Vec<_>>(),
            }),
        );
    }

    fn trace_trace(repo_root: &Path, scenario_id: &str, families: &[&str]) {
        write_json_test(
            repo_root,
            &trace_artifact_path(scenario_id, "trace.json"),
            json!({
                "events": families.iter().map(|family| json!({
                    "normalized_event_family": family,
                })).collect::<Vec<_>>(),
            }),
        );
    }

    fn trace_rejects(repo_root: &Path, scenario_id: &str, reject_ids: &[&str]) {
        write_json_test(
            repo_root,
            &trace_artifact_path(scenario_id, "rejects.json"),
            json!({
                "rejects": reject_ids.iter().map(|reject_id| json!({
                    "reject_id": reject_id,
                })).collect::<Vec<_>>(),
            }),
        );
    }

    fn trace_counters(repo_root: &Path, scenario_id: &str, counters: &[(&str, u64)]) {
        write_json_test(
            repo_root,
            &trace_artifact_path(scenario_id, "counters.json"),
            json!({
                "counters": counters.iter().map(|(counter, value)| json!({
                    "counter": counter,
                    "value": value,
                })).collect::<Vec<_>>(),
            }),
        );
    }

    fn tree_case(repo_root: &Path, case_id: &str, state: &str, published: bool) {
        write_json_test(
            repo_root,
            &tree_artifact_path(case_id, "result.json"),
            json!({
                "case_id": case_id,
                "result_state": state,
                "publication_bundle": if published { json!({ "publication_id": case_id }) } else { Value::Null },
                "reject_detail": if state == "rejected" { json!({ "kind": "ProjectedReject" }) } else { Value::Null },
            }),
        );
        write_json_test(
            repo_root,
            &tree_artifact_path(case_id, "dependency_graph.json"),
            json!({
                "descriptors": [
                    {
                        "descriptor_id": format!("{case_id}:d1"),
                        "kind": "StaticDirect",
                        "owner_node_id": 3,
                        "target_node_id": 2,
                        "requires_rebind_on_structural_change": false
                    }
                ]
            }),
        );
        tree_runtime_effects(repo_root, case_id, &[]);
        tree_published(repo_root, case_id, &[]);
    }

    fn tree_published(repo_root: &Path, case_id: &str, values: &[(u64, &str)]) {
        write_json_test(
            repo_root,
            &tree_artifact_path(case_id, "published_values.json"),
            json!(
                values
                    .iter()
                    .map(|(node_id, value)| json!({
                        "node_id": node_id,
                        "value": value,
                    }))
                    .collect::<Vec<_>>()
            ),
        );
    }

    fn tree_runtime_effects(repo_root: &Path, case_id: &str, families: &[&str]) {
        write_json_test(
            repo_root,
            &tree_artifact_path(case_id, "runtime_effects.json"),
            json!(
                families
                    .iter()
                    .map(|family| json!({ "family": family }))
                    .collect::<Vec<_>>()
            ),
        );
    }

    fn tree_retention_guardrail(repo_root: &Path) {
        write_json_test(
            repo_root,
            &tree_root_artifact_path("retention_guardrail.json"),
            json!({
                "events": W034_OVERLAY_TREE_EVENT_LABELS.iter().map(|label| json!({
                    "label": label,
                })).collect::<Vec<_>>(),
                "pinned_reader_stability": {
                    "stable": true,
                },
                "retention": {
                    "cleanup_blocked_while_reader_pinned": true,
                    "evicted_overlay_count_after_release": 3,
                },
            }),
        );
    }

    fn write_source_run_summary(repo_root: &Path, relative_path: &str) {
        write_json_test(
            repo_root,
            relative_path,
            json!({ "status": "source-present" }),
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
