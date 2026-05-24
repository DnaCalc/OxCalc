#![forbid(unsafe_code)]

//! Consumer-facing OxCalc runtime contract for tree-substrate hosts.

use std::collections::{BTreeMap, BTreeSet};

use oxfml_core::{
    BindContext, BindRequest, FormulaSourceRecord, ParseRequest, bind_formula, parse_formula,
    project_red_view,
};
use thiserror::Error;

use crate::coordinator::{AcceptedCandidateResult, PublicationBundle, RejectDetail, RuntimeEffect};
use crate::dependency::{DependencyGraph, InvalidationClosure};
use crate::formula::{
    TreeCalcFormulaTextPrebindError, TreeFormulaBinding, TreeFormulaCatalog,
    TreeFormulaReferenceCarrier, TreeReference, prebind_treecalc_formula_text,
};
use crate::recalc::{NodeCalcState, OverlayEntry};
use crate::structural::{
    BindArtifactId, FormulaArtifactId, StructuralEdit, StructuralError, StructuralNode,
    StructuralNodeKind, StructuralSnapshot, StructuralSnapshotId, TreeNodeId,
};
use crate::treecalc::{
    DerivationTraceRecord, LocalTreeCalcEngine, LocalTreeCalcEnvironmentContext,
    LocalTreeCalcError, LocalTreeCalcInput, LocalTreeCalcRunArtifacts, LocalTreeCalcRunState,
    LocalTreeCalcSchedulingPolicy,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OxCalcTreeDocument {
    pub structural_snapshot: StructuralSnapshot,
    pub formula_catalog: TreeFormulaCatalog,
    pub seeded_published_values: BTreeMap<TreeNodeId, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OxCalcTreeRecalcRequest {
    pub candidate_result_id: String,
    pub publication_id: String,
    pub compatibility_basis: String,
    pub artifact_token_basis: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OxCalcTreeRunState {
    Published,
    VerifiedClean,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OxCalcTreeRecalcResult {
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

#[derive(Debug, Error, PartialEq, Eq)]
pub enum OxCalcTreeRuntimeError {
    #[error(transparent)]
    Runtime(#[from] LocalTreeCalcError),
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
}

impl OxCalcTreeNodeCreate {
    #[must_use]
    pub fn new(symbol: impl Into<String>, formula_text: impl Into<String>) -> Self {
        Self {
            parent_node_id: None,
            symbol: symbol.into(),
            formula_text: formula_text.into(),
        }
    }

    #[must_use]
    pub fn under(mut self, parent_node_id: TreeNodeId) -> Self {
        self.parent_node_id = Some(parent_node_id);
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OxCalcTreeWorkspaceView {
    pub workspace_id: OxCalcTreeWorkspaceId,
    pub root_node_id: TreeNodeId,
    pub snapshot_id: StructuralSnapshotId,
    pub nodes: Vec<OxCalcTreeNodeView>,
    pub diagnostics: Vec<String>,
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
    FormulaPrebind(#[from] TreeCalcFormulaTextPrebindError),
    #[error(transparent)]
    Runtime(#[from] OxCalcTreeRuntimeError),
}

#[derive(Debug, Clone)]
struct OxCalcTreeWorkspaceState {
    workspace_id: OxCalcTreeWorkspaceId,
    root_node_id: TreeNodeId,
    snapshot: StructuralSnapshot,
    formula_texts: BTreeMap<TreeNodeId, String>,
    formula_text_versions: BTreeMap<TreeNodeId, u64>,
    seeded_published_values: BTreeMap<TreeNodeId, String>,
    last_result: Option<OxCalcTreeRecalcResult>,
}

#[derive(Debug, Clone)]
pub struct OxCalcTreeContext {
    environment: OxCalcTreeEnvironment,
    workspaces: BTreeMap<OxCalcTreeWorkspaceId, OxCalcTreeWorkspaceState>,
    next_node_id: u64,
    next_snapshot_id: u64,
    next_candidate_index: u64,
}

impl Default for OxCalcTreeContext {
    fn default() -> Self {
        Self::new(OxCalcTreeEnvironment::default())
    }
}

impl OxCalcTreeContext {
    #[must_use]
    pub fn new(environment: OxCalcTreeEnvironment) -> Self {
        Self {
            environment,
            workspaces: BTreeMap::new(),
            next_node_id: 1,
            next_snapshot_id: 1,
            next_candidate_index: 1,
        }
    }

    #[must_use]
    pub fn environment(&self) -> &OxCalcTreeEnvironment {
        &self.environment
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
                state.seeded_published_values.remove(removed_node_id);
            }
            state.snapshot = outcome.snapshot;
            state.last_result = None;
        }
        self.advance_snapshot_id();
        Ok(())
    }

    pub fn recalculate(
        &mut self,
        workspace_id: &OxCalcTreeWorkspaceId,
    ) -> Result<OxCalcTreeRecalcResult, OxCalcTreeContextError> {
        let candidate_index = self.next_candidate_index();
        let state = self.workspace(workspace_id)?;
        let catalog_build = build_context_formula_catalog(state)?;
        let document = OxCalcTreeDocument {
            structural_snapshot: state.snapshot.clone(),
            formula_catalog: catalog_build.catalog,
            seeded_published_values: state.seeded_published_values.clone(),
        };
        let snapshot_id = state.snapshot.snapshot_id();
        let facade = OxCalcTreeRuntimeFacade::new(self.environment.clone());
        let mut result = facade.execute(
            document,
            OxCalcTreeRecalcRequest {
                candidate_result_id: format!(
                    "candidate:{}:{}",
                    workspace_id.as_str(),
                    candidate_index
                ),
                publication_id: format!(
                    "publication:{}:{}",
                    workspace_id.as_str(),
                    candidate_index
                ),
                compatibility_basis: format!("snapshot:{}", snapshot_id.0),
                artifact_token_basis: format!("snapshot:{}", snapshot_id.0),
            },
        )?;
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
            .map(|node| node_view_from_state(state, node))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(OxCalcTreeWorkspaceView {
            workspace_id: workspace_id.clone(),
            root_node_id: state.root_node_id,
            snapshot_id: state.snapshot.snapshot_id(),
            nodes,
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
        node_view_from_state(state, node)
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
    let symbol_index = unique_symbols_by_name(&state.snapshot);
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
        let mut expression = prebind_treecalc_formula_text(*owner_node_id, formula_text)?;
        let resolution = direct_name_carriers_from_oxfml_probe(
            state.workspace_id.as_str(),
            formula_text,
            *owner_node_id,
            &state.snapshot,
            &symbol_index,
        )?;
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

struct ContextSymbolIndex {
    unique: BTreeMap<String, TreeNodeId>,
    ambiguous: BTreeSet<String>,
}

fn unique_symbols_by_name(snapshot: &StructuralSnapshot) -> ContextSymbolIndex {
    let mut unique = BTreeMap::new();
    let mut ambiguous = BTreeSet::new();
    for node in snapshot.nodes().values() {
        if node.node_id == snapshot.root_node_id() {
            continue;
        }
        if unique
            .insert(node.symbol.clone(), node.node_id)
            .is_some_and(|existing| existing != node.node_id)
        {
            ambiguous.insert(node.symbol.clone());
        }
    }
    for symbol in &ambiguous {
        unique.remove(symbol);
    }
    ContextSymbolIndex { unique, ambiguous }
}

struct ContextNameCarrierResolution {
    carriers: Vec<TreeFormulaReferenceCarrier>,
    diagnostics: Vec<String>,
}

fn direct_name_carriers_from_oxfml_probe(
    workspace_id: &str,
    formula_text: &str,
    owner_node_id: TreeNodeId,
    snapshot: &StructuralSnapshot,
    symbol_index: &ContextSymbolIndex,
) -> Result<ContextNameCarrierResolution, OxCalcTreeContextError> {
    // OxFml owns formula syntax and lexical scope; OxCalc only resolves names that
    // OxFml exposes as unresolved host candidates.
    let unresolved_names =
        oxfml_unresolved_name_candidates(workspace_id, owner_node_id, formula_text);
    let mut carriers = Vec::new();
    let mut diagnostics = Vec::new();
    for token in unresolved_names {
        let token = token
            .strip_prefix("name:")
            .unwrap_or(token.as_str())
            .to_string();
        if symbol_index.ambiguous.contains(&token) {
            diagnostics.push(format!("ambiguous_host_name:{token}:owner={owner_node_id}"));
            continue;
        }
        let Some(target_node_id) = symbol_index.unique.get(&token).copied() else {
            diagnostics.push(format!(
                "unresolved_host_name:{token}:owner={owner_node_id}"
            ));
            continue;
        };
        if target_node_id == owner_node_id || snapshot.try_get_node(target_node_id).is_none() {
            diagnostics.push(format!(
                "unresolved_host_name:{token}:owner={owner_node_id}"
            ));
            continue;
        }
        carriers.push(TreeFormulaReferenceCarrier::named(
            token,
            TreeReference::DirectNode { target_node_id },
        ));
    }
    Ok(ContextNameCarrierResolution {
        carriers,
        diagnostics,
    })
}

fn oxfml_unresolved_name_candidates(
    workspace_id: &str,
    owner_node_id: TreeNodeId,
    formula_text: &str,
) -> Vec<String> {
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
    bind.bound_formula
        .unresolved_references
        .into_iter()
        .map(|reference| reference.source_text)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn node_view_from_state(
    state: &OxCalcTreeWorkspaceState,
    node: &StructuralNode,
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
pub struct OxCalcTreeEnvironment {
    pub runtime_lane: OxCalcTreeRuntimeLane,
    pub session_id: Option<String>,
    pub host_capabilities: OxCalcTreeHostCapabilitySnapshot,
    pub runtime_policy: OxCalcTreeRuntimePolicy,
}

impl Default for OxCalcTreeEnvironment {
    fn default() -> Self {
        Self {
            runtime_lane: OxCalcTreeRuntimeLane::LocalSequentialTreeCalc,
            session_id: None,
            host_capabilities: OxCalcTreeHostCapabilitySnapshot::default(),
            runtime_policy: OxCalcTreeRuntimePolicy::default(),
        }
    }
}

impl OxCalcTreeEnvironment {
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
                "oxcalc_tree_environment_runtime_lane:{}",
                self.runtime_lane.as_diagnostic_value()
            ),
            format!("oxcalc_tree_environment_session_id:{session_id}"),
            format!(
                "oxcalc_tree_environment_capability_profile_id:{}",
                self.host_capabilities.capability_profile_id
            ),
            format!(
                "oxcalc_tree_environment_capability_dynamic_dependency:{}",
                self.host_capabilities.dynamic_dependency_effects
            ),
            format!(
                "oxcalc_tree_environment_capability_execution_restriction:{}",
                self.host_capabilities.execution_restriction_effects
            ),
            format!(
                "oxcalc_tree_environment_capability_sensitive:{}",
                self.host_capabilities.capability_sensitive_effects
            ),
            format!(
                "oxcalc_tree_environment_capability_shape_topology:{}",
                self.host_capabilities.shape_topology_effects
            ),
            format!(
                "oxcalc_tree_environment_runtime_policy_id:{}",
                self.runtime_policy.policy_id
            ),
            format!(
                "oxcalc_tree_environment_project_runtime_effect_overlays:{}",
                self.runtime_policy.project_runtime_effect_overlays
            ),
            format!(
                "oxcalc_tree_environment_derivation_trace_enabled:{}",
                self.runtime_policy.derivation_trace_enabled
            ),
            format!(
                "oxcalc_tree_environment_scheduling_policy:{}",
                self.runtime_policy.scheduling_policy.diagnostic_name()
            ),
        ]
    }
}

#[derive(Debug, Clone, Default)]
pub struct OxCalcTreeRuntimeFacade {
    environment: OxCalcTreeEnvironment,
    engine: LocalTreeCalcEngine,
}

impl OxCalcTreeRuntimeFacade {
    #[must_use]
    pub fn new(environment: OxCalcTreeEnvironment) -> Self {
        Self {
            environment,
            engine: LocalTreeCalcEngine,
        }
    }

    #[must_use]
    pub fn environment(&self) -> &OxCalcTreeEnvironment {
        &self.environment
    }

    pub fn execute(
        &self,
        document: OxCalcTreeDocument,
        request: OxCalcTreeRecalcRequest,
    ) -> Result<OxCalcTreeRecalcResult, OxCalcTreeRuntimeError> {
        let artifacts = self.engine.execute(LocalTreeCalcInput {
            structural_snapshot: document.structural_snapshot,
            formula_catalog: document.formula_catalog,
            seeded_published_values: document.seeded_published_values,
            seeded_published_runtime_effects: Vec::new(),
            invalidation_seeds: Vec::new(),
            previous_arg_preparation_profile_version: None,
            candidate_result_id: request.candidate_result_id,
            publication_id: request.publication_id,
            compatibility_basis: request.compatibility_basis,
            artifact_token_basis: request.artifact_token_basis,
            environment_context: self.environment.runtime_context(),
        })?;
        let mut result = OxCalcTreeRecalcResult::from(artifacts);
        result.diagnostics.extend(self.environment.diagnostics());
        Ok(result)
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

impl From<LocalTreeCalcRunArtifacts> for OxCalcTreeRecalcResult {
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
    fn treecalc_context_reports_ambiguous_host_names_from_oxcalc_namespace() {
        let mut context = OxCalcTreeContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new("workspace:ambiguous"))
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
            diagnostic.contains("treecalc_context_host_name_resolution:ambiguous_host_name:A")
        }));
    }

    #[test]
    fn treecalc_environment_carries_non_narrow_consumer_inputs() {
        let environment = OxCalcTreeEnvironment::new()
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
        let facade = OxCalcTreeRuntimeFacade::new(environment.clone());

        assert_eq!(facade.environment(), &environment);
        assert_eq!(
            facade.environment().runtime_lane,
            OxCalcTreeRuntimeLane::LocalSequentialTreeCalc
        );
        assert_eq!(
            facade.environment().session_id.as_deref(),
            Some("session:tree-host")
        );
        assert_eq!(
            facade.environment().host_capabilities.capability_profile_id,
            "capability-profile:tree-host"
        );
        assert!(
            facade
                .environment()
                .host_capabilities
                .capability_sensitive_effects
        );
        assert!(
            facade
                .environment()
                .host_capabilities
                .shape_topology_effects
        );
        assert_eq!(
            facade.environment().runtime_policy.policy_id,
            "runtime-policy:tree-host"
        );
    }

    #[test]
    fn treecalc_runtime_facade_projects_environment_diagnostics() {
        let facade = OxCalcTreeRuntimeFacade::new(
            OxCalcTreeEnvironment::new()
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
        );

        let result = facade
            .execute(
                OxCalcTreeDocument {
                    structural_snapshot: snapshot(),
                    formula_catalog: TreeFormulaCatalog::new([TreeFormulaBinding {
                        owner_node_id: TreeNodeId(3),
                        formula_artifact_id: FormulaArtifactId("formula:b".to_string()),
                        bind_artifact_id: Some(BindArtifactId("bind:b".to_string())),
                        expression: fixture_formula(
                            TreeNodeId(3),
                            FixtureFormulaAst::Literal {
                                value: "7".to_string(),
                            },
                        ),
                    }]),
                    seeded_published_values: BTreeMap::new(),
                },
                OxCalcTreeRecalcRequest {
                    candidate_result_id: "cand:environment".to_string(),
                    publication_id: "pub:environment".to_string(),
                    compatibility_basis: "snapshot:1".to_string(),
                    artifact_token_basis: "snapshot:1".to_string(),
                },
            )
            .unwrap();

        assert!(result.diagnostics.iter().any(|diagnostic| {
            diagnostic == "oxcalc_tree_environment_runtime_lane:local_sequential_treecalc"
        }));
        assert!(result.diagnostics.iter().any(|diagnostic| {
            diagnostic == "oxcalc_tree_environment_session_id:session:diagnostic"
        }));
        assert!(result.diagnostics.iter().any(|diagnostic| {
            diagnostic
                == "oxcalc_tree_environment_capability_profile_id:capability-profile:diagnostic"
        }));
        assert!(result.diagnostics.iter().any(|diagnostic| {
            diagnostic == "oxcalc_tree_environment_capability_sensitive:true"
        }));
        assert!(result.diagnostics.iter().any(|diagnostic| {
            diagnostic == "oxcalc_tree_environment_capability_shape_topology:false"
        }));
        assert!(result.diagnostics.iter().any(|diagnostic| {
            diagnostic == "oxcalc_tree_environment_runtime_policy_id:runtime-policy:diagnostic"
        }));
    }

    #[test]
    fn treecalc_runtime_derived_effects_use_environment_context() {
        let facade = OxCalcTreeRuntimeFacade::new(
            OxCalcTreeEnvironment::new()
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
        );

        let result = facade
            .execute(
                OxCalcTreeDocument {
                    structural_snapshot: snapshot(),
                    formula_catalog: TreeFormulaCatalog::new([TreeFormulaBinding {
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
                    seeded_published_values: BTreeMap::new(),
                },
                OxCalcTreeRecalcRequest {
                    candidate_result_id: "cand:environment-runtime".to_string(),
                    publication_id: "pub:environment-runtime".to_string(),
                    compatibility_basis: "snapshot:1".to_string(),
                    artifact_token_basis: "snapshot:1".to_string(),
                },
            )
            .unwrap();

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
        let facade = OxCalcTreeRuntimeFacade::new(OxCalcTreeEnvironment::new());
        let document = OxCalcTreeDocument {
            structural_snapshot: snapshot(),
            formula_catalog: TreeFormulaCatalog::new([TreeFormulaBinding {
                owner_node_id: TreeNodeId(3),
                formula_artifact_id: FormulaArtifactId("formula:b".to_string()),
                bind_artifact_id: Some(BindArtifactId("bind:b".to_string())),
                expression: fixture_formula(
                    TreeNodeId(3),
                    FixtureFormulaAst::Literal {
                        value: "7".to_string(),
                    },
                ),
            }]),
            seeded_published_values: BTreeMap::new(),
        };

        let published = facade
            .execute(
                document.clone(),
                OxCalcTreeRecalcRequest {
                    candidate_result_id: "cand:publish-overlay-projection".to_string(),
                    publication_id: "pub:publish-overlay-projection".to_string(),
                    compatibility_basis: "snapshot:1".to_string(),
                    artifact_token_basis: "snapshot:1".to_string(),
                },
            )
            .unwrap();

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

        let verified_clean = facade
            .execute(
                OxCalcTreeDocument {
                    seeded_published_values: BTreeMap::from([(TreeNodeId(3), "7".to_string())]),
                    ..document
                },
                OxCalcTreeRecalcRequest {
                    candidate_result_id: "cand:verified-clean-overlay-projection".to_string(),
                    publication_id: "pub:verified-clean-overlay-projection".to_string(),
                    compatibility_basis: "snapshot:1".to_string(),
                    artifact_token_basis: "snapshot:1".to_string(),
                },
            )
            .unwrap();

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
    fn treecalc_runtime_facade_executes_published_run() {
        let facade = OxCalcTreeRuntimeFacade::new(OxCalcTreeEnvironment::new());

        let result = facade
            .execute(
                OxCalcTreeDocument {
                    structural_snapshot: snapshot(),
                    formula_catalog: TreeFormulaCatalog::new([TreeFormulaBinding {
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
                    seeded_published_values: BTreeMap::new(),
                },
                OxCalcTreeRecalcRequest {
                    candidate_result_id: "cand:consumer".to_string(),
                    publication_id: "pub:consumer".to_string(),
                    compatibility_basis: "snapshot:1".to_string(),
                    artifact_token_basis: "snapshot:1".to_string(),
                },
            )
            .unwrap();

        assert_eq!(result.run_state, OxCalcTreeRunState::Published);
        assert_eq!(result.published_values[&TreeNodeId(3)], "5");
        assert!(result.publication_bundle.is_some());
    }

    #[test]
    fn treecalc_runtime_facade_exposes_execution_restriction_family_directly() {
        let facade = OxCalcTreeRuntimeFacade::new(OxCalcTreeEnvironment::new());

        let result = facade
            .execute(
                OxCalcTreeDocument {
                    structural_snapshot: snapshot(),
                    formula_catalog: TreeFormulaCatalog::new([TreeFormulaBinding {
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
                    seeded_published_values: BTreeMap::new(),
                },
                OxCalcTreeRecalcRequest {
                    candidate_result_id: "cand:host".to_string(),
                    publication_id: "pub:host".to_string(),
                    compatibility_basis: "snapshot:1".to_string(),
                    artifact_token_basis: "snapshot:1".to_string(),
                },
            )
            .unwrap();

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
    fn treecalc_runtime_facade_exposes_capability_sensitive_family_directly() {
        let facade = OxCalcTreeRuntimeFacade::new(OxCalcTreeEnvironment::new());

        let result = facade
            .execute(
                OxCalcTreeDocument {
                    structural_snapshot: snapshot(),
                    formula_catalog: TreeFormulaCatalog::new([TreeFormulaBinding {
                        owner_node_id: TreeNodeId(3),
                        formula_artifact_id: FormulaArtifactId("formula:b".to_string()),
                        bind_artifact_id: Some(BindArtifactId("bind:b".to_string())),
                        expression: fixture_formula(
                            TreeNodeId(3),
                            FixtureFormulaAst::Reference(TreeReference::CapabilitySensitive {
                                carrier_id: "carrier:capability".to_string(),
                                detail: "host_function_availability".to_string(),
                            }),
                        ),
                    }]),
                    seeded_published_values: BTreeMap::new(),
                },
                OxCalcTreeRecalcRequest {
                    candidate_result_id: "cand:capability".to_string(),
                    publication_id: "pub:capability".to_string(),
                    compatibility_basis: "snapshot:1".to_string(),
                    artifact_token_basis: "snapshot:1".to_string(),
                },
            )
            .unwrap();

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
    fn treecalc_runtime_facade_exposes_shape_topology_family_directly() {
        let facade = OxCalcTreeRuntimeFacade::new(OxCalcTreeEnvironment::new());

        let result = facade
            .execute(
                OxCalcTreeDocument {
                    structural_snapshot: snapshot(),
                    formula_catalog: TreeFormulaCatalog::new([TreeFormulaBinding {
                        owner_node_id: TreeNodeId(3),
                        formula_artifact_id: FormulaArtifactId("formula:b".to_string()),
                        bind_artifact_id: Some(BindArtifactId("bind:b".to_string())),
                        expression: fixture_formula(
                            TreeNodeId(3),
                            FixtureFormulaAst::Reference(TreeReference::ShapeTopology {
                                carrier_id: "carrier:shape".to_string(),
                                detail: "range_shape_projection".to_string(),
                            }),
                        ),
                    }]),
                    seeded_published_values: BTreeMap::new(),
                },
                OxCalcTreeRecalcRequest {
                    candidate_result_id: "cand:shape".to_string(),
                    publication_id: "pub:shape".to_string(),
                    compatibility_basis: "snapshot:1".to_string(),
                    artifact_token_basis: "snapshot:1".to_string(),
                },
            )
            .unwrap();

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
    fn treecalc_runtime_facade_exposes_dynamic_dependency_family_directly() {
        let facade = OxCalcTreeRuntimeFacade::new(OxCalcTreeEnvironment::new());

        let result = facade
            .execute(
                OxCalcTreeDocument {
                    structural_snapshot: snapshot(),
                    formula_catalog: TreeFormulaCatalog::new([TreeFormulaBinding {
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
                    seeded_published_values: BTreeMap::new(),
                },
                OxCalcTreeRecalcRequest {
                    candidate_result_id: "cand:dynamic".to_string(),
                    publication_id: "pub:dynamic".to_string(),
                    compatibility_basis: "snapshot:1".to_string(),
                    artifact_token_basis: "snapshot:1".to_string(),
                },
            )
            .unwrap();

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
