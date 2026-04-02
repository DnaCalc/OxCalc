#![forbid(unsafe_code)]

//! Recalc and overlay lane.

use std::collections::{BTreeMap, BTreeSet};

use thiserror::Error;

use crate::structural::{StructuralSnapshot, StructuralSnapshotId, TreeNodeId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NodeCalcState {
    Clean,
    DirtyPending,
    Needed,
    Evaluating,
    VerifiedClean,
    PublishReady,
    RejectedPendingRepair,
    CycleBlocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OverlayKind {
    InvalidationExecutionState,
    DynamicDependency,
    ExecutionRestriction,
    ShapeTopology,
    CapabilityFenceAttachment,
    ObserverPriorityMetadata,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OverlayKey {
    pub owner_node_id: TreeNodeId,
    pub overlay_kind: OverlayKind,
    pub structural_snapshot_id: StructuralSnapshotId,
    pub compatibility_basis: String,
    pub payload_identity: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverlayEntry {
    pub key: OverlayKey,
    pub is_protected: bool,
    pub is_eviction_eligible: bool,
    pub detail: String,
}

#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum RecalcError {
    #[error("node {node_id} must be in state {expected:?} but is {observed:?}")]
    WrongState {
        node_id: TreeNodeId,
        expected: NodeCalcState,
        observed: NodeCalcState,
    },
    #[error("node {node_id} is not eligible for reject/fallback from state {observed:?}")]
    RejectNotAllowed {
        node_id: TreeNodeId,
        observed: NodeCalcState,
    },
    #[error("node {node_id} is not eligible for release from state {observed:?}")]
    ReleaseNotAllowed {
        node_id: TreeNodeId,
        observed: NodeCalcState,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stage1RecalcTracker {
    snapshot: StructuralSnapshot,
    node_states: BTreeMap<TreeNodeId, NodeCalcState>,
    overlays: BTreeMap<OverlayKey, OverlayEntry>,
    demand_set: BTreeSet<TreeNodeId>,
}

impl Stage1RecalcTracker {
    #[must_use]
    pub fn new(snapshot: StructuralSnapshot) -> Self {
        let node_states = snapshot
            .nodes()
            .keys()
            .map(|node_id| (*node_id, NodeCalcState::Clean))
            .collect();
        Self {
            snapshot,
            node_states,
            overlays: BTreeMap::new(),
            demand_set: BTreeSet::new(),
        }
    }

    #[must_use]
    pub fn snapshot(&self) -> &StructuralSnapshot {
        &self.snapshot
    }

    #[must_use]
    pub fn node_states(&self) -> &BTreeMap<TreeNodeId, NodeCalcState> {
        &self.node_states
    }

    #[must_use]
    pub fn overlays(&self) -> &BTreeMap<OverlayKey, OverlayEntry> {
        &self.overlays
    }

    #[must_use]
    pub fn demand_set(&self) -> &BTreeSet<TreeNodeId> {
        &self.demand_set
    }

    #[must_use]
    pub fn get_state(&self, node_id: TreeNodeId) -> NodeCalcState {
        self.node_states
            .get(&node_id)
            .copied()
            .unwrap_or(NodeCalcState::Clean)
    }

    pub fn mark_dirty(&mut self, node_id: TreeNodeId) {
        self.node_states
            .insert(node_id, NodeCalcState::DirtyPending);
        self.protect_execution_overlay(node_id, "dirty_pending");
    }

    pub fn mark_needed(&mut self, node_id: TreeNodeId) -> Result<(), RecalcError> {
        self.require_state(node_id, NodeCalcState::DirtyPending)?;
        self.node_states.insert(node_id, NodeCalcState::Needed);
        self.demand_set.insert(node_id);
        self.protect_execution_overlay(node_id, "needed");
        Ok(())
    }

    pub fn begin_evaluate(
        &mut self,
        node_id: TreeNodeId,
        compatibility_basis: &str,
    ) -> Result<(), RecalcError> {
        self.require_state(node_id, NodeCalcState::Needed)?;
        self.node_states.insert(node_id, NodeCalcState::Evaluating);
        self.protect_execution_overlay(node_id, "evaluating");
        self.upsert_overlay(
            OverlayKey {
                owner_node_id: node_id,
                overlay_kind: OverlayKind::CapabilityFenceAttachment,
                structural_snapshot_id: self.snapshot.snapshot_id(),
                compatibility_basis: compatibility_basis.to_string(),
                payload_identity: None,
            },
            true,
            false,
            "evaluation_basis",
        );
        Ok(())
    }

    pub fn verify_clean(&mut self, node_id: TreeNodeId) -> Result<(), RecalcError> {
        self.require_state(node_id, NodeCalcState::Evaluating)?;
        self.node_states
            .insert(node_id, NodeCalcState::VerifiedClean);
        self.demand_set.remove(&node_id);
        self.protect_execution_overlay(node_id, "verified_clean");
        Ok(())
    }

    pub fn produce_candidate_result(
        &mut self,
        node_id: TreeNodeId,
        compatibility_basis: &str,
        payload_identity: &str,
    ) -> Result<(), RecalcError> {
        self.require_state(node_id, NodeCalcState::Evaluating)?;
        self.node_states
            .insert(node_id, NodeCalcState::PublishReady);
        self.protect_execution_overlay(node_id, "publish_ready");
        self.upsert_overlay(
            OverlayKey {
                owner_node_id: node_id,
                overlay_kind: OverlayKind::CapabilityFenceAttachment,
                structural_snapshot_id: self.snapshot.snapshot_id(),
                compatibility_basis: compatibility_basis.to_string(),
                payload_identity: Some(payload_identity.to_string()),
            },
            true,
            false,
            "candidate_ready",
        );
        Ok(())
    }

    pub fn produce_dependency_shape_update(
        &mut self,
        node_id: TreeNodeId,
        compatibility_basis: &str,
        payload_identity: &str,
    ) -> Result<(), RecalcError> {
        self.require_state(node_id, NodeCalcState::Evaluating)?;
        self.node_states
            .insert(node_id, NodeCalcState::PublishReady);
        self.upsert_overlay(
            OverlayKey {
                owner_node_id: node_id,
                overlay_kind: OverlayKind::DynamicDependency,
                structural_snapshot_id: self.snapshot.snapshot_id(),
                compatibility_basis: compatibility_basis.to_string(),
                payload_identity: Some(payload_identity.to_string()),
            },
            true,
            false,
            "candidate_shape_update",
        );
        self.protect_execution_overlay(node_id, "publish_ready");
        Ok(())
    }

    pub fn reject_or_fallback(
        &mut self,
        node_id: TreeNodeId,
        reason: &str,
    ) -> Result<(), RecalcError> {
        let observed = self.get_state(node_id);
        if observed != NodeCalcState::Evaluating && observed != NodeCalcState::PublishReady {
            return Err(RecalcError::RejectNotAllowed { node_id, observed });
        }

        self.node_states
            .insert(node_id, NodeCalcState::RejectedPendingRepair);
        self.demand_set.insert(node_id);
        self.overlays.retain(|key, _| {
            !(key.owner_node_id == node_id && key.overlay_kind == OverlayKind::DynamicDependency)
        });
        self.protect_execution_overlay(node_id, &format!("fallback:{reason}"));
        Ok(())
    }

    pub fn reenter_rejected_pending_repair(
        &mut self,
        node_id: TreeNodeId,
    ) -> Result<(), RecalcError> {
        self.require_state(node_id, NodeCalcState::RejectedPendingRepair)?;
        self.node_states.insert(node_id, NodeCalcState::Needed);
        self.protect_execution_overlay(node_id, "reenter_needed");
        Ok(())
    }

    pub fn publish_and_clear(&mut self, node_id: TreeNodeId) -> Result<(), RecalcError> {
        self.require_state(node_id, NodeCalcState::PublishReady)?;
        self.node_states.insert(node_id, NodeCalcState::Clean);
        self.demand_set.remove(&node_id);
        self.mark_execution_overlay_eligible(node_id, "published");
        Ok(())
    }

    pub fn release_and_evict_eligible(&mut self, node_id: TreeNodeId) -> Result<(), RecalcError> {
        let observed = self.get_state(node_id);
        if observed != NodeCalcState::Clean && observed != NodeCalcState::VerifiedClean {
            return Err(RecalcError::ReleaseNotAllowed { node_id, observed });
        }

        self.demand_set.remove(&node_id);
        let keys = self
            .overlays
            .keys()
            .filter(|key| key.owner_node_id == node_id)
            .cloned()
            .collect::<Vec<_>>();
        for key in keys {
            if let Some(entry) = self.overlays.get_mut(&key) {
                entry.is_protected = false;
                entry.is_eviction_eligible = true;
            }
        }
        Ok(())
    }

    pub fn evict_eligible_overlays(&mut self) -> usize {
        let to_evict = self
            .overlays
            .iter()
            .filter(|(_, entry)| entry.is_eviction_eligible && !entry.is_protected)
            .map(|(key, _)| key.clone())
            .collect::<Vec<_>>();
        let count = to_evict.len();
        for key in to_evict {
            self.overlays.remove(&key);
        }
        count
    }

    fn protect_execution_overlay(&mut self, node_id: TreeNodeId, detail: &str) {
        self.upsert_overlay(
            OverlayKey {
                owner_node_id: node_id,
                overlay_kind: OverlayKind::InvalidationExecutionState,
                structural_snapshot_id: self.snapshot.snapshot_id(),
                compatibility_basis: "stage1".to_string(),
                payload_identity: None,
            },
            true,
            false,
            detail,
        );
    }

    fn mark_execution_overlay_eligible(&mut self, node_id: TreeNodeId, detail: &str) {
        self.upsert_overlay(
            OverlayKey {
                owner_node_id: node_id,
                overlay_kind: OverlayKind::InvalidationExecutionState,
                structural_snapshot_id: self.snapshot.snapshot_id(),
                compatibility_basis: "stage1".to_string(),
                payload_identity: None,
            },
            false,
            true,
            detail,
        );
    }

    fn upsert_overlay(
        &mut self,
        key: OverlayKey,
        is_protected: bool,
        is_eviction_eligible: bool,
        detail: &str,
    ) {
        self.overlays.insert(
            key.clone(),
            OverlayEntry {
                key,
                is_protected,
                is_eviction_eligible,
                detail: detail.to_string(),
            },
        );
    }

    fn require_state(
        &self,
        node_id: TreeNodeId,
        expected: NodeCalcState,
    ) -> Result<(), RecalcError> {
        let observed = self.get_state(node_id);
        if observed != expected {
            return Err(RecalcError::WrongState {
                node_id,
                expected,
                observed,
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
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

    #[test]
    fn tracker_marks_publish_and_release() {
        let snapshot = snapshot();
        let mut tracker = Stage1RecalcTracker::new(snapshot);

        tracker.mark_dirty(TreeNodeId(2));
        tracker.mark_needed(TreeNodeId(2)).unwrap();
        tracker.begin_evaluate(TreeNodeId(2), "s0").unwrap();
        tracker
            .produce_candidate_result(TreeNodeId(2), "s0", "cand1")
            .unwrap();
        tracker.publish_and_clear(TreeNodeId(2)).unwrap();
        tracker.release_and_evict_eligible(TreeNodeId(2)).unwrap();

        assert_eq!(tracker.get_state(TreeNodeId(2)), NodeCalcState::Clean);
        assert!(tracker.evict_eligible_overlays() > 0);
    }
}
