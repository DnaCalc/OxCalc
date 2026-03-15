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

        var assertionFailures = TraceCalcAssertionEvaluator.Evaluate(scenario, harness.ReadPublishedValues(), harness.ReadPinnedViews(), harness.TraceEvents, harness.Counters, harness.Rejects);
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
                var pinned = state.Coordinator.PinReader(step.ViewId ?? throw new InvalidOperationException("pin_view requires view_id."));
                state.PinObservedNodes[step.ViewId!] = step.ObservedNodes;
                state.IncrementCounter("reader.pinned");
                state.AddEvent(step.StepId, "reader_pinned", new Dictionary<string, string> { ["view_id"] = pinned.ReaderId, ["snapshot_id"] = scenario.InitialGraph.SnapshotId });
                break;

            case "unpin_view":
                if (!string.IsNullOrEmpty(step.ViewId) && state.Coordinator.UnpinReader(step.ViewId))
                {
                    state.PinObservedNodes.Remove(step.ViewId);
                    state.IncrementCounter("reader.unpinned");
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
                        state.AddEvent(step.StepId, "eviction_eligibility_opened", new Dictionary<string, string> { ["view_id"] = step.ViewId });
                    }
                }
                break;

            case "mark_stale":
                foreach (var target in step.Targets)
                {
                    var targetNodeId = state.ResolveNode(target);
                    if (state.RecalcTracker.GetState(targetNodeId) != NodeCalcState.DirtyPending)
                    {
                        state.RecalcTracker.MarkDirty(targetNodeId);
                    }

                    state.IncrementCounter("recalc.mark_dirty");
                    state.AddEvent(step.StepId, "node_marked_dirty", new Dictionary<string, string> { ["node_id"] = target });
                }
                break;

            case "admit_work":
                state.CurrentAdmissionId = step.AdmissionId ?? throw new InvalidOperationException("admit_work requires admission_id.");
                state.CurrentTargets = step.Targets;
                state.CurrentCompatibilityBasis = step.CompatibilityBasis ?? scenario.InitialGraph.SnapshotId;
                foreach (var target in step.Targets)
                {
                    var targetNodeId = state.ResolveNode(target);
                    if (state.RecalcTracker.GetState(targetNodeId) == NodeCalcState.Clean)
                    {
                        state.RecalcTracker.MarkDirty(targetNodeId);
                    }

                    if (state.RecalcTracker.GetState(targetNodeId) == NodeCalcState.DirtyPending)
                    {
                        state.RecalcTracker.MarkNeeded(targetNodeId);
                    }

                    if (state.RecalcTracker.GetState(targetNodeId) == NodeCalcState.Needed)
                    {
                        state.RecalcTracker.BeginEvaluate(targetNodeId, state.CurrentCompatibilityBasis);
                    }
                }

                state.Coordinator.AdmitCandidateWork(state.BuildPlaceholderCandidate(state.CurrentAdmissionId, state.CurrentCompatibilityBasis, step.Targets));
                state.IncrementCounter("candidate.admitted");
                state.AddEvent(step.StepId, "candidate_admitted", new Dictionary<string, string>
                {
                    ["admission_id"] = state.CurrentAdmissionId,
                    ["compatibility_basis"] = state.CurrentCompatibilityBasis,
                });
                break;

            case "emit_candidate_result":
                var compatibilityBasis = step.CompatibilityBasis ?? state.CurrentCompatibilityBasis ?? scenario.InitialGraph.SnapshotId;
                foreach (var target in state.CurrentTargets)
                {
                    var targetNodeId = state.ResolveNode(target);
                    if (step.DependencyShapeUpdates.Any(update => string.Equals(update.NodeId, target, StringComparison.Ordinal)))
                    {
                        state.RecalcTracker.ProduceDependencyShapeUpdate(targetNodeId, compatibilityBasis, step.CandidateResultId ?? "candidate:unknown");
                    }
                    else
                    {
                        state.RecalcTracker.ProduceCandidateResult(targetNodeId, compatibilityBasis, step.CandidateResultId ?? "candidate:unknown");
                    }
                }

                var candidate = state.BuildCandidate(step, compatibilityBasis);
                state.Coordinator.AdmitCandidateWork(candidate);
                state.Coordinator.RecordAcceptedCandidateResult(candidate.CandidateResultId);
                state.IncrementCounter("candidate.emitted");
                state.AddEvent(step.StepId, "candidate_emitted", new Dictionary<string, string> { ["candidate_result_id"] = candidate.CandidateResultId });
                break;

            case "emit_reject":
                foreach (var target in state.CurrentTargets)
                {
                    var targetNodeId = state.ResolveNode(target);
                    if (state.RecalcTracker.GetState(targetNodeId) == NodeCalcState.Clean)
                    {
                        state.RecalcTracker.MarkDirty(targetNodeId);
                        state.RecalcTracker.MarkNeeded(targetNodeId);
                        state.RecalcTracker.BeginEvaluate(targetNodeId, state.CurrentCompatibilityBasis ?? scenario.InitialGraph.SnapshotId);
                    }

                    state.RecalcTracker.RejectOrFallback(targetNodeId, step.RejectKind ?? "host_injected_failure");
                }

                var reject = state.Coordinator.RejectCandidateWork(state.CurrentAdmissionId ?? step.RejectId ?? "reject:unknown", MapRejectKind(step.RejectKind), step.RejectDetailText ?? string.Empty);
                state.Rejects.Add(new TraceCalcRejectRecord(step.RejectId ?? reject.CandidateResultId, step.RejectKind ?? reject.Kind.ToString().ToSnakeCase(), reject.Detail));
                state.IncrementCounter("candidate.rejected");
                state.AddEvent(step.StepId, "candidate_rejected", new Dictionary<string, string>
                {
                    ["reject_id"] = step.RejectId ?? reject.CandidateResultId,
                    ["reject_kind"] = step.RejectKind ?? reject.Kind.ToString().ToSnakeCase(),
                });
                break;

            case "publish_candidate":
                var publication = state.Coordinator.AcceptAndPublish(step.PublicationId ?? "publication:unknown");
                foreach (var target in state.CurrentTargets)
                {
                    state.RecalcTracker.PublishAndClear(state.ResolveNode(target));
                }

                var retained = state.RecalcTracker.Overlays.Values.Count(entry => entry.Key.OverlayKind == OverlayKind.DynamicDependency && entry.IsProtected);
                if (retained > 0)
                {
                    state.Counters["overlay.retained"] = retained;
                    state.AddEvent(step.StepId, "overlay_retained", new Dictionary<string, string> { ["publication_id"] = publication.PublicationId });
                }

                state.IncrementCounter("candidate.published");
                state.AddEvent(step.StepId, "candidate_published", new Dictionary<string, string>
                {
                    ["candidate_result_id"] = publication.CandidateResultId,
                    ["publication_id"] = publication.PublicationId,
                });
                break;

            case "verify_clean":
                foreach (var target in step.Targets)
                {
                    var targetNodeId = state.ResolveNode(target);
                    if (state.RecalcTracker.GetState(targetNodeId) == NodeCalcState.Clean)
                    {
                        state.RecalcTracker.MarkDirty(targetNodeId);
                        state.RecalcTracker.MarkNeeded(targetNodeId);
                        state.RecalcTracker.BeginEvaluate(targetNodeId, state.CurrentCompatibilityBasis ?? scenario.InitialGraph.SnapshotId);
                    }

                    state.RecalcTracker.VerifyClean(targetNodeId);
                    state.IncrementCounter("recalc.verified_clean");
                    state.AddEvent(step.StepId, "node_verified_clean", new Dictionary<string, string> { ["node_id"] = target });
                }
                break;

            case "seed_overlay":
                var overlayTarget = state.ResolveNode(step.OwnerNodeId ?? throw new InvalidOperationException("seed_overlay requires owner_node_id."));
                state.RecalcTracker.MarkDirty(overlayTarget);
                state.RecalcTracker.MarkNeeded(overlayTarget);
                state.RecalcTracker.BeginEvaluate(overlayTarget, state.CurrentCompatibilityBasis ?? scenario.InitialGraph.SnapshotId);
                state.RecalcTracker.ProduceDependencyShapeUpdate(overlayTarget, state.CurrentCompatibilityBasis ?? scenario.InitialGraph.SnapshotId, "seed_overlay");
                state.IncrementCounter("overlay.retained");
                state.AddEvent(step.StepId, "overlay_retained", new Dictionary<string, string> { ["owner_node_id"] = step.OwnerNodeId ?? string.Empty });
                break;

            case "reset_fixture":
                state.PinObservedNodes.Clear();
                state.Rejects.Clear();
                state.AddEvent(step.StepId, "fixture_reset", new Dictionary<string, string>());
                break;
        }
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
            Dictionary<string, TreeNodeId> nodeIdMap,
            Dictionary<string, IReadOnlyList<string>> pinObservedNodes,
            Dictionary<string, int> counters,
            List<TraceCalcTraceEvent> traceEvents,
            List<TraceCalcRejectRecord> rejects)
        {
            Snapshot = snapshot;
            Coordinator = coordinator;
            RecalcTracker = recalcTracker;
            NodeIdMap = nodeIdMap;
            PinObservedNodes = pinObservedNodes;
            Counters = counters;
            TraceEvents = traceEvents;
            Rejects = rejects;
        }

        public StructuralSnapshot Snapshot { get; }
        public TreeCalcCoordinator Coordinator { get; }
        public Stage1RecalcTracker RecalcTracker { get; }
        public Dictionary<string, TreeNodeId> NodeIdMap { get; }
        public Dictionary<string, IReadOnlyList<string>> PinObservedNodes { get; }
        public Dictionary<string, int> Counters { get; }
        public List<TraceCalcTraceEvent> TraceEvents { get; }
        public List<TraceCalcRejectRecord> Rejects { get; }
        public string? CurrentAdmissionId { get; set; }
        public IReadOnlyList<string> CurrentTargets { get; set; } = [];
        public string? CurrentCompatibilityBasis { get; set; }

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
            return new EngineState(snapshot, coordinator, recalcTracker, nodeIdMap, new Dictionary<string, IReadOnlyList<string>>(StringComparer.Ordinal), new Dictionary<string, int>(StringComparer.Ordinal), new List<TraceCalcTraceEvent>(), new List<TraceCalcRejectRecord>());
        }

        public TreeNodeId ResolveNode(string nodeId) => NodeIdMap.TryGetValue(nodeId, out var resolved)
            ? resolved
            : throw new KeyNotFoundException($"Unknown node '{nodeId}'.");

        public void IncrementCounter(string counterName) => Counters[counterName] = Counters.GetValueOrDefault(counterName) + 1;

        public void AddEvent(string stepId, string label, IReadOnlyDictionary<string, string> payload)
        {
            _eventCounter++;
            TraceEvents.Add(new TraceCalcTraceEvent($"evt-{_eventCounter:D4}", stepId, label, payload));
        }

        public AcceptedCandidateResult BuildPlaceholderCandidate(string candidateResultId, string compatibilityBasis, IReadOnlyList<string> targets) =>
            new(candidateResultId, Snapshot.SnapshotId, "artifact:placeholder", compatibilityBasis, targets.Select(ResolveNode).ToImmutableArray(), ImmutableDictionary<TreeNodeId, string>.Empty, ImmutableArray<DependencyShapeUpdate>.Empty, ImmutableArray<RuntimeEffect>.Empty, ImmutableArray<string>.Empty);

        public AcceptedCandidateResult BuildCandidate(TraceCalcStep step, string compatibilityBasis) =>
            new(
                step.CandidateResultId ?? throw new InvalidOperationException("emit_candidate_result requires candidate_result_id."),
                Snapshot.SnapshotId,
                "artifact:tracecalc",
                compatibilityBasis,
                CurrentTargets.Select(ResolveNode).ToImmutableArray(),
                step.ValueUpdates.ToImmutableDictionary(entry => ResolveNode(entry.NodeId), entry => entry.Value),
                step.DependencyShapeUpdates.Select(update => new DependencyShapeUpdate(update.Kind, update.DependencyId is null ? ImmutableArray<TreeNodeId>.Empty : [ResolveNode(update.DependencyId)])).ToImmutableArray(),
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
