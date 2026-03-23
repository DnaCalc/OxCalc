#![forbid(unsafe_code)]

//! Local sequential TreeCalc runtime facade.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use thiserror::Error;

use crate::coordinator::{
    AcceptedCandidateResult, CoordinatorError, PublicationBundle, RejectDetail, RejectKind,
    RuntimeEffect, TreeCalcCoordinator,
};
use crate::dependency::{
    DependencyGraph, InvalidationClosure, InvalidationReasonKind, InvalidationSeed,
};
use crate::formula::{FormulaBinaryOp, TreeFormula, TreeFormulaCatalog, TreeReference};
use crate::recalc::{
    NodeCalcState, OverlayEntry, OverlayKey, OverlayKind, RecalcError, Stage1RecalcTracker,
};
use crate::structural::{StructuralSnapshot, TreeNodeId};

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
}

impl From<LocalTreeCalcError> for LocalFormulaEvaluationFailure {
    fn from(error: LocalTreeCalcError) -> Self {
        Self {
            error,
            runtime_effects: Vec::new(),
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
        let dependency_graph = DependencyGraph::build(
            &input.structural_snapshot,
            &input
                .formula_catalog
                .to_dependency_descriptors(&input.structural_snapshot),
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
            let binding = input
                .formula_catalog
                .try_get_binding(*node_id)
                .ok_or(LocalTreeCalcError::MissingFormulaBinding { node_id: *node_id })?;
            let computed_value = match evaluate_formula(
                &input.structural_snapshot,
                *node_id,
                &binding.expression,
                &working_values,
            ) {
                Ok(value) => value,
                Err(failure) => {
                    runtime_effects.extend(failure.runtime_effects.clone());
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
        | LocalTreeCalcError::DivisionByZero => RejectKind::HostInjectedFailure,
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

fn evaluate_formula(
    snapshot: &StructuralSnapshot,
    owner_node_id: TreeNodeId,
    formula: &TreeFormula,
    values: &BTreeMap<TreeNodeId, String>,
) -> Result<String, LocalFormulaEvaluationFailure> {
    match formula {
        TreeFormula::Literal { value } => Ok(value.clone()),
        TreeFormula::Reference(reference) => {
            let target_node_id = match reference {
                TreeReference::HostSensitive { carrier_id, detail } => {
                    return Err(LocalFormulaEvaluationFailure {
                        error: LocalTreeCalcError::HostSensitiveReference {
                            owner_node_id,
                            detail: detail.clone(),
                        },
                        runtime_effects: vec![RuntimeEffect {
                            kind: "runtime_effect.host_sensitive_reference".to_string(),
                            detail: format!(
                                "owner_node:{owner_node_id};carrier_id:{carrier_id};detail:{detail}"
                            ),
                        }],
                    });
                }
                TreeReference::DynamicPotential { carrier_id, detail } => {
                    return Err(LocalFormulaEvaluationFailure {
                        error: LocalTreeCalcError::DynamicReference {
                            owner_node_id,
                            detail: detail.clone(),
                        },
                        runtime_effects: vec![RuntimeEffect {
                            kind: "runtime_effect.dynamic_reference".to_string(),
                            detail: format!(
                                "owner_node:{owner_node_id};carrier_id:{carrier_id};detail:{detail}"
                            ),
                        }],
                    });
                }
                TreeReference::Unresolved { token } => {
                    return Err(LocalTreeCalcError::UnresolvedReference {
                        owner_node_id,
                        detail: token.clone(),
                    }
                    .into());
                }
                _ => reference
                    .resolve_target(snapshot, owner_node_id)
                    .ok_or_else(|| {
                        LocalFormulaEvaluationFailure::from(
                            LocalTreeCalcError::UnresolvedReference {
                                owner_node_id,
                                detail: reference.carrier_detail(),
                            },
                        )
                    })?,
            };

            values
                .get(&target_node_id)
                .cloned()
                .ok_or(LocalFormulaEvaluationFailure::from(
                    LocalTreeCalcError::MissingReferencedValue {
                        node_id: target_node_id,
                    },
                ))
        }
        TreeFormula::Binary { op, left, right } => {
            let left_value = evaluate_formula(snapshot, owner_node_id, left, values)?;
            let right_value = evaluate_formula(snapshot, owner_node_id, right, values)?;
            let left_number = parse_i64(owner_node_id, &left_value)
                .map_err(LocalFormulaEvaluationFailure::from)?;
            let right_number = parse_i64(owner_node_id, &right_value)
                .map_err(LocalFormulaEvaluationFailure::from)?;
            let result = match op {
                FormulaBinaryOp::Add => left_number + right_number,
                FormulaBinaryOp::Subtract => left_number - right_number,
                FormulaBinaryOp::Multiply => left_number * right_number,
                FormulaBinaryOp::Divide => {
                    if right_number == 0 {
                        return Err(LocalFormulaEvaluationFailure::from(
                            LocalTreeCalcError::DivisionByZero,
                        ));
                    }
                    left_number / right_number
                }
            };
            Ok(result.to_string())
        }
        TreeFormula::FunctionCall {
            function_name,
            arguments,
            ..
        } => evaluate_function(snapshot, owner_node_id, function_name, arguments, values),
    }
}

fn evaluate_function(
    snapshot: &StructuralSnapshot,
    owner_node_id: TreeNodeId,
    function_name: &str,
    arguments: &[TreeFormula],
    values: &BTreeMap<TreeNodeId, String>,
) -> Result<String, LocalFormulaEvaluationFailure> {
    let upper_name = function_name.to_ascii_uppercase();
    match upper_name.as_str() {
        "SUM" => {
            let mut total = 0i64;
            for argument in arguments {
                let value = evaluate_formula(snapshot, owner_node_id, argument, values)?;
                total += parse_i64(owner_node_id, &value)
                    .map_err(LocalFormulaEvaluationFailure::from)?;
            }
            Ok(total.to_string())
        }
        "IF" => {
            if arguments.len() != 3 {
                return Err(LocalTreeCalcError::UnsupportedFunction {
                    function_name: function_name.to_string(),
                }
                .into());
            }
            let condition = evaluate_formula(snapshot, owner_node_id, &arguments[0], values)?;
            let branch = if parse_i64(owner_node_id, &condition)
                .map_err(LocalFormulaEvaluationFailure::from)?
                != 0
            {
                &arguments[1]
            } else {
                &arguments[2]
            };
            evaluate_formula(snapshot, owner_node_id, branch, values)
        }
        "MAX" => {
            let mut best = None::<i64>;
            for argument in arguments {
                let value = evaluate_formula(snapshot, owner_node_id, argument, values)?;
                let number = parse_i64(owner_node_id, &value)
                    .map_err(LocalFormulaEvaluationFailure::from)?;
                best = Some(best.map_or(number, |current| current.max(number)));
            }
            Ok(best.unwrap_or(0).to_string())
        }
        _ => Err(LocalTreeCalcError::UnsupportedFunction {
            function_name: function_name.to_string(),
        }
        .into()),
    }
}

fn parse_i64(owner_node_id: TreeNodeId, value: &str) -> Result<i64, LocalTreeCalcError> {
    value
        .parse::<i64>()
        .map_err(|_| LocalTreeCalcError::UnsupportedNumericValue {
            node_id: owner_node_id,
            value: value.to_string(),
        })
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
            OverlayKind::DynamicDependency
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
}
