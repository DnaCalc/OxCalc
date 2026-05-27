#![forbid(unsafe_code)]

//! Workspace revision and snapshot-layer identity types.

use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::coordinator::RuntimeEffect;
use crate::dependency::DependencyGraph;
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
    pub fn identity_token(&self) -> String {
        match self {
            Self::CurrentAbsent { reason } => field("absent", reason),
            Self::Current { basis } => field("current", basis),
        }
    }
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
        let snapshot_id = FormulaBindingSnapshotId(identity(
            "formula-binding-snapshot",
            [field("revision_id", &revision_id.0), state.identity_token()],
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
        let snapshot_id = DependencyShapeSnapshotId(identity(
            "dependency-shape-snapshot",
            [
                field("revision_id", &revision_id.0),
                field(
                    "formula_binding_snapshot_id",
                    &formula_binding_snapshot_id.0,
                ),
                state.identity_token(),
            ],
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
        let snapshot_id = DependencyShapeSnapshotId(identity(
            "dependency-shape-snapshot",
            [
                field("revision_id", &revision_id.0),
                field(
                    "formula_binding_snapshot_id",
                    &formula_binding_snapshot_id.0,
                ),
                state.identity_token(),
            ],
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
        let snapshot_id = PublicationSnapshotId(identity(
            "publication-snapshot",
            [field("revision_id", &revision_id.0), state.identity_token()],
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
        let value_fields = published_values.iter().map(|(node_id, value)| {
            field(
                "value",
                &format!("{}={}", node_id.0, length_prefixed(value)),
            )
        });
        let effect_fields = runtime_effects.iter().enumerate().map(|(index, effect)| {
            field(
                "runtime_effect",
                &format!(
                    "{index}:{}:{}:{}",
                    length_prefixed(&effect.kind),
                    length_prefixed(&format!("{:?}", effect.family)),
                    length_prefixed(&effect.detail)
                ),
            )
        });
        let basis = identity(
            "publication-basis",
            value_fields.chain(effect_fields).collect::<Vec<_>>(),
        );
        let state = SnapshotLayerState::Current { basis };
        let snapshot_id = PublicationSnapshotId(identity(
            "publication-snapshot",
            [field("revision_id", &revision_id.0), state.identity_token()],
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
        let overlay_set_id = RuntimeOverlaySetId(identity(
            "runtime-overlay-set",
            [
                field("publication_snapshot_id", &publication_snapshot_id.0),
                state.identity_token(),
            ],
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
        let overlay_set_id = RuntimeOverlaySetId(identity(
            "runtime-overlay-set",
            [
                field("publication_snapshot_id", &publication_snapshot_id.0),
                state.identity_token(),
            ],
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
    identity(
        "workspace-revision",
        [
            field("workspace_id", workspace_id),
            field(
                "structure_snapshot_id",
                &structure_snapshot_id.0.to_string(),
            ),
            field("node_input_snapshot_id", &node_input_snapshot_id.0),
            field("namespace_snapshot_id", &namespace_snapshot_id.0),
        ],
    )
}

fn node_input_snapshot_identity(records: &BTreeMap<TreeNodeId, NodeInputRecord>) -> String {
    identity(
        "node-input-snapshot",
        records
            .values()
            .map(|record| {
                field(
                    "record",
                    &format!(
                        "node={};kind={};epoch={};text={}",
                        record.node_id.0,
                        record.kind.as_identity_token(),
                        record.input_epoch,
                        optional_identity_value(record.text.as_deref())
                    ),
                )
            })
            .collect::<Vec<_>>(),
    )
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
    let mut value = format!("{namespace}:v1");
    for field in fields {
        value.push('|');
        value.push_str(&length_prefixed(&field));
    }
    value
}

fn field(name: &str, value: &str) -> String {
    format!("{name}={}", length_prefixed(value))
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
