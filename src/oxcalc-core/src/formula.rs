#![forbid(unsafe_code)]

//! TreeCalc-local formula and reference substrate.

use std::collections::{BTreeMap, BTreeSet};

use oxfml_core::binding::BoundFormula;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;

use crate::dependency::{
    DependencyDescriptor, DependencyDescriptorKind, InvalidationReasonKind,
    TreeReferenceCollectionDependency, TreeReferenceCollectionFamily, WorkspaceQualifiedTarget,
};
use crate::structural::{BindArtifactId, FormulaArtifactId, StructuralSnapshot, TreeNodeId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum RelativeReferenceBase {
    SelfNode,
    ParentNode,
    Ancestor(usize),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TreeReference {
    DirectNode {
        target_node_id: TreeNodeId,
    },
    ReferenceCollection(TreeCalcReferenceCollection),
    ProjectionPath {
        projection_path: String,
    },
    RelativePath {
        base: RelativeReferenceBase,
        path_segments: Vec<String>,
    },
    SiblingOffset {
        offset: isize,
        tail_segments: Vec<String>,
    },
    QualifiedSiblingOffset {
        base_node_id: TreeNodeId,
        offset: isize,
        tail_segments: Vec<String>,
    },
    QualifiedParentOffset {
        base_node_id: TreeNodeId,
        tail_segments: Vec<String>,
    },
    HostSensitive {
        carrier_id: String,
        detail: String,
    },
    CrossWorkspaceResolved {
        workspace_handle: String,
        target_node_id: TreeNodeId,
        target_node_handle: String,
        availability_version: String,
        carrier_id: String,
        detail: String,
    },
    CapabilitySensitive {
        carrier_id: String,
        detail: String,
    },
    ShapeTopology {
        carrier_id: String,
        detail: String,
    },
    DynamicPotential {
        carrier_id: String,
        detail: String,
    },
    DynamicResolved {
        target_node_id: TreeNodeId,
        carrier_id: String,
        detail: String,
    },
    Unresolved {
        token: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TreeCalcReferenceCollection {
    ChildrenV1(TreeCalcChildrenReferenceCollection),
    ReferenceLiteralArrayV1(TreeCalcReferenceLiteralArrayCollection),
    OrderedSelectorV1(TreeCalcOrderedSelectorReferenceCollection),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum TreeCalcOrderedSelectorFamily {
    SiblingSetV1,
    PrecedingV1,
    FollowingV1,
    AncestorsV1,
    RecursiveDescendantsV1,
}

impl TreeCalcOrderedSelectorFamily {
    #[must_use]
    pub const fn stable_id(self) -> &'static str {
        match self {
            TreeCalcOrderedSelectorFamily::SiblingSetV1 => "siblings",
            TreeCalcOrderedSelectorFamily::PrecedingV1 => "preceding",
            TreeCalcOrderedSelectorFamily::FollowingV1 => "following",
            TreeCalcOrderedSelectorFamily::AncestorsV1 => "ancestors",
            TreeCalcOrderedSelectorFamily::RecursiveDescendantsV1 => "recursive_descendants",
        }
    }

    #[must_use]
    pub const fn selector_name(self) -> &'static str {
        match self {
            TreeCalcOrderedSelectorFamily::SiblingSetV1 => "Siblings",
            TreeCalcOrderedSelectorFamily::PrecedingV1 => "Preceding",
            TreeCalcOrderedSelectorFamily::FollowingV1 => "Following",
            TreeCalcOrderedSelectorFamily::AncestorsV1 => "Ancestors",
            TreeCalcOrderedSelectorFamily::RecursiveDescendantsV1 => "RecursiveDescendants",
        }
    }

    #[must_use]
    pub const fn dependency_family(self) -> TreeReferenceCollectionFamily {
        match self {
            TreeCalcOrderedSelectorFamily::SiblingSetV1 => {
                TreeReferenceCollectionFamily::SiblingSetV1
            }
            TreeCalcOrderedSelectorFamily::PrecedingV1 => {
                TreeReferenceCollectionFamily::PrecedingV1
            }
            TreeCalcOrderedSelectorFamily::FollowingV1 => {
                TreeReferenceCollectionFamily::FollowingV1
            }
            TreeCalcOrderedSelectorFamily::AncestorsV1 => {
                TreeReferenceCollectionFamily::AncestorsV1
            }
            TreeCalcOrderedSelectorFamily::RecursiveDescendantsV1 => {
                TreeReferenceCollectionFamily::RecursiveDescendantsV1
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreeCalcChildrenReferenceCollection {
    pub host_ref_handle: String,
    pub base_node_id: TreeNodeId,
    pub source_span_utf8: Option<(usize, usize)>,
    pub source_token_text: String,
    pub opaque_selector: String,
    pub membership_version: String,
    pub order_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeCalcReferenceLiteralArrayCollection {
    carrier_id: String,
    host_ref_handle: String,
    owner_node_id: TreeNodeId,
    elements: Vec<TreeCalcReferenceLiteralArrayElement>,
    member_node_ids: Vec<TreeNodeId>,
    source_span_utf8: Option<(usize, usize)>,
    source_token_text: String,
    opaque_selector: String,
    membership_version: String,
    order_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TreeCalcReferenceLiteralArrayElement {
    ReferenceNode(TreeNodeId),
    ScalarValue { source_text: String },
}

#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum TreeCalcReferenceLiteralArrayError {
    #[error(
        "mixed scalar/reference array literal '{source_token_text}' cannot be lowered as a reference-only carrier"
    )]
    MixedScalarReferenceArray { source_token_text: String },
    #[error("reference literal array '{source_token_text}' contains no references")]
    EmptyReferenceArray { source_token_text: String },
}

impl TreeCalcReferenceLiteralArrayCollection {
    pub fn reference_only(
        carrier_id: impl Into<String>,
        owner_node_id: TreeNodeId,
        source_token_text: impl Into<String>,
        elements: impl IntoIterator<Item = TreeCalcReferenceLiteralArrayElement>,
    ) -> Result<Self, TreeCalcReferenceLiteralArrayError> {
        let carrier_id = carrier_id.into();
        let host_ref_handle = format!("treecalc-hostref:v1:reference_literal_array:{carrier_id}");
        Self::reference_only_with_handle(
            carrier_id,
            host_ref_handle,
            owner_node_id,
            source_token_text,
            elements,
        )
    }

    pub fn reference_only_with_handle(
        carrier_id: impl Into<String>,
        host_ref_handle: impl Into<String>,
        owner_node_id: TreeNodeId,
        source_token_text: impl Into<String>,
        elements: impl IntoIterator<Item = TreeCalcReferenceLiteralArrayElement>,
    ) -> Result<Self, TreeCalcReferenceLiteralArrayError> {
        let carrier_id = carrier_id.into();
        let host_ref_handle = host_ref_handle.into();
        let source_token_text = source_token_text.into();
        let elements = elements.into_iter().collect::<Vec<_>>();
        let mut member_node_ids = Vec::new();
        for element in &elements {
            match element {
                TreeCalcReferenceLiteralArrayElement::ReferenceNode(node_id) => {
                    member_node_ids.push(*node_id);
                }
                TreeCalcReferenceLiteralArrayElement::ScalarValue { .. } => {
                    return Err(
                        TreeCalcReferenceLiteralArrayError::MixedScalarReferenceArray {
                            source_token_text: source_token_text.clone(),
                        },
                    );
                }
            }
        }
        if member_node_ids.is_empty() {
            return Err(TreeCalcReferenceLiteralArrayError::EmptyReferenceArray {
                source_token_text: source_token_text.clone(),
            });
        }

        Ok(Self {
            host_ref_handle,
            carrier_id,
            owner_node_id,
            elements,
            opaque_selector: format!(
                "oxcalc.treecalc.host_selector.v1:selector=ReferenceLiteralArray;owner={owner_node_id};members={}",
                format_tree_node_ids(&member_node_ids)
            ),
            membership_version: format!(
                "treecalc-membership:v1:family=reference_literal_array;owner={owner_node_id};members={}",
                format_tree_node_id_set(&member_node_ids)
            ),
            order_version: format!(
                "treecalc-order:v1:family=reference_literal_array;owner={owner_node_id};members={}",
                format_tree_node_ids(&member_node_ids)
            ),
            member_node_ids,
            source_span_utf8: None,
            source_token_text,
        })
    }

    #[must_use]
    pub fn with_source_span_utf8(mut self, start_byte: usize, end_byte: usize) -> Self {
        self.source_span_utf8 = Some((start_byte, end_byte));
        self
    }

    #[must_use]
    pub fn carrier_id(&self) -> &str {
        &self.carrier_id
    }

    #[must_use]
    pub fn host_ref_handle(&self) -> &str {
        &self.host_ref_handle
    }

    #[must_use]
    pub const fn owner_node_id(&self) -> TreeNodeId {
        self.owner_node_id
    }

    #[must_use]
    pub fn elements(&self) -> &[TreeCalcReferenceLiteralArrayElement] {
        &self.elements
    }

    #[must_use]
    pub fn member_node_ids(&self) -> &[TreeNodeId] {
        &self.member_node_ids
    }

    #[must_use]
    pub const fn source_span_utf8(&self) -> Option<(usize, usize)> {
        self.source_span_utf8
    }

    #[must_use]
    pub fn source_token_text(&self) -> &str {
        &self.source_token_text
    }

    #[must_use]
    pub fn opaque_selector(&self) -> &str {
        &self.opaque_selector
    }

    #[must_use]
    pub fn membership_version(&self) -> &str {
        &self.membership_version
    }

    #[must_use]
    pub fn order_version(&self) -> &str {
        &self.order_version
    }
}

impl Serialize for TreeCalcReferenceLiteralArrayCollection {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state =
            serializer.serialize_struct("TreeCalcReferenceLiteralArrayCollection", 5)?;
        state.serialize_field("carrier_id", &self.carrier_id)?;
        state.serialize_field("owner_node_id", &self.owner_node_id)?;
        state.serialize_field("source_span_utf8", &self.source_span_utf8)?;
        state.serialize_field("source_token_text", &self.source_token_text)?;
        state.serialize_field("elements", &self.elements)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for TreeCalcReferenceLiteralArrayCollection {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct ReferenceLiteralArrayPacket {
            carrier_id: String,
            owner_node_id: TreeNodeId,
            #[serde(default)]
            source_span_utf8: Option<(usize, usize)>,
            source_token_text: String,
            elements: Vec<TreeCalcReferenceLiteralArrayElement>,
        }

        let packet = ReferenceLiteralArrayPacket::deserialize(deserializer)?;
        let mut collection = Self::reference_only(
            packet.carrier_id,
            packet.owner_node_id,
            packet.source_token_text,
            packet.elements,
        )
        .map_err(serde::de::Error::custom)?;
        collection.source_span_utf8 = packet.source_span_utf8;
        Ok(collection)
    }
}

impl TreeCalcChildrenReferenceCollection {
    #[must_use]
    pub fn new(base_node_id: TreeNodeId, source_token_text: impl Into<String>) -> Self {
        let source_token_text = source_token_text.into();
        Self {
            host_ref_handle: format!("treecalc-hostref:v1:children:{base_node_id}"),
            base_node_id,
            source_span_utf8: None,
            source_token_text,
            opaque_selector: format!(
                "oxcalc.treecalc.host_selector.v1:selector=Children;base={base_node_id};include_meta=false;order=sibling_index"
            ),
            membership_version: format!("treecalc-membership:v1:{base_node_id}"),
            order_version: format!("treecalc-order:v1:{base_node_id}"),
        }
    }

    #[must_use]
    pub fn with_source_span_utf8(mut self, start_byte: usize, end_byte: usize) -> Self {
        self.source_span_utf8 = Some((start_byte, end_byte));
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreeCalcOrderedSelectorReferenceCollection {
    pub family: TreeCalcOrderedSelectorFamily,
    pub host_ref_handle: String,
    pub base_node_id: TreeNodeId,
    pub member_node_ids: Vec<TreeNodeId>,
    pub source_span_utf8: Option<(usize, usize)>,
    pub source_token_text: String,
    pub opaque_selector: String,
    pub membership_version: String,
    pub order_version: String,
}

impl TreeCalcOrderedSelectorReferenceCollection {
    #[must_use]
    pub fn new(
        family: TreeCalcOrderedSelectorFamily,
        base_node_id: TreeNodeId,
        source_token_text: impl Into<String>,
        member_node_ids: impl IntoIterator<Item = TreeNodeId>,
    ) -> Self {
        let source_token_text = source_token_text.into();
        let member_node_ids = member_node_ids.into_iter().collect::<Vec<_>>();
        let family_id = family.stable_id();
        Self {
            family,
            host_ref_handle: format!("treecalc-hostref:v1:{family_id}:{base_node_id}"),
            base_node_id,
            opaque_selector: format!(
                "oxcalc.treecalc.host_selector.v1:selector={};base={base_node_id};order=stable_tree_order;members={}",
                family.selector_name(),
                format_tree_node_ids(&member_node_ids)
            ),
            membership_version: format!(
                "treecalc-membership:v1:family={family_id};base={base_node_id};members={}",
                format_tree_node_id_set(&member_node_ids)
            ),
            order_version: format!(
                "treecalc-order:v1:family={family_id};base={base_node_id};members={}",
                format_tree_node_ids(&member_node_ids)
            ),
            member_node_ids,
            source_span_utf8: None,
            source_token_text,
        }
    }

    #[must_use]
    pub fn with_source_span_utf8(mut self, start_byte: usize, end_byte: usize) -> Self {
        self.source_span_utf8 = Some((start_byte, end_byte));
        self
    }
}

fn format_tree_node_ids(node_ids: &[TreeNodeId]) -> String {
    node_ids
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

fn format_tree_node_id_set(node_ids: &[TreeNodeId]) -> String {
    node_ids
        .iter()
        .copied()
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .map(|node_id| node_id.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

fn resolve_visible_sibling_offset_target(
    snapshot: &StructuralSnapshot,
    meta_node_ids: &BTreeSet<TreeNodeId>,
    base_node_id: TreeNodeId,
    offset: isize,
    tail_segments: &[String],
) -> Option<TreeNodeId> {
    let parent_id = snapshot.parent_id_of(base_node_id)?;
    let parent = snapshot.try_get_node(parent_id)?;
    let visible_siblings = parent
        .child_ids
        .iter()
        .copied()
        .filter(|child_id| !is_meta_effective(*child_id, snapshot, meta_node_ids))
        .collect::<Vec<_>>();
    let base_index = visible_siblings
        .iter()
        .position(|child_id| *child_id == base_node_id)?;
    let target_index = isize::try_from(base_index)
        .ok()?
        .checked_add(offset)
        .and_then(|index| usize::try_from(index).ok())?;
    let sibling_node_id = *visible_siblings.get(target_index)?;
    if tail_segments.is_empty() {
        Some(sibling_node_id)
    } else {
        try_resolve_visible_descendant_path(snapshot, meta_node_ids, sibling_node_id, tail_segments)
    }
}

fn try_resolve_visible_descendant_path(
    snapshot: &StructuralSnapshot,
    meta_node_ids: &BTreeSet<TreeNodeId>,
    start_node_id: TreeNodeId,
    path_segments: &[String],
) -> Option<TreeNodeId> {
    let mut cursor = Some(start_node_id);
    for segment in path_segments {
        cursor = cursor.and_then(|current| {
            let parent = snapshot.try_get_node(current)?;
            parent.child_ids.iter().copied().find(|child_id| {
                snapshot
                    .try_get_node(*child_id)
                    .is_some_and(|child| child.symbol.eq_ignore_ascii_case(segment))
                    && !is_meta_effective(*child_id, snapshot, meta_node_ids)
            })
        });
    }
    cursor
}

fn is_meta_effective(
    node_id: TreeNodeId,
    snapshot: &StructuralSnapshot,
    meta_node_ids: &BTreeSet<TreeNodeId>,
) -> bool {
    let mut cursor = Some(node_id);
    while let Some(current) = cursor {
        if meta_node_ids.contains(&current) {
            return true;
        }
        cursor = snapshot.parent_id_of(current);
    }
    false
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TreeCalcHostPathBaseResolutionLayer {
    ProjectionPath,
    DottedProjectionPath,
    RootDescendantPath,
    WorkspaceRoot,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreeCalcHostPathBaseResolution {
    pub base_token_text: String,
    pub base_node_id: TreeNodeId,
    pub canonical_projection_path: String,
    pub resolution_layer: TreeCalcHostPathBaseResolutionLayer,
    pub resolution_identity: String,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum TreeCalcHostPathBaseResolutionError {
    #[error("TreeCalc explicit host path base token is empty")]
    EmptyBaseToken,
    #[error(
        "TreeCalc cross-workspace base token '{base_token_text}' needs workspace provider and alias semantics"
    )]
    CrossWorkspaceBaseToken { base_token_text: String },
    #[error(
        "TreeCalc explicit host path base token '{base_token_text}' has invalid syntax: {detail}"
    )]
    InvalidBaseTokenSyntax {
        base_token_text: String,
        detail: String,
    },
    #[error("TreeCalc explicit host path base token '{base_token_text}' did not resolve")]
    UnresolvedBaseToken { base_token_text: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TreeCalcCrossWorkspaceAvailabilityStatus {
    Available,
    Unavailable,
    Degraded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TreeCalcCrossWorkspaceDiagnosticCode {
    WorkspaceProviderMissing,
    WorkspaceUnavailable,
    WorkspaceDegraded,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreeCalcCrossWorkspaceDiagnostic {
    pub code: TreeCalcCrossWorkspaceDiagnosticCode,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreeCalcCrossWorkspaceAvailabilityPacket {
    pub workspace_handle: String,
    pub workspace_selector_token: String,
    pub availability_version: String,
    pub status: TreeCalcCrossWorkspaceAvailabilityStatus,
    pub degradation_layer: Option<String>,
    pub diagnostics: Vec<TreeCalcCrossWorkspaceDiagnostic>,
}

impl TreeCalcCrossWorkspaceAvailabilityPacket {
    #[must_use]
    pub fn unavailable(
        workspace_handle: impl Into<String>,
        workspace_selector_token: impl Into<String>,
        availability_version: impl Into<String>,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            workspace_handle: workspace_handle.into(),
            workspace_selector_token: workspace_selector_token.into(),
            availability_version: availability_version.into(),
            status: TreeCalcCrossWorkspaceAvailabilityStatus::Unavailable,
            degradation_layer: Some("workspace_unavailable".to_string()),
            diagnostics: vec![TreeCalcCrossWorkspaceDiagnostic {
                code: TreeCalcCrossWorkspaceDiagnosticCode::WorkspaceUnavailable,
                detail: detail.into(),
            }],
        }
    }

    #[must_use]
    pub fn provider_missing(workspace_selector_token: impl Into<String>) -> Self {
        let workspace_selector_token = workspace_selector_token.into();
        Self {
            workspace_handle: format!("treecalc-workspace:unresolved:{workspace_selector_token}"),
            availability_version: "treecalc-cross-workspace-availability:v1:provider_missing"
                .to_string(),
            workspace_selector_token,
            status: TreeCalcCrossWorkspaceAvailabilityStatus::Unavailable,
            degradation_layer: Some("workspace_provider_missing".to_string()),
            diagnostics: vec![TreeCalcCrossWorkspaceDiagnostic {
                code: TreeCalcCrossWorkspaceDiagnosticCode::WorkspaceProviderMissing,
                detail: "no cross-workspace provider was supplied for this TreeCalc host reference"
                    .to_string(),
            }],
        }
    }

    #[must_use]
    pub fn degraded(
        workspace_handle: impl Into<String>,
        workspace_selector_token: impl Into<String>,
        availability_version: impl Into<String>,
        degradation_layer: impl Into<String>,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            workspace_handle: workspace_handle.into(),
            workspace_selector_token: workspace_selector_token.into(),
            availability_version: availability_version.into(),
            status: TreeCalcCrossWorkspaceAvailabilityStatus::Degraded,
            degradation_layer: Some(degradation_layer.into()),
            diagnostics: vec![TreeCalcCrossWorkspaceDiagnostic {
                code: TreeCalcCrossWorkspaceDiagnosticCode::WorkspaceDegraded,
                detail: detail.into(),
            }],
        }
    }

    #[must_use]
    pub fn available(
        workspace_handle: impl Into<String>,
        workspace_selector_token: impl Into<String>,
        availability_version: impl Into<String>,
    ) -> Self {
        Self {
            workspace_handle: workspace_handle.into(),
            workspace_selector_token: workspace_selector_token.into(),
            availability_version: availability_version.into(),
            status: TreeCalcCrossWorkspaceAvailabilityStatus::Available,
            degradation_layer: None,
            diagnostics: Vec::new(),
        }
    }

    #[must_use]
    pub fn prepared_identity_component(&self) -> String {
        format!(
            "cross_workspace_availability_version={};workspace_handle={};status={:?};degradation_layer={}",
            self.availability_version,
            self.workspace_handle,
            self.status,
            self.degradation_layer.as_deref().unwrap_or("none")
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TreeCalcWorkspaceSelectorKind {
    CurrentWorkspaceRoot,
    AliasOrDirectPath,
    QuotedDirectPath,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TreeCalcWorkspaceResolutionLayer {
    CurrentWorkspace,
    WorkspaceAlias,
    DirectWorkspacePath,
    QuotedWorkspacePath,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreeCalcWorkspaceResolutionPacket {
    pub selector_token_text: String,
    pub selector_kind: TreeCalcWorkspaceSelectorKind,
    pub workspace_handle: String,
    pub resolution_layer: TreeCalcWorkspaceResolutionLayer,
    pub availability_packet: TreeCalcCrossWorkspaceAvailabilityPacket,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreeCalcWorkspaceHostPathBaseResolution {
    pub base_token_text: String,
    pub workspace_selector_token: Option<String>,
    pub local_path_token_text: String,
    pub workspace_handle: String,
    pub base_node_id: TreeNodeId,
    pub base_node_handle: String,
    pub canonical_projection_path: String,
    pub workspace_resolution_layer: TreeCalcWorkspaceResolutionLayer,
    pub local_resolution_layer: TreeCalcHostPathBaseResolutionLayer,
    pub availability_packet: TreeCalcCrossWorkspaceAvailabilityPacket,
    pub resolution_identity: String,
}

impl TreeCalcWorkspaceHostPathBaseResolution {
    #[must_use]
    pub fn to_workspace_qualified_reference(&self, carrier_id: impl Into<String>) -> TreeReference {
        TreeReference::CrossWorkspaceResolved {
            workspace_handle: self.workspace_handle.clone(),
            target_node_id: self.base_node_id,
            target_node_handle: self.base_node_handle.clone(),
            availability_version: self.availability_packet.availability_version.clone(),
            carrier_id: carrier_id.into(),
            detail: self.resolution_identity.clone(),
        }
    }
}

#[derive(Debug)]
pub struct TreeCalcWorkspaceRegistryEntry<'a> {
    pub workspace_handle: String,
    pub snapshot: &'a StructuralSnapshot,
    pub availability_version: String,
}

#[derive(Debug)]
pub struct TreeCalcWorkspaceResolutionRegistry<'a> {
    current_workspace_handle: String,
    workspaces_by_handle: BTreeMap<String, TreeCalcWorkspaceRegistryEntry<'a>>,
    aliases: BTreeMap<String, String>,
}

impl<'a> TreeCalcWorkspaceResolutionRegistry<'a> {
    #[must_use]
    pub fn with_current_workspace(
        workspace_handle: impl Into<String>,
        snapshot: &'a StructuralSnapshot,
        availability_version: impl Into<String>,
    ) -> Self {
        let workspace_handle = workspace_handle.into();
        let mut registry = Self {
            current_workspace_handle: workspace_handle.clone(),
            workspaces_by_handle: BTreeMap::new(),
            aliases: BTreeMap::new(),
        };
        registry.add_workspace(workspace_handle, snapshot, availability_version);
        registry
    }

    pub fn add_workspace(
        &mut self,
        workspace_handle: impl Into<String>,
        snapshot: &'a StructuralSnapshot,
        availability_version: impl Into<String>,
    ) {
        let workspace_handle = workspace_handle.into();
        self.workspaces_by_handle.insert(
            workspace_handle.clone(),
            TreeCalcWorkspaceRegistryEntry {
                workspace_handle,
                snapshot,
                availability_version: availability_version.into(),
            },
        );
    }

    pub fn add_alias(
        &mut self,
        selector_token_text: impl Into<String>,
        workspace_handle: impl Into<String>,
    ) {
        self.aliases
            .insert(selector_token_text.into(), workspace_handle.into());
    }

    #[must_use]
    pub fn workspace_snapshot(&self, workspace_handle: &str) -> Option<&'a StructuralSnapshot> {
        self.workspaces_by_handle
            .get(workspace_handle)
            .map(|entry| entry.snapshot)
    }

    #[must_use]
    pub fn resolve_workspace_selector(
        &self,
        selector_token_text: &str,
        selector_kind: TreeCalcWorkspaceSelectorKind,
    ) -> TreeCalcWorkspaceResolutionPacket {
        if matches!(
            selector_kind,
            TreeCalcWorkspaceSelectorKind::CurrentWorkspaceRoot
        ) {
            let entry = self
                .workspaces_by_handle
                .get(&self.current_workspace_handle)
                .expect("current workspace must be registered");
            return TreeCalcWorkspaceResolutionPacket {
                selector_token_text: selector_token_text.to_string(),
                selector_kind,
                workspace_handle: entry.workspace_handle.clone(),
                resolution_layer: TreeCalcWorkspaceResolutionLayer::CurrentWorkspace,
                availability_packet: TreeCalcCrossWorkspaceAvailabilityPacket::available(
                    entry.workspace_handle.clone(),
                    selector_token_text,
                    entry.availability_version.clone(),
                ),
            };
        }

        let direct_selector = unquoted_workspace_selector(selector_token_text);
        let alias_target = self.aliases.get(direct_selector);
        let resolved_handle = alias_target
            .cloned()
            .unwrap_or_else(|| direct_selector.to_string());
        let resolution_layer = match selector_kind {
            TreeCalcWorkspaceSelectorKind::QuotedDirectPath => {
                TreeCalcWorkspaceResolutionLayer::QuotedWorkspacePath
            }
            TreeCalcWorkspaceSelectorKind::AliasOrDirectPath if alias_target.is_some() => {
                TreeCalcWorkspaceResolutionLayer::WorkspaceAlias
            }
            TreeCalcWorkspaceSelectorKind::AliasOrDirectPath => {
                TreeCalcWorkspaceResolutionLayer::DirectWorkspacePath
            }
            TreeCalcWorkspaceSelectorKind::CurrentWorkspaceRoot => {
                TreeCalcWorkspaceResolutionLayer::CurrentWorkspace
            }
        };

        let availability_packet = self.workspaces_by_handle.get(&resolved_handle).map_or_else(
            || {
                TreeCalcCrossWorkspaceAvailabilityPacket::unavailable(
                    resolved_handle.clone(),
                    selector_token_text,
                    format!(
                        "treecalc-cross-workspace-availability:v1:{resolved_handle}:unavailable"
                    ),
                    "workspace is not loaded or no registered alias matched this selector",
                )
            },
            |entry| {
                TreeCalcCrossWorkspaceAvailabilityPacket::available(
                    entry.workspace_handle.clone(),
                    selector_token_text,
                    entry.availability_version.clone(),
                )
            },
        );

        TreeCalcWorkspaceResolutionPacket {
            selector_token_text: selector_token_text.to_string(),
            selector_kind,
            workspace_handle: resolved_handle,
            resolution_layer,
            availability_packet,
        }
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum TreeCalcWorkspaceHostPathBaseResolutionError {
    #[error("{0}")]
    LocalPath(#[from] TreeCalcHostPathBaseResolutionError),
    #[error("TreeCalc workspace selector '{selector_token_text}' has invalid syntax: {detail}")]
    InvalidWorkspaceSelectorSyntax {
        selector_token_text: String,
        detail: String,
    },
    #[error(
        "TreeCalc workspace selector '{selector_token_text}' is unavailable for workspace handle '{workspace_handle}'"
    )]
    WorkspaceUnavailable {
        selector_token_text: String,
        workspace_handle: String,
        availability_packet: Box<TreeCalcCrossWorkspaceAvailabilityPacket>,
    },
}

pub fn resolve_treecalc_workspace_host_path_base(
    registry: &TreeCalcWorkspaceResolutionRegistry<'_>,
    base_token_text: impl AsRef<str>,
) -> Result<TreeCalcWorkspaceHostPathBaseResolution, TreeCalcWorkspaceHostPathBaseResolutionError> {
    let base_token_text = base_token_text.as_ref().trim();
    if base_token_text.is_empty() {
        return Err(TreeCalcHostPathBaseResolutionError::EmptyBaseToken.into());
    }

    let parsed = parse_treecalc_workspace_base_token(base_token_text)?;
    if let Some(selector_token_text) = parsed.selector_token_text.as_deref()
        && parsed.selector_kind == TreeCalcWorkspaceSelectorKind::AliasOrDirectPath
        && !registry.aliases.contains_key(selector_token_text)
        && !registry
            .workspaces_by_handle
            .contains_key(selector_token_text)
        && !looks_like_direct_workspace_selector(selector_token_text)
    {
        let workspace = registry
            .resolve_workspace_selector("", TreeCalcWorkspaceSelectorKind::CurrentWorkspaceRoot);
        let snapshot = registry
            .workspace_snapshot(&workspace.workspace_handle)
            .expect("current workspace must expose a snapshot");
        let local_path_token_text =
            format!("[{selector_token_text}]{}", parsed.local_path_token_text);
        let local = resolve_treecalc_explicit_host_path_base(snapshot, &local_path_token_text)?;
        let base_node_handle = format!("{}#{}", workspace.workspace_handle, local.base_node_id);
        let resolution_identity = format!(
            "treecalc-workspace-host-path:v1:workspace={};workspace_layer={:?};availability={};local={};bracket_fallback=escaped_current_workspace_path",
            workspace.workspace_handle,
            workspace.resolution_layer,
            workspace.availability_packet.availability_version,
            local.resolution_identity
        );

        return Ok(TreeCalcWorkspaceHostPathBaseResolution {
            base_token_text: base_token_text.to_string(),
            workspace_selector_token: None,
            local_path_token_text,
            workspace_handle: workspace.workspace_handle,
            base_node_id: local.base_node_id,
            base_node_handle,
            canonical_projection_path: local.canonical_projection_path,
            workspace_resolution_layer: workspace.resolution_layer,
            local_resolution_layer: local.resolution_layer,
            availability_packet: workspace.availability_packet,
            resolution_identity,
        });
    }

    let workspace = registry.resolve_workspace_selector(
        parsed.selector_token_text.as_deref().unwrap_or(""),
        parsed.selector_kind,
    );
    if workspace.availability_packet.status != TreeCalcCrossWorkspaceAvailabilityStatus::Available {
        return Err(
            TreeCalcWorkspaceHostPathBaseResolutionError::WorkspaceUnavailable {
                selector_token_text: workspace.selector_token_text,
                workspace_handle: workspace.workspace_handle,
                availability_packet: Box::new(workspace.availability_packet),
            },
        );
    }

    let snapshot = registry
        .workspace_snapshot(&workspace.workspace_handle)
        .expect("available workspace must expose a snapshot");
    let local = if parsed.local_path_token_text.is_empty() {
        host_path_base_resolution(
            snapshot,
            "",
            snapshot.root_node_id(),
            TreeCalcHostPathBaseResolutionLayer::WorkspaceRoot,
        )?
    } else if parsed.selector_token_text.is_some() {
        resolve_treecalc_workspace_root_path_base(snapshot, &parsed.local_path_token_text)?
    } else {
        resolve_treecalc_explicit_host_path_base(snapshot, &parsed.local_path_token_text)?
    };

    let base_node_handle = format!("{}#{}", workspace.workspace_handle, local.base_node_id);
    let resolution_identity = format!(
        "treecalc-workspace-host-path:v1:workspace={};workspace_layer={:?};availability={};local={}",
        workspace.workspace_handle,
        workspace.resolution_layer,
        workspace.availability_packet.availability_version,
        local.resolution_identity
    );

    Ok(TreeCalcWorkspaceHostPathBaseResolution {
        base_token_text: base_token_text.to_string(),
        workspace_selector_token: parsed.selector_token_text,
        local_path_token_text: parsed.local_path_token_text,
        workspace_handle: workspace.workspace_handle,
        base_node_id: local.base_node_id,
        base_node_handle,
        canonical_projection_path: local.canonical_projection_path,
        workspace_resolution_layer: workspace.resolution_layer,
        local_resolution_layer: local.resolution_layer,
        availability_packet: workspace.availability_packet,
        resolution_identity,
    })
}

pub fn resolve_treecalc_explicit_host_path_base(
    snapshot: &StructuralSnapshot,
    base_token_text: impl AsRef<str>,
) -> Result<TreeCalcHostPathBaseResolution, TreeCalcHostPathBaseResolutionError> {
    let base_token_text = base_token_text.as_ref().trim();
    if base_token_text.is_empty() {
        return Err(TreeCalcHostPathBaseResolutionError::EmptyBaseToken);
    }

    if let Some(node_id) = snapshot.try_resolve_projection_path(base_token_text) {
        return host_path_base_resolution(
            snapshot,
            base_token_text,
            node_id,
            TreeCalcHostPathBaseResolutionLayer::ProjectionPath,
        );
    }

    let segments = split_treecalc_host_path_token(base_token_text)?;
    if segments.is_empty() {
        return Err(TreeCalcHostPathBaseResolutionError::UnresolvedBaseToken {
            base_token_text: base_token_text.to_string(),
        });
    }

    let dotted_projection_path = segments.join("/");
    if let Some(node_id) = snapshot.try_resolve_projection_path(&dotted_projection_path) {
        return host_path_base_resolution(
            snapshot,
            base_token_text,
            node_id,
            TreeCalcHostPathBaseResolutionLayer::DottedProjectionPath,
        );
    }

    let node_id = snapshot
        .try_resolve_descendant_path(snapshot.root_node_id(), &segments)
        .ok_or_else(
            || TreeCalcHostPathBaseResolutionError::UnresolvedBaseToken {
                base_token_text: base_token_text.to_string(),
            },
        )?;
    host_path_base_resolution(
        snapshot,
        base_token_text,
        node_id,
        TreeCalcHostPathBaseResolutionLayer::RootDescendantPath,
    )
}

fn host_path_base_resolution(
    snapshot: &StructuralSnapshot,
    base_token_text: &str,
    base_node_id: TreeNodeId,
    resolution_layer: TreeCalcHostPathBaseResolutionLayer,
) -> Result<TreeCalcHostPathBaseResolution, TreeCalcHostPathBaseResolutionError> {
    let canonical_projection_path = snapshot.get_projection_path(base_node_id).map_err(|_| {
        TreeCalcHostPathBaseResolutionError::UnresolvedBaseToken {
            base_token_text: base_token_text.to_string(),
        }
    })?;
    let resolution_identity = format!(
        "treecalc-explicit-host-path:v1:token={base_token_text};canonical={canonical_projection_path};base={base_node_id};layer={resolution_layer:?}"
    );
    Ok(TreeCalcHostPathBaseResolution {
        base_token_text: base_token_text.to_string(),
        base_node_id,
        canonical_projection_path,
        resolution_layer,
        resolution_identity,
    })
}

fn resolve_treecalc_workspace_root_path_base(
    snapshot: &StructuralSnapshot,
    base_token_text: &str,
) -> Result<TreeCalcHostPathBaseResolution, TreeCalcHostPathBaseResolutionError> {
    let segments = split_treecalc_host_path_token(base_token_text)?;
    if segments.is_empty() {
        return host_path_base_resolution(
            snapshot,
            base_token_text,
            snapshot.root_node_id(),
            TreeCalcHostPathBaseResolutionLayer::WorkspaceRoot,
        );
    }

    if let Some(node_id) = snapshot.try_resolve_descendant_path(snapshot.root_node_id(), &segments)
    {
        return host_path_base_resolution(
            snapshot,
            base_token_text,
            node_id,
            TreeCalcHostPathBaseResolutionLayer::RootDescendantPath,
        );
    }

    let projection_path = segments.join("/");
    if let Some(node_id) = snapshot.try_resolve_projection_path(&projection_path) {
        return host_path_base_resolution(
            snapshot,
            base_token_text,
            node_id,
            TreeCalcHostPathBaseResolutionLayer::ProjectionPath,
        );
    }

    Err(TreeCalcHostPathBaseResolutionError::UnresolvedBaseToken {
        base_token_text: base_token_text.to_string(),
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedTreeCalcWorkspaceBaseToken {
    selector_token_text: Option<String>,
    selector_kind: TreeCalcWorkspaceSelectorKind,
    local_path_token_text: String,
}

fn parse_treecalc_workspace_base_token(
    base_token_text: &str,
) -> Result<ParsedTreeCalcWorkspaceBaseToken, TreeCalcWorkspaceHostPathBaseResolutionError> {
    if !base_token_text.starts_with('[') {
        return Ok(ParsedTreeCalcWorkspaceBaseToken {
            selector_token_text: None,
            selector_kind: TreeCalcWorkspaceSelectorKind::CurrentWorkspaceRoot,
            local_path_token_text: base_token_text.to_string(),
        });
    }

    let selector_end = base_token_text.find(']').ok_or_else(|| {
        TreeCalcWorkspaceHostPathBaseResolutionError::InvalidWorkspaceSelectorSyntax {
            selector_token_text: base_token_text.to_string(),
            detail: "unclosed workspace selector".to_string(),
        }
    })?;
    let selector_token_text = &base_token_text[1..selector_end];
    let selector_kind = if selector_token_text.is_empty() {
        TreeCalcWorkspaceSelectorKind::CurrentWorkspaceRoot
    } else if is_quoted_workspace_selector(selector_token_text) {
        TreeCalcWorkspaceSelectorKind::QuotedDirectPath
    } else {
        TreeCalcWorkspaceSelectorKind::AliasOrDirectPath
    };

    Ok(ParsedTreeCalcWorkspaceBaseToken {
        selector_token_text: Some(selector_token_text.to_string()),
        selector_kind,
        local_path_token_text: base_token_text[(selector_end + 1)..].to_string(),
    })
}

fn is_quoted_workspace_selector(selector_token_text: &str) -> bool {
    selector_token_text.len() >= 2
        && selector_token_text.starts_with('\'')
        && selector_token_text.ends_with('\'')
}

fn unquoted_workspace_selector(selector_token_text: &str) -> &str {
    if is_quoted_workspace_selector(selector_token_text) {
        &selector_token_text[1..(selector_token_text.len() - 1)]
    } else {
        selector_token_text
    }
}

fn looks_like_direct_workspace_selector(selector_token_text: &str) -> bool {
    let lower = selector_token_text.to_ascii_lowercase();
    lower.ends_with(".dnatree")
        || lower.ends_with(".xlsx")
        || selector_token_text.contains(':')
        || selector_token_text.contains('\\')
        || selector_token_text.contains('/')
}

pub fn split_treecalc_host_path_token(
    base_token_text: &str,
) -> Result<Vec<String>, TreeCalcHostPathBaseResolutionError> {
    let mut segments = Vec::new();
    let mut segment = String::new();
    let mut in_bracket = false;
    let mut bracket_escape = false;
    for ch in base_token_text.chars() {
        if in_bracket {
            if bracket_escape {
                segment.push(ch);
                bracket_escape = false;
                continue;
            }
            match ch {
                '\'' => {
                    bracket_escape = true;
                }
                ']' => {
                    in_bracket = false;
                }
                _ => segment.push(ch),
            }
            continue;
        }

        match ch {
            '[' => {
                in_bracket = true;
            }
            ']' => {
                return Err(
                    TreeCalcHostPathBaseResolutionError::InvalidBaseTokenSyntax {
                        base_token_text: base_token_text.to_string(),
                        detail: "closing bracket without an open bracket".to_string(),
                    },
                );
            }
            '.' | '/' => {
                push_treecalc_host_path_segment(&mut segments, &mut segment);
            }
            '!' => {
                if !segments.is_empty() || segment.trim().is_empty() {
                    return Err(
                        TreeCalcHostPathBaseResolutionError::InvalidBaseTokenSyntax {
                            base_token_text: base_token_text.to_string(),
                            detail: "'!' is only admitted after the first path segment".to_string(),
                        },
                    );
                }
                push_treecalc_host_path_segment(&mut segments, &mut segment);
            }
            _ => segment.push(ch),
        }
    }
    if bracket_escape {
        return Err(
            TreeCalcHostPathBaseResolutionError::InvalidBaseTokenSyntax {
                base_token_text: base_token_text.to_string(),
                detail: "dangling bracket escape".to_string(),
            },
        );
    }
    if in_bracket {
        return Err(
            TreeCalcHostPathBaseResolutionError::InvalidBaseTokenSyntax {
                base_token_text: base_token_text.to_string(),
                detail: "unclosed bracketed path segment".to_string(),
            },
        );
    }
    push_treecalc_host_path_segment(&mut segments, &mut segment);
    Ok(segments)
}

fn push_treecalc_host_path_segment(segments: &mut Vec<String>, segment: &mut String) {
    let trimmed = segment.trim();
    if !trimmed.is_empty() {
        segments.push(unquote_treecalc_host_path_segment(trimmed));
    }
    segment.clear();
}

fn unquote_treecalc_host_path_segment(segment: &str) -> String {
    segment
        .strip_prefix('\'')
        .and_then(|rest| rest.strip_suffix('\''))
        .or_else(|| {
            segment
                .strip_prefix('"')
                .and_then(|rest| rest.strip_suffix('"'))
        })
        .unwrap_or(segment)
        .to_string()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreeCalcOrderedSelectorTraversalPolicy {
    pub max_recursive_descendants: usize,
}

impl Default for TreeCalcOrderedSelectorTraversalPolicy {
    fn default() -> Self {
        Self {
            max_recursive_descendants: 10_000,
        }
    }
}

impl TreeCalcOrderedSelectorTraversalPolicy {
    #[must_use]
    pub fn stable_id(self) -> String {
        format!(
            "treecalc-traversal-bound:v1:max_recursive_descendants={}",
            self.max_recursive_descendants
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TreeCalcOrderedSelectorTraversalDiagnosticCode {
    RecursiveTailMatchedNoMembers,
    TailMatchedNoMembers,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreeCalcOrderedSelectorTraversalDiagnostic {
    pub code: TreeCalcOrderedSelectorTraversalDiagnosticCode,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreeCalcOrderedSelectorTraversalResult {
    pub member_node_ids: Vec<TreeNodeId>,
    pub diagnostics: Vec<TreeCalcOrderedSelectorTraversalDiagnostic>,
    pub traversal_policy_id: String,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum TreeCalcOrderedSelectorTraversalError {
    #[error("ordered selector base node {base_node_id} is not present in the structural snapshot")]
    UnknownBaseNode { base_node_id: TreeNodeId },
    #[error(
        "recursive ordered selector from {base_node_id} exceeded traversal policy {policy_id} after visiting {visited_count} descendants"
    )]
    RecursiveTraversalLimitExceeded {
        base_node_id: TreeNodeId,
        policy_id: String,
        visited_count: usize,
    },
}

pub fn resolve_treecalc_ordered_selector_traversal(
    snapshot: &StructuralSnapshot,
    family: TreeCalcOrderedSelectorFamily,
    base_node_id: TreeNodeId,
    tail_segments: &[String],
    policy: TreeCalcOrderedSelectorTraversalPolicy,
) -> Result<TreeCalcOrderedSelectorTraversalResult, TreeCalcOrderedSelectorTraversalError> {
    let Some(base_node) = snapshot.try_get_node(base_node_id) else {
        return Err(TreeCalcOrderedSelectorTraversalError::UnknownBaseNode { base_node_id });
    };
    let traversal_policy_id = policy.stable_id();
    let mut diagnostics = Vec::new();
    let member_node_ids = match family {
        TreeCalcOrderedSelectorFamily::SiblingSetV1 => apply_ordered_selector_tail(
            snapshot,
            base_node_id,
            sibling_window(snapshot, base_node_id, None),
            tail_segments,
            &mut diagnostics,
        ),
        TreeCalcOrderedSelectorFamily::PrecedingV1 => apply_ordered_selector_tail(
            snapshot,
            base_node_id,
            sibling_window(snapshot, base_node_id, Some(SiblingWindow::Preceding)),
            tail_segments,
            &mut diagnostics,
        ),
        TreeCalcOrderedSelectorFamily::FollowingV1 => apply_ordered_selector_tail(
            snapshot,
            base_node_id,
            sibling_window(snapshot, base_node_id, Some(SiblingWindow::Following)),
            tail_segments,
            &mut diagnostics,
        ),
        TreeCalcOrderedSelectorFamily::AncestorsV1 => {
            let mut ancestors = Vec::new();
            let mut cursor = base_node.parent_id;
            while let Some(parent_id) = cursor {
                if parent_id == snapshot.root_node_id() {
                    break;
                }
                ancestors.push(parent_id);
                cursor = snapshot
                    .try_get_node(parent_id)
                    .and_then(|node| node.parent_id);
            }
            apply_ordered_selector_tail(
                snapshot,
                base_node_id,
                ancestors,
                tail_segments,
                &mut diagnostics,
            )
        }
        TreeCalcOrderedSelectorFamily::RecursiveDescendantsV1 => {
            let descendants =
                recursive_descendants_preorder(snapshot, base_node_id, base_node, policy)?;
            if tail_segments.is_empty() {
                descendants
            } else {
                let members = std::iter::once(base_node_id)
                    .chain(descendants)
                    .filter_map(|descendant_id| {
                        snapshot.try_resolve_descendant_path(descendant_id, tail_segments)
                    })
                    .collect::<Vec<_>>();
                if members.is_empty() {
                    diagnostics.push(TreeCalcOrderedSelectorTraversalDiagnostic {
                        code: TreeCalcOrderedSelectorTraversalDiagnosticCode::RecursiveTailMatchedNoMembers,
                        detail: format!(
                            "recursive selector tail '{}' matched no members below {base_node_id}",
                            tail_segments.join(".")
                        ),
                    });
                }
                members
            }
        }
    };

    Ok(TreeCalcOrderedSelectorTraversalResult {
        member_node_ids,
        diagnostics,
        traversal_policy_id,
    })
}

/// Resolve a dotted tail (e.g. `@PRECEDING.Margin`) under each ordered-selector member.
/// Members that do not have the tail path resolve to nothing and are dropped; an empty
/// result records a diagnostic. Mirrors the tail handling already done for recursive descent.
fn apply_ordered_selector_tail(
    snapshot: &StructuralSnapshot,
    base_node_id: TreeNodeId,
    members: Vec<TreeNodeId>,
    tail_segments: &[String],
    diagnostics: &mut Vec<TreeCalcOrderedSelectorTraversalDiagnostic>,
) -> Vec<TreeNodeId> {
    if tail_segments.is_empty() {
        return members;
    }
    let resolved = members
        .into_iter()
        .filter_map(|member_id| snapshot.try_resolve_descendant_path(member_id, tail_segments))
        .collect::<Vec<_>>();
    if resolved.is_empty() {
        diagnostics.push(TreeCalcOrderedSelectorTraversalDiagnostic {
            code: TreeCalcOrderedSelectorTraversalDiagnosticCode::TailMatchedNoMembers,
            detail: format!(
                "ordered selector tail '{}' matched no members from {base_node_id}",
                tail_segments.join(".")
            ),
        });
    }
    resolved
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SiblingWindow {
    Preceding,
    Following,
}

fn sibling_window(
    snapshot: &StructuralSnapshot,
    base_node_id: TreeNodeId,
    window: Option<SiblingWindow>,
) -> Vec<TreeNodeId> {
    let Some(parent_id) = snapshot.parent_id_of(base_node_id) else {
        return Vec::new();
    };
    let Some(parent) = snapshot.try_get_node(parent_id) else {
        return Vec::new();
    };
    let Some(base_index) = parent
        .child_ids
        .iter()
        .position(|node_id| *node_id == base_node_id)
    else {
        return Vec::new();
    };

    match window {
        None => parent
            .child_ids
            .iter()
            .copied()
            .filter(|node_id| *node_id != base_node_id)
            .collect(),
        Some(SiblingWindow::Preceding) => parent.child_ids[..base_index].to_vec(),
        Some(SiblingWindow::Following) => parent.child_ids[base_index + 1..].to_vec(),
    }
}

fn recursive_descendants_preorder(
    snapshot: &StructuralSnapshot,
    base_node_id: TreeNodeId,
    base_node: &crate::structural::StructuralNode,
    policy: TreeCalcOrderedSelectorTraversalPolicy,
) -> Result<Vec<TreeNodeId>, TreeCalcOrderedSelectorTraversalError> {
    let mut descendants = Vec::new();
    let mut stack = base_node
        .child_ids
        .iter()
        .rev()
        .copied()
        .collect::<Vec<_>>();
    while let Some(node_id) = stack.pop() {
        if descendants.len() >= policy.max_recursive_descendants {
            return Err(
                TreeCalcOrderedSelectorTraversalError::RecursiveTraversalLimitExceeded {
                    base_node_id,
                    policy_id: policy.stable_id(),
                    visited_count: descendants.len() + 1,
                },
            );
        }
        descendants.push(node_id);
        if let Some(node) = snapshot.try_get_node(node_id) {
            stack.extend(node.child_ids.iter().rev().copied());
        }
    }
    Ok(descendants)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreeReferenceCarrierClass {
    FormulaReference,
    RuntimeFactProjection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TreeReferenceInventoryVariant {
    DirectNode,
    ChildrenV1,
    ReferenceLiteralArray,
    MixedReferenceArray,
    ProjectionPath,
    RelativePathSelf,
    RelativePathParent,
    RelativePathAncestor,
    SiblingOffset,
    ExplicitPath,
    DynamicPotential,
    DynamicResolved,
    HostSensitive,
    CapabilitySensitive,
    ShapeTopology,
    Unresolved,
    SiblingSetSelector,
    PrecedingFollowingSelector,
    AncestorSetSelector,
    CrossWorkspaceReference,
    RecursiveSelector,
    StructuredTableReference,
    BareNameOrCallableReference,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TreeReferenceInventoryStatus {
    AdmittedCurrentCarrier,
    AdmittedImplementationInput,
    TypedExclusion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TreeReferenceInventoryBlocker {
    NeedsResolvableHostReference,
    NeedsOxFmlNameCallPrecedenceEvidence,
    NeedsOxFmlStructuredReferencePacket,
    NeedsStableStructuredTableRowMembershipAndOrderPacket,
    NeedsCrossWorkspaceModel,
    NeedsCrossWorkspaceProvider,
    NeedsWorkspaceQualifiedCarrier,
    NeedsSelectorDependencyModel,
    NeedsReferenceOnlyArrayCarrier,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HostReferenceCorrelationNeed {
    None,
    SourceTokenToFormalReference,
    HostReferenceHandle,
    RuntimeFactCarrier,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NamespaceIdentityNeed {
    None,
    StructureContextVersion,
    HostNamespaceVersion,
    ResolutionRuleVersion,
    CapabilityProfileVersion,
    CrossWorkspaceAvailabilityVersion,
    TableContextIdentity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CallerContextIdentityNeed {
    None,
    CallerNode,
    AncestorWalk,
    SiblingPosition,
    HostRuntimeContext,
    TableCallerRegion,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeReferenceImplementationInput {
    pub variant: TreeReferenceInventoryVariant,
    pub status: TreeReferenceInventoryStatus,
    pub blocker: Option<TreeReferenceInventoryBlocker>,
    pub carrier_class: Option<TreeReferenceCarrierClass>,
    pub host_reference_correlation: HostReferenceCorrelationNeed,
    pub namespace_identity_need: NamespaceIdentityNeed,
    pub caller_context_identity_need: CallerContextIdentityNeed,
    pub dependency_facts: Vec<DependencyDescriptorKind>,
    pub invalidation_facts: Vec<InvalidationReasonKind>,
    pub successor_bead: Option<&'static str>,
    pub evidence_note: &'static str,
}

impl TreeReferenceImplementationInput {
    #[must_use]
    pub fn is_admitted(&self) -> bool {
        matches!(
            self.status,
            TreeReferenceInventoryStatus::AdmittedCurrentCarrier
                | TreeReferenceInventoryStatus::AdmittedImplementationInput
        )
    }
}

#[must_use]
pub fn tree_reference_implementation_inputs() -> Vec<TreeReferenceImplementationInput> {
    use CallerContextIdentityNeed as Caller;
    use DependencyDescriptorKind as Dep;
    use HostReferenceCorrelationNeed as Correlation;
    use InvalidationReasonKind as Invalidates;
    use NamespaceIdentityNeed as Namespace;
    use TreeReferenceInventoryBlocker as Blocker;
    use TreeReferenceInventoryStatus as Status;
    use TreeReferenceInventoryVariant as Variant;

    vec![
        TreeReferenceImplementationInput {
            variant: Variant::DirectNode,
            status: Status::AdmittedCurrentCarrier,
            blocker: None,
            carrier_class: Some(TreeReferenceCarrierClass::FormulaReference),
            host_reference_correlation: Correlation::SourceTokenToFormalReference,
            namespace_identity_need: Namespace::StructureContextVersion,
            caller_context_identity_need: Caller::None,
            dependency_facts: vec![Dep::StaticDirect],
            invalidation_facts: vec![
                Invalidates::StructuralRecalcOnly,
                Invalidates::UpstreamPublication,
            ],
            successor_bead: None,
            evidence_note: "existing direct node carrier and static dependency lowering",
        },
        TreeReferenceImplementationInput {
            variant: Variant::ChildrenV1,
            status: Status::AdmittedCurrentCarrier,
            blocker: None,
            carrier_class: Some(TreeReferenceCarrierClass::FormulaReference),
            host_reference_correlation: Correlation::HostReferenceHandle,
            namespace_identity_need: Namespace::HostNamespaceVersion,
            caller_context_identity_need: Caller::CallerNode,
            dependency_facts: vec![
                Dep::TreeReferenceCollectionMembership,
                Dep::TreeReferenceCollectionMemberValue,
            ],
            invalidation_facts: vec![
                Invalidates::TreeReferenceMembershipChanged,
                Invalidates::TreeReferenceOrderChanged,
                Invalidates::UpstreamPublication,
            ],
            successor_bead: None,
            evidence_note: "W051 ChildrenV1 collection handle with membership and member-value facts",
        },
        TreeReferenceImplementationInput {
            variant: Variant::ReferenceLiteralArray,
            status: Status::AdmittedImplementationInput,
            blocker: None,
            carrier_class: Some(TreeReferenceCarrierClass::FormulaReference),
            host_reference_correlation: Correlation::HostReferenceHandle,
            namespace_identity_need: Namespace::HostNamespaceVersion,
            caller_context_identity_need: Caller::CallerNode,
            dependency_facts: vec![
                Dep::TreeReferenceCollectionMembership,
                Dep::TreeReferenceCollectionMemberValue,
            ],
            invalidation_facts: vec![
                Invalidates::TreeReferenceMembershipChanged,
                Invalidates::TreeReferenceOrderChanged,
                Invalidates::UpstreamPublication,
            ],
            successor_bead: Some("calc-4vs8.31"),
            evidence_note: "calc-4vs8.31 adds reference-literal array collection carriers that preserve authored order and duplicates",
        },
        TreeReferenceImplementationInput {
            variant: Variant::MixedReferenceArray,
            status: Status::TypedExclusion,
            blocker: Some(Blocker::NeedsReferenceOnlyArrayCarrier),
            carrier_class: Some(TreeReferenceCarrierClass::FormulaReference),
            host_reference_correlation: Correlation::HostReferenceHandle,
            namespace_identity_need: Namespace::StructureContextVersion,
            caller_context_identity_need: Caller::CallerNode,
            dependency_facts: vec![Dep::Unresolved],
            invalidation_facts: vec![Invalidates::StructuralRebindRequired],
            successor_bead: Some("calc-4vs8.31"),
            evidence_note: "mixed scalar/reference array literals are typed rejects until scalar/value mixing has an explicit generic reference contract",
        },
        TreeReferenceImplementationInput {
            variant: Variant::ProjectionPath,
            status: Status::AdmittedImplementationInput,
            blocker: None,
            carrier_class: Some(TreeReferenceCarrierClass::FormulaReference),
            host_reference_correlation: Correlation::SourceTokenToFormalReference,
            namespace_identity_need: Namespace::HostNamespaceVersion,
            caller_context_identity_need: Caller::None,
            dependency_facts: vec![Dep::StaticDirect],
            invalidation_facts: vec![
                Invalidates::StructuralRebindRequired,
                Invalidates::UpstreamPublication,
            ],
            successor_bead: Some("calc-4vs8.12"),
            evidence_note: "explicit path carrier exists; wider rebind/invalidation closure belongs to W056 dependency widening",
        },
        TreeReferenceImplementationInput {
            variant: Variant::RelativePathSelf,
            status: Status::AdmittedImplementationInput,
            blocker: None,
            carrier_class: Some(TreeReferenceCarrierClass::FormulaReference),
            host_reference_correlation: Correlation::SourceTokenToFormalReference,
            namespace_identity_need: Namespace::HostNamespaceVersion,
            caller_context_identity_need: Caller::CallerNode,
            dependency_facts: vec![Dep::RelativeBound],
            invalidation_facts: vec![
                Invalidates::StructuralRebindRequired,
                Invalidates::UpstreamPublication,
            ],
            successor_bead: Some("calc-4vs8.12"),
            evidence_note: "self-relative carrier is representable but remains W056 input until exercised",
        },
        TreeReferenceImplementationInput {
            variant: Variant::RelativePathParent,
            status: Status::AdmittedCurrentCarrier,
            blocker: None,
            carrier_class: Some(TreeReferenceCarrierClass::FormulaReference),
            host_reference_correlation: Correlation::SourceTokenToFormalReference,
            namespace_identity_need: Namespace::HostNamespaceVersion,
            caller_context_identity_need: Caller::CallerNode,
            dependency_facts: vec![Dep::RelativeBound],
            invalidation_facts: vec![
                Invalidates::StructuralRebindRequired,
                Invalidates::UpstreamPublication,
            ],
            successor_bead: Some("calc-4vs8.12"),
            evidence_note: "parent-relative carrier and descriptor lowering exist; W056 widens replay-visible invalidation",
        },
        TreeReferenceImplementationInput {
            variant: Variant::RelativePathAncestor,
            status: Status::AdmittedCurrentCarrier,
            blocker: None,
            carrier_class: Some(TreeReferenceCarrierClass::FormulaReference),
            host_reference_correlation: Correlation::SourceTokenToFormalReference,
            namespace_identity_need: Namespace::HostNamespaceVersion,
            caller_context_identity_need: Caller::AncestorWalk,
            dependency_facts: vec![Dep::RelativeBound],
            invalidation_facts: vec![
                Invalidates::StructuralRebindRequired,
                Invalidates::UpstreamPublication,
            ],
            successor_bead: Some("calc-4vs8.12"),
            evidence_note: "ancestor-relative carrier and descriptor lowering exist; W056 widens replay-visible invalidation",
        },
        TreeReferenceImplementationInput {
            variant: Variant::SiblingOffset,
            status: Status::AdmittedCurrentCarrier,
            blocker: None,
            carrier_class: Some(TreeReferenceCarrierClass::FormulaReference),
            host_reference_correlation: Correlation::SourceTokenToFormalReference,
            namespace_identity_need: Namespace::HostNamespaceVersion,
            caller_context_identity_need: Caller::SiblingPosition,
            dependency_facts: vec![Dep::RelativeBound],
            invalidation_facts: vec![
                Invalidates::StructuralRebindRequired,
                Invalidates::UpstreamPublication,
            ],
            successor_bead: Some("calc-4vs8.3"),
            evidence_note: "sibling-offset carrier and descriptor lowering exist; sibling membership/order rebind is W056 widening input",
        },
        TreeReferenceImplementationInput {
            variant: Variant::ExplicitPath,
            status: Status::AdmittedImplementationInput,
            blocker: None,
            carrier_class: Some(TreeReferenceCarrierClass::FormulaReference),
            host_reference_correlation: Correlation::HostReferenceHandle,
            namespace_identity_need: Namespace::ResolutionRuleVersion,
            caller_context_identity_need: Caller::None,
            dependency_facts: vec![Dep::StaticDirect],
            invalidation_facts: vec![
                Invalidates::StructuralRebindRequired,
                Invalidates::UpstreamPublication,
            ],
            successor_bead: Some("calc-4vs8.3"),
            evidence_note: "explicit path syntax remains an OxFml generic host-reference input; OxCalc owns resolved dependency facts",
        },
        TreeReferenceImplementationInput {
            variant: Variant::DynamicPotential,
            status: Status::AdmittedCurrentCarrier,
            blocker: None,
            carrier_class: Some(TreeReferenceCarrierClass::RuntimeFactProjection),
            host_reference_correlation: Correlation::RuntimeFactCarrier,
            namespace_identity_need: Namespace::StructureContextVersion,
            caller_context_identity_need: Caller::HostRuntimeContext,
            dependency_facts: vec![Dep::DynamicPotential],
            invalidation_facts: vec![
                Invalidates::DynamicDependencyActivated,
                Invalidates::DynamicDependencyReclassified,
            ],
            successor_bead: Some("calc-4vs8.3"),
            evidence_note: "unresolved dynamic dependency is a typed runtime fact and no-publish diagnostic input",
        },
        TreeReferenceImplementationInput {
            variant: Variant::DynamicResolved,
            status: Status::AdmittedCurrentCarrier,
            blocker: None,
            carrier_class: Some(TreeReferenceCarrierClass::FormulaReference),
            host_reference_correlation: Correlation::SourceTokenToFormalReference,
            namespace_identity_need: Namespace::StructureContextVersion,
            caller_context_identity_need: Caller::HostRuntimeContext,
            dependency_facts: vec![Dep::DynamicPotential],
            invalidation_facts: vec![
                Invalidates::DynamicDependencyReleased,
                Invalidates::DynamicDependencyReclassified,
                Invalidates::UpstreamPublication,
            ],
            successor_bead: Some("calc-4vs8.3"),
            evidence_note: "resolved dynamic dependency publishes a target edge and remains rebind-sensitive",
        },
        TreeReferenceImplementationInput {
            variant: Variant::HostSensitive,
            status: Status::AdmittedCurrentCarrier,
            blocker: None,
            carrier_class: Some(TreeReferenceCarrierClass::RuntimeFactProjection),
            host_reference_correlation: Correlation::RuntimeFactCarrier,
            namespace_identity_need: Namespace::HostNamespaceVersion,
            caller_context_identity_need: Caller::HostRuntimeContext,
            dependency_facts: vec![Dep::HostSensitive],
            invalidation_facts: vec![
                Invalidates::StructuralRebindRequired,
                Invalidates::ExternallyInvalidated,
            ],
            successor_bead: Some("calc-4vs8.3"),
            evidence_note: "host-sensitive fact is surfaced as dependency diagnostic until a concrete host provider admits it",
        },
        TreeReferenceImplementationInput {
            variant: Variant::CapabilitySensitive,
            status: Status::AdmittedCurrentCarrier,
            blocker: None,
            carrier_class: Some(TreeReferenceCarrierClass::RuntimeFactProjection),
            host_reference_correlation: Correlation::RuntimeFactCarrier,
            namespace_identity_need: Namespace::CapabilityProfileVersion,
            caller_context_identity_need: Caller::HostRuntimeContext,
            dependency_facts: vec![Dep::CapabilitySensitive],
            invalidation_facts: vec![
                Invalidates::DependencyReclassified,
                Invalidates::ExternallyInvalidated,
            ],
            successor_bead: Some("calc-4vs8.3"),
            evidence_note: "capability-sensitive fact remains typed and must not be collapsed into generic failure",
        },
        TreeReferenceImplementationInput {
            variant: Variant::ShapeTopology,
            status: Status::AdmittedCurrentCarrier,
            blocker: None,
            carrier_class: Some(TreeReferenceCarrierClass::RuntimeFactProjection),
            host_reference_correlation: Correlation::RuntimeFactCarrier,
            namespace_identity_need: Namespace::StructureContextVersion,
            caller_context_identity_need: Caller::HostRuntimeContext,
            dependency_facts: vec![Dep::ShapeTopology],
            invalidation_facts: vec![
                Invalidates::DependencyReclassified,
                Invalidates::StructuralRebindRequired,
            ],
            successor_bead: Some("calc-4vs8.3"),
            evidence_note: "shape/topology fact remains typed and replay-visible as dependency-sensitive input",
        },
        TreeReferenceImplementationInput {
            variant: Variant::Unresolved,
            status: Status::TypedExclusion,
            blocker: Some(Blocker::NeedsResolvableHostReference),
            carrier_class: Some(TreeReferenceCarrierClass::FormulaReference),
            host_reference_correlation: Correlation::SourceTokenToFormalReference,
            namespace_identity_need: Namespace::HostNamespaceVersion,
            caller_context_identity_need: Caller::CallerNode,
            dependency_facts: vec![Dep::Unresolved],
            invalidation_facts: vec![Invalidates::StructuralRebindRequired],
            successor_bead: Some("calc-4vs8.3"),
            evidence_note: "unresolved references are explicit descriptors, not silent fallback behavior",
        },
        TreeReferenceImplementationInput {
            variant: Variant::SiblingSetSelector,
            status: Status::AdmittedImplementationInput,
            blocker: None,
            carrier_class: Some(TreeReferenceCarrierClass::FormulaReference),
            host_reference_correlation: Correlation::HostReferenceHandle,
            namespace_identity_need: Namespace::HostNamespaceVersion,
            caller_context_identity_need: Caller::SiblingPosition,
            dependency_facts: vec![
                Dep::TreeReferenceCollectionMembership,
                Dep::TreeReferenceCollectionMemberValue,
            ],
            invalidation_facts: vec![
                Invalidates::TreeReferenceMembershipChanged,
                Invalidates::TreeReferenceOrderChanged,
                Invalidates::UpstreamPublication,
            ],
            successor_bead: Some("calc-4vs8.3"),
            evidence_note: "calc-4vs8.12 adds ordered selector collection dependency carriers for resolved sibling-set packets",
        },
        TreeReferenceImplementationInput {
            variant: Variant::PrecedingFollowingSelector,
            status: Status::AdmittedImplementationInput,
            blocker: None,
            carrier_class: Some(TreeReferenceCarrierClass::FormulaReference),
            host_reference_correlation: Correlation::HostReferenceHandle,
            namespace_identity_need: Namespace::HostNamespaceVersion,
            caller_context_identity_need: Caller::SiblingPosition,
            dependency_facts: vec![
                Dep::TreeReferenceCollectionMembership,
                Dep::TreeReferenceCollectionMemberValue,
            ],
            invalidation_facts: vec![
                Invalidates::TreeReferenceMembershipChanged,
                Invalidates::TreeReferenceOrderChanged,
                Invalidates::UpstreamPublication,
            ],
            successor_bead: Some("calc-4vs8.3"),
            evidence_note: "calc-4vs8.12 adds ordered selector collection dependency carriers for resolved preceding/following packets",
        },
        TreeReferenceImplementationInput {
            variant: Variant::AncestorSetSelector,
            status: Status::AdmittedImplementationInput,
            blocker: None,
            carrier_class: Some(TreeReferenceCarrierClass::FormulaReference),
            host_reference_correlation: Correlation::HostReferenceHandle,
            namespace_identity_need: Namespace::HostNamespaceVersion,
            caller_context_identity_need: Caller::AncestorWalk,
            dependency_facts: vec![
                Dep::TreeReferenceCollectionMembership,
                Dep::TreeReferenceCollectionMemberValue,
            ],
            invalidation_facts: vec![
                Invalidates::TreeReferenceMembershipChanged,
                Invalidates::TreeReferenceOrderChanged,
                Invalidates::UpstreamPublication,
            ],
            successor_bead: Some("calc-4vs8.12"),
            evidence_note: "calc-4vs8.12 adds ordered selector collection dependency carriers for resolved ancestor-set packets",
        },
        TreeReferenceImplementationInput {
            variant: Variant::CrossWorkspaceReference,
            status: Status::AdmittedImplementationInput,
            blocker: None,
            carrier_class: Some(TreeReferenceCarrierClass::FormulaReference),
            host_reference_correlation: Correlation::HostReferenceHandle,
            namespace_identity_need: Namespace::CrossWorkspaceAvailabilityVersion,
            caller_context_identity_need: Caller::None,
            dependency_facts: vec![Dep::HostSensitive],
            invalidation_facts: vec![
                Invalidates::StructuralRebindRequired,
                Invalidates::ExternallyInvalidated,
            ],
            successor_bead: Some("calc-8tox"),
            evidence_note: "calc-8tox adds typed workspace-qualified carriers and reverse-edge facts over the calc-4vs8.30 provider/alias packet; receiving-side corpus activation remains W056 evidence work",
        },
        TreeReferenceImplementationInput {
            variant: Variant::RecursiveSelector,
            status: Status::AdmittedImplementationInput,
            blocker: None,
            carrier_class: Some(TreeReferenceCarrierClass::FormulaReference),
            host_reference_correlation: Correlation::HostReferenceHandle,
            namespace_identity_need: Namespace::HostNamespaceVersion,
            caller_context_identity_need: Caller::CallerNode,
            dependency_facts: vec![
                Dep::TreeReferenceCollectionMembership,
                Dep::TreeReferenceCollectionMemberValue,
            ],
            invalidation_facts: vec![
                Invalidates::TreeReferenceMembershipChanged,
                Invalidates::TreeReferenceOrderChanged,
                Invalidates::UpstreamPublication,
            ],
            successor_bead: Some("calc-4vs8.12"),
            evidence_note: "calc-4vs8.12 adds resolved recursive-descendant collection dependency carriers; traversal bounds remain resolver/corpus scope",
        },
        TreeReferenceImplementationInput {
            variant: Variant::StructuredTableReference,
            status: Status::AdmittedImplementationInput,
            blocker: None,
            carrier_class: Some(TreeReferenceCarrierClass::FormulaReference),
            host_reference_correlation: Correlation::HostReferenceHandle,
            namespace_identity_need: Namespace::TableContextIdentity,
            caller_context_identity_need: Caller::TableCallerRegion,
            dependency_facts: vec![
                Dep::StructuredTableIdentity,
                Dep::StructuredTableRowMembership,
                Dep::StructuredTableRowOrder,
                Dep::StructuredTableColumnIdentity,
                Dep::StructuredTableHeaderText,
                Dep::StructuredTableHeaderRegion,
                Dep::StructuredTableDataRegion,
                Dep::StructuredTableTotalsRegion,
                Dep::StructuredTableCallerContext,
                Dep::StructuredTableEnclosingTable,
            ],
            invalidation_facts: vec![
                Invalidates::StructuredTableContextChanged,
                Invalidates::StructuredTableRowMembershipChanged,
                Invalidates::StructuredTableRowOrderChanged,
                Invalidates::StructuredTableColumnChanged,
                Invalidates::StructuredTableRegionChanged,
                Invalidates::StructuredTableCallerContextChanged,
            ],
            successor_bead: Some("calc-4vs8.2"),
            evidence_note: "calc-4vs8.2/calc-4vs8.4/calc-4vs8.9 add table-context lowering from public OxFml table packets and bind records, including stable row/order and exact region facts",
        },
        TreeReferenceImplementationInput {
            variant: Variant::BareNameOrCallableReference,
            status: Status::AdmittedImplementationInput,
            blocker: None,
            carrier_class: Some(TreeReferenceCarrierClass::FormulaReference),
            host_reference_correlation: Correlation::SourceTokenToFormalReference,
            namespace_identity_need: Namespace::HostNamespaceVersion,
            caller_context_identity_need: Caller::CallerNode,
            dependency_facts: vec![Dep::Unresolved],
            invalidation_facts: vec![Invalidates::StructuralRebindRequired],
            successor_bead: Some("calc-4vs8.32"),
            evidence_note: "calc-4vs8.32 consumes the closed OxFml W074 handoff: host values map to the defined-name lane and lambda-valued host nodes map to the defined-name-LAMBDA lane without an OxCalc precedence mirror",
        },
    ]
}

#[must_use]
pub fn tree_reference_implementation_input(
    variant: TreeReferenceInventoryVariant,
) -> Option<TreeReferenceImplementationInput> {
    tree_reference_implementation_inputs()
        .into_iter()
        .find(|input| input.variant == variant)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum W056NonTableReferenceEvidenceStatus {
    ProductGreen,
    DirectContextSliceGreen,
    DirectContextTypedPending,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct W056NonTableReferenceCategory {
    pub category_id: &'static str,
    pub reference_family: &'static str,
    pub examples: &'static [&'static str],
    pub spec_anchor: &'static str,
    pub expected_outcome_contract: &'static str,
    pub corpus_or_suite: &'static [&'static str],
    pub runnable_suite_command: &'static str,
    pub oxcalc_status: &'static str,
    pub dnatreecalc_status: &'static str,
    pub replay_status: &'static str,
    pub current_test_result: &'static str,
    pub evidence_status: W056NonTableReferenceEvidenceStatus,
    pub specification_is_sufficient_for_cases: bool,
    pub blocks_w056_non_table_closure: bool,
}

pub const W056_NON_TABLE_REFERENCE_CATEGORIES: &[W056NonTableReferenceCategory] = &[
    W056NonTableReferenceCategory {
        category_id: "children_collection",
        reference_family: "children collection references",
        examples: &["@CHILDREN", ".*", "base.@CHILDREN", "base.*"],
        spec_anchor: "DnaTreeCalc CORE_MODEL_SPEC.md §3.5/§3.5b/§3.7; OxCalc W051/W056 ChildrenV1",
        expected_outcome_contract: "ordered regular children only, meta children excluded, stable membership/order dependency facts, sparse ReferenceLike values",
        corpus_or_suite: &[
            "../DnaTreeCalc/docs/test-corpus/references/children-raw-active.json",
            "oxcalc-core treecalc::children_collection_sum_uses_generic_host_context_and_sparse_reference_values",
        ],
        runnable_suite_command: "cargo test -p dnatreecalc-host --test active_children_corpus -- --nocapture",
        oxcalc_status: "implemented and green as ChildrenV1, including direct OxCalcTreeContext raw @CHILDREN/.* product path",
        dnatreecalc_status: "active direct OxCalcTreeContext slice",
        replay_status: "no retained non-table replay requirement yet",
        current_test_result: "green in prior DnaTreeCalc active runner, OxCalc W051 tests, and direct-context raw children tests",
        evidence_status: W056NonTableReferenceEvidenceStatus::ProductGreen,
        specification_is_sufficient_for_cases: true,
        blocks_w056_non_table_closure: false,
    },
    W056NonTableReferenceCategory {
        category_id: "walkup_dotted_descent",
        reference_family: "bare walk-up and dotted descent",
        examples: &["Margin", "Q1.Margin", "A.B.C"],
        spec_anchor: "DnaTreeCalc CORE_MODEL_SPEC.md §3.2/§3.7; references/walkup",
        expected_outcome_contract: "nearest lexical scope wins, then dotted child descent from the resolved base; unresolved names emit typed diagnostics",
        corpus_or_suite: &[
            "../DnaTreeCalc/docs/test-corpus/references/walkup.json",
            "src/dnatreecalc-host/tests/active_walkup_corpus.rs",
        ],
        runnable_suite_command: "cargo test -p dnatreecalc-host --test active_walkup_corpus -- --nocapture",
        oxcalc_status: "direct OxCalcTreeContext raw formula binding resolves bare walk-up and dotted descent through the host-name bind lane; RelativePath carriers still cover lower-level fixture paths",
        dnatreecalc_status: "focused active direct OxCalcTreeContext raw slice; full Q1/cell-like precedence cases remain pending",
        replay_status: "retained non-table replay missing",
        current_test_result: "green active walk-up raw direct-context runner plus OxCalc direct-context walk-up/dotted test",
        evidence_status: W056NonTableReferenceEvidenceStatus::DirectContextSliceGreen,
        specification_is_sufficient_for_cases: true,
        blocks_w056_non_table_closure: true,
    },
    W056NonTableReferenceCategory {
        category_id: "ancestor_root_anchors",
        reference_family: "ancestor and root/current-workspace anchors",
        examples: &["^.Rate", "^^.Year", "[]Sheet1.Margin", "[]"],
        spec_anchor: "DnaTreeCalc CORE_MODEL_SPEC.md §3.1/§3.2/§3.7; references/anchors",
        expected_outcome_contract: "fixed-depth ancestor navigation or workspace-root absolute lookup with #REF!/unresolved diagnostics on invalid use",
        corpus_or_suite: &[
            "../DnaTreeCalc/docs/test-corpus/references/anchors-raw-active.json",
            "../DnaTreeCalc/docs/test-corpus/references/anchors.json",
        ],
        runnable_suite_command: "cargo test -p dnatreecalc-host --test active_anchor_corpus -- --nocapture",
        oxcalc_status: "ancestor RelativePath carriers and direct OxCalcTreeContext raw ancestor-anchor binding exist; workspace-root/alias anchor breadth remains pending",
        dnatreecalc_status: "focused active direct OxCalcTreeContext ancestor-anchor slice; broader workspace-root anchor corpus pending",
        replay_status: "retained non-table replay missing",
        current_test_result: "green active ancestor-anchor direct-context runner; broader anchors still validator-only",
        evidence_status: W056NonTableReferenceEvidenceStatus::DirectContextSliceGreen,
        specification_is_sufficient_for_cases: true,
        blocks_w056_non_table_closure: true,
    },
    W056NonTableReferenceCategory {
        category_id: "workspace_alias_bang_syntax",
        reference_family: "workspace selectors, aliases, and first-position !",
        examples: &[
            "[projections]Branch1.MyNode",
            "Sheet1!Foo",
            "[ws][Branch X].MyNode",
        ],
        spec_anchor: "DnaTreeCalc CORE_MODEL_SPEC.md §3.1/§3.3; calc-4vs8.30",
        expected_outcome_contract: "alias-first workspace resolution, first-position ! as sheet separator only, unavailable workspace diagnostics and availability-version identity",
        corpus_or_suite: &[
            "../DnaTreeCalc/docs/test-corpus/references/cross-workspace.json",
            "oxcalc-core formula::workspace_host_path_base_resolver_uses_aliases_availability_and_rooted_paths",
        ],
        runnable_suite_command: "cargo test -p dnatreecalc-host --test active_dynamic_cross_workspace_corpus -- --nocapture",
        oxcalc_status: "provider/alias packet and workspace-qualified carrier implemented",
        dnatreecalc_status: "active direct OxCalcTreeContext typed-pending/exclusion slice for raw cross-workspace syntax",
        replay_status: "retained non-table replay missing",
        current_test_result: "green direct-context typed-pending runner; product runtime and retained evidence still open",
        evidence_status: W056NonTableReferenceEvidenceStatus::DirectContextTypedPending,
        specification_is_sufficient_for_cases: true,
        blocks_w056_non_table_closure: true,
    },
    W056NonTableReferenceCategory {
        category_id: "escaping_canonicalization_case",
        reference_family: "bracket escaping, case-insensitive lookup, canonical display paths",
        examples: &["[Sales Q1].Margin", "[][Sales Q1].Margin", "sales.margin"],
        spec_anchor: "DnaTreeCalc CORE_MODEL_SPEC.md §3.3/§3.4; references/escaping",
        expected_outcome_contract: "escaped segments preserve literal names, lookup is case-insensitive, output reports canonical display paths and ambiguity diagnostics",
        corpus_or_suite: &[
            "../DnaTreeCalc/docs/test-corpus/references/escaping-raw-active.json",
            "../DnaTreeCalc/docs/test-corpus/references/escaping.json",
        ],
        runnable_suite_command: "cargo test -p dnatreecalc-host --test active_escaping_corpus -- --nocapture",
        oxcalc_status: "structural path base resolver covers bracketed path segments and direct OxCalcTreeContext raw escaped paths for the focused slice",
        dnatreecalc_status: "focused active direct OxCalcTreeContext escaped-path slice; broader canonical display/profile coverage pending",
        replay_status: "retained non-table replay missing",
        current_test_result: "green active escaped-path direct-context runner; broader escaping corpus still pending",
        evidence_status: W056NonTableReferenceEvidenceStatus::DirectContextSliceGreen,
        specification_is_sufficient_for_cases: true,
        blocks_w056_non_table_closure: true,
    },
    W056NonTableReferenceCategory {
        category_id: "meta_invisibility_accessors",
        reference_family: "meta-node invisibility and @ accessors",
        examples: &[
            "@NAME",
            "ref.@INDEX",
            "ref.@PARENT",
            "hidden meta child lookup",
        ],
        spec_anchor: "DnaTreeCalc CORE_MODEL_SPEC.md §2/§3.5/§6 item 9; references/meta-nodes",
        expected_outcome_contract: "meta-effective subtrees are invisible to formula resolution and positional operators; accessors return typed metadata scalars (scalar terminals: a trailing path such as @NAME.x is a kept typed-exclusion, not navigation) or #REF!",
        corpus_or_suite: &[
            "../DnaTreeCalc/docs/test-corpus/references/meta-nodes.json",
            "src/dnatreecalc-host/tests/active_meta_nodes_corpus.rs",
        ],
        runnable_suite_command: "cargo test -p dnatreecalc-host --test active_meta_nodes_corpus -- --nocapture",
        oxcalc_status: "direct OxCalcTreeContext owns is_meta state, hides meta-effective nodes from host-name lookup and sibling navigation, and exposes current metadata accessors for the active slice",
        dnatreecalc_status: "focused active direct OxCalcTreeContext meta invisibility/accessor slice; broader ordered selectors over meta-effective snapshots pending",
        replay_status: "retained non-table replay missing",
        current_test_result: "green active meta-node direct-context runner, including @PARENT, @NAME, @INDEX, and @FORMULA; broader meta snapshot/replay evidence pending",
        evidence_status: W056NonTableReferenceEvidenceStatus::DirectContextSliceGreen,
        specification_is_sufficient_for_cases: true,
        blocks_w056_non_table_closure: true,
    },
    W056NonTableReferenceCategory {
        category_id: "sibling_single_navigation",
        reference_family: "single sibling navigation",
        examples: &["@PREV.Net", "@NEXT.Margin", "ref.@PREV"],
        spec_anchor: "DnaTreeCalc CORE_MODEL_SPEC.md §3.5/§3.5b/§3.7; references/sibling-offsets",
        expected_outcome_contract: "previous/next regular sibling lookup by sibling order with meta nodes skipped and #REF! on out-of-range",
        corpus_or_suite: &[
            "../DnaTreeCalc/docs/test-corpus/references/sibling-offsets.json",
            "src/dnatreecalc-host/tests/active_sibling_offsets_corpus.rs",
        ],
        runnable_suite_command: "cargo test -p dnatreecalc-host --test active_sibling_offsets_corpus -- --nocapture",
        oxcalc_status: "SiblingOffset and QualifiedSiblingOffset carrier/dependency lowering exist; direct OxCalcTreeContext raw @PREV/@NEXT and base.@PREV/base.@NEXT product paths are implemented for focused tail forms and out-of-range relative-bound diagnostics",
        dnatreecalc_status: "active direct OxCalcTreeContext sibling-offset slice covers unqualified tail forms, qualified base tail forms, and out-of-range diagnostics",
        replay_status: "retained non-table replay missing",
        current_test_result: "green active sibling-offset direct-context runner plus OxCalc qualified sibling direct-context test; retained replay pending",
        evidence_status: W056NonTableReferenceEvidenceStatus::DirectContextSliceGreen,
        specification_is_sufficient_for_cases: true,
        blocks_w056_non_table_closure: true,
    },
    W056NonTableReferenceCategory {
        category_id: "ordered_set_selectors",
        reference_family: "ordered sibling and ancestor set selectors",
        examples: &["@PRECEDING", "@FOLLOWING", "@ANCESTORS"],
        spec_anchor: "DnaTreeCalc CORE_MODEL_SPEC.md §3.5/§3.5b/§3.7; references/ordered-raw-active and references/set-membership",
        expected_outcome_contract: "ordered reference collections preserve defined traversal order, exclude meta-effective nodes, and carry membership/order/member-value dependency facts",
        corpus_or_suite: &[
            "../DnaTreeCalc/docs/test-corpus/references/ordered-raw-active.json",
            "../DnaTreeCalc/docs/test-corpus/references/set-membership-active.json",
            "../DnaTreeCalc/docs/test-corpus/references/set-membership.json",
        ],
        runnable_suite_command: "cargo test -p dnatreecalc-host --test active_ordered_corpus --test active_set_membership_corpus -- --nocapture",
        oxcalc_status: "resolved ordered selector carriers, traversal resolver, bounds, and direct OxCalcTreeContext raw @PRECEDING/@FOLLOWING/@ANCESTORS product path implemented, including dotted-tail forms (e.g. @PRECEDING.Margin) resolved under each member",
        dnatreecalc_status: "active direct OxCalcTreeContext ordered-selector slice plus broad raw set-membership mirror for Q1.*, children, ancestors, preceding/following, empty preceding, and recursive all/match forms",
        replay_status: "retained non-table replay missing",
        current_test_result: "green active ordered-selector and set-membership direct-context runners assert OxCalc collection descriptor order through DnaTreeCalc collection projection; retained replay pending",
        evidence_status: W056NonTableReferenceEvidenceStatus::DirectContextSliceGreen,
        specification_is_sufficient_for_cases: true,
        blocks_w056_non_table_closure: true,
    },
    W056NonTableReferenceCategory {
        category_id: "recursive_descent",
        reference_family: "recursive descent",
        examples: &["**.Margin", "Base.**.Margin"],
        spec_anchor: "DnaTreeCalc CORE_MODEL_SPEC.md §3.5b/§3.6/§3.7; references/ordered-raw-active and references/set-membership-active",
        expected_outcome_contract: "stable depth-first preorder descendant traversal, optional tail filtering, traversal-bound diagnostics, and membership/order facts",
        corpus_or_suite: &[
            "../DnaTreeCalc/docs/test-corpus/references/ordered-raw-active.json",
            "../DnaTreeCalc/docs/test-corpus/references/set-membership-active.json",
            "oxcalc-core formula::ordered_selector_traversal_resolver_projects_structural_membership",
        ],
        runnable_suite_command: "cargo test -p dnatreecalc-host --test active_ordered_corpus --test active_set_membership_corpus -- --nocapture",
        oxcalc_status: "recursive ordered selector carrier, traversal-bound resolver, and direct OxCalcTreeContext Base.**.tail product path implemented",
        dnatreecalc_status: "active direct OxCalcTreeContext recursive-selector slice plus broad set-membership mirror for absolute recursive tail and relative all-descendant forms",
        replay_status: "retained non-table replay missing",
        current_test_result: "green active recursive-selector and set-membership direct-context runners plus OxCalc direct-context recursive selector test; traversal-bound replay pending",
        evidence_status: W056NonTableReferenceEvidenceStatus::DirectContextSliceGreen,
        specification_is_sufficient_for_cases: true,
        blocks_w056_non_table_closure: true,
    },
    W056NonTableReferenceCategory {
        category_id: "reference_literals_arrays",
        reference_family: "reference literals and reference arrays",
        examples: &["{A, C, A}", "{A, 1}", "array-valued node reference"],
        spec_anchor: "DnaTreeCalc CORE_MODEL_SPEC.md §3.5b/§6 item 4; references/literals, references/literals-active, arrays/array-references",
        expected_outcome_contract: "reference-only literals preserve authored order and duplicates; mixed scalar/reference arrays are typed rejections until a generic mixing contract exists",
        corpus_or_suite: &[
            "../DnaTreeCalc/docs/test-corpus/references/literals-active.json",
            "../DnaTreeCalc/docs/test-corpus/references/literals.json",
            "../DnaTreeCalc/docs/test-corpus/arrays/array-references.json",
        ],
        runnable_suite_command: "cargo test -p dnatreecalc-host --test active_reference_literals_corpus -- --nocapture",
        oxcalc_status: "ReferenceLiteralArrayV1 implemented; direct OxCalcTreeContext resolves all-reference arrays through host-reference packets; mixed scalar/reference arrays remain typed exclusions",
        dnatreecalc_status: "focused active direct OxCalcTreeContext reference-only array slice; broader arrays corpus pending",
        replay_status: "retained non-table replay missing",
        current_test_result: "green active reference-literal direct-context runner; broad raw arrays suite pending",
        evidence_status: W056NonTableReferenceEvidenceStatus::DirectContextSliceGreen,
        specification_is_sufficient_for_cases: true,
        blocks_w056_non_table_closure: true,
    },
    W056NonTableReferenceCategory {
        category_id: "dynamic_indirect_ctro",
        reference_family: "dynamic INDIRECT and CTRO references",
        examples: &[
            "INDIRECT(\"Sheet1!Foo\")",
            "INDIRECT(selector_node)",
            "dynamic target switch",
        ],
        spec_anchor: "DnaTreeCalc CORE_MODEL_SPEC.md §4/§6 item 7/§10.3; dynamic-references/indirect",
        expected_outcome_contract: "runtime string/path resolution activates, releases, or rejects dynamic dependency facts atomically with prepared identity and diagnostics",
        corpus_or_suite: &[
            "../DnaTreeCalc/docs/test-corpus/dynamic-references/indirect.json",
            "oxcalc-core treecalc dynamic dependency tests",
        ],
        runnable_suite_command: "cargo test -p dnatreecalc-host --test active_dynamic_cross_workspace_corpus -- --nocapture",
        oxcalc_status: "Direct OxCalcTreeContext resolves basic raw INDIRECT reference text through the OxFml/OxFunc FEC resolver and records CTRO dynamic dependency edges; broader CTRO cases remain open",
        dnatreecalc_status: "active direct OxCalcTreeContext dynamic INDIRECT corpus migration pending beyond OxCalc basic proof",
        replay_status: "retained non-table replay missing",
        current_test_result: "green OxCalc direct-context basic INDIRECT proof for INDIRECT(\"B\"&A); dynamic corpus and retained evidence still open",
        evidence_status: W056NonTableReferenceEvidenceStatus::DirectContextSliceGreen,
        specification_is_sufficient_for_cases: true,
        blocks_w056_non_table_closure: true,
    },
    W056NonTableReferenceCategory {
        category_id: "cross_workspace_runtime",
        reference_family: "cross-workspace runtime references",
        examples: &[
            "[accounts]Revenue",
            "[Other.xlsx]Sheet1!Foo",
            "[projections]Branch1.MyNode",
        ],
        spec_anchor: "DnaTreeCalc CORE_MODEL_SPEC.md §3.3/§10.4; references/cross-workspace; calc-8tox",
        expected_outcome_contract: "loaded external workspaces resolve live-latest through opaque workspace-qualified handles; unavailable workspaces fail without stale values",
        corpus_or_suite: &[
            "../DnaTreeCalc/docs/test-corpus/references/cross-workspace.json",
            "oxcalc-core workspace_qualified_runtime_binding_does_not_read_local_node_id_collision",
        ],
        runnable_suite_command: "cargo test -p dnatreecalc-host --test active_dynamic_cross_workspace_corpus -- --nocapture",
        oxcalc_status: "workspace-qualified carriers, reverse edges, and prepared identity implemented; direct OxCalcTreeContext records raw cross-workspace host names as typed pending",
        dnatreecalc_status: "active direct OxCalcTreeContext typed-pending/exclusion slice for raw cross-workspace runtime references",
        replay_status: "retained non-table replay missing",
        current_test_result: "green direct-context typed-pending runner; product runtime and retained evidence still open",
        evidence_status: W056NonTableReferenceEvidenceStatus::DirectContextTypedPending,
        specification_is_sufficient_for_cases: true,
        blocks_w056_non_table_closure: true,
    },
    W056NonTableReferenceCategory {
        category_id: "bare_host_names_defined_name_lane",
        reference_family: "bare host names through the defined-name lane",
        examples: &["=Revenue", "=Margin + 1", "=My.Region.Sales"],
        spec_anchor: "DnaTreeCalc CORE_MODEL_SPEC.md §3.2/§3.9; OxFml W074 handoff consumed by calc-4vs8.32",
        expected_outcome_contract: "TreeCalc host values map to the Excel defined-name value lane with source span/token, namespace version, caller context, diagnostics, and replay identity",
        corpus_or_suite: &[
            "../DnaTreeCalc/docs/test-corpus/references/walkup.json",
            "OxFml fml-ds0.6.3/fml-ds0.6.5 runtime host-name tests",
        ],
        runnable_suite_command: "cargo test -p oxcalc-core w056_inventory_names_admitted_reference_inputs_and_typed_exclusions -- --nocapture",
        oxcalc_status: "W074 handoff consumed; direct OxCalcTreeContext host-name bind lane maps TreeCalc names to the defined-name value lane with source token/span and replay identity; no OxCalc private precedence mirror",
        dnatreecalc_status: "focused active direct OxCalcTreeContext non-cell-like host-name slice; broader name/cell-like precedence corpus pending",
        replay_status: "retained non-table replay missing",
        current_test_result: "OxFml/OxCalc intake green and DnaTreeCalc active walk-up raw direct-context runner green; broader name/cell-like cases pending",
        evidence_status: W056NonTableReferenceEvidenceStatus::DirectContextSliceGreen,
        specification_is_sufficient_for_cases: true,
        blocks_w056_non_table_closure: true,
    },
    W056NonTableReferenceCategory {
        category_id: "node_as_function_lambda",
        reference_family: "node-as-function and lambda-valued host nodes",
        examples: &["Doubler(5)", "My.Node(1, 2)", "^.Rate(x)"],
        spec_anchor: "DnaTreeCalc CORE_MODEL_SPEC.md §3.8/§3.9; references/node-functions",
        expected_outcome_contract: "single-reference lambda-valued nodes are callable through the defined-name-LAMBDA lane; set-valued callees reject",
        corpus_or_suite: &["../DnaTreeCalc/docs/test-corpus/references/node-functions.json"],
        runnable_suite_command: "cargo test -p dnatreecalc-host --test active_node_functions_corpus -- --nocapture",
        oxcalc_status: "current defined-name-LAMBDA mapping admitted by calc-4vs8.32; direct OxCalcTreeContext records callable host names as typed pending/exclusion until the product lane is implemented",
        dnatreecalc_status: "active direct OxCalcTreeContext typed-pending/exclusion runner; callable product runtime pending",
        replay_status: "retained non-table replay missing",
        current_test_result: "green direct-context typed-pending runner; no active callable product runner yet",
        evidence_status: W056NonTableReferenceEvidenceStatus::DirectContextTypedPending,
        specification_is_sufficient_for_cases: true,
        blocks_w056_non_table_closure: true,
    },
    W056NonTableReferenceCategory {
        category_id: "profile_gating",
        reference_family: "TreeCalc capability profile gating",
        examples: &[
            "treecalc-v1 accepts @ANCESTORS",
            "strict-excel rejects .*/^/[ws]",
        ],
        spec_anchor: "DnaTreeCalc CORE_MODEL_SPEC.md §4; profiles/gating",
        expected_outcome_contract: "TreeCalc-only syntax is admitted only under host-capabilities:treecalc-v1 and rejected under host-capabilities:strict-excel",
        corpus_or_suite: &["../DnaTreeCalc/docs/test-corpus/profiles/gating.json"],
        runnable_suite_command: "cargo test -p dnatreecalc-host --test active_profile_gating_corpus -- --nocapture",
        oxcalc_status: "capability-profile identity participates in prepared identity; strict-excel INDIRECT currently returns an explicit profile-not-supported typed exclusion",
        dnatreecalc_status: "active direct OxCalcTreeContext typed-pending runner",
        replay_status: "retained non-table replay missing",
        current_test_result: "green direct-context typed-pending runner; strict-excel INDIRECT(\"Sheet1!Foo\") rejects with typed_exclusion:strict_excel_profile_not_supported until the future Excel-compatible profile is implemented",
        evidence_status: W056NonTableReferenceEvidenceStatus::DirectContextTypedPending,
        specification_is_sufficient_for_cases: true,
        blocks_w056_non_table_closure: true,
    },
    W056NonTableReferenceCategory {
        category_id: "structural_edit_rebind",
        reference_family: "structural edit rebind and propagation",
        examples: &[
            "rename target",
            "move target",
            "delete target",
            "reorder siblings",
        ],
        spec_anchor: "DnaTreeCalc CORE_MODEL_SPEC.md §8a; structural-edits/propagation; OxCalc structural invalidation tests",
        expected_outcome_contract: "edits classify as value-only recalculation, rebind-required, unresolved, or propagation prompt with replay-visible invalidation facts",
        corpus_or_suite: &[
            "../DnaTreeCalc/docs/test-corpus/structural-edits/propagation.json",
            "src/dnatreecalc-host/tests/active_structural_edits_corpus.rs",
            "oxcalc-core structural_invalidation_seeds_mark_relative_reference_rebind_after_rename",
        ],
        runnable_suite_command: "cargo test -p dnatreecalc-host --test active_structural_edits_corpus -- --nocapture",
        oxcalc_status: "structural rebind facts implemented for current carriers",
        dnatreecalc_status: "active direct OxCalcTreeContext structural edit runner covers delete, rename propagation/no-propagation, move-out-of-scope, and insert-shadow consequences",
        replay_status: "retained non-table replay missing",
        current_test_result: "green active structural-edit direct-context runner; retained invalidation/replay evidence still pending",
        evidence_status: W056NonTableReferenceEvidenceStatus::DirectContextSliceGreen,
        specification_is_sufficient_for_cases: true,
        blocks_w056_non_table_closure: true,
    },
    W056NonTableReferenceCategory {
        category_id: "unresolved_diagnostics_self_reference",
        reference_family: "unresolved, invalid, and self-reference diagnostics",
        examples: &["MissingName", "[]", "self reference through walk-up"],
        spec_anchor: "DnaTreeCalc CORE_MODEL_SPEC.md §3.2/§3.5b/§7; references/walkup and references/syntax",
        expected_outcome_contract: "unresolved references do not silently bind; invalid naked navigation and self-reference produce typed diagnostics/cycle outcomes",
        corpus_or_suite: &[
            "../DnaTreeCalc/docs/test-corpus/references/walkup.json",
            "../DnaTreeCalc/docs/test-corpus/references/syntax.json",
        ],
        runnable_suite_command: "cargo test -p dnatreecalc-host --test active_walkup_corpus -- --nocapture",
        oxcalc_status: "unresolved descriptors and cycle diagnostics exist",
        dnatreecalc_status: "walk-up active slice covers unresolved/self-reference; broader syntax pending",
        replay_status: "retained non-table replay missing",
        current_test_result: "green active walk-up diagnostics; broad syntax runner pending",
        evidence_status: W056NonTableReferenceEvidenceStatus::DirectContextSliceGreen,
        specification_is_sufficient_for_cases: true,
        blocks_w056_non_table_closure: true,
    },
];

#[must_use]
pub fn w056_non_table_reference_category(
    category_id: &str,
) -> Option<&'static W056NonTableReferenceCategory> {
    W056_NON_TABLE_REFERENCE_CATEGORIES
        .iter()
        .find(|category| category.category_id == category_id)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreeFormulaHostNameBindPacket {
    pub host_name_handle: String,
    pub canonical_name: String,
    pub host_dependency_key: Option<String>,
    pub source_span_utf8: (usize, usize),
    pub source_token_text: String,
    pub resolution_layer: String,
    pub binding_kind: String,
    pub shape_hint: Option<String>,
    pub caller_context_dependent: bool,
    pub diagnostics: Vec<String>,
    pub replay_identity_contribution: String,
}

impl TreeFormulaHostNameBindPacket {
    #[must_use]
    pub fn direct_tree_node(
        workspace_id: &str,
        owner_node_id: TreeNodeId,
        target_node_id: TreeNodeId,
        canonical_name: impl Into<String>,
        source_span_utf8: (usize, usize),
        source_token_text: impl Into<String>,
    ) -> Self {
        let canonical_name = canonical_name.into();
        let source_token_text = source_token_text.into();
        let host_name_handle = format!(
            "treecalc-hostname:v1:workspace={workspace_id}:owner={}:target={}:name={}",
            owner_node_id.0, target_node_id.0, canonical_name
        );
        Self {
            replay_identity_contribution: format!(
                "treecalc-host-name:v1:handle={host_name_handle};token={source_token_text}"
            ),
            host_name_handle,
            canonical_name,
            host_dependency_key: Some(treecalc_node_host_dependency_key(target_node_id)),
            source_span_utf8,
            source_token_text,
            resolution_layer: "treecalc_host_name".to_string(),
            binding_kind: "defined_name_value_like".to_string(),
            shape_hint: Some("scalar_node_value".to_string()),
            caller_context_dependent: false,
            diagnostics: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TreeFormulaHostValue {
    Text(String),
    Integer(i64),
    ValueError,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreeFormulaHostValueBinding {
    pub source_token: String,
    pub value: TreeFormulaHostValue,
    pub host_ref_handle: String,
    pub source_span_utf8: (usize, usize),
    pub source_token_text: String,
    pub opaque_selector: Option<String>,
    pub carrier_detail: String,
    pub target_node_id: Option<TreeNodeId>,
    pub requires_rebind_on_structural_change: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreeFormula {
    pub source_text: String,
    #[serde(default)]
    pub explicit_references: Vec<TreeReference>,
    #[serde(default)]
    pub host_value_bindings: Vec<TreeFormulaHostValueBinding>,
    #[serde(default, skip)]
    pub bound_formula: Option<BoundFormula>,
    #[serde(default)]
    pub lazy_residual_publication: bool,
}

impl TreeFormula {
    #[must_use]
    pub fn opaque_oxfml(
        source_text: impl Into<String>,
        explicit_references: impl IntoIterator<Item = TreeReference>,
    ) -> Self {
        Self {
            source_text: normalize_formula_source(source_text.into()),
            explicit_references: explicit_references.into_iter().collect(),
            host_value_bindings: Vec::new(),
            bound_formula: None,
            lazy_residual_publication: false,
        }
    }

    #[must_use]
    pub fn with_lazy_residual_publication(mut self, lazy_residual_publication: bool) -> Self {
        self.lazy_residual_publication = lazy_residual_publication;
        self
    }

    #[must_use]
    pub fn with_host_value_bindings(
        mut self,
        host_value_bindings: impl IntoIterator<Item = TreeFormulaHostValueBinding>,
    ) -> Self {
        self.host_value_bindings = host_value_bindings.into_iter().collect();
        self
    }

    #[must_use]
    pub fn with_bound_formula(mut self, bound_formula: Option<BoundFormula>) -> Self {
        self.bound_formula = bound_formula;
        self
    }

    #[must_use]
    pub fn source_text(&self) -> &str {
        &self.source_text
    }

    #[must_use]
    pub fn explicit_references(&self) -> &[TreeReference] {
        &self.explicit_references
    }

    #[must_use]
    pub fn host_value_bindings(&self) -> &[TreeFormulaHostValueBinding] {
        &self.host_value_bindings
    }

    #[must_use]
    pub fn bound_formula(&self) -> Option<&BoundFormula> {
        self.bound_formula.as_ref()
    }
}

#[must_use]
pub fn treecalc_node_host_dependency_key(target_node_id: TreeNodeId) -> String {
    format!("treecalc-node:{}", target_node_id.0)
}

#[must_use]
pub fn treecalc_node_id_from_host_dependency_key(key: &str) -> Option<TreeNodeId> {
    key.strip_prefix("treecalc-node:")
        .and_then(|value| value.parse::<u64>().ok())
        .map(TreeNodeId)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeCalcCollectionHostDependencyKey {
    pub family: String,
    pub base_node_id: TreeNodeId,
    pub member_node_ids: Vec<TreeNodeId>,
}

#[must_use]
pub fn treecalc_collection_host_dependency_key(
    family: &str,
    base_node_id: TreeNodeId,
    member_node_ids: &[TreeNodeId],
) -> String {
    let members = member_node_ids
        .iter()
        .map(|node_id| node_id.0.to_string())
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "treecalc-collection:family={family};base={};members={members}",
        base_node_id.0
    )
}

#[must_use]
pub fn treecalc_collection_from_host_dependency_key(
    key: &str,
) -> Option<TreeCalcCollectionHostDependencyKey> {
    let rest = key.strip_prefix("treecalc-collection:")?;
    let mut family = None;
    let mut base_node_id = None;
    let mut member_node_ids = None;
    for part in rest.split(';') {
        if let Some(value) = part.strip_prefix("family=") {
            family = Some(value.to_string());
        } else if let Some(value) = part.strip_prefix("base=") {
            base_node_id = value.parse::<u64>().ok().map(TreeNodeId);
        } else if let Some(value) = part.strip_prefix("members=") {
            let mut members = Vec::new();
            if !value.is_empty() {
                for item in value.split(',') {
                    members.push(TreeNodeId(item.parse::<u64>().ok()?));
                }
            }
            member_node_ids = Some(members);
        }
    }
    Some(TreeCalcCollectionHostDependencyKey {
        family: family?,
        base_node_id: base_node_id?,
        member_node_ids: member_node_ids?,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum FixtureFormulaBinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FixtureFormulaPolicyClass {
    OpaqueOxfmlSource,
    LegacyStructuredQuarantine,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FixtureFormulaAst {
    Literal {
        value: String,
    },
    Reference(TreeReference),
    Binary {
        op: FixtureFormulaBinaryOp,
        left: Box<FixtureFormulaAst>,
        right: Box<FixtureFormulaAst>,
    },
    FunctionCall {
        function_name: String,
        arguments: Vec<FixtureFormulaAst>,
        may_introduce_dynamic_dependencies: bool,
    },
    /// Fixture-facing source carriage for OxFml helper-carrier witnesses whose
    /// syntax cannot be represented by the structured TreeFormula subset yet.
    RawOxfml {
        source_text: String,
        #[serde(default)]
        explicit_references: Vec<TreeReference>,
    },
}

impl FixtureFormulaAst {
    #[must_use]
    pub fn policy_class(&self) -> FixtureFormulaPolicyClass {
        match self {
            FixtureFormulaAst::RawOxfml { .. } => FixtureFormulaPolicyClass::OpaqueOxfmlSource,
            FixtureFormulaAst::Literal { .. }
            | FixtureFormulaAst::Reference(_)
            | FixtureFormulaAst::Binary { .. }
            | FixtureFormulaAst::FunctionCall { .. } => {
                FixtureFormulaPolicyClass::LegacyStructuredQuarantine
            }
        }
    }

    #[must_use]
    pub fn to_tree_formula(&self, owner_node_id: TreeNodeId) -> TreeFormula {
        let mut state = FixtureFormulaRenderState {
            owner_node_id,
            next_reference_index: 0,
            explicit_references: Vec::new(),
        };
        let source_text = state.render_formula(self);
        let lazy_residual_publication = matches!(
            self,
            FixtureFormulaAst::FunctionCall { function_name, .. }
                if function_name.eq_ignore_ascii_case("IF")
        );
        TreeFormula::opaque_oxfml(source_text, state.explicit_references)
            .with_lazy_residual_publication(lazy_residual_publication)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreeFormulaBinding {
    pub owner_node_id: TreeNodeId,
    pub formula_artifact_id: FormulaArtifactId,
    pub bind_artifact_id: Option<BindArtifactId>,
    pub expression: TreeFormula,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TreeFormulaCatalog {
    bindings_by_owner: BTreeMap<TreeNodeId, TreeFormulaBinding>,
}

impl TreeFormulaCatalog {
    #[must_use]
    pub fn new(bindings: impl IntoIterator<Item = TreeFormulaBinding>) -> Self {
        let bindings_by_owner = bindings
            .into_iter()
            .map(|binding| (binding.owner_node_id, binding))
            .collect::<BTreeMap<_, _>>();
        Self { bindings_by_owner }
    }

    #[must_use]
    pub fn bindings_by_owner(&self) -> &BTreeMap<TreeNodeId, TreeFormulaBinding> {
        &self.bindings_by_owner
    }

    pub fn try_get_binding(&self, owner_node_id: TreeNodeId) -> Option<&TreeFormulaBinding> {
        self.bindings_by_owner.get(&owner_node_id)
    }

    #[must_use]
    pub fn owner_node_ids(&self) -> Vec<TreeNodeId> {
        self.bindings_by_owner.keys().copied().collect()
    }

    #[must_use]
    pub fn to_dependency_descriptors(
        &self,
        snapshot: &StructuralSnapshot,
    ) -> Vec<DependencyDescriptor> {
        let mut descriptors = Vec::new();

        for binding in self.bindings_by_owner.values() {
            descriptors.extend(
                binding
                    .expression
                    .explicit_references()
                    .iter()
                    .enumerate()
                    .flat_map(|(index, reference)| {
                        lower_reference(snapshot, binding, reference, index)
                    }),
            );
        }

        descriptors.sort_by(|left, right| left.descriptor_id.cmp(&right.descriptor_id));
        descriptors
    }
}

fn lower_reference(
    snapshot: &StructuralSnapshot,
    binding: &TreeFormulaBinding,
    reference: &TreeReference,
    index: usize,
) -> Vec<DependencyDescriptor> {
    let descriptor_id = format!("bind:{}:ref:{index}", binding.formula_artifact_id.0);
    if let TreeReference::ReferenceCollection(collection) = reference {
        return lower_reference_collection(snapshot, binding, collection, descriptor_id);
    }

    let kind = reference.descriptor_kind();
    let target_node_id = reference.resolve_target(snapshot, binding.owner_node_id);
    let workspace_target = reference.workspace_target();
    let carrier_detail = reference.carrier_detail();
    let requires_rebind_on_structural_change = reference.requires_rebind_on_structural_change();

    vec![DependencyDescriptor {
        descriptor_id: descriptor_id.clone(),
        source_reference_handle: Some(format!("oxcalc_explicit_reference:{descriptor_id}")),
        owner_node_id: binding.owner_node_id,
        target_node_id,
        workspace_target,
        kind,
        carrier_detail,
        requires_rebind_on_structural_change,
        tree_reference_collection: None,
    }]
}

fn lower_reference_collection(
    snapshot: &StructuralSnapshot,
    binding: &TreeFormulaBinding,
    collection: &TreeCalcReferenceCollection,
    descriptor_id: String,
) -> Vec<DependencyDescriptor> {
    match collection {
        TreeCalcReferenceCollection::ChildrenV1(children) => {
            let member_node_ids = snapshot
                .try_get_node(children.base_node_id)
                .map_or_else(Vec::new, |node| node.child_ids.clone());
            let collection_dependency = TreeReferenceCollectionDependency::children_v1(
                children.host_ref_handle.clone(),
                children.base_node_id,
                member_node_ids.clone(),
            );
            let mut descriptors = vec![DependencyDescriptor {
                descriptor_id: format!("{descriptor_id}:membership"),
                source_reference_handle: Some(children.host_ref_handle.clone()),
                owner_node_id: binding.owner_node_id,
                target_node_id: None,
                workspace_target: None,
                kind: DependencyDescriptorKind::TreeReferenceCollectionMembership,
                carrier_detail: collection_dependency.carrier_detail(),
                tree_reference_collection: Some(collection_dependency),
                requires_rebind_on_structural_change: false,
            }];

            descriptors.extend(member_node_ids.into_iter().enumerate().map(
                |(member_index, member_node_id)| DependencyDescriptor {
                    descriptor_id: format!("{descriptor_id}:member:{member_index}"),
                    source_reference_handle: Some(children.host_ref_handle.clone()),
                    owner_node_id: binding.owner_node_id,
                    target_node_id: Some(member_node_id),
                    workspace_target: None,
                    kind: DependencyDescriptorKind::TreeReferenceCollectionMemberValue,
                    carrier_detail: format!(
                        "treecalc_children_v1_member:handle={}:ordinal={member_index}:target={member_node_id}",
                        children.host_ref_handle
                    ),
                    tree_reference_collection: None,
                    requires_rebind_on_structural_change: false,
                },
            ));

            descriptors
        }
        TreeCalcReferenceCollection::ReferenceLiteralArrayV1(collection) => {
            let collection_dependency =
                TreeReferenceCollectionDependency::reference_literal_array_v1(
                    collection.host_ref_handle.clone(),
                    collection.owner_node_id,
                    collection.member_node_ids.clone(),
                );
            let mut descriptors = vec![DependencyDescriptor {
                descriptor_id: format!("{descriptor_id}:membership"),
                source_reference_handle: Some(collection.host_ref_handle.clone()),
                owner_node_id: binding.owner_node_id,
                target_node_id: None,
                workspace_target: None,
                kind: DependencyDescriptorKind::TreeReferenceCollectionMembership,
                carrier_detail: collection_dependency.carrier_detail(),
                tree_reference_collection: Some(collection_dependency),
                requires_rebind_on_structural_change: false,
            }];

            descriptors.extend(collection.member_node_ids.iter().copied().enumerate().map(
                |(member_index, member_node_id)| DependencyDescriptor {
                    descriptor_id: format!("{descriptor_id}:member:{member_index}"),
                    source_reference_handle: Some(collection.host_ref_handle.clone()),
                    owner_node_id: binding.owner_node_id,
                    target_node_id: Some(member_node_id),
                    workspace_target: None,
                    kind: DependencyDescriptorKind::TreeReferenceCollectionMemberValue,
                    carrier_detail: format!(
                        "treecalc_reference_literal_array_v1_member:handle={}:ordinal={member_index}:target={member_node_id}",
                        collection.host_ref_handle
                    ),
                    tree_reference_collection: None,
                    requires_rebind_on_structural_change: false,
                },
            ));

            descriptors
        }
        TreeCalcReferenceCollection::OrderedSelectorV1(collection) => {
            let collection_dependency = TreeReferenceCollectionDependency::ordered_selector_v1(
                collection.family.dependency_family(),
                collection.host_ref_handle.clone(),
                collection.base_node_id,
                collection.member_node_ids.clone(),
            );
            let mut descriptors = vec![DependencyDescriptor {
                descriptor_id: format!("{descriptor_id}:membership"),
                source_reference_handle: Some(collection.host_ref_handle.clone()),
                owner_node_id: binding.owner_node_id,
                target_node_id: None,
                workspace_target: None,
                kind: DependencyDescriptorKind::TreeReferenceCollectionMembership,
                carrier_detail: collection_dependency.carrier_detail(),
                tree_reference_collection: Some(collection_dependency),
                requires_rebind_on_structural_change: false,
            }];

            descriptors.extend(collection.member_node_ids.iter().copied().enumerate().map(
                |(member_index, member_node_id)| DependencyDescriptor {
                    descriptor_id: format!("{descriptor_id}:member:{member_index}"),
                    source_reference_handle: Some(collection.host_ref_handle.clone()),
                    owner_node_id: binding.owner_node_id,
                    target_node_id: Some(member_node_id),
                    workspace_target: None,
                    kind: DependencyDescriptorKind::TreeReferenceCollectionMemberValue,
                    carrier_detail: format!(
                        "treecalc_ordered_selector_v1_member:family={}:handle={}:ordinal={member_index}:target={member_node_id}",
                        collection.family.stable_id(),
                        collection.host_ref_handle
                    ),
                    tree_reference_collection: None,
                    requires_rebind_on_structural_change: false,
                },
            ));

            descriptors
        }
    }
}

struct FixtureFormulaRenderState {
    owner_node_id: TreeNodeId,
    next_reference_index: usize,
    explicit_references: Vec<TreeReference>,
}

impl FixtureFormulaRenderState {
    fn render_formula(&mut self, formula: &FixtureFormulaAst) -> String {
        match formula {
            FixtureFormulaAst::Literal { value } => render_literal(value),
            FixtureFormulaAst::Reference(reference) => self.render_reference(reference),
            FixtureFormulaAst::Binary { op, left, right } => {
                let left = self.render_formula(left);
                let right = self.render_formula(right);
                let operator = match op {
                    FixtureFormulaBinaryOp::Add => "+",
                    FixtureFormulaBinaryOp::Subtract => "-",
                    FixtureFormulaBinaryOp::Multiply => "*",
                    FixtureFormulaBinaryOp::Divide => "/",
                };
                format!("({left}{operator}{right})")
            }
            FixtureFormulaAst::FunctionCall {
                function_name,
                arguments,
                ..
            } => {
                let arguments = arguments
                    .iter()
                    .map(|argument| self.render_formula(argument))
                    .collect::<Vec<_>>();
                format!(
                    "{}({})",
                    function_name.to_ascii_uppercase(),
                    arguments.join(",")
                )
            }
            FixtureFormulaAst::RawOxfml {
                source_text,
                explicit_references,
            } => {
                for reference in explicit_references {
                    self.record_reference(reference);
                }
                source_text.trim_start_matches('=').to_string()
            }
        }
    }

    fn render_reference(&mut self, reference: &TreeReference) -> String {
        match reference {
            TreeReference::DirectNode { .. }
            | TreeReference::ReferenceCollection(_)
            | TreeReference::ProjectionPath { .. }
            | TreeReference::RelativePath { .. }
            | TreeReference::SiblingOffset { .. }
            | TreeReference::QualifiedSiblingOffset { .. }
            | TreeReference::QualifiedParentOffset { .. }
            | TreeReference::CrossWorkspaceResolved { .. }
            | TreeReference::DynamicResolved { .. } => self.record_named_reference(reference),
            TreeReference::Unresolved { .. } => self.record_unresolved_reference(reference),
            TreeReference::HostSensitive { .. } => {
                self.record_fact(reference);
                "INFO(\"system\")".to_string()
            }
            TreeReference::CapabilitySensitive { .. } => {
                self.record_fact(reference);
                "INFO(\"osversion\")".to_string()
            }
            TreeReference::ShapeTopology { .. } => {
                self.record_fact(reference);
                "ROWS(A1:A1)".to_string()
            }
            TreeReference::DynamicPotential { carrier_id, .. } => {
                self.record_fact(reference);
                let topic = escape_excel_text(carrier_id);
                format!("RTD(\"TREECALC\",\"\",\"{topic}\")")
            }
        }
    }

    fn record_reference(&mut self, reference: &TreeReference) {
        match reference {
            TreeReference::DirectNode { .. }
            | TreeReference::ReferenceCollection(_)
            | TreeReference::ProjectionPath { .. }
            | TreeReference::RelativePath { .. }
            | TreeReference::SiblingOffset { .. }
            | TreeReference::QualifiedSiblingOffset { .. }
            | TreeReference::QualifiedParentOffset { .. }
            | TreeReference::CrossWorkspaceResolved { .. }
            | TreeReference::DynamicResolved { .. } => {
                let _ = self.record_named_reference(reference);
            }
            TreeReference::Unresolved { .. } => {
                let _ = self.record_unresolved_reference(reference);
            }
            TreeReference::HostSensitive { .. }
            | TreeReference::CapabilitySensitive { .. }
            | TreeReference::ShapeTopology { .. }
            | TreeReference::DynamicPotential { .. } => self.record_fact(reference),
        }
    }

    fn record_named_reference(&mut self, reference: &TreeReference) -> String {
        let token = format!(
            "TREE_REF_{}_{}",
            self.owner_node_id.0, self.next_reference_index
        );
        self.next_reference_index += 1;
        self.explicit_references.push(reference.clone());
        token
    }

    fn record_unresolved_reference(&mut self, reference: &TreeReference) -> String {
        let token = format!(
            "TREE_UNRESOLVED_{}_{}",
            self.owner_node_id.0, self.next_reference_index
        );
        self.next_reference_index += 1;
        self.explicit_references.push(reference.clone());
        token
    }

    fn record_fact(&mut self, reference: &TreeReference) {
        self.explicit_references.push(reference.clone());
    }
}

fn normalize_formula_source(source_text: String) -> String {
    if source_text.starts_with('=') {
        source_text
    } else {
        format!("={source_text}")
    }
}

fn render_literal(value: &str) -> String {
    if value.parse::<f64>().is_ok() {
        value.to_string()
    } else {
        format!("\"{}\"", escape_excel_text(value))
    }
}

fn escape_excel_text(value: &str) -> String {
    value.replace('"', "\"\"")
}

impl TreeReference {
    #[must_use]
    pub fn inventory_variant(&self) -> TreeReferenceInventoryVariant {
        match self {
            TreeReference::DirectNode { .. } => TreeReferenceInventoryVariant::DirectNode,
            TreeReference::ReferenceCollection(TreeCalcReferenceCollection::ChildrenV1(_)) => {
                TreeReferenceInventoryVariant::ChildrenV1
            }
            TreeReference::ReferenceCollection(
                TreeCalcReferenceCollection::ReferenceLiteralArrayV1(_),
            ) => TreeReferenceInventoryVariant::ReferenceLiteralArray,
            TreeReference::ReferenceCollection(TreeCalcReferenceCollection::OrderedSelectorV1(
                collection,
            )) => match collection.family {
                TreeCalcOrderedSelectorFamily::SiblingSetV1 => {
                    TreeReferenceInventoryVariant::SiblingSetSelector
                }
                TreeCalcOrderedSelectorFamily::PrecedingV1
                | TreeCalcOrderedSelectorFamily::FollowingV1 => {
                    TreeReferenceInventoryVariant::PrecedingFollowingSelector
                }
                TreeCalcOrderedSelectorFamily::AncestorsV1 => {
                    TreeReferenceInventoryVariant::AncestorSetSelector
                }
                TreeCalcOrderedSelectorFamily::RecursiveDescendantsV1 => {
                    TreeReferenceInventoryVariant::RecursiveSelector
                }
            },
            TreeReference::ProjectionPath { .. } => TreeReferenceInventoryVariant::ProjectionPath,
            TreeReference::RelativePath { base, .. } => match base {
                RelativeReferenceBase::SelfNode => TreeReferenceInventoryVariant::RelativePathSelf,
                RelativeReferenceBase::ParentNode => {
                    TreeReferenceInventoryVariant::RelativePathParent
                }
                RelativeReferenceBase::Ancestor(_) => {
                    TreeReferenceInventoryVariant::RelativePathAncestor
                }
            },
            TreeReference::SiblingOffset { .. } | TreeReference::QualifiedSiblingOffset { .. } => {
                TreeReferenceInventoryVariant::SiblingOffset
            }
            TreeReference::QualifiedParentOffset { .. } => {
                TreeReferenceInventoryVariant::RelativePathParent
            }
            TreeReference::HostSensitive { .. } => TreeReferenceInventoryVariant::HostSensitive,
            TreeReference::CrossWorkspaceResolved { .. } => {
                TreeReferenceInventoryVariant::CrossWorkspaceReference
            }
            TreeReference::CapabilitySensitive { .. } => {
                TreeReferenceInventoryVariant::CapabilitySensitive
            }
            TreeReference::ShapeTopology { .. } => TreeReferenceInventoryVariant::ShapeTopology,
            TreeReference::DynamicPotential { .. } => {
                TreeReferenceInventoryVariant::DynamicPotential
            }
            TreeReference::DynamicResolved { .. } => TreeReferenceInventoryVariant::DynamicResolved,
            TreeReference::Unresolved { .. } => TreeReferenceInventoryVariant::Unresolved,
        }
    }

    #[must_use]
    pub fn implementation_input(&self) -> TreeReferenceImplementationInput {
        tree_reference_implementation_input(self.inventory_variant())
            .expect("all concrete TreeReference variants must have W056 inventory input")
    }

    #[must_use]
    pub fn carrier_class(&self) -> TreeReferenceCarrierClass {
        match self {
            TreeReference::HostSensitive { .. }
            | TreeReference::CapabilitySensitive { .. }
            | TreeReference::ShapeTopology { .. }
            | TreeReference::DynamicPotential { .. } => {
                TreeReferenceCarrierClass::RuntimeFactProjection
            }
            TreeReference::DirectNode { .. }
            | TreeReference::ReferenceCollection(_)
            | TreeReference::ProjectionPath { .. }
            | TreeReference::RelativePath { .. }
            | TreeReference::SiblingOffset { .. }
            | TreeReference::QualifiedSiblingOffset { .. }
            | TreeReference::QualifiedParentOffset { .. }
            | TreeReference::CrossWorkspaceResolved { .. }
            | TreeReference::DynamicResolved { .. }
            | TreeReference::Unresolved { .. } => TreeReferenceCarrierClass::FormulaReference,
        }
    }

    pub fn resolve_target(
        &self,
        snapshot: &StructuralSnapshot,
        owner_node_id: TreeNodeId,
    ) -> Option<TreeNodeId> {
        match self {
            TreeReference::DirectNode { target_node_id } => Some(*target_node_id),
            TreeReference::ReferenceCollection(_) => None,
            TreeReference::ProjectionPath { projection_path } => {
                snapshot.try_resolve_projection_path(projection_path)
            }
            TreeReference::RelativePath {
                base,
                path_segments,
            } => {
                let base_node_id = match base {
                    RelativeReferenceBase::SelfNode => Some(owner_node_id),
                    RelativeReferenceBase::ParentNode => snapshot.parent_id_of(owner_node_id),
                    RelativeReferenceBase::Ancestor(levels_up) => {
                        snapshot.nth_ancestor_of(owner_node_id, *levels_up)
                    }
                };
                base_node_id.and_then(|base_node_id| {
                    snapshot.try_resolve_descendant_path(base_node_id, path_segments)
                })
            }
            TreeReference::SiblingOffset {
                offset,
                tail_segments,
            } => {
                let sibling_node_id = snapshot.try_resolve_sibling_offset(owner_node_id, *offset);
                sibling_node_id.and_then(|sibling_node_id| {
                    if tail_segments.is_empty() {
                        Some(sibling_node_id)
                    } else {
                        snapshot.try_resolve_descendant_path(sibling_node_id, tail_segments)
                    }
                })
            }
            TreeReference::QualifiedSiblingOffset {
                base_node_id,
                offset,
                tail_segments,
            } => {
                let sibling_node_id = snapshot.try_resolve_sibling_offset(*base_node_id, *offset);
                sibling_node_id.and_then(|sibling_node_id| {
                    if tail_segments.is_empty() {
                        Some(sibling_node_id)
                    } else {
                        snapshot.try_resolve_descendant_path(sibling_node_id, tail_segments)
                    }
                })
            }
            TreeReference::QualifiedParentOffset {
                base_node_id,
                tail_segments,
            } => {
                let parent_node_id = snapshot.parent_id_of(*base_node_id);
                parent_node_id.and_then(|parent_node_id| {
                    if tail_segments.is_empty() {
                        Some(parent_node_id)
                    } else {
                        snapshot.try_resolve_descendant_path(parent_node_id, tail_segments)
                    }
                })
            }
            TreeReference::HostSensitive { .. }
            | TreeReference::CrossWorkspaceResolved { .. }
            | TreeReference::CapabilitySensitive { .. }
            | TreeReference::ShapeTopology { .. }
            | TreeReference::Unresolved { .. } => None,
            TreeReference::DynamicPotential { .. } => None,
            TreeReference::DynamicResolved { target_node_id, .. } => Some(*target_node_id),
        }
    }

    pub fn resolve_target_with_meta_visibility(
        &self,
        snapshot: &StructuralSnapshot,
        owner_node_id: TreeNodeId,
        meta_node_ids: &BTreeSet<TreeNodeId>,
    ) -> Option<TreeNodeId> {
        match self {
            TreeReference::SiblingOffset {
                offset,
                tail_segments,
            } => resolve_visible_sibling_offset_target(
                snapshot,
                meta_node_ids,
                owner_node_id,
                *offset,
                tail_segments,
            ),
            TreeReference::QualifiedSiblingOffset {
                base_node_id,
                offset,
                tail_segments,
            } => resolve_visible_sibling_offset_target(
                snapshot,
                meta_node_ids,
                *base_node_id,
                *offset,
                tail_segments,
            ),
            _ => self.resolve_target(snapshot, owner_node_id),
        }
    }

    #[must_use]
    pub fn workspace_target(&self) -> Option<WorkspaceQualifiedTarget> {
        match self {
            TreeReference::CrossWorkspaceResolved {
                workspace_handle,
                target_node_id,
                target_node_handle,
                availability_version,
                ..
            } => Some(WorkspaceQualifiedTarget {
                workspace_handle: workspace_handle.clone(),
                target_node_id: *target_node_id,
                target_node_handle: target_node_handle.clone(),
                availability_version: availability_version.clone(),
            }),
            _ => None,
        }
    }

    #[must_use]
    pub fn descriptor_kind(&self) -> DependencyDescriptorKind {
        match self {
            TreeReference::DirectNode { .. } | TreeReference::ProjectionPath { .. } => {
                DependencyDescriptorKind::StaticDirect
            }
            TreeReference::ReferenceCollection(_) => {
                DependencyDescriptorKind::TreeReferenceCollectionMembership
            }
            TreeReference::RelativePath { .. }
            | TreeReference::SiblingOffset { .. }
            | TreeReference::QualifiedSiblingOffset { .. }
            | TreeReference::QualifiedParentOffset { .. } => {
                DependencyDescriptorKind::RelativeBound
            }
            TreeReference::HostSensitive { .. } => DependencyDescriptorKind::HostSensitive,
            TreeReference::CrossWorkspaceResolved { .. } => DependencyDescriptorKind::HostSensitive,
            TreeReference::CapabilitySensitive { .. } => {
                DependencyDescriptorKind::CapabilitySensitive
            }
            TreeReference::ShapeTopology { .. } => DependencyDescriptorKind::ShapeTopology,
            TreeReference::DynamicPotential { .. } | TreeReference::DynamicResolved { .. } => {
                DependencyDescriptorKind::DynamicPotential
            }
            TreeReference::Unresolved { .. } => DependencyDescriptorKind::Unresolved,
        }
    }

    #[must_use]
    pub fn requires_rebind_on_structural_change(&self) -> bool {
        matches!(
            self,
            TreeReference::RelativePath { .. }
                | TreeReference::SiblingOffset { .. }
                | TreeReference::QualifiedSiblingOffset { .. }
                | TreeReference::QualifiedParentOffset { .. }
                | TreeReference::HostSensitive { .. }
                | TreeReference::CrossWorkspaceResolved { .. }
                | TreeReference::CapabilitySensitive { .. }
                | TreeReference::ShapeTopology { .. }
                | TreeReference::DynamicResolved { .. }
                | TreeReference::Unresolved { .. }
        )
    }

    #[must_use]
    pub fn carrier_detail(&self) -> String {
        match self {
            TreeReference::DirectNode { target_node_id } => {
                format!("direct_node:{target_node_id}")
            }
            TreeReference::ReferenceCollection(TreeCalcReferenceCollection::ChildrenV1(
                collection,
            )) => {
                format!(
                    "treecalc_children_v1:base={}:membership={}:order={}",
                    collection.base_node_id,
                    collection.membership_version,
                    collection.order_version
                )
            }
            TreeReference::ReferenceCollection(
                TreeCalcReferenceCollection::ReferenceLiteralArrayV1(collection),
            ) => {
                format!(
                    "treecalc_reference_literal_array_v1:carrier={}:owner={}:membership={}:order={};members={}",
                    collection.carrier_id,
                    collection.owner_node_id,
                    collection.membership_version,
                    collection.order_version,
                    format_tree_node_ids(&collection.member_node_ids)
                )
            }
            TreeReference::ReferenceCollection(TreeCalcReferenceCollection::OrderedSelectorV1(
                collection,
            )) => {
                format!(
                    "treecalc_ordered_selector_v1:family={}:base={}:membership={}:order={}",
                    collection.family.stable_id(),
                    collection.base_node_id,
                    collection.membership_version,
                    collection.order_version
                )
            }
            TreeReference::ProjectionPath { projection_path } => {
                format!("projection_path:{projection_path}")
            }
            TreeReference::RelativePath {
                base,
                path_segments,
            } => {
                format!("relative_path:{base:?}:{}", path_segments.join("/"))
            }
            TreeReference::SiblingOffset {
                offset,
                tail_segments,
            } => format!("sibling_offset:{offset}:{}", tail_segments.join("/")),
            TreeReference::QualifiedSiblingOffset {
                base_node_id,
                offset,
                tail_segments,
            } => format!(
                "qualified_sibling_offset:base={base_node_id};offset={offset}:{}",
                tail_segments.join("/")
            ),
            TreeReference::QualifiedParentOffset {
                base_node_id,
                tail_segments,
            } => format!(
                "qualified_parent_offset:base={base_node_id}:{}",
                tail_segments.join("/")
            ),
            TreeReference::HostSensitive { carrier_id, detail } => {
                format!("host_sensitive:{carrier_id}:{detail}")
            }
            TreeReference::CrossWorkspaceResolved {
                workspace_handle,
                target_node_id,
                target_node_handle,
                availability_version,
                carrier_id,
                detail,
            } => format!(
                "cross_workspace_resolved:{carrier_id}:workspace={workspace_handle};target={target_node_id};target_handle={target_node_handle};availability={availability_version};{detail}"
            ),
            TreeReference::CapabilitySensitive { carrier_id, detail } => {
                format!("capability_sensitive:{carrier_id}:{detail}")
            }
            TreeReference::ShapeTopology { carrier_id, detail } => {
                format!("shape_topology:{carrier_id}:{detail}")
            }
            TreeReference::DynamicPotential { carrier_id, detail } => {
                format!("dynamic_potential:{carrier_id}:{detail}")
            }
            TreeReference::DynamicResolved {
                target_node_id,
                carrier_id,
                detail,
            } => {
                format!("dynamic_resolved:{target_node_id}:{carrier_id}:{detail}")
            }
            TreeReference::Unresolved { token } => format!("unresolved:{token}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use crate::dependency::{
        DependencyDescriptorKind, DependencyDiagnosticKind, DependencyGraph,
        TreeReferenceCollectionFamily,
    };
    use crate::structural::{StructuralNode, StructuralNodeKind, StructuralSnapshotId};
    use crate::tree_reference_rebind::w056_reference_dependency_surface;

    use super::*;

    fn fixture_formula(owner_node_id: TreeNodeId, ast: FixtureFormulaAst) -> TreeFormula {
        ast.to_tree_formula(owner_node_id)
    }

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
                    child_ids: vec![TreeNodeId(2), TreeNodeId(3)],
                    role: None,
                },
                StructuralNode {
                    node_id: TreeNodeId(2),
                    kind: StructuralNodeKind::Container,
                    symbol: "Branch".to_string(),
                    parent_id: Some(TreeNodeId(1)),
                    child_ids: vec![TreeNodeId(4), TreeNodeId(5)],
                    role: None,
                },
                StructuralNode {
                    node_id: TreeNodeId(3),
                    kind: StructuralNodeKind::Calculation,
                    symbol: "Sibling".to_string(),
                    parent_id: Some(TreeNodeId(1)),
                    child_ids: vec![],
                    role: None,
                },
                StructuralNode {
                    node_id: TreeNodeId(4),
                    kind: StructuralNodeKind::Calculation,
                    symbol: "Leaf".to_string(),
                    parent_id: Some(TreeNodeId(2)),
                    child_ids: vec![],
                    role: None,
                },
                StructuralNode {
                    node_id: TreeNodeId(5),
                    kind: StructuralNodeKind::Calculation,
                    symbol: "Neighbor".to_string(),
                    parent_id: Some(TreeNodeId(2)),
                    child_ids: vec![],
                    role: None,
                },
            ],
        )
        .unwrap()
    }

    fn projections_snapshot() -> StructuralSnapshot {
        StructuralSnapshot::create(
            StructuralSnapshotId(2),
            TreeNodeId(100),
            [
                StructuralNode {
                    node_id: TreeNodeId(100),
                    kind: StructuralNodeKind::Root,
                    symbol: "Projections".to_string(),
                    parent_id: None,
                    child_ids: vec![TreeNodeId(101), TreeNodeId(103)],
                    role: None,
                },
                StructuralNode {
                    node_id: TreeNodeId(101),
                    kind: StructuralNodeKind::Container,
                    symbol: "Branch1".to_string(),
                    parent_id: Some(TreeNodeId(100)),
                    child_ids: vec![TreeNodeId(102)],
                    role: None,
                },
                StructuralNode {
                    node_id: TreeNodeId(102),
                    kind: StructuralNodeKind::Calculation,
                    symbol: "MyNode".to_string(),
                    parent_id: Some(TreeNodeId(101)),
                    child_ids: vec![],
                    role: None,
                },
                StructuralNode {
                    node_id: TreeNodeId(103),
                    kind: StructuralNodeKind::Container,
                    symbol: "Branch X".to_string(),
                    parent_id: Some(TreeNodeId(100)),
                    child_ids: vec![TreeNodeId(104)],
                    role: None,
                },
                StructuralNode {
                    node_id: TreeNodeId(104),
                    kind: StructuralNodeKind::Calculation,
                    symbol: "MyNode".to_string(),
                    parent_id: Some(TreeNodeId(103)),
                    child_ids: vec![],
                    role: None,
                },
            ],
        )
        .unwrap()
    }

    #[test]
    fn explicit_host_path_base_resolver_maps_structural_paths_without_name_precedence() {
        let snapshot = snapshot();

        let exact = resolve_treecalc_explicit_host_path_base(&snapshot, "Root/Branch").unwrap();
        assert_eq!(exact.base_node_id, TreeNodeId(2));
        assert_eq!(exact.canonical_projection_path, "Root/Branch");
        assert_eq!(
            exact.resolution_layer,
            TreeCalcHostPathBaseResolutionLayer::ProjectionPath
        );

        let dotted = resolve_treecalc_explicit_host_path_base(&snapshot, "Root.Branch").unwrap();
        assert_eq!(dotted.base_node_id, TreeNodeId(2));
        assert_eq!(
            dotted.resolution_layer,
            TreeCalcHostPathBaseResolutionLayer::DottedProjectionPath
        );

        let root_descendant =
            resolve_treecalc_explicit_host_path_base(&snapshot, "Branch.[Leaf]").unwrap();
        assert_eq!(root_descendant.base_node_id, TreeNodeId(4));
        assert_eq!(
            root_descendant.resolution_layer,
            TreeCalcHostPathBaseResolutionLayer::RootDescendantPath
        );

        let sheet_position_alias =
            resolve_treecalc_explicit_host_path_base(&snapshot, "Root!Branch").unwrap();
        assert_eq!(sheet_position_alias.base_node_id, TreeNodeId(2));
        assert_eq!(
            sheet_position_alias.resolution_layer,
            TreeCalcHostPathBaseResolutionLayer::DottedProjectionPath
        );

        for malformed in [
            "Branch.[Leaf",
            "Branch.Leaf]",
            "Branch.[Leaf.[Nested]]",
            "!Branch",
            "Root.Branch!Leaf",
        ] {
            let error = resolve_treecalc_explicit_host_path_base(&snapshot, malformed)
                .expect_err("malformed bracketed path should be rejected");
            assert!(
                matches!(
                    error,
                    TreeCalcHostPathBaseResolutionError::InvalidBaseTokenSyntax { .. }
                ),
                "unexpected error for {malformed}: {error:?}"
            );
        }
    }

    #[test]
    fn workspace_host_path_base_resolver_uses_aliases_availability_and_rooted_paths() {
        let accounts = snapshot();
        let projections = projections_snapshot();
        let mut registry = TreeCalcWorkspaceResolutionRegistry::with_current_workspace(
            "treecalc-workspace:accounts",
            &accounts,
            "treecalc-cross-workspace-availability:v1:accounts:loaded",
        );
        registry.add_workspace(
            "treecalc-workspace:projections",
            &projections,
            "treecalc-cross-workspace-availability:v1:projections:loaded",
        );
        registry.add_alias("projections", "treecalc-workspace:projections");
        registry.add_alias(
            "C:\\Work\\projections.dnatree",
            "treecalc-workspace:projections",
        );
        registry.add_alias("missing", "treecalc-workspace:missing");

        let alias =
            resolve_treecalc_workspace_host_path_base(&registry, "[projections]Branch1.MyNode")
                .expect("aliased workspace base path");
        assert_eq!(alias.workspace_handle, "treecalc-workspace:projections");
        assert_eq!(
            alias.workspace_selector_token.as_deref(),
            Some("projections")
        );
        assert_eq!(alias.base_node_id, TreeNodeId(102));
        assert_eq!(
            alias.base_node_handle,
            "treecalc-workspace:projections#node:102"
        );
        assert_eq!(
            alias.canonical_projection_path,
            "Projections/Branch1/MyNode"
        );
        assert_eq!(
            alias.workspace_resolution_layer,
            TreeCalcWorkspaceResolutionLayer::WorkspaceAlias
        );
        assert_eq!(
            alias.local_resolution_layer,
            TreeCalcHostPathBaseResolutionLayer::RootDescendantPath
        );
        assert_eq!(
            alias.availability_packet.status,
            TreeCalcCrossWorkspaceAvailabilityStatus::Available
        );
        assert!(
            alias.resolution_identity.contains(
                "availability=treecalc-cross-workspace-availability:v1:projections:loaded"
            )
        );

        let escaped =
            resolve_treecalc_workspace_host_path_base(&registry, "[projections][Branch X].MyNode")
                .expect("escaped first path segment");
        assert_eq!(escaped.base_node_id, TreeNodeId(104));

        let quoted = resolve_treecalc_workspace_host_path_base(
            &registry,
            "['C:\\Work\\projections.dnatree']Branch1.MyNode",
        )
        .expect("quoted direct path mapped by host registry");
        assert_eq!(
            quoted.workspace_resolution_layer,
            TreeCalcWorkspaceResolutionLayer::QuotedWorkspacePath
        );
        assert_eq!(quoted.base_node_id, TreeNodeId(102));

        let workspace_root = resolve_treecalc_workspace_host_path_base(&registry, "[projections]")
            .expect("workspace root");
        assert_eq!(workspace_root.base_node_id, TreeNodeId(100));
        assert_eq!(
            workspace_root.local_resolution_layer,
            TreeCalcHostPathBaseResolutionLayer::WorkspaceRoot
        );

        let current_sheet_alias =
            resolve_treecalc_workspace_host_path_base(&registry, "Root!Branch")
                .expect("current workspace first-position ! alias");
        assert_eq!(
            current_sheet_alias.workspace_handle,
            "treecalc-workspace:accounts"
        );
        assert_eq!(current_sheet_alias.base_node_id, TreeNodeId(2));

        let projections_only = TreeCalcWorkspaceResolutionRegistry::with_current_workspace(
            "treecalc-workspace:projections",
            &projections,
            "treecalc-cross-workspace-availability:v1:projections:loaded",
        );
        let escaped_fallback =
            resolve_treecalc_workspace_host_path_base(&projections_only, "[Branch X].MyNode")
                .expect("unregistered bracket word falls back to escaped local path");
        assert_eq!(escaped_fallback.workspace_selector_token, None);
        assert_eq!(
            escaped_fallback.workspace_resolution_layer,
            TreeCalcWorkspaceResolutionLayer::CurrentWorkspace
        );
        assert_eq!(escaped_fallback.local_path_token_text, "[Branch X].MyNode");
        assert_eq!(escaped_fallback.base_node_id, TreeNodeId(104));
        assert!(
            escaped_fallback
                .resolution_identity
                .contains("bracket_fallback=escaped_current_workspace_path")
        );

        let unavailable = resolve_treecalc_workspace_host_path_base(&registry, "[missing]Branch1")
            .expect_err("registered but unloaded workspace remains typed");
        let TreeCalcWorkspaceHostPathBaseResolutionError::WorkspaceUnavailable {
            workspace_handle,
            availability_packet,
            ..
        } = unavailable
        else {
            panic!("expected unavailable workspace, got {unavailable:?}");
        };
        assert_eq!(
            workspace_handle, "treecalc-workspace:missing",
            "registered alias resolves to its opaque host workspace handle"
        );
        assert_eq!(
            availability_packet.diagnostics[0].code,
            TreeCalcCrossWorkspaceDiagnosticCode::WorkspaceUnavailable
        );
        assert_eq!(
            availability_packet.workspace_handle,
            "treecalc-workspace:missing"
        );
        assert!(
            availability_packet
                .prepared_identity_component()
                .contains("status=Unavailable")
        );
    }

    #[test]
    fn workspace_qualified_reference_preserves_external_target_identity() {
        let accounts = snapshot();
        let projections = projections_snapshot();
        let mut registry = TreeCalcWorkspaceResolutionRegistry::with_current_workspace(
            "treecalc-workspace:accounts",
            &accounts,
            "treecalc-cross-workspace-availability:v1:accounts:loaded",
        );
        registry.add_workspace(
            "treecalc-workspace:projections",
            &projections,
            "treecalc-cross-workspace-availability:v1:projections:loaded",
        );
        registry.add_alias("projections", "treecalc-workspace:projections");

        let resolution =
            resolve_treecalc_workspace_host_path_base(&registry, "[projections]Branch1.MyNode")
                .expect("cross-workspace base resolution");
        let reference = resolution.to_workspace_qualified_reference("carrier:xws:projections");
        assert_eq!(
            reference.inventory_variant(),
            TreeReferenceInventoryVariant::CrossWorkspaceReference
        );
        assert_eq!(
            reference.descriptor_kind(),
            DependencyDescriptorKind::HostSensitive
        );

        let catalog = TreeFormulaCatalog::new([TreeFormulaBinding {
            owner_node_id: TreeNodeId(4),
            formula_artifact_id: FormulaArtifactId("formula:xws".to_string()),
            bind_artifact_id: Some(BindArtifactId("bind:xws".to_string())),
            expression: TreeFormula::opaque_oxfml("=TREE_REF_4_0", [reference]),
        }]);
        let descriptors = catalog.to_dependency_descriptors(&accounts);
        assert_eq!(descriptors.len(), 1);
        let descriptor = &descriptors[0];
        assert_eq!(descriptor.target_node_id, None);
        let target = descriptor
            .workspace_target
            .as_ref()
            .expect("workspace-qualified target");
        assert_eq!(target.workspace_handle, "treecalc-workspace:projections");
        assert_eq!(target.target_node_id, TreeNodeId(102));
        assert_eq!(
            target.target_node_handle,
            "treecalc-workspace:projections#node:102"
        );
        assert_eq!(
            target.availability_version,
            "treecalc-cross-workspace-availability:v1:projections:loaded"
        );

        let graph = DependencyGraph::build(&accounts, &descriptors);
        assert!(graph.diagnostics.is_empty());
        assert!(graph.reverse_edges.is_empty());
        let external_edges = graph
            .workspace_reverse_edges
            .get("treecalc-workspace:projections#node:102")
            .expect("workspace reverse edge");
        assert_eq!(external_edges.len(), 1);
        assert_eq!(external_edges[0].target, *target);

        let surface = w056_reference_dependency_surface(&graph);
        assert_eq!(surface.context_reverse_edges.len(), 0);
        assert_eq!(surface.workspace_target_reverse_edges.len(), 1);
        assert_eq!(surface.workspace_target_reverse_edges[0].target, *target);
        assert_eq!(
            surface.descriptor_facts[0].workspace_target.as_ref(),
            Some(target)
        );
    }

    #[test]
    fn cross_workspace_availability_packet_is_identity_and_diagnostic_surface_only() {
        let missing = TreeCalcCrossWorkspaceAvailabilityPacket::provider_missing("Book2!Root");
        assert_eq!(
            missing.status,
            TreeCalcCrossWorkspaceAvailabilityStatus::Unavailable
        );
        assert_eq!(
            missing.diagnostics[0].code,
            TreeCalcCrossWorkspaceDiagnosticCode::WorkspaceProviderMissing
        );
        assert!(
            missing
                .prepared_identity_component()
                .contains("cross_workspace_availability_version=")
        );
        assert!(
            missing
                .prepared_identity_component()
                .contains("status=Unavailable")
        );

        let degraded = TreeCalcCrossWorkspaceAvailabilityPacket::degraded(
            "treecalc-workspace:Book2",
            "Book2",
            "treecalc-cross-workspace-availability:v1:Book2:degraded",
            "stale_snapshot",
            "workspace snapshot is stale",
        );
        assert_eq!(
            degraded.status,
            TreeCalcCrossWorkspaceAvailabilityStatus::Degraded
        );
        assert!(
            degraded
                .prepared_identity_component()
                .contains("degradation_layer=stale_snapshot")
        );

        let available = TreeCalcCrossWorkspaceAvailabilityPacket::available(
            "treecalc-workspace:Book2",
            "Book2",
            "treecalc-cross-workspace-availability:v1:Book2:available",
        );
        assert_eq!(
            available.status,
            TreeCalcCrossWorkspaceAvailabilityStatus::Available
        );
        assert!(available.diagnostics.is_empty());
        assert!(
            available
                .prepared_identity_component()
                .contains("status=Available")
        );
    }

    #[test]
    fn ordered_selector_traversal_resolver_projects_structural_membership() {
        let snapshot = snapshot();

        assert_eq!(
            resolve_treecalc_ordered_selector_traversal(
                &snapshot,
                TreeCalcOrderedSelectorFamily::SiblingSetV1,
                TreeNodeId(4),
                &[],
                TreeCalcOrderedSelectorTraversalPolicy::default(),
            )
            .unwrap()
            .member_node_ids,
            vec![TreeNodeId(5)]
        );
        assert_eq!(
            resolve_treecalc_ordered_selector_traversal(
                &snapshot,
                TreeCalcOrderedSelectorFamily::PrecedingV1,
                TreeNodeId(5),
                &[],
                TreeCalcOrderedSelectorTraversalPolicy::default(),
            )
            .unwrap()
            .member_node_ids,
            vec![TreeNodeId(4)]
        );
        assert_eq!(
            resolve_treecalc_ordered_selector_traversal(
                &snapshot,
                TreeCalcOrderedSelectorFamily::FollowingV1,
                TreeNodeId(4),
                &[],
                TreeCalcOrderedSelectorTraversalPolicy::default(),
            )
            .unwrap()
            .member_node_ids,
            vec![TreeNodeId(5)]
        );
        assert_eq!(
            resolve_treecalc_ordered_selector_traversal(
                &snapshot,
                TreeCalcOrderedSelectorFamily::AncestorsV1,
                TreeNodeId(4),
                &[],
                TreeCalcOrderedSelectorTraversalPolicy::default(),
            )
            .unwrap()
            .member_node_ids,
            vec![TreeNodeId(2)]
        );
        assert_eq!(
            resolve_treecalc_ordered_selector_traversal(
                &snapshot,
                TreeCalcOrderedSelectorFamily::RecursiveDescendantsV1,
                TreeNodeId(1),
                &[],
                TreeCalcOrderedSelectorTraversalPolicy::default(),
            )
            .unwrap()
            .member_node_ids,
            vec![TreeNodeId(2), TreeNodeId(4), TreeNodeId(5), TreeNodeId(3)]
        );
    }

    #[test]
    fn ordered_selector_traversal_resolver_reports_bounds_and_missing_tail() {
        let snapshot = snapshot();
        let bounded = resolve_treecalc_ordered_selector_traversal(
            &snapshot,
            TreeCalcOrderedSelectorFamily::RecursiveDescendantsV1,
            TreeNodeId(1),
            &[],
            TreeCalcOrderedSelectorTraversalPolicy {
                max_recursive_descendants: 2,
            },
        )
        .expect_err("recursive traversal should respect explicit bound");
        assert!(matches!(
            bounded,
            TreeCalcOrderedSelectorTraversalError::RecursiveTraversalLimitExceeded {
                base_node_id: TreeNodeId(1),
                visited_count: 3,
                ..
            }
        ));

        let tail_miss = resolve_treecalc_ordered_selector_traversal(
            &snapshot,
            TreeCalcOrderedSelectorFamily::RecursiveDescendantsV1,
            TreeNodeId(1),
            &["Missing".to_string()],
            TreeCalcOrderedSelectorTraversalPolicy::default(),
        )
        .expect("tail miss is diagnostic-bearing empty membership");
        assert!(tail_miss.member_node_ids.is_empty());
        assert_eq!(
            tail_miss.diagnostics[0].code,
            TreeCalcOrderedSelectorTraversalDiagnosticCode::RecursiveTailMatchedNoMembers
        );
    }

    #[test]
    fn ordered_selector_traversal_resolver_applies_non_recursive_tail() {
        let snapshot = snapshot();

        // @PRECEDING.Leaf from `Sibling` (node 3): preceding sibling `Branch` (node 2)
        // has a `Leaf` child (node 4).
        assert_eq!(
            resolve_treecalc_ordered_selector_traversal(
                &snapshot,
                TreeCalcOrderedSelectorFamily::PrecedingV1,
                TreeNodeId(3),
                &["Leaf".to_string()],
                TreeCalcOrderedSelectorTraversalPolicy::default(),
            )
            .unwrap()
            .member_node_ids,
            vec![TreeNodeId(4)]
        );

        // @ANCESTORS.Neighbor from `Leaf` (node 4): ancestor `Branch` (node 2) has a
        // `Neighbor` child (node 5).
        assert_eq!(
            resolve_treecalc_ordered_selector_traversal(
                &snapshot,
                TreeCalcOrderedSelectorFamily::AncestorsV1,
                TreeNodeId(4),
                &["Neighbor".to_string()],
                TreeCalcOrderedSelectorTraversalPolicy::default(),
            )
            .unwrap()
            .member_node_ids,
            vec![TreeNodeId(5)]
        );

        // A tail that matches no member records the non-recursive diagnostic.
        let tail_miss = resolve_treecalc_ordered_selector_traversal(
            &snapshot,
            TreeCalcOrderedSelectorFamily::PrecedingV1,
            TreeNodeId(5),
            &["Missing".to_string()],
            TreeCalcOrderedSelectorTraversalPolicy::default(),
        )
        .unwrap();
        assert!(tail_miss.member_node_ids.is_empty());
        assert_eq!(
            tail_miss.diagnostics[0].code,
            TreeCalcOrderedSelectorTraversalDiagnosticCode::TailMatchedNoMembers
        );
    }

    #[test]
    fn w056_inventory_names_admitted_reference_inputs_and_typed_exclusions() {
        let inputs = tree_reference_implementation_inputs();
        let variants = inputs
            .iter()
            .map(|input| input.variant)
            .collect::<BTreeSet<_>>();

        assert_eq!(inputs.len(), variants.len());
        assert!(variants.contains(&TreeReferenceInventoryVariant::DirectNode));
        assert!(variants.contains(&TreeReferenceInventoryVariant::ChildrenV1));
        assert!(variants.contains(&TreeReferenceInventoryVariant::ReferenceLiteralArray));
        assert!(variants.contains(&TreeReferenceInventoryVariant::MixedReferenceArray));
        assert!(variants.contains(&TreeReferenceInventoryVariant::RelativePathParent));
        assert!(variants.contains(&TreeReferenceInventoryVariant::RelativePathAncestor));
        assert!(variants.contains(&TreeReferenceInventoryVariant::SiblingOffset));
        assert!(variants.contains(&TreeReferenceInventoryVariant::DynamicPotential));
        assert!(variants.contains(&TreeReferenceInventoryVariant::DynamicResolved));
        assert!(variants.contains(&TreeReferenceInventoryVariant::Unresolved));
        assert!(variants.contains(&TreeReferenceInventoryVariant::AncestorSetSelector));
        assert!(variants.contains(&TreeReferenceInventoryVariant::CrossWorkspaceReference));
        assert!(variants.contains(&TreeReferenceInventoryVariant::StructuredTableReference));
        assert!(variants.contains(&TreeReferenceInventoryVariant::BareNameOrCallableReference));

        let admitted = inputs.iter().filter(|input| input.is_admitted()).count();
        let typed_exclusions = inputs
            .iter()
            .filter(|input| input.status == TreeReferenceInventoryStatus::TypedExclusion)
            .count();
        assert!(admitted > typed_exclusions);

        let table = tree_reference_implementation_input(
            TreeReferenceInventoryVariant::StructuredTableReference,
        )
        .expect("table reference inventory");
        assert_eq!(table.blocker, None);
        assert_eq!(table.successor_bead, Some("calc-4vs8.2"));
        assert_eq!(
            table.status,
            TreeReferenceInventoryStatus::AdmittedImplementationInput
        );
        assert!(
            table
                .dependency_facts
                .contains(&DependencyDescriptorKind::StructuredTableIdentity)
        );
        assert!(
            table
                .dependency_facts
                .contains(&DependencyDescriptorKind::StructuredTableCallerContext)
        );
        assert!(
            table
                .invalidation_facts
                .contains(&InvalidationReasonKind::StructuredTableContextChanged)
        );

        let cross_workspace = tree_reference_implementation_input(
            TreeReferenceInventoryVariant::CrossWorkspaceReference,
        )
        .expect("cross-workspace inventory");
        assert_eq!(cross_workspace.blocker, None);
        assert_eq!(
            cross_workspace.status,
            TreeReferenceInventoryStatus::AdmittedImplementationInput
        );
        assert_eq!(
            cross_workspace.namespace_identity_need,
            NamespaceIdentityNeed::CrossWorkspaceAvailabilityVersion
        );
        assert!(
            cross_workspace
                .evidence_note
                .contains("workspace-qualified carriers")
        );

        let bare_name = tree_reference_implementation_input(
            TreeReferenceInventoryVariant::BareNameOrCallableReference,
        )
        .expect("bare name inventory");
        assert_eq!(bare_name.blocker, None);
        assert_eq!(
            bare_name.status,
            TreeReferenceInventoryStatus::AdmittedImplementationInput
        );
        assert_eq!(bare_name.successor_bead, Some("calc-4vs8.32"));
        assert!(bare_name.evidence_note.contains("W074 handoff"));

        let mixed_array =
            tree_reference_implementation_input(TreeReferenceInventoryVariant::MixedReferenceArray)
                .expect("mixed array inventory");
        assert_eq!(
            mixed_array.blocker,
            Some(TreeReferenceInventoryBlocker::NeedsReferenceOnlyArrayCarrier)
        );
    }

    #[test]
    fn w056_non_table_reference_category_matrix_is_complete_and_runnable() {
        let mut category_ids = BTreeSet::new();
        let mut statuses = BTreeSet::new();
        for category in W056_NON_TABLE_REFERENCE_CATEGORIES {
            assert!(
                category_ids.insert(category.category_id),
                "duplicate category {}",
                category.category_id
            );
            statuses.insert(category.evidence_status);
            assert!(
                !category.reference_family.is_empty()
                    && !category.examples.is_empty()
                    && !category.spec_anchor.is_empty()
                    && !category.expected_outcome_contract.is_empty(),
                "{} needs enough descriptive detail for expected-outcome tests",
                category.category_id
            );
            assert!(
                category.specification_is_sufficient_for_cases,
                "{} is not yet specified well enough to write cases",
                category.category_id
            );
            assert!(
                !category.corpus_or_suite.is_empty() && !category.runnable_suite_command.is_empty(),
                "{} needs a runnable corpus or test-suite command",
                category.category_id
            );
            assert!(
                !category.oxcalc_status.is_empty()
                    && !category.dnatreecalc_status.is_empty()
                    && !category.replay_status.is_empty()
                    && !category.current_test_result.is_empty(),
                "{} needs current cross-repo status and test-result text",
                category.category_id
            );
        }

        for required in [
            "children_collection",
            "walkup_dotted_descent",
            "ancestor_root_anchors",
            "workspace_alias_bang_syntax",
            "escaping_canonicalization_case",
            "meta_invisibility_accessors",
            "sibling_single_navigation",
            "ordered_set_selectors",
            "recursive_descent",
            "reference_literals_arrays",
            "dynamic_indirect_ctro",
            "cross_workspace_runtime",
            "bare_host_names_defined_name_lane",
            "node_as_function_lambda",
            "profile_gating",
            "structural_edit_rebind",
            "unresolved_diagnostics_self_reference",
        ] {
            assert!(
                category_ids.contains(required),
                "missing category {required}"
            );
        }

        for required_status in [
            W056NonTableReferenceEvidenceStatus::ProductGreen,
            W056NonTableReferenceEvidenceStatus::DirectContextSliceGreen,
            W056NonTableReferenceEvidenceStatus::DirectContextTypedPending,
        ] {
            assert!(
                statuses.contains(&required_status),
                "missing status {required_status:?}"
            );
        }

        let bare = w056_non_table_reference_category("bare_host_names_defined_name_lane")
            .expect("bare host-name category");
        assert!(bare.oxcalc_status.contains("W074 handoff consumed"));
        assert!(bare.blocks_w056_non_table_closure);

        let children =
            w056_non_table_reference_category("children_collection").expect("children category");
        assert_eq!(
            children.evidence_status,
            W056NonTableReferenceEvidenceStatus::ProductGreen
        );
        assert!(!children.blocks_w056_non_table_closure);

        let profile =
            w056_non_table_reference_category("profile_gating").expect("profile category");
        assert_eq!(
            profile.evidence_status,
            W056NonTableReferenceEvidenceStatus::DirectContextTypedPending
        );
        assert!(
            profile
                .current_test_result
                .contains("strict_excel_profile_not_supported")
        );

        let blockers = W056_NON_TABLE_REFERENCE_CATEGORIES
            .iter()
            .filter(|category| category.blocks_w056_non_table_closure)
            .count();
        assert!(blockers > 0, "parent W056 should not be overclaimed");
    }

    #[test]
    #[ignore = "red/green W056 full non-table closure gate; run explicitly after activating retained evidence"]
    fn w056_non_table_reference_resolution_full_scope_red_green_gate() {
        let blockers = W056_NON_TABLE_REFERENCE_CATEGORIES
            .iter()
            .filter(|category| category.blocks_w056_non_table_closure)
            .map(|category| {
                format!(
                    "{} [{}]: {} / {}",
                    category.category_id,
                    category.reference_family,
                    category.dnatreecalc_status,
                    category.replay_status
                )
            })
            .collect::<Vec<_>>();

        assert!(
            blockers.is_empty(),
            "W056 non-table reference closure still has red categories:\n{}",
            blockers.join("\n")
        );
    }

    #[test]
    fn concrete_tree_references_map_to_w056_inventory_inputs() {
        let cases = [
            (
                TreeReference::DirectNode {
                    target_node_id: TreeNodeId(2),
                },
                TreeReferenceInventoryVariant::DirectNode,
                TreeReferenceInventoryStatus::AdmittedCurrentCarrier,
                HostReferenceCorrelationNeed::SourceTokenToFormalReference,
                NamespaceIdentityNeed::StructureContextVersion,
                CallerContextIdentityNeed::None,
            ),
            (
                TreeReference::ReferenceCollection(TreeCalcReferenceCollection::ChildrenV1(
                    TreeCalcChildrenReferenceCollection::new(TreeNodeId(2), "@CHILDREN"),
                )),
                TreeReferenceInventoryVariant::ChildrenV1,
                TreeReferenceInventoryStatus::AdmittedCurrentCarrier,
                HostReferenceCorrelationNeed::HostReferenceHandle,
                NamespaceIdentityNeed::HostNamespaceVersion,
                CallerContextIdentityNeed::CallerNode,
            ),
            (
                TreeReference::ReferenceCollection(
                    TreeCalcReferenceCollection::ReferenceLiteralArrayV1(
                        TreeCalcReferenceLiteralArrayCollection::reference_only(
                            "array:q1",
                            TreeNodeId(4),
                            "{A,C,A}",
                            [
                                TreeCalcReferenceLiteralArrayElement::ReferenceNode(TreeNodeId(2)),
                                TreeCalcReferenceLiteralArrayElement::ReferenceNode(TreeNodeId(3)),
                                TreeCalcReferenceLiteralArrayElement::ReferenceNode(TreeNodeId(2)),
                            ],
                        )
                        .expect("reference-only array"),
                    ),
                ),
                TreeReferenceInventoryVariant::ReferenceLiteralArray,
                TreeReferenceInventoryStatus::AdmittedImplementationInput,
                HostReferenceCorrelationNeed::HostReferenceHandle,
                NamespaceIdentityNeed::HostNamespaceVersion,
                CallerContextIdentityNeed::CallerNode,
            ),
            (
                TreeReference::ReferenceCollection(TreeCalcReferenceCollection::OrderedSelectorV1(
                    TreeCalcOrderedSelectorReferenceCollection::new(
                        TreeCalcOrderedSelectorFamily::PrecedingV1,
                        TreeNodeId(4),
                        "@PRECEDING",
                        [TreeNodeId(3)],
                    ),
                )),
                TreeReferenceInventoryVariant::PrecedingFollowingSelector,
                TreeReferenceInventoryStatus::AdmittedImplementationInput,
                HostReferenceCorrelationNeed::HostReferenceHandle,
                NamespaceIdentityNeed::HostNamespaceVersion,
                CallerContextIdentityNeed::SiblingPosition,
            ),
            (
                TreeReference::RelativePath {
                    base: RelativeReferenceBase::SelfNode,
                    path_segments: vec!["Leaf".to_string()],
                },
                TreeReferenceInventoryVariant::RelativePathSelf,
                TreeReferenceInventoryStatus::AdmittedImplementationInput,
                HostReferenceCorrelationNeed::SourceTokenToFormalReference,
                NamespaceIdentityNeed::HostNamespaceVersion,
                CallerContextIdentityNeed::CallerNode,
            ),
            (
                TreeReference::RelativePath {
                    base: RelativeReferenceBase::Ancestor(2),
                    path_segments: vec!["Sibling".to_string()],
                },
                TreeReferenceInventoryVariant::RelativePathAncestor,
                TreeReferenceInventoryStatus::AdmittedCurrentCarrier,
                HostReferenceCorrelationNeed::SourceTokenToFormalReference,
                NamespaceIdentityNeed::HostNamespaceVersion,
                CallerContextIdentityNeed::AncestorWalk,
            ),
            (
                TreeReference::SiblingOffset {
                    offset: 1,
                    tail_segments: Vec::new(),
                },
                TreeReferenceInventoryVariant::SiblingOffset,
                TreeReferenceInventoryStatus::AdmittedCurrentCarrier,
                HostReferenceCorrelationNeed::SourceTokenToFormalReference,
                NamespaceIdentityNeed::HostNamespaceVersion,
                CallerContextIdentityNeed::SiblingPosition,
            ),
            (
                TreeReference::CrossWorkspaceResolved {
                    workspace_handle: "treecalc-workspace:projections".to_string(),
                    target_node_id: TreeNodeId(102),
                    target_node_handle: "treecalc-workspace:projections#node:102".to_string(),
                    availability_version:
                        "treecalc-cross-workspace-availability:v1:projections:loaded".to_string(),
                    carrier_id: "carrier:xws".to_string(),
                    detail: "resolved".to_string(),
                },
                TreeReferenceInventoryVariant::CrossWorkspaceReference,
                TreeReferenceInventoryStatus::AdmittedImplementationInput,
                HostReferenceCorrelationNeed::HostReferenceHandle,
                NamespaceIdentityNeed::CrossWorkspaceAvailabilityVersion,
                CallerContextIdentityNeed::None,
            ),
            (
                TreeReference::DynamicPotential {
                    carrier_id: "dyn".to_string(),
                    detail: "late".to_string(),
                },
                TreeReferenceInventoryVariant::DynamicPotential,
                TreeReferenceInventoryStatus::AdmittedCurrentCarrier,
                HostReferenceCorrelationNeed::RuntimeFactCarrier,
                NamespaceIdentityNeed::StructureContextVersion,
                CallerContextIdentityNeed::HostRuntimeContext,
            ),
            (
                TreeReference::Unresolved {
                    token: "../Missing".to_string(),
                },
                TreeReferenceInventoryVariant::Unresolved,
                TreeReferenceInventoryStatus::TypedExclusion,
                HostReferenceCorrelationNeed::SourceTokenToFormalReference,
                NamespaceIdentityNeed::HostNamespaceVersion,
                CallerContextIdentityNeed::CallerNode,
            ),
        ];

        for (
            reference,
            expected_variant,
            expected_status,
            expected_correlation,
            expected_namespace,
            expected_caller,
        ) in cases
        {
            let input = reference.implementation_input();
            assert_eq!(reference.inventory_variant(), expected_variant);
            assert_eq!(input.variant, expected_variant);
            assert_eq!(input.status, expected_status);
            assert_eq!(input.host_reference_correlation, expected_correlation);
            assert_eq!(input.namespace_identity_need, expected_namespace);
            assert_eq!(input.caller_context_identity_need, expected_caller);
            assert_eq!(input.carrier_class, Some(reference.carrier_class()));
        }
    }

    #[test]
    fn children_inventory_preserves_handle_correlation_and_dependency_facts() {
        let input = tree_reference_implementation_input(TreeReferenceInventoryVariant::ChildrenV1)
            .expect("ChildrenV1 inventory");

        assert_eq!(
            input.status,
            TreeReferenceInventoryStatus::AdmittedCurrentCarrier
        );
        assert_eq!(
            input.host_reference_correlation,
            HostReferenceCorrelationNeed::HostReferenceHandle
        );
        assert_eq!(
            input.dependency_facts,
            vec![
                DependencyDescriptorKind::TreeReferenceCollectionMembership,
                DependencyDescriptorKind::TreeReferenceCollectionMemberValue,
            ]
        );
        assert_eq!(
            input.invalidation_facts,
            vec![
                InvalidationReasonKind::TreeReferenceMembershipChanged,
                InvalidationReasonKind::TreeReferenceOrderChanged,
                InvalidationReasonKind::UpstreamPublication,
            ]
        );
    }

    #[test]
    fn ordered_selector_inventory_preserves_collection_dependency_facts() {
        for variant in [
            TreeReferenceInventoryVariant::SiblingSetSelector,
            TreeReferenceInventoryVariant::PrecedingFollowingSelector,
            TreeReferenceInventoryVariant::AncestorSetSelector,
            TreeReferenceInventoryVariant::RecursiveSelector,
        ] {
            let input =
                tree_reference_implementation_input(variant).expect("selector inventory input");
            assert_eq!(
                input.status,
                TreeReferenceInventoryStatus::AdmittedImplementationInput
            );
            assert_eq!(input.blocker, None);
            assert_eq!(
                input.host_reference_correlation,
                HostReferenceCorrelationNeed::HostReferenceHandle
            );
            assert_eq!(
                input.dependency_facts,
                vec![
                    DependencyDescriptorKind::TreeReferenceCollectionMembership,
                    DependencyDescriptorKind::TreeReferenceCollectionMemberValue,
                ]
            );
            assert!(
                input
                    .invalidation_facts
                    .contains(&InvalidationReasonKind::TreeReferenceMembershipChanged)
            );
            assert!(
                input
                    .invalidation_facts
                    .contains(&InvalidationReasonKind::TreeReferenceOrderChanged)
            );
        }
    }

    #[test]
    fn formula_catalog_lowers_direct_and_relative_references() {
        let catalog = TreeFormulaCatalog::new([TreeFormulaBinding {
            owner_node_id: TreeNodeId(4),
            formula_artifact_id: FormulaArtifactId("formula:leaf".to_string()),
            bind_artifact_id: Some(BindArtifactId("bind:leaf".to_string())),
            expression: fixture_formula(
                TreeNodeId(4),
                FixtureFormulaAst::Binary {
                    op: FixtureFormulaBinaryOp::Add,
                    left: Box::new(FixtureFormulaAst::Reference(TreeReference::DirectNode {
                        target_node_id: TreeNodeId(3),
                    })),
                    right: Box::new(FixtureFormulaAst::Reference(TreeReference::SiblingOffset {
                        offset: 1,
                        tail_segments: vec![],
                    })),
                },
            ),
        }]);

        let descriptors = catalog.to_dependency_descriptors(&snapshot());

        assert_eq!(descriptors.len(), 2);
        assert_eq!(descriptors[0].kind, DependencyDescriptorKind::StaticDirect);
        assert_eq!(descriptors[0].target_node_id, Some(TreeNodeId(3)));
        assert_eq!(descriptors[1].kind, DependencyDescriptorKind::RelativeBound);
        assert_eq!(descriptors[1].target_node_id, Some(TreeNodeId(5)));
    }

    #[test]
    fn formula_catalog_lowers_parent_and_ancestor_relative_references() {
        let catalog = TreeFormulaCatalog::new([TreeFormulaBinding {
            owner_node_id: TreeNodeId(4),
            formula_artifact_id: FormulaArtifactId("formula:leaf".to_string()),
            bind_artifact_id: Some(BindArtifactId("bind:leaf".to_string())),
            expression: fixture_formula(
                TreeNodeId(4),
                FixtureFormulaAst::Binary {
                    op: FixtureFormulaBinaryOp::Add,
                    left: Box::new(FixtureFormulaAst::Reference(TreeReference::RelativePath {
                        base: RelativeReferenceBase::ParentNode,
                        path_segments: vec!["Neighbor".to_string()],
                    })),
                    right: Box::new(FixtureFormulaAst::Reference(TreeReference::RelativePath {
                        base: RelativeReferenceBase::Ancestor(2),
                        path_segments: vec!["Sibling".to_string()],
                    })),
                },
            ),
        }]);

        let descriptors = catalog.to_dependency_descriptors(&snapshot());

        assert_eq!(descriptors.len(), 2);
        assert_eq!(descriptors[0].kind, DependencyDescriptorKind::RelativeBound);
        assert_eq!(descriptors[0].target_node_id, Some(TreeNodeId(5)));
        assert_eq!(descriptors[1].kind, DependencyDescriptorKind::RelativeBound);
        assert_eq!(descriptors[1].target_node_id, Some(TreeNodeId(3)));
    }

    #[test]
    fn formula_catalog_surfaces_host_sensitive_and_unresolved_references() {
        let snapshot = snapshot();
        let catalog = TreeFormulaCatalog::new([TreeFormulaBinding {
            owner_node_id: TreeNodeId(4),
            formula_artifact_id: FormulaArtifactId("formula:leaf".to_string()),
            bind_artifact_id: None,
            expression: fixture_formula(
                TreeNodeId(4),
                FixtureFormulaAst::FunctionCall {
                    function_name: "CHOOSE".to_string(),
                    arguments: vec![
                        FixtureFormulaAst::Reference(TreeReference::HostSensitive {
                            carrier_id: "host.selection".to_string(),
                            detail: "active branch".to_string(),
                        }),
                        FixtureFormulaAst::Reference(TreeReference::Unresolved {
                            token: "../Missing".to_string(),
                        }),
                    ],
                    may_introduce_dynamic_dependencies: true,
                },
            ),
        }]);

        let graph =
            DependencyGraph::build(&snapshot, &catalog.to_dependency_descriptors(&snapshot));

        assert_eq!(graph.diagnostics.len(), 2);
        assert_eq!(
            graph.diagnostics[0].kind,
            DependencyDiagnosticKind::HostSensitiveReference
        );
        assert_eq!(
            graph.diagnostics[1].kind,
            DependencyDiagnosticKind::UnresolvedReference
        );
    }

    #[test]
    fn formula_catalog_lowers_resolved_dynamic_reference_as_dynamic_edge() {
        let snapshot = snapshot();
        let catalog = TreeFormulaCatalog::new([TreeFormulaBinding {
            owner_node_id: TreeNodeId(4),
            formula_artifact_id: FormulaArtifactId("formula:dynamic-resolved".to_string()),
            bind_artifact_id: Some(BindArtifactId("bind:dynamic-resolved".to_string())),
            expression: fixture_formula(
                TreeNodeId(4),
                FixtureFormulaAst::Reference(TreeReference::DynamicResolved {
                    target_node_id: TreeNodeId(3),
                    carrier_id: "carrier:dynamic".to_string(),
                    detail: "resolved_late_bound_projection".to_string(),
                }),
            ),
        }]);

        let descriptors = catalog.to_dependency_descriptors(&snapshot);

        assert_eq!(descriptors.len(), 1);
        assert_eq!(
            descriptors[0].kind,
            DependencyDescriptorKind::DynamicPotential
        );
        assert_eq!(descriptors[0].target_node_id, Some(TreeNodeId(3)));
        assert!(descriptors[0].requires_rebind_on_structural_change);
        assert_eq!(
            descriptors[0].carrier_detail,
            "dynamic_resolved:node:3:carrier:dynamic:resolved_late_bound_projection"
        );
    }

    #[test]
    fn formula_catalog_lowers_children_collection_to_membership_and_member_value_edges() {
        let snapshot = snapshot();
        let collection = TreeCalcChildrenReferenceCollection::new(TreeNodeId(2), "@CHILDREN")
            .with_source_span_utf8(5, 14);
        let catalog = TreeFormulaCatalog::new([TreeFormulaBinding {
            owner_node_id: TreeNodeId(3),
            formula_artifact_id: FormulaArtifactId("formula:children".to_string()),
            bind_artifact_id: Some(BindArtifactId("bind:children".to_string())),
            expression: TreeFormula::opaque_oxfml(
                "=SUM(@CHILDREN)",
                [TreeReference::ReferenceCollection(
                    TreeCalcReferenceCollection::ChildrenV1(collection.clone()),
                )],
            ),
        }]);

        let descriptors = catalog.to_dependency_descriptors(&snapshot);

        assert_eq!(descriptors.len(), 3);
        assert_eq!(
            descriptors[0].kind,
            DependencyDescriptorKind::TreeReferenceCollectionMemberValue
        );
        assert_eq!(descriptors[0].target_node_id, Some(TreeNodeId(4)));
        assert_eq!(
            descriptors[0].source_reference_handle.as_deref(),
            Some(collection.host_ref_handle.as_str())
        );
        assert_eq!(
            descriptors[1].kind,
            DependencyDescriptorKind::TreeReferenceCollectionMemberValue
        );
        assert_eq!(descriptors[1].target_node_id, Some(TreeNodeId(5)));
        assert_eq!(
            descriptors[2].kind,
            DependencyDescriptorKind::TreeReferenceCollectionMembership
        );
        assert!(descriptors[2].target_node_id.is_none());
        assert!(
            descriptors[2]
                .carrier_detail
                .contains("members=node:4,node:5")
        );

        let graph = DependencyGraph::build(&snapshot, &descriptors);
        assert!(graph.diagnostics.is_empty());
        assert_eq!(graph.reverse_edges[&TreeNodeId(4)].len(), 1);
        assert_eq!(graph.reverse_edges[&TreeNodeId(5)].len(), 1);
    }

    #[test]
    fn formula_catalog_lowers_ordered_selector_collection_to_membership_and_member_value_edges() {
        let snapshot = snapshot();
        let collection = TreeCalcOrderedSelectorReferenceCollection::new(
            TreeCalcOrderedSelectorFamily::PrecedingV1,
            TreeNodeId(4),
            "@PRECEDING",
            [TreeNodeId(3), TreeNodeId(2)],
        )
        .with_source_span_utf8(5, 15);
        let catalog = TreeFormulaCatalog::new([TreeFormulaBinding {
            owner_node_id: TreeNodeId(4),
            formula_artifact_id: FormulaArtifactId("formula:preceding".to_string()),
            bind_artifact_id: Some(BindArtifactId("bind:preceding".to_string())),
            expression: TreeFormula::opaque_oxfml(
                "=SUM(@PRECEDING)",
                [TreeReference::ReferenceCollection(
                    TreeCalcReferenceCollection::OrderedSelectorV1(collection.clone()),
                )],
            ),
        }]);

        let descriptors = catalog.to_dependency_descriptors(&snapshot);

        assert_eq!(descriptors.len(), 3);
        assert_eq!(
            descriptors[0].kind,
            DependencyDescriptorKind::TreeReferenceCollectionMemberValue
        );
        assert_eq!(descriptors[0].target_node_id, Some(TreeNodeId(3)));
        assert_eq!(
            descriptors[1].kind,
            DependencyDescriptorKind::TreeReferenceCollectionMemberValue
        );
        assert_eq!(descriptors[1].target_node_id, Some(TreeNodeId(2)));
        assert_eq!(
            descriptors[2].kind,
            DependencyDescriptorKind::TreeReferenceCollectionMembership
        );
        let dependency = descriptors[2]
            .tree_reference_collection
            .as_ref()
            .expect("collection dependency");
        assert_eq!(
            dependency.family,
            TreeReferenceCollectionFamily::PrecedingV1
        );
        assert_eq!(
            dependency.member_node_ids,
            vec![TreeNodeId(3), TreeNodeId(2)]
        );
        assert!(descriptors[2].carrier_detail.contains("family=preceding"));

        let graph = DependencyGraph::build(&snapshot, &descriptors);
        assert!(graph.diagnostics.is_empty());
        assert_eq!(graph.reverse_edges[&TreeNodeId(3)].len(), 1);
        assert_eq!(graph.reverse_edges[&TreeNodeId(2)].len(), 1);
    }

    #[test]
    fn formula_catalog_lowers_reference_literal_array_with_order_and_duplicates() {
        let snapshot = snapshot();
        let collection = TreeCalcReferenceLiteralArrayCollection::reference_only(
            "array:q1",
            TreeNodeId(4),
            "{A,C,A}",
            [
                TreeCalcReferenceLiteralArrayElement::ReferenceNode(TreeNodeId(2)),
                TreeCalcReferenceLiteralArrayElement::ReferenceNode(TreeNodeId(3)),
                TreeCalcReferenceLiteralArrayElement::ReferenceNode(TreeNodeId(2)),
            ],
        )
        .expect("reference-only array")
        .with_source_span_utf8(5, 12);
        let catalog = TreeFormulaCatalog::new([TreeFormulaBinding {
            owner_node_id: TreeNodeId(4),
            formula_artifact_id: FormulaArtifactId("formula:reference-array".to_string()),
            bind_artifact_id: Some(BindArtifactId("bind:reference-array".to_string())),
            expression: TreeFormula::opaque_oxfml(
                "=SUM({A,C,A})",
                [TreeReference::ReferenceCollection(
                    TreeCalcReferenceCollection::ReferenceLiteralArrayV1(collection),
                )],
            ),
        }]);

        let descriptors = catalog.to_dependency_descriptors(&snapshot);

        assert_eq!(descriptors.len(), 4);
        assert_eq!(
            descriptors[0].kind,
            DependencyDescriptorKind::TreeReferenceCollectionMemberValue
        );
        assert_eq!(descriptors[0].target_node_id, Some(TreeNodeId(2)));
        assert_eq!(
            descriptors[0].source_reference_handle.as_deref(),
            Some("treecalc-hostref:v1:reference_literal_array:array:q1")
        );
        assert!(descriptors[0].carrier_detail.contains("ordinal=0"));
        assert_eq!(descriptors[1].target_node_id, Some(TreeNodeId(3)));
        assert!(descriptors[1].carrier_detail.contains("ordinal=1"));
        assert_eq!(descriptors[2].target_node_id, Some(TreeNodeId(2)));
        assert!(descriptors[2].carrier_detail.contains("ordinal=2"));
        assert_eq!(
            descriptors[3].kind,
            DependencyDescriptorKind::TreeReferenceCollectionMembership
        );
        let dependency = descriptors[3]
            .tree_reference_collection
            .as_ref()
            .expect("reference literal dependency");
        assert_eq!(
            dependency.family,
            TreeReferenceCollectionFamily::ReferenceLiteralArrayV1
        );
        assert_eq!(
            dependency.member_node_ids,
            vec![TreeNodeId(2), TreeNodeId(3), TreeNodeId(2)]
        );
        assert!(
            dependency
                .order_version
                .contains("members=node:2,node:3,node:2")
        );
        assert!(
            dependency
                .membership_version
                .contains("members=node:2,node:3")
        );

        let graph = DependencyGraph::build(&snapshot, &descriptors);
        assert!(graph.diagnostics.is_empty());
        assert_eq!(graph.reverse_edges[&TreeNodeId(2)].len(), 2);
        assert_eq!(graph.reverse_edges[&TreeNodeId(3)].len(), 1);
    }

    #[test]
    fn mixed_reference_array_is_typed_exclusion_before_lowering() {
        let error = TreeCalcReferenceLiteralArrayCollection::reference_only(
            "array:mixed",
            TreeNodeId(4),
            "{A,1,C}",
            [
                TreeCalcReferenceLiteralArrayElement::ReferenceNode(TreeNodeId(2)),
                TreeCalcReferenceLiteralArrayElement::ScalarValue {
                    source_text: "1".to_string(),
                },
                TreeCalcReferenceLiteralArrayElement::ReferenceNode(TreeNodeId(3)),
            ],
        )
        .expect_err("mixed scalar/reference array should not become reference-only carrier");

        assert_eq!(
            error,
            TreeCalcReferenceLiteralArrayError::MixedScalarReferenceArray {
                source_token_text: "{A,1,C}".to_string()
            }
        );
        let inventory =
            tree_reference_implementation_input(TreeReferenceInventoryVariant::MixedReferenceArray)
                .expect("mixed array inventory");
        assert_eq!(
            inventory.status,
            TreeReferenceInventoryStatus::TypedExclusion
        );
    }

    #[test]
    fn runtime_fact_references_are_not_formula_references() {
        let cases = [
            (
                TreeReference::HostSensitive {
                    carrier_id: "host.selection".to_string(),
                    detail: "active branch".to_string(),
                },
                DependencyDescriptorKind::HostSensitive,
                "host_sensitive:host.selection:active branch",
                true,
            ),
            (
                TreeReference::DynamicPotential {
                    carrier_id: "runtime.topic".to_string(),
                    detail: "late bound".to_string(),
                },
                DependencyDescriptorKind::DynamicPotential,
                "dynamic_potential:runtime.topic:late bound",
                false,
            ),
            (
                TreeReference::CapabilitySensitive {
                    carrier_id: "capability.profile".to_string(),
                    detail: "requires provider".to_string(),
                },
                DependencyDescriptorKind::CapabilitySensitive,
                "capability_sensitive:capability.profile:requires provider",
                true,
            ),
            (
                TreeReference::ShapeTopology {
                    carrier_id: "shape.range".to_string(),
                    detail: "range extent".to_string(),
                },
                DependencyDescriptorKind::ShapeTopology,
                "shape_topology:shape.range:range extent",
                true,
            ),
        ];

        for (reference, expected_kind, expected_detail, expected_rebind) in cases {
            assert_eq!(
                reference.carrier_class(),
                TreeReferenceCarrierClass::RuntimeFactProjection
            );
            assert_eq!(reference.resolve_target(&snapshot(), TreeNodeId(4)), None);
            assert_eq!(reference.descriptor_kind(), expected_kind);
            assert_eq!(reference.carrier_detail(), expected_detail);
            assert_eq!(
                reference.requires_rebind_on_structural_change(),
                expected_rebind
            );
        }
    }

    #[test]
    fn runtime_fact_carriers_surface_diagnostics_without_dependency_edges() {
        let snapshot = snapshot();
        let catalog = TreeFormulaCatalog::new([TreeFormulaBinding {
            owner_node_id: TreeNodeId(4),
            formula_artifact_id: FormulaArtifactId("formula:runtime-facts".to_string()),
            bind_artifact_id: Some(BindArtifactId("bind:runtime-facts".to_string())),
            expression: TreeFormula::opaque_oxfml(
                "=SUM(1,1)",
                [
                    TreeReference::HostSensitive {
                        carrier_id: "host.selection".to_string(),
                        detail: "active branch".to_string(),
                    },
                    TreeReference::DynamicPotential {
                        carrier_id: "runtime.topic".to_string(),
                        detail: "late bound".to_string(),
                    },
                    TreeReference::CapabilitySensitive {
                        carrier_id: "capability.profile".to_string(),
                        detail: "requires provider".to_string(),
                    },
                    TreeReference::ShapeTopology {
                        carrier_id: "shape.range".to_string(),
                        detail: "range extent".to_string(),
                    },
                ],
            ),
        }]);

        let descriptors = catalog.to_dependency_descriptors(&snapshot);
        assert_eq!(descriptors.len(), 4);
        assert!(
            descriptors
                .iter()
                .all(|descriptor| descriptor.target_node_id.is_none())
        );
        assert!(
            descriptors
                .iter()
                .all(|descriptor| descriptor.source_reference_handle.is_some())
        );

        let graph = DependencyGraph::build(&snapshot, &descriptors);
        assert!(graph.edges_by_owner.is_empty());
        assert_eq!(
            graph
                .diagnostics
                .iter()
                .map(|diagnostic| diagnostic.kind)
                .collect::<Vec<_>>(),
            vec![
                DependencyDiagnosticKind::HostSensitiveReference,
                DependencyDiagnosticKind::DynamicPotentialReference,
                DependencyDiagnosticKind::CapabilitySensitiveReference,
                DependencyDiagnosticKind::ShapeTopologyReference,
            ]
        );
    }

    #[test]
    fn first_closed_subset_rebind_flags_match_w026_floor() {
        assert!(
            !TreeReference::DirectNode {
                target_node_id: TreeNodeId(2)
            }
            .requires_rebind_on_structural_change()
        );
        assert!(
            TreeReference::RelativePath {
                base: RelativeReferenceBase::ParentNode,
                path_segments: vec!["A".to_string()],
            }
            .requires_rebind_on_structural_change()
        );
        assert!(
            TreeReference::RelativePath {
                base: RelativeReferenceBase::Ancestor(2),
                path_segments: vec!["A".to_string(), "Leaf".to_string()],
            }
            .requires_rebind_on_structural_change()
        );
        assert!(
            TreeReference::SiblingOffset {
                offset: 1,
                tail_segments: vec![],
            }
            .requires_rebind_on_structural_change()
        );
        assert!(
            TreeReference::HostSensitive {
                carrier_id: "host.selection".to_string(),
                detail: "active branch".to_string(),
            }
            .requires_rebind_on_structural_change()
        );
        assert!(
            !TreeReference::DynamicPotential {
                carrier_id: "runtime.topic".to_string(),
                detail: "late bound".to_string(),
            }
            .requires_rebind_on_structural_change()
        );
        assert!(
            TreeReference::DynamicResolved {
                target_node_id: TreeNodeId(2),
                carrier_id: "runtime.topic".to_string(),
                detail: "late bound".to_string(),
            }
            .requires_rebind_on_structural_change()
        );
        assert!(
            TreeReference::Unresolved {
                token: "../Missing".to_string(),
            }
            .requires_rebind_on_structural_change()
        );
    }

    #[test]
    fn qualified_parent_offset_resolves_base_parent_and_tail() {
        let snapshot = snapshot();

        // base = Leaf(4); parent of Leaf is Branch(2).
        let bare_parent = TreeReference::QualifiedParentOffset {
            base_node_id: TreeNodeId(4),
            tail_segments: Vec::new(),
        };
        assert_eq!(
            bare_parent.resolve_target(&snapshot, TreeNodeId(99)),
            Some(TreeNodeId(2))
        );

        // base = Leaf(4); parent Branch(2); tail Neighbor -> Neighbor(5).
        let tailed_parent = TreeReference::QualifiedParentOffset {
            base_node_id: TreeNodeId(4),
            tail_segments: vec!["Neighbor".to_string()],
        };
        assert_eq!(
            tailed_parent.resolve_target(&snapshot, TreeNodeId(99)),
            Some(TreeNodeId(5))
        );

        // base = Branch(2); parent Root(1); tail Sibling -> Sibling(3).
        let grand_tail = TreeReference::QualifiedParentOffset {
            base_node_id: TreeNodeId(2),
            tail_segments: vec!["Sibling".to_string()],
        };
        assert_eq!(
            grand_tail.resolve_target(&snapshot, TreeNodeId(99)),
            Some(TreeNodeId(3))
        );

        // Owner identity is irrelevant for a base-qualified reference: meta-visibility
        // resolution delegates to resolve_target for this variant.
        assert_eq!(
            bare_parent.resolve_target_with_meta_visibility(
                &snapshot,
                TreeNodeId(99),
                &BTreeSet::new(),
            ),
            Some(TreeNodeId(2))
        );

        // Missing tail child resolves to None.
        let missing_tail = TreeReference::QualifiedParentOffset {
            base_node_id: TreeNodeId(4),
            tail_segments: vec!["Absent".to_string()],
        };
        assert_eq!(missing_tail.resolve_target(&snapshot, TreeNodeId(99)), None);

        // Root has no parent.
        let no_parent = TreeReference::QualifiedParentOffset {
            base_node_id: TreeNodeId(1),
            tail_segments: Vec::new(),
        };
        assert_eq!(no_parent.resolve_target(&snapshot, TreeNodeId(99)), None);

        assert_eq!(
            bare_parent.descriptor_kind(),
            DependencyDescriptorKind::RelativeBound
        );
        assert_eq!(
            bare_parent.inventory_variant(),
            TreeReferenceInventoryVariant::RelativePathParent
        );
        assert_eq!(
            bare_parent.carrier_class(),
            TreeReferenceCarrierClass::FormulaReference
        );
        assert!(bare_parent.requires_rebind_on_structural_change());
        assert_eq!(
            tailed_parent.carrier_detail(),
            "qualified_parent_offset:base=node:4:Neighbor"
        );
    }

    #[test]
    fn raw_oxfml_formula_lowers_declared_explicit_references() {
        let snapshot = snapshot();
        let catalog = TreeFormulaCatalog::new([TreeFormulaBinding {
            owner_node_id: TreeNodeId(4),
            formula_artifact_id: FormulaArtifactId("formula:raw-let-lambda".to_string()),
            bind_artifact_id: Some(BindArtifactId("bind:raw-let-lambda".to_string())),
            expression: fixture_formula(
                TreeNodeId(4),
                FixtureFormulaAst::RawOxfml {
                    source_text: "LET(base,TREE_REF_4_0,LAMBDA(delta,base+delta)(5))".to_string(),
                    explicit_references: vec![
                        TreeReference::DirectNode {
                            target_node_id: TreeNodeId(3),
                        },
                        TreeReference::HostSensitive {
                            carrier_id: "carrier:lambda.host".to_string(),
                            detail: "call_argument_host_query".to_string(),
                        },
                    ],
                },
            ),
        }]);

        let descriptors = catalog.to_dependency_descriptors(&snapshot);

        assert_eq!(descriptors.len(), 2);
        assert_eq!(descriptors[0].kind, DependencyDescriptorKind::StaticDirect);
        assert_eq!(
            descriptors[0].carrier_detail,
            "direct_node:node:3".to_string()
        );
        assert_eq!(descriptors[1].kind, DependencyDescriptorKind::HostSensitive);
        assert_eq!(
            descriptors[1].carrier_detail,
            "host_sensitive:carrier:lambda.host:call_argument_host_query".to_string()
        );
    }
}
