#![forbid(unsafe_code)]

//! TreeCalc implementation of OxFunc's calc-time reference-system provider.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};

use oxfml_core::binding::{
    ProfilePayload, ProfileReferenceRecord, ProfileVersion, ReferenceAtomBindRequest,
    ReferenceAtomBindResult, ReferenceBindProfile, ReferenceDependencyEnvelope,
    ReferenceFingerprintPolicy, ReferenceNameBindRequest, ReferenceNormalFormKey,
    ReferenceOperatorCapabilities, ReferencePolicy, ReferenceProfileFingerprint,
    ReferenceProfileFingerprintContext, ReferenceSelectorBindRequest, ReferenceSelectorSyntax,
    ReferenceSourceInfo, ReferenceStructuredBindRequest, ReferenceValidity,
};
use oxfunc_core::resolver::{
    ReferenceDereferenceRequest, ReferenceEnumerationRequest, ReferenceFacts,
    ReferenceFactsRequest, ReferenceResolutionError, ReferenceSystemError,
    ReferenceSystemOperation, ReferenceSystemProvider, ReferenceTextResolutionMode,
    ReferenceTextResolveRequest, ResolvedReferenceCell, ResolvedReferenceExtent,
    ResolvedReferenceValues, materialize_resolved_reference_values, reference_facts,
};
use oxfunc_core::value::{
    CalcValue, CoreValue, ExcelText, ReferenceDisplay, ReferenceHandle, ReferenceHandleId,
    ReferenceLike, ReferenceSystemId, WorksheetErrorCode,
};
use serde::{Deserialize, Serialize};

use crate::dependency::TreeReferenceCollectionFamily;
use crate::formula::{
    TreeCalcChildrenReferenceCollection, TreeCalcOrderedSelectorFamily,
    TreeCalcOrderedSelectorReferenceCollection, TreeCalcReferenceLiteralArrayCollection,
    TreeCalcReferenceLiteralArrayElement,
};
use crate::sparse_reader::{
    SparseRangeReader, TreeCalcChildrenSparseReader, TreeCalcOrderedSelectorSparseReader,
    TreeCalcReferenceLiteralArraySparseReader,
};
use crate::structural::{StructuralSnapshot, TreeNodeId};
use crate::tree_reference_resolution::{
    ContextHostNameResolution, TREECALC_NAME_AMBIGUOUS_CODE, TREECALC_WORKSPACE_UNKNOWN_CODE,
    is_meta_effective, resolve_context_host_name_token, resolve_workspace_root_path,
    split_workspace_qualifier,
};
use crate::workbook_reference_catalog::{WorkspaceAliasCatalog, WorkspaceAliasLookup};

pub const TREECALC_REFERENCE_SYSTEM_ID: &str = "dna.treecalc.v1";
pub const TREECALC_NODE_PROFILE_ATOM_PREFIX: &str = "TCREF_NODE_";
pub const TREECALC_HANDLE_PROFILE_ATOM_PREFIX: &str = "TCREF_HANDLE_";

pub static TREECALC_REFERENCE_BIND_PROFILE: TreeCalcReferenceBindProfile =
    TreeCalcReferenceBindProfile;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TreeCalcReferenceBindProfile;

pub struct TreeCalcContextReferenceBindProfile<'a> {
    structural_snapshot: &'a StructuralSnapshot,
    meta_node_ids: &'a BTreeSet<TreeNodeId>,
    owner_node_id: TreeNodeId,
    synthetic_aliases: BTreeMap<String, TreeCalcProfileReference>,
    cross_workspace: Option<&'a TreeCalcCrossWorkspaceResolver<'a>>,
}

/// One sibling workspace the cross-workspace resolver can reach through a
/// registered alias (W062 D2 §9): its structural snapshot (resolution roots at
/// *this* snapshot's root — no cross-document walk-up), its meta-node set (for
/// meta-effective visibility), and its availability version (folded into the
/// cross-workspace target's identity so a workspace load/unload re-fingerprints).
#[derive(Debug, Clone, Copy)]
pub struct CrossWorkspaceEntry<'a> {
    /// The opaque workspace handle a resolved alias points to.
    pub workspace_handle: &'a str,
    /// The target workspace's structural snapshot; the path resolves rooted here.
    pub snapshot: &'a StructuralSnapshot,
    /// The target workspace's meta-node set, for meta-effective visibility.
    pub meta_node_ids: &'a BTreeSet<TreeNodeId>,
    /// The target workspace's availability version (D2 §9 identity input).
    pub availability_version: &'a str,
}

/// The tree profile's cross-workspace `!` resolution seat (W062 D2 §2/§9).
///
/// It carries the R3.2 [`WorkspaceAliasCatalog`] (the shared alias-catalog seat)
/// and the reachable sibling workspaces keyed by the opaque workspace handle a
/// resolved alias yields. `bind_name` consults it when OxFml delivers a
/// `parsed_qualifier` (the left side of `Alias!Path`):
///
/// 1. the alias resolves through the catalog (case-insensitive, shared V3 fold);
///    an unknown alias is a typed [`TREECALC_WORKSPACE_UNKNOWN_CODE`] rejection;
/// 2. the right side resolves **rooted at the target workspace's root** with no
///    walk-up ([`resolve_workspace_root_path`]); an unresolvable path is a typed
///    rejection (`#REF!`-class, the dormant-heal contract), never a silent pick;
/// 3. the resolved node becomes a cross-workspace record carrying the workspace
///    handle + target node handle + availability version, so the value flows
///    through the existing `workspace_target` seam (D2 §9).
#[derive(Debug, Clone)]
pub struct TreeCalcCrossWorkspaceResolver<'a> {
    alias_catalog: &'a WorkspaceAliasCatalog,
    workspaces_by_handle: BTreeMap<String, CrossWorkspaceEntry<'a>>,
}

/// Stable diagnostic code for a `!`-qualified reference whose alias resolved to a
/// workspace but whose right-side path did not name a node in that workspace's
/// tree (W062 D2 §9). Distinct from an unknown *alias*: the container was found,
/// the target inside it was not.
pub const TREECALC_WORKSPACE_PATH_UNRESOLVED_CODE: &str = "treecalc.workspace.path_unresolved";

/// Stable diagnostic code for a `!`-qualified reference whose alias IS registered
/// but whose target workspace is not currently loaded/reachable (W062 D2 §9).
/// This is the `#REF!`-class **dormant** outcome (heals on load under the tree
/// profile's `DormantIdentityHeal` policy), deliberately distinct from
/// [`TREECALC_WORKSPACE_UNKNOWN_CODE`] (the `#NAME?`-class *unknown-alias* case).
pub const TREECALC_WORKSPACE_UNAVAILABLE_CODE: &str = "treecalc.workspace.unavailable";

impl<'a> TreeCalcCrossWorkspaceResolver<'a> {
    /// Builds a resolver over the shared alias catalog and a set of reachable
    /// sibling workspaces. The workspaces are keyed by their opaque handle — the
    /// same handle a [`WorkspaceAliasLookup::Routed`] yields — so alias
    /// resolution and workspace lookup compose without a second name lane.
    #[must_use]
    pub fn new(
        alias_catalog: &'a WorkspaceAliasCatalog,
        workspaces: impl IntoIterator<Item = CrossWorkspaceEntry<'a>>,
    ) -> Self {
        Self {
            alias_catalog,
            workspaces_by_handle: workspaces
                .into_iter()
                .map(|entry| (entry.workspace_handle.to_string(), entry))
                .collect(),
        }
    }

    /// Resolve a `!`-qualified token `(alias, right_side)` into a cross-workspace
    /// profile reference, target-rooted (W062 D2 §9). The outcome is a typed
    /// `Bound`/`Rejected` result mirroring the local `bind_name` contract.
    fn resolve(
        &self,
        profile_id: &str,
        request: &ReferenceNameBindRequest,
        alias: &str,
        right_side: &str,
    ) -> ReferenceAtomBindResult {
        // Step 1: resolve the container alias through the shared catalog.
        let workspace_handle = match self.alias_catalog.resolve_alias(alias) {
            WorkspaceAliasLookup::Routed { workspace_id } => workspace_id,
            WorkspaceAliasLookup::Dormant { .. } => {
                return ReferenceAtomBindResult::Rejected {
                    validity: ReferenceValidity::DynamicOrHostSensitive,
                    message: format!(
                        "{TREECALC_WORKSPACE_UNKNOWN_CODE}: unknown TreeCalc workspace alias '{alias}'"
                    ),
                };
            }
        };
        // A registered alias whose workspace snapshot is not currently reachable
        // is a dormant/unavailable target — the `#REF!`-class typed rejection,
        // per the tree profile's lenient `DormantIdentityHeal` contract
        // (D2 §6/§9). This is DISTINCT from an unknown alias (`#NAME?`-class):
        // the container was registered, the workspace just is not loaded, so it
        // carries the dedicated `treecalc.workspace.unavailable` code and heals
        // on load.
        let Some(entry) = self.workspaces_by_handle.get(&workspace_handle) else {
            return ReferenceAtomBindResult::Rejected {
                validity: ReferenceValidity::DynamicOrHostSensitive,
                message: format!(
                    "{TREECALC_WORKSPACE_UNAVAILABLE_CODE}: TreeCalc workspace '{workspace_handle}' (alias '{alias}') is registered but not loaded"
                ),
            };
        };
        // Step 2: resolve the right side ROOTED AT THE TARGET ROOT (no walk-up).
        match resolve_workspace_root_path(entry.snapshot, right_side, entry.meta_node_ids) {
            Some(target_node_id) => {
                // Step 3: emit a cross-workspace record carrying the workspace
                // target so the value flows through the `workspace_target` seam.
                let reference = TreeCalcProfileReference::CrossWorkspaceNode {
                    workspace_handle: workspace_handle.clone(),
                    node_id: target_node_id.0,
                    handle: format!(
                        "{}#{}",
                        workspace_handle,
                        treecalc_node_reference_target(target_node_id)
                    ),
                    source_text: request.source_text.clone(),
                    availability_version: entry.availability_version.to_string(),
                    parsed_qualifier: Some(alias.to_string()),
                };
                ReferenceAtomBindResult::Bound(treecalc_profile_reference_record_from_parts(
                    profile_id,
                    request.source_channel,
                    request.source_span,
                    &request.source_text,
                    Some(alias.to_string()),
                    reference,
                ))
            }
            None => ReferenceAtomBindResult::Rejected {
                validity: ReferenceValidity::DynamicOrHostSensitive,
                message: format!(
                    "{TREECALC_WORKSPACE_PATH_UNRESOLVED_CODE}: '{right_side}' does not resolve in TreeCalc workspace '{workspace_handle}' (alias '{alias}')"
                ),
            },
        }
    }
}

impl<'a> TreeCalcContextReferenceBindProfile<'a> {
    #[must_use]
    pub fn new(
        structural_snapshot: &'a StructuralSnapshot,
        meta_node_ids: &'a BTreeSet<TreeNodeId>,
        owner_node_id: TreeNodeId,
    ) -> Self {
        Self {
            structural_snapshot,
            meta_node_ids,
            owner_node_id,
            synthetic_aliases: BTreeMap::new(),
            cross_workspace: None,
        }
    }

    #[must_use]
    pub fn with_synthetic_aliases(
        mut self,
        synthetic_aliases: impl IntoIterator<Item = (String, TreeCalcProfileReference)>,
    ) -> Self {
        self.synthetic_aliases = synthetic_aliases.into_iter().collect();
        self
    }

    /// Attach the cross-workspace `!` resolution seat (W062 D2 §9). With it,
    /// `bind_name` resolves an `Alias!Path` qualifier through the shared
    /// [`WorkspaceAliasCatalog`] to a sibling workspace, target-rooted. Without
    /// it, a `!`-qualified name is a typed `treecalc.workspace.unknown`
    /// rejection (no alias catalog to resolve against).
    #[must_use]
    pub fn with_cross_workspace_resolver(
        mut self,
        resolver: &'a TreeCalcCrossWorkspaceResolver<'a>,
    ) -> Self {
        self.cross_workspace = Some(resolver);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TreeCalcProfileReference {
    Node {
        node_id: u64,
        handle: String,
        source_text: String,
        parsed_qualifier: Option<String>,
    },
    /// A node in a *sibling workspace*, resolved through the `!` container
    /// qualifier against the shared alias catalog (W062 D2 §9). Carries the
    /// resolved workspace handle and availability version so the value flows
    /// through the `workspace_target` seam; the right side resolved rooted at the
    /// target workspace's root (no walk-up).
    CrossWorkspaceNode {
        workspace_handle: String,
        node_id: u64,
        handle: String,
        source_text: String,
        availability_version: String,
        parsed_qualifier: Option<String>,
    },
    OpaqueHandle {
        handle: String,
        source_text: String,
        parsed_qualifier: Option<String>,
    },
    Selector {
        handle: String,
        source_text: String,
        selector_family: String,
        base_handle: Option<String>,
    },
    StructuredTable {
        handle: String,
        source_text: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TreeCalcSelectorHandle<'a> {
    family: &'a str,
    base_handle: &'a str,
}

impl TreeCalcProfileReference {
    #[must_use]
    pub fn handle(&self) -> &str {
        match self {
            TreeCalcProfileReference::Node { handle, .. }
            | TreeCalcProfileReference::CrossWorkspaceNode { handle, .. }
            | TreeCalcProfileReference::OpaqueHandle { handle, .. }
            | TreeCalcProfileReference::Selector { handle, .. }
            | TreeCalcProfileReference::StructuredTable { handle, .. } => handle,
        }
    }
}

impl ReferenceBindProfile for TreeCalcReferenceBindProfile {
    fn profile_id(&self) -> &str {
        TREECALC_REFERENCE_SYSTEM_ID
    }

    fn profile_version(&self) -> ProfileVersion {
        ProfileVersion::v1()
    }

    fn reference_policy(&self) -> ReferencePolicy {
        ReferencePolicy::ProfileSymbolic
    }

    fn fingerprint_policy(&self) -> ReferenceFingerprintPolicy {
        ReferenceFingerprintPolicy::ExcludeCallerAnchorForTemplate
    }

    fn fingerprint(
        &self,
        context: &ReferenceProfileFingerprintContext,
    ) -> ReferenceProfileFingerprint {
        ReferenceProfileFingerprint(format!(
            "{}:{}:book={}:sheet={}:structure={}",
            self.profile_id(),
            self.profile_version().0,
            context.workbook_id,
            context.sheet_id,
            context.structure_context_version
        ))
    }

    fn operator_capabilities(&self) -> ReferenceOperatorCapabilities {
        ReferenceOperatorCapabilities {
            range: false,
            union: false,
            intersection: false,
            spill: false,
        }
    }

    fn selector_syntax(&self) -> Vec<ReferenceSelectorSyntax> {
        treecalc_selector_syntax()
    }

    fn bind_atom(&self, request: &ReferenceAtomBindRequest) -> ReferenceAtomBindResult {
        let Some(reference) = parse_treecalc_profile_reference_atom(request) else {
            return ReferenceAtomBindResult::LegacyCompatibility;
        };
        ReferenceAtomBindResult::Bound(treecalc_profile_reference_record(
            self.profile_id(),
            request,
            reference,
        ))
    }

    fn bind_structured_reference(
        &self,
        request: &ReferenceStructuredBindRequest,
    ) -> ReferenceAtomBindResult {
        if !looks_like_explicit_treecalc_structured_reference(&request.source_text) {
            return ReferenceAtomBindResult::LegacyCompatibility;
        }
        let handle = treecalc_structured_table_reference_target(&request.source_text);
        ReferenceAtomBindResult::Bound(treecalc_profile_reference_record_from_parts(
            self.profile_id(),
            request.source_channel,
            request.source_span,
            &request.source_text,
            None,
            TreeCalcProfileReference::StructuredTable {
                handle,
                source_text: request.source_text.clone(),
            },
        ))
    }

    fn dependency_hints(
        &self,
        reference: &ProfileReferenceRecord,
        _context: &ReferenceProfileFingerprintContext,
    ) -> ReferenceDependencyEnvelope {
        ReferenceDependencyEnvelope::Static {
            profile_id: self.profile_id().to_string(),
            dependency_key: reference.normal_form_key.0.clone(),
        }
    }
}

impl ReferenceBindProfile for TreeCalcContextReferenceBindProfile<'_> {
    fn profile_id(&self) -> &str {
        TREECALC_REFERENCE_SYSTEM_ID
    }

    fn profile_version(&self) -> ProfileVersion {
        ProfileVersion::v1()
    }

    fn reference_policy(&self) -> ReferencePolicy {
        ReferencePolicy::ProfileSymbolic
    }

    fn fingerprint_policy(&self) -> ReferenceFingerprintPolicy {
        ReferenceFingerprintPolicy::ExcludeCallerAnchorForTemplate
    }

    fn fingerprint(
        &self,
        context: &ReferenceProfileFingerprintContext,
    ) -> ReferenceProfileFingerprint {
        ReferenceProfileFingerprint(format!(
            "{}:{}:book={}:sheet={}:structure={}:owner={}",
            self.profile_id(),
            self.profile_version().0,
            context.workbook_id,
            context.sheet_id,
            context.structure_context_version,
            self.owner_node_id.0
        ))
    }

    fn operator_capabilities(&self) -> ReferenceOperatorCapabilities {
        ReferenceOperatorCapabilities {
            range: false,
            union: false,
            intersection: false,
            spill: false,
        }
    }

    fn selector_syntax(&self) -> Vec<ReferenceSelectorSyntax> {
        treecalc_selector_syntax()
    }

    fn bind_atom(&self, request: &ReferenceAtomBindRequest) -> ReferenceAtomBindResult {
        TREECALC_REFERENCE_BIND_PROFILE.bind_atom(request)
    }

    fn bind_name(&self, request: &ReferenceNameBindRequest) -> ReferenceAtomBindResult {
        if let Some(alias) = request.parsed_qualifier.as_deref() {
            // A `!` container qualifier (W062 D2 §2/§9): the left side (`alias`)
            // names a workspace; the right side (`source_text`) resolves within
            // it. With a cross-workspace resolver seat, resolve through the
            // shared alias catalog, target-rooted. Without one, no catalog is
            // available: a typed `treecalc.workspace.unknown` rejection.
            return match self.cross_workspace {
                Some(resolver) => {
                    resolver.resolve(self.profile_id(), request, alias, &request.source_text)
                }
                None => ReferenceAtomBindResult::Rejected {
                    validity: ReferenceValidity::DynamicOrHostSensitive,
                    message: format!(
                        "{TREECALC_WORKSPACE_UNKNOWN_CODE}: unknown TreeCalc workspace alias '{alias}'"
                    ),
                },
            };
        }
        // Some binder paths deliver the qualifier inline in the source text
        // rather than as a split `parsed_qualifier`. Route those through the same
        // cross-workspace seam so `Alias!Path` resolves identically.
        if let Some((alias, right_side)) = split_workspace_qualifier(&request.source_text) {
            return match self.cross_workspace {
                Some(resolver) => resolver.resolve(self.profile_id(), request, alias, right_side),
                None => ReferenceAtomBindResult::Rejected {
                    validity: ReferenceValidity::DynamicOrHostSensitive,
                    message: format!(
                        "{TREECALC_WORKSPACE_UNKNOWN_CODE}: unknown TreeCalc workspace alias '{alias}'"
                    ),
                },
            };
        }
        if let Some(reference) = self.synthetic_aliases.get(&request.source_text) {
            return ReferenceAtomBindResult::Bound(treecalc_profile_reference_record_from_parts(
                self.profile_id(),
                request.source_channel,
                request.source_span,
                &request.source_text,
                None,
                reference.clone(),
            ));
        }
        match resolve_context_host_name_token(
            &request.source_text,
            self.owner_node_id,
            self.structural_snapshot,
            self.meta_node_ids,
        ) {
            ContextHostNameResolution::Resolved(node_id) => {
                ReferenceAtomBindResult::Bound(treecalc_profile_reference_record_from_parts(
                    self.profile_id(),
                    request.source_channel,
                    request.source_span,
                    &request.source_text,
                    None,
                    TreeCalcProfileReference::Node {
                        node_id: node_id.0,
                        handle: treecalc_node_reference_target(node_id),
                        source_text: request.source_text.clone(),
                        parsed_qualifier: None,
                    },
                ))
            }
            ContextHostNameResolution::Ambiguous => ReferenceAtomBindResult::Rejected {
                validity: ReferenceValidity::DynamicOrHostSensitive,
                message: format!(
                    "{TREECALC_NAME_AMBIGUOUS_CODE}: ambiguous TreeCalc name '{}'",
                    request.source_text
                ),
            },
            ContextHostNameResolution::Unsupported(reason) => ReferenceAtomBindResult::Rejected {
                validity: ReferenceValidity::DynamicOrHostSensitive,
                message: reason.to_string(),
            },
            ContextHostNameResolution::Unresolved => ReferenceAtomBindResult::Unsupported,
        }
    }

    fn bind_selector(&self, request: &ReferenceSelectorBindRequest) -> ReferenceAtomBindResult {
        let base_handle = request
            .base
            .as_ref()
            .and_then(|record| decode_treecalc_reference_payload(&record.profile_payload))
            .map(|reference| reference.handle().to_string())
            .unwrap_or_else(|| treecalc_node_reference_target(self.owner_node_id));
        let handle = format!(
            "treecalc-hostref:v1:selector:{}:base:{}",
            request.selector_family, base_handle
        );
        ReferenceAtomBindResult::Bound(treecalc_profile_reference_record_from_parts(
            self.profile_id(),
            request.source_channel,
            request.source_span,
            &request.source_text,
            None,
            TreeCalcProfileReference::Selector {
                handle,
                source_text: request.source_text.clone(),
                selector_family: request.selector_family.clone(),
                base_handle: Some(base_handle),
            },
        ))
    }

    fn bind_structured_reference(
        &self,
        request: &ReferenceStructuredBindRequest,
    ) -> ReferenceAtomBindResult {
        if !looks_like_explicit_treecalc_structured_reference(&request.source_text) {
            return ReferenceAtomBindResult::LegacyCompatibility;
        }
        let handle = treecalc_structured_table_reference_target(&request.source_text);
        ReferenceAtomBindResult::Bound(treecalc_profile_reference_record_from_parts(
            self.profile_id(),
            request.source_channel,
            request.source_span,
            &request.source_text,
            None,
            TreeCalcProfileReference::StructuredTable {
                handle,
                source_text: request.source_text.clone(),
            },
        ))
    }

    fn dependency_hints(
        &self,
        reference: &ProfileReferenceRecord,
        _context: &ReferenceProfileFingerprintContext,
    ) -> ReferenceDependencyEnvelope {
        ReferenceDependencyEnvelope::Static {
            profile_id: self.profile_id().to_string(),
            dependency_key: reference.normal_form_key.0.clone(),
        }
    }
}

// W062 R3.1 (D2 §1): both shipped tree profile objects are the same profile
// family (`dna.treecalc.v1`) and share the one tree-profile vocabulary — one
// vocabulary per profile. The subtrait is the OxCalc-internal handle; the OxFml
// seam still sees only `dyn ReferenceBindProfile`.
impl crate::reference_vocabulary::OxCalcReferenceProfile for TreeCalcReferenceBindProfile {
    fn vocabulary(&self) -> &dyn crate::reference_vocabulary::StructuralVocabulary {
        &crate::reference_vocabulary::TREECALC_VOCABULARY
    }
}

impl crate::reference_vocabulary::OxCalcReferenceProfile
    for TreeCalcContextReferenceBindProfile<'_>
{
    fn vocabulary(&self) -> &dyn crate::reference_vocabulary::StructuralVocabulary {
        &crate::reference_vocabulary::TREECALC_VOCABULARY
    }
}

#[must_use]
pub fn treecalc_reference_bind_profile() -> &'static dyn ReferenceBindProfile {
    &TREECALC_REFERENCE_BIND_PROFILE
}

#[must_use]
pub fn decode_treecalc_reference_payload(
    payload: &ProfilePayload,
) -> Option<TreeCalcProfileReference> {
    if payload.payload_kind != "treecalc-reference" || payload.encoding != "json" {
        return None;
    }
    serde_json::from_str(&payload.data).ok()
}

#[must_use]
pub fn treecalc_reference_like_from_profile_record(
    record: &ProfileReferenceRecord,
) -> Option<ReferenceLike> {
    if record.profile_id != TREECALC_REFERENCE_SYSTEM_ID {
        return None;
    }
    let reference = decode_treecalc_reference_payload(&record.profile_payload)?;
    if record.normal_form_key.0 != reference.handle() {
        return None;
    }
    Some(treecalc_opaque_reference_like(
        reference.handle().to_string(),
        record
            .render_hint
            .clone()
            .unwrap_or_else(|| reference.handle().to_string()),
    ))
}

#[must_use]
pub fn treecalc_reference_system_id() -> ReferenceSystemId {
    ReferenceSystemId(TREECALC_REFERENCE_SYSTEM_ID.to_string())
}

#[must_use]
pub fn treecalc_node_reference_target(node_id: TreeNodeId) -> String {
    format!("treecalc.node:{}", node_id.0)
}

#[must_use]
pub fn treecalc_structured_table_reference_target(source_text: &str) -> String {
    format!("treecalc.table-ref:v1:{source_text}")
}

#[must_use]
pub fn treecalc_node_id_from_profile_handle(handle: &str) -> Option<TreeNodeId> {
    handle
        .strip_prefix("treecalc.node:")
        .and_then(|id| id.parse::<u64>().ok())
        .map(TreeNodeId)
}

#[must_use]
pub fn treecalc_node_reference_like(node_id: TreeNodeId) -> ReferenceLike {
    let target = treecalc_node_reference_target(node_id);
    treecalc_opaque_reference_like(target.clone(), target)
}

#[must_use]
pub fn treecalc_collection_reference_like(host_ref_handle: impl Into<String>) -> ReferenceLike {
    let host_ref_handle = host_ref_handle.into();
    treecalc_opaque_reference_like(host_ref_handle.clone(), host_ref_handle)
}

#[must_use]
pub fn treecalc_opaque_reference_like(
    handle_id: impl Into<String>,
    display: impl Into<String>,
) -> ReferenceLike {
    let handle_id = handle_id.into();
    let display = display.into();
    ReferenceLike::opaque(
        treecalc_reference_system_id(),
        ReferenceHandle {
            id: ReferenceHandleId::from_bytes(handle_id.as_bytes().to_vec()),
        },
        Some(ReferenceDisplay {
            text: ExcelText::from_interop_assignment(&display),
        }),
    )
}

fn parse_treecalc_profile_reference_atom(
    request: &ReferenceAtomBindRequest,
) -> Option<TreeCalcProfileReference> {
    if request.parsed_qualifier.is_some() {
        return None;
    }
    let source_text = request.source_text.trim();
    if let Some(node_text) =
        strip_ascii_prefix_case_insensitive(source_text, TREECALC_NODE_PROFILE_ATOM_PREFIX)
    {
        let node_id = node_text.parse::<u64>().ok()?;
        return Some(TreeCalcProfileReference::Node {
            node_id,
            handle: treecalc_node_reference_target(TreeNodeId(node_id)),
            source_text: request.source_text.clone(),
            parsed_qualifier: request.parsed_qualifier.clone(),
        });
    }
    if let Some(handle_slug) =
        strip_ascii_prefix_case_insensitive(source_text, TREECALC_HANDLE_PROFILE_ATOM_PREFIX)
    {
        if handle_slug.is_empty()
            || !handle_slug
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
        {
            return None;
        }
        return Some(TreeCalcProfileReference::OpaqueHandle {
            handle: format!("treecalc-hostref:v1:{handle_slug}"),
            source_text: request.source_text.clone(),
            parsed_qualifier: request.parsed_qualifier.clone(),
        });
    }
    None
}

fn treecalc_profile_reference_record(
    profile_id: &str,
    request: &ReferenceAtomBindRequest,
    reference: TreeCalcProfileReference,
) -> ProfileReferenceRecord {
    treecalc_profile_reference_record_from_parts(
        profile_id,
        request.source_channel,
        request.source_span,
        &request.source_text,
        request.parsed_qualifier.clone(),
        reference,
    )
}

fn treecalc_profile_reference_record_from_parts(
    profile_id: &str,
    source_channel: oxfml_core::FormulaChannelKind,
    source_span: oxfml_core::syntax::token::TextSpan,
    source_text: &str,
    parsed_qualifier: Option<String>,
    reference: TreeCalcProfileReference,
) -> ProfileReferenceRecord {
    let handle = reference.handle().to_string();
    let payload_data =
        serde_json::to_string(&reference).expect("treecalc reference payload serializes");
    ProfileReferenceRecord {
        profile_id: profile_id.to_string(),
        profile_version: ProfileVersion::v1(),
        source_info: ReferenceSourceInfo {
            source_channel,
            source_span,
            source_text: source_text.to_string(),
            parsed_qualifier,
            address_fidelity: Some(source_text.to_string()),
        },
        profile_payload: ProfilePayload {
            payload_kind: "treecalc-reference".to_string(),
            encoding: "json".to_string(),
            data: payload_data,
        },
        normal_form_key: ReferenceNormalFormKey(handle),
        render_hint: Some(source_text.to_string()),
        validity: ReferenceValidity::ValidAfterInstantiation,
    }
}

fn treecalc_selector_syntax() -> Vec<ReferenceSelectorSyntax> {
    vec![
        ReferenceSelectorSyntax::collection("CHILDREN", "children"),
        ReferenceSelectorSyntax::structural_selector("CHILDREN", "children"),
        ReferenceSelectorSyntax::collection("*", "children"),
        ReferenceSelectorSyntax::structural_selector("*", "children"),
        ReferenceSelectorSyntax::collection("PRECEDING", "preceding"),
        ReferenceSelectorSyntax::structural_selector("PRECEDING", "preceding"),
        ReferenceSelectorSyntax::collection("FOLLOWING", "following"),
        ReferenceSelectorSyntax::structural_selector("FOLLOWING", "following"),
        ReferenceSelectorSyntax::collection("ANCESTORS", "ancestors"),
        ReferenceSelectorSyntax::structural_selector("ANCESTORS", "ancestors"),
        ReferenceSelectorSyntax::collection("DESCENDANTS", "recursive_descendants"),
        ReferenceSelectorSyntax::structural_selector("DESCENDANTS", "recursive_descendants"),
        ReferenceSelectorSyntax::collection("**", "recursive_descendants"),
        ReferenceSelectorSyntax::structural_selector("**", "recursive_descendants"),
        ReferenceSelectorSyntax::collection("PARENT", "parent"),
        ReferenceSelectorSyntax::structural_selector("PARENT", "parent"),
        ReferenceSelectorSyntax::collection("SELF", "self"),
        ReferenceSelectorSyntax::structural_selector("SELF", "self"),
        ReferenceSelectorSyntax::collection("PREV", "previous"),
        ReferenceSelectorSyntax::structural_selector("PREV", "previous"),
        ReferenceSelectorSyntax::collection("NEXT", "next"),
        ReferenceSelectorSyntax::structural_selector("NEXT", "next"),
        ReferenceSelectorSyntax::collection("NAME", "metadata_name"),
        ReferenceSelectorSyntax::structural_selector("NAME", "metadata_name"),
        ReferenceSelectorSyntax::collection("INDEX", "metadata_index"),
        ReferenceSelectorSyntax::structural_selector("INDEX", "metadata_index"),
        ReferenceSelectorSyntax::collection("FORMULA", "metadata_formula"),
        ReferenceSelectorSyntax::structural_selector("FORMULA", "metadata_formula"),
    ]
}

fn parse_treecalc_selector_handle(handle: &str) -> Option<TreeCalcSelectorHandle<'_>> {
    let rest = handle.strip_prefix("treecalc-hostref:v1:selector:")?;
    let (family, base_handle) = rest.split_once(":base:")?;
    Some(TreeCalcSelectorHandle {
        family,
        base_handle,
    })
}

fn looks_like_explicit_treecalc_structured_reference(source_text: &str) -> bool {
    let Some((table_name, rest)) = source_text.split_once('[') else {
        return false;
    };
    !table_name.trim().is_empty() && rest.ends_with(']')
}

fn visible_child_by_symbol(
    structural_snapshot: &StructuralSnapshot,
    meta_node_ids: &BTreeSet<TreeNodeId>,
    base_node_id: TreeNodeId,
    symbol: &str,
) -> Option<TreeNodeId> {
    structural_snapshot
        .try_get_node(base_node_id)?
        .child_ids
        .iter()
        .copied()
        .find(|child_id| {
            structural_snapshot
                .try_get_node(*child_id)
                .is_some_and(|child| child.symbol.eq_ignore_ascii_case(symbol))
                && !is_meta_effective(*child_id, structural_snapshot, meta_node_ids)
        })
}

fn visible_child_ids(
    structural_snapshot: &StructuralSnapshot,
    meta_node_ids: &BTreeSet<TreeNodeId>,
    base_node_id: TreeNodeId,
) -> Vec<TreeNodeId> {
    structural_snapshot
        .try_get_node(base_node_id)
        .map_or_else(Vec::new, |node| {
            node.child_ids
                .iter()
                .copied()
                .filter(|child_id| {
                    !is_meta_effective(*child_id, structural_snapshot, meta_node_ids)
                })
                .collect()
        })
}

fn sibling_offset_node_id(
    structural_snapshot: &StructuralSnapshot,
    meta_node_ids: &BTreeSet<TreeNodeId>,
    base_node_id: TreeNodeId,
    offset: isize,
) -> Option<TreeNodeId> {
    let parent_id = structural_snapshot.parent_id_of(base_node_id)?;
    let siblings = visible_child_ids(structural_snapshot, meta_node_ids, parent_id);
    let base_index = siblings
        .iter()
        .position(|node_id| *node_id == base_node_id)?;
    siblings
        .get(base_index.checked_add_signed(offset)?)
        .copied()
}

fn self_anchor_node_id(
    structural_snapshot: &StructuralSnapshot,
    meta_node_ids: &BTreeSet<TreeNodeId>,
    base_node_id: TreeNodeId,
) -> TreeNodeId {
    structural_snapshot
        .parent_id_of(base_node_id)
        .filter(|parent_id| !is_meta_effective(*parent_id, structural_snapshot, meta_node_ids))
        .unwrap_or(base_node_id)
}

fn selector_family_node_id(
    structural_snapshot: &StructuralSnapshot,
    meta_node_ids: &BTreeSet<TreeNodeId>,
    base_node_id: TreeNodeId,
    family: &str,
) -> Option<TreeNodeId> {
    match family {
        "self" => Some(self_anchor_node_id(
            structural_snapshot,
            meta_node_ids,
            base_node_id,
        )),
        "parent" => structural_snapshot
            .parent_id_of(base_node_id)
            .filter(|parent_id| !is_meta_effective(*parent_id, structural_snapshot, meta_node_ids)),
        "prev" | "previous" => {
            sibling_offset_node_id(structural_snapshot, meta_node_ids, base_node_id, -1)
        }
        "next" => sibling_offset_node_id(structural_snapshot, meta_node_ids, base_node_id, 1),
        "metadata_name" | "metadata_index" | "metadata_formula" => None,
        member => visible_child_by_symbol(structural_snapshot, meta_node_ids, base_node_id, member),
    }
}

#[must_use]
pub fn treecalc_selector_handle_target_node_id(
    handle: &str,
    owner_node_id: TreeNodeId,
    structural_snapshot: &StructuralSnapshot,
    meta_node_ids: &BTreeSet<TreeNodeId>,
) -> Option<TreeNodeId> {
    if let Some(node_id) = treecalc_node_id_from_profile_handle(handle) {
        return Some(node_id);
    }
    let selector = parse_treecalc_selector_handle(handle)?;
    let base_node_id = treecalc_selector_handle_target_node_id(
        selector.base_handle,
        owner_node_id,
        structural_snapshot,
        meta_node_ids,
    )
    .unwrap_or(owner_node_id);
    selector_family_node_id(
        structural_snapshot,
        meta_node_ids,
        base_node_id,
        selector.family,
    )
}

fn metadata_selector_value(
    family: &str,
    target_node_id: TreeNodeId,
    owner_node_id: TreeNodeId,
    owner_formula_source_text: Option<&str>,
    structural_snapshot: &StructuralSnapshot,
    meta_node_ids: &BTreeSet<TreeNodeId>,
) -> Option<CalcValue> {
    match family {
        "metadata_name" => structural_snapshot
            .try_get_node(target_node_id)
            .map(|node| CalcValue::text(ExcelText::from_interop_assignment(&node.symbol))),
        "metadata_index" => {
            let parent_id = structural_snapshot.parent_id_of(target_node_id)?;
            let ordinal = visible_child_ids(structural_snapshot, meta_node_ids, parent_id)
                .into_iter()
                .position(|node_id| node_id == target_node_id)?
                + 1;
            Some(CalcValue::number(ordinal as f64))
        }
        "metadata_formula" => {
            if target_node_id == owner_node_id {
                Some(CalcValue::text(ExcelText::from_interop_assignment(
                    owner_formula_source_text.unwrap_or_default(),
                )))
            } else {
                Some(CalcValue::error(WorksheetErrorCode::Value))
            }
        }
        _ => None,
    }
}

fn strip_ascii_prefix_case_insensitive<'a>(text: &'a str, prefix: &str) -> Option<&'a str> {
    if text.len() < prefix.len() || !text.is_char_boundary(prefix.len()) {
        return None;
    }
    let (head, tail) = text.split_at(prefix.len());
    head.eq_ignore_ascii_case(prefix).then_some(tail)
}

pub struct TreeCalcReferenceSystemProvider<'a> {
    structural_snapshot: Option<&'a StructuralSnapshot>,
    meta_node_ids: Option<&'a BTreeSet<TreeNodeId>>,
    owner_node_id: Option<TreeNodeId>,
    owner_formula_source_text: Option<&'a str>,
    published_calc_values: Option<&'a BTreeMap<TreeNodeId, CalcValue>>,
    published_text_values: Option<&'a BTreeMap<TreeNodeId, String>>,
    sparse_reference_values: Vec<TreeCalcResolvedReferenceValues>,
    collection_descriptors:
        BTreeMap<TreeCalcReferenceDescriptorIdentity, TreeCalcCollectionReferenceDescriptor>,
    text_resolutions: RefCell<Vec<TreeCalcRuntimeReferenceTextResolution>>,
}

impl<'a> TreeCalcReferenceSystemProvider<'a> {
    #[must_use]
    pub fn new(
        structural_snapshot: &'a StructuralSnapshot,
        meta_node_ids: &'a BTreeSet<TreeNodeId>,
        owner_node_id: TreeNodeId,
        published_calc_values: &'a BTreeMap<TreeNodeId, CalcValue>,
    ) -> Self {
        Self {
            structural_snapshot: Some(structural_snapshot),
            meta_node_ids: Some(meta_node_ids),
            owner_node_id: Some(owner_node_id),
            owner_formula_source_text: None,
            published_calc_values: Some(published_calc_values),
            published_text_values: None,
            sparse_reference_values: Vec::new(),
            collection_descriptors: BTreeMap::new(),
            text_resolutions: RefCell::new(Vec::new()),
        }
    }

    #[must_use]
    pub fn sparse_only() -> Self {
        Self {
            structural_snapshot: None,
            meta_node_ids: None,
            owner_node_id: None,
            owner_formula_source_text: None,
            published_calc_values: None,
            published_text_values: None,
            sparse_reference_values: Vec::new(),
            collection_descriptors: BTreeMap::new(),
            text_resolutions: RefCell::new(Vec::new()),
        }
    }

    #[must_use]
    pub fn with_sparse_reference_values(
        mut self,
        reference: ReferenceLike,
        values: ResolvedReferenceValues,
    ) -> Self {
        self.sparse_reference_values
            .push(TreeCalcResolvedReferenceValues { reference, values });
        self
    }

    #[must_use]
    pub fn with_owner_formula_source_text(mut self, source_text: &'a str) -> Self {
        self.owner_formula_source_text = Some(source_text);
        self
    }

    #[must_use]
    pub fn with_published_text_values(
        mut self,
        published_text_values: &'a BTreeMap<TreeNodeId, String>,
    ) -> Self {
        self.published_text_values = Some(published_text_values);
        self
    }

    #[must_use]
    pub fn with_sparse_reader(
        self,
        reference: ReferenceLike,
        reader: &impl SparseRangeReader,
    ) -> Self {
        self.with_sparse_reference_values(reference, resolved_values_from_sparse_reader(reader))
    }

    #[must_use]
    pub fn with_collection_descriptor(
        mut self,
        descriptor: TreeCalcCollectionReferenceDescriptor,
    ) -> Self {
        self.collection_descriptors
            .insert(descriptor.descriptor_identity(), descriptor);
        self
    }

    #[must_use]
    pub fn collection_descriptor_count(&self) -> usize {
        self.collection_descriptors.len()
    }

    #[must_use]
    pub fn runtime_text_resolutions(&self) -> Vec<TreeCalcRuntimeReferenceTextResolution> {
        self.text_resolutions.borrow().clone()
    }

    fn treecalc_reference_error(&self, reference: &ReferenceLike) -> ReferenceResolutionError {
        ReferenceResolutionError::UnresolvedReference {
            target: reference.target().to_string(),
        }
    }
}

impl ReferenceSystemProvider for TreeCalcReferenceSystemProvider<'_> {
    fn dereference(
        &self,
        request: &ReferenceDereferenceRequest,
    ) -> Result<CalcValue, ReferenceResolutionError> {
        if let Some(values) = self.values_from_collection_descriptor(&request.reference)? {
            return dereference_resolved_reference_values(&values);
        }
        if let Some(entry) = self
            .sparse_reference_values
            .iter()
            .find(|entry| references_match(&entry.reference, &request.reference))
        {
            return dereference_resolved_reference_values(&entry.values);
        }

        let Some(node_id) = treecalc_node_id_from_reference(&request.reference) else {
            let Some(value) = self.value_from_selector_handle(&request.reference)? else {
                return Err(self.treecalc_reference_error(&request.reference));
            };
            return Ok(value);
        };
        Ok(self.value_for_node(node_id))
    }

    fn enumerate_values(
        &self,
        request: &ReferenceEnumerationRequest,
    ) -> Result<Option<ResolvedReferenceValues>, ReferenceResolutionError> {
        if let Some(values) = self.values_from_collection_descriptor(&request.reference)? {
            return Ok(Some(values));
        }

        Ok(self
            .sparse_reference_values
            .iter()
            .find(|entry| references_match(&entry.reference, &request.reference))
            .map(|entry| entry.values.clone())
            .or_else(|| self.values_from_node_reference(&request.reference))
            .or_else(|| self.values_from_selector_reference(&request.reference)))
    }

    fn resolve_text(
        &self,
        request: &ReferenceTextResolveRequest,
    ) -> Result<ReferenceLike, ReferenceSystemError> {
        if request.mode != ReferenceTextResolutionMode::Indirect {
            return Err(ReferenceSystemError::Unsupported {
                operation: ReferenceSystemOperation::ResolveText,
            });
        }
        let (Some(structural_snapshot), Some(meta_node_ids), Some(owner_node_id)) = (
            self.structural_snapshot,
            self.meta_node_ids,
            self.owner_node_id,
        ) else {
            return Err(ReferenceSystemError::Unsupported {
                operation: ReferenceSystemOperation::ResolveText,
            });
        };
        match resolve_context_host_name_token(
            &request.text,
            owner_node_id,
            structural_snapshot,
            meta_node_ids,
        ) {
            ContextHostNameResolution::Resolved(target_node_id) => {
                let reference = treecalc_node_reference_like(target_node_id);
                self.text_resolutions
                    .borrow_mut()
                    .push(TreeCalcRuntimeReferenceTextResolution {
                        owner_node_id,
                        target_node_id,
                        reference_text: request.text.clone(),
                        mode: request.mode,
                        a1_style: request.a1_style,
                        reference_like: reference.clone(),
                    });
                Ok(reference)
            }
            ContextHostNameResolution::Ambiguous => Err(ReferenceSystemError::ProviderFailure {
                detail: format!("ambiguous TreeCalc reference text '{}'", request.text),
            }),
            ContextHostNameResolution::Unsupported(reason) => {
                Err(ReferenceSystemError::ProviderFailure {
                    detail: format!(
                        "unsupported TreeCalc reference text '{}': {reason}",
                        request.text
                    ),
                })
            }
            ContextHostNameResolution::Unresolved => {
                Err(ReferenceSystemError::InvalidReferenceText {
                    text: request.text.clone(),
                })
            }
        }
    }

    fn facts(
        &self,
        request: &ReferenceFactsRequest,
    ) -> Result<ReferenceFacts, ReferenceSystemError> {
        Ok(reference_facts(&request.reference))
    }
}

impl TreeCalcReferenceSystemProvider<'_> {
    fn value_from_selector_handle(
        &self,
        reference: &ReferenceLike,
    ) -> Result<Option<CalcValue>, ReferenceResolutionError> {
        let Some(handle) = treecalc_handle_text(reference) else {
            return Ok(None);
        };
        let Some(selector) = parse_treecalc_selector_handle(&handle) else {
            return Ok(None);
        };
        let (Some(structural_snapshot), Some(meta_node_ids), Some(owner_node_id)) = (
            self.structural_snapshot,
            self.meta_node_ids,
            self.owner_node_id,
        ) else {
            return Ok(None);
        };
        let base_node_id = treecalc_selector_handle_target_node_id(
            selector.base_handle,
            owner_node_id,
            structural_snapshot,
            meta_node_ids,
        )
        .unwrap_or(owner_node_id);
        if let Some(value) = metadata_selector_value(
            selector.family,
            base_node_id,
            owner_node_id,
            self.owner_formula_source_text,
            structural_snapshot,
            meta_node_ids,
        ) {
            return Ok(Some(value));
        }
        let Some(target_node_id) = selector_family_node_id(
            structural_snapshot,
            meta_node_ids,
            base_node_id,
            selector.family,
        ) else {
            return Ok(None);
        };
        Ok(Some(self.value_for_node(target_node_id)))
    }

    fn value_for_node(&self, node_id: TreeNodeId) -> CalcValue {
        if let Some(value) = self
            .published_calc_values
            .and_then(|values| values.get(&node_id))
        {
            return value.clone();
        }
        self.published_text_values
            .and_then(|values| values.get(&node_id))
            .map_or_else(
                || CalcValue::number(0.0),
                |value| treecalc_published_text_to_calc_value(value),
            )
    }

    fn values_from_node_reference(
        &self,
        reference: &ReferenceLike,
    ) -> Option<ResolvedReferenceValues> {
        let node_id = treecalc_node_id_from_reference(reference)?;
        Some(resolved_values_from_calc_value(
            self.value_for_node(node_id),
        ))
    }

    fn values_from_selector_reference(
        &self,
        reference: &ReferenceLike,
    ) -> Option<ResolvedReferenceValues> {
        let value = self.value_from_selector_handle(reference).ok().flatten()?;
        Some(resolved_values_from_calc_value(value))
    }
}

fn treecalc_published_text_to_calc_value(value: &str) -> CalcValue {
    if value.is_empty() {
        return CalcValue::empty();
    }
    value.parse::<f64>().map_or_else(
        |_| CalcValue::text(ExcelText::from_interop_assignment(value)),
        CalcValue::number,
    )
}

fn resolved_values_from_calc_value(value: CalcValue) -> ResolvedReferenceValues {
    match &value.core {
        CoreValue::Array(array) => {
            let shape = array.shape();
            let mut cells = Vec::new();
            for row in 0..shape.rows {
                for col in 0..shape.cols {
                    if let Some(cell) = array.get(row, col) {
                        cells.push(ResolvedReferenceCell::new(row + 1, col + 1, cell.clone()));
                    }
                }
            }
            ResolvedReferenceValues::new(
                ResolvedReferenceExtent::new(shape.rows, shape.cols),
                cells,
                Some("treecalc_node_array".to_string()),
            )
        }
        _ => ResolvedReferenceValues::new(
            ResolvedReferenceExtent::new(1, 1),
            vec![ResolvedReferenceCell::new(1, 1, value)],
            Some("treecalc_node_scalar".to_string()),
        ),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TreeCalcRuntimeReferenceTextResolution {
    pub owner_node_id: TreeNodeId,
    pub target_node_id: TreeNodeId,
    pub reference_text: String,
    pub mode: ReferenceTextResolutionMode,
    pub a1_style: Option<bool>,
    pub reference_like: ReferenceLike,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeCalcCollectionReferenceDescriptor {
    pub host_ref_handle: String,
    pub family: TreeReferenceCollectionFamily,
    pub base_node_id: TreeNodeId,
    pub source_span_utf8: Option<(usize, usize)>,
    pub source_token_text: String,
    pub opaque_selector: String,
    pub member_node_ids: Vec<TreeNodeId>,
    pub membership_version: String,
    pub order_version: String,
}

impl TreeCalcCollectionReferenceDescriptor {
    #[must_use]
    pub fn descriptor_identity(&self) -> TreeCalcReferenceDescriptorIdentity {
        TreeCalcReferenceDescriptorIdentity::from_host_ref_handle(&self.host_ref_handle)
    }

    #[must_use]
    pub fn reference_like(&self) -> ReferenceLike {
        treecalc_collection_reference_like(&self.host_ref_handle)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TreeCalcReferenceDescriptorIdentity(String);

impl TreeCalcReferenceDescriptorIdentity {
    #[must_use]
    pub fn from_host_ref_handle(host_ref_handle: impl Into<String>) -> Self {
        Self(host_ref_handle.into())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

struct TreeCalcResolvedReferenceValues {
    reference: ReferenceLike,
    values: ResolvedReferenceValues,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TreeCalcSparseReferenceCell {
    pub row: usize,
    pub col: usize,
    pub value: CalcValue,
}

impl TreeCalcSparseReferenceCell {
    #[must_use]
    pub fn new(row: usize, col: usize, value: CalcValue) -> Self {
        Self { row, col, value }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TreeCalcSparseReferenceValuesBinding {
    pub reference: ReferenceLike,
    pub declared_rows: usize,
    pub declared_cols: usize,
    pub defined_cells: Vec<TreeCalcSparseReferenceCell>,
    pub reader_identity: Option<String>,
}

impl TreeCalcSparseReferenceValuesBinding {
    #[must_use]
    pub fn resolved_values(&self) -> ResolvedReferenceValues {
        ResolvedReferenceValues::new(
            ResolvedReferenceExtent::new(self.declared_rows, self.declared_cols),
            self.defined_cells
                .iter()
                .map(|cell| ResolvedReferenceCell::new(cell.row, cell.col, cell.value.clone()))
                .collect(),
            self.reader_identity.clone(),
        )
    }
}

impl TreeCalcReferenceSystemProvider<'_> {
    fn values_from_collection_descriptor(
        &self,
        reference: &ReferenceLike,
    ) -> Result<Option<ResolvedReferenceValues>, ReferenceResolutionError> {
        let Some(handle) = treecalc_handle_text(reference) else {
            return Ok(None);
        };
        let descriptor_identity = TreeCalcReferenceDescriptorIdentity::from_host_ref_handle(handle);
        let Some(descriptor) = self.collection_descriptors.get(&descriptor_identity) else {
            return Ok(None);
        };
        let Some(structural_snapshot) = self.structural_snapshot else {
            return Ok(None);
        };
        let Some(published_calc_values) = self.published_calc_values else {
            return Ok(None);
        };

        let values = match descriptor.family {
            TreeReferenceCollectionFamily::ChildrenV1 => {
                let reader = TreeCalcChildrenSparseReader::from_published_calc_values(
                    structural_snapshot,
                    descriptor.children_collection(),
                    published_calc_values,
                )
                .map_err(|error| ReferenceResolutionError::ProviderFailure {
                    detail: format!(
                        "failed to reconstruct TreeCalc children reference '{}': {error}",
                        descriptor.host_ref_handle
                    ),
                })?;
                resolved_values_from_sparse_reader(&reader)
            }
            TreeReferenceCollectionFamily::ReferenceLiteralArrayV1 => {
                let reader = TreeCalcReferenceLiteralArraySparseReader::from_published_calc_values(
                    structural_snapshot,
                    descriptor.reference_literal_array_collection()?,
                    published_calc_values,
                )
                .map_err(|error| ReferenceResolutionError::ProviderFailure {
                    detail: format!(
                        "failed to reconstruct TreeCalc reference-literal array '{}': {error}",
                        descriptor.host_ref_handle
                    ),
                })?;
                resolved_values_from_sparse_reader(&reader)
            }
            TreeReferenceCollectionFamily::SiblingSetV1
            | TreeReferenceCollectionFamily::PrecedingV1
            | TreeReferenceCollectionFamily::FollowingV1
            | TreeReferenceCollectionFamily::AncestorsV1
            | TreeReferenceCollectionFamily::RecursiveDescendantsV1 => {
                let reader = TreeCalcOrderedSelectorSparseReader::from_published_calc_values(
                    structural_snapshot,
                    descriptor.ordered_selector_collection()?,
                    published_calc_values,
                )
                .map_err(|error| ReferenceResolutionError::ProviderFailure {
                    detail: format!(
                        "failed to reconstruct TreeCalc ordered selector reference '{}': {error}",
                        descriptor.host_ref_handle
                    ),
                })?;
                resolved_values_from_sparse_reader(&reader)
            }
        };

        Ok(Some(values))
    }
}

impl TreeCalcCollectionReferenceDescriptor {
    fn children_collection(&self) -> TreeCalcChildrenReferenceCollection {
        TreeCalcChildrenReferenceCollection {
            host_ref_handle: self.host_ref_handle.clone(),
            base_node_id: self.base_node_id,
            source_span_utf8: self.source_span_utf8,
            source_token_text: self.source_token_text.clone(),
            opaque_selector: self.opaque_selector.clone(),
            membership_version: self.membership_version.clone(),
            order_version: self.order_version.clone(),
        }
    }

    fn reference_literal_array_collection(
        &self,
    ) -> Result<TreeCalcReferenceLiteralArrayCollection, ReferenceResolutionError> {
        let carrier_id = self
            .host_ref_handle
            .strip_prefix("treecalc-hostref:v1:reference_literal_array:")
            .unwrap_or(&self.host_ref_handle);
        let elements = self
            .member_node_ids
            .iter()
            .copied()
            .map(TreeCalcReferenceLiteralArrayElement::ReferenceNode);
        let mut collection = TreeCalcReferenceLiteralArrayCollection::reference_only_with_handle(
            carrier_id,
            self.host_ref_handle.clone(),
            self.base_node_id,
            self.source_token_text.clone(),
            elements,
        )
        .map_err(|error| ReferenceResolutionError::ProviderFailure {
            detail: format!(
                "failed to reconstruct TreeCalc reference-literal descriptor '{}': {error}",
                self.host_ref_handle
            ),
        })?;
        if let Some((start, end)) = self.source_span_utf8 {
            collection = collection.with_source_span_utf8(start, end);
        }
        Ok(collection)
    }

    fn ordered_selector_collection(
        &self,
    ) -> Result<TreeCalcOrderedSelectorReferenceCollection, ReferenceResolutionError> {
        let Some(family) = ordered_selector_family_from_dependency(self.family) else {
            return Err(ReferenceResolutionError::ProviderFailure {
                detail: format!(
                    "TreeCalc collection '{}' is not an ordered selector",
                    self.host_ref_handle
                ),
            });
        };
        Ok(TreeCalcOrderedSelectorReferenceCollection {
            family,
            host_ref_handle: self.host_ref_handle.clone(),
            base_node_id: self.base_node_id,
            member_node_ids: self.member_node_ids.clone(),
            source_span_utf8: self.source_span_utf8,
            source_token_text: self.source_token_text.clone(),
            opaque_selector: self.opaque_selector.clone(),
            membership_version: self.membership_version.clone(),
            order_version: self.order_version.clone(),
        })
    }
}

fn ordered_selector_family_from_dependency(
    family: TreeReferenceCollectionFamily,
) -> Option<TreeCalcOrderedSelectorFamily> {
    match family {
        TreeReferenceCollectionFamily::ChildrenV1
        | TreeReferenceCollectionFamily::ReferenceLiteralArrayV1 => None,
        TreeReferenceCollectionFamily::SiblingSetV1 => {
            Some(TreeCalcOrderedSelectorFamily::SiblingSetV1)
        }
        TreeReferenceCollectionFamily::PrecedingV1 => {
            Some(TreeCalcOrderedSelectorFamily::PrecedingV1)
        }
        TreeReferenceCollectionFamily::FollowingV1 => {
            Some(TreeCalcOrderedSelectorFamily::FollowingV1)
        }
        TreeReferenceCollectionFamily::AncestorsV1 => {
            Some(TreeCalcOrderedSelectorFamily::AncestorsV1)
        }
        TreeReferenceCollectionFamily::RecursiveDescendantsV1 => {
            Some(TreeCalcOrderedSelectorFamily::RecursiveDescendantsV1)
        }
    }
}

fn references_match(left: &ReferenceLike, right: &ReferenceLike) -> bool {
    left.system == right.system && left.identity == right.identity
}

fn treecalc_node_id_from_reference(reference: &ReferenceLike) -> Option<TreeNodeId> {
    treecalc_handle_text(reference)
        .or_else(|| {
            reference
                .target()
                .strip_prefix("treecalc.node:")
                .map(str::to_string)
        })
        .and_then(|handle| {
            handle
                .strip_prefix("treecalc.node:")
                .and_then(|id| id.parse::<u64>().ok())
                .map(TreeNodeId)
        })
}

fn treecalc_handle_text(reference: &ReferenceLike) -> Option<String> {
    match &reference.identity {
        oxfunc_core::value::ReferenceIdentity::Opaque(handle) => {
            String::from_utf8(handle.id.bytes.clone()).ok()
        }
        oxfunc_core::value::ReferenceIdentity::Textual(textual) => {
            Some(textual.text.to_string_lossy())
        }
        oxfunc_core::value::ReferenceIdentity::Composite(_) => None,
    }
}

fn resolved_values_from_sparse_reader(reader: &impl SparseRangeReader) -> ResolvedReferenceValues {
    let extent = reader.declared_extent();
    let identity = reader.reader_identity();
    ResolvedReferenceValues::new(
        ResolvedReferenceExtent::new(
            usize::try_from(extent.row_count).unwrap_or(usize::MAX),
            usize::try_from(extent.column_count).unwrap_or(usize::MAX),
        ),
        reader
            .defined_iter()
            .map(|cell| {
                ResolvedReferenceCell::new(
                    sparse_coord_to_resolved_index(cell.coord.row, extent.start.row),
                    sparse_coord_to_resolved_index(cell.coord.column, extent.start.column),
                    cell.value,
                )
            })
            .collect(),
        Some(format!(
            "reader_id={};source={};snapshot={}",
            identity.reader_id, identity.source_identity, identity.snapshot_identity
        )),
    )
}

fn sparse_coord_to_resolved_index(coord: u32, start: u32) -> usize {
    coord
        .checked_sub(start)
        .and_then(|offset| offset.checked_add(1))
        .and_then(|index| usize::try_from(index).ok())
        .unwrap_or(usize::MAX)
}

fn dereference_resolved_reference_values(
    values: &ResolvedReferenceValues,
) -> Result<CalcValue, ReferenceResolutionError> {
    if values.declared_extent.rows == 1 && values.declared_extent.cols == 1 {
        let cell = values
            .defined_cells
            .iter()
            .find(|cell| cell.row == 1 && cell.col == 1)
            .map(|cell| cell.value.clone())
            .unwrap_or_else(CalcValue::empty);
        return Ok(cell);
    }

    materialize_resolved_reference_values(values).map(CalcValue::array)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sparse_reader::{
        SparseCellCoord, SparseRangeExtent, SparseReaderIdentity, WorksheetSparseRangeReader,
    };
    use crate::structural::{
        StructuralNode, StructuralNodeKind, StructuralSnapshot, StructuralSnapshotId,
    };
    use oxfml_core::binding::NameKind;
    use oxfml_core::consumer::editor::{EditorAnalysisStage, EditorEditService, EditorEnvironment};
    use oxfml_core::{
        BindContext, BindRequest, BoundFormula, CompileSemanticPlanRequest, EvaluationContext,
        FormulaSourceRecord, NormalizedReference, ParseRequest, PlacedFormulaIdentity,
        StructureContextVersion, bind_formula, compile_semantic_plan, evaluate_formula,
        parse_formula, project_red_view,
    };
    use oxfunc_core::resolver::{
        ReferenceComposeOperation, ReferenceComposeRequest, ReferenceTransformKind,
        ReferenceTransformRequest,
    };
    use oxfunc_core::value::CalcValue;

    fn snapshot() -> StructuralSnapshot {
        StructuralSnapshot::create(
            StructuralSnapshotId(1),
            TreeNodeId(1),
            vec![
                StructuralNode {
                    node_id: TreeNodeId(1),
                    parent_id: None,
                    symbol: "Root".to_string(),
                    kind: StructuralNodeKind::Root,
                    child_ids: vec![TreeNodeId(2)],
                    role: None,
                    is_meta: false,
                },
                StructuralNode {
                    node_id: TreeNodeId(2),
                    parent_id: Some(TreeNodeId(1)),
                    symbol: "A".to_string(),
                    kind: StructuralNodeKind::Calculation,
                    child_ids: Vec::new(),
                    role: None,
                    is_meta: false,
                },
            ],
        )
        .expect("test snapshot should be valid")
    }

    #[test]
    fn treecalc_provider_dereferences_node_reference() {
        let snapshot = snapshot();
        let meta = BTreeSet::new();
        let values = BTreeMap::from([(TreeNodeId(2), CalcValue::number(42.0))]);
        let provider =
            TreeCalcReferenceSystemProvider::new(&snapshot, &meta, TreeNodeId(1), &values);

        let result = provider
            .dereference(&ReferenceDereferenceRequest {
                reference: treecalc_node_reference_like(TreeNodeId(2)),
            })
            .expect("node reference should dereference");

        assert_eq!(result, CalcValue::number(42.0));
    }

    #[test]
    fn treecalc_bind_profile_binds_opaque_node_atom() {
        let bound = bind_treecalc_profile_formula("treecalc-profile-node", "=TCREF_NODE_2", 1);

        assert_eq!(bound.normalized_references.len(), 1);
        let record = treecalc_profile_record(&bound.normalized_references[0]);
        assert_eq!(record.profile_id, TREECALC_REFERENCE_SYSTEM_ID);
        assert_eq!(record.normal_form_key.0, "treecalc.node:2");
        assert_eq!(
            record.source_info.address_fidelity.as_deref(),
            Some("TCREF_NODE_2")
        );
        match decode_treecalc_reference_payload(&record.profile_payload)
            .expect("treecalc profile payload")
        {
            TreeCalcProfileReference::Node {
                node_id, handle, ..
            } => {
                assert_eq!(node_id, 2);
                assert_eq!(handle, "treecalc.node:2");
            }
            other => panic!("expected node payload, got {other:?}"),
        }

        let reference_like = treecalc_reference_like_from_profile_record(record)
            .expect("profile record should lower to TreeCalc ReferenceLike");
        assert!(references_match(
            &reference_like,
            &treecalc_node_reference_like(TreeNodeId(2))
        ));
    }

    #[test]
    fn treecalc_profile_surfaces_editor_reference_info_through_oxfml_profile_seam() {
        let source = FormulaSourceRecord::new("treecalc-profile-editor-info", 1, "=TCREF_NODE_2");
        let service = EditorEditService::new(
            EditorEnvironment::new(treecalc_profile_bind_context(1))
                .with_reference_bind_profile(treecalc_reference_bind_profile()),
        );

        let opened = service.apply_edit(source, None, EditorAnalysisStage::SyntaxAndBind, None);
        let info = service
            .reference_info_at_cursor(&opened.document, 6, None)
            .expect("TreeCalc profile reference should be visible to editor info");

        assert_eq!(
            info.source_span,
            oxfml_core::syntax::token::TextSpan::new(1, 12)
        );
        assert_eq!(info.source_text, "TCREF_NODE_2");
        assert_eq!(info.profile_record.profile_id, TREECALC_REFERENCE_SYSTEM_ID);
        assert_eq!(
            info.profile_record.render_hint.as_deref(),
            Some("TCREF_NODE_2")
        );
        assert_eq!(info.rendered_text.as_deref(), Some("TCREF_NODE_2"));
        assert!(info.diagnostics.is_empty());
        match decode_treecalc_reference_payload(&info.profile_record.profile_payload)
            .expect("TreeCalc editor info should carry profile payload")
        {
            TreeCalcProfileReference::Node {
                node_id, handle, ..
            } => {
                assert_eq!(node_id, 2);
                assert_eq!(handle, "treecalc.node:2");
            }
            other => panic!("expected TreeCalc node payload, got {other:?}"),
        }
    }

    #[test]
    fn treecalc_bind_profile_template_identity_excludes_caller_anchor() {
        let first = bind_treecalc_profile_formula("treecalc-profile-shared", "=TCREF_NODE_2", 1);
        let second = bind_treecalc_profile_formula("treecalc-profile-shared", "=TCREF_NODE_2", 99);

        assert_eq!(
            first.formula_template_identity,
            second.formula_template_identity
        );
        assert_ne!(
            first.placed_formula_identity,
            second.placed_formula_identity
        );
        assert_ne!(
            second.placed_formula_identity,
            PlacedFormulaIdentity { key: String::new() }
        );
    }

    #[test]
    fn treecalc_profile_record_lowering_rejects_payload_key_mismatch() {
        let bound = bind_treecalc_profile_formula("treecalc-profile-mismatch", "=TCREF_NODE_2", 1);
        let mut record = treecalc_profile_record(&bound.normalized_references[0]).clone();
        record.normal_form_key = ReferenceNormalFormKey("treecalc.node:3".to_string());

        assert_eq!(treecalc_reference_like_from_profile_record(&record), None);
    }

    #[test]
    fn treecalc_profile_symbolic_reference_evaluates_through_tree_provider() {
        let bound = bind_treecalc_profile_formula("treecalc-profile-provider", "=TCREF_NODE_2", 1);
        let semantic_plan = compile_semantic_plan(CompileSemanticPlanRequest {
            bound_formula: bound.clone(),
            oxfunc_catalog_identity: "oxfunc:test".to_string(),
            locale_profile: Some("en-US".to_string()),
            date_system: Some("1900".to_string()),
            format_profile: Some("excel-default".to_string()),
            library_context_snapshot: None,
        })
        .semantic_plan;
        let snapshot = snapshot();
        let meta = BTreeSet::new();
        let values = BTreeMap::from([(TreeNodeId(2), CalcValue::number(42.0))]);
        let provider =
            TreeCalcReferenceSystemProvider::new(&snapshot, &meta, TreeNodeId(1), &values);
        let mut context = EvaluationContext::new(&bound, &semantic_plan);
        context.reference_system_provider = Some(&provider);

        let output = evaluate_formula(context).expect("TreeCalc profile reference should evaluate");

        assert_eq!(output.oxfunc_value, CalcValue::number(42.0));
    }

    #[test]
    fn treecalc_bind_profile_does_not_reopen_legacy_host_name_path() {
        let mut context = treecalc_profile_bind_context(1);
        context
            .names
            .insert("A".to_string(), NameKind::ReferenceLike);
        let bound =
            bind_treecalc_profile_formula_with_context("treecalc-profile-host-name", "=A", context);

        assert!(bound.normalized_references.is_empty());
        assert!(
            bound
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message.contains("unresolved identifier 'A'"))
        );
    }

    #[test]
    fn treecalc_provider_enumerates_sparse_reference_values_by_identity() {
        let snapshot = snapshot();
        let meta = BTreeSet::new();
        let values = BTreeMap::new();
        let reference = treecalc_collection_reference_like("treecalc-hostref:v1:test");
        let reader = WorksheetSparseRangeReader::new(
            SparseReaderIdentity::new("reader:test", "source:test", "snapshot:test"),
            SparseRangeExtent::new(SparseCellCoord::new(0, 0), 1, 2),
            [
                (SparseCellCoord::new(0, 0), CalcValue::number(1.0)),
                (SparseCellCoord::new(0, 1), CalcValue::number(2.0)),
            ],
        )
        .expect("reader should be valid");
        let provider =
            TreeCalcReferenceSystemProvider::new(&snapshot, &meta, TreeNodeId(1), &values)
                .with_sparse_reader(reference.clone(), &reader);

        let result = provider
            .enumerate_values(&ReferenceEnumerationRequest { reference })
            .expect("enumeration should succeed")
            .expect("reference values should be present");

        assert_eq!(result.declared_extent, ResolvedReferenceExtent::new(1, 2));
        assert_eq!(result.defined_cardinality, 2);
    }

    #[test]
    fn treecalc_provider_dereferences_sparse_reference_values_by_identity() {
        let snapshot = snapshot();
        let meta = BTreeSet::new();
        let values = BTreeMap::new();
        let reference = treecalc_collection_reference_like("treecalc-hostref:v1:test");
        let reader = WorksheetSparseRangeReader::new(
            SparseReaderIdentity::new("reader:test", "source:test", "snapshot:test"),
            SparseRangeExtent::new(SparseCellCoord::new(0, 0), 1, 1),
            [(SparseCellCoord::new(0, 0), CalcValue::number(3.0))],
        )
        .expect("reader should be valid");
        let provider =
            TreeCalcReferenceSystemProvider::new(&snapshot, &meta, TreeNodeId(1), &values)
                .with_sparse_reader(reference.clone(), &reader);

        let result = provider
            .dereference(&ReferenceDereferenceRequest { reference })
            .expect("sparse reference should dereference");

        assert_eq!(result, CalcValue::number(3.0));
    }

    #[test]
    fn treecalc_provider_enumerates_collection_from_shared_descriptor() {
        let snapshot = snapshot();
        let meta = BTreeSet::new();
        let values = BTreeMap::from([(TreeNodeId(2), CalcValue::number(42.0))]);
        let handle = "treecalc-hostref:v1:children:1".to_string();
        let reference = treecalc_collection_reference_like(&handle);
        let provider =
            TreeCalcReferenceSystemProvider::new(&snapshot, &meta, TreeNodeId(1), &values)
                .with_collection_descriptor(TreeCalcCollectionReferenceDescriptor {
                    host_ref_handle: handle,
                    family: TreeReferenceCollectionFamily::ChildrenV1,
                    base_node_id: TreeNodeId(1),
                    source_span_utf8: None,
                    source_token_text: "A.@CHILDREN".to_string(),
                    opaque_selector: "oxcalc.treecalc.host_selector.v1:selector=Children;base=1"
                        .to_string(),
                    member_node_ids: vec![TreeNodeId(2)],
                    membership_version: "treecalc-membership:v1:base=1;members=2".to_string(),
                    order_version: "treecalc-order:v1:base=1;members=2".to_string(),
                });

        let result = provider
            .enumerate_values(&ReferenceEnumerationRequest { reference })
            .expect("descriptor-backed enumeration should succeed")
            .expect("descriptor should reconstruct sparse values");

        assert_eq!(result.declared_extent, ResolvedReferenceExtent::new(1, 1));
        assert_eq!(result.defined_cardinality, 1);
        assert_eq!(
            result.defined_cells,
            vec![ResolvedReferenceCell::new(1, 1, CalcValue::number(42.0))]
        );
    }

    #[test]
    fn treecalc_provider_dereferences_collection_from_shared_descriptor() {
        let snapshot = snapshot();
        let meta = BTreeSet::new();
        let values = BTreeMap::from([(TreeNodeId(2), CalcValue::number(42.0))]);
        let handle = "treecalc-hostref:v1:children:1".to_string();
        let reference = treecalc_collection_reference_like(&handle);
        let provider =
            TreeCalcReferenceSystemProvider::new(&snapshot, &meta, TreeNodeId(1), &values)
                .with_collection_descriptor(TreeCalcCollectionReferenceDescriptor {
                    host_ref_handle: handle,
                    family: TreeReferenceCollectionFamily::ChildrenV1,
                    base_node_id: TreeNodeId(1),
                    source_span_utf8: None,
                    source_token_text: "A.@CHILDREN".to_string(),
                    opaque_selector: "oxcalc.treecalc.host_selector.v1:selector=Children;base=1"
                        .to_string(),
                    member_node_ids: vec![TreeNodeId(2)],
                    membership_version: "treecalc-membership:v1:base=1;members=2".to_string(),
                    order_version: "treecalc-order:v1:base=1;members=2".to_string(),
                });

        let result = provider
            .dereference(&ReferenceDereferenceRequest { reference })
            .expect("descriptor-backed reference should dereference");

        assert_eq!(result, CalcValue::number(42.0));
    }

    #[test]
    fn treecalc_provider_interns_collection_descriptors_by_handle() {
        let snapshot = snapshot();
        let meta = BTreeSet::new();
        let values = BTreeMap::from([(TreeNodeId(2), CalcValue::number(42.0))]);
        let handle = "treecalc-hostref:v1:children:1".to_string();
        let descriptor = TreeCalcCollectionReferenceDescriptor {
            host_ref_handle: handle.clone(),
            family: TreeReferenceCollectionFamily::ChildrenV1,
            base_node_id: TreeNodeId(1),
            source_span_utf8: None,
            source_token_text: "A.@CHILDREN".to_string(),
            opaque_selector: "oxcalc.treecalc.host_selector.v1:selector=Children;base=1"
                .to_string(),
            member_node_ids: vec![TreeNodeId(2)],
            membership_version: "treecalc-membership:v1:base=1;members=2".to_string(),
            order_version: "treecalc-order:v1:base=1;members=2".to_string(),
        };
        let provider =
            TreeCalcReferenceSystemProvider::new(&snapshot, &meta, TreeNodeId(1), &values)
                .with_collection_descriptor(descriptor.clone())
                .with_collection_descriptor(descriptor);

        assert_eq!(provider.collection_descriptor_count(), 1);

        let result = provider
            .enumerate_values(&ReferenceEnumerationRequest {
                reference: treecalc_collection_reference_like(handle),
            })
            .expect("descriptor-backed enumeration should succeed")
            .expect("interned descriptor should reconstruct sparse values");

        assert_eq!(result.defined_cardinality, 1);
    }

    #[test]
    fn treecalc_provider_resolves_runtime_text_through_host_names() {
        let snapshot = snapshot();
        let meta = BTreeSet::new();
        let values = BTreeMap::new();
        let provider =
            TreeCalcReferenceSystemProvider::new(&snapshot, &meta, TreeNodeId(1), &values);

        let reference = provider
            .resolve_text(&ReferenceTextResolveRequest {
                text: "A".to_string(),
                mode: ReferenceTextResolutionMode::Indirect,
                a1_style: Some(true),
                caller_context: None,
            })
            .expect("A should resolve as a TreeCalc host reference");

        assert!(references_match(
            &reference,
            &treecalc_node_reference_like(TreeNodeId(2))
        ));
        assert_eq!(provider.runtime_text_resolutions().len(), 1);
    }

    #[test]
    fn treecalc_provider_keeps_transform_and_compose_as_typed_unsupported_requests() {
        let snapshot = snapshot();
        let meta = BTreeSet::new();
        let values = BTreeMap::new();
        let provider =
            TreeCalcReferenceSystemProvider::new(&snapshot, &meta, TreeNodeId(1), &values);
        let reference = treecalc_node_reference_like(TreeNodeId(2));

        assert_eq!(
            provider.transform_reference(&ReferenceTransformRequest {
                reference: reference.clone(),
                transform: ReferenceTransformKind::Offset {
                    row_offset: 1,
                    col_offset: 0,
                    height: None,
                    width: None,
                },
            }),
            Err(ReferenceSystemError::Unsupported {
                operation: ReferenceSystemOperation::Transform,
            })
        );
        assert_eq!(
            provider.compose_references(&ReferenceComposeRequest {
                lhs: reference.clone(),
                rhs: reference,
                operation: ReferenceComposeOperation::Range,
            }),
            Err(ReferenceSystemError::Unsupported {
                operation: ReferenceSystemOperation::Compose,
            })
        );
    }

    fn bind_treecalc_profile_formula(
        stable_id: &str,
        formula_text: &str,
        caller_row: u32,
    ) -> BoundFormula {
        bind_treecalc_profile_formula_with_context(
            stable_id,
            formula_text,
            treecalc_profile_bind_context(caller_row),
        )
    }

    fn bind_treecalc_profile_formula_with_context(
        stable_id: &str,
        formula_text: &str,
        context: BindContext,
    ) -> BoundFormula {
        let source = FormulaSourceRecord::new(stable_id, 1, formula_text);
        let parse = parse_formula(ParseRequest {
            source: source.clone(),
        });
        let red_projection = project_red_view(source.formula_stable_id.clone(), &parse.green_tree);
        bind_formula(BindRequest {
            source,
            green_tree: parse.green_tree,
            red_projection,
            context,
            reference_bind_profile: Some(treecalc_reference_bind_profile()),
        })
        .bound_formula
    }

    fn treecalc_profile_bind_context(caller_row: u32) -> BindContext {
        BindContext {
            caller_row,
            structure_context_version: StructureContextVersion("treecalc-struct:test".to_string()),
            ..BindContext::default()
        }
    }

    fn treecalc_profile_record(normalized: &NormalizedReference) -> &ProfileReferenceRecord {
        match normalized {
            NormalizedReference::ProfileSymbolic(record) => record,
            other => panic!("expected TreeCalc profile symbolic reference, got {other:?}"),
        }
    }

    // =====================================================================
    // W062 R3.6 — Tree-profile precedence corpus (D2 §7, Harvest 1) + the
    // cross-workspace `!` seat (D2 §9). These assert against the REAL profile
    // `bind_name` `Rejected`/`Bound` surfaces — no invented diagnostics — and
    // build ambiguity WITHOUT case-only sibling twins (which `calc-uanv` will
    // forbid); ambiguity here comes from two distinct-cased siblings whose
    // ASCII-folded symbols collide, which is the resolver's actual collision
    // space (`children_by_symbol` keys on the uppercased symbol).
    // =====================================================================

    use crate::reference_vocabulary::ContainerRole;
    use crate::tree_reference_resolution::{
        TREECALC_NAME_AMBIGUOUS_CODE, TREECALC_WORKSPACE_UNKNOWN_CODE,
    };

    fn tree_node(
        node_id: u64,
        symbol: &str,
        parent: Option<u64>,
        kind: StructuralNodeKind,
        children: Vec<u64>,
    ) -> StructuralNode {
        StructuralNode {
            node_id: TreeNodeId(node_id),
            parent_id: parent.map(TreeNodeId),
            symbol: symbol.to_string(),
            kind,
            child_ids: children.into_iter().map(TreeNodeId).collect(),
            role: None,
            is_meta: false,
        }
    }

    fn name_bind_request(
        source_text: &str,
        parsed_qualifier: Option<&str>,
    ) -> ReferenceNameBindRequest {
        ReferenceNameBindRequest {
            source_channel: oxfml_core::FormulaChannelKind::WorksheetA1,
            source_span: oxfml_core::syntax::token::TextSpan::new(1, source_text.len()),
            source_text: source_text.to_string(),
            parsed_qualifier: parsed_qualifier.map(str::to_string),
            workbook_id: "book:default".to_string(),
            sheet_id: "sheet:default".to_string(),
            caller_row: 1,
            caller_col: 1,
        }
    }

    /// The precedence fixture for rules 1, 2, 4 (nearest-scope-wins,
    /// ancestor-own-name-no-priority, self-reference). Rule 3 (within-scope
    /// ambiguity) uses [`ambiguity_snapshot`] instead — see the ground-truth
    /// note there for why the ambiguity fixture is separated.
    /// ```text
    /// Root
    /// ├─ Outer (2)
    /// │   ├─ Widget (4)          <- nearer-scope Widget
    /// │   └─ Inner (5)
    /// │       └─ (owner lives here: node 7 "Cell")
    /// └─ Widget (3)              <- ancestor-distance Widget (root scope)
    /// ```
    /// `Widget` at Root and at Outer both exist; from owner `Cell` under Inner,
    /// nearest-scope-wins must pick Outer's `Widget` (node 4), NOT Root's.
    fn precedence_snapshot() -> (StructuralSnapshot, BTreeSet<TreeNodeId>) {
        let snapshot = StructuralSnapshot::create(
            StructuralSnapshotId(1),
            TreeNodeId(1),
            vec![
                tree_node(1, "Root", None, StructuralNodeKind::Root, vec![2, 3]),
                tree_node(
                    2,
                    "Outer",
                    Some(1),
                    StructuralNodeKind::Container,
                    vec![4, 5],
                ),
                tree_node(3, "Widget", Some(1), StructuralNodeKind::Constant, vec![]),
                tree_node(4, "Widget", Some(2), StructuralNodeKind::Constant, vec![]),
                tree_node(5, "Inner", Some(2), StructuralNodeKind::Container, vec![7]),
                tree_node(7, "Cell", Some(5), StructuralNodeKind::Calculation, vec![]),
            ],
        )
        .expect("precedence fixture valid");
        (snapshot, BTreeSet::new())
    }

    // GROUND-TRUTH note (per the bead's instruction to ground-truth the
    // resolver first): the walk-up resolver (`resolve_context_walkup_symbol`)
    // yields `Ambiguous` iff a single scope holds more than one visible child
    // matching the symbol case-insensitively (`matching_children` uses
    // `eq_ignore_ascii_case`; the index keys on the uppercased symbol). Two
    // structurally-valid siblings collide in that space ONLY when their symbols
    // differ (distinct projection paths — byte-identical siblings are rejected
    // as `DuplicateProjectionPath` today) yet fold equal. That is the resolver's
    // *sole* within-scope collision space; there is no non-case within-scope
    // collision path. `precedence_rule3_*` therefore builds the collision from
    // fold-equal siblings and is written to be calc-uanv-robust in place (it
    // asserts the resolver rejection pre-calc-uanv and tolerates the structural
    // rejection post-calc-uanv), rather than baking a case-twin into a SHARED
    // fixture that would panic-break the whole module.

    // --- Rule 1: nearest-scope-wins -------------------------------------
    #[test]
    fn precedence_rule1_nearest_scope_wins_over_farther_scope() {
        let (snapshot, meta) = precedence_snapshot();
        let profile = TreeCalcContextReferenceBindProfile::new(&snapshot, &meta, TreeNodeId(7));
        // `Widget` exists at Outer (node 4) and at Root (node 3). From owner Cell
        // (under Inner under Outer), the nearer scope (Outer) wins.
        let result = profile.bind_name(&name_bind_request("Widget", None));
        let ReferenceAtomBindResult::Bound(record) = result else {
            panic!("Widget must bind, got {result:?}");
        };
        assert_eq!(
            record.normal_form_key.0,
            treecalc_node_reference_target(TreeNodeId(4))
        );
    }

    // --- Rule 2: ancestor-by-own-name has NO priority -------------------
    #[test]
    fn precedence_rule2_ancestor_own_name_has_no_priority() {
        // Owner is `Outer` itself (node 2). `Outer` is the owner's OWN name, but
        // that ancestor-by-own-name must not outrank a nearer defined symbol.
        // `Widget` from owner Outer resolves to Outer's child Widget (node 4),
        // never to a self/ancestor match on the name `Outer`.
        let (snapshot, meta) = precedence_snapshot();
        let profile = TreeCalcContextReferenceBindProfile::new(&snapshot, &meta, TreeNodeId(2));
        let result = profile.bind_name(&name_bind_request("Widget", None));
        let ReferenceAtomBindResult::Bound(record) = result else {
            panic!("Widget must bind from Outer, got {result:?}");
        };
        assert_eq!(
            record.normal_form_key.0,
            treecalc_node_reference_target(TreeNodeId(4))
        );
    }

    // --- Rule 3: within-scope collision ⇒ typed Rejected w/ stable code --
    #[test]
    fn precedence_rule3_within_scope_collision_is_typed_ambiguous_rejection() {
        // The resolver's SOLE within-scope collision space is fold-equal
        // siblings (ground-truthed on `ambiguity_snapshot`). This case is
        // deliberately calc-uanv-robust: the value under test is that within
        // scope ambiguity is a TYPED `Rejected` carrying `treecalc.name.ambiguous`
        // and NEVER a silent pick. Today the collision is caught at the resolver;
        // once calc-uanv enforces case-insensitive sibling uniqueness, the SAME
        // collision is caught one layer earlier at `StructuralSnapshot::create`.
        // The test asserts the intent in BOTH regimes so it neither rots nor
        // panic-breaks the module when calc-uanv lands (isolated to this test —
        // rules 1/2/4 and the cross-workspace tests use collision-free fixtures).
        match StructuralSnapshot::create(
            StructuralSnapshotId(1),
            TreeNodeId(1),
            vec![
                tree_node(1, "Root", None, StructuralNodeKind::Root, vec![2, 3]),
                tree_node(2, "Widget", Some(1), StructuralNodeKind::Constant, vec![]),
                tree_node(3, "WIDGET", Some(1), StructuralNodeKind::Constant, vec![]),
            ],
        ) {
            Ok(snapshot) => {
                // Pre-calc-uanv: the collision is a resolver-level typed Rejected.
                let meta = BTreeSet::new();
                let profile =
                    TreeCalcContextReferenceBindProfile::new(&snapshot, &meta, TreeNodeId(2));
                let result = profile.bind_name(&name_bind_request("Widget", None));
                let ReferenceAtomBindResult::Rejected { message, .. } = result else {
                    panic!("within-scope collision must be a typed Rejected, got {result:?}");
                };
                assert!(
                    message.starts_with(TREECALC_NAME_AMBIGUOUS_CODE),
                    "ambiguity must carry the stable code, got {message:?}"
                );
            }
            Err(_structural_rejection) => {
                // Post-calc-uanv: the collision is caught structurally at snapshot
                // construction — within-scope ambiguity is still a typed rejection,
                // just one layer earlier. The invariant (never a silent pick) holds.
            }
        }
    }

    // --- Rule 4: self-reference resolves to self ------------------------
    #[test]
    fn precedence_rule4_self_reference_resolves_to_self() {
        // From owner `Widget` at Root (node 3), referencing `Widget` resolves to
        // self (node 3) — the nearest scope containing `Widget` is Root, and the
        // owner's own scope search finds no nearer one. Cycle handling is
        // downstream; binding to self is the resolution contract.
        let (snapshot, meta) = precedence_snapshot();
        let profile = TreeCalcContextReferenceBindProfile::new(&snapshot, &meta, TreeNodeId(3));
        let result = profile.bind_name(&name_bind_request("Widget", None));
        let ReferenceAtomBindResult::Bound(record) = result else {
            panic!("self-reference Widget must bind, got {result:?}");
        };
        // Root scope holds exactly one `Widget` (node 3) — the owner itself.
        assert_eq!(
            record.normal_form_key.0,
            treecalc_node_reference_target(TreeNodeId(3))
        );
    }

    // --- Cross-workspace `!` -------------------------------------------

    /// A minimal sibling-workspace tree: Root → Branch (2) → Leaf (3).
    fn sibling_workspace_snapshot() -> StructuralSnapshot {
        StructuralSnapshot::create(
            StructuralSnapshotId(2),
            TreeNodeId(1),
            vec![
                tree_node(1, "OtherRoot", None, StructuralNodeKind::Root, vec![2]),
                tree_node(2, "Branch", Some(1), StructuralNodeKind::Container, vec![3]),
                tree_node(3, "Leaf", Some(2), StructuralNodeKind::Calculation, vec![]),
            ],
        )
        .expect("sibling workspace snapshot valid")
    }

    fn cross_workspace_resolver_over<'a>(
        catalog: &'a WorkspaceAliasCatalog,
        sibling: &'a StructuralSnapshot,
        sibling_meta: &'a BTreeSet<TreeNodeId>,
    ) -> TreeCalcCrossWorkspaceResolver<'a> {
        TreeCalcCrossWorkspaceResolver::new(
            catalog,
            [CrossWorkspaceEntry {
                workspace_handle: "workspace:other",
                snapshot: sibling,
                meta_node_ids: sibling_meta,
                availability_version: "avail:v1",
            }],
        )
    }

    /// Acceptance: `Other!Branch.Leaf` from workspace A resolves a node in
    /// workspace B via a registered alias, target-rooted, flowing a cross-
    /// workspace record.
    #[test]
    fn cross_workspace_alias_resolves_target_node_in_sibling_workspace() {
        let (local, local_meta) = precedence_snapshot();
        let sibling = sibling_workspace_snapshot();
        let sibling_meta = BTreeSet::new();
        let mut catalog = WorkspaceAliasCatalog::new();
        catalog.register_alias("Other", "workspace:other");
        let resolver = cross_workspace_resolver_over(&catalog, &sibling, &sibling_meta);
        let profile = TreeCalcContextReferenceBindProfile::new(&local, &local_meta, TreeNodeId(7))
            .with_cross_workspace_resolver(&resolver);

        // Qualifier delivered split (the OxFml binder shape): alias on the left,
        // path on the right.
        let result = profile.bind_name(&name_bind_request("Branch.Leaf", Some("Other")));
        let ReferenceAtomBindResult::Bound(record) = result else {
            panic!("Other!Branch.Leaf must bind, got {result:?}");
        };
        match decode_treecalc_reference_payload(&record.profile_payload).unwrap() {
            TreeCalcProfileReference::CrossWorkspaceNode {
                workspace_handle,
                node_id,
                availability_version,
                ..
            } => {
                assert_eq!(workspace_handle, "workspace:other");
                assert_eq!(node_id, 3, "Leaf is node 3 in the sibling workspace");
                assert_eq!(availability_version, "avail:v1");
            }
            other => panic!("expected a cross-workspace node record, got {other:?}"),
        }
    }

    /// Case-insensitive alias (shared V3 fold) and the inline-`!` token form
    /// both route through the same seat.
    #[test]
    fn cross_workspace_alias_is_case_insensitive_and_inline_form_routes() {
        let (local, local_meta) = precedence_snapshot();
        let sibling = sibling_workspace_snapshot();
        let sibling_meta = BTreeSet::new();
        let mut catalog = WorkspaceAliasCatalog::new();
        catalog.register_alias("Other", "workspace:other");
        let resolver = cross_workspace_resolver_over(&catalog, &sibling, &sibling_meta);
        let profile = TreeCalcContextReferenceBindProfile::new(&local, &local_meta, TreeNodeId(7))
            .with_cross_workspace_resolver(&resolver);

        // Case-insensitive alias, split form.
        assert!(matches!(
            profile.bind_name(&name_bind_request("Branch.Leaf", Some("OTHER"))),
            ReferenceAtomBindResult::Bound(_)
        ));
        // Inline `!` form (no split qualifier) routes identically.
        assert!(matches!(
            profile.bind_name(&name_bind_request("other!Branch.Leaf", None)),
            ReferenceAtomBindResult::Bound(_)
        ));
    }

    /// Target-rooted: NO walk-up past the target root. `Leaf` alone (without its
    /// `Branch` parent) must NOT resolve, proving resolution enters the target
    /// at its root and does not walk up into non-ancestor scopes. (If it walked
    /// arbitrarily it might find `Leaf` as a descendant; rooted resolution
    /// requires the full path from root.)
    #[test]
    fn cross_workspace_resolution_is_target_rooted_no_walkup() {
        let (local, local_meta) = precedence_snapshot();
        let sibling = sibling_workspace_snapshot();
        let sibling_meta = BTreeSet::new();
        let mut catalog = WorkspaceAliasCatalog::new();
        catalog.register_alias("Other", "workspace:other");
        let resolver = cross_workspace_resolver_over(&catalog, &sibling, &sibling_meta);
        let profile = TreeCalcContextReferenceBindProfile::new(&local, &local_meta, TreeNodeId(7))
            .with_cross_workspace_resolver(&resolver);

        // `Leaf` is a grandchild of the sibling root, not a direct child. From
        // the ROOT with no walk-up, the first segment `Leaf` is not a visible
        // child of root, so the path does not resolve -> typed Rejected.
        let result = profile.bind_name(&name_bind_request("Leaf", Some("Other")));
        let ReferenceAtomBindResult::Rejected { message, .. } = result else {
            panic!("target-rooted `Leaf` must not resolve, got {result:?}");
        };
        assert!(
            message.starts_with(TREECALC_WORKSPACE_PATH_UNRESOLVED_CODE),
            "unresolved path must carry the path-unresolved code, got {message:?}"
        );
        // But the full rooted path DOES resolve — proving it is the rooting, not
        // a blanket failure.
        assert!(matches!(
            profile.bind_name(&name_bind_request("Branch.Leaf", Some("Other"))),
            ReferenceAtomBindResult::Bound(_)
        ));
    }

    /// Unknown alias ⇒ typed `treecalc.workspace.unknown` rejection.
    #[test]
    fn cross_workspace_unknown_alias_is_typed_rejection() {
        let (local, local_meta) = precedence_snapshot();
        let sibling = sibling_workspace_snapshot();
        let sibling_meta = BTreeSet::new();
        let catalog = WorkspaceAliasCatalog::new(); // no aliases registered
        let resolver = cross_workspace_resolver_over(&catalog, &sibling, &sibling_meta);
        let profile = TreeCalcContextReferenceBindProfile::new(&local, &local_meta, TreeNodeId(7))
            .with_cross_workspace_resolver(&resolver);

        let result = profile.bind_name(&name_bind_request("Branch.Leaf", Some("Nope")));
        let ReferenceAtomBindResult::Rejected { message, .. } = result else {
            panic!("unknown alias must be typed Rejected, got {result:?}");
        };
        assert!(
            message.starts_with(TREECALC_WORKSPACE_UNKNOWN_CODE),
            "unknown alias must carry the stable code, got {message:?}"
        );
    }

    /// A REGISTERED alias whose workspace is not currently loaded is the
    /// `#REF!`-class **unavailable** outcome — distinct from an unknown alias
    /// (`#NAME?`-class), per D2 §9. It carries the dedicated
    /// `treecalc.workspace.unavailable` code and heals on load.
    #[test]
    fn cross_workspace_registered_but_unloaded_alias_is_typed_unavailable() {
        let (local, local_meta) = precedence_snapshot();
        // The alias IS registered, but no workspace snapshot is reachable for its
        // handle (the resolver holds an empty workspace set).
        let mut catalog = WorkspaceAliasCatalog::new();
        catalog.register_alias("Other", "workspace:other");
        let no_workspaces: [CrossWorkspaceEntry; 0] = [];
        let resolver = TreeCalcCrossWorkspaceResolver::new(&catalog, no_workspaces);
        let profile = TreeCalcContextReferenceBindProfile::new(&local, &local_meta, TreeNodeId(7))
            .with_cross_workspace_resolver(&resolver);

        let result = profile.bind_name(&name_bind_request("Branch.Leaf", Some("Other")));
        let ReferenceAtomBindResult::Rejected { message, .. } = result else {
            panic!("registered-but-unloaded must be typed Rejected, got {result:?}");
        };
        assert!(
            message.starts_with(TREECALC_WORKSPACE_UNAVAILABLE_CODE),
            "unloaded workspace must carry the unavailable code (not unknown), got {message:?}"
        );
    }

    /// With no resolver seat at all, a `!` qualifier is a typed
    /// `treecalc.workspace.unknown` rejection (no catalog to resolve against).
    /// The old "pending" stub outcome is fully retired: the outcome is now a
    /// stable typed code, and the message carries only that code and the alias.
    #[test]
    fn cross_workspace_without_resolver_is_typed_unknown() {
        let (local, local_meta) = precedence_snapshot();
        let profile = TreeCalcContextReferenceBindProfile::new(&local, &local_meta, TreeNodeId(7));
        let result = profile.bind_name(&name_bind_request("Branch.Leaf", Some("Other")));
        let ReferenceAtomBindResult::Rejected { message, validity } = result else {
            panic!("no-resolver `!` must be typed Rejected, got {result:?}");
        };
        assert!(message.starts_with(TREECALC_WORKSPACE_UNKNOWN_CODE));
        // The rejection is host-sensitive (workspace availability), not the old
        // `Unsupported` "pending" validity.
        assert_eq!(validity, ReferenceValidity::DynamicOrHostSensitive);
    }

    // --- Vocabulary sanity: the tree profile admits Workspace containers --
    #[test]
    fn tree_profile_vocabulary_admits_workspace_container_role() {
        use crate::reference_vocabulary::OxCalcReferenceProfile;
        let (local, local_meta) = precedence_snapshot();
        let profile = TreeCalcContextReferenceBindProfile::new(&local, &local_meta, TreeNodeId(7));
        assert!(
            profile
                .vocabulary()
                .admits_container_role(ContainerRole::Workspace)
        );
    }
}
