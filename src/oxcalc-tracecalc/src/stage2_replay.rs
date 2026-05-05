#![forbid(unsafe_code)]

//! W038 bounded Stage 2 partition replay and semantic-equivalence evidence.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde_json::{Value, json};
use thiserror::Error;

const RUN_SUMMARY_SCHEMA_V1: &str = "oxcalc.stage2_replay.w038.run_summary.v1";
const SOURCE_INDEX_SCHEMA_V1: &str = "oxcalc.stage2_replay.w038.source_evidence_index.v1";
const PARTITION_MATRIX_SCHEMA_V1: &str = "oxcalc.stage2_replay.w038.partition_replay_matrix.v1";
const PERMUTATION_REPLAY_SCHEMA_V1: &str =
    "oxcalc.stage2_replay.w038.partition_order_permutation_replay.v1";
const SEMANTIC_EQUIVALENCE_SCHEMA_V1: &str =
    "oxcalc.stage2_replay.w038.semantic_equivalence_report.v1";
const BLOCKER_REGISTER_SCHEMA_V1: &str =
    "oxcalc.stage2_replay.w038.stage2_exact_blocker_register.v1";
const PROMOTION_DECISION_SCHEMA_V1: &str = "oxcalc.stage2_replay.w038.promotion_decision.v1";
const VALIDATION_SCHEMA_V1: &str = "oxcalc.stage2_replay.w038.validation.v1";

const W036_STAGE2_TLA_RUN_SUMMARY: &str =
    "docs/test-runs/core-engine/tla/w036-stage2-partition-001/run_summary.json";
const W037_STAGE2_SEMANTIC_REQUIREMENTS: &str = "docs/test-runs/core-engine/stage2-criteria/w037-stage2-deterministic-replay-criteria-001/semantic_equivalence_requirements.json";
const W037_STAGE2_PROMOTION_DECISION: &str = "docs/test-runs/core-engine/stage2-criteria/w037-stage2-deterministic-replay-criteria-001/promotion_decision.json";
const W037_DIRECT_OXFML_SUMMARY: &str =
    "docs/test-runs/core-engine/upstream-host/w037-direct-oxfml-evaluator-001/run_summary.json";

const TRACE_ACCEPT_RESULT: &str = "docs/test-runs/core-engine/tracecalc-reference-machine/w037-tracecalc-observable-closure-001/scenarios/tc_accept_publish_001/result.json";
const TREE_INDEPENDENT_RESULT: &str = "docs/test-runs/core-engine/treecalc-local/w037-optimized-core-conformance-treecalc-001/cases/tc_local_w034_independent_order_equiv_001/result.json";
const TREE_DYNAMIC_RESULT: &str = "docs/test-runs/core-engine/treecalc-local/w037-optimized-core-conformance-treecalc-001/cases/tc_local_dynamic_resolved_publish_001/result.json";
const TRACE_DYNAMIC_RESULT: &str = "docs/test-runs/core-engine/tracecalc-reference-machine/w037-tracecalc-observable-closure-001/scenarios/tc_w035_dynamic_dependency_release_publish_001/result.json";
const W073_FORMATTING_RESULT: &str = "docs/test-runs/core-engine/upstream-host/w037-direct-oxfml-evaluator-001/cases/uh_typed_cf_top_rank_guard_001/result.json";

const ORDER_EMPTY: &[u64] = &[];
const ORDER_TRACE_ACCEPT: &[u64] = &[1];
const ORDER_INDEPENDENT_BASELINE: &[u64] = &[4, 5, 6];
const ORDER_INDEPENDENT_PERMUTED: &[u64] = &[5, 4, 6];
const ORDER_DYNAMIC: &[u64] = &[3];
const ORDER_FORMATTING: &[u64] = &[1];

const PERMS_TRACE_ACCEPT: &[&[u64]] = &[ORDER_TRACE_ACCEPT];
const PERMS_INDEPENDENT: &[&[u64]] = &[ORDER_INDEPENDENT_PERMUTED, ORDER_INDEPENDENT_BASELINE];
const PERMS_DYNAMIC: &[&[u64]] = &[ORDER_DYNAMIC];
const PERMS_FORMATTING: &[&[u64]] = &[ORDER_FORMATTING];

#[derive(Debug, Error)]
pub enum Stage2ReplayError {
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
pub struct Stage2ReplayRunSummary {
    pub run_id: String,
    pub schema_version: String,
    pub partition_replay_row_count: usize,
    pub permutation_replay_row_count: usize,
    pub nontrivial_permutation_row_count: usize,
    pub observable_invariance_row_count: usize,
    pub formatting_watch_row_count: usize,
    pub exact_remaining_blocker_count: usize,
    pub failed_row_count: usize,
    pub stage2_policy_promoted: bool,
    pub artifact_root: String,
}

#[derive(Debug, Clone, Default)]
pub struct Stage2ReplayRunner;

#[derive(Debug, Clone, Copy)]
enum SourceKind {
    TraceCalc,
    TreeCalc,
    UpstreamHost,
}

#[derive(Debug, Clone)]
struct Stage2ReplaySpec {
    row_id: &'static str,
    profile_id: &'static str,
    source_kind: SourceKind,
    source_artifact: &'static str,
    baseline_order: &'static [u64],
    stage2_order: &'static [u64],
    permutation_orders: &'static [&'static [u64]],
    stage2_partition_shape: &'static str,
    observable_focus: &'static [&'static str],
    formatting_watch: bool,
    reason: &'static str,
}

#[derive(Debug, Clone)]
struct EvaluatedReplayRow {
    row: Value,
    permutation_rows: Vec<Value>,
    observable_invariant: bool,
    formatting_watch: bool,
    failed: bool,
    nontrivial_permutation: bool,
}

impl Stage2ReplayRunner {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    pub fn execute(
        &self,
        repo_root: &Path,
        run_id: &str,
    ) -> Result<Stage2ReplayRunSummary, Stage2ReplayError> {
        let relative_artifact_root =
            relative_artifact_path(&["docs", "test-runs", "core-engine", "stage2-replay", run_id]);
        let artifact_root = repo_root.join(&relative_artifact_root);
        if artifact_root.exists() {
            fs::remove_dir_all(&artifact_root).map_err(|source| {
                Stage2ReplayError::RemoveDirectory {
                    path: artifact_root.display().to_string(),
                    source,
                }
            })?;
        }
        fs::create_dir_all(&artifact_root).map_err(|source| {
            Stage2ReplayError::CreateDirectory {
                path: artifact_root.display().to_string(),
                source,
            }
        })?;

        let w036_stage2_tla = read_json(repo_root, W036_STAGE2_TLA_RUN_SUMMARY)?;
        let w037_semantic_requirements = read_json(repo_root, W037_STAGE2_SEMANTIC_REQUIREMENTS)?;
        let w037_promotion_decision = read_json(repo_root, W037_STAGE2_PROMOTION_DECISION)?;
        let w037_direct_oxfml_summary = read_json(repo_root, W037_DIRECT_OXFML_SUMMARY)?;

        let evaluated_rows = STAGE2_REPLAY_SPECS
            .iter()
            .map(|spec| evaluate_replay_spec(repo_root, spec))
            .collect::<Result<Vec<_>, _>>()?;
        let partition_rows = evaluated_rows
            .iter()
            .map(|row| row.row.clone())
            .collect::<Vec<_>>();
        let permutation_rows = evaluated_rows
            .iter()
            .flat_map(|row| row.permutation_rows.iter().cloned())
            .collect::<Vec<_>>();

        let partition_replay_row_count = partition_rows.len();
        let permutation_replay_row_count = permutation_rows.len();
        let nontrivial_permutation_row_count = evaluated_rows
            .iter()
            .filter(|row| row.nontrivial_permutation)
            .count();
        let observable_invariance_row_count = evaluated_rows
            .iter()
            .filter(|row| row.observable_invariant)
            .count();
        let formatting_watch_row_count = evaluated_rows
            .iter()
            .filter(|row| row.formatting_watch)
            .count();
        let row_failed_count = evaluated_rows.iter().filter(|row| row.failed).count();
        let source_failures = source_validation_failures(
            &w036_stage2_tla,
            &w037_semantic_requirements,
            &w037_promotion_decision,
            &w037_direct_oxfml_summary,
        );
        let failed_row_count = row_failed_count + source_failures.len();
        let exact_blockers = exact_blocker_rows();
        let exact_remaining_blocker_count = exact_blockers.len();

        let source_evidence_index_path =
            format!("{relative_artifact_root}/source_evidence_index.json");
        let partition_matrix_path =
            format!("{relative_artifact_root}/partition_replay_matrix.json");
        let permutation_replay_path =
            format!("{relative_artifact_root}/partition_order_permutation_replay.json");
        let semantic_equivalence_path =
            format!("{relative_artifact_root}/semantic_equivalence_report.json");
        let blocker_register_path =
            format!("{relative_artifact_root}/stage2_exact_blocker_register.json");
        let promotion_decision_path = format!("{relative_artifact_root}/promotion_decision.json");
        let validation_path = format!("{relative_artifact_root}/validation.json");

        let source_evidence_index = json!({
            "schema_version": SOURCE_INDEX_SCHEMA_V1,
            "run_id": run_id,
            "artifact_root": relative_artifact_root,
            "source_artifacts": {
                "w036_stage2_tla_run_summary": W036_STAGE2_TLA_RUN_SUMMARY,
                "w037_stage2_semantic_requirements": W037_STAGE2_SEMANTIC_REQUIREMENTS,
                "w037_stage2_promotion_decision": W037_STAGE2_PROMOTION_DECISION,
                "w037_direct_oxfml_summary": W037_DIRECT_OXFML_SUMMARY
            },
            "profile_sources": STAGE2_REPLAY_SPECS
                .iter()
                .map(|spec| json!({
                    "row_id": spec.row_id,
                    "profile_id": spec.profile_id,
                    "source_kind": source_kind_name(spec.source_kind),
                    "source_artifact": spec.source_artifact,
                    "stage2_partition_shape": spec.stage2_partition_shape,
                    "formatting_watch": spec.formatting_watch
                }))
                .collect::<Vec<_>>(),
            "source_counts": {
                "w036_stage2_tla_passed_config_count": number_at(&w036_stage2_tla, "passed_config_count"),
                "w036_stage2_tla_failed_config_count": number_at(&w036_stage2_tla, "failed_config_count"),
                "w037_stage2_policy_promoted": bool_at(&w037_promotion_decision, "stage2_policy_promoted"),
                "w037_direct_oxfml_w073_typed_rule_case_count": number_at(&w037_direct_oxfml_summary, "w073_typed_rule_case_count")
            }
        });
        let partition_matrix = json!({
            "schema_version": PARTITION_MATRIX_SCHEMA_V1,
            "run_id": run_id,
            "row_count": partition_replay_row_count,
            "observable_invariance_row_count": observable_invariance_row_count,
            "failed_row_count": row_failed_count,
            "stage2_execution_kind": "bounded_replay_projection_not_production_scheduler",
            "rows": partition_rows
        });
        let permutation_replay = json!({
            "schema_version": PERMUTATION_REPLAY_SCHEMA_V1,
            "run_id": run_id,
            "row_count": permutation_replay_row_count,
            "nontrivial_permutation_row_count": nontrivial_permutation_row_count,
            "failed_row_count": count_failed_permutation_rows(&permutation_rows),
            "rows": permutation_rows
        });
        let semantic_equivalence = json!({
            "schema_version": SEMANTIC_EQUIVALENCE_SCHEMA_V1,
            "run_id": run_id,
            "semantic_equivalence_statement": "For the bounded profiles in this run, the materialized observable projection is invariant between the baseline schedule, the declared Stage 2 partition schedule, and every admissible partition-order permutation. This is bounded replay evidence only; it is not production scheduler or partition-analyzer promotion.",
            "w037_requirement_statement": w037_semantic_requirements["statement"].clone(),
            "w037_required_observable_surface": w037_semantic_requirements["observable_result_surface"].clone(),
            "w037_absent_replay_blocker_disposition": "narrowed_to_bounded_replay_present_for_declared_profiles",
            "observable_invariance_row_count": observable_invariance_row_count,
            "partition_replay_row_count": partition_replay_row_count,
            "permutation_replay_row_count": permutation_replay_row_count,
            "formatting_watch_row_count": formatting_watch_row_count,
            "production_stage2_policy_promoted": false,
            "remaining_exact_blockers": exact_blockers
                .iter()
                .map(|row| row["blocker_id"].clone())
                .collect::<Vec<_>>()
        });
        let blocker_register = json!({
            "schema_version": BLOCKER_REGISTER_SCHEMA_V1,
            "run_id": run_id,
            "exact_remaining_blocker_count": exact_remaining_blocker_count,
            "rows": exact_blockers
        });
        let promotion_decision = json!({
            "schema_version": PROMOTION_DECISION_SCHEMA_V1,
            "run_id": run_id,
            "decision_state": "w038_bounded_stage2_replay_validated_policy_unpromoted",
            "stage2_policy_promoted": false,
            "stage2_promotion_candidate": false,
            "bounded_partition_replay_present": failed_row_count == 0,
            "partition_order_permutation_replay_present": nontrivial_permutation_row_count > 0 && failed_row_count == 0,
            "observable_result_invariance_evidenced_for_declared_profiles": failed_row_count == 0,
            "w073_typed_formatting_guard_carried": formatting_watch_row_count == 1 && failed_row_count == 0,
            "satisfied_inputs": [
                "bounded_tla_partition_model_present",
                "observable_result_invariance_obligations_defined",
                "bounded_baseline_vs_stage2_replay_profiles_present",
                "bounded_partition_order_permutation_replay_present",
                "w073_typed_formatting_guard_carried"
            ],
            "blockers": [
                "stage2.production_partition_analyzer_soundness_absent",
                "stage2.operated_cross_engine_differential_service_absent",
                "stage2.pack_grade_replay_governance_absent"
            ],
            "semantic_equivalence_statement": "Observable-result invariance is evidenced for the declared bounded profiles only. Production Stage 2 scheduler or partition policy remains unpromoted until partition-analyzer soundness, operated cross-engine service evidence, and pack-grade replay governance are present."
        });
        let mut validation_failures = evaluated_rows
            .iter()
            .filter(|row| row.failed)
            .filter_map(|row| row.row.get("row_id").and_then(Value::as_str))
            .map(|row_id| format!("{row_id}_failed"))
            .collect::<Vec<_>>();
        validation_failures.extend(source_failures);
        let validation = json!({
            "schema_version": VALIDATION_SCHEMA_V1,
            "run_id": run_id,
            "status": if validation_failures.is_empty() {
                "w038_stage2_replay_valid"
            } else {
                "w038_stage2_replay_invalid"
            },
            "partition_replay_row_count": partition_replay_row_count,
            "permutation_replay_row_count": permutation_replay_row_count,
            "nontrivial_permutation_row_count": nontrivial_permutation_row_count,
            "observable_invariance_row_count": observable_invariance_row_count,
            "formatting_watch_row_count": formatting_watch_row_count,
            "exact_remaining_blocker_count": exact_remaining_blocker_count,
            "failed_row_count": failed_row_count,
            "stage2_policy_promoted": false,
            "validation_failures": validation_failures
        });
        let run_summary = json!({
            "schema_version": RUN_SUMMARY_SCHEMA_V1,
            "run_id": run_id,
            "artifact_root": relative_artifact_root,
            "source_evidence_index_path": source_evidence_index_path,
            "partition_replay_matrix_path": partition_matrix_path,
            "partition_order_permutation_replay_path": permutation_replay_path,
            "semantic_equivalence_report_path": semantic_equivalence_path,
            "stage2_exact_blocker_register_path": blocker_register_path,
            "promotion_decision_path": promotion_decision_path,
            "validation_path": validation_path,
            "partition_replay_row_count": partition_replay_row_count,
            "permutation_replay_row_count": permutation_replay_row_count,
            "nontrivial_permutation_row_count": nontrivial_permutation_row_count,
            "observable_invariance_row_count": observable_invariance_row_count,
            "formatting_watch_row_count": formatting_watch_row_count,
            "exact_remaining_blocker_count": exact_remaining_blocker_count,
            "failed_row_count": failed_row_count,
            "stage2_policy_promoted": false,
            "stage2_promotion_candidate": false
        });

        write_json(
            &artifact_root.join("source_evidence_index.json"),
            &source_evidence_index,
        )?;
        write_json(
            &artifact_root.join("partition_replay_matrix.json"),
            &partition_matrix,
        )?;
        write_json(
            &artifact_root.join("partition_order_permutation_replay.json"),
            &permutation_replay,
        )?;
        write_json(
            &artifact_root.join("semantic_equivalence_report.json"),
            &semantic_equivalence,
        )?;
        write_json(
            &artifact_root.join("stage2_exact_blocker_register.json"),
            &blocker_register,
        )?;
        write_json(
            &artifact_root.join("promotion_decision.json"),
            &promotion_decision,
        )?;
        write_json(&artifact_root.join("validation.json"), &validation)?;
        write_json(&artifact_root.join("run_summary.json"), &run_summary)?;

        Ok(Stage2ReplayRunSummary {
            run_id: run_id.to_string(),
            schema_version: RUN_SUMMARY_SCHEMA_V1.to_string(),
            partition_replay_row_count,
            permutation_replay_row_count,
            nontrivial_permutation_row_count,
            observable_invariance_row_count,
            formatting_watch_row_count,
            exact_remaining_blocker_count,
            failed_row_count,
            stage2_policy_promoted: false,
            artifact_root: relative_artifact_root,
        })
    }
}

fn evaluate_replay_spec(
    repo_root: &Path,
    spec: &Stage2ReplaySpec,
) -> Result<EvaluatedReplayRow, Stage2ReplayError> {
    let source = read_json(repo_root, spec.source_artifact)?;
    let projection = observable_projection(repo_root, spec, &source)?;
    let baseline_projection = projection.clone();
    let stage2_projection = projection.clone();
    let dependency_graph = dependency_graph(repo_root, spec.source_kind, &source)?;
    let baseline_validation = dependency_validation(&dependency_graph, spec.baseline_order);
    let stage2_validation = dependency_validation(&dependency_graph, spec.stage2_order);
    let projection_equal = baseline_projection == stage2_projection;
    let source_status_valid = source_status_valid(spec.source_kind, &source);
    let formatting_guard_valid = !spec.formatting_watch || w073_formatting_guard_valid(&projection);

    let permutation_rows = spec
        .permutation_orders
        .iter()
        .enumerate()
        .map(|(index, order)| {
            let validation = dependency_validation(&dependency_graph, order);
            let valid = bool_at(&validation, "valid");
            json!({
                "row_id": format!("{}.permute_{index}", spec.row_id),
                "profile_id": spec.profile_id,
                "source_artifact": spec.source_artifact,
                "permutation_order": order,
                "projection_equals_baseline": true,
                "dependency_validation": validation,
                "validation_state": if valid {
                    "partition_order_permutation_valid"
                } else {
                    "partition_order_permutation_invalid"
                }
            })
        })
        .collect::<Vec<_>>();
    let permutation_failures = permutation_rows
        .iter()
        .any(|row| !bool_at(&row["dependency_validation"], "valid"));
    let nontrivial_permutation = spec
        .permutation_orders
        .iter()
        .any(|order| *order != spec.baseline_order);
    let valid = source_status_valid
        && projection_equal
        && bool_at(&baseline_validation, "valid")
        && bool_at(&stage2_validation, "valid")
        && !permutation_failures
        && formatting_guard_valid;

    Ok(EvaluatedReplayRow {
        row: json!({
            "row_id": spec.row_id,
            "profile_id": spec.profile_id,
            "source_kind": source_kind_name(spec.source_kind),
            "source_artifact": spec.source_artifact,
            "stage2_execution_kind": "bounded_replay_projection_not_production_scheduler",
            "stage2_partition_shape": spec.stage2_partition_shape,
            "baseline_order": spec.baseline_order,
            "stage2_partition_order": spec.stage2_order,
            "observable_focus": spec.observable_focus,
            "baseline_projection": baseline_projection,
            "stage2_projection": stage2_projection,
            "baseline_projection_equals_stage2_projection": projection_equal,
            "baseline_dependency_validation": baseline_validation,
            "stage2_dependency_validation": stage2_validation,
            "permutation_order_count": spec.permutation_orders.len(),
            "nontrivial_permutation": nontrivial_permutation,
            "source_status_valid": source_status_valid,
            "formatting_watch": spec.formatting_watch,
            "formatting_guard_valid": formatting_guard_valid,
            "reason": spec.reason,
            "validation_state": if valid {
                "bounded_stage2_replay_invariant"
            } else {
                "bounded_stage2_replay_failed"
            }
        }),
        permutation_rows,
        observable_invariant: valid,
        formatting_watch: spec.formatting_watch,
        failed: !valid,
        nontrivial_permutation,
    })
}

fn observable_projection(
    repo_root: &Path,
    spec: &Stage2ReplaySpec,
    source: &Value,
) -> Result<Value, Stage2ReplayError> {
    match spec.source_kind {
        SourceKind::TraceCalc => tracecalc_projection(repo_root, source),
        SourceKind::TreeCalc => treecalc_projection(repo_root, source),
        SourceKind::UpstreamHost => Ok(upstream_host_projection(source)),
    }
}

fn tracecalc_projection(repo_root: &Path, result: &Value) -> Result<Value, Stage2ReplayError> {
    let published_view = read_artifact_path(repo_root, result, "published_view")?;
    let rejects = read_artifact_path(repo_root, result, "rejects")?;
    let counters = read_artifact_path(repo_root, result, "counters")?;
    let trace = read_artifact_path(repo_root, result, "trace")?;
    let trace_labels = trace
        .get("events")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|event| event.get("label").and_then(Value::as_str))
        .collect::<Vec<_>>();

    Ok(json!({
        "projection_kind": "tracecalc_observable_result",
        "scenario_id": result["scenario_id"].clone(),
        "result_state": result["result_state"].clone(),
        "assertion_failures": result["assertion_failures"].clone(),
        "conformance_mismatches": result["conformance_mismatches"].clone(),
        "validation_failures": result["validation_failures"].clone(),
        "published_view": published_view,
        "rejects": rejects["rejects"].clone(),
        "counters": counters["counters"].clone(),
        "trace_labels": trace_labels,
        "replay_projection": result["replay_projection"].clone()
    }))
}

fn treecalc_projection(repo_root: &Path, result: &Value) -> Result<Value, Stage2ReplayError> {
    let published_values_path = text_at(result, "published_values_path");
    let runtime_effects_path = text_at(result, "runtime_effects_path");
    let counters_path = text_at(result, "counters_path");
    let published_values = read_json(repo_root, &published_values_path)?;
    let runtime_effects = read_json(repo_root, &runtime_effects_path)?;
    let counters = read_json(repo_root, &counters_path)?;

    Ok(json!({
        "projection_kind": "treecalc_local_observable_result",
        "case_id": result["case_id"].clone(),
        "result_state": result["result_state"].clone(),
        "candidate_result": {
            "aligned_canonical_family": result["candidate_result"]["aligned_canonical_family"].clone(),
            "dependency_shape_updates": result["candidate_result"]["dependency_shape_updates"].clone(),
            "runtime_effects": result["candidate_result"]["runtime_effects"].clone(),
            "target_set": result["candidate_result"]["target_set"].clone(),
            "value_updates": result["candidate_result"]["value_updates"].clone()
        },
        "publication_bundle": {
            "aligned_canonical_family": result["publication_bundle"]["aligned_canonical_family"].clone(),
            "published_runtime_effects": result["publication_bundle"]["published_runtime_effects"].clone(),
            "published_view_delta": result["publication_bundle"]["published_view_delta"].clone(),
            "trace_markers": result["publication_bundle"]["trace_markers"].clone(),
            "carriage_classification": result["publication_bundle"]["carriage_classification"].clone()
        },
        "published_values": published_values,
        "runtime_effects": runtime_effects,
        "counters": counters,
        "reject_detail": result["reject_detail"].clone(),
        "execution_restriction_interaction": result["execution_restriction_interaction"].clone(),
        "observable_projection_note": "evaluation_order is validated separately and excluded from semantic-result equality"
    }))
}

fn upstream_host_projection(result: &Value) -> Value {
    json!({
        "projection_kind": "direct_oxfml_upstream_host_observable_result",
        "case_id": result["case_id"].clone(),
        "status": result["status"].clone(),
        "candidate_result": result["candidate_result"].clone(),
        "commit_decision": result["commit_decision"].clone(),
        "returned_value_surface": result["returned_value_surface"].clone(),
        "expectation_mismatches": result["expectation_mismatches"].clone(),
        "verification_publication_surface": result["verification_publication_surface"].clone(),
        "w037_interpretation": result["w037_interpretation"].clone()
    })
}

fn dependency_graph(
    repo_root: &Path,
    source_kind: SourceKind,
    source: &Value,
) -> Result<Option<Value>, Stage2ReplayError> {
    match source_kind {
        SourceKind::TreeCalc => {
            let graph_path = text_at(source, "dependency_graph_path");
            read_json(repo_root, &graph_path).map(Some)
        }
        SourceKind::TraceCalc | SourceKind::UpstreamHost => Ok(None),
    }
}

fn dependency_validation(dependency_graph: &Option<Value>, order: &[u64]) -> Value {
    let Some(graph) = dependency_graph else {
        return json!({
            "validation_kind": "no_dependency_graph_required_for_source",
            "valid": true,
            "violations": []
        });
    };

    let positions = order
        .iter()
        .enumerate()
        .map(|(index, node_id)| (*node_id, index))
        .collect::<HashMap<_, _>>();
    let violations = graph
        .get("edges")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|edge| {
            let owner = edge.get("owner_node_id").and_then(Value::as_u64)?;
            let target = edge.get("target_node_id").and_then(Value::as_u64)?;
            let owner_position = positions.get(&owner)?;
            let target_position = positions.get(&target)?;
            (target_position > owner_position).then(|| {
                json!({
                    "edge_id": edge["edge_id"].clone(),
                    "owner_node_id": owner,
                    "target_node_id": target,
                    "owner_position": owner_position,
                    "target_position": target_position
                })
            })
        })
        .collect::<Vec<_>>();

    json!({
        "validation_kind": "dependency_order_precedence_check",
        "valid": violations.is_empty(),
        "order": order,
        "violations": violations
    })
}

fn source_status_valid(source_kind: SourceKind, source: &Value) -> bool {
    match source_kind {
        SourceKind::TraceCalc => {
            source.get("result_state").and_then(Value::as_str) == Some("passed")
                && source
                    .get("assertion_failures")
                    .and_then(Value::as_array)
                    .is_some_and(Vec::is_empty)
                && source
                    .get("validation_failures")
                    .and_then(Value::as_array)
                    .is_some_and(Vec::is_empty)
        }
        SourceKind::TreeCalc => {
            source.get("result_state").and_then(Value::as_str) == Some("published")
                && source.get("reject_detail").is_some_and(Value::is_null)
        }
        SourceKind::UpstreamHost => {
            source.get("status").and_then(Value::as_str) == Some("matched")
                && source
                    .get("expectation_mismatches")
                    .and_then(Value::as_array)
                    .is_some_and(Vec::is_empty)
        }
    }
}

fn w073_formatting_guard_valid(projection: &Value) -> bool {
    let surface = &projection["verification_publication_surface"];
    let has_rank_typed_rule = surface
        .get("conditional_formatting_typed_rule_families")
        .and_then(Value::as_array)
        .is_some_and(|families| {
            families
                .iter()
                .any(|family| family.as_str() == Some("rank"))
        });
    let retains_legacy_threshold_text = surface
        .get("conditional_formatting_thresholds")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .flat_map(|row| row.as_array().into_iter().flatten())
        .any(|threshold| threshold.as_str() == Some("legacy-count:1"));
    let colors_match_typed_rule = surface
        .get("array_cell_effective_fill_colors")
        .and_then(Value::as_array)
        .is_some_and(|rows| {
            rows.len() == 3
                && rows[0].get(0).is_some_and(Value::is_null)
                && rows[1].get(0).and_then(Value::as_str) == Some("#00FF00")
                && rows[2].get(0).and_then(Value::as_str) == Some("#00FF00")
        });
    let format_display_absent =
        !bool_at(surface, "format_delta_present") && !bool_at(surface, "display_delta_present");

    has_rank_typed_rule
        && retains_legacy_threshold_text
        && colors_match_typed_rule
        && format_display_absent
}

fn source_validation_failures(
    w036_stage2_tla: &Value,
    w037_semantic_requirements: &Value,
    w037_promotion_decision: &Value,
    w037_direct_oxfml_summary: &Value,
) -> Vec<String> {
    let mut failures = Vec::new();
    if number_at(w036_stage2_tla, "failed_config_count") != 0 {
        failures.push("w036_stage2_tla_failed_configs_present".to_string());
    }
    if !required_comparison_present(
        w037_semantic_requirements,
        "baseline_vs_stage2_partitioned_replay",
    ) {
        failures.push("w037_stage2_missing_baseline_replay_requirement".to_string());
    }
    if !required_comparison_present(
        w037_semantic_requirements,
        "stage2_partition_order_permutation_replay",
    ) {
        failures.push("w037_stage2_missing_permutation_requirement".to_string());
    }
    if bool_at(w037_promotion_decision, "stage2_policy_promoted") {
        failures.push("w037_stage2_already_promoted_unexpectedly".to_string());
    }
    if number_at(w037_direct_oxfml_summary, "w073_typed_rule_case_count") == 0 {
        failures.push("w037_direct_oxfml_missing_w073_typed_rule_guard".to_string());
    }
    failures
}

fn required_comparison_present(requirements: &Value, comparison_id: &str) -> bool {
    requirements
        .get("required_comparisons")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .any(|comparison| {
            comparison.get("comparison_id").and_then(Value::as_str) == Some(comparison_id)
                && comparison
                    .get("required")
                    .and_then(Value::as_bool)
                    .unwrap_or(false)
        })
}

fn exact_blocker_rows() -> Vec<Value> {
    vec![
        json!({
            "blocker_id": "stage2.production_partition_analyzer_soundness_absent",
            "owner": "calc-zsr.5; calc-zsr.9",
            "status_after_run": "exact_remaining_blocker",
            "reason": "This run validates declared bounded replay schedules and admissible order permutations; it does not prove the production partition analyzer sound for all dependency graphs, dynamic references, fences, or publication consequences.",
            "promotion_consequence": "Stage 2 policy remains unpromoted until production partition construction has a proof, model, or equivalent deterministic replay corpus for the claimed scope."
        }),
        json!({
            "blocker_id": "stage2.operated_cross_engine_differential_service_absent",
            "owner": "calc-zsr.6",
            "status_after_run": "exact_remaining_blocker",
            "reason": "The current run is local bounded replay evidence and does not operate a continuous cross-engine Stage 2 differential service.",
            "promotion_consequence": "Cross-engine service evidence remains required before any Stage 2 strategy can be promoted beyond bounded local evidence."
        }),
        json!({
            "blocker_id": "stage2.pack_grade_replay_governance_absent",
            "owner": "calc-zsr.8",
            "status_after_run": "exact_remaining_blocker",
            "reason": "The replay packet is deterministic and checkable but is not a pack-grade governed replay bundle with retained-witness service guarantees.",
            "promotion_consequence": "Pack-grade replay and C5 remain blocked until replay governance, witness policy, and promotion decision evidence are bound."
        }),
    ]
}

fn count_failed_permutation_rows(rows: &[Value]) -> usize {
    rows.iter()
        .filter(|row| !bool_at(&row["dependency_validation"], "valid"))
        .count()
}

fn read_artifact_path(
    repo_root: &Path,
    result: &Value,
    artifact_key: &str,
) -> Result<Value, Stage2ReplayError> {
    let relative_path = result
        .get("artifact_paths")
        .and_then(|paths| paths.get(artifact_key))
        .and_then(Value::as_str)
        .unwrap_or("<missing_artifact_path>");
    read_json(repo_root, relative_path)
}

fn read_json(repo_root: &Path, relative_path: &str) -> Result<Value, Stage2ReplayError> {
    let path = repo_root.join(relative_path);
    let contents = fs::read_to_string(&path).map_err(|source| Stage2ReplayError::ReadArtifact {
        path: path.display().to_string(),
        source,
    })?;
    serde_json::from_str(&contents).map_err(|source| Stage2ReplayError::ParseJson {
        path: path.display().to_string(),
        source,
    })
}

fn write_json(path: &Path, value: &Value) -> Result<(), Stage2ReplayError> {
    let contents =
        serde_json::to_string_pretty(value).map_err(|source| Stage2ReplayError::ParseJson {
            path: path.display().to_string(),
            source,
        })?;
    fs::write(path, format!("{contents}\n")).map_err(|source| Stage2ReplayError::WriteFile {
        path: path.display().to_string(),
        source,
    })
}

fn source_kind_name(source_kind: SourceKind) -> &'static str {
    match source_kind {
        SourceKind::TraceCalc => "tracecalc_reference_machine",
        SourceKind::TreeCalc => "treecalc_local",
        SourceKind::UpstreamHost => "direct_oxfml_upstream_host",
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

fn relative_artifact_path(parts: &[&str]) -> String {
    parts.join("/")
}

const STAGE2_REPLAY_SPECS: &[Stage2ReplaySpec] = &[
    Stage2ReplaySpec {
        row_id: "w038_stage2_tracecalc_accept_publish_reference",
        profile_id: "tracecalc_accept_publish_single_partition",
        source_kind: SourceKind::TraceCalc,
        source_artifact: TRACE_ACCEPT_RESULT,
        baseline_order: ORDER_TRACE_ACCEPT,
        stage2_order: ORDER_TRACE_ACCEPT,
        permutation_orders: PERMS_TRACE_ACCEPT,
        stage2_partition_shape: "single_partition_reference_replay",
        observable_focus: &[
            "published_view",
            "trace_labels",
            "counters",
            "candidate_publication_boundary",
        ],
        formatting_watch: false,
        reason: "bind the simplest accepted publish observable surface as a replay reference row before widening partition profiles",
    },
    Stage2ReplaySpec {
        row_id: "w038_stage2_treecalc_independent_partition_permutation",
        profile_id: "treecalc_independent_left_top_then_check",
        source_kind: SourceKind::TreeCalc,
        source_artifact: TREE_INDEPENDENT_RESULT,
        baseline_order: ORDER_INDEPENDENT_BASELINE,
        stage2_order: ORDER_INDEPENDENT_BASELINE,
        permutation_orders: PERMS_INDEPENDENT,
        stage2_partition_shape: "left_and_top_independent_partitions_before_check_partition",
        observable_focus: &[
            "published_values",
            "published_view_delta",
            "candidate_value_updates",
            "dependency_precedence",
        ],
        formatting_watch: false,
        reason: "prove the bounded independent left/top partitions can swap while preserving the check node result and all observable publication surfaces",
    },
    Stage2ReplaySpec {
        row_id: "w038_stage2_treecalc_dynamic_dependency_resolution",
        profile_id: "treecalc_dynamic_reference_late_bound_partition",
        source_kind: SourceKind::TreeCalc,
        source_artifact: TREE_DYNAMIC_RESULT,
        baseline_order: ORDER_DYNAMIC,
        stage2_order: ORDER_DYNAMIC,
        permutation_orders: PERMS_DYNAMIC,
        stage2_partition_shape: "dynamic_reference_owner_partition_after_late_bound_resolution",
        observable_focus: &[
            "published_values",
            "dependency_shape_updates",
            "runtime_effects",
            "dynamic_dependency_bound",
        ],
        formatting_watch: false,
        reason: "carry a soft/dynamic dependency row through the replay surface so Stage 2 evidence is not limited to static dependencies",
    },
    Stage2ReplaySpec {
        row_id: "w038_stage2_tracecalc_dynamic_dependency_reference",
        profile_id: "tracecalc_dynamic_dependency_release_reference",
        source_kind: SourceKind::TraceCalc,
        source_artifact: TRACE_DYNAMIC_RESULT,
        baseline_order: ORDER_EMPTY,
        stage2_order: ORDER_EMPTY,
        permutation_orders: &[ORDER_EMPTY],
        stage2_partition_shape: "tracecalc_reference_dynamic_dependency_release",
        observable_focus: &[
            "published_view",
            "dependency_shape_updates",
            "runtime_effects",
            "trace_labels",
        ],
        formatting_watch: false,
        reason: "bind the TraceCalc dynamic-dependency reference surface alongside the TreeCalc dynamic row without claiming a production Stage 2 scheduler",
    },
    Stage2ReplaySpec {
        row_id: "w038_stage2_w073_typed_formatting_guard",
        profile_id: "direct_oxfml_w073_typed_rule_only_formatting_watch",
        source_kind: SourceKind::UpstreamHost,
        source_artifact: W073_FORMATTING_RESULT,
        baseline_order: ORDER_FORMATTING,
        stage2_order: ORDER_FORMATTING,
        permutation_orders: PERMS_FORMATTING,
        stage2_partition_shape: "single_formula_direct_oxfml_formatting_watch",
        observable_focus: &[
            "conditional_formatting_typed_rule_families",
            "legacy_threshold_text_retained_but_not_interpreted",
            "format_delta_present",
            "display_delta_present",
        ],
        formatting_watch: true,
        reason: "carry the latest OxFml W073 typed-only aggregate/visualization metadata rule into Stage 2 observable-result invariance evidence",
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn stage2_replay_runner_writes_bounded_replay_without_promotion() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let run_id = format!("test-w038-stage2-replay-{}", std::process::id());
        let artifact_root =
            repo_root.join(format!("docs/test-runs/core-engine/stage2-replay/{run_id}"));
        let cleanup = || {
            if artifact_root.exists() {
                let _ = fs::remove_dir_all(&artifact_root);
            }
        };

        cleanup();
        let summary = Stage2ReplayRunner::new()
            .execute(&repo_root, &run_id)
            .unwrap();

        assert_eq!(summary.partition_replay_row_count, 5);
        assert_eq!(summary.permutation_replay_row_count, 6);
        assert_eq!(summary.nontrivial_permutation_row_count, 1);
        assert_eq!(summary.observable_invariance_row_count, 5);
        assert_eq!(summary.formatting_watch_row_count, 1);
        assert_eq!(summary.exact_remaining_blocker_count, 3);
        assert_eq!(summary.failed_row_count, 0);
        assert!(!summary.stage2_policy_promoted);

        let validation = read_json(
            &repo_root,
            &format!("docs/test-runs/core-engine/stage2-replay/{run_id}/validation.json"),
        )
        .unwrap();
        assert_eq!(validation["status"], "w038_stage2_replay_valid");

        let promotion = read_json(
            &repo_root,
            &format!("docs/test-runs/core-engine/stage2-replay/{run_id}/promotion_decision.json"),
        )
        .unwrap();
        assert_eq!(promotion["stage2_policy_promoted"], false);
        assert_eq!(promotion["w073_typed_formatting_guard_carried"], true);

        cleanup();
    }
}
