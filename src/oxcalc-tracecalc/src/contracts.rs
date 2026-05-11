#![forbid(unsafe_code)]

//! `TraceCalc` contract boundary.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraceCalcScenarioResultState {
    Passed,
    FailedAssertion,
    InvalidScenario,
    ExecutionError,
    UnsupportedFeature,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraceCalcValidationFailureKind {
    JsonParseFailure,
    UnsupportedSchemaVersion,
    MissingRequiredField,
    UnknownStepKind,
    UnknownNodeReference,
    UnknownCandidateReference,
    ManifestMismatch,
    InvalidExpectedShape,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraceCalcConformanceMismatchKind {
    MissingScenarioResult,
    ResultStateMismatch,
    PublishedViewMismatch,
    PinnedViewMismatch,
    RejectMismatch,
    TraceCountMismatch,
    CounterMismatch,
    UnexpectedExtraArtifact,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TraceCalcManifest {
    pub schema_version: String,
    pub corpus_id: String,
    pub base_path: String,
    pub scenarios: Vec<TraceCalcManifestScenario>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TraceCalcManifestScenario {
    pub scenario_id: String,
    pub path: String,
    #[serde(default)]
    pub focus: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TraceCalcScenario {
    pub schema_version: String,
    pub scenario_id: String,
    pub description: String,
    pub calc_space: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub pack_tags: Vec<String>,
    pub initial_graph: TraceCalcInitialGraph,
    #[serde(default)]
    pub initial_runtime: TraceCalcInitialRuntime,
    pub steps: Vec<TraceCalcStep>,
    pub expected: TraceCalcExpected,
    pub replay_projection: Option<TraceCalcReplayProjection>,
    pub witness_anchors: Option<TraceCalcWitnessAnchors>,
    pub generator: Option<Value>,
    #[serde(default)]
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TraceCalcReplayProjection {
    #[serde(default)]
    pub replay_classes: Vec<String>,
    #[serde(default)]
    pub pack_bindings: Vec<String>,
    #[serde(default)]
    pub required_equality_surfaces: Vec<String>,
    pub normalized_event_family_map_ref: String,
    #[serde(default)]
    pub safety_properties: Vec<String>,
    #[serde(default)]
    pub transition_labels: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TraceCalcWitnessAnchors {
    pub scenario_anchor_id: String,
    #[serde(default)]
    pub phase_blocks: Vec<TraceCalcPhaseBlockAnchor>,
    #[serde(default)]
    pub event_groups: Vec<TraceCalcEventGroupAnchor>,
    #[serde(default)]
    pub reject_records: Vec<TraceCalcRejectRecordAnchor>,
    #[serde(default)]
    pub view_slices: Vec<TraceCalcViewSliceAnchor>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TraceCalcPhaseBlockAnchor {
    pub phase_block_id: String,
    #[serde(default)]
    pub step_ids: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TraceCalcEventGroupAnchor {
    pub event_group_id: String,
    #[serde(default)]
    pub step_ids: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TraceCalcRejectRecordAnchor {
    pub reject_record_id: String,
    pub reject_id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TraceCalcViewSliceAnchor {
    pub view_slice_id: String,
    pub view_kind: String,
    pub view_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TraceCalcInitialGraph {
    pub snapshot_id: String,
    pub nodes: Vec<TraceCalcNode>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TraceCalcNode {
    pub node_id: String,
    pub kind: String,
    #[serde(rename = "expr")]
    pub expression: Value,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct TraceCalcInitialRuntime {
    #[serde(default)]
    pub pinned_views: Vec<TraceCalcPinnedViewExpectation>,
    #[serde(default)]
    pub published_values: Vec<TraceCalcValueEntry>,
    #[serde(default)]
    pub published_runtime_effects: Vec<TraceCalcRuntimeEffect>,
    #[serde(default)]
    pub seed_overlays: Vec<TraceCalcSeedOverlay>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TraceCalcSeedOverlay {
    pub overlay_kind: String,
    pub owner_node_id: String,
    pub payload: Option<Value>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct TraceCalcStep {
    pub step_id: String,
    pub kind: String,
    #[serde(default)]
    pub targets: Vec<String>,
    pub admission_id: Option<String>,
    pub compatibility_basis: Option<String>,
    pub candidate_result_id: Option<String>,
    pub publication_id: Option<String>,
    pub view_id: Option<String>,
    pub snapshot_id: Option<String>,
    #[serde(default)]
    pub observed_nodes: Vec<String>,
    #[serde(default)]
    pub value_updates: Vec<TraceCalcValueEntry>,
    #[serde(default)]
    pub dependency_shape_updates: Vec<TraceCalcDependencyShapeUpdate>,
    #[serde(default)]
    pub runtime_effects: Vec<TraceCalcRuntimeEffect>,
    #[serde(default)]
    pub diagnostic_events: Vec<String>,
    pub reject_id: Option<String>,
    pub reject_kind: Option<String>,
    #[serde(rename = "reject_detail")]
    pub reject_detail: Option<Value>,
    pub reject_detail_text: Option<String>,
    pub overlay_kind: Option<String>,
    pub owner_node_id: Option<String>,
    pub payload: Option<Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TraceCalcValueEntry {
    pub node_id: String,
    #[serde(deserialize_with = "deserialize_stringified_value")]
    pub value: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TraceCalcDependencyShapeUpdate {
    pub node_id: String,
    pub kind: String,
    #[serde(rename = "dep_id")]
    pub dependency_id: Option<String>,
    pub payload: Option<Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TraceCalcRuntimeEffect {
    pub effect_kind: String,
    pub owner_node_id: String,
    pub payload: Option<Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TraceCalcExpected {
    pub published_view: TraceCalcPublishedViewExpectation,
    #[serde(default)]
    pub pinned_views: Vec<TraceCalcPinnedViewExpectation>,
    #[serde(default)]
    pub trace_labels: Vec<TraceCalcTraceLabelExpectation>,
    #[serde(default)]
    pub counter_expectations: Vec<TraceCalcCounterExpectation>,
    #[serde(default)]
    pub rejects: Vec<TraceCalcRejectExpectation>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TraceCalcPublishedViewExpectation {
    pub snapshot_id: String,
    #[serde(default)]
    pub node_values: Vec<TraceCalcValueEntry>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TraceCalcPinnedViewExpectation {
    pub view_id: String,
    pub snapshot_id: String,
    #[serde(default)]
    pub observed_nodes: Vec<String>,
    #[serde(default)]
    pub node_values: Vec<TraceCalcValueEntry>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TraceCalcTraceLabelExpectation {
    pub label: String,
    pub count: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TraceCalcCounterExpectation {
    pub counter: String,
    pub comparison: String,
    pub value: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TraceCalcRejectExpectation {
    pub reject_id: String,
    pub reject_kind: String,
    pub detail_contains: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceCalcValidationFailure {
    pub kind: TraceCalcValidationFailureKind,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceCalcTraceEvent {
    pub event_id: String,
    pub step_id: String,
    pub label: String,
    pub payload: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceCalcRejectRecord {
    pub reject_id: String,
    pub reject_kind: String,
    pub reject_detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceCalcPinnedViewRecord {
    pub view_id: String,
    pub snapshot_id: String,
    pub node_values: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceCalcExecutionArtifacts {
    pub scenario_id: String,
    pub result_state: TraceCalcScenarioResultState,
    pub assertion_failures: Vec<String>,
    pub trace_events: Vec<TraceCalcTraceEvent>,
    pub counters: Vec<(String, i64)>,
    pub published_values: Vec<(String, String)>,
    pub pinned_views: Vec<TraceCalcPinnedViewRecord>,
    pub rejects: Vec<TraceCalcRejectRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceCalcConformanceMismatch {
    pub kind: TraceCalcConformanceMismatchKind,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceCalcScenarioResult {
    pub scenario_id: String,
    pub result_state: TraceCalcScenarioResultState,
    pub validation_failures: Vec<TraceCalcValidationFailure>,
    pub assertion_failures: Vec<String>,
    pub conformance_mismatches: Vec<TraceCalcConformanceMismatch>,
    pub artifact_paths: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceCalcRunSummary {
    pub run_id: String,
    pub schema_version: String,
    pub scenario_count: usize,
    pub result_counts: Vec<(String, usize)>,
    pub artifact_root: String,
}

#[derive(Debug, Error)]
pub enum TraceCalcLoadError {
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
}

pub fn load_manifest(path: &Path) -> Result<TraceCalcManifest, TraceCalcLoadError> {
    let text = fs::read_to_string(path).map_err(|source| TraceCalcLoadError::Read {
        path: path.display().to_string(),
        source,
    })?;
    serde_json::from_str(&text).map_err(|source| TraceCalcLoadError::Parse {
        path: path.display().to_string(),
        source,
    })
}

pub fn load_scenario(path: &Path) -> Result<TraceCalcScenario, TraceCalcLoadError> {
    let text = fs::read_to_string(path).map_err(|source| TraceCalcLoadError::Read {
        path: path.display().to_string(),
        source,
    })?;
    serde_json::from_str(&text).map_err(|source| TraceCalcLoadError::Parse {
        path: path.display().to_string(),
        source,
    })
}

pub fn validate_scenario(
    manifest_scenario: &TraceCalcManifestScenario,
    scenario: &TraceCalcScenario,
) -> Vec<TraceCalcValidationFailure> {
    let mut failures = Vec::new();
    if scenario.schema_version != "tracecalc-s1" {
        failures.push(TraceCalcValidationFailure {
            kind: TraceCalcValidationFailureKind::UnsupportedSchemaVersion,
            message: format!(
                "Unsupported scenario schema version '{}'.",
                scenario.schema_version
            ),
        });
    }

    if scenario.calc_space != "TraceCalc" {
        failures.push(TraceCalcValidationFailure {
            kind: TraceCalcValidationFailureKind::InvalidExpectedShape,
            message: format!("Unsupported calc_space '{}'.", scenario.calc_space),
        });
    }

    if manifest_scenario.scenario_id != scenario.scenario_id {
        failures.push(TraceCalcValidationFailure {
            kind: TraceCalcValidationFailureKind::ManifestMismatch,
            message: format!(
                "Manifest scenario id '{}' does not match '{}'.",
                manifest_scenario.scenario_id, scenario.scenario_id
            ),
        });
    }

    if !scenario.pack_tags.is_empty() && scenario.replay_projection.is_none() {
        failures.push(TraceCalcValidationFailure {
            kind: TraceCalcValidationFailureKind::InvalidExpectedShape,
            message: format!(
                "Replay-facing scenario '{}' is missing replay_projection metadata.",
                scenario.scenario_id
            ),
        });
    }

    if scenario
        .replay_projection
        .as_ref()
        .is_some_and(|projection| projection.replay_classes.is_empty())
    {
        failures.push(TraceCalcValidationFailure {
            kind: TraceCalcValidationFailureKind::InvalidExpectedShape,
            message: format!(
                "Replay projection for '{}' must name at least one replay class.",
                scenario.scenario_id
            ),
        });
    }

    let node_ids = scenario
        .initial_graph
        .nodes
        .iter()
        .map(|node| node.node_id.clone())
        .collect::<BTreeSet<_>>();
    let mut candidate_ids = BTreeSet::new();
    let mut view_ids = BTreeSet::new();

    const SUPPORTED_STEP_KINDS: &[&str] = &[
        "pin_view",
        "unpin_view",
        "mark_stale",
        "seed_overlay",
        "reset_fixture",
        "admit_work",
        "emit_candidate_result",
        "emit_iteration_trace",
        "emit_reject",
        "publish_candidate",
        "verify_clean",
    ];

    for step in &scenario.steps {
        if !SUPPORTED_STEP_KINDS.contains(&step.kind.as_str()) {
            failures.push(TraceCalcValidationFailure {
                kind: TraceCalcValidationFailureKind::UnknownStepKind,
                message: format!(
                    "Unknown step kind '{}' in step '{}'.",
                    step.kind, step.step_id
                ),
            });
        }

        for node_id in step.targets.iter().chain(step.observed_nodes.iter()) {
            if !node_ids.contains(node_id) {
                failures.push(TraceCalcValidationFailure {
                    kind: TraceCalcValidationFailureKind::UnknownNodeReference,
                    message: format!(
                        "Unknown node reference '{}' in step '{}'.",
                        node_id, step.step_id
                    ),
                });
            }
        }

        for update in &step.value_updates {
            if !node_ids.contains(&update.node_id) {
                failures.push(TraceCalcValidationFailure {
                    kind: TraceCalcValidationFailureKind::UnknownNodeReference,
                    message: format!(
                        "Unknown value-update node '{}' in step '{}'.",
                        update.node_id, step.step_id
                    ),
                });
            }
        }

        for update in &step.dependency_shape_updates {
            let dependency_ok = update
                .dependency_id
                .as_ref()
                .is_none_or(|dependency_id| node_ids.contains(dependency_id));
            if !node_ids.contains(&update.node_id) || !dependency_ok {
                failures.push(TraceCalcValidationFailure {
                    kind: TraceCalcValidationFailureKind::UnknownNodeReference,
                    message: format!(
                        "Unknown dependency-shape reference in step '{}'.",
                        step.step_id
                    ),
                });
            }
        }

        for effect in &step.runtime_effects {
            if !node_ids.contains(&effect.owner_node_id) {
                failures.push(TraceCalcValidationFailure {
                    kind: TraceCalcValidationFailureKind::UnknownNodeReference,
                    message: format!(
                        "Unknown runtime-effect owner '{}' in step '{}'.",
                        effect.owner_node_id, step.step_id
                    ),
                });
            }
        }

        if step.kind == "emit_candidate_result"
            && let Some(candidate_result_id) = &step.candidate_result_id
        {
            candidate_ids.insert(candidate_result_id.clone());
        }

        if step.kind == "publish_candidate" {
            if let Some(candidate_result_id) = &step.candidate_result_id {
                if !candidate_ids.contains(candidate_result_id) {
                    failures.push(TraceCalcValidationFailure {
                        kind: TraceCalcValidationFailureKind::UnknownCandidateReference,
                        message: format!(
                            "Unknown candidate '{}' in publish step '{}'.",
                            candidate_result_id, step.step_id
                        ),
                    });
                }
            } else {
                failures.push(TraceCalcValidationFailure {
                    kind: TraceCalcValidationFailureKind::UnknownCandidateReference,
                    message: format!(
                        "Publish step '{}' is missing candidate_result_id.",
                        step.step_id
                    ),
                });
            }
        }

        if step.kind == "pin_view"
            && let Some(view_id) = &step.view_id
        {
            view_ids.insert(view_id.clone());
        }

        if step.kind == "unpin_view" {
            match &step.view_id {
                Some(view_id) if view_ids.contains(view_id) => {}
                Some(view_id) => failures.push(TraceCalcValidationFailure {
                    kind: TraceCalcValidationFailureKind::InvalidExpectedShape,
                    message: format!(
                        "Unpin step '{}' references unknown view '{}'.",
                        step.step_id, view_id
                    ),
                }),
                None => failures.push(TraceCalcValidationFailure {
                    kind: TraceCalcValidationFailureKind::InvalidExpectedShape,
                    message: format!("Unpin step '{}' is missing view_id.", step.step_id),
                }),
            }
        }
    }

    for expectation in &scenario.expected.pinned_views {
        for value in &expectation.node_values {
            if !node_ids.contains(&value.node_id) {
                failures.push(TraceCalcValidationFailure {
                    kind: TraceCalcValidationFailureKind::UnknownNodeReference,
                    message: format!(
                        "Unknown pinned-view node '{}' in expected section.",
                        value.node_id
                    ),
                });
            }
        }
    }

    failures
}

pub fn normalize_json_value(value: &Value) -> String {
    match value {
        Value::String(string) => string.clone(),
        _ => value.to_string().trim_matches('"').to_string(),
    }
}

fn deserialize_stringified_value<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?;
    Ok(normalize_json_value(&value))
}
