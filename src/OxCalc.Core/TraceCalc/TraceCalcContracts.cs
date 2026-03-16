using System.Text.Json;
using System.Text.Json.Nodes;

namespace OxCalc.Core.TraceCalc;

public enum TraceCalcScenarioResultState
{
    Passed = 0,
    FailedAssertion = 1,
    InvalidScenario = 2,
    ExecutionError = 3,
    UnsupportedFeature = 4,
}

public enum TraceCalcValidationFailureKind
{
    JsonParseFailure = 0,
    UnsupportedSchemaVersion = 1,
    MissingRequiredField = 2,
    UnknownStepKind = 3,
    UnknownNodeReference = 4,
    UnknownCandidateReference = 5,
    ManifestMismatch = 6,
    InvalidExpectedShape = 7,
}

public enum TraceCalcConformanceMismatchKind
{
    MissingScenarioResult = 0,
    ResultStateMismatch = 1,
    PublishedViewMismatch = 2,
    PinnedViewMismatch = 3,
    RejectMismatch = 4,
    TraceCountMismatch = 5,
    CounterMismatch = 6,
    UnexpectedExtraArtifact = 7,
}

public sealed record TraceCalcManifest(
    string SchemaVersion,
    string CorpusId,
    string BasePath,
    IReadOnlyList<TraceCalcManifestScenario> Scenarios);

public sealed record TraceCalcManifestScenario(
    string ScenarioId,
    string Path,
    IReadOnlyList<string> Focus,
    IReadOnlyList<string> Tags);

public sealed record TraceCalcScenario(
    string SchemaVersion,
    string ScenarioId,
    string Description,
    string CalcSpace,
    IReadOnlyList<string> Tags,
    IReadOnlyList<string> PackTags,
    TraceCalcInitialGraph InitialGraph,
    TraceCalcInitialRuntime InitialRuntime,
    IReadOnlyList<TraceCalcStep> Steps,
    TraceCalcExpected Expected,
    TraceCalcReplayProjection? ReplayProjection,
    TraceCalcWitnessAnchors? WitnessAnchors,
    JsonObject? Generator,
    IReadOnlyList<string> Notes);

public sealed record TraceCalcReplayProjection(
    IReadOnlyList<string> ReplayClasses,
    IReadOnlyList<string> PackBindings,
    IReadOnlyList<string> RequiredEqualitySurfaces,
    string NormalizedEventFamilyMapRef,
    IReadOnlyList<string> SafetyProperties,
    IReadOnlyList<string> TransitionLabels);

public sealed record TraceCalcWitnessAnchors(
    string ScenarioAnchorId,
    IReadOnlyList<TraceCalcPhaseBlockAnchor> PhaseBlocks,
    IReadOnlyList<TraceCalcEventGroupAnchor> EventGroups,
    IReadOnlyList<TraceCalcRejectRecordAnchor> RejectRecords,
    IReadOnlyList<TraceCalcViewSliceAnchor> ViewSlices);

public sealed record TraceCalcPhaseBlockAnchor(
    string PhaseBlockId,
    IReadOnlyList<string> StepIds);

public sealed record TraceCalcEventGroupAnchor(
    string EventGroupId,
    IReadOnlyList<string> StepIds);

public sealed record TraceCalcRejectRecordAnchor(
    string RejectRecordId,
    string RejectId);

public sealed record TraceCalcViewSliceAnchor(
    string ViewSliceId,
    string ViewKind,
    string? ViewId);

public sealed record TraceCalcInitialGraph(
    string SnapshotId,
    IReadOnlyList<TraceCalcNode> Nodes);

public sealed record TraceCalcNode(
    string NodeId,
    string Kind,
    JsonObject Expression);

public sealed record TraceCalcInitialRuntime(
    IReadOnlyList<TraceCalcPinnedViewExpectation> PinnedViews,
    IReadOnlyList<TraceCalcValueEntry> PublishedValues,
    IReadOnlyList<TraceCalcRuntimeEffect> PublishedRuntimeEffects,
    IReadOnlyList<TraceCalcSeedOverlay> SeedOverlays);

public sealed record TraceCalcSeedOverlay(
    string OverlayKind,
    string OwnerNodeId,
    JsonNode? Payload);

public sealed record TraceCalcStep(
    string StepId,
    string Kind,
    IReadOnlyList<string> Targets,
    string? AdmissionId,
    string? CompatibilityBasis,
    string? CandidateResultId,
    string? PublicationId,
    string? ViewId,
    string? SnapshotId,
    IReadOnlyList<string> ObservedNodes,
    IReadOnlyList<TraceCalcValueEntry> ValueUpdates,
    IReadOnlyList<TraceCalcDependencyShapeUpdate> DependencyShapeUpdates,
    IReadOnlyList<TraceCalcRuntimeEffect> RuntimeEffects,
    IReadOnlyList<string> DiagnosticEvents,
    string? RejectId,
    string? RejectKind,
    string? RejectDetailText,
    string? OverlayKind,
    string? OwnerNodeId,
    JsonNode? Payload);

public sealed record TraceCalcValueEntry(string NodeId, string Value);

public sealed record TraceCalcDependencyShapeUpdate(string NodeId, string Kind, string? DependencyId, JsonNode? Payload);

public sealed record TraceCalcRuntimeEffect(string EffectKind, string OwnerNodeId, JsonNode? Payload);

public sealed record TraceCalcExpected(
    TraceCalcPublishedViewExpectation PublishedView,
    IReadOnlyList<TraceCalcPinnedViewExpectation> PinnedViews,
    IReadOnlyList<TraceCalcTraceLabelExpectation> TraceLabels,
    IReadOnlyList<TraceCalcCounterExpectation> CounterExpectations,
    IReadOnlyList<TraceCalcRejectExpectation> Rejects);

public sealed record TraceCalcPublishedViewExpectation(string SnapshotId, IReadOnlyList<TraceCalcValueEntry> NodeValues);

public sealed record TraceCalcPinnedViewExpectation(string ViewId, string SnapshotId, IReadOnlyList<string> ObservedNodes, IReadOnlyList<TraceCalcValueEntry> NodeValues);

public sealed record TraceCalcTraceLabelExpectation(string Label, int Count);

public sealed record TraceCalcCounterExpectation(string Counter, string Comparison, int Value);

public sealed record TraceCalcRejectExpectation(string RejectId, string RejectKind, string? DetailContains);

public sealed record TraceCalcValidationFailure(TraceCalcValidationFailureKind Kind, string Message);

public sealed record TraceCalcTraceEvent(string EventId, string StepId, string Label, IReadOnlyDictionary<string, string> Payload);

public sealed record TraceCalcRejectRecord(string RejectId, string RejectKind, string RejectDetail);

public sealed record TraceCalcPinnedViewRecord(string ViewId, string SnapshotId, IReadOnlyDictionary<string, string> NodeValues);

public sealed record TraceCalcExecutionArtifacts(
    string ScenarioId,
    TraceCalcScenarioResultState ResultState,
    IReadOnlyList<string> AssertionFailures,
    IReadOnlyList<TraceCalcTraceEvent> TraceEvents,
    IReadOnlyDictionary<string, int> Counters,
    IReadOnlyDictionary<string, string> PublishedValues,
    IReadOnlyList<TraceCalcPinnedViewRecord> PinnedViews,
    IReadOnlyList<TraceCalcRejectRecord> Rejects);

public sealed record TraceCalcConformanceMismatch(TraceCalcConformanceMismatchKind Kind, string Message);

public sealed record TraceCalcScenarioResult(
    string ScenarioId,
    TraceCalcScenarioResultState ResultState,
    IReadOnlyList<TraceCalcValidationFailure> ValidationFailures,
    IReadOnlyList<string> AssertionFailures,
    IReadOnlyList<TraceCalcConformanceMismatch> ConformanceMismatches,
    IReadOnlyDictionary<string, string> ArtifactPaths);

public sealed record TraceCalcRunSummary(
    string RunId,
    string SchemaVersion,
    int ScenarioCount,
    IReadOnlyDictionary<string, int> ResultCounts,
    string ArtifactRoot);

public static class TraceCalcJson
{
    private static readonly HashSet<string> SupportedStepKinds =
    [
        "pin_view",
        "unpin_view",
        "mark_stale",
        "seed_overlay",
        "reset_fixture",
        "admit_work",
        "emit_candidate_result",
        "emit_reject",
        "publish_candidate",
        "verify_clean"
    ];

    public static TraceCalcManifest LoadManifest(string path)
    {
        var root = ParseObject(path);
        var scenarios = root.RequireArray("scenarios")
            .Objects()
            .Select(ParseManifestScenario)
            .ToArray();

        return new TraceCalcManifest(
            root.RequireString("schema_version"),
            root.RequireString("corpus_id"),
            root.RequireString("base_path"),
            scenarios);
    }

    public static TraceCalcScenario LoadScenario(string path)
    {
        var root = ParseObject(path);
        var initialGraph = ParseInitialGraph(root.RequireObject("initial_graph"));
        var initialRuntime = ParseInitialRuntime(root.OptionalObject("initial_runtime"));
        var steps = root.RequireArray("steps")
            .Objects()
            .Select(ParseStep)
            .ToArray();

        return new TraceCalcScenario(
            root.RequireString("schema_version"),
            root.RequireString("scenario_id"),
            root.RequireString("description"),
            root.RequireString("calc_space"),
            root.OptionalStringArray("tags"),
            root.OptionalStringArray("pack_tags"),
            initialGraph,
            initialRuntime,
            steps,
            ParseExpected(root.RequireObject("expected")),
            ParseReplayProjection(root.OptionalObject("replay_projection")),
            ParseWitnessAnchors(root.OptionalObject("witness_anchors")),
            root.OptionalObject("generator"),
            root.OptionalStringArray("notes"));
    }

    public static IReadOnlyList<TraceCalcValidationFailure> ValidateScenario(TraceCalcManifestScenario manifestScenario, TraceCalcScenario scenario)
    {
        var failures = new List<TraceCalcValidationFailure>();
        if (!string.Equals(scenario.SchemaVersion, "tracecalc-s1", StringComparison.Ordinal))
        {
            failures.Add(new TraceCalcValidationFailure(TraceCalcValidationFailureKind.UnsupportedSchemaVersion, $"Unsupported scenario schema version '{scenario.SchemaVersion}'."));
        }

        if (!string.Equals(scenario.CalcSpace, "TraceCalc", StringComparison.Ordinal))
        {
            failures.Add(new TraceCalcValidationFailure(TraceCalcValidationFailureKind.InvalidExpectedShape, $"Unsupported calc_space '{scenario.CalcSpace}'."));
        }

        if (!string.Equals(manifestScenario.ScenarioId, scenario.ScenarioId, StringComparison.Ordinal))
        {
            failures.Add(new TraceCalcValidationFailure(TraceCalcValidationFailureKind.ManifestMismatch, $"Manifest scenario id '{manifestScenario.ScenarioId}' does not match '{scenario.ScenarioId}'."));
        }

        if (scenario.PackTags.Count > 0 && scenario.ReplayProjection is null)
        {
            failures.Add(new TraceCalcValidationFailure(TraceCalcValidationFailureKind.InvalidExpectedShape, $"Replay-facing scenario '{scenario.ScenarioId}' is missing replay_projection metadata."));
        }

        if (scenario.ReplayProjection is not null && scenario.ReplayProjection.ReplayClasses.Count == 0)
        {
            failures.Add(new TraceCalcValidationFailure(TraceCalcValidationFailureKind.InvalidExpectedShape, $"Replay projection for '{scenario.ScenarioId}' must name at least one replay class."));
        }

        var nodeIds = scenario.InitialGraph.Nodes.Select(node => node.NodeId).ToHashSet(StringComparer.Ordinal);
        var candidateIds = new HashSet<string>(StringComparer.Ordinal);
        var admittedIds = new HashSet<string>(StringComparer.Ordinal);
        var viewIds = new HashSet<string>(StringComparer.Ordinal);

        foreach (var step in scenario.Steps)
        {
            if (!SupportedStepKinds.Contains(step.Kind))
            {
                failures.Add(new TraceCalcValidationFailure(TraceCalcValidationFailureKind.UnknownStepKind, $"Unknown step kind '{step.Kind}' in step '{step.StepId}'."));
            }

            foreach (var nodeId in step.Targets.Concat(step.ObservedNodes))
            {
                if (!nodeIds.Contains(nodeId))
                {
                    failures.Add(new TraceCalcValidationFailure(TraceCalcValidationFailureKind.UnknownNodeReference, $"Unknown node reference '{nodeId}' in step '{step.StepId}'."));
                }
            }

            foreach (var update in step.ValueUpdates)
            {
                if (!nodeIds.Contains(update.NodeId))
                {
                    failures.Add(new TraceCalcValidationFailure(TraceCalcValidationFailureKind.UnknownNodeReference, $"Unknown value-update node '{update.NodeId}' in step '{step.StepId}'."));
                }
            }

            foreach (var update in step.DependencyShapeUpdates)
            {
                if (!nodeIds.Contains(update.NodeId) || (update.DependencyId is not null && !nodeIds.Contains(update.DependencyId)))
                {
                    failures.Add(new TraceCalcValidationFailure(TraceCalcValidationFailureKind.UnknownNodeReference, $"Unknown dependency-shape reference in step '{step.StepId}'."));
                }
            }

            foreach (var effect in step.RuntimeEffects)
            {
                if (!nodeIds.Contains(effect.OwnerNodeId))
                {
                    failures.Add(new TraceCalcValidationFailure(TraceCalcValidationFailureKind.UnknownNodeReference, $"Unknown runtime-effect owner '{effect.OwnerNodeId}' in step '{step.StepId}'."));
                }
            }

            if (step.Kind == "admit_work" && !string.IsNullOrEmpty(step.AdmissionId))
            {
                admittedIds.Add(step.AdmissionId);
            }

            if (step.Kind == "emit_candidate_result" && !string.IsNullOrEmpty(step.CandidateResultId))
            {
                candidateIds.Add(step.CandidateResultId);
            }

            if (step.Kind == "publish_candidate" && (string.IsNullOrEmpty(step.CandidateResultId) || !candidateIds.Contains(step.CandidateResultId)))
            {
                failures.Add(new TraceCalcValidationFailure(TraceCalcValidationFailureKind.UnknownCandidateReference, $"Unknown candidate '{step.CandidateResultId}' in publish step '{step.StepId}'."));
            }

            if (step.Kind == "pin_view" && !string.IsNullOrEmpty(step.ViewId))
            {
                viewIds.Add(step.ViewId);
            }

            if (step.Kind == "unpin_view" && (string.IsNullOrEmpty(step.ViewId) || !viewIds.Contains(step.ViewId)))
            {
                failures.Add(new TraceCalcValidationFailure(TraceCalcValidationFailureKind.InvalidExpectedShape, $"Unpin step '{step.StepId}' references unknown view '{step.ViewId}'."));
            }
        }

        foreach (var expectation in scenario.Expected.PinnedViews)
        {
            foreach (var value in expectation.NodeValues)
            {
                if (!nodeIds.Contains(value.NodeId))
                {
                    failures.Add(new TraceCalcValidationFailure(TraceCalcValidationFailureKind.UnknownNodeReference, $"Unknown pinned-view node '{value.NodeId}' in expected section."));
                }
            }
        }

        return failures;
    }

    public static string NormalizeValue(JsonNode? node)
    {
        if (node is null)
        {
            return string.Empty;
        }

        return node.GetValueKind() switch
        {
            JsonValueKind.String => node.GetValue<string>(),
            _ => node.ToJsonString(new JsonSerializerOptions { WriteIndented = false })
                .Trim('"')
        };
    }

    private static TraceCalcManifestScenario ParseManifestScenario(JsonObject obj) =>
        new(
            obj.RequireString("scenario_id"),
            obj.RequireString("path"),
            obj.OptionalStringArray("focus"),
            obj.OptionalStringArray("tags"));

    private static TraceCalcInitialGraph ParseInitialGraph(JsonObject obj) =>
        new(
            obj.RequireString("snapshot_id"),
            obj.RequireArray("nodes").Objects().Select(ParseNode).ToArray());

    private static TraceCalcNode ParseNode(JsonObject obj) =>
        new(
            obj.RequireString("node_id"),
            obj.RequireString("kind"),
            obj.RequireObject("expr"));

    private static TraceCalcInitialRuntime ParseInitialRuntime(JsonObject? obj)
    {
        if (obj is null)
        {
            return new TraceCalcInitialRuntime([], [], [], []);
        }

        return new TraceCalcInitialRuntime(
            obj.OptionalArray("pinned_views").Objects().Select(node => ParsePinnedViewExpectation(node, true)).ToArray(),
            obj.OptionalArray("published_values").Objects().Select(ParseValueEntry).ToArray(),
            obj.OptionalArray("published_runtime_effects").Objects().Select(ParseRuntimeEffect).ToArray(),
            obj.OptionalArray("seed_overlays").Objects().Select(ParseSeedOverlay).ToArray());
    }

    private static TraceCalcReplayProjection? ParseReplayProjection(JsonObject? obj)
    {
        if (obj is null)
        {
            return null;
        }

        return new TraceCalcReplayProjection(
            obj.OptionalStringArray("replay_classes"),
            obj.OptionalStringArray("pack_bindings"),
            obj.OptionalStringArray("required_equality_surfaces"),
            obj.RequireString("normalized_event_family_map_ref"),
            obj.OptionalStringArray("safety_properties"),
            obj.OptionalStringArray("transition_labels"));
    }

    private static TraceCalcWitnessAnchors? ParseWitnessAnchors(JsonObject? obj)
    {
        if (obj is null)
        {
            return null;
        }

        return new TraceCalcWitnessAnchors(
            obj.RequireString("scenario_anchor_id"),
            obj.OptionalArray("phase_blocks").Objects().Select(ParsePhaseBlockAnchor).ToArray(),
            obj.OptionalArray("event_groups").Objects().Select(ParseEventGroupAnchor).ToArray(),
            obj.OptionalArray("reject_records").Objects().Select(ParseRejectRecordAnchor).ToArray(),
            obj.OptionalArray("view_slices").Objects().Select(ParseViewSliceAnchor).ToArray());
    }

    private static TraceCalcSeedOverlay ParseSeedOverlay(JsonObject obj) =>
        new(obj.RequireString("overlay_kind"), obj.RequireString("owner_node_id"), obj["payload"]?.DeepClone());

    private static TraceCalcPhaseBlockAnchor ParsePhaseBlockAnchor(JsonObject obj) =>
        new(obj.RequireString("phase_block_id"), obj.OptionalStringArray("step_ids"));

    private static TraceCalcEventGroupAnchor ParseEventGroupAnchor(JsonObject obj) =>
        new(obj.RequireString("event_group_id"), obj.OptionalStringArray("step_ids"));

    private static TraceCalcRejectRecordAnchor ParseRejectRecordAnchor(JsonObject obj) =>
        new(obj.RequireString("reject_record_id"), obj.RequireString("reject_id"));

    private static TraceCalcViewSliceAnchor ParseViewSliceAnchor(JsonObject obj) =>
        new(obj.RequireString("view_slice_id"), obj.RequireString("view_kind"), obj.OptionalString("view_id"));

    private static TraceCalcStep ParseStep(JsonObject obj) =>
        new(
            obj.RequireString("step_id"),
            obj.RequireString("kind"),
            obj.OptionalStringArray("targets"),
            obj.OptionalString("admission_id"),
            obj.OptionalString("compatibility_basis"),
            obj.OptionalString("candidate_result_id"),
            obj.OptionalString("publication_id"),
            obj.OptionalString("view_id"),
            obj.OptionalString("snapshot_id"),
            obj.OptionalStringArray("observed_nodes"),
            obj.OptionalArray("value_updates").Objects().Select(ParseValueEntry).ToArray(),
            obj.OptionalArray("dependency_shape_updates").Objects().Select(ParseDependencyShapeUpdate).ToArray(),
            obj.OptionalArray("runtime_effects").Objects().Select(ParseRuntimeEffect).ToArray(),
            obj.OptionalStringArray("diagnostic_events"),
            obj.OptionalString("reject_id"),
            obj.OptionalString("reject_kind"),
            obj["reject_detail"]?.ToJsonString(new JsonSerializerOptions { WriteIndented = false }),
            obj.OptionalString("overlay_kind"),
            obj.OptionalString("owner_node_id"),
            obj["payload"]?.DeepClone());

    private static TraceCalcValueEntry ParseValueEntry(JsonObject obj) =>
        new(obj.RequireString("node_id"), NormalizeValue(obj.RequireNode("value")));

    private static TraceCalcDependencyShapeUpdate ParseDependencyShapeUpdate(JsonObject obj) =>
        new(obj.RequireString("node_id"), obj.RequireString("kind"), obj.OptionalString("dep_id"), obj["payload"]?.DeepClone());

    private static TraceCalcRuntimeEffect ParseRuntimeEffect(JsonObject obj) =>
        new(obj.RequireString("effect_kind"), obj.RequireString("owner_node_id"), obj["payload"]?.DeepClone());

    private static TraceCalcExpected ParseExpected(JsonObject obj) =>
        new(
            ParsePublishedViewExpectation(obj.RequireObject("published_view")),
            obj.OptionalArray("pinned_views").Objects().Select(node => ParsePinnedViewExpectation(node, false)).ToArray(),
            obj.OptionalArray("trace_labels").Objects().Select(ParseTraceLabelExpectation).ToArray(),
            obj.OptionalArray("counter_expectations").Objects().Select(ParseCounterExpectation).ToArray(),
            obj.OptionalArray("rejects").Objects().Select(ParseRejectExpectation).ToArray());

    private static TraceCalcPublishedViewExpectation ParsePublishedViewExpectation(JsonObject obj) =>
        new(obj.RequireString("snapshot_id"), obj.OptionalArray("node_values").Objects().Select(ParseValueEntry).ToArray());

    private static TraceCalcPinnedViewExpectation ParsePinnedViewExpectation(JsonObject obj, bool allowObservedNodesOnly) =>
        new(
            obj.RequireString("view_id"),
            obj.RequireString("snapshot_id"),
            obj.OptionalStringArray("observed_nodes"),
            allowObservedNodesOnly ? [] : obj.OptionalArray("node_values").Objects().Select(ParseValueEntry).ToArray());

    private static TraceCalcTraceLabelExpectation ParseTraceLabelExpectation(JsonObject obj) =>
        new(obj.RequireString("label"), obj.RequireInt32("count"));

    private static TraceCalcCounterExpectation ParseCounterExpectation(JsonObject obj) =>
        new(obj.RequireString("counter"), obj.RequireString("comparison"), obj.RequireInt32("value"));

    private static TraceCalcRejectExpectation ParseRejectExpectation(JsonObject obj) =>
        new(obj.RequireString("reject_id"), obj.RequireString("reject_kind"), obj.OptionalString("detail_contains"));

    private static JsonObject ParseObject(string path)
    {
        try
        {
            return JsonNode.Parse(File.ReadAllText(path))?.AsObject()
                ?? throw new InvalidOperationException($"File '{path}' does not contain a JSON object.");
        }
        catch (JsonException ex)
        {
            throw new InvalidOperationException($"Failed to parse JSON file '{path}': {ex.Message}", ex);
        }
    }
}

internal static class TraceCalcJsonExtensions
{
    public static JsonObject RequireObject(this JsonNode node) => node as JsonObject
        ?? throw new InvalidOperationException("Expected JSON object.");

    public static JsonObject RequireObject(this JsonObject obj, string propertyName) => obj.RequireNode(propertyName).RequireObject();

    public static JsonObject? OptionalObject(this JsonObject obj, string propertyName) => obj[propertyName] as JsonObject;

    public static JsonArray RequireArray(this JsonObject obj, string propertyName) => obj.RequireNode(propertyName) as JsonArray
        ?? throw new InvalidOperationException($"Property '{propertyName}' must be an array.");

    public static JsonArray OptionalArray(this JsonObject obj, string propertyName) => obj[propertyName] as JsonArray ?? [];

    public static IEnumerable<JsonObject> Objects(this JsonArray array) =>
        array.Where(static node => node is JsonObject).Select(static node => (JsonObject)node!);

    public static JsonNode RequireNode(this JsonObject obj, string propertyName) => obj[propertyName]
        ?? throw new InvalidOperationException($"Missing required property '{propertyName}'.");

    public static string RequireString(this JsonObject obj, string propertyName) => obj[propertyName]?.GetValue<string>()
        ?? throw new InvalidOperationException($"Missing required string property '{propertyName}'.");

    public static string? OptionalString(this JsonObject obj, string propertyName) => obj[propertyName]?.GetValue<string>();

    public static int RequireInt32(this JsonObject obj, string propertyName) => obj[propertyName]?.GetValue<int>()
        ?? throw new InvalidOperationException($"Missing required integer property '{propertyName}'.");

    public static IReadOnlyList<string> OptionalStringArray(this JsonObject obj, string propertyName) =>
        (obj[propertyName] as JsonArray)?.Select(node => node?.GetValue<string>() ?? string.Empty).Where(value => !string.IsNullOrEmpty(value)).ToArray() ?? [];

    public static JsonValueKind GetValueKind(this JsonNode node)
    {
        using var document = JsonDocument.Parse(node.ToJsonString());
        return document.RootElement.ValueKind;
    }
}
