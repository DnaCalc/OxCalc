#![forbid(unsafe_code)]

//! Calculation Repository substrate for W050 session-shaped recalc.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::dependency::{
    DependencyDescriptor, DependencyGraph, InvalidationClosure, InvalidationReasonKind,
    InvalidationSeed,
};
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SubscriptionTopicId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SubscriptionHandle(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubscriptionRegistryEntry {
    pub topic_id: SubscriptionTopicId,
    pub formula_stable_id: String,
    pub subscription_handle: SubscriptionHandle,
    pub topic_descriptor: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SubscriptionLifecycleAction {
    Created,
    Released,
    Replaced,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SubscriptionLifecycleReason {
    PreparedRuntimeEffect,
    FormulaTextChanged,
    NameWorldChanged,
    StructureContextChanged,
    FormulaRemoved,
    PreparedCallableReplaced,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubscriptionLifecycleDiagnostic {
    pub action: SubscriptionLifecycleAction,
    pub reason: SubscriptionLifecycleReason,
    pub topic_id: SubscriptionTopicId,
    pub formula_stable_id: String,
    pub subscription_handle: SubscriptionHandle,
    pub topic_descriptor: String,
}

impl SubscriptionLifecycleDiagnostic {
    #[must_use]
    pub fn replay_detail(&self) -> String {
        format!(
            "subscription_lifecycle:{:?}:{:?}:formula={}:topic={}:handle={}:descriptor={}",
            self.action,
            self.reason,
            self.formula_stable_id,
            self.topic_id.0,
            self.subscription_handle.0,
            self.topic_descriptor
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TopicEnvelope {
    pub topic_id: SubscriptionTopicId,
    pub topic_sequence: u64,
    pub last_observed_payload_ref: String,
    pub ordering_key: String,
    pub dedupe_identity: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TopicEnvelopeUpdate {
    pub topic_id: SubscriptionTopicId,
    pub topic_sequence: u64,
    pub payload_ref: String,
    pub ordering_key: String,
    pub dedupe_identity: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ExternalInvalidationDirtySeed {
    pub topic_id: SubscriptionTopicId,
    pub topic_sequence: u64,
    pub formula_stable_id: String,
    pub node_id: TreeNodeId,
}

impl ExternalInvalidationDirtySeed {
    #[must_use]
    pub fn invalidation_seed(&self) -> InvalidationSeed {
        InvalidationSeed {
            node_id: self.node_id,
            reason: InvalidationReasonKind::ExternallyInvalidated,
        }
    }
}

impl TopicEnvelope {
    fn from_update(update: TopicEnvelopeUpdate) -> Self {
        Self {
            topic_id: update.topic_id,
            topic_sequence: update.topic_sequence,
            last_observed_payload_ref: update.payload_ref,
            ordering_key: update.ordering_key,
            dedupe_identity: update.dedupe_identity,
        }
    }
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
    #[error("formula stable id {formula_stable_id} is not registered in the repository")]
    UnknownFormulaStableId { formula_stable_id: String },
    #[error(
        "subscription formula stable id {entry_formula_stable_id} does not match requested formula stable id {formula_stable_id}"
    )]
    SubscriptionFormulaStableIdMismatch {
        formula_stable_id: String,
        entry_formula_stable_id: String,
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
    subscription_registry: BTreeMap<(SubscriptionTopicId, String), SubscriptionRegistryEntry>,
    topic_envelopes: BTreeMap<SubscriptionTopicId, TopicEnvelope>,
    topic_envelope_dedupe_identities: BTreeSet<String>,
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
            subscription_registry: BTreeMap::new(),
            topic_envelopes: BTreeMap::new(),
            topic_envelope_dedupe_identities: BTreeSet::new(),
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

    #[must_use]
    pub fn subscription_registry(
        &self,
    ) -> &BTreeMap<(SubscriptionTopicId, String), SubscriptionRegistryEntry> {
        &self.subscription_registry
    }

    #[must_use]
    pub fn topic_envelopes(&self) -> &BTreeMap<SubscriptionTopicId, TopicEnvelope> {
        &self.topic_envelopes
    }

    #[must_use]
    pub fn topic_envelope(&self, topic_id: &SubscriptionTopicId) -> Option<&TopicEnvelope> {
        self.topic_envelopes.get(topic_id)
    }

    #[must_use]
    pub fn topic_envelope_dedupe_identities(&self) -> &BTreeSet<String> {
        &self.topic_envelope_dedupe_identities
    }

    #[must_use]
    pub fn subscriptions_for_topic(
        &self,
        topic_id: &SubscriptionTopicId,
    ) -> Vec<&SubscriptionRegistryEntry> {
        self.subscription_registry
            .iter()
            .filter_map(|((candidate_topic_id, _), entry)| {
                (candidate_topic_id == topic_id).then_some(entry)
            })
            .collect()
    }

    #[must_use]
    pub fn subscriptions_for_formula(
        &self,
        formula_stable_id: &str,
    ) -> Vec<&SubscriptionRegistryEntry> {
        self.subscription_registry
            .iter()
            .filter_map(|((_, candidate_formula_stable_id), entry)| {
                (candidate_formula_stable_id == formula_stable_id).then_some(entry)
            })
            .collect()
    }

    pub fn upsert_formula_slot(
        &mut self,
        owner_node_id: TreeNodeId,
        record: FormulaSlotRecord,
    ) -> Result<(), CalculationRepositoryError> {
        self.upsert_formula_slot_with_lifecycle_diagnostics(owner_node_id, record)
            .map(|_| ())
    }

    pub fn upsert_formula_slot_with_lifecycle_diagnostics(
        &mut self,
        owner_node_id: TreeNodeId,
        record: FormulaSlotRecord,
    ) -> Result<Vec<SubscriptionLifecycleDiagnostic>, CalculationRepositoryError> {
        self.require_node(owner_node_id)?;
        if record.owner_node_id != owner_node_id {
            return Err(CalculationRepositoryError::FormulaSlotOwnerMismatch {
                node_id: owner_node_id,
                record_owner_node_id: record.owner_node_id,
            });
        }

        let invalidated_formula = self.formula_slots.get(&owner_node_id).and_then(|existing| {
            formula_slot_subscription_invalidation_reason(existing, &record)
                .map(|reason| (existing.source_identity.formula_stable_id.clone(), reason))
        });
        let diagnostics = if let Some((formula_stable_id, reason)) = invalidated_formula {
            self.release_subscriptions_for_formula_stable_id_with_reason(&formula_stable_id, reason)
        } else {
            Vec::new()
        };

        self.formula_slots.insert(owner_node_id, record);
        self.node_states
            .insert(owner_node_id, NodeCalcState::DirtyPending);
        Ok(diagnostics)
    }

    pub fn remove_formula_slot(
        &mut self,
        owner_node_id: TreeNodeId,
    ) -> Result<Option<FormulaSlotRecord>, CalculationRepositoryError> {
        self.remove_formula_slot_with_lifecycle_diagnostics(owner_node_id)
            .map(|(removed, _)| removed)
    }

    pub fn remove_formula_slot_with_lifecycle_diagnostics(
        &mut self,
        owner_node_id: TreeNodeId,
    ) -> Result<
        (
            Option<FormulaSlotRecord>,
            Vec<SubscriptionLifecycleDiagnostic>,
        ),
        CalculationRepositoryError,
    > {
        self.require_node(owner_node_id)?;
        let removed = self.formula_slots.remove(&owner_node_id);
        let diagnostics = if let Some(record) = &removed {
            let diagnostics = self.release_subscriptions_for_formula_stable_id_with_reason(
                &record.source_identity.formula_stable_id,
                SubscriptionLifecycleReason::FormulaRemoved,
            );
            self.node_states
                .insert(owner_node_id, NodeCalcState::DirtyPending);
            diagnostics
        } else {
            Vec::new()
        };
        Ok((removed, diagnostics))
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

    pub fn register_subscription(
        &mut self,
        entry: SubscriptionRegistryEntry,
    ) -> Result<(), CalculationRepositoryError> {
        if !self.has_formula_stable_id(&entry.formula_stable_id) {
            return Err(CalculationRepositoryError::UnknownFormulaStableId {
                formula_stable_id: entry.formula_stable_id,
            });
        }

        self.subscription_registry.insert(
            (entry.topic_id.clone(), entry.formula_stable_id.clone()),
            entry,
        );
        Ok(())
    }

    pub fn reconcile_subscriptions_for_formula(
        &mut self,
        formula_stable_id: &str,
        entries: impl IntoIterator<Item = SubscriptionRegistryEntry>,
        reason: SubscriptionLifecycleReason,
    ) -> Result<Vec<SubscriptionLifecycleDiagnostic>, CalculationRepositoryError> {
        if !self.has_formula_stable_id(formula_stable_id) {
            return Err(CalculationRepositoryError::UnknownFormulaStableId {
                formula_stable_id: formula_stable_id.to_string(),
            });
        }

        let mut desired = BTreeMap::new();
        for entry in entries {
            if entry.formula_stable_id != formula_stable_id {
                return Err(
                    CalculationRepositoryError::SubscriptionFormulaStableIdMismatch {
                        formula_stable_id: formula_stable_id.to_string(),
                        entry_formula_stable_id: entry.formula_stable_id,
                    },
                );
            }
            desired.insert(
                (entry.topic_id.clone(), entry.formula_stable_id.clone()),
                entry,
            );
        }

        let mut diagnostics = Vec::new();
        for key in self.subscription_keys_for_formula(formula_stable_id) {
            if !desired.contains_key(&key)
                && let Some(entry) = self.subscription_registry.remove(&key)
            {
                diagnostics.push(subscription_lifecycle_diagnostic(
                    SubscriptionLifecycleAction::Released,
                    reason,
                    &entry,
                ));
            }
        }

        for (key, entry) in desired {
            match self.subscription_registry.get(&key).cloned() {
                Some(existing) if existing == entry => {}
                Some(_) => {
                    self.subscription_registry.insert(key, entry.clone());
                    diagnostics.push(subscription_lifecycle_diagnostic(
                        SubscriptionLifecycleAction::Replaced,
                        reason,
                        &entry,
                    ));
                }
                None => {
                    self.subscription_registry.insert(key, entry.clone());
                    diagnostics.push(subscription_lifecycle_diagnostic(
                        SubscriptionLifecycleAction::Created,
                        reason,
                        &entry,
                    ));
                }
            }
        }

        Ok(diagnostics)
    }

    pub fn release_subscription(
        &mut self,
        topic_id: &SubscriptionTopicId,
        formula_stable_id: &str,
    ) -> Option<SubscriptionRegistryEntry> {
        self.subscription_registry
            .remove(&(topic_id.clone(), formula_stable_id.to_string()))
    }

    pub fn release_subscriptions_for_formula_stable_id(
        &mut self,
        formula_stable_id: &str,
    ) -> Vec<SubscriptionRegistryEntry> {
        self.subscription_keys_for_formula(formula_stable_id)
            .into_iter()
            .filter_map(|key| self.subscription_registry.remove(&key))
            .collect()
    }

    pub fn release_subscriptions_for_formula_stable_id_with_reason(
        &mut self,
        formula_stable_id: &str,
        reason: SubscriptionLifecycleReason,
    ) -> Vec<SubscriptionLifecycleDiagnostic> {
        self.release_subscriptions_for_formula_stable_id(formula_stable_id)
            .into_iter()
            .map(|entry| {
                subscription_lifecycle_diagnostic(
                    SubscriptionLifecycleAction::Released,
                    reason,
                    &entry,
                )
            })
            .collect()
    }

    #[must_use]
    pub fn external_invalidation_dirty_seeds(
        &self,
        topic_id: &SubscriptionTopicId,
        topic_sequence: u64,
    ) -> Vec<ExternalInvalidationDirtySeed> {
        let mut dirty_seeds = self
            .subscriptions_for_topic(topic_id)
            .into_iter()
            .filter_map(|entry| {
                self.node_id_for_formula_stable_id(&entry.formula_stable_id)
                    .map(|node_id| ExternalInvalidationDirtySeed {
                        topic_id: topic_id.clone(),
                        topic_sequence,
                        formula_stable_id: entry.formula_stable_id.clone(),
                        node_id,
                    })
            })
            .collect::<Vec<_>>();
        dirty_seeds.sort();
        dirty_seeds
    }

    #[must_use]
    pub fn derive_external_invalidation_closure(
        &self,
        dirty_seeds: &[ExternalInvalidationDirtySeed],
    ) -> InvalidationClosure {
        let seeds = dirty_seeds
            .iter()
            .map(ExternalInvalidationDirtySeed::invalidation_seed)
            .collect::<Vec<_>>();
        self.dependency_graph.derive_invalidation_closure(&seeds)
    }

    pub fn apply_topic_envelope_update(
        &mut self,
        update: TopicEnvelopeUpdate,
    ) -> Option<TopicEnvelope> {
        if self
            .topic_envelope_dedupe_identities
            .contains(&update.dedupe_identity)
        {
            return None;
        }
        self.topic_envelope_dedupe_identities
            .insert(update.dedupe_identity.clone());

        let should_update = self
            .topic_envelopes
            .get(&update.topic_id)
            .is_none_or(|existing| update.topic_sequence >= existing.topic_sequence);
        if !should_update {
            return None;
        }

        let envelope = TopicEnvelope::from_update(update);
        self.topic_envelopes
            .insert(envelope.topic_id.clone(), envelope.clone());
        Some(envelope)
    }

    pub fn apply_topic_envelope_updates(
        &mut self,
        updates: impl IntoIterator<Item = TopicEnvelopeUpdate>,
    ) -> Vec<TopicEnvelope> {
        let mut updates = updates.into_iter().collect::<Vec<_>>();
        updates.sort_by(|left, right| {
            left.ordering_key
                .cmp(&right.ordering_key)
                .then_with(|| left.topic_id.cmp(&right.topic_id))
                .then_with(|| left.topic_sequence.cmp(&right.topic_sequence))
                .then_with(|| left.dedupe_identity.cmp(&right.dedupe_identity))
                .then_with(|| left.payload_ref.cmp(&right.payload_ref))
        });

        updates
            .into_iter()
            .filter_map(|update| self.apply_topic_envelope_update(update))
            .collect()
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

    fn has_formula_stable_id(&self, formula_stable_id: &str) -> bool {
        self.formula_slots
            .values()
            .any(|slot| slot.source_identity.formula_stable_id == formula_stable_id)
    }

    fn node_id_for_formula_stable_id(&self, formula_stable_id: &str) -> Option<TreeNodeId> {
        self.formula_slots.iter().find_map(|(node_id, slot)| {
            (slot.source_identity.formula_stable_id == formula_stable_id).then_some(*node_id)
        })
    }

    fn subscription_keys_for_formula(
        &self,
        formula_stable_id: &str,
    ) -> Vec<(SubscriptionTopicId, String)> {
        self.subscription_registry
            .keys()
            .filter(|(_, candidate_formula_stable_id)| {
                candidate_formula_stable_id == formula_stable_id
            })
            .cloned()
            .collect()
    }
}

fn formula_slot_subscription_invalidation_reason(
    existing: &FormulaSlotRecord,
    next: &FormulaSlotRecord,
) -> Option<SubscriptionLifecycleReason> {
    if existing.source_identity.formula_stable_id != next.source_identity.formula_stable_id {
        return Some(SubscriptionLifecycleReason::PreparedCallableReplaced);
    }
    if existing.source_identity.formula_text_version != next.source_identity.formula_text_version
        || existing.source_identity.formula_token != next.source_identity.formula_token
        || existing.opaque_source_text != next.opaque_source_text
    {
        return Some(SubscriptionLifecycleReason::FormulaTextChanged);
    }
    if existing.bind_artifact_id != next.bind_artifact_id {
        return Some(SubscriptionLifecycleReason::NameWorldChanged);
    }
    if existing.formula_artifact_id != next.formula_artifact_id {
        return Some(SubscriptionLifecycleReason::PreparedCallableReplaced);
    }
    None
}

fn subscription_lifecycle_diagnostic(
    action: SubscriptionLifecycleAction,
    reason: SubscriptionLifecycleReason,
    entry: &SubscriptionRegistryEntry,
) -> SubscriptionLifecycleDiagnostic {
    SubscriptionLifecycleDiagnostic {
        action,
        reason,
        topic_id: entry.topic_id.clone(),
        formula_stable_id: entry.formula_stable_id.clone(),
        subscription_handle: entry.subscription_handle.clone(),
        topic_descriptor: entry.topic_descriptor.clone(),
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

    fn subscription(topic: &str, stable_id: &str, handle: &str) -> SubscriptionRegistryEntry {
        SubscriptionRegistryEntry {
            topic_id: SubscriptionTopicId(topic.to_string()),
            formula_stable_id: stable_id.to_string(),
            subscription_handle: SubscriptionHandle(handle.to_string()),
            topic_descriptor: format!("rtd:{topic}"),
        }
    }

    fn topic_update(
        topic: &str,
        sequence: u64,
        payload_ref: &str,
        ordering_key: &str,
        dedupe_identity: &str,
    ) -> TopicEnvelopeUpdate {
        TopicEnvelopeUpdate {
            topic_id: SubscriptionTopicId(topic.to_string()),
            topic_sequence: sequence,
            payload_ref: payload_ref.to_string(),
            ordering_key: ordering_key.to_string(),
            dedupe_identity: dedupe_identity.to_string(),
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

    #[test]
    fn repository_subscription_registry_persists_across_waves() {
        let mut repository = CalculationRepository::new(snapshot());
        repository
            .upsert_formula_slot(TreeNodeId(2), formula_slot(2, "formula:a", "=RTD(...)"))
            .unwrap();
        repository
            .register_subscription(subscription("topic:rtd:price", "slot:2", "sub:slot2:price"))
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
        let closure =
            repository
                .dependency_graph()
                .derive_invalidation_closure(&[InvalidationSeed {
                    node_id: TreeNodeId(3),
                    reason: InvalidationReasonKind::UpstreamPublication,
                }]);
        repository.apply_invalidation_closure(&closure);
        repository.pin_reader_view("reader:after-wave");

        let topic_id = SubscriptionTopicId("topic:rtd:price".to_string());
        let subscriptions = repository.subscriptions_for_topic(&topic_id);
        assert_eq!(subscriptions.len(), 1);
        assert_eq!(
            subscriptions[0].subscription_handle,
            SubscriptionHandle("sub:slot2:price".to_string())
        );
        assert_eq!(
            repository.subscriptions_for_formula("slot:2")[0].topic_id,
            topic_id
        );
    }

    #[test]
    fn repository_reconciles_subscription_lifecycle_with_replay_diagnostics() {
        let mut repository = CalculationRepository::new(snapshot());
        repository
            .upsert_formula_slot(TreeNodeId(2), formula_slot(2, "formula:a", "=RTD(...)"))
            .unwrap();

        let created = repository
            .reconcile_subscriptions_for_formula(
                "slot:2",
                [subscription("topic:rtd:price", "slot:2", "sub:price:v1")],
                SubscriptionLifecycleReason::PreparedRuntimeEffect,
            )
            .unwrap();

        assert_eq!(created.len(), 1);
        assert_eq!(created[0].action, SubscriptionLifecycleAction::Created);
        assert_eq!(
            created[0].reason,
            SubscriptionLifecycleReason::PreparedRuntimeEffect
        );
        assert_eq!(
            created[0].replay_detail(),
            "subscription_lifecycle:Created:PreparedRuntimeEffect:formula=slot:2:topic=topic:rtd:price:handle=sub:price:v1:descriptor=rtd:topic:rtd:price"
        );

        let unchanged = repository
            .reconcile_subscriptions_for_formula(
                "slot:2",
                [subscription("topic:rtd:price", "slot:2", "sub:price:v1")],
                SubscriptionLifecycleReason::PreparedRuntimeEffect,
            )
            .unwrap();
        assert!(unchanged.is_empty());

        let updated = repository
            .reconcile_subscriptions_for_formula(
                "slot:2",
                [
                    subscription("topic:rtd:price", "slot:2", "sub:price:v2"),
                    subscription("topic:rtd:status", "slot:2", "sub:status"),
                ],
                SubscriptionLifecycleReason::StructureContextChanged,
            )
            .unwrap();

        assert_eq!(
            updated
                .iter()
                .map(|diagnostic| diagnostic.action)
                .collect::<Vec<_>>(),
            vec![
                SubscriptionLifecycleAction::Replaced,
                SubscriptionLifecycleAction::Created,
            ]
        );
        assert!(updated.iter().all(|diagnostic| {
            diagnostic.reason == SubscriptionLifecycleReason::StructureContextChanged
        }));

        let serialized = serde_json::to_value(&updated[0]).unwrap();
        assert_eq!(serialized["action"], "Replaced");
        assert_eq!(serialized["reason"], "StructureContextChanged");
        assert_eq!(serialized["topic_id"], "topic:rtd:price");
        assert_eq!(serialized["formula_stable_id"], "slot:2");
        assert_eq!(serialized["subscription_handle"], "sub:price:v2");

        let released = repository
            .reconcile_subscriptions_for_formula(
                "slot:2",
                [subscription("topic:rtd:status", "slot:2", "sub:status")],
                SubscriptionLifecycleReason::FormulaTextChanged,
            )
            .unwrap();

        assert_eq!(released.len(), 1);
        assert_eq!(released[0].action, SubscriptionLifecycleAction::Released);
        assert_eq!(
            released[0].reason,
            SubscriptionLifecycleReason::FormulaTextChanged
        );
        assert_eq!(
            repository.subscriptions_for_formula("slot:2")[0].topic_id,
            SubscriptionTopicId("topic:rtd:status".to_string())
        );
    }

    #[test]
    fn repository_releases_subscriptions_on_callable_invalidation() {
        let mut repository = CalculationRepository::new(snapshot());
        repository
            .upsert_formula_slot(TreeNodeId(2), formula_slot(2, "formula:a", "=RTD(...)"))
            .unwrap();
        repository
            .register_subscription(subscription("topic:rtd:price", "slot:2", "sub:price"))
            .unwrap();
        repository
            .register_subscription(subscription("topic:rtd:status", "slot:2", "sub:status"))
            .unwrap();

        let mut updated = formula_slot(2, "formula:a", "=RTD(...)+1");
        updated.source_identity.formula_text_version = 2;
        let text_release_diagnostics = repository
            .upsert_formula_slot_with_lifecycle_diagnostics(TreeNodeId(2), updated)
            .unwrap();

        assert_eq!(text_release_diagnostics.len(), 2);
        assert!(text_release_diagnostics.iter().all(|diagnostic| {
            diagnostic.action == SubscriptionLifecycleAction::Released
                && diagnostic.reason == SubscriptionLifecycleReason::FormulaTextChanged
        }));
        assert!(repository.subscriptions_for_formula("slot:2").is_empty());

        repository
            .register_subscription(subscription("topic:rtd:price", "slot:2", "sub:price:v2"))
            .unwrap();
        let mut rebound = formula_slot(2, "formula:a", "=RTD(...)+1");
        rebound.source_identity.formula_text_version = 2;
        rebound.bind_artifact_id = Some(BindArtifactId("bind:changed-name-world".to_string()));
        let name_world_release_diagnostics = repository
            .upsert_formula_slot_with_lifecycle_diagnostics(TreeNodeId(2), rebound)
            .unwrap();

        assert_eq!(name_world_release_diagnostics.len(), 1);
        assert_eq!(
            name_world_release_diagnostics[0].reason,
            SubscriptionLifecycleReason::NameWorldChanged
        );
        assert!(repository.subscriptions_for_formula("slot:2").is_empty());

        repository
            .register_subscription(subscription("topic:rtd:price", "slot:2", "sub:price:v3"))
            .unwrap();
        let (removed, remove_diagnostics) = repository
            .remove_formula_slot_with_lifecycle_diagnostics(TreeNodeId(2))
            .unwrap();

        assert!(removed.is_some());
        assert_eq!(remove_diagnostics.len(), 1);
        assert_eq!(
            remove_diagnostics[0].reason,
            SubscriptionLifecycleReason::FormulaRemoved
        );
        assert!(repository.subscriptions_for_formula("slot:2").is_empty());
    }

    #[test]
    fn repository_topic_envelope_serializes_replay_schema() {
        let envelope = TopicEnvelope {
            topic_id: SubscriptionTopicId("topic:rtd:price".to_string()),
            topic_sequence: 42,
            last_observed_payload_ref: "payload:rtd:price:42".to_string(),
            ordering_key: "wave:7/topic:price/sequence:42".to_string(),
            dedupe_identity: "event:rtd:price:42".to_string(),
        };

        let json = serde_json::to_value(&envelope).unwrap();
        let object = json.as_object().expect("envelope serializes as object");
        assert_eq!(object.len(), 5);
        assert_eq!(json["topic_id"], "topic:rtd:price");
        assert_eq!(json["topic_sequence"], 42);
        assert_eq!(json["last_observed_payload_ref"], "payload:rtd:price:42");
        assert_eq!(json["ordering_key"], "wave:7/topic:price/sequence:42");
        assert_eq!(json["dedupe_identity"], "event:rtd:price:42");

        let round_trip: TopicEnvelope = serde_json::from_value(json).unwrap();
        assert_eq!(round_trip, envelope);

        let update = topic_update(
            "topic:rtd:price",
            43,
            "payload:rtd:price:43",
            "wave:7/topic:price/sequence:43",
            "event:rtd:price:43",
        );
        let update_json = serde_json::to_value(&update).unwrap();
        assert_eq!(update_json["topic_id"], "topic:rtd:price");
        assert_eq!(update_json["payload_ref"], "payload:rtd:price:43");
        let update_round_trip: TopicEnvelopeUpdate = serde_json::from_value(update_json).unwrap();
        assert_eq!(update_round_trip, update);
    }

    #[test]
    fn repository_topic_envelope_updates_are_deterministically_ordered_and_deduped() {
        let updates = vec![
            topic_update(
                "topic:rtd:status",
                1,
                "payload:rtd:status:1",
                "002",
                "event:rtd:status:1",
            ),
            topic_update(
                "topic:rtd:price",
                2,
                "payload:rtd:price:2",
                "003",
                "event:rtd:price:2",
            ),
            topic_update(
                "topic:rtd:price",
                1,
                "payload:rtd:price:1",
                "001",
                "event:rtd:price:1",
            ),
            topic_update(
                "topic:rtd:price",
                2,
                "payload:rtd:price:duplicate",
                "004",
                "event:rtd:price:2",
            ),
            topic_update(
                "topic:rtd:status",
                0,
                "payload:rtd:status:0",
                "005",
                "event:rtd:status:0-late",
            ),
        ];

        let mut forward_repository = CalculationRepository::new(snapshot());
        let mut reverse_repository = CalculationRepository::new(snapshot());

        let forward_applied = forward_repository.apply_topic_envelope_updates(updates.clone());
        let reverse_applied =
            reverse_repository.apply_topic_envelope_updates(updates.into_iter().rev());

        assert_eq!(forward_applied, reverse_applied);
        assert_eq!(
            forward_repository.topic_envelopes(),
            reverse_repository.topic_envelopes()
        );
        assert_eq!(forward_applied.len(), 3);
        assert_eq!(
            forward_applied
                .iter()
                .map(|envelope| envelope.dedupe_identity.as_str())
                .collect::<Vec<_>>(),
            vec![
                "event:rtd:price:1",
                "event:rtd:status:1",
                "event:rtd:price:2",
            ]
        );

        let price_topic = SubscriptionTopicId("topic:rtd:price".to_string());
        let price_envelope = forward_repository
            .topic_envelope(&price_topic)
            .expect("price envelope exists");
        assert_eq!(price_envelope.topic_sequence, 2);
        assert_eq!(
            price_envelope.last_observed_payload_ref,
            "payload:rtd:price:2"
        );
        assert_eq!(price_envelope.dedupe_identity, "event:rtd:price:2");

        let status_topic = SubscriptionTopicId("topic:rtd:status".to_string());
        let status_envelope = forward_repository
            .topic_envelope(&status_topic)
            .expect("status envelope exists");
        assert_eq!(status_envelope.topic_sequence, 1);
        assert_eq!(
            status_envelope.last_observed_payload_ref,
            "payload:rtd:status:1"
        );
        assert!(
            forward_repository
                .topic_envelope_dedupe_identities()
                .contains("event:rtd:status:0-late")
        );
    }
}
