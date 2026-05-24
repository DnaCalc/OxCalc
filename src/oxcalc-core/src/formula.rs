#![forbid(unsafe_code)]

//! TreeCalc-local formula and reference substrate.

use std::collections::BTreeMap;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TreeCalcFormulaTextPrebindDiagnosticCode {
    UnsupportedSelector,
    UnsupportedQualifiedHostReference,
    UnsupportedRawTreeCalcReference,
    MissingOrderedSelectorResolution,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreeCalcFormulaTextPrebindDiagnostic {
    pub code: TreeCalcFormulaTextPrebindDiagnosticCode,
    pub source_span_utf8: (usize, usize),
    pub source_token_text: String,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("TreeCalc formula text prebind failed with {diagnostics_len} diagnostic(s)", diagnostics_len = .diagnostics.len())]
pub struct TreeCalcFormulaTextPrebindError {
    pub diagnostics: Vec<TreeCalcFormulaTextPrebindDiagnostic>,
}

#[must_use]
pub fn treecalc_formula_text_needs_prebind(source_text: &str) -> bool {
    scan_treecalc_formula_text(source_text, TreeNodeId(0)).requires_prebind
}

pub fn prebind_treecalc_formula_text(
    owner_node_id: TreeNodeId,
    source_text: impl AsRef<str>,
) -> Result<TreeFormula, TreeCalcFormulaTextPrebindError> {
    prebind_treecalc_formula_text_with_context(
        owner_node_id,
        source_text,
        &TreeCalcFormulaTextPrebindContext::default(),
    )
}

pub fn prebind_treecalc_formula_text_with_resolved_bases(
    owner_node_id: TreeNodeId,
    source_text: impl AsRef<str>,
    qualified_children_bases: impl IntoIterator<Item = TreeCalcQualifiedChildrenBaseResolution>,
) -> Result<TreeFormula, TreeCalcFormulaTextPrebindError> {
    let context =
        TreeCalcFormulaTextPrebindContext::with_qualified_children_bases(qualified_children_bases);
    prebind_treecalc_formula_text_with_context(owner_node_id, source_text, &context)
}

pub fn prebind_treecalc_formula_text_with_resolved_ordered_selectors(
    owner_node_id: TreeNodeId,
    source_text: impl AsRef<str>,
    ordered_selector_resolutions: impl IntoIterator<Item = TreeCalcOrderedSelectorResolution>,
) -> Result<TreeFormula, TreeCalcFormulaTextPrebindError> {
    let context = TreeCalcFormulaTextPrebindContext::with_ordered_selector_resolutions(
        ordered_selector_resolutions,
    );
    prebind_treecalc_formula_text_with_context(owner_node_id, source_text, &context)
}

#[must_use]
pub fn treecalc_formula_text_qualified_children_base_queries(
    owner_node_id: TreeNodeId,
    source_text: impl AsRef<str>,
) -> Vec<TreeCalcQualifiedChildrenBaseQuery> {
    let source_text = source_text.as_ref();
    scan_treecalc_formula_text(source_text, owner_node_id)
        .children_references
        .iter()
        .filter_map(|reference| {
            TreeCalcQualifiedChildrenBaseQuery::from_raw_reference(source_text, reference)
        })
        .collect()
}

#[must_use]
pub fn treecalc_formula_text_ordered_selector_queries(
    owner_node_id: TreeNodeId,
    source_text: impl AsRef<str>,
) -> Vec<TreeCalcOrderedSelectorQuery> {
    let source_text = source_text.as_ref();
    scan_treecalc_formula_text(source_text, owner_node_id)
        .ordered_selector_references
        .iter()
        .map(|reference| TreeCalcOrderedSelectorQuery::from_raw_reference(source_text, reference))
        .collect()
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreeCalcFormulaTextPrebindContext {
    #[serde(default)]
    pub qualified_children_bases: Vec<TreeCalcQualifiedChildrenBaseResolution>,
    #[serde(default)]
    pub ordered_selector_resolutions: Vec<TreeCalcOrderedSelectorResolution>,
}

impl TreeCalcFormulaTextPrebindContext {
    #[must_use]
    pub fn with_qualified_children_bases(
        qualified_children_bases: impl IntoIterator<Item = TreeCalcQualifiedChildrenBaseResolution>,
    ) -> Self {
        Self {
            qualified_children_bases: qualified_children_bases.into_iter().collect(),
            ordered_selector_resolutions: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_ordered_selector_resolutions(
        ordered_selector_resolutions: impl IntoIterator<Item = TreeCalcOrderedSelectorResolution>,
    ) -> Self {
        Self {
            qualified_children_bases: Vec::new(),
            ordered_selector_resolutions: ordered_selector_resolutions.into_iter().collect(),
        }
    }

    fn resolved_base_for(&self, reference: &RawChildrenReference) -> Option<TreeNodeId> {
        let RawChildrenReferenceBase::Qualified = reference.base else {
            return Some(reference.default_base_node_id);
        };
        let base_span_utf8 = reference.base_span_utf8?;
        let selector_span_utf8 = (reference.selector_start_byte, reference.selector_end_byte);
        self.qualified_children_bases
            .iter()
            .find(|resolution| {
                resolution.source_span_utf8 == (reference.start_byte, reference.end_byte)
                    && resolution.base_span_utf8 == base_span_utf8
                    && resolution.selector_span_utf8 == selector_span_utf8
                    && resolution.source_token_text == reference.source_token_text
            })
            .map(|resolution| resolution.base_node_id)
    }

    fn ordered_selector_resolution_for(
        &self,
        reference: &RawOrderedSelectorReference,
    ) -> Option<&TreeCalcOrderedSelectorResolution> {
        self.ordered_selector_resolutions.iter().find(|resolution| {
            resolution.family == reference.family
                && resolution.source_span_utf8 == (reference.start_byte, reference.end_byte)
                && resolution.base_span_utf8 == reference.base_span_utf8
                && resolution.selector_span_utf8
                    == (reference.selector_start_byte, reference.selector_end_byte)
                && resolution.tail_span_utf8 == reference.tail_span_utf8
                && resolution.source_token_text == reference.source_token_text
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TreeCalcQualifiedBaseResolutionLayer {
    CallerSuppliedResolvedBase,
    ExplicitHostPath,
    OxCalcStructuralPath,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreeCalcQualifiedChildrenBaseResolution {
    pub source_span_utf8: (usize, usize),
    pub base_span_utf8: (usize, usize),
    pub selector_span_utf8: (usize, usize),
    pub source_token_text: String,
    pub base_node_id: TreeNodeId,
    pub resolution_layer: TreeCalcQualifiedBaseResolutionLayer,
    pub resolution_identity: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreeCalcQualifiedChildrenBaseQuery {
    pub source_span_utf8: (usize, usize),
    pub base_span_utf8: (usize, usize),
    pub selector_span_utf8: (usize, usize),
    pub source_token_text: String,
    pub base_token_text: String,
    pub selector_token_text: String,
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
        && registry.aliases.get(selector_token_text).is_none()
        && registry
            .workspaces_by_handle
            .get(selector_token_text)
            .is_none()
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

fn split_treecalc_host_path_token(
    base_token_text: &str,
) -> Result<Vec<String>, TreeCalcHostPathBaseResolutionError> {
    let mut segments = Vec::new();
    let mut segment = String::new();
    let mut bracket_depth = 0usize;
    for ch in base_token_text.chars() {
        match ch {
            '[' if bracket_depth > 0 => {
                return Err(
                    TreeCalcHostPathBaseResolutionError::InvalidBaseTokenSyntax {
                        base_token_text: base_token_text.to_string(),
                        detail: "nested bracketed path segments are not admitted".to_string(),
                    },
                );
            }
            '[' => {
                bracket_depth += 1;
            }
            ']' if bracket_depth > 0 => {
                bracket_depth -= 1;
            }
            ']' => {
                return Err(
                    TreeCalcHostPathBaseResolutionError::InvalidBaseTokenSyntax {
                        base_token_text: base_token_text.to_string(),
                        detail: "closing bracket without an open bracket".to_string(),
                    },
                );
            }
            '.' | '/' if bracket_depth == 0 => {
                push_treecalc_host_path_segment(&mut segments, &mut segment);
            }
            '!' if bracket_depth == 0 => {
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
    if bracket_depth != 0 {
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

impl TreeCalcQualifiedChildrenBaseQuery {
    fn from_raw_reference(source_text: &str, reference: &RawChildrenReference) -> Option<Self> {
        let RawChildrenReferenceBase::Qualified = reference.base else {
            return None;
        };
        let base_span_utf8 = reference.base_span_utf8?;
        let selector_span_utf8 = (reference.selector_start_byte, reference.selector_end_byte);
        Some(Self {
            source_span_utf8: (reference.start_byte, reference.end_byte),
            base_span_utf8,
            selector_span_utf8,
            source_token_text: reference.source_token_text.clone(),
            base_token_text: source_text[base_span_utf8.0..base_span_utf8.1].to_string(),
            selector_token_text: source_text[selector_span_utf8.0..selector_span_utf8.1]
                .to_string(),
        })
    }

    #[must_use]
    pub fn to_resolution(
        &self,
        base_node_id: TreeNodeId,
    ) -> TreeCalcQualifiedChildrenBaseResolution {
        TreeCalcQualifiedChildrenBaseResolution::new(
            self.source_span_utf8,
            self.base_span_utf8,
            self.selector_span_utf8,
            self.source_token_text.clone(),
            base_node_id,
        )
    }

    #[must_use]
    pub fn to_resolution_with_layer(
        &self,
        base_node_id: TreeNodeId,
        resolution_layer: TreeCalcQualifiedBaseResolutionLayer,
        resolution_identity: impl Into<String>,
    ) -> TreeCalcQualifiedChildrenBaseResolution {
        self.to_resolution(base_node_id)
            .with_resolution_layer(resolution_layer, resolution_identity)
    }

    pub fn to_resolution_with_structural_path_base(
        &self,
        snapshot: &StructuralSnapshot,
    ) -> Result<TreeCalcQualifiedChildrenBaseResolution, TreeCalcHostPathBaseResolutionError> {
        let base = resolve_treecalc_explicit_host_path_base(snapshot, &self.base_token_text)?;
        Ok(self.to_resolution_with_layer(
            base.base_node_id,
            TreeCalcQualifiedBaseResolutionLayer::OxCalcStructuralPath,
            base.resolution_identity,
        ))
    }
}

impl TreeCalcQualifiedChildrenBaseResolution {
    #[must_use]
    pub fn new(
        source_span_utf8: (usize, usize),
        base_span_utf8: (usize, usize),
        selector_span_utf8: (usize, usize),
        source_token_text: impl Into<String>,
        base_node_id: TreeNodeId,
    ) -> Self {
        Self {
            source_span_utf8,
            base_span_utf8,
            selector_span_utf8,
            source_token_text: source_token_text.into(),
            base_node_id,
            resolution_layer: TreeCalcQualifiedBaseResolutionLayer::CallerSuppliedResolvedBase,
            resolution_identity: format!(
                "treecalc-qualified-base:v1:source={}-{};base={base_node_id}",
                source_span_utf8.0, source_span_utf8.1
            ),
        }
    }

    #[must_use]
    pub fn with_resolution_layer(
        mut self,
        resolution_layer: TreeCalcQualifiedBaseResolutionLayer,
        resolution_identity: impl Into<String>,
    ) -> Self {
        self.resolution_layer = resolution_layer;
        self.resolution_identity = resolution_identity.into();
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TreeCalcOrderedSelectorResolutionLayer {
    CallerSuppliedResolvedCollection,
    ExplicitHostPath,
    OxCalcStructuralTraversal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreeCalcOrderedSelectorResolution {
    pub family: TreeCalcOrderedSelectorFamily,
    pub source_span_utf8: (usize, usize),
    pub base_span_utf8: Option<(usize, usize)>,
    pub selector_span_utf8: (usize, usize),
    pub tail_span_utf8: Option<(usize, usize)>,
    pub source_token_text: String,
    pub base_node_id: TreeNodeId,
    pub member_node_ids: Vec<TreeNodeId>,
    pub resolution_layer: TreeCalcOrderedSelectorResolutionLayer,
    pub resolution_identity: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreeCalcOrderedSelectorQuery {
    pub family: TreeCalcOrderedSelectorFamily,
    pub source_span_utf8: (usize, usize),
    pub base_span_utf8: Option<(usize, usize)>,
    pub selector_span_utf8: (usize, usize),
    pub tail_span_utf8: Option<(usize, usize)>,
    pub source_token_text: String,
    pub base_token_text: Option<String>,
    pub selector_token_text: String,
    pub tail_token_text: Option<String>,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreeCalcOrderedSelectorTraversalResolution {
    pub resolution: TreeCalcOrderedSelectorResolution,
    pub traversal: TreeCalcOrderedSelectorTraversalResult,
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

impl TreeCalcOrderedSelectorQuery {
    fn from_raw_reference(source_text: &str, reference: &RawOrderedSelectorReference) -> Self {
        Self {
            family: reference.family,
            source_span_utf8: (reference.start_byte, reference.end_byte),
            base_span_utf8: reference.base_span_utf8,
            selector_span_utf8: (reference.selector_start_byte, reference.selector_end_byte),
            tail_span_utf8: reference.tail_span_utf8,
            source_token_text: reference.source_token_text.clone(),
            base_token_text: reference
                .base_span_utf8
                .map(|(start, end)| source_text[start..end].to_string()),
            selector_token_text: source_text
                [reference.selector_start_byte..reference.selector_end_byte]
                .to_string(),
            tail_token_text: reference
                .tail_span_utf8
                .map(|(start, end)| source_text[start..end].to_string()),
        }
    }

    #[must_use]
    pub fn to_resolution(
        &self,
        base_node_id: TreeNodeId,
        member_node_ids: impl IntoIterator<Item = TreeNodeId>,
    ) -> TreeCalcOrderedSelectorResolution {
        TreeCalcOrderedSelectorResolution::new(
            self.family,
            self.source_span_utf8,
            self.base_span_utf8,
            self.selector_span_utf8,
            self.tail_span_utf8,
            self.source_token_text.clone(),
            base_node_id,
            member_node_ids,
        )
    }

    #[must_use]
    pub fn to_resolution_with_layer(
        &self,
        base_node_id: TreeNodeId,
        member_node_ids: impl IntoIterator<Item = TreeNodeId>,
        resolution_layer: TreeCalcOrderedSelectorResolutionLayer,
        resolution_identity: impl Into<String>,
    ) -> TreeCalcOrderedSelectorResolution {
        self.to_resolution(base_node_id, member_node_ids)
            .with_resolution_layer(resolution_layer, resolution_identity)
    }

    pub fn to_resolution_with_structural_traversal(
        &self,
        snapshot: &StructuralSnapshot,
        base_node_id: TreeNodeId,
        policy: TreeCalcOrderedSelectorTraversalPolicy,
    ) -> Result<TreeCalcOrderedSelectorTraversalResolution, TreeCalcOrderedSelectorTraversalError>
    {
        let tail_segments = ordered_selector_tail_segments(self.tail_token_text.as_deref());
        let traversal = resolve_treecalc_ordered_selector_traversal(
            snapshot,
            self.family,
            base_node_id,
            &tail_segments,
            policy,
        )?;
        let resolution_identity = format!(
            "treecalc-ordered-selector:v1:family={};source={}-{};base={base_node_id};resolver=oxcalc_structural_traversal;{}",
            self.family.stable_id(),
            self.source_span_utf8.0,
            self.source_span_utf8.1,
            traversal.traversal_policy_id
        );
        let resolution = self.to_resolution_with_layer(
            base_node_id,
            traversal.member_node_ids.clone(),
            TreeCalcOrderedSelectorResolutionLayer::OxCalcStructuralTraversal,
            resolution_identity,
        );
        Ok(TreeCalcOrderedSelectorTraversalResolution {
            resolution,
            traversal,
        })
    }

    pub fn to_resolution_with_structural_path_base_and_traversal(
        &self,
        snapshot: &StructuralSnapshot,
        policy: TreeCalcOrderedSelectorTraversalPolicy,
    ) -> Result<
        TreeCalcOrderedSelectorTraversalResolution,
        TreeCalcOrderedSelectorStructuralResolutionError,
    > {
        let base_token_text = self
            .base_token_text
            .as_deref()
            .ok_or(TreeCalcOrderedSelectorStructuralResolutionError::MissingQualifiedBaseToken)?;
        let base = resolve_treecalc_explicit_host_path_base(snapshot, base_token_text)?;
        let mut resolved = self
            .to_resolution_with_structural_traversal(snapshot, base.base_node_id, policy)
            .map_err(TreeCalcOrderedSelectorStructuralResolutionError::from)?;
        resolved.resolution.resolution_identity = format!(
            "{};base_resolution={}",
            resolved.resolution.resolution_identity, base.resolution_identity
        );
        Ok(resolved)
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum TreeCalcOrderedSelectorStructuralResolutionError {
    #[error("ordered selector query does not have a qualified base token")]
    MissingQualifiedBaseToken,
    #[error(transparent)]
    BasePath(#[from] TreeCalcHostPathBaseResolutionError),
    #[error(transparent)]
    Traversal(#[from] TreeCalcOrderedSelectorTraversalError),
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
        TreeCalcOrderedSelectorFamily::SiblingSetV1 => sibling_window(snapshot, base_node_id, None),
        TreeCalcOrderedSelectorFamily::PrecedingV1 => {
            sibling_window(snapshot, base_node_id, Some(SiblingWindow::Preceding))
        }
        TreeCalcOrderedSelectorFamily::FollowingV1 => {
            sibling_window(snapshot, base_node_id, Some(SiblingWindow::Following))
        }
        TreeCalcOrderedSelectorFamily::AncestorsV1 => {
            let mut ancestors = Vec::new();
            let mut cursor = base_node.parent_id;
            while let Some(parent_id) = cursor {
                ancestors.push(parent_id);
                cursor = snapshot
                    .try_get_node(parent_id)
                    .and_then(|node| node.parent_id);
            }
            ancestors
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

fn ordered_selector_tail_segments(tail_token_text: Option<&str>) -> Vec<String> {
    tail_token_text
        .map(|tail| {
            tail.trim_start_matches('.')
                .split('.')
                .filter(|segment| !segment.is_empty())
                .map(ToString::to_string)
                .collect()
        })
        .unwrap_or_default()
}

impl TreeCalcOrderedSelectorResolution {
    #[must_use]
    pub fn new(
        family: TreeCalcOrderedSelectorFamily,
        source_span_utf8: (usize, usize),
        base_span_utf8: Option<(usize, usize)>,
        selector_span_utf8: (usize, usize),
        tail_span_utf8: Option<(usize, usize)>,
        source_token_text: impl Into<String>,
        base_node_id: TreeNodeId,
        member_node_ids: impl IntoIterator<Item = TreeNodeId>,
    ) -> Self {
        Self {
            family,
            source_span_utf8,
            base_span_utf8,
            selector_span_utf8,
            tail_span_utf8,
            source_token_text: source_token_text.into(),
            base_node_id,
            member_node_ids: member_node_ids.into_iter().collect(),
            resolution_layer:
                TreeCalcOrderedSelectorResolutionLayer::CallerSuppliedResolvedCollection,
            resolution_identity: format!(
                "treecalc-ordered-selector:v1:family={};source={}-{};base={base_node_id}",
                family.stable_id(),
                source_span_utf8.0,
                source_span_utf8.1
            ),
        }
    }

    #[must_use]
    pub fn with_resolution_layer(
        mut self,
        resolution_layer: TreeCalcOrderedSelectorResolutionLayer,
        resolution_identity: impl Into<String>,
    ) -> Self {
        self.resolution_layer = resolution_layer;
        self.resolution_identity = resolution_identity.into();
        self
    }
}

pub fn prebind_treecalc_formula_text_with_context(
    owner_node_id: TreeNodeId,
    source_text: impl AsRef<str>,
    context: &TreeCalcFormulaTextPrebindContext,
) -> Result<TreeFormula, TreeCalcFormulaTextPrebindError> {
    let source_text = source_text.as_ref();
    let mut scan = scan_treecalc_formula_text(source_text, owner_node_id);
    for reference in &scan.children_references {
        if matches!(reference.base, RawChildrenReferenceBase::Qualified)
            && context.resolved_base_for(reference).is_none()
        {
            scan.diagnostics.push(prebind_diagnostic(
                TreeCalcFormulaTextPrebindDiagnosticCode::UnsupportedQualifiedHostReference,
                source_text,
                reference.start_byte,
                reference.end_byte,
                "qualified TreeCalc children selectors require a caller-supplied resolved base",
            ));
        }
    }
    for reference in &scan.ordered_selector_references {
        if context.ordered_selector_resolution_for(reference).is_none() {
            scan.diagnostics.push(prebind_diagnostic(
                TreeCalcFormulaTextPrebindDiagnosticCode::MissingOrderedSelectorResolution,
                source_text,
                reference.start_byte,
                reference.end_byte,
                "ordered TreeCalc selectors require a caller-supplied resolved collection",
            ));
        }
    }
    if !scan.diagnostics.is_empty() {
        return Err(TreeCalcFormulaTextPrebindError {
            diagnostics: scan.diagnostics,
        });
    }

    if scan.children_references.is_empty() && scan.ordered_selector_references.is_empty() {
        return Ok(TreeFormula::opaque_oxfml(source_text, Vec::new()));
    }

    let mut prebound_references =
        Vec::with_capacity(scan.children_references.len() + scan.ordered_selector_references.len());
    for reference in &scan.children_references {
        let collection = TreeCalcChildrenReferenceCollection::new(
            context
                .resolved_base_for(reference)
                .expect("qualified references were checked before rewrite"),
            reference.source_token_text.clone(),
        )
        .with_source_span_utf8(reference.start_byte, reference.end_byte);
        prebound_references.push(PreboundTreeCalcReference {
            start_byte: reference.start_byte,
            end_byte: reference.end_byte,
            reference: TreeReference::ReferenceCollection(TreeCalcReferenceCollection::ChildrenV1(
                collection,
            )),
        });
    }
    for reference in &scan.ordered_selector_references {
        let resolution = context
            .ordered_selector_resolution_for(reference)
            .expect("ordered selector resolutions were checked before rewrite");
        let collection = TreeCalcOrderedSelectorReferenceCollection::new(
            resolution.family,
            resolution.base_node_id,
            resolution.source_token_text.clone(),
            resolution.member_node_ids.clone(),
        )
        .with_source_span_utf8(reference.start_byte, reference.end_byte);
        prebound_references.push(PreboundTreeCalcReference {
            start_byte: reference.start_byte,
            end_byte: reference.end_byte,
            reference: TreeReference::ReferenceCollection(
                TreeCalcReferenceCollection::OrderedSelectorV1(collection),
            ),
        });
    }
    prebound_references.sort_by_key(|reference| reference.start_byte);

    let mut rewritten = String::with_capacity(source_text.len());
    let mut carriers = Vec::with_capacity(prebound_references.len());
    let mut cursor = 0;
    for (reference_index, reference) in prebound_references.into_iter().enumerate() {
        let neutral_token = format!("TREE_REF_{}_{}", owner_node_id.0, reference_index);
        rewritten.push_str(&source_text[cursor..reference.start_byte]);
        rewritten.push_str(&neutral_token);
        cursor = reference.end_byte;

        carriers.push(TreeFormulaReferenceCarrier::named(
            neutral_token,
            reference.reference,
        ));
    }
    rewritten.push_str(&source_text[cursor..]);

    Ok(TreeFormula::opaque_oxfml(rewritten, carriers))
}

struct PreboundTreeCalcReference {
    start_byte: usize,
    end_byte: usize,
    reference: TreeReference,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TreeCalcFormulaTextScan {
    children_references: Vec<RawChildrenReference>,
    ordered_selector_references: Vec<RawOrderedSelectorReference>,
    diagnostics: Vec<TreeCalcFormulaTextPrebindDiagnostic>,
    requires_prebind: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RawChildrenReference {
    start_byte: usize,
    end_byte: usize,
    selector_start_byte: usize,
    selector_end_byte: usize,
    source_token_text: String,
    default_base_node_id: TreeNodeId,
    base: RawChildrenReferenceBase,
    base_span_utf8: Option<(usize, usize)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RawChildrenReferenceBase {
    Caller,
    Qualified,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RawOrderedSelectorReference {
    family: TreeCalcOrderedSelectorFamily,
    start_byte: usize,
    end_byte: usize,
    selector_start_byte: usize,
    selector_end_byte: usize,
    tail_span_utf8: Option<(usize, usize)>,
    source_token_text: String,
    default_base_node_id: TreeNodeId,
    base: RawOrderedSelectorReferenceBase,
    base_span_utf8: Option<(usize, usize)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RawOrderedSelectorReferenceBase {
    Caller,
    Qualified,
}

fn scan_treecalc_formula_text(
    source_text: &str,
    owner_node_id: TreeNodeId,
) -> TreeCalcFormulaTextScan {
    let mut children_references = Vec::new();
    let mut ordered_selector_references = Vec::new();
    let mut diagnostics = Vec::new();
    let mut requires_prebind = false;
    let mut index = 0;
    let mut in_string = false;

    while index < source_text.len() {
        let Some(current) = source_text[index..].chars().next() else {
            break;
        };

        if in_string {
            if current == '"' {
                let next_index = index + current.len_utf8();
                if source_text[next_index..].starts_with('"') {
                    index = next_index + 1;
                } else {
                    in_string = false;
                    index = next_index;
                }
            } else {
                index += current.len_utf8();
            }
            continue;
        }

        if current == '"' {
            in_string = true;
            index += current.len_utf8();
            continue;
        }

        if source_text[index..].starts_with("@CHILDREN") {
            requires_prebind = true;
            let end_byte = index + "@CHILDREN".len();
            if explicit_qualified_children_selector_start(source_text, index).is_some() {
                let start_byte = explicit_qualified_children_selector_start(source_text, index)
                    .expect("qualified selector start checked above");
                if !is_token_boundary_after(source_text, end_byte) {
                    let end = token_end(source_text, index);
                    diagnostics.push(prebind_diagnostic(
                        TreeCalcFormulaTextPrebindDiagnosticCode::UnsupportedRawTreeCalcReference,
                        source_text,
                        start_byte,
                        end,
                        "only the exact qualified @CHILDREN selector is admitted in this prebind surface",
                    ));
                } else {
                    children_references.push(RawChildrenReference {
                        start_byte,
                        end_byte,
                        selector_start_byte: index,
                        selector_end_byte: end_byte,
                        source_token_text: source_text[start_byte..end_byte].to_string(),
                        default_base_node_id: owner_node_id,
                        base: RawChildrenReferenceBase::Qualified,
                        base_span_utf8: Some((start_byte, index - 1)),
                    });
                }
            } else if !previous_char_allows_free_standing_selector(source_text, index) {
                let start_byte = host_path_start(source_text, index);
                diagnostics.push(prebind_diagnostic(
                    TreeCalcFormulaTextPrebindDiagnosticCode::UnsupportedQualifiedHostReference,
                    source_text,
                    start_byte,
                    end_byte,
                    "qualified TreeCalc children selectors require caller-supplied path resolution",
                ));
            } else if !is_token_boundary_after(source_text, end_byte) {
                let end = token_end(source_text, index);
                diagnostics.push(prebind_diagnostic(
                    TreeCalcFormulaTextPrebindDiagnosticCode::UnsupportedRawTreeCalcReference,
                    source_text,
                    index,
                    end,
                    "only the exact @CHILDREN selector is admitted in this prebind surface",
                ));
            } else {
                children_references.push(RawChildrenReference {
                    start_byte: index,
                    end_byte,
                    selector_start_byte: index,
                    selector_end_byte: end_byte,
                    source_token_text: source_text[index..end_byte].to_string(),
                    default_base_node_id: owner_node_id,
                    base: RawChildrenReferenceBase::Caller,
                    base_span_utf8: None,
                });
            }
            index = end_byte;
            continue;
        }

        if let Some((family, selector_text)) = ordered_at_selector_at(source_text, index) {
            requires_prebind = true;
            let end_byte = index + selector_text.len();
            if explicit_qualified_at_selector_start(source_text, index).is_some() {
                let start_byte = explicit_qualified_at_selector_start(source_text, index)
                    .expect("qualified selector start checked above");
                if !is_token_boundary_after(source_text, end_byte) {
                    let end = token_end(source_text, index);
                    diagnostics.push(prebind_diagnostic(
                        TreeCalcFormulaTextPrebindDiagnosticCode::UnsupportedRawTreeCalcReference,
                        source_text,
                        start_byte,
                        end,
                        "only exact qualified ordered TreeCalc selector tokens are admitted in this prebind surface",
                    ));
                } else {
                    ordered_selector_references.push(RawOrderedSelectorReference {
                        family,
                        start_byte,
                        end_byte,
                        selector_start_byte: index,
                        selector_end_byte: end_byte,
                        tail_span_utf8: None,
                        source_token_text: source_text[start_byte..end_byte].to_string(),
                        default_base_node_id: owner_node_id,
                        base: RawOrderedSelectorReferenceBase::Qualified,
                        base_span_utf8: Some((start_byte, index - 1)),
                    });
                }
            } else if !previous_char_allows_free_standing_selector(source_text, index) {
                let start_byte = host_path_start(source_text, index);
                diagnostics.push(prebind_diagnostic(
                    TreeCalcFormulaTextPrebindDiagnosticCode::UnsupportedQualifiedHostReference,
                    source_text,
                    start_byte,
                    end_byte,
                    "qualified ordered TreeCalc selectors require caller-supplied collection resolution",
                ));
            } else if !is_token_boundary_after(source_text, end_byte) {
                let end = token_end(source_text, index);
                diagnostics.push(prebind_diagnostic(
                    TreeCalcFormulaTextPrebindDiagnosticCode::UnsupportedRawTreeCalcReference,
                    source_text,
                    index,
                    end,
                    "only exact ordered TreeCalc selector tokens are admitted in this prebind surface",
                ));
            } else {
                ordered_selector_references.push(RawOrderedSelectorReference {
                    family,
                    start_byte: index,
                    end_byte,
                    selector_start_byte: index,
                    selector_end_byte: end_byte,
                    tail_span_utf8: None,
                    source_token_text: source_text[index..end_byte].to_string(),
                    default_base_node_id: owner_node_id,
                    base: RawOrderedSelectorReferenceBase::Caller,
                    base_span_utf8: None,
                });
            }
            index = end_byte;
            continue;
        }

        if current == '@' {
            requires_prebind = true;
            let end_byte = token_end(source_text, index);
            diagnostics.push(prebind_diagnostic(
                TreeCalcFormulaTextPrebindDiagnosticCode::UnsupportedSelector,
                source_text,
                index,
                end_byte,
                "unsupported TreeCalc selector; admitted selectors are @CHILDREN, @PRECEDING, @FOLLOWING, and @ANCESTORS",
            ));
            index = end_byte;
            continue;
        }

        if source_text[index..].starts_with(".*") && !source_text[index..].starts_with(".**") {
            requires_prebind = true;
            let end_byte = index + ".*".len();
            if previous_char_can_qualify_host_path(source_text, index) {
                let start_byte = host_path_start(source_text, index);
                if !is_token_boundary_after(source_text, end_byte) {
                    let end = token_end(source_text, index);
                    diagnostics.push(prebind_diagnostic(
                        TreeCalcFormulaTextPrebindDiagnosticCode::UnsupportedRawTreeCalcReference,
                        source_text,
                        start_byte,
                        end,
                        "only the exact qualified .* children sugar is admitted in this prebind surface",
                    ));
                } else {
                    children_references.push(RawChildrenReference {
                        start_byte,
                        end_byte,
                        selector_start_byte: index,
                        selector_end_byte: end_byte,
                        source_token_text: source_text[start_byte..end_byte].to_string(),
                        default_base_node_id: owner_node_id,
                        base: RawChildrenReferenceBase::Qualified,
                        base_span_utf8: Some((start_byte, index)),
                    });
                }
            } else if !is_token_boundary_after(source_text, end_byte) {
                let end = token_end(source_text, index);
                diagnostics.push(prebind_diagnostic(
                    TreeCalcFormulaTextPrebindDiagnosticCode::UnsupportedRawTreeCalcReference,
                    source_text,
                    index,
                    end,
                    "only the exact .* children sugar is admitted in this prebind surface",
                ));
            } else {
                children_references.push(RawChildrenReference {
                    start_byte: index,
                    end_byte,
                    selector_start_byte: index,
                    selector_end_byte: end_byte,
                    source_token_text: source_text[index..end_byte].to_string(),
                    default_base_node_id: owner_node_id,
                    base: RawChildrenReferenceBase::Caller,
                    base_span_utf8: None,
                });
            }
            index = end_byte;
            continue;
        }

        if source_text[index..].starts_with("**") {
            requires_prebind = true;
            let end_byte = index + "**".len();
            let (source_end_byte, tail_span_utf8, unsupported_end_byte) =
                recursive_selector_end(source_text, index, end_byte);
            if let Some(unsupported_end_byte) = unsupported_end_byte {
                let start_byte = explicit_qualified_recursive_selector_start(source_text, index)
                    .unwrap_or(index);
                diagnostics.push(prebind_diagnostic(
                    TreeCalcFormulaTextPrebindDiagnosticCode::UnsupportedRawTreeCalcReference,
                    source_text,
                    start_byte,
                    unsupported_end_byte,
                    "only exact recursive selector tokens or .tail paths are admitted in this prebind surface",
                ));
                index = unsupported_end_byte;
                continue;
            }

            if explicit_qualified_recursive_selector_start(source_text, index).is_some() {
                let start_byte = explicit_qualified_recursive_selector_start(source_text, index)
                    .expect("qualified recursive selector start checked above");
                ordered_selector_references.push(RawOrderedSelectorReference {
                    family: TreeCalcOrderedSelectorFamily::RecursiveDescendantsV1,
                    start_byte,
                    end_byte: source_end_byte,
                    selector_start_byte: index,
                    selector_end_byte: end_byte,
                    tail_span_utf8,
                    source_token_text: source_text[start_byte..source_end_byte].to_string(),
                    default_base_node_id: owner_node_id,
                    base: RawOrderedSelectorReferenceBase::Qualified,
                    base_span_utf8: Some((start_byte, index - 1)),
                });
            } else if !previous_char_allows_free_standing_selector(source_text, index) {
                let start_byte = host_path_start(source_text, index);
                diagnostics.push(prebind_diagnostic(
                    TreeCalcFormulaTextPrebindDiagnosticCode::UnsupportedQualifiedHostReference,
                    source_text,
                    start_byte,
                    source_end_byte,
                    "qualified recursive TreeCalc selectors require caller-supplied collection resolution",
                ));
            } else {
                ordered_selector_references.push(RawOrderedSelectorReference {
                    family: TreeCalcOrderedSelectorFamily::RecursiveDescendantsV1,
                    start_byte: index,
                    end_byte: source_end_byte,
                    selector_start_byte: index,
                    selector_end_byte: end_byte,
                    tail_span_utf8,
                    source_token_text: source_text[index..source_end_byte].to_string(),
                    default_base_node_id: owner_node_id,
                    base: RawOrderedSelectorReferenceBase::Caller,
                    base_span_utf8: None,
                });
            }
            index = source_end_byte;
            continue;
        }

        index += current.len_utf8();
    }

    TreeCalcFormulaTextScan {
        children_references,
        ordered_selector_references,
        diagnostics,
        requires_prebind,
    }
}

fn prebind_diagnostic(
    code: TreeCalcFormulaTextPrebindDiagnosticCode,
    source_text: &str,
    start_byte: usize,
    end_byte: usize,
    detail: impl Into<String>,
) -> TreeCalcFormulaTextPrebindDiagnostic {
    TreeCalcFormulaTextPrebindDiagnostic {
        code,
        source_span_utf8: (start_byte, end_byte),
        source_token_text: source_text[start_byte..end_byte].to_string(),
        detail: detail.into(),
    }
}

fn previous_non_whitespace_char(source_text: &str, end_byte: usize) -> Option<char> {
    source_text[..end_byte]
        .chars()
        .rev()
        .find(|ch| !ch.is_whitespace())
}

fn previous_char_can_qualify_host_path(source_text: &str, end_byte: usize) -> bool {
    previous_non_whitespace_char(source_text, end_byte).is_some_and(is_host_path_tail_char)
}

fn explicit_qualified_children_selector_start(
    source_text: &str,
    selector_start: usize,
) -> Option<usize> {
    explicit_qualified_at_selector_start(source_text, selector_start)
}

fn explicit_qualified_at_selector_start(source_text: &str, selector_start: usize) -> Option<usize> {
    let dot_start = selector_start.checked_sub(1)?;
    if !source_text[..selector_start].ends_with('.') {
        return None;
    }
    let start_byte = host_path_start(source_text, selector_start);
    (start_byte < dot_start).then_some(start_byte)
}

fn explicit_qualified_recursive_selector_start(
    source_text: &str,
    selector_start: usize,
) -> Option<usize> {
    explicit_qualified_at_selector_start(source_text, selector_start)
}

fn ordered_at_selector_at(
    source_text: &str,
    start_byte: usize,
) -> Option<(TreeCalcOrderedSelectorFamily, &'static str)> {
    [
        (TreeCalcOrderedSelectorFamily::PrecedingV1, "@PRECEDING"),
        (TreeCalcOrderedSelectorFamily::FollowingV1, "@FOLLOWING"),
        (TreeCalcOrderedSelectorFamily::AncestorsV1, "@ANCESTORS"),
    ]
    .into_iter()
    .find(|(_, token)| source_text[start_byte..].starts_with(token))
}

fn recursive_selector_end(
    source_text: &str,
    selector_start: usize,
    selector_end: usize,
) -> (usize, Option<(usize, usize)>, Option<usize>) {
    let Some(next) = source_text[selector_end..].chars().next() else {
        return (selector_end, None, None);
    };
    if next == '.' {
        let tail_end = token_end(source_text, selector_start);
        let tail_start = selector_end;
        let tail_text = &source_text[tail_start..tail_end];
        if tail_text.len() <= 1 {
            return (tail_end, None, Some(tail_end));
        }
        return (tail_end, Some((tail_start, tail_end)), None);
    }
    if is_token_body_char(next) {
        let unsupported_end = token_end(source_text, selector_start);
        return (unsupported_end, None, Some(unsupported_end));
    }
    (selector_end, None, None)
}

fn previous_char_allows_free_standing_selector(source_text: &str, end_byte: usize) -> bool {
    previous_non_whitespace_char(source_text, end_byte).is_none_or(|ch| {
        matches!(
            ch,
            '=' | '(' | ',' | ';' | '+' | '-' | '*' | '/' | '^' | '&' | '<' | '>' | '{'
        )
    })
}

fn is_host_path_tail_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '_' | ']' | '\'' | '"')
}

fn host_path_start(source_text: &str, selector_start: usize) -> usize {
    source_text[..selector_start]
        .char_indices()
        .rev()
        .find_map(|(index, ch)| (!is_host_path_prefix_char(ch)).then_some(index + ch.len_utf8()))
        .unwrap_or(0)
}

fn is_host_path_prefix_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric()
        || matches!(
            ch,
            '_' | '.' | '[' | ']' | '\'' | '"' | '!' | '^' | '\\' | '/'
        )
}

fn token_end(source_text: &str, start_byte: usize) -> usize {
    source_text[start_byte..]
        .char_indices()
        .find_map(|(offset, ch)| {
            (offset > 0 && !is_token_body_char(ch)).then_some(start_byte + offset)
        })
        .unwrap_or(source_text.len())
}

fn is_token_body_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '_' | '@' | '.' | '*' | '^' | '[' | ']' | '!' | '\'')
}

fn is_token_boundary_after(source_text: &str, end_byte: usize) -> bool {
    source_text[end_byte..]
        .chars()
        .next()
        .is_none_or(|ch| !is_token_body_char(ch))
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
    ActiveBridgeSliceGreen,
    OxCalcImplementedBridgePending,
    CorpusAuthoredRunnerPending,
    RetainedReplayPending,
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
        runnable_suite_command: "cargo test -p dnatreecalc-host active_children_corpus -- --nocapture",
        oxcalc_status: "implemented and green as ChildrenV1, including direct OxCalcTreeContext raw @CHILDREN/.* product path",
        dnatreecalc_status: "active bridge slice; direct OxCalcTreeContext migration pending",
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
        runnable_suite_command: "cargo test -p dnatreecalc-host active_walkup_corpus_executes_relative_references_through_live_oxcalc_bridge -- --nocapture",
        oxcalc_status: "direct OxCalcTreeContext raw formula binding resolves bare walk-up and dotted descent through the host-name bind lane; RelativePath carriers still cover lower-level fixture paths",
        dnatreecalc_status: "active typed-carrier bridge slice; direct OxCalcTreeContext corpus migration pending",
        replay_status: "retained non-table replay missing",
        current_test_result: "green active walk-up runner plus OxCalc direct-context walk-up/dotted test; product closure still waits for DnaTreeCalc direct-context activation",
        evidence_status: W056NonTableReferenceEvidenceStatus::OxCalcImplementedBridgePending,
        specification_is_sufficient_for_cases: true,
        blocks_w056_non_table_closure: true,
    },
    W056NonTableReferenceCategory {
        category_id: "ancestor_root_anchors",
        reference_family: "ancestor and root/current-workspace anchors",
        examples: &["^.Rate", "^^.Year", "[]Sheet1.Margin", "[]"],
        spec_anchor: "DnaTreeCalc CORE_MODEL_SPEC.md §3.1/§3.2/§3.7; references/anchors",
        expected_outcome_contract: "fixed-depth ancestor navigation or workspace-root absolute lookup with #REF!/unresolved diagnostics on invalid use",
        corpus_or_suite: &["../DnaTreeCalc/docs/test-corpus/references/anchors.json"],
        runnable_suite_command: "pwsh ../DnaTreeCalc/docs/test-corpus/tools/validate-corpus.ps1",
        oxcalc_status: "ancestor RelativePath carriers exist; full raw anchor bridge activation pending",
        dnatreecalc_status: "corpus authored, pending",
        replay_status: "retained non-table replay missing",
        current_test_result: "validator green; no active bridge runner",
        evidence_status: W056NonTableReferenceEvidenceStatus::CorpusAuthoredRunnerPending,
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
        runnable_suite_command: "cargo test -p dnatreecalc-host active_cross_workspace_corpus_resolves_through_oxcalc_workspace_packets -- --nocapture",
        oxcalc_status: "provider/alias packet and workspace-qualified carrier implemented",
        dnatreecalc_status: "active typed cross-workspace bridge slice",
        replay_status: "retained non-table replay missing",
        current_test_result: "active runner command available; retained evidence still open",
        evidence_status: W056NonTableReferenceEvidenceStatus::RetainedReplayPending,
        specification_is_sufficient_for_cases: true,
        blocks_w056_non_table_closure: true,
    },
    W056NonTableReferenceCategory {
        category_id: "escaping_canonicalization_case",
        reference_family: "bracket escaping, case-insensitive lookup, canonical display paths",
        examples: &["[Sales Q1].Margin", "[][Sales Q1].Margin", "sales.margin"],
        spec_anchor: "DnaTreeCalc CORE_MODEL_SPEC.md §3.3/§3.4; references/escaping",
        expected_outcome_contract: "escaped segments preserve literal names, lookup is case-insensitive, output reports canonical display paths and ambiguity diagnostics",
        corpus_or_suite: &["../DnaTreeCalc/docs/test-corpus/references/escaping.json"],
        runnable_suite_command: "pwsh ../DnaTreeCalc/docs/test-corpus/tools/validate-corpus.ps1",
        oxcalc_status: "structural path base resolver covers bracketed path segments for admitted selector bases",
        dnatreecalc_status: "corpus authored, pending raw bridge runner",
        replay_status: "retained non-table replay missing",
        current_test_result: "validator green; no active bridge runner",
        evidence_status: W056NonTableReferenceEvidenceStatus::CorpusAuthoredRunnerPending,
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
        expected_outcome_contract: "meta-effective subtrees are invisible to formula resolution and positional operators; accessors return typed metadata or #REF!",
        corpus_or_suite: &["../DnaTreeCalc/docs/test-corpus/references/meta-nodes.json"],
        runnable_suite_command: "pwsh ../DnaTreeCalc/docs/test-corpus/tools/validate-corpus.ps1",
        oxcalc_status: "requires full metadata-aware structural snapshot activation",
        dnatreecalc_status: "corpus authored, pending",
        replay_status: "retained non-table replay missing",
        current_test_result: "validator green; no active bridge runner",
        evidence_status: W056NonTableReferenceEvidenceStatus::CorpusAuthoredRunnerPending,
        specification_is_sufficient_for_cases: true,
        blocks_w056_non_table_closure: true,
    },
    W056NonTableReferenceCategory {
        category_id: "sibling_single_navigation",
        reference_family: "single sibling navigation",
        examples: &["@PREV.Net", "@NEXT.Margin", "ref.@PREV"],
        spec_anchor: "DnaTreeCalc CORE_MODEL_SPEC.md §3.5/§3.5b/§3.7; references/sibling-offsets",
        expected_outcome_contract: "previous/next regular sibling lookup by sibling order with meta nodes skipped and #REF! on out-of-range",
        corpus_or_suite: &["../DnaTreeCalc/docs/test-corpus/references/sibling-offsets.json"],
        runnable_suite_command: "pwsh ../DnaTreeCalc/docs/test-corpus/tools/validate-corpus.ps1",
        oxcalc_status: "SiblingOffset carrier and dependency lowering exist",
        dnatreecalc_status: "corpus authored, pending bridge activation",
        replay_status: "retained non-table replay missing",
        current_test_result: "validator green; no active bridge runner",
        evidence_status: W056NonTableReferenceEvidenceStatus::OxCalcImplementedBridgePending,
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
            "../DnaTreeCalc/docs/test-corpus/references/set-membership.json",
        ],
        runnable_suite_command: "cargo test -p dnatreecalc-host active_ordered_corpus -- --nocapture",
        oxcalc_status: "resolved ordered selector carriers, traversal resolver, bounds, and direct OxCalcTreeContext raw @PRECEDING/@FOLLOWING/@ANCESTORS product path implemented",
        dnatreecalc_status: "active ordered raw bridge slice; direct OxCalcTreeContext migration and broad set-membership corpus pending",
        replay_status: "retained non-table replay missing",
        current_test_result: "green active ordered slice in prior checks plus direct-context ordered selector test; full family pending",
        evidence_status: W056NonTableReferenceEvidenceStatus::ActiveBridgeSliceGreen,
        specification_is_sufficient_for_cases: true,
        blocks_w056_non_table_closure: true,
    },
    W056NonTableReferenceCategory {
        category_id: "recursive_descent",
        reference_family: "recursive descent",
        examples: &["**.Margin", "Base.**.Margin"],
        spec_anchor: "DnaTreeCalc CORE_MODEL_SPEC.md §3.5b/§3.6/§3.7; references/ordered-raw-active",
        expected_outcome_contract: "stable depth-first preorder descendant traversal, optional tail filtering, traversal-bound diagnostics, and membership/order facts",
        corpus_or_suite: &[
            "../DnaTreeCalc/docs/test-corpus/references/ordered-raw-active.json",
            "oxcalc-core formula::ordered_selector_traversal_resolver_projects_structural_membership",
        ],
        runnable_suite_command: "cargo test -p dnatreecalc-host active_ordered_corpus -- --nocapture",
        oxcalc_status: "recursive ordered selector carrier, traversal-bound resolver, and direct OxCalcTreeContext Base.**.tail product path implemented",
        dnatreecalc_status: "active focused recursive bridge slice; direct OxCalcTreeContext migration and broader set-membership corpus pending",
        replay_status: "retained non-table replay missing",
        current_test_result: "green active slice in prior checks plus direct-context recursive selector test; full family pending",
        evidence_status: W056NonTableReferenceEvidenceStatus::ActiveBridgeSliceGreen,
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
        runnable_suite_command: "cargo test -p dnatreecalc-host active_reference_literal_array_corpus_executes_through_live_oxcalc_bridge -- --nocapture",
        oxcalc_status: "ReferenceLiteralArrayV1 implemented; direct OxCalcTreeContext records raw reference-literal collection as a typed pending lane",
        dnatreecalc_status: "active prepared-carrier slice; broad raw literal and array references pending direct context support",
        replay_status: "retained non-table replay missing",
        current_test_result: "active literal slice green in prior checks; broad raw suite pending",
        evidence_status: W056NonTableReferenceEvidenceStatus::ActiveBridgeSliceGreen,
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
        runnable_suite_command: "cargo test -p dnatreecalc-host active_dynamic_indirect_corpus_executes_through_oxcalc_dynamic_carriers -- --nocapture",
        oxcalc_status: "DynamicPotential and DynamicResolved carriers/facts implemented; direct OxCalcTreeContext records raw INDIRECT as a typed pending lane",
        dnatreecalc_status: "active typed dynamic carrier slice; raw INDIRECT parse breadth pending",
        replay_status: "retained non-table replay missing",
        current_test_result: "active dynamic runner passed in the calc-4vs8.64 pass; retained evidence still open",
        evidence_status: W056NonTableReferenceEvidenceStatus::RetainedReplayPending,
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
        runnable_suite_command: "cargo test -p dnatreecalc-host active_cross_workspace_corpus_resolves_through_oxcalc_workspace_packets -- --nocapture",
        oxcalc_status: "workspace-qualified carriers, reverse edges, and prepared identity implemented; direct OxCalcTreeContext records raw cross-workspace host names as typed pending",
        dnatreecalc_status: "active typed cross-workspace slice",
        replay_status: "retained non-table replay missing",
        current_test_result: "active cross-workspace runner passed in the calc-4vs8.64 pass; retained evidence still open",
        evidence_status: W056NonTableReferenceEvidenceStatus::RetainedReplayPending,
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
        dnatreecalc_status: "raw formula host-name direct-context activation pending beyond typed walk-up carriers",
        replay_status: "retained non-table replay missing",
        current_test_result: "OxFml/OxCalc intake green and direct OxCalcTreeContext host-name tests green; DnaTreeCalc raw runner pending",
        evidence_status: W056NonTableReferenceEvidenceStatus::OxCalcImplementedBridgePending,
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
        runnable_suite_command: "pwsh ../DnaTreeCalc/docs/test-corpus/tools/validate-corpus.ps1",
        oxcalc_status: "current mapping admitted by calc-4vs8.32; direct OxCalcTreeContext records callable host names as W074-blocked typed exclusions",
        dnatreecalc_status: "corpus authored; dtc-z0i.8 in progress",
        replay_status: "retained non-table replay missing",
        current_test_result: "validator green; no active lambda-node runner",
        evidence_status: W056NonTableReferenceEvidenceStatus::CorpusAuthoredRunnerPending,
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
        runnable_suite_command: "pwsh ../DnaTreeCalc/docs/test-corpus/tools/validate-corpus.ps1",
        oxcalc_status: "capability-profile identity participates in prepared identity; full parser gate is OxFml/OxCalc integration work",
        dnatreecalc_status: "corpus authored, pending",
        replay_status: "retained non-table replay missing",
        current_test_result: "validator green; no active bridge runner",
        evidence_status: W056NonTableReferenceEvidenceStatus::CorpusAuthoredRunnerPending,
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
            "oxcalc-core structural_invalidation_seeds_mark_relative_reference_rebind_after_rename",
        ],
        runnable_suite_command: "pwsh ../DnaTreeCalc/docs/test-corpus/tools/validate-corpus.ps1",
        oxcalc_status: "structural rebind facts implemented for current carriers",
        dnatreecalc_status: "corpus authored, pending propagation runner",
        replay_status: "retained non-table replay missing",
        current_test_result: "validator green; no active structural-edit corpus runner",
        evidence_status: W056NonTableReferenceEvidenceStatus::CorpusAuthoredRunnerPending,
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
        runnable_suite_command: "cargo test -p dnatreecalc-host active_walkup_corpus_executes_relative_references_through_live_oxcalc_bridge -- --nocapture",
        oxcalc_status: "unresolved descriptors and cycle diagnostics exist",
        dnatreecalc_status: "walk-up active slice covers unresolved/self-reference; broader syntax pending",
        replay_status: "retained non-table replay missing",
        current_test_result: "green active walk-up diagnostics; broad syntax runner pending",
        evidence_status: W056NonTableReferenceEvidenceStatus::ActiveBridgeSliceGreen,
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
pub struct TreeFormulaReferenceCarrier {
    pub source_token: Option<String>,
    pub reference: TreeReference,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host_name_bind: Option<TreeFormulaHostNameBindPacket>,
}

impl TreeFormulaReferenceCarrier {
    #[must_use]
    pub fn named(source_token: impl Into<String>, reference: TreeReference) -> Self {
        Self {
            source_token: Some(source_token.into()),
            reference,
            host_name_bind: None,
        }
    }

    #[must_use]
    pub fn fact(reference: TreeReference) -> Self {
        Self {
            source_token: None,
            reference,
            host_name_bind: None,
        }
    }

    #[must_use]
    pub fn with_host_name_bind(mut self, bind: TreeFormulaHostNameBindPacket) -> Self {
        self.host_name_bind = Some(bind);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreeFormulaHostNameBindPacket {
    pub host_name_handle: String,
    pub canonical_name: String,
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
pub struct TreeFormula {
    pub source_text: String,
    #[serde(default)]
    pub reference_carriers: Vec<TreeFormulaReferenceCarrier>,
    #[serde(default)]
    pub lazy_residual_publication: bool,
}

impl TreeFormula {
    #[must_use]
    pub fn opaque_oxfml(
        source_text: impl Into<String>,
        reference_carriers: impl IntoIterator<Item = TreeFormulaReferenceCarrier>,
    ) -> Self {
        Self {
            source_text: normalize_formula_source(source_text.into()),
            reference_carriers: reference_carriers.into_iter().collect(),
            lazy_residual_publication: false,
        }
    }

    #[must_use]
    pub fn with_lazy_residual_publication(mut self, lazy_residual_publication: bool) -> Self {
        self.lazy_residual_publication = lazy_residual_publication;
        self
    }

    #[must_use]
    pub fn source_text(&self) -> &str {
        &self.source_text
    }

    #[must_use]
    pub fn reference_carriers(&self) -> &[TreeFormulaReferenceCarrier] {
        &self.reference_carriers
    }
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
        reference_carriers: Vec<TreeReference>,
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
            reference_carriers: Vec::new(),
        };
        let source_text = state.render_formula(self);
        let lazy_residual_publication = matches!(
            self,
            FixtureFormulaAst::FunctionCall { function_name, .. }
                if function_name.eq_ignore_ascii_case("IF")
        );
        TreeFormula::opaque_oxfml(source_text, state.reference_carriers)
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
                    .reference_carriers()
                    .iter()
                    .enumerate()
                    .flat_map(|(index, carrier)| {
                        lower_reference(snapshot, binding, carrier, index)
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
    carrier: &TreeFormulaReferenceCarrier,
    index: usize,
) -> Vec<DependencyDescriptor> {
    let descriptor_id = format!("bind:{}:ref:{index}", binding.formula_artifact_id.0);
    let reference = &carrier.reference;
    if let TreeReference::ReferenceCollection(collection) = reference {
        return lower_reference_collection(snapshot, binding, collection, descriptor_id);
    }

    let kind = reference.descriptor_kind();
    let target_node_id = reference.resolve_target(snapshot, binding.owner_node_id);
    let workspace_target = reference.workspace_target();
    let carrier_detail = reference.carrier_detail();
    let requires_rebind_on_structural_change = reference.requires_rebind_on_structural_change();

    vec![DependencyDescriptor {
        descriptor_id,
        source_reference_handle: carrier
            .source_token
            .as_ref()
            .map(|token| format!("oxcalc_source_token:{token}")),
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
    reference_carriers: Vec<TreeFormulaReferenceCarrier>,
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
                reference_carriers,
            } => {
                for reference in reference_carriers {
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
        self.reference_carriers
            .push(TreeFormulaReferenceCarrier::named(
                token.clone(),
                reference.clone(),
            ));
        token
    }

    fn record_unresolved_reference(&mut self, reference: &TreeReference) -> String {
        let token = format!(
            "TREE_UNRESOLVED_{}_{}",
            self.owner_node_id.0, self.next_reference_index
        );
        self.next_reference_index += 1;
        self.reference_carriers
            .push(TreeFormulaReferenceCarrier::named(
                token.clone(),
                reference.clone(),
            ));
        token
    }

    fn record_fact(&mut self, reference: &TreeReference) {
        self.reference_carriers
            .push(TreeFormulaReferenceCarrier::fact(reference.clone()));
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
            TreeReference::SiblingOffset { .. } => TreeReferenceInventoryVariant::SiblingOffset,
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
            TreeReference::HostSensitive { .. }
            | TreeReference::CrossWorkspaceResolved { .. }
            | TreeReference::CapabilitySensitive { .. }
            | TreeReference::ShapeTopology { .. }
            | TreeReference::Unresolved { .. } => None,
            TreeReference::DynamicPotential { .. } => None,
            TreeReference::DynamicResolved { target_node_id, .. } => Some(*target_node_id),
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
            TreeReference::RelativePath { .. } | TreeReference::SiblingOffset { .. } => {
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
                    formula_artifact_id: None,
                    bind_artifact_id: None,
                    constant_value: None,
                },
                StructuralNode {
                    node_id: TreeNodeId(2),
                    kind: StructuralNodeKind::Container,
                    symbol: "Branch".to_string(),
                    parent_id: Some(TreeNodeId(1)),
                    child_ids: vec![TreeNodeId(4), TreeNodeId(5)],
                    formula_artifact_id: None,
                    bind_artifact_id: None,
                    constant_value: None,
                },
                StructuralNode {
                    node_id: TreeNodeId(3),
                    kind: StructuralNodeKind::Calculation,
                    symbol: "Sibling".to_string(),
                    parent_id: Some(TreeNodeId(1)),
                    child_ids: vec![],
                    formula_artifact_id: None,
                    bind_artifact_id: None,
                    constant_value: None,
                },
                StructuralNode {
                    node_id: TreeNodeId(4),
                    kind: StructuralNodeKind::Calculation,
                    symbol: "Leaf".to_string(),
                    parent_id: Some(TreeNodeId(2)),
                    child_ids: vec![],
                    formula_artifact_id: None,
                    bind_artifact_id: None,
                    constant_value: None,
                },
                StructuralNode {
                    node_id: TreeNodeId(5),
                    kind: StructuralNodeKind::Calculation,
                    symbol: "Neighbor".to_string(),
                    parent_id: Some(TreeNodeId(2)),
                    child_ids: vec![],
                    formula_artifact_id: None,
                    bind_artifact_id: None,
                    constant_value: None,
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
                    formula_artifact_id: None,
                    bind_artifact_id: None,
                    constant_value: None,
                },
                StructuralNode {
                    node_id: TreeNodeId(101),
                    kind: StructuralNodeKind::Container,
                    symbol: "Branch1".to_string(),
                    parent_id: Some(TreeNodeId(100)),
                    child_ids: vec![TreeNodeId(102)],
                    formula_artifact_id: None,
                    bind_artifact_id: None,
                    constant_value: None,
                },
                StructuralNode {
                    node_id: TreeNodeId(102),
                    kind: StructuralNodeKind::Calculation,
                    symbol: "MyNode".to_string(),
                    parent_id: Some(TreeNodeId(101)),
                    child_ids: vec![],
                    formula_artifact_id: None,
                    bind_artifact_id: None,
                    constant_value: None,
                },
                StructuralNode {
                    node_id: TreeNodeId(103),
                    kind: StructuralNodeKind::Container,
                    symbol: "Branch X".to_string(),
                    parent_id: Some(TreeNodeId(100)),
                    child_ids: vec![TreeNodeId(104)],
                    formula_artifact_id: None,
                    bind_artifact_id: None,
                    constant_value: None,
                },
                StructuralNode {
                    node_id: TreeNodeId(104),
                    kind: StructuralNodeKind::Calculation,
                    symbol: "MyNode".to_string(),
                    parent_id: Some(TreeNodeId(103)),
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
    fn raw_treecalc_formula_text_prebinds_children_selector_to_neutral_source() {
        let formula =
            prebind_treecalc_formula_text(TreeNodeId(2), "=SUM(@CHILDREN)").expect("prebind");

        assert_eq!(formula.source_text(), "=SUM(TREE_REF_2_0)");
        assert_eq!(formula.reference_carriers().len(), 1);
        let carrier = &formula.reference_carriers()[0];
        assert_eq!(carrier.source_token.as_deref(), Some("TREE_REF_2_0"));
        match &carrier.reference {
            TreeReference::ReferenceCollection(TreeCalcReferenceCollection::ChildrenV1(
                collection,
            )) => {
                assert_eq!(collection.base_node_id, TreeNodeId(2));
                assert_eq!(collection.source_token_text, "@CHILDREN");
                assert_eq!(collection.source_span_utf8, Some((5, 14)));
                assert_eq!(
                    collection.host_ref_handle,
                    "treecalc-hostref:v1:children:node:2"
                );
            }
            other => panic!("expected ChildrenV1 reference carrier, got {other:?}"),
        }
    }

    #[test]
    fn raw_treecalc_formula_text_prebinds_children_sugar_to_neutral_source() {
        let formula = prebind_treecalc_formula_text(TreeNodeId(2), "=SUM(.*)").expect("prebind");

        assert_eq!(formula.source_text(), "=SUM(TREE_REF_2_0)");
        match &formula.reference_carriers()[0].reference {
            TreeReference::ReferenceCollection(TreeCalcReferenceCollection::ChildrenV1(
                collection,
            )) => {
                assert_eq!(collection.source_token_text, ".*");
                assert_eq!(collection.source_span_utf8, Some((5, 7)));
            }
            other => panic!("expected ChildrenV1 reference carrier, got {other:?}"),
        }
    }

    #[test]
    fn raw_treecalc_formula_text_prebinds_qualified_children_with_resolved_base() {
        let formula = prebind_treecalc_formula_text_with_resolved_bases(
            TreeNodeId(10),
            "=SUM(base.@CHILDREN)",
            [qualified_children_base_resolution(
                "=SUM(base.@CHILDREN)",
                "base.@CHILDREN",
                "@CHILDREN",
                TreeNodeId(2),
            )],
        )
        .expect("resolved qualified prebind");

        assert_eq!(formula.source_text(), "=SUM(TREE_REF_10_0)");
        match &formula.reference_carriers()[0].reference {
            TreeReference::ReferenceCollection(TreeCalcReferenceCollection::ChildrenV1(
                collection,
            )) => {
                assert_eq!(collection.base_node_id, TreeNodeId(2));
                assert_eq!(collection.source_token_text, "base.@CHILDREN");
                assert_eq!(collection.source_span_utf8, Some((5, 19)));
                assert_eq!(
                    collection.host_ref_handle,
                    "treecalc-hostref:v1:children:node:2"
                );
            }
            other => panic!("expected ChildrenV1 reference carrier, got {other:?}"),
        }
    }

    #[test]
    fn raw_treecalc_formula_text_prebinds_qualified_children_sugar_with_resolved_base() {
        let formula = prebind_treecalc_formula_text_with_resolved_bases(
            TreeNodeId(10),
            "=SUM(base.*)",
            [qualified_children_base_resolution(
                "=SUM(base.*)",
                "base.*",
                ".*",
                TreeNodeId(2),
            )],
        )
        .expect("resolved qualified sugar prebind");

        assert_eq!(formula.source_text(), "=SUM(TREE_REF_10_0)");
        match &formula.reference_carriers()[0].reference {
            TreeReference::ReferenceCollection(TreeCalcReferenceCollection::ChildrenV1(
                collection,
            )) => {
                assert_eq!(collection.base_node_id, TreeNodeId(2));
                assert_eq!(collection.source_token_text, "base.*");
                assert_eq!(collection.source_span_utf8, Some((5, 11)));
            }
            other => panic!("expected ChildrenV1 reference carrier, got {other:?}"),
        }
    }

    #[test]
    fn raw_treecalc_formula_text_exposes_public_qualified_children_base_queries() {
        let source_text = "=SUM(left.@CHILDREN,right.*)";
        let queries =
            treecalc_formula_text_qualified_children_base_queries(TreeNodeId(10), source_text);

        assert_eq!(queries.len(), 2);
        assert_eq!(queries[0].source_span_utf8, (5, 19));
        assert_eq!(queries[0].base_span_utf8, (5, 9));
        assert_eq!(queries[0].selector_span_utf8, (10, 19));
        assert_eq!(queries[0].source_token_text, "left.@CHILDREN");
        assert_eq!(queries[0].base_token_text, "left");
        assert_eq!(queries[0].selector_token_text, "@CHILDREN");
        assert_eq!(queries[1].source_span_utf8, (20, 27));
        assert_eq!(queries[1].base_span_utf8, (20, 25));
        assert_eq!(queries[1].selector_span_utf8, (25, 27));
        assert_eq!(queries[1].source_token_text, "right.*");
        assert_eq!(queries[1].base_token_text, "right");
        assert_eq!(queries[1].selector_token_text, ".*");

        let formula = prebind_treecalc_formula_text_with_resolved_bases(
            TreeNodeId(10),
            source_text,
            [
                queries[0].to_resolution(TreeNodeId(2)),
                queries[1].to_resolution(TreeNodeId(3)),
            ],
        )
        .expect("query-derived base resolutions prebind");

        let base_ids = formula
            .reference_carriers()
            .iter()
            .map(|carrier| match &carrier.reference {
                TreeReference::ReferenceCollection(TreeCalcReferenceCollection::ChildrenV1(
                    collection,
                )) => collection.base_node_id,
                other => panic!("expected ChildrenV1 reference carrier, got {other:?}"),
            })
            .collect::<Vec<_>>();
        assert_eq!(base_ids, vec![TreeNodeId(2), TreeNodeId(3)]);
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
            expression: TreeFormula::opaque_oxfml(
                "=TREE_REF_4_0",
                [TreeFormulaReferenceCarrier::named(
                    "TREE_REF_4_0",
                    reference,
                )],
            ),
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
    fn qualified_children_query_can_resolve_base_from_structural_path() {
        let source_text = "=SUM(Branch.@CHILDREN)";
        let query =
            treecalc_formula_text_qualified_children_base_queries(TreeNodeId(10), source_text)
                .into_iter()
                .next()
                .expect("qualified children query");
        let resolution = query
            .to_resolution_with_structural_path_base(&snapshot())
            .expect("structural path base resolution");

        assert_eq!(resolution.base_node_id, TreeNodeId(2));
        assert_eq!(
            resolution.resolution_layer,
            TreeCalcQualifiedBaseResolutionLayer::OxCalcStructuralPath
        );
        assert!(
            resolution
                .resolution_identity
                .contains("treecalc-explicit-host-path:v1")
        );
        assert!(
            resolution
                .resolution_identity
                .contains("canonical=Root/Branch")
        );
    }

    #[test]
    fn raw_treecalc_formula_text_qualified_children_queries_ignore_string_literals() {
        let queries = treecalc_formula_text_qualified_children_base_queries(
            TreeNodeId(10),
            "=CONCAT(\"left.@CHILDREN\",\"right.*\")",
        );

        assert!(queries.is_empty());
    }

    #[test]
    fn raw_treecalc_formula_text_exposes_public_ordered_selector_queries() {
        let source_text =
            "=SUM(@PRECEDING,base.@FOLLOWING,@ANCESTORS,Q2.**,Accounts.2005.**.Margin)";
        let queries = treecalc_formula_text_ordered_selector_queries(TreeNodeId(10), source_text);

        assert_eq!(queries.len(), 5);
        assert_eq!(
            queries[0].family,
            TreeCalcOrderedSelectorFamily::PrecedingV1
        );
        assert_eq!(queries[0].source_token_text, "@PRECEDING");
        assert_eq!(queries[0].base_token_text, None);
        assert_eq!(queries[0].selector_token_text, "@PRECEDING");
        assert_eq!(queries[0].tail_token_text, None);

        assert_eq!(
            queries[1].family,
            TreeCalcOrderedSelectorFamily::FollowingV1
        );
        assert_eq!(queries[1].source_token_text, "base.@FOLLOWING");
        assert_eq!(queries[1].base_token_text.as_deref(), Some("base"));
        assert_eq!(queries[1].selector_token_text, "@FOLLOWING");

        assert_eq!(
            queries[2].family,
            TreeCalcOrderedSelectorFamily::AncestorsV1
        );
        assert_eq!(queries[2].source_token_text, "@ANCESTORS");

        assert_eq!(
            queries[3].family,
            TreeCalcOrderedSelectorFamily::RecursiveDescendantsV1
        );
        assert_eq!(queries[3].source_token_text, "Q2.**");
        assert_eq!(queries[3].base_token_text.as_deref(), Some("Q2"));
        assert_eq!(queries[3].selector_token_text, "**");
        assert_eq!(queries[3].tail_token_text, None);

        assert_eq!(
            queries[4].family,
            TreeCalcOrderedSelectorFamily::RecursiveDescendantsV1
        );
        assert_eq!(queries[4].source_token_text, "Accounts.2005.**.Margin");
        assert_eq!(queries[4].base_token_text.as_deref(), Some("Accounts.2005"));
        assert_eq!(queries[4].selector_token_text, "**");
        assert_eq!(queries[4].tail_token_text.as_deref(), Some(".Margin"));
    }

    #[test]
    fn raw_treecalc_formula_text_prebinds_ordered_selectors_with_resolved_collections() {
        let source_text = "=SUM(@PRECEDING,base.@FOLLOWING,Q2.**.Margin)";
        let queries = treecalc_formula_text_ordered_selector_queries(TreeNodeId(10), source_text);
        let formula = prebind_treecalc_formula_text_with_resolved_ordered_selectors(
            TreeNodeId(10),
            source_text,
            [
                queries[0].to_resolution(TreeNodeId(10), [TreeNodeId(7), TreeNodeId(8)]),
                queries[1].to_resolution(TreeNodeId(2), [TreeNodeId(11)]),
                queries[2].to_resolution(TreeNodeId(3), [TreeNodeId(21), TreeNodeId(22)]),
            ],
        )
        .expect("resolved ordered selector prebind");

        assert_eq!(
            formula.source_text(),
            "=SUM(TREE_REF_10_0,TREE_REF_10_1,TREE_REF_10_2)"
        );
        let collections = formula
            .reference_carriers()
            .iter()
            .map(|carrier| match &carrier.reference {
                TreeReference::ReferenceCollection(
                    TreeCalcReferenceCollection::OrderedSelectorV1(collection),
                ) => collection,
                other => panic!("expected OrderedSelectorV1 reference carrier, got {other:?}"),
            })
            .collect::<Vec<_>>();
        assert_eq!(
            collections
                .iter()
                .map(|collection| collection.family)
                .collect::<Vec<_>>(),
            vec![
                TreeCalcOrderedSelectorFamily::PrecedingV1,
                TreeCalcOrderedSelectorFamily::FollowingV1,
                TreeCalcOrderedSelectorFamily::RecursiveDescendantsV1,
            ]
        );
        assert_eq!(collections[0].base_node_id, TreeNodeId(10));
        assert_eq!(
            collections[0].member_node_ids,
            vec![TreeNodeId(7), TreeNodeId(8)]
        );
        assert_eq!(collections[0].source_token_text, "@PRECEDING");
        assert_eq!(
            collections[0].source_span_utf8,
            Some(queries[0].source_span_utf8)
        );
        assert_eq!(collections[1].base_node_id, TreeNodeId(2));
        assert_eq!(collections[1].source_token_text, "base.@FOLLOWING");
        assert_eq!(collections[2].base_node_id, TreeNodeId(3));
        assert_eq!(collections[2].source_token_text, "Q2.**.Margin");
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
            vec![TreeNodeId(2), TreeNodeId(1)]
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
    fn ordered_selector_query_can_build_structural_traversal_resolution_with_tail() {
        let snapshot = snapshot();
        let source_text = "=SUM(Root.**.Leaf)";
        let query = treecalc_formula_text_ordered_selector_queries(TreeNodeId(10), source_text)
            .into_iter()
            .next()
            .expect("recursive query");

        let resolved = query
            .to_resolution_with_structural_traversal(
                &snapshot,
                TreeNodeId(1),
                TreeCalcOrderedSelectorTraversalPolicy::default(),
            )
            .expect("structural traversal resolution");

        assert_eq!(
            resolved.resolution.resolution_layer,
            TreeCalcOrderedSelectorResolutionLayer::OxCalcStructuralTraversal
        );
        assert_eq!(resolved.resolution.member_node_ids, vec![TreeNodeId(4)]);
        assert_eq!(resolved.traversal.member_node_ids, vec![TreeNodeId(4)]);
        assert!(resolved.traversal.diagnostics.is_empty());
        assert!(
            resolved
                .resolution
                .resolution_identity
                .contains("resolver=oxcalc_structural_traversal")
        );
        assert!(
            resolved
                .resolution
                .resolution_identity
                .contains("max_recursive_descendants=10000")
        );

        let direct_tail_match = query
            .to_resolution_with_structural_traversal(
                &snapshot,
                TreeNodeId(2),
                TreeCalcOrderedSelectorTraversalPolicy::default(),
            )
            .expect("recursive tail can match a direct child of the base");
        assert_eq!(
            direct_tail_match.resolution.member_node_ids,
            vec![TreeNodeId(4)]
        );
    }

    #[test]
    fn ordered_selector_query_can_resolve_base_path_and_traversal_membership() {
        let source_text = "=SUM(Branch.@FOLLOWING,Root.**.Leaf)";
        let queries = treecalc_formula_text_ordered_selector_queries(TreeNodeId(10), source_text);

        let following = queries[0]
            .to_resolution_with_structural_path_base_and_traversal(
                &snapshot(),
                TreeCalcOrderedSelectorTraversalPolicy::default(),
            )
            .expect("structural path and following traversal");
        assert_eq!(following.resolution.base_node_id, TreeNodeId(2));
        assert_eq!(following.resolution.member_node_ids, vec![TreeNodeId(3)]);
        assert_eq!(
            following.resolution.resolution_layer,
            TreeCalcOrderedSelectorResolutionLayer::OxCalcStructuralTraversal
        );
        assert!(
            following
                .resolution
                .resolution_identity
                .contains("base_resolution=treecalc-explicit-host-path:v1")
        );
        assert!(
            following
                .resolution
                .resolution_identity
                .contains("canonical=Root/Branch")
        );

        let recursive = queries[1]
            .to_resolution_with_structural_path_base_and_traversal(
                &snapshot(),
                TreeCalcOrderedSelectorTraversalPolicy::default(),
            )
            .expect("structural path and recursive traversal");
        assert_eq!(recursive.resolution.base_node_id, TreeNodeId(1));
        assert_eq!(recursive.resolution.member_node_ids, vec![TreeNodeId(4)]);
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
    fn raw_treecalc_formula_text_ordered_selector_queries_ignore_string_literals() {
        let queries = treecalc_formula_text_ordered_selector_queries(
            TreeNodeId(10),
            "=CONCAT(\"@PRECEDING\",\"Q2.**\")",
        );

        assert!(queries.is_empty());
    }

    #[test]
    fn raw_treecalc_formula_text_prebinds_distinct_qualified_children_by_span() {
        let source_text = "=SUM(left.@CHILDREN,right.*)";
        let formula = prebind_treecalc_formula_text_with_resolved_bases(
            TreeNodeId(10),
            source_text,
            [
                qualified_children_base_resolution(
                    source_text,
                    "left.@CHILDREN",
                    "@CHILDREN",
                    TreeNodeId(2),
                ),
                qualified_children_base_resolution(source_text, "right.*", ".*", TreeNodeId(3)),
            ],
        )
        .expect("resolved qualified prebind");

        assert_eq!(formula.source_text(), "=SUM(TREE_REF_10_0,TREE_REF_10_1)");
        let base_ids = formula
            .reference_carriers()
            .iter()
            .map(|carrier| match &carrier.reference {
                TreeReference::ReferenceCollection(TreeCalcReferenceCollection::ChildrenV1(
                    collection,
                )) => collection.base_node_id,
                other => panic!("expected ChildrenV1 reference carrier, got {other:?}"),
            })
            .collect::<Vec<_>>();
        assert_eq!(base_ids, vec![TreeNodeId(2), TreeNodeId(3)]);
    }

    #[test]
    fn raw_treecalc_formula_text_rejects_suffixed_qualified_children_selectors() {
        for (source_text, source_token_text, selector_text) in [
            ("=SUM(base.@CHILDRENfoo)", "base.@CHILDREN", "@CHILDREN"),
            ("=SUM(base.@CHILDREN_X)", "base.@CHILDREN", "@CHILDREN"),
            ("=SUM(base.*foo)", "base.*", ".*"),
        ] {
            let error = prebind_treecalc_formula_text_with_resolved_bases(
                TreeNodeId(10),
                source_text,
                [qualified_children_base_resolution(
                    source_text,
                    source_token_text,
                    selector_text,
                    TreeNodeId(2),
                )],
            )
            .expect_err("suffixed qualified selector should be rejected");
            assert_eq!(
                error.diagnostics[0].code,
                TreeCalcFormulaTextPrebindDiagnosticCode::UnsupportedRawTreeCalcReference
            );
        }
    }

    #[test]
    fn raw_treecalc_formula_text_rejects_unsupported_reference_families() {
        let error = prebind_treecalc_formula_text(TreeNodeId(2), "=SUM(@UNKNOWN)")
            .expect_err("unsupported selector should be diagnosed");

        assert_eq!(error.diagnostics.len(), 1);
        assert_eq!(
            error.diagnostics[0].code,
            TreeCalcFormulaTextPrebindDiagnosticCode::UnsupportedSelector
        );
        assert_eq!(error.diagnostics[0].source_span_utf8, (5, 13));
        assert_eq!(error.diagnostics[0].source_token_text, "@UNKNOWN");

        let qualified = prebind_treecalc_formula_text(TreeNodeId(2), "=SUM(base.@CHILDREN)")
            .expect_err("qualified children syntax should wait for typed path resolution");
        assert_eq!(
            qualified.diagnostics[0].code,
            TreeCalcFormulaTextPrebindDiagnosticCode::UnsupportedQualifiedHostReference
        );
        assert_eq!(qualified.diagnostics[0].source_token_text, "base.@CHILDREN");

        let adjacent = prebind_treecalc_formula_text(TreeNodeId(2), "=SUM(Node@CHILDREN)")
            .expect_err("adjacent path-like selector should be diagnosed");
        assert_eq!(
            adjacent.diagnostics[0].code,
            TreeCalcFormulaTextPrebindDiagnosticCode::UnsupportedQualifiedHostReference
        );
        assert_eq!(adjacent.diagnostics[0].source_token_text, "Node@CHILDREN");
    }

    #[test]
    fn raw_treecalc_formula_text_rejects_unresolved_ordered_selectors() {
        let error = prebind_treecalc_formula_text(TreeNodeId(2), "=SUM(@ANCESTORS,Q2.**)")
            .expect_err("ordered selectors require resolved collection packets");

        assert_eq!(error.diagnostics.len(), 2);
        assert_eq!(
            error.diagnostics[0].code,
            TreeCalcFormulaTextPrebindDiagnosticCode::MissingOrderedSelectorResolution
        );
        assert_eq!(error.diagnostics[0].source_token_text, "@ANCESTORS");
        assert_eq!(
            error.diagnostics[1].code,
            TreeCalcFormulaTextPrebindDiagnosticCode::MissingOrderedSelectorResolution
        );
        assert_eq!(error.diagnostics[1].source_token_text, "Q2.**");
    }

    #[test]
    fn raw_treecalc_formula_text_ignores_host_tokens_inside_strings() {
        let formula =
            prebind_treecalc_formula_text(TreeNodeId(2), "=CONCAT(\"@CHILDREN\", \".*\")")
                .expect("string literals should not be prebound");

        assert_eq!(formula.source_text(), "=CONCAT(\"@CHILDREN\", \".*\")");
        assert!(formula.reference_carriers().is_empty());
        assert!(!treecalc_formula_text_needs_prebind("=\"@CHILDREN\""));
    }

    fn qualified_children_base_resolution(
        source_text: &str,
        source_token_text: &str,
        selector_text: &str,
        base_node_id: TreeNodeId,
    ) -> TreeCalcQualifiedChildrenBaseResolution {
        let source_start = source_text.find(source_token_text).expect("source token");
        let source_end = source_start + source_token_text.len();
        let selector_start = source_text[source_start..source_end]
            .find(selector_text)
            .map(|offset| source_start + offset)
            .expect("selector token");
        let selector_end = selector_start + selector_text.len();
        let base_end = if selector_text.starts_with('.') {
            selector_start
        } else {
            selector_start - 1
        };
        TreeCalcQualifiedChildrenBaseResolution::new(
            (source_start, source_end),
            (source_start, base_end),
            (selector_start, selector_end),
            source_token_text,
            base_node_id,
        )
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
            W056NonTableReferenceEvidenceStatus::ActiveBridgeSliceGreen,
            W056NonTableReferenceEvidenceStatus::OxCalcImplementedBridgePending,
            W056NonTableReferenceEvidenceStatus::CorpusAuthoredRunnerPending,
            W056NonTableReferenceEvidenceStatus::RetainedReplayPending,
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
                [TreeFormulaReferenceCarrier::named(
                    "@CHILDREN",
                    TreeReference::ReferenceCollection(TreeCalcReferenceCollection::ChildrenV1(
                        collection.clone(),
                    )),
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
                [TreeFormulaReferenceCarrier::named(
                    "@PRECEDING",
                    TreeReference::ReferenceCollection(
                        TreeCalcReferenceCollection::OrderedSelectorV1(collection.clone()),
                    ),
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
                [TreeFormulaReferenceCarrier::named(
                    "{A,C,A}",
                    TreeReference::ReferenceCollection(
                        TreeCalcReferenceCollection::ReferenceLiteralArrayV1(collection),
                    ),
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
    fn runtime_fact_carriers_are_not_formula_reference_carriers() {
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

            let carrier = TreeFormulaReferenceCarrier::fact(reference);
            assert_eq!(carrier.source_token, None);
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
                ]
                .map(TreeFormulaReferenceCarrier::fact),
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
                .all(|descriptor| descriptor.source_reference_handle.is_none())
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
    fn raw_oxfml_formula_lowers_declared_reference_carriers() {
        let snapshot = snapshot();
        let catalog = TreeFormulaCatalog::new([TreeFormulaBinding {
            owner_node_id: TreeNodeId(4),
            formula_artifact_id: FormulaArtifactId("formula:raw-let-lambda".to_string()),
            bind_artifact_id: Some(BindArtifactId("bind:raw-let-lambda".to_string())),
            expression: fixture_formula(
                TreeNodeId(4),
                FixtureFormulaAst::RawOxfml {
                    source_text: "LET(base,TREE_REF_4_0,LAMBDA(delta,base+delta)(5))".to_string(),
                    reference_carriers: vec![
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
