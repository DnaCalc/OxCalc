#![forbid(unsafe_code)]

use std::env;
use std::path::Path;
use std::process::ExitCode;

use oxcalc_core::treecalc_runner::TreeCalcRunner as LocalTreeCalcRunner;
use oxcalc_core::treecalc_scale::{
    TreeCalcScaleOptions, TreeCalcScaleProfile, TreeCalcScaleRunner,
};
use oxcalc_core::upstream_host_runner::UpstreamHostRunner;
use oxcalc_tracecalc::continuous_assurance::ContinuousAssuranceRunner;
use oxcalc_tracecalc::diversity_seam::DiversitySeamRunner;
use oxcalc_tracecalc::formal_assurance::FormalAssuranceRunner;
use oxcalc_tracecalc::implementation_conformance::ImplementationConformanceRunner;
use oxcalc_tracecalc::independent_conformance::IndependentConformanceRunner;
use oxcalc_tracecalc::operated_assurance::OperatedAssuranceRunner;
use oxcalc_tracecalc::oracle_matrix::TraceCalcOracleMatrixRunner;
use oxcalc_tracecalc::oxfml_fixture_bridge::OxFmlFixtureBridgeRunner;
use oxcalc_tracecalc::oxfml_seam::OxFmlSeamRunner;
use oxcalc_tracecalc::pack_capability::PackCapabilityRunner;
use oxcalc_tracecalc::retained_failures::TraceCalcRetainedFailureRunner;
use oxcalc_tracecalc::runner::TraceCalcRunner;
use oxcalc_tracecalc::scale_semantic_binding::ScaleSemanticBindingRunner;
use oxcalc_tracecalc::stage2_replay::Stage2ReplayRunner;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let Some(first_arg) = args.next() else {
        return Err(
            "usage: oxcalc-tracecalc-cli <run-id> | tracecalc-oracle-matrix <run-id> | implementation-conformance <run-id> | formal-assurance <run-id> | stage2-replay <run-id> | operated-assurance <run-id> | diversity-seam <run-id> | oxfml-seam <run-id> | continuous-assurance <run-id> | retained-failures <run-id> | treecalc <run-id> | oxfml-bridge <run-id> | upstream-host <run-id> | independent-conformance <run-id> | pack-capability <run-id> | scale-semantic-binding <run-id> | treecalc-scale <profile> <run-id> [options]"
                .to_string(),
        );
    };
    if first_arg == "treecalc-scale" {
        let Some(profile_arg) = args.next() else {
            return Err(treecalc_scale_usage());
        };
        let Some(run_id) = args.next() else {
            return Err(treecalc_scale_usage());
        };
        let Some(profile) = TreeCalcScaleProfile::parse(&profile_arg) else {
            return Err(format!(
                "unknown treecalc-scale profile '{profile_arg}'. {}",
                treecalc_scale_usage()
            ));
        };
        let options = parse_treecalc_scale_options(profile, run_id, args)?;
        let repo_root = env::current_dir()
            .map_err(|error| format!("failed to read current directory: {error}"))?
            .canonicalize()
            .map_err(|error| format!("failed to canonicalize repo root: {error}"))?;
        ensure_treecalc_root(&repo_root)?;
        let runner = TreeCalcScaleRunner::new();
        let summary = runner
            .execute(&repo_root, options)
            .map_err(|error| format!("treecalc scale run failed: {error}"))?;
        println!(
            "TreeCalc scale run '{}' ({}) wrote {} formulas, {} descriptors, {} edges to {}.",
            summary.run_id,
            summary.profile,
            summary.formula_count,
            summary.descriptor_count,
            summary.edge_count,
            summary.artifact_root
        );
        return Ok(());
    }

    let (mode, run_id) = match first_arg.as_str() {
        "treecalc" => {
            let Some(run_id) = args.next() else {
                return Err("usage: oxcalc-tracecalc-cli treecalc <run-id>".to_string());
            };
            if args.next().is_some() {
                return Err("usage: oxcalc-tracecalc-cli treecalc <run-id>".to_string());
            }
            ("treecalc", run_id)
        }
        "tracecalc-oracle-matrix" => {
            let Some(run_id) = args.next() else {
                return Err(
                    "usage: oxcalc-tracecalc-cli tracecalc-oracle-matrix <run-id>".to_string(),
                );
            };
            if args.next().is_some() {
                return Err(
                    "usage: oxcalc-tracecalc-cli tracecalc-oracle-matrix <run-id>".to_string(),
                );
            }
            ("tracecalc-oracle-matrix", run_id)
        }
        "implementation-conformance" => {
            let Some(run_id) = args.next() else {
                return Err(
                    "usage: oxcalc-tracecalc-cli implementation-conformance <run-id>".to_string(),
                );
            };
            if args.next().is_some() {
                return Err(
                    "usage: oxcalc-tracecalc-cli implementation-conformance <run-id>".to_string(),
                );
            }
            ("implementation-conformance", run_id)
        }
        "formal-assurance" => {
            let Some(run_id) = args.next() else {
                return Err("usage: oxcalc-tracecalc-cli formal-assurance <run-id>".to_string());
            };
            if args.next().is_some() {
                return Err("usage: oxcalc-tracecalc-cli formal-assurance <run-id>".to_string());
            }
            ("formal-assurance", run_id)
        }
        "stage2-replay" => {
            let Some(run_id) = args.next() else {
                return Err("usage: oxcalc-tracecalc-cli stage2-replay <run-id>".to_string());
            };
            if args.next().is_some() {
                return Err("usage: oxcalc-tracecalc-cli stage2-replay <run-id>".to_string());
            }
            ("stage2-replay", run_id)
        }
        "operated-assurance" => {
            let Some(run_id) = args.next() else {
                return Err("usage: oxcalc-tracecalc-cli operated-assurance <run-id>".to_string());
            };
            if args.next().is_some() {
                return Err("usage: oxcalc-tracecalc-cli operated-assurance <run-id>".to_string());
            }
            ("operated-assurance", run_id)
        }
        "diversity-seam" => {
            let Some(run_id) = args.next() else {
                return Err("usage: oxcalc-tracecalc-cli diversity-seam <run-id>".to_string());
            };
            if args.next().is_some() {
                return Err("usage: oxcalc-tracecalc-cli diversity-seam <run-id>".to_string());
            }
            ("diversity-seam", run_id)
        }
        "oxfml-seam" => {
            let Some(run_id) = args.next() else {
                return Err("usage: oxcalc-tracecalc-cli oxfml-seam <run-id>".to_string());
            };
            if args.next().is_some() {
                return Err("usage: oxcalc-tracecalc-cli oxfml-seam <run-id>".to_string());
            }
            ("oxfml-seam", run_id)
        }
        "continuous-assurance" => {
            let Some(run_id) = args.next() else {
                return Err("usage: oxcalc-tracecalc-cli continuous-assurance <run-id>".to_string());
            };
            if args.next().is_some() {
                return Err("usage: oxcalc-tracecalc-cli continuous-assurance <run-id>".to_string());
            }
            ("continuous-assurance", run_id)
        }
        "retained-failures" => {
            let Some(run_id) = args.next() else {
                return Err("usage: oxcalc-tracecalc-cli retained-failures <run-id>".to_string());
            };
            if args.next().is_some() {
                return Err("usage: oxcalc-tracecalc-cli retained-failures <run-id>".to_string());
            }
            ("retained-failures", run_id)
        }
        "oxfml-bridge" => {
            let Some(run_id) = args.next() else {
                return Err("usage: oxcalc-tracecalc-cli oxfml-bridge <run-id>".to_string());
            };
            if args.next().is_some() {
                return Err("usage: oxcalc-tracecalc-cli oxfml-bridge <run-id>".to_string());
            }
            ("oxfml-bridge", run_id)
        }
        "upstream-host" => {
            let Some(run_id) = args.next() else {
                return Err("usage: oxcalc-tracecalc-cli upstream-host <run-id>".to_string());
            };
            if args.next().is_some() {
                return Err("usage: oxcalc-tracecalc-cli upstream-host <run-id>".to_string());
            }
            ("upstream-host", run_id)
        }
        "independent-conformance" => {
            let Some(run_id) = args.next() else {
                return Err(
                    "usage: oxcalc-tracecalc-cli independent-conformance <run-id>".to_string(),
                );
            };
            if args.next().is_some() {
                return Err(
                    "usage: oxcalc-tracecalc-cli independent-conformance <run-id>".to_string(),
                );
            }
            ("independent-conformance", run_id)
        }
        "pack-capability" => {
            let Some(run_id) = args.next() else {
                return Err("usage: oxcalc-tracecalc-cli pack-capability <run-id>".to_string());
            };
            if args.next().is_some() {
                return Err("usage: oxcalc-tracecalc-cli pack-capability <run-id>".to_string());
            }
            ("pack-capability", run_id)
        }
        "scale-semantic-binding" => {
            let Some(run_id) = args.next() else {
                return Err(
                    "usage: oxcalc-tracecalc-cli scale-semantic-binding <run-id>".to_string(),
                );
            };
            if args.next().is_some() {
                return Err(
                    "usage: oxcalc-tracecalc-cli scale-semantic-binding <run-id>".to_string(),
                );
            }
            ("scale-semantic-binding", run_id)
        }
        run_id => {
            if args.next().is_some() {
                return Err("usage: oxcalc-tracecalc-cli <run-id>".to_string());
            }
            ("tracecalc", run_id.to_string())
        }
    };

    let repo_root = env::current_dir()
        .map_err(|error| format!("failed to read current directory: {error}"))?
        .canonicalize()
        .map_err(|error| format!("failed to canonicalize repo root: {error}"))?;

    match mode {
        "retained-failures" => ensure_retained_failure_root(&repo_root)?,
        "implementation-conformance" => ensure_implementation_conformance_root(&repo_root)?,
        "formal-assurance" => ensure_formal_assurance_root(&repo_root)?,
        "stage2-replay" => ensure_stage2_replay_root(&repo_root)?,
        "operated-assurance" => ensure_operated_assurance_root(&repo_root)?,
        "diversity-seam" => ensure_diversity_seam_root(&repo_root)?,
        "oxfml-seam" => ensure_oxfml_seam_root(&repo_root)?,
        "continuous-assurance" => ensure_continuous_assurance_root(&repo_root)?,
        "tracecalc-oracle-matrix" => ensure_repo_root(&repo_root)?,
        "treecalc" => ensure_treecalc_root(&repo_root)?,
        "oxfml-bridge" => ensure_oxfml_bridge_root(&repo_root)?,
        "upstream-host" => ensure_upstream_host_root(&repo_root)?,
        "independent-conformance" => ensure_independent_conformance_root(&repo_root)?,
        "pack-capability" => ensure_pack_capability_root(&repo_root)?,
        "scale-semantic-binding" => ensure_scale_semantic_binding_root(&repo_root)?,
        _ => ensure_repo_root(&repo_root)?,
    }

    match mode {
        "treecalc" => {
            let runner = LocalTreeCalcRunner::new();
            let summary = runner
                .execute_manifest(&repo_root, &run_id)
                .map_err(|error| format!("treecalc run failed: {error}"))?;
            println!(
                "TreeCalc local run '{run_id}' wrote {} cases to {}.",
                summary.case_count, summary.artifact_root
            );
        }
        "tracecalc-oracle-matrix" => {
            let runner = TraceCalcOracleMatrixRunner::new();
            let summary = runner
                .execute(&repo_root, &run_id)
                .map_err(|error| format!("TraceCalc oracle matrix run failed: {error}"))?;
            println!(
                "TraceCalc oracle matrix run '{run_id}' wrote {} matrix rows ({} covered, {} classified uncovered, {} excluded, {} failed/missing) to {}.",
                summary.matrix_row_count,
                summary.covered_row_count,
                summary.uncovered_row_count,
                summary.excluded_row_count,
                summary.missing_or_failed_row_count,
                summary.artifact_root
            );
        }
        "implementation-conformance" => {
            let runner = ImplementationConformanceRunner::new();
            let summary = runner
                .execute(&repo_root, &run_id)
                .map_err(|error| format!("implementation conformance run failed: {error}"))?;
            if summary.w045_disposition_row_count > 0 {
                println!(
                    "Implementation conformance run '{run_id}' wrote {} W045 dispositions ({} direct-evidence rows, {} exact remaining blockers, {} match-promoted, {} failed) to {}.",
                    summary.w045_disposition_row_count,
                    summary.w045_direct_evidence_bound_count,
                    summary.w045_exact_remaining_blocker_count,
                    summary.w045_match_promoted_count,
                    summary.failed_row_count,
                    summary.artifact_root
                );
            } else if summary.w044_disposition_row_count > 0 {
                println!(
                    "Implementation conformance run '{run_id}' wrote {} W044 dispositions ({} direct-evidence rows, {} exact remaining blockers, {} match-promoted, {} failed) to {}.",
                    summary.w044_disposition_row_count,
                    summary.w044_direct_evidence_bound_count,
                    summary.w044_exact_remaining_blocker_count,
                    summary.w044_match_promoted_count,
                    summary.failed_row_count,
                    summary.artifact_root
                );
            } else if summary.w039_disposition_row_count > 0 {
                println!(
                    "Implementation conformance run '{run_id}' wrote {} W039 dispositions ({} direct-evidence rows, {} exact remaining blockers, {} match-promoted, {} failed) to {}.",
                    summary.w039_disposition_row_count,
                    summary.w039_direct_evidence_bound_count,
                    summary.w039_exact_remaining_blocker_count,
                    summary.w039_match_promoted_count,
                    summary.failed_row_count,
                    summary.artifact_root
                );
            } else if summary.w038_disposition_row_count > 0 {
                println!(
                    "Implementation conformance run '{run_id}' wrote {} W038 dispositions ({} direct-evidence rows, {} accepted boundaries, {} exact remaining blockers, {} match-promoted, {} failed) to {}.",
                    summary.w038_disposition_row_count,
                    summary.w038_direct_evidence_bound_count,
                    summary.w038_accepted_boundary_count,
                    summary.w038_exact_remaining_blocker_count,
                    summary.w038_match_promoted_count,
                    summary.failed_row_count,
                    summary.artifact_root
                );
            } else if summary.w037_decision_row_count > 0 {
                println!(
                    "Implementation conformance run '{run_id}' wrote {} W037 decisions ({} fixed/promoted, {} residual blockers, {} match-promoted, {} failed) to {}.",
                    summary.w037_decision_row_count,
                    summary.w037_fixed_or_promoted_count,
                    summary.w037_residual_blocker_count,
                    summary.w037_match_promoted_count,
                    summary.failed_row_count,
                    summary.artifact_root
                );
            } else if summary.w036_action_row_count > 0 {
                println!(
                    "Implementation conformance run '{run_id}' wrote {} W036 closure actions ({} first-fix harness rows, {} blocker-routed, {} match-promoted, {} failed) to {}.",
                    summary.w036_action_row_count,
                    summary.w036_first_fix_row_count,
                    summary.w036_blocker_routed_row_count,
                    summary.w036_match_promoted_count,
                    summary.failed_row_count,
                    summary.artifact_root
                );
            } else {
                println!(
                    "Implementation conformance run '{run_id}' wrote {} gap dispositions ({} implementation-work, {} spec-deferral, {} failed) to {}.",
                    summary.gap_disposition_row_count,
                    summary.implementation_work_count,
                    summary.spec_evolution_deferral_count,
                    summary.failed_row_count,
                    summary.artifact_root
                );
            }
        }
        "formal-assurance" => {
            let runner = FormalAssuranceRunner::new();
            let summary = runner
                .execute(&repo_root, &run_id)
                .map_err(|error| format!("formal assurance run failed: {error}"))?;
            println!(
                "Formal assurance run '{run_id}' wrote {} assumption rows ({} local proof, {} bounded model, {} external seam, {} exact blockers, {} failed) to {}.",
                summary.assumption_row_count,
                summary.local_proof_row_count,
                summary.bounded_model_row_count,
                summary.accepted_external_seam_count,
                summary.exact_remaining_blocker_count,
                summary.failed_row_count,
                summary.artifact_root
            );
        }
        "stage2-replay" => {
            let runner = Stage2ReplayRunner::new();
            let summary = runner
                .execute(&repo_root, &run_id)
                .map_err(|error| format!("Stage 2 replay run failed: {error}"))?;
            println!(
                "Stage 2 replay run '{run_id}' wrote {} partition rows, {} permutation rows, {} invariant rows, {} exact blockers, and {} failed rows to {}.",
                summary.partition_replay_row_count,
                summary.permutation_replay_row_count,
                summary.observable_invariance_row_count,
                summary.exact_remaining_blocker_count,
                summary.failed_row_count,
                summary.artifact_root
            );
        }
        "operated-assurance" => {
            let runner = OperatedAssuranceRunner::new();
            let summary = runner
                .execute(&repo_root, &run_id)
                .map_err(|error| format!("operated assurance run failed: {error}"))?;
            println!(
                "Operated assurance run '{run_id}' wrote {} source rows, {} history rows, {} alert rules, {} exact blockers, and {} failed rows to {}.",
                summary.source_evidence_row_count,
                summary.multi_run_history_row_count,
                summary.evaluated_alert_rule_count,
                summary.exact_service_blocker_count,
                summary.failed_row_count,
                summary.artifact_root
            );
        }
        "diversity-seam" => {
            let runner = DiversitySeamRunner::new();
            let summary = runner
                .execute(&repo_root, &run_id)
                .map_err(|error| format!("diversity seam run failed: {error}"))?;
            println!(
                "Diversity seam run '{run_id}' wrote {} source rows, {} diversity rows, {} OxFml seam-watch rows, {} exact blockers, and {} failed rows to {}.",
                summary.source_evidence_row_count,
                summary.diversity_disposition_row_count,
                summary.seam_watch_row_count,
                summary.exact_blocker_count,
                summary.failed_row_count,
                summary.artifact_root
            );
        }
        "oxfml-seam" => {
            let runner = OxFmlSeamRunner::new();
            let summary = runner
                .execute(&repo_root, &run_id)
                .map_err(|error| format!("OxFml seam run failed: {error}"))?;
            println!(
                "OxFml seam run '{run_id}' wrote {} source rows, {} surface rows, {} publication/display rows, {} callable rows, {} exact blockers, and {} failed rows to {}.",
                summary.source_evidence_row_count,
                summary.surface_row_count,
                summary.publication_display_row_count,
                summary.callable_metadata_row_count,
                summary.exact_blocker_count,
                summary.failed_row_count,
                summary.artifact_root
            );
        }
        "continuous-assurance" => {
            let runner = ContinuousAssuranceRunner::new();
            let summary = runner
                .execute(&repo_root, &run_id)
                .map_err(|error| format!("continuous assurance run failed: {error}"))?;
            if run_id.starts_with("w037-") {
                println!(
                    "Continuous assurance run '{run_id}' wrote decision '{}' with {} source rows, {} differential rows, {} history rows, {} service-readiness criteria ({} blocked), and {} no-promotion reasons to {}.",
                    summary.decision_status,
                    summary.source_evidence_row_count,
                    summary.cross_engine_gate_row_count,
                    summary.history_window_row_count,
                    summary.service_readiness_criteria_count,
                    summary.service_readiness_blocked_count,
                    summary.no_promotion_reason_count,
                    summary.artifact_root
                );
            } else if run_id.starts_with("w036-") {
                println!(
                    "Continuous assurance run '{run_id}' wrote decision '{}' with {} source rows, {} differential rows, {} history rows, {} threshold rules, {} quarantine rules, and {} no-promotion reasons to {}.",
                    summary.decision_status,
                    summary.source_evidence_row_count,
                    summary.cross_engine_gate_row_count,
                    summary.history_window_row_count,
                    summary.regression_threshold_count,
                    summary.quarantine_rule_count,
                    summary.no_promotion_reason_count,
                    summary.artifact_root
                );
            } else {
                println!(
                    "Continuous assurance run '{run_id}' wrote decision '{}' with {} source rows, {} differential rows, and {} no-promotion reasons to {}.",
                    summary.decision_status,
                    summary.source_evidence_row_count,
                    summary.cross_engine_gate_row_count,
                    summary.no_promotion_reason_count,
                    summary.artifact_root
                );
            }
        }
        "retained-failures" => {
            let runner = TraceCalcRetainedFailureRunner::new();
            let summary = runner
                .execute_manifest(&repo_root, &run_id)
                .map_err(|error| format!("retained-failure run failed: {error}"))?;
            println!(
                "TraceCalc retained-failure run '{run_id}' wrote {} cases to {}.",
                summary.case_count, summary.artifact_root
            );
        }
        "oxfml-bridge" => {
            let runner = OxFmlFixtureBridgeRunner::new();
            let summary = runner
                .execute(&repo_root, &run_id)
                .map_err(|error| format!("OxFml fixture bridge run failed: {error}"))?;
            println!(
                "OxFml fixture bridge run '{run_id}' projected {} fixture cases across {} families to {}.",
                summary.fixture_case_count, summary.family_count, summary.artifact_root
            );
        }
        "upstream-host" => {
            let runner = UpstreamHostRunner::new();
            let summary = runner
                .execute(&repo_root, &run_id)
                .map_err(|error| format!("upstream-host run failed: {error}"))?;
            println!(
                "Upstream-host direct OxFml run '{run_id}' wrote {} cases ({} direct OxFml, {} LET/LAMBDA, {} W073 formatting guard, {} mismatches) to {}.",
                summary.fixture_case_count,
                summary.direct_oxfml_case_count,
                summary.let_lambda_case_count,
                summary.w073_typed_rule_case_count,
                summary.expectation_mismatch_count,
                summary.artifact_root
            );
        }
        "independent-conformance" => {
            let runner = IndependentConformanceRunner::new();
            let summary = runner
                .execute(&repo_root, &run_id)
                .map_err(|error| format!("independent conformance run failed: {error}"))?;
            if summary.w036_differential_row_count > 0 {
                println!(
                    "Independent conformance run '{run_id}' wrote {} comparison rows, {} W036 diversity rows, {} W036 differential rows, and {} promotion blockers to {}.",
                    summary.comparison_row_count,
                    summary.w036_diversity_row_count,
                    summary.w036_differential_row_count,
                    summary.w036_promotion_blocker_count,
                    summary.artifact_root
                );
            } else {
                println!(
                    "Independent conformance run '{run_id}' wrote {} comparison rows to {}.",
                    summary.comparison_row_count, summary.artifact_root
                );
            }
        }
        "pack-capability" => {
            let runner = PackCapabilityRunner::new();
            let summary = runner
                .execute(&repo_root, &run_id)
                .map_err(|error| format!("pack capability run failed: {error}"))?;
            println!(
                "Pack capability run '{run_id}' wrote decision '{}' with {} satisfied inputs, {} blockers, and highest honest capability '{}' to {}.",
                summary.decision_status,
                summary.satisfied_input_count,
                summary.blocker_count,
                summary.highest_honest_capability,
                summary.artifact_root
            );
        }
        "scale-semantic-binding" => {
            let runner = ScaleSemanticBindingRunner::new();
            let summary = runner
                .execute(&repo_root, &run_id)
                .map_err(|error| format!("scale semantic binding run failed: {error}"))?;
            println!(
                "Scale semantic binding run '{run_id}' validated {} scale rows, {} signature rows, {} replay binding rows to {}.",
                summary.validated_scale_run_count,
                summary.scale_signature_row_count,
                summary.replay_binding_row_count,
                summary.artifact_root
            );
        }
        _ => {
            let runner = TraceCalcRunner::new();
            let summary = runner
                .execute_manifest(&repo_root, &run_id, None, None)
                .map_err(|error| format!("tracecalc run failed: {error}"))?;
            println!(
                "TraceCalc Rust run '{run_id}' wrote {} scenarios to {}.",
                summary.scenario_count, summary.artifact_root
            );
        }
    }

    Ok(())
}

fn ensure_repo_root(repo_root: &Path) -> Result<(), String> {
    let manifest_path = repo_root.join("docs/test-corpus/core-engine/tracecalc/MANIFEST.json");
    if manifest_path.exists() {
        Ok(())
    } else {
        Err(format!(
            "current directory is not the OxCalc repo root: missing {}",
            manifest_path.display()
        ))
    }
}

fn ensure_retained_failure_root(repo_root: &Path) -> Result<(), String> {
    let manifest_path =
        repo_root.join("docs/test-fixtures/core-engine/tracecalc-retained-failures/MANIFEST.json");
    if manifest_path.exists() {
        Ok(())
    } else {
        Err(format!(
            "current directory is not the OxCalc repo root for retained-failure runs: missing {}",
            manifest_path.display()
        ))
    }
}

fn ensure_treecalc_root(repo_root: &Path) -> Result<(), String> {
    let manifest_path = repo_root.join("docs/test-fixtures/core-engine/treecalc/MANIFEST.json");
    if manifest_path.exists() {
        Ok(())
    } else {
        Err(format!(
            "current directory is not the OxCalc repo root for treecalc runs: missing {}",
            manifest_path.display()
        ))
    }
}

fn ensure_oxfml_bridge_root(repo_root: &Path) -> Result<(), String> {
    let fixture_path =
        repo_root.join("../OxFml/crates/oxfml_core/tests/fixtures/fec_commit_replay_cases.json");
    if fixture_path.exists() {
        Ok(())
    } else {
        Err(format!(
            "current directory is not the OxCalc repo root for OxFml fixture bridge runs: missing {}",
            fixture_path.display()
        ))
    }
}

fn ensure_upstream_host_root(repo_root: &Path) -> Result<(), String> {
    let manifest_path =
        repo_root.join("docs/test-fixtures/core-engine/upstream-host/MANIFEST.json");
    if manifest_path.exists() {
        Ok(())
    } else {
        Err(format!(
            "current directory is not the OxCalc repo root for upstream-host runs: missing {}",
            manifest_path.display()
        ))
    }
}

fn ensure_independent_conformance_root(repo_root: &Path) -> Result<(), String> {
    let trace_run = repo_root.join(
        "docs/test-runs/core-engine/tracecalc-reference-machine/w034-tracecalc-oracle-deepening-001/run_summary.json",
    );
    let tree_run = repo_root.join(
        "docs/test-runs/core-engine/treecalc-local/w034-independent-conformance-treecalc-001/run_summary.json",
    );
    if trace_run.exists() && tree_run.exists() {
        Ok(())
    } else {
        Err(format!(
            "current directory is not ready for independent conformance: missing {} or {}",
            trace_run.display(),
            tree_run.display()
        ))
    }
}

fn ensure_implementation_conformance_root(repo_root: &Path) -> Result<(), String> {
    let independent_summary = repo_root.join(
        "docs/test-runs/core-engine/independent-conformance/w034-independent-conformance-001/run_summary.json",
    );
    let matrix_summary = repo_root.join(
        "docs/test-runs/core-engine/tracecalc-reference-machine/w035-tracecalc-oracle-matrix-001/oracle-matrix/run_summary.json",
    );
    if independent_summary.exists() && matrix_summary.exists() {
        Ok(())
    } else {
        Err(format!(
            "current directory is not ready for implementation conformance: missing {} or {}",
            independent_summary.display(),
            matrix_summary.display()
        ))
    }
}

fn ensure_continuous_assurance_root(repo_root: &Path) -> Result<(), String> {
    let scale_summary = repo_root.join(
        "docs/test-runs/core-engine/metamorphic-scale-semantic-binding/w034-continuous-scale-gate-binding-001/run_summary.json",
    );
    let implementation_summary = repo_root.join(
        "docs/test-runs/core-engine/implementation-conformance/w035-implementation-conformance-hardening-001/run_summary.json",
    );
    if scale_summary.exists() && implementation_summary.exists() {
        Ok(())
    } else {
        Err(format!(
            "current directory is not ready for continuous assurance: missing {} or {}",
            scale_summary.display(),
            implementation_summary.display()
        ))
    }
}

fn ensure_formal_assurance_root(repo_root: &Path) -> Result<(), String> {
    let formal_summary = repo_root.join(
        "docs/test-runs/core-engine/formal-inventory/w037-proof-model-closure-001/run_summary.json",
    );
    let conformance_blockers = repo_root.join(
        "docs/test-runs/core-engine/implementation-conformance/w038-optimized-core-conformance-disposition-001/w038_exact_remaining_blocker_register.json",
    );
    if formal_summary.exists() && conformance_blockers.exists() {
        Ok(())
    } else {
        Err(format!(
            "current directory is not ready for formal assurance: missing {} or {}",
            formal_summary.display(),
            conformance_blockers.display()
        ))
    }
}

fn ensure_stage2_replay_root(repo_root: &Path) -> Result<(), String> {
    let stage2_requirements = repo_root.join(
        "docs/test-runs/core-engine/stage2-criteria/w037-stage2-deterministic-replay-criteria-001/semantic_equivalence_requirements.json",
    );
    let treecalc_case = repo_root.join(
        "docs/test-runs/core-engine/treecalc-local/w037-optimized-core-conformance-treecalc-001/cases/tc_local_w034_independent_order_equiv_001/result.json",
    );
    if stage2_requirements.exists() && treecalc_case.exists() {
        Ok(())
    } else {
        Err(format!(
            "current directory is not ready for Stage 2 replay: missing {} or {}",
            stage2_requirements.display(),
            treecalc_case.display()
        ))
    }
}

fn ensure_operated_assurance_root(repo_root: &Path) -> Result<(), String> {
    let w037_service_readiness = repo_root.join(
        "docs/test-runs/core-engine/continuous-assurance/w037-operated-assurance-service-pilot-001/service/service_readiness.json",
    );
    let w038_stage2 = repo_root.join(
        "docs/test-runs/core-engine/stage2-replay/w038-stage2-partition-replay-001/run_summary.json",
    );
    if w037_service_readiness.exists() && w038_stage2.exists() {
        Ok(())
    } else {
        Err(format!(
            "current directory is not ready for operated assurance: missing {} or {}",
            w037_service_readiness.display(),
            w038_stage2.display()
        ))
    }
}

fn ensure_diversity_seam_root(repo_root: &Path) -> Result<(), String> {
    let w036_independent = repo_root.join(
        "docs/test-runs/core-engine/independent-conformance/w036-independent-diversity-differential-001/run_summary.json",
    );
    let w037_direct_oxfml = repo_root.join(
        "docs/test-runs/core-engine/upstream-host/w037-direct-oxfml-evaluator-001/run_summary.json",
    );
    let w038_operated = repo_root.join(
        "docs/test-runs/core-engine/operated-assurance/w038-operated-assurance-alert-quarantine-001/run_summary.json",
    );
    if w036_independent.exists() && w037_direct_oxfml.exists() && w038_operated.exists() {
        Ok(())
    } else {
        Err(format!(
            "current directory is not ready for diversity seam: missing {}, {}, or {}",
            w036_independent.display(),
            w037_direct_oxfml.display(),
            w038_operated.display()
        ))
    }
}

fn ensure_oxfml_seam_root(repo_root: &Path) -> Result<(), String> {
    let w039_upstream_host = repo_root.join(
        "docs/test-runs/core-engine/upstream-host/w039-oxfml-seam-breadth-callable-metadata-001/run_summary.json",
    );
    let w039_callable_blockers = repo_root.join(
        "docs/test-runs/core-engine/implementation-conformance/w039-optimized-core-exact-blocker-disposition-001/w039_exact_remaining_blocker_register.json",
    );
    let oxfml_notes = repo_root.join("../OxFml/docs/upstream/NOTES_FOR_OXCALC.md");
    if w039_upstream_host.exists() && w039_callable_blockers.exists() && oxfml_notes.exists() {
        Ok(())
    } else {
        Err(format!(
            "current directory is not ready for OxFml seam: missing {}, {}, or {}",
            w039_upstream_host.display(),
            w039_callable_blockers.display(),
            oxfml_notes.display()
        ))
    }
}

fn ensure_pack_capability_root(repo_root: &Path) -> Result<(), String> {
    let retained_decision = repo_root.join(
        "docs/test-runs/core-engine/tracecalc-retained-failures/w023-sequence3-program-decision/replay-appliance/validation/pack_grade_decision.json",
    );
    let independent_summary = repo_root.join(
        "docs/test-runs/core-engine/independent-conformance/post-w033-independent-conformance-001/run_summary.json",
    );
    if retained_decision.exists() && independent_summary.exists() {
        Ok(())
    } else {
        Err(format!(
            "current directory is not ready for pack capability decision: missing {} or {}",
            retained_decision.display(),
            independent_summary.display()
        ))
    }
}

fn ensure_scale_semantic_binding_root(repo_root: &Path) -> Result<(), String> {
    let scale_summary = repo_root
        .join("docs/test-runs/core-engine/treecalc-scale/million_grid_r1/run_summary.json");
    let trace_scale_seed = repo_root.join(
        "docs/test-runs/core-engine/tracecalc-reference-machine/post-w033-let-lambda-carrier-witness-001/scenarios/tc_scale_chain_seed_001/result.json",
    );
    if scale_summary.exists() && trace_scale_seed.exists() {
        Ok(())
    } else {
        Err(format!(
            "current directory is not ready for scale semantic binding: missing {} or {}",
            scale_summary.display(),
            trace_scale_seed.display()
        ))
    }
}

fn parse_treecalc_scale_options(
    profile: TreeCalcScaleProfile,
    run_id: String,
    mut args: impl Iterator<Item = String>,
) -> Result<TreeCalcScaleOptions, String> {
    let mut options = TreeCalcScaleOptions::default_for(profile, run_id);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--rows" => options.rows = parse_usize_flag(&arg, &mut args)?,
            "--cols" => options.cols = parse_usize_flag(&arg, &mut args)?,
            "--nodes" | "--node-count" => options.node_count = parse_usize_flag(&arg, &mut args)?,
            "--fanout" => options.fanout = parse_usize_flag(&arg, &mut args)?,
            "--left-delta" => options.left_delta = parse_i64_flag(&arg, &mut args)?,
            "--top-delta" => options.top_delta = parse_i64_flag(&arg, &mut args)?,
            "--selector-period" => options.selector_period = parse_usize_flag(&arg, &mut args)?,
            "--recalc-rounds" => options.recalc_rounds = parse_usize_flag(&arg, &mut args)?,
            _ => {
                return Err(format!(
                    "unknown treecalc-scale option '{arg}'. {}",
                    treecalc_scale_usage()
                ));
            }
        }
    }

    Ok(options)
}

fn parse_usize_flag(flag: &str, args: &mut impl Iterator<Item = String>) -> Result<usize, String> {
    let Some(value) = args.next() else {
        return Err(format!(
            "missing value for {flag}. {}",
            treecalc_scale_usage()
        ));
    };
    value
        .parse::<usize>()
        .map_err(|error| format!("invalid value for {flag}: {value} ({error})"))
}

fn parse_i64_flag(flag: &str, args: &mut impl Iterator<Item = String>) -> Result<i64, String> {
    let Some(value) = args.next() else {
        return Err(format!(
            "missing value for {flag}. {}",
            treecalc_scale_usage()
        ));
    };
    value
        .parse::<i64>()
        .map_err(|error| format!("invalid value for {flag}: {value} ({error})"))
}

fn treecalc_scale_usage() -> String {
    "usage: oxcalc-tracecalc-cli treecalc-scale <grid-cross-sum|fanout-bands|dynamic-indirect-stripes|relative-rebind-churn> <run-id> [--rows N] [--cols N] [--nodes N] [--fanout N] [--left-delta N] [--top-delta N] [--selector-period N] [--recalc-rounds N]".to_string()
}
