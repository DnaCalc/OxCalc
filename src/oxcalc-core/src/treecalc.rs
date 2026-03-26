#![forbid(unsafe_code)]

//! Local sequential TreeCalc runtime facade.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use oxfml_core::binding::{bind_formula, BindContext, BindRequest, NameKind};
use oxfml_core::interface::ReturnedValueSurfaceKind;
use oxfml_core::red::project_red_view;
use oxfml_core::source::{FormulaSourceRecord, StructureContextVersion};
use oxfml_core::syntax::parser::{parse_formula, ParseRequest};
use oxfml_core::EvaluationBackend;
use oxfunc_core::value::{EvalValue, ExcelText, ReferenceKind, ReferenceLike};
use thiserror::Error;

use crate::coordinator::{
    AcceptedCandidateResult, CoordinatorError, PublicationBundle, RejectDetail, RejectKind,
    RuntimeEffect, TreeCalcCoordinator,
};
use crate::dependency::{
    DependencyDescriptor, DependencyDescriptorKind, DependencyGraph, InvalidationClosure,
    InvalidationReasonKind, InvalidationSeed,
};
use crate::formula::{FormulaBinaryOp, TreeFormula, TreeFormulaCatalog, TreeReference};
use crate::recalc::{
    NodeCalcState, OverlayEntry, OverlayKey, OverlayKind, RecalcError, Stage1RecalcTracker,
};
use crate::structural::{StructuralSnapshot, TreeNodeId};
use crate::upstream_host::{
    MinimalBindingWorld, MinimalFormulaSlotFacts, MinimalHostInfoMode, MinimalRuntimeCatalogFacts,
    MinimalTypedQueryFacts, MinimalUpstreamHostPacket, UpstreamDefinedNameBinding,
    UpstreamHostAnchor,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LocalTreeCalcRunState {
    Published,
    VerifiedClean,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalTreeCalcInput {
    pub structural_snapshot: StructuralSnapshot,
    pub formula_catalog: TreeFormulaCatalog,
    pub seeded_published_values: BTreeMap<TreeNodeId, String>,
    pub candidate_result_id: String,
    pub publication_id: String,
    pub compatibility_basis: String,
    pub artifact_token_basis: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalTreeCalcRunArtifacts {
    pub result_state: LocalTreeCalcRunState,
    pub dependency_graph: DependencyGraph,
    pub invalidation_closure: InvalidationClosure,
    pub evaluation_order: Vec<TreeNodeId>,
    pub runtime_effects: Vec<RuntimeEffect>,
    pub runtime_effect_overlays: Vec<OverlayEntry>,
    pub local_candidate: Option<LocalEvaluatorCandidate>,
    pub candidate_result: Option<AcceptedCandidateResult>,
    pub publication_bundle: Option<PublicationBundle>,
    pub reject_detail: Option<RejectDetail>,
    pub published_values: BTreeMap<TreeNodeId, String>,
    pub node_states: BTreeMap<TreeNodeId, NodeCalcState>,
    pub diagnostics: Vec<String>,
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
    #[error("no value is available for referenced node {node_id}")]
    MissingReferencedValue { node_id: TreeNodeId },
    #[error("value '{value}' for node {node_id} is not a supported local integer")]
    UnsupportedNumericValue { node_id: TreeNodeId, value: String },
    #[error("function '{function_name}' is not supported in the local sequential evaluator")]
    UnsupportedFunction { function_name: String },
    #[error("formula family contains a cycle; local sequential runtime cannot yet evaluate it")]
    CycleDetected,
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalEvaluatorCandidate {
    pub candidate_result_id: String,
    pub target_set: Vec<TreeNodeId>,
    pub value_updates: BTreeMap<TreeNodeId, String>,
    pub runtime_effects: Vec<RuntimeEffect>,
    pub diagnostic_events: Vec<String>,
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
        let prepared_formulas = input
            .formula_catalog
            .bindings_by_owner()
            .values()
            .map(|binding| {
                prepare_oxfml_formula(&input.structural_snapshot, binding)
                    .map(|prepared| (binding.owner_node_id, prepared))
            })
            .collect::<Result<BTreeMap<_, _>, _>>()?;
        let dependency_graph = DependencyGraph::build(
            &input.structural_snapshot,
            &prepared_formulas
                .values()
                .flat_map(oxfml_dependency_descriptors)
                .collect::<Vec<_>>(),
        );
        let formula_owner_ids = input.formula_catalog.owner_node_ids();
        let invalidation_closure = dependency_graph.derive_invalidation_closure(
            &formula_owner_ids
                .iter()
                .copied()
                .map(|node_id| InvalidationSeed {
                    node_id,
                    reason: InvalidationReasonKind::StructuralRecalcOnly,
                })
                .collect::<Vec<_>>(),
        );

        let mut coordinator = TreeCalcCoordinator::new(input.structural_snapshot.clone());
        coordinator.seed_published_view(&input.seeded_published_values, None, &[]);
        let mut recalc_tracker = Stage1RecalcTracker::new(input.structural_snapshot.clone());
        let mut working_values =
            seed_working_values(&input.structural_snapshot, &input.seeded_published_values);

        let mut value_updates = BTreeMap::new();
        let mut runtime_effects = Vec::new();
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

        for node_id in &formula_owner_ids {
            recalc_tracker.mark_dirty(*node_id);
            recalc_tracker.mark_needed(*node_id)?;
        }

        let evaluation_order =
            match topological_formula_order(&dependency_graph, &formula_owner_ids) {
                Ok(order) => order,
                Err(error) => {
                    return reject_run(
                        &input,
                        &mut coordinator,
                        &mut recalc_tracker,
                        dependency_graph,
                        invalidation_closure,
                        Vec::new(),
                        Vec::new(),
                        Vec::new(),
                        diagnostics,
                        &formula_owner_ids,
                        None,
                        error,
                    );
                }
            };

        for node_id in &evaluation_order {
            recalc_tracker.begin_evaluate(*node_id, &input.compatibility_basis)?;
            let prepared = prepared_formulas
                .get(node_id)
                .ok_or(LocalTreeCalcError::MissingFormulaBinding { node_id: *node_id })?;
            let computed_value = match evaluate_via_oxfml(prepared, &working_values) {
                Ok(value) => value,
                Err(failure) => {
                    runtime_effects.extend(failure.runtime_effects.clone());
                    diagnostics.extend(failure.diagnostics.clone());
                    let runtime_effect_overlays =
                        build_runtime_effect_overlays(&input, *node_id, &failure.runtime_effects);
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
                        &formula_owner_ids,
                        Some(LocalEvaluatorCandidate {
                            candidate_result_id: input.candidate_result_id.clone(),
                            target_set: formula_owner_ids.clone(),
                            value_updates,
                            runtime_effects: failure.runtime_effects,
                            diagnostic_events: vec![failure.error.to_string()],
                        }),
                        failure.error,
                    );
                }
            };
            let published_value = input.seeded_published_values.get(node_id);

            if published_value.is_some_and(|value| value == &computed_value) {
                recalc_tracker.verify_clean(*node_id)?;
                diagnostics.push(format!("verified_clean:{node_id}"));
            } else {
                recalc_tracker.produce_candidate_result(
                    *node_id,
                    &input.compatibility_basis,
                    &input.candidate_result_id,
                )?;
                working_values.insert(*node_id, computed_value.clone());
                value_updates.insert(*node_id, computed_value);
            }
        }

        if value_updates.is_empty() {
            return Ok(LocalTreeCalcRunArtifacts {
                result_state: LocalTreeCalcRunState::VerifiedClean,
                dependency_graph,
                invalidation_closure,
                evaluation_order,
                runtime_effects,
                runtime_effect_overlays: Vec::new(),
                local_candidate: None,
                candidate_result: None,
                publication_bundle: None,
                reject_detail: None,
                published_values: coordinator.published_view().values.clone(),
                node_states: recalc_tracker.node_states().clone(),
                diagnostics,
            });
        }

        let local_candidate = LocalEvaluatorCandidate {
            candidate_result_id: input.candidate_result_id.clone(),
            target_set: evaluation_order.clone(),
            value_updates,
            runtime_effects,
            diagnostic_events: diagnostics.clone(),
        };
        let candidate_result = adapt_local_candidate(&input, &local_candidate);

        coordinator.admit_candidate_work(candidate_result.clone())?;
        coordinator.record_accepted_candidate_result(&input.candidate_result_id)?;
        let publication_bundle = coordinator.accept_and_publish(&input.publication_id)?;
        for node_id in local_candidate.value_updates.keys().copied() {
            recalc_tracker.publish_and_clear(node_id)?;
        }

        Ok(LocalTreeCalcRunArtifacts {
            result_state: LocalTreeCalcRunState::Published,
            dependency_graph,
            invalidation_closure,
            evaluation_order,
            runtime_effects: local_candidate.runtime_effects.clone(),
            runtime_effect_overlays: Vec::new(),
            local_candidate: Some(local_candidate),
            candidate_result: Some(candidate_result),
            publication_bundle: Some(publication_bundle),
            reject_detail: None,
            published_values: coordinator.published_view().values.clone(),
            node_states: recalc_tracker.node_states().clone(),
            diagnostics,
        })
    }
}

fn adapt_local_candidate(
    input: &LocalTreeCalcInput,
    local_candidate: &LocalEvaluatorCandidate,
) -> AcceptedCandidateResult {
    AcceptedCandidateResult {
        candidate_result_id: local_candidate.candidate_result_id.clone(),
        structural_snapshot_id: input.structural_snapshot.snapshot_id(),
        artifact_token_basis: input.artifact_token_basis.clone(),
        compatibility_basis: input.compatibility_basis.clone(),
        target_set: local_candidate.target_set.clone(),
        value_updates: local_candidate.value_updates.clone(),
        dependency_shape_updates: vec![],
        runtime_effects: local_candidate.runtime_effects.clone(),
        diagnostic_events: local_candidate.diagnostic_events.clone(),
    }
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
    formula_owner_ids: &[TreeNodeId],
    local_candidate: Option<LocalEvaluatorCandidate>,
    error: LocalTreeCalcError,
) -> Result<LocalTreeCalcRunArtifacts, LocalTreeCalcError> {
    diagnostics.push(format!("candidate_rejected:{}", error));
    let placeholder_candidate = AcceptedCandidateResult {
        candidate_result_id: input.candidate_result_id.clone(),
        structural_snapshot_id: input.structural_snapshot.snapshot_id(),
        artifact_token_basis: input.artifact_token_basis.clone(),
        compatibility_basis: input.compatibility_basis.clone(),
        target_set: formula_owner_ids.to_vec(),
        value_updates: BTreeMap::new(),
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

    Ok(LocalTreeCalcRunArtifacts {
        result_state: LocalTreeCalcRunState::Rejected,
        dependency_graph,
        invalidation_closure,
        evaluation_order,
        runtime_effects,
        runtime_effect_overlays,
        local_candidate,
        candidate_result: None,
        publication_bundle: None,
        reject_detail: Some(reject_detail),
        published_values: coordinator.published_view().values.clone(),
        node_states: recalc_tracker.node_states().clone(),
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
        | LocalTreeCalcError::UnsupportedNumericValue { .. }
        | LocalTreeCalcError::UnsupportedFunction { .. }
        | LocalTreeCalcError::DivisionByZero
        | LocalTreeCalcError::OxfmlHostFailure { .. }
        | LocalTreeCalcError::OxfmlBindUnresolved { .. }
        | LocalTreeCalcError::OxfmlCommitRejected { .. } => RejectKind::HostInjectedFailure,
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
    runtime_effects
        .iter()
        .enumerate()
        .map(|(index, runtime_effect)| OverlayEntry {
            key: OverlayKey {
                owner_node_id,
                overlay_kind: OverlayKind::DynamicDependency,
                structural_snapshot_id: input.structural_snapshot.snapshot_id(),
                compatibility_basis: input.compatibility_basis.clone(),
                payload_identity: Some(format!(
                    "{}:runtime_effect:{index}",
                    input.candidate_result_id
                )),
            },
            is_protected: true,
            is_eviction_eligible: false,
            detail: format!("{}|{}", runtime_effect.kind, runtime_effect.detail),
        })
        .collect()
}

fn seed_working_values(
    snapshot: &StructuralSnapshot,
    seeded_published_values: &BTreeMap<TreeNodeId, String>,
) -> BTreeMap<TreeNodeId, String> {
    let mut values = seeded_published_values.clone();
    for node in snapshot.nodes().values() {
        if let Some(constant_value) = &node.constant_value {
            values.insert(node.node_id, constant_value.clone());
        }
    }
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

#[derive(Debug, Clone, PartialEq, Eq)]
enum ResidualCarrierKind {
    HostSensitive,
    DynamicPotential,
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
    target_node_id: TreeNodeId,
    kind: DependencyDescriptorKind,
    carrier_detail: String,
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
    unresolved_bindings: Vec<SyntheticUnresolvedBinding>,
    residuals: Vec<ResidualCarrier>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PreparedOxfmlFormula {
    binding: crate::formula::TreeFormulaBinding,
    source: FormulaSourceRecord,
    translated: TranslatedFormula,
    bound_formula: oxfml_core::binding::BoundFormula,
    bind_diagnostics: Vec<String>,
}

fn prepare_oxfml_formula(
    snapshot: &StructuralSnapshot,
    binding: &crate::formula::TreeFormulaBinding,
) -> Result<PreparedOxfmlFormula, LocalTreeCalcError> {
    let translated = translate_formula(snapshot, binding.owner_node_id, &binding.expression);
    let source = FormulaSourceRecord::new(
        binding.formula_artifact_id.to_string(),
        binding.owner_node_id.0,
        translated.source_text.clone(),
    );
    let parse = parse_formula(ParseRequest {
        source: source.clone(),
    });
    let red_projection = project_red_view(source.formula_stable_id.clone(), &parse.green_tree);
    let bind_result = bind_formula(BindRequest {
        source: source.clone(),
        green_tree: parse.green_tree,
        red_projection,
        context: BindContext {
            caller_row: synthetic_cell_row(binding.owner_node_id),
            caller_col: 1,
            formula_token: source.formula_token(),
            structure_context_version: StructureContextVersion(snapshot.snapshot_id().to_string()),
            names: translated
                .reference_bindings
                .iter()
                .map(|reference| (reference.token.clone(), NameKind::ReferenceLike))
                .collect(),
            ..BindContext::default()
        },
    });

    Ok(PreparedOxfmlFormula {
        binding: binding.clone(),
        source,
        translated,
        bind_diagnostics: bind_result
            .bound_formula
            .diagnostics
            .iter()
            .map(|diagnostic| format!("oxfml_bind_diagnostic:{}", diagnostic.message))
            .collect(),
        bound_formula: bind_result.bound_formula,
    })
}

fn oxfml_dependency_descriptors(prepared: &PreparedOxfmlFormula) -> Vec<DependencyDescriptor> {
    let bound_name_tokens = prepared
        .bound_formula
        .normalized_references
        .iter()
        .filter_map(|reference| match reference {
            oxfml_core::binding::NormalizedReference::Name(name) => Some(name.name.as_str()),
            _ => None,
        })
        .collect::<BTreeSet<_>>();

    let mut descriptors = prepared
        .translated
        .reference_bindings
        .iter()
        .enumerate()
        .filter(|(_, reference)| bound_name_tokens.contains(reference.token.as_str()))
        .map(|(index, reference)| DependencyDescriptor {
            descriptor_id: format!(
                "bind:{}:oxfml_ref:{index}",
                prepared.binding.formula_artifact_id.0
            ),
            owner_node_id: prepared.binding.owner_node_id,
            target_node_id: Some(reference.target_node_id),
            kind: reference.kind,
            carrier_detail: reference.carrier_detail.clone(),
            requires_rebind_on_structural_change: reference.requires_rebind_on_structural_change,
        })
        .collect::<Vec<_>>();

    descriptors.extend(
        prepared
            .translated
            .unresolved_bindings
            .iter()
            .enumerate()
            .map(|(index, unresolved)| DependencyDescriptor {
                descriptor_id: format!(
                    "bind:{}:oxfml_unresolved:{index}",
                    prepared.binding.formula_artifact_id.0
                ),
                owner_node_id: prepared.binding.owner_node_id,
                target_node_id: None,
                kind: unresolved.kind,
                carrier_detail: unresolved.carrier_detail.clone(),
                requires_rebind_on_structural_change: unresolved
                    .requires_rebind_on_structural_change,
            }),
    );

    descriptors.extend(prepared.translated.residuals.iter().enumerate().map(
        |(index, residual)| DependencyDescriptor {
            descriptor_id: format!(
                "bind:{}:oxfml_residual:{index}",
                prepared.binding.formula_artifact_id.0
            ),
            owner_node_id: prepared.binding.owner_node_id,
            target_node_id: None,
            kind: match residual.kind {
                ResidualCarrierKind::HostSensitive => DependencyDescriptorKind::HostSensitive,
                ResidualCarrierKind::DynamicPotential => DependencyDescriptorKind::DynamicPotential,
            },
            carrier_detail: format!("residual:{}:{}", residual.carrier_id, residual.detail),
            requires_rebind_on_structural_change: matches!(
                residual.kind,
                ResidualCarrierKind::HostSensitive
            ),
        },
    ));

    descriptors.sort_by(|left, right| left.descriptor_id.cmp(&right.descriptor_id));
    descriptors
}

fn evaluate_via_oxfml(
    prepared: &PreparedOxfmlFormula,
    working_values: &BTreeMap<TreeNodeId, String>,
) -> Result<String, LocalFormulaEvaluationFailure> {
    if let Some(unresolved) = prepared.bound_formula.unresolved_references.first() {
        return Err(LocalFormulaEvaluationFailure {
            error: LocalTreeCalcError::OxfmlBindUnresolved {
                owner_node_id: prepared.binding.owner_node_id,
                detail: format!("{} ({})", unresolved.source_text, unresolved.reason),
            },
            runtime_effects: Vec::new(),
            diagnostics: prepared.bind_diagnostics.clone(),
        });
    }

    let run = match build_upstream_host_packet(prepared, working_values)
        .recalc(EvaluationBackend::OxFuncBacked)
    {
        Ok(run) => run,
        Err(detail) => {
            if let Some(residual) = prepared.translated.residuals.first() {
                let runtime_effects = prepared
                    .translated
                    .residuals
                    .iter()
                    .map(residual_runtime_effect)
                    .collect::<Vec<_>>();
                let error = match residual.kind {
                    ResidualCarrierKind::HostSensitive => {
                        LocalTreeCalcError::HostSensitiveReference {
                            owner_node_id: residual.owner_node_id,
                            detail: residual.detail.clone(),
                        }
                    }
                    ResidualCarrierKind::DynamicPotential => LocalTreeCalcError::DynamicReference {
                        owner_node_id: residual.owner_node_id,
                        detail: residual.detail.clone(),
                    },
                };

                return Err(LocalFormulaEvaluationFailure {
                    error,
                    runtime_effects,
                    diagnostics: prepared
                        .bind_diagnostics
                        .iter()
                        .cloned()
                        .chain(std::iter::once(format!("oxfml_host_error:{detail}")))
                        .collect(),
                });
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

    if matches!(
        run.returned_value_surface.kind,
        ReturnedValueSurfaceKind::TypedHostProviderOutcome
    ) {
        let runtime_effects = prepared
            .translated
            .residuals
            .iter()
            .map(residual_runtime_effect)
            .collect::<Vec<_>>();

        if let Some(residual) = prepared.translated.residuals.first() {
            let error = match residual.kind {
                ResidualCarrierKind::HostSensitive => LocalTreeCalcError::HostSensitiveReference {
                    owner_node_id: residual.owner_node_id,
                    detail: residual.detail.clone(),
                },
                ResidualCarrierKind::DynamicPotential => LocalTreeCalcError::DynamicReference {
                    owner_node_id: residual.owner_node_id,
                    detail: residual.detail.clone(),
                },
            };

            return Err(LocalFormulaEvaluationFailure {
                error,
                runtime_effects,
                diagnostics: prepared
                    .bind_diagnostics
                    .iter()
                    .cloned()
                    .chain(
                        run.trace_events
                            .iter()
                            .map(|event| format!("oxfml_trace:{:?}", event.event_kind)),
                    )
                    .collect(),
            });
        }
    }

    match &run.commit_decision {
        oxfml_core::seam::AcceptDecision::Accepted(bundle) => Ok(value_payload_to_string(
            &bundle.value_delta.published_payload,
        )),
        oxfml_core::seam::AcceptDecision::Rejected(reject) => Err(LocalFormulaEvaluationFailure {
            error: LocalTreeCalcError::OxfmlCommitRejected {
                owner_node_id: prepared.binding.owner_node_id,
                detail: format!("{:?}", reject.reject_code),
            },
            runtime_effects: Vec::new(),
            diagnostics: vec![format!("oxfml_reject:{:?}", reject.reject_code)],
        }),
    }
}

fn build_upstream_host_packet(
    prepared: &PreparedOxfmlFormula,
    working_values: &BTreeMap<TreeNodeId, String>,
) -> MinimalUpstreamHostPacket {
    let mut defined_name_bindings = BTreeMap::new();
    for reference in &prepared.translated.reference_bindings {
        defined_name_bindings.insert(
            reference.token.clone(),
            UpstreamDefinedNameBinding::Reference(ReferenceLike {
                kind: ReferenceKind::A1,
                target: synthetic_cell_target(reference.target_node_id),
            }),
        );
    }

    let cell_fixture = working_values
        .iter()
        .map(|(node_id, value)| (synthetic_cell_target(*node_id), string_to_eval_value(value)))
        .collect();

    MinimalUpstreamHostPacket {
        formula_slot: MinimalFormulaSlotFacts {
            fixture_input_id: format!("fixture:{}", prepared.source.formula_stable_id.0),
            formula_slot_id: Some(prepared.binding.owner_node_id.0.to_string()),
            formula_stable_id: prepared.source.formula_stable_id.0.clone(),
            formula_text: prepared.source.entered_formula_text.clone(),
            formula_text_version: prepared.source.formula_text_version.0,
            formula_channel_kind: prepared.source.formula_channel_kind,
            caller_anchor: UpstreamHostAnchor {
                row: synthetic_cell_row(prepared.binding.owner_node_id),
                col: 1,
            },
            active_selection_anchor: None,
            structure_context_version: prepared.bound_formula.structure_context_version.clone(),
        },
        binding_world: MinimalBindingWorld {
            cell_fixture,
            defined_name_bindings,
            table_catalog: Vec::new(),
            enclosing_table_ref: None,
            caller_table_region: None,
        },
        typed_query_facts: MinimalTypedQueryFacts {
            host_info_mode: if prepared
                .translated
                .residuals
                .iter()
                .any(|residual| matches!(residual.kind, ResidualCarrierKind::HostSensitive))
            {
                MinimalHostInfoMode::ProviderFailure {
                    detail: "treecalc.host_sensitive_reference".to_string(),
                }
            } else {
                MinimalHostInfoMode::Disabled
            },
            rtd_mode: if prepared
                .translated
                .residuals
                .iter()
                .any(|residual| matches!(residual.kind, ResidualCarrierKind::DynamicPotential))
            {
                crate::upstream_host::MinimalRtdMode::CapabilityDenied
            } else {
                crate::upstream_host::MinimalRtdMode::Disabled
            },
            locale_context_kind: crate::upstream_host::MinimalLocaleContextKind::Disabled,
            now_serial: None,
            random_value: None,
            registered_external_present: false,
        },
        runtime_catalog: MinimalRuntimeCatalogFacts::default(),
    }
}

fn translate_formula(
    snapshot: &StructuralSnapshot,
    owner_node_id: TreeNodeId,
    formula: &TreeFormula,
) -> TranslatedFormula {
    let mut state = TranslationState {
        snapshot,
        owner_node_id,
        next_reference_index: 0,
        reference_bindings: Vec::new(),
        unresolved_bindings: Vec::new(),
        residuals: Vec::new(),
    };
    let source_text = state.translate(formula);
    TranslatedFormula {
        source_text,
        reference_bindings: state.reference_bindings,
        unresolved_bindings: state.unresolved_bindings,
        residuals: state.residuals,
    }
}

struct TranslationState<'a> {
    snapshot: &'a StructuralSnapshot,
    owner_node_id: TreeNodeId,
    next_reference_index: usize,
    reference_bindings: Vec<SyntheticReferenceBinding>,
    unresolved_bindings: Vec<SyntheticUnresolvedBinding>,
    residuals: Vec<ResidualCarrier>,
}

impl TranslationState<'_> {
    fn translate(&mut self, formula: &TreeFormula) -> String {
        match formula {
            TreeFormula::Literal { value } => render_literal(value),
            TreeFormula::Reference(reference) => self.translate_reference(reference),
            TreeFormula::Binary { op, left, right } => {
                let left = self.translate(left);
                let right = self.translate(right);
                let operator = match op {
                    FormulaBinaryOp::Add => "+",
                    FormulaBinaryOp::Subtract => "-",
                    FormulaBinaryOp::Multiply => "*",
                    FormulaBinaryOp::Divide => "/",
                };
                format!("({left}{operator}{right})")
            }
            TreeFormula::FunctionCall {
                function_name,
                arguments,
                ..
            } => {
                let arguments = arguments
                    .iter()
                    .map(|argument| self.translate(argument))
                    .collect::<Vec<_>>();
                format!(
                    "{}({})",
                    function_name.to_ascii_uppercase(),
                    arguments.join(",")
                )
            }
        }
    }

    fn translate_reference(&mut self, reference: &TreeReference) -> String {
        match reference {
            TreeReference::DirectNode { target_node_id } => self.bind_target(
                *target_node_id,
                reference.descriptor_kind(),
                reference.carrier_detail(),
                reference.requires_rebind_on_structural_change(),
            ),
            TreeReference::ProjectionPath { .. }
            | TreeReference::RelativePath { .. }
            | TreeReference::SiblingOffset { .. } => {
                if let Some(target_node_id) =
                    reference.resolve_target(self.snapshot, self.owner_node_id)
                {
                    self.bind_target(
                        target_node_id,
                        reference.descriptor_kind(),
                        reference.carrier_detail(),
                        reference.requires_rebind_on_structural_change(),
                    )
                } else {
                    self.bind_unresolved(
                        reference.descriptor_kind(),
                        reference.carrier_detail(),
                        reference.requires_rebind_on_structural_change(),
                    )
                }
            }
            TreeReference::HostSensitive { carrier_id, detail } => {
                self.residuals.push(ResidualCarrier {
                    kind: ResidualCarrierKind::HostSensitive,
                    owner_node_id: self.owner_node_id,
                    carrier_id: carrier_id.clone(),
                    detail: detail.clone(),
                });
                "INFO(\"system\")".to_string()
            }
            TreeReference::DynamicPotential { carrier_id, detail } => {
                self.residuals.push(ResidualCarrier {
                    kind: ResidualCarrierKind::DynamicPotential,
                    owner_node_id: self.owner_node_id,
                    carrier_id: carrier_id.clone(),
                    detail: detail.clone(),
                });
                let topic = escape_excel_text(carrier_id);
                format!("RTD(\"TREECALC\",\"\",\"{topic}\")")
            }
            TreeReference::Unresolved { token: _ } => self.bind_unresolved(
                reference.descriptor_kind(),
                reference.carrier_detail(),
                reference.requires_rebind_on_structural_change(),
            ),
        }
    }

    fn bind_target(
        &mut self,
        target_node_id: TreeNodeId,
        kind: DependencyDescriptorKind,
        carrier_detail: String,
        requires_rebind_on_structural_change: bool,
    ) -> String {
        let token = format!(
            "TREE_REF_{}_{}",
            self.owner_node_id.0, self.next_reference_index
        );
        self.next_reference_index += 1;
        self.reference_bindings.push(SyntheticReferenceBinding {
            token: token.clone(),
            target_node_id,
            kind,
            carrier_detail,
            requires_rebind_on_structural_change,
        });
        token
    }

    fn bind_unresolved(
        &mut self,
        kind: DependencyDescriptorKind,
        carrier_detail: String,
        requires_rebind_on_structural_change: bool,
    ) -> String {
        let token = format!(
            "TREE_UNRESOLVED_{}_{}",
            self.owner_node_id.0, self.next_reference_index
        );
        self.next_reference_index += 1;
        self.unresolved_bindings.push(SyntheticUnresolvedBinding {
            token: token.clone(),
            kind,
            carrier_detail,
            requires_rebind_on_structural_change,
        });
        token
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

fn synthetic_cell_row(node_id: TreeNodeId) -> u32 {
    u32::try_from(node_id.0).unwrap_or(u32::MAX)
}

fn synthetic_cell_target(node_id: TreeNodeId) -> String {
    format!("A{}", synthetic_cell_row(node_id))
}

fn string_to_eval_value(value: &str) -> EvalValue {
    if let Ok(number) = value.parse::<f64>() {
        EvalValue::Number(number)
    } else if let Ok(logical) = value.parse::<bool>() {
        EvalValue::Logical(logical)
    } else {
        EvalValue::Text(ExcelText::from_interop_assignment(value))
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
    let kind = match residual.kind {
        ResidualCarrierKind::HostSensitive => "runtime_effect.host_sensitive_reference",
        ResidualCarrierKind::DynamicPotential => "runtime_effect.dynamic_reference",
    };
    RuntimeEffect {
        kind: kind.to_string(),
        detail: format!(
            "owner_node:{};carrier_id:{};detail:{}",
            residual.owner_node_id, residual.carrier_id, residual.detail
        ),
    }
}

#[cfg(test)]
mod tests {
    use crate::formula::{RelativeReferenceBase, TreeFormulaBinding};
    use crate::structural::{
        BindArtifactId, FormulaArtifactId, StructuralNode, StructuralNodeKind, StructuralSnapshotId,
    };

    use super::*;

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
                StructuralNode {
                    node_id: TreeNodeId(4),
                    kind: StructuralNodeKind::Calculation,
                    symbol: "C".to_string(),
                    parent_id: Some(TreeNodeId(1)),
                    child_ids: vec![],
                    formula_artifact_id: Some(FormulaArtifactId("formula:c".to_string())),
                    bind_artifact_id: Some(BindArtifactId("bind:c".to_string())),
                    constant_value: None,
                },
            ],
        )
        .unwrap()
    }

    #[test]
    fn local_treecalc_engine_publishes_local_formula_results() {
        let engine = LocalTreeCalcEngine;
        let run = engine
            .execute(LocalTreeCalcInput {
                structural_snapshot: snapshot(),
                formula_catalog: TreeFormulaCatalog::new([
                    TreeFormulaBinding {
                        owner_node_id: TreeNodeId(3),
                        formula_artifact_id: FormulaArtifactId("formula:b".to_string()),
                        bind_artifact_id: Some(BindArtifactId("bind:b".to_string())),
                        expression: TreeFormula::Binary {
                            op: FormulaBinaryOp::Add,
                            left: Box::new(TreeFormula::Reference(TreeReference::DirectNode {
                                target_node_id: TreeNodeId(2),
                            })),
                            right: Box::new(TreeFormula::Literal {
                                value: "3".to_string(),
                            }),
                        },
                    },
                    TreeFormulaBinding {
                        owner_node_id: TreeNodeId(4),
                        formula_artifact_id: FormulaArtifactId("formula:c".to_string()),
                        bind_artifact_id: Some(BindArtifactId("bind:c".to_string())),
                        expression: TreeFormula::FunctionCall {
                            function_name: "SUM".to_string(),
                            arguments: vec![
                                TreeFormula::Reference(TreeReference::RelativePath {
                                    base: RelativeReferenceBase::ParentNode,
                                    path_segments: vec!["A".to_string()],
                                }),
                                TreeFormula::Reference(TreeReference::DirectNode {
                                    target_node_id: TreeNodeId(3),
                                }),
                            ],
                            may_introduce_dynamic_dependencies: false,
                        },
                    },
                ]),
                seeded_published_values: BTreeMap::new(),
                candidate_result_id: "cand:local".to_string(),
                publication_id: "pub:local".to_string(),
                compatibility_basis: "snapshot:1".to_string(),
                artifact_token_basis: "snapshot:1".to_string(),
            })
            .unwrap();

        assert_eq!(run.result_state, LocalTreeCalcRunState::Published);
        assert_eq!(run.evaluation_order, vec![TreeNodeId(3), TreeNodeId(4)]);
        assert!(run.local_candidate.is_some());
        assert!(run.candidate_result.is_some());
        assert!(run.runtime_effects.is_empty());
        assert!(run.runtime_effect_overlays.is_empty());
        assert_eq!(run.published_values[&TreeNodeId(3)], "5");
        assert_eq!(run.published_values[&TreeNodeId(4)], "7");
        assert!(run.publication_bundle.is_some());
    }

    #[test]
    fn local_treecalc_engine_marks_verified_clean_when_seed_matches() {
        let engine = LocalTreeCalcEngine;
        let mut seeded = BTreeMap::new();
        seeded.insert(TreeNodeId(3), "5".to_string());

        let run = engine
            .execute(LocalTreeCalcInput {
                structural_snapshot: snapshot(),
                formula_catalog: TreeFormulaCatalog::new([TreeFormulaBinding {
                    owner_node_id: TreeNodeId(3),
                    formula_artifact_id: FormulaArtifactId("formula:b".to_string()),
                    bind_artifact_id: Some(BindArtifactId("bind:b".to_string())),
                    expression: TreeFormula::Binary {
                        op: FormulaBinaryOp::Add,
                        left: Box::new(TreeFormula::Reference(TreeReference::DirectNode {
                            target_node_id: TreeNodeId(2),
                        })),
                        right: Box::new(TreeFormula::Literal {
                            value: "3".to_string(),
                        }),
                    },
                }]),
                seeded_published_values: seeded,
                candidate_result_id: "cand:verified".to_string(),
                publication_id: "pub:verified".to_string(),
                compatibility_basis: "snapshot:1".to_string(),
                artifact_token_basis: "snapshot:1".to_string(),
            })
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
    }

    #[test]
    fn local_treecalc_engine_rejects_cycles_in_formula_family() {
        let engine = LocalTreeCalcEngine;
        let run = engine
            .execute(LocalTreeCalcInput {
                structural_snapshot: snapshot(),
                formula_catalog: TreeFormulaCatalog::new([
                    TreeFormulaBinding {
                        owner_node_id: TreeNodeId(3),
                        formula_artifact_id: FormulaArtifactId("formula:b".to_string()),
                        bind_artifact_id: None,
                        expression: TreeFormula::Reference(TreeReference::DirectNode {
                            target_node_id: TreeNodeId(4),
                        }),
                    },
                    TreeFormulaBinding {
                        owner_node_id: TreeNodeId(4),
                        formula_artifact_id: FormulaArtifactId("formula:c".to_string()),
                        bind_artifact_id: None,
                        expression: TreeFormula::Reference(TreeReference::DirectNode {
                            target_node_id: TreeNodeId(3),
                        }),
                    },
                ]),
                seeded_published_values: BTreeMap::new(),
                candidate_result_id: "cand:cycle".to_string(),
                publication_id: "pub:cycle".to_string(),
                compatibility_basis: "snapshot:1".to_string(),
                artifact_token_basis: "snapshot:1".to_string(),
            })
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
            .execute(LocalTreeCalcInput {
                structural_snapshot: snapshot(),
                formula_catalog: TreeFormulaCatalog::new([TreeFormulaBinding {
                    owner_node_id: TreeNodeId(3),
                    formula_artifact_id: FormulaArtifactId("formula:b".to_string()),
                    bind_artifact_id: Some(BindArtifactId("bind:b".to_string())),
                    expression: TreeFormula::Reference(TreeReference::HostSensitive {
                        carrier_id: "carrier:host".to_string(),
                        detail: "active_selection".to_string(),
                    }),
                }]),
                seeded_published_values: BTreeMap::new(),
                candidate_result_id: "cand:host".to_string(),
                publication_id: "pub:host".to_string(),
                compatibility_basis: "snapshot:1".to_string(),
                artifact_token_basis: "snapshot:1".to_string(),
            })
            .unwrap();

        assert_eq!(run.result_state, LocalTreeCalcRunState::Rejected);
        assert_eq!(run.runtime_effects.len(), 1);
        assert_eq!(run.runtime_effect_overlays.len(), 1);
        assert_eq!(
            run.runtime_effects[0].kind,
            "runtime_effect.host_sensitive_reference"
        );
        assert!(run.runtime_effects[0]
            .detail
            .contains("carrier_id:carrier:host"));
        assert_eq!(
            run.local_candidate
                .as_ref()
                .map(|candidate| candidate.runtime_effects.clone())
                .unwrap(),
            run.runtime_effects
        );
        assert_eq!(
            run.runtime_effect_overlays[0].key.overlay_kind,
            OverlayKind::DynamicDependency
        );
        assert!(run.runtime_effect_overlays[0]
            .detail
            .contains("runtime_effect.host_sensitive_reference"));
    }

    #[test]
    fn local_treecalc_engine_emits_runtime_effect_for_dynamic_reference() {
        let engine = LocalTreeCalcEngine;
        let run = engine
            .execute(LocalTreeCalcInput {
                structural_snapshot: snapshot(),
                formula_catalog: TreeFormulaCatalog::new([TreeFormulaBinding {
                    owner_node_id: TreeNodeId(3),
                    formula_artifact_id: FormulaArtifactId("formula:b".to_string()),
                    bind_artifact_id: Some(BindArtifactId("bind:b".to_string())),
                    expression: TreeFormula::Reference(TreeReference::DynamicPotential {
                        carrier_id: "carrier:dynamic".to_string(),
                        detail: "late_bound_projection".to_string(),
                    }),
                }]),
                seeded_published_values: BTreeMap::new(),
                candidate_result_id: "cand:dynamic".to_string(),
                publication_id: "pub:dynamic".to_string(),
                compatibility_basis: "snapshot:1".to_string(),
                artifact_token_basis: "snapshot:1".to_string(),
            })
            .unwrap();

        assert_eq!(run.result_state, LocalTreeCalcRunState::Rejected);
        assert_eq!(run.runtime_effects.len(), 1);
        assert_eq!(run.runtime_effect_overlays.len(), 1);
        assert_eq!(
            run.runtime_effects[0].kind,
            "runtime_effect.dynamic_reference"
        );
        assert!(run.runtime_effects[0]
            .detail
            .contains("carrier_id:carrier:dynamic"));
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
}
