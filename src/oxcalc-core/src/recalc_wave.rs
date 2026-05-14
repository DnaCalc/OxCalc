#![forbid(unsafe_code)]

//! W050 recalc-wave lifecycle over the OxFml runtime session driver.

use oxfml_core::consumer::runtime::{
    RuntimeEnvironment, RuntimeFormulaRequest, RuntimeFormulaResult, RuntimeManagedOpenResult,
};
use thiserror::Error;

use crate::coordinator::{PublicationBundle, RejectDetail};
use crate::oxfml_session::{OxfmlRecalcSessionDriver, OxfmlRecalcSessionError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RecalcWavePhase {
    WavePreparation,
    EnsurePrepared,
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
            Self::DependencyDerivation => 2,
            Self::ScheduleInvoke => 3,
            Self::CoordinatorCommit => 4,
            Self::CloseCapture => 5,
        }
    }
}

const ORDERED_PHASES: [RecalcWavePhase; 6] = [
    RecalcWavePhase::WavePreparation,
    RecalcWavePhase::EnsurePrepared,
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
        let wave_id = wave_id.into();
        let mut wave = Self {
            driver,
            trace: RecalcWaveTrace {
                wave_id: wave_id.clone(),
                events: Vec::new(),
            },
            last_phase: None,
            coordinator_decision_seen: false,
            closed: false,
        };
        wave.record_phase(
            RecalcWavePhase::WavePreparation,
            RecalcWaveAuthority::OxCalcRepository,
            format!("wave:{wave_id}:open"),
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
        self.driver.ensure_prepared(request).map_err(Into::into)
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
            let last_ordinal = last_phase.ordinal();
            let observed_ordinal = phase.ordinal();
            if observed_ordinal < last_ordinal || observed_ordinal > last_ordinal + 1 {
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

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use oxfml_core::EvaluationBackend;
    use oxfml_core::consumer::runtime::{RuntimeEnvironment, RuntimeFormulaRequest};
    use oxfml_core::interface::TypedContextQueryBundle;
    use oxfml_core::seam::ValuePayload;
    use oxfml_core::source::FormulaSourceRecord;

    use crate::coordinator::{AcceptedCandidateResult, TreeCalcCoordinator};
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

    #[test]
    fn recalc_wave_traces_six_phases_over_runtime_session() {
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
}
