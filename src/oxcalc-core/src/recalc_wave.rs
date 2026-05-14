#![forbid(unsafe_code)]

//! W050 recalc-wave lifecycle over the OxFml runtime session driver.

use oxfml_core::consumer::runtime::{
    RuntimeEnvironment, RuntimeFormulaRequest, RuntimeFormulaResult, RuntimeManagedOpenResult,
};
use thiserror::Error;

use crate::coordinator::{PublicationBundle, RejectDetail};
use crate::correctness_floor::{
    CorrectnessFloorProfile, CorrectnessFloorReplayRecord, CorrectnessFloorReplayValidationError,
};
use crate::oxfml_session::{OxfmlRecalcSessionDriver, OxfmlRecalcSessionError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RecalcWavePhase {
    WavePreparation,
    EnsurePrepared,
    Compilation,
    DependencyDerivation,
    ScheduleInvoke,
    CoordinatorCommit,
    CloseCapture,
}

impl RecalcWavePhase {
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::WavePreparation => "wave_preparation",
            Self::EnsurePrepared => "ensure_prepared",
            Self::Compilation => "compilation",
            Self::DependencyDerivation => "dependency_derivation",
            Self::ScheduleInvoke => "schedule_invoke",
            Self::CoordinatorCommit => "coordinator_commit",
            Self::CloseCapture => "close_capture",
        }
    }

    fn ordinal(self) -> usize {
        match self {
            Self::WavePreparation => 0,
            Self::EnsurePrepared => 1,
            Self::Compilation => 2,
            Self::DependencyDerivation => 3,
            Self::ScheduleInvoke => 4,
            Self::CoordinatorCommit => 5,
            Self::CloseCapture => 6,
        }
    }
}

const ORDERED_PHASES: [RecalcWavePhase; 7] = [
    RecalcWavePhase::WavePreparation,
    RecalcWavePhase::EnsurePrepared,
    RecalcWavePhase::Compilation,
    RecalcWavePhase::DependencyDerivation,
    RecalcWavePhase::ScheduleInvoke,
    RecalcWavePhase::CoordinatorCommit,
    RecalcWavePhase::CloseCapture,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RecalcWaveAuthority {
    OxCalcRepository,
    OxFmlRuntimeSession,
    OxCalcScheduler,
    OxCalcCoordinator,
    OxCalcReplay,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecalcWaveTraceEvent {
    pub phase: RecalcWavePhase,
    pub authority: RecalcWaveAuthority,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecalcWaveTrace {
    wave_id: String,
    correctness_floor_profile: CorrectnessFloorProfile,
    events: Vec<RecalcWaveTraceEvent>,
}

impl RecalcWaveTrace {
    #[must_use]
    pub fn wave_id(&self) -> &str {
        &self.wave_id
    }

    #[must_use]
    pub fn events(&self) -> &[RecalcWaveTraceEvent] {
        &self.events
    }

    #[must_use]
    pub fn correctness_floor_profile(&self) -> &CorrectnessFloorProfile {
        &self.correctness_floor_profile
    }

    #[must_use]
    pub fn correctness_floor_replay_record(&self) -> CorrectnessFloorReplayRecord {
        self.correctness_floor_profile.replay_record()
    }

    pub fn validate_replay_selectors(
        &self,
        active: &CorrectnessFloorProfile,
    ) -> Result<(), CorrectnessFloorReplayValidationError> {
        CorrectnessFloorProfile::validate_replay_record(
            &self.correctness_floor_replay_record(),
            active,
        )
    }

    #[must_use]
    pub fn phase_sequence(&self) -> Vec<RecalcWavePhase> {
        let mut phases = Vec::new();
        for event in &self.events {
            if phases.last().copied() != Some(event.phase) {
                phases.push(event.phase);
            }
        }
        phases
    }

    fn push(
        &mut self,
        phase: RecalcWavePhase,
        authority: RecalcWaveAuthority,
        detail: impl Into<String>,
    ) {
        self.events.push(RecalcWaveTraceEvent {
            phase,
            authority,
            detail: detail.into(),
        });
    }
}

#[derive(Debug, Error, PartialEq)]
pub enum RecalcWaveError {
    #[error("wave {wave_id} cannot enter {observed:?}; expected {expected:?}")]
    OutOfOrder {
        wave_id: String,
        expected: RecalcWavePhase,
        observed: RecalcWavePhase,
    },
    #[error("wave {wave_id} already has a coordinator decision")]
    DuplicateCoordinatorDecision { wave_id: String },
    #[error("wave {wave_id} is already closed")]
    AlreadyClosed { wave_id: String },
    #[error(transparent)]
    OxfmlSession(#[from] OxfmlRecalcSessionError),
}

pub struct OxfmlRecalcWave<'a> {
    driver: OxfmlRecalcSessionDriver<'a>,
    trace: RecalcWaveTrace,
    last_phase: Option<RecalcWavePhase>,
    coordinator_decision_seen: bool,
    closed: bool,
}

impl<'a> OxfmlRecalcWave<'a> {
    #[must_use]
    pub fn new(wave_id: impl Into<String>, environment: RuntimeEnvironment<'a>) -> Self {
        Self::from_driver(wave_id, OxfmlRecalcSessionDriver::new(environment))
    }

    #[must_use]
    pub fn from_driver(wave_id: impl Into<String>, driver: OxfmlRecalcSessionDriver<'a>) -> Self {
        Self::from_driver_with_correctness_floor_profile(
            wave_id,
            driver,
            CorrectnessFloorProfile::default(),
        )
    }

    #[must_use]
    pub fn new_with_correctness_floor_profile(
        wave_id: impl Into<String>,
        environment: RuntimeEnvironment<'a>,
        correctness_floor_profile: CorrectnessFloorProfile,
    ) -> Self {
        Self::from_driver_with_correctness_floor_profile(
            wave_id,
            OxfmlRecalcSessionDriver::new(environment),
            correctness_floor_profile,
        )
    }

    #[must_use]
    pub fn from_driver_with_correctness_floor_profile(
        wave_id: impl Into<String>,
        driver: OxfmlRecalcSessionDriver<'a>,
        correctness_floor_profile: CorrectnessFloorProfile,
    ) -> Self {
        let wave_id = wave_id.into();
        let profile_key = correctness_floor_profile.replay_profile_key();
        let mut wave = Self {
            driver,
            trace: RecalcWaveTrace {
                wave_id: wave_id.clone(),
                correctness_floor_profile,
                events: Vec::new(),
            },
            last_phase: None,
            coordinator_decision_seen: false,
            closed: false,
        };
        wave.record_phase(
            RecalcWavePhase::WavePreparation,
            RecalcWaveAuthority::OxCalcRepository,
            format!("wave:{wave_id}:open;correctness_floor_profile:{profile_key}"),
        )
        .expect("new wave starts at wave preparation");
        wave
    }

    pub fn ensure_prepared<'q>(
        &mut self,
        request: &RuntimeFormulaRequest<'q>,
    ) -> Result<RuntimeManagedOpenResult, RecalcWaveError> {
        self.record_phase(
            RecalcWavePhase::EnsurePrepared,
            RecalcWaveAuthority::OxFmlRuntimeSession,
            format!("formula:{}", request.source().formula_stable_id.0),
        )?;
        let prepared = self.driver.ensure_prepared(request)?;
        self.record_phase(
            RecalcWavePhase::Compilation,
            RecalcWaveAuthority::OxFmlRuntimeSession,
            compilation_trace_detail(request, &prepared),
        )?;
        Ok(prepared)
    }

    pub fn derive_dependencies(
        &mut self,
        descriptor_count: usize,
        affected_node_count: usize,
    ) -> Result<(), RecalcWaveError> {
        self.record_phase(
            RecalcWavePhase::DependencyDerivation,
            RecalcWaveAuthority::OxCalcRepository,
            format!("descriptors:{descriptor_count};affected_nodes:{affected_node_count}"),
        )
    }

    pub fn invoke<'q>(
        &mut self,
        request: RuntimeFormulaRequest<'q>,
        scheduled_node_count: usize,
    ) -> Result<RuntimeFormulaResult, RecalcWaveError> {
        self.record_phase(
            RecalcWavePhase::ScheduleInvoke,
            RecalcWaveAuthority::OxFmlRuntimeSession,
            format!(
                "formula:{};scheduled_nodes:{}",
                request.source().formula_stable_id.0,
                scheduled_node_count
            ),
        )?;
        self.driver.invoke(request).map_err(Into::into)
    }

    pub fn record_coordinator_publication(
        &mut self,
        publication: &PublicationBundle,
    ) -> Result<(), RecalcWaveError> {
        if self.coordinator_decision_seen {
            return Err(RecalcWaveError::DuplicateCoordinatorDecision {
                wave_id: self.trace.wave_id.clone(),
            });
        }
        self.coordinator_decision_seen = true;
        self.record_phase(
            RecalcWavePhase::CoordinatorCommit,
            RecalcWaveAuthority::OxCalcCoordinator,
            format!(
                "publication:{};candidate:{}",
                publication.publication_id, publication.candidate_result_id
            ),
        )
    }

    pub fn record_coordinator_no_publish(
        &mut self,
        reject: &RejectDetail,
    ) -> Result<(), RecalcWaveError> {
        if self.coordinator_decision_seen {
            return Err(RecalcWaveError::DuplicateCoordinatorDecision {
                wave_id: self.trace.wave_id.clone(),
            });
        }
        self.coordinator_decision_seen = true;
        self.record_phase(
            RecalcWavePhase::CoordinatorCommit,
            RecalcWaveAuthority::OxCalcCoordinator,
            format!(
                "no_publish:{};kind:{:?}",
                reject.candidate_result_id, reject.kind
            ),
        )
    }

    pub fn close_and_capture(
        &mut self,
        replay_artifact_id: impl Into<String>,
    ) -> Result<RecalcWaveTrace, RecalcWaveError> {
        self.record_phase(
            RecalcWavePhase::CloseCapture,
            RecalcWaveAuthority::OxCalcReplay,
            replay_artifact_id,
        )?;
        self.closed = true;
        Ok(self.trace.clone())
    }

    #[must_use]
    pub fn trace(&self) -> &RecalcWaveTrace {
        &self.trace
    }

    fn record_phase(
        &mut self,
        phase: RecalcWavePhase,
        authority: RecalcWaveAuthority,
        detail: impl Into<String>,
    ) -> Result<(), RecalcWaveError> {
        if self.closed {
            return Err(RecalcWaveError::AlreadyClosed {
                wave_id: self.trace.wave_id.clone(),
            });
        }

        if let Some(last_phase) = self.last_phase {
            let repeated_prepare_compile_pair = last_phase == RecalcWavePhase::Compilation
                && phase == RecalcWavePhase::EnsurePrepared;
            let last_ordinal = last_phase.ordinal();
            let observed_ordinal = phase.ordinal();
            if !repeated_prepare_compile_pair
                && (observed_ordinal < last_ordinal || observed_ordinal > last_ordinal + 1)
            {
                let expected = ORDERED_PHASES
                    .get(last_ordinal + 1)
                    .copied()
                    .unwrap_or(last_phase);
                return Err(RecalcWaveError::OutOfOrder {
                    wave_id: self.trace.wave_id.clone(),
                    expected,
                    observed: phase,
                });
            }
        } else if phase != RecalcWavePhase::WavePreparation {
            return Err(RecalcWaveError::OutOfOrder {
                wave_id: self.trace.wave_id.clone(),
                expected: RecalcWavePhase::WavePreparation,
                observed: phase,
            });
        }

        self.last_phase = Some(phase);
        self.trace.push(phase, authority, detail);
        Ok(())
    }
}

fn compilation_trace_detail<'q>(
    request: &RuntimeFormulaRequest<'q>,
    prepared: &RuntimeManagedOpenResult,
) -> String {
    format!(
        "formula:{};session:{};library_context_snapshot_ref:{:?};syntax_diagnostics:{};bind_diagnostics:{};semantic_diagnostics:{};semantic_plan_function_bindings:{}",
        request.source().formula_stable_id.0,
        prepared.session_id,
        prepared.library_context_snapshot_ref,
        prepared.syntax_diagnostics.len(),
        prepared.bind_diagnostics.len(),
        prepared.semantic_plan.diagnostics.len(),
        prepared.semantic_plan.function_bindings.len()
    )
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::fs;
    use std::path::PathBuf;

    use oxfml_core::EvaluationBackend;
    use oxfml_core::consumer::runtime::{RuntimeEnvironment, RuntimeFormulaRequest};
    use oxfml_core::interface::TypedContextQueryBundle;
    use oxfml_core::seam::ValuePayload;
    use oxfml_core::source::FormulaSourceRecord;
    use serde_json::json;

    use crate::coordinator::{AcceptedCandidateResult, TreeCalcCoordinator};
    use crate::correctness_floor::{
        CorrectnessFloorProfile, CorrectnessFloorReplayRecord,
        CorrectnessFloorReplayValidationError,
    };
    use crate::error_algebra::ErrorAlgebra;
    use crate::numerical_reduction::NumericalReductionPolicy;
    use crate::structural::{
        StructuralNode, StructuralNodeKind, StructuralSnapshot, StructuralSnapshotId, TreeNodeId,
    };

    use super::*;

    fn snapshot() -> StructuralSnapshot {
        StructuralSnapshot::create(
            StructuralSnapshotId(1),
            TreeNodeId(1),
            [
                StructuralNode {
                    node_id: TreeNodeId(1),
                    kind: StructuralNodeKind::Root,
                    symbol: "Root".to_string(),
                    parent_id: None,
                    child_ids: vec![TreeNodeId(2)],
                    formula_artifact_id: None,
                    bind_artifact_id: None,
                    constant_value: None,
                },
                StructuralNode {
                    node_id: TreeNodeId(2),
                    kind: StructuralNodeKind::Calculation,
                    symbol: "A".to_string(),
                    parent_id: Some(TreeNodeId(1)),
                    child_ids: vec![],
                    formula_artifact_id: None,
                    bind_artifact_id: None,
                    constant_value: None,
                },
            ],
        )
        .unwrap()
    }

    fn request(formula_stable_id: &str, formula_text: &str) -> RuntimeFormulaRequest<'static> {
        RuntimeFormulaRequest::new(
            FormulaSourceRecord::new(formula_stable_id, 1, formula_text),
            TypedContextQueryBundle::default(),
        )
        .with_backend(EvaluationBackend::OxFuncBacked)
    }

    fn candidate(snapshot: &StructuralSnapshot) -> AcceptedCandidateResult {
        AcceptedCandidateResult {
            candidate_result_id: "candidate:b4".to_string(),
            structural_snapshot_id: snapshot.snapshot_id(),
            artifact_token_basis: "artifact:b4".to_string(),
            compatibility_basis: "compat:b4".to_string(),
            target_set: vec![TreeNodeId(2)],
            value_updates: BTreeMap::from([(TreeNodeId(2), "3".to_string())]),
            dependency_shape_updates: Vec::new(),
            runtime_effects: Vec::new(),
            diagnostic_events: Vec::new(),
        }
    }

    fn e3_artifact_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/test-runs/core-engine/w050-e3-correctness-floor-replay-hooks-001")
    }

    fn correctness_floor_replay_artifact_json() -> serde_json::Value {
        let active = CorrectnessFloorProfile::default();
        let active_record = active.replay_record();
        let numerical_policy_mismatch = CorrectnessFloorReplayRecord {
            numerical_reduction_policy: NumericalReductionPolicy::PairwiseTree
                .selector_key()
                .to_string(),
            ..active_record.clone()
        };
        let error_algebra_mismatch = CorrectnessFloorReplayRecord {
            error_algebra: "ProfileDeclaredTest".to_string(),
            ..active_record.clone()
        };

        json!({
            "run_id": "w050-e3-correctness-floor-replay-hooks-001",
            "validation_status": "pass",
            "primary_validation_command": "cargo test -p oxcalc-core correctness_floor_replay -- --nocapture",
            "active_profile_key": active.replay_profile_key(),
            "trace_replay_record": active_record,
            "accepted_replay": {
                "recorded": active.replay_record(),
                "active": active.replay_record(),
                "validation": "accepted"
            },
            "rejected_replays": [
                {
                    "mismatch_kind": "numerical_reduction_policy",
                    "recorded": numerical_policy_mismatch,
                    "active": active.replay_record(),
                    "expected_error": "correctness_floor_selector_mismatch"
                },
                {
                    "mismatch_kind": "error_algebra",
                    "recorded": error_algebra_mismatch,
                    "active": active.replay_record(),
                    "expected_error": "correctness_floor_selector_mismatch"
                }
            ]
        })
    }

    #[test]
    fn recalc_wave_traces_seven_phases_over_runtime_session() {
        let mut wave = OxfmlRecalcWave::new("wave:b4", RuntimeEnvironment::new());
        let request = request("formula:b4", "=SUM(1,2)");

        let open = wave
            .ensure_prepared(&request)
            .expect("prepare phase should use the OxFml session facade");
        assert!(open.session_id.starts_with("session:"));

        wave.derive_dependencies(0, 1)
            .expect("dependency derivation phase should be accepted");
        let run = wave
            .invoke(request, 1)
            .expect("invoke phase should use the OxFml session facade");
        assert_eq!(
            run.candidate_result.value_delta.published_payload,
            ValuePayload::Number("3".to_string())
        );

        let snapshot = snapshot();
        let mut coordinator = TreeCalcCoordinator::new(snapshot.clone());
        coordinator
            .admit_candidate_work(candidate(&snapshot))
            .expect("candidate should be admitted");
        coordinator
            .record_accepted_candidate_result("candidate:b4")
            .expect("candidate should be accepted");
        let publication = coordinator
            .accept_and_publish("publication:b4")
            .expect("coordinator should publish exactly once");

        wave.record_coordinator_publication(&publication)
            .expect("coordinator publication should be the commit phase");
        let duplicate = wave
            .record_coordinator_publication(&publication)
            .expect_err("a wave must not carry a second coordinator decision");
        assert!(matches!(
            duplicate,
            RecalcWaveError::DuplicateCoordinatorDecision { .. }
        ));

        let trace = wave
            .close_and_capture("replay:b4")
            .expect("close/capture should finish the wave");

        assert_eq!(
            trace.phase_sequence(),
            vec![
                RecalcWavePhase::WavePreparation,
                RecalcWavePhase::EnsurePrepared,
                RecalcWavePhase::Compilation,
                RecalcWavePhase::DependencyDerivation,
                RecalcWavePhase::ScheduleInvoke,
                RecalcWavePhase::CoordinatorCommit,
                RecalcWavePhase::CloseCapture,
            ]
        );
        let commit = trace
            .events()
            .iter()
            .find(|event| event.phase == RecalcWavePhase::CoordinatorCommit)
            .expect("trace should contain coordinator commit phase");
        assert_eq!(commit.authority, RecalcWaveAuthority::OxCalcCoordinator);
        assert_eq!(coordinator.counters().publication_count, 1);
        assert_eq!(
            trace.correctness_floor_replay_record(),
            CorrectnessFloorProfile::default().replay_record()
        );
    }

    #[test]
    fn recalc_wave_records_compilation_as_observable_trace_phase() {
        let mut wave = OxfmlRecalcWave::new("wave:f6:compilation", RuntimeEnvironment::new());
        let request = request("formula:f6:compilation", "=SUM(1,2)");
        let prepared = wave
            .ensure_prepared(&request)
            .expect("prepare should emit compilation phase");

        let trace = wave.trace();
        assert_eq!(
            trace.phase_sequence(),
            vec![
                RecalcWavePhase::WavePreparation,
                RecalcWavePhase::EnsurePrepared,
                RecalcWavePhase::Compilation,
            ]
        );
        let compilation = trace
            .events()
            .iter()
            .find(|event| event.phase == RecalcWavePhase::Compilation)
            .expect("compilation phase should be trace-visible");
        assert_eq!(
            compilation.authority,
            RecalcWaveAuthority::OxFmlRuntimeSession
        );
        assert_eq!(
            compilation.detail,
            format!(
                "formula:formula:f6:compilation;session:{};library_context_snapshot_ref:{:?};syntax_diagnostics:0;bind_diagnostics:0;semantic_diagnostics:{};semantic_plan_function_bindings:{}",
                prepared.session_id,
                prepared.library_context_snapshot_ref,
                prepared.semantic_plan.diagnostics.len(),
                prepared.semantic_plan.function_bindings.len()
            )
        );
    }

    #[test]
    fn recalc_wave_allows_multiple_prepare_compile_pairs_before_dependency_derivation() {
        let mut wave = OxfmlRecalcWave::new("wave:f6:multi-prepare", RuntimeEnvironment::new());
        wave.ensure_prepared(&request("formula:f6:first", "=SUM(1,2)"))
            .expect("first prepare should compile");
        wave.ensure_prepared(&request("formula:f6:second", "=SUM(3,4)"))
            .expect("second prepare should compile before dependency derivation");
        wave.derive_dependencies(0, 2)
            .expect("dependency derivation follows repeated prepare/compile pairs");

        assert_eq!(
            wave.trace().phase_sequence(),
            vec![
                RecalcWavePhase::WavePreparation,
                RecalcWavePhase::EnsurePrepared,
                RecalcWavePhase::Compilation,
                RecalcWavePhase::EnsurePrepared,
                RecalcWavePhase::Compilation,
                RecalcWavePhase::DependencyDerivation,
            ]
        );
    }

    #[test]
    fn recalc_wave_rejects_skipped_dependency_phase() {
        let mut wave = OxfmlRecalcWave::new("wave:b4:skip", RuntimeEnvironment::new());
        let request = request("formula:b4:skip", "=SUM(1,2)");
        wave.ensure_prepared(&request)
            .expect("prepare phase should be accepted");

        let error = wave
            .invoke(request, 1)
            .expect_err("invoke before dependency derivation should be rejected");

        assert_eq!(
            error,
            RecalcWaveError::OutOfOrder {
                wave_id: "wave:b4:skip".to_string(),
                expected: RecalcWavePhase::DependencyDerivation,
                observed: RecalcWavePhase::ScheduleInvoke,
            }
        );
    }

    #[test]
    fn correctness_floor_replay_selectors_are_recorded_in_wave_trace() {
        let profile = CorrectnessFloorProfile::new(
            "profile:correctness-floor:test",
            NumericalReductionPolicy::KahanCompensated,
            ErrorAlgebra::CanonicalExcelLegacy,
        );
        let wave = OxfmlRecalcWave::new_with_correctness_floor_profile(
            "wave:profile-selectors",
            RuntimeEnvironment::new(),
            profile.clone(),
        );
        let trace = wave.trace();

        assert_eq!(trace.correctness_floor_profile(), &profile);
        assert_eq!(
            trace.correctness_floor_replay_record(),
            CorrectnessFloorReplayRecord {
                profile_version: "profile:correctness-floor:test".to_string(),
                numerical_reduction_policy: "KahanCompensated".to_string(),
                error_algebra: "CanonicalExcelLegacy".to_string(),
                semantic_kernel_metadata_version: None,
            }
        );
        assert_eq!(trace.validate_replay_selectors(&profile), Ok(()));
        assert!(
            trace.events()[0]
                .detail
                .contains("correctness_floor_profile:profile:correctness-floor:test|numerical_reduction_policy:KahanCompensated|error_algebra:CanonicalExcelLegacy")
        );
    }

    #[test]
    fn correctness_floor_replay_rejects_selector_mismatch() {
        let active = CorrectnessFloorProfile::default();
        let mut recorded = active.replay_record();
        recorded.numerical_reduction_policy = "PairwiseTree".to_string();

        let error = CorrectnessFloorProfile::validate_replay_record(&recorded, &active)
            .expect_err("recorded numerical policy mismatch should reject replay");
        assert!(matches!(
            error,
            CorrectnessFloorReplayValidationError::SelectorMismatch { .. }
        ));

        let mut recorded = active.replay_record();
        recorded.error_algebra = "ProfileDeclaredTest".to_string();
        let error = CorrectnessFloorProfile::validate_replay_record(&recorded, &active)
            .expect_err("recorded error algebra mismatch should reject replay");
        assert!(matches!(
            error,
            CorrectnessFloorReplayValidationError::SelectorMismatch { .. }
        ));
    }

    #[test]
    fn checked_in_correctness_floor_replay_hook_artifact_matches_runtime_validation() {
        let artifact_path = e3_artifact_root().join("run_artifact.json");
        let artifact = serde_json::from_str::<serde_json::Value>(
            &fs::read_to_string(artifact_path).expect("E3 run artifact should be checked in"),
        )
        .expect("E3 run artifact should be valid JSON");

        assert_eq!(artifact, correctness_floor_replay_artifact_json());
    }
}
