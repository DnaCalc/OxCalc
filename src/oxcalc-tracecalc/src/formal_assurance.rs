#![forbid(unsafe_code)]

//! W038 proof/model assumption-discharge packet emission.

use std::fs;
use std::path::Path;

use serde_json::{Value, json};
use thiserror::Error;

const FORMAL_ASSURANCE_RUN_SUMMARY_SCHEMA_V1: &str = "oxcalc.formal_assurance.run_summary.v1";
const FORMAL_ASSURANCE_SOURCE_INDEX_SCHEMA_V1: &str =
    "oxcalc.formal_assurance.source_evidence_index.v1";
const FORMAL_ASSURANCE_ASSUMPTION_LEDGER_SCHEMA_V1: &str =
    "oxcalc.formal_assurance.w038_assumption_discharge_ledger.v1";
const FORMAL_ASSURANCE_TOTALITY_REGISTER_SCHEMA_V1: &str =
    "oxcalc.formal_assurance.w038_totality_boundary_register.v1";
const FORMAL_ASSURANCE_MODEL_BOUND_REGISTER_SCHEMA_V1: &str =
    "oxcalc.formal_assurance.w038_model_bound_register.v1";
const FORMAL_ASSURANCE_BLOCKER_REGISTER_SCHEMA_V1: &str =
    "oxcalc.formal_assurance.w038_exact_proof_model_blocker_register.v1";
const FORMAL_ASSURANCE_W039_PROOF_MODEL_LEDGER_SCHEMA_V1: &str =
    "oxcalc.formal_assurance.w039_proof_model_totality_ledger.v1";
const FORMAL_ASSURANCE_W039_TOTALITY_REGISTER_SCHEMA_V1: &str =
    "oxcalc.formal_assurance.w039_totality_boundary_register.v1";
const FORMAL_ASSURANCE_W039_MODEL_BOUND_REGISTER_SCHEMA_V1: &str =
    "oxcalc.formal_assurance.w039_model_bound_register.v1";
const FORMAL_ASSURANCE_W039_BLOCKER_REGISTER_SCHEMA_V1: &str =
    "oxcalc.formal_assurance.w039_exact_proof_model_blocker_register.v1";
const FORMAL_ASSURANCE_W040_RUST_LEDGER_SCHEMA_V1: &str =
    "oxcalc.formal_assurance.w040_rust_totality_refinement_ledger.v1";
const FORMAL_ASSURANCE_W040_TOTALITY_REGISTER_SCHEMA_V1: &str =
    "oxcalc.formal_assurance.w040_rust_totality_boundary_register.v1";
const FORMAL_ASSURANCE_W040_REFINEMENT_REGISTER_SCHEMA_V1: &str =
    "oxcalc.formal_assurance.w040_rust_refinement_register.v1";
const FORMAL_ASSURANCE_W040_BLOCKER_REGISTER_SCHEMA_V1: &str =
    "oxcalc.formal_assurance.w040_rust_exact_blocker_register.v1";
const FORMAL_ASSURANCE_W040_LEAN_TLA_LEDGER_SCHEMA_V1: &str =
    "oxcalc.formal_assurance.w040_lean_tla_discharge_ledger.v1";
const FORMAL_ASSURANCE_W040_LEAN_PROOF_REGISTER_SCHEMA_V1: &str =
    "oxcalc.formal_assurance.w040_lean_proof_register.v1";
const FORMAL_ASSURANCE_W040_TLA_MODEL_REGISTER_SCHEMA_V1: &str =
    "oxcalc.formal_assurance.w040_tla_model_bound_register.v1";
const FORMAL_ASSURANCE_W040_LEAN_TLA_BLOCKER_REGISTER_SCHEMA_V1: &str =
    "oxcalc.formal_assurance.w040_lean_tla_exact_blocker_register.v1";
const FORMAL_ASSURANCE_VALIDATION_SCHEMA_V1: &str = "oxcalc.formal_assurance.validation.v1";

const W037_FORMAL_INVENTORY_RUN_ID: &str = "w037-proof-model-closure-001";
const W037_STAGE2_CRITERIA_RUN_ID: &str = "w037-stage2-deterministic-replay-criteria-001";
const W038_IMPLEMENTATION_CONFORMANCE_RUN_ID: &str =
    "w038-optimized-core-conformance-disposition-001";
const W038_TRACECALC_AUTHORITY_RUN_ID: &str = "w038-tracecalc-authority-discharge-001";
const W038_LEAN_ASSUMPTION_FILE: &str =
    "formal/lean/OxCalc/CoreEngine/W038AssumptionDischargeAndTotality.lean";
const W039_RESIDUAL_LEDGER_RUN_ID: &str = "w039-residual-successor-obligation-ledger-001";
const W039_IMPLEMENTATION_CONFORMANCE_RUN_ID: &str =
    "w039-optimized-core-exact-blocker-disposition-001";
const W039_LEAN_TOTALITY_FILE: &str =
    "formal/lean/OxCalc/CoreEngine/W039ProofModelTotalityClosure.lean";
const W040_DIRECT_OBLIGATION_RUN_ID: &str = "w040-direct-verification-obligation-map-001";
const W040_IMPLEMENTATION_CONFORMANCE_RUN_ID: &str =
    "w040-optimized-core-exact-blocker-fixes-differentials-001";
const W040_TREECALC_RUN_ID: &str = "w040-optimized-core-dynamic-release-reclassification-001";
const W040_LEAN_RUST_TOTALITY_FILE: &str =
    "formal/lean/OxCalc/CoreEngine/W040RustTotalityAndRefinement.lean";
const W040_RUST_FORMAL_ASSURANCE_RUN_ID: &str = "w040-rust-totality-refinement-proof-tranche-001";
const W040_LEAN_TLA_DISCHARGE_FILE: &str =
    "formal/lean/OxCalc/CoreEngine/W040LeanTlaFullVerificationDischarge.lean";
const W039_STAGE2_POLICY_FILE: &str =
    "formal/lean/OxCalc/CoreEngine/W039Stage2ProductionPolicy.lean";
const W040_RUST_PANIC_AUDIT_FILES: &[&str] = &[
    "src/oxcalc-core/src/coordinator.rs",
    "src/oxcalc-core/src/dependency.rs",
    "src/oxcalc-core/src/formula.rs",
    "src/oxcalc-core/src/recalc.rs",
    "src/oxcalc-core/src/structural.rs",
    "src/oxcalc-core/src/treecalc.rs",
    "src/oxcalc-core/src/treecalc_fixture.rs",
    "src/oxcalc-core/src/treecalc_runner.rs",
    "src/oxcalc-core/src/treecalc_scale.rs",
    "src/oxcalc-core/src/upstream_host.rs",
    "src/oxcalc-core/src/upstream_host_fixture.rs",
    "src/oxcalc-core/src/upstream_host_runner.rs",
];

#[derive(Debug, Error)]
pub enum FormalAssuranceError {
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
pub struct FormalAssuranceRunSummary {
    pub run_id: String,
    pub schema_version: String,
    pub assumption_row_count: usize,
    pub local_proof_row_count: usize,
    pub bounded_model_row_count: usize,
    pub accepted_external_seam_count: usize,
    pub accepted_boundary_count: usize,
    pub totality_boundary_count: usize,
    pub exact_remaining_blocker_count: usize,
    pub failed_row_count: usize,
    pub artifact_root: String,
}

#[derive(Debug, Clone, Default)]
pub struct FormalAssuranceRunner;

#[derive(Debug, Clone)]
struct AssumptionDischargeSpec {
    row_id: &'static str,
    source_id: &'static str,
    w038_obligation_id: &'static str,
    disposition_kind: &'static str,
    disposition: &'static str,
    local_checked_proof: bool,
    bounded_model: bool,
    accepted_external_seam: bool,
    accepted_boundary: bool,
    totality_boundary: bool,
    exact_remaining_blocker: bool,
    exact_remaining_blocker_bead: Option<&'static str>,
    authority_owner: &'static str,
    promotion_consequence: &'static str,
    reason: &'static str,
    evidence_paths: &'static [&'static str],
    required_evidence_checks: &'static [&'static str],
}

#[derive(Debug, Clone)]
struct EvaluatedAssumptionRow {
    row: Value,
    local_checked_proof: bool,
    bounded_model: bool,
    accepted_external_seam: bool,
    accepted_boundary: bool,
    totality_boundary: bool,
    exact_remaining_blocker: bool,
    valid: bool,
}

impl FormalAssuranceRunner {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    pub fn execute(
        &self,
        repo_root: &Path,
        run_id: &str,
    ) -> Result<FormalAssuranceRunSummary, FormalAssuranceError> {
        if run_id.contains("w040-lean-tla") {
            return self.execute_w040_lean_tla_discharge(repo_root, run_id);
        }
        if run_id.contains("w040-rust") {
            return self.execute_w040_rust_totality_refinement(repo_root, run_id);
        }
        if run_id.contains("w039") {
            return self.execute_w039(repo_root, run_id);
        }

        let relative_artifact_root = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "formal-assurance",
            run_id,
        ]);
        let artifact_root = repo_root.join(&relative_artifact_root);
        if artifact_root.exists() {
            fs::remove_dir_all(&artifact_root).map_err(|source| {
                FormalAssuranceError::RemoveDirectory {
                    path: artifact_root.display().to_string(),
                    source,
                }
            })?;
        }
        fs::create_dir_all(&artifact_root).map_err(|source| {
            FormalAssuranceError::CreateDirectory {
                path: artifact_root.display().to_string(),
                source,
            }
        })?;

        let w037_formal_summary_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "formal-inventory",
            W037_FORMAL_INVENTORY_RUN_ID,
            "run_summary.json",
        ]);
        let w037_formal_validation_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "formal-inventory",
            W037_FORMAL_INVENTORY_RUN_ID,
            "validation.json",
        ]);
        let w037_formal_blockers_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "formal-inventory",
            W037_FORMAL_INVENTORY_RUN_ID,
            "promotion_blockers.json",
        ]);
        let w037_stage2_decision_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "stage2-criteria",
            W037_STAGE2_CRITERIA_RUN_ID,
            "promotion_decision.json",
        ]);
        let w038_conformance_summary_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "implementation-conformance",
            W038_IMPLEMENTATION_CONFORMANCE_RUN_ID,
            "run_summary.json",
        ]);
        let w038_conformance_blockers_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "implementation-conformance",
            W038_IMPLEMENTATION_CONFORMANCE_RUN_ID,
            "w038_exact_remaining_blocker_register.json",
        ]);
        let w038_tracecalc_authority_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "tracecalc-authority",
            W038_TRACECALC_AUTHORITY_RUN_ID,
            "run_summary.json",
        ]);

        let w037_formal_summary = read_json(repo_root, &w037_formal_summary_path)?;
        let w037_formal_validation = read_json(repo_root, &w037_formal_validation_path)?;
        let w037_formal_blockers = read_json(repo_root, &w037_formal_blockers_path)?;
        let w037_stage2_decision = read_json(repo_root, &w037_stage2_decision_path)?;
        let w038_conformance_summary = read_json(repo_root, &w038_conformance_summary_path)?;
        let w038_conformance_blockers = read_json(repo_root, &w038_conformance_blockers_path)?;
        let w038_tracecalc_authority = read_json(repo_root, &w038_tracecalc_authority_path)?;

        let evaluated_rows = W038_ASSUMPTION_DISCHARGE_SPECS
            .iter()
            .map(|spec| {
                evaluate_assumption_row(
                    repo_root,
                    spec,
                    &w037_formal_summary,
                    &w037_formal_validation,
                    &w037_formal_blockers,
                    &w037_stage2_decision,
                    &w038_conformance_summary,
                    &w038_conformance_blockers,
                    &w038_tracecalc_authority,
                )
            })
            .collect::<Vec<_>>();

        let assumption_rows = evaluated_rows
            .iter()
            .map(|row| row.row.clone())
            .collect::<Vec<_>>();
        let totality_rows = evaluated_rows
            .iter()
            .filter(|row| row.totality_boundary)
            .map(|row| row.row.clone())
            .collect::<Vec<_>>();
        let model_bound_rows = evaluated_rows
            .iter()
            .filter(|row| row.bounded_model)
            .map(|row| row.row.clone())
            .collect::<Vec<_>>();
        let blocker_rows = evaluated_rows
            .iter()
            .filter(|row| row.exact_remaining_blocker)
            .map(|row| row.row.clone())
            .collect::<Vec<_>>();

        let local_proof_row_count = evaluated_rows
            .iter()
            .filter(|row| row.local_checked_proof)
            .count();
        let bounded_model_row_count = evaluated_rows
            .iter()
            .filter(|row| row.bounded_model)
            .count();
        let accepted_external_seam_count = evaluated_rows
            .iter()
            .filter(|row| row.accepted_external_seam)
            .count();
        let accepted_boundary_count = evaluated_rows
            .iter()
            .filter(|row| row.accepted_boundary)
            .count();
        let totality_boundary_count = totality_rows.len();
        let exact_remaining_blocker_count = blocker_rows.len();
        let failed_row_count = evaluated_rows.iter().filter(|row| !row.valid).count();
        let source_failures = source_validation_failures(
            &w037_formal_summary,
            &w037_formal_blockers,
            &w038_conformance_summary,
            &w038_conformance_blockers,
            &w038_tracecalc_authority,
        );

        let source_evidence_index_path =
            format!("{relative_artifact_root}/source_evidence_index.json");
        let assumption_ledger_path =
            format!("{relative_artifact_root}/w038_assumption_discharge_ledger.json");
        let totality_register_path =
            format!("{relative_artifact_root}/w038_totality_boundary_register.json");
        let model_bound_register_path =
            format!("{relative_artifact_root}/w038_model_bound_register.json");
        let blocker_register_path =
            format!("{relative_artifact_root}/w038_exact_proof_model_blocker_register.json");
        let validation_path = format!("{relative_artifact_root}/validation.json");

        let source_evidence_index = json!({
            "schema_version": FORMAL_ASSURANCE_SOURCE_INDEX_SCHEMA_V1,
            "run_id": run_id,
            "source_artifacts": {
                "w037_formal_summary": w037_formal_summary_path,
                "w037_formal_validation": w037_formal_validation_path,
                "w037_formal_promotion_blockers": w037_formal_blockers_path,
                "w037_stage2_promotion_decision": w037_stage2_decision_path,
                "w038_implementation_conformance_summary": w038_conformance_summary_path,
                "w038_implementation_conformance_exact_blockers": w038_conformance_blockers_path,
                "w038_tracecalc_authority_summary": w038_tracecalc_authority_path,
                "w038_lean_assumption_file": W038_LEAN_ASSUMPTION_FILE
            },
            "source_counts": {
                "w037_formal_blocker_count": counter_value(&w037_formal_blockers, "blocker_count"),
                "w037_lean_file_count": counter_value(&w037_formal_summary, "lean_file_count"),
                "w037_tla_routine_config_count": counter_value(&w037_formal_summary, "tla_routine_config_count"),
                "w038_exact_conformance_blocker_count": counter_value(&w038_conformance_blockers, "exact_remaining_blocker_count")
            }
        });

        let assumption_ledger = json!({
            "schema_version": FORMAL_ASSURANCE_ASSUMPTION_LEDGER_SCHEMA_V1,
            "run_id": run_id,
            "assumption_row_count": assumption_rows.len(),
            "rows": assumption_rows
        });
        let totality_register = json!({
            "schema_version": FORMAL_ASSURANCE_TOTALITY_REGISTER_SCHEMA_V1,
            "run_id": run_id,
            "totality_boundary_count": totality_boundary_count,
            "rows": totality_rows
        });
        let model_bound_register = json!({
            "schema_version": FORMAL_ASSURANCE_MODEL_BOUND_REGISTER_SCHEMA_V1,
            "run_id": run_id,
            "bounded_model_row_count": bounded_model_row_count,
            "rows": model_bound_rows
        });
        let blocker_register = json!({
            "schema_version": FORMAL_ASSURANCE_BLOCKER_REGISTER_SCHEMA_V1,
            "run_id": run_id,
            "exact_remaining_blocker_count": exact_remaining_blocker_count,
            "rows": blocker_rows
        });

        let mut validation_failures = evaluated_rows
            .iter()
            .filter(|row| !row.valid)
            .map(|row| {
                format!(
                    "{}_failed_checks",
                    row.row
                        .get("row_id")
                        .and_then(Value::as_str)
                        .unwrap_or("unknown_row")
                )
            })
            .collect::<Vec<_>>();
        validation_failures.extend(source_failures);
        let validation_status = if validation_failures.is_empty() {
            "formal_assurance_w038_assumption_discharge_valid"
        } else {
            "formal_assurance_w038_assumption_discharge_invalid"
        };
        let validation = json!({
            "schema_version": FORMAL_ASSURANCE_VALIDATION_SCHEMA_V1,
            "run_id": run_id,
            "status": validation_status,
            "assumption_row_count": assumption_rows.len(),
            "local_proof_row_count": local_proof_row_count,
            "bounded_model_row_count": bounded_model_row_count,
            "accepted_external_seam_count": accepted_external_seam_count,
            "accepted_boundary_count": accepted_boundary_count,
            "totality_boundary_count": totality_boundary_count,
            "exact_remaining_blocker_count": exact_remaining_blocker_count,
            "failed_row_count": failed_row_count,
            "validation_failures": validation_failures
        });
        let run_summary = json!({
            "schema_version": FORMAL_ASSURANCE_RUN_SUMMARY_SCHEMA_V1,
            "run_id": run_id,
            "artifact_root": relative_artifact_root,
            "source_evidence_index_path": source_evidence_index_path,
            "assumption_discharge_ledger_path": assumption_ledger_path,
            "totality_boundary_register_path": totality_register_path,
            "model_bound_register_path": model_bound_register_path,
            "exact_proof_model_blocker_register_path": blocker_register_path,
            "validation_path": validation_path,
            "assumption_row_count": assumption_rows.len(),
            "local_proof_row_count": local_proof_row_count,
            "bounded_model_row_count": bounded_model_row_count,
            "accepted_external_seam_count": accepted_external_seam_count,
            "accepted_boundary_count": accepted_boundary_count,
            "totality_boundary_count": totality_boundary_count,
            "exact_remaining_blocker_count": exact_remaining_blocker_count,
            "failed_row_count": failed_row_count,
            "promotion_claims": {
                "full_lean_verification_promoted": false,
                "full_tla_verification_promoted": false,
                "general_oxfunc_kernel_promoted": false,
                "stage2_policy_promoted": false,
                "pack_grade_replay_promoted": false,
                "c5_promoted": false
            }
        });

        write_json(
            &artifact_root.join("source_evidence_index.json"),
            &source_evidence_index,
        )?;
        write_json(
            &artifact_root.join("w038_assumption_discharge_ledger.json"),
            &assumption_ledger,
        )?;
        write_json(
            &artifact_root.join("w038_totality_boundary_register.json"),
            &totality_register,
        )?;
        write_json(
            &artifact_root.join("w038_model_bound_register.json"),
            &model_bound_register,
        )?;
        write_json(
            &artifact_root.join("w038_exact_proof_model_blocker_register.json"),
            &blocker_register,
        )?;
        write_json(&artifact_root.join("validation.json"), &validation)?;
        write_json(&artifact_root.join("run_summary.json"), &run_summary)?;

        Ok(FormalAssuranceRunSummary {
            run_id: run_id.to_string(),
            schema_version: FORMAL_ASSURANCE_RUN_SUMMARY_SCHEMA_V1.to_string(),
            assumption_row_count: assumption_rows.len(),
            local_proof_row_count,
            bounded_model_row_count,
            accepted_external_seam_count,
            accepted_boundary_count,
            totality_boundary_count,
            exact_remaining_blocker_count,
            failed_row_count,
            artifact_root: relative_artifact_root,
        })
    }

    fn execute_w040_lean_tla_discharge(
        &self,
        repo_root: &Path,
        run_id: &str,
    ) -> Result<FormalAssuranceRunSummary, FormalAssuranceError> {
        let relative_artifact_root = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "formal-assurance",
            run_id,
        ]);
        let artifact_root = repo_root.join(&relative_artifact_root);
        if artifact_root.exists() {
            fs::remove_dir_all(&artifact_root).map_err(|source| {
                FormalAssuranceError::RemoveDirectory {
                    path: artifact_root.display().to_string(),
                    source,
                }
            })?;
        }
        fs::create_dir_all(&artifact_root).map_err(|source| {
            FormalAssuranceError::CreateDirectory {
                path: artifact_root.display().to_string(),
                source,
            }
        })?;

        let w040_obligation_summary_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "release-grade-ledger",
            W040_DIRECT_OBLIGATION_RUN_ID,
            "run_summary.json",
        ]);
        let w040_obligation_map_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "release-grade-ledger",
            W040_DIRECT_OBLIGATION_RUN_ID,
            "direct_verification_obligation_map.json",
        ]);
        let w037_formal_summary_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "formal-inventory",
            W037_FORMAL_INVENTORY_RUN_ID,
            "run_summary.json",
        ]);
        let w037_formal_validation_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "formal-inventory",
            W037_FORMAL_INVENTORY_RUN_ID,
            "validation.json",
        ]);
        let w037_tla_inventory_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "formal-inventory",
            W037_FORMAL_INVENTORY_RUN_ID,
            "tla_inventory.json",
        ]);
        let w039_formal_summary_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "formal-assurance",
            "w039-proof-model-totality-closure-001",
            "run_summary.json",
        ]);
        let w039_formal_validation_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "formal-assurance",
            "w039-proof-model-totality-closure-001",
            "validation.json",
        ]);
        let w040_rust_summary_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "formal-assurance",
            W040_RUST_FORMAL_ASSURANCE_RUN_ID,
            "run_summary.json",
        ]);
        let w040_rust_validation_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "formal-assurance",
            W040_RUST_FORMAL_ASSURANCE_RUN_ID,
            "validation.json",
        ]);
        let w040_rust_blockers_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "formal-assurance",
            W040_RUST_FORMAL_ASSURANCE_RUN_ID,
            "w040_rust_exact_blocker_register.json",
        ]);

        let w040_obligation_summary = read_json(repo_root, &w040_obligation_summary_path)?;
        let w040_obligation_map = read_json(repo_root, &w040_obligation_map_path)?;
        let w037_formal_summary = read_json(repo_root, &w037_formal_summary_path)?;
        let w037_formal_validation = read_json(repo_root, &w037_formal_validation_path)?;
        let w037_tla_inventory = read_json(repo_root, &w037_tla_inventory_path)?;
        let w039_formal_summary = read_json(repo_root, &w039_formal_summary_path)?;
        let w039_formal_validation = read_json(repo_root, &w039_formal_validation_path)?;
        let w040_rust_summary = read_json(repo_root, &w040_rust_summary_path)?;
        let w040_rust_validation = read_json(repo_root, &w040_rust_validation_path)?;
        let w040_rust_blockers = read_json(repo_root, &w040_rust_blockers_path)?;

        let lean_discharge_file_present = repo_root.join(W040_LEAN_TLA_DISCHARGE_FILE).exists();
        let lean_rust_file_present = repo_root.join(W040_LEAN_RUST_TOTALITY_FILE).exists();
        let stage2_policy_file_present = repo_root.join(W039_STAGE2_POLICY_FILE).exists();
        let lean_placeholder_count = lean_placeholder_count(repo_root)?;
        let routine_tla_config_count =
            counter_value(&w037_formal_summary, "tla_routine_config_count");
        let routine_tla_failed_count =
            counter_value(&w037_formal_summary, "tla_failed_config_count");
        let tla_inventory_passed_count = counter_value(&w037_tla_inventory, "passed_config_count");

        let proof_rows = vec![
            json!({
                "row_id": "w040_lean_inventory_checked_no_placeholder_evidence",
                "w040_obligation_id": "W040-OBL-008",
                "source_inputs": ["W037 formal inventory", W040_LEAN_TLA_DISCHARGE_FILE],
                "disposition_kind": "checked_lean_inventory_evidence",
                "disposition": "bind the current Lean file inventory and zero-placeholder audit as checked evidence, without promoting full Lean verification",
                "local_checked_proof": true,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": false,
                "exact_remaining_blocker": false,
                "authority_owner": "calc-tv5.4",
                "promotion_consequence": "full Lean verification remains unpromoted until all semantic proof boundaries are discharged",
                "reason": "The Lean inventory is typechecked and the local placeholder census is zero, but the inventory is not a whole-engine semantic proof.",
                "evidence_paths": [&w037_formal_summary_path, &w037_formal_validation_path, W040_LEAN_TLA_DISCHARGE_FILE],
                "observed": {
                    "lean_file_count": counter_value(&w037_formal_summary, "lean_file_count"),
                    "lean_placeholder_count": lean_placeholder_count
                },
                "failures": if bool_at(&w037_formal_summary, "all_checked_artifacts_passed") && lean_placeholder_count == 0 && lean_discharge_file_present { Vec::<String>::new() } else { vec!["w040_lean_inventory_or_placeholder_check_failed".to_string()] },
                "validation_state": if bool_at(&w037_formal_summary, "all_checked_artifacts_passed") && lean_placeholder_count == 0 && lean_discharge_file_present { "w040_lean_proof_row_validated" } else { "w040_lean_proof_row_failed" }
            }),
            json!({
                "row_id": "w040_lean_rust_totality_classification_bridge",
                "w040_obligation_id": "W040-OBL-008",
                "source_inputs": ["W040 Rust totality/refinement packet", W040_LEAN_RUST_TOTALITY_FILE],
                "disposition_kind": "checked_lean_bridge_evidence",
                "disposition": "bind the W040 Rust totality/refinement classification as a checked Lean input to the Lean/TLA tranche",
                "local_checked_proof": true,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": false,
                "exact_remaining_blocker": false,
                "authority_owner": "calc-tv5.4",
                "promotion_consequence": "Rust totality and full Lean verification remain unpromoted",
                "reason": "The preceding W040 Rust packet is valid and non-promoting; this row binds it as a proof input rather than a discharge.",
                "evidence_paths": [&w040_rust_summary_path, &w040_rust_validation_path, W040_LEAN_RUST_TOTALITY_FILE],
                "failures": if string_value(&w040_rust_validation, "status") == "formal_assurance_w040_rust_totality_refinement_valid" && lean_rust_file_present { Vec::<String>::new() } else { vec!["w040_rust_formal_assurance_not_valid".to_string()] },
                "validation_state": if string_value(&w040_rust_validation, "status") == "formal_assurance_w040_rust_totality_refinement_valid" && lean_rust_file_present { "w040_lean_proof_row_validated" } else { "w040_lean_proof_row_failed" }
            }),
            json!({
                "row_id": "w040_lean_stage2_policy_predicate_carried",
                "w040_obligation_id": "W040-OBL-008",
                "source_inputs": ["W039 Stage 2 Lean predicate"],
                "disposition_kind": "checked_lean_policy_predicate",
                "disposition": "carry the checked Stage 2 promotion predicate as a Lean proof input while retaining production-policy blockers",
                "local_checked_proof": true,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": false,
                "exact_remaining_blocker": false,
                "authority_owner": "calc-tv5.4; calc-tv5.5",
                "promotion_consequence": "Stage 2 production policy remains unpromoted",
                "reason": "The predicate proves no-promotion under current evidence; it is not production partition analyzer soundness.",
                "evidence_paths": [W039_STAGE2_POLICY_FILE],
                "failures": if stage2_policy_file_present { Vec::<String>::new() } else { vec!["w039_stage2_policy_file_missing".to_string()] },
                "validation_state": if stage2_policy_file_present { "w040_lean_proof_row_validated" } else { "w040_lean_proof_row_failed" }
            }),
            json!({
                "row_id": "w040_tla_routine_config_bounded_model_boundary",
                "w040_obligation_id": "W040-OBL-009",
                "source_inputs": ["W037 TLA inventory", "routine TLC config set"],
                "disposition_kind": "bounded_model_with_exact_totality_boundary",
                "disposition": "bind the routine TLC config set as bounded model evidence while retaining unbounded model coverage as exact blocker",
                "local_checked_proof": false,
                "bounded_model": true,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": true,
                "exact_remaining_blocker": true,
                "authority_owner": "calc-tv5.4; calc-tv5.10",
                "promotion_consequence": "full TLA verification remains unpromoted",
                "reason": "The routine TLC floor has 11 bounded configs with zero recorded failures, but does not cover the unbounded scheduler and partition universe.",
                "evidence_paths": [&w037_tla_inventory_path, &w037_formal_summary_path],
                "observed": {
                    "routine_tla_config_count": routine_tla_config_count,
                    "tla_inventory_passed_count": tla_inventory_passed_count,
                    "routine_tla_failed_count": routine_tla_failed_count
                },
                "failures": if routine_tla_config_count == 11 && tla_inventory_passed_count == 11 && routine_tla_failed_count == 0 { Vec::<String>::new() } else { vec!["w040_tla_routine_config_floor_changed".to_string()] },
                "validation_state": if routine_tla_config_count == 11 && tla_inventory_passed_count == 11 && routine_tla_failed_count == 0 { "w040_tla_model_row_validated" } else { "w040_tla_model_row_failed" }
            }),
            json!({
                "row_id": "w040_tla_stage2_partition_bounded_model_evidence",
                "w040_obligation_id": "W040-OBL-009",
                "source_inputs": ["CoreEngineW036Stage2Partition bounded configs"],
                "disposition_kind": "bounded_stage2_partition_model_evidence",
                "disposition": "bind the W036 Stage 2 partition configs as bounded coverage for scheduler readiness, partition cross-dependency, fence reject, and multi-reader profiles",
                "local_checked_proof": false,
                "bounded_model": true,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": false,
                "exact_remaining_blocker": false,
                "authority_owner": "calc-tv5.4; calc-tv5.5",
                "promotion_consequence": "Stage 2 production policy remains unpromoted",
                "reason": "The bounded configs provide concrete model coverage but do not prove production analyzer soundness or unbounded fairness.",
                "evidence_paths": [
                    "formal/tla/CoreEngineW036Stage2Partition.tla",
                    "formal/tla/CoreEngineW036Stage2Partition.scheduler_blocked.cfg",
                    "formal/tla/CoreEngineW036Stage2Partition.partition_cross_dep.cfg",
                    "formal/tla/CoreEngineW036Stage2Partition.bounded_ready.cfg",
                    "formal/tla/CoreEngineW036Stage2Partition.fence_reject.cfg",
                    "formal/tla/CoreEngineW036Stage2Partition.multi_reader.cfg"
                ],
                "failures": if routine_tla_config_count >= 11 && routine_tla_failed_count == 0 { Vec::<String>::new() } else { vec!["w040_stage2_partition_tla_floor_changed".to_string()] },
                "validation_state": if routine_tla_config_count >= 11 && routine_tla_failed_count == 0 { "w040_tla_model_row_validated" } else { "w040_tla_model_row_failed" }
            }),
            json!({
                "row_id": "w040_tla_fairness_scheduler_assumption_boundary",
                "w040_obligation_id": "W040-OBL-009",
                "source_inputs": ["TLA model bounds and scheduler assumptions"],
                "disposition_kind": "exact_model_assumption_boundary",
                "disposition": "retain scheduler fairness and unbounded interleaving assumptions as explicit model boundaries",
                "local_checked_proof": false,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": true,
                "exact_remaining_blocker": true,
                "authority_owner": "calc-tv5.4; calc-tv5.5; calc-tv5.10",
                "promotion_consequence": "full TLA verification and Stage 2 policy remain unpromoted",
                "reason": "Current TLC configs are bounded and do not discharge fairness or unbounded scheduler coverage for promoted profiles.",
                "evidence_paths": [&w040_obligation_map_path, &w037_tla_inventory_path],
                "failures": if w040_obligation_exists(&w040_obligation_map, "W040-OBL-009") { Vec::<String>::new() } else { vec!["w040_tla_obligation_missing".to_string()] },
                "validation_state": if w040_obligation_exists(&w040_obligation_map, "W040-OBL-009") { "w040_lean_tla_exact_blocker_validated" } else { "w040_lean_tla_exact_blocker_failed" }
            }),
            json!({
                "row_id": "w040_full_lean_verification_exact_blocker",
                "w040_obligation_id": "W040-OBL-008",
                "source_inputs": ["W040 direct obligation map", "Lean proof inventory"],
                "disposition_kind": "exact_lean_verification_blocker",
                "disposition": "retain full Lean verification as exact blocker despite checked local proof rows",
                "local_checked_proof": true,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": true,
                "exact_remaining_blocker": true,
                "authority_owner": "calc-tv5.4; calc-tv5.10",
                "promotion_consequence": "full Lean verification remains unpromoted",
                "reason": "Checked classification files do not prove every Rust, OxFml, and coordinator semantic path for the claimed scope.",
                "evidence_paths": [&w040_obligation_map_path, W040_LEAN_TLA_DISCHARGE_FILE],
                "failures": if w040_obligation_exists(&w040_obligation_map, "W040-OBL-008") && lean_discharge_file_present { Vec::<String>::new() } else { vec!["w040_full_lean_blocker_input_missing".to_string()] },
                "validation_state": if w040_obligation_exists(&w040_obligation_map, "W040-OBL-008") && lean_discharge_file_present { "w040_lean_tla_exact_blocker_validated" } else { "w040_lean_tla_exact_blocker_failed" }
            }),
            json!({
                "row_id": "w040_full_tla_verification_exact_blocker",
                "w040_obligation_id": "W040-OBL-009",
                "source_inputs": ["W040 direct obligation map", "bounded TLC evidence"],
                "disposition_kind": "exact_tla_verification_blocker",
                "disposition": "retain full TLA verification as exact blocker because current coverage is bounded",
                "local_checked_proof": false,
                "bounded_model": true,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": true,
                "exact_remaining_blocker": true,
                "authority_owner": "calc-tv5.4; calc-tv5.10",
                "promotion_consequence": "full TLA verification remains unpromoted",
                "reason": "Bounded TLC runs do not discharge unbounded model completeness, fairness, or production partition analyzer soundness.",
                "evidence_paths": [&w040_obligation_map_path, &w037_tla_inventory_path],
                "failures": if w040_obligation_exists(&w040_obligation_map, "W040-OBL-009") && routine_tla_config_count == 11 { Vec::<String>::new() } else { vec!["w040_full_tla_blocker_input_missing".to_string()] },
                "validation_state": if w040_obligation_exists(&w040_obligation_map, "W040-OBL-009") && routine_tla_config_count == 11 { "w040_lean_tla_exact_blocker_validated" } else { "w040_lean_tla_exact_blocker_failed" }
            }),
            json!({
                "row_id": "w040_rust_totality_dependency_exact_blocker",
                "w040_obligation_id": "W040-OBL-007",
                "source_inputs": ["W040 Rust totality/refinement blockers"],
                "disposition_kind": "exact_rust_dependency_blocker",
                "disposition": "retain Rust totality/refinement dependency as a Lean/TLA proof blocker",
                "local_checked_proof": true,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": true,
                "exact_remaining_blocker": true,
                "authority_owner": "calc-tv5.4; calc-tv5.10",
                "promotion_consequence": "full Lean/TLA verification remains unpromoted",
                "reason": "The preceding W040 Rust packet intentionally retains five exact blockers, so Lean/TLA full verification cannot be promoted over them.",
                "evidence_paths": [&w040_rust_summary_path, &w040_rust_blockers_path],
                "failures": if counter_value(&w040_rust_summary, "exact_remaining_blocker_count") == 5 && counter_value(&w040_rust_blockers, "exact_remaining_blocker_count") == 5 { Vec::<String>::new() } else { vec!["w040_rust_blocker_count_changed".to_string()] },
                "validation_state": if counter_value(&w040_rust_summary, "exact_remaining_blocker_count") == 5 && counter_value(&w040_rust_blockers, "exact_remaining_blocker_count") == 5 { "w040_lean_tla_exact_blocker_validated" } else { "w040_lean_tla_exact_blocker_failed" }
            }),
            json!({
                "row_id": "w040_let_lambda_external_oxfunc_boundary",
                "w040_obligation_id": "W040-OBL-020",
                "source_inputs": ["W040 direct obligation map"],
                "disposition_kind": "accepted_external_seam_boundary",
                "disposition": "keep LET/LAMBDA carrier interaction inside OxCalc/OxFml formalization while excluding general OxFunc kernels",
                "local_checked_proof": false,
                "bounded_model": false,
                "accepted_external_seam": true,
                "accepted_boundary": true,
                "totality_boundary": false,
                "exact_remaining_blocker": false,
                "authority_owner": "calc-tv5.4; calc-tv5.8; external:OxFunc",
                "promotion_consequence": "general OxFunc kernels remain unpromoted inside OxCalc",
                "reason": "W040 scope includes the carrier seam but not broad OxFunc semantic kernels.",
                "evidence_paths": [&w040_obligation_map_path],
                "failures": if w040_obligation_exists(&w040_obligation_map, "W040-OBL-020") { Vec::<String>::new() } else { vec!["w040_let_lambda_boundary_missing".to_string()] },
                "validation_state": if w040_obligation_exists(&w040_obligation_map, "W040-OBL-020") { "w040_lean_tla_boundary_validated" } else { "w040_lean_tla_boundary_failed" }
            }),
            json!({
                "row_id": "w040_formal_model_spec_evolution_guard",
                "w040_obligation_id": "W040-OBL-008",
                "source_inputs": ["W040 direct obligation map", "W040 workset"],
                "disposition_kind": "accepted_spec_evolution_guard",
                "disposition": "preserve Lean/TLA formalization as spec-evolution and implementation-improvement work",
                "local_checked_proof": true,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": true,
                "totality_boundary": false,
                "exact_remaining_blocker": false,
                "authority_owner": "calc-tv5.4; calc-tv5.10",
                "promotion_consequence": "future proof/model evidence may correct specs or implementation before promotion",
                "reason": "W040 explicitly allows proof/model evidence to evolve the specs rather than testing against a fixed initial document set.",
                "evidence_paths": [&w040_obligation_map_path, "docs/worksets/W040_CORE_FORMALIZATION_RELEASE_GRADE_DIRECT_VERIFICATION.md"],
                "failures": if w040_obligation_exists(&w040_obligation_map, "W040-OBL-008") { Vec::<String>::new() } else { vec!["w040_lean_obligation_missing".to_string()] },
                "validation_state": if w040_obligation_exists(&w040_obligation_map, "W040-OBL-008") { "w040_lean_tla_boundary_validated" } else { "w040_lean_tla_boundary_failed" }
            }),
        ];

        let local_proof_row_count = proof_rows
            .iter()
            .filter(|row| bool_at(row, "local_checked_proof"))
            .count();
        let bounded_model_row_count = proof_rows
            .iter()
            .filter(|row| bool_at(row, "bounded_model"))
            .count();
        let accepted_external_seam_count = proof_rows
            .iter()
            .filter(|row| bool_at(row, "accepted_external_seam"))
            .count();
        let accepted_boundary_count = proof_rows
            .iter()
            .filter(|row| bool_at(row, "accepted_boundary"))
            .count();
        let totality_rows = proof_rows
            .iter()
            .filter(|row| bool_at(row, "totality_boundary"))
            .cloned()
            .collect::<Vec<_>>();
        let lean_proof_rows = proof_rows
            .iter()
            .filter(|row| bool_at(row, "local_checked_proof"))
            .cloned()
            .collect::<Vec<_>>();
        let model_bound_rows = proof_rows
            .iter()
            .filter(|row| bool_at(row, "bounded_model"))
            .cloned()
            .collect::<Vec<_>>();
        let blocker_rows = proof_rows
            .iter()
            .filter(|row| bool_at(row, "exact_remaining_blocker"))
            .cloned()
            .collect::<Vec<_>>();
        let failed_row_count = proof_rows
            .iter()
            .filter(|row| {
                !row.get("failures")
                    .and_then(Value::as_array)
                    .is_some_and(Vec::is_empty)
            })
            .count();

        let mut validation_failures = Vec::new();
        if counter_value(&w040_obligation_summary, "obligation_count") != 23 {
            validation_failures.push("w040_obligation_count_changed".to_string());
        }
        if !w040_obligation_exists(&w040_obligation_map, "W040-OBL-008")
            || !w040_obligation_exists(&w040_obligation_map, "W040-OBL-009")
        {
            validation_failures.push("w040_lean_tla_obligation_rows_missing".to_string());
        }
        if !bool_at(&w037_formal_summary, "all_checked_artifacts_passed") {
            validation_failures.push("w037_formal_artifacts_not_all_checked".to_string());
        }
        if string_value(&w037_formal_validation, "validation_state")
            != "w037_proof_model_closure_inventory_validated"
        {
            validation_failures.push("w037_formal_validation_not_valid".to_string());
        }
        if string_value(&w039_formal_validation, "status")
            != "formal_assurance_w039_totality_closure_valid"
        {
            validation_failures.push("w039_formal_validation_not_valid".to_string());
        }
        if string_value(&w040_rust_validation, "status")
            != "formal_assurance_w040_rust_totality_refinement_valid"
        {
            validation_failures.push("w040_rust_validation_not_valid".to_string());
        }
        if bool_at(
            w039_formal_summary
                .get("promotion_claims")
                .unwrap_or(&Value::Null),
            "full_lean_verification_promoted",
        ) || bool_at(
            w039_formal_summary
                .get("promotion_claims")
                .unwrap_or(&Value::Null),
            "full_tla_verification_promoted",
        ) {
            validation_failures.push("w039_lean_tla_was_promoted".to_string());
        }
        if lean_placeholder_count != 0 {
            validation_failures.push("w040_lean_placeholder_count_nonzero".to_string());
        }
        if routine_tla_config_count != 11
            || tla_inventory_passed_count != 11
            || routine_tla_failed_count != 0
        {
            validation_failures.push("w040_tla_routine_floor_changed".to_string());
        }
        if !lean_discharge_file_present {
            validation_failures.push("w040_lean_tla_discharge_file_missing".to_string());
        }
        if failed_row_count != 0 {
            validation_failures.push("w040_lean_tla_row_failures_present".to_string());
        }
        if blocker_rows.len() != 5 {
            validation_failures.push("w040_expected_five_lean_tla_exact_blockers".to_string());
        }

        let source_evidence_index_path =
            format!("{relative_artifact_root}/source_evidence_index.json");
        let ledger_path = format!("{relative_artifact_root}/w040_lean_tla_discharge_ledger.json");
        let lean_register_path = format!("{relative_artifact_root}/w040_lean_proof_register.json");
        let model_register_path =
            format!("{relative_artifact_root}/w040_tla_model_bound_register.json");
        let blocker_register_path =
            format!("{relative_artifact_root}/w040_lean_tla_exact_blocker_register.json");
        let validation_path = format!("{relative_artifact_root}/validation.json");

        write_json(
            &artifact_root.join("source_evidence_index.json"),
            &json!({
                "schema_version": FORMAL_ASSURANCE_SOURCE_INDEX_SCHEMA_V1,
                "run_id": run_id,
                "source_artifacts": {
                    "w040_direct_obligation_summary": w040_obligation_summary_path,
                    "w040_direct_obligation_map": w040_obligation_map_path,
                    "w037_formal_inventory_summary": w037_formal_summary_path,
                    "w037_formal_inventory_validation": w037_formal_validation_path,
                    "w037_tla_inventory": w037_tla_inventory_path,
                    "w039_formal_assurance_summary": w039_formal_summary_path,
                    "w039_formal_assurance_validation": w039_formal_validation_path,
                    "w040_rust_formal_assurance_summary": w040_rust_summary_path,
                    "w040_rust_formal_assurance_validation": w040_rust_validation_path,
                    "w040_rust_exact_blockers": w040_rust_blockers_path,
                    "w040_lean_tla_discharge_file": W040_LEAN_TLA_DISCHARGE_FILE,
                    "w039_stage2_policy_file": W039_STAGE2_POLICY_FILE
                },
                "source_counts": {
                    "w040_obligation_count": counter_value(&w040_obligation_summary, "obligation_count"),
                    "w037_lean_file_count": counter_value(&w037_formal_summary, "lean_file_count"),
                    "w037_tla_routine_config_count": routine_tla_config_count,
                    "w037_tla_inventory_passed_count": tla_inventory_passed_count,
                    "w037_tla_failed_config_count": routine_tla_failed_count,
                    "w039_formal_exact_blocker_count": counter_value(&w039_formal_summary, "exact_remaining_blocker_count"),
                    "w040_rust_exact_blocker_count": counter_value(&w040_rust_summary, "exact_remaining_blocker_count"),
                    "lean_placeholder_count": lean_placeholder_count
                }
            }),
        )?;
        write_json(
            &artifact_root.join("w040_lean_tla_discharge_ledger.json"),
            &json!({
                "schema_version": FORMAL_ASSURANCE_W040_LEAN_TLA_LEDGER_SCHEMA_V1,
                "run_id": run_id,
                "proof_model_row_count": proof_rows.len(),
                "local_proof_row_count": local_proof_row_count,
                "bounded_model_row_count": bounded_model_row_count,
                "accepted_external_seam_count": accepted_external_seam_count,
                "accepted_boundary_count": accepted_boundary_count,
                "totality_boundary_count": totality_rows.len(),
                "exact_remaining_blocker_count": blocker_rows.len(),
                "rows": proof_rows
            }),
        )?;
        write_json(
            &artifact_root.join("w040_lean_proof_register.json"),
            &json!({
                "schema_version": FORMAL_ASSURANCE_W040_LEAN_PROOF_REGISTER_SCHEMA_V1,
                "run_id": run_id,
                "local_proof_row_count": lean_proof_rows.len(),
                "rows": lean_proof_rows
            }),
        )?;
        write_json(
            &artifact_root.join("w040_tla_model_bound_register.json"),
            &json!({
                "schema_version": FORMAL_ASSURANCE_W040_TLA_MODEL_REGISTER_SCHEMA_V1,
                "run_id": run_id,
                "bounded_model_row_count": model_bound_rows.len(),
                "rows": model_bound_rows
            }),
        )?;
        write_json(
            &artifact_root.join("w040_lean_tla_exact_blocker_register.json"),
            &json!({
                "schema_version": FORMAL_ASSURANCE_W040_LEAN_TLA_BLOCKER_REGISTER_SCHEMA_V1,
                "run_id": run_id,
                "exact_remaining_blocker_count": blocker_rows.len(),
                "rows": blocker_rows
            }),
        )?;

        let validation_status = if validation_failures.is_empty() {
            "formal_assurance_w040_lean_tla_discharge_valid"
        } else {
            "formal_assurance_w040_lean_tla_discharge_invalid"
        };
        write_json(
            &artifact_root.join("validation.json"),
            &json!({
                "schema_version": FORMAL_ASSURANCE_VALIDATION_SCHEMA_V1,
                "run_id": run_id,
                "status": validation_status,
                "proof_model_row_count": proof_rows.len(),
                "local_proof_row_count": local_proof_row_count,
                "bounded_model_row_count": bounded_model_row_count,
                "accepted_external_seam_count": accepted_external_seam_count,
                "accepted_boundary_count": accepted_boundary_count,
                "totality_boundary_count": totality_rows.len(),
                "exact_remaining_blocker_count": blocker_rows.len(),
                "failed_row_count": failed_row_count,
                "validation_failures": validation_failures
            }),
        )?;
        write_json(
            &artifact_root.join("run_summary.json"),
            &json!({
                "schema_version": FORMAL_ASSURANCE_RUN_SUMMARY_SCHEMA_V1,
                "run_id": run_id,
                "artifact_root": relative_artifact_root,
                "source_evidence_index_path": source_evidence_index_path,
                "assumption_discharge_ledger_path": ledger_path,
                "lean_proof_register_path": lean_register_path,
                "model_bound_register_path": model_register_path,
                "exact_proof_model_blocker_register_path": blocker_register_path,
                "validation_path": validation_path,
                "assumption_row_count": proof_rows.len(),
                "local_proof_row_count": local_proof_row_count,
                "bounded_model_row_count": bounded_model_row_count,
                "accepted_external_seam_count": accepted_external_seam_count,
                "accepted_boundary_count": accepted_boundary_count,
                "totality_boundary_count": totality_rows.len(),
                "exact_remaining_blocker_count": blocker_rows.len(),
                "failed_row_count": failed_row_count,
                "promotion_claims": {
                    "full_lean_verification_promoted": false,
                    "full_tla_verification_promoted": false,
                    "rust_engine_totality_promoted": false,
                    "stage2_policy_promoted": false,
                    "pack_grade_replay_promoted": false,
                    "c5_promoted": false,
                    "general_oxfunc_kernel_promoted": false
                }
            }),
        )?;

        Ok(FormalAssuranceRunSummary {
            run_id: run_id.to_string(),
            schema_version: FORMAL_ASSURANCE_RUN_SUMMARY_SCHEMA_V1.to_string(),
            assumption_row_count: proof_rows.len(),
            local_proof_row_count,
            bounded_model_row_count,
            accepted_external_seam_count,
            accepted_boundary_count,
            totality_boundary_count: totality_rows.len(),
            exact_remaining_blocker_count: blocker_rows.len(),
            failed_row_count,
            artifact_root: relative_artifact_root,
        })
    }

    fn execute_w040_rust_totality_refinement(
        &self,
        repo_root: &Path,
        run_id: &str,
    ) -> Result<FormalAssuranceRunSummary, FormalAssuranceError> {
        let relative_artifact_root = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "formal-assurance",
            run_id,
        ]);
        let artifact_root = repo_root.join(&relative_artifact_root);
        if artifact_root.exists() {
            fs::remove_dir_all(&artifact_root).map_err(|source| {
                FormalAssuranceError::RemoveDirectory {
                    path: artifact_root.display().to_string(),
                    source,
                }
            })?;
        }
        fs::create_dir_all(&artifact_root).map_err(|source| {
            FormalAssuranceError::CreateDirectory {
                path: artifact_root.display().to_string(),
                source,
            }
        })?;

        let w040_obligation_summary_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "release-grade-ledger",
            W040_DIRECT_OBLIGATION_RUN_ID,
            "run_summary.json",
        ]);
        let w040_obligation_map_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "release-grade-ledger",
            W040_DIRECT_OBLIGATION_RUN_ID,
            "direct_verification_obligation_map.json",
        ]);
        let w039_formal_summary_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "formal-assurance",
            "w039-proof-model-totality-closure-001",
            "run_summary.json",
        ]);
        let w039_formal_validation_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "formal-assurance",
            "w039-proof-model-totality-closure-001",
            "validation.json",
        ]);
        let w040_conformance_summary_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "implementation-conformance",
            W040_IMPLEMENTATION_CONFORMANCE_RUN_ID,
            "run_summary.json",
        ]);
        let w040_conformance_validation_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "implementation-conformance",
            W040_IMPLEMENTATION_CONFORMANCE_RUN_ID,
            "validation.json",
        ]);
        let w040_conformance_blockers_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "implementation-conformance",
            W040_IMPLEMENTATION_CONFORMANCE_RUN_ID,
            "w040_exact_remaining_blocker_register.json",
        ]);
        let w040_dynamic_evidence_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "implementation-conformance",
            W040_IMPLEMENTATION_CONFORMANCE_RUN_ID,
            "dynamic_release_reclassification_evidence.json",
        ]);
        let w040_treecalc_summary_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "treecalc-local",
            W040_TREECALC_RUN_ID,
            "run_summary.json",
        ]);
        let w040_treecalc_post_edit_result_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "treecalc-local",
            W040_TREECALC_RUN_ID,
            "cases",
            "tc_local_dynamic_release_reclassification_post_edit_001",
            "post_edit",
            "result.json",
        ]);
        let w040_treecalc_post_edit_closure_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "treecalc-local",
            W040_TREECALC_RUN_ID,
            "cases",
            "tc_local_dynamic_release_reclassification_post_edit_001",
            "post_edit",
            "invalidation_closure.json",
        ]);

        let w040_obligation_summary = read_json(repo_root, &w040_obligation_summary_path)?;
        let w040_obligation_map = read_json(repo_root, &w040_obligation_map_path)?;
        let w039_formal_summary = read_json(repo_root, &w039_formal_summary_path)?;
        let w039_formal_validation = read_json(repo_root, &w039_formal_validation_path)?;
        let w040_conformance_summary = read_json(repo_root, &w040_conformance_summary_path)?;
        let w040_conformance_validation = read_json(repo_root, &w040_conformance_validation_path)?;
        let w040_conformance_blockers = read_json(repo_root, &w040_conformance_blockers_path)?;
        let w040_dynamic_evidence = read_json(repo_root, &w040_dynamic_evidence_path)?;
        let w040_treecalc_summary = read_json(repo_root, &w040_treecalc_summary_path)?;
        let w040_treecalc_post_edit_result =
            read_json(repo_root, &w040_treecalc_post_edit_result_path)?;
        let w040_treecalc_post_edit_closure =
            read_json(repo_root, &w040_treecalc_post_edit_closure_path)?;

        let lean_file_present = repo_root.join(W040_LEAN_RUST_TOTALITY_FILE).exists();
        let panic_marker_count = panic_marker_count(repo_root, W040_RUST_PANIC_AUDIT_FILES)?;
        let dynamic_closure_requires_rebind = w040_treecalc_post_edit_closure
            .as_array()
            .is_some_and(|rows| {
                rows.iter().any(|row| {
                    row.get("node_id").and_then(Value::as_u64) == Some(3)
                        && bool_at(row, "requires_rebind")
                })
            });
        let post_edit_rejected_for_rebind =
            string_value(&w040_treecalc_post_edit_result, "result_state") == "rejected"
                && w040_treecalc_post_edit_result
                    .get("reject_detail")
                    .and_then(|detail| detail.get("kind"))
                    .and_then(Value::as_str)
                    == Some("HostInjectedFailure");

        let proof_rows = vec![
            json!({
                "row_id": "w040_result_error_carrier_totality_evidence",
                "w040_obligation_id": "W040-OBL-006",
                "source_inputs": ["Rust typed error carriers", W040_LEAN_RUST_TOTALITY_FILE],
                "disposition_kind": "direct_totality_evidence",
                "disposition": "bind promoted core public paths to typed Result/error carriers rather than panic-as-contract",
                "local_checked_proof": true,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": false,
                "refinement_row": false,
                "exact_remaining_blocker": false,
                "authority_owner": "calc-tv5.3",
                "promotion_consequence": "Rust totality remains unpromoted because this is a carrier row, not whole-engine proof",
                "reason": "Core execution, fixture, runner, structural, and coordinator surfaces expose Result/typed error APIs for promoted evidence paths.",
                "evidence_paths": [
                    "src/oxcalc-core/src/coordinator.rs",
                    "src/oxcalc-core/src/recalc.rs",
                    "src/oxcalc-core/src/structural.rs",
                    "src/oxcalc-core/src/treecalc.rs",
                    "src/oxcalc-core/src/treecalc_fixture.rs",
                    "src/oxcalc-core/src/treecalc_runner.rs",
                    W040_LEAN_RUST_TOTALITY_FILE
                ],
                "failures": if lean_file_present { Vec::<String>::new() } else { vec!["w040_lean_rust_totality_file_missing".to_string()] },
                "validation_state": if lean_file_present { "w040_rust_totality_row_validated" } else { "w040_rust_totality_row_failed" }
            }),
            json!({
                "row_id": "w040_fixture_invalidation_seed_error_totality_evidence",
                "w040_obligation_id": "W040-OBL-006",
                "source_inputs": ["W040 TreeCalc fixture parsing", "W040 optimized/core evidence"],
                "disposition_kind": "direct_totality_evidence",
                "disposition": "unsupported explicit invalidation seed reasons now flow through typed fixture errors",
                "local_checked_proof": true,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": false,
                "refinement_row": false,
                "exact_remaining_blocker": false,
                "authority_owner": "calc-tv5.3",
                "promotion_consequence": "the fixture parse path is evidenced, while whole-engine panic freedom remains blocked",
                "reason": "The calc-tv5.2 implementation added explicit dependency-change seed parsing and unsupported-reason error reporting for the local runner.",
                "evidence_paths": [
                    "src/oxcalc-core/src/treecalc_fixture.rs",
                    &w040_conformance_summary_path,
                    &w040_conformance_validation_path
                ],
                "failures": if string_value(&w040_conformance_validation, "status") == "optimized_core_exact_blockers_narrowed_no_promotion_valid" { Vec::<String>::new() } else { vec!["w040_conformance_validation_not_valid".to_string()] },
                "validation_state": if string_value(&w040_conformance_validation, "status") == "optimized_core_exact_blockers_narrowed_no_promotion_valid" { "w040_rust_totality_row_validated" } else { "w040_rust_totality_row_failed" }
            }),
            json!({
                "row_id": "w040_dependency_seed_rebind_refinement_evidence",
                "w040_obligation_id": "W040-OBL-007",
                "source_inputs": ["W040 dynamic release/reclassification TreeCalc run"],
                "disposition_kind": "direct_refinement_evidence",
                "disposition": "explicit DependencyRemoved and DependencyReclassified seeds force rebind/no-publication behavior",
                "local_checked_proof": true,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": false,
                "refinement_row": true,
                "exact_remaining_blocker": false,
                "authority_owner": "calc-tv5.2; calc-tv5.3",
                "promotion_consequence": "refinement is evidenced only for the explicit-seed slice; automatic dependency-set transition remains blocked",
                "reason": "The post-edit closure marks node 3 as requiring rebind and the post-edit result rejects with HostInjectedFailure without a new publication.",
                "evidence_paths": [
                    &w040_dynamic_evidence_path,
                    &w040_treecalc_summary_path,
                    &w040_treecalc_post_edit_result_path,
                    &w040_treecalc_post_edit_closure_path
                ],
                "observed": {
                    "treecalc_case_count": counter_value(&w040_treecalc_summary, "case_count"),
                    "treecalc_expectation_mismatch_count": counter_value(&w040_treecalc_summary, "expectation_mismatch_count"),
                    "dynamic_closure_requires_rebind": dynamic_closure_requires_rebind,
                    "post_edit_rejected_for_rebind": post_edit_rejected_for_rebind
                },
                "failures": if counter_value(&w040_treecalc_summary, "expectation_mismatch_count") == 0 && dynamic_closure_requires_rebind && post_edit_rejected_for_rebind { Vec::<String>::new() } else { vec!["w040_dependency_seed_rebind_evidence_missing".to_string()] },
                "validation_state": if counter_value(&w040_treecalc_summary, "expectation_mismatch_count") == 0 && dynamic_closure_requires_rebind && post_edit_rejected_for_rebind { "w040_rust_refinement_row_validated" } else { "w040_rust_refinement_row_failed" }
            }),
            json!({
                "row_id": "w040_dynamic_transition_refinement_exact_blocker",
                "w040_obligation_id": "W040-OBL-007",
                "source_inputs": ["W040 optimized/core exact blocker register"],
                "disposition_kind": "exact_refinement_blocker",
                "disposition": "automatic dynamic dependency-set release/reclassification transition differential remains absent",
                "local_checked_proof": true,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": true,
                "refinement_row": true,
                "exact_remaining_blocker": true,
                "authority_owner": "calc-tv5.3; calc-tv5.5; calc-tv5.10",
                "promotion_consequence": "Rust refinement and full optimized/core verification remain unpromoted",
                "reason": "The W040 direct evidence is explicit-seed based and does not yet compare predecessor/successor dynamic descriptors without manual seed injection.",
                "evidence_paths": [&w040_conformance_blockers_path, &w040_dynamic_evidence_path],
                "failures": if row_with_field_exists(&w040_conformance_blockers, "row_id", "w040_dynamic_release_reclassification_transition_exact_blocker") { Vec::<String>::new() } else { vec!["w040_dynamic_transition_blocker_missing".to_string()] },
                "validation_state": if row_with_field_exists(&w040_conformance_blockers, "row_id", "w040_dynamic_release_reclassification_transition_exact_blocker") { "w040_rust_exact_blocker_validated" } else { "w040_rust_exact_blocker_failed" }
            }),
            json!({
                "row_id": "w040_runtime_panic_surface_totality_boundary",
                "w040_obligation_id": "W040-OBL-006",
                "source_inputs": ["Rust panic marker audit", W040_LEAN_RUST_TOTALITY_FILE],
                "disposition_kind": "exact_totality_boundary",
                "disposition": "retain a whole-engine panic-free proof blocker while panic/unwrap/expect markers remain in core Rust surfaces",
                "local_checked_proof": true,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": true,
                "refinement_row": false,
                "exact_remaining_blocker": true,
                "authority_owner": "calc-tv5.3; calc-tv5.10",
                "promotion_consequence": "Rust-engine totality and panic-free core domain remain unpromoted",
                "reason": "The marker census is a guard, not a semantic proof; observed panic-family markers require review or proof before a panic-free claim.",
                "evidence_paths": W040_RUST_PANIC_AUDIT_FILES,
                "observed": {
                    "panic_marker_count": panic_marker_count,
                    "audited_file_count": W040_RUST_PANIC_AUDIT_FILES.len()
                },
                "failures": Vec::<String>::new(),
                "validation_state": "w040_rust_exact_blocker_validated"
            }),
            json!({
                "row_id": "w040_snapshot_fence_refinement_boundary",
                "w040_obligation_id": "W040-OBL-003",
                "source_inputs": ["W040 optimized/core exact blocker register"],
                "disposition_kind": "exact_refinement_blocker",
                "disposition": "retain the snapshot-fence counterpart blocker as a refinement boundary",
                "local_checked_proof": false,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": true,
                "refinement_row": true,
                "exact_remaining_blocker": true,
                "authority_owner": "calc-tv5.5",
                "promotion_consequence": "coordinator/Stage 2 refinement remains unpromoted",
                "reason": "The stale accepted-candidate snapshot-fence counterpart remains owned by Stage 2/coordinator evidence and is not discharged by Rust carrier proof.",
                "evidence_paths": [&w040_conformance_blockers_path],
                "failures": if row_with_field_exists(&w040_conformance_blockers, "row_id", "w040_snapshot_fence_counterpart_exact_blocker") { Vec::<String>::new() } else { vec!["w040_snapshot_fence_blocker_missing".to_string()] },
                "validation_state": if row_with_field_exists(&w040_conformance_blockers, "row_id", "w040_snapshot_fence_counterpart_exact_blocker") { "w040_rust_exact_blocker_validated" } else { "w040_rust_exact_blocker_failed" }
            }),
            json!({
                "row_id": "w040_capability_view_fence_refinement_boundary",
                "w040_obligation_id": "W040-OBL-003",
                "source_inputs": ["W040 optimized/core exact blocker register"],
                "disposition_kind": "exact_refinement_blocker",
                "disposition": "retain the capability-view counterpart blocker as a refinement boundary",
                "local_checked_proof": false,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": true,
                "refinement_row": true,
                "exact_remaining_blocker": true,
                "authority_owner": "calc-tv5.5",
                "promotion_consequence": "coordinator/Stage 2 refinement remains unpromoted",
                "reason": "The compatibility-fenced capability-view mismatch counterpart remains owned by Stage 2/coordinator evidence and is not discharged by Rust carrier proof.",
                "evidence_paths": [&w040_conformance_blockers_path],
                "failures": if row_with_field_exists(&w040_conformance_blockers, "row_id", "w040_capability_view_fence_counterpart_exact_blocker") { Vec::<String>::new() } else { vec!["w040_capability_view_fence_blocker_missing".to_string()] },
                "validation_state": if row_with_field_exists(&w040_conformance_blockers, "row_id", "w040_capability_view_fence_counterpart_exact_blocker") { "w040_rust_exact_blocker_validated" } else { "w040_rust_exact_blocker_failed" }
            }),
            json!({
                "row_id": "w040_callable_metadata_projection_totality_boundary",
                "w040_obligation_id": "W040-OBL-004",
                "source_inputs": ["W040 optimized/core exact blocker register", "LET/LAMBDA carrier boundary"],
                "disposition_kind": "exact_totality_boundary",
                "disposition": "carry callable metadata projection as an exact totality/refinement blocker",
                "local_checked_proof": true,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": true,
                "refinement_row": true,
                "exact_remaining_blocker": true,
                "authority_owner": "calc-tv5.8; external:OxFunc",
                "promotion_consequence": "callable metadata projection and broad callable conformance remain unpromoted",
                "reason": "The narrow LET/LAMBDA carrier seam is in scope, but general OxFunc kernels and metadata projection sufficiency are not discharged.",
                "evidence_paths": [&w040_conformance_blockers_path, W040_LEAN_RUST_TOTALITY_FILE],
                "failures": if row_with_field_exists(&w040_conformance_blockers, "row_id", "w040_callable_metadata_projection_exact_blocker") && lean_file_present { Vec::<String>::new() } else { vec!["w040_callable_metadata_or_lean_blocker_missing".to_string()] },
                "validation_state": if row_with_field_exists(&w040_conformance_blockers, "row_id", "w040_callable_metadata_projection_exact_blocker") && lean_file_present { "w040_rust_exact_blocker_validated" } else { "w040_rust_exact_blocker_failed" }
            }),
            json!({
                "row_id": "w040_let_lambda_carrier_external_boundary",
                "w040_obligation_id": "W040-OBL-020",
                "source_inputs": ["W040 direct obligation map", W040_LEAN_RUST_TOTALITY_FILE],
                "disposition_kind": "accepted_external_seam_boundary",
                "disposition": "keep LET/LAMBDA carrier interaction inside OxCalc/OxFml formalization while excluding general OxFunc kernels",
                "local_checked_proof": false,
                "bounded_model": false,
                "accepted_external_seam": true,
                "accepted_boundary": true,
                "totality_boundary": false,
                "refinement_row": false,
                "exact_remaining_blocker": false,
                "authority_owner": "calc-tv5.4; calc-tv5.8; external:OxFunc",
                "promotion_consequence": "general OxFunc kernels remain unpromoted inside OxCalc",
                "reason": "W040 scope includes the carrier seam but not broad OxFunc semantic kernels.",
                "evidence_paths": [&w040_obligation_map_path, W040_LEAN_RUST_TOTALITY_FILE],
                "failures": if w040_obligation_exists(&w040_obligation_map, "W040-OBL-020") && lean_file_present { Vec::<String>::new() } else { vec!["w040_let_lambda_boundary_missing".to_string()] },
                "validation_state": if w040_obligation_exists(&w040_obligation_map, "W040-OBL-020") && lean_file_present { "w040_rust_boundary_validated" } else { "w040_rust_boundary_failed" }
            }),
            json!({
                "row_id": "w040_spec_evolution_refinement_guard",
                "w040_obligation_id": "W040-OBL-007",
                "source_inputs": ["W040 workset and obligation map"],
                "disposition_kind": "accepted_spec_evolution_guard",
                "disposition": "preserve formalization as spec evolution and implementation improvement, not a fixed-spec test only",
                "local_checked_proof": true,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": true,
                "totality_boundary": false,
                "refinement_row": false,
                "exact_remaining_blocker": false,
                "authority_owner": "calc-tv5.3; calc-tv5.10",
                "promotion_consequence": "future proof evidence may correct specs or implementation before promotion",
                "reason": "The W040 charter records spec-evolution hooks for Rust totality and refinement obligations.",
                "evidence_paths": [&w040_obligation_map_path, "docs/worksets/W040_CORE_FORMALIZATION_RELEASE_GRADE_DIRECT_VERIFICATION.md"],
                "failures": if w040_obligation_exists(&w040_obligation_map, "W040-OBL-007") { Vec::<String>::new() } else { vec!["w040_refinement_obligation_missing".to_string()] },
                "validation_state": if w040_obligation_exists(&w040_obligation_map, "W040-OBL-007") { "w040_rust_boundary_validated" } else { "w040_rust_boundary_failed" }
            }),
        ];

        let local_proof_row_count = proof_rows
            .iter()
            .filter(|row| bool_at(row, "local_checked_proof"))
            .count();
        let bounded_model_row_count = proof_rows
            .iter()
            .filter(|row| bool_at(row, "bounded_model"))
            .count();
        let accepted_external_seam_count = proof_rows
            .iter()
            .filter(|row| bool_at(row, "accepted_external_seam"))
            .count();
        let accepted_boundary_count = proof_rows
            .iter()
            .filter(|row| bool_at(row, "accepted_boundary"))
            .count();
        let totality_rows = proof_rows
            .iter()
            .filter(|row| bool_at(row, "totality_boundary"))
            .cloned()
            .collect::<Vec<_>>();
        let refinement_rows = proof_rows
            .iter()
            .filter(|row| bool_at(row, "refinement_row"))
            .cloned()
            .collect::<Vec<_>>();
        let blocker_rows = proof_rows
            .iter()
            .filter(|row| bool_at(row, "exact_remaining_blocker"))
            .cloned()
            .collect::<Vec<_>>();
        let failed_row_count = proof_rows
            .iter()
            .filter(|row| {
                !row.get("failures")
                    .and_then(Value::as_array)
                    .is_some_and(Vec::is_empty)
            })
            .count();

        let mut validation_failures = Vec::new();
        if counter_value(&w040_obligation_summary, "obligation_count") != 23 {
            validation_failures.push("w040_obligation_count_changed".to_string());
        }
        if !array_contains_string(
            w040_obligation_summary
                .get("no_promotion_claims")
                .unwrap_or(&Value::Null),
            "rust_totality_and_refinement",
        ) {
            validation_failures.push("w040_rust_no_promotion_guard_missing".to_string());
        }
        if !w040_obligation_exists(&w040_obligation_map, "W040-OBL-006")
            || !w040_obligation_exists(&w040_obligation_map, "W040-OBL-007")
        {
            validation_failures.push("w040_rust_obligation_rows_missing".to_string());
        }
        if string_value(&w039_formal_validation, "status")
            != "formal_assurance_w039_totality_closure_valid"
        {
            validation_failures.push("w039_formal_validation_not_valid".to_string());
        }
        if bool_at(
            w039_formal_summary
                .get("promotion_claims")
                .unwrap_or(&Value::Null),
            "rust_engine_totality_promoted",
        ) {
            validation_failures.push("w039_rust_totality_was_promoted".to_string());
        }
        if counter_value(&w040_conformance_summary, "exact_remaining_blocker_count") != 4 {
            validation_failures.push("w040_conformance_exact_blocker_count_changed".to_string());
        }
        if string_value(&w040_conformance_validation, "status")
            != "optimized_core_exact_blockers_narrowed_no_promotion_valid"
        {
            validation_failures.push("w040_conformance_validation_not_valid".to_string());
        }
        if counter_value(&w040_conformance_blockers, "exact_remaining_blocker_count") != 4 {
            validation_failures.push("w040_conformance_blocker_register_count_changed".to_string());
        }
        if counter_value(&w040_treecalc_summary, "expectation_mismatch_count") != 0 {
            validation_failures.push("w040_treecalc_expectation_mismatch_present".to_string());
        }
        if string_value(&w040_dynamic_evidence, "treecalc_run_id") != W040_TREECALC_RUN_ID {
            validation_failures.push("w040_dynamic_evidence_run_id_changed".to_string());
        }
        if !lean_file_present {
            validation_failures.push("w040_lean_rust_totality_file_missing".to_string());
        }
        if panic_marker_count == 0 {
            validation_failures.push("w040_panic_marker_audit_unexpected_zero".to_string());
        }
        if failed_row_count != 0 {
            validation_failures.push("w040_rust_totality_row_failures_present".to_string());
        }
        if blocker_rows.len() != 5 {
            validation_failures.push("w040_expected_five_rust_exact_blockers".to_string());
        }

        let source_evidence_index_path =
            format!("{relative_artifact_root}/source_evidence_index.json");
        let rust_ledger_path =
            format!("{relative_artifact_root}/w040_rust_totality_refinement_ledger.json");
        let totality_register_path =
            format!("{relative_artifact_root}/w040_rust_totality_boundary_register.json");
        let refinement_register_path =
            format!("{relative_artifact_root}/w040_rust_refinement_register.json");
        let blocker_register_path =
            format!("{relative_artifact_root}/w040_rust_exact_blocker_register.json");
        let validation_path = format!("{relative_artifact_root}/validation.json");

        write_json(
            &artifact_root.join("source_evidence_index.json"),
            &json!({
                "schema_version": FORMAL_ASSURANCE_SOURCE_INDEX_SCHEMA_V1,
                "run_id": run_id,
                "source_artifacts": {
                    "w040_direct_obligation_summary": w040_obligation_summary_path,
                    "w040_direct_obligation_map": w040_obligation_map_path,
                    "w039_formal_assurance_summary": w039_formal_summary_path,
                    "w039_formal_assurance_validation": w039_formal_validation_path,
                    "w040_implementation_conformance_summary": w040_conformance_summary_path,
                    "w040_implementation_conformance_validation": w040_conformance_validation_path,
                    "w040_implementation_conformance_exact_blockers": w040_conformance_blockers_path,
                    "w040_dynamic_release_reclassification_evidence": w040_dynamic_evidence_path,
                    "w040_treecalc_summary": w040_treecalc_summary_path,
                    "w040_treecalc_post_edit_result": w040_treecalc_post_edit_result_path,
                    "w040_treecalc_post_edit_closure": w040_treecalc_post_edit_closure_path,
                    "w040_lean_rust_totality_file": W040_LEAN_RUST_TOTALITY_FILE
                },
                "source_counts": {
                    "w040_obligation_count": counter_value(&w040_obligation_summary, "obligation_count"),
                    "w039_formal_exact_blocker_count": counter_value(&w039_formal_summary, "exact_remaining_blocker_count"),
                    "w040_conformance_exact_blocker_count": counter_value(&w040_conformance_summary, "exact_remaining_blocker_count"),
                    "w040_treecalc_case_count": counter_value(&w040_treecalc_summary, "case_count"),
                    "panic_marker_count": panic_marker_count
                }
            }),
        )?;
        write_json(
            &artifact_root.join("w040_rust_totality_refinement_ledger.json"),
            &json!({
                "schema_version": FORMAL_ASSURANCE_W040_RUST_LEDGER_SCHEMA_V1,
                "run_id": run_id,
                "rust_row_count": proof_rows.len(),
                "local_proof_row_count": local_proof_row_count,
                "bounded_model_row_count": bounded_model_row_count,
                "accepted_external_seam_count": accepted_external_seam_count,
                "accepted_boundary_count": accepted_boundary_count,
                "totality_boundary_count": totality_rows.len(),
                "refinement_row_count": refinement_rows.len(),
                "exact_remaining_blocker_count": blocker_rows.len(),
                "rows": proof_rows
            }),
        )?;
        write_json(
            &artifact_root.join("w040_rust_totality_boundary_register.json"),
            &json!({
                "schema_version": FORMAL_ASSURANCE_W040_TOTALITY_REGISTER_SCHEMA_V1,
                "run_id": run_id,
                "totality_boundary_count": totality_rows.len(),
                "rows": totality_rows
            }),
        )?;
        write_json(
            &artifact_root.join("w040_rust_refinement_register.json"),
            &json!({
                "schema_version": FORMAL_ASSURANCE_W040_REFINEMENT_REGISTER_SCHEMA_V1,
                "run_id": run_id,
                "refinement_row_count": refinement_rows.len(),
                "rows": refinement_rows
            }),
        )?;
        write_json(
            &artifact_root.join("w040_rust_exact_blocker_register.json"),
            &json!({
                "schema_version": FORMAL_ASSURANCE_W040_BLOCKER_REGISTER_SCHEMA_V1,
                "run_id": run_id,
                "exact_remaining_blocker_count": blocker_rows.len(),
                "rows": blocker_rows
            }),
        )?;

        let validation_status = if validation_failures.is_empty() {
            "formal_assurance_w040_rust_totality_refinement_valid"
        } else {
            "formal_assurance_w040_rust_totality_refinement_invalid"
        };
        write_json(
            &artifact_root.join("validation.json"),
            &json!({
                "schema_version": FORMAL_ASSURANCE_VALIDATION_SCHEMA_V1,
                "run_id": run_id,
                "status": validation_status,
                "rust_row_count": proof_rows.len(),
                "local_proof_row_count": local_proof_row_count,
                "bounded_model_row_count": bounded_model_row_count,
                "accepted_external_seam_count": accepted_external_seam_count,
                "accepted_boundary_count": accepted_boundary_count,
                "totality_boundary_count": totality_rows.len(),
                "refinement_row_count": refinement_rows.len(),
                "exact_remaining_blocker_count": blocker_rows.len(),
                "failed_row_count": failed_row_count,
                "validation_failures": validation_failures
            }),
        )?;
        write_json(
            &artifact_root.join("run_summary.json"),
            &json!({
                "schema_version": FORMAL_ASSURANCE_RUN_SUMMARY_SCHEMA_V1,
                "run_id": run_id,
                "artifact_root": relative_artifact_root,
                "source_evidence_index_path": source_evidence_index_path,
                "assumption_discharge_ledger_path": rust_ledger_path,
                "totality_boundary_register_path": totality_register_path,
                "refinement_register_path": refinement_register_path,
                "exact_proof_model_blocker_register_path": blocker_register_path,
                "validation_path": validation_path,
                "assumption_row_count": proof_rows.len(),
                "local_proof_row_count": local_proof_row_count,
                "bounded_model_row_count": bounded_model_row_count,
                "accepted_external_seam_count": accepted_external_seam_count,
                "accepted_boundary_count": accepted_boundary_count,
                "totality_boundary_count": totality_rows.len(),
                "refinement_row_count": refinement_rows.len(),
                "exact_remaining_blocker_count": blocker_rows.len(),
                "failed_row_count": failed_row_count,
                "promotion_claims": {
                    "rust_engine_totality_promoted": false,
                    "rust_refinement_promoted": false,
                    "full_optimized_core_verification_promoted": false,
                    "full_lean_verification_promoted": false,
                    "full_tla_verification_promoted": false,
                    "stage2_policy_promoted": false,
                    "callable_metadata_projection_promoted": false,
                    "pack_grade_replay_promoted": false,
                    "c5_promoted": false,
                    "general_oxfunc_kernel_promoted": false
                }
            }),
        )?;

        Ok(FormalAssuranceRunSummary {
            run_id: run_id.to_string(),
            schema_version: FORMAL_ASSURANCE_RUN_SUMMARY_SCHEMA_V1.to_string(),
            assumption_row_count: proof_rows.len(),
            local_proof_row_count,
            bounded_model_row_count,
            accepted_external_seam_count,
            accepted_boundary_count,
            totality_boundary_count: totality_rows.len(),
            exact_remaining_blocker_count: blocker_rows.len(),
            failed_row_count,
            artifact_root: relative_artifact_root,
        })
    }

    fn execute_w039(
        &self,
        repo_root: &Path,
        run_id: &str,
    ) -> Result<FormalAssuranceRunSummary, FormalAssuranceError> {
        let relative_artifact_root = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "formal-assurance",
            run_id,
        ]);
        let artifact_root = repo_root.join(&relative_artifact_root);
        if artifact_root.exists() {
            fs::remove_dir_all(&artifact_root).map_err(|source| {
                FormalAssuranceError::RemoveDirectory {
                    path: artifact_root.display().to_string(),
                    source,
                }
            })?;
        }
        fs::create_dir_all(&artifact_root).map_err(|source| {
            FormalAssuranceError::CreateDirectory {
                path: artifact_root.display().to_string(),
                source,
            }
        })?;

        let w039_ledger_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "release-grade-ledger",
            W039_RESIDUAL_LEDGER_RUN_ID,
            "successor_obligation_ledger.json",
        ]);
        let w038_formal_summary_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "formal-assurance",
            "w038-proof-model-assumption-discharge-001",
            "run_summary.json",
        ]);
        let w038_formal_validation_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "formal-assurance",
            "w038-proof-model-assumption-discharge-001",
            "validation.json",
        ]);
        let w038_formal_blockers_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "formal-assurance",
            "w038-proof-model-assumption-discharge-001",
            "w038_exact_proof_model_blocker_register.json",
        ]);
        let w039_conformance_summary_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "implementation-conformance",
            W039_IMPLEMENTATION_CONFORMANCE_RUN_ID,
            "run_summary.json",
        ]);
        let w039_conformance_validation_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "implementation-conformance",
            W039_IMPLEMENTATION_CONFORMANCE_RUN_ID,
            "validation.json",
        ]);
        let w039_conformance_blockers_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "implementation-conformance",
            W039_IMPLEMENTATION_CONFORMANCE_RUN_ID,
            "w039_exact_remaining_blocker_register.json",
        ]);

        let w039_ledger = read_json(repo_root, &w039_ledger_path)?;
        let w038_formal_summary = read_json(repo_root, &w038_formal_summary_path)?;
        let w038_formal_validation = read_json(repo_root, &w038_formal_validation_path)?;
        let w038_formal_blockers = read_json(repo_root, &w038_formal_blockers_path)?;
        let w039_conformance_summary = read_json(repo_root, &w039_conformance_summary_path)?;
        let w039_conformance_validation = read_json(repo_root, &w039_conformance_validation_path)?;
        let w039_conformance_blockers = read_json(repo_root, &w039_conformance_blockers_path)?;

        let lean_file_present = repo_root.join(W039_LEAN_TOTALITY_FILE).exists();
        let proof_rows = vec![
            json!({
                "row_id": "w039_proof_lean_totality_boundary",
                "w039_obligation_id": "W039-OBL-006",
                "source_inputs": ["W038 proof/model assumption-discharge", W039_LEAN_TOTALITY_FILE],
                "disposition_kind": "explicit_totality_boundary",
                "disposition": "bind W039 checked Lean classification while carrying full Lean and Rust-engine totality as exact boundaries",
                "local_checked_proof": true,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": true,
                "exact_remaining_blocker": true,
                "authority_owner": "calc-f7o.3; calc-f7o.9",
                "promotion_consequence": "full Lean verification remains unpromoted",
                "reason": "W039 adds a checked Lean classification file, but total Rust-engine proof and all external seams are not discharged.",
                "evidence_paths": [W039_LEAN_TOTALITY_FILE, &w038_formal_summary_path],
                "failures": if lean_file_present { Vec::<String>::new() } else { vec!["w039_lean_file_missing".to_string()] },
            }),
            json!({
                "row_id": "w039_model_tla_bounded_model_boundary",
                "w039_obligation_id": "W039-OBL-007",
                "source_inputs": ["W038 bounded TLC routine floor", W039_LEAN_TOTALITY_FILE],
                "disposition_kind": "bounded_model_with_exact_totality_boundary",
                "disposition": "retain the checked TLA/TLC surface as bounded model evidence while carrying unbounded/model-completeness as exact blocker",
                "local_checked_proof": false,
                "bounded_model": true,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": true,
                "exact_remaining_blocker": true,
                "authority_owner": "calc-f7o.3; calc-f7o.4; calc-f7o.9",
                "promotion_consequence": "full TLA verification remains unpromoted",
                "reason": "W039 preserves bounded model evidence and promotion predicates, but does not claim unbounded scheduler/partition model coverage.",
                "evidence_paths": ["formal/tla/CoreEngineW036Stage2Partition.tla", &w038_formal_summary_path],
                "failures": Vec::<String>::new(),
            }),
            json!({
                "row_id": "w039_rust_engine_refinement_boundary",
                "w039_obligation_id": "W039-OBL-008",
                "source_inputs": ["W039 optimized/core exact blocker disposition", W039_LEAN_TOTALITY_FILE],
                "disposition_kind": "exact_optimized_core_refinement_blocker",
                "disposition": "carry Rust-engine totality and refinement as exact blockers while optimized/core exact blockers remain",
                "local_checked_proof": true,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": true,
                "exact_remaining_blocker": true,
                "authority_owner": "calc-f7o.2; calc-f7o.3; calc-f7o.9",
                "promotion_consequence": "full formalization and optimized/core verification remain unpromoted",
                "reason": "The W039 implementation-conformance packet retains four exact optimized/core blockers, so refinement cannot be promoted.",
                "evidence_paths": [&w039_conformance_summary_path, &w039_conformance_blockers_path, W039_LEAN_TOTALITY_FILE],
                "failures": if counter_value(&w039_conformance_summary, "w039_exact_remaining_blocker_count") == 4 { Vec::<String>::new() } else { vec!["w039_conformance_blocker_count_changed".to_string()] },
            }),
            json!({
                "row_id": "w039_callable_metadata_projection_totality_boundary",
                "w039_obligation_id": "W039-OBL-004",
                "source_inputs": ["W039 optimized/core exact blocker disposition", "W038 callable metadata proof/seam blocker"],
                "disposition_kind": "exact_callable_metadata_totality_blocker",
                "disposition": "carry callable metadata projection as an exact proof/seam blocker",
                "local_checked_proof": true,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": true,
                "exact_remaining_blocker": true,
                "authority_owner": "calc-f7o.3; calc-f7o.7; external:OxFunc",
                "promotion_consequence": "callable metadata projection remains unpromoted",
                "reason": "A metadata projection fixture or carrier sufficiency proof is still absent.",
                "evidence_paths": [&w039_conformance_blockers_path, W039_LEAN_TOTALITY_FILE],
                "failures": if row_with_field_exists(&w039_conformance_blockers, "row_id", "w039_callable_metadata_projection_exact_blocker") { Vec::<String>::new() } else { vec!["w039_callable_metadata_blocker_missing".to_string()] },
            }),
            json!({
                "row_id": "w039_let_lambda_external_oxfunc_boundary",
                "w039_obligation_id": "W039-OBL-019",
                "source_inputs": ["W038 external OxFunc boundary", W039_LEAN_TOTALITY_FILE],
                "disposition_kind": "accepted_external_seam_boundary",
                "disposition": "keep LET/LAMBDA as a narrow carrier seam and general OxFunc kernels as external-owner scope",
                "local_checked_proof": false,
                "bounded_model": false,
                "accepted_external_seam": true,
                "accepted_boundary": true,
                "totality_boundary": false,
                "exact_remaining_blocker": false,
                "authority_owner": "calc-f7o.3; calc-f7o.7; external:OxFunc",
                "promotion_consequence": "general OxFunc kernels are not promoted inside OxCalc",
                "reason": "OxCalc formalization includes only the carrier seam it consumes.",
                "evidence_paths": [W039_LEAN_TOTALITY_FILE, &w038_formal_blockers_path],
                "failures": Vec::<String>::new(),
            }),
            json!({
                "row_id": "w039_stage2_partition_policy_proof_gate",
                "w039_obligation_id": "W039-OBL-009",
                "source_inputs": ["W038 proof/model blocker register", "W039 promotion-readiness map"],
                "disposition_kind": "exact_promotion_gate_blocker",
                "disposition": "carry Stage 2 partition policy as exact proof/model and replay-governance blocker",
                "local_checked_proof": false,
                "bounded_model": true,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": false,
                "exact_remaining_blocker": true,
                "authority_owner": "calc-f7o.4",
                "promotion_consequence": "Stage 2 production policy remains unpromoted",
                "reason": "Production partition analyzer soundness and observable-result invariance are owned by calc-f7o.4.",
                "evidence_paths": [&w038_formal_blockers_path, &w039_ledger_path],
                "failures": Vec::<String>::new(),
            }),
            json!({
                "row_id": "w039_pack_c5_release_proof_gate",
                "w039_obligation_id": "W039-OBL-020",
                "source_inputs": ["W039 promotion-readiness map", "W038 proof/model blocker register"],
                "disposition_kind": "exact_promotion_gate_blocker",
                "disposition": "carry pack-grade replay and C5 as release-decision blockers rather than deriving them from proof/model evidence",
                "local_checked_proof": false,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": false,
                "exact_remaining_blocker": true,
                "authority_owner": "calc-f7o.8; calc-f7o.9",
                "promotion_consequence": "pack-grade replay and C5 remain unpromoted",
                "reason": "Proof/model evidence is necessary but insufficient for pack-grade replay or C5.",
                "evidence_paths": [&w039_ledger_path, &w038_formal_blockers_path],
                "failures": Vec::<String>::new(),
            }),
        ];

        let local_proof_row_count = proof_rows
            .iter()
            .filter(|row| bool_at(row, "local_checked_proof"))
            .count();
        let bounded_model_row_count = proof_rows
            .iter()
            .filter(|row| bool_at(row, "bounded_model"))
            .count();
        let accepted_external_seam_count = proof_rows
            .iter()
            .filter(|row| bool_at(row, "accepted_external_seam"))
            .count();
        let accepted_boundary_count = proof_rows
            .iter()
            .filter(|row| bool_at(row, "accepted_boundary"))
            .count();
        let totality_rows = proof_rows
            .iter()
            .filter(|row| bool_at(row, "totality_boundary"))
            .cloned()
            .collect::<Vec<_>>();
        let model_bound_rows = proof_rows
            .iter()
            .filter(|row| bool_at(row, "bounded_model"))
            .cloned()
            .collect::<Vec<_>>();
        let blocker_rows = proof_rows
            .iter()
            .filter(|row| bool_at(row, "exact_remaining_blocker"))
            .cloned()
            .collect::<Vec<_>>();
        let failed_row_count = proof_rows
            .iter()
            .filter(|row| {
                !row.get("failures")
                    .and_then(Value::as_array)
                    .is_some_and(Vec::is_empty)
            })
            .count();

        let mut validation_failures = Vec::new();
        if counter_value(&w039_ledger, "obligation_count") != 20 {
            validation_failures.push("w039_obligation_count_changed".to_string());
        }
        if counter_value(&w038_formal_summary, "exact_remaining_blocker_count") != 6 {
            validation_failures.push("w038_formal_blocker_count_changed".to_string());
        }
        if string_value(&w038_formal_validation, "status")
            != "formal_assurance_w038_assumption_discharge_valid"
        {
            validation_failures.push("w038_formal_validation_not_valid".to_string());
        }
        if counter_value(&w038_formal_blockers, "exact_remaining_blocker_count") != 6 {
            validation_failures.push("w038_formal_blocker_register_count_changed".to_string());
        }
        if counter_value(
            &w039_conformance_summary,
            "w039_exact_remaining_blocker_count",
        ) != 4
        {
            validation_failures.push("w039_conformance_blocker_count_changed".to_string());
        }
        if string_value(&w039_conformance_validation, "status")
            != "implementation_conformance_w039_exact_blockers_valid"
        {
            validation_failures.push("w039_conformance_validation_not_valid".to_string());
        }
        if counter_value(&w039_conformance_blockers, "exact_remaining_blocker_count") != 4 {
            validation_failures.push("w039_conformance_blocker_register_count_changed".to_string());
        }
        if !lean_file_present {
            validation_failures.push("w039_lean_totality_file_missing".to_string());
        }
        if failed_row_count != 0 {
            validation_failures.push("w039_proof_model_row_failures_present".to_string());
        }
        if blocker_rows.len() != 6 {
            validation_failures.push("w039_expected_six_exact_proof_model_blockers".to_string());
        }

        let source_evidence_index_path =
            format!("{relative_artifact_root}/source_evidence_index.json");
        let proof_model_ledger_path =
            format!("{relative_artifact_root}/w039_proof_model_totality_ledger.json");
        let totality_register_path =
            format!("{relative_artifact_root}/w039_totality_boundary_register.json");
        let model_bound_register_path =
            format!("{relative_artifact_root}/w039_model_bound_register.json");
        let blocker_register_path =
            format!("{relative_artifact_root}/w039_exact_proof_model_blocker_register.json");
        let validation_path = format!("{relative_artifact_root}/validation.json");

        write_json(
            &artifact_root.join("source_evidence_index.json"),
            &json!({
                "schema_version": FORMAL_ASSURANCE_SOURCE_INDEX_SCHEMA_V1,
                "run_id": run_id,
                "source_artifacts": {
                    "w039_successor_obligation_ledger": w039_ledger_path,
                    "w038_formal_assurance_summary": w038_formal_summary_path,
                    "w038_formal_assurance_validation": w038_formal_validation_path,
                    "w038_formal_exact_blockers": w038_formal_blockers_path,
                    "w039_implementation_conformance_summary": w039_conformance_summary_path,
                    "w039_implementation_conformance_validation": w039_conformance_validation_path,
                    "w039_implementation_conformance_exact_blockers": w039_conformance_blockers_path,
                    "w039_lean_totality_file": W039_LEAN_TOTALITY_FILE
                },
                "source_counts": {
                    "w039_obligation_count": counter_value(&w039_ledger, "obligation_count"),
                    "w038_formal_exact_blocker_count": counter_value(&w038_formal_summary, "exact_remaining_blocker_count"),
                    "w039_conformance_exact_blocker_count": counter_value(&w039_conformance_summary, "w039_exact_remaining_blocker_count")
                }
            }),
        )?;
        write_json(
            &artifact_root.join("w039_proof_model_totality_ledger.json"),
            &json!({
                "schema_version": FORMAL_ASSURANCE_W039_PROOF_MODEL_LEDGER_SCHEMA_V1,
                "run_id": run_id,
                "proof_model_row_count": proof_rows.len(),
                "local_proof_row_count": local_proof_row_count,
                "bounded_model_row_count": bounded_model_row_count,
                "accepted_external_seam_count": accepted_external_seam_count,
                "accepted_boundary_count": accepted_boundary_count,
                "totality_boundary_count": totality_rows.len(),
                "exact_remaining_blocker_count": blocker_rows.len(),
                "rows": proof_rows
            }),
        )?;
        write_json(
            &artifact_root.join("w039_totality_boundary_register.json"),
            &json!({
                "schema_version": FORMAL_ASSURANCE_W039_TOTALITY_REGISTER_SCHEMA_V1,
                "run_id": run_id,
                "totality_boundary_count": totality_rows.len(),
                "rows": totality_rows
            }),
        )?;
        write_json(
            &artifact_root.join("w039_model_bound_register.json"),
            &json!({
                "schema_version": FORMAL_ASSURANCE_W039_MODEL_BOUND_REGISTER_SCHEMA_V1,
                "run_id": run_id,
                "bounded_model_row_count": model_bound_rows.len(),
                "rows": model_bound_rows
            }),
        )?;
        write_json(
            &artifact_root.join("w039_exact_proof_model_blocker_register.json"),
            &json!({
                "schema_version": FORMAL_ASSURANCE_W039_BLOCKER_REGISTER_SCHEMA_V1,
                "run_id": run_id,
                "exact_remaining_blocker_count": blocker_rows.len(),
                "rows": blocker_rows
            }),
        )?;

        let validation_status = if validation_failures.is_empty() {
            "formal_assurance_w039_totality_closure_valid"
        } else {
            "formal_assurance_w039_totality_closure_invalid"
        };
        write_json(
            &artifact_root.join("validation.json"),
            &json!({
                "schema_version": FORMAL_ASSURANCE_VALIDATION_SCHEMA_V1,
                "run_id": run_id,
                "status": validation_status,
                "proof_model_row_count": proof_rows.len(),
                "local_proof_row_count": local_proof_row_count,
                "bounded_model_row_count": bounded_model_row_count,
                "accepted_external_seam_count": accepted_external_seam_count,
                "accepted_boundary_count": accepted_boundary_count,
                "totality_boundary_count": totality_rows.len(),
                "exact_remaining_blocker_count": blocker_rows.len(),
                "failed_row_count": failed_row_count,
                "validation_failures": validation_failures
            }),
        )?;
        write_json(
            &artifact_root.join("run_summary.json"),
            &json!({
                "schema_version": FORMAL_ASSURANCE_RUN_SUMMARY_SCHEMA_V1,
                "run_id": run_id,
                "artifact_root": relative_artifact_root,
                "source_evidence_index_path": source_evidence_index_path,
                "assumption_discharge_ledger_path": proof_model_ledger_path,
                "totality_boundary_register_path": totality_register_path,
                "model_bound_register_path": model_bound_register_path,
                "exact_proof_model_blocker_register_path": blocker_register_path,
                "validation_path": validation_path,
                "assumption_row_count": proof_rows.len(),
                "local_proof_row_count": local_proof_row_count,
                "bounded_model_row_count": bounded_model_row_count,
                "accepted_external_seam_count": accepted_external_seam_count,
                "accepted_boundary_count": accepted_boundary_count,
                "totality_boundary_count": totality_rows.len(),
                "exact_remaining_blocker_count": blocker_rows.len(),
                "failed_row_count": failed_row_count,
                "promotion_claims": {
                    "full_lean_verification_promoted": false,
                    "full_tla_verification_promoted": false,
                    "rust_engine_totality_promoted": false,
                    "stage2_policy_promoted": false,
                    "pack_grade_replay_promoted": false,
                    "c5_promoted": false,
                    "general_oxfunc_kernel_promoted": false
                }
            }),
        )?;

        Ok(FormalAssuranceRunSummary {
            run_id: run_id.to_string(),
            schema_version: FORMAL_ASSURANCE_RUN_SUMMARY_SCHEMA_V1.to_string(),
            assumption_row_count: proof_rows.len(),
            local_proof_row_count,
            bounded_model_row_count,
            accepted_external_seam_count,
            accepted_boundary_count,
            totality_boundary_count: totality_rows.len(),
            exact_remaining_blocker_count: blocker_rows.len(),
            failed_row_count,
            artifact_root: relative_artifact_root,
        })
    }
}

fn evaluate_assumption_row(
    repo_root: &Path,
    spec: &AssumptionDischargeSpec,
    w037_formal_summary: &Value,
    w037_formal_validation: &Value,
    w037_formal_blockers: &Value,
    w037_stage2_decision: &Value,
    w038_conformance_summary: &Value,
    w038_conformance_blockers: &Value,
    w038_tracecalc_authority: &Value,
) -> EvaluatedAssumptionRow {
    let evidence_checks = spec
        .required_evidence_checks
        .iter()
        .map(|check_id| {
            evaluate_assumption_check(
                repo_root,
                check_id,
                w037_formal_summary,
                w037_formal_validation,
                w037_formal_blockers,
                w037_stage2_decision,
                w038_conformance_summary,
                w038_conformance_blockers,
                w038_tracecalc_authority,
            )
        })
        .collect::<Vec<_>>();
    let failures = evidence_checks
        .iter()
        .filter(|check| check.get("status").and_then(Value::as_str) != Some("passed"))
        .map(|check| {
            check
                .get("check_id")
                .and_then(Value::as_str)
                .unwrap_or("unknown_check")
                .to_string()
        })
        .collect::<Vec<_>>();
    let valid = failures.is_empty();
    let source_row = find_source_row(
        spec.source_id,
        w037_formal_blockers,
        w038_conformance_blockers,
    );

    EvaluatedAssumptionRow {
        row: json!({
            "row_id": spec.row_id,
            "source_id": spec.source_id,
            "source_row": source_row,
            "w038_obligation_id": spec.w038_obligation_id,
            "disposition_kind": spec.disposition_kind,
            "disposition": spec.disposition,
            "local_checked_proof": spec.local_checked_proof,
            "bounded_model": spec.bounded_model,
            "accepted_external_seam": spec.accepted_external_seam,
            "accepted_boundary": spec.accepted_boundary,
            "totality_boundary": spec.totality_boundary,
            "exact_remaining_blocker": spec.exact_remaining_blocker,
            "exact_remaining_blocker_bead": spec.exact_remaining_blocker_bead,
            "authority_owner": spec.authority_owner,
            "promotion_consequence": spec.promotion_consequence,
            "reason": spec.reason,
            "evidence_paths": spec.evidence_paths,
            "evidence_checks": evidence_checks,
            "failures": failures,
            "validation_state": if valid {
                "w038_assumption_disposition_validated"
            } else {
                "w038_assumption_disposition_failed"
            }
        }),
        local_checked_proof: spec.local_checked_proof,
        bounded_model: spec.bounded_model,
        accepted_external_seam: spec.accepted_external_seam,
        accepted_boundary: spec.accepted_boundary,
        totality_boundary: spec.totality_boundary,
        exact_remaining_blocker: spec.exact_remaining_blocker,
        valid,
    }
}

fn evaluate_assumption_check(
    repo_root: &Path,
    check_id: &str,
    w037_formal_summary: &Value,
    w037_formal_validation: &Value,
    w037_formal_blockers: &Value,
    w037_stage2_decision: &Value,
    w038_conformance_summary: &Value,
    w038_conformance_blockers: &Value,
    w038_tracecalc_authority: &Value,
) -> Value {
    match check_id {
        "w037_formal_inventory_all_checked" => {
            let passed = bool_at(w037_formal_summary, "all_checked_artifacts_passed")
                && counter_value(w037_formal_summary, "lean_explicit_axiom_count") == 0
                && counter_value(w037_formal_summary, "lean_sorry_placeholder_count") == 0
                && counter_value(w037_formal_summary, "tla_failed_config_count") == 0;
            check(
                check_id,
                passed,
                &[
                    "docs/test-runs/core-engine/formal-inventory/w037-proof-model-closure-001/run_summary.json",
                ],
                json!({
                    "all_checked_artifacts_passed": w037_formal_summary["all_checked_artifacts_passed"].clone(),
                    "lean_file_count": w037_formal_summary["lean_file_count"].clone(),
                    "tla_routine_config_count": w037_formal_summary["tla_routine_config_count"].clone(),
                    "lean_explicit_axiom_count": w037_formal_summary["lean_explicit_axiom_count"].clone(),
                    "lean_sorry_placeholder_count": w037_formal_summary["lean_sorry_placeholder_count"].clone(),
                    "tla_failed_config_count": w037_formal_summary["tla_failed_config_count"].clone()
                }),
            )
        }
        "w037_validation_commands_all_passed" => {
            let commands = w037_formal_validation
                .get("commands")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            let passed = !commands.is_empty()
                && commands.iter().all(|command| {
                    command
                        .get("result")
                        .and_then(Value::as_str)
                        .is_some_and(|result| result.starts_with("passed"))
                });
            check(
                check_id,
                passed,
                &[
                    "docs/test-runs/core-engine/formal-inventory/w037-proof-model-closure-001/validation.json",
                ],
                json!({ "command_count": commands.len() }),
            )
        }
        "w038_lean_assumption_file_present" => {
            let passed = repo_root.join(W038_LEAN_ASSUMPTION_FILE).exists();
            check(
                check_id,
                passed,
                &[W038_LEAN_ASSUMPTION_FILE],
                json!({ "present": passed }),
            )
        }
        "w037_full_lean_blocker_present" => blocker_check(
            check_id,
            w037_formal_blockers,
            "proof.full_lean_verification_not_promoted",
        ),
        "w037_full_tla_blocker_present" => blocker_check(
            check_id,
            w037_formal_blockers,
            "model.full_tla_verification_not_promoted",
        ),
        "w037_general_oxfunc_boundary_present" => blocker_check(
            check_id,
            w037_formal_blockers,
            "callable.general_oxfunc_kernel_not_promoted",
        ),
        "w037_pack_blocker_present" => blocker_check(
            check_id,
            w037_formal_blockers,
            "pack.grade_replay_not_promoted",
        ),
        "w037_c5_blocker_present" => {
            blocker_check(check_id, w037_formal_blockers, "capability.c5_not_promoted")
        }
        "w037_spec_evolution_guard_present" => {
            blocker_check(check_id, w037_formal_blockers, "spec.evolution_not_frozen")
        }
        "w037_stage2_replay_blocker_present" => {
            let formal_blocker_present = row_with_field_exists(
                w037_formal_blockers,
                "blocker_id",
                "stage2.replay_equivalence_not_bound",
            );
            let stage2_decision_blocks = !bool_at(w037_stage2_decision, "stage2_policy_promoted")
                && array_contains_string(
                    w037_stage2_decision.get("blockers").unwrap_or(&Value::Null),
                    "stage2.deterministic_partition_replay_absent",
                );
            check(
                check_id,
                formal_blocker_present && stage2_decision_blocks,
                &[
                    "docs/test-runs/core-engine/formal-inventory/w037-proof-model-closure-001/promotion_blockers.json",
                    "docs/test-runs/core-engine/stage2-criteria/w037-stage2-deterministic-replay-criteria-001/promotion_decision.json",
                ],
                json!({
                    "formal_blocker_present": formal_blocker_present,
                    "stage2_policy_promoted": w037_stage2_decision["stage2_policy_promoted"].clone(),
                    "stage2_blockers": w037_stage2_decision["blockers"].clone()
                }),
            )
        }
        "w038_callable_metadata_blocker_present" => {
            let blocker_present = row_with_field_exists(
                w038_conformance_blockers,
                "row_id",
                "w038_disposition_callable_metadata_projection_exact_blocker",
            );
            let passed = blocker_present
                && counter_value(w038_conformance_summary, "w038_match_promoted_count") == 0;
            check(
                check_id,
                passed,
                &[
                    "docs/test-runs/core-engine/implementation-conformance/w038-optimized-core-conformance-disposition-001/run_summary.json",
                    "docs/test-runs/core-engine/implementation-conformance/w038-optimized-core-conformance-disposition-001/w038_exact_remaining_blocker_register.json",
                ],
                json!({
                    "callable_metadata_blocker_present": blocker_present,
                    "w038_match_promoted_count": w038_conformance_summary["w038_match_promoted_count"].clone()
                }),
            )
        }
        "w038_tracecalc_authority_external_exclusion" => {
            let passed = counter_value(
                w038_tracecalc_authority,
                "accepted_external_exclusion_count",
            ) == 1
                && counter_value(
                    w038_tracecalc_authority,
                    "remaining_tracecalc_authority_blocker_count",
                ) == 0
                && !bool_at(w038_tracecalc_authority, "general_oxfunc_kernel_promoted");
            check(
                check_id,
                passed,
                &[
                    "docs/test-runs/core-engine/tracecalc-authority/w038-tracecalc-authority-discharge-001/run_summary.json",
                ],
                json!({
                    "accepted_external_exclusion_count": w038_tracecalc_authority["accepted_external_exclusion_count"].clone(),
                    "remaining_tracecalc_authority_blocker_count": w038_tracecalc_authority["remaining_tracecalc_authority_blocker_count"].clone(),
                    "general_oxfunc_kernel_promoted": w038_tracecalc_authority["general_oxfunc_kernel_promoted"].clone()
                }),
            )
        }
        unknown => check(unknown, false, &[], json!({ "unknown_check": unknown })),
    }
}

fn source_validation_failures(
    w037_formal_summary: &Value,
    w037_formal_blockers: &Value,
    w038_conformance_summary: &Value,
    w038_conformance_blockers: &Value,
    w038_tracecalc_authority: &Value,
) -> Vec<String> {
    let mut failures = Vec::new();
    if counter_value(w037_formal_blockers, "blocker_count") != 7 {
        failures.push("w037_formal_blocker_count_not_7".to_string());
    }
    if !bool_at(w037_formal_summary, "all_checked_artifacts_passed") {
        failures.push("w037_formal_artifacts_not_all_checked".to_string());
    }
    if counter_value(w038_conformance_summary, "w038_match_promoted_count") != 0 {
        failures.push("w038_conformance_promoted_declared_gap".to_string());
    }
    if counter_value(w038_conformance_blockers, "exact_remaining_blocker_count") != 4 {
        failures.push("w038_conformance_exact_blocker_count_not_4".to_string());
    }
    if counter_value(
        w038_tracecalc_authority,
        "remaining_tracecalc_authority_blocker_count",
    ) != 0
    {
        failures.push("w038_tracecalc_authority_blockers_remain".to_string());
    }
    failures
}

fn blocker_check(check_id: &str, blockers: &Value, blocker_id: &str) -> Value {
    let present = row_with_field_exists(blockers, "blocker_id", blocker_id);
    check(
        check_id,
        present,
        &[
            "docs/test-runs/core-engine/formal-inventory/w037-proof-model-closure-001/promotion_blockers.json",
        ],
        json!({ "blocker_id": blocker_id, "present": present }),
    )
}

fn check(check_id: &str, passed: bool, artifact_paths: &[&str], observed: Value) -> Value {
    json!({
        "check_id": check_id,
        "status": if passed { "passed" } else { "failed" },
        "artifact_paths": artifact_paths,
        "observed": observed
    })
}

fn find_source_row(
    source_id: &str,
    formal_blockers: &Value,
    conformance_blockers: &Value,
) -> Value {
    find_row_by_field(formal_blockers, "blocker_id", source_id)
        .or_else(|| find_row_by_field(conformance_blockers, "row_id", source_id))
        .unwrap_or(Value::Null)
}

fn find_row_by_field(value: &Value, field: &str, expected: &str) -> Option<Value> {
    value
        .get("rows")
        .and_then(Value::as_array)
        .and_then(|rows| {
            rows.iter()
                .find(|row| row.get(field).and_then(Value::as_str) == Some(expected))
                .cloned()
        })
}

fn row_with_field_exists(value: &Value, field: &str, expected: &str) -> bool {
    find_row_by_field(value, field, expected).is_some()
}

fn w040_obligation_exists(value: &Value, obligation_id: &str) -> bool {
    value
        .get("obligations")
        .and_then(Value::as_array)
        .is_some_and(|rows| {
            rows.iter()
                .any(|row| row.get("obligation_id").and_then(Value::as_str) == Some(obligation_id))
        })
}

fn array_contains_string(value: &Value, expected: &str) -> bool {
    value
        .as_array()
        .is_some_and(|items| items.iter().any(|item| item.as_str() == Some(expected)))
}

fn bool_at(value: &Value, key: &str) -> bool {
    value.get(key).and_then(Value::as_bool).unwrap_or(false)
}

fn string_value<'a>(value: &'a Value, key: &str) -> &'a str {
    value.get(key).and_then(Value::as_str).unwrap_or("")
}

fn counter_value(value: &Value, key: &str) -> usize {
    value.get(key).and_then(Value::as_u64).unwrap_or(0) as usize
}

fn read_json(repo_root: &Path, relative_path: &str) -> Result<Value, FormalAssuranceError> {
    let path = repo_root.join(relative_path);
    let contents =
        fs::read_to_string(&path).map_err(|source| FormalAssuranceError::ReadArtifact {
            path: path.display().to_string(),
            source,
        })?;
    serde_json::from_str(&contents).map_err(|source| FormalAssuranceError::ParseJson {
        path: path.display().to_string(),
        source,
    })
}

fn panic_marker_count(
    repo_root: &Path,
    relative_files: &[&str],
) -> Result<usize, FormalAssuranceError> {
    let mut count = 0;
    for relative_file in relative_files {
        let path = repo_root.join(relative_file);
        let contents =
            fs::read_to_string(&path).map_err(|source| FormalAssuranceError::ReadArtifact {
                path: path.display().to_string(),
                source,
            })?;
        count += contents
            .lines()
            .filter(|line| {
                line.contains(".unwrap(")
                    || line.contains(".expect(")
                    || line.contains("panic!(")
                    || line.contains("todo!(")
                    || line.contains("unimplemented!(")
            })
            .count();
    }
    Ok(count)
}

fn lean_placeholder_count(repo_root: &Path) -> Result<usize, FormalAssuranceError> {
    let lean_root = repo_root.join("formal/lean");
    let mut stack = vec![lean_root];
    let mut count = 0;
    while let Some(path) = stack.pop() {
        let entries = fs::read_dir(&path).map_err(|source| FormalAssuranceError::ReadArtifact {
            path: path.display().to_string(),
            source,
        })?;
        for entry in entries {
            let entry = entry.map_err(|source| FormalAssuranceError::ReadArtifact {
                path: path.display().to_string(),
                source,
            })?;
            let entry_path = entry.path();
            if entry_path.is_dir() {
                stack.push(entry_path);
                continue;
            }
            if entry_path
                .extension()
                .and_then(|extension| extension.to_str())
                != Some("lean")
            {
                continue;
            }
            let contents = fs::read_to_string(&entry_path).map_err(|source| {
                FormalAssuranceError::ReadArtifact {
                    path: entry_path.display().to_string(),
                    source,
                }
            })?;
            count += contents
                .lines()
                .filter(|line| {
                    let trimmed = line.trim_start();
                    trimmed.starts_with("axiom ")
                        || trimmed == "sorry"
                        || trimmed.starts_with("sorry ")
                        || trimmed == "admit"
                        || trimmed.starts_with("admit ")
                })
                .count();
        }
    }
    Ok(count)
}

fn write_json(path: &Path, value: &Value) -> Result<(), FormalAssuranceError> {
    let bytes =
        serde_json::to_vec_pretty(value).map_err(|source| FormalAssuranceError::ParseJson {
            path: path.display().to_string(),
            source,
        })?;
    fs::write(path, bytes).map_err(|source| FormalAssuranceError::WriteFile {
        path: path.display().to_string(),
        source,
    })
}

fn relative_artifact_path(parts: &[&str]) -> String {
    parts.join("/")
}

const W038_ASSUMPTION_DISCHARGE_SPECS: &[AssumptionDischargeSpec] = &[
    AssumptionDischargeSpec {
        row_id: "w038_proof_full_lean_totality_boundary",
        source_id: "proof.full_lean_verification_not_promoted",
        w038_obligation_id: "W038-OBL-008",
        disposition_kind: "explicit_totality_boundary",
        disposition: "bind checked Lean inventory and W038 assumption-discharge file while carrying Rust-engine totality as an exact proof boundary",
        local_checked_proof: true,
        bounded_model: false,
        accepted_external_seam: false,
        accepted_boundary: false,
        totality_boundary: true,
        exact_remaining_blocker: true,
        exact_remaining_blocker_bead: Some("calc-zsr.9"),
        authority_owner: "calc-zsr.4; calc-zsr.9",
        promotion_consequence: "full Lean verification remains unpromoted until Rust-engine totality and all external seams are proven for the claimed scope",
        reason: "W037 proves an axiom-free, placeholder-free inventory floor; W038 makes the remaining totality boundary explicit rather than counting inventory as total proof.",
        evidence_paths: &[
            W038_LEAN_ASSUMPTION_FILE,
            "formal/lean/OxCalc/CoreEngine/W037ProofModelClosureInventory.lean",
            "docs/test-runs/core-engine/formal-inventory/w037-proof-model-closure-001/run_summary.json",
            "docs/test-runs/core-engine/formal-inventory/w037-proof-model-closure-001/validation.json",
        ],
        required_evidence_checks: &[
            "w037_formal_inventory_all_checked",
            "w037_validation_commands_all_passed",
            "w038_lean_assumption_file_present",
            "w037_full_lean_blocker_present",
        ],
    },
    AssumptionDischargeSpec {
        row_id: "w038_model_full_tla_bounded_model_boundary",
        source_id: "model.full_tla_verification_not_promoted",
        w038_obligation_id: "W038-OBL-009",
        disposition_kind: "bounded_model_with_exact_totality_boundary",
        disposition: "bind the routine TLC floor as bounded model evidence while carrying unbounded model coverage as an exact boundary",
        local_checked_proof: false,
        bounded_model: true,
        accepted_external_seam: false,
        accepted_boundary: false,
        totality_boundary: true,
        exact_remaining_blocker: true,
        exact_remaining_blocker_bead: Some("calc-zsr.5; calc-zsr.9"),
        authority_owner: "calc-zsr.4; calc-zsr.5; calc-zsr.9",
        promotion_consequence: "full TLA verification remains unpromoted until bounded configs are widened or the unbounded/model-completeness claim is otherwise discharged",
        reason: "The W037 inventory and W036 Stage 2 model checks are valuable bounded evidence, not an unbounded proof over the scheduler and partition universe.",
        evidence_paths: &[
            "formal/tla/CoreEngineW036Stage2Partition.tla",
            "docs/test-runs/core-engine/formal-inventory/w037-proof-model-closure-001/tla_inventory.json",
            "docs/test-runs/core-engine/formal-inventory/w037-proof-model-closure-001/run_summary.json",
        ],
        required_evidence_checks: &[
            "w037_formal_inventory_all_checked",
            "w037_full_tla_blocker_present",
        ],
    },
    AssumptionDischargeSpec {
        row_id: "w038_external_general_oxfunc_lambda_kernel_boundary",
        source_id: "callable.general_oxfunc_kernel_not_promoted",
        w038_obligation_id: "W038-OBL-010",
        disposition_kind: "accepted_external_seam_boundary",
        disposition: "accept general OxFunc callable-kernel semantics as external while preserving the narrow OxCalc LET/LAMBDA carrier proof surface",
        local_checked_proof: false,
        bounded_model: false,
        accepted_external_seam: true,
        accepted_boundary: true,
        totality_boundary: false,
        exact_remaining_blocker: false,
        exact_remaining_blocker_bead: None,
        authority_owner: "external:OxFunc; calc-zsr.7 seam watch",
        promotion_consequence: "OxCalc may claim only the narrow carrier boundary it exercises; it does not promote general OxFunc kernel semantics",
        reason: "W038 TraceCalc authority already accepts the general OxFunc kernel row as external, and W037 proof inventory keeps the opaque-kernel boundary explicit.",
        evidence_paths: &[
            "formal/lean/OxCalc/CoreEngine/W036CallableBoundaryInventory.lean",
            "formal/lean/OxCalc/CoreEngine/W037ProofModelClosureInventory.lean",
            "docs/test-runs/core-engine/tracecalc-authority/w038-tracecalc-authority-discharge-001/run_summary.json",
        ],
        required_evidence_checks: &[
            "w037_general_oxfunc_boundary_present",
            "w038_tracecalc_authority_external_exclusion",
        ],
    },
    AssumptionDischargeSpec {
        row_id: "w038_model_stage2_replay_equivalence_exact_blocker",
        source_id: "stage2.replay_equivalence_not_bound",
        w038_obligation_id: "W038-OBL-009",
        disposition_kind: "exact_remaining_blocker",
        disposition: "carry Stage 2 replay/equivalence as an exact downstream model-and-replay blocker",
        local_checked_proof: false,
        bounded_model: true,
        accepted_external_seam: false,
        accepted_boundary: false,
        totality_boundary: false,
        exact_remaining_blocker: true,
        exact_remaining_blocker_bead: Some("calc-zsr.5"),
        authority_owner: "calc-zsr.5",
        promotion_consequence: "Stage 2 policy remains unpromoted until deterministic partition replay and observable-result invariance evidence are executed",
        reason: "W037 Stage 2 criteria still names deterministic partition replay absence despite bounded TLA partition models.",
        evidence_paths: &[
            "formal/lean/OxCalc/CoreEngine/W037Stage2PromotionCriteria.lean",
            "docs/test-runs/core-engine/stage2-criteria/w037-stage2-deterministic-replay-criteria-001/promotion_decision.json",
        ],
        required_evidence_checks: &["w037_stage2_replay_blocker_present"],
    },
    AssumptionDischargeSpec {
        row_id: "w038_pack_grade_replay_exact_blocker",
        source_id: "pack.grade_replay_not_promoted",
        w038_obligation_id: "W038-OBL-019",
        disposition_kind: "exact_remaining_blocker",
        disposition: "carry pack-grade replay as a pack-governance blocker rather than deriving it from proof/model inventory",
        local_checked_proof: false,
        bounded_model: false,
        accepted_external_seam: false,
        accepted_boundary: false,
        totality_boundary: false,
        exact_remaining_blocker: true,
        exact_remaining_blocker_bead: Some("calc-zsr.8"),
        authority_owner: "calc-zsr.8",
        promotion_consequence: "pack-grade replay remains unpromoted until replay governance and retained-witness service evidence are bound",
        reason: "Proof/model evidence is necessary but not sufficient for pack-grade replay promotion.",
        evidence_paths: &[
            "docs/test-runs/core-engine/formal-inventory/w037-proof-model-closure-001/promotion_blockers.json",
        ],
        required_evidence_checks: &["w037_pack_blocker_present"],
    },
    AssumptionDischargeSpec {
        row_id: "w038_c5_exact_blocker",
        source_id: "capability.c5_not_promoted",
        w038_obligation_id: "W038-OBL-020",
        disposition_kind: "exact_remaining_blocker",
        disposition: "carry C5 as a release-decision blocker rather than deriving it from proof/model inventory",
        local_checked_proof: false,
        bounded_model: false,
        accepted_external_seam: false,
        accepted_boundary: false,
        totality_boundary: false,
        exact_remaining_blocker: true,
        exact_remaining_blocker_bead: Some("calc-zsr.8; calc-zsr.9"),
        authority_owner: "calc-zsr.8; calc-zsr.9",
        promotion_consequence: "C5 remains unpromoted until direct W038 evidence satisfies pack, service, Stage 2, proof/model, and conformance gates",
        reason: "Checked proof/model artifacts do not by themselves satisfy the broader C5 release-grade decision surface.",
        evidence_paths: &[
            "docs/test-runs/core-engine/formal-inventory/w037-proof-model-closure-001/promotion_blockers.json",
        ],
        required_evidence_checks: &["w037_c5_blocker_present"],
    },
    AssumptionDischargeSpec {
        row_id: "w038_spec_evolution_guard",
        source_id: "spec.evolution_not_frozen",
        w038_obligation_id: "W038-OBL-008",
        disposition_kind: "accepted_spec_evolution_guard",
        disposition: "preserve spec evolution as an explicit guard while binding current proof/model assumptions",
        local_checked_proof: true,
        bounded_model: false,
        accepted_external_seam: false,
        accepted_boundary: true,
        totality_boundary: false,
        exact_remaining_blocker: false,
        exact_remaining_blocker_bead: None,
        authority_owner: "calc-zsr.4; calc-zsr.9",
        promotion_consequence: "future proof/model evidence may correct specs; W038 does not freeze the initial model universe",
        reason: "The formalization path uses proof/model evidence to evolve the spec rather than testing against a fixed initial document set.",
        evidence_paths: &[
            "formal/lean/OxCalc/CoreEngine/W038AssumptionDischargeAndTotality.lean",
            "docs/test-runs/core-engine/formal-inventory/w037-proof-model-closure-001/promotion_blockers.json",
        ],
        required_evidence_checks: &[
            "w038_lean_assumption_file_present",
            "w037_spec_evolution_guard_present",
        ],
    },
    AssumptionDischargeSpec {
        row_id: "w038_callable_metadata_projection_exact_blocker",
        source_id: "w038_disposition_callable_metadata_projection_exact_blocker",
        w038_obligation_id: "W038-OBL-007",
        disposition_kind: "exact_remaining_proof_seam_blocker",
        disposition: "carry callable metadata projection as an exact proof/seam blocker while preserving value-only TreeCalc and direct OxFml callable-carrier evidence",
        local_checked_proof: true,
        bounded_model: false,
        accepted_external_seam: false,
        accepted_boundary: false,
        totality_boundary: true,
        exact_remaining_blocker: true,
        exact_remaining_blocker_bead: Some("calc-zsr.7"),
        authority_owner: "calc-zsr.4; calc-zsr.7; external:OxFunc",
        promotion_consequence: "callable metadata projection remains unpromoted until carrier sufficiency is proven or a metadata projection fixture is added",
        reason: "The W038 conformance disposition proves the value-only carrier boundary but not metadata projection totality or sufficiency.",
        evidence_paths: &[
            "docs/test-runs/core-engine/implementation-conformance/w038-optimized-core-conformance-disposition-001/w038_exact_remaining_blocker_register.json",
            "formal/lean/OxCalc/CoreEngine/W036CallableBoundaryInventory.lean",
            "formal/lean/OxCalc/CoreEngine/W038AssumptionDischargeAndTotality.lean",
        ],
        required_evidence_checks: &[
            "w038_callable_metadata_blocker_present",
            "w038_lean_assumption_file_present",
        ],
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn formal_assurance_runner_classifies_w038_assumptions() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let run_id = format!("test-w038-formal-assurance-{}", std::process::id());
        let artifact_root = repo_root.join(format!(
            "docs/test-runs/core-engine/formal-assurance/{run_id}"
        ));
        let cleanup = || {
            if artifact_root.exists() {
                let _ = fs::remove_dir_all(&artifact_root);
            }
        };

        cleanup();
        let summary = FormalAssuranceRunner::new()
            .execute(&repo_root, &run_id)
            .unwrap();

        assert_eq!(summary.assumption_row_count, 8);
        assert_eq!(summary.local_proof_row_count, 3);
        assert_eq!(summary.bounded_model_row_count, 2);
        assert_eq!(summary.accepted_external_seam_count, 1);
        assert_eq!(summary.accepted_boundary_count, 2);
        assert_eq!(summary.totality_boundary_count, 3);
        assert_eq!(summary.exact_remaining_blocker_count, 6);
        assert_eq!(summary.failed_row_count, 0);

        let validation = read_json(
            &repo_root,
            &format!("docs/test-runs/core-engine/formal-assurance/{run_id}/validation.json"),
        )
        .unwrap();
        assert_eq!(
            validation["status"],
            "formal_assurance_w038_assumption_discharge_valid"
        );

        cleanup();
    }

    #[test]
    fn formal_assurance_runner_classifies_w039_totality_boundaries() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let run_id = format!("test-w039-formal-assurance-{}", std::process::id());
        let artifact_root = repo_root.join(format!(
            "docs/test-runs/core-engine/formal-assurance/{run_id}"
        ));
        let cleanup = || {
            if artifact_root.exists() {
                let _ = fs::remove_dir_all(&artifact_root);
            }
        };

        cleanup();
        let summary = FormalAssuranceRunner::new()
            .execute(&repo_root, &run_id)
            .unwrap();

        assert_eq!(summary.assumption_row_count, 7);
        assert_eq!(summary.local_proof_row_count, 3);
        assert_eq!(summary.bounded_model_row_count, 2);
        assert_eq!(summary.accepted_external_seam_count, 1);
        assert_eq!(summary.accepted_boundary_count, 1);
        assert_eq!(summary.totality_boundary_count, 4);
        assert_eq!(summary.exact_remaining_blocker_count, 6);
        assert_eq!(summary.failed_row_count, 0);

        let validation = read_json(
            &repo_root,
            &format!("docs/test-runs/core-engine/formal-assurance/{run_id}/validation.json"),
        )
        .unwrap();
        assert_eq!(
            validation["status"],
            "formal_assurance_w039_totality_closure_valid"
        );

        let blocker_register = read_json(
            &repo_root,
            &format!(
                "docs/test-runs/core-engine/formal-assurance/{run_id}/w039_exact_proof_model_blocker_register.json"
            ),
        )
        .unwrap();
        assert_eq!(blocker_register["exact_remaining_blocker_count"], 6);

        cleanup();
    }

    #[test]
    fn formal_assurance_runner_classifies_w040_rust_totality_and_refinement() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let run_id = format!("test-w040-rust-formal-assurance-{}", std::process::id());
        let artifact_root = repo_root.join(format!(
            "docs/test-runs/core-engine/formal-assurance/{run_id}"
        ));
        let cleanup = || {
            if artifact_root.exists() {
                let _ = fs::remove_dir_all(&artifact_root);
            }
        };

        cleanup();
        let summary = FormalAssuranceRunner::new()
            .execute(&repo_root, &run_id)
            .unwrap();

        assert_eq!(summary.assumption_row_count, 10);
        assert_eq!(summary.local_proof_row_count, 7);
        assert_eq!(summary.bounded_model_row_count, 0);
        assert_eq!(summary.accepted_external_seam_count, 1);
        assert_eq!(summary.accepted_boundary_count, 2);
        assert_eq!(summary.totality_boundary_count, 5);
        assert_eq!(summary.exact_remaining_blocker_count, 5);
        assert_eq!(summary.failed_row_count, 0);

        let validation = read_json(
            &repo_root,
            &format!("docs/test-runs/core-engine/formal-assurance/{run_id}/validation.json"),
        )
        .unwrap();
        assert_eq!(
            validation["status"],
            "formal_assurance_w040_rust_totality_refinement_valid"
        );

        let blocker_register = read_json(
            &repo_root,
            &format!(
                "docs/test-runs/core-engine/formal-assurance/{run_id}/w040_rust_exact_blocker_register.json"
            ),
        )
        .unwrap();
        assert_eq!(blocker_register["exact_remaining_blocker_count"], 5);

        cleanup();
    }

    #[test]
    fn formal_assurance_runner_classifies_w040_lean_tla_discharge() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let run_id = format!("test-w040-lean-tla-formal-assurance-{}", std::process::id());
        let artifact_root = repo_root.join(format!(
            "docs/test-runs/core-engine/formal-assurance/{run_id}"
        ));
        let cleanup = || {
            if artifact_root.exists() {
                let _ = fs::remove_dir_all(&artifact_root);
            }
        };

        cleanup();
        let summary = FormalAssuranceRunner::new()
            .execute(&repo_root, &run_id)
            .unwrap();

        assert_eq!(summary.assumption_row_count, 11);
        assert_eq!(summary.local_proof_row_count, 6);
        assert_eq!(summary.bounded_model_row_count, 3);
        assert_eq!(summary.accepted_external_seam_count, 1);
        assert_eq!(summary.accepted_boundary_count, 2);
        assert_eq!(summary.totality_boundary_count, 5);
        assert_eq!(summary.exact_remaining_blocker_count, 5);
        assert_eq!(summary.failed_row_count, 0);

        let validation = read_json(
            &repo_root,
            &format!("docs/test-runs/core-engine/formal-assurance/{run_id}/validation.json"),
        )
        .unwrap();
        assert_eq!(
            validation["status"],
            "formal_assurance_w040_lean_tla_discharge_valid"
        );

        let blocker_register = read_json(
            &repo_root,
            &format!(
                "docs/test-runs/core-engine/formal-assurance/{run_id}/w040_lean_tla_exact_blocker_register.json"
            ),
        )
        .unwrap();
        assert_eq!(blocker_register["exact_remaining_blocker_count"], 5);

        cleanup();
    }
}
