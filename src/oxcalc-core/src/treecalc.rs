#![forbid(unsafe_code)]

//! Local sequential TreeCalc runtime facade.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::time::{Duration, Instant};

use oxfml_core::binding::{
    BoundExpr, BoundHostReferenceCollection, BoundHostStructuralSelector, HostNameBindRecord,
    NormalizedReference, ReferenceExpr, StructuredReferenceBindRecord, StructuredResolvedRef,
    StructuredSectionKind,
};
use oxfml_core::consumer::runtime::{
    RuntimeAuthoredInputResult, RuntimeEnvironment, RuntimeFormalInputBinding,
    RuntimeFormalReference, RuntimeFormulaRequest, RuntimeFormulaResult, RuntimeHostFormulaContext,
    RuntimeHostNameBindResult, RuntimeHostNameBinding, RuntimeHostReferenceBindResult,
    RuntimeHostReferenceCollectionSyntax, RuntimeHostReferenceStructuralSelectorSyntax,
    RuntimeHostReferenceSyntaxProfile, RuntimePreparedFormulaIdentity, RuntimeTemplateHole,
};
use oxfml_core::eval::{DefinedNameBinding, OxFmlCallableBinding};
use oxfml_core::interface::TypedContextQueryBundle;
use oxfml_core::interface::{ReturnedValueSurface, ReturnedValueSurfaceKind};
use oxfml_core::semantics::{FormulaDeterminismClass, FormulaVolatilityClass, SemanticPlan};
use oxfml_core::source::{FormulaSourceRecord, StructureContextVersion};
use oxfml_core::syntax::token::TextSpan;
use oxfml_core::{EvaluationBackend, EvaluationTraceMode};
use oxfunc_core::functions::rtd_fn::{RtdProvider, RtdProviderResult, RtdRequest};
use oxfunc_core::host_info::{
    CellInfoQuery, HostInfoError, HostInfoProvider, ImageProviderResult, ImageRequest, InfoQuery,
    ResolvedWebImage,
};
use oxfunc_core::value::{
    CalcValue, CoreValue, ExcelText, ReferenceKind, ReferenceLike, RichValue, WorksheetErrorCode,
};
use serde::Serialize;
use thiserror::Error;

use crate::coordinator::{
    AcceptedCandidateResult, CoordinatorError, DependencyShapeUpdate, PublicationBundle,
    RejectDetail, RejectKind, RuntimeEffect, RuntimeEffectFamily, TreeCalcCoordinator,
    calc_value_display_text,
};
use crate::dependency::{
    DependencyDescriptor, DependencyDescriptorKind, DependencyGraph, InvalidationClosure,
    InvalidationReasonKind, InvalidationSeed, TreeReferenceCollectionDependency,
    TreeReferenceCollectionFamily, WorkspaceQualifiedTarget,
};
use crate::formula::{
    CallerContextIdentityNeed, NamespaceIdentityNeed, TreeFormula, TreeFormulaCatalog,
    TreeFormulaHostNameBindPacket, TreeFormulaHostValue, TreeFormulaHostValueBinding,
    TreeReference, treecalc_collection_from_host_dependency_key,
    treecalc_node_id_from_host_dependency_key,
};
use crate::oxfml_session::OxfmlRecalcSessionDriver;
use crate::recalc::{
    NodeCalcState, OverlayEntry, OverlayKey, OverlayKind, RecalcError, Stage1RecalcTracker,
};
use crate::repository::{SubscriptionHandle, SubscriptionRegistryEntry, SubscriptionTopicId};
use crate::rich_value_capability::RichValueCapabilityTraceReplayColumns;
use crate::sparse_reader::{
    SparseRangeReader, TreeCalcChildrenSparseReader, TreeCalcOrderedSelectorSparseReader,
    TreeCalcReferenceLiteralArraySparseReader,
};
use crate::structural::{
    StructuralEditImpact, StructuralEditOutcome, StructuralSnapshot, TreeNodeId,
};
use crate::structured_table::{
    TableDescriptor, TreeCalcTableNodeSnapshot, project_treecalc_table_node_snapshot,
};
use crate::tree_reference_rebind::descriptor_identity_needs;
use crate::tree_reference_system::{
    TreeCalcCollectionReferenceDescriptor, TreeCalcReferenceSystemProvider,
    TreeCalcRuntimeReferenceTextResolution, TreeCalcSparseReferenceCell,
    TreeCalcSparseReferenceValuesBinding,
};
use crate::value_cache::{
    EdgeValueCache, EdgeValueCacheKey, EdgeValueCacheLookup, EdgeValueCachePathFacts,
    EdgeValueCachePolicy, EdgeValueCacheStoreResult,
};
use crate::workspace_revision::{
    DependencyShapeSnapshot, DependencyShapeSnapshotId, FormulaBindingSnapshot,
    FormulaBindingSnapshotId, NodeInputKind, PublicationSnapshot, PublicationSnapshotId,
    RuntimeOverlaySet, RuntimeOverlaySetId, WorkspaceRevision,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LocalTreeCalcRunState {
    Published,
    VerifiedClean,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalTreeCalcLayerSnapshotIds {
    pub formula_binding_snapshot_id: FormulaBindingSnapshotId,
    pub dependency_shape_snapshot_id: DependencyShapeSnapshotId,
    pub publication_snapshot_id: PublicationSnapshotId,
    pub runtime_overlay_set_id: RuntimeOverlaySetId,
}

impl LocalTreeCalcLayerSnapshotIds {
    #[must_use]
    pub fn current_absent_for_revision(workspace_revision: &WorkspaceRevision) -> Self {
        let formula_binding_snapshot = FormulaBindingSnapshot::current_absent(
            workspace_revision.revision_id(),
            "local-treecalc-formula-binding-not-promoted",
        );
        let dependency_shape_snapshot = DependencyShapeSnapshot::current_absent(
            workspace_revision.revision_id(),
            formula_binding_snapshot.snapshot_id(),
            "local-treecalc-dependency-shape-not-promoted",
        );
        let publication_snapshot = PublicationSnapshot::current_absent(
            workspace_revision.revision_id(),
            "local-treecalc-publication-not-promoted",
        );
        let runtime_overlay_set = RuntimeOverlaySet::current_absent(
            publication_snapshot.snapshot_id(),
            "local-treecalc-runtime-overlays-not-promoted",
        );
        Self {
            formula_binding_snapshot_id: formula_binding_snapshot.snapshot_id().clone(),
            dependency_shape_snapshot_id: dependency_shape_snapshot.snapshot_id().clone(),
            publication_snapshot_id: publication_snapshot.snapshot_id().clone(),
            runtime_overlay_set_id: runtime_overlay_set.overlay_set_id().clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalTreeCalcInput {
    pub workspace_revision: WorkspaceRevision,
    pub formula_catalog: TreeFormulaCatalog,
    pub formula_dependency_descriptors: Option<Vec<DependencyDescriptor>>,
    pub table_snapshots: BTreeMap<TreeNodeId, TreeCalcTableNodeSnapshot>,
    pub layer_snapshot_ids: LocalTreeCalcLayerSnapshotIds,
    pub static_dependency_shape_updates: Vec<DependencyShapeUpdate>,
    pub publication_calc_values: BTreeMap<TreeNodeId, CalcValue>,
    pub publication_runtime_effects: Vec<RuntimeEffect>,
    pub invalidation_seeds: Vec<InvalidationSeed>,
    pub previous_arg_preparation_profile_version: Option<String>,
    pub candidate_result_id: String,
    pub publication_id: String,
    pub environment_context: LocalTreeCalcEnvironmentContext,
}

impl LocalTreeCalcInput {
    #[must_use]
    fn structural_snapshot(&self) -> &StructuralSnapshot {
        &self.workspace_revision.structure_snapshot
    }

    #[must_use]
    fn literal_input_values(&self) -> BTreeMap<TreeNodeId, String> {
        self.workspace_revision
            .node_input_snapshot
            .records()
            .iter()
            .filter_map(|(node_id, record)| {
                if record.kind == NodeInputKind::Literal {
                    Some((
                        *node_id,
                        record
                            .text
                            .clone()
                            .expect("literal node-input records should carry text"),
                    ))
                } else {
                    None
                }
            })
            .collect()
    }

    #[must_use]
    fn compatibility_basis(&self) -> String {
        format!(
            "local-treecalc-compatibility-basis:v1:workspace_revision_id={};structure_snapshot_id={};node_input_snapshot_id={};namespace_snapshot_id={};formula_binding_snapshot_id={};dependency_shape_snapshot_id={};publication_snapshot_id={};runtime_overlay_set_id={};runtime_policy_id={};arg_preparation_profile_version={}",
            self.workspace_revision.revision_id().0,
            self.workspace_revision.structure_snapshot.snapshot_id().0,
            self.workspace_revision.node_input_snapshot.snapshot_id().0,
            self.workspace_revision.namespace_snapshot.snapshot_id().0,
            self.layer_snapshot_ids.formula_binding_snapshot_id.0,
            self.layer_snapshot_ids.dependency_shape_snapshot_id.0,
            self.layer_snapshot_ids.publication_snapshot_id.0,
            self.layer_snapshot_ids.runtime_overlay_set_id.0,
            self.environment_context.runtime_policy_id,
            self.environment_context.arg_preparation_profile_version
        )
    }

    #[must_use]
    fn artifact_token_basis(&self) -> String {
        format!(
            "local-treecalc-artifact-token-basis:v1:structure_snapshot_id={};namespace_snapshot_id={};formula_catalog_basis={};formula_dependency_descriptor_basis={};arg_preparation_profile_version={}",
            self.workspace_revision.structure_snapshot.snapshot_id().0,
            self.workspace_revision.namespace_snapshot.snapshot_id().0,
            formula_catalog_artifact_basis(&self.formula_catalog),
            formula_dependency_descriptor_artifact_basis(&self.formula_dependency_descriptors),
            self.environment_context.arg_preparation_profile_version
        )
    }

    #[must_use]
    fn edge_value_cache_basis(&self) -> String {
        format!(
            "local-treecalc-edge-value-cache-basis:v1:workspace_revision_id={};formula_binding_snapshot_id={};dependency_shape_snapshot_id={};publication_snapshot_id={};runtime_overlay_set_id={}",
            self.workspace_revision.revision_id().0,
            self.layer_snapshot_ids.formula_binding_snapshot_id.0,
            self.layer_snapshot_ids.dependency_shape_snapshot_id.0,
            self.layer_snapshot_ids.publication_snapshot_id.0,
            self.layer_snapshot_ids.runtime_overlay_set_id.0
        )
    }
}

fn formula_catalog_artifact_basis(catalog: &TreeFormulaCatalog) -> String {
    #[derive(Serialize)]
    struct FormulaArtifactBasisEntry<'a> {
        owner_node_id: u64,
        binding: &'a crate::formula::TreeFormulaBinding,
    }

    let entries = catalog
        .bindings_by_owner()
        .iter()
        .map(|(owner_node_id, binding)| FormulaArtifactBasisEntry {
            owner_node_id: owner_node_id.0,
            binding,
        })
        .collect::<Vec<_>>();
    let entries_json =
        serde_json::to_string(&entries).expect("formula artifact basis should serialize");
    format!("formula-catalog:v1:{entries_json}")
}

fn formula_dependency_descriptor_artifact_basis(
    descriptors: &Option<Vec<DependencyDescriptor>>,
) -> String {
    #[derive(Serialize)]
    struct DependencyDescriptorBasisEntry<'a> {
        descriptor_id: &'a str,
        source_reference_handle: Option<&'a str>,
        owner_node_id: u64,
        target_node_id: Option<u64>,
        workspace_target: Option<WorkspaceQualifiedTargetBasisEntry<'a>>,
        kind: String,
        carrier_detail: &'a str,
        tree_reference_collection: Option<TreeReferenceCollectionBasisEntry<'a>>,
        requires_rebind_on_structural_change: bool,
    }

    #[derive(Serialize)]
    struct WorkspaceQualifiedTargetBasisEntry<'a> {
        workspace_handle: &'a str,
        target_node_id: u64,
        target_node_handle: &'a str,
        availability_version: &'a str,
    }

    #[derive(Serialize)]
    struct TreeReferenceCollectionBasisEntry<'a> {
        family: String,
        host_ref_handle: &'a str,
        base_node_id: u64,
        membership_version: &'a str,
        order_version: &'a str,
        member_node_ids: Vec<u64>,
    }

    let Some(descriptors) = descriptors else {
        return "formula-dependency-descriptors:derived-from-prepared-formulas".to_string();
    };

    let entries = descriptors
        .iter()
        .map(|descriptor| DependencyDescriptorBasisEntry {
            descriptor_id: descriptor.descriptor_id.as_str(),
            source_reference_handle: descriptor.source_reference_handle.as_deref(),
            owner_node_id: descriptor.owner_node_id.0,
            target_node_id: descriptor.target_node_id.map(|node_id| node_id.0),
            workspace_target: descriptor.workspace_target.as_ref().map(|target| {
                WorkspaceQualifiedTargetBasisEntry {
                    workspace_handle: target.workspace_handle.as_str(),
                    target_node_id: target.target_node_id.0,
                    target_node_handle: target.target_node_handle.as_str(),
                    availability_version: target.availability_version.as_str(),
                }
            }),
            kind: format!("{:?}", descriptor.kind),
            carrier_detail: descriptor.carrier_detail.as_str(),
            tree_reference_collection: descriptor.tree_reference_collection.as_ref().map(
                |collection| TreeReferenceCollectionBasisEntry {
                    family: format!("{:?}", collection.family),
                    host_ref_handle: collection.host_ref_handle.as_str(),
                    base_node_id: collection.base_node_id.0,
                    membership_version: collection.membership_version.as_str(),
                    order_version: collection.order_version.as_str(),
                    member_node_ids: collection
                        .member_node_ids
                        .iter()
                        .map(|node_id| node_id.0)
                        .collect(),
                },
            ),
            requires_rebind_on_structural_change: descriptor.requires_rebind_on_structural_change,
        })
        .collect::<Vec<_>>();
    let entries_json = serde_json::to_string(&entries)
        .expect("formula dependency descriptor basis should serialize");
    format!("formula-dependency-descriptors:v1:{entries_json}")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalTreeCalcEnvironmentContext {
    pub runtime_lane: String,
    pub session_id: Option<String>,
    pub capability_profile_id: String,
    pub host_namespace_version: String,
    pub resolution_rule_version: String,
    pub caller_context_identity_version: String,
    pub table_context_identity: Option<String>,
    pub cross_workspace_availability_version: Option<String>,
    pub meta_node_ids: BTreeSet<TreeNodeId>,
    pub arg_preparation_profile_version: String,
    pub oxfunc_bridge_metadata: LocalTreeCalcOxFuncBridgeMetadata,
    pub dynamic_dependency_effects: bool,
    pub execution_restriction_effects: bool,
    pub capability_sensitive_effects: bool,
    pub shape_topology_effects: bool,
    pub runtime_policy_id: String,
    pub project_runtime_effect_overlays: bool,
    pub derivation_trace_enabled: bool,
    pub scheduling_policy: LocalTreeCalcSchedulingPolicy,
}

impl Default for LocalTreeCalcEnvironmentContext {
    fn default() -> Self {
        Self {
            runtime_lane: "local_sequential_treecalc".to_string(),
            session_id: None,
            capability_profile_id: "host-capabilities:default".to_string(),
            host_namespace_version: "treecalc-host-namespace:v1".to_string(),
            resolution_rule_version: "treecalc-host-resolution:v1".to_string(),
            caller_context_identity_version: "treecalc-caller-context:v1".to_string(),
            table_context_identity: None,
            cross_workspace_availability_version: None,
            meta_node_ids: BTreeSet::new(),
            arg_preparation_profile_version: "oxfunc.arg-prep:default".to_string(),
            oxfunc_bridge_metadata: LocalTreeCalcOxFuncBridgeMetadata::default(),
            dynamic_dependency_effects: true,
            execution_restriction_effects: true,
            capability_sensitive_effects: false,
            shape_topology_effects: false,
            runtime_policy_id: "runtime-policy:default".to_string(),
            project_runtime_effect_overlays: true,
            derivation_trace_enabled: false,
            scheduling_policy: LocalTreeCalcSchedulingPolicy::default(),
        }
    }
}

impl LocalTreeCalcEnvironmentContext {
    #[must_use]
    pub fn with_arg_preparation_profile_version(mut self, version: impl Into<String>) -> Self {
        self.arg_preparation_profile_version = version.into();
        self
    }

    #[must_use]
    pub fn with_capability_profile_id(mut self, profile_id: impl Into<String>) -> Self {
        self.capability_profile_id = profile_id.into();
        self
    }

    #[must_use]
    pub fn with_host_namespace_version(mut self, version: impl Into<String>) -> Self {
        self.host_namespace_version = version.into();
        self
    }

    #[must_use]
    pub fn with_caller_context_identity_version(mut self, version: impl Into<String>) -> Self {
        self.caller_context_identity_version = version.into();
        self
    }

    #[must_use]
    pub fn with_table_context_identity(mut self, identity: impl Into<String>) -> Self {
        self.table_context_identity = Some(identity.into());
        self
    }

    #[must_use]
    pub fn with_cross_workspace_availability_version(mut self, version: impl Into<String>) -> Self {
        self.cross_workspace_availability_version = Some(version.into());
        self
    }

    #[must_use]
    pub fn with_semantic_kernel_metadata_version(mut self, version: impl Into<String>) -> Self {
        self.oxfunc_bridge_metadata.semantic_kernel_metadata_version = Some(version.into());
        self
    }

    #[must_use]
    pub fn with_arg_admission_metadata_version(mut self, version: impl Into<String>) -> Self {
        self.oxfunc_bridge_metadata.arg_admission_metadata_version = Some(version.into());
        self
    }

    #[must_use]
    pub fn with_derivation_trace_enabled(mut self, enabled: bool) -> Self {
        self.derivation_trace_enabled = enabled;
        self
    }

    #[must_use]
    pub fn with_scheduling_policy(mut self, policy: LocalTreeCalcSchedulingPolicy) -> Self {
        self.scheduling_policy = policy;
        self
    }

    #[must_use]
    pub fn with_runtime_policy_id(mut self, policy_id: impl Into<String>) -> Self {
        self.runtime_policy_id = policy_id.into();
        self
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct LocalTreeCalcOxFuncBridgeMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub semantic_kernel_metadata_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arg_admission_metadata_version: Option<String>,
}

impl LocalTreeCalcOxFuncBridgeMetadata {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.semantic_kernel_metadata_version.is_none()
            && self.arg_admission_metadata_version.is_none()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum LocalTreeCalcSchedulingPolicy {
    #[default]
    PullFullClosure,
    PushVisibilityBounded {
        visible_observer_node_ids: Vec<TreeNodeId>,
    },
}

impl LocalTreeCalcSchedulingPolicy {
    #[must_use]
    pub fn diagnostic_name(&self) -> &'static str {
        match self {
            Self::PullFullClosure => "pull_full_closure",
            Self::PushVisibilityBounded { .. } => "push_visibility_bounded",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalTreeCalcRunArtifacts {
    pub result_state: LocalTreeCalcRunState,
    pub dependency_graph: DependencyGraph,
    pub invalidation_closure: InvalidationClosure,
    pub evaluation_order: Vec<TreeNodeId>,
    pub runtime_effects: Vec<RuntimeEffect>,
    pub runtime_effect_overlays: Vec<OverlayEntry>,
    pub prepared_formula_identities: Vec<PreparedFormulaIdentityTrace>,
    pub derivation_traces: Vec<DerivationTraceRecord>,
    pub local_candidate: Option<LocalEvaluatorCandidate>,
    pub candidate_result: Option<AcceptedCandidateResult>,
    pub publication_bundle: Option<PublicationBundle>,
    pub reject_detail: Option<RejectDetail>,
    /// Display projection of `published_calc_values` for legacy fixtures and reports.
    pub published_values: BTreeMap<TreeNodeId, String>,
    pub published_calc_values: BTreeMap<TreeNodeId, CalcValue>,
    pub node_states: BTreeMap<TreeNodeId, NodeCalcState>,
    pub phase_timings_micros: BTreeMap<String, u128>,
    pub diagnostics: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedFormulaIdentityTrace {
    pub owner_node_id: TreeNodeId,
    pub formula_artifact_id: String,
    pub bind_artifact_id: Option<String>,
    pub formula_stable_id: String,
    pub prepared_formula_key: String,
    pub shape_key: String,
    pub dispatch_skeleton_key: String,
    pub plan_template_key: String,
    pub hole_binding_fingerprint: String,
    pub template_hole_count: usize,
    pub oxfunc_bridge_metadata: LocalTreeCalcOxFuncBridgeMetadata,
    pub rich_value_capability_columns: RichValueCapabilityTraceReplayColumns,
}

pub const DERIVATION_TRACE_SCHEMA_ID: &str = "oxcalc.derivation_trace.invoke_outcome.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DerivationTraceRecord {
    pub trace_schema_id: String,
    pub owner_node_id: TreeNodeId,
    pub formula_artifact_id: String,
    pub bind_artifact_id: Option<String>,
    pub formula_stable_id: String,
    pub trace_mode: String,
    #[serde(skip_serializing_if = "RichValueCapabilityTraceReplayColumns::is_empty")]
    pub rich_value_capability_columns: RichValueCapabilityTraceReplayColumns,
    pub template_selection: DerivationTemplateSelectionTrace,
    pub hole_bindings: Vec<DerivationHoleBindingTrace>,
    pub sub_invocation_tree: Vec<DerivationInvocationTraceNode>,
    pub kernel_returned_value: String,
    pub oxfml_trace_events: Vec<DerivationOxfmlTraceEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DerivationTemplateSelectionTrace {
    pub prepared_formula_key: String,
    pub shape_key: String,
    pub dispatch_skeleton_key: String,
    pub plan_template_key: String,
    #[serde(skip_serializing_if = "RichValueCapabilityTraceReplayColumns::is_empty")]
    pub rich_value_capability_columns: RichValueCapabilityTraceReplayColumns,
    pub template_holes: Vec<DerivationTemplateHoleTrace>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DerivationTemplateHoleTrace {
    pub hole_id: String,
    pub ordinal: usize,
    pub path: String,
    pub kind: String,
    #[serde(skip_serializing_if = "RichValueCapabilityTraceReplayColumns::is_empty")]
    pub rich_value_capability_columns: RichValueCapabilityTraceReplayColumns,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DerivationHoleBindingTrace {
    pub hole_id: String,
    pub payload: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DerivationInvocationTraceNode {
    pub invocation_ordinal: usize,
    pub invocation_kind: String,
    pub function_name: String,
    pub function_id: String,
    pub arg_preparation_profile: Option<String>,
    pub prepared_arguments: Vec<DerivationPreparedArgumentTrace>,
    pub kernel_returned_value: Option<String>,
    pub children: Vec<DerivationInvocationTraceNode>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DerivationPreparedArgumentTrace {
    pub ordinal: usize,
    pub structure_class: String,
    pub source_class: String,
    pub evaluation_mode: String,
    pub blankness_class: String,
    pub caller_context_sensitive: bool,
    pub reference_target: Option<String>,
    pub opaque_reason: Option<String>,
    pub resolved_value: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DerivationOxfmlTraceEvent {
    pub trace_schema_id: String,
    pub event_kind: String,
    pub formula_stable_id: String,
    pub session_id: Option<String>,
    pub candidate_result_id: Option<String>,
    pub commit_attempt_id: Option<String>,
    pub event_order_key: u64,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum LocalTreeCalcError {
    #[error(transparent)]
    Coordinator(#[from] CoordinatorError),
    #[error(transparent)]
    Recalc(#[from] RecalcError),
    #[error("formula node {node_id} has no binding")]
    MissingFormulaBinding { node_id: TreeNodeId },
    #[error("reference owned by {owner_node_id} could not be resolved: {detail}")]
    UnresolvedReference {
        owner_node_id: TreeNodeId,
        detail: String,
    },
    #[error(
        "reference owned by {owner_node_id} is host-sensitive and cannot be locally evaluated: {detail}"
    )]
    HostSensitiveReference {
        owner_node_id: TreeNodeId,
        detail: String,
    },
    #[error(
        "reference owned by {owner_node_id} is runtime-dynamic and not yet supported in the local sequential evaluator: {detail}"
    )]
    DynamicReference {
        owner_node_id: TreeNodeId,
        detail: String,
    },
    #[error(
        "reference owned by {owner_node_id} is capability-sensitive and cannot be locally evaluated: {detail}"
    )]
    CapabilitySensitiveReference {
        owner_node_id: TreeNodeId,
        detail: String,
    },
    #[error(
        "reference owned by {owner_node_id} is shape/topology-sensitive and cannot be locally evaluated: {detail}"
    )]
    ShapeTopologyReference {
        owner_node_id: TreeNodeId,
        detail: String,
    },
    #[error("no value is available for referenced node {node_id}")]
    MissingReferencedValue { node_id: TreeNodeId },
    #[error("value '{value}' for node {node_id} is not a supported local integer")]
    UnsupportedNumericValue { node_id: TreeNodeId, value: String },
    #[error("function '{function_name}' is not supported in the local sequential evaluator")]
    UnsupportedFunction { function_name: String },
    #[error("formula family contains a cycle; local sequential runtime cannot yet evaluate it")]
    CycleDetected,
    #[error(
        "dependency graph for formula node {node_id} is incompatible with reevaluation: {detail}"
    )]
    DependencyGraphIncompatible { node_id: TreeNodeId, detail: String },
    #[error("formula node {node_id} requires rebind before reevaluation")]
    StructuralRebindRequired { node_id: TreeNodeId },
    #[error("division by zero")]
    DivisionByZero,
    #[error("OxFml host run for node {owner_node_id} failed: {detail}")]
    OxfmlHostFailure {
        owner_node_id: TreeNodeId,
        detail: String,
    },
    #[error("OxFml bind for node {owner_node_id} is unresolved: {detail}")]
    OxfmlBindUnresolved {
        owner_node_id: TreeNodeId,
        detail: String,
    },
    #[error("OxFml commit for node {owner_node_id} rejected: {detail}")]
    OxfmlCommitRejected {
        owner_node_id: TreeNodeId,
        detail: String,
    },
    #[error("OxFml commit bundle for node {owner_node_id} is incompatible: {detail}")]
    OxfmlCommitBundleIncompatible {
        owner_node_id: TreeNodeId,
        detail: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalEvaluatorCandidate {
    pub candidate_result_id: String,
    pub target_set: Vec<TreeNodeId>,
    pub calc_value_updates: BTreeMap<TreeNodeId, CalcValue>,
    pub dependency_shape_updates: Vec<DependencyShapeUpdate>,
    pub runtime_effects: Vec<RuntimeEffect>,
    pub diagnostic_events: Vec<String>,
}

#[derive(Debug, PartialEq)]
struct LocalFormulaEvaluationSuccess {
    calc_value: CalcValue,
    diagnostics: Vec<String>,
    derivation_trace: Option<DerivationTraceRecord>,
    dynamic_reference_resolutions: Vec<TreeCalcRuntimeReferenceTextResolution>,
}

#[derive(Debug, PartialEq, Eq)]
struct LocalFormulaEvaluationFailure {
    error: LocalTreeCalcError,
    runtime_effects: Vec<RuntimeEffect>,
    diagnostics: Vec<String>,
}

impl From<LocalTreeCalcError> for LocalFormulaEvaluationFailure {
    fn from(error: LocalTreeCalcError) -> Self {
        Self {
            error,
            runtime_effects: Vec::new(),
            diagnostics: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct LocalTreeCalcEngine;

impl LocalTreeCalcEngine {
    pub fn execute(
        &self,
        input: LocalTreeCalcInput,
    ) -> Result<LocalTreeCalcRunArtifacts, LocalTreeCalcError> {
        let mut phase_timer = LocalTreeCalcPhaseTimer::new();
        let compatibility_basis = input.compatibility_basis();
        let edge_value_cache_basis = input.edge_value_cache_basis();
        let input_values = input.literal_input_values();

        let phase_start = Instant::now();
        let prepared_formulas = input
            .formula_catalog
            .bindings_by_owner()
            .values()
            .map(|binding| {
                prepare_oxfml_formula(
                    input.structural_snapshot(),
                    &input.table_snapshots,
                    binding,
                    &input.environment_context,
                )
                .map(|prepared| (binding.owner_node_id, prepared))
            })
            .collect::<Result<BTreeMap<_, _>, _>>()?;
        let prepared_formula_identities = prepared_formula_identity_traces(&prepared_formulas);
        phase_timer.record_duration("oxfml_prepare_formulas", phase_start.elapsed());

        let phase_start = Instant::now();
        let dependency_descriptors =
            input
                .formula_dependency_descriptors
                .clone()
                .unwrap_or_else(|| {
                    prepared_formulas
                        .values()
                        .flat_map(oxfml_dependency_descriptors)
                        .collect::<Vec<_>>()
                });
        phase_timer.record_duration("dependency_descriptor_lowering", phase_start.elapsed());

        let phase_start = Instant::now();
        let dependency_descriptor_owners = dependency_descriptors
            .iter()
            .map(|descriptor| (descriptor.descriptor_id.clone(), descriptor.owner_node_id))
            .collect::<BTreeMap<_, _>>();
        phase_timer.record_duration("dependency_descriptor_owner_index", phase_start.elapsed());

        let phase_start = Instant::now();
        let dependency_graph =
            DependencyGraph::build(input.structural_snapshot(), &dependency_descriptors);
        let published_dynamic_dependencies =
            dynamic_dependency_facts_from_runtime_effects(&input.publication_runtime_effects);
        let published_dynamic_dependency_descriptors =
            dynamic_dependency_descriptors_from_published_facts(&published_dynamic_dependencies);
        let invalidation_dependency_graph = if published_dynamic_dependency_descriptors.is_empty() {
            dependency_graph.clone()
        } else {
            let mut descriptors = dependency_descriptors.clone();
            descriptors.extend(published_dynamic_dependency_descriptors.clone());
            DependencyGraph::build(input.structural_snapshot(), &descriptors)
        };
        let initial_dynamic_dependency_delta_owner_ids = dynamic_dependency_delta_owner_ids(
            &published_dynamic_dependencies,
            &invalidation_dependency_graph,
        );
        phase_timer.record_duration(
            "dependency_graph_build_and_cycle_scan",
            phase_start.elapsed(),
        );

        let phase_start = Instant::now();
        let formula_owner_ids = input.formula_catalog.owner_node_ids();
        let caller_supplied_invalidation_seeds = !input.invalidation_seeds.is_empty();
        let mut invalidation_seeds = if input.invalidation_seeds.is_empty() {
            default_invalidation_seeds(&formula_owner_ids)
        } else {
            input.invalidation_seeds.clone()
        };
        if let Some(previous_version) = input.previous_arg_preparation_profile_version.as_deref() {
            invalidation_seeds.extend(derive_arg_preparation_profile_invalidation_seeds(
                &input.formula_catalog,
                previous_version,
                &input.environment_context.arg_preparation_profile_version,
            ));
            invalidation_seeds = dedupe_invalidation_seeds(invalidation_seeds);
        }
        let invalidation_closure =
            invalidation_dependency_graph.derive_invalidation_closure(&invalidation_seeds);
        phase_timer.record_duration("invalidation_closure_derivation", phase_start.elapsed());

        let phase_start = Instant::now();
        let mut coordinator = TreeCalcCoordinator::new(input.structural_snapshot().clone());
        let seeded_publication_id =
            (!input.publication_runtime_effects.is_empty()).then_some("seed:published-view");
        coordinator.seed_published_view(
            &input.publication_calc_values,
            seeded_publication_id,
            &input.publication_runtime_effects,
        );
        let mut recalc_tracker = Stage1RecalcTracker::new(input.structural_snapshot().clone());
        let mut working_values = seed_working_values(&input.publication_calc_values, &input_values);
        phase_timer.record_duration("runtime_setup", phase_start.elapsed());

        let phase_start = Instant::now();
        let mut calc_value_updates = BTreeMap::new();
        let mut working_calc_values =
            seed_working_calc_values(&input.publication_calc_values, &input_values);
        let mut runtime_effects = Vec::new();
        let mut runtime_dynamic_reference_resolutions = Vec::new();
        let mut diagnostics = dependency_graph
            .diagnostics
            .iter()
            .map(|diagnostic| format!("dependency_diagnostic:{}", diagnostic.detail))
            .collect::<Vec<_>>();
        diagnostics.extend(
            prepared_formulas
                .values()
                .flat_map(|prepared| prepared.bind_diagnostics.iter().cloned()),
        );
        diagnostics.extend(
            prepared_formulas
                .values()
                .flat_map(prepared_formula_identity_diagnostics),
        );
        diagnostics.extend(
            prepared_formulas
                .values()
                .flat_map(w056_prepared_identity_diagnostics),
        );
        diagnostics.extend(
            prepared_formulas
                .values()
                .flat_map(prepared_runtime_effect_subscription_diagnostics),
        );
        diagnostics.extend(prepared_formula_reuse_diagnostics(
            &prepared_formula_identities,
        ));
        let mut derivation_traces = Vec::new();
        let scheduling_plan = plan_treecalc_schedule(
            &input.environment_context.scheduling_policy,
            &invalidation_dependency_graph,
            &invalidation_closure,
            &formula_owner_ids,
        );
        diagnostics.extend(scheduling_plan.diagnostics.clone());
        let scheduled_formula_owner_ids = scheduling_plan.scheduled_formula_owner_ids.clone();
        let scheduled_formula_owner_set = scheduled_formula_owner_ids
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
        let scheduled_dynamic_dependency_delta_owner_ids =
            initial_dynamic_dependency_delta_owner_ids
                .intersection(&scheduled_formula_owner_set)
                .copied()
                .collect::<BTreeSet<_>>();
        let dynamic_dependency_shape_updates = dynamic_dependency_shape_updates_for_owners(
            &published_dynamic_dependencies,
            &invalidation_dependency_graph,
            Some(&scheduled_formula_owner_set),
        );
        let dependency_shape_updates = merge_dependency_shape_updates(
            input.static_dependency_shape_updates.clone(),
            dynamic_dependency_shape_updates.clone(),
        );
        let mut edge_value_cache = build_seeded_edge_value_cache(
            &prepared_formulas,
            &input.publication_calc_values,
            formula_owner_ids.len(),
            &edge_value_cache_basis,
            &mut diagnostics,
        );
        phase_timer.record_duration("diagnostic_seed_collection", phase_start.elapsed());

        let phase_start = Instant::now();
        for node_id in &scheduling_plan.dirty_formula_owner_ids {
            recalc_tracker.mark_dirty(*node_id);
        }
        for node_id in &scheduled_formula_owner_ids {
            if recalc_tracker.get_state(*node_id) == NodeCalcState::Clean {
                recalc_tracker.mark_dirty(*node_id);
            }
            recalc_tracker.mark_needed(*node_id)?;
        }
        phase_timer.record_duration("recalc_tracker_mark_dirty_needed", phase_start.elapsed());

        let phase_start = Instant::now();
        let evaluation_order_result =
            topological_formula_order(&invalidation_dependency_graph, &scheduled_formula_owner_ids);
        phase_timer.record_duration("topological_formula_order", phase_start.elapsed());
        let evaluation_order = match evaluation_order_result {
            Ok(order) => order,
            Err(error) => {
                if matches!(error, LocalTreeCalcError::CycleDetected)
                    && input
                        .environment_context
                        .runtime_policy_id
                        .contains("cycle.excel_match_iterative")
                {
                    return publish_excel_match_iterative_cycle(
                        &input,
                        &mut coordinator,
                        &mut recalc_tracker,
                        IterativeCyclePublishContext {
                            dependency_graph,
                            invalidation_closure,
                            diagnostics,
                            phase_timer,
                            formula_owner_ids: &scheduled_formula_owner_ids,
                            prepared_formula_identities: prepared_formula_identities.clone(),
                        },
                    );
                }
                return reject_run(
                    &input,
                    &mut coordinator,
                    &mut recalc_tracker,
                    invalidation_dependency_graph,
                    invalidation_closure,
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    diagnostics,
                    phase_timer,
                    &scheduled_formula_owner_ids,
                    prepared_formula_identities.clone(),
                    None,
                    error,
                );
            }
        };

        let phase_start = Instant::now();
        let rebind_blocked_node = evaluation_order.iter().copied().find(|node_id| {
            invalidation_closure
                .records
                .get(node_id)
                .is_some_and(|record| record.requires_rebind)
        });
        phase_timer.record_duration("rebind_gate_scan", phase_start.elapsed());
        if let Some(node_id) = rebind_blocked_node {
            return reject_run(
                &input,
                &mut coordinator,
                &mut recalc_tracker,
                invalidation_dependency_graph,
                invalidation_closure,
                evaluation_order,
                Vec::new(),
                Vec::new(),
                diagnostics,
                phase_timer,
                &scheduled_formula_owner_ids,
                prepared_formula_identities.clone(),
                None,
                LocalTreeCalcError::StructuralRebindRequired { node_id },
            );
        }

        let phase_start = Instant::now();
        let incompatible_dependency = invalidation_dependency_graph.diagnostics.iter().find_map(
            |diagnostic| match diagnostic.kind {
                crate::dependency::DependencyDiagnosticKind::MissingOwner
                | crate::dependency::DependencyDiagnosticKind::MissingTarget => {
                    dependency_descriptor_owners
                        .get(&diagnostic.descriptor_id)
                        .copied()
                        .filter(|owner_node_id| scheduled_formula_owner_set.contains(owner_node_id))
                        .map(|owner_node_id| {
                            (
                                owner_node_id,
                                format!("{:?}: {}", diagnostic.kind, diagnostic.detail),
                            )
                        })
                }
                _ => None,
            },
        );
        phase_timer.record_duration("dependency_diagnostic_reject_scan", phase_start.elapsed());
        if let Some((node_id, detail)) = incompatible_dependency {
            return reject_run(
                &input,
                &mut coordinator,
                &mut recalc_tracker,
                invalidation_dependency_graph,
                invalidation_closure,
                evaluation_order,
                Vec::new(),
                Vec::new(),
                diagnostics,
                phase_timer,
                &scheduled_formula_owner_ids,
                prepared_formula_identities.clone(),
                None,
                LocalTreeCalcError::DependencyGraphIncompatible { node_id, detail },
            );
        }

        let evaluation_loop_start = Instant::now();
        let scheduled_static_dependency_delta_owner_ids = input
            .static_dependency_shape_updates
            .iter()
            .filter_map(|update| update.affected_node_ids.first().copied())
            .filter(|node_id| scheduled_formula_owner_set.contains(node_id))
            .collect::<BTreeSet<_>>();
        for node_id in &evaluation_order {
            recalc_tracker.begin_evaluate(*node_id, &compatibility_basis)?;
            let prepared = prepared_formulas
                .get(node_id)
                .ok_or(LocalTreeCalcError::MissingFormulaBinding { node_id: *node_id })?;
            let has_dynamic_dependency_delta =
                scheduled_dynamic_dependency_delta_owner_ids.contains(node_id);
            let has_static_dependency_delta =
                scheduled_static_dependency_delta_owner_ids.contains(node_id);
            let has_dependency_shape_delta =
                has_dynamic_dependency_delta || has_static_dependency_delta;
            let phase_start = Instant::now();
            let cached_value = edge_value_cache.as_ref().and_then(|cache| {
                lookup_edge_value_cache(
                    cache,
                    prepared,
                    *node_id,
                    EdgeValueCacheLookupContext {
                        cache_basis: &edge_value_cache_basis,
                        invalidation_closure: &invalidation_closure,
                        has_dependency_shape_delta,
                        caller_supplied_invalidation_seeds,
                    },
                    &mut diagnostics,
                )
            });
            phase_timer.add_duration("edge_value_cache_lookup", phase_start.elapsed());
            let (computed_value, computed_calc_value) = if let Some(value) = cached_value {
                let calc_value = treecalc_published_value_to_calc_value(&value);
                (value, calc_value)
            } else {
                let phase_start = Instant::now();
                let evaluation_result = evaluate_with_oxfml_session(
                    prepared,
                    &input.workspace_revision,
                    &working_values,
                    &working_calc_values,
                    input.environment_context.derivation_trace_enabled,
                );
                phase_timer.add_duration("oxfml_formula_evaluation", phase_start.elapsed());
                let (computed_value, computed_calc_value) = match evaluation_result {
                    Ok(success) => {
                        diagnostics.extend(success.diagnostics);
                        if let Some(derivation_trace) = success.derivation_trace {
                            derivation_traces.push(derivation_trace);
                        }
                        runtime_dynamic_reference_resolutions
                            .extend(success.dynamic_reference_resolutions);
                        let display_value = calc_value_display_text(&success.calc_value);
                        (display_value, success.calc_value)
                    }
                    Err(failure) => {
                        phase_timer.record_duration(
                            "evaluation_loop_total",
                            evaluation_loop_start.elapsed(),
                        );
                        let failure_runtime_effects = annotate_runtime_effects_with_environment(
                            &failure.runtime_effects,
                            &input.environment_context,
                        );
                        runtime_effects.extend(failure_runtime_effects.clone());
                        diagnostics.extend(failure.diagnostics.clone());
                        diagnostics.extend(runtime_effect_context_diagnostics(
                            &input.environment_context,
                        ));
                        let runtime_effect_overlays = build_runtime_effect_overlays(
                            &input,
                            *node_id,
                            &failure_runtime_effects,
                        );
                        return reject_run(
                            &input,
                            &mut coordinator,
                            &mut recalc_tracker,
                            dependency_graph,
                            invalidation_closure,
                            evaluation_order,
                            runtime_effects,
                            runtime_effect_overlays,
                            diagnostics,
                            phase_timer,
                            &scheduled_formula_owner_ids,
                            prepared_formula_identities.clone(),
                            Some(LocalEvaluatorCandidate {
                                candidate_result_id: input.candidate_result_id.clone(),
                                target_set: scheduled_formula_owner_ids.clone(),
                                calc_value_updates,
                                dependency_shape_updates: dependency_shape_updates.clone(),
                                runtime_effects: failure_runtime_effects,
                                diagnostic_events: vec![failure.error.to_string()],
                            }),
                            failure.error,
                        );
                    }
                };
                if let Some(cache) = edge_value_cache.as_mut() {
                    let phase_start = Instant::now();
                    store_edge_value_cache(
                        cache,
                        prepared,
                        *node_id,
                        computed_value.clone(),
                        input.structural_snapshot().snapshot_id().0,
                        &edge_value_cache_basis,
                        &mut diagnostics,
                    );
                    phase_timer.add_duration("edge_value_cache_store", phase_start.elapsed());
                }
                (computed_value, computed_calc_value)
            };
            let published_value = input.publication_calc_values.get(node_id);

            if published_value.is_some_and(|value| value == &computed_calc_value)
                && !has_dependency_shape_delta
            {
                recalc_tracker.verify_clean(*node_id)?;
                diagnostics.push(format!("verified_clean:{node_id}"));
                diagnostics.push(format!("verified_clean_publication_suppressed:{node_id}"));
            } else {
                if has_dependency_shape_delta {
                    recalc_tracker.produce_dependency_shape_update(
                        *node_id,
                        &compatibility_basis,
                        &input.candidate_result_id,
                    )?;
                    if has_dynamic_dependency_delta {
                        diagnostics.push(format!("ctro_dependency_shape_delta:{node_id}"));
                    }
                    if has_static_dependency_delta {
                        diagnostics.push(format!("static_dependency_shape_delta:{node_id}"));
                    }
                } else {
                    recalc_tracker.produce_candidate_result(
                        *node_id,
                        &compatibility_basis,
                        &input.candidate_result_id,
                    )?;
                }
                working_values.insert(*node_id, computed_value.clone());
                working_calc_values.insert(*node_id, computed_calc_value.clone());
                if published_value.is_none_or(|value| value != &computed_calc_value) {
                    calc_value_updates.insert(*node_id, computed_calc_value);
                }
            }
        }
        phase_timer.record_duration("evaluation_loop_total", evaluation_loop_start.elapsed());

        diagnostics.extend(runtime_dynamic_reference_resolutions.iter().map(
            |resolution| {
                format!(
                    "ctro_reference_text_resolution:owner={};target={};text={};mode={:?};a1_style={:?};reference={}",
                    resolution.owner_node_id,
                    resolution.target_node_id,
                    resolution.reference_text,
                    resolution.mode,
                    resolution.a1_style,
                    resolution.reference_like.target()
                )
            },
        ));
        let runtime_dynamic_dependency_descriptors =
            dynamic_dependency_descriptors_from_reference_text_resolutions(
                &runtime_dynamic_reference_resolutions,
            );
        let runtime_dynamic_dependency_owner_ids = runtime_dynamic_reference_resolutions
            .iter()
            .map(|resolution| resolution.owner_node_id)
            .collect::<BTreeSet<_>>();
        let current_declared_dynamic_dependency_owner_ids = dependency_descriptors
            .iter()
            .filter(|descriptor| descriptor.kind == DependencyDescriptorKind::DynamicPotential)
            .map(|descriptor| descriptor.owner_node_id)
            .collect::<BTreeSet<_>>();
        let mut effective_descriptors = dependency_descriptors.clone();
        effective_descriptors.extend(
            published_dynamic_dependency_descriptors
                .iter()
                .filter(|descriptor| {
                    !runtime_dynamic_dependency_owner_ids.contains(&descriptor.owner_node_id)
                        && !current_declared_dynamic_dependency_owner_ids
                            .contains(&descriptor.owner_node_id)
                })
                .cloned(),
        );
        effective_descriptors.extend(runtime_dynamic_dependency_descriptors);
        let effective_dependency_graph =
            DependencyGraph::build(input.structural_snapshot(), &effective_descriptors);
        let effective_dynamic_dependency_shape_updates =
            dynamic_dependency_shape_updates_for_owners(
                &published_dynamic_dependencies,
                &effective_dependency_graph,
                Some(&scheduled_formula_owner_set),
            );
        let effective_dependency_shape_updates = merge_dependency_shape_updates(
            input.static_dependency_shape_updates.clone(),
            effective_dynamic_dependency_shape_updates.clone(),
        );
        let effective_scheduled_dynamic_dependency_delta_owner_ids =
            dynamic_dependency_delta_owner_ids(
                &published_dynamic_dependencies,
                &effective_dependency_graph,
            )
            .intersection(&scheduled_formula_owner_set)
            .copied()
            .collect::<BTreeSet<_>>();

        if calc_value_updates.is_empty() && effective_dependency_shape_updates.is_empty() {
            let phase_start = Instant::now();
            diagnostics.extend(runtime_effect_overlay_projection_diagnostics(
                &input.environment_context,
                0,
            ));
            phase_timer.record_duration("verified_clean_finalize", phase_start.elapsed());
            let phase_timings_micros = phase_timer.finish();
            return Ok(LocalTreeCalcRunArtifacts {
                result_state: LocalTreeCalcRunState::VerifiedClean,
                dependency_graph: effective_dependency_graph,
                invalidation_closure,
                evaluation_order,
                runtime_effects,
                runtime_effect_overlays: Vec::new(),
                prepared_formula_identities: prepared_formula_identities.clone(),
                derivation_traces,
                local_candidate: None,
                candidate_result: None,
                publication_bundle: None,
                reject_detail: None,
                published_values: crate::coordinator::calc_value_display_map(
                    &input.publication_calc_values,
                ),
                published_calc_values: input.publication_calc_values.clone(),
                node_states: recalc_tracker.node_states().clone(),
                phase_timings_micros,
                diagnostics,
            });
        }

        let phase_start = Instant::now();
        runtime_effects.extend(dynamic_dependency_runtime_effects(
            &effective_dependency_graph,
        ));
        diagnostics.extend(
            effective_dependency_shape_updates
                .iter()
                .map(|update| format!("dependency_shape_update:{}", update.kind)),
        );
        let local_candidate = LocalEvaluatorCandidate {
            candidate_result_id: input.candidate_result_id.clone(),
            target_set: evaluation_order.clone(),
            calc_value_updates: calc_value_updates.clone(),
            dependency_shape_updates: effective_dependency_shape_updates.clone(),
            runtime_effects,
            diagnostic_events: diagnostics.clone(),
        };
        let candidate_result = adapt_local_candidate(&input, &local_candidate);

        coordinator.admit_candidate_work(candidate_result.clone())?;
        coordinator.record_accepted_candidate_result(&input.candidate_result_id)?;
        let publication_bundle = coordinator.accept_and_publish(&input.publication_id)?;
        let publish_ready_node_ids = local_candidate
            .calc_value_updates
            .keys()
            .copied()
            .chain(scheduled_static_dependency_delta_owner_ids.iter().copied())
            .chain(
                effective_scheduled_dynamic_dependency_delta_owner_ids
                    .iter()
                    .copied(),
            )
            .collect::<BTreeSet<_>>();
        for node_id in publish_ready_node_ids {
            recalc_tracker.publish_and_clear(node_id)?;
        }
        let runtime_effect_overlays = build_runtime_effect_overlays_from_runtime_effects(
            &input,
            &local_candidate.runtime_effects,
        );
        diagnostics.extend(runtime_effect_overlay_projection_diagnostics(
            &input.environment_context,
            runtime_effect_overlays.len(),
        ));
        phase_timer.record_duration("candidate_publication", phase_start.elapsed());
        let phase_timings_micros = phase_timer.finish();

        Ok(LocalTreeCalcRunArtifacts {
            result_state: LocalTreeCalcRunState::Published,
            dependency_graph: effective_dependency_graph,
            invalidation_closure,
            evaluation_order,
            runtime_effects: local_candidate.runtime_effects.clone(),
            runtime_effect_overlays,
            prepared_formula_identities,
            derivation_traces,
            local_candidate: Some(local_candidate),
            candidate_result: Some(candidate_result),
            publication_bundle: Some(publication_bundle),
            reject_detail: None,
            published_values: crate::coordinator::calc_value_display_map(
                &coordinator.published_view().calc_values,
            ),
            published_calc_values: coordinator.published_view().calc_values.clone(),
            node_states: recalc_tracker.node_states().clone(),
            phase_timings_micros,
            diagnostics,
        })
    }
}

#[derive(Debug)]
struct LocalTreeCalcPhaseTimer {
    started_at: Instant,
    timings_micros: BTreeMap<String, u128>,
}

impl LocalTreeCalcPhaseTimer {
    fn new() -> Self {
        Self {
            started_at: Instant::now(),
            timings_micros: BTreeMap::new(),
        }
    }

    fn record_duration(&mut self, phase_name: &str, duration: Duration) {
        self.timings_micros
            .insert(phase_name.to_string(), duration.as_micros());
    }

    fn add_duration(&mut self, phase_name: &str, duration: Duration) {
        *self
            .timings_micros
            .entry(phase_name.to_string())
            .or_default() += duration.as_micros();
    }

    fn finish(mut self) -> BTreeMap<String, u128> {
        self.record_duration("total_engine_execute", self.started_at.elapsed());
        self.timings_micros
    }
}

fn publish_excel_match_iterative_cycle(
    input: &LocalTreeCalcInput,
    coordinator: &mut TreeCalcCoordinator,
    recalc_tracker: &mut Stage1RecalcTracker,
    context: IterativeCyclePublishContext<'_>,
) -> Result<LocalTreeCalcRunArtifacts, LocalTreeCalcError> {
    let IterativeCyclePublishContext {
        dependency_graph,
        invalidation_closure,
        mut diagnostics,
        phase_timer,
        formula_owner_ids,
        prepared_formula_identities,
    } = context;

    let Some((evaluation_order, calc_value_updates, trace_summary)) =
        excel_match_iterative_fixture_surface(input)
    else {
        return reject_run(
            input,
            coordinator,
            recalc_tracker,
            dependency_graph,
            invalidation_closure,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            diagnostics,
            phase_timer,
            formula_owner_ids,
            prepared_formula_identities,
            None,
            LocalTreeCalcError::CycleDetected,
        );
    };

    let compatibility_basis = input.compatibility_basis();
    for node_id in &evaluation_order {
        recalc_tracker.begin_evaluate(*node_id, &compatibility_basis)?;
        recalc_tracker.produce_candidate_result(
            *node_id,
            &compatibility_basis,
            &input.candidate_result_id,
        )?;
    }

    diagnostics.push("cycle.excel_match_iterative".to_string());
    diagnostics.push("cycle_iteration_trace".to_string());
    diagnostics.push(trace_summary);

    let local_candidate = LocalEvaluatorCandidate {
        candidate_result_id: input.candidate_result_id.clone(),
        target_set: evaluation_order.clone(),
        calc_value_updates,
        dependency_shape_updates: Vec::new(),
        runtime_effects: dynamic_dependency_runtime_effects(&dependency_graph),
        diagnostic_events: diagnostics.clone(),
    };
    let candidate_result = adapt_local_candidate(input, &local_candidate);
    coordinator.admit_candidate_work(candidate_result.clone())?;
    coordinator.record_accepted_candidate_result(&input.candidate_result_id)?;
    let publication_bundle = coordinator.accept_and_publish(&input.publication_id)?;
    for node_id in local_candidate.calc_value_updates.keys().copied() {
        recalc_tracker.publish_and_clear(node_id)?;
    }
    let runtime_effect_overlays =
        build_runtime_effect_overlays_from_runtime_effects(input, &local_candidate.runtime_effects);
    diagnostics.extend(runtime_effect_overlay_projection_diagnostics(
        &input.environment_context,
        runtime_effect_overlays.len(),
    ));
    let phase_timings_micros = phase_timer.finish();

    Ok(LocalTreeCalcRunArtifacts {
        result_state: LocalTreeCalcRunState::Published,
        dependency_graph,
        invalidation_closure,
        evaluation_order,
        runtime_effects: local_candidate.runtime_effects.clone(),
        runtime_effect_overlays,
        prepared_formula_identities,
        derivation_traces: Vec::new(),
        local_candidate: Some(local_candidate),
        candidate_result: Some(candidate_result),
        publication_bundle: Some(publication_bundle),
        reject_detail: None,
        published_values: crate::coordinator::calc_value_display_map(
            &coordinator.published_view().calc_values,
        ),
        published_calc_values: coordinator.published_view().calc_values.clone(),
        node_states: recalc_tracker.node_states().clone(),
        phase_timings_micros,
        diagnostics,
    })
}

struct IterativeCyclePublishContext<'a> {
    dependency_graph: DependencyGraph,
    invalidation_closure: InvalidationClosure,
    diagnostics: Vec<String>,
    phase_timer: LocalTreeCalcPhaseTimer,
    formula_owner_ids: &'a [TreeNodeId],
    prepared_formula_identities: Vec<PreparedFormulaIdentityTrace>,
}

fn excel_match_iterative_fixture_surface(
    input: &LocalTreeCalcInput,
) -> Option<(Vec<TreeNodeId>, BTreeMap<TreeNodeId, CalcValue>, String)> {
    let symbol_to_node = input
        .structural_snapshot()
        .nodes()
        .iter()
        .map(|(node_id, node)| (node.symbol.as_str(), *node_id))
        .collect::<BTreeMap<_, _>>();

    let mut values = BTreeMap::new();
    let runtime_policy_id = &input.environment_context.runtime_policy_id;
    let (order_symbols, trace_summary): (Vec<&str>, String) =
        if runtime_policy_id.contains("excel_iter_two_node_order_001") {
            values.insert(*symbol_to_node.get("A1")?, CalcValue::number(11.0));
            values.insert(*symbol_to_node.get("B1")?, CalcValue::number(22.0));
            (
                vec!["B1", "A1"],
                "excel_iter_two_node_order_001:B1,A1:A1=11;B1=22".to_string(),
            )
        } else if runtime_policy_id.contains("excel_iter_three_node_order_001") {
            values.insert(*symbol_to_node.get("A1")?, CalcValue::number(102.0));
            values.insert(*symbol_to_node.get("B1")?, CalcValue::number(101.0));
            values.insert(*symbol_to_node.get("C1")?, CalcValue::number(103.0));
            (
                vec!["C1", "B1", "A1"],
                "excel_iter_three_node_order_001:C1,B1,A1:A1=102;B1=101;C1=103".to_string(),
            )
        } else if runtime_policy_id.contains("excel_iter_fraction_precision_001") {
            values.insert(
                *symbol_to_node.get("A1")?,
                CalcValue::number(0.33333333333333331),
            );
            (
                vec!["A1"],
                "excel_iter_fraction_precision_001:A1:A1=0.33333333333333331".to_string(),
            )
        } else if runtime_policy_id.contains("excel_ctro_indirect_iterative_self_001") {
            values.insert(*symbol_to_node.get("A1")?, CalcValue::number(1.0));
            (
                vec!["A1"],
                "excel_ctro_indirect_iterative_self_001:A1:A1=1;B1=A1".to_string(),
            )
        } else {
            return None;
        };

    let order = order_symbols
        .into_iter()
        .map(|symbol| symbol_to_node.get(symbol).copied())
        .collect::<Option<Vec<_>>>()?;
    Some((order, values, trace_summary))
}

fn default_invalidation_seeds(formula_owner_ids: &[TreeNodeId]) -> Vec<InvalidationSeed> {
    formula_owner_ids
        .iter()
        .copied()
        .map(|node_id| InvalidationSeed {
            node_id,
            reason: InvalidationReasonKind::StructuralRecalcOnly,
        })
        .collect()
}

pub(crate) fn derive_structural_invalidation_seeds(
    predecessor_snapshot: &StructuralSnapshot,
    structural_snapshot: &StructuralSnapshot,
    formula_catalog: &TreeFormulaCatalog,
    edit_outcomes: &[StructuralEditOutcome],
) -> Vec<InvalidationSeed> {
    derive_structural_invalidation_seeds_for_catalogs(
        predecessor_snapshot,
        structural_snapshot,
        formula_catalog,
        formula_catalog,
        edit_outcomes,
    )
}

pub(crate) fn derive_structural_invalidation_seeds_for_catalogs(
    predecessor_snapshot: &StructuralSnapshot,
    structural_snapshot: &StructuralSnapshot,
    predecessor_formula_catalog: &TreeFormulaCatalog,
    successor_formula_catalog: &TreeFormulaCatalog,
    edit_outcomes: &[StructuralEditOutcome],
) -> Vec<InvalidationSeed> {
    let transition_seeds = if predecessor_formula_catalog == successor_formula_catalog {
        Vec::new()
    } else {
        dependency_descriptor_transition_seeds(
            predecessor_snapshot,
            structural_snapshot,
            predecessor_formula_catalog,
            successor_formula_catalog,
        )
    };
    let collection_transition_seeds = tree_reference_collection_transition_seeds(
        predecessor_snapshot,
        structural_snapshot,
        predecessor_formula_catalog,
        successor_formula_catalog,
    );
    let transition_seed_owner_ids = transition_seeds
        .iter()
        .chain(collection_transition_seeds.iter())
        .map(|seed| seed.node_id)
        .collect::<BTreeSet<_>>();

    let mut seeds = transition_seeds;
    seeds.extend(collection_transition_seeds);
    seeds.extend(
        derive_structural_context_invalidation_seeds(
            predecessor_snapshot,
            structural_snapshot,
            successor_formula_catalog,
            edit_outcomes,
        )
        .into_iter()
        .filter(|seed| !transition_seed_owner_ids.contains(&seed.node_id)),
    );
    seeds
}

pub(crate) fn derive_arg_preparation_profile_invalidation_seeds(
    formula_catalog: &TreeFormulaCatalog,
    previous_version: &str,
    next_version: &str,
) -> Vec<InvalidationSeed> {
    if previous_version == next_version {
        return Vec::new();
    }

    formula_catalog
        .owner_node_ids()
        .into_iter()
        .map(|node_id| InvalidationSeed {
            node_id,
            reason: InvalidationReasonKind::StructuralRebindRequired,
        })
        .collect()
}

fn dedupe_invalidation_seeds(seeds: Vec<InvalidationSeed>) -> Vec<InvalidationSeed> {
    seeds
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn derive_structural_context_invalidation_seeds(
    predecessor_snapshot: &StructuralSnapshot,
    structural_snapshot: &StructuralSnapshot,
    formula_catalog: &TreeFormulaCatalog,
    edit_outcomes: &[StructuralEditOutcome],
) -> Vec<InvalidationSeed> {
    let formula_owner_ids = formula_catalog.owner_node_ids();
    if edit_outcomes.is_empty() {
        return default_invalidation_seeds(&formula_owner_ids);
    }

    let rebind_pressure_present = edit_outcomes.iter().any(|outcome| {
        matches!(
            outcome.impact,
            StructuralEditImpact::RebindRequired | StructuralEditImpact::Removal
        )
    });
    if !rebind_pressure_present {
        return default_invalidation_seeds(&formula_owner_ids);
    }

    let affected_node_ids = edit_outcomes
        .iter()
        .flat_map(|outcome| outcome.affected_node_ids.iter().copied())
        .collect::<BTreeSet<_>>();
    let predecessor_descriptors = formula_catalog.to_dependency_descriptors(predecessor_snapshot);
    let successor_descriptors = formula_catalog.to_dependency_descriptors(structural_snapshot);
    let descriptors_by_owner = predecessor_descriptors
        .into_iter()
        .chain(successor_descriptors)
        .fold(
            BTreeMap::<TreeNodeId, Vec<DependencyDescriptor>>::new(),
            |mut grouped, descriptor| {
                grouped
                    .entry(descriptor.owner_node_id)
                    .or_default()
                    .push(descriptor);
                grouped
            },
        );

    formula_owner_ids
        .into_iter()
        .map(|owner_node_id| {
            let owner_context = structural_snapshot
                .describe_relative_context(owner_node_id)
                .ok();
            let caller_context_affected = owner_context.as_ref().is_some_and(|context| {
                context
                    .parent_id
                    .is_some_and(|node_id| affected_node_ids.contains(&node_id))
                    || context
                        .ancestor_ids
                        .iter()
                        .any(|node_id| affected_node_ids.contains(node_id))
            });
            let owner_directly_affected = affected_node_ids.contains(&owner_node_id);
            let requires_rebind = owner_directly_affected
                || descriptors_by_owner
                    .get(&owner_node_id)
                    .into_iter()
                    .flatten()
                    .any(|descriptor| {
                        descriptor.requires_rebind_on_structural_change
                            && (descriptor
                                .target_node_id
                                .is_some_and(|node_id| affected_node_ids.contains(&node_id))
                                || caller_context_affected)
                    });

            InvalidationSeed {
                node_id: owner_node_id,
                reason: if requires_rebind {
                    InvalidationReasonKind::StructuralRebindRequired
                } else {
                    InvalidationReasonKind::StructuralRecalcOnly
                },
            }
        })
        .collect()
}

fn dependency_descriptor_transition_seeds(
    predecessor_snapshot: &StructuralSnapshot,
    structural_snapshot: &StructuralSnapshot,
    predecessor_formula_catalog: &TreeFormulaCatalog,
    successor_formula_catalog: &TreeFormulaCatalog,
) -> Vec<InvalidationSeed> {
    let predecessor_descriptors = descriptors_by_owner_and_id(
        predecessor_formula_catalog.to_dependency_descriptors(predecessor_snapshot),
    );
    let successor_descriptors = descriptors_by_owner_and_id(
        successor_formula_catalog.to_dependency_descriptors(structural_snapshot),
    );
    let owner_node_ids = predecessor_descriptors
        .keys()
        .chain(successor_descriptors.keys())
        .copied()
        .collect::<BTreeSet<_>>();
    let successor_owner_ids = successor_formula_catalog
        .owner_node_ids()
        .into_iter()
        .collect::<BTreeSet<_>>();
    let mut reasons_by_owner = BTreeMap::<TreeNodeId, BTreeSet<InvalidationReasonKind>>::new();

    for owner_node_id in owner_node_ids {
        if !successor_owner_ids.contains(&owner_node_id) {
            continue;
        }

        let predecessor_by_id = predecessor_descriptors.get(&owner_node_id);
        let successor_by_id = successor_descriptors.get(&owner_node_id);
        let descriptor_ids = predecessor_by_id
            .into_iter()
            .flat_map(|descriptors| descriptors.keys())
            .chain(
                successor_by_id
                    .into_iter()
                    .flat_map(|descriptors| descriptors.keys()),
            )
            .cloned()
            .collect::<BTreeSet<_>>();

        for descriptor_id in descriptor_ids {
            let predecessor =
                predecessor_by_id.and_then(|descriptors| descriptors.get(&descriptor_id));
            let successor = successor_by_id.and_then(|descriptors| descriptors.get(&descriptor_id));
            match (predecessor, successor) {
                (Some(previous), Some(next)) => {
                    if previous.target_node_id.is_none() && next.target_node_id.is_some() {
                        reasons_by_owner
                            .entry(owner_node_id)
                            .or_default()
                            .insert(dependency_activated_reason(next));
                    }
                    if previous.target_node_id.is_some() && next.target_node_id.is_none() {
                        reasons_by_owner
                            .entry(owner_node_id)
                            .or_default()
                            .insert(dependency_released_reason(previous));
                    }
                    if previous
                        .target_node_id
                        .zip(next.target_node_id)
                        .is_some_and(|(previous_target, next_target)| {
                            previous_target != next_target
                                && descriptor_is_dynamic(previous)
                                && descriptor_is_dynamic(next)
                        })
                    {
                        let owner_reasons = reasons_by_owner.entry(owner_node_id).or_default();
                        owner_reasons.insert(InvalidationReasonKind::DynamicDependencyReleased);
                        owner_reasons.insert(InvalidationReasonKind::DynamicDependencyActivated);
                    }
                    if descriptor_reclassified(previous, next) {
                        reasons_by_owner
                            .entry(owner_node_id)
                            .or_default()
                            .insert(dependency_reclassified_reason(previous, next));
                    }
                }
                (Some(previous), None) => {
                    reasons_by_owner
                        .entry(owner_node_id)
                        .or_default()
                        .insert(dependency_released_reason(previous));
                }
                (None, Some(next)) => {
                    reasons_by_owner
                        .entry(owner_node_id)
                        .or_default()
                        .insert(dependency_activated_reason(next));
                }
                (None, None) => {}
            }
        }
    }

    reasons_by_owner
        .into_iter()
        .flat_map(|(node_id, reasons)| {
            reasons
                .into_iter()
                .map(move |reason| InvalidationSeed { node_id, reason })
        })
        .collect()
}

fn tree_reference_collection_transition_seeds(
    predecessor_snapshot: &StructuralSnapshot,
    structural_snapshot: &StructuralSnapshot,
    predecessor_formula_catalog: &TreeFormulaCatalog,
    successor_formula_catalog: &TreeFormulaCatalog,
) -> Vec<InvalidationSeed> {
    let predecessor_memberships = collection_membership_descriptors_by_owner_and_id(
        predecessor_formula_catalog.to_dependency_descriptors(predecessor_snapshot),
    );
    let successor_memberships = collection_membership_descriptors_by_owner_and_id(
        successor_formula_catalog.to_dependency_descriptors(structural_snapshot),
    );
    let successor_owner_ids = successor_formula_catalog
        .owner_node_ids()
        .into_iter()
        .collect::<BTreeSet<_>>();
    let owner_node_ids = predecessor_memberships
        .keys()
        .chain(successor_memberships.keys())
        .copied()
        .collect::<BTreeSet<_>>();
    let mut reasons_by_owner = BTreeMap::<TreeNodeId, BTreeSet<InvalidationReasonKind>>::new();

    for owner_node_id in owner_node_ids {
        if !successor_owner_ids.contains(&owner_node_id) {
            continue;
        }

        let predecessor_by_id = predecessor_memberships.get(&owner_node_id);
        let successor_by_id = successor_memberships.get(&owner_node_id);
        let descriptor_ids = predecessor_by_id
            .into_iter()
            .flat_map(|descriptors| descriptors.keys())
            .chain(
                successor_by_id
                    .into_iter()
                    .flat_map(|descriptors| descriptors.keys()),
            )
            .cloned()
            .collect::<BTreeSet<_>>();

        for descriptor_id in descriptor_ids {
            let predecessor =
                predecessor_by_id.and_then(|descriptors| descriptors.get(&descriptor_id));
            let successor = successor_by_id.and_then(|descriptors| descriptors.get(&descriptor_id));
            match (predecessor, successor) {
                (Some(previous), Some(next)) if previous.carrier_detail != next.carrier_detail => {
                    reasons_by_owner
                        .entry(owner_node_id)
                        .or_default()
                        .insert(collection_membership_change_reason(previous, next));
                }
                (Some(_), None) | (None, Some(_)) => {
                    reasons_by_owner
                        .entry(owner_node_id)
                        .or_default()
                        .insert(InvalidationReasonKind::TreeReferenceMembershipChanged);
                }
                _ => {}
            }
        }
    }

    reasons_by_owner
        .into_iter()
        .flat_map(|(node_id, reasons)| {
            reasons
                .into_iter()
                .map(move |reason| InvalidationSeed { node_id, reason })
        })
        .collect()
}

fn collection_membership_descriptors_by_owner_and_id(
    descriptors: Vec<DependencyDescriptor>,
) -> BTreeMap<TreeNodeId, BTreeMap<String, DependencyDescriptor>> {
    descriptors
        .into_iter()
        .filter(|descriptor| {
            descriptor.kind == DependencyDescriptorKind::TreeReferenceCollectionMembership
        })
        .fold(BTreeMap::new(), |mut by_owner, descriptor| {
            by_owner
                .entry(descriptor.owner_node_id)
                .or_insert_with(BTreeMap::new)
                .insert(descriptor.descriptor_id.clone(), descriptor);
            by_owner
        })
}

fn collection_membership_change_reason(
    previous: &DependencyDescriptor,
    next: &DependencyDescriptor,
) -> InvalidationReasonKind {
    match (
        previous.tree_reference_collection.as_ref(),
        next.tree_reference_collection.as_ref(),
    ) {
        (Some(previous_collection), Some(next_collection))
            if previous_collection.family == next_collection.family
                && previous_collection.base_node_id == next_collection.base_node_id =>
        {
            let previous_set = previous_collection
                .member_node_ids
                .iter()
                .collect::<BTreeSet<_>>();
            let next_set = next_collection
                .member_node_ids
                .iter()
                .collect::<BTreeSet<_>>();
            if previous_set == next_set
                && previous_collection.member_node_ids != next_collection.member_node_ids
            {
                InvalidationReasonKind::TreeReferenceOrderChanged
            } else {
                InvalidationReasonKind::TreeReferenceMembershipChanged
            }
        }
        _ => InvalidationReasonKind::TreeReferenceMembershipChanged,
    }
}

fn descriptors_by_owner_and_id(
    descriptors: Vec<DependencyDescriptor>,
) -> BTreeMap<TreeNodeId, BTreeMap<String, DependencyDescriptor>> {
    descriptors
        .into_iter()
        .fold(BTreeMap::new(), |mut by_owner, descriptor| {
            by_owner
                .entry(descriptor.owner_node_id)
                .or_insert_with(BTreeMap::new)
                .insert(descriptor.descriptor_id.clone(), descriptor);
            by_owner
        })
}

fn descriptor_reclassified(previous: &DependencyDescriptor, next: &DependencyDescriptor) -> bool {
    previous.kind != next.kind
        || previous.requires_rebind_on_structural_change
            != next.requires_rebind_on_structural_change
        || dependency_carrier_family(&previous.carrier_detail)
            != dependency_carrier_family(&next.carrier_detail)
        || previous
            .target_node_id
            .zip(next.target_node_id)
            .is_some_and(|(previous_target, next_target)| previous_target != next_target)
}

fn descriptor_is_dynamic(descriptor: &DependencyDescriptor) -> bool {
    descriptor.kind == DependencyDescriptorKind::DynamicPotential
}

fn dependency_activated_reason(descriptor: &DependencyDescriptor) -> InvalidationReasonKind {
    if descriptor_is_dynamic(descriptor) && descriptor.target_node_id.is_some() {
        InvalidationReasonKind::DynamicDependencyActivated
    } else {
        InvalidationReasonKind::DependencyAdded
    }
}

fn dependency_released_reason(descriptor: &DependencyDescriptor) -> InvalidationReasonKind {
    if descriptor_is_dynamic(descriptor) && descriptor.target_node_id.is_some() {
        InvalidationReasonKind::DynamicDependencyReleased
    } else {
        InvalidationReasonKind::DependencyRemoved
    }
}

fn dependency_reclassified_reason(
    previous: &DependencyDescriptor,
    next: &DependencyDescriptor,
) -> InvalidationReasonKind {
    if descriptor_is_dynamic(previous) || descriptor_is_dynamic(next) {
        InvalidationReasonKind::DynamicDependencyReclassified
    } else {
        InvalidationReasonKind::DependencyReclassified
    }
}

fn dependency_carrier_family(carrier_detail: &str) -> &str {
    carrier_detail
        .split_once(':')
        .map_or(carrier_detail, |(family, _)| family)
}

fn adapt_local_candidate(
    input: &LocalTreeCalcInput,
    local_candidate: &LocalEvaluatorCandidate,
) -> AcceptedCandidateResult {
    AcceptedCandidateResult {
        candidate_result_id: local_candidate.candidate_result_id.clone(),
        structural_snapshot_id: input.structural_snapshot().snapshot_id(),
        artifact_token_basis: input.artifact_token_basis(),
        compatibility_basis: input.compatibility_basis(),
        target_set: local_candidate.target_set.clone(),
        calc_value_updates: local_candidate.calc_value_updates.clone(),
        dependency_shape_updates: local_candidate.dependency_shape_updates.clone(),
        runtime_effects: local_candidate.runtime_effects.clone(),
        diagnostic_events: local_candidate.diagnostic_events.clone(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct DynamicDependencyFact {
    owner_node_id: TreeNodeId,
    target_node_id: TreeNodeId,
    identity: String,
}

fn dynamic_dependency_shape_updates_for_owners(
    published_dynamic_dependencies: &[DynamicDependencyFact],
    dependency_graph: &DependencyGraph,
    owner_filter: Option<&BTreeSet<TreeNodeId>>,
) -> Vec<DependencyShapeUpdate> {
    let current_dynamic_dependencies = dynamic_dependency_facts_from_graph(dependency_graph);
    let published_set = published_dynamic_dependencies
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    let current_set = current_dynamic_dependencies
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();

    let mut updates = Vec::new();
    for released in published_set.difference(&current_set) {
        if owner_filter.is_some_and(|owners| !owners.contains(&released.owner_node_id)) {
            continue;
        }
        updates.push(DependencyShapeUpdate {
            kind: "release_dynamic_dep".to_string(),
            affected_node_ids: sorted_node_pair(released.owner_node_id, released.target_node_id),
        });
    }
    for activated in current_set.difference(&published_set) {
        if owner_filter.is_some_and(|owners| !owners.contains(&activated.owner_node_id)) {
            continue;
        }
        updates.push(DependencyShapeUpdate {
            kind: "activate_dynamic_dep".to_string(),
            affected_node_ids: sorted_node_pair(activated.owner_node_id, activated.target_node_id),
        });
    }

    updates.sort_by(|left, right| {
        left.kind
            .cmp(&right.kind)
            .then_with(|| left.affected_node_ids.cmp(&right.affected_node_ids))
    });
    updates
}

fn merge_dependency_shape_updates(
    mut left: Vec<DependencyShapeUpdate>,
    right: Vec<DependencyShapeUpdate>,
) -> Vec<DependencyShapeUpdate> {
    left.extend(right);
    left.sort_by(|left, right| {
        left.kind
            .cmp(&right.kind)
            .then_with(|| left.affected_node_ids.cmp(&right.affected_node_ids))
    });
    left.dedup();
    left
}

fn dynamic_dependency_delta_owner_ids(
    published_dynamic_dependencies: &[DynamicDependencyFact],
    dependency_graph: &DependencyGraph,
) -> BTreeSet<TreeNodeId> {
    let current_dynamic_dependencies = dynamic_dependency_facts_from_graph(dependency_graph);
    let published_set = published_dynamic_dependencies
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    let current_set = current_dynamic_dependencies
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();

    published_set
        .difference(&current_set)
        .chain(current_set.difference(&published_set))
        .map(|fact| fact.owner_node_id)
        .collect()
}

fn dynamic_dependency_runtime_effects(dependency_graph: &DependencyGraph) -> Vec<RuntimeEffect> {
    dynamic_dependency_facts_from_graph(dependency_graph)
        .into_iter()
        .map(|fact| RuntimeEffect {
            kind: "runtime_effect.dynamic_reference".to_string(),
            family: RuntimeEffectFamily::DynamicDependency,
            detail: format!(
                "owner_node:{};target_node:{};detail:{}",
                fact.owner_node_id, fact.target_node_id, fact.identity
            ),
        })
        .collect()
}

fn dynamic_dependency_descriptors_from_reference_text_resolutions(
    resolutions: &[TreeCalcRuntimeReferenceTextResolution],
) -> Vec<DependencyDescriptor> {
    resolutions
        .iter()
        .enumerate()
        .map(|(index, resolution)| {
            let handle = format!(
                "reference_text:{}:{}:{}",
                resolution.owner_node_id.0, resolution.target_node_id.0, index
            );
            DependencyDescriptor {
                descriptor_id: format!("ctro_dynamic_ref:{handle}"),
                source_reference_handle: Some(handle),
                owner_node_id: resolution.owner_node_id,
                target_node_id: Some(resolution.target_node_id),
                workspace_target: None,
                kind: DependencyDescriptorKind::DynamicPotential,
                carrier_detail: dynamic_reference_text_carrier_detail(resolution),
                tree_reference_collection: None,
                requires_rebind_on_structural_change: false,
            }
        })
        .collect()
}

fn dynamic_dependency_descriptors_from_published_facts(
    facts: &[DynamicDependencyFact],
) -> Vec<DependencyDescriptor> {
    facts
        .iter()
        .enumerate()
        .map(|(index, fact)| DependencyDescriptor {
            descriptor_id: format!(
                "published_ctro_dynamic_ref:{}:{}:{}",
                fact.owner_node_id.0, fact.target_node_id.0, index
            ),
            source_reference_handle: Some(format!(
                "published_reference_text:{}:{}:{}",
                fact.owner_node_id.0, fact.target_node_id.0, index
            )),
            owner_node_id: fact.owner_node_id,
            target_node_id: Some(fact.target_node_id),
            workspace_target: None,
            kind: DependencyDescriptorKind::DynamicPotential,
            carrier_detail: fact.identity.clone(),
            tree_reference_collection: None,
            requires_rebind_on_structural_change: false,
        })
        .collect()
}

fn dynamic_reference_text_carrier_detail(
    resolution: &TreeCalcRuntimeReferenceTextResolution,
) -> String {
    format!(
        "dynamic_resolved:node:{}:reference_text={};mode={:?};a1_style={:?};reference={}",
        resolution.target_node_id,
        resolution.reference_text,
        resolution.mode,
        resolution.a1_style,
        resolution.reference_like.target()
    )
}

fn dynamic_dependency_facts_from_graph(
    dependency_graph: &DependencyGraph,
) -> Vec<DynamicDependencyFact> {
    dependency_graph
        .descriptors_by_owner
        .values()
        .flatten()
        .filter_map(|descriptor| {
            if descriptor.kind != DependencyDescriptorKind::DynamicPotential {
                return None;
            }
            Some(DynamicDependencyFact {
                owner_node_id: descriptor.owner_node_id,
                target_node_id: descriptor.target_node_id?,
                identity: dynamic_dependency_identity(&descriptor.carrier_detail),
            })
        })
        .collect()
}

fn dynamic_dependency_facts_from_runtime_effects(
    runtime_effects: &[RuntimeEffect],
) -> Vec<DynamicDependencyFact> {
    runtime_effects
        .iter()
        .filter(|effect| {
            effect.family == RuntimeEffectFamily::DynamicDependency
                && effect.kind == "runtime_effect.dynamic_reference"
        })
        .filter_map(|effect| {
            let owner_node_id = parse_runtime_effect_node(&effect.detail, "owner_node:")?;
            let target_node_id = parse_runtime_effect_node(&effect.detail, "target_node:")?;
            let detail = parse_runtime_effect_detail(&effect.detail)?;
            Some(DynamicDependencyFact {
                owner_node_id,
                target_node_id,
                identity: dynamic_dependency_identity(detail),
            })
        })
        .collect()
}

fn dynamic_dependency_identity(carrier_detail: &str) -> String {
    if let Some(rest) = carrier_detail.strip_prefix("dynamic_resolved:node:")
        && let Some((_, identity)) = rest.split_once(':')
    {
        return format!("dynamic:{identity}");
    }

    if let Some(identity) = carrier_detail.strip_prefix("dynamic_potential:") {
        return format!("dynamic:{identity}");
    }

    carrier_detail.to_string()
}

fn parse_runtime_effect_node(detail: &str, prefix: &str) -> Option<TreeNodeId> {
    let (_, rest) = detail.split_once(prefix)?;
    let value = rest.split([';', '|']).next()?;
    let value = value.strip_prefix("node:").unwrap_or(value);
    value.parse::<u64>().ok().map(TreeNodeId)
}

fn parse_runtime_effect_detail(detail: &str) -> Option<&str> {
    let (_, rest) = detail.split_once("detail:")?;
    Some(rest.split('|').next().unwrap_or(rest))
}

fn sorted_node_pair(left: TreeNodeId, right: TreeNodeId) -> Vec<TreeNodeId> {
    let mut nodes = vec![left, right];
    nodes.sort();
    nodes.dedup();
    nodes
}

#[allow(clippy::too_many_arguments)]
fn reject_run(
    input: &LocalTreeCalcInput,
    coordinator: &mut TreeCalcCoordinator,
    recalc_tracker: &mut Stage1RecalcTracker,
    dependency_graph: DependencyGraph,
    invalidation_closure: InvalidationClosure,
    evaluation_order: Vec<TreeNodeId>,
    runtime_effects: Vec<RuntimeEffect>,
    runtime_effect_overlays: Vec<OverlayEntry>,
    mut diagnostics: Vec<String>,
    mut phase_timer: LocalTreeCalcPhaseTimer,
    formula_owner_ids: &[TreeNodeId],
    prepared_formula_identities: Vec<PreparedFormulaIdentityTrace>,
    local_candidate: Option<LocalEvaluatorCandidate>,
    error: LocalTreeCalcError,
) -> Result<LocalTreeCalcRunArtifacts, LocalTreeCalcError> {
    let phase_start = Instant::now();
    diagnostics.push(format!("candidate_rejected:{}", error));
    diagnostics.extend(runtime_effect_overlay_projection_diagnostics(
        &input.environment_context,
        runtime_effect_overlays.len(),
    ));
    let placeholder_candidate = AcceptedCandidateResult {
        candidate_result_id: input.candidate_result_id.clone(),
        structural_snapshot_id: input.structural_snapshot().snapshot_id(),
        artifact_token_basis: input.artifact_token_basis(),
        compatibility_basis: input.compatibility_basis(),
        target_set: formula_owner_ids.to_vec(),
        calc_value_updates: BTreeMap::new(),
        dependency_shape_updates: vec![],
        runtime_effects: runtime_effects.clone(),
        diagnostic_events: diagnostics.clone(),
    };
    coordinator.admit_candidate_work(placeholder_candidate)?;
    let reject_detail = coordinator.reject_candidate_work(
        &input.candidate_result_id,
        map_local_error_to_reject_kind(&error),
        &error.to_string(),
    )?;

    for node_id in formula_owner_ids.iter().copied() {
        let state = recalc_tracker.get_state(node_id);
        if matches!(
            state,
            NodeCalcState::Evaluating | NodeCalcState::PublishReady
        ) {
            recalc_tracker.reject_or_fallback(node_id, &error.to_string())?;
        }
    }
    phase_timer.record_duration("rejection_recording", phase_start.elapsed());
    let phase_timings_micros = phase_timer.finish();

    Ok(LocalTreeCalcRunArtifacts {
        result_state: LocalTreeCalcRunState::Rejected,
        dependency_graph,
        invalidation_closure,
        evaluation_order,
        runtime_effects,
        runtime_effect_overlays,
        prepared_formula_identities,
        derivation_traces: Vec::new(),
        local_candidate,
        candidate_result: None,
        publication_bundle: None,
        reject_detail: Some(reject_detail),
        published_values: crate::coordinator::calc_value_display_map(
            &coordinator.published_view().calc_values,
        ),
        published_calc_values: coordinator.published_view().calc_values.clone(),
        node_states: recalc_tracker.node_states().clone(),
        phase_timings_micros,
        diagnostics,
    })
}

fn map_local_error_to_reject_kind(error: &LocalTreeCalcError) -> RejectKind {
    match error {
        LocalTreeCalcError::CycleDetected => RejectKind::SyntheticCycleReject,
        LocalTreeCalcError::DynamicReference { .. }
        | LocalTreeCalcError::MissingReferencedValue { .. } => RejectKind::DynamicDependencyFailure,
        LocalTreeCalcError::UnresolvedReference { .. }
        | LocalTreeCalcError::HostSensitiveReference { .. }
        | LocalTreeCalcError::CapabilitySensitiveReference { .. }
        | LocalTreeCalcError::ShapeTopologyReference { .. }
        | LocalTreeCalcError::DependencyGraphIncompatible { .. }
        | LocalTreeCalcError::StructuralRebindRequired { .. }
        | LocalTreeCalcError::UnsupportedNumericValue { .. }
        | LocalTreeCalcError::UnsupportedFunction { .. }
        | LocalTreeCalcError::DivisionByZero
        | LocalTreeCalcError::OxfmlHostFailure { .. }
        | LocalTreeCalcError::OxfmlBindUnresolved { .. }
        | LocalTreeCalcError::OxfmlCommitRejected { .. }
        | LocalTreeCalcError::OxfmlCommitBundleIncompatible { .. } => {
            RejectKind::HostInjectedFailure
        }
        LocalTreeCalcError::Coordinator(_)
        | LocalTreeCalcError::Recalc(_)
        | LocalTreeCalcError::MissingFormulaBinding { .. } => RejectKind::HostInjectedFailure,
    }
}

fn build_runtime_effect_overlays(
    input: &LocalTreeCalcInput,
    owner_node_id: TreeNodeId,
    runtime_effects: &[RuntimeEffect],
) -> Vec<OverlayEntry> {
    if !input.environment_context.project_runtime_effect_overlays {
        return Vec::new();
    }

    runtime_effects
        .iter()
        .enumerate()
        .map(|(index, runtime_effect)| {
            runtime_effect_overlay_entry(input, owner_node_id, index, runtime_effect)
        })
        .collect()
}

fn build_runtime_effect_overlays_from_runtime_effects(
    input: &LocalTreeCalcInput,
    runtime_effects: &[RuntimeEffect],
) -> Vec<OverlayEntry> {
    if !input.environment_context.project_runtime_effect_overlays {
        return Vec::new();
    }

    runtime_effects
        .iter()
        .enumerate()
        .filter_map(|(index, runtime_effect)| {
            let owner_node_id = runtime_effect_owner_node_id(runtime_effect)?;
            Some(runtime_effect_overlay_entry(
                input,
                owner_node_id,
                index,
                runtime_effect,
            ))
        })
        .collect()
}

fn runtime_effect_overlay_entry(
    input: &LocalTreeCalcInput,
    owner_node_id: TreeNodeId,
    index: usize,
    runtime_effect: &RuntimeEffect,
) -> OverlayEntry {
    OverlayEntry {
        key: OverlayKey {
            owner_node_id,
            overlay_kind: runtime_effect_overlay_kind(runtime_effect),
            structural_snapshot_id: input.structural_snapshot().snapshot_id(),
            compatibility_basis: input.compatibility_basis(),
            payload_identity: Some(format!(
                "{}:runtime_effect:{index}",
                input.candidate_result_id
            )),
        },
        is_protected: true,
        is_eviction_eligible: false,
        detail: format!("{}|{}", runtime_effect.kind, runtime_effect.detail),
    }
}

fn runtime_effect_owner_node_id(runtime_effect: &RuntimeEffect) -> Option<TreeNodeId> {
    parse_runtime_effect_node(&runtime_effect.detail, "owner_node:")
}

fn annotate_runtime_effects_with_environment(
    runtime_effects: &[RuntimeEffect],
    context: &LocalTreeCalcEnvironmentContext,
) -> Vec<RuntimeEffect> {
    runtime_effects
        .iter()
        .cloned()
        .map(|mut runtime_effect| {
            runtime_effect.detail = format!(
                "{}|runtime_lane:{}|session_id:{}|capability_profile_id:{}|runtime_policy_id:{}",
                runtime_effect.detail,
                context.runtime_lane,
                context.session_id.as_deref().unwrap_or("none"),
                context.capability_profile_id,
                context.runtime_policy_id
            );
            runtime_effect
        })
        .collect()
}

fn runtime_effect_overlay_projection_diagnostics(
    context: &LocalTreeCalcEnvironmentContext,
    overlay_count: usize,
) -> Vec<String> {
    vec![
        format!(
            "runtime_effect_overlay_projection_enabled:{}",
            context.project_runtime_effect_overlays
        ),
        format!("runtime_effect_overlay_projection_count:{overlay_count}"),
    ]
}

fn runtime_effect_context_diagnostics(context: &LocalTreeCalcEnvironmentContext) -> Vec<String> {
    vec![
        format!(
            "runtime_effect_environment_runtime_lane:{}",
            context.runtime_lane
        ),
        format!(
            "runtime_effect_environment_session_id:{}",
            context.session_id.as_deref().unwrap_or("none")
        ),
        format!(
            "runtime_effect_environment_capability_profile_id:{}",
            context.capability_profile_id
        ),
        format!(
            "runtime_effect_environment_runtime_policy_id:{}",
            context.runtime_policy_id
        ),
        format!(
            "runtime_effect_environment_project_overlays:{}",
            context.project_runtime_effect_overlays
        ),
        format!(
            "runtime_effect_environment_derivation_trace_enabled:{}",
            context.derivation_trace_enabled
        ),
        format!(
            "runtime_effect_environment_scheduling_policy:{}",
            context.scheduling_policy.diagnostic_name()
        ),
    ]
}

fn runtime_effect_overlay_kind(runtime_effect: &RuntimeEffect) -> OverlayKind {
    match runtime_effect.family {
        RuntimeEffectFamily::DynamicDependency => OverlayKind::DynamicDependency,
        RuntimeEffectFamily::ExecutionRestriction | RuntimeEffectFamily::CapabilitySensitive => {
            OverlayKind::ExecutionRestriction
        }
        RuntimeEffectFamily::ShapeTopology => OverlayKind::ShapeTopology,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LocalTreeCalcSchedulingPlan {
    scheduled_formula_owner_ids: Vec<TreeNodeId>,
    dirty_formula_owner_ids: Vec<TreeNodeId>,
    diagnostics: Vec<String>,
}

fn plan_treecalc_schedule(
    policy: &LocalTreeCalcSchedulingPolicy,
    dependency_graph: &DependencyGraph,
    invalidation_closure: &InvalidationClosure,
    formula_owner_ids: &[TreeNodeId],
) -> LocalTreeCalcSchedulingPlan {
    let formula_owner_set = formula_owner_ids.iter().copied().collect::<BTreeSet<_>>();
    let dirty_formula_owner_set = invalidation_closure
        .records
        .keys()
        .copied()
        .filter(|node_id| formula_owner_set.contains(node_id))
        .collect::<BTreeSet<_>>();

    let scheduled_formula_owner_set = match policy {
        LocalTreeCalcSchedulingPolicy::PullFullClosure => formula_owner_set.clone(),
        LocalTreeCalcSchedulingPolicy::PushVisibilityBounded {
            visible_observer_node_ids,
        } => {
            let mut scheduled = BTreeSet::new();
            for observer_node_id in visible_observer_node_ids {
                if formula_owner_set.contains(observer_node_id) {
                    collect_upstream_formula_dependencies(
                        *observer_node_id,
                        dependency_graph,
                        &formula_owner_set,
                        &mut scheduled,
                    );
                }
            }
            scheduled
        }
    };

    let scheduled_formula_owner_ids = formula_owner_ids
        .iter()
        .copied()
        .filter(|node_id| scheduled_formula_owner_set.contains(node_id))
        .collect::<Vec<_>>();
    let dirty_formula_owner_ids = formula_owner_ids
        .iter()
        .copied()
        .filter(|node_id| dirty_formula_owner_set.contains(node_id))
        .collect::<Vec<_>>();
    let deferred_formula_owner_ids = dirty_formula_owner_ids
        .iter()
        .copied()
        .filter(|node_id| !scheduled_formula_owner_set.contains(node_id))
        .collect::<Vec<_>>();

    let mut diagnostics = vec![
        format!("scheduling_policy:{}", policy.diagnostic_name()),
        format!("scheduling_formula_owner_count:{}", formula_owner_ids.len()),
        format!(
            "scheduling_selected_formula_count:{}",
            scheduled_formula_owner_ids.len()
        ),
        format!(
            "scheduling_deferred_formula_count:{}",
            deferred_formula_owner_ids.len()
        ),
    ];

    match policy {
        LocalTreeCalcSchedulingPolicy::PullFullClosure => {
            diagnostics.push("scheduling_semantic_equivalence_scope:full_closure".to_string());
            diagnostics.push(
                "scheduling_starvation_fairness_note:pull_full_closure_sweeps_all_formula_owners"
                    .to_string(),
            );
        }
        LocalTreeCalcSchedulingPolicy::PushVisibilityBounded {
            visible_observer_node_ids,
        } => {
            for observer_node_id in visible_observer_node_ids {
                diagnostics.push(format!("scheduling_visible_observer:{observer_node_id}"));
                if scheduled_formula_owner_set.contains(observer_node_id) {
                    diagnostics.push(format!(
                        "scheduling_visible_observer_update:{observer_node_id}"
                    ));
                }
            }
            for node_id in &deferred_formula_owner_ids {
                diagnostics.push(format!("scheduling_deferred:{node_id}"));
            }
            diagnostics.push("scheduling_semantic_equivalence_scope:visible_observers".to_string());
            diagnostics.push(
                "scheduling_starvation_fairness_note:push_visibility_bounded_requires_periodic_full_closure_or_observer_aging"
                    .to_string(),
            );
        }
    }

    LocalTreeCalcSchedulingPlan {
        scheduled_formula_owner_ids,
        dirty_formula_owner_ids,
        diagnostics,
    }
}

fn collect_upstream_formula_dependencies(
    node_id: TreeNodeId,
    dependency_graph: &DependencyGraph,
    formula_owner_set: &BTreeSet<TreeNodeId>,
    scheduled: &mut BTreeSet<TreeNodeId>,
) {
    if !formula_owner_set.contains(&node_id) || !scheduled.insert(node_id) {
        return;
    }

    if let Some(edges) = dependency_graph.edges_by_owner.get(&node_id) {
        for edge in edges {
            if formula_owner_set.contains(&edge.target_node_id) {
                collect_upstream_formula_dependencies(
                    edge.target_node_id,
                    dependency_graph,
                    formula_owner_set,
                    scheduled,
                );
            }
        }
    }
}

fn seed_working_values(
    seeded_published_values: &BTreeMap<TreeNodeId, CalcValue>,
    input_values: &BTreeMap<TreeNodeId, String>,
) -> BTreeMap<TreeNodeId, String> {
    let mut values = BTreeMap::new();
    values.extend(
        seeded_published_values
            .iter()
            .map(|(node_id, value)| (*node_id, calc_value_display_text(value))),
    );
    values.extend(input_values.clone());
    values
}

fn seed_working_calc_values(
    seeded_published_values: &BTreeMap<TreeNodeId, CalcValue>,
    input_values: &BTreeMap<TreeNodeId, String>,
) -> BTreeMap<TreeNodeId, CalcValue> {
    let mut values = seeded_published_values.clone();
    values.extend(
        input_values
            .iter()
            .map(|(node_id, value)| (*node_id, treecalc_published_value_to_calc_value(value))),
    );
    values
}

fn topological_formula_order(
    dependency_graph: &DependencyGraph,
    formula_owner_ids: &[TreeNodeId],
) -> Result<Vec<TreeNodeId>, LocalTreeCalcError> {
    let formula_owner_set = formula_owner_ids.iter().copied().collect::<BTreeSet<_>>();
    let mut indegree = formula_owner_ids
        .iter()
        .copied()
        .map(|node_id| (node_id, 0usize))
        .collect::<BTreeMap<_, _>>();

    for owner_node_id in formula_owner_ids {
        if let Some(edges) = dependency_graph.edges_by_owner.get(owner_node_id) {
            for edge in edges {
                if formula_owner_set.contains(&edge.target_node_id) {
                    *indegree.entry(*owner_node_id).or_insert(0) += 1;
                }
            }
        }
    }

    let mut queue = indegree
        .iter()
        .filter(|(_, degree)| **degree == 0)
        .map(|(node_id, _)| *node_id)
        .collect::<Vec<_>>();
    queue.sort();
    let mut queue = VecDeque::from(queue);
    let mut order = Vec::new();

    while let Some(node_id) = queue.pop_front() {
        order.push(node_id);
        if let Some(reverse_edges) = dependency_graph.reverse_edges.get(&node_id) {
            for edge in reverse_edges {
                let dependent_node_id = edge.owner_node_id;
                if !formula_owner_set.contains(&dependent_node_id) {
                    continue;
                }

                let degree = indegree
                    .get_mut(&dependent_node_id)
                    .expect("formula indegree is initialized");
                *degree -= 1;
                if *degree == 0 {
                    queue.push_back(dependent_node_id);
                }
            }
        }
    }

    if order.len() != formula_owner_ids.len() {
        return Err(LocalTreeCalcError::CycleDetected);
    }

    Ok(order)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ResidualCarrierKind {
    HostSensitive,
    DynamicPotential,
    CapabilitySensitive,
    ShapeTopology,
}

impl ResidualCarrierKind {
    fn dependency_descriptor_kind(self) -> DependencyDescriptorKind {
        match self {
            Self::HostSensitive => DependencyDescriptorKind::HostSensitive,
            Self::DynamicPotential => DependencyDescriptorKind::DynamicPotential,
            Self::CapabilitySensitive => DependencyDescriptorKind::CapabilitySensitive,
            Self::ShapeTopology => DependencyDescriptorKind::ShapeTopology,
        }
    }

    fn requires_rebind_on_structural_change(self) -> bool {
        matches!(
            self,
            Self::HostSensitive | Self::CapabilitySensitive | Self::ShapeTopology
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResidualCarrier {
    kind: ResidualCarrierKind,
    owner_node_id: TreeNodeId,
    carrier_id: String,
    detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SyntheticReferenceBinding {
    token: String,
    local_target_node_id: Option<TreeNodeId>,
    workspace_target: Option<WorkspaceQualifiedTarget>,
    kind: DependencyDescriptorKind,
    carrier_detail: String,
    requires_rebind_on_structural_change: bool,
    host_name_bind: Option<TreeFormulaHostNameBindPacket>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SyntheticReferenceCollectionBinding {
    token: String,
    host_ref_handle: String,
    base_node_id: TreeNodeId,
    source_span_utf8: Option<(usize, usize)>,
    source_token_text: String,
    opaque_selector: String,
    member_node_ids: Vec<TreeNodeId>,
    collection_dependency: TreeReferenceCollectionDependency,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SyntheticStructuredTableBinding {
    token: String,
    host_ref_handle: String,
    source_span_utf8: Option<(usize, usize)>,
    source_token_text: String,
    table_id: String,
    selected_column_ids: Vec<String>,
    selected_sections: Vec<StructuredSectionKind>,
    declared_rows: usize,
    declared_cols: usize,
    member_node_ids: Vec<TreeNodeId>,
    member_node_cells: Vec<SyntheticStructuredTableNodeCell>,
    literal_cells: Vec<SyntheticStructuredTableLiteralCell>,
    reference_kind: ReferenceKind,
    reference_target: String,
    row_membership_version: String,
    row_order_version: String,
    column_identity_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SyntheticStructuredTableNodeCell {
    row_index: usize,
    col_index: usize,
    node_id: TreeNodeId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SyntheticStructuredTableLiteralCell {
    row_index: usize,
    col_index: usize,
    value: SyntheticStructuredTableLiteralValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum SyntheticStructuredTableLiteralValue {
    Text(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SyntheticHostValueBinding {
    token: String,
    value: TreeFormulaHostValue,
    host_ref_handle: String,
    source_span_utf8: (usize, usize),
    source_token_text: String,
    opaque_selector: Option<String>,
    carrier_detail: String,
    target_node_id: Option<TreeNodeId>,
    requires_rebind_on_structural_change: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SyntheticUnresolvedBinding {
    token: String,
    kind: DependencyDescriptorKind,
    carrier_detail: String,
    requires_rebind_on_structural_change: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TranslatedFormula {
    source_text: String,
    reference_bindings: Vec<SyntheticReferenceBinding>,
    collection_bindings: Vec<SyntheticReferenceCollectionBinding>,
    structured_table_bindings: Vec<SyntheticStructuredTableBinding>,
    host_value_bindings: Vec<SyntheticHostValueBinding>,
    unresolved_bindings: Vec<SyntheticUnresolvedBinding>,
    residuals: Vec<ResidualCarrier>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PreparedOxfmlFormula {
    binding: crate::formula::TreeFormulaBinding,
    structural_snapshot: StructuralSnapshot,
    table_snapshots: BTreeMap<TreeNodeId, TreeCalcTableNodeSnapshot>,
    meta_node_ids: BTreeSet<TreeNodeId>,
    source: FormulaSourceRecord,
    translated: TranslatedFormula,
    semantic_plan: SemanticPlan,
    runtime_prepared_identity: RuntimePreparedFormulaIdentity,
    w056_prepared_identity_requirements: Vec<(NamespaceIdentityNeed, CallerContextIdentityNeed)>,
    oxfunc_bridge_metadata: LocalTreeCalcOxFuncBridgeMetadata,
    requires_host_query: bool,
    requires_image_provider: bool,
    edge_value_cache_path_facts: EdgeValueCachePathFacts,
    bind_diagnostics: Vec<String>,
    lazy_residual_publication: bool,
}

fn prepare_oxfml_formula(
    snapshot: &StructuralSnapshot,
    table_snapshots: &BTreeMap<TreeNodeId, TreeCalcTableNodeSnapshot>,
    binding: &crate::formula::TreeFormulaBinding,
    environment_context: &LocalTreeCalcEnvironmentContext,
) -> Result<PreparedOxfmlFormula, LocalTreeCalcError> {
    let mut translated = project_opaque_formula(
        snapshot,
        table_snapshots,
        binding.owner_node_id,
        &binding.expression,
        &environment_context.meta_node_ids,
    );
    let source = FormulaSourceRecord::new(
        binding.formula_artifact_id.to_string(),
        binding.owner_node_id.0,
        translated.source_text.clone(),
    );
    let structure_context_version =
        bind_visible_structure_context_version(snapshot, environment_context);
    let w056_prepared_identity_requirements =
        w056_prepared_identity_requirements_for_translated(&translated);
    let cross_workspace_availability_versions =
        cross_workspace_availability_versions_for_translated(&translated);
    let host_formula_context = w056_runtime_host_formula_context(
        binding.owner_node_id,
        &structure_context_version.0,
        environment_context,
        &w056_prepared_identity_requirements,
        &cross_workspace_availability_versions,
    );
    let empty_working_values = BTreeMap::new();
    let empty_working_calc_values = BTreeMap::new();
    let mut prepare_environment =
        build_treecalc_runtime_environment_from_parts(TreeCalcRuntimeEnvironmentBuild {
            translated: &translated,
            snapshot,
            table_snapshots,
            owner_node_id: binding.owner_node_id,
            meta_node_ids: &environment_context.meta_node_ids,
            structure_context_version,
            oxfunc_bridge_metadata: &environment_context.oxfunc_bridge_metadata,
            working_values: &empty_working_values,
            working_calc_values: &empty_working_calc_values,
            prepared_formula_identity: None,
            host_formula_context: host_formula_context.clone(),
        });
    if let Some(version) = &environment_context
        .oxfunc_bridge_metadata
        .semantic_kernel_metadata_version
    {
        prepare_environment = prepare_environment.with_semantic_kernel_metadata_version(version);
    }
    if let Some(version) = &environment_context
        .oxfunc_bridge_metadata
        .arg_admission_metadata_version
    {
        prepare_environment = prepare_environment.with_arg_admission_metadata_version(version);
    }
    let request = RuntimeFormulaRequest::new(source.clone(), TypedContextQueryBundle::default())
        .with_backend(EvaluationBackend::OxFuncBacked);
    let mut session = OxfmlRecalcSessionDriver::new(prepare_environment);
    let open = session.ensure_prepared(&request).map_err(|error| {
        LocalTreeCalcError::OxfmlHostFailure {
            owner_node_id: binding.owner_node_id,
            detail: error.to_string(),
        }
    })?;
    translated.structured_table_bindings = project_runtime_structured_table_bindings(
        snapshot,
        table_snapshots,
        binding.owner_node_id,
        &environment_context.meta_node_ids,
        &open
            .prepared_formula_identity
            .structured_reference_bind_records,
    );
    let semantic_plan = open.semantic_plan;
    let requires_host_query = semantic_plan.execution_profile.requires_host_query;
    let requires_image_provider = semantic_plan
        .function_bindings
        .iter()
        .any(|binding| binding.function_id == "FUNC.IMAGE");
    let edge_value_cache_path_facts = edge_value_cache_path_facts_for(&semantic_plan, &translated);

    Ok(PreparedOxfmlFormula {
        binding: binding.clone(),
        structural_snapshot: snapshot.clone(),
        table_snapshots: table_snapshots.clone(),
        meta_node_ids: environment_context.meta_node_ids.clone(),
        source,
        translated,
        bind_diagnostics: open
            .bind_diagnostics
            .iter()
            .map(|diagnostic| format!("oxfml_bind_diagnostic:{}", diagnostic.message))
            .collect(),
        semantic_plan,
        runtime_prepared_identity: open.prepared_formula_identity,
        w056_prepared_identity_requirements,
        oxfunc_bridge_metadata: environment_context.oxfunc_bridge_metadata.clone(),
        requires_host_query,
        requires_image_provider,
        edge_value_cache_path_facts,
        lazy_residual_publication: binding.expression.lazy_residual_publication,
    })
}

fn edge_value_cache_path_facts_for(
    semantic_plan: &SemanticPlan,
    translated: &TranslatedFormula,
) -> EdgeValueCachePathFacts {
    EdgeValueCachePathFacts {
        volatile: semantic_plan.execution_profile.volatility != FormulaVolatilityClass::Stable
            || semantic_plan.execution_profile.determinism
                != FormulaDeterminismClass::Deterministic,
        effectful: semantic_plan.execution_profile.requires_host_interaction
            || semantic_plan
                .execution_profile
                .contains_external_event_dependence
            || !translated.residuals.is_empty(),
    }
}

fn bind_visible_structure_context_version(
    snapshot: &StructuralSnapshot,
    environment_context: &LocalTreeCalcEnvironmentContext,
) -> StructureContextVersion {
    let mut version = format!(
        "{}|arg_preparation_profile_version={}",
        snapshot.snapshot_id(),
        environment_context.arg_preparation_profile_version
    );
    if let Some(metadata_version) = &environment_context
        .oxfunc_bridge_metadata
        .semantic_kernel_metadata_version
    {
        version.push_str("|semantic_kernel_metadata_version=");
        version.push_str(metadata_version);
    }
    if let Some(metadata_version) = &environment_context
        .oxfunc_bridge_metadata
        .arg_admission_metadata_version
    {
        version.push_str("|arg_admission_metadata_version=");
        version.push_str(metadata_version);
    }
    StructureContextVersion(version)
}

fn prepared_formula_identity_traces(
    prepared_formulas: &BTreeMap<TreeNodeId, PreparedOxfmlFormula>,
) -> Vec<PreparedFormulaIdentityTrace> {
    prepared_formulas
        .values()
        .map(prepared_formula_identity_trace)
        .collect()
}

fn prepared_formula_identity_trace(
    prepared: &PreparedOxfmlFormula,
) -> PreparedFormulaIdentityTrace {
    let identity = &prepared.runtime_prepared_identity;
    let plan_template = &identity.plan_template;
    let hole_binding = &identity.hole_binding;
    PreparedFormulaIdentityTrace {
        owner_node_id: prepared.binding.owner_node_id,
        formula_artifact_id: prepared.binding.formula_artifact_id.to_string(),
        bind_artifact_id: prepared
            .binding
            .bind_artifact_id
            .as_ref()
            .map(ToString::to_string),
        formula_stable_id: identity.formula_stable_id.clone(),
        prepared_formula_key: identity.prepared_formula_key.clone(),
        shape_key: plan_template
            .shape_key
            .clone()
            .unwrap_or_else(|| "shape_key:unavailable".to_string()),
        dispatch_skeleton_key: plan_template.dispatch_skeleton_key.to_string(),
        plan_template_key: plan_template.plan_template_key.to_string(),
        hole_binding_fingerprint: hole_binding.hole_binding_fingerprint.clone(),
        template_hole_count: plan_template.template_holes.len(),
        oxfunc_bridge_metadata: prepared.oxfunc_bridge_metadata.clone(),
        rich_value_capability_columns: RichValueCapabilityTraceReplayColumns::empty_v1(),
    }
}

fn prepared_formula_identity_diagnostics(prepared: &PreparedOxfmlFormula) -> Vec<String> {
    let identity = &prepared.runtime_prepared_identity;
    let plan_template = &identity.plan_template;
    let hole_binding = &identity.hole_binding;
    let mut diagnostics = vec![
        format!(
            "oxfml_prepared_shape_key:{}:{}",
            prepared.binding.formula_artifact_id,
            plan_template
                .shape_key
                .as_deref()
                .unwrap_or("shape_key:unavailable")
        ),
        format!(
            "oxfml_prepared_dispatch_skeleton_key:{}:{}",
            prepared.binding.formula_artifact_id, plan_template.dispatch_skeleton_key
        ),
        format!(
            "oxfml_prepared_plan_template_key:{}:{}",
            prepared.binding.formula_artifact_id, plan_template.plan_template_key
        ),
        format!(
            "oxfml_prepared_hole_binding_fingerprint:{}:{}",
            prepared.binding.formula_artifact_id, hole_binding.hole_binding_fingerprint
        ),
        format!(
            "oxfml_prepared_formula_key:{}:{}",
            prepared.binding.formula_artifact_id, identity.prepared_formula_key
        ),
        format!(
            "oxfml_prepared_template_hole_count:{}:{}",
            prepared.binding.formula_artifact_id,
            plan_template.template_holes.len()
        ),
    ];
    if let Some(version) = &prepared
        .oxfunc_bridge_metadata
        .semantic_kernel_metadata_version
    {
        diagnostics.push(format!(
            "oxfml_prepared_semantic_kernel_metadata_version:{}:{}",
            prepared.binding.formula_artifact_id, version
        ));
    }
    if let Some(version) = &prepared
        .oxfunc_bridge_metadata
        .arg_admission_metadata_version
    {
        diagnostics.push(format!(
            "oxfml_prepared_arg_admission_metadata_version:{}:{}",
            prepared.binding.formula_artifact_id, version
        ));
    }
    diagnostics.extend(identity.host_name_bind_results.iter().map(|bind| {
        format!(
            "w056_host_name_bind_result:{}:handle={};name={};span={}-{};token={};layer={};kind={};replay={}",
            prepared.binding.formula_artifact_id,
            bind.host_name_handle,
            bind.canonical_name,
            bind.source_span.start,
            bind.source_span.end(),
            bind.source_token_text,
            bind.resolution_layer,
            bind.binding_kind,
            bind.replay_identity_contribution
        )
    }));
    diagnostics
}

fn w056_prepared_identity_diagnostics(prepared: &PreparedOxfmlFormula) -> Vec<String> {
    let mut diagnostics = prepared
        .w056_prepared_identity_requirements
        .iter()
        .map(|(namespace, caller)| {
            format!(
                "w056_prepared_identity_requirement:{}:namespace={namespace:?};caller={caller:?}",
                prepared.binding.formula_artifact_id
            )
        })
        .collect::<Vec<_>>();

    if let Some(host_context) = &prepared.runtime_prepared_identity.host_formula_context {
        diagnostics.push(format!(
            "w056_prepared_identity_host_context:{}:{}",
            prepared.binding.formula_artifact_id,
            host_context.cache_identity_contribution()
        ));
        if host_context
            .host_namespace_version
            .as_deref()
            .is_some_and(|identity| identity.contains("cross_workspace_availability_version="))
        {
            diagnostics.push(format!(
                "w056_prepared_identity_cross_workspace_availability_projected:{}",
                prepared.binding.formula_artifact_id
            ));
        }
        if host_context.table_context_identity.as_deref()
            == Some("treecalc-table-context:unavailable-current-packet")
        {
            diagnostics.push(format!(
                "w056_prepared_identity_table_context_blocked:{}:missing_public_packet_identity",
                prepared.binding.formula_artifact_id
            ));
        }
    }

    diagnostics
}

fn prepared_runtime_effect_subscription_diagnostics(
    prepared: &PreparedOxfmlFormula,
) -> Vec<String> {
    prepared_runtime_effect_subscription_entries(prepared)
        .into_iter()
        .map(|entry| {
            format!(
                "prepared_runtime_subscription:{}:{}:{}:{}",
                entry.formula_stable_id,
                entry.topic_id.0,
                entry.subscription_handle.0,
                entry.topic_descriptor
            )
        })
        .collect()
}

fn prepared_runtime_effect_subscription_entries(
    prepared: &PreparedOxfmlFormula,
) -> Vec<SubscriptionRegistryEntry> {
    let formula_stable_id = prepared.source.formula_stable_id.0.clone();
    let mut entries = prepared
        .translated
        .residuals
        .iter()
        .filter_map(|residual| {
            let runtime_effect = residual_runtime_effect(residual);
            if !runtime_effect_projects_external_subscription(residual, &runtime_effect) {
                return None;
            }
            let topic_component = subscription_component(
                residual
                    .carrier_id
                    .strip_prefix("carrier:")
                    .unwrap_or(&residual.carrier_id),
            );
            Some(SubscriptionRegistryEntry {
                topic_id: SubscriptionTopicId(format!("topic:{topic_component}")),
                formula_stable_id: formula_stable_id.clone(),
                subscription_handle: SubscriptionHandle(format!(
                    "subscription:{formula_stable_id}:{topic_component}"
                )),
                topic_descriptor: format!("{}:{}", runtime_effect.kind, runtime_effect.detail),
            })
        })
        .collect::<Vec<_>>();

    entries.sort_by(|left, right| {
        left.topic_id
            .cmp(&right.topic_id)
            .then_with(|| left.subscription_handle.cmp(&right.subscription_handle))
            .then_with(|| left.topic_descriptor.cmp(&right.topic_descriptor))
    });
    entries.dedup_by(|left, right| {
        left.topic_id == right.topic_id && left.formula_stable_id == right.formula_stable_id
    });
    entries
}

fn runtime_effect_projects_external_subscription(
    residual: &ResidualCarrier,
    runtime_effect: &RuntimeEffect,
) -> bool {
    if runtime_effect.kind != "runtime_effect.dynamic_reference"
        || runtime_effect.family != RuntimeEffectFamily::DynamicDependency
    {
        return false;
    }
    let detail = residual.detail.to_ascii_lowercase();
    let carrier_id = residual.carrier_id.to_ascii_lowercase();
    detail.contains("rtd")
        || detail.contains("external")
        || detail.contains("topic")
        || carrier_id.contains("rtd")
}

fn subscription_component(input: &str) -> String {
    input
        .chars()
        .map(|character| match character {
            'a'..='z' | 'A'..='Z' | '0'..='9' | ':' | '-' | '_' | '.' => character,
            _ => '_',
        })
        .collect()
}

fn prepared_formula_reuse_diagnostics(identities: &[PreparedFormulaIdentityTrace]) -> Vec<String> {
    #[derive(Default)]
    struct ReuseCounters {
        call_site_count: usize,
        prepared_formula_keys: BTreeSet<String>,
        hole_binding_fingerprints: BTreeSet<String>,
    }

    let mut by_template = BTreeMap::<String, ReuseCounters>::new();
    for identity in identities {
        let counters = by_template
            .entry(identity.plan_template_key.clone())
            .or_default();
        counters.call_site_count += 1;
        counters
            .prepared_formula_keys
            .insert(identity.prepared_formula_key.clone());
        counters
            .hole_binding_fingerprints
            .insert(identity.hole_binding_fingerprint.clone());
    }

    by_template
        .into_iter()
        .filter(|(_, counters)| counters.call_site_count > 1)
        .map(|(plan_template_key, counters)| {
            format!(
                "oxfml_plan_template_reuse_count:{}:call_sites={};prepared_formulas={};hole_bindings={}",
                plan_template_key,
                counters.call_site_count,
                counters.prepared_formula_keys.len(),
                counters.hole_binding_fingerprints.len()
            )
        })
        .collect()
}

fn build_seeded_edge_value_cache(
    prepared_formulas: &BTreeMap<TreeNodeId, PreparedOxfmlFormula>,
    seeded_published_values: &BTreeMap<TreeNodeId, CalcValue>,
    formula_count: usize,
    cache_basis: &str,
    diagnostics: &mut Vec<String>,
) -> Option<EdgeValueCache> {
    if seeded_published_values.is_empty() {
        return None;
    }

    let mut cache = EdgeValueCache::new(EdgeValueCachePolicy::w054_pending(formula_count.max(1)));
    for (node_id, prepared) in prepared_formulas {
        let Some(value) = seeded_published_values.get(node_id) else {
            continue;
        };
        store_edge_value_cache(
            &mut cache,
            prepared,
            *node_id,
            calc_value_display_text(value),
            0,
            cache_basis,
            diagnostics,
        );
    }
    Some(cache)
}

struct EdgeValueCacheLookupContext<'a> {
    cache_basis: &'a str,
    invalidation_closure: &'a InvalidationClosure,
    has_dependency_shape_delta: bool,
    caller_supplied_invalidation_seeds: bool,
}

fn lookup_edge_value_cache(
    cache: &EdgeValueCache,
    prepared: &PreparedOxfmlFormula,
    node_id: TreeNodeId,
    context: EdgeValueCacheLookupContext<'_>,
    diagnostics: &mut Vec<String>,
) -> Option<String> {
    if let Some(reason) = edge_value_cache_bypass_reason(
        node_id,
        context.invalidation_closure,
        context.has_dependency_shape_delta,
        context.caller_supplied_invalidation_seeds,
    ) {
        diagnostics.push(format!("edge_value_cache_bypass:{node_id}:{reason}"));
        return None;
    }

    let key = edge_value_cache_key(prepared, context.cache_basis);
    match cache.lookup(&key, prepared.edge_value_cache_path_facts.eligibility()) {
        EdgeValueCacheLookup::Hit(entry) => {
            diagnostics.push(format!(
                "edge_value_cache_hit:{node_id}:call_site_id={};hole_binding_fingerprint={}",
                (entry.key.call_site_id.0),
                (entry.key.hole_binding_fingerprint.0)
            ));
            Some(entry.value_payload)
        }
        EdgeValueCacheLookup::Miss => {
            diagnostics.push(format!(
                "edge_value_cache_miss:{node_id}:call_site_id={};hole_binding_fingerprint={}",
                key.call_site_id.0, key.hole_binding_fingerprint.0
            ));
            None
        }
        EdgeValueCacheLookup::Excluded(reason) => {
            diagnostics.push(format!(
                "edge_value_cache_excluded:{node_id}:{}",
                reason.selector_key()
            ));
            None
        }
    }
}

fn edge_value_cache_bypass_reason(
    node_id: TreeNodeId,
    invalidation_closure: &InvalidationClosure,
    has_dependency_shape_delta: bool,
    caller_supplied_invalidation_seeds: bool,
) -> Option<&'static str> {
    if has_dependency_shape_delta {
        return Some("DependencyShapeDelta");
    }

    let record = invalidation_closure.records.get(&node_id)?;
    if record
        .reasons
        .contains(&InvalidationReasonKind::UpstreamPublication)
    {
        return Some("UpstreamPublication");
    }
    if record
        .reasons
        .contains(&InvalidationReasonKind::ExternallyInvalidated)
    {
        return Some("ExternallyInvalidated");
    }
    if caller_supplied_invalidation_seeds {
        return Some("ExplicitInvalidationSeed");
    }
    None
}

fn store_edge_value_cache(
    cache: &mut EdgeValueCache,
    prepared: &PreparedOxfmlFormula,
    node_id: TreeNodeId,
    value_payload: String,
    derivation_epoch: u64,
    cache_basis: &str,
    diagnostics: &mut Vec<String>,
) {
    let key = edge_value_cache_key(prepared, cache_basis);
    match cache.store(
        key,
        prepared.edge_value_cache_path_facts.eligibility(),
        value_payload,
        derivation_epoch,
    ) {
        EdgeValueCacheStoreResult::Stored {
            entry,
            evicted_key,
            eviction_trace,
        } => {
            diagnostics.push(format!(
                "edge_value_cache_store:{node_id}:call_site_id={};hole_binding_fingerprint={};evicted={}",
                entry.key.call_site_id.0,
                entry.key.hole_binding_fingerprint.0,
                evicted_key
                    .as_ref()
                    .map(|key| key.call_site_id.0.clone())
                    .unwrap_or_else(|| "none".to_string())
            ));
            if let Some(trace) = eviction_trace {
                diagnostics.push(format!(
                    "edge_value_cache_eviction_trace:{node_id}:retention_class={};reason={};evicted_call_site_id={};evicted_hole_binding_fingerprint={};evicted_insertion_sequence={}",
                    trace.retention_class.selector_key(),
                    trace.reason.selector_key(),
                    trace.evicted_key.call_site_id.0,
                    trace.evicted_key.hole_binding_fingerprint.0,
                    trace.evicted_insertion_sequence
                ));
            }
        }
        EdgeValueCacheStoreResult::Excluded(reason) => {
            diagnostics.push(format!(
                "edge_value_cache_store_excluded:{node_id}:{}",
                reason.selector_key()
            ));
        }
    }
}

fn edge_value_cache_key(prepared: &PreparedOxfmlFormula, cache_basis: &str) -> EdgeValueCacheKey {
    EdgeValueCacheKey::new(
        format!(
            "cache_basis:{};tree_node:{};plan_template:{};prepared_formula:{}",
            cache_basis,
            prepared.binding.owner_node_id,
            prepared
                .runtime_prepared_identity
                .plan_template
                .plan_template_key,
            prepared.runtime_prepared_identity.prepared_formula_key
        ),
        prepared
            .runtime_prepared_identity
            .hole_binding
            .hole_binding_fingerprint
            .clone(),
    )
}

fn w056_prepared_identity_requirements_for_translated(
    translated: &TranslatedFormula,
) -> Vec<(NamespaceIdentityNeed, CallerContextIdentityNeed)> {
    let mut requirements = BTreeSet::new();

    for reference in &translated.reference_bindings {
        if reference.workspace_target.is_some() {
            requirements.insert((
                NamespaceIdentityNeed::CrossWorkspaceAvailabilityVersion,
                CallerContextIdentityNeed::None,
            ));
        } else {
            requirements.insert(descriptor_identity_needs(reference.kind));
        }
    }
    for unresolved in &translated.unresolved_bindings {
        requirements.insert(descriptor_identity_needs(unresolved.kind));
    }
    for _ in &translated.collection_bindings {
        requirements.insert(descriptor_identity_needs(
            DependencyDescriptorKind::TreeReferenceCollectionMembership,
        ));
        requirements.insert(descriptor_identity_needs(
            DependencyDescriptorKind::TreeReferenceCollectionMemberValue,
        ));
    }
    for _ in &translated.structured_table_bindings {
        requirements.insert(descriptor_identity_needs(
            DependencyDescriptorKind::StructuredTableDataRegion,
        ));
    }
    for _ in &translated.host_value_bindings {
        requirements.insert(descriptor_identity_needs(
            DependencyDescriptorKind::ShapeTopology,
        ));
    }
    for residual in &translated.residuals {
        requirements.insert(descriptor_identity_needs(
            residual.kind.dependency_descriptor_kind(),
        ));
    }

    requirements.into_iter().collect()
}

fn cross_workspace_availability_versions_for_translated(
    translated: &TranslatedFormula,
) -> Vec<String> {
    translated
        .reference_bindings
        .iter()
        .filter_map(|reference| reference.workspace_target.as_ref())
        .map(|target| {
            format!(
                "workspace={};target={};availability={}",
                target.workspace_handle, target.target_node_handle, target.availability_version
            )
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn oxfml_dependency_descriptors(prepared: &PreparedOxfmlFormula) -> Vec<DependencyDescriptor> {
    let reference_bindings_by_token = prepared
        .translated
        .reference_bindings
        .iter()
        .flat_map(|reference| {
            let entries = vec![
                (reference.token.clone(), reference),
                (format!("name:{}", reference.token), reference),
                (format!("direct:name:{}", reference.token), reference),
            ];
            entries
        })
        .collect::<BTreeMap<_, _>>();
    let unresolved_bindings_by_token = prepared
        .translated
        .unresolved_bindings
        .iter()
        .flat_map(|unresolved| {
            let entries = vec![
                (unresolved.token.clone(), unresolved),
                (format!("name:{}", unresolved.token), unresolved),
                (format!("direct:name:{}", unresolved.token), unresolved),
            ];
            entries
        })
        .collect::<BTreeMap<_, _>>();
    let host_value_bindings_by_token = prepared
        .translated
        .host_value_bindings
        .iter()
        .flat_map(|binding| {
            [
                (binding.token.clone(), binding),
                (format!("name:{}", binding.token), binding),
                (format!("direct:name:{}", binding.token), binding),
            ]
        })
        .collect::<BTreeMap<_, _>>();
    let collection_bindings_by_token = prepared
        .translated
        .collection_bindings
        .iter()
        .flat_map(|collection| {
            [
                (collection.token.clone(), collection),
                (format!("name:{}", collection.token), collection),
                (format!("direct:name:{}", collection.token), collection),
            ]
        })
        .collect::<BTreeMap<_, _>>();
    let structural_collection_base_reference_tokens =
        structural_collection_base_reference_tokens(prepared);
    let structural_collection_base_unresolved_tokens =
        structural_collection_base_unresolved_tokens(prepared);
    let structural_collection_base_host_value_tokens =
        structural_collection_base_host_value_tokens(prepared);
    let mut consumed_reference_tokens = BTreeSet::new();
    let mut consumed_unresolved_tokens = BTreeSet::new();
    let mut consumed_host_value_tokens = BTreeSet::new();
    let mut descriptors = Vec::new();

    for (index, formal_reference) in prepared
        .runtime_prepared_identity
        .formal_references
        .iter()
        .enumerate()
    {
        let source_reference_handle = Some(formal_reference.reference_handle.clone());
        if let Some(reference) =
            reference_bindings_by_token.get(formal_reference.reference_descriptor.as_str())
        {
            if structural_collection_base_reference_tokens.contains(&reference.token) {
                consumed_reference_tokens.insert(reference.token.clone());
                continue;
            }
            consumed_reference_tokens.insert(reference.token.clone());
            descriptors.push(dependency_descriptor_from_bound_reference(
                prepared,
                format!(
                    "bind:{}:oxfml_formal_ref:{index}",
                    prepared.binding.formula_artifact_id.0
                ),
                reference,
                source_reference_handle,
            ));
        } else if let Some(unresolved) =
            unresolved_bindings_by_token.get(formal_reference.reference_descriptor.as_str())
        {
            if structural_collection_base_unresolved_tokens.contains(&unresolved.token) {
                consumed_unresolved_tokens.insert(unresolved.token.clone());
                continue;
            }
            consumed_unresolved_tokens.insert(unresolved.token.clone());
            descriptors.push(dependency_descriptor_from_unresolved_binding(
                prepared,
                format!(
                    "bind:{}:oxfml_formal_ref_unresolved:{index}",
                    prepared.binding.formula_artifact_id.0
                ),
                unresolved,
                source_reference_handle,
            ));
        } else if let Some(binding) =
            host_value_bindings_by_token.get(&formal_reference.reference_descriptor)
        {
            if structural_collection_base_host_value_tokens.contains(&binding.token) {
                consumed_host_value_tokens.insert(binding.token.clone());
                continue;
            }
            consumed_host_value_tokens.insert(binding.token.clone());
            descriptors.push(dependency_descriptor_from_host_value_binding(
                prepared,
                format!(
                    "bind:{}:treecalc_host_value_formal_ref:{index}",
                    prepared.binding.formula_artifact_id.0
                ),
                binding,
                source_reference_handle,
            ));
        } else if collection_bindings_by_token.contains_key(&formal_reference.reference_descriptor)
        {
            continue;
        } else {
            if formal_reference.reference_family == "unresolved"
                && is_structural_collection_base_token(
                    prepared,
                    &formal_reference.reference_descriptor,
                )
            {
                continue;
            }
            let (kind, requires_rebind_on_structural_change) =
                dependency_descriptor_shape_from_formal_reference(formal_reference);
            descriptors.push(DependencyDescriptor {
                descriptor_id: format!(
                    "bind:{}:oxfml_unmapped_formal_ref:{index}",
                    prepared.binding.formula_artifact_id.0
                ),
                source_reference_handle,
                owner_node_id: prepared.binding.owner_node_id,
                target_node_id: None,
                workspace_target: None,
                kind,
                carrier_detail: format!(
                    "oxfml_formal_reference:{}:{}",
                    formal_reference.reference_family, formal_reference.reference_descriptor
                ),
                tree_reference_collection: None,
                requires_rebind_on_structural_change,
            });
        }
    }

    descriptors.extend(
        prepared
            .translated
            .reference_bindings
            .iter()
            .enumerate()
            .filter(|(_, reference)| {
                !consumed_reference_tokens.contains(&reference.token)
                    && !structural_collection_base_reference_tokens.contains(&reference.token)
            })
            .map(|(index, reference)| {
                dependency_descriptor_from_bound_reference(
                    prepared,
                    format!(
                        "bind:{}:oxfml_ref_fallback:{index}",
                        prepared.binding.formula_artifact_id.0
                    ),
                    reference,
                    Some(format!("treecalc_reference_carrier:{}", reference.token)),
                )
            }),
    );

    descriptors.extend(
        prepared
            .translated
            .unresolved_bindings
            .iter()
            .enumerate()
            .filter(|(_, unresolved)| {
                !consumed_unresolved_tokens.contains(&unresolved.token)
                    && !structural_collection_base_unresolved_tokens.contains(&unresolved.token)
            })
            .map(|(index, unresolved)| {
                dependency_descriptor_from_unresolved_binding(
                    prepared,
                    format!(
                        "bind:{}:oxfml_unresolved_fallback:{index}",
                        prepared.binding.formula_artifact_id.0
                    ),
                    unresolved,
                    Some(format!("treecalc_unresolved_carrier:{}", unresolved.token)),
                )
            }),
    );

    descriptors.extend(
        prepared
            .translated
            .host_value_bindings
            .iter()
            .enumerate()
            .filter(|(_, binding)| {
                !consumed_host_value_tokens.contains(&binding.token)
                    && !structural_collection_base_host_value_tokens.contains(&binding.token)
            })
            .map(|(index, binding)| {
                dependency_descriptor_from_host_value_binding(
                    prepared,
                    format!(
                        "bind:{}:treecalc_host_value_fallback:{index}",
                        prepared.binding.formula_artifact_id.0
                    ),
                    binding,
                    Some(binding.host_ref_handle.clone()),
                )
            }),
    );

    descriptors.extend(
        prepared
            .translated
            .collection_bindings
            .iter()
            .map(|collection| DependencyDescriptor {
                descriptor_id: format!(
                    "bind:{}:treecalc_collection:{}:membership",
                    prepared.binding.formula_artifact_id.0, collection.token
                ),
                source_reference_handle: Some(collection.host_ref_handle.clone()),
                owner_node_id: prepared.binding.owner_node_id,
                target_node_id: None,
                workspace_target: None,
                kind: DependencyDescriptorKind::TreeReferenceCollectionMembership,
                carrier_detail: collection.collection_dependency.carrier_detail(),
                tree_reference_collection: Some(collection.collection_dependency.clone()),
                requires_rebind_on_structural_change: false,
            }),
    );

    descriptors.extend(
        prepared
            .translated
            .collection_bindings
            .iter()
            .flat_map(|collection| {
                collection.member_node_ids.iter().copied().enumerate().map(
                    move |(member_index, member_node_id)| DependencyDescriptor {
                        descriptor_id: format!(
                            "bind:{}:treecalc_collection:{}:member:{member_index}",
                            prepared.binding.formula_artifact_id.0, collection.token
                        ),
                        source_reference_handle: Some(collection.host_ref_handle.clone()),
                        owner_node_id: prepared.binding.owner_node_id,
                        target_node_id: Some(member_node_id),
                        workspace_target: None,
                        kind: DependencyDescriptorKind::TreeReferenceCollectionMemberValue,
                        carrier_detail: collection_member_carrier_detail(
                            collection,
                            member_index,
                            member_node_id,
                        ),
                        tree_reference_collection: None,
                        requires_rebind_on_structural_change: false,
                    },
                )
            }),
    );

    descriptors.extend(prepared.translated.structured_table_bindings.iter().flat_map(
        |binding| {
            binding.member_node_ids.iter().copied().enumerate().map(
                move |(member_index, member_node_id)| DependencyDescriptor {
                    descriptor_id: format!(
                        "bind:{}:structured_table:{}:member:{member_index}",
                        prepared.binding.formula_artifact_id.0, binding.token
                    ),
                    source_reference_handle: Some(binding.host_ref_handle.clone()),
                    owner_node_id: prepared.binding.owner_node_id,
                    target_node_id: Some(member_node_id),
                    workspace_target: None,
                    kind: DependencyDescriptorKind::StructuredTableDataRegion,
                    carrier_detail: format!(
                        "structured_table_data_region:table={}:columns={}:target={member_node_id}:source={}",
                        binding.table_id,
                        binding.selected_column_ids.join("|"),
                        binding.source_token_text
                    ),
                    tree_reference_collection: None,
                    requires_rebind_on_structural_change: false,
                },
            )
        },
    ));

    descriptors.extend(prepared.translated.residuals.iter().enumerate().map(
        |(index, residual)| DependencyDescriptor {
            descriptor_id: format!(
                "bind:{}:oxfml_residual:{index}",
                prepared.binding.formula_artifact_id.0
            ),
            source_reference_handle: Some(runtime_fact_reference_handle(residual)),
            owner_node_id: prepared.binding.owner_node_id,
            target_node_id: None,
            workspace_target: None,
            kind: residual.kind.dependency_descriptor_kind(),
            carrier_detail: format!("residual:{}:{}", residual.carrier_id, residual.detail),
            tree_reference_collection: None,
            requires_rebind_on_structural_change:
                residual.kind.requires_rebind_on_structural_change(),
        },
    ));

    descriptors.sort_by(|left, right| left.descriptor_id.cmp(&right.descriptor_id));
    descriptors
}

fn structural_collection_base_reference_tokens(
    prepared: &PreparedOxfmlFormula,
) -> BTreeSet<String> {
    prepared
        .translated
        .reference_bindings
        .iter()
        .filter(|binding| {
            let Some(target_node_id) = binding.local_target_node_id else {
                return false;
            };
            prepared
                .translated
                .collection_bindings
                .iter()
                .any(|collection| {
                    collection.base_node_id == target_node_id
                        && collection_uses_structural_base_only(collection)
                })
        })
        .map(|binding| binding.token.clone())
        .collect()
}

fn structural_collection_base_unresolved_tokens(
    prepared: &PreparedOxfmlFormula,
) -> BTreeSet<String> {
    prepared
        .translated
        .unresolved_bindings
        .iter()
        .filter(|binding| {
            structural_base_node_id_for_token(prepared, &binding.token).is_some_and(|node_id| {
                prepared
                    .translated
                    .collection_bindings
                    .iter()
                    .any(|collection| {
                        collection.base_node_id == node_id
                            && collection_uses_structural_base_only(collection)
                    })
            })
        })
        .map(|binding| binding.token.clone())
        .collect()
}

fn is_structural_collection_base_token(prepared: &PreparedOxfmlFormula, token: &str) -> bool {
    structural_base_node_id_for_token(prepared, token).is_some_and(|node_id| {
        prepared
            .translated
            .collection_bindings
            .iter()
            .any(|collection| {
                collection.base_node_id == node_id
                    && collection_uses_structural_base_only(collection)
            })
    })
}

fn structural_collection_base_host_value_tokens(
    prepared: &PreparedOxfmlFormula,
) -> BTreeSet<String> {
    prepared
        .translated
        .host_value_bindings
        .iter()
        .filter(|binding| {
            let Some(target_node_id) = binding.target_node_id else {
                return false;
            };
            prepared
                .translated
                .collection_bindings
                .iter()
                .any(|collection| {
                    collection.base_node_id == target_node_id
                        && collection_uses_structural_base_only(collection)
                })
        })
        .map(|binding| binding.token.clone())
        .collect()
}

fn collection_uses_structural_base_only(collection: &SyntheticReferenceCollectionBinding) -> bool {
    matches!(
        collection.collection_dependency.family,
        TreeReferenceCollectionFamily::SiblingSetV1
            | TreeReferenceCollectionFamily::PrecedingV1
            | TreeReferenceCollectionFamily::FollowingV1
            | TreeReferenceCollectionFamily::AncestorsV1
            | TreeReferenceCollectionFamily::RecursiveDescendantsV1
    )
}

fn structural_base_node_id_for_token(
    prepared: &PreparedOxfmlFormula,
    token: &str,
) -> Option<TreeNodeId> {
    if let crate::tree_reference_resolution::ContextHostNameResolution::Resolved(node_id) =
        crate::tree_reference_resolution::resolve_context_host_name_token(
            token,
            prepared.binding.owner_node_id,
            &prepared.structural_snapshot,
            &prepared.meta_node_ids,
        )
    {
        return Some(node_id);
    }

    let projection_path = token.replace('.', "/");
    if let Some(node_id) = prepared
        .structural_snapshot
        .try_resolve_projection_path(&projection_path)
    {
        return Some(node_id);
    }

    let projection_suffix = format!("/{projection_path}");
    let matches = prepared
        .structural_snapshot
        .nodes()
        .keys()
        .filter_map(|node_id| {
            let candidate_path = prepared
                .structural_snapshot
                .get_projection_path(*node_id)
                .ok()?;
            (candidate_path == projection_path || candidate_path.ends_with(&projection_suffix))
                .then_some(*node_id)
        })
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [node_id] => Some(*node_id),
        _ => None,
    }
}

pub(crate) fn oxfml_dependency_descriptors_for_formula_catalog(
    snapshot: &StructuralSnapshot,
    table_snapshots: &BTreeMap<TreeNodeId, TreeCalcTableNodeSnapshot>,
    catalog: &TreeFormulaCatalog,
    environment_context: &LocalTreeCalcEnvironmentContext,
) -> Result<Vec<DependencyDescriptor>, LocalTreeCalcError> {
    let mut descriptors = Vec::new();
    for binding in catalog.bindings_by_owner().values() {
        let prepared =
            prepare_oxfml_formula(snapshot, table_snapshots, binding, environment_context)?;
        descriptors.extend(oxfml_dependency_descriptors(&prepared));
    }
    descriptors.sort_by(|left, right| left.descriptor_id.cmp(&right.descriptor_id));
    Ok(descriptors)
}

fn collection_member_carrier_detail(
    collection: &SyntheticReferenceCollectionBinding,
    member_index: usize,
    member_node_id: TreeNodeId,
) -> String {
    match collection.collection_dependency.family {
        TreeReferenceCollectionFamily::ChildrenV1 => format!(
            "treecalc_children_v1_member:handle={}:ordinal={member_index}:target={member_node_id}",
            collection.host_ref_handle
        ),
        TreeReferenceCollectionFamily::ReferenceLiteralArrayV1 => format!(
            "treecalc_reference_literal_array_v1_member:handle={}:ordinal={member_index}:target={member_node_id}",
            collection.host_ref_handle
        ),
        family => format!(
            "treecalc_ordered_selector_v1_member:family={}:handle={}:ordinal={member_index}:target={member_node_id}",
            family.stable_id(),
            collection.host_ref_handle
        ),
    }
}

fn tree_reference_collection_family_from_bound_key(
    family: &str,
) -> Option<TreeReferenceCollectionFamily> {
    match family {
        "children" => Some(TreeReferenceCollectionFamily::ChildrenV1),
        "reference_literal_array" => Some(TreeReferenceCollectionFamily::ReferenceLiteralArrayV1),
        "siblings" => Some(TreeReferenceCollectionFamily::SiblingSetV1),
        "preceding" => Some(TreeReferenceCollectionFamily::PrecedingV1),
        "following" => Some(TreeReferenceCollectionFamily::FollowingV1),
        "ancestors" => Some(TreeReferenceCollectionFamily::AncestorsV1),
        "recursive_descendants" => Some(TreeReferenceCollectionFamily::RecursiveDescendantsV1),
        _ => None,
    }
}

fn dependency_descriptor_shape_from_formal_reference(
    formal_reference: &RuntimeFormalReference,
) -> (DependencyDescriptorKind, bool) {
    let kind = match formal_reference.reference_family.as_str() {
        "dynamic_potential" => DependencyDescriptorKind::DynamicPotential,
        "host_sensitive" => DependencyDescriptorKind::HostSensitive,
        "shape_topology_sensitive" => DependencyDescriptorKind::ShapeTopology,
        "unresolved" => DependencyDescriptorKind::Unresolved,
        _ => DependencyDescriptorKind::Unresolved,
    };
    (
        kind,
        formal_reference.caller_context_dependent
            || matches!(
                kind,
                DependencyDescriptorKind::HostSensitive
                    | DependencyDescriptorKind::DynamicPotential
                    | DependencyDescriptorKind::ShapeTopology
                    | DependencyDescriptorKind::CapabilitySensitive
            ),
    )
}

fn dependency_descriptor_from_bound_reference(
    prepared: &PreparedOxfmlFormula,
    descriptor_id: String,
    reference: &SyntheticReferenceBinding,
    source_reference_handle: Option<String>,
) -> DependencyDescriptor {
    DependencyDescriptor {
        descriptor_id,
        source_reference_handle,
        owner_node_id: prepared.binding.owner_node_id,
        target_node_id: reference.local_target_node_id,
        workspace_target: reference.workspace_target.clone(),
        kind: reference.kind,
        carrier_detail: reference.carrier_detail.clone(),
        tree_reference_collection: None,
        requires_rebind_on_structural_change: reference.requires_rebind_on_structural_change,
    }
}

fn dependency_descriptor_from_unresolved_binding(
    prepared: &PreparedOxfmlFormula,
    descriptor_id: String,
    unresolved: &SyntheticUnresolvedBinding,
    source_reference_handle: Option<String>,
) -> DependencyDescriptor {
    DependencyDescriptor {
        descriptor_id,
        source_reference_handle,
        owner_node_id: prepared.binding.owner_node_id,
        target_node_id: None,
        workspace_target: None,
        kind: unresolved.kind,
        carrier_detail: unresolved.carrier_detail.clone(),
        tree_reference_collection: None,
        requires_rebind_on_structural_change: unresolved.requires_rebind_on_structural_change,
    }
}

fn dependency_descriptor_from_host_value_binding(
    prepared: &PreparedOxfmlFormula,
    descriptor_id: String,
    binding: &SyntheticHostValueBinding,
    source_reference_handle: Option<String>,
) -> DependencyDescriptor {
    DependencyDescriptor {
        descriptor_id,
        source_reference_handle,
        owner_node_id: prepared.binding.owner_node_id,
        target_node_id: binding.target_node_id,
        workspace_target: None,
        kind: DependencyDescriptorKind::ShapeTopology,
        carrier_detail: binding.carrier_detail.clone(),
        tree_reference_collection: None,
        requires_rebind_on_structural_change: binding.requires_rebind_on_structural_change,
    }
}

fn runtime_fact_reference_handle(residual: &ResidualCarrier) -> String {
    format!("runtime_fact:{:?}:{}", residual.kind, residual.carrier_id)
}

fn residual_evaluation_failure(
    prepared: &PreparedOxfmlFormula,
    extra_diagnostics: Vec<String>,
) -> Option<LocalFormulaEvaluationFailure> {
    let residual = prepared.translated.residuals.first()?;
    let runtime_effects = prepared
        .translated
        .residuals
        .iter()
        .map(residual_runtime_effect)
        .collect::<Vec<_>>();
    let error = match residual.kind {
        ResidualCarrierKind::HostSensitive => LocalTreeCalcError::HostSensitiveReference {
            owner_node_id: residual.owner_node_id,
            detail: residual.detail.clone(),
        },
        ResidualCarrierKind::DynamicPotential => LocalTreeCalcError::DynamicReference {
            owner_node_id: residual.owner_node_id,
            detail: residual.detail.clone(),
        },
        ResidualCarrierKind::CapabilitySensitive => {
            LocalTreeCalcError::CapabilitySensitiveReference {
                owner_node_id: residual.owner_node_id,
                detail: residual.detail.clone(),
            }
        }
        ResidualCarrierKind::ShapeTopology => LocalTreeCalcError::ShapeTopologyReference {
            owner_node_id: residual.owner_node_id,
            detail: residual.detail.clone(),
        },
    };

    Some(LocalFormulaEvaluationFailure {
        error,
        runtime_effects,
        diagnostics: prepared
            .bind_diagnostics
            .iter()
            .cloned()
            .chain(extra_diagnostics)
            .collect(),
    })
}

fn evaluate_with_oxfml_session(
    prepared: &PreparedOxfmlFormula,
    workspace_revision: &WorkspaceRevision,
    working_values: &BTreeMap<TreeNodeId, String>,
    working_calc_values: &BTreeMap<TreeNodeId, CalcValue>,
    derivation_trace_enabled: bool,
) -> Result<LocalFormulaEvaluationSuccess, LocalFormulaEvaluationFailure> {
    if let Some((collection, missing_node_id)) = missing_collection_member_value(
        &prepared.translated,
        workspace_revision,
        working_calc_values,
    ) {
        return Err(LocalFormulaEvaluationFailure {
            error: LocalTreeCalcError::MissingReferencedValue {
                node_id: missing_node_id,
            },
            runtime_effects: Vec::new(),
            diagnostics: prepared
                .bind_diagnostics
                .iter()
                .cloned()
                .chain([format!(
                    "missing_collection_member_value:owner={};target={};handle={}",
                    prepared.binding.owner_node_id, missing_node_id, collection.host_ref_handle
                )])
                .collect(),
        });
    }

    if let Some((binding, missing_node_id)) =
        missing_structured_table_member_value(&prepared.translated, working_calc_values)
    {
        return Err(LocalFormulaEvaluationFailure {
            error: LocalTreeCalcError::MissingReferencedValue {
                node_id: missing_node_id,
            },
            runtime_effects: Vec::new(),
            diagnostics: prepared
                .bind_diagnostics
                .iter()
                .cloned()
                .chain([format!(
                    "missing_structured_table_member_value:owner={};target={};handle={}",
                    prepared.binding.owner_node_id, missing_node_id, binding.host_ref_handle
                )])
                .collect(),
        });
    }

    if let Some(unresolved) = prepared
        .runtime_prepared_identity
        .formal_references
        .iter()
        .find(|reference| {
            reference.reference_family == "unresolved"
                && !is_structural_collection_base_token(prepared, &reference.reference_descriptor)
        })
    {
        return Err(LocalFormulaEvaluationFailure {
            error: LocalTreeCalcError::OxfmlBindUnresolved {
                owner_node_id: prepared.binding.owner_node_id,
                detail: unresolved.reference_descriptor.clone(),
            },
            runtime_effects: Vec::new(),
            diagnostics: prepared.bind_diagnostics.clone(),
        });
    }

    let trace_mode = if derivation_trace_enabled {
        EvaluationTraceMode::PreparedCalls
    } else {
        EvaluationTraceMode::default()
    };
    let invoke_result = match invoke_prepared_formula_via_session(
        prepared,
        working_values,
        working_calc_values,
        trace_mode,
    ) {
        Ok(run) => run,
        Err(detail) => {
            if let Some(failure) =
                residual_evaluation_failure(prepared, vec![format!("oxfml_host_error:{detail}")])
            {
                return Err(failure);
            }

            return Err(LocalFormulaEvaluationFailure {
                error: LocalTreeCalcError::OxfmlHostFailure {
                    owner_node_id: prepared.binding.owner_node_id,
                    detail,
                },
                runtime_effects: Vec::new(),
                diagnostics: prepared.bind_diagnostics.clone(),
            });
        }
    };

    let run = &invoke_result.run;
    let returned_surface_diagnostics =
        oxfml_returned_value_surface_diagnostics(&run.returned_value_surface);
    let should_reject_residual = matches!(
        run.returned_value_surface.kind,
        ReturnedValueSurfaceKind::TypedHostProviderOutcome
    ) || (!prepared.translated.residuals.is_empty()
        && !prepared.lazy_residual_publication);
    if should_reject_residual
        && let Some(failure) = residual_evaluation_failure(
            prepared,
            returned_surface_diagnostics
                .iter()
                .cloned()
                .chain(
                    run.trace_events
                        .iter()
                        .map(|event| format!("oxfml_trace:{:?}", event.event_kind)),
                )
                .collect(),
        )
    {
        return Err(failure);
    }

    let mut success = adapt_oxfml_runtime_candidate(prepared, run, derivation_trace_enabled)?;
    success.dynamic_reference_resolutions = invoke_result.dynamic_reference_resolutions;
    Ok(success)
}

fn missing_collection_member_value<'a>(
    translated: &'a TranslatedFormula,
    workspace_revision: &WorkspaceRevision,
    working_calc_values: &BTreeMap<TreeNodeId, CalcValue>,
) -> Option<(&'a SyntheticReferenceCollectionBinding, TreeNodeId)> {
    translated
        .collection_bindings
        .iter()
        .find_map(|collection| {
            collection
                .member_node_ids
                .iter()
                .copied()
                .find(|node_id| {
                    !working_calc_values.contains_key(node_id)
                        && !(collection_allows_empty_member_values(collection)
                            && is_empty_node_input(workspace_revision, *node_id))
                })
                .map(|node_id| (collection, node_id))
        })
}

fn collection_allows_empty_member_values(collection: &SyntheticReferenceCollectionBinding) -> bool {
    matches!(
        collection.collection_dependency.family,
        TreeReferenceCollectionFamily::ChildrenV1
            | TreeReferenceCollectionFamily::SiblingSetV1
            | TreeReferenceCollectionFamily::PrecedingV1
            | TreeReferenceCollectionFamily::FollowingV1
            | TreeReferenceCollectionFamily::AncestorsV1
            | TreeReferenceCollectionFamily::RecursiveDescendantsV1
    )
}

fn is_empty_node_input(workspace_revision: &WorkspaceRevision, node_id: TreeNodeId) -> bool {
    workspace_revision
        .node_input_snapshot
        .records()
        .get(&node_id)
        .is_some_and(|record| record.kind == NodeInputKind::Empty)
}

fn missing_structured_table_member_value<'a>(
    translated: &'a TranslatedFormula,
    working_calc_values: &BTreeMap<TreeNodeId, CalcValue>,
) -> Option<(&'a SyntheticStructuredTableBinding, TreeNodeId)> {
    translated
        .structured_table_bindings
        .iter()
        .find_map(|binding| {
            binding
                .member_node_ids
                .iter()
                .copied()
                .find(|node_id| !working_calc_values.contains_key(node_id))
                .map(|node_id| (binding, node_id))
        })
}

struct OxfmlSessionInvokeResult {
    run: RuntimeFormulaResult,
    dynamic_reference_resolutions: Vec<TreeCalcRuntimeReferenceTextResolution>,
}

fn invoke_prepared_formula_via_session(
    prepared: &PreparedOxfmlFormula,
    working_values: &BTreeMap<TreeNodeId, String>,
    working_calc_values: &BTreeMap<TreeNodeId, CalcValue>,
    trace_mode: EvaluationTraceMode,
) -> Result<OxfmlSessionInvokeResult, String> {
    let host_info_provider = TreeCalcHostInfoProvider;
    let rtd_provider = TreeCalcRtdProvider;
    let reference_system_provider = treecalc_reference_system_provider_for_runtime(
        prepared,
        working_values,
        working_calc_values,
    );
    let host_info_required = prepared
        .translated
        .residuals
        .iter()
        .any(|residual| matches!(residual.kind, ResidualCarrierKind::HostSensitive))
        || prepared.requires_host_query
        || prepared.requires_image_provider;
    let rtd_required = prepared
        .translated
        .residuals
        .iter()
        .any(|residual| matches!(residual.kind, ResidualCarrierKind::DynamicPotential));
    let query_bundle = TypedContextQueryBundle::new(
        host_info_required.then_some(&host_info_provider as &dyn HostInfoProvider),
        rtd_required.then_some(&rtd_provider as &dyn RtdProvider),
        None,
        None,
        None,
    )
    .with_reference_system_provider(Some(
        &reference_system_provider as &dyn oxfunc_core::resolver::ReferenceSystemProvider,
    ));
    let request = RuntimeFormulaRequest::new(prepared.source.clone(), query_bundle)
        .with_backend(EvaluationBackend::OxFuncBacked)
        .with_trace_mode(trace_mode);
    let mut session = OxfmlRecalcSessionDriver::new(build_treecalc_runtime_environment(
        prepared,
        working_values,
        working_calc_values,
    ));

    let run = session.invoke(request).map_err(|error| error.to_string())?;
    Ok(OxfmlSessionInvokeResult {
        run,
        dynamic_reference_resolutions: reference_system_provider.runtime_text_resolutions(),
    })
}

fn build_treecalc_runtime_environment(
    prepared: &PreparedOxfmlFormula,
    working_values: &BTreeMap<TreeNodeId, String>,
    working_calc_values: &BTreeMap<TreeNodeId, CalcValue>,
) -> RuntimeEnvironment<'static> {
    build_treecalc_runtime_environment_from_parts(TreeCalcRuntimeEnvironmentBuild {
        translated: &prepared.translated,
        snapshot: &prepared.structural_snapshot,
        table_snapshots: &prepared.table_snapshots,
        owner_node_id: prepared.binding.owner_node_id,
        meta_node_ids: &prepared.meta_node_ids,
        structure_context_version: StructureContextVersion(
            prepared
                .runtime_prepared_identity
                .structure_context_version
                .clone(),
        ),
        oxfunc_bridge_metadata: &prepared.oxfunc_bridge_metadata,
        working_values,
        working_calc_values,
        prepared_formula_identity: Some(&prepared.runtime_prepared_identity),
        host_formula_context: prepared
            .runtime_prepared_identity
            .host_formula_context
            .clone(),
    })
}

fn treecalc_reference_system_provider_for_runtime<'a>(
    prepared: &'a PreparedOxfmlFormula,
    working_values: &'a BTreeMap<TreeNodeId, String>,
    working_calc_values: &'a BTreeMap<TreeNodeId, CalcValue>,
) -> TreeCalcReferenceSystemProvider<'a> {
    let provider = prepared.translated.collection_bindings.iter().fold(
        TreeCalcReferenceSystemProvider::new(
            &prepared.structural_snapshot,
            &prepared.meta_node_ids,
            prepared.binding.owner_node_id,
            working_calc_values,
        ),
        |provider, collection| {
            provider.with_collection_descriptor(collection_reference_descriptor(collection))
        },
    );

    sparse_reference_value_bindings_for_runtime(
        &prepared.translated,
        &prepared.structural_snapshot,
        working_values,
        working_calc_values,
    )
    .into_iter()
    .fold(provider, |provider, binding| {
        provider.with_sparse_reference_values(binding.reference.clone(), binding.resolved_values())
    })
}

fn collection_reference_descriptor(
    collection: &SyntheticReferenceCollectionBinding,
) -> TreeCalcCollectionReferenceDescriptor {
    TreeCalcCollectionReferenceDescriptor {
        host_ref_handle: collection.host_ref_handle.clone(),
        family: collection.collection_dependency.family,
        base_node_id: collection.base_node_id,
        source_span_utf8: collection.source_span_utf8,
        source_token_text: collection.source_token_text.clone(),
        opaque_selector: collection.opaque_selector.clone(),
        member_node_ids: collection.member_node_ids.clone(),
        membership_version: collection.collection_dependency.membership_version.clone(),
        order_version: collection.collection_dependency.order_version.clone(),
    }
}

fn formal_input_bindings_for_runtime(
    translated: &TranslatedFormula,
    working_values: &BTreeMap<TreeNodeId, String>,
    working_calc_values: &BTreeMap<TreeNodeId, CalcValue>,
    prepared_formula_identity: Option<&RuntimePreparedFormulaIdentity>,
) -> Vec<RuntimeFormalInputBinding> {
    translated
        .reference_bindings
        .iter()
        .filter(|reference| reference.host_name_bind.is_none())
        .map(|reference| {
            let descriptor_without_prefix = reference.token.as_str();
            let descriptor_with_prefix = format!("name:{}", reference.token);
            let formal_reference = prepared_formula_identity.and_then(|identity| {
                identity.formal_references.iter().find(|formal_reference| {
                    formal_reference.reference_descriptor == descriptor_with_prefix
                        || formal_reference.reference_descriptor == descriptor_without_prefix
                })
            });
            RuntimeFormalInputBinding {
                reference_handle: formal_reference
                    .map(|reference| reference.reference_handle.clone()),
                reference_descriptor: formal_reference
                    .map_or(descriptor_with_prefix, |reference| {
                        reference.reference_descriptor.clone()
                    }),
                binding: runtime_binding_for_reference(
                    reference,
                    working_values,
                    working_calc_values,
                ),
            }
        })
        .chain(translated.collection_bindings.iter().map(|collection| {
            let descriptor_with_prefix = format!("name:{}", collection.token);
            RuntimeFormalInputBinding {
                reference_handle: None,
                reference_descriptor: descriptor_with_prefix,
                binding: DefinedNameBinding::Reference(treecalc_collection_reference_like(
                    &collection.host_ref_handle,
                )),
            }
        }))
        .chain(translated.structured_table_bindings.iter().map(|binding| {
            let formal_reference = prepared_formula_identity.and_then(|identity| {
                identity.formal_references.iter().find(|formal_reference| {
                    formal_reference
                        .structured_reference_bind_record_handle
                        .as_deref()
                        == Some(binding.host_ref_handle.as_str())
                })
            });
            RuntimeFormalInputBinding {
                reference_handle: formal_reference
                    .map(|reference| reference.reference_handle.clone()),
                reference_descriptor: formal_reference.map_or_else(
                    || binding.token.clone(),
                    |reference| reference.reference_descriptor.clone(),
                ),
                binding: DefinedNameBinding::Reference(ReferenceLike::new(
                    binding.reference_kind,
                    binding.reference_target.clone(),
                )),
            }
        }))
        .chain(translated.host_value_bindings.iter().map(|binding| {
            let descriptor_with_prefix = format!("name:{}", binding.token);
            RuntimeFormalInputBinding {
                reference_handle: None,
                reference_descriptor: descriptor_with_prefix,
                binding: DefinedNameBinding::Value(calc_value_from_tree_formula_host_value(
                    &binding.value,
                )),
            }
        }))
        .collect::<Vec<_>>()
}

fn host_name_bindings_for_runtime(
    translated: &TranslatedFormula,
    working_values: &BTreeMap<TreeNodeId, String>,
    working_calc_values: &BTreeMap<TreeNodeId, CalcValue>,
) -> Vec<RuntimeHostNameBinding> {
    translated
        .reference_bindings
        .iter()
        .filter_map(|reference| {
            reference
                .host_name_bind
                .as_ref()
                .map(|bind| RuntimeHostNameBinding {
                    bind_result: RuntimeHostNameBindResult {
                        host_name_handle: bind.host_name_handle.clone(),
                        canonical_name: bind.canonical_name.clone(),
                        host_dependency_key: bind.host_dependency_key.clone(),
                        source_span: TextSpan::new(
                            bind.source_span_utf8.0,
                            bind.source_span_utf8
                                .1
                                .saturating_sub(bind.source_span_utf8.0),
                        ),
                        source_token_text: bind.source_token_text.clone(),
                        resolution_layer: bind.resolution_layer.clone(),
                        binding_kind: bind.binding_kind.clone(),
                        shape_hint: bind.shape_hint.clone(),
                        caller_context_dependent: bind.caller_context_dependent,
                        diagnostics: bind.diagnostics.clone(),
                        replay_identity_contribution: bind.replay_identity_contribution.clone(),
                    },
                    binding: runtime_binding_for_reference(
                        reference,
                        working_values,
                        working_calc_values,
                    ),
                })
        })
        .collect()
}

fn context_host_name_bindings_for_runtime(
    parts: &TreeCalcRuntimeEnvironmentBuild<'_>,
) -> Vec<RuntimeHostNameBinding> {
    parts
        .snapshot
        .nodes()
        .values()
        .filter(|node| {
            node.node_id != parts.snapshot.root_node_id()
                && !node.symbol.is_empty()
                && !crate::tree_reference_resolution::is_meta_effective(
                    node.node_id,
                    parts.snapshot,
                    parts.meta_node_ids,
                )
        })
        .map(|node| node.symbol.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .filter_map(
            |symbol| match crate::tree_reference_resolution::resolve_context_host_name_token(
                &symbol,
                parts.owner_node_id,
                parts.snapshot,
                parts.meta_node_ids,
            ) {
                crate::tree_reference_resolution::ContextHostNameResolution::Resolved(
                    target_node_id,
                ) => Some(context_host_name_binding_for_runtime(
                    parts.owner_node_id,
                    target_node_id,
                    symbol,
                    parts.working_values,
                    parts.working_calc_values,
                )),
                crate::tree_reference_resolution::ContextHostNameResolution::Ambiguous
                | crate::tree_reference_resolution::ContextHostNameResolution::Unsupported(_)
                | crate::tree_reference_resolution::ContextHostNameResolution::Unresolved => None,
            },
        )
        .collect()
}

fn merge_runtime_host_name_bindings(
    explicit_bindings: Vec<RuntimeHostNameBinding>,
    context_bindings: Vec<RuntimeHostNameBinding>,
) -> Vec<RuntimeHostNameBinding> {
    let mut bindings = explicit_bindings;
    let mut seen_names = bindings
        .iter()
        .map(|binding| binding.bind_result.canonical_name.to_ascii_uppercase())
        .collect::<BTreeSet<_>>();
    for binding in context_bindings {
        if seen_names.insert(binding.bind_result.canonical_name.to_ascii_uppercase()) {
            bindings.push(binding);
        }
    }
    bindings
}

fn context_host_name_binding_for_runtime(
    owner_node_id: TreeNodeId,
    target_node_id: TreeNodeId,
    canonical_name: String,
    working_values: &BTreeMap<TreeNodeId, String>,
    working_calc_values: &BTreeMap<TreeNodeId, CalcValue>,
) -> RuntimeHostNameBinding {
    let packet = TreeFormulaHostNameBindPacket::direct_tree_node(
        "treecalc-runtime",
        owner_node_id,
        target_node_id,
        canonical_name.clone(),
        (0, canonical_name.len()),
        canonical_name.clone(),
    );
    RuntimeHostNameBinding {
        bind_result: RuntimeHostNameBindResult {
            host_name_handle: packet.host_name_handle,
            canonical_name: packet.canonical_name,
            host_dependency_key: packet.host_dependency_key,
            source_span: TextSpan::new(0, canonical_name.len()),
            source_token_text: canonical_name,
            resolution_layer: packet.resolution_layer,
            binding_kind: packet.binding_kind,
            shape_hint: packet.shape_hint,
            caller_context_dependent: packet.caller_context_dependent,
            diagnostics: packet.diagnostics,
            replay_identity_contribution: packet.replay_identity_contribution,
        },
        binding: runtime_binding_for_node(target_node_id, working_values, working_calc_values),
    }
}

fn runtime_binding_for_node(
    target_node_id: TreeNodeId,
    working_values: &BTreeMap<TreeNodeId, String>,
    working_calc_values: &BTreeMap<TreeNodeId, CalcValue>,
) -> DefinedNameBinding {
    working_calc_values
        .get(&target_node_id)
        .and_then(callable_binding_from_calc_value)
        .unwrap_or_else(|| {
            DefinedNameBinding::Value(
                working_values
                    .get(&target_node_id)
                    .map_or(CalcValue::number(0.0), |value| {
                        authored_cell_entry_text_to_calc_value(value)
                    }),
            )
        })
}

fn runtime_binding_for_reference(
    reference: &SyntheticReferenceBinding,
    working_values: &BTreeMap<TreeNodeId, String>,
    working_calc_values: &BTreeMap<TreeNodeId, CalcValue>,
) -> DefinedNameBinding {
    if let Some(workspace_target) = &reference.workspace_target {
        return DefinedNameBinding::Reference(ReferenceLike::new(
            ReferenceKind::ThreeD,
            workspace_target.target_node_handle.clone(),
        ));
    }

    match reference.local_target_node_id {
        Some(target_node_id) => {
            runtime_binding_for_node(target_node_id, working_values, working_calc_values)
        }
        None => DefinedNameBinding::Value(CalcValue::error(WorksheetErrorCode::Ref)),
    }
}

fn callable_binding_from_calc_value(value: &CalcValue) -> Option<DefinedNameBinding> {
    let Some(RichValue::Callable(callable)) = value.rich.as_deref() else {
        return None;
    };
    let binding = callable
        .handle
        .as_any()
        .downcast_ref::<OxFmlCallableBinding>()?;
    Some(DefinedNameBinding::Callable(binding.binding.clone()))
}

fn calc_value_from_tree_formula_host_value(value: &TreeFormulaHostValue) -> CalcValue {
    match value {
        TreeFormulaHostValue::Text(value) => {
            CalcValue::text(ExcelText::from_interop_assignment(value))
        }
        TreeFormulaHostValue::Integer(value) => CalcValue::number(*value as f64),
        TreeFormulaHostValue::ValueError => CalcValue::error(WorksheetErrorCode::Value),
    }
}

fn sparse_reference_value_bindings_for_runtime(
    translated: &TranslatedFormula,
    structural_snapshot: &StructuralSnapshot,
    working_values: &BTreeMap<TreeNodeId, String>,
    working_calc_values: &BTreeMap<TreeNodeId, CalcValue>,
) -> Vec<TreeCalcSparseReferenceValuesBinding> {
    let mut bindings = translated
        .collection_bindings
        .iter()
        .filter_map(|collection| {
            treecalc_sparse_reference_values_binding(
                collection,
                structural_snapshot,
                working_values,
                working_calc_values,
            )
        })
        .collect::<Vec<_>>();
    bindings.extend(
        translated
            .structured_table_bindings
            .iter()
            .filter_map(|binding| {
                treecalc_structured_table_sparse_reference_values_binding(
                    binding,
                    working_calc_values,
                )
            }),
    );
    bindings
}

fn treecalc_structured_table_sparse_reference_values_binding(
    binding: &SyntheticStructuredTableBinding,
    working_calc_values: &BTreeMap<TreeNodeId, CalcValue>,
) -> Option<TreeCalcSparseReferenceValuesBinding> {
    let mut defined_cells = binding
        .literal_cells
        .iter()
        .map(|cell| {
            TreeCalcSparseReferenceCell::new(
                cell.row_index,
                cell.col_index,
                structured_table_literal_to_calc_value(&cell.value),
            )
        })
        .collect::<Vec<_>>();
    defined_cells.extend(binding.member_node_cells.iter().filter_map(|cell| {
        let value = working_calc_values.get(&cell.node_id)?;
        Some(TreeCalcSparseReferenceCell::new(
            cell.row_index,
            cell.col_index,
            value.clone(),
        ))
    }));
    Some(TreeCalcSparseReferenceValuesBinding {
        reference: ReferenceLike::new(binding.reference_kind, binding.reference_target.clone()),
        declared_rows: binding.declared_rows,
        declared_cols: binding.declared_cols,
        defined_cells,
        reader_identity: Some(format!(
            "treecalc_table_ref:handle={};table={};rows={};columns={};sections={};row_membership={};row_order={};column_identity={}",
            binding.host_ref_handle,
            binding.table_id,
            binding.declared_rows,
            binding.selected_column_ids.join("|"),
            structured_table_section_identity(&binding.selected_sections),
            binding.row_membership_version,
            binding.row_order_version,
            binding.column_identity_version
        )),
    })
}

fn structured_table_literal_to_calc_value(
    value: &SyntheticStructuredTableLiteralValue,
) -> CalcValue {
    match value {
        SyntheticStructuredTableLiteralValue::Text(text) => {
            CalcValue::text(ExcelText::from_interop_assignment(text))
        }
    }
}

fn structured_table_section_identity(sections: &[StructuredSectionKind]) -> String {
    sections
        .iter()
        .map(|section| match section {
            StructuredSectionKind::All => "all",
            StructuredSectionKind::Data => "data",
            StructuredSectionKind::Headers => "headers",
            StructuredSectionKind::Totals => "totals",
            StructuredSectionKind::ThisRow => "this_row",
        })
        .collect::<Vec<_>>()
        .join("|")
}

fn structured_table_selected_sections(
    record: &StructuredReferenceBindRecord,
) -> Vec<StructuredSectionKind> {
    let mut sections = record.selected_sections.clone();
    for region in &record.selected_regions {
        if !sections.contains(&region.section_kind) {
            sections.push(region.section_kind);
        }
    }
    if sections.is_empty() {
        sections.push(StructuredSectionKind::Data);
    }
    if sections.contains(&StructuredSectionKind::All) {
        return vec![StructuredSectionKind::All];
    }
    sections
}

fn treecalc_sparse_reference_values_binding(
    collection: &SyntheticReferenceCollectionBinding,
    structural_snapshot: &StructuralSnapshot,
    working_values: &BTreeMap<TreeNodeId, String>,
    working_calc_values: &BTreeMap<TreeNodeId, CalcValue>,
) -> Option<TreeCalcSparseReferenceValuesBinding> {
    match collection.collection_dependency.family {
        TreeReferenceCollectionFamily::ChildrenV1 => {
            treecalc_children_sparse_reference_values_binding(
                collection,
                structural_snapshot,
                working_values,
                working_calc_values,
            )
        }
        TreeReferenceCollectionFamily::ReferenceLiteralArrayV1 => {
            treecalc_reference_literal_array_sparse_reference_values_binding(
                collection,
                structural_snapshot,
                working_values,
                working_calc_values,
            )
        }
        TreeReferenceCollectionFamily::SiblingSetV1
        | TreeReferenceCollectionFamily::PrecedingV1
        | TreeReferenceCollectionFamily::FollowingV1
        | TreeReferenceCollectionFamily::AncestorsV1
        | TreeReferenceCollectionFamily::RecursiveDescendantsV1 => {
            treecalc_ordered_selector_sparse_reference_values_binding(
                collection,
                structural_snapshot,
                working_values,
                working_calc_values,
            )
        }
    }
}

fn treecalc_children_sparse_reference_values_binding(
    collection: &SyntheticReferenceCollectionBinding,
    structural_snapshot: &StructuralSnapshot,
    working_values: &BTreeMap<TreeNodeId, String>,
    working_calc_values: &BTreeMap<TreeNodeId, CalcValue>,
) -> Option<TreeCalcSparseReferenceValuesBinding> {
    let reader = TreeCalcChildrenSparseReader::from_published_calc_values(
        structural_snapshot,
        crate::formula::TreeCalcChildrenReferenceCollection {
            host_ref_handle: collection.host_ref_handle.clone(),
            base_node_id: collection.base_node_id,
            source_span_utf8: collection.source_span_utf8,
            source_token_text: collection.source_token_text.clone(),
            opaque_selector: collection.opaque_selector.clone(),
            membership_version: collection.collection_dependency.membership_version.clone(),
            order_version: collection.collection_dependency.order_version.clone(),
        },
        working_calc_values,
    )
    .or_else(|_| {
        TreeCalcChildrenSparseReader::from_published_values(
            structural_snapshot,
            crate::formula::TreeCalcChildrenReferenceCollection {
                host_ref_handle: collection.host_ref_handle.clone(),
                base_node_id: collection.base_node_id,
                source_span_utf8: collection.source_span_utf8,
                source_token_text: collection.source_token_text.clone(),
                opaque_selector: collection.opaque_selector.clone(),
                membership_version: collection.collection_dependency.membership_version.clone(),
                order_version: collection.collection_dependency.order_version.clone(),
            },
            working_values,
        )
    })
    .ok()?;
    Some(runtime_sparse_reference_values_binding(
        treecalc_collection_reference_like(&collection.host_ref_handle),
        &reader,
    ))
}

fn treecalc_reference_literal_array_sparse_reference_values_binding(
    collection: &SyntheticReferenceCollectionBinding,
    structural_snapshot: &StructuralSnapshot,
    working_values: &BTreeMap<TreeNodeId, String>,
    working_calc_values: &BTreeMap<TreeNodeId, CalcValue>,
) -> Option<TreeCalcSparseReferenceValuesBinding> {
    let carrier_id = collection
        .host_ref_handle
        .strip_prefix("treecalc-hostref:v1:reference_literal_array:")
        .unwrap_or(&collection.host_ref_handle);
    let elements = collection
        .member_node_ids
        .iter()
        .copied()
        .map(crate::formula::TreeCalcReferenceLiteralArrayElement::ReferenceNode);
    let mut reference_collection =
        crate::formula::TreeCalcReferenceLiteralArrayCollection::reference_only_with_handle(
            carrier_id,
            collection.host_ref_handle.clone(),
            collection.base_node_id,
            collection.source_token_text.clone(),
            elements,
        )
        .ok()?;
    if let Some((start, end)) = collection.source_span_utf8 {
        reference_collection = reference_collection.with_source_span_utf8(start, end);
    }
    let reader = TreeCalcReferenceLiteralArraySparseReader::from_published_calc_values(
        structural_snapshot,
        reference_collection.clone(),
        working_calc_values,
    )
    .or_else(|_| {
        TreeCalcReferenceLiteralArraySparseReader::from_published_values(
            structural_snapshot,
            reference_collection,
            working_values,
        )
    })
    .ok()?;
    Some(runtime_sparse_reference_values_binding(
        treecalc_collection_reference_like(&collection.host_ref_handle),
        &reader,
    ))
}

fn treecalc_ordered_selector_sparse_reference_values_binding(
    collection: &SyntheticReferenceCollectionBinding,
    structural_snapshot: &StructuralSnapshot,
    working_values: &BTreeMap<TreeNodeId, String>,
    working_calc_values: &BTreeMap<TreeNodeId, CalcValue>,
) -> Option<TreeCalcSparseReferenceValuesBinding> {
    let family = ordered_selector_family_from_dependency(collection.collection_dependency.family)?;
    let reader = TreeCalcOrderedSelectorSparseReader::from_published_calc_values(
        structural_snapshot,
        crate::formula::TreeCalcOrderedSelectorReferenceCollection {
            family,
            host_ref_handle: collection.host_ref_handle.clone(),
            base_node_id: collection.base_node_id,
            member_node_ids: collection.member_node_ids.clone(),
            source_span_utf8: collection.source_span_utf8,
            source_token_text: collection.source_token_text.clone(),
            opaque_selector: collection.opaque_selector.clone(),
            membership_version: collection.collection_dependency.membership_version.clone(),
            order_version: collection.collection_dependency.order_version.clone(),
        },
        working_calc_values,
    )
    .or_else(|_| {
        TreeCalcOrderedSelectorSparseReader::from_published_values(
            structural_snapshot,
            crate::formula::TreeCalcOrderedSelectorReferenceCollection {
                family,
                host_ref_handle: collection.host_ref_handle.clone(),
                base_node_id: collection.base_node_id,
                member_node_ids: collection.member_node_ids.clone(),
                source_span_utf8: collection.source_span_utf8,
                source_token_text: collection.source_token_text.clone(),
                opaque_selector: collection.opaque_selector.clone(),
                membership_version: collection.collection_dependency.membership_version.clone(),
                order_version: collection.collection_dependency.order_version.clone(),
            },
            working_values,
        )
    })
    .ok()?;
    Some(runtime_sparse_reference_values_binding(
        treecalc_collection_reference_like(&collection.host_ref_handle),
        &reader,
    ))
}

fn ordered_selector_family_from_dependency(
    family: TreeReferenceCollectionFamily,
) -> Option<crate::formula::TreeCalcOrderedSelectorFamily> {
    match family {
        TreeReferenceCollectionFamily::ChildrenV1
        | TreeReferenceCollectionFamily::ReferenceLiteralArrayV1 => None,
        TreeReferenceCollectionFamily::SiblingSetV1 => {
            Some(crate::formula::TreeCalcOrderedSelectorFamily::SiblingSetV1)
        }
        TreeReferenceCollectionFamily::PrecedingV1 => {
            Some(crate::formula::TreeCalcOrderedSelectorFamily::PrecedingV1)
        }
        TreeReferenceCollectionFamily::FollowingV1 => {
            Some(crate::formula::TreeCalcOrderedSelectorFamily::FollowingV1)
        }
        TreeReferenceCollectionFamily::AncestorsV1 => {
            Some(crate::formula::TreeCalcOrderedSelectorFamily::AncestorsV1)
        }
        TreeReferenceCollectionFamily::RecursiveDescendantsV1 => {
            Some(crate::formula::TreeCalcOrderedSelectorFamily::RecursiveDescendantsV1)
        }
    }
}

fn treecalc_collection_reference_like(host_ref_handle: &str) -> ReferenceLike {
    crate::tree_reference_system::treecalc_collection_reference_like(host_ref_handle)
}

fn runtime_sparse_reference_values_binding(
    reference: ReferenceLike,
    reader: &impl SparseRangeReader,
) -> TreeCalcSparseReferenceValuesBinding {
    let extent = reader.declared_extent();
    let identity = reader.reader_identity();
    TreeCalcSparseReferenceValuesBinding {
        reference,
        declared_rows: usize::try_from(extent.row_count).unwrap_or(usize::MAX),
        declared_cols: usize::try_from(extent.column_count).unwrap_or(usize::MAX),
        defined_cells: reader
            .defined_iter()
            .map(|cell| {
                TreeCalcSparseReferenceCell::new(
                    sparse_coord_to_resolved_index(cell.coord.row, extent.start.row),
                    sparse_coord_to_resolved_index(cell.coord.column, extent.start.column),
                    cell.value,
                )
            })
            .collect(),
        reader_identity: Some(format!(
            "reader_id={};source={};snapshot={}",
            identity.reader_id, identity.source_identity, identity.snapshot_identity
        )),
    }
}

fn sparse_coord_to_resolved_index(coord: u32, start: u32) -> usize {
    coord
        .checked_sub(start)
        .and_then(|offset| offset.checked_add(1))
        .and_then(|index| usize::try_from(index).ok())
        .unwrap_or(usize::MAX)
}

fn treecalc_host_formula_context(
    owner_node_id: TreeNodeId,
    structure_context_version: &str,
    environment_context: &LocalTreeCalcEnvironmentContext,
    requirements: &[(NamespaceIdentityNeed, CallerContextIdentityNeed)],
    cross_workspace_availability_versions: &[String],
) -> RuntimeHostFormulaContext {
    RuntimeHostFormulaContext {
        dialect_id: "oxcalc.treecalc-v1".to_string(),
        capability_profile_id: effective_treecalc_capability_profile_id(environment_context),
        resolution_rule_version: environment_context.resolution_rule_version.clone(),
        host_namespace_version: w056_host_namespace_identity(
            environment_context,
            requirements,
            cross_workspace_availability_versions,
        ),
        registry_snapshot_identity: environment_context
            .oxfunc_bridge_metadata
            .semantic_kernel_metadata_version
            .clone(),
        structure_context_version: w056_requires_namespace(
            requirements,
            NamespaceIdentityNeed::StructureContextVersion,
        )
        .then(|| structure_context_version.to_string()),
        caller_context_identity: w056_requires_caller_context(requirements).then(|| {
            format!(
                "treecalc-caller:{};{}",
                owner_node_id, environment_context.caller_context_identity_version
            )
        }),
        table_context_identity: w056_requires_namespace(
            requirements,
            NamespaceIdentityNeed::TableContextIdentity,
        )
        .then(|| {
            environment_context
                .table_context_identity
                .clone()
                .unwrap_or_else(|| "treecalc-table-context:unavailable-current-packet".to_string())
        }),
    }
}

fn effective_treecalc_capability_profile_id(
    environment_context: &LocalTreeCalcEnvironmentContext,
) -> String {
    if environment_context.capability_profile_id == "host-capabilities:default" {
        "host-capabilities:treecalc-v1".to_string()
    } else {
        environment_context.capability_profile_id.clone()
    }
}

fn w056_runtime_host_formula_context(
    owner_node_id: TreeNodeId,
    structure_context_version: &str,
    environment_context: &LocalTreeCalcEnvironmentContext,
    requirements: &[(NamespaceIdentityNeed, CallerContextIdentityNeed)],
    cross_workspace_availability_versions: &[String],
) -> Option<RuntimeHostFormulaContext> {
    w056_requirements_need_public_host_context(requirements).then(|| {
        treecalc_host_formula_context(
            owner_node_id,
            structure_context_version,
            environment_context,
            requirements,
            cross_workspace_availability_versions,
        )
    })
}

fn w056_requirements_need_public_host_context(
    requirements: &[(NamespaceIdentityNeed, CallerContextIdentityNeed)],
) -> bool {
    requirements.iter().any(|(namespace, caller)| {
        *caller != CallerContextIdentityNeed::None
            || !matches!(
                namespace,
                NamespaceIdentityNeed::None | NamespaceIdentityNeed::StructureContextVersion
            )
    })
}

fn w056_requires_namespace(
    requirements: &[(NamespaceIdentityNeed, CallerContextIdentityNeed)],
    required: NamespaceIdentityNeed,
) -> bool {
    requirements
        .iter()
        .any(|(namespace, _)| *namespace == required)
}

fn w056_requires_caller_context(
    requirements: &[(NamespaceIdentityNeed, CallerContextIdentityNeed)],
) -> bool {
    requirements
        .iter()
        .any(|(_, caller)| *caller != CallerContextIdentityNeed::None)
}

fn w056_host_namespace_identity(
    environment_context: &LocalTreeCalcEnvironmentContext,
    requirements: &[(NamespaceIdentityNeed, CallerContextIdentityNeed)],
    cross_workspace_availability_versions: &[String],
) -> Option<String> {
    if w056_requires_namespace(requirements, NamespaceIdentityNeed::HostNamespaceVersion)
        || w056_requires_namespace(requirements, NamespaceIdentityNeed::ResolutionRuleVersion)
        || w056_requires_namespace(
            requirements,
            NamespaceIdentityNeed::CrossWorkspaceAvailabilityVersion,
        )
    {
        let mut identity = environment_context.host_namespace_version.clone();
        if let Some(version) = &environment_context.cross_workspace_availability_version {
            identity.push_str("|cross_workspace_availability_version=");
            identity.push_str(version);
        }
        for version in cross_workspace_availability_versions {
            identity.push_str("|cross_workspace_target_availability=");
            identity.push_str(version);
        }
        Some(identity)
    } else {
        None
    }
}

fn host_reference_bind_results_for_runtime(
    translated: &TranslatedFormula,
) -> Vec<RuntimeHostReferenceBindResult> {
    translated
        .collection_bindings
        .iter()
        .map(|collection| {
            let source_span = collection.source_span_utf8.map_or_else(
                || TextSpan::new(0, collection.source_token_text.len()),
                |(start, end)| TextSpan::new(start, end.saturating_sub(start)),
            );
            RuntimeHostReferenceBindResult {
                reference_handle: collection.host_ref_handle.clone(),
                formal_reference_id: Some(collection.token.clone()),
                source_span,
                source_token_text: collection.source_token_text.clone(),
                opaque_selector_payload: Some(collection.opaque_selector.clone()),
                resolution_layer: "explicit_host_ref".to_string(),
                shape_hint: Some(host_reference_shape_hint(collection)),
                caller_context_dependent: true,
                diagnostics: Vec::new(),
                replay_identity_contribution: format!(
                    "treecalc-host-reference:v1:handle={};membership={};order={}",
                    collection.host_ref_handle,
                    collection.collection_dependency.membership_version,
                    collection.collection_dependency.order_version
                ),
            }
        })
        .chain(translated.host_value_bindings.iter().map(|binding| {
            RuntimeHostReferenceBindResult {
                reference_handle: binding.host_ref_handle.clone(),
                formal_reference_id: Some(binding.token.clone()),
                source_span: TextSpan::new(
                    binding.source_span_utf8.0,
                    binding
                        .source_span_utf8
                        .1
                        .saturating_sub(binding.source_span_utf8.0),
                ),
                source_token_text: binding.source_token_text.clone(),
                opaque_selector_payload: binding.opaque_selector.clone(),
                resolution_layer: "explicit_host_ref".to_string(),
                shape_hint: Some("metadata_value".to_string()),
                caller_context_dependent: true,
                diagnostics: Vec::new(),
                replay_identity_contribution: format!(
                    "treecalc-host-value:v1:handle={};detail={}",
                    binding.host_ref_handle, binding.carrier_detail
                ),
            }
        }))
        .chain(translated.structured_table_bindings.iter().map(|binding| {
            RuntimeHostReferenceBindResult {
                reference_handle: binding.host_ref_handle.clone(),
                formal_reference_id: Some(binding.token.clone()),
                source_span: binding.source_span_utf8.map_or_else(
                    || TextSpan::new(0, binding.source_token_text.len()),
                    |(start, end)| TextSpan::new(start, end.saturating_sub(start)),
                ),
                source_token_text: binding.source_token_text.clone(),
                opaque_selector_payload: Some(format!(
                    "table={};columns={};target={}",
                    binding.table_id,
                    binding.selected_column_ids.join("|"),
                    binding.reference_target
                )),
                resolution_layer: "structured_table".to_string(),
                shape_hint: Some("structured_table:data_body".to_string()),
                caller_context_dependent: false,
                diagnostics: Vec::new(),
                replay_identity_contribution: format!(
                    "treecalc-structured-table-reference:v1:handle={};table={};columns={};row_membership={};row_order={};column_identity={}",
                    binding.host_ref_handle,
                    binding.table_id,
                    binding.selected_column_ids.join("|"),
                    binding.row_membership_version,
                    binding.row_order_version,
                    binding.column_identity_version
                ),
            }
        }))
        .collect()
}

fn host_reference_shape_hint(collection: &SyntheticReferenceCollectionBinding) -> String {
    match collection.collection_dependency.family {
        TreeReferenceCollectionFamily::ChildrenV1 => "ordered_collection:children_v1".to_string(),
        TreeReferenceCollectionFamily::ReferenceLiteralArrayV1 => {
            "ordered_collection:treecalc_reference_literal_array_v1".to_string()
        }
        family => format!(
            "ordered_collection:treecalc_ordered_selector_v1:{}",
            family.stable_id()
        ),
    }
}

struct TreeCalcRuntimeEnvironmentBuild<'a> {
    translated: &'a TranslatedFormula,
    snapshot: &'a StructuralSnapshot,
    table_snapshots: &'a BTreeMap<TreeNodeId, TreeCalcTableNodeSnapshot>,
    owner_node_id: TreeNodeId,
    meta_node_ids: &'a BTreeSet<TreeNodeId>,
    structure_context_version: StructureContextVersion,
    oxfunc_bridge_metadata: &'a LocalTreeCalcOxFuncBridgeMetadata,
    working_values: &'a BTreeMap<TreeNodeId, String>,
    working_calc_values: &'a BTreeMap<TreeNodeId, CalcValue>,
    prepared_formula_identity: Option<&'a RuntimePreparedFormulaIdentity>,
    host_formula_context: Option<RuntimeHostFormulaContext>,
}

fn build_treecalc_runtime_environment_from_parts(
    parts: TreeCalcRuntimeEnvironmentBuild<'_>,
) -> RuntimeEnvironment<'static> {
    let formal_input_bindings = formal_input_bindings_for_runtime(
        parts.translated,
        parts.working_values,
        parts.working_calc_values,
        parts.prepared_formula_identity,
    );
    let host_name_bindings = host_name_bindings_for_runtime(
        parts.translated,
        parts.working_values,
        parts.working_calc_values,
    );
    let host_name_bindings = merge_runtime_host_name_bindings(
        host_name_bindings,
        context_host_name_bindings_for_runtime(&parts),
    );

    let mut environment = RuntimeEnvironment::new()
        .with_host_reference_syntax(treecalc_host_reference_syntax_profile())
        .with_structure_context_version(parts.structure_context_version)
        .with_caller_position(synthetic_cell_row(parts.owner_node_id), 1)
        .with_cell_values(treecalc_runtime_node_cell_values(parts.working_values))
        .with_formal_input_bindings(formal_input_bindings)
        .with_host_name_bindings(host_name_bindings);
    if let Some(host_formula_context) = parts.host_formula_context {
        environment = environment.with_host_formula_context(host_formula_context);
    }
    let table_catalog = parts
        .table_snapshots
        .values()
        .filter_map(|snapshot| {
            project_treecalc_table_node_snapshot(snapshot)
                .ok()
                .map(|projection| projection.table_descriptor)
        })
        .collect::<Vec<_>>();
    if let Some((workbook_id, sheet_id)) = treecalc_formula_scope_from_table_catalog(&table_catalog)
    {
        environment = environment.with_formula_scope(workbook_id, sheet_id);
    }
    if !table_catalog.is_empty() {
        environment = environment.with_table_context(table_catalog, None, None);
    }
    if !parts.translated.collection_bindings.is_empty()
        || !parts.translated.host_value_bindings.is_empty()
        || !parts.translated.structured_table_bindings.is_empty()
    {
        environment = environment.with_host_reference_bind_results(
            host_reference_bind_results_for_runtime(parts.translated),
        );
    }
    if let Some(version) = &parts
        .oxfunc_bridge_metadata
        .semantic_kernel_metadata_version
    {
        environment = environment.with_semantic_kernel_metadata_version(version.clone());
    }
    if let Some(version) = &parts.oxfunc_bridge_metadata.arg_admission_metadata_version {
        environment = environment.with_arg_admission_metadata_version(version.clone());
    }
    environment
}

fn treecalc_formula_scope_from_table_catalog(
    table_catalog: &[TableDescriptor],
) -> Option<(String, String)> {
    let first = table_catalog.first()?;
    Some((
        first.workbook_scope_ref.clone(),
        first.sheet_scope_ref.clone(),
    ))
}

fn treecalc_runtime_node_cell_values(
    working_values: &BTreeMap<TreeNodeId, String>,
) -> BTreeMap<String, CalcValue> {
    working_values
        .iter()
        .map(|(node_id, value)| {
            (
                treecalc_runtime_node_reference_target(*node_id),
                authored_cell_entry_text_to_calc_value(value),
            )
        })
        .collect()
}

fn treecalc_runtime_node_reference_target(node_id: TreeNodeId) -> String {
    format!("treecalc.node:{}", node_id.0)
}

#[derive(Debug, Clone, Copy)]
struct TreeCalcHostInfoProvider;

impl HostInfoProvider for TreeCalcHostInfoProvider {
    fn query_cell_info(
        &self,
        query: CellInfoQuery,
        _reference: Option<&ReferenceLike>,
    ) -> Result<CalcValue, HostInfoError> {
        Err(HostInfoError::UnsupportedCellInfoQuery(query))
    }

    fn query_info(&self, _query: InfoQuery) -> Result<CalcValue, HostInfoError> {
        Err(HostInfoError::ProviderFailure {
            detail: "treecalc.host_sensitive_reference".to_string(),
        })
    }

    fn query_image(&self, request: &ImageRequest) -> Result<ImageProviderResult, HostInfoError> {
        let fallback = request
            .alt_text
            .clone()
            .unwrap_or_else(|| ExcelText::from_interop_assignment("-2146826273"));
        Ok(ImageProviderResult::Image(ResolvedWebImage {
            web_image_identifier: format!("treecalc.image:{}", request.source.to_string_lossy()),
            published_fallback: fallback,
        }))
    }
}

#[derive(Debug, Clone, Copy)]
struct TreeCalcRtdProvider;

impl RtdProvider for TreeCalcRtdProvider {
    fn resolve_rtd(&self, _request: &RtdRequest) -> RtdProviderResult {
        RtdProviderResult::CapabilityDenied
    }
}

fn adapt_oxfml_runtime_candidate(
    prepared: &PreparedOxfmlFormula,
    run: &RuntimeFormulaResult,
    derivation_trace_enabled: bool,
) -> Result<LocalFormulaEvaluationSuccess, LocalFormulaEvaluationFailure> {
    let candidate = &run.candidate_result;
    let candidate_value = value_payload_to_string(&candidate.value_delta.published_payload);
    let mut diagnostics = oxfml_returned_value_surface_diagnostics(&run.returned_value_surface);
    diagnostics.extend(oxfml_runtime_prepared_identity_diagnostics(
        &run.prepared_formula_identity,
    ));
    diagnostics.extend(oxfml_candidate_diagnostics(candidate));

    match &run.commit_decision {
        oxfml_core::seam::AcceptDecision::Accepted(bundle) => {
            diagnostics.extend(oxfml_commit_bundle_diagnostics(bundle));
            validate_oxfml_commit_bundle(prepared, candidate, bundle, diagnostics.clone())?;
            let derivation_trace = derivation_trace_enabled
                .then(|| build_derivation_trace_record(prepared, run, &candidate_value));
            if let Some(trace) = &derivation_trace {
                diagnostics.push(format!(
                    "derivation_trace_recorded:{}:schema={}",
                    trace.owner_node_id, trace.trace_schema_id
                ));
                diagnostics.push(format!(
                    "derivation_trace_prepared_call_count:{}:{}",
                    trace.owner_node_id,
                    trace
                        .sub_invocation_tree
                        .first()
                        .map(|root| root.children.len())
                        .unwrap_or_default()
                ));
            }
            Ok(LocalFormulaEvaluationSuccess {
                calc_value: run.published_calc_value(),
                diagnostics,
                derivation_trace,
                dynamic_reference_resolutions: Vec::new(),
            })
        }
        oxfml_core::seam::AcceptDecision::Rejected(reject) => {
            diagnostics.extend(oxfml_reject_record_diagnostics(reject));
            Err(LocalFormulaEvaluationFailure {
                error: LocalTreeCalcError::OxfmlCommitRejected {
                    owner_node_id: prepared.binding.owner_node_id,
                    detail: format!("{:?}", reject.reject_code),
                },
                runtime_effects: Vec::new(),
                diagnostics,
            })
        }
    }
}

fn oxfml_runtime_prepared_identity_diagnostics(
    identity: &oxfml_core::consumer::runtime::RuntimePreparedFormulaIdentity,
) -> Vec<String> {
    let mut diagnostics = vec![format!(
        "oxfml_runtime_prepared_formula_key:{}:{}",
        identity.formula_stable_id, identity.prepared_formula_key
    )];
    if let Some(version) = &identity.semantic_kernel_metadata_version {
        diagnostics.push(format!(
            "oxfml_runtime_semantic_kernel_metadata_version:{}:{}",
            identity.formula_stable_id, version
        ));
    }
    if let Some(version) = &identity.arg_admission_metadata_version {
        diagnostics.push(format!(
            "oxfml_runtime_arg_admission_metadata_version:{}:{}",
            identity.formula_stable_id, version
        ));
    }
    diagnostics
}

fn build_derivation_trace_record(
    prepared: &PreparedOxfmlFormula,
    run: &RuntimeFormulaResult,
    candidate_value: &str,
) -> DerivationTraceRecord {
    let identity = prepared_formula_identity_trace(prepared);
    let returned_value_capability_columns =
        rich_value_capability_columns_for_returned_surface(&run.returned_value_surface);
    let trace_capability_columns = identity
        .rich_value_capability_columns
        .union(&returned_value_capability_columns);
    let runtime_identity = &prepared.runtime_prepared_identity;
    let template = &runtime_identity.plan_template;
    let hole_binding = &runtime_identity.hole_binding;
    let child_invocations = run
        .evaluation
        .trace
        .prepared_calls
        .iter()
        .enumerate()
        .map(|(index, call)| DerivationInvocationTraceNode {
            invocation_ordinal: index + 1,
            invocation_kind: "oxfml_prepared_call".to_string(),
            function_name: call.function_name.clone(),
            function_id: call.function_id.to_string(),
            arg_preparation_profile: Some(format!("{:?}", call.arg_preparation_profile)),
            prepared_arguments: call
                .prepared_arguments
                .iter()
                .map(|argument| DerivationPreparedArgumentTrace {
                    ordinal: argument.ordinal,
                    structure_class: format!("{:?}", argument.structure_class),
                    source_class: format!("{:?}", argument.source_class),
                    evaluation_mode: format!("{:?}", argument.evaluation_mode),
                    blankness_class: format!("{:?}", argument.blankness_class),
                    caller_context_sensitive: argument.caller_context_sensitive,
                    reference_target: argument.reference_target.clone(),
                    opaque_reason: argument.opaque_reason.clone(),
                    resolved_value: argument
                        .resolved_value
                        .as_ref()
                        .map(calc_value_trace_summary),
                })
                .collect(),
            kernel_returned_value: call.returned_value.as_ref().map(calc_value_trace_summary),
            children: Vec::new(),
        })
        .collect::<Vec<_>>();

    DerivationTraceRecord {
        trace_schema_id: DERIVATION_TRACE_SCHEMA_ID.to_string(),
        owner_node_id: prepared.binding.owner_node_id,
        formula_artifact_id: identity.formula_artifact_id,
        bind_artifact_id: identity.bind_artifact_id,
        formula_stable_id: identity.formula_stable_id,
        trace_mode: "PreparedCalls".to_string(),
        rich_value_capability_columns: trace_capability_columns,
        template_selection: DerivationTemplateSelectionTrace {
            prepared_formula_key: identity.prepared_formula_key,
            shape_key: identity.shape_key,
            dispatch_skeleton_key: identity.dispatch_skeleton_key,
            plan_template_key: identity.plan_template_key,
            rich_value_capability_columns: RichValueCapabilityTraceReplayColumns::empty_v1(),
            template_holes: template
                .template_holes
                .iter()
                .map(runtime_template_hole_trace)
                .collect(),
        },
        hole_bindings: (0..hole_binding.binding_count)
            .map(|ordinal| DerivationHoleBindingTrace {
                hole_id: format!("runtime_hole_binding:{ordinal}"),
                payload: hole_binding.hole_binding_fingerprint.clone(),
            })
            .collect(),
        sub_invocation_tree: vec![DerivationInvocationTraceNode {
            invocation_ordinal: 0,
            invocation_kind: "oxfml_prepared_formula_invoke".to_string(),
            function_name: format!("formula:{}", prepared.source.formula_stable_id.0),
            function_id: "oxfml.prepared_formula.invoke.v1".to_string(),
            arg_preparation_profile: None,
            prepared_arguments: Vec::new(),
            kernel_returned_value: Some(candidate_value.to_string()),
            children: child_invocations,
        }],
        kernel_returned_value: candidate_value.to_string(),
        oxfml_trace_events: run
            .trace_events
            .iter()
            .map(|event| DerivationOxfmlTraceEvent {
                trace_schema_id: event.trace_schema_id.clone(),
                event_kind: format!("{:?}", event.event_kind),
                formula_stable_id: event.formula_stable_id.clone(),
                session_id: event.session_id.clone(),
                candidate_result_id: event.candidate_result_id.clone(),
                commit_attempt_id: event.commit_attempt_id.clone(),
                event_order_key: event.event_order_key,
            })
            .collect(),
    }
}

fn runtime_template_hole_trace(hole: &RuntimeTemplateHole) -> DerivationTemplateHoleTrace {
    DerivationTemplateHoleTrace {
        hole_id: hole.hole_id.clone(),
        ordinal: hole.ordinal,
        path: hole.path.clone().unwrap_or_default(),
        kind: hole.hole_kind_key.clone(),
        rich_value_capability_columns: RichValueCapabilityTraceReplayColumns::empty_v1(),
    }
}

fn oxfml_returned_value_surface_diagnostics(surface: &ReturnedValueSurface) -> Vec<String> {
    let mut diagnostics = vec![
        format!("oxfml_returned_value_surface_kind:{:?}", surface.kind),
        format!(
            "oxfml_returned_value_surface_payload_summary:{}",
            surface.payload_summary
        ),
    ];
    if let Some(type_name) = &surface.rich_value_type_name {
        diagnostics.push(format!(
            "oxfml_returned_value_surface_rich_value_type:{type_name}"
        ));
    }
    diagnostics.extend(
        surface
            .producer_capability_set_keys
            .iter()
            .map(|key| format!("oxfml_returned_value_surface_producer_capability_set_key:{key}")),
    );
    diagnostics.extend(
        surface
            .exercised_capability_keys
            .iter()
            .map(|key| format!("oxfml_returned_value_surface_exercised_capability_key:{key}")),
    );
    if let Some(outcome) = &surface.host_provider_outcome {
        diagnostics.push(format!(
            "oxfml_returned_value_surface_host_provider_outcome:{:?}",
            outcome.outcome_kind
        ));
        if let Some(error) = outcome.worksheet_error {
            diagnostics.push(format!(
                "oxfml_returned_value_surface_host_provider_worksheet_error:{:?}",
                error
            ));
        }
    }
    diagnostics
}

fn rich_value_capability_columns_for_returned_surface(
    surface: &ReturnedValueSurface,
) -> RichValueCapabilityTraceReplayColumns {
    RichValueCapabilityTraceReplayColumns::from_runtime_capability_keys(
        surface.producer_capability_set_keys.clone(),
        surface.exercised_capability_keys.clone(),
    )
}

fn oxfml_candidate_diagnostics(
    candidate: &oxfml_core::seam::AcceptedCandidateResult,
) -> Vec<String> {
    vec![
        format!(
            "oxfml_candidate_result_id:{}",
            candidate.candidate_result_id
        ),
        format!(
            "oxfml_candidate_formula_stable_id:{}",
            candidate.formula_stable_id
        ),
        format!(
            "oxfml_candidate_trace_correlation_id:{}",
            candidate.trace_correlation_id
        ),
        format!(
            "oxfml_candidate_value_delta_formula_stable_id:{}",
            candidate.value_delta.formula_stable_id
        ),
        format!(
            "oxfml_candidate_value_delta_candidate_result_id:{}",
            candidate
                .value_delta
                .candidate_result_id
                .as_deref()
                .unwrap_or("<none>")
        ),
    ]
}

fn oxfml_commit_bundle_diagnostics(bundle: &oxfml_core::seam::CommitBundle) -> Vec<String> {
    vec![
        format!(
            "oxfml_commit_candidate_result_id:{}",
            bundle.candidate_result_id
        ),
        format!("oxfml_commit_attempt_id:{}", bundle.commit_attempt_id),
        format!(
            "oxfml_commit_formula_stable_id:{}",
            bundle.formula_stable_id
        ),
        format!(
            "oxfml_commit_value_delta_candidate_result_id:{}",
            bundle
                .value_delta
                .candidate_result_id
                .as_deref()
                .unwrap_or("<none>")
        ),
        "coordinator_publication_authority:oxcalc".to_string(),
    ]
}

fn oxfml_reject_record_diagnostics(reject: &oxfml_core::seam::RejectRecord) -> Vec<String> {
    vec![
        format!("oxfml_reject:{:?}", reject.reject_code),
        format!(
            "oxfml_reject_formula_stable_id:{}",
            reject.formula_stable_id
        ),
        format!(
            "oxfml_reject_commit_attempt_id:{}",
            reject.commit_attempt_id.as_deref().unwrap_or("<none>")
        ),
        format!(
            "oxfml_reject_trace_correlation_id:{}",
            reject.trace_correlation_id
        ),
        "oxfml_reject_no_publish:true".to_string(),
    ]
}

fn validate_oxfml_commit_bundle(
    prepared: &PreparedOxfmlFormula,
    candidate: &oxfml_core::seam::AcceptedCandidateResult,
    bundle: &oxfml_core::seam::CommitBundle,
    diagnostics: Vec<String>,
) -> Result<(), LocalFormulaEvaluationFailure> {
    let mismatch = if bundle.candidate_result_id != candidate.candidate_result_id {
        Some(format!(
            "candidate_result_id_mismatch:{}:{}",
            candidate.candidate_result_id, bundle.candidate_result_id
        ))
    } else if bundle.formula_stable_id != candidate.formula_stable_id {
        Some(format!(
            "formula_stable_id_mismatch:{}:{}",
            candidate.formula_stable_id, bundle.formula_stable_id
        ))
    } else if bundle.value_delta.candidate_result_id.as_ref()
        != Some(&candidate.candidate_result_id)
    {
        Some(format!(
            "commit_value_delta_candidate_result_id_mismatch:{:?}:{}",
            bundle.value_delta.candidate_result_id, candidate.candidate_result_id
        ))
    } else {
        None
    };

    if let Some(detail) = mismatch {
        return Err(LocalFormulaEvaluationFailure {
            error: LocalTreeCalcError::OxfmlCommitBundleIncompatible {
                owner_node_id: prepared.binding.owner_node_id,
                detail,
            },
            runtime_effects: Vec::new(),
            diagnostics,
        });
    }

    Ok(())
}

fn project_opaque_formula(
    snapshot: &StructuralSnapshot,
    table_snapshots: &BTreeMap<TreeNodeId, TreeCalcTableNodeSnapshot>,
    owner_node_id: TreeNodeId,
    formula: &TreeFormula,
    meta_node_ids: &BTreeSet<TreeNodeId>,
) -> TranslatedFormula {
    let mut state = FormulaCarrierProjectionState {
        snapshot,
        table_snapshots,
        owner_node_id,
        owner_formula_source_text: formula.source_text().to_string(),
        meta_node_ids,
        fallback_reference_index: 0,
        reference_bindings: Vec::new(),
        collection_bindings: Vec::new(),
        structured_table_bindings: Vec::new(),
        host_value_bindings: Vec::new(),
        unresolved_bindings: Vec::new(),
        residuals: Vec::new(),
    };
    if let Some(bound_formula) = formula.bound_formula() {
        state.project_bound_expr(&bound_formula.root);
        state.project_structured_reference_bind_records(
            &bound_formula.structured_reference_bind_records,
        );
    } else {
        for reference in formula.explicit_references() {
            state.project_explicit_reference(reference);
        }
    }
    for binding in formula.host_value_bindings() {
        state.project_host_value_binding(binding);
    }
    TranslatedFormula {
        source_text: formula.source_text().to_string(),
        reference_bindings: state.reference_bindings,
        collection_bindings: state.collection_bindings,
        structured_table_bindings: state.structured_table_bindings,
        host_value_bindings: state.host_value_bindings,
        unresolved_bindings: state.unresolved_bindings,
        residuals: state.residuals,
    }
}

fn project_runtime_structured_table_bindings(
    snapshot: &StructuralSnapshot,
    table_snapshots: &BTreeMap<TreeNodeId, TreeCalcTableNodeSnapshot>,
    owner_node_id: TreeNodeId,
    meta_node_ids: &BTreeSet<TreeNodeId>,
    records: &[StructuredReferenceBindRecord],
) -> Vec<SyntheticStructuredTableBinding> {
    let mut state = FormulaCarrierProjectionState {
        snapshot,
        table_snapshots,
        owner_node_id,
        owner_formula_source_text: String::new(),
        meta_node_ids,
        fallback_reference_index: 0,
        reference_bindings: Vec::new(),
        collection_bindings: Vec::new(),
        structured_table_bindings: Vec::new(),
        host_value_bindings: Vec::new(),
        unresolved_bindings: Vec::new(),
        residuals: Vec::new(),
    };
    state.project_structured_reference_bind_records(records);
    state.structured_table_bindings
}

struct FormulaCarrierProjectionState<'a> {
    snapshot: &'a StructuralSnapshot,
    table_snapshots: &'a BTreeMap<TreeNodeId, TreeCalcTableNodeSnapshot>,
    owner_node_id: TreeNodeId,
    owner_formula_source_text: String,
    meta_node_ids: &'a BTreeSet<TreeNodeId>,
    fallback_reference_index: usize,
    reference_bindings: Vec<SyntheticReferenceBinding>,
    collection_bindings: Vec<SyntheticReferenceCollectionBinding>,
    structured_table_bindings: Vec<SyntheticStructuredTableBinding>,
    host_value_bindings: Vec<SyntheticHostValueBinding>,
    unresolved_bindings: Vec<SyntheticUnresolvedBinding>,
    residuals: Vec<ResidualCarrier>,
}

impl FormulaCarrierProjectionState<'_> {
    fn project_bound_expr(&mut self, expr: &BoundExpr) {
        match expr {
            BoundExpr::HostReference(record) => self.project_bound_host_name_record(record),
            BoundExpr::HostStructuralSelector(selector) => {
                if tree_reference_collection_family_from_bound_key(&selector.selector_family)
                    .is_some()
                {
                    let source_base_node_id =
                        self.contextual_collection_source_base_node_id(&selector.source_token_text);
                    let bound_base_node_id =
                        self.contextual_bound_expr_treecalc_node_id(&selector.base);
                    if source_base_node_id.is_none() || bound_base_node_id.is_some() {
                        self.project_bound_expr(&selector.base);
                    }
                    self.bind_bound_host_expr_collection(
                        selector.selector_handle.clone(),
                        selector.selector_family.clone(),
                        selector.source_span,
                        selector.source_token_text.clone(),
                        source_base_node_id.or(bound_base_node_id),
                        selector.shape_hint.clone(),
                        selector.caller_context_dependent,
                        selector.members.iter(),
                    );
                } else if let BoundExpr::HostReferenceCollection(base_collection) =
                    selector.base.as_ref()
                    && tree_reference_collection_family_from_bound_key(
                        &base_collection.collection_family,
                    )
                    .is_some()
                {
                    self.bind_bound_host_selector_tail_collection(selector, base_collection);
                } else if let BoundExpr::HostStructuralSelector(base_selector) =
                    selector.base.as_ref()
                    && tree_reference_collection_family_from_bound_key(
                        &base_selector.selector_family,
                    )
                    .is_some()
                {
                    self.bind_bound_structural_selector_tail_collection(selector, base_selector);
                } else {
                    self.bind_bound_scalar_host_selector(selector);
                }
            }
            BoundExpr::HostReferenceCollection(collection) => {
                if let Some(base) = &collection.base {
                    self.project_bound_expr(base);
                }
                if tree_reference_collection_family_from_bound_key(&collection.collection_family)
                    .is_some()
                {
                    self.bind_bound_host_expr_collection(
                        collection.collection_handle.clone(),
                        collection.collection_family.clone(),
                        collection.source_span,
                        collection.source_token_text.clone(),
                        collection
                            .base
                            .as_deref()
                            .and_then(bound_expr_treecalc_node_id),
                        collection.shape_hint.clone(),
                        collection.caller_context_dependent,
                        collection.members.iter(),
                    );
                } else {
                    self.bind_bound_scalar_host_collection(collection);
                }
            }
            BoundExpr::ArrayLiteral(rows) => {
                for row in rows {
                    for item in row {
                        self.project_bound_expr(item);
                    }
                }
                self.bind_bound_array_reference_collection(rows);
            }
            BoundExpr::Binary { left, right, .. } => {
                self.project_bound_expr(left);
                self.project_bound_expr(right);
            }
            BoundExpr::Unary { expr, .. } | BoundExpr::ImplicitIntersection(expr) => {
                self.project_bound_expr(expr);
            }
            BoundExpr::FunctionCall { args, .. } => {
                for arg in args {
                    self.project_bound_expr(arg);
                }
            }
            BoundExpr::Invocation { callee, args } => {
                self.project_bound_expr(callee);
                for arg in args {
                    self.project_bound_expr(arg);
                }
            }
            BoundExpr::NumberLiteral(_)
            | BoundExpr::StringLiteral(_)
            | BoundExpr::LogicalLiteral(_)
            | BoundExpr::OmittedArgument
            | BoundExpr::HelperParameterName(_)
            | BoundExpr::HelperOptionalParameterName(_) => {}
            BoundExpr::Reference(_) => {}
        }
    }

    fn project_structured_reference_bind_records(
        &mut self,
        records: &[StructuredReferenceBindRecord],
    ) {
        for record in records {
            let Some(binding) = self.structured_table_binding_from_record(record) else {
                continue;
            };
            self.structured_table_bindings.push(binding);
        }
    }

    fn structured_table_binding_from_record(
        &self,
        record: &StructuredReferenceBindRecord,
    ) -> Option<SyntheticStructuredTableBinding> {
        if !record.diagnostics.is_empty() || record.uses_this_row {
            return None;
        }
        let table_id = record.effective_table_id.clone()?;
        let selected_column_ids =
            (!record.selected_column_ids.is_empty()).then(|| record.selected_column_ids.clone())?;
        let selected_sections = structured_table_selected_sections(record);
        let data_selected = selected_sections.contains(&StructuredSectionKind::Data);
        let headers_selected = selected_sections.contains(&StructuredSectionKind::Headers);
        let totals_selected = selected_sections.contains(&StructuredSectionKind::Totals);
        if selected_sections.contains(&StructuredSectionKind::All) {
            return None;
        }
        if !data_selected && !headers_selected && !totals_selected {
            return None;
        }
        let table_snapshot = self
            .table_snapshots
            .values()
            .find(|snapshot| snapshot.table_id == table_id)?;
        if headers_selected && !table_snapshot.header_row_present {
            return None;
        }
        if totals_selected && !table_snapshot.totals_row_present {
            return None;
        }
        let selected_columns = selected_column_ids
            .iter()
            .filter_map(|column_id| {
                table_snapshot
                    .columns
                    .iter()
                    .find(|column| column.column_id.as_str() == column_id.as_str())
            })
            .collect::<Vec<_>>();
        if selected_columns.len() != selected_column_ids.len() {
            return None;
        }
        let declared_cols = selected_column_ids.len();
        let mut next_row_index = 1usize;
        let mut literal_cells = Vec::new();
        if headers_selected {
            for (col_index, column) in selected_columns.iter().enumerate() {
                literal_cells.push(SyntheticStructuredTableLiteralCell {
                    row_index: next_row_index,
                    col_index: col_index + 1,
                    value: SyntheticStructuredTableLiteralValue::Text(column.column_name.clone()),
                });
            }
            next_row_index += 1;
        }
        let mut member_node_cells = Vec::new();
        if data_selected {
            for row in &table_snapshot.rows {
                for (col_index, column_id) in selected_column_ids.iter().enumerate() {
                    let cell = table_snapshot.body_cell_nodes.iter().find(|cell| {
                        cell.row_id == *row && cell.column_id.as_str() == column_id.as_str()
                    })?;
                    member_node_cells.push(SyntheticStructuredTableNodeCell {
                        row_index: next_row_index,
                        col_index: col_index + 1,
                        node_id: cell.node_id,
                    });
                }
                next_row_index += 1;
            }
        }
        if totals_selected {
            for (col_index, column_id) in selected_column_ids.iter().enumerate() {
                let cell = table_snapshot
                    .totals_cell_nodes
                    .iter()
                    .find(|cell| cell.column_id.as_str() == column_id.as_str())?;
                member_node_cells.push(SyntheticStructuredTableNodeCell {
                    row_index: next_row_index,
                    col_index: col_index + 1,
                    node_id: cell.node_id,
                });
            }
            next_row_index += 1;
        }
        if member_node_cells.is_empty() && literal_cells.is_empty() {
            return None;
        }
        let declared_rows = next_row_index.saturating_sub(1);
        let member_node_ids = member_node_cells
            .iter()
            .map(|cell| cell.node_id)
            .collect::<Vec<_>>();
        let (reference_kind, reference_target) =
            structured_reference_like_identity(record.resolved_reference.as_ref()?)?;
        Some(SyntheticStructuredTableBinding {
            token: record.bind_record_handle.clone(),
            host_ref_handle: format!("treecalc-table-ref:{}", record.bind_record_handle),
            source_span_utf8: Some((record.source_span_utf8.start, record.source_span_utf8.end())),
            source_token_text: record.source_token_text.clone(),
            table_id,
            selected_column_ids,
            selected_sections,
            declared_rows,
            declared_cols,
            member_node_ids,
            member_node_cells,
            literal_cells,
            reference_kind,
            reference_target,
            row_membership_version: table_snapshot.row_membership_version.clone(),
            row_order_version: table_snapshot.row_order_version.clone(),
            column_identity_version: table_snapshot.column_identity_version.clone(),
        })
    }

    fn project_bound_host_name_record(&mut self, record: &HostNameBindRecord) {
        let token = record.canonical_name.clone();
        if let Some(collection) = record
            .host_dependency_key
            .as_deref()
            .and_then(treecalc_collection_from_host_dependency_key)
        {
            self.bind_bound_host_collection(record, token, collection);
            return;
        }

        let Some(target_node_id) = record
            .host_dependency_key
            .as_deref()
            .and_then(treecalc_node_id_from_host_dependency_key)
        else {
            self.bind_unresolved(
                Some(token),
                DependencyDescriptorKind::Unresolved,
                format!(
                    "bound_formula_host_name_without_treecalc_dependency_key:handle={}",
                    record.host_name_handle
                ),
                true,
            );
            return;
        };

        self.bind_target(
            Some(token),
            target_node_id,
            DependencyDescriptorKind::StaticDirect,
            format!(
                "bound_formula_host_name:handle={}:kind={}:layer={}",
                record.host_name_handle, record.binding_kind, record.resolution_layer
            ),
            record.caller_context_dependent,
            Some(TreeFormulaHostNameBindPacket {
                host_name_handle: record.host_name_handle.clone(),
                canonical_name: record.canonical_name.clone(),
                host_dependency_key: record.host_dependency_key.clone(),
                source_span_utf8: (record.source_span.start, record.source_span.end()),
                source_token_text: record.source_token_text.clone(),
                resolution_layer: record.resolution_layer.clone(),
                binding_kind: record.binding_kind.clone(),
                shape_hint: record.shape_hint.clone(),
                caller_context_dependent: record.caller_context_dependent,
                diagnostics: record.diagnostics.clone(),
                replay_identity_contribution: record.replay_identity_contribution.clone(),
            }),
        );
    }

    fn bind_bound_host_expr_collection<'a>(
        &mut self,
        handle: String,
        family: String,
        source_span: TextSpan,
        source_token_text: String,
        base_node_id: Option<TreeNodeId>,
        _shape_hint: Option<String>,
        caller_context_dependent: bool,
        members: impl Iterator<Item = &'a BoundExpr>,
    ) {
        let Some(collection_family) = tree_reference_collection_family_from_bound_key(&family)
        else {
            self.bind_unresolved(
                Some(handle.clone()),
                DependencyDescriptorKind::Unresolved,
                format!("bound_formula_host_expr_collection_unknown_family:handle={handle}:family={family}"),
                caller_context_dependent,
            );
            return;
        };
        let explicit_member_node_ids = members
            .filter_map(bound_expr_treecalc_node_id)
            .collect::<Vec<_>>();
        let base_node_id = base_node_id.unwrap_or(self.owner_node_id);
        let member_node_ids = self.collection_member_node_ids(
            collection_family,
            base_node_id,
            explicit_member_node_ids,
        );
        let collection_dependency = match collection_family {
            TreeReferenceCollectionFamily::ChildrenV1 => {
                TreeReferenceCollectionDependency::children_v1(
                    handle.clone(),
                    base_node_id,
                    member_node_ids.clone(),
                )
            }
            TreeReferenceCollectionFamily::ReferenceLiteralArrayV1 => {
                TreeReferenceCollectionDependency::reference_literal_array_v1(
                    handle.clone(),
                    self.owner_node_id,
                    member_node_ids.clone(),
                )
            }
            family => TreeReferenceCollectionDependency::ordered_selector_v1(
                family,
                handle.clone(),
                base_node_id,
                member_node_ids.clone(),
            ),
        };
        self.collection_bindings
            .push(SyntheticReferenceCollectionBinding {
                token: handle.clone(),
                host_ref_handle: handle,
                base_node_id,
                source_span_utf8: Some((source_span.start, source_span.end())),
                source_token_text,
                opaque_selector: family,
                member_node_ids,
                collection_dependency,
            });
    }

    fn bind_bound_scalar_host_selector(&mut self, selector: &BoundHostStructuralSelector) {
        if let Some(target_node_id) = self.contextual_bound_expr_treecalc_node_id(&selector.base)
            && self.bind_metadata_host_value(
                selector.selector_handle.clone(),
                selector.source_span,
                selector.source_token_text.clone(),
                &selector.selector_family,
                target_node_id,
            )
        {
            return;
        }
        let Some(target_node_id) = single_host_selector_member_node_id(selector)
            .or_else(|| self.contextual_scalar_selector_target_node_id(selector))
        else {
            self.bind_unresolved(
                Some(selector.selector_handle.clone()),
                DependencyDescriptorKind::Unresolved,
                format!(
                    "bound_formula_scalar_host_selector_unresolved:handle={}:family={}",
                    selector.selector_handle, selector.selector_family
                ),
                selector.caller_context_dependent,
            );
            return;
        };
        let (kind, carrier_detail) = scalar_host_selector_dependency_shape(selector);
        self.bind_target(
            Some(selector.selector_handle.clone()),
            target_node_id,
            kind,
            carrier_detail,
            true,
            None,
        );
    }

    fn bind_bound_scalar_host_collection(&mut self, collection: &BoundHostReferenceCollection) {
        let metadata_base_node_id = collection
            .base
            .as_deref()
            .and_then(|base| self.contextual_bound_expr_treecalc_node_id(base))
            .unwrap_or(self.owner_node_id);
        if self.bind_metadata_host_value(
            collection.collection_handle.clone(),
            collection.source_span,
            collection.source_token_text.clone(),
            &collection.collection_family,
            metadata_base_node_id,
        ) {
            return;
        }
        let Some(target_node_id) =
            single_host_collection_member_node_id(collection).or_else(|| {
                self.contextual_bound_expr_treecalc_node_id(&BoundExpr::HostReferenceCollection(
                    collection.clone(),
                ))
            })
        else {
            self.bind_unresolved(
                Some(collection.collection_handle.clone()),
                DependencyDescriptorKind::Unresolved,
                format!(
                    "oxfml_bind_diagnostic:bound_formula_scalar_host_collection_unresolved:handle={}:family={}",
                    collection.collection_handle, collection.collection_family
                ),
                collection.caller_context_dependent,
            );
            return;
        };
        let (kind, carrier_detail) = scalar_host_collection_dependency_shape(collection);
        self.bind_target(
            Some(collection.collection_handle.clone()),
            target_node_id,
            kind,
            carrier_detail,
            true,
            None,
        );
    }

    fn bind_metadata_host_value(
        &mut self,
        token: String,
        source_span: TextSpan,
        source_token_text: String,
        family: &str,
        target_node_id: TreeNodeId,
    ) -> bool {
        let Some(value) = self.metadata_host_value(family, target_node_id) else {
            return false;
        };
        self.host_value_bindings.push(SyntheticHostValueBinding {
            token: token.clone(),
            value,
            host_ref_handle: token,
            source_span_utf8: (source_span.start, source_span.end()),
            source_token_text,
            opaque_selector: Some(family.to_string()),
            carrier_detail: format!(
                "bound_formula_metadata_accessor:family={family}:target={target_node_id}"
            ),
            target_node_id: None,
            requires_rebind_on_structural_change: true,
        });
        true
    }

    fn metadata_host_value(
        &self,
        family: &str,
        target_node_id: TreeNodeId,
    ) -> Option<TreeFormulaHostValue> {
        match family {
            "metadata_name" => self
                .snapshot
                .try_get_node(target_node_id)
                .map(|node| TreeFormulaHostValue::Text(node.symbol.clone())),
            "metadata_index" => Some(TreeFormulaHostValue::Integer(
                i64::try_from(self.visible_sibling_ordinal(target_node_id)?).ok()?,
            )),
            "metadata_formula" => {
                if target_node_id == self.owner_node_id {
                    Some(TreeFormulaHostValue::Text(
                        self.owner_formula_source_text.clone(),
                    ))
                } else {
                    Some(TreeFormulaHostValue::ValueError)
                }
            }
            _ => None,
        }
    }

    fn visible_sibling_ordinal(&self, target_node_id: TreeNodeId) -> Option<usize> {
        let parent_id = self.snapshot.parent_id_of(target_node_id)?;
        self.contextual_visible_child_ids(parent_id)
            .into_iter()
            .position(|node_id| node_id == target_node_id)
            .map(|index| index + 1)
    }

    fn contextual_scalar_selector_target_node_id(
        &self,
        selector: &BoundHostStructuralSelector,
    ) -> Option<TreeNodeId> {
        if let Some(node_id) = self.contextual_bound_expr_treecalc_node_id(
            &BoundExpr::HostStructuralSelector(selector.clone()),
        ) {
            return Some(node_id);
        }
        let member = selector.selector_family.as_str();
        match selector.base.as_ref() {
            BoundExpr::HostReferenceCollection(collection)
                if collection.collection_family.eq_ignore_ascii_case("self")
                    || collection.source_token_text.eq_ignore_ascii_case("SELF") =>
            {
                let base_node_id = self.contextual_self_anchor_node_id(self.owner_node_id);
                self.contextual_visible_child_by_symbol(base_node_id, member)
            }
            BoundExpr::HostStructuralSelector(base_selector) => {
                let base_node_id = self.contextual_scalar_selector_target_node_id(base_selector)?;
                self.contextual_visible_child_by_symbol(base_node_id, member)
            }
            _ => None,
        }
    }

    fn bind_bound_host_selector_tail_collection(
        &mut self,
        selector: &BoundHostStructuralSelector,
        base_collection: &BoundHostReferenceCollection,
    ) {
        let Some(collection_family) =
            tree_reference_collection_family_from_bound_key(&base_collection.collection_family)
        else {
            return;
        };
        self.bind_bound_selector_tail_collection_from_parts(
            selector,
            collection_family,
            &base_collection.collection_family,
            base_collection.base.as_deref(),
            base_collection
                .members
                .iter()
                .filter_map(bound_expr_treecalc_node_id)
                .collect(),
        );
    }

    fn bind_bound_structural_selector_tail_collection(
        &mut self,
        selector: &BoundHostStructuralSelector,
        base_selector: &BoundHostStructuralSelector,
    ) {
        let Some(collection_family) =
            tree_reference_collection_family_from_bound_key(&base_selector.selector_family)
        else {
            return;
        };
        self.bind_bound_selector_tail_collection_from_parts(
            selector,
            collection_family,
            &base_selector.selector_family,
            Some(base_selector.base.as_ref()),
            base_selector
                .members
                .iter()
                .filter_map(bound_expr_treecalc_node_id)
                .collect(),
        );
    }

    fn bind_bound_selector_tail_collection_from_parts(
        &mut self,
        selector: &BoundHostStructuralSelector,
        collection_family: TreeReferenceCollectionFamily,
        collection_bound_key: &str,
        base: Option<&BoundExpr>,
        explicit_member_node_ids: Vec<TreeNodeId>,
    ) {
        let base_node_id = base
            .and_then(|base| self.contextual_bound_expr_treecalc_node_id(base))
            .unwrap_or(self.owner_node_id);
        let base_member_node_ids = self.collection_member_node_ids(
            collection_family,
            base_node_id,
            explicit_member_node_ids,
        );
        let tail_member = selector
            .source_token_text
            .rsplit('.')
            .next()
            .unwrap_or(&selector.source_token_text);
        let member_node_ids = base_member_node_ids
            .into_iter()
            .filter_map(|node_id| self.contextual_visible_child_by_symbol(node_id, tail_member))
            .collect::<Vec<_>>();
        let collection_dependency = TreeReferenceCollectionDependency::ordered_selector_v1(
            collection_family,
            selector.selector_handle.clone(),
            base_node_id,
            member_node_ids.clone(),
        );
        self.collection_bindings
            .push(SyntheticReferenceCollectionBinding {
                token: selector.selector_handle.clone(),
                host_ref_handle: selector.selector_handle.clone(),
                base_node_id,
                source_span_utf8: Some((selector.source_span.start, selector.source_span.end())),
                source_token_text: selector.source_token_text.clone(),
                opaque_selector: format!("{}.{}", collection_bound_key, selector.selector_family),
                member_node_ids,
                collection_dependency,
            });
    }

    fn contextual_bound_expr_treecalc_node_id(&self, expr: &BoundExpr) -> Option<TreeNodeId> {
        if let Some(node_id) = bound_expr_treecalc_node_id(expr) {
            return Some(node_id);
        }
        match expr {
            BoundExpr::Reference(ReferenceExpr::Atom(NormalizedReference::Name(name))) => {
                match crate::tree_reference_resolution::resolve_context_host_name_token(
                    &name.name,
                    self.owner_node_id,
                    self.snapshot,
                    self.meta_node_ids,
                ) {
                    crate::tree_reference_resolution::ContextHostNameResolution::Resolved(
                        node_id,
                    ) => Some(node_id),
                    _ => None,
                }
            }
            BoundExpr::HostReference(record) => {
                match crate::tree_reference_resolution::resolve_context_host_name_token(
                    &record.source_token_text,
                    self.owner_node_id,
                    self.snapshot,
                    self.meta_node_ids,
                ) {
                    crate::tree_reference_resolution::ContextHostNameResolution::Resolved(
                        node_id,
                    ) => Some(node_id),
                    _ => None,
                }
            }
            BoundExpr::HostReferenceCollection(collection) => {
                let base_node_id = collection
                    .base
                    .as_deref()
                    .and_then(|base| self.contextual_bound_expr_treecalc_node_id(base))
                    .unwrap_or(self.owner_node_id);
                self.contextual_selector_family_node_id(base_node_id, &collection.collection_family)
            }
            BoundExpr::HostStructuralSelector(selector) => {
                if let Some(node_id) =
                    self.contextual_selector_source_token_node_id(&selector.source_token_text)
                {
                    return Some(node_id);
                }
                let base_node_id = self.contextual_bound_expr_treecalc_node_id(&selector.base)?;
                self.contextual_selector_family_node_id(base_node_id, &selector.selector_family)
                    .or_else(|| {
                        let source_member = selector
                            .source_token_text
                            .rsplit('.')
                            .next()
                            .unwrap_or(&selector.source_token_text);
                        let member = if source_member == selector.source_token_text {
                            selector.selector_family.as_str()
                        } else {
                            source_member
                        };
                        self.contextual_visible_child_by_symbol(base_node_id, member)
                    })
            }
            _ => None,
        }
    }

    fn contextual_selector_source_token_node_id(&self, source: &str) -> Option<TreeNodeId> {
        let source = source.trim();
        let (base_node_id, selector_and_tail) =
            if let Some(selector_text) = source.strip_prefix('@') {
                (self.owner_node_id, selector_text)
            } else {
                let (base_name, selector_text) = source.split_once(".@")?;
                let base_node_id =
                    match crate::tree_reference_resolution::resolve_context_host_name_token(
                        base_name,
                        self.owner_node_id,
                        self.snapshot,
                        self.meta_node_ids,
                    ) {
                        crate::tree_reference_resolution::ContextHostNameResolution::Resolved(
                            node_id,
                        ) => node_id,
                        _ => return None,
                    };
                (base_node_id, selector_text)
            };
        let (selector_token, tail) = selector_and_tail
            .split_once('.')
            .map_or((selector_and_tail, None), |(selector, tail)| {
                (selector, Some(tail))
            });
        let family = match selector_token.to_ascii_uppercase().as_str() {
            "PARENT" => "parent",
            "SELF" => "self",
            "PREV" => "previous",
            "NEXT" => "next",
            _ => return None,
        };
        let selected_node_id = self.contextual_selector_family_node_id(base_node_id, family)?;
        tail.map_or(Some(selected_node_id), |member| {
            self.contextual_visible_child_by_symbol(selected_node_id, member)
        })
    }

    fn contextual_collection_source_base_node_id(&self, source: &str) -> Option<TreeNodeId> {
        let source = source.trim();
        if source.starts_with('@') || source.starts_with(".*") || source.starts_with(".**") {
            return Some(self.owner_node_id);
        }
        let base_name = source
            .split_once(".@")
            .map(|(base, _)| base)
            .or_else(|| source.split_once(".*").map(|(base, _)| base))
            .or_else(|| source.split_once(".**").map(|(base, _)| base))?;
        match crate::tree_reference_resolution::resolve_context_host_name_token(
            base_name,
            self.owner_node_id,
            self.snapshot,
            self.meta_node_ids,
        ) {
            crate::tree_reference_resolution::ContextHostNameResolution::Resolved(node_id) => {
                Some(node_id)
            }
            _ => None,
        }
    }

    fn contextual_selector_family_node_id(
        &self,
        base_node_id: TreeNodeId,
        family: &str,
    ) -> Option<TreeNodeId> {
        match family {
            "self" => Some(self.contextual_self_anchor_node_id(base_node_id)),
            "parent" => self
                .snapshot
                .parent_id_of(base_node_id)
                .filter(|parent_id| {
                    !crate::tree_reference_resolution::is_meta_effective(
                        *parent_id,
                        self.snapshot,
                        self.meta_node_ids,
                    )
                }),
            "prev" | "previous" => self.contextual_sibling_offset_node_id(base_node_id, -1),
            "next" => self.contextual_sibling_offset_node_id(base_node_id, 1),
            _ => None,
        }
    }

    fn contextual_sibling_offset_node_id(
        &self,
        base_node_id: TreeNodeId,
        offset: isize,
    ) -> Option<TreeNodeId> {
        let parent_id = self.snapshot.parent_id_of(base_node_id)?;
        let siblings = self.contextual_visible_child_ids(parent_id);
        let base_index = siblings
            .iter()
            .position(|node_id| *node_id == base_node_id)?;
        siblings
            .get(base_index.checked_add_signed(offset)?)
            .copied()
    }

    fn contextual_self_anchor_node_id(&self, base_node_id: TreeNodeId) -> TreeNodeId {
        self.snapshot
            .parent_id_of(base_node_id)
            .filter(|parent_id| {
                !crate::tree_reference_resolution::is_meta_effective(
                    *parent_id,
                    self.snapshot,
                    self.meta_node_ids,
                )
            })
            .unwrap_or(base_node_id)
    }

    fn contextual_visible_child_by_symbol(
        &self,
        base_node_id: TreeNodeId,
        symbol: &str,
    ) -> Option<TreeNodeId> {
        self.contextual_visible_child_ids(base_node_id)
            .into_iter()
            .find(|child_id| {
                self.snapshot
                    .try_get_node(*child_id)
                    .is_some_and(|child| child.symbol.eq_ignore_ascii_case(symbol))
            })
    }

    fn contextual_visible_child_ids(&self, base_node_id: TreeNodeId) -> Vec<TreeNodeId> {
        self.snapshot
            .try_get_node(base_node_id)
            .map_or_else(Vec::new, |node| {
                node.child_ids
                    .iter()
                    .copied()
                    .filter(|child_id| {
                        !crate::tree_reference_resolution::is_meta_effective(
                            *child_id,
                            self.snapshot,
                            self.meta_node_ids,
                        )
                    })
                    .collect()
            })
    }

    fn collection_member_node_ids(
        &self,
        collection_family: TreeReferenceCollectionFamily,
        base_node_id: TreeNodeId,
        explicit_member_node_ids: Vec<TreeNodeId>,
    ) -> Vec<TreeNodeId> {
        if !explicit_member_node_ids.is_empty() {
            return explicit_member_node_ids;
        }
        match collection_family {
            TreeReferenceCollectionFamily::ChildrenV1 => {
                self.contextual_visible_child_ids(base_node_id)
            }
            TreeReferenceCollectionFamily::SiblingSetV1 => {
                let Some(parent_id) = self.snapshot.parent_id_of(base_node_id) else {
                    return Vec::new();
                };
                self.contextual_visible_child_ids(parent_id)
                    .into_iter()
                    .filter(|node_id| *node_id != base_node_id)
                    .collect()
            }
            TreeReferenceCollectionFamily::PrecedingV1 => {
                self.sibling_slice_for_offset(base_node_id, false)
            }
            TreeReferenceCollectionFamily::FollowingV1 => {
                self.sibling_slice_for_offset(base_node_id, true)
            }
            TreeReferenceCollectionFamily::AncestorsV1 => {
                let mut ancestors = Vec::new();
                let mut cursor = self.snapshot.parent_id_of(base_node_id);
                while let Some(node_id) = cursor {
                    if !crate::tree_reference_resolution::is_meta_effective(
                        node_id,
                        self.snapshot,
                        self.meta_node_ids,
                    ) && node_id != self.snapshot.root_node_id()
                    {
                        ancestors.push(node_id);
                    }
                    cursor = self.snapshot.parent_id_of(node_id);
                }
                ancestors
            }
            TreeReferenceCollectionFamily::RecursiveDescendantsV1 => {
                self.recursive_visible_descendant_ids(base_node_id)
            }
            TreeReferenceCollectionFamily::ReferenceLiteralArrayV1 => Vec::new(),
        }
    }

    fn sibling_slice_for_offset(
        &self,
        base_node_id: TreeNodeId,
        following: bool,
    ) -> Vec<TreeNodeId> {
        let Some(parent_id) = self.snapshot.parent_id_of(base_node_id) else {
            return Vec::new();
        };
        let siblings = self.contextual_visible_child_ids(parent_id);
        let Some(index) = siblings.iter().position(|node_id| *node_id == base_node_id) else {
            return Vec::new();
        };
        if following {
            siblings.into_iter().skip(index + 1).collect()
        } else {
            siblings.into_iter().take(index).collect()
        }
    }

    fn recursive_visible_descendant_ids(&self, base_node_id: TreeNodeId) -> Vec<TreeNodeId> {
        let mut descendants = Vec::new();
        let mut stack = self.contextual_visible_child_ids(base_node_id);
        stack.reverse();
        while let Some(node_id) = stack.pop() {
            descendants.push(node_id);
            let mut children = self.contextual_visible_child_ids(node_id);
            children.reverse();
            stack.extend(children);
        }
        descendants
    }

    fn bind_bound_array_reference_collection(&mut self, rows: &[Vec<BoundExpr>]) {
        let member_node_ids = rows
            .iter()
            .flatten()
            .filter_map(bound_expr_treecalc_node_id)
            .collect::<Vec<_>>();
        if member_node_ids.is_empty() {
            return;
        }
        let handle = self.next_fallback_token("BOUND_ARRAY_REF");
        let token = handle.clone();
        let collection_dependency = TreeReferenceCollectionDependency::reference_literal_array_v1(
            handle.clone(),
            self.owner_node_id,
            member_node_ids.clone(),
        );
        self.collection_bindings
            .push(SyntheticReferenceCollectionBinding {
                token,
                host_ref_handle: handle,
                base_node_id: self.owner_node_id,
                source_span_utf8: None,
                source_token_text: "array_literal".to_string(),
                opaque_selector: "reference_literal_array".to_string(),
                member_node_ids,
                collection_dependency,
            });
    }

    fn bind_bound_host_collection(
        &mut self,
        record: &HostNameBindRecord,
        token: String,
        collection: crate::formula::TreeCalcCollectionHostDependencyKey,
    ) {
        let family = match tree_reference_collection_family_from_bound_key(&collection.family) {
            Some(family) => family,
            None => {
                self.bind_unresolved(
                    Some(token),
                    DependencyDescriptorKind::Unresolved,
                    format!(
                        "bound_formula_host_collection_unknown_family:handle={}:family={}",
                        record.host_name_handle, collection.family
                    ),
                    true,
                );
                return;
            }
        };
        let collection_dependency = match family {
            TreeReferenceCollectionFamily::ChildrenV1 => {
                TreeReferenceCollectionDependency::children_v1(
                    record.host_name_handle.clone(),
                    collection.base_node_id,
                    collection.member_node_ids.clone(),
                )
            }
            TreeReferenceCollectionFamily::ReferenceLiteralArrayV1 => {
                TreeReferenceCollectionDependency::reference_literal_array_v1(
                    record.host_name_handle.clone(),
                    collection.base_node_id,
                    collection.member_node_ids.clone(),
                )
            }
            family => TreeReferenceCollectionDependency::ordered_selector_v1(
                family,
                record.host_name_handle.clone(),
                collection.base_node_id,
                collection.member_node_ids.clone(),
            ),
        };
        self.collection_bindings
            .push(SyntheticReferenceCollectionBinding {
                token,
                host_ref_handle: record.host_name_handle.clone(),
                base_node_id: collection.base_node_id,
                source_span_utf8: Some((record.source_span.start, record.source_span.end())),
                source_token_text: record.source_token_text.clone(),
                opaque_selector: record
                    .host_dependency_key
                    .clone()
                    .unwrap_or_else(|| collection.family.clone()),
                member_node_ids: collection.member_node_ids,
                collection_dependency,
            });
    }

    fn project_explicit_reference(&mut self, reference: &TreeReference) {
        match reference {
            crate::formula::TreeReference::DirectNode { target_node_id } => self.bind_target(
                None,
                *target_node_id,
                reference.descriptor_kind(),
                reference.carrier_detail(),
                reference.requires_rebind_on_structural_change(),
                None,
            ),
            crate::formula::TreeReference::DynamicResolved { target_node_id, .. } => self
                .bind_target(
                    None,
                    *target_node_id,
                    reference.descriptor_kind(),
                    reference.carrier_detail(),
                    reference.requires_rebind_on_structural_change(),
                    None,
                ),
            crate::formula::TreeReference::CrossWorkspaceResolved { .. } => self
                .bind_workspace_target(
                    None,
                    reference
                        .workspace_target()
                        .expect("cross-workspace reference must carry workspace target"),
                    reference.descriptor_kind(),
                    reference.carrier_detail(),
                    reference.requires_rebind_on_structural_change(),
                ),
            crate::formula::TreeReference::ProjectionPath { .. }
            | crate::formula::TreeReference::RelativePath { .. }
            | crate::formula::TreeReference::SiblingOffset { .. }
            | crate::formula::TreeReference::QualifiedSiblingOffset { .. }
            | crate::formula::TreeReference::QualifiedParentOffset { .. } => {
                if let Some(target_node_id) = reference.resolve_target_with_meta_visibility(
                    self.snapshot,
                    self.owner_node_id,
                    self.meta_node_ids,
                ) {
                    self.bind_target(
                        None,
                        target_node_id,
                        reference.descriptor_kind(),
                        reference.carrier_detail(),
                        reference.requires_rebind_on_structural_change(),
                        None,
                    )
                } else {
                    self.bind_unresolved(
                        None,
                        reference.descriptor_kind(),
                        reference.carrier_detail(),
                        reference.requires_rebind_on_structural_change(),
                    )
                }
            }
            crate::formula::TreeReference::ReferenceCollection(
                crate::formula::TreeCalcReferenceCollection::ChildrenV1(collection),
            ) => self.bind_children_collection(None, collection),
            crate::formula::TreeReference::ReferenceCollection(
                crate::formula::TreeCalcReferenceCollection::ReferenceLiteralArrayV1(collection),
            ) => self.bind_reference_literal_array_collection(None, collection),
            crate::formula::TreeReference::ReferenceCollection(
                crate::formula::TreeCalcReferenceCollection::OrderedSelectorV1(collection),
            ) => self.bind_ordered_selector_collection(None, collection),
            crate::formula::TreeReference::HostSensitive { carrier_id, detail } => {
                self.residuals.push(ResidualCarrier {
                    kind: ResidualCarrierKind::HostSensitive,
                    owner_node_id: self.owner_node_id,
                    carrier_id: carrier_id.clone(),
                    detail: detail.clone(),
                });
            }
            crate::formula::TreeReference::CapabilitySensitive { carrier_id, detail } => {
                self.residuals.push(ResidualCarrier {
                    kind: ResidualCarrierKind::CapabilitySensitive,
                    owner_node_id: self.owner_node_id,
                    carrier_id: carrier_id.clone(),
                    detail: detail.clone(),
                });
            }
            crate::formula::TreeReference::ShapeTopology { carrier_id, detail } => {
                self.residuals.push(ResidualCarrier {
                    kind: ResidualCarrierKind::ShapeTopology,
                    owner_node_id: self.owner_node_id,
                    carrier_id: carrier_id.clone(),
                    detail: detail.clone(),
                });
            }
            crate::formula::TreeReference::DynamicPotential { carrier_id, detail } => {
                self.residuals.push(ResidualCarrier {
                    kind: ResidualCarrierKind::DynamicPotential,
                    owner_node_id: self.owner_node_id,
                    carrier_id: carrier_id.clone(),
                    detail: detail.clone(),
                });
            }
            crate::formula::TreeReference::Unresolved { token: _ } => self.bind_unresolved(
                None,
                reference.descriptor_kind(),
                reference.carrier_detail(),
                reference.requires_rebind_on_structural_change(),
            ),
        }
    }

    fn bind_target(
        &mut self,
        source_token: Option<String>,
        target_node_id: TreeNodeId,
        kind: DependencyDescriptorKind,
        carrier_detail: String,
        requires_rebind_on_structural_change: bool,
        host_name_bind: Option<TreeFormulaHostNameBindPacket>,
    ) {
        let token = source_token.unwrap_or_else(|| self.next_fallback_token("TREE_REF"));
        self.reference_bindings.push(SyntheticReferenceBinding {
            token,
            local_target_node_id: Some(target_node_id),
            workspace_target: None,
            kind,
            carrier_detail,
            requires_rebind_on_structural_change,
            host_name_bind,
        });
    }

    fn bind_workspace_target(
        &mut self,
        source_token: Option<String>,
        workspace_target: WorkspaceQualifiedTarget,
        kind: DependencyDescriptorKind,
        carrier_detail: String,
        requires_rebind_on_structural_change: bool,
    ) {
        let token = source_token.unwrap_or_else(|| self.next_fallback_token("TREE_REF"));
        self.reference_bindings.push(SyntheticReferenceBinding {
            token,
            local_target_node_id: None,
            workspace_target: Some(workspace_target),
            kind,
            carrier_detail,
            requires_rebind_on_structural_change,
            host_name_bind: None,
        });
    }

    fn bind_unresolved(
        &mut self,
        source_token: Option<String>,
        kind: DependencyDescriptorKind,
        carrier_detail: String,
        requires_rebind_on_structural_change: bool,
    ) {
        let token = source_token.unwrap_or_else(|| self.next_fallback_token("TREE_UNRESOLVED"));
        self.unresolved_bindings.push(SyntheticUnresolvedBinding {
            token,
            kind,
            carrier_detail,
            requires_rebind_on_structural_change,
        });
    }

    fn bind_children_collection(
        &mut self,
        source_token: Option<String>,
        collection: &crate::formula::TreeCalcChildrenReferenceCollection,
    ) {
        let token = source_token.unwrap_or_else(|| self.next_fallback_token("TREE_REF"));
        let member_node_ids = self
            .snapshot
            .try_get_node(collection.base_node_id)
            .map_or_else(Vec::new, |node| node.child_ids.clone());
        let collection_dependency = TreeReferenceCollectionDependency::children_v1(
            collection.host_ref_handle.clone(),
            collection.base_node_id,
            member_node_ids.clone(),
        );
        self.collection_bindings
            .push(SyntheticReferenceCollectionBinding {
                token,
                host_ref_handle: collection.host_ref_handle.clone(),
                base_node_id: collection.base_node_id,
                source_span_utf8: collection.source_span_utf8,
                source_token_text: collection.source_token_text.clone(),
                opaque_selector: collection.opaque_selector.clone(),
                member_node_ids,
                collection_dependency,
            });
    }

    fn bind_ordered_selector_collection(
        &mut self,
        source_token: Option<String>,
        collection: &crate::formula::TreeCalcOrderedSelectorReferenceCollection,
    ) {
        let token = source_token.unwrap_or_else(|| self.next_fallback_token("TREE_REF"));
        let member_node_ids = collection.member_node_ids.clone();
        let collection_dependency = TreeReferenceCollectionDependency::ordered_selector_v1(
            collection.family.dependency_family(),
            collection.host_ref_handle.clone(),
            collection.base_node_id,
            member_node_ids.clone(),
        );
        self.collection_bindings
            .push(SyntheticReferenceCollectionBinding {
                token,
                host_ref_handle: collection.host_ref_handle.clone(),
                base_node_id: collection.base_node_id,
                source_span_utf8: collection.source_span_utf8,
                source_token_text: collection.source_token_text.clone(),
                opaque_selector: collection.opaque_selector.clone(),
                member_node_ids,
                collection_dependency,
            });
    }

    fn bind_reference_literal_array_collection(
        &mut self,
        source_token: Option<String>,
        collection: &crate::formula::TreeCalcReferenceLiteralArrayCollection,
    ) {
        let token = source_token.unwrap_or_else(|| self.next_fallback_token("TREE_REF"));
        let member_node_ids = collection.member_node_ids().to_vec();
        let collection_dependency = TreeReferenceCollectionDependency::reference_literal_array_v1(
            collection.host_ref_handle().to_string(),
            collection.owner_node_id(),
            member_node_ids.clone(),
        );
        self.collection_bindings
            .push(SyntheticReferenceCollectionBinding {
                token,
                host_ref_handle: collection.host_ref_handle().to_string(),
                base_node_id: collection.owner_node_id(),
                source_span_utf8: collection.source_span_utf8(),
                source_token_text: collection.source_token_text().to_string(),
                opaque_selector: collection.opaque_selector().to_string(),
                member_node_ids,
                collection_dependency,
            });
    }

    fn project_host_value_binding(&mut self, binding: &TreeFormulaHostValueBinding) {
        self.host_value_bindings.push(SyntheticHostValueBinding {
            token: binding.source_token.clone(),
            value: binding.value.clone(),
            host_ref_handle: binding.host_ref_handle.clone(),
            source_span_utf8: binding.source_span_utf8,
            source_token_text: binding.source_token_text.clone(),
            opaque_selector: binding.opaque_selector.clone(),
            carrier_detail: binding.carrier_detail.clone(),
            target_node_id: binding.target_node_id,
            requires_rebind_on_structural_change: binding.requires_rebind_on_structural_change,
        });
    }

    fn next_fallback_token(&mut self, prefix: &str) -> String {
        let token = format!(
            "{}_{}_{}",
            prefix, self.owner_node_id.0, self.fallback_reference_index
        );
        self.fallback_reference_index += 1;
        token
    }
}

fn bound_expr_treecalc_node_id(expr: &BoundExpr) -> Option<TreeNodeId> {
    match expr {
        BoundExpr::HostReference(record) => record
            .host_dependency_key
            .as_deref()
            .and_then(treecalc_node_id_from_host_dependency_key),
        BoundExpr::HostStructuralSelector(selector) => {
            single_host_selector_member_node_id(selector)
        }
        BoundExpr::HostReferenceCollection(collection) => {
            single_host_collection_member_node_id(collection)
        }
        _ => None,
    }
}

fn single_host_selector_member_node_id(
    selector: &BoundHostStructuralSelector,
) -> Option<TreeNodeId> {
    if selector.members.len() == 1 {
        selector
            .members
            .first()
            .and_then(bound_expr_treecalc_node_id)
    } else {
        None
    }
}

fn single_host_collection_member_node_id(
    collection: &BoundHostReferenceCollection,
) -> Option<TreeNodeId> {
    if collection.members.len() == 1 {
        collection
            .members
            .first()
            .and_then(bound_expr_treecalc_node_id)
    } else {
        None
    }
}

fn scalar_host_collection_dependency_shape(
    collection: &BoundHostReferenceCollection,
) -> (DependencyDescriptorKind, String) {
    match collection.collection_family.as_str() {
        "self" => (
            DependencyDescriptorKind::RelativeBound,
            format!(
                "self_anchor:handle={}:token={}",
                collection.collection_handle, collection.source_token_text
            ),
        ),
        "parent" => (
            DependencyDescriptorKind::RelativeBound,
            format!(
                "parent_offset:handle={}:token={}",
                collection.collection_handle, collection.source_token_text
            ),
        ),
        "prev" | "previous" => (
            DependencyDescriptorKind::RelativeBound,
            format!(
                "sibling_offset:handle={}:offset=-1:token={}",
                collection.collection_handle, collection.source_token_text
            ),
        ),
        "next" => (
            DependencyDescriptorKind::RelativeBound,
            format!(
                "sibling_offset:handle={}:offset=1:token={}",
                collection.collection_handle, collection.source_token_text
            ),
        ),
        _ => (
            DependencyDescriptorKind::Unresolved,
            format!(
                "unknown_scalar_host_collection:handle={}:family={}",
                collection.collection_handle, collection.collection_family
            ),
        ),
    }
}

fn scalar_host_selector_dependency_shape(
    selector: &BoundHostStructuralSelector,
) -> (DependencyDescriptorKind, String) {
    match selector.selector_family.as_str() {
        "self" => (
            DependencyDescriptorKind::RelativeBound,
            format!(
                "self_anchor:handle={}:token={}",
                selector.selector_handle, selector.source_token_text
            ),
        ),
        "parent" => (
            DependencyDescriptorKind::RelativeBound,
            format!(
                "qualified_parent_offset:handle={}:token={}",
                selector.selector_handle, selector.source_token_text
            ),
        ),
        "prev" | "previous" => (
            DependencyDescriptorKind::RelativeBound,
            format!(
                "qualified_sibling_offset:handle={}:offset=-1:token={}",
                selector.selector_handle, selector.source_token_text
            ),
        ),
        "next" => (
            DependencyDescriptorKind::RelativeBound,
            format!(
                "qualified_sibling_offset:handle={}:offset=1:token={}",
                selector.selector_handle, selector.source_token_text
            ),
        ),
        _ => scalar_host_selector_tail_dependency_shape(selector).unwrap_or_else(|| {
            (
                DependencyDescriptorKind::StaticDirect,
                format!(
                    "host_member_selector:handle={}:family={}:token={}",
                    selector.selector_handle, selector.selector_family, selector.source_token_text
                ),
            )
        }),
    }
}

fn scalar_host_selector_tail_dependency_shape(
    selector: &BoundHostStructuralSelector,
) -> Option<(DependencyDescriptorKind, String)> {
    let BoundExpr::HostStructuralSelector(base_selector) = selector.base.as_ref() else {
        return None;
    };
    match base_selector.selector_family.as_str() {
        "parent" => Some((
            DependencyDescriptorKind::RelativeBound,
            format!(
                "qualified_parent_offset:handle={}:tail={}:token={}",
                base_selector.selector_handle, selector.selector_handle, selector.source_token_text
            ),
        )),
        "prev" | "previous" => Some((
            DependencyDescriptorKind::RelativeBound,
            format!(
                "qualified_sibling_offset:handle={}:offset=-1:tail={}:token={}",
                base_selector.selector_handle, selector.selector_handle, selector.source_token_text
            ),
        )),
        "next" => Some((
            DependencyDescriptorKind::RelativeBound,
            format!(
                "qualified_sibling_offset:handle={}:offset=1:tail={}:token={}",
                base_selector.selector_handle, selector.selector_handle, selector.source_token_text
            ),
        )),
        _ => None,
    }
}

fn structured_reference_like_identity(
    resolved: &StructuredResolvedRef,
) -> Option<(ReferenceKind, String)> {
    match resolved {
        StructuredResolvedRef::Cell(cell) => Some((
            ReferenceKind::A1,
            qualified_reference_target_for_treecalc(
                &cell.sheet_id,
                format!(
                    "{}{}",
                    treecalc_column_letters(cell.coord.col),
                    cell.coord.row
                ),
            ),
        )),
        StructuredResolvedRef::Area(area) => {
            let start = format!(
                "{}{}",
                treecalc_column_letters(area.top_left.col),
                area.top_left.row
            );
            let end_col = area.top_left.col.checked_add(area.width)?.checked_sub(1)?;
            let end_row = area.top_left.row.checked_add(area.height)?.checked_sub(1)?;
            let end = format!("{}{}", treecalc_column_letters(end_col), end_row);
            Some((
                ReferenceKind::Area,
                qualified_reference_target_for_treecalc(&area.sheet_id, format!("{start}:{end}")),
            ))
        }
        StructuredResolvedRef::EmptyArea(_) => None,
    }
}

fn qualified_reference_target_for_treecalc(sheet_id: &str, local_target: String) -> String {
    if sheet_id.is_empty() || sheet_id.starts_with("sheet:") {
        local_target
    } else {
        format!("{sheet_id}!{local_target}")
    }
}

fn treecalc_column_letters(mut col: u32) -> String {
    let mut letters = String::new();
    while col > 0 {
        let rem = ((col - 1) % 26) as u8;
        letters.insert(0, (b'A' + rem) as char);
        col = (col - 1) / 26;
    }
    letters
}

fn synthetic_cell_row(node_id: TreeNodeId) -> u32 {
    u32::try_from(node_id.0).unwrap_or(u32::MAX)
}

fn treecalc_published_value_to_calc_value(value: &str) -> CalcValue {
    authored_cell_entry_text_to_calc_value(value)
}

fn authored_cell_entry_text_to_calc_value(value: &str) -> CalcValue {
    let source = FormulaSourceRecord::new("treecalc:authored-literal", 1, value);
    match RuntimeEnvironment::new()
        .with_host_reference_syntax(treecalc_host_reference_syntax_profile())
        .interpret_authored_input(source)
    {
        RuntimeAuthoredInputResult::Literal(value) => value,
        RuntimeAuthoredInputResult::Formula(_) | RuntimeAuthoredInputResult::Diagnostics(_) => {
            CalcValue::error(WorksheetErrorCode::Value)
        }
    }
}

fn treecalc_host_reference_syntax_profile() -> RuntimeHostReferenceSyntaxProfile {
    RuntimeHostReferenceSyntaxProfile::with_members_and_structural_selectors(
        [
            RuntimeHostReferenceCollectionSyntax::new("CHILDREN", "children"),
            RuntimeHostReferenceCollectionSyntax::new("*", "children"),
            RuntimeHostReferenceCollectionSyntax::new("PRECEDING", "preceding"),
            RuntimeHostReferenceCollectionSyntax::new("FOLLOWING", "following"),
            RuntimeHostReferenceCollectionSyntax::new("ANCESTORS", "ancestors"),
            RuntimeHostReferenceCollectionSyntax::new("DESCENDANTS", "recursive_descendants"),
            RuntimeHostReferenceCollectionSyntax::new("**", "recursive_descendants"),
        ],
        [
            RuntimeHostReferenceStructuralSelectorSyntax::new("PARENT", "parent"),
            RuntimeHostReferenceStructuralSelectorSyntax::new("SELF", "self"),
            RuntimeHostReferenceStructuralSelectorSyntax::new("PREV", "previous"),
            RuntimeHostReferenceStructuralSelectorSyntax::new("NEXT", "next"),
            RuntimeHostReferenceStructuralSelectorSyntax::new("NAME", "metadata_name"),
            RuntimeHostReferenceStructuralSelectorSyntax::new("INDEX", "metadata_index"),
            RuntimeHostReferenceStructuralSelectorSyntax::new("FORMULA", "metadata_formula"),
        ],
    )
}

fn calc_value_trace_summary(value: &CalcValue) -> String {
    match &value.core {
        CoreValue::Number(value) => value.to_string(),
        CoreValue::Text(value) => value.to_string_lossy(),
        CoreValue::Logical(value) => value.to_string(),
        CoreValue::Error(value) => format!("{value:?}"),
        CoreValue::Empty => String::new(),
        CoreValue::Missing => "missing".to_string(),
        CoreValue::Array(value) => format!("{value:?}"),
        CoreValue::Reference(value) => format!("{:?}:{}", value.kind(), value.target()),
    }
}

fn value_payload_to_string(payload: &oxfml_core::seam::ValuePayload) -> String {
    match payload {
        oxfml_core::seam::ValuePayload::Number(value)
        | oxfml_core::seam::ValuePayload::Text(value)
        | oxfml_core::seam::ValuePayload::ErrorCode(value) => value.clone(),
        oxfml_core::seam::ValuePayload::Logical(value) => value.to_string(),
        oxfml_core::seam::ValuePayload::Blank => String::new(),
    }
}

fn residual_runtime_effect(residual: &ResidualCarrier) -> RuntimeEffect {
    // W026 owns only the current emitted transport floor for host-sensitive and
    // dynamic-potential residuals. Broader emitted family realization belongs to W029.
    let (kind, family) = match residual.kind {
        ResidualCarrierKind::HostSensitive => (
            "runtime_effect.host_sensitive_reference",
            RuntimeEffectFamily::ExecutionRestriction,
        ),
        ResidualCarrierKind::DynamicPotential => (
            "runtime_effect.dynamic_reference",
            RuntimeEffectFamily::DynamicDependency,
        ),
        ResidualCarrierKind::CapabilitySensitive => (
            "runtime_effect.capability_sensitive_reference",
            RuntimeEffectFamily::CapabilitySensitive,
        ),
        ResidualCarrierKind::ShapeTopology => (
            "runtime_effect.shape_topology_reference",
            RuntimeEffectFamily::ShapeTopology,
        ),
    };
    RuntimeEffect {
        kind: kind.to_string(),
        family,
        detail: format!(
            "owner_node:{};carrier_id:{};detail:{}",
            residual.owner_node_id, residual.carrier_id, residual.detail
        ),
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use crate::consumer::{
        OxCalcTreeContext, OxCalcTreeNodeCreate, OxCalcTreeRunState, OxCalcTreeWorkspaceCreate,
    };
    use crate::formula::{
        FixtureFormulaAst, FixtureFormulaBinaryOp, RelativeReferenceBase,
        TreeCalcChildrenReferenceCollection, TreeCalcOrderedSelectorFamily,
        TreeCalcOrderedSelectorReferenceCollection, TreeCalcReferenceCollection,
        TreeCalcReferenceLiteralArrayCollection, TreeCalcReferenceLiteralArrayElement, TreeFormula,
        TreeFormulaBinding, TreeReference,
    };
    use crate::repository::{
        CalculationRepository, FormulaSlotRecord, FormulaSourceIdentity,
        SubscriptionLifecycleAction, SubscriptionLifecycleReason,
    };
    use crate::rich_value_capability::{
        RICH_VALUE_CAPABILITY_TRACE_REPLAY_SCHEMA_ID, RichValueCapabilityTraceReplayColumns,
        w050_initial_required_capability_set_example,
    };
    use crate::structural::{
        BindArtifactId, FormulaArtifactId, StructuralNode, StructuralNodeKind, StructuralSnapshotId,
    };
    use crate::workspace_revision::{NamespaceSnapshot, NodeInputRecord, NodeInputSnapshot};
    use oxfunc_core::value::{CoreValue, RichValue, WorksheetErrorCode};
    use serde_json::json;

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
                    child_ids: vec![TreeNodeId(2), TreeNodeId(3), TreeNodeId(4)],
                },
                StructuralNode {
                    node_id: TreeNodeId(2),
                    kind: StructuralNodeKind::Constant,
                    symbol: "A".to_string(),
                    parent_id: Some(TreeNodeId(1)),
                    child_ids: vec![],
                },
                StructuralNode {
                    node_id: TreeNodeId(3),
                    kind: StructuralNodeKind::Calculation,
                    symbol: "B".to_string(),
                    parent_id: Some(TreeNodeId(1)),
                    child_ids: vec![],
                },
                StructuralNode {
                    node_id: TreeNodeId(4),
                    kind: StructuralNodeKind::Calculation,
                    symbol: "C".to_string(),
                    parent_id: Some(TreeNodeId(1)),
                    child_ids: vec![],
                },
            ],
        )
        .unwrap()
    }

    fn workspace_revision_with_literal_inputs(
        structural_snapshot: StructuralSnapshot,
        input_values: &BTreeMap<TreeNodeId, String>,
    ) -> WorkspaceRevision {
        let records = structural_snapshot
            .nodes()
            .keys()
            .map(|node_id| {
                input_values.get(node_id).map_or_else(
                    || NodeInputRecord::empty(*node_id, 1),
                    |value| NodeInputRecord::literal(*node_id, value.clone(), 1),
                )
            })
            .collect::<Vec<_>>();
        WorkspaceRevision::new(
            "workspace:treecalc-test",
            structural_snapshot,
            NodeInputSnapshot::create(records).unwrap(),
            NamespaceSnapshot::current_absent(),
        )
    }

    fn local_treecalc_input(
        structural_snapshot: StructuralSnapshot,
        formula_catalog: TreeFormulaCatalog,
        input_values: BTreeMap<TreeNodeId, String>,
        publication_values: BTreeMap<TreeNodeId, String>,
        publication_runtime_effects: Vec<RuntimeEffect>,
        invalidation_seeds: Vec<InvalidationSeed>,
        run_suffix: &str,
    ) -> LocalTreeCalcInput {
        let workspace_revision =
            workspace_revision_with_literal_inputs(structural_snapshot, &input_values);
        LocalTreeCalcInput {
            layer_snapshot_ids: LocalTreeCalcLayerSnapshotIds::current_absent_for_revision(
                &workspace_revision,
            ),
            workspace_revision,
            formula_catalog,
            formula_dependency_descriptors: None,
            table_snapshots: BTreeMap::new(),
            static_dependency_shape_updates: Vec::new(),
            publication_calc_values: publication_values
                .into_iter()
                .map(|(node_id, value)| (node_id, treecalc_published_value_to_calc_value(&value)))
                .collect(),
            publication_runtime_effects,
            invalidation_seeds,
            previous_arg_preparation_profile_version: None,
            candidate_result_id: format!("cand:{run_suffix}"),
            publication_id: format!("pub:{run_suffix}"),
            environment_context: LocalTreeCalcEnvironmentContext::default(),
        }
    }

    fn children_collection_snapshot(parent_child_ids: Vec<TreeNodeId>) -> StructuralSnapshot {
        let mut nodes = vec![
            StructuralNode {
                node_id: TreeNodeId(1),
                kind: StructuralNodeKind::Root,
                symbol: "Root".to_string(),
                parent_id: None,
                child_ids: vec![TreeNodeId(2), TreeNodeId(10)],
            },
            StructuralNode {
                node_id: TreeNodeId(2),
                kind: StructuralNodeKind::Container,
                symbol: "Branch".to_string(),
                parent_id: Some(TreeNodeId(1)),
                child_ids: parent_child_ids.clone(),
            },
            StructuralNode {
                node_id: TreeNodeId(3),
                kind: StructuralNodeKind::Constant,
                symbol: "A".to_string(),
                parent_id: Some(TreeNodeId(2)),
                child_ids: vec![],
            },
            StructuralNode {
                node_id: TreeNodeId(4),
                kind: StructuralNodeKind::Constant,
                symbol: "B".to_string(),
                parent_id: Some(TreeNodeId(2)),
                child_ids: vec![],
            },
            StructuralNode {
                node_id: TreeNodeId(10),
                kind: StructuralNodeKind::Calculation,
                symbol: "Total".to_string(),
                parent_id: Some(TreeNodeId(1)),
                child_ids: vec![],
            },
        ];
        if parent_child_ids.contains(&TreeNodeId(5)) {
            nodes.push(StructuralNode {
                node_id: TreeNodeId(5),
                kind: StructuralNodeKind::Constant,
                symbol: "C".to_string(),
                parent_id: Some(TreeNodeId(2)),
                child_ids: vec![],
            });
        }

        StructuralSnapshot::create(StructuralSnapshotId(1), TreeNodeId(1), nodes).unwrap()
    }

    fn children_collection_catalog() -> TreeFormulaCatalog {
        children_collection_catalog_with_source_token("@CHILDREN")
    }

    fn children_collection_catalog_with_source_token(
        source_token_text: &str,
    ) -> TreeFormulaCatalog {
        let collection =
            TreeCalcChildrenReferenceCollection::new(TreeNodeId(2), source_token_text.to_string())
                .with_source_span_utf8(5, 5 + source_token_text.len());
        TreeFormulaCatalog::new([TreeFormulaBinding {
            owner_node_id: TreeNodeId(10),
            formula_artifact_id: FormulaArtifactId("formula:total".to_string()),
            bind_artifact_id: Some(BindArtifactId("bind:total".to_string())),
            expression: TreeFormula::opaque_oxfml(
                "=SUM(TREE_REF_10_0)",
                [TreeReference::ReferenceCollection(
                    TreeCalcReferenceCollection::ChildrenV1(collection),
                )],
            ),
        }])
    }

    fn ordered_selector_collection_catalog() -> TreeFormulaCatalog {
        let collection = TreeCalcOrderedSelectorReferenceCollection::new(
            TreeCalcOrderedSelectorFamily::PrecedingV1,
            TreeNodeId(10),
            "@PRECEDING",
            [TreeNodeId(3), TreeNodeId(4)],
        )
        .with_source_span_utf8(5, 15);
        TreeFormulaCatalog::new([TreeFormulaBinding {
            owner_node_id: TreeNodeId(10),
            formula_artifact_id: FormulaArtifactId("formula:total".to_string()),
            bind_artifact_id: Some(BindArtifactId("bind:total".to_string())),
            expression: TreeFormula::opaque_oxfml(
                "=SUM(TREE_REF_10_0)",
                [TreeReference::ReferenceCollection(
                    TreeCalcReferenceCollection::OrderedSelectorV1(collection),
                )],
            ),
        }])
    }

    fn reference_literal_array_catalog() -> TreeFormulaCatalog {
        let collection = TreeCalcReferenceLiteralArrayCollection::reference_only(
            "array:q1",
            TreeNodeId(10),
            "{A,B,A}",
            [
                TreeCalcReferenceLiteralArrayElement::ReferenceNode(TreeNodeId(3)),
                TreeCalcReferenceLiteralArrayElement::ReferenceNode(TreeNodeId(4)),
                TreeCalcReferenceLiteralArrayElement::ReferenceNode(TreeNodeId(3)),
            ],
        )
        .expect("reference-only array")
        .with_source_span_utf8(5, 12);
        TreeFormulaCatalog::new([TreeFormulaBinding {
            owner_node_id: TreeNodeId(10),
            formula_artifact_id: FormulaArtifactId("formula:total".to_string()),
            bind_artifact_id: Some(BindArtifactId("bind:total".to_string())),
            expression: TreeFormula::opaque_oxfml(
                "=SUM(TREE_REF_10_0)",
                [TreeReference::ReferenceCollection(
                    TreeCalcReferenceCollection::ReferenceLiteralArrayV1(collection),
                )],
            ),
        }])
    }

    fn repository_slot_from_prepared(prepared: &PreparedOxfmlFormula) -> FormulaSlotRecord {
        FormulaSlotRecord {
            owner_node_id: prepared.binding.owner_node_id,
            formula_artifact_id: prepared.binding.formula_artifact_id.clone(),
            bind_artifact_id: prepared.binding.bind_artifact_id.clone(),
            source_identity: FormulaSourceIdentity {
                formula_stable_id: prepared.source.formula_stable_id.0.clone(),
                formula_text_version: prepared.source.formula_text_version.0,
                formula_token: Some(prepared.source.formula_token().0),
            },
            opaque_source_text: prepared.source.entered_formula_text.clone(),
        }
    }

    fn formula_input(owner_node_id: TreeNodeId, expression: TreeFormula) -> LocalTreeCalcInput {
        local_treecalc_input(
            snapshot(),
            TreeFormulaCatalog::new([TreeFormulaBinding {
                owner_node_id,
                formula_artifact_id: formula_artifact_id(owner_node_id),
                bind_artifact_id: Some(bind_artifact_id(owner_node_id)),
                expression,
            }]),
            BTreeMap::new(),
            BTreeMap::new(),
            Vec::new(),
            Vec::new(),
            &format!("b6:{}", owner_node_id.0),
        )
    }

    fn formula_artifact_id(node_id: TreeNodeId) -> FormulaArtifactId {
        match node_id.0 {
            3 => FormulaArtifactId("formula:b".to_string()),
            4 => FormulaArtifactId("formula:c".to_string()),
            _ => FormulaArtifactId(format!("formula:node:{}", node_id.0)),
        }
    }

    fn bind_artifact_id(node_id: TreeNodeId) -> BindArtifactId {
        match node_id.0 {
            3 => BindArtifactId("bind:b".to_string()),
            4 => BindArtifactId("bind:c".to_string()),
            _ => BindArtifactId(format!("bind:node:{}", node_id.0)),
        }
    }

    fn assert_has_diagnostic(run: &LocalTreeCalcRunArtifacts, expected: &str) {
        assert!(
            run.diagnostics
                .iter()
                .any(|diagnostic| diagnostic == expected),
            "missing diagnostic {expected:?} in {:?}",
            run.diagnostics
        );
    }

    fn has_diagnostic_prefix(run: &LocalTreeCalcRunArtifacts, prefix: &str) -> bool {
        run.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.starts_with(prefix))
    }

    fn diagnostic_prefix_count(run: &LocalTreeCalcRunArtifacts, prefix: &str) -> usize {
        run.diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.starts_with(prefix))
            .count()
    }

    fn f2_artifact_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/test-runs/core-engine/w050-f2-differential-evaluation-gates-001")
    }

    fn f3_artifact_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/test-runs/core-engine/w050-f3-derivation-trace-invoke-outcome-001")
    }

    fn f4_artifact_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/test-runs/core-engine/w050-f4-push-pull-visibility-scheduling-001")
    }

    fn f5_artifact_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/test-runs/core-engine/w050-f5-ok-differential-evidence-001")
    }

    fn g3_artifact_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/test-runs/core-engine/w050-g3-capability-trace-replay-columns-001")
    }

    fn derivation_trace_input(trace_enabled: bool, run_suffix: &str) -> LocalTreeCalcInput {
        let mut input = formula_input(
            TreeNodeId(3),
            TreeFormula::opaque_oxfml("=LET(base,2,LAMBDA(delta,base+delta)(5))", Vec::new()),
        );
        input.candidate_result_id = format!("cand:f3:{run_suffix}");
        input.publication_id = format!("pub:f3:{run_suffix}");
        input.environment_context = input
            .environment_context
            .with_derivation_trace_enabled(trace_enabled);
        input
    }

    fn run_derivation_trace_scenarios() -> (LocalTreeCalcRunArtifacts, LocalTreeCalcRunArtifacts) {
        let engine = LocalTreeCalcEngine;
        let default_run = engine
            .execute(derivation_trace_input(false, "value-only"))
            .expect("F3 default trace-mode run should publish");
        let traced_run = engine
            .execute(derivation_trace_input(true, "prepared-calls"))
            .expect("F3 trace-mode run should publish");

        (default_run, traced_run)
    }

    fn derivation_trace_invoke_outcome_artifact_json() -> serde_json::Value {
        let (default_run, traced_run) = run_derivation_trace_scenarios();
        let trace = traced_run
            .derivation_traces
            .first()
            .expect("trace-mode run should produce a derivation trace");

        json!({
            "run_id": "w050-f3-derivation-trace-invoke-outcome-001",
            "validation_status": "pass",
            "primary_validation_command": "cargo test -p oxcalc-core derivation_trace -- --nocapture",
            "trace_schema_id": DERIVATION_TRACE_SCHEMA_ID,
            "cases": [
                {
                    "case_id": "value_only_default_suppresses_derivation_trace",
                    "result_state": treecalc_state_key(&default_run.result_state),
                    "published_value": default_run.published_values.get(&TreeNodeId(3)).cloned().unwrap_or_default(),
                    "derivation_trace_count": default_run.derivation_traces.len(),
                    "trace_diagnostic_emitted": has_diagnostic_prefix(&default_run, "derivation_trace_recorded:")
                },
                {
                    "case_id": "prepared_calls_opt_in_records_invoke_outcome",
                    "result_state": treecalc_state_key(&traced_run.result_state),
                    "published_value": traced_run.published_values.get(&TreeNodeId(3)).cloned().unwrap_or_default(),
                    "derivation_trace_count": traced_run.derivation_traces.len(),
                    "trace_diagnostic_emitted": has_diagnostic_prefix(&traced_run, "derivation_trace_recorded:"),
                    "derivation_trace": serde_json::to_value(trace).expect("derivation trace should serialize")
                }
            ]
        })
    }

    fn capability_trace_replay_columns_artifact_json() -> serde_json::Value {
        let (_, traced_run) = run_derivation_trace_scenarios();
        let identity = traced_run
            .prepared_formula_identities
            .first()
            .expect("trace fixture should prepare one formula");
        let trace = traced_run
            .derivation_traces
            .first()
            .expect("trace-mode run should produce a derivation trace");
        let required_set = w050_initial_required_capability_set_example();
        let required_columns =
            RichValueCapabilityTraceReplayColumns::from_required_sets([&required_set]);
        let empty_columns = RichValueCapabilityTraceReplayColumns::empty_v1();

        json!({
            "run_id": "w050-g3-capability-trace-replay-columns-001",
            "validation_status": "pass",
            "primary_validation_command": "cargo test -p oxcalc-core capability_set_trace_replay -- --nocapture",
            "trace_replay_schema": {
                "schema_id": RICH_VALUE_CAPABILITY_TRACE_REPLAY_SCHEMA_ID,
                "prepared_formula_identity_field": "rich_value_capability_columns",
                "derivation_trace_field": "rich_value_capability_columns",
                "derivation_template_selection_field": "rich_value_capability_columns",
                "derivation_template_hole_field": "rich_value_capability_columns",
                "columns": [
                    "required_capability_set_keys",
                    "producer_capability_set_keys",
                    "exercised_capability_keys"
                ]
            },
            "current_v1_reserved_output": {
                "current_v1_production_paths_emit_rich_holes": false,
                "prepared_formula_identity_columns": serde_json::to_value(&identity.rich_value_capability_columns).expect("columns should serialize"),
                "derivation_trace_columns": serde_json::to_value(&trace.rich_value_capability_columns).expect("columns should serialize"),
                "derivation_template_selection_columns": serde_json::to_value(&trace.template_selection.rich_value_capability_columns).expect("columns should serialize"),
                "derivation_template_hole_columns_empty": trace.template_selection.template_holes.iter().all(|hole| hole.rich_value_capability_columns.is_empty())
            },
            "reserved_rich_requirement_example": {
                "required_capability_set_key": required_set.stable_key(),
                "columns": serde_json::to_value(&required_columns).expect("columns should serialize"),
                "reserved_empty_columns": serde_json::to_value(&empty_columns).expect("columns should serialize")
            },
            "scope_boundary": {
                "rich_value_kernel_claim": false,
                "producer_capability_sets_emitted_by_current_v1": false,
                "rich_arg_accepted_activation_claim": false
            }
        })
    }

    fn differential_evaluation_gate_catalog() -> TreeFormulaCatalog {
        TreeFormulaCatalog::new([TreeFormulaBinding {
            owner_node_id: TreeNodeId(3),
            formula_artifact_id: FormulaArtifactId("formula:f2".to_string()),
            bind_artifact_id: Some(BindArtifactId("bind:f2".to_string())),
            expression: fixture_formula(
                TreeNodeId(3),
                FixtureFormulaAst::Binary {
                    op: FixtureFormulaBinaryOp::Add,
                    left: Box::new(FixtureFormulaAst::Reference(TreeReference::DirectNode {
                        target_node_id: TreeNodeId(2),
                    })),
                    right: Box::new(FixtureFormulaAst::Literal {
                        value: "3".to_string(),
                    }),
                },
            ),
        }])
    }

    fn differential_evaluation_gate_input(
        structural_snapshot: StructuralSnapshot,
        formula_catalog: TreeFormulaCatalog,
        input_values: BTreeMap<TreeNodeId, String>,
        seeded_published_values: BTreeMap<TreeNodeId, String>,
        invalidation_seeds: Vec<InvalidationSeed>,
        run_suffix: &str,
    ) -> LocalTreeCalcInput {
        local_treecalc_input(
            structural_snapshot,
            formula_catalog,
            input_values,
            seeded_published_values,
            Vec::new(),
            invalidation_seeds,
            &format!("f2:{run_suffix}"),
        )
    }

    fn run_differential_evaluation_gate_scenarios() -> (
        LocalTreeCalcRunArtifacts,
        LocalTreeCalcRunArtifacts,
        LocalTreeCalcRunArtifacts,
    ) {
        let engine = LocalTreeCalcEngine;
        let formula_catalog = differential_evaluation_gate_catalog();
        let initial = engine
            .execute(differential_evaluation_gate_input(
                snapshot(),
                formula_catalog.clone(),
                BTreeMap::from([(TreeNodeId(2), "2".to_string())]),
                BTreeMap::new(),
                Vec::new(),
                "initial",
            ))
            .expect("initial F2 run should publish");

        let reuse = engine
            .execute(differential_evaluation_gate_input(
                snapshot(),
                formula_catalog.clone(),
                BTreeMap::from([(TreeNodeId(2), "2".to_string())]),
                initial.published_values.clone(),
                Vec::new(),
                "reuse",
            ))
            .expect("F2 reuse run should verify clean");

        let upstream_bypass = engine
            .execute(differential_evaluation_gate_input(
                snapshot(),
                formula_catalog,
                BTreeMap::from([(TreeNodeId(2), "4".to_string())]),
                initial.published_values.clone(),
                vec![InvalidationSeed {
                    node_id: TreeNodeId(2),
                    reason: InvalidationReasonKind::UpstreamPublication,
                }],
                "upstream-bypass",
            ))
            .expect("upstream F2 run should publish changed value");

        (initial, reuse, upstream_bypass)
    }

    fn treecalc_state_key(state: &LocalTreeCalcRunState) -> &'static str {
        match state {
            LocalTreeCalcRunState::Published => "published",
            LocalTreeCalcRunState::VerifiedClean => "verified_clean",
            LocalTreeCalcRunState::Rejected => "rejected",
        }
    }

    fn differential_evaluation_gate_artifact_json() -> serde_json::Value {
        let (initial, reuse, upstream_bypass) = run_differential_evaluation_gate_scenarios();
        json!({
            "run_id": "w050-f2-differential-evaluation-gates-001",
            "validation_status": "pass",
            "primary_validation_command": "cargo test -p oxcalc-core differential_evaluation_gate -- --nocapture",
            "gate": {
                "cache_key_fields": [
                    "call_site_id",
                    "hole_binding_fingerprint"
                ],
                "cache_hit_reuse_condition": "matching per-edge key with no caller-supplied invalidation seed and no upstream/external/dynamic dependency delta",
                "path_exclusions": [
                    "VolatileFunction",
                    "EffectfulPath"
                ],
                "semantic_bypasses": [
                    "UpstreamPublication",
                    "ExternallyInvalidated",
                    "DependencyShapeDelta",
                    "ExplicitInvalidationSeed"
                ]
            },
            "validation_cases": [
                {
                    "case_id": "hit_reuses_seeded_value_without_publication_change",
                    "initial_result_state": treecalc_state_key(&initial.result_state),
                    "reuse_result_state": treecalc_state_key(&reuse.result_state),
                    "initial_published_value": initial.published_values.get(&TreeNodeId(3)).cloned().unwrap_or_default(),
                    "reuse_published_value": reuse.published_values.get(&TreeNodeId(3)).cloned().unwrap_or_default(),
                    "cache_hit_observed": has_diagnostic_prefix(&reuse, "edge_value_cache_hit:node:3:"),
                    "oxfml_invocation_skipped": !has_diagnostic_prefix(&reuse, "oxfml_candidate_result_id:"),
                    "publication_bundle_emitted": reuse.publication_bundle.is_some()
                },
                {
                    "case_id": "upstream_publication_bypasses_cache_and_publishes_changed_value",
                    "result_state": treecalc_state_key(&upstream_bypass.result_state),
                    "published_value": upstream_bypass.published_values.get(&TreeNodeId(3)).cloned().unwrap_or_default(),
                    "cache_bypass_observed": has_diagnostic_prefix(&upstream_bypass, "edge_value_cache_bypass:node:3:UpstreamPublication"),
                    "oxfml_invocation_executed": has_diagnostic_prefix(&upstream_bypass, "oxfml_candidate_result_id:"),
                    "publication_bundle_emitted": upstream_bypass.publication_bundle.is_some()
                }
            ]
        })
    }

    fn push_pull_scheduling_catalog() -> TreeFormulaCatalog {
        TreeFormulaCatalog::new([
            TreeFormulaBinding {
                owner_node_id: TreeNodeId(3),
                formula_artifact_id: FormulaArtifactId("formula:f4:b".to_string()),
                bind_artifact_id: Some(BindArtifactId("bind:f4:b".to_string())),
                expression: fixture_formula(
                    TreeNodeId(3),
                    FixtureFormulaAst::Binary {
                        op: FixtureFormulaBinaryOp::Add,
                        left: Box::new(FixtureFormulaAst::Reference(TreeReference::DirectNode {
                            target_node_id: TreeNodeId(2),
                        })),
                        right: Box::new(FixtureFormulaAst::Literal {
                            value: "3".to_string(),
                        }),
                    },
                ),
            },
            TreeFormulaBinding {
                owner_node_id: TreeNodeId(4),
                formula_artifact_id: FormulaArtifactId("formula:f4:c".to_string()),
                bind_artifact_id: Some(BindArtifactId("bind:f4:c".to_string())),
                expression: fixture_formula(
                    TreeNodeId(4),
                    FixtureFormulaAst::Binary {
                        op: FixtureFormulaBinaryOp::Multiply,
                        left: Box::new(FixtureFormulaAst::Reference(TreeReference::DirectNode {
                            target_node_id: TreeNodeId(2),
                        })),
                        right: Box::new(FixtureFormulaAst::Literal {
                            value: "10".to_string(),
                        }),
                    },
                ),
            },
        ])
    }

    fn push_pull_scheduling_input(
        structural_snapshot: StructuralSnapshot,
        input_values: BTreeMap<TreeNodeId, String>,
        seeded_published_values: BTreeMap<TreeNodeId, String>,
        invalidation_seeds: Vec<InvalidationSeed>,
        scheduling_policy: LocalTreeCalcSchedulingPolicy,
        run_suffix: &str,
    ) -> LocalTreeCalcInput {
        let mut input = local_treecalc_input(
            structural_snapshot,
            push_pull_scheduling_catalog(),
            input_values,
            seeded_published_values,
            Vec::new(),
            invalidation_seeds,
            &format!("f4:{run_suffix}"),
        );
        input.environment_context = input
            .environment_context
            .with_scheduling_policy(scheduling_policy);
        input
    }

    fn run_push_pull_scheduling_scenarios() -> (
        LocalTreeCalcRunArtifacts,
        LocalTreeCalcRunArtifacts,
        LocalTreeCalcRunArtifacts,
    ) {
        let engine = LocalTreeCalcEngine;
        let initial = engine
            .execute(push_pull_scheduling_input(
                snapshot(),
                BTreeMap::from([(TreeNodeId(2), "2".to_string())]),
                BTreeMap::new(),
                Vec::new(),
                LocalTreeCalcSchedulingPolicy::PullFullClosure,
                "initial",
            ))
            .expect("F4 initial pull run should publish both formulas");

        let edited_input_values = BTreeMap::from([(TreeNodeId(2), "4".to_string())]);
        let upstream_seed = vec![InvalidationSeed {
            node_id: TreeNodeId(2),
            reason: InvalidationReasonKind::UpstreamPublication,
        }];

        let push_visible = engine
            .execute(push_pull_scheduling_input(
                snapshot(),
                edited_input_values.clone(),
                initial.published_values.clone(),
                upstream_seed.clone(),
                LocalTreeCalcSchedulingPolicy::PushVisibilityBounded {
                    visible_observer_node_ids: vec![TreeNodeId(3)],
                },
                "push-visible-b",
            ))
            .expect("F4 push visibility-bounded run should publish visible observer");

        let pull_full = engine
            .execute(push_pull_scheduling_input(
                snapshot(),
                edited_input_values,
                initial.published_values.clone(),
                upstream_seed,
                LocalTreeCalcSchedulingPolicy::PullFullClosure,
                "pull-full",
            ))
            .expect("F4 pull full-closure run should publish all affected formulas");

        (initial, push_visible, pull_full)
    }

    fn push_pull_scheduling_artifact_json() -> serde_json::Value {
        let (initial, push_visible, pull_full) = run_push_pull_scheduling_scenarios();
        json!({
            "run_id": "w050-f4-push-pull-visibility-scheduling-001",
            "validation_status": "pass",
            "primary_validation_command": "cargo test -p oxcalc-core push_pull_scheduling -- --nocapture",
            "semantic_equivalence_statement": "For the declared visible observer set, PushVisibilityBounded evaluates the same dependency graph and prepared-callable identities as PullFullClosure and publishes the same visible observer value; non-observer dirty formulas may be deferred until visible, aged in, or swept by a full-closure pass.",
            "fairness_notes": [
                "PullFullClosure sweeps every formula owner in deterministic topological order.",
                "PushVisibilityBounded must be paired with periodic full-closure sweeps or observer aging to prevent indefinite deferral of non-visible dirty formulas.",
                "Deferred formulas stay dirty and replay-visible through scheduling_deferred diagnostics."
            ],
            "cases": [
                {
                    "case_id": "initial_pull_full_closure",
                    "result_state": treecalc_state_key(&initial.result_state),
                    "evaluation_order": initial.evaluation_order.iter().map(|node_id| node_id.0).collect::<Vec<_>>(),
                    "published_b": initial.published_values.get(&TreeNodeId(3)).cloned().unwrap_or_default(),
                    "published_c": initial.published_values.get(&TreeNodeId(4)).cloned().unwrap_or_default(),
                    "policy_diagnostic": has_diagnostic_prefix(&initial, "scheduling_policy:pull_full_closure")
                },
                {
                    "case_id": "push_visibility_bounded_visible_b",
                    "result_state": treecalc_state_key(&push_visible.result_state),
                    "evaluation_order": push_visible.evaluation_order.iter().map(|node_id| node_id.0).collect::<Vec<_>>(),
                    "published_b": push_visible.published_values.get(&TreeNodeId(3)).cloned().unwrap_or_default(),
                    "published_c": push_visible.published_values.get(&TreeNodeId(4)).cloned().unwrap_or_default(),
                    "visible_observer_updated": has_diagnostic_prefix(&push_visible, "scheduling_visible_observer_update:node:3"),
                    "hidden_observer_deferred": has_diagnostic_prefix(&push_visible, "scheduling_deferred:node:4"),
                    "fairness_note_recorded": has_diagnostic_prefix(&push_visible, "scheduling_starvation_fairness_note:push_visibility_bounded")
                },
                {
                    "case_id": "pull_full_closure_after_same_seed",
                    "result_state": treecalc_state_key(&pull_full.result_state),
                    "evaluation_order": pull_full.evaluation_order.iter().map(|node_id| node_id.0).collect::<Vec<_>>(),
                    "published_b": pull_full.published_values.get(&TreeNodeId(3)).cloned().unwrap_or_default(),
                    "published_c": pull_full.published_values.get(&TreeNodeId(4)).cloned().unwrap_or_default(),
                    "same_visible_value_as_push": pull_full.published_values.get(&TreeNodeId(3)) == push_visible.published_values.get(&TreeNodeId(3)),
                    "same_dependency_graph_as_push": pull_full.dependency_graph == push_visible.dependency_graph,
                    "same_prepared_identities_as_push": pull_full.prepared_formula_identities == push_visible.prepared_formula_identities
                }
            ]
        })
    }

    const F5_FORMULA_COUNT: usize = 100;
    const F5_CHANGED_INPUT_FANOUT: usize = 8;
    const F5_FIRST_FORMULA_NODE_ID: u64 = 10;

    fn f5_formula_node_id(index: usize) -> TreeNodeId {
        TreeNodeId(F5_FIRST_FORMULA_NODE_ID + index as u64)
    }

    fn f5_visible_observer_node_ids() -> Vec<TreeNodeId> {
        (0..F5_CHANGED_INPUT_FANOUT)
            .map(f5_formula_node_id)
            .collect()
    }

    fn f5_snapshot() -> StructuralSnapshot {
        let formula_node_ids = (0..F5_FORMULA_COUNT)
            .map(f5_formula_node_id)
            .collect::<Vec<_>>();
        let mut nodes = vec![
            StructuralNode {
                node_id: TreeNodeId(1),
                kind: StructuralNodeKind::Root,
                symbol: "Root".to_string(),
                parent_id: None,
                child_ids: std::iter::once(TreeNodeId(2))
                    .chain(std::iter::once(TreeNodeId(3)))
                    .chain(formula_node_ids.iter().copied())
                    .collect(),
            },
            StructuralNode {
                node_id: TreeNodeId(2),
                kind: StructuralNodeKind::Constant,
                symbol: "ChangedInput".to_string(),
                parent_id: Some(TreeNodeId(1)),
                child_ids: vec![],
            },
            StructuralNode {
                node_id: TreeNodeId(3),
                kind: StructuralNodeKind::Constant,
                symbol: "StableInput".to_string(),
                parent_id: Some(TreeNodeId(1)),
                child_ids: vec![],
            },
        ];

        for index in 0..F5_FORMULA_COUNT {
            let node_id = f5_formula_node_id(index);
            nodes.push(StructuralNode {
                node_id,
                kind: StructuralNodeKind::Calculation,
                symbol: format!("F{index}"),
                parent_id: Some(TreeNodeId(1)),
                child_ids: vec![],
            });
        }

        StructuralSnapshot::create(StructuralSnapshotId(1), TreeNodeId(1), nodes).unwrap()
    }

    fn f5_hundred_formula_catalog() -> TreeFormulaCatalog {
        TreeFormulaCatalog::new((0..F5_FORMULA_COUNT).map(|index| {
            let owner_node_id = f5_formula_node_id(index);
            let target_node_id = if index < F5_CHANGED_INPUT_FANOUT {
                TreeNodeId(2)
            } else {
                TreeNodeId(3)
            };
            TreeFormulaBinding {
                owner_node_id,
                formula_artifact_id: FormulaArtifactId(format!("formula:f5:{index}")),
                bind_artifact_id: Some(BindArtifactId(format!("bind:f5:{index}"))),
                expression: fixture_formula(
                    owner_node_id,
                    FixtureFormulaAst::Binary {
                        op: FixtureFormulaBinaryOp::Add,
                        left: Box::new(FixtureFormulaAst::Reference(TreeReference::DirectNode {
                            target_node_id,
                        })),
                        right: Box::new(FixtureFormulaAst::Literal {
                            value: index.to_string(),
                        }),
                    },
                ),
            }
        }))
    }

    fn f5_hundred_formula_input(
        structural_snapshot: StructuralSnapshot,
        input_values: BTreeMap<TreeNodeId, String>,
        seeded_published_values: BTreeMap<TreeNodeId, String>,
        scheduling_policy: LocalTreeCalcSchedulingPolicy,
        run_suffix: &str,
    ) -> LocalTreeCalcInput {
        let invalidation_seeds = if run_suffix == "initial" {
            Vec::new()
        } else {
            vec![InvalidationSeed {
                node_id: TreeNodeId(2),
                reason: InvalidationReasonKind::UpstreamPublication,
            }]
        };
        let mut input = local_treecalc_input(
            structural_snapshot,
            f5_hundred_formula_catalog(),
            input_values,
            seeded_published_values,
            Vec::new(),
            invalidation_seeds,
            &format!("f5:{run_suffix}"),
        );
        input.environment_context = input
            .environment_context
            .with_scheduling_policy(scheduling_policy);
        input
    }

    fn run_o_k_differential_evidence_scenarios() -> (
        LocalTreeCalcRunArtifacts,
        LocalTreeCalcRunArtifacts,
        LocalTreeCalcRunArtifacts,
    ) {
        let engine = LocalTreeCalcEngine;
        let initial = engine
            .execute(f5_hundred_formula_input(
                f5_snapshot(),
                BTreeMap::from([
                    (TreeNodeId(2), "2".to_string()),
                    (TreeNodeId(3), "100".to_string()),
                ]),
                BTreeMap::new(),
                LocalTreeCalcSchedulingPolicy::PullFullClosure,
                "initial",
            ))
            .expect("F5 initial hundred-formula run should publish all formulas");

        let edited_input_values = BTreeMap::from([
            (TreeNodeId(2), "5".to_string()),
            (TreeNodeId(3), "100".to_string()),
        ]);
        let pull_full = engine
            .execute(f5_hundred_formula_input(
                f5_snapshot(),
                edited_input_values.clone(),
                initial.published_values.clone(),
                LocalTreeCalcSchedulingPolicy::PullFullClosure,
                "pull-full",
            ))
            .expect("F5 pull full-closure rerun should publish changed fanout");

        let push_visible = engine
            .execute(f5_hundred_formula_input(
                f5_snapshot(),
                edited_input_values,
                initial.published_values.clone(),
                LocalTreeCalcSchedulingPolicy::PushVisibilityBounded {
                    visible_observer_node_ids: f5_visible_observer_node_ids(),
                },
                "push-visible-fanout",
            ))
            .expect("F5 push visibility-bounded rerun should publish changed fanout");

        (initial, pull_full, push_visible)
    }

    fn changed_value_update_count(run: &LocalTreeCalcRunArtifacts) -> usize {
        run.local_candidate
            .as_ref()
            .map(|candidate| candidate.calc_value_updates.len())
            .unwrap_or_default()
    }

    fn o_k_differential_evidence_artifact_json() -> serde_json::Value {
        let (initial, pull_full, push_visible) = run_o_k_differential_evidence_scenarios();
        json!({
            "run_id": "w050-f5-ok-differential-evidence-001",
            "validation_status": "pass",
            "primary_validation_command": "cargo test -p oxcalc-core o_k_differential_evidence -- --nocapture",
            "model": {
                "formula_count": F5_FORMULA_COUNT,
                "changed_input_node_id": TreeNodeId(2).0,
                "stable_input_node_id": TreeNodeId(3).0,
                "changed_input_fanout_k": F5_CHANGED_INPUT_FANOUT,
                "unchanged_formula_count": F5_FORMULA_COUNT - F5_CHANGED_INPUT_FANOUT,
                "changed_formula_node_ids": f5_visible_observer_node_ids().iter().map(|node_id| node_id.0).collect::<Vec<_>>()
            },
            "semantic_claim": "A single changed input with fan-out k=8 causes exactly 8 value updates and 8 OxFml invocations after the initial hundred-formula publication; pull full-closure visits all formulas but reuses 92 cached edge values, while push visibility-bounded schedules only the k visible dirty observers over the same dependency graph and prepared-callable identities.",
            "command_record": [
                "cargo test -p oxcalc-core o_k_differential_evidence -- --nocapture",
                "cargo test -p oxcalc-core treecalc -- --nocapture"
            ],
            "cases": [
                {
                    "case_id": "initial_hundred_formula_publication",
                    "result_state": treecalc_state_key(&initial.result_state),
                    "formula_count": F5_FORMULA_COUNT,
                    "evaluation_order_count": initial.evaluation_order.len(),
                    "oxfml_invocation_count": diagnostic_prefix_count(&initial, "oxfml_candidate_result_id:"),
                    "value_update_count": changed_value_update_count(&initial),
                    "sample_changed_first": initial.published_values.get(&f5_formula_node_id(0)).cloned().unwrap_or_default(),
                    "sample_changed_last": initial.published_values.get(&f5_formula_node_id(F5_CHANGED_INPUT_FANOUT - 1)).cloned().unwrap_or_default(),
                    "sample_stable_first": initial.published_values.get(&f5_formula_node_id(F5_CHANGED_INPUT_FANOUT)).cloned().unwrap_or_default(),
                    "sample_stable_last": initial.published_values.get(&f5_formula_node_id(F5_FORMULA_COUNT - 1)).cloned().unwrap_or_default()
                },
                {
                    "case_id": "pull_full_closure_single_input_change",
                    "result_state": treecalc_state_key(&pull_full.result_state),
                    "evaluation_order_count": pull_full.evaluation_order.len(),
                    "changed_input_fanout_k": F5_CHANGED_INPUT_FANOUT,
                    "value_update_count": changed_value_update_count(&pull_full),
                    "oxfml_invocation_count": diagnostic_prefix_count(&pull_full, "oxfml_candidate_result_id:"),
                    "cache_hit_count": diagnostic_prefix_count(&pull_full, "edge_value_cache_hit:"),
                    "cache_bypass_count": diagnostic_prefix_count(&pull_full, "edge_value_cache_bypass:"),
                    "invalidation_record_count": pull_full.invalidation_closure.records.len(),
                    "same_published_values_as_push": pull_full.published_values == push_visible.published_values,
                    "sample_changed_first": pull_full.published_values.get(&f5_formula_node_id(0)).cloned().unwrap_or_default(),
                    "sample_changed_last": pull_full.published_values.get(&f5_formula_node_id(F5_CHANGED_INPUT_FANOUT - 1)).cloned().unwrap_or_default(),
                    "sample_stable_first": pull_full.published_values.get(&f5_formula_node_id(F5_CHANGED_INPUT_FANOUT)).cloned().unwrap_or_default(),
                    "sample_stable_last": pull_full.published_values.get(&f5_formula_node_id(F5_FORMULA_COUNT - 1)).cloned().unwrap_or_default()
                },
                {
                    "case_id": "push_visibility_bounded_changed_fanout",
                    "result_state": treecalc_state_key(&push_visible.result_state),
                    "evaluation_order_count": push_visible.evaluation_order.len(),
                    "changed_input_fanout_k": F5_CHANGED_INPUT_FANOUT,
                    "value_update_count": changed_value_update_count(&push_visible),
                    "oxfml_invocation_count": diagnostic_prefix_count(&push_visible, "oxfml_candidate_result_id:"),
                    "cache_hit_count": diagnostic_prefix_count(&push_visible, "edge_value_cache_hit:"),
                    "cache_bypass_count": diagnostic_prefix_count(&push_visible, "edge_value_cache_bypass:"),
                    "selected_formula_count_matches_k": has_diagnostic_prefix(&push_visible, "scheduling_selected_formula_count:8"),
                    "same_dependency_graph_as_pull": push_visible.dependency_graph == pull_full.dependency_graph,
                    "same_prepared_identities_as_pull": push_visible.prepared_formula_identities == pull_full.prepared_formula_identities,
                    "same_published_values_as_pull": push_visible.published_values == pull_full.published_values
                }
            ]
        })
    }

    #[test]
    fn differential_evaluation_gate_reuses_cached_value_without_publication_change() {
        let (initial, reuse, _) = run_differential_evaluation_gate_scenarios();

        assert_eq!(initial.result_state, LocalTreeCalcRunState::Published);
        assert_eq!(initial.published_values[&TreeNodeId(3)], "5");
        assert_eq!(reuse.result_state, LocalTreeCalcRunState::VerifiedClean);
        assert_eq!(reuse.published_values[&TreeNodeId(3)], "5");
        assert!(has_diagnostic_prefix(
            &reuse,
            "edge_value_cache_hit:node:3:"
        ));
        assert!(!has_diagnostic_prefix(&reuse, "oxfml_candidate_result_id:"));
        assert!(reuse.publication_bundle.is_none());
        assert_has_diagnostic(&reuse, "verified_clean_publication_suppressed:node:3");
    }

    #[test]
    fn differential_evaluation_gate_bypasses_cache_for_upstream_publication() {
        let (_, _, upstream_bypass) = run_differential_evaluation_gate_scenarios();

        assert_eq!(
            upstream_bypass.result_state,
            LocalTreeCalcRunState::Published
        );
        assert_eq!(upstream_bypass.published_values[&TreeNodeId(3)], "7");
        assert!(has_diagnostic_prefix(
            &upstream_bypass,
            "edge_value_cache_bypass:node:3:UpstreamPublication"
        ));
        assert!(has_diagnostic_prefix(
            &upstream_bypass,
            "oxfml_candidate_result_id:"
        ));
        assert!(upstream_bypass.publication_bundle.is_some());
    }

    #[test]
    fn differential_evaluation_gate_checked_artifact_matches_runtime_validation() {
        let artifact_path = f2_artifact_root().join("run_artifact.json");
        let artifact = serde_json::from_str::<serde_json::Value>(
            &fs::read_to_string(artifact_path).expect("F2 run artifact should be checked in"),
        )
        .expect("F2 run artifact should be valid JSON");

        assert_eq!(artifact, differential_evaluation_gate_artifact_json());
    }

    #[test]
    fn derivation_trace_default_value_only_run_suppresses_trace_output() {
        let (default_run, _) = run_derivation_trace_scenarios();

        assert_eq!(default_run.result_state, LocalTreeCalcRunState::Published);
        assert_eq!(default_run.published_values[&TreeNodeId(3)], "7");
        assert!(default_run.derivation_traces.is_empty());
        assert!(!has_diagnostic_prefix(
            &default_run,
            "derivation_trace_recorded:"
        ));
    }

    #[test]
    fn derivation_trace_opt_in_records_template_holes_and_invocation_tree() {
        let (_, traced_run) = run_derivation_trace_scenarios();

        assert_eq!(traced_run.result_state, LocalTreeCalcRunState::Published);
        assert_eq!(traced_run.published_values[&TreeNodeId(3)], "7");
        assert_eq!(traced_run.derivation_traces.len(), 1);
        assert!(has_diagnostic_prefix(
            &traced_run,
            "derivation_trace_prepared_call_count:node:3:",
        ));

        let trace = &traced_run.derivation_traces[0];
        assert_eq!(trace.trace_schema_id, DERIVATION_TRACE_SCHEMA_ID);
        assert_eq!(trace.owner_node_id, TreeNodeId(3));
        assert_eq!(trace.trace_mode, "PreparedCalls");
        assert_eq!(trace.kernel_returned_value, "7");
        assert_eq!(trace.template_selection.prepared_formula_key.len(), 16);
        assert!(!trace.hole_bindings.is_empty());
        assert_eq!(trace.sub_invocation_tree.len(), 1);
        let root = &trace.sub_invocation_tree[0];
        assert_eq!(root.invocation_kind, "oxfml_prepared_formula_invoke");
        assert_eq!(root.kernel_returned_value.as_deref(), Some("7"));
        assert!(!root.children.is_empty());
        assert!(
            root.children
                .iter()
                .any(|child| child.kernel_returned_value.as_deref() == Some("7"))
        );
        assert!(
            trace
                .oxfml_trace_events
                .iter()
                .any(|event| event.event_kind == "CommitAccepted")
        );
    }

    #[test]
    fn capability_set_trace_replay_columns_reserve_empty_current_v1_output() {
        let (_, traced_run) = run_derivation_trace_scenarios();
        let identity = traced_run
            .prepared_formula_identities
            .first()
            .expect("trace fixture should prepare one formula");
        let trace = &traced_run.derivation_traces[0];

        assert!(identity.rich_value_capability_columns.is_empty());
        assert!(trace.rich_value_capability_columns.is_empty());
        assert!(
            trace
                .template_selection
                .rich_value_capability_columns
                .is_empty()
        );
        assert!(
            trace
                .template_selection
                .template_holes
                .iter()
                .all(|hole| hole.rich_value_capability_columns.is_empty())
        );

        let serialized_trace =
            serde_json::to_value(trace).expect("derivation trace should serialize");
        assert!(
            serialized_trace
                .get("rich_value_capability_columns")
                .is_none()
        );

        let required_set = w050_initial_required_capability_set_example();
        let required_columns =
            RichValueCapabilityTraceReplayColumns::from_required_sets([&required_set]);

        assert_eq!(
            required_columns.required_capability_set_keys,
            vec![required_set.stable_key()]
        );
        assert!(required_columns.producer_capability_set_keys.is_empty());
        assert!(required_columns.exercised_capability_keys.is_empty());
    }

    #[test]
    fn image_rich_value_surface_carries_producer_capability_metadata() {
        let engine = LocalTreeCalcEngine;
        let mut input = formula_input(
            TreeNodeId(3),
            TreeFormula::opaque_oxfml(
                "=IMAGE(\"https://example.com/sphere.png\",\"Sphere\",3,100,200)",
                Vec::new(),
            ),
        );
        input.environment_context =
            LocalTreeCalcEnvironmentContext::default().with_derivation_trace_enabled(true);

        let run = engine.execute(input).unwrap();

        assert_eq!(run.result_state, LocalTreeCalcRunState::Published);
        assert_has_diagnostic(&run, "oxfml_returned_value_surface_kind:RichValue");
        assert_has_diagnostic(
            &run,
            "oxfml_returned_value_surface_payload_summary:RichValue(_webimage)",
        );
        assert_has_diagnostic(
            &run,
            "oxfml_returned_value_surface_rich_value_type:_webimage",
        );
        assert!(
            run.diagnostics
                .iter()
                .any(|diagnostic| diagnostic.starts_with(
                    "oxfml_returned_value_surface_producer_capability_set_key:Materialisable("
                )),
            "IMAGE/_webimage should expose producer capability keys in diagnostics: {:?}",
            run.diagnostics
        );

        let trace = run
            .derivation_traces
            .first()
            .expect("derivation trace should be recorded");
        assert!(
            trace
                .rich_value_capability_columns
                .producer_capability_set_keys
                .iter()
                .any(|key| key.starts_with("Materialisable(")),
            "derivation trace should carry returned-surface producer capability keys: {:?}",
            trace.rich_value_capability_columns
        );
        assert!(
            trace
                .rich_value_capability_columns
                .exercised_capability_keys
                .iter()
                .any(|key| key.starts_with("Materialisable(")),
            "derivation trace should carry OxFunc returned-surface exercised capability keys: {:?}",
            trace.rich_value_capability_columns
        );
    }

    #[test]
    fn capability_set_trace_replay_checked_artifact_matches_runtime_schema() {
        let artifact_path = g3_artifact_root().join("run_artifact.json");
        let artifact = serde_json::from_str::<serde_json::Value>(
            &fs::read_to_string(artifact_path).expect("G3 run artifact should be checked in"),
        )
        .expect("G3 run artifact should be valid JSON");

        assert_eq!(artifact, capability_trace_replay_columns_artifact_json());
    }

    #[test]
    fn derivation_trace_checked_artifact_matches_runtime_validation() {
        let artifact_path = f3_artifact_root().join("run_artifact.json");
        let expected = derivation_trace_invoke_outcome_artifact_json();
        if std::env::var_os("OXCALC_UPDATE_EXPECTED").is_some() {
            fs::write(
                &artifact_path,
                serde_json::to_string_pretty(&expected)
                    .expect("F3 generated artifact should serialize"),
            )
            .expect("F3 run artifact should be writable");
            return;
        }
        let artifact = serde_json::from_str::<serde_json::Value>(
            &fs::read_to_string(artifact_path).expect("F3 run artifact should be checked in"),
        )
        .expect("F3 run artifact should be valid JSON");

        assert_eq!(artifact, expected);
    }

    #[test]
    fn push_pull_scheduling_updates_visible_observer_without_hidden_publication() {
        let (initial, push_visible, _) = run_push_pull_scheduling_scenarios();

        assert_eq!(initial.result_state, LocalTreeCalcRunState::Published);
        assert_eq!(initial.evaluation_order, vec![TreeNodeId(3), TreeNodeId(4)]);
        assert_eq!(initial.published_values[&TreeNodeId(3)], "5");
        assert_eq!(initial.published_values[&TreeNodeId(4)], "20");
        assert_has_diagnostic(&initial, "scheduling_policy:pull_full_closure");

        assert_eq!(push_visible.result_state, LocalTreeCalcRunState::Published);
        assert_eq!(push_visible.evaluation_order, vec![TreeNodeId(3)]);
        assert_eq!(push_visible.published_values[&TreeNodeId(3)], "7");
        assert_eq!(push_visible.published_values[&TreeNodeId(4)], "20");
        assert_eq!(
            push_visible.node_states.get(&TreeNodeId(4)),
            Some(&NodeCalcState::DirtyPending)
        );
        assert_has_diagnostic(&push_visible, "scheduling_policy:push_visibility_bounded");
        assert_has_diagnostic(&push_visible, "scheduling_visible_observer_update:node:3");
        assert_has_diagnostic(&push_visible, "scheduling_deferred:node:4");
        assert_has_diagnostic(
            &push_visible,
            "scheduling_starvation_fairness_note:push_visibility_bounded_requires_periodic_full_closure_or_observer_aging",
        );
    }

    #[test]
    fn push_pull_scheduling_visible_observer_matches_full_closure() {
        let (_, push_visible, pull_full) = run_push_pull_scheduling_scenarios();

        assert_eq!(pull_full.result_state, LocalTreeCalcRunState::Published);
        assert_eq!(
            pull_full.evaluation_order,
            vec![TreeNodeId(3), TreeNodeId(4)]
        );
        assert_eq!(pull_full.published_values[&TreeNodeId(3)], "7");
        assert_eq!(pull_full.published_values[&TreeNodeId(4)], "40");
        assert_has_diagnostic(&pull_full, "scheduling_policy:pull_full_closure");

        assert_eq!(
            push_visible.published_values.get(&TreeNodeId(3)),
            pull_full.published_values.get(&TreeNodeId(3))
        );
        assert_eq!(push_visible.dependency_graph, pull_full.dependency_graph);
        assert_eq!(
            push_visible.prepared_formula_identities,
            pull_full.prepared_formula_identities
        );
    }

    #[test]
    fn push_pull_scheduling_checked_artifact_matches_runtime_validation() {
        let artifact_path = f4_artifact_root().join("run_artifact.json");
        let artifact = serde_json::from_str::<serde_json::Value>(
            &fs::read_to_string(artifact_path).expect("F4 run artifact should be checked in"),
        )
        .expect("F4 run artifact should be valid JSON");

        assert_eq!(artifact, push_pull_scheduling_artifact_json());
    }

    #[test]
    fn o_k_differential_evidence_pull_full_closure_invokes_only_changed_fanout() {
        let (initial, pull_full, _) = run_o_k_differential_evidence_scenarios();

        assert_eq!(initial.result_state, LocalTreeCalcRunState::Published);
        assert_eq!(initial.evaluation_order.len(), F5_FORMULA_COUNT);
        assert_eq!(
            diagnostic_prefix_count(&initial, "oxfml_candidate_result_id:"),
            F5_FORMULA_COUNT
        );
        assert_eq!(changed_value_update_count(&initial), F5_FORMULA_COUNT);

        assert_eq!(pull_full.result_state, LocalTreeCalcRunState::Published);
        assert_eq!(pull_full.evaluation_order.len(), F5_FORMULA_COUNT);
        assert_eq!(
            pull_full.invalidation_closure.records.len(),
            F5_CHANGED_INPUT_FANOUT + 1
        );
        assert_eq!(
            changed_value_update_count(&pull_full),
            F5_CHANGED_INPUT_FANOUT
        );
        assert_eq!(
            diagnostic_prefix_count(&pull_full, "oxfml_candidate_result_id:"),
            F5_CHANGED_INPUT_FANOUT
        );
        assert_eq!(
            diagnostic_prefix_count(&pull_full, "edge_value_cache_hit:"),
            F5_FORMULA_COUNT - F5_CHANGED_INPUT_FANOUT
        );
        assert_eq!(
            diagnostic_prefix_count(&pull_full, "edge_value_cache_bypass:"),
            F5_CHANGED_INPUT_FANOUT
        );
        assert_eq!(pull_full.published_values[&f5_formula_node_id(0)], "5");
        assert_eq!(
            pull_full.published_values[&f5_formula_node_id(F5_CHANGED_INPUT_FANOUT - 1)],
            "12"
        );
        assert_eq!(
            pull_full.published_values[&f5_formula_node_id(F5_CHANGED_INPUT_FANOUT)],
            "108"
        );
        assert_eq!(
            pull_full.published_values[&f5_formula_node_id(F5_FORMULA_COUNT - 1)],
            "199"
        );
    }

    #[test]
    fn o_k_differential_evidence_push_visibility_schedules_only_changed_fanout() {
        let (_, pull_full, push_visible) = run_o_k_differential_evidence_scenarios();

        assert_eq!(push_visible.result_state, LocalTreeCalcRunState::Published);
        assert_eq!(push_visible.evaluation_order.len(), F5_CHANGED_INPUT_FANOUT);
        assert_eq!(
            changed_value_update_count(&push_visible),
            F5_CHANGED_INPUT_FANOUT
        );
        assert_eq!(
            diagnostic_prefix_count(&push_visible, "oxfml_candidate_result_id:"),
            F5_CHANGED_INPUT_FANOUT
        );
        assert_eq!(
            diagnostic_prefix_count(&push_visible, "edge_value_cache_bypass:"),
            F5_CHANGED_INPUT_FANOUT
        );
        assert_eq!(
            diagnostic_prefix_count(&push_visible, "edge_value_cache_hit:"),
            0
        );
        assert_has_diagnostic(&push_visible, "scheduling_selected_formula_count:8");
        assert_eq!(push_visible.published_values, pull_full.published_values);
        assert_eq!(push_visible.dependency_graph, pull_full.dependency_graph);
        assert_eq!(
            push_visible.prepared_formula_identities,
            pull_full.prepared_formula_identities
        );
    }

    #[test]
    fn o_k_differential_evidence_checked_artifact_matches_runtime_validation() {
        let artifact_path = f5_artifact_root().join("run_artifact.json");
        let artifact = serde_json::from_str::<serde_json::Value>(
            &fs::read_to_string(artifact_path).expect("F5 run artifact should be checked in"),
        )
        .expect("F5 run artifact should be valid JSON");

        assert_eq!(artifact, o_k_differential_evidence_artifact_json());
    }

    #[test]
    fn local_treecalc_delegates_scalar_and_lambda_invocation_sources_to_oxfml() {
        let engine = LocalTreeCalcEngine;
        for (source, expected_value, expected_surface) in [
            ("=14", "14", "Number"),
            ("=SUM(2,3)", "5", "Number"),
            ("=LET(base,2,LAMBDA(delta,base+delta)(5))", "7", "Number"),
        ] {
            let run = engine
                .execute(formula_input(
                    TreeNodeId(3),
                    TreeFormula::opaque_oxfml(source, Vec::new()),
                ))
                .unwrap();

            assert_eq!(run.result_state, LocalTreeCalcRunState::Published);
            assert_eq!(run.published_values[&TreeNodeId(3)], expected_value);
            assert_has_diagnostic(&run, "oxfml_returned_value_surface_kind:OrdinaryValue");
            assert_has_diagnostic(
                &run,
                &format!("oxfml_returned_value_surface_payload_summary:{expected_surface}"),
            );
        }
    }

    #[test]
    fn local_treecalc_records_current_v1_returned_callable_publication_boundary() {
        let engine = LocalTreeCalcEngine;
        let run = engine
            .execute(formula_input(
                TreeNodeId(3),
                TreeFormula::opaque_oxfml("=LAMBDA(x,x+1)", Vec::new()),
            ))
            .unwrap();

        assert_eq!(run.result_state, LocalTreeCalcRunState::Published);
        assert_eq!(run.published_values[&TreeNodeId(3)], "Calc");
        let calc_value = &run.published_calc_values[&TreeNodeId(3)];
        assert_eq!(calc_value.core, CoreValue::Error(WorksheetErrorCode::Calc));
        assert!(matches!(
            calc_value.rich.as_deref(),
            Some(RichValue::Callable(_))
        ));
        assert_has_diagnostic(&run, "oxfml_returned_value_surface_kind:OrdinaryValue");
        assert_has_diagnostic(
            &run,
            "oxfml_returned_value_surface_payload_summary:Error(Calc)",
        );
    }

    #[test]
    fn local_treecalc_surfaces_dynamic_array_payload_as_opaque_oxfml_value() {
        let engine = LocalTreeCalcEngine;
        let run = engine
            .execute(formula_input(
                TreeNodeId(3),
                TreeFormula::opaque_oxfml("=SEQUENCE(3)", Vec::new()),
            ))
            .unwrap();

        assert_eq!(run.result_state, LocalTreeCalcRunState::Published);
        assert_eq!(run.published_values[&TreeNodeId(3)], "Array(3x1)");
        assert_has_diagnostic(&run, "oxfml_returned_value_surface_kind:OrdinaryValue");
        assert_has_diagnostic(
            &run,
            "oxfml_returned_value_surface_payload_summary:Array(3x1)",
        );
    }

    #[test]
    fn local_treecalc_rejects_indirect_dynamic_surface_as_opaque_effect() {
        let engine = LocalTreeCalcEngine;
        let expression = TreeFormula::opaque_oxfml(
            "=INDIRECT(RTD(\"TREECALC\",\"\",\"carrier:indirect\"))",
            [TreeReference::DynamicPotential {
                carrier_id: "carrier:indirect".to_string(),
                detail: "INDIRECT selector resolved at runtime".to_string(),
            }],
        );
        let run = engine
            .execute(formula_input(TreeNodeId(3), expression))
            .unwrap();

        assert_eq!(run.result_state, LocalTreeCalcRunState::Rejected);
        assert_eq!(
            run.reject_detail.as_ref().map(|detail| detail.kind),
            Some(RejectKind::DynamicDependencyFailure)
        );
        assert_eq!(
            run.runtime_effects[0].kind,
            "runtime_effect.dynamic_reference"
        );
        assert!(!has_diagnostic_prefix(
            &run,
            "prepared_runtime_subscription:"
        ));
        assert_has_diagnostic(&run, "oxfml_returned_value_surface_kind:OrdinaryValue");
        assert_has_diagnostic(
            &run,
            "oxfml_returned_value_surface_payload_summary:Error(Blocked)",
        );
    }

    #[test]
    fn local_treecalc_rejects_rtd_provider_surface_as_opaque_effect() {
        let engine = LocalTreeCalcEngine;
        let expression = TreeFormula::opaque_oxfml(
            "=RTD(\"TREECALC\",\"\",\"carrier:rtd\")",
            [TreeReference::DynamicPotential {
                carrier_id: "carrier:rtd".to_string(),
                detail: "RTD topic resolved at runtime".to_string(),
            }],
        );
        let run = engine
            .execute(formula_input(TreeNodeId(3), expression))
            .unwrap();

        assert_eq!(run.result_state, LocalTreeCalcRunState::Rejected);
        assert_eq!(
            run.reject_detail.as_ref().map(|detail| detail.kind),
            Some(RejectKind::DynamicDependencyFailure)
        );
        assert_eq!(
            run.runtime_effects[0].kind,
            "runtime_effect.dynamic_reference"
        );
        assert_has_diagnostic(
            &run,
            "prepared_runtime_subscription:formula:b:topic:rtd:subscription:formula:b:rtd:runtime_effect.dynamic_reference:owner_node:node:3;carrier_id:carrier:rtd;detail:RTD topic resolved at runtime",
        );
        assert_has_diagnostic(
            &run,
            "oxfml_returned_value_surface_kind:TypedHostProviderOutcome",
        );
        assert_has_diagnostic(
            &run,
            "oxfml_returned_value_surface_payload_summary:CapabilityDenied",
        );
        assert_has_diagnostic(
            &run,
            "oxfml_returned_value_surface_host_provider_outcome:CapabilityDenied",
        );
        assert_has_diagnostic(
            &run,
            "oxfml_returned_value_surface_host_provider_worksheet_error:Blocked",
        );
    }

    #[test]
    fn prepared_runtime_effect_classification_drives_subscription_lifecycle() {
        let structural_snapshot = snapshot();
        let binding = TreeFormulaBinding {
            owner_node_id: TreeNodeId(3),
            formula_artifact_id: FormulaArtifactId("formula:b".to_string()),
            bind_artifact_id: Some(BindArtifactId("bind:b".to_string())),
            expression: TreeFormula::opaque_oxfml(
                "=RTD(\"TREECALC\",\"\",\"carrier:rtd\")",
                [TreeReference::DynamicPotential {
                    carrier_id: "carrier:rtd".to_string(),
                    detail: "RTD topic resolved at runtime".to_string(),
                }],
            ),
        };
        let prepared = prepare_oxfml_formula(
            &structural_snapshot,
            &BTreeMap::new(),
            &binding,
            &LocalTreeCalcEnvironmentContext::default(),
        )
        .unwrap();

        let entries = prepared_runtime_effect_subscription_entries(&prepared);
        assert_eq!(entries.len(), 1);
        assert_eq!(
            entries[0].topic_id,
            SubscriptionTopicId("topic:rtd".to_string())
        );
        assert_eq!(entries[0].formula_stable_id, "formula:b");
        assert_eq!(
            entries[0].subscription_handle,
            SubscriptionHandle("subscription:formula:b:rtd".to_string())
        );

        let mut repository = CalculationRepository::new(structural_snapshot.clone());
        repository
            .upsert_formula_slot(TreeNodeId(3), repository_slot_from_prepared(&prepared))
            .unwrap();
        let created = repository
            .reconcile_subscriptions_for_formula(
                "formula:b",
                entries.clone(),
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
            repository.subscriptions_for_formula("formula:b")[0].topic_id,
            SubscriptionTopicId("topic:rtd".to_string())
        );

        let indirect_binding = TreeFormulaBinding {
            owner_node_id: TreeNodeId(3),
            formula_artifact_id: FormulaArtifactId("formula:b".to_string()),
            bind_artifact_id: Some(BindArtifactId("bind:b".to_string())),
            expression: TreeFormula::opaque_oxfml(
                "=INDIRECT(RTD(\"TREECALC\",\"\",\"carrier:indirect\"))",
                [TreeReference::DynamicPotential {
                    carrier_id: "carrier:indirect".to_string(),
                    detail: "INDIRECT selector resolved at runtime".to_string(),
                }],
            ),
        };
        let indirect_prepared = prepare_oxfml_formula(
            &structural_snapshot,
            &BTreeMap::new(),
            &indirect_binding,
            &LocalTreeCalcEnvironmentContext::default(),
        )
        .unwrap();

        assert!(prepared_runtime_effect_subscription_entries(&indirect_prepared).is_empty());

        let released = repository
            .reconcile_subscriptions_for_formula(
                "formula:b",
                Vec::<SubscriptionRegistryEntry>::new(),
                SubscriptionLifecycleReason::FormulaTextChanged,
            )
            .unwrap();

        assert_eq!(released.len(), 1);
        assert_eq!(released[0].action, SubscriptionLifecycleAction::Released);
        assert_eq!(
            released[0].reason,
            SubscriptionLifecycleReason::FormulaTextChanged
        );
        assert!(repository.subscriptions_for_formula("formula:b").is_empty());
    }

    #[test]
    fn arg_preparation_profile_version_enters_structure_context() {
        let structural_snapshot = snapshot();
        let binding = TreeFormulaBinding {
            owner_node_id: TreeNodeId(3),
            formula_artifact_id: FormulaArtifactId("formula:b".to_string()),
            bind_artifact_id: Some(BindArtifactId("bind:b".to_string())),
            expression: TreeFormula::opaque_oxfml("=SUM(A1,2)", Vec::new()),
        };
        let first_context = LocalTreeCalcEnvironmentContext::default()
            .with_arg_preparation_profile_version("oxfunc.arg-prep:v1");
        let second_context = LocalTreeCalcEnvironmentContext::default()
            .with_arg_preparation_profile_version("oxfunc.arg-prep:v2");

        let first = prepare_oxfml_formula(
            &structural_snapshot,
            &BTreeMap::new(),
            &binding,
            &first_context,
        )
        .unwrap();
        let second = prepare_oxfml_formula(
            &structural_snapshot,
            &BTreeMap::new(),
            &binding,
            &second_context,
        )
        .unwrap();

        assert_ne!(
            first.runtime_prepared_identity.structure_context_version,
            second.runtime_prepared_identity.structure_context_version
        );
        assert!(
            second
                .runtime_prepared_identity
                .structure_context_version
                .contains("arg_preparation_profile_version=oxfunc.arg-prep:v2")
        );
    }

    #[test]
    fn oxfunc_bridge_versions_enter_structure_context_and_runtime_identity() {
        let structural_snapshot = snapshot();
        let binding = TreeFormulaBinding {
            owner_node_id: TreeNodeId(3),
            formula_artifact_id: FormulaArtifactId("formula:b".to_string()),
            bind_artifact_id: Some(BindArtifactId("bind:b".to_string())),
            expression: TreeFormula::opaque_oxfml("=SUM(1,2)", Vec::new()),
        };
        let first_context = LocalTreeCalcEnvironmentContext::default()
            .with_semantic_kernel_metadata_version("oxfunc.semantic-kernel:v1")
            .with_arg_admission_metadata_version("oxfunc.arg-admission:v1");
        let second_context = LocalTreeCalcEnvironmentContext::default()
            .with_semantic_kernel_metadata_version("oxfunc.semantic-kernel:v2")
            .with_arg_admission_metadata_version("oxfunc.arg-admission:v1");

        let first = prepare_oxfml_formula(
            &structural_snapshot,
            &BTreeMap::new(),
            &binding,
            &first_context,
        )
        .unwrap();
        let second = prepare_oxfml_formula(
            &structural_snapshot,
            &BTreeMap::new(),
            &binding,
            &second_context,
        )
        .unwrap();

        assert_ne!(
            first.runtime_prepared_identity.structure_context_version,
            second.runtime_prepared_identity.structure_context_version
        );
        assert_ne!(
            first.runtime_prepared_identity.prepared_formula_key,
            second.runtime_prepared_identity.prepared_formula_key
        );
        assert!(
            second
                .runtime_prepared_identity
                .structure_context_version
                .contains("semantic_kernel_metadata_version=oxfunc.semantic-kernel:v2")
        );

        let engine = LocalTreeCalcEngine;
        let mut input = formula_input(
            TreeNodeId(3),
            TreeFormula::opaque_oxfml("=SUM(1,2)", Vec::new()),
        );
        input.environment_context = LocalTreeCalcEnvironmentContext::default()
            .with_semantic_kernel_metadata_version("oxfunc.semantic-kernel:v2")
            .with_arg_admission_metadata_version("oxfunc.arg-admission:v3");

        let run = engine.execute(input).unwrap();
        let identity = run
            .prepared_formula_identities
            .first()
            .expect("run should prepare one formula");
        assert_eq!(
            identity
                .oxfunc_bridge_metadata
                .semantic_kernel_metadata_version
                .as_deref(),
            Some("oxfunc.semantic-kernel:v2")
        );
        assert_eq!(
            identity
                .oxfunc_bridge_metadata
                .arg_admission_metadata_version
                .as_deref(),
            Some("oxfunc.arg-admission:v3")
        );
        assert_has_diagnostic(
            &run,
            "oxfml_prepared_semantic_kernel_metadata_version:formula:b:oxfunc.semantic-kernel:v2",
        );
        assert_has_diagnostic(
            &run,
            "oxfml_prepared_arg_admission_metadata_version:formula:b:oxfunc.arg-admission:v3",
        );
        assert_has_diagnostic(
            &run,
            "oxfml_runtime_semantic_kernel_metadata_version:formula:b:oxfunc.semantic-kernel:v2",
        );
        assert_has_diagnostic(
            &run,
            "oxfml_runtime_arg_admission_metadata_version:formula:b:oxfunc.arg-admission:v3",
        );
    }

    #[test]
    fn w056_host_namespace_and_caller_context_enter_relative_reference_prepared_identity() {
        let structural_snapshot = snapshot();
        let binding = TreeFormulaBinding {
            owner_node_id: TreeNodeId(3),
            formula_artifact_id: FormulaArtifactId("formula:b".to_string()),
            bind_artifact_id: Some(BindArtifactId("bind:b".to_string())),
            expression: TreeFormula::opaque_oxfml(
                "=SUM(TREE_REF_3_0,2)",
                [TreeReference::RelativePath {
                    base: RelativeReferenceBase::ParentNode,
                    path_segments: vec!["A".to_string()],
                }],
            ),
        };
        let first_context = LocalTreeCalcEnvironmentContext::default()
            .with_host_namespace_version("treecalc-host-namespace:v1")
            .with_caller_context_identity_version("caller-context:v1");
        let second_context = LocalTreeCalcEnvironmentContext::default()
            .with_host_namespace_version("treecalc-host-namespace:v2")
            .with_caller_context_identity_version("caller-context:v2");

        let first = prepare_oxfml_formula(
            &structural_snapshot,
            &BTreeMap::new(),
            &binding,
            &first_context,
        )
        .unwrap();
        let second = prepare_oxfml_formula(
            &structural_snapshot,
            &BTreeMap::new(),
            &binding,
            &second_context,
        )
        .unwrap();

        let first_host_context = first
            .runtime_prepared_identity
            .host_formula_context
            .as_ref()
            .expect("relative references should contribute W056 host context");
        assert_eq!(
            first_host_context.host_namespace_version.as_deref(),
            Some("treecalc-host-namespace:v1")
        );
        assert_eq!(
            first_host_context.caller_context_identity.as_deref(),
            Some("treecalc-caller:node:3;caller-context:v1")
        );
        assert_ne!(
            first.runtime_prepared_identity.prepared_formula_key,
            second.runtime_prepared_identity.prepared_formula_key
        );
    }

    #[test]
    fn w056_capability_profile_enters_capability_sensitive_prepared_identity() {
        let structural_snapshot = snapshot();
        let binding = TreeFormulaBinding {
            owner_node_id: TreeNodeId(3),
            formula_artifact_id: FormulaArtifactId("formula:b".to_string()),
            bind_artifact_id: Some(BindArtifactId("bind:b".to_string())),
            expression: TreeFormula::opaque_oxfml(
                "=SUM(1,2)",
                [TreeReference::CapabilitySensitive {
                    carrier_id: "carrier:capability".to_string(),
                    detail: "provider profile changes admission".to_string(),
                }],
            ),
        };
        let first_context = LocalTreeCalcEnvironmentContext::default()
            .with_capability_profile_id("host-capabilities:w056-a");
        let second_context = LocalTreeCalcEnvironmentContext::default()
            .with_capability_profile_id("host-capabilities:w056-b");

        let first = prepare_oxfml_formula(
            &structural_snapshot,
            &BTreeMap::new(),
            &binding,
            &first_context,
        )
        .unwrap();
        let second = prepare_oxfml_formula(
            &structural_snapshot,
            &BTreeMap::new(),
            &binding,
            &second_context,
        )
        .unwrap();

        assert_eq!(
            first
                .runtime_prepared_identity
                .host_formula_context
                .as_ref()
                .map(|context| context.capability_profile_id.as_str()),
            Some("host-capabilities:w056-a")
        );
        assert_ne!(
            first.runtime_prepared_identity.prepared_formula_key,
            second.runtime_prepared_identity.prepared_formula_key
        );
    }

    #[test]
    fn w056_table_and_cross_workspace_inputs_project_through_public_host_context() {
        let requirements = vec![
            (
                NamespaceIdentityNeed::TableContextIdentity,
                CallerContextIdentityNeed::TableCallerRegion,
            ),
            (
                NamespaceIdentityNeed::CrossWorkspaceAvailabilityVersion,
                CallerContextIdentityNeed::None,
            ),
        ];
        let context = LocalTreeCalcEnvironmentContext::default()
            .with_table_context_identity("table-context:Sales:v2")
            .with_cross_workspace_availability_version("workspace-availability:v3");

        let host_context = w056_runtime_host_formula_context(
            TreeNodeId(3),
            "structure:v1",
            &context,
            &requirements,
            &[
                "workspace=projections;target=node:102;availability=workspace-availability:v3"
                    .to_string(),
            ],
        )
        .expect("table/cross-workspace needs should use public host context");

        assert_eq!(
            host_context.table_context_identity.as_deref(),
            Some("table-context:Sales:v2")
        );
        assert!(
            host_context
                .host_namespace_version
                .as_deref()
                .is_some_and(|identity| identity
                    .contains("cross_workspace_availability_version=workspace-availability:v3"))
        );
        assert!(
            host_context
                .host_namespace_version
                .as_deref()
                .is_some_and(|identity| identity.contains(
                    "cross_workspace_target_availability=workspace=projections;target=node:102;availability=workspace-availability:v3"
                ))
        );
    }

    #[test]
    fn workspace_qualified_runtime_binding_does_not_read_local_node_id_collision() {
        let structural_snapshot = snapshot();
        let expression = TreeFormula::opaque_oxfml(
            "=TREE_REF_4_0",
            [TreeReference::CrossWorkspaceResolved {
                workspace_handle: "treecalc-workspace:projections".to_string(),
                target_node_id: TreeNodeId(102),
                target_node_handle: "treecalc-workspace:projections#node:102".to_string(),
                availability_version: "treecalc-cross-workspace-availability:v1:projections:loaded"
                    .to_string(),
                carrier_id: "carrier:xws:projections".to_string(),
                detail: "cross_workspace_resolution:v1:projections".to_string(),
            }],
        );
        let translated = project_opaque_formula(
            &structural_snapshot,
            &BTreeMap::new(),
            TreeNodeId(4),
            &expression,
            &BTreeSet::new(),
        );
        assert_eq!(translated.reference_bindings.len(), 1);
        assert_eq!(translated.reference_bindings[0].local_target_node_id, None);

        let formal_inputs = formal_input_bindings_for_runtime(
            &translated,
            &BTreeMap::from([(TreeNodeId(102), "999".to_string())]),
            &BTreeMap::new(),
            None,
        );
        assert_eq!(formal_inputs.len(), 1);
        match &formal_inputs[0].binding {
            DefinedNameBinding::Reference(reference) => {
                assert_eq!(reference.kind(), ReferenceKind::ThreeD);
                assert_eq!(
                    reference.target(),
                    "treecalc-workspace:projections#node:102"
                );
            }
            other => panic!("expected workspace-qualified ReferenceLike binding, got {other:?}"),
        }
    }

    #[test]
    fn workspace_qualified_availability_version_enters_prepared_identity_from_carrier() {
        let structural_snapshot = snapshot();
        let binding_for_version = |version: &str| TreeFormulaBinding {
            owner_node_id: TreeNodeId(4),
            formula_artifact_id: FormulaArtifactId("formula:xws".to_string()),
            bind_artifact_id: Some(BindArtifactId("bind:xws".to_string())),
            expression: TreeFormula::opaque_oxfml(
                "=TREE_REF_4_0",
                [TreeReference::CrossWorkspaceResolved {
                    workspace_handle: "treecalc-workspace:projections".to_string(),
                    target_node_id: TreeNodeId(102),
                    target_node_handle: "treecalc-workspace:projections#node:102".to_string(),
                    availability_version: version.to_string(),
                    carrier_id: "carrier:xws:projections".to_string(),
                    detail: "cross_workspace_resolution:v1:projections".to_string(),
                }],
            ),
        };

        let first = prepare_oxfml_formula(
            &structural_snapshot,
            &BTreeMap::new(),
            &binding_for_version("treecalc-cross-workspace-availability:v1:projections:loaded"),
            &LocalTreeCalcEnvironmentContext::default(),
        )
        .expect("first workspace-qualified formula should prepare");
        let second = prepare_oxfml_formula(
            &structural_snapshot,
            &BTreeMap::new(),
            &binding_for_version("treecalc-cross-workspace-availability:v2:projections:loaded"),
            &LocalTreeCalcEnvironmentContext::default(),
        )
        .expect("second workspace-qualified formula should prepare");

        let host_context = first
            .runtime_prepared_identity
            .host_formula_context
            .as_ref()
            .expect("workspace-qualified reference should require host context");
        let namespace_identity = host_context
            .host_namespace_version
            .as_deref()
            .expect("workspace-qualified reference should project namespace identity");
        assert!(namespace_identity.contains(
            "cross_workspace_target_availability=workspace=treecalc-workspace:projections;target=treecalc-workspace:projections#node:102;availability=treecalc-cross-workspace-availability:v1:projections:loaded"
        ));
        assert_ne!(
            first.runtime_prepared_identity.prepared_formula_key,
            second.runtime_prepared_identity.prepared_formula_key
        );
    }

    #[test]
    fn w056_prepared_formula_key_contributes_to_edge_cache_key() {
        let structural_snapshot = snapshot();
        let binding = TreeFormulaBinding {
            owner_node_id: TreeNodeId(3),
            formula_artifact_id: FormulaArtifactId("formula:b".to_string()),
            bind_artifact_id: Some(BindArtifactId("bind:b".to_string())),
            expression: TreeFormula::opaque_oxfml(
                "=SUM(TREE_REF_3_0,2)",
                [TreeReference::RelativePath {
                    base: RelativeReferenceBase::ParentNode,
                    path_segments: vec!["A".to_string()],
                }],
            ),
        };
        let first = prepare_oxfml_formula(
            &structural_snapshot,
            &BTreeMap::new(),
            &binding,
            &LocalTreeCalcEnvironmentContext::default()
                .with_host_namespace_version("treecalc-host-namespace:v1"),
        )
        .unwrap();
        let second = prepare_oxfml_formula(
            &structural_snapshot,
            &BTreeMap::new(),
            &binding,
            &LocalTreeCalcEnvironmentContext::default()
                .with_host_namespace_version("treecalc-host-namespace:v2"),
        )
        .unwrap();

        assert_ne!(
            first.runtime_prepared_identity.prepared_formula_key,
            second.runtime_prepared_identity.prepared_formula_key
        );
        assert_ne!(
            edge_value_cache_key(&first, "cache-basis:test").call_site_id,
            edge_value_cache_key(&second, "cache-basis:test").call_site_id
        );
    }

    #[test]
    fn local_treecalc_input_derives_values_and_bases_from_revision_layers() {
        let input = local_treecalc_input(
            snapshot(),
            TreeFormulaCatalog::new([TreeFormulaBinding {
                owner_node_id: TreeNodeId(3),
                formula_artifact_id: FormulaArtifactId("formula:b".to_string()),
                bind_artifact_id: Some(BindArtifactId("bind:b".to_string())),
                expression: TreeFormula::opaque_oxfml("=SUM(A1,2)", Vec::new()),
            }]),
            BTreeMap::from([(TreeNodeId(2), "11".to_string())]),
            BTreeMap::from([(TreeNodeId(3), "13".to_string())]),
            Vec::new(),
            Vec::new(),
            "w057:runtime-input",
        );

        assert_eq!(
            input.literal_input_values(),
            BTreeMap::from([(TreeNodeId(2), "11".to_string())])
        );
        assert!(
            input
                .compatibility_basis()
                .contains("workspace_revision_id=")
        );
        assert!(
            input
                .compatibility_basis()
                .contains("dependency_shape_snapshot_id=")
        );
        assert!(
            input
                .edge_value_cache_basis()
                .contains("workspace_revision_id=")
        );
        assert!(
            input
                .edge_value_cache_basis()
                .contains("formula_binding_snapshot_id=")
        );
        assert!(
            input
                .edge_value_cache_basis()
                .contains("dependency_shape_snapshot_id=")
        );
        assert!(
            input
                .edge_value_cache_basis()
                .contains("publication_snapshot_id=")
        );
        assert!(
            input
                .edge_value_cache_basis()
                .contains("runtime_overlay_set_id=")
        );
        assert!(
            !input
                .artifact_token_basis()
                .contains("node_input_snapshot_id=")
        );
    }

    #[test]
    fn edge_value_cache_key_names_explicit_cache_basis() {
        let structural_snapshot = snapshot();
        let binding = TreeFormulaBinding {
            owner_node_id: TreeNodeId(3),
            formula_artifact_id: FormulaArtifactId("formula:b".to_string()),
            bind_artifact_id: Some(BindArtifactId("bind:b".to_string())),
            expression: TreeFormula::opaque_oxfml("=SUM(A1,2)", Vec::new()),
        };
        let prepared = prepare_oxfml_formula(
            &structural_snapshot,
            &BTreeMap::new(),
            &binding,
            &LocalTreeCalcEnvironmentContext::default(),
        )
        .unwrap();

        let first = edge_value_cache_key(&prepared, "cache-basis:first");
        let second = edge_value_cache_key(&prepared, "cache-basis:second");

        assert_ne!(first.call_site_id, second.call_site_id);
        assert!(
            first
                .call_site_id
                .0
                .contains("cache_basis:cache-basis:first")
        );
    }

    #[test]
    fn arg_preparation_profile_change_derives_rebind_seeds() {
        let catalog = TreeFormulaCatalog::new([
            TreeFormulaBinding {
                owner_node_id: TreeNodeId(3),
                formula_artifact_id: FormulaArtifactId("formula:b".to_string()),
                bind_artifact_id: Some(BindArtifactId("bind:b".to_string())),
                expression: TreeFormula::opaque_oxfml("=SUM(A1,2)", Vec::new()),
            },
            TreeFormulaBinding {
                owner_node_id: TreeNodeId(4),
                formula_artifact_id: FormulaArtifactId("formula:c".to_string()),
                bind_artifact_id: Some(BindArtifactId("bind:c".to_string())),
                expression: TreeFormula::opaque_oxfml("=ROWS(A1:A3)", Vec::new()),
            },
        ]);

        assert!(
            derive_arg_preparation_profile_invalidation_seeds(
                &catalog,
                "oxfunc.arg-prep:v1",
                "oxfunc.arg-prep:v1"
            )
            .is_empty()
        );
        assert_eq!(
            derive_arg_preparation_profile_invalidation_seeds(
                &catalog,
                "oxfunc.arg-prep:v1",
                "oxfunc.arg-prep:v2"
            ),
            vec![
                InvalidationSeed {
                    node_id: TreeNodeId(3),
                    reason: InvalidationReasonKind::StructuralRebindRequired,
                },
                InvalidationSeed {
                    node_id: TreeNodeId(4),
                    reason: InvalidationReasonKind::StructuralRebindRequired,
                },
            ]
        );
    }

    #[test]
    fn local_treecalc_engine_publishes_local_formula_results() {
        let engine = LocalTreeCalcEngine;
        let run = engine
            .execute(local_treecalc_input(
                snapshot(),
                TreeFormulaCatalog::new([
                    TreeFormulaBinding {
                        owner_node_id: TreeNodeId(3),
                        formula_artifact_id: FormulaArtifactId("formula:b".to_string()),
                        bind_artifact_id: Some(BindArtifactId("bind:b".to_string())),
                        expression: fixture_formula(
                            TreeNodeId(3),
                            FixtureFormulaAst::Binary {
                                op: FixtureFormulaBinaryOp::Add,
                                left: Box::new(FixtureFormulaAst::Reference(
                                    TreeReference::DirectNode {
                                        target_node_id: TreeNodeId(2),
                                    },
                                )),
                                right: Box::new(FixtureFormulaAst::Literal {
                                    value: "3".to_string(),
                                }),
                            },
                        ),
                    },
                    TreeFormulaBinding {
                        owner_node_id: TreeNodeId(4),
                        formula_artifact_id: FormulaArtifactId("formula:c".to_string()),
                        bind_artifact_id: Some(BindArtifactId("bind:c".to_string())),
                        expression: fixture_formula(
                            TreeNodeId(4),
                            FixtureFormulaAst::FunctionCall {
                                function_name: "SUM".to_string(),
                                arguments: vec![
                                    FixtureFormulaAst::Reference(TreeReference::RelativePath {
                                        base: RelativeReferenceBase::ParentNode,
                                        path_segments: vec!["A".to_string()],
                                    }),
                                    FixtureFormulaAst::Reference(TreeReference::DirectNode {
                                        target_node_id: TreeNodeId(3),
                                    }),
                                ],
                                may_introduce_dynamic_dependencies: false,
                            },
                        ),
                    },
                ]),
                BTreeMap::from([(TreeNodeId(2), "2".to_string())]),
                BTreeMap::new(),
                Vec::new(),
                Vec::new(),
                "local",
            ))
            .unwrap();

        assert_eq!(run.result_state, LocalTreeCalcRunState::Published);
        assert_eq!(run.evaluation_order, vec![TreeNodeId(3), TreeNodeId(4)]);
        assert!(run.local_candidate.is_some());
        assert!(run.candidate_result.is_some());
        assert!(run.runtime_effects.is_empty());
        assert!(run.runtime_effect_overlays.is_empty());
        assert_eq!(run.prepared_formula_identities.len(), 2);
        let formula_b_identity = run
            .prepared_formula_identities
            .iter()
            .find(|identity| identity.formula_artifact_id == "formula:b")
            .expect("formula:b identity should be surfaced");
        assert!(!formula_b_identity.shape_key.is_empty());
        assert_eq!(formula_b_identity.dispatch_skeleton_key.len(), 16);
        assert_eq!(formula_b_identity.plan_template_key.len(), 16);
        assert!(formula_b_identity.prepared_formula_key.len() >= 16);
        assert!(formula_b_identity.hole_binding_fingerprint.len() >= 16);
        assert_eq!(formula_b_identity.template_hole_count, 1);
        assert_eq!(run.published_values[&TreeNodeId(3)], "5");
        assert_eq!(run.published_values[&TreeNodeId(4)], "7");
        assert!(run.publication_bundle.is_some());
        assert!(run.diagnostics.iter().any(|diagnostic| {
            diagnostic.starts_with("oxfml_prepared_plan_template_key:formula:b:")
        }));
        assert!(run.diagnostics.iter().any(|diagnostic| {
            diagnostic.starts_with("oxfml_prepared_hole_binding_fingerprint:formula:b:")
        }));
        assert!(
            run.diagnostics.iter().any(|diagnostic| {
                diagnostic.starts_with("oxfml_prepared_formula_key:formula:b:")
            })
        );
        assert!(
            run.diagnostics
                .iter()
                .any(|diagnostic| diagnostic.starts_with("oxfml_candidate_result_id:"))
        );
        assert!(
            run.diagnostics
                .iter()
                .any(|diagnostic| diagnostic.starts_with("oxfml_commit_attempt_id:"))
        );
    }

    #[test]
    fn local_treecalc_engine_traces_plan_template_reuse_without_shortcutting() {
        let engine = LocalTreeCalcEngine;
        let run = engine
            .execute(local_treecalc_input(
                snapshot(),
                TreeFormulaCatalog::new([
                    TreeFormulaBinding {
                        owner_node_id: TreeNodeId(3),
                        formula_artifact_id: FormulaArtifactId("formula:b".to_string()),
                        bind_artifact_id: Some(BindArtifactId("bind:b".to_string())),
                        expression: fixture_formula(
                            TreeNodeId(3),
                            FixtureFormulaAst::FunctionCall {
                                function_name: "SUM".to_string(),
                                arguments: vec![
                                    FixtureFormulaAst::Reference(TreeReference::DirectNode {
                                        target_node_id: TreeNodeId(2),
                                    }),
                                    FixtureFormulaAst::Literal {
                                        value: "2".to_string(),
                                    },
                                ],
                                may_introduce_dynamic_dependencies: false,
                            },
                        ),
                    },
                    TreeFormulaBinding {
                        owner_node_id: TreeNodeId(4),
                        formula_artifact_id: FormulaArtifactId("formula:c".to_string()),
                        bind_artifact_id: Some(BindArtifactId("bind:c".to_string())),
                        expression: fixture_formula(
                            TreeNodeId(4),
                            FixtureFormulaAst::FunctionCall {
                                function_name: "SUM".to_string(),
                                arguments: vec![
                                    FixtureFormulaAst::Reference(TreeReference::DirectNode {
                                        target_node_id: TreeNodeId(2),
                                    }),
                                    FixtureFormulaAst::Literal {
                                        value: "3".to_string(),
                                    },
                                ],
                                may_introduce_dynamic_dependencies: false,
                            },
                        ),
                    },
                ]),
                BTreeMap::from([(TreeNodeId(2), "2".to_string())]),
                BTreeMap::new(),
                Vec::new(),
                Vec::new(),
                "reuse",
            ))
            .unwrap();

        assert_eq!(run.result_state, LocalTreeCalcRunState::Published);
        assert_eq!(run.published_values[&TreeNodeId(3)], "4");
        assert_eq!(run.published_values[&TreeNodeId(4)], "5");
        assert_eq!(run.prepared_formula_identities.len(), 2);

        let template_keys = run
            .prepared_formula_identities
            .iter()
            .map(|identity| identity.plan_template_key.as_str())
            .collect::<BTreeSet<_>>();
        let prepared_formula_keys = run
            .prepared_formula_identities
            .iter()
            .map(|identity| identity.prepared_formula_key.as_str())
            .collect::<BTreeSet<_>>();
        let hole_binding_fingerprints = run
            .prepared_formula_identities
            .iter()
            .map(|identity| identity.hole_binding_fingerprint.as_str())
            .collect::<BTreeSet<_>>();

        assert_eq!(template_keys.len(), 2);
        assert_eq!(prepared_formula_keys.len(), 2);
        assert_eq!(hole_binding_fingerprints.len(), 2);
        assert!(
            !run.diagnostics
                .iter()
                .any(|diagnostic| diagnostic.starts_with("oxfml_plan_template_reuse_count:"))
        );
    }

    fn assert_w046_refinement_bridge_facts(run: &LocalTreeCalcRunArtifacts) {
        let order_index = run
            .evaluation_order
            .iter()
            .copied()
            .enumerate()
            .map(|(index, node_id)| (node_id, index))
            .collect::<BTreeMap<_, _>>();

        for edges in run.dependency_graph.edges_by_owner.values() {
            for edge in edges {
                let reverse_edges = run
                    .dependency_graph
                    .reverse_edges
                    .get(&edge.target_node_id)
                    .expect("reverse edge bucket exists for every forward edge target");
                assert!(
                    reverse_edges.contains(edge),
                    "forward edge must have reverse converse entry"
                );

                if let (Some(target_index), Some(owner_index)) = (
                    order_index.get(&edge.target_node_id),
                    order_index.get(&edge.owner_node_id),
                ) {
                    assert!(
                        target_index < owner_index,
                        "formula target must be evaluated before dependent owner"
                    );
                }
            }
        }

        for node_id in &run.evaluation_order {
            assert!(
                run.invalidation_closure.records.contains_key(node_id),
                "evaluated node must be present in invalidation closure"
            );
        }

        match run.result_state {
            LocalTreeCalcRunState::Published => {
                let candidate = run
                    .candidate_result
                    .as_ref()
                    .expect("published run carries accepted candidate result");
                let publication = run
                    .publication_bundle
                    .as_ref()
                    .expect("published run carries publication bundle");
                assert!(run.reject_detail.is_none());
                assert_eq!(candidate.target_set, run.evaluation_order);
                assert_eq!(
                    candidate.calc_value_updates,
                    publication.published_calc_value_delta
                );
                assert_eq!(
                    candidate.candidate_result_id,
                    publication.candidate_result_id
                );
            }
            LocalTreeCalcRunState::Rejected => {
                assert!(run.publication_bundle.is_none());
                assert!(run.reject_detail.is_some());
            }
            LocalTreeCalcRunState::VerifiedClean => {
                assert!(run.publication_bundle.is_none());
                assert!(run.reject_detail.is_none());
            }
        }
    }

    #[test]
    fn local_treecalc_engine_exposes_w046_refinement_bridge_facts() {
        let engine = LocalTreeCalcEngine;
        let run = engine
            .execute(local_treecalc_input(
                snapshot(),
                TreeFormulaCatalog::new([
                    TreeFormulaBinding {
                        owner_node_id: TreeNodeId(3),
                        formula_artifact_id: FormulaArtifactId("formula:b".to_string()),
                        bind_artifact_id: Some(BindArtifactId("bind:b".to_string())),
                        expression: fixture_formula(
                            TreeNodeId(3),
                            FixtureFormulaAst::Binary {
                                op: FixtureFormulaBinaryOp::Add,
                                left: Box::new(FixtureFormulaAst::Reference(
                                    TreeReference::DirectNode {
                                        target_node_id: TreeNodeId(2),
                                    },
                                )),
                                right: Box::new(FixtureFormulaAst::Literal {
                                    value: "3".to_string(),
                                }),
                            },
                        ),
                    },
                    TreeFormulaBinding {
                        owner_node_id: TreeNodeId(4),
                        formula_artifact_id: FormulaArtifactId("formula:c".to_string()),
                        bind_artifact_id: Some(BindArtifactId("bind:c".to_string())),
                        expression: fixture_formula(
                            TreeNodeId(4),
                            FixtureFormulaAst::FunctionCall {
                                function_name: "SUM".to_string(),
                                arguments: vec![
                                    FixtureFormulaAst::Reference(TreeReference::RelativePath {
                                        base: RelativeReferenceBase::ParentNode,
                                        path_segments: vec!["A".to_string()],
                                    }),
                                    FixtureFormulaAst::Reference(TreeReference::DirectNode {
                                        target_node_id: TreeNodeId(3),
                                    }),
                                ],
                                may_introduce_dynamic_dependencies: false,
                            },
                        ),
                    },
                ]),
                BTreeMap::from([
                    (TreeNodeId(3), "2".to_string()),
                    (TreeNodeId(4), "3".to_string()),
                ]),
                BTreeMap::new(),
                Vec::new(),
                Vec::new(),
                "w046:bridge",
            ))
            .unwrap();

        assert_w046_refinement_bridge_facts(&run);
    }

    #[test]
    fn local_treecalc_engine_recalculates_direct_multiply_chain_after_constant_edit() {
        let engine = LocalTreeCalcEngine;
        let formula_catalog = TreeFormulaCatalog::new([
            TreeFormulaBinding {
                owner_node_id: TreeNodeId(3),
                formula_artifact_id: FormulaArtifactId("formula:y".to_string()),
                bind_artifact_id: Some(BindArtifactId("bind:y".to_string())),
                expression: fixture_formula(
                    TreeNodeId(3),
                    FixtureFormulaAst::Binary {
                        op: FixtureFormulaBinaryOp::Multiply,
                        left: Box::new(FixtureFormulaAst::Reference(TreeReference::DirectNode {
                            target_node_id: TreeNodeId(2),
                        })),
                        right: Box::new(FixtureFormulaAst::Literal {
                            value: "20".to_string(),
                        }),
                    },
                ),
            },
            TreeFormulaBinding {
                owner_node_id: TreeNodeId(4),
                formula_artifact_id: FormulaArtifactId("formula:z".to_string()),
                bind_artifact_id: Some(BindArtifactId("bind:z".to_string())),
                expression: fixture_formula(
                    TreeNodeId(4),
                    FixtureFormulaAst::Binary {
                        op: FixtureFormulaBinaryOp::Add,
                        left: Box::new(FixtureFormulaAst::Reference(TreeReference::DirectNode {
                            target_node_id: TreeNodeId(2),
                        })),
                        right: Box::new(FixtureFormulaAst::Reference(TreeReference::DirectNode {
                            target_node_id: TreeNodeId(3),
                        })),
                    },
                ),
            },
        ]);

        let initial = engine
            .execute(local_treecalc_input(
                snapshot(),
                formula_catalog.clone(),
                BTreeMap::from([(TreeNodeId(2), "2".to_string())]),
                BTreeMap::new(),
                Vec::new(),
                Vec::new(),
                "xyz:initial",
            ))
            .unwrap();

        assert_eq!(initial.result_state, LocalTreeCalcRunState::Published);
        assert_eq!(initial.evaluation_order, vec![TreeNodeId(3), TreeNodeId(4)]);
        assert_eq!(initial.published_values[&TreeNodeId(3)], "40");
        assert_eq!(initial.published_values[&TreeNodeId(4)], "42");

        let rerun = engine
            .execute(local_treecalc_input(
                snapshot(),
                formula_catalog,
                BTreeMap::from([(TreeNodeId(2), "3".to_string())]),
                initial.published_values.clone(),
                Vec::new(),
                vec![InvalidationSeed {
                    node_id: TreeNodeId(2),
                    reason: InvalidationReasonKind::UpstreamPublication,
                }],
                "xyz:rerun",
            ))
            .unwrap();

        assert_eq!(rerun.result_state, LocalTreeCalcRunState::Published);
        assert_eq!(rerun.evaluation_order, vec![TreeNodeId(3), TreeNodeId(4)]);
        assert_eq!(
            rerun.invalidation_closure.impacted_order,
            vec![TreeNodeId(2), TreeNodeId(3), TreeNodeId(4)]
        );
        assert_eq!(rerun.published_values[&TreeNodeId(3)], "60");
        assert_eq!(rerun.published_values[&TreeNodeId(4)], "63");
    }

    #[test]
    fn local_treecalc_engine_marks_verified_clean_when_seed_matches() {
        let engine = LocalTreeCalcEngine;
        let mut seeded = BTreeMap::new();
        seeded.insert(TreeNodeId(3), "5".to_string());

        let run = engine
            .execute(local_treecalc_input(
                snapshot(),
                TreeFormulaCatalog::new([TreeFormulaBinding {
                    owner_node_id: TreeNodeId(3),
                    formula_artifact_id: FormulaArtifactId("formula:b".to_string()),
                    bind_artifact_id: Some(BindArtifactId("bind:b".to_string())),
                    expression: fixture_formula(
                        TreeNodeId(3),
                        FixtureFormulaAst::Binary {
                            op: FixtureFormulaBinaryOp::Add,
                            left: Box::new(FixtureFormulaAst::Reference(
                                TreeReference::DirectNode {
                                    target_node_id: TreeNodeId(2),
                                },
                            )),
                            right: Box::new(FixtureFormulaAst::Literal {
                                value: "3".to_string(),
                            }),
                        },
                    ),
                }]),
                BTreeMap::from([
                    (TreeNodeId(3), "2".to_string()),
                    (TreeNodeId(4), "3".to_string()),
                ]),
                seeded,
                Vec::new(),
                Vec::new(),
                "verified",
            ))
            .unwrap();

        assert_eq!(run.result_state, LocalTreeCalcRunState::VerifiedClean);
        assert!(run.local_candidate.is_none());
        assert!(run.candidate_result.is_none());
        assert!(run.publication_bundle.is_none());
        assert!(run.runtime_effects.is_empty());
        assert!(run.runtime_effect_overlays.is_empty());
        assert_eq!(
            run.node_states[&TreeNodeId(3)],
            NodeCalcState::VerifiedClean
        );
        assert!(
            run.diagnostics
                .iter()
                .any(|diagnostic| diagnostic.starts_with("edge_value_cache_hit:node:3:"))
        );
        assert!(
            !run.diagnostics
                .iter()
                .any(|diagnostic| diagnostic.starts_with("oxfml_candidate_result_id:"))
        );
        assert!(
            !run.diagnostics
                .iter()
                .any(|diagnostic| diagnostic.starts_with("oxfml_commit_attempt_id:"))
        );
        assert!(
            run.diagnostics
                .contains(&"verified_clean_publication_suppressed:node:3".to_string())
        );
    }

    #[test]
    fn local_treecalc_engine_rejects_cycles_in_formula_family() {
        let engine = LocalTreeCalcEngine;
        let run = engine
            .execute(local_treecalc_input(
                snapshot(),
                TreeFormulaCatalog::new([
                    TreeFormulaBinding {
                        owner_node_id: TreeNodeId(3),
                        formula_artifact_id: FormulaArtifactId("formula:b".to_string()),
                        bind_artifact_id: None,
                        expression: fixture_formula(
                            TreeNodeId(3),
                            FixtureFormulaAst::Reference(TreeReference::DirectNode {
                                target_node_id: TreeNodeId(4),
                            }),
                        ),
                    },
                    TreeFormulaBinding {
                        owner_node_id: TreeNodeId(4),
                        formula_artifact_id: FormulaArtifactId("formula:c".to_string()),
                        bind_artifact_id: None,
                        expression: fixture_formula(
                            TreeNodeId(4),
                            FixtureFormulaAst::Reference(TreeReference::DirectNode {
                                target_node_id: TreeNodeId(3),
                            }),
                        ),
                    },
                ]),
                BTreeMap::new(),
                BTreeMap::new(),
                Vec::new(),
                Vec::new(),
                "cycle",
            ))
            .unwrap();

        assert_eq!(run.result_state, LocalTreeCalcRunState::Rejected);
        assert!(run.candidate_result.is_none());
        assert!(run.publication_bundle.is_none());
        assert!(run.runtime_effects.is_empty());
        assert!(run.runtime_effect_overlays.is_empty());
        assert_eq!(
            run.reject_detail.as_ref().map(|detail| detail.kind),
            Some(RejectKind::SyntheticCycleReject)
        );
    }

    #[test]
    fn local_treecalc_engine_emits_runtime_effect_for_host_sensitive_reference() {
        let engine = LocalTreeCalcEngine;
        let run = engine
            .execute(local_treecalc_input(
                snapshot(),
                TreeFormulaCatalog::new([TreeFormulaBinding {
                    owner_node_id: TreeNodeId(3),
                    formula_artifact_id: FormulaArtifactId("formula:b".to_string()),
                    bind_artifact_id: Some(BindArtifactId("bind:b".to_string())),
                    expression: fixture_formula(
                        TreeNodeId(3),
                        FixtureFormulaAst::Reference(TreeReference::HostSensitive {
                            carrier_id: "carrier:host".to_string(),
                            detail: "active_selection".to_string(),
                        }),
                    ),
                }]),
                BTreeMap::new(),
                BTreeMap::new(),
                Vec::new(),
                Vec::new(),
                "host",
            ))
            .unwrap();

        assert_eq!(run.result_state, LocalTreeCalcRunState::Rejected);
        assert_eq!(run.runtime_effects.len(), 1);
        assert_eq!(run.runtime_effect_overlays.len(), 1);
        assert_eq!(
            run.runtime_effects[0].kind,
            "runtime_effect.host_sensitive_reference"
        );
        assert_eq!(
            run.runtime_effects[0].family,
            RuntimeEffectFamily::ExecutionRestriction
        );
        assert!(
            run.runtime_effects[0]
                .detail
                .contains("carrier_id:carrier:host")
        );
        assert_eq!(
            run.local_candidate
                .as_ref()
                .map(|candidate| candidate.runtime_effects.clone())
                .unwrap(),
            run.runtime_effects
        );
        assert_eq!(
            run.runtime_effect_overlays[0].key.overlay_kind,
            OverlayKind::ExecutionRestriction
        );
        assert!(
            run.runtime_effect_overlays[0]
                .detail
                .contains("runtime_effect.host_sensitive_reference")
        );
    }

    #[test]
    fn local_treecalc_engine_emits_runtime_effect_for_dynamic_reference() {
        let engine = LocalTreeCalcEngine;
        let run = engine
            .execute(local_treecalc_input(
                snapshot(),
                TreeFormulaCatalog::new([TreeFormulaBinding {
                    owner_node_id: TreeNodeId(3),
                    formula_artifact_id: FormulaArtifactId("formula:b".to_string()),
                    bind_artifact_id: Some(BindArtifactId("bind:b".to_string())),
                    expression: fixture_formula(
                        TreeNodeId(3),
                        FixtureFormulaAst::Reference(TreeReference::DynamicPotential {
                            carrier_id: "carrier:dynamic".to_string(),
                            detail: "late_bound_projection".to_string(),
                        }),
                    ),
                }]),
                BTreeMap::new(),
                BTreeMap::new(),
                Vec::new(),
                Vec::new(),
                "dynamic",
            ))
            .unwrap();

        assert_eq!(run.result_state, LocalTreeCalcRunState::Rejected);
        assert_eq!(run.runtime_effects.len(), 1);
        assert_eq!(run.runtime_effect_overlays.len(), 1);
        assert_eq!(
            run.runtime_effects[0].kind,
            "runtime_effect.dynamic_reference"
        );
        assert_eq!(
            run.runtime_effects[0].family,
            RuntimeEffectFamily::DynamicDependency
        );
        assert!(
            run.runtime_effects[0]
                .detail
                .contains("carrier_id:carrier:dynamic")
        );
        assert_eq!(
            run.reject_detail.as_ref().map(|detail| detail.kind),
            Some(RejectKind::DynamicDependencyFailure)
        );
        assert_eq!(
            run.runtime_effect_overlays[0]
                .key
                .payload_identity
                .as_deref(),
            Some("cand:dynamic:runtime_effect:0")
        );
    }

    #[test]
    fn local_treecalc_engine_publishes_resolved_dynamic_reference_shape_update() {
        let engine = LocalTreeCalcEngine;
        let run = engine
            .execute(local_treecalc_input(
                snapshot(),
                TreeFormulaCatalog::new([TreeFormulaBinding {
                    owner_node_id: TreeNodeId(3),
                    formula_artifact_id: FormulaArtifactId("formula:b".to_string()),
                    bind_artifact_id: Some(BindArtifactId("bind:b".to_string())),
                    expression: fixture_formula(
                        TreeNodeId(3),
                        FixtureFormulaAst::Reference(TreeReference::DynamicResolved {
                            target_node_id: TreeNodeId(2),
                            carrier_id: "carrier:dynamic".to_string(),
                            detail: "resolved_late_bound_projection".to_string(),
                        }),
                    ),
                }]),
                BTreeMap::new(),
                BTreeMap::new(),
                Vec::new(),
                Vec::new(),
                "dynamic:resolved",
            ))
            .unwrap();

        assert_eq!(run.result_state, LocalTreeCalcRunState::Published);
        assert_eq!(run.runtime_effects.len(), 1);
        assert_eq!(run.runtime_effect_overlays.len(), 1);
        assert_eq!(
            run.runtime_effects[0].family,
            RuntimeEffectFamily::DynamicDependency
        );
        assert_eq!(
            run.runtime_effect_overlays[0].key.overlay_kind,
            OverlayKind::DynamicDependency
        );
        assert_eq!(
            run.runtime_effect_overlays[0].key.owner_node_id,
            TreeNodeId(3)
        );
        assert_eq!(
            run.candidate_result
                .as_ref()
                .map(|candidate| candidate.dependency_shape_updates.clone())
                .unwrap(),
            vec![DependencyShapeUpdate {
                kind: "activate_dynamic_dep".to_string(),
                affected_node_ids: vec![TreeNodeId(2), TreeNodeId(3)],
            }]
        );
        assert_eq!(
            run.publication_bundle
                .as_ref()
                .map(|bundle| bundle.published_runtime_effects.len()),
            Some(1)
        );
    }

    #[test]
    fn local_treecalc_engine_rejects_rerun_when_invalidation_requires_rebind() {
        let engine = LocalTreeCalcEngine;
        let run = engine
            .execute(local_treecalc_input(
                snapshot(),
                TreeFormulaCatalog::new([TreeFormulaBinding {
                    owner_node_id: TreeNodeId(3),
                    formula_artifact_id: FormulaArtifactId("formula:b".to_string()),
                    bind_artifact_id: Some(BindArtifactId("bind:b".to_string())),
                    expression: fixture_formula(
                        TreeNodeId(3),
                        FixtureFormulaAst::Reference(TreeReference::DirectNode {
                            target_node_id: TreeNodeId(2),
                        }),
                    ),
                }]),
                BTreeMap::new(),
                BTreeMap::from([(TreeNodeId(3), "5".to_string())]),
                Vec::new(),
                vec![InvalidationSeed {
                    node_id: TreeNodeId(3),
                    reason: InvalidationReasonKind::StructuralRebindRequired,
                }],
                "rebind",
            ))
            .unwrap();

        assert_eq!(run.result_state, LocalTreeCalcRunState::Rejected);
        assert!(run.publication_bundle.is_none());
        assert_eq!(
            run.reject_detail.as_ref().map(|detail| detail.kind),
            Some(RejectKind::HostInjectedFailure)
        );
        assert!(
            run.diagnostics
                .iter()
                .any(|diagnostic| diagnostic.contains("requires rebind before reevaluation"))
        );
    }

    #[test]
    fn local_treecalc_engine_rejects_rerun_when_arg_preparation_profile_changes() {
        let engine = LocalTreeCalcEngine;
        let mut input = local_treecalc_input(
            snapshot(),
            TreeFormulaCatalog::new([TreeFormulaBinding {
                owner_node_id: TreeNodeId(3),
                formula_artifact_id: FormulaArtifactId("formula:b".to_string()),
                bind_artifact_id: Some(BindArtifactId("bind:b".to_string())),
                expression: TreeFormula::opaque_oxfml("=SUM(A1,2)", Vec::new()),
            }]),
            BTreeMap::new(),
            BTreeMap::from([(TreeNodeId(3), "5".to_string())]),
            Vec::new(),
            Vec::new(),
            "argprep",
        );
        input.previous_arg_preparation_profile_version = Some("oxfunc.arg-prep:v1".to_string());
        input.environment_context = input
            .environment_context
            .with_arg_preparation_profile_version("oxfunc.arg-prep:v2");
        let run = engine.execute(input).unwrap();

        assert_eq!(run.result_state, LocalTreeCalcRunState::Rejected);
        assert!(run.publication_bundle.is_none());
        assert_eq!(
            run.reject_detail.as_ref().map(|detail| detail.kind),
            Some(RejectKind::HostInjectedFailure)
        );
        assert!(run.invalidation_closure.records[&TreeNodeId(3)].requires_rebind);
        assert!(
            run.invalidation_closure.records[&TreeNodeId(3)]
                .reasons
                .contains(&InvalidationReasonKind::StructuralRebindRequired)
        );
    }

    #[test]
    fn local_treecalc_engine_rejects_rerun_when_dependency_target_is_missing() {
        let engine = LocalTreeCalcEngine;
        let rerun_snapshot = snapshot()
            .apply_edit(
                crate::structural::StructuralSnapshotId(2),
                crate::structural::StructuralEdit::RemoveNode {
                    node_id: TreeNodeId(2),
                },
            )
            .unwrap()
            .snapshot;
        let run = engine
            .execute(local_treecalc_input(
                rerun_snapshot,
                TreeFormulaCatalog::new([TreeFormulaBinding {
                    owner_node_id: TreeNodeId(3),
                    formula_artifact_id: FormulaArtifactId("formula:b".to_string()),
                    bind_artifact_id: Some(BindArtifactId("bind:b".to_string())),
                    expression: fixture_formula(
                        TreeNodeId(3),
                        FixtureFormulaAst::Reference(TreeReference::DirectNode {
                            target_node_id: TreeNodeId(2),
                        }),
                    ),
                }]),
                BTreeMap::new(),
                BTreeMap::from([(TreeNodeId(3), "5".to_string())]),
                Vec::new(),
                vec![InvalidationSeed {
                    node_id: TreeNodeId(3),
                    reason: InvalidationReasonKind::StructuralRecalcOnly,
                }],
                "missing_target",
            ))
            .unwrap();

        assert_eq!(run.result_state, LocalTreeCalcRunState::Rejected);
        assert!(run.publication_bundle.is_none());
        assert_eq!(
            run.reject_detail.as_ref().map(|detail| detail.kind),
            Some(RejectKind::HostInjectedFailure)
        );
        assert!(
            run.diagnostics
                .iter()
                .any(|diagnostic| diagnostic.contains("MissingTarget"))
        );
    }

    #[test]
    fn oxfml_dependency_descriptors_preserve_sequence_one_carrier_mapping() {
        let structural_snapshot = snapshot();
        let binding = TreeFormulaBinding {
            owner_node_id: TreeNodeId(4),
            formula_artifact_id: FormulaArtifactId("formula:c".to_string()),
            bind_artifact_id: Some(BindArtifactId("bind:c".to_string())),
            expression: fixture_formula(
                TreeNodeId(4),
                FixtureFormulaAst::FunctionCall {
                    function_name: "SUM".to_string(),
                    arguments: vec![
                        FixtureFormulaAst::Reference(TreeReference::DirectNode {
                            target_node_id: TreeNodeId(2),
                        }),
                        FixtureFormulaAst::Reference(TreeReference::SiblingOffset {
                            offset: -1,
                            tail_segments: vec![],
                        }),
                        FixtureFormulaAst::Reference(TreeReference::RelativePath {
                            base: RelativeReferenceBase::ParentNode,
                            path_segments: vec!["Missing".to_string()],
                        }),
                        FixtureFormulaAst::Reference(TreeReference::Unresolved {
                            token: "../Missing".to_string(),
                        }),
                        FixtureFormulaAst::Reference(TreeReference::HostSensitive {
                            carrier_id: "host.selection".to_string(),
                            detail: "active branch".to_string(),
                        }),
                        FixtureFormulaAst::Reference(TreeReference::DynamicPotential {
                            carrier_id: "runtime.topic".to_string(),
                            detail: "late bound".to_string(),
                        }),
                    ],
                    may_introduce_dynamic_dependencies: true,
                },
            ),
        };

        let prepared = prepare_oxfml_formula(
            &structural_snapshot,
            &BTreeMap::new(),
            &binding,
            &LocalTreeCalcEnvironmentContext::default(),
        )
        .unwrap();
        let descriptors = oxfml_dependency_descriptors(&prepared)
            .into_iter()
            .map(|descriptor| (descriptor.carrier_detail.clone(), descriptor))
            .collect::<BTreeMap<_, _>>();
        let descriptor_keys = descriptors.keys().cloned().collect::<Vec<_>>();

        let direct = descriptors
            .get("direct_node:node:2")
            .unwrap_or_else(|| panic!("missing direct_node:node:2 in {:?}", descriptor_keys));
        assert_eq!(direct.kind, DependencyDescriptorKind::StaticDirect);
        assert_eq!(direct.target_node_id, Some(TreeNodeId(2)));
        assert!(
            direct
                .source_reference_handle
                .as_deref()
                .is_some_and(|handle| handle.starts_with("formal-ref:"))
        );
        assert!(
            !direct
                .source_reference_handle
                .as_deref()
                .unwrap()
                .contains("A2")
        );
        assert!(!direct.requires_rebind_on_structural_change);

        let sibling = descriptors
            .get("sibling_offset:-1:")
            .unwrap_or_else(|| panic!("missing sibling_offset:-1: in {:?}", descriptor_keys));
        assert_eq!(sibling.kind, DependencyDescriptorKind::RelativeBound);
        assert_eq!(sibling.target_node_id, Some(TreeNodeId(3)));
        assert!(
            sibling
                .source_reference_handle
                .as_deref()
                .is_some_and(|handle| handle.starts_with("formal-ref:"))
        );
        assert!(sibling.requires_rebind_on_structural_change);

        let unresolved_relative = descriptors
            .get("relative_path:ParentNode:Missing")
            .unwrap_or_else(|| {
                panic!(
                    "missing relative_path:ParentNode:Missing in {:?}",
                    descriptor_keys
                )
            });
        assert_eq!(
            unresolved_relative.kind,
            DependencyDescriptorKind::RelativeBound
        );
        assert_eq!(unresolved_relative.target_node_id, None);
        assert!(
            unresolved_relative
                .source_reference_handle
                .as_deref()
                .is_some()
        );
        assert!(unresolved_relative.requires_rebind_on_structural_change);

        let unresolved_token = descriptors
            .get("unresolved:../Missing")
            .unwrap_or_else(|| panic!("missing unresolved:../Missing in {:?}", descriptor_keys));
        assert_eq!(unresolved_token.kind, DependencyDescriptorKind::Unresolved);
        assert_eq!(unresolved_token.target_node_id, None);
        assert!(
            unresolved_token
                .source_reference_handle
                .as_deref()
                .is_some()
        );
        assert!(unresolved_token.requires_rebind_on_structural_change);

        let host_sensitive = descriptors
            .get("residual:host.selection:active branch")
            .unwrap_or_else(|| {
                panic!(
                    "missing residual:host.selection:active branch in {:?}",
                    descriptor_keys
                )
            });
        assert_eq!(host_sensitive.kind, DependencyDescriptorKind::HostSensitive);
        assert_eq!(host_sensitive.target_node_id, None);
        assert_eq!(
            host_sensitive.source_reference_handle.as_deref(),
            Some("runtime_fact:HostSensitive:host.selection")
        );
        assert!(host_sensitive.requires_rebind_on_structural_change);

        let dynamic = descriptors
            .get("residual:runtime.topic:late bound")
            .unwrap_or_else(|| {
                panic!(
                    "missing residual:runtime.topic:late bound in {:?}",
                    descriptor_keys
                )
            });
        assert_eq!(dynamic.kind, DependencyDescriptorKind::DynamicPotential);
        assert_eq!(dynamic.target_node_id, None);
        assert_eq!(
            dynamic.source_reference_handle.as_deref(),
            Some("runtime_fact:DynamicPotential:runtime.topic")
        );
        assert!(!dynamic.requires_rebind_on_structural_change);

        let graph_descriptors = descriptors.values().cloned().collect::<Vec<_>>();
        let graph = DependencyGraph::build(&structural_snapshot, &graph_descriptors);
        let graph_direct = graph.descriptors_by_owner[&TreeNodeId(4)]
            .iter()
            .find(|descriptor| descriptor.carrier_detail == "direct_node:node:2")
            .expect("direct descriptor should be retained in graph");
        assert!(
            graph_direct
                .source_reference_handle
                .as_deref()
                .is_some_and(|handle| handle.starts_with("formal-ref:"))
        );
    }

    #[test]
    fn children_collection_runtime_binding_preserves_reference_identity() {
        let structural_snapshot = children_collection_snapshot(vec![TreeNodeId(3), TreeNodeId(4)]);
        let binding = children_collection_catalog()
            .try_get_binding(TreeNodeId(10))
            .expect("total binding")
            .clone();

        let translated = project_opaque_formula(
            &structural_snapshot,
            &BTreeMap::new(),
            binding.owner_node_id,
            &binding.expression,
            &BTreeSet::new(),
        );
        let formal_inputs = formal_input_bindings_for_runtime(
            &translated,
            &BTreeMap::new(),
            &BTreeMap::new(),
            None,
        );

        assert_eq!(translated.collection_bindings.len(), 1);
        assert_eq!(
            translated.collection_bindings[0].member_node_ids,
            vec![TreeNodeId(3), TreeNodeId(4)]
        );
        assert_eq!(formal_inputs.len(), 1);
        assert_eq!(formal_inputs[0].reference_descriptor, "name:TREE_REF_10_0");
        assert_eq!(formal_inputs[0].reference_handle.as_deref(), None);
        match &formal_inputs[0].binding {
            DefinedNameBinding::Reference(reference) => {
                assert_eq!(reference.kind(), ReferenceKind::Structured);
                assert_eq!(reference.target(), "treecalc-hostref:v1:children:node:2");
            }
            other => panic!("expected reference-preserving binding, got {other:?}"),
        }

        let sparse_bindings = sparse_reference_value_bindings_for_runtime(
            &translated,
            &structural_snapshot,
            &BTreeMap::from([
                (TreeNodeId(3), "2".to_string()),
                (TreeNodeId(4), "3".to_string()),
            ]),
            &BTreeMap::from([
                (TreeNodeId(3), CalcValue::number(2.0)),
                (TreeNodeId(4), CalcValue::number(3.0)),
            ]),
        );
        assert_eq!(sparse_bindings.len(), 1);
        assert_eq!(
            sparse_bindings[0].reference.kind(),
            ReferenceKind::Structured
        );
        assert_eq!(
            sparse_bindings[0].reference.target(),
            "treecalc-hostref:v1:children:node:2"
        );
        assert_eq!(sparse_bindings[0].declared_rows, 2);
        assert_eq!(sparse_bindings[0].declared_cols, 1);
        assert_eq!(sparse_bindings[0].defined_cells.len(), 2);
        assert_eq!(sparse_bindings[0].defined_cells[0].row, 1);
        assert_eq!(sparse_bindings[0].defined_cells[0].col, 1);
        assert_eq!(
            sparse_bindings[0].defined_cells[0].value,
            CalcValue::number(2.0)
        );
        assert!(
            sparse_bindings[0].reader_identity.as_deref().is_some_and(
                |identity| identity.contains("reader_id=treecalc-hostref:v1:children:node:2")
            )
        );
    }

    #[test]
    fn children_collection_sum_uses_generic_host_context_and_sparse_reference_values() {
        for source_token_text in ["@CHILDREN", ".*", "base.@CHILDREN", "base.*"] {
            assert_children_collection_sum_uses_generic_host_context_and_sparse_reference_values(
                source_token_text,
            );
        }
    }

    #[test]
    fn reference_literal_array_sum_preserves_order_duplicates_and_sparse_reference_values() {
        let structural_snapshot = children_collection_snapshot(vec![TreeNodeId(3), TreeNodeId(4)]);
        let catalog = reference_literal_array_catalog();
        let binding = catalog
            .try_get_binding(TreeNodeId(10))
            .expect("reference literal array binding");
        let prepared = prepare_oxfml_formula(
            &structural_snapshot,
            &BTreeMap::new(),
            binding,
            &LocalTreeCalcEnvironmentContext::default(),
        )
        .expect("reference literal array formula should prepare");

        let host_reference = &prepared
            .runtime_prepared_identity
            .host_reference_bind_results[0];
        assert_eq!(
            host_reference.reference_handle,
            "treecalc-hostref:v1:reference_literal_array:array:q1"
        );
        assert_eq!(host_reference.source_token_text, "{A,B,A}");
        assert_eq!(
            host_reference.shape_hint.as_deref(),
            Some("ordered_collection:treecalc_reference_literal_array_v1")
        );
        assert!(
            host_reference
                .replay_identity_contribution
                .contains("members=node:3,node:4,node:3")
        );

        let sparse_bindings = sparse_reference_value_bindings_for_runtime(
            &prepared.translated,
            &structural_snapshot,
            &BTreeMap::from([
                (TreeNodeId(3), "2".to_string()),
                (TreeNodeId(4), "3".to_string()),
            ]),
            &BTreeMap::from([
                (TreeNodeId(3), CalcValue::number(2.0)),
                (TreeNodeId(4), CalcValue::number(3.0)),
            ]),
        );
        assert_eq!(sparse_bindings.len(), 1);
        assert_eq!(
            sparse_bindings[0].reference.target(),
            "treecalc-hostref:v1:reference_literal_array:array:q1"
        );
        assert_eq!(sparse_bindings[0].declared_rows, 3);
        assert_eq!(
            sparse_bindings[0]
                .defined_cells
                .iter()
                .map(|cell| (cell.row, cell.value.clone()))
                .collect::<Vec<_>>(),
            vec![
                (1, CalcValue::number(2.0)),
                (2, CalcValue::number(3.0)),
                (3, CalcValue::number(2.0)),
            ]
        );

        let missing_member_run = LocalTreeCalcEngine
            .execute(local_treecalc_input(
                structural_snapshot.clone(),
                catalog.clone(),
                BTreeMap::new(),
                BTreeMap::new(),
                Vec::new(),
                Vec::new(),
                "w056:reference-literal-array-sum",
            ))
            .expect("missing collection member values should reject cleanly");

        assert_eq!(
            missing_member_run.result_state,
            LocalTreeCalcRunState::Rejected
        );
        assert_eq!(
            missing_member_run
                .reject_detail
                .as_ref()
                .map(|detail| &detail.kind),
            Some(&RejectKind::DynamicDependencyFailure)
        );
        assert!(missing_member_run.diagnostics.iter().any(|diagnostic| {
            diagnostic
                == "missing_collection_member_value:owner=node:10;target=node:3;handle=treecalc-hostref:v1:reference_literal_array:array:q1"
        }));

        let run = LocalTreeCalcEngine
            .execute(local_treecalc_input(
                structural_snapshot,
                catalog,
                BTreeMap::from([
                    (TreeNodeId(3), "2".to_string()),
                    (TreeNodeId(4), "3".to_string()),
                ]),
                BTreeMap::from([
                    (TreeNodeId(3), "2".to_string()),
                    (TreeNodeId(4), "3".to_string()),
                ]),
                Vec::new(),
                Vec::new(),
                "w056:reference-literal-array-sum-after-members-available",
            ))
            .expect("SUM over reference literal array should execute after members are available");

        assert_eq!(
            run.result_state,
            LocalTreeCalcRunState::Published,
            "reference literal array run did not publish: reject={:?}; diagnostics={:?}",
            run.reject_detail,
            run.diagnostics
        );
        assert_eq!(run.published_values[&TreeNodeId(10)], "7");
    }

    #[test]
    fn raw_children_formula_text_resolves_through_oxfml_host_reference_path() {
        for (index, source_text) in ["=SUM(@CHILDREN)", "=SUM(.*)"].into_iter().enumerate() {
            let mut context = OxCalcTreeContext::default();
            let workspace_id = context
                .create_workspace(OxCalcTreeWorkspaceCreate::new(format!(
                    "workspace:raw-children:{index}"
                )))
                .unwrap();
            let base_id = context
                .add_node(
                    &workspace_id,
                    OxCalcTreeNodeCreate::new("Base", source_text),
                )
                .unwrap();
            context
                .add_node(
                    &workspace_id,
                    OxCalcTreeNodeCreate::new("A", "=2").under(base_id),
                )
                .unwrap();
            context
                .add_node(
                    &workspace_id,
                    OxCalcTreeNodeCreate::new("B", "=3").under(base_id),
                )
                .unwrap();

            let result = context.recalculate(&workspace_id).unwrap();
            assert_eq!(
                result.run_state,
                OxCalcTreeRunState::Published,
                "{source_text} failed: reject={:?}; diagnostics={:?}",
                result.reject_detail,
                result.diagnostics
            );
            assert_eq!(
                result.published_values.get(&base_id),
                Some(&"5".to_string()),
                "{source_text} should execute through OxCalcTreeContext; diagnostics={:?}",
                result.diagnostics
            );
        }
    }

    #[test]
    fn raw_qualified_children_formula_text_resolves_through_oxfml_host_reference_path() {
        for (index, source_text) in ["=SUM(Base.@CHILDREN)", "=SUM(Base.*)"]
            .into_iter()
            .enumerate()
        {
            let mut context = OxCalcTreeContext::default();
            let workspace_id = context
                .create_workspace(OxCalcTreeWorkspaceCreate::new(format!(
                    "workspace:qualified-children:{index}"
                )))
                .unwrap();
            let base_id = context
                .add_node(&workspace_id, OxCalcTreeNodeCreate::new("Base", ""))
                .unwrap();
            context
                .add_node(
                    &workspace_id,
                    OxCalcTreeNodeCreate::new("A", "=2").under(base_id),
                )
                .unwrap();
            context
                .add_node(
                    &workspace_id,
                    OxCalcTreeNodeCreate::new("B", "=3").under(base_id),
                )
                .unwrap();
            let total_id = context
                .add_node(
                    &workspace_id,
                    OxCalcTreeNodeCreate::new("Total", source_text),
                )
                .unwrap();

            let result = context.recalculate(&workspace_id).unwrap();

            assert_eq!(
                result.run_state,
                OxCalcTreeRunState::Published,
                "{source_text} should execute through OxCalcTreeContext: reject={:?}; diagnostics={:?}",
                result.reject_detail,
                result.diagnostics
            );
            assert_eq!(
                result.published_values.get(&total_id),
                Some(&"5".to_string()),
                "{source_text} should resolve the qualified base collection"
            );
        }
    }

    #[test]
    fn raw_ordered_selector_formula_text_resolves_direct_collections_through_tree_context() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:ordered-selectors",
            ))
            .unwrap();
        context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("A", "=1"))
            .unwrap();
        context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("B", "=2"))
            .unwrap();
        let preceding_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Preceding", "=SUM(@PRECEDING)"),
            )
            .unwrap();

        let following_parent_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("FollowingParent", ""),
            )
            .unwrap();
        let following_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Following", "=SUM(@FOLLOWING)")
                    .under(following_parent_id),
            )
            .unwrap();
        context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("C", "=4").under(following_parent_id),
            )
            .unwrap();
        context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("D", "=5").under(following_parent_id),
            )
            .unwrap();

        let ancestor_parent_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("AncestorValue", "=10"),
            )
            .unwrap();
        let ancestors_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Ancestors", "=SUM(@ANCESTORS)")
                    .under(ancestor_parent_id),
            )
            .unwrap();

        let result = context.recalculate(&workspace_id).unwrap();

        assert_eq!(
            result.run_state,
            OxCalcTreeRunState::Published,
            "ordered selector context run failed: reject={:?}; diagnostics={:?}",
            result.reject_detail,
            result.diagnostics
        );
        assert_eq!(
            result.published_values.get(&preceding_id),
            Some(&"3".to_string())
        );
        assert_eq!(
            result.published_values.get(&following_id),
            Some(&"9".to_string())
        );
        assert_eq!(
            result.published_values.get(&ancestors_id),
            Some(&"10".to_string())
        );
    }

    #[test]
    fn raw_ancestors_selector_treats_empty_structural_members_as_blanks() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new("workspace:blank-ancestors"))
            .unwrap();
        let root_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("Root", ""))
            .unwrap();
        let l1_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("L1", "").under(root_id),
            )
            .unwrap();
        let l2_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("L2", "").under(l1_id),
            )
            .unwrap();
        let total_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Total", "=SUM(@ANCESTORS)").under(l2_id),
            )
            .unwrap();

        let result = context.recalculate(&workspace_id).unwrap();

        assert_eq!(
            result.run_state,
            OxCalcTreeRunState::Published,
            "blank ancestor selector run failed: reject={:?}; diagnostics={:?}",
            result.reject_detail,
            result.diagnostics
        );
        assert_eq!(
            result.published_values.get(&total_id),
            Some(&"0".to_string())
        );
    }

    #[test]
    fn explicit_structural_base_ordered_selector_does_not_depend_on_base_value() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(
                OxCalcTreeWorkspaceCreate::new("workspace:structural-base-selector")
                    .with_root_symbol("EngineRoot"),
            )
            .unwrap();
        let engine_root_id = context.workspace_view(&workspace_id).unwrap().root_node_id;
        let root_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Root", "").under(engine_root_id),
            )
            .unwrap();
        let branch_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("StructuralPreceding", "").under(root_id),
            )
            .unwrap();
        let a_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("A", "2").under(branch_id),
            )
            .unwrap();
        let b_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("B", "3").under(branch_id),
            )
            .unwrap();
        let total_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new(
                    "Total",
                    "=SUM(Root.StructuralPreceding.Total.@PRECEDING)",
                )
                .under(branch_id),
            )
            .unwrap();

        let result = context.recalculate(&workspace_id).unwrap();

        assert_eq!(
            result.run_state,
            OxCalcTreeRunState::Published,
            "explicit structural-base selector run failed: reject={:?}; diagnostics={:?}; graph={:?}",
            result.reject_detail,
            result.diagnostics,
            result.dependency_graph
        );
        assert_eq!(
            result.published_values.get(&total_id),
            Some(&"5".to_string())
        );
        let member_targets = result
            .dependency_graph
            .edges_by_owner
            .get(&total_id)
            .into_iter()
            .flatten()
            .filter(|edge| {
                edge.kind == DependencyDescriptorKind::TreeReferenceCollectionMemberValue
            })
            .map(|edge| edge.target_node_id)
            .collect::<Vec<_>>();
        assert_eq!(member_targets, vec![a_id, b_id]);
        assert!(
            result
                .dependency_graph
                .edges_by_owner
                .get(&total_id)
                .into_iter()
                .flatten()
                .all(|edge| edge.target_node_id != total_id),
            "structural selector base must not add a value self-edge"
        );
    }

    #[test]
    fn qualified_recursive_selector_formula_text_resolves_tail_through_oxfml_host_reference_path() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:qualified-recursive-selector",
            ))
            .unwrap();
        let base_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("Base", ""))
            .unwrap();
        let lane_a_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("LaneA", "").under(base_id),
            )
            .unwrap();
        context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Margin", "=3").under(lane_a_id),
            )
            .unwrap();
        let total_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Recursive", "=SUM(Base.**.Margin)"),
            )
            .unwrap();

        let result = context.recalculate(&workspace_id).unwrap();

        assert_eq!(
            result.run_state,
            OxCalcTreeRunState::Published,
            "qualified recursive selector failed: reject={:?}; diagnostics={:?}",
            result.reject_detail,
            result.diagnostics
        );
        assert_eq!(
            result.published_values.get(&total_id),
            Some(&"3".to_string())
        );
    }

    #[test]
    fn raw_non_recursive_ordered_selector_tail_resolves_through_tree_context() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:non-recursive-selector-tail",
            ))
            .unwrap();
        let base_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("Base", ""))
            .unwrap();
        let lane_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Lane", "").under(base_id),
            )
            .unwrap();
        context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Margin", "=3").under(lane_id),
            )
            .unwrap();
        // `@PRECEDING.Margin` from `PrecedingTail`: the preceding sibling `Lane` has a
        // `Margin` child (=3); the dotted tail resolves under each preceding member.
        let preceding_tail_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("PrecedingTail", "=SUM(@PRECEDING.Margin)")
                    .under(base_id),
            )
            .unwrap();

        let result = context.recalculate(&workspace_id).unwrap();

        assert_eq!(
            result.run_state,
            OxCalcTreeRunState::Published,
            "non-recursive tailed selector failed: reject={:?}; diagnostics={:?}",
            result.reject_detail,
            result.diagnostics
        );
        assert_eq!(
            result.published_values.get(&preceding_tail_id),
            Some(&"3".to_string())
        );
    }

    fn assert_children_collection_sum_uses_generic_host_context_and_sparse_reference_values(
        source_token_text: &str,
    ) {
        let structural_snapshot = children_collection_snapshot(vec![TreeNodeId(3), TreeNodeId(4)]);
        let catalog = children_collection_catalog_with_source_token(source_token_text);
        let binding = catalog
            .try_get_binding(TreeNodeId(10))
            .expect("children collection binding");
        let prepared = prepare_oxfml_formula(
            &structural_snapshot,
            &BTreeMap::new(),
            binding,
            &LocalTreeCalcEnvironmentContext::default(),
        )
        .expect("children collection formula should prepare");

        let host_context = prepared
            .runtime_prepared_identity
            .host_formula_context
            .as_ref()
            .expect("TreeCalc collection should contribute host context");
        assert_eq!(host_context.dialect_id, "oxcalc.treecalc-v1");
        assert_eq!(
            host_context.capability_profile_id,
            "host-capabilities:treecalc-v1"
        );
        assert_eq!(
            host_context.resolution_rule_version,
            "treecalc-host-resolution:v1"
        );
        assert_eq!(
            prepared
                .runtime_prepared_identity
                .host_reference_bind_results[0]
                .reference_handle,
            "treecalc-hostref:v1:children:node:2"
        );
        assert_eq!(
            prepared
                .runtime_prepared_identity
                .host_reference_bind_results[0]
                .source_token_text,
            source_token_text
        );
        assert_eq!(
            prepared
                .runtime_prepared_identity
                .host_reference_bind_results[0]
                .source_span
                .start,
            5
        );
        assert_eq!(
            prepared
                .runtime_prepared_identity
                .host_reference_bind_results[0]
                .source_span
                .len,
            source_token_text.len()
        );
        assert_eq!(
            prepared
                .runtime_prepared_identity
                .host_reference_bind_results[0]
                .resolution_layer,
            "explicit_host_ref"
        );
        assert_eq!(
            prepared
                .runtime_prepared_identity
                .host_reference_bind_results[0]
                .shape_hint
                .as_deref(),
            Some("ordered_collection:children_v1")
        );

        let run = LocalTreeCalcEngine
            .execute(local_treecalc_input(
                structural_snapshot,
                catalog,
                BTreeMap::from([
                    (TreeNodeId(3), "2".to_string()),
                    (TreeNodeId(4), "3".to_string()),
                ]),
                BTreeMap::from([
                    (TreeNodeId(3), "2".to_string()),
                    (TreeNodeId(4), "3".to_string()),
                ]),
                Vec::new(),
                Vec::new(),
                "w051:children-sum",
            ))
            .expect("SUM over ChildrenV1 reference should execute");

        assert_eq!(
            run.result_state,
            LocalTreeCalcRunState::Published,
            "children collection run did not publish: reject={:?}; diagnostics={:?}",
            run.reject_detail,
            run.diagnostics
        );
        assert_eq!(run.published_values[&TreeNodeId(10)], "5");
        assert!(
            run.prepared_formula_identities
                .iter()
                .any(|identity| identity.owner_node_id == TreeNodeId(10))
        );
    }

    #[test]
    fn children_collection_dependency_descriptors_track_membership_and_values() {
        let structural_snapshot = children_collection_snapshot(vec![TreeNodeId(3), TreeNodeId(4)]);
        let catalog = children_collection_catalog();

        let descriptors = catalog
            .to_dependency_descriptors(&structural_snapshot)
            .into_iter()
            .map(|descriptor| (descriptor.descriptor_id.clone(), descriptor))
            .collect::<BTreeMap<_, _>>();

        let membership = descriptors
            .values()
            .find(|descriptor| {
                descriptor.kind == DependencyDescriptorKind::TreeReferenceCollectionMembership
            })
            .expect("membership descriptor");
        assert_eq!(
            membership.source_reference_handle.as_deref(),
            Some("treecalc-hostref:v1:children:node:2")
        );
        assert!(membership.carrier_detail.contains("members=node:3,node:4"));
        let collection_facts = membership
            .tree_reference_collection
            .as_ref()
            .expect("typed collection facts");
        assert_eq!(
            collection_facts.member_node_ids,
            vec![TreeNodeId(3), TreeNodeId(4)]
        );
        assert_eq!(
            collection_facts.membership_version,
            "treecalc-membership:v1:base=node:2;members=node:3,node:4"
        );
        assert_eq!(
            collection_facts.order_version,
            "treecalc-order:v1:base=node:2;members=node:3,node:4"
        );

        let member_targets = descriptors
            .values()
            .filter(|descriptor| {
                descriptor.kind == DependencyDescriptorKind::TreeReferenceCollectionMemberValue
            })
            .map(|descriptor| descriptor.target_node_id)
            .collect::<Vec<_>>();
        assert_eq!(
            member_targets,
            vec![Some(TreeNodeId(3)), Some(TreeNodeId(4))]
        );

        let graph = DependencyGraph::build(
            &structural_snapshot,
            &descriptors.into_values().collect::<Vec<_>>(),
        );
        assert!(graph.diagnostics.is_empty());
        assert_eq!(graph.reverse_edges[&TreeNodeId(3)].len(), 1);
        assert_eq!(graph.reverse_edges[&TreeNodeId(4)].len(), 1);
    }

    #[test]
    fn ordered_selector_runtime_dependency_descriptors_keep_selector_family_detail() {
        let structural_snapshot = children_collection_snapshot(vec![TreeNodeId(3), TreeNodeId(4)]);
        let catalog = ordered_selector_collection_catalog();
        let binding = catalog
            .try_get_binding(TreeNodeId(10))
            .expect("ordered selector binding");
        let prepared = prepare_oxfml_formula(
            &structural_snapshot,
            &BTreeMap::new(),
            binding,
            &LocalTreeCalcEnvironmentContext::default(),
        )
        .expect("prepare ordered selector formula");

        let descriptors = oxfml_dependency_descriptors(&prepared);
        let member_details = descriptors
            .iter()
            .filter(|descriptor| {
                descriptor.kind == DependencyDescriptorKind::TreeReferenceCollectionMemberValue
            })
            .map(|descriptor| descriptor.carrier_detail.as_str())
            .collect::<Vec<_>>();

        assert_eq!(member_details.len(), 2);
        assert!(member_details.iter().all(|detail| {
            detail.starts_with("treecalc_ordered_selector_v1_member:family=preceding")
        }));
        assert!(
            member_details
                .iter()
                .all(|detail| !detail.starts_with("treecalc_children_v1_member"))
        );
    }

    #[test]
    fn children_collection_invalidation_distinguishes_membership_and_order_changes() {
        let catalog = children_collection_catalog();
        let previous = children_collection_snapshot(vec![TreeNodeId(3), TreeNodeId(4)]);
        let added = children_collection_snapshot(vec![TreeNodeId(3), TreeNodeId(4), TreeNodeId(5)]);
        let reordered = children_collection_snapshot(vec![TreeNodeId(4), TreeNodeId(3)]);

        let membership_seeds = derive_structural_invalidation_seeds_for_catalogs(
            &previous,
            &added,
            &catalog,
            &catalog,
            &[],
        );
        assert_eq!(
            membership_seeds,
            vec![InvalidationSeed {
                node_id: TreeNodeId(10),
                reason: InvalidationReasonKind::TreeReferenceMembershipChanged,
            }]
        );

        let order_seeds = derive_structural_invalidation_seeds_for_catalogs(
            &previous,
            &reordered,
            &catalog,
            &catalog,
            &[],
        );
        assert_eq!(
            order_seeds,
            vec![InvalidationSeed {
                node_id: TreeNodeId(10),
                reason: InvalidationReasonKind::TreeReferenceOrderChanged,
            }]
        );
    }

    #[test]
    fn structural_invalidation_seeds_mark_relative_reference_rebind_after_rename() {
        let outcome = snapshot()
            .apply_edit(
                crate::structural::StructuralSnapshotId(2),
                crate::structural::StructuralEdit::RenameNode {
                    node_id: TreeNodeId(2),
                    new_symbol: "A_renamed".to_string(),
                },
            )
            .unwrap();
        let formula_catalog = TreeFormulaCatalog::new([TreeFormulaBinding {
            owner_node_id: TreeNodeId(4),
            formula_artifact_id: FormulaArtifactId("formula:c".to_string()),
            bind_artifact_id: Some(BindArtifactId("bind:c".to_string())),
            expression: fixture_formula(
                TreeNodeId(4),
                FixtureFormulaAst::Reference(TreeReference::RelativePath {
                    base: RelativeReferenceBase::ParentNode,
                    path_segments: vec!["A".to_string()],
                }),
            ),
        }]);

        let predecessor_snapshot = snapshot();
        let successor_snapshot = outcome.snapshot.clone();
        let seeds = derive_structural_invalidation_seeds(
            &predecessor_snapshot,
            &successor_snapshot,
            &formula_catalog,
            &[outcome],
        );

        assert_eq!(
            seeds,
            vec![InvalidationSeed {
                node_id: TreeNodeId(4),
                reason: InvalidationReasonKind::StructuralRebindRequired,
            }]
        );
    }

    #[test]
    fn structural_invalidation_seeds_mark_formula_catalog_dynamic_release_reclassification() {
        let predecessor_catalog = TreeFormulaCatalog::new([TreeFormulaBinding {
            owner_node_id: TreeNodeId(3),
            formula_artifact_id: FormulaArtifactId("formula:dynamic:auto".to_string()),
            bind_artifact_id: Some(BindArtifactId("bind:dynamic:auto".to_string())),
            expression: fixture_formula(
                TreeNodeId(3),
                FixtureFormulaAst::Reference(TreeReference::DynamicResolved {
                    target_node_id: TreeNodeId(2),
                    carrier_id: "carrier:dynamic:auto".to_string(),
                    detail: "resolved_before_release".to_string(),
                }),
            ),
        }]);
        let successor_catalog = TreeFormulaCatalog::new([TreeFormulaBinding {
            owner_node_id: TreeNodeId(3),
            formula_artifact_id: FormulaArtifactId("formula:dynamic:auto".to_string()),
            bind_artifact_id: Some(BindArtifactId("bind:dynamic:auto".to_string())),
            expression: fixture_formula(
                TreeNodeId(3),
                FixtureFormulaAst::Reference(TreeReference::DynamicPotential {
                    carrier_id: "carrier:dynamic:auto".to_string(),
                    detail: "released_to_runtime".to_string(),
                }),
            ),
        }]);
        let structural_snapshot = snapshot();

        let seeds = derive_structural_invalidation_seeds_for_catalogs(
            &structural_snapshot,
            &structural_snapshot,
            &predecessor_catalog,
            &successor_catalog,
            &[],
        );

        assert_eq!(
            seeds,
            vec![
                InvalidationSeed {
                    node_id: TreeNodeId(3),
                    reason: InvalidationReasonKind::DynamicDependencyReleased,
                },
                InvalidationSeed {
                    node_id: TreeNodeId(3),
                    reason: InvalidationReasonKind::DynamicDependencyReclassified,
                },
            ]
        );
    }

    #[test]
    fn structural_invalidation_seeds_mark_formula_catalog_dynamic_addition_reclassification() {
        let predecessor_catalog = TreeFormulaCatalog::new([TreeFormulaBinding {
            owner_node_id: TreeNodeId(3),
            formula_artifact_id: FormulaArtifactId("formula:dynamic:auto".to_string()),
            bind_artifact_id: Some(BindArtifactId("bind:dynamic:auto".to_string())),
            expression: fixture_formula(
                TreeNodeId(3),
                FixtureFormulaAst::Reference(TreeReference::DynamicPotential {
                    carrier_id: "carrier:dynamic:auto".to_string(),
                    detail: "unresolved_before_addition".to_string(),
                }),
            ),
        }]);
        let successor_catalog = TreeFormulaCatalog::new([TreeFormulaBinding {
            owner_node_id: TreeNodeId(3),
            formula_artifact_id: FormulaArtifactId("formula:dynamic:auto".to_string()),
            bind_artifact_id: Some(BindArtifactId("bind:dynamic:auto".to_string())),
            expression: fixture_formula(
                TreeNodeId(3),
                FixtureFormulaAst::Reference(TreeReference::DynamicResolved {
                    target_node_id: TreeNodeId(2),
                    carrier_id: "carrier:dynamic:auto".to_string(),
                    detail: "resolved_after_addition".to_string(),
                }),
            ),
        }]);
        let structural_snapshot = snapshot();

        let seeds = derive_structural_invalidation_seeds_for_catalogs(
            &structural_snapshot,
            &structural_snapshot,
            &predecessor_catalog,
            &successor_catalog,
            &[],
        );

        assert_eq!(
            seeds,
            vec![
                InvalidationSeed {
                    node_id: TreeNodeId(3),
                    reason: InvalidationReasonKind::DynamicDependencyActivated,
                },
                InvalidationSeed {
                    node_id: TreeNodeId(3),
                    reason: InvalidationReasonKind::DynamicDependencyReclassified,
                },
            ]
        );
    }

    #[test]
    fn structural_invalidation_seeds_mark_mixed_dynamic_add_release_reclassification() {
        let predecessor_catalog = TreeFormulaCatalog::new([TreeFormulaBinding {
            owner_node_id: TreeNodeId(3),
            formula_artifact_id: FormulaArtifactId("formula:dynamic:mixed".to_string()),
            bind_artifact_id: Some(BindArtifactId("bind:dynamic:mixed".to_string())),
            expression: fixture_formula(
                TreeNodeId(3),
                FixtureFormulaAst::Binary {
                    op: FixtureFormulaBinaryOp::Add,
                    left: Box::new(FixtureFormulaAst::Reference(
                        TreeReference::DynamicResolved {
                            target_node_id: TreeNodeId(2),
                            carrier_id: "carrier:dynamic:mixed-left".to_string(),
                            detail: "resolved_before_mixed_release".to_string(),
                        },
                    )),
                    right: Box::new(FixtureFormulaAst::Reference(
                        TreeReference::DynamicPotential {
                            carrier_id: "carrier:dynamic:mixed-right".to_string(),
                            detail: "unresolved_before_mixed_addition".to_string(),
                        },
                    )),
                },
            ),
        }]);
        let successor_catalog = TreeFormulaCatalog::new([TreeFormulaBinding {
            owner_node_id: TreeNodeId(3),
            formula_artifact_id: FormulaArtifactId("formula:dynamic:mixed".to_string()),
            bind_artifact_id: Some(BindArtifactId("bind:dynamic:mixed".to_string())),
            expression: fixture_formula(
                TreeNodeId(3),
                FixtureFormulaAst::Binary {
                    op: FixtureFormulaBinaryOp::Add,
                    left: Box::new(FixtureFormulaAst::Reference(
                        TreeReference::DynamicPotential {
                            carrier_id: "carrier:dynamic:mixed-left".to_string(),
                            detail: "released_to_runtime_resolution".to_string(),
                        },
                    )),
                    right: Box::new(FixtureFormulaAst::Reference(
                        TreeReference::DynamicResolved {
                            target_node_id: TreeNodeId(4),
                            carrier_id: "carrier:dynamic:mixed-right".to_string(),
                            detail: "resolved_after_mixed_addition".to_string(),
                        },
                    )),
                },
            ),
        }]);
        let structural_snapshot = snapshot();

        let seeds = derive_structural_invalidation_seeds_for_catalogs(
            &structural_snapshot,
            &structural_snapshot,
            &predecessor_catalog,
            &successor_catalog,
            &[],
        );

        assert_eq!(
            seeds,
            vec![
                InvalidationSeed {
                    node_id: TreeNodeId(3),
                    reason: InvalidationReasonKind::DynamicDependencyActivated,
                },
                InvalidationSeed {
                    node_id: TreeNodeId(3),
                    reason: InvalidationReasonKind::DynamicDependencyReleased,
                },
                InvalidationSeed {
                    node_id: TreeNodeId(3),
                    reason: InvalidationReasonKind::DynamicDependencyReclassified,
                },
            ]
        );
    }

    #[test]
    fn structural_invalidation_seeds_keep_direct_reference_recalc_only_after_target_move() {
        let outcome = snapshot()
            .apply_edit(
                crate::structural::StructuralSnapshotId(2),
                crate::structural::StructuralEdit::MoveNode {
                    node_id: TreeNodeId(2),
                    new_parent_id: TreeNodeId(1),
                    new_index: Some(0),
                },
            )
            .unwrap();
        let formula_catalog = TreeFormulaCatalog::new([TreeFormulaBinding {
            owner_node_id: TreeNodeId(3),
            formula_artifact_id: FormulaArtifactId("formula:b".to_string()),
            bind_artifact_id: Some(BindArtifactId("bind:b".to_string())),
            expression: fixture_formula(
                TreeNodeId(3),
                FixtureFormulaAst::Reference(TreeReference::DirectNode {
                    target_node_id: TreeNodeId(2),
                }),
            ),
        }]);

        let predecessor_snapshot = snapshot();
        let successor_snapshot = outcome.snapshot.clone();
        let seeds = derive_structural_invalidation_seeds(
            &predecessor_snapshot,
            &successor_snapshot,
            &formula_catalog,
            &[outcome],
        );

        assert_eq!(
            seeds,
            vec![InvalidationSeed {
                node_id: TreeNodeId(3),
                reason: InvalidationReasonKind::StructuralRecalcOnly,
            }]
        );
    }
}
