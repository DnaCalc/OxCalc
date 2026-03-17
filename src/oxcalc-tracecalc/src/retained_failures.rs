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

        write_json(
            &artifact_root.join("manifest_selection.json"),
            &json!(manifest.cases),
        )?;

        let mut lifecycle_counts = BTreeMap::new();
        let mut case_summaries = Vec::new();

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

            let case_directory = artifact_root.join("cases").join(&case.case_id);
            create_directory(&case_directory)?;
            let witness_bundle_directory = case_directory.join("witness_bundle");
            create_directory(&witness_bundle_directory)?;

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
                witness_bundle_directory.join("scenario.json"),
                scenario_text,
            )
            .map_err(|source| TraceCalcRetainedFailureError::WriteFile {
                path: witness_bundle_directory
                    .join("scenario.json")
                    .display()
                    .to_string(),
                source,
            })?;

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
                    "witness_bundle_scenario".to_string(),
                    relative_artifact_path([
                        &relative_artifact_root,
                        "cases",
                        &case.case_id,
                        "witness_bundle",
                        "scenario.json",
                    ]),
                ),
            ]);

            write_json(
                &case_directory.join("case_summary.json"),
                &serde_json::to_value(&TraceCalcRetainedFailureCaseSummary {
                    case_id: case.case_id.clone(),
                    description: case.description.clone(),
                    source_scenario_id: scenario.scenario_id.clone(),
                    lifecycle_state: witness.lifecycle.lifecycle_state.clone(),
                    replay_validation_assessed: replay_validation.replay_validation_assessed,
                    replay_valid: replay_validation.scenario_replay_valid,
                    predicate_preserved: replay_validation.predicate_preserved,
                    artifact_paths: case_artifact_paths.clone(),
                    notes: case.notes.clone(),
                })
                .expect("case summary serialization"),
            )?;

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
    let predicate_preserved = match case.preservation_check.kind.as_str() {
        "reject_id_present" => {
            scenario
                .expected
                .rejects
                .iter()
                .any(|reject| reject.reject_id == case.preservation_check.reference)
                && selected_units.iter().any(|unit| {
                    unit.reject_id.as_deref() == Some(case.preservation_check.reference.as_str())
                })
        }
        "counter_present" => {
            scenario
                .expected
                .counter_expectations
                .iter()
                .any(|counter| counter.counter == case.preservation_check.reference)
                && selected_unit_ids.iter().any(|unit_id| {
                    unit_id == "scenario"
                        || unit_id.starts_with("phase:")
                        || unit_id.starts_with("events:")
                })
        }
        "published_view_present" => {
            !scenario.expected.published_view.snapshot_id.is_empty()
                && selected_unit_ids
                    .iter()
                    .any(|unit_id| unit_id == "scenario" || unit_id.starts_with("view:"))
        }
        kind => {
            return Err(
                TraceCalcRetainedFailureError::UnsupportedPreservationCheck {
                    case_id: case.case_id.clone(),
                    kind: kind.to_string(),
                },
            );
        }
    };

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
        assert_eq!(summary.case_count, 3);
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

        cleanup();
    }
}
