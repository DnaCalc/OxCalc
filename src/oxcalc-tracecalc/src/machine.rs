#![forbid(unsafe_code)]

//! Shared `TraceCalc` engine and reference-machine execution.

use std::collections::{BTreeMap, BTreeSet};

use oxcalc_core::coordinator::{
    AcceptedCandidateResult, CoordinatorError, DependencyShapeUpdate, RejectKind, RuntimeEffect,
    RuntimeEffectFamily, TreeCalcCoordinator,
};
use oxcalc_core::recalc::{NodeCalcState, OverlayKey, OverlayKind, Stage1RecalcTracker};
use oxcalc_core::structural::{
    StructuralNode, StructuralNodeKind, StructuralSnapshot, StructuralSnapshotBuilder,
    StructuralSnapshotId, TreeNodeId,
};
use thiserror::Error;

use crate::assertions::evaluate_assertions;
use crate::contracts::{
    TraceCalcExecutionArtifacts, TraceCalcPinnedViewRecord, TraceCalcRejectRecord,
    TraceCalcScenario, TraceCalcScenarioResultState, TraceCalcStep, TraceCalcTraceEvent,
};
use crate::planner::{TraceCalcScenarioPlanner, TraceCalcWorksetPlan};

#[derive(Debug, Clone, Default)]
pub struct TraceCalcEngineMachine;

#[derive(Debug, Clone, Default)]
pub struct TraceCalcReferenceMachine;

#[derive(Debug, Error)]
pub enum TraceCalcExecutionError {
    #[error("{0}")]
    Message(String),
    #[error(transparent)]
    Structural(#[from] oxcalc_core::structural::StructuralError),
    #[error(transparent)]
    Coordinator(#[from] CoordinatorError),
    #[error(transparent)]
    Recalc(#[from] oxcalc_core::recalc::RecalcError),
}

impl TraceCalcEngineMachine {
    pub fn execute(
        &self,
        scenario: &TraceCalcScenario,
    ) -> Result<TraceCalcExecutionArtifacts, TraceCalcExecutionError> {
        execute_shared(scenario)
    }
}

impl TraceCalcReferenceMachine {
    pub fn execute(
        &self,
        scenario: &TraceCalcScenario,
    ) -> Result<TraceCalcExecutionArtifacts, TraceCalcExecutionError> {
        execute_shared(scenario)
    }
}

fn execute_shared(
    scenario: &TraceCalcScenario,
) -> Result<TraceCalcExecutionArtifacts, TraceCalcExecutionError> {
    let mut state = MachineState::create(scenario)?;
    for step in &scenario.steps {
        execute_step(&mut state, scenario, step)?;
    }

    let published_values = state.read_published_values();
    let pinned_views = state.read_pinned_views();
    let counters = state.counter_entries();
    let pinned_projection = pinned_views
        .iter()
        .map(|view| (view.view_id.clone(), view.node_values.clone()))
        .collect::<Vec<_>>();

    let assertion_failures = evaluate_assertions(
        scenario,
        &published_values,
        &pinned_projection,
        &state.trace_events,
        &counters,
        &state.rejects,
    );

    Ok(TraceCalcExecutionArtifacts {
        scenario_id: scenario.scenario_id.clone(),
        result_state: if assertion_failures.is_empty() {
            TraceCalcScenarioResultState::Passed
        } else {
            TraceCalcScenarioResultState::FailedAssertion
        },
        assertion_failures,
        trace_events: state.trace_events,
        counters,
        published_values,
        pinned_views,
        rejects: state.rejects,
    })
}

fn execute_step(
    state: &mut MachineState,
    scenario: &TraceCalcScenario,
    step: &TraceCalcStep,
) -> Result<(), TraceCalcExecutionError> {
    match step.kind.as_str() {
        "pin_view" => pin_view(state, scenario, step),
        "unpin_view" => unpin_view(state, step),
        "mark_stale" => {
            mark_stale(state, step);
            Ok(())
        }
        "admit_work" => admit_work(state, scenario, step),
        "emit_candidate_result" => emit_candidate_result(state, scenario, step),
        "emit_reject" => emit_reject(state, scenario, step),
        "publish_candidate" => publish_candidate(state, step),
        "verify_clean" => verify_clean(state, scenario, step),
        "seed_overlay" => seed_overlay(state, scenario, step),
        "reset_fixture" => {
            reset_fixture(state, step);
            Ok(())
        }
        other => Err(TraceCalcExecutionError::Message(format!(
            "Unsupported step kind '{other}'."
        ))),
    }
}

fn pin_view(
    state: &mut MachineState,
    scenario: &TraceCalcScenario,
    step: &TraceCalcStep,
) -> Result<(), TraceCalcExecutionError> {
    let view_id = step.view_id.as_deref().ok_or_else(|| {
        TraceCalcExecutionError::Message("pin_view requires view_id.".to_string())
    })?;
    let pinned = state.coordinator.pin_reader(view_id);
    state
        .pin_observed_nodes
        .insert(view_id.to_string(), step.observed_nodes.clone());
    state.increment_counter("reader.pinned");
    state.set_counter(
        "pinned_reader_count",
        i64::try_from(state.coordinator.pinned_readers().len()).unwrap_or(0),
    );
    state.add_event(
        &step.step_id,
        "reader_pinned",
        vec![
            ("view_id".to_string(), pinned.reader_id),
            (
                "snapshot_id".to_string(),
                scenario.initial_graph.snapshot_id.clone(),
            ),
        ],
    );
    Ok(())
}

fn unpin_view(
    state: &mut MachineState,
    step: &TraceCalcStep,
) -> Result<(), TraceCalcExecutionError> {
    let Some(view_id) = step.view_id.as_deref() else {
        return Ok(());
    };
    if !state.coordinator.unpin_reader(view_id) {
        return Ok(());
    }

    state.pin_observed_nodes.remove(view_id);
    state.increment_counter("reader.unpinned");
    state.increment_counter("release_events");
    state.set_counter(
        "pinned_reader_count",
        i64::try_from(state.coordinator.pinned_readers().len()).unwrap_or(0),
    );
    state.add_event(
        &step.step_id,
        "reader_unpinned",
        vec![("view_id".to_string(), view_id.to_string())],
    );

    let remaining_reader_count = state.coordinator.pinned_readers().len();
    if remaining_reader_count > 0 {
        state.increment_counter("retention_waiting_on_readers");
        state.increment_counter("overlay.release_deferred");
        state.add_event(
            &step.step_id,
            "overlay_release_deferred_for_remaining_readers",
            vec![
                ("view_id".to_string(), view_id.to_string()),
                (
                    "remaining_reader_count".to_string(),
                    remaining_reader_count.to_string(),
                ),
            ],
        );
        return Ok(());
    }

    let mut eviction_opened = false;
    for node_id in state.node_id_map.values().copied().collect::<Vec<_>>() {
        let node_state = state.recalc_tracker.get_state(node_id);
        if matches!(
            node_state,
            NodeCalcState::Clean | NodeCalcState::VerifiedClean
        ) {
            state.recalc_tracker.release_and_evict_eligible(node_id)?;
            eviction_opened = true;
        }
    }

    if eviction_opened {
        state.increment_counter("overlay.eviction_eligible");
        state.increment_counter("eviction_eligibility_opened");
        state.add_event(
            &step.step_id,
            "eviction_eligibility_opened",
            vec![("view_id".to_string(), view_id.to_string())],
        );
    }

    let evicted = state.recalc_tracker.evict_eligible_overlays();
    if evicted > 0 {
        state.add_to_counter("overlay_evictions", i64::try_from(evicted).unwrap_or(0));
        state.add_event(
            &step.step_id,
            "overlay_released",
            vec![("evicted_count".to_string(), evicted.to_string())],
        );
    }

    Ok(())
}

fn mark_stale(state: &mut MachineState, step: &TraceCalcStep) {
    for target in &step.targets {
        state.dirty_seeds.insert(target.clone());
        let target_node_id = state.resolve_node(target);
        if state.recalc_tracker.get_state(target_node_id) == NodeCalcState::Clean {
            state.recalc_tracker.mark_dirty(target_node_id);
            state.increment_counter("recalc.mark_dirty");
            state.increment_counter("nodes_marked_dirty");
        }

        state.add_event(
            &step.step_id,
            "node_marked_dirty",
            vec![("node_id".to_string(), target.clone())],
        );
    }
}

fn admit_work(
    state: &mut MachineState,
    scenario: &TraceCalcScenario,
    step: &TraceCalcStep,
) -> Result<(), TraceCalcExecutionError> {
    let admission_id = step.admission_id.as_ref().ok_or_else(|| {
        TraceCalcExecutionError::Message("admit_work requires admission_id.".to_string())
    })?;
    let compatibility_basis = step
        .compatibility_basis
        .clone()
        .unwrap_or_else(|| scenario.initial_graph.snapshot_id.clone());

    state.current_admission_id = Some(admission_id.clone());
    state.current_compatibility_basis = Some(compatibility_basis.clone());
    state.current_plan = state
        .planner
        .plan_workset(&step.targets, &state.dirty_seeds);
    state.current_targets = state.current_plan.ordered_nodes.clone();

    emit_plan_artifacts(state, &step.step_id);
    advance_scheduled_nodes_to_evaluating(state, &step.step_id, &compatibility_basis)?;

    let placeholder = state.build_placeholder_candidate(
        admission_id,
        &compatibility_basis,
        &state.current_targets,
    );
    state.coordinator.admit_candidate_work(placeholder)?;
    state.increment_counter("candidate.admitted");
    state.increment_counter("candidate_admissions");
    state.add_event(
        &step.step_id,
        "candidate_admitted",
        vec![
            ("admission_id".to_string(), admission_id.clone()),
            ("compatibility_basis".to_string(), compatibility_basis),
            (
                "target_count".to_string(),
                state.current_targets.len().to_string(),
            ),
        ],
    );
    Ok(())
}

fn emit_plan_artifacts(state: &mut MachineState, step_id: &str) {
    if state.current_plan.groups.is_empty() {
        return;
    }

    let groups = state.current_plan.groups.clone();
    for (index, group) in groups.into_iter().enumerate() {
        if group.len() > 1 {
            state.increment_counter("cycle_region_groups");
            state.add_event(
                step_id,
                "cycle_region_detected",
                vec![
                    ("group_index".to_string(), index.to_string()),
                    ("nodes".to_string(), group.join(",")),
                ],
            );
        }

        state.add_event(
            step_id,
            if group.len() > 1 {
                "scc_group_scheduled"
            } else {
                "topo_group_scheduled"
            },
            vec![
                ("group_index".to_string(), index.to_string()),
                ("nodes".to_string(), group.join(",")),
            ],
        );
    }
}

fn advance_scheduled_nodes_to_evaluating(
    state: &mut MachineState,
    step_id: &str,
    compatibility_basis: &str,
) -> Result<(), TraceCalcExecutionError> {
    for node_id_text in state.current_targets.clone() {
        let node_id = state.resolve_node(&node_id_text);
        let mut current_state = state.recalc_tracker.get_state(node_id);
        if current_state == NodeCalcState::Clean {
            state.recalc_tracker.mark_dirty(node_id);
            state.increment_counter("recalc.mark_dirty");
            state.increment_counter("nodes_marked_dirty");
            state.add_event(
                step_id,
                "node_marked_dirty",
                vec![("node_id".to_string(), node_id_text.clone())],
            );
            current_state = state.recalc_tracker.get_state(node_id);
        }

        if current_state == NodeCalcState::RejectedPendingRepair {
            state
                .recalc_tracker
                .reenter_rejected_pending_repair(node_id)?;
            state.increment_counter("nodes_marked_needed");
            state.add_event(
                step_id,
                "fallback_reentered",
                vec![("node_id".to_string(), node_id_text.clone())],
            );
            current_state = state.recalc_tracker.get_state(node_id);
        }

        if current_state == NodeCalcState::DirtyPending {
            state.recalc_tracker.mark_needed(node_id)?;
            state.increment_counter("nodes_marked_needed");
            state.add_event(
                step_id,
                "node_marked_needed",
                vec![("node_id".to_string(), node_id_text.clone())],
            );
            current_state = state.recalc_tracker.get_state(node_id);
        }

        if current_state == NodeCalcState::Needed {
            state
                .recalc_tracker
                .begin_evaluate(node_id, compatibility_basis)?;
            state.add_event(
                step_id,
                "evaluation_started",
                vec![
                    ("node_id".to_string(), node_id_text),
                    (
                        "compatibility_basis".to_string(),
                        compatibility_basis.to_string(),
                    ),
                ],
            );
        }
    }

    Ok(())
}

fn emit_candidate_result(
    state: &mut MachineState,
    scenario: &TraceCalcScenario,
    step: &TraceCalcStep,
) -> Result<(), TraceCalcExecutionError> {
    let compatibility_basis = step
        .compatibility_basis
        .clone()
        .or_else(|| state.current_compatibility_basis.clone())
        .unwrap_or_else(|| scenario.initial_graph.snapshot_id.clone());
    let candidate_result_id = step.candidate_result_id.as_deref().ok_or_else(|| {
        TraceCalcExecutionError::Message(
            "emit_candidate_result requires candidate_result_id.".into(),
        )
    })?;

    for target in state.current_targets.clone() {
        let target_node_id = state.resolve_node(&target);
        let has_dependency_shape_update = step
            .dependency_shape_updates
            .iter()
            .any(|update| update.node_id == target);
        let overlay_kind = if has_dependency_shape_update {
            OverlayKind::DynamicDependency
        } else {
            OverlayKind::CapabilityFenceAttachment
        };
        let overlay_key = OverlayKey {
            owner_node_id: target_node_id,
            overlay_kind,
            structural_snapshot_id: state.snapshot.snapshot_id(),
            compatibility_basis: compatibility_basis.clone(),
            payload_identity: Some(candidate_result_id.to_string()),
        };

        let had_overlay = state.recalc_tracker.overlays().contains_key(&overlay_key);
        state.increment_counter("overlay_lookups");
        if had_overlay {
            state.increment_counter("overlay_hits");
            if state
                .recalc_tracker
                .overlays()
                .get(&overlay_key)
                .is_some_and(|entry| entry.is_protected)
            {
                state.increment_counter("overlay_reuse_after_retention");
            }
        } else {
            state.increment_counter("overlay_misses");
            state.increment_counter("overlay_creations");
        }

        if has_dependency_shape_update {
            state.recalc_tracker.produce_dependency_shape_update(
                target_node_id,
                &compatibility_basis,
                candidate_result_id,
            )?;
            state.add_event(
                &step.step_id,
                "candidate_shape_update_produced",
                vec![("node_id".to_string(), target.clone())],
            );
        } else {
            state.recalc_tracker.produce_candidate_result(
                target_node_id,
                &compatibility_basis,
                candidate_result_id,
            )?;
        }
    }

    let candidate = state.build_candidate(step, &compatibility_basis);
    state.coordinator.admit_candidate_work(candidate.clone())?;
    state
        .coordinator
        .record_accepted_candidate_result(&candidate.candidate_result_id)?;
    state.increment_counter("candidate.emitted");
    state.increment_counter("accepted_candidate_results");
    state.add_event(
        &step.step_id,
        "candidate_emitted",
        vec![
            (
                "candidate_result_id".to_string(),
                candidate.candidate_result_id.clone(),
            ),
            (
                "target_count".to_string(),
                candidate.target_set.len().to_string(),
            ),
        ],
    );
    Ok(())
}

fn emit_reject(
    state: &mut MachineState,
    scenario: &TraceCalcScenario,
    step: &TraceCalcStep,
) -> Result<(), TraceCalcExecutionError> {
    let reject_kind = step
        .reject_kind
        .clone()
        .unwrap_or_else(|| "host_injected_failure".to_string());
    let compatibility_basis = state
        .current_compatibility_basis
        .clone()
        .unwrap_or_else(|| scenario.initial_graph.snapshot_id.clone());

    if state.current_targets.is_empty() {
        state.current_plan = state
            .planner
            .plan_workset(&step.targets, &state.dirty_seeds);
        state.current_targets = state.current_plan.ordered_nodes.clone();
        advance_scheduled_nodes_to_evaluating(state, &step.step_id, &compatibility_basis)?;
    }

    for target in state.current_targets.clone() {
        state
            .recalc_tracker
            .reject_or_fallback(state.resolve_node(&target), &reject_kind)?;
    }

    let reject_id = step
        .reject_id
        .clone()
        .or_else(|| state.current_admission_id.clone())
        .unwrap_or_else(|| "reject:unknown".to_string());
    let reject_detail_text = step
        .reject_detail_text
        .clone()
        .or_else(|| {
            step.reject_detail
                .as_ref()
                .map(serde_json::Value::to_string)
        })
        .unwrap_or_default();
    let reject_detail = state.coordinator.reject_candidate_work(
        state.current_admission_id.as_deref().unwrap_or(&reject_id),
        map_reject_kind(&reject_kind),
        &reject_detail_text,
    )?;
    state.rejects.push(TraceCalcRejectRecord {
        reject_id: step
            .reject_id
            .clone()
            .unwrap_or_else(|| reject_detail.candidate_result_id.clone()),
        reject_kind: reject_kind.clone(),
        reject_detail: reject_detail.detail,
    });
    state.increment_counter("candidate.rejected");
    state.increment_counter("abandoned_candidates");
    state.increment_counter(&format!("rejects_by_class.{reject_kind}"));
    state.increment_counter(&format!("fallback_by_reason.{reject_kind}"));
    state.add_to_counter(
        "fallback_affected_work_volume",
        i64::try_from(state.current_targets.len()).unwrap_or(0),
    );
    state.add_event(
        &step.step_id,
        "candidate_rejected",
        vec![
            (
                "reject_id".to_string(),
                step.reject_id.clone().unwrap_or_else(|| reject_id.clone()),
            ),
            ("reject_kind".to_string(), reject_kind),
            (
                "target_count".to_string(),
                state.current_targets.len().to_string(),
            ),
        ],
    );
    Ok(())
}

fn publish_candidate(
    state: &mut MachineState,
    step: &TraceCalcStep,
) -> Result<(), TraceCalcExecutionError> {
    let publication_id = step
        .publication_id
        .as_deref()
        .unwrap_or("publication:unknown");
    let publication = state.coordinator.accept_and_publish(publication_id)?;
    for target in state.current_targets.clone() {
        let node_id = state.resolve_node(&target);
        state.recalc_tracker.publish_and_clear(node_id)?;
        state.dirty_seeds.remove(&target);
    }

    let retained = state
        .recalc_tracker
        .overlays()
        .values()
        .filter(|entry| {
            entry.key.overlay_kind == OverlayKind::DynamicDependency && entry.is_protected
        })
        .count();
    if retained > 0 {
        state.set_counter("overlay.retained", i64::try_from(retained).unwrap_or(0));
        if !state.coordinator.pinned_readers().is_empty() {
            state.increment_counter("retention_blocked_cleanup");
        }
        state.add_event(
            &step.step_id,
            "overlay_retained",
            vec![(
                "publication_id".to_string(),
                publication.publication_id.clone(),
            )],
        );
    }

    state.increment_counter("candidate.published");
    state.increment_counter("publications_committed");
    state.add_event(
        &step.step_id,
        "candidate_published",
        vec![
            (
                "candidate_result_id".to_string(),
                publication.candidate_result_id.clone(),
            ),
            (
                "publication_id".to_string(),
                publication.publication_id.clone(),
            ),
        ],
    );
    state.clear_current_work();
    Ok(())
}

fn verify_clean(
    state: &mut MachineState,
    scenario: &TraceCalcScenario,
    step: &TraceCalcStep,
) -> Result<(), TraceCalcExecutionError> {
    let compatibility_basis = state
        .current_compatibility_basis
        .clone()
        .unwrap_or_else(|| scenario.initial_graph.snapshot_id.clone());
    if state.current_targets.is_empty() {
        state.current_plan = state
            .planner
            .plan_workset(&step.targets, &state.dirty_seeds);
        state.current_targets = state.current_plan.ordered_nodes.clone();
        advance_scheduled_nodes_to_evaluating(state, &step.step_id, &compatibility_basis)?;
    }

    for target in state.current_targets.clone() {
        let target_node_id = state.resolve_node(&target);
        state.recalc_tracker.verify_clean(target_node_id)?;
        state.dirty_seeds.remove(&target);
        state.increment_counter("recalc.verified_clean");
        state.increment_counter("verified_clean_nodes");
        state.add_event(
            &step.step_id,
            "node_verified_clean",
            vec![("node_id".to_string(), target)],
        );
    }

    state.clear_current_work();
    Ok(())
}

fn seed_overlay(
    state: &mut MachineState,
    scenario: &TraceCalcScenario,
    step: &TraceCalcStep,
) -> Result<(), TraceCalcExecutionError> {
    let compatibility_basis = state
        .current_compatibility_basis
        .clone()
        .unwrap_or_else(|| scenario.initial_graph.snapshot_id.clone());
    let owner_node_id = step.owner_node_id.as_ref().ok_or_else(|| {
        TraceCalcExecutionError::Message("seed_overlay requires owner_node_id.".to_string())
    })?;
    let overlay_target = state.resolve_node(owner_node_id);
    state.recalc_tracker.mark_dirty(overlay_target);
    state.recalc_tracker.mark_needed(overlay_target)?;
    state
        .recalc_tracker
        .begin_evaluate(overlay_target, &compatibility_basis)?;
    state.recalc_tracker.produce_dependency_shape_update(
        overlay_target,
        &compatibility_basis,
        "seed_overlay",
    )?;
    state.increment_counter("overlay.retained");
    state.increment_counter("overlay_creations");
    state.add_event(
        &step.step_id,
        "overlay_retained",
        vec![("owner_node_id".to_string(), owner_node_id.clone())],
    );
    Ok(())
}

fn reset_fixture(state: &mut MachineState, step: &TraceCalcStep) {
    state.pin_observed_nodes.clear();
    state.rejects.clear();
    state.dirty_seeds.clear();
    state.clear_current_work();
    state.add_event(&step.step_id, "fixture_reset", Vec::new());
}

fn map_reject_kind(reject_kind: &str) -> RejectKind {
    match reject_kind {
        "snapshot_mismatch" => RejectKind::SnapshotMismatch,
        "artifact_token_mismatch" => RejectKind::ArtifactTokenMismatch,
        "profile_version_mismatch" => RejectKind::ProfileVersionMismatch,
        "capability_mismatch" => RejectKind::CapabilityMismatch,
        "publication_fence_mismatch" => RejectKind::PublicationFenceMismatch,
        "dynamic_dependency_failure" => RejectKind::DynamicDependencyFailure,
        "synthetic_cycle_reject" => RejectKind::SyntheticCycleReject,
        _ => RejectKind::HostInjectedFailure,
    }
}

fn runtime_effect_family(effect_kind: &str) -> RuntimeEffectFamily {
    match effect_kind {
        "dynamic_ref_activated" | "dynamic_ref_released" | "runtime_effect.dynamic_reference" => {
            RuntimeEffectFamily::DynamicDependency
        }
        "runtime_effect.host_sensitive_reference" => RuntimeEffectFamily::ExecutionRestriction,
        _ if effect_kind.contains("capability") => RuntimeEffectFamily::CapabilitySensitive,
        _ if effect_kind.contains("shape") || effect_kind.contains("topology") => {
            RuntimeEffectFamily::ShapeTopology
        }
        _ if effect_kind.contains("host_sensitive")
            || effect_kind.contains("execution_restriction") =>
        {
            RuntimeEffectFamily::ExecutionRestriction
        }
        _ => RuntimeEffectFamily::ExecutionRestriction,
    }
}

struct MachineState {
    snapshot: StructuralSnapshot,
    external_snapshot_id: String,
    coordinator: TreeCalcCoordinator,
    recalc_tracker: Stage1RecalcTracker,
    planner: TraceCalcScenarioPlanner,
    node_id_map: BTreeMap<String, TreeNodeId>,
    reverse_node_id_map: BTreeMap<TreeNodeId, String>,
    pin_observed_nodes: BTreeMap<String, Vec<String>>,
    counters: BTreeMap<String, i64>,
    trace_events: Vec<TraceCalcTraceEvent>,
    rejects: Vec<TraceCalcRejectRecord>,
    current_plan: TraceCalcWorksetPlan,
    current_targets: Vec<String>,
    current_admission_id: Option<String>,
    current_compatibility_basis: Option<String>,
    dirty_seeds: BTreeSet<String>,
    event_counter: usize,
}

impl MachineState {
    fn create(scenario: &TraceCalcScenario) -> Result<Self, TraceCalcExecutionError> {
        let snapshot = build_snapshot(scenario)?;
        let node_id_map = scenario
            .initial_graph
            .nodes
            .iter()
            .enumerate()
            .map(|(index, node)| (node.node_id.clone(), TreeNodeId((index + 2) as u64)))
            .collect::<BTreeMap<_, _>>();
        let reverse_node_id_map = node_id_map
            .iter()
            .map(|(node_id, tree_node_id)| (*tree_node_id, node_id.clone()))
            .collect::<BTreeMap<_, _>>();

        let mut coordinator = TreeCalcCoordinator::new(snapshot.clone());
        let published_values = scenario
            .initial_runtime
            .published_values
            .iter()
            .map(|entry| (node_id_map[&entry.node_id], entry.value.clone()))
            .collect::<BTreeMap<_, _>>();
        let runtime_effects = scenario
            .initial_runtime
            .published_runtime_effects
            .iter()
            .map(|effect| RuntimeEffect {
                kind: effect.effect_kind.clone(),
                family: runtime_effect_family(&effect.effect_kind),
                detail: effect
                    .payload
                    .as_ref()
                    .map(serde_json::Value::to_string)
                    .unwrap_or_default(),
            })
            .collect::<Vec<_>>();
        coordinator.seed_published_view(&published_values, None, &runtime_effects);

        let mut state = Self {
            snapshot: snapshot.clone(),
            external_snapshot_id: scenario.initial_graph.snapshot_id.clone(),
            coordinator,
            recalc_tracker: Stage1RecalcTracker::new(snapshot),
            planner: TraceCalcScenarioPlanner::new(scenario),
            node_id_map,
            reverse_node_id_map,
            pin_observed_nodes: BTreeMap::new(),
            counters: BTreeMap::new(),
            trace_events: Vec::new(),
            rejects: Vec::new(),
            current_plan: TraceCalcWorksetPlan::empty(),
            current_targets: Vec::new(),
            current_admission_id: None,
            current_compatibility_basis: None,
            dirty_seeds: BTreeSet::new(),
            event_counter: 0,
        };

        for pinned_view in &scenario.initial_runtime.pinned_views {
            let pinned = state.coordinator.pin_reader(&pinned_view.view_id);
            state
                .pin_observed_nodes
                .insert(pinned.reader_id, pinned_view.observed_nodes.clone());
        }

        Ok(state)
    }

    fn resolve_node(&self, node_id: &str) -> TreeNodeId {
        self.node_id_map[node_id]
    }

    fn scenario_node_id(&self, node_id: TreeNodeId) -> String {
        self.reverse_node_id_map
            .get(&node_id)
            .cloned()
            .unwrap_or_else(|| format!("{node_id}"))
    }

    fn add_event(&mut self, step_id: &str, label: &str, mut payload: Vec<(String, String)>) {
        self.event_counter += 1;
        payload.sort_by(|left, right| left.0.cmp(&right.0));
        self.trace_events.push(TraceCalcTraceEvent {
            event_id: format!("evt-{:04}", self.event_counter),
            step_id: step_id.to_string(),
            label: label.to_string(),
            payload,
        });
    }

    fn increment_counter(&mut self, counter: &str) {
        *self.counters.entry(counter.to_string()).or_insert(0) += 1;
    }

    fn set_counter(&mut self, counter: &str, value: i64) {
        self.counters.insert(counter.to_string(), value);
    }

    fn add_to_counter(&mut self, counter: &str, value: i64) {
        *self.counters.entry(counter.to_string()).or_insert(0) += value;
    }

    fn counter_entries(&self) -> Vec<(String, i64)> {
        self.counters
            .iter()
            .map(|(counter, value)| (counter.clone(), *value))
            .collect()
    }

    fn read_published_values(&self) -> Vec<(String, String)> {
        let mut values = self
            .coordinator
            .published_view()
            .values
            .iter()
            .map(|(node_id, value)| (self.scenario_node_id(*node_id), value.clone()))
            .collect::<Vec<_>>();
        values.sort_by(|left, right| left.0.cmp(&right.0));
        values
    }

    fn read_pinned_views(&self) -> Vec<TraceCalcPinnedViewRecord> {
        let mut views = self
            .coordinator
            .pinned_readers()
            .into_iter()
            .map(|view| {
                let observed_nodes = self
                    .pin_observed_nodes
                    .get(&view.reader_id)
                    .cloned()
                    .unwrap_or_default();
                let mut node_values = view
                    .values
                    .iter()
                    .map(|(node_id, value)| (self.scenario_node_id(*node_id), value.clone()))
                    .filter(|(node_id, _)| {
                        observed_nodes.is_empty()
                            || observed_nodes.iter().any(|observed| observed == node_id)
                    })
                    .collect::<Vec<_>>();
                node_values.sort_by(|left, right| left.0.cmp(&right.0));

                TraceCalcPinnedViewRecord {
                    view_id: view.reader_id,
                    snapshot_id: self.external_snapshot_id.clone(),
                    node_values,
                }
            })
            .collect::<Vec<_>>();
        views.sort_by(|left, right| left.view_id.cmp(&right.view_id));
        views
    }

    fn build_placeholder_candidate(
        &self,
        candidate_result_id: &str,
        compatibility_basis: &str,
        current_targets: &[String],
    ) -> AcceptedCandidateResult {
        AcceptedCandidateResult {
            candidate_result_id: candidate_result_id.to_string(),
            structural_snapshot_id: self.snapshot.snapshot_id(),
            artifact_token_basis: compatibility_basis.to_string(),
            compatibility_basis: compatibility_basis.to_string(),
            target_set: current_targets
                .iter()
                .map(|target| self.resolve_node(target))
                .collect(),
            value_updates: BTreeMap::new(),
            dependency_shape_updates: Vec::new(),
            runtime_effects: Vec::new(),
            diagnostic_events: Vec::new(),
        }
    }

    fn build_candidate(
        &self,
        step: &TraceCalcStep,
        compatibility_basis: &str,
    ) -> AcceptedCandidateResult {
        let value_updates = step
            .value_updates
            .iter()
            .map(|entry| (self.resolve_node(&entry.node_id), entry.value.clone()))
            .collect::<BTreeMap<_, _>>();
        let dependency_shape_updates = step
            .dependency_shape_updates
            .iter()
            .map(|update| {
                let mut affected_node_ids = vec![self.resolve_node(&update.node_id)];
                if let Some(dependency_id) = &update.dependency_id {
                    affected_node_ids.push(self.resolve_node(dependency_id));
                }
                DependencyShapeUpdate {
                    kind: update.kind.clone(),
                    affected_node_ids,
                }
            })
            .collect::<Vec<_>>();
        let runtime_effects = step
            .runtime_effects
            .iter()
            .map(|effect| RuntimeEffect {
                kind: effect.effect_kind.clone(),
                family: runtime_effect_family(&effect.effect_kind),
                detail: effect
                    .payload
                    .as_ref()
                    .map(serde_json::Value::to_string)
                    .unwrap_or_default(),
            })
            .collect::<Vec<_>>();

        AcceptedCandidateResult {
            candidate_result_id: step
                .candidate_result_id
                .clone()
                .unwrap_or_else(|| "candidate:unknown".to_string()),
            structural_snapshot_id: self.snapshot.snapshot_id(),
            artifact_token_basis: compatibility_basis.to_string(),
            compatibility_basis: compatibility_basis.to_string(),
            target_set: self
                .current_targets
                .iter()
                .map(|target| self.resolve_node(target))
                .collect(),
            value_updates,
            dependency_shape_updates,
            runtime_effects,
            diagnostic_events: step.diagnostic_events.clone(),
        }
    }

    fn clear_current_work(&mut self) {
        self.current_plan = TraceCalcWorksetPlan::empty();
        self.current_targets.clear();
        self.current_admission_id = None;
        self.current_compatibility_basis = None;
    }
}

fn build_snapshot(
    scenario: &TraceCalcScenario,
) -> Result<StructuralSnapshot, TraceCalcExecutionError> {
    let root_node_id = TreeNodeId(1);
    let mut builder = StructuralSnapshotBuilder::new(None);
    builder.set_node(StructuralNode {
        node_id: root_node_id,
        kind: StructuralNodeKind::Root,
        symbol: "root".to_string(),
        parent_id: None,
        child_ids: Vec::new(),
        formula_artifact_id: None,
        bind_artifact_id: None,
        constant_value: None,
    });
    builder.set_root(root_node_id);

    let mut child_ids = Vec::new();
    for (index, node) in scenario.initial_graph.nodes.iter().enumerate() {
        let node_id = TreeNodeId((index + 2) as u64);
        child_ids.push(node_id);
        builder.set_node(StructuralNode {
            node_id,
            kind: match node.kind.as_str() {
                "value" => StructuralNodeKind::Calculation,
                _ => StructuralNodeKind::Container,
            },
            symbol: node.node_id.clone(),
            parent_id: Some(root_node_id),
            child_ids: Vec::new(),
            formula_artifact_id: None,
            bind_artifact_id: None,
            constant_value: None,
        });
    }
    builder.replace_children(root_node_id, child_ids)?;
    builder.build(StructuralSnapshotId(1)).map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use crate::contracts::{TraceCalcManifestScenario, load_scenario, validate_scenario};

    use super::*;

    #[test]
    fn engine_machine_executes_accept_publish_scenario() {
        let repo_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let scenario_path = repo_root.join(
            "docs/test-corpus/core-engine/tracecalc/hand-auditable/tc_accept_publish_001.json",
        );
        let scenario = load_scenario(&scenario_path).unwrap();
        let validation = validate_scenario(
            &TraceCalcManifestScenario {
                scenario_id: scenario.scenario_id.clone(),
                path: "hand-auditable/tc_accept_publish_001.json".to_string(),
                focus: Vec::new(),
                tags: Vec::new(),
            },
            &scenario,
        );
        assert!(validation.is_empty());

        let machine = TraceCalcEngineMachine;
        let result = machine.execute(&scenario).unwrap();

        assert_eq!(result.result_state, TraceCalcScenarioResultState::Passed);
        assert!(result.assertion_failures.is_empty());
    }
}
