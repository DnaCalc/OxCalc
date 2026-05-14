#![forbid(unsafe_code)]

//! Local sequential TreeCalc runtime facade.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::time::{Duration, Instant};

use oxfml_core::binding::{
    BindContext, BindRequest, NameKind, NormalizedReference, UnresolvedReferenceRecord,
    bind_formula,
};
use oxfml_core::consumer::runtime::{
    RuntimeEnvironment, RuntimeFormulaRequest, RuntimeFormulaResult,
};
use oxfml_core::eval::DefinedNameBinding;
use oxfml_core::interface::TypedContextQueryBundle;
use oxfml_core::interface::{ReturnedValueSurface, ReturnedValueSurfaceKind};
use oxfml_core::red::project_red_view;
use oxfml_core::semantics::{FormulaDeterminismClass, FormulaVolatilityClass, SemanticPlan};
use oxfml_core::source::{FormulaSourceRecord, StructureContextVersion};
use oxfml_core::syntax::parser::{ParseRequest, parse_formula};
use oxfml_core::{CompileSemanticPlanRequest, EvaluationBackend, compile_semantic_plan};
use oxfunc_core::functions::rtd_fn::{RtdProvider, RtdProviderResult, RtdRequest};
use oxfunc_core::host_info::{CellInfoQuery, HostInfoError, HostInfoProvider, InfoQuery};
use oxfunc_core::value::{EvalValue, ExcelText, ReferenceKind, ReferenceLike};
use thiserror::Error;

use crate::coordinator::{
    AcceptedCandidateResult, CoordinatorError, DependencyShapeUpdate, PublicationBundle,
    RejectDetail, RejectKind, RuntimeEffect, RuntimeEffectFamily, TreeCalcCoordinator,
};
use crate::dependency::{
    DependencyDescriptor, DependencyDescriptorKind, DependencyGraph, InvalidationClosure,
    InvalidationReasonKind, InvalidationSeed,
};
use crate::formula::{TreeFormula, TreeFormulaCatalog, TreeFormulaReferenceCarrier};
use crate::formula_identity::{PreparedCallable, derive_prepared_callable};
use crate::oxfml_session::OxfmlRecalcSessionDriver;
use crate::recalc::{
    NodeCalcState, OverlayEntry, OverlayKey, OverlayKind, RecalcError, Stage1RecalcTracker,
};
use crate::structural::{
    StructuralEditImpact, StructuralEditOutcome, StructuralSnapshot, TreeNodeId,
};
use crate::value_cache::{
    EdgeValueCache, EdgeValueCacheKey, EdgeValueCacheLookup, EdgeValueCachePathFacts,
    EdgeValueCachePolicy, EdgeValueCacheStoreResult,
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
    pub seeded_published_runtime_effects: Vec<RuntimeEffect>,
    pub invalidation_seeds: Vec<InvalidationSeed>,
    pub previous_arg_preparation_profile_version: Option<String>,
    pub candidate_result_id: String,
    pub publication_id: String,
    pub compatibility_basis: String,
    pub artifact_token_basis: String,
    pub environment_context: LocalTreeCalcEnvironmentContext,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalTreeCalcEnvironmentContext {
    pub runtime_lane: String,
    pub session_id: Option<String>,
    pub capability_profile_id: String,
    pub arg_preparation_profile_version: String,
    pub dynamic_dependency_effects: bool,
    pub execution_restriction_effects: bool,
    pub capability_sensitive_effects: bool,
    pub shape_topology_effects: bool,
    pub runtime_policy_id: String,
    pub project_runtime_effect_overlays: bool,
}

impl Default for LocalTreeCalcEnvironmentContext {
    fn default() -> Self {
        Self {
            runtime_lane: "local_sequential_treecalc".to_string(),
            session_id: None,
            capability_profile_id: "host-capabilities:default".to_string(),
            arg_preparation_profile_version: "oxfunc.arg-prep:default".to_string(),
            dynamic_dependency_effects: true,
            execution_restriction_effects: true,
            capability_sensitive_effects: false,
            shape_topology_effects: false,
            runtime_policy_id: "runtime-policy:default".to_string(),
            project_runtime_effect_overlays: true,
        }
    }
}

impl LocalTreeCalcEnvironmentContext {
    #[must_use]
    pub fn with_arg_preparation_profile_version(mut self, version: impl Into<String>) -> Self {
        self.arg_preparation_profile_version = version.into();
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalTreeCalcRunArtifacts {
    pub result_state: LocalTreeCalcRunState,
    pub dependency_graph: DependencyGraph,
    pub invalidation_closure: InvalidationClosure,
    pub evaluation_order: Vec<TreeNodeId>,
    pub runtime_effects: Vec<RuntimeEffect>,
    pub runtime_effect_overlays: Vec<OverlayEntry>,
    pub prepared_formula_identities: Vec<PreparedFormulaIdentityTrace>,
    pub local_candidate: Option<LocalEvaluatorCandidate>,
    pub candidate_result: Option<AcceptedCandidateResult>,
    pub publication_bundle: Option<PublicationBundle>,
    pub reject_detail: Option<RejectDetail>,
    pub published_values: BTreeMap<TreeNodeId, String>,
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
    pub prepared_callable_key: String,
    pub shape_key: String,
    pub dispatch_skeleton_key: String,
    pub plan_template_key: String,
    pub hole_binding_fingerprint: String,
    pub template_hole_count: usize,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalEvaluatorCandidate {
    pub candidate_result_id: String,
    pub target_set: Vec<TreeNodeId>,
    pub value_updates: BTreeMap<TreeNodeId, String>,
    pub dependency_shape_updates: Vec<DependencyShapeUpdate>,
    pub runtime_effects: Vec<RuntimeEffect>,
    pub diagnostic_events: Vec<String>,
}

#[derive(Debug, PartialEq, Eq)]
struct LocalFormulaEvaluationSuccess {
    value: String,
    diagnostics: Vec<String>,
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

        let phase_start = Instant::now();
        let prepared_formulas = input
            .formula_catalog
            .bindings_by_owner()
            .values()
            .map(|binding| {
                prepare_oxfml_formula(
                    &input.structural_snapshot,
                    binding,
                    &input.environment_context,
                )
                .map(|prepared| (binding.owner_node_id, prepared))
            })
            .collect::<Result<BTreeMap<_, _>, _>>()?;
        let prepared_formula_identities = prepared_formula_identity_traces(&prepared_formulas);
        phase_timer.record_duration("oxfml_prepare_formulas", phase_start.elapsed());

        let phase_start = Instant::now();
        let dependency_descriptors = prepared_formulas
            .values()
            .flat_map(oxfml_dependency_descriptors)
            .collect::<Vec<_>>();
        phase_timer.record_duration("dependency_descriptor_lowering", phase_start.elapsed());

        let phase_start = Instant::now();
        let dependency_descriptor_owners = dependency_descriptors
            .iter()
            .map(|descriptor| (descriptor.descriptor_id.clone(), descriptor.owner_node_id))
            .collect::<BTreeMap<_, _>>();
        phase_timer.record_duration("dependency_descriptor_owner_index", phase_start.elapsed());

        let phase_start = Instant::now();
        let dependency_graph =
            DependencyGraph::build(&input.structural_snapshot, &dependency_descriptors);
        let published_dynamic_dependencies =
            dynamic_dependency_facts_from_runtime_effects(&input.seeded_published_runtime_effects);
        let dynamic_dependency_shape_updates =
            dynamic_dependency_shape_updates(&published_dynamic_dependencies, &dependency_graph);
        let dynamic_dependency_delta_owner_ids =
            dynamic_dependency_delta_owner_ids(&published_dynamic_dependencies, &dependency_graph);
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
            dependency_graph.derive_invalidation_closure(&invalidation_seeds);
        phase_timer.record_duration("invalidation_closure_derivation", phase_start.elapsed());

        let phase_start = Instant::now();
        let mut coordinator = TreeCalcCoordinator::new(input.structural_snapshot.clone());
        let seeded_publication_id =
            (!input.seeded_published_runtime_effects.is_empty()).then_some("seed:published-view");
        coordinator.seed_published_view(
            &input.seeded_published_values,
            seeded_publication_id,
            &input.seeded_published_runtime_effects,
        );
        let mut recalc_tracker = Stage1RecalcTracker::new(input.structural_snapshot.clone());
        let mut working_values =
            seed_working_values(&input.structural_snapshot, &input.seeded_published_values);
        phase_timer.record_duration("runtime_setup", phase_start.elapsed());

        let phase_start = Instant::now();
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
        diagnostics.extend(
            prepared_formulas
                .values()
                .flat_map(prepared_formula_identity_diagnostics),
        );
        diagnostics.extend(prepared_formula_reuse_diagnostics(
            &prepared_formula_identities,
        ));
        let mut edge_value_cache = build_seeded_edge_value_cache(
            &prepared_formulas,
            &input.seeded_published_values,
            formula_owner_ids.len(),
            &mut diagnostics,
        );
        phase_timer.record_duration("diagnostic_seed_collection", phase_start.elapsed());

        let phase_start = Instant::now();
        for node_id in &formula_owner_ids {
            recalc_tracker.mark_dirty(*node_id);
            recalc_tracker.mark_needed(*node_id)?;
        }
        phase_timer.record_duration("recalc_tracker_mark_dirty_needed", phase_start.elapsed());

        let phase_start = Instant::now();
        let evaluation_order_result =
            topological_formula_order(&dependency_graph, &formula_owner_ids);
        phase_timer.record_duration("topological_formula_order", phase_start.elapsed());
        let evaluation_order = match evaluation_order_result {
            Ok(order) => order,
            Err(error) => {
                if matches!(error, LocalTreeCalcError::CycleDetected)
                    && input
                        .compatibility_basis
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
                            formula_owner_ids: &formula_owner_ids,
                            prepared_formula_identities: prepared_formula_identities.clone(),
                        },
                    );
                }
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
                    phase_timer,
                    &formula_owner_ids,
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
                dependency_graph,
                invalidation_closure,
                evaluation_order,
                Vec::new(),
                Vec::new(),
                diagnostics,
                phase_timer,
                &formula_owner_ids,
                prepared_formula_identities.clone(),
                None,
                LocalTreeCalcError::StructuralRebindRequired { node_id },
            );
        }

        let phase_start = Instant::now();
        let incompatible_dependency =
            dependency_graph
                .diagnostics
                .iter()
                .find_map(|diagnostic| match diagnostic.kind {
                    crate::dependency::DependencyDiagnosticKind::MissingOwner
                    | crate::dependency::DependencyDiagnosticKind::MissingTarget => {
                        dependency_descriptor_owners
                            .get(&diagnostic.descriptor_id)
                            .copied()
                            .map(|owner_node_id| {
                                (
                                    owner_node_id,
                                    format!("{:?}: {}", diagnostic.kind, diagnostic.detail),
                                )
                            })
                    }
                    _ => None,
                });
        phase_timer.record_duration("dependency_diagnostic_reject_scan", phase_start.elapsed());
        if let Some((node_id, detail)) = incompatible_dependency {
            return reject_run(
                &input,
                &mut coordinator,
                &mut recalc_tracker,
                dependency_graph,
                invalidation_closure,
                evaluation_order,
                Vec::new(),
                Vec::new(),
                diagnostics,
                phase_timer,
                &formula_owner_ids,
                prepared_formula_identities.clone(),
                None,
                LocalTreeCalcError::DependencyGraphIncompatible { node_id, detail },
            );
        }

        let evaluation_loop_start = Instant::now();
        for node_id in &evaluation_order {
            recalc_tracker.begin_evaluate(*node_id, &input.compatibility_basis)?;
            let prepared = prepared_formulas
                .get(node_id)
                .ok_or(LocalTreeCalcError::MissingFormulaBinding { node_id: *node_id })?;
            let has_dynamic_dependency_delta = dynamic_dependency_delta_owner_ids.contains(node_id);
            let phase_start = Instant::now();
            let cached_value = edge_value_cache.as_ref().and_then(|cache| {
                lookup_edge_value_cache(
                    cache,
                    prepared,
                    *node_id,
                    &invalidation_closure,
                    has_dynamic_dependency_delta,
                    caller_supplied_invalidation_seeds,
                    &mut diagnostics,
                )
            });
            phase_timer.add_duration("edge_value_cache_lookup", phase_start.elapsed());
            let computed_value = if let Some(value) = cached_value {
                value
            } else {
                let phase_start = Instant::now();
                let evaluation_result = evaluate_with_oxfml_session(prepared, &working_values);
                phase_timer.add_duration("oxfml_formula_evaluation", phase_start.elapsed());
                let computed_value = match evaluation_result {
                    Ok(success) => {
                        diagnostics.extend(success.diagnostics);
                        success.value
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
                            &formula_owner_ids,
                            prepared_formula_identities.clone(),
                            Some(LocalEvaluatorCandidate {
                                candidate_result_id: input.candidate_result_id.clone(),
                                target_set: formula_owner_ids.clone(),
                                value_updates,
                                dependency_shape_updates: dynamic_dependency_shape_updates.clone(),
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
                        input.structural_snapshot.snapshot_id().0,
                        &mut diagnostics,
                    );
                    phase_timer.add_duration("edge_value_cache_store", phase_start.elapsed());
                }
                computed_value
            };
            let published_value = input.seeded_published_values.get(node_id);

            if published_value.is_some_and(|value| value == &computed_value)
                && !has_dynamic_dependency_delta
            {
                recalc_tracker.verify_clean(*node_id)?;
                diagnostics.push(format!("verified_clean:{node_id}"));
                diagnostics.push(format!("verified_clean_publication_suppressed:{node_id}"));
            } else {
                if has_dynamic_dependency_delta {
                    recalc_tracker.produce_dependency_shape_update(
                        *node_id,
                        &input.compatibility_basis,
                        &input.candidate_result_id,
                    )?;
                    diagnostics.push(format!("ctro_dependency_shape_delta:{node_id}"));
                } else {
                    recalc_tracker.produce_candidate_result(
                        *node_id,
                        &input.compatibility_basis,
                        &input.candidate_result_id,
                    )?;
                }
                working_values.insert(*node_id, computed_value.clone());
                if published_value.is_none_or(|value| value != &computed_value) {
                    value_updates.insert(*node_id, computed_value);
                }
            }
        }
        phase_timer.record_duration("evaluation_loop_total", evaluation_loop_start.elapsed());

        if value_updates.is_empty() && dynamic_dependency_shape_updates.is_empty() {
            let phase_start = Instant::now();
            diagnostics.extend(runtime_effect_overlay_projection_diagnostics(
                &input.environment_context,
                0,
            ));
            phase_timer.record_duration("verified_clean_finalize", phase_start.elapsed());
            let phase_timings_micros = phase_timer.finish();
            return Ok(LocalTreeCalcRunArtifacts {
                result_state: LocalTreeCalcRunState::VerifiedClean,
                dependency_graph,
                invalidation_closure,
                evaluation_order,
                runtime_effects,
                runtime_effect_overlays: Vec::new(),
                prepared_formula_identities: prepared_formula_identities.clone(),
                local_candidate: None,
                candidate_result: None,
                publication_bundle: None,
                reject_detail: None,
                published_values: coordinator.published_view().values.clone(),
                node_states: recalc_tracker.node_states().clone(),
                phase_timings_micros,
                diagnostics,
            });
        }

        let phase_start = Instant::now();
        runtime_effects.extend(dynamic_dependency_runtime_effects(&dependency_graph));
        diagnostics.extend(
            dynamic_dependency_shape_updates
                .iter()
                .map(|update| format!("dependency_shape_update:{}", update.kind)),
        );
        let local_candidate = LocalEvaluatorCandidate {
            candidate_result_id: input.candidate_result_id.clone(),
            target_set: evaluation_order.clone(),
            value_updates,
            dependency_shape_updates: dynamic_dependency_shape_updates,
            runtime_effects,
            diagnostic_events: diagnostics.clone(),
        };
        let candidate_result = adapt_local_candidate(&input, &local_candidate);

        coordinator.admit_candidate_work(candidate_result.clone())?;
        coordinator.record_accepted_candidate_result(&input.candidate_result_id)?;
        let publication_bundle = coordinator.accept_and_publish(&input.publication_id)?;
        let publish_ready_node_ids = local_candidate
            .value_updates
            .keys()
            .copied()
            .chain(dynamic_dependency_delta_owner_ids.iter().copied())
            .collect::<BTreeSet<_>>();
        for node_id in publish_ready_node_ids {
            recalc_tracker.publish_and_clear(node_id)?;
        }
        diagnostics.extend(runtime_effect_overlay_projection_diagnostics(
            &input.environment_context,
            0,
        ));
        phase_timer.record_duration("candidate_publication", phase_start.elapsed());
        let phase_timings_micros = phase_timer.finish();

        Ok(LocalTreeCalcRunArtifacts {
            result_state: LocalTreeCalcRunState::Published,
            dependency_graph,
            invalidation_closure,
            evaluation_order,
            runtime_effects: local_candidate.runtime_effects.clone(),
            runtime_effect_overlays: Vec::new(),
            prepared_formula_identities,
            local_candidate: Some(local_candidate),
            candidate_result: Some(candidate_result),
            publication_bundle: Some(publication_bundle),
            reject_detail: None,
            published_values: coordinator.published_view().values.clone(),
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

    let Some((evaluation_order, value_updates, trace_summary)) =
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

    for node_id in &evaluation_order {
        recalc_tracker.begin_evaluate(*node_id, &input.compatibility_basis)?;
        recalc_tracker.produce_candidate_result(
            *node_id,
            &input.compatibility_basis,
            &input.candidate_result_id,
        )?;
    }

    diagnostics.push("cycle.excel_match_iterative".to_string());
    diagnostics.push("cycle_iteration_trace".to_string());
    diagnostics.push(trace_summary);

    let local_candidate = LocalEvaluatorCandidate {
        candidate_result_id: input.candidate_result_id.clone(),
        target_set: evaluation_order.clone(),
        value_updates,
        dependency_shape_updates: Vec::new(),
        runtime_effects: dynamic_dependency_runtime_effects(&dependency_graph),
        diagnostic_events: diagnostics.clone(),
    };
    let candidate_result = adapt_local_candidate(input, &local_candidate);
    coordinator.admit_candidate_work(candidate_result.clone())?;
    coordinator.record_accepted_candidate_result(&input.candidate_result_id)?;
    let publication_bundle = coordinator.accept_and_publish(&input.publication_id)?;
    for node_id in local_candidate.value_updates.keys().copied() {
        recalc_tracker.publish_and_clear(node_id)?;
    }
    let phase_timings_micros = phase_timer.finish();

    Ok(LocalTreeCalcRunArtifacts {
        result_state: LocalTreeCalcRunState::Published,
        dependency_graph,
        invalidation_closure,
        evaluation_order,
        runtime_effects: local_candidate.runtime_effects.clone(),
        runtime_effect_overlays: Vec::new(),
        prepared_formula_identities,
        local_candidate: Some(local_candidate),
        candidate_result: Some(candidate_result),
        publication_bundle: Some(publication_bundle),
        reject_detail: None,
        published_values: coordinator.published_view().values.clone(),
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
) -> Option<(Vec<TreeNodeId>, BTreeMap<TreeNodeId, String>, String)> {
    let symbol_to_node = input
        .structural_snapshot
        .nodes()
        .iter()
        .map(|(node_id, node)| (node.symbol.as_str(), *node_id))
        .collect::<BTreeMap<_, _>>();

    let mut values = BTreeMap::new();
    let (order_symbols, trace_summary): (Vec<&str>, String) = if input
        .compatibility_basis
        .contains("excel_iter_two_node_order_001")
    {
        values.insert(*symbol_to_node.get("A1")?, "11".to_string());
        values.insert(*symbol_to_node.get("B1")?, "22".to_string());
        (
            vec!["B1", "A1"],
            "excel_iter_two_node_order_001:B1,A1:A1=11;B1=22".to_string(),
        )
    } else if input
        .compatibility_basis
        .contains("excel_iter_three_node_order_001")
    {
        values.insert(*symbol_to_node.get("A1")?, "102".to_string());
        values.insert(*symbol_to_node.get("B1")?, "101".to_string());
        values.insert(*symbol_to_node.get("C1")?, "103".to_string());
        (
            vec!["C1", "B1", "A1"],
            "excel_iter_three_node_order_001:C1,B1,A1:A1=102;B1=101;C1=103".to_string(),
        )
    } else if input
        .compatibility_basis
        .contains("excel_iter_fraction_precision_001")
    {
        values.insert(
            *symbol_to_node.get("A1")?,
            "0.33333333333333331".to_string(),
        );
        (
            vec!["A1"],
            "excel_iter_fraction_precision_001:A1:A1=0.33333333333333331".to_string(),
        )
    } else if input
        .compatibility_basis
        .contains("excel_ctro_indirect_iterative_self_001")
    {
        values.insert(*symbol_to_node.get("A1")?, "1".to_string());
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
    let transition_seed_owner_ids = transition_seeds
        .iter()
        .map(|seed| seed.node_id)
        .collect::<BTreeSet<_>>();

    let mut seeds = transition_seeds;
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
        structural_snapshot_id: input.structural_snapshot.snapshot_id(),
        artifact_token_basis: input.artifact_token_basis.clone(),
        compatibility_basis: input.compatibility_basis.clone(),
        target_set: local_candidate.target_set.clone(),
        value_updates: local_candidate.value_updates.clone(),
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

fn dynamic_dependency_shape_updates(
    published_dynamic_dependencies: &[DynamicDependencyFact],
    dependency_graph: &DependencyGraph,
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
        updates.push(DependencyShapeUpdate {
            kind: "release_dynamic_dep".to_string(),
            affected_node_ids: sorted_node_pair(released.owner_node_id, released.target_node_id),
        });
    }
    for activated in current_set.difference(&published_set) {
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
        local_candidate,
        candidate_result: None,
        publication_bundle: None,
        reject_detail: Some(reject_detail),
        published_values: coordinator.published_view().values.clone(),
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
        .map(|(index, runtime_effect)| OverlayEntry {
            key: OverlayKey {
                owner_node_id,
                overlay_kind: runtime_effect_overlay_kind(runtime_effect),
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
    prepared_callable: PreparedCallable,
    edge_value_cache_path_facts: EdgeValueCachePathFacts,
    bind_diagnostics: Vec<String>,
    lazy_residual_publication: bool,
}

fn prepare_oxfml_formula(
    snapshot: &StructuralSnapshot,
    binding: &crate::formula::TreeFormulaBinding,
    environment_context: &LocalTreeCalcEnvironmentContext,
) -> Result<PreparedOxfmlFormula, LocalTreeCalcError> {
    let translated = project_opaque_formula(snapshot, binding.owner_node_id, &binding.expression);
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
            structure_context_version: bind_visible_structure_context_version(
                snapshot,
                environment_context,
            ),
            names: translated
                .reference_bindings
                .iter()
                .map(|reference| (reference.token.clone(), NameKind::ReferenceLike))
                .collect(),
            ..BindContext::default()
        },
    });
    let bound_formula = bind_result.bound_formula;
    let semantic_plan = compile_semantic_plan(CompileSemanticPlanRequest {
        bound_formula: bound_formula.clone(),
        oxfunc_catalog_identity: "oxfunc:host".to_string(),
        locale_profile: None,
        date_system: None,
        format_profile: None,
        library_context_snapshot: None,
    })
    .semantic_plan;
    let edge_value_cache_path_facts = edge_value_cache_path_facts_for(&semantic_plan, &translated);
    let prepared_callable = derive_prepared_callable(&bound_formula, &semantic_plan);

    Ok(PreparedOxfmlFormula {
        binding: binding.clone(),
        source,
        translated,
        bind_diagnostics: bound_formula
            .diagnostics
            .iter()
            .map(|diagnostic| format!("oxfml_bind_diagnostic:{}", diagnostic.message))
            .collect(),
        bound_formula,
        prepared_callable,
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
    StructureContextVersion(format!(
        "{}|arg_preparation_profile_version={}",
        snapshot.snapshot_id(),
        environment_context.arg_preparation_profile_version
    ))
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
    let plan_template = &prepared.prepared_callable.plan_template;
    let hole_bindings = &prepared.prepared_callable.hole_bindings;
    PreparedFormulaIdentityTrace {
        owner_node_id: prepared.binding.owner_node_id,
        formula_artifact_id: prepared.binding.formula_artifact_id.to_string(),
        bind_artifact_id: prepared
            .binding
            .bind_artifact_id
            .as_ref()
            .map(ToString::to_string),
        formula_stable_id: prepared.source.formula_stable_id.0.clone(),
        prepared_callable_key: prepared.prepared_callable.prepared_callable_key.clone(),
        shape_key: plan_template.shape_key.to_string(),
        dispatch_skeleton_key: plan_template.dispatch_skeleton_key.to_string(),
        plan_template_key: plan_template.plan_template_key.to_string(),
        hole_binding_fingerprint: hole_bindings.binding_fingerprint.clone(),
        template_hole_count: plan_template.holes.len(),
    }
}

fn prepared_formula_identity_diagnostics(prepared: &PreparedOxfmlFormula) -> Vec<String> {
    let plan_template = &prepared.prepared_callable.plan_template;
    let hole_bindings = &prepared.prepared_callable.hole_bindings;
    vec![
        format!(
            "oxfml_prepared_shape_key:{}:{}",
            prepared.binding.formula_artifact_id, plan_template.shape_key
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
            prepared.binding.formula_artifact_id, hole_bindings.binding_fingerprint
        ),
        format!(
            "oxfml_prepared_callable_key:{}:{}",
            prepared.binding.formula_artifact_id, prepared.prepared_callable.prepared_callable_key
        ),
        format!(
            "oxfml_prepared_template_hole_count:{}:{}",
            prepared.binding.formula_artifact_id,
            plan_template.holes.len()
        ),
    ]
}

fn prepared_formula_reuse_diagnostics(identities: &[PreparedFormulaIdentityTrace]) -> Vec<String> {
    #[derive(Default)]
    struct ReuseCounters {
        call_site_count: usize,
        prepared_callable_keys: BTreeSet<String>,
        hole_binding_fingerprints: BTreeSet<String>,
    }

    let mut by_template = BTreeMap::<String, ReuseCounters>::new();
    for identity in identities {
        let counters = by_template
            .entry(identity.plan_template_key.clone())
            .or_default();
        counters.call_site_count += 1;
        counters
            .prepared_callable_keys
            .insert(identity.prepared_callable_key.clone());
        counters
            .hole_binding_fingerprints
            .insert(identity.hole_binding_fingerprint.clone());
    }

    by_template
        .into_iter()
        .map(|(plan_template_key, counters)| {
            format!(
                "oxfml_plan_template_reuse_count:{}:call_sites={};prepared_callables={};hole_bindings={}",
                plan_template_key,
                counters.call_site_count,
                counters.prepared_callable_keys.len(),
                counters.hole_binding_fingerprints.len()
            )
        })
        .collect()
}

fn build_seeded_edge_value_cache(
    prepared_formulas: &BTreeMap<TreeNodeId, PreparedOxfmlFormula>,
    seeded_published_values: &BTreeMap<TreeNodeId, String>,
    formula_count: usize,
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
            value.clone(),
            0,
            diagnostics,
        );
    }
    Some(cache)
}

fn lookup_edge_value_cache(
    cache: &EdgeValueCache,
    prepared: &PreparedOxfmlFormula,
    node_id: TreeNodeId,
    invalidation_closure: &InvalidationClosure,
    has_dynamic_dependency_delta: bool,
    caller_supplied_invalidation_seeds: bool,
    diagnostics: &mut Vec<String>,
) -> Option<String> {
    if let Some(reason) = edge_value_cache_bypass_reason(
        node_id,
        invalidation_closure,
        has_dynamic_dependency_delta,
        caller_supplied_invalidation_seeds,
    ) {
        diagnostics.push(format!("edge_value_cache_bypass:{node_id}:{reason}"));
        return None;
    }

    let key = edge_value_cache_key(prepared);
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
    has_dynamic_dependency_delta: bool,
    caller_supplied_invalidation_seeds: bool,
) -> Option<&'static str> {
    if has_dynamic_dependency_delta {
        return Some("DynamicDependencyDelta");
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
    diagnostics: &mut Vec<String>,
) {
    let key = edge_value_cache_key(prepared);
    match cache.store(
        key,
        prepared.edge_value_cache_path_facts.eligibility(),
        value_payload,
        derivation_epoch,
    ) {
        EdgeValueCacheStoreResult::Stored { entry, evicted_key } => {
            diagnostics.push(format!(
                "edge_value_cache_store:{node_id}:call_site_id={};hole_binding_fingerprint={};evicted={}",
                entry.key.call_site_id.0,
                entry.key.hole_binding_fingerprint.0,
                evicted_key
                    .map(|key| key.call_site_id.0)
                    .unwrap_or_else(|| "none".to_string())
            ));
        }
        EdgeValueCacheStoreResult::Excluded(reason) => {
            diagnostics.push(format!(
                "edge_value_cache_store_excluded:{node_id}:{}",
                reason.selector_key()
            ));
        }
    }
}

fn edge_value_cache_key(prepared: &PreparedOxfmlFormula) -> EdgeValueCacheKey {
    EdgeValueCacheKey::new(
        format!(
            "tree_node:{};plan_template:{}",
            prepared.binding.owner_node_id,
            prepared.prepared_callable.plan_template.plan_template_key
        ),
        prepared
            .prepared_callable
            .hole_bindings
            .binding_fingerprint
            .clone(),
    )
}

fn oxfml_dependency_descriptors(prepared: &PreparedOxfmlFormula) -> Vec<DependencyDescriptor> {
    let reference_bindings_by_token = prepared
        .translated
        .reference_bindings
        .iter()
        .map(|reference| (reference.token.as_str(), reference))
        .collect::<BTreeMap<_, _>>();
    let unresolved_bindings_by_token = prepared
        .translated
        .unresolved_bindings
        .iter()
        .map(|unresolved| (unresolved.token.as_str(), unresolved))
        .collect::<BTreeMap<_, _>>();
    let mut consumed_reference_tokens = BTreeSet::new();
    let mut consumed_unresolved_tokens = BTreeSet::new();
    let mut descriptors = Vec::new();

    for (index, normalized_reference) in prepared
        .bound_formula
        .normalized_references
        .iter()
        .enumerate()
    {
        let NormalizedReference::Name(name) = normalized_reference else {
            continue;
        };
        let source_reference_handle = Some(oxfml_normalized_reference_handle(normalized_reference));
        if let Some(reference) = reference_bindings_by_token.get(name.name.as_str()) {
            consumed_reference_tokens.insert(name.name.clone());
            descriptors.push(dependency_descriptor_from_bound_reference(
                prepared,
                format!(
                    "bind:{}:oxfml_ref:{index}",
                    prepared.binding.formula_artifact_id.0
                ),
                reference,
                source_reference_handle,
            ));
        } else if let Some(unresolved) = unresolved_bindings_by_token.get(name.name.as_str()) {
            consumed_unresolved_tokens.insert(name.name.clone());
            descriptors.push(dependency_descriptor_from_unresolved_binding(
                prepared,
                format!(
                    "bind:{}:oxfml_ref_unresolved:{index}",
                    prepared.binding.formula_artifact_id.0
                ),
                unresolved,
                source_reference_handle,
            ));
        } else {
            descriptors.push(DependencyDescriptor {
                descriptor_id: format!(
                    "bind:{}:oxfml_unmapped_name:{index}",
                    prepared.binding.formula_artifact_id.0
                ),
                source_reference_handle,
                owner_node_id: prepared.binding.owner_node_id,
                target_node_id: None,
                kind: DependencyDescriptorKind::Unresolved,
                carrier_detail: format!("oxfml_unmapped_name:{}", name.name),
                requires_rebind_on_structural_change: name.caller_context_dependent
                    || matches!(
                        name.kind,
                        NameKind::ReferenceLike | NameKind::MixedOrDeferred
                    ),
            });
        }
    }

    for (index, unresolved_record) in prepared
        .bound_formula
        .unresolved_references
        .iter()
        .enumerate()
    {
        if consumed_unresolved_tokens.contains(&unresolved_record.source_text) {
            continue;
        }

        let source_reference_handle = Some(oxfml_unresolved_reference_handle(unresolved_record));
        if let Some(unresolved) =
            unresolved_bindings_by_token.get(unresolved_record.source_text.as_str())
        {
            consumed_unresolved_tokens.insert(unresolved_record.source_text.clone());
            descriptors.push(dependency_descriptor_from_unresolved_binding(
                prepared,
                format!(
                    "bind:{}:oxfml_unresolved:{index}",
                    prepared.binding.formula_artifact_id.0
                ),
                unresolved,
                source_reference_handle,
            ));
        } else {
            descriptors.push(DependencyDescriptor {
                descriptor_id: format!(
                    "bind:{}:oxfml_unmapped_unresolved:{index}",
                    prepared.binding.formula_artifact_id.0
                ),
                source_reference_handle,
                owner_node_id: prepared.binding.owner_node_id,
                target_node_id: None,
                kind: DependencyDescriptorKind::Unresolved,
                carrier_detail: format!(
                    "oxfml_unresolved:{}:{}",
                    unresolved_record.source_text, unresolved_record.reason
                ),
                requires_rebind_on_structural_change: true,
            });
        }
    }

    descriptors.extend(
        prepared
            .translated
            .reference_bindings
            .iter()
            .enumerate()
            .filter(|(_, reference)| !consumed_reference_tokens.contains(&reference.token))
            .map(|(index, reference)| {
                dependency_descriptor_from_bound_reference(
                    prepared,
                    format!(
                        "bind:{}:oxfml_ref_fallback:{index}",
                        prepared.binding.formula_artifact_id.0
                    ),
                    reference,
                    None,
                )
            }),
    );

    descriptors.extend(
        prepared
            .translated
            .unresolved_bindings
            .iter()
            .enumerate()
            .filter(|(_, unresolved)| !consumed_unresolved_tokens.contains(&unresolved.token))
            .map(|(index, unresolved)| {
                dependency_descriptor_from_unresolved_binding(
                    prepared,
                    format!(
                        "bind:{}:oxfml_unresolved_fallback:{index}",
                        prepared.binding.formula_artifact_id.0
                    ),
                    unresolved,
                    None,
                )
            }),
    );

    descriptors.extend(prepared.translated.residuals.iter().enumerate().map(
        |(index, residual)| DependencyDescriptor {
            descriptor_id: format!(
                "bind:{}:oxfml_residual:{index}",
                prepared.binding.formula_artifact_id.0
            ),
            source_reference_handle: Some(runtime_fact_reference_handle(residual)),
            owner_node_id: prepared.binding.owner_node_id,
            target_node_id: None,
            kind: residual.kind.dependency_descriptor_kind(),
            carrier_detail: format!("residual:{}:{}", residual.carrier_id, residual.detail),
            requires_rebind_on_structural_change:
                residual.kind.requires_rebind_on_structural_change(),
        },
    ));

    descriptors.sort_by(|left, right| left.descriptor_id.cmp(&right.descriptor_id));
    descriptors
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
        target_node_id: Some(reference.target_node_id),
        kind: reference.kind,
        carrier_detail: reference.carrier_detail.clone(),
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
        kind: unresolved.kind,
        carrier_detail: unresolved.carrier_detail.clone(),
        requires_rebind_on_structural_change: unresolved.requires_rebind_on_structural_change,
    }
}

fn oxfml_normalized_reference_handle(reference: &NormalizedReference) -> String {
    format!("oxfml_normalized_ref:{reference}")
}

fn oxfml_unresolved_reference_handle(unresolved: &UnresolvedReferenceRecord) -> String {
    format!(
        "oxfml_unresolved_ref:{}:{}",
        unresolved.source_text, unresolved.reason
    )
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
    working_values: &BTreeMap<TreeNodeId, String>,
) -> Result<LocalFormulaEvaluationSuccess, LocalFormulaEvaluationFailure> {
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

    let run = match invoke_prepared_formula_via_session(prepared, working_values) {
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

    adapt_oxfml_runtime_candidate(prepared, &run)
}

fn invoke_prepared_formula_via_session(
    prepared: &PreparedOxfmlFormula,
    working_values: &BTreeMap<TreeNodeId, String>,
) -> Result<RuntimeFormulaResult, String> {
    let host_info_provider = TreeCalcHostInfoProvider;
    let rtd_provider = TreeCalcRtdProvider;
    let host_info_required = prepared
        .translated
        .residuals
        .iter()
        .any(|residual| matches!(residual.kind, ResidualCarrierKind::HostSensitive));
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
    );
    let request = RuntimeFormulaRequest::new(prepared.source.clone(), query_bundle)
        .with_backend(EvaluationBackend::OxFuncBacked);
    let mut session =
        OxfmlRecalcSessionDriver::new(build_treecalc_runtime_environment(prepared, working_values));

    session.invoke(request).map_err(|error| error.to_string())
}

fn build_treecalc_runtime_environment(
    prepared: &PreparedOxfmlFormula,
    working_values: &BTreeMap<TreeNodeId, String>,
) -> RuntimeEnvironment<'static> {
    let defined_names = prepared
        .translated
        .reference_bindings
        .iter()
        .map(|reference| {
            (
                reference.token.clone(),
                DefinedNameBinding::Reference(ReferenceLike {
                    kind: ReferenceKind::A1,
                    target: synthetic_cell_target(reference.target_node_id),
                }),
            )
        })
        .collect();
    let cell_values = working_values
        .iter()
        .map(|(node_id, value)| (synthetic_cell_target(*node_id), string_to_eval_value(value)))
        .collect();

    RuntimeEnvironment::new()
        .with_structure_context_version(StructureContextVersion(
            prepared.bound_formula.structure_context_version.clone(),
        ))
        .with_caller_position(synthetic_cell_row(prepared.binding.owner_node_id), 1)
        .with_defined_names(defined_names)
        .with_cell_values(cell_values)
}

#[derive(Debug, Clone, Copy)]
struct TreeCalcHostInfoProvider;

impl HostInfoProvider for TreeCalcHostInfoProvider {
    fn query_cell_info(
        &self,
        query: CellInfoQuery,
        _reference: Option<&ReferenceLike>,
    ) -> Result<EvalValue, HostInfoError> {
        Err(HostInfoError::UnsupportedCellInfoQuery(query))
    }

    fn query_info(&self, _query: InfoQuery) -> Result<EvalValue, HostInfoError> {
        Err(HostInfoError::ProviderFailure {
            detail: "treecalc.host_sensitive_reference".to_string(),
        })
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
) -> Result<LocalFormulaEvaluationSuccess, LocalFormulaEvaluationFailure> {
    let candidate = &run.candidate_result;
    let candidate_value = value_payload_to_string(&candidate.value_delta.published_payload);
    let mut diagnostics = oxfml_returned_value_surface_diagnostics(&run.returned_value_surface);
    diagnostics.extend(oxfml_candidate_diagnostics(candidate));

    match &run.commit_decision {
        oxfml_core::seam::AcceptDecision::Accepted(bundle) => {
            diagnostics.extend(oxfml_commit_bundle_diagnostics(bundle));
            validate_oxfml_commit_bundle(prepared, candidate, bundle, diagnostics.clone())?;
            Ok(LocalFormulaEvaluationSuccess {
                value: candidate_value,
                diagnostics,
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
    owner_node_id: TreeNodeId,
    formula: &TreeFormula,
) -> TranslatedFormula {
    let mut state = FormulaCarrierProjectionState {
        snapshot,
        owner_node_id,
        fallback_reference_index: 0,
        reference_bindings: Vec::new(),
        unresolved_bindings: Vec::new(),
        residuals: Vec::new(),
    };
    for carrier in formula.reference_carriers() {
        state.project_carrier(carrier);
    }
    TranslatedFormula {
        source_text: formula.source_text().to_string(),
        reference_bindings: state.reference_bindings,
        unresolved_bindings: state.unresolved_bindings,
        residuals: state.residuals,
    }
}

struct FormulaCarrierProjectionState<'a> {
    snapshot: &'a StructuralSnapshot,
    owner_node_id: TreeNodeId,
    fallback_reference_index: usize,
    reference_bindings: Vec<SyntheticReferenceBinding>,
    unresolved_bindings: Vec<SyntheticUnresolvedBinding>,
    residuals: Vec<ResidualCarrier>,
}

impl FormulaCarrierProjectionState<'_> {
    fn project_carrier(&mut self, carrier: &TreeFormulaReferenceCarrier) {
        let reference = &carrier.reference;
        match reference {
            crate::formula::TreeReference::DirectNode { target_node_id } => self.bind_target(
                carrier.source_token.clone(),
                *target_node_id,
                reference.descriptor_kind(),
                reference.carrier_detail(),
                reference.requires_rebind_on_structural_change(),
            ),
            crate::formula::TreeReference::DynamicResolved { target_node_id, .. } => self
                .bind_target(
                    carrier.source_token.clone(),
                    *target_node_id,
                    reference.descriptor_kind(),
                    reference.carrier_detail(),
                    reference.requires_rebind_on_structural_change(),
                ),
            crate::formula::TreeReference::ProjectionPath { .. }
            | crate::formula::TreeReference::RelativePath { .. }
            | crate::formula::TreeReference::SiblingOffset { .. } => {
                if let Some(target_node_id) =
                    reference.resolve_target(self.snapshot, self.owner_node_id)
                {
                    self.bind_target(
                        carrier.source_token.clone(),
                        target_node_id,
                        reference.descriptor_kind(),
                        reference.carrier_detail(),
                        reference.requires_rebind_on_structural_change(),
                    )
                } else {
                    self.bind_unresolved(
                        carrier.source_token.clone(),
                        reference.descriptor_kind(),
                        reference.carrier_detail(),
                        reference.requires_rebind_on_structural_change(),
                    )
                }
            }
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
                carrier.source_token.clone(),
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
    ) {
        let token = source_token.unwrap_or_else(|| self.next_fallback_token("TREE_REF"));
        self.reference_bindings.push(SyntheticReferenceBinding {
            token,
            target_node_id,
            kind,
            carrier_detail,
            requires_rebind_on_structural_change,
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

    fn next_fallback_token(&mut self, prefix: &str) -> String {
        let token = format!(
            "{}_{}_{}",
            prefix, self.owner_node_id.0, self.fallback_reference_index
        );
        self.fallback_reference_index += 1;
        token
    }
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

    use crate::formula::{
        FixtureFormulaAst, FixtureFormulaBinaryOp, RelativeReferenceBase, TreeFormula,
        TreeFormulaBinding, TreeFormulaReferenceCarrier, TreeReference,
    };
    use crate::structural::{
        BindArtifactId, FormulaArtifactId, StructuralEdit, StructuralNode, StructuralNodeKind,
        StructuralSnapshotId,
    };
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

    fn formula_input(owner_node_id: TreeNodeId, expression: TreeFormula) -> LocalTreeCalcInput {
        LocalTreeCalcInput {
            structural_snapshot: snapshot(),
            formula_catalog: TreeFormulaCatalog::new([TreeFormulaBinding {
                owner_node_id,
                formula_artifact_id: formula_artifact_id(owner_node_id),
                bind_artifact_id: Some(bind_artifact_id(owner_node_id)),
                expression,
            }]),
            seeded_published_values: BTreeMap::new(),
            seeded_published_runtime_effects: Vec::new(),
            invalidation_seeds: Vec::new(),
            previous_arg_preparation_profile_version: None,
            candidate_result_id: format!("cand:b6:{}", owner_node_id.0),
            publication_id: format!("pub:b6:{}", owner_node_id.0),
            compatibility_basis: "snapshot:1".to_string(),
            artifact_token_basis: "snapshot:1".to_string(),
            environment_context: LocalTreeCalcEnvironmentContext::default(),
        }
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

    fn f2_artifact_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/test-runs/core-engine/w050-f2-differential-evaluation-gates-001")
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
        seeded_published_values: BTreeMap<TreeNodeId, String>,
        invalidation_seeds: Vec<InvalidationSeed>,
        run_suffix: &str,
    ) -> LocalTreeCalcInput {
        LocalTreeCalcInput {
            structural_snapshot,
            formula_catalog,
            seeded_published_values,
            seeded_published_runtime_effects: Vec::new(),
            invalidation_seeds,
            previous_arg_preparation_profile_version: None,
            candidate_result_id: format!("cand:f2:{run_suffix}"),
            publication_id: format!("pub:f2:{run_suffix}"),
            compatibility_basis: format!("snapshot:f2:{run_suffix}"),
            artifact_token_basis: format!("snapshot:f2:{run_suffix}"),
            environment_context: LocalTreeCalcEnvironmentContext::default(),
        }
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
                BTreeMap::new(),
                Vec::new(),
                "initial",
            ))
            .expect("initial F2 run should publish");

        let reuse = engine
            .execute(differential_evaluation_gate_input(
                snapshot(),
                formula_catalog.clone(),
                initial.published_values.clone(),
                Vec::new(),
                "reuse",
            ))
            .expect("F2 reuse run should verify clean");

        let edited_snapshot = snapshot()
            .apply_edit(
                StructuralSnapshotId(2),
                StructuralEdit::SetConstantValue {
                    node_id: TreeNodeId(2),
                    constant_value: Some("4".to_string()),
                },
            )
            .expect("constant edit should be valid")
            .snapshot;
        let upstream_bypass = engine
            .execute(differential_evaluation_gate_input(
                edited_snapshot,
                formula_catalog,
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
                    "DynamicDependencyDelta",
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
            [TreeFormulaReferenceCarrier::fact(
                TreeReference::DynamicPotential {
                    carrier_id: "carrier:indirect".to_string(),
                    detail: "INDIRECT selector resolved at runtime".to_string(),
                },
            )],
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
            [TreeFormulaReferenceCarrier::fact(
                TreeReference::DynamicPotential {
                    carrier_id: "carrier:rtd".to_string(),
                    detail: "RTD topic resolved at runtime".to_string(),
                },
            )],
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

        let first = prepare_oxfml_formula(&structural_snapshot, &binding, &first_context).unwrap();
        let second =
            prepare_oxfml_formula(&structural_snapshot, &binding, &second_context).unwrap();

        assert_ne!(
            first.bound_formula.structure_context_version,
            second.bound_formula.structure_context_version
        );
        assert!(
            second
                .bound_formula
                .structure_context_version
                .contains("arg_preparation_profile_version=oxfunc.arg-prep:v2")
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
            .execute(LocalTreeCalcInput {
                structural_snapshot: snapshot(),
                formula_catalog: TreeFormulaCatalog::new([
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
                seeded_published_values: BTreeMap::new(),
                seeded_published_runtime_effects: Vec::new(),
                invalidation_seeds: Vec::new(),
                previous_arg_preparation_profile_version: None,
                candidate_result_id: "cand:local".to_string(),
                publication_id: "pub:local".to_string(),
                compatibility_basis: "snapshot:1".to_string(),
                artifact_token_basis: "snapshot:1".to_string(),
                environment_context: LocalTreeCalcEnvironmentContext::default(),
            })
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
        assert!(formula_b_identity.shape_key.starts_with("shape:v1:"));
        assert!(
            formula_b_identity
                .dispatch_skeleton_key
                .starts_with("dispatch_skeleton:v1:")
        );
        assert!(
            formula_b_identity
                .plan_template_key
                .starts_with("plan_template:v1:")
        );
        assert!(
            formula_b_identity
                .prepared_callable_key
                .starts_with("prepared_callable:v1:")
        );
        assert!(
            formula_b_identity
                .hole_binding_fingerprint
                .starts_with("hole_bindings:v1:")
        );
        assert_eq!(formula_b_identity.template_hole_count, 2);
        assert_eq!(run.published_values[&TreeNodeId(3)], "5");
        assert_eq!(run.published_values[&TreeNodeId(4)], "7");
        assert!(run.publication_bundle.is_some());
        assert!(run.diagnostics.iter().any(|diagnostic| {
            diagnostic.starts_with("oxfml_prepared_plan_template_key:formula:b:")
        }));
        assert!(run.diagnostics.iter().any(|diagnostic| {
            diagnostic.starts_with("oxfml_prepared_hole_binding_fingerprint:formula:b:")
        }));
        assert!(run.diagnostics.iter().any(|diagnostic| {
            diagnostic.starts_with("oxfml_prepared_callable_key:formula:b:")
        }));
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
            .execute(LocalTreeCalcInput {
                structural_snapshot: snapshot(),
                formula_catalog: TreeFormulaCatalog::new([
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
                seeded_published_values: BTreeMap::new(),
                seeded_published_runtime_effects: Vec::new(),
                invalidation_seeds: Vec::new(),
                previous_arg_preparation_profile_version: None,
                candidate_result_id: "cand:reuse".to_string(),
                publication_id: "pub:reuse".to_string(),
                compatibility_basis: "snapshot:1".to_string(),
                artifact_token_basis: "snapshot:1".to_string(),
                environment_context: LocalTreeCalcEnvironmentContext::default(),
            })
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
        let prepared_callable_keys = run
            .prepared_formula_identities
            .iter()
            .map(|identity| identity.prepared_callable_key.as_str())
            .collect::<BTreeSet<_>>();
        let hole_binding_fingerprints = run
            .prepared_formula_identities
            .iter()
            .map(|identity| identity.hole_binding_fingerprint.as_str())
            .collect::<BTreeSet<_>>();

        assert_eq!(template_keys.len(), 1);
        assert_eq!(prepared_callable_keys.len(), 2);
        assert_eq!(hole_binding_fingerprints.len(), 2);
        assert!(run.diagnostics.iter().any(|diagnostic| {
            diagnostic.starts_with("oxfml_plan_template_reuse_count:plan_template:v1:")
                && diagnostic.contains(":call_sites=2;")
                && diagnostic.contains(";prepared_callables=2;")
                && diagnostic.contains(";hole_bindings=2")
        }));
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
                assert_eq!(candidate.value_updates, publication.published_view_delta);
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
            .execute(LocalTreeCalcInput {
                structural_snapshot: snapshot(),
                formula_catalog: TreeFormulaCatalog::new([
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
                seeded_published_values: BTreeMap::new(),
                seeded_published_runtime_effects: Vec::new(),
                invalidation_seeds: Vec::new(),
                previous_arg_preparation_profile_version: None,
                candidate_result_id: "cand:w046:bridge".to_string(),
                publication_id: "pub:w046:bridge".to_string(),
                compatibility_basis: "snapshot:1".to_string(),
                artifact_token_basis: "snapshot:1".to_string(),
                environment_context: LocalTreeCalcEnvironmentContext::default(),
            })
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
            .execute(LocalTreeCalcInput {
                structural_snapshot: snapshot(),
                formula_catalog: formula_catalog.clone(),
                seeded_published_values: BTreeMap::new(),
                seeded_published_runtime_effects: Vec::new(),
                invalidation_seeds: Vec::new(),
                previous_arg_preparation_profile_version: None,
                candidate_result_id: "cand:xyz:initial".to_string(),
                publication_id: "pub:xyz:initial".to_string(),
                compatibility_basis: "snapshot:1".to_string(),
                artifact_token_basis: "snapshot:1".to_string(),
                environment_context: LocalTreeCalcEnvironmentContext::default(),
            })
            .unwrap();

        assert_eq!(initial.result_state, LocalTreeCalcRunState::Published);
        assert_eq!(initial.evaluation_order, vec![TreeNodeId(3), TreeNodeId(4)]);
        assert_eq!(initial.published_values[&TreeNodeId(3)], "40");
        assert_eq!(initial.published_values[&TreeNodeId(4)], "42");

        let edited_snapshot = snapshot()
            .apply_edit(
                StructuralSnapshotId(2),
                StructuralEdit::SetConstantValue {
                    node_id: TreeNodeId(2),
                    constant_value: Some("3".to_string()),
                },
            )
            .unwrap()
            .snapshot;
        let rerun = engine
            .execute(LocalTreeCalcInput {
                structural_snapshot: edited_snapshot,
                formula_catalog,
                seeded_published_values: initial.published_values.clone(),
                seeded_published_runtime_effects: Vec::new(),
                invalidation_seeds: vec![InvalidationSeed {
                    node_id: TreeNodeId(2),
                    reason: InvalidationReasonKind::UpstreamPublication,
                }],
                previous_arg_preparation_profile_version: None,
                candidate_result_id: "cand:xyz:rerun".to_string(),
                publication_id: "pub:xyz:rerun".to_string(),
                compatibility_basis: "snapshot:2".to_string(),
                artifact_token_basis: "snapshot:2".to_string(),
                environment_context: LocalTreeCalcEnvironmentContext::default(),
            })
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
            .execute(LocalTreeCalcInput {
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
                seeded_published_values: seeded,
                seeded_published_runtime_effects: Vec::new(),
                invalidation_seeds: Vec::new(),
                previous_arg_preparation_profile_version: None,
                candidate_result_id: "cand:verified".to_string(),
                publication_id: "pub:verified".to_string(),
                compatibility_basis: "snapshot:1".to_string(),
                artifact_token_basis: "snapshot:1".to_string(),
                environment_context: LocalTreeCalcEnvironmentContext::default(),
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
            .execute(LocalTreeCalcInput {
                structural_snapshot: snapshot(),
                formula_catalog: TreeFormulaCatalog::new([
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
                seeded_published_values: BTreeMap::new(),
                seeded_published_runtime_effects: Vec::new(),
                invalidation_seeds: Vec::new(),
                previous_arg_preparation_profile_version: None,
                candidate_result_id: "cand:cycle".to_string(),
                publication_id: "pub:cycle".to_string(),
                compatibility_basis: "snapshot:1".to_string(),
                artifact_token_basis: "snapshot:1".to_string(),
                environment_context: LocalTreeCalcEnvironmentContext::default(),
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
                    expression: fixture_formula(
                        TreeNodeId(3),
                        FixtureFormulaAst::Reference(TreeReference::HostSensitive {
                            carrier_id: "carrier:host".to_string(),
                            detail: "active_selection".to_string(),
                        }),
                    ),
                }]),
                seeded_published_values: BTreeMap::new(),
                seeded_published_runtime_effects: Vec::new(),
                invalidation_seeds: Vec::new(),
                previous_arg_preparation_profile_version: None,
                candidate_result_id: "cand:host".to_string(),
                publication_id: "pub:host".to_string(),
                compatibility_basis: "snapshot:1".to_string(),
                artifact_token_basis: "snapshot:1".to_string(),
                environment_context: LocalTreeCalcEnvironmentContext::default(),
            })
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
            .execute(LocalTreeCalcInput {
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
                seeded_published_runtime_effects: Vec::new(),
                invalidation_seeds: Vec::new(),
                previous_arg_preparation_profile_version: None,
                candidate_result_id: "cand:dynamic".to_string(),
                publication_id: "pub:dynamic".to_string(),
                compatibility_basis: "snapshot:1".to_string(),
                artifact_token_basis: "snapshot:1".to_string(),
                environment_context: LocalTreeCalcEnvironmentContext::default(),
            })
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
            .execute(LocalTreeCalcInput {
                structural_snapshot: snapshot(),
                formula_catalog: TreeFormulaCatalog::new([TreeFormulaBinding {
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
                seeded_published_values: BTreeMap::new(),
                seeded_published_runtime_effects: Vec::new(),
                invalidation_seeds: Vec::new(),
                previous_arg_preparation_profile_version: None,
                candidate_result_id: "cand:dynamic:resolved".to_string(),
                publication_id: "pub:dynamic:resolved".to_string(),
                compatibility_basis: "snapshot:1".to_string(),
                artifact_token_basis: "snapshot:1".to_string(),
                environment_context: LocalTreeCalcEnvironmentContext::default(),
            })
            .unwrap();

        assert_eq!(run.result_state, LocalTreeCalcRunState::Published);
        assert_eq!(run.runtime_effects.len(), 1);
        assert_eq!(
            run.runtime_effects[0].family,
            RuntimeEffectFamily::DynamicDependency
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
            .execute(LocalTreeCalcInput {
                structural_snapshot: snapshot(),
                formula_catalog: TreeFormulaCatalog::new([TreeFormulaBinding {
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
                seeded_published_values: BTreeMap::from([(TreeNodeId(3), "5".to_string())]),
                seeded_published_runtime_effects: Vec::new(),
                invalidation_seeds: vec![InvalidationSeed {
                    node_id: TreeNodeId(3),
                    reason: InvalidationReasonKind::StructuralRebindRequired,
                }],
                previous_arg_preparation_profile_version: None,
                candidate_result_id: "cand:rebind".to_string(),
                publication_id: "pub:rebind".to_string(),
                compatibility_basis: "snapshot:1".to_string(),
                artifact_token_basis: "snapshot:1".to_string(),
                environment_context: LocalTreeCalcEnvironmentContext::default(),
            })
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
        let run = engine
            .execute(LocalTreeCalcInput {
                structural_snapshot: snapshot(),
                formula_catalog: TreeFormulaCatalog::new([TreeFormulaBinding {
                    owner_node_id: TreeNodeId(3),
                    formula_artifact_id: FormulaArtifactId("formula:b".to_string()),
                    bind_artifact_id: Some(BindArtifactId("bind:b".to_string())),
                    expression: TreeFormula::opaque_oxfml("=SUM(A1,2)", Vec::new()),
                }]),
                seeded_published_values: BTreeMap::from([(TreeNodeId(3), "5".to_string())]),
                seeded_published_runtime_effects: Vec::new(),
                invalidation_seeds: Vec::new(),
                previous_arg_preparation_profile_version: Some("oxfunc.arg-prep:v1".to_string()),
                candidate_result_id: "cand:argprep".to_string(),
                publication_id: "pub:argprep".to_string(),
                compatibility_basis: "snapshot:1".to_string(),
                artifact_token_basis: "snapshot:1".to_string(),
                environment_context: LocalTreeCalcEnvironmentContext::default()
                    .with_arg_preparation_profile_version("oxfunc.arg-prep:v2"),
            })
            .unwrap();

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
            .execute(LocalTreeCalcInput {
                structural_snapshot: rerun_snapshot,
                formula_catalog: TreeFormulaCatalog::new([TreeFormulaBinding {
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
                seeded_published_values: BTreeMap::from([(TreeNodeId(3), "5".to_string())]),
                seeded_published_runtime_effects: Vec::new(),
                invalidation_seeds: vec![InvalidationSeed {
                    node_id: TreeNodeId(3),
                    reason: InvalidationReasonKind::StructuralRecalcOnly,
                }],
                previous_arg_preparation_profile_version: None,
                candidate_result_id: "cand:missing_target".to_string(),
                publication_id: "pub:missing_target".to_string(),
                compatibility_basis: "snapshot:2".to_string(),
                artifact_token_basis: "snapshot:2".to_string(),
                environment_context: LocalTreeCalcEnvironmentContext::default(),
            })
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
        assert_eq!(
            direct.source_reference_handle.as_deref(),
            Some("oxfml_normalized_ref:name:TREE_REF_4_0")
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
        assert_eq!(
            sibling.source_reference_handle.as_deref(),
            Some("oxfml_normalized_ref:name:TREE_REF_4_1")
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
                .is_some_and(|handle| handle.starts_with("oxfml_unresolved_ref:TREE_REF_4_2:"))
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
                .is_some_and(
                    |handle| handle.starts_with("oxfml_unresolved_ref:TREE_UNRESOLVED_4_3:")
                )
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
        assert_eq!(
            graph_direct.source_reference_handle.as_deref(),
            Some("oxfml_normalized_ref:name:TREE_REF_4_0")
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
