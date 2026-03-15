using System.Collections.Immutable;

namespace OxCalc.Core.TraceCalc;

internal sealed record TraceCalcCandidate(
    string CandidateResultId,
    string CompatibilityBasis,
    IReadOnlyDictionary<string, string> ValueUpdates,
    IReadOnlyList<TraceCalcDependencyShapeUpdate> DependencyShapeUpdates,
    IReadOnlyList<TraceCalcRuntimeEffect> RuntimeEffects,
    IReadOnlyList<string> Targets,
    IReadOnlyList<string> DiagnosticEvents);

internal sealed record TraceCalcPinnedViewState(
    string ViewId,
    string SnapshotId,
    IReadOnlyList<string> ObservedNodes,
    Dictionary<string, string> Values);

public sealed class TraceCalcReferenceMachine
{
    public TraceCalcExecutionArtifacts Execute(TraceCalcScenario scenario)
    {
        var state = new ReferenceState(scenario);
        foreach (var step in scenario.Steps)
        {
            ExecuteStep(state, scenario, step);
        }

        var assertionFailures = TraceCalcAssertionEvaluator.Evaluate(scenario, state.PublishedValues, state.ActivePinnedViews.Values.Select(ToPinnedRecord), state.TraceEvents, state.Counters, state.Rejects);
        return new TraceCalcExecutionArtifacts(
            scenario.ScenarioId,
            assertionFailures.Count == 0 ? TraceCalcScenarioResultState.Passed : TraceCalcScenarioResultState.FailedAssertion,
            assertionFailures,
            state.TraceEvents,
            state.Counters,
            state.PublishedValues,
            state.ActivePinnedViews.Values.Select(ToPinnedRecord).ToArray(),
            state.Rejects.ToArray());
    }

    private static void ExecuteStep(ReferenceState state, TraceCalcScenario scenario, TraceCalcStep step)
    {
        switch (step.Kind)
        {
            case "pin_view":
                var pinned = new TraceCalcPinnedViewState(
                    step.ViewId ?? throw new InvalidOperationException("pin_view requires view_id."),
                    step.SnapshotId ?? scenario.InitialGraph.SnapshotId,
                    step.ObservedNodes,
                    step.ObservedNodes.ToDictionary(nodeId => nodeId, nodeId => state.PublishedValues.GetValueOrDefault(nodeId, string.Empty), StringComparer.Ordinal));
                state.ActivePinnedViews[pinned.ViewId] = pinned;
                state.IncrementCounter("reader.pinned");
                state.AddEvent(step.StepId, "reader_pinned", new Dictionary<string, string> { ["view_id"] = pinned.ViewId, ["snapshot_id"] = pinned.SnapshotId });
                break;

            case "unpin_view":
                if (!string.IsNullOrEmpty(step.ViewId) && state.ActivePinnedViews.Remove(step.ViewId))
                {
                    state.IncrementCounter("reader.unpinned");
                    state.AddEvent(step.StepId, "reader_unpinned", new Dictionary<string, string> { ["view_id"] = step.ViewId });
                    if (state.OverlayRetainedCount > 0)
                    {
                        state.IncrementCounter("overlay.eviction_eligible");
                        state.AddEvent(step.StepId, "eviction_eligibility_opened", new Dictionary<string, string> { ["view_id"] = step.ViewId });
                    }
                }
                break;

            case "mark_stale":
                foreach (var target in step.Targets)
                {
                    state.IncrementCounter("recalc.mark_dirty");
                    state.AddEvent(step.StepId, "node_marked_dirty", new Dictionary<string, string> { ["node_id"] = target });
                }
                break;

            case "admit_work":
                state.CurrentAdmissionId = step.AdmissionId;
                state.CurrentTargets = step.Targets;
                state.CurrentCompatibilityBasis = step.CompatibilityBasis ?? scenario.InitialGraph.SnapshotId;
                state.IncrementCounter("candidate.admitted");
                state.AddEvent(step.StepId, "candidate_admitted", new Dictionary<string, string>
                {
                    ["admission_id"] = step.AdmissionId ?? string.Empty,
                    ["compatibility_basis"] = state.CurrentCompatibilityBasis,
                });
                break;

            case "emit_candidate_result":
                var candidate = new TraceCalcCandidate(
                    step.CandidateResultId ?? throw new InvalidOperationException("emit_candidate_result requires candidate_result_id."),
                    step.CompatibilityBasis ?? state.CurrentCompatibilityBasis ?? scenario.InitialGraph.SnapshotId,
                    step.ValueUpdates.ToDictionary(entry => entry.NodeId, entry => entry.Value, StringComparer.Ordinal),
                    step.DependencyShapeUpdates,
                    step.RuntimeEffects,
                    state.CurrentTargets,
                    step.DiagnosticEvents);
                state.Candidates[candidate.CandidateResultId] = candidate;
                state.IncrementCounter("candidate.emitted");
                state.AddEvent(step.StepId, "candidate_emitted", new Dictionary<string, string> { ["candidate_result_id"] = candidate.CandidateResultId });
                break;

            case "emit_reject":
                var reject = new TraceCalcRejectRecord(
                    step.RejectId ?? state.CurrentAdmissionId ?? "reject:unknown",
                    step.RejectKind ?? "host_injected_failure",
                    step.RejectDetailText ?? string.Empty);
                state.Rejects.Add(reject);
                state.IncrementCounter("candidate.rejected");
                state.AddEvent(step.StepId, "candidate_rejected", new Dictionary<string, string>
                {
                    ["reject_id"] = reject.RejectId,
                    ["reject_kind"] = reject.RejectKind,
                });
                break;

            case "publish_candidate":
                var publishCandidateId = step.CandidateResultId ?? throw new InvalidOperationException("publish_candidate requires candidate_result_id.");
                if (!state.Candidates.TryGetValue(publishCandidateId, out var publishedCandidate))
                {
                    throw new InvalidOperationException($"Unknown candidate '{publishCandidateId}'.");
                }

                foreach (var update in publishedCandidate.ValueUpdates)
                {
                    state.PublishedValues[update.Key] = update.Value;
                }

                if (publishedCandidate.DependencyShapeUpdates.Count > 0 || publishedCandidate.RuntimeEffects.Count > 0)
                {
                    state.OverlayRetainedCount++;
                    state.IncrementCounter("overlay.retained");
                    state.AddEvent(step.StepId, "overlay_retained", new Dictionary<string, string> { ["candidate_result_id"] = publishCandidateId });
                }

                state.Candidates.Remove(publishCandidateId);
                state.IncrementCounter("candidate.published");
                state.AddEvent(step.StepId, "candidate_published", new Dictionary<string, string>
                {
                    ["candidate_result_id"] = publishCandidateId,
                    ["publication_id"] = step.PublicationId ?? string.Empty,
                });
                break;

            case "verify_clean":
                foreach (var target in step.Targets)
                {
                    state.IncrementCounter("recalc.verified_clean");
                    state.AddEvent(step.StepId, "node_verified_clean", new Dictionary<string, string> { ["node_id"] = target });
                }
                break;

            case "seed_overlay":
                state.OverlayRetainedCount++;
                state.IncrementCounter("overlay.retained");
                state.AddEvent(step.StepId, "overlay_retained", new Dictionary<string, string> { ["owner_node_id"] = step.OwnerNodeId ?? string.Empty });
                break;

            case "reset_fixture":
                state.ActivePinnedViews.Clear();
                state.Candidates.Clear();
                state.Rejects.Clear();
                state.AddEvent(step.StepId, "fixture_reset", new Dictionary<string, string>());
                break;
        }
    }

    private static TraceCalcPinnedViewRecord ToPinnedRecord(TraceCalcPinnedViewState state) =>
        new(state.ViewId, state.SnapshotId, state.Values.ToImmutableDictionary());

    private sealed class ReferenceState
    {
        private int _eventCounter;

        public ReferenceState(TraceCalcScenario scenario)
        {
            PublishedValues = scenario.InitialRuntime.PublishedValues.ToDictionary(entry => entry.NodeId, entry => entry.Value, StringComparer.Ordinal);
            ActivePinnedViews = new Dictionary<string, TraceCalcPinnedViewState>(StringComparer.Ordinal);
            Candidates = new Dictionary<string, TraceCalcCandidate>(StringComparer.Ordinal);
            Rejects = new List<TraceCalcRejectRecord>();
            TraceEvents = new List<TraceCalcTraceEvent>();
            Counters = new Dictionary<string, int>(StringComparer.Ordinal);
            OverlayRetainedCount = scenario.InitialRuntime.SeedOverlays.Count + scenario.InitialRuntime.PublishedRuntimeEffects.Count;
        }

        public Dictionary<string, string> PublishedValues { get; }
        public Dictionary<string, TraceCalcPinnedViewState> ActivePinnedViews { get; }
        public Dictionary<string, TraceCalcCandidate> Candidates { get; }
        public List<TraceCalcRejectRecord> Rejects { get; }
        public List<TraceCalcTraceEvent> TraceEvents { get; }
        public Dictionary<string, int> Counters { get; }
        public string? CurrentAdmissionId { get; set; }
        public IReadOnlyList<string> CurrentTargets { get; set; } = [];
        public string? CurrentCompatibilityBasis { get; set; }
        public int OverlayRetainedCount { get; set; }

        public void IncrementCounter(string counterName) => Counters[counterName] = Counters.GetValueOrDefault(counterName) + 1;

        public void AddEvent(string stepId, string label, IReadOnlyDictionary<string, string> payload)
        {
            _eventCounter++;
            TraceEvents.Add(new TraceCalcTraceEvent($"evt-{_eventCounter:D4}", stepId, label, payload));
        }
    }
}
