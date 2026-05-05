#![forbid(unsafe_code)]

//! W039/W040/W041/W042 OxFml seam breadth, publication/display, and callable metadata packet emission.

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

const W040_RUN_SUMMARY_SCHEMA_V1: &str = "oxcalc.oxfml_seam.w040.run_summary.v1";
const W040_SOURCE_INDEX_SCHEMA_V1: &str = "oxcalc.oxfml_seam.w040.source_evidence_index.v1";
const W040_CONSUMED_SURFACE_SCHEMA_V1: &str = "oxcalc.oxfml_seam.w040.consumed_surface_register.v1";
const W040_PUBLICATION_DISPLAY_SCHEMA_V1: &str =
    "oxcalc.oxfml_seam.w040.publication_display_boundary_register.v1";
const W040_CALLABLE_METADATA_SCHEMA_V1: &str =
    "oxcalc.oxfml_seam.w040.callable_metadata_implementation_register.v1";
const W040_BLOCKER_REGISTER_SCHEMA_V1: &str = "oxcalc.oxfml_seam.w040.exact_blocker_register.v1";
const W040_PROMOTION_DECISION_SCHEMA_V1: &str = "oxcalc.oxfml_seam.w040.promotion_decision.v1";
const W040_VALIDATION_SCHEMA_V1: &str = "oxcalc.oxfml_seam.w040.validation.v1";
const W041_RUN_SUMMARY_SCHEMA_V1: &str = "oxcalc.oxfml_seam.w041.run_summary.v1";
const W041_SOURCE_INDEX_SCHEMA_V1: &str = "oxcalc.oxfml_seam.w041.source_evidence_index.v1";
const W041_CONSUMED_SURFACE_SCHEMA_V1: &str = "oxcalc.oxfml_seam.w041.consumed_surface_register.v1";
const W041_PUBLICATION_DISPLAY_SCHEMA_V1: &str =
    "oxcalc.oxfml_seam.w041.publication_display_boundary_register.v1";
const W041_CALLABLE_METADATA_SCHEMA_V1: &str =
    "oxcalc.oxfml_seam.w041.callable_carrier_and_metadata_register.v1";
const W041_REGISTERED_EXTERNAL_SCHEMA_V1: &str =
    "oxcalc.oxfml_seam.w041.registered_external_provider_register.v1";
const W041_BLOCKER_REGISTER_SCHEMA_V1: &str = "oxcalc.oxfml_seam.w041.exact_blocker_register.v1";
const W041_PROMOTION_DECISION_SCHEMA_V1: &str = "oxcalc.oxfml_seam.w041.promotion_decision.v1";
const W041_VALIDATION_SCHEMA_V1: &str = "oxcalc.oxfml_seam.w041.validation.v1";
const W042_RUN_SUMMARY_SCHEMA_V1: &str = "oxcalc.oxfml_seam.w042.run_summary.v1";
const W042_SOURCE_INDEX_SCHEMA_V1: &str = "oxcalc.oxfml_seam.w042.source_evidence_index.v1";
const W042_CONSUMED_SURFACE_SCHEMA_V1: &str = "oxcalc.oxfml_seam.w042.consumed_surface_register.v1";
const W042_PUBLICATION_DISPLAY_SCHEMA_V1: &str =
    "oxcalc.oxfml_seam.w042.publication_display_boundary_register.v1";
const W042_CALLABLE_METADATA_SCHEMA_V1: &str =
    "oxcalc.oxfml_seam.w042.callable_carrier_and_metadata_register.v1";
const W042_REGISTERED_EXTERNAL_SCHEMA_V1: &str =
    "oxcalc.oxfml_seam.w042.registered_external_provider_register.v1";
const W042_BLOCKER_REGISTER_SCHEMA_V1: &str = "oxcalc.oxfml_seam.w042.exact_blocker_register.v1";
const W042_PROMOTION_DECISION_SCHEMA_V1: &str = "oxcalc.oxfml_seam.w042.promotion_decision.v1";
const W042_VALIDATION_SCHEMA_V1: &str = "oxcalc.oxfml_seam.w042.validation.v1";

const W039_UPSTREAM_HOST_SUMMARY: &str = "docs/test-runs/core-engine/upstream-host/w039-oxfml-seam-breadth-callable-metadata-001/run_summary.json";
const W039_UPSTREAM_HOST_CASE_INDEX: &str = "docs/test-runs/core-engine/upstream-host/w039-oxfml-seam-breadth-callable-metadata-001/case_index.json";
const W039_W073_RESULT: &str = "docs/test-runs/core-engine/upstream-host/w039-oxfml-seam-breadth-callable-metadata-001/cases/uh_typed_cf_top_rank_guard_001/result.json";
const W039_LET_LEXICAL_RESULT: &str = "docs/test-runs/core-engine/upstream-host/w039-oxfml-seam-breadth-callable-metadata-001/cases/uh_let_lambda_lexical_capture_eval_001/result.json";
const W039_RETURNED_LAMBDA_RESULT: &str = "docs/test-runs/core-engine/upstream-host/w039-oxfml-seam-breadth-callable-metadata-001/cases/uh_returned_lambda_invocation_eval_001/result.json";
const W039_FORMATTING_INTAKE: &str = "docs/test-runs/core-engine/release-grade-ledger/w039-residual-successor-obligation-ledger-001/w073_formatting_intake.json";
const W039_IMPLEMENTATION_BLOCKERS: &str = "docs/test-runs/core-engine/implementation-conformance/w039-optimized-core-exact-blocker-disposition-001/w039_exact_remaining_blocker_register.json";
const W039_DIVERSITY_DECISION: &str = "docs/test-runs/core-engine/diversity-seam/w039-independent-evaluator-cross-engine-diversity-001/promotion_decision.json";

const W040_UPSTREAM_HOST_SUMMARY: &str = "docs/test-runs/core-engine/upstream-host/w040-oxfml-seam-breadth-callable-metadata-001/run_summary.json";
const W040_UPSTREAM_HOST_CASE_INDEX: &str = "docs/test-runs/core-engine/upstream-host/w040-oxfml-seam-breadth-callable-metadata-001/case_index.json";
const W040_W073_RESULT: &str = "docs/test-runs/core-engine/upstream-host/w040-oxfml-seam-breadth-callable-metadata-001/cases/uh_typed_cf_top_rank_guard_001/result.json";
const W040_LET_LEXICAL_RESULT: &str = "docs/test-runs/core-engine/upstream-host/w040-oxfml-seam-breadth-callable-metadata-001/cases/uh_let_lambda_lexical_capture_eval_001/result.json";
const W040_RETURNED_LAMBDA_RESULT: &str = "docs/test-runs/core-engine/upstream-host/w040-oxfml-seam-breadth-callable-metadata-001/cases/uh_returned_lambda_invocation_eval_001/result.json";
const W040_FORMATTING_INTAKE: &str = "docs/test-runs/core-engine/release-grade-ledger/w040-direct-verification-obligation-map-001/w073_formatting_intake.json";
const W040_OBLIGATION_MAP: &str = "docs/test-runs/core-engine/release-grade-ledger/w040-direct-verification-obligation-map-001/direct_verification_obligation_map.json";
const W040_IMPLEMENTATION_BLOCKERS: &str = "docs/test-runs/core-engine/implementation-conformance/w040-optimized-core-exact-blocker-fixes-differentials-001/w040_exact_remaining_blocker_register.json";
const W040_DIVERSITY_DECISION: &str = "docs/test-runs/core-engine/diversity-seam/w040-independent-evaluator-operated-differential-001/promotion_decision.json";
const W040_OXFML_SEAM_DECISION: &str = "docs/test-runs/core-engine/oxfml-seam/w040-oxfml-seam-breadth-callable-metadata-001/promotion_decision.json";
const W041_UPSTREAM_HOST_SUMMARY: &str = "docs/test-runs/core-engine/upstream-host/w041-oxfml-broad-display-callable-carrier-001/run_summary.json";
const W041_UPSTREAM_HOST_CASE_INDEX: &str = "docs/test-runs/core-engine/upstream-host/w041-oxfml-broad-display-callable-carrier-001/case_index.json";
const W041_W073_RESULT: &str = "docs/test-runs/core-engine/upstream-host/w041-oxfml-broad-display-callable-carrier-001/cases/uh_typed_cf_top_rank_guard_001/result.json";
const W041_LET_LEXICAL_RESULT: &str = "docs/test-runs/core-engine/upstream-host/w041-oxfml-broad-display-callable-carrier-001/cases/uh_let_lambda_lexical_capture_eval_001/result.json";
const W041_RETURNED_LAMBDA_RESULT: &str = "docs/test-runs/core-engine/upstream-host/w041-oxfml-broad-display-callable-carrier-001/cases/uh_returned_lambda_invocation_eval_001/result.json";
const W041_OBLIGATION_MAP: &str = "docs/test-runs/core-engine/release-grade-ledger/w041-residual-release-grade-successor-obligation-map-001/successor_obligation_map.json";
const W041_IMPLEMENTATION_BLOCKERS: &str = "docs/test-runs/core-engine/implementation-conformance/w041-optimized-core-residual-blocker-differentials-001/w041_exact_remaining_blocker_register.json";
const W041_DIVERSITY_DECISION: &str = "docs/test-runs/core-engine/diversity-seam/w041-independent-evaluator-breadth-operated-differential-001/promotion_decision.json";
const W042_UPSTREAM_HOST_SUMMARY: &str = "docs/test-runs/core-engine/upstream-host/w042-oxfml-public-migration-callable-carrier-registered-external-001/run_summary.json";
const W042_UPSTREAM_HOST_CASE_INDEX: &str = "docs/test-runs/core-engine/upstream-host/w042-oxfml-public-migration-callable-carrier-registered-external-001/case_index.json";
const W042_W073_RESULT: &str = "docs/test-runs/core-engine/upstream-host/w042-oxfml-public-migration-callable-carrier-registered-external-001/cases/uh_typed_cf_top_rank_guard_001/result.json";
const W042_LET_LEXICAL_RESULT: &str = "docs/test-runs/core-engine/upstream-host/w042-oxfml-public-migration-callable-carrier-registered-external-001/cases/uh_let_lambda_lexical_capture_eval_001/result.json";
const W042_RETURNED_LAMBDA_RESULT: &str = "docs/test-runs/core-engine/upstream-host/w042-oxfml-public-migration-callable-carrier-registered-external-001/cases/uh_returned_lambda_invocation_eval_001/result.json";
const W042_FORMATTING_INTAKE: &str = "docs/test-runs/core-engine/release-grade-ledger/w042-residual-release-grade-closure-obligation-ledger-001/w073_formatting_intake.json";
const W042_OBLIGATION_MAP: &str = "docs/test-runs/core-engine/release-grade-ledger/w042-residual-release-grade-closure-obligation-ledger-001/closure_obligation_map.json";
const W042_IMPLEMENTATION_SUMMARY: &str = "docs/test-runs/core-engine/implementation-conformance/w042-optimized-core-counterpart-conformance-callable-metadata-001/run_summary.json";
const W042_CALLABLE_METADATA_REGISTER: &str = "docs/test-runs/core-engine/implementation-conformance/w042-optimized-core-counterpart-conformance-callable-metadata-001/w042_callable_metadata_projection_register.json";
const W042_DIVERSITY_DECISION: &str = "docs/test-runs/core-engine/diversity-seam/w042-independent-evaluator-breadth-mismatch-quarantine-operated-differential-001/promotion_decision.json";
const W042_DIVERSITY_BLOCKERS: &str = "docs/test-runs/core-engine/diversity-seam/w042-independent-evaluator-breadth-mismatch-quarantine-operated-differential-001/w042_exact_diversity_blocker_register.json";
const W041_OXFML_SEAM_DECISION: &str = "docs/test-runs/core-engine/oxfml-seam/w041-oxfml-broad-display-publication-callable-carrier-001/promotion_decision.json";
const OXFML_W073_WORKSET: &str =
    "../OxFml/docs/worksets/W073_conditional_formatting_typed_visualization_payload.md";
const OXFML_W073_HANDOFF: &str =
    "../OxFml/docs/handoffs/HANDOFF-DNAONECALC-012_W073_TYPED_CF_PAYLOAD_FIRST_SLICE.md";
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
        if run_id.starts_with("w042-") || run_id.starts_with("test-w042-") {
            return self.execute_w042(repo_root, run_id);
        }
        if run_id.starts_with("w041-") || run_id.starts_with("test-w041-") {
            return self.execute_w041(repo_root, run_id);
        }
        if run_id.starts_with("w040-") || run_id.starts_with("test-w040-") {
            return self.execute_w040(repo_root, run_id);
        }

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

    fn execute_w042(
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

        let upstream_summary = read_json(repo_root, W042_UPSTREAM_HOST_SUMMARY)?;
        let upstream_case_index = read_json(repo_root, W042_UPSTREAM_HOST_CASE_INDEX)?;
        let w073_result = read_json(repo_root, W042_W073_RESULT)?;
        let let_lexical_result = read_json(repo_root, W042_LET_LEXICAL_RESULT)?;
        let returned_lambda_result = read_json(repo_root, W042_RETURNED_LAMBDA_RESULT)?;
        let formatting_intake = read_json(repo_root, W042_FORMATTING_INTAKE)?;
        let obligation_map = read_json(repo_root, W042_OBLIGATION_MAP)?;
        let implementation_summary = read_json(repo_root, W042_IMPLEMENTATION_SUMMARY)?;
        let callable_metadata_register = read_json(repo_root, W042_CALLABLE_METADATA_REGISTER)?;
        let predecessor_oxfml_decision = read_json(repo_root, W041_OXFML_SEAM_DECISION)?;
        let diversity_decision = read_json(repo_root, W042_DIVERSITY_DECISION)?;
        let diversity_blockers = read_json(repo_root, W042_DIVERSITY_BLOCKERS)?;
        let oxfml_notes = read_text(repo_root, OXFML_INBOUND_NOTES)?;
        let w073_workset = read_text(repo_root, OXFML_W073_WORKSET)?;
        let w073_handoff = read_text(repo_root, OXFML_W073_HANDOFF)?;

        let source_rows = source_rows_w042(
            &upstream_summary,
            &upstream_case_index,
            &formatting_intake,
            &obligation_map,
            &implementation_summary,
            &callable_metadata_register,
            &predecessor_oxfml_decision,
            &diversity_decision,
            &diversity_blockers,
            &oxfml_notes,
            &w073_workset,
            &w073_handoff,
        );
        let surface_rows =
            consumed_surface_rows_w042(&upstream_summary, &upstream_case_index, &oxfml_notes);
        let publication_display_rows = publication_display_rows_w042(
            &w073_result,
            &formatting_intake,
            &oxfml_notes,
            &w073_workset,
            &w073_handoff,
        );
        let callable_rows = callable_metadata_rows_w042(
            &let_lexical_result,
            &returned_lambda_result,
            &obligation_map,
            &callable_metadata_register,
            &oxfml_notes,
        );
        let registered_external_rows =
            registered_external_rows_w042(&upstream_case_index, &obligation_map, &oxfml_notes);
        let blockers = exact_blockers_w042(
            &obligation_map,
            &callable_metadata_register,
            &diversity_blockers,
            &oxfml_notes,
            &w073_handoff,
        );

        let mut validation_failures = source_validation_failures(&source_rows);
        validation_failures.extend(oxfml_seam_validation_failures_w042(
            &surface_rows,
            &publication_display_rows,
            &callable_rows,
            &registered_external_rows,
            &blockers,
        ));
        let failed_row_count = validation_failures.len();
        let oxfml_handoff_triggered = validation_failures
            .iter()
            .any(|failure| failure.contains("mismatch") || failure.contains("handoff"));

        let source_evidence_index_path =
            format!("{relative_artifact_root}/source_evidence_index.json");
        let surface_register_path =
            format!("{relative_artifact_root}/w042_oxfml_consumed_surface_register.json");
        let publication_display_path =
            format!("{relative_artifact_root}/w042_publication_display_boundary_register.json");
        let callable_metadata_path =
            format!("{relative_artifact_root}/w042_callable_carrier_and_metadata_register.json");
        let registered_external_path =
            format!("{relative_artifact_root}/w042_registered_external_provider_register.json");
        let blocker_register_path =
            format!("{relative_artifact_root}/w042_exact_oxfml_seam_blocker_register.json");
        let promotion_decision_path = format!("{relative_artifact_root}/promotion_decision.json");
        let validation_path = format!("{relative_artifact_root}/validation.json");

        let source_evidence_index = json!({
            "schema_version": W042_SOURCE_INDEX_SCHEMA_V1,
            "run_id": run_id,
            "artifact_root": relative_artifact_root,
            "source_evidence_row_count": source_rows.len(),
            "rows": source_rows,
            "source_artifacts": {
                "w042_upstream_host_summary": W042_UPSTREAM_HOST_SUMMARY,
                "w042_upstream_host_case_index": W042_UPSTREAM_HOST_CASE_INDEX,
                "w042_w073_result": W042_W073_RESULT,
                "w042_let_lexical_result": W042_LET_LEXICAL_RESULT,
                "w042_returned_lambda_result": W042_RETURNED_LAMBDA_RESULT,
                "w042_formatting_intake": W042_FORMATTING_INTAKE,
                "w042_obligation_map": W042_OBLIGATION_MAP,
                "w042_implementation_summary": W042_IMPLEMENTATION_SUMMARY,
                "w042_callable_metadata_register": W042_CALLABLE_METADATA_REGISTER,
                "w041_oxfml_seam_decision": W041_OXFML_SEAM_DECISION,
                "w042_diversity_decision": W042_DIVERSITY_DECISION,
                "w042_diversity_blockers": W042_DIVERSITY_BLOCKERS,
                "oxfml_inbound_notes": OXFML_INBOUND_NOTES,
                "oxfml_w073_workset": OXFML_W073_WORKSET,
                "oxfml_w073_handoff": OXFML_W073_HANDOFF
            }
        });
        let surface_register = json!({
            "schema_version": W042_CONSUMED_SURFACE_SCHEMA_V1,
            "run_id": run_id,
            "row_count": surface_rows.len(),
            "broad_oxfml_seam_promoted": false,
            "public_consumer_surface_migration_verified": false,
            "rows": surface_rows
        });
        let publication_display_register = json!({
            "schema_version": W042_PUBLICATION_DISPLAY_SCHEMA_V1,
            "run_id": run_id,
            "row_count": publication_display_rows.len(),
            "format_delta_display_delta_distinct": true,
            "w073_typed_only_formatting_guard_retained": true,
            "broad_display_publication_promoted": false,
            "provider_failure_callable_publication_promoted": false,
            "rows": publication_display_rows
        });
        let callable_metadata_register_json = json!({
            "schema_version": W042_CALLABLE_METADATA_SCHEMA_V1,
            "run_id": run_id,
            "row_count": callable_rows.len(),
            "callable_metadata_projection_promoted": false,
            "callable_carrier_sufficiency_proven": false,
            "general_oxfunc_kernel_promoted": false,
            "rows": callable_rows
        });
        let registered_external_register = json!({
            "schema_version": W042_REGISTERED_EXTERNAL_SCHEMA_V1,
            "run_id": run_id,
            "row_count": registered_external_rows.len(),
            "registered_external_callable_projection_promoted": false,
            "provider_failure_callable_publication_promoted": false,
            "rows": registered_external_rows
        });
        let blocker_register = json!({
            "schema_version": W042_BLOCKER_REGISTER_SCHEMA_V1,
            "run_id": run_id,
            "exact_blocker_count": blockers.len(),
            "rows": blockers
        });
        let promotion_decision = json!({
            "schema_version": W042_PROMOTION_DECISION_SCHEMA_V1,
            "run_id": run_id,
            "decision_state": "w042_oxfml_public_migration_callable_carrier_registered_external_bound_without_promotion",
            "broad_oxfml_seam_promoted": false,
            "broad_display_publication_promoted": false,
            "public_consumer_surface_migration_verified": false,
            "callable_metadata_projection_promoted": false,
            "callable_carrier_sufficiency_proven": false,
            "registered_external_callable_projection_promoted": false,
            "provider_failure_callable_publication_promoted": false,
            "general_oxfunc_kernel_promoted": false,
            "w073_typed_only_formatting_guard_retained": true,
            "w073_formatting_handoff_triggered": false,
            "oxfml_handoff_triggered": oxfml_handoff_triggered,
            "pack_grade_replay_promoted": false,
            "c5_promoted": false,
            "stage2_policy_promoted": false,
            "release_grade_verification_promoted": false,
            "source_evidence_row_count": source_rows.len(),
            "surface_row_count": surface_rows.len(),
            "publication_display_row_count": publication_display_rows.len(),
            "callable_metadata_row_count": callable_rows.len(),
            "registered_external_row_count": registered_external_rows.len(),
            "exact_blocker_count": blockers.len(),
            "blockers": blockers
                .iter()
                .map(|row| row["blocker_id"].clone())
                .collect::<Vec<_>>(),
            "semantic_equivalence_statement": "This W042 OxFml seam runner binds fresh upstream-host evidence, W042 closure obligations, current OxFml public consumer notes, W073 typed-only formatting guards, publication/display boundary rows, callable carrier rows, registered-external/provider watch rows, and exact blockers only. It does not change evaluator kernels, coordinator scheduling, recalc, publication, replay, pack, service, TraceCalc, TreeCalc, OxFml, OxFunc, Lean, or TLA semantics."
        });
        let validation = json!({
            "schema_version": W042_VALIDATION_SCHEMA_V1,
            "run_id": run_id,
            "status": if validation_failures.is_empty() {
                "w042_oxfml_public_migration_callable_carrier_packet_valid"
            } else {
                "w042_oxfml_public_migration_callable_carrier_packet_invalid"
            },
            "source_evidence_row_count": source_rows.len(),
            "surface_row_count": surface_rows.len(),
            "publication_display_row_count": publication_display_rows.len(),
            "callable_metadata_row_count": callable_rows.len(),
            "registered_external_row_count": registered_external_rows.len(),
            "exact_blocker_count": blockers.len(),
            "failed_row_count": failed_row_count,
            "oxfml_handoff_triggered": oxfml_handoff_triggered,
            "callable_metadata_projection_promoted": false,
            "broad_oxfml_seam_promoted": false,
            "validation_failures": validation_failures
        });
        let run_summary = json!({
            "schema_version": W042_RUN_SUMMARY_SCHEMA_V1,
            "run_id": run_id,
            "artifact_root": relative_artifact_root,
            "source_evidence_index_path": source_evidence_index_path,
            "w042_oxfml_consumed_surface_register_path": surface_register_path,
            "w042_publication_display_boundary_register_path": publication_display_path,
            "w042_callable_carrier_and_metadata_register_path": callable_metadata_path,
            "w042_registered_external_provider_register_path": registered_external_path,
            "w042_exact_oxfml_seam_blocker_register_path": blocker_register_path,
            "promotion_decision_path": promotion_decision_path,
            "validation_path": validation_path,
            "source_evidence_row_count": source_rows.len(),
            "surface_row_count": surface_rows.len(),
            "publication_display_row_count": publication_display_rows.len(),
            "callable_metadata_row_count": callable_rows.len(),
            "registered_external_row_count": registered_external_rows.len(),
            "exact_blocker_count": blockers.len(),
            "failed_row_count": failed_row_count,
            "oxfml_handoff_triggered": oxfml_handoff_triggered,
            "callable_metadata_projection_promoted": false,
            "broad_oxfml_seam_promoted": false,
            "w073_typed_only_formatting_guard_retained": true
        });

        write_json(
            &artifact_root.join("source_evidence_index.json"),
            &source_evidence_index,
        )?;
        write_json(
            &artifact_root.join("w042_oxfml_consumed_surface_register.json"),
            &surface_register,
        )?;
        write_json(
            &artifact_root.join("w042_publication_display_boundary_register.json"),
            &publication_display_register,
        )?;
        write_json(
            &artifact_root.join("w042_callable_carrier_and_metadata_register.json"),
            &callable_metadata_register_json,
        )?;
        write_json(
            &artifact_root.join("w042_registered_external_provider_register.json"),
            &registered_external_register,
        )?;
        write_json(
            &artifact_root.join("w042_exact_oxfml_seam_blocker_register.json"),
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
            schema_version: W042_RUN_SUMMARY_SCHEMA_V1.to_string(),
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

    fn execute_w041(
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

        let upstream_summary = read_json(repo_root, W041_UPSTREAM_HOST_SUMMARY)?;
        let upstream_case_index = read_json(repo_root, W041_UPSTREAM_HOST_CASE_INDEX)?;
        let w073_result = read_json(repo_root, W041_W073_RESULT)?;
        let let_lexical_result = read_json(repo_root, W041_LET_LEXICAL_RESULT)?;
        let returned_lambda_result = read_json(repo_root, W041_RETURNED_LAMBDA_RESULT)?;
        let formatting_intake = read_json(repo_root, W040_FORMATTING_INTAKE)?;
        let obligation_map = read_json(repo_root, W041_OBLIGATION_MAP)?;
        let implementation_blockers = read_json(repo_root, W041_IMPLEMENTATION_BLOCKERS)?;
        let predecessor_oxfml_decision = read_json(repo_root, W040_OXFML_SEAM_DECISION)?;
        let diversity_decision = read_json(repo_root, W041_DIVERSITY_DECISION)?;
        let oxfml_notes = read_text(repo_root, OXFML_INBOUND_NOTES)?;

        let source_rows = source_rows_w041(
            &upstream_summary,
            &upstream_case_index,
            &formatting_intake,
            &obligation_map,
            &implementation_blockers,
            &predecessor_oxfml_decision,
            &diversity_decision,
            &oxfml_notes,
        );
        let surface_rows =
            consumed_surface_rows_w041(&upstream_summary, &upstream_case_index, &oxfml_notes);
        let publication_display_rows =
            publication_display_rows_w041(&w073_result, &formatting_intake, &oxfml_notes);
        let callable_rows = callable_metadata_rows_w041(
            &let_lexical_result,
            &returned_lambda_result,
            &obligation_map,
            &implementation_blockers,
            &oxfml_notes,
        );
        let registered_external_rows =
            registered_external_rows_w041(&upstream_case_index, &obligation_map, &oxfml_notes);
        let blockers = exact_blockers_w041(&obligation_map, &implementation_blockers, &oxfml_notes);

        let mut validation_failures = source_validation_failures(&source_rows);
        validation_failures.extend(oxfml_seam_validation_failures_w041(
            &surface_rows,
            &publication_display_rows,
            &callable_rows,
            &registered_external_rows,
            &blockers,
        ));
        let failed_row_count = validation_failures.len();
        let oxfml_handoff_triggered = validation_failures
            .iter()
            .any(|failure| failure.contains("mismatch") || failure.contains("handoff"));

        let source_evidence_index_path =
            format!("{relative_artifact_root}/source_evidence_index.json");
        let surface_register_path =
            format!("{relative_artifact_root}/w041_oxfml_consumed_surface_register.json");
        let publication_display_path =
            format!("{relative_artifact_root}/w041_publication_display_boundary_register.json");
        let callable_metadata_path =
            format!("{relative_artifact_root}/w041_callable_carrier_and_metadata_register.json");
        let registered_external_path =
            format!("{relative_artifact_root}/w041_registered_external_provider_register.json");
        let blocker_register_path =
            format!("{relative_artifact_root}/w041_exact_oxfml_seam_blocker_register.json");
        let promotion_decision_path = format!("{relative_artifact_root}/promotion_decision.json");
        let validation_path = format!("{relative_artifact_root}/validation.json");

        let source_evidence_index = json!({
            "schema_version": W041_SOURCE_INDEX_SCHEMA_V1,
            "run_id": run_id,
            "artifact_root": relative_artifact_root,
            "source_evidence_row_count": source_rows.len(),
            "rows": source_rows,
            "source_artifacts": {
                "w041_upstream_host_summary": W041_UPSTREAM_HOST_SUMMARY,
                "w041_upstream_host_case_index": W041_UPSTREAM_HOST_CASE_INDEX,
                "w041_w073_result": W041_W073_RESULT,
                "w041_let_lexical_result": W041_LET_LEXICAL_RESULT,
                "w041_returned_lambda_result": W041_RETURNED_LAMBDA_RESULT,
                "w040_formatting_intake": W040_FORMATTING_INTAKE,
                "w041_obligation_map": W041_OBLIGATION_MAP,
                "w041_implementation_blockers": W041_IMPLEMENTATION_BLOCKERS,
                "w040_oxfml_seam_decision": W040_OXFML_SEAM_DECISION,
                "w041_diversity_decision": W041_DIVERSITY_DECISION,
                "oxfml_inbound_notes": OXFML_INBOUND_NOTES
            }
        });
        let surface_register = json!({
            "schema_version": W041_CONSUMED_SURFACE_SCHEMA_V1,
            "run_id": run_id,
            "row_count": surface_rows.len(),
            "broad_oxfml_seam_promoted": false,
            "public_consumer_surface_migration_verified": false,
            "rows": surface_rows
        });
        let publication_display_register = json!({
            "schema_version": W041_PUBLICATION_DISPLAY_SCHEMA_V1,
            "run_id": run_id,
            "row_count": publication_display_rows.len(),
            "format_delta_display_delta_distinct": true,
            "w073_typed_only_formatting_guard_retained": true,
            "broad_display_publication_promoted": false,
            "rows": publication_display_rows
        });
        let callable_metadata_register = json!({
            "schema_version": W041_CALLABLE_METADATA_SCHEMA_V1,
            "run_id": run_id,
            "row_count": callable_rows.len(),
            "callable_metadata_projection_promoted": false,
            "callable_carrier_sufficiency_proven": false,
            "general_oxfunc_kernel_promoted": false,
            "rows": callable_rows
        });
        let registered_external_register = json!({
            "schema_version": W041_REGISTERED_EXTERNAL_SCHEMA_V1,
            "run_id": run_id,
            "row_count": registered_external_rows.len(),
            "registered_external_callable_projection_promoted": false,
            "provider_failure_callable_publication_promoted": false,
            "rows": registered_external_rows
        });
        let blocker_register = json!({
            "schema_version": W041_BLOCKER_REGISTER_SCHEMA_V1,
            "run_id": run_id,
            "exact_blocker_count": blockers.len(),
            "rows": blockers
        });
        let promotion_decision = json!({
            "schema_version": W041_PROMOTION_DECISION_SCHEMA_V1,
            "run_id": run_id,
            "decision_state": "w041_oxfml_broad_publication_callable_carrier_bound_without_promotion",
            "broad_oxfml_seam_promoted": false,
            "broad_display_publication_promoted": false,
            "public_consumer_surface_migration_verified": false,
            "callable_metadata_projection_promoted": false,
            "callable_carrier_sufficiency_proven": false,
            "registered_external_callable_projection_promoted": false,
            "provider_failure_callable_publication_promoted": false,
            "general_oxfunc_kernel_promoted": false,
            "w073_typed_only_formatting_guard_retained": true,
            "w073_formatting_handoff_triggered": false,
            "oxfml_handoff_triggered": oxfml_handoff_triggered,
            "pack_grade_replay_promoted": false,
            "c5_promoted": false,
            "stage2_policy_promoted": false,
            "source_evidence_row_count": source_rows.len(),
            "surface_row_count": surface_rows.len(),
            "publication_display_row_count": publication_display_rows.len(),
            "callable_metadata_row_count": callable_rows.len(),
            "registered_external_row_count": registered_external_rows.len(),
            "exact_blocker_count": blockers.len(),
            "blockers": blockers
                .iter()
                .map(|row| row["blocker_id"].clone())
                .collect::<Vec<_>>(),
            "semantic_equivalence_statement": "This W041 OxFml seam runner binds fresh upstream-host evidence, W041 obligation rows, current OxFml public consumer notes, W073 typed-only formatting guards, publication/display boundary rows, callable carrier rows, registered-external/provider watch rows, and exact blockers only. It does not change evaluator kernels, coordinator scheduling, recalc, publication, replay, pack, service, TraceCalc, TreeCalc, OxFml, OxFunc, Lean, or TLA semantics."
        });
        let validation = json!({
            "schema_version": W041_VALIDATION_SCHEMA_V1,
            "run_id": run_id,
            "status": if validation_failures.is_empty() {
                "w041_oxfml_broad_publication_callable_carrier_packet_valid"
            } else {
                "w041_oxfml_broad_publication_callable_carrier_packet_invalid"
            },
            "source_evidence_row_count": source_rows.len(),
            "surface_row_count": surface_rows.len(),
            "publication_display_row_count": publication_display_rows.len(),
            "callable_metadata_row_count": callable_rows.len(),
            "registered_external_row_count": registered_external_rows.len(),
            "exact_blocker_count": blockers.len(),
            "failed_row_count": failed_row_count,
            "oxfml_handoff_triggered": oxfml_handoff_triggered,
            "callable_metadata_projection_promoted": false,
            "broad_oxfml_seam_promoted": false,
            "validation_failures": validation_failures
        });
        let run_summary = json!({
            "schema_version": W041_RUN_SUMMARY_SCHEMA_V1,
            "run_id": run_id,
            "artifact_root": relative_artifact_root,
            "source_evidence_index_path": source_evidence_index_path,
            "w041_oxfml_consumed_surface_register_path": surface_register_path,
            "w041_publication_display_boundary_register_path": publication_display_path,
            "w041_callable_carrier_and_metadata_register_path": callable_metadata_path,
            "w041_registered_external_provider_register_path": registered_external_path,
            "w041_exact_oxfml_seam_blocker_register_path": blocker_register_path,
            "promotion_decision_path": promotion_decision_path,
            "validation_path": validation_path,
            "source_evidence_row_count": source_rows.len(),
            "surface_row_count": surface_rows.len(),
            "publication_display_row_count": publication_display_rows.len(),
            "callable_metadata_row_count": callable_rows.len(),
            "registered_external_row_count": registered_external_rows.len(),
            "exact_blocker_count": blockers.len(),
            "failed_row_count": failed_row_count,
            "oxfml_handoff_triggered": oxfml_handoff_triggered,
            "callable_metadata_projection_promoted": false,
            "broad_oxfml_seam_promoted": false,
            "w073_typed_only_formatting_guard_retained": true
        });

        write_json(
            &artifact_root.join("source_evidence_index.json"),
            &source_evidence_index,
        )?;
        write_json(
            &artifact_root.join("w041_oxfml_consumed_surface_register.json"),
            &surface_register,
        )?;
        write_json(
            &artifact_root.join("w041_publication_display_boundary_register.json"),
            &publication_display_register,
        )?;
        write_json(
            &artifact_root.join("w041_callable_carrier_and_metadata_register.json"),
            &callable_metadata_register,
        )?;
        write_json(
            &artifact_root.join("w041_registered_external_provider_register.json"),
            &registered_external_register,
        )?;
        write_json(
            &artifact_root.join("w041_exact_oxfml_seam_blocker_register.json"),
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
            schema_version: W041_RUN_SUMMARY_SCHEMA_V1.to_string(),
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

    fn execute_w040(
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

        let upstream_summary = read_json(repo_root, W040_UPSTREAM_HOST_SUMMARY)?;
        let upstream_case_index = read_json(repo_root, W040_UPSTREAM_HOST_CASE_INDEX)?;
        let w073_result = read_json(repo_root, W040_W073_RESULT)?;
        let let_lexical_result = read_json(repo_root, W040_LET_LEXICAL_RESULT)?;
        let returned_lambda_result = read_json(repo_root, W040_RETURNED_LAMBDA_RESULT)?;
        let formatting_intake = read_json(repo_root, W040_FORMATTING_INTAKE)?;
        let obligation_map = read_json(repo_root, W040_OBLIGATION_MAP)?;
        let implementation_blockers = read_json(repo_root, W040_IMPLEMENTATION_BLOCKERS)?;
        let diversity_decision = read_json(repo_root, W040_DIVERSITY_DECISION)?;
        let oxfml_notes = read_text(repo_root, OXFML_INBOUND_NOTES)?;

        let source_rows = source_rows_w040(
            &upstream_summary,
            &upstream_case_index,
            &formatting_intake,
            &obligation_map,
            &implementation_blockers,
            &diversity_decision,
            &oxfml_notes,
        );
        let surface_rows =
            consumed_surface_rows_w040(&upstream_summary, &upstream_case_index, &oxfml_notes);
        let publication_display_rows =
            publication_display_rows_w040(&w073_result, &formatting_intake, &oxfml_notes);
        let callable_rows = callable_metadata_rows_w040(
            &let_lexical_result,
            &returned_lambda_result,
            &obligation_map,
            &implementation_blockers,
            &oxfml_notes,
        );
        let blockers = exact_blockers_w040(&obligation_map, &implementation_blockers, &oxfml_notes);

        let mut validation_failures = source_validation_failures(&source_rows);
        validation_failures.extend(oxfml_seam_validation_failures_w040(
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
            format!("{relative_artifact_root}/w040_oxfml_consumed_surface_register.json");
        let publication_display_path =
            format!("{relative_artifact_root}/w040_publication_display_boundary_register.json");
        let callable_metadata_path =
            format!("{relative_artifact_root}/w040_callable_metadata_implementation_register.json");
        let blocker_register_path =
            format!("{relative_artifact_root}/w040_exact_oxfml_seam_blocker_register.json");
        let promotion_decision_path = format!("{relative_artifact_root}/promotion_decision.json");
        let validation_path = format!("{relative_artifact_root}/validation.json");

        let source_evidence_index = json!({
            "schema_version": W040_SOURCE_INDEX_SCHEMA_V1,
            "run_id": run_id,
            "artifact_root": relative_artifact_root,
            "source_evidence_row_count": source_rows.len(),
            "rows": source_rows,
            "source_artifacts": {
                "w040_upstream_host_summary": W040_UPSTREAM_HOST_SUMMARY,
                "w040_upstream_host_case_index": W040_UPSTREAM_HOST_CASE_INDEX,
                "w040_w073_result": W040_W073_RESULT,
                "w040_let_lexical_result": W040_LET_LEXICAL_RESULT,
                "w040_returned_lambda_result": W040_RETURNED_LAMBDA_RESULT,
                "w040_formatting_intake": W040_FORMATTING_INTAKE,
                "w040_obligation_map": W040_OBLIGATION_MAP,
                "w040_implementation_blockers": W040_IMPLEMENTATION_BLOCKERS,
                "w040_diversity_decision": W040_DIVERSITY_DECISION,
                "oxfml_inbound_notes": OXFML_INBOUND_NOTES
            }
        });
        let surface_register = json!({
            "schema_version": W040_CONSUMED_SURFACE_SCHEMA_V1,
            "run_id": run_id,
            "row_count": surface_rows.len(),
            "broad_oxfml_seam_promoted": false,
            "public_consumer_surface_migration_verified": false,
            "rows": surface_rows
        });
        let publication_display_register = json!({
            "schema_version": W040_PUBLICATION_DISPLAY_SCHEMA_V1,
            "run_id": run_id,
            "row_count": publication_display_rows.len(),
            "format_delta_display_delta_distinct": true,
            "w073_typed_only_formatting_guard_retained": true,
            "broad_display_publication_promoted": false,
            "rows": publication_display_rows
        });
        let callable_metadata_register = json!({
            "schema_version": W040_CALLABLE_METADATA_SCHEMA_V1,
            "run_id": run_id,
            "row_count": callable_rows.len(),
            "callable_metadata_projection_promoted": false,
            "callable_carrier_sufficiency_proven": false,
            "general_oxfunc_kernel_promoted": false,
            "rows": callable_rows
        });
        let blocker_register = json!({
            "schema_version": W040_BLOCKER_REGISTER_SCHEMA_V1,
            "run_id": run_id,
            "exact_blocker_count": blockers.len(),
            "rows": blockers
        });
        let promotion_decision = json!({
            "schema_version": W040_PROMOTION_DECISION_SCHEMA_V1,
            "run_id": run_id,
            "decision_state": "w040_oxfml_seam_breadth_callable_metadata_bound_without_promotion",
            "broad_oxfml_seam_promoted": false,
            "broad_display_publication_promoted": false,
            "public_consumer_surface_migration_verified": false,
            "callable_metadata_projection_promoted": false,
            "callable_carrier_sufficiency_proven": false,
            "general_oxfunc_kernel_promoted": false,
            "w073_typed_only_formatting_guard_retained": true,
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
            "semantic_equivalence_statement": "This W040 OxFml seam runner binds fresh direct upstream-host evidence, W040 obligation-map rows, current OxFml public-surface notes, W073 typed-only formatting guards, publication/display boundary rows, callable carrier rows, registered-external watch rows, and exact blockers only. It does not change evaluator kernels, coordinator scheduling, recalc, publication, replay, pack, service, TraceCalc, TreeCalc, OxFml, OxFunc, Lean, or TLA semantics."
        });
        let validation = json!({
            "schema_version": W040_VALIDATION_SCHEMA_V1,
            "run_id": run_id,
            "status": if validation_failures.is_empty() {
                "w040_oxfml_seam_breadth_callable_metadata_packet_valid"
            } else {
                "w040_oxfml_seam_breadth_callable_metadata_packet_invalid"
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
            "schema_version": W040_RUN_SUMMARY_SCHEMA_V1,
            "run_id": run_id,
            "artifact_root": relative_artifact_root,
            "source_evidence_index_path": source_evidence_index_path,
            "w040_oxfml_consumed_surface_register_path": surface_register_path,
            "w040_publication_display_boundary_register_path": publication_display_path,
            "w040_callable_metadata_implementation_register_path": callable_metadata_path,
            "w040_exact_oxfml_seam_blocker_register_path": blocker_register_path,
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
            &artifact_root.join("w040_oxfml_consumed_surface_register.json"),
            &surface_register,
        )?;
        write_json(
            &artifact_root.join("w040_publication_display_boundary_register.json"),
            &publication_display_register,
        )?;
        write_json(
            &artifact_root.join("w040_callable_metadata_implementation_register.json"),
            &callable_metadata_register,
        )?;
        write_json(
            &artifact_root.join("w040_exact_oxfml_seam_blocker_register.json"),
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
            schema_version: W040_RUN_SUMMARY_SCHEMA_V1.to_string(),
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

#[allow(clippy::too_many_arguments)]
fn source_rows_w042(
    upstream_summary: &Value,
    upstream_case_index: &Value,
    formatting_intake: &Value,
    obligation_map: &Value,
    implementation_summary: &Value,
    callable_metadata_register: &Value,
    predecessor_oxfml_decision: &Value,
    diversity_decision: &Value,
    diversity_blockers: &Value,
    oxfml_notes: &str,
    w073_workset: &str,
    w073_handoff: &str,
) -> Vec<Value> {
    vec![
        json!({
            "row_id": "source.w042_upstream_host_direct_oxfml",
            "artifact": W042_UPSTREAM_HOST_SUMMARY,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": number_at(upstream_summary, "expectation_mismatch_count"),
            "failed_row_count": 0,
            "promotion_guard": bool_at(upstream_summary, "all_expectations_matched")
                && number_at(upstream_summary, "direct_oxfml_case_count") >= 3
                && number_at(upstream_summary, "let_lambda_case_count") >= 2
                && number_at(upstream_summary, "w073_typed_rule_case_count") >= 1
                && !bool_at(&upstream_summary["promotion_limits"], "general_oxfunc_kernel_claimed")
                && !bool_at(&upstream_summary["promotion_limits"], "pack_grade_replay_promoted")
                && !bool_at(&upstream_summary["promotion_limits"], "c5_promoted"),
            "semantic_state": "fresh_w042_direct_oxfml_runtime_slice_bound"
        }),
        json!({
            "row_id": "source.w042_upstream_host_case_index",
            "artifact": W042_UPSTREAM_HOST_CASE_INDEX,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": 0,
            "promotion_guard": tag_count(upstream_case_index, "direct-oxfml") >= 3
                && tag_count(upstream_case_index, "let-lambda") >= 2
                && tag_count(upstream_case_index, "w073") >= 1
                && tag_count(upstream_case_index, "structured-reference") >= 4
                && tag_count(upstream_case_index, "host-info") >= 2
                && tag_count(upstream_case_index, "rtd") >= 1,
            "semantic_state": "case_index_covers_w042_direct_oxfml_let_lambda_w073_provider_and_table_surfaces"
        }),
        json!({
            "row_id": "source.w042_closure_obligation_map",
            "artifact": W042_OBLIGATION_MAP,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": 0,
            "promotion_guard": number_at(obligation_map, "obligation_count") == 33
                && item_with_field_exists(obligation_map, "obligations", "id", "W042-OBL-024")
                && item_with_field_exists(obligation_map, "obligations", "id", "W042-OBL-025")
                && item_with_field_exists(obligation_map, "obligations", "id", "W042-OBL-026")
                && item_with_field_exists(obligation_map, "obligations", "id", "W042-OBL-027")
                && item_with_field_exists(obligation_map, "obligations", "id", "W042-OBL-028")
                && item_with_field_exists(obligation_map, "obligations", "id", "W042-OBL-029")
                && item_with_field_exists(obligation_map, "obligations", "id", "W042-OBL-033"),
            "semantic_state": "w042_oxfml_callable_public_migration_provider_obligations_bound"
        }),
        json!({
            "row_id": "source.w042_w073_formatting_intake",
            "artifact": W042_FORMATTING_INTAKE,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": 0,
            "promotion_guard": array_len_at(formatting_intake, "typed_rule_only_families") == 7
                && !bool_at(formatting_intake, "threshold_fallback_allowed_for_typed_families")
                && !bool_at(formatting_intake, "old_aggregate_visualization_option_strings_interpreted_as_typed_metadata")
                && text_at(formatting_intake, "status") == "typed_only_direct_replacement_guard_retained",
            "semantic_state": "w073_typed_only_formatting_intake_bound_for_w042"
        }),
        json!({
            "row_id": "source.oxfml_w073_direct_replacement_workset",
            "artifact": OXFML_W073_WORKSET,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": 0,
            "promotion_guard": w073_workset.contains("only accepted metadata source")
                && w073_workset.contains("intentionally ignored")
                && w073_workset.contains("bounded_visualization_threshold_strings_are_not_interpreted")
                && w073_workset.contains("bounded_aggregate_option_strings_are_not_interpreted"),
            "semantic_state": "current_oxfml_w073_direct_replacement_contract_reviewed"
        }),
        json!({
            "row_id": "source.oxfml_w073_downstream_handoff",
            "artifact": OXFML_W073_HANDOFF,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": 0,
            "promotion_guard": w073_handoff.contains("DNA OneCalc should update request construction")
                && w073_handoff.contains("will no longer produce aggregate or visualization effects")
                && w073_handoff.contains("typed_rule"),
            "semantic_state": "w073_public_request_construction_uptake_remains_downstream_handoff"
        }),
        json!({
            "row_id": "source.w042_callable_metadata_projection_blocker",
            "artifact": W042_CALLABLE_METADATA_REGISTER,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": 0,
            "promotion_guard": !bool_at(callable_metadata_register, "callable_metadata_projection_promoted")
                && row_with_field_exists(
                    callable_metadata_register,
                    "row_id",
                    "w042_callable_metadata_projection_exact_blocker"
                ),
            "semantic_state": "w042_callable_metadata_exact_blocker_retained"
        }),
        json!({
            "row_id": "source.w042_optimized_core_no_callable_or_registered_promotion",
            "artifact": W042_IMPLEMENTATION_SUMMARY,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": number_at(implementation_summary, "failed_row_count"),
            "promotion_guard": !bool_at(implementation_summary, "callable_metadata_projection_promoted")
                && !bool_at(implementation_summary, "full_optimized_core_verification_promoted")
                && number_at(implementation_summary, "exact_remaining_blocker_count") == 3,
            "semantic_state": "w042_optimized_core_callable_projection_no_promotion_guard_bound"
        }),
        json!({
            "row_id": "source.w041_oxfml_predecessor_no_promotion",
            "artifact": W041_OXFML_SEAM_DECISION,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": 0,
            "promotion_guard": !bool_at(predecessor_oxfml_decision, "broad_oxfml_seam_promoted")
                && !bool_at(predecessor_oxfml_decision, "callable_metadata_projection_promoted")
                && !bool_at(predecessor_oxfml_decision, "public_consumer_surface_migration_verified")
                && !bool_at(predecessor_oxfml_decision, "registered_external_callable_projection_promoted")
                && !bool_at(predecessor_oxfml_decision, "provider_failure_callable_publication_promoted"),
            "semantic_state": "predecessor_w041_oxfml_packet_preserves_no_promotion"
        }),
        json!({
            "row_id": "source.w042_diversity_no_broad_oxfml_promotion",
            "artifact": W042_DIVERSITY_DECISION,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": 0,
            "promotion_guard": !bool_at(diversity_decision, "broad_oxfml_seam_promoted")
                && !bool_at(diversity_decision, "callable_metadata_projection_promoted")
                && !bool_at(diversity_decision, "w073_formatting_handoff_triggered")
                && !bool_at(diversity_decision, "pack_grade_replay_promoted")
                && !bool_at(diversity_decision, "c5_promoted"),
            "semantic_state": "w042_diversity_packet_preserves_oxfml_no_promotion"
        }),
        json!({
            "row_id": "source.w042_diversity_oxfml_callable_blocker",
            "artifact": W042_DIVERSITY_BLOCKERS,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": 0,
            "promotion_guard": number_at(diversity_blockers, "exact_blocker_count") == 7
                && row_with_field_exists(
                    diversity_blockers,
                    "blocker_id",
                    "w042_diversity.oxfml_callable_breadth_dependency_absent"
                ),
            "semantic_state": "w042_diversity_retains_oxfml_callable_dependency_blocker"
        }),
        json!({
            "row_id": "source.oxfml_current_public_consumer_surface",
            "artifact": OXFML_INBOUND_NOTES,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": 0,
            "promotion_guard": oxfml_notes.contains("consumer::runtime")
                && oxfml_notes.contains("consumer::editor")
                && oxfml_notes.contains("consumer::replay")
                && oxfml_notes.contains("public `substrate::...` access is gone")
                && oxfml_notes.contains("test_support"),
            "semantic_state": "current_oxfml_public_consumer_surface_bound_as_watch_lane"
        }),
        json!({
            "row_id": "source.oxfml_runtime_facade_contract_notes",
            "artifact": OXFML_INBOUND_NOTES,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": 0,
            "promotion_guard": oxfml_notes.contains("RuntimeEnvironment")
                && oxfml_notes.contains("RuntimeFormulaRequest")
                && oxfml_notes.contains("RuntimeFormulaResult")
                && oxfml_notes.contains("RuntimeSessionFacade"),
            "semantic_state": "runtime_facade_contract_named_without_oxcalc_api_freeze"
        }),
        json!({
            "row_id": "source.oxfml_registered_external_notes",
            "artifact": OXFML_INBOUND_NOTES,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": 0,
            "promotion_guard": oxfml_notes.contains("RegisteredExternalDescriptor")
                && oxfml_notes.contains("RegisteredExternalCatalogMutationRequest")
                && oxfml_notes.contains("RegisteredExternalCatalogController")
                && oxfml_notes.contains("seven-field descriptor"),
            "semantic_state": "registered_external_packet_bound_at_note_level"
        }),
        json!({
            "row_id": "source.oxfml_fixture_standin_notes",
            "artifact": OXFML_INBOUND_NOTES,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": 0,
            "promotion_guard": oxfml_notes.contains("stand-in packet")
                && oxfml_notes.contains("structure-context identity")
                && oxfml_notes.contains("RegisteredExternalProvider")
                && oxfml_notes.contains("host/coordinator-supplied truths"),
            "semantic_state": "fixture_host_standin_packet_bound_without_production_api_freeze"
        }),
        json!({
            "row_id": "source.oxfml_publication_topology_residual_notes",
            "artifact": OXFML_INBOUND_NOTES,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": 0,
            "promotion_guard": oxfml_notes.contains("publication and topology consequence breadth")
                && oxfml_notes.contains("canonical but narrower")
                && oxfml_notes.contains("format_delta")
                && oxfml_notes.contains("display_delta"),
            "semantic_state": "publication_topology_residuals_bound_as_note_level_narrower_scope"
        }),
        json!({
            "row_id": "source.oxfml_registered_external_snapshot_consequence_notes",
            "artifact": OXFML_INBOUND_NOTES,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": 0,
            "promotion_guard": oxfml_notes.contains("new `LibraryContextSnapshot` generation")
                && oxfml_notes.contains("bind invalidation")
                && oxfml_notes.contains("targeted reevaluation by default"),
            "semantic_state": "registered_external_snapshot_and_invalidation_consequences_bound_as_note_level"
        }),
    ]
}

fn consumed_surface_rows_w042(
    upstream_summary: &Value,
    upstream_case_index: &Value,
    oxfml_notes: &str,
) -> Vec<Value> {
    vec![
        json!({
            "row_id": "surface.w042_direct_oxfml_runtime_facade",
            "disposition_kind": "exercised_current_surface",
            "source": W042_UPSTREAM_HOST_SUMMARY,
            "surface_state": "direct_runtime_facade_exercised_under_w042",
            "direct_oxfml_case_count": number_at(upstream_summary, "direct_oxfml_case_count"),
            "expectation_mismatch_count": number_at(upstream_summary, "expectation_mismatch_count"),
            "promotion_consequence": "current direct OxFml runtime surface is bound for this fixture slice without broad seam closure"
        }),
        json!({
            "row_id": "surface.w042_public_consumer_entry_points",
            "disposition_kind": "note_level_watch",
            "source": OXFML_INBOUND_NOTES,
            "surface_state": "current_public_consumer_surface_named",
            "consumer_runtime_present": oxfml_notes.contains("consumer::runtime"),
            "consumer_editor_present": oxfml_notes.contains("consumer::editor"),
            "consumer_replay_present": oxfml_notes.contains("consumer::replay"),
            "public_substrate_removed": oxfml_notes.contains("public `substrate::...` access is gone"),
            "promotion_consequence": "OxCalc records the current public consumer surface without claiming complete call-site migration or API freeze"
        }),
        json!({
            "row_id": "surface.w042_host_query_and_provider_families",
            "disposition_kind": "exercised_bounded_surface",
            "source": W042_UPSTREAM_HOST_CASE_INDEX,
            "surface_state": "host_info_rtd_reference_families_exercised",
            "host_info_case_count": tag_count(upstream_case_index, "host-info"),
            "rtd_case_count": tag_count(upstream_case_index, "rtd"),
            "bind_context_case_count": tag_count(upstream_case_index, "bind-context"),
            "promotion_consequence": "bounded host-query fixture coverage is present, not full provider-family closure"
        }),
        json!({
            "row_id": "surface.w042_structured_reference_table_context",
            "disposition_kind": "exercised_bounded_surface",
            "source": W042_UPSTREAM_HOST_CASE_INDEX,
            "surface_state": "table_context_and_structured_reference_cases_exercised",
            "table_context_case_count": tag_count(upstream_case_index, "table-context"),
            "structured_reference_case_count": tag_count(upstream_case_index, "structured-reference"),
            "promotion_consequence": "table packet direction is exercised for the bounded fixture slice, not broad workbook table closure"
        }),
        json!({
            "row_id": "surface.w042_runtime_facade_contract",
            "disposition_kind": "note_level_watch",
            "source": OXFML_INBOUND_NOTES,
            "surface_state": "runtime_facade_contract_named_without_shared_freeze",
            "runtime_formula_result_present": oxfml_notes.contains("RuntimeFormulaResult"),
            "runtime_session_facade_present": oxfml_notes.contains("RuntimeSessionFacade"),
            "promotion_consequence": "runtime facade contract is consumed as current direction, not shared seam freeze or broad host closure"
        }),
        json!({
            "row_id": "surface.w042_immutable_editor_packet",
            "disposition_kind": "note_level_watch",
            "source": OXFML_INBOUND_NOTES,
            "surface_state": "immutable_edit_packet_direction_named",
            "immutable_edit_request_present": oxfml_notes.contains("immutable edit request"),
            "validated_completion_present": oxfml_notes.contains("validated completion"),
            "promotion_consequence": "editor packet direction is watch-bound and does not promote public migration verification"
        }),
        json!({
            "row_id": "surface.w042_fixture_host_standin_packet",
            "disposition_kind": "note_level_watch",
            "source": OXFML_INBOUND_NOTES,
            "surface_state": "fixture_host_standin_packet_converged_for_deterministic_artifacts",
            "standin_packet_present": oxfml_notes.contains("stand-in packet"),
            "structure_context_identity_present": oxfml_notes.contains("structure-context identity"),
            "promotion_consequence": "fixture-host convergence supports deterministic artifacts but does not freeze the production OxCalc coordinator API"
        }),
        json!({
            "row_id": "surface.w042_registered_external_packet",
            "disposition_kind": "note_level_watch",
            "source": OXFML_INBOUND_NOTES,
            "surface_state": "registered_external_packet_converged_at_note_level",
            "descriptor_present": oxfml_notes.contains("RegisteredExternalDescriptor"),
            "mutation_request_present": oxfml_notes.contains("RegisteredExternalCatalogMutationRequest"),
            "promotion_consequence": "registered external packet naming remains note-level; callable metadata projection is not promoted"
        }),
        json!({
            "row_id": "surface.w042_execution_restriction_transport_residual",
            "disposition_kind": "canonical_but_narrower_watch",
            "source": OXFML_INBOUND_NOTES,
            "surface_state": "execution_restriction_transport_canonical_but_narrower",
            "execution_restriction_note_present": oxfml_notes.contains("execution-restriction transport"),
            "promotion_consequence": "execution-restriction transport remains a note-level narrowing lane until TreeCalc evidence exposes a concrete insufficiency"
        }),
        json!({
            "row_id": "surface.w042_caller_anchor_address_mode_residual",
            "disposition_kind": "canonical_but_narrower_watch",
            "source": OXFML_INBOUND_NOTES,
            "surface_state": "caller_anchor_address_mode_carriage_canonical_but_narrower",
            "caller_anchor_present": oxfml_notes.contains("caller_anchor"),
            "address_mode_present": oxfml_notes.contains("address-mode"),
            "promotion_consequence": "caller-anchor/address-mode carriage remains narrower than full relative-reference closure"
        }),
        json!({
            "row_id": "surface.w042_public_consumer_migration_verification",
            "disposition_kind": "exact_remaining_blocker",
            "source": OXFML_INBOUND_NOTES,
            "surface_state": "public_consumer_surface_migration_not_verified",
            "reason": "current public consumer surface is named and exercised through fixtures, but all OxCalc integration call sites are not migrated and verified in this bead",
            "promotion_consequence": "public consumer-surface migration verification remains unpromoted"
        }),
        json!({
            "row_id": "surface.w042_w073_public_request_construction_uptake",
            "disposition_kind": "exact_remaining_watch_blocker",
            "source": OXFML_W073_HANDOFF,
            "surface_state": "public_request_construction_uptake_not_verified_by_oxcalc",
            "reason": "W073 direct typed-rule request construction is documented for downstream uptake, but this OxCalc bead does not verify DNA OneCalc request construction.",
            "promotion_consequence": "public request-construction migration remains a downstream watch lane"
        }),
    ]
}

fn publication_display_rows_w042(
    w073_result: &Value,
    formatting_intake: &Value,
    oxfml_notes: &str,
    w073_workset: &str,
    w073_handoff: &str,
) -> Vec<Value> {
    let publication_surface = &w073_result["verification_publication_surface"];
    let candidate_result = &w073_result["candidate_result"];
    vec![
        json!({
            "row_id": "publication.w042_w073_typed_only_formatting_guard",
            "disposition_kind": "exercised_current_surface",
            "source": W042_W073_RESULT,
            "boundary_state": "typed_rule_only_guard_exercised_under_w042",
            "typed_rule_family_count": array_len_at(publication_surface, "conditional_formatting_typed_rule_families"),
            "legacy_thresholds_present": array_len_at(publication_surface, "conditional_formatting_thresholds") > 0,
            "typed_only_family_count": array_len_at(formatting_intake, "typed_rule_only_families"),
            "format_delta_present": bool_at(publication_surface, "format_delta_present"),
            "display_delta_present": bool_at(publication_surface, "display_delta_present"),
            "promotion_consequence": "W073 typed_rule-only behavior is carried as current consumed evidence without broad formatting closure"
        }),
        json!({
            "row_id": "publication.w042_w073_no_threshold_fallback_for_aggregate_visualization",
            "disposition_kind": "exercised_current_surface",
            "source": W042_FORMATTING_INTAKE,
            "boundary_state": "direct_replacement_contract_retained",
            "threshold_fallback_allowed_for_typed_families": bool_at(
                formatting_intake,
                "threshold_fallback_allowed_for_typed_families"
            ),
            "old_string_interpretation_allowed": bool_at(
                formatting_intake,
                "old_aggregate_visualization_option_strings_interpreted_as_typed_metadata"
            ),
            "threshold_text_family_count": array_len_at(formatting_intake, "thresholds_remain_meaningful_for"),
            "promotion_consequence": "OxCalc W042 evidence must not infer fallback from W072 threshold strings for W073 aggregate/visualization families"
        }),
        json!({
            "row_id": "publication.w042_w073_old_string_non_interpretation_evidence",
            "disposition_kind": "exercised_current_surface",
            "source": OXFML_W073_WORKSET,
            "boundary_state": "old_bounded_strings_are_not_interpreted_for_w073_families",
            "visualization_old_string_test_present": w073_workset.contains("bounded_visualization_threshold_strings_are_not_interpreted"),
            "aggregate_old_string_test_present": w073_workset.contains("bounded_aggregate_option_strings_are_not_interpreted"),
            "handoff_requires_typed_rule": w073_handoff.contains("emit `typed_rule`"),
            "promotion_consequence": "typed-only contract is watched without making OxCalc own downstream request construction"
        }),
        json!({
            "row_id": "publication.w042_format_delta_display_delta_distinct",
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
            "row_id": "publication.w042_candidate_commit_value_shape_surface",
            "disposition_kind": "exercised_current_surface",
            "source": W042_W073_RESULT,
            "boundary_state": "candidate_commit_value_shape_surface_exercised",
            "candidate_result_id": text_at(candidate_result, "candidate_result_id"),
            "published_value_class": text_at(candidate_result, "published_value_class"),
            "shape_delta_present": bool_at(candidate_result, "shape_delta_present"),
            "commit_kind": text_at(&w073_result["commit_decision"], "kind"),
            "promotion_consequence": "candidate/commit and value/shape publication surfaces are exercised for the W073 fixture only"
        }),
        json!({
            "row_id": "publication.w042_topology_effect_fact_watch",
            "disposition_kind": "note_level_watch",
            "source": OXFML_INBOUND_NOTES,
            "boundary_state": "topology_effect_facts_reachable_but_narrower",
            "topology_fact_note_present": oxfml_notes.contains("topology/effect fact refs"),
            "candidate_result_id_note_present": oxfml_notes.contains("candidate_result_id"),
            "promotion_consequence": "topology/effect fact carriage is recognized without broad topology/publication closure"
        }),
        json!({
            "row_id": "publication.w042_runtime_derived_effects_watch",
            "disposition_kind": "canonical_but_narrower_watch",
            "source": OXFML_INBOUND_NOTES,
            "boundary_state": "runtime_derived_effects_are_surfaced_but_not_full_transport_freeze",
            "capability_sensitive_present": oxfml_notes.contains("capability-sensitive"),
            "execution_restriction_present": oxfml_notes.contains("execution-restriction"),
            "promotion_consequence": "runtime-derived effects remain consumed facts, not final broad transport closure"
        }),
        json!({
            "row_id": "publication.w042_provider_failure_callable_publication_watch",
            "disposition_kind": "note_level_watch",
            "source": OXFML_INBOUND_NOTES,
            "boundary_state": "provider_failure_callable_publication_watch_lane",
            "provider_failure_note_present": oxfml_notes.contains("provider-failure"),
            "callable_publication_note_present": oxfml_notes.contains("callable-publication"),
            "promotion_consequence": "provider-failure and callable-publication remain watch lanes only until they become coordinator-visible in exercised evidence"
        }),
        json!({
            "row_id": "publication.w042_publication_topology_consequence_breadth",
            "disposition_kind": "canonical_but_narrower_watch",
            "source": OXFML_INBOUND_NOTES,
            "boundary_state": "publication_topology_consequence_breadth_narrower",
            "publication_topology_note_present": oxfml_notes.contains("publication and topology consequence breadth"),
            "canonical_but_narrower_present": oxfml_notes.contains("canonical but narrower"),
            "promotion_consequence": "publication/topology consequence breadth remains note-level until direct TreeCalc-facing evidence requires a narrower handoff"
        }),
        json!({
            "row_id": "publication.w042_broad_display_publication_breadth",
            "disposition_kind": "exact_remaining_blocker",
            "source": OXFML_INBOUND_NOTES,
            "boundary_state": "broad_display_publication_unpromoted",
            "reason": "current direct fixture coverage does not exercise broad display-facing categories, broad topology/publication consequences, or all future format/display families",
            "promotion_consequence": "broad OxFml display/publication closure remains unpromoted"
        }),
    ]
}

fn callable_metadata_rows_w042(
    let_lexical_result: &Value,
    returned_lambda_result: &Value,
    obligation_map: &Value,
    callable_metadata_register: &Value,
    oxfml_notes: &str,
) -> Vec<Value> {
    vec![
        json!({
            "row_id": "callable.w042_let_lambda_lexical_capture",
            "disposition_kind": "exercised_current_surface",
            "source": W042_LET_LEXICAL_RESULT,
            "callable_state": "narrow_let_lambda_carrier_exercised",
            "function_id_count": array_len_at(&let_lexical_result["evaluation_trace"], "function_ids"),
            "prepared_call_count": number_at(&let_lexical_result["evaluation_trace"], "prepared_call_count"),
            "narrow_let_lambda_carrier": bool_at(&let_lexical_result["w037_interpretation"], "narrow_let_lambda_carrier"),
            "promotion_consequence": "lexical LET/LAMBDA carrier is exercised without general OxFunc kernel promotion"
        }),
        json!({
            "row_id": "callable.w042_returned_lambda_invocation",
            "disposition_kind": "exercised_current_surface",
            "source": W042_RETURNED_LAMBDA_RESULT,
            "callable_state": "returned_lambda_value_carrier_exercised",
            "function_id_count": array_len_at(&returned_lambda_result["evaluation_trace"], "function_ids"),
            "prepared_call_count": number_at(&returned_lambda_result["evaluation_trace"], "prepared_call_count"),
            "payload_summary": text_at(&returned_lambda_result["returned_value_surface"], "payload_summary"),
            "promotion_consequence": "returned-lambda invocation is value-carrier evidence, not metadata projection evidence"
        }),
        json!({
            "row_id": "callable.w042_callable_metadata_projection",
            "disposition_kind": "exact_remaining_blocker",
            "source": W042_CALLABLE_METADATA_REGISTER,
            "callable_state": "metadata_projection_absent",
            "blocker_present": row_with_field_exists(
                callable_metadata_register,
                "row_id",
                "w042_callable_metadata_projection_exact_blocker"
            ),
            "promotion_consequence": "callable metadata projection remains blocked until a projection fixture or carrier sufficiency proof exists"
        }),
        json!({
            "row_id": "callable.w042_carrier_sufficiency_proof",
            "disposition_kind": "exact_remaining_blocker",
            "source": W042_OBLIGATION_MAP,
            "callable_state": "carrier_sufficiency_proof_absent",
            "obligation_present": item_with_field_exists(
                obligation_map,
                "obligations",
                "id",
                "W042-OBL-026"
            ),
            "promotion_consequence": "callable carrier sufficiency remains unproven for broad metadata projection"
        }),
        json!({
            "row_id": "callable.w042_callable_metadata_publication_surface",
            "disposition_kind": "exact_remaining_blocker",
            "source": W042_OBLIGATION_MAP,
            "callable_state": "callable_metadata_publication_surface_not_evidenced",
            "obligation_present": item_with_field_exists(
                obligation_map,
                "obligations",
                "id",
                "W042-OBL-027"
            ),
            "promotion_consequence": "callable metadata publication remains separated from LET/LAMBDA value-carrier evidence"
        }),
        json!({
            "row_id": "callable.w042_registered_external_callable_metadata",
            "disposition_kind": "note_level_watch",
            "source": OXFML_INBOUND_NOTES,
            "callable_state": "registered_external_metadata_not_current_projection",
            "registered_external_descriptor_present": oxfml_notes.contains("RegisteredExternalDescriptor"),
            "mutation_request_present": oxfml_notes.contains("RegisteredExternalCatalogMutationRequest"),
            "promotion_consequence": "registered-external packet alignment does not close callable metadata projection in OxCalc"
        }),
        json!({
            "row_id": "callable.w042_registered_external_snapshot_consequence",
            "disposition_kind": "note_level_watch",
            "source": OXFML_INBOUND_NOTES,
            "callable_state": "registered_external_snapshot_invalidation_note_level",
            "snapshot_generation_present": oxfml_notes.contains("new `LibraryContextSnapshot` generation"),
            "bind_invalidation_present": oxfml_notes.contains("bind invalidation"),
            "promotion_consequence": "snapshot and invalidation consequences are note-level convergence, not optimized/core implementation promotion"
        }),
        json!({
            "row_id": "callable.w042_provider_failure_callable_publication_watch",
            "disposition_kind": "note_level_watch",
            "source": OXFML_INBOUND_NOTES,
            "callable_state": "provider_failure_and_callable_publication_watch_only",
            "provider_failure_present": oxfml_notes.contains("provider-failure"),
            "callable_publication_present": oxfml_notes.contains("callable-publication"),
            "promotion_consequence": "provider-failure and callable-publication remain watch lanes until concrete coordinator-visible evidence exists"
        }),
        json!({
            "row_id": "callable.w042_registered_external_seven_field_descriptor",
            "disposition_kind": "note_level_watch",
            "source": OXFML_INBOUND_NOTES,
            "callable_state": "registered_external_descriptor_shape_converged_at_note_level",
            "seven_field_descriptor_present": oxfml_notes.contains("seven-field descriptor"),
            "stable_registration_id_present": oxfml_notes.contains("stable_registration_id"),
            "promotion_consequence": "descriptor shape is note-level converged and not callable projection implementation evidence"
        }),
        json!({
            "row_id": "callable.w042_general_oxfunc_kernel_boundary",
            "disposition_kind": "accepted_external_boundary",
            "source": OXFML_INBOUND_NOTES,
            "callable_state": "general_oxfunc_kernel_external",
            "notes_name_oxfunc": oxfml_notes.contains("OxFunc"),
            "promotion_consequence": "OxCalc keeps only the narrow LET/LAMBDA carrier seam in this formalization scope"
        }),
    ]
}

fn registered_external_rows_w042(
    upstream_case_index: &Value,
    obligation_map: &Value,
    oxfml_notes: &str,
) -> Vec<Value> {
    vec![
        json!({
            "row_id": "registered_external.w042_host_provider_fixture_slice",
            "disposition_kind": "exercised_bounded_surface",
            "source": W042_UPSTREAM_HOST_CASE_INDEX,
            "provider_state": "host_info_rtd_provider_slice_exercised",
            "host_info_case_count": tag_count(upstream_case_index, "host-info"),
            "rtd_case_count": tag_count(upstream_case_index, "rtd"),
            "promotion_consequence": "host-provider fixture rows are present, but deferred provider families remain out of scope"
        }),
        json!({
            "row_id": "registered_external.w042_registered_external_optional_standin",
            "disposition_kind": "note_level_watch",
            "source": OXFML_INBOUND_NOTES,
            "provider_state": "registered_external_provider_optional_standin",
            "registered_external_provider_present": oxfml_notes.contains("RegisteredExternalProvider"),
            "host_coordinator_supplied_truth_present": oxfml_notes.contains("host/coordinator-supplied truths"),
            "promotion_consequence": "optional stand-in packet presence does not freeze the production coordinator API"
        }),
        json!({
            "row_id": "registered_external.w042_direct_packet_field_names",
            "disposition_kind": "note_level_watch",
            "source": OXFML_INBOUND_NOTES,
            "provider_state": "direct_packet_field_names_converged_at_note_level",
            "register_id_request_present": oxfml_notes.contains("RegisterIdRequest"),
            "registered_external_call_request_present": oxfml_notes.contains("RegisteredExternalCallRequest"),
            "promotion_consequence": "direct packet naming is note-level aligned and not a shared seam-freeze promotion"
        }),
        json!({
            "row_id": "registered_external.w042_seven_field_descriptor",
            "disposition_kind": "note_level_watch",
            "source": OXFML_INBOUND_NOTES,
            "provider_state": "minimum_descriptor_shape_converged_at_note_level",
            "seven_field_descriptor_present": oxfml_notes.contains("seven-field descriptor"),
            "stable_registration_id_present": oxfml_notes.contains("stable_registration_id"),
            "promotion_consequence": "registered-external descriptor shape remains note-level convergence until exercised projection evidence exists"
        }),
        json!({
            "row_id": "registered_external.w042_catalog_mutation_funnel",
            "disposition_kind": "note_level_watch",
            "source": OXFML_INBOUND_NOTES,
            "provider_state": "catalog_mutation_funnel_remains_oxfml_owned",
            "catalog_mutation_request_present": oxfml_notes.contains("RegisteredExternalCatalogMutationRequest"),
            "catalog_controller_present": oxfml_notes.contains("RegisteredExternalCatalogController"),
            "promotion_consequence": "host/coordinator registration remains typed mutation requests funneled into OxFunc-owned catalog truth"
        }),
        json!({
            "row_id": "registered_external.w042_snapshot_invalidation_consequences",
            "disposition_kind": "note_level_watch",
            "source": OXFML_INBOUND_NOTES,
            "provider_state": "snapshot_and_invalidation_consequences_note_level",
            "snapshot_generation_present": oxfml_notes.contains("new `LibraryContextSnapshot` generation"),
            "bind_invalidation_present": oxfml_notes.contains("bind invalidation"),
            "targeted_reevaluation_present": oxfml_notes.contains("targeted reevaluation by default"),
            "promotion_consequence": "snapshot-generation and invalidation consequences are not yet optimized/core implementation evidence"
        }),
        json!({
            "row_id": "registered_external.w042_provider_failure_watch_lane",
            "disposition_kind": "note_level_watch",
            "source": OXFML_INBOUND_NOTES,
            "provider_state": "provider_failure_watch_lane_only",
            "provider_failure_present": oxfml_notes.contains("provider-failure"),
            "promotion_consequence": "provider-failure remains a watch lane until it becomes coordinator-visible in exercised evidence"
        }),
        json!({
            "row_id": "registered_external.w042_registered_external_projection_blocked",
            "disposition_kind": "exact_remaining_blocker",
            "source": W042_OBLIGATION_MAP,
            "provider_state": "registered_external_callable_projection_not_promoted",
            "obligation_present": item_with_field_exists(
                obligation_map,
                "obligations",
                "id",
                "W042-OBL-028"
            ),
            "promotion_consequence": "registered-external callable projection remains blocked until concrete OxCalc projection evidence exists"
        }),
    ]
}

fn exact_blockers_w042(
    obligation_map: &Value,
    callable_metadata_register: &Value,
    diversity_blockers: &Value,
    oxfml_notes: &str,
    w073_handoff: &str,
) -> Vec<Value> {
    vec![
        json!({
            "blocker_id": "w042_oxfml.callable_metadata_projection_absent",
            "owner": "calc-czd.8",
            "status_after_run": "exact_remaining_blocker",
            "evidence": W042_CALLABLE_METADATA_REGISTER,
            "blocker_present": row_with_field_exists(
                callable_metadata_register,
                "row_id",
                "w042_callable_metadata_projection_exact_blocker"
            ),
            "reason": "direct LET/LAMBDA value-carrier evidence exists, but callable metadata projection fixture or implementation evidence remains absent.",
            "promotion_consequence": "callable metadata projection remains unpromoted"
        }),
        json!({
            "blocker_id": "w042_oxfml.callable_carrier_sufficiency_proof_absent",
            "owner": "calc-czd.4; calc-czd.8",
            "status_after_run": "exact_remaining_blocker",
            "evidence": W042_OBLIGATION_MAP,
            "obligation_present": item_with_field_exists(
                obligation_map,
                "obligations",
                "id",
                "W042-OBL-026"
            ),
            "reason": "the consumed LET/LAMBDA carrier rows are exercised, but no proof shows that carrier sufficiency replaces callable metadata projection for the broader consumed surface.",
            "promotion_consequence": "broad callable conformance remains unpromoted"
        }),
        json!({
            "blocker_id": "w042_oxfml.broad_display_publication_breadth_unexercised",
            "owner": "calc-czd.8",
            "status_after_run": "exact_remaining_blocker",
            "evidence": OXFML_INBOUND_NOTES,
            "format_display_notes_present": oxfml_notes.contains("format_delta") && oxfml_notes.contains("display_delta"),
            "reason": "the current direct slice carries typed formatting and distinct format/display categories, but not broad display-facing or publication/topology closure.",
            "promotion_consequence": "broad OxFml display/publication closure remains unpromoted"
        }),
        json!({
            "blocker_id": "w042_oxfml.public_consumer_surface_migration_not_verified",
            "owner": "calc-czd.8; future implementation lane",
            "status_after_run": "exact_remaining_watch_blocker",
            "evidence": OXFML_INBOUND_NOTES,
            "consumer_surface_present": oxfml_notes.contains("consumer::runtime")
                && oxfml_notes.contains("consumer::editor")
                && oxfml_notes.contains("consumer::replay"),
            "reason": "OxFml names the public consumer surface, but this bead does not migrate or verify all OxCalc integration call sites against that surface.",
            "promotion_consequence": "public surface alignment remains watch-bound rather than promoted"
        }),
        json!({
            "blocker_id": "w042_oxfml.w073_public_request_construction_uptake_not_verified",
            "owner": "calc-czd.8; downstream:DNA OneCalc",
            "status_after_run": "exact_remaining_watch_blocker",
            "evidence": OXFML_W073_HANDOFF,
            "handoff_requires_typed_rule": w073_handoff.contains("emit `typed_rule`"),
            "reason": "OxFml W073 is now typed-rule-only for aggregate and visualization metadata, but this OxCalc bead does not verify downstream request construction.",
            "promotion_consequence": "W073 public request-construction uptake remains unpromoted in OxCalc"
        }),
        json!({
            "blocker_id": "w042_oxfml.registered_external_callable_projection_deferred",
            "owner": "calc-czd.8; external:OxFml/OxFunc",
            "status_after_run": "exact_remaining_watch_blocker",
            "evidence": OXFML_INBOUND_NOTES,
            "registered_external_descriptor_present": oxfml_notes.contains("RegisteredExternalDescriptor"),
            "reason": "registered-external packet naming is note-level converged, but no OxCalc callable metadata projection fixture is exercised.",
            "promotion_consequence": "registered external callable metadata projection remains unpromoted"
        }),
        json!({
            "blocker_id": "w042_oxfml.provider_failure_callable_publication_watch_lane",
            "owner": "calc-czd.8; OxFml watch lane",
            "status_after_run": "exact_remaining_watch_blocker",
            "evidence": OXFML_INBOUND_NOTES,
            "provider_failure_note_present": oxfml_notes.contains("provider-failure"),
            "callable_publication_note_present": oxfml_notes.contains("callable-publication"),
            "reason": "provider-failure and callable-publication are explicitly watch lanes until coordinator-visible evidence exercises them.",
            "promotion_consequence": "provider failure and callable publication seam closure remains unpromoted"
        }),
        json!({
            "blocker_id": "w042_oxfml.publication_topology_consequence_breadth_narrower",
            "owner": "calc-czd.8; W026 residual lane",
            "status_after_run": "exact_remaining_watch_blocker",
            "evidence": OXFML_INBOUND_NOTES,
            "publication_topology_note_present": oxfml_notes.contains("publication and topology consequence breadth"),
            "reason": "publication and topology consequence breadth is canonical but narrower and has not been frozen as broad coordinator seam closure.",
            "promotion_consequence": "broad publication/topology closure remains unpromoted"
        }),
        json!({
            "blocker_id": "w042_oxfml.diversity_dependency_oxfml_callable_breadth_absent",
            "owner": "calc-czd.7; calc-czd.8",
            "status_after_run": "exact_remaining_blocker",
            "evidence": W042_DIVERSITY_BLOCKERS,
            "blocker_present": row_with_field_exists(
                diversity_blockers,
                "blocker_id",
                "w042_diversity.oxfml_callable_breadth_dependency_absent"
            ),
            "reason": "W042 diversity evidence explicitly depends on OxFml/callable breadth that this bead classifies but does not fully promote.",
            "promotion_consequence": "diversity and release-grade promotion remain blocked"
        }),
        json!({
            "blocker_id": "w042_oxfml.general_oxfunc_kernel_external",
            "owner": "external:OxFunc",
            "status_after_run": "accepted_external_boundary",
            "evidence": W042_OBLIGATION_MAP,
            "obligation_present": item_with_field_exists(
                obligation_map,
                "obligations",
                "id",
                "W042-OBL-033"
            ),
            "reason": "W042 includes the narrow LET/LAMBDA carrier seam, not general OxFunc kernel formalization inside OxCalc.",
            "promotion_consequence": "general OxFunc kernel promotion remains outside OxCalc"
        }),
    ]
}

#[allow(clippy::too_many_arguments)]
fn source_rows_w041(
    upstream_summary: &Value,
    upstream_case_index: &Value,
    formatting_intake: &Value,
    obligation_map: &Value,
    implementation_blockers: &Value,
    predecessor_oxfml_decision: &Value,
    diversity_decision: &Value,
    oxfml_notes: &str,
) -> Vec<Value> {
    vec![
        json!({
            "row_id": "source.w041_upstream_host_direct_oxfml",
            "artifact": W041_UPSTREAM_HOST_SUMMARY,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": number_at(upstream_summary, "expectation_mismatch_count"),
            "failed_row_count": 0,
            "promotion_guard": bool_at(upstream_summary, "all_expectations_matched")
                && number_at(upstream_summary, "direct_oxfml_case_count") >= 3
                && number_at(upstream_summary, "let_lambda_case_count") >= 2
                && number_at(upstream_summary, "w073_typed_rule_case_count") >= 1
                && !bool_at(&upstream_summary["promotion_limits"], "general_oxfunc_kernel_claimed")
                && !bool_at(&upstream_summary["promotion_limits"], "pack_grade_replay_promoted")
                && !bool_at(&upstream_summary["promotion_limits"], "c5_promoted"),
            "semantic_state": "fresh_w041_direct_oxfml_runtime_slice_bound"
        }),
        json!({
            "row_id": "source.w041_upstream_host_case_index",
            "artifact": W041_UPSTREAM_HOST_CASE_INDEX,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": 0,
            "promotion_guard": tag_count(upstream_case_index, "direct-oxfml") >= 3
                && tag_count(upstream_case_index, "let-lambda") >= 2
                && tag_count(upstream_case_index, "w073") >= 1
                && tag_count(upstream_case_index, "structured-reference") >= 4
                && tag_count(upstream_case_index, "host-info") >= 2
                && tag_count(upstream_case_index, "rtd") >= 1,
            "semantic_state": "case_index_covers_w041_direct_oxfml_let_lambda_w073_provider_and_table_surfaces"
        }),
        json!({
            "row_id": "source.w041_successor_obligation_map",
            "artifact": W041_OBLIGATION_MAP,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": 0,
            "promotion_guard": number_at(obligation_map, "obligation_count") == 28
                && item_with_field_exists(obligation_map, "obligations", "obligation_id", "W041-OBL-005")
                && item_with_field_exists(obligation_map, "obligations", "obligation_id", "W041-OBL-021")
                && item_with_field_exists(obligation_map, "obligations", "obligation_id", "W041-OBL-022")
                && item_with_field_exists(obligation_map, "obligations", "obligation_id", "W041-OBL-023")
                && item_with_field_exists(obligation_map, "obligations", "obligation_id", "W041-OBL-024")
                && item_with_field_exists(obligation_map, "obligations", "obligation_id", "W041-OBL-028"),
            "semantic_state": "w041_oxfml_callable_and_provider_obligations_bound"
        }),
        json!({
            "row_id": "source.w041_w073_formatting_intake",
            "artifact": W040_FORMATTING_INTAKE,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": 0,
            "promotion_guard": array_len_at(formatting_intake, "typed_rule_only_families") == 7
                && !bool_at(formatting_intake, "w072_threshold_fallback_allowed_for_aggregate_visualization")
                && text_at(formatting_intake, "contract_mode") == "direct_replacement_for_aggregate_and_visualization_metadata",
            "semantic_state": "w073_typed_only_formatting_intake_bound_for_w041"
        }),
        json!({
            "row_id": "source.w041_callable_metadata_blocker",
            "artifact": W041_IMPLEMENTATION_BLOCKERS,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": 0,
            "promotion_guard": row_with_field_exists(
                implementation_blockers,
                "row_id",
                "w041_callable_metadata_projection_exact_blocker"
            ),
            "semantic_state": "w041_callable_metadata_exact_blocker_present"
        }),
        json!({
            "row_id": "source.w040_oxfml_predecessor_no_promotion",
            "artifact": W040_OXFML_SEAM_DECISION,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": 0,
            "promotion_guard": !bool_at(predecessor_oxfml_decision, "broad_oxfml_seam_promoted")
                && !bool_at(predecessor_oxfml_decision, "callable_metadata_projection_promoted")
                && !bool_at(predecessor_oxfml_decision, "public_consumer_surface_migration_verified")
                && !bool_at(predecessor_oxfml_decision, "w073_formatting_handoff_triggered"),
            "semantic_state": "predecessor_w040_oxfml_packet_preserves_no_promotion"
        }),
        json!({
            "row_id": "source.w041_diversity_no_broad_oxfml_promotion",
            "artifact": W041_DIVERSITY_DECISION,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": 0,
            "promotion_guard": !bool_at(diversity_decision, "broad_oxfml_seam_promoted")
                && !bool_at(diversity_decision, "callable_metadata_projection_promoted")
                && !bool_at(diversity_decision, "w073_formatting_handoff_triggered")
                && !bool_at(diversity_decision, "pack_grade_replay_promoted")
                && !bool_at(diversity_decision, "c5_promoted"),
            "semantic_state": "w041_diversity_packet_preserves_oxfml_no_promotion"
        }),
        json!({
            "row_id": "source.oxfml_current_public_consumer_surface",
            "artifact": OXFML_INBOUND_NOTES,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": 0,
            "promotion_guard": oxfml_notes.contains("consumer::runtime")
                && oxfml_notes.contains("consumer::editor")
                && oxfml_notes.contains("consumer::replay")
                && oxfml_notes.contains("public `substrate::...` access is gone")
                && oxfml_notes.contains("test_support"),
            "semantic_state": "current_oxfml_public_consumer_surface_bound_as_watch_lane"
        }),
        json!({
            "row_id": "source.oxfml_runtime_facade_contract_notes",
            "artifact": OXFML_INBOUND_NOTES,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": 0,
            "promotion_guard": oxfml_notes.contains("RuntimeEnvironment")
                && oxfml_notes.contains("RuntimeFormulaRequest")
                && oxfml_notes.contains("RuntimeFormulaResult")
                && oxfml_notes.contains("RuntimeSessionFacade"),
            "semantic_state": "runtime_facade_contract_named_without_oxcalc_api_freeze"
        }),
        json!({
            "row_id": "source.oxfml_registered_external_notes",
            "artifact": OXFML_INBOUND_NOTES,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": 0,
            "promotion_guard": oxfml_notes.contains("RegisteredExternalDescriptor")
                && oxfml_notes.contains("RegisteredExternalCatalogMutationRequest")
                && oxfml_notes.contains("RegisteredExternalCatalogController")
                && oxfml_notes.contains("seven-field descriptor"),
            "semantic_state": "registered_external_packet_bound_at_note_level"
        }),
        json!({
            "row_id": "source.oxfml_fixture_standin_notes",
            "artifact": OXFML_INBOUND_NOTES,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": 0,
            "promotion_guard": oxfml_notes.contains("stand-in packet")
                && oxfml_notes.contains("structure-context identity")
                && oxfml_notes.contains("RegisteredExternalProvider")
                && oxfml_notes.contains("host/coordinator-supplied truths"),
            "semantic_state": "fixture_host_standin_packet_bound_without_production_api_freeze"
        }),
        json!({
            "row_id": "source.oxfml_publication_topology_residual_notes",
            "artifact": OXFML_INBOUND_NOTES,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": 0,
            "promotion_guard": oxfml_notes.contains("publication and topology consequence breadth")
                && oxfml_notes.contains("canonical but narrower")
                && oxfml_notes.contains("format_delta")
                && oxfml_notes.contains("display_delta"),
            "semantic_state": "publication_topology_residuals_bound_as_note_level_narrower_scope"
        }),
    ]
}

fn consumed_surface_rows_w041(
    upstream_summary: &Value,
    upstream_case_index: &Value,
    oxfml_notes: &str,
) -> Vec<Value> {
    vec![
        json!({
            "row_id": "surface.w041_direct_oxfml_runtime_facade",
            "disposition_kind": "exercised_current_surface",
            "source": W041_UPSTREAM_HOST_SUMMARY,
            "surface_state": "direct_runtime_facade_exercised_under_w041",
            "direct_oxfml_case_count": number_at(upstream_summary, "direct_oxfml_case_count"),
            "expectation_mismatch_count": number_at(upstream_summary, "expectation_mismatch_count"),
            "promotion_consequence": "current direct OxFml runtime surface is bound for this fixture slice without broad seam closure"
        }),
        json!({
            "row_id": "surface.w041_public_consumer_entry_points",
            "disposition_kind": "note_level_watch",
            "source": OXFML_INBOUND_NOTES,
            "surface_state": "current_public_consumer_surface_named",
            "consumer_runtime_present": oxfml_notes.contains("consumer::runtime"),
            "consumer_editor_present": oxfml_notes.contains("consumer::editor"),
            "consumer_replay_present": oxfml_notes.contains("consumer::replay"),
            "public_substrate_removed": oxfml_notes.contains("public `substrate::...` access is gone"),
            "promotion_consequence": "OxCalc records the current public consumer surface without claiming complete call-site migration or API freeze"
        }),
        json!({
            "row_id": "surface.w041_host_query_and_provider_families",
            "disposition_kind": "exercised_bounded_surface",
            "source": W041_UPSTREAM_HOST_CASE_INDEX,
            "surface_state": "host_info_rtd_reference_families_exercised",
            "host_info_case_count": tag_count(upstream_case_index, "host-info"),
            "rtd_case_count": tag_count(upstream_case_index, "rtd"),
            "bind_context_case_count": tag_count(upstream_case_index, "bind-context"),
            "promotion_consequence": "bounded host-query fixture coverage is present, not full provider-family closure"
        }),
        json!({
            "row_id": "surface.w041_structured_reference_table_context",
            "disposition_kind": "exercised_bounded_surface",
            "source": W041_UPSTREAM_HOST_CASE_INDEX,
            "surface_state": "table_context_and_structured_reference_cases_exercised",
            "table_context_case_count": tag_count(upstream_case_index, "table-context"),
            "structured_reference_case_count": tag_count(upstream_case_index, "structured-reference"),
            "promotion_consequence": "table packet direction is exercised for the bounded fixture slice, not broad workbook table closure"
        }),
        json!({
            "row_id": "surface.w041_runtime_facade_contract",
            "disposition_kind": "note_level_watch",
            "source": OXFML_INBOUND_NOTES,
            "surface_state": "runtime_facade_contract_named_without_shared_freeze",
            "runtime_formula_result_present": oxfml_notes.contains("RuntimeFormulaResult"),
            "runtime_session_facade_present": oxfml_notes.contains("RuntimeSessionFacade"),
            "promotion_consequence": "runtime facade contract is consumed as current direction, not shared seam freeze or broad host closure"
        }),
        json!({
            "row_id": "surface.w041_immutable_editor_packet",
            "disposition_kind": "note_level_watch",
            "source": OXFML_INBOUND_NOTES,
            "surface_state": "immutable_edit_packet_direction_named",
            "immutable_edit_request_present": oxfml_notes.contains("immutable edit request"),
            "validated_completion_present": oxfml_notes.contains("validated completion"),
            "promotion_consequence": "editor packet direction is watch-bound and does not promote public migration verification"
        }),
        json!({
            "row_id": "surface.w041_fixture_host_standin_packet",
            "disposition_kind": "note_level_watch",
            "source": OXFML_INBOUND_NOTES,
            "surface_state": "fixture_host_standin_packet_converged_for_deterministic_artifacts",
            "standin_packet_present": oxfml_notes.contains("stand-in packet"),
            "structure_context_identity_present": oxfml_notes.contains("structure-context identity"),
            "promotion_consequence": "fixture-host convergence supports deterministic artifacts but does not freeze the production OxCalc coordinator API"
        }),
        json!({
            "row_id": "surface.w041_execution_restriction_transport_residual",
            "disposition_kind": "canonical_but_narrower_watch",
            "source": OXFML_INBOUND_NOTES,
            "surface_state": "execution_restriction_transport_canonical_but_narrower",
            "execution_restriction_note_present": oxfml_notes.contains("execution-restriction transport"),
            "promotion_consequence": "execution-restriction transport remains a note-level narrowing lane until TreeCalc evidence exposes a concrete insufficiency"
        }),
        json!({
            "row_id": "surface.w041_caller_anchor_address_mode_residual",
            "disposition_kind": "canonical_but_narrower_watch",
            "source": OXFML_INBOUND_NOTES,
            "surface_state": "caller_anchor_address_mode_carriage_canonical_but_narrower",
            "caller_anchor_present": oxfml_notes.contains("caller_anchor"),
            "address_mode_present": oxfml_notes.contains("address-mode"),
            "promotion_consequence": "caller-anchor/address-mode carriage remains narrower than full relative-reference closure"
        }),
        json!({
            "row_id": "surface.w041_public_consumer_migration_verification",
            "disposition_kind": "exact_remaining_blocker",
            "source": OXFML_INBOUND_NOTES,
            "surface_state": "public_consumer_surface_migration_not_verified",
            "reason": "current public consumer surface is named and exercised through fixtures, but all OxCalc integration call sites are not migrated and verified in this bead",
            "promotion_consequence": "public consumer-surface migration verification remains unpromoted"
        }),
    ]
}

fn publication_display_rows_w041(
    w073_result: &Value,
    formatting_intake: &Value,
    oxfml_notes: &str,
) -> Vec<Value> {
    let publication_surface = &w073_result["verification_publication_surface"];
    let candidate_result = &w073_result["candidate_result"];
    vec![
        json!({
            "row_id": "publication.w041_w073_typed_only_formatting_guard",
            "disposition_kind": "exercised_current_surface",
            "source": W041_W073_RESULT,
            "boundary_state": "typed_rule_only_guard_exercised_under_w041",
            "typed_rule_family_count": array_len_at(publication_surface, "conditional_formatting_typed_rule_families"),
            "legacy_thresholds_present": array_len_at(publication_surface, "conditional_formatting_thresholds") > 0,
            "typed_only_family_count": array_len_at(formatting_intake, "typed_rule_only_families"),
            "format_delta_present": bool_at(publication_surface, "format_delta_present"),
            "display_delta_present": bool_at(publication_surface, "display_delta_present"),
            "promotion_consequence": "W073 typed_rule-only behavior is carried as current consumed evidence without broad formatting closure"
        }),
        json!({
            "row_id": "publication.w041_w073_no_threshold_fallback_for_aggregate_visualization",
            "disposition_kind": "exercised_current_surface",
            "source": W040_FORMATTING_INTAKE,
            "boundary_state": "direct_replacement_contract_retained",
            "contract_mode": text_at(formatting_intake, "contract_mode"),
            "w072_threshold_fallback_allowed_for_aggregate_visualization": bool_at(
                formatting_intake,
                "w072_threshold_fallback_allowed_for_aggregate_visualization"
            ),
            "threshold_text_family_count": array_len_at(formatting_intake, "threshold_text_families"),
            "promotion_consequence": "OxCalc W041 evidence must not infer fallback from W072 threshold strings for W073 aggregate/visualization families"
        }),
        json!({
            "row_id": "publication.w041_format_delta_display_delta_distinct",
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
            "row_id": "publication.w041_candidate_commit_value_shape_surface",
            "disposition_kind": "exercised_current_surface",
            "source": W041_W073_RESULT,
            "boundary_state": "candidate_commit_value_shape_surface_exercised",
            "candidate_result_id": text_at(candidate_result, "candidate_result_id"),
            "published_value_class": text_at(candidate_result, "published_value_class"),
            "shape_delta_present": bool_at(candidate_result, "shape_delta_present"),
            "commit_kind": text_at(&w073_result["commit_decision"], "kind"),
            "promotion_consequence": "candidate/commit and value/shape publication surfaces are exercised for the W073 fixture only"
        }),
        json!({
            "row_id": "publication.w041_topology_effect_fact_watch",
            "disposition_kind": "note_level_watch",
            "source": OXFML_INBOUND_NOTES,
            "boundary_state": "topology_effect_facts_reachable_but_narrower",
            "topology_fact_note_present": oxfml_notes.contains("topology/effect fact refs"),
            "candidate_result_id_note_present": oxfml_notes.contains("candidate_result_id"),
            "promotion_consequence": "topology/effect fact carriage is recognized without broad topology/publication closure"
        }),
        json!({
            "row_id": "publication.w041_publication_topology_consequence_breadth",
            "disposition_kind": "canonical_but_narrower_watch",
            "source": OXFML_INBOUND_NOTES,
            "boundary_state": "publication_topology_consequence_breadth_narrower",
            "publication_topology_note_present": oxfml_notes.contains("publication and topology consequence breadth"),
            "canonical_but_narrower_present": oxfml_notes.contains("canonical but narrower"),
            "promotion_consequence": "publication/topology consequence breadth remains note-level until direct TreeCalc-facing evidence requires a narrower handoff"
        }),
        json!({
            "row_id": "publication.w041_provider_failure_callable_publication_watch",
            "disposition_kind": "note_level_watch",
            "source": OXFML_INBOUND_NOTES,
            "boundary_state": "provider_failure_callable_publication_watch_lane",
            "provider_failure_note_present": oxfml_notes.contains("provider-failure"),
            "callable_publication_note_present": oxfml_notes.contains("callable-publication"),
            "promotion_consequence": "provider-failure and callable-publication remain watch lanes only until they become coordinator-visible in exercised evidence"
        }),
        json!({
            "row_id": "publication.w041_broad_display_publication_breadth",
            "disposition_kind": "exact_remaining_blocker",
            "source": OXFML_INBOUND_NOTES,
            "boundary_state": "broad_display_publication_unpromoted",
            "reason": "current direct fixture coverage does not exercise broad display-facing categories, broad topology/publication consequences, or all future format/display families",
            "promotion_consequence": "broad OxFml display/publication closure remains unpromoted"
        }),
    ]
}

fn callable_metadata_rows_w041(
    let_lexical_result: &Value,
    returned_lambda_result: &Value,
    obligation_map: &Value,
    implementation_blockers: &Value,
    oxfml_notes: &str,
) -> Vec<Value> {
    vec![
        json!({
            "row_id": "callable.w041_let_lambda_lexical_capture",
            "disposition_kind": "exercised_current_surface",
            "source": W041_LET_LEXICAL_RESULT,
            "callable_state": "narrow_let_lambda_carrier_exercised",
            "function_id_count": array_len_at(&let_lexical_result["evaluation_trace"], "function_ids"),
            "prepared_call_count": number_at(&let_lexical_result["evaluation_trace"], "prepared_call_count"),
            "narrow_let_lambda_carrier": bool_at(&let_lexical_result["w037_interpretation"], "narrow_let_lambda_carrier"),
            "promotion_consequence": "lexical LET/LAMBDA carrier is exercised without general OxFunc kernel promotion"
        }),
        json!({
            "row_id": "callable.w041_returned_lambda_invocation",
            "disposition_kind": "exercised_current_surface",
            "source": W041_RETURNED_LAMBDA_RESULT,
            "callable_state": "returned_lambda_value_carrier_exercised",
            "function_id_count": array_len_at(&returned_lambda_result["evaluation_trace"], "function_ids"),
            "prepared_call_count": number_at(&returned_lambda_result["evaluation_trace"], "prepared_call_count"),
            "payload_summary": text_at(&returned_lambda_result["returned_value_surface"], "payload_summary"),
            "promotion_consequence": "returned-lambda invocation is value-carrier evidence, not metadata projection evidence"
        }),
        json!({
            "row_id": "callable.w041_callable_metadata_projection",
            "disposition_kind": "exact_remaining_blocker",
            "source": W041_IMPLEMENTATION_BLOCKERS,
            "callable_state": "metadata_projection_absent",
            "blocker_present": row_with_field_exists(
                implementation_blockers,
                "row_id",
                "w041_callable_metadata_projection_exact_blocker"
            ),
            "promotion_consequence": "callable metadata projection remains blocked until a projection fixture or carrier sufficiency proof exists"
        }),
        json!({
            "row_id": "callable.w041_carrier_sufficiency_proof",
            "disposition_kind": "exact_remaining_blocker",
            "source": W041_OBLIGATION_MAP,
            "callable_state": "carrier_sufficiency_proof_absent",
            "obligation_present": item_with_field_exists(
                obligation_map,
                "obligations",
                "obligation_id",
                "W041-OBL-023"
            ),
            "promotion_consequence": "callable carrier sufficiency remains unproven for broad metadata projection"
        }),
        json!({
            "row_id": "callable.w041_registered_external_callable_metadata",
            "disposition_kind": "note_level_watch",
            "source": OXFML_INBOUND_NOTES,
            "callable_state": "registered_external_metadata_not_current_projection",
            "registered_external_descriptor_present": oxfml_notes.contains("RegisteredExternalDescriptor"),
            "mutation_request_present": oxfml_notes.contains("RegisteredExternalCatalogMutationRequest"),
            "promotion_consequence": "registered-external packet alignment does not close callable metadata projection in OxCalc"
        }),
        json!({
            "row_id": "callable.w041_registered_external_snapshot_consequence",
            "disposition_kind": "note_level_watch",
            "source": OXFML_INBOUND_NOTES,
            "callable_state": "registered_external_snapshot_invalidation_note_level",
            "snapshot_generation_present": oxfml_notes.contains("new `LibraryContextSnapshot` generation"),
            "bind_invalidation_present": oxfml_notes.contains("bind invalidation"),
            "promotion_consequence": "snapshot and invalidation consequences are note-level convergence, not optimized/core implementation promotion"
        }),
        json!({
            "row_id": "callable.w041_provider_failure_callable_publication_watch",
            "disposition_kind": "note_level_watch",
            "source": OXFML_INBOUND_NOTES,
            "callable_state": "provider_failure_and_callable_publication_watch_only",
            "provider_failure_present": oxfml_notes.contains("provider-failure"),
            "callable_publication_present": oxfml_notes.contains("callable-publication"),
            "promotion_consequence": "provider-failure and callable-publication remain watch lanes until concrete coordinator-visible evidence exists"
        }),
        json!({
            "row_id": "callable.w041_general_oxfunc_kernel_boundary",
            "disposition_kind": "accepted_external_boundary",
            "source": OXFML_INBOUND_NOTES,
            "callable_state": "general_oxfunc_kernel_external",
            "notes_name_oxfunc": oxfml_notes.contains("OxFunc"),
            "promotion_consequence": "OxCalc keeps only the narrow LET/LAMBDA carrier seam in this formalization scope"
        }),
    ]
}

fn registered_external_rows_w041(
    upstream_case_index: &Value,
    obligation_map: &Value,
    oxfml_notes: &str,
) -> Vec<Value> {
    vec![
        json!({
            "row_id": "registered_external.w041_host_provider_fixture_slice",
            "disposition_kind": "exercised_bounded_surface",
            "source": W041_UPSTREAM_HOST_CASE_INDEX,
            "provider_state": "host_info_rtd_provider_slice_exercised",
            "host_info_case_count": tag_count(upstream_case_index, "host-info"),
            "rtd_case_count": tag_count(upstream_case_index, "rtd"),
            "promotion_consequence": "host-provider fixture rows are present, but deferred provider families remain out of scope"
        }),
        json!({
            "row_id": "registered_external.w041_registered_external_optional_standin",
            "disposition_kind": "note_level_watch",
            "source": OXFML_INBOUND_NOTES,
            "provider_state": "registered_external_provider_optional_standin",
            "registered_external_provider_present": oxfml_notes.contains("RegisteredExternalProvider"),
            "host_coordinator_supplied_truth_present": oxfml_notes.contains("host/coordinator-supplied truths"),
            "promotion_consequence": "optional stand-in packet presence does not freeze the production coordinator API"
        }),
        json!({
            "row_id": "registered_external.w041_direct_packet_field_names",
            "disposition_kind": "note_level_watch",
            "source": OXFML_INBOUND_NOTES,
            "provider_state": "direct_packet_field_names_converged_at_note_level",
            "register_id_request_present": oxfml_notes.contains("RegisterIdRequest"),
            "registered_external_call_request_present": oxfml_notes.contains("RegisteredExternalCallRequest"),
            "promotion_consequence": "direct packet naming is note-level aligned and not a shared seam-freeze promotion"
        }),
        json!({
            "row_id": "registered_external.w041_catalog_mutation_funnel",
            "disposition_kind": "note_level_watch",
            "source": OXFML_INBOUND_NOTES,
            "provider_state": "catalog_mutation_funnel_remains_oxfml_owned",
            "catalog_mutation_request_present": oxfml_notes.contains("RegisteredExternalCatalogMutationRequest"),
            "catalog_controller_present": oxfml_notes.contains("RegisteredExternalCatalogController"),
            "promotion_consequence": "host/coordinator registration remains typed mutation requests funneled into OxFunc-owned catalog truth"
        }),
        json!({
            "row_id": "registered_external.w041_provider_failure_watch_lane",
            "disposition_kind": "note_level_watch",
            "source": OXFML_INBOUND_NOTES,
            "provider_state": "provider_failure_watch_lane_only",
            "provider_failure_present": oxfml_notes.contains("provider-failure"),
            "promotion_consequence": "provider-failure remains a watch lane until it becomes coordinator-visible in exercised evidence"
        }),
        json!({
            "row_id": "registered_external.w041_registered_external_projection_blocked",
            "disposition_kind": "exact_remaining_blocker",
            "source": W041_OBLIGATION_MAP,
            "provider_state": "registered_external_callable_projection_not_promoted",
            "obligation_present": item_with_field_exists(
                obligation_map,
                "obligations",
                "obligation_id",
                "W041-OBL-024"
            ),
            "promotion_consequence": "registered-external callable projection remains blocked until concrete OxCalc projection evidence exists"
        }),
    ]
}

fn exact_blockers_w041(
    obligation_map: &Value,
    implementation_blockers: &Value,
    oxfml_notes: &str,
) -> Vec<Value> {
    vec![
        json!({
            "blocker_id": "w041_oxfml.callable_metadata_projection_absent",
            "owner": "calc-sui.8",
            "status_after_run": "exact_remaining_blocker",
            "evidence": W041_IMPLEMENTATION_BLOCKERS,
            "blocker_present": row_with_field_exists(
                implementation_blockers,
                "row_id",
                "w041_callable_metadata_projection_exact_blocker"
            ),
            "reason": "direct LET/LAMBDA value-carrier evidence exists, but callable metadata projection fixture or implementation evidence remains absent.",
            "promotion_consequence": "callable metadata projection remains unpromoted"
        }),
        json!({
            "blocker_id": "w041_oxfml.callable_carrier_sufficiency_proof_absent",
            "owner": "calc-sui.4; calc-sui.8",
            "status_after_run": "exact_remaining_blocker",
            "evidence": W041_OBLIGATION_MAP,
            "obligation_present": item_with_field_exists(
                obligation_map,
                "obligations",
                "obligation_id",
                "W041-OBL-023"
            ),
            "reason": "the consumed LET/LAMBDA carrier rows are exercised, but no proof shows that carrier sufficiency replaces callable metadata projection for the broader consumed surface.",
            "promotion_consequence": "broad callable conformance remains unpromoted"
        }),
        json!({
            "blocker_id": "w041_oxfml.broad_display_publication_breadth_unexercised",
            "owner": "calc-sui.8",
            "status_after_run": "exact_remaining_blocker",
            "evidence": OXFML_INBOUND_NOTES,
            "format_display_notes_present": oxfml_notes.contains("format_delta") && oxfml_notes.contains("display_delta"),
            "reason": "the current direct slice carries typed formatting and distinct format/display categories, but not broad display-facing or publication/topology closure.",
            "promotion_consequence": "broad OxFml display/publication closure remains unpromoted"
        }),
        json!({
            "blocker_id": "w041_oxfml.public_consumer_surface_migration_not_verified",
            "owner": "calc-sui.8; future implementation lane",
            "status_after_run": "exact_remaining_watch_blocker",
            "evidence": OXFML_INBOUND_NOTES,
            "consumer_surface_present": oxfml_notes.contains("consumer::runtime")
                && oxfml_notes.contains("consumer::editor")
                && oxfml_notes.contains("consumer::replay"),
            "reason": "OxFml names the public consumer surface, but this bead does not migrate or verify all OxCalc integration call sites against that surface.",
            "promotion_consequence": "public surface alignment remains watch-bound rather than promoted"
        }),
        json!({
            "blocker_id": "w041_oxfml.registered_external_callable_projection_deferred",
            "owner": "calc-sui.8; external:OxFml/OxFunc",
            "status_after_run": "exact_remaining_watch_blocker",
            "evidence": OXFML_INBOUND_NOTES,
            "registered_external_descriptor_present": oxfml_notes.contains("RegisteredExternalDescriptor"),
            "reason": "registered-external packet naming is note-level converged, but no OxCalc callable metadata projection fixture is exercised.",
            "promotion_consequence": "registered external callable metadata projection remains unpromoted"
        }),
        json!({
            "blocker_id": "w041_oxfml.provider_failure_callable_publication_watch_lane",
            "owner": "calc-sui.8; OxFml watch lane",
            "status_after_run": "exact_remaining_watch_blocker",
            "evidence": OXFML_INBOUND_NOTES,
            "provider_failure_note_present": oxfml_notes.contains("provider-failure"),
            "callable_publication_note_present": oxfml_notes.contains("callable-publication"),
            "reason": "provider-failure and callable-publication are explicitly watch lanes until coordinator-visible evidence exercises them.",
            "promotion_consequence": "provider failure and callable publication seam closure remains unpromoted"
        }),
        json!({
            "blocker_id": "w041_oxfml.publication_topology_consequence_breadth_narrower",
            "owner": "calc-sui.8; W026 residual lane",
            "status_after_run": "exact_remaining_watch_blocker",
            "evidence": OXFML_INBOUND_NOTES,
            "publication_topology_note_present": oxfml_notes.contains("publication and topology consequence breadth"),
            "reason": "publication and topology consequence breadth is canonical but narrower and has not been frozen as broad coordinator seam closure.",
            "promotion_consequence": "broad publication/topology closure remains unpromoted"
        }),
        json!({
            "blocker_id": "w041_oxfml.general_oxfunc_kernel_external",
            "owner": "external:OxFunc",
            "status_after_run": "accepted_external_boundary",
            "evidence": W041_OBLIGATION_MAP,
            "obligation_present": item_with_field_exists(
                obligation_map,
                "obligations",
                "obligation_id",
                "W041-OBL-028"
            ),
            "reason": "W041 includes the narrow LET/LAMBDA carrier seam, not general OxFunc kernel formalization inside OxCalc.",
            "promotion_consequence": "general OxFunc kernel promotion remains outside OxCalc"
        }),
    ]
}

fn source_rows_w040(
    upstream_summary: &Value,
    upstream_case_index: &Value,
    formatting_intake: &Value,
    obligation_map: &Value,
    implementation_blockers: &Value,
    diversity_decision: &Value,
    oxfml_notes: &str,
) -> Vec<Value> {
    vec![
        json!({
            "row_id": "source.w040_upstream_host_direct_oxfml",
            "artifact": W040_UPSTREAM_HOST_SUMMARY,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": number_at(upstream_summary, "expectation_mismatch_count"),
            "failed_row_count": 0,
            "promotion_guard": bool_at(upstream_summary, "all_expectations_matched")
                && number_at(upstream_summary, "direct_oxfml_case_count") >= 3
                && !bool_at(&upstream_summary["promotion_limits"], "general_oxfunc_kernel_claimed")
                && !bool_at(&upstream_summary["promotion_limits"], "pack_grade_replay_promoted")
                && !bool_at(&upstream_summary["promotion_limits"], "c5_promoted"),
            "semantic_state": "fresh_w040_direct_oxfml_runtime_slice_bound"
        }),
        json!({
            "row_id": "source.w040_upstream_host_case_index",
            "artifact": W040_UPSTREAM_HOST_CASE_INDEX,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": 0,
            "promotion_guard": tag_count(upstream_case_index, "direct-oxfml") >= 3
                && tag_count(upstream_case_index, "let-lambda") >= 2
                && tag_count(upstream_case_index, "w073") >= 1
                && tag_count(upstream_case_index, "structured-reference") >= 4
                && tag_count(upstream_case_index, "host-info") >= 2
                && tag_count(upstream_case_index, "rtd") >= 1,
            "semantic_state": "case_index_covers_direct_oxfml_let_lambda_w073_provider_and_table_surfaces"
        }),
        json!({
            "row_id": "source.w040_direct_obligation_map",
            "artifact": W040_OBLIGATION_MAP,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": 0,
            "promotion_guard": number_at(obligation_map, "obligation_count") == 23
                && item_with_field_exists(obligation_map, "obligations", "obligation_id", "W040-OBL-018")
                && item_with_field_exists(obligation_map, "obligations", "obligation_id", "W040-OBL-019")
                && item_with_field_exists(obligation_map, "obligations", "obligation_id", "W040-OBL-020"),
            "semantic_state": "w040_oxfml_and_callable_obligations_bound"
        }),
        json!({
            "row_id": "source.w040_w073_formatting_intake",
            "artifact": W040_FORMATTING_INTAKE,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": 0,
            "promotion_guard": array_len_at(formatting_intake, "typed_rule_only_families") == 7
                && !bool_at(formatting_intake, "w072_threshold_fallback_allowed_for_aggregate_visualization"),
            "semantic_state": "w073_typed_only_formatting_intake_bound_for_w040"
        }),
        json!({
            "row_id": "source.w040_callable_metadata_blocker",
            "artifact": W040_IMPLEMENTATION_BLOCKERS,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": 0,
            "promotion_guard": row_with_field_exists(
                implementation_blockers,
                "row_id",
                "w040_callable_metadata_projection_exact_blocker"
            ),
            "semantic_state": "w040_callable_metadata_exact_blocker_present"
        }),
        json!({
            "row_id": "source.w040_diversity_no_broad_oxfml_promotion",
            "artifact": W040_DIVERSITY_DECISION,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": 0,
            "promotion_guard": !bool_at(diversity_decision, "broad_oxfml_seam_promoted")
                && !bool_at(diversity_decision, "callable_metadata_projection_promoted")
                && !bool_at(diversity_decision, "w073_formatting_handoff_triggered"),
            "semantic_state": "predecessor_w040_diversity_packet_preserves_oxfml_no_promotion"
        }),
        json!({
            "row_id": "source.oxfml_inbound_notes_current_surface",
            "artifact": OXFML_INBOUND_NOTES,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": 0,
            "promotion_guard": oxfml_notes.contains("consumer::runtime")
                && oxfml_notes.contains("consumer::editor")
                && oxfml_notes.contains("consumer::replay")
                && oxfml_notes.contains("format_delta")
                && oxfml_notes.contains("display_delta")
                && oxfml_notes.contains("RegisteredExternalDescriptor"),
            "semantic_state": "current_inbound_oxfml_public_and_callable_notes_surface_present"
        }),
        json!({
            "row_id": "source.oxfml_public_surface_update",
            "artifact": OXFML_INBOUND_NOTES,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": 0,
            "promotion_guard": oxfml_notes.contains("public `substrate::...` access is gone")
                && oxfml_notes.contains("test_support"),
            "semantic_state": "current_public_surface_update_bound_without_oxcalc_migration_claim"
        }),
    ]
}

fn consumed_surface_rows_w040(
    upstream_summary: &Value,
    upstream_case_index: &Value,
    oxfml_notes: &str,
) -> Vec<Value> {
    vec![
        json!({
            "row_id": "surface.w040_direct_oxfml_runtime_facade",
            "disposition_kind": "exercised_current_surface",
            "source": W040_UPSTREAM_HOST_SUMMARY,
            "surface_state": "direct_runtime_facade_exercised_under_w040",
            "direct_oxfml_case_count": number_at(upstream_summary, "direct_oxfml_case_count"),
            "expectation_mismatch_count": number_at(upstream_summary, "expectation_mismatch_count"),
            "promotion_consequence": "current direct OxFml runtime surface is bound for this fixture slice without broad seam closure"
        }),
        json!({
            "row_id": "surface.w040_public_consumer_entry_points",
            "disposition_kind": "note_level_watch",
            "source": OXFML_INBOUND_NOTES,
            "surface_state": "current_public_consumer_surface_named",
            "consumer_runtime_present": oxfml_notes.contains("consumer::runtime"),
            "consumer_editor_present": oxfml_notes.contains("consumer::editor"),
            "consumer_replay_present": oxfml_notes.contains("consumer::replay"),
            "public_substrate_removed": oxfml_notes.contains("public `substrate::...` access is gone"),
            "promotion_consequence": "OxCalc records the current public consumer surface without claiming complete call-site migration or API freeze"
        }),
        json!({
            "row_id": "surface.w040_host_query_and_provider_families",
            "disposition_kind": "exercised_bounded_surface",
            "source": W040_UPSTREAM_HOST_CASE_INDEX,
            "surface_state": "host_info_rtd_reference_families_exercised",
            "host_info_case_count": tag_count(upstream_case_index, "host-info"),
            "rtd_case_count": tag_count(upstream_case_index, "rtd"),
            "bind_context_case_count": tag_count(upstream_case_index, "bind-context"),
            "promotion_consequence": "bounded host-query fixture coverage is present, not full provider-family closure"
        }),
        json!({
            "row_id": "surface.w040_structured_reference_table_context",
            "disposition_kind": "exercised_bounded_surface",
            "source": W040_UPSTREAM_HOST_CASE_INDEX,
            "surface_state": "table_context_and_structured_reference_cases_exercised",
            "table_context_case_count": tag_count(upstream_case_index, "table-context"),
            "structured_reference_case_count": tag_count(upstream_case_index, "structured-reference"),
            "promotion_consequence": "table packet direction is exercised for the bounded fixture slice, not broad workbook table closure"
        }),
        json!({
            "row_id": "surface.w040_registered_external_packet",
            "disposition_kind": "note_level_watch",
            "source": OXFML_INBOUND_NOTES,
            "surface_state": "registered_external_packet_converged_at_note_level",
            "descriptor_present": oxfml_notes.contains("RegisteredExternalDescriptor"),
            "mutation_request_present": oxfml_notes.contains("RegisteredExternalCatalogMutationRequest"),
            "promotion_consequence": "registered external packet naming remains note-level; callable metadata projection is not promoted"
        }),
        json!({
            "row_id": "surface.w040_fixture_host_and_standin_packet",
            "disposition_kind": "note_level_watch",
            "source": OXFML_INBOUND_NOTES,
            "surface_state": "fixture_host_packet_converged_for_deterministic_artifacts",
            "standin_packet_present": oxfml_notes.contains("stand-in packet"),
            "structure_context_identity_present": oxfml_notes.contains("structure-context identity"),
            "promotion_consequence": "fixture-host convergence supports deterministic artifacts but does not freeze the production OxCalc coordinator API"
        }),
    ]
}

fn publication_display_rows_w040(
    w073_result: &Value,
    formatting_intake: &Value,
    oxfml_notes: &str,
) -> Vec<Value> {
    let publication_surface = &w073_result["verification_publication_surface"];
    let candidate_result = &w073_result["candidate_result"];
    vec![
        json!({
            "row_id": "publication.w040_w073_typed_only_formatting_guard",
            "disposition_kind": "exercised_current_surface",
            "source": W040_W073_RESULT,
            "boundary_state": "typed_rule_only_guard_exercised_under_w040",
            "typed_rule_family_count": array_len_at(publication_surface, "conditional_formatting_typed_rule_families"),
            "legacy_thresholds_present": array_len_at(publication_surface, "conditional_formatting_thresholds") > 0,
            "typed_only_family_count": array_len_at(formatting_intake, "typed_rule_only_families"),
            "format_delta_present": bool_at(publication_surface, "format_delta_present"),
            "display_delta_present": bool_at(publication_surface, "display_delta_present"),
            "promotion_consequence": "W073 typed_rule-only behavior is carried as current consumed evidence without broad formatting closure"
        }),
        json!({
            "row_id": "publication.w040_w073_no_threshold_fallback_for_aggregate_visualization",
            "disposition_kind": "exercised_current_surface",
            "source": W040_FORMATTING_INTAKE,
            "boundary_state": "direct_replacement_contract_retained",
            "contract_mode": text_at(formatting_intake, "contract_mode"),
            "w072_threshold_fallback_allowed_for_aggregate_visualization": bool_at(
                formatting_intake,
                "w072_threshold_fallback_allowed_for_aggregate_visualization"
            ),
            "threshold_text_family_count": array_len_at(formatting_intake, "threshold_text_families"),
            "promotion_consequence": "OxCalc W040 evidence must not infer fallback from W072 threshold strings for W073 aggregate/visualization families"
        }),
        json!({
            "row_id": "publication.w040_format_delta_display_delta_distinct",
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
            "row_id": "publication.w040_candidate_commit_value_shape_surface",
            "disposition_kind": "exercised_current_surface",
            "source": W040_W073_RESULT,
            "boundary_state": "candidate_commit_and_shape_surface_exercised",
            "candidate_result_id": text_at(candidate_result, "candidate_result_id"),
            "published_value_class": text_at(candidate_result, "published_value_class"),
            "shape_delta_present": bool_at(candidate_result, "shape_delta_present"),
            "commit_kind": text_at(&w073_result["commit_decision"], "kind"),
            "promotion_consequence": "candidate/commit and value/shape publication surfaces are exercised for the W073 fixture only"
        }),
        json!({
            "row_id": "publication.w040_broad_display_publication_breadth",
            "disposition_kind": "exact_remaining_blocker",
            "source": OXFML_INBOUND_NOTES,
            "boundary_state": "broad_display_publication_unpromoted",
            "reason": "current direct fixture coverage does not exercise broad display-facing categories, broad topology/publication consequences, or all future format/display families",
            "promotion_consequence": "broad OxFml display/publication closure remains unpromoted"
        }),
    ]
}

fn callable_metadata_rows_w040(
    let_lexical_result: &Value,
    returned_lambda_result: &Value,
    obligation_map: &Value,
    implementation_blockers: &Value,
    oxfml_notes: &str,
) -> Vec<Value> {
    vec![
        json!({
            "row_id": "callable.w040_let_lambda_lexical_capture",
            "disposition_kind": "exercised_current_surface",
            "source": W040_LET_LEXICAL_RESULT,
            "callable_state": "narrow_let_lambda_carrier_exercised",
            "function_id_count": array_len_at(&let_lexical_result["evaluation_trace"], "function_ids"),
            "prepared_call_count": number_at(&let_lexical_result["evaluation_trace"], "prepared_call_count"),
            "narrow_let_lambda_carrier": bool_at(&let_lexical_result["w037_interpretation"], "narrow_let_lambda_carrier"),
            "promotion_consequence": "lexical LET/LAMBDA carrier is exercised without general OxFunc kernel promotion"
        }),
        json!({
            "row_id": "callable.w040_returned_lambda_invocation",
            "disposition_kind": "exercised_current_surface",
            "source": W040_RETURNED_LAMBDA_RESULT,
            "callable_state": "returned_lambda_value_carrier_exercised",
            "function_id_count": array_len_at(&returned_lambda_result["evaluation_trace"], "function_ids"),
            "prepared_call_count": number_at(&returned_lambda_result["evaluation_trace"], "prepared_call_count"),
            "payload_summary": text_at(&returned_lambda_result["returned_value_surface"], "payload_summary"),
            "promotion_consequence": "returned-lambda invocation is value-carrier evidence, not metadata projection evidence"
        }),
        json!({
            "row_id": "callable.w040_callable_metadata_projection",
            "disposition_kind": "exact_remaining_blocker",
            "source": W040_IMPLEMENTATION_BLOCKERS,
            "callable_state": "metadata_projection_absent",
            "blocker_present": row_with_field_exists(
                implementation_blockers,
                "row_id",
                "w040_callable_metadata_projection_exact_blocker"
            ),
            "promotion_consequence": "callable metadata projection remains blocked until a projection fixture or carrier sufficiency proof exists"
        }),
        json!({
            "row_id": "callable.w040_carrier_sufficiency_proof",
            "disposition_kind": "exact_remaining_blocker",
            "source": W040_OBLIGATION_MAP,
            "callable_state": "carrier_sufficiency_proof_absent",
            "obligation_present": item_with_field_exists(
                obligation_map,
                "obligations",
                "obligation_id",
                "W040-OBL-004"
            ),
            "promotion_consequence": "callable carrier sufficiency remains unproven for broad metadata projection"
        }),
        json!({
            "row_id": "callable.w040_registered_external_callable_metadata",
            "disposition_kind": "note_level_watch",
            "source": OXFML_INBOUND_NOTES,
            "callable_state": "registered_external_metadata_not_current_projection",
            "registered_external_descriptor_present": oxfml_notes.contains("RegisteredExternalDescriptor"),
            "mutation_request_present": oxfml_notes.contains("RegisteredExternalCatalogMutationRequest"),
            "promotion_consequence": "registered-external packet alignment does not close callable metadata projection in OxCalc"
        }),
        json!({
            "row_id": "callable.w040_general_oxfunc_kernel_boundary",
            "disposition_kind": "accepted_external_boundary",
            "source": OXFML_INBOUND_NOTES,
            "callable_state": "general_oxfunc_kernel_external",
            "notes_name_oxfunc": oxfml_notes.contains("OxFunc"),
            "promotion_consequence": "OxCalc keeps only the narrow LET/LAMBDA carrier seam in this formalization scope"
        }),
    ]
}

fn exact_blockers_w040(
    obligation_map: &Value,
    implementation_blockers: &Value,
    oxfml_notes: &str,
) -> Vec<Value> {
    vec![
        json!({
            "blocker_id": "w040_oxfml.callable_metadata_projection_absent",
            "owner": "calc-tv5.8",
            "status_after_run": "exact_remaining_blocker",
            "evidence": W040_IMPLEMENTATION_BLOCKERS,
            "blocker_present": row_with_field_exists(
                implementation_blockers,
                "row_id",
                "w040_callable_metadata_projection_exact_blocker"
            ),
            "reason": "direct LET/LAMBDA value-carrier evidence exists, but callable metadata projection fixture or implementation evidence remains absent.",
            "promotion_consequence": "callable metadata projection remains unpromoted"
        }),
        json!({
            "blocker_id": "w040_oxfml.callable_carrier_sufficiency_proof_absent",
            "owner": "calc-tv5.8",
            "status_after_run": "exact_remaining_blocker",
            "evidence": W040_OBLIGATION_MAP,
            "obligation_present": item_with_field_exists(
                obligation_map,
                "obligations",
                "obligation_id",
                "W040-OBL-004"
            ),
            "reason": "the consumed LET/LAMBDA carrier rows are exercised, but no proof shows that carrier sufficiency replaces callable metadata projection for the broader consumed surface.",
            "promotion_consequence": "broad callable conformance remains unpromoted"
        }),
        json!({
            "blocker_id": "w040_oxfml.broad_display_publication_breadth_unexercised",
            "owner": "calc-tv5.8",
            "status_after_run": "exact_remaining_blocker",
            "evidence": OXFML_INBOUND_NOTES,
            "format_display_notes_present": oxfml_notes.contains("format_delta") && oxfml_notes.contains("display_delta"),
            "reason": "the current direct slice carries typed formatting and distinct format/display categories, but not broad display-facing or publication/topology closure.",
            "promotion_consequence": "broad OxFml display/publication closure remains unpromoted"
        }),
        json!({
            "blocker_id": "w040_oxfml.public_consumer_surface_migration_not_verified",
            "owner": "calc-tv5.8; future implementation lane",
            "status_after_run": "exact_remaining_watch_blocker",
            "evidence": OXFML_INBOUND_NOTES,
            "consumer_surface_present": oxfml_notes.contains("consumer::runtime")
                && oxfml_notes.contains("consumer::editor")
                && oxfml_notes.contains("consumer::replay"),
            "reason": "OxFml names the public consumer surface, but this bead does not migrate or verify all OxCalc integration call sites against that surface.",
            "promotion_consequence": "public surface alignment remains watch-bound rather than promoted"
        }),
        json!({
            "blocker_id": "w040_oxfml.registered_external_callable_projection_deferred",
            "owner": "calc-tv5.8; external:OxFml/OxFunc",
            "status_after_run": "exact_remaining_watch_blocker",
            "evidence": OXFML_INBOUND_NOTES,
            "registered_external_descriptor_present": oxfml_notes.contains("RegisteredExternalDescriptor"),
            "reason": "registered-external packet naming is note-level converged, but no OxCalc callable metadata projection fixture is exercised.",
            "promotion_consequence": "registered external callable metadata projection remains unpromoted"
        }),
        json!({
            "blocker_id": "w040_oxfml.general_oxfunc_kernel_external",
            "owner": "external:OxFunc",
            "status_after_run": "accepted_external_boundary",
            "evidence": OXFML_INBOUND_NOTES,
            "reason": "W040 includes the narrow LET/LAMBDA carrier seam, not general OxFunc kernel formalization inside OxCalc.",
            "promotion_consequence": "general OxFunc kernel promotion remains outside OxCalc"
        }),
    ]
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

fn oxfml_seam_validation_failures_w040(
    surface_rows: &[Value],
    publication_display_rows: &[Value],
    callable_rows: &[Value],
    blockers: &[Value],
) -> Vec<String> {
    let mut failures = Vec::new();
    if !surface_rows.iter().any(|row| {
        row.get("row_id").and_then(Value::as_str)
            == Some("surface.w040_public_consumer_entry_points")
    }) {
        failures.push("w040_oxfml.public_consumer_surface_row_missing".to_string());
    }
    if !publication_display_rows.iter().any(|row| {
        row.get("row_id").and_then(Value::as_str)
            == Some("publication.w040_w073_no_threshold_fallback_for_aggregate_visualization")
            && row
                .get("w072_threshold_fallback_allowed_for_aggregate_visualization")
                .and_then(Value::as_bool)
                == Some(false)
    }) {
        failures.push("w040_oxfml.w073_no_threshold_fallback_row_missing".to_string());
    }
    if !publication_display_rows.iter().any(|row| {
        row.get("row_id").and_then(Value::as_str)
            == Some("publication.w040_format_delta_display_delta_distinct")
    }) {
        failures.push("w040_oxfml.format_display_boundary_row_missing".to_string());
    }
    if !callable_rows.iter().any(|row| {
        row.get("row_id").and_then(Value::as_str)
            == Some("callable.w040_callable_metadata_projection")
            && row.get("disposition_kind").and_then(Value::as_str)
                == Some("exact_remaining_blocker")
    }) {
        failures.push("w040_oxfml.callable_metadata_blocker_row_missing".to_string());
    }
    if !callable_rows.iter().any(|row| {
        row.get("row_id").and_then(Value::as_str) == Some("callable.w040_carrier_sufficiency_proof")
            && row.get("disposition_kind").and_then(Value::as_str)
                == Some("exact_remaining_blocker")
    }) {
        failures.push("w040_oxfml.carrier_sufficiency_blocker_row_missing".to_string());
    }
    if blockers.len() < 6 {
        failures.push("w040_oxfml.exact_blocker_count_below_gate".to_string());
    }
    if surface_rows
        .iter()
        .chain(publication_display_rows.iter())
        .chain(callable_rows.iter())
        .any(|row| row.get("disposition_kind").and_then(Value::as_str) == Some("promoted"))
    {
        failures.push("w040_oxfml.unexpected_promotion_row".to_string());
    }
    failures
}

fn oxfml_seam_validation_failures_w042(
    surface_rows: &[Value],
    publication_display_rows: &[Value],
    callable_rows: &[Value],
    registered_external_rows: &[Value],
    blockers: &[Value],
) -> Vec<String> {
    let mut failures = Vec::new();
    if !surface_rows.iter().any(|row| {
        row.get("row_id").and_then(Value::as_str)
            == Some("surface.w042_public_consumer_entry_points")
    }) {
        failures.push("w042_oxfml.public_consumer_surface_row_missing".to_string());
    }
    if !surface_rows.iter().any(|row| {
        row.get("row_id").and_then(Value::as_str)
            == Some("surface.w042_public_consumer_migration_verification")
            && row.get("disposition_kind").and_then(Value::as_str)
                == Some("exact_remaining_blocker")
    }) {
        failures.push("w042_oxfml.public_consumer_migration_blocker_row_missing".to_string());
    }
    if !publication_display_rows.iter().any(|row| {
        row.get("row_id").and_then(Value::as_str)
            == Some("publication.w042_w073_no_threshold_fallback_for_aggregate_visualization")
            && row
                .get("threshold_fallback_allowed_for_typed_families")
                .and_then(Value::as_bool)
                == Some(false)
            && row
                .get("old_string_interpretation_allowed")
                .and_then(Value::as_bool)
                == Some(false)
    }) {
        failures.push("w042_oxfml.w073_no_threshold_fallback_row_missing".to_string());
    }
    if !publication_display_rows.iter().any(|row| {
        row.get("row_id").and_then(Value::as_str)
            == Some("publication.w042_format_delta_display_delta_distinct")
            && row
                .get("format_delta_note_present")
                .and_then(Value::as_bool)
                == Some(true)
            && row
                .get("display_delta_note_present")
                .and_then(Value::as_bool)
                == Some(true)
    }) {
        failures.push("w042_oxfml.format_display_boundary_row_missing".to_string());
    }
    if !callable_rows.iter().any(|row| {
        row.get("row_id").and_then(Value::as_str)
            == Some("callable.w042_callable_metadata_projection")
            && row.get("disposition_kind").and_then(Value::as_str)
                == Some("exact_remaining_blocker")
    }) {
        failures.push("w042_oxfml.callable_metadata_blocker_row_missing".to_string());
    }
    if !callable_rows.iter().any(|row| {
        row.get("row_id").and_then(Value::as_str) == Some("callable.w042_carrier_sufficiency_proof")
            && row.get("disposition_kind").and_then(Value::as_str)
                == Some("exact_remaining_blocker")
    }) {
        failures.push("w042_oxfml.carrier_sufficiency_blocker_row_missing".to_string());
    }
    if !registered_external_rows.iter().any(|row| {
        row.get("row_id").and_then(Value::as_str)
            == Some("registered_external.w042_direct_packet_field_names")
            && row
                .get("register_id_request_present")
                .and_then(Value::as_bool)
                == Some(true)
            && row
                .get("registered_external_call_request_present")
                .and_then(Value::as_bool)
                == Some(true)
    }) {
        failures.push("w042_oxfml.registered_external_direct_packet_row_missing".to_string());
    }
    if !registered_external_rows.iter().any(|row| {
        row.get("row_id").and_then(Value::as_str)
            == Some("registered_external.w042_seven_field_descriptor")
            && row
                .get("seven_field_descriptor_present")
                .and_then(Value::as_bool)
                == Some(true)
            && row
                .get("stable_registration_id_present")
                .and_then(Value::as_bool)
                == Some(true)
    }) {
        failures.push("w042_oxfml.registered_external_descriptor_row_missing".to_string());
    }
    if !registered_external_rows.iter().any(|row| {
        row.get("row_id").and_then(Value::as_str)
            == Some("registered_external.w042_snapshot_invalidation_consequences")
            && row
                .get("snapshot_generation_present")
                .and_then(Value::as_bool)
                == Some(true)
            && row
                .get("bind_invalidation_present")
                .and_then(Value::as_bool)
                == Some(true)
    }) {
        failures
            .push("w042_oxfml.registered_external_snapshot_consequence_row_missing".to_string());
    }
    if blockers.len() < 10 {
        failures.push("w042_oxfml.exact_blocker_count_below_gate".to_string());
    }
    if surface_rows
        .iter()
        .chain(publication_display_rows.iter())
        .chain(callable_rows.iter())
        .chain(registered_external_rows.iter())
        .any(|row| row.get("disposition_kind").and_then(Value::as_str) == Some("promoted"))
    {
        failures.push("w042_oxfml.unexpected_promotion_row".to_string());
    }
    failures
}

fn oxfml_seam_validation_failures_w041(
    surface_rows: &[Value],
    publication_display_rows: &[Value],
    callable_rows: &[Value],
    registered_external_rows: &[Value],
    blockers: &[Value],
) -> Vec<String> {
    let mut failures = Vec::new();
    if !surface_rows.iter().any(|row| {
        row.get("row_id").and_then(Value::as_str)
            == Some("surface.w041_public_consumer_entry_points")
    }) {
        failures.push("w041_oxfml.public_consumer_surface_row_missing".to_string());
    }
    if !publication_display_rows.iter().any(|row| {
        row.get("row_id").and_then(Value::as_str)
            == Some("publication.w041_w073_no_threshold_fallback_for_aggregate_visualization")
            && row
                .get("w072_threshold_fallback_allowed_for_aggregate_visualization")
                .and_then(Value::as_bool)
                == Some(false)
    }) {
        failures.push("w041_oxfml.w073_no_threshold_fallback_row_missing".to_string());
    }
    if !publication_display_rows.iter().any(|row| {
        row.get("row_id").and_then(Value::as_str)
            == Some("publication.w041_format_delta_display_delta_distinct")
            && row
                .get("format_delta_note_present")
                .and_then(Value::as_bool)
                == Some(true)
            && row
                .get("display_delta_note_present")
                .and_then(Value::as_bool)
                == Some(true)
    }) {
        failures.push("w041_oxfml.format_display_boundary_row_missing".to_string());
    }
    if !callable_rows.iter().any(|row| {
        row.get("row_id").and_then(Value::as_str)
            == Some("callable.w041_callable_metadata_projection")
            && row.get("disposition_kind").and_then(Value::as_str)
                == Some("exact_remaining_blocker")
    }) {
        failures.push("w041_oxfml.callable_metadata_blocker_row_missing".to_string());
    }
    if !callable_rows.iter().any(|row| {
        row.get("row_id").and_then(Value::as_str) == Some("callable.w041_carrier_sufficiency_proof")
            && row.get("disposition_kind").and_then(Value::as_str)
                == Some("exact_remaining_blocker")
    }) {
        failures.push("w041_oxfml.carrier_sufficiency_blocker_row_missing".to_string());
    }
    if !registered_external_rows.iter().any(|row| {
        row.get("row_id").and_then(Value::as_str)
            == Some("registered_external.w041_direct_packet_field_names")
            && row
                .get("register_id_request_present")
                .and_then(Value::as_bool)
                == Some(true)
            && row
                .get("registered_external_call_request_present")
                .and_then(Value::as_bool)
                == Some(true)
    }) {
        failures.push("w041_oxfml.registered_external_direct_packet_row_missing".to_string());
    }
    if blockers.len() < 8 {
        failures.push("w041_oxfml.exact_blocker_count_below_gate".to_string());
    }
    if surface_rows
        .iter()
        .chain(publication_display_rows.iter())
        .chain(callable_rows.iter())
        .chain(registered_external_rows.iter())
        .any(|row| row.get("disposition_kind").and_then(Value::as_str) == Some("promoted"))
    {
        failures.push("w041_oxfml.unexpected_promotion_row".to_string());
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

fn item_with_field_exists(value: &Value, array_key: &str, field: &str, expected: &str) -> bool {
    value
        .get(array_key)
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

    #[test]
    fn oxfml_seam_runner_binds_w040_consumed_surfaces_without_promotion() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let run_id = format!("test-w040-oxfml-seam-{}", std::process::id());
        let artifact_root =
            repo_root.join(format!("docs/test-runs/core-engine/oxfml-seam/{run_id}"));
        let cleanup = || {
            if artifact_root.exists() {
                let _ = fs::remove_dir_all(&artifact_root);
            }
        };

        cleanup();
        let summary = OxFmlSeamRunner::new().execute(&repo_root, &run_id).unwrap();

        assert_eq!(summary.schema_version, W040_RUN_SUMMARY_SCHEMA_V1);
        assert_eq!(summary.source_evidence_row_count, 8);
        assert_eq!(summary.surface_row_count, 6);
        assert_eq!(summary.publication_display_row_count, 5);
        assert_eq!(summary.callable_metadata_row_count, 6);
        assert_eq!(summary.exact_blocker_count, 6);
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
            "w040_oxfml_seam_breadth_callable_metadata_packet_valid"
        );

        let decision = read_json(
            &repo_root,
            &format!("docs/test-runs/core-engine/oxfml-seam/{run_id}/promotion_decision.json"),
        )
        .unwrap();
        assert_eq!(decision["w073_typed_only_formatting_guard_retained"], true);
        assert_eq!(
            decision["public_consumer_surface_migration_verified"],
            false
        );
        assert_eq!(decision["callable_metadata_projection_promoted"], false);
        assert_eq!(decision["callable_carrier_sufficiency_proven"], false);
        assert_eq!(decision["broad_oxfml_seam_promoted"], false);
        assert_eq!(decision["general_oxfunc_kernel_promoted"], false);

        cleanup();
    }

    #[test]
    fn oxfml_seam_runner_binds_w041_broad_publication_callable_carrier_without_promotion() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let run_id = format!("test-w041-oxfml-seam-{}", std::process::id());
        let artifact_root =
            repo_root.join(format!("docs/test-runs/core-engine/oxfml-seam/{run_id}"));
        let cleanup = || {
            if artifact_root.exists() {
                let _ = fs::remove_dir_all(&artifact_root);
            }
        };

        cleanup();
        let summary = OxFmlSeamRunner::new().execute(&repo_root, &run_id).unwrap();

        assert_eq!(summary.schema_version, W041_RUN_SUMMARY_SCHEMA_V1);
        assert_eq!(summary.source_evidence_row_count, 12);
        assert_eq!(summary.surface_row_count, 10);
        assert_eq!(summary.publication_display_row_count, 8);
        assert_eq!(summary.callable_metadata_row_count, 8);
        assert_eq!(summary.exact_blocker_count, 8);
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
            "w041_oxfml_broad_publication_callable_carrier_packet_valid"
        );
        assert_eq!(validation["registered_external_row_count"], 6);

        let decision = read_json(
            &repo_root,
            &format!("docs/test-runs/core-engine/oxfml-seam/{run_id}/promotion_decision.json"),
        )
        .unwrap();
        assert_eq!(decision["w073_typed_only_formatting_guard_retained"], true);
        assert_eq!(
            decision["public_consumer_surface_migration_verified"],
            false
        );
        assert_eq!(decision["callable_metadata_projection_promoted"], false);
        assert_eq!(decision["callable_carrier_sufficiency_proven"], false);
        assert_eq!(
            decision["registered_external_callable_projection_promoted"],
            false
        );
        assert_eq!(decision["broad_oxfml_seam_promoted"], false);
        assert_eq!(decision["general_oxfunc_kernel_promoted"], false);

        cleanup();
    }

    #[test]
    fn oxfml_seam_runner_binds_w042_public_migration_callable_carrier_without_promotion() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let run_id = format!("test-w042-oxfml-seam-{}", std::process::id());
        let artifact_root =
            repo_root.join(format!("docs/test-runs/core-engine/oxfml-seam/{run_id}"));
        let cleanup = || {
            if artifact_root.exists() {
                let _ = fs::remove_dir_all(&artifact_root);
            }
        };

        cleanup();
        let summary = OxFmlSeamRunner::new().execute(&repo_root, &run_id).unwrap();

        assert_eq!(summary.schema_version, W042_RUN_SUMMARY_SCHEMA_V1);
        assert_eq!(summary.source_evidence_row_count, 17);
        assert_eq!(summary.surface_row_count, 12);
        assert_eq!(summary.publication_display_row_count, 10);
        assert_eq!(summary.callable_metadata_row_count, 10);
        assert_eq!(summary.exact_blocker_count, 10);
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
            "w042_oxfml_public_migration_callable_carrier_packet_valid"
        );
        assert_eq!(validation["registered_external_row_count"], 8);

        let decision = read_json(
            &repo_root,
            &format!("docs/test-runs/core-engine/oxfml-seam/{run_id}/promotion_decision.json"),
        )
        .unwrap();
        assert_eq!(decision["w073_typed_only_formatting_guard_retained"], true);
        assert_eq!(
            decision["public_consumer_surface_migration_verified"],
            false
        );
        assert_eq!(decision["callable_metadata_projection_promoted"], false);
        assert_eq!(decision["callable_carrier_sufficiency_proven"], false);
        assert_eq!(
            decision["registered_external_callable_projection_promoted"],
            false
        );
        assert_eq!(
            decision["provider_failure_callable_publication_promoted"],
            false
        );
        assert_eq!(decision["broad_oxfml_seam_promoted"], false);
        assert_eq!(decision["general_oxfunc_kernel_promoted"], false);

        cleanup();
    }
}
