#![forbid(unsafe_code)]

//! TreeCalc-local formula and reference substrate.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::dependency::{
    DependencyDescriptor, DependencyDescriptorKind, InvalidationReasonKind,
    TreeReferenceCollectionDependency, TreeReferenceCollectionFamily,
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
    NeedsSelectorDependencyModel,
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
            status: Status::TypedExclusion,
            blocker: Some(Blocker::NeedsCrossWorkspaceModel),
            carrier_class: Some(TreeReferenceCarrierClass::FormulaReference),
            host_reference_correlation: Correlation::HostReferenceHandle,
            namespace_identity_need: Namespace::CrossWorkspaceAvailabilityVersion,
            caller_context_identity_need: Caller::None,
            dependency_facts: vec![Dep::Unresolved],
            invalidation_facts: vec![
                Invalidates::StructuralRebindRequired,
                Invalidates::ExternallyInvalidated,
            ],
            successor_bead: Some("calc-4vs8.3"),
            evidence_note: "cross-workspace availability and degradation need a versioned workspace model before execution",
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
            status: Status::TypedExclusion,
            blocker: Some(Blocker::NeedsOxFmlNameCallPrecedenceEvidence),
            carrier_class: Some(TreeReferenceCarrierClass::FormulaReference),
            host_reference_correlation: Correlation::SourceTokenToFormalReference,
            namespace_identity_need: Namespace::HostNamespaceVersion,
            caller_context_identity_need: Caller::CallerNode,
            dependency_facts: vec![Dep::Unresolved],
            invalidation_facts: vec![Invalidates::StructuralRebindRequired],
            successor_bead: None,
            evidence_note: "bare names and callables stay gated on OxFml W074-CALC005 precedence evidence",
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreeFormulaReferenceCarrier {
    pub source_token: Option<String>,
    pub reference: TreeReference,
}

impl TreeFormulaReferenceCarrier {
    #[must_use]
    pub fn named(source_token: impl Into<String>, reference: TreeReference) -> Self {
        Self {
            source_token: Some(source_token.into()),
            reference,
        }
    }

    #[must_use]
    pub fn fact(reference: TreeReference) -> Self {
        Self {
            source_token: None,
            reference,
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
            | TreeReference::CapabilitySensitive { .. }
            | TreeReference::ShapeTopology { .. }
            | TreeReference::Unresolved { .. } => None,
            TreeReference::DynamicPotential { .. } => None,
            TreeReference::DynamicResolved { target_node_id, .. } => Some(*target_node_id),
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

        let bare_name = tree_reference_implementation_input(
            TreeReferenceInventoryVariant::BareNameOrCallableReference,
        )
        .expect("bare name inventory");
        assert_eq!(
            bare_name.blocker,
            Some(TreeReferenceInventoryBlocker::NeedsOxFmlNameCallPrecedenceEvidence)
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
