#![forbid(unsafe_code)]

//! Consumer-facing OxCalc runtime contract for tree-substrate hosts.

use std::collections::{BTreeMap, VecDeque};

use oxfml_core::{
    BindContext, BindRequest, FormulaSourceRecord, ParseRequest, bind_formula, parse_formula,
    project_red_view,
};
use thiserror::Error;

use crate::coordinator::{AcceptedCandidateResult, PublicationBundle, RejectDetail, RuntimeEffect};
use crate::dependency::{DependencyGraph, InvalidationClosure};
use crate::formula::{
    TreeCalcFormulaTextPrebindContext, TreeCalcFormulaTextPrebindError,
    TreeCalcOrderedSelectorTraversalPolicy, TreeCalcQualifiedBaseResolutionLayer,
    TreeCalcQualifiedChildrenBaseQuery, TreeCalcQualifiedChildrenBaseResolution,
    TreeFormulaBinding, TreeFormulaCatalog, TreeFormulaHostNameBindPacket,
    TreeFormulaReferenceCarrier, TreeReference, prebind_treecalc_formula_text_with_context,
    treecalc_formula_text_ordered_selector_queries,
    treecalc_formula_text_qualified_children_base_queries,
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
    Runtime(#[from] LocalTreeCalcError),
}

#[derive(Debug, Clone)]
struct OxCalcTreeWorkspaceState {
    workspace_id: OxCalcTreeWorkspaceId,
    root_node_id: TreeNodeId,
    snapshot: StructuralSnapshot,
    formula_texts: BTreeMap<TreeNodeId, String>,
    formula_text_versions: BTreeMap<TreeNodeId, u64>,
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
        let (prebind_context, prebind_diagnostics) =
            context_prebind_resolution(state, *owner_node_id, formula_text);
        let mut expression = prebind_treecalc_formula_text_with_context(
            *owner_node_id,
            formula_text,
            &prebind_context,
        )?;
        let prebound_treecalc_spans = prebound_treecalc_source_spans(&expression);
        diagnostics.extend(
            prebind_diagnostics
                .into_iter()
                .map(|diagnostic| format!("treecalc_context_prebind_resolution:{diagnostic}")),
        );
        let resolution = direct_name_carriers_from_oxfml_probe(
            state.workspace_id.as_str(),
            formula_text,
            *owner_node_id,
            &state.snapshot,
            &prebound_treecalc_spans,
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

fn prebound_treecalc_source_spans(expression: &crate::formula::TreeFormula) -> Vec<(usize, usize)> {
    expression
        .reference_carriers
        .iter()
        .filter_map(|carrier| match &carrier.reference {
            TreeReference::ReferenceCollection(collection) => match collection {
                crate::formula::TreeCalcReferenceCollection::ChildrenV1(children) => {
                    children.source_span_utf8
                }
                crate::formula::TreeCalcReferenceCollection::ReferenceLiteralArrayV1(literal) => {
                    literal.source_span_utf8()
                }
                crate::formula::TreeCalcReferenceCollection::OrderedSelectorV1(ordered) => {
                    ordered.source_span_utf8
                }
            },
            _ => None,
        })
        .collect()
}

fn context_prebind_resolution(
    state: &OxCalcTreeWorkspaceState,
    owner_node_id: TreeNodeId,
    formula_text: &str,
) -> (TreeCalcFormulaTextPrebindContext, Vec<String>) {
    let mut diagnostics = Vec::new();
    let qualified_children_bases =
        treecalc_formula_text_qualified_children_base_queries(owner_node_id, formula_text)
            .into_iter()
            .filter_map(|query| {
                context_qualified_children_base_resolution(&query, owner_node_id, &state.snapshot)
                    .map_err(|diagnostic| diagnostics.push(diagnostic))
                    .ok()
            })
            .collect::<Vec<_>>();

    let ordered_selector_resolutions =
        treecalc_formula_text_ordered_selector_queries(owner_node_id, formula_text)
            .into_iter()
            .filter_map(|query| {
                let base_resolution = query.base_token_text.as_deref().map_or(
                    ContextHostNameResolution::Resolved(owner_node_id),
                    |base_token_text| {
                        resolve_context_host_name_token(
                            base_token_text,
                            owner_node_id,
                            &state.snapshot,
                        )
                    },
                );
                let ContextHostNameResolution::Resolved(base_node_id) = base_resolution else {
                    diagnostics.push(format!(
                        "ordered_selector_unresolved:{}:{}-{}:{}",
                        query.source_token_text,
                        query.source_span_utf8.0,
                        query.source_span_utf8.1,
                        base_resolution.diagnostic_label()
                    ));
                    return None;
                };

                query
                    .to_resolution_with_structural_traversal(
                        &state.snapshot,
                        base_node_id,
                        TreeCalcOrderedSelectorTraversalPolicy::default(),
                    )
                    .map(|resolved| {
                        diagnostics.extend(resolved.traversal.diagnostics.iter().map(
                            |diagnostic| {
                                format!(
                                    "ordered_selector_traversal:{:?}:{}",
                                    diagnostic.code, diagnostic.detail
                                )
                            },
                        ));
                        resolved.resolution
                    })
                    .map_err(|error| {
                        diagnostics.push(format!(
                            "ordered_selector_unresolved:{}:{}-{}:{error}",
                            query.source_token_text,
                            query.source_span_utf8.0,
                            query.source_span_utf8.1
                        ));
                    })
                    .ok()
            })
            .collect::<Vec<_>>();

    (
        TreeCalcFormulaTextPrebindContext {
            qualified_children_bases,
            ordered_selector_resolutions,
        },
        diagnostics,
    )
}

fn context_qualified_children_base_resolution(
    query: &TreeCalcQualifiedChildrenBaseQuery,
    owner_node_id: TreeNodeId,
    snapshot: &StructuralSnapshot,
) -> Result<TreeCalcQualifiedChildrenBaseResolution, String> {
    match resolve_context_host_name_token(&query.base_token_text, owner_node_id, snapshot) {
        ContextHostNameResolution::Resolved(base_node_id) => Ok(query.to_resolution_with_layer(
            base_node_id,
            TreeCalcQualifiedBaseResolutionLayer::OxCalcStructuralPath,
            format!(
                "treecalc-context-walkup-base:v1:token={};source={}-{};base={base_node_id}",
                query.base_token_text, query.source_span_utf8.0, query.source_span_utf8.1
            ),
        )),
        other => Err(format!(
            "qualified_children_base_unresolved:{}:{}-{}:{}",
            query.source_token_text,
            query.source_span_utf8.0,
            query.source_span_utf8.1,
            other.diagnostic_label()
        )),
    }
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
    prebound_treecalc_spans: &[(usize, usize)],
) -> Result<ContextNameCarrierResolution, OxCalcTreeContextError> {
    // OxFml owns formula syntax and lexical scope; OxCalc only resolves names that
    // OxFml exposes as unresolved host candidates.
    let probe = oxfml_context_bind_probe(workspace_id, owner_node_id, formula_text);
    let mut carriers = Vec::new();
    let mut diagnostics = typed_exclusion_diagnostics(&probe, snapshot, owner_node_id);
    for candidate in probe.unresolved_names {
        if source_span_is_inside_any(candidate.source_span_utf8, prebound_treecalc_spans) {
            continue;
        }
        let token = candidate
            .source_text
            .strip_prefix("name:")
            .unwrap_or(candidate.source_text.as_str())
            .to_string();
        let resolution = resolve_context_host_name_token(&token, owner_node_id, snapshot);
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

impl ContextHostNameResolution {
    fn diagnostic_label(&self) -> &'static str {
        match self {
            Self::Resolved(_) => "resolved",
            Self::Ambiguous => "ambiguous",
            Self::Unsupported(reason) => reason,
            Self::Unresolved => "unresolved",
        }
    }
}

fn resolve_context_host_name_token(
    token: &str,
    owner_node_id: TreeNodeId,
    snapshot: &StructuralSnapshot,
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
    if segments.is_empty() {
        return ContextHostNameResolution::Unresolved;
    }

    if snapshot
        .try_get_node(snapshot.root_node_id())
        .is_some_and(|root| root.symbol.eq_ignore_ascii_case(&segments[0]))
    {
        return snapshot
            .try_resolve_descendant_path(snapshot.root_node_id(), &segments[1..])
            .map_or(
                ContextHostNameResolution::Unresolved,
                ContextHostNameResolution::Resolved,
            );
    }

    let base = match resolve_context_walkup_symbol(&segments[0], owner_node_id, snapshot) {
        ContextHostNameResolution::Resolved(base_node_id) => base_node_id,
        other => return other,
    };
    if segments.len() == 1 {
        return ContextHostNameResolution::Resolved(base);
    }
    snapshot
        .try_resolve_descendant_path(base, &segments[1..])
        .map_or(
            ContextHostNameResolution::Unresolved,
            ContextHostNameResolution::Resolved,
        )
}

fn resolve_context_walkup_symbol(
    symbol: &str,
    owner_node_id: TreeNodeId,
    snapshot: &StructuralSnapshot,
) -> ContextHostNameResolution {
    let mut scope = Some(owner_node_id);
    while let Some(scope_node_id) = scope {
        match resolve_child_symbol_in_scope(symbol, scope_node_id, snapshot) {
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
        })
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [] => ContextHostNameResolution::Unresolved,
        [node_id] => ContextHostNameResolution::Resolved(*node_id),
        _ => ContextHostNameResolution::Ambiguous,
    }
}

fn typed_exclusion_diagnostics(
    probe: &ContextOxfmlBindProbe,
    snapshot: &StructuralSnapshot,
    owner_node_id: TreeNodeId,
) -> Vec<String> {
    let mut diagnostics = Vec::new();
    for call in &probe.function_calls {
        if call.function_name.eq_ignore_ascii_case("INDIRECT") {
            diagnostics.push(format!(
                "typed_exclusion:dynamic_indirect_raw_context_pending:{}:{}-{}:owner={owner_node_id}",
                call.function_name, call.source_span_utf8.0, call.source_span_utf8.1
            ));
        } else if matches!(
            resolve_context_host_name_token(&call.function_name, owner_node_id, snapshot),
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
    if probe.reference_literal_syntax_seen {
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
                OxCalcTreeNodeCreate::new("Literal", "=SUM({A,A})"),
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
