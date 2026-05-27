#![forbid(unsafe_code)]

//! Consumer-facing OxCalc runtime contract for tree-substrate hosts.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use oxfml_core::consumer::runtime::{RuntimeHostFormulaContext, RuntimeHostReferenceSyntaxMatch};
use oxfml_core::{
    BindContext, BindRequest, FormulaSourceRecord, ParseRequest, StructuredReferenceBindRecord,
    bind_formula, parse_formula, project_red_view,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::coordinator::{
    AcceptedCandidateResult, DependencyShapeUpdate, PublicationBundle, RejectDetail, RuntimeEffect,
};
use crate::dependency::{
    DependencyGraph, InvalidationClosure, InvalidationReasonKind, InvalidationSeed,
};
use crate::formula::{
    RelativeReferenceBase, TreeCalcChildrenReferenceCollection, TreeCalcOrderedSelectorFamily,
    TreeCalcOrderedSelectorReferenceCollection, TreeCalcOrderedSelectorTraversalPolicy,
    TreeCalcReferenceCollection, TreeCalcReferenceLiteralArrayCollection,
    TreeCalcReferenceLiteralArrayElement, TreeFormula, TreeFormulaBinding, TreeFormulaCatalog,
    TreeFormulaHostNameBindPacket, TreeFormulaHostValue, TreeFormulaHostValueBinding,
    TreeFormulaReferenceCarrier, TreeReference, resolve_treecalc_ordered_selector_traversal,
    treecalc_host_reference_carrier_from_syntax_match,
};
use crate::recalc::{NodeCalcState, OverlayEntry};
use crate::structural::{
    BindArtifactId, FormulaArtifactId, StructuralEdit, StructuralError, StructuralNode,
    StructuralNodeKind, StructuralSnapshot, StructuralSnapshotId, StructuralTableShape, TreeNodeId,
};
use crate::structured_table::{
    StructuredTableBindRecordIntakeError, StructuredTableContextPacket,
    StructuredTableDependencyLowering, StructuredTableDependencyLoweringRequest,
    StructuredTableReferenceIntake, TableCallerRegion, TableRef, TreeCalcDynamicTableRebindReport,
    TreeCalcDynamicTableRebindRequest, TreeCalcTableCatalogResolution,
    TreeCalcTableCatalogResolveRequest, TreeCalcTableCatalogResolverContext,
    TreeCalcTableCatalogWorkspace, TreeCalcTableDeletedFact, TreeCalcTableDependencyInventory,
    TreeCalcTableLifecycleContextVersions, TreeCalcTableNodeProjection, TreeCalcTableNodeSnapshot,
    TreeCalcTableProjectionError, classify_treecalc_dynamic_table_rebind,
    inventory_treecalc_table_dependency_facts, lower_structured_table_dependencies,
    project_treecalc_table_node_snapshot, resolve_treecalc_table_catalog_reference,
};
use crate::tree_reference_resolution::{
    ContextHostNameResolution, is_meta_effective, resolve_context_host_name_token,
    resolve_context_host_path_token,
};
use crate::treecalc::{
    DerivationTraceRecord, LocalTreeCalcEngine, LocalTreeCalcEnvironmentContext,
    LocalTreeCalcError, LocalTreeCalcInput, LocalTreeCalcRunArtifacts, LocalTreeCalcRunState,
    LocalTreeCalcSchedulingPolicy, treecalc_host_reference_syntax_rules,
};
use crate::workspace_revision::{
    DependencyShapeSnapshot, DependencyShapeSnapshotId, FormulaBindingSnapshot,
    FormulaBindingSnapshotId, NamespaceSnapshot, NamespaceSnapshotId, NodeInputKind,
    NodeInputRecord, NodeInputSnapshot, NodeInputSnapshotId, PublicationSnapshot,
    PublicationSnapshotId, RuntimeOverlaySet, RuntimeOverlaySetId, WorkspaceRevision,
    WorkspaceRevisionError, WorkspaceRevisionId,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OxCalcTreeRunState {
    Published,
    VerifiedClean,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
    pub node_states: BTreeMap<TreeNodeId, NodeCalcState>,
    pub phase_timings_micros: BTreeMap<String, u128>,
    pub diagnostics: Vec<String>,
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
    pub symbol: String,
    pub formula_text: String,
    pub is_meta: bool,
}

impl OxCalcTreeNodeCreate {
    #[must_use]
    pub fn new(symbol: impl Into<String>, formula_text: impl Into<String>) -> Self {
        Self {
            parent_node_id: None,
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OxCalcTreeNodeView {
    pub node_id: TreeNodeId,
    pub symbol: String,
    pub canonical_path: String,
    pub display_path: String,
    pub formula_text: String,
    pub value_text: Option<String>,
    pub input_value_epoch: Option<u64>,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OxCalcTreeWorkspaceView {
    pub workspace_id: OxCalcTreeWorkspaceId,
    pub root_node_id: TreeNodeId,
    pub snapshot_id: StructuralSnapshotId,
    pub workspace_revision_id: WorkspaceRevisionId,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OxCalcTreeWorkspaceSnapshot {
    pub workspace_id: OxCalcTreeWorkspaceId,
    pub root_node_id: TreeNodeId,
    pub structural_snapshot: StructuralSnapshot,
    pub formula_texts: BTreeMap<TreeNodeId, String>,
    pub formula_text_versions: BTreeMap<TreeNodeId, u64>,
    pub input_values: BTreeMap<TreeNodeId, String>,
    pub input_value_epochs: BTreeMap<TreeNodeId, u64>,
    pub value_epoch: u64,
    pub meta_node_ids: BTreeSet<TreeNodeId>,
    pub table_snapshots: BTreeMap<TreeNodeId, TreeCalcTableNodeSnapshot>,
    pub deleted_table_facts: Vec<TreeCalcTableDeletedFact>,
    pub table_state_version: u64,
    pub seeded_published_values: BTreeMap<TreeNodeId, String>,
    pub seeded_published_runtime_effects: Vec<RuntimeEffect>,
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
    #[error("node {node_id} is formula-backed; use set_node_formula_text for formula changes")]
    InputValueOnFormulaNode { node_id: TreeNodeId },
    #[error(transparent)]
    WorkspaceRevision(#[from] WorkspaceRevisionError),
}

#[derive(Debug, Clone)]
struct OxCalcTreeWorkspaceState {
    workspace_id: OxCalcTreeWorkspaceId,
    root_node_id: TreeNodeId,
    snapshot: StructuralSnapshot,
    workspace_revision: WorkspaceRevision,
    formula_binding_snapshot: FormulaBindingSnapshot,
    dependency_shape_snapshot: DependencyShapeSnapshot,
    publication_snapshot: PublicationSnapshot,
    runtime_overlay_set: RuntimeOverlaySet,
    formula_texts: BTreeMap<TreeNodeId, String>,
    formula_text_versions: BTreeMap<TreeNodeId, u64>,
    input_values: BTreeMap<TreeNodeId, String>,
    input_value_epochs: BTreeMap<TreeNodeId, u64>,
    value_epoch: u64,
    meta_node_ids: BTreeSet<TreeNodeId>,
    table_snapshots: BTreeMap<TreeNodeId, TreeCalcTableNodeSnapshot>,
    deleted_table_facts: Vec<TreeCalcTableDeletedFact>,
    table_state_version: u64,
    seeded_published_values: BTreeMap<TreeNodeId, String>,
    seeded_published_runtime_effects: Vec<RuntimeEffect>,
    pending_invalidation_seeds: Vec<InvalidationSeed>,
    pending_formula_edit_diagnostics: Vec<String>,
    pending_node_input_kind_transitions: Vec<ContextNodeInputKindTransition>,
    pending_dependency_shape_updates: Vec<DependencyShapeUpdate>,
    last_result: Option<OxCalcTreeCalculationOutcome>,
}

#[derive(Debug, Clone)]
pub struct OxCalcTreeContext {
    options: OxCalcTreeContextOptions,
    workspaces: BTreeMap<OxCalcTreeWorkspaceId, OxCalcTreeWorkspaceState>,
    next_node_id: u64,
    next_snapshot_id: u64,
    next_candidate_index: u64,
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
            next_node_id: 1,
            next_snapshot_id: 1,
            next_candidate_index: 1,
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
            let namespace_snapshot = namespace_snapshot_for_context(&options, &state.workspace_id);
            if namespace_snapshot.snapshot_id()
                != state.workspace_revision.namespace_snapshot.snapshot_id()
            {
                let has_published_baseline = !state.seeded_published_values.is_empty()
                    || !state.seeded_published_runtime_effects.is_empty();
                replace_namespace_snapshot(state, namespace_snapshot);
                state.pending_invalidation_seeds.clear();
                if has_published_baseline {
                    seed_namespace_recalc_invalidation(state);
                }
                clear_pending_edit_transition_facts(state);
            }
            state.last_result = None;
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
        let state = OxCalcTreeWorkspaceState {
            workspace_id: workspace_id.clone(),
            root_node_id,
            snapshot,
            workspace_revision,
            formula_binding_snapshot,
            dependency_shape_snapshot,
            publication_snapshot,
            runtime_overlay_set,
            formula_texts: BTreeMap::from([(root_node_id, String::new())]),
            formula_text_versions: BTreeMap::from([(root_node_id, 1)]),
            input_values: BTreeMap::new(),
            input_value_epochs: BTreeMap::new(),
            value_epoch: 0,
            meta_node_ids: BTreeSet::new(),
            table_snapshots: BTreeMap::new(),
            deleted_table_facts: Vec::new(),
            table_state_version: 1,
            seeded_published_values: BTreeMap::new(),
            seeded_published_runtime_effects: Vec::new(),
            pending_invalidation_seeds: Vec::new(),
            pending_formula_edit_diagnostics: Vec::new(),
            pending_node_input_kind_transitions: Vec::new(),
            pending_dependency_shape_updates: Vec::new(),
            last_result: None,
        };
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
        let node_id = self.next_node_id();
        let snapshot_id = self.next_snapshot_id();
        {
            let state = self.workspace_mut(workspace_id)?;
            let parent_id = request.parent_node_id.unwrap_or(state.root_node_id);
            let version = 1;
            let formula_text = request.formula_text;
            let node = StructuralNode {
                node_id,
                kind: node_kind_for_formula_text(&formula_text),
                symbol: request.symbol,
                parent_id: Some(parent_id),
                child_ids: Vec::new(),
            };
            let outcome = state.snapshot.apply_edit(
                snapshot_id,
                StructuralEdit::InsertNode {
                    node,
                    parent_id,
                    index: None,
                },
            )?;
            state.snapshot = outcome.snapshot;
            state.formula_text_versions.insert(node_id, version);
            let input_epoch =
                if let Some(input_value) = constant_value_for_formula_text(&formula_text) {
                    bump_input_value_epoch(state, node_id);
                    let input_epoch = state
                        .input_value_epochs
                        .get(&node_id)
                        .copied()
                        .unwrap_or(state.value_epoch);
                    state.input_values.insert(node_id, input_value);
                    input_epoch
                } else {
                    version
                };
            let input_record =
                direct_context_node_input_record(node_id, &formula_text, input_epoch);
            state.formula_texts.insert(node_id, formula_text);
            replace_node_input_record(state, input_record);
            if request.is_meta {
                state.meta_node_ids.insert(node_id);
            }
            state.pending_invalidation_seeds.clear();
            clear_pending_edit_transition_facts(state);
            state.seeded_published_values.clear();
            state.seeded_published_runtime_effects.clear();
            state.last_result = None;
        }
        self.advance_node_id();
        self.advance_snapshot_id();
        Ok(node_id)
    }

    pub fn set_node_formula_text(
        &mut self,
        workspace_id: &OxCalcTreeWorkspaceId,
        node_id: TreeNodeId,
        formula_text: impl Into<String>,
    ) -> Result<(), OxCalcTreeContextError> {
        let formula_text = formula_text.into();
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
            let predecessor_is_formula = predecessor_input_kind == NodeInputKind::FormulaText;
            let successor_is_formula = successor_input_kind == NodeInputKind::FormulaText;

            if !predecessor_is_formula && !successor_is_formula {
                let input_record = successor_non_formula_input_record(state, node_id, formula_text);
                replace_non_formula_node_input_record(state, input_record);
                state.seeded_published_values.remove(&node_id);
                push_pending_invalidation_seed(
                    state,
                    node_id,
                    InvalidationReasonKind::UpstreamPublication,
                );
                state.last_result = None;
                return Ok(());
            }

            if predecessor_is_formula || successor_is_formula {
                let predecessor_build = build_context_formula_catalog(state)?;
                let predecessor_unresolved =
                    context_formula_catalog_has_unresolved(node_id, &predecessor_build.diagnostics);
                let predecessor_catalog = predecessor_build.catalog;
                let predecessor_literal_value = (!predecessor_is_formula)
                    .then(|| current_literal_value_for_node(state, node_id))
                    .flatten();
                let version = state
                    .formula_text_versions
                    .get(&node_id)
                    .copied()
                    .unwrap_or_default()
                    + 1;
                state.formula_text_versions.insert(node_id, version);
                let successor_input_record = if successor_input_kind == NodeInputKind::FormulaText {
                    if let Some(value) = predecessor_literal_value {
                        state.seeded_published_values.insert(node_id, value);
                    }
                    NodeInputRecord::formula_text(node_id, formula_text.clone(), version)
                } else {
                    state.seeded_published_values.remove(&node_id);
                    successor_non_formula_input_record(state, node_id, formula_text.clone())
                };
                if successor_input_record.kind == NodeInputKind::FormulaText {
                    replace_formula_text_node_input_record(state, successor_input_record);
                } else {
                    replace_non_formula_node_input_record(state, successor_input_record);
                }
                let successor_build = build_context_formula_catalog(state)?;
                let successor_unresolved =
                    context_formula_catalog_has_unresolved(node_id, &successor_build.diagnostics);
                let successor_catalog = successor_build.catalog;
                let classification = classify_context_formula_edit(
                    &state.snapshot,
                    node_id,
                    &predecessor_catalog,
                    &successor_catalog,
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
        if is_formula_text(&input_value) {
            return Err(OxCalcTreeContextError::InputValueIsFormula { node_id });
        }
        self.set_node_non_formula_input_value(workspace_id, node_id, input_value)
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
            state.seeded_published_values.remove(&node_id);
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
            state.snapshot = outcome.snapshot;
            refresh_workspace_revision_and_absent_layers(state);
            state.pending_invalidation_seeds.clear();
            clear_pending_edit_transition_facts(state);
            state.seeded_published_values.clear();
            state.seeded_published_runtime_effects.clear();
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
            state.snapshot = outcome.snapshot;
            refresh_workspace_revision_and_absent_layers(state);
            state.pending_invalidation_seeds.clear();
            clear_pending_edit_transition_facts(state);
            state.seeded_published_values.clear();
            state.seeded_published_runtime_effects.clear();
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
                state.formula_texts.remove(removed_node_id);
                state.formula_text_versions.remove(removed_node_id);
                state.input_values.remove(removed_node_id);
                state.input_value_epochs.remove(removed_node_id);
                remove_node_input_record(state, *removed_node_id);
                state.meta_node_ids.remove(removed_node_id);
                state.pending_invalidation_seeds.clear();
                if let Some(snapshot) = state.table_snapshots.remove(removed_node_id) {
                    let deleted = deleted_table_fact_from_snapshot(state, &snapshot);
                    state.deleted_table_facts.push(deleted);
                    state.table_state_version += 1;
                }
            }
            remove_deleted_publication_and_runtime_facts(state, &outcome.affected_node_ids);
            state.snapshot = outcome.snapshot;
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
            state.snapshot = outcome.snapshot;
            state.table_snapshots.insert(node_id, normalized);
            state.table_state_version = next_table_state_version;
            refresh_workspace_revision_and_absent_layers(state);
            state.pending_invalidation_seeds.clear();
            clear_pending_edit_transition_facts(state);
            state.seeded_published_values.clear();
            state.seeded_published_runtime_effects.clear();
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
                .table_snapshots
                .remove(&node_id)
                .expect("table presence was checked before structural clear");
            let outcome = state
                .snapshot
                .apply_edit(snapshot_id, StructuralEdit::ClearTableShape { node_id })?;
            state.snapshot = outcome.snapshot;
            let deleted = deleted_table_fact_from_snapshot(state, &snapshot);
            state.deleted_table_facts.push(deleted);
            state.table_state_version += 1;
            refresh_workspace_revision_and_absent_layers(state);
            state.pending_invalidation_seeds.clear();
            clear_pending_edit_transition_facts(state);
            state.seeded_published_values.clear();
            state.seeded_published_runtime_effects.clear();
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
            workspace_id: state.workspace_id.clone(),
            root_node_id: state.root_node_id,
            structural_snapshot: state.snapshot.clone(),
            formula_texts: state.formula_texts.clone(),
            formula_text_versions: state.formula_text_versions.clone(),
            input_values: state.input_values.clone(),
            input_value_epochs: state.input_value_epochs.clone(),
            value_epoch: state.value_epoch,
            meta_node_ids: state.meta_node_ids.clone(),
            table_snapshots: state.table_snapshots.clone(),
            deleted_table_facts: state.deleted_table_facts.clone(),
            table_state_version: state.table_state_version,
            seeded_published_values: state.seeded_published_values.clone(),
            seeded_published_runtime_effects: state.seeded_published_runtime_effects.clone(),
        })
    }

    pub fn workspace_revision(
        &self,
        workspace_id: &OxCalcTreeWorkspaceId,
    ) -> Result<WorkspaceRevision, OxCalcTreeContextError> {
        Ok(self.workspace(workspace_id)?.workspace_revision.clone())
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
        let node_input_snapshot =
            node_input_snapshot_from_legacy_maps(&snapshot.structural_snapshot, &snapshot)?;
        let structural_snapshot = snapshot.structural_snapshot;
        let namespace_snapshot = namespace_snapshot_for_context(&self.options, &workspace_id);
        let workspace_revision = WorkspaceRevision::new(
            workspace_id.as_str(),
            structural_snapshot.clone(),
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
        let state = OxCalcTreeWorkspaceState {
            workspace_id: workspace_id.clone(),
            root_node_id: snapshot.root_node_id,
            snapshot: structural_snapshot,
            workspace_revision,
            formula_binding_snapshot,
            dependency_shape_snapshot,
            publication_snapshot,
            runtime_overlay_set,
            formula_texts: snapshot.formula_texts,
            formula_text_versions: snapshot.formula_text_versions,
            input_values: snapshot.input_values,
            input_value_epochs: snapshot.input_value_epochs,
            value_epoch: snapshot.value_epoch,
            meta_node_ids: snapshot.meta_node_ids,
            table_snapshots: snapshot.table_snapshots,
            deleted_table_facts: snapshot.deleted_table_facts,
            table_state_version: snapshot.table_state_version,
            seeded_published_values: snapshot.seeded_published_values,
            seeded_published_runtime_effects: snapshot.seeded_published_runtime_effects,
            pending_invalidation_seeds: Vec::new(),
            pending_formula_edit_diagnostics: Vec::new(),
            pending_node_input_kind_transitions: Vec::new(),
            pending_dependency_shape_updates: Vec::new(),
            last_result: None,
        };
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
        let catalog_build = build_context_formula_catalog(state)?;
        let pending_formula_edit_diagnostics = state.pending_formula_edit_diagnostics.clone();
        let pending_node_input_kind_transitions = state.pending_node_input_kind_transitions.clone();
        let pending_dependency_shape_updates = state.pending_dependency_shape_updates.clone();
        let revision_identity_basis = workspace_revision_identity_basis(state);
        let environment_context = runtime_context_for_workspace_state(&self.options, state);
        let artifacts = LocalTreeCalcEngine.execute(LocalTreeCalcInput {
            structural_snapshot: state.snapshot.clone(),
            formula_catalog: catalog_build.catalog,
            input_values: state.input_values.clone(),
            static_dependency_shape_updates: pending_dependency_shape_updates,
            seeded_published_values: state.seeded_published_values.clone(),
            seeded_published_runtime_effects: state.seeded_published_runtime_effects.clone(),
            invalidation_seeds: state.pending_invalidation_seeds.clone(),
            previous_arg_preparation_profile_version: None,
            candidate_result_id: format!("candidate:{}:{}", workspace_id.as_str(), candidate_index),
            publication_id: format!("publication:{}:{}", workspace_id.as_str(), candidate_index),
            compatibility_basis: revision_identity_basis.clone(),
            artifact_token_basis: revision_identity_basis,
            environment_context,
        })?;
        let mut result = OxCalcTreeCalculationOutcome::from(artifacts);
        result.diagnostics.extend(self.options.diagnostics());
        result.diagnostics.extend(catalog_build.diagnostics);
        result.diagnostics.extend(pending_formula_edit_diagnostics);
        result.diagnostics.extend(
            pending_node_input_kind_transitions
                .iter()
                .map(ContextNodeInputKindTransition::diagnostic),
        );
        let state = self.workspace_mut(workspace_id)?;
        state.seeded_published_values = result.published_values.clone();
        state.seeded_published_runtime_effects = result.runtime_effects.clone();
        state.pending_invalidation_seeds.clear();
        state.pending_formula_edit_diagnostics.clear();
        state.pending_node_input_kind_transitions.clear();
        state.pending_dependency_shape_updates.clear();
        state.last_result = Some(result.clone());
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
    if snapshot.structural_snapshot.root_node_id() != snapshot.root_node_id {
        return Err(OxCalcTreeContextError::InvalidWorkspaceSnapshot {
            detail: format!(
                "root_node_id {} does not match structural snapshot root {}",
                snapshot.root_node_id,
                snapshot.structural_snapshot.root_node_id()
            ),
        });
    }
    if snapshot
        .structural_snapshot
        .try_get_node(snapshot.root_node_id)
        .is_none()
    {
        return Err(OxCalcTreeContextError::InvalidWorkspaceSnapshot {
            detail: format!("root node {} is missing", snapshot.root_node_id),
        });
    }

    let structural_node_ids = snapshot
        .structural_snapshot
        .nodes()
        .keys()
        .copied()
        .collect::<std::collections::BTreeSet<_>>();
    let formula_text_node_ids = snapshot
        .formula_texts
        .keys()
        .copied()
        .collect::<std::collections::BTreeSet<_>>();
    let formula_version_node_ids = snapshot
        .formula_text_versions
        .keys()
        .copied()
        .collect::<std::collections::BTreeSet<_>>();
    if formula_text_node_ids != structural_node_ids {
        return Err(OxCalcTreeContextError::InvalidWorkspaceSnapshot {
            detail: format!(
                "formula_texts keys {:?} do not match structural node ids {:?}",
                formula_text_node_ids, structural_node_ids
            ),
        });
    }
    if formula_version_node_ids != structural_node_ids {
        return Err(OxCalcTreeContextError::InvalidWorkspaceSnapshot {
            detail: format!(
                "formula_text_versions keys {:?} do not match structural node ids {:?}",
                formula_version_node_ids, structural_node_ids
            ),
        });
    }

    for node_id in snapshot
        .formula_texts
        .keys()
        .chain(snapshot.formula_text_versions.keys())
        .chain(snapshot.input_values.keys())
        .chain(snapshot.input_value_epochs.keys())
        .chain(snapshot.meta_node_ids.iter())
        .chain(snapshot.table_snapshots.keys())
        .chain(snapshot.seeded_published_values.keys())
    {
        if snapshot
            .structural_snapshot
            .try_get_node(*node_id)
            .is_none()
        {
            return Err(OxCalcTreeContextError::InvalidWorkspaceSnapshot {
                detail: format!("node-scoped snapshot data references unknown node {node_id}"),
            });
        }
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

fn node_input_snapshot_from_legacy_maps(
    structural_snapshot: &StructuralSnapshot,
    snapshot: &OxCalcTreeWorkspaceSnapshot,
) -> Result<NodeInputSnapshot, OxCalcTreeContextError> {
    NodeInputSnapshot::create(structural_snapshot.nodes().keys().map(|node_id| {
        let formula_text = snapshot
            .formula_texts
            .get(node_id)
            .map_or("", String::as_str);
        let input_epoch = if !formula_text.is_empty() && !is_formula_text(formula_text) {
            snapshot
                .input_value_epochs
                .get(node_id)
                .copied()
                .unwrap_or(snapshot.value_epoch)
        } else {
            snapshot
                .formula_text_versions
                .get(node_id)
                .copied()
                .unwrap_or(1)
        };
        direct_context_node_input_record(*node_id, formula_text, input_epoch)
    }))
    .map_err(Into::into)
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
    runtime_context.meta_node_ids = state.meta_node_ids.clone();
    runtime_context
}

fn workspace_revision_identity_basis(state: &OxCalcTreeWorkspaceState) -> String {
    format!(
        "workspace-revision-basis:v1:workspace_revision_id={};structure_snapshot_id={};node_input_snapshot_id={};namespace_snapshot_id={}",
        state.workspace_revision.revision_id().0,
        state.snapshot.snapshot_id().0,
        state.workspace_revision.node_input_snapshot.snapshot_id().0,
        state.workspace_revision.namespace_snapshot.snapshot_id().0
    )
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
    let input_epoch = bump_input_value_epoch(state, node_id);
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
    let node_id = record.node_id;
    match record.kind {
        NodeInputKind::Empty => {
            state.formula_texts.insert(node_id, String::new());
            state.input_values.remove(&node_id);
            state.input_value_epochs.remove(&node_id);
        }
        NodeInputKind::Literal => {
            let literal_text = record.text.clone().unwrap_or_default();
            state.formula_texts.insert(node_id, literal_text.clone());
            state.input_values.insert(node_id, literal_text);
            state.input_value_epochs.insert(node_id, record.input_epoch);
        }
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
    let node_id = record.node_id;
    match record.kind {
        NodeInputKind::FormulaText => {
            let formula_text = record.text.clone().unwrap_or_default();
            state.formula_texts.insert(node_id, formula_text);
            state.input_values.remove(&node_id);
            state.input_value_epochs.remove(&node_id);
        }
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
    state.workspace_revision = WorkspaceRevision::new(
        state.workspace_id.as_str(),
        state.snapshot.clone(),
        node_input_snapshot,
        namespace_snapshot,
    );
    refresh_absent_snapshot_layer_shells(state);
}

fn replace_namespace_snapshot(
    state: &mut OxCalcTreeWorkspaceState,
    namespace_snapshot: NamespaceSnapshot,
) {
    let node_input_snapshot = state.workspace_revision.node_input_snapshot.clone();
    state.workspace_revision = WorkspaceRevision::new(
        state.workspace_id.as_str(),
        state.snapshot.clone(),
        node_input_snapshot,
        namespace_snapshot,
    );
    refresh_absent_snapshot_layer_shells(state);
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
    state.workspace_revision = WorkspaceRevision::new(
        state.workspace_id.as_str(),
        state.snapshot.clone(),
        node_input_snapshot,
        namespace_snapshot,
    );
    refresh_absent_snapshot_layer_shells(state);
}

fn refresh_absent_snapshot_layer_shells(state: &mut OxCalcTreeWorkspaceState) {
    state.formula_binding_snapshot = FormulaBindingSnapshot::current_absent(
        state.workspace_revision.revision_id(),
        "w057.2-formula-binding-not-yet-promoted",
    );
    state.dependency_shape_snapshot = DependencyShapeSnapshot::current_absent(
        state.workspace_revision.revision_id(),
        state.formula_binding_snapshot.snapshot_id(),
        "w057.2-dependency-shape-not-yet-promoted",
    );
    state.publication_snapshot = PublicationSnapshot::current_absent(
        state.workspace_revision.revision_id(),
        "w057.2-publication-not-yet-promoted",
    );
    state.runtime_overlay_set = RuntimeOverlaySet::current_absent(
        state.publication_snapshot.snapshot_id(),
        "w057.2-runtime-overlays-not-yet-promoted",
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
    for removed_node_id in &removed_node_ids {
        state.seeded_published_values.remove(removed_node_id);
    }
    state
        .seeded_published_runtime_effects
        .retain(|effect| !runtime_effect_mentions_any_node(effect, &removed_node_ids));

    // Direct-context structural delete does not yet classify compatible retained publication.
    // Drop the remaining baseline after the node-scoped facts have been explicitly removed.
    state.seeded_published_values.clear();
    state.seeded_published_runtime_effects.clear();
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

fn is_formula_text(formula_text: &str) -> bool {
    formula_text.trim_start().starts_with('=')
}

fn bump_input_value_epoch(state: &mut OxCalcTreeWorkspaceState, node_id: TreeNodeId) -> u64 {
    state.value_epoch = state.value_epoch.saturating_add(1);
    state.input_value_epochs.insert(node_id, state.value_epoch);
    state.value_epoch
}

fn current_literal_value_for_node(
    state: &OxCalcTreeWorkspaceState,
    node_id: TreeNodeId,
) -> Option<String> {
    state
        .input_values
        .get(&node_id)
        .cloned()
        .or_else(|| {
            state
                .last_result
                .as_ref()
                .and_then(|result| result.published_values.get(&node_id).cloned())
        })
        .or_else(|| state.seeded_published_values.get(&node_id).cloned())
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
    diagnostics: Vec<String>,
}

fn build_context_formula_catalog(
    state: &OxCalcTreeWorkspaceState,
) -> Result<ContextFormulaCatalogBuild, OxCalcTreeContextError> {
    let mut bindings = Vec::new();
    let mut diagnostics = Vec::new();

    for (owner_node_id, formula_text) in &state.formula_texts {
        if !is_formula_text(formula_text) {
            continue;
        }
        let version = state
            .formula_text_versions
            .get(owner_node_id)
            .copied()
            .unwrap_or(1);
        let mut host_packet_build =
            context_formula_from_oxfml_host_reference_packets(state, *owner_node_id, formula_text);
        diagnostics.extend(
            host_packet_build.diagnostics.drain(..).map(|diagnostic| {
                format!("treecalc_context_host_reference_resolution:{diagnostic}")
            }),
        );
        let resolution = direct_name_carriers_from_oxfml_probe(
            state.workspace_id.as_str(),
            host_packet_build.expression.source_text(),
            *owner_node_id,
            &state.snapshot,
            &state.meta_node_ids,
            &host_packet_build.source_spans,
        )?;
        let mut expression = host_packet_build.expression;
        expression
            .reference_carriers
            .extend(resolution.carriers.into_iter());
        diagnostics.extend(
            resolution
                .diagnostics
                .into_iter()
                .map(|diagnostic| format!("treecalc_context_host_name_resolution:{diagnostic}")),
        );
        bindings.push(TreeFormulaBinding {
            owner_node_id: *owner_node_id,
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

    Ok(ContextFormulaCatalogBuild {
        catalog: TreeFormulaCatalog::new(bindings),
        diagnostics,
    })
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
    predecessor_catalog: &TreeFormulaCatalog,
    successor_catalog: &TreeFormulaCatalog,
    transition: ContextFormulaEditTransition,
) -> ContextFormulaEditClassification {
    let predecessor_descriptors = predecessor_catalog.to_dependency_descriptors(snapshot);
    let successor_descriptors = successor_catalog.to_dependency_descriptors(snapshot);
    let predecessor_signatures =
        formula_dependency_signatures(owner_node_id, &predecessor_descriptors);
    let successor_signatures = formula_dependency_signatures(owner_node_id, &successor_descriptors);
    let affected_node_ids = formula_dependency_shape_affected_node_ids(
        owner_node_id,
        &predecessor_descriptors,
        &successor_descriptors,
    );

    let successor_graph = DependencyGraph::build(snapshot, &successor_descriptors);
    if successor_graph
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

    let predecessor_unresolved = transition.predecessor_unresolved
        || predecessor_signatures
            .iter()
            .any(|signature| signature.contains("kind=Unresolved"));
    let successor_unresolved = transition.successor_unresolved
        || successor_signatures
            .iter()
            .any(|signature| signature.contains("kind=Unresolved"));
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

    let predecessor_dynamic = predecessor_signatures
        .iter()
        .any(|signature| signature.contains("kind=DynamicPotential"));
    let successor_dynamic = successor_signatures
        .iter()
        .any(|signature| signature.contains("kind=DynamicPotential"));
    let dynamic_changed = (predecessor_dynamic || successor_dynamic)
        && predecessor_signatures != successor_signatures;
    if dynamic_changed {
        return ContextFormulaEditClassification {
            label: "dynamic_dependency_changed",
            affected_node_ids,
        };
    }

    if predecessor_signatures == successor_signatures {
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
                || diagnostic.contains("unresolved_reference"))
    })
}

fn formula_dependency_signatures(
    owner_node_id: TreeNodeId,
    descriptors: &[crate::dependency::DependencyDescriptor],
) -> BTreeSet<String> {
    descriptors
        .iter()
        .filter(|descriptor| descriptor.owner_node_id == owner_node_id)
        .map(|descriptor| {
            format!(
                "kind={:?};target={:?};workspace={:?};rebind={};detail={}",
                descriptor.kind,
                descriptor.target_node_id,
                descriptor.workspace_target,
                descriptor.requires_rebind_on_structural_change,
                descriptor.carrier_detail
            )
        })
        .collect()
}

fn formula_dependency_shape_affected_node_ids(
    owner_node_id: TreeNodeId,
    predecessor_descriptors: &[crate::dependency::DependencyDescriptor],
    successor_descriptors: &[crate::dependency::DependencyDescriptor],
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
    source_spans: Vec<(usize, usize)>,
    diagnostics: Vec<String>,
}

fn context_formula_from_oxfml_host_reference_packets(
    state: &OxCalcTreeWorkspaceState,
    owner_node_id: TreeNodeId,
    formula_text: &str,
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
    let host_context = RuntimeHostFormulaContext {
        dialect_id: "oxcalc.treecalc-v1".to_string(),
        capability_profile_id: "host-capabilities:treecalc-v1".to_string(),
        resolution_rule_version: "treecalc-host-resolution:v1".to_string(),
        host_namespace_version: None,
        registry_snapshot_identity: None,
        structure_context_version: None,
        caller_context_identity: Some(format!("treecalc-caller:{owner_node_id}")),
        table_context_identity: None,
        host_reference_syntax_rules: treecalc_host_reference_syntax_rules(),
    };
    let mut matches = host_context.declared_host_reference_syntax_matches(&source);
    matches.sort_by_key(|syntax_match| syntax_match.source_span.start);
    let mut accepted_matches = Vec::new();
    let mut carriers = Vec::new();
    let mut host_value_bindings = Vec::new();
    let mut source_spans = Vec::new();
    let mut diagnostics = Vec::new();
    let mut accepted_end = 0;

    for syntax_match in matches {
        let start = syntax_match.source_span.start;
        let end = start + syntax_match.source_span.len;
        if start < accepted_end {
            diagnostics.push(format!(
                "overlapping_host_reference_packet:{}:{}-{end}",
                syntax_match.source_token_text, start
            ));
            continue;
        }
        if host_reference_match_requires_unimplemented_tail(formula_text, &syntax_match) {
            diagnostics.push(format!(
                "typed_exclusion:tailed_host_reference_packet_pending:{}:{}-{end}",
                syntax_match.source_token_text, start
            ));
            continue;
        }
        let Some(resolution) = context_reference_resolution_from_oxfml_match(
            state,
            owner_node_id,
            &syntax_match,
            &mut diagnostics,
        ) else {
            continue;
        };
        match resolution {
            ContextHostReferenceResolution::Reference(mut carrier) => {
                carrier.source_token = Some(syntax_match.formal_token_text());
                carriers.push(carrier);
            }
            ContextHostReferenceResolution::Value(mut binding) => {
                binding.source_token = syntax_match.formal_token_text();
                host_value_bindings.push(binding);
            }
        }
        source_spans.push((start, end));
        accepted_end = end;
        accepted_matches.push(syntax_match);
    }

    let projection = host_context.project_host_reference_syntax_matches(&source, accepted_matches);
    diagnostics.extend(projection.diagnostics);
    ContextHostReferencePacketBuild {
        expression: TreeFormula::opaque_oxfml(projection.source.entered_formula_text, carriers)
            .with_host_value_bindings(host_value_bindings),
        source_spans,
        diagnostics,
    }
}

#[allow(clippy::large_enum_variant)]
enum ContextHostReferenceResolution {
    Reference(TreeFormulaReferenceCarrier),
    Value(TreeFormulaHostValueBinding),
}

fn context_reference_resolution_from_oxfml_match(
    state: &OxCalcTreeWorkspaceState,
    owner_node_id: TreeNodeId,
    syntax_match: &RuntimeHostReferenceSyntaxMatch,
    diagnostics: &mut Vec<String>,
) -> Option<ContextHostReferenceResolution> {
    let Some(payload) = syntax_match.opaque_selector_payload.as_deref() else {
        diagnostics.push(format!(
            "missing_host_reference_selector_payload:{}",
            syntax_match.source_token_text
        ));
        return None;
    };
    let snapshot = &state.snapshot;
    let meta_node_ids = &state.meta_node_ids;
    let selector = parse_context_host_reference_selector_payload(payload);
    match selector.selector_family.as_deref() {
        Some("children" | "children-sugar") => {
            if selector.base_token_text.is_none() {
                return treecalc_host_reference_carrier_from_syntax_match(
                    owner_node_id,
                    syntax_match,
                )
                .map_err(|error| {
                    diagnostics.push(format!(
                        "children_selector_resolution_error:{}:{}",
                        syntax_match.source_token_text, error
                    ));
                })
                .ok()
                .map(ContextHostReferenceResolution::Reference);
            }
            let base_node_id = resolve_context_host_reference_base_node_id(
                owner_node_id,
                syntax_match,
                snapshot,
                meta_node_ids,
                selector.base_token_text.as_deref(),
                diagnostics,
            )?;
            let start = syntax_match.source_span.start;
            let end = start + syntax_match.source_span.len;
            let collection = TreeCalcChildrenReferenceCollection::new(
                base_node_id,
                syntax_match.source_token_text.clone(),
            )
            .with_source_span_utf8(start, end);
            Some(ContextHostReferenceResolution::Reference(
                TreeFormulaReferenceCarrier::named(
                    syntax_match.syntax_match_handle.clone(),
                    TreeReference::ReferenceCollection(TreeCalcReferenceCollection::ChildrenV1(
                        collection,
                    )),
                ),
            ))
        }
        Some("preceding" | "following" | "ancestors" | "recursive-descent") => {
            let family = match selector.selector_family.as_deref() {
                Some("preceding") => TreeCalcOrderedSelectorFamily::PrecedingV1,
                Some("following") => TreeCalcOrderedSelectorFamily::FollowingV1,
                Some("ancestors") => TreeCalcOrderedSelectorFamily::AncestorsV1,
                Some("recursive-descent") => TreeCalcOrderedSelectorFamily::RecursiveDescendantsV1,
                _ => unreachable!("selector-family pattern is exhaustive"),
            };
            if selector.tail_token_text.is_some()
                && family != TreeCalcOrderedSelectorFamily::RecursiveDescendantsV1
            {
                diagnostics.push(format!(
                    "typed_exclusion:tailed_non_recursive_host_reference_packet_pending:{}:owner={owner_node_id}",
                    syntax_match.source_token_text
                ));
                return None;
            }
            let base_node_id = resolve_context_host_reference_base_node_id(
                owner_node_id,
                syntax_match,
                snapshot,
                meta_node_ids,
                selector.base_token_text.as_deref(),
                diagnostics,
            )?;
            let tail_segments = selector
                .tail_token_text
                .as_deref()
                .map(split_context_host_reference_tail_token)
                .unwrap_or_default();
            let traversal = resolve_treecalc_ordered_selector_traversal(
                snapshot,
                family,
                base_node_id,
                &tail_segments,
                TreeCalcOrderedSelectorTraversalPolicy::default(),
            )
            .map_err(|error| {
                diagnostics.push(format!(
                    "ordered_selector_resolution_error:{}:{}",
                    syntax_match.source_token_text, error
                ));
            })
            .ok()?;
            diagnostics.extend(traversal.diagnostics.iter().map(|diagnostic| {
                format!(
                    "ordered_selector_traversal:{:?}:{}",
                    diagnostic.code, diagnostic.detail
                )
            }));
            let start = syntax_match.source_span.start;
            let end = start + syntax_match.source_span.len;
            let collection = TreeCalcOrderedSelectorReferenceCollection::new(
                family,
                base_node_id,
                syntax_match.source_token_text.clone(),
                traversal.member_node_ids,
            )
            .with_source_span_utf8(start, end);
            Some(ContextHostReferenceResolution::Reference(
                TreeFormulaReferenceCarrier::named(
                    syntax_match.syntax_match_handle.clone(),
                    TreeReference::ReferenceCollection(
                        TreeCalcReferenceCollection::OrderedSelectorV1(collection),
                    ),
                ),
            ))
        }
        Some("sibling-prev" | "sibling-next") => {
            let offset = match selector.selector_family.as_deref() {
                Some("sibling-prev") => -1,
                Some("sibling-next") => 1,
                _ => unreachable!("selector-family pattern is exhaustive"),
            };
            let base_node_id = resolve_context_host_reference_base_node_id(
                owner_node_id,
                syntax_match,
                snapshot,
                meta_node_ids,
                selector.base_token_text.as_deref(),
                diagnostics,
            )?;
            let tail_segments = selector
                .tail_token_text
                .as_deref()
                .map(split_context_host_reference_tail_token)
                .unwrap_or_default();
            let reference = if selector.base_token_text.is_some() {
                TreeReference::QualifiedSiblingOffset {
                    base_node_id,
                    offset,
                    tail_segments,
                }
            } else {
                TreeReference::SiblingOffset {
                    offset,
                    tail_segments,
                }
            };
            Some(ContextHostReferenceResolution::Reference(
                TreeFormulaReferenceCarrier::named(
                    syntax_match.syntax_match_handle.clone(),
                    reference,
                ),
            ))
        }
        Some("reference-literal-array") => {
            let elements = resolve_context_reference_literal_array_elements(
                owner_node_id,
                syntax_match,
                snapshot,
                meta_node_ids,
                selector.element_token_texts.as_deref(),
                diagnostics,
            )?;
            let start = syntax_match.source_span.start;
            let end = start + syntax_match.source_span.len;
            let collection = TreeCalcReferenceLiteralArrayCollection::reference_only(
                syntax_match.syntax_match_handle.clone(),
                owner_node_id,
                syntax_match.source_token_text.clone(),
                elements,
            )
            .map_err(|error| {
                diagnostics.push(format!(
                    "typed_exclusion:reference_literal_collection_raw_context_pending:{}:{}",
                    syntax_match.source_token_text, error
                ));
            })
            .ok()?
            .with_source_span_utf8(start, end);
            Some(ContextHostReferenceResolution::Reference(
                TreeFormulaReferenceCarrier::named(
                    syntax_match.syntax_match_handle.clone(),
                    TreeReference::ReferenceCollection(
                        TreeCalcReferenceCollection::ReferenceLiteralArrayV1(collection),
                    ),
                ),
            ))
        }
        Some("ancestor-anchor") => {
            let repeat_count = selector.repeat_count.unwrap_or(1);
            if repeat_count == 0 {
                diagnostics.push(format!(
                    "invalid_ancestor_anchor_repeat_count:{}:owner={owner_node_id}",
                    syntax_match.source_token_text
                ));
                return None;
            }
            Some(ContextHostReferenceResolution::Reference(
                TreeFormulaReferenceCarrier::named(
                    syntax_match.syntax_match_handle.clone(),
                    TreeReference::RelativePath {
                        base: if repeat_count == 1 {
                            RelativeReferenceBase::ParentNode
                        } else {
                            RelativeReferenceBase::Ancestor(repeat_count)
                        },
                        path_segments: selector
                            .tail_token_text
                            .as_deref()
                            .map(split_context_host_reference_tail_token)
                            .unwrap_or_default(),
                    },
                ),
            ))
        }
        Some("parent-accessor") => {
            if selector.base_token_text.is_some() {
                diagnostics.push(format!(
                    "typed_exclusion:qualified_parent_accessor_host_reference_packet_pending:{}:owner={owner_node_id}",
                    syntax_match.source_token_text
                ));
                return None;
            }
            Some(ContextHostReferenceResolution::Reference(
                TreeFormulaReferenceCarrier::named(
                    syntax_match.syntax_match_handle.clone(),
                    TreeReference::RelativePath {
                        base: RelativeReferenceBase::ParentNode,
                        path_segments: selector
                            .tail_token_text
                            .as_deref()
                            .map(split_context_host_reference_tail_token)
                            .unwrap_or_default(),
                    },
                ),
            ))
        }
        Some("metadata-name" | "metadata-index" | "metadata-formula") => {
            if selector.tail_token_text.is_some() {
                diagnostics.push(format!(
                    "typed_exclusion:tailed_metadata_value_host_reference_packet_pending:{}:owner={owner_node_id}",
                    syntax_match.source_token_text
                ));
                return None;
            }
            let target_node_id = if selector.base_token_text.is_some() {
                resolve_context_host_reference_base_node_id(
                    owner_node_id,
                    syntax_match,
                    snapshot,
                    meta_node_ids,
                    selector.base_token_text.as_deref(),
                    diagnostics,
                )?
            } else {
                owner_node_id
            };
            let value = match selector.selector_family.as_deref() {
                Some("metadata-name") => snapshot
                    .try_get_node(target_node_id)
                    .map(|node| TreeFormulaHostValue::Text(node.symbol.clone()))
                    .unwrap_or(TreeFormulaHostValue::ValueError),
                Some("metadata-index") => {
                    context_metadata_index_value(snapshot, meta_node_ids, target_node_id)
                }
                Some("metadata-formula") => TreeFormulaHostValue::Text(
                    state
                        .formula_texts
                        .get(&target_node_id)
                        .cloned()
                        .unwrap_or_default(),
                ),
                _ => unreachable!("selector-family pattern is exhaustive"),
            };
            Some(ContextHostReferenceResolution::Value(
                context_host_value_binding(syntax_match, target_node_id, value),
            ))
        }
        Some("escaped-path") => {
            let path_token_text = selector
                .path_token_text
                .as_deref()
                .unwrap_or(syntax_match.source_token_text.as_str());
            match resolve_context_host_path_token(
                path_token_text,
                owner_node_id,
                snapshot,
                meta_node_ids,
            ) {
                ContextHostNameResolution::Resolved(target_node_id) => Some(
                    ContextHostReferenceResolution::Reference(TreeFormulaReferenceCarrier::named(
                        syntax_match.syntax_match_handle.clone(),
                        TreeReference::DirectNode { target_node_id },
                    )),
                ),
                ContextHostNameResolution::Ambiguous => {
                    diagnostics.push(format!(
                        "ambiguous_escaped_host_path:{}:{}:owner={owner_node_id}",
                        syntax_match.source_token_text, path_token_text
                    ));
                    None
                }
                ContextHostNameResolution::Unsupported(reason) => {
                    diagnostics.push(format!(
                        "typed_exclusion:{reason}:{}:{}:owner={owner_node_id}",
                        syntax_match.source_token_text, path_token_text
                    ));
                    None
                }
                ContextHostNameResolution::Unresolved => {
                    diagnostics.push(format!(
                        "unresolved_escaped_host_path:{}:{}:owner={owner_node_id}",
                        syntax_match.source_token_text, path_token_text
                    ));
                    None
                }
            }
        }
        Some(selector_family) => {
            diagnostics.push(format!(
                "unsupported_host_reference_selector_payload:{}:{}",
                syntax_match.source_token_text, selector_family
            ));
            None
        }
        None => {
            diagnostics.push(format!(
                "unsupported_host_reference_selector_payload:{}:{}",
                syntax_match.source_token_text, payload
            ));
            None
        }
    }
}

#[derive(Debug, Default)]
struct ContextHostReferenceSelectorPayload {
    selector_family: Option<String>,
    base_token_text: Option<String>,
    tail_token_text: Option<String>,
    path_token_text: Option<String>,
    element_token_texts: Option<String>,
    repeat_count: Option<usize>,
}

fn parse_context_host_reference_selector_payload(
    payload: &str,
) -> ContextHostReferenceSelectorPayload {
    let mut parsed = ContextHostReferenceSelectorPayload::default();
    for part in payload.split(';') {
        if let Some(selector_family) = part.strip_prefix("selector-family:") {
            parsed.selector_family = Some(selector_family.to_string());
        } else if let Some(base_token_text) = part.strip_prefix("base_token_text=") {
            parsed.base_token_text = Some(base_token_text.to_string());
        } else if let Some(tail_token_text) = part.strip_prefix("tail_token_text=") {
            parsed.tail_token_text = Some(tail_token_text.to_string());
        } else if let Some(path_token_text) = part.strip_prefix("path_token_text=") {
            parsed.path_token_text = Some(path_token_text.to_string());
        } else if let Some(element_token_texts) = part.strip_prefix("element_token_texts=") {
            parsed.element_token_texts = Some(element_token_texts.to_string());
        } else if let Some(repeat_count) = part.strip_prefix("repeat_count=")
            && let Ok(repeat_count) = repeat_count.parse::<usize>()
        {
            parsed.repeat_count = Some(repeat_count);
        }
    }
    parsed
}

fn context_host_value_binding(
    syntax_match: &RuntimeHostReferenceSyntaxMatch,
    target_node_id: TreeNodeId,
    value: TreeFormulaHostValue,
) -> TreeFormulaHostValueBinding {
    let start = syntax_match.source_span.start;
    let end = start + syntax_match.source_span.len;
    TreeFormulaHostValueBinding {
        source_token: syntax_match.formal_token_text(),
        value,
        host_ref_handle: syntax_match.syntax_match_handle.clone(),
        source_span_utf8: (start, end),
        source_token_text: syntax_match.source_token_text.clone(),
        opaque_selector: syntax_match.opaque_selector_payload.clone(),
        carrier_detail: format!(
            "treecalc_metadata_value:{}:target={}",
            syntax_match.source_token_text, target_node_id
        ),
        target_node_id: None,
        requires_rebind_on_structural_change: true,
    }
}

fn context_metadata_index_value(
    snapshot: &StructuralSnapshot,
    meta_node_ids: &BTreeSet<TreeNodeId>,
    target_node_id: TreeNodeId,
) -> TreeFormulaHostValue {
    let Some(parent_id) = snapshot.parent_id_of(target_node_id) else {
        return TreeFormulaHostValue::ValueError;
    };
    let Some(parent) = snapshot.try_get_node(parent_id) else {
        return TreeFormulaHostValue::ValueError;
    };
    parent
        .child_ids
        .iter()
        .copied()
        .filter(|child_id| !is_meta_effective(*child_id, snapshot, meta_node_ids))
        .position(|child_id| child_id == target_node_id)
        .and_then(|zero_based| i64::try_from(zero_based + 1).ok())
        .map(TreeFormulaHostValue::Integer)
        .unwrap_or(TreeFormulaHostValue::ValueError)
}

fn resolve_context_reference_literal_array_elements(
    owner_node_id: TreeNodeId,
    syntax_match: &RuntimeHostReferenceSyntaxMatch,
    snapshot: &StructuralSnapshot,
    meta_node_ids: &BTreeSet<TreeNodeId>,
    element_token_texts: Option<&str>,
    diagnostics: &mut Vec<String>,
) -> Option<Vec<TreeCalcReferenceLiteralArrayElement>> {
    let Some(element_token_texts) = element_token_texts else {
        diagnostics.push(format!(
            "missing_reference_literal_array_elements:{}:owner={owner_node_id}",
            syntax_match.source_token_text
        ));
        return None;
    };
    let mut elements = Vec::new();
    for element_token_text in element_token_texts.split('|') {
        match resolve_context_host_name_token(
            element_token_text,
            owner_node_id,
            snapshot,
            meta_node_ids,
        ) {
            ContextHostNameResolution::Resolved(target_node_id) => {
                elements.push(TreeCalcReferenceLiteralArrayElement::ReferenceNode(
                    target_node_id,
                ));
            }
            ContextHostNameResolution::Ambiguous => {
                diagnostics.push(format!(
                    "ambiguous_reference_literal_array_element:{}:{}:owner={owner_node_id}",
                    syntax_match.source_token_text, element_token_text
                ));
                return None;
            }
            ContextHostNameResolution::Unsupported(reason) => {
                diagnostics.push(format!(
                    "typed_exclusion:{reason}:{}:{}:owner={owner_node_id}",
                    syntax_match.source_token_text, element_token_text
                ));
                return None;
            }
            ContextHostNameResolution::Unresolved
                if context_reference_literal_element_is_scalar(element_token_text) =>
            {
                elements.push(TreeCalcReferenceLiteralArrayElement::ScalarValue {
                    source_text: element_token_text.to_string(),
                });
            }
            ContextHostNameResolution::Unresolved => {
                diagnostics.push(format!(
                    "unresolved_reference_literal_array_element:{}:{}:owner={owner_node_id}",
                    syntax_match.source_token_text, element_token_text
                ));
                return None;
            }
        }
    }
    Some(elements)
}

fn context_reference_literal_element_is_scalar(element_token_text: &str) -> bool {
    element_token_text.parse::<f64>().is_ok()
        || (element_token_text.starts_with('"') && element_token_text.ends_with('"'))
}

fn split_context_host_reference_tail_token(tail_token_text: &str) -> Vec<String> {
    tail_token_text
        .split('.')
        .filter(|segment| !segment.is_empty())
        .map(str::to_string)
        .collect()
}

fn resolve_context_host_reference_base_node_id(
    owner_node_id: TreeNodeId,
    syntax_match: &RuntimeHostReferenceSyntaxMatch,
    snapshot: &StructuralSnapshot,
    meta_node_ids: &BTreeSet<TreeNodeId>,
    base_token_text: Option<&str>,
    diagnostics: &mut Vec<String>,
) -> Option<TreeNodeId> {
    let Some(base_token_text) = base_token_text else {
        return Some(owner_node_id);
    };
    match resolve_context_host_name_token(base_token_text, owner_node_id, snapshot, meta_node_ids) {
        ContextHostNameResolution::Resolved(base_node_id) => Some(base_node_id),
        ContextHostNameResolution::Ambiguous => {
            diagnostics.push(format!(
                "ambiguous_host_reference_base:{}:{}:owner={owner_node_id}",
                syntax_match.source_token_text, base_token_text
            ));
            None
        }
        ContextHostNameResolution::Unsupported(reason) => {
            diagnostics.push(format!(
                "typed_exclusion:{reason}:{}:{}:owner={owner_node_id}",
                syntax_match.source_token_text, base_token_text
            ));
            None
        }
        ContextHostNameResolution::Unresolved => {
            diagnostics.push(format!(
                "unresolved_host_reference_base:{}:{}:owner={owner_node_id}",
                syntax_match.source_token_text, base_token_text
            ));
            None
        }
    }
}

fn host_reference_match_requires_unimplemented_tail(
    formula_text: &str,
    syntax_match: &RuntimeHostReferenceSyntaxMatch,
) -> bool {
    let start = syntax_match.source_span.start;
    let end = start + syntax_match.source_span.len;
    previous_non_whitespace_char(formula_text, start).is_some_and(|ch| ch == '.')
        || (syntax_match.source_token_text == "**"
            && next_non_whitespace_char(formula_text, end).is_some_and(|ch| ch == '.'))
}

fn previous_non_whitespace_char(text: &str, end: usize) -> Option<char> {
    text[..end].chars().rev().find(|ch| !ch.is_whitespace())
}

fn next_non_whitespace_char(text: &str, start: usize) -> Option<char> {
    text[start..].chars().find(|ch| !ch.is_whitespace())
}

struct ContextNameCarrierResolution {
    carriers: Vec<TreeFormulaReferenceCarrier>,
    diagnostics: Vec<String>,
}

struct ContextOxfmlBindProbe {
    unresolved_names: Vec<ContextSourceToken>,
    function_calls: Vec<ContextFunctionCall>,
    qualified_host_names: Vec<String>,
    reference_literal_syntax_seen: bool,
}

struct ContextSourceToken {
    source_text: String,
    source_span_utf8: (usize, usize),
}

struct ContextFunctionCall {
    function_name: String,
    source_span_utf8: (usize, usize),
}

fn direct_name_carriers_from_oxfml_probe(
    workspace_id: &str,
    formula_text: &str,
    owner_node_id: TreeNodeId,
    snapshot: &StructuralSnapshot,
    meta_node_ids: &BTreeSet<TreeNodeId>,
    host_reference_spans: &[(usize, usize)],
) -> Result<ContextNameCarrierResolution, OxCalcTreeContextError> {
    // OxFml owns formula syntax and lexical scope; OxCalc only resolves names that
    // OxFml exposes as unresolved host candidates.
    let probe = oxfml_context_bind_probe(workspace_id, owner_node_id, formula_text);
    let mut carriers = Vec::new();
    let mut diagnostics = typed_exclusion_diagnostics(
        &probe,
        snapshot,
        meta_node_ids,
        owner_node_id,
        host_reference_spans,
    );
    for candidate in probe.unresolved_names {
        if source_span_is_inside_any(candidate.source_span_utf8, host_reference_spans) {
            continue;
        }
        let token = candidate
            .source_text
            .strip_prefix("name:")
            .unwrap_or(candidate.source_text.as_str())
            .to_string();
        if token.starts_with("HOST_REF_") {
            continue;
        }
        let resolution =
            resolve_context_host_name_token(&token, owner_node_id, snapshot, meta_node_ids);
        let target_node_id = match resolution {
            ContextHostNameResolution::Resolved(target_node_id) => target_node_id,
            ContextHostNameResolution::Ambiguous => {
                diagnostics.push(format!("ambiguous_host_name:{token}:owner={owner_node_id}"));
                continue;
            }
            ContextHostNameResolution::Unsupported(reason) => {
                diagnostics.push(format!(
                    "typed_exclusion:{reason}:{token}:owner={owner_node_id}"
                ));
                continue;
            }
            ContextHostNameResolution::Unresolved => {
                diagnostics.push(format!(
                    "unresolved_host_name:{token}:owner={owner_node_id}"
                ));
                continue;
            }
        };
        carriers.push(
            TreeFormulaReferenceCarrier::named(
                token.clone(),
                TreeReference::DirectNode { target_node_id },
            )
            .with_host_name_bind(TreeFormulaHostNameBindPacket::direct_tree_node(
                workspace_id,
                owner_node_id,
                target_node_id,
                token,
                candidate.source_span_utf8,
                candidate.source_text,
            )),
        );
    }
    Ok(ContextNameCarrierResolution {
        carriers,
        diagnostics,
    })
}

fn source_span_is_inside_any(span: (usize, usize), spans: &[(usize, usize)]) -> bool {
    spans
        .iter()
        .any(|ignored| span.0 >= ignored.0 && span.1 <= ignored.1)
}

fn typed_exclusion_diagnostics(
    probe: &ContextOxfmlBindProbe,
    snapshot: &StructuralSnapshot,
    meta_node_ids: &BTreeSet<TreeNodeId>,
    owner_node_id: TreeNodeId,
    host_reference_spans: &[(usize, usize)],
) -> Vec<String> {
    let mut diagnostics = Vec::new();
    for call in &probe.function_calls {
        if matches!(
            resolve_context_host_name_token(
                &call.function_name,
                owner_node_id,
                snapshot,
                meta_node_ids,
            ),
            ContextHostNameResolution::Resolved(_) | ContextHostNameResolution::Ambiguous
        ) {
            diagnostics.push(format!(
                "typed_exclusion:node_as_function_w074_pending:{}:{}-{}:owner={owner_node_id}",
                call.function_name, call.source_span_utf8.0, call.source_span_utf8.1
            ));
        }
    }
    for qualified in &probe.qualified_host_names {
        diagnostics.push(format!(
            "typed_exclusion:cross_workspace_host_name_pending:{qualified}:owner={owner_node_id}"
        ));
    }
    if probe.reference_literal_syntax_seen
        && probe.unresolved_names.iter().any(|reference| {
            !source_span_is_inside_any(reference.source_span_utf8, host_reference_spans)
        })
    {
        diagnostics.push(format!(
            "typed_exclusion:reference_literal_collection_raw_context_pending:owner={owner_node_id}"
        ));
    }
    diagnostics
}

fn oxfml_context_bind_probe(
    workspace_id: &str,
    owner_node_id: TreeNodeId,
    formula_text: &str,
) -> ContextOxfmlBindProbe {
    let source = FormulaSourceRecord::new(
        format!("treecalc-context-probe:{workspace_id}:{}", owner_node_id.0),
        owner_node_id.0,
        formula_text,
    );
    let parse = parse_formula(ParseRequest {
        source: source.clone(),
    });
    let red_projection = project_red_view(source.formula_stable_id.clone(), &parse.green_tree);
    let bind = bind_formula(BindRequest {
        source: source.clone(),
        green_tree: parse.green_tree,
        red_projection,
        context: BindContext {
            formula_token: source.formula_token(),
            ..BindContext::default()
        },
    });
    let mut unresolved_spans_by_text = BTreeMap::<String, VecDeque<(usize, usize)>>::new();
    for diagnostic in &bind.bound_formula.diagnostics {
        if let Some(source_text) = diagnostic
            .message
            .strip_prefix("unresolved identifier '")
            .and_then(|rest| rest.strip_suffix('\''))
        {
            unresolved_spans_by_text
                .entry(source_text.to_string())
                .or_default()
                .push_back((diagnostic.span.start, diagnostic.span.end()));
        }
    }

    let unresolved_names = bind
        .bound_formula
        .unresolved_references
        .iter()
        .map(|reference| ContextSourceToken {
            source_text: reference.source_text.clone(),
            source_span_utf8: unresolved_spans_by_text
                .get_mut(&reference.source_text)
                .and_then(VecDeque::pop_front)
                .unwrap_or((0, reference.source_text.len())),
        })
        .collect::<Vec<_>>();
    let function_calls = bind
        .bound_formula
        .function_call_sources
        .iter()
        .map(|call| ContextFunctionCall {
            function_name: call.function_name.clone(),
            source_span_utf8: (call.callee_span.start, call.callee_span.end()),
        })
        .collect::<Vec<_>>();
    let qualified_host_names = bind
        .bound_formula
        .normalized_references
        .iter()
        .filter_map(|reference| match reference {
            oxfml_core::NormalizedReference::Name(name) if name.name.contains('!') => {
                Some(name.name.clone())
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    ContextOxfmlBindProbe {
        reference_literal_syntax_seen: formula_text.contains('{')
            && formula_text.contains('}')
            && unresolved_names
                .iter()
                .any(|reference| formula_text.contains(&reference.source_text)),
        unresolved_names,
        function_calls,
        qualified_host_names,
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
        display_path: canonical_path.clone(),
        canonical_path,
        formula_text: state
            .formula_texts
            .get(&node.node_id)
            .cloned()
            .unwrap_or_default(),
        value_text: state
            .last_result
            .as_ref()
            .and_then(|result| result.published_values.get(&node.node_id))
            .cloned()
            .or_else(|| state.seeded_published_values.get(&node.node_id).cloned())
            .or_else(|| state.input_values.get(&node.node_id).cloned()),
        input_value_epoch: state.input_value_epochs.get(&node.node_id).copied(),
        calc_state: state
            .last_result
            .as_ref()
            .and_then(|result| result.node_states.get(&node.node_id))
            .copied(),
        is_meta: state.meta_node_ids.contains(&node.node_id),
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
}

impl Default for OxCalcTreeContextOptions {
    fn default() -> Self {
        Self {
            runtime_lane: OxCalcTreeRuntimeLane::LocalSequentialTreeCalc,
            session_id: None,
            namespace: OxCalcTreeNamespaceOptions::default(),
            host_capabilities: OxCalcTreeHostCapabilitySnapshot::default(),
            runtime_policy: OxCalcTreeRuntimePolicy::default(),
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
            meta_node_ids: BTreeSet::new(),
            arg_preparation_profile_version: self.namespace.function_registry_version.clone(),
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
            published_values: value.published_values,
            node_states: value.node_states,
            phase_timings_micros: value.phase_timings_micros,
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
    };
    use crate::recalc::OverlayKind;
    use crate::structural::{
        BindArtifactId, FormulaArtifactId, StructuralNode, StructuralNodeKind, StructuralSnapshot,
        StructuralSnapshotId,
    };
    use crate::structured_table::{
        StructuredTableDependencyFactKind, StructuredTableRegionSelection,
        TreeCalcDynamicTableRebindCause, TreeCalcDynamicTableRebindStatus,
        TreeCalcDynamicTableReferenceTargetKind, TreeCalcTableColumnBodyMetadata,
        TreeCalcTableColumnSnapshot, TreeCalcTableRowId, TreeCalcTableVirtualAnchor,
    };
    use crate::workspace_revision::NodeInputKind;

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
            ],
        )
        .unwrap()
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
            header_row_present: true,
            totals_row_present: false,
            table_namespace_version: "host-supplied-namespace-should-not-win".to_string(),
            row_membership_version: "rows:v1".to_string(),
            row_order_version: "row-order:v1".to_string(),
            column_identity_version: "columns:v1".to_string(),
        }
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

        assert_eq!(result.run_state, OxCalcTreeRunState::Published);
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
    fn treecalc_context_namespace_mutation_advances_revision_and_prepared_basis() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new("workspace:w057-namespace"))
            .unwrap();
        let metadata_id = context
            .add_node(
                &workspace_id,
                OxCalcTreeNodeCreate::new("NameFormula", "=@NAME"),
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

        let result = context.recalculate(&workspace_id).unwrap();
        let candidate = result
            .candidate_result
            .as_ref()
            .expect("namespace-mutated recalc should publish candidate work");
        assert_eq!(
            result.published_values.get(&metadata_id),
            Some(&"NameFormula".to_string())
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
        assert!(
            result
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.starts_with("w056_prepared_identity_host_context:"))
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
        assert!(!exported.formula_texts.contains_key(&b2_id));
        assert!(!exported.formula_text_versions.contains_key(&b2_id));
        assert!(!exported.input_values.contains_key(&b2_id));
        assert!(!exported.input_value_epochs.contains_key(&b2_id));
        assert!(exported.seeded_published_values.is_empty());
        assert!(!exported.table_snapshots.contains_key(&b2_id));
        assert!(
            exported
                .deleted_table_facts
                .iter()
                .any(|fact| fact.table_id == "table:sales")
        );
        assert!(exported.seeded_published_runtime_effects.is_empty());
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
        let before_a_formula_version = before_snapshot.formula_text_versions[&a_id];

        context
            .set_node_formula_text(&workspace_id, a_id, "=4")
            .unwrap();
        let after_edit_before_recalc = context.workspace_view(&workspace_id).unwrap();
        let result = context.recalculate(&workspace_id).unwrap();
        let after_recalc = context.workspace_view(&workspace_id).unwrap();
        let after_snapshot = context.export_workspace_snapshot(&workspace_id).unwrap();

        assert_eq!(result.run_state, OxCalcTreeRunState::Published);
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
            after_snapshot.formula_text_versions[&a_id],
            before_a_formula_version + 1
        );
        assert_eq!(
            after_snapshot.structural_snapshot.snapshot_id(),
            before_snapshot.structural_snapshot.snapshot_id()
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
        let before_b_formula_version = before_snapshot.formula_text_versions[&b_id];

        context
            .set_node_formula_text(&workspace_id, b_id, "=C+1")
            .unwrap();
        let after_edit_before_recalc = context.workspace_view(&workspace_id).unwrap();
        let result = context.recalculate(&workspace_id).unwrap();
        let after_recalc = context.workspace_view(&workspace_id).unwrap();
        let after_snapshot = context.export_workspace_snapshot(&workspace_id).unwrap();

        assert_eq!(result.run_state, OxCalcTreeRunState::Published);
        assert_eq!(result.published_values.get(&b_id), Some(&"11".to_string()));
        assert_eq!(result.published_values.get(&d_id), Some(&"12".to_string()));
        assert_eq!(
            before_edit.snapshot_id,
            after_edit_before_recalc.snapshot_id
        );
        assert_eq!(before_edit.snapshot_id, after_recalc.snapshot_id);
        assert_eq!(before_edit.value_epoch, after_recalc.value_epoch);
        assert_eq!(
            after_snapshot.formula_text_versions[&b_id],
            before_b_formula_version + 1
        );
        assert_eq!(
            after_snapshot.structural_snapshot.snapshot_id(),
            before_snapshot.structural_snapshot.snapshot_id()
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
        assert!(a_id.0 > 0);
    }

    #[test]
    fn treecalc_context_formula_edit_unresolved_to_resolved_preserves_structure() {
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
        let initial = context.recalculate(&workspace_id).unwrap();
        assert_eq!(initial.run_state, OxCalcTreeRunState::Rejected);
        let before_edit = context.workspace_view(&workspace_id).unwrap();

        context
            .set_node_formula_text(&workspace_id, b_id, "=A+1")
            .unwrap();
        let after_edit_before_recalc = context.workspace_view(&workspace_id).unwrap();
        let result = context.recalculate(&workspace_id).unwrap();
        let after_recalc = context.workspace_view(&workspace_id).unwrap();

        assert_eq!(result.run_state, OxCalcTreeRunState::Published);
        assert_eq!(result.published_values.get(&b_id), Some(&"4".to_string()));
        assert_eq!(
            before_edit.snapshot_id,
            after_edit_before_recalc.snapshot_id
        );
        assert_eq!(before_edit.snapshot_id, after_recalc.snapshot_id);
        assert_eq!(before_edit.value_epoch, after_recalc.value_epoch);
        assert!(result.diagnostics.iter().any(|diagnostic| {
            diagnostic == &format!("formula_edit_classification:{b_id}:unresolved_to_resolved")
        }));
        assert!(
            result
                .publication_bundle
                .as_ref()
                .is_some_and(
                    |publication| publication
                        .dependency_shape_updates
                        .iter()
                        .any(|update| update.kind == "static_dependency_resolved"
                            && update.affected_node_ids.contains(&b_id)
                            && update.affected_node_ids.contains(&a_id))
                )
        );
        assert!(a_id.0 > 0);
    }

    #[test]
    fn treecalc_context_formula_edit_resolved_to_unresolved_rejects_without_structural_change() {
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
        let before_edit = context.workspace_view(&workspace_id).unwrap();

        context
            .set_node_formula_text(&workspace_id, b_id, "=Missing+1")
            .unwrap();
        let after_edit_before_recalc = context.workspace_view(&workspace_id).unwrap();
        let result = context.recalculate(&workspace_id).unwrap();
        let after_recalc = context.workspace_view(&workspace_id).unwrap();

        assert_eq!(result.run_state, OxCalcTreeRunState::Rejected);
        assert_eq!(
            before_edit.snapshot_id,
            after_edit_before_recalc.snapshot_id
        );
        assert_eq!(before_edit.snapshot_id, after_recalc.snapshot_id);
        assert_eq!(before_edit.value_epoch, after_recalc.value_epoch);
        assert_eq!(result.published_values.get(&b_id), Some(&"4".to_string()));
        assert!(result.diagnostics.iter().any(|diagnostic| {
            diagnostic == &format!("formula_edit_classification:{b_id}:resolved_to_unresolved")
        }));
        assert!(result.publication_bundle.is_none());
        assert!(a_id.0 > 0);
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
        let before_a_formula_version = before_snapshot.formula_text_versions[&a_id];

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
            after_snapshot.formula_text_versions[&a_id],
            before_a_formula_version + 1
        );
        assert!(!after_snapshot.input_values.contains_key(&a_id));
        assert_eq!(
            after_snapshot.structural_snapshot,
            before_snapshot.structural_snapshot
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
        let before_a_formula_version = before_snapshot.formula_text_versions[&a_id];

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
        assert_eq!(
            after_edit_a.input_value_epoch,
            Some(after_edit_before_recalc.value_epoch)
        );
        assert_eq!(after_recalc_a.value_text.as_deref(), Some("7"));
        assert_eq!(
            after_snapshot.formula_text_versions[&a_id],
            before_a_formula_version + 1
        );
        assert_eq!(
            after_snapshot.input_values.get(&a_id).map(String::as_str),
            Some("7")
        );
        assert_eq!(
            after_snapshot.structural_snapshot,
            before_snapshot.structural_snapshot
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
            exported.structural_snapshot.snapshot_id(),
            before_edit.snapshot_id
        );
        assert_eq!(
            exported.input_values.get(&a_id).map(String::as_str),
            Some("4")
        );
        assert_eq!(
            exported.structural_snapshot,
            before_snapshot.structural_snapshot
        );
        assert!(matches!(
            context
                .set_node_input_value(&workspace_id, b_id, "9")
                .unwrap_err(),
            OxCalcTreeContextError::InputValueOnFormulaNode { .. }
        ));
        assert!(matches!(
            context
                .set_node_input_value(&workspace_id, a_id, "=9")
                .unwrap_err(),
            OxCalcTreeContextError::InputValueIsFormula { .. }
        ));
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
        assert!(!exported.input_values.contains_key(&a_id));
        assert!(!exported.input_value_epochs.contains_key(&a_id));
        assert!(matches!(
            context
                .clear_node_input_value(&workspace_id, b_id)
                .unwrap_err(),
            OxCalcTreeContextError::InputValueOnFormulaNode { .. }
        ));
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
        assert_eq!(
            exported.input_values.get(&a_id).map(String::as_str),
            Some("3")
        );

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
        assert_eq!(
            reexported.input_values.get(&a_id).map(String::as_str),
            Some("4")
        );
        assert_eq!(reexported.structural_snapshot, exported.structural_snapshot);
    }

    #[test]
    fn treecalc_context_formula_artifacts_are_built_from_formula_text_layer() {
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
        exported.formula_texts.insert(b_id, "=A+2".to_string());
        *exported.formula_text_versions.get_mut(&b_id).unwrap() += 1;

        let mut imported_context = OxCalcTreeContext::default();
        let imported_workspace_id = imported_context
            .import_workspace_snapshot(exported)
            .unwrap();
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
        assert_eq!(snapshot.workspace_id, workspace_id);
        assert_eq!(snapshot.root_node_id, TreeNodeId(1));
        assert_eq!(
            snapshot.formula_texts.get(&a_id).map(String::as_str),
            Some("=3")
        );
        assert_eq!(
            snapshot.formula_texts.get(&b_id).map(String::as_str),
            Some("=A+1")
        );
        assert_eq!(
            snapshot
                .table_snapshots
                .get(&sales_id)
                .map(|table| table.table_id.as_str()),
            Some("table:sales")
        );
        assert_eq!(
            snapshot.seeded_published_values.get(&b_id),
            Some(&"4".to_string())
        );

        let serialized = serde_json::to_string_pretty(&snapshot).unwrap();
        let reparsed: OxCalcTreeWorkspaceSnapshot = serde_json::from_str(&serialized).unwrap();
        assert_eq!(reparsed.workspace_id, workspace_id);
        assert_eq!(reparsed.root_node_id, snapshot.root_node_id);
        assert_eq!(
            reparsed.formula_texts.get(&b_id).map(String::as_str),
            Some("=A+1")
        );
        assert_eq!(
            reparsed
                .table_snapshots
                .get(&sales_id)
                .map(|table| table.table_id.as_str()),
            Some("table:sales")
        );
        assert_eq!(
            reparsed.seeded_published_values.get(&b_id),
            Some(&"4".to_string())
        );

        let mut imported_context = OxCalcTreeContext::default();
        let imported_workspace_id = imported_context
            .import_workspace_snapshot(reparsed)
            .unwrap();
        assert_eq!(imported_workspace_id, workspace_id);

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
        snapshot.formula_texts.remove(&a_id);

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

        for expected in [
            "typed_exclusion:node_as_function_w074_pending",
            "typed_exclusion:reference_literal_collection_raw_context_pending",
            "typed_exclusion:cross_workspace_host_name_pending",
        ] {
            assert!(
                result
                    .diagnostics
                    .iter()
                    .any(|diagnostic| diagnostic.contains(expected)),
                "missing {expected} in {:?}",
                result.diagnostics
            );
        }
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
                diagnostic.contains("oxfml_bind_diagnostic")
                    || diagnostic.contains("unresolved_host_name")
            }),
            "out-of-range sibling navigation must wait for an OxFml packet, diagnostics={:?}",
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
                OxCalcTreeNodeCreate::new("Previous", "=@PREV").under(section_id),
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
                diagnostic.contains("unresolved_host_name:Secret")
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
        context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new("B", "=A+1"))
            .unwrap();

        let result = context.recalculate(&workspace_id).unwrap();

        assert!(result.diagnostics.iter().any(|diagnostic| {
            diagnostic.contains("treecalc_context_host_name_resolution:unresolved_host_name:A")
        }));
    }

    fn run_local_engine_fixture(
        options: OxCalcTreeContextOptions,
        formula_catalog: TreeFormulaCatalog,
        seeded_published_values: BTreeMap<TreeNodeId, String>,
        run_suffix: &str,
    ) -> OxCalcTreeCalculationOutcome {
        let artifacts = LocalTreeCalcEngine
            .execute(LocalTreeCalcInput {
                structural_snapshot: snapshot(),
                formula_catalog,
                input_values: BTreeMap::from([(TreeNodeId(2), "2".to_string())]),
                static_dependency_shape_updates: Vec::new(),
                seeded_published_values,
                seeded_published_runtime_effects: Vec::new(),
                invalidation_seeds: Vec::new(),
                previous_arg_preparation_profile_version: None,
                candidate_result_id: format!("candidate:{run_suffix}"),
                publication_id: format!("publication:{run_suffix}"),
                compatibility_basis: "snapshot:1".to_string(),
                artifact_token_basis: "snapshot:1".to_string(),
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
