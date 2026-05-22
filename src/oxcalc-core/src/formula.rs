#![forbid(unsafe_code)]

//! TreeCalc-local formula and reference substrate.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::dependency::{
    DependencyDescriptor, DependencyDescriptorKind, InvalidationReasonKind,
    TreeReferenceCollectionDependency,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TreeCalcFormulaTextPrebindDiagnosticCode {
    UnsupportedSelector,
    UnsupportedQualifiedHostReference,
    UnsupportedRawTreeCalcReference,
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

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreeCalcFormulaTextPrebindContext {
    #[serde(default)]
    pub qualified_children_bases: Vec<TreeCalcQualifiedChildrenBaseResolution>,
}

impl TreeCalcFormulaTextPrebindContext {
    #[must_use]
    pub fn with_qualified_children_bases(
        qualified_children_bases: impl IntoIterator<Item = TreeCalcQualifiedChildrenBaseResolution>,
    ) -> Self {
        Self {
            qualified_children_bases: qualified_children_bases.into_iter().collect(),
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
    if !scan.diagnostics.is_empty() {
        return Err(TreeCalcFormulaTextPrebindError {
            diagnostics: scan.diagnostics,
        });
    }

    if scan.children_references.is_empty() {
        return Ok(TreeFormula::opaque_oxfml(source_text, Vec::new()));
    }

    let mut rewritten = String::with_capacity(source_text.len());
    let mut carriers = Vec::with_capacity(scan.children_references.len());
    let mut cursor = 0;
    for (reference_index, reference) in scan.children_references.into_iter().enumerate() {
        let neutral_token = format!("TREE_REF_{}_{}", owner_node_id.0, reference_index);
        rewritten.push_str(&source_text[cursor..reference.start_byte]);
        rewritten.push_str(&neutral_token);
        cursor = reference.end_byte;

        let collection = TreeCalcChildrenReferenceCollection::new(
            context
                .resolved_base_for(&reference)
                .expect("qualified references were checked before rewrite"),
            reference.source_token_text,
        )
        .with_source_span_utf8(reference.start_byte, reference.end_byte);
        carriers.push(TreeFormulaReferenceCarrier::named(
            neutral_token,
            TreeReference::ReferenceCollection(TreeCalcReferenceCollection::ChildrenV1(collection)),
        ));
    }
    rewritten.push_str(&source_text[cursor..]);

    Ok(TreeFormula::opaque_oxfml(rewritten, carriers))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TreeCalcFormulaTextScan {
    children_references: Vec<RawChildrenReference>,
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

fn scan_treecalc_formula_text(
    source_text: &str,
    owner_node_id: TreeNodeId,
) -> TreeCalcFormulaTextScan {
    let mut children_references = Vec::new();
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

        if current == '@' {
            requires_prebind = true;
            let end_byte = token_end(source_text, index);
            diagnostics.push(prebind_diagnostic(
                TreeCalcFormulaTextPrebindDiagnosticCode::UnsupportedSelector,
                source_text,
                index,
                end_byte,
                "unsupported TreeCalc selector; admitted first-scope selector is @CHILDREN",
            ));
            index = end_byte;
            continue;
        }

        if source_text[index..].starts_with(".*") {
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
            diagnostics.push(prebind_diagnostic(
                TreeCalcFormulaTextPrebindDiagnosticCode::UnsupportedSelector,
                source_text,
                index,
                end_byte,
                "recursive TreeCalc selectors are outside the first prebind surface",
            ));
            index = end_byte;
            continue;
        }

        index += current.len_utf8();
    }

    TreeCalcFormulaTextScan {
        children_references,
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
    let dot_start = selector_start.checked_sub(1)?;
    if !source_text[..selector_start].ends_with('.') {
        return None;
    }
    let start_byte = host_path_start(source_text, selector_start);
    (start_byte < dot_start).then_some(start_byte)
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
            successor_bead: Some("calc-4vs8.3"),
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
            successor_bead: Some("calc-4vs8.3"),
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
            successor_bead: Some("calc-4vs8.3"),
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
            successor_bead: Some("calc-4vs8.3"),
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
            blocker: Some(Blocker::NeedsSelectorDependencyModel),
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
            evidence_note: "sibling set selectors follow the ChildrenV1 collection pattern after dependency widening",
        },
        TreeReferenceImplementationInput {
            variant: Variant::PrecedingFollowingSelector,
            status: Status::AdmittedImplementationInput,
            blocker: Some(Blocker::NeedsSelectorDependencyModel),
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
            evidence_note: "preceding/following selectors need ordered-set lowering and invalidation facts",
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
            status: Status::TypedExclusion,
            blocker: Some(Blocker::NeedsSelectorDependencyModel),
            carrier_class: Some(TreeReferenceCarrierClass::FormulaReference),
            host_reference_correlation: Correlation::HostReferenceHandle,
            namespace_identity_need: Namespace::HostNamespaceVersion,
            caller_context_identity_need: Caller::CallerNode,
            dependency_facts: vec![Dep::TreeReferenceCollectionMembership],
            invalidation_facts: vec![
                Invalidates::TreeReferenceMembershipChanged,
                Invalidates::TreeReferenceOrderChanged,
            ],
            successor_bead: Some("calc-4vs8.3"),
            evidence_note: "recursive selectors are held out until traversal bounds and dependency fanout are specified",
        },
        TreeReferenceImplementationInput {
            variant: Variant::StructuredTableReference,
            status: Status::AdmittedImplementationInput,
            blocker: Some(Blocker::NeedsStableStructuredTableRowMembershipAndOrderPacket),
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
            evidence_note: "calc-4vs8.2 adds typed table-context lowering for available generic OxFml table facts; stable row membership/order and exact header/totals ranges remain upstream packet blockers",
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

    use crate::dependency::{DependencyDescriptorKind, DependencyDiagnosticKind, DependencyGraph};
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
            ("=SUM(base.**)", "base.*", ".*"),
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
        let error = prebind_treecalc_formula_text(TreeNodeId(2), "=SUM(@ANCESTORS)")
            .expect_err("unsupported selector should be diagnosed");

        assert_eq!(error.diagnostics.len(), 1);
        assert_eq!(
            error.diagnostics[0].code,
            TreeCalcFormulaTextPrebindDiagnosticCode::UnsupportedSelector
        );
        assert_eq!(error.diagnostics[0].source_span_utf8, (5, 15));
        assert_eq!(error.diagnostics[0].source_token_text, "@ANCESTORS");

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
        assert_eq!(
            table.blocker,
            Some(TreeReferenceInventoryBlocker::NeedsStableStructuredTableRowMembershipAndOrderPacket)
        );
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
