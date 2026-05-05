#![forbid(unsafe_code)]

//! `TraceCalc` runner and artifact emission.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde_json::json;
use thiserror::Error;

use crate::assertions::{compare_artifacts, to_snake_case};
use crate::contracts::{
    TraceCalcConformanceMismatch, TraceCalcExecutionArtifacts, TraceCalcLoadError,
    TraceCalcRunSummary, TraceCalcScenario, TraceCalcScenarioResult, TraceCalcScenarioResultState,
    TraceCalcValidationFailure, TraceCalcValidationFailureKind, load_manifest, load_scenario,
    validate_scenario,
};
use crate::machine::{TraceCalcEngineMachine, TraceCalcReferenceMachine};
use crate::replay_mappings::{
    normalize_event_family, registry_mismatch_kind, required_equality_surface, severity_class,
};
use crate::witness::{TraceCalcWitnessSeedInputs, build_witness_seed};

const REPLAY_BUNDLE_MANIFEST_SCHEMA_V1: &str = "oxcalc.local.replay_bundle_manifest.v1";
const REPLAY_RUN_MANIFEST_SCHEMA_V1: &str = "oxcalc.local.replay_run_manifest.v1";
const REPLAY_ADAPTER_CAPABILITY_SNAPSHOT_SCHEMA_V1: &str =
    "oxcalc.local.adapter_capability_snapshot.v1";
const REPLAY_BUNDLE_VALIDATION_SCHEMA_V1: &str = "oxcalc.local.replay_bundle_validation.v1";
const REPLAY_EXPLAIN_RECORD_SCHEMA_V1: &str = "oxcalc.local.replay_explain_record.v1";
const FOUNDATION_REPLAY_REGISTRY_VERSION: &str =
    "foundation.replay.authoritative-pass-01.2026-03-15";

#[derive(Debug, Error)]
pub enum TraceCalcRunnerError {
    #[error(transparent)]
    Load(#[from] TraceCalcLoadError),
    #[error("failed to create artifact root {path}: {source}")]
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
}

#[derive(Debug, Default)]
pub struct TraceCalcRunner {
    reference_machine: TraceCalcReferenceMachine,
    engine_machine: TraceCalcEngineMachine,
}

impl TraceCalcRunner {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn execute_manifest(
        &self,
        repo_root: &Path,
        run_id: &str,
        scenario_filter: Option<&str>,
        tags: Option<&[String]>,
    ) -> Result<TraceCalcRunSummary, TraceCalcRunnerError> {
        let manifest_path = repo_root.join("docs/test-corpus/core-engine/tracecalc/MANIFEST.json");
        let manifest = load_manifest(&manifest_path)?;
        let selected_scenarios = manifest
            .scenarios
            .iter()
            .filter(|entry| {
                scenario_filter.is_none_or(|scenario_id| entry.scenario_id == scenario_id)
            })
            .filter(|entry| {
                tags.is_none_or(|required_tags| {
                    required_tags
                        .iter()
                        .any(|tag| entry.tags.iter().any(|candidate| candidate == tag))
                })
            })
            .cloned()
            .collect::<Vec<_>>();

        let artifact_root = repo_root.join(format!(
            "docs/test-runs/core-engine/tracecalc-reference-machine/{run_id}"
        ));
        let relative_artifact_root = relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "tracecalc-reference-machine",
            run_id,
        ]);
        if artifact_root.exists() {
            fs::remove_dir_all(&artifact_root).map_err(|source| {
                TraceCalcRunnerError::RemoveDirectory {
                    path: artifact_root.display().to_string(),
                    source,
                }
            })?;
        }

        create_directory(&artifact_root)?;
        create_directory(&artifact_root.join("scenarios"))?;
        create_directory(&artifact_root.join("conformance"))?;
        create_directory(&artifact_root.join("replay-appliance"))?;
        create_directory(&artifact_root.join("replay-appliance/adapter_capabilities"))?;
        create_directory(&artifact_root.join("replay-appliance/reductions"))?;
        create_directory(&artifact_root.join("replay-appliance/runs"))?;
        create_directory(&artifact_root.join("replay-appliance/validation"))?;
        create_directory(&artifact_root.join("replay-appliance/runs").join(run_id))?;
        create_directory(
            &artifact_root
                .join("replay-appliance/runs")
                .join(run_id)
                .join("scenarios"),
        )?;
        create_directory(
            &artifact_root
                .join("replay-appliance/runs")
                .join(run_id)
                .join("diff"),
        )?;
        create_directory(
            &artifact_root
                .join("replay-appliance/runs")
                .join(run_id)
                .join("oracle"),
        )?;
        create_directory(&artifact_root.join("replay-appliance/witnesses"))?;
        write_bundle_capability_snapshot(&artifact_root, run_id)?;

        write_json(
            &artifact_root.join("manifest_selection.json"),
            &json!(
                selected_scenarios
                    .iter()
                    .map(|entry| {
                        json!({
                            "scenario_id": entry.scenario_id,
                            "path": entry.path,
                            "focus": entry.focus,
                            "tags": entry.tags,
                        })
                    })
                    .collect::<Vec<_>>()
            ),
        )?;

        let mut scenario_results = Vec::new();
        let mut oracle_baseline = Vec::new();
        let mut engine_diff = Vec::new();
        let mut bundle_scenarios = Vec::new();

        for entry in &selected_scenarios {
            let scenario_directory = artifact_root.join("scenarios").join(&entry.scenario_id);
            create_directory(&scenario_directory)?;
            let scenario_path = repo_root
                .join("docs/test-corpus/core-engine/tracecalc")
                .join(entry.path.replace('/', "\\"));

            let mut assertion_failures = Vec::new();
            let loaded = load_scenario(&scenario_path);
            match loaded {
                Ok(scenario) => {
                    let validation_failures = validate_scenario(entry, &scenario);
                    if validation_failures.is_empty() {
                        let oracle_outcome = self.reference_machine.execute(&scenario);
                        let engine_outcome = self.engine_machine.execute(&scenario);
                        match (oracle_outcome, engine_outcome) {
                            (Ok(oracle_artifacts), Ok(engine_artifacts)) => {
                                let conformance_mismatches =
                                    compare_artifacts(&oracle_artifacts, &engine_artifacts);
                                assertion_failures
                                    .extend(oracle_artifacts.assertion_failures.clone());
                                assertion_failures.extend(
                                    engine_artifacts
                                        .assertion_failures
                                        .iter()
                                        .map(|message| format!("engine: {message}")),
                                );
                                let result_state = if assertion_failures.is_empty()
                                    && conformance_mismatches.is_empty()
                                {
                                    TraceCalcScenarioResultState::Passed
                                } else {
                                    TraceCalcScenarioResultState::FailedAssertion
                                };
                                let artifact_paths = all_artifact_paths(
                                    &relative_artifact_root,
                                    Some(&scenario),
                                    &entry.scenario_id,
                                );
                                write_scenario_artifacts(
                                    &scenario_directory,
                                    run_id,
                                    Some(&scenario),
                                    &entry.scenario_id,
                                    result_state,
                                    &validation_failures,
                                    &assertion_failures,
                                    &conformance_mismatches,
                                    &oracle_artifacts,
                                    &artifact_paths,
                                )?;
                                write_witness_seed_artifacts(
                                    &artifact_root,
                                    TraceCalcWitnessSeedInputs {
                                        run_id,
                                        relative_artifact_root: &relative_artifact_root,
                                        scenario: &scenario,
                                        result_state,
                                        validation_failures: &validation_failures,
                                        assertion_failures: &assertion_failures,
                                        scenario_artifact_paths: &artifact_paths,
                                        conformance_mismatches: &conformance_mismatches,
                                    },
                                )?;
                                bundle_scenarios.push(write_bundle_scenario_projection(
                                    &artifact_root,
                                    run_id,
                                    Some(&scenario),
                                    &entry.scenario_id,
                                    result_state,
                                    &validation_failures,
                                    &assertion_failures,
                                    &conformance_mismatches,
                                    &oracle_artifacts,
                                    &artifact_paths,
                                )?);
                                oracle_baseline.push(oracle_baseline_object(
                                    &entry.scenario_id,
                                    &oracle_artifacts,
                                ));
                                engine_diff.push(engine_diff_object(
                                    &entry.scenario_id,
                                    &oracle_artifacts,
                                    &engine_artifacts,
                                    &conformance_mismatches,
                                ));
                                scenario_results.push(TraceCalcScenarioResult {
                                    scenario_id: entry.scenario_id.clone(),
                                    result_state,
                                    validation_failures,
                                    assertion_failures: assertion_failures.clone(),
                                    conformance_mismatches,
                                    artifact_paths,
                                });
                            }
                            (Err(error), _) | (_, Err(error)) => {
                                let validation = vec![TraceCalcValidationFailure {
                                    kind: TraceCalcValidationFailureKind::JsonParseFailure,
                                    message: error.to_string(),
                                }];
                                let empty = create_empty_artifacts(
                                    &entry.scenario_id,
                                    TraceCalcScenarioResultState::ExecutionError,
                                );
                                let artifact_paths = all_artifact_paths(
                                    &relative_artifact_root,
                                    None,
                                    &entry.scenario_id,
                                );
                                write_scenario_artifacts(
                                    &scenario_directory,
                                    run_id,
                                    None,
                                    &entry.scenario_id,
                                    TraceCalcScenarioResultState::ExecutionError,
                                    &validation,
                                    &assertion_failures,
                                    &[],
                                    &empty,
                                    &artifact_paths,
                                )?;
                                bundle_scenarios.push(write_bundle_scenario_projection(
                                    &artifact_root,
                                    run_id,
                                    None,
                                    &entry.scenario_id,
                                    TraceCalcScenarioResultState::ExecutionError,
                                    &validation,
                                    &assertion_failures,
                                    &[],
                                    &empty,
                                    &artifact_paths,
                                )?);
                                oracle_baseline
                                    .push(oracle_baseline_object(&entry.scenario_id, &empty));
                                engine_diff.push(json!({
                                    "scenario_id": entry.scenario_id,
                                    "oracle_result_state": to_snake_case("ExecutionError"),
                                    "engine_result_state": to_snake_case("ExecutionError"),
                                    "mismatches": [],
                                }));
                                scenario_results.push(TraceCalcScenarioResult {
                                    scenario_id: entry.scenario_id.clone(),
                                    result_state: TraceCalcScenarioResultState::ExecutionError,
                                    validation_failures: validation,
                                    assertion_failures,
                                    conformance_mismatches: Vec::new(),
                                    artifact_paths,
                                });
                            }
                        }
                    } else {
                        let empty = create_empty_artifacts(
                            &entry.scenario_id,
                            TraceCalcScenarioResultState::InvalidScenario,
                        );
                        let artifact_paths = all_artifact_paths(
                            &relative_artifact_root,
                            Some(&scenario),
                            &entry.scenario_id,
                        );
                        write_scenario_artifacts(
                            &scenario_directory,
                            run_id,
                            Some(&scenario),
                            &entry.scenario_id,
                            TraceCalcScenarioResultState::InvalidScenario,
                            &validation_failures,
                            &assertion_failures,
                            &[],
                            &empty,
                            &artifact_paths,
                        )?;
                        write_witness_seed_artifacts(
                            &artifact_root,
                            TraceCalcWitnessSeedInputs {
                                run_id,
                                relative_artifact_root: &relative_artifact_root,
                                scenario: &scenario,
                                result_state: TraceCalcScenarioResultState::InvalidScenario,
                                validation_failures: &validation_failures,
                                assertion_failures: &assertion_failures,
                                scenario_artifact_paths: &artifact_paths,
                                conformance_mismatches: &[],
                            },
                        )?;
                        bundle_scenarios.push(write_bundle_scenario_projection(
                            &artifact_root,
                            run_id,
                            Some(&scenario),
                            &entry.scenario_id,
                            TraceCalcScenarioResultState::InvalidScenario,
                            &validation_failures,
                            &assertion_failures,
                            &[],
                            &empty,
                            &artifact_paths,
                        )?);
                        oracle_baseline.push(oracle_baseline_object(&entry.scenario_id, &empty));
                        engine_diff.push(json!({
                            "scenario_id": entry.scenario_id,
                            "oracle_result_state": to_snake_case("InvalidScenario"),
                            "engine_result_state": to_snake_case("InvalidScenario"),
                            "mismatches": [],
                        }));
                        scenario_results.push(TraceCalcScenarioResult {
                            scenario_id: entry.scenario_id.clone(),
                            result_state: TraceCalcScenarioResultState::InvalidScenario,
                            validation_failures,
                            assertion_failures,
                            conformance_mismatches: Vec::new(),
                            artifact_paths,
                        });
                    }
                }
                Err(error) => {
                    let validation = vec![TraceCalcValidationFailure {
                        kind: TraceCalcValidationFailureKind::JsonParseFailure,
                        message: error.to_string(),
                    }];
                    let empty = create_empty_artifacts(
                        &entry.scenario_id,
                        TraceCalcScenarioResultState::ExecutionError,
                    );
                    let artifact_paths =
                        all_artifact_paths(&relative_artifact_root, None, &entry.scenario_id);
                    write_scenario_artifacts(
                        &scenario_directory,
                        run_id,
                        None,
                        &entry.scenario_id,
                        TraceCalcScenarioResultState::ExecutionError,
                        &validation,
                        &assertion_failures,
                        &[],
                        &empty,
                        &artifact_paths,
                    )?;
                    bundle_scenarios.push(write_bundle_scenario_projection(
                        &artifact_root,
                        run_id,
                        None,
                        &entry.scenario_id,
                        TraceCalcScenarioResultState::ExecutionError,
                        &validation,
                        &assertion_failures,
                        &[],
                        &empty,
                        &artifact_paths,
                    )?);
                    oracle_baseline.push(oracle_baseline_object(&entry.scenario_id, &empty));
                    engine_diff.push(json!({
                        "scenario_id": entry.scenario_id,
                        "oracle_result_state": to_snake_case("ExecutionError"),
                        "engine_result_state": to_snake_case("ExecutionError"),
                        "mismatches": [],
                    }));
                    scenario_results.push(TraceCalcScenarioResult {
                        scenario_id: entry.scenario_id.clone(),
                        result_state: TraceCalcScenarioResultState::ExecutionError,
                        validation_failures: validation,
                        assertion_failures,
                        conformance_mismatches: Vec::new(),
                        artifact_paths,
                    });
                }
            }
        }

        write_json(
            &artifact_root.join("conformance/oracle_baseline.json"),
            &json!(oracle_baseline),
        )?;
        write_json(
            &artifact_root.join("conformance/engine_diff.json"),
            &json!(engine_diff),
        )?;
        write_bundle_run_projection(
            &artifact_root,
            run_id,
            &relative_artifact_root,
            &bundle_scenarios,
            &oracle_baseline,
            &engine_diff,
        )?;
        write_bundle_explain_records(&artifact_root, run_id, &engine_diff)?;
        write_bundle_validation(repo_root, &artifact_root, run_id, &bundle_scenarios)?;

        let mut result_counts = BTreeMap::new();
        for result in &scenario_results {
            *result_counts
                .entry(to_snake_case(&format!("{:?}", result.result_state)))
                .or_insert(0_usize) += 1;
        }
        let summary = TraceCalcRunSummary {
            run_id: run_id.to_string(),
            schema_version: manifest.schema_version,
            scenario_count: scenario_results.len(),
            result_counts: result_counts.into_iter().collect(),
            artifact_root: artifact_root.display().to_string(),
        };
        write_json(
            &artifact_root.join("run_summary.json"),
            &json!({
                "run_id": summary.run_id,
                "schema_version": summary.schema_version,
                "scenario_count": summary.scenario_count,
                "result_counts": BTreeMap::from_iter(summary.result_counts.clone()),
                "artifact_root": relative_artifact_root,
            }),
        )?;
        Ok(summary)
    }
}

fn create_empty_artifacts(
    scenario_id: &str,
    state: TraceCalcScenarioResultState,
) -> TraceCalcExecutionArtifacts {
    TraceCalcExecutionArtifacts {
        scenario_id: scenario_id.to_string(),
        result_state: state,
        assertion_failures: Vec::new(),
        trace_events: Vec::new(),
        counters: Vec::new(),
        published_values: Vec::new(),
        pinned_views: Vec::new(),
        rejects: Vec::new(),
    }
}

fn create_directory(path: &Path) -> Result<(), TraceCalcRunnerError> {
    fs::create_dir_all(path).map_err(|source| TraceCalcRunnerError::CreateDirectory {
        path: path.display().to_string(),
        source,
    })
}

fn write_json(path: &Path, value: &serde_json::Value) -> Result<(), TraceCalcRunnerError> {
    let text = serde_json::to_string_pretty(value).expect("json serialization should succeed");
    fs::write(path, text).map_err(|source| TraceCalcRunnerError::WriteFile {
        path: path.display().to_string(),
        source,
    })
}

fn write_jsonl(path: &Path, lines: &[String]) -> Result<(), TraceCalcRunnerError> {
    let mut text = lines.join("\n");
    if !text.is_empty() {
        text.push('\n');
    }
    fs::write(path, text).map_err(|source| TraceCalcRunnerError::WriteFile {
        path: path.display().to_string(),
        source,
    })
}

fn write_bundle_capability_snapshot(
    artifact_root: &Path,
    run_id: &str,
) -> Result<(), TraceCalcRunnerError> {
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
fn write_bundle_scenario_projection(
    artifact_root: &Path,
    run_id: &str,
    scenario: Option<&TraceCalcScenario>,
    scenario_id: &str,
    result_state: TraceCalcScenarioResultState,
    validation_failures: &[TraceCalcValidationFailure],
    assertion_failures: &[String],
    conformance_mismatches: &[TraceCalcConformanceMismatch],
    artifacts: &TraceCalcExecutionArtifacts,
    artifact_paths: &[(String, String)],
) -> Result<serde_json::Value, TraceCalcRunnerError> {
    let bundle_scenario_root = artifact_root
        .join("replay-appliance/runs")
        .join(run_id)
        .join("scenarios")
        .join(scenario_id);
    create_directory(&bundle_scenario_root)?;
    create_directory(&bundle_scenario_root.join("views"))?;

    let relative_bundle_root = relative_artifact_path([
        "docs",
        "test-runs",
        "core-engine",
        "tracecalc-reference-machine",
        run_id,
        "replay-appliance",
        "runs",
        run_id,
        "scenarios",
        scenario_id,
    ]);

    write_json(
        &bundle_scenario_root.join("result.json"),
        &json!({
            "scenario_id": scenario_id,
            "result_state": to_snake_case(&format!("{result_state:?}")),
            "validation_failures": validation_failures.iter().map(|failure| json!({
                "kind": to_snake_case(&format!("{:?}", failure.kind)),
                "message": failure.message,
            })).collect::<Vec<_>>(),
            "assertion_failures": assertion_failures,
            "conformance_mismatches": conformance_mismatches.iter().map(mismatch_object).collect::<Vec<_>>(),
            "replay_projection": scenario.and_then(|scenario| scenario.replay_projection.as_ref()).map(|projection| json!({
                "replay_classes": projection.replay_classes,
                "pack_bindings": projection.pack_bindings,
                "required_equality_surfaces": projection.required_equality_surfaces,
                "normalized_event_family_map_ref": projection.normalized_event_family_map_ref,
                "safety_properties": projection.safety_properties,
                "transition_labels": projection.transition_labels,
            })),
            "source_artifact_paths": BTreeMap::from_iter(artifact_paths.iter().cloned()),
        }),
    )?;
    write_jsonl(
        &bundle_scenario_root.join("events.jsonl"),
        &artifacts
            .trace_events
            .iter()
            .map(|event| {
                serde_json::to_string(&json!({
                    "event_id": event.event_id,
                    "step_id": event.step_id,
                    "source_label": event.label,
                    "normalized_event_family": normalize_event_family(&event.label),
                    "payload": BTreeMap::from_iter(event.payload.clone()),
                }))
                .expect("event serialization should succeed")
            })
            .collect::<Vec<_>>(),
    )?;
    write_json(
        &bundle_scenario_root.join("counters.json"),
        &json!({
            "scenario_id": scenario_id,
            "counters": counter_entries(&artifacts.counters),
        }),
    )?;
    write_json(
        &bundle_scenario_root.join("views/published_view.json"),
        &json!({
            "scenario_id": scenario_id,
            "snapshot_id": scenario.map(|scenario| scenario.initial_graph.snapshot_id.clone()).unwrap_or_default(),
            "node_values": value_entries(&artifacts.published_values),
        }),
    )?;
    write_json(
        &bundle_scenario_root.join("views/pinned_views.json"),
        &json!({
            "scenario_id": scenario_id,
            "views": artifacts.pinned_views.iter().map(|view| json!({
                "view_id": view.view_id,
                "snapshot_id": view.snapshot_id,
                "node_values": value_entries(&view.node_values),
            })).collect::<Vec<_>>(),
        }),
    )?;
    write_json(
        &bundle_scenario_root.join("views/reject_set.json"),
        &json!({
            "scenario_id": scenario_id,
            "rejects": artifacts.rejects.iter().map(|reject| json!({
                "reject_id": reject.reject_id,
                "reject_kind": reject.reject_kind,
                "reject_detail": reject.reject_detail,
            })).collect::<Vec<_>>(),
        }),
    )?;

    Ok(json!({
        "scenario_id": scenario_id,
        "result_state": to_snake_case(&format!("{result_state:?}")),
        "replay_classes": scenario
            .and_then(|scenario| scenario.replay_projection.as_ref())
            .map(|projection| projection.replay_classes.clone())
            .unwrap_or_default(),
        "required_equality_surfaces": scenario
            .and_then(|scenario| scenario.replay_projection.as_ref())
            .map(|projection| projection.required_equality_surfaces.clone())
            .unwrap_or_default(),
        "bundle_artifact_paths": {
            "result": relative_artifact_path([&relative_bundle_root, "result.json"]),
            "events": relative_artifact_path([&relative_bundle_root, "events.jsonl"]),
            "counters": relative_artifact_path([&relative_bundle_root, "counters.json"]),
            "published_view": relative_artifact_path([&relative_bundle_root, "views", "published_view.json"]),
            "pinned_views": relative_artifact_path([&relative_bundle_root, "views", "pinned_views.json"]),
            "reject_set": relative_artifact_path([&relative_bundle_root, "views", "reject_set.json"]),
        },
        "source_artifact_paths": BTreeMap::from_iter(artifact_paths.iter().cloned()),
    }))
}

fn write_bundle_run_projection(
    artifact_root: &Path,
    run_id: &str,
    relative_artifact_root: &str,
    bundle_scenarios: &[serde_json::Value],
    oracle_baseline: &[serde_json::Value],
    engine_diff: &[serde_json::Value],
) -> Result<(), TraceCalcRunnerError> {
    let replay_root = artifact_root.join("replay-appliance");
    let replay_run_root = replay_root.join("runs").join(run_id);

    write_json(
        &replay_run_root.join("oracle/oracle_baseline.json"),
        &json!(oracle_baseline),
    )?;
    write_json(
        &replay_run_root.join("diff/engine_diff.json"),
        &json!(engine_diff),
    )?;
    write_json(
        &replay_run_root.join("run_manifest.json"),
        &json!({
            "schema_version": REPLAY_RUN_MANIFEST_SCHEMA_V1,
            "run_kind": "tracecalc_reference_run",
            "run_id": run_id,
            "source_artifact_root": relative_artifact_root,
            "source_run_summary_path": relative_artifact_path([relative_artifact_root, "run_summary.json"]),
            "source_manifest_selection_path": relative_artifact_path([relative_artifact_root, "manifest_selection.json"]),
            "source_oracle_baseline_path": relative_artifact_path([relative_artifact_root, "conformance", "oracle_baseline.json"]),
            "source_engine_diff_path": relative_artifact_path([relative_artifact_root, "conformance", "engine_diff.json"]),
            "bundle_diff_path": relative_artifact_path([
                relative_artifact_root,
                "replay-appliance",
                "runs",
                run_id,
                "diff",
                "engine_diff.json",
            ]),
            "scenarios": bundle_scenarios,
        }),
    )?;
    write_json(
        &replay_root.join("bundle_manifest.json"),
        &json!({
            "schema_version": REPLAY_BUNDLE_MANIFEST_SCHEMA_V1,
            "bundle_kind": "tracecalc_reference_run",
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

fn write_bundle_validation(
    repo_root: &Path,
    artifact_root: &Path,
    run_id: &str,
    bundle_scenarios: &[serde_json::Value],
) -> Result<(), TraceCalcRunnerError> {
    let mut checked_paths = vec![
        relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "tracecalc-reference-machine",
            run_id,
            "replay-appliance",
            "bundle_manifest.json",
        ]),
        relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "tracecalc-reference-machine",
            run_id,
            "replay-appliance",
            "adapter_capabilities",
            "oxcalc.json",
        ]),
        relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "tracecalc-reference-machine",
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
            "tracecalc-reference-machine",
            run_id,
            "replay-appliance",
            "runs",
            run_id,
            "diff",
            "engine_diff.json",
        ]),
        relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "tracecalc-reference-machine",
            run_id,
            "replay-appliance",
            "runs",
            run_id,
            "diff",
            "explain_records.json",
        ]),
    ];
    for scenario in bundle_scenarios {
        if let Some(paths) = scenario["bundle_artifact_paths"].as_object() {
            checked_paths.extend(
                paths
                    .values()
                    .filter_map(|value| value.as_str())
                    .map(str::to_string),
            );
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
            "bundle_kind": "tracecalc_reference_run",
            "run_id": run_id,
            "status": if missing_paths.is_empty() { "bundle_valid" } else { "bundle_degraded" },
            "degraded_capture": !missing_paths.is_empty(),
            "checked_paths": checked_paths,
            "missing_paths": missing_paths,
        }),
    )
}

fn write_bundle_explain_records(
    artifact_root: &Path,
    run_id: &str,
    engine_diff: &[serde_json::Value],
) -> Result<(), TraceCalcRunnerError> {
    let explain_records = engine_diff
        .iter()
        .flat_map(|entry| {
            let scenario_id = entry["scenario_id"]
                .as_str()
                .unwrap_or_default()
                .to_string();
            entry["mismatches"]
                .as_array()
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .enumerate()
                .map(move |(index, mismatch)| {
                    json!({
                        "schema_version": REPLAY_EXPLAIN_RECORD_SCHEMA_V1,
                        "explain_id": format!("{scenario_id}--why-diff-{}", index + 1),
                        "explain_kind": "why_diff",
                        "scenario_id": scenario_id,
                        "mismatch_kind": mismatch["mismatch_kind"],
                        "severity_class": mismatch["severity_class"],
                        "required_equality_surface": mismatch["required_equality_surface"],
                        "message": mismatch["message"],
                        "source_refs": {
                            "bundle_diff_path": relative_artifact_path([
                                "docs",
                                "test-runs",
                                "core-engine",
                                "tracecalc-reference-machine",
                                run_id,
                                "replay-appliance",
                                "runs",
                                run_id,
                                "diff",
                                "engine_diff.json",
                            ]),
                        },
                    })
                })
        })
        .collect::<Vec<_>>();

    write_json(
        &artifact_root
            .join("replay-appliance/runs")
            .join(run_id)
            .join("diff/explain_records.json"),
        &json!(explain_records),
    )
}

#[allow(clippy::too_many_arguments)]
fn write_scenario_artifacts(
    scenario_directory: &Path,
    run_id: &str,
    scenario: Option<&TraceCalcScenario>,
    scenario_id: &str,
    result_state: TraceCalcScenarioResultState,
    validation_failures: &[TraceCalcValidationFailure],
    assertion_failures: &[String],
    conformance_mismatches: &[TraceCalcConformanceMismatch],
    artifacts: &TraceCalcExecutionArtifacts,
    artifact_paths: &[(String, String)],
) -> Result<(), TraceCalcRunnerError> {
    write_json(
        &scenario_directory.join("result.json"),
        &json!({
            "scenario_id": scenario_id,
            "result_state": to_snake_case(&format!("{result_state:?}")),
            "validation_failures": validation_failures.iter().map(|failure| json!({
                "kind": to_snake_case(&format!("{:?}", failure.kind)),
                "message": failure.message,
            })).collect::<Vec<_>>(),
            "assertion_failures": assertion_failures,
            "conformance_mismatches": conformance_mismatches.iter().map(mismatch_object).collect::<Vec<_>>(),
            "replay_projection": scenario.and_then(|scenario| scenario.replay_projection.as_ref()).map(|projection| json!({
                "replay_classes": projection.replay_classes,
                "pack_bindings": projection.pack_bindings,
                "required_equality_surfaces": projection.required_equality_surfaces,
                "normalized_event_family_map_ref": projection.normalized_event_family_map_ref,
                "safety_properties": projection.safety_properties,
                "transition_labels": projection.transition_labels,
            })),
            "artifact_paths": BTreeMap::from_iter(artifact_paths.iter().cloned()),
        }),
    )?;

    write_json(
        &scenario_directory.join("trace.json"),
        &json!({
            "scenario_id": scenario_id,
            "run_id": run_id,
            "replay_projection": scenario.and_then(|scenario| scenario.replay_projection.as_ref()).map(|projection| json!({
                "replay_classes": projection.replay_classes,
                "required_equality_surfaces": projection.required_equality_surfaces,
                "normalized_event_family_map_ref": projection.normalized_event_family_map_ref,
            })),
            "events": artifacts.trace_events.iter().map(|event| json!({
                "event_id": event.event_id,
                "step_id": event.step_id,
                "label": event.label,
                "normalized_event_family": normalize_event_family(&event.label),
                "payload": BTreeMap::from_iter(event.payload.clone()),
            })).collect::<Vec<_>>(),
        }),
    )?;

    write_json(
        &scenario_directory.join("counters.json"),
        &json!({
            "scenario_id": scenario_id,
            "counters": counter_entries(&artifacts.counters),
        }),
    )?;

    write_json(
        &scenario_directory.join("published_view.json"),
        &json!({
            "scenario_id": scenario_id,
            "snapshot_id": scenario.map(|scenario| scenario.initial_graph.snapshot_id.clone()).unwrap_or_default(),
            "node_values": value_entries(&artifacts.published_values),
        }),
    )?;

    write_json(
        &scenario_directory.join("pinned_views.json"),
        &json!({
            "scenario_id": scenario_id,
            "views": artifacts.pinned_views.iter().map(|view| json!({
                "view_id": view.view_id,
                "snapshot_id": view.snapshot_id,
                "node_values": value_entries(&view.node_values),
            })).collect::<Vec<_>>(),
        }),
    )?;

    write_json(
        &scenario_directory.join("rejects.json"),
        &json!({
            "scenario_id": scenario_id,
            "rejects": artifacts.rejects.iter().map(|reject| json!({
                "reject_id": reject.reject_id,
                "reject_kind": reject.reject_kind,
                "reject_detail": reject.reject_detail,
            })).collect::<Vec<_>>(),
        }),
    )?;
    Ok(())
}

fn write_witness_seed_artifacts(
    artifact_root: &Path,
    inputs: TraceCalcWitnessSeedInputs<'_>,
) -> Result<(), TraceCalcRunnerError> {
    let Some(seed) = build_witness_seed(inputs) else {
        return Ok(());
    };

    let reduction_directory = artifact_root
        .join("replay-appliance")
        .join("reductions")
        .join(&seed.reduction_id);
    create_directory(&reduction_directory)?;
    write_json(
        &reduction_directory.join("reduction_manifest.json"),
        &serde_json::to_value(&seed.reduction_manifest)
            .expect("reduction manifest serialization should succeed"),
    )?;

    let witness_directory = artifact_root
        .join("replay-appliance")
        .join("witnesses")
        .join(&seed.witness_id);
    create_directory(&witness_directory)?;
    write_json(
        &witness_directory.join("lifecycle.json"),
        &serde_json::to_value(&seed.lifecycle)
            .expect("witness lifecycle serialization should succeed"),
    )?;

    Ok(())
}

fn all_artifact_paths(
    relative_artifact_root: &str,
    scenario: Option<&TraceCalcScenario>,
    scenario_id: &str,
) -> Vec<(String, String)> {
    let mut artifact_paths = scenario_artifact_paths(relative_artifact_root, scenario_id);
    if let Some(scenario) = scenario
        && let Some((witness_id, reduction_id)) = witness_artifact_ids(scenario)
    {
        artifact_paths.push((
            "witness_lifecycle".to_string(),
            relative_artifact_path([
                relative_artifact_root,
                "replay-appliance",
                "witnesses",
                &witness_id,
                "lifecycle.json",
            ]),
        ));
        artifact_paths.push((
            "reduction_manifest".to_string(),
            relative_artifact_path([
                relative_artifact_root,
                "replay-appliance",
                "reductions",
                &reduction_id,
                "reduction_manifest.json",
            ]),
        ));
    }
    artifact_paths
}

fn witness_artifact_ids(scenario: &TraceCalcScenario) -> Option<(String, String)> {
    scenario.witness_anchors.as_ref()?;
    Some((
        format!("{}--witness-seed", scenario.scenario_id),
        format!("{}--reduction-seed", scenario.scenario_id),
    ))
}

fn scenario_artifact_paths(
    relative_artifact_root: &str,
    scenario_id: &str,
) -> Vec<(String, String)> {
    let relative_scenario_root =
        relative_artifact_path([relative_artifact_root, "scenarios", scenario_id]);
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

fn oracle_baseline_object(
    scenario_id: &str,
    artifacts: &TraceCalcExecutionArtifacts,
) -> serde_json::Value {
    json!({
        "scenario_id": scenario_id,
        "result_state": to_snake_case(&format!("{:?}", artifacts.result_state)),
        "published_values": value_entries(&artifacts.published_values),
        "pinned_views": artifacts.pinned_views.iter().map(|view| json!({
            "view_id": view.view_id,
            "snapshot_id": view.snapshot_id,
            "node_values": value_entries(&view.node_values),
        })).collect::<Vec<_>>(),
        "counters": counter_entries(&artifacts.counters),
        "rejects": artifacts.rejects.iter().map(|reject| json!({
            "reject_id": reject.reject_id,
            "reject_kind": reject.reject_kind,
            "reject_detail": reject.reject_detail,
        })).collect::<Vec<_>>(),
    })
}

fn engine_diff_object(
    scenario_id: &str,
    oracle_artifacts: &TraceCalcExecutionArtifacts,
    engine_artifacts: &TraceCalcExecutionArtifacts,
    mismatches: &[TraceCalcConformanceMismatch],
) -> serde_json::Value {
    json!({
        "scenario_id": scenario_id,
        "oracle_result_state": to_snake_case(&format!("{:?}", oracle_artifacts.result_state)),
        "engine_result_state": to_snake_case(&format!("{:?}", engine_artifacts.result_state)),
        "mismatches": mismatches.iter().map(mismatch_object).collect::<Vec<_>>(),
    })
}

fn mismatch_object(mismatch: &TraceCalcConformanceMismatch) -> serde_json::Value {
    json!({
        "kind": to_snake_case(&format!("{:?}", mismatch.kind)),
        "mismatch_kind": registry_mismatch_kind(mismatch.kind),
        "severity_class": severity_class(mismatch.kind),
        "required_equality_surface": required_equality_surface(mismatch.kind),
        "message": mismatch.message,
    })
}

fn counter_entries(entries: &[(String, i64)]) -> Vec<serde_json::Value> {
    let mut ordered = entries.to_vec();
    ordered.sort_by(|left, right| left.0.cmp(&right.0));
    ordered
        .into_iter()
        .map(|(counter, value)| json!({ "counter": counter, "value": value }))
        .collect()
}

fn value_entries(entries: &[(String, String)]) -> Vec<serde_json::Value> {
    let mut ordered = entries.to_vec();
    ordered.sort_by(|left, right| left.0.cmp(&right.0));
    ordered
        .into_iter()
        .map(|(node_id, value)| json!({ "node_id": node_id, "value": value }))
        .collect()
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

    use serde_json::Value;

    use super::*;

    #[test]
    fn execute_manifest_produces_passing_conformance_artifacts_for_seed_corpus() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let run_id = format!("test-tracecalc-rust-{}", std::process::id());
        let artifact_root = repo_root.join(format!(
            "docs/test-runs/core-engine/tracecalc-reference-machine/{run_id}"
        ));
        let runner = TraceCalcRunner::new();

        let cleanup = || {
            if artifact_root.exists() {
                let _ = fs::remove_dir_all(&artifact_root);
            }
        };

        cleanup();
        let summary = runner
            .execute_manifest(&repo_root, &run_id, None, None)
            .unwrap();
        assert_eq!(summary.run_id, run_id);
        assert_eq!(summary.scenario_count, 31);
        assert!(artifact_root.join("run_summary.json").exists());
        assert!(artifact_root.join("manifest_selection.json").exists());
        assert!(
            artifact_root
                .join("conformance/oracle_baseline.json")
                .exists()
        );
        assert!(artifact_root.join("conformance/engine_diff.json").exists());
        assert!(
            artifact_root
                .join(
                    "replay-appliance/reductions/tc_publication_fence_reject_001--reduction-seed/reduction_manifest.json",
                )
                .exists()
        );
        assert!(
            artifact_root
                .join(
                    "replay-appliance/witnesses/tc_publication_fence_reject_001--witness-seed/lifecycle.json",
                )
                .exists()
        );
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
                    "replay-appliance/runs/{run_id}/scenarios/tc_verify_clean_no_publish_001/events.jsonl"
                ))
                .exists()
        );
        assert!(
            artifact_root
                .join(format!(
                    "replay-appliance/runs/{run_id}/diff/engine_diff.json"
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
                .join(format!(
                    "replay-appliance/runs/{run_id}/diff/explain_records.json"
                ))
                .exists()
        );

        let diff_document = serde_json::from_str::<Value>(
            &fs::read_to_string(artifact_root.join("conformance/engine_diff.json")).unwrap(),
        )
        .unwrap();
        let diff_entries = diff_document.as_array().unwrap();
        assert!(
            diff_entries
                .iter()
                .any(|entry| entry["scenario_id"] == "tc_verify_clean_no_publish_001")
        );
        assert!(
            diff_entries
                .iter()
                .any(|entry| entry["scenario_id"] == "tc_fallback_reentry_001")
        );
        assert!(
            diff_entries
                .iter()
                .all(|entry| entry["mismatches"].as_array().unwrap().is_empty())
        );

        let verify_trace = serde_json::from_str::<Value>(
            &fs::read_to_string(
                artifact_root.join("scenarios/tc_verify_clean_no_publish_001/trace.json"),
            )
            .unwrap(),
        )
        .unwrap();
        let events = verify_trace["events"].as_array().unwrap();
        assert!(
            events
                .iter()
                .any(|event| event["label"] == "node_verified_clean")
        );
        assert!(
            events
                .iter()
                .any(|event| event["normalized_event_family"] == "candidate.verified_clean")
        );

        let witness_lifecycle = serde_json::from_str::<Value>(
            &fs::read_to_string(
                artifact_root.join(
                    "replay-appliance/witnesses/tc_publication_fence_reject_001--witness-seed/lifecycle.json",
                ),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(witness_lifecycle["lifecycle_state"], "wit.generated_local");
        assert_eq!(witness_lifecycle["pack_eligible"], false);

        let scenario_result = serde_json::from_str::<Value>(
            &fs::read_to_string(
                artifact_root.join("scenarios/tc_publication_fence_reject_001/result.json"),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(
            scenario_result["artifact_paths"]["witness_lifecycle"],
            "docs/test-runs/core-engine/tracecalc-reference-machine/".to_string()
                + &run_id
                + "/replay-appliance/witnesses/tc_publication_fence_reject_001--witness-seed/lifecycle.json"
        );

        let reduction_manifest = serde_json::from_str::<Value>(
            &fs::read_to_string(
                artifact_root.join(
                    "replay-appliance/reductions/tc_publication_fence_reject_001--reduction-seed/reduction_manifest.json",
                ),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(
            reduction_manifest["status_id"],
            "oxcalc.reduction.seeded_local"
        );
        assert!(
            reduction_manifest["units"]
                .as_array()
                .unwrap()
                .iter()
                .any(|unit| unit["unit_kind"] == "reject_record" && unit["reject_id"] == "rej1")
        );

        let bundle_manifest = serde_json::from_str::<Value>(
            &fs::read_to_string(artifact_root.join("replay-appliance/bundle_manifest.json"))
                .unwrap(),
        )
        .unwrap();
        assert_eq!(
            bundle_manifest["projection_status"],
            "projection_validated_with_explain"
        );

        let bundle_events = fs::read_to_string(artifact_root.join(format!(
            "replay-appliance/runs/{run_id}/scenarios/tc_verify_clean_no_publish_001/events.jsonl"
        )))
        .unwrap();
        assert!(bundle_events.contains("\"normalized_event_family\":\"candidate.verified_clean\""));

        let bundle_validation = serde_json::from_str::<Value>(
            &fs::read_to_string(
                artifact_root.join("replay-appliance/validation/bundle_validation.json"),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(bundle_validation["status"], "bundle_valid");

        let explain_records = serde_json::from_str::<Value>(
            &fs::read_to_string(artifact_root.join(format!(
                "replay-appliance/runs/{run_id}/diff/explain_records.json"
            )))
            .unwrap(),
        )
        .unwrap();
        assert!(explain_records.as_array().unwrap().is_empty());

        cleanup();
    }
}
