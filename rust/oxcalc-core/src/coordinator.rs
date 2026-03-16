#![forbid(unsafe_code)]

//! Coordinator and publication lane.

use std::collections::BTreeMap;

use thiserror::Error;

use crate::structural::{
    PinnedStructuralView, StructuralSnapshot, StructuralSnapshotId, TreeNodeId,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeEffect {
    pub kind: String,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyShapeUpdate {
    pub kind: String,
    pub affected_node_ids: Vec<TreeNodeId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcceptedCandidateResult {
    pub candidate_result_id: String,
    pub structural_snapshot_id: StructuralSnapshotId,
    pub artifact_token_basis: String,
    pub compatibility_basis: String,
    pub target_set: Vec<TreeNodeId>,
    pub value_updates: BTreeMap<TreeNodeId, String>,
    pub dependency_shape_updates: Vec<DependencyShapeUpdate>,
    pub runtime_effects: Vec<RuntimeEffect>,
    pub diagnostic_events: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicationBundle {
    pub publication_id: String,
    pub candidate_result_id: String,
    pub structural_snapshot_id: StructuralSnapshotId,
    pub published_view_delta: BTreeMap<TreeNodeId, String>,
    pub published_runtime_effects: Vec<RuntimeEffect>,
    pub trace_markers: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishedView {
    pub snapshot: StructuralSnapshot,
    pub publication: Option<PublicationBundle>,
    pub values: BTreeMap<TreeNodeId, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PinnedPublicationView {
    pub reader_id: String,
    pub snapshot_id: StructuralSnapshotId,
    pub publication_id: Option<String>,
    pub structural_view: PinnedStructuralView,
    pub values: BTreeMap<TreeNodeId, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RejectKind {
    SnapshotMismatch,
    ArtifactTokenMismatch,
    ProfileVersionMismatch,
    CapabilityMismatch,
    PublicationFenceMismatch,
    DynamicDependencyFailure,
    SyntheticCycleReject,
    HostInjectedFailure,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RejectDetail {
    pub candidate_result_id: String,
    pub kind: RejectKind,
    pub detail: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CoordinatorCounters {
    pub publication_count: u32,
    pub reject_count: u32,
    pub pin_count: u32,
}

impl CoordinatorCounters {
    #[must_use]
    pub fn increment_publications(self) -> Self {
        Self {
            publication_count: self.publication_count + 1,
            ..self
        }
    }

    #[must_use]
    pub fn increment_rejects(self) -> Self {
        Self {
            reject_count: self.reject_count + 1,
            ..self
        }
    }

    #[must_use]
    pub fn increment_pins(self) -> Self {
        Self {
            pin_count: self.pin_count + 1,
            ..self
        }
    }
}

#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum CoordinatorError {
    #[error(
        "candidate snapshot {candidate_snapshot_id} does not match coordinator snapshot {coordinator_snapshot_id}"
    )]
    SnapshotMismatch {
        candidate_snapshot_id: StructuralSnapshotId,
        coordinator_snapshot_id: StructuralSnapshotId,
    },
    #[error("candidate {candidate_result_id} is not currently admitted")]
    CandidateNotAdmitted { candidate_result_id: String },
    #[error("no accepted candidate result is available for publication")]
    MissingAcceptedCandidate,
    #[error("candidate {candidate_result_id} is not currently known to the coordinator")]
    UnknownCandidate { candidate_result_id: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeCalcCoordinator {
    snapshot: StructuralSnapshot,
    in_flight_candidate: Option<AcceptedCandidateResult>,
    accepted_candidate: Option<AcceptedCandidateResult>,
    published_view: PublishedView,
    counters: CoordinatorCounters,
    reject_log: Vec<RejectDetail>,
    pins: BTreeMap<String, PinnedPublicationView>,
}

impl TreeCalcCoordinator {
    #[must_use]
    pub fn new(snapshot: StructuralSnapshot) -> Self {
        let published_view = PublishedView {
            snapshot: snapshot.clone(),
            publication: None,
            values: BTreeMap::new(),
        };

        Self {
            snapshot,
            in_flight_candidate: None,
            accepted_candidate: None,
            published_view,
            counters: CoordinatorCounters::default(),
            reject_log: Vec::new(),
            pins: BTreeMap::new(),
        }
    }

    #[must_use]
    pub fn snapshot(&self) -> &StructuralSnapshot {
        &self.snapshot
    }

    #[must_use]
    pub fn in_flight_candidate(&self) -> Option<&AcceptedCandidateResult> {
        self.in_flight_candidate.as_ref()
    }

    #[must_use]
    pub fn accepted_candidate(&self) -> Option<&AcceptedCandidateResult> {
        self.accepted_candidate.as_ref()
    }

    #[must_use]
    pub fn published_view(&self) -> &PublishedView {
        &self.published_view
    }

    #[must_use]
    pub fn counters(&self) -> CoordinatorCounters {
        self.counters
    }

    #[must_use]
    pub fn reject_log(&self) -> &[RejectDetail] {
        &self.reject_log
    }

    #[must_use]
    pub fn pinned_readers(&self) -> Vec<PinnedPublicationView> {
        self.pins.values().cloned().collect()
    }

    pub fn seed_published_view(
        &mut self,
        values: &BTreeMap<TreeNodeId, String>,
        publication_id: Option<&str>,
        runtime_effects: &[RuntimeEffect],
    ) {
        let publication = publication_id.map(|publication_id| PublicationBundle {
            publication_id: publication_id.to_string(),
            candidate_result_id: "seed:published-view".to_string(),
            structural_snapshot_id: self.snapshot.snapshot_id(),
            published_view_delta: values.clone(),
            published_runtime_effects: runtime_effects.to_vec(),
            trace_markers: vec!["publication_seeded".to_string()],
        });

        self.published_view = PublishedView {
            snapshot: self.snapshot.clone(),
            publication,
            values: values.clone(),
        };
    }

    pub fn admit_candidate_work(
        &mut self,
        candidate: AcceptedCandidateResult,
    ) -> Result<(), CoordinatorError> {
        if candidate.structural_snapshot_id != self.snapshot.snapshot_id() {
            return Err(CoordinatorError::SnapshotMismatch {
                candidate_snapshot_id: candidate.structural_snapshot_id,
                coordinator_snapshot_id: self.snapshot.snapshot_id(),
            });
        }

        self.in_flight_candidate = Some(candidate);
        Ok(())
    }

    pub fn record_accepted_candidate_result(
        &mut self,
        candidate_result_id: &str,
    ) -> Result<(), CoordinatorError> {
        match &self.in_flight_candidate {
            Some(candidate) if candidate.candidate_result_id == candidate_result_id => {
                self.accepted_candidate = self.in_flight_candidate.clone();
                Ok(())
            }
            _ => Err(CoordinatorError::CandidateNotAdmitted {
                candidate_result_id: candidate_result_id.to_string(),
            }),
        }
    }

    pub fn accept_and_publish(
        &mut self,
        publication_id: &str,
    ) -> Result<PublicationBundle, CoordinatorError> {
        let accepted_candidate = self
            .accepted_candidate
            .clone()
            .ok_or(CoordinatorError::MissingAcceptedCandidate)?;

        let mut published_values = self.published_view.values.clone();
        published_values.extend(accepted_candidate.value_updates.clone());

        let bundle = PublicationBundle {
            publication_id: publication_id.to_string(),
            candidate_result_id: accepted_candidate.candidate_result_id.clone(),
            structural_snapshot_id: self.snapshot.snapshot_id(),
            published_view_delta: accepted_candidate.value_updates.clone(),
            published_runtime_effects: accepted_candidate.runtime_effects.clone(),
            trace_markers: vec!["publication_committed".to_string()],
        };

        self.published_view = PublishedView {
            snapshot: self.snapshot.clone(),
            publication: Some(bundle.clone()),
            values: published_values,
        };
        self.in_flight_candidate = None;
        self.accepted_candidate = None;
        self.counters = self.counters.increment_publications();
        Ok(bundle)
    }

    pub fn reject_candidate_work(
        &mut self,
        candidate_result_id: &str,
        kind: RejectKind,
        detail: &str,
    ) -> Result<RejectDetail, CoordinatorError> {
        let in_flight_matches = self
            .in_flight_candidate
            .as_ref()
            .is_some_and(|candidate| candidate.candidate_result_id == candidate_result_id);
        let accepted_matches = self
            .accepted_candidate
            .as_ref()
            .is_some_and(|candidate| candidate.candidate_result_id == candidate_result_id);

        if !(in_flight_matches || accepted_matches) {
            return Err(CoordinatorError::UnknownCandidate {
                candidate_result_id: candidate_result_id.to_string(),
            });
        }

        let reject = RejectDetail {
            candidate_result_id: candidate_result_id.to_string(),
            kind,
            detail: detail.to_string(),
        };
        self.reject_log.push(reject.clone());

        if in_flight_matches {
            self.in_flight_candidate = None;
        }

        if accepted_matches {
            self.accepted_candidate = None;
        }

        self.counters = self.counters.increment_rejects();
        Ok(reject)
    }

    pub fn pin_reader(&mut self, reader_id: &str) -> PinnedPublicationView {
        let view = PinnedPublicationView {
            reader_id: reader_id.to_string(),
            snapshot_id: self.snapshot.snapshot_id(),
            publication_id: self
                .published_view
                .publication
                .as_ref()
                .map(|publication| publication.publication_id.clone()),
            structural_view: self.snapshot.pin(),
            values: self.published_view.values.clone(),
        };
        self.pins.insert(reader_id.to_string(), view.clone());
        self.counters = self.counters.increment_pins();
        view
    }

    pub fn unpin_reader(&mut self, reader_id: &str) -> bool {
        self.pins.remove(reader_id).is_some()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

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
                },
                StructuralNode {
                    node_id: TreeNodeId(2),
                    kind: StructuralNodeKind::Calculation,
                    symbol: "A".to_string(),
                    parent_id: Some(TreeNodeId(1)),
                    child_ids: vec![],
                    formula_artifact_id: None,
                },
            ],
        )
        .unwrap()
    }

    #[test]
    fn coordinator_publishes_accepted_candidate() {
        let snapshot = snapshot();
        let mut coordinator = TreeCalcCoordinator::new(snapshot);
        let mut updates = BTreeMap::new();
        updates.insert(TreeNodeId(2), "1".to_string());

        let candidate = AcceptedCandidateResult {
            candidate_result_id: "cand1".to_string(),
            structural_snapshot_id: StructuralSnapshotId(1),
            artifact_token_basis: "s0".to_string(),
            compatibility_basis: "s0".to_string(),
            target_set: vec![TreeNodeId(2)],
            value_updates: updates,
            dependency_shape_updates: vec![],
            runtime_effects: vec![],
            diagnostic_events: vec![],
        };

        coordinator.admit_candidate_work(candidate).unwrap();
        coordinator
            .record_accepted_candidate_result("cand1")
            .unwrap();
        let publication = coordinator.accept_and_publish("pub1").unwrap();

        assert_eq!(publication.publication_id, "pub1");
        assert_eq!(coordinator.counters().publication_count, 1);
    }
}
