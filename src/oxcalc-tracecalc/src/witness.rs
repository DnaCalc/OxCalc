#![forbid(unsafe_code)]

//! Witness-seed and reduction-unit support for `TraceCalc`.

use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use crate::assertions::to_snake_case;
use crate::contracts::{
    TraceCalcConformanceMismatch, TraceCalcScenario, TraceCalcScenarioResultState,
    TraceCalcValidationFailure,
};

const WITNESS_LIFECYCLE_SCHEMA_V1: &str = "oxcalc.tracecalc.witness_lifecycle.v1";
const REDUCTION_MANIFEST_SCHEMA_V1: &str = "oxcalc.tracecalc.reduction_manifest.v1";
const LOCAL_REDUCTION_STATUS_ID: &str = "oxcalc.reduction.seeded_local";
const LOCAL_REDUCTION_EXPLANATORY_STATUS_ID: &str = "oxcalc.reduction.explanatory_only";
const LOCAL_REDUCTION_QUARANTINED_STATUS_ID: &str = "oxcalc.reduction.quarantined_local";
const LOCAL_REDUCTION_STATUS_SCOPE: &str = "local_only_until_foundation_binding";

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct TraceCalcWitnessSeed {
    pub witness_id: String,
    pub reduction_id: String,
    pub lifecycle: TraceCalcWitnessLifecycleRecord,
    pub reduction_manifest: TraceCalcReductionManifest,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct TraceCalcWitnessLifecycleRecord {
    pub schema_version: String,
    pub witness_id: String,
    pub scenario_id: String,
    pub run_id: String,
    pub lifecycle_state: String,
    pub pack_eligible: bool,
    pub replay_validity_assessed: bool,
    pub quarantine_reason: Option<String>,
    pub reduction_manifest_path: String,
    pub source_artifact_paths: BTreeMap<String, String>,
    pub anchor_counts: TraceCalcWitnessAnchorCounts,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct TraceCalcWitnessAnchorCounts {
    pub phase_block_count: usize,
    pub event_group_count: usize,
    pub reject_record_count: usize,
    pub view_slice_count: usize,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct TraceCalcReductionManifest {
    pub schema_version: String,
    pub reduction_id: String,
    pub witness_id: String,
    pub scenario_id: String,
    pub run_id: String,
    pub status_id: String,
    pub status_scope: String,
    pub source_semantics_authority: String,
    pub required_equality_surfaces: Vec<String>,
    pub mismatch_kinds: Vec<String>,
    pub units: Vec<TraceCalcReductionUnit>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct TraceCalcReductionUnit {
    pub unit_id: String,
    pub unit_kind: String,
    pub anchor_id: String,
    pub closure_unit_ids: Vec<String>,
    pub step_ids: Vec<String>,
    pub reject_id: Option<String>,
    pub view_kind: Option<String>,
    pub view_id: Option<String>,
}

pub struct TraceCalcWitnessSeedInputs<'a> {
    pub run_id: &'a str,
    pub relative_artifact_root: &'a str,
    pub scenario: &'a TraceCalcScenario,
    pub result_state: TraceCalcScenarioResultState,
    pub validation_failures: &'a [TraceCalcValidationFailure],
    pub assertion_failures: &'a [String],
    pub scenario_artifact_paths: &'a [(String, String)],
    pub conformance_mismatches: &'a [TraceCalcConformanceMismatch],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TraceCalcWitnessSeedState {
    GeneratedLocal,
    ExplanatoryOnly,
    Quarantined,
}

impl TraceCalcWitnessSeedState {
    fn lifecycle_state_id(self) -> &'static str {
        match self {
            Self::GeneratedLocal => "wit.generated_local",
            Self::ExplanatoryOnly => "wit.explanatory_only",
            Self::Quarantined => "wit.quarantined",
        }
    }

    fn reduction_status_id(self) -> &'static str {
        match self {
            Self::GeneratedLocal => LOCAL_REDUCTION_STATUS_ID,
            Self::ExplanatoryOnly => LOCAL_REDUCTION_EXPLANATORY_STATUS_ID,
            Self::Quarantined => LOCAL_REDUCTION_QUARANTINED_STATUS_ID,
        }
    }

    fn replay_validity_assessed(self) -> bool {
        matches!(self, Self::ExplanatoryOnly)
    }

    fn quarantine_reason(self) -> Option<&'static str> {
        match self {
            Self::Quarantined => Some("capture_insufficient"),
            _ => None,
        }
    }
}

pub fn build_witness_seed(inputs: TraceCalcWitnessSeedInputs<'_>) -> Option<TraceCalcWitnessSeed> {
    let TraceCalcWitnessSeedInputs {
        run_id,
        relative_artifact_root,
        scenario,
        result_state,
        validation_failures,
        assertion_failures,
        scenario_artifact_paths,
        conformance_mismatches,
    } = inputs;

    let anchors = scenario.witness_anchors.as_ref()?;
    let witness_id = format!("{}--witness-seed", scenario.scenario_id);
    let reduction_id = format!("{}--reduction-seed", scenario.scenario_id);
    let reduction_manifest_path = relative_artifact_path([
        relative_artifact_root,
        "replay-appliance",
        "reductions",
        &reduction_id,
        "reduction_manifest.json",
    ]);

    let phase_units = anchors
        .phase_blocks
        .iter()
        .map(|anchor| TraceCalcReductionUnit {
            unit_id: format!("phase:{}", anchor.phase_block_id),
            unit_kind: "phase_block".to_string(),
            anchor_id: anchor.phase_block_id.clone(),
            closure_unit_ids: Vec::new(),
            step_ids: anchor.step_ids.clone(),
            reject_id: None,
            view_kind: None,
            view_id: None,
        })
        .collect::<Vec<_>>();

    let reject_units = anchors
        .reject_records
        .iter()
        .map(|anchor| {
            let step_ids = scenario
                .steps
                .iter()
                .filter(|step| step.reject_id.as_deref() == Some(anchor.reject_id.as_str()))
                .map(|step| step.step_id.clone())
                .collect::<Vec<_>>();
            TraceCalcReductionUnit {
                unit_id: format!("reject:{}", anchor.reject_record_id),
                unit_kind: "reject_record".to_string(),
                anchor_id: anchor.reject_record_id.clone(),
                closure_unit_ids: Vec::new(),
                step_ids,
                reject_id: Some(anchor.reject_id.clone()),
                view_kind: None,
                view_id: None,
            }
        })
        .collect::<Vec<_>>();

    let event_units = anchors
        .event_groups
        .iter()
        .map(|anchor| {
            let closure_unit_ids = reject_units
                .iter()
                .filter(|unit| shares_any_step(&anchor.step_ids, &unit.step_ids))
                .map(|unit| unit.unit_id.clone())
                .collect::<Vec<_>>();
            TraceCalcReductionUnit {
                unit_id: format!("events:{}", anchor.event_group_id),
                unit_kind: "event_group".to_string(),
                anchor_id: anchor.event_group_id.clone(),
                closure_unit_ids,
                step_ids: anchor.step_ids.clone(),
                reject_id: None,
                view_kind: None,
                view_id: None,
            }
        })
        .collect::<Vec<_>>();

    let view_units = anchors
        .view_slices
        .iter()
        .map(|anchor| {
            let step_ids = scenario
                .steps
                .iter()
                .filter(|step| match anchor.view_kind.as_str() {
                    "pinned_view" => step.view_id == anchor.view_id,
                    "published_view" => step.kind == "publish_candidate",
                    _ => false,
                })
                .map(|step| step.step_id.clone())
                .collect::<Vec<_>>();
            let closure_unit_ids = event_units
                .iter()
                .filter(|unit| shares_any_step(&unit.step_ids, &step_ids))
                .map(|unit| unit.unit_id.clone())
                .collect::<Vec<_>>();
            TraceCalcReductionUnit {
                unit_id: format!("view:{}", anchor.view_slice_id),
                unit_kind: "view_slice".to_string(),
                anchor_id: anchor.view_slice_id.clone(),
                closure_unit_ids,
                step_ids,
                reject_id: None,
                view_kind: Some(anchor.view_kind.clone()),
                view_id: anchor.view_id.clone(),
            }
        })
        .collect::<Vec<_>>();

    let mut units = Vec::new();
    units.extend(phase_units);
    units.extend(event_units);
    units.extend(reject_units);
    units.extend(view_units);

    let scenario_closure = units
        .iter()
        .map(|unit| unit.unit_id.clone())
        .collect::<Vec<_>>();
    units.insert(
        0,
        TraceCalcReductionUnit {
            unit_id: "scenario".to_string(),
            unit_kind: "scenario".to_string(),
            anchor_id: anchors.scenario_anchor_id.clone(),
            closure_unit_ids: scenario_closure,
            step_ids: scenario
                .steps
                .iter()
                .map(|step| step.step_id.clone())
                .collect::<Vec<_>>(),
            reject_id: None,
            view_kind: None,
            view_id: None,
        },
    );

    let event_group_units = units
        .iter()
        .filter(|unit| unit.unit_kind == "event_group")
        .map(|unit| (unit.unit_id.clone(), unit.step_ids.clone()))
        .collect::<Vec<_>>();
    for phase_unit in units
        .iter_mut()
        .filter(|unit| unit.unit_kind == "phase_block")
    {
        phase_unit.closure_unit_ids = event_group_units
            .iter()
            .filter(|(_, step_ids)| shares_any_step(&phase_unit.step_ids, step_ids))
            .map(|(unit_id, _)| unit_id.clone())
            .collect::<Vec<_>>();
    }

    let mismatch_kinds = conformance_mismatches
        .iter()
        .map(|mismatch| to_snake_case(&format!("{:?}", mismatch.kind)))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let seed_state = classify_seed_state(
        result_state,
        validation_failures,
        assertion_failures,
        conformance_mismatches,
    );

    let lifecycle = TraceCalcWitnessLifecycleRecord {
        schema_version: WITNESS_LIFECYCLE_SCHEMA_V1.to_string(),
        witness_id: witness_id.clone(),
        scenario_id: scenario.scenario_id.clone(),
        run_id: run_id.to_string(),
        lifecycle_state: seed_state.lifecycle_state_id().to_string(),
        pack_eligible: false,
        replay_validity_assessed: seed_state.replay_validity_assessed(),
        quarantine_reason: seed_state.quarantine_reason().map(str::to_string),
        reduction_manifest_path: reduction_manifest_path.clone(),
        source_artifact_paths: BTreeMap::from_iter(scenario_artifact_paths.iter().cloned().chain(
            [(
                "engine_diff".to_string(),
                relative_artifact_path([relative_artifact_root, "conformance", "engine_diff.json"]),
            )],
        )),
        anchor_counts: TraceCalcWitnessAnchorCounts {
            phase_block_count: anchors.phase_blocks.len(),
            event_group_count: anchors.event_groups.len(),
            reject_record_count: anchors.reject_records.len(),
            view_slice_count: anchors.view_slices.len(),
        },
    };

    let reduction_manifest = TraceCalcReductionManifest {
        schema_version: REDUCTION_MANIFEST_SCHEMA_V1.to_string(),
        reduction_id: reduction_id.clone(),
        witness_id: witness_id.clone(),
        scenario_id: scenario.scenario_id.clone(),
        run_id: run_id.to_string(),
        status_id: seed_state.reduction_status_id().to_string(),
        status_scope: LOCAL_REDUCTION_STATUS_SCOPE.to_string(),
        source_semantics_authority: "oxcalc.tracecalc.reference_machine".to_string(),
        required_equality_surfaces: scenario
            .replay_projection
            .as_ref()
            .map(|projection| projection.required_equality_surfaces.clone())
            .unwrap_or_default(),
        mismatch_kinds,
        units,
    };

    Some(TraceCalcWitnessSeed {
        witness_id,
        reduction_id,
        lifecycle,
        reduction_manifest,
    })
}

fn classify_seed_state(
    result_state: TraceCalcScenarioResultState,
    validation_failures: &[TraceCalcValidationFailure],
    assertion_failures: &[String],
    conformance_mismatches: &[TraceCalcConformanceMismatch],
) -> TraceCalcWitnessSeedState {
    if !validation_failures.is_empty()
        || matches!(
            result_state,
            TraceCalcScenarioResultState::InvalidScenario
                | TraceCalcScenarioResultState::ExecutionError
                | TraceCalcScenarioResultState::UnsupportedFeature
        )
    {
        TraceCalcWitnessSeedState::Quarantined
    } else if !assertion_failures.is_empty()
        || !conformance_mismatches.is_empty()
        || result_state == TraceCalcScenarioResultState::FailedAssertion
    {
        TraceCalcWitnessSeedState::ExplanatoryOnly
    } else {
        TraceCalcWitnessSeedState::GeneratedLocal
    }
}

fn shares_any_step(left: &[String], right: &[String]) -> bool {
    let left_steps = left.iter().collect::<BTreeSet<_>>();
    right.iter().any(|step_id| left_steps.contains(step_id))
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

    use crate::contracts::{
        TraceCalcConformanceMismatch, TraceCalcConformanceMismatchKind,
        TraceCalcScenarioResultState, TraceCalcValidationFailure, TraceCalcValidationFailureKind,
        load_scenario,
    };

    use super::*;

    #[test]
    fn witness_seed_uses_declared_anchors_and_local_seed_status() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let scenario = load_scenario(
            &repo_root.join(
                "docs/test-corpus/core-engine/tracecalc/hand-auditable/tc_publication_fence_reject_001.json",
            ),
        )
        .unwrap();

        let artifact_paths = vec![
            (
                "result".to_string(),
                "docs/test-runs/core-engine/tracecalc-reference-machine/test/scenarios/tc_publication_fence_reject_001/result.json"
                    .to_string(),
            ),
            (
                "trace".to_string(),
                "docs/test-runs/core-engine/tracecalc-reference-machine/test/scenarios/tc_publication_fence_reject_001/trace.json"
                    .to_string(),
            ),
        ];

        let seed = build_witness_seed(TraceCalcWitnessSeedInputs {
            run_id: "test-run",
            relative_artifact_root: "docs/test-runs/core-engine/tracecalc-reference-machine/test-run",
            scenario: &scenario,
            result_state: TraceCalcScenarioResultState::Passed,
            validation_failures: &[],
            assertion_failures: &[],
            scenario_artifact_paths: &artifact_paths,
            conformance_mismatches: &[],
        })
        .unwrap();

        assert_eq!(
            seed.witness_id,
            "tc_publication_fence_reject_001--witness-seed"
        );
        assert_eq!(
            seed.reduction_manifest.status_id,
            "oxcalc.reduction.seeded_local"
        );
        assert_eq!(seed.lifecycle.lifecycle_state, "wit.generated_local");
        assert!(!seed.lifecycle.pack_eligible);
        assert_eq!(seed.lifecycle.anchor_counts.reject_record_count, 1);
        assert_eq!(seed.reduction_manifest.units.len(), 5);
        assert!(
            seed.reduction_manifest
                .units
                .iter()
                .any(|unit| unit.unit_kind == "reject_record"
                    && unit.reject_id.as_deref() == Some("rej1")
                    && unit.step_ids == vec!["st3".to_string()])
        );
        assert!(
            seed.reduction_manifest
                .units
                .iter()
                .any(|unit| unit.unit_kind == "scenario" && unit.closure_unit_ids.len() == 4)
        );
    }

    #[test]
    fn witness_seed_marks_failed_assertion_as_explanatory_only() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let scenario = load_scenario(
            &repo_root.join(
                "docs/test-corpus/core-engine/tracecalc/hand-auditable/tc_publication_fence_reject_001.json",
            ),
        )
        .unwrap();

        let assertion_failures = vec!["engine diverged".to_string()];
        let conformance_mismatches = vec![TraceCalcConformanceMismatch {
            kind: TraceCalcConformanceMismatchKind::RejectMismatch,
            message: "reject mismatch".to_string(),
        }];
        let seed = build_witness_seed(TraceCalcWitnessSeedInputs {
            run_id: "test-run",
            relative_artifact_root: "docs/test-runs/core-engine/tracecalc-reference-machine/test-run",
            scenario: &scenario,
            result_state: TraceCalcScenarioResultState::FailedAssertion,
            validation_failures: &[],
            assertion_failures: &assertion_failures,
            scenario_artifact_paths: &[],
            conformance_mismatches: &conformance_mismatches,
        })
        .unwrap();

        assert_eq!(seed.lifecycle.lifecycle_state, "wit.explanatory_only");
        assert!(seed.lifecycle.replay_validity_assessed);
        assert_eq!(
            seed.reduction_manifest.status_id,
            "oxcalc.reduction.explanatory_only"
        );
    }

    #[test]
    fn witness_seed_quarantines_when_capture_is_insufficient() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let scenario = load_scenario(
            &repo_root.join(
                "docs/test-corpus/core-engine/tracecalc/hand-auditable/tc_publication_fence_reject_001.json",
            ),
        )
        .unwrap();

        let validation_failures = [TraceCalcValidationFailure {
            kind: TraceCalcValidationFailureKind::InvalidExpectedShape,
            message: "invalid shape".to_string(),
        }];
        let seed = build_witness_seed(TraceCalcWitnessSeedInputs {
            run_id: "test-run",
            relative_artifact_root: "docs/test-runs/core-engine/tracecalc-reference-machine/test-run",
            scenario: &scenario,
            result_state: TraceCalcScenarioResultState::InvalidScenario,
            validation_failures: &validation_failures,
            assertion_failures: &[],
            scenario_artifact_paths: &[],
            conformance_mismatches: &[],
        })
        .unwrap();

        assert_eq!(seed.lifecycle.lifecycle_state, "wit.quarantined");
        assert_eq!(
            seed.lifecycle.quarantine_reason.as_deref(),
            Some("capture_insufficient")
        );
        assert_eq!(
            seed.reduction_manifest.status_id,
            "oxcalc.reduction.quarantined_local"
        );
    }
}
