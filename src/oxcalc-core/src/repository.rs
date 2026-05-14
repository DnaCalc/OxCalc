#![forbid(unsafe_code)]

//! Calculation Repository substrate for W050 session-shaped recalc.

use std::collections::BTreeMap;

use thiserror::Error;

use crate::dependency::{DependencyDescriptor, DependencyGraph, InvalidationClosure};
use crate::recalc::{NodeCalcState, OverlayEntry, OverlayKey};
use crate::structural::{
    BindArtifactId, FormulaArtifactId, PinnedStructuralView, StructuralSnapshot,
    StructuralSnapshotId, TreeNodeId,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormulaSourceIdentity {
    pub formula_stable_id: String,
    pub formula_text_version: u64,
    pub formula_token: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormulaSlotRecord {
    pub owner_node_id: TreeNodeId,
    pub formula_artifact_id: FormulaArtifactId,
    pub bind_artifact_id: Option<BindArtifactId>,
    pub source_identity: FormulaSourceIdentity,
    pub opaque_source_text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OxfmlArtifactKind {
    FormulaSource,
    GreenTree,
    RedView,
    BoundFormula,
    SemanticPlan,
    PreparedCallable,
    PlanTemplate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OxfmlArtifactHandle {
    pub formula_artifact_id: FormulaArtifactId,
    pub kind: OxfmlArtifactKind,
    pub artifact_key: String,
    pub structure_context_version: String,
    pub profile_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepositoryPinnedReaderView {
    pub reader_id: String,
    pub snapshot_id: StructuralSnapshotId,
    pub structural_view: PinnedStructuralView,
    pub published_values: BTreeMap<TreeNodeId, String>,
}

#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum CalculationRepositoryError {
    #[error("node {node_id} is not present in repository snapshot {snapshot_id}")]
    UnknownNode {
        snapshot_id: StructuralSnapshotId,
        node_id: TreeNodeId,
    },
    #[error("formula slot owner {node_id} does not match record owner {record_owner_node_id}")]
    FormulaSlotOwnerMismatch {
        node_id: TreeNodeId,
        record_owner_node_id: TreeNodeId,
    },
    #[error("formula artifact {formula_artifact_id} is not registered in the repository")]
    UnknownFormulaArtifact {
        formula_artifact_id: FormulaArtifactId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CalculationRepository {
    structural_snapshot: StructuralSnapshot,
    formula_slots: BTreeMap<TreeNodeId, FormulaSlotRecord>,
    dependency_graph: DependencyGraph,
    node_states: BTreeMap<TreeNodeId, NodeCalcState>,
    overlays: BTreeMap<OverlayKey, OverlayEntry>,
    pinned_reader_views: BTreeMap<String, RepositoryPinnedReaderView>,
    oxfml_artifact_handles: BTreeMap<(FormulaArtifactId, OxfmlArtifactKind), OxfmlArtifactHandle>,
    published_values: BTreeMap<TreeNodeId, String>,
}

impl CalculationRepository {
    #[must_use]
    pub fn new(structural_snapshot: StructuralSnapshot) -> Self {
        let dependency_graph = DependencyGraph::build(&structural_snapshot, &[]);
        let node_states = structural_snapshot
            .nodes()
            .keys()
            .map(|node_id| (*node_id, NodeCalcState::Clean))
            .collect();

        Self {
            structural_snapshot,
            formula_slots: BTreeMap::new(),
            dependency_graph,
            node_states,
            overlays: BTreeMap::new(),
            pinned_reader_views: BTreeMap::new(),
            oxfml_artifact_handles: BTreeMap::new(),
            published_values: BTreeMap::new(),
        }
    }

    #[must_use]
    pub fn structural_snapshot(&self) -> &StructuralSnapshot {
        &self.structural_snapshot
    }

    #[must_use]
    pub fn formula_slots(&self) -> &BTreeMap<TreeNodeId, FormulaSlotRecord> {
        &self.formula_slots
    }

    #[must_use]
    pub fn dependency_graph(&self) -> &DependencyGraph {
        &self.dependency_graph
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
    pub fn pinned_reader_views(&self) -> &BTreeMap<String, RepositoryPinnedReaderView> {
        &self.pinned_reader_views
    }

    #[must_use]
    pub fn oxfml_artifact_handles(
        &self,
    ) -> &BTreeMap<(FormulaArtifactId, OxfmlArtifactKind), OxfmlArtifactHandle> {
        &self.oxfml_artifact_handles
    }

    #[must_use]
    pub fn published_values(&self) -> &BTreeMap<TreeNodeId, String> {
        &self.published_values
    }

    pub fn upsert_formula_slot(
        &mut self,
        owner_node_id: TreeNodeId,
        record: FormulaSlotRecord,
    ) -> Result<(), CalculationRepositoryError> {
        self.require_node(owner_node_id)?;
        if record.owner_node_id != owner_node_id {
            return Err(CalculationRepositoryError::FormulaSlotOwnerMismatch {
                node_id: owner_node_id,
                record_owner_node_id: record.owner_node_id,
            });
        }

        self.formula_slots.insert(owner_node_id, record);
        self.node_states
            .insert(owner_node_id, NodeCalcState::DirtyPending);
        Ok(())
    }

    pub fn remove_formula_slot(
        &mut self,
        owner_node_id: TreeNodeId,
    ) -> Result<Option<FormulaSlotRecord>, CalculationRepositoryError> {
        self.require_node(owner_node_id)?;
        let removed = self.formula_slots.remove(&owner_node_id);
        if removed.is_some() {
            self.node_states
                .insert(owner_node_id, NodeCalcState::DirtyPending);
        }
        Ok(removed)
    }

    pub fn record_oxfml_artifact_handle(
        &mut self,
        handle: OxfmlArtifactHandle,
    ) -> Result<(), CalculationRepositoryError> {
        let owner_node_id = self
            .formula_slots
            .iter()
            .find_map(|(node_id, slot)| {
                (slot.formula_artifact_id == handle.formula_artifact_id).then_some(*node_id)
            })
            .ok_or_else(|| CalculationRepositoryError::UnknownFormulaArtifact {
                formula_artifact_id: handle.formula_artifact_id.clone(),
            })?;
        self.require_node(owner_node_id)?;
        self.oxfml_artifact_handles.insert(
            (handle.formula_artifact_id.clone(), handle.kind.clone()),
            handle,
        );
        Ok(())
    }

    pub fn rebuild_dependency_graph(&mut self, descriptors: &[DependencyDescriptor]) {
        self.dependency_graph = DependencyGraph::build(&self.structural_snapshot, descriptors);
    }

    pub fn apply_invalidation_closure(&mut self, closure: &InvalidationClosure) {
        for record in closure.records.values() {
            self.node_states.insert(record.node_id, record.calc_state);
        }
    }

    pub fn upsert_overlay(&mut self, entry: OverlayEntry) {
        self.overlays.insert(entry.key.clone(), entry);
    }

    pub fn seed_published_value(
        &mut self,
        node_id: TreeNodeId,
        value: impl Into<String>,
    ) -> Result<(), CalculationRepositoryError> {
        self.require_node(node_id)?;
        self.published_values.insert(node_id, value.into());
        Ok(())
    }

    pub fn pin_reader_view(&mut self, reader_id: impl Into<String>) -> RepositoryPinnedReaderView {
        let reader_id = reader_id.into();
        let view = RepositoryPinnedReaderView {
            reader_id: reader_id.clone(),
            snapshot_id: self.structural_snapshot.snapshot_id(),
            structural_view: self.structural_snapshot.pin(),
            published_values: self.published_values.clone(),
        };
        self.pinned_reader_views.insert(reader_id, view.clone());
        view
    }

    pub fn unpin_reader_view(&mut self, reader_id: &str) -> bool {
        self.pinned_reader_views.remove(reader_id).is_some()
    }

    fn require_node(&self, node_id: TreeNodeId) -> Result<(), CalculationRepositoryError> {
        if self.structural_snapshot.try_get_node(node_id).is_some() {
            return Ok(());
        }

        Err(CalculationRepositoryError::UnknownNode {
            snapshot_id: self.structural_snapshot.snapshot_id(),
            node_id,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::dependency::{
        DependencyDescriptor, DependencyDescriptorKind, InvalidationReasonKind, InvalidationSeed,
    };
    use crate::recalc::{OverlayKey, OverlayKind};
    use crate::structural::{StructuralNode, StructuralNodeKind, StructuralSnapshotId};

    use super::*;

    fn node(
        node_id: u64,
        kind: StructuralNodeKind,
        symbol: &str,
        parent_id: Option<u64>,
        child_ids: &[u64],
    ) -> StructuralNode {
        StructuralNode {
            node_id: TreeNodeId(node_id),
            kind,
            symbol: symbol.to_string(),
            parent_id: parent_id.map(TreeNodeId),
            child_ids: child_ids.iter().copied().map(TreeNodeId).collect(),
            formula_artifact_id: None,
            bind_artifact_id: None,
            constant_value: None,
        }
    }

    fn snapshot() -> StructuralSnapshot {
        StructuralSnapshot::create(
            StructuralSnapshotId(1),
            TreeNodeId(1),
            [
                node(1, StructuralNodeKind::Root, "Root", None, &[2, 3]),
                node(2, StructuralNodeKind::Calculation, "A", Some(1), &[]),
                node(3, StructuralNodeKind::Calculation, "B", Some(1), &[]),
            ],
        )
        .unwrap()
    }

    fn formula_slot(node_id: u64, formula_id: &str, text: &str) -> FormulaSlotRecord {
        FormulaSlotRecord {
            owner_node_id: TreeNodeId(node_id),
            formula_artifact_id: FormulaArtifactId(formula_id.to_string()),
            bind_artifact_id: Some(BindArtifactId(format!("bind:{formula_id}"))),
            source_identity: FormulaSourceIdentity {
                formula_stable_id: format!("slot:{node_id}"),
                formula_text_version: 1,
                formula_token: Some(format!("token:{formula_id}")),
            },
            opaque_source_text: text.to_string(),
        }
    }

    #[test]
    fn repository_tracks_formula_slots_artifact_handles_and_graph() {
        let mut repository = CalculationRepository::new(snapshot());
        repository
            .upsert_formula_slot(TreeNodeId(2), formula_slot(2, "formula:a", "=B"))
            .unwrap();
        repository
            .record_oxfml_artifact_handle(OxfmlArtifactHandle {
                formula_artifact_id: FormulaArtifactId("formula:a".to_string()),
                kind: OxfmlArtifactKind::SemanticPlan,
                artifact_key: "semantic-plan:a:v1".to_string(),
                structure_context_version: "snapshot:1".to_string(),
                profile_version: "profile:test".to_string(),
            })
            .unwrap();

        repository.rebuild_dependency_graph(&[DependencyDescriptor {
            descriptor_id: "dep:a:b".to_string(),
            source_reference_handle: None,
            owner_node_id: TreeNodeId(2),
            target_node_id: Some(TreeNodeId(3)),
            kind: DependencyDescriptorKind::StaticDirect,
            carrier_detail: "formal_ref:B".to_string(),
            requires_rebind_on_structural_change: false,
        }]);

        assert_eq!(
            repository.formula_slots()[&TreeNodeId(2)]
                .source_identity
                .formula_stable_id,
            "slot:2"
        );
        assert_eq!(
            repository.oxfml_artifact_handles()[&(
                FormulaArtifactId("formula:a".to_string()),
                OxfmlArtifactKind::SemanticPlan
            )]
                .artifact_key,
            "semantic-plan:a:v1"
        );
        assert_eq!(
            repository.dependency_graph().reverse_edges[&TreeNodeId(3)][0].owner_node_id,
            TreeNodeId(2)
        );
        assert_eq!(
            repository.node_states()[&TreeNodeId(2)],
            NodeCalcState::DirtyPending
        );
    }

    #[test]
    fn repository_applies_invalidation_closure_to_node_states() {
        let mut repository = CalculationRepository::new(snapshot());
        repository.rebuild_dependency_graph(&[DependencyDescriptor {
            descriptor_id: "dep:a:b".to_string(),
            source_reference_handle: None,
            owner_node_id: TreeNodeId(2),
            target_node_id: Some(TreeNodeId(3)),
            kind: DependencyDescriptorKind::StaticDirect,
            carrier_detail: "formal_ref:B".to_string(),
            requires_rebind_on_structural_change: false,
        }]);

        let closure =
            repository
                .dependency_graph()
                .derive_invalidation_closure(&[InvalidationSeed {
                    node_id: TreeNodeId(3),
                    reason: InvalidationReasonKind::UpstreamPublication,
                }]);
        repository.apply_invalidation_closure(&closure);

        assert_eq!(
            repository.node_states()[&TreeNodeId(3)],
            NodeCalcState::Needed
        );
        assert_eq!(
            repository.node_states()[&TreeNodeId(2)],
            NodeCalcState::DirtyPending
        );
    }

    #[test]
    fn repository_pins_reader_views_and_tracks_overlays() {
        let mut repository = CalculationRepository::new(snapshot());
        repository
            .seed_published_value(TreeNodeId(2), "42")
            .expect("node exists");
        let overlay_key = OverlayKey {
            owner_node_id: TreeNodeId(2),
            overlay_kind: OverlayKind::InvalidationExecutionState,
            structural_snapshot_id: StructuralSnapshotId(1),
            compatibility_basis: "stage1".to_string(),
            payload_identity: None,
        };
        repository.upsert_overlay(OverlayEntry {
            key: overlay_key.clone(),
            is_protected: true,
            is_eviction_eligible: false,
            detail: "dirty_pending".to_string(),
        });

        let view = repository.pin_reader_view("reader:1");

        assert_eq!(view.snapshot_id, StructuralSnapshotId(1));
        assert_eq!(view.published_values[&TreeNodeId(2)], "42");
        assert!(repository.overlays().contains_key(&overlay_key));
        assert!(repository.unpin_reader_view("reader:1"));
        assert!(repository.pinned_reader_views().is_empty());
    }
}
