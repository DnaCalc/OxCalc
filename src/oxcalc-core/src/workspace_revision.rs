#![forbid(unsafe_code)]

//! Workspace revision and snapshot-layer identity types.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::coordinator::{PublicationBundle, RuntimeEffect, calc_value_display_text};
use crate::dependency::{DependencyGraph, InvalidationReasonKind};
use crate::recalc::OverlayEntry;
use crate::structural::{StructuralSnapshot, StructuralSnapshotId, TreeNodeId};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct WorkspaceRevisionId(pub String);

impl Display for WorkspaceRevisionId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct NodeInputSnapshotId(pub String);

impl Display for NodeInputSnapshotId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct NamespaceSnapshotId(pub String);

impl Display for NamespaceSnapshotId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct FormulaBindingSnapshotId(pub String);

impl Display for FormulaBindingSnapshotId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DependencyShapeSnapshotId(pub String);

impl Display for DependencyShapeSnapshotId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PublicationSnapshotId(pub String);

impl Display for PublicationSnapshotId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct RuntimeOverlaySetId(pub String);

impl Display for RuntimeOverlaySetId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum NodeInputKind {
    Empty,
    Literal,
    FormulaText,
    HostOwned,
}

impl NodeInputKind {
    #[must_use]
    pub fn as_identity_token(self) -> &'static str {
        match self {
            Self::Empty => "empty",
            Self::Literal => "literal",
            Self::FormulaText => "formula-text",
            Self::HostOwned => "host-owned",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeInputRecord {
    pub node_id: TreeNodeId,
    pub kind: NodeInputKind,
    pub text: Option<String>,
    pub input_epoch: u64,
}

impl NodeInputRecord {
    #[must_use]
    pub fn empty(node_id: TreeNodeId, input_epoch: u64) -> Self {
        Self {
            node_id,
            kind: NodeInputKind::Empty,
            text: None,
            input_epoch,
        }
    }

    #[must_use]
    pub fn literal(node_id: TreeNodeId, literal_text: impl Into<String>, input_epoch: u64) -> Self {
        Self {
            node_id,
            kind: NodeInputKind::Literal,
            text: Some(literal_text.into()),
            input_epoch,
        }
    }

    #[must_use]
    pub fn formula_text(
        node_id: TreeNodeId,
        formula_text: impl Into<String>,
        input_epoch: u64,
    ) -> Self {
        Self {
            node_id,
            kind: NodeInputKind::FormulaText,
            text: Some(formula_text.into()),
            input_epoch,
        }
    }

    #[must_use]
    pub fn host_owned(
        node_id: TreeNodeId,
        host_input_identity: impl Into<String>,
        input_epoch: u64,
    ) -> Self {
        Self {
            node_id,
            kind: NodeInputKind::HostOwned,
            text: Some(host_input_identity.into()),
            input_epoch,
        }
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum WorkspaceRevisionError {
    #[error("duplicate node-input record for {node_id}")]
    DuplicateNodeInputRecord { node_id: TreeNodeId },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeInputSnapshot {
    snapshot_id: NodeInputSnapshotId,
    records: BTreeMap<TreeNodeId, NodeInputRecord>,
}

impl NodeInputSnapshot {
    pub fn create(
        records: impl IntoIterator<Item = NodeInputRecord>,
    ) -> Result<Self, WorkspaceRevisionError> {
        let mut record_map = BTreeMap::new();
        for record in records {
            let node_id = record.node_id;
            if record_map.insert(node_id, record).is_some() {
                return Err(WorkspaceRevisionError::DuplicateNodeInputRecord { node_id });
            }
        }
        Ok(Self::from_record_map(record_map))
    }

    #[must_use]
    pub fn from_record_map(records: BTreeMap<TreeNodeId, NodeInputRecord>) -> Self {
        let snapshot_id = NodeInputSnapshotId(node_input_snapshot_identity(&records));
        Self {
            snapshot_id,
            records,
        }
    }

    #[must_use]
    pub fn snapshot_id(&self) -> &NodeInputSnapshotId {
        &self.snapshot_id
    }

    #[must_use]
    pub fn records(&self) -> &BTreeMap<TreeNodeId, NodeInputRecord> {
        &self.records
    }

    #[must_use]
    pub fn try_get_record(&self, node_id: TreeNodeId) -> Option<&NodeInputRecord> {
        self.records.get(&node_id)
    }

    #[must_use]
    pub fn with_record(&self, record: NodeInputRecord) -> Self {
        let mut records = self.records.clone();
        records.insert(record.node_id, record);
        Self::from_record_map(records)
    }

    #[must_use]
    pub fn without_record(&self, node_id: TreeNodeId) -> Self {
        let mut records = self.records.clone();
        records.remove(&node_id);
        Self::from_record_map(records)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NamespaceSnapshot {
    snapshot_id: NamespaceSnapshotId,
    pub host_namespace_version: String,
    pub function_registry_version: String,
    pub capability_profile_id: String,
    pub resolution_rule_version: String,
    pub caller_context_identity_version: String,
    pub workspace_availability_version: Option<String>,
    pub workspace_alias_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub meta_node_membership_version: Option<String>,
}

impl NamespaceSnapshot {
    #[must_use]
    pub fn new(
        host_namespace_version: impl Into<String>,
        function_registry_version: impl Into<String>,
        capability_profile_id: impl Into<String>,
        resolution_rule_version: impl Into<String>,
        caller_context_identity_version: impl Into<String>,
        workspace_availability_version: Option<String>,
        workspace_alias_version: Option<String>,
    ) -> Self {
        let snapshot = Self {
            snapshot_id: NamespaceSnapshotId(String::new()),
            host_namespace_version: host_namespace_version.into(),
            function_registry_version: function_registry_version.into(),
            capability_profile_id: capability_profile_id.into(),
            resolution_rule_version: resolution_rule_version.into(),
            caller_context_identity_version: caller_context_identity_version.into(),
            workspace_availability_version,
            workspace_alias_version,
            meta_node_membership_version: None,
        };
        snapshot.with_computed_identity()
    }

    #[must_use]
    pub fn current_absent() -> Self {
        Self::new(
            "namespace:absent",
            "function-registry:absent",
            "capability-profile:absent",
            "resolution-rule:absent",
            "caller-context:absent",
            None,
            None,
        )
    }

    #[must_use]
    pub fn snapshot_id(&self) -> &NamespaceSnapshotId {
        &self.snapshot_id
    }

    #[must_use]
    pub fn with_meta_node_ids(mut self, meta_node_ids: &BTreeSet<TreeNodeId>) -> Self {
        self.meta_node_membership_version = (!meta_node_ids.is_empty()).then(|| {
            format!(
                "meta-node-membership:v1:{}",
                meta_node_ids
                    .iter()
                    .map(|node_id| node_id.0.to_string())
                    .collect::<Vec<_>>()
                    .join(",")
            )
        });
        self.with_computed_identity()
    }

    fn with_computed_identity(mut self) -> Self {
        self.snapshot_id = NamespaceSnapshotId(identity(
            "namespace-snapshot",
            [
                field("host_namespace_version", &self.host_namespace_version),
                field("function_registry_version", &self.function_registry_version),
                field("capability_profile_id", &self.capability_profile_id),
                field("resolution_rule_version", &self.resolution_rule_version),
                field(
                    "caller_context_identity_version",
                    &self.caller_context_identity_version,
                ),
                optional_field(
                    "workspace_availability_version",
                    self.workspace_availability_version.as_deref(),
                ),
                optional_field(
                    "workspace_alias_version",
                    self.workspace_alias_version.as_deref(),
                ),
                optional_field(
                    "meta_node_membership_version",
                    self.meta_node_membership_version.as_deref(),
                ),
            ],
        ));
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceRevision {
    revision_id: WorkspaceRevisionId,
    pub workspace_id: String,
    pub structure_snapshot: StructuralSnapshot,
    pub node_input_snapshot: NodeInputSnapshot,
    pub namespace_snapshot: NamespaceSnapshot,
}

impl WorkspaceRevision {
    #[must_use]
    pub fn new(
        workspace_id: impl Into<String>,
        structure_snapshot: StructuralSnapshot,
        node_input_snapshot: NodeInputSnapshot,
        namespace_snapshot: NamespaceSnapshot,
    ) -> Self {
        let workspace_id = workspace_id.into();
        let revision_id = WorkspaceRevisionId(workspace_revision_identity(
            &workspace_id,
            structure_snapshot.snapshot_id(),
            node_input_snapshot.snapshot_id(),
            namespace_snapshot.snapshot_id(),
        ));
        Self {
            revision_id,
            workspace_id,
            structure_snapshot,
            node_input_snapshot,
            namespace_snapshot,
        }
    }

    #[must_use]
    pub fn revision_id(&self) -> &WorkspaceRevisionId {
        &self.revision_id
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceRevisionInvalidationSummaryEntry {
    pub node_id: TreeNodeId,
    pub requires_rebind: bool,
    pub reasons: Vec<InvalidationReasonKind>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceRevisionTransactionSummary {
    pub transaction_id: String,
    pub invalidated_nodes: Vec<WorkspaceRevisionInvalidationSummaryEntry>,
    pub requires_rebind: Vec<TreeNodeId>,
    pub estimated_node_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceRevisionGraphEntry {
    pub revision_id: WorkspaceRevisionId,
    pub parent_revision_id: Option<WorkspaceRevisionId>,
    pub structure_snapshot_id: StructuralSnapshotId,
    pub node_input_snapshot_id: NodeInputSnapshotId,
    pub namespace_snapshot_id: NamespaceSnapshotId,
    pub transaction_id: Option<String>,
    pub transaction_summary: Option<WorkspaceRevisionTransactionSummary>,
}

impl WorkspaceRevisionGraphEntry {
    #[must_use]
    pub fn from_revision(
        revision: &WorkspaceRevision,
        parent_revision_id: Option<WorkspaceRevisionId>,
        transaction_id: Option<String>,
        transaction_summary: Option<WorkspaceRevisionTransactionSummary>,
    ) -> Self {
        let transaction_id = transaction_id.or_else(|| {
            transaction_summary
                .as_ref()
                .map(|summary| summary.transaction_id.clone())
        });
        Self {
            revision_id: revision.revision_id().clone(),
            parent_revision_id,
            structure_snapshot_id: revision.structure_snapshot.snapshot_id(),
            node_input_snapshot_id: revision.node_input_snapshot.snapshot_id().clone(),
            namespace_snapshot_id: revision.namespace_snapshot.snapshot_id().clone(),
            transaction_id,
            transaction_summary,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceRevisionGraph {
    current_revision_id: WorkspaceRevisionId,
    entries: BTreeMap<WorkspaceRevisionId, WorkspaceRevisionGraphEntry>,
}

impl WorkspaceRevisionGraph {
    #[must_use]
    pub fn initial(revision: &WorkspaceRevision) -> Self {
        let entry = WorkspaceRevisionGraphEntry::from_revision(revision, None, None, None);
        let current_revision_id = entry.revision_id.clone();
        let entries = BTreeMap::from([(entry.revision_id.clone(), entry)]);
        Self {
            current_revision_id,
            entries,
        }
    }

    pub fn record_successor(
        &mut self,
        predecessor_revision_id: &WorkspaceRevisionId,
        successor_revision: &WorkspaceRevision,
        transaction_id: Option<String>,
        transaction_summary: Option<WorkspaceRevisionTransactionSummary>,
    ) {
        let successor_revision_id = successor_revision.revision_id();
        if successor_revision_id == predecessor_revision_id {
            self.current_revision_id = successor_revision_id.clone();
            return;
        }
        self.entries.insert(
            successor_revision_id.clone(),
            WorkspaceRevisionGraphEntry::from_revision(
                successor_revision,
                Some(predecessor_revision_id.clone()),
                transaction_id,
                transaction_summary,
            ),
        );
        self.current_revision_id = successor_revision_id.clone();
    }

    pub fn navigate_to(
        &mut self,
        revision_id: &WorkspaceRevisionId,
    ) -> Result<(), WorkspaceRevisionGraphNavigationError> {
        if self.entries.contains_key(revision_id) {
            self.current_revision_id = revision_id.clone();
            Ok(())
        } else {
            Err(WorkspaceRevisionGraphNavigationError::UnknownRevision {
                revision_id: revision_id.clone(),
            })
        }
    }

    #[must_use]
    pub fn current_revision_id(&self) -> &WorkspaceRevisionId {
        &self.current_revision_id
    }

    #[must_use]
    pub fn current_parent_revision_id(&self) -> Option<&WorkspaceRevisionId> {
        self.entries
            .get(&self.current_revision_id)
            .and_then(|entry| entry.parent_revision_id.as_ref())
    }

    #[must_use]
    pub fn entries(&self) -> &BTreeMap<WorkspaceRevisionId, WorkspaceRevisionGraphEntry> {
        &self.entries
    }

    pub fn set_current_transaction_summary(
        &mut self,
        transaction_summary: WorkspaceRevisionTransactionSummary,
    ) {
        if let Some(entry) = self.entries.get_mut(&self.current_revision_id) {
            entry.transaction_id = Some(transaction_summary.transaction_id.clone());
            entry.transaction_summary = Some(transaction_summary);
        }
    }

    pub fn evict(
        &mut self,
        revision_id: &WorkspaceRevisionId,
    ) -> Result<(), WorkspaceRevisionGraphEvictionError> {
        if revision_id == &self.current_revision_id {
            return Err(WorkspaceRevisionGraphEvictionError::CurrentRevision {
                revision_id: revision_id.clone(),
            });
        }
        self.entries.remove(revision_id);
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum WorkspaceRevisionGraphNavigationError {
    #[error("workspace revision '{revision_id}' is not retained")]
    UnknownRevision { revision_id: WorkspaceRevisionId },
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum WorkspaceRevisionGraphEvictionError {
    #[error("workspace revision '{revision_id}' is current and cannot be evicted")]
    CurrentRevision { revision_id: WorkspaceRevisionId },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SnapshotLayerState {
    CurrentAbsent { reason: String },
    Current { basis: String },
}

impl SnapshotLayerState {
    #[must_use]
    pub fn current_absent(reason: impl Into<String>) -> Self {
        Self::CurrentAbsent {
            reason: reason.into(),
        }
    }

    #[must_use]
    pub fn current(basis: impl Into<String>) -> Self {
        Self::Current {
            basis: basis.into(),
        }
    }

    #[must_use]
    pub fn identity_token(&self) -> String {
        match self {
            Self::CurrentAbsent { reason } => field("absent", reason),
            Self::Current { basis } => field("current", basis),
        }
    }

    /// `identity_token`, with the unbounded `Current` basis collapsed to a
    /// fixed-width digest. Layer snapshot ids are built from this token and
    /// get embedded into the engine's per-run compatibility and cache bases,
    /// which a published run folds back into the next revision's layer bases
    /// — embedding the full basis made each published run's ids grow by the
    /// size of the previous run's entire trace (the warm-recalc memory
    /// explosion). The ids only participate in equality comparisons, and the
    /// full basis remains stored on the layer state itself, so substituting
    /// a deterministic digest preserves id equality for equal bases.
    fn identity_digest_token(&self) -> String {
        match self {
            Self::CurrentAbsent { reason } => field("absent", reason),
            Self::Current { basis } => field("current", &identity_basis_digest(basis)),
        }
    }
}

/// Mints a layer snapshot id from named id fields plus the layer state's
/// digest token. Byte-identical to the eager `identity`/`field` fold; the
/// id fields embed O(records) revision ids, so streaming copies each once
/// instead of four times on every edit.
fn mint_layer_snapshot_id(
    namespace: &str,
    id_fields: &[(&str, &str)],
    state: &SnapshotLayerState,
) -> String {
    let mut value = identity_seed(namespace);
    for (name, id) in id_fields {
        push_identity_field(&mut value, name, id);
    }
    push_identity_prebuilt_field(&mut value, &state.identity_digest_token());
    value
}

/// Collapses an unbounded layer basis to a fixed-width token for use inside
/// layer snapshot ids. Two independently seeded 64-bit lanes give a 128-bit
/// token, making accidental id aliasing across distinct bases negligible.
fn identity_basis_digest(basis: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::Hasher;

    let lane = |seed: &[u8]| {
        let mut hasher = DefaultHasher::new();
        hasher.write(seed);
        hasher.write(basis.as_bytes());
        hasher.finish()
    };
    format!(
        "digest:v1:{:016x}{:016x}",
        lane(b"layer-identity-digest:lane:1"),
        lane(b"layer-identity-digest:lane:2")
    )
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FormulaBindingSnapshot {
    snapshot_id: FormulaBindingSnapshotId,
    pub revision_id: WorkspaceRevisionId,
    pub state: SnapshotLayerState,
}

impl FormulaBindingSnapshot {
    #[must_use]
    pub fn current_absent(revision_id: &WorkspaceRevisionId, reason: impl Into<String>) -> Self {
        let state = SnapshotLayerState::current_absent(reason);
        let snapshot_id = FormulaBindingSnapshotId(mint_layer_snapshot_id(
            "formula-binding-snapshot",
            &[("revision_id", &revision_id.0)],
            &state,
        ));
        Self {
            snapshot_id,
            revision_id: revision_id.clone(),
            state,
        }
    }

    #[must_use]
    pub fn current(revision_id: &WorkspaceRevisionId, basis: impl Into<String>) -> Self {
        let state = SnapshotLayerState::current(basis);
        let snapshot_id = FormulaBindingSnapshotId(mint_layer_snapshot_id(
            "formula-binding-snapshot",
            &[("revision_id", &revision_id.0)],
            &state,
        ));
        Self {
            snapshot_id,
            revision_id: revision_id.clone(),
            state,
        }
    }

    #[must_use]
    pub fn snapshot_id(&self) -> &FormulaBindingSnapshotId {
        &self.snapshot_id
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DependencyShapeSnapshot {
    snapshot_id: DependencyShapeSnapshotId,
    pub revision_id: WorkspaceRevisionId,
    pub formula_binding_snapshot_id: FormulaBindingSnapshotId,
    pub state: SnapshotLayerState,
}

impl DependencyShapeSnapshot {
    #[must_use]
    pub fn current_absent(
        revision_id: &WorkspaceRevisionId,
        formula_binding_snapshot_id: &FormulaBindingSnapshotId,
        reason: impl Into<String>,
    ) -> Self {
        let state = SnapshotLayerState::current_absent(reason);
        let snapshot_id = DependencyShapeSnapshotId(mint_layer_snapshot_id(
            "dependency-shape-snapshot",
            &[
                ("revision_id", &revision_id.0),
                (
                    "formula_binding_snapshot_id",
                    &formula_binding_snapshot_id.0,
                ),
            ],
            &state,
        ));
        Self {
            snapshot_id,
            revision_id: revision_id.clone(),
            formula_binding_snapshot_id: formula_binding_snapshot_id.clone(),
            state,
        }
    }

    #[must_use]
    pub fn from_dependency_graph(
        revision_id: &WorkspaceRevisionId,
        formula_binding_snapshot_id: &FormulaBindingSnapshotId,
        dependency_graph: &DependencyGraph,
    ) -> Self {
        let state = SnapshotLayerState::Current {
            basis: dependency_shape_basis(dependency_graph),
        };
        let snapshot_id = DependencyShapeSnapshotId(mint_layer_snapshot_id(
            "dependency-shape-snapshot",
            &[
                ("revision_id", &revision_id.0),
                (
                    "formula_binding_snapshot_id",
                    &formula_binding_snapshot_id.0,
                ),
            ],
            &state,
        ));
        Self {
            snapshot_id,
            revision_id: revision_id.clone(),
            formula_binding_snapshot_id: formula_binding_snapshot_id.clone(),
            state,
        }
    }

    #[must_use]
    pub fn snapshot_id(&self) -> &DependencyShapeSnapshotId {
        &self.snapshot_id
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicationSnapshot {
    snapshot_id: PublicationSnapshotId,
    pub revision_id: WorkspaceRevisionId,
    pub state: SnapshotLayerState,
}

impl PublicationSnapshot {
    #[must_use]
    pub fn current_absent(revision_id: &WorkspaceRevisionId, reason: impl Into<String>) -> Self {
        let state = SnapshotLayerState::current_absent(reason);
        let snapshot_id = PublicationSnapshotId(mint_layer_snapshot_id(
            "publication-snapshot",
            &[("revision_id", &revision_id.0)],
            &state,
        ));
        Self {
            snapshot_id,
            revision_id: revision_id.clone(),
            state,
        }
    }

    #[must_use]
    pub fn from_published_values(
        revision_id: &WorkspaceRevisionId,
        published_values: &BTreeMap<TreeNodeId, String>,
        runtime_effects: &[RuntimeEffect],
    ) -> Self {
        let basis = publication_basis(
            published_values,
            None,
            runtime_effects,
            std::iter::empty::<&String>(),
        );
        Self::from_basis(revision_id, basis)
    }

    #[must_use]
    pub fn from_publication_bundle(
        revision_id: &WorkspaceRevisionId,
        published_values: &BTreeMap<TreeNodeId, String>,
        publication_bundle: &PublicationBundle,
        diagnostics: &[String],
    ) -> Self {
        let basis = publication_basis(
            published_values,
            Some(publication_bundle),
            &publication_bundle.published_runtime_effects,
            diagnostics.iter(),
        );
        Self::from_basis(revision_id, basis)
    }

    fn from_basis(revision_id: &WorkspaceRevisionId, basis: String) -> Self {
        let state = SnapshotLayerState::Current { basis };
        let snapshot_id = PublicationSnapshotId(mint_layer_snapshot_id(
            "publication-snapshot",
            &[("revision_id", &revision_id.0)],
            &state,
        ));
        Self {
            snapshot_id,
            revision_id: revision_id.clone(),
            state,
        }
    }

    #[must_use]
    pub fn snapshot_id(&self) -> &PublicationSnapshotId {
        &self.snapshot_id
    }
}

// The basis folds every published value and diagnostic line per published
// run; stream the bytes through one reused scratch buffer instead of
// building several intermediate strings per field. Output bytes are
// identical to the eager `identity`/`field` formulation (pinned by
// `streaming_identity` tests).
fn publication_basis<'a>(
    published_values: &BTreeMap<TreeNodeId, String>,
    publication_bundle: Option<&PublicationBundle>,
    runtime_effects: &[RuntimeEffect],
    diagnostics: impl IntoIterator<Item = &'a String>,
) -> String {
    use std::fmt::Write as _;
    let mut basis = identity_seed("publication-basis");
    let mut scratch = String::new();
    for (node_id, value) in published_values {
        scratch.clear();
        let _ = write!(scratch, "{}={}:", node_id.0, value.len());
        scratch.push_str(value);
        push_identity_field(&mut basis, "value", &scratch);
    }
    if let Some(bundle) = publication_bundle {
        push_identity_field(&mut basis, "publication_id", &bundle.publication_id);
        push_identity_field(
            &mut basis,
            "candidate_result_id",
            &bundle.candidate_result_id,
        );
        push_identity_field(
            &mut basis,
            "structural_snapshot_id",
            &bundle.structural_snapshot_id.0.to_string(),
        );
        for (node_id, value) in &bundle.published_calc_value_delta {
            scratch.clear();
            let display_text = calc_value_display_text(value);
            let _ = write!(scratch, "{}={}:", node_id.0, display_text.len());
            scratch.push_str(&display_text);
            push_identity_field(&mut basis, "delta", &scratch);
        }
        for (index, update) in bundle.dependency_shape_updates.iter().enumerate() {
            scratch.clear();
            let _ = write!(scratch, "{index}:kind={}:", update.kind.len());
            scratch.push_str(&update.kind);
            scratch.push_str(";affected=");
            for (position, node_id) in update.affected_node_ids.iter().enumerate() {
                if position > 0 {
                    scratch.push(',');
                }
                let _ = write!(scratch, "{}", node_id.0);
            }
            push_identity_field(&mut basis, "dependency_shape_update", &scratch);
        }
        for (index, marker) in bundle.trace_markers.iter().enumerate() {
            scratch.clear();
            let _ = write!(scratch, "{index}:{}:", marker.len());
            scratch.push_str(marker);
            push_identity_field(&mut basis, "trace_marker", &scratch);
        }
    }
    for (index, effect) in runtime_effects.iter().enumerate() {
        scratch.clear();
        let family_text = format!("{:?}", effect.family);
        let _ = write!(scratch, "{index}:{}:", effect.kind.len());
        scratch.push_str(&effect.kind);
        let _ = write!(scratch, ":{}:", family_text.len());
        scratch.push_str(&family_text);
        let _ = write!(scratch, ":{}:", effect.detail.len());
        scratch.push_str(&effect.detail);
        push_identity_field(&mut basis, "runtime_effect", &scratch);
    }
    for (index, diagnostic) in diagnostics.into_iter().enumerate() {
        scratch.clear();
        let _ = write!(scratch, "{index}:{}:", diagnostic.len());
        scratch.push_str(diagnostic);
        push_identity_field(&mut basis, "diagnostic", &scratch);
    }
    basis
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeOverlaySet {
    overlay_set_id: RuntimeOverlaySetId,
    pub publication_snapshot_id: PublicationSnapshotId,
    pub state: SnapshotLayerState,
}

impl RuntimeOverlaySet {
    #[must_use]
    pub fn current_absent(
        publication_snapshot_id: &PublicationSnapshotId,
        reason: impl Into<String>,
    ) -> Self {
        let state = SnapshotLayerState::current_absent(reason);
        let overlay_set_id = RuntimeOverlaySetId(mint_layer_snapshot_id(
            "runtime-overlay-set",
            &[("publication_snapshot_id", &publication_snapshot_id.0)],
            &state,
        ));
        Self {
            overlay_set_id,
            publication_snapshot_id: publication_snapshot_id.clone(),
            state,
        }
    }

    #[must_use]
    pub fn from_overlays(
        publication_snapshot_id: &PublicationSnapshotId,
        overlays: &[OverlayEntry],
    ) -> Self {
        let basis = identity(
            "runtime-overlay-basis",
            overlays
                .iter()
                .enumerate()
                .map(|(index, overlay)| {
                    field(
                        "overlay",
                        &format!(
                            "{index}:owner={};kind={:?};snapshot={};compat={};payload={};protected={};evictable={};detail={}",
                            overlay.key.owner_node_id.0,
                            overlay.key.overlay_kind,
                            overlay.key.structural_snapshot_id.0,
                            length_prefixed(&overlay.key.compatibility_basis),
                            optional_identity_value(overlay.key.payload_identity.as_deref()),
                            overlay.is_protected,
                            overlay.is_eviction_eligible,
                            length_prefixed(&overlay.detail)
                        ),
                    )
                })
                .collect::<Vec<_>>(),
        );
        let state = SnapshotLayerState::Current { basis };
        let overlay_set_id = RuntimeOverlaySetId(mint_layer_snapshot_id(
            "runtime-overlay-set",
            &[("publication_snapshot_id", &publication_snapshot_id.0)],
            &state,
        ));
        Self {
            overlay_set_id,
            publication_snapshot_id: publication_snapshot_id.clone(),
            state,
        }
    }

    #[must_use]
    pub fn overlay_set_id(&self) -> &RuntimeOverlaySetId {
        &self.overlay_set_id
    }
}

#[must_use]
pub fn workspace_revision_identity(
    workspace_id: &str,
    structure_snapshot_id: StructuralSnapshotId,
    node_input_snapshot_id: &NodeInputSnapshotId,
    namespace_snapshot_id: &NamespaceSnapshotId,
) -> String {
    // Runs on every edit and embeds the O(records) node-input snapshot id;
    // stream the fields to copy that id once instead of four times.
    let mut value = identity_seed("workspace-revision");
    push_identity_field(&mut value, "workspace_id", workspace_id);
    push_identity_field(
        &mut value,
        "structure_snapshot_id",
        &structure_snapshot_id.0.to_string(),
    );
    push_identity_field(
        &mut value,
        "node_input_snapshot_id",
        &node_input_snapshot_id.0,
    );
    push_identity_field(
        &mut value,
        "namespace_snapshot_id",
        &namespace_snapshot_id.0,
    );
    value
}

// This runs on every node-input edit and is O(records); stream the bytes
// through one reused scratch buffer instead of building five intermediate
// strings per record. Output bytes are identical to the eager
// `identity`/`field` formulation (pinned by `streaming_identity` tests).
fn node_input_snapshot_identity(records: &BTreeMap<TreeNodeId, NodeInputRecord>) -> String {
    use std::fmt::Write as _;
    let mut value = identity_seed("node-input-snapshot");
    value.reserve(records.len() * 64);
    let mut scratch = String::new();
    for record in records.values() {
        scratch.clear();
        let _ = write!(
            scratch,
            "node={};kind={};epoch={};text=",
            record.node_id.0,
            record.kind.as_identity_token(),
            record.input_epoch,
        );
        match record.text.as_deref() {
            Some(text) => {
                let _ = write!(scratch, "some:{}:", text.len());
                scratch.push_str(text);
            }
            None => scratch.push_str("none"),
        }
        push_identity_field(&mut value, "record", &scratch);
    }
    value
}

fn dependency_shape_basis(dependency_graph: &DependencyGraph) -> String {
    let descriptor_fields =
        dependency_graph
            .descriptors_by_owner
            .iter()
            .flat_map(|(owner_node_id, descriptors)| {
                descriptors
                    .iter()
                    .enumerate()
                    .map(move |(index, descriptor)| {
                        let collection = descriptor
                            .tree_reference_collection
                            .as_ref()
                            .map(|collection| format!("{collection:?}"));
                        field(
                            "descriptor",
                            &format!(
                                "owner={};index={};id={};source={};target={};workspace={};kind={:?};rebind={};collection={};detail={}",
                                owner_node_id.0,
                                index,
                                length_prefixed(&descriptor.descriptor_id),
                                optional_identity_value(
                                    descriptor.source_reference_handle.as_deref()
                                ),
                                descriptor.target_node_id.map_or_else(
                                    || "none".to_string(),
                                    |node_id| format!("some:{}", node_id.0)
                                ),
                                descriptor.workspace_target.as_ref().map_or_else(
                                    || "none".to_string(),
                                    |target| {
                                        format!(
                                            "some:{}:{}:{}:{}",
                                            length_prefixed(&target.workspace_handle),
                                            target.target_node_id.0,
                                            length_prefixed(&target.target_node_handle),
                                            length_prefixed(&target.availability_version)
                                        )
                                    }
                                ),
                                descriptor.kind,
                                descriptor.requires_rebind_on_structural_change,
                                optional_identity_value(collection.as_deref()),
                                length_prefixed(&descriptor.carrier_detail)
                            ),
                        )
                    })
            });
    let diagnostic_fields = dependency_graph.diagnostics.iter().map(|diagnostic| {
        field(
            "diagnostic",
            &format!(
                "id={};kind={:?};detail={}",
                length_prefixed(&diagnostic.descriptor_id),
                diagnostic.kind,
                length_prefixed(&diagnostic.detail)
            ),
        )
    });
    let cycle_fields = dependency_graph
        .cycle_groups
        .iter()
        .enumerate()
        .map(|(index, group)| {
            field(
                "cycle_group",
                &format!(
                    "index={};nodes={}",
                    index,
                    group
                        .iter()
                        .map(|node_id| node_id.0.to_string())
                        .collect::<Vec<_>>()
                        .join(",")
                ),
            )
        });
    identity(
        "dependency-shape-basis",
        descriptor_fields
            .chain(diagnostic_fields)
            .chain(cycle_fields)
            .collect::<Vec<_>>(),
    )
}

fn identity(namespace: &str, fields: impl IntoIterator<Item = String>) -> String {
    let mut value = identity_seed(namespace);
    for field in fields {
        push_identity_prebuilt_field(&mut value, &field);
    }
    value
}

fn field(name: &str, value: &str) -> String {
    use std::fmt::Write as _;
    let mut out = String::with_capacity(name.len() + value.len() + 24);
    let _ = write!(out, "{name}={}:", value.len());
    out.push_str(value);
    out
}

/// Streaming equivalents of `identity`/`field`. Identity strings here are
/// observable (snapshot and revision ids), so the bytes are frozen; these
/// helpers produce exactly the bytes the eager helpers produce, without
/// materializing the intermediate per-field strings. Hot per-edit builders
/// (node-input identities, layer-id minting, publication bases) write
/// through these.
fn identity_seed(namespace: &str) -> String {
    let mut value = String::with_capacity(namespace.len() + 3);
    value.push_str(namespace);
    value.push_str(":v1");
    value
}

/// Appends `|{field_len}:{name}={value_len}:{value}` — byte-identical to
/// folding `field(name, value)` into `identity`.
fn push_identity_field(out: &mut String, name: &str, value: &str) {
    use std::fmt::Write as _;
    let field_len = name.len() + 1 + decimal_digits(value.len()) + 1 + value.len();
    let _ = write!(out, "|{field_len}:{name}={}:", value.len());
    out.push_str(value);
}

/// Appends `|{len}:{field}` for an already-built field token (for example
/// `SnapshotLayerState::identity_token` output).
fn push_identity_prebuilt_field(out: &mut String, field: &str) {
    use std::fmt::Write as _;
    let _ = write!(out, "|{}:", field.len());
    out.push_str(field);
}

fn decimal_digits(value: usize) -> usize {
    let mut digits = 1;
    let mut rest = value / 10;
    while rest > 0 {
        digits += 1;
        rest /= 10;
    }
    digits
}

fn optional_field(name: &str, value: Option<&str>) -> String {
    field(name, &optional_identity_value(value))
}

fn optional_identity_value(value: Option<&str>) -> String {
    match value {
        Some(value) => format!("some:{}", length_prefixed(value)),
        None => "none".to_string(),
    }
}

fn length_prefixed(value: &str) -> String {
    format!("{}:{value}", value.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dependency::{DependencyDescriptor, DependencyDescriptorKind};
    use crate::structural::{StructuralNode, StructuralNodeKind};

    fn single_root_structure(snapshot_id: u64) -> StructuralSnapshot {
        let root_id = TreeNodeId(1);
        StructuralSnapshot::create(
            StructuralSnapshotId(snapshot_id),
            root_id,
            [StructuralNode {
                node_id: root_id,
                kind: StructuralNodeKind::Root,
                symbol: "Root".to_string(),
                parent_id: None,
                child_ids: Vec::new(),
            }],
        )
        .unwrap()
    }

    #[test]
    fn node_input_record_constructors_do_not_infer_formula_syntax() {
        let literal = NodeInputRecord::literal(TreeNodeId(7), "=1+1", 3);
        let formula = NodeInputRecord::formula_text(TreeNodeId(8), "plain text", 4);

        assert_eq!(literal.kind, NodeInputKind::Literal);
        assert_eq!(literal.text.as_deref(), Some("=1+1"));
        assert_eq!(formula.kind, NodeInputKind::FormulaText);
        assert_eq!(formula.text.as_deref(), Some("plain text"));
    }

    #[test]
    fn workspace_revision_identity_tracks_roots_deterministically() {
        let root_id = TreeNodeId(1);
        let structure = single_root_structure(1);
        let empty_inputs = NodeInputSnapshot::create([NodeInputRecord::empty(root_id, 1)]).unwrap();
        let literal_inputs =
            NodeInputSnapshot::create([NodeInputRecord::literal(root_id, "x", 2)]).unwrap();
        let namespace = NamespaceSnapshot::current_absent();

        let empty_revision = WorkspaceRevision::new(
            "workspace:test",
            structure.clone(),
            empty_inputs,
            namespace.clone(),
        );
        let repeat_revision = WorkspaceRevision::new(
            "workspace:test",
            structure.clone(),
            empty_revision.node_input_snapshot.clone(),
            namespace.clone(),
        );
        let literal_revision =
            WorkspaceRevision::new("workspace:test", structure, literal_inputs, namespace);

        assert_eq!(empty_revision.revision_id(), repeat_revision.revision_id());
        assert_ne!(empty_revision.revision_id(), literal_revision.revision_id());
    }

    #[test]
    fn skeletal_layer_absence_identities_are_explicit_and_stable() {
        let root_id = TreeNodeId(1);
        let revision = WorkspaceRevision::new(
            "workspace:test",
            single_root_structure(1),
            NodeInputSnapshot::create([NodeInputRecord::empty(root_id, 1)]).unwrap(),
            NamespaceSnapshot::current_absent(),
        );

        let formula_binding =
            FormulaBindingSnapshot::current_absent(revision.revision_id(), "not-yet-derived");
        let dependency_shape = DependencyShapeSnapshot::current_absent(
            revision.revision_id(),
            formula_binding.snapshot_id(),
            "not-yet-derived",
        );
        let publication =
            PublicationSnapshot::current_absent(revision.revision_id(), "not-yet-published");
        let overlays =
            RuntimeOverlaySet::current_absent(publication.snapshot_id(), "not-yet-projected");

        assert!(formula_binding.snapshot_id().0.contains("absent"));
        assert!(dependency_shape.snapshot_id().0.contains("absent"));
        assert!(publication.snapshot_id().0.contains("absent"));
        assert!(overlays.overlay_set_id().0.contains("absent"));
        assert_eq!(
            FormulaBindingSnapshot::current_absent(revision.revision_id(), "not-yet-derived")
                .snapshot_id(),
            formula_binding.snapshot_id()
        );
    }

    #[test]
    fn formula_binding_current_identity_tracks_basis() {
        let root_id = TreeNodeId(1);
        let revision = WorkspaceRevision::new(
            "workspace:test",
            single_root_structure(1),
            NodeInputSnapshot::create([NodeInputRecord::formula_text(root_id, "=1", 1)]).unwrap(),
            NamespaceSnapshot::current_absent(),
        );

        let left = FormulaBindingSnapshot::current(revision.revision_id(), "binding-basis:a");
        let repeat = FormulaBindingSnapshot::current(revision.revision_id(), "binding-basis:a");
        let right = FormulaBindingSnapshot::current(revision.revision_id(), "binding-basis:b");

        assert!(left.snapshot_id().0.contains("current"));
        assert_eq!(left.snapshot_id(), repeat.snapshot_id());
        assert_ne!(left.snapshot_id(), right.snapshot_id());
    }

    #[test]
    fn dependency_shape_identity_includes_descriptor_content_not_only_counts() {
        let root_id = TreeNodeId(1);
        let revision = WorkspaceRevision::new(
            "workspace:test",
            single_root_structure(1),
            NodeInputSnapshot::create([NodeInputRecord::empty(root_id, 1)]).unwrap(),
            NamespaceSnapshot::current_absent(),
        );
        let formula_binding =
            FormulaBindingSnapshot::current_absent(revision.revision_id(), "not-yet-derived");
        let left = DependencyShapeSnapshot::from_dependency_graph(
            revision.revision_id(),
            formula_binding.snapshot_id(),
            &dependency_graph_with_single_target(TreeNodeId(2)),
        );
        let right = DependencyShapeSnapshot::from_dependency_graph(
            revision.revision_id(),
            formula_binding.snapshot_id(),
            &dependency_graph_with_single_target(TreeNodeId(3)),
        );

        assert_ne!(left.snapshot_id(), right.snapshot_id());
    }

    /// Pins the streaming identity builders to the byte layout of the
    /// original eager `identity`/`field`/`length_prefixed` formulation.
    /// Identity strings are observable (snapshot/revision ids), so any
    /// byte drift here is a behavior change, not just a perf regression.
    mod streaming_identity {
        use super::super::*;
        use crate::coordinator::DependencyShapeUpdate;
        use oxfunc_core::value::CalcValue;

        fn reference_length_prefixed(value: &str) -> String {
            format!("{}:{value}", value.len())
        }

        fn reference_field(name: &str, value: &str) -> String {
            format!("{name}={}", reference_length_prefixed(value))
        }

        fn reference_identity(namespace: &str, fields: impl IntoIterator<Item = String>) -> String {
            let mut value = format!("{namespace}:v1");
            for field in fields {
                value.push('|');
                value.push_str(&reference_length_prefixed(&field));
            }
            value
        }

        #[test]
        fn field_and_identity_match_reference_bytes() {
            let lengths = [0usize, 1, 9, 10, 99, 100, 1234];
            for length in lengths {
                let value = "v".repeat(length);
                assert_eq!(field("name", &value), reference_field("name", &value));
                let mut streamed = identity_seed("ns");
                push_identity_field(&mut streamed, "name", &value);
                push_identity_prebuilt_field(&mut streamed, "raw-token");
                assert_eq!(
                    streamed,
                    reference_identity(
                        "ns",
                        [reference_field("name", &value), "raw-token".to_string()]
                    )
                );
            }
        }

        #[test]
        fn node_input_snapshot_identity_matches_reference_bytes() {
            let records = BTreeMap::from([
                (TreeNodeId(1), NodeInputRecord::empty(TreeNodeId(1), 1)),
                (
                    TreeNodeId(2),
                    NodeInputRecord::literal(TreeNodeId(2), "literal value", 3),
                ),
                (
                    TreeNodeId(30),
                    NodeInputRecord::formula_text(TreeNodeId(30), "=N1+N2", 12),
                ),
                (
                    TreeNodeId(31),
                    NodeInputRecord::host_owned(TreeNodeId(31), "host:identity", 4),
                ),
            ]);
            let reference = reference_identity(
                "node-input-snapshot",
                records.values().map(|record| {
                    reference_field(
                        "record",
                        &format!(
                            "node={};kind={};epoch={};text={}",
                            record.node_id.0,
                            record.kind.as_identity_token(),
                            record.input_epoch,
                            match record.text.as_deref() {
                                Some(text) => format!("some:{}", reference_length_prefixed(text)),
                                None => "none".to_string(),
                            }
                        ),
                    )
                }),
            );
            assert_eq!(node_input_snapshot_identity(&records), reference);
        }

        #[test]
        fn workspace_revision_identity_matches_reference_bytes() {
            let node_input_snapshot_id = NodeInputSnapshotId("input:id".to_string());
            let namespace_snapshot_id = NamespaceSnapshotId("namespace:id".to_string());
            assert_eq!(
                workspace_revision_identity(
                    "workspace:test",
                    StructuralSnapshotId(42),
                    &node_input_snapshot_id,
                    &namespace_snapshot_id,
                ),
                reference_identity(
                    "workspace-revision",
                    [
                        reference_field("workspace_id", "workspace:test"),
                        reference_field("structure_snapshot_id", "42"),
                        reference_field("node_input_snapshot_id", "input:id"),
                        reference_field("namespace_snapshot_id", "namespace:id"),
                    ],
                )
            );
        }

        #[test]
        fn mint_layer_snapshot_id_matches_reference_bytes() {
            for state in [
                SnapshotLayerState::current_absent("why-not"),
                SnapshotLayerState::current("layer basis bytes"),
            ] {
                assert_eq!(
                    mint_layer_snapshot_id(
                        "layer-ns",
                        &[("revision_id", "rev:1"), ("other_id", "other:2")],
                        &state,
                    ),
                    reference_identity(
                        "layer-ns",
                        [
                            reference_field("revision_id", "rev:1"),
                            reference_field("other_id", "other:2"),
                            state.identity_digest_token(),
                        ],
                    )
                );
            }
        }

        #[test]
        fn publication_basis_matches_reference_bytes() {
            let published_values = BTreeMap::from([
                (TreeNodeId(2), "2".to_string()),
                (TreeNodeId(7), "value seven".to_string()),
            ]);
            let bundle = PublicationBundle {
                publication_id: "publication:test:1".to_string(),
                candidate_result_id: "candidate:test:1".to_string(),
                structural_snapshot_id: StructuralSnapshotId(9),
                published_calc_value_delta: BTreeMap::from([
                    (TreeNodeId(2), CalcValue::number(2.0)),
                    (TreeNodeId(7), CalcValue::number(49.5)),
                ]),
                dependency_shape_updates: vec![
                    DependencyShapeUpdate {
                        kind: "added".to_string(),
                        affected_node_ids: vec![TreeNodeId(2), TreeNodeId(7)],
                    },
                    DependencyShapeUpdate {
                        kind: "removed".to_string(),
                        affected_node_ids: Vec::new(),
                    },
                ],
                published_runtime_effects: Vec::new(),
                trace_markers: vec!["marker-a".to_string(), "marker-b".to_string()],
            };
            let runtime_effects = vec![RuntimeEffect {
                kind: "effect-kind".to_string(),
                family: crate::coordinator::RuntimeEffectFamily::DynamicDependency,
                detail: "effect detail".to_string(),
            }];
            let diagnostics = vec!["diag one".to_string(), String::new()];

            let value_fields = published_values.iter().map(|(node_id, value)| {
                reference_field(
                    "value",
                    &format!("{}={}", node_id.0, reference_length_prefixed(value)),
                )
            });
            let header_fields = [
                reference_field("publication_id", &bundle.publication_id),
                reference_field("candidate_result_id", &bundle.candidate_result_id),
                reference_field(
                    "structural_snapshot_id",
                    &bundle.structural_snapshot_id.0.to_string(),
                ),
            ];
            let delta_fields = bundle
                .published_calc_value_delta
                .iter()
                .map(|(node_id, value)| {
                    reference_field(
                        "delta",
                        &format!(
                            "{}={}",
                            node_id.0,
                            reference_length_prefixed(&calc_value_display_text(value))
                        ),
                    )
                });
            let dependency_shape_fields =
                bundle
                    .dependency_shape_updates
                    .iter()
                    .enumerate()
                    .map(|(index, update)| {
                        reference_field(
                            "dependency_shape_update",
                            &format!(
                                "{index}:kind={};affected={}",
                                reference_length_prefixed(&update.kind),
                                update
                                    .affected_node_ids
                                    .iter()
                                    .map(|node_id| node_id.0.to_string())
                                    .collect::<Vec<_>>()
                                    .join(",")
                            ),
                        )
                    });
            let trace_fields = bundle
                .trace_markers
                .iter()
                .enumerate()
                .map(|(index, marker)| {
                    reference_field(
                        "trace_marker",
                        &format!("{index}:{}", reference_length_prefixed(marker)),
                    )
                });
            let effect_fields = runtime_effects.iter().enumerate().map(|(index, effect)| {
                reference_field(
                    "runtime_effect",
                    &format!(
                        "{index}:{}:{}:{}",
                        reference_length_prefixed(&effect.kind),
                        reference_length_prefixed(&format!("{:?}", effect.family)),
                        reference_length_prefixed(&effect.detail)
                    ),
                )
            });
            let diagnostic_fields = diagnostics.iter().enumerate().map(|(index, diagnostic)| {
                reference_field(
                    "diagnostic",
                    &format!("{index}:{}", reference_length_prefixed(diagnostic)),
                )
            });
            let reference = reference_identity(
                "publication-basis",
                value_fields
                    .chain(header_fields)
                    .chain(delta_fields)
                    .chain(dependency_shape_fields)
                    .chain(trace_fields)
                    .chain(effect_fields)
                    .chain(diagnostic_fields)
                    .collect::<Vec<_>>(),
            );
            assert_eq!(
                publication_basis(
                    &published_values,
                    Some(&bundle),
                    &runtime_effects,
                    diagnostics.iter(),
                ),
                reference
            );

            // No-bundle shape (the `from_published_values` path).
            let reference_without_bundle = reference_identity(
                "publication-basis",
                published_values
                    .iter()
                    .map(|(node_id, value)| {
                        reference_field(
                            "value",
                            &format!("{}={}", node_id.0, reference_length_prefixed(value)),
                        )
                    })
                    .collect::<Vec<_>>(),
            );
            assert_eq!(
                publication_basis(&published_values, None, &[], std::iter::empty::<&String>(),),
                reference_without_bundle
            );
        }
    }

    fn dependency_graph_with_single_target(target_node_id: TreeNodeId) -> DependencyGraph {
        let owner_node_id = TreeNodeId(1);
        DependencyGraph {
            snapshot_id: StructuralSnapshotId(1),
            descriptors_by_owner: BTreeMap::from([(
                owner_node_id,
                vec![DependencyDescriptor {
                    descriptor_id: "descriptor:a".to_string(),
                    source_reference_handle: None,
                    owner_node_id,
                    target_node_id: Some(target_node_id),
                    workspace_target: None,
                    kind: DependencyDescriptorKind::StaticDirect,
                    carrier_detail: "carrier:a".to_string(),
                    tree_reference_collection: None,
                    requires_rebind_on_structural_change: false,
                }],
            )]),
            edges_by_owner: BTreeMap::new(),
            reverse_edges: BTreeMap::new(),
            workspace_reverse_edges: BTreeMap::new(),
            cycle_groups: Vec::new(),
            diagnostics: Vec::new(),
        }
    }
}
