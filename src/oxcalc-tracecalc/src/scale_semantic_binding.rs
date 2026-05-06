#![forbid(unsafe_code)]

//! Post-W033 scale and metamorphic semantic-binding packet emission.

use std::fs;
use std::path::Path;

use serde_json::{Value, json};
use thiserror::Error;

const SCALE_SEMANTIC_RUN_SUMMARY_SCHEMA_V1: &str = "oxcalc.scale_semantic_binding.run_summary.v1";
const SCALE_SEMANTIC_EVIDENCE_SCHEMA_V1: &str = "oxcalc.scale_semantic_binding.evidence_index.v1";
const SCALE_SIGNATURE_DIFF_SCHEMA_V1: &str =
    "oxcalc.scale_semantic_binding.scale_signature_diff.v1";
const SCALE_REPLAY_BINDING_SCHEMA_V1: &str =
    "oxcalc.scale_semantic_binding.replay_conformance_bindings.v1";
const SCALE_NO_PROMOTION_SCHEMA_V1: &str = "oxcalc.scale_semantic_binding.no_promotion_decision.v1";
const SCALE_BUNDLE_MANIFEST_SCHEMA_V1: &str = "oxcalc.scale_semantic_binding.bundle_manifest.v1";
const SCALE_BUNDLE_VALIDATION_SCHEMA_V1: &str =
    "oxcalc.scale_semantic_binding.bundle_validation.v1";
const SCALE_CONTINUOUS_CRITERIA_SCHEMA_V1: &str =
    "oxcalc.scale_semantic_binding.continuous_criteria.v1";

const SCALE_RUN_IDS: &[&str] = &[
    "million_grid_r1",
    "million_grid_r2",
    "million_indirect_r1",
    "million_fanout_f8_r1",
    "million_fanout_f8_calc1024_r1",
    "million_relative_rebind_f8_r1",
    "million_fanout_f16_r1",
];
const POST_W033_TRACECALC_RUN_ID: &str = "post-w033-let-lambda-carrier-witness-001";
const TRACECALC_SCALE_SCENARIO_ID: &str = "tc_scale_chain_seed_001";
const POST_W033_INDEPENDENT_CONFORMANCE_RUN_ID: &str = "post-w033-independent-conformance-001";
const POST_W033_PACK_CAPABILITY_RUN_ID: &str = "post-w033-pack-capability-decision-001";
const W034_TRACECALC_RUN_ID: &str = "w034-tracecalc-oracle-deepening-001";
const W034_INDEPENDENT_CONFORMANCE_RUN_ID: &str = "w034-independent-conformance-001";
const W034_PACK_CAPABILITY_RUN_ID: &str = "w034-pack-capability-gate-binding-001";
const W037_TRACECALC_RUN_ID: &str = "w037-tracecalc-observable-closure-001";
const W043_PACK_CAPABILITY_RUN_ID: &str =
    "w043-pack-grade-replay-governance-c5-release-reassessment-001";
const W044_PROFILE_ID: &str = "w044_release_scale_replay_performance_scaling";
const W044_IMPLEMENTATION_CONFORMANCE_SUMMARY_PATH: &str = "docs/test-runs/core-engine/implementation-conformance/w044-optimized-core-dynamic-transition-callable-metadata-001/run_summary.json";
const W044_RUST_FORMAL_ASSURANCE_SUMMARY_PATH: &str = "docs/test-runs/core-engine/formal-assurance/w044-rust-totality-refinement-panic-surface-expansion-001/run_summary.json";
const W044_LEAN_TLA_FORMAL_ASSURANCE_SUMMARY_PATH: &str = "docs/test-runs/core-engine/formal-assurance/w044-lean-tla-unbounded-fairness-full-verification-expansion-001/run_summary.json";
const W044_STAGE2_REPLAY_SUMMARY_PATH: &str = "docs/test-runs/core-engine/stage2-replay/w044-stage2-production-partition-analyzer-scheduler-equivalence-001/run_summary.json";
const W044_OPERATED_ASSURANCE_SUMMARY_PATH: &str = "docs/test-runs/core-engine/operated-assurance/w044-operated-assurance-retained-history-witness-slo-alert-service-001/run_summary.json";
const W044_DIVERSITY_SEAM_SUMMARY_PATH: &str = "docs/test-runs/core-engine/diversity-seam/w044-independent-evaluator-breadth-mismatch-quarantine-differential-service-001/run_summary.json";
const W044_OXFML_SEAM_SUMMARY_PATH: &str = "docs/test-runs/core-engine/oxfml-seam/w044-oxfml-public-migration-typed-formatting-callable-registered-external-001/run_summary.json";
const W044_GUARD_ARTIFACT_PATHS: &[&str] = &[
    W044_IMPLEMENTATION_CONFORMANCE_SUMMARY_PATH,
    W044_RUST_FORMAL_ASSURANCE_SUMMARY_PATH,
    W044_LEAN_TLA_FORMAL_ASSURANCE_SUMMARY_PATH,
    W044_STAGE2_REPLAY_SUMMARY_PATH,
    W044_OPERATED_ASSURANCE_SUMMARY_PATH,
    W044_DIVERSITY_SEAM_SUMMARY_PATH,
    W044_OXFML_SEAM_SUMMARY_PATH,
];
const W044_REQUIRED_PHASE_TIMINGS: &[&str] = &[
    "model_build_structural_snapshot_and_formula_catalog",
    "dependency_descriptor_lowering",
    "dependency_graph_build_and_cycle_scan",
    "soft_reference_update_rebind_seed_derivation",
    "invalidation_closure_derivation",
    "synthetic_closed_form_recalc",
    "validation_checks",
];
const W034_FORMAL_GATE_ARTIFACTS: &[&str] = &[
    "docs/spec/core-engine/w034-formalization/W034_LEAN_PROOF_FAMILY_DEEPENING.md",
    "docs/spec/core-engine/w034-formalization/W034_TLA_MODEL_FAMILY_AND_CONTENTION_PRECONDITIONS.md",
    "docs/spec/core-engine/w034-formalization/W034_PACK_CAPABILITY_AND_CONTINUOUS_SCALE_GATE_BINDING.md",
    "formal/lean/OxCalc/CoreEngine/W034PublicationFences.lean",
    "formal/lean/OxCalc/CoreEngine/W034DependenciesOverlays.lean",
    "formal/lean/OxCalc/CoreEngine/W034LetLambdaReplay.lean",
    "formal/lean/OxCalc/CoreEngine/W034RefinementObligations.lean",
    "formal/tla/CoreEngineW034Interleavings.tla",
    "formal/tla/CoreEngineW034Interleavings.smoke.cfg",
];

#[derive(Debug, Error)]
pub enum ScaleSemanticBindingError {
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
pub struct ScaleSemanticBindingRunSummary {
    pub run_id: String,
    pub schema_version: String,
    pub scale_run_row_count: usize,
    pub validated_scale_run_count: usize,
    pub scale_signature_row_count: usize,
    pub replay_binding_row_count: usize,
    pub missing_artifact_count: usize,
    pub unexpected_mismatch_count: usize,
    pub no_promotion_reason_count: usize,
    pub artifact_root: String,
}

#[derive(Debug, Clone, Default)]
pub struct ScaleSemanticBindingRunner;

#[derive(Debug, Clone)]
struct ArtifactRead {
    relative_path: String,
    value: Option<Value>,
}

#[derive(Debug, Clone)]
struct ScaleRunObservation {
    run_id: &'static str,
    value: Option<Value>,
    row: Value,
    missing_artifacts: Vec<String>,
    failures: Vec<String>,
}

#[derive(Debug, Clone, Default)]
struct Evaluation {
    scale_rows: Vec<Value>,
    signature_rows: Vec<Value>,
    replay_rows: Vec<Value>,
    missing_artifacts: Vec<String>,
    unexpected_mismatches: Vec<String>,
    no_promotion_reasons: Vec<String>,
    validated_scale_runs: usize,
}

#[derive(Debug, Clone)]
struct ScaleSemanticProfile {
    profile_id: &'static str,
    family_packet: &'static str,
    tracecalc_run_id: &'static str,
    tracecalc_scale_scenario_id: &'static str,
    independent_conformance_run_id: &'static str,
    pack_capability_run_id: &'static str,
    formal_gate_artifacts: &'static [&'static str],
    additional_no_promotion_reasons: &'static [&'static str],
}

impl ScaleSemanticBindingRunner {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    pub fn execute(
        &self,
        repo_root: &Path,
        run_id: &str,
    ) -> Result<ScaleSemanticBindingRunSummary, ScaleSemanticBindingError> {
        let artifact_root = repo_root.join(format!(
            "docs/test-runs/core-engine/metamorphic-scale-semantic-binding/{run_id}"
        ));
        let relative_artifact_root = relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "metamorphic-scale-semantic-binding",
            run_id,
        ]);

        if artifact_root.exists() {
            fs::remove_dir_all(&artifact_root).map_err(|source| {
                ScaleSemanticBindingError::RemoveDirectory {
                    path: artifact_root.display().to_string(),
                    source,
                }
            })?;
        }

        create_directory(&artifact_root)?;
        create_directory(&artifact_root.join("decision"))?;
        create_directory(&artifact_root.join("differentials"))?;
        create_directory(&artifact_root.join("evidence"))?;
        create_directory(&artifact_root.join("replay-appliance"))?;
        create_directory(&artifact_root.join("replay-appliance/validation"))?;

        let profile = scale_semantic_profile(run_id);
        let evaluation = evaluate(repo_root, &profile)?;

        write_json(
            &artifact_root.join("evidence/scale_semantic_evidence_index.json"),
            &json!({
                "schema_version": SCALE_SEMANTIC_EVIDENCE_SCHEMA_V1,
                "run_id": run_id,
                "evidence_profile": profile.profile_id,
                "artifact_root": relative_artifact_root,
                "family_packet": profile.family_packet,
                "scale_run_row_count": evaluation.scale_rows.len(),
                "validated_scale_run_count": evaluation.validated_scale_runs,
                "missing_artifact_count": evaluation.missing_artifacts.len(),
                "unexpected_mismatch_count": evaluation.unexpected_mismatches.len(),
                "rows": evaluation.scale_rows,
            }),
        )?;

        write_json(
            &artifact_root.join("differentials/scale_signature_differential.json"),
            &json!({
                "schema_version": SCALE_SIGNATURE_DIFF_SCHEMA_V1,
                "run_id": run_id,
                "evidence_profile": profile.profile_id,
                "row_count": evaluation.signature_rows.len(),
                "unexpected_mismatch_count": count_failure_rows(&evaluation.signature_rows),
                "rows": evaluation.signature_rows,
            }),
        )?;

        write_json(
            &artifact_root.join("replay_conformance_bindings.json"),
            &json!({
                "schema_version": SCALE_REPLAY_BINDING_SCHEMA_V1,
                "run_id": run_id,
                "evidence_profile": profile.profile_id,
                "tracecalc_reference_run_id": profile.tracecalc_run_id,
                "tracecalc_scale_scenario_id": profile.tracecalc_scale_scenario_id,
                "independent_conformance_run_id": profile.independent_conformance_run_id,
                "pack_capability_run_id": profile.pack_capability_run_id,
                "row_count": evaluation.replay_rows.len(),
                "unexpected_mismatch_count": count_failure_rows(&evaluation.replay_rows),
                "rows": evaluation.replay_rows,
            }),
        )?;

        write_json(
            &artifact_root.join("decision/scale_no_promotion_decision.json"),
            &json!({
                "schema_version": SCALE_NO_PROMOTION_SCHEMA_V1,
                "run_id": run_id,
                "evidence_profile": profile.profile_id,
                "decision_status": if evaluation.missing_artifacts.is_empty() && evaluation.unexpected_mismatches.is_empty() {
                    "semantic_binding_recorded_without_performance_promotion"
                } else {
                    "semantic_binding_has_unresolved_evidence"
                },
                "scale_semantic_evidence_recorded": evaluation.missing_artifacts.is_empty() && evaluation.unexpected_mismatches.is_empty(),
                "performance_claim_promoted": false,
                "pack_capability_promoted": false,
                "continuous_scale_assurance_promoted": false,
                "no_promotion_reason_ids": evaluation.no_promotion_reasons,
                "semantic_equivalence_statement": "This runner reads existing scale, replay, conformance, and pack-decision artifacts and emits binding evidence only. It does not change coordinator scheduling, invalidation, recalc, publication, reject, or evaluator behavior.",
            }),
        )?;

        write_json(
            &artifact_root.join("decision/continuous_scale_assurance_criteria.json"),
            &continuous_scale_criteria(run_id, &relative_artifact_root, &profile, &evaluation),
        )?;

        let required_artifacts = required_artifacts(run_id, &profile);
        write_json(
            &artifact_root.join("replay-appliance/bundle_manifest.json"),
            &json!({
                "schema_version": SCALE_BUNDLE_MANIFEST_SCHEMA_V1,
                "run_id": run_id,
                "evidence_profile": profile.profile_id,
                "artifact_root": relative_artifact_root,
                "claimed_capability": "scale_semantic_binding_packet",
                "excluded_capabilities": [
                    "performance_correctness_proof",
                    "continuous_scale_assurance",
                    "pack_grade_replay",
                    "stage2_contention_readiness"
                ],
                "required_artifacts": required_artifacts,
            }),
        )?;

        let summary = ScaleSemanticBindingRunSummary {
            run_id: run_id.to_string(),
            schema_version: SCALE_SEMANTIC_RUN_SUMMARY_SCHEMA_V1.to_string(),
            scale_run_row_count: evaluation.scale_rows.len(),
            validated_scale_run_count: evaluation.validated_scale_runs,
            scale_signature_row_count: evaluation.signature_rows.len(),
            replay_binding_row_count: evaluation.replay_rows.len(),
            missing_artifact_count: evaluation.missing_artifacts.len(),
            unexpected_mismatch_count: evaluation.unexpected_mismatches.len(),
            no_promotion_reason_count: evaluation.no_promotion_reasons.len(),
            artifact_root: relative_artifact_root.clone(),
        };
        write_json(
            &artifact_root.join("run_summary.json"),
            &json!({
                "schema_version": summary.schema_version,
                "run_id": summary.run_id,
                "evidence_profile": profile.profile_id,
                "scale_run_row_count": summary.scale_run_row_count,
                "validated_scale_run_count": summary.validated_scale_run_count,
                "scale_signature_row_count": summary.scale_signature_row_count,
                "replay_binding_row_count": summary.replay_binding_row_count,
                "missing_artifact_count": summary.missing_artifact_count,
                "unexpected_mismatch_count": summary.unexpected_mismatch_count,
                "no_promotion_reason_count": summary.no_promotion_reason_count,
                "artifact_root": summary.artifact_root,
                "scale_semantic_evidence_index_path": format!("{relative_artifact_root}/evidence/scale_semantic_evidence_index.json"),
                "scale_signature_differential_path": format!("{relative_artifact_root}/differentials/scale_signature_differential.json"),
                "replay_conformance_bindings_path": format!("{relative_artifact_root}/replay_conformance_bindings.json"),
                "decision_path": format!("{relative_artifact_root}/decision/scale_no_promotion_decision.json"),
                "continuous_scale_criteria_path": format!("{relative_artifact_root}/decision/continuous_scale_assurance_criteria.json"),
                "bundle_validation_path": format!("{relative_artifact_root}/replay-appliance/validation/bundle_validation.json"),
            }),
        )?;

        let validation_path =
            artifact_root.join("replay-appliance/validation/bundle_validation.json");
        write_json(
            &validation_path,
            &json!({
                "schema_version": SCALE_BUNDLE_VALIDATION_SCHEMA_V1,
                "run_id": run_id,
                "status": "pending_final_validation_write",
            }),
        )?;
        let missing_required_paths = required_artifacts
            .iter()
            .filter(|relative_path| !repo_root.join(relative_path.as_str()).exists())
            .cloned()
            .collect::<Vec<_>>();
        write_json(
            &validation_path,
            &json!({
                "schema_version": SCALE_BUNDLE_VALIDATION_SCHEMA_V1,
                "run_id": run_id,
                "status": if missing_required_paths.is_empty() { "bundle_valid" } else { "missing_required_artifacts" },
                "missing_paths": missing_required_paths,
                "validated_required_artifact_count": required_artifacts.len(),
                "unexpected_mismatch_count": evaluation.unexpected_mismatches.len(),
                "source_missing_artifact_count": evaluation.missing_artifacts.len(),
            }),
        )?;

        Ok(summary)
    }
}

fn evaluate(
    repo_root: &Path,
    profile: &ScaleSemanticProfile,
) -> Result<Evaluation, ScaleSemanticBindingError> {
    let observations = SCALE_RUN_IDS
        .iter()
        .map(|run_id| scale_run_observation(repo_root, run_id))
        .collect::<Result<Vec<_>, _>>()?;

    let mut evaluation = Evaluation {
        scale_rows: observations
            .iter()
            .map(|observation| observation.row.clone())
            .collect(),
        validated_scale_runs: observations
            .iter()
            .filter(|observation| {
                observation.failures.is_empty() && observation.missing_artifacts.is_empty()
            })
            .count(),
        ..Evaluation::default()
    };
    for observation in &observations {
        evaluation
            .missing_artifacts
            .extend(observation.missing_artifacts.clone());
        evaluation.unexpected_mismatches.extend(
            observation
                .failures
                .iter()
                .map(|failure| format!("scale_run:{}:{failure}", observation.run_id)),
        );
    }

    evaluation.signature_rows = scale_signature_rows(&observations);
    if is_w044_profile(profile) {
        evaluation
            .signature_rows
            .push(w044_phase_timing_split_row(&observations));
    }
    for row in &evaluation.signature_rows {
        collect_row_failures(row, &mut evaluation.unexpected_mismatches);
    }

    evaluation.replay_rows = replay_binding_rows(repo_root, profile)?;
    for row in &evaluation.replay_rows {
        collect_row_failures(row, &mut evaluation.unexpected_mismatches);
        collect_row_missing(row, &mut evaluation.missing_artifacts);
    }

    evaluation.no_promotion_reasons = vec![
        "scale.performance.measurement_not_a_correctness_proof".to_string(),
        "scale.performance.single_day_baseline_not_continuous_assurance".to_string(),
        "scale.performance.semantic_binding_not_scheduler_policy_promotion".to_string(),
        "scale.performance.not_pack_grade_replay".to_string(),
        "scale.performance.stage2_contention_not_promoted".to_string(),
    ];
    evaluation.no_promotion_reasons.extend(
        profile
            .additional_no_promotion_reasons
            .iter()
            .map(|reason| (*reason).to_string()),
    );

    Ok(evaluation)
}

fn scale_run_observation(
    repo_root: &Path,
    run_id: &'static str,
) -> Result<ScaleRunObservation, ScaleSemanticBindingError> {
    let relative_path = scale_run_summary_path(run_id);
    let value = read_json(repo_root, &relative_path)?;
    let mut failures = Vec::new();
    let mut missing_artifacts = Vec::new();
    if value.is_none() {
        missing_artifacts.push(relative_path.clone());
    }

    if let Some(value) = &value {
        if !bool_pointer(value, "/validation/passed") {
            failures.push("validation_summary_not_passed".to_string());
        }
        if !all_validation_checks_passed(value) {
            failures.push("one_or_more_validation_checks_failed".to_string());
        }
        if number_pointer(value, "/model/node_count") < 1_000_000 {
            failures.push("node_count_below_million_scale_floor".to_string());
        }
        if string_pointer(value, "/validation/synthetic_recalc/expected_after_sum")
            != string_pointer(value, "/validation/synthetic_recalc/observed_after_sum")
        {
            failures.push("synthetic_after_sum_mismatch".to_string());
        }
        if string_pointer(value, "/validation/synthetic_recalc/expected_delta_sum")
            != string_pointer(value, "/validation/synthetic_recalc/observed_delta_sum")
        {
            failures.push("synthetic_delta_sum_mismatch".to_string());
        }
    }

    let row = json!({
        "run_id": run_id,
        "artifact_path": relative_path,
        "profile": value.as_ref().map_or("<missing>".to_string(), |value| string_pointer(value, "/profile")),
        "evidence_state": if missing_artifacts.is_empty() && failures.is_empty() {
            "validated_closed_form_scale_semantic_evidence"
        } else if missing_artifacts.is_empty() {
            "semantic_validation_mismatch"
        } else {
            "missing_artifact"
        },
        "missing_artifacts": missing_artifacts,
        "failures": failures,
        "semantic_surfaces": value.as_ref().map_or_else(|| json!({}), |value| json!({
            "validation_passed": bool_pointer(value, "/validation/passed"),
            "node_count": number_pointer(value, "/model/node_count"),
            "formula_count": number_pointer(value, "/model/formula_count"),
            "dependency_descriptor_count": number_pointer(value, "/model/dependency_descriptor_count"),
            "dependency_edge_count": number_pointer(value, "/model/dependency_edge_count"),
            "dependency_diagnostic_count": number_pointer(value, "/model/dependency_diagnostic_count"),
            "invalidation_impacted_count": number_pointer(value, "/model/invalidation_impacted_count"),
            "soft_reference_rebind_seed_count": number_pointer(value, "/validation/soft_reference_update/rebind_seed_count"),
            "synthetic_after_sum": string_pointer(value, "/validation/synthetic_recalc/observed_after_sum"),
            "synthetic_delta_sum": string_pointer(value, "/validation/synthetic_recalc/observed_delta_sum"),
            "recalc_rounds": number_pointer(value, "/validation/synthetic_recalc/recalc_rounds"),
            "reference_visits": number_pointer(value, "/validation/synthetic_recalc/reference_visits"),
        })),
        "model_shape": value.as_ref().map_or_else(|| json!({}), |value| json!({
            "profile_details": value.pointer("/model/profile_details").cloned().unwrap_or(Value::Null),
            "descriptor_kind_counts": value.pointer("/model/descriptor_kind_counts").cloned().unwrap_or(Value::Null),
            "diagnostic_kind_counts": value.pointer("/model/diagnostic_kind_counts").cloned().unwrap_or(Value::Null),
        })),
        "timing_surfaces": value.as_ref().map_or_else(|| json!({}), |value| json!({
            "total_elapsed_ms": value.pointer("/total_elapsed_ms").cloned().unwrap_or(Value::Null),
            "phase_timings_ms": value.get("phase_timings_ms").cloned().unwrap_or(Value::Null),
        })),
        "semantic_note": "Scale timing is used only after closed-form semantic validation passes; timing alone is not promoted as correctness evidence.",
    });

    let row_missing_artifacts = row_array_strings(&row, "missing_artifacts");
    let row_failures = row_array_strings(&row, "failures");

    Ok(ScaleRunObservation {
        run_id,
        value,
        row,
        missing_artifacts: row_missing_artifacts,
        failures: row_failures,
    })
}

fn scale_signature_rows(observations: &[ScaleRunObservation]) -> Vec<Value> {
    let grid_r1 = observation_value(observations, "million_grid_r1");
    let grid_r2 = observation_value(observations, "million_grid_r2");
    let indirect = observation_value(observations, "million_indirect_r1");
    let fanout_f8 = observation_value(observations, "million_fanout_f8_r1");
    let fanout_calc = observation_value(observations, "million_fanout_f8_calc1024_r1");
    let relative = observation_value(observations, "million_relative_rebind_f8_r1");
    let fanout_f16 = observation_value(observations, "million_fanout_f16_r1");

    vec![
        compare_same_semantic_signature(
            "meta_scale_grid_repeat_invariance",
            "W033-META-013",
            "repeat grid-cross-sum with same semantic parameters",
            grid_r1,
            grid_r2,
            &[
                "/profile",
                "/model/node_count",
                "/model/formula_count",
                "/model/dependency_descriptor_count",
                "/model/dependency_edge_count",
                "/model/dependency_diagnostic_count",
                "/validation/synthetic_recalc/observed_after_sum",
                "/validation/synthetic_recalc/observed_delta_sum",
            ],
        ),
        compare_fanout_calc_amplification(fanout_f8, fanout_calc),
        compare_dynamic_indirect_to_grid(indirect, grid_r1),
        compare_relative_rebind(relative),
        compare_fanout_edge_widening(fanout_f8, fanout_f16),
    ]
}

fn w044_phase_timing_split_row(observations: &[ScaleRunObservation]) -> Value {
    let mut failures = Vec::new();
    let mut missing_artifacts = Vec::new();
    let mut run_checks = Vec::new();
    for observation in observations {
        if observation.value.is_none() {
            missing_artifacts.push(scale_run_summary_path(observation.run_id));
            continue;
        }
        let value = observation
            .value
            .as_ref()
            .expect("checked above as present");
        let mut phase_checks = Vec::new();
        for phase in W044_REQUIRED_PHASE_TIMINGS {
            let pointer = format!("/phase_timings_ms/{phase}");
            let present = value.pointer(&pointer).and_then(Value::as_f64).is_some();
            if !present {
                failures.push(format!(
                    "{}:missing_phase_timing:{phase}",
                    observation.run_id
                ));
            }
            phase_checks.push(json!({
                "phase": phase,
                "present": present,
                "elapsed_ms": value.pointer(&pointer).cloned().unwrap_or(Value::Null),
            }));
        }
        run_checks.push(json!({
            "run_id": observation.run_id,
            "phase_checks": phase_checks,
            "total_elapsed_ms": value.pointer("/total_elapsed_ms").cloned().unwrap_or(Value::Null),
        }));
    }
    signature_row(
        "w044_phase_timing_split_guard",
        "W044-SCALE-001",
        "separate dependency lowering, dependency graph build, soft-reference update, invalidation closure, pure recalc, and validation timing surfaces",
        missing_artifacts,
        failures,
        json!({
            "required_phase_timings": W044_REQUIRED_PHASE_TIMINGS,
            "run_checks": run_checks,
            "capability_consequence": "phase timings are measurement surfaces only and cannot promote performance-derived correctness",
        }),
    )
}

fn compare_same_semantic_signature(
    row_id: &str,
    family_id: &str,
    transformation: &str,
    left: Option<&Value>,
    right: Option<&Value>,
    checked_pointers: &[&str],
) -> Value {
    let mut failures = Vec::new();
    let mut missing_artifacts = Vec::new();
    if left.is_none() {
        missing_artifacts.push("left_scale_run_summary".to_string());
    }
    if right.is_none() {
        missing_artifacts.push("right_scale_run_summary".to_string());
    }
    let mut pointer_checks = Vec::new();
    if let (Some(left), Some(right)) = (left, right) {
        for pointer in checked_pointers {
            let left_value = left.pointer(pointer).cloned().unwrap_or(Value::Null);
            let right_value = right.pointer(pointer).cloned().unwrap_or(Value::Null);
            let matched = left_value == right_value;
            if !matched {
                failures.push(format!("pointer_mismatch:{pointer}"));
            }
            pointer_checks.push(json!({
                "pointer": pointer,
                "left": left_value,
                "right": right_value,
                "matched": matched,
            }));
        }
    }
    signature_row(
        row_id,
        family_id,
        transformation,
        missing_artifacts,
        failures,
        json!({
            "checked_pointers": pointer_checks,
            "allowed_internal_difference": "phase timings may differ; semantic counts and closed-form outputs must match",
        }),
    )
}

fn compare_fanout_calc_amplification(left: Option<&Value>, right: Option<&Value>) -> Value {
    let mut failures = Vec::new();
    let mut missing_artifacts = Vec::new();
    if left.is_none() {
        missing_artifacts.push("fanout_baseline_summary".to_string());
    }
    if right.is_none() {
        missing_artifacts.push("fanout_calc_amplified_summary".to_string());
    }
    let mut details = json!({});
    if let (Some(left), Some(right)) = (left, right) {
        let baseline_visits = number_pointer(left, "/validation/synthetic_recalc/reference_visits");
        let amplified_visits =
            number_pointer(right, "/validation/synthetic_recalc/reference_visits");
        let rounds = number_pointer(right, "/validation/synthetic_recalc/recalc_rounds");
        if rounds != 1024 {
            failures.push("unexpected_recalc_round_count".to_string());
        }
        if amplified_visits != baseline_visits.saturating_mul(rounds) {
            failures.push("reference_visits_do_not_scale_with_rounds".to_string());
        }
        for pointer in [
            "/profile",
            "/model/node_count",
            "/model/formula_count",
            "/model/dependency_descriptor_count",
            "/model/dependency_edge_count",
            "/model/dependency_diagnostic_count",
        ] {
            if left.pointer(pointer) != right.pointer(pointer) {
                failures.push(format!("model_pointer_mismatch:{pointer}"));
            }
        }
        details = json!({
            "baseline_reference_visits": baseline_visits,
            "amplified_reference_visits": amplified_visits,
            "amplified_recalc_rounds": rounds,
            "expected_amplified_reference_visits": baseline_visits.saturating_mul(rounds),
            "allowed_internal_difference": "synthetic recalc work is intentionally amplified while model/dependency surfaces stay unchanged",
        });
    }
    signature_row(
        "meta_scale_calc_amplification_binding",
        "W033-META-013",
        "repeat synthetic recalc over identical fanout model",
        missing_artifacts,
        failures,
        details,
    )
}

fn compare_dynamic_indirect_to_grid(indirect: Option<&Value>, grid: Option<&Value>) -> Value {
    let mut failures = Vec::new();
    let mut missing_artifacts = Vec::new();
    if indirect.is_none() {
        missing_artifacts.push("dynamic_indirect_summary".to_string());
    }
    if grid.is_none() {
        missing_artifacts.push("grid_baseline_summary".to_string());
    }
    let mut details = json!({});
    if let (Some(indirect), Some(grid)) = (indirect, grid) {
        let dynamic_slots = number_pointer(indirect, "/validation/synthetic_recalc/dynamic_slots");
        let diagnostics = number_pointer(indirect, "/model/dependency_diagnostic_count");
        let dynamic_descriptors =
            number_pointer(indirect, "/model/expected/dynamic_descriptor_count");
        if dynamic_slots != 1_000_000 {
            failures.push("dynamic_slot_count_not_million_floor".to_string());
        }
        if diagnostics != dynamic_slots || dynamic_descriptors != dynamic_slots {
            failures.push("dynamic_diagnostics_do_not_match_dynamic_slots".to_string());
        }
        if string_pointer(indirect, "/validation/synthetic_recalc/observed_after_sum")
            != string_pointer(grid, "/validation/synthetic_recalc/observed_after_sum")
        {
            failures.push("static_base_after_sum_differs_from_grid_baseline".to_string());
        }
        details = json!({
            "dynamic_slots": dynamic_slots,
            "dynamic_diagnostics": diagnostics,
            "dynamic_descriptors": dynamic_descriptors,
            "static_base_after_sum": string_pointer(indirect, "/validation/synthetic_recalc/observed_after_sum"),
            "grid_after_sum": string_pointer(grid, "/validation/synthetic_recalc/observed_after_sum"),
            "expected_relation": "dynamic potential references add diagnostics/carriers while preserving static closed-form base sum",
        });
    }
    signature_row(
        "meta_dynamic_indirect_semantic_binding",
        "W033-META-006",
        "add INDIRECT-shaped dynamic potential carrier to grid base",
        missing_artifacts,
        failures,
        details,
    )
}

fn compare_relative_rebind(relative: Option<&Value>) -> Value {
    let mut failures = Vec::new();
    let mut missing_artifacts = Vec::new();
    if relative.is_none() {
        missing_artifacts.push("relative_rebind_summary".to_string());
    }
    let mut details = json!({});
    if let Some(relative) = relative {
        let formula_count = number_pointer(relative, "/model/formula_count");
        let rebind_count = number_pointer(
            relative,
            "/validation/soft_reference_update/rebind_seed_count",
        );
        let expected_rebind_count = number_pointer(
            relative,
            "/validation/soft_reference_update/expected_rebind_seed_count",
        );
        if rebind_count != formula_count || expected_rebind_count != formula_count {
            failures.push("relative_rebind_seed_count_does_not_match_formula_count".to_string());
        }
        details = json!({
            "formula_count": formula_count,
            "rebind_seed_count": rebind_count,
            "expected_rebind_seed_count": expected_rebind_count,
            "expected_relation": "rename of the relative anchor produces one rebind seed per formula owner",
        });
    }
    signature_row(
        "meta_relative_rebind_churn_binding",
        "W033-META-001/W033-META-007/W033-META-013",
        "structural rename through relative references",
        missing_artifacts,
        failures,
        details,
    )
}

fn compare_fanout_edge_widening(f8: Option<&Value>, f16: Option<&Value>) -> Value {
    let mut failures = Vec::new();
    let mut missing_artifacts = Vec::new();
    if f8.is_none() {
        missing_artifacts.push("fanout_8_summary".to_string());
    }
    if f16.is_none() {
        missing_artifacts.push("fanout_16_summary".to_string());
    }
    let mut details = json!({});
    if let (Some(f8), Some(f16)) = (f8, f16) {
        let f8_edges = number_pointer(f8, "/model/dependency_edge_count");
        let f16_edges = number_pointer(f16, "/model/dependency_edge_count");
        if f16_edges <= f8_edges {
            failures.push("fanout_16_edges_not_greater_than_fanout_8_edges".to_string());
        }
        if number_pointer(f8, "/model/dependency_diagnostic_count") != 0
            || number_pointer(f16, "/model/dependency_diagnostic_count") != 0
        {
            failures.push("fanout_static_runs_have_unexpected_diagnostics".to_string());
        }
        details = json!({
            "fanout_8_edges": f8_edges,
            "fanout_16_edges": f16_edges,
            "expected_relation": "edge volume widens with fanout while closed-form validation remains true",
        });
    }
    signature_row(
        "meta_fanout_edge_widening_binding",
        "W033-META-013",
        "increase fanout from 8 to 16",
        missing_artifacts,
        failures,
        details,
    )
}

fn signature_row(
    row_id: &str,
    family_id: &str,
    transformation: &str,
    missing_artifacts: Vec<String>,
    failures: Vec<String>,
    details: Value,
) -> Value {
    json!({
        "row_id": row_id,
        "family_id": family_id,
        "transformation": transformation,
        "comparison_state": if missing_artifacts.is_empty() && failures.is_empty() {
            "semantic_binding_matched"
        } else if missing_artifacts.is_empty() {
            "unexpected_mismatch"
        } else {
            "missing_artifact"
        },
        "missing_artifacts": missing_artifacts,
        "failures": failures,
        "details": details,
    })
}

fn replay_binding_rows(
    repo_root: &Path,
    profile: &ScaleSemanticProfile,
) -> Result<Vec<Value>, ScaleSemanticBindingError> {
    let trace_result = read_artifact(
        repo_root,
        trace_scenario_artifact_path(
            profile.tracecalc_run_id,
            profile.tracecalc_scale_scenario_id,
            "result.json",
        ),
    )?;
    let trace_bundle = read_artifact(
        repo_root,
        relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "tracecalc-reference-machine",
            profile.tracecalc_run_id,
            "replay-appliance",
            "validation",
            "bundle_validation.json",
        ]),
    )?;
    let independent_summary = read_artifact(
        repo_root,
        relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "independent-conformance",
            profile.independent_conformance_run_id,
            "run_summary.json",
        ]),
    )?;
    let independent_bundle = read_artifact(
        repo_root,
        relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "independent-conformance",
            profile.independent_conformance_run_id,
            "replay-appliance",
            "validation",
            "bundle_validation.json",
        ]),
    )?;
    let pack_summary = read_artifact(
        repo_root,
        relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "pack-capability",
            profile.pack_capability_run_id,
            "run_summary.json",
        ]),
    )?;

    let mut rows = vec![
        trace_scale_seed_binding_row(trace_result, trace_bundle),
        independent_conformance_binding_row(independent_summary, independent_bundle),
        pack_no_promotion_binding_row(pack_summary),
    ];
    if let Some(row) = formal_gate_binding_row(repo_root, profile) {
        rows.push(row);
    }
    if is_w044_profile(profile) {
        rows.extend(w044_semantic_guard_binding_rows(repo_root)?);
    }
    Ok(rows)
}

fn trace_scale_seed_binding_row(result: ArtifactRead, bundle: ArtifactRead) -> Value {
    let mut failures = Vec::new();
    let mut missing = Vec::new();
    if result.value.is_none() {
        missing.push(result.relative_path.clone());
    }
    if bundle.value.is_none() {
        missing.push(bundle.relative_path.clone());
    }
    if let Some(value) = &result.value {
        if string_pointer(value, "/result_state") != "passed" {
            failures.push("trace_scale_seed_result_not_passed".to_string());
        }
        if !array_pointer_is_empty(value, "/assertion_failures")
            || !array_pointer_is_empty(value, "/validation_failures")
            || !array_pointer_is_empty(value, "/conformance_mismatches")
        {
            failures.push("trace_scale_seed_has_failures_or_mismatches".to_string());
        }
    }
    if let Some(value) = &bundle.value
        && string_pointer(value, "/status") != "bundle_valid"
    {
        failures.push("tracecalc_scale_seed_bundle_not_valid".to_string());
    }
    replay_row(
        "tracecalc_scale_seed_replay_binding",
        "TraceCalc scale seed replay binds generated scale lanes to an oracle scenario.",
        missing,
        failures,
        json!({
            "result_artifact": result.relative_path,
            "bundle_artifact": bundle.relative_path,
            "required_scale_classes": ["oxcalc.local.scale_seed"],
        }),
    )
}

fn independent_conformance_binding_row(summary: ArtifactRead, bundle: ArtifactRead) -> Value {
    let mut failures = Vec::new();
    let mut missing = Vec::new();
    if summary.value.is_none() {
        missing.push(summary.relative_path.clone());
    }
    if bundle.value.is_none() {
        missing.push(bundle.relative_path.clone());
    }
    if let Some(value) = &summary.value {
        if number_pointer(value, "/unexpected_mismatch_count") != 0 {
            failures.push("independent_conformance_has_unexpected_mismatches".to_string());
        }
        if number_pointer(value, "/missing_artifact_count") != 0 {
            failures.push("independent_conformance_has_missing_artifacts".to_string());
        }
    }
    if let Some(value) = &bundle.value
        && string_pointer(value, "/status") != "bundle_valid"
    {
        failures.push("independent_conformance_bundle_not_valid".to_string());
    }
    replay_row(
        "independent_conformance_projection_binding",
        "Scale evidence is tied to the post-W033 TreeCalc/TraceCalc projection comparison rather than timing alone.",
        missing,
        failures,
        json!({
            "summary_artifact": summary.relative_path,
            "bundle_artifact": bundle.relative_path,
        }),
    )
}

fn pack_no_promotion_binding_row(summary: ArtifactRead) -> Value {
    let mut failures = Vec::new();
    let mut missing = Vec::new();
    if summary.value.is_none() {
        missing.push(summary.relative_path.clone());
    }
    if let Some(value) = &summary.value {
        if string_pointer(value, "/decision_status") != "capability_not_promoted" {
            failures.push("pack_capability_decision_unexpectedly_promoted".to_string());
        }
        if string_pointer(value, "/highest_honest_capability") != "cap.C4.distill_valid" {
            failures.push("pack_capability_highest_honest_capability_changed".to_string());
        }
    }
    replay_row(
        "pack_capability_no_promotion_binding",
        "Scale semantic binding does not override pack-capability no-promotion.",
        missing,
        failures,
        json!({
            "summary_artifact": summary.relative_path,
            "expected_decision_status": "capability_not_promoted",
        }),
    )
}

fn formal_gate_binding_row(repo_root: &Path, profile: &ScaleSemanticProfile) -> Option<Value> {
    if profile.formal_gate_artifacts.is_empty() {
        return None;
    }

    let missing_artifacts = profile
        .formal_gate_artifacts
        .iter()
        .filter(|relative_path| !repo_root.join(relative_path).exists())
        .map(|relative_path| (*relative_path).to_string())
        .collect::<Vec<_>>();

    Some(replay_row(
        "w034_formal_gate_binding",
        "Continuous scale evidence is bound to W034 Lean/TLA/pack gate artifacts as bounded evidence, not proof of performance correctness.",
        missing_artifacts,
        Vec::new(),
        json!({
            "formal_gate_artifacts": profile.formal_gate_artifacts,
            "capability_consequence": "bounded_formal_and_scale_evidence_does_not_promote_continuous_scale_assurance",
        }),
    ))
}

fn w044_semantic_guard_binding_rows(
    repo_root: &Path,
) -> Result<Vec<Value>, ScaleSemanticBindingError> {
    Ok(vec![
        w044_guard_binding_row(
            repo_root,
            "w044_optimized_core_dynamic_transition_guard",
            "W044 optimized/core dynamic-transition evidence is present before interpreting scale timings.",
            W044_IMPLEMENTATION_CONFORMANCE_SUMMARY_PATH,
            &[("/failed_row_count", 0), ("/w044_match_promoted_count", 0)],
            &[
                ("/w044_disposition_row_count", 6),
                ("/w044_direct_evidence_bound_count", 2),
            ],
            &[],
        )?,
        w044_guard_binding_row(
            repo_root,
            "w044_rust_totality_refinement_guard",
            "W044 Rust totality/refinement evidence is present and remains non-promoting.",
            W044_RUST_FORMAL_ASSURANCE_SUMMARY_PATH,
            &[("/failed_row_count", 0)],
            &[("/local_proof_row_count", 11), ("/refinement_row_count", 9)],
            &[
                ("/promotion_claims/rust_engine_totality_promoted", false),
                ("/promotion_claims/rust_refinement_promoted", false),
                ("/promotion_claims/pack_grade_replay_promoted", false),
            ],
        )?,
        w044_guard_binding_row(
            repo_root,
            "w044_lean_tla_guard",
            "W044 Lean/TLA bounded proof/model evidence is present and remains non-promoting.",
            W044_LEAN_TLA_FORMAL_ASSURANCE_SUMMARY_PATH,
            &[("/failed_row_count", 0)],
            &[
                ("/local_proof_row_count", 10),
                ("/bounded_model_row_count", 4),
            ],
            &[
                ("/promotion_claims/full_lean_verification_promoted", false),
                ("/promotion_claims/full_tla_verification_promoted", false),
                ("/promotion_claims/unbounded_model_coverage_promoted", false),
            ],
        )?,
        w044_guard_binding_row(
            repo_root,
            "w044_stage2_scheduler_equivalence_guard",
            "W044 Stage 2 declared scheduler/pack equivalence evidence is present without policy promotion.",
            W044_STAGE2_REPLAY_SUMMARY_PATH,
            &[("/failed_row_count", 0)],
            &[
                ("/policy_row_count", 25),
                ("/observable_invariance_row_count", 5),
            ],
            &[
                ("/declared_scheduler_equivalence_evidenced", true),
                ("/declared_pack_equivalence_evidenced", true),
                ("/stage2_policy_promoted", false),
                ("/pack_grade_replay_promoted", false),
            ],
        )?,
        w044_guard_binding_row(
            repo_root,
            "w044_operated_assurance_guard",
            "W044 operated-assurance evidence is present while operated services and SLO enforcement remain unpromoted.",
            W044_OPERATED_ASSURANCE_SUMMARY_PATH,
            &[("/failed_row_count", 0)],
            &[
                ("/service_readiness_criteria_count", 25),
                ("/multi_run_history_row_count", 40),
            ],
            &[
                ("/file_backed_service_envelope_present", true),
                ("/operated_continuous_assurance_service_promoted", false),
                ("/retention_slo_enforcement_promoted", false),
            ],
        )?,
        w044_guard_binding_row(
            repo_root,
            "w044_diversity_service_guard",
            "W044 diversity/mismatch evidence is present while service promotions remain blocked.",
            W044_DIVERSITY_SEAM_SUMMARY_PATH,
            &[("/failed_row_count", 0)],
            &[
                ("/w044_independent_reference_model_case_count", 8),
                ("/w044_independent_reference_model_match_count", 8),
                ("/accepted_boundary_count", 25),
            ],
            &[
                ("/fully_independent_evaluator_promoted", false),
                ("/mismatch_quarantine_service_promoted", false),
                (
                    "/operated_cross_engine_differential_service_promoted",
                    false,
                ),
            ],
        )?,
        w044_guard_binding_row(
            repo_root,
            "w044_oxfml_typed_formatting_guard",
            "W044 OxFml typed formatting request construction is bound while downstream/public/callable blockers remain.",
            W044_OXFML_SEAM_SUMMARY_PATH,
            &[("/failed_row_count", 0)],
            &[
                ("/source_evidence_row_count", 15),
                ("/publication_display_row_count", 11),
            ],
            &[
                ("/w073_oxcalc_fixture_request_construction_verified", true),
                ("/w073_typed_only_formatting_guard_retained", true),
                (
                    "/w073_downstream_dnaonecalc_request_construction_verified",
                    false,
                ),
                ("/broad_oxfml_seam_promoted", false),
                ("/callable_metadata_projection_promoted", false),
            ],
        )?,
    ])
}

fn w044_guard_binding_row(
    repo_root: &Path,
    row_id: &str,
    purpose: &str,
    relative_path: &str,
    exact_numbers: &[(&str, u64)],
    minimum_numbers: &[(&str, u64)],
    expected_bools: &[(&str, bool)],
) -> Result<Value, ScaleSemanticBindingError> {
    let artifact = read_artifact(repo_root, relative_path.to_string())?;
    let mut failures = Vec::new();
    let mut missing = Vec::new();
    let mut checks = Vec::new();
    if artifact.value.is_none() {
        missing.push(artifact.relative_path.clone());
    }
    if let Some(value) = &artifact.value {
        for (pointer, expected) in exact_numbers {
            let observed = number_pointer(value, pointer);
            let matched = observed == *expected;
            if !matched {
                failures.push(format!("number_mismatch:{pointer}"));
            }
            checks.push(json!({
                "pointer": pointer,
                "relation": "equals",
                "expected": expected,
                "observed": observed,
                "matched": matched,
            }));
        }
        for (pointer, minimum) in minimum_numbers {
            let observed = number_pointer(value, pointer);
            let matched = observed >= *minimum;
            if !matched {
                failures.push(format!("number_below_minimum:{pointer}"));
            }
            checks.push(json!({
                "pointer": pointer,
                "relation": "at_least",
                "minimum": minimum,
                "observed": observed,
                "matched": matched,
            }));
        }
        for (pointer, expected) in expected_bools {
            let observed = bool_pointer(value, pointer);
            let matched = observed == *expected;
            if !matched {
                failures.push(format!("bool_mismatch:{pointer}"));
            }
            checks.push(json!({
                "pointer": pointer,
                "relation": "equals",
                "expected": expected,
                "observed": observed,
                "matched": matched,
            }));
        }
    }
    Ok(replay_row(
        row_id,
        purpose,
        missing,
        failures,
        json!({
            "artifact": artifact.relative_path,
            "checks": checks,
            "capability_consequence": "W044 semantic guard evidence is a precondition for interpreting scale timings, not performance-derived correctness proof.",
        }),
    ))
}

fn replay_row(
    row_id: &str,
    purpose: &str,
    missing_artifacts: Vec<String>,
    failures: Vec<String>,
    details: Value,
) -> Value {
    json!({
        "row_id": row_id,
        "purpose": purpose,
        "binding_state": if missing_artifacts.is_empty() && failures.is_empty() {
            "binding_valid"
        } else if missing_artifacts.is_empty() {
            "unexpected_mismatch"
        } else {
            "missing_artifact"
        },
        "missing_artifacts": missing_artifacts,
        "failures": failures,
        "details": details,
    })
}

fn continuous_scale_criteria(
    run_id: &str,
    artifact_root: &str,
    profile: &ScaleSemanticProfile,
    evaluation: &Evaluation,
) -> Value {
    let semantic_binding_valid =
        evaluation.missing_artifacts.is_empty() && evaluation.unexpected_mismatches.is_empty();
    let formal_gate_missing = profile.formal_gate_artifacts.iter().any(|artifact| {
        evaluation
            .missing_artifacts
            .iter()
            .any(|missing| missing == artifact)
    });
    let mut criteria = vec![
        json!({
            "criterion_id": "scale.semantic.closed_form_validation",
            "state": if evaluation.validated_scale_runs == SCALE_RUN_IDS.len() { "satisfied" } else { "partial" },
            "validated_scale_run_count": evaluation.validated_scale_runs,
            "required_scale_run_count": SCALE_RUN_IDS.len(),
            "capability_consequence": "semantic_input_only"
        }),
        json!({
            "criterion_id": "scale.semantic.metamorphic_signature_binding",
            "state": if count_failure_rows(&evaluation.signature_rows) == 0 { "satisfied" } else { "unexpected_mismatch" },
            "signature_row_count": evaluation.signature_rows.len(),
            "capability_consequence": "semantic_input_only"
        }),
        json!({
            "criterion_id": "scale.semantic.replay_conformance_pack_binding",
            "state": if semantic_binding_valid { "satisfied" } else { "partial" },
            "replay_binding_row_count": evaluation.replay_rows.len(),
            "capability_consequence": "prevents_timing_only_correctness_claim"
        }),
        json!({
            "criterion_id": "scale.formal.w034_gate_binding",
            "state": if profile.formal_gate_artifacts.is_empty() {
                "not_applicable_to_profile"
            } else if formal_gate_missing {
                "missing_artifact"
            } else {
                "bounded_no_promotion"
            },
            "capability_consequence": "Lean/TLA smoke and proof slices support review but do not promote full verification or continuous scale assurance"
        }),
        json!({
            "criterion_id": "scale.continuous.scheduled_regression_floor",
            "state": "missing",
            "capability_consequence": "continuous_scale_assurance_not_promoted"
        }),
        json!({
            "criterion_id": "scale.continuous.cross_engine_diff_service",
            "state": "missing",
            "capability_consequence": "continuous_scale_assurance_not_promoted"
        }),
    ];
    if is_w044_profile(profile) {
        criteria.push(json!({
            "criterion_id": "scale.w044.phase_timing_split",
            "state": if row_id_has_no_failures(&evaluation.signature_rows, "w044_phase_timing_split_guard") { "satisfied" } else { "unexpected_mismatch" },
            "required_phase_timings": W044_REQUIRED_PHASE_TIMINGS,
            "capability_consequence": "phase timings distinguish dependency build, soft-reference update, invalidation closure, and pure recalc without promoting timing as correctness proof"
        }));
        criteria.push(json!({
            "criterion_id": "scale.w044.semantic_guard_binding",
            "state": if w044_guard_rows_valid(&evaluation.replay_rows) { "satisfied" } else { "partial" },
            "required_guard_row_count": W044_GUARD_ARTIFACT_PATHS.len(),
            "capability_consequence": "release-scale evidence is subordinate to W044 semantic guard packets"
        }));
    }

    json!({
        "schema_version": SCALE_CONTINUOUS_CRITERIA_SCHEMA_V1,
        "run_id": run_id,
        "evidence_profile": profile.profile_id,
        "artifact_root": artifact_root,
        "continuous_scale_assurance_promoted": false,
        "performance_claim_promoted": false,
        "criteria": criteria,
        "no_promotion_reason_ids": evaluation.no_promotion_reasons,
        "semantic_equivalence_statement": "The criteria packet classifies already-emitted evidence and does not change runtime scheduling, invalidation, recalc, publication, reject, or evaluator behavior.",
    })
}

fn scale_semantic_profile(run_id: &str) -> ScaleSemanticProfile {
    if run_id.starts_with("w044-") {
        ScaleSemanticProfile {
            profile_id: W044_PROFILE_ID,
            family_packet: "docs/spec/core-engine/w044-formalization/W044_RELEASE_SCALE_REPLAY_PERFORMANCE_AND_SCALING_EVIDENCE_UNDER_SEMANTIC_GUARDS.md",
            tracecalc_run_id: W037_TRACECALC_RUN_ID,
            tracecalc_scale_scenario_id: TRACECALC_SCALE_SCENARIO_ID,
            independent_conformance_run_id: W034_INDEPENDENT_CONFORMANCE_RUN_ID,
            pack_capability_run_id: W043_PACK_CAPABILITY_RUN_ID,
            formal_gate_artifacts: &[],
            additional_no_promotion_reasons: &[
                "scale.w044.phase_timing_split_is_measurement_only",
                "scale.w044.semantic_guard_binding_required_before_pack_reassessment",
                "scale.w044.no_operated_continuous_scale_service",
                "scale.w044.no_release_grade_correctness_from_performance",
            ],
        }
    } else if run_id.starts_with("w034-") {
        ScaleSemanticProfile {
            profile_id: "w034_continuous_scale_gate_binding",
            family_packet: "docs/spec/core-engine/w034-formalization/W034_PACK_CAPABILITY_AND_CONTINUOUS_SCALE_GATE_BINDING.md",
            tracecalc_run_id: W034_TRACECALC_RUN_ID,
            tracecalc_scale_scenario_id: TRACECALC_SCALE_SCENARIO_ID,
            independent_conformance_run_id: W034_INDEPENDENT_CONFORMANCE_RUN_ID,
            pack_capability_run_id: W034_PACK_CAPABILITY_RUN_ID,
            formal_gate_artifacts: W034_FORMAL_GATE_ARTIFACTS,
            additional_no_promotion_reasons: &[
                "scale.continuous.no_scheduled_regression_suite",
                "scale.continuous.no_cross_engine_continuous_diff_service",
                "scale.continuous.formal_gates_bounded_smoke_only",
            ],
        }
    } else {
        ScaleSemanticProfile {
            profile_id: "post_w033_metamorphic_scale_semantic_binding",
            family_packet: "docs/spec/core-engine/w033-formalization/W033_METAMORPHIC_DIFFERENTIAL_TEST_FAMILIES.md",
            tracecalc_run_id: POST_W033_TRACECALC_RUN_ID,
            tracecalc_scale_scenario_id: TRACECALC_SCALE_SCENARIO_ID,
            independent_conformance_run_id: POST_W033_INDEPENDENT_CONFORMANCE_RUN_ID,
            pack_capability_run_id: POST_W033_PACK_CAPABILITY_RUN_ID,
            formal_gate_artifacts: &[],
            additional_no_promotion_reasons: &[],
        }
    }
}

fn is_w044_profile(profile: &ScaleSemanticProfile) -> bool {
    profile.profile_id == W044_PROFILE_ID
}

fn observation_value<'a>(
    observations: &'a [ScaleRunObservation],
    run_id: &str,
) -> Option<&'a Value> {
    observations
        .iter()
        .find(|observation| observation.run_id == run_id)
        .and_then(|observation| observation.value.as_ref())
}

fn all_validation_checks_passed(value: &Value) -> bool {
    value
        .pointer("/validation/checks")
        .and_then(Value::as_array)
        .is_some_and(|checks| {
            !checks.is_empty()
                && checks
                    .iter()
                    .all(|check| check.get("passed").and_then(Value::as_bool) == Some(true))
        })
}

fn count_failure_rows(rows: &[Value]) -> usize {
    rows.iter()
        .filter(|row| {
            row.get("failures")
                .and_then(Value::as_array)
                .is_some_and(|failures| !failures.is_empty())
                || row
                    .get("missing_artifacts")
                    .and_then(Value::as_array)
                    .is_some_and(|missing| !missing.is_empty())
        })
        .count()
}

fn row_id_has_no_failures(rows: &[Value], row_id: &str) -> bool {
    rows.iter()
        .find(|row| row.get("row_id").and_then(Value::as_str) == Some(row_id))
        .is_some_and(|row| {
            row.get("failures")
                .and_then(Value::as_array)
                .is_none_or(Vec::is_empty)
                && row
                    .get("missing_artifacts")
                    .and_then(Value::as_array)
                    .is_none_or(Vec::is_empty)
        })
}

fn w044_guard_rows_valid(rows: &[Value]) -> bool {
    let guard_count = rows
        .iter()
        .filter(|row| {
            row.get("row_id")
                .and_then(Value::as_str)
                .is_some_and(|row_id| row_id.starts_with("w044_"))
        })
        .count();
    guard_count == W044_GUARD_ARTIFACT_PATHS.len()
        && rows
            .iter()
            .filter(|row| {
                row.get("row_id")
                    .and_then(Value::as_str)
                    .is_some_and(|row_id| row_id.starts_with("w044_"))
            })
            .all(|row| {
                row.get("failures")
                    .and_then(Value::as_array)
                    .is_none_or(Vec::is_empty)
                    && row
                        .get("missing_artifacts")
                        .and_then(Value::as_array)
                        .is_none_or(Vec::is_empty)
            })
}

fn collect_row_failures(row: &Value, failures: &mut Vec<String>) {
    let row_id = row.get("row_id").and_then(Value::as_str).unwrap_or("row");
    if let Some(row_failures) = row.get("failures").and_then(Value::as_array) {
        failures.extend(
            row_failures
                .iter()
                .filter_map(Value::as_str)
                .map(|failure| format!("{row_id}:{failure}")),
        );
    }
}

fn collect_row_missing(row: &Value, missing: &mut Vec<String>) {
    if let Some(row_missing) = row.get("missing_artifacts").and_then(Value::as_array) {
        missing.extend(
            row_missing
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string),
        );
    }
}

fn row_array_strings(row: &Value, key: &str) -> Vec<String> {
    row.get(key)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_string)
        .collect()
}

fn read_artifact(
    repo_root: &Path,
    relative_path: String,
) -> Result<ArtifactRead, ScaleSemanticBindingError> {
    Ok(ArtifactRead {
        value: read_json(repo_root, &relative_path)?,
        relative_path,
    })
}

fn read_json(
    repo_root: &Path,
    relative_path: &str,
) -> Result<Option<Value>, ScaleSemanticBindingError> {
    let path = repo_root.join(relative_path);
    if !path.exists() {
        return Ok(None);
    }
    let content =
        fs::read_to_string(&path).map_err(|source| ScaleSemanticBindingError::ReadArtifact {
            path: path.display().to_string(),
            source,
        })?;
    serde_json::from_str(&content).map(Some).map_err(|source| {
        ScaleSemanticBindingError::ParseJson {
            path: path.display().to_string(),
            source,
        }
    })
}

fn write_json(path: &Path, value: &Value) -> Result<(), ScaleSemanticBindingError> {
    let content = serde_json::to_string_pretty(value).expect("JSON serialization should succeed");
    fs::write(path, format!("{content}\n")).map_err(|source| ScaleSemanticBindingError::WriteFile {
        path: path.display().to_string(),
        source,
    })
}

fn create_directory(path: &Path) -> Result<(), ScaleSemanticBindingError> {
    fs::create_dir_all(path).map_err(|source| ScaleSemanticBindingError::CreateDirectory {
        path: path.display().to_string(),
        source,
    })
}

fn bool_pointer(value: &Value, pointer: &str) -> bool {
    value
        .pointer(pointer)
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

fn number_pointer(value: &Value, pointer: &str) -> u64 {
    value.pointer(pointer).and_then(Value::as_u64).unwrap_or(0)
}

fn string_pointer(value: &Value, pointer: &str) -> String {
    value
        .pointer(pointer)
        .and_then(Value::as_str)
        .unwrap_or("<missing>")
        .to_string()
}

fn array_pointer_is_empty(value: &Value, pointer: &str) -> bool {
    value
        .pointer(pointer)
        .and_then(Value::as_array)
        .is_none_or(Vec::is_empty)
}

fn scale_run_summary_path(run_id: &str) -> String {
    relative_artifact_path([
        "docs",
        "test-runs",
        "core-engine",
        "treecalc-scale",
        run_id,
        "run_summary.json",
    ])
}

fn trace_scenario_artifact_path(
    tracecalc_run_id: &str,
    scenario_id: &str,
    artifact_name: &str,
) -> String {
    relative_artifact_path([
        "docs",
        "test-runs",
        "core-engine",
        "tracecalc-reference-machine",
        tracecalc_run_id,
        "scenarios",
        scenario_id,
        artifact_name,
    ])
}

fn required_artifacts(run_id: &str, profile: &ScaleSemanticProfile) -> Vec<String> {
    [
        "run_summary.json",
        "evidence/scale_semantic_evidence_index.json",
        "differentials/scale_signature_differential.json",
        "replay_conformance_bindings.json",
        "decision/scale_no_promotion_decision.json",
        "decision/continuous_scale_assurance_criteria.json",
        "replay-appliance/bundle_manifest.json",
        "replay-appliance/validation/bundle_validation.json",
    ]
    .iter()
    .map(|artifact| {
        relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "metamorphic-scale-semantic-binding",
            run_id,
            artifact,
        ])
    })
    .chain(
        SCALE_RUN_IDS
            .iter()
            .map(|run_id| scale_run_summary_path(run_id)),
    )
    .chain([
        trace_scenario_artifact_path(
            profile.tracecalc_run_id,
            profile.tracecalc_scale_scenario_id,
            "result.json",
        ),
        relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "tracecalc-reference-machine",
            profile.tracecalc_run_id,
            "replay-appliance",
            "validation",
            "bundle_validation.json",
        ]),
        relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "independent-conformance",
            profile.independent_conformance_run_id,
            "run_summary.json",
        ]),
        relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "independent-conformance",
            profile.independent_conformance_run_id,
            "replay-appliance",
            "validation",
            "bundle_validation.json",
        ]),
        relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "pack-capability",
            profile.pack_capability_run_id,
            "run_summary.json",
        ]),
    ])
    .chain(
        profile
            .formal_gate_artifacts
            .iter()
            .map(|artifact| (*artifact).to_string()),
    )
    .chain(
        (if is_w044_profile(profile) {
            W044_GUARD_ARTIFACT_PATHS
        } else {
            &[]
        })
        .iter()
        .map(|artifact| (*artifact).to_string()),
    )
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
    fn scale_semantic_binding_runner_writes_valid_no_promotion_packet() {
        let repo_root = unique_temp_repo();
        create_source_artifacts(&repo_root);

        let summary = ScaleSemanticBindingRunner::new()
            .execute(&repo_root, "scale-binding-test")
            .expect("scale binding packet should write");

        assert_eq!(summary.scale_run_row_count, 7);
        assert_eq!(summary.validated_scale_run_count, 7);
        assert_eq!(summary.scale_signature_row_count, 5);
        assert_eq!(summary.replay_binding_row_count, 3);
        assert_eq!(summary.missing_artifact_count, 0);
        assert_eq!(summary.unexpected_mismatch_count, 0);
        assert!(summary.no_promotion_reason_count > 0);

        let validation = read_required_json(
            &repo_root,
            "docs/test-runs/core-engine/metamorphic-scale-semantic-binding/scale-binding-test/replay-appliance/validation/bundle_validation.json",
        );
        assert_eq!(validation["status"], "bundle_valid");

        let decision = read_required_json(
            &repo_root,
            "docs/test-runs/core-engine/metamorphic-scale-semantic-binding/scale-binding-test/decision/scale_no_promotion_decision.json",
        );
        assert_eq!(decision["performance_claim_promoted"], false);

        fs::remove_dir_all(repo_root.parent().unwrap()).unwrap();
    }

    #[test]
    fn scale_semantic_binding_runner_writes_w034_continuous_gate_packet() {
        let repo_root = unique_temp_repo();
        create_w034_source_artifacts(&repo_root);

        let summary = ScaleSemanticBindingRunner::new()
            .execute(&repo_root, "w034-scale-binding-test")
            .expect("W034 scale binding packet should write");

        assert_eq!(summary.scale_run_row_count, 7);
        assert_eq!(summary.validated_scale_run_count, 7);
        assert_eq!(summary.scale_signature_row_count, 5);
        assert_eq!(summary.replay_binding_row_count, 4);
        assert_eq!(summary.missing_artifact_count, 0);
        assert_eq!(summary.unexpected_mismatch_count, 0);

        let criteria = read_required_json(
            &repo_root,
            "docs/test-runs/core-engine/metamorphic-scale-semantic-binding/w034-scale-binding-test/decision/continuous_scale_assurance_criteria.json",
        );
        assert_eq!(
            criteria["evidence_profile"],
            "w034_continuous_scale_gate_binding"
        );
        assert_eq!(criteria["continuous_scale_assurance_promoted"], false);

        let validation = read_required_json(
            &repo_root,
            "docs/test-runs/core-engine/metamorphic-scale-semantic-binding/w034-scale-binding-test/replay-appliance/validation/bundle_validation.json",
        );
        assert_eq!(validation["status"], "bundle_valid");

        fs::remove_dir_all(repo_root.parent().unwrap()).unwrap();
    }

    #[test]
    fn scale_semantic_binding_runner_writes_w044_release_scale_guard_packet() {
        let repo_root = unique_temp_repo();
        create_w044_source_artifacts(&repo_root);

        let summary = ScaleSemanticBindingRunner::new()
            .execute(&repo_root, "w044-release-scale-binding-test")
            .expect("W044 scale binding packet should write");

        assert_eq!(summary.scale_run_row_count, 7);
        assert_eq!(summary.validated_scale_run_count, 7);
        assert_eq!(summary.scale_signature_row_count, 6);
        assert_eq!(summary.replay_binding_row_count, 10);
        assert_eq!(summary.missing_artifact_count, 0);
        assert_eq!(summary.unexpected_mismatch_count, 0);

        let criteria = read_required_json(
            &repo_root,
            "docs/test-runs/core-engine/metamorphic-scale-semantic-binding/w044-release-scale-binding-test/decision/continuous_scale_assurance_criteria.json",
        );
        assert_eq!(criteria["evidence_profile"], W044_PROFILE_ID);
        assert!(
            criteria["criteria"]
                .as_array()
                .unwrap()
                .iter()
                .any(|criterion| {
                    criterion["criterion_id"] == "scale.w044.phase_timing_split"
                        && criterion["state"] == "satisfied"
                })
        );
        assert!(
            criteria["criteria"]
                .as_array()
                .unwrap()
                .iter()
                .any(|criterion| {
                    criterion["criterion_id"] == "scale.w044.semantic_guard_binding"
                        && criterion["state"] == "satisfied"
                })
        );

        let validation = read_required_json(
            &repo_root,
            "docs/test-runs/core-engine/metamorphic-scale-semantic-binding/w044-release-scale-binding-test/replay-appliance/validation/bundle_validation.json",
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
            "oxcalc-scale-semantic-binding-test-{}-{nanos}",
            std::process::id()
        ));
        let repo_root = base.join("OxCalc");
        fs::create_dir_all(&repo_root).unwrap();
        repo_root
    }

    fn create_source_artifacts(repo_root: &Path) {
        scale_summary(
            repo_root,
            ScaleFixture {
                run_id: "million_grid_r1",
                profile: "grid-cross-sum",
                node_count: 1_002_001,
                formula_count: 1_000_000,
                descriptor_count: 2_000_000,
                edge_count: 2_000_000,
                diagnostic_count: 0,
                impacted_count: 2_001,
                dynamic_descriptor_count: 0,
                after_sum: "92299000",
                delta_sum: "18000",
                recalc_rounds: 1,
                reference_visits: 2_000_000,
                dynamic_slots: 0,
                rebind_seed_count: 0,
            },
        );
        scale_summary(
            repo_root,
            ScaleFixture {
                run_id: "million_grid_r2",
                profile: "grid-cross-sum",
                node_count: 1_002_001,
                formula_count: 1_000_000,
                descriptor_count: 2_000_000,
                edge_count: 2_000_000,
                diagnostic_count: 0,
                impacted_count: 2_001,
                dynamic_descriptor_count: 0,
                after_sum: "92299000",
                delta_sum: "18000",
                recalc_rounds: 1,
                reference_visits: 2_000_000,
                dynamic_slots: 0,
                rebind_seed_count: 0,
            },
        );
        scale_summary(
            repo_root,
            ScaleFixture {
                run_id: "million_indirect_r1",
                profile: "dynamic-indirect-stripes",
                node_count: 1_002_001,
                formula_count: 1_000_000,
                descriptor_count: 3_000_000,
                edge_count: 2_000_000,
                diagnostic_count: 1_000_000,
                impacted_count: 2_001,
                dynamic_descriptor_count: 1_000_000,
                after_sum: "92299000",
                delta_sum: "18000",
                recalc_rounds: 1,
                reference_visits: 3_000_000,
                dynamic_slots: 1_000_000,
                rebind_seed_count: 0,
            },
        );
        scale_summary(
            repo_root,
            ScaleFixture {
                run_id: "million_fanout_f8_r1",
                profile: "fanout-bands",
                node_count: 1_000_000,
                formula_count: 999_991,
                descriptor_count: 7_999_928,
                edge_count: 7_999_928,
                diagnostic_count: 0,
                impacted_count: 999_992,
                dynamic_descriptor_count: 0,
                after_sum: "42999613",
                delta_sum: "6999937",
                recalc_rounds: 1,
                reference_visits: 7_999_928,
                dynamic_slots: 0,
                rebind_seed_count: 0,
            },
        );
        scale_summary(
            repo_root,
            ScaleFixture {
                run_id: "million_fanout_f8_calc1024_r1",
                profile: "fanout-bands",
                node_count: 1_000_000,
                formula_count: 999_991,
                descriptor_count: 7_999_928,
                edge_count: 7_999_928,
                diagnostic_count: 0,
                impacted_count: 999_992,
                dynamic_descriptor_count: 0,
                after_sum: "44031603712",
                delta_sum: "7167935488",
                recalc_rounds: 1024,
                reference_visits: 8_191_926_272,
                dynamic_slots: 0,
                rebind_seed_count: 0,
            },
        );
        scale_summary(
            repo_root,
            ScaleFixture {
                run_id: "million_relative_rebind_f8_r1",
                profile: "relative-rebind-churn",
                node_count: 1_000_000,
                formula_count: 999_991,
                descriptor_count: 7_999_928,
                edge_count: 7_999_928,
                diagnostic_count: 0,
                impacted_count: 999_992,
                dynamic_descriptor_count: 0,
                after_sum: "42999613",
                delta_sum: "6999937",
                recalc_rounds: 1,
                reference_visits: 7_999_928,
                dynamic_slots: 0,
                rebind_seed_count: 999_991,
            },
        );
        scale_summary(
            repo_root,
            ScaleFixture {
                run_id: "million_fanout_f16_r1",
                profile: "fanout-bands",
                node_count: 1_000_000,
                formula_count: 999_983,
                descriptor_count: 15_999_728,
                edge_count: 15_999_728,
                diagnostic_count: 0,
                impacted_count: 999_984,
                dynamic_descriptor_count: 0,
                after_sum: "142997569",
                delta_sum: "6999881",
                recalc_rounds: 1,
                reference_visits: 15_999_728,
                dynamic_slots: 0,
                rebind_seed_count: 0,
            },
        );

        write_json_test(
            repo_root,
            &trace_scenario_artifact_path(
                POST_W033_TRACECALC_RUN_ID,
                TRACECALC_SCALE_SCENARIO_ID,
                "result.json",
            ),
            json!({
                "result_state": "passed",
                "assertion_failures": [],
                "validation_failures": [],
                "conformance_mismatches": [],
            }),
        );
        write_json_test(
            repo_root,
            "docs/test-runs/core-engine/tracecalc-reference-machine/post-w033-let-lambda-carrier-witness-001/replay-appliance/validation/bundle_validation.json",
            json!({ "status": "bundle_valid" }),
        );
        write_json_test(
            repo_root,
            "docs/test-runs/core-engine/independent-conformance/post-w033-independent-conformance-001/run_summary.json",
            json!({
                "unexpected_mismatch_count": 0,
                "missing_artifact_count": 0,
            }),
        );
        write_json_test(
            repo_root,
            "docs/test-runs/core-engine/independent-conformance/post-w033-independent-conformance-001/replay-appliance/validation/bundle_validation.json",
            json!({ "status": "bundle_valid" }),
        );
        write_json_test(
            repo_root,
            "docs/test-runs/core-engine/pack-capability/post-w033-pack-capability-decision-001/run_summary.json",
            json!({
                "decision_status": "capability_not_promoted",
                "highest_honest_capability": "cap.C4.distill_valid",
            }),
        );
    }

    fn create_w034_source_artifacts(repo_root: &Path) {
        create_source_artifacts(repo_root);
        write_json_test(
            repo_root,
            &trace_scenario_artifact_path(
                W034_TRACECALC_RUN_ID,
                TRACECALC_SCALE_SCENARIO_ID,
                "result.json",
            ),
            json!({
                "result_state": "passed",
                "assertion_failures": [],
                "validation_failures": [],
                "conformance_mismatches": [],
            }),
        );
        write_json_test(
            repo_root,
            "docs/test-runs/core-engine/tracecalc-reference-machine/w034-tracecalc-oracle-deepening-001/replay-appliance/validation/bundle_validation.json",
            json!({ "status": "bundle_valid" }),
        );
        write_json_test(
            repo_root,
            "docs/test-runs/core-engine/independent-conformance/w034-independent-conformance-001/run_summary.json",
            json!({
                "unexpected_mismatch_count": 0,
                "missing_artifact_count": 0,
            }),
        );
        write_json_test(
            repo_root,
            "docs/test-runs/core-engine/independent-conformance/w034-independent-conformance-001/replay-appliance/validation/bundle_validation.json",
            json!({ "status": "bundle_valid" }),
        );
        write_json_test(
            repo_root,
            "docs/test-runs/core-engine/pack-capability/w034-pack-capability-gate-binding-001/run_summary.json",
            json!({
                "decision_status": "capability_not_promoted",
                "highest_honest_capability": "cap.C4.distill_valid",
            }),
        );
        for artifact in W034_FORMAL_GATE_ARTIFACTS {
            write_text_test(repo_root, artifact, "W034 formal gate artifact\n");
        }
    }

    fn create_w044_source_artifacts(repo_root: &Path) {
        create_w034_source_artifacts(repo_root);
        write_json_test(
            repo_root,
            &trace_scenario_artifact_path(
                W037_TRACECALC_RUN_ID,
                TRACECALC_SCALE_SCENARIO_ID,
                "result.json",
            ),
            json!({
                "result_state": "passed",
                "assertion_failures": [],
                "validation_failures": [],
                "conformance_mismatches": [],
            }),
        );
        write_json_test(
            repo_root,
            "docs/test-runs/core-engine/tracecalc-reference-machine/w037-tracecalc-observable-closure-001/replay-appliance/validation/bundle_validation.json",
            json!({ "status": "bundle_valid" }),
        );
        write_json_test(
            repo_root,
            "docs/test-runs/core-engine/pack-capability/w043-pack-grade-replay-governance-c5-release-reassessment-001/run_summary.json",
            json!({
                "decision_status": "capability_not_promoted",
                "highest_honest_capability": "cap.C4.distill_valid",
            }),
        );
        write_json_test(
            repo_root,
            W044_IMPLEMENTATION_CONFORMANCE_SUMMARY_PATH,
            json!({
                "failed_row_count": 0,
                "w044_disposition_row_count": 6,
                "w044_direct_evidence_bound_count": 2,
                "w044_match_promoted_count": 0,
            }),
        );
        write_json_test(
            repo_root,
            W044_RUST_FORMAL_ASSURANCE_SUMMARY_PATH,
            json!({
                "failed_row_count": 0,
                "local_proof_row_count": 11,
                "refinement_row_count": 9,
                "promotion_claims": {
                    "rust_engine_totality_promoted": false,
                    "rust_refinement_promoted": false,
                    "pack_grade_replay_promoted": false,
                },
            }),
        );
        write_json_test(
            repo_root,
            W044_LEAN_TLA_FORMAL_ASSURANCE_SUMMARY_PATH,
            json!({
                "failed_row_count": 0,
                "local_proof_row_count": 10,
                "bounded_model_row_count": 4,
                "promotion_claims": {
                    "full_lean_verification_promoted": false,
                    "full_tla_verification_promoted": false,
                    "unbounded_model_coverage_promoted": false,
                },
            }),
        );
        write_json_test(
            repo_root,
            W044_STAGE2_REPLAY_SUMMARY_PATH,
            json!({
                "failed_row_count": 0,
                "policy_row_count": 25,
                "observable_invariance_row_count": 5,
                "declared_scheduler_equivalence_evidenced": true,
                "declared_pack_equivalence_evidenced": true,
                "stage2_policy_promoted": false,
                "pack_grade_replay_promoted": false,
            }),
        );
        write_json_test(
            repo_root,
            W044_OPERATED_ASSURANCE_SUMMARY_PATH,
            json!({
                "failed_row_count": 0,
                "service_readiness_criteria_count": 25,
                "multi_run_history_row_count": 40,
                "file_backed_service_envelope_present": true,
                "operated_continuous_assurance_service_promoted": false,
                "retention_slo_enforcement_promoted": false,
            }),
        );
        write_json_test(
            repo_root,
            W044_DIVERSITY_SEAM_SUMMARY_PATH,
            json!({
                "failed_row_count": 0,
                "w044_independent_reference_model_case_count": 8,
                "w044_independent_reference_model_match_count": 8,
                "accepted_boundary_count": 25,
                "fully_independent_evaluator_promoted": false,
                "mismatch_quarantine_service_promoted": false,
                "operated_cross_engine_differential_service_promoted": false,
            }),
        );
        write_json_test(
            repo_root,
            W044_OXFML_SEAM_SUMMARY_PATH,
            json!({
                "failed_row_count": 0,
                "source_evidence_row_count": 15,
                "publication_display_row_count": 11,
                "w073_oxcalc_fixture_request_construction_verified": true,
                "w073_typed_only_formatting_guard_retained": true,
                "w073_downstream_dnaonecalc_request_construction_verified": false,
                "broad_oxfml_seam_promoted": false,
                "callable_metadata_projection_promoted": false,
            }),
        );
    }

    struct ScaleFixture {
        run_id: &'static str,
        profile: &'static str,
        node_count: u64,
        formula_count: u64,
        descriptor_count: u64,
        edge_count: u64,
        diagnostic_count: u64,
        impacted_count: u64,
        dynamic_descriptor_count: u64,
        after_sum: &'static str,
        delta_sum: &'static str,
        recalc_rounds: u64,
        reference_visits: u64,
        dynamic_slots: u64,
        rebind_seed_count: u64,
    }

    fn scale_summary(repo_root: &Path, fixture: ScaleFixture) {
        write_json_test(
            repo_root,
            &scale_run_summary_path(fixture.run_id),
            json!({
                "schema_version": "oxcalc.treecalc.scale_run_summary.v1",
                "run_id": fixture.run_id,
                "profile": fixture.profile,
                "model": {
                    "node_count": fixture.node_count,
                    "formula_count": fixture.formula_count,
                    "dependency_descriptor_count": fixture.descriptor_count,
                    "dependency_edge_count": fixture.edge_count,
                    "dependency_diagnostic_count": fixture.diagnostic_count,
                    "invalidation_impacted_count": fixture.impacted_count,
                    "expected": {
                        "dynamic_descriptor_count": fixture.dynamic_descriptor_count,
                    },
                },
                "phase_timings_ms": {
                    "model_build_structural_snapshot_and_formula_catalog": 1.0,
                    "dependency_descriptor_lowering": 1.0,
                    "dependency_graph_build_and_cycle_scan": 1.0,
                    "soft_reference_update_rebind_seed_derivation": 1.0,
                    "invalidation_closure_derivation": 1.0,
                    "synthetic_closed_form_recalc": 1.0,
                    "validation_checks": 1.0,
                },
                "total_elapsed_ms": 10.0,
                "validation": {
                    "passed": true,
                    "checks": [
                        { "name": "synthetic_after_sum", "expected": fixture.after_sum, "observed": fixture.after_sum, "passed": true },
                        { "name": "synthetic_delta_sum", "expected": fixture.delta_sum, "observed": fixture.delta_sum, "passed": true }
                    ],
                    "soft_reference_update": {
                        "rebind_seed_count": fixture.rebind_seed_count,
                        "expected_rebind_seed_count": fixture.rebind_seed_count,
                    },
                    "synthetic_recalc": {
                        "expected_after_sum": fixture.after_sum,
                        "observed_after_sum": fixture.after_sum,
                        "expected_delta_sum": fixture.delta_sum,
                        "observed_delta_sum": fixture.delta_sum,
                        "recalc_rounds": fixture.recalc_rounds,
                        "reference_visits": fixture.reference_visits,
                        "dynamic_slots": fixture.dynamic_slots,
                    }
                }
            }),
        );
    }

    fn write_json_test(repo_root: &Path, relative_path: &str, value: Value) {
        let path = repo_root.join(relative_path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, serde_json::to_string_pretty(&value).unwrap() + "\n").unwrap();
    }

    fn write_text_test(repo_root: &Path, relative_path: &str, value: &str) {
        let path = repo_root.join(relative_path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, value).unwrap();
    }

    fn read_required_json(repo_root: &Path, relative_path: &str) -> Value {
        serde_json::from_str(&fs::read_to_string(repo_root.join(relative_path)).unwrap()).unwrap()
    }
}
