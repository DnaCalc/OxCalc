#![forbid(unsafe_code)]

//! Retained-failure fixture runner for `TraceCalc`.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::json;
use thiserror::Error;

use crate::assertions::compare_artifacts;
use crate::contracts::{TraceCalcScenario, TraceCalcScenarioResultState, load_scenario};
use crate::machine::{TraceCalcEngineMachine, TraceCalcReferenceMachine};
use crate::witness::{
    TraceCalcReductionUnit, TraceCalcWitnessSeed, TraceCalcWitnessSeedInputs, build_witness_seed,
};

const RETAINED_FAILURE_MANIFEST_SCHEMA_V1: &str = "oxcalc.tracecalc.retained_failure_manifest.v1";
const RETAINED_FAILURE_CASE_SCHEMA_V1: &str = "oxcalc.tracecalc.retained_failure_case.v1";
const RETAINED_LOCAL_REDUCTION_STATUS_ID: &str = "oxcalc.reduction.retained_local";
const RETAINED_STATUS_SCOPE: &str = "local_only_until_foundation_binding";
const REPLAY_BUNDLE_MANIFEST_SCHEMA_V1: &str = "oxcalc.local.replay_bundle_manifest.v1";
const REPLAY_RUN_MANIFEST_SCHEMA_V1: &str = "oxcalc.local.replay_run_manifest.v1";
const REPLAY_ADAPTER_CAPABILITY_SNAPSHOT_SCHEMA_V1: &str =
    "oxcalc.local.adapter_capability_snapshot.v1";
const REPLAY_BUNDLE_VALIDATION_SCHEMA_V1: &str = "oxcalc.local.replay_bundle_validation.v1";
const REPLAY_EXPLAIN_RECORD_SCHEMA_V1: &str = "oxcalc.local.replay_explain_record.v1";
const DISTILL_VALIDATION_SCHEMA_V1: &str = "oxcalc.tracecalc.distill_validation.v1";
const DISTILLATION_MANIFEST_SCHEMA_V1: &str = "oxcalc.tracecalc.distillation_manifest.v1";
const PACK_CANDIDATE_ASSESSMENT_SCHEMA_V1: &str = "oxcalc.tracecalc.pack_candidate_assessment.v1";
const PACK_CANDIDATE_VALIDATION_SCHEMA_V1: &str = "oxcalc.tracecalc.pack_candidate_validation.v1";
const FOUNDATION_REPLAY_REGISTRY_VERSION: &str =
    "foundation.replay.authoritative-pass-01.2026-03-15";

#[derive(Debug, Clone, Deserialize)]
struct RetainedFailureManifest {
    schema_version: String,
    base_path: String,
    cases: Vec<RetainedFailureManifestEntry>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct RetainedFailureManifestEntry {
    case_id: String,
    path: String,
    #[serde(default)]
    tags: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct RetainedFailureCase {
    schema_version: String,
    case_id: String,
    description: String,
    source_run_id: String,
    source_scenario_path: String,
    lifecycle_target: String,
    target_mismatch_kind: String,
    #[serde(default)]
    required_equality_surfaces: Vec<String>,
    #[serde(default)]
    selected_unit_ids: Vec<String>,
    preservation_check: RetainedFailurePreservationCheck,
    quarantine_reason: Option<String>,
    #[serde(default)]
    pack_candidate_rehearsal: bool,
    #[serde(default)]
    notes: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct RetainedFailurePreservationCheck {
    kind: String,
    reference: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct TraceCalcRetainedFailureRunSummary {
    pub run_id: String,
    pub schema_version: String,
    pub case_count: usize,
    pub lifecycle_counts: Vec<(String, usize)>,
    pub artifact_root: String,
}

#[derive(Debug, Clone, Serialize)]
struct TraceCalcRetainedFailureCaseSummary {
    case_id: String,
    description: String,
    source_scenario_id: String,
    lifecycle_state: String,
    replay_validation_assessed: bool,
    replay_valid: Option<bool>,
    predicate_preserved: bool,
    artifact_paths: BTreeMap<String, String>,
    notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct TraceCalcRetainedFailureReplayValidation {
    case_id: String,
    lifecycle_target: String,
    replay_validation_assessed: bool,
    scenario_replay_valid: Option<bool>,
    engine_conformance_match: Option<bool>,
    predicate_preserved: bool,
    selected_unit_ids: Vec<String>,
    required_equality_surfaces: Vec<String>,
    target_mismatch_kind: String,
}

#[derive(Debug, Clone, Serialize)]
struct TraceCalcDistillationManifest {
    schema_version: String,
    distillation_id: String,
    witness_id: String,
    case_id: String,
    source_scenario_id: String,
    source_run_id: String,
    lifecycle_state: String,
    selected_unit_ids: Vec<String>,
    selected_step_ids: Vec<String>,
    removed_step_ids: Vec<String>,
    required_equality_surfaces: Vec<String>,
    target_mismatch_kind: String,
    dependency_projection_status: String,
    semantic_display_boundary_status: String,
}

#[derive(Debug, Clone, Serialize)]
struct TraceCalcDistillValidation {
    schema_version: String,
    distillation_id: String,
    case_id: String,
    lifecycle_state: String,
    distill_status: String,
    replay_validation_assessed: bool,
    reduced_scenario_replay_valid: Option<bool>,
    reduced_engine_conformance_match: Option<bool>,
    predicate_preserved: bool,
    dependency_projection: TraceCalcDependencyProjectionAssessment,
    semantic_display_boundary: TraceCalcSemanticDisplayBoundaryAssessment,
    selected_unit_ids: Vec<String>,
    selected_step_ids: Vec<String>,
    removed_step_ids: Vec<String>,
    required_equality_surfaces: Vec<String>,
    target_mismatch_kind: String,
}

#[derive(Debug, Clone, Serialize)]
struct TraceCalcDependencyProjectionAssessment {
    projection_status: String,
    dependency_shape_kinds: Vec<String>,
    runtime_effect_kinds: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct TraceCalcSemanticDisplayBoundaryAssessment {
    boundary_status: String,
    semantic_surface_only: bool,
    format_delta_exercised: bool,
    display_delta_exercised: bool,
    note: String,
}

#[derive(Debug, Clone, Serialize)]
struct TraceCalcPackCandidateAssessment {
    schema_version: String,
    case_id: String,
    lifecycle_state: String,
    distill_status: String,
    candidate_state: String,
    pack_eligible: bool,
    replay_valid: Option<bool>,
    predicate_preserved: bool,
    dependency_projection_status: String,
    semantic_display_boundary_status: String,
    blocked_by: Vec<String>,
}

#[derive(Debug, Error)]
pub enum TraceCalcRetainedFailureError {
    #[error("failed to read {path}: {source}")]
    Read {
        path: String,
        source: std::io::Error,
    },
    #[error("failed to parse json from {path}: {source}")]
    Parse {
        path: String,
        source: serde_json::Error,
    },
    #[error("failed to create directory {path}: {source}")]
    CreateDirectory {
        path: String,
        source: std::io::Error,
    },
    #[error("failed to write artifact {path}: {source}")]
    WriteFile {
        path: String,
        source: std::io::Error,
    },
    #[error("failed to remove existing artifact root {path}: {source}")]
    RemoveDirectory {
        path: String,
        source: std::io::Error,
    },
    #[error("retained-failure case '{case_id}' uses unknown reduction unit '{unit_id}'")]
    UnknownReductionUnit { case_id: String, unit_id: String },
    #[error(
        "retained-failure case '{case_id}' has unsupported lifecycle target '{lifecycle_target}'"
    )]
    UnsupportedLifecycleTarget {
        case_id: String,
        lifecycle_target: String,
    },
    #[error("retained-failure case '{case_id}' has unsupported preservation check '{kind}'")]
    UnsupportedPreservationCheck { case_id: String, kind: String },
    #[error(
        "source scenario '{scenario_id}' for retained-failure case '{case_id}' is missing witness anchors"
    )]
    MissingWitnessAnchors {
        case_id: String,
        scenario_id: String,
    },
    #[error("replay validation failed for retained-local case '{case_id}'")]
    ReplayValidationFailed { case_id: String },
}

#[derive(Debug, Default)]
pub struct TraceCalcRetainedFailureRunner {
    reference_machine: TraceCalcReferenceMachine,
    engine_machine: TraceCalcEngineMachine,
}

impl TraceCalcRetainedFailureRunner {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn execute_manifest(
        &self,
        repo_root: &Path,
        run_id: &str,
    ) -> Result<TraceCalcRetainedFailureRunSummary, TraceCalcRetainedFailureError> {
        let manifest_path = repo_root
            .join("docs/test-fixtures/core-engine/tracecalc-retained-failures/MANIFEST.json");
        let manifest = load_json::<RetainedFailureManifest>(&manifest_path)?;
        assert_eq!(manifest.schema_version, RETAINED_FAILURE_MANIFEST_SCHEMA_V1);
        let artifact_root = repo_root.join(format!(
            "docs/test-runs/core-engine/tracecalc-retained-failures/{run_id}"
        ));
        let relative_artifact_root = relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "tracecalc-retained-failures",
            run_id,
        ]);

        if artifact_root.exists() {
            fs::remove_dir_all(&artifact_root).map_err(|source| {
                TraceCalcRetainedFailureError::RemoveDirectory {
                    path: artifact_root.display().to_string(),
                    source,
                }
            })?;
        }

        create_directory(&artifact_root)?;
        create_directory(&artifact_root.join("cases"))?;
        create_directory(&artifact_root.join("replay-appliance"))?;
        create_directory(&artifact_root.join("replay-appliance/adapter_capabilities"))?;
        create_directory(&artifact_root.join("replay-appliance/runs"))?;
        create_directory(&artifact_root.join("replay-appliance/validation"))?;
        create_directory(&artifact_root.join("replay-appliance/runs").join(run_id))?;
        create_directory(
            &artifact_root
                .join("replay-appliance/runs")
                .join(run_id)
                .join("cases"),
        )?;
        write_bundle_capability_snapshot(&artifact_root, run_id)?;

        write_json(
            &artifact_root.join("manifest_selection.json"),
            &json!(manifest.cases),
        )?;

        let mut lifecycle_counts = BTreeMap::new();
        let mut case_summaries = Vec::new();
        let mut bundle_cases = Vec::new();
        let mut distill_cases = Vec::new();
        let mut pack_candidate_cases = Vec::new();

        for entry in &manifest.cases {
            let case_path = repo_root
                .join("docs/test-fixtures/core-engine/tracecalc-retained-failures")
                .join(manifest.base_path.replace('/', "\\"))
                .join(entry.path.replace('/', "\\"));
            let case = load_json::<RetainedFailureCase>(&case_path)?;
            assert_eq!(case.schema_version, RETAINED_FAILURE_CASE_SCHEMA_V1);
            let scenario_path = repo_root
                .join("docs/test-corpus/core-engine/tracecalc")
                .join(case.source_scenario_path.replace('/', "\\"));
            let scenario_text = fs::read_to_string(&scenario_path).map_err(|source| {
                TraceCalcRetainedFailureError::Read {
                    path: scenario_path.display().to_string(),
                    source,
                }
            })?;
            let scenario = load_scenario(&scenario_path).map_err(|error| {
                TraceCalcRetainedFailureError::Read {
                    path: scenario_path.display().to_string(),
                    source: std::io::Error::other(error.to_string()),
                }
            })?;
            if scenario.witness_anchors.is_none() {
                return Err(TraceCalcRetainedFailureError::MissingWitnessAnchors {
                    case_id: case.case_id,
                    scenario_id: scenario.scenario_id,
                });
            }

            let source_artifact_paths =
                source_run_artifact_paths(&case.source_run_id, &scenario.scenario_id);
            let mut witness = build_witness_seed(TraceCalcWitnessSeedInputs {
                run_id,
                relative_artifact_root: &relative_artifact_root,
                scenario: &scenario,
                result_state: TraceCalcScenarioResultState::Passed,
                validation_failures: &[],
                assertion_failures: &[],
                scenario_artifact_paths: &source_artifact_paths,
                conformance_mismatches: &[],
            })
            .ok_or_else(|| TraceCalcRetainedFailureError::MissingWitnessAnchors {
                case_id: case.case_id.clone(),
                scenario_id: scenario.scenario_id.clone(),
            })?;

            let selected_unit_ids = expand_selected_units(
                &case.case_id,
                &witness.reduction_manifest.units,
                &case.selected_unit_ids,
            )?;
            witness
                .reduction_manifest
                .units
                .retain(|unit| selected_unit_ids.contains(unit.unit_id.as_str()));
            witness.reduction_manifest.required_equality_surfaces =
                case.required_equality_surfaces.clone();
            witness.reduction_manifest.mismatch_kinds = vec![case.target_mismatch_kind.clone()];
            apply_case_lifecycle(&case, &mut witness)?;

            let replay_validation = build_replay_validation(
                &case,
                &scenario,
                &selected_unit_ids,
                &witness.reduction_manifest.units,
                &self.reference_machine,
                &self.engine_machine,
            )?;
            if case.lifecycle_target == "wit.retained_local"
                && (!replay_validation.replay_validation_assessed
                    || replay_validation.scenario_replay_valid != Some(true)
                    || replay_validation.engine_conformance_match != Some(true)
                    || !replay_validation.predicate_preserved)
            {
                return Err(TraceCalcRetainedFailureError::ReplayValidationFailed {
                    case_id: case.case_id.clone(),
                });
            }
            let reduced_scenario =
                build_reduced_scenario(&case, &scenario, &witness.reduction_manifest.units);
            let distill_validation = build_distill_validation(
                &case,
                &reduced_scenario,
                &witness.reduction_manifest.units,
                &self.reference_machine,
                &self.engine_machine,
            )?;
            let distillation_manifest = build_distillation_manifest(
                &case,
                &scenario,
                &reduced_scenario,
                &witness,
                &distill_validation,
            );
            let pack_candidate_assessment =
                build_pack_candidate_assessment(&case, &witness, &distill_validation);

            let case_directory = artifact_root.join("cases").join(&case.case_id);
            create_directory(&case_directory)?;
            let witness_bundle_directory = case_directory.join("witness_bundle");
            create_directory(&witness_bundle_directory)?;
            let case_summary = TraceCalcRetainedFailureCaseSummary {
                case_id: case.case_id.clone(),
                description: case.description.clone(),
                source_scenario_id: scenario.scenario_id.clone(),
                lifecycle_state: witness.lifecycle.lifecycle_state.clone(),
                replay_validation_assessed: replay_validation.replay_validation_assessed,
                replay_valid: replay_validation.scenario_replay_valid,
                predicate_preserved: replay_validation.predicate_preserved,
                artifact_paths: BTreeMap::new(),
                notes: case.notes.clone(),
            };

            write_json(
                &case_directory.join("lifecycle.json"),
                &serde_json::to_value(&witness.lifecycle).expect("lifecycle serialization"),
            )?;
            write_json(
                &case_directory.join("reduction_manifest.json"),
                &serde_json::to_value(&witness.reduction_manifest)
                    .expect("reduction serialization"),
            )?;
            write_json(
                &case_directory.join("replay_validation.json"),
                &serde_json::to_value(&replay_validation).expect("validation serialization"),
            )?;
            fs::write(
                witness_bundle_directory.join("source_scenario.json"),
                &scenario_text,
            )
            .map_err(|source| TraceCalcRetainedFailureError::WriteFile {
                path: witness_bundle_directory
                    .join("source_scenario.json")
                    .display()
                    .to_string(),
                source,
            })?;
            write_json(
                &witness_bundle_directory.join("reduced_scenario.json"),
                &serde_json::to_value(&reduced_scenario).expect("reduced scenario serialization"),
            )?;
            write_json(
                &case_directory.join("distillation_manifest.json"),
                &serde_json::to_value(&distillation_manifest)
                    .expect("distillation manifest serialization"),
            )?;
            write_json(
                &case_directory.join("distill_validation.json"),
                &serde_json::to_value(&distill_validation)
                    .expect("distill validation serialization"),
            )?;
            write_json(
                &case_directory.join("pack_candidate_assessment.json"),
                &serde_json::to_value(&pack_candidate_assessment)
                    .expect("pack candidate assessment serialization"),
            )?;

            let case_artifact_paths = BTreeMap::from([
                (
                    "lifecycle".to_string(),
                    relative_artifact_path([
                        &relative_artifact_root,
                        "cases",
                        &case.case_id,
                        "lifecycle.json",
                    ]),
                ),
                (
                    "reduction_manifest".to_string(),
                    relative_artifact_path([
                        &relative_artifact_root,
                        "cases",
                        &case.case_id,
                        "reduction_manifest.json",
                    ]),
                ),
                (
                    "replay_validation".to_string(),
                    relative_artifact_path([
                        &relative_artifact_root,
                        "cases",
                        &case.case_id,
                        "replay_validation.json",
                    ]),
                ),
                (
                    "source_scenario".to_string(),
                    relative_artifact_path([
                        &relative_artifact_root,
                        "cases",
                        &case.case_id,
                        "witness_bundle",
                        "source_scenario.json",
                    ]),
                ),
                (
                    "reduced_scenario".to_string(),
                    relative_artifact_path([
                        &relative_artifact_root,
                        "cases",
                        &case.case_id,
                        "witness_bundle",
                        "reduced_scenario.json",
                    ]),
                ),
                (
                    "distillation_manifest".to_string(),
                    relative_artifact_path([
                        &relative_artifact_root,
                        "cases",
                        &case.case_id,
                        "distillation_manifest.json",
                    ]),
                ),
                (
                    "distill_validation".to_string(),
                    relative_artifact_path([
                        &relative_artifact_root,
                        "cases",
                        &case.case_id,
                        "distill_validation.json",
                    ]),
                ),
                (
                    "pack_candidate_assessment".to_string(),
                    relative_artifact_path([
                        &relative_artifact_root,
                        "cases",
                        &case.case_id,
                        "pack_candidate_assessment.json",
                    ]),
                ),
            ]);

            write_json(
                &case_directory.join("case_summary.json"),
                &serde_json::to_value(&TraceCalcRetainedFailureCaseSummary {
                    artifact_paths: case_artifact_paths.clone(),
                    ..case_summary.clone()
                })
                .expect("case summary serialization"),
            )?;
            bundle_cases.push(write_bundle_case_projection(
                &artifact_root,
                run_id,
                &relative_artifact_root,
                &case.case_id,
                &case_summary,
                &witness.lifecycle,
                &witness.reduction_manifest,
                &replay_validation,
                &distillation_manifest,
                &distill_validation,
                &pack_candidate_assessment,
                &scenario_text,
                &reduced_scenario,
                &case_artifact_paths,
            )?);
            distill_cases.push(json!({
                "case_id": case.case_id,
                "lifecycle_state": witness.lifecycle.lifecycle_state,
                "distill_status": distill_validation.distill_status,
                "bundle_distill_validation_path": relative_artifact_path([
                    &relative_artifact_root,
                    "replay-appliance",
                    "runs",
                    run_id,
                    "cases",
                    &case.case_id,
                    "distill_validation.json",
                ]),
            }));
            pack_candidate_cases.push(json!({
                "case_id": case.case_id,
                "candidate_state": pack_candidate_assessment.candidate_state,
                "pack_eligible": pack_candidate_assessment.pack_eligible,
                "bundle_pack_candidate_assessment_path": relative_artifact_path([
                    &relative_artifact_root,
                    "replay-appliance",
                    "runs",
                    run_id,
                    "cases",
                    &case.case_id,
                    "pack_candidate_assessment.json",
                ]),
            }));

            *lifecycle_counts
                .entry(witness.lifecycle.lifecycle_state.clone())
                .or_insert(0_usize) += 1;
            case_summaries.push(json!({
                "case_id": case.case_id,
                "source_scenario_id": scenario.scenario_id,
                "lifecycle_state": witness.lifecycle.lifecycle_state,
                "artifact_paths": case_artifact_paths,
            }));
        }

        write_json(
            &artifact_root.join("case_index.json"),
            &json!(case_summaries),
        )?;

        let summary = TraceCalcRetainedFailureRunSummary {
            run_id: run_id.to_string(),
            schema_version: manifest.schema_version,
            case_count: manifest.cases.len(),
            lifecycle_counts: lifecycle_counts.into_iter().collect(),
            artifact_root: artifact_root.display().to_string(),
        };
        write_json(
            &artifact_root.join("run_summary.json"),
            &serde_json::to_value(&summary).expect("summary serialization"),
        )?;
        write_bundle_run_projection(
            &artifact_root,
            run_id,
            &relative_artifact_root,
            &bundle_cases,
            &summary,
        )?;
        write_distill_validation(&artifact_root, run_id, &distill_cases)?;
        write_pack_candidate_validation(&artifact_root, run_id, &pack_candidate_cases)?;
        write_bundle_validation(repo_root, &artifact_root, run_id, &bundle_cases)?;
        Ok(summary)
    }
}

fn build_replay_validation(
    case: &RetainedFailureCase,
    scenario: &TraceCalcScenario,
    selected_unit_ids: &BTreeSet<String>,
    selected_units: &[TraceCalcReductionUnit],
    reference_machine: &TraceCalcReferenceMachine,
    engine_machine: &TraceCalcEngineMachine,
) -> Result<TraceCalcRetainedFailureReplayValidation, TraceCalcRetainedFailureError> {
    let predicate_preserved =
        evaluate_preservation_check(case, scenario, selected_unit_ids, selected_units)?;

    let (replay_validation_assessed, scenario_replay_valid, engine_conformance_match) =
        match case.lifecycle_target.as_str() {
            "wit.retained_local" | "wit.explanatory_only" => {
                let oracle_artifacts = reference_machine.execute(scenario).map_err(|error| {
                    TraceCalcRetainedFailureError::Read {
                        path: scenario.scenario_id.clone(),
                        source: std::io::Error::other(error.to_string()),
                    }
                })?;
                let engine_artifacts = engine_machine.execute(scenario).map_err(|error| {
                    TraceCalcRetainedFailureError::Read {
                        path: scenario.scenario_id.clone(),
                        source: std::io::Error::other(error.to_string()),
                    }
                })?;
                let conformance_mismatches =
                    compare_artifacts(&oracle_artifacts, &engine_artifacts);
                (
                    true,
                    Some(oracle_artifacts.result_state == TraceCalcScenarioResultState::Passed),
                    Some(conformance_mismatches.is_empty()),
                )
            }
            "wit.quarantined" => (false, None, None),
            lifecycle_target => {
                return Err(TraceCalcRetainedFailureError::UnsupportedLifecycleTarget {
                    case_id: case.case_id.clone(),
                    lifecycle_target: lifecycle_target.to_string(),
                });
            }
        };

    Ok(TraceCalcRetainedFailureReplayValidation {
        case_id: case.case_id.clone(),
        lifecycle_target: case.lifecycle_target.clone(),
        replay_validation_assessed,
        scenario_replay_valid,
        engine_conformance_match,
        predicate_preserved,
        selected_unit_ids: selected_unit_ids.iter().cloned().collect(),
        required_equality_surfaces: case.required_equality_surfaces.clone(),
        target_mismatch_kind: case.target_mismatch_kind.clone(),
    })
}

fn build_distill_validation(
    case: &RetainedFailureCase,
    reduced_scenario: &TraceCalcScenario,
    selected_units: &[TraceCalcReductionUnit],
    reference_machine: &TraceCalcReferenceMachine,
    engine_machine: &TraceCalcEngineMachine,
) -> Result<TraceCalcDistillValidation, TraceCalcRetainedFailureError> {
    let selected_unit_ids = selected_units
        .iter()
        .map(|unit| unit.unit_id.clone())
        .collect::<BTreeSet<_>>();
    let selected_step_ids = selected_units
        .iter()
        .flat_map(|unit| unit.step_ids.iter().cloned())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let removed_step_ids = selected_units
        .first()
        .map(|_| {
            reduced_scenario
                .notes
                .iter()
                .filter_map(|note| note.strip_prefix("removed_step_id="))
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let predicate_preserved =
        evaluate_preservation_check(case, reduced_scenario, &selected_unit_ids, selected_units)?;
    let dependency_projection = build_dependency_projection_assessment(reduced_scenario);
    let semantic_display_boundary = build_semantic_display_boundary_assessment();

    let (
        replay_validation_assessed,
        reduced_scenario_replay_valid,
        reduced_engine_conformance_match,
    ) = match case.lifecycle_target.as_str() {
        "wit.retained_local" | "wit.explanatory_only" => {
            let oracle_artifacts =
                reference_machine
                    .execute(reduced_scenario)
                    .map_err(|error| TraceCalcRetainedFailureError::Read {
                        path: reduced_scenario.scenario_id.clone(),
                        source: std::io::Error::other(error.to_string()),
                    })?;
            let engine_artifacts = engine_machine.execute(reduced_scenario).map_err(|error| {
                TraceCalcRetainedFailureError::Read {
                    path: reduced_scenario.scenario_id.clone(),
                    source: std::io::Error::other(error.to_string()),
                }
            })?;
            let conformance_mismatches = compare_artifacts(&oracle_artifacts, &engine_artifacts);
            (
                true,
                Some(oracle_artifacts.result_state == TraceCalcScenarioResultState::Passed),
                Some(conformance_mismatches.is_empty()),
            )
        }
        "wit.quarantined" => (false, None, None),
        lifecycle_target => {
            return Err(TraceCalcRetainedFailureError::UnsupportedLifecycleTarget {
                case_id: case.case_id.clone(),
                lifecycle_target: lifecycle_target.to_string(),
            });
        }
    };

    let distill_status = match case.lifecycle_target.as_str() {
        "wit.retained_local"
            if replay_validation_assessed
                && reduced_scenario_replay_valid == Some(true)
                && reduced_engine_conformance_match == Some(true)
                && predicate_preserved =>
        {
            "distill_valid"
        }
        "wit.explanatory_only" => "distill_explanatory_only",
        "wit.quarantined" => "distill_quarantined",
        _ => "distill_degraded",
    }
    .to_string();

    Ok(TraceCalcDistillValidation {
        schema_version: DISTILL_VALIDATION_SCHEMA_V1.to_string(),
        distillation_id: format!("{}--distilled", case.case_id),
        case_id: case.case_id.clone(),
        lifecycle_state: case.lifecycle_target.clone(),
        distill_status,
        replay_validation_assessed,
        reduced_scenario_replay_valid,
        reduced_engine_conformance_match,
        predicate_preserved,
        dependency_projection,
        semantic_display_boundary,
        selected_unit_ids: selected_unit_ids.into_iter().collect(),
        selected_step_ids,
        removed_step_ids,
        required_equality_surfaces: case.required_equality_surfaces.clone(),
        target_mismatch_kind: case.target_mismatch_kind.clone(),
    })
}

fn evaluate_preservation_check(
    case: &RetainedFailureCase,
    scenario: &TraceCalcScenario,
    selected_unit_ids: &BTreeSet<String>,
    selected_units: &[TraceCalcReductionUnit],
) -> Result<bool, TraceCalcRetainedFailureError> {
    match case.preservation_check.kind.as_str() {
        "reject_id_present" => Ok(scenario
            .expected
            .rejects
            .iter()
            .any(|reject| reject.reject_id == case.preservation_check.reference)
            && selected_units.iter().any(|unit| {
                unit.reject_id.as_deref() == Some(case.preservation_check.reference.as_str())
            })),
        "counter_present" => Ok(scenario
            .expected
            .counter_expectations
            .iter()
            .any(|counter| counter.counter == case.preservation_check.reference)
            && selected_unit_ids.iter().any(|unit_id| {
                unit_id == "scenario"
                    || unit_id.starts_with("phase:")
                    || unit_id.starts_with("events:")
            })),
        "published_view_present" => Ok(!scenario.expected.published_view.snapshot_id.is_empty()
            && selected_unit_ids
                .iter()
                .any(|unit_id| unit_id == "scenario" || unit_id.starts_with("view:"))),
        "dependency_shape_kind_present" => Ok(scenario.steps.iter().any(|step| {
            step.dependency_shape_updates
                .iter()
                .any(|update| update.kind == case.preservation_check.reference)
        })),
        "runtime_effect_present" => Ok(scenario.steps.iter().any(|step| {
            step.runtime_effects
                .iter()
                .any(|effect| effect.effect_kind == case.preservation_check.reference)
        })),
        kind => Err(
            TraceCalcRetainedFailureError::UnsupportedPreservationCheck {
                case_id: case.case_id.clone(),
                kind: kind.to_string(),
            },
        ),
    }
}

fn build_reduced_scenario(
    case: &RetainedFailureCase,
    scenario: &TraceCalcScenario,
    selected_units: &[TraceCalcReductionUnit],
) -> TraceCalcScenario {
    let selected_step_ids = selected_units
        .iter()
        .flat_map(|unit| unit.step_ids.iter().cloned())
        .collect::<BTreeSet<_>>();
    let removed_step_ids = scenario
        .steps
        .iter()
        .filter(|step| !selected_step_ids.contains(&step.step_id))
        .map(|step| step.step_id.clone())
        .collect::<Vec<_>>();

    let mut reduced = scenario.clone();
    reduced.scenario_id = format!("{}--distilled", case.case_id);
    reduced.description = format!("Distilled witness slice for {}", scenario.scenario_id);
    reduced.steps = scenario
        .steps
        .iter()
        .filter(|step| selected_step_ids.contains(&step.step_id))
        .cloned()
        .collect();
    if let Some(anchors) = &scenario.witness_anchors {
        let selected_unit_id_set = selected_units
            .iter()
            .map(|unit| unit.unit_id.as_str())
            .collect::<BTreeSet<_>>();
        reduced.witness_anchors = Some(crate::contracts::TraceCalcWitnessAnchors {
            scenario_anchor_id: anchors.scenario_anchor_id.clone(),
            phase_blocks: anchors
                .phase_blocks
                .iter()
                .filter_map(|anchor| {
                    let step_ids = anchor
                        .step_ids
                        .iter()
                        .filter(|step_id| selected_step_ids.contains(step_id.as_str()))
                        .cloned()
                        .collect::<Vec<_>>();
                    (!step_ids.is_empty()
                        || selected_unit_id_set
                            .contains(format!("phase:{}", anchor.phase_block_id).as_str()))
                    .then(|| crate::contracts::TraceCalcPhaseBlockAnchor {
                        phase_block_id: anchor.phase_block_id.clone(),
                        step_ids,
                    })
                })
                .collect(),
            event_groups: anchors
                .event_groups
                .iter()
                .filter_map(|anchor| {
                    let step_ids = anchor
                        .step_ids
                        .iter()
                        .filter(|step_id| selected_step_ids.contains(step_id.as_str()))
                        .cloned()
                        .collect::<Vec<_>>();
                    (!step_ids.is_empty()
                        || selected_unit_id_set
                            .contains(format!("events:{}", anchor.event_group_id).as_str()))
                    .then(|| crate::contracts::TraceCalcEventGroupAnchor {
                        event_group_id: anchor.event_group_id.clone(),
                        step_ids,
                    })
                })
                .collect(),
            reject_records: anchors
                .reject_records
                .iter()
                .filter(|anchor| {
                    selected_unit_id_set
                        .contains(format!("reject:{}", anchor.reject_record_id).as_str())
                })
                .cloned()
                .collect(),
            view_slices: anchors
                .view_slices
                .iter()
                .filter(|anchor| {
                    selected_unit_id_set.contains(format!("view:{}", anchor.view_slice_id).as_str())
                })
                .cloned()
                .collect(),
        });
    }
    reduced.notes.push(format!(
        "Distilled from retained-failure case '{}' using declared reduction units.",
        case.case_id
    ));
    for step_id in removed_step_ids {
        reduced.notes.push(format!("removed_step_id={step_id}"));
    }
    reduced
}

fn build_dependency_projection_assessment(
    scenario: &TraceCalcScenario,
) -> TraceCalcDependencyProjectionAssessment {
    let dependency_shape_kinds = scenario
        .steps
        .iter()
        .flat_map(|step| {
            step.dependency_shape_updates
                .iter()
                .map(|update| update.kind.clone())
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let runtime_effect_kinds = scenario
        .steps
        .iter()
        .flat_map(|step| {
            step.runtime_effects
                .iter()
                .map(|effect| effect.effect_kind.clone())
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let projection_status = if dependency_shape_kinds.is_empty() && runtime_effect_kinds.is_empty()
    {
        "dp.not_applicable"
    } else {
        "dp.projected_in_reduced_witness"
    }
    .to_string();

    TraceCalcDependencyProjectionAssessment {
        projection_status,
        dependency_shape_kinds,
        runtime_effect_kinds,
    }
}

fn build_semantic_display_boundary_assessment() -> TraceCalcSemanticDisplayBoundaryAssessment {
    TraceCalcSemanticDisplayBoundaryAssessment {
        boundary_status: "boundary.semantic_only_tracecalc_scope".to_string(),
        semantic_surface_only: true,
        format_delta_exercised: false,
        display_delta_exercised: false,
        note: "Current TraceCalc Stage 1 distillation covers semantic surfaces only; format/display delta families remain outside the exercised scope.".to_string(),
    }
}

fn build_distillation_manifest(
    case: &RetainedFailureCase,
    source_scenario: &TraceCalcScenario,
    reduced_scenario: &TraceCalcScenario,
    witness: &TraceCalcWitnessSeed,
    distill_validation: &TraceCalcDistillValidation,
) -> TraceCalcDistillationManifest {
    let selected_step_ids = reduced_scenario
        .steps
        .iter()
        .map(|step| step.step_id.clone())
        .collect::<Vec<_>>();
    let removed_step_ids = source_scenario
        .steps
        .iter()
        .filter(|step| !selected_step_ids.contains(&step.step_id))
        .map(|step| step.step_id.clone())
        .collect::<Vec<_>>();

    TraceCalcDistillationManifest {
        schema_version: DISTILLATION_MANIFEST_SCHEMA_V1.to_string(),
        distillation_id: format!("{}--distilled", case.case_id),
        witness_id: witness.witness_id.clone(),
        case_id: case.case_id.clone(),
        source_scenario_id: source_scenario.scenario_id.clone(),
        source_run_id: case.source_run_id.clone(),
        lifecycle_state: witness.lifecycle.lifecycle_state.clone(),
        selected_unit_ids: witness
            .reduction_manifest
            .units
            .iter()
            .map(|unit| unit.unit_id.clone())
            .collect(),
        selected_step_ids,
        removed_step_ids,
        required_equality_surfaces: case.required_equality_surfaces.clone(),
        target_mismatch_kind: case.target_mismatch_kind.clone(),
        dependency_projection_status: distill_validation
            .dependency_projection
            .projection_status
            .clone(),
        semantic_display_boundary_status: distill_validation
            .semantic_display_boundary
            .boundary_status
            .clone(),
    }
}

fn build_pack_candidate_assessment(
    case: &RetainedFailureCase,
    witness: &TraceCalcWitnessSeed,
    distill_validation: &TraceCalcDistillValidation,
) -> TraceCalcPackCandidateAssessment {
    let mut blocked_by = vec![
        "boundary.semantic_display.unexercised".to_string(),
        "pack.grade.validator.unproven".to_string(),
    ];
    let candidate_state = match witness.lifecycle.lifecycle_state.as_str() {
        "wit.retained_local"
            if case.pack_candidate_rehearsal
                && distill_validation.distill_status == "distill_valid" =>
        {
            "pc.rehearsal_only"
        }
        "wit.explanatory_only" => {
            blocked_by.push("lifecycle.explanatory_only".to_string());
            "pc.blocked_by_lifecycle"
        }
        "wit.quarantined" => {
            blocked_by.push("lifecycle.quarantined".to_string());
            "pc.blocked_by_lifecycle"
        }
        _ => {
            blocked_by.push("distill.validation.not_valid".to_string());
            "pc.blocked_by_distill"
        }
    }
    .to_string();

    TraceCalcPackCandidateAssessment {
        schema_version: PACK_CANDIDATE_ASSESSMENT_SCHEMA_V1.to_string(),
        case_id: case.case_id.clone(),
        lifecycle_state: witness.lifecycle.lifecycle_state.clone(),
        distill_status: distill_validation.distill_status.clone(),
        candidate_state,
        pack_eligible: false,
        replay_valid: distill_validation.reduced_scenario_replay_valid,
        predicate_preserved: distill_validation.predicate_preserved,
        dependency_projection_status: distill_validation
            .dependency_projection
            .projection_status
            .clone(),
        semantic_display_boundary_status: distill_validation
            .semantic_display_boundary
            .boundary_status
            .clone(),
        blocked_by,
    }
}

fn apply_case_lifecycle(
    case: &RetainedFailureCase,
    witness: &mut TraceCalcWitnessSeed,
) -> Result<(), TraceCalcRetainedFailureError> {
    match case.lifecycle_target.as_str() {
        "wit.retained_local" => {
            witness.lifecycle.lifecycle_state = "wit.retained_local".to_string();
            witness.lifecycle.pack_eligible = false;
            witness.lifecycle.replay_validity_assessed = true;
            witness.lifecycle.quarantine_reason = None;
            witness.reduction_manifest.status_id = RETAINED_LOCAL_REDUCTION_STATUS_ID.to_string();
            witness.reduction_manifest.status_scope = RETAINED_STATUS_SCOPE.to_string();
        }
        "wit.explanatory_only" => {
            witness.lifecycle.lifecycle_state = "wit.explanatory_only".to_string();
            witness.lifecycle.pack_eligible = false;
            witness.lifecycle.replay_validity_assessed = true;
            witness.lifecycle.quarantine_reason = None;
            witness.reduction_manifest.status_id = "oxcalc.reduction.explanatory_only".to_string();
            witness.reduction_manifest.status_scope = RETAINED_STATUS_SCOPE.to_string();
        }
        "wit.quarantined" => {
            witness.lifecycle.lifecycle_state = "wit.quarantined".to_string();
            witness.lifecycle.pack_eligible = false;
            witness.lifecycle.replay_validity_assessed = false;
            witness.lifecycle.quarantine_reason = Some(
                case.quarantine_reason
                    .clone()
                    .unwrap_or_else(|| "capture_insufficient".to_string()),
            );
            witness.reduction_manifest.status_id = "oxcalc.reduction.quarantined_local".to_string();
            witness.reduction_manifest.status_scope = RETAINED_STATUS_SCOPE.to_string();
        }
        lifecycle_target => {
            return Err(TraceCalcRetainedFailureError::UnsupportedLifecycleTarget {
                case_id: case.case_id.clone(),
                lifecycle_target: lifecycle_target.to_string(),
            });
        }
    }
    Ok(())
}

fn expand_selected_units(
    case_id: &str,
    units: &[TraceCalcReductionUnit],
    selected_unit_ids: &[String],
) -> Result<BTreeSet<String>, TraceCalcRetainedFailureError> {
    let unit_map = units
        .iter()
        .map(|unit| (unit.unit_id.clone(), unit))
        .collect::<BTreeMap<_, _>>();
    let mut selected = BTreeSet::new();

    for unit_id in selected_unit_ids {
        if !unit_map.contains_key(unit_id) {
            return Err(TraceCalcRetainedFailureError::UnknownReductionUnit {
                case_id: case_id.to_string(),
                unit_id: unit_id.clone(),
            });
        }
        selected.insert(unit_id.clone());
    }

    let mut changed = true;
    while changed {
        changed = false;
        let current = selected.iter().cloned().collect::<Vec<_>>();
        for unit_id in current {
            if let Some(unit) = unit_map.get(&unit_id) {
                for closure_unit_id in &unit.closure_unit_ids {
                    if selected.insert(closure_unit_id.clone()) {
                        changed = true;
                    }
                }
            }
        }
    }

    Ok(selected)
}

fn source_run_artifact_paths(source_run_id: &str, scenario_id: &str) -> Vec<(String, String)> {
    let relative_scenario_root = relative_artifact_path([
        "docs",
        "test-runs",
        "core-engine",
        "tracecalc-reference-machine",
        source_run_id,
        "scenarios",
        scenario_id,
    ]);
    vec![
        (
            "result".to_string(),
            relative_artifact_path([&relative_scenario_root, "result.json"]),
        ),
        (
            "trace".to_string(),
            relative_artifact_path([&relative_scenario_root, "trace.json"]),
        ),
        (
            "counters".to_string(),
            relative_artifact_path([&relative_scenario_root, "counters.json"]),
        ),
        (
            "published_view".to_string(),
            relative_artifact_path([&relative_scenario_root, "published_view.json"]),
        ),
        (
            "pinned_views".to_string(),
            relative_artifact_path([&relative_scenario_root, "pinned_views.json"]),
        ),
        (
            "rejects".to_string(),
            relative_artifact_path([&relative_scenario_root, "rejects.json"]),
        ),
    ]
}

fn load_json<T: for<'de> Deserialize<'de>>(
    path: &Path,
) -> Result<T, TraceCalcRetainedFailureError> {
    let text = fs::read_to_string(path).map_err(|source| TraceCalcRetainedFailureError::Read {
        path: path.display().to_string(),
        source,
    })?;
    serde_json::from_str(&text).map_err(|source| TraceCalcRetainedFailureError::Parse {
        path: path.display().to_string(),
        source,
    })
}

fn create_directory(path: &Path) -> Result<(), TraceCalcRetainedFailureError> {
    fs::create_dir_all(path).map_err(|source| TraceCalcRetainedFailureError::CreateDirectory {
        path: path.display().to_string(),
        source,
    })
}

fn write_json(path: &Path, value: &serde_json::Value) -> Result<(), TraceCalcRetainedFailureError> {
    let text = serde_json::to_string_pretty(value).expect("json serialization should succeed");
    fs::write(path, text).map_err(|source| TraceCalcRetainedFailureError::WriteFile {
        path: path.display().to_string(),
        source,
    })
}

fn write_bundle_capability_snapshot(
    artifact_root: &Path,
    run_id: &str,
) -> Result<(), TraceCalcRetainedFailureError> {
    write_json(
        &artifact_root.join("replay-appliance/adapter_capabilities/oxcalc.json"),
        &json!({
            "schema_version": REPLAY_ADAPTER_CAPABILITY_SNAPSHOT_SCHEMA_V1,
            "adapter_id": "oxcalc-tracecalc-replay-adapter",
            "lane_id": "oxcalc",
            "run_id": run_id,
            "canonical_manifest_ref": "docs/spec/core-engine/CORE_ENGINE_REPLAY_ADAPTER_CAPABILITY_MANIFEST_V1.json",
            "claimed_capability_levels": ["cap.C0.ingest_valid", "cap.C1.replay_valid", "cap.C2.diff_valid", "cap.C3.explain_valid", "cap.C4.distill_valid"],
            "target_capability_levels": ["cap.C5.pack_valid"],
            "projection_scope": "run_local_snapshot_only",
            "known_limits": [
                "oxcalc.local.limit.explain_coverage_is_current_family_only",
                "oxcalc.local.limit.pack_valid_not_proven"
            ],
            "registry_version_ref": FOUNDATION_REPLAY_REGISTRY_VERSION,
        }),
    )
}

#[allow(clippy::too_many_arguments)]
fn write_bundle_case_projection(
    artifact_root: &Path,
    run_id: &str,
    relative_artifact_root: &str,
    case_id: &str,
    case_summary: &TraceCalcRetainedFailureCaseSummary,
    lifecycle: &crate::witness::TraceCalcWitnessLifecycleRecord,
    reduction_manifest: &crate::witness::TraceCalcReductionManifest,
    replay_validation: &TraceCalcRetainedFailureReplayValidation,
    distillation_manifest: &TraceCalcDistillationManifest,
    distill_validation: &TraceCalcDistillValidation,
    pack_candidate_assessment: &TraceCalcPackCandidateAssessment,
    scenario_text: &str,
    reduced_scenario: &TraceCalcScenario,
    source_artifact_paths: &BTreeMap<String, String>,
) -> Result<serde_json::Value, TraceCalcRetainedFailureError> {
    let bundle_case_root = artifact_root
        .join("replay-appliance/runs")
        .join(run_id)
        .join("cases")
        .join(case_id);
    create_directory(&bundle_case_root)?;
    create_directory(&bundle_case_root.join("witness_bundle"))?;

    fs::write(
        bundle_case_root.join("witness_bundle/source_scenario.json"),
        scenario_text,
    )
    .map_err(|source| TraceCalcRetainedFailureError::WriteFile {
        path: bundle_case_root
            .join("witness_bundle/source_scenario.json")
            .display()
            .to_string(),
        source,
    })?;
    write_json(
        &bundle_case_root.join("witness_bundle/reduced_scenario.json"),
        &serde_json::to_value(reduced_scenario).expect("reduced scenario serialization"),
    )?;
    write_json(
        &bundle_case_root.join("lifecycle.json"),
        &serde_json::to_value(lifecycle).expect("lifecycle serialization"),
    )?;
    write_json(
        &bundle_case_root.join("reduction_manifest.json"),
        &serde_json::to_value(reduction_manifest).expect("reduction serialization"),
    )?;
    write_json(
        &bundle_case_root.join("replay_validation.json"),
        &serde_json::to_value(replay_validation).expect("replay validation serialization"),
    )?;
    write_json(
        &bundle_case_root.join("distillation_manifest.json"),
        &serde_json::to_value(distillation_manifest).expect("distillation manifest serialization"),
    )?;
    write_json(
        &bundle_case_root.join("distill_validation.json"),
        &serde_json::to_value(distill_validation).expect("distill validation serialization"),
    )?;
    write_json(
        &bundle_case_root.join("pack_candidate_assessment.json"),
        &serde_json::to_value(pack_candidate_assessment)
            .expect("pack candidate assessment serialization"),
    )?;

    let relative_bundle_root = relative_artifact_path([
        relative_artifact_root,
        "replay-appliance",
        "runs",
        run_id,
        "cases",
        case_id,
    ]);
    let bundle_artifact_paths = BTreeMap::from([
        (
            "lifecycle".to_string(),
            relative_artifact_path([&relative_bundle_root, "lifecycle.json"]),
        ),
        (
            "reduction_manifest".to_string(),
            relative_artifact_path([&relative_bundle_root, "reduction_manifest.json"]),
        ),
        (
            "replay_validation".to_string(),
            relative_artifact_path([&relative_bundle_root, "replay_validation.json"]),
        ),
        (
            "source_scenario".to_string(),
            relative_artifact_path([
                &relative_bundle_root,
                "witness_bundle",
                "source_scenario.json",
            ]),
        ),
        (
            "reduced_scenario".to_string(),
            relative_artifact_path([
                &relative_bundle_root,
                "witness_bundle",
                "reduced_scenario.json",
            ]),
        ),
        (
            "distillation_manifest".to_string(),
            relative_artifact_path([&relative_bundle_root, "distillation_manifest.json"]),
        ),
        (
            "distill_validation".to_string(),
            relative_artifact_path([&relative_bundle_root, "distill_validation.json"]),
        ),
        (
            "pack_candidate_assessment".to_string(),
            relative_artifact_path([&relative_bundle_root, "pack_candidate_assessment.json"]),
        ),
    ]);
    write_json(
        &bundle_case_root.join("case_summary.json"),
        &serde_json::to_value(&TraceCalcRetainedFailureCaseSummary {
            artifact_paths: bundle_artifact_paths.clone(),
            ..case_summary.clone()
        })
        .expect("case summary serialization"),
    )?;
    write_json(
        &bundle_case_root.join("explain.json"),
        &json!({
            "schema_version": REPLAY_EXPLAIN_RECORD_SCHEMA_V1,
            "explain_id": format!("{case_id}--why-diff"),
            "explain_kind": "why_diff",
            "case_id": case_id,
            "source_scenario_id": case_summary.source_scenario_id,
            "lifecycle_state": case_summary.lifecycle_state,
            "source_target_mismatch_kind": replay_validation.target_mismatch_kind,
            "mismatch_kind": normalized_mismatch_kind(&replay_validation.target_mismatch_kind),
            "predicate_preserved": replay_validation.predicate_preserved,
            "replay_validation_assessed": replay_validation.replay_validation_assessed,
            "replay_valid": replay_validation.scenario_replay_valid,
            "distill_status": distill_validation.distill_status,
            "selected_unit_ids": replay_validation.selected_unit_ids,
            "required_equality_surfaces": replay_validation.required_equality_surfaces,
            "source_refs": {
                "lifecycle": bundle_artifact_paths["lifecycle"],
                "reduction_manifest": bundle_artifact_paths["reduction_manifest"],
                "replay_validation": bundle_artifact_paths["replay_validation"],
                "distillation_manifest": bundle_artifact_paths["distillation_manifest"],
                "distill_validation": bundle_artifact_paths["distill_validation"],
            },
        }),
    )?;

    Ok(json!({
        "case_id": case_id,
        "source_scenario_id": case_summary.source_scenario_id,
        "lifecycle_state": case_summary.lifecycle_state,
        "bundle_artifact_paths": bundle_artifact_paths,
        "source_artifact_paths": source_artifact_paths,
        "target_mismatch_kind": replay_validation.target_mismatch_kind,
        "required_equality_surfaces": replay_validation.required_equality_surfaces,
        "explain_path": relative_artifact_path([&relative_bundle_root, "explain.json"]),
    }))
}

fn write_bundle_run_projection(
    artifact_root: &Path,
    run_id: &str,
    relative_artifact_root: &str,
    bundle_cases: &[serde_json::Value],
    summary: &TraceCalcRetainedFailureRunSummary,
) -> Result<(), TraceCalcRetainedFailureError> {
    let replay_root = artifact_root.join("replay-appliance");
    let replay_run_root = replay_root.join("runs").join(run_id);
    write_json(
        &replay_run_root.join("run_manifest.json"),
        &json!({
            "schema_version": REPLAY_RUN_MANIFEST_SCHEMA_V1,
            "run_kind": "tracecalc_retained_failure_run",
            "run_id": run_id,
            "source_artifact_root": relative_artifact_root,
            "source_run_summary_path": relative_artifact_path([relative_artifact_root, "run_summary.json"]),
            "source_case_index_path": relative_artifact_path([relative_artifact_root, "case_index.json"]),
            "cases": bundle_cases,
            "lifecycle_counts": summary.lifecycle_counts,
        }),
    )?;
    write_json(
        &replay_root.join("bundle_manifest.json"),
        &json!({
            "schema_version": REPLAY_BUNDLE_MANIFEST_SCHEMA_V1,
            "bundle_kind": "tracecalc_retained_failure_run",
            "lane_id": "oxcalc",
            "run_id": run_id,
            "source_artifact_root": relative_artifact_root,
            "run_manifest_path": relative_artifact_path([
                relative_artifact_root,
                "replay-appliance",
                "runs",
                run_id,
                "run_manifest.json",
            ]),
            "adapter_capabilities_path": relative_artifact_path([
                relative_artifact_root,
                "replay-appliance",
                "adapter_capabilities",
                "oxcalc.json",
            ]),
            "preserved_view_families": [
                "published_view",
                "pinned_view",
                "reject_set",
                "assertion_result_set",
                "counter_set",
            ],
            "projection_status": "projection_validated_with_explain",
            "registry_version_ref": FOUNDATION_REPLAY_REGISTRY_VERSION,
        }),
    )
}

fn normalized_mismatch_kind(source_kind: &str) -> &str {
    match source_kind {
        "missing_scenario_result" => "mm.scenario.presence",
        "result_state_mismatch" => "mm.result.state",
        "published_view_mismatch" | "pinned_view_mismatch" => "mm.view.value",
        "reject_mismatch" => "mm.reject.kind",
        "trace_count_mismatch" => "mm.trace.event",
        "counter_mismatch" => "mm.counter.value",
        "unexpected_extra_artifact" => "mm.sidecar.payload",
        "dependency_projection_effect" => "oxcalc.local.mm.dependency_projection",
        _ => "oxcalc.local.mm.unknown",
    }
}

fn write_bundle_validation(
    repo_root: &Path,
    artifact_root: &Path,
    run_id: &str,
    bundle_cases: &[serde_json::Value],
) -> Result<(), TraceCalcRetainedFailureError> {
    let mut checked_paths = vec![
        relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "tracecalc-retained-failures",
            run_id,
            "replay-appliance",
            "bundle_manifest.json",
        ]),
        relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "tracecalc-retained-failures",
            run_id,
            "replay-appliance",
            "adapter_capabilities",
            "oxcalc.json",
        ]),
        relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "tracecalc-retained-failures",
            run_id,
            "replay-appliance",
            "runs",
            run_id,
            "run_manifest.json",
        ]),
        relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "tracecalc-retained-failures",
            run_id,
            "replay-appliance",
            "validation",
            "distill_validation.json",
        ]),
        relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "tracecalc-retained-failures",
            run_id,
            "replay-appliance",
            "validation",
            "pack_candidate_validation.json",
        ]),
    ];
    for case in bundle_cases {
        if let Some(paths) = case["bundle_artifact_paths"].as_object() {
            checked_paths.extend(
                paths
                    .values()
                    .filter_map(|value| value.as_str())
                    .map(str::to_string),
            );
        }
        if let Some(explain_path) = case["explain_path"].as_str() {
            checked_paths.push(explain_path.to_string());
        }
    }

    let missing_paths = checked_paths
        .iter()
        .filter(|path| !repo_root.join(path).exists())
        .cloned()
        .collect::<Vec<_>>();

    write_json(
        &artifact_root.join("replay-appliance/validation/bundle_validation.json"),
        &json!({
            "schema_version": REPLAY_BUNDLE_VALIDATION_SCHEMA_V1,
            "bundle_kind": "tracecalc_retained_failure_run",
            "run_id": run_id,
            "status": if missing_paths.is_empty() { "bundle_valid" } else { "bundle_degraded" },
            "degraded_capture": !missing_paths.is_empty(),
            "checked_paths": checked_paths,
            "missing_paths": missing_paths,
        }),
    )
}

fn write_distill_validation(
    artifact_root: &Path,
    run_id: &str,
    distill_cases: &[serde_json::Value],
) -> Result<(), TraceCalcRetainedFailureError> {
    let distill_valid_count = distill_cases
        .iter()
        .filter(|entry| entry["distill_status"] == "distill_valid")
        .count();
    write_json(
        &artifact_root.join("replay-appliance/validation/distill_validation.json"),
        &json!({
            "schema_version": DISTILL_VALIDATION_SCHEMA_V1,
            "run_id": run_id,
            "status": if distill_valid_count > 0 { "cap.C4.distill_valid" } else { "distill_degraded" },
            "distill_valid_case_count": distill_valid_count,
            "cases": distill_cases,
        }),
    )
}

fn write_pack_candidate_validation(
    artifact_root: &Path,
    run_id: &str,
    pack_candidate_cases: &[serde_json::Value],
) -> Result<(), TraceCalcRetainedFailureError> {
    let rehearsal_count = pack_candidate_cases
        .iter()
        .filter(|entry| entry["candidate_state"] == "pc.rehearsal_only")
        .count();
    write_json(
        &artifact_root.join("replay-appliance/validation/pack_candidate_validation.json"),
        &json!({
            "schema_version": PACK_CANDIDATE_VALIDATION_SCHEMA_V1,
            "run_id": run_id,
            "status": "pack_candidate_rehearsed_only",
            "pack_valid": false,
            "rehearsal_case_count": rehearsal_count,
            "blocked_by": [
                "boundary.semantic_display.unexercised",
                "pack.grade.validator.unproven"
            ],
            "cases": pack_candidate_cases,
        }),
    )
}

fn relative_artifact_path<'a>(segments: impl IntoIterator<Item = &'a str>) -> String {
    segments
        .into_iter()
        .filter(|segment| !segment.trim().is_empty())
        .map(|segment| segment.replace('\\', "/").trim_matches('/').to_string())
        .collect::<Vec<_>>()
        .join("/")
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn retained_failure_manifest_emits_lifecycle_variety() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let run_id = format!("test-retained-failure-{}", std::process::id());
        let artifact_root = repo_root.join(format!(
            "docs/test-runs/core-engine/tracecalc-retained-failures/{run_id}"
        ));
        let runner = TraceCalcRetainedFailureRunner::new();

        let cleanup = || {
            if artifact_root.exists() {
                let _ = fs::remove_dir_all(&artifact_root);
            }
        };

        cleanup();
        let summary = runner.execute_manifest(&repo_root, &run_id).unwrap();
        assert_eq!(summary.case_count, 4);
        assert!(
            artifact_root
                .join("replay-appliance/bundle_manifest.json")
                .exists()
        );
        assert!(
            artifact_root
                .join("replay-appliance/adapter_capabilities/oxcalc.json")
                .exists()
        );
        assert!(
            artifact_root
                .join(format!("replay-appliance/runs/{run_id}/run_manifest.json"))
                .exists()
        );
        assert!(
            artifact_root
                .join(format!(
                    "replay-appliance/runs/{run_id}/cases/rf_publication_fence_retained_local_001/lifecycle.json"
                ))
                .exists()
        );
        assert!(
            artifact_root
                .join("replay-appliance/validation/bundle_validation.json")
                .exists()
        );
        assert!(
            artifact_root
                .join("replay-appliance/validation/distill_validation.json")
                .exists()
        );
        assert!(
            artifact_root
                .join("replay-appliance/validation/pack_candidate_validation.json")
                .exists()
        );
        assert!(
            artifact_root
                .join(format!(
                    "replay-appliance/runs/{run_id}/cases/rf_publication_fence_retained_local_001/explain.json"
                ))
                .exists()
        );
        assert!(
            artifact_root
                .join("cases/rf_publication_fence_retained_local_001/lifecycle.json")
                .exists()
        );
        let retained_lifecycle = load_json::<serde_json::Value>(
            &artifact_root.join("cases/rf_publication_fence_retained_local_001/lifecycle.json"),
        )
        .unwrap();
        assert_eq!(retained_lifecycle["lifecycle_state"], "wit.retained_local");

        let explanatory_lifecycle = load_json::<serde_json::Value>(
            &artifact_root.join("cases/rf_fallback_explanatory_only_001/lifecycle.json"),
        )
        .unwrap();
        assert_eq!(
            explanatory_lifecycle["lifecycle_state"],
            "wit.explanatory_only"
        );

        let quarantined_lifecycle = load_json::<serde_json::Value>(
            &artifact_root.join("cases/rf_verify_clean_quarantined_001/lifecycle.json"),
        )
        .unwrap();
        assert_eq!(quarantined_lifecycle["lifecycle_state"], "wit.quarantined");

        let replay_validation = load_json::<serde_json::Value>(
            &artifact_root
                .join("cases/rf_publication_fence_retained_local_001/replay_validation.json"),
        )
        .unwrap();
        assert_eq!(replay_validation["scenario_replay_valid"], true);
        assert_eq!(replay_validation["predicate_preserved"], true);

        let bundle_validation = load_json::<serde_json::Value>(
            &artifact_root.join("replay-appliance/validation/bundle_validation.json"),
        )
        .unwrap();
        assert_eq!(bundle_validation["status"], "bundle_valid");

        let distill_validation = load_json::<serde_json::Value>(
            &artifact_root.join("replay-appliance/validation/distill_validation.json"),
        )
        .unwrap();
        assert_eq!(distill_validation["status"], "cap.C4.distill_valid");

        let pack_candidate_validation = load_json::<serde_json::Value>(
            &artifact_root.join("replay-appliance/validation/pack_candidate_validation.json"),
        )
        .unwrap();
        assert_eq!(
            pack_candidate_validation["status"],
            "pack_candidate_rehearsed_only"
        );

        let explain_record = load_json::<serde_json::Value>(
            &artifact_root.join(format!(
                "replay-appliance/runs/{run_id}/cases/rf_publication_fence_retained_local_001/explain.json"
            )),
        )
        .unwrap();
        assert_eq!(explain_record["explain_kind"], "why_diff");
        assert_eq!(explain_record["mismatch_kind"], "mm.reject.kind");

        let reduced_scenario = load_json::<serde_json::Value>(&artifact_root.join(
            "cases/rf_dynamic_dependency_retained_local_001/witness_bundle/reduced_scenario.json",
        ))
        .unwrap();
        assert_eq!(
            reduced_scenario["scenario_id"],
            "rf_dynamic_dependency_retained_local_001--distilled"
        );

        let dynamic_distill = load_json::<serde_json::Value>(
            &artifact_root
                .join("cases/rf_dynamic_dependency_retained_local_001/distill_validation.json"),
        )
        .unwrap();
        assert_eq!(dynamic_distill["distill_status"], "distill_valid");
        assert_eq!(
            dynamic_distill["dependency_projection"]["projection_status"],
            "dp.projected_in_reduced_witness"
        );

        cleanup();
    }
}
