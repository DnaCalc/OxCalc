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
const FORMAL_ASSURANCE_W041_RUST_LEDGER_SCHEMA_V1: &str =
    "oxcalc.formal_assurance.w041_rust_totality_refinement_ledger.v1";
const FORMAL_ASSURANCE_W041_TOTALITY_REGISTER_SCHEMA_V1: &str =
    "oxcalc.formal_assurance.w041_rust_totality_boundary_register.v1";
const FORMAL_ASSURANCE_W041_REFINEMENT_REGISTER_SCHEMA_V1: &str =
    "oxcalc.formal_assurance.w041_rust_refinement_register.v1";
const FORMAL_ASSURANCE_W041_BLOCKER_REGISTER_SCHEMA_V1: &str =
    "oxcalc.formal_assurance.w041_rust_exact_blocker_register.v1";
const FORMAL_ASSURANCE_W041_LEAN_TLA_LEDGER_SCHEMA_V1: &str =
    "oxcalc.formal_assurance.w041_lean_tla_discharge_ledger.v1";
const FORMAL_ASSURANCE_W041_LEAN_PROOF_REGISTER_SCHEMA_V1: &str =
    "oxcalc.formal_assurance.w041_lean_proof_register.v1";
const FORMAL_ASSURANCE_W041_TLA_MODEL_REGISTER_SCHEMA_V1: &str =
    "oxcalc.formal_assurance.w041_tla_model_bound_register.v1";
const FORMAL_ASSURANCE_W041_LEAN_TLA_BLOCKER_REGISTER_SCHEMA_V1: &str =
    "oxcalc.formal_assurance.w041_lean_tla_exact_blocker_register.v1";
const FORMAL_ASSURANCE_W042_RUST_LEDGER_SCHEMA_V1: &str =
    "oxcalc.formal_assurance.w042_rust_totality_refinement_ledger.v1";
const FORMAL_ASSURANCE_W042_TOTALITY_REGISTER_SCHEMA_V1: &str =
    "oxcalc.formal_assurance.w042_rust_totality_boundary_register.v1";
const FORMAL_ASSURANCE_W042_REFINEMENT_REGISTER_SCHEMA_V1: &str =
    "oxcalc.formal_assurance.w042_rust_refinement_register.v1";
const FORMAL_ASSURANCE_W042_BLOCKER_REGISTER_SCHEMA_V1: &str =
    "oxcalc.formal_assurance.w042_rust_exact_blocker_register.v1";
const FORMAL_ASSURANCE_W042_LEAN_TLA_LEDGER_SCHEMA_V1: &str =
    "oxcalc.formal_assurance.w042_lean_tla_discharge_ledger.v1";
const FORMAL_ASSURANCE_W042_LEAN_PROOF_REGISTER_SCHEMA_V1: &str =
    "oxcalc.formal_assurance.w042_lean_proof_register.v1";
const FORMAL_ASSURANCE_W042_TLA_MODEL_REGISTER_SCHEMA_V1: &str =
    "oxcalc.formal_assurance.w042_tla_model_bound_register.v1";
const FORMAL_ASSURANCE_W042_LEAN_TLA_BLOCKER_REGISTER_SCHEMA_V1: &str =
    "oxcalc.formal_assurance.w042_lean_tla_exact_blocker_register.v1";
const FORMAL_ASSURANCE_W043_RUST_LEDGER_SCHEMA_V1: &str =
    "oxcalc.formal_assurance.w043_rust_totality_refinement_ledger.v1";
const FORMAL_ASSURANCE_W043_TOTALITY_REGISTER_SCHEMA_V1: &str =
    "oxcalc.formal_assurance.w043_rust_totality_boundary_register.v1";
const FORMAL_ASSURANCE_W043_REFINEMENT_REGISTER_SCHEMA_V1: &str =
    "oxcalc.formal_assurance.w043_rust_refinement_register.v1";
const FORMAL_ASSURANCE_W043_BLOCKER_REGISTER_SCHEMA_V1: &str =
    "oxcalc.formal_assurance.w043_rust_exact_blocker_register.v1";
const FORMAL_ASSURANCE_W043_LEAN_TLA_LEDGER_SCHEMA_V1: &str =
    "oxcalc.formal_assurance.w043_lean_tla_discharge_ledger.v1";
const FORMAL_ASSURANCE_W043_LEAN_PROOF_REGISTER_SCHEMA_V1: &str =
    "oxcalc.formal_assurance.w043_lean_proof_register.v1";
const FORMAL_ASSURANCE_W043_TLA_MODEL_REGISTER_SCHEMA_V1: &str =
    "oxcalc.formal_assurance.w043_tla_model_bound_register.v1";
const FORMAL_ASSURANCE_W043_LEAN_TLA_BLOCKER_REGISTER_SCHEMA_V1: &str =
    "oxcalc.formal_assurance.w043_lean_tla_exact_blocker_register.v1";
const FORMAL_ASSURANCE_W044_RUST_LEDGER_SCHEMA_V1: &str =
    "oxcalc.formal_assurance.w044_rust_totality_refinement_ledger.v1";
const FORMAL_ASSURANCE_W044_TOTALITY_REGISTER_SCHEMA_V1: &str =
    "oxcalc.formal_assurance.w044_rust_totality_boundary_register.v1";
const FORMAL_ASSURANCE_W044_REFINEMENT_REGISTER_SCHEMA_V1: &str =
    "oxcalc.formal_assurance.w044_rust_refinement_register.v1";
const FORMAL_ASSURANCE_W044_BLOCKER_REGISTER_SCHEMA_V1: &str =
    "oxcalc.formal_assurance.w044_rust_exact_blocker_register.v1";
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
const W040_LEAN_TLA_FORMAL_ASSURANCE_RUN_ID: &str = "w040-lean-tla-full-verification-discharge-001";
const W041_RESIDUAL_LEDGER_RUN_ID: &str =
    "w041-residual-release-grade-successor-obligation-map-001";
const W041_IMPLEMENTATION_CONFORMANCE_RUN_ID: &str =
    "w041-optimized-core-residual-blocker-differentials-001";
const W041_TREECALC_RUN_ID: &str = "w041-optimized-core-automatic-dynamic-transition-001";
const W041_RUST_FORMAL_ASSURANCE_RUN_ID: &str = "w041-rust-totality-refinement-proof-tranche-001";
const W041_LEAN_RUST_TOTALITY_FILE: &str =
    "formal/lean/OxCalc/CoreEngine/W041RustTotalityAndRefinement.lean";
const W041_LEAN_TLA_DISCHARGE_FILE: &str =
    "formal/lean/OxCalc/CoreEngine/W041LeanTlaFullVerificationAndFairnessDischarge.lean";
const W041_LEAN_TLA_FORMAL_ASSURANCE_RUN_ID: &str =
    "w041-lean-tla-full-verification-fairness-discharge-001";
const W041_STAGE2_POLICY_FILE: &str =
    "formal/lean/OxCalc/CoreEngine/W041Stage2ProductionAnalyzerAndPackEquivalence.lean";
const W041_STAGE2_REPLAY_RUN_ID: &str = "w041-stage2-production-analyzer-pack-equivalence-001";
const W042_RESIDUAL_LEDGER_RUN_ID: &str =
    "w042-residual-release-grade-closure-obligation-ledger-001";
const W042_IMPLEMENTATION_CONFORMANCE_RUN_ID: &str =
    "w042-optimized-core-counterpart-conformance-callable-metadata-001";
const W042_TREECALC_RUN_ID: &str = "w042-optimized-core-counterpart-conformance-treecalc-001";
const W042_LEAN_RUST_TOTALITY_FILE: &str =
    "formal/lean/OxCalc/CoreEngine/W042RustTotalityAndRefinement.lean";
const W042_RUST_FORMAL_ASSURANCE_RUN_ID: &str =
    "w042-rust-totality-refinement-core-panic-boundary-001";
const W042_LEAN_TLA_DISCHARGE_FILE: &str =
    "formal/lean/OxCalc/CoreEngine/W042LeanTlaFairnessFullVerificationExpansion.lean";
const W042_LEAN_TLA_FORMAL_ASSURANCE_RUN_ID: &str =
    "w042-lean-tla-fairness-full-verification-expansion-001";
const W042_STAGE2_POLICY_FILE: &str =
    "formal/lean/OxCalc/CoreEngine/W042Stage2ProductionAnalyzerAndPackGradeEquivalence.lean";
const W042_STAGE2_REPLAY_RUN_ID: &str =
    "w042-stage2-production-analyzer-pack-grade-equivalence-closure-001";
const W043_RESIDUAL_LEDGER_RUN_ID: &str =
    "w043-residual-release-grade-proof-service-obligation-map-001";
const W043_IMPLEMENTATION_CONFORMANCE_RUN_ID: &str =
    "w043-optimized-core-broad-conformance-callable-metadata-closure-001";
const W043_TREECALC_RUN_ID: &str = "w043-optimized-core-broad-conformance-treecalc-001";
const W043_LEAN_RUST_TOTALITY_FILE: &str =
    "formal/lean/OxCalc/CoreEngine/W043RustTotalityAndRefinement.lean";
const W043_LEAN_TLA_DISCHARGE_FILE: &str =
    "formal/lean/OxCalc/CoreEngine/W043LeanTlaFullVerificationAndFairness.lean";
const W043_RUST_FORMAL_ASSURANCE_RUN_ID: &str =
    "w043-rust-totality-refinement-panic-free-frontier-001";
const W043_LEAN_TLA_FORMAL_ASSURANCE_RUN_ID: &str =
    "w043-lean-tla-full-verification-unbounded-fairness-001";
const W044_RESIDUAL_LEDGER_RUN_ID: &str =
    "w044-residual-release-grade-blocker-reclassification-map-001";
const W044_IMPLEMENTATION_CONFORMANCE_RUN_ID: &str =
    "w044-optimized-core-dynamic-transition-callable-metadata-001";
const W044_TREECALC_RUN_ID: &str = "w044-optimized-core-dynamic-transition-treecalc-001";
const W044_LEAN_RUST_TOTALITY_FILE: &str =
    "formal/lean/OxCalc/CoreEngine/W044RustTotalityAndRefinement.lean";
const W044_RUST_FORMAL_ASSURANCE_RUN_ID: &str =
    "w044-rust-totality-refinement-panic-surface-expansion-001";
const W039_STAGE2_POLICY_FILE: &str =
    "formal/lean/OxCalc/CoreEngine/W039Stage2ProductionPolicy.lean";
const W040_STAGE2_POLICY_FILE: &str =
    "formal/lean/OxCalc/CoreEngine/W040Stage2ProductionPolicyAndEquivalence.lean";
const W040_STAGE2_REPLAY_RUN_ID: &str = "w040-stage2-production-policy-equivalence-001";
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
        if run_id == W044_RUST_FORMAL_ASSURANCE_RUN_ID || run_id.contains("w044-rust") {
            return self.execute_w044_rust_totality_refinement(repo_root, run_id);
        }
        if run_id.contains("w043-lean-tla") {
            return self.execute_w043_lean_tla_discharge(repo_root, run_id);
        }
        if run_id.contains("w043-rust") {
            return self.execute_w043_rust_totality_refinement(repo_root, run_id);
        }
        if run_id.contains("w042-lean-tla") {
            return self.execute_w042_lean_tla_fairness_expansion(repo_root, run_id);
        }
        if run_id.contains("w042-rust") {
            return self.execute_w042_rust_totality_refinement(repo_root, run_id);
        }
        if run_id.contains("w041-lean-tla") {
            return self.execute_w041_lean_tla_discharge(repo_root, run_id);
        }
        if run_id.contains("w041-rust") {
            return self.execute_w041_rust_totality_refinement(repo_root, run_id);
        }
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

    fn execute_w041_rust_totality_refinement(
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

        let w041_obligation_summary_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "release-grade-ledger",
            W041_RESIDUAL_LEDGER_RUN_ID,
            "run_summary.json",
        ]);
        let w041_obligation_map_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "release-grade-ledger",
            W041_RESIDUAL_LEDGER_RUN_ID,
            "successor_obligation_map.json",
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
        let w041_conformance_summary_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "implementation-conformance",
            W041_IMPLEMENTATION_CONFORMANCE_RUN_ID,
            "run_summary.json",
        ]);
        let w041_conformance_validation_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "implementation-conformance",
            W041_IMPLEMENTATION_CONFORMANCE_RUN_ID,
            "validation.json",
        ]);
        let w041_conformance_blockers_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "implementation-conformance",
            W041_IMPLEMENTATION_CONFORMANCE_RUN_ID,
            "w041_exact_remaining_blocker_register.json",
        ]);
        let w041_dynamic_evidence_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "implementation-conformance",
            W041_IMPLEMENTATION_CONFORMANCE_RUN_ID,
            "dynamic_release_reclassification_auto_transition_evidence.json",
        ]);
        let w041_treecalc_summary_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "treecalc-local",
            W041_TREECALC_RUN_ID,
            "run_summary.json",
        ]);
        let w041_treecalc_auto_post_edit_result_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "treecalc-local",
            W041_TREECALC_RUN_ID,
            "cases",
            "tc_local_dynamic_release_reclassification_auto_post_edit_001",
            "post_edit",
            "result.json",
        ]);
        let w041_treecalc_auto_post_edit_closure_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "treecalc-local",
            W041_TREECALC_RUN_ID,
            "cases",
            "tc_local_dynamic_release_reclassification_auto_post_edit_001",
            "post_edit",
            "invalidation_closure.json",
        ]);
        let w041_treecalc_auto_post_edit_seeds_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "treecalc-local",
            W041_TREECALC_RUN_ID,
            "cases",
            "tc_local_dynamic_release_reclassification_auto_post_edit_001",
            "post_edit",
            "invalidation_seeds.json",
        ]);

        let w041_obligation_summary = read_json(repo_root, &w041_obligation_summary_path)?;
        let w041_obligation_map = read_json(repo_root, &w041_obligation_map_path)?;
        let w040_rust_summary = read_json(repo_root, &w040_rust_summary_path)?;
        let w040_rust_validation = read_json(repo_root, &w040_rust_validation_path)?;
        let w040_rust_blockers = read_json(repo_root, &w040_rust_blockers_path)?;
        let w041_conformance_summary = read_json(repo_root, &w041_conformance_summary_path)?;
        let w041_conformance_validation = read_json(repo_root, &w041_conformance_validation_path)?;
        let w041_conformance_blockers = read_json(repo_root, &w041_conformance_blockers_path)?;
        let w041_dynamic_evidence = read_json(repo_root, &w041_dynamic_evidence_path)?;
        let w041_treecalc_summary = read_json(repo_root, &w041_treecalc_summary_path)?;
        let w041_treecalc_auto_post_edit_result =
            read_json(repo_root, &w041_treecalc_auto_post_edit_result_path)?;
        let w041_treecalc_auto_post_edit_closure =
            read_json(repo_root, &w041_treecalc_auto_post_edit_closure_path)?;
        let w041_treecalc_auto_post_edit_seeds =
            read_json(repo_root, &w041_treecalc_auto_post_edit_seeds_path)?;

        let lean_file_present = repo_root.join(W041_LEAN_RUST_TOTALITY_FILE).exists();
        let w040_lean_file_present = repo_root.join(W040_LEAN_RUST_TOTALITY_FILE).exists();
        let panic_marker_count = panic_marker_count(repo_root, W040_RUST_PANIC_AUDIT_FILES)?;
        let automatic_transition_seeds_present = w041_treecalc_auto_post_edit_seeds
            .as_array()
            .is_some_and(|seeds| {
                seeds.iter().any(|seed| {
                    seed.get("reason").and_then(Value::as_str) == Some("DependencyRemoved")
                }) && seeds.iter().any(|seed| {
                    seed.get("reason").and_then(Value::as_str) == Some("DependencyReclassified")
                })
            });
        let automatic_transition_closure_requires_rebind = w041_treecalc_auto_post_edit_closure
            .as_array()
            .is_some_and(|rows| {
                rows.iter().any(|row| {
                    row.get("node_id").and_then(Value::as_u64) == Some(3)
                        && bool_at(row, "requires_rebind")
                })
            });
        let automatic_transition_rejected_for_rebind =
            string_value(&w041_treecalc_auto_post_edit_result, "result_state") == "rejected"
                && w041_treecalc_auto_post_edit_result
                    .get("reject_detail")
                    .and_then(|detail| detail.get("kind"))
                    .and_then(Value::as_str)
                    == Some("HostInjectedFailure");

        let proof_rows = vec![
            json!({
                "row_id": "w041_result_error_carrier_totality_evidence",
                "w041_obligation_id": "W041-OBL-007",
                "source_inputs": ["Rust typed error carriers", W041_LEAN_RUST_TOTALITY_FILE],
                "disposition_kind": "direct_totality_evidence",
                "disposition": "retain promoted core public paths as typed Result/error carrier evidence rather than panic-as-contract",
                "local_checked_proof": true,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": false,
                "refinement_row": false,
                "automatic_dynamic_transition_row": false,
                "exact_remaining_blocker": false,
                "authority_owner": "calc-sui.3",
                "promotion_consequence": "Rust totality remains unpromoted because this is carrier evidence, not whole-engine proof",
                "reason": "Core execution, fixture, runner, structural, and coordinator surfaces expose Result/typed error APIs for promoted evidence paths.",
                "evidence_paths": [
                    "src/oxcalc-core/src/coordinator.rs",
                    "src/oxcalc-core/src/recalc.rs",
                    "src/oxcalc-core/src/structural.rs",
                    "src/oxcalc-core/src/treecalc.rs",
                    "src/oxcalc-core/src/treecalc_fixture.rs",
                    "src/oxcalc-core/src/treecalc_runner.rs",
                    W041_LEAN_RUST_TOTALITY_FILE
                ],
                "failures": if lean_file_present { Vec::<String>::new() } else { vec!["w041_lean_rust_totality_file_missing".to_string()] },
                "validation_state": if lean_file_present { "w041_rust_totality_row_validated" } else { "w041_rust_totality_row_failed" }
            }),
            json!({
                "row_id": "w041_fixture_successor_catalog_totality_evidence",
                "w041_obligation_id": "W041-OBL-008",
                "source_inputs": ["W041 TreeCalc post-edit successor formula catalog", "W041 optimized/core evidence"],
                "disposition_kind": "direct_totality_evidence",
                "disposition": "post-edit successor formula-catalog fixture handling is exercised through deterministic typed fixture execution",
                "local_checked_proof": true,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": false,
                "refinement_row": false,
                "automatic_dynamic_transition_row": false,
                "exact_remaining_blocker": false,
                "authority_owner": "calc-sui.2; calc-sui.3",
                "promotion_consequence": "the fixture catalog path is evidenced, while whole-engine Rust totality remains blocked",
                "reason": "The W041 fixture supplies a successor formula catalog and the runner emits deterministic post-edit artifacts with zero expectation mismatches.",
                "evidence_paths": [
                    "src/oxcalc-core/src/treecalc_fixture.rs",
                    &w041_conformance_summary_path,
                    &w041_conformance_validation_path,
                    &w041_treecalc_summary_path
                ],
                "failures": if string_value(&w041_conformance_validation, "status") == "optimized_core_w041_dynamic_transition_packet_valid" && counter_value(&w041_treecalc_summary, "expectation_mismatch_count") == 0 { Vec::<String>::new() } else { vec!["w041_successor_catalog_fixture_evidence_missing".to_string()] },
                "validation_state": if string_value(&w041_conformance_validation, "status") == "optimized_core_w041_dynamic_transition_packet_valid" && counter_value(&w041_treecalc_summary, "expectation_mismatch_count") == 0 { "w041_rust_totality_row_validated" } else { "w041_rust_totality_row_failed" }
            }),
            json!({
                "row_id": "w041_explicit_dependency_seed_rebind_regression_evidence",
                "w041_obligation_id": "W041-OBL-009",
                "source_inputs": ["W040 Rust totality/refinement packet"],
                "disposition_kind": "direct_refinement_evidence",
                "disposition": "retain explicit DependencyRemoved and DependencyReclassified seed behavior as regression refinement evidence",
                "local_checked_proof": true,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": false,
                "refinement_row": true,
                "automatic_dynamic_transition_row": false,
                "exact_remaining_blocker": false,
                "authority_owner": "calc-sui.3",
                "promotion_consequence": "explicit-seed refinement evidence is retained but is no longer the only dynamic-transition evidence",
                "reason": "The predecessor W040 Rust packet remains valid and recorded zero failed rows.",
                "evidence_paths": [&w040_rust_summary_path, &w040_rust_validation_path],
                "failures": if string_value(&w040_rust_validation, "status") == "formal_assurance_w040_rust_totality_refinement_valid" && counter_value(&w040_rust_summary, "failed_row_count") == 0 { Vec::<String>::new() } else { vec!["w040_rust_regression_evidence_not_valid".to_string()] },
                "validation_state": if string_value(&w040_rust_validation, "status") == "formal_assurance_w040_rust_totality_refinement_valid" && counter_value(&w040_rust_summary, "failed_row_count") == 0 { "w041_rust_refinement_row_validated" } else { "w041_rust_refinement_row_failed" }
            }),
            json!({
                "row_id": "w041_automatic_dynamic_transition_refinement_evidence",
                "w041_obligation_id": "W041-OBL-009",
                "source_inputs": ["W041 dynamic release/reclassification TreeCalc run"],
                "disposition_kind": "direct_refinement_evidence",
                "disposition": "automatic resolved-to-potential dynamic transition derives DependencyRemoved and DependencyReclassified and forces rebind/no-publication behavior",
                "local_checked_proof": true,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": false,
                "refinement_row": true,
                "automatic_dynamic_transition_row": true,
                "exact_remaining_blocker": false,
                "authority_owner": "calc-sui.2; calc-sui.3",
                "promotion_consequence": "dynamic-transition refinement is evidenced for the exercised pattern, while full optimized/core verification remains blocked by other exact blockers",
                "reason": "The post-edit closure marks node 3 as requiring rebind, the automatic seeds include DependencyRemoved and DependencyReclassified, and the post-edit result rejects with HostInjectedFailure without a new publication.",
                "evidence_paths": [
                    &w041_dynamic_evidence_path,
                    &w041_treecalc_summary_path,
                    &w041_treecalc_auto_post_edit_result_path,
                    &w041_treecalc_auto_post_edit_closure_path,
                    &w041_treecalc_auto_post_edit_seeds_path
                ],
                "observed": {
                    "treecalc_case_count": counter_value(&w041_treecalc_summary, "case_count"),
                    "treecalc_expectation_mismatch_count": counter_value(&w041_treecalc_summary, "expectation_mismatch_count"),
                    "automatic_transition_seeds_present": automatic_transition_seeds_present,
                    "automatic_transition_closure_requires_rebind": automatic_transition_closure_requires_rebind,
                    "automatic_transition_rejected_for_rebind": automatic_transition_rejected_for_rebind
                },
                "failures": if counter_value(&w041_treecalc_summary, "expectation_mismatch_count") == 0 && automatic_transition_seeds_present && automatic_transition_closure_requires_rebind && automatic_transition_rejected_for_rebind { Vec::<String>::new() } else { vec!["w041_automatic_dynamic_transition_refinement_evidence_missing".to_string()] },
                "validation_state": if counter_value(&w041_treecalc_summary, "expectation_mismatch_count") == 0 && automatic_transition_seeds_present && automatic_transition_closure_requires_rebind && automatic_transition_rejected_for_rebind { "w041_rust_refinement_row_validated" } else { "w041_rust_refinement_row_failed" }
            }),
            json!({
                "row_id": "w041_runtime_panic_surface_totality_boundary",
                "w041_obligation_id": "W041-OBL-007",
                "source_inputs": ["Rust panic marker audit", W041_LEAN_RUST_TOTALITY_FILE],
                "disposition_kind": "exact_totality_boundary",
                "disposition": "retain a whole-engine panic-free proof blocker while panic/unwrap/expect markers remain in core Rust surfaces",
                "local_checked_proof": true,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": true,
                "refinement_row": false,
                "automatic_dynamic_transition_row": false,
                "exact_remaining_blocker": true,
                "authority_owner": "calc-sui.3; calc-sui.10",
                "promotion_consequence": "Rust-engine totality and panic-free core domain remain unpromoted",
                "reason": "The marker census is a guard, not a semantic proof; observed panic-family markers require review or proof before a panic-free claim.",
                "evidence_paths": W040_RUST_PANIC_AUDIT_FILES,
                "observed": {
                    "panic_marker_count": panic_marker_count,
                    "audited_file_count": W040_RUST_PANIC_AUDIT_FILES.len()
                },
                "failures": Vec::<String>::new(),
                "validation_state": "w041_rust_exact_blocker_validated"
            }),
            json!({
                "row_id": "w041_snapshot_fence_refinement_boundary",
                "w041_obligation_id": "W041-OBL-008",
                "source_inputs": ["W041 optimized/core exact blocker register"],
                "disposition_kind": "exact_refinement_blocker",
                "disposition": "retain the snapshot-fence counterpart blocker as a refinement boundary",
                "local_checked_proof": false,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": true,
                "refinement_row": true,
                "automatic_dynamic_transition_row": false,
                "exact_remaining_blocker": true,
                "authority_owner": "calc-sui.5",
                "promotion_consequence": "coordinator/Stage 2 refinement remains unpromoted",
                "reason": "The stale accepted-candidate snapshot-fence counterpart remains owned by Stage 2/coordinator evidence and is not discharged by Rust carrier proof.",
                "evidence_paths": [&w041_conformance_blockers_path],
                "failures": if row_with_field_exists(&w041_conformance_blockers, "row_id", "w041_snapshot_fence_counterpart_exact_blocker") { Vec::<String>::new() } else { vec!["w041_snapshot_fence_blocker_missing".to_string()] },
                "validation_state": if row_with_field_exists(&w041_conformance_blockers, "row_id", "w041_snapshot_fence_counterpart_exact_blocker") { "w041_rust_exact_blocker_validated" } else { "w041_rust_exact_blocker_failed" }
            }),
            json!({
                "row_id": "w041_capability_view_fence_refinement_boundary",
                "w041_obligation_id": "W041-OBL-008",
                "source_inputs": ["W041 optimized/core exact blocker register"],
                "disposition_kind": "exact_refinement_blocker",
                "disposition": "retain the capability-view counterpart blocker as a refinement boundary",
                "local_checked_proof": false,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": true,
                "refinement_row": true,
                "automatic_dynamic_transition_row": false,
                "exact_remaining_blocker": true,
                "authority_owner": "calc-sui.5",
                "promotion_consequence": "coordinator/Stage 2 refinement remains unpromoted",
                "reason": "The compatibility-fenced capability-view mismatch counterpart remains owned by Stage 2/coordinator evidence and is not discharged by Rust carrier proof.",
                "evidence_paths": [&w041_conformance_blockers_path],
                "failures": if row_with_field_exists(&w041_conformance_blockers, "row_id", "w041_capability_view_fence_counterpart_exact_blocker") { Vec::<String>::new() } else { vec!["w041_capability_view_fence_blocker_missing".to_string()] },
                "validation_state": if row_with_field_exists(&w041_conformance_blockers, "row_id", "w041_capability_view_fence_counterpart_exact_blocker") { "w041_rust_exact_blocker_validated" } else { "w041_rust_exact_blocker_failed" }
            }),
            json!({
                "row_id": "w041_callable_metadata_projection_totality_boundary",
                "w041_obligation_id": "W041-OBL-008",
                "source_inputs": ["W041 optimized/core exact blocker register", "LET/LAMBDA carrier boundary"],
                "disposition_kind": "exact_totality_boundary",
                "disposition": "carry callable metadata projection as an exact totality/refinement blocker",
                "local_checked_proof": true,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": true,
                "refinement_row": true,
                "automatic_dynamic_transition_row": false,
                "exact_remaining_blocker": true,
                "authority_owner": "calc-sui.8; external:OxFunc",
                "promotion_consequence": "callable metadata projection and broad callable conformance remain unpromoted",
                "reason": "The narrow LET/LAMBDA carrier seam is in scope, but general OxFunc kernels and metadata projection sufficiency are not discharged.",
                "evidence_paths": [&w041_conformance_blockers_path, W041_LEAN_RUST_TOTALITY_FILE],
                "failures": if row_with_field_exists(&w041_conformance_blockers, "row_id", "w041_callable_metadata_projection_exact_blocker") && lean_file_present { Vec::<String>::new() } else { vec!["w041_callable_metadata_or_lean_blocker_missing".to_string()] },
                "validation_state": if row_with_field_exists(&w041_conformance_blockers, "row_id", "w041_callable_metadata_projection_exact_blocker") && lean_file_present { "w041_rust_exact_blocker_validated" } else { "w041_rust_exact_blocker_failed" }
            }),
            json!({
                "row_id": "w041_let_lambda_carrier_external_boundary",
                "w041_obligation_id": "W041-OBL-028",
                "source_inputs": ["W041 successor obligation map", W041_LEAN_RUST_TOTALITY_FILE],
                "disposition_kind": "accepted_external_seam_boundary",
                "disposition": "keep LET/LAMBDA carrier interaction inside OxCalc/OxFml formalization while excluding general OxFunc kernels",
                "local_checked_proof": false,
                "bounded_model": false,
                "accepted_external_seam": true,
                "accepted_boundary": true,
                "totality_boundary": false,
                "refinement_row": false,
                "automatic_dynamic_transition_row": false,
                "exact_remaining_blocker": false,
                "authority_owner": "calc-sui.3; calc-sui.8; external:OxFunc",
                "promotion_consequence": "general OxFunc kernels remain unpromoted inside OxCalc",
                "reason": "W041 scope includes the carrier seam but not broad OxFunc semantic kernels.",
                "evidence_paths": [&w041_obligation_map_path, W041_LEAN_RUST_TOTALITY_FILE],
                "failures": if w040_obligation_exists(&w041_obligation_map, "W041-OBL-028") && lean_file_present { Vec::<String>::new() } else { vec!["w041_let_lambda_boundary_missing".to_string()] },
                "validation_state": if w040_obligation_exists(&w041_obligation_map, "W041-OBL-028") && lean_file_present { "w041_rust_boundary_validated" } else { "w041_rust_boundary_failed" }
            }),
            json!({
                "row_id": "w041_spec_evolution_refinement_guard",
                "w041_obligation_id": "W041-OBL-009",
                "source_inputs": ["W041 workset and obligation map"],
                "disposition_kind": "accepted_spec_evolution_guard",
                "disposition": "preserve formalization as spec evolution and implementation improvement, not a fixed-spec test only",
                "local_checked_proof": true,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": true,
                "totality_boundary": false,
                "refinement_row": false,
                "automatic_dynamic_transition_row": false,
                "exact_remaining_blocker": false,
                "authority_owner": "calc-sui.3; calc-sui.10",
                "promotion_consequence": "future proof evidence may correct specs or implementation before promotion",
                "reason": "The W041 charter records spec-evolution hooks for Rust totality and refinement obligations.",
                "evidence_paths": [&w041_obligation_map_path, "docs/worksets/W041_CORE_FORMALIZATION_RELEASE_GRADE_SUCCESSOR_VERIFICATION.md"],
                "failures": if w040_obligation_exists(&w041_obligation_map, "W041-OBL-009") { Vec::<String>::new() } else { vec!["w041_refinement_obligation_missing".to_string()] },
                "validation_state": if w040_obligation_exists(&w041_obligation_map, "W041-OBL-009") { "w041_rust_boundary_validated" } else { "w041_rust_boundary_failed" }
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
        let automatic_dynamic_transition_row_count = proof_rows
            .iter()
            .filter(|row| bool_at(row, "automatic_dynamic_transition_row"))
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
        if counter_value(&w041_obligation_summary, "obligation_count") != 28 {
            validation_failures.push("w041_obligation_count_changed".to_string());
        }
        if !array_contains_string(
            w041_obligation_summary
                .get("no_promotion_claims")
                .unwrap_or(&Value::Null),
            "rust_totality_and_refinement",
        ) {
            validation_failures.push("w041_rust_no_promotion_guard_missing".to_string());
        }
        if !w040_obligation_exists(&w041_obligation_map, "W041-OBL-007")
            || !w040_obligation_exists(&w041_obligation_map, "W041-OBL-008")
            || !w040_obligation_exists(&w041_obligation_map, "W041-OBL-009")
        {
            validation_failures.push("w041_rust_obligation_rows_missing".to_string());
        }
        if string_value(&w040_rust_validation, "status")
            != "formal_assurance_w040_rust_totality_refinement_valid"
        {
            validation_failures.push("w040_rust_formal_assurance_not_valid".to_string());
        }
        if counter_value(&w040_rust_blockers, "exact_remaining_blocker_count") != 5 {
            validation_failures.push("w040_rust_blocker_register_count_changed".to_string());
        }
        if bool_at(
            w040_rust_summary
                .get("promotion_claims")
                .unwrap_or(&Value::Null),
            "rust_engine_totality_promoted",
        ) {
            validation_failures.push("w040_rust_totality_was_promoted".to_string());
        }
        if string_value(&w041_conformance_validation, "status")
            != "optimized_core_w041_dynamic_transition_packet_valid"
        {
            validation_failures.push("w041_conformance_validation_not_valid".to_string());
        }
        if counter_value(&w041_conformance_summary, "exact_remaining_blocker_count") != 3 {
            validation_failures.push("w041_conformance_exact_blocker_count_changed".to_string());
        }
        if counter_value(
            &w041_conformance_summary,
            "dynamic_transition_implementation_evidence_count",
        ) != 1
        {
            validation_failures.push("w041_dynamic_transition_evidence_count_missing".to_string());
        }
        if counter_value(&w041_conformance_blockers, "exact_remaining_blocker_count") != 3 {
            validation_failures.push("w041_conformance_blocker_register_count_changed".to_string());
        }
        if counter_value(&w041_treecalc_summary, "expectation_mismatch_count") != 0 {
            validation_failures.push("w041_treecalc_expectation_mismatch_present".to_string());
        }
        if string_value(&w041_dynamic_evidence, "treecalc_run_id") != W041_TREECALC_RUN_ID {
            validation_failures.push("w041_dynamic_evidence_run_id_changed".to_string());
        }
        if !lean_file_present || !w040_lean_file_present {
            validation_failures.push("w041_or_w040_lean_rust_totality_file_missing".to_string());
        }
        if panic_marker_count == 0 {
            validation_failures.push("w041_panic_marker_audit_unexpected_zero".to_string());
        }
        if failed_row_count != 0 {
            validation_failures.push("w041_rust_totality_row_failures_present".to_string());
        }
        if blocker_rows.len() != 4 {
            validation_failures.push("w041_expected_four_rust_exact_blockers".to_string());
        }
        if automatic_dynamic_transition_row_count != 1 {
            validation_failures
                .push("w041_expected_one_automatic_dynamic_transition_row".to_string());
        }

        let source_evidence_index_path =
            format!("{relative_artifact_root}/source_evidence_index.json");
        let rust_ledger_path =
            format!("{relative_artifact_root}/w041_rust_totality_refinement_ledger.json");
        let totality_register_path =
            format!("{relative_artifact_root}/w041_rust_totality_boundary_register.json");
        let refinement_register_path =
            format!("{relative_artifact_root}/w041_rust_refinement_register.json");
        let blocker_register_path =
            format!("{relative_artifact_root}/w041_rust_exact_blocker_register.json");
        let validation_path = format!("{relative_artifact_root}/validation.json");

        write_json(
            &artifact_root.join("source_evidence_index.json"),
            &json!({
                "schema_version": FORMAL_ASSURANCE_SOURCE_INDEX_SCHEMA_V1,
                "run_id": run_id,
                "source_artifacts": {
                    "w041_successor_obligation_summary": w041_obligation_summary_path,
                    "w041_successor_obligation_map": w041_obligation_map_path,
                    "w040_rust_formal_assurance_summary": w040_rust_summary_path,
                    "w040_rust_formal_assurance_validation": w040_rust_validation_path,
                    "w040_rust_exact_blockers": w040_rust_blockers_path,
                    "w041_implementation_conformance_summary": w041_conformance_summary_path,
                    "w041_implementation_conformance_validation": w041_conformance_validation_path,
                    "w041_implementation_conformance_exact_blockers": w041_conformance_blockers_path,
                    "w041_dynamic_release_reclassification_auto_evidence": w041_dynamic_evidence_path,
                    "w041_treecalc_summary": w041_treecalc_summary_path,
                    "w041_treecalc_auto_post_edit_result": w041_treecalc_auto_post_edit_result_path,
                    "w041_treecalc_auto_post_edit_closure": w041_treecalc_auto_post_edit_closure_path,
                    "w041_treecalc_auto_post_edit_seeds": w041_treecalc_auto_post_edit_seeds_path,
                    "w041_lean_rust_totality_file": W041_LEAN_RUST_TOTALITY_FILE
                },
                "source_counts": {
                    "w041_obligation_count": counter_value(&w041_obligation_summary, "obligation_count"),
                    "w040_rust_exact_blocker_count": counter_value(&w040_rust_summary, "exact_remaining_blocker_count"),
                    "w041_conformance_exact_blocker_count": counter_value(&w041_conformance_summary, "exact_remaining_blocker_count"),
                    "w041_treecalc_case_count": counter_value(&w041_treecalc_summary, "case_count"),
                    "panic_marker_count": panic_marker_count
                }
            }),
        )?;
        write_json(
            &artifact_root.join("w041_rust_totality_refinement_ledger.json"),
            &json!({
                "schema_version": FORMAL_ASSURANCE_W041_RUST_LEDGER_SCHEMA_V1,
                "run_id": run_id,
                "rust_row_count": proof_rows.len(),
                "local_proof_row_count": local_proof_row_count,
                "bounded_model_row_count": bounded_model_row_count,
                "accepted_external_seam_count": accepted_external_seam_count,
                "accepted_boundary_count": accepted_boundary_count,
                "automatic_dynamic_transition_row_count": automatic_dynamic_transition_row_count,
                "totality_boundary_count": totality_rows.len(),
                "refinement_row_count": refinement_rows.len(),
                "exact_remaining_blocker_count": blocker_rows.len(),
                "rows": proof_rows
            }),
        )?;
        write_json(
            &artifact_root.join("w041_rust_totality_boundary_register.json"),
            &json!({
                "schema_version": FORMAL_ASSURANCE_W041_TOTALITY_REGISTER_SCHEMA_V1,
                "run_id": run_id,
                "totality_boundary_count": totality_rows.len(),
                "rows": totality_rows
            }),
        )?;
        write_json(
            &artifact_root.join("w041_rust_refinement_register.json"),
            &json!({
                "schema_version": FORMAL_ASSURANCE_W041_REFINEMENT_REGISTER_SCHEMA_V1,
                "run_id": run_id,
                "refinement_row_count": refinement_rows.len(),
                "automatic_dynamic_transition_row_count": automatic_dynamic_transition_row_count,
                "rows": refinement_rows
            }),
        )?;
        write_json(
            &artifact_root.join("w041_rust_exact_blocker_register.json"),
            &json!({
                "schema_version": FORMAL_ASSURANCE_W041_BLOCKER_REGISTER_SCHEMA_V1,
                "run_id": run_id,
                "exact_remaining_blocker_count": blocker_rows.len(),
                "rows": blocker_rows
            }),
        )?;

        let validation_status = if validation_failures.is_empty() {
            "formal_assurance_w041_rust_totality_refinement_valid"
        } else {
            "formal_assurance_w041_rust_totality_refinement_invalid"
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
                "automatic_dynamic_transition_row_count": automatic_dynamic_transition_row_count,
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
                "automatic_dynamic_transition_row_count": automatic_dynamic_transition_row_count,
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
                    "callable_carrier_sufficiency_promoted": false,
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

    fn execute_w043_lean_tla_discharge(
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

        let w043_obligation_summary_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "release-grade-ledger",
            W043_RESIDUAL_LEDGER_RUN_ID,
            "run_summary.json",
        ]);
        let w043_obligation_map_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "release-grade-ledger",
            W043_RESIDUAL_LEDGER_RUN_ID,
            "proof_service_obligation_map.json",
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
        let w042_lean_tla_summary_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "formal-assurance",
            W042_LEAN_TLA_FORMAL_ASSURANCE_RUN_ID,
            "run_summary.json",
        ]);
        let w042_lean_tla_validation_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "formal-assurance",
            W042_LEAN_TLA_FORMAL_ASSURANCE_RUN_ID,
            "validation.json",
        ]);
        let w042_lean_tla_blockers_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "formal-assurance",
            W042_LEAN_TLA_FORMAL_ASSURANCE_RUN_ID,
            "w042_lean_tla_exact_blocker_register.json",
        ]);
        let w043_rust_summary_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "formal-assurance",
            W043_RUST_FORMAL_ASSURANCE_RUN_ID,
            "run_summary.json",
        ]);
        let w043_rust_validation_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "formal-assurance",
            W043_RUST_FORMAL_ASSURANCE_RUN_ID,
            "validation.json",
        ]);
        let w043_rust_blockers_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "formal-assurance",
            W043_RUST_FORMAL_ASSURANCE_RUN_ID,
            "w043_rust_exact_blocker_register.json",
        ]);
        let w043_rust_refinement_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "formal-assurance",
            W043_RUST_FORMAL_ASSURANCE_RUN_ID,
            "w043_rust_refinement_register.json",
        ]);
        let w043_rust_ledger_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "formal-assurance",
            W043_RUST_FORMAL_ASSURANCE_RUN_ID,
            "w043_rust_totality_refinement_ledger.json",
        ]);
        let w042_stage2_summary_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "stage2-replay",
            W042_STAGE2_REPLAY_RUN_ID,
            "run_summary.json",
        ]);
        let w042_stage2_validation_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "stage2-replay",
            W042_STAGE2_REPLAY_RUN_ID,
            "validation.json",
        ]);
        let w042_stage2_gate_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "stage2-replay",
            W042_STAGE2_REPLAY_RUN_ID,
            "w042_stage2_policy_gate_register.json",
        ]);
        let w042_stage2_blockers_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "stage2-replay",
            W042_STAGE2_REPLAY_RUN_ID,
            "w042_stage2_exact_blocker_register.json",
        ]);
        let w043_w073_formatting_intake_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "implementation-conformance",
            W043_IMPLEMENTATION_CONFORMANCE_RUN_ID,
            "w073_formatting_intake.json",
        ]);

        let w043_obligation_summary = read_json(repo_root, &w043_obligation_summary_path)?;
        let w043_obligation_map = read_json(repo_root, &w043_obligation_map_path)?;
        let w037_formal_summary = read_json(repo_root, &w037_formal_summary_path)?;
        let w037_formal_validation = read_json(repo_root, &w037_formal_validation_path)?;
        let w037_tla_inventory = read_json(repo_root, &w037_tla_inventory_path)?;
        let w042_lean_tla_summary = read_json(repo_root, &w042_lean_tla_summary_path)?;
        let w042_lean_tla_validation = read_json(repo_root, &w042_lean_tla_validation_path)?;
        let w042_lean_tla_blockers = read_json(repo_root, &w042_lean_tla_blockers_path)?;
        let w043_rust_summary = read_json(repo_root, &w043_rust_summary_path)?;
        let w043_rust_validation = read_json(repo_root, &w043_rust_validation_path)?;
        let w043_rust_blockers = read_json(repo_root, &w043_rust_blockers_path)?;
        let w043_rust_refinement = read_json(repo_root, &w043_rust_refinement_path)?;
        let w043_rust_ledger = read_json(repo_root, &w043_rust_ledger_path)?;
        let w042_stage2_summary = read_json(repo_root, &w042_stage2_summary_path)?;
        let w042_stage2_validation = read_json(repo_root, &w042_stage2_validation_path)?;
        let w042_stage2_gate = read_json(repo_root, &w042_stage2_gate_path)?;
        let w042_stage2_blockers = read_json(repo_root, &w042_stage2_blockers_path)?;
        let w043_w073_formatting_intake = read_json(repo_root, &w043_w073_formatting_intake_path)?;

        let lean_discharge_file_present = repo_root.join(W043_LEAN_TLA_DISCHARGE_FILE).exists();
        let w042_lean_discharge_file_present =
            repo_root.join(W042_LEAN_TLA_DISCHARGE_FILE).exists();
        let w043_rust_file_present = repo_root.join(W043_LEAN_RUST_TOTALITY_FILE).exists();
        let w042_stage2_policy_file_present = repo_root.join(W042_STAGE2_POLICY_FILE).exists();
        let lean_placeholder_count = lean_placeholder_count(repo_root)?;
        let routine_tla_config_count =
            counter_value(&w037_formal_summary, "tla_routine_config_count");
        let routine_tla_failed_count =
            counter_value(&w037_formal_summary, "tla_failed_config_count");
        let tla_inventory_passed_count = counter_value(&w037_tla_inventory, "passed_config_count");
        let w043_dynamic_addition_refinement_present = row_with_field_exists(
            &w043_rust_refinement,
            "row_id",
            "w043_automatic_dynamic_addition_refinement_evidence",
        );
        let w043_dynamic_release_refinement_present = row_with_field_exists(
            &w043_rust_refinement,
            "row_id",
            "w043_automatic_dynamic_release_refinement_evidence",
        );
        let w043_callable_value_carrier_present = row_with_field_exists(
            &w043_rust_ledger,
            "row_id",
            "w043_callable_value_carrier_totality_evidence",
        );
        let w042_fairness_stage2_blocker_present = row_with_field_exists(
            &w042_stage2_blockers,
            "row_id",
            "w042_stage2_scheduler_fairness_unbounded_equivalence_blocker",
        );
        let w073_typed_only_guard_present = !bool_at(
            &w043_w073_formatting_intake,
            "threshold_fallback_allowed_for_typed_families",
        ) && w043_w073_formatting_intake
            .get("typed_rule_only_families")
            .and_then(Value::as_array)
            .is_some_and(|families| families.len() == 7);
        let w037_formal_inventory_valid = string_value(&w037_formal_validation, "validation_state")
            == "w037_proof_model_closure_inventory_validated";

        let proof_rows = vec![
            json!({
                "row_id": "w043_lean_inventory_checked_no_placeholder_evidence",
                "w043_obligation_id": "W043-OBL-012",
                "source_inputs": ["W037 formal inventory", W043_LEAN_TLA_DISCHARGE_FILE],
                "disposition_kind": "checked_lean_inventory_evidence",
                "disposition": "bind the current Lean inventory and zero-placeholder audit as checked W043 evidence without promoting full Lean verification",
                "local_checked_proof": true,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": false,
                "exact_remaining_blocker": false,
                "authority_owner": "calc-2p3.4",
                "promotion_consequence": "full Lean verification remains unpromoted until all semantic proof boundaries are discharged",
                "reason": "The Lean inventory is typechecked and the placeholder census is zero, but this remains classification evidence rather than whole-engine semantic proof.",
                "evidence_paths": [&w037_formal_summary_path, &w037_formal_validation_path, W043_LEAN_TLA_DISCHARGE_FILE],
                "observed": {
                    "lean_file_count": counter_value(&w037_formal_summary, "lean_file_count"),
                    "lean_placeholder_count": lean_placeholder_count
                },
                "failures": if bool_at(&w037_formal_summary, "all_checked_artifacts_passed") && w037_formal_inventory_valid && lean_placeholder_count == 0 && lean_discharge_file_present { Vec::<String>::new() } else { vec!["w043_lean_inventory_or_placeholder_check_failed".to_string()] },
                "validation_state": if bool_at(&w037_formal_summary, "all_checked_artifacts_passed") && w037_formal_inventory_valid && lean_placeholder_count == 0 && lean_discharge_file_present { "w043_lean_proof_row_validated" } else { "w043_lean_proof_row_failed" }
            }),
            json!({
                "row_id": "w043_lean_tla_predecessor_bridge",
                "w043_obligation_id": "W043-OBL-012",
                "source_inputs": ["W042 Lean/TLA proof-model packet", W042_LEAN_TLA_DISCHARGE_FILE],
                "disposition_kind": "checked_lean_bridge_evidence",
                "disposition": "bind the W042 Lean/TLA packet as a checked non-promoting predecessor input",
                "local_checked_proof": true,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": false,
                "exact_remaining_blocker": false,
                "authority_owner": "calc-2p3.4",
                "promotion_consequence": "full Lean/TLA verification remains unpromoted",
                "reason": "The W042 Lean/TLA packet remains valid and records five exact blockers; W043.4 builds on it without treating it as full verification.",
                "evidence_paths": [&w042_lean_tla_summary_path, &w042_lean_tla_validation_path, W042_LEAN_TLA_DISCHARGE_FILE],
                "failures": if string_value(&w042_lean_tla_validation, "status") == "formal_assurance_w042_lean_tla_fairness_expansion_valid" && w042_lean_discharge_file_present { Vec::<String>::new() } else { vec!["w042_lean_tla_predecessor_not_valid".to_string()] },
                "validation_state": if string_value(&w042_lean_tla_validation, "status") == "formal_assurance_w042_lean_tla_fairness_expansion_valid" && w042_lean_discharge_file_present { "w043_lean_proof_row_validated" } else { "w043_lean_proof_row_failed" }
            }),
            json!({
                "row_id": "w043_lean_rust_dynamic_addition_refinement_bridge",
                "w043_obligation_id": "W043-OBL-014",
                "source_inputs": ["W043 Rust totality/refinement packet", W043_LEAN_RUST_TOTALITY_FILE],
                "disposition_kind": "checked_lean_refinement_bridge",
                "disposition": "bind the W043 automatic dependency-addition refinement row as a checked Lean/TLA proof input",
                "local_checked_proof": true,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": false,
                "exact_remaining_blocker": false,
                "authority_owner": "calc-2p3.3; calc-2p3.4",
                "promotion_consequence": "Rust refinement and full optimized/core verification remain unpromoted because retained blockers remain",
                "reason": "W043.3 records automatic DependencyAdded plus DependencyReclassified refinement evidence while keeping broader dynamic coverage and Rust totality blockers.",
                "evidence_paths": [&w043_rust_summary_path, &w043_rust_validation_path, &w043_rust_refinement_path, W043_LEAN_RUST_TOTALITY_FILE],
                "observed": {
                    "automatic_dynamic_transition_row_count": counter_value(&w043_rust_summary, "automatic_dynamic_transition_row_count"),
                    "w043_rust_exact_blocker_count": counter_value(&w043_rust_summary, "exact_remaining_blocker_count")
                },
                "failures": if string_value(&w043_rust_validation, "status") == "formal_assurance_w043_rust_totality_refinement_valid" && counter_value(&w043_rust_summary, "automatic_dynamic_transition_row_count") == 2 && w043_dynamic_addition_refinement_present && w043_rust_file_present { Vec::<String>::new() } else { vec!["w043_rust_dynamic_addition_refinement_bridge_missing".to_string()] },
                "validation_state": if string_value(&w043_rust_validation, "status") == "formal_assurance_w043_rust_totality_refinement_valid" && counter_value(&w043_rust_summary, "automatic_dynamic_transition_row_count") == 2 && w043_dynamic_addition_refinement_present && w043_rust_file_present { "w043_lean_proof_row_validated" } else { "w043_lean_proof_row_failed" }
            }),
            json!({
                "row_id": "w043_lean_rust_dynamic_release_refinement_bridge",
                "w043_obligation_id": "W043-OBL-014",
                "source_inputs": ["W043 Rust totality/refinement packet", W043_LEAN_RUST_TOTALITY_FILE],
                "disposition_kind": "checked_lean_refinement_bridge",
                "disposition": "bind the W043 automatic dependency-release refinement row as a checked Lean/TLA proof input",
                "local_checked_proof": true,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": false,
                "exact_remaining_blocker": false,
                "authority_owner": "calc-2p3.3; calc-2p3.4",
                "promotion_consequence": "Rust refinement and full optimized/core verification remain unpromoted because retained blockers remain",
                "reason": "W043.3 records automatic DependencyRemoved plus DependencyReclassified refinement evidence while keeping broader dynamic coverage and Rust totality blockers.",
                "evidence_paths": [&w043_rust_summary_path, &w043_rust_validation_path, &w043_rust_refinement_path, W043_LEAN_RUST_TOTALITY_FILE],
                "observed": {
                    "automatic_dynamic_transition_row_count": counter_value(&w043_rust_summary, "automatic_dynamic_transition_row_count"),
                    "w043_rust_exact_blocker_count": counter_value(&w043_rust_summary, "exact_remaining_blocker_count")
                },
                "failures": if string_value(&w043_rust_validation, "status") == "formal_assurance_w043_rust_totality_refinement_valid" && counter_value(&w043_rust_summary, "automatic_dynamic_transition_row_count") == 2 && w043_dynamic_release_refinement_present && w043_rust_file_present { Vec::<String>::new() } else { vec!["w043_rust_dynamic_release_refinement_bridge_missing".to_string()] },
                "validation_state": if string_value(&w043_rust_validation, "status") == "formal_assurance_w043_rust_totality_refinement_valid" && counter_value(&w043_rust_summary, "automatic_dynamic_transition_row_count") == 2 && w043_dynamic_release_refinement_present && w043_rust_file_present { "w043_lean_proof_row_validated" } else { "w043_lean_proof_row_failed" }
            }),
            json!({
                "row_id": "w043_lean_callable_carrier_boundary_bridge",
                "w043_obligation_id": "W043-OBL-015",
                "source_inputs": ["W043 Rust callable value-carrier row", "W043 proof-service obligation map"],
                "disposition_kind": "checked_lean_callable_carrier_bridge",
                "disposition": "bind the ordinary LET/LAMBDA value-carrier row as checked input while keeping callable carrier sufficiency and general OxFunc kernels unpromoted",
                "local_checked_proof": true,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": false,
                "exact_remaining_blocker": false,
                "authority_owner": "calc-2p3.3; calc-2p3.4; calc-2p3.8",
                "promotion_consequence": "callable carrier sufficiency and general OxFunc kernel claims remain unpromoted",
                "reason": "W043.3 proves ordinary value-carrier publication for the current LET/LAMBDA fixture, not metadata projection or broad OxFunc semantics.",
                "evidence_paths": [&w043_rust_ledger_path, &w043_obligation_map_path],
                "observed": {
                    "callable_value_carrier_row_present": w043_callable_value_carrier_present
                },
                "failures": if w043_callable_value_carrier_present && w040_obligation_exists(&w043_obligation_map, "W043-OBL-015") && w040_obligation_exists(&w043_obligation_map, "W043-OBL-030") { Vec::<String>::new() } else { vec!["w043_callable_carrier_bridge_missing".to_string()] },
                "validation_state": if w043_callable_value_carrier_present && w040_obligation_exists(&w043_obligation_map, "W043-OBL-015") && w040_obligation_exists(&w043_obligation_map, "W043-OBL-030") { "w043_lean_proof_row_validated" } else { "w043_lean_proof_row_failed" }
            }),
            json!({
                "row_id": "w043_lean_stage2_analyzer_pack_predicate_carried",
                "w043_obligation_id": "W043-OBL-014",
                "source_inputs": ["W042 Stage 2 Lean predicate and replay packet"],
                "disposition_kind": "checked_lean_policy_predicate",
                "disposition": "carry the checked W042 Stage 2 analyzer and pack-equivalence predicate as proof input while retaining production-policy blockers",
                "local_checked_proof": true,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": false,
                "exact_remaining_blocker": false,
                "authority_owner": "calc-2p3.4; calc-2p3.5",
                "promotion_consequence": "Stage 2 production policy remains unpromoted",
                "reason": "The W042 predicate and replay packet prove no-promotion under current bounded evidence; they are not production analyzer soundness or fairness discharge.",
                "evidence_paths": [W042_STAGE2_POLICY_FILE, &w042_stage2_summary_path, &w042_stage2_validation_path],
                "failures": if w042_stage2_policy_file_present && string_value(&w042_stage2_validation, "status") == "w042_stage2_pack_grade_equivalence_valid" && !bool_at(&w042_stage2_summary, "stage2_policy_promoted") { Vec::<String>::new() } else { vec!["w042_stage2_policy_input_missing_or_promoted".to_string()] },
                "validation_state": if w042_stage2_policy_file_present && string_value(&w042_stage2_validation, "status") == "w042_stage2_pack_grade_equivalence_valid" && !bool_at(&w042_stage2_summary, "stage2_policy_promoted") { "w043_lean_proof_row_validated" } else { "w043_lean_proof_row_failed" }
            }),
            json!({
                "row_id": "w043_tla_routine_config_bounded_model_boundary",
                "w043_obligation_id": "W043-OBL-013",
                "source_inputs": ["W037 TLA inventory", "routine TLC config set"],
                "disposition_kind": "bounded_model_with_exact_totality_boundary",
                "disposition": "bind the routine TLC config set as bounded model evidence while retaining unbounded model coverage as exact blocker",
                "local_checked_proof": false,
                "bounded_model": true,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": true,
                "exact_remaining_blocker": true,
                "authority_owner": "calc-2p3.4; calc-2p3.10",
                "promotion_consequence": "full TLA verification remains unpromoted",
                "reason": "The routine TLC floor has 11 bounded configs with zero recorded failures, but does not cover the unbounded scheduler and partition universe.",
                "evidence_paths": [&w037_tla_inventory_path, &w037_formal_summary_path],
                "observed": {
                    "routine_tla_config_count": routine_tla_config_count,
                    "tla_inventory_passed_count": tla_inventory_passed_count,
                    "routine_tla_failed_count": routine_tla_failed_count
                },
                "failures": if routine_tla_config_count == 11 && tla_inventory_passed_count == 11 && routine_tla_failed_count == 0 { Vec::<String>::new() } else { vec!["w043_tla_routine_config_floor_changed".to_string()] },
                "validation_state": if routine_tla_config_count == 11 && tla_inventory_passed_count == 11 && routine_tla_failed_count == 0 { "w043_tla_model_row_validated" } else { "w043_tla_model_row_failed" }
            }),
            json!({
                "row_id": "w043_tla_stage2_partition_bounded_model_evidence",
                "w043_obligation_id": "W043-OBL-013",
                "source_inputs": ["CoreEngineW036Stage2Partition bounded configs"],
                "disposition_kind": "bounded_stage2_partition_model_evidence",
                "disposition": "bind W036 Stage 2 partition configs as bounded coverage for scheduler readiness, partition cross-dependency, fence reject, and multi-reader profiles",
                "local_checked_proof": false,
                "bounded_model": true,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": false,
                "exact_remaining_blocker": false,
                "authority_owner": "calc-2p3.4; calc-2p3.5",
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
                "failures": if routine_tla_config_count >= 11 && routine_tla_failed_count == 0 { Vec::<String>::new() } else { vec!["w043_stage2_partition_tla_floor_changed".to_string()] },
                "validation_state": if routine_tla_config_count >= 11 && routine_tla_failed_count == 0 { "w043_tla_model_row_validated" } else { "w043_tla_model_row_failed" }
            }),
            json!({
                "row_id": "w043_stage2_equivalence_bounded_model_input",
                "w043_obligation_id": "W043-OBL-014",
                "source_inputs": ["W042 Stage 2 analyzer and pack-equivalence packet"],
                "disposition_kind": "bounded_stage2_equivalence_model_evidence",
                "disposition": "bind W042 bounded partition replay, permutation, observable-invariance, analyzer, and pack-equivalence evidence as model input without promoting Stage 2",
                "local_checked_proof": false,
                "bounded_model": true,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": false,
                "exact_remaining_blocker": false,
                "authority_owner": "calc-2p3.4; calc-2p3.5",
                "promotion_consequence": "Stage 2 production policy and full TLA verification remain unpromoted",
                "reason": "W042 has bounded declared-profile evidence, counterpart rows, and pack-equivalence inputs, but production analyzer soundness, fairness, operated differential service, and pack-grade governance remain absent.",
                "evidence_paths": [&w042_stage2_summary_path, &w042_stage2_validation_path, &w042_stage2_gate_path, W042_STAGE2_POLICY_FILE],
                "observed": {
                    "partition_replay_row_count": counter_value(&w042_stage2_summary, "partition_replay_row_count"),
                    "permutation_replay_row_count": counter_value(&w042_stage2_summary, "permutation_replay_row_count"),
                    "observable_invariance_row_count": counter_value(&w042_stage2_summary, "observable_invariance_row_count"),
                    "satisfied_policy_row_count": counter_value(&w042_stage2_summary, "satisfied_policy_row_count"),
                    "exact_remaining_blocker_count": counter_value(&w042_stage2_summary, "exact_remaining_blocker_count")
                },
                "failures": if string_value(&w042_stage2_validation, "status") == "w042_stage2_pack_grade_equivalence_valid" && counter_value(&w042_stage2_summary, "partition_replay_row_count") == 5 && counter_value(&w042_stage2_summary, "observable_invariance_row_count") == 5 && !bool_at(&w042_stage2_summary, "stage2_policy_promoted") { Vec::<String>::new() } else { vec!["w042_stage2_equivalence_input_not_valid".to_string()] },
                "validation_state": if string_value(&w042_stage2_validation, "status") == "w042_stage2_pack_grade_equivalence_valid" && counter_value(&w042_stage2_summary, "partition_replay_row_count") == 5 && counter_value(&w042_stage2_summary, "observable_invariance_row_count") == 5 && !bool_at(&w042_stage2_summary, "stage2_policy_promoted") { "w043_tla_model_row_validated" } else { "w043_tla_model_row_failed" }
            }),
            json!({
                "row_id": "w043_tla_fairness_scheduler_unbounded_boundary",
                "w043_obligation_id": "W043-OBL-013",
                "source_inputs": ["W043 proof-service obligation map", "W042 Stage 2 blockers"],
                "disposition_kind": "exact_model_assumption_boundary",
                "disposition": "retain scheduler fairness, unbounded interleaving, and model-completeness coverage as exact model blockers",
                "local_checked_proof": false,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": true,
                "exact_remaining_blocker": true,
                "authority_owner": "calc-2p3.4; calc-2p3.5; calc-2p3.10",
                "promotion_consequence": "full TLA verification and Stage 2 production policy remain unpromoted",
                "reason": "Bounded TLC and Stage 2 replay evidence do not prove scheduler fairness or unbounded model coverage.",
                "evidence_paths": [&w043_obligation_map_path, &w042_stage2_blockers_path],
                "failures": if w040_obligation_exists(&w043_obligation_map, "W043-OBL-013") && w040_obligation_exists(&w043_obligation_map, "W043-OBL-014") && w042_fairness_stage2_blocker_present { Vec::<String>::new() } else { vec!["w043_fairness_unbounded_boundary_missing".to_string()] },
                "validation_state": if w040_obligation_exists(&w043_obligation_map, "W043-OBL-013") && w040_obligation_exists(&w043_obligation_map, "W043-OBL-014") && w042_fairness_stage2_blocker_present { "w043_lean_tla_exact_blocker_validated" } else { "w043_lean_tla_exact_blocker_failed" }
            }),
            json!({
                "row_id": "w043_full_lean_verification_exact_blocker",
                "w043_obligation_id": "W043-OBL-012",
                "source_inputs": ["W043 proof-service obligation map", W043_LEAN_TLA_DISCHARGE_FILE],
                "disposition_kind": "exact_lean_verification_blocker",
                "disposition": "retain full Lean verification as exact blocker beyond checked row classification and placeholder-free inventory",
                "local_checked_proof": true,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": true,
                "exact_remaining_blocker": true,
                "authority_owner": "calc-2p3.4; calc-2p3.10",
                "promotion_consequence": "full Lean verification remains unpromoted",
                "reason": "Checked classification files do not prove all Rust, scheduler, Stage 2, callable, service, and OxFml surfaces.",
                "evidence_paths": [&w043_obligation_map_path, W043_LEAN_TLA_DISCHARGE_FILE],
                "failures": if w040_obligation_exists(&w043_obligation_map, "W043-OBL-012") && lean_discharge_file_present { Vec::<String>::new() } else { vec!["w043_full_lean_blocker_missing".to_string()] },
                "validation_state": if w040_obligation_exists(&w043_obligation_map, "W043-OBL-012") && lean_discharge_file_present { "w043_lean_tla_exact_blocker_validated" } else { "w043_lean_tla_exact_blocker_failed" }
            }),
            json!({
                "row_id": "w043_full_tla_verification_exact_blocker",
                "w043_obligation_id": "W043-OBL-013",
                "source_inputs": ["W043 proof-service obligation map", "W037 TLA inventory"],
                "disposition_kind": "exact_tla_verification_blocker",
                "disposition": "retain full TLA verification as exact blocker beyond bounded config coverage",
                "local_checked_proof": false,
                "bounded_model": true,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": true,
                "exact_remaining_blocker": true,
                "authority_owner": "calc-2p3.4; calc-2p3.10",
                "promotion_consequence": "full TLA verification remains unpromoted",
                "reason": "The W037 inventory and W043 proof packet bind bounded TLC rows but not unbounded model coverage.",
                "evidence_paths": [&w043_obligation_map_path, &w037_tla_inventory_path],
                "failures": if w040_obligation_exists(&w043_obligation_map, "W043-OBL-013") && routine_tla_config_count == 11 { Vec::<String>::new() } else { vec!["w043_full_tla_blocker_missing".to_string()] },
                "validation_state": if w040_obligation_exists(&w043_obligation_map, "W043-OBL-013") && routine_tla_config_count == 11 { "w043_lean_tla_exact_blocker_validated" } else { "w043_lean_tla_exact_blocker_failed" }
            }),
            json!({
                "row_id": "w043_rust_totality_dependency_exact_blocker",
                "w043_obligation_id": "W043-OBL-014",
                "source_inputs": ["W043 Rust exact blocker register", "W043 Lean/TLA bridge"],
                "disposition_kind": "exact_rust_dependency_blocker",
                "disposition": "retain Rust totality/refinement dependency as exact proof/model blocker while W043 Rust packet still carries exact blockers",
                "local_checked_proof": true,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": true,
                "exact_remaining_blocker": true,
                "authority_owner": "calc-2p3.3; calc-2p3.4; calc-2p3.10",
                "promotion_consequence": "Rust totality/refinement and full optimized/core verification remain unpromoted",
                "reason": "W043.3 adds two automatic dynamic transition rows but retains runtime panic, broader dynamic, callable metadata, and full optimized/core blockers.",
                "evidence_paths": [&w043_rust_summary_path, &w043_rust_blockers_path, W043_LEAN_RUST_TOTALITY_FILE],
                "failures": if counter_value(&w043_rust_summary, "exact_remaining_blocker_count") == 4 && counter_value(&w043_rust_blockers, "exact_remaining_blocker_count") == 4 { Vec::<String>::new() } else { vec!["w043_rust_dependency_blocker_missing".to_string()] },
                "validation_state": if counter_value(&w043_rust_summary, "exact_remaining_blocker_count") == 4 && counter_value(&w043_rust_blockers, "exact_remaining_blocker_count") == 4 { "w043_lean_tla_exact_blocker_validated" } else { "w043_lean_tla_exact_blocker_failed" }
            }),
            json!({
                "row_id": "w043_let_lambda_external_oxfunc_boundary",
                "w043_obligation_id": "W043-OBL-036",
                "source_inputs": ["W043 proof-service obligation map", "LET/LAMBDA carrier seam"],
                "disposition_kind": "accepted_external_seam_boundary",
                "disposition": "keep LET/LAMBDA carrier interaction inside OxCalc/OxFml formalization while excluding general OxFunc kernels",
                "local_checked_proof": false,
                "bounded_model": false,
                "accepted_external_seam": true,
                "accepted_boundary": true,
                "totality_boundary": false,
                "exact_remaining_blocker": false,
                "authority_owner": "calc-2p3.4; calc-2p3.8; external:OxFunc",
                "promotion_consequence": "general OxFunc kernels remain unpromoted inside OxCalc",
                "reason": "W043 scope includes the carrier seam but not broad OxFunc semantic kernels.",
                "evidence_paths": [&w043_obligation_map_path, W043_LEAN_TLA_DISCHARGE_FILE],
                "failures": if w040_obligation_exists(&w043_obligation_map, "W043-OBL-015") && w040_obligation_exists(&w043_obligation_map, "W043-OBL-036") { Vec::<String>::new() } else { vec!["w043_let_lambda_external_boundary_missing".to_string()] },
                "validation_state": if w040_obligation_exists(&w043_obligation_map, "W043-OBL-015") && w040_obligation_exists(&w043_obligation_map, "W043-OBL-036") { "w043_lean_tla_boundary_validated" } else { "w043_lean_tla_boundary_failed" }
            }),
            json!({
                "row_id": "w043_formal_model_spec_evolution_guard",
                "w043_obligation_id": "W043-OBL-014",
                "source_inputs": ["W043 proof-service obligation map"],
                "disposition_kind": "accepted_spec_evolution_guard",
                "disposition": "preserve formalization as spec evolution and implementation improvement, not a fixed-spec verification only",
                "local_checked_proof": true,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": true,
                "totality_boundary": false,
                "exact_remaining_blocker": false,
                "authority_owner": "calc-2p3.4; calc-2p3.10",
                "promotion_consequence": "future proof/model evidence may correct specs or implementation before promotion",
                "reason": "W043 remains a formalization, spec-evolution, and engine-improvement workset.",
                "evidence_paths": [&w043_obligation_map_path, "docs/worksets/W043_CORE_FORMALIZATION_RELEASE_GRADE_PROOF_AND_OPERATED_SERVICE_INTEGRATION.md"],
                "failures": if w040_obligation_exists(&w043_obligation_map, "W043-OBL-012") && w040_obligation_exists(&w043_obligation_map, "W043-OBL-014") { Vec::<String>::new() } else { vec!["w043_lean_tla_obligation_missing".to_string()] },
                "validation_state": if w040_obligation_exists(&w043_obligation_map, "W043-OBL-012") && w040_obligation_exists(&w043_obligation_map, "W043-OBL-014") { "w043_lean_tla_boundary_validated" } else { "w043_lean_tla_boundary_failed" }
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
        if counter_value(&w043_obligation_summary, "obligation_count") != 36 {
            validation_failures.push("w043_obligation_count_changed".to_string());
        }
        if !array_contains_string(
            w043_obligation_summary
                .get("no_promotion_claims")
                .unwrap_or(&Value::Null),
            "full_lean_tla_verification",
        ) || !array_contains_string(
            w043_obligation_summary
                .get("no_promotion_claims")
                .unwrap_or(&Value::Null),
            "unbounded_scheduler_fairness",
        ) {
            validation_failures.push("w043_lean_tla_no_promotion_guard_missing".to_string());
        }
        if !w040_obligation_exists(&w043_obligation_map, "W043-OBL-012")
            || !w040_obligation_exists(&w043_obligation_map, "W043-OBL-013")
            || !w040_obligation_exists(&w043_obligation_map, "W043-OBL-014")
        {
            validation_failures.push("w043_lean_tla_obligation_rows_missing".to_string());
        }
        if !w037_formal_inventory_valid {
            validation_failures.push("w037_formal_inventory_not_valid".to_string());
        }
        if string_value(&w042_lean_tla_validation, "status")
            != "formal_assurance_w042_lean_tla_fairness_expansion_valid"
        {
            validation_failures.push("w042_lean_tla_predecessor_not_valid".to_string());
        }
        if counter_value(&w042_lean_tla_summary, "failed_row_count") != 0 {
            validation_failures.push("w042_lean_tla_failed_row_count_changed".to_string());
        }
        if counter_value(&w042_lean_tla_blockers, "exact_remaining_blocker_count") != 5 {
            validation_failures.push("w042_lean_tla_blocker_count_changed".to_string());
        }
        if string_value(&w043_rust_validation, "status")
            != "formal_assurance_w043_rust_totality_refinement_valid"
        {
            validation_failures.push("w043_rust_formal_assurance_not_valid".to_string());
        }
        if counter_value(&w043_rust_summary, "failed_row_count") != 0 {
            validation_failures.push("w043_rust_failed_row_count_changed".to_string());
        }
        if counter_value(&w043_rust_summary, "automatic_dynamic_transition_row_count") != 2 {
            validation_failures.push("w043_rust_dynamic_bridge_count_changed".to_string());
        }
        if string_value(&w042_stage2_validation, "status")
            != "w042_stage2_pack_grade_equivalence_valid"
        {
            validation_failures.push("w042_stage2_packet_not_valid".to_string());
        }
        if counter_value(&w042_stage2_summary, "exact_remaining_blocker_count") != 6 {
            validation_failures.push("w042_stage2_blocker_count_changed".to_string());
        }
        if bool_at(&w042_stage2_summary, "stage2_policy_promoted")
            || bool_at(&w042_stage2_summary, "pack_grade_replay_promoted")
            || bool_at(&w042_stage2_gate, "pack_grade_replay_promoted")
        {
            validation_failures.push("w042_stage2_promoted_unexpectedly".to_string());
        }
        if lean_placeholder_count != 0 {
            validation_failures.push("w043_lean_placeholder_count_nonzero".to_string());
        }
        if !lean_discharge_file_present {
            validation_failures.push("w043_lean_tla_discharge_file_missing".to_string());
        }
        if !w073_typed_only_guard_present {
            validation_failures.push("w043_w073_typed_only_guard_missing".to_string());
        }
        if failed_row_count != 0 {
            validation_failures.push("w043_lean_tla_row_failures_present".to_string());
        }
        if blocker_rows.len() != 5 {
            validation_failures.push("w043_expected_five_lean_tla_exact_blockers".to_string());
        }

        let source_evidence_index_path =
            format!("{relative_artifact_root}/source_evidence_index.json");
        let ledger_path = format!("{relative_artifact_root}/w043_lean_tla_discharge_ledger.json");
        let lean_register_path = format!("{relative_artifact_root}/w043_lean_proof_register.json");
        let model_register_path =
            format!("{relative_artifact_root}/w043_tla_model_bound_register.json");
        let blocker_register_path =
            format!("{relative_artifact_root}/w043_lean_tla_exact_blocker_register.json");
        let validation_path = format!("{relative_artifact_root}/validation.json");

        write_json(
            &artifact_root.join("source_evidence_index.json"),
            &json!({
                "schema_version": FORMAL_ASSURANCE_SOURCE_INDEX_SCHEMA_V1,
                "run_id": run_id,
                "canonical_run_id": W043_LEAN_TLA_FORMAL_ASSURANCE_RUN_ID,
                "source_artifacts": {
                    "w043_proof_service_obligation_summary": w043_obligation_summary_path,
                    "w043_proof_service_obligation_map": w043_obligation_map_path,
                    "w037_formal_inventory_summary": w037_formal_summary_path,
                    "w037_formal_inventory_validation": w037_formal_validation_path,
                    "w037_tla_inventory": w037_tla_inventory_path,
                    "w042_lean_tla_summary": w042_lean_tla_summary_path,
                    "w042_lean_tla_validation": w042_lean_tla_validation_path,
                    "w042_lean_tla_exact_blockers": w042_lean_tla_blockers_path,
                    "w043_rust_formal_assurance_summary": w043_rust_summary_path,
                    "w043_rust_formal_assurance_validation": w043_rust_validation_path,
                    "w043_rust_exact_blockers": w043_rust_blockers_path,
                    "w043_rust_refinement_register": w043_rust_refinement_path,
                    "w043_rust_totality_refinement_ledger": w043_rust_ledger_path,
                    "w042_stage2_summary": w042_stage2_summary_path,
                    "w042_stage2_validation": w042_stage2_validation_path,
                    "w042_stage2_policy_gate": w042_stage2_gate_path,
                    "w042_stage2_exact_blockers": w042_stage2_blockers_path,
                    "w043_w073_formatting_intake": w043_w073_formatting_intake_path,
                    "w043_lean_tla_discharge_file": W043_LEAN_TLA_DISCHARGE_FILE,
                    "w043_rust_lean_file": W043_LEAN_RUST_TOTALITY_FILE,
                    "w042_lean_tla_discharge_file": W042_LEAN_TLA_DISCHARGE_FILE,
                    "w042_stage2_policy_file": W042_STAGE2_POLICY_FILE
                },
                "source_counts": {
                    "w043_obligation_count": counter_value(&w043_obligation_summary, "obligation_count"),
                    "w037_lean_file_count": counter_value(&w037_formal_summary, "lean_file_count"),
                    "w037_tla_routine_config_count": routine_tla_config_count,
                    "w037_tla_inventory_passed_count": tla_inventory_passed_count,
                    "w037_tla_failed_config_count": routine_tla_failed_count,
                    "w042_lean_tla_exact_blocker_count": counter_value(&w042_lean_tla_summary, "exact_remaining_blocker_count"),
                    "w043_rust_exact_blocker_count": counter_value(&w043_rust_summary, "exact_remaining_blocker_count"),
                    "w043_dynamic_refinement_row_count": counter_value(&w043_rust_summary, "automatic_dynamic_transition_row_count"),
                    "w042_stage2_exact_blocker_count": counter_value(&w042_stage2_summary, "exact_remaining_blocker_count"),
                    "w042_stage2_gate_exact_blocker_count": counter_value(&w042_stage2_gate, "exact_remaining_blocker_count"),
                    "w042_stage2_policy_row_count": counter_value(&w042_stage2_summary, "policy_row_count"),
                    "w073_typed_only_family_count": w043_w073_formatting_intake
                        .get("typed_rule_only_families")
                        .and_then(Value::as_array)
                        .map_or(0, Vec::len),
                    "lean_placeholder_count": lean_placeholder_count
                }
            }),
        )?;
        write_json(
            &artifact_root.join("w043_lean_tla_discharge_ledger.json"),
            &json!({
                "schema_version": FORMAL_ASSURANCE_W043_LEAN_TLA_LEDGER_SCHEMA_V1,
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
            &artifact_root.join("w043_lean_proof_register.json"),
            &json!({
                "schema_version": FORMAL_ASSURANCE_W043_LEAN_PROOF_REGISTER_SCHEMA_V1,
                "run_id": run_id,
                "local_proof_row_count": lean_proof_rows.len(),
                "rows": lean_proof_rows
            }),
        )?;
        write_json(
            &artifact_root.join("w043_tla_model_bound_register.json"),
            &json!({
                "schema_version": FORMAL_ASSURANCE_W043_TLA_MODEL_REGISTER_SCHEMA_V1,
                "run_id": run_id,
                "bounded_model_row_count": model_bound_rows.len(),
                "rows": model_bound_rows
            }),
        )?;
        write_json(
            &artifact_root.join("w043_lean_tla_exact_blocker_register.json"),
            &json!({
                "schema_version": FORMAL_ASSURANCE_W043_LEAN_TLA_BLOCKER_REGISTER_SCHEMA_V1,
                "run_id": run_id,
                "exact_remaining_blocker_count": blocker_rows.len(),
                "rows": blocker_rows
            }),
        )?;

        let validation_status = if validation_failures.is_empty() {
            "formal_assurance_w043_lean_tla_fairness_valid"
        } else {
            "formal_assurance_w043_lean_tla_fairness_invalid"
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
                "dynamic_refinement_bridge_row_count": 2,
                "w073_typed_only_guard_present": w073_typed_only_guard_present,
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
                "dynamic_refinement_bridge_row_count": 2,
                "promotion_claims": {
                    "full_lean_verification_promoted": false,
                    "full_tla_verification_promoted": false,
                    "scheduler_fairness_promoted": false,
                    "unbounded_model_coverage_promoted": false,
                    "rust_engine_totality_promoted": false,
                    "rust_refinement_promoted": false,
                    "full_optimized_core_verification_promoted": false,
                    "stage2_policy_promoted": false,
                    "pack_grade_replay_promoted": false,
                    "c5_promoted": false,
                    "callable_carrier_sufficiency_promoted": false,
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

    fn execute_w044_rust_totality_refinement(
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

        let w044_summary_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "release-grade-ledger",
            W044_RESIDUAL_LEDGER_RUN_ID,
            "run_summary.json",
        ]);
        let w044_blocker_map_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "release-grade-ledger",
            W044_RESIDUAL_LEDGER_RUN_ID,
            "blocker_reclassification_map.json",
        ]);
        let w043_rust_summary_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "formal-assurance",
            W043_RUST_FORMAL_ASSURANCE_RUN_ID,
            "run_summary.json",
        ]);
        let w043_rust_validation_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "formal-assurance",
            W043_RUST_FORMAL_ASSURANCE_RUN_ID,
            "validation.json",
        ]);
        let w043_rust_refinement_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "formal-assurance",
            W043_RUST_FORMAL_ASSURANCE_RUN_ID,
            "w043_rust_refinement_register.json",
        ]);
        let w043_rust_blockers_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "formal-assurance",
            W043_RUST_FORMAL_ASSURANCE_RUN_ID,
            "w043_rust_exact_blocker_register.json",
        ]);
        let w044_conformance_summary_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "implementation-conformance",
            W044_IMPLEMENTATION_CONFORMANCE_RUN_ID,
            "run_summary.json",
        ]);
        let w044_conformance_validation_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "implementation-conformance",
            W044_IMPLEMENTATION_CONFORMANCE_RUN_ID,
            "validation.json",
        ]);
        let w044_disposition_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "implementation-conformance",
            W044_IMPLEMENTATION_CONFORMANCE_RUN_ID,
            "w044_optimized_core_disposition_register.json",
        ]);
        let w044_blocker_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "implementation-conformance",
            W044_IMPLEMENTATION_CONFORMANCE_RUN_ID,
            "w044_exact_remaining_blocker_register.json",
        ]);
        let w044_dynamic_transition_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "implementation-conformance",
            W044_IMPLEMENTATION_CONFORMANCE_RUN_ID,
            "w044_dynamic_transition_evidence.json",
        ]);
        let w044_callable_projection_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "implementation-conformance",
            W044_IMPLEMENTATION_CONFORMANCE_RUN_ID,
            "w044_callable_metadata_projection_register.json",
        ]);
        let w044_treecalc_summary_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "treecalc-local",
            W044_TREECALC_RUN_ID,
            "run_summary.json",
        ]);
        let w044_mixed_post_edit_result_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "treecalc-local",
            W044_TREECALC_RUN_ID,
            "cases",
            "tc_local_dynamic_mixed_add_release_auto_post_edit_001",
            "post_edit",
            "result.json",
        ]);
        let w044_let_lambda_result_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "treecalc-local",
            W044_TREECALC_RUN_ID,
            "cases",
            "tc_local_w034_higher_order_let_lambda_publish_001",
            "result.json",
        ]);

        let w044_summary = read_json(repo_root, &w044_summary_path)?;
        let w044_blocker_map = read_json(repo_root, &w044_blocker_map_path)?;
        let w043_rust_summary = read_json(repo_root, &w043_rust_summary_path)?;
        let w043_rust_validation = read_json(repo_root, &w043_rust_validation_path)?;
        let w043_rust_refinement = read_json(repo_root, &w043_rust_refinement_path)?;
        let w043_rust_blockers = read_json(repo_root, &w043_rust_blockers_path)?;
        let w044_conformance_summary = read_json(repo_root, &w044_conformance_summary_path)?;
        let w044_conformance_validation = read_json(repo_root, &w044_conformance_validation_path)?;
        let w044_disposition = read_json(repo_root, &w044_disposition_path)?;
        let w044_blockers = read_json(repo_root, &w044_blocker_path)?;
        let w044_dynamic_transition = read_json(repo_root, &w044_dynamic_transition_path)?;
        let w044_callable_projection = read_json(repo_root, &w044_callable_projection_path)?;
        let w044_treecalc_summary = read_json(repo_root, &w044_treecalc_summary_path)?;
        let w044_mixed_post_edit_result = read_json(repo_root, &w044_mixed_post_edit_result_path)?;
        let w044_let_lambda_result = read_json(repo_root, &w044_let_lambda_result_path)?;

        let lean_file_present = repo_root.join(W044_LEAN_RUST_TOTALITY_FILE).exists();
        let w043_lean_file_present = repo_root.join(W043_LEAN_RUST_TOTALITY_FILE).exists();
        let panic_marker_count = panic_marker_count(repo_root, W040_RUST_PANIC_AUDIT_FILES)?;
        let mixed_dynamic_reasons_present = [
            "DependencyAdded",
            "DependencyRemoved",
            "DependencyReclassified",
        ]
        .iter()
        .all(|reason| {
            array_contains_string(
                w044_dynamic_transition
                    .get("observed_seed_reasons")
                    .unwrap_or(&Value::Null),
                reason,
            )
        }) && [
            "DependencyAdded",
            "DependencyRemoved",
            "DependencyReclassified",
        ]
        .iter()
        .all(|reason| {
            array_contains_string(
                w044_dynamic_transition
                    .get("observed_closure_reasons")
                    .unwrap_or(&Value::Null),
                reason,
            )
        });
        let mixed_dynamic_rejected_for_rebind =
            string_value(&w044_dynamic_transition, "post_edit_result_state") == "rejected"
                && string_value(&w044_dynamic_transition, "post_edit_reject_kind")
                    == "HostInjectedFailure"
                && string_value(&w044_mixed_post_edit_result, "result_state") == "rejected"
                && w044_mixed_post_edit_result
                    .get("reject_detail")
                    .and_then(|detail| detail.get("kind"))
                    .and_then(Value::as_str)
                    == Some("HostInjectedFailure");
        let mixed_dynamic_no_publication = w044_mixed_post_edit_result
            .get("published_values")
            .and_then(Value::as_object)
            .is_some_and(serde_json::Map::is_empty);
        let let_lambda_value_observed = string_value(&w044_let_lambda_result, "result_state")
            == "published"
            && treecalc_result_publishes_value(&w044_let_lambda_result, "3", "17");
        let w073_typed_only_guard_present =
            bool_at(&w044_summary, "oxfml_formatting_update_incorporated")
                && !bool_at(
                    &w044_summary,
                    "w073_downstream_request_construction_uptake_verified",
                );

        let proof_rows = vec![
            json!({
                "row_id": "w044_result_error_carrier_totality_evidence",
                "w044_obligation_id": "W044-OBL-013",
                "source_inputs": ["Rust typed error carriers", W044_LEAN_RUST_TOTALITY_FILE],
                "disposition_kind": "direct_totality_evidence",
                "disposition": "retain promoted core public paths as typed Result/error carrier evidence rather than panic-as-contract",
                "local_checked_proof": true,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": false,
                "refinement_row": false,
                "automatic_dynamic_transition_row": false,
                "exact_remaining_blocker": false,
                "authority_owner": "calc-b1t.3",
                "promotion_consequence": "Rust totality remains unpromoted because this is carrier evidence, not whole-engine proof",
                "reason": "Core execution, fixture, runner, structural, and coordinator surfaces expose Result/typed error APIs for promoted evidence paths.",
                "evidence_paths": [
                    "src/oxcalc-core/src/coordinator.rs",
                    "src/oxcalc-core/src/recalc.rs",
                    "src/oxcalc-core/src/structural.rs",
                    "src/oxcalc-core/src/treecalc.rs",
                    "src/oxcalc-core/src/treecalc_fixture.rs",
                    "src/oxcalc-core/src/treecalc_runner.rs",
                    W044_LEAN_RUST_TOTALITY_FILE
                ],
                "failures": if lean_file_present { Vec::<String>::new() } else { vec!["w044_lean_rust_totality_file_missing".to_string()] },
                "validation_state": if lean_file_present { "w044_rust_totality_row_validated" } else { "w044_rust_totality_row_failed" }
            }),
            json!({
                "row_id": "w044_treecalc_packet_totality_evidence",
                "w044_obligation_id": "W044-OBL-013",
                "source_inputs": ["W044 TreeCalc replay", "W044 optimized/core conformance packet"],
                "disposition_kind": "direct_totality_evidence",
                "disposition": "current W044 replay emits deterministic typed artifacts for mixed dynamic transition, reject, no-publication, and LET/LAMBDA value-carrier paths",
                "local_checked_proof": true,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": false,
                "refinement_row": false,
                "automatic_dynamic_transition_row": false,
                "exact_remaining_blocker": false,
                "authority_owner": "calc-b1t.2; calc-b1t.3",
                "promotion_consequence": "the exercised W044 paths are evidenced, while whole-engine Rust totality remains blocked",
                "reason": "The W044 TreeCalc replay emits 28 cases with zero expectation mismatches and the W044.2 conformance packet validates its source artifacts.",
                "evidence_paths": [&w044_conformance_summary_path, &w044_treecalc_summary_path],
                "failures": if counter_value(&w044_conformance_summary, "failed_row_count") == 0 && counter_value(&w044_treecalc_summary, "expectation_mismatch_count") == 0 && counter_value(&w044_treecalc_summary, "case_count") == 28 { Vec::<String>::new() } else { vec!["w044_treecalc_packet_totality_evidence_missing".to_string()] },
                "validation_state": if counter_value(&w044_conformance_summary, "failed_row_count") == 0 && counter_value(&w044_treecalc_summary, "expectation_mismatch_count") == 0 && counter_value(&w044_treecalc_summary, "case_count") == 28 { "w044_rust_totality_row_validated" } else { "w044_rust_totality_row_failed" }
            }),
            json!({
                "row_id": "w044_w043_rust_refinement_regression_evidence",
                "w044_obligation_id": "W044-OBL-014",
                "source_inputs": ["W043 Rust totality/refinement packet"],
                "disposition_kind": "carried_refinement_evidence",
                "disposition": "retain W043 Rust refinement classifications as predecessor regression evidence for the W044 frontier",
                "local_checked_proof": true,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": false,
                "refinement_row": true,
                "automatic_dynamic_transition_row": false,
                "exact_remaining_blocker": false,
                "authority_owner": "calc-b1t.3",
                "promotion_consequence": "predecessor refinement evidence is retained but is not a full Rust-engine refinement proof",
                "reason": "The W043 Rust packet remains valid, has zero failed rows, and records two automatic dynamic transition refinement rows.",
                "evidence_paths": [&w043_rust_summary_path, &w043_rust_validation_path, &w043_rust_refinement_path],
                "failures": if string_value(&w043_rust_validation, "status") == "formal_assurance_w043_rust_totality_refinement_valid" && counter_value(&w043_rust_summary, "failed_row_count") == 0 && counter_value(&w043_rust_summary, "automatic_dynamic_transition_row_count") == 2 { Vec::<String>::new() } else { vec!["w043_rust_regression_evidence_not_valid".to_string()] },
                "validation_state": if string_value(&w043_rust_validation, "status") == "formal_assurance_w043_rust_totality_refinement_valid" && counter_value(&w043_rust_summary, "failed_row_count") == 0 && counter_value(&w043_rust_summary, "automatic_dynamic_transition_row_count") == 2 { "w044_rust_refinement_row_validated" } else { "w044_rust_refinement_row_failed" }
            }),
            json!({
                "row_id": "w044_mixed_dynamic_add_release_refinement_evidence",
                "w044_obligation_id": "W044-OBL-014",
                "source_inputs": ["W044 mixed dynamic add/release TreeCalc run"],
                "disposition_kind": "direct_refinement_evidence",
                "disposition": "automatic mixed dynamic transition derives DependencyAdded, DependencyRemoved, and DependencyReclassified and forces rebind behavior in the W044 replay",
                "local_checked_proof": true,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": false,
                "refinement_row": true,
                "automatic_dynamic_transition_row": true,
                "exact_remaining_blocker": false,
                "authority_owner": "calc-b1t.2; calc-b1t.3",
                "promotion_consequence": "dynamic-transition refinement is evidenced for the exercised mixed add/release pattern, while broader dynamic coverage remains blocked",
                "reason": "The W044 mixed post-edit closure records all three dependency transition reasons and the post-edit result rejects with HostInjectedFailure before publication.",
                "evidence_paths": [&w044_dynamic_transition_path, &w044_mixed_post_edit_result_path],
                "observed": {
                    "mixed_dynamic_reasons_present": mixed_dynamic_reasons_present,
                    "mixed_dynamic_rejected_for_rebind": mixed_dynamic_rejected_for_rebind
                },
                "failures": if bool_at(&w044_dynamic_transition, "direct_evidence_bound") && mixed_dynamic_reasons_present && mixed_dynamic_rejected_for_rebind { Vec::<String>::new() } else { vec!["w044_mixed_dynamic_refinement_evidence_missing".to_string()] },
                "validation_state": if bool_at(&w044_dynamic_transition, "direct_evidence_bound") && mixed_dynamic_reasons_present && mixed_dynamic_rejected_for_rebind { "w044_rust_refinement_row_validated" } else { "w044_rust_refinement_row_failed" }
            }),
            json!({
                "row_id": "w044_publication_fence_no_publish_refinement_evidence",
                "w044_obligation_id": "W044-OBL-014",
                "source_inputs": ["W044 mixed dynamic post-edit no-publication result"],
                "disposition_kind": "direct_refinement_evidence",
                "disposition": "bind rebind-required reject/no-publication behavior as publication-fence refinement evidence for the exercised mixed dynamic transition",
                "local_checked_proof": true,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": false,
                "refinement_row": true,
                "automatic_dynamic_transition_row": false,
                "exact_remaining_blocker": false,
                "authority_owner": "calc-b1t.2; calc-b1t.3",
                "promotion_consequence": "publication-fence behavior is evidenced for this reject path, but production publication-fence breadth remains blocked",
                "reason": "The W044 mixed post-edit result rejects as HostInjectedFailure and exposes an empty published-values object.",
                "evidence_paths": [&w044_mixed_post_edit_result_path],
                "observed": {
                    "mixed_dynamic_no_publication": mixed_dynamic_no_publication,
                    "post_edit_result_state": string_value(&w044_mixed_post_edit_result, "result_state")
                },
                "failures": if mixed_dynamic_no_publication && mixed_dynamic_rejected_for_rebind { Vec::<String>::new() } else { vec!["w044_publication_fence_no_publish_evidence_missing".to_string()] },
                "validation_state": if mixed_dynamic_no_publication && mixed_dynamic_rejected_for_rebind { "w044_rust_refinement_row_validated" } else { "w044_rust_refinement_row_failed" }
            }),
            json!({
                "row_id": "w044_w043_dynamic_transition_regression_evidence",
                "w044_obligation_id": "W044-OBL-014",
                "source_inputs": ["W043 automatic dynamic transition Rust rows"],
                "disposition_kind": "carried_refinement_evidence",
                "disposition": "carry W043 automatic addition and release refinement rows as regression evidence under W044",
                "local_checked_proof": true,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": false,
                "refinement_row": true,
                "automatic_dynamic_transition_row": false,
                "exact_remaining_blocker": false,
                "authority_owner": "calc-b1t.3",
                "promotion_consequence": "carried dynamic evidence remains regression evidence, not broad dynamic transition promotion",
                "reason": "The W043 refinement register contains automatic addition and release transition rows that remain valid predecessor evidence.",
                "evidence_paths": [&w043_rust_refinement_path],
                "failures": if row_with_field_exists(&w043_rust_refinement, "row_id", "w043_automatic_dynamic_addition_refinement_evidence") && row_with_field_exists(&w043_rust_refinement, "row_id", "w043_automatic_dynamic_release_refinement_evidence") { Vec::<String>::new() } else { vec!["w043_dynamic_transition_regression_rows_missing".to_string()] },
                "validation_state": if row_with_field_exists(&w043_rust_refinement, "row_id", "w043_automatic_dynamic_addition_refinement_evidence") && row_with_field_exists(&w043_rust_refinement, "row_id", "w043_automatic_dynamic_release_refinement_evidence") { "w044_rust_refinement_row_validated" } else { "w044_rust_refinement_row_failed" }
            }),
            json!({
                "row_id": "w044_snapshot_fence_breadth_refinement_boundary",
                "w044_obligation_id": "W044-OBL-014",
                "source_inputs": ["W044 exact blocker register"],
                "disposition_kind": "exact_refinement_blocker",
                "disposition": "retain snapshot-fence counterpart breadth as an exact Rust refinement blocker beyond declared-profile evidence",
                "local_checked_proof": false,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": false,
                "refinement_row": true,
                "automatic_dynamic_transition_row": false,
                "exact_remaining_blocker": true,
                "authority_owner": "calc-b1t.3; calc-b1t.5",
                "promotion_consequence": "snapshot-fence breadth and Stage 2 production policy remain unpromoted",
                "reason": "W044.2 retains snapshot-fence counterpart breadth as an exact blocker.",
                "evidence_paths": [&w044_blocker_path],
                "failures": if row_with_field_exists(&w044_blockers, "row_id", "w044_snapshot_fence_counterpart_breadth_exact_blocker") { Vec::<String>::new() } else { vec!["w044_snapshot_fence_breadth_blocker_missing".to_string()] },
                "validation_state": if row_with_field_exists(&w044_blockers, "row_id", "w044_snapshot_fence_counterpart_breadth_exact_blocker") { "w044_rust_exact_blocker_validated" } else { "w044_rust_exact_blocker_failed" }
            }),
            json!({
                "row_id": "w044_capability_view_breadth_refinement_boundary",
                "w044_obligation_id": "W044-OBL-014",
                "source_inputs": ["W044 exact blocker register"],
                "disposition_kind": "exact_refinement_blocker",
                "disposition": "retain capability-view counterpart breadth as an exact Rust refinement blocker beyond declared-profile evidence",
                "local_checked_proof": false,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": false,
                "refinement_row": true,
                "automatic_dynamic_transition_row": false,
                "exact_remaining_blocker": true,
                "authority_owner": "calc-b1t.3; calc-b1t.5",
                "promotion_consequence": "capability-view breadth and Stage 2 production policy remain unpromoted",
                "reason": "W044.2 retains capability-view counterpart breadth as an exact blocker.",
                "evidence_paths": [&w044_blocker_path],
                "failures": if row_with_field_exists(&w044_blockers, "row_id", "w044_capability_view_counterpart_breadth_exact_blocker") { Vec::<String>::new() } else { vec!["w044_capability_view_breadth_blocker_missing".to_string()] },
                "validation_state": if row_with_field_exists(&w044_blockers, "row_id", "w044_capability_view_counterpart_breadth_exact_blocker") { "w044_rust_exact_blocker_validated" } else { "w044_rust_exact_blocker_failed" }
            }),
            json!({
                "row_id": "w044_callable_value_carrier_totality_evidence",
                "w044_obligation_id": "W044-OBL-015",
                "source_inputs": ["W044 TreeCalc LET/LAMBDA value row"],
                "disposition_kind": "direct_callable_value_carrier_totality_evidence",
                "disposition": "bind ordinary LET/LAMBDA value-carrier publication as Rust totality evidence for the current value path while keeping callable metadata separate",
                "local_checked_proof": true,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": false,
                "refinement_row": false,
                "automatic_dynamic_transition_row": false,
                "exact_remaining_blocker": false,
                "authority_owner": "calc-b1t.3; calc-b1t.8",
                "promotion_consequence": "ordinary value-carrier behavior is evidenced; callable carrier sufficiency and metadata projection remain unpromoted",
                "reason": "The W044 TreeCalc LET/LAMBDA fixture publishes value 17 through candidate and publication value carriers.",
                "evidence_paths": [&w044_let_lambda_result_path, &w044_callable_projection_path],
                "failures": if let_lambda_value_observed { Vec::<String>::new() } else { vec!["w044_callable_value_carrier_evidence_missing".to_string()] },
                "validation_state": if let_lambda_value_observed { "w044_rust_totality_row_validated" } else { "w044_rust_totality_row_failed" }
            }),
            json!({
                "row_id": "w044_runtime_panic_surface_totality_boundary",
                "w044_obligation_id": "W044-OBL-012",
                "source_inputs": ["Rust panic-marker audit", W044_LEAN_RUST_TOTALITY_FILE],
                "disposition_kind": "exact_totality_boundary",
                "disposition": "retain a whole-engine panic-free proof blocker while panic/unwrap/expect markers remain in audited Rust surfaces",
                "local_checked_proof": true,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": true,
                "refinement_row": false,
                "automatic_dynamic_transition_row": false,
                "exact_remaining_blocker": true,
                "authority_owner": "calc-b1t.3; calc-b1t.10",
                "promotion_consequence": "Rust-engine totality and panic-free core domain remain unpromoted",
                "reason": "The marker census is a guard, not a semantic proof; observed panic-family markers require review or proof before a panic-free claim.",
                "evidence_paths": W040_RUST_PANIC_AUDIT_FILES,
                "observed": { "panic_marker_count": panic_marker_count },
                "failures": if panic_marker_count > 0 && lean_file_present { Vec::<String>::new() } else { vec!["w044_panic_surface_audit_missing".to_string()] },
                "validation_state": if panic_marker_count > 0 && lean_file_present { "w044_rust_exact_blocker_validated" } else { "w044_rust_exact_blocker_failed" }
            }),
            json!({
                "row_id": "w044_broader_dynamic_transition_coverage_refinement_boundary",
                "w044_obligation_id": "W044-OBL-014",
                "source_inputs": ["W044 exact blocker register"],
                "disposition_kind": "exact_refinement_blocker",
                "disposition": "retain broader dynamic transition coverage as an exact Rust refinement blocker after adding mixed transition evidence",
                "local_checked_proof": false,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": true,
                "refinement_row": true,
                "automatic_dynamic_transition_row": false,
                "exact_remaining_blocker": true,
                "authority_owner": "calc-b1t.3; calc-b1t.5; calc-b1t.10",
                "promotion_consequence": "full optimized/core and Rust refinement remain blocked until broader dynamic coverage or a sufficiency proof exists",
                "reason": "W044.2 narrows mixed dynamic evidence but retains broader dynamic transition coverage as an exact blocker.",
                "evidence_paths": [&w044_blocker_path, &w044_dynamic_transition_path],
                "failures": if row_with_field_exists(&w044_blockers, "row_id", "w044_broader_dynamic_transition_remaining_exact_blocker") { Vec::<String>::new() } else { vec!["w044_broader_dynamic_blocker_missing".to_string()] },
                "validation_state": if row_with_field_exists(&w044_blockers, "row_id", "w044_broader_dynamic_transition_remaining_exact_blocker") { "w044_rust_exact_blocker_validated" } else { "w044_rust_exact_blocker_failed" }
            }),
            json!({
                "row_id": "w044_callable_metadata_projection_totality_boundary",
                "w044_obligation_id": "W044-OBL-015",
                "source_inputs": ["W044 callable metadata projection register"],
                "disposition_kind": "exact_totality_boundary",
                "disposition": "retain callable metadata projection as an exact Rust totality/refinement blocker",
                "local_checked_proof": true,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": true,
                "refinement_row": true,
                "automatic_dynamic_transition_row": false,
                "exact_remaining_blocker": true,
                "authority_owner": "calc-b1t.3; calc-b1t.8",
                "promotion_consequence": "callable metadata projection and callable carrier sufficiency remain unpromoted",
                "reason": "W044.2 classifies value carrier evidence separately from metadata projection evidence.",
                "evidence_paths": [&w044_callable_projection_path, &w044_blocker_path],
                "failures": if row_with_field_exists(&w044_callable_projection, "row_id", "w044_callable_metadata_projection_exact_blocker") && row_with_field_exists(&w044_blockers, "row_id", "w044_callable_metadata_projection_exact_blocker") && lean_file_present { Vec::<String>::new() } else { vec!["w044_callable_metadata_projection_blocker_missing".to_string()] },
                "validation_state": if row_with_field_exists(&w044_callable_projection, "row_id", "w044_callable_metadata_projection_exact_blocker") && row_with_field_exists(&w044_blockers, "row_id", "w044_callable_metadata_projection_exact_blocker") && lean_file_present { "w044_rust_exact_blocker_validated" } else { "w044_rust_exact_blocker_failed" }
            }),
            json!({
                "row_id": "w044_full_optimized_core_release_grade_conformance_boundary",
                "w044_obligation_id": "W044-OBL-014",
                "source_inputs": ["W044 residual release-grade ledger", "W044 optimized/core conformance packet"],
                "disposition_kind": "exact_release_grade_boundary",
                "disposition": "retain full optimized/core release-grade conformance as a Rust refinement boundary until all W044 promotion contracts are discharged",
                "local_checked_proof": false,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": true,
                "refinement_row": true,
                "automatic_dynamic_transition_row": false,
                "exact_remaining_blocker": true,
                "authority_owner": "calc-b1t.3; calc-b1t.10; calc-b1t.11",
                "promotion_consequence": "full optimized/core verification, C5, pack-grade replay, and release-grade verification remain unpromoted",
                "reason": "The W044 residual ledger and W044.2 conformance packet both retain no-promotion claims for full optimized/core verification.",
                "evidence_paths": [&w044_summary_path, &w044_conformance_summary_path],
                "failures": if array_contains_string(w044_summary.get("no_promotion_claims").unwrap_or(&Value::Null), "full_optimized_core_verification") && counter_value(&w044_conformance_summary, "w044_match_promoted_count") == 0 { Vec::<String>::new() } else { vec!["w044_full_optimized_core_boundary_missing".to_string()] },
                "validation_state": if array_contains_string(w044_summary.get("no_promotion_claims").unwrap_or(&Value::Null), "full_optimized_core_verification") && counter_value(&w044_conformance_summary, "w044_match_promoted_count") == 0 { "w044_rust_exact_blocker_validated" } else { "w044_rust_exact_blocker_failed" }
            }),
            json!({
                "row_id": "w044_let_lambda_carrier_external_boundary",
                "w044_obligation_id": "W044-OBL-015",
                "source_inputs": ["W044 residual release-grade ledger", W044_LEAN_RUST_TOTALITY_FILE],
                "disposition_kind": "accepted_external_seam_boundary",
                "disposition": "keep LET/LAMBDA carrier interaction inside OxCalc/OxFml formalization while excluding general OxFunc kernels",
                "local_checked_proof": false,
                "bounded_model": false,
                "accepted_external_seam": true,
                "accepted_boundary": true,
                "totality_boundary": false,
                "refinement_row": false,
                "automatic_dynamic_transition_row": false,
                "exact_remaining_blocker": false,
                "authority_owner": "calc-b1t.3; calc-b1t.8; external:OxFunc",
                "promotion_consequence": "general OxFunc kernels remain unpromoted inside OxCalc",
                "reason": "W044 scope includes the LET/LAMBDA carrier seam but not broad OxFunc semantic kernels.",
                "evidence_paths": [&w044_summary_path, W044_LEAN_RUST_TOTALITY_FILE],
                "failures": if array_contains_string(w044_summary.get("no_promotion_claims").unwrap_or(&Value::Null), "general_oxfunc_kernels") && lean_file_present { Vec::<String>::new() } else { vec!["w044_let_lambda_boundary_missing".to_string()] },
                "validation_state": if array_contains_string(w044_summary.get("no_promotion_claims").unwrap_or(&Value::Null), "general_oxfunc_kernels") && lean_file_present { "w044_rust_boundary_validated" } else { "w044_rust_boundary_failed" }
            }),
            json!({
                "row_id": "w044_spec_evolution_refinement_guard",
                "w044_obligation_id": "W044-OBL-003",
                "source_inputs": ["W044 workset and residual blocker map"],
                "disposition_kind": "accepted_spec_evolution_guard",
                "disposition": "preserve formalization as spec evolution and implementation improvement, not a fixed-spec test only",
                "local_checked_proof": true,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": true,
                "totality_boundary": false,
                "refinement_row": false,
                "automatic_dynamic_transition_row": false,
                "exact_remaining_blocker": false,
                "authority_owner": "calc-b1t.3; calc-b1t.11",
                "promotion_consequence": "future proof evidence may correct specs or implementation before promotion",
                "reason": "The W044 workset records spec-evolution hooks for Rust totality and refinement obligations.",
                "evidence_paths": [&w044_blocker_map_path, "docs/worksets/W044_CORE_FORMALIZATION_RELEASE_GRADE_BLOCKER_BURN_DOWN_AND_SERVICE_PROOF_CLOSURE.md"],
                "failures": if row_with_field_exists(&w044_blocker_map, "source_lane", "w043_residual.rust_totality_refinement_panic_surface") { Vec::<String>::new() } else { vec!["w044_refinement_obligation_missing".to_string()] },
                "validation_state": if row_with_field_exists(&w044_blocker_map, "source_lane", "w043_residual.rust_totality_refinement_panic_surface") { "w044_rust_boundary_validated" } else { "w044_rust_boundary_failed" }
            }),
            json!({
                "row_id": "w044_w073_typed_formatting_rust_boundary_guard",
                "w044_obligation_id": "W044-OBL-033",
                "source_inputs": ["W044 W073 typed-only formatting intake"],
                "disposition_kind": "accepted_formatting_boundary",
                "disposition": "carry W073 typed-rule-only formatting intake as an accepted non-Rust promotion boundary",
                "local_checked_proof": true,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": true,
                "totality_boundary": false,
                "refinement_row": false,
                "automatic_dynamic_transition_row": false,
                "exact_remaining_blocker": false,
                "authority_owner": "calc-b1t.3; calc-b1t.8",
                "promotion_consequence": "W073 formatting intake does not promote Rust totality, callable metadata, broad OxFml, or optimized/core verification",
                "reason": "W044.1 records typed-rule-only formatting intake and downstream request construction remains unverified.",
                "evidence_paths": [&w044_summary_path],
                "failures": if w073_typed_only_guard_present { Vec::<String>::new() } else { vec!["w044_w073_typed_guard_missing".to_string()] },
                "validation_state": if w073_typed_only_guard_present { "w044_rust_boundary_validated" } else { "w044_rust_boundary_failed" }
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
        let automatic_dynamic_transition_row_count = proof_rows
            .iter()
            .filter(|row| bool_at(row, "automatic_dynamic_transition_row"))
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
        if counter_value(&w044_summary, "obligation_count") != 45 {
            validation_failures.push("w044_obligation_count_changed".to_string());
        }
        if !array_contains_string(
            w044_summary
                .get("no_promotion_claims")
                .unwrap_or(&Value::Null),
            "rust_totality_and_refinement",
        ) || !array_contains_string(
            w044_summary
                .get("no_promotion_claims")
                .unwrap_or(&Value::Null),
            "panic_free_core_domain",
        ) {
            validation_failures.push("w044_rust_no_promotion_guard_missing".to_string());
        }
        if string_value(&w043_rust_validation, "status")
            != "formal_assurance_w043_rust_totality_refinement_valid"
        {
            validation_failures.push("w043_rust_formal_assurance_not_valid".to_string());
        }
        if counter_value(&w043_rust_summary, "failed_row_count") != 0 {
            validation_failures.push("w043_rust_failed_row_count_changed".to_string());
        }
        if counter_value(&w043_rust_blockers, "exact_remaining_blocker_count") != 4 {
            validation_failures.push("w043_rust_exact_blocker_count_changed".to_string());
        }
        if string_value(&w044_conformance_validation, "status")
            != "implementation_conformance_w044_dynamic_transition_callable_metadata_valid"
        {
            validation_failures.push("w044_conformance_validation_not_passed".to_string());
        }
        if counter_value(
            &w044_conformance_summary,
            "w044_exact_remaining_blocker_count",
        ) != 4
        {
            validation_failures.push("w044_conformance_exact_blocker_count_changed".to_string());
        }
        if !row_with_field_exists(
            &w044_disposition,
            "row_id",
            "w044_dynamic_mixed_add_release_direct_evidence",
        ) {
            validation_failures.push("w044_mixed_dynamic_disposition_missing".to_string());
        }
        if counter_value(&w044_treecalc_summary, "expectation_mismatch_count") != 0 {
            validation_failures.push("w044_treecalc_expectation_mismatch_present".to_string());
        }
        if counter_value(&w044_treecalc_summary, "case_count") != 28 {
            validation_failures.push("w044_treecalc_case_count_changed".to_string());
        }
        if !lean_file_present || !w043_lean_file_present {
            validation_failures.push("w044_or_w043_lean_rust_totality_file_missing".to_string());
        }
        if panic_marker_count == 0 {
            validation_failures.push("w044_panic_marker_audit_unexpected_zero".to_string());
        }
        if !w073_typed_only_guard_present {
            validation_failures.push("w044_w073_typed_only_guard_missing".to_string());
        }
        if failed_row_count != 0 {
            validation_failures.push("w044_rust_totality_row_failures_present".to_string());
        }
        if blocker_rows.len() != 6 {
            validation_failures.push("w044_expected_six_rust_exact_blockers".to_string());
        }
        if automatic_dynamic_transition_row_count != 1 {
            validation_failures.push("w044_expected_one_mixed_dynamic_transition_row".to_string());
        }
        if totality_rows.len() != 4 {
            validation_failures.push("w044_expected_four_totality_boundaries".to_string());
        }

        let source_evidence_index_path =
            format!("{relative_artifact_root}/source_evidence_index.json");
        let rust_ledger_path =
            format!("{relative_artifact_root}/w044_rust_totality_refinement_ledger.json");
        let totality_register_path =
            format!("{relative_artifact_root}/w044_rust_totality_boundary_register.json");
        let refinement_register_path =
            format!("{relative_artifact_root}/w044_rust_refinement_register.json");
        let blocker_register_path =
            format!("{relative_artifact_root}/w044_rust_exact_blocker_register.json");
        let validation_path = format!("{relative_artifact_root}/validation.json");

        write_json(
            &artifact_root.join("source_evidence_index.json"),
            &json!({
                "schema_version": FORMAL_ASSURANCE_SOURCE_INDEX_SCHEMA_V1,
                "run_id": run_id,
                "source_artifacts": {
                    "w044_residual_summary": w044_summary_path,
                    "w044_blocker_reclassification_map": w044_blocker_map_path,
                    "w043_rust_formal_assurance_summary": w043_rust_summary_path,
                    "w043_rust_formal_assurance_validation": w043_rust_validation_path,
                    "w043_rust_refinement_register": w043_rust_refinement_path,
                    "w043_rust_exact_blockers": w043_rust_blockers_path,
                    "w044_implementation_conformance_summary": w044_conformance_summary_path,
                    "w044_implementation_conformance_validation": w044_conformance_validation_path,
                    "w044_implementation_conformance_disposition": w044_disposition_path,
                    "w044_implementation_conformance_exact_blockers": w044_blocker_path,
                    "w044_dynamic_transition_evidence": w044_dynamic_transition_path,
                    "w044_callable_metadata_projection_register": w044_callable_projection_path,
                    "w044_treecalc_summary": w044_treecalc_summary_path,
                    "w044_mixed_dynamic_post_edit_result": w044_mixed_post_edit_result_path,
                    "w044_treecalc_let_lambda_result": w044_let_lambda_result_path,
                    "w044_lean_rust_totality_file": W044_LEAN_RUST_TOTALITY_FILE
                },
                "source_counts": {
                    "w044_obligation_count": counter_value(&w044_summary, "obligation_count"),
                    "w043_rust_exact_blocker_count": counter_value(&w043_rust_blockers, "exact_remaining_blocker_count"),
                    "w044_conformance_exact_blocker_count": counter_value(&w044_conformance_summary, "w044_exact_remaining_blocker_count"),
                    "w044_treecalc_case_count": counter_value(&w044_treecalc_summary, "case_count"),
                    "panic_marker_count": panic_marker_count,
                    "mixed_dynamic_reasons_present": mixed_dynamic_reasons_present,
                    "mixed_dynamic_no_publication": mixed_dynamic_no_publication
                }
            }),
        )?;
        write_json(
            &artifact_root.join("w044_rust_totality_refinement_ledger.json"),
            &json!({
                "schema_version": FORMAL_ASSURANCE_W044_RUST_LEDGER_SCHEMA_V1,
                "run_id": run_id,
                "rust_row_count": proof_rows.len(),
                "local_proof_row_count": local_proof_row_count,
                "bounded_model_row_count": bounded_model_row_count,
                "accepted_external_seam_count": accepted_external_seam_count,
                "accepted_boundary_count": accepted_boundary_count,
                "automatic_dynamic_transition_row_count": automatic_dynamic_transition_row_count,
                "totality_boundary_count": totality_rows.len(),
                "refinement_row_count": refinement_rows.len(),
                "exact_remaining_blocker_count": blocker_rows.len(),
                "rows": proof_rows
            }),
        )?;
        write_json(
            &artifact_root.join("w044_rust_totality_boundary_register.json"),
            &json!({
                "schema_version": FORMAL_ASSURANCE_W044_TOTALITY_REGISTER_SCHEMA_V1,
                "run_id": run_id,
                "totality_boundary_count": totality_rows.len(),
                "rows": totality_rows
            }),
        )?;
        write_json(
            &artifact_root.join("w044_rust_refinement_register.json"),
            &json!({
                "schema_version": FORMAL_ASSURANCE_W044_REFINEMENT_REGISTER_SCHEMA_V1,
                "run_id": run_id,
                "refinement_row_count": refinement_rows.len(),
                "automatic_dynamic_transition_row_count": automatic_dynamic_transition_row_count,
                "rows": refinement_rows
            }),
        )?;
        write_json(
            &artifact_root.join("w044_rust_exact_blocker_register.json"),
            &json!({
                "schema_version": FORMAL_ASSURANCE_W044_BLOCKER_REGISTER_SCHEMA_V1,
                "run_id": run_id,
                "exact_remaining_blocker_count": blocker_rows.len(),
                "rows": blocker_rows
            }),
        )?;

        let validation_status = if validation_failures.is_empty() {
            "formal_assurance_w044_rust_totality_refinement_valid"
        } else {
            "formal_assurance_w044_rust_totality_refinement_invalid"
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
                "automatic_dynamic_transition_row_count": automatic_dynamic_transition_row_count,
                "totality_boundary_count": totality_rows.len(),
                "refinement_row_count": refinement_rows.len(),
                "exact_remaining_blocker_count": blocker_rows.len(),
                "failed_row_count": failed_row_count,
                "mixed_dynamic_seed_reasons_verified": ["DependencyAdded", "DependencyRemoved", "DependencyReclassified"],
                "mixed_dynamic_no_publication_verified": mixed_dynamic_no_publication,
                "w073_typed_only_guard_present": w073_typed_only_guard_present,
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
                "automatic_dynamic_transition_row_count": automatic_dynamic_transition_row_count,
                "totality_boundary_count": totality_rows.len(),
                "refinement_row_count": refinement_rows.len(),
                "exact_remaining_blocker_count": blocker_rows.len(),
                "failed_row_count": failed_row_count,
                "panic_marker_count": panic_marker_count,
                "promotion_claims": {
                    "rust_engine_totality_promoted": false,
                    "rust_refinement_promoted": false,
                    "panic_free_core_domain_promoted": false,
                    "full_optimized_core_verification_promoted": false,
                    "full_lean_verification_promoted": false,
                    "full_tla_verification_promoted": false,
                    "stage2_policy_promoted": false,
                    "callable_metadata_projection_promoted": false,
                    "callable_carrier_sufficiency_promoted": false,
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

    fn execute_w043_rust_totality_refinement(
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

        let w043_obligation_summary_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "release-grade-ledger",
            W043_RESIDUAL_LEDGER_RUN_ID,
            "run_summary.json",
        ]);
        let w043_obligation_map_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "release-grade-ledger",
            W043_RESIDUAL_LEDGER_RUN_ID,
            "proof_service_obligation_map.json",
        ]);
        let w042_rust_summary_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "formal-assurance",
            W042_RUST_FORMAL_ASSURANCE_RUN_ID,
            "run_summary.json",
        ]);
        let w042_rust_validation_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "formal-assurance",
            W042_RUST_FORMAL_ASSURANCE_RUN_ID,
            "validation.json",
        ]);
        let w042_rust_refinement_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "formal-assurance",
            W042_RUST_FORMAL_ASSURANCE_RUN_ID,
            "w042_rust_refinement_register.json",
        ]);
        let w043_conformance_summary_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "implementation-conformance",
            W043_IMPLEMENTATION_CONFORMANCE_RUN_ID,
            "run_summary.json",
        ]);
        let w043_conformance_register_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "implementation-conformance",
            W043_IMPLEMENTATION_CONFORMANCE_RUN_ID,
            "w043_counterpart_conformance_register.json",
        ]);
        let w043_conformance_blockers_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "implementation-conformance",
            W043_IMPLEMENTATION_CONFORMANCE_RUN_ID,
            "w043_exact_remaining_blocker_register.json",
        ]);
        let w043_callable_projection_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "implementation-conformance",
            W043_IMPLEMENTATION_CONFORMANCE_RUN_ID,
            "w043_callable_metadata_projection_register.json",
        ]);
        let w043_dynamic_transition_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "implementation-conformance",
            W043_IMPLEMENTATION_CONFORMANCE_RUN_ID,
            "w043_dynamic_transition_evidence.json",
        ]);
        let w043_w073_formatting_intake_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "implementation-conformance",
            W043_IMPLEMENTATION_CONFORMANCE_RUN_ID,
            "w073_formatting_intake.json",
        ]);
        let w043_treecalc_summary_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "treecalc-local",
            W043_TREECALC_RUN_ID,
            "run_summary.json",
        ]);
        let w043_treecalc_add_post_edit_result_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "treecalc-local",
            W043_TREECALC_RUN_ID,
            "cases",
            "tc_local_dynamic_addition_auto_post_edit_001",
            "post_edit",
            "result.json",
        ]);
        let w043_treecalc_add_post_edit_closure_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "treecalc-local",
            W043_TREECALC_RUN_ID,
            "cases",
            "tc_local_dynamic_addition_auto_post_edit_001",
            "post_edit",
            "invalidation_closure.json",
        ]);
        let w043_treecalc_add_post_edit_seeds_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "treecalc-local",
            W043_TREECALC_RUN_ID,
            "cases",
            "tc_local_dynamic_addition_auto_post_edit_001",
            "post_edit",
            "invalidation_seeds.json",
        ]);
        let w043_treecalc_release_post_edit_result_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "treecalc-local",
            W043_TREECALC_RUN_ID,
            "cases",
            "tc_local_dynamic_release_reclassification_auto_post_edit_001",
            "post_edit",
            "result.json",
        ]);
        let w043_treecalc_release_post_edit_closure_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "treecalc-local",
            W043_TREECALC_RUN_ID,
            "cases",
            "tc_local_dynamic_release_reclassification_auto_post_edit_001",
            "post_edit",
            "invalidation_closure.json",
        ]);
        let w043_treecalc_release_post_edit_seeds_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "treecalc-local",
            W043_TREECALC_RUN_ID,
            "cases",
            "tc_local_dynamic_release_reclassification_auto_post_edit_001",
            "post_edit",
            "invalidation_seeds.json",
        ]);
        let w043_treecalc_capability_result_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "treecalc-local",
            W043_TREECALC_RUN_ID,
            "cases",
            "tc_local_capability_sensitive_reject_001",
            "result.json",
        ]);
        let w043_treecalc_let_lambda_result_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "treecalc-local",
            W043_TREECALC_RUN_ID,
            "cases",
            "tc_local_w034_higher_order_let_lambda_publish_001",
            "result.json",
        ]);

        let w043_obligation_summary = read_json(repo_root, &w043_obligation_summary_path)?;
        let w043_obligation_map = read_json(repo_root, &w043_obligation_map_path)?;
        let w042_rust_summary = read_json(repo_root, &w042_rust_summary_path)?;
        let w042_rust_validation = read_json(repo_root, &w042_rust_validation_path)?;
        let w042_rust_refinement = read_json(repo_root, &w042_rust_refinement_path)?;
        let w043_conformance_summary = read_json(repo_root, &w043_conformance_summary_path)?;
        let w043_conformance_register = read_json(repo_root, &w043_conformance_register_path)?;
        let w043_conformance_blockers = read_json(repo_root, &w043_conformance_blockers_path)?;
        let w043_callable_projection = read_json(repo_root, &w043_callable_projection_path)?;
        let w043_dynamic_transition = read_json(repo_root, &w043_dynamic_transition_path)?;
        let w043_w073_formatting_intake = read_json(repo_root, &w043_w073_formatting_intake_path)?;
        let w043_treecalc_summary = read_json(repo_root, &w043_treecalc_summary_path)?;
        let w043_treecalc_add_post_edit_result =
            read_json(repo_root, &w043_treecalc_add_post_edit_result_path)?;
        let w043_treecalc_add_post_edit_closure =
            read_json(repo_root, &w043_treecalc_add_post_edit_closure_path)?;
        let w043_treecalc_add_post_edit_seeds =
            read_json(repo_root, &w043_treecalc_add_post_edit_seeds_path)?;
        let w043_treecalc_release_post_edit_result =
            read_json(repo_root, &w043_treecalc_release_post_edit_result_path)?;
        let w043_treecalc_release_post_edit_closure =
            read_json(repo_root, &w043_treecalc_release_post_edit_closure_path)?;
        let w043_treecalc_release_post_edit_seeds =
            read_json(repo_root, &w043_treecalc_release_post_edit_seeds_path)?;
        let w043_treecalc_capability_result =
            read_json(repo_root, &w043_treecalc_capability_result_path)?;
        let w043_treecalc_let_lambda_result =
            read_json(repo_root, &w043_treecalc_let_lambda_result_path)?;

        let lean_file_present = repo_root.join(W043_LEAN_RUST_TOTALITY_FILE).exists();
        let w042_lean_file_present = repo_root.join(W042_LEAN_RUST_TOTALITY_FILE).exists();
        let panic_marker_count = panic_marker_count(repo_root, W040_RUST_PANIC_AUDIT_FILES)?;
        let addition_transition_seeds_present = w043_treecalc_add_post_edit_seeds
            .as_array()
            .is_some_and(|seeds| {
                seeds.iter().any(|seed| {
                    seed.get("reason").and_then(Value::as_str) == Some("DependencyAdded")
                }) && seeds.iter().any(|seed| {
                    seed.get("reason").and_then(Value::as_str) == Some("DependencyReclassified")
                })
            });
        let release_transition_seeds_present = w043_treecalc_release_post_edit_seeds
            .as_array()
            .is_some_and(|seeds| {
                seeds.iter().any(|seed| {
                    seed.get("reason").and_then(Value::as_str) == Some("DependencyRemoved")
                }) && seeds.iter().any(|seed| {
                    seed.get("reason").and_then(Value::as_str) == Some("DependencyReclassified")
                })
            });
        let addition_transition_closure_requires_rebind = w043_treecalc_add_post_edit_closure
            .as_array()
            .is_some_and(|rows| {
                rows.iter().any(|row| {
                    row.get("node_id").and_then(Value::as_u64) == Some(3)
                        && bool_at(row, "requires_rebind")
                })
            });
        let release_transition_closure_requires_rebind = w043_treecalc_release_post_edit_closure
            .as_array()
            .is_some_and(|rows| {
                rows.iter().any(|row| {
                    row.get("node_id").and_then(Value::as_u64) == Some(3)
                        && bool_at(row, "requires_rebind")
                })
            });
        let addition_transition_rejected_for_rebind =
            string_value(&w043_treecalc_add_post_edit_result, "result_state") == "rejected"
                && w043_treecalc_add_post_edit_result
                    .get("reject_detail")
                    .and_then(|detail| detail.get("kind"))
                    .and_then(Value::as_str)
                    == Some("HostInjectedFailure")
                && w043_treecalc_add_post_edit_result
                    .get("published_values")
                    .and_then(Value::as_object)
                    .is_some_and(serde_json::Map::is_empty);
        let release_transition_rejected_for_rebind =
            string_value(&w043_treecalc_release_post_edit_result, "result_state") == "rejected"
                && w043_treecalc_release_post_edit_result
                    .get("reject_detail")
                    .and_then(|detail| detail.get("kind"))
                    .and_then(Value::as_str)
                    == Some("HostInjectedFailure");
        let capability_reject_observed =
            string_value(&w043_treecalc_capability_result, "result_state") == "rejected"
                && w043_treecalc_capability_result
                    .get("reject_detail")
                    .and_then(|detail| detail.get("kind"))
                    .and_then(Value::as_str)
                    == Some("HostInjectedFailure");
        let let_lambda_value_observed =
            string_value(&w043_treecalc_let_lambda_result, "result_state") == "published"
                && treecalc_result_publishes_value(&w043_treecalc_let_lambda_result, "3", "17");
        let w073_typed_only_guard_present = !bool_at(
            &w043_w073_formatting_intake,
            "threshold_fallback_allowed_for_typed_families",
        ) && w043_w073_formatting_intake
            .get("typed_rule_only_families")
            .and_then(Value::as_array)
            .is_some_and(|families| families.len() == 7);

        let proof_rows = vec![
            json!({
                "row_id": "w043_result_error_carrier_totality_evidence",
                "w043_obligation_id": "W043-OBL-010",
                "source_inputs": ["Rust typed error carriers", W043_LEAN_RUST_TOTALITY_FILE],
                "disposition_kind": "direct_totality_evidence",
                "disposition": "retain promoted core public paths as typed Result/error carrier evidence rather than panic-as-contract",
                "local_checked_proof": true,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": false,
                "refinement_row": false,
                "automatic_dynamic_transition_row": false,
                "exact_remaining_blocker": false,
                "authority_owner": "calc-2p3.3",
                "promotion_consequence": "Rust totality remains unpromoted because this is carrier evidence, not whole-engine proof",
                "reason": "Core execution, fixture, runner, structural, and coordinator surfaces expose Result/typed error APIs for promoted evidence paths.",
                "evidence_paths": [
                    "src/oxcalc-core/src/coordinator.rs",
                    "src/oxcalc-core/src/recalc.rs",
                    "src/oxcalc-core/src/structural.rs",
                    "src/oxcalc-core/src/treecalc.rs",
                    "src/oxcalc-core/src/treecalc_fixture.rs",
                    "src/oxcalc-core/src/treecalc_runner.rs",
                    W043_LEAN_RUST_TOTALITY_FILE
                ],
                "failures": if lean_file_present { Vec::<String>::new() } else { vec!["w043_lean_rust_totality_file_missing".to_string()] },
                "validation_state": if lean_file_present { "w043_rust_totality_row_validated" } else { "w043_rust_totality_row_failed" }
            }),
            json!({
                "row_id": "w043_treecalc_packet_totality_evidence",
                "w043_obligation_id": "W043-OBL-010",
                "source_inputs": ["W043 TreeCalc replay", "W043 optimized/core conformance packet"],
                "disposition_kind": "direct_totality_evidence",
                "disposition": "current W043 replay emits deterministic typed artifacts for dependency addition, dependency release, reject, and LET/LAMBDA value-carrier paths",
                "local_checked_proof": true,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": false,
                "refinement_row": false,
                "automatic_dynamic_transition_row": false,
                "exact_remaining_blocker": false,
                "authority_owner": "calc-2p3.2; calc-2p3.3",
                "promotion_consequence": "the exercised W043 paths are evidenced, while whole-engine Rust totality remains blocked",
                "reason": "The W043 TreeCalc replay emits 27 cases with zero expectation mismatches and the W043.2 conformance packet validates its source artifacts.",
                "evidence_paths": [
                    &w043_conformance_summary_path,
                    &w043_treecalc_summary_path
                ],
                "failures": if string_value(&w043_conformance_summary, "validation_state") == "passed" && counter_value(&w043_treecalc_summary, "expectation_mismatch_count") == 0 { Vec::<String>::new() } else { vec!["w043_treecalc_packet_totality_evidence_missing".to_string()] },
                "validation_state": if string_value(&w043_conformance_summary, "validation_state") == "passed" && counter_value(&w043_treecalc_summary, "expectation_mismatch_count") == 0 { "w043_rust_totality_row_validated" } else { "w043_rust_totality_row_failed" }
            }),
            json!({
                "row_id": "w043_w042_rust_refinement_regression_evidence",
                "w043_obligation_id": "W043-OBL-011",
                "source_inputs": ["W042 Rust totality/refinement packet"],
                "disposition_kind": "direct_refinement_evidence",
                "disposition": "retain W042 Rust refinement classifications as predecessor regression evidence for the W043 frontier",
                "local_checked_proof": true,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": false,
                "refinement_row": true,
                "automatic_dynamic_transition_row": false,
                "exact_remaining_blocker": false,
                "authority_owner": "calc-2p3.3",
                "promotion_consequence": "predecessor refinement evidence is retained but is not a full Rust-engine refinement proof",
                "reason": "The W042 Rust packet remains valid, has zero failed rows, and records the previous automatic dynamic transition refinement row.",
                "evidence_paths": [&w042_rust_summary_path, &w042_rust_validation_path, &w042_rust_refinement_path],
                "failures": if string_value(&w042_rust_validation, "status") == "formal_assurance_w042_rust_totality_refinement_valid" && counter_value(&w042_rust_summary, "failed_row_count") == 0 && row_with_field_exists(&w042_rust_refinement, "row_id", "w042_automatic_dynamic_transition_refinement_evidence") { Vec::<String>::new() } else { vec!["w042_rust_regression_evidence_not_valid".to_string()] },
                "validation_state": if string_value(&w042_rust_validation, "status") == "formal_assurance_w042_rust_totality_refinement_valid" && counter_value(&w042_rust_summary, "failed_row_count") == 0 && row_with_field_exists(&w042_rust_refinement, "row_id", "w042_automatic_dynamic_transition_refinement_evidence") { "w043_rust_refinement_row_validated" } else { "w043_rust_refinement_row_failed" }
            }),
            json!({
                "row_id": "w043_automatic_dynamic_addition_refinement_evidence",
                "w043_obligation_id": "W043-OBL-011",
                "source_inputs": ["W043 automatic dependency addition TreeCalc run"],
                "disposition_kind": "direct_refinement_evidence",
                "disposition": "automatic potential-to-resolved dynamic transition derives DependencyAdded and DependencyReclassified and forces rebind/no-publication behavior in the W043 replay",
                "local_checked_proof": true,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": false,
                "refinement_row": true,
                "automatic_dynamic_transition_row": true,
                "exact_remaining_blocker": false,
                "authority_owner": "calc-2p3.2; calc-2p3.3",
                "promotion_consequence": "dynamic-transition refinement is evidenced for the exercised addition pattern, while broader dynamic coverage remains blocked",
                "reason": "The W043 addition post-edit closure marks node 3 as requiring rebind, the automatic seeds include DependencyAdded and DependencyReclassified, and the post-edit result rejects with HostInjectedFailure before publication.",
                "evidence_paths": [
                    &w043_dynamic_transition_path,
                    &w043_treecalc_add_post_edit_result_path,
                    &w043_treecalc_add_post_edit_closure_path,
                    &w043_treecalc_add_post_edit_seeds_path
                ],
                "observed": {
                    "addition_transition_seeds_present": addition_transition_seeds_present,
                    "addition_transition_closure_requires_rebind": addition_transition_closure_requires_rebind,
                    "addition_transition_rejected_for_rebind": addition_transition_rejected_for_rebind
                },
                "failures": if addition_transition_seeds_present && addition_transition_closure_requires_rebind && addition_transition_rejected_for_rebind { Vec::<String>::new() } else { vec!["w043_automatic_dynamic_addition_refinement_evidence_missing".to_string()] },
                "validation_state": if addition_transition_seeds_present && addition_transition_closure_requires_rebind && addition_transition_rejected_for_rebind { "w043_rust_refinement_row_validated" } else { "w043_rust_refinement_row_failed" }
            }),
            json!({
                "row_id": "w043_automatic_dynamic_release_refinement_evidence",
                "w043_obligation_id": "W043-OBL-011",
                "source_inputs": ["W043 carried automatic dependency release TreeCalc run"],
                "disposition_kind": "direct_refinement_evidence",
                "disposition": "automatic resolved-to-potential dynamic transition derives DependencyRemoved and DependencyReclassified and forces rebind behavior in the W043 replay",
                "local_checked_proof": true,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": false,
                "refinement_row": true,
                "automatic_dynamic_transition_row": true,
                "exact_remaining_blocker": false,
                "authority_owner": "calc-2p3.2; calc-2p3.3",
                "promotion_consequence": "dynamic-transition refinement is evidenced for the exercised release pattern, while broader dynamic coverage remains blocked",
                "reason": "The W043 release post-edit closure marks node 3 as requiring rebind, the automatic seeds include DependencyRemoved and DependencyReclassified, and the post-edit result rejects with HostInjectedFailure.",
                "evidence_paths": [
                    &w043_dynamic_transition_path,
                    &w043_treecalc_release_post_edit_result_path,
                    &w043_treecalc_release_post_edit_closure_path,
                    &w043_treecalc_release_post_edit_seeds_path
                ],
                "observed": {
                    "release_transition_seeds_present": release_transition_seeds_present,
                    "release_transition_closure_requires_rebind": release_transition_closure_requires_rebind,
                    "release_transition_rejected_for_rebind": release_transition_rejected_for_rebind
                },
                "failures": if release_transition_seeds_present && release_transition_closure_requires_rebind && release_transition_rejected_for_rebind { Vec::<String>::new() } else { vec!["w043_automatic_dynamic_release_refinement_evidence_missing".to_string()] },
                "validation_state": if release_transition_seeds_present && release_transition_closure_requires_rebind && release_transition_rejected_for_rebind { "w043_rust_refinement_row_validated" } else { "w043_rust_refinement_row_failed" }
            }),
            json!({
                "row_id": "w043_snapshot_fence_declared_profile_refinement_evidence",
                "w043_obligation_id": "W043-OBL-011",
                "source_inputs": ["W043 optimized/core counterpart conformance register"],
                "disposition_kind": "direct_declared_profile_refinement_evidence",
                "disposition": "bind declared-profile snapshot-fence reject/no-publish counterpart as Rust refinement evidence for exercised declared profiles only",
                "local_checked_proof": true,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": false,
                "refinement_row": true,
                "automatic_dynamic_transition_row": false,
                "exact_remaining_blocker": false,
                "authority_owner": "calc-2p3.2; calc-2p3.3; calc-2p3.5",
                "promotion_consequence": "snapshot-fence refinement remains declared-profile evidence and does not promote Stage 2 policy",
                "reason": "W043.2 carries declared-profile snapshot counterpart evidence without broad Stage 2 or release-grade promotion.",
                "evidence_paths": [&w043_conformance_register_path],
                "failures": if row_with_field_exists(&w043_conformance_register, "row_id", "w043_snapshot_fence_counterpart_declared_profile_evidence") { Vec::<String>::new() } else { vec!["w043_snapshot_fence_declared_profile_row_missing".to_string()] },
                "validation_state": if row_with_field_exists(&w043_conformance_register, "row_id", "w043_snapshot_fence_counterpart_declared_profile_evidence") { "w043_rust_refinement_row_validated" } else { "w043_rust_refinement_row_failed" }
            }),
            json!({
                "row_id": "w043_capability_view_declared_profile_refinement_evidence",
                "w043_obligation_id": "W043-OBL-011",
                "source_inputs": ["W043 optimized/core counterpart conformance register", "capability-sensitive TreeCalc reject"],
                "disposition_kind": "direct_declared_profile_refinement_evidence",
                "disposition": "bind declared-profile capability-view reject/no-publish counterpart as Rust refinement evidence for exercised declared profiles only",
                "local_checked_proof": true,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": false,
                "refinement_row": true,
                "automatic_dynamic_transition_row": false,
                "exact_remaining_blocker": false,
                "authority_owner": "calc-2p3.2; calc-2p3.3; calc-2p3.5",
                "promotion_consequence": "capability-view refinement remains declared-profile evidence and does not promote Stage 2 policy",
                "reason": "W043.2 carries declared-profile capability counterpart evidence and the W043 TreeCalc run still rejects the capability-sensitive fixture through a typed host-injected failure.",
                "evidence_paths": [&w043_conformance_register_path, &w043_treecalc_capability_result_path],
                "failures": if row_with_field_exists(&w043_conformance_register, "row_id", "w043_capability_view_counterpart_declared_profile_evidence") && capability_reject_observed { Vec::<String>::new() } else { vec!["w043_capability_view_declared_profile_row_missing".to_string()] },
                "validation_state": if row_with_field_exists(&w043_conformance_register, "row_id", "w043_capability_view_counterpart_declared_profile_evidence") && capability_reject_observed { "w043_rust_refinement_row_validated" } else { "w043_rust_refinement_row_failed" }
            }),
            json!({
                "row_id": "w043_callable_value_carrier_totality_evidence",
                "w043_obligation_id": "W043-OBL-010",
                "source_inputs": ["W043 TreeCalc LET/LAMBDA value row"],
                "disposition_kind": "direct_callable_value_carrier_totality_evidence",
                "disposition": "bind ordinary LET/LAMBDA value-carrier publication as Rust totality evidence for the current value path while keeping callable metadata separate",
                "local_checked_proof": true,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": false,
                "refinement_row": false,
                "automatic_dynamic_transition_row": false,
                "exact_remaining_blocker": false,
                "authority_owner": "calc-2p3.2; calc-2p3.3; calc-2p3.8",
                "promotion_consequence": "ordinary value-carrier behavior is evidenced; callable carrier sufficiency and metadata projection remain unpromoted",
                "reason": "The W043 TreeCalc LET/LAMBDA fixture publishes value 17 through candidate and publication value carriers.",
                "evidence_paths": [&w043_treecalc_let_lambda_result_path, &w043_callable_projection_path],
                "failures": if let_lambda_value_observed { Vec::<String>::new() } else { vec!["w043_let_lambda_value_carrier_not_observed".to_string()] },
                "validation_state": if let_lambda_value_observed { "w043_rust_totality_row_validated" } else { "w043_rust_totality_row_failed" }
            }),
            json!({
                "row_id": "w043_runtime_panic_surface_totality_boundary",
                "w043_obligation_id": "W043-OBL-009",
                "source_inputs": ["Rust panic-marker audit", W043_LEAN_RUST_TOTALITY_FILE],
                "disposition_kind": "exact_totality_boundary",
                "disposition": "retain a whole-engine panic-free proof blocker while panic/unwrap/expect markers remain in audited Rust surfaces",
                "local_checked_proof": true,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": true,
                "refinement_row": false,
                "automatic_dynamic_transition_row": false,
                "exact_remaining_blocker": true,
                "authority_owner": "calc-2p3.3; calc-2p3.10",
                "promotion_consequence": "Rust-engine totality and panic-free core domain remain unpromoted",
                "reason": "The marker census is a guard, not a semantic proof; observed panic-family markers require review or proof before a panic-free claim.",
                "evidence_paths": W040_RUST_PANIC_AUDIT_FILES,
                "observed": {
                    "panic_marker_count": panic_marker_count,
                    "audited_file_count": W040_RUST_PANIC_AUDIT_FILES.len()
                },
                "failures": Vec::<String>::new(),
                "validation_state": "w043_rust_exact_blocker_validated"
            }),
            json!({
                "row_id": "w043_broader_dynamic_transition_coverage_refinement_boundary",
                "w043_obligation_id": "W043-OBL-011",
                "source_inputs": ["W043 optimized/core exact blocker register"],
                "disposition_kind": "exact_refinement_blocker",
                "disposition": "retain broader automatic dynamic dependency transition coverage as a Rust refinement boundary beyond the exercised addition and release patterns",
                "local_checked_proof": false,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": true,
                "refinement_row": true,
                "automatic_dynamic_transition_row": false,
                "exact_remaining_blocker": true,
                "authority_owner": "calc-2p3.3; calc-2p3.4; calc-2p3.5; calc-2p3.10",
                "promotion_consequence": "broader dynamic dependency transition coverage and full optimized/core verification remain unpromoted",
                "reason": "W043.2 adds dependency-addition evidence and carries dependency-release evidence, but it retains broader transition coverage as an exact blocker.",
                "evidence_paths": [&w043_conformance_blockers_path],
                "failures": if row_with_field_exists(&w043_conformance_blockers, "row_id", "w043_broader_dynamic_transition_coverage_remaining_exact_blocker") { Vec::<String>::new() } else { vec!["w043_broader_dynamic_transition_blocker_missing".to_string()] },
                "validation_state": if row_with_field_exists(&w043_conformance_blockers, "row_id", "w043_broader_dynamic_transition_coverage_remaining_exact_blocker") { "w043_rust_exact_blocker_validated" } else { "w043_rust_exact_blocker_failed" }
            }),
            json!({
                "row_id": "w043_callable_metadata_projection_totality_boundary",
                "w043_obligation_id": "W043-OBL-010",
                "source_inputs": ["W043 callable metadata projection register", "LET/LAMBDA carrier boundary"],
                "disposition_kind": "exact_totality_boundary",
                "disposition": "carry callable metadata projection as an exact totality/refinement blocker while retaining value-carrier evidence separately",
                "local_checked_proof": true,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": true,
                "refinement_row": true,
                "automatic_dynamic_transition_row": false,
                "exact_remaining_blocker": true,
                "authority_owner": "calc-2p3.4; calc-2p3.8; external:OxFunc",
                "promotion_consequence": "callable metadata projection and broad callable conformance remain unpromoted",
                "reason": "The narrow LET/LAMBDA carrier seam is in scope, but general OxFunc kernels and metadata projection sufficiency are not discharged.",
                "evidence_paths": [&w043_callable_projection_path, W043_LEAN_RUST_TOTALITY_FILE],
                "failures": if row_with_field_exists(&w043_callable_projection, "row_id", "w043_callable_metadata_projection_exact_blocker") && lean_file_present { Vec::<String>::new() } else { vec!["w043_callable_metadata_projection_blocker_missing".to_string()] },
                "validation_state": if row_with_field_exists(&w043_callable_projection, "row_id", "w043_callable_metadata_projection_exact_blocker") && lean_file_present { "w043_rust_exact_blocker_validated" } else { "w043_rust_exact_blocker_failed" }
            }),
            json!({
                "row_id": "w043_full_optimized_core_release_grade_conformance_boundary",
                "w043_obligation_id": "W043-OBL-011",
                "source_inputs": ["W043 optimized/core exact blocker register"],
                "disposition_kind": "exact_release_grade_boundary",
                "disposition": "retain full optimized/core release-grade conformance as a boundary over the Rust tranche until later W043 proof, service, diversity, OxFml, Stage 2, and pack gates are discharged",
                "local_checked_proof": false,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": true,
                "refinement_row": true,
                "automatic_dynamic_transition_row": false,
                "exact_remaining_blocker": true,
                "authority_owner": "calc-2p3.10",
                "promotion_consequence": "full optimized/core verification and release-grade verification remain unpromoted",
                "reason": "W043.2 records full optimized/core release-grade conformance as blocked by later W043 lanes.",
                "evidence_paths": [&w043_conformance_blockers_path],
                "failures": if row_with_field_exists(&w043_conformance_blockers, "row_id", "w043_full_optimized_core_release_grade_conformance_exact_blocker") { Vec::<String>::new() } else { vec!["w043_full_optimized_core_boundary_missing".to_string()] },
                "validation_state": if row_with_field_exists(&w043_conformance_blockers, "row_id", "w043_full_optimized_core_release_grade_conformance_exact_blocker") { "w043_rust_exact_blocker_validated" } else { "w043_rust_exact_blocker_failed" }
            }),
            json!({
                "row_id": "w043_let_lambda_carrier_external_boundary",
                "w043_obligation_id": "W043-OBL-015",
                "source_inputs": ["W043 proof-service obligation map", W043_LEAN_RUST_TOTALITY_FILE],
                "disposition_kind": "accepted_external_seam_boundary",
                "disposition": "keep LET/LAMBDA carrier interaction inside OxCalc/OxFml formalization while excluding general OxFunc kernels",
                "local_checked_proof": false,
                "bounded_model": false,
                "accepted_external_seam": true,
                "accepted_boundary": true,
                "totality_boundary": false,
                "refinement_row": false,
                "automatic_dynamic_transition_row": false,
                "exact_remaining_blocker": false,
                "authority_owner": "calc-2p3.3; calc-2p3.4; calc-2p3.8; external:OxFunc",
                "promotion_consequence": "general OxFunc kernels remain unpromoted inside OxCalc",
                "reason": "W043 scope includes the carrier seam but not broad OxFunc semantic kernels.",
                "evidence_paths": [&w043_obligation_map_path, W043_LEAN_RUST_TOTALITY_FILE],
                "failures": if w040_obligation_exists(&w043_obligation_map, "W043-OBL-015") && lean_file_present { Vec::<String>::new() } else { vec!["w043_let_lambda_boundary_missing".to_string()] },
                "validation_state": if w040_obligation_exists(&w043_obligation_map, "W043-OBL-015") && lean_file_present { "w043_rust_boundary_validated" } else { "w043_rust_boundary_failed" }
            }),
            json!({
                "row_id": "w043_spec_evolution_refinement_guard",
                "w043_obligation_id": "W043-OBL-011",
                "source_inputs": ["W043 workset and proof-service obligation map"],
                "disposition_kind": "accepted_spec_evolution_guard",
                "disposition": "preserve formalization as spec evolution and implementation improvement, not a fixed-spec test only",
                "local_checked_proof": true,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": true,
                "totality_boundary": false,
                "refinement_row": false,
                "automatic_dynamic_transition_row": false,
                "exact_remaining_blocker": false,
                "authority_owner": "calc-2p3.3; calc-2p3.10",
                "promotion_consequence": "future proof evidence may correct specs or implementation before promotion",
                "reason": "The W043 charter records spec-evolution hooks for Rust totality and refinement obligations.",
                "evidence_paths": [&w043_obligation_map_path, "docs/worksets/W043_CORE_FORMALIZATION_RELEASE_GRADE_PROOF_AND_OPERATED_SERVICE_INTEGRATION.md"],
                "failures": if w040_obligation_exists(&w043_obligation_map, "W043-OBL-011") { Vec::<String>::new() } else { vec!["w043_refinement_obligation_missing".to_string()] },
                "validation_state": if w040_obligation_exists(&w043_obligation_map, "W043-OBL-011") { "w043_rust_boundary_validated" } else { "w043_rust_boundary_failed" }
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
        let automatic_dynamic_transition_row_count = proof_rows
            .iter()
            .filter(|row| bool_at(row, "automatic_dynamic_transition_row"))
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
        if counter_value(&w043_obligation_summary, "obligation_count") != 36 {
            validation_failures.push("w043_obligation_count_changed".to_string());
        }
        if !array_contains_string(
            w043_obligation_summary
                .get("no_promotion_claims")
                .unwrap_or(&Value::Null),
            "rust_totality_and_refinement",
        ) || !array_contains_string(
            w043_obligation_summary
                .get("no_promotion_claims")
                .unwrap_or(&Value::Null),
            "panic_free_core_domain",
        ) {
            validation_failures.push("w043_rust_no_promotion_guard_missing".to_string());
        }
        if !w040_obligation_exists(&w043_obligation_map, "W043-OBL-009")
            || !w040_obligation_exists(&w043_obligation_map, "W043-OBL-010")
            || !w040_obligation_exists(&w043_obligation_map, "W043-OBL-011")
        {
            validation_failures.push("w043_rust_obligation_rows_missing".to_string());
        }
        if string_value(&w042_rust_validation, "status")
            != "formal_assurance_w042_rust_totality_refinement_valid"
        {
            validation_failures.push("w042_rust_formal_assurance_not_valid".to_string());
        }
        if counter_value(&w042_rust_summary, "failed_row_count") != 0 {
            validation_failures.push("w042_rust_failed_row_count_changed".to_string());
        }
        if string_value(&w043_conformance_summary, "validation_state") != "passed" {
            validation_failures.push("w043_conformance_validation_not_passed".to_string());
        }
        if counter_value(&w043_conformance_summary, "exact_remaining_blocker_count") != 3 {
            validation_failures.push("w043_conformance_exact_blocker_count_changed".to_string());
        }
        if !bool_at(
            &w043_conformance_summary,
            "dynamic_addition_reclassification_evidenced",
        ) || !bool_at(
            &w043_conformance_summary,
            "dynamic_release_reclassification_evidenced",
        ) {
            validation_failures.push("w043_dynamic_transition_evidence_missing".to_string());
        }
        if counter_value(&w043_treecalc_summary, "expectation_mismatch_count") != 0 {
            validation_failures.push("w043_treecalc_expectation_mismatch_present".to_string());
        }
        if counter_value(&w043_treecalc_summary, "case_count") != 27 {
            validation_failures.push("w043_treecalc_case_count_changed".to_string());
        }
        if !lean_file_present || !w042_lean_file_present {
            validation_failures.push("w043_or_w042_lean_rust_totality_file_missing".to_string());
        }
        if panic_marker_count == 0 {
            validation_failures.push("w043_panic_marker_audit_unexpected_zero".to_string());
        }
        if !w073_typed_only_guard_present {
            validation_failures.push("w043_w073_typed_only_guard_missing".to_string());
        }
        if failed_row_count != 0 {
            validation_failures.push("w043_rust_totality_row_failures_present".to_string());
        }
        if blocker_rows.len() != 4 {
            validation_failures.push("w043_expected_four_rust_exact_blockers".to_string());
        }
        if automatic_dynamic_transition_row_count != 2 {
            validation_failures
                .push("w043_expected_two_automatic_dynamic_transition_rows".to_string());
        }

        let source_evidence_index_path =
            format!("{relative_artifact_root}/source_evidence_index.json");
        let rust_ledger_path =
            format!("{relative_artifact_root}/w043_rust_totality_refinement_ledger.json");
        let totality_register_path =
            format!("{relative_artifact_root}/w043_rust_totality_boundary_register.json");
        let refinement_register_path =
            format!("{relative_artifact_root}/w043_rust_refinement_register.json");
        let blocker_register_path =
            format!("{relative_artifact_root}/w043_rust_exact_blocker_register.json");
        let validation_path = format!("{relative_artifact_root}/validation.json");

        write_json(
            &artifact_root.join("source_evidence_index.json"),
            &json!({
                "schema_version": FORMAL_ASSURANCE_SOURCE_INDEX_SCHEMA_V1,
                "run_id": run_id,
                "source_artifacts": {
                    "w043_proof_service_obligation_summary": w043_obligation_summary_path,
                    "w043_proof_service_obligation_map": w043_obligation_map_path,
                    "w042_rust_formal_assurance_summary": w042_rust_summary_path,
                    "w042_rust_formal_assurance_validation": w042_rust_validation_path,
                    "w042_rust_refinement_register": w042_rust_refinement_path,
                    "w043_implementation_conformance_summary": w043_conformance_summary_path,
                    "w043_implementation_conformance_register": w043_conformance_register_path,
                    "w043_implementation_conformance_exact_blockers": w043_conformance_blockers_path,
                    "w043_callable_metadata_projection_register": w043_callable_projection_path,
                    "w043_dynamic_transition_evidence": w043_dynamic_transition_path,
                    "w043_w073_formatting_intake": w043_w073_formatting_intake_path,
                    "w043_treecalc_summary": w043_treecalc_summary_path,
                    "w043_treecalc_addition_post_edit_result": w043_treecalc_add_post_edit_result_path,
                    "w043_treecalc_addition_post_edit_closure": w043_treecalc_add_post_edit_closure_path,
                    "w043_treecalc_addition_post_edit_seeds": w043_treecalc_add_post_edit_seeds_path,
                    "w043_treecalc_release_post_edit_result": w043_treecalc_release_post_edit_result_path,
                    "w043_treecalc_release_post_edit_closure": w043_treecalc_release_post_edit_closure_path,
                    "w043_treecalc_release_post_edit_seeds": w043_treecalc_release_post_edit_seeds_path,
                    "w043_treecalc_capability_result": w043_treecalc_capability_result_path,
                    "w043_treecalc_let_lambda_result": w043_treecalc_let_lambda_result_path,
                    "w043_lean_rust_totality_file": W043_LEAN_RUST_TOTALITY_FILE
                },
                "source_counts": {
                    "w043_obligation_count": counter_value(&w043_obligation_summary, "obligation_count"),
                    "w042_rust_exact_blocker_count": counter_value(&w042_rust_summary, "exact_remaining_blocker_count"),
                    "w043_conformance_exact_blocker_count": counter_value(&w043_conformance_summary, "exact_remaining_blocker_count"),
                    "w043_treecalc_case_count": counter_value(&w043_treecalc_summary, "case_count"),
                    "w043_dynamic_transition_row_count": w043_dynamic_transition
                        .get("dynamic_transition_rows")
                        .and_then(Value::as_array)
                        .map_or(0, Vec::len),
                    "panic_marker_count": panic_marker_count,
                    "w073_typed_only_family_count": w043_w073_formatting_intake
                        .get("typed_rule_only_families")
                        .and_then(Value::as_array)
                        .map_or(0, Vec::len)
                }
            }),
        )?;
        write_json(
            &artifact_root.join("w043_rust_totality_refinement_ledger.json"),
            &json!({
                "schema_version": FORMAL_ASSURANCE_W043_RUST_LEDGER_SCHEMA_V1,
                "run_id": run_id,
                "rust_row_count": proof_rows.len(),
                "local_proof_row_count": local_proof_row_count,
                "bounded_model_row_count": bounded_model_row_count,
                "accepted_external_seam_count": accepted_external_seam_count,
                "accepted_boundary_count": accepted_boundary_count,
                "automatic_dynamic_transition_row_count": automatic_dynamic_transition_row_count,
                "totality_boundary_count": totality_rows.len(),
                "refinement_row_count": refinement_rows.len(),
                "exact_remaining_blocker_count": blocker_rows.len(),
                "rows": proof_rows
            }),
        )?;
        write_json(
            &artifact_root.join("w043_rust_totality_boundary_register.json"),
            &json!({
                "schema_version": FORMAL_ASSURANCE_W043_TOTALITY_REGISTER_SCHEMA_V1,
                "run_id": run_id,
                "totality_boundary_count": totality_rows.len(),
                "rows": totality_rows
            }),
        )?;
        write_json(
            &artifact_root.join("w043_rust_refinement_register.json"),
            &json!({
                "schema_version": FORMAL_ASSURANCE_W043_REFINEMENT_REGISTER_SCHEMA_V1,
                "run_id": run_id,
                "refinement_row_count": refinement_rows.len(),
                "automatic_dynamic_transition_row_count": automatic_dynamic_transition_row_count,
                "rows": refinement_rows
            }),
        )?;
        write_json(
            &artifact_root.join("w043_rust_exact_blocker_register.json"),
            &json!({
                "schema_version": FORMAL_ASSURANCE_W043_BLOCKER_REGISTER_SCHEMA_V1,
                "run_id": run_id,
                "exact_remaining_blocker_count": blocker_rows.len(),
                "rows": blocker_rows
            }),
        )?;

        let validation_status = if validation_failures.is_empty() {
            "formal_assurance_w043_rust_totality_refinement_valid"
        } else {
            "formal_assurance_w043_rust_totality_refinement_invalid"
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
                "automatic_dynamic_transition_row_count": automatic_dynamic_transition_row_count,
                "totality_boundary_count": totality_rows.len(),
                "refinement_row_count": refinement_rows.len(),
                "exact_remaining_blocker_count": blocker_rows.len(),
                "failed_row_count": failed_row_count,
                "addition_transition_seed_reasons_verified": ["DependencyAdded", "DependencyReclassified"],
                "release_transition_seed_reasons_verified": ["DependencyRemoved", "DependencyReclassified"],
                "w073_typed_only_guard_present": w073_typed_only_guard_present,
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
                "automatic_dynamic_transition_row_count": automatic_dynamic_transition_row_count,
                "totality_boundary_count": totality_rows.len(),
                "refinement_row_count": refinement_rows.len(),
                "exact_remaining_blocker_count": blocker_rows.len(),
                "failed_row_count": failed_row_count,
                "panic_marker_count": panic_marker_count,
                "promotion_claims": {
                    "rust_engine_totality_promoted": false,
                    "rust_refinement_promoted": false,
                    "panic_free_core_domain_promoted": false,
                    "full_optimized_core_verification_promoted": false,
                    "full_lean_verification_promoted": false,
                    "full_tla_verification_promoted": false,
                    "stage2_policy_promoted": false,
                    "callable_metadata_projection_promoted": false,
                    "callable_carrier_sufficiency_promoted": false,
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

    fn execute_w042_rust_totality_refinement(
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

        let w042_obligation_summary_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "release-grade-ledger",
            W042_RESIDUAL_LEDGER_RUN_ID,
            "run_summary.json",
        ]);
        let w042_obligation_map_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "release-grade-ledger",
            W042_RESIDUAL_LEDGER_RUN_ID,
            "closure_obligation_map.json",
        ]);
        let w041_rust_summary_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "formal-assurance",
            W041_RUST_FORMAL_ASSURANCE_RUN_ID,
            "run_summary.json",
        ]);
        let w041_rust_validation_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "formal-assurance",
            W041_RUST_FORMAL_ASSURANCE_RUN_ID,
            "validation.json",
        ]);
        let w041_rust_refinement_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "formal-assurance",
            W041_RUST_FORMAL_ASSURANCE_RUN_ID,
            "w041_rust_refinement_register.json",
        ]);
        let w042_conformance_summary_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "implementation-conformance",
            W042_IMPLEMENTATION_CONFORMANCE_RUN_ID,
            "run_summary.json",
        ]);
        let w042_conformance_register_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "implementation-conformance",
            W042_IMPLEMENTATION_CONFORMANCE_RUN_ID,
            "w042_counterpart_conformance_register.json",
        ]);
        let w042_conformance_blockers_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "implementation-conformance",
            W042_IMPLEMENTATION_CONFORMANCE_RUN_ID,
            "w042_exact_remaining_blocker_register.json",
        ]);
        let w042_callable_projection_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "implementation-conformance",
            W042_IMPLEMENTATION_CONFORMANCE_RUN_ID,
            "w042_callable_metadata_projection_register.json",
        ]);
        let w042_w073_formatting_intake_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "implementation-conformance",
            W042_IMPLEMENTATION_CONFORMANCE_RUN_ID,
            "w073_formatting_intake.json",
        ]);
        let w042_treecalc_summary_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "treecalc-local",
            W042_TREECALC_RUN_ID,
            "run_summary.json",
        ]);
        let w042_treecalc_auto_post_edit_result_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "treecalc-local",
            W042_TREECALC_RUN_ID,
            "cases",
            "tc_local_dynamic_release_reclassification_auto_post_edit_001",
            "post_edit",
            "result.json",
        ]);
        let w042_treecalc_auto_post_edit_closure_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "treecalc-local",
            W042_TREECALC_RUN_ID,
            "cases",
            "tc_local_dynamic_release_reclassification_auto_post_edit_001",
            "post_edit",
            "invalidation_closure.json",
        ]);
        let w042_treecalc_auto_post_edit_seeds_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "treecalc-local",
            W042_TREECALC_RUN_ID,
            "cases",
            "tc_local_dynamic_release_reclassification_auto_post_edit_001",
            "post_edit",
            "invalidation_seeds.json",
        ]);
        let w042_treecalc_capability_result_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "treecalc-local",
            W042_TREECALC_RUN_ID,
            "cases",
            "tc_local_capability_sensitive_reject_001",
            "result.json",
        ]);
        let w042_treecalc_let_lambda_result_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "treecalc-local",
            W042_TREECALC_RUN_ID,
            "cases",
            "tc_local_w034_higher_order_let_lambda_publish_001",
            "result.json",
        ]);

        let w042_obligation_summary = read_json(repo_root, &w042_obligation_summary_path)?;
        let w042_obligation_map = read_json(repo_root, &w042_obligation_map_path)?;
        let w041_rust_summary = read_json(repo_root, &w041_rust_summary_path)?;
        let w041_rust_validation = read_json(repo_root, &w041_rust_validation_path)?;
        let w041_rust_refinement = read_json(repo_root, &w041_rust_refinement_path)?;
        let w042_conformance_summary = read_json(repo_root, &w042_conformance_summary_path)?;
        let w042_conformance_register = read_json(repo_root, &w042_conformance_register_path)?;
        let w042_conformance_blockers = read_json(repo_root, &w042_conformance_blockers_path)?;
        let w042_callable_projection = read_json(repo_root, &w042_callable_projection_path)?;
        let w042_treecalc_summary = read_json(repo_root, &w042_treecalc_summary_path)?;
        let w042_treecalc_auto_post_edit_result =
            read_json(repo_root, &w042_treecalc_auto_post_edit_result_path)?;
        let w042_treecalc_auto_post_edit_closure =
            read_json(repo_root, &w042_treecalc_auto_post_edit_closure_path)?;
        let w042_treecalc_auto_post_edit_seeds =
            read_json(repo_root, &w042_treecalc_auto_post_edit_seeds_path)?;
        let w042_treecalc_capability_result =
            read_json(repo_root, &w042_treecalc_capability_result_path)?;
        let w042_treecalc_let_lambda_result =
            read_json(repo_root, &w042_treecalc_let_lambda_result_path)?;

        let lean_file_present = repo_root.join(W042_LEAN_RUST_TOTALITY_FILE).exists();
        let w041_lean_file_present = repo_root.join(W041_LEAN_RUST_TOTALITY_FILE).exists();
        let panic_marker_count = panic_marker_count(repo_root, W040_RUST_PANIC_AUDIT_FILES)?;
        let automatic_transition_seeds_present = w042_treecalc_auto_post_edit_seeds
            .as_array()
            .is_some_and(|seeds| {
                seeds.iter().any(|seed| {
                    seed.get("reason").and_then(Value::as_str) == Some("DependencyRemoved")
                }) && seeds.iter().any(|seed| {
                    seed.get("reason").and_then(Value::as_str) == Some("DependencyReclassified")
                })
            });
        let automatic_transition_closure_requires_rebind = w042_treecalc_auto_post_edit_closure
            .as_array()
            .is_some_and(|rows| {
                rows.iter().any(|row| {
                    row.get("node_id").and_then(Value::as_u64) == Some(3)
                        && bool_at(row, "requires_rebind")
                })
            });
        let automatic_transition_rejected_for_rebind =
            string_value(&w042_treecalc_auto_post_edit_result, "result_state") == "rejected"
                && w042_treecalc_auto_post_edit_result
                    .get("reject_detail")
                    .and_then(|detail| detail.get("kind"))
                    .and_then(Value::as_str)
                    == Some("HostInjectedFailure");
        let capability_reject_observed =
            string_value(&w042_treecalc_capability_result, "result_state") == "rejected"
                && w042_treecalc_capability_result
                    .get("reject_detail")
                    .and_then(|detail| detail.get("kind"))
                    .and_then(Value::as_str)
                    == Some("HostInjectedFailure");
        let let_lambda_value_observed =
            string_value(&w042_treecalc_let_lambda_result, "result_state") == "published"
                && treecalc_result_publishes_value(&w042_treecalc_let_lambda_result, "3", "17");

        let proof_rows = vec![
            json!({
                "row_id": "w042_result_error_carrier_totality_evidence",
                "w042_obligation_id": "W042-OBL-007",
                "source_inputs": ["Rust typed error carriers", W042_LEAN_RUST_TOTALITY_FILE],
                "disposition_kind": "direct_totality_evidence",
                "disposition": "retain promoted core public paths as typed Result/error carrier evidence rather than panic-as-contract",
                "local_checked_proof": true,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": false,
                "refinement_row": false,
                "automatic_dynamic_transition_row": false,
                "exact_remaining_blocker": false,
                "authority_owner": "calc-czd.3",
                "promotion_consequence": "Rust totality remains unpromoted because this is carrier evidence, not whole-engine proof",
                "reason": "Core execution, fixture, runner, structural, and coordinator surfaces expose Result/typed error APIs for promoted evidence paths.",
                "evidence_paths": [
                    "src/oxcalc-core/src/coordinator.rs",
                    "src/oxcalc-core/src/recalc.rs",
                    "src/oxcalc-core/src/structural.rs",
                    "src/oxcalc-core/src/treecalc.rs",
                    "src/oxcalc-core/src/treecalc_fixture.rs",
                    "src/oxcalc-core/src/treecalc_runner.rs",
                    W042_LEAN_RUST_TOTALITY_FILE
                ],
                "failures": if lean_file_present { Vec::<String>::new() } else { vec!["w042_lean_rust_totality_file_missing".to_string()] },
                "validation_state": if lean_file_present { "w042_rust_totality_row_validated" } else { "w042_rust_totality_row_failed" }
            }),
            json!({
                "row_id": "w042_treecalc_counterpart_packet_totality_evidence",
                "w042_obligation_id": "W042-OBL-008",
                "source_inputs": ["W042 TreeCalc replay", "W042 optimized/core conformance packet"],
                "disposition_kind": "direct_totality_evidence",
                "disposition": "current W042 conformance replay emits deterministic typed artifacts for dependency, reject, and LET/LAMBDA value-carrier paths",
                "local_checked_proof": true,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": false,
                "refinement_row": false,
                "automatic_dynamic_transition_row": false,
                "exact_remaining_blocker": false,
                "authority_owner": "calc-czd.2; calc-czd.3",
                "promotion_consequence": "the exercised W042 paths are evidenced, while whole-engine Rust totality remains blocked",
                "reason": "The W042 TreeCalc replay emits 26 cases with zero expectation mismatches and the W042.2 conformance packet validates its source artifacts.",
                "evidence_paths": [
                    &w042_conformance_summary_path,
                    &w042_treecalc_summary_path
                ],
                "failures": if string_value(&w042_conformance_summary, "validation_state") == "passed" && counter_value(&w042_treecalc_summary, "expectation_mismatch_count") == 0 { Vec::<String>::new() } else { vec!["w042_counterpart_totality_evidence_missing".to_string()] },
                "validation_state": if string_value(&w042_conformance_summary, "validation_state") == "passed" && counter_value(&w042_treecalc_summary, "expectation_mismatch_count") == 0 { "w042_rust_totality_row_validated" } else { "w042_rust_totality_row_failed" }
            }),
            json!({
                "row_id": "w042_explicit_dependency_seed_rebind_regression_evidence",
                "w042_obligation_id": "W042-OBL-009",
                "source_inputs": ["W041 Rust totality/refinement packet"],
                "disposition_kind": "direct_refinement_evidence",
                "disposition": "retain explicit DependencyRemoved and DependencyReclassified seed behavior as regression refinement evidence",
                "local_checked_proof": true,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": false,
                "refinement_row": true,
                "automatic_dynamic_transition_row": false,
                "exact_remaining_blocker": false,
                "authority_owner": "calc-czd.3",
                "promotion_consequence": "explicit-seed refinement evidence is retained but is not full dynamic-transition coverage",
                "reason": "The predecessor W041 Rust packet remains valid and recorded zero failed rows.",
                "evidence_paths": [&w041_rust_summary_path, &w041_rust_validation_path],
                "failures": if string_value(&w041_rust_validation, "status") == "formal_assurance_w041_rust_totality_refinement_valid" && counter_value(&w041_rust_summary, "failed_row_count") == 0 { Vec::<String>::new() } else { vec!["w041_rust_regression_evidence_not_valid".to_string()] },
                "validation_state": if string_value(&w041_rust_validation, "status") == "formal_assurance_w041_rust_totality_refinement_valid" && counter_value(&w041_rust_summary, "failed_row_count") == 0 { "w042_rust_refinement_row_validated" } else { "w042_rust_refinement_row_failed" }
            }),
            json!({
                "row_id": "w042_automatic_dynamic_transition_refinement_evidence",
                "w042_obligation_id": "W042-OBL-009",
                "source_inputs": ["W042 dynamic release/reclassification TreeCalc run"],
                "disposition_kind": "direct_refinement_evidence",
                "disposition": "automatic resolved-to-potential dynamic transition derives DependencyRemoved and DependencyReclassified and forces rebind/no-publication behavior in the W042 replay",
                "local_checked_proof": true,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": false,
                "refinement_row": true,
                "automatic_dynamic_transition_row": true,
                "exact_remaining_blocker": false,
                "authority_owner": "calc-czd.2; calc-czd.3",
                "promotion_consequence": "dynamic-transition refinement is evidenced for the exercised pattern, while broader dynamic coverage remains blocked",
                "reason": "The W042 post-edit closure marks node 3 as requiring rebind, the automatic seeds include DependencyRemoved and DependencyReclassified, and the post-edit result rejects with HostInjectedFailure without a new publication.",
                "evidence_paths": [
                    &w042_conformance_register_path,
                    &w042_treecalc_summary_path,
                    &w042_treecalc_auto_post_edit_result_path,
                    &w042_treecalc_auto_post_edit_closure_path,
                    &w042_treecalc_auto_post_edit_seeds_path
                ],
                "observed": {
                    "treecalc_case_count": counter_value(&w042_treecalc_summary, "case_count"),
                    "treecalc_expectation_mismatch_count": counter_value(&w042_treecalc_summary, "expectation_mismatch_count"),
                    "automatic_transition_seeds_present": automatic_transition_seeds_present,
                    "automatic_transition_closure_requires_rebind": automatic_transition_closure_requires_rebind,
                    "automatic_transition_rejected_for_rebind": automatic_transition_rejected_for_rebind
                },
                "failures": if counter_value(&w042_treecalc_summary, "expectation_mismatch_count") == 0 && automatic_transition_seeds_present && automatic_transition_closure_requires_rebind && automatic_transition_rejected_for_rebind { Vec::<String>::new() } else { vec!["w042_automatic_dynamic_transition_refinement_evidence_missing".to_string()] },
                "validation_state": if counter_value(&w042_treecalc_summary, "expectation_mismatch_count") == 0 && automatic_transition_seeds_present && automatic_transition_closure_requires_rebind && automatic_transition_rejected_for_rebind { "w042_rust_refinement_row_validated" } else { "w042_rust_refinement_row_failed" }
            }),
            json!({
                "row_id": "w042_snapshot_fence_declared_profile_refinement_evidence",
                "w042_obligation_id": "W042-OBL-003",
                "source_inputs": ["W042 optimized/core counterpart conformance register"],
                "disposition_kind": "direct_declared_profile_refinement_evidence",
                "disposition": "bind declared-profile snapshot-fence reject/no-publish counterpart as Rust refinement evidence for the exercised declared profiles only",
                "local_checked_proof": true,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": false,
                "refinement_row": true,
                "automatic_dynamic_transition_row": false,
                "exact_remaining_blocker": false,
                "authority_owner": "calc-czd.2; calc-czd.3; calc-czd.5",
                "promotion_consequence": "snapshot-fence counterpart is refinement evidence for declared profiles, not production Stage 2 policy or full optimized/core verification",
                "reason": "W042.2 binds the W041 Stage 2 declared-profile snapshot-fence counterpart row without match promotion.",
                "evidence_paths": [&w042_conformance_register_path],
                "failures": if row_with_field_exists(&w042_conformance_register, "row_id", "w042_snapshot_fence_counterpart_declared_profile_evidence") { Vec::<String>::new() } else { vec!["w042_snapshot_declared_profile_refinement_missing".to_string()] },
                "validation_state": if row_with_field_exists(&w042_conformance_register, "row_id", "w042_snapshot_fence_counterpart_declared_profile_evidence") { "w042_rust_refinement_row_validated" } else { "w042_rust_refinement_row_failed" }
            }),
            json!({
                "row_id": "w042_capability_view_declared_profile_refinement_evidence",
                "w042_obligation_id": "W042-OBL-004",
                "source_inputs": ["W042 optimized/core counterpart conformance register", "W042 capability-sensitive TreeCalc result"],
                "disposition_kind": "direct_declared_profile_refinement_evidence",
                "disposition": "bind declared-profile capability-view reject/no-publish counterpart and current capability-sensitive reject result as Rust refinement evidence for the exercised declared profiles only",
                "local_checked_proof": true,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": false,
                "refinement_row": true,
                "automatic_dynamic_transition_row": false,
                "exact_remaining_blocker": false,
                "authority_owner": "calc-czd.2; calc-czd.3; calc-czd.5",
                "promotion_consequence": "capability-view counterpart is refinement evidence for declared profiles, not broad capability semantics or production policy",
                "reason": "W042.2 binds the W041 Stage 2 declared-profile capability-view row and fresh TreeCalc capability-sensitive rejection evidence without equating the two surfaces.",
                "evidence_paths": [&w042_conformance_register_path, &w042_treecalc_capability_result_path],
                "observed": {
                    "capability_reject_observed": capability_reject_observed
                },
                "failures": if row_with_field_exists(&w042_conformance_register, "row_id", "w042_capability_view_counterpart_declared_profile_evidence") && capability_reject_observed { Vec::<String>::new() } else { vec!["w042_capability_declared_profile_refinement_missing".to_string()] },
                "validation_state": if row_with_field_exists(&w042_conformance_register, "row_id", "w042_capability_view_counterpart_declared_profile_evidence") && capability_reject_observed { "w042_rust_refinement_row_validated" } else { "w042_rust_refinement_row_failed" }
            }),
            json!({
                "row_id": "w042_callable_value_carrier_totality_evidence",
                "w042_obligation_id": "W042-OBL-008",
                "source_inputs": ["W042 TreeCalc LET/LAMBDA value carrier result", "W042 callable value-carrier row"],
                "disposition_kind": "direct_callable_value_carrier_totality_evidence",
                "disposition": "ordinary LET/LAMBDA value-carrier publication is evidenced without treating value publication as callable metadata projection",
                "local_checked_proof": true,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": false,
                "refinement_row": false,
                "automatic_dynamic_transition_row": false,
                "exact_remaining_blocker": false,
                "authority_owner": "calc-czd.2; calc-czd.3; calc-czd.8",
                "promotion_consequence": "callable value-carrier behavior is evidenced; callable metadata projection and carrier sufficiency remain unpromoted",
                "reason": "The W042 LET/LAMBDA TreeCalc case publishes ordinary value 17 while W042.2 explicitly keeps callable metadata projection blocked.",
                "evidence_paths": [&w042_conformance_register_path, &w042_treecalc_let_lambda_result_path],
                "observed": {
                    "let_lambda_value_observed": let_lambda_value_observed
                },
                "failures": if row_with_field_exists(&w042_conformance_register, "row_id", "w042_callable_value_carrier_boundary_evidence") && let_lambda_value_observed { Vec::<String>::new() } else { vec!["w042_callable_value_carrier_totality_missing".to_string()] },
                "validation_state": if row_with_field_exists(&w042_conformance_register, "row_id", "w042_callable_value_carrier_boundary_evidence") && let_lambda_value_observed { "w042_rust_totality_row_validated" } else { "w042_rust_totality_row_failed" }
            }),
            json!({
                "row_id": "w042_runtime_panic_surface_totality_boundary",
                "w042_obligation_id": "W042-OBL-007",
                "source_inputs": ["Rust panic marker audit", W042_LEAN_RUST_TOTALITY_FILE],
                "disposition_kind": "exact_totality_boundary",
                "disposition": "retain a whole-engine panic-free proof blocker while panic/unwrap/expect markers remain in core Rust surfaces",
                "local_checked_proof": true,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": true,
                "refinement_row": false,
                "automatic_dynamic_transition_row": false,
                "exact_remaining_blocker": true,
                "authority_owner": "calc-czd.3; calc-czd.10",
                "promotion_consequence": "Rust-engine totality and panic-free core domain remain unpromoted",
                "reason": "The marker census is a guard, not a semantic proof; observed panic-family markers require review or proof before a panic-free claim.",
                "evidence_paths": W040_RUST_PANIC_AUDIT_FILES,
                "observed": {
                    "panic_marker_count": panic_marker_count,
                    "audited_file_count": W040_RUST_PANIC_AUDIT_FILES.len()
                },
                "failures": Vec::<String>::new(),
                "validation_state": "w042_rust_exact_blocker_validated"
            }),
            json!({
                "row_id": "w042_broader_dynamic_transition_coverage_refinement_boundary",
                "w042_obligation_id": "W042-OBL-002",
                "source_inputs": ["W042 optimized/core exact blocker register"],
                "disposition_kind": "exact_refinement_blocker",
                "disposition": "retain broader automatic dynamic dependency transition coverage as a Rust refinement boundary beyond the exercised resolved-to-potential pattern",
                "local_checked_proof": false,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": true,
                "refinement_row": true,
                "automatic_dynamic_transition_row": false,
                "exact_remaining_blocker": true,
                "authority_owner": "calc-czd.3; calc-czd.4; calc-czd.5",
                "promotion_consequence": "broader dynamic dependency transition coverage and full optimized/core verification remain unpromoted",
                "reason": "W042.2 validates the exercised pattern but retains broader transition coverage as an exact blocker.",
                "evidence_paths": [&w042_conformance_blockers_path],
                "failures": if row_with_field_exists(&w042_conformance_blockers, "row_id", "w042_broader_dynamic_transition_coverage_exact_blocker") { Vec::<String>::new() } else { vec!["w042_broader_dynamic_transition_blocker_missing".to_string()] },
                "validation_state": if row_with_field_exists(&w042_conformance_blockers, "row_id", "w042_broader_dynamic_transition_coverage_exact_blocker") { "w042_rust_exact_blocker_validated" } else { "w042_rust_exact_blocker_failed" }
            }),
            json!({
                "row_id": "w042_callable_metadata_projection_totality_boundary",
                "w042_obligation_id": "W042-OBL-005",
                "source_inputs": ["W042 callable metadata projection register", "LET/LAMBDA carrier boundary"],
                "disposition_kind": "exact_totality_boundary",
                "disposition": "carry callable metadata projection as an exact totality/refinement blocker while retaining value-carrier evidence separately",
                "local_checked_proof": true,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": true,
                "refinement_row": true,
                "automatic_dynamic_transition_row": false,
                "exact_remaining_blocker": true,
                "authority_owner": "calc-czd.4; calc-czd.8; external:OxFunc",
                "promotion_consequence": "callable metadata projection and broad callable conformance remain unpromoted",
                "reason": "The narrow LET/LAMBDA carrier seam is in scope, but general OxFunc kernels and metadata projection sufficiency are not discharged.",
                "evidence_paths": [&w042_callable_projection_path, W042_LEAN_RUST_TOTALITY_FILE],
                "failures": if row_with_field_exists(&w042_callable_projection, "row_id", "w042_callable_metadata_projection_exact_blocker") && lean_file_present { Vec::<String>::new() } else { vec!["w042_callable_metadata_projection_blocker_missing".to_string()] },
                "validation_state": if row_with_field_exists(&w042_callable_projection, "row_id", "w042_callable_metadata_projection_exact_blocker") && lean_file_present { "w042_rust_exact_blocker_validated" } else { "w042_rust_exact_blocker_failed" }
            }),
            json!({
                "row_id": "w042_full_optimized_core_release_grade_conformance_boundary",
                "w042_obligation_id": "W042-OBL-032",
                "source_inputs": ["W042 optimized/core exact blocker register"],
                "disposition_kind": "exact_release_grade_boundary",
                "disposition": "retain full optimized/core release-grade conformance as a boundary over the Rust tranche until later W042 proof, service, diversity, OxFml, Stage 2, and pack gates are discharged",
                "local_checked_proof": false,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": true,
                "refinement_row": true,
                "automatic_dynamic_transition_row": false,
                "exact_remaining_blocker": true,
                "authority_owner": "calc-czd.10",
                "promotion_consequence": "full optimized/core verification and release-grade verification remain unpromoted",
                "reason": "W042.2 records full optimized/core release-grade conformance as blocked by later W042 lanes.",
                "evidence_paths": [&w042_conformance_blockers_path],
                "failures": if row_with_field_exists(&w042_conformance_blockers, "row_id", "w042_full_optimized_core_release_grade_conformance_exact_blocker") { Vec::<String>::new() } else { vec!["w042_full_optimized_core_boundary_missing".to_string()] },
                "validation_state": if row_with_field_exists(&w042_conformance_blockers, "row_id", "w042_full_optimized_core_release_grade_conformance_exact_blocker") { "w042_rust_exact_blocker_validated" } else { "w042_rust_exact_blocker_failed" }
            }),
            json!({
                "row_id": "w042_let_lambda_carrier_external_boundary",
                "w042_obligation_id": "W042-OBL-012",
                "source_inputs": ["W042 closure obligation map", W042_LEAN_RUST_TOTALITY_FILE],
                "disposition_kind": "accepted_external_seam_boundary",
                "disposition": "keep LET/LAMBDA carrier interaction inside OxCalc/OxFml formalization while excluding general OxFunc kernels",
                "local_checked_proof": false,
                "bounded_model": false,
                "accepted_external_seam": true,
                "accepted_boundary": true,
                "totality_boundary": false,
                "refinement_row": false,
                "automatic_dynamic_transition_row": false,
                "exact_remaining_blocker": false,
                "authority_owner": "calc-czd.3; calc-czd.4; calc-czd.8; external:OxFunc",
                "promotion_consequence": "general OxFunc kernels remain unpromoted inside OxCalc",
                "reason": "W042 scope includes the carrier seam but not broad OxFunc semantic kernels.",
                "evidence_paths": [&w042_obligation_map_path, W042_LEAN_RUST_TOTALITY_FILE],
                "failures": if w040_obligation_exists(&w042_obligation_map, "W042-OBL-012") && lean_file_present { Vec::<String>::new() } else { vec!["w042_let_lambda_boundary_missing".to_string()] },
                "validation_state": if w040_obligation_exists(&w042_obligation_map, "W042-OBL-012") && lean_file_present { "w042_rust_boundary_validated" } else { "w042_rust_boundary_failed" }
            }),
            json!({
                "row_id": "w042_spec_evolution_refinement_guard",
                "w042_obligation_id": "W042-OBL-009",
                "source_inputs": ["W042 workset and obligation map"],
                "disposition_kind": "accepted_spec_evolution_guard",
                "disposition": "preserve formalization as spec evolution and implementation improvement, not a fixed-spec test only",
                "local_checked_proof": true,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": true,
                "totality_boundary": false,
                "refinement_row": false,
                "automatic_dynamic_transition_row": false,
                "exact_remaining_blocker": false,
                "authority_owner": "calc-czd.3; calc-czd.10",
                "promotion_consequence": "future proof evidence may correct specs or implementation before promotion",
                "reason": "The W042 charter records spec-evolution hooks for Rust totality and refinement obligations.",
                "evidence_paths": [&w042_obligation_map_path, "docs/worksets/W042_CORE_FORMALIZATION_RELEASE_GRADE_EVIDENCE_CLOSURE_EXPANSION.md"],
                "failures": if w040_obligation_exists(&w042_obligation_map, "W042-OBL-009") { Vec::<String>::new() } else { vec!["w042_refinement_obligation_missing".to_string()] },
                "validation_state": if w040_obligation_exists(&w042_obligation_map, "W042-OBL-009") { "w042_rust_boundary_validated" } else { "w042_rust_boundary_failed" }
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
        let automatic_dynamic_transition_row_count = proof_rows
            .iter()
            .filter(|row| bool_at(row, "automatic_dynamic_transition_row"))
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
        if counter_value(&w042_obligation_summary, "obligation_count") != 33 {
            validation_failures.push("w042_obligation_count_changed".to_string());
        }
        if !array_contains_string(
            w042_obligation_summary
                .get("no_promotion_claims")
                .unwrap_or(&Value::Null),
            "rust_totality_and_refinement",
        ) {
            validation_failures.push("w042_rust_no_promotion_guard_missing".to_string());
        }
        if !w040_obligation_exists(&w042_obligation_map, "W042-OBL-007")
            || !w040_obligation_exists(&w042_obligation_map, "W042-OBL-008")
            || !w040_obligation_exists(&w042_obligation_map, "W042-OBL-009")
        {
            validation_failures.push("w042_rust_obligation_rows_missing".to_string());
        }
        if string_value(&w041_rust_validation, "status")
            != "formal_assurance_w041_rust_totality_refinement_valid"
        {
            validation_failures.push("w041_rust_formal_assurance_not_valid".to_string());
        }
        if counter_value(&w041_rust_summary, "failed_row_count") != 0 {
            validation_failures.push("w041_rust_failed_row_count_changed".to_string());
        }
        if !row_with_field_exists(
            &w041_rust_refinement,
            "row_id",
            "w041_automatic_dynamic_transition_refinement_evidence",
        ) {
            validation_failures.push("w041_dynamic_refinement_row_missing".to_string());
        }
        if string_value(&w042_conformance_summary, "validation_state") != "passed" {
            validation_failures.push("w042_conformance_validation_not_passed".to_string());
        }
        if counter_value(&w042_conformance_summary, "exact_remaining_blocker_count") != 3 {
            validation_failures.push("w042_conformance_exact_blocker_count_changed".to_string());
        }
        if !bool_at(
            &w042_conformance_summary,
            "snapshot_counterpart_evidenced_for_declared_profile",
        ) || !bool_at(
            &w042_conformance_summary,
            "capability_counterpart_evidenced_for_declared_profile",
        ) {
            validation_failures.push("w042_declared_profile_counterparts_missing".to_string());
        }
        if bool_at(
            &w042_conformance_summary,
            "full_optimized_core_verification_promoted",
        ) || bool_at(
            &w042_conformance_summary,
            "callable_metadata_projection_promoted",
        ) {
            validation_failures.push("w042_conformance_promoted_unexpectedly".to_string());
        }
        if counter_value(&w042_treecalc_summary, "expectation_mismatch_count") != 0 {
            validation_failures.push("w042_treecalc_expectation_mismatch_present".to_string());
        }
        if !lean_file_present || !w041_lean_file_present {
            validation_failures.push("w042_or_w041_lean_rust_totality_file_missing".to_string());
        }
        if panic_marker_count == 0 {
            validation_failures.push("w042_panic_marker_audit_unexpected_zero".to_string());
        }
        if failed_row_count != 0 {
            validation_failures.push("w042_rust_totality_row_failures_present".to_string());
        }
        if blocker_rows.len() != 4 {
            validation_failures.push("w042_expected_four_rust_exact_blockers".to_string());
        }
        if automatic_dynamic_transition_row_count != 1 {
            validation_failures
                .push("w042_expected_one_automatic_dynamic_transition_row".to_string());
        }

        let source_evidence_index_path =
            format!("{relative_artifact_root}/source_evidence_index.json");
        let rust_ledger_path =
            format!("{relative_artifact_root}/w042_rust_totality_refinement_ledger.json");
        let totality_register_path =
            format!("{relative_artifact_root}/w042_rust_totality_boundary_register.json");
        let refinement_register_path =
            format!("{relative_artifact_root}/w042_rust_refinement_register.json");
        let blocker_register_path =
            format!("{relative_artifact_root}/w042_rust_exact_blocker_register.json");
        let validation_path = format!("{relative_artifact_root}/validation.json");

        write_json(
            &artifact_root.join("source_evidence_index.json"),
            &json!({
                "schema_version": FORMAL_ASSURANCE_SOURCE_INDEX_SCHEMA_V1,
                "run_id": run_id,
                "source_artifacts": {
                    "w042_closure_obligation_summary": w042_obligation_summary_path,
                    "w042_closure_obligation_map": w042_obligation_map_path,
                    "w041_rust_formal_assurance_summary": w041_rust_summary_path,
                    "w041_rust_formal_assurance_validation": w041_rust_validation_path,
                    "w041_rust_refinement_register": w041_rust_refinement_path,
                    "w042_implementation_conformance_summary": w042_conformance_summary_path,
                    "w042_implementation_conformance_register": w042_conformance_register_path,
                    "w042_implementation_conformance_exact_blockers": w042_conformance_blockers_path,
                    "w042_callable_metadata_projection_register": w042_callable_projection_path,
                    "w042_w073_formatting_intake": w042_w073_formatting_intake_path,
                    "w042_treecalc_summary": w042_treecalc_summary_path,
                    "w042_treecalc_auto_post_edit_result": w042_treecalc_auto_post_edit_result_path,
                    "w042_treecalc_auto_post_edit_closure": w042_treecalc_auto_post_edit_closure_path,
                    "w042_treecalc_auto_post_edit_seeds": w042_treecalc_auto_post_edit_seeds_path,
                    "w042_treecalc_capability_result": w042_treecalc_capability_result_path,
                    "w042_treecalc_let_lambda_result": w042_treecalc_let_lambda_result_path,
                    "w042_lean_rust_totality_file": W042_LEAN_RUST_TOTALITY_FILE
                },
                "source_counts": {
                    "w042_obligation_count": counter_value(&w042_obligation_summary, "obligation_count"),
                    "w041_rust_exact_blocker_count": counter_value(&w041_rust_summary, "exact_remaining_blocker_count"),
                    "w042_conformance_exact_blocker_count": counter_value(&w042_conformance_summary, "exact_remaining_blocker_count"),
                    "w042_treecalc_case_count": counter_value(&w042_treecalc_summary, "case_count"),
                    "panic_marker_count": panic_marker_count
                }
            }),
        )?;
        write_json(
            &artifact_root.join("w042_rust_totality_refinement_ledger.json"),
            &json!({
                "schema_version": FORMAL_ASSURANCE_W042_RUST_LEDGER_SCHEMA_V1,
                "run_id": run_id,
                "rust_row_count": proof_rows.len(),
                "local_proof_row_count": local_proof_row_count,
                "bounded_model_row_count": bounded_model_row_count,
                "accepted_external_seam_count": accepted_external_seam_count,
                "accepted_boundary_count": accepted_boundary_count,
                "automatic_dynamic_transition_row_count": automatic_dynamic_transition_row_count,
                "totality_boundary_count": totality_rows.len(),
                "refinement_row_count": refinement_rows.len(),
                "exact_remaining_blocker_count": blocker_rows.len(),
                "rows": proof_rows
            }),
        )?;
        write_json(
            &artifact_root.join("w042_rust_totality_boundary_register.json"),
            &json!({
                "schema_version": FORMAL_ASSURANCE_W042_TOTALITY_REGISTER_SCHEMA_V1,
                "run_id": run_id,
                "totality_boundary_count": totality_rows.len(),
                "rows": totality_rows
            }),
        )?;
        write_json(
            &artifact_root.join("w042_rust_refinement_register.json"),
            &json!({
                "schema_version": FORMAL_ASSURANCE_W042_REFINEMENT_REGISTER_SCHEMA_V1,
                "run_id": run_id,
                "refinement_row_count": refinement_rows.len(),
                "automatic_dynamic_transition_row_count": automatic_dynamic_transition_row_count,
                "rows": refinement_rows
            }),
        )?;
        write_json(
            &artifact_root.join("w042_rust_exact_blocker_register.json"),
            &json!({
                "schema_version": FORMAL_ASSURANCE_W042_BLOCKER_REGISTER_SCHEMA_V1,
                "run_id": run_id,
                "exact_remaining_blocker_count": blocker_rows.len(),
                "rows": blocker_rows
            }),
        )?;

        let validation_status = if validation_failures.is_empty() {
            "formal_assurance_w042_rust_totality_refinement_valid"
        } else {
            "formal_assurance_w042_rust_totality_refinement_invalid"
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
                "automatic_dynamic_transition_row_count": automatic_dynamic_transition_row_count,
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
                "automatic_dynamic_transition_row_count": automatic_dynamic_transition_row_count,
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
                    "callable_carrier_sufficiency_promoted": false,
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

    fn execute_w041_lean_tla_discharge(
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

        let w041_obligation_summary_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "release-grade-ledger",
            W041_RESIDUAL_LEDGER_RUN_ID,
            "run_summary.json",
        ]);
        let w041_obligation_map_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "release-grade-ledger",
            W041_RESIDUAL_LEDGER_RUN_ID,
            "successor_obligation_map.json",
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
        let w040_lean_tla_summary_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "formal-assurance",
            W040_LEAN_TLA_FORMAL_ASSURANCE_RUN_ID,
            "run_summary.json",
        ]);
        let w040_lean_tla_validation_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "formal-assurance",
            W040_LEAN_TLA_FORMAL_ASSURANCE_RUN_ID,
            "validation.json",
        ]);
        let w040_lean_tla_blockers_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "formal-assurance",
            W040_LEAN_TLA_FORMAL_ASSURANCE_RUN_ID,
            "w040_lean_tla_exact_blocker_register.json",
        ]);
        let w041_rust_summary_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "formal-assurance",
            W041_RUST_FORMAL_ASSURANCE_RUN_ID,
            "run_summary.json",
        ]);
        let w041_rust_validation_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "formal-assurance",
            W041_RUST_FORMAL_ASSURANCE_RUN_ID,
            "validation.json",
        ]);
        let w041_rust_blockers_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "formal-assurance",
            W041_RUST_FORMAL_ASSURANCE_RUN_ID,
            "w041_rust_exact_blocker_register.json",
        ]);
        let w041_rust_refinement_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "formal-assurance",
            W041_RUST_FORMAL_ASSURANCE_RUN_ID,
            "w041_rust_refinement_register.json",
        ]);
        let w040_stage2_summary_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "stage2-replay",
            W040_STAGE2_REPLAY_RUN_ID,
            "run_summary.json",
        ]);
        let w040_stage2_validation_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "stage2-replay",
            W040_STAGE2_REPLAY_RUN_ID,
            "validation.json",
        ]);
        let w040_stage2_gate_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "stage2-replay",
            W040_STAGE2_REPLAY_RUN_ID,
            "w040_stage2_policy_gate_register.json",
        ]);

        let w041_obligation_summary = read_json(repo_root, &w041_obligation_summary_path)?;
        let w041_obligation_map = read_json(repo_root, &w041_obligation_map_path)?;
        let w037_formal_summary = read_json(repo_root, &w037_formal_summary_path)?;
        let w037_formal_validation = read_json(repo_root, &w037_formal_validation_path)?;
        let w037_tla_inventory = read_json(repo_root, &w037_tla_inventory_path)?;
        let w040_lean_tla_summary = read_json(repo_root, &w040_lean_tla_summary_path)?;
        let w040_lean_tla_validation = read_json(repo_root, &w040_lean_tla_validation_path)?;
        let w040_lean_tla_blockers = read_json(repo_root, &w040_lean_tla_blockers_path)?;
        let w041_rust_summary = read_json(repo_root, &w041_rust_summary_path)?;
        let w041_rust_validation = read_json(repo_root, &w041_rust_validation_path)?;
        let w041_rust_blockers = read_json(repo_root, &w041_rust_blockers_path)?;
        let w041_rust_refinement = read_json(repo_root, &w041_rust_refinement_path)?;
        let w040_stage2_summary = read_json(repo_root, &w040_stage2_summary_path)?;
        let w040_stage2_validation = read_json(repo_root, &w040_stage2_validation_path)?;
        let w040_stage2_gate = read_json(repo_root, &w040_stage2_gate_path)?;

        let lean_discharge_file_present = repo_root.join(W041_LEAN_TLA_DISCHARGE_FILE).exists();
        let w040_lean_discharge_file_present =
            repo_root.join(W040_LEAN_TLA_DISCHARGE_FILE).exists();
        let w041_rust_file_present = repo_root.join(W041_LEAN_RUST_TOTALITY_FILE).exists();
        let w040_stage2_policy_file_present = repo_root.join(W040_STAGE2_POLICY_FILE).exists();
        let lean_placeholder_count = lean_placeholder_count(repo_root)?;
        let routine_tla_config_count =
            counter_value(&w037_formal_summary, "tla_routine_config_count");
        let routine_tla_failed_count =
            counter_value(&w037_formal_summary, "tla_failed_config_count");
        let tla_inventory_passed_count = counter_value(&w037_tla_inventory, "passed_config_count");
        let w041_dynamic_refinement_present = row_with_field_exists(
            &w041_rust_refinement,
            "row_id",
            "w041_automatic_dynamic_transition_refinement_evidence",
        );

        let proof_rows = vec![
            json!({
                "row_id": "w041_lean_inventory_checked_no_placeholder_evidence",
                "w041_obligation_id": "W041-OBL-010",
                "source_inputs": ["W037 formal inventory", W041_LEAN_TLA_DISCHARGE_FILE],
                "disposition_kind": "checked_lean_inventory_evidence",
                "disposition": "bind the current Lean inventory and zero-placeholder audit as checked evidence, without promoting full Lean verification",
                "local_checked_proof": true,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": false,
                "exact_remaining_blocker": false,
                "authority_owner": "calc-sui.4",
                "promotion_consequence": "full Lean verification remains unpromoted until all semantic proof boundaries are discharged",
                "reason": "The Lean inventory is typechecked and the local placeholder census is zero, but this remains a classification inventory rather than a whole-engine semantic proof.",
                "evidence_paths": [&w037_formal_summary_path, &w037_formal_validation_path, W041_LEAN_TLA_DISCHARGE_FILE],
                "observed": {
                    "lean_file_count": counter_value(&w037_formal_summary, "lean_file_count"),
                    "lean_placeholder_count": lean_placeholder_count
                },
                "failures": if bool_at(&w037_formal_summary, "all_checked_artifacts_passed") && lean_placeholder_count == 0 && lean_discharge_file_present { Vec::<String>::new() } else { vec!["w041_lean_inventory_or_placeholder_check_failed".to_string()] },
                "validation_state": if bool_at(&w037_formal_summary, "all_checked_artifacts_passed") && lean_placeholder_count == 0 && lean_discharge_file_present { "w041_lean_proof_row_validated" } else { "w041_lean_proof_row_failed" }
            }),
            json!({
                "row_id": "w041_lean_tla_predecessor_bridge",
                "w041_obligation_id": "W041-OBL-010",
                "source_inputs": ["W040 Lean/TLA proof-model packet", W040_LEAN_TLA_DISCHARGE_FILE],
                "disposition_kind": "checked_lean_bridge_evidence",
                "disposition": "bind the W040 Lean/TLA packet as a checked non-promoting predecessor input",
                "local_checked_proof": true,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": false,
                "exact_remaining_blocker": false,
                "authority_owner": "calc-sui.4",
                "promotion_consequence": "full Lean/TLA verification remains unpromoted",
                "reason": "The W040 Lean/TLA packet remains valid and records five exact blockers; W041.4 builds on it without treating it as full verification.",
                "evidence_paths": [&w040_lean_tla_summary_path, &w040_lean_tla_validation_path, W040_LEAN_TLA_DISCHARGE_FILE],
                "failures": if string_value(&w040_lean_tla_validation, "status") == "formal_assurance_w040_lean_tla_discharge_valid" && w040_lean_discharge_file_present { Vec::<String>::new() } else { vec!["w040_lean_tla_predecessor_not_valid".to_string()] },
                "validation_state": if string_value(&w040_lean_tla_validation, "status") == "formal_assurance_w040_lean_tla_discharge_valid" && w040_lean_discharge_file_present { "w041_lean_proof_row_validated" } else { "w041_lean_proof_row_failed" }
            }),
            json!({
                "row_id": "w041_lean_rust_dynamic_refinement_bridge",
                "w041_obligation_id": "W041-OBL-009",
                "source_inputs": ["W041 Rust totality/refinement packet", W041_LEAN_RUST_TOTALITY_FILE],
                "disposition_kind": "checked_lean_refinement_bridge",
                "disposition": "bind the W041 automatic dynamic transition refinement row as a checked Lean/TLA proof input",
                "local_checked_proof": true,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": false,
                "exact_remaining_blocker": false,
                "authority_owner": "calc-sui.3; calc-sui.4",
                "promotion_consequence": "Rust refinement and full optimized/core verification remain unpromoted because retained blockers remain",
                "reason": "W041.3 moved the exercised automatic dynamic transition into direct refinement evidence, but the Rust packet still retains four exact blockers.",
                "evidence_paths": [&w041_rust_summary_path, &w041_rust_validation_path, &w041_rust_refinement_path, W041_LEAN_RUST_TOTALITY_FILE],
                "observed": {
                    "automatic_dynamic_transition_row_count": counter_value(&w041_rust_summary, "automatic_dynamic_transition_row_count"),
                    "w041_rust_exact_blocker_count": counter_value(&w041_rust_summary, "exact_remaining_blocker_count")
                },
                "failures": if string_value(&w041_rust_validation, "status") == "formal_assurance_w041_rust_totality_refinement_valid" && counter_value(&w041_rust_summary, "automatic_dynamic_transition_row_count") == 1 && w041_dynamic_refinement_present && w041_rust_file_present { Vec::<String>::new() } else { vec!["w041_rust_dynamic_refinement_bridge_missing".to_string()] },
                "validation_state": if string_value(&w041_rust_validation, "status") == "formal_assurance_w041_rust_totality_refinement_valid" && counter_value(&w041_rust_summary, "automatic_dynamic_transition_row_count") == 1 && w041_dynamic_refinement_present && w041_rust_file_present { "w041_lean_proof_row_validated" } else { "w041_lean_proof_row_failed" }
            }),
            json!({
                "row_id": "w041_lean_stage2_policy_predicate_carried",
                "w041_obligation_id": "W041-OBL-011",
                "source_inputs": ["W040 Stage 2 Lean predicate and replay packet"],
                "disposition_kind": "checked_lean_policy_predicate",
                "disposition": "carry the checked W040 Stage 2 promotion predicate as a Lean proof input while retaining production-policy blockers",
                "local_checked_proof": true,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": false,
                "exact_remaining_blocker": false,
                "authority_owner": "calc-sui.4; calc-sui.5",
                "promotion_consequence": "Stage 2 production policy remains unpromoted",
                "reason": "The predicate proves no-promotion under current evidence; it is not production partition analyzer soundness or fairness discharge.",
                "evidence_paths": [W040_STAGE2_POLICY_FILE, &w040_stage2_summary_path, &w040_stage2_validation_path],
                "failures": if w040_stage2_policy_file_present && string_value(&w040_stage2_validation, "status") == "w040_stage2_policy_equivalence_valid" && !bool_at(&w040_stage2_summary, "stage2_policy_promoted") { Vec::<String>::new() } else { vec!["w040_stage2_policy_input_missing_or_promoted".to_string()] },
                "validation_state": if w040_stage2_policy_file_present && string_value(&w040_stage2_validation, "status") == "w040_stage2_policy_equivalence_valid" && !bool_at(&w040_stage2_summary, "stage2_policy_promoted") { "w041_lean_proof_row_validated" } else { "w041_lean_proof_row_failed" }
            }),
            json!({
                "row_id": "w041_tla_routine_config_bounded_model_boundary",
                "w041_obligation_id": "W041-OBL-011",
                "source_inputs": ["W037 TLA inventory", "routine TLC config set"],
                "disposition_kind": "bounded_model_with_exact_totality_boundary",
                "disposition": "bind the routine TLC config set as bounded model evidence while retaining unbounded model coverage as exact blocker",
                "local_checked_proof": false,
                "bounded_model": true,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": true,
                "exact_remaining_blocker": true,
                "authority_owner": "calc-sui.4; calc-sui.10",
                "promotion_consequence": "full TLA verification remains unpromoted",
                "reason": "The routine TLC floor has 11 bounded configs with zero recorded failures, but does not cover the unbounded scheduler and partition universe.",
                "evidence_paths": [&w037_tla_inventory_path, &w037_formal_summary_path],
                "observed": {
                    "routine_tla_config_count": routine_tla_config_count,
                    "tla_inventory_passed_count": tla_inventory_passed_count,
                    "routine_tla_failed_count": routine_tla_failed_count
                },
                "failures": if routine_tla_config_count == 11 && tla_inventory_passed_count == 11 && routine_tla_failed_count == 0 { Vec::<String>::new() } else { vec!["w041_tla_routine_config_floor_changed".to_string()] },
                "validation_state": if routine_tla_config_count == 11 && tla_inventory_passed_count == 11 && routine_tla_failed_count == 0 { "w041_tla_model_row_validated" } else { "w041_tla_model_row_failed" }
            }),
            json!({
                "row_id": "w041_tla_stage2_partition_bounded_model_evidence",
                "w041_obligation_id": "W041-OBL-011",
                "source_inputs": ["CoreEngineW036Stage2Partition bounded configs"],
                "disposition_kind": "bounded_stage2_partition_model_evidence",
                "disposition": "bind the W036 Stage 2 partition configs as bounded coverage for scheduler readiness, partition cross-dependency, fence reject, and multi-reader profiles",
                "local_checked_proof": false,
                "bounded_model": true,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": false,
                "exact_remaining_blocker": false,
                "authority_owner": "calc-sui.4; calc-sui.5",
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
                "failures": if routine_tla_config_count >= 11 && routine_tla_failed_count == 0 { Vec::<String>::new() } else { vec!["w041_stage2_partition_tla_floor_changed".to_string()] },
                "validation_state": if routine_tla_config_count >= 11 && routine_tla_failed_count == 0 { "w041_tla_model_row_validated" } else { "w041_tla_model_row_failed" }
            }),
            json!({
                "row_id": "w041_stage2_equivalence_bounded_model_input",
                "w041_obligation_id": "W041-OBL-011",
                "source_inputs": ["W040 Stage 2 policy/equivalence packet"],
                "disposition_kind": "bounded_stage2_equivalence_model_evidence",
                "disposition": "bind W040 bounded partition replay, permutation, observable-invariance, and analyzer evidence as model input without promoting Stage 2",
                "local_checked_proof": false,
                "bounded_model": true,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": false,
                "exact_remaining_blocker": false,
                "authority_owner": "calc-sui.4; calc-sui.5",
                "promotion_consequence": "Stage 2 production policy and full TLA verification remain unpromoted",
                "reason": "W040 has bounded profile evidence and fence counterparts, but production analyzer soundness, fairness, operated differential service, and pack-grade governance remain absent.",
                "evidence_paths": [&w040_stage2_summary_path, &w040_stage2_validation_path, &w040_stage2_gate_path, W040_STAGE2_POLICY_FILE],
                "observed": {
                    "partition_replay_row_count": counter_value(&w040_stage2_summary, "partition_replay_row_count"),
                    "permutation_replay_row_count": counter_value(&w040_stage2_summary, "permutation_replay_row_count"),
                    "observable_invariance_row_count": counter_value(&w040_stage2_summary, "observable_invariance_row_count"),
                    "bounded_partition_analyzer_evidenced": bool_at(&w040_stage2_summary, "bounded_partition_analyzer_evidenced"),
                    "stage2_policy_promoted": bool_at(&w040_stage2_summary, "stage2_policy_promoted")
                },
                "failures": if string_value(&w040_stage2_validation, "status") == "w040_stage2_policy_equivalence_valid" && counter_value(&w040_stage2_summary, "partition_replay_row_count") == 5 && counter_value(&w040_stage2_summary, "permutation_replay_row_count") == 6 && counter_value(&w040_stage2_summary, "observable_invariance_row_count") == 5 && bool_at(&w040_stage2_summary, "bounded_partition_analyzer_evidenced") && !bool_at(&w040_stage2_summary, "stage2_policy_promoted") { Vec::<String>::new() } else { vec!["w040_stage2_equivalence_input_changed".to_string()] },
                "validation_state": if string_value(&w040_stage2_validation, "status") == "w040_stage2_policy_equivalence_valid" && counter_value(&w040_stage2_summary, "partition_replay_row_count") == 5 && counter_value(&w040_stage2_summary, "permutation_replay_row_count") == 6 && counter_value(&w040_stage2_summary, "observable_invariance_row_count") == 5 && bool_at(&w040_stage2_summary, "bounded_partition_analyzer_evidenced") && !bool_at(&w040_stage2_summary, "stage2_policy_promoted") { "w041_tla_model_row_validated" } else { "w041_tla_model_row_failed" }
            }),
            json!({
                "row_id": "w041_tla_fairness_scheduler_assumption_boundary",
                "w041_obligation_id": "W041-OBL-011",
                "source_inputs": ["TLA model bounds and scheduler assumptions"],
                "disposition_kind": "exact_model_assumption_boundary",
                "disposition": "retain scheduler fairness and unbounded interleaving assumptions as explicit model boundaries",
                "local_checked_proof": false,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": true,
                "exact_remaining_blocker": true,
                "authority_owner": "calc-sui.4; calc-sui.5; calc-sui.10",
                "promotion_consequence": "full TLA verification and Stage 2 policy remain unpromoted",
                "reason": "Current TLC configs and bounded Stage 2 evidence do not discharge fairness or unbounded scheduler coverage for promoted profiles.",
                "evidence_paths": [&w041_obligation_map_path, &w037_tla_inventory_path, &w040_stage2_gate_path],
                "failures": if w040_obligation_exists(&w041_obligation_map, "W041-OBL-011") { Vec::<String>::new() } else { vec!["w041_tla_obligation_missing".to_string()] },
                "validation_state": if w040_obligation_exists(&w041_obligation_map, "W041-OBL-011") { "w041_lean_tla_exact_blocker_validated" } else { "w041_lean_tla_exact_blocker_failed" }
            }),
            json!({
                "row_id": "w041_full_lean_verification_exact_blocker",
                "w041_obligation_id": "W041-OBL-010",
                "source_inputs": ["W041 successor obligation map", "Lean proof inventory"],
                "disposition_kind": "exact_lean_verification_blocker",
                "disposition": "retain full Lean verification as exact blocker despite checked local proof rows",
                "local_checked_proof": true,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": true,
                "exact_remaining_blocker": true,
                "authority_owner": "calc-sui.4; calc-sui.10",
                "promotion_consequence": "full Lean verification remains unpromoted",
                "reason": "Checked classification files do not prove every Rust, OxFml, TLA, and coordinator semantic path for the claimed scope.",
                "evidence_paths": [&w041_obligation_map_path, W041_LEAN_TLA_DISCHARGE_FILE],
                "failures": if w040_obligation_exists(&w041_obligation_map, "W041-OBL-010") && lean_discharge_file_present { Vec::<String>::new() } else { vec!["w041_full_lean_blocker_input_missing".to_string()] },
                "validation_state": if w040_obligation_exists(&w041_obligation_map, "W041-OBL-010") && lean_discharge_file_present { "w041_lean_tla_exact_blocker_validated" } else { "w041_lean_tla_exact_blocker_failed" }
            }),
            json!({
                "row_id": "w041_full_tla_verification_exact_blocker",
                "w041_obligation_id": "W041-OBL-011",
                "source_inputs": ["W041 successor obligation map", "bounded TLC evidence"],
                "disposition_kind": "exact_tla_verification_blocker",
                "disposition": "retain full TLA verification as exact blocker because current coverage is bounded",
                "local_checked_proof": false,
                "bounded_model": true,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": true,
                "exact_remaining_blocker": true,
                "authority_owner": "calc-sui.4; calc-sui.10",
                "promotion_consequence": "full TLA verification remains unpromoted",
                "reason": "Bounded TLC runs do not discharge unbounded model completeness, fairness, or production partition analyzer soundness.",
                "evidence_paths": [&w041_obligation_map_path, &w037_tla_inventory_path],
                "failures": if w040_obligation_exists(&w041_obligation_map, "W041-OBL-011") && routine_tla_config_count == 11 { Vec::<String>::new() } else { vec!["w041_full_tla_blocker_input_missing".to_string()] },
                "validation_state": if w040_obligation_exists(&w041_obligation_map, "W041-OBL-011") && routine_tla_config_count == 11 { "w041_lean_tla_exact_blocker_validated" } else { "w041_lean_tla_exact_blocker_failed" }
            }),
            json!({
                "row_id": "w041_rust_totality_dependency_exact_blocker",
                "w041_obligation_id": "W041-OBL-009",
                "source_inputs": ["W041 Rust totality/refinement blockers"],
                "disposition_kind": "exact_rust_dependency_blocker",
                "disposition": "retain Rust totality/refinement dependency as a Lean/TLA proof blocker",
                "local_checked_proof": true,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": true,
                "exact_remaining_blocker": true,
                "authority_owner": "calc-sui.4; calc-sui.10",
                "promotion_consequence": "full Lean/TLA verification remains unpromoted",
                "reason": "The preceding W041 Rust packet intentionally retains four exact blockers, so Lean/TLA full verification cannot be promoted over them.",
                "evidence_paths": [&w041_rust_summary_path, &w041_rust_blockers_path],
                "failures": if counter_value(&w041_rust_summary, "exact_remaining_blocker_count") == 4 && counter_value(&w041_rust_blockers, "exact_remaining_blocker_count") == 4 { Vec::<String>::new() } else { vec!["w041_rust_blocker_count_changed".to_string()] },
                "validation_state": if counter_value(&w041_rust_summary, "exact_remaining_blocker_count") == 4 && counter_value(&w041_rust_blockers, "exact_remaining_blocker_count") == 4 { "w041_lean_tla_exact_blocker_validated" } else { "w041_lean_tla_exact_blocker_failed" }
            }),
            json!({
                "row_id": "w041_let_lambda_external_oxfunc_boundary",
                "w041_obligation_id": "W041-OBL-028",
                "source_inputs": ["W041 successor obligation map"],
                "disposition_kind": "accepted_external_seam_boundary",
                "disposition": "keep LET/LAMBDA carrier interaction inside OxCalc/OxFml formalization while excluding general OxFunc kernels",
                "local_checked_proof": false,
                "bounded_model": false,
                "accepted_external_seam": true,
                "accepted_boundary": true,
                "totality_boundary": false,
                "exact_remaining_blocker": false,
                "authority_owner": "calc-sui.4; calc-sui.8; external:OxFunc",
                "promotion_consequence": "general OxFunc kernels remain unpromoted inside OxCalc",
                "reason": "W041 scope includes the carrier seam but not broad OxFunc semantic kernels.",
                "evidence_paths": [&w041_obligation_map_path],
                "failures": if w040_obligation_exists(&w041_obligation_map, "W041-OBL-028") { Vec::<String>::new() } else { vec!["w041_let_lambda_boundary_missing".to_string()] },
                "validation_state": if w040_obligation_exists(&w041_obligation_map, "W041-OBL-028") { "w041_lean_tla_boundary_validated" } else { "w041_lean_tla_boundary_failed" }
            }),
            json!({
                "row_id": "w041_formal_model_spec_evolution_guard",
                "w041_obligation_id": "W041-OBL-010",
                "source_inputs": ["W041 successor obligation map", "W041 workset"],
                "disposition_kind": "accepted_spec_evolution_guard",
                "disposition": "preserve Lean/TLA formalization as spec-evolution and implementation-improvement work",
                "local_checked_proof": true,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": true,
                "totality_boundary": false,
                "exact_remaining_blocker": false,
                "authority_owner": "calc-sui.4; calc-sui.10",
                "promotion_consequence": "future proof/model evidence may correct specs or implementation before promotion",
                "reason": "W041 explicitly allows proof/model evidence to evolve the specs rather than testing against a fixed initial document set.",
                "evidence_paths": [&w041_obligation_map_path, "docs/worksets/W041_CORE_FORMALIZATION_RELEASE_GRADE_SUCCESSOR_VERIFICATION.md"],
                "failures": if w040_obligation_exists(&w041_obligation_map, "W041-OBL-010") { Vec::<String>::new() } else { vec!["w041_lean_obligation_missing".to_string()] },
                "validation_state": if w040_obligation_exists(&w041_obligation_map, "W041-OBL-010") { "w041_lean_tla_boundary_validated" } else { "w041_lean_tla_boundary_failed" }
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
        if counter_value(&w041_obligation_summary, "obligation_count") != 28 {
            validation_failures.push("w041_obligation_count_changed".to_string());
        }
        if !array_contains_string(
            w041_obligation_summary
                .get("no_promotion_claims")
                .unwrap_or(&Value::Null),
            "full_lean_tla_verification",
        ) {
            validation_failures.push("w041_lean_tla_no_promotion_guard_missing".to_string());
        }
        if !w040_obligation_exists(&w041_obligation_map, "W041-OBL-010")
            || !w040_obligation_exists(&w041_obligation_map, "W041-OBL-011")
        {
            validation_failures.push("w041_lean_tla_obligation_rows_missing".to_string());
        }
        if !bool_at(&w037_formal_summary, "all_checked_artifacts_passed") {
            validation_failures.push("w037_formal_artifacts_not_all_checked".to_string());
        }
        if string_value(&w037_formal_validation, "validation_state")
            != "w037_proof_model_closure_inventory_validated"
        {
            validation_failures.push("w037_formal_validation_not_valid".to_string());
        }
        if string_value(&w040_lean_tla_validation, "status")
            != "formal_assurance_w040_lean_tla_discharge_valid"
        {
            validation_failures.push("w040_lean_tla_validation_not_valid".to_string());
        }
        if counter_value(&w040_lean_tla_blockers, "exact_remaining_blocker_count") != 5 {
            validation_failures.push("w040_lean_tla_blocker_count_changed".to_string());
        }
        if string_value(&w041_rust_validation, "status")
            != "formal_assurance_w041_rust_totality_refinement_valid"
        {
            validation_failures.push("w041_rust_validation_not_valid".to_string());
        }
        if counter_value(&w041_rust_blockers, "exact_remaining_blocker_count") != 4 {
            validation_failures.push("w041_rust_blocker_count_changed".to_string());
        }
        if string_value(&w040_stage2_validation, "status") != "w040_stage2_policy_equivalence_valid"
        {
            validation_failures.push("w040_stage2_validation_not_valid".to_string());
        }
        if bool_at(&w040_stage2_summary, "stage2_policy_promoted")
            || bool_at(&w040_stage2_gate, "stage2_policy_promoted")
        {
            validation_failures.push("w040_stage2_policy_was_promoted".to_string());
        }
        if lean_placeholder_count != 0 {
            validation_failures.push("w041_lean_placeholder_count_nonzero".to_string());
        }
        if routine_tla_config_count != 11
            || tla_inventory_passed_count != 11
            || routine_tla_failed_count != 0
        {
            validation_failures.push("w041_tla_routine_floor_changed".to_string());
        }
        if !lean_discharge_file_present {
            validation_failures.push("w041_lean_tla_discharge_file_missing".to_string());
        }
        if failed_row_count != 0 {
            validation_failures.push("w041_lean_tla_row_failures_present".to_string());
        }
        if proof_rows.len() != 13 {
            validation_failures.push("w041_expected_thirteen_lean_tla_rows".to_string());
        }
        if blocker_rows.len() != 5 {
            validation_failures.push("w041_expected_five_lean_tla_exact_blockers".to_string());
        }

        let source_evidence_index_path =
            format!("{relative_artifact_root}/source_evidence_index.json");
        let ledger_path = format!("{relative_artifact_root}/w041_lean_tla_discharge_ledger.json");
        let lean_register_path = format!("{relative_artifact_root}/w041_lean_proof_register.json");
        let model_register_path =
            format!("{relative_artifact_root}/w041_tla_model_bound_register.json");
        let blocker_register_path =
            format!("{relative_artifact_root}/w041_lean_tla_exact_blocker_register.json");
        let validation_path = format!("{relative_artifact_root}/validation.json");

        write_json(
            &artifact_root.join("source_evidence_index.json"),
            &json!({
                "schema_version": FORMAL_ASSURANCE_SOURCE_INDEX_SCHEMA_V1,
                "run_id": run_id,
                "source_artifacts": {
                    "w041_successor_obligation_summary": w041_obligation_summary_path,
                    "w041_successor_obligation_map": w041_obligation_map_path,
                    "w037_formal_inventory_summary": w037_formal_summary_path,
                    "w037_formal_inventory_validation": w037_formal_validation_path,
                    "w037_tla_inventory": w037_tla_inventory_path,
                    "w040_lean_tla_summary": w040_lean_tla_summary_path,
                    "w040_lean_tla_validation": w040_lean_tla_validation_path,
                    "w040_lean_tla_exact_blockers": w040_lean_tla_blockers_path,
                    "w041_rust_formal_assurance_summary": w041_rust_summary_path,
                    "w041_rust_formal_assurance_validation": w041_rust_validation_path,
                    "w041_rust_exact_blockers": w041_rust_blockers_path,
                    "w041_rust_refinement_register": w041_rust_refinement_path,
                    "w040_stage2_summary": w040_stage2_summary_path,
                    "w040_stage2_validation": w040_stage2_validation_path,
                    "w040_stage2_policy_gate": w040_stage2_gate_path,
                    "w041_lean_tla_discharge_file": W041_LEAN_TLA_DISCHARGE_FILE,
                    "w041_rust_lean_file": W041_LEAN_RUST_TOTALITY_FILE,
                    "w040_stage2_policy_file": W040_STAGE2_POLICY_FILE
                },
                "source_counts": {
                    "w041_obligation_count": counter_value(&w041_obligation_summary, "obligation_count"),
                    "w037_lean_file_count": counter_value(&w037_formal_summary, "lean_file_count"),
                    "w037_tla_routine_config_count": routine_tla_config_count,
                    "w037_tla_inventory_passed_count": tla_inventory_passed_count,
                    "w037_tla_failed_config_count": routine_tla_failed_count,
                    "w040_lean_tla_exact_blocker_count": counter_value(&w040_lean_tla_summary, "exact_remaining_blocker_count"),
                    "w041_rust_exact_blocker_count": counter_value(&w041_rust_summary, "exact_remaining_blocker_count"),
                    "w041_dynamic_refinement_row_count": counter_value(&w041_rust_summary, "automatic_dynamic_transition_row_count"),
                    "w040_stage2_exact_blocker_count": counter_value(&w040_stage2_summary, "exact_remaining_blocker_count"),
                    "lean_placeholder_count": lean_placeholder_count
                }
            }),
        )?;
        write_json(
            &artifact_root.join("w041_lean_tla_discharge_ledger.json"),
            &json!({
                "schema_version": FORMAL_ASSURANCE_W041_LEAN_TLA_LEDGER_SCHEMA_V1,
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
            &artifact_root.join("w041_lean_proof_register.json"),
            &json!({
                "schema_version": FORMAL_ASSURANCE_W041_LEAN_PROOF_REGISTER_SCHEMA_V1,
                "run_id": run_id,
                "local_proof_row_count": lean_proof_rows.len(),
                "rows": lean_proof_rows
            }),
        )?;
        write_json(
            &artifact_root.join("w041_tla_model_bound_register.json"),
            &json!({
                "schema_version": FORMAL_ASSURANCE_W041_TLA_MODEL_REGISTER_SCHEMA_V1,
                "run_id": run_id,
                "bounded_model_row_count": model_bound_rows.len(),
                "rows": model_bound_rows
            }),
        )?;
        write_json(
            &artifact_root.join("w041_lean_tla_exact_blocker_register.json"),
            &json!({
                "schema_version": FORMAL_ASSURANCE_W041_LEAN_TLA_BLOCKER_REGISTER_SCHEMA_V1,
                "run_id": run_id,
                "exact_remaining_blocker_count": blocker_rows.len(),
                "rows": blocker_rows
            }),
        )?;

        let validation_status = if validation_failures.is_empty() {
            "formal_assurance_w041_lean_tla_discharge_valid"
        } else {
            "formal_assurance_w041_lean_tla_discharge_invalid"
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
                    "rust_refinement_promoted": false,
                    "full_optimized_core_verification_promoted": false,
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

    fn execute_w042_lean_tla_fairness_expansion(
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

        let w042_obligation_summary_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "release-grade-ledger",
            W042_RESIDUAL_LEDGER_RUN_ID,
            "run_summary.json",
        ]);
        let w042_obligation_map_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "release-grade-ledger",
            W042_RESIDUAL_LEDGER_RUN_ID,
            "closure_obligation_map.json",
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
        let w041_lean_tla_summary_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "formal-assurance",
            W041_LEAN_TLA_FORMAL_ASSURANCE_RUN_ID,
            "run_summary.json",
        ]);
        let w041_lean_tla_validation_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "formal-assurance",
            W041_LEAN_TLA_FORMAL_ASSURANCE_RUN_ID,
            "validation.json",
        ]);
        let w041_lean_tla_blockers_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "formal-assurance",
            W041_LEAN_TLA_FORMAL_ASSURANCE_RUN_ID,
            "w041_lean_tla_exact_blocker_register.json",
        ]);
        let w042_rust_summary_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "formal-assurance",
            W042_RUST_FORMAL_ASSURANCE_RUN_ID,
            "run_summary.json",
        ]);
        let w042_rust_validation_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "formal-assurance",
            W042_RUST_FORMAL_ASSURANCE_RUN_ID,
            "validation.json",
        ]);
        let w042_rust_blockers_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "formal-assurance",
            W042_RUST_FORMAL_ASSURANCE_RUN_ID,
            "w042_rust_exact_blocker_register.json",
        ]);
        let w042_rust_refinement_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "formal-assurance",
            W042_RUST_FORMAL_ASSURANCE_RUN_ID,
            "w042_rust_refinement_register.json",
        ]);
        let w042_rust_ledger_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "formal-assurance",
            W042_RUST_FORMAL_ASSURANCE_RUN_ID,
            "w042_rust_totality_refinement_ledger.json",
        ]);
        let w041_stage2_summary_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "stage2-replay",
            W041_STAGE2_REPLAY_RUN_ID,
            "run_summary.json",
        ]);
        let w041_stage2_validation_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "stage2-replay",
            W041_STAGE2_REPLAY_RUN_ID,
            "validation.json",
        ]);
        let w041_stage2_gate_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "stage2-replay",
            W041_STAGE2_REPLAY_RUN_ID,
            "w041_stage2_policy_gate_register.json",
        ]);
        let w041_stage2_blockers_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "stage2-replay",
            W041_STAGE2_REPLAY_RUN_ID,
            "w041_stage2_exact_blocker_register.json",
        ]);
        let w042_w073_formatting_intake_path = relative_artifact_path(&[
            "docs",
            "test-runs",
            "core-engine",
            "implementation-conformance",
            W042_IMPLEMENTATION_CONFORMANCE_RUN_ID,
            "w073_formatting_intake.json",
        ]);

        let w042_obligation_summary = read_json(repo_root, &w042_obligation_summary_path)?;
        let w042_obligation_map = read_json(repo_root, &w042_obligation_map_path)?;
        let w037_formal_summary = read_json(repo_root, &w037_formal_summary_path)?;
        let w037_formal_validation = read_json(repo_root, &w037_formal_validation_path)?;
        let w037_tla_inventory = read_json(repo_root, &w037_tla_inventory_path)?;
        let w041_lean_tla_summary = read_json(repo_root, &w041_lean_tla_summary_path)?;
        let w041_lean_tla_validation = read_json(repo_root, &w041_lean_tla_validation_path)?;
        let w041_lean_tla_blockers = read_json(repo_root, &w041_lean_tla_blockers_path)?;
        let w042_rust_summary = read_json(repo_root, &w042_rust_summary_path)?;
        let w042_rust_validation = read_json(repo_root, &w042_rust_validation_path)?;
        let w042_rust_blockers = read_json(repo_root, &w042_rust_blockers_path)?;
        let w042_rust_refinement = read_json(repo_root, &w042_rust_refinement_path)?;
        let w042_rust_ledger = read_json(repo_root, &w042_rust_ledger_path)?;
        let w041_stage2_summary = read_json(repo_root, &w041_stage2_summary_path)?;
        let w041_stage2_validation = read_json(repo_root, &w041_stage2_validation_path)?;
        let w041_stage2_gate = read_json(repo_root, &w041_stage2_gate_path)?;
        let w041_stage2_blockers = read_json(repo_root, &w041_stage2_blockers_path)?;
        let w042_w073_formatting_intake = read_json(repo_root, &w042_w073_formatting_intake_path)?;

        let lean_discharge_file_present = repo_root.join(W042_LEAN_TLA_DISCHARGE_FILE).exists();
        let w041_lean_discharge_file_present =
            repo_root.join(W041_LEAN_TLA_DISCHARGE_FILE).exists();
        let w042_rust_file_present = repo_root.join(W042_LEAN_RUST_TOTALITY_FILE).exists();
        let w041_stage2_policy_file_present = repo_root.join(W041_STAGE2_POLICY_FILE).exists();
        let lean_placeholder_count = lean_placeholder_count(repo_root)?;
        let routine_tla_config_count =
            counter_value(&w037_formal_summary, "tla_routine_config_count");
        let routine_tla_failed_count =
            counter_value(&w037_formal_summary, "tla_failed_config_count");
        let tla_inventory_passed_count = counter_value(&w037_tla_inventory, "passed_config_count");
        let w042_dynamic_refinement_present = row_with_field_exists(
            &w042_rust_refinement,
            "row_id",
            "w042_automatic_dynamic_transition_refinement_evidence",
        );
        let w042_callable_value_carrier_present = row_with_field_exists(
            &w042_rust_ledger,
            "row_id",
            "w042_callable_value_carrier_totality_evidence",
        );
        let w041_fairness_stage2_blocker_present = row_with_field_exists(
            &w041_stage2_blockers,
            "row_id",
            "w041_stage2_fairness_scheduler_unbounded_coverage_blocker",
        );

        let proof_rows = vec![
            json!({
                "row_id": "w042_lean_inventory_checked_no_placeholder_evidence",
                "w042_obligation_id": "W042-OBL-010",
                "source_inputs": ["W037 formal inventory", W042_LEAN_TLA_DISCHARGE_FILE],
                "disposition_kind": "checked_lean_inventory_evidence",
                "disposition": "bind the current Lean inventory and zero-placeholder audit as checked W042 evidence without promoting full Lean verification",
                "local_checked_proof": true,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": false,
                "exact_remaining_blocker": false,
                "authority_owner": "calc-czd.4",
                "promotion_consequence": "full Lean verification remains unpromoted until all semantic proof boundaries are discharged",
                "reason": "The Lean inventory is typechecked and the placeholder census is zero, but this remains classification evidence rather than whole-engine semantic proof.",
                "evidence_paths": [&w037_formal_summary_path, &w037_formal_validation_path, W042_LEAN_TLA_DISCHARGE_FILE],
                "observed": {
                    "lean_file_count": counter_value(&w037_formal_summary, "lean_file_count"),
                    "lean_placeholder_count": lean_placeholder_count
                },
                "failures": if bool_at(&w037_formal_summary, "all_checked_artifacts_passed") && lean_placeholder_count == 0 && lean_discharge_file_present { Vec::<String>::new() } else { vec!["w042_lean_inventory_or_placeholder_check_failed".to_string()] },
                "validation_state": if bool_at(&w037_formal_summary, "all_checked_artifacts_passed") && lean_placeholder_count == 0 && lean_discharge_file_present { "w042_lean_proof_row_validated" } else { "w042_lean_proof_row_failed" }
            }),
            json!({
                "row_id": "w042_lean_tla_predecessor_bridge",
                "w042_obligation_id": "W042-OBL-010",
                "source_inputs": ["W041 Lean/TLA proof-model packet", W041_LEAN_TLA_DISCHARGE_FILE],
                "disposition_kind": "checked_lean_bridge_evidence",
                "disposition": "bind the W041 Lean/TLA packet as a checked non-promoting predecessor input",
                "local_checked_proof": true,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": false,
                "exact_remaining_blocker": false,
                "authority_owner": "calc-czd.4",
                "promotion_consequence": "full Lean/TLA verification remains unpromoted",
                "reason": "The W041 Lean/TLA packet remains valid and records five exact blockers; W042.4 builds on it without treating it as full verification.",
                "evidence_paths": [&w041_lean_tla_summary_path, &w041_lean_tla_validation_path, W041_LEAN_TLA_DISCHARGE_FILE],
                "failures": if string_value(&w041_lean_tla_validation, "status") == "formal_assurance_w041_lean_tla_discharge_valid" && w041_lean_discharge_file_present { Vec::<String>::new() } else { vec!["w041_lean_tla_predecessor_not_valid".to_string()] },
                "validation_state": if string_value(&w041_lean_tla_validation, "status") == "formal_assurance_w041_lean_tla_discharge_valid" && w041_lean_discharge_file_present { "w042_lean_proof_row_validated" } else { "w042_lean_proof_row_failed" }
            }),
            json!({
                "row_id": "w042_lean_rust_dynamic_refinement_bridge",
                "w042_obligation_id": "W042-OBL-009",
                "source_inputs": ["W042 Rust totality/refinement packet", W042_LEAN_RUST_TOTALITY_FILE],
                "disposition_kind": "checked_lean_refinement_bridge",
                "disposition": "bind the W042 automatic dynamic transition refinement row as a checked Lean/TLA proof input",
                "local_checked_proof": true,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": false,
                "exact_remaining_blocker": false,
                "authority_owner": "calc-czd.3; calc-czd.4",
                "promotion_consequence": "Rust refinement and full optimized/core verification remain unpromoted because retained blockers remain",
                "reason": "W042.3 retains the exercised automatic dynamic transition as direct refinement evidence while keeping broader dynamic coverage and Rust totality blockers.",
                "evidence_paths": [&w042_rust_summary_path, &w042_rust_validation_path, &w042_rust_refinement_path, W042_LEAN_RUST_TOTALITY_FILE],
                "observed": {
                    "automatic_dynamic_transition_row_count": counter_value(&w042_rust_summary, "automatic_dynamic_transition_row_count"),
                    "w042_rust_exact_blocker_count": counter_value(&w042_rust_summary, "exact_remaining_blocker_count")
                },
                "failures": if string_value(&w042_rust_validation, "status") == "formal_assurance_w042_rust_totality_refinement_valid" && counter_value(&w042_rust_summary, "automatic_dynamic_transition_row_count") == 1 && w042_dynamic_refinement_present && w042_rust_file_present { Vec::<String>::new() } else { vec!["w042_rust_dynamic_refinement_bridge_missing".to_string()] },
                "validation_state": if string_value(&w042_rust_validation, "status") == "formal_assurance_w042_rust_totality_refinement_valid" && counter_value(&w042_rust_summary, "automatic_dynamic_transition_row_count") == 1 && w042_dynamic_refinement_present && w042_rust_file_present { "w042_lean_proof_row_validated" } else { "w042_lean_proof_row_failed" }
            }),
            json!({
                "row_id": "w042_lean_callable_carrier_boundary_bridge",
                "w042_obligation_id": "W042-OBL-012",
                "source_inputs": ["W042 Rust callable value-carrier row", "W042 closure obligation map"],
                "disposition_kind": "checked_lean_callable_carrier_bridge",
                "disposition": "bind the ordinary LET/LAMBDA value-carrier row as checked input while keeping callable carrier sufficiency and general OxFunc kernels unpromoted",
                "local_checked_proof": true,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": false,
                "exact_remaining_blocker": false,
                "authority_owner": "calc-czd.3; calc-czd.4; calc-czd.8",
                "promotion_consequence": "callable carrier sufficiency and general OxFunc kernel claims remain unpromoted",
                "reason": "W042.3 proves ordinary value-carrier publication for the current LET/LAMBDA fixture, not metadata projection or broad OxFunc semantics.",
                "evidence_paths": [&w042_rust_ledger_path, &w042_obligation_map_path],
                "observed": {
                    "callable_value_carrier_row_present": w042_callable_value_carrier_present
                },
                "failures": if w042_callable_value_carrier_present && w040_obligation_exists(&w042_obligation_map, "W042-OBL-012") && w040_obligation_exists(&w042_obligation_map, "W042-OBL-026") { Vec::<String>::new() } else { vec!["w042_callable_carrier_bridge_missing".to_string()] },
                "validation_state": if w042_callable_value_carrier_present && w040_obligation_exists(&w042_obligation_map, "W042-OBL-012") && w040_obligation_exists(&w042_obligation_map, "W042-OBL-026") { "w042_lean_proof_row_validated" } else { "w042_lean_proof_row_failed" }
            }),
            json!({
                "row_id": "w042_lean_stage2_analyzer_pack_predicate_carried",
                "w042_obligation_id": "W042-OBL-011",
                "source_inputs": ["W041 Stage 2 Lean predicate and replay packet"],
                "disposition_kind": "checked_lean_policy_predicate",
                "disposition": "carry the checked W041 Stage 2 analyzer and pack-equivalence predicate as proof input while retaining production-policy blockers",
                "local_checked_proof": true,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": false,
                "exact_remaining_blocker": false,
                "authority_owner": "calc-czd.4; calc-czd.5",
                "promotion_consequence": "Stage 2 production policy remains unpromoted",
                "reason": "The W041 predicate and replay packet prove no-promotion under current bounded evidence; they are not production analyzer soundness or fairness discharge.",
                "evidence_paths": [W041_STAGE2_POLICY_FILE, &w041_stage2_summary_path, &w041_stage2_validation_path],
                "failures": if w041_stage2_policy_file_present && string_value(&w041_stage2_validation, "status") == "w041_stage2_analyzer_pack_equivalence_valid" && !bool_at(&w041_stage2_summary, "stage2_policy_promoted") { Vec::<String>::new() } else { vec!["w041_stage2_policy_input_missing_or_promoted".to_string()] },
                "validation_state": if w041_stage2_policy_file_present && string_value(&w041_stage2_validation, "status") == "w041_stage2_analyzer_pack_equivalence_valid" && !bool_at(&w041_stage2_summary, "stage2_policy_promoted") { "w042_lean_proof_row_validated" } else { "w042_lean_proof_row_failed" }
            }),
            json!({
                "row_id": "w042_tla_routine_config_bounded_model_boundary",
                "w042_obligation_id": "W042-OBL-011",
                "source_inputs": ["W037 TLA inventory", "routine TLC config set"],
                "disposition_kind": "bounded_model_with_exact_totality_boundary",
                "disposition": "bind the routine TLC config set as bounded model evidence while retaining unbounded model coverage as exact blocker",
                "local_checked_proof": false,
                "bounded_model": true,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": true,
                "exact_remaining_blocker": true,
                "authority_owner": "calc-czd.4; calc-czd.10",
                "promotion_consequence": "full TLA verification remains unpromoted",
                "reason": "The routine TLC floor has 11 bounded configs with zero recorded failures, but does not cover the unbounded scheduler and partition universe.",
                "evidence_paths": [&w037_tla_inventory_path, &w037_formal_summary_path],
                "observed": {
                    "routine_tla_config_count": routine_tla_config_count,
                    "tla_inventory_passed_count": tla_inventory_passed_count,
                    "routine_tla_failed_count": routine_tla_failed_count
                },
                "failures": if routine_tla_config_count == 11 && tla_inventory_passed_count == 11 && routine_tla_failed_count == 0 { Vec::<String>::new() } else { vec!["w042_tla_routine_config_floor_changed".to_string()] },
                "validation_state": if routine_tla_config_count == 11 && tla_inventory_passed_count == 11 && routine_tla_failed_count == 0 { "w042_tla_model_row_validated" } else { "w042_tla_model_row_failed" }
            }),
            json!({
                "row_id": "w042_tla_stage2_partition_bounded_model_evidence",
                "w042_obligation_id": "W042-OBL-011",
                "source_inputs": ["CoreEngineW036Stage2Partition bounded configs"],
                "disposition_kind": "bounded_stage2_partition_model_evidence",
                "disposition": "bind W036 Stage 2 partition configs as bounded coverage for scheduler readiness, partition cross-dependency, fence reject, and multi-reader profiles",
                "local_checked_proof": false,
                "bounded_model": true,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": false,
                "exact_remaining_blocker": false,
                "authority_owner": "calc-czd.4; calc-czd.5",
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
                "failures": if routine_tla_config_count >= 11 && routine_tla_failed_count == 0 { Vec::<String>::new() } else { vec!["w042_stage2_partition_tla_floor_changed".to_string()] },
                "validation_state": if routine_tla_config_count >= 11 && routine_tla_failed_count == 0 { "w042_tla_model_row_validated" } else { "w042_tla_model_row_failed" }
            }),
            json!({
                "row_id": "w042_stage2_equivalence_bounded_model_input",
                "w042_obligation_id": "W042-OBL-011",
                "source_inputs": ["W041 Stage 2 analyzer and pack-equivalence packet"],
                "disposition_kind": "bounded_stage2_equivalence_model_evidence",
                "disposition": "bind W041 bounded partition replay, permutation, observable-invariance, analyzer, and pack-equivalence evidence as model input without promoting Stage 2",
                "local_checked_proof": false,
                "bounded_model": true,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": false,
                "exact_remaining_blocker": false,
                "authority_owner": "calc-czd.4; calc-czd.5",
                "promotion_consequence": "Stage 2 production policy and full TLA verification remain unpromoted",
                "reason": "W041 has bounded declared-profile evidence, counterpart rows, and pack-equivalence inputs, but production analyzer soundness, fairness, operated differential service, and pack-grade governance remain absent.",
                "evidence_paths": [&w041_stage2_summary_path, &w041_stage2_validation_path, &w041_stage2_gate_path, W041_STAGE2_POLICY_FILE],
                "observed": {
                    "partition_replay_row_count": counter_value(&w041_stage2_summary, "partition_replay_row_count"),
                    "permutation_replay_row_count": counter_value(&w041_stage2_summary, "permutation_replay_row_count"),
                    "observable_invariance_row_count": counter_value(&w041_stage2_summary, "observable_invariance_row_count"),
                    "bounded_partition_analyzer_evidenced": bool_at(&w041_stage2_summary, "bounded_partition_analyzer_evidenced"),
                    "declared_pack_equivalence_evidenced": bool_at(&w041_stage2_summary, "declared_pack_equivalence_evidenced"),
                    "stage2_policy_promoted": bool_at(&w041_stage2_summary, "stage2_policy_promoted")
                },
                "failures": if string_value(&w041_stage2_validation, "status") == "w041_stage2_analyzer_pack_equivalence_valid" && bool_at(&w041_stage2_summary, "bounded_partition_analyzer_evidenced") && bool_at(&w041_stage2_summary, "declared_pack_equivalence_evidenced") && !bool_at(&w041_stage2_summary, "stage2_policy_promoted") { Vec::<String>::new() } else { vec!["w041_stage2_equivalence_input_missing_or_promoted".to_string()] },
                "validation_state": if string_value(&w041_stage2_validation, "status") == "w041_stage2_analyzer_pack_equivalence_valid" && bool_at(&w041_stage2_summary, "bounded_partition_analyzer_evidenced") && bool_at(&w041_stage2_summary, "declared_pack_equivalence_evidenced") && !bool_at(&w041_stage2_summary, "stage2_policy_promoted") { "w042_tla_model_row_validated" } else { "w042_tla_model_row_failed" }
            }),
            json!({
                "row_id": "w042_tla_fairness_scheduler_unbounded_boundary",
                "w042_obligation_id": "W042-OBL-011",
                "source_inputs": ["TLA model bounds and scheduler assumptions"],
                "disposition_kind": "exact_model_assumption_boundary",
                "disposition": "retain scheduler fairness, unbounded interleaving, and model-completeness assumptions as explicit model boundaries",
                "local_checked_proof": false,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": true,
                "exact_remaining_blocker": true,
                "authority_owner": "calc-czd.4; calc-czd.5; calc-czd.10",
                "promotion_consequence": "full TLA verification and Stage 2 policy remain unpromoted",
                "reason": "Current TLC configs and bounded Stage 2 evidence do not discharge fairness or unbounded scheduler coverage for promoted profiles.",
                "evidence_paths": [&w042_obligation_map_path, &w037_tla_inventory_path, &w041_stage2_gate_path],
                "failures": if w040_obligation_exists(&w042_obligation_map, "W042-OBL-011") && w040_obligation_exists(&w042_obligation_map, "W042-OBL-014") && w041_fairness_stage2_blocker_present { Vec::<String>::new() } else { vec!["w042_fairness_scheduler_boundary_missing".to_string()] },
                "validation_state": if w040_obligation_exists(&w042_obligation_map, "W042-OBL-011") && w040_obligation_exists(&w042_obligation_map, "W042-OBL-014") && w041_fairness_stage2_blocker_present { "w042_lean_tla_exact_blocker_validated" } else { "w042_lean_tla_exact_blocker_failed" }
            }),
            json!({
                "row_id": "w042_full_lean_verification_exact_blocker",
                "w042_obligation_id": "W042-OBL-010",
                "source_inputs": ["W042 closure obligation map", "Lean proof inventory"],
                "disposition_kind": "exact_lean_verification_blocker",
                "disposition": "retain full Lean verification as exact blocker despite checked local proof rows",
                "local_checked_proof": true,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": true,
                "exact_remaining_blocker": true,
                "authority_owner": "calc-czd.4; calc-czd.10",
                "promotion_consequence": "full Lean verification remains unpromoted",
                "reason": "Checked classification files do not prove every Rust, OxFml, TLA, coordinator, service, and pack semantic path for the claimed scope.",
                "evidence_paths": [&w042_obligation_map_path, W042_LEAN_TLA_DISCHARGE_FILE],
                "failures": if w040_obligation_exists(&w042_obligation_map, "W042-OBL-010") && lean_discharge_file_present { Vec::<String>::new() } else { vec!["w042_full_lean_blocker_input_missing".to_string()] },
                "validation_state": if w040_obligation_exists(&w042_obligation_map, "W042-OBL-010") && lean_discharge_file_present { "w042_lean_tla_exact_blocker_validated" } else { "w042_lean_tla_exact_blocker_failed" }
            }),
            json!({
                "row_id": "w042_full_tla_verification_exact_blocker",
                "w042_obligation_id": "W042-OBL-011",
                "source_inputs": ["W042 closure obligation map", "bounded TLC evidence"],
                "disposition_kind": "exact_tla_verification_blocker",
                "disposition": "retain full TLA verification as exact blocker because current coverage is bounded",
                "local_checked_proof": false,
                "bounded_model": true,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": true,
                "exact_remaining_blocker": true,
                "authority_owner": "calc-czd.4; calc-czd.10",
                "promotion_consequence": "full TLA verification remains unpromoted",
                "reason": "Bounded TLC runs do not discharge unbounded model completeness, fairness, or production partition analyzer soundness.",
                "evidence_paths": [&w042_obligation_map_path, &w037_tla_inventory_path],
                "failures": if w040_obligation_exists(&w042_obligation_map, "W042-OBL-011") && routine_tla_config_count == 11 { Vec::<String>::new() } else { vec!["w042_full_tla_blocker_input_missing".to_string()] },
                "validation_state": if w040_obligation_exists(&w042_obligation_map, "W042-OBL-011") && routine_tla_config_count == 11 { "w042_lean_tla_exact_blocker_validated" } else { "w042_lean_tla_exact_blocker_failed" }
            }),
            json!({
                "row_id": "w042_rust_totality_dependency_exact_blocker",
                "w042_obligation_id": "W042-OBL-009",
                "source_inputs": ["W042 Rust totality/refinement blockers"],
                "disposition_kind": "exact_rust_dependency_blocker",
                "disposition": "retain Rust totality/refinement dependency as a Lean/TLA proof blocker",
                "local_checked_proof": true,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": false,
                "totality_boundary": true,
                "exact_remaining_blocker": true,
                "authority_owner": "calc-czd.4; calc-czd.10",
                "promotion_consequence": "full Lean/TLA verification remains unpromoted",
                "reason": "The preceding W042 Rust packet intentionally retains four exact blockers, so Lean/TLA full verification cannot be promoted over them.",
                "evidence_paths": [&w042_rust_summary_path, &w042_rust_blockers_path],
                "failures": if counter_value(&w042_rust_summary, "exact_remaining_blocker_count") == 4 && counter_value(&w042_rust_blockers, "exact_remaining_blocker_count") == 4 { Vec::<String>::new() } else { vec!["w042_rust_blocker_count_changed".to_string()] },
                "validation_state": if counter_value(&w042_rust_summary, "exact_remaining_blocker_count") == 4 && counter_value(&w042_rust_blockers, "exact_remaining_blocker_count") == 4 { "w042_lean_tla_exact_blocker_validated" } else { "w042_lean_tla_exact_blocker_failed" }
            }),
            json!({
                "row_id": "w042_let_lambda_external_oxfunc_boundary",
                "w042_obligation_id": "W042-OBL-033",
                "source_inputs": ["W042 closure obligation map"],
                "disposition_kind": "accepted_external_seam_boundary",
                "disposition": "keep LET/LAMBDA carrier interaction inside OxCalc/OxFml formalization while excluding general OxFunc kernels",
                "local_checked_proof": false,
                "bounded_model": false,
                "accepted_external_seam": true,
                "accepted_boundary": true,
                "totality_boundary": false,
                "exact_remaining_blocker": false,
                "authority_owner": "calc-czd.4; calc-czd.8; external:OxFunc",
                "promotion_consequence": "general OxFunc kernels remain unpromoted inside OxCalc",
                "reason": "W042 scope includes the carrier seam but not broad OxFunc semantic kernels.",
                "evidence_paths": [&w042_obligation_map_path],
                "failures": if w040_obligation_exists(&w042_obligation_map, "W042-OBL-012") && w040_obligation_exists(&w042_obligation_map, "W042-OBL-033") { Vec::<String>::new() } else { vec!["w042_let_lambda_boundary_missing".to_string()] },
                "validation_state": if w040_obligation_exists(&w042_obligation_map, "W042-OBL-012") && w040_obligation_exists(&w042_obligation_map, "W042-OBL-033") { "w042_lean_tla_boundary_validated" } else { "w042_lean_tla_boundary_failed" }
            }),
            json!({
                "row_id": "w042_formal_model_spec_evolution_guard",
                "w042_obligation_id": "W042-OBL-010",
                "source_inputs": ["W042 closure obligation map", "W042 workset"],
                "disposition_kind": "accepted_spec_evolution_guard",
                "disposition": "preserve Lean/TLA formalization as spec-evolution and implementation-improvement work",
                "local_checked_proof": true,
                "bounded_model": false,
                "accepted_external_seam": false,
                "accepted_boundary": true,
                "totality_boundary": false,
                "exact_remaining_blocker": false,
                "authority_owner": "calc-czd.4; calc-czd.10",
                "promotion_consequence": "future proof/model evidence may correct specs or implementation before promotion",
                "reason": "W042 explicitly allows proof/model evidence to evolve the specs rather than testing against a fixed initial document set.",
                "evidence_paths": [&w042_obligation_map_path, "docs/worksets/W042_CORE_FORMALIZATION_RELEASE_GRADE_EVIDENCE_CLOSURE_EXPANSION.md"],
                "failures": if w040_obligation_exists(&w042_obligation_map, "W042-OBL-010") && w040_obligation_exists(&w042_obligation_map, "W042-OBL-011") { Vec::<String>::new() } else { vec!["w042_lean_tla_obligation_missing".to_string()] },
                "validation_state": if w040_obligation_exists(&w042_obligation_map, "W042-OBL-010") && w040_obligation_exists(&w042_obligation_map, "W042-OBL-011") { "w042_lean_tla_boundary_validated" } else { "w042_lean_tla_boundary_failed" }
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
        if counter_value(&w042_obligation_summary, "obligation_count") != 33 {
            validation_failures.push("w042_obligation_count_changed".to_string());
        }
        if !array_contains_string(
            w042_obligation_summary
                .get("no_promotion_claims")
                .unwrap_or(&Value::Null),
            "full_lean_tla_verification",
        ) {
            validation_failures.push("w042_lean_tla_no_promotion_guard_missing".to_string());
        }
        if !w040_obligation_exists(&w042_obligation_map, "W042-OBL-010")
            || !w040_obligation_exists(&w042_obligation_map, "W042-OBL-011")
            || !w040_obligation_exists(&w042_obligation_map, "W042-OBL-012")
        {
            validation_failures.push("w042_lean_tla_obligation_rows_missing".to_string());
        }
        if !bool_at(&w037_formal_summary, "all_checked_artifacts_passed") {
            validation_failures.push("w037_formal_artifacts_not_all_checked".to_string());
        }
        if string_value(&w037_formal_validation, "validation_state")
            != "w037_proof_model_closure_inventory_validated"
        {
            validation_failures.push("w037_formal_validation_not_valid".to_string());
        }
        if string_value(&w041_lean_tla_validation, "status")
            != "formal_assurance_w041_lean_tla_discharge_valid"
        {
            validation_failures.push("w041_lean_tla_validation_not_valid".to_string());
        }
        if counter_value(&w041_lean_tla_blockers, "exact_remaining_blocker_count") != 5 {
            validation_failures.push("w041_lean_tla_blocker_count_changed".to_string());
        }
        if string_value(&w042_rust_validation, "status")
            != "formal_assurance_w042_rust_totality_refinement_valid"
        {
            validation_failures.push("w042_rust_validation_not_valid".to_string());
        }
        if counter_value(&w042_rust_blockers, "exact_remaining_blocker_count") != 4 {
            validation_failures.push("w042_rust_blocker_count_changed".to_string());
        }
        if string_value(&w041_stage2_validation, "status")
            != "w041_stage2_analyzer_pack_equivalence_valid"
        {
            validation_failures.push("w041_stage2_validation_not_valid".to_string());
        }
        if bool_at(&w041_stage2_summary, "stage2_policy_promoted")
            || bool_at(&w041_stage2_gate, "stage2_policy_promoted")
        {
            validation_failures.push("w041_stage2_policy_was_promoted".to_string());
        }
        if counter_value(&w041_stage2_blockers, "exact_remaining_blocker_count") != 4 {
            validation_failures.push("w041_stage2_blocker_count_changed".to_string());
        }
        let w073_typed_only_families = w042_w073_formatting_intake
            .get("typed_only_families")
            .unwrap_or(&Value::Null);
        if !array_contains_string(w073_typed_only_families, "colorScale")
            || !array_contains_string(w073_typed_only_families, "dataBar")
            || !array_contains_string(w073_typed_only_families, "iconSet")
            || !array_contains_string(w073_typed_only_families, "top")
            || !array_contains_string(w073_typed_only_families, "bottom")
            || !array_contains_string(w073_typed_only_families, "aboveAverage")
            || !array_contains_string(w073_typed_only_families, "belowAverage")
            || bool_at(
                &w042_w073_formatting_intake["oxcalc_w042_consequence"],
                "core_engine_code_change_required_in_calc_czd_2",
            )
        {
            validation_failures.push("w042_w073_typed_only_intake_changed".to_string());
        }
        if lean_placeholder_count != 0 {
            validation_failures.push("w042_lean_placeholder_count_nonzero".to_string());
        }
        if routine_tla_config_count != 11
            || tla_inventory_passed_count != 11
            || routine_tla_failed_count != 0
        {
            validation_failures.push("w042_tla_routine_floor_changed".to_string());
        }
        if !lean_discharge_file_present {
            validation_failures.push("w042_lean_tla_discharge_file_missing".to_string());
        }
        if failed_row_count != 0 {
            validation_failures.push("w042_lean_tla_row_failures_present".to_string());
        }
        if proof_rows.len() != 14 {
            validation_failures.push("w042_expected_fourteen_lean_tla_rows".to_string());
        }
        if blocker_rows.len() != 5 {
            validation_failures.push("w042_expected_five_lean_tla_exact_blockers".to_string());
        }

        let source_evidence_index_path =
            format!("{relative_artifact_root}/source_evidence_index.json");
        let ledger_path = format!("{relative_artifact_root}/w042_lean_tla_discharge_ledger.json");
        let lean_register_path = format!("{relative_artifact_root}/w042_lean_proof_register.json");
        let model_register_path =
            format!("{relative_artifact_root}/w042_tla_model_bound_register.json");
        let blocker_register_path =
            format!("{relative_artifact_root}/w042_lean_tla_exact_blocker_register.json");
        let validation_path = format!("{relative_artifact_root}/validation.json");

        write_json(
            &artifact_root.join("source_evidence_index.json"),
            &json!({
                "schema_version": FORMAL_ASSURANCE_SOURCE_INDEX_SCHEMA_V1,
                "run_id": run_id,
                "source_artifacts": {
                    "w042_closure_obligation_summary": w042_obligation_summary_path,
                    "w042_closure_obligation_map": w042_obligation_map_path,
                    "w037_formal_inventory_summary": w037_formal_summary_path,
                    "w037_formal_inventory_validation": w037_formal_validation_path,
                    "w037_tla_inventory": w037_tla_inventory_path,
                    "w041_lean_tla_summary": w041_lean_tla_summary_path,
                    "w041_lean_tla_validation": w041_lean_tla_validation_path,
                    "w041_lean_tla_exact_blockers": w041_lean_tla_blockers_path,
                    "w042_rust_formal_assurance_summary": w042_rust_summary_path,
                    "w042_rust_formal_assurance_validation": w042_rust_validation_path,
                    "w042_rust_exact_blockers": w042_rust_blockers_path,
                    "w042_rust_refinement_register": w042_rust_refinement_path,
                    "w042_rust_totality_refinement_ledger": w042_rust_ledger_path,
                    "w041_stage2_summary": w041_stage2_summary_path,
                    "w041_stage2_validation": w041_stage2_validation_path,
                    "w041_stage2_policy_gate": w041_stage2_gate_path,
                    "w041_stage2_exact_blockers": w041_stage2_blockers_path,
                    "w042_w073_formatting_intake": w042_w073_formatting_intake_path,
                    "w042_lean_tla_discharge_file": W042_LEAN_TLA_DISCHARGE_FILE,
                    "w042_rust_lean_file": W042_LEAN_RUST_TOTALITY_FILE,
                    "w041_lean_tla_discharge_file": W041_LEAN_TLA_DISCHARGE_FILE,
                    "w041_stage2_policy_file": W041_STAGE2_POLICY_FILE
                },
                "source_counts": {
                    "w042_obligation_count": counter_value(&w042_obligation_summary, "obligation_count"),
                    "w037_lean_file_count": counter_value(&w037_formal_summary, "lean_file_count"),
                    "w037_tla_routine_config_count": routine_tla_config_count,
                    "w037_tla_inventory_passed_count": tla_inventory_passed_count,
                    "w037_tla_failed_config_count": routine_tla_failed_count,
                    "w041_lean_tla_exact_blocker_count": counter_value(&w041_lean_tla_summary, "exact_remaining_blocker_count"),
                    "w042_rust_exact_blocker_count": counter_value(&w042_rust_summary, "exact_remaining_blocker_count"),
                    "w042_dynamic_refinement_row_count": counter_value(&w042_rust_summary, "automatic_dynamic_transition_row_count"),
                    "w041_stage2_exact_blocker_count": counter_value(&w041_stage2_summary, "exact_remaining_blocker_count"),
                    "w041_stage2_policy_row_count": counter_value(&w041_stage2_summary, "policy_row_count"),
                    "w073_typed_only_family_count": w042_w073_formatting_intake.get("typed_only_families").and_then(Value::as_array).map_or(0, Vec::len),
                    "lean_placeholder_count": lean_placeholder_count
                }
            }),
        )?;
        write_json(
            &artifact_root.join("w042_lean_tla_discharge_ledger.json"),
            &json!({
                "schema_version": FORMAL_ASSURANCE_W042_LEAN_TLA_LEDGER_SCHEMA_V1,
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
            &artifact_root.join("w042_lean_proof_register.json"),
            &json!({
                "schema_version": FORMAL_ASSURANCE_W042_LEAN_PROOF_REGISTER_SCHEMA_V1,
                "run_id": run_id,
                "local_proof_row_count": lean_proof_rows.len(),
                "rows": lean_proof_rows
            }),
        )?;
        write_json(
            &artifact_root.join("w042_tla_model_bound_register.json"),
            &json!({
                "schema_version": FORMAL_ASSURANCE_W042_TLA_MODEL_REGISTER_SCHEMA_V1,
                "run_id": run_id,
                "bounded_model_row_count": model_bound_rows.len(),
                "rows": model_bound_rows
            }),
        )?;
        write_json(
            &artifact_root.join("w042_lean_tla_exact_blocker_register.json"),
            &json!({
                "schema_version": FORMAL_ASSURANCE_W042_LEAN_TLA_BLOCKER_REGISTER_SCHEMA_V1,
                "run_id": run_id,
                "exact_remaining_blocker_count": blocker_rows.len(),
                "rows": blocker_rows
            }),
        )?;

        let validation_status = if validation_failures.is_empty() {
            "formal_assurance_w042_lean_tla_fairness_expansion_valid"
        } else {
            "formal_assurance_w042_lean_tla_fairness_expansion_invalid"
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
                    "rust_refinement_promoted": false,
                    "full_optimized_core_verification_promoted": false,
                    "stage2_policy_promoted": false,
                    "pack_grade_replay_promoted": false,
                    "c5_promoted": false,
                    "callable_carrier_sufficiency_promoted": false,
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
            rows.iter().any(|row| {
                row.get("obligation_id").and_then(Value::as_str) == Some(obligation_id)
                    || row.get("id").and_then(Value::as_str) == Some(obligation_id)
            })
        })
}

fn treecalc_result_publishes_value(value: &Value, node_id: &str, expected_value: &str) -> bool {
    let numeric_node_id = node_id.parse::<u64>().ok();
    value
        .get("published_values")
        .and_then(Value::as_array)
        .is_some_and(|values| {
            values.iter().any(|value| {
                numeric_node_id.is_some_and(|node_id| {
                    value.get("node_id").and_then(Value::as_u64) == Some(node_id)
                }) && value.get("value").and_then(Value::as_str) == Some(expected_value)
            })
        })
        || value
            .get("candidate_result")
            .and_then(|candidate| candidate.get("value_updates"))
            .and_then(|updates| updates.get(node_id))
            .and_then(Value::as_str)
            == Some(expected_value)
        || value
            .get("publication_bundle")
            .and_then(|publication| publication.get("published_view_delta"))
            .and_then(|updates| updates.get(node_id))
            .and_then(Value::as_str)
            == Some(expected_value)
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
    fn formal_assurance_runner_classifies_w041_rust_totality_and_refinement() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let run_id = format!("test-w041-rust-formal-assurance-{}", std::process::id());
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
        assert_eq!(summary.totality_boundary_count, 4);
        assert_eq!(summary.exact_remaining_blocker_count, 4);
        assert_eq!(summary.failed_row_count, 0);

        let validation = read_json(
            &repo_root,
            &format!("docs/test-runs/core-engine/formal-assurance/{run_id}/validation.json"),
        )
        .unwrap();
        assert_eq!(
            validation["status"],
            "formal_assurance_w041_rust_totality_refinement_valid"
        );
        assert_eq!(validation["automatic_dynamic_transition_row_count"], 1);

        let blocker_register = read_json(
            &repo_root,
            &format!(
                "docs/test-runs/core-engine/formal-assurance/{run_id}/w041_rust_exact_blocker_register.json"
            ),
        )
        .unwrap();
        assert_eq!(blocker_register["exact_remaining_blocker_count"], 4);

        cleanup();
    }

    #[test]
    fn formal_assurance_runner_classifies_w042_rust_totality_and_refinement() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let run_id = format!("test-w042-rust-formal-assurance-{}", std::process::id());
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

        assert_eq!(summary.assumption_row_count, 13);
        assert_eq!(summary.local_proof_row_count, 10);
        assert_eq!(summary.bounded_model_row_count, 0);
        assert_eq!(summary.accepted_external_seam_count, 1);
        assert_eq!(summary.accepted_boundary_count, 2);
        assert_eq!(summary.totality_boundary_count, 4);
        assert_eq!(summary.exact_remaining_blocker_count, 4);
        assert_eq!(summary.failed_row_count, 0);

        let validation = read_json(
            &repo_root,
            &format!("docs/test-runs/core-engine/formal-assurance/{run_id}/validation.json"),
        )
        .unwrap();
        assert_eq!(
            validation["status"],
            "formal_assurance_w042_rust_totality_refinement_valid"
        );
        assert_eq!(validation["automatic_dynamic_transition_row_count"], 1);

        let blocker_register = read_json(
            &repo_root,
            &format!(
                "docs/test-runs/core-engine/formal-assurance/{run_id}/w042_rust_exact_blocker_register.json"
            ),
        )
        .unwrap();
        assert_eq!(blocker_register["exact_remaining_blocker_count"], 4);

        cleanup();
    }

    #[test]
    fn formal_assurance_runner_classifies_w043_rust_totality_and_refinement() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let run_id = format!("test-w043-rust-formal-assurance-{}", std::process::id());
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

        assert_eq!(summary.assumption_row_count, 14);
        assert_eq!(summary.local_proof_row_count, 11);
        assert_eq!(summary.bounded_model_row_count, 0);
        assert_eq!(summary.accepted_external_seam_count, 1);
        assert_eq!(summary.accepted_boundary_count, 2);
        assert_eq!(summary.totality_boundary_count, 4);
        assert_eq!(summary.exact_remaining_blocker_count, 4);
        assert_eq!(summary.failed_row_count, 0);

        let validation = read_json(
            &repo_root,
            &format!("docs/test-runs/core-engine/formal-assurance/{run_id}/validation.json"),
        )
        .unwrap();
        assert_eq!(
            validation["status"],
            "formal_assurance_w043_rust_totality_refinement_valid"
        );
        assert_eq!(validation["automatic_dynamic_transition_row_count"], 2);
        assert_eq!(validation["w073_typed_only_guard_present"], true);

        let refinement_register = read_json(
            &repo_root,
            &format!(
                "docs/test-runs/core-engine/formal-assurance/{run_id}/w043_rust_refinement_register.json"
            ),
        )
        .unwrap();
        assert_eq!(
            refinement_register["automatic_dynamic_transition_row_count"],
            2
        );

        let blocker_register = read_json(
            &repo_root,
            &format!(
                "docs/test-runs/core-engine/formal-assurance/{run_id}/w043_rust_exact_blocker_register.json"
            ),
        )
        .unwrap();
        assert_eq!(blocker_register["exact_remaining_blocker_count"], 4);

        cleanup();
    }

    #[test]
    fn formal_assurance_runner_classifies_w044_rust_totality_and_refinement() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let run_id = format!("test-w044-rust-formal-assurance-{}", std::process::id());
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

        assert_eq!(summary.assumption_row_count, 16);
        assert_eq!(summary.local_proof_row_count, 11);
        assert_eq!(summary.bounded_model_row_count, 0);
        assert_eq!(summary.accepted_external_seam_count, 1);
        assert_eq!(summary.accepted_boundary_count, 3);
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
            "formal_assurance_w044_rust_totality_refinement_valid"
        );
        assert_eq!(validation["automatic_dynamic_transition_row_count"], 1);
        assert_eq!(validation["w073_typed_only_guard_present"], true);

        let refinement_register = read_json(
            &repo_root,
            &format!(
                "docs/test-runs/core-engine/formal-assurance/{run_id}/w044_rust_refinement_register.json"
            ),
        )
        .unwrap();
        assert_eq!(
            refinement_register["automatic_dynamic_transition_row_count"],
            1
        );
        assert!(
            refinement_register["rows"]
                .as_array()
                .unwrap()
                .iter()
                .any(|row| row["row_id"] == "w044_mixed_dynamic_add_release_refinement_evidence")
        );

        let blocker_register = read_json(
            &repo_root,
            &format!(
                "docs/test-runs/core-engine/formal-assurance/{run_id}/w044_rust_exact_blocker_register.json"
            ),
        )
        .unwrap();
        assert_eq!(blocker_register["exact_remaining_blocker_count"], 6);

        cleanup();
    }

    #[test]
    fn formal_assurance_runner_classifies_w042_lean_tla_fairness_expansion() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let run_id = format!("test-w042-lean-tla-formal-assurance-{}", std::process::id());
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

        assert_eq!(summary.assumption_row_count, 14);
        assert_eq!(summary.local_proof_row_count, 8);
        assert_eq!(summary.bounded_model_row_count, 4);
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
            "formal_assurance_w042_lean_tla_fairness_expansion_valid"
        );

        let blocker_register = read_json(
            &repo_root,
            &format!(
                "docs/test-runs/core-engine/formal-assurance/{run_id}/w042_lean_tla_exact_blocker_register.json"
            ),
        )
        .unwrap();
        assert_eq!(blocker_register["exact_remaining_blocker_count"], 5);

        let source_index = read_json(
            &repo_root,
            &format!(
                "docs/test-runs/core-engine/formal-assurance/{run_id}/source_evidence_index.json"
            ),
        )
        .unwrap();
        assert_eq!(
            source_index["source_counts"]["w073_typed_only_family_count"],
            7
        );
        assert_eq!(
            source_index["source_artifacts"]["w042_w073_formatting_intake"],
            "docs/test-runs/core-engine/implementation-conformance/w042-optimized-core-counterpart-conformance-callable-metadata-001/w073_formatting_intake.json"
        );

        cleanup();
    }

    #[test]
    fn formal_assurance_runner_classifies_w043_lean_tla_fairness_expansion() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let run_id = format!("test-w043-lean-tla-formal-assurance-{}", std::process::id());
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

        assert_eq!(summary.assumption_row_count, 15);
        assert_eq!(summary.local_proof_row_count, 9);
        assert_eq!(summary.bounded_model_row_count, 4);
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
            "formal_assurance_w043_lean_tla_fairness_valid"
        );
        assert_eq!(validation["dynamic_refinement_bridge_row_count"], 2);
        assert_eq!(validation["w073_typed_only_guard_present"], true);

        let blocker_register = read_json(
            &repo_root,
            &format!(
                "docs/test-runs/core-engine/formal-assurance/{run_id}/w043_lean_tla_exact_blocker_register.json"
            ),
        )
        .unwrap();
        assert_eq!(blocker_register["exact_remaining_blocker_count"], 5);

        cleanup();
    }

    #[test]
    fn formal_assurance_runner_classifies_w041_lean_tla_discharge() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let run_id = format!("test-w041-lean-tla-formal-assurance-{}", std::process::id());
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

        assert_eq!(summary.assumption_row_count, 13);
        assert_eq!(summary.local_proof_row_count, 7);
        assert_eq!(summary.bounded_model_row_count, 4);
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
            "formal_assurance_w041_lean_tla_discharge_valid"
        );

        let blocker_register = read_json(
            &repo_root,
            &format!(
                "docs/test-runs/core-engine/formal-assurance/{run_id}/w041_lean_tla_exact_blocker_register.json"
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
