using System.Collections.Immutable;
using OxCalc.Core.Coordinator;
using OxCalc.Core.Recalc;
using OxCalc.Core.Structural;

namespace OxCalc.Core.TraceCalc;

public sealed class TraceCalcEngineMachine
{
    public TraceCalcExecutionArtifacts Execute(TraceCalcScenario scenario)
    {
        var harness = EngineState.Create(scenario);
        foreach (var step in scenario.Steps)
        {
            ExecuteStep(harness, scenario, step);
        }

        var assertionFailures = TraceCalcAssertionEvaluator.Evaluate(
            scenario,
            harness.ReadPublishedValues(),
            harness.ReadPinnedViews(),
            harness.TraceEvents,
            harness.Counters,
            harness.Rejects);

        return new TraceCalcExecutionArtifacts(
            scenario.ScenarioId,
            assertionFailures.Count == 0 ? TraceCalcScenarioResultState.Passed : TraceCalcScenarioResultState.FailedAssertion,
            assertionFailures,
            harness.TraceEvents,
            harness.Counters,
            harness.ReadPublishedValues(),
            harness.ReadPinnedViews().ToArray(),
            harness.Rejects.ToArray());
    }

    private static void ExecuteStep(EngineState state, TraceCalcScenario scenario, TraceCalcStep step)
    {
        switch (step.Kind)
        {
            case "pin_view":
                PinView(state, scenario, step);
                break;
            case "unpin_view":
                UnpinView(state, step);
                break;
            case "mark_stale":
                MarkStale(state, step);
                break;
            case "admit_work":
                AdmitWork(state, scenario, step);
                break;
            case "emit_candidate_result":
                EmitCandidateResult(state, scenario, step);
                break;
            case "emit_reject":
                EmitReject(state, scenario, step);
                break;
            case "publish_candidate":
                PublishCandidate(state, step);
                break;
            case "verify_clean":
                VerifyClean(state, scenario, step);
                break;
            case "seed_overlay":
                SeedOverlay(state, scenario, step);
                break;
            case "reset_fixture":
                ResetFixture(state, step);
                break;
        }
    }

    private static void PinView(EngineState state, TraceCalcScenario scenario, TraceCalcStep step)
    {
        var pinned = state.Coordinator.PinReader(step.ViewId ?? throw new InvalidOperationException("pin_view requires view_id."));
        state.PinObservedNodes[step.ViewId!] = step.ObservedNodes;
        state.IncrementCounter("reader.pinned");
        state.SetCounter("pinned_reader_count", state.Coordinator.PinnedReaders.Count);
        state.AddEvent(step.StepId, "reader_pinned", new Dictionary<string, string>
        {
            ["view_id"] = pinned.ReaderId,
            ["snapshot_id"] = scenario.InitialGraph.SnapshotId,
        });
    }

    private static void UnpinView(EngineState state, TraceCalcStep step)
    {
        if (string.IsNullOrEmpty(step.ViewId) || !state.Coordinator.UnpinReader(step.ViewId))
        {
            return;
        }

        state.PinObservedNodes.Remove(step.ViewId);
        state.IncrementCounter("reader.unpinned");
        state.IncrementCounter("release_events");
        state.SetCounter("pinned_reader_count", state.Coordinator.PinnedReaders.Count);
        state.AddEvent(step.StepId, "reader_unpinned", new Dictionary<string, string> { ["view_id"] = step.ViewId });

        var evictionOpened = false;
        foreach (var nodeId in state.NodeIdMap.Values)
        {
            if (state.RecalcTracker.GetState(nodeId) is NodeCalcState.Clean or NodeCalcState.VerifiedClean)
            {
                state.RecalcTracker.ReleaseAndEvictEligible(nodeId);
                evictionOpened = true;
            }
        }

        if (evictionOpened)
        {
            state.IncrementCounter("overlay.eviction_eligible");
            state.IncrementCounter("eviction_eligibility_opened");
            state.AddEvent(step.StepId, "eviction_eligibility_opened", new Dictionary<string, string> { ["view_id"] = step.ViewId });
        }

        var evicted = state.RecalcTracker.EvictEligibleOverlays();
        if (evicted > 0)
        {
            state.AddToCounter("overlay_evictions", evicted);
            state.AddEvent(step.StepId, "overlay_released", new Dictionary<string, string> { ["evicted_count"] = evicted.ToString() });
        }
    }

    private static void MarkStale(EngineState state, TraceCalcStep step)
    {
        foreach (var target in step.Targets)
        {
            state.DirtySeeds.Add(target);
            var targetNodeId = state.ResolveNode(target);
            if (state.RecalcTracker.GetState(targetNodeId) == NodeCalcState.Clean)
            {
                state.RecalcTracker.MarkDirty(targetNodeId);
                state.IncrementCounter("recalc.mark_dirty");
                state.IncrementCounter("nodes_marked_dirty");
            }

            state.AddEvent(step.StepId, "node_marked_dirty", new Dictionary<string, string> { ["node_id"] = target });
        }
    }

    private static void AdmitWork(EngineState state, TraceCalcScenario scenario, TraceCalcStep step)
    {
        state.CurrentAdmissionId = step.AdmissionId ?? throw new InvalidOperationException("admit_work requires admission_id.");
        state.CurrentCompatibilityBasis = step.CompatibilityBasis ?? scenario.InitialGraph.SnapshotId;
        state.CurrentPlan = state.Planner.PlanWorkset(step.Targets, state.DirtySeeds);
        state.CurrentTargets = state.CurrentPlan.OrderedNodes;

        EmitPlanArtifacts(state, step.StepId);
        AdvanceScheduledNodesToEvaluating(state, step.StepId, state.CurrentCompatibilityBasis);

        state.Coordinator.AdmitCandidateWork(state.BuildPlaceholderCandidate(state.CurrentAdmissionId, state.CurrentCompatibilityBasis, state.CurrentTargets));
        state.IncrementCounter("candidate.admitted");
        state.IncrementCounter("candidate_admissions");
        state.AddEvent(step.StepId, "candidate_admitted", new Dictionary<string, string>
        {
            ["admission_id"] = state.CurrentAdmissionId,
            ["compatibility_basis"] = state.CurrentCompatibilityBasis,
            ["target_count"] = state.CurrentTargets.Count.ToString(),
        });
    }

    private static void EmitPlanArtifacts(EngineState state, string stepId)
    {
        if (state.CurrentPlan.Groups.Length == 0)
        {
            return;
        }

        for (var index = 0; index < state.CurrentPlan.Groups.Length; index++)
        {
            var group = state.CurrentPlan.Groups[index];
            if (group.Length > 1)
            {
                state.IncrementCounter("cycle_region_groups");
                state.AddEvent(stepId, "cycle_region_detected", new Dictionary<string, string>
                {
                    ["group_index"] = index.ToString(),
                    ["nodes"] = string.Join(",", group),
                });
            }

            state.AddEvent(stepId, group.Length > 1 ? "scc_group_scheduled" : "topo_group_scheduled", new Dictionary<string, string>
            {
                ["group_index"] = index.ToString(),
                ["nodes"] = string.Join(",", group),
            });
        }
    }

    private static void AdvanceScheduledNodesToEvaluating(EngineState state, string stepId, string compatibilityBasis)
    {
        foreach (var nodeIdText in state.CurrentTargets)
        {
            var nodeId = state.ResolveNode(nodeIdText);
            var currentState = state.RecalcTracker.GetState(nodeId);
            if (currentState == NodeCalcState.Clean)
            {
                state.RecalcTracker.MarkDirty(nodeId);
                state.IncrementCounter("recalc.mark_dirty");
                state.IncrementCounter("nodes_marked_dirty");
                state.AddEvent(stepId, "node_marked_dirty", new Dictionary<string, string> { ["node_id"] = nodeIdText });
                currentState = state.RecalcTracker.GetState(nodeId);
            }

            if (currentState == NodeCalcState.RejectedPendingRepair)
            {
                state.RecalcTracker.ReenterRejectedPendingRepair(nodeId);
                state.IncrementCounter("nodes_marked_needed");
                state.AddEvent(stepId, "fallback_reentered", new Dictionary<string, string> { ["node_id"] = nodeIdText });
                currentState = state.RecalcTracker.GetState(nodeId);
            }

            if (currentState == NodeCalcState.DirtyPending)
            {
                state.RecalcTracker.MarkNeeded(nodeId);
                state.IncrementCounter("nodes_marked_needed");
                state.AddEvent(stepId, "node_marked_needed", new Dictionary<string, string> { ["node_id"] = nodeIdText });
                currentState = state.RecalcTracker.GetState(nodeId);
            }

            if (currentState == NodeCalcState.Needed)
            {
                state.RecalcTracker.BeginEvaluate(nodeId, compatibilityBasis);
                state.AddEvent(stepId, "evaluation_started", new Dictionary<string, string>
                {
                    ["node_id"] = nodeIdText,
                    ["compatibility_basis"] = compatibilityBasis,
                });
            }
        }
    }

    private static void EmitCandidateResult(EngineState state, TraceCalcScenario scenario, TraceCalcStep step)
    {
        var compatibilityBasis = step.CompatibilityBasis ?? state.CurrentCompatibilityBasis ?? scenario.InitialGraph.SnapshotId;
        var candidateResultId = step.CandidateResultId ?? throw new InvalidOperationException("emit_candidate_result requires candidate_result_id.");
        foreach (var target in state.CurrentTargets)
        {
            var targetNodeId = state.ResolveNode(target);
            var hasDependencyShapeUpdate = step.DependencyShapeUpdates.Any(update => string.Equals(update.NodeId, target, StringComparison.Ordinal));
            var overlayKind = hasDependencyShapeUpdate ? OverlayKind.DynamicDependency : OverlayKind.CapabilityFenceAttachment;
            var overlayKey = new OverlayKey(targetNodeId, overlayKind, state.Snapshot.SnapshotId, compatibilityBasis, candidateResultId);
            var hadOverlay = state.RecalcTracker.Overlays.TryGetValue(overlayKey, out var existingOverlay);
            state.IncrementCounter("overlay_lookups");
            if (hadOverlay)
            {
                state.IncrementCounter("overlay_hits");
                if (existingOverlay!.IsProtected)
                {
                    state.IncrementCounter("overlay_reuse_after_retention");
                }
            }
            else
            {
                state.IncrementCounter("overlay_misses");
                state.IncrementCounter("overlay_creations");
            }

            if (hasDependencyShapeUpdate)
            {
                state.RecalcTracker.ProduceDependencyShapeUpdate(targetNodeId, compatibilityBasis, candidateResultId);
                state.AddEvent(step.StepId, "candidate_shape_update_produced", new Dictionary<string, string> { ["node_id"] = target });
            }
            else
            {
                state.RecalcTracker.ProduceCandidateResult(targetNodeId, compatibilityBasis, candidateResultId);
            }
        }

        var candidate = state.BuildCandidate(step, compatibilityBasis);
        state.Coordinator.AdmitCandidateWork(candidate);
        state.Coordinator.RecordAcceptedCandidateResult(candidate.CandidateResultId);
        state.IncrementCounter("candidate.emitted");
        state.IncrementCounter("accepted_candidate_results");
        state.AddEvent(step.StepId, "candidate_emitted", new Dictionary<string, string>
        {
            ["candidate_result_id"] = candidate.CandidateResultId,
            ["target_count"] = candidate.TargetSet.Length.ToString(),
        });
    }

    private static void EmitReject(EngineState state, TraceCalcScenario scenario, TraceCalcStep step)
    {
        var rejectKind = step.RejectKind ?? "host_injected_failure";
        var compatibilityBasis = state.CurrentCompatibilityBasis ?? scenario.InitialGraph.SnapshotId;
        if (state.CurrentTargets.Count == 0)
        {
            state.CurrentPlan = state.Planner.PlanWorkset(step.Targets, state.DirtySeeds);
            state.CurrentTargets = state.CurrentPlan.OrderedNodes;
            AdvanceScheduledNodesToEvaluating(state, step.StepId, compatibilityBasis);
        }

        foreach (var target in state.CurrentTargets)
        {
            state.RecalcTracker.RejectOrFallback(state.ResolveNode(target), rejectKind);
        }

        var reject = state.Coordinator.RejectCandidateWork(state.CurrentAdmissionId ?? step.RejectId ?? "reject:unknown", MapRejectKind(rejectKind), step.RejectDetailText ?? string.Empty);
        state.Rejects.Add(new TraceCalcRejectRecord(step.RejectId ?? reject.CandidateResultId, rejectKind, reject.Detail));
        state.IncrementCounter("candidate.rejected");
        state.IncrementCounter("abandoned_candidates");
        state.IncrementCounter($"rejects_by_class.{rejectKind}");
        state.IncrementCounter($"fallback_by_reason.{rejectKind}");
        state.AddToCounter("fallback_affected_work_volume", state.CurrentTargets.Count);
        state.AddEvent(step.StepId, "candidate_rejected", new Dictionary<string, string>
        {
            ["reject_id"] = step.RejectId ?? reject.CandidateResultId,
            ["reject_kind"] = rejectKind,
            ["target_count"] = state.CurrentTargets.Count.ToString(),
        });
    }

    private static void PublishCandidate(EngineState state, TraceCalcStep step)
    {
        var publication = state.Coordinator.AcceptAndPublish(step.PublicationId ?? "publication:unknown");
        foreach (var target in state.CurrentTargets)
        {
            var nodeId = state.ResolveNode(target);
            state.RecalcTracker.PublishAndClear(nodeId);
            state.DirtySeeds.Remove(target);
        }

        var retained = state.RecalcTracker.Overlays.Values.Count(entry => entry.Key.OverlayKind == OverlayKind.DynamicDependency && entry.IsProtected);
        if (retained > 0)
        {
            state.SetCounter("overlay.retained", retained);
            if (state.Coordinator.PinnedReaders.Count > 0)
            {
                state.IncrementCounter("retention_blocked_cleanup");
            }

            state.AddEvent(step.StepId, "overlay_retained", new Dictionary<string, string> { ["publication_id"] = publication.PublicationId });
        }

        state.IncrementCounter("candidate.published");
        state.IncrementCounter("publications_committed");
        state.AddEvent(step.StepId, "candidate_published", new Dictionary<string, string>
        {
            ["candidate_result_id"] = publication.CandidateResultId,
            ["publication_id"] = publication.PublicationId,
        });
        state.ClearCurrentWork();
    }

    private static void VerifyClean(EngineState state, TraceCalcScenario scenario, TraceCalcStep step)
    {
        var compatibilityBasis = state.CurrentCompatibilityBasis ?? scenario.InitialGraph.SnapshotId;
        if (state.CurrentTargets.Count == 0)
        {
            state.CurrentPlan = state.Planner.PlanWorkset(step.Targets, state.DirtySeeds);
            state.CurrentTargets = state.CurrentPlan.OrderedNodes;
            AdvanceScheduledNodesToEvaluating(state, step.StepId, compatibilityBasis);
        }

        foreach (var target in state.CurrentTargets)
        {
            var targetNodeId = state.ResolveNode(target);
            state.RecalcTracker.VerifyClean(targetNodeId);
            state.DirtySeeds.Remove(target);
            state.IncrementCounter("recalc.verified_clean");
            state.IncrementCounter("verified_clean_nodes");
            state.AddEvent(step.StepId, "node_verified_clean", new Dictionary<string, string> { ["node_id"] = target });
        }

        state.ClearCurrentWork();
    }

    private static void SeedOverlay(EngineState state, TraceCalcScenario scenario, TraceCalcStep step)
    {
        var compatibilityBasis = state.CurrentCompatibilityBasis ?? scenario.InitialGraph.SnapshotId;
        var ownerNodeId = step.OwnerNodeId ?? throw new InvalidOperationException("seed_overlay requires owner_node_id.");
        var overlayTarget = state.ResolveNode(ownerNodeId);
        state.RecalcTracker.MarkDirty(overlayTarget);
        state.RecalcTracker.MarkNeeded(overlayTarget);
        state.RecalcTracker.BeginEvaluate(overlayTarget, compatibilityBasis);
        state.RecalcTracker.ProduceDependencyShapeUpdate(overlayTarget, compatibilityBasis, "seed_overlay");
        state.IncrementCounter("overlay.retained");
        state.IncrementCounter("overlay_creations");
        state.AddEvent(step.StepId, "overlay_retained", new Dictionary<string, string> { ["owner_node_id"] = ownerNodeId });
    }

    private static void ResetFixture(EngineState state, TraceCalcStep step)
    {
        state.PinObservedNodes.Clear();
        state.Rejects.Clear();
        state.CurrentTargets = [];
        state.CurrentPlan = new TraceCalcWorksetPlan([], [], [], []);
        state.DirtySeeds.Clear();
        state.AddEvent(step.StepId, "fixture_reset", new Dictionary<string, string>());
    }

    private static RejectKind MapRejectKind(string? rejectKind) => rejectKind switch
    {
        "snapshot_mismatch" => RejectKind.SnapshotMismatch,
        "artifact_token_mismatch" => RejectKind.ArtifactTokenMismatch,
        "profile_version_mismatch" => RejectKind.ProfileVersionMismatch,
        "capability_mismatch" => RejectKind.CapabilityMismatch,
        "publication_fence_mismatch" => RejectKind.PublicationFenceMismatch,
        "dynamic_dependency_failure" => RejectKind.DynamicDependencyFailure,
        "synthetic_cycle_reject" => RejectKind.SyntheticCycleReject,
        _ => RejectKind.HostInjectedFailure,
    };

    private sealed class EngineState
    {
        private int _eventCounter;

        private EngineState(
            StructuralSnapshot snapshot,
            TreeCalcCoordinator coordinator,
            Stage1RecalcTracker recalcTracker,
            TraceCalcScenarioPlanner planner,
            Dictionary<string, TreeNodeId> nodeIdMap,
            Dictionary<string, IReadOnlyList<string>> pinObservedNodes,
            Dictionary<string, int> counters,
            List<TraceCalcTraceEvent> traceEvents,
            List<TraceCalcRejectRecord> rejects)
        {
            Snapshot = snapshot;
            Coordinator = coordinator;
            RecalcTracker = recalcTracker;
            Planner = planner;
            NodeIdMap = nodeIdMap;
            PinObservedNodes = pinObservedNodes;
            Counters = counters;
            TraceEvents = traceEvents;
            Rejects = rejects;
            DirtySeeds = new HashSet<string>(StringComparer.Ordinal);
            CurrentPlan = new TraceCalcWorksetPlan([], [], [], []);
        }

        public StructuralSnapshot Snapshot { get; }
        public TreeCalcCoordinator Coordinator { get; }
        public Stage1RecalcTracker RecalcTracker { get; }
        public TraceCalcScenarioPlanner Planner { get; }
        public Dictionary<string, TreeNodeId> NodeIdMap { get; }
        public Dictionary<string, IReadOnlyList<string>> PinObservedNodes { get; }
        public Dictionary<string, int> Counters { get; }
        public List<TraceCalcTraceEvent> TraceEvents { get; }
        public List<TraceCalcRejectRecord> Rejects { get; }
        public HashSet<string> DirtySeeds { get; }
        public string? CurrentAdmissionId { get; set; }
        public IReadOnlyList<string> CurrentTargets { get; set; } = [];
        public string? CurrentCompatibilityBasis { get; set; }
        public TraceCalcWorksetPlan CurrentPlan { get; set; }

        public static EngineState Create(TraceCalcScenario scenario)
        {
            var builder = new StructuralSnapshotBuilder();
            var nodeIdMap = new Dictionary<string, TreeNodeId>(StringComparer.Ordinal);
            var rootNodeId = new TreeNodeId(1);
            builder.SetNode(new StructuralNode(rootNodeId, StructuralNodeKind.Root, "root", null, ImmutableArray<TreeNodeId>.Empty));
            builder.SetRoot(rootNodeId);
            long nextNodeId = 2;
            var childIds = new List<TreeNodeId>();
            foreach (var node in scenario.InitialGraph.Nodes)
            {
                var nodeId = new TreeNodeId(nextNodeId++);
                nodeIdMap[node.NodeId] = nodeId;
                builder.SetNode(new StructuralNode(nodeId, StructuralNodeKind.Calculation, node.NodeId, rootNodeId, ImmutableArray<TreeNodeId>.Empty));
                childIds.Add(nodeId);
            }

            builder.ReplaceChildren(rootNodeId, childIds);
            var snapshot = builder.Build(new StructuralSnapshotId(1));
            var coordinator = new TreeCalcCoordinator(snapshot);
            var initialValues = scenario.InitialRuntime.PublishedValues.ToDictionary(entry => nodeIdMap[entry.NodeId], entry => entry.Value);
            coordinator.SeedPublishedView(initialValues, scenario.InitialRuntime.PublishedValues.Count > 0 ? "publication:seed" : null, []);
            var recalcTracker = new Stage1RecalcTracker(snapshot);
            var planner = new TraceCalcScenarioPlanner(scenario);
            return new EngineState(
                snapshot,
                coordinator,
                recalcTracker,
                planner,
                nodeIdMap,
                new Dictionary<string, IReadOnlyList<string>>(StringComparer.Ordinal),
                new Dictionary<string, int>(StringComparer.Ordinal),
                [],
                []);
        }

        public TreeNodeId ResolveNode(string nodeId) => NodeIdMap.TryGetValue(nodeId, out var resolved)
            ? resolved
            : throw new KeyNotFoundException($"Unknown node '{nodeId}'.");

        public void IncrementCounter(string counterName) => Counters[counterName] = Counters.GetValueOrDefault(counterName) + 1;

        public void AddToCounter(string counterName, int delta) => Counters[counterName] = Counters.GetValueOrDefault(counterName) + delta;

        public void SetCounter(string counterName, int value) => Counters[counterName] = value;

        public void AddEvent(string stepId, string label, IReadOnlyDictionary<string, string> payload)
        {
            _eventCounter++;
            TraceEvents.Add(new TraceCalcTraceEvent($"evt-{_eventCounter:D4}", stepId, label, payload));
        }

        public void ClearCurrentWork()
        {
            CurrentAdmissionId = null;
            CurrentCompatibilityBasis = null;
            CurrentTargets = [];
            CurrentPlan = new TraceCalcWorksetPlan([], [], [], []);
        }

        public AcceptedCandidateResult BuildPlaceholderCandidate(string candidateResultId, string compatibilityBasis, IReadOnlyList<string> targets) =>
            new(
                candidateResultId,
                Snapshot.SnapshotId,
                "artifact:placeholder",
                compatibilityBasis,
                targets.Select(ResolveNode).ToImmutableArray(),
                ImmutableDictionary<TreeNodeId, string>.Empty,
                ImmutableArray<DependencyShapeUpdate>.Empty,
                ImmutableArray<RuntimeEffect>.Empty,
                ["candidate_recorded"]);

        public AcceptedCandidateResult BuildCandidate(TraceCalcStep step, string compatibilityBasis) =>
            new(
                step.CandidateResultId ?? throw new InvalidOperationException("emit_candidate_result requires candidate_result_id."),
                Snapshot.SnapshotId,
                "artifact:tracecalc",
                compatibilityBasis,
                CurrentTargets.Select(ResolveNode).ToImmutableArray(),
                step.ValueUpdates.ToImmutableDictionary(entry => ResolveNode(entry.NodeId), entry => entry.Value),
                step.DependencyShapeUpdates.Select(update => new DependencyShapeUpdate(
                    update.Kind,
                    update.DependencyId is null ? ImmutableArray<TreeNodeId>.Empty : [ResolveNode(update.DependencyId)])).ToImmutableArray(),
                step.RuntimeEffects.Select(effect => new RuntimeEffect(effect.EffectKind, effect.Payload?.ToJsonString() ?? string.Empty)).ToImmutableArray(),
                step.DiagnosticEvents.ToImmutableArray());

        public Dictionary<string, string> ReadPublishedValues() => Coordinator.PublishedView.Values.ToDictionary(
            pair => NodeIdMap.Single(kvp => kvp.Value == pair.Key).Key,
            pair => pair.Value,
            StringComparer.Ordinal);

        public IReadOnlyList<TraceCalcPinnedViewRecord> ReadPinnedViews() => Coordinator.PinnedReaders.Select(reader =>
        {
            var observedNodes = PinObservedNodes.GetValueOrDefault(reader.ReaderId) ?? [];
            var values = observedNodes.ToDictionary(nodeId => nodeId, nodeId => reader.Values.GetValueOrDefault(ResolveNode(nodeId), string.Empty), StringComparer.Ordinal);
            return new TraceCalcPinnedViewRecord(reader.ReaderId, "s0", values);
        }).ToArray();
    }
}
