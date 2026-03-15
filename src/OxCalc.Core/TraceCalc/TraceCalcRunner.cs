using System.Text.Json;

namespace OxCalc.Core.TraceCalc;

public sealed class TraceCalcRunner
{
    private static readonly JsonSerializerOptions JsonOptions = new()
    {
        WriteIndented = true,
    };

    private readonly TraceCalcReferenceMachine _referenceMachine = new();
    private readonly TraceCalcEngineMachine _engineMachine = new();

    public TraceCalcRunSummary ExecuteManifest(string repoRoot, string runId, string? scenarioId = null, IReadOnlyList<string>? tags = null)
    {
        var manifestPath = Path.Combine(repoRoot, "docs", "test-corpus", "core-engine", "tracecalc", "MANIFEST.json");
        var manifest = TraceCalcJson.LoadManifest(manifestPath);
        var selectedScenarios = manifest.Scenarios
            .Where(entry => scenarioId is null || string.Equals(entry.ScenarioId, scenarioId, StringComparison.Ordinal))
            .Where(entry => tags is null || tags.Count == 0 || tags.Any(tag => entry.Tags.Contains(tag, StringComparer.Ordinal)))
            .ToArray();

        var artifactRoot = Path.Combine(repoRoot, "docs", "test-runs", "core-engine", "tracecalc-reference-machine", runId);
        var relativeArtifactRoot = RelativeArtifactPath("docs", "test-runs", "core-engine", "tracecalc-reference-machine", runId);
        if (Directory.Exists(artifactRoot))
        {
            Directory.Delete(artifactRoot, recursive: true);
        }

        Directory.CreateDirectory(artifactRoot);
        Directory.CreateDirectory(Path.Combine(artifactRoot, "scenarios"));
        Directory.CreateDirectory(Path.Combine(artifactRoot, "conformance"));

        File.WriteAllText(
            Path.Combine(artifactRoot, "manifest_selection.json"),
            JsonSerializer.Serialize(
                selectedScenarios.Select(entry => new
                {
                    scenario_id = entry.ScenarioId,
                    path = entry.Path,
                    focus = entry.Focus,
                    tags = entry.Tags,
                }),
                JsonOptions));

        var scenarioResults = new List<TraceCalcScenarioResult>();
        var oracleBaseline = new List<object>();
        var engineDiff = new List<object>();

        foreach (var entry in selectedScenarios)
        {
            var scenarioDirectory = Path.Combine(artifactRoot, "scenarios", entry.ScenarioId);
            Directory.CreateDirectory(scenarioDirectory);
            var scenarioPath = Path.Combine(repoRoot, "docs", "test-corpus", "core-engine", "tracecalc", entry.Path.Replace('/', Path.DirectorySeparatorChar));
            TraceCalcExecutionArtifacts oracleArtifacts;
            TraceCalcExecutionArtifacts engineArtifacts;
            IReadOnlyList<TraceCalcValidationFailure> validationFailures = [];
            IReadOnlyList<TraceCalcConformanceMismatch> conformanceMismatches;
            TraceCalcScenarioResultState resultState;
            List<string> assertionFailures = [];

            try
            {
                var scenario = TraceCalcJson.LoadScenario(scenarioPath);
                validationFailures = TraceCalcJson.ValidateScenario(entry, scenario);

                if (validationFailures.Count > 0)
                {
                    oracleArtifacts = CreateEmptyArtifacts(entry.ScenarioId, TraceCalcScenarioResultState.InvalidScenario);
                    engineArtifacts = oracleArtifacts;
                    conformanceMismatches = [];
                    resultState = TraceCalcScenarioResultState.InvalidScenario;
                }
                else
                {
                    oracleArtifacts = _referenceMachine.Execute(scenario);
                    engineArtifacts = _engineMachine.Execute(scenario);
                    conformanceMismatches = TraceCalcConformanceComparer.Compare(oracleArtifacts, engineArtifacts);
                    assertionFailures.AddRange(oracleArtifacts.AssertionFailures);
                    assertionFailures.AddRange(engineArtifacts.AssertionFailures.Select(message => $"engine: {message}"));
                    resultState = assertionFailures.Count == 0 && conformanceMismatches.Count == 0
                        ? TraceCalcScenarioResultState.Passed
                        : TraceCalcScenarioResultState.FailedAssertion;
                }

                WriteScenarioArtifacts(scenarioDirectory, runId, scenario, entry.ScenarioId, resultState, validationFailures, assertionFailures, conformanceMismatches, oracleArtifacts, relativeArtifactRoot);

                oracleBaseline.Add(new
                {
                    scenario_id = entry.ScenarioId,
                    result_state = ToSnakeCase(oracleArtifacts.ResultState),
                    published_values = ToValueEntries(oracleArtifacts.PublishedValues),
                    pinned_views = oracleArtifacts.PinnedViews.Select(ToPinnedViewObject).ToArray(),
                    counters = ToCounterEntries(oracleArtifacts.Counters),
                    rejects = oracleArtifacts.Rejects.Select(ToRejectObject).ToArray(),
                });

                engineDiff.Add(new
                {
                    scenario_id = entry.ScenarioId,
                    oracle_result_state = ToSnakeCase(oracleArtifacts.ResultState),
                    engine_result_state = ToSnakeCase(engineArtifacts.ResultState),
                    mismatches = conformanceMismatches.Select(ToMismatchObject).ToArray(),
                });
            }
            catch (Exception ex) when (ex is InvalidOperationException or IOException or UnauthorizedAccessException)
            {
                oracleArtifacts = CreateEmptyArtifacts(entry.ScenarioId, TraceCalcScenarioResultState.ExecutionError);
                engineArtifacts = oracleArtifacts;
                conformanceMismatches = [];
                assertionFailures.Add(ex.Message);
                resultState = TraceCalcScenarioResultState.ExecutionError;

                WriteScenarioArtifacts(
                    scenarioDirectory,
                    runId,
                    scenario: null,
                    entry.ScenarioId,
                    resultState,
                    [new TraceCalcValidationFailure(TraceCalcValidationFailureKind.JsonParseFailure, ex.Message)],
                    assertionFailures,
                    conformanceMismatches,
                    oracleArtifacts,
                    relativeArtifactRoot);

                oracleBaseline.Add(new
                {
                    scenario_id = entry.ScenarioId,
                    result_state = ToSnakeCase(oracleArtifacts.ResultState),
                    published_values = Array.Empty<object>(),
                    pinned_views = Array.Empty<object>(),
                    counters = Array.Empty<object>(),
                    rejects = Array.Empty<object>(),
                });

                engineDiff.Add(new
                {
                    scenario_id = entry.ScenarioId,
                    oracle_result_state = ToSnakeCase(oracleArtifacts.ResultState),
                    engine_result_state = ToSnakeCase(engineArtifacts.ResultState),
                    mismatches = Array.Empty<object>(),
                });
            }

            var result = new TraceCalcScenarioResult(
                entry.ScenarioId,
                resultState,
                validationFailures,
                assertionFailures,
                conformanceMismatches,
                new Dictionary<string, string>(StringComparer.Ordinal)
                {
                    ["result"] = RelativeArtifactPath(relativeArtifactRoot, "scenarios", entry.ScenarioId, "result.json"),
                    ["trace"] = RelativeArtifactPath(relativeArtifactRoot, "scenarios", entry.ScenarioId, "trace.json"),
                    ["counters"] = RelativeArtifactPath(relativeArtifactRoot, "scenarios", entry.ScenarioId, "counters.json"),
                    ["published_view"] = RelativeArtifactPath(relativeArtifactRoot, "scenarios", entry.ScenarioId, "published_view.json"),
                    ["pinned_views"] = RelativeArtifactPath(relativeArtifactRoot, "scenarios", entry.ScenarioId, "pinned_views.json"),
                    ["rejects"] = RelativeArtifactPath(relativeArtifactRoot, "scenarios", entry.ScenarioId, "rejects.json"),
                });
            scenarioResults.Add(result);
        }

        File.WriteAllText(Path.Combine(artifactRoot, "conformance", "oracle_baseline.json"), JsonSerializer.Serialize(oracleBaseline, JsonOptions));
        File.WriteAllText(Path.Combine(artifactRoot, "conformance", "engine_diff.json"), JsonSerializer.Serialize(engineDiff, JsonOptions));

        var resultCounts = scenarioResults
            .GroupBy(result => result.ResultState.ToString(), StringComparer.Ordinal)
            .ToDictionary(group => group.Key, group => group.Count(), StringComparer.Ordinal);
        var summary = new TraceCalcRunSummary(runId, manifest.SchemaVersion, scenarioResults.Count, resultCounts, artifactRoot);
        File.WriteAllText(Path.Combine(artifactRoot, "run_summary.json"), JsonSerializer.Serialize(new
        {
            run_id = summary.RunId,
            schema_version = summary.SchemaVersion,
            scenario_count = summary.ScenarioCount,
            result_counts = summary.ResultCounts.ToDictionary(pair => ToSnakeCase(pair.Key), pair => pair.Value, StringComparer.Ordinal),
            artifact_root = relativeArtifactRoot,
        }, JsonOptions));
        return summary;
    }

    private static TraceCalcExecutionArtifacts CreateEmptyArtifacts(string scenarioId, TraceCalcScenarioResultState state) =>
        new(scenarioId, state, [], [], new Dictionary<string, int>(), new Dictionary<string, string>(), [], []);

    private static void WriteScenarioArtifacts(
        string scenarioDirectory,
        string runId,
        TraceCalcScenario? scenario,
        string scenarioId,
        TraceCalcScenarioResultState resultState,
        IReadOnlyList<TraceCalcValidationFailure> validationFailures,
        IReadOnlyList<string> assertionFailures,
        IReadOnlyList<TraceCalcConformanceMismatch> conformanceMismatches,
        TraceCalcExecutionArtifacts artifacts,
        string relativeArtifactRoot)
    {
        var relativeScenarioRoot = RelativeArtifactPath(relativeArtifactRoot, "scenarios", scenarioId);
        File.WriteAllText(Path.Combine(scenarioDirectory, "result.json"), JsonSerializer.Serialize(new
        {
            scenario_id = scenarioId,
            result_state = ToSnakeCase(resultState),
            validation_failures = validationFailures.Select(failure => new { kind = ToSnakeCase(failure.Kind), message = failure.Message }).ToArray(),
            assertion_failures = assertionFailures,
            conformance_mismatches = conformanceMismatches.Select(ToMismatchObject).ToArray(),
            artifact_paths = new Dictionary<string, string>(StringComparer.Ordinal)
            {
                ["result"] = RelativeArtifactPath(relativeScenarioRoot, "result.json"),
                ["trace"] = RelativeArtifactPath(relativeScenarioRoot, "trace.json"),
                ["counters"] = RelativeArtifactPath(relativeScenarioRoot, "counters.json"),
                ["published_view"] = RelativeArtifactPath(relativeScenarioRoot, "published_view.json"),
                ["pinned_views"] = RelativeArtifactPath(relativeScenarioRoot, "pinned_views.json"),
                ["rejects"] = RelativeArtifactPath(relativeScenarioRoot, "rejects.json"),
            },
        }, JsonOptions));

        File.WriteAllText(Path.Combine(scenarioDirectory, "trace.json"), JsonSerializer.Serialize(new
        {
            scenario_id = scenarioId,
            run_id = runId,
            events = artifacts.TraceEvents.Select(ToTraceEventObject).ToArray(),
        }, JsonOptions));

        File.WriteAllText(Path.Combine(scenarioDirectory, "counters.json"), JsonSerializer.Serialize(new
        {
            scenario_id = scenarioId,
            counters = ToCounterEntries(artifacts.Counters),
        }, JsonOptions));

        File.WriteAllText(Path.Combine(scenarioDirectory, "published_view.json"), JsonSerializer.Serialize(new
        {
            scenario_id = scenarioId,
            snapshot_id = scenario?.InitialGraph.SnapshotId ?? string.Empty,
            node_values = ToValueEntries(artifacts.PublishedValues),
        }, JsonOptions));

        File.WriteAllText(Path.Combine(scenarioDirectory, "pinned_views.json"), JsonSerializer.Serialize(new
        {
            scenario_id = scenarioId,
            views = artifacts.PinnedViews.Select(ToPinnedViewObject).ToArray(),
        }, JsonOptions));

        File.WriteAllText(Path.Combine(scenarioDirectory, "rejects.json"), JsonSerializer.Serialize(new
        {
            scenario_id = scenarioId,
            rejects = artifacts.Rejects.Select(ToRejectObject).ToArray(),
        }, JsonOptions));
    }

    private static object[] ToCounterEntries(IReadOnlyDictionary<string, int> counters) =>
        counters.OrderBy(pair => pair.Key, StringComparer.Ordinal)
            .Select(pair => new { counter = pair.Key, value = pair.Value })
            .Cast<object>()
            .ToArray();

    private static object[] ToValueEntries(IReadOnlyDictionary<string, string> values) =>
        values.OrderBy(pair => pair.Key, StringComparer.Ordinal)
            .Select(pair => new { node_id = pair.Key, value = pair.Value })
            .Cast<object>()
            .ToArray();

    private static object ToPinnedViewObject(TraceCalcPinnedViewRecord view) => new
    {
        view_id = view.ViewId,
        snapshot_id = view.SnapshotId,
        node_values = ToValueEntries(view.NodeValues),
    };

    private static object ToRejectObject(TraceCalcRejectRecord reject) => new
    {
        reject_id = reject.RejectId,
        reject_kind = reject.RejectKind,
        reject_detail = reject.RejectDetail,
    };

    private static object ToTraceEventObject(TraceCalcTraceEvent traceEvent) => new
    {
        event_id = traceEvent.EventId,
        step_id = traceEvent.StepId,
        label = traceEvent.Label,
        payload = traceEvent.Payload,
    };

    private static object ToMismatchObject(TraceCalcConformanceMismatch mismatch) => new
    {
        kind = ToSnakeCase(mismatch.Kind),
        message = mismatch.Message,
    };

    private static string ToSnakeCase<TEnum>(TEnum value) where TEnum : struct, Enum
    {
        var name = value.ToString();
        return ToSnakeCase(name);
    }

    private static string ToSnakeCase(string value)
    {
        if (string.IsNullOrEmpty(value))
        {
            return string.Empty;
        }

        var buffer = new System.Text.StringBuilder(value.Length + 8);
        for (var index = 0; index < value.Length; index++)
        {
            var current = value[index];
            if (char.IsUpper(current) && index > 0)
            {
                buffer.Append('_');
            }

            buffer.Append(char.ToLowerInvariant(current));
        }

        return buffer.ToString();
    }

    private static string RelativeArtifactPath(params string[] segments) =>
        string.Join("/", segments
            .Where(static segment => !string.IsNullOrWhiteSpace(segment))
            .Select(static segment => segment.Replace('\\', '/').Trim('/')));
}
