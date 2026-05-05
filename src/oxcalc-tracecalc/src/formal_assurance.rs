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
const FORMAL_ASSURANCE_VALIDATION_SCHEMA_V1: &str = "oxcalc.formal_assurance.validation.v1";

const W037_FORMAL_INVENTORY_RUN_ID: &str = "w037-proof-model-closure-001";
const W037_STAGE2_CRITERIA_RUN_ID: &str = "w037-stage2-deterministic-replay-criteria-001";
const W038_IMPLEMENTATION_CONFORMANCE_RUN_ID: &str =
    "w038-optimized-core-conformance-disposition-001";
const W038_TRACECALC_AUTHORITY_RUN_ID: &str = "w038-tracecalc-authority-discharge-001";
const W038_LEAN_ASSUMPTION_FILE: &str =
    "formal/lean/OxCalc/CoreEngine/W038AssumptionDischargeAndTotality.lean";

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

fn array_contains_string(value: &Value, expected: &str) -> bool {
    value
        .as_array()
        .is_some_and(|items| items.iter().any(|item| item.as_str() == Some(expected)))
}

fn bool_at(value: &Value, key: &str) -> bool {
    value.get(key).and_then(Value::as_bool).unwrap_or(false)
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
}
