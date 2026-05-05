#![forbid(unsafe_code)]

//! W039 OxFml seam breadth, publication/display, and callable metadata packet emission.

use std::fs;
use std::path::Path;

use serde_json::{Value, json};
use thiserror::Error;

const RUN_SUMMARY_SCHEMA_V1: &str = "oxcalc.oxfml_seam.w039.run_summary.v1";
const SOURCE_INDEX_SCHEMA_V1: &str = "oxcalc.oxfml_seam.w039.source_evidence_index.v1";
const SURFACE_REGISTER_SCHEMA_V1: &str = "oxcalc.oxfml_seam.w039.surface_register.v1";
const PUBLICATION_DISPLAY_SCHEMA_V1: &str =
    "oxcalc.oxfml_seam.w039.publication_display_boundary_register.v1";
const CALLABLE_METADATA_SCHEMA_V1: &str = "oxcalc.oxfml_seam.w039.callable_metadata_register.v1";
const BLOCKER_REGISTER_SCHEMA_V1: &str = "oxcalc.oxfml_seam.w039.exact_blocker_register.v1";
const PROMOTION_DECISION_SCHEMA_V1: &str = "oxcalc.oxfml_seam.w039.promotion_decision.v1";
const VALIDATION_SCHEMA_V1: &str = "oxcalc.oxfml_seam.w039.validation.v1";

const W039_UPSTREAM_HOST_SUMMARY: &str = "docs/test-runs/core-engine/upstream-host/w039-oxfml-seam-breadth-callable-metadata-001/run_summary.json";
const W039_UPSTREAM_HOST_CASE_INDEX: &str = "docs/test-runs/core-engine/upstream-host/w039-oxfml-seam-breadth-callable-metadata-001/case_index.json";
const W039_W073_RESULT: &str = "docs/test-runs/core-engine/upstream-host/w039-oxfml-seam-breadth-callable-metadata-001/cases/uh_typed_cf_top_rank_guard_001/result.json";
const W039_LET_LEXICAL_RESULT: &str = "docs/test-runs/core-engine/upstream-host/w039-oxfml-seam-breadth-callable-metadata-001/cases/uh_let_lambda_lexical_capture_eval_001/result.json";
const W039_RETURNED_LAMBDA_RESULT: &str = "docs/test-runs/core-engine/upstream-host/w039-oxfml-seam-breadth-callable-metadata-001/cases/uh_returned_lambda_invocation_eval_001/result.json";
const W039_FORMATTING_INTAKE: &str = "docs/test-runs/core-engine/release-grade-ledger/w039-residual-successor-obligation-ledger-001/w073_formatting_intake.json";
const W039_IMPLEMENTATION_BLOCKERS: &str = "docs/test-runs/core-engine/implementation-conformance/w039-optimized-core-exact-blocker-disposition-001/w039_exact_remaining_blocker_register.json";
const W039_DIVERSITY_DECISION: &str = "docs/test-runs/core-engine/diversity-seam/w039-independent-evaluator-cross-engine-diversity-001/promotion_decision.json";
const OXFML_INBOUND_NOTES: &str = "../OxFml/docs/upstream/NOTES_FOR_OXCALC.md";

#[derive(Debug, Error)]
pub enum OxFmlSeamError {
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
pub struct OxFmlSeamRunSummary {
    pub run_id: String,
    pub schema_version: String,
    pub source_evidence_row_count: usize,
    pub surface_row_count: usize,
    pub publication_display_row_count: usize,
    pub callable_metadata_row_count: usize,
    pub exact_blocker_count: usize,
    pub failed_row_count: usize,
    pub oxfml_handoff_triggered: bool,
    pub callable_metadata_projection_promoted: bool,
    pub broad_oxfml_seam_promoted: bool,
    pub artifact_root: String,
}

#[derive(Debug, Clone, Default)]
pub struct OxFmlSeamRunner;

impl OxFmlSeamRunner {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    pub fn execute(
        &self,
        repo_root: &Path,
        run_id: &str,
    ) -> Result<OxFmlSeamRunSummary, OxFmlSeamError> {
        let relative_artifact_root =
            relative_artifact_path(&["docs", "test-runs", "core-engine", "oxfml-seam", run_id]);
        let artifact_root = repo_root.join(&relative_artifact_root);
        if artifact_root.exists() {
            fs::remove_dir_all(&artifact_root).map_err(|source| {
                OxFmlSeamError::RemoveDirectory {
                    path: artifact_root.display().to_string(),
                    source,
                }
            })?;
        }
        fs::create_dir_all(&artifact_root).map_err(|source| OxFmlSeamError::CreateDirectory {
            path: artifact_root.display().to_string(),
            source,
        })?;

        let upstream_summary = read_json(repo_root, W039_UPSTREAM_HOST_SUMMARY)?;
        let upstream_case_index = read_json(repo_root, W039_UPSTREAM_HOST_CASE_INDEX)?;
        let w073_result = read_json(repo_root, W039_W073_RESULT)?;
        let let_lexical_result = read_json(repo_root, W039_LET_LEXICAL_RESULT)?;
        let returned_lambda_result = read_json(repo_root, W039_RETURNED_LAMBDA_RESULT)?;
        let formatting_intake = read_json(repo_root, W039_FORMATTING_INTAKE)?;
        let implementation_blockers = read_json(repo_root, W039_IMPLEMENTATION_BLOCKERS)?;
        let diversity_decision = read_json(repo_root, W039_DIVERSITY_DECISION)?;
        let oxfml_notes = read_text(repo_root, OXFML_INBOUND_NOTES)?;

        let source_rows = source_rows(
            &upstream_summary,
            &upstream_case_index,
            &formatting_intake,
            &implementation_blockers,
            &diversity_decision,
            &oxfml_notes,
        );
        let surface_rows = surface_rows(&upstream_summary, &upstream_case_index, &oxfml_notes);
        let publication_display_rows =
            publication_display_rows(&w073_result, &formatting_intake, &oxfml_notes);
        let callable_rows = callable_metadata_rows(
            &let_lexical_result,
            &returned_lambda_result,
            &implementation_blockers,
            &oxfml_notes,
        );
        let blockers = exact_blockers(&implementation_blockers, &oxfml_notes);

        let mut validation_failures = source_validation_failures(&source_rows);
        validation_failures.extend(oxfml_seam_validation_failures(
            &surface_rows,
            &publication_display_rows,
            &callable_rows,
            &blockers,
        ));
        let failed_row_count = validation_failures.len();
        let oxfml_handoff_triggered = validation_failures
            .iter()
            .any(|failure| failure.contains("mismatch") || failure.contains("handoff"));

        let source_evidence_index_path =
            format!("{relative_artifact_root}/source_evidence_index.json");
        let surface_register_path =
            format!("{relative_artifact_root}/w039_oxfml_surface_register.json");
        let publication_display_path =
            format!("{relative_artifact_root}/w039_publication_display_boundary_register.json");
        let callable_metadata_path =
            format!("{relative_artifact_root}/w039_callable_metadata_register.json");
        let blocker_register_path =
            format!("{relative_artifact_root}/w039_exact_oxfml_seam_blocker_register.json");
        let promotion_decision_path = format!("{relative_artifact_root}/promotion_decision.json");
        let validation_path = format!("{relative_artifact_root}/validation.json");

        let source_evidence_index = json!({
            "schema_version": SOURCE_INDEX_SCHEMA_V1,
            "run_id": run_id,
            "artifact_root": relative_artifact_root,
            "source_evidence_row_count": source_rows.len(),
            "rows": source_rows,
            "source_artifacts": {
                "w039_upstream_host_summary": W039_UPSTREAM_HOST_SUMMARY,
                "w039_upstream_host_case_index": W039_UPSTREAM_HOST_CASE_INDEX,
                "w039_w073_result": W039_W073_RESULT,
                "w039_let_lexical_result": W039_LET_LEXICAL_RESULT,
                "w039_returned_lambda_result": W039_RETURNED_LAMBDA_RESULT,
                "w039_formatting_intake": W039_FORMATTING_INTAKE,
                "w039_implementation_blockers": W039_IMPLEMENTATION_BLOCKERS,
                "w039_diversity_decision": W039_DIVERSITY_DECISION,
                "oxfml_inbound_notes": OXFML_INBOUND_NOTES
            }
        });
        let surface_register = json!({
            "schema_version": SURFACE_REGISTER_SCHEMA_V1,
            "run_id": run_id,
            "row_count": surface_rows.len(),
            "broad_oxfml_seam_promoted": false,
            "rows": surface_rows
        });
        let publication_display_register = json!({
            "schema_version": PUBLICATION_DISPLAY_SCHEMA_V1,
            "run_id": run_id,
            "row_count": publication_display_rows.len(),
            "format_delta_display_delta_distinct": true,
            "broad_display_publication_promoted": false,
            "rows": publication_display_rows
        });
        let callable_metadata_register = json!({
            "schema_version": CALLABLE_METADATA_SCHEMA_V1,
            "run_id": run_id,
            "row_count": callable_rows.len(),
            "callable_metadata_projection_promoted": false,
            "general_oxfunc_kernel_promoted": false,
            "rows": callable_rows
        });
        let blocker_register = json!({
            "schema_version": BLOCKER_REGISTER_SCHEMA_V1,
            "run_id": run_id,
            "exact_blocker_count": blockers.len(),
            "rows": blockers
        });
        let promotion_decision = json!({
            "schema_version": PROMOTION_DECISION_SCHEMA_V1,
            "run_id": run_id,
            "decision_state": "w039_oxfml_seam_breadth_and_callable_metadata_bound_without_promotion",
            "broad_oxfml_seam_promoted": false,
            "broad_display_publication_promoted": false,
            "callable_metadata_projection_promoted": false,
            "general_oxfunc_kernel_promoted": false,
            "w073_formatting_handoff_triggered": false,
            "oxfml_handoff_triggered": oxfml_handoff_triggered,
            "pack_grade_replay_promoted": false,
            "c5_promoted": false,
            "stage2_policy_promoted": false,
            "source_evidence_row_count": source_rows.len(),
            "surface_row_count": surface_rows.len(),
            "publication_display_row_count": publication_display_rows.len(),
            "callable_metadata_row_count": callable_rows.len(),
            "exact_blocker_count": blockers.len(),
            "blockers": blockers
                .iter()
                .map(|row| row["blocker_id"].clone())
                .collect::<Vec<_>>(),
            "semantic_equivalence_statement": "This W039 OxFml seam runner binds direct upstream-host evidence, publication/display boundary rows, callable carrier rows, and exact seam blockers only. It does not change evaluator kernels, coordinator scheduling, recalc, publication, replay, pack, service, TraceCalc, TreeCalc, OxFml, OxFunc, Lean, or TLA semantics."
        });
        let validation = json!({
            "schema_version": VALIDATION_SCHEMA_V1,
            "run_id": run_id,
            "status": if validation_failures.is_empty() {
                "w039_oxfml_seam_breadth_callable_metadata_packet_valid"
            } else {
                "w039_oxfml_seam_breadth_callable_metadata_packet_invalid"
            },
            "source_evidence_row_count": source_rows.len(),
            "surface_row_count": surface_rows.len(),
            "publication_display_row_count": publication_display_rows.len(),
            "callable_metadata_row_count": callable_rows.len(),
            "exact_blocker_count": blockers.len(),
            "failed_row_count": failed_row_count,
            "oxfml_handoff_triggered": oxfml_handoff_triggered,
            "callable_metadata_projection_promoted": false,
            "broad_oxfml_seam_promoted": false,
            "validation_failures": validation_failures
        });
        let run_summary = json!({
            "schema_version": RUN_SUMMARY_SCHEMA_V1,
            "run_id": run_id,
            "artifact_root": relative_artifact_root,
            "source_evidence_index_path": source_evidence_index_path,
            "w039_oxfml_surface_register_path": surface_register_path,
            "w039_publication_display_boundary_register_path": publication_display_path,
            "w039_callable_metadata_register_path": callable_metadata_path,
            "w039_exact_oxfml_seam_blocker_register_path": blocker_register_path,
            "promotion_decision_path": promotion_decision_path,
            "validation_path": validation_path,
            "source_evidence_row_count": source_rows.len(),
            "surface_row_count": surface_rows.len(),
            "publication_display_row_count": publication_display_rows.len(),
            "callable_metadata_row_count": callable_rows.len(),
            "exact_blocker_count": blockers.len(),
            "failed_row_count": failed_row_count,
            "oxfml_handoff_triggered": oxfml_handoff_triggered,
            "callable_metadata_projection_promoted": false,
            "broad_oxfml_seam_promoted": false
        });

        write_json(
            &artifact_root.join("source_evidence_index.json"),
            &source_evidence_index,
        )?;
        write_json(
            &artifact_root.join("w039_oxfml_surface_register.json"),
            &surface_register,
        )?;
        write_json(
            &artifact_root.join("w039_publication_display_boundary_register.json"),
            &publication_display_register,
        )?;
        write_json(
            &artifact_root.join("w039_callable_metadata_register.json"),
            &callable_metadata_register,
        )?;
        write_json(
            &artifact_root.join("w039_exact_oxfml_seam_blocker_register.json"),
            &blocker_register,
        )?;
        write_json(
            &artifact_root.join("promotion_decision.json"),
            &promotion_decision,
        )?;
        write_json(&artifact_root.join("validation.json"), &validation)?;
        write_json(&artifact_root.join("run_summary.json"), &run_summary)?;

        Ok(OxFmlSeamRunSummary {
            run_id: run_id.to_string(),
            schema_version: RUN_SUMMARY_SCHEMA_V1.to_string(),
            source_evidence_row_count: source_rows.len(),
            surface_row_count: surface_rows.len(),
            publication_display_row_count: publication_display_rows.len(),
            callable_metadata_row_count: callable_rows.len(),
            exact_blocker_count: blockers.len(),
            failed_row_count,
            oxfml_handoff_triggered,
            callable_metadata_projection_promoted: false,
            broad_oxfml_seam_promoted: false,
            artifact_root: relative_artifact_root,
        })
    }
}

fn source_rows(
    upstream_summary: &Value,
    upstream_case_index: &Value,
    formatting_intake: &Value,
    implementation_blockers: &Value,
    diversity_decision: &Value,
    oxfml_notes: &str,
) -> Vec<Value> {
    vec![
        json!({
            "row_id": "source.w039_upstream_host_direct_oxfml",
            "artifact": W039_UPSTREAM_HOST_SUMMARY,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": number_at(upstream_summary, "expectation_mismatch_count"),
            "failed_row_count": 0,
            "promotion_guard": !bool_at(&upstream_summary["promotion_limits"], "general_oxfunc_kernel_claimed")
                && !bool_at(&upstream_summary["promotion_limits"], "pack_grade_replay_promoted")
                && !bool_at(&upstream_summary["promotion_limits"], "c5_promoted"),
            "semantic_state": "fresh_direct_oxfml_runtime_slice_bound"
        }),
        json!({
            "row_id": "source.w039_upstream_host_case_index",
            "artifact": W039_UPSTREAM_HOST_CASE_INDEX,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": 0,
            "promotion_guard": tag_count(upstream_case_index, "direct-oxfml") >= 3
                && tag_count(upstream_case_index, "let-lambda") >= 2
                && tag_count(upstream_case_index, "w073") >= 1,
            "semantic_state": "case_index_covers_direct_oxfml_let_lambda_and_w073"
        }),
        json!({
            "row_id": "source.w039_w073_formatting_intake",
            "artifact": W039_FORMATTING_INTAKE,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": 0,
            "promotion_guard": array_len_at(formatting_intake, "typed_only_families") == 7
                && text_at(formatting_intake, "thresholds_rule").contains("intentionally ignored"),
            "semantic_state": "w073_typed_only_formatting_intake_bound"
        }),
        json!({
            "row_id": "source.w039_callable_metadata_blocker",
            "artifact": W039_IMPLEMENTATION_BLOCKERS,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": 0,
            "promotion_guard": row_with_field_exists(
                implementation_blockers,
                "row_id",
                "w039_callable_metadata_projection_exact_blocker"
            ),
            "semantic_state": "callable_metadata_exact_blocker_present"
        }),
        json!({
            "row_id": "source.w039_diversity_no_broad_oxfml_promotion",
            "artifact": W039_DIVERSITY_DECISION,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": 0,
            "promotion_guard": !bool_at(diversity_decision, "broad_oxfml_seam_promoted")
                && !bool_at(diversity_decision, "callable_metadata_projection_promoted"),
            "semantic_state": "predecessor_diversity_packet_preserves_oxfml_no_promotion"
        }),
        json!({
            "row_id": "source.oxfml_inbound_notes",
            "artifact": OXFML_INBOUND_NOTES,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": 0,
            "promotion_guard": oxfml_notes.contains("consumer::runtime")
                && oxfml_notes.contains("format_delta")
                && oxfml_notes.contains("display_delta")
                && oxfml_notes.contains("RegisteredExternalDescriptor"),
            "semantic_state": "current_inbound_oxfml_notes_surface_present"
        }),
    ]
}

fn surface_rows(
    upstream_summary: &Value,
    upstream_case_index: &Value,
    oxfml_notes: &str,
) -> Vec<Value> {
    vec![
        json!({
            "row_id": "surface.direct_oxfml_runtime_facade",
            "disposition_kind": "exercised_current_surface",
            "source": W039_UPSTREAM_HOST_SUMMARY,
            "surface_state": "direct_runtime_facade_exercised",
            "direct_oxfml_case_count": number_at(upstream_summary, "direct_oxfml_case_count"),
            "expectation_mismatch_count": number_at(upstream_summary, "expectation_mismatch_count"),
            "promotion_consequence": "current direct OxFml runtime surface is bound for this fixture slice without broad seam closure"
        }),
        json!({
            "row_id": "surface.public_consumer_entry_points",
            "disposition_kind": "note_level_watch",
            "source": OXFML_INBOUND_NOTES,
            "surface_state": "current_public_consumer_surface_named",
            "consumer_runtime_present": oxfml_notes.contains("consumer::runtime"),
            "consumer_editor_present": oxfml_notes.contains("consumer::editor"),
            "consumer_replay_present": oxfml_notes.contains("consumer::replay"),
            "promotion_consequence": "OxCalc records the intended public consumer surface without claiming migration or API freeze"
        }),
        json!({
            "row_id": "surface.host_query_and_provider_families",
            "disposition_kind": "exercised_bounded_surface",
            "source": W039_UPSTREAM_HOST_CASE_INDEX,
            "surface_state": "host_info_rtd_reference_families_exercised",
            "host_info_case_count": tag_count(upstream_case_index, "host-info"),
            "rtd_case_count": tag_count(upstream_case_index, "rtd"),
            "bind_context_case_count": tag_count(upstream_case_index, "bind-context"),
            "promotion_consequence": "bounded host-query fixture coverage is present, not full provider-family closure"
        }),
        json!({
            "row_id": "surface.structured_reference_table_context",
            "disposition_kind": "exercised_bounded_surface",
            "source": W039_UPSTREAM_HOST_CASE_INDEX,
            "surface_state": "table_context_and_structured_reference_cases_exercised",
            "table_context_case_count": tag_count(upstream_case_index, "table-context"),
            "structured_reference_case_count": tag_count(upstream_case_index, "structured-reference"),
            "promotion_consequence": "table packet direction is exercised for the bounded fixture slice, not broad workbook table closure"
        }),
        json!({
            "row_id": "surface.registered_external_packet",
            "disposition_kind": "note_level_watch",
            "source": OXFML_INBOUND_NOTES,
            "surface_state": "registered_external_packet_converged_at_note_level",
            "descriptor_present": oxfml_notes.contains("RegisteredExternalDescriptor"),
            "mutation_request_present": oxfml_notes.contains("RegisteredExternalCatalogMutationRequest"),
            "promotion_consequence": "registered external packet naming remains note-level; callable metadata projection is not promoted"
        }),
    ]
}

fn publication_display_rows(
    w073_result: &Value,
    formatting_intake: &Value,
    oxfml_notes: &str,
) -> Vec<Value> {
    let publication_surface = &w073_result["verification_publication_surface"];
    let candidate_result = &w073_result["candidate_result"];
    vec![
        json!({
            "row_id": "publication.w073_typed_only_formatting_guard",
            "disposition_kind": "exercised_current_surface",
            "source": W039_W073_RESULT,
            "boundary_state": "typed_rule_only_guard_exercised",
            "typed_rule_family_count": array_len_at(publication_surface, "conditional_formatting_typed_rule_families"),
            "legacy_thresholds_present": array_len_at(publication_surface, "conditional_formatting_thresholds") > 0,
            "typed_only_family_count": array_len_at(formatting_intake, "typed_only_families"),
            "format_delta_present": bool_at(publication_surface, "format_delta_present"),
            "display_delta_present": bool_at(publication_surface, "display_delta_present"),
            "promotion_consequence": "W073 typed_rule-only behavior is carried as current consumed evidence without broad formatting closure"
        }),
        json!({
            "row_id": "publication.format_delta_display_delta_distinct",
            "disposition_kind": "accepted_boundary",
            "source": OXFML_INBOUND_NOTES,
            "boundary_state": "distinct_categories_retained",
            "format_delta_note_present": oxfml_notes.contains("format_delta"),
            "display_delta_note_present": oxfml_notes.contains("display_delta"),
            "w073_guard_format_delta_present": bool_at(publication_surface, "format_delta_present"),
            "w073_guard_display_delta_present": bool_at(publication_surface, "display_delta_present"),
            "promotion_consequence": "format_delta and display_delta remain distinct categories; this packet does not claim broad display-facing closure"
        }),
        json!({
            "row_id": "publication.candidate_commit_value_shape_surface",
            "disposition_kind": "exercised_current_surface",
            "source": W039_W073_RESULT,
            "boundary_state": "candidate_commit_and_shape_surface_exercised",
            "candidate_result_id": text_at(candidate_result, "candidate_result_id"),
            "published_value_class": text_at(candidate_result, "published_value_class"),
            "shape_delta_present": bool_at(candidate_result, "shape_delta_present"),
            "commit_kind": text_at(&w073_result["commit_decision"], "kind"),
            "promotion_consequence": "candidate/commit and value/shape publication surfaces are exercised for the W073 fixture only"
        }),
        json!({
            "row_id": "publication.broad_display_publication_breadth",
            "disposition_kind": "exact_remaining_blocker",
            "source": OXFML_INBOUND_NOTES,
            "boundary_state": "broad_display_publication_unpromoted",
            "reason": "current direct fixture coverage does not exercise broad display-facing categories, broad topology/publication consequences, or all future format/display families",
            "promotion_consequence": "broad OxFml display/publication closure remains unpromoted"
        }),
    ]
}

fn callable_metadata_rows(
    let_lexical_result: &Value,
    returned_lambda_result: &Value,
    implementation_blockers: &Value,
    oxfml_notes: &str,
) -> Vec<Value> {
    vec![
        json!({
            "row_id": "callable.let_lambda_lexical_capture",
            "disposition_kind": "exercised_current_surface",
            "source": W039_LET_LEXICAL_RESULT,
            "callable_state": "narrow_let_lambda_carrier_exercised",
            "function_id_count": array_len_at(&let_lexical_result["evaluation_trace"], "function_ids"),
            "prepared_call_count": number_at(&let_lexical_result["evaluation_trace"], "prepared_call_count"),
            "narrow_let_lambda_carrier": bool_at(&let_lexical_result["w037_interpretation"], "narrow_let_lambda_carrier"),
            "promotion_consequence": "lexical LET/LAMBDA carrier is exercised without general OxFunc kernel promotion"
        }),
        json!({
            "row_id": "callable.returned_lambda_invocation",
            "disposition_kind": "exercised_current_surface",
            "source": W039_RETURNED_LAMBDA_RESULT,
            "callable_state": "returned_lambda_value_carrier_exercised",
            "function_id_count": array_len_at(&returned_lambda_result["evaluation_trace"], "function_ids"),
            "prepared_call_count": number_at(&returned_lambda_result["evaluation_trace"], "prepared_call_count"),
            "payload_summary": text_at(&returned_lambda_result["returned_value_surface"], "payload_summary"),
            "promotion_consequence": "returned-lambda invocation is value-carrier evidence, not metadata projection evidence"
        }),
        json!({
            "row_id": "callable.callable_metadata_projection",
            "disposition_kind": "exact_remaining_blocker",
            "source": W039_IMPLEMENTATION_BLOCKERS,
            "callable_state": "metadata_projection_absent",
            "blocker_present": row_with_field_exists(
                implementation_blockers,
                "row_id",
                "w039_callable_metadata_projection_exact_blocker"
            ),
            "promotion_consequence": "callable metadata projection remains blocked until a projection fixture or carrier sufficiency proof exists"
        }),
        json!({
            "row_id": "callable.general_oxfunc_kernel_boundary",
            "disposition_kind": "accepted_external_boundary",
            "source": OXFML_INBOUND_NOTES,
            "callable_state": "general_oxfunc_kernel_external",
            "notes_name_oxfunc": oxfml_notes.contains("OxFunc"),
            "promotion_consequence": "OxCalc keeps only the narrow LET/LAMBDA carrier seam in this formalization scope"
        }),
        json!({
            "row_id": "callable.registered_external_callable_metadata",
            "disposition_kind": "note_level_watch",
            "source": OXFML_INBOUND_NOTES,
            "callable_state": "registered_external_metadata_not_current_projection",
            "registered_external_descriptor_present": oxfml_notes.contains("RegisteredExternalDescriptor"),
            "promotion_consequence": "registered-external packet alignment does not close callable metadata projection in OxCalc"
        }),
    ]
}

fn exact_blockers(implementation_blockers: &Value, oxfml_notes: &str) -> Vec<Value> {
    vec![
        json!({
            "blocker_id": "w039_oxfml.callable_metadata_projection_absent",
            "owner": "calc-f7o.7",
            "status_after_run": "exact_remaining_blocker",
            "evidence": W039_IMPLEMENTATION_BLOCKERS,
            "blocker_present": row_with_field_exists(
                implementation_blockers,
                "row_id",
                "w039_callable_metadata_projection_exact_blocker"
            ),
            "reason": "direct LET/LAMBDA value-carrier evidence exists, but callable metadata projection fixture or carrier sufficiency proof is absent.",
            "promotion_consequence": "callable metadata projection remains unpromoted"
        }),
        json!({
            "blocker_id": "w039_oxfml.broad_display_publication_breadth_unexercised",
            "owner": "calc-f7o.7",
            "status_after_run": "exact_remaining_blocker",
            "evidence": OXFML_INBOUND_NOTES,
            "format_display_notes_present": oxfml_notes.contains("format_delta") && oxfml_notes.contains("display_delta"),
            "reason": "the current direct slice carries typed formatting and distinct format/display categories, but not broad display-facing or publication/topology closure.",
            "promotion_consequence": "broad OxFml display/publication closure remains unpromoted"
        }),
        json!({
            "blocker_id": "w039_oxfml.public_consumer_surface_migration_not_verified",
            "owner": "calc-f7o.7; future implementation lane",
            "status_after_run": "exact_remaining_watch_blocker",
            "evidence": OXFML_INBOUND_NOTES,
            "consumer_surface_present": oxfml_notes.contains("consumer::runtime")
                && oxfml_notes.contains("consumer::editor")
                && oxfml_notes.contains("consumer::replay"),
            "reason": "OxFml names the public consumer surface, but this bead does not migrate or verify all OxCalc integration call sites against that surface.",
            "promotion_consequence": "public surface alignment remains watch-bound rather than promoted"
        }),
        json!({
            "blocker_id": "w039_oxfml.registered_external_callable_projection_deferred",
            "owner": "calc-f7o.7; external:OxFml/OxFunc",
            "status_after_run": "exact_remaining_watch_blocker",
            "evidence": OXFML_INBOUND_NOTES,
            "registered_external_descriptor_present": oxfml_notes.contains("RegisteredExternalDescriptor"),
            "reason": "registered-external packet naming is note-level converged, but no OxCalc callable metadata projection fixture is exercised.",
            "promotion_consequence": "registered external callable metadata projection remains unpromoted"
        }),
        json!({
            "blocker_id": "w039_oxfml.general_oxfunc_kernel_external",
            "owner": "external:OxFunc",
            "status_after_run": "accepted_external_boundary",
            "evidence": OXFML_INBOUND_NOTES,
            "reason": "W039 includes the narrow LET/LAMBDA carrier seam, not general OxFunc kernel formalization inside OxCalc.",
            "promotion_consequence": "general OxFunc kernel promotion remains outside OxCalc"
        }),
    ]
}

fn source_validation_failures(source_rows: &[Value]) -> Vec<String> {
    source_rows
        .iter()
        .flat_map(|row| {
            let row_id = text_at(row, "row_id");
            let mut failures = Vec::new();
            if number_at(row, "missing_artifact_count") > 0 {
                failures.push(format!("{row_id}.missing_artifact_count_nonzero"));
            }
            if number_at(row, "unexpected_mismatch_count") > 0 {
                failures.push(format!("{row_id}.unexpected_mismatch_count_nonzero"));
            }
            if number_at(row, "failed_row_count") > 0 {
                failures.push(format!("{row_id}.failed_row_count_nonzero"));
            }
            if !bool_at(row, "promotion_guard") {
                failures.push(format!("{row_id}.promotion_guard_failed"));
            }
            failures
        })
        .collect()
}

fn oxfml_seam_validation_failures(
    surface_rows: &[Value],
    publication_display_rows: &[Value],
    callable_rows: &[Value],
    blockers: &[Value],
) -> Vec<String> {
    let mut failures = Vec::new();
    if !surface_rows.iter().any(|row| {
        row.get("row_id").and_then(Value::as_str) == Some("surface.public_consumer_entry_points")
    }) {
        failures.push("w039_oxfml.public_consumer_surface_row_missing".to_string());
    }
    if !publication_display_rows.iter().any(|row| {
        row.get("row_id").and_then(Value::as_str)
            == Some("publication.format_delta_display_delta_distinct")
    }) {
        failures.push("w039_oxfml.format_display_boundary_row_missing".to_string());
    }
    if !callable_rows.iter().any(|row| {
        row.get("row_id").and_then(Value::as_str) == Some("callable.callable_metadata_projection")
            && row.get("disposition_kind").and_then(Value::as_str)
                == Some("exact_remaining_blocker")
    }) {
        failures.push("w039_oxfml.callable_metadata_blocker_row_missing".to_string());
    }
    if blockers.len() < 5 {
        failures.push("w039_oxfml.exact_blocker_count_below_gate".to_string());
    }
    if surface_rows
        .iter()
        .chain(publication_display_rows.iter())
        .chain(callable_rows.iter())
        .any(|row| row.get("disposition_kind").and_then(Value::as_str) == Some("promoted"))
    {
        failures.push("w039_oxfml.unexpected_promotion_row".to_string());
    }
    failures
}

fn tag_count(case_index: &Value, expected_tag: &str) -> usize {
    case_index
        .get("cases")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|case| {
            case.get("tags")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .any(|tag| tag.as_str() == Some(expected_tag))
        })
        .count()
}

fn row_with_field_exists(value: &Value, field: &str, expected: &str) -> bool {
    value
        .get("rows")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .any(|row| row.get(field).and_then(Value::as_str) == Some(expected))
}

fn read_json(repo_root: &Path, relative_path: &str) -> Result<Value, OxFmlSeamError> {
    let path = repo_root.join(relative_path);
    let contents = fs::read_to_string(&path).map_err(|source| OxFmlSeamError::ReadArtifact {
        path: path.display().to_string(),
        source,
    })?;
    serde_json::from_str(&contents).map_err(|source| OxFmlSeamError::ParseJson {
        path: path.display().to_string(),
        source,
    })
}

fn read_text(repo_root: &Path, relative_path: &str) -> Result<String, OxFmlSeamError> {
    let path = repo_root.join(relative_path);
    fs::read_to_string(&path).map_err(|source| OxFmlSeamError::ReadArtifact {
        path: path.display().to_string(),
        source,
    })
}

fn write_json(path: &Path, value: &Value) -> Result<(), OxFmlSeamError> {
    let contents =
        serde_json::to_string_pretty(value).map_err(|source| OxFmlSeamError::ParseJson {
            path: path.display().to_string(),
            source,
        })?;
    fs::write(path, format!("{contents}\n")).map_err(|source| OxFmlSeamError::WriteFile {
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

fn array_len_at(value: &Value, key: &str) -> usize {
    value.get(key).and_then(Value::as_array).map_or(0, Vec::len)
}

fn bool_at(value: &Value, key: &str) -> bool {
    value.get(key).and_then(Value::as_bool).unwrap_or(false)
}

fn relative_artifact_path(parts: &[&str]) -> String {
    parts.join("/")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn oxfml_seam_runner_classifies_w039_surfaces_without_promotion() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let run_id = format!("test-w039-oxfml-seam-{}", std::process::id());
        let artifact_root =
            repo_root.join(format!("docs/test-runs/core-engine/oxfml-seam/{run_id}"));
        let cleanup = || {
            if artifact_root.exists() {
                let _ = fs::remove_dir_all(&artifact_root);
            }
        };

        cleanup();
        let summary = OxFmlSeamRunner::new().execute(&repo_root, &run_id).unwrap();

        assert_eq!(summary.schema_version, RUN_SUMMARY_SCHEMA_V1);
        assert_eq!(summary.source_evidence_row_count, 6);
        assert_eq!(summary.surface_row_count, 5);
        assert_eq!(summary.publication_display_row_count, 4);
        assert_eq!(summary.callable_metadata_row_count, 5);
        assert_eq!(summary.exact_blocker_count, 5);
        assert_eq!(summary.failed_row_count, 0);
        assert!(!summary.oxfml_handoff_triggered);
        assert!(!summary.callable_metadata_projection_promoted);
        assert!(!summary.broad_oxfml_seam_promoted);

        let validation = read_json(
            &repo_root,
            &format!("docs/test-runs/core-engine/oxfml-seam/{run_id}/validation.json"),
        )
        .unwrap();
        assert_eq!(
            validation["status"],
            "w039_oxfml_seam_breadth_callable_metadata_packet_valid"
        );

        let decision = read_json(
            &repo_root,
            &format!("docs/test-runs/core-engine/oxfml-seam/{run_id}/promotion_decision.json"),
        )
        .unwrap();
        assert_eq!(decision["callable_metadata_projection_promoted"], false);
        assert_eq!(decision["broad_oxfml_seam_promoted"], false);
        assert_eq!(decision["general_oxfunc_kernel_promoted"], false);

        cleanup();
    }
}
