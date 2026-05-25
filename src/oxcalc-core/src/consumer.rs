#![forbid(unsafe_code)]

//! Consumer-facing OxCalc runtime contract for tree-substrate hosts.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use oxfml_core::consumer::runtime::{RuntimeHostFormulaContext, RuntimeHostReferenceSyntaxMatch};
use oxfml_core::{
    BindContext, BindRequest, FormulaSourceRecord, ParseRequest, StructuredReferenceBindRecord,
    bind_formula, parse_formula, project_red_view,
};
use thiserror::Error;

use crate::coordinator::{AcceptedCandidateResult, PublicationBundle, RejectDetail, RuntimeEffect};
use crate::dependency::{DependencyGraph, InvalidationClosure};
use crate::formula::{
    RelativeReferenceBase, TreeCalcChildrenReferenceCollection, TreeCalcOrderedSelectorFamily,
    TreeCalcOrderedSelectorReferenceCollection, TreeCalcOrderedSelectorTraversalPolicy,
    TreeCalcReferenceCollection, TreeCalcReferenceLiteralArrayCollection,
    TreeCalcReferenceLiteralArrayElement, TreeFormula, TreeFormulaBinding, TreeFormulaCatalog,
    TreeFormulaHostNameBindPacket, TreeFormulaReferenceCarrier, TreeReference,
    resolve_treecalc_ordered_selector_traversal, split_treecalc_host_path_token,
    treecalc_host_reference_carrier_from_syntax_match,
};
use crate::recalc::{NodeCalcState, OverlayEntry};
use crate::structural::{
    BindArtifactId, FormulaArtifactId, StructuralEdit, StructuralError, StructuralNode,
    StructuralNodeKind, StructuralSnapshot, StructuralSnapshotId, TreeNodeId,
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
use crate::treecalc::{
    DerivationTraceRecord, LocalTreeCalcEngine, LocalTreeCalcEnvironmentContext,
    LocalTreeCalcError, LocalTreeCalcInput, LocalTreeCalcRunArtifacts, LocalTreeCalcRunState,
    LocalTreeCalcSchedulingPolicy, treecalc_host_reference_syntax_rules,
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
    pub nodes: Vec<OxCalcTreeNodeView>,
    pub tables: Vec<OxCalcTreeTableView>,
    pub diagnostics: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OxCalcTreeWorkspaceSnapshot {
    pub workspace_id: OxCalcTreeWorkspaceId,
    pub root_node_id: TreeNodeId,
    pub structural_snapshot: StructuralSnapshot,
    pub formula_texts: BTreeMap<TreeNodeId, String>,
    pub formula_text_versions: BTreeMap<TreeNodeId, u64>,
    pub meta_node_ids: BTreeSet<TreeNodeId>,
    pub table_snapshots: BTreeMap<TreeNodeId, TreeCalcTableNodeSnapshot>,
    pub deleted_table_facts: Vec<TreeCalcTableDeletedFact>,
    pub table_state_version: u64,
    pub seeded_published_values: BTreeMap<TreeNodeId, String>,
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
}

#[derive(Debug, Clone)]
struct OxCalcTreeWorkspaceState {
    workspace_id: OxCalcTreeWorkspaceId,
    root_node_id: TreeNodeId,
    snapshot: StructuralSnapshot,
    formula_texts: BTreeMap<TreeNodeId, String>,
    formula_text_versions: BTreeMap<TreeNodeId, u64>,
    meta_node_ids: BTreeSet<TreeNodeId>,
    table_snapshots: BTreeMap<TreeNodeId, TreeCalcTableNodeSnapshot>,
    deleted_table_facts: Vec<TreeCalcTableDeletedFact>,
    table_state_version: u64,
    seeded_published_values: BTreeMap<TreeNodeId, String>,
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
            formula_artifact_id: None,
            bind_artifact_id: None,
            constant_value: None,
        };
        let snapshot = StructuralSnapshot::create(snapshot_id, root_node_id, [root])?;
        let workspace_id = request.workspace_id;
        let state = OxCalcTreeWorkspaceState {
            workspace_id: workspace_id.clone(),
            root_node_id,
            snapshot,
            formula_texts: BTreeMap::from([(root_node_id, String::new())]),
            formula_text_versions: BTreeMap::from([(root_node_id, 1)]),
            meta_node_ids: BTreeSet::new(),
            table_snapshots: BTreeMap::new(),
            deleted_table_facts: Vec::new(),
            table_state_version: 1,
            seeded_published_values: BTreeMap::new(),
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
            let node = StructuralNode {
                node_id,
                kind: node_kind_for_formula_text(&request.formula_text),
                symbol: request.symbol,
                parent_id: Some(parent_id),
                child_ids: Vec::new(),
                formula_artifact_id: formula_artifact_id_for(
                    state.workspace_id.as_str(),
                    node_id,
                    version,
                    &request.formula_text,
                ),
                bind_artifact_id: bind_artifact_id_for(
                    state.workspace_id.as_str(),
                    node_id,
                    version,
                    &request.formula_text,
                ),
                constant_value: constant_value_for_formula_text(&request.formula_text),
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
            state.formula_texts.insert(node_id, request.formula_text);
            if request.is_meta {
                state.meta_node_ids.insert(node_id);
            }
            state.seeded_published_values.clear();
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
        let attachment_snapshot_id = self.next_snapshot_id();
        let value_snapshot_id = self.next_snapshot_id_after(1);
        {
            let state = self.workspace_mut(workspace_id)?;
            let version = state
                .formula_text_versions
                .get(&node_id)
                .copied()
                .unwrap_or_default()
                + 1;
            let attachment = state.snapshot.apply_edit(
                attachment_snapshot_id,
                StructuralEdit::ReplaceFormulaAttachment {
                    node_id,
                    formula_artifact_id: formula_artifact_id_for(
                        state.workspace_id.as_str(),
                        node_id,
                        version,
                        &formula_text,
                    ),
                    bind_artifact_id: bind_artifact_id_for(
                        state.workspace_id.as_str(),
                        node_id,
                        version,
                        &formula_text,
                    ),
                },
            )?;
            let value = attachment.snapshot.apply_edit(
                value_snapshot_id,
                StructuralEdit::SetConstantValue {
                    node_id,
                    constant_value: constant_value_for_formula_text(&formula_text),
                },
            )?;
            state.snapshot = value.snapshot;
            state.formula_text_versions.insert(node_id, version);
            state.formula_texts.insert(node_id, formula_text);
            state.seeded_published_values.clear();
            state.last_result = None;
        }
        self.advance_snapshot_id_by(2);
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
            state.seeded_published_values.clear();
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
            state.seeded_published_values.clear();
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
                state.meta_node_ids.remove(removed_node_id);
                state.seeded_published_values.remove(removed_node_id);
                if let Some(snapshot) = state.table_snapshots.remove(removed_node_id) {
                    let deleted = deleted_table_fact_from_snapshot(state, &snapshot);
                    state.deleted_table_facts.push(deleted);
                    state.table_state_version += 1;
                }
            }
            state.snapshot = outcome.snapshot;
            state.seeded_published_values.clear();
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
        {
            let state = self.workspace_mut(workspace_id)?;
            state
                .snapshot
                .try_get_node(node_id)
                .ok_or(StructuralError::UnknownNode { node_id })?;
            let next_table_state_version = state.table_state_version + 1;
            let normalized = normalize_context_table_snapshot_with_version(
                state,
                node_id,
                &snapshot,
                next_table_state_version,
            )?;
            project_treecalc_table_node_snapshot(&normalized)
                .map_err(|error| OxCalcTreeContextError::TableProjection { error })?;
            state.table_snapshots.insert(node_id, snapshot);
            state.table_state_version = next_table_state_version;
            state.seeded_published_values.clear();
            state.last_result = None;
        }
        self.table_view(workspace_id, node_id)?
            .ok_or(StructuralError::UnknownNode { node_id }.into())
    }

    pub fn clear_node_table(
        &mut self,
        workspace_id: &OxCalcTreeWorkspaceId,
        node_id: TreeNodeId,
    ) -> Result<Option<TreeCalcTableNodeSnapshot>, OxCalcTreeContextError> {
        let state = self.workspace_mut(workspace_id)?;
        let Some(snapshot) = state.table_snapshots.remove(&node_id) else {
            return Ok(None);
        };
        let deleted = deleted_table_fact_from_snapshot(state, &snapshot);
        state.deleted_table_facts.push(deleted);
        state.table_state_version += 1;
        state.seeded_published_values.clear();
        state.last_result = None;
        Ok(Some(snapshot))
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
            meta_node_ids: state.meta_node_ids.clone(),
            table_snapshots: state.table_snapshots.clone(),
            deleted_table_facts: state.deleted_table_facts.clone(),
            table_state_version: state.table_state_version,
            seeded_published_values: state.seeded_published_values.clone(),
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
        let structural_snapshot = snapshot.structural_snapshot;
        let state = OxCalcTreeWorkspaceState {
            workspace_id: workspace_id.clone(),
            root_node_id: snapshot.root_node_id,
            snapshot: structural_snapshot,
            formula_texts: snapshot.formula_texts,
            formula_text_versions: snapshot.formula_text_versions,
            meta_node_ids: snapshot.meta_node_ids,
            table_snapshots: snapshot.table_snapshots,
            deleted_table_facts: snapshot.deleted_table_facts,
            table_state_version: snapshot.table_state_version,
            seeded_published_values: snapshot.seeded_published_values,
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
        let snapshot_id = state.snapshot.snapshot_id();
        let artifacts = LocalTreeCalcEngine.execute(LocalTreeCalcInput {
            structural_snapshot: state.snapshot.clone(),
            formula_catalog: catalog_build.catalog,
            seeded_published_values: state.seeded_published_values.clone(),
            seeded_published_runtime_effects: Vec::new(),
            invalidation_seeds: Vec::new(),
            previous_arg_preparation_profile_version: None,
            candidate_result_id: format!("candidate:{}:{}", workspace_id.as_str(), candidate_index),
            publication_id: format!("publication:{}:{}", workspace_id.as_str(), candidate_index),
            compatibility_basis: format!("snapshot:{}", snapshot_id.0),
            artifact_token_basis: format!("snapshot:{}", snapshot_id.0),
            environment_context: self.options.runtime_context(),
        })?;
        let mut result = OxCalcTreeCalculationOutcome::from(artifacts);
        result.diagnostics.extend(self.options.diagnostics());
        result.diagnostics.extend(catalog_build.diagnostics);
        let state = self.workspace_mut(workspace_id)?;
        state.seeded_published_values = result.published_values.clone();
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
    let canonical_path = state.snapshot.get_projection_path(node_id)?;
    let mut normalized = snapshot.clone();
    normalized.table_node_id = node_id;
    normalized.display_path = canonical_path.clone();
    normalized.canonical_path = canonical_path;
    normalized.table_namespace_version =
        context_table_namespace_version(state, node_id, table_state_version);
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

fn context_table_namespace_version(
    state: &OxCalcTreeWorkspaceState,
    node_id: TreeNodeId,
    table_state_version: u64,
) -> String {
    format!(
        "treecalc-table-namespace:v1:{}:{}:snapshot={}:tables={}",
        state.workspace_id.as_str(),
        node_id.0,
        state.snapshot.snapshot_id().0,
        table_state_version
    )
}

fn formula_artifact_id_for(
    workspace_id: &str,
    node_id: TreeNodeId,
    formula_text_version: u64,
    formula_text: &str,
) -> Option<FormulaArtifactId> {
    is_formula_text(formula_text).then(|| {
        FormulaArtifactId(format!(
            "formula:{}:{}:v{}",
            workspace_id, node_id.0, formula_text_version
        ))
    })
}

fn bind_artifact_id_for(
    workspace_id: &str,
    node_id: TreeNodeId,
    formula_text_version: u64,
    formula_text: &str,
) -> Option<BindArtifactId> {
    is_formula_text(formula_text).then(|| {
        BindArtifactId(format!(
            "bind:{}:{}:v{}",
            workspace_id, node_id.0, formula_text_version
        ))
    })
}

fn constant_value_for_formula_text(formula_text: &str) -> Option<String> {
    (!is_formula_text(formula_text) && !formula_text.is_empty()).then(|| formula_text.to_string())
}

fn is_formula_text(formula_text: &str) -> bool {
    formula_text.trim_start().starts_with('=')
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
        let Some(mut carrier) = context_reference_carrier_from_oxfml_match(
            owner_node_id,
            &syntax_match,
            &state.snapshot,
            &state.meta_node_ids,
            &mut diagnostics,
        ) else {
            continue;
        };
        carrier.source_token = Some(syntax_match.formal_token_text());
        carriers.push(carrier);
        source_spans.push((start, end));
        accepted_end = end;
        accepted_matches.push(syntax_match);
    }

    let projection = host_context.project_host_reference_syntax_matches(&source, accepted_matches);
    diagnostics.extend(projection.diagnostics);
    ContextHostReferencePacketBuild {
        expression: TreeFormula::opaque_oxfml(projection.source.entered_formula_text, carriers),
        source_spans,
        diagnostics,
    }
}

fn context_reference_carrier_from_oxfml_match(
    owner_node_id: TreeNodeId,
    syntax_match: &RuntimeHostReferenceSyntaxMatch,
    snapshot: &StructuralSnapshot,
    meta_node_ids: &BTreeSet<TreeNodeId>,
    diagnostics: &mut Vec<String>,
) -> Option<TreeFormulaReferenceCarrier> {
    let Some(payload) = syntax_match.opaque_selector_payload.as_deref() else {
        diagnostics.push(format!(
            "missing_host_reference_selector_payload:{}",
            syntax_match.source_token_text
        ));
        return None;
    };
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
                .ok();
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
            Some(TreeFormulaReferenceCarrier::named(
                syntax_match.syntax_match_handle.clone(),
                TreeReference::ReferenceCollection(TreeCalcReferenceCollection::ChildrenV1(
                    collection,
                )),
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
            Some(TreeFormulaReferenceCarrier::named(
                syntax_match.syntax_match_handle.clone(),
                TreeReference::ReferenceCollection(TreeCalcReferenceCollection::OrderedSelectorV1(
                    collection,
                )),
            ))
        }
        Some("sibling-prev" | "sibling-next") => {
            if selector.base_token_text.is_some() {
                diagnostics.push(format!(
                    "typed_exclusion:qualified_sibling_offset_host_reference_packet_pending:{}:owner={owner_node_id}",
                    syntax_match.source_token_text
                ));
                return None;
            }
            let offset = match selector.selector_family.as_deref() {
                Some("sibling-prev") => -1,
                Some("sibling-next") => 1,
                _ => unreachable!("selector-family pattern is exhaustive"),
            };
            let tail_segments = selector
                .tail_token_text
                .as_deref()
                .map(split_context_host_reference_tail_token)
                .unwrap_or_default();
            if let Some(target_node_id) = resolve_visible_sibling_offset_target(
                snapshot,
                meta_node_ids,
                owner_node_id,
                offset,
                &tail_segments,
            ) {
                Some(TreeFormulaReferenceCarrier::named(
                    syntax_match.syntax_match_handle.clone(),
                    TreeReference::DirectNode { target_node_id },
                ))
            } else {
                Some(TreeFormulaReferenceCarrier::named(
                    syntax_match.syntax_match_handle.clone(),
                    TreeReference::SiblingOffset {
                        offset,
                        tail_segments,
                    },
                ))
            }
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
            Some(TreeFormulaReferenceCarrier::named(
                syntax_match.syntax_match_handle.clone(),
                TreeReference::ReferenceCollection(
                    TreeCalcReferenceCollection::ReferenceLiteralArrayV1(collection),
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
            Some(TreeFormulaReferenceCarrier::named(
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
                ContextHostNameResolution::Resolved(target_node_id) => {
                    Some(TreeFormulaReferenceCarrier::named(
                        syntax_match.syntax_match_handle.clone(),
                        TreeReference::DirectNode { target_node_id },
                    ))
                }
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

enum ContextHostNameResolution {
    Resolved(TreeNodeId),
    Ambiguous,
    Unsupported(&'static str),
    Unresolved,
}

fn resolve_context_host_path_token(
    token: &str,
    owner_node_id: TreeNodeId,
    snapshot: &StructuralSnapshot,
    meta_node_ids: &BTreeSet<TreeNodeId>,
) -> ContextHostNameResolution {
    if token.contains('!') {
        return ContextHostNameResolution::Unsupported("cross_workspace_host_path_pending");
    }
    let segments = match split_treecalc_host_path_token(token) {
        Ok(segments) => segments,
        Err(_) => {
            return ContextHostNameResolution::Unsupported("invalid_bracket_escaped_host_path");
        }
    };
    resolve_context_host_path_segments(&segments, owner_node_id, snapshot, meta_node_ids)
}

fn resolve_context_host_name_token(
    token: &str,
    owner_node_id: TreeNodeId,
    snapshot: &StructuralSnapshot,
    meta_node_ids: &BTreeSet<TreeNodeId>,
) -> ContextHostNameResolution {
    if token.contains('[') || token.contains(']') {
        return ContextHostNameResolution::Unsupported("bracket_escaped_host_path_pending");
    }
    if token.contains('!') {
        return ContextHostNameResolution::Unsupported("cross_workspace_host_path_pending");
    }
    let segments = token
        .split('.')
        .filter(|segment| !segment.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();
    resolve_context_host_path_segments(&segments, owner_node_id, snapshot, meta_node_ids)
}

fn resolve_context_host_path_segments(
    segments: &[String],
    owner_node_id: TreeNodeId,
    snapshot: &StructuralSnapshot,
    meta_node_ids: &BTreeSet<TreeNodeId>,
) -> ContextHostNameResolution {
    if segments.is_empty() {
        return ContextHostNameResolution::Unresolved;
    }

    if snapshot
        .try_get_node(snapshot.root_node_id())
        .is_some_and(|root| root.symbol.eq_ignore_ascii_case(&segments[0]))
    {
        return try_resolve_visible_descendant_path(
            snapshot,
            meta_node_ids,
            snapshot.root_node_id(),
            &segments[1..],
        )
        .map_or(
            ContextHostNameResolution::Unresolved,
            ContextHostNameResolution::Resolved,
        );
    }

    let base =
        match resolve_context_walkup_symbol(&segments[0], owner_node_id, snapshot, meta_node_ids) {
            ContextHostNameResolution::Resolved(base_node_id) => base_node_id,
            other => return other,
        };
    if segments.len() == 1 {
        return ContextHostNameResolution::Resolved(base);
    }
    try_resolve_visible_descendant_path(snapshot, meta_node_ids, base, &segments[1..]).map_or(
        ContextHostNameResolution::Unresolved,
        ContextHostNameResolution::Resolved,
    )
}

fn resolve_context_walkup_symbol(
    symbol: &str,
    owner_node_id: TreeNodeId,
    snapshot: &StructuralSnapshot,
    meta_node_ids: &BTreeSet<TreeNodeId>,
) -> ContextHostNameResolution {
    let mut scope = Some(owner_node_id);
    while let Some(scope_node_id) = scope {
        match resolve_child_symbol_in_scope(symbol, scope_node_id, snapshot, meta_node_ids) {
            ContextHostNameResolution::Unresolved => {
                scope = snapshot.parent_id_of(scope_node_id);
            }
            other => return other,
        }
    }
    ContextHostNameResolution::Unresolved
}

fn resolve_child_symbol_in_scope(
    symbol: &str,
    scope_node_id: TreeNodeId,
    snapshot: &StructuralSnapshot,
    meta_node_ids: &BTreeSet<TreeNodeId>,
) -> ContextHostNameResolution {
    let Some(scope_node) = snapshot.try_get_node(scope_node_id) else {
        return ContextHostNameResolution::Unresolved;
    };
    let matches = scope_node
        .child_ids
        .iter()
        .copied()
        .filter(|child_id| {
            snapshot
                .try_get_node(*child_id)
                .is_some_and(|child| child.symbol.eq_ignore_ascii_case(symbol))
                && !is_meta_effective(*child_id, snapshot, meta_node_ids)
        })
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [] => ContextHostNameResolution::Unresolved,
        [node_id] => ContextHostNameResolution::Resolved(*node_id),
        _ => ContextHostNameResolution::Ambiguous,
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

fn resolve_visible_sibling_offset_target(
    snapshot: &StructuralSnapshot,
    meta_node_ids: &BTreeSet<TreeNodeId>,
    owner_node_id: TreeNodeId,
    offset: isize,
    tail_segments: &[String],
) -> Option<TreeNodeId> {
    let parent_id = snapshot.parent_id_of(owner_node_id)?;
    let parent = snapshot.try_get_node(parent_id)?;
    let visible_siblings = parent
        .child_ids
        .iter()
        .copied()
        .filter(|child_id| !is_meta_effective(*child_id, snapshot, meta_node_ids))
        .collect::<Vec<_>>();
    let owner_index = visible_siblings
        .iter()
        .position(|child_id| *child_id == owner_node_id)?;
    let target_index = isize::try_from(owner_index)
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

fn typed_exclusion_diagnostics(
    probe: &ContextOxfmlBindProbe,
    snapshot: &StructuralSnapshot,
    meta_node_ids: &BTreeSet<TreeNodeId>,
    owner_node_id: TreeNodeId,
    host_reference_spans: &[(usize, usize)],
) -> Vec<String> {
    let mut diagnostics = Vec::new();
    for call in &probe.function_calls {
        if call.function_name.eq_ignore_ascii_case("INDIRECT") {
            diagnostics.push(format!(
                "typed_exclusion:dynamic_indirect_raw_context_pending:{}:{}-{}:owner={owner_node_id}",
                call.function_name, call.source_span_utf8.0, call.source_span_utf8.1
            ));
        } else if matches!(
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
            .or_else(|| node.constant_value.clone()),
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
    pub host_capabilities: OxCalcTreeHostCapabilitySnapshot,
    pub runtime_policy: OxCalcTreeRuntimePolicy,
}

impl Default for OxCalcTreeContextOptions {
    fn default() -> Self {
        Self {
            runtime_lane: OxCalcTreeRuntimeLane::LocalSequentialTreeCalc,
            session_id: None,
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
            host_namespace_version: "treecalc-host-namespace:v1".to_string(),
            resolution_rule_version: "treecalc-host-resolution:v1".to_string(),
            caller_context_identity_version: "treecalc-caller-context:v1".to_string(),
            table_context_identity: None,
            cross_workspace_availability_version: None,
            arg_preparation_profile_version: "oxfunc.arg-prep:default".to_string(),
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
                    kind: StructuralNodeKind::Constant,
                    symbol: "A".to_string(),
                    parent_id: Some(TreeNodeId(1)),
                    child_ids: vec![],
                    formula_artifact_id: None,
                    bind_artifact_id: None,
                    constant_value: Some("2".to_string()),
                },
                StructuralNode {
                    node_id: TreeNodeId(3),
                    kind: StructuralNodeKind::Calculation,
                    symbol: "B".to_string(),
                    parent_id: Some(TreeNodeId(1)),
                    child_ids: vec![],
                    formula_artifact_id: Some(FormulaArtifactId("formula:b".to_string())),
                    bind_artifact_id: Some(BindArtifactId("bind:b".to_string())),
                    constant_value: None,
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

        context
            .set_node_formula_text(&workspace_id, a_id, "=4")
            .unwrap();
        let result = context.recalculate(&workspace_id).unwrap();

        assert_eq!(result.run_state, OxCalcTreeRunState::Published);
        assert_eq!(result.published_values.get(&a_id), Some(&"4".to_string()));
        assert_eq!(result.published_values.get(&b_id), Some(&"5".to_string()));
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

        let view = context
            .set_node_table(&workspace_id, sales_id, sales_table_snapshot(sales_id))
            .unwrap();

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
        assert_eq!(
            removed.map(|snapshot| snapshot.table_id),
            Some("table:sales".to_string())
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

        let mut imported_context = OxCalcTreeContext::default();
        let imported_workspace_id = imported_context
            .import_workspace_snapshot(snapshot)
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
            "typed_exclusion:dynamic_indirect_raw_context_pending",
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
        context
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
