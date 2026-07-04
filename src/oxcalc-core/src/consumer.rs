#![forbid(unsafe_code)]

//! Consumer-facing OxCalc runtime contract for tree-substrate hosts.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt;
use std::sync::Arc;

use oxfml_core::consumer::runtime::{
    RuntimeAuthoredInputResult, RuntimeDryBindInputKind, RuntimeDryBindProfileViolationKind,
    RuntimeDryBindVerdict, RuntimeEnvironment, RuntimeHostFormulaContext,
};
use oxfml_core::{FormulaSourceRecord, StructuredReferenceBindRecord};
use oxfunc_core::value::{
    ArrayShape, CalcArray, CalcValue, CoreValue, ExcelText, WorksheetErrorCode,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::coordinator::{
    AcceptedCandidateResult, DependencyShapeUpdate, PublicationBundle, RejectDetail, RejectKind,
    RuntimeEffect, calc_value_display_map, calc_value_display_text,
};
use crate::dependency::{
    DependencyDescriptor, DependencyDescriptorKind, DependencyGraph, InvalidationClosure,
    InvalidationReasonKind, InvalidationSeed, TreeReferenceCollectionDependency,
    TreeReferenceCollectionFamily, WorkspaceQualifiedTarget,
};
use crate::formula::{TreeFormula, TreeFormulaBinding, TreeFormulaCatalog};
use crate::grid::authored::{GridAuthoredCell, GridFormulaCell};
use crate::grid::coords::{ExcelGridBounds, ExcelGridCellAddress};
use crate::grid::error::GridRefError;
use crate::grid::geometry::GridRect;
use crate::grid::machine::{
    GRID_CALC_REF_DEFAULT_MATERIALIZATION_LIMIT, GridDifferentialMismatch, GridEngineMode,
    GridOptimizedSheet, GridSpillFact, GridTableOverlay,
};
use crate::recalc::{NodeCalcState, OverlayEntry};
use crate::structural::{
    BindArtifactId, FormulaArtifactId, StructuralEdit, StructuralError, StructuralGridShape,
    StructuralNode, StructuralNodeKind, StructuralSnapshot, StructuralSnapshotId,
    StructuralTableShape, TreeNodeId,
};
use crate::structured_table::{
    StructuredTableBindRecordIntakeError, StructuredTableContextPacket,
    StructuredTableDependencyLowering, StructuredTableDependencyLoweringRequest,
    StructuredTableReferenceIntake, TableCallerRegion, TableRef, TreeCalcDynamicTableRebindReport,
    TreeCalcDynamicTableRebindRequest, TreeCalcTableCatalogResolution,
    TreeCalcTableCatalogResolveRequest, TreeCalcTableCatalogResolverContext,
    TreeCalcTableCatalogWorkspace, TreeCalcTableColumnBodyMetadata,
    TreeCalcTableColumnFormulaRuntimeRequest, TreeCalcTableColumnSnapshot,
    TreeCalcTableDeletedFact, TreeCalcTableDependencyInventory, TreeCalcTableFormulaMetadata,
    TreeCalcTableFormulaRuntimeContext, TreeCalcTableLifecycleContextVersions,
    TreeCalcTableNodeProjection, TreeCalcTableNodeSnapshot, TreeCalcTableProjectionError,
    TreeCalcTableUpdateScenarioKind, classify_treecalc_dynamic_table_rebind,
    classify_treecalc_table_update, dry_bind_treecalc_table_column_formula,
    dry_bind_treecalc_table_totals_formula, inventory_treecalc_table_dependency_facts,
    lower_structured_table_dependencies, project_treecalc_table_node_snapshot,
    resolve_treecalc_table_catalog_reference,
};
use crate::tree_reference_rebind::descriptor_invalidation_facts;
use crate::tree_reference_resolution::TreeNameResolutionIndex;
use crate::tree_reference_system::{
    TreeCalcContextReferenceBindProfile, treecalc_reference_bind_profile,
};
use crate::treecalc::{
    DerivationTraceRecord, LocalTreeCalcEngine, LocalTreeCalcEnvironmentContext,
    LocalTreeCalcError, LocalTreeCalcInput, LocalTreeCalcLayerSnapshotIds, LocalTreeCalcPhaseKey,
    LocalTreeCalcRunArtifacts, LocalTreeCalcRunState, LocalTreeCalcSchedulingPolicy,
    OxCalcTreeBindingDiagnostic, PreparedFormulaRetention,
    dynamic_dependency_descriptors_from_published_facts,
    dynamic_dependency_facts_from_runtime_effects,
};
use crate::workspace_revision::{
    DependencyShapeSnapshot, DependencyShapeSnapshotId, FormulaBindingSnapshot,
    FormulaBindingSnapshotId, NamespaceSnapshot, NamespaceSnapshotId, NodeInputKind,
    NodeInputRecord, NodeInputSnapshot, NodeInputSnapshotId, PublicationSnapshot,
    PublicationSnapshotId, RuntimeOverlaySet, RuntimeOverlaySetId, WorkspaceRevision,
    WorkspaceRevisionError, WorkspaceRevisionGraph, WorkspaceRevisionGraphEntry,
    WorkspaceRevisionGraphEvictionError, WorkspaceRevisionGraphNavigationError,
    WorkspaceRevisionId, WorkspaceRevisionInvalidationSummaryEntry,
    WorkspaceRevisionTransactionSummary,
};

pub const OXCALC_TREE_WORKSPACE_SNAPSHOT_SCHEMA_V1: &str = "oxcalc.tree.workspace_snapshot.v1";
/// v2 stores `publication_values` as structured [`SnapshotCalcValue`] rather
/// than display strings, so computed array values survive a persist/reload
/// round-trip with their shape and data intact. v1 documents (display-string
/// values) are not forward-compatible and are rejected on import.
pub const OXCALC_TREE_WORKSPACE_SNAPSHOT_SCHEMA_V2: &str = "oxcalc.tree.workspace_snapshot.v2";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OxCalcTreeRunState {
    Published,
    VerifiedClean,
    Rejected,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OxCalcTreeCalculationOutcome {
    pub run_state: OxCalcTreeRunState,
    pub dependency_graph: DependencyGraph,
    pub invalidation_closure: InvalidationClosure,
    pub evaluation_order: Vec<TreeNodeId>,
    pub runtime_effects: Vec<RuntimeEffect>,
    pub runtime_effect_overlays: Vec<OverlayEntry>,
    pub derivation_traces: Vec<DerivationTraceRecord>,
    pub candidate_result: Option<AcceptedCandidateResult>,
    pub publication_bundle: Option<PublicationBundle>,
    pub reject_detail: Option<RejectDetail>,
    pub published_values: BTreeMap<TreeNodeId, String>,
    pub published_calc_values: BTreeMap<TreeNodeId, CalcValue>,
    pub published_value_epochs: BTreeMap<TreeNodeId, u64>,
    pub node_states: BTreeMap<TreeNodeId, NodeCalcState>,
    pub phase_timings_micros: BTreeMap<LocalTreeCalcPhaseKey, u128>,
    pub binding_diagnostics: Vec<OxCalcTreeBindingDiagnostic>,
    pub diagnostics: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OxCalcTreeDryBindInputKind {
    Literal,
    Formula,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OxCalcTreeDryBindDiagnosticStage {
    Syntax,
    Bind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OxCalcTreeDryBindDiagnostic {
    pub stage: OxCalcTreeDryBindDiagnosticStage,
    pub message: String,
    pub span_start_utf8: usize,
    pub span_len_utf8: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OxCalcTreeDryBindProfileViolation {
    pub kind: OxCalcTreeDryBindProfileViolationKind,
    pub feature: String,
    pub message: String,
    pub span_start_utf8: usize,
    pub span_len_utf8: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OxCalcTreeDryBindProfileViolationKind {
    FunctionUnavailable {
        function_id: String,
        function_name: String,
        reason: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OxCalcTreeDryBindVerdict {
    pub node_id: TreeNodeId,
    pub input_kind: OxCalcTreeDryBindInputKind,
    pub legal: bool,
    pub diagnostics: Vec<OxCalcTreeDryBindDiagnostic>,
    pub profile_violations: Vec<OxCalcTreeDryBindProfileViolation>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct OxCalcTreeWorkspaceId(pub String);

impl OxCalcTreeWorkspaceId {
    #[must_use]
    pub fn new(workspace_id: impl Into<String>) -> Self {
        Self(workspace_id.into())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for OxCalcTreeWorkspaceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OxCalcTreeWorkspaceCreate {
    pub workspace_id: OxCalcTreeWorkspaceId,
    pub root_symbol: String,
}

impl OxCalcTreeWorkspaceCreate {
    #[must_use]
    pub fn new(workspace_id: impl Into<String>) -> Self {
        Self {
            workspace_id: OxCalcTreeWorkspaceId::new(workspace_id),
            root_symbol: "Root".to_string(),
        }
    }

    #[must_use]
    pub fn with_root_symbol(mut self, root_symbol: impl Into<String>) -> Self {
        self.root_symbol = root_symbol.into();
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OxCalcTreeNodeCreate {
    pub parent_node_id: Option<TreeNodeId>,
    pub reserved_node_id: Option<TreeNodeId>,
    pub symbol: String,
    pub formula_text: String,
    pub is_meta: bool,
}

impl OxCalcTreeNodeCreate {
    #[must_use]
    pub fn new(symbol: impl Into<String>, formula_text: impl Into<String>) -> Self {
        Self {
            parent_node_id: None,
            reserved_node_id: None,
            symbol: symbol.into(),
            formula_text: formula_text.into(),
            is_meta: false,
        }
    }

    #[must_use]
    pub fn under(mut self, parent_node_id: TreeNodeId) -> Self {
        self.parent_node_id = Some(parent_node_id);
        self
    }

    #[must_use]
    pub fn with_meta(mut self, is_meta: bool) -> Self {
        self.is_meta = is_meta;
        self
    }

    #[must_use]
    pub fn with_reserved_node_id(mut self, node_id: TreeNodeId) -> Self {
        self.reserved_node_id = Some(node_id);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OxCalcTreeTransactionId(String);

impl OxCalcTreeTransactionId {
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for OxCalcTreeTransactionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OxCalcTreeEditTransaction {
    pub workspace_id: OxCalcTreeWorkspaceId,
    pub edits: Vec<OxCalcTreeEdit>,
    pub recalc_policy: TransactionRecalcPolicy,
}

impl OxCalcTreeEditTransaction {
    #[must_use]
    pub fn new(workspace_id: OxCalcTreeWorkspaceId) -> Self {
        Self {
            workspace_id,
            edits: Vec::new(),
            recalc_policy: TransactionRecalcPolicy::RecalculateAndPublishOnce,
        }
    }

    #[must_use]
    pub fn with_edit(mut self, edit: OxCalcTreeEdit) -> Self {
        self.edits.push(edit);
        self
    }

    #[must_use]
    pub fn with_recalc_policy(mut self, recalc_policy: TransactionRecalcPolicy) -> Self {
        self.recalc_policy = recalc_policy;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OxCalcTreeEdit {
    AddNode {
        request: OxCalcTreeNodeCreate,
    },
    SetNodeInput {
        node_id: TreeNodeId,
        input: String,
    },
    SetNodeFormulaText {
        node_id: TreeNodeId,
        formula_text: String,
    },
    SetNodeTable {
        node_id: TreeNodeId,
        snapshot: TreeCalcTableNodeSnapshot,
    },
    SetNodeMeta {
        node_id: TreeNodeId,
        is_meta: bool,
    },
    SetReferenceCollectionMembership {
        owner_node_id: TreeNodeId,
        source_reference_handle: String,
        member_node_ids: Vec<TreeNodeId>,
    },
    RenameNode {
        node_id: TreeNodeId,
        new_symbol: String,
    },
    MoveNode {
        node_id: TreeNodeId,
        new_parent_id: TreeNodeId,
        new_index: Option<usize>,
    },
    ReorderNode {
        node_id: TreeNodeId,
        new_index: usize,
    },
    DeleteNode {
        node_id: TreeNodeId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OxCalcTreeEditResult {
    NodeAdded { node_id: TreeNodeId },
    Applied,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionRecalcPolicy {
    RecalculateAndPublishOnce,
    ApplyOnly,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OxCalcTreeTransactionOutcome {
    pub transaction_id: OxCalcTreeTransactionId,
    pub workspace_revision_id: WorkspaceRevisionId,
    pub predecessor_workspace_revision_id: WorkspaceRevisionId,
    pub successor_workspace_revision_id: WorkspaceRevisionId,
    pub calculation: Option<OxCalcTreeCalculationOutcome>,
    pub edit_count: usize,
    pub edit_results: Vec<OxCalcTreeEditResult>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OxCalcTreeRevisionNavigationOutcome {
    pub predecessor_workspace_revision_id: WorkspaceRevisionId,
    pub successor_workspace_revision_id: WorkspaceRevisionId,
    pub workspace_revision_id: WorkspaceRevisionId,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CandidateOverlayHandle(String);

impl CandidateOverlayHandle {
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CandidateOverlayHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OxCalcTreeOpenCandidateRequest {
    pub workspace_id: OxCalcTreeWorkspaceId,
    pub basis_revision_id: WorkspaceRevisionId,
    pub parent_candidate: Option<CandidateOverlayHandle>,
}

impl OxCalcTreeOpenCandidateRequest {
    #[must_use]
    pub fn new(
        workspace_id: OxCalcTreeWorkspaceId,
        basis_revision_id: WorkspaceRevisionId,
    ) -> Self {
        Self {
            workspace_id,
            basis_revision_id,
            parent_candidate: None,
        }
    }

    #[must_use]
    pub fn with_parent_candidate(mut self, parent_candidate: CandidateOverlayHandle) -> Self {
        self.parent_candidate = Some(parent_candidate);
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OxCalcTreeCandidateView {
    pub handle: CandidateOverlayHandle,
    pub workspace_id: OxCalcTreeWorkspaceId,
    pub basis_revision_id: WorkspaceRevisionId,
    pub parent_candidate: Option<CandidateOverlayHandle>,
    pub retention_pin_count: usize,
    pub workspace_revision_id: WorkspaceRevisionId,
    pub workspace_revision_graph_entries: Vec<WorkspaceRevisionGraphEntry>,
    pub nodes: Vec<OxCalcTreeNodeView>,
    pub run_state: Option<OxCalcTreeRunState>,
    pub value_epoch_basis: u64,
    pub publication_value_epoch_basis: u64,
    pub calculation: Option<OxCalcTreeCalculationOutcome>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OxCalcTreeCandidateCommitOutcome {
    pub handle: CandidateOverlayHandle,
    pub workspace_id: OxCalcTreeWorkspaceId,
    pub basis_revision_id: WorkspaceRevisionId,
    pub predecessor_workspace_revision_id: WorkspaceRevisionId,
    pub successor_workspace_revision_id: WorkspaceRevisionId,
    pub workspace_revision_graph_entries: Vec<WorkspaceRevisionGraphEntry>,
    pub calculation: Option<OxCalcTreeCalculationOutcome>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OxCalcTreeCandidateReapPolicy {
    pub max_retained_candidates: usize,
}

impl OxCalcTreeCandidateReapPolicy {
    #[must_use]
    pub fn max_retained(max_retained_candidates: usize) -> Self {
        Self {
            max_retained_candidates,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OxCalcTreeCandidatePressure {
    pub retained_candidate_count: usize,
    pub child_protected_candidate_count: usize,
    pub host_pinned_candidate_count: usize,
    pub protected_candidate_count: usize,
    pub reclaimable_candidate_count: usize,
    pub over_budget_candidate_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OxCalcTreeCandidateReapReport {
    pub pressure_before: OxCalcTreeCandidatePressure,
    pub pressure_after: OxCalcTreeCandidatePressure,
    pub reaped_handles: Vec<CandidateOverlayHandle>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OxCalcTreeCandidateRebaseConflictKind {
    OverlappingNodeEdits,
    ReplayValidationRejected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OxCalcTreeCandidateRebaseConflictReport {
    pub handle: CandidateOverlayHandle,
    pub basis_revision_id: WorkspaceRevisionId,
    pub current_revision_id: WorkspaceRevisionId,
    pub kind: OxCalcTreeCandidateRebaseConflictKind,
    pub candidate_touched_nodes: Vec<TreeNodeId>,
    pub live_touched_nodes: Vec<TreeNodeId>,
    pub overlapping_nodes: Vec<TreeNodeId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OxCalcTreePreviewMutation {
    InvalidateNode {
        node_id: TreeNodeId,
        reason: InvalidationReasonKind,
    },
    SetNodeInput {
        node_id: TreeNodeId,
    },
    SetNodeFormulaText {
        node_id: TreeNodeId,
        formula_text: String,
    },
    SetNodeTable {
        node_id: TreeNodeId,
        snapshot: TreeCalcTableNodeSnapshot,
        scenario: TreeCalcTableUpdateScenarioKind,
    },
    RenameNode {
        node_id: TreeNodeId,
    },
    MoveNode {
        node_id: TreeNodeId,
    },
    ReorderNode {
        node_id: TreeNodeId,
    },
    DeleteNode {
        node_id: TreeNodeId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OxCalcTreeInvalidationPlanEntry {
    pub node_id: TreeNodeId,
    pub requires_rebind: bool,
    pub reasons: Vec<InvalidationReasonKind>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OxCalcTreeInvalidationPlan {
    pub invalidated_nodes: Vec<OxCalcTreeInvalidationPlanEntry>,
    pub evaluation_order: Vec<TreeNodeId>,
    pub requires_rebind: Vec<TreeNodeId>,
    pub estimated_node_count: usize,
    pub cycle_risk: Vec<Vec<TreeNodeId>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OxCalcTreeNodeView {
    pub node_id: TreeNodeId,
    pub symbol: String,
    pub parent_node_id: Option<TreeNodeId>,
    pub child_node_ids: Vec<TreeNodeId>,
    pub canonical_path: String,
    pub display_path: String,
    pub formula_text: String,
    pub value_text: Option<String>,
    pub calc_value: Option<CalcValue>,
    pub input_value_epoch: Option<u64>,
    pub published_value_epoch: Option<u64>,
    pub calc_state: Option<NodeCalcState>,
    pub is_meta: bool,
    pub table: Option<OxCalcTreeTableView>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OxCalcTreeTableView {
    pub table_node_id: TreeNodeId,
    pub table_id: String,
    pub table_name: String,
    pub display_path: String,
    pub canonical_path: String,
    pub snapshot: TreeCalcTableNodeSnapshot,
    pub projection: TreeCalcTableNodeProjection,
    pub dependency_inventory: TreeCalcTableDependencyInventory,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OxCalcTreeWorkspaceView {
    pub workspace_id: OxCalcTreeWorkspaceId,
    pub root_node_id: TreeNodeId,
    pub snapshot_id: StructuralSnapshotId,
    pub workspace_revision_id: WorkspaceRevisionId,
    pub workspace_revision_parent_id: Option<WorkspaceRevisionId>,
    pub retained_workspace_revision_count: usize,
    pub workspace_revision_graph_entries: Vec<WorkspaceRevisionGraphEntry>,
    pub node_input_snapshot_id: NodeInputSnapshotId,
    pub namespace_snapshot_id: NamespaceSnapshotId,
    pub formula_binding_snapshot_id: FormulaBindingSnapshotId,
    pub dependency_shape_snapshot_id: DependencyShapeSnapshotId,
    pub publication_snapshot_id: PublicationSnapshotId,
    pub runtime_overlay_set_id: RuntimeOverlaySetId,
    pub value_epoch: u64,
    pub nodes: Vec<OxCalcTreeNodeView>,
    pub tables: Vec<OxCalcTreeTableView>,
    pub diagnostics: Vec<String>,
}

// Not `Eq`: `publication_values` now carries structured `SnapshotCalcValue`s
// whose `Number(f64)` variant is only `PartialEq`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OxCalcTreeWorkspaceSnapshot {
    pub schema_version: String,
    pub workspace_id: OxCalcTreeWorkspaceId,
    pub root_node_id: TreeNodeId,
    pub workspace_revision: WorkspaceRevision,
    pub formula_binding_snapshot: FormulaBindingSnapshot,
    pub dependency_shape_snapshot: DependencyShapeSnapshot,
    pub publication_snapshot: PublicationSnapshot,
    pub runtime_overlay_set: RuntimeOverlaySet,
    pub input_epoch_watermark: u64,
    #[serde(default)]
    pub publication_value_epoch_watermark: u64,
    pub table_snapshots: BTreeMap<TreeNodeId, TreeCalcTableNodeSnapshot>,
    pub deleted_table_facts: Vec<TreeCalcTableDeletedFact>,
    pub table_state_version: u64,
    pub publication_values: BTreeMap<TreeNodeId, SnapshotCalcValue>,
    #[serde(default)]
    pub publication_value_epochs: BTreeMap<TreeNodeId, u64>,
    pub publication_runtime_effects: Vec<RuntimeEffect>,
}

/// A serializable, faithful representation of a published [`CalcValue`] for the
/// workspace snapshot.
///
/// Earlier snapshots stored each published value as its display string (e.g. a
/// 10x10 array as the text `"Array(10x10)"`), which destroyed array data and
/// reconstructed it as a 1x1 text literal on import — collapsing dynamic-array
/// results (`SEQUENCE`, `MAP`, spills) after a persist/reload. This type
/// captures scalars and (nested) arrays exactly. Reference values — which never
/// appear as published results — and rich presentation metadata are not
/// represented; a reference falls back to its display text so the conversion is
/// total.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SnapshotCalcValue {
    Number(f64),
    Text(String),
    Logical(bool),
    /// A worksheet error, keyed by the stable variant token (see
    /// [`worksheet_error_code_token`]).
    Error(String),
    Empty,
    Missing,
    Array {
        rows: usize,
        cols: usize,
        /// Row-major, `rows * cols` cells.
        cells: Vec<SnapshotCalcValue>,
    },
    /// A value with no structured representation (e.g. a reference), preserved
    /// only as its display text.
    Opaque(String),
}

impl SnapshotCalcValue {
    /// Capture a computed value for serialization. Lossless for scalars and
    /// (nested) arrays; drops rich presentation metadata.
    fn from_calc_value(value: &CalcValue) -> Self {
        match &value.core {
            CoreValue::Number(number) => Self::Number(*number),
            CoreValue::Text(text) => Self::Text(text.to_string_lossy()),
            CoreValue::Logical(logical) => Self::Logical(*logical),
            CoreValue::Error(code) => Self::Error(worksheet_error_code_token(*code).to_string()),
            CoreValue::Empty => Self::Empty,
            CoreValue::Missing => Self::Missing,
            CoreValue::Array(array) => {
                let shape = array.shape();
                Self::Array {
                    rows: shape.rows,
                    cols: shape.cols,
                    cells: array.iter_row_major().map(Self::from_calc_value).collect(),
                }
            }
            CoreValue::Reference(_) => Self::Opaque(calc_value_display_text(value)),
        }
    }

    /// Reconstruct a computed value on import.
    fn to_calc_value(&self) -> CalcValue {
        match self {
            Self::Number(number) => CalcValue::number(*number),
            Self::Text(text) => CalcValue::text(ExcelText::from_interop_assignment(text)),
            Self::Logical(logical) => CalcValue::logical(*logical),
            Self::Error(token) => CalcValue::error(worksheet_error_code_from_token(token)),
            Self::Empty => CalcValue::empty(),
            Self::Missing => CalcValue::new(CoreValue::Missing),
            Self::Array { rows, cols, cells } => {
                let calc_cells: Vec<CalcValue> = cells.iter().map(Self::to_calc_value).collect();
                CalcArray::new(
                    ArrayShape {
                        rows: *rows,
                        cols: *cols,
                    },
                    calc_cells,
                )
                .map_or_else(
                    // A malformed array (shape/cell-count mismatch) cannot occur
                    // for our own output; degrade to a #VALUE! rather than panic.
                    || CalcValue::error(WorksheetErrorCode::Value),
                    |array| CalcValue::new(CoreValue::Array(array)),
                )
            }
            Self::Opaque(text) => CalcValue::text(ExcelText::from_interop_assignment(text)),
        }
    }
}

/// Stable, round-trip-safe token for a worksheet error code. Internal to the
/// snapshot wire form — not a user-facing display string.
fn worksheet_error_code_token(code: WorksheetErrorCode) -> &'static str {
    match code {
        WorksheetErrorCode::Null => "Null",
        WorksheetErrorCode::Div0 => "Div0",
        WorksheetErrorCode::Value => "Value",
        WorksheetErrorCode::Ref => "Ref",
        WorksheetErrorCode::Name => "Name",
        WorksheetErrorCode::Num => "Num",
        WorksheetErrorCode::NA => "NA",
        WorksheetErrorCode::Busy => "Busy",
        WorksheetErrorCode::GettingData => "GettingData",
        WorksheetErrorCode::Spill => "Spill",
        WorksheetErrorCode::Calc => "Calc",
        WorksheetErrorCode::Field => "Field",
        WorksheetErrorCode::Blocked => "Blocked",
        WorksheetErrorCode::Connect => "Connect",
    }
}

fn worksheet_error_code_from_token(token: &str) -> WorksheetErrorCode {
    match token {
        "Null" => WorksheetErrorCode::Null,
        "Div0" => WorksheetErrorCode::Div0,
        "Value" => WorksheetErrorCode::Value,
        "Ref" => WorksheetErrorCode::Ref,
        "Name" => WorksheetErrorCode::Name,
        "Num" => WorksheetErrorCode::Num,
        "NA" => WorksheetErrorCode::NA,
        "Busy" => WorksheetErrorCode::Busy,
        "GettingData" => WorksheetErrorCode::GettingData,
        "Spill" => WorksheetErrorCode::Spill,
        "Calc" => WorksheetErrorCode::Calc,
        "Field" => WorksheetErrorCode::Field,
        "Blocked" => WorksheetErrorCode::Blocked,
        "Connect" => WorksheetErrorCode::Connect,
        // An unknown token (forward-compat) degrades to a generic value error.
        _ => WorksheetErrorCode::Value,
    }
}

#[derive(Debug, Error)]
pub enum OxCalcTreeContextError {
    #[error("workspace '{workspace_id}' already exists")]
    DuplicateWorkspace { workspace_id: String },
    #[error("unknown workspace '{workspace_id}'")]
    UnknownWorkspace { workspace_id: String },
    #[error("node {node_id} has no parent and cannot be reordered")]
    CannotReorderRoot { node_id: TreeNodeId },
    #[error(transparent)]
    Structural(#[from] StructuralError),
    #[error(transparent)]
    Runtime(#[from] LocalTreeCalcError),
    #[error("invalid TreeCalc table snapshot: {error:?}")]
    TableProjection { error: TreeCalcTableProjectionError },
    #[error("grid backing engine error: {error:?}")]
    GridEngine { error: GridRefError },
    #[error(
        "invalid OxFml structured-reference bind record for TreeCalc table lowering: {error:?}"
    )]
    TableBindRecordIntake {
        error: StructuredTableBindRecordIntakeError,
    },
    #[error("invalid OxCalc tree workspace snapshot: {detail}")]
    InvalidWorkspaceSnapshot { detail: String },
    #[error("input value for node {node_id} may not be formula text")]
    InputValueIsFormula { node_id: TreeNodeId },
    #[error("authored input for node {node_id} was rejected by OxFml: {diagnostics:?}")]
    AuthoredInputDiagnostics {
        node_id: TreeNodeId,
        diagnostics: Vec<String>,
    },
    #[error("node {node_id} is formula-backed; use set_node_formula_text for formula changes")]
    InputValueOnFormulaNode { node_id: TreeNodeId },
    #[error("node {owner_node_id} has no reference collection '{source_reference_handle}'")]
    UnknownReferenceCollection {
        owner_node_id: TreeNodeId,
        source_reference_handle: String,
    },
    #[error(
        "reference collection '{source_reference_handle}' on node {owner_node_id} is derived and not editable: {family:?}"
    )]
    ReferenceCollectionNotEditable {
        owner_node_id: TreeNodeId,
        source_reference_handle: String,
        family: TreeReferenceCollectionFamily,
    },
    #[error("transaction {transaction_id} rejected during recalc: {diagnostics:?}")]
    TransactionRejected {
        transaction_id: OxCalcTreeTransactionId,
        diagnostics: Vec<String>,
    },
    #[error("workspace revision '{revision_id}' is not retained for workspace '{workspace_id:?}'")]
    WorkspaceRevisionNotRetained {
        workspace_id: OxCalcTreeWorkspaceId,
        revision_id: WorkspaceRevisionId,
    },
    #[error("candidate '{handle}' already exists")]
    DuplicateCandidate { handle: CandidateOverlayHandle },
    #[error("unknown candidate '{handle}'")]
    UnknownCandidate { handle: CandidateOverlayHandle },
    #[error(
        "candidate '{handle}' belongs to workspace '{candidate_workspace_id:?}', not '{transaction_workspace_id:?}'"
    )]
    CandidateWorkspaceMismatch {
        handle: CandidateOverlayHandle,
        candidate_workspace_id: OxCalcTreeWorkspaceId,
        transaction_workspace_id: OxCalcTreeWorkspaceId,
    },
    #[error("candidate parent '{parent_handle}' is not retained")]
    CandidateParentNotRetained {
        parent_handle: CandidateOverlayHandle,
    },
    #[error(
        "candidate parent '{parent_handle}' belongs to workspace '{parent_workspace_id}', not '{workspace_id}'"
    )]
    CandidateParentWorkspaceMismatch {
        parent_handle: CandidateOverlayHandle,
        parent_workspace_id: OxCalcTreeWorkspaceId,
        workspace_id: OxCalcTreeWorkspaceId,
    },
    #[error(
        "candidate parent '{parent_handle}' basis '{parent_basis_revision_id}' does not match requested basis '{basis_revision_id}'"
    )]
    CandidateParentBasisMismatch {
        parent_handle: CandidateOverlayHandle,
        parent_basis_revision_id: WorkspaceRevisionId,
        basis_revision_id: WorkspaceRevisionId,
    },
    #[error("candidate '{handle}' has retained child candidate '{child_handle}'")]
    CandidateHasRetainedChild {
        handle: CandidateOverlayHandle,
        child_handle: CandidateOverlayHandle,
    },
    #[error(
        "candidate '{handle}' cannot be rebased while it has retained child candidate '{child_handle}'"
    )]
    CandidateRebaseHasRetainedChild {
        handle: CandidateOverlayHandle,
        child_handle: CandidateOverlayHandle,
    },
    #[error("candidate '{handle}' has no host retention pin to release")]
    CandidateRetentionPinNotHeld { handle: CandidateOverlayHandle },
    #[error(
        "candidate '{handle}' basis '{basis_revision_id}' is not the current workspace revision '{current_revision_id}'"
    )]
    CandidateBasisNotCurrent {
        handle: CandidateOverlayHandle,
        basis_revision_id: WorkspaceRevisionId,
        current_revision_id: WorkspaceRevisionId,
    },
    #[error(
        "candidate '{handle}' rebase from '{basis_revision_id}' to '{current_revision_id}' conflicts on nodes {overlapping_nodes:?}"
    )]
    CandidateRebaseConflict {
        handle: CandidateOverlayHandle,
        basis_revision_id: WorkspaceRevisionId,
        current_revision_id: WorkspaceRevisionId,
        overlapping_nodes: Vec<TreeNodeId>,
        report: OxCalcTreeCandidateRebaseConflictReport,
    },
    #[error(transparent)]
    WorkspaceRevision(#[from] WorkspaceRevisionError),
    #[error(transparent)]
    WorkspaceRevisionNavigation(#[from] WorkspaceRevisionGraphNavigationError),
    #[error(transparent)]
    WorkspaceRevisionEviction(#[from] WorkspaceRevisionGraphEvictionError),
}

// Heavy payloads are `Arc`-shared between the live workspace state, retained
// revision entries, and the last calculation outcome, so revision retention
// is O(1) reference bumps instead of deep copies. In-place edits go through
// `Arc::make_mut` (copy-on-write), which keeps retained history immutable.
/// Seed for attaching a grid backing to a sheet node: the grid's coordinate
/// identity (workbook/sheet), its bounds, the authored cells to populate, and
/// any committed document-state overlays (structured tables, merged regions).
/// Spills are not seeded - they are a calc result, produced by recalculating the
/// authored cells.
#[derive(Debug, Clone)]
pub struct GridBackingSeed {
    pub workbook_id: String,
    pub sheet_id: String,
    pub bounds: ExcelGridBounds,
    pub authored: Vec<(ExcelGridCellAddress, GridAuthoredCell)>,
    /// Structured-table overlays to install as committed document state.
    pub table_overlays: Vec<GridTableOverlay>,
    /// Merged-region rectangles to install as committed document state.
    pub merged_regions: Vec<GridRect>,
}

/// A region-granular grid edit applied via `apply_grid_edit`.
#[derive(Debug, Clone)]
pub enum OxCalcTreeGridOp {
    /// Set a single cell's authored content (a literal or a formula).
    SetCell {
        address: ExcelGridCellAddress,
        cell: GridAuthoredCell,
    },
    /// Fill a rect with one repeated R1C1-relative formula - compiled to a
    /// single repeated-formula region, not N authored cells ("drag a formula
    /// over a range"). References shifting out of bounds resolve to `#REF!`.
    FillRange {
        rect: GridRect,
        formula: GridFormulaCell,
    },
}

/// A readout of a node's grid backing: the computed value of each requested
/// cell, plus any permanent-pair differential mismatches (empty when the
/// reference and optimized engines agree, which is the invariant).
#[derive(Debug, Clone)]
pub struct OxCalcTreeGridView {
    pub grid_node_id: TreeNodeId,
    pub grid_id: String,
    pub bounds: ExcelGridBounds,
    pub cells: Vec<OxCalcTreeGridCellReadout>,
    /// Read-only overlay descriptors (tables, spills, merged regions) clipped to
    /// the registered interest window.
    pub overlays: OxCalcTreeGridOverlays,
    /// Bumped whenever the window-clipped overlay set changes; a client pulls
    /// "overlays changed since" by comparing against it (independent of the cell
    /// value epochs).
    pub overlay_epoch: u64,
    pub differential_mismatches: Vec<GridDifferentialMismatch>,
}

/// The read-only overlay descriptors for a grid view, window-clipped to the
/// registered interest. Tables and merged regions are committed document state;
/// spills are a calc result of the recalc.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OxCalcTreeGridOverlays {
    pub tables: Vec<OxCalcTreeGridTableOverlayReadout>,
    pub spills: Vec<OxCalcTreeGridSpillOverlayReadout>,
    pub merged: Vec<OxCalcTreeGridMergedOverlayReadout>,
}

/// An overlay rectangle in absolute 1-based grid coordinates, clipped to the
/// interest window. Each `clipped_*` flag records whether that edge was cut by
/// the window (so a renderer can show a "continues beyond the window"
/// affordance rather than a hard border).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OxCalcTreeGridOverlayRect {
    pub top_row: u32,
    pub left_col: u32,
    pub bottom_row: u32,
    pub right_col: u32,
    pub clipped_top: bool,
    pub clipped_left: bool,
    pub clipped_bottom: bool,
    pub clipped_right: bool,
}

/// One column band of a table overlay, clipped to the interest window.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OxCalcTreeGridTableColumnBand {
    pub column_id: String,
    pub column_name: String,
    pub ordinal: u32,
    pub data_rect: OxCalcTreeGridOverlayRect,
}

/// A structured-table overlay descriptor (geometry + identity), clipped to the
/// interest window. Carries no row values - those are projected separately as a
/// shared table projection keyed by the table's node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OxCalcTreeGridTableOverlayReadout {
    pub table_id: String,
    pub table_name: String,
    pub table_range: OxCalcTreeGridOverlayRect,
    pub header_rect: Option<OxCalcTreeGridOverlayRect>,
    pub totals_rect: Option<OxCalcTreeGridOverlayRect>,
    pub columns: Vec<OxCalcTreeGridTableColumnBand>,
}

/// A spilled-array overlay descriptor: the (unclipped) anchor cell that produced
/// the spill, the window-clipped extent, and whether the spill is blocked
/// (`#SPILL!`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OxCalcTreeGridSpillOverlayReadout {
    pub anchor: ExcelGridCellAddress,
    pub extent: OxCalcTreeGridOverlayRect,
    pub blocked: bool,
}

/// A merged-region overlay descriptor, clipped to the interest window.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OxCalcTreeGridMergedOverlayReadout {
    pub rect: OxCalcTreeGridOverlayRect,
}

#[derive(Debug, Clone)]
pub struct OxCalcTreeGridCellReadout {
    pub address: ExcelGridCellAddress,
    pub value: CalcValue,
    /// The recalc epoch at which this cell's value last changed; a client can
    /// pull "changes since epoch E" by comparing against it.
    pub value_epoch: u64,
}

/// A client's regions of interest in a grid: the visible viewport plus any
/// off-screen monitored rects. Registering interest scopes what `grid_view`
/// returns and what `poll_grid_changes` reports. On the backing, `None` interest
/// means the whole grid; an empty `GridInterestRegions` means nothing.
#[derive(Debug, Clone, Default)]
pub struct GridInterestRegions {
    pub viewport: Option<GridRect>,
    pub monitored: Vec<GridRect>,
}

impl GridInterestRegions {
    fn contains(&self, address: &ExcelGridCellAddress) -> bool {
        self.viewport
            .as_ref()
            .is_some_and(|rect| rect.contains(address))
            || self.monitored.iter().any(|rect| rect.contains(address))
    }
}

/// A grid recalc epoch a client holds to pull "changes since" via
/// `poll_grid_changes`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct GridInterestEpoch(pub u64);

/// The scoped result of `poll_grid_changes`: the interested cells whose value
/// changed after `from_epoch`, up to `to_epoch`. `resync` is set when the
/// caller's `since` is incoherent with the backing (e.g. the grid was
/// recreated), in which case `changed` is the full current interested readout.
#[derive(Debug, Clone)]
pub struct GridDeltaPacket {
    pub grid_node_id: TreeNodeId,
    pub from_epoch: GridInterestEpoch,
    pub to_epoch: GridInterestEpoch,
    pub resync: bool,
    pub changed: Vec<OxCalcTreeGridCellReadout>,
}

/// A published grid cell: its computed value and the recalc epoch at which the
/// value last changed.
#[derive(Debug, Clone)]
struct GridPublishedCell {
    value: CalcValue,
    value_epoch: u64,
}

/// Per-node grid backing held in the workspace state: one optimized engine plus
/// its cached publication (computed values + per-cell value epochs), recomputed
/// on edit by `recalc` and read cheaply by `grid_view`. The reference twin is
/// derived on demand by `run_engine_mode_with_oxfml`, so the differential is
/// checked at recalc time and its result cached.
#[derive(Debug, Clone)]
struct GridBackingState {
    grid_id: String,
    authored_addresses: BTreeSet<ExcelGridCellAddress>,
    sheet: GridOptimizedSheet,
    published: BTreeMap<ExcelGridCellAddress, GridPublishedCell>,
    differential_mismatches: Vec<GridDifferentialMismatch>,
    recalc_epoch: u64,
    /// Regions the client is interested in. `None` = the whole grid; `Some`
    /// scopes the cached publication (and thus reads and deltas) to those rects.
    interest: Option<GridInterestRegions>,
    /// The window-clipped overlay descriptors, refreshed on recalc (parallel to
    /// `published` for cells).
    published_overlays: OxCalcTreeGridOverlays,
    /// Bumped on recalc whenever `published_overlays` changes; lets a client pull
    /// "overlays changed since" independently of cell value epochs.
    overlay_epoch: u64,
}

impl GridBackingState {
    /// Recalculate the backing (both engines, for the differential), refresh the
    /// cached publication, and bump the value epoch of each cell whose value
    /// changed. Cheap reads then come from `published` without re-running the
    /// engines.
    fn recalc(&mut self) -> Result<(), GridRefError> {
        let probes = match &self.interest {
            None => self.authored_addresses.iter().cloned().collect::<Vec<_>>(),
            Some(regions) => self
                .authored_addresses
                .iter()
                .filter(|address| regions.contains(address))
                .cloned()
                .collect::<Vec<_>>(),
        };
        let report = self.sheet.run_engine_mode_with_oxfml(
            GridEngineMode::Both,
            probes,
            GRID_CALC_REF_DEFAULT_MATERIALIZATION_LIMIT,
        )?;
        self.recalc_epoch += 1;
        let epoch = self.recalc_epoch;
        let readout = report
            .optimized
            .as_ref()
            .map(|run| run.readout.clone())
            .unwrap_or_default();
        let spill_facts = report
            .optimized
            .as_ref()
            .map(|run| run.spill_facts.clone())
            .unwrap_or_default();
        let mut published = BTreeMap::new();
        for cell in readout {
            // Reuse the prior epoch when the value is unchanged; otherwise stamp
            // this recalc's epoch.
            let unchanged_epoch = self
                .published
                .get(&cell.address)
                .filter(|prev| prev.value == cell.computed)
                .map(|prev| prev.value_epoch);
            published.insert(
                cell.address,
                GridPublishedCell {
                    value: cell.computed,
                    value_epoch: unchanged_epoch.unwrap_or(epoch),
                },
            );
        }
        self.published = published;
        self.differential_mismatches = report.mismatches;
        // Refresh the window-clipped overlay descriptors. Tables and merged
        // regions are committed document state read off the sheet; spills are the
        // calc result of this recalc. Bump the overlay epoch only when the
        // projected set actually changes, so an overlay-only consumer is not
        // woken by value-only recalcs (and vice versa).
        let overlays = project_grid_overlays(&self.sheet, &spill_facts, &self.interest);
        if overlays != self.published_overlays {
            self.overlay_epoch += 1;
            self.published_overlays = overlays;
        }
        Ok(())
    }
}

/// The interest window for overlay clipping: `WholeGrid` (no clipping) for an
/// unscoped backing, the bounding rectangle of the registered interest regions,
/// or `Empty` when interest is registered but covers nothing.
///
/// Note: overlays clip to the *bounding box* of the interest, whereas cells use
/// exact per-rect membership ([`GridInterestRegions::contains`]). With disjoint
/// interest rects an overlay sitting in the gap can surface even where no cell
/// renders - intentional: overlay descriptors are coarse adornments, and a
/// bounding box keeps a single overlay from fragmenting across windows.
enum OverlayWindow {
    WholeGrid,
    Rect {
        top_row: u32,
        left_col: u32,
        bottom_row: u32,
        right_col: u32,
    },
    Empty,
}

fn overlay_window(interest: &Option<GridInterestRegions>) -> OverlayWindow {
    let Some(regions) = interest else {
        return OverlayWindow::WholeGrid;
    };
    let mut bounds: Option<(u32, u32, u32, u32)> = None;
    let mut absorb = |rect: &GridRect| {
        bounds = Some(match bounds {
            None => (rect.top_row, rect.left_col, rect.bottom_row, rect.right_col),
            Some((top, left, bottom, right)) => (
                top.min(rect.top_row),
                left.min(rect.left_col),
                bottom.max(rect.bottom_row),
                right.max(rect.right_col),
            ),
        });
    };
    if let Some(viewport) = &regions.viewport {
        absorb(viewport);
    }
    for rect in &regions.monitored {
        absorb(rect);
    }
    match bounds {
        Some((top_row, left_col, bottom_row, right_col)) => OverlayWindow::Rect {
            top_row,
            left_col,
            bottom_row,
            right_col,
        },
        None => OverlayWindow::Empty,
    }
}

/// Clip an overlay rectangle to the interest window, returning `None` when the
/// rectangle lies entirely outside the window. Each `clipped_*` flag records
/// whether the window cut that edge.
fn clip_overlay_rect(rect: &GridRect, window: &OverlayWindow) -> Option<OxCalcTreeGridOverlayRect> {
    match window {
        OverlayWindow::Empty => None,
        OverlayWindow::WholeGrid => Some(OxCalcTreeGridOverlayRect {
            top_row: rect.top_row,
            left_col: rect.left_col,
            bottom_row: rect.bottom_row,
            right_col: rect.right_col,
            clipped_top: false,
            clipped_left: false,
            clipped_bottom: false,
            clipped_right: false,
        }),
        OverlayWindow::Rect {
            top_row,
            left_col,
            bottom_row,
            right_col,
        } => {
            let top = rect.top_row.max(*top_row);
            let left = rect.left_col.max(*left_col);
            let bottom = rect.bottom_row.min(*bottom_row);
            let right = rect.right_col.min(*right_col);
            if top > bottom || left > right {
                return None;
            }
            Some(OxCalcTreeGridOverlayRect {
                top_row: top,
                left_col: left,
                bottom_row: bottom,
                right_col: right,
                clipped_top: top > rect.top_row,
                clipped_left: left > rect.left_col,
                clipped_bottom: bottom < rect.bottom_row,
                clipped_right: right < rect.right_col,
            })
        }
    }
}

/// Project the sheet's committed table/merged overlays plus this recalc's spill
/// facts into window-clipped, read-only overlay descriptors.
fn project_grid_overlays(
    sheet: &GridOptimizedSheet,
    spill_facts: &[GridSpillFact],
    interest: &Option<GridInterestRegions>,
) -> OxCalcTreeGridOverlays {
    let window = overlay_window(interest);
    let mut tables = Vec::new();
    for table in sheet.table_overlays().values() {
        let Some(table_range) = clip_overlay_rect(&table.table_range, &window) else {
            continue;
        };
        let header_rect = table
            .header_rect
            .as_ref()
            .and_then(|rect| clip_overlay_rect(rect, &window));
        let totals_rect = table
            .totals_rect
            .as_ref()
            .and_then(|rect| clip_overlay_rect(rect, &window));
        let columns = table
            .columns
            .iter()
            .filter_map(|column| {
                clip_overlay_rect(&column.data_rect, &window).map(|data_rect| {
                    OxCalcTreeGridTableColumnBand {
                        column_id: column.column_id.clone(),
                        column_name: column.column_name.clone(),
                        ordinal: column.ordinal,
                        data_rect,
                    }
                })
            })
            .collect();
        tables.push(OxCalcTreeGridTableOverlayReadout {
            table_id: table.table_id.clone(),
            table_name: table.table_name.clone(),
            table_range,
            header_rect,
            totals_rect,
            columns,
        });
    }
    let mut spills = Vec::new();
    for fact in spill_facts {
        let Some(extent) = clip_overlay_rect(&fact.extent, &window) else {
            continue;
        };
        spills.push(OxCalcTreeGridSpillOverlayReadout {
            anchor: fact.anchor.clone(),
            extent,
            blocked: fact.blocked,
        });
    }
    let mut merged = Vec::new();
    for region in sheet.merged_regions() {
        let Some(rect) = clip_overlay_rect(&region.rect, &window) else {
            continue;
        };
        merged.push(OxCalcTreeGridMergedOverlayReadout { rect });
    }
    OxCalcTreeGridOverlays {
        tables,
        spills,
        merged,
    }
}

#[derive(Debug, Clone)]
struct OxCalcTreeWorkspaceState {
    workspace_id: OxCalcTreeWorkspaceId,
    root_node_id: TreeNodeId,
    snapshot: Arc<StructuralSnapshot>,
    workspace_revision: Arc<WorkspaceRevision>,
    workspace_revision_graph: WorkspaceRevisionGraph,
    retained_workspace_revisions: BTreeMap<WorkspaceRevisionId, RetainedWorkspaceRevisionState>,
    retained_workspace_revision_order: VecDeque<WorkspaceRevisionId>,
    candidate_pinned_workspace_revisions: BTreeMap<WorkspaceRevisionId, usize>,
    revision_retention_policy: OxCalcTreeRevisionRetentionPolicy,
    formula_binding_snapshot: FormulaBindingSnapshot,
    dependency_shape_snapshot: DependencyShapeSnapshot,
    publication_snapshot: Arc<PublicationSnapshot>,
    runtime_overlay_set: Arc<RuntimeOverlaySet>,
    value_epoch: u64,
    publication_value_epoch: u64,
    table_snapshots: Arc<BTreeMap<TreeNodeId, TreeCalcTableNodeSnapshot>>,
    deleted_table_facts: Vec<TreeCalcTableDeletedFact>,
    table_state_version: u64,
    // Per-node grid backings. Held only in the live state for now; per-revision
    // retention (undo/redo of grid edits) lands with the grid edit verbs.
    grids: Arc<BTreeMap<TreeNodeId, GridBackingState>>,
    grid_state_version: u64,
    publication_payload: Arc<PublishedRuntimeLayerPayload>,
    pending_invalidation_seeds: Vec<InvalidationSeed>,
    pending_formula_edit_diagnostics: Vec<String>,
    pending_node_input_kind_transitions: Vec<ContextNodeInputKindTransition>,
    pending_dependency_shape_updates: Vec<DependencyShapeUpdate>,
    last_result: Option<Arc<OxCalcTreeCalculationOutcome>>,
    // Cache of prepared formulas keyed to their full prepare basis; reuse
    // is equality-gated in the engine, so this only affects speed.
    prepared_formula_retention: PreparedFormulaRetention,
}

#[derive(Debug, Clone)]
struct RetainedWorkspaceRevisionState {
    root_node_id: TreeNodeId,
    snapshot: Arc<StructuralSnapshot>,
    workspace_revision: Arc<WorkspaceRevision>,
    formula_binding_snapshot: FormulaBindingSnapshot,
    dependency_shape_snapshot: DependencyShapeSnapshot,
    publication_snapshot: Arc<PublicationSnapshot>,
    runtime_overlay_set: Arc<RuntimeOverlaySet>,
    value_epoch: u64,
    publication_value_epoch: u64,
    table_snapshots: Arc<BTreeMap<TreeNodeId, TreeCalcTableNodeSnapshot>>,
    deleted_table_facts: Vec<TreeCalcTableDeletedFact>,
    table_state_version: u64,
    publication_payload: Arc<PublishedRuntimeLayerPayload>,
    last_result: Option<Arc<OxCalcTreeCalculationOutcome>>,
}

impl OxCalcTreeWorkspaceState {
    // Copy-on-write accessors: clone the shared payload only when a retained
    // revision (or candidate) still points at it, so history stays immutable.
    fn publication_payload_mut(&mut self) -> &mut PublishedRuntimeLayerPayload {
        Arc::make_mut(&mut self.publication_payload)
    }

    // Equivalent to `publication_payload_mut().clear()` without the
    // copy-on-write clone of the payload that is about to be emptied.
    fn clear_publication_payload(&mut self) {
        self.publication_payload = Arc::new(PublishedRuntimeLayerPayload::default());
    }

    fn table_snapshots_mut(&mut self) -> &mut BTreeMap<TreeNodeId, TreeCalcTableNodeSnapshot> {
        Arc::make_mut(&mut self.table_snapshots)
    }

    fn grids_mut(&mut self) -> &mut BTreeMap<TreeNodeId, GridBackingState> {
        Arc::make_mut(&mut self.grids)
    }
}

#[derive(Debug, Clone)]
struct CandidateOverlayState {
    handle: CandidateOverlayHandle,
    workspace_id: OxCalcTreeWorkspaceId,
    basis_revision_id: WorkspaceRevisionId,
    parent_candidate: Option<CandidateOverlayHandle>,
    retention_pin_count: usize,
    value_epoch_basis: u64,
    publication_value_epoch_basis: u64,
    parent_private_edit_count: usize,
    private_edit_transactions: Vec<OxCalcTreeEditTransaction>,
    workspace_state: OxCalcTreeWorkspaceState,
}

#[derive(Debug, Clone, Default)]
struct PublishedRuntimeLayerPayload {
    // Payload for the publication/runtime layer, not authored node-input truth.
    values_by_node: BTreeMap<TreeNodeId, CalcValue>,
    value_epochs_by_node: BTreeMap<TreeNodeId, u64>,
    runtime_effects: Vec<RuntimeEffect>,
}

impl PublishedRuntimeLayerPayload {
    fn new(
        values_by_node: BTreeMap<TreeNodeId, CalcValue>,
        value_epochs_by_node: BTreeMap<TreeNodeId, u64>,
        runtime_effects: Vec<RuntimeEffect>,
    ) -> Self {
        Self {
            values_by_node,
            value_epochs_by_node,
            runtime_effects,
        }
    }

    fn is_empty(&self) -> bool {
        self.values_by_node.is_empty() && self.runtime_effects.is_empty()
    }

    fn clear(&mut self) {
        self.values_by_node.clear();
        self.value_epochs_by_node.clear();
        self.runtime_effects.clear();
    }
}

#[derive(Debug, Clone)]
pub struct OxCalcTreeContext {
    options: OxCalcTreeContextOptions,
    workspaces: BTreeMap<OxCalcTreeWorkspaceId, OxCalcTreeWorkspaceState>,
    candidates: BTreeMap<CandidateOverlayHandle, CandidateOverlayState>,
    next_node_id: u64,
    next_snapshot_id: u64,
    next_candidate_index: u64,
    next_candidate_overlay_index: u64,
    next_transaction_index: u64,
}

impl Default for OxCalcTreeContext {
    fn default() -> Self {
        Self::new(OxCalcTreeContextOptions::default())
    }
}

impl OxCalcTreeContext {
    #[must_use]
    pub fn new(options: OxCalcTreeContextOptions) -> Self {
        Self {
            options,
            workspaces: BTreeMap::new(),
            candidates: BTreeMap::new(),
            next_node_id: 1,
            next_snapshot_id: 1,
            next_candidate_index: 1,
            next_candidate_overlay_index: 1,
            next_transaction_index: 1,
        }
    }

    #[must_use]
    pub fn options(&self) -> &OxCalcTreeContextOptions {
        &self.options
    }

    pub fn set_options(&mut self, options: OxCalcTreeContextOptions) {
        self.options = options;
        let options = self.options.clone();
        for state in self.workspaces.values_mut() {
            state.revision_retention_policy = options.revision_retention_policy;
            let namespace_snapshot = namespace_snapshot_for_context(&options, &state.workspace_id);
            if namespace_snapshot.snapshot_id()
                != state.workspace_revision.namespace_snapshot.snapshot_id()
            {
                let has_published_baseline = !state.publication_payload.is_empty();
                replace_namespace_snapshot(state, namespace_snapshot);
                state.pending_invalidation_seeds.clear();
                if has_published_baseline {
                    seed_namespace_recalc_invalidation(state);
                }
                clear_pending_edit_transition_facts(state);
            }
            state.last_result = None;
            enforce_workspace_revision_retention_policy(state);
        }
    }

    pub fn create_workspace(
        &mut self,
        request: OxCalcTreeWorkspaceCreate,
    ) -> Result<OxCalcTreeWorkspaceId, OxCalcTreeContextError> {
        if self.workspaces.contains_key(&request.workspace_id) {
            return Err(OxCalcTreeContextError::DuplicateWorkspace {
                workspace_id: request.workspace_id.0,
            });
        }

        let root_node_id = self.next_node_id();
        let snapshot_id = self.next_snapshot_id();
        let root = StructuralNode {
            node_id: root_node_id,
            kind: StructuralNodeKind::Root,
            symbol: request.root_symbol,
            parent_id: None,
            child_ids: Vec::new(),
            role: None,
            is_meta: false,
        };
        let snapshot = StructuralSnapshot::create(snapshot_id, root_node_id, [root])?;
        let workspace_id = request.workspace_id;
        let node_input_snapshot =
            NodeInputSnapshot::create([NodeInputRecord::empty(root_node_id, 1)])?;
        let namespace_snapshot = namespace_snapshot_for_context(&self.options, &workspace_id);
        let workspace_revision = WorkspaceRevision::new(
            workspace_id.as_str(),
            snapshot.clone(),
            node_input_snapshot,
            namespace_snapshot,
        );
        let formula_binding_snapshot = FormulaBindingSnapshot::current_absent(
            workspace_revision.revision_id(),
            "w057.2-formula-binding-not-yet-promoted",
        );
        let dependency_shape_snapshot = DependencyShapeSnapshot::current_absent(
            workspace_revision.revision_id(),
            formula_binding_snapshot.snapshot_id(),
            "w057.2-dependency-shape-not-yet-promoted",
        );
        let publication_snapshot = PublicationSnapshot::current_absent(
            workspace_revision.revision_id(),
            "w057.2-publication-not-yet-promoted",
        );
        let runtime_overlay_set = RuntimeOverlaySet::current_absent(
            publication_snapshot.snapshot_id(),
            "w057.2-runtime-overlays-not-yet-promoted",
        );
        let workspace_revision_graph = WorkspaceRevisionGraph::initial(&workspace_revision);
        let state = OxCalcTreeWorkspaceState {
            workspace_id: workspace_id.clone(),
            root_node_id,
            snapshot: Arc::new(snapshot),
            workspace_revision: Arc::new(workspace_revision),
            workspace_revision_graph,
            retained_workspace_revisions: BTreeMap::new(),
            retained_workspace_revision_order: VecDeque::new(),
            candidate_pinned_workspace_revisions: BTreeMap::new(),
            revision_retention_policy: self.options.revision_retention_policy,
            formula_binding_snapshot,
            dependency_shape_snapshot,
            publication_snapshot: Arc::new(publication_snapshot),
            runtime_overlay_set: Arc::new(runtime_overlay_set),
            value_epoch: 0,
            publication_value_epoch: 0,
            table_snapshots: Arc::new(BTreeMap::new()),
            deleted_table_facts: Vec::new(),
            table_state_version: 1,
            grids: Arc::new(BTreeMap::new()),
            grid_state_version: 1,
            publication_payload: Arc::new(PublishedRuntimeLayerPayload::default()),
            pending_invalidation_seeds: Vec::new(),
            pending_formula_edit_diagnostics: Vec::new(),
            pending_node_input_kind_transitions: Vec::new(),
            pending_dependency_shape_updates: Vec::new(),
            last_result: None,
            prepared_formula_retention: PreparedFormulaRetention::default(),
        };
        let mut state = state;
        retain_current_workspace_revision(&mut state);
        self.workspaces.insert(workspace_id.clone(), state);
        self.advance_node_id();
        self.advance_snapshot_id();
        Ok(workspace_id)
    }

    pub fn add_node(
        &mut self,
        workspace_id: &OxCalcTreeWorkspaceId,
        request: OxCalcTreeNodeCreate,
    ) -> Result<TreeNodeId, OxCalcTreeContextError> {
        let node_id = request
            .reserved_node_id
            .unwrap_or_else(|| self.next_node_id());
        let snapshot_id = self.next_snapshot_id();
        {
            let state = self.workspace_mut(workspace_id)?;
            if state.snapshot.try_get_node(node_id).is_some() {
                return Err(OxCalcTreeContextError::Structural(
                    StructuralError::DuplicateNodeId {
                        snapshot_id: state.snapshot.snapshot_id(),
                        node_id,
                    },
                ));
            }
            let parent_id = request.parent_node_id.unwrap_or(state.root_node_id);
            let formula_text = request.formula_text;
            let node = StructuralNode {
                node_id,
                kind: node_kind_for_formula_text(&formula_text),
                symbol: request.symbol,
                parent_id: Some(parent_id),
                child_ids: Vec::new(),
                role: None,
                // Creation-time meta is an authored structural fact: it rides
                // on the inserted node and is validated by the same build path.
                is_meta: request.is_meta,
            };
            let outcome = state.snapshot.apply_edit(
                snapshot_id,
                StructuralEdit::InsertNode {
                    node,
                    parent_id,
                    index: None,
                },
            )?;
            state.snapshot = Arc::new(outcome.snapshot);
            let input_epoch = if constant_value_for_formula_text(&formula_text).is_some() {
                bump_input_value_epoch(state)
            } else {
                1
            };
            let input_record =
                direct_context_node_input_record(node_id, &formula_text, input_epoch);
            replace_node_input_record(state, input_record);
            state.pending_invalidation_seeds.clear();
            clear_pending_edit_transition_facts(state);
            state.clear_publication_payload();
            state.last_result = None;
        }
        self.next_node_id = self.next_node_id.max(node_id.0 + 1);
        self.advance_snapshot_id();
        Ok(node_id)
    }

    #[must_use]
    pub fn reserve_node_id(&mut self) -> TreeNodeId {
        let node_id = self.next_node_id();
        self.advance_node_id();
        node_id
    }

    pub fn set_node_meta(
        &mut self,
        workspace_id: &OxCalcTreeWorkspaceId,
        node_id: TreeNodeId,
        is_meta: bool,
    ) -> Result<(), OxCalcTreeContextError> {
        // Meta membership is a structural fact carried on the node, so a
        // membership change is an ordinary structural edit: it mints a new
        // structural snapshot id and therefore changes workspace revision
        // identity directly, with no side set and no namespace-version hack.
        let snapshot_id = self.next_snapshot_id();
        {
            let state = self.workspace_mut(workspace_id)?;
            let outcome = state.snapshot.apply_edit(
                snapshot_id,
                StructuralEdit::SetNodeMeta { node_id, is_meta },
            )?;
            state.snapshot = Arc::new(outcome.snapshot);
            refresh_workspace_revision_and_absent_layers(state);
            state.pending_invalidation_seeds.clear();
            clear_pending_edit_transition_facts(state);
            state.clear_publication_payload();
            state.last_result = None;
        }
        self.advance_snapshot_id();
        Ok(())
    }

    pub fn set_node_formula_text(
        &mut self,
        workspace_id: &OxCalcTreeWorkspaceId,
        node_id: TreeNodeId,
        formula_text: impl Into<String>,
    ) -> Result<(), OxCalcTreeContextError> {
        let formula_text = formula_text.into();
        let options = self.options.clone();
        {
            let state = self.workspace_mut(workspace_id)?;
            state
                .snapshot
                .try_get_node(node_id)
                .ok_or(StructuralError::UnknownNode { node_id })?;
            let predecessor_input_record = state
                .workspace_revision
                .node_input_snapshot
                .try_get_record(node_id)
                .cloned()
                .ok_or_else(|| OxCalcTreeContextError::InvalidWorkspaceSnapshot {
                    detail: format!("node input snapshot is missing record for {node_id}"),
                })?;
            let predecessor_input_kind = predecessor_input_record.kind;
            let successor_input_kind = node_input_kind_for_formula_edit_text(&formula_text);
            if successor_input_kind == NodeInputKind::FormulaText {
                match interpret_authored_input_text(&formula_text) {
                    RuntimeAuthoredInputResult::Formula(_) => {}
                    RuntimeAuthoredInputResult::Literal(_) => {}
                    RuntimeAuthoredInputResult::Diagnostics(diagnostics) => {
                        return Err(OxCalcTreeContextError::AuthoredInputDiagnostics {
                            node_id,
                            diagnostics: authored_input_diagnostics_to_strings(diagnostics),
                        });
                    }
                }
            }
            let predecessor_is_formula = predecessor_input_kind == NodeInputKind::FormulaText;
            let successor_is_formula = successor_input_kind == NodeInputKind::FormulaText;

            if !predecessor_is_formula && !successor_is_formula {
                let input_record = successor_non_formula_input_record(state, node_id, formula_text);
                replace_non_formula_node_input_record(state, input_record);
                state
                    .publication_payload_mut()
                    .values_by_node
                    .remove(&node_id);
                push_pending_invalidation_seed(
                    state,
                    node_id,
                    InvalidationReasonKind::UpstreamPublication,
                );
                state.last_result = None;
                return Ok(());
            }

            if predecessor_is_formula || successor_is_formula {
                let predecessor_build = build_context_formula_catalog(state, &options)?;
                let predecessor_unresolved =
                    context_formula_catalog_has_unresolved(node_id, &predecessor_build.diagnostics);
                let predecessor_literal_value = (!predecessor_is_formula)
                    .then(|| current_literal_value_for_node(state, node_id))
                    .flatten();
                let successor_formula_epoch = predecessor_input_record.input_epoch + 1;
                let successor_input_record = if successor_input_kind == NodeInputKind::FormulaText {
                    if let Some(value) = predecessor_literal_value {
                        state
                            .publication_payload_mut()
                            .values_by_node
                            .insert(node_id, value);
                    }
                    NodeInputRecord::formula_text(
                        node_id,
                        formula_text.clone(),
                        successor_formula_epoch,
                    )
                } else {
                    state
                        .publication_payload_mut()
                        .values_by_node
                        .remove(&node_id);
                    successor_non_formula_input_record(state, node_id, formula_text.clone())
                };
                if successor_input_record.kind == NodeInputKind::FormulaText {
                    replace_formula_text_node_input_record(state, successor_input_record);
                } else {
                    replace_non_formula_node_input_record(state, successor_input_record);
                }
                let successor_build = build_context_formula_catalog(state, &options)?;
                state.prepared_formula_retention =
                    successor_build.prepared_formula_retention.clone();
                let successor_unresolved =
                    context_formula_catalog_has_unresolved(node_id, &successor_build.diagnostics);
                let classification = classify_context_formula_edit(
                    &state.snapshot,
                    node_id,
                    &predecessor_build.dependency_descriptors,
                    &successor_build.dependency_descriptors,
                    ContextFormulaEditTransition {
                        predecessor_formula_present: predecessor_is_formula,
                        successor_formula_present: successor_is_formula,
                        predecessor_unresolved,
                        successor_unresolved,
                    },
                );
                state.pending_formula_edit_diagnostics.push(format!(
                    "formula_edit_classification:{node_id}:{}",
                    classification.label
                ));
                state
                    .pending_node_input_kind_transitions
                    .push(ContextNodeInputKindTransition {
                        node_id,
                        predecessor_kind: predecessor_input_kind,
                        successor_kind: successor_input_kind,
                    });
                if let Some(update) = dependency_shape_update_for_formula_edit(&classification) {
                    state.pending_dependency_shape_updates.push(update);
                }
                push_pending_invalidation_seed(
                    state,
                    node_id,
                    if successor_is_formula {
                        InvalidationReasonKind::StructuralRecalcOnly
                    } else {
                        InvalidationReasonKind::UpstreamPublication
                    },
                );
                state.last_result = None;
                return Ok(());
            }
        }
        Ok(())
    }

    pub fn set_node_input_value(
        &mut self,
        workspace_id: &OxCalcTreeWorkspaceId,
        node_id: TreeNodeId,
        input_value: impl Into<String>,
    ) -> Result<(), OxCalcTreeContextError> {
        let input_value = input_value.into();
        match interpret_authored_input_text(&input_value) {
            RuntimeAuthoredInputResult::Literal(_) => {
                self.set_node_non_formula_input_value(workspace_id, node_id, input_value)
            }
            RuntimeAuthoredInputResult::Formula(_) => {
                self.set_node_formula_text(workspace_id, node_id, input_value)
            }
            RuntimeAuthoredInputResult::Diagnostics(diagnostics) => {
                Err(OxCalcTreeContextError::AuthoredInputDiagnostics {
                    node_id,
                    diagnostics: authored_input_diagnostics_to_strings(diagnostics),
                })
            }
        }
    }

    pub fn apply_edit_transaction(
        &mut self,
        transaction: OxCalcTreeEditTransaction,
    ) -> Result<OxCalcTreeTransactionOutcome, OxCalcTreeContextError> {
        let transaction_id = self.next_transaction_id(&transaction.workspace_id);
        let pre_transaction = self.clone();
        let result = self.apply_edit_transaction_inner(transaction, transaction_id.clone());
        match result {
            Ok(outcome) => {
                self.advance_transaction_index();
                Ok(outcome)
            }
            Err(error) => {
                *self = pre_transaction;
                self.advance_transaction_index();
                Err(error)
            }
        }
    }

    pub fn open_candidate(
        &mut self,
        request: OxCalcTreeOpenCandidateRequest,
    ) -> Result<OxCalcTreeCandidateView, OxCalcTreeContextError> {
        let (basis_state, private_edit_transactions) =
            if let Some(parent_candidate) = &request.parent_candidate {
                let parent = self.candidates.get(parent_candidate).ok_or_else(|| {
                    OxCalcTreeContextError::CandidateParentNotRetained {
                        parent_handle: parent_candidate.clone(),
                    }
                })?;
                if parent.workspace_id != request.workspace_id {
                    return Err(OxCalcTreeContextError::CandidateParentWorkspaceMismatch {
                        parent_handle: parent_candidate.clone(),
                        parent_workspace_id: parent.workspace_id.clone(),
                        workspace_id: request.workspace_id.clone(),
                    });
                }
                if parent.basis_revision_id != request.basis_revision_id {
                    return Err(OxCalcTreeContextError::CandidateParentBasisMismatch {
                        parent_handle: parent_candidate.clone(),
                        parent_basis_revision_id: parent.basis_revision_id.clone(),
                        basis_revision_id: request.basis_revision_id.clone(),
                    });
                }
                (
                    parent.workspace_state.clone(),
                    parent.private_edit_transactions.clone(),
                )
            } else {
                let workspace = self.workspace(&request.workspace_id)?;
                let retained = workspace
                    .retained_workspace_revisions
                    .get(&request.basis_revision_id)
                    .cloned()
                    .ok_or_else(|| OxCalcTreeContextError::WorkspaceRevisionNotRetained {
                        workspace_id: request.workspace_id.clone(),
                        revision_id: request.basis_revision_id.clone(),
                    })?;
                let mut basis_state = workspace.clone();
                restore_retained_workspace_revision(&mut basis_state, retained);
                (basis_state, Vec::new())
            };

        let handle = self.next_candidate_overlay_handle(&request.workspace_id);
        if self.candidates.contains_key(&handle) {
            return Err(OxCalcTreeContextError::DuplicateCandidate { handle });
        }
        let candidate = CandidateOverlayState {
            handle: handle.clone(),
            workspace_id: request.workspace_id,
            basis_revision_id: request.basis_revision_id,
            parent_candidate: request.parent_candidate,
            retention_pin_count: 0,
            value_epoch_basis: basis_state.value_epoch,
            publication_value_epoch_basis: basis_state.publication_value_epoch,
            parent_private_edit_count: private_edit_transactions.len(),
            private_edit_transactions,
            workspace_state: basis_state,
        };
        let view = self.candidate_view_from_state(&candidate)?;
        pin_candidate_basis_revision(
            self.workspace_mut(&candidate.workspace_id)?,
            candidate.basis_revision_id.clone(),
        );
        self.candidates.insert(handle, candidate);
        self.advance_candidate_overlay_index();
        Ok(view)
    }

    pub fn candidate_view(
        &self,
        handle: &CandidateOverlayHandle,
    ) -> Result<OxCalcTreeCandidateView, OxCalcTreeContextError> {
        let candidate = self.candidates.get(handle).ok_or_else(|| {
            OxCalcTreeContextError::UnknownCandidate {
                handle: handle.clone(),
            }
        })?;
        self.candidate_view_from_state(candidate)
    }

    pub fn apply_candidate_edit_transaction(
        &mut self,
        handle: &CandidateOverlayHandle,
        transaction: OxCalcTreeEditTransaction,
    ) -> Result<OxCalcTreeCandidateView, OxCalcTreeContextError> {
        let mut candidate = self.candidates.remove(handle).ok_or_else(|| {
            OxCalcTreeContextError::UnknownCandidate {
                handle: handle.clone(),
            }
        })?;
        if transaction.workspace_id != candidate.workspace_id {
            let error = OxCalcTreeContextError::CandidateWorkspaceMismatch {
                handle: handle.clone(),
                candidate_workspace_id: candidate.workspace_id.clone(),
                transaction_workspace_id: transaction.workspace_id,
            };
            self.candidates.insert(handle.clone(), candidate);
            return Err(error);
        }

        let mut temp = self.clone();
        temp.workspaces.clear();
        temp.candidates.clear();
        temp.workspaces.insert(
            candidate.workspace_id.clone(),
            candidate.workspace_state.clone(),
        );
        let planned_summary_mutations = preview_mutations_for_candidate_transaction(&transaction);
        let planned_invalidation = planned_summary_mutations
            .as_ref()
            .map(|mutations| temp.plan_invalidation(&candidate.workspace_id, mutations))
            .transpose()?;
        let transaction = transaction.with_recalc_policy(TransactionRecalcPolicy::ApplyOnly);
        let replay_transaction = transaction.clone();
        let result = temp.apply_edit_transaction(transaction);
        match result {
            Ok(outcome) => {
                candidate.workspace_state = temp
                    .workspaces
                    .remove(&candidate.workspace_id)
                    .expect("candidate workspace should remain present");
                if let Some(plan) = planned_invalidation {
                    let summary = workspace_revision_transaction_summary_from_plan(
                        &outcome.transaction_id,
                        plan,
                    );
                    candidate
                        .workspace_state
                        .workspace_revision_graph
                        .set_current_transaction_summary(summary);
                }
                self.next_node_id = self.next_node_id.max(temp.next_node_id);
                self.next_snapshot_id = self.next_snapshot_id.max(temp.next_snapshot_id);
                self.next_transaction_index =
                    self.next_transaction_index.max(temp.next_transaction_index);
                candidate.private_edit_transactions.push(replay_transaction);
                self.candidates.insert(handle.clone(), candidate);
                self.refresh_child_candidates_from_parent(handle)?;
                self.candidate_view(handle)
            }
            Err(error) => {
                self.candidates.insert(handle.clone(), candidate);
                Err(error)
            }
        }
    }

    pub fn evaluate_candidate(
        &mut self,
        handle: &CandidateOverlayHandle,
    ) -> Result<OxCalcTreeCandidateView, OxCalcTreeContextError> {
        let mut candidate = self.candidates.remove(handle).ok_or_else(|| {
            OxCalcTreeContextError::UnknownCandidate {
                handle: handle.clone(),
            }
        })?;
        let mut temp = self.clone();
        temp.workspaces.clear();
        temp.candidates.clear();
        temp.workspaces.insert(
            candidate.workspace_id.clone(),
            candidate.workspace_state.clone(),
        );
        let result = temp.recalculate(&candidate.workspace_id);
        match result {
            Ok(_) => {
                candidate.workspace_state = temp
                    .workspaces
                    .remove(&candidate.workspace_id)
                    .expect("candidate workspace should remain present");
                self.next_candidate_index =
                    self.next_candidate_index.max(temp.next_candidate_index);
                self.candidates.insert(handle.clone(), candidate);
                self.refresh_child_candidates_from_parent(handle)?;
                self.candidate_view(handle)
            }
            Err(error) => {
                self.candidates.insert(handle.clone(), candidate);
                Err(error)
            }
        }
    }

    pub fn rebase_candidate_to_current_revision(
        &mut self,
        handle: &CandidateOverlayHandle,
    ) -> Result<OxCalcTreeCandidateView, OxCalcTreeContextError> {
        if let Some(child_handle) = self.retained_child_candidate(handle) {
            return Err(OxCalcTreeContextError::CandidateRebaseHasRetainedChild {
                handle: handle.clone(),
                child_handle,
            });
        }
        self.refresh_candidate_layer_from_parent(handle)?;
        let mut candidate = self.candidates.remove(handle).ok_or_else(|| {
            OxCalcTreeContextError::UnknownCandidate {
                handle: handle.clone(),
            }
        })?;
        let current_workspace = self.workspace(&candidate.workspace_id)?.clone();
        let current_revision_id = current_workspace.workspace_revision.revision_id().clone();
        if current_revision_id == candidate.basis_revision_id {
            candidate.parent_candidate = None;
            let view = self.candidate_view_from_state(&candidate)?;
            self.candidates.insert(handle.clone(), candidate);
            return Ok(view);
        }
        if let Some(report) =
            self.candidate_rebase_conflict_report(&candidate, &current_workspace)?
        {
            let error = OxCalcTreeContextError::CandidateRebaseConflict {
                handle: handle.clone(),
                basis_revision_id: report.basis_revision_id.clone(),
                current_revision_id: report.current_revision_id.clone(),
                overlapping_nodes: report.overlapping_nodes.clone(),
                report,
            };
            self.candidates.insert(handle.clone(), candidate);
            return Err(error);
        }

        let mut temp = self.clone();
        temp.workspaces.clear();
        temp.candidates.clear();
        temp.workspaces
            .insert(candidate.workspace_id.clone(), current_workspace.clone());
        for transaction in candidate.private_edit_transactions.clone() {
            if let Err(error) = temp.apply_edit_transaction(transaction) {
                if matches!(error, OxCalcTreeContextError::Structural(_)) {
                    let report = self.candidate_rebase_validation_conflict_report(
                        &candidate,
                        &current_workspace,
                    )?;
                    let error = OxCalcTreeContextError::CandidateRebaseConflict {
                        handle: handle.clone(),
                        basis_revision_id: report.basis_revision_id.clone(),
                        current_revision_id: report.current_revision_id.clone(),
                        overlapping_nodes: report.overlapping_nodes.clone(),
                        report,
                    };
                    self.candidates.insert(handle.clone(), candidate);
                    return Err(error);
                }
                self.candidates.insert(handle.clone(), candidate);
                return Err(error);
            }
        }

        let rebased_workspace_state = temp
            .workspaces
            .remove(&candidate.workspace_id)
            .expect("candidate workspace should remain present after rebase");
        let old_basis_revision_id = candidate.basis_revision_id.clone();
        candidate.basis_revision_id = current_revision_id.clone();
        candidate.parent_candidate = None;
        candidate.parent_private_edit_count = 0;
        candidate.value_epoch_basis = rebased_workspace_state.value_epoch;
        candidate.publication_value_epoch_basis = rebased_workspace_state.publication_value_epoch;
        candidate.workspace_state = rebased_workspace_state;
        self.next_node_id = self.next_node_id.max(temp.next_node_id);
        self.next_snapshot_id = self.next_snapshot_id.max(temp.next_snapshot_id);
        self.next_transaction_index = self.next_transaction_index.max(temp.next_transaction_index);

        if old_basis_revision_id != current_revision_id {
            let workspace = self.workspace_mut(&candidate.workspace_id)?;
            pin_candidate_basis_revision(workspace, current_revision_id);
            unpin_candidate_basis_revision(workspace, &old_basis_revision_id);
        }

        let view = self.candidate_view_from_state(&candidate)?;
        self.candidates.insert(handle.clone(), candidate);
        Ok(view)
    }

    pub fn discard_candidate(
        &mut self,
        handle: &CandidateOverlayHandle,
    ) -> Result<OxCalcTreeCandidateView, OxCalcTreeContextError> {
        if let Some(child_handle) = self.retained_child_candidate(handle) {
            return Err(OxCalcTreeContextError::CandidateHasRetainedChild {
                handle: handle.clone(),
                child_handle,
            });
        }
        let candidate = self.candidates.remove(handle).ok_or_else(|| {
            OxCalcTreeContextError::UnknownCandidate {
                handle: handle.clone(),
            }
        })?;
        unpin_candidate_basis_revision(
            self.workspace_mut(&candidate.workspace_id)?,
            &candidate.basis_revision_id,
        );
        self.candidate_view_from_state(&candidate)
    }

    pub fn pin_candidate_retention(
        &mut self,
        handle: &CandidateOverlayHandle,
    ) -> Result<OxCalcTreeCandidateView, OxCalcTreeContextError> {
        let mut candidate = self.candidates.remove(handle).ok_or_else(|| {
            OxCalcTreeContextError::UnknownCandidate {
                handle: handle.clone(),
            }
        })?;
        candidate.retention_pin_count += 1;
        let view = self.candidate_view_from_state(&candidate)?;
        self.candidates.insert(handle.clone(), candidate);
        Ok(view)
    }

    pub fn unpin_candidate_retention(
        &mut self,
        handle: &CandidateOverlayHandle,
    ) -> Result<OxCalcTreeCandidateView, OxCalcTreeContextError> {
        let mut candidate = self.candidates.remove(handle).ok_or_else(|| {
            OxCalcTreeContextError::UnknownCandidate {
                handle: handle.clone(),
            }
        })?;
        if candidate.retention_pin_count == 0 {
            self.candidates.insert(handle.clone(), candidate);
            return Err(OxCalcTreeContextError::CandidateRetentionPinNotHeld {
                handle: handle.clone(),
            });
        }
        candidate.retention_pin_count -= 1;
        let view = self.candidate_view_from_state(&candidate)?;
        self.candidates.insert(handle.clone(), candidate);
        Ok(view)
    }

    pub fn candidate_pressure(
        &self,
        policy: &OxCalcTreeCandidateReapPolicy,
    ) -> OxCalcTreeCandidatePressure {
        candidate_pressure_for(&self.candidates, policy)
    }

    pub fn reap_candidates(
        &mut self,
        policy: OxCalcTreeCandidateReapPolicy,
    ) -> Result<OxCalcTreeCandidateReapReport, OxCalcTreeContextError> {
        let pressure_before = self.candidate_pressure(&policy);
        let mut reaped_handles = Vec::new();
        while self.candidates.len() > policy.max_retained_candidates {
            let Some(handle) = self
                .candidates
                .keys()
                .find(|handle| !self.is_candidate_protected(handle))
                .cloned()
            else {
                break;
            };
            self.remove_candidate_for_reap(&handle)?;
            reaped_handles.push(handle);
        }
        let pressure_after = self.candidate_pressure(&policy);
        Ok(OxCalcTreeCandidateReapReport {
            pressure_before,
            pressure_after,
            reaped_handles,
        })
    }

    pub fn commit_candidate(
        &mut self,
        handle: &CandidateOverlayHandle,
    ) -> Result<OxCalcTreeCandidateCommitOutcome, OxCalcTreeContextError> {
        if let Some(child_handle) = self.retained_child_candidate(handle) {
            return Err(OxCalcTreeContextError::CandidateHasRetainedChild {
                handle: handle.clone(),
                child_handle,
            });
        }
        self.refresh_candidate_layer_from_parent(handle)?;
        let candidate = self.candidates.remove(handle).ok_or_else(|| {
            OxCalcTreeContextError::UnknownCandidate {
                handle: handle.clone(),
            }
        })?;

        let current_revision_id = self
            .workspace(&candidate.workspace_id)?
            .workspace_revision
            .revision_id()
            .clone();
        if current_revision_id != candidate.basis_revision_id {
            let error = OxCalcTreeContextError::CandidateBasisNotCurrent {
                handle: handle.clone(),
                basis_revision_id: candidate.basis_revision_id.clone(),
                current_revision_id,
            };
            self.candidates.insert(handle.clone(), candidate);
            return Err(error);
        }

        let successor_workspace_revision_id = candidate
            .workspace_state
            .workspace_revision
            .revision_id()
            .clone();
        let calculation = candidate.workspace_state.last_result.as_deref().cloned();
        let workspace_id = candidate.workspace_id.clone();
        let basis_revision_id = candidate.basis_revision_id.clone();
        let candidate_pinned_workspace_revisions = self
            .workspace(&workspace_id)?
            .candidate_pinned_workspace_revisions
            .clone();
        let workspace_revision_graph_entries = candidate
            .workspace_state
            .workspace_revision_graph
            .entries()
            .values()
            .cloned()
            .collect();

        let mut workspace_state = candidate.workspace_state;
        workspace_state.candidate_pinned_workspace_revisions = candidate_pinned_workspace_revisions;
        self.workspaces
            .insert(workspace_id.clone(), workspace_state);
        unpin_candidate_basis_revision(self.workspace_mut(&workspace_id)?, &basis_revision_id);
        Ok(OxCalcTreeCandidateCommitOutcome {
            handle: handle.clone(),
            workspace_id,
            basis_revision_id: basis_revision_id.clone(),
            predecessor_workspace_revision_id: basis_revision_id,
            successor_workspace_revision_id,
            workspace_revision_graph_entries,
            calculation,
        })
    }

    fn retained_child_candidate(
        &self,
        handle: &CandidateOverlayHandle,
    ) -> Option<CandidateOverlayHandle> {
        self.candidates
            .iter()
            .find_map(|(candidate_handle, candidate)| {
                (candidate.parent_candidate.as_ref() == Some(handle))
                    .then(|| candidate_handle.clone())
            })
    }

    fn is_candidate_protected(&self, handle: &CandidateOverlayHandle) -> bool {
        self.candidates
            .get(handle)
            .is_some_and(|candidate| candidate.retention_pin_count > 0)
            || self.retained_child_candidate(handle).is_some()
    }

    fn refresh_child_candidates_from_parent(
        &mut self,
        parent_handle: &CandidateOverlayHandle,
    ) -> Result<(), OxCalcTreeContextError> {
        let child_handles = self
            .candidates
            .iter()
            .filter_map(|(handle, candidate)| {
                (candidate.parent_candidate.as_ref() == Some(parent_handle)).then(|| handle.clone())
            })
            .collect::<Vec<_>>();
        for child_handle in child_handles {
            self.refresh_candidate_layer_from_parent(&child_handle)?;
            self.refresh_child_candidates_from_parent(&child_handle)?;
        }
        Ok(())
    }

    fn refresh_candidate_layer_from_parent(
        &mut self,
        handle: &CandidateOverlayHandle,
    ) -> Result<(), OxCalcTreeContextError> {
        let Some(parent_handle) = self
            .candidates
            .get(handle)
            .and_then(|candidate| candidate.parent_candidate.clone())
        else {
            return Ok(());
        };
        let parent = self
            .candidates
            .get(&parent_handle)
            .cloned()
            .ok_or_else(|| OxCalcTreeContextError::CandidateParentNotRetained {
                parent_handle: parent_handle.clone(),
            })?;
        if self.candidates.get(handle).is_some_and(|candidate| {
            candidate.parent_private_edit_count == parent.private_edit_transactions.len()
        }) {
            return Ok(());
        }
        let mut candidate = self.candidates.remove(handle).ok_or_else(|| {
            OxCalcTreeContextError::UnknownCandidate {
                handle: handle.clone(),
            }
        })?;
        let local_edit_transactions = candidate
            .private_edit_transactions
            .iter()
            .skip(candidate.parent_private_edit_count)
            .cloned()
            .collect::<Vec<_>>();
        let parent_private_edit_count = parent.private_edit_transactions.len();
        let value_epoch_basis = parent.workspace_state.value_epoch;
        let publication_value_epoch_basis = parent.workspace_state.publication_value_epoch;
        let mut workspace_state = parent.workspace_state.clone();

        if !local_edit_transactions.is_empty() {
            let mut temp = self.clone();
            temp.workspaces.clear();
            temp.candidates.clear();
            temp.workspaces
                .insert(candidate.workspace_id.clone(), workspace_state);
            for transaction in local_edit_transactions.clone() {
                if let Err(error) = temp.apply_edit_transaction(transaction) {
                    self.candidates.insert(handle.clone(), candidate);
                    return Err(error);
                }
            }
            workspace_state = temp
                .workspaces
                .remove(&candidate.workspace_id)
                .expect("candidate workspace should remain present after layer refresh");
            self.next_node_id = self.next_node_id.max(temp.next_node_id);
            self.next_snapshot_id = self.next_snapshot_id.max(temp.next_snapshot_id);
            self.next_transaction_index =
                self.next_transaction_index.max(temp.next_transaction_index);
        }

        candidate.value_epoch_basis = value_epoch_basis;
        candidate.publication_value_epoch_basis = publication_value_epoch_basis;
        candidate.parent_private_edit_count = parent_private_edit_count;
        candidate.private_edit_transactions = parent.private_edit_transactions.clone();
        candidate
            .private_edit_transactions
            .extend(local_edit_transactions);
        candidate.workspace_state = workspace_state;
        self.candidates.insert(handle.clone(), candidate);
        Ok(())
    }

    fn candidate_rebase_conflict_report(
        &self,
        candidate: &CandidateOverlayState,
        current_workspace: &OxCalcTreeWorkspaceState,
    ) -> Result<Option<OxCalcTreeCandidateRebaseConflictReport>, OxCalcTreeContextError> {
        let retained_basis = current_workspace
            .retained_workspace_revisions
            .get(&candidate.basis_revision_id)
            .cloned()
            .ok_or_else(|| OxCalcTreeContextError::WorkspaceRevisionNotRetained {
                workspace_id: candidate.workspace_id.clone(),
                revision_id: candidate.basis_revision_id.clone(),
            })?;
        let mut basis_state = current_workspace.clone();
        restore_retained_workspace_revision(&mut basis_state, retained_basis);
        let candidate_touches = rebase_touches_for_candidate_transactions(
            &candidate.private_edit_transactions,
            &basis_state.snapshot,
            basis_state.root_node_id,
        );
        let live_touches = rebase_touches_between_revisions(
            &basis_state.workspace_revision,
            &current_workspace.workspace_revision,
        );
        let overlapping_nodes = candidate_touches.conflicting_nodes_with(&live_touches);
        if overlapping_nodes.is_empty() {
            return Ok(None);
        }
        Ok(Some(OxCalcTreeCandidateRebaseConflictReport {
            handle: candidate.handle.clone(),
            basis_revision_id: candidate.basis_revision_id.clone(),
            current_revision_id: current_workspace.workspace_revision.revision_id().clone(),
            kind: OxCalcTreeCandidateRebaseConflictKind::OverlappingNodeEdits,
            candidate_touched_nodes: candidate_touches.touched_nodes(),
            live_touched_nodes: live_touches.touched_nodes(),
            overlapping_nodes,
        }))
    }

    fn candidate_rebase_validation_conflict_report(
        &self,
        candidate: &CandidateOverlayState,
        current_workspace: &OxCalcTreeWorkspaceState,
    ) -> Result<OxCalcTreeCandidateRebaseConflictReport, OxCalcTreeContextError> {
        let retained_basis = current_workspace
            .retained_workspace_revisions
            .get(&candidate.basis_revision_id)
            .cloned()
            .ok_or_else(|| OxCalcTreeContextError::WorkspaceRevisionNotRetained {
                workspace_id: candidate.workspace_id.clone(),
                revision_id: candidate.basis_revision_id.clone(),
            })?;
        let mut basis_state = current_workspace.clone();
        restore_retained_workspace_revision(&mut basis_state, retained_basis);
        let candidate_touches = rebase_touches_for_candidate_transactions(
            &candidate.private_edit_transactions,
            &basis_state.snapshot,
            basis_state.root_node_id,
        );
        let live_touches = rebase_touches_between_revisions(
            &basis_state.workspace_revision,
            &current_workspace.workspace_revision,
        );
        let overlapping_nodes = candidate_touches
            .touched_node_set()
            .intersection(&live_touches.touched_node_set())
            .copied()
            .collect::<Vec<_>>();
        let overlapping_nodes = if overlapping_nodes.is_empty() {
            candidate_touches.touched_nodes()
        } else {
            overlapping_nodes
        };
        Ok(OxCalcTreeCandidateRebaseConflictReport {
            handle: candidate.handle.clone(),
            basis_revision_id: candidate.basis_revision_id.clone(),
            current_revision_id: current_workspace.workspace_revision.revision_id().clone(),
            kind: OxCalcTreeCandidateRebaseConflictKind::ReplayValidationRejected,
            candidate_touched_nodes: candidate_touches.touched_nodes(),
            live_touched_nodes: live_touches.touched_nodes(),
            overlapping_nodes,
        })
    }

    fn remove_candidate_for_reap(
        &mut self,
        handle: &CandidateOverlayHandle,
    ) -> Result<(), OxCalcTreeContextError> {
        let candidate = self.candidates.remove(handle).ok_or_else(|| {
            OxCalcTreeContextError::UnknownCandidate {
                handle: handle.clone(),
            }
        })?;
        unpin_candidate_basis_revision(
            self.workspace_mut(&candidate.workspace_id)?,
            &candidate.basis_revision_id,
        );
        Ok(())
    }

    fn candidate_view_from_state(
        &self,
        candidate: &CandidateOverlayState,
    ) -> Result<OxCalcTreeCandidateView, OxCalcTreeContextError> {
        let nodes = candidate
            .workspace_state
            .snapshot
            .nodes()
            .values()
            .map(|node| {
                let table = candidate
                    .workspace_state
                    .table_snapshots
                    .get(&node.node_id)
                    .map(|snapshot| {
                        self.table_view_from_snapshot(
                            &candidate.workspace_state,
                            node.node_id,
                            snapshot,
                        )
                    })
                    .transpose()?;
                node_view_from_state(&candidate.workspace_state, node, table)
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(OxCalcTreeCandidateView {
            handle: candidate.handle.clone(),
            workspace_id: candidate.workspace_id.clone(),
            basis_revision_id: candidate.basis_revision_id.clone(),
            parent_candidate: candidate.parent_candidate.clone(),
            retention_pin_count: candidate.retention_pin_count,
            workspace_revision_id: candidate
                .workspace_state
                .workspace_revision
                .revision_id()
                .clone(),
            workspace_revision_graph_entries: candidate
                .workspace_state
                .workspace_revision_graph
                .entries()
                .values()
                .cloned()
                .collect(),
            nodes,
            run_state: candidate
                .workspace_state
                .last_result
                .as_ref()
                .map(|result| result.run_state),
            value_epoch_basis: candidate.value_epoch_basis,
            publication_value_epoch_basis: candidate.publication_value_epoch_basis,
            calculation: candidate.workspace_state.last_result.as_deref().cloned(),
        })
    }

    pub fn plan_invalidation(
        &self,
        workspace_id: &OxCalcTreeWorkspaceId,
        mutations: &[OxCalcTreePreviewMutation],
    ) -> Result<OxCalcTreeInvalidationPlan, OxCalcTreeContextError> {
        let state = self.workspace(workspace_id)?;
        let catalog_build = build_context_formula_catalog(state, &self.options)?;
        let graph = DependencyGraph::build(&state.snapshot, &catalog_build.dependency_descriptors);
        let mut seeds = Vec::new();
        for mutation in mutations {
            seeds.extend(preview_mutation_seeds(
                state,
                mutation,
                &catalog_build.dependency_descriptors,
            )?);
        }
        let seeds = dedupe_preview_invalidation_seeds(seeds);
        let closure = graph.derive_invalidation_closure(&seeds);
        let invalidated_nodes = closure
            .impacted_order
            .iter()
            .filter_map(|node_id| closure.records.get(node_id))
            .map(|record| OxCalcTreeInvalidationPlanEntry {
                node_id: record.node_id,
                requires_rebind: record.requires_rebind,
                reasons: record.reasons.clone(),
            })
            .collect::<Vec<_>>();
        let evaluation_order = closure
            .impacted_order
            .iter()
            .copied()
            .filter(|node_id| graph.descriptors_by_owner.contains_key(node_id))
            .collect::<Vec<_>>();
        let requires_rebind = invalidated_nodes
            .iter()
            .filter(|entry| entry.requires_rebind)
            .map(|entry| entry.node_id)
            .collect::<Vec<_>>();
        let impacted = closure
            .impacted_order
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
        let cycle_risk = graph
            .cycle_groups
            .iter()
            .filter(|group| group.iter().any(|node_id| impacted.contains(node_id)))
            .cloned()
            .collect::<Vec<_>>();
        Ok(OxCalcTreeInvalidationPlan {
            estimated_node_count: invalidated_nodes.len(),
            invalidated_nodes,
            evaluation_order,
            requires_rebind,
            cycle_risk,
        })
    }

    /// Engine-owned dependency-graph facts for the workspace's current
    /// committed revision, independent of any retained calculation outcome.
    ///
    /// Static descriptors are lowered from the current bound formula catalog.
    /// Retained published dynamic-dependency facts stand in for owners whose
    /// dynamic references have not been re-evaluated, using the same
    /// declared-potential filter the engine applies when composing a run's
    /// effective dependency graph.
    pub fn current_dependency_graph(
        &self,
        workspace_id: &OxCalcTreeWorkspaceId,
    ) -> Result<DependencyGraph, OxCalcTreeContextError> {
        let state = self.workspace(workspace_id)?;
        let catalog_build = build_context_formula_catalog(state, &self.options)?;
        let mut descriptors = catalog_build.dependency_descriptors;
        let declared_dynamic_owner_ids = descriptors
            .iter()
            .filter(|descriptor| descriptor.kind == DependencyDescriptorKind::DynamicPotential)
            .map(|descriptor| descriptor.owner_node_id)
            .collect::<BTreeSet<_>>();
        let published_dynamic_descriptors = dynamic_dependency_descriptors_from_published_facts(
            &dynamic_dependency_facts_from_runtime_effects(
                &state.publication_payload.runtime_effects,
            ),
        );
        descriptors.extend(
            published_dynamic_descriptors
                .into_iter()
                .filter(|descriptor| {
                    !declared_dynamic_owner_ids.contains(&descriptor.owner_node_id)
                }),
        );
        Ok(DependencyGraph::build(&state.snapshot, &descriptors))
    }

    pub fn dry_bind_node_formula_text(
        &self,
        workspace_id: &OxCalcTreeWorkspaceId,
        node_id: TreeNodeId,
        formula_text: impl Into<String>,
    ) -> Result<OxCalcTreeDryBindVerdict, OxCalcTreeContextError> {
        let formula_text = formula_text.into();
        let state = self.workspace(workspace_id)?;
        state
            .snapshot
            .try_get_node(node_id)
            .ok_or(StructuralError::UnknownNode { node_id })?;
        context_dry_bind_formula_text(state, node_id, &formula_text)
    }

    pub fn dry_bind_new_node_formula_text(
        &self,
        workspace_id: &OxCalcTreeWorkspaceId,
        request: OxCalcTreeNodeCreate,
    ) -> Result<OxCalcTreeDryBindVerdict, OxCalcTreeContextError> {
        let formula_text = request.formula_text.clone();
        let mut preview = self.clone();
        let node_id = preview.add_node(workspace_id, request)?;
        preview.dry_bind_node_formula_text(workspace_id, node_id, formula_text)
    }

    pub fn dry_bind_candidate_new_node_formula_text(
        &self,
        handle: &CandidateOverlayHandle,
        request: OxCalcTreeNodeCreate,
    ) -> Result<OxCalcTreeDryBindVerdict, OxCalcTreeContextError> {
        let candidate = self.candidates.get(handle).ok_or_else(|| {
            OxCalcTreeContextError::UnknownCandidate {
                handle: handle.clone(),
            }
        })?;
        let formula_text = request.formula_text.clone();
        let mut preview = self.clone();
        preview.workspaces.insert(
            candidate.workspace_id.clone(),
            candidate.workspace_state.clone(),
        );
        let node_id = preview.add_node(&candidate.workspace_id, request)?;
        preview.dry_bind_node_formula_text(&candidate.workspace_id, node_id, formula_text)
    }

    pub fn dry_bind_table_column_formula_text(
        &self,
        workspace_id: &OxCalcTreeWorkspaceId,
        table_node_id: TreeNodeId,
        column_id: impl Into<String>,
        formula_text: impl Into<String>,
    ) -> Result<OxCalcTreeDryBindVerdict, OxCalcTreeContextError> {
        let column_id = column_id.into();
        let formula_text = formula_text.into();
        let state = self.workspace(workspace_id)?;
        state
            .snapshot
            .try_get_node(table_node_id)
            .ok_or(StructuralError::UnknownNode {
                node_id: table_node_id,
            })?;
        let snapshot = state.table_snapshots.get(&table_node_id).ok_or_else(|| {
            OxCalcTreeContextError::InvalidWorkspaceSnapshot {
                detail: format!("node {table_node_id:?} has no table snapshot"),
            }
        })?;
        let view = self.table_view_from_snapshot(state, table_node_id, snapshot)?;
        let request = TreeCalcTableColumnFormulaRuntimeRequest {
            target_column_id: column_id.clone(),
            formula_stable_id: format!(
                "treecalc-table-column-dry-bind:{}:{}:{}",
                state.workspace_id.as_str(),
                table_node_id.0,
                column_id
            ),
            formula_text_version: state.value_epoch,
            formula_text,
            values: Vec::new(),
            runtime_context: TreeCalcTableFormulaRuntimeContext::default(),
        };
        let report =
            dry_bind_treecalc_table_column_formula(&view.snapshot, &view.projection, &request)
                .map_err(|error| OxCalcTreeContextError::InvalidWorkspaceSnapshot {
                    detail: format!("table body formula dry-bind failed: {error:?}"),
                })?;
        Ok(oxcalc_dry_bind_verdict_from_oxfml(
            table_node_id,
            report.verdict,
        ))
    }

    pub fn dry_bind_new_table_column_formula_text(
        &self,
        workspace_id: &OxCalcTreeWorkspaceId,
        table_node_id: TreeNodeId,
        column_id: impl Into<String>,
        column_name: impl Into<String>,
        formula_text: impl Into<String>,
    ) -> Result<OxCalcTreeDryBindVerdict, OxCalcTreeContextError> {
        let column_id = column_id.into();
        let column_name = column_name.into();
        let formula_text = formula_text.into();
        let state = self.workspace(workspace_id)?;
        state
            .snapshot
            .try_get_node(table_node_id)
            .ok_or(StructuralError::UnknownNode {
                node_id: table_node_id,
            })?;
        let snapshot = state.table_snapshots.get(&table_node_id).ok_or_else(|| {
            OxCalcTreeContextError::InvalidWorkspaceSnapshot {
                detail: format!("node {table_node_id:?} has no table snapshot"),
            }
        })?;
        if snapshot
            .columns
            .iter()
            .any(|column| column.column_id == column_id)
        {
            return Err(OxCalcTreeContextError::InvalidWorkspaceSnapshot {
                detail: format!("table already has column '{column_id}'"),
            });
        }
        let mut preview_snapshot = snapshot.clone();
        let column_ordinal = preview_snapshot
            .columns
            .iter()
            .map(|column| column.ordinal)
            .max()
            .unwrap_or(0)
            .saturating_add(1);
        preview_snapshot.columns.push(TreeCalcTableColumnSnapshot {
            column_id: column_id.clone(),
            column_name,
            ordinal: column_ordinal,
            body_metadata: TreeCalcTableColumnBodyMetadata::Formula(TreeCalcTableFormulaMetadata {
                formula_artifact_id: format!(
                    "treecalc-table-new-column-dry-bind:{}:{}:{}",
                    state.workspace_id.as_str(),
                    table_node_id.0,
                    column_id
                ),
                bind_artifact_id: None,
                formula_text_version: state.value_epoch.to_string(),
                formula_text: formula_text.clone(),
            }),
            totals_metadata: None,
        });
        let view = self.table_view_from_snapshot(state, table_node_id, &preview_snapshot)?;
        let request = TreeCalcTableColumnFormulaRuntimeRequest {
            target_column_id: column_id.clone(),
            formula_stable_id: format!(
                "treecalc-table-new-column-dry-bind:{}:{}:{}",
                state.workspace_id.as_str(),
                table_node_id.0,
                column_id
            ),
            formula_text_version: state.value_epoch,
            formula_text,
            values: Vec::new(),
            runtime_context: TreeCalcTableFormulaRuntimeContext::default(),
        };
        let report =
            dry_bind_treecalc_table_column_formula(&view.snapshot, &view.projection, &request)
                .map_err(|error| OxCalcTreeContextError::InvalidWorkspaceSnapshot {
                    detail: format!("table new column formula dry-bind failed: {error:?}"),
                })?;
        Ok(oxcalc_dry_bind_verdict_from_oxfml(
            table_node_id,
            report.verdict,
        ))
    }

    pub fn dry_bind_table_totals_formula_text(
        &self,
        workspace_id: &OxCalcTreeWorkspaceId,
        table_node_id: TreeNodeId,
        column_id: impl Into<String>,
        formula_text: impl Into<String>,
    ) -> Result<OxCalcTreeDryBindVerdict, OxCalcTreeContextError> {
        let column_id = column_id.into();
        let formula_text = formula_text.into();
        let state = self.workspace(workspace_id)?;
        state
            .snapshot
            .try_get_node(table_node_id)
            .ok_or(StructuralError::UnknownNode {
                node_id: table_node_id,
            })?;
        let snapshot = state.table_snapshots.get(&table_node_id).ok_or_else(|| {
            OxCalcTreeContextError::InvalidWorkspaceSnapshot {
                detail: format!("node {table_node_id:?} has no table snapshot"),
            }
        })?;
        let view = self.table_view_from_snapshot(state, table_node_id, snapshot)?;
        let request = TreeCalcTableColumnFormulaRuntimeRequest {
            target_column_id: column_id.clone(),
            formula_stable_id: format!(
                "treecalc-table-totals-dry-bind:{}:{}:{}",
                state.workspace_id.as_str(),
                table_node_id.0,
                column_id
            ),
            formula_text_version: state.value_epoch,
            formula_text,
            values: Vec::new(),
            runtime_context: TreeCalcTableFormulaRuntimeContext::default(),
        };
        let report =
            dry_bind_treecalc_table_totals_formula(&view.snapshot, &view.projection, &request)
                .map_err(|error| OxCalcTreeContextError::InvalidWorkspaceSnapshot {
                    detail: format!("table totals formula dry-bind failed: {error:?}"),
                })?;
        Ok(oxcalc_dry_bind_verdict_from_oxfml(
            table_node_id,
            report.verdict,
        ))
    }

    fn apply_edit_transaction_inner(
        &mut self,
        transaction: OxCalcTreeEditTransaction,
        transaction_id: OxCalcTreeTransactionId,
    ) -> Result<OxCalcTreeTransactionOutcome, OxCalcTreeContextError> {
        let edit_count = transaction.edits.len();
        let predecessor_workspace_revision_id = self
            .workspace(&transaction.workspace_id)?
            .workspace_revision
            .revision_id()
            .clone();
        let predecessor_workspace_revision_graph = self
            .workspace(&transaction.workspace_id)?
            .workspace_revision_graph
            .clone();
        let mut edit_results = Vec::with_capacity(edit_count);
        for edit in transaction.edits {
            edit_results.push(self.apply_transaction_edit(&transaction.workspace_id, edit)?);
        }
        let calculation = match transaction.recalc_policy {
            TransactionRecalcPolicy::RecalculateAndPublishOnce => {
                let outcome = self.recalculate(&transaction.workspace_id)?;
                if outcome.run_state == OxCalcTreeRunState::Rejected {
                    return Err(OxCalcTreeContextError::TransactionRejected {
                        transaction_id,
                        diagnostics: outcome.diagnostics,
                    });
                }
                Some(outcome)
            }
            TransactionRecalcPolicy::ApplyOnly => None,
        };
        let successor_workspace_revision_id = self
            .workspace(&transaction.workspace_id)?
            .workspace_revision
            .revision_id()
            .clone();
        {
            let transaction_summary =
                workspace_revision_transaction_summary(&transaction_id, calculation.as_ref());
            let state = self.workspace_mut(&transaction.workspace_id)?;
            state.workspace_revision_graph = predecessor_workspace_revision_graph;
            state.workspace_revision_graph.record_successor(
                &predecessor_workspace_revision_id,
                &state.workspace_revision,
                Some(transaction_id.to_string()),
                transaction_summary,
            );
            retain_current_workspace_revision(state);
        }
        Ok(OxCalcTreeTransactionOutcome {
            transaction_id,
            workspace_revision_id: successor_workspace_revision_id.clone(),
            predecessor_workspace_revision_id,
            successor_workspace_revision_id,
            calculation,
            edit_count,
            edit_results,
        })
    }

    fn apply_transaction_edit(
        &mut self,
        workspace_id: &OxCalcTreeWorkspaceId,
        edit: OxCalcTreeEdit,
    ) -> Result<OxCalcTreeEditResult, OxCalcTreeContextError> {
        match edit {
            OxCalcTreeEdit::AddNode { request } => {
                let node_id = self.add_node(workspace_id, request)?;
                Ok(OxCalcTreeEditResult::NodeAdded { node_id })
            }
            OxCalcTreeEdit::SetNodeInput { node_id, input } => {
                self.set_node_input_value(workspace_id, node_id, input)?;
                Ok(OxCalcTreeEditResult::Applied)
            }
            OxCalcTreeEdit::SetNodeFormulaText {
                node_id,
                formula_text,
            } => {
                self.set_node_formula_text(workspace_id, node_id, formula_text)?;
                Ok(OxCalcTreeEditResult::Applied)
            }
            OxCalcTreeEdit::SetNodeTable { node_id, snapshot } => {
                self.set_node_table(workspace_id, node_id, snapshot)?;
                Ok(OxCalcTreeEditResult::Applied)
            }
            OxCalcTreeEdit::SetNodeMeta { node_id, is_meta } => {
                self.set_node_meta(workspace_id, node_id, is_meta)?;
                Ok(OxCalcTreeEditResult::Applied)
            }
            OxCalcTreeEdit::SetReferenceCollectionMembership {
                owner_node_id,
                source_reference_handle,
                member_node_ids,
            } => {
                self.set_reference_collection_membership(
                    workspace_id,
                    owner_node_id,
                    source_reference_handle,
                    member_node_ids,
                )?;
                Ok(OxCalcTreeEditResult::Applied)
            }
            OxCalcTreeEdit::RenameNode {
                node_id,
                new_symbol,
            } => {
                self.rename_node(workspace_id, node_id, new_symbol)?;
                Ok(OxCalcTreeEditResult::Applied)
            }
            OxCalcTreeEdit::MoveNode {
                node_id,
                new_parent_id,
                new_index,
            } => {
                self.move_node(workspace_id, node_id, new_parent_id, new_index)?;
                Ok(OxCalcTreeEditResult::Applied)
            }
            OxCalcTreeEdit::ReorderNode { node_id, new_index } => {
                self.reorder_node(workspace_id, node_id, new_index)?;
                Ok(OxCalcTreeEditResult::Applied)
            }
            OxCalcTreeEdit::DeleteNode { node_id } => {
                self.delete_node(workspace_id, node_id)?;
                Ok(OxCalcTreeEditResult::Applied)
            }
        }
    }

    pub fn set_reference_collection_membership(
        &mut self,
        workspace_id: &OxCalcTreeWorkspaceId,
        owner_node_id: TreeNodeId,
        source_reference_handle: impl Into<String>,
        member_node_ids: Vec<TreeNodeId>,
    ) -> Result<(), OxCalcTreeContextError> {
        let source_reference_handle = source_reference_handle.into();
        let state = self.workspace(workspace_id)?;
        state
            .snapshot
            .try_get_node(owner_node_id)
            .ok_or(StructuralError::UnknownNode {
                node_id: owner_node_id,
            })?;
        for member_node_id in &member_node_ids {
            state
                .snapshot
                .try_get_node(*member_node_id)
                .ok_or(StructuralError::UnknownNode {
                    node_id: *member_node_id,
                })?;
        }

        let catalog_build = build_context_formula_catalog(state, &self.options)?;
        let graph = DependencyGraph::build(&state.snapshot, &catalog_build.dependency_descriptors);
        let collection = reference_collection_dependency_for_handle(
            &graph,
            owner_node_id,
            &source_reference_handle,
        )
        .ok_or_else(|| OxCalcTreeContextError::UnknownReferenceCollection {
            owner_node_id,
            source_reference_handle: source_reference_handle.clone(),
        })?;

        match collection.family {
            TreeReferenceCollectionFamily::ChildrenV1
            | TreeReferenceCollectionFamily::ReferenceLiteralArrayV1
            | TreeReferenceCollectionFamily::SiblingSetV1
            | TreeReferenceCollectionFamily::PrecedingV1
            | TreeReferenceCollectionFamily::FollowingV1
            | TreeReferenceCollectionFamily::AncestorsV1
            | TreeReferenceCollectionFamily::RecursiveDescendantsV1 => {
                Err(OxCalcTreeContextError::ReferenceCollectionNotEditable {
                    owner_node_id,
                    source_reference_handle,
                    family: collection.family,
                })
            }
        }
    }

    pub fn clear_node_input_value(
        &mut self,
        workspace_id: &OxCalcTreeWorkspaceId,
        node_id: TreeNodeId,
    ) -> Result<(), OxCalcTreeContextError> {
        self.set_node_non_formula_input_value(workspace_id, node_id, String::new())
    }

    fn set_node_non_formula_input_value(
        &mut self,
        workspace_id: &OxCalcTreeWorkspaceId,
        node_id: TreeNodeId,
        input_value: String,
    ) -> Result<(), OxCalcTreeContextError> {
        {
            let state = self.workspace_mut(workspace_id)?;
            state
                .snapshot
                .try_get_node(node_id)
                .ok_or(StructuralError::UnknownNode { node_id })?;
            let current_input_record = state
                .workspace_revision
                .node_input_snapshot
                .try_get_record(node_id)
                .ok_or_else(|| OxCalcTreeContextError::InvalidWorkspaceSnapshot {
                    detail: format!("node input snapshot is missing record for {node_id}"),
                })?;
            if current_input_record.kind == NodeInputKind::FormulaText {
                return Err(OxCalcTreeContextError::InputValueOnFormulaNode { node_id });
            }
            let input_record = successor_non_formula_input_record(state, node_id, input_value);
            replace_non_formula_node_input_record(state, input_record);
            state
                .publication_payload_mut()
                .values_by_node
                .remove(&node_id);
            push_pending_invalidation_seed(
                state,
                node_id,
                InvalidationReasonKind::UpstreamPublication,
            );
            state.last_result = None;
        }
        Ok(())
    }

    pub fn rename_node(
        &mut self,
        workspace_id: &OxCalcTreeWorkspaceId,
        node_id: TreeNodeId,
        new_symbol: impl Into<String>,
    ) -> Result<(), OxCalcTreeContextError> {
        let snapshot_id = self.next_snapshot_id();
        {
            let state = self.workspace_mut(workspace_id)?;
            let outcome = state.snapshot.apply_edit(
                snapshot_id,
                StructuralEdit::RenameNode {
                    node_id,
                    new_symbol: new_symbol.into(),
                },
            )?;
            state.snapshot = Arc::new(outcome.snapshot);
            refresh_workspace_revision_and_absent_layers(state);
            state.pending_invalidation_seeds.clear();
            clear_pending_edit_transition_facts(state);
            state.clear_publication_payload();
            state.last_result = None;
        }
        self.advance_snapshot_id();
        Ok(())
    }

    pub fn move_node(
        &mut self,
        workspace_id: &OxCalcTreeWorkspaceId,
        node_id: TreeNodeId,
        new_parent_id: TreeNodeId,
        new_index: Option<usize>,
    ) -> Result<(), OxCalcTreeContextError> {
        let snapshot_id = self.next_snapshot_id();
        {
            let state = self.workspace_mut(workspace_id)?;
            let outcome = state.snapshot.apply_edit(
                snapshot_id,
                StructuralEdit::MoveNode {
                    node_id,
                    new_parent_id,
                    new_index,
                },
            )?;
            state.snapshot = Arc::new(outcome.snapshot);
            refresh_workspace_revision_and_absent_layers(state);
            state.pending_invalidation_seeds.clear();
            clear_pending_edit_transition_facts(state);
            state.clear_publication_payload();
            state.last_result = None;
        }
        self.advance_snapshot_id();
        Ok(())
    }

    pub fn reorder_node(
        &mut self,
        workspace_id: &OxCalcTreeWorkspaceId,
        node_id: TreeNodeId,
        new_index: usize,
    ) -> Result<(), OxCalcTreeContextError> {
        let parent_id = self
            .workspace(workspace_id)?
            .snapshot
            .parent_id_of(node_id)
            .ok_or(OxCalcTreeContextError::CannotReorderRoot { node_id })?;
        self.move_node(workspace_id, node_id, parent_id, Some(new_index))
    }

    pub fn delete_node(
        &mut self,
        workspace_id: &OxCalcTreeWorkspaceId,
        node_id: TreeNodeId,
    ) -> Result<(), OxCalcTreeContextError> {
        let snapshot_id = self.next_snapshot_id();
        {
            let state = self.workspace_mut(workspace_id)?;
            let outcome = state
                .snapshot
                .apply_edit(snapshot_id, StructuralEdit::RemoveNode { node_id })?;
            for removed_node_id in &outcome.affected_node_ids {
                remove_node_input_record(state, *removed_node_id);
                state.pending_invalidation_seeds.clear();
                if let Some(snapshot) = state.table_snapshots_mut().remove(removed_node_id) {
                    let deleted = deleted_table_fact_from_snapshot(state, &snapshot);
                    state.deleted_table_facts.push(deleted);
                    state.table_state_version += 1;
                }
            }
            remove_deleted_publication_and_runtime_facts(state, &outcome.affected_node_ids);
            state.snapshot = Arc::new(outcome.snapshot);
            refresh_workspace_revision_and_absent_layers(state);
            state.pending_invalidation_seeds.clear();
            clear_pending_edit_transition_facts(state);
            state.last_result = None;
        }
        self.advance_snapshot_id();
        Ok(())
    }

    pub fn set_node_table(
        &mut self,
        workspace_id: &OxCalcTreeWorkspaceId,
        node_id: TreeNodeId,
        snapshot: TreeCalcTableNodeSnapshot,
    ) -> Result<OxCalcTreeTableView, OxCalcTreeContextError> {
        let snapshot_id = self.next_snapshot_id();
        {
            let state = self.workspace_mut(workspace_id)?;
            state
                .snapshot
                .try_get_node(node_id)
                .ok_or(StructuralError::UnknownNode { node_id })?;
            let next_table_state_version = state.table_state_version + 1;
            let normalized = normalize_context_table_snapshot_with_snapshot_id(
                state,
                snapshot_id,
                node_id,
                &snapshot,
                next_table_state_version,
            )?;
            project_treecalc_table_node_snapshot(&normalized)
                .map_err(|error| OxCalcTreeContextError::TableProjection { error })?;
            let table_shape = structural_table_shape_from_table_snapshot(&normalized);
            let outcome = state.snapshot.apply_edit(
                snapshot_id,
                StructuralEdit::SetTableShape {
                    node_id,
                    table_shape,
                },
            )?;
            state.snapshot = Arc::new(outcome.snapshot);
            state.table_snapshots_mut().insert(node_id, normalized);
            state.table_state_version = next_table_state_version;
            refresh_workspace_revision_and_absent_layers(state);
            state.pending_invalidation_seeds.clear();
            clear_pending_edit_transition_facts(state);
            state.clear_publication_payload();
            state.last_result = None;
        }
        self.advance_snapshot_id();
        self.table_view(workspace_id, node_id)?
            .ok_or(StructuralError::UnknownNode { node_id }.into())
    }

    pub fn clear_node_table(
        &mut self,
        workspace_id: &OxCalcTreeWorkspaceId,
        node_id: TreeNodeId,
    ) -> Result<Option<TreeCalcTableNodeSnapshot>, OxCalcTreeContextError> {
        if !self
            .workspace(workspace_id)?
            .table_snapshots
            .contains_key(&node_id)
        {
            return Ok(None);
        }

        let snapshot_id = self.next_snapshot_id();
        let removed = {
            let state = self.workspace_mut(workspace_id)?;
            let snapshot = state
                .table_snapshots_mut()
                .remove(&node_id)
                .expect("table presence was checked before structural clear");
            let outcome = state
                .snapshot
                .apply_edit(snapshot_id, StructuralEdit::ClearTableShape { node_id })?;
            state.snapshot = Arc::new(outcome.snapshot);
            let deleted = deleted_table_fact_from_snapshot(state, &snapshot);
            state.deleted_table_facts.push(deleted);
            state.table_state_version += 1;
            refresh_workspace_revision_and_absent_layers(state);
            state.pending_invalidation_seeds.clear();
            clear_pending_edit_transition_facts(state);
            state.clear_publication_payload();
            state.last_result = None;
            snapshot
        };
        self.advance_snapshot_id();
        Ok(Some(removed))
    }

    pub fn table_view(
        &self,
        workspace_id: &OxCalcTreeWorkspaceId,
        node_id: TreeNodeId,
    ) -> Result<Option<OxCalcTreeTableView>, OxCalcTreeContextError> {
        let state = self.workspace(workspace_id)?;
        state
            .table_snapshots
            .get(&node_id)
            .map(|snapshot| self.table_view_from_snapshot(state, node_id, snapshot))
            .transpose()
    }

    /// Attach (or replace) a grid backing on a sheet node: build the optimized
    /// grid engine from the seed, record the structural grid shape, and return
    /// the recalculated readout. The reference twin is derived on demand for the
    /// differential, so any divergence shows up in the returned view's
    /// `differential_mismatches`.
    pub fn set_node_grid(
        &mut self,
        workspace_id: &OxCalcTreeWorkspaceId,
        node_id: TreeNodeId,
        seed: GridBackingSeed,
    ) -> Result<OxCalcTreeGridView, OxCalcTreeContextError> {
        let snapshot_id = self.next_snapshot_id();
        {
            let state = self.workspace_mut(workspace_id)?;
            state
                .snapshot
                .try_get_node(node_id)
                .ok_or(StructuralError::UnknownNode { node_id })?;
            let next_grid_state_version = state.grid_state_version + 1;
            let grid_id = format!("{}:{}", seed.workbook_id, seed.sheet_id);

            let mut sheet = GridOptimizedSheet::new(
                seed.workbook_id.clone(),
                seed.sheet_id.clone(),
                seed.bounds,
            );
            let mut authored_addresses = BTreeSet::new();
            for (address, cell) in &seed.authored {
                match cell {
                    GridAuthoredCell::Literal(value) => sheet
                        .set_literal(address.clone(), value.clone())
                        .map_err(|error| OxCalcTreeContextError::GridEngine { error })?,
                    GridAuthoredCell::Formula(formula) => sheet
                        .set_formula(address.clone(), formula.clone())
                        .map_err(|error| OxCalcTreeContextError::GridEngine { error })?,
                }
                authored_addresses.insert(address.clone());
            }
            // Install committed document-state overlays (structured tables,
            // merged regions). Spills are not seeded - they are produced by the
            // recalc below.
            for table in &seed.table_overlays {
                sheet
                    .set_table_overlay(table.clone())
                    .map_err(|error| OxCalcTreeContextError::GridEngine { error })?;
            }
            for rect in &seed.merged_regions {
                sheet
                    .add_merged_region(rect.clone())
                    .map_err(|error| OxCalcTreeContextError::GridEngine { error })?;
            }

            let grid_shape = StructuralGridShape {
                grid_id: grid_id.clone(),
                sheet_name: seed.sheet_id.clone(),
                bounds_identity: format!("{}x{}", seed.bounds.max_rows, seed.bounds.max_cols),
                cell_population_version: format!("cells:v{next_grid_state_version}"),
                axis_state_version: format!("axes:v{next_grid_state_version}"),
                overlay_set_version: format!("overlays:v{next_grid_state_version}"),
                merged_region_version: format!("merges:v{next_grid_state_version}"),
            };
            let outcome = state.snapshot.apply_edit(
                snapshot_id,
                StructuralEdit::SetGridShape {
                    node_id,
                    grid_shape,
                },
            )?;
            state.snapshot = Arc::new(outcome.snapshot);
            let mut backing = GridBackingState {
                grid_id,
                authored_addresses,
                sheet,
                published: BTreeMap::new(),
                differential_mismatches: Vec::new(),
                recalc_epoch: 0,
                interest: None,
                published_overlays: OxCalcTreeGridOverlays::default(),
                overlay_epoch: 0,
            };
            backing
                .recalc()
                .map_err(|error| OxCalcTreeContextError::GridEngine { error })?;
            state.grids_mut().insert(node_id, backing);
            state.grid_state_version = next_grid_state_version;
            refresh_workspace_revision_and_absent_layers(state);
            state.pending_invalidation_seeds.clear();
            clear_pending_edit_transition_facts(state);
            state.clear_publication_payload();
            state.last_result = None;
        }
        self.advance_snapshot_id();
        self.grid_view(workspace_id, node_id)?
            .ok_or(StructuralError::UnknownNode { node_id }.into())
    }

    /// Remove a node's grid backing. Returns `true` if one was present.
    pub fn clear_node_grid(
        &mut self,
        workspace_id: &OxCalcTreeWorkspaceId,
        node_id: TreeNodeId,
    ) -> Result<bool, OxCalcTreeContextError> {
        if !self.workspace(workspace_id)?.grids.contains_key(&node_id) {
            return Ok(false);
        }
        let snapshot_id = self.next_snapshot_id();
        {
            let state = self.workspace_mut(workspace_id)?;
            state.grids_mut().remove(&node_id);
            let outcome = state
                .snapshot
                .apply_edit(snapshot_id, StructuralEdit::ClearGridShape { node_id })?;
            state.snapshot = Arc::new(outcome.snapshot);
            state.grid_state_version += 1;
            refresh_workspace_revision_and_absent_layers(state);
            state.pending_invalidation_seeds.clear();
            clear_pending_edit_transition_facts(state);
            state.clear_publication_payload();
            state.last_result = None;
        }
        self.advance_snapshot_id();
        Ok(true)
    }

    /// Read a node's grid backing: recalculate (running both the reference and
    /// optimized engines for the differential) and return the computed value of
    /// each authored cell. Returns `None` if the node has no grid backing.
    /// The tree nodes that currently carry a grid backing, in node-id order. A
    /// consumer projecting the workspace iterates these and calls
    /// [`grid_view`](Self::grid_view) for each (rather than probing every node).
    pub fn grid_backed_node_ids(
        &self,
        workspace_id: &OxCalcTreeWorkspaceId,
    ) -> Result<Vec<TreeNodeId>, OxCalcTreeContextError> {
        let state = self.workspace(workspace_id)?;
        Ok(state.grids.keys().copied().collect())
    }

    pub fn grid_view(
        &self,
        workspace_id: &OxCalcTreeWorkspaceId,
        node_id: TreeNodeId,
    ) -> Result<Option<OxCalcTreeGridView>, OxCalcTreeContextError> {
        let state = self.workspace(workspace_id)?;
        let Some(grid) = state.grids.get(&node_id) else {
            return Ok(None);
        };
        let cells = grid
            .published
            .iter()
            .map(|(address, cell)| OxCalcTreeGridCellReadout {
                address: address.clone(),
                value: cell.value.clone(),
                value_epoch: cell.value_epoch,
            })
            .collect();
        Ok(Some(OxCalcTreeGridView {
            grid_node_id: node_id,
            grid_id: grid.grid_id.clone(),
            bounds: grid.sheet.bounds(),
            cells,
            overlays: grid.published_overlays.clone(),
            overlay_epoch: grid.overlay_epoch,
            differential_mismatches: grid.differential_mismatches.clone(),
        }))
    }

    /// Register (or replace) a client's regions of interest in a node's grid and
    /// re-materialize the cached publication scoped to them. Returns the current
    /// grid epoch as the baseline for `poll_grid_changes`. Read-shaping: it does
    /// not advance the workspace revision. Returns `None` if the node has no grid
    /// backing.
    ///
    /// Contract: after a scope change, read the full current scope with
    /// `grid_view`, then `poll_grid_changes(since = returned epoch)` for the
    /// incremental changes within that (now stable) scope. `poll` is purely
    /// "changed since"; it does not re-deliver unchanged cells that a scope
    /// widening newly brought into view.
    ///
    /// Note: the recalc is currently whole-sheet internally; only the cached
    /// readout is scoped. Compute-only-visible windowing (via the visible-rect
    /// recalc) is the scale refinement.
    pub fn register_grid_interest(
        &mut self,
        workspace_id: &OxCalcTreeWorkspaceId,
        node_id: TreeNodeId,
        regions: GridInterestRegions,
    ) -> Result<Option<GridInterestEpoch>, OxCalcTreeContextError> {
        let state = self.workspace_mut(workspace_id)?;
        if !state.grids.contains_key(&node_id) {
            return Ok(None);
        }
        let grid = state
            .grids_mut()
            .get_mut(&node_id)
            .expect("grid presence was checked");
        grid.interest = Some(regions);
        grid.recalc()
            .map_err(|error| OxCalcTreeContextError::GridEngine { error })?;
        Ok(Some(GridInterestEpoch(grid.recalc_epoch)))
    }

    /// Pull the grid cells (within the registered interest) whose value changed
    /// after `since`. Strictly pull and passive: nothing is computed until this
    /// is called, and there is no callback. Returns `None` if the node has no
    /// grid backing.
    pub fn poll_grid_changes(
        &self,
        workspace_id: &OxCalcTreeWorkspaceId,
        node_id: TreeNodeId,
        since: GridInterestEpoch,
    ) -> Result<Option<GridDeltaPacket>, OxCalcTreeContextError> {
        let state = self.workspace(workspace_id)?;
        let Some(grid) = state.grids.get(&node_id) else {
            return Ok(None);
        };
        // The published cache holds every interested cell with its last-changed
        // epoch, so "changed since E" is a filter. A `since` ahead of the current
        // epoch is incoherent (e.g. the grid was recreated) -> resync.
        let resync = since.0 > grid.recalc_epoch;
        let changed = grid
            .published
            .iter()
            .filter(|(_, cell)| resync || cell.value_epoch > since.0)
            .map(|(address, cell)| OxCalcTreeGridCellReadout {
                address: address.clone(),
                value: cell.value.clone(),
                value_epoch: cell.value_epoch,
            })
            .collect();
        Ok(Some(GridDeltaPacket {
            grid_node_id: node_id,
            from_epoch: since,
            to_epoch: GridInterestEpoch(grid.recalc_epoch),
            resync,
            changed,
        }))
    }

    /// Apply a region-granular edit to a node's grid backing, recalculate, and
    /// return the updated (scoped) view. The recalc bumps the value epoch of
    /// every cell whose value changed, so a client can observe the edit's effect
    /// incrementally via `poll_grid_changes`. Returns `None` if the node has no
    /// grid backing. `FillRange` compiles to a single repeated-formula region;
    /// the permanent-pair differential (run inside `recalc`) is the
    /// materialization-invariance check (the reference engine expands the region
    /// per cell and must agree with the optimized region).
    pub fn apply_grid_edit(
        &mut self,
        workspace_id: &OxCalcTreeWorkspaceId,
        node_id: TreeNodeId,
        op: OxCalcTreeGridOp,
    ) -> Result<Option<OxCalcTreeGridView>, OxCalcTreeContextError> {
        {
            let state = self.workspace_mut(workspace_id)?;
            if !state.grids.contains_key(&node_id) {
                return Ok(None);
            }
            let grid = state
                .grids_mut()
                .get_mut(&node_id)
                .expect("grid presence was checked");
            match op {
                OxCalcTreeGridOp::SetCell { address, cell } => {
                    match cell {
                        GridAuthoredCell::Literal(value) => {
                            grid.sheet.set_literal(address.clone(), value)
                        }
                        GridAuthoredCell::Formula(formula) => {
                            grid.sheet.set_formula(address.clone(), formula)
                        }
                    }
                    .map_err(|error| OxCalcTreeContextError::GridEngine { error })?;
                    grid.authored_addresses.insert(address);
                }
                OxCalcTreeGridOp::FillRange { rect, formula } => {
                    // Enumerate the filled cells first so an over-large fill fails
                    // before mutating the sheet (no partial application). These
                    // become readable (authored) cells. (Region-based authored
                    // tracking, avoiding per-cell enumeration for whole-column
                    // fills, is the scale refinement.)
                    let cells = rect
                        .scalar_cells(GRID_CALC_REF_DEFAULT_MATERIALIZATION_LIMIT)
                        .map_err(|error| OxCalcTreeContextError::GridEngine { error })?;
                    grid.sheet
                        .put_repeated_formula_region(rect.clone(), formula)
                        .map_err(|error| OxCalcTreeContextError::GridEngine { error })?;
                    for address in cells {
                        grid.authored_addresses.insert(address);
                    }
                }
            }
            grid.recalc()
                .map_err(|error| OxCalcTreeContextError::GridEngine { error })?;
        }
        self.grid_view(workspace_id, node_id)
    }

    pub fn workspace_table_views(
        &self,
        workspace_id: &OxCalcTreeWorkspaceId,
    ) -> Result<Vec<OxCalcTreeTableView>, OxCalcTreeContextError> {
        let state = self.workspace(workspace_id)?;
        self.table_views_for_state(state)
    }

    pub fn table_context_packet(
        &self,
        workspace_id: &OxCalcTreeWorkspaceId,
        enclosing_table_ref: Option<TableRef>,
        caller_table_region: Option<TableCallerRegion>,
    ) -> Result<StructuredTableContextPacket, OxCalcTreeContextError> {
        let state = self.workspace(workspace_id)?;
        let projections = self.table_projections_for_state(state)?;
        Ok(StructuredTableContextPacket::from_oxfml_table_packet(
            projections
                .into_iter()
                .map(|projection| projection.table_descriptor)
                .collect(),
            enclosing_table_ref,
            caller_table_region,
        ))
    }

    pub fn resolve_table_reference(
        &self,
        workspace_id: &OxCalcTreeWorkspaceId,
        request: &TreeCalcTableCatalogResolveRequest,
    ) -> Result<TreeCalcTableCatalogResolution, OxCalcTreeContextError> {
        let context = self.table_catalog_resolver_context(workspace_id)?;
        Ok(resolve_treecalc_table_catalog_reference(&context, request))
    }

    pub fn lower_table_reference(
        &self,
        workspace_id: &OxCalcTreeWorkspaceId,
        owner_node_id: TreeNodeId,
        reference: StructuredTableReferenceIntake,
        enclosing_table_ref: Option<TableRef>,
        caller_table_region: Option<TableCallerRegion>,
    ) -> Result<StructuredTableDependencyLowering, OxCalcTreeContextError> {
        let context_packet =
            self.table_context_packet(workspace_id, enclosing_table_ref, caller_table_region)?;
        Ok(lower_structured_table_dependencies(
            &StructuredTableDependencyLoweringRequest {
                owner_node_id,
                source_reference_handle: Some(reference.reference_handle.clone()),
                context_packet,
                reference,
            },
        ))
    }

    pub fn lower_table_bind_record(
        &self,
        workspace_id: &OxCalcTreeWorkspaceId,
        owner_node_id: TreeNodeId,
        record: &StructuredReferenceBindRecord,
        enclosing_table_ref: Option<TableRef>,
        caller_table_region: Option<TableCallerRegion>,
    ) -> Result<StructuredTableDependencyLowering, OxCalcTreeContextError> {
        let context_packet =
            self.table_context_packet(workspace_id, enclosing_table_ref, caller_table_region)?;
        let request = StructuredTableDependencyLoweringRequest::from_oxfml_bind_record(
            owner_node_id,
            context_packet,
            record,
        )
        .map_err(|error| OxCalcTreeContextError::TableBindRecordIntake { error })?;
        Ok(lower_structured_table_dependencies(&request))
    }

    pub fn classify_dynamic_table_rebind(
        &self,
        workspace_id: &OxCalcTreeWorkspaceId,
        mut request: TreeCalcDynamicTableRebindRequest,
    ) -> Result<TreeCalcDynamicTableRebindReport, OxCalcTreeContextError> {
        let state = self.workspace(workspace_id)?;
        request.context_versions = self.table_lifecycle_context_versions(state);
        Ok(classify_treecalc_dynamic_table_rebind(&request))
    }

    pub fn export_workspace_snapshot(
        &self,
        workspace_id: &OxCalcTreeWorkspaceId,
    ) -> Result<OxCalcTreeWorkspaceSnapshot, OxCalcTreeContextError> {
        let state = self.workspace(workspace_id)?;
        Ok(OxCalcTreeWorkspaceSnapshot {
            schema_version: OXCALC_TREE_WORKSPACE_SNAPSHOT_SCHEMA_V2.to_string(),
            workspace_id: state.workspace_id.clone(),
            root_node_id: state.root_node_id,
            workspace_revision: (*state.workspace_revision).clone(),
            formula_binding_snapshot: state.formula_binding_snapshot.clone(),
            dependency_shape_snapshot: state.dependency_shape_snapshot.clone(),
            publication_snapshot: (*state.publication_snapshot).clone(),
            runtime_overlay_set: (*state.runtime_overlay_set).clone(),
            input_epoch_watermark: state.value_epoch,
            publication_value_epoch_watermark: state.publication_value_epoch,
            table_snapshots: (*state.table_snapshots).clone(),
            deleted_table_facts: state.deleted_table_facts.clone(),
            table_state_version: state.table_state_version,
            publication_values: state
                .publication_payload
                .values_by_node
                .iter()
                .map(|(node_id, value)| (*node_id, SnapshotCalcValue::from_calc_value(value)))
                .collect(),
            publication_value_epochs: state.publication_payload.value_epochs_by_node.clone(),
            publication_runtime_effects: state.publication_payload.runtime_effects.clone(),
        })
    }

    pub fn workspace_revision(
        &self,
        workspace_id: &OxCalcTreeWorkspaceId,
    ) -> Result<WorkspaceRevision, OxCalcTreeContextError> {
        Ok((*self.workspace(workspace_id)?.workspace_revision).clone())
    }

    pub fn navigate_workspace_revision(
        &mut self,
        workspace_id: &OxCalcTreeWorkspaceId,
        revision_id: &WorkspaceRevisionId,
    ) -> Result<OxCalcTreeRevisionNavigationOutcome, OxCalcTreeContextError> {
        let predecessor_workspace_revision_id = self
            .workspace(workspace_id)?
            .workspace_revision
            .revision_id()
            .clone();
        let state = self.workspace_mut(workspace_id)?;
        let retained = state
            .retained_workspace_revisions
            .get(revision_id)
            .cloned()
            .ok_or_else(|| OxCalcTreeContextError::WorkspaceRevisionNotRetained {
                workspace_id: workspace_id.clone(),
                revision_id: revision_id.clone(),
            })?;
        state.workspace_revision_graph.navigate_to(revision_id)?;
        restore_retained_workspace_revision(state, retained);
        let successor_workspace_revision_id = state.workspace_revision.revision_id().clone();
        Ok(OxCalcTreeRevisionNavigationOutcome {
            predecessor_workspace_revision_id,
            successor_workspace_revision_id: successor_workspace_revision_id.clone(),
            workspace_revision_id: successor_workspace_revision_id,
        })
    }

    pub fn import_workspace_snapshot(
        &mut self,
        snapshot: OxCalcTreeWorkspaceSnapshot,
    ) -> Result<OxCalcTreeWorkspaceId, OxCalcTreeContextError> {
        if self.workspaces.contains_key(&snapshot.workspace_id) {
            return Err(OxCalcTreeContextError::DuplicateWorkspace {
                workspace_id: snapshot.workspace_id.as_str().to_string(),
            });
        }
        validate_workspace_snapshot(&snapshot)?;

        let workspace_id = snapshot.workspace_id.clone();
        let workspace_revision = snapshot.workspace_revision.clone();
        let structural_snapshot = workspace_revision.structure_snapshot.clone();
        let workspace_revision_graph = WorkspaceRevisionGraph::initial(&workspace_revision);
        let publication_value_epochs = snapshot.publication_value_epochs;
        let publication_value_epoch = snapshot.publication_value_epoch_watermark.max(
            publication_value_epochs
                .values()
                .copied()
                .max()
                .unwrap_or(0),
        );
        let state = OxCalcTreeWorkspaceState {
            workspace_id: workspace_id.clone(),
            root_node_id: snapshot.root_node_id,
            snapshot: Arc::new(structural_snapshot),
            workspace_revision: Arc::new(workspace_revision),
            workspace_revision_graph,
            retained_workspace_revisions: BTreeMap::new(),
            retained_workspace_revision_order: VecDeque::new(),
            candidate_pinned_workspace_revisions: BTreeMap::new(),
            revision_retention_policy: self.options.revision_retention_policy,
            formula_binding_snapshot: snapshot.formula_binding_snapshot,
            dependency_shape_snapshot: snapshot.dependency_shape_snapshot,
            publication_snapshot: Arc::new(snapshot.publication_snapshot),
            runtime_overlay_set: Arc::new(snapshot.runtime_overlay_set),
            value_epoch: snapshot.input_epoch_watermark,
            publication_value_epoch,
            table_snapshots: Arc::new(snapshot.table_snapshots),
            deleted_table_facts: snapshot.deleted_table_facts,
            table_state_version: snapshot.table_state_version,
            // Grid backings are not part of the serializable workspace snapshot
            // yet (grid persistence is Phase 4); a restored workspace starts with
            // no grid backings.
            grids: Arc::new(BTreeMap::new()),
            grid_state_version: 1,
            publication_payload: Arc::new(PublishedRuntimeLayerPayload::new(
                snapshot
                    .publication_values
                    .iter()
                    .map(|(node_id, value)| (*node_id, value.to_calc_value()))
                    .collect(),
                publication_value_epochs,
                snapshot.publication_runtime_effects,
            )),
            pending_invalidation_seeds: Vec::new(),
            pending_formula_edit_diagnostics: Vec::new(),
            pending_node_input_kind_transitions: Vec::new(),
            pending_dependency_shape_updates: Vec::new(),
            last_result: None,
            prepared_formula_retention: PreparedFormulaRetention::default(),
        };
        let mut state = state;
        retain_current_workspace_revision(&mut state);
        self.advance_allocators_past_state(&state);
        self.workspaces.insert(workspace_id.clone(), state);
        Ok(workspace_id)
    }

    pub fn recalculate(
        &mut self,
        workspace_id: &OxCalcTreeWorkspaceId,
    ) -> Result<OxCalcTreeCalculationOutcome, OxCalcTreeContextError> {
        let candidate_index = self.next_candidate_index();
        let state = self.workspace(workspace_id)?;
        let catalog_build = build_context_formula_catalog(state, &self.options)?;
        let prepared_formula_retention = catalog_build.prepared_formula_retention;
        let formula_dependency_descriptors = catalog_build.dependency_descriptors;
        let formula_binding_snapshot = FormulaBindingSnapshot::current(
            state.workspace_revision.revision_id(),
            formula_binding_snapshot_basis(&catalog_build.catalog),
        );
        let pending_formula_edit_diagnostics = state.pending_formula_edit_diagnostics.clone();
        let pending_node_input_kind_transitions = state.pending_node_input_kind_transitions.clone();
        let pending_dependency_shape_updates = state.pending_dependency_shape_updates.clone();
        let environment_context = runtime_context_for_workspace_state(&self.options, state);
        let candidate_result_id =
            format!("candidate:{}:{}", workspace_id.as_str(), candidate_index);
        let artifacts = LocalTreeCalcEngine.execute_with_retained_preparations(
            LocalTreeCalcInput {
                workspace_revision: (*state.workspace_revision).clone(),
                formula_catalog: catalog_build.catalog,
                formula_dependency_descriptors: Some(formula_dependency_descriptors),
                table_snapshots: (*state.table_snapshots).clone(),
                layer_snapshot_ids: LocalTreeCalcLayerSnapshotIds {
                    formula_binding_snapshot_id: formula_binding_snapshot.snapshot_id().clone(),
                    dependency_shape_snapshot_id: state
                        .dependency_shape_snapshot
                        .snapshot_id()
                        .clone(),
                    publication_snapshot_id: state.publication_snapshot.snapshot_id().clone(),
                    runtime_overlay_set_id: state.runtime_overlay_set.overlay_set_id().clone(),
                },
                static_dependency_shape_updates: pending_dependency_shape_updates,
                publication_calc_values: state.publication_payload.values_by_node.clone(),
                publication_runtime_effects: state.publication_payload.runtime_effects.clone(),
                invalidation_seeds: state.pending_invalidation_seeds.clone(),
                previous_arg_preparation_profile_version: None,
                candidate_result_id: candidate_result_id.clone(),
                publication_id: format!(
                    "publication:{}:{}",
                    workspace_id.as_str(),
                    candidate_index
                ),
                environment_context,
            },
            Some(&prepared_formula_retention),
        )?;
        let mut result = OxCalcTreeCalculationOutcome::from(artifacts);
        result.diagnostics.extend(self.options.diagnostics());
        result.diagnostics.extend(catalog_build.diagnostics);
        result.diagnostics.extend(pending_formula_edit_diagnostics);
        result.diagnostics.extend(
            pending_node_input_kind_transitions
                .iter()
                .map(ContextNodeInputKindTransition::diagnostic),
        );
        apply_context_profile_rejection(&mut result, candidate_result_id);
        let state = self.workspace_mut(workspace_id)?;
        state.prepared_formula_retention = prepared_formula_retention;
        state.formula_binding_snapshot = formula_binding_snapshot;
        state.dependency_shape_snapshot = match result.run_state {
            OxCalcTreeRunState::Rejected => DependencyShapeSnapshot::current_absent(
                state.workspace_revision.revision_id(),
                state.formula_binding_snapshot.snapshot_id(),
                "rejected-candidate-dependency-shape-not-published",
            ),
            OxCalcTreeRunState::Published | OxCalcTreeRunState::VerifiedClean => {
                DependencyShapeSnapshot::from_dependency_graph(
                    state.workspace_revision.revision_id(),
                    state.formula_binding_snapshot.snapshot_id(),
                    &result.dependency_graph,
                )
            }
        };
        if result.run_state == OxCalcTreeRunState::Published {
            let (publication_value_epoch, published_value_epochs) = published_value_epochs_after(
                &state.publication_payload.values_by_node,
                &state.publication_payload.value_epochs_by_node,
                &result.published_calc_values,
                state.publication_value_epoch,
            );
            state.publication_value_epoch = publication_value_epoch;
            // Fresh payload (the old one may be Arc-shared with retained
            // revisions); same fields the in-place assignments used to set.
            state.publication_payload = Arc::new(PublishedRuntimeLayerPayload::new(
                result.published_calc_values.clone(),
                published_value_epochs.clone(),
                result.publication_bundle.as_ref().map_or_else(
                    || result.runtime_effects.clone(),
                    |bundle| bundle.published_runtime_effects.clone(),
                ),
            ));
            result.published_value_epochs = published_value_epochs;
            state.publication_snapshot = Arc::new(
                if let Some(publication_bundle) = result.publication_bundle.as_ref() {
                    PublicationSnapshot::from_publication_bundle(
                        state.workspace_revision.revision_id(),
                        &calc_value_display_map(&result.published_calc_values),
                        publication_bundle,
                        &result.diagnostics,
                    )
                } else {
                    PublicationSnapshot::from_published_values(
                        state.workspace_revision.revision_id(),
                        &calc_value_display_map(&result.published_calc_values),
                        &result.runtime_effects,
                    )
                },
            );
            state.runtime_overlay_set = Arc::new(RuntimeOverlaySet::from_overlays(
                state.publication_snapshot.snapshot_id(),
                &result.runtime_effect_overlays,
            ));
        } else {
            result.published_value_epochs = state.publication_payload.value_epochs_by_node.clone();
        }
        state.pending_invalidation_seeds.clear();
        state.pending_formula_edit_diagnostics.clear();
        state.pending_node_input_kind_transitions.clear();
        state.pending_dependency_shape_updates.clear();
        // Single deep copy of the outcome: the live state and the retained
        // revision entry share one Arc; the caller gets the moved original.
        state.last_result = Some(Arc::new(result.clone()));
        retain_current_workspace_revision(state);
        self.advance_candidate_index();
        Ok(result)
    }

    pub fn workspace_view(
        &self,
        workspace_id: &OxCalcTreeWorkspaceId,
    ) -> Result<OxCalcTreeWorkspaceView, OxCalcTreeContextError> {
        let state = self.workspace(workspace_id)?;
        let nodes = state
            .snapshot
            .nodes()
            .values()
            .map(|node| {
                let table = state
                    .table_snapshots
                    .get(&node.node_id)
                    .map(|snapshot| self.table_view_from_snapshot(state, node.node_id, snapshot))
                    .transpose()?;
                node_view_from_state(state, node, table)
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(OxCalcTreeWorkspaceView {
            workspace_id: workspace_id.clone(),
            root_node_id: state.root_node_id,
            snapshot_id: state.snapshot.snapshot_id(),
            workspace_revision_id: state.workspace_revision.revision_id().clone(),
            workspace_revision_parent_id: state
                .workspace_revision_graph
                .current_parent_revision_id()
                .cloned(),
            retained_workspace_revision_count: state.workspace_revision_graph.entries().len(),
            workspace_revision_graph_entries: state
                .workspace_revision_graph
                .entries()
                .values()
                .cloned()
                .collect(),
            node_input_snapshot_id: state
                .workspace_revision
                .node_input_snapshot
                .snapshot_id()
                .clone(),
            namespace_snapshot_id: state
                .workspace_revision
                .namespace_snapshot
                .snapshot_id()
                .clone(),
            formula_binding_snapshot_id: state.formula_binding_snapshot.snapshot_id().clone(),
            dependency_shape_snapshot_id: state.dependency_shape_snapshot.snapshot_id().clone(),
            publication_snapshot_id: state.publication_snapshot.snapshot_id().clone(),
            runtime_overlay_set_id: state.runtime_overlay_set.overlay_set_id().clone(),
            value_epoch: state.value_epoch,
            nodes,
            tables: self.table_views_for_state(state)?,
            diagnostics: state
                .last_result
                .as_ref()
                .map_or_else(Vec::new, |result| result.diagnostics.clone()),
        })
    }

    pub fn node_view(
        &self,
        workspace_id: &OxCalcTreeWorkspaceId,
        node_id: TreeNodeId,
    ) -> Result<OxCalcTreeNodeView, OxCalcTreeContextError> {
        let state = self.workspace(workspace_id)?;
        let node = state
            .snapshot
            .try_get_node(node_id)
            .ok_or(StructuralError::UnknownNode { node_id })?;
        let table = state
            .table_snapshots
            .get(&node_id)
            .map(|snapshot| self.table_view_from_snapshot(state, node_id, snapshot))
            .transpose()?;
        node_view_from_state(state, node, table)
    }

    fn next_node_id(&self) -> TreeNodeId {
        TreeNodeId(self.next_node_id)
    }

    fn advance_node_id(&mut self) {
        self.next_node_id += 1;
    }

    fn next_snapshot_id(&self) -> StructuralSnapshotId {
        self.next_snapshot_id_after(0)
    }

    fn next_snapshot_id_after(&self, offset: u64) -> StructuralSnapshotId {
        StructuralSnapshotId(self.next_snapshot_id + offset)
    }

    fn advance_snapshot_id(&mut self) {
        self.advance_snapshot_id_by(1);
    }

    fn advance_snapshot_id_by(&mut self, count: u64) {
        self.next_snapshot_id += count;
    }

    fn next_candidate_index(&self) -> u64 {
        self.next_candidate_index
    }

    fn advance_candidate_index(&mut self) {
        self.next_candidate_index += 1;
    }

    fn next_candidate_overlay_handle(
        &self,
        workspace_id: &OxCalcTreeWorkspaceId,
    ) -> CandidateOverlayHandle {
        CandidateOverlayHandle::new(format!(
            "candidate-overlay:{}:{}",
            workspace_id.as_str(),
            self.next_candidate_overlay_index
        ))
    }

    fn advance_candidate_overlay_index(&mut self) {
        self.next_candidate_overlay_index += 1;
    }

    fn next_transaction_id(&self, workspace_id: &OxCalcTreeWorkspaceId) -> OxCalcTreeTransactionId {
        OxCalcTreeTransactionId::new(format!(
            "transaction:{}:{}",
            workspace_id.as_str(),
            self.next_transaction_index
        ))
    }

    fn advance_transaction_index(&mut self) {
        self.next_transaction_index += 1;
    }

    fn advance_allocators_past_state(&mut self, state: &OxCalcTreeWorkspaceState) {
        if let Some(max_node_id) = state.snapshot.nodes().keys().map(|node_id| node_id.0).max() {
            self.next_node_id = self.next_node_id.max(max_node_id + 1);
        }
        self.next_snapshot_id = self
            .next_snapshot_id
            .max(state.snapshot.snapshot_id().0 + 1);
    }

    fn workspace(
        &self,
        workspace_id: &OxCalcTreeWorkspaceId,
    ) -> Result<&OxCalcTreeWorkspaceState, OxCalcTreeContextError> {
        self.workspaces
            .get(workspace_id)
            .ok_or_else(|| OxCalcTreeContextError::UnknownWorkspace {
                workspace_id: workspace_id.as_str().to_string(),
            })
    }

    fn workspace_mut(
        &mut self,
        workspace_id: &OxCalcTreeWorkspaceId,
    ) -> Result<&mut OxCalcTreeWorkspaceState, OxCalcTreeContextError> {
        self.workspaces.get_mut(workspace_id).ok_or_else(|| {
            OxCalcTreeContextError::UnknownWorkspace {
                workspace_id: workspace_id.as_str().to_string(),
            }
        })
    }

    fn table_views_for_state(
        &self,
        state: &OxCalcTreeWorkspaceState,
    ) -> Result<Vec<OxCalcTreeTableView>, OxCalcTreeContextError> {
        state
            .table_snapshots
            .iter()
            .map(|(node_id, snapshot)| self.table_view_from_snapshot(state, *node_id, snapshot))
            .collect()
    }

    fn table_projections_for_state(
        &self,
        state: &OxCalcTreeWorkspaceState,
    ) -> Result<Vec<TreeCalcTableNodeProjection>, OxCalcTreeContextError> {
        state
            .table_snapshots
            .iter()
            .map(|(node_id, snapshot)| {
                let normalized = normalize_context_table_snapshot(state, *node_id, snapshot)?;
                project_treecalc_table_node_snapshot(&normalized)
                    .map_err(|error| OxCalcTreeContextError::TableProjection { error })
            })
            .collect()
    }

    fn table_view_from_snapshot(
        &self,
        state: &OxCalcTreeWorkspaceState,
        node_id: TreeNodeId,
        snapshot: &TreeCalcTableNodeSnapshot,
    ) -> Result<OxCalcTreeTableView, OxCalcTreeContextError> {
        let normalized = normalize_context_table_snapshot(state, node_id, snapshot)?;
        let projection = project_treecalc_table_node_snapshot(&normalized)
            .map_err(|error| OxCalcTreeContextError::TableProjection { error })?;
        let dependency_inventory = inventory_treecalc_table_dependency_facts(
            &normalized,
            &projection,
            &self.table_lifecycle_context_versions(state),
            None,
            false,
        );
        Ok(OxCalcTreeTableView {
            table_node_id: normalized.table_node_id,
            table_id: normalized.table_id.clone(),
            table_name: normalized.table_name.clone(),
            display_path: normalized.display_path.clone(),
            canonical_path: normalized.canonical_path.clone(),
            snapshot: normalized,
            projection,
            dependency_inventory,
        })
    }

    fn table_catalog_resolver_context(
        &self,
        workspace_id: &OxCalcTreeWorkspaceId,
    ) -> Result<TreeCalcTableCatalogResolverContext, OxCalcTreeContextError> {
        let current_state = self.workspace(workspace_id)?;
        let mut context = TreeCalcTableCatalogResolverContext::for_current_workspace(
            workspace_id.as_str(),
            self.table_projections_for_state(current_state)?,
        );
        let versions = self.table_lifecycle_context_versions(current_state);
        context.host_namespace_version = versions
            .host_namespace_version
            .clone()
            .unwrap_or_else(|| "treecalc-host-namespace:v1".to_string());
        context.structure_context_version = versions.structure_context_version;
        context.resolution_rule_version = versions.resolution_rule_version;

        for (other_workspace_id, other_state) in &self.workspaces {
            if other_workspace_id == workspace_id {
                continue;
            }
            context = context.with_workspace(TreeCalcTableCatalogWorkspace::available_current(
                other_workspace_id.as_str(),
                self.table_projections_for_state(other_state)?,
            ));
        }

        for state in self.workspaces.values() {
            for deleted_table in &state.deleted_table_facts {
                context = context.with_deleted_table(deleted_table.clone());
            }
        }

        Ok(context)
    }

    fn table_lifecycle_context_versions(
        &self,
        state: &OxCalcTreeWorkspaceState,
    ) -> TreeCalcTableLifecycleContextVersions {
        let runtime_context = self.options.runtime_context();
        TreeCalcTableLifecycleContextVersions {
            host_namespace_version: Some(runtime_context.host_namespace_version),
            structure_context_version: context_structure_version(state),
            registry_snapshot_identity: runtime_context.arg_preparation_profile_version,
            resolution_rule_version: runtime_context.resolution_rule_version,
            workspace_availability_version: Some(format!(
                "treecalc-workspace-availability:v1:{}:available",
                state.workspace_id.as_str()
            )),
            workspace_alias_version: Some(format!(
                "treecalc-workspace-alias:v1:{}:{}",
                state.workspace_id.as_str(),
                state.snapshot.snapshot_id().0
            )),
        }
    }
}

fn validate_workspace_snapshot(
    snapshot: &OxCalcTreeWorkspaceSnapshot,
) -> Result<(), OxCalcTreeContextError> {
    if snapshot.schema_version != OXCALC_TREE_WORKSPACE_SNAPSHOT_SCHEMA_V2 {
        return Err(OxCalcTreeContextError::InvalidWorkspaceSnapshot {
            detail: format!(
                "unsupported schema_version {}; expected {}",
                snapshot.schema_version, OXCALC_TREE_WORKSPACE_SNAPSHOT_SCHEMA_V2
            ),
        });
    }
    if snapshot.workspace_revision.workspace_id != snapshot.workspace_id.as_str() {
        return Err(OxCalcTreeContextError::InvalidWorkspaceSnapshot {
            detail: format!(
                "workspace_revision workspace_id {} does not match snapshot workspace_id {}",
                snapshot.workspace_revision.workspace_id,
                snapshot.workspace_id.as_str()
            ),
        });
    }

    let structural_snapshot = &snapshot.workspace_revision.structure_snapshot;
    let node_input_snapshot = &snapshot.workspace_revision.node_input_snapshot;
    let namespace_snapshot = &snapshot.workspace_revision.namespace_snapshot;

    if structural_snapshot.root_node_id() != snapshot.root_node_id {
        return Err(OxCalcTreeContextError::InvalidWorkspaceSnapshot {
            detail: format!(
                "root_node_id {} does not match structural snapshot root {}",
                snapshot.root_node_id,
                structural_snapshot.root_node_id()
            ),
        });
    }
    if structural_snapshot
        .try_get_node(snapshot.root_node_id)
        .is_none()
    {
        return Err(OxCalcTreeContextError::InvalidWorkspaceSnapshot {
            detail: format!("root node {} is missing", snapshot.root_node_id),
        });
    }

    let structural_node_ids = snapshot
        .workspace_revision
        .structure_snapshot
        .nodes()
        .keys()
        .copied()
        .collect::<std::collections::BTreeSet<_>>();
    let input_node_ids = node_input_snapshot
        .records()
        .keys()
        .copied()
        .collect::<std::collections::BTreeSet<_>>();
    if input_node_ids != structural_node_ids {
        return Err(OxCalcTreeContextError::InvalidWorkspaceSnapshot {
            detail: format!(
                "node_input_snapshot keys {:?} do not match structural node ids {:?}",
                input_node_ids, structural_node_ids
            ),
        });
    }

    for node_id in snapshot
        .table_snapshots
        .keys()
        .chain(snapshot.publication_values.keys())
    {
        if structural_snapshot.try_get_node(*node_id).is_none() {
            return Err(OxCalcTreeContextError::InvalidWorkspaceSnapshot {
                detail: format!("node-scoped snapshot data references unknown node {node_id}"),
            });
        }
    }

    for record in node_input_snapshot.records().values() {
        let text_required = matches!(
            record.kind,
            NodeInputKind::Literal | NodeInputKind::FormulaText | NodeInputKind::HostOwned
        );
        if text_required && record.text.is_none() {
            return Err(OxCalcTreeContextError::InvalidWorkspaceSnapshot {
                detail: format!(
                    "node input record for {} is {:?} but has no input text",
                    record.node_id, record.kind
                ),
            });
        }
        if record.kind == NodeInputKind::Empty && record.text.is_some() {
            return Err(OxCalcTreeContextError::InvalidWorkspaceSnapshot {
                detail: format!(
                    "empty node input record for {} carries text",
                    record.node_id
                ),
            });
        }
    }
    let max_non_formula_input_epoch = node_input_snapshot
        .records()
        .values()
        .filter(|record| {
            matches!(
                record.kind,
                NodeInputKind::Literal | NodeInputKind::HostOwned
            ) || (record.kind == NodeInputKind::Empty && record.input_epoch > 1)
        })
        .map(|record| record.input_epoch)
        .max()
        .unwrap_or_default();
    if snapshot.input_epoch_watermark < max_non_formula_input_epoch {
        return Err(OxCalcTreeContextError::InvalidWorkspaceSnapshot {
            detail: format!(
                "input_epoch_watermark {} is below max non-formula input epoch {}",
                snapshot.input_epoch_watermark, max_non_formula_input_epoch
            ),
        });
    }

    let recomputed_node_input =
        NodeInputSnapshot::from_record_map(node_input_snapshot.records().clone());
    if recomputed_node_input.snapshot_id() != node_input_snapshot.snapshot_id() {
        return Err(OxCalcTreeContextError::InvalidWorkspaceSnapshot {
            detail: format!(
                "node_input_snapshot id {} does not match its records",
                node_input_snapshot.snapshot_id()
            ),
        });
    }
    let recomputed_namespace = NamespaceSnapshot::new(
        namespace_snapshot.host_namespace_version.clone(),
        namespace_snapshot.function_registry_version.clone(),
        namespace_snapshot.capability_profile_id.clone(),
        namespace_snapshot.resolution_rule_version.clone(),
        namespace_snapshot.caller_context_identity_version.clone(),
        namespace_snapshot.workspace_availability_version.clone(),
        namespace_snapshot.workspace_alias_version.clone(),
    );
    if recomputed_namespace.snapshot_id() != namespace_snapshot.snapshot_id() {
        return Err(OxCalcTreeContextError::InvalidWorkspaceSnapshot {
            detail: format!(
                "namespace_snapshot id {} does not match its facts",
                namespace_snapshot.snapshot_id()
            ),
        });
    }
    let recomputed_revision = WorkspaceRevision::new(
        snapshot.workspace_id.as_str(),
        structural_snapshot.clone(),
        recomputed_node_input,
        recomputed_namespace,
    );
    if recomputed_revision.revision_id() != snapshot.workspace_revision.revision_id() {
        return Err(OxCalcTreeContextError::InvalidWorkspaceSnapshot {
            detail: format!(
                "workspace_revision id {} does not match its roots",
                snapshot.workspace_revision.revision_id()
            ),
        });
    }
    if snapshot.formula_binding_snapshot.revision_id != *snapshot.workspace_revision.revision_id() {
        return Err(OxCalcTreeContextError::InvalidWorkspaceSnapshot {
            detail: "formula_binding_snapshot revision does not match workspace_revision"
                .to_string(),
        });
    }
    if snapshot.dependency_shape_snapshot.revision_id != *snapshot.workspace_revision.revision_id()
    {
        return Err(OxCalcTreeContextError::InvalidWorkspaceSnapshot {
            detail: "dependency_shape_snapshot revision does not match workspace_revision"
                .to_string(),
        });
    }
    if snapshot
        .dependency_shape_snapshot
        .formula_binding_snapshot_id
        != *snapshot.formula_binding_snapshot.snapshot_id()
    {
        return Err(OxCalcTreeContextError::InvalidWorkspaceSnapshot {
            detail: "dependency_shape_snapshot does not point at the formula_binding_snapshot"
                .to_string(),
        });
    }
    if snapshot.runtime_overlay_set.publication_snapshot_id
        != *snapshot.publication_snapshot.snapshot_id()
    {
        return Err(OxCalcTreeContextError::InvalidWorkspaceSnapshot {
            detail: "runtime_overlay_set does not point at the publication_snapshot".to_string(),
        });
    }

    for (node_id, table_snapshot) in &snapshot.table_snapshots {
        if table_snapshot.table_node_id != *node_id {
            return Err(OxCalcTreeContextError::InvalidWorkspaceSnapshot {
                detail: format!(
                    "table snapshot for node {node_id} declares table_node_id {}",
                    table_snapshot.table_node_id
                ),
            });
        }
    }

    Ok(())
}

fn node_kind_for_formula_text(formula_text: &str) -> StructuralNodeKind {
    if is_formula_text(formula_text) {
        StructuralNodeKind::Calculation
    } else if formula_text.is_empty() {
        StructuralNodeKind::Container
    } else {
        StructuralNodeKind::Constant
    }
}

fn direct_context_node_input_record(
    node_id: TreeNodeId,
    formula_text: &str,
    input_epoch: u64,
) -> NodeInputRecord {
    if formula_text.is_empty() {
        NodeInputRecord::empty(node_id, input_epoch)
    } else if is_formula_text(formula_text) {
        NodeInputRecord::formula_text(node_id, formula_text, input_epoch)
    } else {
        NodeInputRecord::literal(node_id, formula_text, input_epoch)
    }
}

fn node_input_kind_for_formula_edit_text(formula_text: &str) -> NodeInputKind {
    if formula_text.is_empty() {
        NodeInputKind::Empty
    } else if is_formula_text(formula_text) {
        NodeInputKind::FormulaText
    } else {
        NodeInputKind::Literal
    }
}

fn namespace_snapshot_for_context(
    options: &OxCalcTreeContextOptions,
    workspace_id: &OxCalcTreeWorkspaceId,
) -> NamespaceSnapshot {
    let runtime_context = options.runtime_context();
    NamespaceSnapshot::new(
        runtime_context.host_namespace_version,
        runtime_context.arg_preparation_profile_version,
        runtime_context.capability_profile_id,
        runtime_context.resolution_rule_version,
        runtime_context.caller_context_identity_version,
        runtime_context
            .cross_workspace_availability_version
            .or_else(|| {
                Some(format!(
                    "treecalc-workspace-availability:v1:{}:available",
                    workspace_id.as_str()
                ))
            }),
        Some(format!(
            "treecalc-workspace-alias:v1:{}:default",
            workspace_id.as_str()
        )),
    )
}

fn runtime_context_for_workspace_state(
    options: &OxCalcTreeContextOptions,
    state: &OxCalcTreeWorkspaceState,
) -> LocalTreeCalcEnvironmentContext {
    let namespace_snapshot = &state.workspace_revision.namespace_snapshot;
    let mut runtime_context = options.runtime_context();
    runtime_context.host_namespace_version = namespace_snapshot.host_namespace_version.clone();
    runtime_context.arg_preparation_profile_version =
        namespace_snapshot.function_registry_version.clone();
    runtime_context.capability_profile_id = namespace_snapshot.capability_profile_id.clone();
    runtime_context.resolution_rule_version = namespace_snapshot.resolution_rule_version.clone();
    runtime_context.caller_context_identity_version =
        namespace_snapshot.caller_context_identity_version.clone();
    runtime_context.cross_workspace_availability_version = options
        .namespace
        .cross_workspace_availability_version
        .clone();
    runtime_context.meta_node_ids = state.snapshot.meta_node_ids();
    runtime_context
}

fn replace_node_input_record(state: &mut OxCalcTreeWorkspaceState, record: NodeInputRecord) {
    let node_input_snapshot = state
        .workspace_revision
        .node_input_snapshot
        .with_record(record);
    replace_node_input_snapshot(state, node_input_snapshot);
}

fn successor_non_formula_input_record(
    state: &mut OxCalcTreeWorkspaceState,
    node_id: TreeNodeId,
    input_value: String,
) -> NodeInputRecord {
    let input_epoch = bump_input_value_epoch(state);
    if input_value.is_empty() {
        NodeInputRecord::empty(node_id, input_epoch)
    } else {
        NodeInputRecord::literal(node_id, input_value, input_epoch)
    }
}

fn replace_non_formula_node_input_record(
    state: &mut OxCalcTreeWorkspaceState,
    record: NodeInputRecord,
) {
    match record.kind {
        NodeInputKind::Empty | NodeInputKind::Literal => {}
        NodeInputKind::FormulaText | NodeInputKind::HostOwned => {
            unreachable!("non-formula input replacement received a formula or host-owned record")
        }
    }
    replace_node_input_record(state, record);
}

fn replace_formula_text_node_input_record(
    state: &mut OxCalcTreeWorkspaceState,
    record: NodeInputRecord,
) {
    match record.kind {
        NodeInputKind::FormulaText => {}
        NodeInputKind::Empty | NodeInputKind::Literal | NodeInputKind::HostOwned => {
            unreachable!("formula text replacement received a non-formula record")
        }
    }
    replace_node_input_record(state, record);
}

fn remove_node_input_record(state: &mut OxCalcTreeWorkspaceState, node_id: TreeNodeId) {
    let node_input_snapshot = state
        .workspace_revision
        .node_input_snapshot
        .without_record(node_id);
    replace_node_input_snapshot(state, node_input_snapshot);
}

fn replace_node_input_snapshot(
    state: &mut OxCalcTreeWorkspaceState,
    node_input_snapshot: NodeInputSnapshot,
) {
    let namespace_snapshot = state.workspace_revision.namespace_snapshot.clone();
    let workspace_revision = WorkspaceRevision::new(
        state.workspace_id.as_str(),
        (*state.snapshot).clone(),
        node_input_snapshot,
        namespace_snapshot,
    );
    replace_workspace_revision(state, workspace_revision);
    refresh_formula_and_dependency_absent_layer_shells(state);
    retain_current_workspace_revision(state);
}

fn replace_namespace_snapshot(
    state: &mut OxCalcTreeWorkspaceState,
    namespace_snapshot: NamespaceSnapshot,
) {
    let node_input_snapshot = state.workspace_revision.node_input_snapshot.clone();
    let workspace_revision = WorkspaceRevision::new(
        state.workspace_id.as_str(),
        (*state.snapshot).clone(),
        node_input_snapshot,
        namespace_snapshot,
    );
    replace_workspace_revision(state, workspace_revision);
    refresh_formula_and_dependency_absent_layer_shells(state);
    retain_current_workspace_revision(state);
}

fn seed_namespace_recalc_invalidation(state: &mut OxCalcTreeWorkspaceState) {
    let formula_node_ids = state
        .workspace_revision
        .node_input_snapshot
        .records()
        .values()
        .filter(|record| record.kind == NodeInputKind::FormulaText)
        .map(|record| record.node_id)
        .collect::<Vec<_>>();
    for node_id in formula_node_ids {
        push_pending_invalidation_seed(
            state,
            node_id,
            InvalidationReasonKind::StructuralRecalcOnly,
        );
    }
}

fn clear_pending_edit_transition_facts(state: &mut OxCalcTreeWorkspaceState) {
    state.pending_formula_edit_diagnostics.clear();
    state.pending_node_input_kind_transitions.clear();
    state.pending_dependency_shape_updates.clear();
}

fn refresh_workspace_revision_and_absent_layers(state: &mut OxCalcTreeWorkspaceState) {
    let node_input_snapshot = state.workspace_revision.node_input_snapshot.clone();
    let namespace_snapshot = state.workspace_revision.namespace_snapshot.clone();
    let workspace_revision = WorkspaceRevision::new(
        state.workspace_id.as_str(),
        (*state.snapshot).clone(),
        node_input_snapshot,
        namespace_snapshot,
    );
    replace_workspace_revision(state, workspace_revision);
    refresh_absent_snapshot_layer_shells(state);
    retain_current_workspace_revision(state);
}

fn replace_workspace_revision(
    state: &mut OxCalcTreeWorkspaceState,
    workspace_revision: WorkspaceRevision,
) {
    let predecessor_revision_id = state.workspace_revision.revision_id().clone();
    state.workspace_revision = Arc::new(workspace_revision);
    state.workspace_revision_graph.record_successor(
        &predecessor_revision_id,
        &state.workspace_revision,
        None,
        None,
    );
}

fn workspace_revision_transaction_summary(
    transaction_id: &OxCalcTreeTransactionId,
    calculation: Option<&OxCalcTreeCalculationOutcome>,
) -> Option<WorkspaceRevisionTransactionSummary> {
    let calculation = calculation?;
    let invalidated_nodes = calculation
        .invalidation_closure
        .impacted_order
        .iter()
        .filter_map(|node_id| calculation.invalidation_closure.records.get(node_id))
        .map(|record| WorkspaceRevisionInvalidationSummaryEntry {
            node_id: record.node_id,
            requires_rebind: record.requires_rebind,
            reasons: record.reasons.clone(),
        })
        .collect::<Vec<_>>();
    let requires_rebind = invalidated_nodes
        .iter()
        .filter(|entry| entry.requires_rebind)
        .map(|entry| entry.node_id)
        .collect::<Vec<_>>();
    Some(WorkspaceRevisionTransactionSummary {
        transaction_id: transaction_id.to_string(),
        estimated_node_count: invalidated_nodes.len(),
        invalidated_nodes,
        requires_rebind,
    })
}

fn workspace_revision_transaction_summary_from_plan(
    transaction_id: &OxCalcTreeTransactionId,
    plan: OxCalcTreeInvalidationPlan,
) -> WorkspaceRevisionTransactionSummary {
    let invalidated_nodes = plan
        .invalidated_nodes
        .into_iter()
        .map(|entry| WorkspaceRevisionInvalidationSummaryEntry {
            node_id: entry.node_id,
            requires_rebind: entry.requires_rebind,
            reasons: entry.reasons,
        })
        .collect::<Vec<_>>();
    WorkspaceRevisionTransactionSummary {
        transaction_id: transaction_id.to_string(),
        estimated_node_count: invalidated_nodes.len(),
        requires_rebind: invalidated_nodes
            .iter()
            .filter(|entry| entry.requires_rebind)
            .map(|entry| entry.node_id)
            .collect(),
        invalidated_nodes,
    }
}

fn candidate_pressure_for(
    candidates: &BTreeMap<CandidateOverlayHandle, CandidateOverlayState>,
    policy: &OxCalcTreeCandidateReapPolicy,
) -> OxCalcTreeCandidatePressure {
    let retained_candidate_count = candidates.len();
    let child_protected_handles = candidates
        .values()
        .filter_map(|candidate| candidate.parent_candidate.as_ref())
        .cloned()
        .collect::<BTreeSet<_>>();
    let host_pinned_handles = candidates
        .iter()
        .filter_map(|(handle, candidate)| {
            (candidate.retention_pin_count > 0).then(|| handle.clone())
        })
        .collect::<BTreeSet<_>>();
    let protected_handles = child_protected_handles
        .union(&host_pinned_handles)
        .cloned()
        .collect::<BTreeSet<_>>();
    let child_protected_candidate_count = child_protected_handles.len();
    let host_pinned_candidate_count = host_pinned_handles.len();
    let protected_candidate_count = protected_handles.len();
    let reclaimable_candidate_count =
        retained_candidate_count.saturating_sub(protected_candidate_count);
    OxCalcTreeCandidatePressure {
        retained_candidate_count,
        child_protected_candidate_count,
        host_pinned_candidate_count,
        protected_candidate_count,
        reclaimable_candidate_count,
        over_budget_candidate_count: retained_candidate_count
            .saturating_sub(policy.max_retained_candidates),
    }
}

#[derive(Debug, Clone, Default)]
struct CandidateRebaseTouches {
    content_nodes: BTreeSet<TreeNodeId>,
    structural_lanes: BTreeMap<TreeNodeId, BTreeSet<CandidateRebaseLaneTouch>>,
    structural_nodes: BTreeSet<TreeNodeId>,
    renamed_nodes: BTreeSet<TreeNodeId>,
    moved_nodes: BTreeSet<TreeNodeId>,
    reordered_nodes: BTreeSet<TreeNodeId>,
    deleted_nodes: BTreeSet<TreeNodeId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum CandidateRebaseLaneTouch {
    Add,
    Rename(TreeNodeId),
    MoveIn(TreeNodeId),
    MoveOut(TreeNodeId),
    Reorder,
    Delete,
    Meta(TreeNodeId),
}

impl CandidateRebaseTouches {
    fn touched_nodes(&self) -> Vec<TreeNodeId> {
        self.content_nodes
            .iter()
            .chain(self.structural_lanes.keys())
            .chain(self.structural_nodes.iter())
            .chain(self.deleted_nodes.iter())
            .copied()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }

    fn conflicting_nodes_with(&self, other: &Self) -> Vec<TreeNodeId> {
        let mut conflicts = BTreeSet::new();
        conflicts.extend(
            self.content_nodes
                .intersection(&other.content_nodes)
                .copied(),
        );
        for lane in self
            .structural_lanes
            .keys()
            .filter(|lane| other.structural_lanes.contains_key(*lane))
        {
            if lane_touches_conflict(
                self.structural_lanes
                    .get(lane)
                    .expect("lane key came from this map"),
                other
                    .structural_lanes
                    .get(lane)
                    .expect("lane key was checked in other map"),
            ) {
                conflicts.insert(*lane);
            }
        }
        conflicts.extend(
            self.structural_nodes
                .intersection(&other.structural_nodes)
                .filter(|node_id| {
                    !compatible_rename_move_node(self, other, **node_id)
                        && !compatible_rename_reorder_node(self, other, **node_id)
                })
                .copied(),
        );
        conflicts.extend(
            self.deleted_nodes
                .intersection(&other.touched_node_set())
                .copied(),
        );
        conflicts.extend(
            other
                .deleted_nodes
                .intersection(&self.touched_node_set())
                .copied(),
        );
        conflicts.into_iter().collect()
    }

    fn touched_node_set(&self) -> BTreeSet<TreeNodeId> {
        self.touched_nodes().into_iter().collect()
    }

    fn touch_lane(&mut self, lane: TreeNodeId, touch: CandidateRebaseLaneTouch) {
        self.structural_lanes.entry(lane).or_default().insert(touch);
    }
}

fn lane_touches_conflict(
    left: &BTreeSet<CandidateRebaseLaneTouch>,
    right: &BTreeSet<CandidateRebaseLaneTouch>,
) -> bool {
    left.iter().any(|left_touch| {
        right
            .iter()
            .any(|right_touch| lane_touch_conflicts(*left_touch, *right_touch))
    })
}

fn lane_touch_conflicts(left: CandidateRebaseLaneTouch, right: CandidateRebaseLaneTouch) -> bool {
    let compatible = matches!(
        (left, right),
        (CandidateRebaseLaneTouch::Rename(node), CandidateRebaseLaneTouch::MoveIn(other))
            | (CandidateRebaseLaneTouch::Rename(node), CandidateRebaseLaneTouch::MoveOut(other))
            | (CandidateRebaseLaneTouch::MoveIn(node), CandidateRebaseLaneTouch::Rename(other))
            | (CandidateRebaseLaneTouch::MoveOut(node), CandidateRebaseLaneTouch::Rename(other))
            if node == other
    ) || matches!(
        (left, right),
        (
            CandidateRebaseLaneTouch::Rename(_),
            CandidateRebaseLaneTouch::Add
        ) | (
            CandidateRebaseLaneTouch::Add,
            CandidateRebaseLaneTouch::Rename(_)
        ) | (
            CandidateRebaseLaneTouch::Rename(_),
            CandidateRebaseLaneTouch::Reorder
        ) | (
            CandidateRebaseLaneTouch::Reorder,
            CandidateRebaseLaneTouch::Rename(_)
        ) | (
            CandidateRebaseLaneTouch::Add,
            CandidateRebaseLaneTouch::Reorder
        ) | (
            CandidateRebaseLaneTouch::Reorder,
            CandidateRebaseLaneTouch::Add
        ) | (
            CandidateRebaseLaneTouch::Add,
            CandidateRebaseLaneTouch::Delete
        ) | (
            CandidateRebaseLaneTouch::Delete,
            CandidateRebaseLaneTouch::Add
        ) | (
            CandidateRebaseLaneTouch::Delete,
            CandidateRebaseLaneTouch::Reorder
        ) | (
            CandidateRebaseLaneTouch::Reorder,
            CandidateRebaseLaneTouch::Delete
        )
    );
    !compatible
}

fn compatible_rename_move_node(
    left: &CandidateRebaseTouches,
    right: &CandidateRebaseTouches,
    node_id: TreeNodeId,
) -> bool {
    (left.renamed_nodes.contains(&node_id) && right.moved_nodes.contains(&node_id))
        || (left.moved_nodes.contains(&node_id) && right.renamed_nodes.contains(&node_id))
}

fn compatible_rename_reorder_node(
    left: &CandidateRebaseTouches,
    right: &CandidateRebaseTouches,
    node_id: TreeNodeId,
) -> bool {
    (left.renamed_nodes.contains(&node_id) && right.reordered_nodes.contains(&node_id))
        || (left.reordered_nodes.contains(&node_id) && right.renamed_nodes.contains(&node_id))
}

fn rebase_touches_for_candidate_transactions(
    transactions: &[OxCalcTreeEditTransaction],
    basis_snapshot: &StructuralSnapshot,
    root_node_id: TreeNodeId,
) -> CandidateRebaseTouches {
    let mut touches = CandidateRebaseTouches::default();
    let mut rolling_snapshot = basis_snapshot.clone();
    for transaction in transactions {
        for edit in &transaction.edits {
            touches.extend(rebase_touches_for_edit(
                edit,
                &rolling_snapshot,
                root_node_id,
            ));
            if let Some(structural_edit) =
                structural_edit_for_touch_replay(edit, &rolling_snapshot, root_node_id)
            {
                if let Ok(outcome) =
                    rolling_snapshot.apply_edit(rolling_snapshot.snapshot_id(), structural_edit)
                {
                    rolling_snapshot = outcome.snapshot;
                }
            }
        }
    }
    touches
}

impl CandidateRebaseTouches {
    fn extend(&mut self, other: CandidateRebaseTouches) {
        self.content_nodes.extend(other.content_nodes);
        for (lane, lane_touches) in other.structural_lanes {
            self.structural_lanes
                .entry(lane)
                .or_default()
                .extend(lane_touches);
        }
        self.structural_nodes.extend(other.structural_nodes);
        self.renamed_nodes.extend(other.renamed_nodes);
        self.moved_nodes.extend(other.moved_nodes);
        self.reordered_nodes.extend(other.reordered_nodes);
        self.deleted_nodes.extend(other.deleted_nodes);
    }
}

fn rebase_touches_for_edit(
    edit: &OxCalcTreeEdit,
    snapshot: &StructuralSnapshot,
    root_node_id: TreeNodeId,
) -> CandidateRebaseTouches {
    let mut touches = CandidateRebaseTouches::default();
    match edit {
        OxCalcTreeEdit::AddNode { request } => {
            touches.touch_lane(
                request.parent_node_id.unwrap_or(root_node_id),
                CandidateRebaseLaneTouch::Add,
            );
            if let Some(reserved_node_id) = request.reserved_node_id {
                touches.structural_nodes.insert(reserved_node_id);
            }
        }
        OxCalcTreeEdit::SetNodeInput { node_id, .. }
        | OxCalcTreeEdit::SetNodeFormulaText { node_id, .. }
        | OxCalcTreeEdit::SetNodeTable { node_id, .. } => {
            touches.content_nodes.insert(*node_id);
        }
        OxCalcTreeEdit::SetNodeMeta { node_id, .. } => {
            touches.structural_nodes.insert(*node_id);
            if let Some(parent_id) = snapshot.parent_id_of(*node_id) {
                touches.touch_lane(parent_id, CandidateRebaseLaneTouch::Meta(*node_id));
            }
        }
        OxCalcTreeEdit::RenameNode { node_id, .. } => {
            touches.structural_nodes.insert(*node_id);
            touches.renamed_nodes.insert(*node_id);
            if let Some(parent_id) = snapshot.parent_id_of(*node_id) {
                touches.touch_lane(parent_id, CandidateRebaseLaneTouch::Rename(*node_id));
            }
        }
        OxCalcTreeEdit::ReorderNode { node_id, .. } => {
            touches.structural_nodes.insert(*node_id);
            touches.reordered_nodes.insert(*node_id);
            if let Some(parent_id) = snapshot.parent_id_of(*node_id) {
                touches.touch_lane(parent_id, CandidateRebaseLaneTouch::Reorder);
            }
        }
        OxCalcTreeEdit::DeleteNode { node_id } => {
            if let Some(parent_id) = snapshot.parent_id_of(*node_id) {
                touches.touch_lane(parent_id, CandidateRebaseLaneTouch::Delete);
            }
            touches
                .deleted_nodes
                .extend(descendant_or_self_node_ids(snapshot, *node_id));
        }
        OxCalcTreeEdit::MoveNode {
            node_id,
            new_parent_id,
            ..
        } => {
            touches.structural_nodes.insert(*node_id);
            touches.moved_nodes.insert(*node_id);
            touches.touch_lane(*new_parent_id, CandidateRebaseLaneTouch::MoveIn(*node_id));
            if let Some(old_parent_id) = snapshot.parent_id_of(*node_id) {
                touches.touch_lane(old_parent_id, CandidateRebaseLaneTouch::MoveOut(*node_id));
            }
        }
        OxCalcTreeEdit::SetReferenceCollectionMembership {
            owner_node_id,
            member_node_ids,
            ..
        } => {
            touches.content_nodes.insert(*owner_node_id);
            touches
                .content_nodes
                .extend(member_node_ids.iter().copied());
        }
    }
    touches
}

fn structural_edit_for_touch_replay(
    edit: &OxCalcTreeEdit,
    snapshot: &StructuralSnapshot,
    root_node_id: TreeNodeId,
) -> Option<StructuralEdit> {
    match edit {
        OxCalcTreeEdit::AddNode { request } => {
            let node_id = request.reserved_node_id?;
            Some(StructuralEdit::InsertNode {
                node: StructuralNode {
                    node_id,
                    kind: node_kind_for_formula_text(&request.formula_text),
                    symbol: request.symbol.clone(),
                    parent_id: Some(request.parent_node_id.unwrap_or(root_node_id)),
                    child_ids: Vec::new(),
                    role: None,
                    is_meta: false,
                },
                parent_id: request.parent_node_id.unwrap_or(root_node_id),
                index: None,
            })
        }
        OxCalcTreeEdit::RenameNode {
            node_id,
            new_symbol,
        } => Some(StructuralEdit::RenameNode {
            node_id: *node_id,
            new_symbol: new_symbol.clone(),
        }),
        OxCalcTreeEdit::MoveNode {
            node_id,
            new_parent_id,
            new_index,
        } => Some(StructuralEdit::MoveNode {
            node_id: *node_id,
            new_parent_id: *new_parent_id,
            new_index: *new_index,
        }),
        OxCalcTreeEdit::ReorderNode { node_id, new_index } => {
            let parent_id = snapshot.parent_id_of(*node_id)?;
            Some(StructuralEdit::MoveNode {
                node_id: *node_id,
                new_parent_id: parent_id,
                new_index: Some(*new_index),
            })
        }
        OxCalcTreeEdit::DeleteNode { node_id } => {
            Some(StructuralEdit::RemoveNode { node_id: *node_id })
        }
        OxCalcTreeEdit::SetNodeTable { .. } => None,
        OxCalcTreeEdit::SetNodeInput { .. }
        | OxCalcTreeEdit::SetNodeFormulaText { .. }
        | OxCalcTreeEdit::SetNodeMeta { .. }
        | OxCalcTreeEdit::SetReferenceCollectionMembership { .. } => None,
    }
}

fn descendant_or_self_node_ids(
    snapshot: &StructuralSnapshot,
    node_id: TreeNodeId,
) -> Vec<TreeNodeId> {
    let mut touched = Vec::new();
    let mut stack = vec![node_id];
    while let Some(current) = stack.pop() {
        let Some(node) = snapshot.try_get_node(current) else {
            continue;
        };
        touched.push(current);
        for child_id in node.child_ids.iter().rev() {
            stack.push(*child_id);
        }
    }
    touched
}

fn rebase_touches_between_revisions(
    basis: &WorkspaceRevision,
    current: &WorkspaceRevision,
) -> CandidateRebaseTouches {
    let mut touches = CandidateRebaseTouches::default();
    for node_id in basis
        .node_input_snapshot
        .records()
        .keys()
        .chain(current.node_input_snapshot.records().keys())
    {
        if basis.node_input_snapshot.try_get_record(*node_id)
            != current.node_input_snapshot.try_get_record(*node_id)
        {
            touches.content_nodes.insert(*node_id);
        }
    }
    for node_id in basis
        .structure_snapshot
        .nodes()
        .keys()
        .chain(current.structure_snapshot.nodes().keys())
    {
        let basis_node = basis.structure_snapshot.try_get_node(*node_id);
        let current_node = current.structure_snapshot.try_get_node(*node_id);
        match (basis_node, current_node) {
            (Some(basis_node), Some(current_node)) => {
                if basis_node.parent_id != current_node.parent_id {
                    touches.structural_nodes.insert(*node_id);
                    touches.moved_nodes.insert(*node_id);
                    if let Some(parent_id) = basis_node.parent_id {
                        touches.touch_lane(parent_id, CandidateRebaseLaneTouch::MoveOut(*node_id));
                    }
                    if let Some(parent_id) = current_node.parent_id {
                        touches.touch_lane(parent_id, CandidateRebaseLaneTouch::MoveIn(*node_id));
                    }
                }
                if basis_node.symbol != current_node.symbol || basis_node.kind != current_node.kind
                {
                    touches.structural_nodes.insert(*node_id);
                    touches.renamed_nodes.insert(*node_id);
                    if let Some(parent_id) = basis_node.parent_id.or(current_node.parent_id) {
                        touches.touch_lane(parent_id, CandidateRebaseLaneTouch::Rename(*node_id));
                    }
                }
                if basis_node.child_ids != current_node.child_ids
                    && same_child_members(&basis_node.child_ids, &current_node.child_ids)
                {
                    touches.touch_lane(*node_id, CandidateRebaseLaneTouch::Reorder);
                }
            }
            (Some(basis_node), None) => {
                touches.deleted_nodes.insert(*node_id);
                if let Some(parent_id) = basis_node.parent_id {
                    touches.touch_lane(parent_id, CandidateRebaseLaneTouch::Delete);
                }
            }
            (None, Some(current_node)) => {
                touches.structural_nodes.insert(*node_id);
                if let Some(parent_id) = current_node.parent_id {
                    touches.touch_lane(parent_id, CandidateRebaseLaneTouch::Add);
                }
            }
            (None, None) => {}
        }
    }
    touches
}

fn same_child_members(left: &[TreeNodeId], right: &[TreeNodeId]) -> bool {
    left.iter().copied().collect::<BTreeSet<_>>() == right.iter().copied().collect::<BTreeSet<_>>()
}

fn preview_mutations_for_candidate_transaction(
    transaction: &OxCalcTreeEditTransaction,
) -> Option<Vec<OxCalcTreePreviewMutation>> {
    transaction
        .edits
        .iter()
        .map(preview_mutation_for_candidate_edit)
        .collect()
}

fn preview_mutation_for_candidate_edit(edit: &OxCalcTreeEdit) -> Option<OxCalcTreePreviewMutation> {
    match edit {
        OxCalcTreeEdit::SetNodeInput { node_id, input } => {
            Some(OxCalcTreePreviewMutation::SetNodeFormulaText {
                node_id: *node_id,
                formula_text: input.clone(),
            })
        }
        OxCalcTreeEdit::SetNodeFormulaText {
            node_id,
            formula_text,
        } => Some(OxCalcTreePreviewMutation::SetNodeFormulaText {
            node_id: *node_id,
            formula_text: formula_text.clone(),
        }),
        OxCalcTreeEdit::RenameNode { node_id, .. } => {
            Some(OxCalcTreePreviewMutation::RenameNode { node_id: *node_id })
        }
        OxCalcTreeEdit::MoveNode { node_id, .. } => {
            Some(OxCalcTreePreviewMutation::MoveNode { node_id: *node_id })
        }
        OxCalcTreeEdit::ReorderNode { node_id, .. } => {
            Some(OxCalcTreePreviewMutation::ReorderNode { node_id: *node_id })
        }
        OxCalcTreeEdit::DeleteNode { node_id } => {
            Some(OxCalcTreePreviewMutation::DeleteNode { node_id: *node_id })
        }
        OxCalcTreeEdit::AddNode { .. }
        | OxCalcTreeEdit::SetNodeTable { .. }
        | OxCalcTreeEdit::SetNodeMeta { .. }
        | OxCalcTreeEdit::SetReferenceCollectionMembership { .. } => None,
    }
}

fn retain_current_workspace_revision(state: &mut OxCalcTreeWorkspaceState) {
    let revision_id = state.workspace_revision.revision_id().clone();
    if !state
        .retained_workspace_revisions
        .contains_key(&revision_id)
    {
        state
            .retained_workspace_revision_order
            .push_back(revision_id.clone());
    }
    state.retained_workspace_revisions.insert(
        revision_id.clone(),
        RetainedWorkspaceRevisionState {
            root_node_id: state.root_node_id,
            // Arc bumps: retained entries share the heavy payloads with the
            // live state; copy-on-write happens at the mutation sites.
            snapshot: Arc::clone(&state.snapshot),
            workspace_revision: Arc::clone(&state.workspace_revision),
            formula_binding_snapshot: state.formula_binding_snapshot.clone(),
            dependency_shape_snapshot: state.dependency_shape_snapshot.clone(),
            publication_snapshot: Arc::clone(&state.publication_snapshot),
            runtime_overlay_set: Arc::clone(&state.runtime_overlay_set),
            value_epoch: state.value_epoch,
            publication_value_epoch: state.publication_value_epoch,
            table_snapshots: Arc::clone(&state.table_snapshots),
            deleted_table_facts: state.deleted_table_facts.clone(),
            table_state_version: state.table_state_version,
            publication_payload: Arc::clone(&state.publication_payload),
            last_result: state.last_result.clone(),
        },
    );
    enforce_workspace_revision_retention_policy(state);
}

fn enforce_workspace_revision_retention_policy(state: &mut OxCalcTreeWorkspaceState) {
    let max_retained_revisions = state
        .revision_retention_policy
        .max_retained_revisions
        .max(1);
    let mut skipped = 0usize;
    while state.retained_workspace_revisions.len() > max_retained_revisions {
        let Some(candidate) = state.retained_workspace_revision_order.pop_front() else {
            break;
        };
        if candidate == *state.workspace_revision.revision_id()
            || state
                .candidate_pinned_workspace_revisions
                .contains_key(&candidate)
        {
            state.retained_workspace_revision_order.push_back(candidate);
            skipped += 1;
            if skipped >= state.retained_workspace_revision_order.len() {
                break;
            }
            continue;
        }
        skipped = 0;
        state.retained_workspace_revisions.remove(&candidate);
        let _ = state.workspace_revision_graph.evict(&candidate);
    }
}

fn pin_candidate_basis_revision(
    state: &mut OxCalcTreeWorkspaceState,
    revision_id: WorkspaceRevisionId,
) {
    *state
        .candidate_pinned_workspace_revisions
        .entry(revision_id)
        .or_insert(0) += 1;
    enforce_workspace_revision_retention_policy(state);
}

fn unpin_candidate_basis_revision(
    state: &mut OxCalcTreeWorkspaceState,
    revision_id: &WorkspaceRevisionId,
) {
    let Some(count) = state
        .candidate_pinned_workspace_revisions
        .get_mut(revision_id)
    else {
        enforce_workspace_revision_retention_policy(state);
        return;
    };
    *count = count.saturating_sub(1);
    if *count == 0 {
        state
            .candidate_pinned_workspace_revisions
            .remove(revision_id);
    }
    enforce_workspace_revision_retention_policy(state);
}

fn restore_retained_workspace_revision(
    state: &mut OxCalcTreeWorkspaceState,
    retained: RetainedWorkspaceRevisionState,
) {
    state.root_node_id = retained.root_node_id;
    state.snapshot = retained.snapshot;
    state.workspace_revision = retained.workspace_revision;
    state.formula_binding_snapshot = retained.formula_binding_snapshot;
    state.dependency_shape_snapshot = retained.dependency_shape_snapshot;
    state.publication_snapshot = retained.publication_snapshot;
    state.runtime_overlay_set = retained.runtime_overlay_set;
    state.value_epoch = retained.value_epoch;
    state.publication_value_epoch = retained.publication_value_epoch;
    state.table_snapshots = retained.table_snapshots;
    state.deleted_table_facts = retained.deleted_table_facts;
    state.table_state_version = retained.table_state_version;
    state.publication_payload = retained.publication_payload;
    state.pending_invalidation_seeds.clear();
    state.pending_formula_edit_diagnostics.clear();
    state.pending_node_input_kind_transitions.clear();
    state.pending_dependency_shape_updates.clear();
    state.last_result = retained.last_result;
}

fn refresh_absent_snapshot_layer_shells(state: &mut OxCalcTreeWorkspaceState) {
    refresh_formula_and_dependency_absent_layer_shells(state);
    state.publication_snapshot = Arc::new(PublicationSnapshot::current_absent(
        state.workspace_revision.revision_id(),
        "w057.2-publication-not-yet-promoted",
    ));
    state.runtime_overlay_set = Arc::new(RuntimeOverlaySet::current_absent(
        state.publication_snapshot.snapshot_id(),
        "w057.2-runtime-overlays-not-yet-promoted",
    ));
}

fn refresh_formula_and_dependency_absent_layer_shells(state: &mut OxCalcTreeWorkspaceState) {
    state.formula_binding_snapshot = FormulaBindingSnapshot::current_absent(
        state.workspace_revision.revision_id(),
        "w057.2-formula-binding-not-yet-promoted",
    );
    state.dependency_shape_snapshot = DependencyShapeSnapshot::current_absent(
        state.workspace_revision.revision_id(),
        state.formula_binding_snapshot.snapshot_id(),
        "w057.2-dependency-shape-not-yet-promoted",
    );
}

fn normalize_context_table_snapshot(
    state: &OxCalcTreeWorkspaceState,
    node_id: TreeNodeId,
    snapshot: &TreeCalcTableNodeSnapshot,
) -> Result<TreeCalcTableNodeSnapshot, OxCalcTreeContextError> {
    normalize_context_table_snapshot_with_version(
        state,
        node_id,
        snapshot,
        state.table_state_version,
    )
}

fn normalize_context_table_snapshot_with_version(
    state: &OxCalcTreeWorkspaceState,
    node_id: TreeNodeId,
    snapshot: &TreeCalcTableNodeSnapshot,
    table_state_version: u64,
) -> Result<TreeCalcTableNodeSnapshot, OxCalcTreeContextError> {
    normalize_context_table_snapshot_with_snapshot_id(
        state,
        state.snapshot.snapshot_id(),
        node_id,
        snapshot,
        table_state_version,
    )
}

fn normalize_context_table_snapshot_with_snapshot_id(
    state: &OxCalcTreeWorkspaceState,
    structural_snapshot_id: StructuralSnapshotId,
    node_id: TreeNodeId,
    snapshot: &TreeCalcTableNodeSnapshot,
    table_state_version: u64,
) -> Result<TreeCalcTableNodeSnapshot, OxCalcTreeContextError> {
    let canonical_path = state.snapshot.get_projection_path(node_id)?;
    let mut normalized = snapshot.clone();
    normalized.table_node_id = node_id;
    normalized.display_path = canonical_path.clone();
    normalized.canonical_path = canonical_path;
    normalized.table_namespace_version = context_table_namespace_version_for_snapshot_id(
        state,
        node_id,
        structural_snapshot_id,
        table_state_version,
    );
    if normalized.row_membership_version.is_empty() {
        normalized.row_membership_version = format!(
            "treecalc-table-row-membership:v1:{}:{}",
            state.workspace_id.as_str(),
            node_id.0
        );
    }
    if normalized.row_order_version.is_empty() {
        normalized.row_order_version = format!(
            "treecalc-table-row-order:v1:{}:{}",
            state.workspace_id.as_str(),
            node_id.0
        );
    }
    if normalized.column_identity_version.is_empty() {
        normalized.column_identity_version = format!(
            "treecalc-table-columns:v1:{}:{}",
            state.workspace_id.as_str(),
            node_id.0
        );
    }
    Ok(normalized)
}

fn structural_table_shape_from_table_snapshot(
    snapshot: &TreeCalcTableNodeSnapshot,
) -> StructuralTableShape {
    let body_shape_identity = snapshot
        .columns
        .iter()
        .map(|column| {
            format!(
                "{}:{}:{}:{}",
                column.column_id,
                column.column_name,
                column.ordinal,
                column.body_metadata.identity_fragment()
            )
        })
        .collect::<Vec<_>>()
        .join("|");
    let body_cell_node_identity = snapshot
        .body_cell_nodes
        .iter()
        .map(|cell| format!("{}:{}:{}", cell.row_id.0, cell.column_id, cell.node_id))
        .collect::<Vec<_>>()
        .join("|");
    let body_shape_identity = format!("{body_shape_identity};cells={body_cell_node_identity}");
    let totals_shape_identity = snapshot
        .columns
        .iter()
        .map(|column| {
            format!(
                "{}:{}:{}",
                column.column_id,
                column.ordinal,
                column.totals_metadata.as_ref().map_or_else(
                    || "none".to_string(),
                    |metadata| metadata.identity_fragment()
                )
            )
        })
        .collect::<Vec<_>>()
        .join("|");
    StructuralTableShape {
        table_id: snapshot.table_id.clone(),
        table_name: snapshot.table_name.clone(),
        virtual_anchor_identity: format!(
            "{}:{}:{}:{}",
            snapshot.virtual_anchor.workbook_scope_ref,
            snapshot.virtual_anchor.sheet_scope_ref,
            snapshot.virtual_anchor.start_row,
            snapshot.virtual_anchor.start_col
        ),
        row_membership_version: snapshot.row_membership_version.clone(),
        row_order_version: snapshot.row_order_version.clone(),
        column_identity_version: snapshot.column_identity_version.clone(),
        body_shape_identity,
        totals_shape_identity,
        header_row_present: snapshot.header_row_present,
        totals_row_present: snapshot.totals_row_present,
        row_count: snapshot.rows.len(),
        column_count: snapshot.columns.len(),
    }
}

fn deleted_table_fact_from_snapshot(
    state: &OxCalcTreeWorkspaceState,
    snapshot: &TreeCalcTableNodeSnapshot,
) -> TreeCalcTableDeletedFact {
    TreeCalcTableDeletedFact {
        workspace_handle: state.workspace_id.as_str().to_string(),
        table_id: snapshot.table_id.clone(),
        selector_token_text: snapshot.table_name.clone(),
        table_namespace_version: snapshot.table_namespace_version.clone(),
    }
}

fn context_structure_version(state: &OxCalcTreeWorkspaceState) -> String {
    format!(
        "treecalc-structure:v1:{}:snapshot={}:tables={}",
        state.workspace_id.as_str(),
        state.snapshot.snapshot_id().0,
        state.table_state_version
    )
}

fn context_table_namespace_version_for_snapshot_id(
    state: &OxCalcTreeWorkspaceState,
    node_id: TreeNodeId,
    structural_snapshot_id: StructuralSnapshotId,
    table_state_version: u64,
) -> String {
    format!(
        "treecalc-table-namespace:v1:{}:{}:snapshot={}:tables={}",
        state.workspace_id.as_str(),
        node_id.0,
        structural_snapshot_id.0,
        table_state_version
    )
}

fn remove_deleted_publication_and_runtime_facts(
    state: &mut OxCalcTreeWorkspaceState,
    removed_node_ids: &[TreeNodeId],
) {
    let removed_node_ids = removed_node_ids.iter().copied().collect::<BTreeSet<_>>();
    let publication_payload = state.publication_payload_mut();
    for removed_node_id in &removed_node_ids {
        publication_payload.values_by_node.remove(removed_node_id);
    }
    publication_payload
        .runtime_effects
        .retain(|effect| !runtime_effect_mentions_any_node(effect, &removed_node_ids));

    // Direct-context structural delete does not yet classify compatible retained publication.
    // Drop the remaining baseline after the node-scoped facts have been explicitly removed.
    publication_payload.clear();
}

fn runtime_effect_mentions_any_node(
    effect: &RuntimeEffect,
    node_ids: &BTreeSet<TreeNodeId>,
) -> bool {
    node_ids
        .iter()
        .any(|node_id| runtime_effect_mentions_node(effect, *node_id))
}

fn runtime_effect_mentions_node(effect: &RuntimeEffect, node_id: TreeNodeId) -> bool {
    let node_token = format!("node:{}", node_id.0);
    let raw_token = node_id.0.to_string();
    effect.detail.split([';', '|']).any(|segment| {
        segment
            .strip_prefix("owner_node:")
            .or_else(|| segment.strip_prefix("target_node:"))
            .is_some_and(|value| value == node_token || value == raw_token)
    })
}

fn constant_value_for_formula_text(formula_text: &str) -> Option<String> {
    (!is_formula_text(formula_text) && !formula_text.is_empty()).then(|| formula_text.to_string())
}

fn interpret_authored_input_text(input_text: &str) -> RuntimeAuthoredInputResult {
    RuntimeEnvironment::new()
        .with_reference_bind_profile(treecalc_reference_bind_profile())
        .interpret_authored_input(FormulaSourceRecord::new(
            "treecalc-context:authored-input",
            1,
            input_text,
        ))
}

// Only the test fixtures still reconstruct a CalcValue from authored text;
// the snapshot import path now uses faithful `SnapshotCalcValue` values.
#[cfg(test)]
fn authored_input_text_to_calc_value(input_text: &str) -> CalcValue {
    match interpret_authored_input_text(input_text) {
        RuntimeAuthoredInputResult::Literal(value) => value,
        RuntimeAuthoredInputResult::Formula(_) | RuntimeAuthoredInputResult::Diagnostics(_) => {
            CalcValue::error(oxfunc_core::value::WorksheetErrorCode::Value)
        }
    }
}

fn authored_input_diagnostics_to_strings(
    diagnostics: oxfml_core::consumer::runtime::RuntimeAuthoredInputDiagnostics,
) -> Vec<String> {
    diagnostics
        .syntax_diagnostics
        .into_iter()
        .map(|diagnostic| format!("syntax:{diagnostic:?}"))
        .chain(
            diagnostics
                .bind_diagnostics
                .into_iter()
                .map(|diagnostic| format!("bind:{diagnostic:?}")),
        )
        .collect()
}

fn is_formula_text(formula_text: &str) -> bool {
    formula_text.trim_start().starts_with('=')
}

fn bump_input_value_epoch(state: &mut OxCalcTreeWorkspaceState) -> u64 {
    state.value_epoch = state.value_epoch.saturating_add(1);
    state.value_epoch
}

fn published_value_epochs_after(
    previous_values: &BTreeMap<TreeNodeId, CalcValue>,
    previous_epochs: &BTreeMap<TreeNodeId, u64>,
    published_values: &BTreeMap<TreeNodeId, CalcValue>,
    mut publication_value_epoch: u64,
) -> (u64, BTreeMap<TreeNodeId, u64>) {
    let mut epochs = BTreeMap::new();
    for (node_id, value) in published_values {
        let epoch = if previous_values
            .get(node_id)
            .is_some_and(|previous| previous == value)
        {
            previous_epochs.get(node_id).copied().unwrap_or_else(|| {
                publication_value_epoch = publication_value_epoch.saturating_add(1);
                publication_value_epoch
            })
        } else {
            publication_value_epoch = publication_value_epoch.saturating_add(1);
            publication_value_epoch
        };
        epochs.insert(*node_id, epoch);
    }
    (publication_value_epoch, epochs)
}

fn current_literal_value_for_node(
    state: &OxCalcTreeWorkspaceState,
    node_id: TreeNodeId,
) -> Option<CalcValue> {
    literal_value_from_node_input_snapshot(&state.workspace_revision.node_input_snapshot, node_id)
        .map(|value| match interpret_authored_input_text(&value) {
            RuntimeAuthoredInputResult::Literal(value) => value,
            RuntimeAuthoredInputResult::Formula(_) | RuntimeAuthoredInputResult::Diagnostics(_) => {
                CalcValue::error(oxfunc_core::value::WorksheetErrorCode::Value)
            }
        })
        .or_else(|| {
            state
                .last_result
                .as_ref()
                .and_then(|result| result.published_calc_values.get(&node_id).cloned())
        })
        .or_else(|| {
            state
                .publication_payload
                .values_by_node
                .get(&node_id)
                .cloned()
        })
}

fn input_text_from_node_input_snapshot(
    node_input_snapshot: &NodeInputSnapshot,
    node_id: TreeNodeId,
) -> String {
    node_input_snapshot
        .try_get_record(node_id)
        .and_then(|record| record.text.clone())
        .unwrap_or_default()
}

fn literal_value_from_node_input_snapshot(
    node_input_snapshot: &NodeInputSnapshot,
    node_id: TreeNodeId,
) -> Option<String> {
    node_input_snapshot
        .try_get_record(node_id)
        .filter(|record| record.kind == NodeInputKind::Literal)
        .and_then(|record| record.text.clone())
}

fn literal_epoch_from_node_input_snapshot(
    node_input_snapshot: &NodeInputSnapshot,
    node_id: TreeNodeId,
) -> Option<u64> {
    node_input_snapshot
        .try_get_record(node_id)
        .filter(|record| record.kind == NodeInputKind::Literal)
        .map(|record| record.input_epoch)
}

fn push_pending_invalidation_seed(
    state: &mut OxCalcTreeWorkspaceState,
    node_id: TreeNodeId,
    reason: InvalidationReasonKind,
) {
    let seed = InvalidationSeed { node_id, reason };
    if !state.pending_invalidation_seeds.contains(&seed) {
        state.pending_invalidation_seeds.push(seed);
    }
}

struct ContextFormulaCatalogBuild {
    catalog: TreeFormulaCatalog,
    dependency_descriptors: Vec<DependencyDescriptor>,
    diagnostics: Vec<String>,
    /// Preparations backing `dependency_descriptors`; callers with mutable
    /// state store this back so later catalog builds and engine runs reuse
    /// them instead of re-preparing unchanged formulas.
    prepared_formula_retention: PreparedFormulaRetention,
}

fn build_context_formula_catalog(
    state: &OxCalcTreeWorkspaceState,
    options: &OxCalcTreeContextOptions,
) -> Result<ContextFormulaCatalogBuild, OxCalcTreeContextError> {
    let mut bindings = Vec::new();
    let mut diagnostics = Vec::new();

    // One index for the whole catalog: binding resolves names for every
    // formula, and the per-call scan path is O(nodes) per name token. Meta
    // membership is derived from the structural snapshot's `is_meta` facts.
    let meta_node_ids = state.snapshot.meta_node_ids();
    let name_resolution_index = TreeNameResolutionIndex::build(&state.snapshot, &meta_node_ids);
    for record in state
        .workspace_revision
        .node_input_snapshot
        .records()
        .values()
    {
        if record.kind != NodeInputKind::FormulaText {
            continue;
        }
        let owner_node_id = record.node_id;
        let formula_text = record.text.as_deref().unwrap_or_default();
        let version = record.input_epoch;
        let mut host_packet_build = context_formula_from_oxfml_host_reference_packets(
            state,
            owner_node_id,
            formula_text,
            Some(&name_resolution_index),
        );
        diagnostics.extend(
            host_packet_build.diagnostics.drain(..).map(|diagnostic| {
                format!("treecalc_context_host_reference_resolution:{diagnostic}")
            }),
        );
        let expression = host_packet_build.expression;
        if let Some(diagnostic) =
            strict_excel_unsupported_profile_diagnostic(state, owner_node_id, formula_text)
        {
            diagnostics.push(diagnostic);
        }
        bindings.push(TreeFormulaBinding {
            owner_node_id,
            formula_artifact_id: FormulaArtifactId(format!(
                "formula:{}:{}:v{}",
                state.workspace_id.as_str(),
                owner_node_id.0,
                version
            )),
            bind_artifact_id: Some(BindArtifactId(format!(
                "bind:{}:{}:v{}",
                state.workspace_id.as_str(),
                owner_node_id.0,
                version
            ))),
            expression,
        });
    }

    let catalog = TreeFormulaCatalog::new(bindings);
    let environment_context = runtime_context_for_workspace_state(options, state);
    let prepared_formula_retention = PreparedFormulaRetention::prepare_catalog(
        Some(&state.prepared_formula_retention),
        &state.snapshot,
        &state.table_snapshots,
        &catalog,
        &environment_context,
    )?;
    let dependency_descriptors = prepared_formula_retention.dependency_descriptors();
    diagnostics.extend(
        prepared_formula_retention
            .bind_diagnostics()
            .into_iter()
            .map(|(owner_node_id, diagnostic)| {
                format!("oxfml_bind_diagnostic:owner={owner_node_id}:{diagnostic}")
            }),
    );

    Ok(ContextFormulaCatalogBuild {
        catalog,
        dependency_descriptors,
        diagnostics,
        prepared_formula_retention,
    })
}

fn strict_excel_unsupported_profile_diagnostic(
    state: &OxCalcTreeWorkspaceState,
    owner_node_id: TreeNodeId,
    formula_text: &str,
) -> Option<String> {
    let profile = state
        .workspace_revision
        .namespace_snapshot
        .capability_profile_id
        .as_str();
    if profile != "host-capabilities:strict-excel" {
        return None;
    }
    let compact_upper = formula_text
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect::<String>()
        .to_ascii_uppercase();
    let exclusion = if compact_upper.contains("INDIRECT(") {
        Some("INDIRECT")
    } else if strict_excel_treecalc_only_syntax(&compact_upper) {
        Some("TREECALC_HOST_REFERENCE_SYNTAX")
    } else {
        None
    };
    let exclusion = exclusion?;

    Some(format!(
        "typed_exclusion:strict_excel_profile_not_supported:{exclusion}:owner={owner_node_id}:profile={profile}"
    ))
}

fn strict_excel_treecalc_only_syntax(compact_upper_formula: &str) -> bool {
    compact_upper_formula.contains('@')
        || compact_upper_formula.contains("^.")
        || compact_upper_formula.contains("[]")
        || compact_upper_formula.contains(".*")
        || compact_upper_formula.contains(".**")
        || compact_upper_formula.contains('[')
        || reference_array_literal_contains_identifier(compact_upper_formula)
}

fn reference_array_literal_contains_identifier(compact_upper_formula: &str) -> bool {
    let Some(start) = compact_upper_formula.find('{') else {
        return false;
    };
    let Some(end_offset) = compact_upper_formula[start + 1..].find('}') else {
        return false;
    };
    let contents = &compact_upper_formula[start + 1..start + 1 + end_offset];
    contents
        .chars()
        .any(|ch| ch.is_ascii_alphabetic() || ch == '_' || ch == '.')
}

fn apply_context_profile_rejection(
    result: &mut OxCalcTreeCalculationOutcome,
    candidate_result_id: String,
) {
    let Some(diagnostic) = result
        .diagnostics
        .iter()
        .find(|diagnostic| {
            diagnostic.contains("typed_exclusion:strict_excel_profile_not_supported:INDIRECT")
        })
        .cloned()
    else {
        return;
    };

    result.run_state = OxCalcTreeRunState::Rejected;
    result.candidate_result = None;
    result.publication_bundle = None;
    result.reject_detail = Some(RejectDetail {
        candidate_result_id,
        kind: RejectKind::HostInjectedFailure,
        detail: diagnostic,
    });
}

fn formula_binding_snapshot_basis(catalog: &TreeFormulaCatalog) -> String {
    #[derive(Serialize)]
    struct FormulaBindingSnapshotBasisEntry<'a> {
        owner_node_id: u64,
        binding: &'a TreeFormulaBinding,
    }

    let entries = catalog
        .bindings_by_owner()
        .iter()
        .map(
            |(owner_node_id, binding)| FormulaBindingSnapshotBasisEntry {
                owner_node_id: owner_node_id.0,
                binding,
            },
        )
        .collect::<Vec<_>>();
    let entries_json =
        serde_json::to_string(&entries).expect("formula binding basis has JSON serialization");
    format!("formula-binding-catalog:v1:{entries_json}")
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ContextFormulaEditClassification {
    label: &'static str,
    affected_node_ids: Vec<TreeNodeId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ContextNodeInputKindTransition {
    node_id: TreeNodeId,
    predecessor_kind: NodeInputKind,
    successor_kind: NodeInputKind,
}

impl ContextNodeInputKindTransition {
    fn diagnostic(&self) -> String {
        format!(
            "formula_input_kind_transition:{}:{}->{}",
            self.node_id,
            self.predecessor_kind.as_identity_token(),
            self.successor_kind.as_identity_token()
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ContextFormulaEditTransition {
    predecessor_formula_present: bool,
    successor_formula_present: bool,
    predecessor_unresolved: bool,
    successor_unresolved: bool,
}

fn classify_context_formula_edit(
    snapshot: &StructuralSnapshot,
    owner_node_id: TreeNodeId,
    predecessor_descriptors: &[DependencyDescriptor],
    successor_descriptors: &[DependencyDescriptor],
    transition: ContextFormulaEditTransition,
) -> ContextFormulaEditClassification {
    let predecessor_facts =
        context_dependency_shape_facts(snapshot, owner_node_id, predecessor_descriptors);
    let successor_facts =
        context_dependency_shape_facts(snapshot, owner_node_id, successor_descriptors);
    let affected_node_ids = formula_dependency_shape_affected_node_ids(
        owner_node_id,
        &predecessor_facts.descriptors,
        &successor_facts.descriptors,
    );

    if successor_facts
        .graph
        .cycle_groups
        .iter()
        .any(|group| group.contains(&owner_node_id))
    {
        return ContextFormulaEditClassification {
            label: "cycle_candidate",
            affected_node_ids,
        };
    }

    if !transition.predecessor_formula_present && transition.successor_formula_present {
        return ContextFormulaEditClassification {
            label: if transition.successor_unresolved {
                "literal_to_unresolved_formula"
            } else {
                "literal_to_formula"
            },
            affected_node_ids,
        };
    }

    if transition.predecessor_formula_present && !transition.successor_formula_present {
        return ContextFormulaEditClassification {
            label: if transition.predecessor_unresolved {
                "unresolved_formula_to_literal"
            } else {
                "formula_to_literal"
            },
            affected_node_ids,
        };
    }

    let predecessor_unresolved =
        transition.predecessor_unresolved || predecessor_facts.has_unresolved;
    let successor_unresolved = transition.successor_unresolved || successor_facts.has_unresolved;
    if predecessor_unresolved && !successor_unresolved {
        return ContextFormulaEditClassification {
            label: "unresolved_to_resolved",
            affected_node_ids,
        };
    }
    if !predecessor_unresolved && successor_unresolved {
        return ContextFormulaEditClassification {
            label: "resolved_to_unresolved",
            affected_node_ids,
        };
    }

    let dynamic_changed = (predecessor_facts.has_dynamic_potential
        || successor_facts.has_dynamic_potential)
        && predecessor_facts.signatures != successor_facts.signatures;
    if dynamic_changed {
        return ContextFormulaEditClassification {
            label: "dynamic_dependency_changed",
            affected_node_ids,
        };
    }

    if predecessor_facts.signatures == successor_facts.signatures {
        return ContextFormulaEditClassification {
            label: "same_dependencies",
            affected_node_ids,
        };
    }

    ContextFormulaEditClassification {
        label: "dependency_shape_changed",
        affected_node_ids,
    }
}

fn dependency_shape_update_for_formula_edit(
    classification: &ContextFormulaEditClassification,
) -> Option<DependencyShapeUpdate> {
    let kind = match classification.label {
        "dependency_shape_changed" => "static_dependency_shape_changed",
        "unresolved_to_resolved" => "static_dependency_resolved",
        "resolved_to_unresolved" => "static_dependency_unresolved",
        "dynamic_dependency_changed" => "static_dynamic_dependency_shape_changed",
        "literal_to_formula" => "static_formula_dependency_activated",
        "literal_to_unresolved_formula" => "static_dependency_unresolved",
        "formula_to_literal" => "static_formula_dependency_released",
        "unresolved_formula_to_literal" => "static_formula_dependency_released_unresolved",
        _ => return None,
    };
    Some(DependencyShapeUpdate {
        kind: kind.to_string(),
        affected_node_ids: classification.affected_node_ids.clone(),
    })
}

fn context_formula_catalog_has_unresolved(
    owner_node_id: TreeNodeId,
    diagnostics: &[String],
) -> bool {
    let owner_token = format!("owner={owner_node_id}");
    diagnostics.iter().any(|diagnostic| {
        diagnostic.contains(&owner_token)
            && (diagnostic.contains("unresolved_host_name")
                || diagnostic.contains("UnresolvedReference")
                || diagnostic.contains("unresolved_reference")
                || diagnostic.contains("unresolved identifier")
                || diagnostic.contains("did not bind"))
    })
}

fn preview_mutation_seeds(
    state: &OxCalcTreeWorkspaceState,
    mutation: &OxCalcTreePreviewMutation,
    dependency_descriptors: &[DependencyDescriptor],
) -> Result<Vec<InvalidationSeed>, OxCalcTreeContextError> {
    match mutation {
        OxCalcTreePreviewMutation::InvalidateNode { node_id, reason } => {
            ensure_preview_node_exists(state, *node_id)?;
            Ok(vec![InvalidationSeed {
                node_id: *node_id,
                reason: *reason,
            }])
        }
        OxCalcTreePreviewMutation::SetNodeInput { node_id } => {
            ensure_preview_node_exists(state, *node_id)?;
            Ok(vec![InvalidationSeed {
                node_id: *node_id,
                reason: InvalidationReasonKind::UpstreamPublication,
            }])
        }
        OxCalcTreePreviewMutation::SetNodeFormulaText {
            node_id,
            formula_text,
        } => {
            ensure_preview_node_exists(state, *node_id)?;
            let predecessor_kind = state
                .workspace_revision
                .node_input_snapshot
                .try_get_record(*node_id)
                .map(|record| record.kind)
                .ok_or_else(|| OxCalcTreeContextError::InvalidWorkspaceSnapshot {
                    detail: format!("node input snapshot is missing record for {node_id}"),
                })?;
            let successor_kind = node_input_kind_for_formula_edit_text(formula_text);
            let reason = if predecessor_kind == NodeInputKind::FormulaText
                || successor_kind == NodeInputKind::FormulaText
            {
                InvalidationReasonKind::StructuralRebindRequired
            } else {
                InvalidationReasonKind::UpstreamPublication
            };
            Ok(vec![InvalidationSeed {
                node_id: *node_id,
                reason,
            }])
        }
        OxCalcTreePreviewMutation::SetNodeTable {
            node_id,
            snapshot,
            scenario,
        } => {
            ensure_preview_node_exists(state, *node_id)?;
            let before_snapshot = state.table_snapshots.get(node_id).ok_or_else(|| {
                OxCalcTreeContextError::InvalidWorkspaceSnapshot {
                    detail: format!("node {node_id:?} has no table snapshot"),
                }
            })?;
            let before_projection = project_treecalc_table_node_snapshot(before_snapshot)
                .map_err(|error| OxCalcTreeContextError::TableProjection { error })?;
            let after_projection = project_treecalc_table_node_snapshot(snapshot)
                .map_err(|error| OxCalcTreeContextError::TableProjection { error })?;
            let impact = classify_treecalc_table_update(
                *scenario,
                Some(&before_projection),
                Some(&after_projection),
                [*node_id],
                Vec::<String>::new(),
            );
            let mut seeds = impact.invalidation_seeds;
            seeds.extend(table_preview_dependent_seeds(
                dependency_descriptors,
                &before_projection,
                &impact.changed_dependency_kinds,
            ));
            Ok(seeds)
        }
        OxCalcTreePreviewMutation::RenameNode { node_id }
        | OxCalcTreePreviewMutation::MoveNode { node_id }
        | OxCalcTreePreviewMutation::ReorderNode { node_id }
        | OxCalcTreePreviewMutation::DeleteNode { node_id } => {
            ensure_preview_node_exists(state, *node_id)?;
            Ok(vec![InvalidationSeed {
                node_id: *node_id,
                reason: InvalidationReasonKind::StructuralRebindRequired,
            }])
        }
    }
}

fn ensure_preview_node_exists(
    state: &OxCalcTreeWorkspaceState,
    node_id: TreeNodeId,
) -> Result<(), OxCalcTreeContextError> {
    state
        .snapshot
        .try_get_node(node_id)
        .map(|_| ())
        .ok_or_else(|| OxCalcTreeContextError::Structural(StructuralError::UnknownNode { node_id }))
}

fn reference_collection_dependency_for_handle<'a>(
    graph: &'a DependencyGraph,
    owner_node_id: TreeNodeId,
    source_reference_handle: &str,
) -> Option<&'a TreeReferenceCollectionDependency> {
    graph
        .descriptors_by_owner
        .get(&owner_node_id)?
        .iter()
        .filter(|descriptor| {
            descriptor.kind == DependencyDescriptorKind::TreeReferenceCollectionMembership
        })
        .filter_map(|descriptor| descriptor.tree_reference_collection.as_ref())
        .find(|collection| collection.host_ref_handle == source_reference_handle)
}

fn table_preview_dependent_seeds(
    dependency_descriptors: &[DependencyDescriptor],
    projection: &TreeCalcTableNodeProjection,
    changed_dependency_kinds: &BTreeSet<DependencyDescriptorKind>,
) -> Vec<InvalidationSeed> {
    dependency_descriptors
        .iter()
        .filter(|descriptor| changed_dependency_kinds.contains(&descriptor.kind))
        .filter(|descriptor| descriptor_matches_table_projection(descriptor, projection))
        .flat_map(|descriptor| {
            descriptor_invalidation_facts(descriptor)
                .into_iter()
                .map(move |reason| InvalidationSeed {
                    node_id: descriptor.owner_node_id,
                    reason,
                })
        })
        .collect()
}

fn descriptor_matches_table_projection(
    descriptor: &DependencyDescriptor,
    projection: &TreeCalcTableNodeProjection,
) -> bool {
    if descriptor.target_node_id == Some(projection.table_node_id) {
        return true;
    }

    let detail = descriptor.carrier_detail.as_str();
    table_projection_identity_fragments(projection)
        .iter()
        .any(|fragment| !fragment.is_empty() && detail.contains(fragment))
}

fn table_projection_identity_fragments(projection: &TreeCalcTableNodeProjection) -> Vec<&str> {
    vec![
        projection.table_id.as_str(),
        projection.table_context_identity.as_str(),
        projection.table_invalidation_identity.as_str(),
        projection.table_namespace_identity.as_str(),
        projection.table_namespace_version.as_str(),
        projection.table_namespace_token.as_str(),
        projection.row_membership_identity.as_str(),
        projection.row_order_identity.as_str(),
        projection.oxcalc_row_membership_identity.as_str(),
        projection.oxcalc_row_order_identity.as_str(),
        projection.column_identity.as_str(),
        projection.oxcalc_column_identity.as_str(),
        projection.virtual_anchor_identity.as_str(),
        projection.virtual_anchor_token.as_str(),
        projection.body_metadata_identity.as_str(),
        projection.body_metadata_token.as_str(),
        projection.totals_metadata_identity.as_str(),
        projection.totals_metadata_token.as_str(),
    ]
}

fn dedupe_preview_invalidation_seeds(seeds: Vec<InvalidationSeed>) -> Vec<InvalidationSeed> {
    seeds
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ContextDependencyShapeFacts {
    descriptors: Vec<DependencyDescriptor>,
    graph: DependencyGraph,
    signatures: BTreeSet<ContextDependencyShapeSignature>,
    has_unresolved: bool,
    has_dynamic_potential: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ContextDependencyShapeSignature {
    kind: DependencyDescriptorKind,
    target_node_id: Option<TreeNodeId>,
    workspace_target: Option<WorkspaceQualifiedTarget>,
    carrier_shape_detail: String,
    tree_reference_collection: Option<ContextTreeReferenceCollectionSignature>,
    requires_rebind_on_structural_change: bool,
}

impl From<&DependencyDescriptor> for ContextDependencyShapeSignature {
    fn from(descriptor: &DependencyDescriptor) -> Self {
        Self {
            kind: descriptor.kind,
            target_node_id: descriptor.target_node_id,
            workspace_target: descriptor.workspace_target.clone(),
            carrier_shape_detail: context_dependency_shape_carrier_detail(descriptor),
            tree_reference_collection: descriptor
                .tree_reference_collection
                .as_ref()
                .map(Into::into),
            requires_rebind_on_structural_change: descriptor.requires_rebind_on_structural_change,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ContextTreeReferenceCollectionSignature {
    family: TreeReferenceCollectionFamily,
    base_node_id: TreeNodeId,
    membership_version: String,
    order_version: String,
    member_node_ids: Vec<TreeNodeId>,
}

impl From<&TreeReferenceCollectionDependency> for ContextTreeReferenceCollectionSignature {
    fn from(collection: &TreeReferenceCollectionDependency) -> Self {
        Self {
            family: collection.family,
            base_node_id: collection.base_node_id,
            membership_version: collection.membership_version.clone(),
            order_version: collection.order_version.clone(),
            member_node_ids: collection.member_node_ids.clone(),
        }
    }
}

fn context_dependency_shape_carrier_detail(descriptor: &DependencyDescriptor) -> String {
    // Source handles and source-position-derived host handles belong to formula
    // artifact identity. Dependency-shape classification compares the typed
    // dependency consequence, not the textual carrier's current span.
    match descriptor.kind {
        DependencyDescriptorKind::TreeReferenceCollectionMembership => {
            "tree_reference_collection_membership".to_string()
        }
        DependencyDescriptorKind::TreeReferenceCollectionMemberValue => {
            "tree_reference_collection_member_value".to_string()
        }
        _ => descriptor.carrier_detail.clone(),
    }
}

fn context_dependency_shape_facts(
    snapshot: &StructuralSnapshot,
    owner_node_id: TreeNodeId,
    descriptors: &[DependencyDescriptor],
) -> ContextDependencyShapeFacts {
    let owner_descriptors = descriptors
        .iter()
        .filter(|descriptor| descriptor.owner_node_id == owner_node_id)
        .collect::<Vec<_>>();
    let signatures = owner_descriptors
        .iter()
        .copied()
        .map(Into::into)
        .collect::<BTreeSet<_>>();
    let has_unresolved = owner_descriptors
        .iter()
        .any(|descriptor| descriptor.kind == DependencyDescriptorKind::Unresolved);
    let has_dynamic_potential = owner_descriptors
        .iter()
        .any(|descriptor| descriptor.kind == DependencyDescriptorKind::DynamicPotential);
    let graph = DependencyGraph::build(snapshot, descriptors);
    ContextDependencyShapeFacts {
        descriptors: descriptors.to_vec(),
        graph,
        signatures,
        has_unresolved,
        has_dynamic_potential,
    }
}

fn formula_dependency_shape_affected_node_ids(
    owner_node_id: TreeNodeId,
    predecessor_descriptors: &[DependencyDescriptor],
    successor_descriptors: &[DependencyDescriptor],
) -> Vec<TreeNodeId> {
    let mut target_node_ids = predecessor_descriptors
        .iter()
        .chain(successor_descriptors.iter())
        .filter(|descriptor| descriptor.owner_node_id == owner_node_id)
        .filter_map(|descriptor| descriptor.target_node_id)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .filter(|node_id| *node_id != owner_node_id)
        .collect::<Vec<_>>();
    let mut affected_node_ids = vec![owner_node_id];
    affected_node_ids.append(&mut target_node_ids);
    affected_node_ids
}

struct ContextHostReferencePacketBuild {
    expression: TreeFormula,
    diagnostics: Vec<String>,
}

fn context_dry_bind_formula_text(
    state: &OxCalcTreeWorkspaceState,
    owner_node_id: TreeNodeId,
    formula_text: &str,
) -> Result<OxCalcTreeDryBindVerdict, OxCalcTreeContextError> {
    let source = FormulaSourceRecord::new(
        format!(
            "treecalc-context-dry-bind:{}:{}",
            state.workspace_id.as_str(),
            owner_node_id.0
        ),
        owner_node_id.0,
        formula_text,
    );
    let mut table_context_packet = None;
    if !state.table_snapshots.is_empty() {
        table_context_packet = Some(context_formula_table_context_packet(state)?);
    }
    let host_context = context_formula_host_context(
        state,
        owner_node_id,
        table_context_packet
            .as_ref()
            .map(|packet| packet.table_context_identity.clone()),
    );
    let meta_node_ids = state.snapshot.meta_node_ids();
    let reference_bind_profile = TreeCalcContextReferenceBindProfile::new(
        state.snapshot.as_ref(),
        &meta_node_ids,
        owner_node_id,
    );
    let mut runtime_environment = RuntimeEnvironment::new()
        .with_host_formula_context(host_context)
        .with_reference_bind_profile(&reference_bind_profile);
    if let Some(packet) = &table_context_packet {
        runtime_environment = runtime_environment.with_table_context(
            packet.table_catalog.clone(),
            packet.enclosing_table_ref.clone(),
            packet.caller_table_region.clone(),
        );
    }

    Ok(oxcalc_dry_bind_verdict_from_oxfml(
        owner_node_id,
        runtime_environment.dry_bind_authored_input(source),
    ))
}

fn oxcalc_dry_bind_verdict_from_oxfml(
    node_id: TreeNodeId,
    verdict: RuntimeDryBindVerdict,
) -> OxCalcTreeDryBindVerdict {
    let mut diagnostics = verdict
        .syntax_diagnostics
        .into_iter()
        .map(|diagnostic| OxCalcTreeDryBindDiagnostic {
            stage: OxCalcTreeDryBindDiagnosticStage::Syntax,
            message: diagnostic.message,
            span_start_utf8: diagnostic.span.start,
            span_len_utf8: diagnostic.span.len,
        })
        .collect::<Vec<_>>();
    diagnostics.extend(verdict.bind_diagnostics.into_iter().map(|diagnostic| {
        OxCalcTreeDryBindDiagnostic {
            stage: OxCalcTreeDryBindDiagnosticStage::Bind,
            message: diagnostic.message,
            span_start_utf8: diagnostic.span.start,
            span_len_utf8: diagnostic.span.len,
        }
    }));
    let profile_violations = verdict
        .profile_violations
        .into_iter()
        .map(|violation| OxCalcTreeDryBindProfileViolation {
            kind: match violation.kind {
                RuntimeDryBindProfileViolationKind::FunctionUnavailable {
                    function_id,
                    function_name,
                    reason,
                } => OxCalcTreeDryBindProfileViolationKind::FunctionUnavailable {
                    function_id,
                    function_name,
                    reason,
                },
            },
            feature: violation.feature,
            message: violation.message,
            span_start_utf8: violation.span.start,
            span_len_utf8: violation.span.len,
        })
        .collect();
    OxCalcTreeDryBindVerdict {
        node_id,
        input_kind: match verdict.input_kind {
            RuntimeDryBindInputKind::Literal => OxCalcTreeDryBindInputKind::Literal,
            RuntimeDryBindInputKind::Formula => OxCalcTreeDryBindInputKind::Formula,
        },
        legal: verdict.legal,
        diagnostics,
        profile_violations,
    }
}

fn context_formula_from_oxfml_host_reference_packets(
    state: &OxCalcTreeWorkspaceState,
    owner_node_id: TreeNodeId,
    formula_text: &str,
    name_resolution_index: Option<&TreeNameResolutionIndex>,
) -> ContextHostReferencePacketBuild {
    let source = FormulaSourceRecord::new(
        format!(
            "treecalc-context-host-reference:{}:{}",
            state.workspace_id.as_str(),
            owner_node_id.0
        ),
        owner_node_id.0,
        formula_text,
    );
    let mut diagnostics = Vec::new();
    let table_context_packet = (!state.table_snapshots.is_empty())
        .then(|| {
            context_formula_table_context_packet(state)
                .inspect_err(|error| diagnostics.push(format!("table_context_packet:{error:?}")))
                .ok()
        })
        .flatten();
    let host_context = context_formula_host_context(
        state,
        owner_node_id,
        table_context_packet
            .as_ref()
            .map(|packet| packet.table_context_identity.clone()),
    );
    let _ = name_resolution_index;
    let meta_node_ids = state.snapshot.meta_node_ids();
    let reference_bind_profile = TreeCalcContextReferenceBindProfile::new(
        state.snapshot.as_ref(),
        &meta_node_ids,
        owner_node_id,
    );
    let mut runtime_environment = RuntimeEnvironment::new()
        .with_host_formula_context(host_context)
        .with_reference_bind_profile(&reference_bind_profile);
    if let Some(packet) = &table_context_packet {
        runtime_environment = runtime_environment.with_table_context(
            packet.table_catalog.clone(),
            packet.enclosing_table_ref.clone(),
            packet.caller_table_region.clone(),
        );
    }
    let bound_formula = match runtime_environment.interpret_authored_input(source.clone()) {
        RuntimeAuthoredInputResult::Formula(bound_formula) => Some(bound_formula),
        RuntimeAuthoredInputResult::Literal(_) => {
            diagnostics.push(format!(
                "oxfml_authored_input_unexpected_literal_after_host_reference_parse:{owner_node_id}"
            ));
            None
        }
        RuntimeAuthoredInputResult::Diagnostics(authored_diagnostics) => {
            diagnostics.extend(
                authored_input_diagnostics_to_strings(authored_diagnostics)
                    .into_iter()
                    .map(|diagnostic| format!("oxfml_authored_input:{diagnostic}")),
            );
            None
        }
    };
    ContextHostReferencePacketBuild {
        expression: TreeFormula::opaque_oxfml(
            source.entered_formula_text,
            Vec::<crate::formula::TreeReference>::new(),
        )
        .with_bound_formula(bound_formula.clone()),
        diagnostics,
    }
}

fn context_formula_table_context_packet(
    state: &OxCalcTreeWorkspaceState,
) -> Result<StructuredTableContextPacket, OxCalcTreeContextError> {
    let projections = state
        .table_snapshots
        .iter()
        .map(|(node_id, snapshot)| {
            let normalized = normalize_context_table_snapshot(state, *node_id, snapshot)?;
            project_treecalc_table_node_snapshot(&normalized)
                .map_err(|error| OxCalcTreeContextError::TableProjection { error })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(StructuredTableContextPacket::from_oxfml_table_packet(
        projections
            .into_iter()
            .map(|projection| projection.table_descriptor)
            .collect(),
        None,
        None,
    ))
}

fn context_formula_host_context(
    state: &OxCalcTreeWorkspaceState,
    owner_node_id: TreeNodeId,
    table_context_identity: Option<String>,
) -> RuntimeHostFormulaContext {
    let namespace_snapshot = &state.workspace_revision.namespace_snapshot;
    RuntimeHostFormulaContext {
        dialect_id: "oxcalc.treecalc-v1".to_string(),
        capability_profile_id: namespace_snapshot.capability_profile_id.clone(),
        resolution_rule_version: namespace_snapshot.resolution_rule_version.clone(),
        host_namespace_version: Some(namespace_snapshot.host_namespace_version.clone()),
        registry_snapshot_identity: Some(namespace_snapshot.function_registry_version.clone()),
        structure_context_version: Some(context_structure_version(state)),
        caller_context_identity: Some(format!(
            "treecalc-caller:{};{}",
            owner_node_id, namespace_snapshot.caller_context_identity_version
        )),
        table_context_identity,
    }
}

fn node_view_from_state(
    state: &OxCalcTreeWorkspaceState,
    node: &StructuralNode,
    table: Option<OxCalcTreeTableView>,
) -> Result<OxCalcTreeNodeView, OxCalcTreeContextError> {
    let canonical_path = state.snapshot.get_projection_path(node.node_id)?;
    Ok(OxCalcTreeNodeView {
        node_id: node.node_id,
        symbol: node.symbol.clone(),
        parent_node_id: node.parent_id,
        child_node_ids: node.child_ids.clone(),
        display_path: canonical_path.clone(),
        canonical_path,
        formula_text: input_text_from_node_input_snapshot(
            &state.workspace_revision.node_input_snapshot,
            node.node_id,
        ),
        value_text: state
            .last_result
            .as_ref()
            .and_then(|result| result.published_values.get(&node.node_id))
            .cloned()
            .or_else(|| {
                state
                    .publication_payload
                    .values_by_node
                    .get(&node.node_id)
                    .map(calc_value_display_text)
            })
            .or_else(|| {
                literal_value_from_node_input_snapshot(
                    &state.workspace_revision.node_input_snapshot,
                    node.node_id,
                )
            }),
        calc_value: current_literal_value_for_node(state, node.node_id),
        input_value_epoch: literal_epoch_from_node_input_snapshot(
            &state.workspace_revision.node_input_snapshot,
            node.node_id,
        ),
        published_value_epoch: state
            .publication_payload
            .value_epochs_by_node
            .get(&node.node_id)
            .copied(),
        calc_state: state
            .last_result
            .as_ref()
            .and_then(|result| result.node_states.get(&node.node_id))
            .copied(),
        is_meta: node.is_meta,
        table,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OxCalcTreeRuntimeLane {
    LocalSequentialTreeCalc,
}

impl OxCalcTreeRuntimeLane {
    #[must_use]
    pub fn as_diagnostic_value(&self) -> &'static str {
        match self {
            Self::LocalSequentialTreeCalc => "local_sequential_treecalc",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OxCalcTreeNamespaceOptions {
    pub host_namespace_version: String,
    pub function_registry_version: String,
    pub resolution_rule_version: String,
    pub caller_context_identity_version: String,
    pub cross_workspace_availability_version: Option<String>,
}

impl Default for OxCalcTreeNamespaceOptions {
    fn default() -> Self {
        Self {
            host_namespace_version: "treecalc-host-namespace:v1".to_string(),
            function_registry_version: "oxfunc.arg-prep:default".to_string(),
            resolution_rule_version: "treecalc-host-resolution:v1".to_string(),
            caller_context_identity_version: "treecalc-caller-context:v1".to_string(),
            cross_workspace_availability_version: None,
        }
    }
}

impl OxCalcTreeNamespaceOptions {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn with_host_namespace_version(mut self, version: impl Into<String>) -> Self {
        self.host_namespace_version = version.into();
        self
    }

    #[must_use]
    pub fn with_function_registry_version(mut self, version: impl Into<String>) -> Self {
        self.function_registry_version = version.into();
        self
    }

    #[must_use]
    pub fn with_resolution_rule_version(mut self, version: impl Into<String>) -> Self {
        self.resolution_rule_version = version.into();
        self
    }

    #[must_use]
    pub fn with_caller_context_identity_version(mut self, version: impl Into<String>) -> Self {
        self.caller_context_identity_version = version.into();
        self
    }

    #[must_use]
    pub fn with_cross_workspace_availability_version(mut self, version: impl Into<String>) -> Self {
        self.cross_workspace_availability_version = Some(version.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OxCalcTreeHostCapabilitySnapshot {
    pub capability_profile_id: String,
    pub dynamic_dependency_effects: bool,
    pub execution_restriction_effects: bool,
    pub capability_sensitive_effects: bool,
    pub shape_topology_effects: bool,
}

impl Default for OxCalcTreeHostCapabilitySnapshot {
    fn default() -> Self {
        Self {
            capability_profile_id: "host-capabilities:default".to_string(),
            dynamic_dependency_effects: true,
            execution_restriction_effects: true,
            capability_sensitive_effects: false,
            shape_topology_effects: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OxCalcTreeRuntimePolicy {
    pub policy_id: String,
    pub emit_environment_diagnostics: bool,
    pub project_runtime_effect_overlays: bool,
    pub derivation_trace_enabled: bool,
    pub scheduling_policy: LocalTreeCalcSchedulingPolicy,
}

impl Default for OxCalcTreeRuntimePolicy {
    fn default() -> Self {
        Self {
            policy_id: "runtime-policy:default".to_string(),
            emit_environment_diagnostics: true,
            project_runtime_effect_overlays: true,
            derivation_trace_enabled: false,
            scheduling_policy: LocalTreeCalcSchedulingPolicy::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OxCalcTreeContextOptions {
    pub runtime_lane: OxCalcTreeRuntimeLane,
    pub session_id: Option<String>,
    pub namespace: OxCalcTreeNamespaceOptions,
    pub host_capabilities: OxCalcTreeHostCapabilitySnapshot,
    pub runtime_policy: OxCalcTreeRuntimePolicy,
    pub revision_retention_policy: OxCalcTreeRevisionRetentionPolicy,
}

impl Default for OxCalcTreeContextOptions {
    fn default() -> Self {
        Self {
            runtime_lane: OxCalcTreeRuntimeLane::LocalSequentialTreeCalc,
            session_id: None,
            namespace: OxCalcTreeNamespaceOptions::default(),
            host_capabilities: OxCalcTreeHostCapabilitySnapshot::default(),
            runtime_policy: OxCalcTreeRuntimePolicy::default(),
            revision_retention_policy: OxCalcTreeRevisionRetentionPolicy::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OxCalcTreeRevisionRetentionPolicy {
    pub max_retained_revisions: usize,
}

impl Default for OxCalcTreeRevisionRetentionPolicy {
    fn default() -> Self {
        Self {
            max_retained_revisions: 256,
        }
    }
}

impl OxCalcTreeRevisionRetentionPolicy {
    #[must_use]
    pub const fn bounded(max_retained_revisions: usize) -> Self {
        Self {
            max_retained_revisions,
        }
    }
}

impl OxCalcTreeContextOptions {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn with_session_id(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }

    #[must_use]
    pub fn with_namespace(mut self, namespace: OxCalcTreeNamespaceOptions) -> Self {
        self.namespace = namespace;
        self
    }

    #[must_use]
    pub fn with_host_capabilities(
        mut self,
        host_capabilities: OxCalcTreeHostCapabilitySnapshot,
    ) -> Self {
        self.host_capabilities = host_capabilities;
        self
    }

    #[must_use]
    pub fn with_runtime_policy(mut self, runtime_policy: OxCalcTreeRuntimePolicy) -> Self {
        self.runtime_policy = runtime_policy;
        self
    }

    #[must_use]
    pub fn with_revision_retention_policy(
        mut self,
        revision_retention_policy: OxCalcTreeRevisionRetentionPolicy,
    ) -> Self {
        self.revision_retention_policy = revision_retention_policy;
        self
    }

    #[must_use]
    pub fn runtime_context(&self) -> LocalTreeCalcEnvironmentContext {
        LocalTreeCalcEnvironmentContext {
            runtime_lane: self.runtime_lane.as_diagnostic_value().to_string(),
            session_id: self.session_id.clone(),
            capability_profile_id: self.host_capabilities.capability_profile_id.clone(),
            host_namespace_version: self.namespace.host_namespace_version.clone(),
            resolution_rule_version: self.namespace.resolution_rule_version.clone(),
            caller_context_identity_version: self.namespace.caller_context_identity_version.clone(),
            table_context_identity: None,
            cross_workspace_availability_version: self
                .namespace
                .cross_workspace_availability_version
                .clone(),
            arg_preparation_profile_version: self.namespace.function_registry_version.clone(),
            meta_node_ids: BTreeSet::new(),
            oxfunc_bridge_metadata: Default::default(),
            dynamic_dependency_effects: self.host_capabilities.dynamic_dependency_effects,
            execution_restriction_effects: self.host_capabilities.execution_restriction_effects,
            capability_sensitive_effects: self.host_capabilities.capability_sensitive_effects,
            shape_topology_effects: self.host_capabilities.shape_topology_effects,
            runtime_policy_id: self.runtime_policy.policy_id.clone(),
            project_runtime_effect_overlays: self.runtime_policy.project_runtime_effect_overlays,
            derivation_trace_enabled: self.runtime_policy.derivation_trace_enabled,
            scheduling_policy: self.runtime_policy.scheduling_policy.clone(),
        }
    }

    #[must_use]
    pub fn diagnostics(&self) -> Vec<String> {
        if !self.runtime_policy.emit_environment_diagnostics {
            return Vec::new();
        }

        let session_id = self.session_id.as_deref().unwrap_or("none");
        vec![
            format!(
                "oxcalc_tree_context_options_runtime_lane:{}",
                self.runtime_lane.as_diagnostic_value()
            ),
            format!("oxcalc_tree_context_options_session_id:{session_id}"),
            format!(
                "oxcalc_tree_context_options_host_namespace_version:{}",
                self.namespace.host_namespace_version
            ),
            format!(
                "oxcalc_tree_context_options_function_registry_version:{}",
                self.namespace.function_registry_version
            ),
            format!(
                "oxcalc_tree_context_options_resolution_rule_version:{}",
                self.namespace.resolution_rule_version
            ),
            format!(
                "oxcalc_tree_context_options_caller_context_identity_version:{}",
                self.namespace.caller_context_identity_version
            ),
            format!(
                "oxcalc_tree_context_options_capability_profile_id:{}",
                self.host_capabilities.capability_profile_id
            ),
            format!(
                "oxcalc_tree_context_options_capability_dynamic_dependency:{}",
                self.host_capabilities.dynamic_dependency_effects
            ),
            format!(
                "oxcalc_tree_context_options_capability_execution_restriction:{}",
                self.host_capabilities.execution_restriction_effects
            ),
            format!(
                "oxcalc_tree_context_options_capability_sensitive:{}",
                self.host_capabilities.capability_sensitive_effects
            ),
            format!(
                "oxcalc_tree_context_options_capability_shape_topology:{}",
                self.host_capabilities.shape_topology_effects
            ),
            format!(
                "oxcalc_tree_context_options_runtime_policy_id:{}",
                self.runtime_policy.policy_id
            ),
            format!(
                "oxcalc_tree_context_options_project_runtime_effect_overlays:{}",
                self.runtime_policy.project_runtime_effect_overlays
            ),
            format!(
                "oxcalc_tree_context_options_derivation_trace_enabled:{}",
                self.runtime_policy.derivation_trace_enabled
            ),
            format!(
                "oxcalc_tree_context_options_scheduling_policy:{}",
                self.runtime_policy.scheduling_policy.diagnostic_name()
            ),
        ]
    }
}

impl From<LocalTreeCalcRunState> for OxCalcTreeRunState {
    fn from(value: LocalTreeCalcRunState) -> Self {
        match value {
            LocalTreeCalcRunState::Published => Self::Published,
            LocalTreeCalcRunState::VerifiedClean => Self::VerifiedClean,
            LocalTreeCalcRunState::Rejected => Self::Rejected,
        }
    }
}

impl From<LocalTreeCalcRunArtifacts> for OxCalcTreeCalculationOutcome {
    fn from(value: LocalTreeCalcRunArtifacts) -> Self {
        Self {
            run_state: value.result_state.into(),
            dependency_graph: value.dependency_graph,
            invalidation_closure: value.invalidation_closure,
            evaluation_order: value.evaluation_order,
            runtime_effects: value.runtime_effects,
            runtime_effect_overlays: value.runtime_effect_overlays,
            derivation_traces: value.derivation_traces,
            candidate_result: value.candidate_result,
            publication_bundle: value.publication_bundle,
            reject_detail: value.reject_detail,
            published_values: calc_value_display_map(&value.published_calc_values),
            published_calc_values: value.published_calc_values,
            published_value_epochs: BTreeMap::new(),
            node_states: value.node_states,
            phase_timings_micros: value.phase_timings_micros,
            binding_diagnostics: value.binding_diagnostics,
            diagnostics: value.diagnostics,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coordinator::{RejectKind, RuntimeEffectFamily};
    use crate::dependency::{DependencyDescriptorKind, InvalidationReasonKind};
    use crate::formula::{
        FixtureFormulaAst, FixtureFormulaBinaryOp, TreeFormula, TreeFormulaBinding, TreeReference,
        W056NonTableReferenceEvidenceStatus, w056_non_table_reference_category,
    };
    use crate::recalc::OverlayKind;
    use crate::structural::{
        BindArtifactId, FormulaArtifactId, StructuralNode, StructuralNodeKind, StructuralSnapshot,
        StructuralSnapshotId,
    };
    use crate::structured_table::{
        StructuredTableDependencyFactKind, StructuredTableRegionSelection,
        TreeCalcDynamicTableRebindCause, TreeCalcDynamicTableRebindStatus,
        TreeCalcDynamicTableReferenceTargetKind, TreeCalcTableBodyCellNodeBinding,
        TreeCalcTableColumnBodyMetadata, TreeCalcTableColumnFormulaRuntimeRequest,
        TreeCalcTableColumnSnapshot, TreeCalcTableFormulaMetadata,
        TreeCalcTableFormulaRuntimeContext, TreeCalcTableRowId, TreeCalcTableSparseValue,
        TreeCalcTableTotalsCellNodeBinding, TreeCalcTableVirtualAnchor,
        evaluate_treecalc_table_column_formula_rows, evaluate_treecalc_table_totals_formula,
    };
    use crate::workspace_revision::{NodeInputKind, SnapshotLayerState};
    use oxfunc_core::value::{CoreValue, RichValue, WorksheetErrorCode};

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
                    is_meta: false,
                },
                StructuralNode {
                    node_id: TreeNodeId(2),
                    kind: StructuralNodeKind::Constant,
                    symbol: "A".to_string(),
                    parent_id: Some(TreeNodeId(1)),
                    child_ids: vec![],
                    role: None,
                    is_meta: false,
                },
                StructuralNode {
                    node_id: TreeNodeId(3),
                    kind: StructuralNodeKind::Calculation,
                    symbol: "B".to_string(),
                    parent_id: Some(TreeNodeId(1)),
                    child_ids: vec![],
                    role: None,
                    is_meta: false,
                },
            ],
        )
        .unwrap()
    }

    fn child_display_paths_for_state(
        state: &OxCalcTreeWorkspaceState,
        parent_id: TreeNodeId,
    ) -> Vec<String> {
        let parent = state
            .snapshot
            .try_get_node(parent_id)
            .expect("parent should exist in snapshot");
        parent
            .child_ids
            .iter()
            .map(|child_id| {
                node_view_from_state(
                    state,
                    state
                        .snapshot
                        .try_get_node(*child_id)
                        .expect("child id should resolve"),
                    None,
                )
                .expect("child node should project")
                .display_path
            })
            .collect()
    }

    fn exported_input_record(
        snapshot: &OxCalcTreeWorkspaceSnapshot,
        node_id: TreeNodeId,
    ) -> &NodeInputRecord {
        snapshot
            .workspace_revision
            .node_input_snapshot
            .try_get_record(node_id)
            .expect("exported snapshot should contain node input record")
    }

    fn exported_input_text(
        snapshot: &OxCalcTreeWorkspaceSnapshot,
        node_id: TreeNodeId,
    ) -> Option<&str> {
        exported_input_record(snapshot, node_id).text.as_deref()
    }

    fn exported_input_epoch(snapshot: &OxCalcTreeWorkspaceSnapshot, node_id: TreeNodeId) -> u64 {
        exported_input_record(snapshot, node_id).input_epoch
    }

    fn fixture_formula(owner_node_id: TreeNodeId, ast: FixtureFormulaAst) -> TreeFormula {
        ast.to_tree_formula(owner_node_id)
    }

    fn sales_table_snapshot(table_node_id: TreeNodeId) -> TreeCalcTableNodeSnapshot {
        TreeCalcTableNodeSnapshot {
            table_node_id,
            table_id: "table:sales".to_string(),
            table_name: "SalesTable".to_string(),
            display_path: "stale/display/path".to_string(),
            canonical_path: "stale/canonical/path".to_string(),
            virtual_anchor: TreeCalcTableVirtualAnchor {
                workbook_scope_ref: "Book1".to_string(),
                sheet_scope_ref: "Sheet1".to_string(),
                start_row: 1,
                start_col: 1,
            },
            rows: vec![
                TreeCalcTableRowId("row:1".to_string()),
                TreeCalcTableRowId("row:2".to_string()),
            ],
            columns: vec![TreeCalcTableColumnSnapshot {
                column_id: "table:sales:col:amount".to_string(),
                column_name: "Amount".to_string(),
                ordinal: 1,
                body_metadata: TreeCalcTableColumnBodyMetadata::ConstantCells,
                totals_metadata: None,
            }],
            body_cell_nodes: Vec::new(),
            totals_cell_nodes: Vec::new(),
            header_row_present: true,
            totals_row_present: false,
            table_namespace_version: "host-supplied-namespace-should-not-win".to_string(),
            row_membership_version: "rows:v1".to_string(),
            row_order_version: "row-order:v1".to_string(),
            column_identity_version: "columns:v1".to_string(),
        }
    }

    fn sales_table_snapshot_with_body_cell_nodes(
        table_node_id: TreeNodeId,
        row_1_amount_node_id: TreeNodeId,
        row_2_amount_node_id: TreeNodeId,
        totals_amount_node_id: TreeNodeId,
    ) -> TreeCalcTableNodeSnapshot {
        let mut snapshot = sales_table_snapshot(table_node_id);
        snapshot.totals_row_present = true;
        snapshot.columns[0].totals_metadata = Some(TreeCalcTableFormulaMetadata {
            formula_artifact_id: "formula:totals:amount".to_string(),
            bind_artifact_id: Some("bind:totals:amount".to_string()),
            formula_text_version: "v1".to_string(),
            formula_text: "=SUM([Amount])".to_string(),
        });
        snapshot.body_cell_nodes = vec![
            TreeCalcTableBodyCellNodeBinding {
                row_id: TreeCalcTableRowId("row:1".to_string()),
                column_id: "table:sales:col:amount".to_string(),
                node_id: row_1_amount_node_id,
            },
            TreeCalcTableBodyCellNodeBinding {
                row_id: TreeCalcTableRowId("row:2".to_string()),
                column_id: "table:sales:col:amount".to_string(),
                node_id: row_2_amount_node_id,
            },
        ];
        snapshot.totals_cell_nodes = vec![TreeCalcTableTotalsCellNodeBinding {
            column_id: "table:sales:col:amount".to_string(),
            node_id: totals_amount_node_id,
        }];
        snapshot
    }

    fn context_with_sales_table(
        workspace_name: &str,
    ) -> (OxCalcTreeContext, OxCalcTreeWorkspaceId, TreeNodeId) {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(workspace_name))
            .unwrap();
        let sales_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("Sales", ""))
            .unwrap();
        let row_1_amount_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Amount_r1", "10").under(sales_id),
            )
            .unwrap();
        let row_2_amount_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Amount_r2", "20").under(sales_id),
            )
            .unwrap();
        let totals_amount_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Amount_total", "30").under(sales_id),
            )
            .unwrap();
        context
            .set_node_table(
                &workspace_id,
                sales_id,
                sales_table_snapshot_with_body_cell_nodes(
                    sales_id,
                    row_1_amount_id,
                    row_2_amount_id,
                    totals_amount_id,
                ),
            )
            .unwrap();
        (context, workspace_id, sales_id)
    }

    fn runtime_sales_table_snapshot(table_node_id: TreeNodeId) -> TreeCalcTableNodeSnapshot {
        TreeCalcTableNodeSnapshot {
            table_node_id,
            table_id: "tree-table:sales".to_string(),
            table_name: "SalesTable".to_string(),
            display_path: "Sales Table".to_string(),
            canonical_path: "Root/SalesTable".to_string(),
            virtual_anchor: TreeCalcTableVirtualAnchor {
                workbook_scope_ref: "treecalc-workbook:main".to_string(),
                sheet_scope_ref: "sheet:default".to_string(),
                start_row: 3,
                start_col: 2,
            },
            rows: vec![
                TreeCalcTableRowId("row:west".to_string()),
                TreeCalcTableRowId("row:east".to_string()),
                TreeCalcTableRowId("row:north".to_string()),
            ],
            columns: vec![
                TreeCalcTableColumnSnapshot {
                    column_id: "col:amount".to_string(),
                    column_name: "Amount".to_string(),
                    ordinal: 2,
                    body_metadata: TreeCalcTableColumnBodyMetadata::ConstantCells,
                    totals_metadata: Some(TreeCalcTableFormulaMetadata {
                        formula_artifact_id: "formula:totals:amount".to_string(),
                        bind_artifact_id: Some("bind:totals:amount".to_string()),
                        formula_text_version: "v1".to_string(),
                        formula_text: "=SUM([Amount])".to_string(),
                    }),
                },
                TreeCalcTableColumnSnapshot {
                    column_id: "col:region".to_string(),
                    column_name: "Region".to_string(),
                    ordinal: 1,
                    body_metadata: TreeCalcTableColumnBodyMetadata::ConstantCells,
                    totals_metadata: None,
                },
                TreeCalcTableColumnSnapshot {
                    column_id: "col:tax".to_string(),
                    column_name: "Tax".to_string(),
                    ordinal: 3,
                    body_metadata: TreeCalcTableColumnBodyMetadata::Formula(
                        TreeCalcTableFormulaMetadata {
                            formula_artifact_id: "formula:body:tax".to_string(),
                            bind_artifact_id: Some("bind:body:tax".to_string()),
                            formula_text_version: "v1".to_string(),
                            formula_text: "=[@Amount]*0.1".to_string(),
                        },
                    ),
                    totals_metadata: Some(TreeCalcTableFormulaMetadata {
                        formula_artifact_id: "formula:totals:tax".to_string(),
                        bind_artifact_id: None,
                        formula_text_version: "v1".to_string(),
                        formula_text: "=SUM([Tax])".to_string(),
                    }),
                },
            ],
            body_cell_nodes: Vec::new(),
            totals_cell_nodes: Vec::new(),
            header_row_present: true,
            totals_row_present: true,
            table_namespace_version: "namespace:v1".to_string(),
            row_membership_version: "row-membership:v1".to_string(),
            row_order_version: "row-order:v1".to_string(),
            column_identity_version: "columns:v1".to_string(),
        }
    }

    fn runtime_sales_amount_values() -> Vec<TreeCalcTableSparseValue> {
        vec![
            TreeCalcTableSparseValue::data("row:west", "col:amount", CalcValue::number(10.0)),
            TreeCalcTableSparseValue::data("row:east", "col:amount", CalcValue::number(20.0)),
            TreeCalcTableSparseValue::data("row:north", "col:amount", CalcValue::number(30.0)),
        ]
    }

    #[test]
    fn treecalc_context_direct_workspace_api_evaluates_bare_name_formula() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new("workspace:direct"))
            .unwrap();
        let a_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("A", "=3"))
            .unwrap();
        let b_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("B", "=A+1"))
            .unwrap();

        let result = context.recalculate(&workspace_id).unwrap();

        assert_eq!(
            result.run_state,
            OxCalcTreeRunState::Published,
            "static-vs-ctro recalculation should publish: reject={:?}; diagnostics={:?}",
            result.reject_detail,
            result.diagnostics
        );
        assert_eq!(result.published_values.get(&a_id), Some(&"3".to_string()));
        assert_eq!(result.published_values.get(&b_id), Some(&"4".to_string()));

        let a_view = context.node_view(&workspace_id, a_id).unwrap();
        let b_view = context.node_view(&workspace_id, b_id).unwrap();
        assert_eq!(a_view.canonical_path, "Root/A");
        assert_eq!(a_view.formula_text, "=3");
        assert_eq!(a_view.value_text.as_deref(), Some("3"));
        assert_eq!(b_view.canonical_path, "Root/B");
        assert_eq!(b_view.formula_text, "=A+1");
        assert_eq!(b_view.value_text.as_deref(), Some("4"));
    }

    #[test]
    fn grid_backing_custody_attaches_reads_and_clears() {
        use crate::grid::authored::{GridAuthoredCell, GridFormulaCell};
        use crate::grid::coords::{ExcelGridBounds, ExcelGridCellAddress};

        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new("workspace:grid"))
            .unwrap();
        let sheet_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("Sheet1", ""))
            .unwrap();

        let address = |row, col| ExcelGridCellAddress::new("book:grid", "sheet:grid", row, col);
        let seed = GridBackingSeed {
            workbook_id: "book:grid".to_string(),
            sheet_id: "sheet:grid".to_string(),
            bounds: ExcelGridBounds::strict_excel(),
            authored: vec![
                (
                    address(1, 1),
                    GridAuthoredCell::Literal(CalcValue::number(7.0)),
                ),
                (
                    address(1, 2),
                    GridAuthoredCell::Formula(GridFormulaCell::new(
                        "=A1*3",
                        "excel.grid.v1:cell:R[0]C[-1]*3",
                    )),
                ),
            ],
            table_overlays: Vec::new(),
            merged_regions: Vec::new(),
        };

        let view = context
            .set_node_grid(&workspace_id, sheet_id, seed)
            .unwrap();

        // The sheet node owns a grid backing and both engines agree.
        assert!(
            view.differential_mismatches.is_empty(),
            "reference and optimized engines must agree: {:?}",
            view.differential_mismatches
        );
        assert_eq!(view.grid_id, "book:grid:sheet:grid");
        // The grid-backed node is enumerable for projection.
        assert_eq!(
            context.grid_backed_node_ids(&workspace_id).unwrap(),
            vec![sheet_id]
        );
        let value_at = |view: &OxCalcTreeGridView, row, col| {
            view.cells
                .iter()
                .find(|cell| cell.address == address(row, col))
                .map(|cell| cell.value.clone())
        };
        assert_eq!(value_at(&view, 1, 1), Some(CalcValue::number(7.0)));
        assert_eq!(value_at(&view, 1, 2), Some(CalcValue::number(21.0)));
        // Each cell carries a value epoch stamped by the recalc (the foundation
        // for region-scoped deltas in the next bead).
        assert!(view.cells.iter().all(|cell| cell.value_epoch >= 1));

        // grid_view reads the same values back (from the cache, no re-recalc).
        let reread = context.grid_view(&workspace_id, sheet_id).unwrap().unwrap();
        assert_eq!(value_at(&reread, 1, 2), Some(CalcValue::number(21.0)));

        // Clearing removes the backing; a second clear is a no-op.
        assert!(context.clear_node_grid(&workspace_id, sheet_id).unwrap());
        assert!(
            context
                .grid_view(&workspace_id, sheet_id)
                .unwrap()
                .is_none()
        );
        assert!(!context.clear_node_grid(&workspace_id, sheet_id).unwrap());
    }

    #[test]
    fn grid_interest_scopes_reads_and_poll_changes() {
        use crate::grid::authored::{GridAuthoredCell, GridFormulaCell};
        use crate::grid::coords::{ExcelGridBounds, ExcelGridCellAddress};
        use crate::grid::geometry::GridRect;

        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new("workspace:grid-interest"))
            .unwrap();
        let sheet_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("Sheet1", ""))
            .unwrap();

        let bounds = ExcelGridBounds::strict_excel();
        let address = |row, col| ExcelGridCellAddress::new("book:gi", "sheet:gi", row, col);
        let seed = GridBackingSeed {
            workbook_id: "book:gi".to_string(),
            sheet_id: "sheet:gi".to_string(),
            bounds,
            authored: vec![
                (
                    address(1, 1),
                    GridAuthoredCell::Literal(CalcValue::number(7.0)),
                ),
                (
                    address(1, 2),
                    GridAuthoredCell::Formula(GridFormulaCell::new(
                        "=A1*3",
                        "excel.grid.v1:cell:R[0]C[-1]*3",
                    )),
                ),
            ],
            table_overlays: Vec::new(),
            merged_regions: Vec::new(),
        };
        let full = context
            .set_node_grid(&workspace_id, sheet_id, seed)
            .unwrap();
        assert_eq!(full.cells.len(), 2);

        // Register interest in column A only (A1:A1); the view narrows to A1.
        let epoch = context
            .register_grid_interest(
                &workspace_id,
                sheet_id,
                GridInterestRegions {
                    viewport: Some(
                        GridRect::new("book:gi", "sheet:gi", 1, 1, 1, 1, bounds).unwrap(),
                    ),
                    monitored: Vec::new(),
                },
            )
            .unwrap()
            .unwrap();
        let scoped = context.grid_view(&workspace_id, sheet_id).unwrap().unwrap();
        assert_eq!(scoped.cells.len(), 1);
        assert_eq!(scoped.cells[0].address, address(1, 1));

        // Pull since epoch 0: the interested cell changed since then.
        let delta = context
            .poll_grid_changes(&workspace_id, sheet_id, GridInterestEpoch(0))
            .unwrap()
            .unwrap();
        assert!(!delta.resync);
        assert_eq!(delta.changed.len(), 1);
        assert_eq!(delta.changed[0].address, address(1, 1));
        assert_eq!(delta.to_epoch, epoch);

        // Pull since the current epoch: nothing changed after registration.
        let quiet = context
            .poll_grid_changes(&workspace_id, sheet_id, epoch)
            .unwrap()
            .unwrap();
        assert!(quiet.changed.is_empty());
    }

    #[test]
    fn grid_edit_setcell_and_fillrange_publish_and_bump_epochs() {
        use crate::grid::authored::{GridAuthoredCell, GridFormulaCell};
        use crate::grid::coords::{ExcelGridBounds, ExcelGridCellAddress};
        use crate::grid::geometry::GridRect;

        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new("workspace:grid-edit"))
            .unwrap();
        let sheet_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("Sheet1", ""))
            .unwrap();

        let bounds = ExcelGridBounds::strict_excel();
        let address = |row, col| ExcelGridCellAddress::new("book:ge", "sheet:ge", row, col);
        let value_at = |view: &OxCalcTreeGridView, row, col| {
            view.cells
                .iter()
                .find(|cell| cell.address == address(row, col))
                .map(|cell| cell.value.clone())
        };
        let seed = GridBackingSeed {
            workbook_id: "book:ge".to_string(),
            sheet_id: "sheet:ge".to_string(),
            bounds,
            authored: vec![
                (
                    address(1, 1),
                    GridAuthoredCell::Literal(CalcValue::number(7.0)),
                ),
                (
                    address(1, 2),
                    GridAuthoredCell::Formula(GridFormulaCell::new(
                        "=A1*3",
                        "excel.grid.v1:cell:R[0]C[-1]*3",
                    )),
                ),
            ],
            table_overlays: Vec::new(),
            merged_regions: Vec::new(),
        };
        context
            .set_node_grid(&workspace_id, sheet_id, seed)
            .unwrap();
        let baseline = context
            .poll_grid_changes(&workspace_id, sheet_id, GridInterestEpoch(0))
            .unwrap()
            .unwrap()
            .to_epoch;

        // SetCell A1: 7 -> 10; the dependent B1 (=A1*3) recomputes to 30.
        let after_set = context
            .apply_grid_edit(
                &workspace_id,
                sheet_id,
                OxCalcTreeGridOp::SetCell {
                    address: address(1, 1),
                    cell: GridAuthoredCell::Literal(CalcValue::number(10.0)),
                },
            )
            .unwrap()
            .unwrap();
        assert!(after_set.differential_mismatches.is_empty());
        assert_eq!(value_at(&after_set, 1, 1), Some(CalcValue::number(10.0)));
        assert_eq!(value_at(&after_set, 1, 2), Some(CalcValue::number(30.0)));

        // The edit bumped the epoch of both the edited cell and its dependent.
        let delta = context
            .poll_grid_changes(&workspace_id, sheet_id, baseline)
            .unwrap()
            .unwrap();
        let changed: BTreeSet<_> = delta.changed.iter().map(|c| c.address.clone()).collect();
        assert!(changed.contains(&address(1, 1)));
        assert!(changed.contains(&address(1, 2)));

        // FillRange C1:C3 with the repeated R1C1 formula =A1 (one region, not 3
        // cells). The differential being clean is the materialization-invariance
        // check; C1 resolves to A1 = 10.
        let fill_rect = GridRect::new("book:ge", "sheet:ge", 1, 3, 3, 3, bounds).unwrap();
        let after_fill = context
            .apply_grid_edit(
                &workspace_id,
                sheet_id,
                OxCalcTreeGridOp::FillRange {
                    rect: fill_rect,
                    formula: GridFormulaCell::new("=A1", "excel.grid.v1:cell:R[0]C[-2]"),
                },
            )
            .unwrap()
            .unwrap();
        assert!(
            after_fill.differential_mismatches.is_empty(),
            "fill-range materialization invariance: {:?}",
            after_fill.differential_mismatches
        );
        assert_eq!(value_at(&after_fill, 1, 3), Some(CalcValue::number(10.0)));
    }

    #[test]
    fn grid_overlays_surface_committed_tables_and_merged_window_clipped() {
        use crate::grid::authored::GridAuthoredCell;
        use crate::grid::coords::{ExcelGridBounds, ExcelGridCellAddress};
        use crate::grid::geometry::GridRect;
        use crate::grid::machine::{GridTableColumn, GridTableOverlay};

        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new("workspace:grid-overlays"))
            .unwrap();
        let sheet_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("Sheet1", ""))
            .unwrap();

        let bounds = ExcelGridBounds::strict_excel();
        let address = |row, col| ExcelGridCellAddress::new("book:ov", "sheet:ov", row, col);
        let rect = |top, left, bottom, right| {
            GridRect::new("book:ov", "sheet:ov", top, left, bottom, right, bounds).unwrap()
        };

        // A structured table A1:B4 (header row 1, two data columns) and a merged
        // region C1:D2 - both committed document state.
        let table = GridTableOverlay::new(
            "table1",
            "Sales",
            rect(1, 1, 4, 2),
            vec![
                GridTableColumn::new("table1:region", "Region", 1, rect(2, 1, 4, 1)),
                GridTableColumn::new("table1:amount", "Amount", 2, rect(2, 2, 4, 2)),
            ],
        )
        .with_header_rect(rect(1, 1, 1, 2));
        let seed = GridBackingSeed {
            workbook_id: "book:ov".to_string(),
            sheet_id: "sheet:ov".to_string(),
            bounds,
            authored: vec![(
                address(2, 1),
                GridAuthoredCell::Literal(CalcValue::number(1.0)),
            )],
            table_overlays: vec![table],
            merged_regions: vec![rect(1, 3, 2, 4)],
        };
        let view = context
            .set_node_grid(&workspace_id, sheet_id, seed)
            .unwrap();

        // Whole grid (no interest registered): overlays surface unclipped.
        assert_eq!(view.overlays.tables.len(), 1);
        let table_overlay = &view.overlays.tables[0];
        assert_eq!(table_overlay.table_id, "table1");
        assert_eq!(table_overlay.table_name, "Sales");
        let range = &table_overlay.table_range;
        assert_eq!(
            (
                range.top_row,
                range.left_col,
                range.bottom_row,
                range.right_col
            ),
            (1, 1, 4, 2)
        );
        assert!(
            !range.clipped_top
                && !range.clipped_left
                && !range.clipped_bottom
                && !range.clipped_right
        );
        assert!(table_overlay.header_rect.is_some());
        assert_eq!(table_overlay.columns.len(), 2);
        assert_eq!(view.overlays.merged.len(), 1);
        assert!(view.overlays.spills.is_empty());
        // Committed overlays bumped the overlay epoch off its zero baseline.
        let whole_grid_epoch = view.overlay_epoch;
        assert!(whole_grid_epoch >= 1);

        // Scope interest to A1:B2: the table clips (bottom edge cut), the merged
        // region C1:D2 falls entirely outside the window and drops out, and the
        // clipped overlay set advances the overlay epoch.
        context
            .register_grid_interest(
                &workspace_id,
                sheet_id,
                GridInterestRegions {
                    viewport: Some(rect(1, 1, 2, 2)),
                    monitored: Vec::new(),
                },
            )
            .unwrap()
            .unwrap();
        let scoped = context.grid_view(&workspace_id, sheet_id).unwrap().unwrap();
        assert_eq!(scoped.overlays.tables.len(), 1);
        let scoped_range = &scoped.overlays.tables[0].table_range;
        assert_eq!(scoped_range.bottom_row, 2);
        assert!(scoped_range.clipped_bottom);
        assert!(!scoped_range.clipped_top);
        assert_eq!(scoped.overlays.merged.len(), 0);
        let scoped_epoch = scoped.overlay_epoch;
        assert!(scoped_epoch > whole_grid_epoch);

        // A value-only edit inside the window does not disturb the overlay set,
        // so the overlay epoch holds steady (overlays and values track epochs
        // independently).
        let after = context
            .apply_grid_edit(
                &workspace_id,
                sheet_id,
                OxCalcTreeGridOp::SetCell {
                    address: address(2, 1),
                    cell: GridAuthoredCell::Literal(CalcValue::number(2.0)),
                },
            )
            .unwrap()
            .unwrap();
        assert_eq!(
            after.overlay_epoch, scoped_epoch,
            "a value-only edit must not bump the overlay epoch"
        );
    }

    #[test]
    fn project_grid_overlays_clips_all_families_including_spills() {
        use crate::grid::coords::{ExcelGridBounds, ExcelGridCellAddress};
        use crate::grid::geometry::GridRect;
        use crate::grid::machine::{
            GridOptimizedSheet, GridSpillFact, GridTableColumn, GridTableOverlay,
        };

        let bounds = ExcelGridBounds::strict_excel();
        let rect = |top, left, bottom, right| {
            GridRect::new("book:ov2", "sheet:ov2", top, left, bottom, right, bounds).unwrap()
        };
        let mut sheet = GridOptimizedSheet::new("book:ov2", "sheet:ov2", bounds);
        sheet
            .set_table_overlay(
                GridTableOverlay::new(
                    "t",
                    "T",
                    rect(1, 1, 3, 2),
                    vec![GridTableColumn::new("t:c1", "C1", 1, rect(1, 1, 3, 1))],
                )
                .with_header_rect(rect(1, 1, 1, 2)),
            )
            .unwrap();
        sheet.add_merged_region(rect(5, 5, 6, 6)).unwrap();
        // A spill anchored at D1 (col 4) spilling down D1:D4, surfaced as a calc
        // result (here supplied directly to the projector).
        let spills = vec![GridSpillFact {
            anchor: ExcelGridCellAddress::new("book:ov2", "sheet:ov2", 1, 4),
            extent: rect(1, 4, 4, 4),
            blocked: false,
        }];

        // Whole grid: every family present and unclipped.
        let all = project_grid_overlays(&sheet, &spills, &None);
        assert_eq!(all.tables.len(), 1);
        assert_eq!(all.spills.len(), 1);
        assert_eq!(all.merged.len(), 1);
        assert_eq!(
            all.spills[0].anchor,
            ExcelGridCellAddress::new("book:ov2", "sheet:ov2", 1, 4)
        );
        assert!(!all.spills[0].blocked);
        assert!(!all.spills[0].extent.clipped_bottom);

        // Window rows 1-2, cols 1-4: the table and spill clip at the bottom; the
        // merged region (rows 5-6) is wholly outside and drops.
        let scoped = project_grid_overlays(
            &sheet,
            &spills,
            &Some(GridInterestRegions {
                viewport: Some(rect(1, 1, 2, 4)),
                monitored: Vec::new(),
            }),
        );
        assert_eq!(scoped.tables.len(), 1);
        assert_eq!(scoped.tables[0].table_range.bottom_row, 2);
        assert!(scoped.tables[0].table_range.clipped_bottom);
        assert_eq!(scoped.spills.len(), 1);
        assert_eq!(scoped.spills[0].extent.bottom_row, 2);
        assert!(scoped.spills[0].extent.clipped_bottom);
        assert_eq!(scoped.merged.len(), 0);

        // Interest registered but covering nothing: no overlays surface.
        let empty = project_grid_overlays(&sheet, &spills, &Some(GridInterestRegions::default()));
        assert!(empty.tables.is_empty() && empty.spills.is_empty() && empty.merged.is_empty());
    }

    #[test]
    fn treecalc_context_current_dependency_graph_tracks_structural_edits_between_runs() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:current-dependency-graph",
            ))
            .unwrap();
        let a_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("A", "=3"))
            .unwrap();
        let b_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("B", "=A+1"))
            .unwrap();

        let pre_run = context.current_dependency_graph(&workspace_id).unwrap();
        assert!(
            pre_run
                .edges_by_owner
                .get(&b_id)
                .into_iter()
                .flatten()
                .any(|edge| edge.target_node_id == a_id),
            "dependency facts should be readable before any calculation run"
        );

        let outcome = context.recalculate(&workspace_id).unwrap();
        assert_eq!(outcome.run_state, OxCalcTreeRunState::Published);

        let c_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("C", "=B*2"))
            .unwrap();

        let current = context.current_dependency_graph(&workspace_id).unwrap();
        assert!(
            current
                .edges_by_owner
                .get(&b_id)
                .into_iter()
                .flatten()
                .any(|edge| edge.target_node_id == a_id),
            "existing dependency edges should survive a structural edit without recalc"
        );
        assert!(
            current
                .edges_by_owner
                .get(&c_id)
                .into_iter()
                .flatten()
                .any(|edge| edge.target_node_id == b_id),
            "a node added after the last run should already carry its dependency edge"
        );
        assert!(
            current
                .reverse_edges
                .get(&b_id)
                .into_iter()
                .flatten()
                .any(|edge| edge.owner_node_id == c_id),
            "reverse edges should name the post-run dependent"
        );

        let view = context.workspace_view(&workspace_id).unwrap();
        assert_eq!(
            current.snapshot_id, view.snapshot_id,
            "current dependency graph should be built on the current structural snapshot"
        );
        assert_ne!(
            current.snapshot_id, outcome.dependency_graph.snapshot_id,
            "the retained run outcome graph is stale after the structural edit"
        );
    }

    #[test]
    fn treecalc_context_recalculation_retains_published_array_calc_values() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new("workspace:array-retention"))
            .unwrap();
        let array_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("ArrayNode", "=SEQUENCE(2,2)"),
            )
            .unwrap();

        let initial = context.recalculate(&workspace_id).unwrap();
        assert_eq!(initial.run_state, OxCalcTreeRunState::Published);
        let CoreValue::Array(initial_array) = &initial.published_calc_values[&array_id].core else {
            panic!(
                "first recalc should publish a typed array CalcValue, got {:?}",
                initial.published_calc_values[&array_id]
            );
        };
        assert_eq!(initial_array.shape().rows, 2);
        assert_eq!(initial_array.shape().cols, 2);

        let rerun = context.recalculate(&workspace_id).unwrap();
        let CoreValue::Array(retained_array) = &rerun.published_calc_values[&array_id].core else {
            panic!(
                "recalc should retain the typed array CalcValue, got {:?}",
                rerun.published_calc_values[&array_id]
            );
        };
        assert_eq!(retained_array.shape().rows, 2);
        assert_eq!(retained_array.shape().cols, 2);
    }

    #[test]
    fn treecalc_context_workspace_creation_builds_revision_and_root_input_snapshot() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new("workspace:w057-revision"))
            .unwrap();

        let workspace_view = context.workspace_view(&workspace_id).unwrap();
        let revision = context.workspace_revision(&workspace_id).unwrap();
        let root_record = revision
            .node_input_snapshot
            .try_get_record(workspace_view.root_node_id)
            .expect("root input record should exist");

        assert_eq!(revision.workspace_id.as_str(), workspace_id.as_str());
        assert_eq!(
            revision.structure_snapshot.snapshot_id(),
            workspace_view.snapshot_id
        );
        assert_eq!(
            revision.revision_id(),
            &workspace_view.workspace_revision_id
        );
        assert_eq!(
            revision.node_input_snapshot.snapshot_id(),
            &workspace_view.node_input_snapshot_id
        );
        assert_eq!(
            revision.namespace_snapshot.snapshot_id(),
            &workspace_view.namespace_snapshot_id
        );
        assert_eq!(root_record.kind, NodeInputKind::Empty);
        assert_eq!(root_record.text, None);
        assert!(
            workspace_view
                .formula_binding_snapshot_id
                .0
                .contains("absent")
        );
        assert!(
            workspace_view
                .dependency_shape_snapshot_id
                .0
                .contains("absent")
        );
        assert!(workspace_view.publication_snapshot_id.0.contains("absent"));
        assert!(workspace_view.runtime_overlay_set_id.0.contains("absent"));
    }

    #[test]
    fn treecalc_context_catalog_uses_node_input_snapshot_as_formula_authority() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:w057-formula-authority",
            ))
            .unwrap();
        let a_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("A", "3"))
            .unwrap();
        let b_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("B", "4"))
            .unwrap();

        {
            let state = context.workspace_mut(&workspace_id).unwrap();
            let node_input_snapshot = state
                .workspace_revision
                .node_input_snapshot
                .with_record(NodeInputRecord::formula_text(a_id, "=B+1", 99));
            replace_node_input_snapshot(state, node_input_snapshot);
        }

        let state = context.workspace(&workspace_id).unwrap();
        let a_record = state
            .workspace_revision
            .node_input_snapshot
            .try_get_record(a_id)
            .unwrap();
        let catalog_build =
            build_context_formula_catalog(state, &OxCalcTreeContextOptions::default()).unwrap();

        let a_binding = catalog_build.catalog.try_get_binding(a_id).unwrap();
        assert_eq!(a_record.kind, NodeInputKind::FormulaText);
        assert_eq!(a_record.text.as_deref(), Some("=B+1"));
        assert_eq!(
            a_binding.formula_artifact_id.0,
            "formula:workspace:w057-formula-authority:2:v99"
        );
        assert!(catalog_build.catalog.try_get_binding(b_id).is_none());
    }

    #[test]
    fn treecalc_context_recalculate_publishes_formula_binding_and_dependency_shape_snapshots() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:w057-formula-binding",
            ))
            .unwrap();
        context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("A", "=3"))
            .unwrap();
        let before = context.workspace_view(&workspace_id).unwrap();

        let result = context.recalculate(&workspace_id).unwrap();
        let after = context.workspace_view(&workspace_id).unwrap();
        let state = context.workspace(&workspace_id).unwrap();

        assert_eq!(
            result.run_state,
            OxCalcTreeRunState::Published,
            "static-vs-ctro recalculation should publish: reject={:?}; diagnostics={:?}",
            result.reject_detail,
            result.diagnostics
        );
        assert!(before.formula_binding_snapshot_id.0.contains("absent"));
        assert!(after.formula_binding_snapshot_id.0.contains("current"));
        assert!(matches!(
            &state.formula_binding_snapshot.state,
            SnapshotLayerState::Current { .. }
        ));
        assert!(after.dependency_shape_snapshot_id.0.contains("current"));
        assert!(matches!(
            &state.dependency_shape_snapshot.state,
            SnapshotLayerState::Current { .. }
        ));
        assert!(after.publication_snapshot_id.0.contains("current"));
        assert!(matches!(
            &state.publication_snapshot.state,
            SnapshotLayerState::Current { .. }
        ));
        assert!(after.runtime_overlay_set_id.0.contains("current"));
        assert!(matches!(
            &state.runtime_overlay_set.state,
            SnapshotLayerState::Current { .. }
        ));
        assert_eq!(
            state.dependency_shape_snapshot.formula_binding_snapshot_id,
            after.formula_binding_snapshot_id
        );
    }

    #[test]
    fn treecalc_context_namespace_mutation_advances_revision_and_prepared_basis() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new("workspace:w057-namespace"))
            .unwrap();
        context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("A", "=2"))
            .unwrap();
        let metadata_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("NameFormula", "=A+1"),
            )
            .unwrap();
        let before = context.workspace_view(&workspace_id).unwrap();

        let host_capabilities = OxCalcTreeHostCapabilitySnapshot {
            capability_profile_id: "capability-profile:w057-v2".to_string(),
            ..Default::default()
        };
        context.set_options(
            context
                .options()
                .clone()
                .with_namespace(
                    OxCalcTreeNamespaceOptions::new()
                        .with_host_namespace_version("treecalc-host-namespace:w057-v2")
                        .with_function_registry_version("oxfunc.arg-prep:w057-v2")
                        .with_resolution_rule_version("treecalc-host-resolution:w057-v2")
                        .with_caller_context_identity_version("treecalc-caller-context:w057-v2")
                        .with_cross_workspace_availability_version(
                            "treecalc-cross-workspace:w057-v2",
                        ),
                )
                .with_host_capabilities(host_capabilities),
        );
        let after = context.workspace_view(&workspace_id).unwrap();
        let after_revision = context.workspace_revision(&workspace_id).unwrap();

        assert_eq!(before.snapshot_id, after.snapshot_id);
        assert_eq!(before.node_input_snapshot_id, after.node_input_snapshot_id);
        assert_ne!(before.namespace_snapshot_id, after.namespace_snapshot_id);
        assert_ne!(before.workspace_revision_id, after.workspace_revision_id);
        assert_eq!(
            after_revision.namespace_snapshot.host_namespace_version,
            "treecalc-host-namespace:w057-v2"
        );
        assert_eq!(
            after_revision.namespace_snapshot.function_registry_version,
            "oxfunc.arg-prep:w057-v2"
        );
        assert_eq!(
            after_revision.namespace_snapshot.capability_profile_id,
            "capability-profile:w057-v2"
        );
        let formula_host_context = context_formula_host_context(
            context.workspace(&workspace_id).unwrap(),
            metadata_id,
            None,
        );
        assert_eq!(
            formula_host_context.host_namespace_version.as_deref(),
            Some("treecalc-host-namespace:w057-v2")
        );
        assert_eq!(
            formula_host_context.registry_snapshot_identity.as_deref(),
            Some("oxfunc.arg-prep:w057-v2")
        );
        assert_eq!(
            formula_host_context.resolution_rule_version,
            "treecalc-host-resolution:w057-v2"
        );
        assert_eq!(
            formula_host_context.caller_context_identity,
            Some(format!(
                "treecalc-caller:{};treecalc-caller-context:w057-v2",
                metadata_id
            ))
        );

        let result = context.recalculate(&workspace_id).unwrap();
        let candidate = result
            .candidate_result
            .as_ref()
            .expect("namespace-mutated recalc should publish candidate work");
        assert_eq!(
            result.published_values.get(&metadata_id),
            Some(&"3".to_string())
        );
        assert!(
            candidate
                .compatibility_basis
                .contains(&after.workspace_revision_id.0)
        );
        assert!(
            candidate
                .compatibility_basis
                .contains(&after.namespace_snapshot_id.0)
        );
        assert!(
            candidate
                .artifact_token_basis
                .contains(&after.namespace_snapshot_id.0)
        );
        assert!(result.diagnostics.iter().any(|diagnostic| {
            diagnostic
                == "oxcalc_tree_context_options_host_namespace_version:treecalc-host-namespace:w057-v2"
        }));
        assert!(result.diagnostics.iter().any(|diagnostic| {
            diagnostic
                == "oxcalc_tree_context_options_caller_context_identity_version:treecalc-caller-context:w057-v2"
        }));
    }

    #[test]
    fn treecalc_context_namespace_mutation_after_publish_recalculates_under_new_basis() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:w057-namespace-published",
            ))
            .unwrap();
        let a_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("A", "1"))
            .unwrap();
        let b_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("B", "=A+1"))
            .unwrap();
        let initial = context.recalculate(&workspace_id).unwrap();
        assert_eq!(initial.run_state, OxCalcTreeRunState::Published);
        assert_eq!(initial.published_values.get(&b_id), Some(&"2".to_string()));
        let before = context.workspace_view(&workspace_id).unwrap();

        context.set_options(
            context.options().clone().with_namespace(
                OxCalcTreeNamespaceOptions::new()
                    .with_host_namespace_version("treecalc-host-namespace:w057-published")
                    .with_function_registry_version("oxfunc.arg-prep:w057-published"),
            ),
        );
        let after = context.workspace_view(&workspace_id).unwrap();
        let result = context.recalculate(&workspace_id).unwrap();

        assert_eq!(before.snapshot_id, after.snapshot_id);
        assert_eq!(before.node_input_snapshot_id, after.node_input_snapshot_id);
        assert_ne!(before.namespace_snapshot_id, after.namespace_snapshot_id);
        assert_ne!(before.workspace_revision_id, after.workspace_revision_id);
        assert!(
            matches!(
                result.run_state,
                OxCalcTreeRunState::Published | OxCalcTreeRunState::VerifiedClean
            ),
            "namespace-mutated published baseline should not reject: {:?}",
            result.reject_detail
        );
        assert_eq!(result.published_values.get(&b_id), Some(&"2".to_string()));
        assert!(
            result
                .invalidation_closure
                .records
                .get(&b_id)
                .is_some_and(|record| record
                    .reasons
                    .contains(&InvalidationReasonKind::StructuralRecalcOnly))
        );
        assert!(a_id.0 > 0);
    }

    #[test]
    fn treecalc_context_structural_edits_advance_revision_roots_and_preserve_inputs() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:w057-structural-revisions",
            ))
            .unwrap();
        let a_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("A", "3"))
            .unwrap();
        let branch_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("Branch", ""))
            .unwrap();
        let b_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("B", "=A+1").under(branch_id),
            )
            .unwrap();
        let before = context.workspace_view(&workspace_id).unwrap();
        let before_revision = context.workspace_revision(&workspace_id).unwrap();
        let before_records = before_revision.node_input_snapshot.records().clone();

        context
            .rename_node(&workspace_id, a_id, "A_renamed")
            .unwrap();
        let after_rename = context.workspace_view(&workspace_id).unwrap();
        let after_rename_revision = context.workspace_revision(&workspace_id).unwrap();

        assert_ne!(before.snapshot_id, after_rename.snapshot_id);
        assert_ne!(
            before.workspace_revision_id,
            after_rename.workspace_revision_id
        );
        assert_eq!(
            before.node_input_snapshot_id,
            after_rename.node_input_snapshot_id
        );
        assert_eq!(
            before.namespace_snapshot_id,
            after_rename.namespace_snapshot_id
        );
        assert_eq!(
            after_rename_revision.structure_snapshot.snapshot_id(),
            after_rename.snapshot_id
        );
        assert_eq!(
            after_rename_revision.node_input_snapshot.records(),
            &before_records
        );

        context
            .move_node(&workspace_id, b_id, before.root_node_id, Some(0))
            .unwrap();
        let after_move = context.workspace_view(&workspace_id).unwrap();
        let after_move_revision = context.workspace_revision(&workspace_id).unwrap();
        assert_ne!(after_rename.snapshot_id, after_move.snapshot_id);
        assert_ne!(
            after_rename.workspace_revision_id,
            after_move.workspace_revision_id
        );
        assert_eq!(
            after_rename.node_input_snapshot_id,
            after_move.node_input_snapshot_id
        );
        assert_eq!(
            after_rename.namespace_snapshot_id,
            after_move.namespace_snapshot_id
        );
        assert_eq!(
            after_move_revision.node_input_snapshot.records(),
            &before_records
        );

        context.reorder_node(&workspace_id, a_id, 0).unwrap();
        let after_reorder = context.workspace_view(&workspace_id).unwrap();
        assert_ne!(after_move.snapshot_id, after_reorder.snapshot_id);
        assert_ne!(
            after_move.workspace_revision_id,
            after_reorder.workspace_revision_id
        );
        assert_eq!(
            after_move.node_input_snapshot_id,
            after_reorder.node_input_snapshot_id
        );

        let c_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("C", "5"))
            .unwrap();
        let after_add = context.workspace_view(&workspace_id).unwrap();
        let after_add_revision = context.workspace_revision(&workspace_id).unwrap();

        assert_ne!(after_reorder.snapshot_id, after_add.snapshot_id);
        assert_ne!(
            after_reorder.workspace_revision_id,
            after_add.workspace_revision_id
        );
        assert_ne!(
            after_reorder.node_input_snapshot_id,
            after_add.node_input_snapshot_id
        );
        for node_id in [a_id, branch_id, b_id] {
            assert_eq!(
                before_records.get(&node_id),
                after_add_revision
                    .node_input_snapshot
                    .records()
                    .get(&node_id)
            );
        }
        assert_eq!(
            after_add_revision
                .node_input_snapshot
                .try_get_record(c_id)
                .map(|record| record.kind),
            Some(NodeInputKind::Literal)
        );
    }

    #[test]
    fn treecalc_context_delete_prunes_inputs_publication_runtime_and_table_shape() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:w057-delete-prune",
            ))
            .unwrap();
        let a_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("A", "2"))
            .unwrap();
        let b1_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("B1", "4"))
            .unwrap();
        let b2_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("B2", "5"))
            .unwrap();
        let c_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("C", "=INDIRECT(\"B\"&A)"),
            )
            .unwrap();
        context
            .set_node_table(&workspace_id, b2_id, sales_table_snapshot(b2_id))
            .unwrap();
        let initial = context.recalculate(&workspace_id).unwrap();
        assert_eq!(initial.published_values.get(&c_id), Some(&"5".to_string()));
        assert!(initial.runtime_effects.iter().any(|effect| {
            effect.family == RuntimeEffectFamily::DynamicDependency
                && effect.detail.contains(&format!("target_node:{b2_id}"))
        }));
        let before_delete = context.workspace_view(&workspace_id).unwrap();
        let before_delete_revision = context.workspace_revision(&workspace_id).unwrap();
        assert!(
            before_delete_revision
                .structure_snapshot
                .table_shapes()
                .contains_key(&b2_id)
        );

        context.delete_node(&workspace_id, b2_id).unwrap();
        let after_delete = context.workspace_view(&workspace_id).unwrap();
        let after_delete_revision = context.workspace_revision(&workspace_id).unwrap();
        let exported = context.export_workspace_snapshot(&workspace_id).unwrap();

        assert_ne!(before_delete.snapshot_id, after_delete.snapshot_id);
        assert_ne!(
            before_delete.workspace_revision_id,
            after_delete.workspace_revision_id
        );
        assert_ne!(
            before_delete.node_input_snapshot_id,
            after_delete.node_input_snapshot_id
        );
        assert!(
            after_delete_revision
                .structure_snapshot
                .try_get_node(b2_id)
                .is_none()
        );
        assert!(
            after_delete_revision
                .node_input_snapshot
                .try_get_record(b2_id)
                .is_none()
        );
        assert!(
            !after_delete_revision
                .structure_snapshot
                .table_shapes()
                .contains_key(&b2_id)
        );
        assert!(
            exported
                .workspace_revision
                .node_input_snapshot
                .try_get_record(b2_id)
                .is_none()
        );
        assert!(exported.publication_values.is_empty());
        assert!(!exported.table_snapshots.contains_key(&b2_id));
        assert!(
            exported
                .deleted_table_facts
                .iter()
                .any(|fact| fact.table_id == "table:sales")
        );
        assert!(exported.publication_runtime_effects.is_empty());
        assert_eq!(
            after_delete_revision
                .node_input_snapshot
                .try_get_record(a_id)
                .map(|record| record.kind),
            Some(NodeInputKind::Literal)
        );
        assert_eq!(
            after_delete_revision
                .node_input_snapshot
                .try_get_record(b1_id)
                .map(|record| record.kind),
            Some(NodeInputKind::Literal)
        );
        assert_eq!(
            after_delete_revision
                .node_input_snapshot
                .try_get_record(c_id)
                .map(|record| record.kind),
            Some(NodeInputKind::FormulaText)
        );
    }

    #[test]
    fn treecalc_context_formula_edit_recalculates_dependents() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new("workspace:edit"))
            .unwrap();
        let a_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("A", "=3"))
            .unwrap();
        let b_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("B", "=A+1"))
            .unwrap();
        context.recalculate(&workspace_id).unwrap();
        let before_edit = context.workspace_view(&workspace_id).unwrap();
        let before_snapshot = context.export_workspace_snapshot(&workspace_id).unwrap();
        let before_a_formula_version = exported_input_epoch(&before_snapshot, a_id);

        context
            .set_node_formula_text(&workspace_id, a_id, "=4")
            .unwrap();
        let after_edit_before_recalc = context.workspace_view(&workspace_id).unwrap();
        let result = context.recalculate(&workspace_id).unwrap();
        let after_recalc = context.workspace_view(&workspace_id).unwrap();
        let after_snapshot = context.export_workspace_snapshot(&workspace_id).unwrap();

        assert_eq!(
            result.run_state,
            OxCalcTreeRunState::Published,
            "callable profile reference run failed: reject={:?}; diagnostics={:?}",
            result.reject_detail,
            result.diagnostics
        );
        assert_eq!(result.published_values.get(&a_id), Some(&"4".to_string()));
        assert_eq!(result.published_values.get(&b_id), Some(&"5".to_string()));
        assert_eq!(
            before_edit.snapshot_id,
            after_edit_before_recalc.snapshot_id
        );
        assert_eq!(before_edit.snapshot_id, after_recalc.snapshot_id);
        assert_ne!(
            before_edit.node_input_snapshot_id,
            after_edit_before_recalc.node_input_snapshot_id
        );
        assert_eq!(
            after_edit_before_recalc.node_input_snapshot_id,
            after_recalc.node_input_snapshot_id
        );
        assert_ne!(
            before_edit.workspace_revision_id,
            after_edit_before_recalc.workspace_revision_id
        );
        assert_eq!(
            after_edit_before_recalc.workspace_revision_id,
            after_recalc.workspace_revision_id
        );
        assert_eq!(before_edit.value_epoch, after_recalc.value_epoch);
        assert_eq!(
            exported_input_epoch(&after_snapshot, a_id),
            before_a_formula_version + 1
        );
        assert_eq!(
            after_snapshot
                .workspace_revision
                .structure_snapshot
                .snapshot_id(),
            before_snapshot
                .workspace_revision
                .structure_snapshot
                .snapshot_id()
        );
        assert!(
            result
                .invalidation_closure
                .records
                .get(&a_id)
                .is_some_and(|record| record
                    .reasons
                    .contains(&InvalidationReasonKind::StructuralRecalcOnly))
        );
        assert!(
            result
                .invalidation_closure
                .records
                .get(&b_id)
                .is_some_and(|record| record
                    .reasons
                    .contains(&InvalidationReasonKind::UpstreamPublication))
        );
        assert!(result.diagnostics.iter().any(|diagnostic| {
            diagnostic == &format!("edge_value_cache_bypass:{a_id}:ExplicitInvalidationSeed")
        }));
        assert!(result.diagnostics.iter().any(|diagnostic| {
            diagnostic == &format!("edge_value_cache_bypass:{b_id}:UpstreamPublication")
        }));
        assert!(result.diagnostics.iter().any(|diagnostic| {
            diagnostic == &format!("formula_edit_classification:{a_id}:same_dependencies")
        }));
        assert!(result.diagnostics.iter().any(|diagnostic| {
            diagnostic
                == &format!("formula_input_kind_transition:{a_id}:formula-text->formula-text")
        }));
        assert!(
            result
                .candidate_result
                .as_ref()
                .is_some_and(|candidate| candidate.dependency_shape_updates.is_empty())
        );
    }

    #[test]
    fn treecalc_context_edit_transaction_publishes_once_for_multiple_node_edits() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new("workspace:transaction"))
            .unwrap();
        let a_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("A", "1"))
            .unwrap();
        let b_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("B", "=A+1"))
            .unwrap();
        let c_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("C", "=B+1"))
            .unwrap();
        context.recalculate(&workspace_id).unwrap();
        let before_revision = context.workspace_revision(&workspace_id).unwrap();
        let before_view = context.workspace_view(&workspace_id).unwrap();
        assert_eq!(before_view.retained_workspace_revision_count, 4);
        assert!(before_view.workspace_revision_parent_id.is_some());
        assert!(
            before_view
                .workspace_revision_graph_entries
                .iter()
                .any(|entry| {
                    entry.revision_id == *before_revision.revision_id()
                        && entry.parent_revision_id == before_view.workspace_revision_parent_id
                })
        );

        let transaction = OxCalcTreeEditTransaction::new(workspace_id.clone())
            .with_edit(OxCalcTreeEdit::SetNodeInput {
                node_id: a_id,
                input: "5".to_string(),
            })
            .with_edit(OxCalcTreeEdit::SetNodeFormulaText {
                node_id: c_id,
                formula_text: "=A+3".to_string(),
            });
        let outcome = context.apply_edit_transaction(transaction).unwrap();

        assert_eq!(
            outcome.transaction_id.as_str(),
            "transaction:workspace:transaction:1"
        );
        assert_eq!(outcome.edit_count, 2);
        assert_eq!(
            outcome.edit_results,
            vec![OxCalcTreeEditResult::Applied, OxCalcTreeEditResult::Applied]
        );
        assert_ne!(
            outcome.workspace_revision_id,
            *before_revision.revision_id()
        );
        assert_eq!(
            outcome.predecessor_workspace_revision_id,
            *before_revision.revision_id()
        );
        assert_eq!(
            outcome.successor_workspace_revision_id,
            outcome.workspace_revision_id
        );
        let calculation = outcome
            .calculation
            .expect("transaction should recalculate once");
        assert_eq!(calculation.run_state, OxCalcTreeRunState::Published);
        assert_eq!(
            calculation
                .candidate_result
                .as_ref()
                .map(|candidate| candidate.candidate_result_id.as_str()),
            Some("candidate:workspace:transaction:2")
        );
        assert_eq!(
            calculation.published_values.get(&b_id),
            Some(&"6".to_string())
        );
        assert_eq!(
            calculation.published_values.get(&c_id),
            Some(&"8".to_string())
        );
        let view = context.workspace_view(&workspace_id).unwrap();
        assert_eq!(
            view.workspace_revision_parent_id,
            Some(before_revision.revision_id().clone())
        );
        assert_eq!(view.retained_workspace_revision_count, 5);
        assert!(view.workspace_revision_graph_entries.iter().any(|entry| {
            entry.revision_id == outcome.workspace_revision_id
                && entry.parent_revision_id == Some(before_revision.revision_id().clone())
        }));
        let revision_entry = view
            .workspace_revision_graph_entries
            .iter()
            .find(|entry| entry.revision_id == outcome.workspace_revision_id)
            .expect("successor revision should be retained");
        let transaction_summary = revision_entry
            .transaction_summary
            .as_ref()
            .expect("transaction revision should retain invalidation summary");
        assert_eq!(
            transaction_summary.transaction_id,
            outcome.transaction_id.to_string()
        );
        assert_eq!(
            transaction_summary.estimated_node_count,
            transaction_summary.invalidated_nodes.len()
        );
        assert!(
            transaction_summary
                .invalidated_nodes
                .iter()
                .any(|entry| entry.node_id == a_id)
        );
        assert!(
            transaction_summary
                .invalidated_nodes
                .iter()
                .all(|entry| !entry.reasons.is_empty())
        );
        assert_eq!(
            view.nodes
                .iter()
                .find(|node| node.node_id == a_id)
                .and_then(|node| node.value_text.as_deref()),
            Some("5")
        );
    }

    #[test]
    fn treecalc_context_candidate_evaluation_does_not_publish_workspace_state() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new("workspace:candidate"))
            .unwrap();
        let a_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("A", "1"))
            .unwrap();
        let b_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("B", "=A+1"))
            .unwrap();
        let initial = context.recalculate(&workspace_id).unwrap();
        assert_eq!(initial.run_state, OxCalcTreeRunState::Published);
        assert_eq!(initial.published_values.get(&b_id), Some(&"2".to_string()));

        let before_view = context.workspace_view(&workspace_id).unwrap();
        let before_b = context.node_view(&workspace_id, b_id).unwrap();
        let before_revision_id = before_view.workspace_revision_id.clone();
        let before_publication_snapshot_id = before_view.publication_snapshot_id.clone();
        let before_runtime_overlay_set_id = before_view.runtime_overlay_set_id.clone();
        let before_b_epoch = before_b.published_value_epoch;

        let candidate = context
            .open_candidate(OxCalcTreeOpenCandidateRequest::new(
                workspace_id.clone(),
                before_revision_id.clone(),
            ))
            .unwrap();
        assert_eq!(candidate.basis_revision_id, before_revision_id);
        assert_eq!(candidate.workspace_revision_id, before_revision_id);
        assert_eq!(candidate.run_state, Some(OxCalcTreeRunState::Published));

        context
            .apply_candidate_edit_transaction(
                &candidate.handle,
                OxCalcTreeEditTransaction::new(workspace_id.clone()).with_edit(
                    OxCalcTreeEdit::SetNodeInput {
                        node_id: a_id,
                        input: "5".to_string(),
                    },
                ),
            )
            .unwrap();
        let evaluated = context.evaluate_candidate(&candidate.handle).unwrap();
        assert_eq!(evaluated.run_state, Some(OxCalcTreeRunState::Published));
        let candidate_result = evaluated
            .calculation
            .as_ref()
            .expect("candidate evaluation should retain private calculation");
        assert_eq!(
            candidate_result.published_values.get(&b_id),
            Some(&"6".to_string())
        );

        let after_view = context.workspace_view(&workspace_id).unwrap();
        let after_b = context.node_view(&workspace_id, b_id).unwrap();
        assert_eq!(after_view.workspace_revision_id, before_revision_id);
        assert_eq!(
            after_view.publication_snapshot_id,
            before_publication_snapshot_id
        );
        assert_eq!(
            after_view.runtime_overlay_set_id,
            before_runtime_overlay_set_id
        );
        assert_eq!(after_b.value_text, Some("2".to_string()));
        assert_eq!(after_b.published_value_epoch, before_b_epoch);

        let discarded = context.discard_candidate(&candidate.handle).unwrap();
        assert_eq!(discarded.handle, candidate.handle);
        assert!(matches!(
            context.candidate_view(&discarded.handle),
            Err(OxCalcTreeContextError::UnknownCandidate { .. })
        ));
    }

    #[test]
    fn treecalc_context_candidate_commit_publishes_private_candidate_state() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new("workspace:candidate-commit"))
            .unwrap();
        let a_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("A", "1"))
            .unwrap();
        let b_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("B", "=A+1"))
            .unwrap();
        context.recalculate(&workspace_id).unwrap();
        let before_view = context.workspace_view(&workspace_id).unwrap();
        let before_revision_id = before_view.workspace_revision_id.clone();

        let candidate = context
            .open_candidate(OxCalcTreeOpenCandidateRequest::new(
                workspace_id.clone(),
                before_revision_id.clone(),
            ))
            .unwrap();
        let edited = context
            .apply_candidate_edit_transaction(
                &candidate.handle,
                OxCalcTreeEditTransaction::new(workspace_id.clone()).with_edit(
                    OxCalcTreeEdit::SetNodeInput {
                        node_id: a_id,
                        input: "5".to_string(),
                    },
                ),
            )
            .unwrap();
        let private_edit_entry = edited
            .workspace_revision_graph_entries
            .iter()
            .find(|entry| entry.revision_id == edited.workspace_revision_id)
            .expect("candidate edit revision should be retained");
        assert_eq!(
            private_edit_entry.transaction_id.as_deref(),
            Some("transaction:workspace:candidate-commit:1")
        );
        let private_summary = private_edit_entry
            .transaction_summary
            .as_ref()
            .expect("candidate private edit should carry planned invalidation summary");
        assert_eq!(
            private_summary.transaction_id,
            "transaction:workspace:candidate-commit:1"
        );
        assert_eq!(private_summary.estimated_node_count, 2);
        assert_eq!(
            private_summary
                .invalidated_nodes
                .iter()
                .map(|entry| entry.node_id)
                .collect::<Vec<_>>(),
            vec![a_id, b_id],
            "{private_summary:?}"
        );
        assert_eq!(
            private_summary.requires_rebind,
            Vec::<TreeNodeId>::new(),
            "{private_summary:?}"
        );
        context.evaluate_candidate(&candidate.handle).unwrap();

        let commit = context.commit_candidate(&candidate.handle).unwrap();
        assert_eq!(commit.handle, candidate.handle);
        assert_eq!(commit.workspace_id, workspace_id);
        assert_eq!(commit.basis_revision_id, before_revision_id);
        assert_eq!(commit.predecessor_workspace_revision_id, before_revision_id);
        assert_ne!(commit.successor_workspace_revision_id, before_revision_id);
        let committed_private_edit_entry = commit
            .workspace_revision_graph_entries
            .iter()
            .find(|entry| entry.revision_id == commit.successor_workspace_revision_id)
            .expect("candidate commit should expose promoted private revision entry");
        assert_eq!(
            committed_private_edit_entry.transaction_id.as_deref(),
            Some("transaction:workspace:candidate-commit:1")
        );
        assert_eq!(
            committed_private_edit_entry.transaction_summary,
            Some(private_summary.clone())
        );
        assert_eq!(
            commit
                .calculation
                .as_ref()
                .and_then(|calculation| calculation.published_values.get(&b_id)),
            Some(&"6".to_string())
        );

        let after_view = context.workspace_view(&workspace_id).unwrap();
        let after_b = context.node_view(&workspace_id, b_id).unwrap();
        assert_eq!(
            after_view.workspace_revision_id,
            commit.successor_workspace_revision_id
        );
        assert_eq!(after_b.value_text, Some("6".to_string()));
        assert!(matches!(
            context.candidate_view(&candidate.handle),
            Err(OxCalcTreeContextError::UnknownCandidate { .. })
        ));
    }

    #[test]
    fn treecalc_context_candidate_commit_rejects_when_basis_is_no_longer_current() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new("workspace:candidate-stale"))
            .unwrap();
        let a_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("A", "1"))
            .unwrap();
        let b_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("B", "=A+1"))
            .unwrap();
        context.recalculate(&workspace_id).unwrap();
        let basis_revision_id = context
            .workspace_view(&workspace_id)
            .unwrap()
            .workspace_revision_id;

        let candidate = context
            .open_candidate(OxCalcTreeOpenCandidateRequest::new(
                workspace_id.clone(),
                basis_revision_id.clone(),
            ))
            .unwrap();
        context
            .apply_candidate_edit_transaction(
                &candidate.handle,
                OxCalcTreeEditTransaction::new(workspace_id.clone()).with_edit(
                    OxCalcTreeEdit::SetNodeInput {
                        node_id: a_id,
                        input: "5".to_string(),
                    },
                ),
            )
            .unwrap();
        context.evaluate_candidate(&candidate.handle).unwrap();

        context
            .apply_edit_transaction(
                OxCalcTreeEditTransaction::new(workspace_id.clone()).with_edit(
                    OxCalcTreeEdit::SetNodeInput {
                        node_id: a_id,
                        input: "9".to_string(),
                    },
                ),
            )
            .unwrap();
        let current_revision_id = context
            .workspace_view(&workspace_id)
            .unwrap()
            .workspace_revision_id;

        let error = context.commit_candidate(&candidate.handle).unwrap_err();
        assert!(matches!(
            error,
            OxCalcTreeContextError::CandidateBasisNotCurrent { .. }
        ));
        assert_eq!(
            context
                .candidate_view(&candidate.handle)
                .unwrap()
                .basis_revision_id,
            basis_revision_id
        );
        assert_eq!(
            context.node_view(&workspace_id, b_id).unwrap().value_text,
            Some("10".to_string())
        );
        assert_ne!(current_revision_id, basis_revision_id);
    }

    #[test]
    fn treecalc_context_rejects_rebase_when_live_and_candidate_edit_same_node() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:candidate-rebase-conflict",
            ))
            .unwrap();
        let a_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("A", "1"))
            .unwrap();
        let b_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("B", "=A+1"))
            .unwrap();
        context.recalculate(&workspace_id).unwrap();
        let basis_revision_id = context
            .workspace_view(&workspace_id)
            .unwrap()
            .workspace_revision_id;

        let candidate = context
            .open_candidate(OxCalcTreeOpenCandidateRequest::new(
                workspace_id.clone(),
                basis_revision_id.clone(),
            ))
            .unwrap();
        context
            .apply_candidate_edit_transaction(
                &candidate.handle,
                OxCalcTreeEditTransaction::new(workspace_id.clone()).with_edit(
                    OxCalcTreeEdit::SetNodeInput {
                        node_id: a_id,
                        input: "5".to_string(),
                    },
                ),
            )
            .unwrap();

        context
            .apply_edit_transaction(
                OxCalcTreeEditTransaction::new(workspace_id.clone()).with_edit(
                    OxCalcTreeEdit::SetNodeInput {
                        node_id: a_id,
                        input: "2".to_string(),
                    },
                ),
            )
            .unwrap();
        let current_revision_id = context
            .workspace_view(&workspace_id)
            .unwrap()
            .workspace_revision_id;
        assert_ne!(current_revision_id, basis_revision_id);
        assert!(matches!(
            context.commit_candidate(&candidate.handle),
            Err(OxCalcTreeContextError::CandidateBasisNotCurrent { .. })
        ));

        let error = context
            .rebase_candidate_to_current_revision(&candidate.handle)
            .unwrap_err();
        let OxCalcTreeContextError::CandidateRebaseConflict {
            handle,
            basis_revision_id: conflict_basis,
            current_revision_id: conflict_current,
            overlapping_nodes,
            report,
        } = error
        else {
            panic!("expected candidate rebase conflict");
        };
        assert_eq!(handle, candidate.handle);
        assert_eq!(conflict_basis, basis_revision_id);
        assert_eq!(conflict_current, current_revision_id);
        assert_eq!(overlapping_nodes, vec![a_id]);
        assert_eq!(report.overlapping_nodes, vec![a_id]);
        assert!(report.candidate_touched_nodes.contains(&a_id));
        assert!(report.live_touched_nodes.contains(&a_id));
        assert_eq!(
            context
                .candidate_view(&candidate.handle)
                .unwrap()
                .basis_revision_id,
            basis_revision_id
        );
        assert_eq!(
            context.node_view(&workspace_id, b_id).unwrap().value_text,
            Some("3".to_string()),
            "failed rebase must not publish or discard workspace state"
        );
    }

    #[test]
    fn treecalc_context_rebases_candidate_add_when_live_only_edits_parent_content() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:candidate-rebase-add-parent-content",
            ))
            .unwrap();
        let parent_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("Parent", "1"))
            .unwrap();
        context.recalculate(&workspace_id).unwrap();
        let basis_revision_id = context
            .workspace_view(&workspace_id)
            .unwrap()
            .workspace_revision_id;

        let candidate = context
            .open_candidate(OxCalcTreeOpenCandidateRequest::new(
                workspace_id.clone(),
                basis_revision_id.clone(),
            ))
            .unwrap();
        let candidate_child_id = context.reserve_node_id();
        context
            .apply_candidate_edit_transaction(
                &candidate.handle,
                OxCalcTreeEditTransaction::new(workspace_id.clone()).with_edit(
                    OxCalcTreeEdit::AddNode {
                        request: OxCalcTreeNodeCreate::new("CandidateChild", "7")
                            .under(parent_id)
                            .with_reserved_node_id(candidate_child_id),
                    },
                ),
            )
            .unwrap();

        context
            .apply_edit_transaction(
                OxCalcTreeEditTransaction::new(workspace_id.clone()).with_edit(
                    OxCalcTreeEdit::SetNodeInput {
                        node_id: parent_id,
                        input: "2".to_string(),
                    },
                ),
            )
            .unwrap();
        let current_revision_id = context
            .workspace_view(&workspace_id)
            .unwrap()
            .workspace_revision_id;
        assert_ne!(current_revision_id, basis_revision_id);

        let rebased = context
            .rebase_candidate_to_current_revision(&candidate.handle)
            .unwrap();
        assert_eq!(rebased.basis_revision_id, current_revision_id);
        assert!(
            rebased
                .nodes
                .iter()
                .any(|node| node.node_id == candidate_child_id)
        );
        assert_eq!(
            context
                .node_view(&workspace_id, parent_id)
                .unwrap()
                .value_text,
            Some("2".to_string())
        );
        assert!(
            context
                .node_view(&workspace_id, candidate_child_id)
                .is_err(),
            "rebase must not publish candidate-only structure"
        );

        let commit = context.commit_candidate(&candidate.handle).unwrap();
        assert_eq!(commit.basis_revision_id, current_revision_id);
        assert_eq!(
            context
                .node_view(&workspace_id, candidate_child_id)
                .unwrap()
                .formula_text,
            "7"
        );
        assert_eq!(
            context
                .node_view(&workspace_id, parent_id)
                .unwrap()
                .value_text,
            Some("2".to_string())
        );
    }

    #[test]
    fn treecalc_context_rebases_candidate_rename_when_live_edits_same_node_content() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:candidate-rebase-rename-content",
            ))
            .unwrap();
        let node_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("Source", "1"))
            .unwrap();
        context.recalculate(&workspace_id).unwrap();
        let basis_revision_id = context
            .workspace_view(&workspace_id)
            .unwrap()
            .workspace_revision_id;

        let candidate = context
            .open_candidate(OxCalcTreeOpenCandidateRequest::new(
                workspace_id.clone(),
                basis_revision_id.clone(),
            ))
            .unwrap();
        context
            .apply_candidate_edit_transaction(
                &candidate.handle,
                OxCalcTreeEditTransaction::new(workspace_id.clone()).with_edit(
                    OxCalcTreeEdit::RenameNode {
                        node_id,
                        new_symbol: "Renamed".to_string(),
                    },
                ),
            )
            .unwrap();

        context
            .apply_edit_transaction(
                OxCalcTreeEditTransaction::new(workspace_id.clone()).with_edit(
                    OxCalcTreeEdit::SetNodeInput {
                        node_id,
                        input: "2".to_string(),
                    },
                ),
            )
            .unwrap();
        let current_revision_id = context
            .workspace_view(&workspace_id)
            .unwrap()
            .workspace_revision_id;

        let rebased = context
            .rebase_candidate_to_current_revision(&candidate.handle)
            .unwrap();
        assert_eq!(rebased.basis_revision_id, current_revision_id);
        assert!(
            rebased
                .nodes
                .iter()
                .any(|node| node.node_id == node_id && node.symbol == "Renamed")
        );
        assert_eq!(
            context
                .node_view(&workspace_id, node_id)
                .unwrap()
                .display_path,
            "Root/Source"
        );
        assert_eq!(
            context
                .node_view(&workspace_id, node_id)
                .unwrap()
                .value_text,
            Some("2".to_string())
        );

        context.commit_candidate(&candidate.handle).unwrap();
        let committed = context.node_view(&workspace_id, node_id).unwrap();
        assert_eq!(committed.display_path, "Root/Renamed");
        assert_eq!(committed.value_text, Some("2".to_string()));
    }

    #[test]
    fn treecalc_context_rebases_candidate_move_when_live_edits_moved_node_content() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:candidate-rebase-move-content",
            ))
            .unwrap();
        let source_parent_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("SourceParent", ""))
            .unwrap();
        let destination_parent_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("DestinationParent", ""),
            )
            .unwrap();
        let moved_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Moved", "1").under(source_parent_id),
            )
            .unwrap();
        context.recalculate(&workspace_id).unwrap();
        let basis_revision_id = context
            .workspace_view(&workspace_id)
            .unwrap()
            .workspace_revision_id;

        let candidate = context
            .open_candidate(OxCalcTreeOpenCandidateRequest::new(
                workspace_id.clone(),
                basis_revision_id.clone(),
            ))
            .unwrap();
        context
            .apply_candidate_edit_transaction(
                &candidate.handle,
                OxCalcTreeEditTransaction::new(workspace_id.clone()).with_edit(
                    OxCalcTreeEdit::MoveNode {
                        node_id: moved_id,
                        new_parent_id: destination_parent_id,
                        new_index: None,
                    },
                ),
            )
            .unwrap();

        context
            .apply_edit_transaction(
                OxCalcTreeEditTransaction::new(workspace_id.clone()).with_edit(
                    OxCalcTreeEdit::SetNodeInput {
                        node_id: moved_id,
                        input: "2".to_string(),
                    },
                ),
            )
            .unwrap();
        let current_revision_id = context
            .workspace_view(&workspace_id)
            .unwrap()
            .workspace_revision_id;

        let rebased = context
            .rebase_candidate_to_current_revision(&candidate.handle)
            .unwrap();
        assert_eq!(rebased.basis_revision_id, current_revision_id);
        assert!(rebased.nodes.iter().any(|node| node.node_id == moved_id
            && node.display_path == "Root/DestinationParent/Moved"
            && node.formula_text == "2"));
        assert_eq!(
            context
                .node_view(&workspace_id, moved_id)
                .unwrap()
                .display_path,
            "Root/SourceParent/Moved"
        );

        context.commit_candidate(&candidate.handle).unwrap();
        let committed = context.node_view(&workspace_id, moved_id).unwrap();
        assert_eq!(committed.display_path, "Root/DestinationParent/Moved");
        assert_eq!(committed.value_text, Some("2".to_string()));
    }

    #[test]
    fn treecalc_context_rebases_multi_edit_candidate_over_live_content_edits() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:candidate-rebase-multi-structural-content",
            ))
            .unwrap();
        let root_id = context.workspace_view(&workspace_id).unwrap().root_node_id;
        let source_parent_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("SourceParent", "10").under(root_id),
            )
            .unwrap();
        let destination_parent_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("DestinationParent", "").under(root_id),
            )
            .unwrap();
        let renamed_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("RenameMe", "1").under(source_parent_id),
            )
            .unwrap();
        let moved_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("MoveMe", "2").under(source_parent_id),
            )
            .unwrap();
        context.recalculate(&workspace_id).unwrap();
        let basis_revision_id = context
            .workspace_view(&workspace_id)
            .unwrap()
            .workspace_revision_id;

        let candidate = context
            .open_candidate(OxCalcTreeOpenCandidateRequest::new(
                workspace_id.clone(),
                basis_revision_id,
            ))
            .unwrap();
        context
            .apply_candidate_edit_transaction(
                &candidate.handle,
                OxCalcTreeEditTransaction::new(workspace_id.clone())
                    .with_edit(OxCalcTreeEdit::RenameNode {
                        node_id: renamed_id,
                        new_symbol: "Renamed".to_string(),
                    })
                    .with_edit(OxCalcTreeEdit::MoveNode {
                        node_id: moved_id,
                        new_parent_id: destination_parent_id,
                        new_index: None,
                    })
                    .with_edit(OxCalcTreeEdit::AddNode {
                        request: OxCalcTreeNodeCreate::new("CandidateOnly", "5")
                            .under(source_parent_id),
                    }),
            )
            .unwrap();

        context
            .apply_edit_transaction(
                OxCalcTreeEditTransaction::new(workspace_id.clone())
                    .with_edit(OxCalcTreeEdit::SetNodeInput {
                        node_id: source_parent_id,
                        input: "11".to_string(),
                    })
                    .with_edit(OxCalcTreeEdit::SetNodeInput {
                        node_id: renamed_id,
                        input: "3".to_string(),
                    })
                    .with_edit(OxCalcTreeEdit::SetNodeInput {
                        node_id: moved_id,
                        input: "4".to_string(),
                    }),
            )
            .unwrap();
        let current_revision_id = context
            .workspace_view(&workspace_id)
            .unwrap()
            .workspace_revision_id;

        let rebased = context
            .rebase_candidate_to_current_revision(&candidate.handle)
            .unwrap();
        assert_eq!(rebased.basis_revision_id, current_revision_id);
        assert!(rebased.nodes.iter().any(|node| node.node_id == renamed_id
            && node.display_path == "Root/SourceParent/Renamed"
            && node.formula_text == "3"));
        assert!(rebased.nodes.iter().any(|node| node.node_id == moved_id
            && node.display_path == "Root/DestinationParent/MoveMe"
            && node.formula_text == "4"));
        assert!(rebased.nodes.iter().any(|node| {
            node.display_path == "Root/SourceParent/CandidateOnly" && node.formula_text == "5"
        }));
        assert_eq!(
            context
                .node_view(&workspace_id, renamed_id)
                .unwrap()
                .display_path,
            "Root/SourceParent/RenameMe"
        );
        assert!(
            context
                .workspace_view(&workspace_id)
                .unwrap()
                .nodes
                .iter()
                .all(|node| node.display_path != "Root/SourceParent/CandidateOnly")
        );

        context.commit_candidate(&candidate.handle).unwrap();
        let committed_renamed = context.node_view(&workspace_id, renamed_id).unwrap();
        let committed_moved = context.node_view(&workspace_id, moved_id).unwrap();
        assert_eq!(committed_renamed.display_path, "Root/SourceParent/Renamed");
        assert_eq!(committed_renamed.value_text, Some("3".to_string()));
        assert_eq!(
            committed_moved.display_path,
            "Root/DestinationParent/MoveMe"
        );
        assert_eq!(committed_moved.value_text, Some("4".to_string()));
        assert!(
            context
                .workspace_view(&workspace_id)
                .unwrap()
                .nodes
                .iter()
                .any(
                    |node| node.display_path == "Root/SourceParent/CandidateOnly"
                        && node.value_text == Some("5".to_string())
                )
        );
    }

    #[test]
    fn treecalc_context_rebases_candidate_rename_over_live_move_same_node() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:candidate-rebase-rename-over-move",
            ))
            .unwrap();
        let source_parent_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("Source", ""))
            .unwrap();
        let destination_parent_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("Destination", ""))
            .unwrap();
        let node_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Original", "1").under(source_parent_id),
            )
            .unwrap();
        context.recalculate(&workspace_id).unwrap();
        let basis_revision_id = context
            .workspace_view(&workspace_id)
            .unwrap()
            .workspace_revision_id;

        let candidate = context
            .open_candidate(OxCalcTreeOpenCandidateRequest::new(
                workspace_id.clone(),
                basis_revision_id,
            ))
            .unwrap();
        context
            .apply_candidate_edit_transaction(
                &candidate.handle,
                OxCalcTreeEditTransaction::new(workspace_id.clone()).with_edit(
                    OxCalcTreeEdit::RenameNode {
                        node_id,
                        new_symbol: "Renamed".to_string(),
                    },
                ),
            )
            .unwrap();

        context
            .apply_edit_transaction(
                OxCalcTreeEditTransaction::new(workspace_id.clone()).with_edit(
                    OxCalcTreeEdit::MoveNode {
                        node_id,
                        new_parent_id: destination_parent_id,
                        new_index: None,
                    },
                ),
            )
            .unwrap();
        let current_revision_id = context
            .workspace_view(&workspace_id)
            .unwrap()
            .workspace_revision_id;

        let rebased = context
            .rebase_candidate_to_current_revision(&candidate.handle)
            .unwrap();
        assert_eq!(rebased.basis_revision_id, current_revision_id);
        assert!(
            rebased
                .nodes
                .iter()
                .any(|node| node.node_id == node_id
                    && node.display_path == "Root/Destination/Renamed")
        );
        assert_eq!(
            context
                .node_view(&workspace_id, node_id)
                .unwrap()
                .display_path,
            "Root/Destination/Original"
        );

        context.commit_candidate(&candidate.handle).unwrap();
        assert_eq!(
            context
                .node_view(&workspace_id, node_id)
                .unwrap()
                .display_path,
            "Root/Destination/Renamed"
        );
    }

    #[test]
    fn treecalc_context_rebases_candidate_move_over_live_rename_same_node() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:candidate-rebase-move-over-rename",
            ))
            .unwrap();
        let source_parent_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("Source", ""))
            .unwrap();
        let destination_parent_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("Destination", ""))
            .unwrap();
        let node_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Original", "1").under(source_parent_id),
            )
            .unwrap();
        context.recalculate(&workspace_id).unwrap();
        let basis_revision_id = context
            .workspace_view(&workspace_id)
            .unwrap()
            .workspace_revision_id;

        let candidate = context
            .open_candidate(OxCalcTreeOpenCandidateRequest::new(
                workspace_id.clone(),
                basis_revision_id,
            ))
            .unwrap();
        context
            .apply_candidate_edit_transaction(
                &candidate.handle,
                OxCalcTreeEditTransaction::new(workspace_id.clone()).with_edit(
                    OxCalcTreeEdit::MoveNode {
                        node_id,
                        new_parent_id: destination_parent_id,
                        new_index: None,
                    },
                ),
            )
            .unwrap();

        context
            .apply_edit_transaction(
                OxCalcTreeEditTransaction::new(workspace_id.clone()).with_edit(
                    OxCalcTreeEdit::RenameNode {
                        node_id,
                        new_symbol: "Renamed".to_string(),
                    },
                ),
            )
            .unwrap();
        let current_revision_id = context
            .workspace_view(&workspace_id)
            .unwrap()
            .workspace_revision_id;

        let rebased = context
            .rebase_candidate_to_current_revision(&candidate.handle)
            .unwrap();
        assert_eq!(rebased.basis_revision_id, current_revision_id);
        assert!(
            rebased
                .nodes
                .iter()
                .any(|node| node.node_id == node_id
                    && node.display_path == "Root/Destination/Renamed")
        );
        assert_eq!(
            context
                .node_view(&workspace_id, node_id)
                .unwrap()
                .display_path,
            "Root/Source/Renamed"
        );

        context.commit_candidate(&candidate.handle).unwrap();
        assert_eq!(
            context
                .node_view(&workspace_id, node_id)
                .unwrap()
                .display_path,
            "Root/Destination/Renamed"
        );
    }

    #[test]
    fn treecalc_context_rejects_rebase_when_live_and_candidate_rename_same_node() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:candidate-rebase-rename-conflict",
            ))
            .unwrap();
        let node_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("Original", "1"))
            .unwrap();
        context.recalculate(&workspace_id).unwrap();
        let basis_revision_id = context
            .workspace_view(&workspace_id)
            .unwrap()
            .workspace_revision_id;

        let candidate = context
            .open_candidate(OxCalcTreeOpenCandidateRequest::new(
                workspace_id.clone(),
                basis_revision_id,
            ))
            .unwrap();
        context
            .apply_candidate_edit_transaction(
                &candidate.handle,
                OxCalcTreeEditTransaction::new(workspace_id.clone()).with_edit(
                    OxCalcTreeEdit::RenameNode {
                        node_id,
                        new_symbol: "CandidateName".to_string(),
                    },
                ),
            )
            .unwrap();

        context
            .apply_edit_transaction(
                OxCalcTreeEditTransaction::new(workspace_id.clone()).with_edit(
                    OxCalcTreeEdit::RenameNode {
                        node_id,
                        new_symbol: "LiveName".to_string(),
                    },
                ),
            )
            .unwrap();
        let error = context
            .rebase_candidate_to_current_revision(&candidate.handle)
            .unwrap_err();
        let OxCalcTreeContextError::CandidateRebaseConflict {
            overlapping_nodes,
            report,
            ..
        } = error
        else {
            panic!("expected candidate rebase conflict");
        };
        assert!(overlapping_nodes.contains(&node_id));
        assert!(report.candidate_touched_nodes.contains(&node_id));
        assert!(report.live_touched_nodes.contains(&node_id));
    }

    #[test]
    fn treecalc_context_rebases_candidate_rename_over_live_sibling_add_without_name_collision() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:candidate-rebase-rename-over-add",
            ))
            .unwrap();
        let parent_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("Parent", ""))
            .unwrap();
        let node_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Original", "1").under(parent_id),
            )
            .unwrap();
        context.recalculate(&workspace_id).unwrap();
        let basis_revision_id = context
            .workspace_view(&workspace_id)
            .unwrap()
            .workspace_revision_id;

        let candidate = context
            .open_candidate(OxCalcTreeOpenCandidateRequest::new(
                workspace_id.clone(),
                basis_revision_id,
            ))
            .unwrap();
        context
            .apply_candidate_edit_transaction(
                &candidate.handle,
                OxCalcTreeEditTransaction::new(workspace_id.clone()).with_edit(
                    OxCalcTreeEdit::RenameNode {
                        node_id,
                        new_symbol: "Renamed".to_string(),
                    },
                ),
            )
            .unwrap();

        let live_added_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("LiveAdded", "2").under(parent_id),
            )
            .unwrap();
        let current_revision_id = context
            .workspace_view(&workspace_id)
            .unwrap()
            .workspace_revision_id;

        let rebased = context
            .rebase_candidate_to_current_revision(&candidate.handle)
            .unwrap();
        assert_eq!(rebased.basis_revision_id, current_revision_id);
        assert!(
            rebased
                .nodes
                .iter()
                .any(|node| node.node_id == node_id && node.display_path == "Root/Parent/Renamed")
        );
        assert!(
            rebased
                .nodes
                .iter()
                .any(|node| node.node_id == live_added_id
                    && node.display_path == "Root/Parent/LiveAdded")
        );
        assert_eq!(
            context
                .node_view(&workspace_id, node_id)
                .unwrap()
                .display_path,
            "Root/Parent/Original"
        );

        context.commit_candidate(&candidate.handle).unwrap();
        assert_eq!(
            context
                .node_view(&workspace_id, node_id)
                .unwrap()
                .display_path,
            "Root/Parent/Renamed"
        );
        assert_eq!(
            context
                .node_view(&workspace_id, live_added_id)
                .unwrap()
                .display_path,
            "Root/Parent/LiveAdded"
        );
    }

    #[test]
    fn treecalc_context_rejects_candidate_rename_over_live_sibling_add_name_collision() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:candidate-rebase-rename-add-collision",
            ))
            .unwrap();
        let parent_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("Parent", ""))
            .unwrap();
        let node_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Original", "1").under(parent_id),
            )
            .unwrap();
        context.recalculate(&workspace_id).unwrap();
        let basis_revision_id = context
            .workspace_view(&workspace_id)
            .unwrap()
            .workspace_revision_id;

        let candidate = context
            .open_candidate(OxCalcTreeOpenCandidateRequest::new(
                workspace_id.clone(),
                basis_revision_id,
            ))
            .unwrap();
        context
            .apply_candidate_edit_transaction(
                &candidate.handle,
                OxCalcTreeEditTransaction::new(workspace_id.clone()).with_edit(
                    OxCalcTreeEdit::RenameNode {
                        node_id,
                        new_symbol: "LiveAdded".to_string(),
                    },
                ),
            )
            .unwrap();

        context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("LiveAdded", "2").under(parent_id),
            )
            .unwrap();
        let error = context
            .rebase_candidate_to_current_revision(&candidate.handle)
            .unwrap_err();
        let OxCalcTreeContextError::CandidateRebaseConflict {
            overlapping_nodes,
            report,
            ..
        } = error
        else {
            panic!("expected candidate rebase conflict");
        };
        assert_eq!(
            report.kind,
            OxCalcTreeCandidateRebaseConflictKind::ReplayValidationRejected
        );
        assert!(overlapping_nodes.contains(&parent_id));
        assert!(report.candidate_touched_nodes.contains(&parent_id));
        assert!(report.live_touched_nodes.contains(&parent_id));
        assert_eq!(
            context
                .node_view(&workspace_id, node_id)
                .unwrap()
                .display_path,
            "Root/Parent/Original"
        );
    }

    #[test]
    fn treecalc_context_rebases_candidate_rename_over_live_sibling_reorder() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:candidate-rebase-rename-over-reorder",
            ))
            .unwrap();
        let parent_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("Parent", ""))
            .unwrap();
        let node_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Original", "1").under(parent_id),
            )
            .unwrap();
        let reordered_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Reordered", "2").under(parent_id),
            )
            .unwrap();
        context.recalculate(&workspace_id).unwrap();
        let basis_revision_id = context
            .workspace_view(&workspace_id)
            .unwrap()
            .workspace_revision_id;

        let candidate = context
            .open_candidate(OxCalcTreeOpenCandidateRequest::new(
                workspace_id.clone(),
                basis_revision_id,
            ))
            .unwrap();
        context
            .apply_candidate_edit_transaction(
                &candidate.handle,
                OxCalcTreeEditTransaction::new(workspace_id.clone()).with_edit(
                    OxCalcTreeEdit::RenameNode {
                        node_id,
                        new_symbol: "Renamed".to_string(),
                    },
                ),
            )
            .unwrap();

        context
            .apply_edit_transaction(
                OxCalcTreeEditTransaction::new(workspace_id.clone()).with_edit(
                    OxCalcTreeEdit::ReorderNode {
                        node_id: reordered_id,
                        new_index: 0,
                    },
                ),
            )
            .unwrap();
        let current_revision_id = context
            .workspace_view(&workspace_id)
            .unwrap()
            .workspace_revision_id;

        let rebased = context
            .rebase_candidate_to_current_revision(&candidate.handle)
            .unwrap();
        assert_eq!(rebased.basis_revision_id, current_revision_id);
        assert!(
            rebased
                .nodes
                .iter()
                .any(|node| node.node_id == node_id && node.display_path == "Root/Parent/Renamed")
        );
        assert_eq!(
            context
                .node_view(&workspace_id, node_id)
                .unwrap()
                .display_path,
            "Root/Parent/Original"
        );

        context.commit_candidate(&candidate.handle).unwrap();
        assert_eq!(
            context
                .node_view(&workspace_id, node_id)
                .unwrap()
                .display_path,
            "Root/Parent/Renamed"
        );
        assert_eq!(
            context
                .node_view(&workspace_id, reordered_id)
                .unwrap()
                .display_path,
            "Root/Parent/Reordered"
        );
    }

    #[test]
    fn treecalc_context_rebases_candidate_reorder_over_live_sibling_rename() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:candidate-rebase-reorder-over-rename",
            ))
            .unwrap();
        let parent_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("Parent", ""))
            .unwrap();
        let node_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Original", "1").under(parent_id),
            )
            .unwrap();
        let renamed_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("RenameMe", "2").under(parent_id),
            )
            .unwrap();
        context.recalculate(&workspace_id).unwrap();
        let basis_revision_id = context
            .workspace_view(&workspace_id)
            .unwrap()
            .workspace_revision_id;

        let candidate = context
            .open_candidate(OxCalcTreeOpenCandidateRequest::new(
                workspace_id.clone(),
                basis_revision_id,
            ))
            .unwrap();
        context
            .apply_candidate_edit_transaction(
                &candidate.handle,
                OxCalcTreeEditTransaction::new(workspace_id.clone()).with_edit(
                    OxCalcTreeEdit::ReorderNode {
                        node_id,
                        new_index: 1,
                    },
                ),
            )
            .unwrap();

        context
            .apply_edit_transaction(
                OxCalcTreeEditTransaction::new(workspace_id.clone()).with_edit(
                    OxCalcTreeEdit::RenameNode {
                        node_id: renamed_id,
                        new_symbol: "Renamed".to_string(),
                    },
                ),
            )
            .unwrap();
        let current_revision_id = context
            .workspace_view(&workspace_id)
            .unwrap()
            .workspace_revision_id;

        let rebased = context
            .rebase_candidate_to_current_revision(&candidate.handle)
            .unwrap();
        assert_eq!(rebased.basis_revision_id, current_revision_id);
        assert!(
            rebased.nodes.iter().any(
                |node| node.node_id == renamed_id && node.display_path == "Root/Parent/Renamed"
            )
        );
        assert_eq!(
            context
                .node_view(&workspace_id, renamed_id)
                .unwrap()
                .display_path,
            "Root/Parent/Renamed"
        );

        context.commit_candidate(&candidate.handle).unwrap();
        assert_eq!(
            context
                .node_view(&workspace_id, node_id)
                .unwrap()
                .display_path,
            "Root/Parent/Original"
        );
        assert_eq!(
            context
                .node_view(&workspace_id, renamed_id)
                .unwrap()
                .display_path,
            "Root/Parent/Renamed"
        );
    }

    #[test]
    fn treecalc_context_rebases_candidate_add_over_live_sibling_delete() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:candidate-rebase-add-over-delete",
            ))
            .unwrap();
        let parent_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("Parent", ""))
            .unwrap();
        let deleted_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("DeleteMe", "1").under(parent_id),
            )
            .unwrap();
        context.recalculate(&workspace_id).unwrap();
        let basis_revision_id = context
            .workspace_view(&workspace_id)
            .unwrap()
            .workspace_revision_id;

        let candidate = context
            .open_candidate(OxCalcTreeOpenCandidateRequest::new(
                workspace_id.clone(),
                basis_revision_id,
            ))
            .unwrap();
        context
            .apply_candidate_edit_transaction(
                &candidate.handle,
                OxCalcTreeEditTransaction::new(workspace_id.clone()).with_edit(
                    OxCalcTreeEdit::AddNode {
                        request: OxCalcTreeNodeCreate::new("CandidateAdded", "2").under(parent_id),
                    },
                ),
            )
            .unwrap();

        context.delete_node(&workspace_id, deleted_id).unwrap();
        let current_revision_id = context
            .workspace_view(&workspace_id)
            .unwrap()
            .workspace_revision_id;

        let rebased = context
            .rebase_candidate_to_current_revision(&candidate.handle)
            .unwrap();
        assert_eq!(rebased.basis_revision_id, current_revision_id);
        assert!(rebased.nodes.iter().any(|node| {
            node.display_path == "Root/Parent/CandidateAdded" && node.formula_text == "2"
        }));
        assert!(
            context.node_view(&workspace_id, deleted_id).is_err(),
            "live deleted sibling should stay deleted"
        );

        context.commit_candidate(&candidate.handle).unwrap();
        assert!(
            context.node_view(&workspace_id, deleted_id).is_err(),
            "commit must not resurrect live deleted sibling"
        );
        assert!(
            context
                .workspace_view(&workspace_id)
                .unwrap()
                .nodes
                .iter()
                .any(|node| node.display_path == "Root/Parent/CandidateAdded"
                    && node.formula_text == "2")
        );
    }

    #[test]
    fn treecalc_context_rebases_candidate_delete_over_live_sibling_add() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:candidate-rebase-delete-over-add",
            ))
            .unwrap();
        let parent_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("Parent", ""))
            .unwrap();
        let deleted_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("DeleteMe", "1").under(parent_id),
            )
            .unwrap();
        context.recalculate(&workspace_id).unwrap();
        let basis_revision_id = context
            .workspace_view(&workspace_id)
            .unwrap()
            .workspace_revision_id;

        let candidate = context
            .open_candidate(OxCalcTreeOpenCandidateRequest::new(
                workspace_id.clone(),
                basis_revision_id,
            ))
            .unwrap();
        context
            .apply_candidate_edit_transaction(
                &candidate.handle,
                OxCalcTreeEditTransaction::new(workspace_id.clone()).with_edit(
                    OxCalcTreeEdit::DeleteNode {
                        node_id: deleted_id,
                    },
                ),
            )
            .unwrap();

        let live_added_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("LiveAdded", "2").under(parent_id),
            )
            .unwrap();
        let current_revision_id = context
            .workspace_view(&workspace_id)
            .unwrap()
            .workspace_revision_id;

        let rebased = context
            .rebase_candidate_to_current_revision(&candidate.handle)
            .unwrap();
        assert_eq!(rebased.basis_revision_id, current_revision_id);
        assert!(
            !rebased.nodes.iter().any(|node| node.node_id == deleted_id),
            "candidate-deleted sibling should stay absent from rebased candidate view"
        );
        assert!(
            rebased
                .nodes
                .iter()
                .any(|node| node.node_id == live_added_id
                    && node.display_path == "Root/Parent/LiveAdded")
        );
        assert_eq!(
            context
                .node_view(&workspace_id, live_added_id)
                .unwrap()
                .display_path,
            "Root/Parent/LiveAdded"
        );

        context.commit_candidate(&candidate.handle).unwrap();
        assert!(
            context.node_view(&workspace_id, deleted_id).is_err(),
            "commit should apply candidate delete"
        );
        assert_eq!(
            context
                .node_view(&workspace_id, live_added_id)
                .unwrap()
                .display_path,
            "Root/Parent/LiveAdded"
        );
    }

    #[test]
    fn treecalc_context_rebases_candidate_add_over_live_sibling_reorder() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:candidate-rebase-add-over-reorder",
            ))
            .unwrap();
        let parent_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("Parent", ""))
            .unwrap();
        let first_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("First", "1").under(parent_id),
            )
            .unwrap();
        let second_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Second", "2").under(parent_id),
            )
            .unwrap();
        context.recalculate(&workspace_id).unwrap();
        let basis_revision_id = context
            .workspace_view(&workspace_id)
            .unwrap()
            .workspace_revision_id;

        let candidate = context
            .open_candidate(OxCalcTreeOpenCandidateRequest::new(
                workspace_id.clone(),
                basis_revision_id,
            ))
            .unwrap();
        context
            .apply_candidate_edit_transaction(
                &candidate.handle,
                OxCalcTreeEditTransaction::new(workspace_id.clone()).with_edit(
                    OxCalcTreeEdit::AddNode {
                        request: OxCalcTreeNodeCreate::new("CandidateAdded", "3").under(parent_id),
                    },
                ),
            )
            .unwrap();

        context
            .reorder_node(&workspace_id, second_id, 0)
            .expect("live reorder should succeed");
        let current_revision_id = context
            .workspace_view(&workspace_id)
            .unwrap()
            .workspace_revision_id;

        let rebased = context
            .rebase_candidate_to_current_revision(&candidate.handle)
            .unwrap();
        assert_eq!(rebased.basis_revision_id, current_revision_id);
        let sibling_paths = child_display_paths_for_state(
            &context
                .candidates
                .get(&candidate.handle)
                .expect("rebased candidate should be retained")
                .workspace_state,
            parent_id,
        );
        assert_eq!(
            sibling_paths,
            vec![
                "Root/Parent/Second".to_string(),
                "Root/Parent/First".to_string(),
                "Root/Parent/CandidateAdded".to_string()
            ]
        );
        assert_eq!(
            context
                .node_view(&workspace_id, first_id)
                .unwrap()
                .display_path,
            "Root/Parent/First"
        );

        context.commit_candidate(&candidate.handle).unwrap();
        let committed_paths =
            child_display_paths_for_state(context.workspace(&workspace_id).unwrap(), parent_id);
        assert_eq!(
            committed_paths,
            vec![
                "Root/Parent/Second".to_string(),
                "Root/Parent/First".to_string(),
                "Root/Parent/CandidateAdded".to_string()
            ]
        );
    }

    #[test]
    fn treecalc_context_rebases_candidate_reorder_over_live_sibling_add() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:candidate-rebase-reorder-over-add",
            ))
            .unwrap();
        let parent_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("Parent", ""))
            .unwrap();
        let first_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("First", "1").under(parent_id),
            )
            .unwrap();
        let second_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Second", "2").under(parent_id),
            )
            .unwrap();
        context.recalculate(&workspace_id).unwrap();
        let basis_revision_id = context
            .workspace_view(&workspace_id)
            .unwrap()
            .workspace_revision_id;

        let candidate = context
            .open_candidate(OxCalcTreeOpenCandidateRequest::new(
                workspace_id.clone(),
                basis_revision_id,
            ))
            .unwrap();
        context
            .apply_candidate_edit_transaction(
                &candidate.handle,
                OxCalcTreeEditTransaction::new(workspace_id.clone()).with_edit(
                    OxCalcTreeEdit::ReorderNode {
                        node_id: second_id,
                        new_index: 0,
                    },
                ),
            )
            .unwrap();

        context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("LiveAdded", "3").under(parent_id),
            )
            .unwrap();
        let current_revision_id = context
            .workspace_view(&workspace_id)
            .unwrap()
            .workspace_revision_id;

        let rebased = context
            .rebase_candidate_to_current_revision(&candidate.handle)
            .unwrap();
        assert_eq!(rebased.basis_revision_id, current_revision_id);
        let sibling_paths = child_display_paths_for_state(
            &context
                .candidates
                .get(&candidate.handle)
                .expect("rebased candidate should be retained")
                .workspace_state,
            parent_id,
        );
        assert_eq!(
            sibling_paths,
            vec![
                "Root/Parent/Second".to_string(),
                "Root/Parent/First".to_string(),
                "Root/Parent/LiveAdded".to_string()
            ]
        );

        context.commit_candidate(&candidate.handle).unwrap();
        let committed_paths =
            child_display_paths_for_state(context.workspace(&workspace_id).unwrap(), parent_id);
        assert_eq!(
            committed_paths,
            vec![
                "Root/Parent/Second".to_string(),
                "Root/Parent/First".to_string(),
                "Root/Parent/LiveAdded".to_string()
            ]
        );
        assert_eq!(
            context
                .node_view(&workspace_id, first_id)
                .unwrap()
                .display_path,
            "Root/Parent/First"
        );
    }

    #[test]
    fn treecalc_context_rebases_candidate_delete_over_live_sibling_reorder() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:candidate-rebase-delete-over-reorder",
            ))
            .unwrap();
        let parent_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("Parent", ""))
            .unwrap();
        let first_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("First", "1").under(parent_id),
            )
            .unwrap();
        let second_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Second", "2").under(parent_id),
            )
            .unwrap();
        let delete_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("DeleteMe", "3").under(parent_id),
            )
            .unwrap();
        context.recalculate(&workspace_id).unwrap();
        let basis_revision_id = context
            .workspace_view(&workspace_id)
            .unwrap()
            .workspace_revision_id;

        let candidate = context
            .open_candidate(OxCalcTreeOpenCandidateRequest::new(
                workspace_id.clone(),
                basis_revision_id,
            ))
            .unwrap();
        context
            .apply_candidate_edit_transaction(
                &candidate.handle,
                OxCalcTreeEditTransaction::new(workspace_id.clone())
                    .with_edit(OxCalcTreeEdit::DeleteNode { node_id: delete_id }),
            )
            .unwrap();

        context.reorder_node(&workspace_id, second_id, 0).unwrap();
        let current_revision_id = context
            .workspace_view(&workspace_id)
            .unwrap()
            .workspace_revision_id;

        let rebased = context
            .rebase_candidate_to_current_revision(&candidate.handle)
            .unwrap();
        assert_eq!(rebased.basis_revision_id, current_revision_id);
        let sibling_paths = child_display_paths_for_state(
            &context
                .candidates
                .get(&candidate.handle)
                .expect("rebased candidate should be retained")
                .workspace_state,
            parent_id,
        );
        assert_eq!(
            sibling_paths,
            vec![
                "Root/Parent/Second".to_string(),
                "Root/Parent/First".to_string()
            ]
        );
        assert!(!rebased.nodes.iter().any(|node| node.node_id == delete_id));

        context.commit_candidate(&candidate.handle).unwrap();
        assert!(context.node_view(&workspace_id, delete_id).is_err());
        let committed_paths =
            child_display_paths_for_state(context.workspace(&workspace_id).unwrap(), parent_id);
        assert_eq!(
            committed_paths,
            vec![
                "Root/Parent/Second".to_string(),
                "Root/Parent/First".to_string()
            ]
        );
        assert_eq!(
            context
                .node_view(&workspace_id, first_id)
                .unwrap()
                .display_path,
            "Root/Parent/First"
        );
    }

    #[test]
    fn treecalc_context_rebases_candidate_reorder_over_live_sibling_delete() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:candidate-rebase-reorder-over-delete",
            ))
            .unwrap();
        let parent_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("Parent", ""))
            .unwrap();
        let first_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("First", "1").under(parent_id),
            )
            .unwrap();
        let second_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Second", "2").under(parent_id),
            )
            .unwrap();
        let delete_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("DeleteMe", "3").under(parent_id),
            )
            .unwrap();
        context.recalculate(&workspace_id).unwrap();
        let basis_revision_id = context
            .workspace_view(&workspace_id)
            .unwrap()
            .workspace_revision_id;

        let candidate = context
            .open_candidate(OxCalcTreeOpenCandidateRequest::new(
                workspace_id.clone(),
                basis_revision_id,
            ))
            .unwrap();
        context
            .apply_candidate_edit_transaction(
                &candidate.handle,
                OxCalcTreeEditTransaction::new(workspace_id.clone()).with_edit(
                    OxCalcTreeEdit::ReorderNode {
                        node_id: second_id,
                        new_index: 0,
                    },
                ),
            )
            .unwrap();

        context.delete_node(&workspace_id, delete_id).unwrap();
        let current_revision_id = context
            .workspace_view(&workspace_id)
            .unwrap()
            .workspace_revision_id;

        let rebased = context
            .rebase_candidate_to_current_revision(&candidate.handle)
            .unwrap();
        assert_eq!(rebased.basis_revision_id, current_revision_id);
        let sibling_paths = child_display_paths_for_state(
            &context
                .candidates
                .get(&candidate.handle)
                .expect("rebased candidate should be retained")
                .workspace_state,
            parent_id,
        );
        assert_eq!(
            sibling_paths,
            vec![
                "Root/Parent/Second".to_string(),
                "Root/Parent/First".to_string()
            ]
        );
        assert!(!rebased.nodes.iter().any(|node| node.node_id == delete_id));

        context.commit_candidate(&candidate.handle).unwrap();
        assert!(context.node_view(&workspace_id, delete_id).is_err());
        let committed_paths =
            child_display_paths_for_state(context.workspace(&workspace_id).unwrap(), parent_id);
        assert_eq!(
            committed_paths,
            vec![
                "Root/Parent/Second".to_string(),
                "Root/Parent/First".to_string()
            ]
        );
        assert_eq!(
            context
                .node_view(&workspace_id, first_id)
                .unwrap()
                .display_path,
            "Root/Parent/First"
        );
    }

    #[test]
    fn treecalc_context_rejects_rebase_when_candidate_add_conflicts_with_live_parent_order() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:candidate-rebase-add-conflict",
            ))
            .unwrap();
        let root_id = context.workspace_view(&workspace_id).unwrap().root_node_id;
        context.recalculate(&workspace_id).unwrap();
        let basis_revision_id = context
            .workspace_view(&workspace_id)
            .unwrap()
            .workspace_revision_id;

        let candidate = context
            .open_candidate(OxCalcTreeOpenCandidateRequest::new(
                workspace_id.clone(),
                basis_revision_id.clone(),
            ))
            .unwrap();
        context
            .apply_candidate_edit_transaction(
                &candidate.handle,
                OxCalcTreeEditTransaction::new(workspace_id.clone()).with_edit(
                    OxCalcTreeEdit::AddNode {
                        request: OxCalcTreeNodeCreate::new("CandidateChild", "7").under(root_id),
                    },
                ),
            )
            .unwrap();

        context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("LiveChild", "9").under(root_id),
            )
            .unwrap();
        let current_revision_id = context
            .workspace_view(&workspace_id)
            .unwrap()
            .workspace_revision_id;
        let error = context
            .rebase_candidate_to_current_revision(&candidate.handle)
            .unwrap_err();
        let OxCalcTreeContextError::CandidateRebaseConflict {
            overlapping_nodes,
            report,
            ..
        } = error
        else {
            panic!("expected candidate rebase conflict");
        };
        assert_eq!(overlapping_nodes, vec![root_id]);
        assert_eq!(report.basis_revision_id, basis_revision_id);
        assert_eq!(report.current_revision_id, current_revision_id);
        assert!(report.candidate_touched_nodes.contains(&root_id));
        assert!(report.live_touched_nodes.contains(&root_id));
        assert_eq!(
            context
                .candidate_view(&candidate.handle)
                .unwrap()
                .basis_revision_id,
            basis_revision_id
        );
    }

    #[test]
    fn treecalc_context_rejects_rebase_when_candidate_move_conflicts_with_live_destination_order() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:candidate-rebase-move-conflict",
            ))
            .unwrap();
        let root_id = context.workspace_view(&workspace_id).unwrap().root_node_id;
        let group_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Group", "").under(root_id),
            )
            .unwrap();
        let a_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("A", "1").under(root_id),
            )
            .unwrap();
        context.recalculate(&workspace_id).unwrap();
        let basis_revision_id = context
            .workspace_view(&workspace_id)
            .unwrap()
            .workspace_revision_id;

        let candidate = context
            .open_candidate(OxCalcTreeOpenCandidateRequest::new(
                workspace_id.clone(),
                basis_revision_id.clone(),
            ))
            .unwrap();
        context
            .apply_candidate_edit_transaction(
                &candidate.handle,
                OxCalcTreeEditTransaction::new(workspace_id.clone()).with_edit(
                    OxCalcTreeEdit::MoveNode {
                        node_id: a_id,
                        new_parent_id: group_id,
                        new_index: None,
                    },
                ),
            )
            .unwrap();

        context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("LiveChild", "9").under(group_id),
            )
            .unwrap();
        let error = context
            .rebase_candidate_to_current_revision(&candidate.handle)
            .unwrap_err();
        let OxCalcTreeContextError::CandidateRebaseConflict {
            overlapping_nodes,
            report,
            ..
        } = error
        else {
            panic!("expected candidate rebase conflict");
        };
        assert_eq!(overlapping_nodes, vec![group_id]);
        assert!(report.candidate_touched_nodes.contains(&group_id));
        assert!(report.live_touched_nodes.contains(&group_id));
        assert_eq!(
            context
                .candidate_view(&candidate.handle)
                .unwrap()
                .basis_revision_id,
            basis_revision_id
        );
    }

    #[test]
    fn treecalc_context_rejects_rebase_when_candidate_move_conflicts_with_live_old_parent_order() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:candidate-rebase-move-old-parent-conflict",
            ))
            .unwrap();
        let root_id = context.workspace_view(&workspace_id).unwrap().root_node_id;
        let source_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Source", "").under(root_id),
            )
            .unwrap();
        let dest_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Dest", "").under(root_id),
            )
            .unwrap();
        let a_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("A", "1").under(source_id),
            )
            .unwrap();
        context.recalculate(&workspace_id).unwrap();
        let basis_revision_id = context
            .workspace_view(&workspace_id)
            .unwrap()
            .workspace_revision_id;

        let candidate = context
            .open_candidate(OxCalcTreeOpenCandidateRequest::new(
                workspace_id.clone(),
                basis_revision_id.clone(),
            ))
            .unwrap();
        context
            .apply_candidate_edit_transaction(
                &candidate.handle,
                OxCalcTreeEditTransaction::new(workspace_id.clone()).with_edit(
                    OxCalcTreeEdit::MoveNode {
                        node_id: a_id,
                        new_parent_id: dest_id,
                        new_index: None,
                    },
                ),
            )
            .unwrap();

        context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("LiveChild", "9").under(source_id),
            )
            .unwrap();
        let error = context
            .rebase_candidate_to_current_revision(&candidate.handle)
            .unwrap_err();
        let OxCalcTreeContextError::CandidateRebaseConflict {
            overlapping_nodes,
            report,
            ..
        } = error
        else {
            panic!("expected candidate rebase conflict");
        };
        assert_eq!(overlapping_nodes, vec![source_id]);
        assert!(report.candidate_touched_nodes.contains(&source_id));
        assert!(report.live_touched_nodes.contains(&source_id));
    }

    #[test]
    fn treecalc_context_rejects_rebase_when_candidate_delete_conflicts_with_live_descendant_edit() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:candidate-rebase-delete-descendant-conflict",
            ))
            .unwrap();
        let root_id = context.workspace_view(&workspace_id).unwrap().root_node_id;
        let parent_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Parent", "").under(root_id),
            )
            .unwrap();
        let child_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Child", "1").under(parent_id),
            )
            .unwrap();
        context.recalculate(&workspace_id).unwrap();
        let basis_revision_id = context
            .workspace_view(&workspace_id)
            .unwrap()
            .workspace_revision_id;

        let candidate = context
            .open_candidate(OxCalcTreeOpenCandidateRequest::new(
                workspace_id.clone(),
                basis_revision_id.clone(),
            ))
            .unwrap();
        context
            .apply_candidate_edit_transaction(
                &candidate.handle,
                OxCalcTreeEditTransaction::new(workspace_id.clone())
                    .with_edit(OxCalcTreeEdit::DeleteNode { node_id: parent_id }),
            )
            .unwrap();

        context
            .set_node_input_value(&workspace_id, child_id, "2")
            .unwrap();
        let error = context
            .rebase_candidate_to_current_revision(&candidate.handle)
            .unwrap_err();
        let OxCalcTreeContextError::CandidateRebaseConflict {
            overlapping_nodes,
            report,
            ..
        } = error
        else {
            panic!("expected candidate rebase conflict");
        };
        assert_eq!(overlapping_nodes, vec![child_id]);
        assert!(report.candidate_touched_nodes.contains(&parent_id));
        assert!(report.candidate_touched_nodes.contains(&child_id));
        assert!(report.live_touched_nodes.contains(&child_id));
    }

    #[test]
    fn treecalc_context_rejects_rebase_when_candidate_reorder_conflicts_with_live_parent_order() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:candidate-rebase-reorder-conflict",
            ))
            .unwrap();
        let root_id = context.workspace_view(&workspace_id).unwrap().root_node_id;
        let parent_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Parent", "").under(root_id),
            )
            .unwrap();
        let a_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("A", "1").under(parent_id),
            )
            .unwrap();
        let b_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("B", "2").under(parent_id),
            )
            .unwrap();
        context.recalculate(&workspace_id).unwrap();
        let basis_revision_id = context
            .workspace_view(&workspace_id)
            .unwrap()
            .workspace_revision_id;

        let candidate = context
            .open_candidate(OxCalcTreeOpenCandidateRequest::new(
                workspace_id.clone(),
                basis_revision_id,
            ))
            .unwrap();
        context
            .apply_candidate_edit_transaction(
                &candidate.handle,
                OxCalcTreeEditTransaction::new(workspace_id.clone()).with_edit(
                    OxCalcTreeEdit::ReorderNode {
                        node_id: a_id,
                        new_index: 1,
                    },
                ),
            )
            .unwrap();

        context.reorder_node(&workspace_id, b_id, 0).unwrap();
        let error = context
            .rebase_candidate_to_current_revision(&candidate.handle)
            .unwrap_err();
        let OxCalcTreeContextError::CandidateRebaseConflict {
            overlapping_nodes,
            report,
            ..
        } = error
        else {
            panic!("expected candidate rebase conflict");
        };
        assert_eq!(overlapping_nodes, vec![parent_id]);
        assert!(report.candidate_touched_nodes.contains(&parent_id));
        assert!(report.live_touched_nodes.contains(&parent_id));
    }

    #[test]
    fn treecalc_context_rebases_unparented_candidate_to_current_revision_without_overlap() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new("workspace:candidate-rebase"))
            .unwrap();
        let a_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("A", "1"))
            .unwrap();
        let b_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("B", "=A+1"))
            .unwrap();
        let c_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("C", "10"))
            .unwrap();
        context.recalculate(&workspace_id).unwrap();
        let basis_revision_id = context
            .workspace_view(&workspace_id)
            .unwrap()
            .workspace_revision_id;

        let candidate = context
            .open_candidate(OxCalcTreeOpenCandidateRequest::new(
                workspace_id.clone(),
                basis_revision_id.clone(),
            ))
            .unwrap();
        context
            .apply_candidate_edit_transaction(
                &candidate.handle,
                OxCalcTreeEditTransaction::new(workspace_id.clone()).with_edit(
                    OxCalcTreeEdit::SetNodeInput {
                        node_id: a_id,
                        input: "5".to_string(),
                    },
                ),
            )
            .unwrap();

        context
            .apply_edit_transaction(
                OxCalcTreeEditTransaction::new(workspace_id.clone()).with_edit(
                    OxCalcTreeEdit::SetNodeInput {
                        node_id: c_id,
                        input: "20".to_string(),
                    },
                ),
            )
            .unwrap();
        let current_revision_id = context
            .workspace_view(&workspace_id)
            .unwrap()
            .workspace_revision_id;
        assert_ne!(current_revision_id, basis_revision_id);

        let rebased = context
            .rebase_candidate_to_current_revision(&candidate.handle)
            .unwrap();
        assert_eq!(rebased.basis_revision_id, current_revision_id);
        assert!(
            rebased.calculation.is_none(),
            "rebase should replay private edits without carrying stale calculation results"
        );
        let rebased = context.evaluate_candidate(&candidate.handle).unwrap();
        assert_eq!(
            rebased
                .calculation
                .as_ref()
                .and_then(|calculation| calculation.published_values.get(&b_id)),
            Some(&"6".to_string())
        );
        assert_eq!(
            context.node_view(&workspace_id, b_id).unwrap().value_text,
            Some("2".to_string()),
            "rebased candidate evaluation must not publish"
        );
        assert_eq!(
            context.node_view(&workspace_id, c_id).unwrap().value_text,
            Some("20".to_string())
        );

        let commit = context.commit_candidate(&candidate.handle).unwrap();
        assert_eq!(commit.basis_revision_id, current_revision_id);
        assert_eq!(
            context.node_view(&workspace_id, b_id).unwrap().value_text,
            Some("6".to_string())
        );
        assert_eq!(
            context.node_view(&workspace_id, c_id).unwrap().value_text,
            Some("20".to_string())
        );
    }

    #[test]
    fn treecalc_context_child_candidate_starts_from_parent_private_state() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new("workspace:candidate-layer"))
            .unwrap();
        let a_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("A", "1"))
            .unwrap();
        let b_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("B", "=A+1"))
            .unwrap();
        context.recalculate(&workspace_id).unwrap();
        let basis_revision_id = context
            .workspace_view(&workspace_id)
            .unwrap()
            .workspace_revision_id;

        let parent = context
            .open_candidate(OxCalcTreeOpenCandidateRequest::new(
                workspace_id.clone(),
                basis_revision_id.clone(),
            ))
            .unwrap();
        context
            .apply_candidate_edit_transaction(
                &parent.handle,
                OxCalcTreeEditTransaction::new(workspace_id.clone()).with_edit(
                    OxCalcTreeEdit::SetNodeInput {
                        node_id: a_id,
                        input: "5".to_string(),
                    },
                ),
            )
            .unwrap();
        let parent = context.evaluate_candidate(&parent.handle).unwrap();
        assert_eq!(
            parent
                .calculation
                .as_ref()
                .and_then(|calculation| calculation.published_values.get(&b_id)),
            Some(&"6".to_string())
        );

        let child = context
            .open_candidate(
                OxCalcTreeOpenCandidateRequest::new(
                    workspace_id.clone(),
                    basis_revision_id.clone(),
                )
                .with_parent_candidate(parent.handle.clone()),
            )
            .unwrap();
        assert_eq!(child.parent_candidate, Some(parent.handle.clone()));
        let child = context.evaluate_candidate(&child.handle).unwrap();
        assert_eq!(
            child
                .calculation
                .as_ref()
                .and_then(|calculation| calculation.published_values.get(&b_id)),
            Some(&"6".to_string())
        );
        assert_eq!(
            context.node_view(&workspace_id, b_id).unwrap().value_text,
            Some("2".to_string())
        );

        let parent_discard = context.discard_candidate(&parent.handle).unwrap_err();
        assert!(matches!(
            parent_discard,
            OxCalcTreeContextError::CandidateHasRetainedChild { .. }
        ));
        context.discard_candidate(&child.handle).unwrap();
        context.discard_candidate(&parent.handle).unwrap();
    }

    #[test]
    fn treecalc_context_child_candidate_tracks_parent_private_edits_after_open() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:candidate-live-layer",
            ))
            .unwrap();
        let a_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("A", "1"))
            .unwrap();
        let b_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("B", "=A+1"))
            .unwrap();
        context.recalculate(&workspace_id).unwrap();
        let basis_revision_id = context
            .workspace_view(&workspace_id)
            .unwrap()
            .workspace_revision_id;

        let parent = context
            .open_candidate(OxCalcTreeOpenCandidateRequest::new(
                workspace_id.clone(),
                basis_revision_id.clone(),
            ))
            .unwrap();
        let child = context
            .open_candidate(
                OxCalcTreeOpenCandidateRequest::new(
                    workspace_id.clone(),
                    basis_revision_id.clone(),
                )
                .with_parent_candidate(parent.handle.clone()),
            )
            .unwrap();

        context
            .apply_candidate_edit_transaction(
                &parent.handle,
                OxCalcTreeEditTransaction::new(workspace_id.clone()).with_edit(
                    OxCalcTreeEdit::SetNodeInput {
                        node_id: a_id,
                        input: "5".to_string(),
                    },
                ),
            )
            .unwrap();
        let child = context.evaluate_candidate(&child.handle).unwrap();

        assert_eq!(child.parent_candidate, Some(parent.handle.clone()));
        assert_eq!(
            child
                .calculation
                .as_ref()
                .and_then(|calculation| calculation.published_values.get(&b_id)),
            Some(&"6".to_string()),
            "child candidate should refresh from parent private state edited after child open"
        );
        assert_eq!(
            context.node_view(&workspace_id, b_id).unwrap().value_text,
            Some("2".to_string()),
            "candidate layering must not publish workspace state"
        );
    }

    #[test]
    fn treecalc_context_child_candidate_commit_publishes_layered_state() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:candidate-layer-commit",
            ))
            .unwrap();
        let a_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("A", "1"))
            .unwrap();
        let b_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("B", "=A+1"))
            .unwrap();
        context.recalculate(&workspace_id).unwrap();
        let basis_revision_id = context
            .workspace_view(&workspace_id)
            .unwrap()
            .workspace_revision_id;

        let parent = context
            .open_candidate(OxCalcTreeOpenCandidateRequest::new(
                workspace_id.clone(),
                basis_revision_id.clone(),
            ))
            .unwrap();
        context
            .apply_candidate_edit_transaction(
                &parent.handle,
                OxCalcTreeEditTransaction::new(workspace_id.clone()).with_edit(
                    OxCalcTreeEdit::SetNodeInput {
                        node_id: a_id,
                        input: "5".to_string(),
                    },
                ),
            )
            .unwrap();

        let child = context
            .open_candidate(
                OxCalcTreeOpenCandidateRequest::new(
                    workspace_id.clone(),
                    basis_revision_id.clone(),
                )
                .with_parent_candidate(parent.handle.clone()),
            )
            .unwrap();
        context
            .apply_candidate_edit_transaction(
                &child.handle,
                OxCalcTreeEditTransaction::new(workspace_id.clone()).with_edit(
                    OxCalcTreeEdit::SetNodeInput {
                        node_id: a_id,
                        input: "7".to_string(),
                    },
                ),
            )
            .unwrap();
        context.evaluate_candidate(&child.handle).unwrap();

        let parent_commit = context.commit_candidate(&parent.handle).unwrap_err();
        assert!(matches!(
            parent_commit,
            OxCalcTreeContextError::CandidateHasRetainedChild { .. }
        ));

        let commit = context.commit_candidate(&child.handle).unwrap();
        assert_eq!(commit.basis_revision_id, basis_revision_id);
        assert_eq!(
            commit
                .calculation
                .as_ref()
                .and_then(|calculation| calculation.published_values.get(&b_id)),
            Some(&"8".to_string())
        );
        assert_eq!(
            context.node_view(&workspace_id, b_id).unwrap().value_text,
            Some("8".to_string())
        );
        assert!(matches!(
            context.commit_candidate(&parent.handle),
            Err(OxCalcTreeContextError::CandidateBasisNotCurrent { .. })
        ));
    }

    #[test]
    fn treecalc_context_rebases_parented_candidate_by_flattening_layered_edits() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:candidate-parent-rebase",
            ))
            .unwrap();
        let a_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("A", "1"))
            .unwrap();
        let b_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("B", "=A+1"))
            .unwrap();
        let c_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("C", "10"))
            .unwrap();
        context.recalculate(&workspace_id).unwrap();
        let basis_revision_id = context
            .workspace_view(&workspace_id)
            .unwrap()
            .workspace_revision_id;

        let parent = context
            .open_candidate(OxCalcTreeOpenCandidateRequest::new(
                workspace_id.clone(),
                basis_revision_id.clone(),
            ))
            .unwrap();
        context
            .apply_candidate_edit_transaction(
                &parent.handle,
                OxCalcTreeEditTransaction::new(workspace_id.clone()).with_edit(
                    OxCalcTreeEdit::SetNodeInput {
                        node_id: a_id,
                        input: "5".to_string(),
                    },
                ),
            )
            .unwrap();
        let child = context
            .open_candidate(
                OxCalcTreeOpenCandidateRequest::new(
                    workspace_id.clone(),
                    basis_revision_id.clone(),
                )
                .with_parent_candidate(parent.handle.clone()),
            )
            .unwrap();

        context
            .apply_candidate_edit_transaction(
                &child.handle,
                OxCalcTreeEditTransaction::new(workspace_id.clone()).with_edit(
                    OxCalcTreeEdit::SetNodeInput {
                        node_id: a_id,
                        input: "7".to_string(),
                    },
                ),
            )
            .unwrap();

        context
            .apply_edit_transaction(
                OxCalcTreeEditTransaction::new(workspace_id.clone()).with_edit(
                    OxCalcTreeEdit::SetNodeInput {
                        node_id: c_id,
                        input: "20".to_string(),
                    },
                ),
            )
            .unwrap();
        let current_revision_id = context
            .workspace_view(&workspace_id)
            .unwrap()
            .workspace_revision_id;

        let rebased = context
            .rebase_candidate_to_current_revision(&child.handle)
            .unwrap();
        assert_eq!(rebased.basis_revision_id, current_revision_id);
        assert_eq!(rebased.parent_candidate, None);
        assert!(
            rebased.calculation.is_none(),
            "rebase should flatten parented private edits without carrying stale calculation results"
        );
        assert!(
            context.discard_candidate(&parent.handle).is_ok(),
            "flattened child should no longer protect the parent"
        );

        let rebased = context.evaluate_candidate(&child.handle).unwrap();
        assert_eq!(
            rebased
                .calculation
                .as_ref()
                .and_then(|calculation| calculation.published_values.get(&b_id)),
            Some(&"8".to_string())
        );
        assert_eq!(
            context.node_view(&workspace_id, b_id).unwrap().value_text,
            Some("2".to_string()),
            "rebased parented candidate evaluation must not publish"
        );
        assert_eq!(
            context.node_view(&workspace_id, c_id).unwrap().value_text,
            Some("20".to_string())
        );

        let commit = context.commit_candidate(&child.handle).unwrap();
        assert_eq!(commit.basis_revision_id, current_revision_id);
        assert_eq!(
            context.node_view(&workspace_id, b_id).unwrap().value_text,
            Some("8".to_string())
        );
        assert_eq!(
            context.node_view(&workspace_id, c_id).unwrap().value_text,
            Some("20".to_string())
        );
    }

    #[test]
    fn treecalc_context_navigates_retained_workspace_revisions_and_branches() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new("workspace:navigation"))
            .unwrap();
        let a_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("A", "1"))
            .unwrap();
        let b_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("B", "=A+1"))
            .unwrap();
        context.recalculate(&workspace_id).unwrap();
        let original_revision = context.workspace_revision(&workspace_id).unwrap();
        let original_view = context.workspace_view(&workspace_id).unwrap();

        let transaction = OxCalcTreeEditTransaction::new(workspace_id.clone()).with_edit(
            OxCalcTreeEdit::SetNodeInput {
                node_id: a_id,
                input: "5".to_string(),
            },
        );
        let edited_outcome = context.apply_edit_transaction(transaction).unwrap();
        let edited_revision_id = edited_outcome.workspace_revision_id.clone();
        let edited_view = context.workspace_view(&workspace_id).unwrap();
        assert_eq!(
            edited_view
                .nodes
                .iter()
                .find(|node| node.node_id == b_id)
                .and_then(|node| node.value_text.as_deref()),
            Some("6")
        );

        let navigation = context
            .navigate_workspace_revision(&workspace_id, original_revision.revision_id())
            .unwrap();
        assert_eq!(
            navigation.predecessor_workspace_revision_id,
            edited_revision_id
        );
        assert_eq!(
            navigation.successor_workspace_revision_id,
            *original_revision.revision_id()
        );
        let restored_view = context.workspace_view(&workspace_id).unwrap();
        assert_eq!(
            restored_view.workspace_revision_id,
            *original_revision.revision_id()
        );
        assert_eq!(
            restored_view.workspace_revision_parent_id,
            original_view.workspace_revision_parent_id
        );
        assert_eq!(
            restored_view
                .nodes
                .iter()
                .find(|node| node.node_id == a_id)
                .and_then(|node| node.value_text.as_deref()),
            Some("1")
        );
        assert_eq!(
            restored_view
                .nodes
                .iter()
                .find(|node| node.node_id == b_id)
                .and_then(|node| node.value_text.as_deref()),
            Some("2")
        );
        let missing_revision_id = WorkspaceRevisionId("workspace-revision:missing".to_string());
        assert!(
            context
                .navigate_workspace_revision(&workspace_id, &missing_revision_id)
                .is_err()
        );
        assert_eq!(
            context
                .workspace_view(&workspace_id)
                .unwrap()
                .workspace_revision_id,
            *original_revision.revision_id()
        );

        let branch = OxCalcTreeEditTransaction::new(workspace_id.clone()).with_edit(
            OxCalcTreeEdit::SetNodeInput {
                node_id: a_id,
                input: "7".to_string(),
            },
        );
        let branch_outcome = context.apply_edit_transaction(branch).unwrap();
        let branch_view = context.workspace_view(&workspace_id).unwrap();
        assert_eq!(
            branch_outcome.predecessor_workspace_revision_id,
            *original_revision.revision_id()
        );
        assert_eq!(
            branch_view.workspace_revision_parent_id,
            Some(original_revision.revision_id().clone())
        );
        assert!(
            branch_view
                .workspace_revision_graph_entries
                .iter()
                .any(|entry| {
                    entry.revision_id == edited_revision_id
                        && entry.parent_revision_id == Some(original_revision.revision_id().clone())
                })
        );
        assert!(
            branch_view
                .workspace_revision_graph_entries
                .iter()
                .any(|entry| {
                    entry.revision_id == branch_outcome.workspace_revision_id
                        && entry.parent_revision_id == Some(original_revision.revision_id().clone())
                })
        );
        assert_eq!(
            branch_view
                .nodes
                .iter()
                .find(|node| node.node_id == b_id)
                .and_then(|node| node.value_text.as_deref()),
            Some("8")
        );
    }

    #[test]
    fn treecalc_context_bounds_retained_workspace_revisions_oldest_first() {
        let mut context = OxCalcTreeContext::new(
            OxCalcTreeContextOptions::default()
                .with_revision_retention_policy(OxCalcTreeRevisionRetentionPolicy::bounded(3)),
        );
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new("workspace:bounded-history"))
            .unwrap();
        let initial_revision = context
            .workspace_revision(&workspace_id)
            .unwrap()
            .revision_id()
            .clone();

        context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("A", "1"))
            .unwrap();
        context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("B", "=A+1"))
            .unwrap();
        context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("C", "=B+1"))
            .unwrap();

        let view = context.workspace_view(&workspace_id).unwrap();
        assert_eq!(view.retained_workspace_revision_count, 3);
        assert_eq!(view.workspace_revision_graph_entries.len(), 3);
        assert!(
            !view
                .workspace_revision_graph_entries
                .iter()
                .any(|entry| entry.revision_id == initial_revision)
        );

        let navigation = context.navigate_workspace_revision(&workspace_id, &initial_revision);
        assert!(matches!(
            navigation,
            Err(OxCalcTreeContextError::WorkspaceRevisionNotRetained { .. })
        ));
        assert_eq!(
            context
                .workspace_view(&workspace_id)
                .unwrap()
                .workspace_revision_id,
            view.workspace_revision_id
        );
    }

    #[test]
    fn treecalc_context_open_candidate_pins_basis_revision_under_bounded_retention() {
        let mut context = OxCalcTreeContext::new(
            OxCalcTreeContextOptions::default()
                .with_revision_retention_policy(OxCalcTreeRevisionRetentionPolicy::bounded(2)),
        );
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:candidate-basis-pin",
            ))
            .unwrap();
        let a_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("A", "1"))
            .unwrap();
        context.recalculate(&workspace_id).unwrap();
        let pinned_revision = context
            .workspace_view(&workspace_id)
            .unwrap()
            .workspace_revision_id;

        let candidate = context
            .open_candidate(OxCalcTreeOpenCandidateRequest::new(
                workspace_id.clone(),
                pinned_revision.clone(),
            ))
            .unwrap();
        for input in ["2", "3"] {
            context
                .apply_edit_transaction(
                    OxCalcTreeEditTransaction::new(workspace_id.clone()).with_edit(
                        OxCalcTreeEdit::SetNodeInput {
                            node_id: a_id,
                            input: input.to_string(),
                        },
                    ),
                )
                .unwrap();
        }

        let view = context.workspace_view(&workspace_id).unwrap();
        let live_revision = view.workspace_revision_id.clone();
        assert_ne!(live_revision, pinned_revision);
        assert!(view.retained_workspace_revision_count >= 2);
        assert!(
            view.workspace_revision_graph_entries
                .iter()
                .any(|entry| entry.revision_id == pinned_revision)
        );
        context
            .navigate_workspace_revision(&workspace_id, &pinned_revision)
            .expect("open candidate basis should remain retained");
        context
            .navigate_workspace_revision(&workspace_id, &live_revision)
            .expect("live revision should remain retained");

        context.discard_candidate(&candidate.handle).unwrap();
        for input in ["4", "5", "6"] {
            context
                .apply_edit_transaction(
                    OxCalcTreeEditTransaction::new(workspace_id.clone()).with_edit(
                        OxCalcTreeEdit::SetNodeInput {
                            node_id: a_id,
                            input: input.to_string(),
                        },
                    ),
                )
                .unwrap();
        }
        assert!(matches!(
            context.navigate_workspace_revision(&workspace_id, &pinned_revision),
            Err(OxCalcTreeContextError::WorkspaceRevisionNotRetained { .. })
        ));
    }

    #[test]
    fn treecalc_context_candidate_basis_pin_is_reference_counted() {
        let mut context = OxCalcTreeContext::new(
            OxCalcTreeContextOptions::default()
                .with_revision_retention_policy(OxCalcTreeRevisionRetentionPolicy::bounded(2)),
        );
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:candidate-basis-pin-count",
            ))
            .unwrap();
        let a_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("A", "1"))
            .unwrap();
        context.recalculate(&workspace_id).unwrap();
        let pinned_revision = context
            .workspace_view(&workspace_id)
            .unwrap()
            .workspace_revision_id;

        let first = context
            .open_candidate(OxCalcTreeOpenCandidateRequest::new(
                workspace_id.clone(),
                pinned_revision.clone(),
            ))
            .unwrap();
        let second = context
            .open_candidate(OxCalcTreeOpenCandidateRequest::new(
                workspace_id.clone(),
                pinned_revision.clone(),
            ))
            .unwrap();
        context.discard_candidate(&first.handle).unwrap();
        for input in ["2", "3"] {
            context
                .apply_edit_transaction(
                    OxCalcTreeEditTransaction::new(workspace_id.clone()).with_edit(
                        OxCalcTreeEdit::SetNodeInput {
                            node_id: a_id,
                            input: input.to_string(),
                        },
                    ),
                )
                .unwrap();
        }
        let live_revision = context
            .workspace_view(&workspace_id)
            .unwrap()
            .workspace_revision_id;
        context
            .navigate_workspace_revision(&workspace_id, &pinned_revision)
            .expect("second candidate should keep shared basis retained");
        context
            .navigate_workspace_revision(&workspace_id, &live_revision)
            .expect("live revision should remain retained");

        context.discard_candidate(&second.handle).unwrap();
        for input in ["4", "5", "6"] {
            context
                .apply_edit_transaction(
                    OxCalcTreeEditTransaction::new(workspace_id.clone()).with_edit(
                        OxCalcTreeEdit::SetNodeInput {
                            node_id: a_id,
                            input: input.to_string(),
                        },
                    ),
                )
                .unwrap();
        }
        assert!(matches!(
            context.navigate_workspace_revision(&workspace_id, &pinned_revision),
            Err(OxCalcTreeContextError::WorkspaceRevisionNotRetained { .. })
        ));
    }

    #[test]
    fn treecalc_context_reaps_candidates_to_budget_and_reports_pressure() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:candidate-reap-budget",
            ))
            .unwrap();
        context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("A", "1"))
            .unwrap();
        context.recalculate(&workspace_id).unwrap();
        let basis_revision = context
            .workspace_view(&workspace_id)
            .unwrap()
            .workspace_revision_id;
        let first = context
            .open_candidate(OxCalcTreeOpenCandidateRequest::new(
                workspace_id.clone(),
                basis_revision.clone(),
            ))
            .unwrap();
        let second = context
            .open_candidate(OxCalcTreeOpenCandidateRequest::new(
                workspace_id.clone(),
                basis_revision.clone(),
            ))
            .unwrap();
        let third = context
            .open_candidate(OxCalcTreeOpenCandidateRequest::new(
                workspace_id.clone(),
                basis_revision,
            ))
            .unwrap();

        let policy = OxCalcTreeCandidateReapPolicy::max_retained(1);
        let pressure = context.candidate_pressure(&policy);
        assert_eq!(pressure.retained_candidate_count, 3);
        assert_eq!(pressure.child_protected_candidate_count, 0);
        assert_eq!(pressure.host_pinned_candidate_count, 0);
        assert_eq!(pressure.protected_candidate_count, 0);
        assert_eq!(pressure.reclaimable_candidate_count, 3);
        assert_eq!(pressure.over_budget_candidate_count, 2);

        let report = context.reap_candidates(policy).unwrap();
        assert_eq!(
            report.reaped_handles,
            vec![first.handle.clone(), second.handle.clone()]
        );
        assert_eq!(report.pressure_after.retained_candidate_count, 1);
        assert_eq!(report.pressure_after.over_budget_candidate_count, 0);
        assert!(matches!(
            context.candidate_view(&first.handle),
            Err(OxCalcTreeContextError::UnknownCandidate { .. })
        ));
        assert!(matches!(
            context.candidate_view(&second.handle),
            Err(OxCalcTreeContextError::UnknownCandidate { .. })
        ));
        context
            .candidate_view(&third.handle)
            .expect("budget survivor should remain retained");
    }

    #[test]
    fn treecalc_context_reaper_protects_parent_candidate_with_retained_child() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:candidate-reap-parent",
            ))
            .unwrap();
        context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("A", "1"))
            .unwrap();
        context.recalculate(&workspace_id).unwrap();
        let basis_revision = context
            .workspace_view(&workspace_id)
            .unwrap()
            .workspace_revision_id;
        let parent = context
            .open_candidate(OxCalcTreeOpenCandidateRequest::new(
                workspace_id.clone(),
                basis_revision.clone(),
            ))
            .unwrap();
        let child = context
            .open_candidate(
                OxCalcTreeOpenCandidateRequest::new(workspace_id.clone(), basis_revision)
                    .with_parent_candidate(parent.handle.clone()),
            )
            .unwrap();

        let report = context
            .reap_candidates(OxCalcTreeCandidateReapPolicy::max_retained(1))
            .unwrap();
        assert_eq!(report.reaped_handles, vec![child.handle.clone()]);
        assert_eq!(report.pressure_before.child_protected_candidate_count, 1);
        assert_eq!(report.pressure_before.host_pinned_candidate_count, 0);
        assert_eq!(report.pressure_before.protected_candidate_count, 1);
        assert_eq!(report.pressure_after.retained_candidate_count, 1);
        context
            .candidate_view(&parent.handle)
            .expect("parent should be retained while it was protected by child");
        assert!(matches!(
            context.candidate_view(&child.handle),
            Err(OxCalcTreeContextError::UnknownCandidate { .. })
        ));
    }

    #[test]
    fn treecalc_context_reaper_protects_host_pinned_candidates() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:candidate-reap-host-pin",
            ))
            .unwrap();
        context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("A", "1"))
            .unwrap();
        context.recalculate(&workspace_id).unwrap();
        let basis_revision = context
            .workspace_view(&workspace_id)
            .unwrap()
            .workspace_revision_id;
        let pinned = context
            .open_candidate(OxCalcTreeOpenCandidateRequest::new(
                workspace_id.clone(),
                basis_revision.clone(),
            ))
            .unwrap();
        let reclaimable = context
            .open_candidate(OxCalcTreeOpenCandidateRequest::new(
                workspace_id.clone(),
                basis_revision,
            ))
            .unwrap();

        let pinned_view = context.pin_candidate_retention(&pinned.handle).unwrap();
        assert_eq!(pinned_view.retention_pin_count, 1);
        let pressure = context.candidate_pressure(&OxCalcTreeCandidateReapPolicy::max_retained(1));
        assert_eq!(pressure.retained_candidate_count, 2);
        assert_eq!(pressure.child_protected_candidate_count, 0);
        assert_eq!(pressure.host_pinned_candidate_count, 1);
        assert_eq!(pressure.protected_candidate_count, 1);
        assert_eq!(pressure.reclaimable_candidate_count, 1);

        let report = context
            .reap_candidates(OxCalcTreeCandidateReapPolicy::max_retained(1))
            .unwrap();
        assert_eq!(report.reaped_handles, vec![reclaimable.handle.clone()]);
        context
            .candidate_view(&pinned.handle)
            .expect("host-pinned candidate should survive reaping");
        assert!(matches!(
            context.candidate_view(&reclaimable.handle),
            Err(OxCalcTreeContextError::UnknownCandidate { .. })
        ));
    }

    #[test]
    fn treecalc_context_candidate_retention_unpin_rejects_without_pin() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:candidate-unpin-without-pin",
            ))
            .unwrap();
        context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("A", "1"))
            .unwrap();
        context.recalculate(&workspace_id).unwrap();
        let basis_revision = context
            .workspace_view(&workspace_id)
            .unwrap()
            .workspace_revision_id;
        let candidate = context
            .open_candidate(OxCalcTreeOpenCandidateRequest::new(
                workspace_id,
                basis_revision,
            ))
            .unwrap();

        let error = context
            .unpin_candidate_retention(&candidate.handle)
            .unwrap_err();
        assert!(matches!(
            error,
            OxCalcTreeContextError::CandidateRetentionPinNotHeld { .. }
        ));
        let pinned = context.pin_candidate_retention(&candidate.handle).unwrap();
        assert_eq!(pinned.retention_pin_count, 1);
        let unpinned = context
            .unpin_candidate_retention(&candidate.handle)
            .unwrap();
        assert_eq!(unpinned.retention_pin_count, 0);
    }

    #[test]
    fn treecalc_context_candidate_commit_preserves_other_candidate_basis_pins() {
        let mut context = OxCalcTreeContext::new(
            OxCalcTreeContextOptions::default()
                .with_revision_retention_policy(OxCalcTreeRevisionRetentionPolicy::bounded(2)),
        );
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:candidate-commit-preserves-pins",
            ))
            .unwrap();
        let a_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("A", "1"))
            .unwrap();
        context.recalculate(&workspace_id).unwrap();
        let pinned_revision = context
            .workspace_view(&workspace_id)
            .unwrap()
            .workspace_revision_id;

        let committing_candidate = context
            .open_candidate(OxCalcTreeOpenCandidateRequest::new(
                workspace_id.clone(),
                pinned_revision.clone(),
            ))
            .unwrap();
        let retained_candidate = context
            .open_candidate(OxCalcTreeOpenCandidateRequest::new(
                workspace_id.clone(),
                pinned_revision.clone(),
            ))
            .unwrap();
        context
            .apply_candidate_edit_transaction(
                &committing_candidate.handle,
                OxCalcTreeEditTransaction::new(workspace_id.clone()).with_edit(
                    OxCalcTreeEdit::SetNodeInput {
                        node_id: a_id,
                        input: "2".to_string(),
                    },
                ),
            )
            .unwrap();
        context
            .evaluate_candidate(&committing_candidate.handle)
            .unwrap();
        context
            .commit_candidate(&committing_candidate.handle)
            .unwrap();
        for input in ["3", "4"] {
            context
                .apply_edit_transaction(
                    OxCalcTreeEditTransaction::new(workspace_id.clone()).with_edit(
                        OxCalcTreeEdit::SetNodeInput {
                            node_id: a_id,
                            input: input.to_string(),
                        },
                    ),
                )
                .unwrap();
        }
        let live_revision = context
            .workspace_view(&workspace_id)
            .unwrap()
            .workspace_revision_id;
        context
            .navigate_workspace_revision(&workspace_id, &pinned_revision)
            .expect("retained sibling candidate should keep shared basis retained after commit");
        context
            .navigate_workspace_revision(&workspace_id, &live_revision)
            .expect("live revision should remain retained");

        context
            .discard_candidate(&retained_candidate.handle)
            .unwrap();
        for input in ["5", "6", "7"] {
            context
                .apply_edit_transaction(
                    OxCalcTreeEditTransaction::new(workspace_id.clone()).with_edit(
                        OxCalcTreeEdit::SetNodeInput {
                            node_id: a_id,
                            input: input.to_string(),
                        },
                    ),
                )
                .unwrap();
        }
        assert!(matches!(
            context.navigate_workspace_revision(&workspace_id, &pinned_revision),
            Err(OxCalcTreeContextError::WorkspaceRevisionNotRetained { .. })
        ));
    }

    #[test]
    fn treecalc_context_candidate_projects_private_structural_edits() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:candidate-structural-projection",
            ))
            .unwrap();
        let a_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("A", "1"))
            .unwrap();
        context.recalculate(&workspace_id).unwrap();
        let basis_revision = context
            .workspace_view(&workspace_id)
            .unwrap()
            .workspace_revision_id;

        let candidate = context
            .open_candidate(OxCalcTreeOpenCandidateRequest::new(
                workspace_id.clone(),
                basis_revision.clone(),
            ))
            .unwrap();
        let renamed = context
            .apply_candidate_edit_transaction(
                &candidate.handle,
                OxCalcTreeEditTransaction::new(workspace_id.clone()).with_edit(
                    OxCalcTreeEdit::RenameNode {
                        node_id: a_id,
                        new_symbol: "Renamed".to_string(),
                    },
                ),
            )
            .unwrap();
        assert_eq!(
            renamed
                .nodes
                .iter()
                .find(|node| node.node_id == a_id)
                .map(|node| node.display_path.as_str()),
            Some("Root/Renamed")
        );
        assert_eq!(
            context
                .node_view(&workspace_id, a_id)
                .unwrap()
                .display_path
                .as_str(),
            "Root/A"
        );
        let commit = context.commit_candidate(&candidate.handle).unwrap();
        assert_ne!(commit.successor_workspace_revision_id, basis_revision);
        assert_eq!(
            context
                .node_view(&workspace_id, a_id)
                .unwrap()
                .display_path
                .as_str(),
            "Root/Renamed"
        );
    }

    #[test]
    fn treecalc_context_dry_binds_candidate_new_node_against_private_structure() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:candidate-new-node-dry-bind",
            ))
            .unwrap();
        let a_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("A", "1"))
            .unwrap();
        context.recalculate(&workspace_id).unwrap();
        let basis_revision = context
            .workspace_view(&workspace_id)
            .unwrap()
            .workspace_revision_id;

        let candidate = context
            .open_candidate(OxCalcTreeOpenCandidateRequest::new(
                workspace_id.clone(),
                basis_revision,
            ))
            .unwrap();
        context
            .apply_candidate_edit_transaction(
                &candidate.handle,
                OxCalcTreeEditTransaction::new(workspace_id.clone()).with_edit(
                    OxCalcTreeEdit::RenameNode {
                        node_id: a_id,
                        new_symbol: "PrivateA".to_string(),
                    },
                ),
            )
            .unwrap();

        let live_verdict = context
            .dry_bind_new_node_formula_text(
                &workspace_id,
                OxCalcTreeNodeCreate::new("LiveFormula", "=PrivateA+1"),
            )
            .unwrap();
        assert!(!live_verdict.legal);

        let candidate_verdict = context
            .dry_bind_candidate_new_node_formula_text(
                &candidate.handle,
                OxCalcTreeNodeCreate::new("CandidateFormula", "=PrivateA+1"),
            )
            .unwrap();
        assert!(candidate_verdict.legal, "{candidate_verdict:?}");
    }

    #[test]
    fn treecalc_context_edit_transaction_can_reference_reserved_added_node_ids() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:transaction-reserved-ids",
            ))
            .unwrap();
        let table_node_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("Sales", ""))
            .unwrap();
        let row_1_amount_id = context.reserve_node_id();
        let row_2_amount_id = context.reserve_node_id();
        let mut snapshot = sales_table_snapshot(table_node_id);
        snapshot.body_cell_nodes = vec![
            TreeCalcTableBodyCellNodeBinding {
                row_id: TreeCalcTableRowId("row:1".to_string()),
                column_id: "table:sales:col:amount".to_string(),
                node_id: row_1_amount_id,
            },
            TreeCalcTableBodyCellNodeBinding {
                row_id: TreeCalcTableRowId("row:2".to_string()),
                column_id: "table:sales:col:amount".to_string(),
                node_id: row_2_amount_id,
            },
        ];

        let transaction = OxCalcTreeEditTransaction::new(workspace_id.clone())
            .with_recalc_policy(TransactionRecalcPolicy::ApplyOnly)
            .with_edit(OxCalcTreeEdit::AddNode {
                request: OxCalcTreeNodeCreate::new("Amount_r1", "10")
                    .under(table_node_id)
                    .with_meta(true)
                    .with_reserved_node_id(row_1_amount_id),
            })
            .with_edit(OxCalcTreeEdit::AddNode {
                request: OxCalcTreeNodeCreate::new("Amount_r2", "12")
                    .under(table_node_id)
                    .with_meta(true)
                    .with_reserved_node_id(row_2_amount_id),
            })
            .with_edit(OxCalcTreeEdit::SetNodeTable {
                node_id: table_node_id,
                snapshot,
            });
        let outcome = context.apply_edit_transaction(transaction).unwrap();

        assert_eq!(
            outcome.edit_results,
            vec![
                OxCalcTreeEditResult::NodeAdded {
                    node_id: row_1_amount_id
                },
                OxCalcTreeEditResult::NodeAdded {
                    node_id: row_2_amount_id
                },
                OxCalcTreeEditResult::Applied,
            ]
        );
        let table = context
            .table_view(&workspace_id, table_node_id)
            .unwrap()
            .expect("table projects after transaction");
        assert_eq!(table.snapshot.body_cell_nodes.len(), 2);
        assert!(
            table
                .snapshot
                .body_cell_nodes
                .iter()
                .any(|binding| binding.node_id == row_1_amount_id)
        );
        assert!(
            table
                .snapshot
                .body_cell_nodes
                .iter()
                .any(|binding| binding.node_id == row_2_amount_id)
        );
    }

    #[test]
    fn treecalc_context_edit_transaction_sets_node_meta_revisioned() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:transaction-set-meta",
            ))
            .unwrap();
        let node_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("Scratch", "1"))
            .unwrap();
        let before_revision = context.workspace_revision(&workspace_id).unwrap();

        let outcome = context
            .apply_edit_transaction(
                OxCalcTreeEditTransaction::new(workspace_id.clone())
                    .with_recalc_policy(TransactionRecalcPolicy::ApplyOnly)
                    .with_edit(OxCalcTreeEdit::SetNodeMeta {
                        node_id,
                        is_meta: true,
                    }),
            )
            .unwrap();

        assert_eq!(outcome.edit_results, vec![OxCalcTreeEditResult::Applied]);
        assert_ne!(
            outcome.workspace_revision_id,
            *before_revision.revision_id()
        );
        assert!(
            context
                .workspace_view(&workspace_id)
                .unwrap()
                .nodes
                .iter()
                .any(|node| node.node_id == node_id && node.is_meta)
        );

        let hidden_revision = outcome.workspace_revision_id;
        let shown = context
            .apply_edit_transaction(
                OxCalcTreeEditTransaction::new(workspace_id.clone())
                    .with_recalc_policy(TransactionRecalcPolicy::ApplyOnly)
                    .with_edit(OxCalcTreeEdit::SetNodeMeta {
                        node_id,
                        is_meta: false,
                    }),
            )
            .unwrap();

        assert_ne!(shown.workspace_revision_id, hidden_revision);
        assert!(
            context
                .workspace_view(&workspace_id)
                .unwrap()
                .nodes
                .iter()
                .any(|node| node.node_id == node_id && !node.is_meta)
        );
    }

    #[test]
    fn treecalc_context_edit_transaction_rolls_back_on_edit_failure() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:transaction-rollback",
            ))
            .unwrap();
        let a_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("A", "1"))
            .unwrap();
        let b_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("B", "=A+1"))
            .unwrap();
        let before_result = context.recalculate(&workspace_id).unwrap();
        let before_revision = context.workspace_revision(&workspace_id).unwrap();

        let transaction = OxCalcTreeEditTransaction::new(workspace_id.clone())
            .with_edit(OxCalcTreeEdit::SetNodeInput {
                node_id: a_id,
                input: "9".to_string(),
            })
            .with_edit(OxCalcTreeEdit::RenameNode {
                node_id: TreeNodeId(99_999),
                new_symbol: "Missing".to_string(),
            });
        let error = context
            .apply_edit_transaction(transaction)
            .expect_err("unknown node should reject the transaction");
        assert!(matches!(
            error,
            OxCalcTreeContextError::Structural(StructuralError::UnknownNode { .. })
        ));

        let after_revision = context.workspace_revision(&workspace_id).unwrap();
        assert_eq!(after_revision.revision_id(), before_revision.revision_id());
        let after_view = context.workspace_view(&workspace_id).unwrap();
        assert_eq!(
            after_view
                .nodes
                .iter()
                .find(|node| node.node_id == a_id)
                .and_then(|node| node.value_text.as_deref()),
            Some("1")
        );
        assert_eq!(
            context
                .workspace(&workspace_id)
                .unwrap()
                .last_result
                .as_ref()
                .map(|result| result.published_values.clone()),
            Some(before_result.published_values)
        );
        assert_eq!(
            context
                .apply_edit_transaction(
                    OxCalcTreeEditTransaction::new(workspace_id.clone())
                        .with_recalc_policy(TransactionRecalcPolicy::ApplyOnly)
                )
                .unwrap()
                .transaction_id
                .as_str(),
            "transaction:workspace:transaction-rollback:2"
        );
        assert!(
            after_view.nodes.iter().any(|node| node.node_id == b_id),
            "rollback must preserve existing dependent node"
        );
    }

    #[test]
    fn treecalc_context_edit_transaction_rolls_back_on_recalc_rejection() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:transaction-recalc-reject",
            ))
            .unwrap();
        let a_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("A", "1"))
            .unwrap();
        let b_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("B", "=A+1"))
            .unwrap();
        let before_result = context.recalculate(&workspace_id).unwrap();
        let before_revision = context.workspace_revision(&workspace_id).unwrap();

        // A self-cycle still rejects at recalc time (unlike an unresolved name,
        // which now commits as #NAME?), so it exercises rollback-on-recalc-
        // rejection.
        let transaction = OxCalcTreeEditTransaction::new(workspace_id.clone()).with_edit(
            OxCalcTreeEdit::SetNodeFormulaText {
                node_id: b_id,
                formula_text: "=B+1".to_string(),
            },
        );
        let error = context
            .apply_edit_transaction(transaction)
            .expect_err("cyclic formula should reject transactional recalc");
        assert!(matches!(
            error,
            OxCalcTreeContextError::TransactionRejected { .. }
        ));

        let after_revision = context.workspace_revision(&workspace_id).unwrap();
        assert_eq!(after_revision.revision_id(), before_revision.revision_id());
        let after_view = context.workspace_view(&workspace_id).unwrap();
        assert_eq!(
            after_view
                .nodes
                .iter()
                .find(|node| node.node_id == b_id)
                .map(|node| node.formula_text.as_str()),
            Some("=A+1")
        );
        assert_eq!(
            context
                .workspace(&workspace_id)
                .unwrap()
                .last_result
                .as_ref()
                .map(|result| result.published_values.clone()),
            Some(before_result.published_values)
        );
        assert_eq!(
            after_view
                .nodes
                .iter()
                .find(|node| node.node_id == a_id)
                .and_then(|node| node.value_text.as_deref()),
            Some("1")
        );
    }

    #[test]
    fn treecalc_context_edit_transaction_moves_and_edits_with_one_publication() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new("workspace:transaction-move"))
            .unwrap();
        let branch_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("Branch", ""))
            .unwrap();
        let a_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("A", "1"))
            .unwrap();
        let b_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("B", "=A+1"))
            .unwrap();
        let c_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("C", "3"))
            .unwrap();
        context.recalculate(&workspace_id).unwrap();

        let transaction = OxCalcTreeEditTransaction::new(workspace_id.clone())
            .with_edit(OxCalcTreeEdit::MoveNode {
                node_id: c_id,
                new_parent_id: branch_id,
                new_index: Some(0),
            })
            .with_edit(OxCalcTreeEdit::SetNodeInput {
                node_id: a_id,
                input: "5".to_string(),
            });
        let outcome = context.apply_edit_transaction(transaction).unwrap();

        let calculation = outcome
            .calculation
            .expect("transaction should recalculate once");
        assert_eq!(
            calculation
                .candidate_result
                .as_ref()
                .map(|candidate| candidate.candidate_result_id.as_str()),
            Some("candidate:workspace:transaction-move:2")
        );
        let view = context.workspace_view(&workspace_id).unwrap();
        assert_eq!(
            view.nodes
                .iter()
                .find(|node| node.node_id == c_id)
                .map(|node| node.display_path.as_str()),
            Some("Root/Branch/C")
        );
        assert_eq!(
            view.nodes
                .iter()
                .find(|node| node.node_id == a_id)
                .and_then(|node| node.value_text.as_deref()),
            Some("5")
        );
        assert_eq!(
            calculation.published_values.get(&b_id),
            Some(&"6".to_string())
        );
    }

    #[test]
    fn treecalc_context_edit_transaction_adds_nodes_and_returns_created_ids() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new("workspace:transaction-add"))
            .unwrap();
        let branch_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("Branch", ""))
            .unwrap();
        context.recalculate(&workspace_id).unwrap();

        let transaction = OxCalcTreeEditTransaction::new(workspace_id.clone())
            .with_edit(OxCalcTreeEdit::AddNode {
                request: OxCalcTreeNodeCreate::new("A", "2"),
            })
            .with_edit(OxCalcTreeEdit::AddNode {
                request: OxCalcTreeNodeCreate::new("Child", "=A+1").under(branch_id),
            });
        let outcome = context.apply_edit_transaction(transaction).unwrap();
        assert_eq!(outcome.edit_count, 2);
        assert_eq!(outcome.edit_results.len(), 2);
        let a_id = match outcome.edit_results[0] {
            OxCalcTreeEditResult::NodeAdded { node_id } => node_id,
            OxCalcTreeEditResult::Applied => panic!("first edit must return created node id"),
        };
        let child_id = match outcome.edit_results[1] {
            OxCalcTreeEditResult::NodeAdded { node_id } => node_id,
            OxCalcTreeEditResult::Applied => panic!("second edit must return created node id"),
        };
        let calculation = outcome
            .calculation
            .expect("transaction should recalculate once");
        assert_eq!(calculation.run_state, OxCalcTreeRunState::Published);
        assert_eq!(
            calculation
                .candidate_result
                .as_ref()
                .map(|candidate| candidate.candidate_result_id.as_str()),
            Some("candidate:workspace:transaction-add:2")
        );
        assert_eq!(
            calculation.published_values.get(&child_id),
            Some(&"3".to_string())
        );
        let view = context.workspace_view(&workspace_id).unwrap();
        assert_eq!(
            view.nodes
                .iter()
                .find(|node| node.node_id == a_id)
                .map(|node| node.display_path.as_str()),
            Some("Root/A")
        );
        assert_eq!(
            view.nodes
                .iter()
                .find(|node| node.node_id == child_id)
                .map(|node| node.display_path.as_str()),
            Some("Root/Branch/Child")
        );
    }

    #[test]
    fn treecalc_context_plans_invalidation_without_mutating_workspace() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new("workspace:preview-plan"))
            .unwrap();
        let a_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("A", "1"))
            .unwrap();
        let b_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("B", "=A+1"))
            .unwrap();
        let c_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("C", "=B+1"))
            .unwrap();
        context.recalculate(&workspace_id).unwrap();
        let before_revision = context.workspace_revision(&workspace_id).unwrap();

        let value_plan = context
            .plan_invalidation(
                &workspace_id,
                &[OxCalcTreePreviewMutation::SetNodeInput { node_id: a_id }],
            )
            .unwrap();
        assert_eq!(value_plan.estimated_node_count, 3);
        assert_eq!(
            value_plan
                .invalidated_nodes
                .iter()
                .map(|entry| entry.node_id)
                .collect::<Vec<_>>(),
            vec![a_id, b_id, c_id]
        );
        assert!(value_plan.requires_rebind.is_empty());
        assert_eq!(value_plan.evaluation_order, vec![b_id, c_id]);
        assert!(value_plan.cycle_risk.is_empty());

        let formula_plan = context
            .plan_invalidation(
                &workspace_id,
                &[OxCalcTreePreviewMutation::SetNodeFormulaText {
                    node_id: b_id,
                    formula_text: "=A+2".to_string(),
                }],
            )
            .unwrap();
        assert_eq!(formula_plan.estimated_node_count, 2);
        assert_eq!(formula_plan.requires_rebind, vec![b_id]);
        assert_eq!(formula_plan.evaluation_order, vec![b_id, c_id]);
        assert!(
            formula_plan
                .invalidated_nodes
                .iter()
                .find(|entry| entry.node_id == b_id)
                .is_some_and(|entry| entry
                    .reasons
                    .contains(&InvalidationReasonKind::StructuralRebindRequired))
        );

        let after_revision = context.workspace_revision(&workspace_id).unwrap();
        assert_eq!(after_revision.revision_id(), before_revision.revision_id());
    }

    #[test]
    fn treecalc_context_plans_table_snapshot_invalidation_without_mutating_workspace() {
        let (mut context, workspace_id, sales_id) =
            context_with_sales_table("workspace:table-preview-plan");
        let summary_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Summary", "=SUM(SalesTable[Amount])"),
            )
            .unwrap();
        context.recalculate(&workspace_id).unwrap();
        let before_revision = context.workspace_revision(&workspace_id).unwrap();
        let before_table = context
            .table_view(&workspace_id, sales_id)
            .unwrap()
            .unwrap();
        let mut preview_snapshot = before_table.snapshot.clone();
        let next_ordinal = preview_snapshot
            .columns
            .iter()
            .map(|column| column.ordinal)
            .max()
            .unwrap_or(0)
            .saturating_add(1);
        preview_snapshot.columns.push(TreeCalcTableColumnSnapshot {
            column_id: "col:discount".to_string(),
            column_name: "Discount".to_string(),
            ordinal: next_ordinal,
            body_metadata: TreeCalcTableColumnBodyMetadata::Formula(TreeCalcTableFormulaMetadata {
                formula_artifact_id: "formula:body:discount".to_string(),
                bind_artifact_id: Some("bind:body:discount".to_string()),
                formula_text_version: "v2".to_string(),
                formula_text: "=[@Amount]*0.05".to_string(),
            }),
            totals_metadata: None,
        });
        preview_snapshot.column_identity_version = "columns:v2".to_string();

        let plan = context
            .plan_invalidation(
                &workspace_id,
                &[OxCalcTreePreviewMutation::SetNodeTable {
                    node_id: sales_id,
                    snapshot: preview_snapshot,
                    scenario: TreeCalcTableUpdateScenarioKind::ColumnInsert,
                }],
            )
            .unwrap();

        let table_entry = plan
            .invalidated_nodes
            .iter()
            .find(|entry| entry.node_id == sales_id)
            .expect("table node should be invalidated");
        assert!(table_entry.requires_rebind);
        assert!(
            table_entry
                .reasons
                .contains(&InvalidationReasonKind::StructuredTableColumnChanged)
        );
        assert!(
            table_entry
                .reasons
                .contains(&InvalidationReasonKind::StructuredTableContextChanged)
        );
        assert!(plan.requires_rebind.contains(&sales_id));
        assert!(
            plan.invalidated_nodes
                .iter()
                .any(|entry| entry.node_id == summary_id),
            "structured-reference dependent should be planned from committed graph: {plan:?}"
        );

        let after_revision = context.workspace_revision(&workspace_id).unwrap();
        assert_eq!(after_revision.revision_id(), before_revision.revision_id());
        let after_table = context
            .table_view(&workspace_id, sales_id)
            .unwrap()
            .unwrap();
        assert_eq!(
            after_table.snapshot.columns.len(),
            before_table.snapshot.columns.len()
        );
        assert!(
            !after_table
                .snapshot
                .columns
                .iter()
                .any(|column| column.column_id == "col:discount")
        );
    }

    #[test]
    fn treecalc_context_dry_binds_formula_text_without_mutating_workspace() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new("workspace:dry-bind"))
            .unwrap();
        let a_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("A", "1"))
            .unwrap();
        let b_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("B", "=A+1"))
            .unwrap();
        context.recalculate(&workspace_id).unwrap();
        let before_revision = context.workspace_revision(&workspace_id).unwrap();

        let verdict = context
            .dry_bind_node_formula_text(&workspace_id, b_id, "=A+2")
            .unwrap();

        assert_eq!(verdict.node_id, b_id);
        assert_eq!(verdict.input_kind, OxCalcTreeDryBindInputKind::Formula);
        assert!(verdict.legal, "{verdict:?}");
        assert!(verdict.diagnostics.is_empty());
        assert!(verdict.profile_violations.is_empty());
        let after_revision = context.workspace_revision(&workspace_id).unwrap();
        assert_eq!(after_revision.revision_id(), before_revision.revision_id());
        assert_eq!(
            context
                .workspace_view(&workspace_id)
                .unwrap()
                .nodes
                .iter()
                .find(|node| node.node_id == b_id)
                .map(|node| node.formula_text.as_str()),
            Some("=A+1")
        );
        assert_eq!(
            context
                .workspace_view(&workspace_id)
                .unwrap()
                .nodes
                .iter()
                .find(|node| node.node_id == a_id)
                .and_then(|node| node.value_text.as_deref()),
            Some("1")
        );
    }

    #[test]
    fn treecalc_context_dry_bind_reports_syntax_and_bind_diagnostics() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new("workspace:dry-bind-diag"))
            .unwrap();
        let node_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("A", "1"))
            .unwrap();

        let syntax = context
            .dry_bind_node_formula_text(&workspace_id, node_id, "=1+")
            .unwrap();
        assert!(!syntax.legal);
        assert!(
            syntax
                .diagnostics
                .iter()
                .any(|diagnostic| { diagnostic.stage == OxCalcTreeDryBindDiagnosticStage::Syntax })
        );

        let bind = context
            .dry_bind_node_formula_text(&workspace_id, node_id, "=LAMBDA(x,x,x)")
            .unwrap();
        assert!(!bind.legal);
        assert!(bind.diagnostics.iter().any(|diagnostic| {
            diagnostic.stage == OxCalcTreeDryBindDiagnosticStage::Bind
                && diagnostic.message == "duplicate LAMBDA parameter name 'x'"
        }));
    }

    #[test]
    fn treecalc_context_dry_binds_new_node_formula_without_mutation() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:dry-bind-new-node",
            ))
            .unwrap();
        let root_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("Root", ""))
            .unwrap();
        context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("A", "1").under(root_id),
            )
            .unwrap();
        let before_revision = context.workspace_revision(&workspace_id).unwrap();

        let verdict = context
            .dry_bind_new_node_formula_text(
                &workspace_id,
                OxCalcTreeNodeCreate::new("B", "=A+1").under(root_id),
            )
            .unwrap();

        assert_eq!(verdict.input_kind, OxCalcTreeDryBindInputKind::Formula);
        assert!(verdict.legal, "{verdict:?}");
        assert!(verdict.diagnostics.is_empty());
        assert!(
            context
                .workspace_view(&workspace_id)
                .unwrap()
                .nodes
                .iter()
                .all(|node| node.symbol != "B")
        );
        assert_eq!(
            context.workspace_revision(&workspace_id).unwrap(),
            before_revision
        );
    }

    #[test]
    fn treecalc_context_dry_bind_new_node_reports_invalid_formula_without_mutation() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:dry-bind-new-node-invalid",
            ))
            .unwrap();
        let root_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("Root", ""))
            .unwrap();
        let before_revision = context.workspace_revision(&workspace_id).unwrap();

        let verdict = context
            .dry_bind_new_node_formula_text(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Broken", "=1+").under(root_id),
            )
            .unwrap();

        assert!(!verdict.legal, "{verdict:?}");
        assert!(
            verdict
                .diagnostics
                .iter()
                .any(|diagnostic| { diagnostic.stage == OxCalcTreeDryBindDiagnosticStage::Syntax })
        );
        assert!(
            context
                .workspace_view(&workspace_id)
                .unwrap()
                .nodes
                .iter()
                .all(|node| node.symbol != "Broken")
        );
        assert_eq!(
            context.workspace_revision(&workspace_id).unwrap(),
            before_revision
        );
    }

    #[test]
    fn treecalc_context_dry_binds_table_body_formula_with_current_row_context() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:dry-bind-table-body",
            ))
            .unwrap();
        let sales_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("SalesTable", ""))
            .unwrap();
        context
            .set_node_table(
                &workspace_id,
                sales_id,
                runtime_sales_table_snapshot(sales_id),
            )
            .unwrap();
        context.recalculate(&workspace_id).unwrap();
        let before_revision = context.workspace_revision(&workspace_id).unwrap();

        let verdict = context
            .dry_bind_table_column_formula_text(
                &workspace_id,
                sales_id,
                "col:tax",
                "=[@Amount]*0.2",
            )
            .unwrap();

        assert_eq!(verdict.node_id, sales_id);
        assert_eq!(verdict.input_kind, OxCalcTreeDryBindInputKind::Formula);
        assert!(verdict.legal, "{verdict:?}");
        assert!(verdict.diagnostics.is_empty(), "{verdict:?}");
        assert!(verdict.profile_violations.is_empty(), "{verdict:?}");
        let after_revision = context.workspace_revision(&workspace_id).unwrap();
        assert_eq!(after_revision.revision_id(), before_revision.revision_id());
        assert_eq!(
            context
                .table_view(&workspace_id, sales_id)
                .unwrap()
                .and_then(|view| {
                    view.snapshot.columns.into_iter().find_map(|column| {
                        if column.column_id == "col:tax" {
                            match column.body_metadata {
                                TreeCalcTableColumnBodyMetadata::Formula(formula) => {
                                    Some(formula.formula_text)
                                }
                                TreeCalcTableColumnBodyMetadata::ConstantCells => None,
                            }
                        } else {
                            None
                        }
                    })
                })
                .as_deref(),
            Some("=[@Amount]*0.1")
        );
    }

    #[test]
    fn treecalc_context_dry_binds_new_table_formula_column_without_mutating_shape() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:dry-bind-new-table-column",
            ))
            .unwrap();
        let sales_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("SalesTable", ""))
            .unwrap();
        context
            .set_node_table(
                &workspace_id,
                sales_id,
                runtime_sales_table_snapshot(sales_id),
            )
            .unwrap();
        context.recalculate(&workspace_id).unwrap();
        let before_revision = context.workspace_revision(&workspace_id).unwrap();
        let before_columns = context
            .table_view(&workspace_id, sales_id)
            .unwrap()
            .expect("table projects")
            .snapshot
            .columns
            .len();

        let verdict = context
            .dry_bind_new_table_column_formula_text(
                &workspace_id,
                sales_id,
                "col:double",
                "Double",
                "=[@Amount]*2",
            )
            .unwrap();

        assert_eq!(verdict.node_id, sales_id);
        assert_eq!(verdict.input_kind, OxCalcTreeDryBindInputKind::Formula);
        assert!(verdict.legal, "{verdict:?}");
        assert!(verdict.diagnostics.is_empty(), "{verdict:?}");
        assert!(verdict.profile_violations.is_empty(), "{verdict:?}");
        let after_revision = context.workspace_revision(&workspace_id).unwrap();
        assert_eq!(after_revision.revision_id(), before_revision.revision_id());
        let after_view = context
            .table_view(&workspace_id, sales_id)
            .unwrap()
            .expect("table still projects");
        assert_eq!(after_view.snapshot.columns.len(), before_columns);
        assert!(
            !after_view
                .snapshot
                .columns
                .iter()
                .any(|column| column.column_id == "col:double")
        );
    }

    #[test]
    fn treecalc_context_dry_binds_table_totals_formula_with_table_context() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:dry-bind-table-totals",
            ))
            .unwrap();
        let sales_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("SalesTable", ""))
            .unwrap();
        context
            .set_node_table(
                &workspace_id,
                sales_id,
                runtime_sales_table_snapshot(sales_id),
            )
            .unwrap();
        context.recalculate(&workspace_id).unwrap();

        let verdict = context
            .dry_bind_table_totals_formula_text(&workspace_id, sales_id, "col:tax", "=SUM([Tax])")
            .unwrap();

        assert_eq!(verdict.node_id, sales_id);
        assert_eq!(verdict.input_kind, OxCalcTreeDryBindInputKind::Formula);
        assert!(verdict.legal, "{verdict:?}");
        assert!(verdict.diagnostics.is_empty(), "{verdict:?}");

        let syntax = context
            .dry_bind_table_totals_formula_text(&workspace_id, sales_id, "col:tax", "=SUM([Tax]) +")
            .unwrap();
        assert!(!syntax.legal);
        assert!(
            syntax
                .diagnostics
                .iter()
                .any(|diagnostic| { diagnostic.stage == OxCalcTreeDryBindDiagnosticStage::Syntax })
        );
    }

    #[test]
    fn treecalc_context_edit_transaction_updates_table_snapshot() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:transaction-table",
            ))
            .unwrap();
        let sales_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("Sales", ""))
            .unwrap();
        context
            .set_node_table(&workspace_id, sales_id, sales_table_snapshot(sales_id))
            .unwrap();
        context.recalculate(&workspace_id).unwrap();

        let mut table = context
            .table_view(&workspace_id, sales_id)
            .unwrap()
            .expect("table projects before transaction")
            .snapshot;
        table.table_name = "Revenue".to_string();
        let transaction = OxCalcTreeEditTransaction::new(workspace_id.clone()).with_edit(
            OxCalcTreeEdit::SetNodeTable {
                node_id: sales_id,
                snapshot: table,
            },
        );
        let outcome = context.apply_edit_transaction(transaction).unwrap();

        assert_eq!(outcome.edit_results, vec![OxCalcTreeEditResult::Applied]);
        assert_eq!(
            outcome
                .calculation
                .as_ref()
                .map(|calculation| calculation.run_state),
            Some(OxCalcTreeRunState::VerifiedClean)
        );
        assert_eq!(
            context
                .table_view(&workspace_id, sales_id)
                .unwrap()
                .map(|view| view.table_name),
            Some("Revenue".to_string())
        );
    }

    #[test]
    fn treecalc_context_rejects_unparseable_authored_formula_input() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:parse-reject-input",
            ))
            .unwrap();
        let a_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("A", "7"))
            .unwrap();
        let before = context.export_workspace_snapshot(&workspace_id).unwrap();

        let err = context
            .set_node_input_value(&workspace_id, a_id, "=1+")
            .expect_err("parse diagnostics reject formula acceptance");
        assert!(matches!(
            err,
            OxCalcTreeContextError::AuthoredInputDiagnostics { node_id, .. } if node_id == a_id
        ));

        let after = context.export_workspace_snapshot(&workspace_id).unwrap();
        assert_eq!(
            exported_input_text(&after, a_id),
            exported_input_text(&before, a_id)
        );
        assert_eq!(
            exported_input_record(&after, a_id).kind,
            exported_input_record(&before, a_id).kind
        );
        assert_eq!(
            exported_input_epoch(&after, a_id),
            exported_input_epoch(&before, a_id)
        );
    }

    #[test]
    fn treecalc_context_rejects_unparseable_formula_edit_text() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:parse-reject-edit",
            ))
            .unwrap();
        let a_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("A", "=1+1"))
            .unwrap();
        let before = context.export_workspace_snapshot(&workspace_id).unwrap();

        let err = context
            .set_node_formula_text(&workspace_id, a_id, "=1+")
            .expect_err("parse diagnostics reject formula edit acceptance");
        assert!(matches!(
            err,
            OxCalcTreeContextError::AuthoredInputDiagnostics { node_id, .. } if node_id == a_id
        ));

        let after = context.export_workspace_snapshot(&workspace_id).unwrap();
        assert_eq!(
            exported_input_text(&after, a_id),
            exported_input_text(&before, a_id)
        );
        assert_eq!(
            exported_input_record(&after, a_id).kind,
            exported_input_record(&before, a_id).kind
        );
        assert_eq!(
            exported_input_epoch(&after, a_id),
            exported_input_epoch(&before, a_id)
        );
    }

    #[test]
    fn treecalc_context_formula_edit_same_host_reference_target_ignores_source_span_handle() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:host-reference-source-shift",
            ))
            .unwrap();
        context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("A", "3"))
            .unwrap();
        let b_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("B", "=A+1"))
            .unwrap();
        context.recalculate(&workspace_id).unwrap();

        context
            .set_node_formula_text(&workspace_id, b_id, "=0+A+2")
            .unwrap();
        let result = context.recalculate(&workspace_id).unwrap();

        assert_eq!(
            result.run_state,
            OxCalcTreeRunState::Published,
            "static-vs-ctro recalculation should publish: reject={:?}; diagnostics={:?}",
            result.reject_detail,
            result.diagnostics
        );
        assert_eq!(result.published_values.get(&b_id), Some(&"5".to_string()));
        assert!(result.diagnostics.iter().any(|diagnostic| {
            diagnostic == &format!("formula_edit_classification:{b_id}:same_dependencies")
        }));
        assert!(
            result
                .candidate_result
                .as_ref()
                .is_some_and(|candidate| candidate.dependency_shape_updates.is_empty())
        );
    }

    #[test]
    #[ignore = "same-shape collection edit with host structural selector needs a BoundFormula identity comparison update after carrier removal"]
    fn treecalc_context_formula_edit_same_collection_shape_ignores_source_span_handle() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:collection-source-shift",
            ))
            .unwrap();
        let total_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Total", "=SUM(Total.@CHILDREN)"),
            )
            .unwrap();
        context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("C1", "2").under(total_id),
            )
            .unwrap();
        context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("C2", "3").under(total_id),
            )
            .unwrap();
        context.recalculate(&workspace_id).unwrap();

        context
            .set_node_formula_text(&workspace_id, total_id, "=0+SUM(Total.@CHILDREN)")
            .unwrap();
        let result = context.recalculate(&workspace_id).unwrap();

        assert!(matches!(
            result.run_state,
            OxCalcTreeRunState::Published | OxCalcTreeRunState::VerifiedClean
        ));
        assert_eq!(
            result.published_values.get(&total_id),
            Some(&"5".to_string())
        );
        assert!(result.diagnostics.iter().any(|diagnostic| {
            diagnostic == &format!("formula_edit_classification:{total_id}:same_dependencies")
        }));
        assert!(
            result
                .candidate_result
                .as_ref()
                .is_none_or(|candidate| candidate.dependency_shape_updates.is_empty())
        );
    }

    #[test]
    fn treecalc_context_formula_edit_changed_dependency_preserves_structure_and_recalculates() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:formula-dependency-change",
            ))
            .unwrap();
        let a_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("A", "3"))
            .unwrap();
        let c_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("C", "10"))
            .unwrap();
        let b_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("B", "=A+1"))
            .unwrap();
        let d_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("D", "=B+1"))
            .unwrap();
        context.recalculate(&workspace_id).unwrap();
        let before_edit = context.workspace_view(&workspace_id).unwrap();
        let before_snapshot = context.export_workspace_snapshot(&workspace_id).unwrap();
        let before_b_formula_version = exported_input_epoch(&before_snapshot, b_id);

        context
            .set_node_formula_text(&workspace_id, b_id, "=C+1")
            .unwrap();
        let after_edit_before_recalc = context.workspace_view(&workspace_id).unwrap();
        let result = context.recalculate(&workspace_id).unwrap();
        let after_recalc = context.workspace_view(&workspace_id).unwrap();
        let after_snapshot = context.export_workspace_snapshot(&workspace_id).unwrap();

        assert_eq!(
            result.run_state,
            OxCalcTreeRunState::Published,
            "callable profile reference run failed: reject={:?}; diagnostics={:?}",
            result.reject_detail,
            result.diagnostics
        );
        assert_eq!(result.published_values.get(&b_id), Some(&"11".to_string()));
        assert_eq!(result.published_values.get(&d_id), Some(&"12".to_string()));
        assert_eq!(
            before_edit.snapshot_id,
            after_edit_before_recalc.snapshot_id
        );
        assert_eq!(before_edit.snapshot_id, after_recalc.snapshot_id);
        assert_eq!(before_edit.value_epoch, after_recalc.value_epoch);
        assert_eq!(
            exported_input_epoch(&after_snapshot, b_id),
            before_b_formula_version + 1
        );
        assert_eq!(
            after_snapshot
                .workspace_revision
                .structure_snapshot
                .snapshot_id(),
            before_snapshot
                .workspace_revision
                .structure_snapshot
                .snapshot_id()
        );
        assert!(
            result
                .invalidation_closure
                .records
                .get(&b_id)
                .is_some_and(|record| record
                    .reasons
                    .contains(&InvalidationReasonKind::StructuralRecalcOnly))
        );
        assert!(
            result
                .invalidation_closure
                .records
                .get(&d_id)
                .is_some_and(|record| record
                    .reasons
                    .contains(&InvalidationReasonKind::UpstreamPublication))
        );
        assert!(result.diagnostics.iter().any(|diagnostic| {
            diagnostic == &format!("formula_edit_classification:{b_id}:dependency_shape_changed")
        }));
        assert!(result.candidate_result.as_ref().is_some_and(|candidate| {
            candidate.dependency_shape_updates.iter().any(|update| {
                update.kind == "static_dependency_shape_changed"
                    && update.affected_node_ids.contains(&b_id)
                    && update.affected_node_ids.contains(&a_id)
                    && update.affected_node_ids.contains(&c_id)
            })
        }));
        assert!(
            result
                .publication_bundle
                .as_ref()
                .is_some_and(
                    |publication| publication
                        .dependency_shape_updates
                        .iter()
                        .any(|update| update.kind == "static_dependency_shape_changed"
                            && update.affected_node_ids.contains(&b_id)
                            && update.affected_node_ids.contains(&a_id)
                            && update.affected_node_ids.contains(&c_id))
                )
        );
        assert!(a_id.0 > 0);
        assert!(c_id.0 > 0);
    }

    #[test]
    fn treecalc_context_formula_edit_cycle_reject_preserves_structure_and_prior_publication() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:formula-cycle-edit",
            ))
            .unwrap();
        let a_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("A", "1"))
            .unwrap();
        let b_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("B", "=A+1"))
            .unwrap();
        let initial = context.recalculate(&workspace_id).unwrap();
        assert_eq!(initial.published_values.get(&b_id), Some(&"2".to_string()));
        let before_edit = context.workspace_view(&workspace_id).unwrap();
        let before_publication_snapshot_id = before_edit.publication_snapshot_id.clone();
        let before_runtime_overlay_set_id = before_edit.runtime_overlay_set_id.clone();

        context
            .set_node_formula_text(&workspace_id, b_id, "=B+1")
            .unwrap();
        let after_edit_before_recalc = context.workspace_view(&workspace_id).unwrap();
        let result = context.recalculate(&workspace_id).unwrap();
        let after_recalc = context.workspace_view(&workspace_id).unwrap();

        assert_eq!(result.run_state, OxCalcTreeRunState::Rejected);
        assert_eq!(
            result.reject_detail.as_ref().map(|detail| detail.kind),
            Some(RejectKind::SyntheticCycleReject)
        );
        assert_eq!(
            before_edit.snapshot_id,
            after_edit_before_recalc.snapshot_id
        );
        assert_eq!(before_edit.snapshot_id, after_recalc.snapshot_id);
        assert_eq!(before_edit.value_epoch, after_recalc.value_epoch);
        assert_eq!(result.published_values.get(&b_id), Some(&"2".to_string()));
        assert!(result.diagnostics.iter().any(|diagnostic| {
            diagnostic == &format!("formula_edit_classification:{b_id}:cycle_candidate")
        }));
        assert!(result.publication_bundle.is_none());
        assert_eq!(
            after_recalc.publication_snapshot_id,
            before_publication_snapshot_id
        );
        assert_eq!(
            after_recalc.runtime_overlay_set_id,
            before_runtime_overlay_set_id
        );
        assert!(
            after_recalc
                .dependency_shape_snapshot_id
                .0
                .contains("absent")
        );
        assert!(a_id.0 > 0);
    }

    #[test]
    fn treecalc_context_formula_edit_unresolved_to_resolved_heals_and_publishes() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:formula-unresolved-resolved",
            ))
            .unwrap();
        let a_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("A", "3"))
            .unwrap();
        let b_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("B", "=Missing+1"))
            .unwrap();
        // Excel-faithful: an unresolved name commits with #NAME? (it is not
        // rejected) -- a tree node is a defined name, so a missing name behaves
        // like Excel's #NAME?.
        let initial = context.recalculate(&workspace_id).unwrap();
        assert_eq!(initial.run_state, OxCalcTreeRunState::Published);
        assert_eq!(
            initial.published_values.get(&b_id),
            Some(&"#NAME?".to_string())
        );

        // Editing the reference to a resolvable name heals it to a value.
        context
            .set_node_formula_text(&workspace_id, b_id, "=A+1")
            .unwrap();
        let result = context.recalculate(&workspace_id).unwrap();

        assert_eq!(result.run_state, OxCalcTreeRunState::Published);
        assert_eq!(result.published_values.get(&b_id), Some(&"4".to_string()));
        assert!(result.diagnostics.iter().any(|diagnostic| {
            diagnostic == &format!("formula_edit_classification:{b_id}:unresolved_to_resolved")
        }));
        assert!(a_id.0 > 0);
    }

    #[test]
    fn treecalc_context_formula_edit_resolved_to_unresolved_commits_with_name_error() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:formula-resolved-unresolved",
            ))
            .unwrap();
        let a_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("A", "3"))
            .unwrap();
        let b_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("B", "=A+1"))
            .unwrap();
        let initial = context.recalculate(&workspace_id).unwrap();
        assert_eq!(initial.published_values.get(&b_id), Some(&"4".to_string()));

        context
            .set_node_formula_text(&workspace_id, b_id, "=Missing+1")
            .unwrap();
        let result = context.recalculate(&workspace_id).unwrap();

        // Excel-faithful: editing a formula to reference an unknown name commits
        // with #NAME? (it is not rejected; the prior value is not preserved).
        assert_eq!(result.run_state, OxCalcTreeRunState::Published);
        assert_eq!(
            result.published_values.get(&b_id),
            Some(&"#NAME?".to_string())
        );
        assert!(result.publication_bundle.is_some());
        // The edit is still classified as resolved -> unresolved.
        assert!(result.diagnostics.iter().any(|diagnostic| {
            diagnostic == &format!("formula_edit_classification:{b_id}:resolved_to_unresolved")
        }));
        assert!(a_id.0 > 0);
    }

    #[test]
    fn treecalc_context_delete_referenced_node_commits_with_name_error_and_heals_on_readd() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:delete-referenced-node",
            ))
            .unwrap();
        let a_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("A", "3"))
            .unwrap();
        let b_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("B", "=A+1"))
            .unwrap();
        let initial = context.recalculate(&workspace_id).unwrap();
        assert_eq!(initial.published_values.get(&b_id), Some(&"4".to_string()));

        // Deleting the referenced node orphans B's reference. Excel-faithful: a
        // tree node is a defined name, so the now-missing name evaluates to
        // #NAME? and the recalc COMMITS the change (it is not rejected, and the
        // formula text "=A+1" is left unchanged).
        context.delete_node(&workspace_id, a_id).unwrap();
        let after_delete = context.recalculate(&workspace_id).unwrap();
        assert_eq!(after_delete.run_state, OxCalcTreeRunState::Published);
        assert_eq!(
            after_delete.published_values.get(&b_id),
            Some(&"#NAME?".to_string())
        );
        assert!(after_delete.publication_bundle.is_some());

        // Re-adding the name heals the reference: B's unchanged formula resolves
        // against the new node (Excel re-creates a deleted defined name the same
        // way).
        context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("A", "10"))
            .unwrap();
        let after_readd = context.recalculate(&workspace_id).unwrap();
        assert_eq!(after_readd.run_state, OxCalcTreeRunState::Published);
        assert_eq!(
            after_readd.published_values.get(&b_id),
            Some(&"11".to_string())
        );
    }

    #[test]
    fn treecalc_context_literal_to_formula_preserves_structure_and_publishes_activation() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:literal-to-formula",
            ))
            .unwrap();
        let a_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("A", "3"))
            .unwrap();
        let c_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("C", "10"))
            .unwrap();
        let b_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("B", "=A+1"))
            .unwrap();
        context.recalculate(&workspace_id).unwrap();
        let before_edit = context.workspace_view(&workspace_id).unwrap();
        let before_snapshot = context.export_workspace_snapshot(&workspace_id).unwrap();
        let before_a_formula_version = exported_input_epoch(&before_snapshot, a_id);

        context
            .set_node_formula_text(&workspace_id, a_id, "=C+1")
            .unwrap();
        let after_edit_before_recalc = context.workspace_view(&workspace_id).unwrap();
        let after_edit_a = context.node_view(&workspace_id, a_id).unwrap();
        let result = context.recalculate(&workspace_id).unwrap();
        let after_recalc = context.workspace_view(&workspace_id).unwrap();
        let after_snapshot = context.export_workspace_snapshot(&workspace_id).unwrap();

        assert_eq!(result.run_state, OxCalcTreeRunState::Published);
        assert_eq!(result.published_values.get(&a_id), Some(&"11".to_string()));
        assert_eq!(result.published_values.get(&b_id), Some(&"12".to_string()));
        assert_eq!(
            before_edit.snapshot_id,
            after_edit_before_recalc.snapshot_id
        );
        assert_eq!(before_edit.snapshot_id, after_recalc.snapshot_id);
        assert_eq!(before_edit.value_epoch, after_recalc.value_epoch);
        assert_eq!(after_edit_a.formula_text, "=C+1");
        assert_eq!(after_edit_a.value_text.as_deref(), Some("3"));
        assert_eq!(after_edit_a.input_value_epoch, None);
        assert_eq!(
            exported_input_epoch(&after_snapshot, a_id),
            before_a_formula_version + 1
        );
        assert_eq!(
            exported_input_record(&after_snapshot, a_id).kind,
            NodeInputKind::FormulaText
        );
        assert_eq!(
            after_snapshot.workspace_revision.structure_snapshot,
            before_snapshot.workspace_revision.structure_snapshot
        );
        assert!(result.diagnostics.iter().any(|diagnostic| {
            diagnostic == &format!("formula_edit_classification:{a_id}:literal_to_formula")
        }));
        assert!(result.diagnostics.iter().any(|diagnostic| {
            diagnostic == &format!("formula_input_kind_transition:{a_id}:literal->formula-text")
        }));
        assert!(
            result
                .publication_bundle
                .as_ref()
                .is_some_and(
                    |publication| publication
                        .dependency_shape_updates
                        .iter()
                        .any(
                            |update| update.kind == "static_formula_dependency_activated"
                                && update.affected_node_ids.contains(&a_id)
                                && update.affected_node_ids.contains(&c_id)
                        )
                )
        );
    }

    #[test]
    fn treecalc_context_formula_to_literal_preserves_structure_and_publishes_release() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:formula-to-literal",
            ))
            .unwrap();
        let c_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("C", "10"))
            .unwrap();
        let a_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("A", "=C+1"))
            .unwrap();
        let b_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("B", "=A+1"))
            .unwrap();
        context.recalculate(&workspace_id).unwrap();
        let before_edit = context.workspace_view(&workspace_id).unwrap();
        let before_snapshot = context.export_workspace_snapshot(&workspace_id).unwrap();
        let before_a_formula_version = exported_input_epoch(&before_snapshot, a_id);

        context
            .set_node_formula_text(&workspace_id, a_id, "7")
            .unwrap();
        let after_edit_before_recalc = context.workspace_view(&workspace_id).unwrap();
        let after_edit_a = context.node_view(&workspace_id, a_id).unwrap();
        let result = context.recalculate(&workspace_id).unwrap();
        let after_recalc = context.workspace_view(&workspace_id).unwrap();
        let after_recalc_a = context.node_view(&workspace_id, a_id).unwrap();
        let after_snapshot = context.export_workspace_snapshot(&workspace_id).unwrap();

        assert_eq!(result.run_state, OxCalcTreeRunState::Published);
        assert_eq!(result.published_values.get(&b_id), Some(&"8".to_string()));
        assert_eq!(
            before_edit.snapshot_id,
            after_edit_before_recalc.snapshot_id
        );
        assert_eq!(before_edit.snapshot_id, after_recalc.snapshot_id);
        assert_ne!(
            before_edit.node_input_snapshot_id,
            after_edit_before_recalc.node_input_snapshot_id
        );
        assert_eq!(
            after_edit_before_recalc.node_input_snapshot_id,
            after_recalc.node_input_snapshot_id
        );
        assert_ne!(
            before_edit.workspace_revision_id,
            after_edit_before_recalc.workspace_revision_id
        );
        assert_eq!(
            after_edit_before_recalc.workspace_revision_id,
            after_recalc.workspace_revision_id
        );
        assert_eq!(
            after_edit_before_recalc.value_epoch,
            before_edit.value_epoch + 1
        );
        assert_eq!(
            after_recalc.value_epoch,
            after_edit_before_recalc.value_epoch
        );
        assert_eq!(after_edit_a.formula_text, "7");
        assert_eq!(after_edit_a.value_text.as_deref(), Some("7"));
        assert_eq!(after_edit_a.calc_value, Some(CalcValue::number(7.0)));
        assert_eq!(
            after_edit_a.input_value_epoch,
            Some(after_edit_before_recalc.value_epoch)
        );
        assert_eq!(after_recalc_a.value_text.as_deref(), Some("7"));
        assert_eq!(after_recalc_a.calc_value, Some(CalcValue::number(7.0)));
        assert_eq!(
            exported_input_epoch(&after_snapshot, a_id),
            before_a_formula_version + 1
        );
        assert_eq!(exported_input_text(&after_snapshot, a_id), Some("7"));
        assert_eq!(
            exported_input_record(&after_snapshot, a_id).kind,
            NodeInputKind::Literal
        );
        assert_eq!(
            after_snapshot.workspace_revision.structure_snapshot,
            before_snapshot.workspace_revision.structure_snapshot
        );
        assert!(result.diagnostics.iter().any(|diagnostic| {
            diagnostic == &format!("formula_edit_classification:{a_id}:formula_to_literal")
        }));
        assert!(result.diagnostics.iter().any(|diagnostic| {
            diagnostic == &format!("formula_input_kind_transition:{a_id}:formula-text->literal")
        }));
        assert!(
            result
                .publication_bundle
                .as_ref()
                .is_some_and(
                    |publication| publication
                        .dependency_shape_updates
                        .iter()
                        .any(|update| update.kind == "static_formula_dependency_released"
                            && update.affected_node_ids.contains(&a_id)
                            && update.affected_node_ids.contains(&c_id))
                )
        );
        assert!(
            result
                .invalidation_closure
                .records
                .get(&b_id)
                .is_some_and(|record| record
                    .reasons
                    .contains(&InvalidationReasonKind::UpstreamPublication))
        );
    }

    #[test]
    fn treecalc_context_stamps_per_node_published_value_epochs() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:published-value-epochs",
            ))
            .unwrap();
        let a_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("A", "=2"))
            .unwrap();
        let b_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("B", "=A+1"))
            .unwrap();
        let c_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("C", "=100"))
            .unwrap();

        let initial = context.recalculate(&workspace_id).unwrap();
        assert_eq!(initial.run_state, OxCalcTreeRunState::Published);
        let initial_a_epoch = initial.published_value_epochs[&a_id];
        let initial_b_epoch = initial.published_value_epochs[&b_id];
        let initial_c_epoch = initial.published_value_epochs[&c_id];
        assert_ne!(initial_a_epoch, initial_b_epoch);
        assert_ne!(initial_b_epoch, initial_c_epoch);

        context
            .set_node_formula_text(&workspace_id, a_id, "=3")
            .unwrap();
        let edited = context.recalculate(&workspace_id).unwrap();
        assert_eq!(edited.run_state, OxCalcTreeRunState::Published);
        assert_eq!(edited.published_values.get(&a_id), Some(&"3".to_string()));
        assert_eq!(edited.published_values.get(&b_id), Some(&"4".to_string()));
        assert_eq!(edited.published_values.get(&c_id), Some(&"100".to_string()));
        assert_ne!(edited.published_value_epochs[&a_id], initial_a_epoch);
        assert_ne!(edited.published_value_epochs[&b_id], initial_b_epoch);
        assert_eq!(edited.published_value_epochs[&c_id], initial_c_epoch);

        let after_edit_a = context.node_view(&workspace_id, a_id).unwrap();
        let after_edit_b = context.node_view(&workspace_id, b_id).unwrap();
        let after_edit_c = context.node_view(&workspace_id, c_id).unwrap();
        assert_eq!(
            after_edit_a.published_value_epoch,
            Some(edited.published_value_epochs[&a_id])
        );
        assert_eq!(
            after_edit_b.published_value_epoch,
            Some(edited.published_value_epochs[&b_id])
        );
        assert_eq!(after_edit_c.published_value_epoch, Some(initial_c_epoch));
        assert_eq!(after_edit_a.input_value_epoch, None);

        let snapshot = context.export_workspace_snapshot(&workspace_id).unwrap();
        assert_eq!(
            snapshot.publication_value_epochs.get(&c_id).copied(),
            Some(initial_c_epoch)
        );
        let mut imported_context = OxCalcTreeContext::default();
        let imported_workspace_id = imported_context
            .import_workspace_snapshot(snapshot)
            .unwrap();
        assert_eq!(
            imported_context
                .node_view(&imported_workspace_id, c_id)
                .unwrap()
                .published_value_epoch,
            Some(initial_c_epoch)
        );
    }

    #[test]
    fn treecalc_context_empty_formula_transitions_record_input_kind_changes() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:empty-formula-transitions",
            ))
            .unwrap();
        let c_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("C", "10"))
            .unwrap();
        let a_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("A", ""))
            .unwrap();
        let before_formula = context.workspace_view(&workspace_id).unwrap();

        context
            .set_node_formula_text(&workspace_id, a_id, "=C+1")
            .unwrap();
        let formula_edit = context.workspace_view(&workspace_id).unwrap();
        let formula_result = context.recalculate(&workspace_id).unwrap();
        let formula_revision = context.workspace_revision(&workspace_id).unwrap();

        assert_eq!(formula_result.run_state, OxCalcTreeRunState::Published);
        assert_eq!(
            formula_result.published_values.get(&a_id),
            Some(&"11".to_string())
        );
        assert_eq!(before_formula.snapshot_id, formula_edit.snapshot_id);
        assert_ne!(
            before_formula.node_input_snapshot_id,
            formula_edit.node_input_snapshot_id
        );
        assert_eq!(
            formula_revision
                .node_input_snapshot
                .try_get_record(a_id)
                .map(|record| record.kind),
            Some(NodeInputKind::FormulaText)
        );
        assert!(formula_result.diagnostics.iter().any(|diagnostic| {
            diagnostic == &format!("formula_input_kind_transition:{a_id}:empty->formula-text")
        }));

        let before_empty = context.workspace_view(&workspace_id).unwrap();
        context
            .set_node_formula_text(&workspace_id, a_id, "")
            .unwrap();
        let empty_edit = context.workspace_view(&workspace_id).unwrap();
        let empty_result = context.recalculate(&workspace_id).unwrap();
        let empty_revision = context.workspace_revision(&workspace_id).unwrap();

        assert_eq!(empty_result.run_state, OxCalcTreeRunState::Published);
        assert_eq!(before_empty.snapshot_id, empty_edit.snapshot_id);
        assert_ne!(
            before_empty.node_input_snapshot_id,
            empty_edit.node_input_snapshot_id
        );
        assert_eq!(empty_edit.value_epoch, before_empty.value_epoch + 1);
        assert_eq!(
            empty_revision
                .node_input_snapshot
                .try_get_record(a_id)
                .map(|record| record.kind),
            Some(NodeInputKind::Empty)
        );
        assert!(empty_result.diagnostics.iter().any(|diagnostic| {
            diagnostic == &format!("formula_input_kind_transition:{a_id}:formula-text->empty")
        }));
        assert!(
            empty_result
                .publication_bundle
                .as_ref()
                .is_some_and(
                    |publication| publication
                        .dependency_shape_updates
                        .iter()
                        .any(|update| update.kind == "static_formula_dependency_released"
                            && update.affected_node_ids.contains(&a_id)
                            && update.affected_node_ids.contains(&c_id))
                )
        );
    }

    #[test]
    fn treecalc_context_structural_reset_clears_pending_formula_transition_facts() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:structural-clears-formula-transition",
            ))
            .unwrap();
        let a_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("A", "1"))
            .unwrap();
        let b_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("B", "=A+1"))
            .unwrap();
        context.recalculate(&workspace_id).unwrap();

        context
            .set_node_formula_text(&workspace_id, b_id, "=A+2")
            .unwrap();
        context.rename_node(&workspace_id, a_id, "X").unwrap();
        let result = context.recalculate(&workspace_id).unwrap();

        assert!(!result.diagnostics.iter().any(|diagnostic| {
            diagnostic == &format!("formula_edit_classification:{b_id}:same_dependencies")
        }));
        assert!(!result.diagnostics.iter().any(|diagnostic| {
            diagnostic
                == &format!("formula_input_kind_transition:{b_id}:formula-text->formula-text")
        }));
    }

    #[test]
    fn treecalc_context_literal_to_formula_cycle_reject_preserves_prior_literal_value() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:literal-formula-cycle",
            ))
            .unwrap();
        let a_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("A", "3"))
            .unwrap();
        let b_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("B", "=A+1"))
            .unwrap();
        context.recalculate(&workspace_id).unwrap();
        context
            .set_node_input_value(&workspace_id, a_id, "4")
            .unwrap();
        let input_result = context.recalculate(&workspace_id).unwrap();
        assert_eq!(
            input_result.published_values.get(&b_id),
            Some(&"5".to_string())
        );
        let before_edit = context.workspace_view(&workspace_id).unwrap();

        context
            .set_node_formula_text(&workspace_id, a_id, "=B+1")
            .unwrap();
        let after_edit_before_recalc = context.workspace_view(&workspace_id).unwrap();
        let after_edit_a = context.node_view(&workspace_id, a_id).unwrap();
        let result = context.recalculate(&workspace_id).unwrap();
        let after_reject_a = context.node_view(&workspace_id, a_id).unwrap();

        assert_eq!(result.run_state, OxCalcTreeRunState::Rejected);
        assert_eq!(
            before_edit.snapshot_id,
            after_edit_before_recalc.snapshot_id
        );
        assert_eq!(
            before_edit.value_epoch,
            after_edit_before_recalc.value_epoch
        );
        assert_eq!(after_edit_a.formula_text, "=B+1");
        assert_eq!(after_edit_a.value_text.as_deref(), Some("4"));
        assert_eq!(after_edit_a.input_value_epoch, None);
        assert_eq!(result.published_values.get(&a_id), Some(&"4".to_string()));
        assert_eq!(result.published_values.get(&b_id), Some(&"5".to_string()));
        assert_eq!(
            result.reject_detail.as_ref().map(|detail| detail.kind),
            Some(RejectKind::SyntheticCycleReject)
        );
        assert!(result.diagnostics.iter().any(|diagnostic| {
            diagnostic == &format!("formula_edit_classification:{a_id}:cycle_candidate")
        }));
        assert!(result.diagnostics.iter().any(|diagnostic| {
            diagnostic == &format!("formula_input_kind_transition:{a_id}:literal->formula-text")
        }));
        assert!(result.publication_bundle.is_none());
        assert_eq!(after_reject_a.value_text.as_deref(), Some("4"));
    }

    #[test]
    fn treecalc_context_input_value_update_recalculates_dependents_without_full_reset() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new("workspace:input-value"))
            .unwrap();
        let a_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("A", "3"))
            .unwrap();
        let b_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("B", "=A+1"))
            .unwrap();
        context.recalculate(&workspace_id).unwrap();
        let before_edit = context.workspace_view(&workspace_id).unwrap();
        let before_a = context.node_view(&workspace_id, a_id).unwrap();
        let before_snapshot = context.export_workspace_snapshot(&workspace_id).unwrap();

        context
            .set_node_input_value(&workspace_id, a_id, "4")
            .unwrap();
        let after_edit_before_recalc = context.workspace_view(&workspace_id).unwrap();
        let result = context.recalculate(&workspace_id).unwrap();
        let after_recalc = context.workspace_view(&workspace_id).unwrap();
        let after_recalc_revision = context.workspace_revision(&workspace_id).unwrap();

        assert_eq!(result.run_state, OxCalcTreeRunState::Published);
        assert_eq!(result.published_values.get(&b_id), Some(&"5".to_string()));
        assert_eq!(
            before_edit.snapshot_id,
            after_edit_before_recalc.snapshot_id
        );
        assert_eq!(before_edit.snapshot_id, after_recalc.snapshot_id);
        assert_ne!(
            before_edit.node_input_snapshot_id,
            after_edit_before_recalc.node_input_snapshot_id
        );
        assert_eq!(
            after_edit_before_recalc.node_input_snapshot_id,
            after_recalc.node_input_snapshot_id
        );
        assert_ne!(
            before_edit.workspace_revision_id,
            after_edit_before_recalc.workspace_revision_id
        );
        assert_eq!(
            after_edit_before_recalc.workspace_revision_id,
            after_recalc.workspace_revision_id
        );
        assert_eq!(
            after_edit_before_recalc.value_epoch,
            before_edit.value_epoch + 1
        );
        assert_eq!(before_a.input_value_epoch, Some(before_edit.value_epoch));
        assert_eq!(
            context
                .node_view(&workspace_id, a_id)
                .unwrap()
                .value_text
                .as_deref(),
            Some("4")
        );
        assert_eq!(
            context
                .node_view(&workspace_id, a_id)
                .unwrap()
                .input_value_epoch,
            Some(after_edit_before_recalc.value_epoch)
        );
        let a_input_record = after_recalc_revision
            .node_input_snapshot
            .try_get_record(a_id)
            .expect("A should keep an input record after literal edit");
        assert_eq!(a_input_record.kind, NodeInputKind::Literal);
        assert_eq!(a_input_record.text.as_deref(), Some("4"));
        assert_eq!(a_input_record.input_epoch, after_recalc.value_epoch);
        assert!(
            result
                .invalidation_closure
                .records
                .get(&a_id)
                .is_some_and(|record| record
                    .reasons
                    .contains(&InvalidationReasonKind::UpstreamPublication))
        );
        assert!(
            result
                .invalidation_closure
                .records
                .get(&b_id)
                .is_some_and(|record| record
                    .reasons
                    .contains(&InvalidationReasonKind::UpstreamPublication))
        );
        assert!(result.diagnostics.iter().any(|diagnostic| {
            diagnostic == &format!("edge_value_cache_bypass:{b_id}:UpstreamPublication")
        }));
        assert!(
            !result
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.contains("formula_edit_classification")),
            "literal input edits must not enter the formula edit classifier: {:?}",
            result.diagnostics
        );
        let exported = context.export_workspace_snapshot(&workspace_id).unwrap();
        assert_eq!(
            exported.workspace_revision.structure_snapshot.snapshot_id(),
            before_edit.snapshot_id
        );
        assert_eq!(exported_input_text(&exported, a_id), Some("4"));
        assert_eq!(
            exported.workspace_revision.structure_snapshot,
            before_snapshot.workspace_revision.structure_snapshot
        );
        assert!(matches!(
            context
                .set_node_input_value(&workspace_id, b_id, "9")
                .unwrap_err(),
            OxCalcTreeContextError::InputValueOnFormulaNode { .. }
        ));
        context
            .set_node_input_value(&workspace_id, a_id, "=9")
            .unwrap();
        let formula_input_record = context
            .export_workspace_snapshot(&workspace_id)
            .unwrap()
            .workspace_revision
            .node_input_snapshot
            .try_get_record(a_id)
            .cloned()
            .expect("A should have formula input after leading-equals input edit");
        assert_eq!(formula_input_record.kind, NodeInputKind::FormulaText);
        assert_eq!(formula_input_record.text.as_deref(), Some("=9"));
    }

    #[test]
    fn treecalc_context_clear_input_value_uses_empty_node_input_snapshot() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new("workspace:input-clear"))
            .unwrap();
        let a_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("A", "3"))
            .unwrap();
        let b_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("B", "=A+1"))
            .unwrap();
        context.recalculate(&workspace_id).unwrap();
        let before_clear = context.workspace_view(&workspace_id).unwrap();

        context.clear_node_input_value(&workspace_id, a_id).unwrap();
        let after_clear = context.workspace_view(&workspace_id).unwrap();
        let after_clear_revision = context.workspace_revision(&workspace_id).unwrap();
        let after_clear_a = context.node_view(&workspace_id, a_id).unwrap();
        let exported = context.export_workspace_snapshot(&workspace_id).unwrap();

        assert_eq!(before_clear.snapshot_id, after_clear.snapshot_id);
        assert_ne!(
            before_clear.node_input_snapshot_id,
            after_clear.node_input_snapshot_id
        );
        assert_ne!(
            before_clear.workspace_revision_id,
            after_clear.workspace_revision_id
        );
        assert_eq!(after_clear.value_epoch, before_clear.value_epoch + 1);
        let a_input_record = after_clear_revision
            .node_input_snapshot
            .try_get_record(a_id)
            .expect("A should keep an empty input record after clear");
        assert_eq!(a_input_record.kind, NodeInputKind::Empty);
        assert_eq!(a_input_record.text, None);
        assert_eq!(a_input_record.input_epoch, after_clear.value_epoch);
        assert_eq!(after_clear_a.formula_text, "");
        assert_eq!(after_clear_a.value_text, None);
        assert_eq!(after_clear_a.input_value_epoch, None);
        assert_eq!(
            exported_input_record(&exported, a_id).kind,
            NodeInputKind::Empty
        );
        assert_eq!(exported_input_record(&exported, a_id).text, None);
        assert!(matches!(
            context
                .clear_node_input_value(&workspace_id, b_id)
                .unwrap_err(),
            OxCalcTreeContextError::InputValueOnFormulaNode { .. }
        ));
    }

    #[test]
    fn snapshot_calc_value_round_trips_scalars_and_arrays() {
        use oxfunc_core::value::{
            ArrayShape, CalcArray, CalcValue, CoreValue, ExcelText, WorksheetErrorCode,
        };

        // A 2x3 array of mixed scalars — exactly the case the old display-string
        // format destroyed (it stored only the text "Array(2x3)" and rebuilt a
        // 1x1 text literal on import).
        let array = CalcValue::new(CoreValue::Array(
            CalcArray::new(
                ArrayShape { rows: 2, cols: 3 },
                vec![
                    CalcValue::number(1.0),
                    CalcValue::text(ExcelText::from_interop_assignment("hi")),
                    CalcValue::logical(true),
                    CalcValue::error(WorksheetErrorCode::Div0),
                    CalcValue::empty(),
                    CalcValue::number(6.5),
                ],
            )
            .unwrap(),
        ));

        for original in [
            CalcValue::number(42.0),
            CalcValue::text(ExcelText::from_interop_assignment("hello")),
            CalcValue::logical(false),
            CalcValue::error(WorksheetErrorCode::Value),
            CalcValue::empty(),
            CalcValue::new(CoreValue::Missing),
            array,
        ] {
            let snapshot = SnapshotCalcValue::from_calc_value(&original);
            // Survives a serde JSON round-trip ...
            let json = serde_json::to_string(&snapshot).unwrap();
            let reparsed: SnapshotCalcValue = serde_json::from_str(&json).unwrap();
            assert_eq!(reparsed, snapshot);
            // ... and reconstructs the original CalcValue exactly (shape + data).
            assert_eq!(reparsed.to_calc_value(), original);
        }
    }

    #[test]
    fn treecalc_context_input_truth_roundtrips_through_snapshot_layer() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:input-authority-guard",
            ))
            .unwrap();
        let a_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("A", "3"))
            .unwrap();
        let b_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("B", "=A+1"))
            .unwrap();
        context.recalculate(&workspace_id).unwrap();

        let exported = context.export_workspace_snapshot(&workspace_id).unwrap();
        assert_eq!(exported_input_text(&exported, a_id), Some("3"));

        let mut imported_context = OxCalcTreeContext::default();
        let imported_workspace_id = imported_context
            .import_workspace_snapshot(exported.clone())
            .unwrap();
        assert_eq!(
            imported_context
                .node_view(&imported_workspace_id, a_id)
                .unwrap()
                .value_text
                .as_deref(),
            Some("3")
        );

        imported_context
            .set_node_input_value(&imported_workspace_id, a_id, "4")
            .unwrap();
        let result = imported_context
            .recalculate(&imported_workspace_id)
            .unwrap();
        let reexported = imported_context
            .export_workspace_snapshot(&imported_workspace_id)
            .unwrap();

        assert_eq!(result.run_state, OxCalcTreeRunState::Published);
        assert_eq!(result.published_values.get(&b_id), Some(&"5".to_string()));
        assert_eq!(
            imported_context
                .node_view(&imported_workspace_id, a_id)
                .unwrap()
                .value_text
                .as_deref(),
            Some("4")
        );
        assert_eq!(exported_input_text(&reexported, a_id), Some("4"));
        assert_eq!(
            reexported.workspace_revision.structure_snapshot,
            exported.workspace_revision.structure_snapshot
        );
    }

    #[test]
    fn treecalc_context_formula_artifacts_are_built_from_node_input_snapshot() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:formula-authority-guard",
            ))
            .unwrap();
        let a_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("A", "3"))
            .unwrap();
        let b_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("B", "=A+1"))
            .unwrap();
        context.recalculate(&workspace_id).unwrap();

        let mut exported = context.export_workspace_snapshot(&workspace_id).unwrap();
        let mut input_records = exported
            .workspace_revision
            .node_input_snapshot
            .records()
            .clone();
        let successor_epoch = input_records
            .get(&b_id)
            .expect("B input record should exist before mutation")
            .input_epoch
            + 1;
        input_records.insert(
            b_id,
            NodeInputRecord::formula_text(b_id, "=A+2", successor_epoch),
        );
        exported.workspace_revision = WorkspaceRevision::new(
            exported.workspace_id.as_str(),
            exported.workspace_revision.structure_snapshot.clone(),
            NodeInputSnapshot::from_record_map(input_records),
            exported.workspace_revision.namespace_snapshot.clone(),
        );
        exported.formula_binding_snapshot = FormulaBindingSnapshot::current_absent(
            exported.workspace_revision.revision_id(),
            "test-mutated-formula-input",
        );
        exported.dependency_shape_snapshot = DependencyShapeSnapshot::current_absent(
            exported.workspace_revision.revision_id(),
            exported.formula_binding_snapshot.snapshot_id(),
            "test-mutated-formula-input",
        );

        let mut imported_context = OxCalcTreeContext::default();
        let imported_workspace_id = imported_context
            .import_workspace_snapshot(exported)
            .unwrap();
        let imported_revision = imported_context
            .workspace_revision(&imported_workspace_id)
            .unwrap();
        let imported_b_record = imported_revision
            .node_input_snapshot
            .try_get_record(b_id)
            .expect("B input record should be imported into the node input snapshot");
        assert_eq!(imported_b_record.kind, NodeInputKind::FormulaText);
        assert_eq!(imported_b_record.text.as_deref(), Some("=A+2"));
        imported_context
            .set_node_input_value(&imported_workspace_id, a_id, "4")
            .unwrap();
        let result = imported_context
            .recalculate(&imported_workspace_id)
            .unwrap();
        let b_descriptors = result
            .dependency_graph
            .descriptors_by_owner
            .get(&b_id)
            .expect("B dependency descriptors should be rebuilt from formula text");

        assert_eq!(result.run_state, OxCalcTreeRunState::Published);
        assert_eq!(result.published_values.get(&b_id), Some(&"6".to_string()));
        assert!(
            b_descriptors
                .iter()
                .any(|descriptor| descriptor.descriptor_id.contains(&format!(
                    "formula:{}:{}:v",
                    imported_workspace_id.as_str(),
                    b_id.0
                )))
        );
    }

    #[test]
    fn treecalc_context_owns_node_table_lifecycle_and_views() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new("workspace:tables"))
            .unwrap();
        let sales_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("Sales", ""))
            .unwrap();
        let before_table_set = context.workspace_view(&workspace_id).unwrap();

        let view = context
            .set_node_table(&workspace_id, sales_id, sales_table_snapshot(sales_id))
            .unwrap();
        let after_table_set = context.workspace_view(&workspace_id).unwrap();
        let after_table_set_revision = context.workspace_revision(&workspace_id).unwrap();

        assert_ne!(before_table_set.snapshot_id, after_table_set.snapshot_id);
        assert_ne!(
            before_table_set.workspace_revision_id,
            after_table_set.workspace_revision_id
        );
        assert_eq!(
            before_table_set.node_input_snapshot_id,
            after_table_set.node_input_snapshot_id
        );
        assert!(
            after_table_set_revision
                .structure_snapshot
                .table_shapes()
                .contains_key(&sales_id)
        );
        assert_eq!(view.table_node_id, sales_id);
        assert_eq!(view.table_id, "table:sales");
        assert_eq!(view.canonical_path, "Root/Sales");
        assert_eq!(view.snapshot.canonical_path, "Root/Sales");
        assert_ne!(
            view.snapshot.table_namespace_version,
            "host-supplied-namespace-should-not-win"
        );
        assert!(
            view.projection
                .table_context_identity
                .contains("treecalc.table_context.v1")
        );
        assert!(
            view.dependency_inventory
                .facts
                .iter()
                .any(|fact| fact.kind == StructuredTableDependencyFactKind::RowMembership)
        );

        let node_view = context.node_view(&workspace_id, sales_id).unwrap();
        assert_eq!(
            node_view
                .table
                .as_ref()
                .map(|table| table.table_id.as_str()),
            Some("table:sales")
        );
        let workspace_view = context.workspace_view(&workspace_id).unwrap();
        assert_eq!(workspace_view.tables.len(), 1);

        let removed = context.clear_node_table(&workspace_id, sales_id).unwrap();
        let after_table_clear = context.workspace_view(&workspace_id).unwrap();
        let after_table_clear_revision = context.workspace_revision(&workspace_id).unwrap();
        assert_eq!(
            removed.map(|snapshot| snapshot.table_id),
            Some("table:sales".to_string())
        );
        assert_ne!(after_table_set.snapshot_id, after_table_clear.snapshot_id);
        assert_ne!(
            after_table_set.workspace_revision_id,
            after_table_clear.workspace_revision_id
        );
        assert_eq!(
            after_table_set.node_input_snapshot_id,
            after_table_clear.node_input_snapshot_id
        );
        assert!(
            !after_table_clear_revision
                .structure_snapshot
                .table_shapes()
                .contains_key(&sales_id)
        );
        assert!(
            context
                .table_view(&workspace_id, sales_id)
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn treecalc_context_routes_table_catalog_lowering_and_dynamic_rebind() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new("workspace:table-routing"))
            .unwrap();
        let sales_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("Sales", ""))
            .unwrap();
        context
            .set_node_table(&workspace_id, sales_id, sales_table_snapshot(sales_id))
            .unwrap();

        let resolved = context
            .resolve_table_reference(
                &workspace_id,
                &TreeCalcTableCatalogResolveRequest::table_name_or_path("SalesTable"),
            )
            .unwrap();
        assert_eq!(resolved.effective_table_id.as_deref(), Some("table:sales"));
        assert_eq!(resolved.table_node_id, Some(sales_id));
        assert!(resolved.diagnostics.is_empty());

        let lowering = context
            .lower_table_reference(
                &workspace_id,
                sales_id,
                StructuredTableReferenceIntake::explicit_table("hostref:table:1", "table:sales")
                    .with_selected_columns(["table:sales:col:amount".to_string()])
                    .with_selected_regions([StructuredTableRegionSelection::Data]),
                None,
                None,
            )
            .unwrap();
        let lowered_kinds = lowering
            .facts
            .iter()
            .map(|fact| fact.kind)
            .collect::<std::collections::BTreeSet<_>>();
        assert!(lowered_kinds.contains(&StructuredTableDependencyFactKind::RowMembership));
        assert!(lowered_kinds.contains(&StructuredTableDependencyFactKind::DataRegion));

        let report = context
            .classify_dynamic_table_rebind(
                &workspace_id,
                TreeCalcDynamicTableRebindRequest {
                    selector_handle: resolved.table_reference_handle.clone(),
                    selector_identity: resolved.opaque_selector.clone(),
                    source_reference_handle: Some("hostref:table:1".to_string()),
                    target_kind: TreeCalcDynamicTableReferenceTargetKind::Column,
                    cause: TreeCalcDynamicTableRebindCause::SelectorTextChanged,
                    before_resolved_table_identity: resolved.virtual_anchor_identity.clone(),
                    after_resolved_table_identity: None,
                    caller_context_id: None,
                    context_versions: TreeCalcTableLifecycleContextVersions::default(),
                    oxfml_structured_bind_packet_available: true,
                },
            )
            .unwrap();
        assert_eq!(
            report.status,
            TreeCalcDynamicTableRebindStatus::RebindRequired
        );
        assert!(report.prepared_identity_inputs.contains(
            &crate::structured_table::TreeCalcTablePreparedIdentityInput::TableContextIdentity
        ));
        assert!(report.oxfml_generic_bind_packet_available);
    }

    #[test]
    fn node_table_progressive_01_can_create_table_node() {
        let (context, workspace_id, sales_id) =
            context_with_sales_table("workspace:node-table-progressive-01");

        let table = context
            .table_view(&workspace_id, sales_id)
            .unwrap()
            .unwrap();
        assert_eq!(table.table_node_id, sales_id);
        assert_eq!(table.table_id, "table:sales");
        assert_eq!(table.table_name, "SalesTable");
        assert_eq!(table.projection.table_descriptor.table_range_ref, "A1:A4");
    }

    #[test]
    fn node_table_progressive_02_table_is_visible_from_node_and_workspace_views() {
        let (context, workspace_id, sales_id) =
            context_with_sales_table("workspace:node-table-progressive-02");

        let node = context.node_view(&workspace_id, sales_id).unwrap();
        let workspace = context.workspace_view(&workspace_id).unwrap();
        assert_eq!(
            node.table.as_ref().map(|table| table.table_id.as_str()),
            Some("table:sales")
        );
        assert_eq!(workspace.tables.len(), 1);
        assert_eq!(workspace.tables[0].table_node_id, sales_id);
    }

    #[test]
    fn node_table_progressive_03_clearing_table_removes_shape_but_not_node() {
        let (mut context, workspace_id, sales_id) =
            context_with_sales_table("workspace:node-table-progressive-03");

        let removed = context.clear_node_table(&workspace_id, sales_id).unwrap();
        let node = context.node_view(&workspace_id, sales_id).unwrap();

        assert_eq!(
            removed.map(|table| table.table_id),
            Some("table:sales".to_string())
        );
        assert!(node.table.is_none());
        assert!(
            context
                .table_view(&workspace_id, sales_id)
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn node_table_progressive_04_table_context_packet_exposes_oxfml_descriptor() {
        let (context, workspace_id, _sales_id) =
            context_with_sales_table("workspace:node-table-progressive-04");

        let packet = context
            .table_context_packet(&workspace_id, None, None)
            .unwrap();

        assert_eq!(packet.table_catalog.len(), 1);
        assert_eq!(packet.table_catalog[0].table_name, "SalesTable");
        assert_eq!(packet.table_catalog[0].columns[0].column_name, "Amount");
        assert!(packet.table_context_identity.contains("SalesTable"));
    }

    #[test]
    fn node_table_progressive_05_table_catalog_resolves_table_name() {
        let (context, workspace_id, sales_id) =
            context_with_sales_table("workspace:node-table-progressive-05");

        let resolved = context
            .resolve_table_reference(
                &workspace_id,
                &TreeCalcTableCatalogResolveRequest::table_name_or_path("SalesTable"),
            )
            .unwrap();

        assert_eq!(resolved.table_node_id, Some(sales_id));
        assert_eq!(resolved.effective_table_id.as_deref(), Some("table:sales"));
        assert!(resolved.diagnostics.is_empty());
    }

    #[test]
    fn node_table_progressive_06_explicit_column_lowering_builds_dependency_facts() {
        let (context, workspace_id, sales_id) =
            context_with_sales_table("workspace:node-table-progressive-06");

        let lowering = context
            .lower_table_reference(
                &workspace_id,
                sales_id,
                StructuredTableReferenceIntake::explicit_table(
                    "hostref:table:amount",
                    "table:sales",
                )
                .with_selected_columns(["table:sales:col:amount".to_string()])
                .with_selected_regions([StructuredTableRegionSelection::Data]),
                None,
                None,
            )
            .unwrap();
        let kinds = lowering
            .facts
            .iter()
            .map(|fact| fact.kind)
            .collect::<std::collections::BTreeSet<_>>();

        assert!(kinds.contains(&StructuredTableDependencyFactKind::RowMembership));
        assert!(kinds.contains(&StructuredTableDependencyFactKind::RowOrder));
        assert!(kinds.contains(&StructuredTableDependencyFactKind::DataRegion));
    }

    #[test]
    fn node_table_progressive_07_other_node_formula_binds_structured_reference() {
        let (mut context, workspace_id, _sales_id) =
            context_with_sales_table("workspace:node-table-progressive-07");
        let summary_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Summary", "=SUM(SalesTable[Amount])"),
            )
            .unwrap();

        let state = context.workspace(&workspace_id).unwrap();
        let catalog_build = build_context_formula_catalog(state, &context.options).unwrap();
        let binding = catalog_build.catalog.try_get_binding(summary_id).unwrap();
        let bound_formula = binding
            .expression
            .bound_formula()
            .expect("formula should carry OxFml bound formula");

        assert!(catalog_build.diagnostics.is_empty());
        assert!(bound_formula.structured_reference_bind_records.is_empty());
        let profile_record = bound_formula
            .normalized_references
            .iter()
            .find_map(|reference| match reference {
                oxfml_core::binding::NormalizedReference::ProfileSymbolic(record)
                    if record.source_info.source_text == "SalesTable[Amount]" =>
                {
                    Some(record)
                }
                _ => None,
            })
            .expect("structured table reference should bind as a profile-symbolic record");
        assert_eq!(profile_record.profile_id, "dna.treecalc.v1");
    }

    #[test]
    fn node_table_progressive_08_unrelated_node_formula_still_recalculates_with_table_present() {
        let (mut context, workspace_id, _sales_id) =
            context_with_sales_table("workspace:node-table-progressive-08");
        let a_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("A", "=3"))
            .unwrap();
        let b_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("B", "=A+1"))
            .unwrap();

        let result = context.recalculate(&workspace_id).unwrap();

        assert_eq!(result.run_state, OxCalcTreeRunState::Published);
        assert_eq!(result.published_values.get(&a_id), Some(&"3".to_string()));
        assert_eq!(result.published_values.get(&b_id), Some(&"4".to_string()));
    }

    #[test]
    fn node_table_progressive_09_table_column_formula_runtime_uses_current_row() {
        let snapshot = runtime_sales_table_snapshot(TreeNodeId(2));
        let projection = project_treecalc_table_node_snapshot(&snapshot).unwrap();
        let request = TreeCalcTableColumnFormulaRuntimeRequest {
            target_column_id: "col:tax".to_string(),
            formula_stable_id: "formula:body:tax".to_string(),
            formula_text_version: 1,
            formula_text: "=SUM([@Amount])/10".to_string(),
            values: runtime_sales_amount_values(),
            runtime_context: TreeCalcTableFormulaRuntimeContext::default(),
        };

        let report =
            evaluate_treecalc_table_column_formula_rows(&snapshot, &projection, &request).unwrap();
        let values = report
            .cell_results
            .iter()
            .map(|cell| cell.value.clone())
            .collect::<Vec<_>>();

        assert_eq!(
            values,
            vec![
                CalcValue::number(1.0),
                CalcValue::number(2.0),
                CalcValue::number(3.0)
            ]
        );
        assert_eq!(report.cell_results.len(), 3);
        assert!(
            report
                .cell_results
                .iter()
                .all(|cell| cell.host_formula_context.table_context_identity.is_some())
        );
    }

    #[test]
    fn node_table_progressive_10_table_totals_formula_runtime_aggregates_column() {
        let snapshot = runtime_sales_table_snapshot(TreeNodeId(2));
        let projection = project_treecalc_table_node_snapshot(&snapshot).unwrap();
        let request = TreeCalcTableColumnFormulaRuntimeRequest {
            target_column_id: "col:amount".to_string(),
            formula_stable_id: "formula:totals:amount".to_string(),
            formula_text_version: 1,
            formula_text: "=SUM(SalesTable[Amount])".to_string(),
            values: runtime_sales_amount_values(),
            runtime_context: TreeCalcTableFormulaRuntimeContext::default(),
        };

        let result =
            evaluate_treecalc_table_totals_formula(&snapshot, &projection, &request).unwrap();

        assert_eq!(result.value, CalcValue::number(60.0));
        assert_eq!(result.structured_reference_handles.len(), 1);
    }

    #[test]
    fn node_table_progressive_11_table_body_cells_are_node_identities() {
        let (context, workspace_id, sales_id) =
            context_with_sales_table("workspace:node-table-progressive-11");
        let table = context
            .table_view(&workspace_id, sales_id)
            .unwrap()
            .expect("sales table should exist");
        let body_cell_node_ids = table
            .snapshot
            .body_cell_nodes
            .iter()
            .map(|cell| cell.node_id)
            .collect::<Vec<_>>();

        assert_eq!(body_cell_node_ids.len(), 2);
        assert_eq!(table.snapshot.totals_cell_nodes.len(), 1);
        let revision = context.workspace_revision(&workspace_id).unwrap();
        for node_id in &body_cell_node_ids {
            assert_eq!(
                revision.structure_snapshot.parent_id_of(*node_id),
                Some(sales_id)
            );
        }
        assert_eq!(
            revision
                .node_input_snapshot
                .try_get_record(body_cell_node_ids[0])
                .and_then(|record| record.text.as_deref()),
            Some("10")
        );
        assert_eq!(
            revision
                .node_input_snapshot
                .try_get_record(body_cell_node_ids[1])
                .and_then(|record| record.text.as_deref()),
            Some("20")
        );
        assert_eq!(
            revision
                .structure_snapshot
                .parent_id_of(table.snapshot.totals_cell_nodes[0].node_id),
            Some(sales_id)
        );
        assert_eq!(
            revision
                .node_input_snapshot
                .try_get_record(table.snapshot.totals_cell_nodes[0].node_id)
                .and_then(|record| record.text.as_deref()),
            Some("30")
        );
    }

    #[test]
    fn node_table_progressive_12_other_node_structured_reference_is_value_backed() {
        let (mut context, workspace_id, sales_id) =
            context_with_sales_table("workspace:node-table-progressive-12");
        let summary_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Summary", "=SUM(SalesTable[Amount])"),
            )
            .unwrap();

        let result = context.recalculate(&workspace_id).unwrap();

        assert_eq!(
            result.published_values.get(&summary_id),
            Some(&"30".to_string()),
            "expected structured table reference to publish through body-cell nodes: run_state={:?}; reject={:?}; diagnostics={:?}",
            result.run_state,
            result.reject_detail,
            result.diagnostics
        );
        let table = context
            .table_view(&workspace_id, sales_id)
            .unwrap()
            .expect("sales table should exist");
        let first_body_cell_node_id = table.snapshot.body_cell_nodes[0].node_id;

        context
            .set_node_input_value(&workspace_id, first_body_cell_node_id, "40")
            .unwrap();
        let updated = context.recalculate(&workspace_id).unwrap();
        assert_eq!(
            updated.published_values.get(&summary_id),
            Some(&"60".to_string())
        );
    }

    #[test]
    fn node_table_progressive_13_other_node_header_structured_reference_is_shape_backed() {
        let (mut context, workspace_id, _sales_id) =
            context_with_sales_table("workspace:node-table-progressive-13");
        let summary_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new(
                    "HeaderCount",
                    "=COUNTA(SalesTable[[#Headers],[Amount]])",
                ),
            )
            .unwrap();

        let result = context.recalculate(&workspace_id).unwrap();

        assert_eq!(
            result.published_values.get(&summary_id),
            Some(&"1".to_string()),
            "expected structured table header reference to publish through table shape: run_state={:?}; reject={:?}; diagnostics={:?}",
            result.run_state,
            result.reject_detail,
            result.diagnostics
        );
    }

    #[test]
    fn node_table_progressive_14_other_node_totals_structured_reference_is_value_backed() {
        let (mut context, workspace_id, sales_id) =
            context_with_sales_table("workspace:node-table-progressive-14");
        let summary_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("SummaryValue", "=SalesTable[[#Totals],[Amount]]"),
            )
            .unwrap();

        let result = context.recalculate(&workspace_id).unwrap();

        assert_eq!(
            result.published_values.get(&summary_id),
            Some(&"30".to_string()),
            "expected structured table totals reference to publish through totals-cell node: run_state={:?}; reject={:?}; diagnostics={:?}",
            result.run_state,
            result.reject_detail,
            result.diagnostics
        );
        let table = context
            .table_view(&workspace_id, sales_id)
            .unwrap()
            .expect("sales table should exist");
        let totals_amount_node_id = table.snapshot.totals_cell_nodes[0].node_id;

        context
            .set_node_input_value(&workspace_id, totals_amount_node_id, "80")
            .unwrap();
        let updated = context.recalculate(&workspace_id).unwrap();
        assert_eq!(
            updated.published_values.get(&summary_id),
            Some(&"80".to_string())
        );
    }

    #[test]
    fn treecalc_context_export_import_preserves_identity_and_recalc_state() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new("workspace:persist"))
            .unwrap();
        let a_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("A", "=3"))
            .unwrap();
        let b_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("B", "=A+1"))
            .unwrap();
        let sales_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("Sales", ""))
            .unwrap();
        context
            .set_node_table(&workspace_id, sales_id, sales_table_snapshot(sales_id))
            .unwrap();

        let first_result = context.recalculate(&workspace_id).unwrap();
        assert_eq!(
            first_result.published_values.get(&a_id),
            Some(&"3".to_string())
        );
        assert_eq!(
            first_result.published_values.get(&b_id),
            Some(&"4".to_string())
        );
        let table_before = context
            .table_view(&workspace_id, sales_id)
            .unwrap()
            .unwrap();

        let snapshot = context.export_workspace_snapshot(&workspace_id).unwrap();
        let before_view = context.workspace_view(&workspace_id).unwrap();
        assert_eq!(snapshot.workspace_id, workspace_id);
        assert_eq!(
            snapshot.schema_version,
            OXCALC_TREE_WORKSPACE_SNAPSHOT_SCHEMA_V2
        );
        assert_eq!(snapshot.root_node_id, TreeNodeId(1));
        assert_eq!(
            snapshot.workspace_revision.revision_id(),
            &before_view.workspace_revision_id
        );
        assert_eq!(
            snapshot.formula_binding_snapshot.snapshot_id(),
            &before_view.formula_binding_snapshot_id
        );
        assert_eq!(
            snapshot.dependency_shape_snapshot.snapshot_id(),
            &before_view.dependency_shape_snapshot_id
        );
        assert_eq!(
            snapshot.publication_snapshot.snapshot_id(),
            &before_view.publication_snapshot_id
        );
        assert_eq!(
            snapshot.runtime_overlay_set.overlay_set_id(),
            &before_view.runtime_overlay_set_id
        );
        assert_eq!(exported_input_text(&snapshot, a_id), Some("=3"));
        assert_eq!(exported_input_text(&snapshot, b_id), Some("=A+1"));
        assert_eq!(
            snapshot
                .table_snapshots
                .get(&sales_id)
                .map(|table| table.table_id.as_str()),
            Some("table:sales")
        );
        assert_eq!(
            snapshot.publication_values.get(&b_id),
            Some(&SnapshotCalcValue::Number(4.0))
        );

        let serialized = serde_json::to_string_pretty(&snapshot).unwrap();
        let reparsed: OxCalcTreeWorkspaceSnapshot = serde_json::from_str(&serialized).unwrap();
        assert_eq!(reparsed.workspace_id, workspace_id);
        assert_eq!(reparsed.root_node_id, snapshot.root_node_id);
        assert_eq!(
            reparsed.schema_version,
            OXCALC_TREE_WORKSPACE_SNAPSHOT_SCHEMA_V2
        );
        assert_eq!(exported_input_text(&reparsed, b_id), Some("=A+1"));
        assert_eq!(
            reparsed
                .table_snapshots
                .get(&sales_id)
                .map(|table| table.table_id.as_str()),
            Some("table:sales")
        );
        assert_eq!(
            reparsed.publication_values.get(&b_id),
            Some(&SnapshotCalcValue::Number(4.0))
        );

        let mut imported_context = OxCalcTreeContext::default();
        let imported_workspace_id = imported_context
            .import_workspace_snapshot(reparsed)
            .unwrap();
        assert_eq!(imported_workspace_id, workspace_id);
        let imported_view_before_recalc = imported_context
            .workspace_view(&imported_workspace_id)
            .unwrap();
        assert_eq!(
            imported_view_before_recalc.workspace_revision_id,
            snapshot.workspace_revision.revision_id().clone()
        );
        assert_eq!(
            imported_view_before_recalc.formula_binding_snapshot_id,
            snapshot.formula_binding_snapshot.snapshot_id().clone()
        );
        assert_eq!(
            imported_view_before_recalc.dependency_shape_snapshot_id,
            snapshot.dependency_shape_snapshot.snapshot_id().clone()
        );
        assert_eq!(
            imported_view_before_recalc.publication_snapshot_id,
            snapshot.publication_snapshot.snapshot_id().clone()
        );
        assert_eq!(
            imported_view_before_recalc.runtime_overlay_set_id,
            snapshot.runtime_overlay_set.overlay_set_id().clone()
        );

        let imported_b_before_recalc = imported_context
            .node_view(&imported_workspace_id, b_id)
            .unwrap();
        assert_eq!(imported_b_before_recalc.value_text.as_deref(), Some("4"));

        let table_after_import = imported_context
            .table_view(&imported_workspace_id, sales_id)
            .unwrap()
            .unwrap();
        assert_eq!(table_after_import.table_id, table_before.table_id);
        assert_eq!(
            table_after_import.projection.table_context_identity,
            table_before.projection.table_context_identity
        );
        assert_eq!(
            table_after_import.snapshot.table_namespace_version,
            table_before.snapshot.table_namespace_version
        );

        let imported_result = imported_context
            .recalculate(&imported_workspace_id)
            .unwrap();
        assert_eq!(imported_result.run_state, OxCalcTreeRunState::VerifiedClean);
        assert_eq!(
            imported_result.published_values.get(&a_id),
            Some(&"3".to_string())
        );
        assert_eq!(
            imported_result.published_values.get(&b_id),
            Some(&"4".to_string())
        );

        let c_id = imported_context
            .add_node(
                &imported_workspace_id,
                OxCalcTreeNodeCreate::new("C", "=B+1"),
            )
            .unwrap();
        assert_eq!(c_id, TreeNodeId(5));
        let final_result = imported_context
            .recalculate(&imported_workspace_id)
            .unwrap();
        assert_eq!(
            final_result.published_values.get(&c_id),
            Some(&"5".to_string())
        );
    }

    #[test]
    fn treecalc_context_import_rejects_snapshots_missing_formula_truth() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new("workspace:invalid-import"))
            .unwrap();
        let a_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("A", "=3"))
            .unwrap();
        let mut snapshot = context.export_workspace_snapshot(&workspace_id).unwrap();
        let mut records = snapshot
            .workspace_revision
            .node_input_snapshot
            .records()
            .clone();
        records.remove(&a_id);
        snapshot.workspace_revision = WorkspaceRevision::new(
            snapshot.workspace_id.as_str(),
            snapshot.workspace_revision.structure_snapshot.clone(),
            NodeInputSnapshot::from_record_map(records),
            snapshot.workspace_revision.namespace_snapshot.clone(),
        );
        snapshot.formula_binding_snapshot = FormulaBindingSnapshot::current_absent(
            snapshot.workspace_revision.revision_id(),
            "test-missing-input-truth",
        );
        snapshot.dependency_shape_snapshot = DependencyShapeSnapshot::current_absent(
            snapshot.workspace_revision.revision_id(),
            snapshot.formula_binding_snapshot.snapshot_id(),
            "test-missing-input-truth",
        );

        let err = OxCalcTreeContext::default()
            .import_workspace_snapshot(snapshot)
            .unwrap_err();
        assert!(matches!(
            err,
            OxCalcTreeContextError::InvalidWorkspaceSnapshot { .. }
        ));
    }

    #[test]
    fn treecalc_context_import_rejects_unsupported_snapshot_schema_version() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:invalid-schema-import",
            ))
            .unwrap();
        context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("A", "=3"))
            .unwrap();
        let mut snapshot = context.export_workspace_snapshot(&workspace_id).unwrap();
        snapshot.schema_version = "oxcalc.tree.workspace_snapshot.legacy".to_string();

        let err = OxCalcTreeContext::default()
            .import_workspace_snapshot(snapshot)
            .unwrap_err();
        assert!(matches!(
            err,
            OxCalcTreeContextError::InvalidWorkspaceSnapshot { .. }
        ));
    }

    #[test]
    fn treecalc_context_import_rejects_input_epoch_watermark_regression() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:invalid-epoch-import",
            ))
            .unwrap();
        let a_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("A", "3"))
            .unwrap();
        context
            .set_node_input_value(&workspace_id, a_id, "4")
            .unwrap();
        let mut snapshot = context.export_workspace_snapshot(&workspace_id).unwrap();
        assert!(
            snapshot.input_epoch_watermark >= exported_input_epoch(&snapshot, a_id),
            "export should not create a regressing input epoch watermark"
        );
        snapshot.input_epoch_watermark = exported_input_epoch(&snapshot, a_id) - 1;

        let err = OxCalcTreeContext::default()
            .import_workspace_snapshot(snapshot)
            .unwrap_err();
        assert!(matches!(
            err,
            OxCalcTreeContextError::InvalidWorkspaceSnapshot { .. }
        ));
    }

    #[test]
    fn treecalc_context_raw_dotted_name_uses_host_name_bind_result() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new("workspace:dotted"))
            .unwrap();
        let a_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("A", ""))
            .unwrap();
        context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Child", "=3").under(a_id),
            )
            .unwrap();
        let b_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("B", "=A.Child+1"))
            .unwrap();

        let result = context.recalculate(&workspace_id).unwrap();

        assert_eq!(result.run_state, OxCalcTreeRunState::Published);
        assert_eq!(result.published_values.get(&b_id), Some(&"4".to_string()));
        assert!(result.diagnostics.iter().any(|diagnostic| {
            diagnostic.contains("w056_host_name_bind_result:")
                && diagnostic.contains("token=A.Child")
                && diagnostic.contains("span=1-8")
        }));
    }

    #[test]
    fn treecalc_context_resolves_bare_names_by_lexical_walkup() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new("workspace:walkup"))
            .unwrap();
        let accounts_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("Accounts", ""))
            .unwrap();
        let year_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Y2005", "").under(accounts_id),
            )
            .unwrap();
        let q1_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Q1", "").under(year_id),
            )
            .unwrap();
        context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Margin", "=3").under(q1_id),
            )
            .unwrap();
        let income_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Income", "=Sales+Margin").under(q1_id),
            )
            .unwrap();
        let sales_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Sales", "=2").under(income_id),
            )
            .unwrap();
        let deep_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Deep", "=Margin+1").under(sales_id),
            )
            .unwrap();
        let dotted_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Dotted", "=Q1.Margin+1").under(year_id),
            )
            .unwrap();

        let result = context.recalculate(&workspace_id).unwrap();

        assert_eq!(result.run_state, OxCalcTreeRunState::Published);
        assert_eq!(
            result.published_values.get(&income_id),
            Some(&"5".to_string())
        );
        assert_eq!(
            result.published_values.get(&deep_id),
            Some(&"4".to_string())
        );
        assert_eq!(
            result.published_values.get(&dotted_id),
            Some(&"4".to_string())
        );
    }

    struct W056ActiveReferenceCorpusOutcome {
        result: OxCalcTreeCalculationOutcome,
        owner_node_id: TreeNodeId,
        expected_value: &'static str,
    }

    #[test]
    fn w056_active_reference_corpus_executes_broad_raw_formulas_through_oxfml_path() {
        let cases: &[(&str, &str, fn() -> W056ActiveReferenceCorpusOutcome)] = &[
            (
                "children_collection",
                "unqualified children collection",
                w056_active_corpus_children_collection,
            ),
            (
                "walkup_dotted_descent",
                "bare walk-up and dotted descent",
                w056_active_corpus_walkup_dotted_descent,
            ),
            (
                "ancestor_root_anchors",
                "ancestor anchors",
                w056_active_corpus_ancestor_anchors,
            ),
            (
                "escaping_canonicalization_case",
                "bracket-escaped paths",
                w056_active_corpus_escaped_paths,
            ),
            (
                "meta_invisibility_accessors",
                "metadata accessors",
                w056_active_corpus_metadata_accessors,
            ),
            (
                "sibling_single_navigation",
                "single sibling navigation",
                w056_active_corpus_sibling_navigation,
            ),
            (
                "ordered_set_selectors",
                "ordered set selectors",
                w056_active_corpus_ordered_selectors,
            ),
            (
                "recursive_descent",
                "recursive descent with tail",
                w056_active_corpus_recursive_descent,
            ),
            (
                "reference_literals_arrays",
                "reference literal arrays",
                w056_active_corpus_reference_literal_array,
            ),
            (
                "dynamic_indirect_ctro",
                "dynamic INDIRECT CTRO",
                w056_active_corpus_dynamic_indirect,
            ),
            (
                "bare_host_names_defined_name_lane",
                "bare host-name defined-name lane",
                w056_active_corpus_bare_host_name,
            ),
        ];

        for (category_id, scenario_id, run_case) in cases {
            let category = w056_non_table_reference_category(category_id)
                .unwrap_or_else(|| panic!("missing W056 category {category_id}"));
            assert!(
                matches!(
                    category.evidence_status,
                    W056NonTableReferenceEvidenceStatus::ProductGreen
                        | W056NonTableReferenceEvidenceStatus::DirectContextSliceGreen
                ),
                "{category_id} should not be in the active green OxFml corpus with status {:?}",
                category.evidence_status
            );

            let outcome = run_case();
            assert_eq!(
                outcome.result.run_state,
                OxCalcTreeRunState::Published,
                "{category_id}/{scenario_id} should publish: reject={:?}; diagnostics={:?}",
                outcome.result.reject_detail,
                outcome.result.diagnostics
            );
            assert_eq!(
                outcome
                    .result
                    .published_values
                    .get(&outcome.owner_node_id)
                    .map(String::as_str),
                Some(outcome.expected_value),
                "{category_id}/{scenario_id} published the wrong value"
            );
            assert!(
                outcome
                    .result
                    .diagnostics
                    .iter()
                    .any(|diagnostic| diagnostic.contains("oxfml_candidate_result_id:")),
                "{category_id}/{scenario_id} did not invoke the OxFml candidate path: {:?}",
                outcome.result.diagnostics
            );
            assert!(
                outcome_owner_has_source_reference_handle(&outcome),
                "{category_id}/{scenario_id} did not publish a source-correlated reference descriptor: {:?}",
                outcome
                    .result
                    .dependency_graph
                    .descriptors_by_owner
                    .get(&outcome.owner_node_id)
            );
        }
    }

    fn outcome_owner_has_source_reference_handle(
        outcome: &W056ActiveReferenceCorpusOutcome,
    ) -> bool {
        outcome
            .result
            .dependency_graph
            .descriptors_by_owner
            .get(&outcome.owner_node_id)
            .into_iter()
            .flatten()
            .any(|descriptor| descriptor.source_reference_handle.is_some())
    }

    fn w056_active_corpus_children_collection() -> W056ActiveReferenceCorpusOutcome {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:w056-active-children",
            ))
            .unwrap();
        let owner_node_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Base", "=SUM(@CHILDREN)"),
            )
            .unwrap();
        context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("A", "=2").under(owner_node_id),
            )
            .unwrap();
        context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("B", "=3").under(owner_node_id),
            )
            .unwrap();

        W056ActiveReferenceCorpusOutcome {
            result: context.recalculate(&workspace_id).unwrap(),
            owner_node_id,
            expected_value: "5",
        }
    }

    fn w056_active_corpus_walkup_dotted_descent() -> W056ActiveReferenceCorpusOutcome {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:w056-active-walkup",
            ))
            .unwrap();
        let section_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("Section", ""))
            .unwrap();
        context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Margin", "=3").under(section_id),
            )
            .unwrap();
        let _bare_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Bare", "=Margin+1").under(section_id),
            )
            .unwrap();
        let owner_node_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Dotted", "=Section.Margin+1"),
            )
            .unwrap();

        W056ActiveReferenceCorpusOutcome {
            result: context.recalculate(&workspace_id).unwrap(),
            owner_node_id,
            expected_value: "4",
        }
    }

    fn w056_active_corpus_ancestor_anchors() -> W056ActiveReferenceCorpusOutcome {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:w056-active-anchors",
            ))
            .unwrap();
        let accounts_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("Accounts", ""))
            .unwrap();
        let year_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Y2005", "").under(accounts_id),
            )
            .unwrap();
        let q1_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Q1", "").under(year_id),
            )
            .unwrap();
        context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Margin", "2").under(q1_id),
            )
            .unwrap();
        context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Total", "3").under(year_id),
            )
            .unwrap();
        let owner_node_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("AnchorProbe", "=^.Margin+^^.Total").under(q1_id),
            )
            .unwrap();

        W056ActiveReferenceCorpusOutcome {
            result: context.recalculate(&workspace_id).unwrap(),
            owner_node_id,
            expected_value: "5",
        }
    }

    fn w056_active_corpus_escaped_paths() -> W056ActiveReferenceCorpusOutcome {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:w056-active-escaping",
            ))
            .unwrap();
        context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("Sales Q1", "5"))
            .unwrap();
        let region_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("Region", ""))
            .unwrap();
        context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Net Revenue", "10").under(region_id),
            )
            .unwrap();
        let owner_node_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Escaped", "=[Sales Q1]+Region.[Net Revenue]"),
            )
            .unwrap();

        W056ActiveReferenceCorpusOutcome {
            result: context.recalculate(&workspace_id).unwrap(),
            owner_node_id,
            expected_value: "15",
        }
    }

    fn w056_active_corpus_metadata_accessors() -> W056ActiveReferenceCorpusOutcome {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:w056-active-metadata",
            ))
            .unwrap();
        let section_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("Section", ""))
            .unwrap();
        let owner_node_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("NameProbe", "=@NAME").under(section_id),
            )
            .unwrap();

        W056ActiveReferenceCorpusOutcome {
            result: context.recalculate(&workspace_id).unwrap(),
            owner_node_id,
            expected_value: "NameProbe",
        }
    }

    fn w056_active_corpus_sibling_navigation() -> W056ActiveReferenceCorpusOutcome {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:w056-active-sibling",
            ))
            .unwrap();
        let year_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("Y2005", ""))
            .unwrap();
        let q1_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Q1", "").under(year_id),
            )
            .unwrap();
        context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Net", "7").under(q1_id),
            )
            .unwrap();
        let owner_node_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Q2", "=@PREV.Net+1").under(year_id),
            )
            .unwrap();

        W056ActiveReferenceCorpusOutcome {
            result: context.recalculate(&workspace_id).unwrap(),
            owner_node_id,
            expected_value: "8",
        }
    }

    fn w056_active_corpus_ordered_selectors() -> W056ActiveReferenceCorpusOutcome {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:w056-active-ordered",
            ))
            .unwrap();
        context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("A", "1"))
            .unwrap();
        context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("B", "2"))
            .unwrap();
        let owner_node_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Preceding", "=SUM(@PRECEDING)"),
            )
            .unwrap();

        W056ActiveReferenceCorpusOutcome {
            result: context.recalculate(&workspace_id).unwrap(),
            owner_node_id,
            expected_value: "3",
        }
    }

    fn w056_active_corpus_recursive_descent() -> W056ActiveReferenceCorpusOutcome {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:w056-active-recursive",
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
                OxCalcTreeNodeCreate::new("Margin", "3").under(lane_a_id),
            )
            .unwrap();
        let lane_b_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("LaneB", "").under(base_id),
            )
            .unwrap();
        context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Margin", "4").under(lane_b_id),
            )
            .unwrap();
        let owner_node_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Recursive", "=SUM(Base.**.Margin)"),
            )
            .unwrap();

        W056ActiveReferenceCorpusOutcome {
            result: context.recalculate(&workspace_id).unwrap(),
            owner_node_id,
            expected_value: "7",
        }
    }

    fn w056_active_corpus_reference_literal_array() -> W056ActiveReferenceCorpusOutcome {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:w056-active-reference-literals",
            ))
            .unwrap();
        context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("A", "3"))
            .unwrap();
        context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("C", "5"))
            .unwrap();
        let owner_node_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("LiteralSum", "=SUM({A,C,A})"),
            )
            .unwrap();

        W056ActiveReferenceCorpusOutcome {
            result: context.recalculate(&workspace_id).unwrap(),
            owner_node_id,
            expected_value: "11",
        }
    }

    fn w056_active_corpus_dynamic_indirect() -> W056ActiveReferenceCorpusOutcome {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:w056-active-dynamic",
            ))
            .unwrap();
        context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("A", "2"))
            .unwrap();
        context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("B1", "4"))
            .unwrap();
        context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("B2", "5"))
            .unwrap();
        let owner_node_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Dynamic", "=INDIRECT(\"B\"&A)"),
            )
            .unwrap();

        W056ActiveReferenceCorpusOutcome {
            result: context.recalculate(&workspace_id).unwrap(),
            owner_node_id,
            expected_value: "5",
        }
    }

    fn w056_active_corpus_bare_host_name() -> W056ActiveReferenceCorpusOutcome {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:w056-active-bare-name",
            ))
            .unwrap();
        context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("Revenue", "6"))
            .unwrap();
        let owner_node_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Total", "=Revenue+1"),
            )
            .unwrap();

        W056ActiveReferenceCorpusOutcome {
            result: context.recalculate(&workspace_id).unwrap(),
            owner_node_id,
            expected_value: "7",
        }
    }

    #[test]
    fn treecalc_context_records_typed_exclusions_for_blocked_raw_families() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new("workspace:exclusions"))
            .unwrap();
        context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("A", "=3"))
            .unwrap();
        context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Dyn", "=INDIRECT(\"A\")"),
            )
            .unwrap();
        context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("Call", "=A()"))
            .unwrap();
        context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Literal", "=SUM({A,1})"),
            )
            .unwrap();
        context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Cross", "=Remote!A"),
            )
            .unwrap();

        let result = context.recalculate(&workspace_id).unwrap();

        assert_eq!(result.run_state, OxCalcTreeRunState::Rejected);
        assert!(result.diagnostics.iter().any(|diagnostic| {
            diagnostic.contains("unresolved identifier 'Remote!A'")
                || diagnostic.contains("did not bind 'Remote!A'")
                || diagnostic.contains("UnresolvedReference { target: \"Remote!A\" }")
        }));
        assert!(
            !result
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.contains("node_as_function_w074_pending")),
            "node function calls are callable candidates now, not typed exclusions: {:?}",
            result.diagnostics
        );
    }

    #[test]
    fn treecalc_context_records_typed_exclusion_for_tailed_metadata_accessor() {
        // Metadata accessor values (@NAME/@INDEX/@FORMULA) are scalar terminals:
        // a trailing path on the value (e.g. @NAME.x) is a typed exclusion, not
        // navigation. This keeps the scalar-terminal boundary explicit.
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new("workspace:tailed-metadata"))
            .unwrap();
        let section_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("Section", ""))
            .unwrap();
        context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("TailedName", "=@NAME.x").under(section_id),
            )
            .unwrap();

        let result = context.recalculate(&workspace_id).unwrap();

        assert!(
            result.diagnostics.iter().any(|diagnostic| diagnostic
                .contains("bound_formula_profile_selector_unresolved")
                || diagnostic.contains("UnresolvedReference { target: \"@NAME.x\" }")),
            "missing tailed-metadata unresolved diagnostic in {:?}",
            result.diagnostics
        );
    }

    #[test]
    fn let_lambda_capturing_sibling_node_resolves_and_invalidates() {
        // Case 2 anchor: a single-node LET+LAMBDA that captures a sibling node A.
        // The lambda lives entirely inside one OxFml evaluation (LET-local `f`);
        // A is captured via the host reference-resolution callback. Confirms this
        // works end-to-end today, including invalidation when A changes.
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:let-lambda-capture",
            ))
            .unwrap();
        let a_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("A", "=3"))
            .unwrap();
        let c_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("C", "=LET(f, LAMBDA(X, X+A), f(2))"),
            )
            .unwrap();

        let result = context.recalculate(&workspace_id).unwrap();
        assert_eq!(
            result.run_state,
            OxCalcTreeRunState::Published,
            "case-2 run failed: reject={:?}; diagnostics={:?}",
            result.reject_detail,
            result.diagnostics
        );
        assert_eq!(result.published_values.get(&c_id), Some(&"5".to_string()));

        // Edit A; C must recompute (capture is live through the dependency edge).
        context
            .set_node_formula_text(&workspace_id, a_id, "=10")
            .unwrap();
        let result2 = context.recalculate(&workspace_id).unwrap();
        assert_eq!(result2.published_values.get(&c_id), Some(&"12".to_string()));
    }

    #[test]
    fn treecalc_context_maps_node_array_with_lambda_capturing_host_name() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:map-lambda-node-array",
            ))
            .unwrap();
        let _x_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("x", "1"))
            .unwrap();
        let _a_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("a", "=SEQUENCE(5,5)"),
            )
            .unwrap();
        let m_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("m", "=MAP(a,LAMBDA(v,v+x))"),
            )
            .unwrap();

        let result = context.recalculate(&workspace_id).unwrap();

        assert_eq!(
            result.run_state,
            OxCalcTreeRunState::Published,
            "MAP/LAMBDA node-array run failed: reject={:?}; diagnostics={:?}",
            result.reject_detail,
            result.diagnostics
        );
        let mapped = result
            .published_calc_values
            .get(&m_id)
            .expect("mapped node publishes a CalcValue");
        let CoreValue::Array(array) = &mapped.core else {
            panic!("expected mapped array, got {mapped:?}");
        };
        assert_eq!(
            (array.shape().rows, array.shape().cols),
            (5, 5),
            "mapped value has wrong shape: {mapped:?}"
        );
        assert_eq!(array.get(0, 0), Some(&CalcValue::number(2.0)));
        assert_eq!(array.get(4, 4), Some(&CalcValue::number(26.0)));
    }

    #[test]
    fn treecalc_context_supplies_node_array_calc_value_to_host_name_bindings() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:node-array-host-name-binding",
            ))
            .unwrap();
        let _a_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("a", "=SEQUENCE(2,2)"),
            )
            .unwrap();
        let sum_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("sum", "=SUM(a)"))
            .unwrap();
        let index_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("indexed", "=INDEX(a,2,2)"),
            )
            .unwrap();

        let result = context.recalculate(&workspace_id).unwrap();

        assert_eq!(result.run_state, OxCalcTreeRunState::Published);
        assert_eq!(
            result.published_calc_values.get(&sum_id),
            Some(&CalcValue::number(10.0))
        );
        assert_eq!(
            result.published_calc_values.get(&index_id),
            Some(&CalcValue::number(4.0))
        );
    }

    #[test]
    fn treecalc_context_strict_excel_indirect_is_explicit_profile_pending() {
        let mut context =
            OxCalcTreeContext::new(OxCalcTreeContextOptions::new().with_host_capabilities(
                OxCalcTreeHostCapabilitySnapshot {
                    capability_profile_id: "host-capabilities:strict-excel".to_string(),
                    dynamic_dependency_effects: true,
                    execution_restriction_effects: true,
                    capability_sensitive_effects: true,
                    shape_topology_effects: true,
                },
            ));
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:strict-excel-indirect",
            ))
            .unwrap();
        let owner_node_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Probe", "=INDIRECT(\"Sheet1!Foo\")"),
            )
            .unwrap();

        let result = context.recalculate(&workspace_id).unwrap();

        assert_eq!(result.run_state, OxCalcTreeRunState::Rejected);
        assert!(result.diagnostics.iter().any(|diagnostic| {
            diagnostic.contains("typed_exclusion:strict_excel_profile_not_supported:INDIRECT")
        }));
        assert_eq!(
            result.reject_detail.as_ref().map(|detail| detail.kind),
            Some(RejectKind::HostInjectedFailure)
        );
        assert!(owner_node_id.0 > 0);
    }

    #[test]
    fn treecalc_context_indirect_resolves_reference_text_and_records_ctro_edge() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new("workspace:indirect"))
            .unwrap();
        let a_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("A", "2"))
            .unwrap();
        context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("B1", "4"))
            .unwrap();
        let b2_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("B2", "5"))
            .unwrap();
        let b3_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("B3", "6"))
            .unwrap();
        let c_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("C", "=INDIRECT(\"B\"&A)"),
            )
            .unwrap();

        let initial = context.recalculate(&workspace_id).unwrap();

        assert_eq!(initial.run_state, OxCalcTreeRunState::Published);
        assert_eq!(initial.published_values.get(&c_id), Some(&"5".to_string()));
        assert_eq!(initial.runtime_effect_overlays.len(), 1);
        assert_eq!(
            initial.runtime_effect_overlays[0].key.overlay_kind,
            OverlayKind::DynamicDependency
        );
        let ctro_b2_diagnostic =
            format!("ctro_reference_text_resolution:owner={c_id};target={b2_id};text=B2");
        assert!(
            initial
                .diagnostics
                .iter()
                .any(|diagnostic| { diagnostic.contains(&ctro_b2_diagnostic) })
        );
        assert!(
            !initial.diagnostics.iter().any(|diagnostic| diagnostic
                .contains("typed_exclusion:dynamic_indirect_raw_context_pending")),
            "INDIRECT should execute through the FEC reference-text resolver, not a typed exclusion: {:?}",
            initial.diagnostics
        );
        let initial_c_edges = initial
            .dependency_graph
            .edges_by_owner
            .get(&c_id)
            .expect("C should have static and dynamic dependencies");
        assert!(initial_c_edges.iter().any(|edge| {
            edge.target_node_id == a_id && edge.kind == DependencyDescriptorKind::StaticDirect
        }));
        assert!(initial_c_edges.iter().any(|edge| {
            edge.target_node_id == b2_id && edge.kind == DependencyDescriptorKind::DynamicPotential
        }));
        assert!(initial.runtime_effects.iter().any(|effect| {
            effect.family == RuntimeEffectFamily::DynamicDependency
                && effect.detail.contains(&format!("owner_node:{c_id}"))
                && effect.detail.contains(&format!("target_node:{b2_id}"))
        }));
        let initial_view = context.workspace_view(&workspace_id).unwrap();
        let initial_state = context.workspace(&workspace_id).unwrap();
        assert!(initial_view.publication_snapshot_id.0.contains("current"));
        assert!(initial_view.runtime_overlay_set_id.0.contains("current"));
        assert!(matches!(
            &initial_state.runtime_overlay_set.state,
            SnapshotLayerState::Current { .. }
        ));
        let before_dynamic_target_edit = context.workspace_view(&workspace_id).unwrap();

        context
            .set_node_input_value(&workspace_id, b2_id, "7")
            .unwrap();
        let after_dynamic_target_edit = context.workspace_view(&workspace_id).unwrap();
        let target_value_edit = context.recalculate(&workspace_id).unwrap();
        assert_eq!(
            before_dynamic_target_edit.snapshot_id,
            after_dynamic_target_edit.snapshot_id
        );
        assert_eq!(
            after_dynamic_target_edit.value_epoch,
            before_dynamic_target_edit.value_epoch + 1
        );
        assert_eq!(
            target_value_edit.published_values.get(&c_id),
            Some(&"7".to_string())
        );
        assert_eq!(target_value_edit.runtime_effect_overlays.len(), 1);
        assert!(
            target_value_edit
                .diagnostics
                .iter()
                .any(|diagnostic| { diagnostic.contains(&ctro_b2_diagnostic) })
        );
        assert!(
            target_value_edit
                .invalidation_closure
                .records
                .get(&c_id)
                .is_some_and(|record| record
                    .reasons
                    .contains(&InvalidationReasonKind::UpstreamPublication)),
            "C should be invalidated through the published dynamic B2 overlay: {:?}",
            target_value_edit.invalidation_closure
        );
        assert!(target_value_edit.diagnostics.iter().any(|diagnostic| {
            diagnostic == &format!("edge_value_cache_bypass:{c_id}:UpstreamPublication")
        }));

        context
            .set_node_input_value(&workspace_id, a_id, "3")
            .unwrap();
        let selector_value_edit = context.recalculate(&workspace_id).unwrap();
        assert_eq!(
            selector_value_edit.published_values.get(&c_id),
            Some(&"6".to_string())
        );
        assert_eq!(selector_value_edit.runtime_effect_overlays.len(), 1);
        let switched_c_edges = selector_value_edit
            .dependency_graph
            .edges_by_owner
            .get(&c_id)
            .expect("C should keep static A and current dynamic target dependencies");
        assert!(switched_c_edges.iter().any(|edge| {
            edge.target_node_id == b3_id && edge.kind == DependencyDescriptorKind::DynamicPotential
        }));
        assert!(selector_value_edit.diagnostics.iter().any(|diagnostic| {
            diagnostic.contains("dependency_shape_update:release_dynamic_dep")
        }));
        assert!(selector_value_edit.diagnostics.iter().any(|diagnostic| {
            diagnostic.contains("dependency_shape_update:activate_dynamic_dep")
        }));
    }

    #[test]
    fn treecalc_context_raw_reference_literal_array_resolves_through_oxfml_host_reference_path() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:raw-reference-literal",
            ))
            .unwrap();
        let a_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("A", "=3"))
            .unwrap();
        let c_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("C", "=5"))
            .unwrap();
        let sum_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Sum", "=SUM({A,C,A})"),
            )
            .unwrap();

        let result = context.recalculate(&workspace_id).unwrap();

        assert_eq!(result.run_state, OxCalcTreeRunState::Published);
        assert_eq!(
            result.published_values.get(&sum_id),
            Some(&"11".to_string())
        );
        assert!(
            !result.diagnostics.iter().any(|diagnostic| diagnostic
                .contains("typed_exclusion:reference_literal_collection_raw_context_pending")),
            "reference literal arrays should execute through OxFml host packets without a typed exclusion: {:?}",
            result.diagnostics
        );
        assert!(a_id != c_id);
    }

    #[test]
    fn treecalc_context_contrasts_static_reference_array_and_dynamic_ctro_indirect() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:static-reference-vs-ctro",
            ))
            .unwrap();
        let a_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("A", "=2"))
            .unwrap();
        let b_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("B", "=3"))
            .unwrap();
        let static_sum_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("StaticSum", "=SUM({A,B,A})"),
            )
            .unwrap();
        let dynamic_sum_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("DynamicSum", "=SUM(A,INDIRECT(\"B\"),A)"),
            )
            .unwrap();

        let result = context.recalculate(&workspace_id).unwrap();

        assert_eq!(
            result.run_state,
            OxCalcTreeRunState::Published,
            "static-vs-ctro recalculation should publish: reject={:?}; diagnostics={:?}",
            result.reject_detail,
            result.diagnostics
        );
        assert_eq!(
            result.published_values.get(&static_sum_id),
            Some(&"7".to_string())
        );
        assert_eq!(
            result.published_values.get(&dynamic_sum_id),
            Some(&"7".to_string())
        );

        let static_edges = result
            .dependency_graph
            .edges_by_owner
            .get(&static_sum_id)
            .expect("static reference array should declare member-value dependencies");
        assert_eq!(
            static_edges
                .iter()
                .filter(|edge| edge.target_node_id == a_id
                    && edge.kind == DependencyDescriptorKind::TreeReferenceCollectionMemberValue)
                .count(),
            2
        );
        assert!(static_edges.iter().any(|edge| {
            edge.target_node_id == b_id
                && edge.kind == DependencyDescriptorKind::TreeReferenceCollectionMemberValue
        }));

        let dynamic_edges = result
            .dependency_graph
            .edges_by_owner
            .get(&dynamic_sum_id)
            .expect("dynamic INDIRECT should keep static and CTRO dependencies");
        assert!(dynamic_edges.iter().any(|edge| {
            edge.target_node_id == a_id && edge.kind == DependencyDescriptorKind::StaticDirect
        }));
        assert!(dynamic_edges.iter().any(|edge| {
            edge.target_node_id == b_id && edge.kind == DependencyDescriptorKind::DynamicPotential
        }));
        assert!(result.runtime_effects.iter().any(|effect| {
            effect.family == RuntimeEffectFamily::DynamicDependency
                && effect
                    .detail
                    .contains(&format!("owner_node:{dynamic_sum_id}"))
                && effect.detail.contains(&format!("target_node:{b_id}"))
        }));
        assert!(result.diagnostics.iter().any(|diagnostic| {
            diagnostic.contains(&format!(
                "ctro_reference_text_resolution:owner={dynamic_sum_id};target={b_id};text=B"
            ))
        }));
    }

    #[test]
    fn treecalc_context_raw_sibling_navigation_resolves_through_oxfml_host_reference_path() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new("workspace:raw-sibling"))
            .unwrap();
        let year_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("Y2005", ""))
            .unwrap();
        let q1_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Q1", "=SUM({Bonus})+@NEXT.Margin+2").under(year_id),
            )
            .unwrap();
        let q2_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Q2", "=@PREV.Net+1").under(year_id),
            )
            .unwrap();
        let q1_net_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Net", "=7").under(q1_id),
            )
            .unwrap();
        context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Bonus", "=1").under(q1_id),
            )
            .unwrap();
        let q2_margin_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Margin", "=11").under(q2_id),
            )
            .unwrap();
        let qualified_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Qualified", "=Q2.@PREV.Net+Q1.@NEXT.Margin")
                    .under(year_id),
            )
            .unwrap();

        let result = context.recalculate(&workspace_id).unwrap();

        assert_eq!(
            result.run_state,
            OxCalcTreeRunState::Published,
            "sibling navigation failed: reject={:?}; diagnostics={:?}",
            result.reject_detail,
            result.diagnostics
        );
        assert_eq!(result.published_values.get(&q1_id), Some(&"14".to_string()));
        assert_eq!(result.published_values.get(&q2_id), Some(&"8".to_string()));
        assert_eq!(
            result.published_values.get(&qualified_id),
            Some(&"18".to_string())
        );
        let qualified_edges = result
            .dependency_graph
            .edges_by_owner
            .get(&qualified_id)
            .expect("qualified sibling references should publish dependency edges");
        assert!(
            qualified_edges
                .iter()
                .any(|edge| edge.target_node_id == q1_net_id)
        );
        assert!(
            qualified_edges
                .iter()
                .any(|edge| edge.target_node_id == q2_margin_id)
        );
        assert!(
            result
                .dependency_graph
                .descriptors_by_owner
                .get(&qualified_id)
                .into_iter()
                .flatten()
                .any(|descriptor| descriptor
                    .carrier_detail
                    .contains("qualified_sibling_offset")),
            "qualified sibling dependencies must stay rebind-sensitive: {:?}",
            result
                .dependency_graph
                .descriptors_by_owner
                .get(&qualified_id)
        );
        assert_ne!(q1_id, q2_id);
        assert!(q2_margin_id.0 > 0);
    }

    #[test]
    fn treecalc_context_raw_sibling_navigation_out_of_range_is_typed_pending() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:raw-sibling-out-of-range",
            ))
            .unwrap();
        let year_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("Y2005", ""))
            .unwrap();
        context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Q1", "=1").under(year_id),
            )
            .unwrap();
        let total_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Total", "=@NEXT").under(year_id),
            )
            .unwrap();

        let result = context.recalculate(&workspace_id).unwrap();

        assert_eq!(result.run_state, OxCalcTreeRunState::Rejected);
        assert!(
            result.diagnostics.iter().any(|diagnostic| {
                diagnostic.contains("bound_formula_profile_selector_unresolved")
                    || diagnostic.contains("UnresolvedReference { target: \"@NEXT\" }")
            }),
            "out-of-range sibling navigation must stay typed through the profile seam, diagnostics={:?}",
            result.diagnostics
        );
        assert!(total_id.0 > 0);
    }

    #[test]
    fn treecalc_context_raw_ancestor_anchors_resolve_through_oxfml_host_reference_path() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new("workspace:ancestor-anchors"))
            .unwrap();
        let accounts_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("Accounts", "4"))
            .unwrap();
        let year_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Y2005", "").under(accounts_id),
            )
            .unwrap();
        let q1_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Q1", "").under(year_id),
            )
            .unwrap();
        context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Margin", "2").under(q1_id),
            )
            .unwrap();
        let parent_anchor_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("ParentAnchor", "=^").under(q1_id),
            )
            .unwrap();
        let parent_child_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("ParentChild", "=^.Margin").under(q1_id),
            )
            .unwrap();
        let total_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Total", "3").under(year_id),
            )
            .unwrap();
        let grandparent_child_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("GrandparentChild", "=^^.Total").under(q1_id),
            )
            .unwrap();
        let deep_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Deep", "=^^^").under(q1_id),
            )
            .unwrap();

        let result = context.recalculate(&workspace_id).unwrap();

        assert_eq!(
            result.run_state,
            OxCalcTreeRunState::Published,
            "ancestor anchors failed: reject={:?}; diagnostics={:?}",
            result.reject_detail,
            result.diagnostics
        );
        assert_eq!(
            result.published_values.get(&parent_anchor_id),
            Some(&"0".to_string())
        );
        assert_eq!(
            result.published_values.get(&parent_child_id),
            Some(&"2".to_string())
        );
        assert_eq!(
            result.published_values.get(&grandparent_child_id),
            Some(&"3".to_string())
        );
        assert_eq!(
            result.published_values.get(&deep_id),
            Some(&"4".to_string())
        );
        assert!(total_id.0 > 0);
    }

    #[test]
    fn treecalc_context_raw_qualified_parent_accessor_resolves_through_oxfml_host_reference_path() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:qualified-parent-accessor",
            ))
            .unwrap();
        let year_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("Y2005", "7"))
            .unwrap();
        let q1_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Q1", "=5").under(year_id),
            )
            .unwrap();
        assert!(q1_id != year_id);
        // Total is a child of Y2005, reachable as the tail off Q1.@PARENT.
        let total_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Total", "=3").under(year_id),
            )
            .unwrap();
        // Owner sits beside Q1 so the bare base name `Q1` resolves through the
        // walk-up scope, mirroring the qualified-sibling navigation corpus.
        // base.^ : the parent of Q1 is Y2005 (value 7).
        let qualified_parent_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("QualifiedParent", "=Q1.@PARENT").under(year_id),
            )
            .unwrap();
        // base.^.tail : the parent of Q1 is Y2005, then descend to Total (value 3).
        let qualified_parent_tail_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("QualifiedParentTail", "=Q1.@PARENT.Total")
                    .under(year_id),
            )
            .unwrap();

        let result = context.recalculate(&workspace_id).unwrap();

        assert_eq!(
            result.run_state,
            OxCalcTreeRunState::Published,
            "qualified parent accessor failed: reject={:?}; diagnostics={:?}",
            result.reject_detail,
            result.diagnostics
        );
        assert!(
            !result.diagnostics.iter().any(|diagnostic| diagnostic
                .contains("qualified_parent_accessor_host_reference_packet_pending")),
            "qualified parent accessor must no longer be a typed exclusion: {:?}",
            result.diagnostics
        );
        assert_eq!(
            result.published_values.get(&qualified_parent_id),
            Some(&"7".to_string())
        );
        assert_eq!(
            result.published_values.get(&qualified_parent_tail_id),
            Some(&"3".to_string())
        );
        // The qualified parent reference must lower to a rebind-sensitive edge onto
        // the resolved target (Total) so structural edits re-bind it.
        let tail_edges = result
            .dependency_graph
            .edges_by_owner
            .get(&qualified_parent_tail_id)
            .expect("qualified parent tail should publish dependency edges");
        assert!(
            tail_edges
                .iter()
                .any(|edge| edge.target_node_id == total_id)
        );
        assert!(
            result
                .dependency_graph
                .descriptors_by_owner
                .get(&qualified_parent_tail_id)
                .into_iter()
                .flatten()
                .any(|descriptor| descriptor
                    .carrier_detail
                    .contains("qualified_parent_offset")),
            "qualified parent dependencies must stay rebind-sensitive: {:?}",
            result
                .dependency_graph
                .descriptors_by_owner
                .get(&qualified_parent_tail_id)
        );
    }

    #[test]
    fn treecalc_context_raw_bracket_escaped_paths_resolve_through_oxfml_host_reference_path() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new("workspace:escaped-paths"))
            .unwrap();
        context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("Sales Q1", "5"))
            .unwrap();
        context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("Foo[Bar", "6"))
            .unwrap();
        context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("Foo]Bar", "7"))
            .unwrap();
        context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("Foo'Bar", "8"))
            .unwrap();
        context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("@Special", "9"))
            .unwrap();
        let region_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("Region", ""))
            .unwrap();
        context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Net Revenue", "10").under(region_id),
            )
            .unwrap();

        let sales_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("SalesFormula", "=[Sales Q1]+1"),
            )
            .unwrap();
        let revenue_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("RevenueFormula", "=Region.[Net Revenue]+1"),
            )
            .unwrap();
        let open_bracket_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("OpenBracketFormula", "=[Foo'[Bar]+1"),
            )
            .unwrap();
        let close_bracket_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("CloseBracketFormula", "=[Foo']Bar]+1"),
            )
            .unwrap();
        let apostrophe_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("ApostropheFormula", "=[Foo''Bar]+1"),
            )
            .unwrap();
        let at_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("AtFormula", "=['@Special]+1"),
            )
            .unwrap();

        let result = context.recalculate(&workspace_id).unwrap();

        assert_eq!(
            result.run_state,
            OxCalcTreeRunState::Published,
            "escaped paths failed: reject={:?}; diagnostics={:?}",
            result.reject_detail,
            result.diagnostics
        );
        assert_eq!(
            result.published_values.get(&sales_id),
            Some(&"6".to_string())
        );
        assert_eq!(
            result.published_values.get(&revenue_id),
            Some(&"11".to_string())
        );
        assert_eq!(
            result.published_values.get(&open_bracket_id),
            Some(&"7".to_string())
        );
        assert_eq!(
            result.published_values.get(&close_bracket_id),
            Some(&"8".to_string())
        );
        assert_eq!(
            result.published_values.get(&apostrophe_id),
            Some(&"9".to_string())
        );
        assert_eq!(result.published_values.get(&at_id), Some(&"10".to_string()));
    }

    #[test]
    fn treecalc_context_meta_nodes_are_invisible_to_name_and_sibling_resolution() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new("workspace:meta-sibling"))
            .unwrap();
        let section_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("Section", ""))
            .unwrap();
        let rate_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Rate", "0.1").under(section_id),
            )
            .unwrap();
        let config_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Config", "")
                    .with_meta(true)
                    .under(section_id),
            )
            .unwrap();
        let secret_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Secret", "42")
                    .with_meta(true)
                    .under(config_id),
            )
            .unwrap();
        let previous_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Previous", "=Rate").under(section_id),
            )
            .unwrap();

        let result = context.recalculate(&workspace_id).unwrap();
        let workspace_view = context.workspace_view(&workspace_id).unwrap();
        assert_eq!(
            result.published_values.get(&previous_id),
            Some(&"0.1".to_string())
        );
        assert!(
            result
                .dependency_graph
                .edges_by_owner
                .get(&previous_id)
                .into_iter()
                .flatten()
                .any(|edge| edge.target_node_id == rate_id)
        );
        assert!(
            workspace_view
                .nodes
                .iter()
                .any(|node| node.node_id == config_id && node.is_meta)
        );
        assert!(
            workspace_view
                .nodes
                .iter()
                .any(|node| node.node_id == secret_id && node.is_meta)
        );

        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new("workspace:meta-hidden"))
            .unwrap();
        let section_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("Section", ""))
            .unwrap();
        context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Rate", "0.1").under(section_id),
            )
            .unwrap();
        let config_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Config", "")
                    .with_meta(true)
                    .under(section_id),
            )
            .unwrap();
        let secret_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Secret", "42")
                    .with_meta(true)
                    .under(config_id),
            )
            .unwrap();
        let hidden_name_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("HiddenName", "=Secret").under(section_id),
            )
            .unwrap();

        let result = context.recalculate(&workspace_id).unwrap();
        let workspace_view = context.workspace_view(&workspace_id).unwrap();

        assert!(
            result.diagnostics.iter().any(|diagnostic| {
                diagnostic.contains("unresolved identifier 'Secret'")
                    || diagnostic.contains("candidate_rejected:OxFml bind")
                    || diagnostic
                        .contains("oxfml_returned_value_surface_payload_summary:Error(Name)")
            }),
            "hidden meta subtree lookup should remain unresolved, diagnostics={:?}",
            result.diagnostics
        );
        assert!(
            !result
                .dependency_graph
                .edges_by_owner
                .get(&hidden_name_id)
                .into_iter()
                .flatten()
                .any(|edge| edge.target_node_id == secret_id)
        );
        assert!(
            workspace_view
                .nodes
                .iter()
                .any(|node| node.node_id == config_id && node.is_meta)
        );
        assert!(
            workspace_view
                .nodes
                .iter()
                .any(|node| node.node_id == secret_id && node.is_meta)
        );
    }

    #[test]
    fn treecalc_context_raw_metadata_accessors_resolve_through_oxfml_host_reference_path() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new("workspace:meta-accessors"))
            .unwrap();
        let section_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("Section", "7"))
            .unwrap();
        let rate_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Rate", "0.1").under(section_id),
            )
            .unwrap();
        context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Config", "")
                    .with_meta(true)
                    .under(section_id),
            )
            .unwrap();
        let parent_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("ParentProbe", "=@PARENT+1").under(section_id),
            )
            .unwrap();
        let name_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("NameProbe", "=@NAME").under(section_id),
            )
            .unwrap();
        let index_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("IndexProbe", "=@INDEX").under(section_id),
            )
            .unwrap();
        let formula_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("FormulaProbe", "=@FORMULA").under(section_id),
            )
            .unwrap();
        let qualified_name_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("QualifiedName", "=Rate.@NAME").under(section_id),
            )
            .unwrap();
        let qualified_index_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("QualifiedIndex", "=IndexProbe.@INDEX").under(section_id),
            )
            .unwrap();

        let result = context.recalculate(&workspace_id).unwrap();

        assert_eq!(
            result.run_state,
            OxCalcTreeRunState::Published,
            "metadata accessors failed: reject={:?}; diagnostics={:?}",
            result.reject_detail,
            result.diagnostics
        );
        assert_eq!(
            result.published_values.get(&parent_id),
            Some(&"8".to_string())
        );
        assert_eq!(
            result.published_values.get(&name_id),
            Some(&"NameProbe".to_string())
        );
        assert_eq!(
            result.published_values.get(&index_id),
            Some(&"4".to_string())
        );
        assert_eq!(
            result.published_values.get(&formula_id),
            Some(&"=@FORMULA".to_string())
        );
        assert_eq!(
            result.published_values.get(&qualified_name_id),
            Some(&"Rate".to_string())
        );
        assert_eq!(
            result.published_values.get(&qualified_index_id),
            Some(&"4".to_string())
        );
        assert!(rate_id.0 > 0);
    }

    #[test]
    fn treecalc_context_failed_edits_do_not_consume_stable_node_ids() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new("workspace:ids"))
            .unwrap();
        let missing_workspace = OxCalcTreeWorkspaceId::new("workspace:missing");

        assert!(
            context
                .add_node(&missing_workspace, OxCalcTreeNodeCreate::new("Lost", "=1"))
                .is_err()
        );
        let a_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("A", "=1"))
            .unwrap();
        assert_eq!(a_id, TreeNodeId(2));

        assert!(
            context
                .add_node(
                    &workspace_id,
                    OxCalcTreeNodeCreate::new("Bad", "=2").under(TreeNodeId(999))
                )
                .is_err()
        );
        let b_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("B", "=A+1"))
            .unwrap();
        assert_eq!(b_id, TreeNodeId(3));
    }

    #[test]
    fn treecalc_context_uses_oxfml_scope_when_resolving_host_name_candidates() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new("workspace:lexical"))
            .unwrap();
        let a_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("A", "=99"))
            .unwrap();
        let b_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("B", "=LET(A,1,A+1)"),
            )
            .unwrap();
        let c_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("C", "=LAMBDA(A,A+1)(3)"),
            )
            .unwrap();
        let d_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("D", "=LET(F,LAMBDA(A,A+1),F(3))"),
            )
            .unwrap();

        let result = context.recalculate(&workspace_id).unwrap();

        assert_eq!(result.run_state, OxCalcTreeRunState::Published);
        assert_eq!(result.published_values.get(&a_id), Some(&"99".to_string()));
        assert_eq!(result.published_values.get(&b_id), Some(&"2".to_string()));
        assert_eq!(result.published_values.get(&c_id), Some(&"4".to_string()));
        assert_eq!(result.published_values.get(&d_id), Some(&"4".to_string()));
        for node_id in [b_id, c_id, d_id] {
            assert!(
                result
                    .dependency_graph
                    .edges_by_owner
                    .get(&node_id)
                    .is_none_or(Vec::is_empty),
                "OxCalc must not add a host-name dependency for an OxFml lexical binding"
            );
        }
    }

    #[test]
    fn treecalc_context_can_call_lambda_value_published_by_another_node() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new("workspace:node-callable"))
            .unwrap();
        let f_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("F", "=LAMBDA(x,x+1)"),
            )
            .unwrap();
        let result_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("Result", "=F(12)"))
            .unwrap();

        let state = context.workspaces.get(&workspace_id).unwrap();
        let catalog_build = build_context_formula_catalog(state, &context.options).unwrap();
        let f_descriptor = catalog_build
            .dependency_descriptors
            .iter()
            .find(|descriptor| {
                descriptor.owner_node_id == result_id
                    && descriptor.target_node_id == Some(f_id)
                    && descriptor.kind == DependencyDescriptorKind::StaticDirect
            })
            .expect("callable invocation should have a static descriptor to the referenced node");
        assert!(
            f_descriptor
                .carrier_detail
                .contains("bound_formula_profile_reference"),
            "profile references should be projected from BoundFormula, got {}",
            f_descriptor.carrier_detail
        );

        let result = context.recalculate(&workspace_id).unwrap();

        assert_eq!(
            result.run_state,
            OxCalcTreeRunState::Published,
            "callable profile reference run failed: reject={:?}; diagnostics={:?}",
            result.reject_detail,
            result.diagnostics
        );
        assert_eq!(
            result.published_values.get(&f_id),
            Some(&"#CALC!".to_string())
        );
        assert_eq!(
            result.published_values.get(&result_id),
            Some(&"13".to_string())
        );
        let callable = &result.published_calc_values[&f_id];
        assert_eq!(callable.core, CoreValue::Error(WorksheetErrorCode::Calc));
        assert!(matches!(
            callable.rich.as_deref(),
            Some(RichValue::Callable(_))
        ));
        let result_edges = result
            .dependency_graph
            .edges_by_owner
            .get(&result_id)
            .expect("callable invocation should depend on the referenced node");
        assert!(result_edges.iter().any(|edge| {
            edge.target_node_id == f_id && edge.kind == DependencyDescriptorKind::StaticDirect
        }));
    }

    // W062 R4.1 — direct callable capture regression pins (D3 §0 probes 1 & 2).
    //
    // Pins the GREEN behaviors verified in D3 §0 so they cannot silently
    // regress when tree scheduling moves off `pull_full_closure` to seeded
    // closure (D3 §8, bead R4.9). Probe 1 pins the value-level property (edit
    // a captured node, the caller recomputes); probe 2 pins the graph shape
    // (the capture edge F->A and the call edge Result->F) that is what makes
    // the invalidation correct under seeded closure — a value-only probe would
    // pass today even with a broken graph because the full sweep re-evaluates
    // everything. The RED transitive-callable probe (D3 §0 probe 3) is
    // deliberately NOT here: it is a fail-until-fixed test that lands WITH its
    // fix in bead R4.9.

    #[test]
    fn treecalc_context_pins_direct_callable_capture_recomputes_on_captured_edit() {
        // D3 §0 probe 1 (green): edit the captured node, the caller recomputes.
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:callable-capture-recompute",
            ))
            .unwrap();
        let a_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("A", "=3"))
            .unwrap();
        context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("F", "=LAMBDA(x,x+A)"),
            )
            .unwrap();
        let result_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("Result", "=F(2)"))
            .unwrap();

        let result = context.recalculate(&workspace_id).unwrap();
        assert_eq!(
            result.run_state,
            OxCalcTreeRunState::Published,
            "direct callable capture recalc must publish: reject={:?}; diagnostics={:?}",
            result.reject_detail,
            result.diagnostics
        );
        assert_eq!(
            result.published_values.get(&result_id),
            Some(&"5".to_string()),
            "Result==F(2)==2+A==2+3 must publish 5"
        );

        // Edit the captured node A; the caller Result must recompute: dirty
        // closure of A reaches F via the capture edge (F->A) and then Result
        // via the call edge (Result->F).
        context
            .set_node_formula_text(&workspace_id, a_id, "=10")
            .unwrap();
        let result = context.recalculate(&workspace_id).unwrap();
        assert_eq!(
            result.run_state,
            OxCalcTreeRunState::Published,
            "recalc after captured-node edit must publish: reject={:?}; diagnostics={:?}",
            result.reject_detail,
            result.diagnostics
        );
        assert_eq!(
            result.published_values.get(&result_id),
            Some(&"12".to_string()),
            "after A==10, Result==F(2)==2+A==2+10 must recompute to 12"
        );
    }

    #[test]
    fn treecalc_context_pins_direct_callable_capture_dependency_edges() {
        // D3 §0 probe 2 (green): edges — F has edge->A (capture), Result has
        // edge->F (call), both StaticDirect, in edges_by_owner.
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:callable-capture-edges",
            ))
            .unwrap();
        let a_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("A", "=3"))
            .unwrap();
        let f_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("F", "=LAMBDA(x,x+A)"),
            )
            .unwrap();
        let result_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("Result", "=F(2)"))
            .unwrap();

        let result = context.recalculate(&workspace_id).unwrap();
        assert_eq!(
            result.run_state,
            OxCalcTreeRunState::Published,
            "callable capture recalc must publish before asserting edges: reject={:?}; diagnostics={:?}",
            result.reject_detail,
            result.diagnostics
        );

        // Capture edge: F depends on the captured free host reference A.
        let f_edges = result
            .dependency_graph
            .edges_by_owner
            .get(&f_id)
            .expect("lambda capturing free host reference A must own a dependency edge");
        assert!(
            f_edges.iter().any(|edge| {
                edge.target_node_id == a_id && edge.kind == DependencyDescriptorKind::StaticDirect
            }),
            "F=LAMBDA(x,x+A) must carry a StaticDirect capture edge to A; edges={f_edges:?}"
        );

        // Call edge: Result depends on the invoked callable F.
        let result_edges = result
            .dependency_graph
            .edges_by_owner
            .get(&result_id)
            .expect("callable invocation Result=F(2) must own a dependency edge");
        assert!(
            result_edges.iter().any(|edge| {
                edge.target_node_id == f_id && edge.kind == DependencyDescriptorKind::StaticDirect
            }),
            "Result=F(2) must carry a StaticDirect call edge to F; edges={result_edges:?}"
        );
    }

    #[test]
    fn treecalc_context_builds_qualified_children_graph_from_bound_formula() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:bound-selector-graph",
            ))
            .unwrap();
        let base_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("Base", ""))
            .unwrap();
        let a_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("A", "=2").under(base_id),
            )
            .unwrap();
        let b_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("B", "=3").under(base_id),
            )
            .unwrap();
        let total_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Total", "=SUM(Base.@CHILDREN)"),
            )
            .unwrap();

        let state = context.workspaces.get(&workspace_id).unwrap();
        let catalog_build = build_context_formula_catalog(state, &context.options).unwrap();
        let total_binding = catalog_build.catalog.try_get_binding(total_id).unwrap();
        assert!(
            total_binding.expression.explicit_references().is_empty(),
            "host selector product references should be carried by BoundFormula expressions"
        );
        assert!(
            total_binding
                .expression
                .bound_formula()
                .is_some_and(|bound| matches!(
                    bound.root,
                    oxfml_core::binding::BoundExpr::FunctionCall { ref args, .. }
                        if args.iter().any(|arg| matches!(
                            arg,
                            oxfml_core::binding::BoundExpr::Reference(
                                oxfml_core::binding::ReferenceExpr::Atom(
                                    oxfml_core::binding::NormalizedReference::ProfileSymbolic(_)
                                )
                            )
                        ))
                ))
        );

        assert!(
            catalog_build
                .dependency_descriptors
                .iter()
                .any(|descriptor| {
                    descriptor.owner_node_id == total_id
                        && descriptor.kind
                            == DependencyDescriptorKind::TreeReferenceCollectionMembership
                        && descriptor
                            .tree_reference_collection
                            .as_ref()
                            .is_some_and(|collection| collection.base_node_id == base_id)
                })
        );
        assert!(
            catalog_build
                .dependency_descriptors
                .iter()
                .any(|descriptor| {
                    descriptor.owner_node_id == total_id
                        && descriptor.kind
                            == DependencyDescriptorKind::TreeReferenceCollectionMembership
                        && descriptor.carrier_detail.contains("treecalc_children_v1")
                })
        );
        for member_id in [a_id, b_id] {
            assert!(
                catalog_build
                    .dependency_descriptors
                    .iter()
                    .any(|descriptor| {
                        descriptor.owner_node_id == total_id
                            && descriptor.target_node_id == Some(member_id)
                            && descriptor.kind
                                == DependencyDescriptorKind::TreeReferenceCollectionMemberValue
                    })
            );
        }

        let result = context.recalculate(&workspace_id).unwrap();
        assert_eq!(result.run_state, OxCalcTreeRunState::Published);
        assert_eq!(
            result.published_values.get(&total_id),
            Some(&"5".to_string())
        );
    }

    #[test]
    fn treecalc_context_rejects_membership_write_to_derived_children_collection() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:derived-children-membership-write",
            ))
            .unwrap();
        let base_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("Base", ""))
            .unwrap();
        let a_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("A", "=2").under(base_id),
            )
            .unwrap();
        let b_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("B", "=3").under(base_id),
            )
            .unwrap();
        let total_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Total", "=SUM(Base.@CHILDREN)"),
            )
            .unwrap();

        let result = context.recalculate(&workspace_id).unwrap();
        let source_reference_handle = result
            .dependency_graph
            .descriptors_by_owner
            .get(&total_id)
            .unwrap()
            .iter()
            .find_map(|descriptor| {
                descriptor
                    .tree_reference_collection
                    .as_ref()
                    .map(|collection| collection.host_ref_handle.clone())
            })
            .unwrap();

        let error = context
            .apply_edit_transaction(
                OxCalcTreeEditTransaction::new(workspace_id.clone()).with_edit(
                    OxCalcTreeEdit::SetReferenceCollectionMembership {
                        owner_node_id: total_id,
                        source_reference_handle: source_reference_handle.clone(),
                        member_node_ids: vec![b_id, a_id],
                    },
                ),
            )
            .unwrap_err();

        assert!(matches!(
            error,
            OxCalcTreeContextError::ReferenceCollectionNotEditable {
                owner_node_id,
                source_reference_handle: rejected_handle,
                family: TreeReferenceCollectionFamily::ChildrenV1,
            } if owner_node_id == total_id && rejected_handle == source_reference_handle
        ));
    }

    #[test]
    fn treecalc_context_rejects_membership_write_to_unknown_collection_handle() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:unknown-membership-write",
            ))
            .unwrap();
        let a_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("A", "=2"))
            .unwrap();
        let total_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("Total", "=A"))
            .unwrap();

        let error = context
            .apply_edit_transaction(
                OxCalcTreeEditTransaction::new(workspace_id.clone()).with_edit(
                    OxCalcTreeEdit::SetReferenceCollectionMembership {
                        owner_node_id: total_id,
                        source_reference_handle: "treecalc-hostref:v1:missing".to_string(),
                        member_node_ids: vec![a_id],
                    },
                ),
            )
            .unwrap_err();

        assert!(matches!(
            error,
            OxCalcTreeContextError::UnknownReferenceCollection {
                owner_node_id,
                source_reference_handle,
            } if owner_node_id == total_id
                && source_reference_handle == "treecalc-hostref:v1:missing"
        ));
    }

    #[test]
    fn treecalc_context_resolves_symbolic_base_children_sugar() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:symbolic-base-children-sugar",
            ))
            .unwrap();
        let accounts_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("Accounts", ""))
            .unwrap();
        let year_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("2005", "").under(accounts_id),
            )
            .unwrap();
        let q1_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Q1", "").under(year_id),
            )
            .unwrap();
        context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Income", "10").under(q1_id),
            )
            .unwrap();
        context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Margin", "20").under(q1_id),
            )
            .unwrap();
        context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Net", "30").under(q1_id),
            )
            .unwrap();
        let total_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Total", "=SUM(Q1.*)").under(year_id),
            )
            .unwrap();
        let result = context.recalculate(&workspace_id).unwrap();

        assert_eq!(
            result.run_state,
            OxCalcTreeRunState::Published,
            "symbolic-base children sugar failed: reject={:?}; diagnostics={:?}",
            result.reject_detail,
            result.diagnostics
        );
        assert_eq!(
            result.published_values.get(&total_id),
            Some(&"60".to_string())
        );
    }

    #[test]
    fn treecalc_context_builds_dotted_path_graph_from_bound_formula() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new(
                "workspace:bound-dotted-path",
            ))
            .unwrap();
        let b_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("B", ""))
            .unwrap();
        let c_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("C", "").under(b_id),
            )
            .unwrap();
        let a_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("A", "=4").under(c_id),
            )
            .unwrap();
        let total_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("Total", "=B.C.A+1"),
            )
            .unwrap();

        let state = context.workspaces.get(&workspace_id).unwrap();
        let catalog_build = build_context_formula_catalog(state, &context.options).unwrap();
        let total_binding = catalog_build.catalog.try_get_binding(total_id).unwrap();
        assert!(total_binding.expression.explicit_references().is_empty());
        assert!(
            total_binding
                .expression
                .bound_formula()
                .is_some_and(|bound| bound.normalized_references.iter().any(|reference| {
                    matches!(
                        reference,
                        oxfml_core::binding::NormalizedReference::ProfileSymbolic(_)
                    )
                }))
        );
        assert!(
            catalog_build
                .dependency_descriptors
                .iter()
                .any(|descriptor| {
                    descriptor.owner_node_id == total_id
                        && descriptor.target_node_id == Some(a_id)
                        && descriptor.kind == DependencyDescriptorKind::StaticDirect
                        && descriptor
                            .carrier_detail
                            .contains("bound_formula_profile_reference")
                })
        );

        let result = context.recalculate(&workspace_id).unwrap();
        assert_eq!(result.run_state, OxCalcTreeRunState::Published);
        assert_eq!(
            result.published_values.get(&total_id),
            Some(&"5".to_string())
        );
    }

    #[test]
    fn treecalc_context_reports_unresolved_host_names_outside_walkup_scope() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new("workspace:unresolved"))
            .unwrap();
        let p_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("P", ""))
            .unwrap();
        let q_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("Q", ""))
            .unwrap();
        context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("A", "=1").under(p_id),
            )
            .unwrap();
        context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("A", "=2").under(q_id),
            )
            .unwrap();
        let b_id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("B", "=A+1"))
            .unwrap();

        let result = context.recalculate(&workspace_id).unwrap();

        assert!(result.diagnostics.iter().any(|diagnostic| {
            diagnostic.contains("oxfml_bind_diagnostic:unresolved identifier 'A'")
                || diagnostic.contains("candidate_rejected:OxFml bind")
        }));
        assert!(result.binding_diagnostics.iter().any(|diagnostic| {
            diagnostic.owner_node_id == b_id
                && diagnostic.message == "unresolved identifier 'A'"
                && diagnostic.span_start_utf8 == 1
                && diagnostic.span_len_utf8 == 1
        }));
    }

    fn run_local_engine_fixture(
        options: OxCalcTreeContextOptions,
        formula_catalog: TreeFormulaCatalog,
        fixture_publication_values: BTreeMap<TreeNodeId, String>,
        run_suffix: &str,
    ) -> OxCalcTreeCalculationOutcome {
        let structural_snapshot = snapshot();
        let fixture_literals = BTreeMap::from([(TreeNodeId(2), "2".to_string())]);
        let node_input_records = structural_snapshot
            .nodes()
            .keys()
            .map(|node_id| {
                fixture_literals.get(node_id).map_or_else(
                    || NodeInputRecord::empty(*node_id, 1),
                    |value| NodeInputRecord::literal(*node_id, value.clone(), 1),
                )
            })
            .collect::<Vec<_>>();
        let node_input_snapshot = NodeInputSnapshot::create(node_input_records).unwrap();
        let workspace_revision = WorkspaceRevision::new(
            "workspace:local-engine-fixture",
            structural_snapshot,
            node_input_snapshot,
            NamespaceSnapshot::current_absent(),
        );
        let artifacts = LocalTreeCalcEngine
            .execute(LocalTreeCalcInput {
                layer_snapshot_ids: LocalTreeCalcLayerSnapshotIds::current_absent_for_revision(
                    &workspace_revision,
                ),
                workspace_revision,
                formula_catalog,
                formula_dependency_descriptors: None,
                table_snapshots: BTreeMap::new(),
                static_dependency_shape_updates: Vec::new(),
                publication_calc_values: fixture_publication_values
                    .iter()
                    .map(|(node_id, value)| (*node_id, authored_input_text_to_calc_value(value)))
                    .collect(),
                publication_runtime_effects: Vec::new(),
                invalidation_seeds: Vec::new(),
                previous_arg_preparation_profile_version: None,
                candidate_result_id: format!("candidate:{run_suffix}"),
                publication_id: format!("publication:{run_suffix}"),
                environment_context: options.runtime_context(),
            })
            .unwrap();
        let mut outcome = OxCalcTreeCalculationOutcome::from(artifacts);
        outcome.diagnostics.extend(options.diagnostics());
        outcome
    }

    fn fixture_catalog(expression: TreeFormula) -> TreeFormulaCatalog {
        TreeFormulaCatalog::new([TreeFormulaBinding {
            owner_node_id: TreeNodeId(3),
            formula_artifact_id: FormulaArtifactId("formula:b".to_string()),
            bind_artifact_id: Some(BindArtifactId("bind:b".to_string())),
            expression,
        }])
    }

    #[test]
    fn treecalc_context_options_carry_non_narrow_consumer_inputs() {
        let options = OxCalcTreeContextOptions::new()
            .with_session_id("session:tree-host")
            .with_host_capabilities(OxCalcTreeHostCapabilitySnapshot {
                capability_profile_id: "capability-profile:tree-host".to_string(),
                dynamic_dependency_effects: true,
                execution_restriction_effects: true,
                capability_sensitive_effects: true,
                shape_topology_effects: true,
            })
            .with_runtime_policy(OxCalcTreeRuntimePolicy {
                policy_id: "runtime-policy:tree-host".to_string(),
                emit_environment_diagnostics: true,
                project_runtime_effect_overlays: true,
                derivation_trace_enabled: false,
                scheduling_policy: LocalTreeCalcSchedulingPolicy::default(),
            });

        assert_eq!(
            options.runtime_lane,
            OxCalcTreeRuntimeLane::LocalSequentialTreeCalc
        );
        assert_eq!(options.session_id.as_deref(), Some("session:tree-host"));
        assert_eq!(
            options.host_capabilities.capability_profile_id,
            "capability-profile:tree-host"
        );
        assert!(options.host_capabilities.capability_sensitive_effects);
        assert!(options.host_capabilities.shape_topology_effects);
        assert_eq!(options.runtime_policy.policy_id, "runtime-policy:tree-host");
    }

    #[test]
    fn treecalc_context_options_project_runtime_diagnostics() {
        let result = run_local_engine_fixture(
            OxCalcTreeContextOptions::new()
                .with_session_id("session:diagnostic")
                .with_host_capabilities(OxCalcTreeHostCapabilitySnapshot {
                    capability_profile_id: "capability-profile:diagnostic".to_string(),
                    dynamic_dependency_effects: true,
                    execution_restriction_effects: true,
                    capability_sensitive_effects: true,
                    shape_topology_effects: false,
                })
                .with_runtime_policy(OxCalcTreeRuntimePolicy {
                    policy_id: "runtime-policy:diagnostic".to_string(),
                    emit_environment_diagnostics: true,
                    project_runtime_effect_overlays: true,
                    derivation_trace_enabled: false,
                    scheduling_policy: LocalTreeCalcSchedulingPolicy::default(),
                }),
            fixture_catalog(fixture_formula(
                TreeNodeId(3),
                FixtureFormulaAst::Literal {
                    value: "7".to_string(),
                },
            )),
            BTreeMap::new(),
            "diagnostic",
        );

        assert!(result.diagnostics.iter().any(|diagnostic| {
            diagnostic == "oxcalc_tree_context_options_runtime_lane:local_sequential_treecalc"
        }));
        assert!(result.diagnostics.iter().any(|diagnostic| {
            diagnostic == "oxcalc_tree_context_options_session_id:session:diagnostic"
        }));
        assert!(result.diagnostics.iter().any(|diagnostic| {
            diagnostic
                == "oxcalc_tree_context_options_capability_profile_id:capability-profile:diagnostic"
        }));
        assert!(result.diagnostics.iter().any(|diagnostic| {
            diagnostic == "oxcalc_tree_context_options_capability_sensitive:true"
        }));
        assert!(result.diagnostics.iter().any(|diagnostic| {
            diagnostic == "oxcalc_tree_context_options_capability_shape_topology:false"
        }));
        assert!(result.diagnostics.iter().any(|diagnostic| {
            diagnostic == "oxcalc_tree_context_options_runtime_policy_id:runtime-policy:diagnostic"
        }));
    }

    #[test]
    fn treecalc_runtime_derived_effects_use_context_options() {
        let result = run_local_engine_fixture(
            OxCalcTreeContextOptions::new()
                .with_session_id("session:runtime-effects")
                .with_host_capabilities(OxCalcTreeHostCapabilitySnapshot {
                    capability_profile_id: "capability-profile:runtime-effects".to_string(),
                    dynamic_dependency_effects: true,
                    execution_restriction_effects: true,
                    capability_sensitive_effects: false,
                    shape_topology_effects: false,
                })
                .with_runtime_policy(OxCalcTreeRuntimePolicy {
                    policy_id: "runtime-policy:no-overlays".to_string(),
                    emit_environment_diagnostics: true,
                    project_runtime_effect_overlays: false,
                    derivation_trace_enabled: false,
                    scheduling_policy: LocalTreeCalcSchedulingPolicy::default(),
                }),
            fixture_catalog(fixture_formula(
                TreeNodeId(3),
                FixtureFormulaAst::Reference(TreeReference::DynamicPotential {
                    carrier_id: "carrier:dynamic".to_string(),
                    detail: "late_bound_projection".to_string(),
                }),
            )),
            BTreeMap::new(),
            "runtime-effects",
        );

        assert_eq!(result.run_state, OxCalcTreeRunState::Rejected);
        assert!(result.publication_bundle.is_none());
        assert!(result.runtime_effect_overlays.is_empty());
        assert_eq!(result.runtime_effects.len(), 1);
        assert_eq!(
            result.runtime_effects[0].family,
            RuntimeEffectFamily::DynamicDependency
        );
        assert!(
            result.runtime_effects[0]
                .detail
                .contains("session_id:session:runtime-effects")
        );
        assert!(
            result.runtime_effects[0]
                .detail
                .contains("capability_profile_id:capability-profile:runtime-effects")
        );
        assert!(
            result.runtime_effects[0]
                .detail
                .contains("runtime_policy_id:runtime-policy:no-overlays")
        );
        assert!(result.diagnostics.iter().any(|diagnostic| {
            diagnostic == "runtime_effect_environment_session_id:session:runtime-effects"
        }));
        assert!(result.diagnostics.iter().any(|diagnostic| {
            diagnostic
                == "runtime_effect_environment_capability_profile_id:capability-profile:runtime-effects"
        }));
        assert!(result.diagnostics.iter().any(|diagnostic| {
            diagnostic == "runtime_effect_environment_project_overlays:false"
        }));
        assert!(
            result.diagnostics.iter().any(|diagnostic| {
                diagnostic == "runtime_effect_overlay_projection_enabled:false"
            })
        );
        assert!(
            result
                .diagnostics
                .iter()
                .any(|diagnostic| { diagnostic == "runtime_effect_overlay_projection_count:0" })
        );
    }

    #[test]
    fn treecalc_runtime_overlay_projection_is_explicit_on_publish_and_verified_clean() {
        let catalog = fixture_catalog(fixture_formula(
            TreeNodeId(3),
            FixtureFormulaAst::Literal {
                value: "7".to_string(),
            },
        ));
        let published = run_local_engine_fixture(
            OxCalcTreeContextOptions::new(),
            catalog.clone(),
            BTreeMap::new(),
            "publish-overlay-projection",
        );

        assert_eq!(published.run_state, OxCalcTreeRunState::Published);
        assert!(published.runtime_effect_overlays.is_empty());
        assert!(
            published.diagnostics.iter().any(|diagnostic| {
                diagnostic == "runtime_effect_overlay_projection_enabled:true"
            })
        );
        assert!(
            published
                .diagnostics
                .iter()
                .any(|diagnostic| { diagnostic == "runtime_effect_overlay_projection_count:0" })
        );

        let verified_clean = run_local_engine_fixture(
            OxCalcTreeContextOptions::new(),
            catalog,
            BTreeMap::from([(TreeNodeId(3), "7".to_string())]),
            "verified-clean-overlay-projection",
        );

        assert_eq!(verified_clean.run_state, OxCalcTreeRunState::VerifiedClean);
        assert!(verified_clean.runtime_effect_overlays.is_empty());
        assert!(verified_clean.publication_bundle.is_none());
        assert!(
            verified_clean.diagnostics.iter().any(|diagnostic| {
                diagnostic == "runtime_effect_overlay_projection_enabled:true"
            })
        );
        assert!(
            verified_clean
                .diagnostics
                .iter()
                .any(|diagnostic| { diagnostic == "runtime_effect_overlay_projection_count:0" })
        );
    }

    #[test]
    fn direct_context_runtime_path_executes_published_run() {
        let result = run_local_engine_fixture(
            OxCalcTreeContextOptions::new(),
            fixture_catalog(fixture_formula(
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
            )),
            BTreeMap::new(),
            "consumer",
        );

        assert_eq!(result.run_state, OxCalcTreeRunState::Published);
        assert_eq!(result.published_values[&TreeNodeId(3)], "5");
        assert!(result.publication_bundle.is_some());
    }

    #[test]
    fn direct_context_result_exposes_execution_restriction_family_directly() {
        let result = run_local_engine_fixture(
            OxCalcTreeContextOptions::new(),
            fixture_catalog(fixture_formula(
                TreeNodeId(3),
                FixtureFormulaAst::Reference(TreeReference::HostSensitive {
                    carrier_id: "carrier:host".to_string(),
                    detail: "active_selection".to_string(),
                }),
            )),
            BTreeMap::new(),
            "host",
        );

        assert_eq!(result.run_state, OxCalcTreeRunState::Rejected);
        assert!(result.publication_bundle.is_none());
        assert_eq!(
            result.reject_detail.map(|detail| detail.kind),
            Some(RejectKind::HostInjectedFailure)
        );
        assert_eq!(result.runtime_effects.len(), 1);
        assert_eq!(
            result.runtime_effects[0].family,
            RuntimeEffectFamily::ExecutionRestriction
        );
        assert_eq!(result.runtime_effect_overlays.len(), 1);
        assert_eq!(
            result.runtime_effect_overlays[0].key.overlay_kind,
            OverlayKind::ExecutionRestriction
        );
    }

    #[test]
    fn direct_context_result_exposes_capability_sensitive_family_directly() {
        let result = run_local_engine_fixture(
            OxCalcTreeContextOptions::new(),
            fixture_catalog(fixture_formula(
                TreeNodeId(3),
                FixtureFormulaAst::Reference(TreeReference::CapabilitySensitive {
                    carrier_id: "carrier:capability".to_string(),
                    detail: "host_function_availability".to_string(),
                }),
            )),
            BTreeMap::new(),
            "capability",
        );

        assert_eq!(result.run_state, OxCalcTreeRunState::Rejected);
        assert!(result.publication_bundle.is_none());
        assert_eq!(result.runtime_effects.len(), 1);
        assert_eq!(
            result.runtime_effects[0].family,
            RuntimeEffectFamily::CapabilitySensitive
        );
        assert_eq!(result.runtime_effect_overlays.len(), 1);
        assert_eq!(
            result.runtime_effect_overlays[0].key.overlay_kind,
            OverlayKind::ExecutionRestriction
        );
        assert!(
            result
                .diagnostics
                .iter()
                .any(|diagnostic| { diagnostic == "runtime_effect_overlay_projection_count:1" })
        );
    }

    #[test]
    fn direct_context_result_exposes_shape_topology_family_directly() {
        let result = run_local_engine_fixture(
            OxCalcTreeContextOptions::new(),
            fixture_catalog(fixture_formula(
                TreeNodeId(3),
                FixtureFormulaAst::Reference(TreeReference::ShapeTopology {
                    carrier_id: "carrier:shape".to_string(),
                    detail: "range_shape_projection".to_string(),
                }),
            )),
            BTreeMap::new(),
            "shape",
        );

        assert_eq!(result.run_state, OxCalcTreeRunState::Rejected);
        assert!(result.publication_bundle.is_none());
        assert_eq!(result.runtime_effects.len(), 1);
        assert_eq!(
            result.runtime_effects[0].family,
            RuntimeEffectFamily::ShapeTopology
        );
        assert_eq!(result.runtime_effect_overlays.len(), 1);
        assert_eq!(
            result.runtime_effect_overlays[0].key.overlay_kind,
            OverlayKind::ShapeTopology
        );
    }

    #[test]
    fn direct_context_result_exposes_dynamic_dependency_family_directly() {
        let result = run_local_engine_fixture(
            OxCalcTreeContextOptions::new(),
            fixture_catalog(fixture_formula(
                TreeNodeId(3),
                FixtureFormulaAst::Reference(TreeReference::DynamicPotential {
                    carrier_id: "carrier:dynamic".to_string(),
                    detail: "late_bound_projection".to_string(),
                }),
            )),
            BTreeMap::new(),
            "dynamic",
        );

        assert_eq!(result.run_state, OxCalcTreeRunState::Rejected);
        assert_eq!(
            result.reject_detail.map(|detail| detail.kind),
            Some(RejectKind::DynamicDependencyFailure)
        );
        assert_eq!(result.runtime_effects.len(), 1);
        assert_eq!(
            result.runtime_effects[0].family,
            RuntimeEffectFamily::DynamicDependency
        );
        assert_eq!(result.runtime_effect_overlays.len(), 1);
        assert_eq!(
            result.runtime_effect_overlays[0].key.overlay_kind,
            OverlayKind::DynamicDependency
        );
    }
}
