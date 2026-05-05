#![forbid(unsafe_code)]

//! W038/W039/W040/W041 independent-evaluator diversity and OxFml seam-watch packet emission.

use std::fs;
use std::path::Path;

use serde_json::{Value, json};
use thiserror::Error;

const RUN_SUMMARY_SCHEMA_V1: &str = "oxcalc.diversity_seam.w038.run_summary.v1";
const SOURCE_INDEX_SCHEMA_V1: &str = "oxcalc.diversity_seam.w038.source_evidence_index.v1";
const DIVERSITY_DISPOSITION_SCHEMA_V1: &str =
    "oxcalc.diversity_seam.w038.implementation_diversity_disposition.v1";
const OXFML_SEAM_WATCH_SCHEMA_V1: &str = "oxcalc.diversity_seam.w038.oxfml_seam_watch_packet.v1";
const BLOCKER_REGISTER_SCHEMA_V1: &str =
    "oxcalc.diversity_seam.w038.exact_diversity_seam_blocker_register.v1";
const PROMOTION_DECISION_SCHEMA_V1: &str = "oxcalc.diversity_seam.w038.promotion_decision.v1";
const VALIDATION_SCHEMA_V1: &str = "oxcalc.diversity_seam.w038.validation.v1";
const W039_RUN_SUMMARY_SCHEMA_V1: &str = "oxcalc.diversity_seam.w039.run_summary.v1";
const W039_SOURCE_INDEX_SCHEMA_V1: &str = "oxcalc.diversity_seam.w039.source_evidence_index.v1";
const W039_INDEPENDENT_EVALUATOR_SCHEMA_V1: &str =
    "oxcalc.diversity_seam.w039.independent_evaluator_row_set.v1";
const W039_CROSS_ENGINE_DIVERSITY_SCHEMA_V1: &str =
    "oxcalc.diversity_seam.w039.cross_engine_diversity_register.v1";
const W039_DIFFERENTIAL_AUTHORITY_SCHEMA_V1: &str =
    "oxcalc.diversity_seam.w039.differential_service_authority_register.v1";
const W039_BLOCKER_REGISTER_SCHEMA_V1: &str =
    "oxcalc.diversity_seam.w039.exact_diversity_blocker_register.v1";
const W039_PROMOTION_DECISION_SCHEMA_V1: &str = "oxcalc.diversity_seam.w039.promotion_decision.v1";
const W039_VALIDATION_SCHEMA_V1: &str = "oxcalc.diversity_seam.w039.validation.v1";
const W040_RUN_SUMMARY_SCHEMA_V1: &str = "oxcalc.diversity_seam.w040.run_summary.v1";
const W040_SOURCE_INDEX_SCHEMA_V1: &str = "oxcalc.diversity_seam.w040.source_evidence_index.v1";
const W040_INDEPENDENT_IMPLEMENTATION_SCHEMA_V1: &str =
    "oxcalc.diversity_seam.w040.independent_scalar_evaluator_implementation.v1";
const W040_INDEPENDENT_EVALUATOR_SCHEMA_V1: &str =
    "oxcalc.diversity_seam.w040.independent_evaluator_row_set.v1";
const W040_CROSS_ENGINE_DIVERSITY_SCHEMA_V1: &str =
    "oxcalc.diversity_seam.w040.cross_engine_differential_register.v1";
const W040_DIFFERENTIAL_AUTHORITY_SCHEMA_V1: &str =
    "oxcalc.diversity_seam.w040.differential_authority_register.v1";
const W040_BLOCKER_REGISTER_SCHEMA_V1: &str =
    "oxcalc.diversity_seam.w040.exact_diversity_blocker_register.v1";
const W040_PROMOTION_DECISION_SCHEMA_V1: &str = "oxcalc.diversity_seam.w040.promotion_decision.v1";
const W040_VALIDATION_SCHEMA_V1: &str = "oxcalc.diversity_seam.w040.validation.v1";
const W041_RUN_SUMMARY_SCHEMA_V1: &str = "oxcalc.diversity_seam.w041.run_summary.v1";
const W041_SOURCE_INDEX_SCHEMA_V1: &str = "oxcalc.diversity_seam.w041.source_evidence_index.v1";
const W041_INDEPENDENT_IMPLEMENTATION_SCHEMA_V1: &str =
    "oxcalc.diversity_seam.w041.independent_formula_evaluator_implementation.v1";
const W041_INDEPENDENT_EVALUATOR_SCHEMA_V1: &str =
    "oxcalc.diversity_seam.w041.independent_evaluator_breadth_register.v1";
const W041_CROSS_ENGINE_DIVERSITY_SCHEMA_V1: &str =
    "oxcalc.diversity_seam.w041.cross_engine_differential_service_register.v1";
const W041_MISMATCH_AUTHORITY_SCHEMA_V1: &str =
    "oxcalc.diversity_seam.w041.mismatch_authority_router.v1";
const W041_BLOCKER_REGISTER_SCHEMA_V1: &str =
    "oxcalc.diversity_seam.w041.exact_diversity_blocker_register.v1";
const W041_PROMOTION_DECISION_SCHEMA_V1: &str = "oxcalc.diversity_seam.w041.promotion_decision.v1";
const W041_VALIDATION_SCHEMA_V1: &str = "oxcalc.diversity_seam.w041.validation.v1";

const W036_INDEPENDENT_SUMMARY: &str = "docs/test-runs/core-engine/independent-conformance/w036-independent-diversity-differential-001/run_summary.json";
const W036_CROSS_ENGINE_SUMMARY: &str = "docs/test-runs/core-engine/cross-engine-differential/w036-independent-diversity-differential-001/run_summary.json";
const W037_DIRECT_OXFML_SUMMARY: &str =
    "docs/test-runs/core-engine/upstream-host/w037-direct-oxfml-evaluator-001/run_summary.json";
const W038_CONFORMANCE_SUMMARY: &str = "docs/test-runs/core-engine/implementation-conformance/w038-optimized-core-conformance-disposition-001/run_summary.json";
const W038_CONFORMANCE_BLOCKERS: &str = "docs/test-runs/core-engine/implementation-conformance/w038-optimized-core-conformance-disposition-001/w038_exact_remaining_blocker_register.json";
const W038_FORMAL_ASSURANCE_SUMMARY: &str = "docs/test-runs/core-engine/formal-assurance/w038-proof-model-assumption-discharge-001/run_summary.json";
const W038_STAGE2_REPLAY_SUMMARY: &str =
    "docs/test-runs/core-engine/stage2-replay/w038-stage2-partition-replay-001/run_summary.json";
const W038_OPERATED_ASSURANCE_SUMMARY: &str = "docs/test-runs/core-engine/operated-assurance/w038-operated-assurance-alert-quarantine-001/run_summary.json";
const W038_DIVERSITY_SUMMARY: &str =
    "docs/test-runs/core-engine/diversity-seam/w038-diversity-seam-watch-001/run_summary.json";
const W038_DIVERSITY_DISPOSITION: &str = "docs/test-runs/core-engine/diversity-seam/w038-diversity-seam-watch-001/implementation_diversity_disposition.json";
const W038_DIVERSITY_BLOCKERS: &str = "docs/test-runs/core-engine/diversity-seam/w038-diversity-seam-watch-001/exact_diversity_seam_blocker_register.json";
const W039_RESIDUAL_LEDGER_SUMMARY: &str = "docs/test-runs/core-engine/release-grade-ledger/w039-residual-successor-obligation-ledger-001/run_summary.json";
const W039_FORMATTING_INTAKE: &str = "docs/test-runs/core-engine/release-grade-ledger/w039-residual-successor-obligation-ledger-001/w073_formatting_intake.json";
const W039_CONFORMANCE_SUMMARY: &str = "docs/test-runs/core-engine/implementation-conformance/w039-optimized-core-exact-blocker-disposition-001/run_summary.json";
const W039_PROOF_MODEL_SUMMARY: &str = "docs/test-runs/core-engine/formal-assurance/w039-proof-model-totality-closure-001/run_summary.json";
const W039_STAGE2_SUMMARY: &str = "docs/test-runs/core-engine/stage2-replay/w039-stage2-production-policy-replay-governance-001/run_summary.json";
const W039_STAGE2_BLOCKERS: &str = "docs/test-runs/core-engine/stage2-replay/w039-stage2-production-policy-replay-governance-001/w039_stage2_exact_blocker_register.json";
const W039_OPERATED_ASSURANCE_SUMMARY: &str = "docs/test-runs/core-engine/operated-assurance/w039-operated-assurance-retained-history-001/run_summary.json";
const W039_CROSS_ENGINE_SERVICE_SUBSTRATE: &str = "docs/test-runs/core-engine/operated-assurance/w039-operated-assurance-retained-history-001/w039_cross_engine_service_substrate.json";
const W039_OPERATED_SERVICE_BLOCKERS: &str = "docs/test-runs/core-engine/operated-assurance/w039-operated-assurance-retained-history-001/w039_exact_service_blocker_register.json";
const W039_DIVERSITY_SUMMARY: &str = "docs/test-runs/core-engine/diversity-seam/w039-independent-evaluator-cross-engine-diversity-001/run_summary.json";
const W039_INDEPENDENT_ROW_SET: &str = "docs/test-runs/core-engine/diversity-seam/w039-independent-evaluator-cross-engine-diversity-001/w039_independent_evaluator_row_set.json";
const W039_CROSS_ENGINE_DIVERSITY: &str = "docs/test-runs/core-engine/diversity-seam/w039-independent-evaluator-cross-engine-diversity-001/w039_cross_engine_diversity_register.json";
const W039_DIFFERENTIAL_AUTHORITY: &str = "docs/test-runs/core-engine/diversity-seam/w039-independent-evaluator-cross-engine-diversity-001/w039_differential_service_authority_register.json";
const W039_DIVERSITY_BLOCKERS: &str = "docs/test-runs/core-engine/diversity-seam/w039-independent-evaluator-cross-engine-diversity-001/w039_exact_diversity_blocker_register.json";
const W039_DIVERSITY_DECISION: &str = "docs/test-runs/core-engine/diversity-seam/w039-independent-evaluator-cross-engine-diversity-001/promotion_decision.json";
const W040_DIRECT_OBLIGATION_SUMMARY: &str = "docs/test-runs/core-engine/release-grade-ledger/w040-direct-verification-obligation-map-001/run_summary.json";
const W040_DIRECT_OBLIGATION_MAP: &str = "docs/test-runs/core-engine/release-grade-ledger/w040-direct-verification-obligation-map-001/direct_verification_obligation_map.json";
const W040_FORMATTING_INTAKE: &str = "docs/test-runs/core-engine/release-grade-ledger/w040-direct-verification-obligation-map-001/w073_formatting_intake.json";
const W040_CONFORMANCE_SUMMARY: &str = "docs/test-runs/core-engine/implementation-conformance/w040-optimized-core-exact-blocker-fixes-differentials-001/run_summary.json";
const W040_TREECALC_SUMMARY: &str = "docs/test-runs/core-engine/treecalc-local/w040-optimized-core-dynamic-release-reclassification-001/run_summary.json";
const W040_RUST_TOTALITY_SUMMARY: &str = "docs/test-runs/core-engine/formal-assurance/w040-rust-totality-refinement-proof-tranche-001/run_summary.json";
const W040_LEAN_TLA_SUMMARY: &str = "docs/test-runs/core-engine/formal-assurance/w040-lean-tla-full-verification-discharge-001/run_summary.json";
const W040_STAGE2_SUMMARY: &str = "docs/test-runs/core-engine/stage2-replay/w040-stage2-production-policy-equivalence-001/run_summary.json";
const W040_STAGE2_BLOCKERS: &str = "docs/test-runs/core-engine/stage2-replay/w040-stage2-production-policy-equivalence-001/w040_stage2_exact_blocker_register.json";
const W040_OPERATED_ASSURANCE_SUMMARY: &str = "docs/test-runs/core-engine/operated-assurance/w040-operated-assurance-retained-history-service-001/run_summary.json";
const W040_RETAINED_HISTORY_STORE: &str = "docs/test-runs/core-engine/operated-assurance/w040-operated-assurance-retained-history-service-001/w040_retained_history_store_query.json";
const W040_CROSS_ENGINE_SERVICE_REGISTER: &str = "docs/test-runs/core-engine/operated-assurance/w040-operated-assurance-retained-history-service-001/w040_cross_engine_service_register.json";
const W040_OPERATED_SERVICE_BLOCKERS: &str = "docs/test-runs/core-engine/operated-assurance/w040-operated-assurance-retained-history-service-001/w040_exact_service_blocker_register.json";
const W040_DIVERSITY_SUMMARY: &str = "docs/test-runs/core-engine/diversity-seam/w040-independent-evaluator-operated-differential-001/run_summary.json";
const W040_INDEPENDENT_IMPLEMENTATION: &str = "docs/test-runs/core-engine/diversity-seam/w040-independent-evaluator-operated-differential-001/w040_independent_scalar_evaluator_implementation.json";
const W040_INDEPENDENT_ROW_SET: &str = "docs/test-runs/core-engine/diversity-seam/w040-independent-evaluator-operated-differential-001/w040_independent_evaluator_row_set.json";
const W040_CROSS_ENGINE_DIVERSITY: &str = "docs/test-runs/core-engine/diversity-seam/w040-independent-evaluator-operated-differential-001/w040_cross_engine_differential_register.json";
const W040_DIFFERENTIAL_AUTHORITY: &str = "docs/test-runs/core-engine/diversity-seam/w040-independent-evaluator-operated-differential-001/w040_differential_authority_register.json";
const W040_DIVERSITY_BLOCKERS: &str = "docs/test-runs/core-engine/diversity-seam/w040-independent-evaluator-operated-differential-001/w040_exact_diversity_blocker_register.json";
const W040_DIVERSITY_DECISION: &str = "docs/test-runs/core-engine/diversity-seam/w040-independent-evaluator-operated-differential-001/promotion_decision.json";
const W041_OBLIGATION_SUMMARY: &str = "docs/test-runs/core-engine/release-grade-ledger/w041-residual-release-grade-successor-obligation-map-001/run_summary.json";
const W041_OBLIGATION_MAP: &str = "docs/test-runs/core-engine/release-grade-ledger/w041-residual-release-grade-successor-obligation-map-001/successor_obligation_map.json";
const W041_CONFORMANCE_SUMMARY: &str = "docs/test-runs/core-engine/implementation-conformance/w041-optimized-core-residual-blocker-differentials-001/run_summary.json";
const W041_TREECALC_SUMMARY: &str = "docs/test-runs/core-engine/treecalc-local/w041-optimized-core-automatic-dynamic-transition-001/run_summary.json";
const W041_RUST_TOTALITY_SUMMARY: &str = "docs/test-runs/core-engine/formal-assurance/w041-rust-totality-refinement-proof-tranche-001/run_summary.json";
const W041_LEAN_TLA_SUMMARY: &str = "docs/test-runs/core-engine/formal-assurance/w041-lean-tla-full-verification-fairness-discharge-001/run_summary.json";
const W041_STAGE2_SUMMARY: &str = "docs/test-runs/core-engine/stage2-replay/w041-stage2-production-analyzer-pack-equivalence-001/run_summary.json";
const W041_STAGE2_BLOCKERS: &str = "docs/test-runs/core-engine/stage2-replay/w041-stage2-production-analyzer-pack-equivalence-001/w041_stage2_exact_blocker_register.json";
const W041_OPERATED_ASSURANCE_SUMMARY: &str = "docs/test-runs/core-engine/operated-assurance/w041-operated-assurance-retained-history-alert-dispatch-001/run_summary.json";
const W041_RETAINED_HISTORY_QUERY: &str = "docs/test-runs/core-engine/operated-assurance/w041-operated-assurance-retained-history-alert-dispatch-001/w041_retained_history_service_query.json";
const W041_CROSS_ENGINE_SERVICE_REGISTER: &str = "docs/test-runs/core-engine/operated-assurance/w041-operated-assurance-retained-history-alert-dispatch-001/w041_cross_engine_service_register.json";
const W041_ALERT_DISPATCH_REGISTER: &str = "docs/test-runs/core-engine/operated-assurance/w041-operated-assurance-retained-history-alert-dispatch-001/w041_alert_dispatch_service_register.json";
const W041_OPERATED_SERVICE_BLOCKERS: &str = "docs/test-runs/core-engine/operated-assurance/w041-operated-assurance-retained-history-alert-dispatch-001/w041_exact_service_blocker_register.json";
const OXFML_INBOUND_NOTES: &str = "../OxFml/docs/upstream/NOTES_FOR_OXCALC.md";

#[derive(Debug, Error)]
pub enum DiversitySeamError {
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
pub struct DiversitySeamRunSummary {
    pub run_id: String,
    pub schema_version: String,
    pub source_evidence_row_count: usize,
    pub diversity_disposition_row_count: usize,
    pub seam_watch_row_count: usize,
    pub aligned_seam_watch_row_count: usize,
    pub accepted_boundary_count: usize,
    pub exact_blocker_count: usize,
    pub failed_row_count: usize,
    pub fully_independent_evaluator_promoted: bool,
    pub artifact_root: String,
}

#[derive(Debug, Clone, Default)]
pub struct DiversitySeamRunner;

impl DiversitySeamRunner {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    pub fn execute(
        &self,
        repo_root: &Path,
        run_id: &str,
    ) -> Result<DiversitySeamRunSummary, DiversitySeamError> {
        if run_id.starts_with("w041-") || run_id.starts_with("test-w041-") {
            return self.execute_w041(repo_root, run_id);
        }
        if run_id.starts_with("w040-") || run_id.starts_with("test-w040-") {
            return self.execute_w040(repo_root, run_id);
        }
        if run_id.starts_with("w039-") || run_id.starts_with("test-w039-") {
            return self.execute_w039(repo_root, run_id);
        }

        let relative_artifact_root =
            relative_artifact_path(&["docs", "test-runs", "core-engine", "diversity-seam", run_id]);
        let artifact_root = repo_root.join(&relative_artifact_root);
        if artifact_root.exists() {
            fs::remove_dir_all(&artifact_root).map_err(|source| {
                DiversitySeamError::RemoveDirectory {
                    path: artifact_root.display().to_string(),
                    source,
                }
            })?;
        }
        fs::create_dir_all(&artifact_root).map_err(|source| {
            DiversitySeamError::CreateDirectory {
                path: artifact_root.display().to_string(),
                source,
            }
        })?;

        let w036_independent = read_json(repo_root, W036_INDEPENDENT_SUMMARY)?;
        let w036_cross_engine = read_json(repo_root, W036_CROSS_ENGINE_SUMMARY)?;
        let w037_direct_oxfml = read_json(repo_root, W037_DIRECT_OXFML_SUMMARY)?;
        let w038_conformance = read_json(repo_root, W038_CONFORMANCE_SUMMARY)?;
        let w038_conformance_blockers = read_json(repo_root, W038_CONFORMANCE_BLOCKERS)?;
        let w038_formal = read_json(repo_root, W038_FORMAL_ASSURANCE_SUMMARY)?;
        let w038_stage2 = read_json(repo_root, W038_STAGE2_REPLAY_SUMMARY)?;
        let w038_operated = read_json(repo_root, W038_OPERATED_ASSURANCE_SUMMARY)?;

        let source_rows = source_rows(
            &w036_independent,
            &w036_cross_engine,
            &w037_direct_oxfml,
            &w038_conformance,
            &w038_formal,
            &w038_stage2,
            &w038_operated,
        );
        let diversity_rows = diversity_rows(&w036_independent, &w036_cross_engine);
        let seam_watch_rows = seam_watch_rows(
            repo_root,
            &w037_direct_oxfml,
            &w038_conformance,
            &w038_conformance_blockers,
            &w038_formal,
            &w038_stage2,
        );
        let blockers = exact_blockers();
        let source_failures = source_validation_failures(&source_rows);
        let seam_failures = seam_validation_failures(&seam_watch_rows);
        let failed_row_count = source_failures.len() + seam_failures.len();
        let accepted_boundary_count = diversity_rows
            .iter()
            .chain(seam_watch_rows.iter())
            .filter(|row| {
                row.get("disposition_kind").and_then(Value::as_str) == Some("accepted_boundary")
            })
            .count();
        let aligned_seam_watch_row_count = seam_watch_rows
            .iter()
            .filter(|row| row.get("watch_state").and_then(Value::as_str) == Some("aligned"))
            .count();

        let source_evidence_index_path =
            format!("{relative_artifact_root}/source_evidence_index.json");
        let diversity_disposition_path =
            format!("{relative_artifact_root}/implementation_diversity_disposition.json");
        let seam_watch_path = format!("{relative_artifact_root}/oxfml_seam_watch_packet.json");
        let blocker_register_path =
            format!("{relative_artifact_root}/exact_diversity_seam_blocker_register.json");
        let promotion_decision_path = format!("{relative_artifact_root}/promotion_decision.json");
        let validation_path = format!("{relative_artifact_root}/validation.json");

        let source_evidence_index = json!({
            "schema_version": SOURCE_INDEX_SCHEMA_V1,
            "run_id": run_id,
            "artifact_root": relative_artifact_root,
            "source_evidence_row_count": source_rows.len(),
            "rows": source_rows,
            "source_artifacts": {
                "w036_independent_summary": W036_INDEPENDENT_SUMMARY,
                "w036_cross_engine_summary": W036_CROSS_ENGINE_SUMMARY,
                "w037_direct_oxfml_summary": W037_DIRECT_OXFML_SUMMARY,
                "w038_conformance_summary": W038_CONFORMANCE_SUMMARY,
                "w038_conformance_blockers": W038_CONFORMANCE_BLOCKERS,
                "w038_formal_assurance_summary": W038_FORMAL_ASSURANCE_SUMMARY,
                "w038_stage2_replay_summary": W038_STAGE2_REPLAY_SUMMARY,
                "w038_operated_assurance_summary": W038_OPERATED_ASSURANCE_SUMMARY,
                "oxfml_inbound_notes": OXFML_INBOUND_NOTES
            }
        });
        let diversity_disposition = json!({
            "schema_version": DIVERSITY_DISPOSITION_SCHEMA_V1,
            "run_id": run_id,
            "row_count": diversity_rows.len(),
            "fully_independent_evaluator_promoted": false,
            "rows": diversity_rows
        });
        let seam_watch = json!({
            "schema_version": OXFML_SEAM_WATCH_SCHEMA_V1,
            "run_id": run_id,
            "row_count": seam_watch_rows.len(),
            "aligned_row_count": aligned_seam_watch_row_count,
            "handoff_triggered": false,
            "rows": seam_watch_rows
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
            "decision_state": "w038_diversity_and_oxfml_seam_watch_bound_without_full_independence_promotion",
            "fully_independent_evaluator_promoted": false,
            "general_oxfunc_kernel_promoted": false,
            "callable_metadata_projection_promoted": false,
            "oxfml_handoff_triggered": false,
            "pack_grade_replay_promoted": false,
            "c5_promoted": false,
            "stage2_policy_promoted": false,
            "diversity_disposition_row_count": diversity_rows.len(),
            "seam_watch_row_count": seam_watch_rows.len(),
            "accepted_boundary_count": accepted_boundary_count,
            "exact_blocker_count": blockers.len(),
            "blockers": blockers
                .iter()
                .map(|row| row["blocker_id"].clone())
                .collect::<Vec<_>>(),
            "semantic_equivalence_statement": "This runner classifies implementation diversity and OxFml seam-watch evidence only. It does not change evaluator kernels, coordinator scheduling, recalc, publication, replay, pack, service, TraceCalc, TreeCalc, OxFml, OxFunc, Lean, or TLA semantics."
        });
        let mut validation_failures = source_failures;
        validation_failures.extend(seam_failures);
        let validation = json!({
            "schema_version": VALIDATION_SCHEMA_V1,
            "run_id": run_id,
            "status": if validation_failures.is_empty() {
                "w038_diversity_seam_packet_valid"
            } else {
                "w038_diversity_seam_packet_invalid"
            },
            "source_evidence_row_count": source_rows.len(),
            "diversity_disposition_row_count": diversity_rows.len(),
            "seam_watch_row_count": seam_watch_rows.len(),
            "aligned_seam_watch_row_count": aligned_seam_watch_row_count,
            "accepted_boundary_count": accepted_boundary_count,
            "exact_blocker_count": blockers.len(),
            "failed_row_count": failed_row_count,
            "fully_independent_evaluator_promoted": false,
            "validation_failures": validation_failures
        });
        let run_summary = json!({
            "schema_version": RUN_SUMMARY_SCHEMA_V1,
            "run_id": run_id,
            "artifact_root": relative_artifact_root,
            "source_evidence_index_path": source_evidence_index_path,
            "implementation_diversity_disposition_path": diversity_disposition_path,
            "oxfml_seam_watch_packet_path": seam_watch_path,
            "exact_diversity_seam_blocker_register_path": blocker_register_path,
            "promotion_decision_path": promotion_decision_path,
            "validation_path": validation_path,
            "source_evidence_row_count": source_rows.len(),
            "diversity_disposition_row_count": diversity_rows.len(),
            "seam_watch_row_count": seam_watch_rows.len(),
            "aligned_seam_watch_row_count": aligned_seam_watch_row_count,
            "accepted_boundary_count": accepted_boundary_count,
            "exact_blocker_count": blockers.len(),
            "failed_row_count": failed_row_count,
            "fully_independent_evaluator_promoted": false,
            "oxfml_handoff_triggered": false
        });

        write_json(
            &artifact_root.join("source_evidence_index.json"),
            &source_evidence_index,
        )?;
        write_json(
            &artifact_root.join("implementation_diversity_disposition.json"),
            &diversity_disposition,
        )?;
        write_json(
            &artifact_root.join("oxfml_seam_watch_packet.json"),
            &seam_watch,
        )?;
        write_json(
            &artifact_root.join("exact_diversity_seam_blocker_register.json"),
            &blocker_register,
        )?;
        write_json(
            &artifact_root.join("promotion_decision.json"),
            &promotion_decision,
        )?;
        write_json(&artifact_root.join("validation.json"), &validation)?;
        write_json(&artifact_root.join("run_summary.json"), &run_summary)?;

        Ok(DiversitySeamRunSummary {
            run_id: run_id.to_string(),
            schema_version: RUN_SUMMARY_SCHEMA_V1.to_string(),
            source_evidence_row_count: source_rows.len(),
            diversity_disposition_row_count: diversity_rows.len(),
            seam_watch_row_count: seam_watch_rows.len(),
            aligned_seam_watch_row_count,
            accepted_boundary_count,
            exact_blocker_count: blockers.len(),
            failed_row_count,
            fully_independent_evaluator_promoted: false,
            artifact_root: relative_artifact_root,
        })
    }

    fn execute_w041(
        &self,
        repo_root: &Path,
        run_id: &str,
    ) -> Result<DiversitySeamRunSummary, DiversitySeamError> {
        let relative_artifact_root =
            relative_artifact_path(&["docs", "test-runs", "core-engine", "diversity-seam", run_id]);
        let artifact_root = repo_root.join(&relative_artifact_root);
        if artifact_root.exists() {
            fs::remove_dir_all(&artifact_root).map_err(|source| {
                DiversitySeamError::RemoveDirectory {
                    path: artifact_root.display().to_string(),
                    source,
                }
            })?;
        }
        fs::create_dir_all(&artifact_root).map_err(|source| {
            DiversitySeamError::CreateDirectory {
                path: artifact_root.display().to_string(),
                source,
            }
        })?;

        let w041_obligation_summary = read_json(repo_root, W041_OBLIGATION_SUMMARY)?;
        let w041_obligation_map = read_json(repo_root, W041_OBLIGATION_MAP)?;
        let w040_formatting_intake = read_json(repo_root, W040_FORMATTING_INTAKE)?;
        let w040_diversity = read_json(repo_root, W040_DIVERSITY_SUMMARY)?;
        let w040_implementation = read_json(repo_root, W040_INDEPENDENT_IMPLEMENTATION)?;
        let w040_independent = read_json(repo_root, W040_INDEPENDENT_ROW_SET)?;
        let w040_cross_engine = read_json(repo_root, W040_CROSS_ENGINE_DIVERSITY)?;
        let w040_authority = read_json(repo_root, W040_DIFFERENTIAL_AUTHORITY)?;
        let w040_blockers = read_json(repo_root, W040_DIVERSITY_BLOCKERS)?;
        let w040_decision = read_json(repo_root, W040_DIVERSITY_DECISION)?;
        let w041_conformance = read_json(repo_root, W041_CONFORMANCE_SUMMARY)?;
        let w041_treecalc = read_json(repo_root, W041_TREECALC_SUMMARY)?;
        let w041_rust = read_json(repo_root, W041_RUST_TOTALITY_SUMMARY)?;
        let w041_lean_tla = read_json(repo_root, W041_LEAN_TLA_SUMMARY)?;
        let w041_stage2 = read_json(repo_root, W041_STAGE2_SUMMARY)?;
        let w041_stage2_blockers = read_json(repo_root, W041_STAGE2_BLOCKERS)?;
        let w041_operated = read_json(repo_root, W041_OPERATED_ASSURANCE_SUMMARY)?;
        let w041_retained = read_json(repo_root, W041_RETAINED_HISTORY_QUERY)?;
        let w041_cross_engine_service = read_json(repo_root, W041_CROSS_ENGINE_SERVICE_REGISTER)?;
        let w041_alert_dispatch = read_json(repo_root, W041_ALERT_DISPATCH_REGISTER)?;
        let w041_service_blockers = read_json(repo_root, W041_OPERATED_SERVICE_BLOCKERS)?;

        let implementation_rows = w041_independent_formula_evaluator_rows();
        let source_rows = w041_source_rows(
            &w041_obligation_summary,
            &w041_obligation_map,
            &w040_formatting_intake,
            &w040_diversity,
            &w040_implementation,
            &w040_decision,
            &w041_conformance,
            &w041_treecalc,
            &w041_rust,
            &w041_lean_tla,
            &w041_stage2,
            &w041_operated,
            &w041_cross_engine_service,
            &w041_alert_dispatch,
        );
        let independent_rows = w041_independent_evaluator_rows(
            &w040_independent,
            &w040_diversity,
            &w041_conformance,
            &w041_treecalc,
            &w041_rust,
            &w041_lean_tla,
            &w041_stage2,
            &w041_operated,
            &implementation_rows,
        );
        let cross_engine_rows = w041_cross_engine_differential_rows(
            &w040_cross_engine,
            &w041_stage2,
            &w041_operated,
            &w041_retained,
            &w041_cross_engine_service,
            &w041_alert_dispatch,
            &w040_formatting_intake,
            &implementation_rows,
        );
        let authority_rows = w041_mismatch_authority_rows(
            &w040_authority,
            &w041_conformance,
            &w041_operated,
            &w041_cross_engine_service,
            &w041_alert_dispatch,
            &implementation_rows,
        );
        let blockers = w041_exact_blockers(
            &w040_blockers,
            &w041_stage2_blockers,
            &w041_service_blockers,
            &w041_cross_engine_service,
        );
        let accepted_boundary_count = independent_rows
            .iter()
            .chain(cross_engine_rows.iter())
            .chain(authority_rows.iter())
            .filter(|row| {
                matches!(
                    row.get("disposition_kind").and_then(Value::as_str),
                    Some("accepted_boundary")
                        | Some("accepted_external_slice")
                        | Some("broadened_independent_implementation")
                        | Some("service_contract_boundary")
                )
            })
            .count();
        let service_blocked_count = cross_engine_rows
            .iter()
            .chain(authority_rows.iter())
            .filter(|row| {
                row.get("service_state").and_then(Value::as_str) == Some("blocked")
                    || row
                        .get("authority_state")
                        .and_then(Value::as_str)
                        .is_some_and(|state| state.starts_with("blocked"))
            })
            .count();

        let mut validation_failures = source_validation_failures(&source_rows);
        validation_failures.extend(w041_diversity_validation_failures(
            &implementation_rows,
            &independent_rows,
            &cross_engine_rows,
            &authority_rows,
            &blockers,
        ));
        let failed_row_count = validation_failures.len();
        let implementation_case_count = implementation_rows.len();
        let implementation_match_count = implementation_rows
            .iter()
            .filter(|row| bool_at(row, "matches_expected"))
            .count();

        let source_evidence_index_path =
            format!("{relative_artifact_root}/source_evidence_index.json");
        let independent_implementation_path = format!(
            "{relative_artifact_root}/w041_independent_formula_evaluator_implementation.json"
        );
        let independent_row_set_path =
            format!("{relative_artifact_root}/w041_independent_evaluator_breadth_register.json");
        let cross_engine_path = format!(
            "{relative_artifact_root}/w041_cross_engine_differential_service_register.json"
        );
        let mismatch_authority_path =
            format!("{relative_artifact_root}/w041_mismatch_authority_router.json");
        let blocker_register_path =
            format!("{relative_artifact_root}/w041_exact_diversity_blocker_register.json");
        let promotion_decision_path = format!("{relative_artifact_root}/promotion_decision.json");
        let validation_path = format!("{relative_artifact_root}/validation.json");

        let source_evidence_index = json!({
            "schema_version": W041_SOURCE_INDEX_SCHEMA_V1,
            "run_id": run_id,
            "artifact_root": relative_artifact_root,
            "source_evidence_row_count": source_rows.len(),
            "rows": source_rows,
            "source_artifacts": {
                "w041_obligation_summary": W041_OBLIGATION_SUMMARY,
                "w041_obligation_map": W041_OBLIGATION_MAP,
                "w040_formatting_intake": W040_FORMATTING_INTAKE,
                "w040_diversity_summary": W040_DIVERSITY_SUMMARY,
                "w040_independent_implementation": W040_INDEPENDENT_IMPLEMENTATION,
                "w040_independent_row_set": W040_INDEPENDENT_ROW_SET,
                "w040_cross_engine_differential": W040_CROSS_ENGINE_DIVERSITY,
                "w040_differential_authority": W040_DIFFERENTIAL_AUTHORITY,
                "w040_diversity_blockers": W040_DIVERSITY_BLOCKERS,
                "w040_diversity_decision": W040_DIVERSITY_DECISION,
                "w041_conformance_summary": W041_CONFORMANCE_SUMMARY,
                "w041_treecalc_summary": W041_TREECALC_SUMMARY,
                "w041_rust_totality_summary": W041_RUST_TOTALITY_SUMMARY,
                "w041_lean_tla_summary": W041_LEAN_TLA_SUMMARY,
                "w041_stage2_summary": W041_STAGE2_SUMMARY,
                "w041_stage2_blockers": W041_STAGE2_BLOCKERS,
                "w041_operated_assurance_summary": W041_OPERATED_ASSURANCE_SUMMARY,
                "w041_retained_history_query": W041_RETAINED_HISTORY_QUERY,
                "w041_cross_engine_service_register": W041_CROSS_ENGINE_SERVICE_REGISTER,
                "w041_alert_dispatch_register": W041_ALERT_DISPATCH_REGISTER,
                "w041_operated_service_blockers": W041_OPERATED_SERVICE_BLOCKERS,
                "oxfml_inbound_notes": OXFML_INBOUND_NOTES
            }
        });
        let independent_implementation = json!({
            "schema_version": W041_INDEPENDENT_IMPLEMENTATION_SCHEMA_V1,
            "run_id": run_id,
            "implementation_kind": "broadened_independent_formula_fragment_evaluator",
            "implementation_authority": "local_w041_diversity_runner_independent_from_tracecalc_optimized_core_oxfml_and_oxfunc",
            "case_count": implementation_case_count,
            "match_count": implementation_match_count,
            "failed_case_count": implementation_case_count - implementation_match_count,
            "fully_independent_evaluator_promoted": false,
            "scope_limit": "integer names and literals with +, -, *, /, parentheses, comparisons, and IF(condition, true_expr, false_expr)",
            "rows": implementation_rows
        });
        let independent_row_set = json!({
            "schema_version": W041_INDEPENDENT_EVALUATOR_SCHEMA_V1,
            "run_id": run_id,
            "row_count": independent_rows.len(),
            "broadened_independent_implementation_case_count": implementation_case_count,
            "broadened_independent_implementation_match_count": implementation_match_count,
            "fully_independent_evaluator_promoted": false,
            "rows": independent_rows
        });
        let cross_engine = json!({
            "schema_version": W041_CROSS_ENGINE_DIVERSITY_SCHEMA_V1,
            "run_id": run_id,
            "row_count": cross_engine_rows.len(),
            "service_blocked_count": service_blocked_count,
            "broadened_independent_implementation_case_count": implementation_case_count,
            "operated_cross_engine_differential_service_promoted": false,
            "w073_typed_only_formatting_guard_retained": true,
            "rows": cross_engine_rows
        });
        let mismatch_authority = json!({
            "schema_version": W041_MISMATCH_AUTHORITY_SCHEMA_V1,
            "run_id": run_id,
            "row_count": authority_rows.len(),
            "accepted_boundary_count": accepted_boundary_count,
            "broadened_independent_implementation_case_count": implementation_case_count,
            "operated_service_authority_promoted": false,
            "fully_independent_evaluator_promoted": false,
            "rows": authority_rows
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
            "decision_state": "w041_broadened_independent_formula_fragment_bound_without_diversity_or_service_promotion",
            "broadened_independent_formula_evaluator_present": true,
            "broadened_independent_implementation_case_count": implementation_case_count,
            "broadened_independent_implementation_match_count": implementation_match_count,
            "fully_independent_evaluator_promoted": false,
            "independent_evaluator_breadth_promoted": false,
            "operated_cross_engine_differential_service_promoted": false,
            "cross_engine_diversity_service_promoted": false,
            "mismatch_quarantine_service_promoted": false,
            "broad_oxfml_seam_promoted": false,
            "callable_metadata_projection_promoted": false,
            "w073_formatting_handoff_triggered": false,
            "pack_grade_replay_promoted": false,
            "c5_promoted": false,
            "stage2_policy_promoted": false,
            "source_evidence_row_count": source_rows.len(),
            "independent_evaluator_row_count": independent_rows.len(),
            "cross_engine_differential_row_count": cross_engine_rows.len(),
            "mismatch_authority_row_count": authority_rows.len(),
            "accepted_boundary_count": accepted_boundary_count,
            "service_blocked_count": service_blocked_count,
            "exact_blocker_count": blockers.len(),
            "blockers": blockers
                .iter()
                .map(|row| row["blocker_id"].clone())
                .collect::<Vec<_>>(),
            "semantic_equivalence_statement": "This W041 diversity runner adds a broadened independent formula-fragment evaluator and classifies diversity, differential service, and mismatch-authority evidence only. It does not change evaluator kernels used by OxCalc, coordinator scheduling, recalc, publication, replay, pack, service, TraceCalc, TreeCalc, OxFml, OxFunc, Lean, or TLA semantics."
        });
        let validation = json!({
            "schema_version": W041_VALIDATION_SCHEMA_V1,
            "run_id": run_id,
            "status": if validation_failures.is_empty() {
                "w041_broadened_independent_evaluator_diversity_packet_valid"
            } else {
                "w041_broadened_independent_evaluator_diversity_packet_invalid"
            },
            "source_evidence_row_count": source_rows.len(),
            "broadened_independent_implementation_case_count": implementation_case_count,
            "broadened_independent_implementation_match_count": implementation_match_count,
            "independent_evaluator_row_count": independent_rows.len(),
            "cross_engine_differential_row_count": cross_engine_rows.len(),
            "mismatch_authority_row_count": authority_rows.len(),
            "accepted_boundary_count": accepted_boundary_count,
            "service_blocked_count": service_blocked_count,
            "exact_blocker_count": blockers.len(),
            "failed_row_count": failed_row_count,
            "fully_independent_evaluator_promoted": false,
            "operated_cross_engine_differential_service_promoted": false,
            "validation_failures": validation_failures
        });
        let run_summary = json!({
            "schema_version": W041_RUN_SUMMARY_SCHEMA_V1,
            "run_id": run_id,
            "artifact_root": relative_artifact_root,
            "source_evidence_index_path": source_evidence_index_path,
            "w041_independent_formula_evaluator_implementation_path": independent_implementation_path,
            "w041_independent_evaluator_breadth_register_path": independent_row_set_path,
            "w041_cross_engine_differential_service_register_path": cross_engine_path,
            "w041_mismatch_authority_router_path": mismatch_authority_path,
            "w041_exact_diversity_blocker_register_path": blocker_register_path,
            "promotion_decision_path": promotion_decision_path,
            "validation_path": validation_path,
            "source_evidence_row_count": source_rows.len(),
            "broadened_independent_implementation_case_count": implementation_case_count,
            "broadened_independent_implementation_match_count": implementation_match_count,
            "independent_evaluator_row_count": independent_rows.len(),
            "cross_engine_differential_row_count": cross_engine_rows.len(),
            "mismatch_authority_row_count": authority_rows.len(),
            "accepted_boundary_count": accepted_boundary_count,
            "service_blocked_count": service_blocked_count,
            "exact_blocker_count": blockers.len(),
            "failed_row_count": failed_row_count,
            "fully_independent_evaluator_promoted": false,
            "operated_cross_engine_differential_service_promoted": false,
            "w073_typed_only_formatting_guard_retained": true
        });

        write_json(
            &artifact_root.join("source_evidence_index.json"),
            &source_evidence_index,
        )?;
        write_json(
            &artifact_root.join("w041_independent_formula_evaluator_implementation.json"),
            &independent_implementation,
        )?;
        write_json(
            &artifact_root.join("w041_independent_evaluator_breadth_register.json"),
            &independent_row_set,
        )?;
        write_json(
            &artifact_root.join("w041_cross_engine_differential_service_register.json"),
            &cross_engine,
        )?;
        write_json(
            &artifact_root.join("w041_mismatch_authority_router.json"),
            &mismatch_authority,
        )?;
        write_json(
            &artifact_root.join("w041_exact_diversity_blocker_register.json"),
            &blocker_register,
        )?;
        write_json(
            &artifact_root.join("promotion_decision.json"),
            &promotion_decision,
        )?;
        write_json(&artifact_root.join("validation.json"), &validation)?;
        write_json(&artifact_root.join("run_summary.json"), &run_summary)?;

        Ok(DiversitySeamRunSummary {
            run_id: run_id.to_string(),
            schema_version: W041_RUN_SUMMARY_SCHEMA_V1.to_string(),
            source_evidence_row_count: source_rows.len(),
            diversity_disposition_row_count: independent_rows.len(),
            seam_watch_row_count: cross_engine_rows.len(),
            aligned_seam_watch_row_count: accepted_boundary_count,
            accepted_boundary_count,
            exact_blocker_count: blockers.len(),
            failed_row_count,
            fully_independent_evaluator_promoted: false,
            artifact_root: relative_artifact_root,
        })
    }

    fn execute_w040(
        &self,
        repo_root: &Path,
        run_id: &str,
    ) -> Result<DiversitySeamRunSummary, DiversitySeamError> {
        let relative_artifact_root =
            relative_artifact_path(&["docs", "test-runs", "core-engine", "diversity-seam", run_id]);
        let artifact_root = repo_root.join(&relative_artifact_root);
        if artifact_root.exists() {
            fs::remove_dir_all(&artifact_root).map_err(|source| {
                DiversitySeamError::RemoveDirectory {
                    path: artifact_root.display().to_string(),
                    source,
                }
            })?;
        }
        fs::create_dir_all(&artifact_root).map_err(|source| {
            DiversitySeamError::CreateDirectory {
                path: artifact_root.display().to_string(),
                source,
            }
        })?;

        let w040_direct_summary = read_json(repo_root, W040_DIRECT_OBLIGATION_SUMMARY)?;
        let w040_direct_map = read_json(repo_root, W040_DIRECT_OBLIGATION_MAP)?;
        let w040_formatting_intake = read_json(repo_root, W040_FORMATTING_INTAKE)?;
        let w039_diversity = read_json(repo_root, W039_DIVERSITY_SUMMARY)?;
        let w039_independent = read_json(repo_root, W039_INDEPENDENT_ROW_SET)?;
        let w039_cross_engine = read_json(repo_root, W039_CROSS_ENGINE_DIVERSITY)?;
        let w039_authority = read_json(repo_root, W039_DIFFERENTIAL_AUTHORITY)?;
        let w039_blockers = read_json(repo_root, W039_DIVERSITY_BLOCKERS)?;
        let w039_decision = read_json(repo_root, W039_DIVERSITY_DECISION)?;
        let w040_conformance = read_json(repo_root, W040_CONFORMANCE_SUMMARY)?;
        let w040_treecalc = read_json(repo_root, W040_TREECALC_SUMMARY)?;
        let w040_rust = read_json(repo_root, W040_RUST_TOTALITY_SUMMARY)?;
        let w040_lean_tla = read_json(repo_root, W040_LEAN_TLA_SUMMARY)?;
        let w040_stage2 = read_json(repo_root, W040_STAGE2_SUMMARY)?;
        let w040_stage2_blockers = read_json(repo_root, W040_STAGE2_BLOCKERS)?;
        let w040_operated = read_json(repo_root, W040_OPERATED_ASSURANCE_SUMMARY)?;
        let w040_retained = read_json(repo_root, W040_RETAINED_HISTORY_STORE)?;
        let w040_cross_engine_service = read_json(repo_root, W040_CROSS_ENGINE_SERVICE_REGISTER)?;
        let w040_service_blockers = read_json(repo_root, W040_OPERATED_SERVICE_BLOCKERS)?;

        let implementation_rows = w040_independent_scalar_evaluator_rows();
        let source_rows = w040_source_rows(
            &w040_direct_summary,
            &w040_direct_map,
            &w040_formatting_intake,
            &w039_diversity,
            &w039_decision,
            &w040_conformance,
            &w040_treecalc,
            &w040_rust,
            &w040_lean_tla,
            &w040_stage2,
            &w040_operated,
        );
        let independent_rows = w040_independent_evaluator_rows(
            &w039_independent,
            &w039_diversity,
            &w040_conformance,
            &w040_treecalc,
            &w040_rust,
            &w040_lean_tla,
            &implementation_rows,
        );
        let cross_engine_rows = w040_cross_engine_differential_rows(
            &w039_cross_engine,
            &w040_stage2,
            &w040_operated,
            &w040_retained,
            &w040_cross_engine_service,
            &w040_formatting_intake,
            &implementation_rows,
        );
        let authority_rows = w040_differential_authority_rows(
            &w039_authority,
            &w040_conformance,
            &w040_operated,
            &w040_cross_engine_service,
            &implementation_rows,
        );
        let blockers = w040_exact_blockers(
            &w039_blockers,
            &w040_stage2_blockers,
            &w040_service_blockers,
        );
        let accepted_boundary_count = independent_rows
            .iter()
            .chain(cross_engine_rows.iter())
            .chain(authority_rows.iter())
            .filter(|row| {
                matches!(
                    row.get("disposition_kind").and_then(Value::as_str),
                    Some("accepted_boundary")
                        | Some("accepted_external_slice")
                        | Some("bounded_independent_implementation")
                )
            })
            .count();
        let service_blocked_count = cross_engine_rows
            .iter()
            .chain(authority_rows.iter())
            .filter(|row| {
                row.get("service_state").and_then(Value::as_str) == Some("blocked")
                    || row.get("authority_state").and_then(Value::as_str)
                        == Some("blocked_no_operated_service")
            })
            .count();

        let mut validation_failures = source_validation_failures(&source_rows);
        validation_failures.extend(w040_diversity_validation_failures(
            &implementation_rows,
            &independent_rows,
            &cross_engine_rows,
            &authority_rows,
            &blockers,
        ));
        let failed_row_count = validation_failures.len();
        let implementation_case_count = implementation_rows.len();
        let implementation_match_count = implementation_rows
            .iter()
            .filter(|row| bool_at(row, "matches_expected"))
            .count();

        let source_evidence_index_path =
            format!("{relative_artifact_root}/source_evidence_index.json");
        let independent_implementation_path = format!(
            "{relative_artifact_root}/w040_independent_scalar_evaluator_implementation.json"
        );
        let independent_row_set_path =
            format!("{relative_artifact_root}/w040_independent_evaluator_row_set.json");
        let cross_engine_path =
            format!("{relative_artifact_root}/w040_cross_engine_differential_register.json");
        let differential_authority_path =
            format!("{relative_artifact_root}/w040_differential_authority_register.json");
        let blocker_register_path =
            format!("{relative_artifact_root}/w040_exact_diversity_blocker_register.json");
        let promotion_decision_path = format!("{relative_artifact_root}/promotion_decision.json");
        let validation_path = format!("{relative_artifact_root}/validation.json");

        let source_evidence_index = json!({
            "schema_version": W040_SOURCE_INDEX_SCHEMA_V1,
            "run_id": run_id,
            "artifact_root": relative_artifact_root,
            "source_evidence_row_count": source_rows.len(),
            "rows": source_rows,
            "source_artifacts": {
                "w040_direct_obligation_summary": W040_DIRECT_OBLIGATION_SUMMARY,
                "w040_direct_obligation_map": W040_DIRECT_OBLIGATION_MAP,
                "w040_formatting_intake": W040_FORMATTING_INTAKE,
                "w039_diversity_summary": W039_DIVERSITY_SUMMARY,
                "w039_independent_row_set": W039_INDEPENDENT_ROW_SET,
                "w039_cross_engine_diversity": W039_CROSS_ENGINE_DIVERSITY,
                "w039_differential_authority": W039_DIFFERENTIAL_AUTHORITY,
                "w039_diversity_blockers": W039_DIVERSITY_BLOCKERS,
                "w039_diversity_decision": W039_DIVERSITY_DECISION,
                "w040_conformance_summary": W040_CONFORMANCE_SUMMARY,
                "w040_treecalc_summary": W040_TREECALC_SUMMARY,
                "w040_rust_totality_summary": W040_RUST_TOTALITY_SUMMARY,
                "w040_lean_tla_summary": W040_LEAN_TLA_SUMMARY,
                "w040_stage2_summary": W040_STAGE2_SUMMARY,
                "w040_stage2_blockers": W040_STAGE2_BLOCKERS,
                "w040_operated_assurance_summary": W040_OPERATED_ASSURANCE_SUMMARY,
                "w040_retained_history_store": W040_RETAINED_HISTORY_STORE,
                "w040_cross_engine_service_register": W040_CROSS_ENGINE_SERVICE_REGISTER,
                "w040_operated_service_blockers": W040_OPERATED_SERVICE_BLOCKERS,
                "oxfml_inbound_notes": OXFML_INBOUND_NOTES
            }
        });
        let independent_implementation = json!({
            "schema_version": W040_INDEPENDENT_IMPLEMENTATION_SCHEMA_V1,
            "run_id": run_id,
            "implementation_kind": "bounded_independent_scalar_arithmetic_evaluator",
            "implementation_authority": "local_w040_diversity_runner_independent_from_tracecalc_and_optimized_core",
            "case_count": implementation_case_count,
            "match_count": implementation_match_count,
            "failed_case_count": implementation_case_count - implementation_match_count,
            "fully_independent_evaluator_promoted": false,
            "scope_limit": "integer arithmetic literals, names, +, *, and parentheses only",
            "rows": implementation_rows
        });
        let independent_row_set = json!({
            "schema_version": W040_INDEPENDENT_EVALUATOR_SCHEMA_V1,
            "run_id": run_id,
            "row_count": independent_rows.len(),
            "bounded_independent_implementation_case_count": implementation_case_count,
            "bounded_independent_implementation_match_count": implementation_match_count,
            "fully_independent_evaluator_promoted": false,
            "rows": independent_rows
        });
        let cross_engine = json!({
            "schema_version": W040_CROSS_ENGINE_DIVERSITY_SCHEMA_V1,
            "run_id": run_id,
            "row_count": cross_engine_rows.len(),
            "service_blocked_count": service_blocked_count,
            "bounded_independent_implementation_case_count": implementation_case_count,
            "operated_cross_engine_differential_service_promoted": false,
            "w073_typed_only_formatting_guard_retained": true,
            "rows": cross_engine_rows
        });
        let differential_authority = json!({
            "schema_version": W040_DIFFERENTIAL_AUTHORITY_SCHEMA_V1,
            "run_id": run_id,
            "row_count": authority_rows.len(),
            "accepted_boundary_count": accepted_boundary_count,
            "bounded_independent_implementation_case_count": implementation_case_count,
            "operated_service_authority_promoted": false,
            "fully_independent_evaluator_promoted": false,
            "rows": authority_rows
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
            "decision_state": "w040_bounded_independent_scalar_evaluator_bound_without_diversity_promotion",
            "bounded_independent_scalar_evaluator_present": true,
            "bounded_independent_implementation_case_count": implementation_case_count,
            "bounded_independent_implementation_match_count": implementation_match_count,
            "fully_independent_evaluator_promoted": false,
            "independent_evaluator_row_set_promoted": false,
            "operated_cross_engine_differential_service_promoted": false,
            "cross_engine_diversity_service_promoted": false,
            "broad_oxfml_seam_promoted": false,
            "callable_metadata_projection_promoted": false,
            "w073_formatting_handoff_triggered": false,
            "pack_grade_replay_promoted": false,
            "c5_promoted": false,
            "stage2_policy_promoted": false,
            "source_evidence_row_count": source_rows.len(),
            "independent_evaluator_row_count": independent_rows.len(),
            "cross_engine_differential_row_count": cross_engine_rows.len(),
            "differential_authority_row_count": authority_rows.len(),
            "accepted_boundary_count": accepted_boundary_count,
            "exact_blocker_count": blockers.len(),
            "blockers": blockers
                .iter()
                .map(|row| row["blocker_id"].clone())
                .collect::<Vec<_>>(),
            "semantic_equivalence_statement": "This W040 diversity runner adds a bounded independent scalar arithmetic evaluator and classifies diversity/service authority evidence only. It does not change evaluator kernels used by OxCalc, coordinator scheduling, recalc, publication, replay, pack, service, TraceCalc, TreeCalc, OxFml, OxFunc, Lean, or TLA semantics."
        });
        let validation = json!({
            "schema_version": W040_VALIDATION_SCHEMA_V1,
            "run_id": run_id,
            "status": if validation_failures.is_empty() {
                "w040_bounded_independent_evaluator_diversity_packet_valid"
            } else {
                "w040_bounded_independent_evaluator_diversity_packet_invalid"
            },
            "source_evidence_row_count": source_rows.len(),
            "bounded_independent_implementation_case_count": implementation_case_count,
            "bounded_independent_implementation_match_count": implementation_match_count,
            "independent_evaluator_row_count": independent_rows.len(),
            "cross_engine_differential_row_count": cross_engine_rows.len(),
            "differential_authority_row_count": authority_rows.len(),
            "accepted_boundary_count": accepted_boundary_count,
            "service_blocked_count": service_blocked_count,
            "exact_blocker_count": blockers.len(),
            "failed_row_count": failed_row_count,
            "fully_independent_evaluator_promoted": false,
            "operated_cross_engine_differential_service_promoted": false,
            "validation_failures": validation_failures
        });
        let run_summary = json!({
            "schema_version": W040_RUN_SUMMARY_SCHEMA_V1,
            "run_id": run_id,
            "artifact_root": relative_artifact_root,
            "source_evidence_index_path": source_evidence_index_path,
            "w040_independent_scalar_evaluator_implementation_path": independent_implementation_path,
            "w040_independent_evaluator_row_set_path": independent_row_set_path,
            "w040_cross_engine_differential_register_path": cross_engine_path,
            "w040_differential_authority_register_path": differential_authority_path,
            "w040_exact_diversity_blocker_register_path": blocker_register_path,
            "promotion_decision_path": promotion_decision_path,
            "validation_path": validation_path,
            "source_evidence_row_count": source_rows.len(),
            "bounded_independent_implementation_case_count": implementation_case_count,
            "bounded_independent_implementation_match_count": implementation_match_count,
            "independent_evaluator_row_count": independent_rows.len(),
            "cross_engine_differential_row_count": cross_engine_rows.len(),
            "differential_authority_row_count": authority_rows.len(),
            "accepted_boundary_count": accepted_boundary_count,
            "service_blocked_count": service_blocked_count,
            "exact_blocker_count": blockers.len(),
            "failed_row_count": failed_row_count,
            "fully_independent_evaluator_promoted": false,
            "operated_cross_engine_differential_service_promoted": false,
            "w073_typed_only_formatting_guard_retained": true
        });

        write_json(
            &artifact_root.join("source_evidence_index.json"),
            &source_evidence_index,
        )?;
        write_json(
            &artifact_root.join("w040_independent_scalar_evaluator_implementation.json"),
            &independent_implementation,
        )?;
        write_json(
            &artifact_root.join("w040_independent_evaluator_row_set.json"),
            &independent_row_set,
        )?;
        write_json(
            &artifact_root.join("w040_cross_engine_differential_register.json"),
            &cross_engine,
        )?;
        write_json(
            &artifact_root.join("w040_differential_authority_register.json"),
            &differential_authority,
        )?;
        write_json(
            &artifact_root.join("w040_exact_diversity_blocker_register.json"),
            &blocker_register,
        )?;
        write_json(
            &artifact_root.join("promotion_decision.json"),
            &promotion_decision,
        )?;
        write_json(&artifact_root.join("validation.json"), &validation)?;
        write_json(&artifact_root.join("run_summary.json"), &run_summary)?;

        Ok(DiversitySeamRunSummary {
            run_id: run_id.to_string(),
            schema_version: W040_RUN_SUMMARY_SCHEMA_V1.to_string(),
            source_evidence_row_count: source_rows.len(),
            diversity_disposition_row_count: independent_rows.len(),
            seam_watch_row_count: cross_engine_rows.len(),
            aligned_seam_watch_row_count: accepted_boundary_count,
            accepted_boundary_count,
            exact_blocker_count: blockers.len(),
            failed_row_count,
            fully_independent_evaluator_promoted: false,
            artifact_root: relative_artifact_root,
        })
    }

    fn execute_w039(
        &self,
        repo_root: &Path,
        run_id: &str,
    ) -> Result<DiversitySeamRunSummary, DiversitySeamError> {
        let relative_artifact_root =
            relative_artifact_path(&["docs", "test-runs", "core-engine", "diversity-seam", run_id]);
        let artifact_root = repo_root.join(&relative_artifact_root);
        if artifact_root.exists() {
            fs::remove_dir_all(&artifact_root).map_err(|source| {
                DiversitySeamError::RemoveDirectory {
                    path: artifact_root.display().to_string(),
                    source,
                }
            })?;
        }
        fs::create_dir_all(&artifact_root).map_err(|source| {
            DiversitySeamError::CreateDirectory {
                path: artifact_root.display().to_string(),
                source,
            }
        })?;

        let w036_independent = read_json(repo_root, W036_INDEPENDENT_SUMMARY)?;
        let w036_cross_engine = read_json(repo_root, W036_CROSS_ENGINE_SUMMARY)?;
        let w037_direct_oxfml = read_json(repo_root, W037_DIRECT_OXFML_SUMMARY)?;
        let w038_diversity = read_json(repo_root, W038_DIVERSITY_SUMMARY)?;
        let w038_diversity_disposition = read_json(repo_root, W038_DIVERSITY_DISPOSITION)?;
        let w038_diversity_blockers = read_json(repo_root, W038_DIVERSITY_BLOCKERS)?;
        let w039_residual_ledger = read_json(repo_root, W039_RESIDUAL_LEDGER_SUMMARY)?;
        let w039_formatting_intake = read_json(repo_root, W039_FORMATTING_INTAKE)?;
        let w039_conformance = read_json(repo_root, W039_CONFORMANCE_SUMMARY)?;
        let w039_proof_model = read_json(repo_root, W039_PROOF_MODEL_SUMMARY)?;
        let w039_stage2 = read_json(repo_root, W039_STAGE2_SUMMARY)?;
        let w039_stage2_blockers = read_json(repo_root, W039_STAGE2_BLOCKERS)?;
        let w039_operated = read_json(repo_root, W039_OPERATED_ASSURANCE_SUMMARY)?;
        let w039_cross_engine_substrate =
            read_json(repo_root, W039_CROSS_ENGINE_SERVICE_SUBSTRATE)?;
        let w039_operated_service_blockers = read_json(repo_root, W039_OPERATED_SERVICE_BLOCKERS)?;

        let source_rows = w039_source_rows(
            repo_root,
            &w036_independent,
            &w036_cross_engine,
            &w037_direct_oxfml,
            &w038_diversity,
            &w039_residual_ledger,
            &w039_formatting_intake,
            &w039_conformance,
            &w039_proof_model,
            &w039_stage2,
            &w039_operated,
        );
        let independent_rows = w039_independent_evaluator_rows(
            &w036_independent,
            &w036_cross_engine,
            &w037_direct_oxfml,
            &w038_diversity,
            &w038_diversity_disposition,
            &w039_conformance,
            &w039_proof_model,
        );
        let cross_engine_rows = w039_cross_engine_diversity_rows(
            &w036_cross_engine,
            &w038_diversity,
            &w039_stage2,
            &w039_operated,
            &w039_cross_engine_substrate,
            &w039_formatting_intake,
        );
        let authority_rows = w039_differential_authority_rows(
            &w036_independent,
            &w036_cross_engine,
            &w037_direct_oxfml,
            &w039_conformance,
            &w039_operated,
            &w039_cross_engine_substrate,
        );
        let blockers = w039_exact_blockers(
            &w038_diversity_blockers,
            &w039_stage2_blockers,
            &w039_operated_service_blockers,
        );
        let accepted_boundary_count = independent_rows
            .iter()
            .chain(cross_engine_rows.iter())
            .chain(authority_rows.iter())
            .filter(|row| {
                row.get("disposition_kind").and_then(Value::as_str) == Some("accepted_boundary")
                    || row.get("disposition_kind").and_then(Value::as_str)
                        == Some("accepted_external_slice")
            })
            .count();
        let service_blocked_count = cross_engine_rows
            .iter()
            .chain(authority_rows.iter())
            .filter(|row| {
                row.get("service_state").and_then(Value::as_str) == Some("blocked")
                    || row.get("authority_state").and_then(Value::as_str)
                        == Some("blocked_no_operated_service")
            })
            .count();

        let mut validation_failures = source_validation_failures(&source_rows);
        validation_failures.extend(w039_diversity_validation_failures(
            &independent_rows,
            &cross_engine_rows,
            &authority_rows,
            &blockers,
        ));
        let failed_row_count = validation_failures.len();

        let source_evidence_index_path =
            format!("{relative_artifact_root}/source_evidence_index.json");
        let independent_row_set_path =
            format!("{relative_artifact_root}/w039_independent_evaluator_row_set.json");
        let cross_engine_diversity_path =
            format!("{relative_artifact_root}/w039_cross_engine_diversity_register.json");
        let differential_authority_path =
            format!("{relative_artifact_root}/w039_differential_service_authority_register.json");
        let blocker_register_path =
            format!("{relative_artifact_root}/w039_exact_diversity_blocker_register.json");
        let promotion_decision_path = format!("{relative_artifact_root}/promotion_decision.json");
        let validation_path = format!("{relative_artifact_root}/validation.json");

        let source_evidence_index = json!({
            "schema_version": W039_SOURCE_INDEX_SCHEMA_V1,
            "run_id": run_id,
            "artifact_root": relative_artifact_root,
            "source_evidence_row_count": source_rows.len(),
            "rows": source_rows,
            "source_artifacts": {
                "w036_independent_summary": W036_INDEPENDENT_SUMMARY,
                "w036_cross_engine_summary": W036_CROSS_ENGINE_SUMMARY,
                "w037_direct_oxfml_summary": W037_DIRECT_OXFML_SUMMARY,
                "w038_diversity_summary": W038_DIVERSITY_SUMMARY,
                "w038_diversity_disposition": W038_DIVERSITY_DISPOSITION,
                "w038_diversity_blockers": W038_DIVERSITY_BLOCKERS,
                "w039_residual_ledger_summary": W039_RESIDUAL_LEDGER_SUMMARY,
                "w039_formatting_intake": W039_FORMATTING_INTAKE,
                "w039_conformance_summary": W039_CONFORMANCE_SUMMARY,
                "w039_proof_model_summary": W039_PROOF_MODEL_SUMMARY,
                "w039_stage2_summary": W039_STAGE2_SUMMARY,
                "w039_operated_assurance_summary": W039_OPERATED_ASSURANCE_SUMMARY,
                "w039_cross_engine_service_substrate": W039_CROSS_ENGINE_SERVICE_SUBSTRATE,
                "oxfml_inbound_notes": OXFML_INBOUND_NOTES
            }
        });
        let independent_row_set = json!({
            "schema_version": W039_INDEPENDENT_EVALUATOR_SCHEMA_V1,
            "run_id": run_id,
            "row_count": independent_rows.len(),
            "fully_independent_evaluator_promoted": false,
            "independent_implementation_row_count": 0,
            "rows": independent_rows
        });
        let cross_engine_diversity = json!({
            "schema_version": W039_CROSS_ENGINE_DIVERSITY_SCHEMA_V1,
            "run_id": run_id,
            "row_count": cross_engine_rows.len(),
            "service_blocked_count": service_blocked_count,
            "operated_cross_engine_differential_service_promoted": false,
            "w073_typed_only_formatting_guard_retained": true,
            "rows": cross_engine_rows
        });
        let differential_authority = json!({
            "schema_version": W039_DIFFERENTIAL_AUTHORITY_SCHEMA_V1,
            "run_id": run_id,
            "row_count": authority_rows.len(),
            "accepted_boundary_count": accepted_boundary_count,
            "operated_service_authority_promoted": false,
            "independent_evaluator_authority_promoted": false,
            "rows": authority_rows
        });
        let blocker_register = json!({
            "schema_version": W039_BLOCKER_REGISTER_SCHEMA_V1,
            "run_id": run_id,
            "exact_blocker_count": blockers.len(),
            "rows": blockers
        });
        let promotion_decision = json!({
            "schema_version": W039_PROMOTION_DECISION_SCHEMA_V1,
            "run_id": run_id,
            "decision_state": "w039_independent_evaluator_and_cross_engine_diversity_bound_without_promotion",
            "fully_independent_evaluator_promoted": false,
            "independent_evaluator_row_set_promoted": false,
            "operated_cross_engine_differential_service_promoted": false,
            "cross_engine_diversity_service_promoted": false,
            "broad_oxfml_seam_promoted": false,
            "callable_metadata_projection_promoted": false,
            "w073_formatting_handoff_triggered": false,
            "pack_grade_replay_promoted": false,
            "c5_promoted": false,
            "stage2_policy_promoted": false,
            "source_evidence_row_count": source_rows.len(),
            "independent_evaluator_row_count": independent_rows.len(),
            "cross_engine_diversity_row_count": cross_engine_rows.len(),
            "differential_authority_row_count": authority_rows.len(),
            "accepted_boundary_count": accepted_boundary_count,
            "exact_blocker_count": blockers.len(),
            "blockers": blockers
                .iter()
                .map(|row| row["blocker_id"].clone())
                .collect::<Vec<_>>(),
            "semantic_equivalence_statement": "This W039 diversity runner binds source evidence, independent-evaluator authority rows, cross-engine diversity rows, and service-authority blockers only. It does not change evaluator kernels, coordinator scheduling, recalc, publication, replay, pack, service, TraceCalc, TreeCalc, OxFml, OxFunc, Lean, or TLA semantics."
        });
        let validation = json!({
            "schema_version": W039_VALIDATION_SCHEMA_V1,
            "run_id": run_id,
            "status": if validation_failures.is_empty() {
                "w039_independent_evaluator_cross_engine_diversity_packet_valid"
            } else {
                "w039_independent_evaluator_cross_engine_diversity_packet_invalid"
            },
            "source_evidence_row_count": source_rows.len(),
            "independent_evaluator_row_count": independent_rows.len(),
            "cross_engine_diversity_row_count": cross_engine_rows.len(),
            "differential_authority_row_count": authority_rows.len(),
            "accepted_boundary_count": accepted_boundary_count,
            "service_blocked_count": service_blocked_count,
            "exact_blocker_count": blockers.len(),
            "failed_row_count": failed_row_count,
            "fully_independent_evaluator_promoted": false,
            "operated_cross_engine_differential_service_promoted": false,
            "validation_failures": validation_failures
        });
        let run_summary = json!({
            "schema_version": W039_RUN_SUMMARY_SCHEMA_V1,
            "run_id": run_id,
            "artifact_root": relative_artifact_root,
            "source_evidence_index_path": source_evidence_index_path,
            "w039_independent_evaluator_row_set_path": independent_row_set_path,
            "w039_cross_engine_diversity_register_path": cross_engine_diversity_path,
            "w039_differential_service_authority_register_path": differential_authority_path,
            "w039_exact_diversity_blocker_register_path": blocker_register_path,
            "promotion_decision_path": promotion_decision_path,
            "validation_path": validation_path,
            "source_evidence_row_count": source_rows.len(),
            "independent_evaluator_row_count": independent_rows.len(),
            "cross_engine_diversity_row_count": cross_engine_rows.len(),
            "differential_authority_row_count": authority_rows.len(),
            "accepted_boundary_count": accepted_boundary_count,
            "service_blocked_count": service_blocked_count,
            "exact_blocker_count": blockers.len(),
            "failed_row_count": failed_row_count,
            "fully_independent_evaluator_promoted": false,
            "operated_cross_engine_differential_service_promoted": false,
            "w073_typed_only_formatting_guard_retained": true
        });

        write_json(
            &artifact_root.join("source_evidence_index.json"),
            &source_evidence_index,
        )?;
        write_json(
            &artifact_root.join("w039_independent_evaluator_row_set.json"),
            &independent_row_set,
        )?;
        write_json(
            &artifact_root.join("w039_cross_engine_diversity_register.json"),
            &cross_engine_diversity,
        )?;
        write_json(
            &artifact_root.join("w039_differential_service_authority_register.json"),
            &differential_authority,
        )?;
        write_json(
            &artifact_root.join("w039_exact_diversity_blocker_register.json"),
            &blocker_register,
        )?;
        write_json(
            &artifact_root.join("promotion_decision.json"),
            &promotion_decision,
        )?;
        write_json(&artifact_root.join("validation.json"), &validation)?;
        write_json(&artifact_root.join("run_summary.json"), &run_summary)?;

        Ok(DiversitySeamRunSummary {
            run_id: run_id.to_string(),
            schema_version: W039_RUN_SUMMARY_SCHEMA_V1.to_string(),
            source_evidence_row_count: source_rows.len(),
            diversity_disposition_row_count: independent_rows.len(),
            seam_watch_row_count: cross_engine_rows.len(),
            aligned_seam_watch_row_count: accepted_boundary_count,
            accepted_boundary_count,
            exact_blocker_count: blockers.len(),
            failed_row_count,
            fully_independent_evaluator_promoted: false,
            artifact_root: relative_artifact_root,
        })
    }
}

fn source_rows(
    w036_independent: &Value,
    w036_cross_engine: &Value,
    w037_direct_oxfml: &Value,
    w038_conformance: &Value,
    w038_formal: &Value,
    w038_stage2: &Value,
    w038_operated: &Value,
) -> Vec<Value> {
    vec![
        json!({
            "row_id": "source.w036_independent_conformance",
            "artifact": W036_INDEPENDENT_SUMMARY,
            "missing_artifact_count": number_at(w036_independent, "missing_artifact_count"),
            "unexpected_mismatch_count": number_at(w036_independent, "unexpected_mismatch_count"),
            "failed_row_count": 0,
            "promotion_guard": !bool_at(w036_independent, "w036_full_independent_evaluator_promoted"),
            "semantic_state": "w036_independent_projection_differential_bound"
        }),
        json!({
            "row_id": "source.w036_cross_engine_differential",
            "artifact": W036_CROSS_ENGINE_SUMMARY,
            "missing_artifact_count": number_at(w036_cross_engine, "missing_artifact_count"),
            "unexpected_mismatch_count": number_at(w036_cross_engine, "unexpected_mismatch_count"),
            "failed_row_count": 0,
            "promotion_guard": !bool_at(w036_cross_engine, "full_independent_evaluator_promoted")
                && !bool_at(w036_cross_engine, "continuous_cross_engine_service_promoted"),
            "semantic_state": "w036_file_backed_cross_engine_differential_bound"
        }),
        json!({
            "row_id": "source.w037_direct_oxfml",
            "artifact": W037_DIRECT_OXFML_SUMMARY,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": number_at(w037_direct_oxfml, "expectation_mismatch_count"),
            "failed_row_count": 0,
            "promotion_guard": !bool_at(&w037_direct_oxfml["promotion_limits"], "general_oxfunc_kernel_claimed")
                && !bool_at(&w037_direct_oxfml["promotion_limits"], "pack_grade_replay_promoted")
                && !bool_at(&w037_direct_oxfml["promotion_limits"], "c5_promoted"),
            "semantic_state": "direct_oxfml_runtime_slice_bound"
        }),
        json!({
            "row_id": "source.w038_conformance",
            "artifact": W038_CONFORMANCE_SUMMARY,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": number_at(w038_conformance, "failed_row_count"),
            "promotion_guard": number_at(w038_conformance, "w038_match_promoted_count") == 0,
            "semantic_state": "w038_conformance_blockers_bound"
        }),
        json!({
            "row_id": "source.w038_formal_assurance",
            "artifact": W038_FORMAL_ASSURANCE_SUMMARY,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": number_at(w038_formal, "failed_row_count"),
            "promotion_guard": !bool_at(&w038_formal["promotion_claims"], "general_oxfunc_kernel_promoted"),
            "semantic_state": "w038_formal_external_kernel_boundary_bound"
        }),
        json!({
            "row_id": "source.w038_stage2_replay",
            "artifact": W038_STAGE2_REPLAY_SUMMARY,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": number_at(w038_stage2, "failed_row_count"),
            "promotion_guard": !bool_at(w038_stage2, "stage2_policy_promoted"),
            "semantic_state": "w038_stage2_formatting_watch_bound"
        }),
        json!({
            "row_id": "source.w038_operated_assurance",
            "artifact": W038_OPERATED_ASSURANCE_SUMMARY,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": number_at(w038_operated, "failed_row_count"),
            "promotion_guard": !bool_at(w038_operated, "operated_continuous_assurance_service_promoted")
                && !bool_at(w038_operated, "operated_cross_engine_differential_service_promoted"),
            "semantic_state": "w038_operated_service_blockers_bound"
        }),
    ]
}

fn diversity_rows(w036_independent: &Value, w036_cross_engine: &Value) -> Vec<Value> {
    vec![
        json!({
            "row_id": "diversity.tracecalc_reference_machine",
            "disposition_kind": "accepted_boundary",
            "diversity_state": "reference_oracle_not_independent_production_evaluator",
            "evidence": W036_INDEPENDENT_SUMMARY,
            "comparison_row_count": number_at(w036_independent, "comparison_row_count"),
            "promotion_consequence": "TraceCalc remains correctness oracle for covered reference behavior, not a fully independent optimized implementation."
        }),
        json!({
            "row_id": "diversity.treecalc_core_projection",
            "disposition_kind": "projection_evidence_not_independent",
            "diversity_state": "shared_implementation_projection",
            "evidence": W036_INDEPENDENT_SUMMARY,
            "exact_value_match_count": number_at(w036_independent, "exact_value_match_count"),
            "promotion_consequence": "TreeCalc/CoreEngine projection rows are useful differential evidence but cannot promote full independent evaluator diversity."
        }),
        json!({
            "row_id": "diversity.file_backed_cross_engine_differential",
            "disposition_kind": "accepted_boundary",
            "diversity_state": "file_backed_differential_not_operated_service",
            "evidence": W036_CROSS_ENGINE_SUMMARY,
            "differential_row_count": number_at(w036_cross_engine, "differential_row_count"),
            "promotion_consequence": "File-backed differential rows support assurance but do not promote an operated cross-engine service or independent evaluator."
        }),
        json!({
            "row_id": "diversity.direct_oxfml_external_evaluator_slice",
            "disposition_kind": "accepted_external_slice",
            "diversity_state": "external_formula_evaluator_slice_not_full_core_engine",
            "evidence": W037_DIRECT_OXFML_SUMMARY,
            "promotion_consequence": "Direct OxFml evidence strengthens seam confidence for formulas, LET/LAMBDA carrier, and W073 formatting, but does not independently implement OxCalc coordination."
        }),
        json!({
            "row_id": "diversity.fully_independent_evaluator_absent",
            "disposition_kind": "exact_remaining_blocker",
            "diversity_state": "blocked",
            "evidence": W036_INDEPENDENT_SUMMARY,
            "promotion_consequence": "Fully independent evaluator claims remain blocked until an independently implemented row set exists."
        }),
    ]
}

fn seam_watch_rows(
    repo_root: &Path,
    w037_direct_oxfml: &Value,
    w038_conformance: &Value,
    w038_conformance_blockers: &Value,
    w038_formal: &Value,
    w038_stage2: &Value,
) -> Vec<Value> {
    let oxfml_notes_present = repo_root.join(OXFML_INBOUND_NOTES).exists();
    let callable_metadata_blocker_present = row_with_field_exists(
        w038_conformance_blockers,
        "row_id",
        "w038_disposition_callable_metadata_projection_exact_blocker",
    );

    vec![
        json!({
            "row_id": "seam.oxfml_w073_typed_formatting",
            "watch_state": "aligned",
            "disposition_kind": "watch_input_bound",
            "source": W037_DIRECT_OXFML_SUMMARY,
            "evidence": {
                "w073_typed_rule_case_count": number_at(w037_direct_oxfml, "w073_typed_rule_case_count"),
                "w038_stage2_formatting_watch_row_count": number_at(w038_stage2, "formatting_watch_row_count")
            },
            "current_read": "W073 aggregate and visualization conditional-formatting metadata is typed_rule-only for the watched families; legacy thresholds are not interpreted there."
        }),
        json!({
            "row_id": "seam.format_delta_display_delta_distinct",
            "watch_state": "aligned",
            "disposition_kind": "watch_input_bound",
            "source": OXFML_INBOUND_NOTES,
            "evidence": {
                "oxfml_notes_present": oxfml_notes_present,
                "w038_stage2_formatting_watch_row_count": number_at(w038_stage2, "formatting_watch_row_count")
            },
            "current_read": "format_delta and display_delta remain distinct; broad display-facing closure is not inferred."
        }),
        json!({
            "row_id": "seam.direct_oxfml_runtime_facade",
            "watch_state": "aligned",
            "disposition_kind": "direct_external_slice_bound",
            "source": W037_DIRECT_OXFML_SUMMARY,
            "evidence": {
                "direct_oxfml_case_count": number_at(w037_direct_oxfml, "direct_oxfml_case_count"),
                "expectation_mismatch_count": number_at(w037_direct_oxfml, "expectation_mismatch_count")
            },
            "current_read": "OxCalc consumes direct OxFml runtime evidence through the current facade slice without reopening OxFml ownership."
        }),
        json!({
            "row_id": "seam.let_lambda_narrow_carrier",
            "watch_state": "aligned",
            "disposition_kind": "accepted_boundary",
            "source": W037_DIRECT_OXFML_SUMMARY,
            "evidence": {
                "let_lambda_case_count": number_at(w037_direct_oxfml, "let_lambda_case_count"),
                "general_oxfunc_kernel_promoted": bool_at(&w038_formal["promotion_claims"], "general_oxfunc_kernel_promoted")
            },
            "current_read": "LET/LAMBDA is included as a narrow carrier seam; general OxFunc kernels remain external."
        }),
        json!({
            "row_id": "seam.callable_metadata_projection",
            "watch_state": if callable_metadata_blocker_present { "blocked_exact" } else { "missing_blocker" },
            "disposition_kind": "exact_remaining_blocker",
            "source": W038_CONFORMANCE_BLOCKERS,
            "evidence": {
                "callable_metadata_blocker_present": callable_metadata_blocker_present,
                "w038_exact_remaining_blocker_count": number_at(w038_conformance, "w038_exact_remaining_blocker_count")
            },
            "current_read": "value-only callable carrier evidence is bound; callable metadata projection remains unpromoted."
        }),
        json!({
            "row_id": "seam.host_runtime_and_public_consumer_surface",
            "watch_state": "aligned",
            "disposition_kind": "note_level_watch",
            "source": OXFML_INBOUND_NOTES,
            "evidence": { "oxfml_notes_present": oxfml_notes_present },
            "current_read": "current OxFml consumer surface points ordinary downstream use at consumer::runtime, consumer::editor, and consumer::replay; OxCalc does not build a new long-term wrapper around the older flat root."
        }),
        json!({
            "row_id": "seam.structured_reference_table_packet",
            "watch_state": "aligned",
            "disposition_kind": "note_level_watch",
            "source": OXFML_INBOUND_NOTES,
            "evidence": { "oxfml_notes_present": oxfml_notes_present },
            "current_read": "table_catalog, enclosing_table_ref, and caller_table_region remain the aligned first structured-reference packet direction."
        }),
        json!({
            "row_id": "seam.registered_external_packet",
            "watch_state": "aligned",
            "disposition_kind": "note_level_watch",
            "source": OXFML_INBOUND_NOTES,
            "evidence": { "oxfml_notes_present": oxfml_notes_present },
            "current_read": "registered-external direct packet naming and seven-field descriptor read are note-level converged, without coordinator API freeze."
        }),
    ]
}

fn exact_blockers() -> Vec<Value> {
    vec![
        json!({
            "blocker_id": "diversity.fully_independent_evaluator_absent",
            "owner": "calc-zsr.7; calc-zsr.9",
            "status_after_run": "exact_remaining_blocker",
            "reason": "TraceCalc, TreeCalc/CoreEngine projection, direct OxFml, and file-backed differential rows do not constitute an independently implemented OxCalc evaluator row set.",
            "promotion_consequence": "fully independent evaluator diversity remains unpromoted"
        }),
        json!({
            "blocker_id": "diversity.operated_cross_engine_service_absent",
            "owner": "calc-zsr.6; calc-zsr.7",
            "status_after_run": "exact_remaining_blocker",
            "reason": "cross-engine evidence remains file-backed and does not run as an operated differential service.",
            "promotion_consequence": "continuous cross-engine diversity claims remain unpromoted"
        }),
        json!({
            "blocker_id": "seam.callable_metadata_projection_absent",
            "owner": "calc-zsr.7; external:OxFunc",
            "status_after_run": "exact_remaining_blocker",
            "reason": "narrow LET/LAMBDA value carrier evidence is present, but callable metadata projection is not exercised.",
            "promotion_consequence": "callable metadata projection remains unpromoted"
        }),
        json!({
            "blocker_id": "seam.broad_oxfml_display_and_publication_breadth_unfrozen",
            "owner": "calc-zsr.7; OxFml watch lane",
            "status_after_run": "exact_remaining_watch_blocker",
            "reason": "format_delta/display_delta distinction and current publication consequences are aligned, but broad display-facing and publication/topology breadth remains narrower than full future scope.",
            "promotion_consequence": "broad OxFml seam closure remains unpromoted until exercised evidence requires and supports it"
        }),
    ]
}

fn w039_source_rows(
    repo_root: &Path,
    w036_independent: &Value,
    w036_cross_engine: &Value,
    w037_direct_oxfml: &Value,
    w038_diversity: &Value,
    w039_residual_ledger: &Value,
    w039_formatting_intake: &Value,
    w039_conformance: &Value,
    w039_proof_model: &Value,
    w039_stage2: &Value,
    w039_operated: &Value,
) -> Vec<Value> {
    vec![
        json!({
            "row_id": "source.w036_independent_conformance",
            "artifact": W036_INDEPENDENT_SUMMARY,
            "missing_artifact_count": number_at(w036_independent, "missing_artifact_count"),
            "unexpected_mismatch_count": number_at(w036_independent, "unexpected_mismatch_count"),
            "failed_row_count": 0,
            "promotion_guard": !bool_at(w036_independent, "w036_full_independent_evaluator_promoted"),
            "semantic_state": "w036_tracecalc_treecalc_projection_differential_bound"
        }),
        json!({
            "row_id": "source.w036_cross_engine_differential",
            "artifact": W036_CROSS_ENGINE_SUMMARY,
            "missing_artifact_count": number_at(w036_cross_engine, "missing_artifact_count"),
            "unexpected_mismatch_count": number_at(w036_cross_engine, "unexpected_mismatch_count"),
            "failed_row_count": 0,
            "promotion_guard": !bool_at(w036_cross_engine, "full_independent_evaluator_promoted")
                && !bool_at(w036_cross_engine, "continuous_cross_engine_service_promoted"),
            "semantic_state": "w036_file_backed_cross_engine_differential_bound"
        }),
        json!({
            "row_id": "source.w037_direct_oxfml",
            "artifact": W037_DIRECT_OXFML_SUMMARY,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": number_at(w037_direct_oxfml, "expectation_mismatch_count"),
            "failed_row_count": 0,
            "promotion_guard": !bool_at(&w037_direct_oxfml["promotion_limits"], "general_oxfunc_kernel_claimed")
                && !bool_at(&w037_direct_oxfml["promotion_limits"], "pack_grade_replay_promoted")
                && !bool_at(&w037_direct_oxfml["promotion_limits"], "c5_promoted"),
            "semantic_state": "direct_oxfml_runtime_slice_and_w073_guard_bound"
        }),
        json!({
            "row_id": "source.w038_diversity_seam_watch",
            "artifact": W038_DIVERSITY_SUMMARY,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": number_at(w038_diversity, "failed_row_count"),
            "promotion_guard": !bool_at(w038_diversity, "fully_independent_evaluator_promoted"),
            "semantic_state": "w038_diversity_and_seam_watch_bound_without_promotion"
        }),
        json!({
            "row_id": "source.w039_residual_successor_ledger",
            "artifact": W039_RESIDUAL_LEDGER_SUMMARY,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": 0,
            "promotion_guard": text_at(w039_residual_ledger, "w073_formatting_intake") == "typed_only_guard_recorded",
            "semantic_state": "w039_obligation_and_promotion_map_bound"
        }),
        json!({
            "row_id": "source.w039_w073_formatting_intake",
            "artifact": W039_FORMATTING_INTAKE,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": 0,
            "promotion_guard": text_at(w039_formatting_intake, "thresholds_rule").contains("intentionally ignored")
                && array_len_at(w039_formatting_intake, "typed_only_families") == 7,
            "semantic_state": "current_oxfml_w073_typed_only_formatting_intake_retained",
            "oxfml_inbound_notes_present": repo_root.join(OXFML_INBOUND_NOTES).exists()
        }),
        json!({
            "row_id": "source.w039_optimized_core_conformance",
            "artifact": W039_CONFORMANCE_SUMMARY,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": number_at(w039_conformance, "failed_row_count"),
            "promotion_guard": number_at(w039_conformance, "w039_match_promoted_count") == 0,
            "semantic_state": "optimized_core_exact_blockers_retained_without_full_core_promotion"
        }),
        json!({
            "row_id": "source.w039_proof_model",
            "artifact": W039_PROOF_MODEL_SUMMARY,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": number_at(w039_proof_model, "failed_row_count"),
            "promotion_guard": !bool_at(&w039_proof_model["promotion_claims"], "full_lean_verification_promoted")
                && !bool_at(&w039_proof_model["promotion_claims"], "full_tla_verification_promoted")
                && !bool_at(&w039_proof_model["promotion_claims"], "general_oxfunc_kernel_promoted"),
            "semantic_state": "proof_model_totality_bound_without_full_verification_promotion"
        }),
        json!({
            "row_id": "source.w039_stage2_policy_governance",
            "artifact": W039_STAGE2_SUMMARY,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": number_at(w039_stage2, "failed_row_count"),
            "promotion_guard": !bool_at(w039_stage2, "stage2_policy_promoted"),
            "semantic_state": "stage2_policy_governance_bound_without_production_policy_promotion"
        }),
        json!({
            "row_id": "source.w039_operated_assurance_substrate",
            "artifact": W039_OPERATED_ASSURANCE_SUMMARY,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": number_at(w039_operated, "failed_row_count"),
            "promotion_guard": !bool_at(w039_operated, "operated_continuous_assurance_service_promoted")
                && !bool_at(w039_operated, "operated_cross_engine_differential_service_promoted")
                && !bool_at(w039_operated, "retained_history_service_promoted"),
            "semantic_state": "operated_assurance_substrate_bound_without_operated_service_promotion"
        }),
    ]
}

fn w039_independent_evaluator_rows(
    w036_independent: &Value,
    w036_cross_engine: &Value,
    w037_direct_oxfml: &Value,
    w038_diversity: &Value,
    w038_diversity_disposition: &Value,
    w039_conformance: &Value,
    w039_proof_model: &Value,
) -> Vec<Value> {
    vec![
        json!({
            "row_id": "independent.tracecalc_reference_oracle",
            "disposition_kind": "accepted_boundary",
            "independence_state": "reference_oracle_not_independent_production_evaluator",
            "evidence": W036_INDEPENDENT_SUMMARY,
            "comparison_row_count": number_at(w036_independent, "comparison_row_count"),
            "promotion_consequence": "TraceCalc remains the correctness oracle for covered behavior, not a second production evaluator."
        }),
        json!({
            "row_id": "independent.treecalc_core_projection",
            "disposition_kind": "projection_evidence_not_independent",
            "independence_state": "shared_projection_over_oxcalc_core_behavior",
            "evidence": W036_INDEPENDENT_SUMMARY,
            "exact_value_match_count": number_at(w036_independent, "exact_value_match_count"),
            "promotion_consequence": "Shared TreeCalc/CoreEngine projection evidence cannot count as independent implementation authority."
        }),
        json!({
            "row_id": "independent.optimized_core_exact_blocker_disposition",
            "disposition_kind": "same_engine_family_not_independent",
            "independence_state": "optimized_core_disposition_same_authority_family",
            "evidence": W039_CONFORMANCE_SUMMARY,
            "w039_direct_evidence_bound_count": number_at(w039_conformance, "w039_direct_evidence_bound_count"),
            "w039_exact_remaining_blocker_count": number_at(w039_conformance, "w039_exact_remaining_blocker_count"),
            "promotion_consequence": "Optimized/core evidence strengthens conformance, but does not establish a separate evaluator implementation."
        }),
        json!({
            "row_id": "independent.direct_oxfml_external_formula_slice",
            "disposition_kind": "accepted_external_slice",
            "independence_state": "external_formula_evaluator_slice_not_oxcalc_coordinator",
            "evidence": W037_DIRECT_OXFML_SUMMARY,
            "direct_oxfml_case_count": number_at(w037_direct_oxfml, "direct_oxfml_case_count"),
            "let_lambda_case_count": number_at(w037_direct_oxfml, "let_lambda_case_count"),
            "promotion_consequence": "Direct OxFml evidence counts for the consumed formula seam, including narrow LET/LAMBDA and W073 guards, but not for OxCalc coordinator implementation diversity."
        }),
        json!({
            "row_id": "independent.file_backed_cross_engine_differential",
            "disposition_kind": "accepted_boundary",
            "independence_state": "file_backed_differential_not_operated_implementation",
            "evidence": W036_CROSS_ENGINE_SUMMARY,
            "differential_row_count": number_at(w036_cross_engine, "differential_row_count"),
            "promotion_consequence": "File-backed differential rows remain useful assurance evidence and do not promote an independent evaluator."
        }),
        json!({
            "row_id": "independent.formal_model_not_runtime_evaluator",
            "disposition_kind": "accepted_boundary",
            "independence_state": "proof_model_not_executable_runtime_evaluator",
            "evidence": W039_PROOF_MODEL_SUMMARY,
            "local_proof_row_count": number_at(w039_proof_model, "local_proof_row_count"),
            "totality_boundary_count": number_at(w039_proof_model, "totality_boundary_count"),
            "promotion_consequence": "Lean/TLA evidence strengthens specification and proof obligations but is not a separately implemented evaluator row set."
        }),
        json!({
            "row_id": "independent.fully_independent_evaluator_row_set_absent",
            "disposition_kind": "exact_remaining_blocker",
            "independence_state": "independent_implementation_absent",
            "evidence": W038_DIVERSITY_DISPOSITION,
            "w038_diversity_disposition_row_count": number_at(w038_diversity, "diversity_disposition_row_count"),
            "w038_rows_present": row_count_at(w038_diversity_disposition),
            "promotion_consequence": "Full independent-evaluator diversity remains blocked until an independently implemented row set exists and has replay/differential evidence."
        }),
    ]
}

fn w039_cross_engine_diversity_rows(
    w036_cross_engine: &Value,
    w038_diversity: &Value,
    w039_stage2: &Value,
    w039_operated: &Value,
    w039_cross_engine_substrate: &Value,
    w039_formatting_intake: &Value,
) -> Vec<Value> {
    vec![
        json!({
            "row_id": "cross_engine.file_backed_differential_harness",
            "disposition_kind": "accepted_boundary",
            "service_state": "file_backed_not_operated",
            "evidence": W036_CROSS_ENGINE_SUMMARY,
            "differential_row_count": number_at(w036_cross_engine, "differential_row_count"),
            "promotion_consequence": "The harness is retained as deterministic evidence, not as an operated service."
        }),
        json!({
            "row_id": "cross_engine.w038_diversity_seam_baseline",
            "disposition_kind": "accepted_boundary",
            "service_state": "baseline_bound",
            "evidence": W038_DIVERSITY_SUMMARY,
            "diversity_disposition_row_count": number_at(w038_diversity, "diversity_disposition_row_count"),
            "seam_watch_row_count": number_at(w038_diversity, "seam_watch_row_count"),
            "promotion_consequence": "W038 baseline is carried forward as evidence lineage only."
        }),
        json!({
            "row_id": "cross_engine.stage2_service_dependency",
            "disposition_kind": "service_dependency_blocked",
            "service_state": "blocked",
            "evidence": W039_STAGE2_SUMMARY,
            "stage2_policy_promoted": bool_at(w039_stage2, "stage2_policy_promoted"),
            "exact_remaining_blocker_count": number_at(w039_stage2, "exact_remaining_blocker_count"),
            "promotion_consequence": "Stage 2 service-dependent diversity cannot promote while production policy and operated Stage 2 differential evidence are blocked."
        }),
        json!({
            "row_id": "cross_engine.operated_service_substrate",
            "disposition_kind": "service_substrate_bound",
            "service_state": if bool_at(w039_cross_engine_substrate, "operated_cross_engine_differential_service_present") { "present" } else { "blocked" },
            "evidence": W039_CROSS_ENGINE_SERVICE_SUBSTRATE,
            "file_backed_pilot_present": bool_at(w039_cross_engine_substrate, "file_backed_pilot_present"),
            "operated_cross_engine_differential_service_present": bool_at(w039_cross_engine_substrate, "operated_cross_engine_differential_service_present"),
            "promotion_consequence": "The substrate records service shape and blocked claims, but does not promote an operated cross-engine service."
        }),
        json!({
            "row_id": "cross_engine.retained_history_support",
            "disposition_kind": "supporting_evidence_not_service",
            "service_state": "supporting_history_only",
            "evidence": W039_OPERATED_ASSURANCE_SUMMARY,
            "multi_run_history_row_count": number_at(w039_operated, "multi_run_history_row_count"),
            "retained_history_service_promoted": bool_at(w039_operated, "retained_history_service_promoted"),
            "promotion_consequence": "Retained history supports future service operation and replay audit, but the retained-history service remains unpromoted."
        }),
        json!({
            "row_id": "cross_engine.w073_typed_only_formatting_guard",
            "disposition_kind": "accepted_boundary",
            "service_state": "observable_guard_retained",
            "evidence": W039_FORMATTING_INTAKE,
            "typed_only_family_count": array_len_at(w039_formatting_intake, "typed_only_families"),
            "thresholds_rule": text_at(w039_formatting_intake, "thresholds_rule"),
            "promotion_consequence": "Diversity evidence must not compare formatting rows that still rely on W072 bounded threshold strings for W073 aggregate or visualization families."
        }),
        json!({
            "row_id": "cross_engine.mismatch_triage_and_quarantine_service",
            "disposition_kind": "exact_remaining_blocker",
            "service_state": "blocked",
            "evidence": W039_OPERATED_ASSURANCE_SUMMARY,
            "alert_decision_count": number_at(w039_operated, "alert_decision_count"),
            "quarantine_decision_count": number_at(w039_operated, "quarantine_decision_count"),
            "promotion_consequence": "Cross-engine diversity needs an operated mismatch triage/quarantine path before service-level promotion."
        }),
    ]
}

fn w039_differential_authority_rows(
    w036_independent: &Value,
    w036_cross_engine: &Value,
    w037_direct_oxfml: &Value,
    w039_conformance: &Value,
    w039_operated: &Value,
    w039_cross_engine_substrate: &Value,
) -> Vec<Value> {
    vec![
        json!({
            "row_id": "authority.tracecalc_correctness_oracle",
            "disposition_kind": "accepted_boundary",
            "authority_state": "reference_oracle",
            "evidence": W036_INDEPENDENT_SUMMARY,
            "comparison_row_count": number_at(w036_independent, "comparison_row_count"),
            "authority_limit": "oracle authority for covered observable behavior only"
        }),
        json!({
            "row_id": "authority.optimized_core_same_implementation_family",
            "disposition_kind": "same_authority_family",
            "authority_state": "not_independent",
            "evidence": W039_CONFORMANCE_SUMMARY,
            "w039_exact_remaining_blocker_count": number_at(w039_conformance, "w039_exact_remaining_blocker_count"),
            "authority_limit": "optimized/core conformance does not supply a separate implementation authority"
        }),
        json!({
            "row_id": "authority.direct_oxfml_external_formula_evaluator",
            "disposition_kind": "accepted_external_slice",
            "authority_state": "external_formula_evaluator_only",
            "evidence": W037_DIRECT_OXFML_SUMMARY,
            "direct_oxfml_case_count": number_at(w037_direct_oxfml, "direct_oxfml_case_count"),
            "authority_limit": "external evaluator authority is limited to the consumed OxFml seam and does not implement OxCalc scheduling/publication"
        }),
        json!({
            "row_id": "authority.file_backed_cross_engine_differential",
            "disposition_kind": "accepted_boundary",
            "authority_state": "file_backed_artifact_authority",
            "evidence": W036_CROSS_ENGINE_SUMMARY,
            "differential_row_count": number_at(w036_cross_engine, "differential_row_count"),
            "authority_limit": "file-backed row agreement cannot substitute for an operated differential service"
        }),
        json!({
            "row_id": "authority.operated_cross_engine_service",
            "disposition_kind": "exact_remaining_blocker",
            "authority_state": "blocked_no_operated_service",
            "evidence": W039_CROSS_ENGINE_SERVICE_SUBSTRATE,
            "operated_cross_engine_differential_service_present": bool_at(w039_cross_engine_substrate, "operated_cross_engine_differential_service_present"),
            "authority_limit": "no recurring service endpoint, retained service history, or service-level mismatch action path exists"
        }),
        json!({
            "row_id": "authority.independent_evaluator_implementation",
            "disposition_kind": "exact_remaining_blocker",
            "authority_state": "blocked_no_independent_implementation",
            "evidence": W039_OPERATED_ASSURANCE_SUMMARY,
            "operated_cross_engine_differential_service_promoted": bool_at(w039_operated, "operated_cross_engine_differential_service_promoted"),
            "authority_limit": "no independently implemented evaluator row set exists"
        }),
        json!({
            "row_id": "authority.release_grade_diversity_promotion",
            "disposition_kind": "exact_remaining_blocker",
            "authority_state": "blocked_no_release_grade_authority",
            "evidence": W039_OPERATED_ASSURANCE_SUMMARY,
            "exact_service_blocker_count": number_at(w039_operated, "exact_service_blocker_count"),
            "authority_limit": "release-grade diversity authority remains unavailable while independent implementation and operated service blockers remain"
        }),
    ]
}

fn w039_exact_blockers(
    w038_diversity_blockers: &Value,
    w039_stage2_blockers: &Value,
    w039_operated_service_blockers: &Value,
) -> Vec<Value> {
    vec![
        json!({
            "blocker_id": "w039_diversity.fully_independent_evaluator_implementation_absent",
            "owner": "calc-f7o.6",
            "status_after_run": "exact_remaining_blocker",
            "evidence": W038_DIVERSITY_BLOCKERS,
            "predecessor_blocker_present": row_with_field_exists(
                w038_diversity_blockers,
                "blocker_id",
                "diversity.fully_independent_evaluator_absent"
            ),
            "reason": "TraceCalc, TreeCalc/CoreEngine projection, direct OxFml, formal model, and file-backed differential rows do not constitute an independently implemented OxCalc evaluator row set.",
            "promotion_consequence": "fully independent evaluator diversity remains unpromoted"
        }),
        json!({
            "blocker_id": "w039_diversity.independent_evaluator_authority_absent",
            "owner": "calc-f7o.6",
            "status_after_run": "exact_remaining_blocker",
            "evidence": W039_CONFORMANCE_SUMMARY,
            "reason": "No row set is backed by a separate implementation authority with its own execution kernel and replay evidence.",
            "promotion_consequence": "independent evaluator row-set promotion remains unavailable"
        }),
        json!({
            "blocker_id": "w039_diversity.operated_cross_engine_service_absent",
            "owner": "calc-f7o.6; calc-f7o.5",
            "status_after_run": "exact_remaining_blocker",
            "evidence": W039_CROSS_ENGINE_SERVICE_SUBSTRATE,
            "predecessor_blocker_present": row_with_field_exists(
                w038_diversity_blockers,
                "blocker_id",
                "diversity.operated_cross_engine_service_absent"
            ),
            "reason": "W039 has a file-backed cross-engine substrate, not an operated recurring differential service.",
            "promotion_consequence": "operated cross-engine diversity service remains unpromoted"
        }),
        json!({
            "blocker_id": "w039_diversity.stage2_differential_service_dependency_absent",
            "owner": "calc-f7o.4; calc-f7o.6",
            "status_after_run": "exact_remaining_blocker",
            "evidence": W039_STAGE2_BLOCKERS,
            "stage2_blocker_rows": row_count_at(w039_stage2_blockers),
            "reason": "Stage 2 diversity needs production policy, partition soundness, and operated differential service evidence before strategy-level diversity promotion.",
            "promotion_consequence": "Stage 2 diversity remains a blocked dependency for release-grade promotion"
        }),
        json!({
            "blocker_id": "w039_diversity.mismatch_triage_and_quarantine_service_absent",
            "owner": "calc-f7o.5; calc-f7o.6",
            "status_after_run": "exact_remaining_blocker",
            "evidence": W039_OPERATED_SERVICE_BLOCKERS,
            "service_blocker_rows": row_count_at(w039_operated_service_blockers),
            "reason": "The retained history and alert rules are bound, but no external alert dispatcher or mismatch quarantine service is operated.",
            "promotion_consequence": "service-level diversity assurance remains unpromoted"
        }),
        json!({
            "blocker_id": "w039_diversity.release_grade_promotion_authority_absent",
            "owner": "calc-f7o.9",
            "status_after_run": "exact_remaining_blocker",
            "evidence": W039_OPERATED_ASSURANCE_SUMMARY,
            "reason": "Release-grade diversity cannot be claimed from proxy evidence, file-backed rows, shared projections, or retained blockers.",
            "promotion_consequence": "release-grade verification, pack/C5, and diversity promotion remain unavailable to this bead"
        }),
    ]
}

fn w039_diversity_validation_failures(
    independent_rows: &[Value],
    cross_engine_rows: &[Value],
    authority_rows: &[Value],
    blockers: &[Value],
) -> Vec<String> {
    let mut failures = Vec::new();
    if !independent_rows.iter().any(|row| {
        row.get("independence_state").and_then(Value::as_str)
            == Some("independent_implementation_absent")
    }) {
        failures.push("w039_diversity.independent_implementation_absent_row_missing".to_string());
    }
    if !cross_engine_rows
        .iter()
        .any(|row| row.get("service_state").and_then(Value::as_str) == Some("blocked"))
    {
        failures.push("w039_diversity.blocked_cross_engine_service_row_missing".to_string());
    }
    if !authority_rows.iter().any(|row| {
        row.get("authority_state").and_then(Value::as_str) == Some("blocked_no_operated_service")
    }) {
        failures.push("w039_diversity.operated_service_authority_blocker_missing".to_string());
    }
    if blockers.len() < 6 {
        failures.push("w039_diversity.exact_blocker_count_below_gate".to_string());
    }
    if independent_rows
        .iter()
        .chain(cross_engine_rows.iter())
        .chain(authority_rows.iter())
        .any(|row| row.get("disposition_kind").and_then(Value::as_str) == Some("promoted"))
    {
        failures.push("w039_diversity.unexpected_promotion_row".to_string());
    }
    failures
}

#[derive(Debug, Clone)]
struct IndependentScalarCase {
    case_id: &'static str,
    expression: &'static str,
    environment: &'static [(&'static str, i64)],
    expected: i64,
}

fn independent_scalar_cases() -> Vec<IndependentScalarCase> {
    const EMPTY_ENV: &[(&str, i64)] = &[];
    const X3_ENV: &[(&str, i64)] = &[("X", 3)];
    const XY_ENV: &[(&str, i64)] = &[("X", 3), ("Y", 60)];

    vec![
        IndependentScalarCase {
            case_id: "scalar.literal",
            expression: "2",
            environment: EMPTY_ENV,
            expected: 2,
        },
        IndependentScalarCase {
            case_id: "scalar.precedence",
            expression: "2+3*4",
            environment: EMPTY_ENV,
            expected: 14,
        },
        IndependentScalarCase {
            case_id: "scalar.parentheses",
            expression: "(2+3)*4",
            environment: EMPTY_ENV,
            expected: 20,
        },
        IndependentScalarCase {
            case_id: "scalar.named_reference_y",
            expression: "X*20",
            environment: X3_ENV,
            expected: 60,
        },
        IndependentScalarCase {
            case_id: "scalar.incremental_chain_z",
            expression: "X+Y",
            environment: XY_ENV,
            expected: 63,
        },
    ]
}

fn eval_scalar_expression(
    expression: &str,
    environment: &[(&'static str, i64)],
) -> Result<i64, String> {
    let mut parser = ScalarParser {
        input: expression.as_bytes(),
        position: 0,
        environment,
    };
    let value = parser.parse_expr()?;
    parser.skip_ws();
    if parser.position == parser.input.len() {
        Ok(value)
    } else {
        Err(format!(
            "unexpected trailing input at byte {}",
            parser.position
        ))
    }
}

struct ScalarParser<'a> {
    input: &'a [u8],
    position: usize,
    environment: &'a [(&'static str, i64)],
}

impl ScalarParser<'_> {
    fn parse_expr(&mut self) -> Result<i64, String> {
        let mut value = self.parse_term()?;
        loop {
            self.skip_ws();
            if !self.consume(b'+') {
                return Ok(value);
            }
            value += self.parse_term()?;
        }
    }

    fn parse_term(&mut self) -> Result<i64, String> {
        let mut value = self.parse_factor()?;
        loop {
            self.skip_ws();
            if !self.consume(b'*') {
                return Ok(value);
            }
            value *= self.parse_factor()?;
        }
    }

    fn parse_factor(&mut self) -> Result<i64, String> {
        self.skip_ws();
        if self.consume(b'(') {
            let value = self.parse_expr()?;
            self.skip_ws();
            if self.consume(b')') {
                return Ok(value);
            }
            return Err(format!("missing ')' at byte {}", self.position));
        }
        if self.peek().is_some_and(|byte| byte.is_ascii_digit()) {
            return self.parse_number();
        }
        if self
            .peek()
            .is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'_')
        {
            return self.parse_identifier();
        }
        Err(format!("expected factor at byte {}", self.position))
    }

    fn parse_number(&mut self) -> Result<i64, String> {
        let start = self.position;
        while self.peek().is_some_and(|byte| byte.is_ascii_digit()) {
            self.position += 1;
        }
        std::str::from_utf8(&self.input[start..self.position])
            .map_err(|err| err.to_string())?
            .parse::<i64>()
            .map_err(|err| err.to_string())
    }

    fn parse_identifier(&mut self) -> Result<i64, String> {
        let start = self.position;
        while self
            .peek()
            .is_some_and(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        {
            self.position += 1;
        }
        let name = std::str::from_utf8(&self.input[start..self.position])
            .map_err(|err| err.to_string())?;
        self.environment
            .iter()
            .find_map(|(candidate, value)| (*candidate == name).then_some(*value))
            .ok_or_else(|| format!("unknown name {name}"))
    }

    fn skip_ws(&mut self) {
        while self.peek().is_some_and(|byte| byte.is_ascii_whitespace()) {
            self.position += 1;
        }
    }

    fn consume(&mut self, expected: u8) -> bool {
        if self.peek() == Some(expected) {
            self.position += 1;
            true
        } else {
            false
        }
    }

    fn peek(&self) -> Option<u8> {
        self.input.get(self.position).copied()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FormulaValue {
    Int(i64),
    Bool(bool),
}

impl FormulaValue {
    fn as_int(self) -> Result<i64, String> {
        match self {
            Self::Int(value) => Ok(value),
            Self::Bool(value) => Err(format!("expected integer value, found boolean {value}")),
        }
    }

    fn as_bool(self) -> Result<bool, String> {
        match self {
            Self::Bool(value) => Ok(value),
            Self::Int(value) => Err(format!("expected boolean value, found integer {value}")),
        }
    }
}

#[derive(Debug, Clone)]
struct IndependentFormulaCase {
    case_id: &'static str,
    expression: &'static str,
    environment: &'static [(&'static str, FormulaValue)],
    expected: FormulaValue,
}

fn w041_independent_formula_cases() -> Vec<IndependentFormulaCase> {
    const EMPTY_ENV: &[(&str, FormulaValue)] = &[];
    const X3_ENV: &[(&str, FormulaValue)] = &[("X", FormulaValue::Int(3))];
    const XY_ENV: &[(&str, FormulaValue)] =
        &[("X", FormulaValue::Int(3)), ("Y", FormulaValue::Int(60))];

    vec![
        IndependentFormulaCase {
            case_id: "formula.literal",
            expression: "2",
            environment: EMPTY_ENV,
            expected: FormulaValue::Int(2),
        },
        IndependentFormulaCase {
            case_id: "formula.precedence",
            expression: "2+3*4",
            environment: EMPTY_ENV,
            expected: FormulaValue::Int(14),
        },
        IndependentFormulaCase {
            case_id: "formula.subtraction_unary",
            expression: "-(X-5)",
            environment: X3_ENV,
            expected: FormulaValue::Int(2),
        },
        IndependentFormulaCase {
            case_id: "formula.division",
            expression: "Y/X+1",
            environment: XY_ENV,
            expected: FormulaValue::Int(21),
        },
        IndependentFormulaCase {
            case_id: "formula.comparison_equal",
            expression: "X*20=Y",
            environment: XY_ENV,
            expected: FormulaValue::Bool(true),
        },
        IndependentFormulaCase {
            case_id: "formula.if_branch",
            expression: "IF(X>2,Y,X)",
            environment: XY_ENV,
            expected: FormulaValue::Int(60),
        },
        IndependentFormulaCase {
            case_id: "formula.incremental_chain_z",
            expression: "X+Y",
            environment: XY_ENV,
            expected: FormulaValue::Int(63),
        },
        IndependentFormulaCase {
            case_id: "formula.not_equal_guard",
            expression: "IF(X<>2,63,0)",
            environment: X3_ENV,
            expected: FormulaValue::Int(63),
        },
    ]
}

fn eval_formula_fragment(
    expression: &str,
    environment: &[(&'static str, FormulaValue)],
) -> Result<FormulaValue, String> {
    let mut parser = FormulaParser {
        input: expression.as_bytes(),
        position: 0,
        environment,
    };
    let value = parser.parse_expression()?;
    parser.skip_ws();
    if parser.position == parser.input.len() {
        Ok(value)
    } else {
        Err(format!(
            "unexpected trailing input at byte {}",
            parser.position
        ))
    }
}

struct FormulaParser<'a> {
    input: &'a [u8],
    position: usize,
    environment: &'a [(&'static str, FormulaValue)],
}

impl FormulaParser<'_> {
    fn parse_expression(&mut self) -> Result<FormulaValue, String> {
        self.parse_comparison()
    }

    fn parse_comparison(&mut self) -> Result<FormulaValue, String> {
        let left = self.parse_additive()?;
        self.skip_ws();
        let operator = if self.consume_two(b'>', b'=') {
            Some(">=")
        } else if self.consume_two(b'<', b'=') {
            Some("<=")
        } else if self.consume_two(b'<', b'>') {
            Some("<>")
        } else if self.consume(b'>') {
            Some(">")
        } else if self.consume(b'<') {
            Some("<")
        } else if self.consume(b'=') {
            Some("=")
        } else {
            None
        };

        let Some(operator) = operator else {
            return Ok(left);
        };
        let right = self.parse_additive()?;
        let left = left.as_int()?;
        let right = right.as_int()?;
        Ok(FormulaValue::Bool(match operator {
            ">" => left > right,
            ">=" => left >= right,
            "<" => left < right,
            "<=" => left <= right,
            "=" => left == right,
            "<>" => left != right,
            _ => unreachable!("operator is constrained above"),
        }))
    }

    fn parse_additive(&mut self) -> Result<FormulaValue, String> {
        let mut value = self.parse_term()?.as_int()?;
        loop {
            self.skip_ws();
            if self.consume(b'+') {
                value += self.parse_term()?.as_int()?;
            } else if self.consume(b'-') {
                value -= self.parse_term()?.as_int()?;
            } else {
                return Ok(FormulaValue::Int(value));
            }
        }
    }

    fn parse_term(&mut self) -> Result<FormulaValue, String> {
        let mut value = self.parse_factor()?.as_int()?;
        loop {
            self.skip_ws();
            if self.consume(b'*') {
                value *= self.parse_factor()?.as_int()?;
            } else if self.consume(b'/') {
                let divisor = self.parse_factor()?.as_int()?;
                if divisor == 0 {
                    return Err(format!("division by zero at byte {}", self.position));
                }
                value /= divisor;
            } else {
                return Ok(FormulaValue::Int(value));
            }
        }
    }

    fn parse_factor(&mut self) -> Result<FormulaValue, String> {
        self.skip_ws();
        if self.consume(b'-') {
            return Ok(FormulaValue::Int(-self.parse_factor()?.as_int()?));
        }
        if self.consume(b'(') {
            let value = self.parse_expression()?;
            self.skip_ws();
            if self.consume(b')') {
                return Ok(value);
            }
            return Err(format!("missing ')' at byte {}", self.position));
        }
        if self.peek().is_some_and(|byte| byte.is_ascii_digit()) {
            return self.parse_number().map(FormulaValue::Int);
        }
        if self
            .peek()
            .is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'_')
        {
            let name = self.parse_name()?;
            if name.eq_ignore_ascii_case("IF") {
                return self.parse_if_call();
            }
            return self
                .environment
                .iter()
                .find_map(|(candidate, value)| (*candidate == name).then_some(*value))
                .ok_or_else(|| format!("unknown name {name}"));
        }
        Err(format!("expected factor at byte {}", self.position))
    }

    fn parse_if_call(&mut self) -> Result<FormulaValue, String> {
        self.skip_ws();
        if !self.consume(b'(') {
            return Err(format!("expected '(' after IF at byte {}", self.position));
        }
        let condition = self.parse_expression()?.as_bool()?;
        self.skip_ws();
        if !self.consume(b',') {
            return Err(format!("expected first IF comma at byte {}", self.position));
        }
        let true_value = self.parse_expression()?;
        self.skip_ws();
        if !self.consume(b',') {
            return Err(format!(
                "expected second IF comma at byte {}",
                self.position
            ));
        }
        let false_value = self.parse_expression()?;
        self.skip_ws();
        if !self.consume(b')') {
            return Err(format!("missing ')' after IF at byte {}", self.position));
        }
        Ok(if condition { true_value } else { false_value })
    }

    fn parse_number(&mut self) -> Result<i64, String> {
        let start = self.position;
        while self.peek().is_some_and(|byte| byte.is_ascii_digit()) {
            self.position += 1;
        }
        std::str::from_utf8(&self.input[start..self.position])
            .map_err(|err| err.to_string())?
            .parse::<i64>()
            .map_err(|err| err.to_string())
    }

    fn parse_name(&mut self) -> Result<String, String> {
        let start = self.position;
        while self
            .peek()
            .is_some_and(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        {
            self.position += 1;
        }
        std::str::from_utf8(&self.input[start..self.position])
            .map(str::to_string)
            .map_err(|err| err.to_string())
    }

    fn skip_ws(&mut self) {
        while self.peek().is_some_and(|byte| byte.is_ascii_whitespace()) {
            self.position += 1;
        }
    }

    fn consume(&mut self, expected: u8) -> bool {
        if self.peek() == Some(expected) {
            self.position += 1;
            true
        } else {
            false
        }
    }

    fn consume_two(&mut self, first: u8, second: u8) -> bool {
        if self.input.get(self.position..self.position + 2) == Some([first, second].as_slice()) {
            self.position += 2;
            true
        } else {
            false
        }
    }

    fn peek(&self) -> Option<u8> {
        self.input.get(self.position).copied()
    }
}

fn formula_value_json(value: Option<&FormulaValue>) -> Value {
    match value {
        Some(FormulaValue::Int(value)) => json!(value),
        Some(FormulaValue::Bool(value)) => json!(value),
        None => Value::Null,
    }
}

fn w041_independent_formula_evaluator_rows() -> Vec<Value> {
    w041_independent_formula_cases()
        .into_iter()
        .map(|case| {
            let result = eval_formula_fragment(case.expression, case.environment);
            let (actual_value, error) = match result {
                Ok(value) => (Some(value), None),
                Err(err) => (None, Some(err)),
            };
            let environment = case
                .environment
                .iter()
                .map(|(name, value)| {
                    json!({
                        "name": name,
                        "value": formula_value_json(Some(value))
                    })
                })
                .collect::<Vec<_>>();
            json!({
                "case_id": case.case_id,
                "expression": case.expression,
                "environment": environment,
                "expected_value": formula_value_json(Some(&case.expected)),
                "actual_value": formula_value_json(actual_value.as_ref()),
                "error": error,
                "matches_expected": actual_value == Some(case.expected),
                "implementation_path": "src/oxcalc-tracecalc/src/diversity_seam.rs::eval_formula_fragment",
                "implementation_kind": "broadened_independent_formula_fragment_evaluator",
                "scope_limit": "integer names and literals with +, -, *, /, parentheses, comparisons, and IF(condition, true_expr, false_expr)",
                "independent_from": ["TraceCalc", "optimized_core", "TreeCalc", "OxFml", "OxFunc"]
            })
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn w041_source_rows(
    w041_obligation_summary: &Value,
    w041_obligation_map: &Value,
    w040_formatting_intake: &Value,
    w040_diversity: &Value,
    w040_implementation: &Value,
    w040_decision: &Value,
    w041_conformance: &Value,
    w041_treecalc: &Value,
    w041_rust: &Value,
    w041_lean_tla: &Value,
    w041_stage2: &Value,
    w041_operated: &Value,
    w041_cross_engine_service: &Value,
    w041_alert_dispatch: &Value,
) -> Vec<Value> {
    vec![
        json!({
            "row_id": "source.w041_successor_obligation_map",
            "artifact": W041_OBLIGATION_MAP,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": 0,
            "promotion_guard": text_at(w041_obligation_summary, "status") == "residual_successor_obligation_map_validated"
                && number_at(w041_obligation_map, "obligation_count") == 28
                && bool_at(w041_obligation_summary, "oxfml_formatting_update_incorporated"),
            "semantic_state": "w041_successor_obligations_bound_with_formatting_guard"
        }),
        json!({
            "row_id": "source.w040_diversity_predecessor",
            "artifact": W040_DIVERSITY_SUMMARY,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": number_at(w040_diversity, "failed_row_count"),
            "promotion_guard": !bool_at(w040_diversity, "fully_independent_evaluator_promoted")
                && !bool_at(w040_diversity, "operated_cross_engine_differential_service_promoted")
                && !bool_at(w040_decision, "fully_independent_evaluator_promoted"),
            "semantic_state": "w040_diversity_predecessor_bound_without_promotion"
        }),
        json!({
            "row_id": "source.w040_independent_scalar_predecessor",
            "artifact": W040_INDEPENDENT_IMPLEMENTATION,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": number_at(w040_implementation, "failed_case_count"),
            "promotion_guard": number_at(w040_implementation, "case_count") == 5
                && number_at(w040_implementation, "match_count") == 5
                && !bool_at(w040_implementation, "fully_independent_evaluator_promoted"),
            "semantic_state": "w040_bounded_scalar_slice_narrowed_by_w041_formula_fragment"
        }),
        json!({
            "row_id": "source.w041_optimized_core_conformance",
            "artifact": W041_CONFORMANCE_SUMMARY,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": number_at(w041_conformance, "treecalc_expectation_mismatch_count"),
            "failed_row_count": number_at(w041_conformance, "failed_row_count"),
            "promotion_guard": number_at(w041_conformance, "match_promoted_count") == 0
                && number_at(w041_conformance, "exact_remaining_blocker_count") == 3,
            "semantic_state": "w041_optimized_core_conformance_bound_without_full_verification"
        }),
        json!({
            "row_id": "source.w041_treecalc_dynamic_transition",
            "artifact": W041_TREECALC_SUMMARY,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": number_at(w041_treecalc, "expectation_mismatch_count"),
            "failed_row_count": 0,
            "promotion_guard": number_at(w041_treecalc, "case_count") == 26
                && number_at(w041_treecalc, "expectation_mismatch_count") == 0,
            "semantic_state": "w041_treecalc_dynamic_transition_bound"
        }),
        json!({
            "row_id": "source.w041_rust_refinement",
            "artifact": W041_RUST_TOTALITY_SUMMARY,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": number_at(w041_rust, "failed_row_count"),
            "promotion_guard": !bool_at_path(w041_rust, &["promotion_claims", "rust_engine_totality_promoted"])
                && !bool_at_path(w041_rust, &["promotion_claims", "rust_refinement_promoted"]),
            "semantic_state": "w041_rust_totality_refinement_bound_without_full_proof_promotion"
        }),
        json!({
            "row_id": "source.w041_lean_tla_boundaries",
            "artifact": W041_LEAN_TLA_SUMMARY,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": number_at(w041_lean_tla, "failed_row_count"),
            "promotion_guard": !bool_at_path(w041_lean_tla, &["promotion_claims", "full_lean_verification_promoted"])
                && !bool_at_path(w041_lean_tla, &["promotion_claims", "full_tla_verification_promoted"]),
            "semantic_state": "w041_lean_tla_boundaries_bound_without_full_model_promotion"
        }),
        json!({
            "row_id": "source.w041_stage2_service_dependency",
            "artifact": W041_STAGE2_SUMMARY,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": number_at(w041_stage2, "failed_row_count"),
            "promotion_guard": !bool_at(w041_stage2, "stage2_policy_promoted")
                && !bool_at(w041_stage2, "stage2_promotion_candidate")
                && number_at(w041_stage2, "exact_remaining_blocker_count") == 4,
            "semantic_state": "w041_stage2_service_dependency_bound_without_policy_promotion"
        }),
        json!({
            "row_id": "source.w041_operated_assurance_service_envelope",
            "artifact": W041_OPERATED_ASSURANCE_SUMMARY,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": number_at(w041_operated, "failed_row_count"),
            "promotion_guard": bool_at(w041_operated, "file_backed_service_envelope_present")
                && bool_at(w041_operated, "retained_history_query_api_contract_present")
                && !bool_at(w041_operated, "operated_cross_engine_differential_service_promoted"),
            "semantic_state": "w041_file_backed_service_envelope_bound_without_operated_service_promotion"
        }),
        json!({
            "row_id": "source.w041_cross_engine_service_blocker",
            "artifact": W041_CROSS_ENGINE_SERVICE_REGISTER,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": 0,
            "promotion_guard": bool_at(w041_cross_engine_service, "file_backed_cross_engine_substrate_present")
                && !bool_at(w041_cross_engine_service, "operated_cross_engine_differential_service_present")
                && array_contains_string(
                    w041_cross_engine_service,
                    "blocked_service_claims",
                    "recurring_cross_engine_diff_scheduler"
                ),
            "semantic_state": "w041_cross_engine_service_blocker_retained"
        }),
        json!({
            "row_id": "source.w041_w073_typed_formatting_guard",
            "artifact": W040_FORMATTING_INTAKE,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": 0,
            "promotion_guard": text_at(w040_formatting_intake, "contract_mode") == "direct_replacement_for_aggregate_and_visualization_metadata"
                && array_len_at(w040_formatting_intake, "typed_rule_only_families") == 7
                && !bool_at(w040_formatting_intake, "w072_threshold_fallback_allowed_for_aggregate_visualization"),
            "semantic_state": "w073_typed_rule_only_formatting_guard_retained_for_diversity"
        }),
        json!({
            "row_id": "source.w041_diversity_promotion_guard",
            "artifact": W041_ALERT_DISPATCH_REGISTER,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": 0,
            "promotion_guard": !bool_at(w040_decision, "fully_independent_evaluator_promoted")
                && !bool_at(w040_decision, "operated_cross_engine_differential_service_promoted")
                && !bool_at(w041_stage2, "stage2_policy_promoted")
                && !bool_at(w041_operated, "operated_cross_engine_differential_service_promoted")
                && !bool_at(w041_alert_dispatch, "external_alert_dispatcher_promoted")
                && !bool_at(w041_alert_dispatch, "quarantine_service_promoted"),
            "semantic_state": "diversity_stage2_pack_service_promotion_guard_clean"
        }),
    ]
}

#[allow(clippy::too_many_arguments)]
fn w041_independent_evaluator_rows(
    w040_independent: &Value,
    w040_diversity: &Value,
    w041_conformance: &Value,
    w041_treecalc: &Value,
    w041_rust: &Value,
    w041_lean_tla: &Value,
    w041_stage2: &Value,
    w041_operated: &Value,
    implementation_rows: &[Value],
) -> Vec<Value> {
    let match_count = implementation_rows
        .iter()
        .filter(|row| bool_at(row, "matches_expected"))
        .count();
    vec![
        json!({
            "row_id": "independent.w041_broadened_formula_evaluator",
            "disposition_kind": "broadened_independent_implementation",
            "independence_state": "broadened_independent_formula_fragment_present",
            "evidence": "w041_independent_formula_evaluator_implementation.json",
            "case_count": implementation_rows.len(),
            "match_count": match_count,
            "promotion_consequence": "W041 broadens the independent slice beyond W040 scalar arithmetic, but still does not cover full TreeCalc/OxCalc evaluator breadth."
        }),
        json!({
            "row_id": "independent.w040_bounded_scalar_predecessor_narrowed",
            "disposition_kind": "accepted_boundary",
            "independence_state": "predecessor_bounded_scalar_slice_subsumed_by_formula_fragment",
            "evidence": W040_INDEPENDENT_ROW_SET,
            "w040_row_count": number_at(w040_independent, "row_count"),
            "w040_fully_independent_evaluator_promoted": bool_at(w040_diversity, "fully_independent_evaluator_promoted"),
            "promotion_consequence": "W040 bounded scalar evidence remains evidence lineage only."
        }),
        json!({
            "row_id": "independent.tracecalc_reference_oracle_boundary",
            "disposition_kind": "accepted_boundary",
            "independence_state": "tracecalc_reference_oracle_not_independent_runtime",
            "evidence": W041_TREECALC_SUMMARY,
            "treecalc_case_count": number_at(w041_treecalc, "case_count"),
            "promotion_consequence": "TraceCalc-backed expectations remain oracle/reference evidence, not a second full evaluator."
        }),
        json!({
            "row_id": "independent.optimized_core_same_family",
            "disposition_kind": "same_engine_family_not_independent",
            "independence_state": "optimized_core_conformance_same_authority_family",
            "evidence": W041_CONFORMANCE_SUMMARY,
            "direct_evidence_bound_count": number_at(w041_conformance, "direct_evidence_bound_count"),
            "exact_remaining_blocker_count": number_at(w041_conformance, "exact_remaining_blocker_count"),
            "promotion_consequence": "Optimized/core evidence strengthens conformance but is not independent implementation authority."
        }),
        json!({
            "row_id": "independent.treecalc_dynamic_transition_same_family",
            "disposition_kind": "same_engine_family_not_independent",
            "independence_state": "treecalc_dynamic_transition_same_oxcalc_family",
            "evidence": W041_TREECALC_SUMMARY,
            "expectation_mismatch_count": number_at(w041_treecalc, "expectation_mismatch_count"),
            "promotion_consequence": "TreeCalc dynamic-transition evidence is core conformance evidence, not independent evaluator diversity."
        }),
        json!({
            "row_id": "independent.direct_oxfml_external_formula_slice",
            "disposition_kind": "accepted_external_slice",
            "independence_state": "external_formula_evaluator_not_oxcalc_coordinator",
            "evidence": W037_DIRECT_OXFML_SUMMARY,
            "promotion_consequence": "OxFml formula evidence remains an external formula-evaluator slice, including narrow LET/LAMBDA seam evidence, not OxCalc coordinator breadth."
        }),
        json!({
            "row_id": "independent.proof_model_not_runtime_evaluator",
            "disposition_kind": "accepted_boundary",
            "independence_state": "proof_model_not_executable_runtime_evaluator",
            "evidence": W041_LEAN_TLA_SUMMARY,
            "lean_local_proof_row_count": number_at(w041_lean_tla, "local_proof_row_count"),
            "rust_refinement_row_count": number_at(w041_rust, "refinement_row_count"),
            "promotion_consequence": "Proof/model evidence supports specification and refinement, but does not supply runtime diversity."
        }),
        json!({
            "row_id": "independent.stage2_declared_profile_not_independent_runtime",
            "disposition_kind": "accepted_boundary",
            "independence_state": "stage2_policy_profiles_not_second_evaluator",
            "evidence": W041_STAGE2_SUMMARY,
            "policy_row_count": number_at(w041_stage2, "policy_row_count"),
            "stage2_policy_promoted": bool_at(w041_stage2, "stage2_policy_promoted"),
            "promotion_consequence": "Stage 2 profile evidence remains policy/replay evidence, not separate evaluator implementation authority."
        }),
        json!({
            "row_id": "independent.full_independent_evaluator_breadth_blocked",
            "disposition_kind": "exact_remaining_blocker",
            "independence_state": "full_independent_implementation_breadth_absent",
            "evidence": W041_OBLIGATION_MAP,
            "service_readiness_blocked_count": number_at(w041_operated, "service_readiness_blocked_count"),
            "promotion_consequence": "Full independent-evaluator diversity remains blocked beyond the broadened formula-fragment slice."
        }),
        json!({
            "row_id": "independent.oxfml_callable_breadth_dependency",
            "disposition_kind": "exact_remaining_blocker",
            "independence_state": "callable_and_publication_breadth_dependency_retained",
            "evidence": W041_OBLIGATION_MAP,
            "promotion_consequence": "LET/LAMBDA and callable-carrier interaction remains in scope for later breadth, with OxFunc peeking only where the seam threads through OxCalc/OxFml."
        }),
    ]
}

fn w041_cross_engine_differential_rows(
    w040_cross_engine: &Value,
    w041_stage2: &Value,
    w041_operated: &Value,
    w041_retained: &Value,
    w041_cross_engine_service: &Value,
    w041_alert_dispatch: &Value,
    w040_formatting_intake: &Value,
    implementation_rows: &[Value],
) -> Vec<Value> {
    let match_count = implementation_rows
        .iter()
        .filter(|row| bool_at(row, "matches_expected"))
        .count();
    vec![
        json!({
            "row_id": "cross_engine.w041_broadened_formula_differential",
            "disposition_kind": "accepted_boundary",
            "service_state": "file_backed_broadened_formula_rows",
            "evidence": "w041_independent_formula_evaluator_implementation.json",
            "case_count": implementation_rows.len(),
            "match_count": match_count,
            "promotion_consequence": "Broadened formula rows can feed later cross-engine service work but are not an operated service."
        }),
        json!({
            "row_id": "cross_engine.w040_file_backed_differential_predecessor",
            "disposition_kind": "accepted_boundary",
            "service_state": "file_backed_not_operated",
            "evidence": W040_CROSS_ENGINE_DIVERSITY,
            "w040_row_count": number_at(w040_cross_engine, "row_count"),
            "promotion_consequence": "W040 file-backed diversity remains evidence lineage, not service promotion."
        }),
        json!({
            "row_id": "cross_engine.w041_service_envelope_contract",
            "disposition_kind": "service_contract_boundary",
            "service_state": "file_backed_service_envelope_only",
            "evidence": W041_OPERATED_ASSURANCE_SUMMARY,
            "service_envelope_row_count": number_at(w041_operated, "service_envelope_row_count"),
            "file_backed_service_envelope_present": bool_at(w041_operated, "file_backed_service_envelope_present"),
            "promotion_consequence": "The service envelope records rows needed by a future operated service without claiming operation."
        }),
        json!({
            "row_id": "cross_engine.w041_retained_history_query_support",
            "disposition_kind": "supporting_evidence_not_service",
            "service_state": "supporting_history_query_and_correlation_only",
            "evidence": W041_RETAINED_HISTORY_QUERY,
            "query_register_row_count": number_at(w041_retained, "query_register_row_count"),
            "replay_correlation_row_count": number_at(w041_retained, "replay_correlation_row_count"),
            "promotion_consequence": "Retained query/correlation artifacts support later differential service operation but are not themselves a service."
        }),
        json!({
            "row_id": "cross_engine.w041_stage2_service_dependency",
            "disposition_kind": "service_dependency_blocked",
            "service_state": "blocked",
            "evidence": W041_STAGE2_SUMMARY,
            "stage2_policy_promoted": bool_at(w041_stage2, "stage2_policy_promoted"),
            "exact_remaining_blocker_count": number_at(w041_stage2, "exact_remaining_blocker_count"),
            "promotion_consequence": "Stage 2 service-dependent diversity remains blocked by operated service and pack-governance dependencies."
        }),
        json!({
            "row_id": "cross_engine.w041_cross_engine_service_register",
            "disposition_kind": "service_substrate_bound",
            "service_state": if bool_at(w041_cross_engine_service, "operated_cross_engine_differential_service_present") { "present" } else { "blocked" },
            "evidence": W041_CROSS_ENGINE_SERVICE_REGISTER,
            "file_backed_cross_engine_substrate_present": bool_at(w041_cross_engine_service, "file_backed_cross_engine_substrate_present"),
            "operated_cross_engine_differential_service_present": bool_at(w041_cross_engine_service, "operated_cross_engine_differential_service_present"),
            "promotion_consequence": "The W041 service register records the operated-service blocker exactly."
        }),
        json!({
            "row_id": "cross_engine.w041_mismatch_triage_and_quarantine_contract",
            "disposition_kind": "exact_remaining_blocker",
            "service_state": "blocked",
            "evidence": W041_ALERT_DISPATCH_REGISTER,
            "evaluated_rule_count": number_at(w041_alert_dispatch, "evaluated_rule_count"),
            "external_alert_dispatcher_promoted": bool_at(w041_alert_dispatch, "external_alert_dispatcher_promoted"),
            "quarantine_service_promoted": bool_at(w041_alert_dispatch, "quarantine_service_promoted"),
            "promotion_consequence": "Local alert/quarantine rules are evaluated, but no external mismatch triage/quarantine service is operated."
        }),
        json!({
            "row_id": "cross_engine.w073_typed_only_formatting_guard",
            "disposition_kind": "accepted_boundary",
            "service_state": "observable_guard_retained",
            "evidence": W040_FORMATTING_INTAKE,
            "typed_rule_only_family_count": array_len_at(w040_formatting_intake, "typed_rule_only_families"),
            "w072_threshold_fallback_allowed": bool_at(w040_formatting_intake, "w072_threshold_fallback_allowed_for_aggregate_visualization"),
            "promotion_consequence": "Diversity comparisons must preserve the W073 typed_rule-only aggregate/visualization formatting guard."
        }),
        json!({
            "row_id": "cross_engine.pack_governance_dependency",
            "disposition_kind": "exact_remaining_blocker",
            "service_state": "blocked",
            "evidence": W041_OBLIGATION_MAP,
            "promotion_consequence": "Pack/C5 and release-grade diversity remain unavailable while operated differential service and independent breadth are blocked."
        }),
    ]
}

fn w041_mismatch_authority_rows(
    w040_authority: &Value,
    w041_conformance: &Value,
    w041_operated: &Value,
    w041_cross_engine_service: &Value,
    w041_alert_dispatch: &Value,
    implementation_rows: &[Value],
) -> Vec<Value> {
    let match_count = implementation_rows
        .iter()
        .filter(|row| bool_at(row, "matches_expected"))
        .count();
    vec![
        json!({
            "row_id": "authority.w041_broadened_independent_formula_evaluator",
            "disposition_kind": "broadened_independent_implementation",
            "authority_state": "broadened_independent_formula_fragment_runtime",
            "evidence": "w041_independent_formula_evaluator_implementation.json",
            "case_count": implementation_rows.len(),
            "match_count": match_count,
            "authority_limit": "formula fragment only; no scheduling, publication, effect, callable-carrier, or full OxFml/OxFunc kernel authority"
        }),
        json!({
            "row_id": "authority.tracecalc_correctness_oracle",
            "disposition_kind": "accepted_boundary",
            "authority_state": "reference_oracle",
            "evidence": W041_TREECALC_SUMMARY,
            "authority_limit": "oracle authority for covered observable behavior only"
        }),
        json!({
            "row_id": "authority.optimized_core_same_implementation_family",
            "disposition_kind": "same_authority_family",
            "authority_state": "not_independent",
            "evidence": W041_CONFORMANCE_SUMMARY,
            "exact_remaining_blocker_count": number_at(w041_conformance, "exact_remaining_blocker_count"),
            "authority_limit": "optimized/core conformance does not supply separate implementation authority"
        }),
        json!({
            "row_id": "authority.direct_oxfml_external_formula_evaluator",
            "disposition_kind": "accepted_external_slice",
            "authority_state": "external_formula_evaluator_only",
            "evidence": W037_DIRECT_OXFML_SUMMARY,
            "authority_limit": "external evaluator authority is limited to the consumed OxFml seam and does not implement OxCalc scheduling/publication"
        }),
        json!({
            "row_id": "authority.service_envelope_file_backed_authority",
            "disposition_kind": "service_contract_boundary",
            "authority_state": "file_backed_artifact_authority",
            "evidence": W041_OPERATED_ASSURANCE_SUMMARY,
            "service_envelope_row_count": number_at(w041_operated, "service_envelope_row_count"),
            "authority_limit": "file-backed row agreement and retained artifacts cannot substitute for an operated differential service"
        }),
        json!({
            "row_id": "authority.mismatch_router_contract",
            "disposition_kind": "service_contract_boundary",
            "authority_state": "local_router_contract_not_service",
            "evidence": W041_ALERT_DISPATCH_REGISTER,
            "evaluated_rule_count": number_at(w041_alert_dispatch, "evaluated_rule_count"),
            "authority_limit": "local alert dispatch classification is a contract row set, not an external quarantine service"
        }),
        json!({
            "row_id": "authority.operated_cross_engine_service",
            "disposition_kind": "exact_remaining_blocker",
            "authority_state": "blocked_no_operated_service",
            "evidence": W041_CROSS_ENGINE_SERVICE_REGISTER,
            "operated_cross_engine_differential_service_present": bool_at(w041_cross_engine_service, "operated_cross_engine_differential_service_present"),
            "authority_limit": "no recurring service endpoint, retained service history, or service-level mismatch action path exists"
        }),
        json!({
            "row_id": "authority.full_independent_evaluator_breadth",
            "disposition_kind": "exact_remaining_blocker",
            "authority_state": "blocked_beyond_formula_fragment",
            "evidence": "w041_independent_formula_evaluator_implementation.json",
            "authority_limit": "broadened formula fragment does not cover TreeCalc scheduling, references, publication, effects, full OxFml seam breadth, or callable metadata"
        }),
        json!({
            "row_id": "authority.release_grade_diversity_promotion",
            "disposition_kind": "exact_remaining_blocker",
            "authority_state": "blocked_no_release_grade_authority",
            "evidence": W040_DIFFERENTIAL_AUTHORITY,
            "w040_authority_row_count": number_at(w040_authority, "row_count"),
            "exact_service_blocker_count": number_at(w041_operated, "exact_service_blocker_count"),
            "authority_limit": "release-grade diversity authority remains unavailable while full independent implementation and operated service blockers remain"
        }),
    ]
}

fn w041_exact_blockers(
    w040_blockers: &Value,
    w041_stage2_blockers: &Value,
    w041_service_blockers: &Value,
    w041_cross_engine_service: &Value,
) -> Vec<Value> {
    vec![
        json!({
            "blocker_id": "w041_diversity.full_independent_evaluator_breadth_absent",
            "owner": "calc-sui.7",
            "status_after_run": "exact_remaining_blocker",
            "evidence": "w041_independent_formula_evaluator_implementation.json",
            "predecessor_blocker_present": row_with_field_exists(
                w040_blockers,
                "blocker_id",
                "w040_diversity.full_independent_evaluator_breadth_absent"
            ),
            "reason": "W041 introduces a broadened independent formula-fragment evaluator, but full TreeCalc/OxCalc evaluator breadth remains absent.",
            "promotion_consequence": "fully independent evaluator diversity remains unpromoted"
        }),
        json!({
            "blocker_id": "w041_diversity.operated_cross_engine_differential_service_absent",
            "owner": "calc-sui.6; calc-sui.7",
            "status_after_run": "exact_remaining_blocker",
            "evidence": W041_CROSS_ENGINE_SERVICE_REGISTER,
            "file_backed_cross_engine_substrate_present": bool_at(w041_cross_engine_service, "file_backed_cross_engine_substrate_present"),
            "operated_cross_engine_differential_service_present": bool_at(w041_cross_engine_service, "operated_cross_engine_differential_service_present"),
            "reason": "W041 service artifacts are file-backed and do not operate a recurring cross-engine differential service.",
            "promotion_consequence": "operated cross-engine diversity service remains unpromoted"
        }),
        json!({
            "blocker_id": "w041_diversity.mismatch_triage_and_quarantine_service_absent",
            "owner": "calc-sui.6; calc-sui.7",
            "status_after_run": "exact_remaining_blocker",
            "evidence": W041_OPERATED_SERVICE_BLOCKERS,
            "service_blocker_rows": row_count_at(w041_service_blockers),
            "reason": "Local alert/quarantine rows are evaluated, but no external mismatch triage/quarantine service is operated.",
            "promotion_consequence": "service-level diversity assurance remains unpromoted"
        }),
        json!({
            "blocker_id": "w041_diversity.stage2_operated_differential_dependency_absent",
            "owner": "calc-sui.5; calc-sui.7",
            "status_after_run": "exact_remaining_blocker",
            "evidence": W041_STAGE2_BLOCKERS,
            "stage2_blocker_rows": row_count_at(w041_stage2_blockers),
            "reason": "Stage 2 diversity still needs production policy, partition soundness, pack governance, and operated differential service evidence before strategy-level diversity promotion.",
            "promotion_consequence": "Stage 2 diversity remains a blocked dependency for release-grade promotion"
        }),
        json!({
            "blocker_id": "w041_diversity.oxfml_callable_breadth_dependency_absent",
            "owner": "calc-sui.7; calc-sui.8",
            "status_after_run": "exact_remaining_blocker",
            "evidence": W041_OBLIGATION_MAP,
            "reason": "The independent fragment covers a formula slice, but does not settle broad OxFml display publication, callable-carrier sufficiency, or LET/LAMBDA threading through OxFunc-adjacent behavior.",
            "promotion_consequence": "broad OxFml callable/display publication remains a successor dependency"
        }),
        json!({
            "blocker_id": "w041_diversity.release_grade_promotion_authority_absent",
            "owner": "calc-sui.10",
            "status_after_run": "exact_remaining_blocker",
            "evidence": W041_OBLIGATION_MAP,
            "reason": "Release-grade diversity cannot be claimed from bounded formula rows, proxy evidence, file-backed rows, shared projections, or retained blockers.",
            "promotion_consequence": "release-grade verification, pack/C5, and diversity promotion remain unavailable to this bead"
        }),
    ]
}

fn w041_diversity_validation_failures(
    implementation_rows: &[Value],
    independent_rows: &[Value],
    cross_engine_rows: &[Value],
    authority_rows: &[Value],
    blockers: &[Value],
) -> Vec<String> {
    let mut failures = Vec::new();
    if implementation_rows.len() < 8
        || implementation_rows
            .iter()
            .any(|row| !bool_at(row, "matches_expected"))
    {
        failures.push("w041_diversity.independent_formula_case_mismatch".to_string());
    }
    if !independent_rows.iter().any(|row| {
        row.get("independence_state").and_then(Value::as_str)
            == Some("broadened_independent_formula_fragment_present")
    }) {
        failures.push("w041_diversity.broadened_formula_row_missing".to_string());
    }
    if !independent_rows.iter().any(|row| {
        row.get("independence_state").and_then(Value::as_str)
            == Some("full_independent_implementation_breadth_absent")
    }) {
        failures.push("w041_diversity.full_breadth_blocker_row_missing".to_string());
    }
    if !cross_engine_rows
        .iter()
        .any(|row| row.get("service_state").and_then(Value::as_str) == Some("blocked"))
    {
        failures.push("w041_diversity.blocked_cross_engine_service_row_missing".to_string());
    }
    if !authority_rows.iter().any(|row| {
        row.get("authority_state").and_then(Value::as_str) == Some("blocked_no_operated_service")
    }) {
        failures.push("w041_diversity.operated_service_authority_blocker_missing".to_string());
    }
    if blockers.len() < 6 {
        failures.push("w041_diversity.exact_blocker_count_below_gate".to_string());
    }
    if independent_rows
        .iter()
        .chain(cross_engine_rows.iter())
        .chain(authority_rows.iter())
        .any(|row| row.get("disposition_kind").and_then(Value::as_str) == Some("promoted"))
    {
        failures.push("w041_diversity.unexpected_promotion_row".to_string());
    }
    failures
}

#[allow(clippy::too_many_arguments)]
fn w040_source_rows(
    w040_direct_summary: &Value,
    w040_direct_map: &Value,
    w040_formatting_intake: &Value,
    w039_diversity: &Value,
    w039_decision: &Value,
    w040_conformance: &Value,
    w040_treecalc: &Value,
    w040_rust: &Value,
    w040_lean_tla: &Value,
    w040_stage2: &Value,
    w040_operated: &Value,
) -> Vec<Value> {
    vec![
        json!({
            "row_id": "source.w040_direct_obligation_map",
            "artifact": W040_DIRECT_OBLIGATION_MAP,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": 0,
            "promotion_guard": text_at(w040_direct_summary, "status") == "direct_verification_obligation_map_validated"
                && number_at(w040_direct_map, "obligation_count") == 23,
            "semantic_state": "w040_diversity_obligations_bound"
        }),
        json!({
            "row_id": "source.w040_w073_typed_formatting_guard",
            "artifact": W040_FORMATTING_INTAKE,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": 0,
            "promotion_guard": text_at(w040_formatting_intake, "contract_mode") == "direct_replacement_for_aggregate_and_visualization_metadata"
                && !bool_at(w040_formatting_intake, "w072_threshold_fallback_allowed_for_aggregate_visualization"),
            "semantic_state": "w073_typed_only_formatting_guard_bound"
        }),
        json!({
            "row_id": "source.w039_diversity_predecessor",
            "artifact": W039_DIVERSITY_SUMMARY,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": number_at(w039_diversity, "failed_row_count"),
            "promotion_guard": !bool_at(w039_decision, "fully_independent_evaluator_promoted")
                && !bool_at(w039_decision, "operated_cross_engine_differential_service_promoted"),
            "semantic_state": "w039_diversity_predecessor_bound_without_promotion"
        }),
        json!({
            "row_id": "source.w040_optimized_core_conformance",
            "artifact": W040_CONFORMANCE_SUMMARY,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": number_at(w040_conformance, "treecalc_expectation_mismatch_count"),
            "failed_row_count": number_at(w040_conformance, "failed_row_count"),
            "promotion_guard": number_at(w040_conformance, "match_promoted_count") == 0,
            "semantic_state": "w040_optimized_core_conformance_bound_without_full_verification"
        }),
        json!({
            "row_id": "source.w040_treecalc_direct_evidence",
            "artifact": W040_TREECALC_SUMMARY,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": number_at(w040_treecalc, "expectation_mismatch_count"),
            "failed_row_count": 0,
            "promotion_guard": number_at(w040_treecalc, "case_count") == 25
                && number_at(w040_treecalc, "expectation_mismatch_count") == 0,
            "semantic_state": "w040_treecalc_direct_evidence_bound"
        }),
        json!({
            "row_id": "source.w040_rust_totality_refinement",
            "artifact": W040_RUST_TOTALITY_SUMMARY,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": number_at(w040_rust, "failed_row_count"),
            "promotion_guard": !bool_at(w040_rust, "full_rust_totality_promoted")
                && !bool_at(w040_rust, "full_refinement_promoted"),
            "semantic_state": "w040_rust_totality_refinement_bound_with_blockers"
        }),
        json!({
            "row_id": "source.w040_lean_tla_boundaries",
            "artifact": W040_LEAN_TLA_SUMMARY,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": number_at(w040_lean_tla, "failed_row_count"),
            "promotion_guard": !bool_at(w040_lean_tla, "full_lean_verification_promoted")
                && !bool_at(w040_lean_tla, "full_tla_verification_promoted"),
            "semantic_state": "w040_lean_tla_boundaries_bound_without_full_verification"
        }),
        json!({
            "row_id": "source.w040_stage2_service_dependency",
            "artifact": W040_STAGE2_SUMMARY,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": number_at(w040_stage2, "failed_row_count"),
            "promotion_guard": !bool_at(w040_stage2, "stage2_policy_promoted"),
            "semantic_state": "w040_stage2_service_dependency_bound"
        }),
        json!({
            "row_id": "source.w040_operated_assurance_service_artifacts",
            "artifact": W040_OPERATED_ASSURANCE_SUMMARY,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": number_at(w040_operated, "failed_row_count"),
            "promotion_guard": bool_at(w040_operated, "retained_history_artifact_store_present")
                && bool_at(w040_operated, "retained_history_query_register_present")
                && !bool_at(w040_operated, "operated_cross_engine_differential_service_promoted")
                && !bool_at(w040_operated, "retained_history_service_promoted"),
            "semantic_state": "w040_service_artifacts_bound_without_operated_service_promotion"
        }),
        json!({
            "row_id": "source.w040_promotion_guard",
            "artifact": W040_OPERATED_ASSURANCE_SUMMARY,
            "missing_artifact_count": 0,
            "unexpected_mismatch_count": 0,
            "failed_row_count": 0,
            "promotion_guard": !bool_at(w040_operated, "operated_cross_engine_differential_service_promoted")
                && !bool_at(w040_operated, "operated_continuous_assurance_service_promoted")
                && !bool_at(w040_operated, "retained_history_service_promoted"),
            "semantic_state": "diversity_service_promotion_guard_clean"
        }),
    ]
}

fn w040_independent_scalar_evaluator_rows() -> Vec<Value> {
    independent_scalar_cases()
        .into_iter()
        .map(|case| {
            let result = eval_scalar_expression(case.expression, case.environment);
            let actual_value = result.ok();
            let matches_expected = actual_value == Some(case.expected);
            let environment = case
                .environment
                .iter()
                .map(|(name, value)| json!({ "name": name, "value": value }))
                .collect::<Vec<_>>();
            json!({
                "case_id": case.case_id,
                "expression": case.expression,
                "environment": environment,
                "expected_value": case.expected,
                "actual_value": actual_value,
                "matches_expected": matches_expected,
                "implementation_path": "src/oxcalc-tracecalc/src/diversity_seam.rs::eval_scalar_expression",
                "implementation_kind": "bounded_independent_scalar_arithmetic_evaluator",
                "scope_limit": "integer literals, names, +, *, and parentheses only",
                "independent_from": ["TraceCalc", "optimized_core", "OxFml", "OxFunc"]
            })
        })
        .collect()
}

fn w040_independent_evaluator_rows(
    w039_independent: &Value,
    w039_diversity: &Value,
    w040_conformance: &Value,
    w040_treecalc: &Value,
    w040_rust: &Value,
    w040_lean_tla: &Value,
    implementation_rows: &[Value],
) -> Vec<Value> {
    vec![
        json!({
            "row_id": "independent.w040_bounded_scalar_evaluator",
            "disposition_kind": "bounded_independent_implementation",
            "independence_state": "bounded_independent_scalar_implementation_present",
            "evidence": "w040_independent_scalar_evaluator_implementation.json",
            "case_count": implementation_rows.len(),
            "match_count": implementation_rows.iter().filter(|row| bool_at(row, "matches_expected")).count(),
            "promotion_consequence": "This is real independent implementation evidence for a bounded arithmetic slice, not full independent evaluator promotion."
        }),
        json!({
            "row_id": "independent.w039_predecessor_absence_narrowed",
            "disposition_kind": "accepted_boundary",
            "independence_state": "predecessor_full_independent_absence_narrowed_by_bounded_slice",
            "evidence": W039_INDEPENDENT_ROW_SET,
            "w039_independent_implementation_row_count": number_at(w039_independent, "independent_implementation_row_count"),
            "w039_fully_independent_evaluator_promoted": bool_at(w039_diversity, "fully_independent_evaluator_promoted"),
            "promotion_consequence": "W039 full absence is narrowed only for the W040 bounded scalar evaluator slice."
        }),
        json!({
            "row_id": "independent.optimized_core_same_family",
            "disposition_kind": "same_engine_family_not_independent",
            "independence_state": "optimized_core_conformance_same_authority_family",
            "evidence": W040_CONFORMANCE_SUMMARY,
            "direct_evidence_bound_count": number_at(w040_conformance, "direct_evidence_bound_count"),
            "exact_remaining_blocker_count": number_at(w040_conformance, "exact_remaining_blocker_count"),
            "promotion_consequence": "Optimized/core evidence strengthens conformance but is not independent implementation authority."
        }),
        json!({
            "row_id": "independent.treecalc_direct_evidence_same_family",
            "disposition_kind": "same_engine_family_not_independent",
            "independence_state": "treecalc_evidence_same_oxcalc_family",
            "evidence": W040_TREECALC_SUMMARY,
            "case_count": number_at(w040_treecalc, "case_count"),
            "expectation_mismatch_count": number_at(w040_treecalc, "expectation_mismatch_count"),
            "promotion_consequence": "TreeCalc direct evidence is core conformance evidence, not independent evaluator diversity."
        }),
        json!({
            "row_id": "independent.rust_totality_refinement_not_diversity",
            "disposition_kind": "accepted_boundary",
            "independence_state": "proof_and_refinement_support_not_second_runtime",
            "evidence": W040_RUST_TOTALITY_SUMMARY,
            "refinement_row_count": number_at(w040_rust, "refinement_row_count"),
            "totality_boundary_count": number_at(w040_rust, "totality_boundary_count"),
            "promotion_consequence": "Rust refinement evidence supports correctness, but is not a separately implemented evaluator."
        }),
        json!({
            "row_id": "independent.lean_tla_not_runtime_evaluator",
            "disposition_kind": "accepted_boundary",
            "independence_state": "proof_model_not_executable_runtime_evaluator",
            "evidence": W040_LEAN_TLA_SUMMARY,
            "local_proof_row_count": number_at(w040_lean_tla, "local_proof_row_count"),
            "bounded_model_row_count": number_at(w040_lean_tla, "bounded_model_row_count"),
            "promotion_consequence": "Lean/TLA evidence strengthens specification and model coverage but is not runtime evaluator implementation authority."
        }),
        json!({
            "row_id": "independent.general_oxfml_formula_slice_external",
            "disposition_kind": "accepted_external_slice",
            "independence_state": "external_formula_evaluator_not_oxcalc_coordinator",
            "evidence": W037_DIRECT_OXFML_SUMMARY,
            "promotion_consequence": "OxFml remains valid external formula-evaluator evidence, but does not independently implement OxCalc scheduling/publication."
        }),
        json!({
            "row_id": "independent.full_independent_evaluator_breadth_blocked",
            "disposition_kind": "exact_remaining_blocker",
            "independence_state": "full_independent_implementation_breadth_absent",
            "evidence": W040_DIRECT_OBLIGATION_MAP,
            "promotion_consequence": "Full independent-evaluator diversity remains blocked beyond the bounded scalar implementation slice."
        }),
    ]
}

fn w040_cross_engine_differential_rows(
    w039_cross_engine: &Value,
    w040_stage2: &Value,
    w040_operated: &Value,
    w040_retained: &Value,
    w040_cross_engine_service: &Value,
    w040_formatting_intake: &Value,
    implementation_rows: &[Value],
) -> Vec<Value> {
    vec![
        json!({
            "row_id": "cross_engine.w040_bounded_scalar_independent_differential",
            "disposition_kind": "accepted_boundary",
            "service_state": "file_backed_bounded_independent_rows",
            "evidence": "w040_independent_scalar_evaluator_implementation.json",
            "case_count": implementation_rows.len(),
            "match_count": implementation_rows.iter().filter(|row| bool_at(row, "matches_expected")).count(),
            "promotion_consequence": "Bounded scalar rows can feed later differential service work but are not an operated cross-engine service."
        }),
        json!({
            "row_id": "cross_engine.w039_file_backed_differential_predecessor",
            "disposition_kind": "accepted_boundary",
            "service_state": "file_backed_not_operated",
            "evidence": W039_CROSS_ENGINE_DIVERSITY,
            "w039_row_count": number_at(w039_cross_engine, "row_count"),
            "promotion_consequence": "W039 file-backed diversity remains evidence lineage, not service promotion."
        }),
        json!({
            "row_id": "cross_engine.w040_stage2_service_dependency",
            "disposition_kind": "service_dependency_blocked",
            "service_state": "blocked",
            "evidence": W040_STAGE2_SUMMARY,
            "stage2_policy_promoted": bool_at(w040_stage2, "stage2_policy_promoted"),
            "exact_remaining_blocker_count": number_at(w040_stage2, "exact_remaining_blocker_count"),
            "promotion_consequence": "Stage 2 service-dependent diversity remains blocked by operated service and pack-governance dependencies."
        }),
        json!({
            "row_id": "cross_engine.w040_operated_service_artifacts",
            "disposition_kind": "service_artifacts_bound",
            "service_state": "file_backed_artifacts_without_operated_service",
            "evidence": W040_OPERATED_ASSURANCE_SUMMARY,
            "file_backed_operated_runner_present": bool_at(w040_operated, "file_backed_operated_runner_present"),
            "operated_cross_engine_differential_service_promoted": bool_at(w040_operated, "operated_cross_engine_differential_service_promoted"),
            "promotion_consequence": "W040 service artifacts improve evidence retention and queryability but do not operate a differential service."
        }),
        json!({
            "row_id": "cross_engine.w040_cross_engine_service_register",
            "disposition_kind": "service_substrate_bound",
            "service_state": if bool_at(w040_cross_engine_service, "operated_cross_engine_differential_service_present") { "present" } else { "blocked" },
            "evidence": W040_CROSS_ENGINE_SERVICE_REGISTER,
            "file_backed_cross_engine_substrate_present": bool_at(w040_cross_engine_service, "file_backed_cross_engine_substrate_present"),
            "operated_cross_engine_differential_service_present": bool_at(w040_cross_engine_service, "operated_cross_engine_differential_service_present"),
            "promotion_consequence": "The W040 service register records the operated-service blocker exactly."
        }),
        json!({
            "row_id": "cross_engine.w040_retained_history_query_support",
            "disposition_kind": "supporting_evidence_not_service",
            "service_state": "supporting_history_and_query_only",
            "evidence": W040_RETAINED_HISTORY_STORE,
            "store_record_count": number_at(w040_retained, "store_record_count"),
            "query_register_row_count": number_at(w040_retained, "query_register_row_count"),
            "replay_correlation_row_count": number_at(w040_retained, "replay_correlation_row_count"),
            "promotion_consequence": "Retained query/correlation artifacts support later differential service operation but are not themselves a service."
        }),
        json!({
            "row_id": "cross_engine.w073_typed_only_formatting_guard",
            "disposition_kind": "accepted_boundary",
            "service_state": "observable_guard_retained",
            "evidence": W040_FORMATTING_INTAKE,
            "w072_threshold_fallback_allowed": bool_at(w040_formatting_intake, "w072_threshold_fallback_allowed_for_aggregate_visualization"),
            "promotion_consequence": "Diversity comparisons must preserve the W073 typed-only aggregate/visualization formatting guard."
        }),
        json!({
            "row_id": "cross_engine.mismatch_triage_and_quarantine_service",
            "disposition_kind": "exact_remaining_blocker",
            "service_state": "blocked",
            "evidence": W040_OPERATED_ASSURANCE_SUMMARY,
            "alert_decision_count": number_at(w040_operated, "alert_decision_count"),
            "quarantine_decision_count": number_at(w040_operated, "quarantine_decision_count"),
            "promotion_consequence": "Cross-engine diversity needs an operated mismatch triage/quarantine service before service-level promotion."
        }),
    ]
}

fn w040_differential_authority_rows(
    w039_authority: &Value,
    w040_conformance: &Value,
    w040_operated: &Value,
    w040_cross_engine_service: &Value,
    implementation_rows: &[Value],
) -> Vec<Value> {
    vec![
        json!({
            "row_id": "authority.w040_bounded_independent_scalar_evaluator",
            "disposition_kind": "bounded_independent_implementation",
            "authority_state": "bounded_independent_runtime",
            "evidence": "w040_independent_scalar_evaluator_implementation.json",
            "case_count": implementation_rows.len(),
            "match_count": implementation_rows.iter().filter(|row| bool_at(row, "matches_expected")).count(),
            "authority_limit": "bounded integer scalar arithmetic only"
        }),
        json!({
            "row_id": "authority.w039_predecessor_authority_boundaries",
            "disposition_kind": "accepted_boundary",
            "authority_state": "predecessor_authority_boundaries_retained",
            "evidence": W039_DIFFERENTIAL_AUTHORITY,
            "row_count": number_at(w039_authority, "row_count"),
            "authority_limit": "W039 boundaries remain in force beyond the bounded scalar slice"
        }),
        json!({
            "row_id": "authority.optimized_core_same_implementation_family",
            "disposition_kind": "same_authority_family",
            "authority_state": "not_independent",
            "evidence": W040_CONFORMANCE_SUMMARY,
            "exact_remaining_blocker_count": number_at(w040_conformance, "exact_remaining_blocker_count"),
            "authority_limit": "optimized/core conformance does not supply separate implementation authority"
        }),
        json!({
            "row_id": "authority.file_backed_service_artifacts",
            "disposition_kind": "accepted_boundary",
            "authority_state": "file_backed_artifact_authority",
            "evidence": W040_OPERATED_ASSURANCE_SUMMARY,
            "retained_history_query_register_present": bool_at(w040_operated, "retained_history_query_register_present"),
            "authority_limit": "file-backed row agreement and retained artifacts cannot substitute for an operated differential service"
        }),
        json!({
            "row_id": "authority.operated_cross_engine_service",
            "disposition_kind": "exact_remaining_blocker",
            "authority_state": "blocked_no_operated_service",
            "evidence": W040_CROSS_ENGINE_SERVICE_REGISTER,
            "operated_cross_engine_differential_service_present": bool_at(w040_cross_engine_service, "operated_cross_engine_differential_service_present"),
            "authority_limit": "no recurring service endpoint, retained service history, or service-level mismatch action path exists"
        }),
        json!({
            "row_id": "authority.full_independent_evaluator_implementation",
            "disposition_kind": "exact_remaining_blocker",
            "authority_state": "blocked_beyond_bounded_scalar_slice",
            "evidence": "w040_independent_scalar_evaluator_implementation.json",
            "authority_limit": "bounded scalar evaluator does not cover TreeCalc scheduling, references, publication, effects, OxFml seam breadth, or callable metadata"
        }),
        json!({
            "row_id": "authority.release_grade_diversity_promotion",
            "disposition_kind": "exact_remaining_blocker",
            "authority_state": "blocked_no_release_grade_authority",
            "evidence": W040_OPERATED_ASSURANCE_SUMMARY,
            "exact_service_blocker_count": number_at(w040_operated, "exact_service_blocker_count"),
            "authority_limit": "release-grade diversity authority remains unavailable while full independent implementation and operated service blockers remain"
        }),
    ]
}

fn w040_exact_blockers(
    w039_blockers: &Value,
    w040_stage2_blockers: &Value,
    w040_service_blockers: &Value,
) -> Vec<Value> {
    vec![
        json!({
            "blocker_id": "w040_diversity.full_independent_evaluator_breadth_absent",
            "owner": "calc-tv5.7",
            "status_after_run": "exact_remaining_blocker",
            "evidence": "w040_independent_scalar_evaluator_implementation.json",
            "predecessor_blocker_count": number_at(w039_blockers, "exact_blocker_count"),
            "reason": "W040 introduces a bounded independent scalar arithmetic evaluator, but full TreeCalc/OxCalc evaluator breadth remains absent.",
            "promotion_consequence": "fully independent evaluator diversity remains unpromoted"
        }),
        json!({
            "blocker_id": "w040_diversity.operated_cross_engine_service_absent",
            "owner": "calc-tv5.7; calc-tv5.6",
            "status_after_run": "exact_remaining_blocker",
            "evidence": W040_CROSS_ENGINE_SERVICE_REGISTER,
            "service_blocker_rows": row_count_at(w040_service_blockers),
            "reason": "W040 service artifacts are file-backed and do not operate a recurring cross-engine differential service.",
            "promotion_consequence": "operated cross-engine diversity service remains unpromoted"
        }),
        json!({
            "blocker_id": "w040_diversity.stage2_differential_service_dependency_absent",
            "owner": "calc-tv5.5; calc-tv5.7",
            "status_after_run": "exact_remaining_blocker",
            "evidence": W040_STAGE2_BLOCKERS,
            "stage2_blocker_rows": row_count_at(w040_stage2_blockers),
            "reason": "Stage 2 diversity still needs production policy, partition soundness, pack governance, and operated differential service evidence before strategy-level diversity promotion.",
            "promotion_consequence": "Stage 2 diversity remains a blocked dependency for release-grade promotion"
        }),
        json!({
            "blocker_id": "w040_diversity.mismatch_triage_and_quarantine_service_absent",
            "owner": "calc-tv5.6; calc-tv5.7",
            "status_after_run": "exact_remaining_blocker",
            "evidence": W040_OPERATED_SERVICE_BLOCKERS,
            "reason": "Local alert/quarantine rows are evaluated, but no external mismatch triage/quarantine service is operated.",
            "promotion_consequence": "service-level diversity assurance remains unpromoted"
        }),
        json!({
            "blocker_id": "w040_diversity.release_grade_promotion_authority_absent",
            "owner": "calc-tv5.10",
            "status_after_run": "exact_remaining_blocker",
            "evidence": W040_DIRECT_OBLIGATION_MAP,
            "reason": "Release-grade diversity cannot be claimed from bounded scalar rows, proxy evidence, file-backed rows, shared projections, or retained blockers.",
            "promotion_consequence": "release-grade verification, pack/C5, and diversity promotion remain unavailable to this bead"
        }),
    ]
}

fn w040_diversity_validation_failures(
    implementation_rows: &[Value],
    independent_rows: &[Value],
    cross_engine_rows: &[Value],
    authority_rows: &[Value],
    blockers: &[Value],
) -> Vec<String> {
    let mut failures = Vec::new();
    if implementation_rows.is_empty()
        || implementation_rows
            .iter()
            .any(|row| !bool_at(row, "matches_expected"))
    {
        failures.push("w040_diversity.independent_scalar_case_mismatch".to_string());
    }
    if !independent_rows.iter().any(|row| {
        row.get("independence_state").and_then(Value::as_str)
            == Some("bounded_independent_scalar_implementation_present")
    }) {
        failures.push("w040_diversity.bounded_independent_row_missing".to_string());
    }
    if !independent_rows.iter().any(|row| {
        row.get("independence_state").and_then(Value::as_str)
            == Some("full_independent_implementation_breadth_absent")
    }) {
        failures.push("w040_diversity.full_breadth_blocker_row_missing".to_string());
    }
    if !cross_engine_rows
        .iter()
        .any(|row| row.get("service_state").and_then(Value::as_str) == Some("blocked"))
    {
        failures.push("w040_diversity.blocked_cross_engine_service_row_missing".to_string());
    }
    if !authority_rows.iter().any(|row| {
        row.get("authority_state").and_then(Value::as_str) == Some("blocked_no_operated_service")
    }) {
        failures.push("w040_diversity.operated_service_authority_blocker_missing".to_string());
    }
    if blockers.len() < 5 {
        failures.push("w040_diversity.exact_blocker_count_below_gate".to_string());
    }
    if independent_rows
        .iter()
        .chain(cross_engine_rows.iter())
        .chain(authority_rows.iter())
        .any(|row| row.get("disposition_kind").and_then(Value::as_str) == Some("promoted"))
    {
        failures.push("w040_diversity.unexpected_promotion_row".to_string());
    }
    failures
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

fn seam_validation_failures(seam_rows: &[Value]) -> Vec<String> {
    seam_rows
        .iter()
        .filter_map(|row| {
            let row_id = text_at(row, "row_id");
            let watch_state = row.get("watch_state").and_then(Value::as_str);
            (watch_state == Some("missing_blocker")).then(|| format!("{row_id}.missing_blocker"))
        })
        .collect()
}

fn row_with_field_exists(value: &Value, field: &str, expected: &str) -> bool {
    value
        .get("rows")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .any(|row| row.get(field).and_then(Value::as_str) == Some(expected))
}

fn read_json(repo_root: &Path, relative_path: &str) -> Result<Value, DiversitySeamError> {
    let path = repo_root.join(relative_path);
    let contents =
        fs::read_to_string(&path).map_err(|source| DiversitySeamError::ReadArtifact {
            path: path.display().to_string(),
            source,
        })?;
    serde_json::from_str(&contents).map_err(|source| DiversitySeamError::ParseJson {
        path: path.display().to_string(),
        source,
    })
}

fn write_json(path: &Path, value: &Value) -> Result<(), DiversitySeamError> {
    let contents =
        serde_json::to_string_pretty(value).map_err(|source| DiversitySeamError::ParseJson {
            path: path.display().to_string(),
            source,
        })?;
    fs::write(path, format!("{contents}\n")).map_err(|source| DiversitySeamError::WriteFile {
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

fn array_contains_string(value: &Value, key: &str, expected: &str) -> bool {
    value
        .get(key)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .any(|item| item.as_str() == Some(expected))
}

fn row_count_at(value: &Value) -> usize {
    array_len_at(value, "rows")
}

fn bool_at(value: &Value, key: &str) -> bool {
    value.get(key).and_then(Value::as_bool).unwrap_or(false)
}

fn bool_at_path(value: &Value, path: &[&str]) -> bool {
    path.iter()
        .try_fold(value, |current, key| current.get(*key))
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

fn relative_artifact_path(parts: &[&str]) -> String {
    parts.join("/")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn diversity_seam_runner_classifies_w038_seams_without_promotion() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let run_id = format!("test-w038-diversity-seam-{}", std::process::id());
        let artifact_root = repo_root.join(format!(
            "docs/test-runs/core-engine/diversity-seam/{run_id}"
        ));
        let cleanup = || {
            if artifact_root.exists() {
                let _ = fs::remove_dir_all(&artifact_root);
            }
        };

        cleanup();
        let summary = DiversitySeamRunner::new()
            .execute(&repo_root, &run_id)
            .unwrap();

        assert_eq!(summary.source_evidence_row_count, 7);
        assert_eq!(summary.diversity_disposition_row_count, 5);
        assert_eq!(summary.seam_watch_row_count, 8);
        assert_eq!(summary.aligned_seam_watch_row_count, 7);
        assert_eq!(summary.accepted_boundary_count, 3);
        assert_eq!(summary.exact_blocker_count, 4);
        assert_eq!(summary.failed_row_count, 0);
        assert!(!summary.fully_independent_evaluator_promoted);

        let validation = read_json(
            &repo_root,
            &format!("docs/test-runs/core-engine/diversity-seam/{run_id}/validation.json"),
        )
        .unwrap();
        assert_eq!(validation["status"], "w038_diversity_seam_packet_valid");

        let decision = read_json(
            &repo_root,
            &format!("docs/test-runs/core-engine/diversity-seam/{run_id}/promotion_decision.json"),
        )
        .unwrap();
        assert_eq!(decision["fully_independent_evaluator_promoted"], false);
        assert_eq!(decision["oxfml_handoff_triggered"], false);

        cleanup();
    }

    #[test]
    fn diversity_seam_runner_classifies_w039_independence_without_promotion() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let run_id = format!("test-w039-diversity-seam-{}", std::process::id());
        let artifact_root = repo_root.join(format!(
            "docs/test-runs/core-engine/diversity-seam/{run_id}"
        ));
        let cleanup = || {
            if artifact_root.exists() {
                let _ = fs::remove_dir_all(&artifact_root);
            }
        };

        cleanup();
        let summary = DiversitySeamRunner::new()
            .execute(&repo_root, &run_id)
            .unwrap();

        assert_eq!(summary.schema_version, W039_RUN_SUMMARY_SCHEMA_V1);
        assert_eq!(summary.source_evidence_row_count, 10);
        assert_eq!(summary.diversity_disposition_row_count, 7);
        assert_eq!(summary.seam_watch_row_count, 7);
        assert_eq!(summary.accepted_boundary_count, 10);
        assert_eq!(summary.exact_blocker_count, 6);
        assert_eq!(summary.failed_row_count, 0);
        assert!(!summary.fully_independent_evaluator_promoted);

        let validation = read_json(
            &repo_root,
            &format!("docs/test-runs/core-engine/diversity-seam/{run_id}/validation.json"),
        )
        .unwrap();
        assert_eq!(
            validation["status"],
            "w039_independent_evaluator_cross_engine_diversity_packet_valid"
        );
        assert_eq!(
            validation["operated_cross_engine_differential_service_promoted"],
            false
        );

        let decision = read_json(
            &repo_root,
            &format!("docs/test-runs/core-engine/diversity-seam/{run_id}/promotion_decision.json"),
        )
        .unwrap();
        assert_eq!(decision["fully_independent_evaluator_promoted"], false);
        assert_eq!(
            decision["operated_cross_engine_differential_service_promoted"],
            false
        );
        assert_eq!(decision["w073_formatting_handoff_triggered"], false);

        cleanup();
    }

    #[test]
    fn diversity_seam_runner_binds_w040_bounded_independent_evaluator_without_promotion() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let run_id = format!("test-w040-diversity-seam-{}", std::process::id());
        let artifact_root = repo_root.join(format!(
            "docs/test-runs/core-engine/diversity-seam/{run_id}"
        ));
        let cleanup = || {
            if artifact_root.exists() {
                let _ = fs::remove_dir_all(&artifact_root);
            }
        };

        cleanup();
        let summary = DiversitySeamRunner::new()
            .execute(&repo_root, &run_id)
            .unwrap();

        assert_eq!(summary.schema_version, W040_RUN_SUMMARY_SCHEMA_V1);
        assert_eq!(summary.source_evidence_row_count, 10);
        assert_eq!(summary.diversity_disposition_row_count, 8);
        assert_eq!(summary.seam_watch_row_count, 8);
        assert_eq!(summary.exact_blocker_count, 5);
        assert_eq!(summary.failed_row_count, 0);
        assert!(!summary.fully_independent_evaluator_promoted);

        let implementation = read_json(
            &repo_root,
            &format!(
                "docs/test-runs/core-engine/diversity-seam/{run_id}/w040_independent_scalar_evaluator_implementation.json"
            ),
        )
        .unwrap();
        assert_eq!(implementation["case_count"], 5);
        assert_eq!(implementation["match_count"], 5);
        assert_eq!(
            implementation["fully_independent_evaluator_promoted"],
            false
        );

        let validation = read_json(
            &repo_root,
            &format!("docs/test-runs/core-engine/diversity-seam/{run_id}/validation.json"),
        )
        .unwrap();
        assert_eq!(
            validation["status"],
            "w040_bounded_independent_evaluator_diversity_packet_valid"
        );
        assert_eq!(
            validation["operated_cross_engine_differential_service_promoted"],
            false
        );

        let decision = read_json(
            &repo_root,
            &format!("docs/test-runs/core-engine/diversity-seam/{run_id}/promotion_decision.json"),
        )
        .unwrap();
        assert_eq!(
            decision["bounded_independent_scalar_evaluator_present"],
            true
        );
        assert_eq!(decision["fully_independent_evaluator_promoted"], false);
        assert_eq!(
            decision["operated_cross_engine_differential_service_promoted"],
            false
        );

        cleanup();
    }

    #[test]
    fn diversity_seam_runner_binds_w041_independent_evaluator_breadth_without_promotion() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let run_id = format!("test-w041-diversity-seam-{}", std::process::id());
        let artifact_root = repo_root.join(format!(
            "docs/test-runs/core-engine/diversity-seam/{run_id}"
        ));
        let cleanup = || {
            if artifact_root.exists() {
                let _ = fs::remove_dir_all(&artifact_root);
            }
        };

        cleanup();
        let summary = DiversitySeamRunner::new()
            .execute(&repo_root, &run_id)
            .unwrap();

        assert_eq!(summary.schema_version, W041_RUN_SUMMARY_SCHEMA_V1);
        assert_eq!(summary.source_evidence_row_count, 12);
        assert_eq!(summary.diversity_disposition_row_count, 10);
        assert_eq!(summary.seam_watch_row_count, 9);
        assert_eq!(summary.exact_blocker_count, 6);
        assert_eq!(summary.failed_row_count, 0);
        assert!(!summary.fully_independent_evaluator_promoted);

        let implementation = read_json(
            &repo_root,
            &format!(
                "docs/test-runs/core-engine/diversity-seam/{run_id}/w041_independent_formula_evaluator_implementation.json"
            ),
        )
        .unwrap();
        assert_eq!(implementation["case_count"], 8);
        assert_eq!(implementation["match_count"], 8);
        assert_eq!(
            implementation["fully_independent_evaluator_promoted"],
            false
        );

        let validation = read_json(
            &repo_root,
            &format!("docs/test-runs/core-engine/diversity-seam/{run_id}/validation.json"),
        )
        .unwrap();
        assert_eq!(
            validation["status"],
            "w041_broadened_independent_evaluator_diversity_packet_valid"
        );
        assert_eq!(
            validation["operated_cross_engine_differential_service_promoted"],
            false
        );

        let decision = read_json(
            &repo_root,
            &format!("docs/test-runs/core-engine/diversity-seam/{run_id}/promotion_decision.json"),
        )
        .unwrap();
        assert_eq!(
            decision["broadened_independent_formula_evaluator_present"],
            true
        );
        assert_eq!(decision["fully_independent_evaluator_promoted"], false);
        assert_eq!(
            decision["operated_cross_engine_differential_service_promoted"],
            false
        );
        assert_eq!(decision["exact_blocker_count"], 6);

        cleanup();
    }
}
