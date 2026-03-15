namespace OxCalc.Core.TraceCalc;

public static class TraceCalcAssertionEvaluator
{
    public static IReadOnlyList<string> Evaluate(
        TraceCalcScenario scenario,
        IReadOnlyDictionary<string, string> publishedValues,
        IEnumerable<TraceCalcPinnedViewRecord> pinnedViews,
        IReadOnlyList<TraceCalcTraceEvent> traceEvents,
        IReadOnlyDictionary<string, int> counters,
        IReadOnlyList<TraceCalcRejectRecord> rejects)
    {
        var failures = new List<string>();

        foreach (var expectedValue in scenario.Expected.PublishedView.NodeValues)
        {
            if (!publishedValues.TryGetValue(expectedValue.NodeId, out var observed) || !string.Equals(observed, expectedValue.Value, StringComparison.Ordinal))
            {
                failures.Add($"Published value mismatch for node '{expectedValue.NodeId}': expected '{expectedValue.Value}' but observed '{observed ?? "<missing>"}'.");
            }
        }

        var pinnedViewMap = pinnedViews.ToDictionary(view => view.ViewId, StringComparer.Ordinal);
        foreach (var expectedView in scenario.Expected.PinnedViews)
        {
            if (!pinnedViewMap.TryGetValue(expectedView.ViewId, out var observedView))
            {
                failures.Add($"Missing pinned view '{expectedView.ViewId}'.");
                continue;
            }

            foreach (var expectedValue in expectedView.NodeValues)
            {
                if (!observedView.NodeValues.TryGetValue(expectedValue.NodeId, out var observed) || !string.Equals(observed, expectedValue.Value, StringComparison.Ordinal))
                {
                    failures.Add($"Pinned view mismatch for '{expectedView.ViewId}' node '{expectedValue.NodeId}': expected '{expectedValue.Value}' but observed '{observed ?? "<missing>"}'.");
                }
            }
        }

        var traceCounts = traceEvents.GroupBy(evt => evt.Label, StringComparer.Ordinal).ToDictionary(group => group.Key, group => group.Count(), StringComparer.Ordinal);
        foreach (var expectation in scenario.Expected.TraceLabels)
        {
            var observed = traceCounts.GetValueOrDefault(expectation.Label);
            if (observed != expectation.Count)
            {
                failures.Add($"Trace label count mismatch for '{expectation.Label}': expected {expectation.Count} but observed {observed}.");
            }
        }

        foreach (var expectation in scenario.Expected.CounterExpectations)
        {
            var observed = counters.GetValueOrDefault(expectation.Counter);
            if (!Compare(observed, expectation.Comparison, expectation.Value))
            {
                failures.Add($"Counter mismatch for '{expectation.Counter}': expected {expectation.Comparison} {expectation.Value} but observed {observed}.");
            }
        }

        var rejectMap = rejects.ToDictionary(reject => reject.RejectId, StringComparer.Ordinal);
        foreach (var expectation in scenario.Expected.Rejects)
        {
            if (!rejectMap.TryGetValue(expectation.RejectId, out var observedReject))
            {
                failures.Add($"Missing reject '{expectation.RejectId}'.");
                continue;
            }

            if (!string.Equals(observedReject.RejectKind, expectation.RejectKind, StringComparison.Ordinal))
            {
                failures.Add($"Reject kind mismatch for '{expectation.RejectId}': expected '{expectation.RejectKind}' but observed '{observedReject.RejectKind}'.");
            }

            if (!string.IsNullOrEmpty(expectation.DetailContains) && !observedReject.RejectDetail.Contains(expectation.DetailContains, StringComparison.OrdinalIgnoreCase))
            {
                failures.Add($"Reject detail mismatch for '{expectation.RejectId}': expected detail containing '{expectation.DetailContains}'.");
            }
        }

        return failures;
    }

    private static bool Compare(int observed, string comparison, int expected) => comparison switch
    {
        "eq" => observed == expected,
        "ge" => observed >= expected,
        "gt" => observed > expected,
        "le" => observed <= expected,
        "lt" => observed < expected,
        _ => false,
    };
}

public static class TraceCalcConformanceComparer
{
    public static IReadOnlyList<TraceCalcConformanceMismatch> Compare(TraceCalcExecutionArtifacts oracle, TraceCalcExecutionArtifacts engine)
    {
        var mismatches = new List<TraceCalcConformanceMismatch>();
        if (oracle.ResultState != engine.ResultState)
        {
            mismatches.Add(new TraceCalcConformanceMismatch(TraceCalcConformanceMismatchKind.ResultStateMismatch, $"Result-state mismatch: oracle '{oracle.ResultState}', engine '{engine.ResultState}'."));
        }

        foreach (var pair in oracle.PublishedValues)
        {
            if (!engine.PublishedValues.TryGetValue(pair.Key, out var observed) || !string.Equals(observed, pair.Value, StringComparison.Ordinal))
            {
                mismatches.Add(new TraceCalcConformanceMismatch(TraceCalcConformanceMismatchKind.PublishedViewMismatch, $"Published view mismatch for node '{pair.Key}'."));
            }
        }

        var oraclePinned = oracle.PinnedViews.ToDictionary(view => view.ViewId, StringComparer.Ordinal);
        var enginePinned = engine.PinnedViews.ToDictionary(view => view.ViewId, StringComparer.Ordinal);
        foreach (var pair in oraclePinned)
        {
            if (!enginePinned.TryGetValue(pair.Key, out var observedView))
            {
                mismatches.Add(new TraceCalcConformanceMismatch(TraceCalcConformanceMismatchKind.PinnedViewMismatch, $"Missing pinned view '{pair.Key}' in engine output."));
                continue;
            }

            foreach (var valuePair in pair.Value.NodeValues)
            {
                if (!observedView.NodeValues.TryGetValue(valuePair.Key, out var observedValue) || !string.Equals(observedValue, valuePair.Value, StringComparison.Ordinal))
                {
                    mismatches.Add(new TraceCalcConformanceMismatch(TraceCalcConformanceMismatchKind.PinnedViewMismatch, $"Pinned-view mismatch for '{pair.Key}' node '{valuePair.Key}'."));
                }
            }
        }

        var oracleRejects = oracle.Rejects.Select(reject => $"{reject.RejectId}:{reject.RejectKind}:{reject.RejectDetail}").OrderBy(value => value, StringComparer.Ordinal).ToArray();
        var engineRejects = engine.Rejects.Select(reject => $"{reject.RejectId}:{reject.RejectKind}:{reject.RejectDetail}").OrderBy(value => value, StringComparer.Ordinal).ToArray();
        if (!oracleRejects.SequenceEqual(engineRejects, StringComparer.Ordinal))
        {
            mismatches.Add(new TraceCalcConformanceMismatch(TraceCalcConformanceMismatchKind.RejectMismatch, "Reject outputs differ between oracle and engine."));
        }

        var oracleTraceCounts = oracle.TraceEvents.GroupBy(evt => evt.Label, StringComparer.Ordinal).ToDictionary(group => group.Key, group => group.Count(), StringComparer.Ordinal);
        var engineTraceCounts = engine.TraceEvents.GroupBy(evt => evt.Label, StringComparer.Ordinal).ToDictionary(group => group.Key, group => group.Count(), StringComparer.Ordinal);
        foreach (var pair in oracleTraceCounts)
        {
            if (engineTraceCounts.GetValueOrDefault(pair.Key) != pair.Value)
            {
                mismatches.Add(new TraceCalcConformanceMismatch(TraceCalcConformanceMismatchKind.TraceCountMismatch, $"Trace count mismatch for label '{pair.Key}'."));
            }
        }

        foreach (var pair in oracle.Counters)
        {
            if (engine.Counters.GetValueOrDefault(pair.Key) != pair.Value)
            {
                mismatches.Add(new TraceCalcConformanceMismatch(TraceCalcConformanceMismatchKind.CounterMismatch, $"Counter mismatch for '{pair.Key}'."));
            }
        }

        return mismatches;
    }
}

internal static class TraceCalcStringExtensions
{
    public static string ToSnakeCase(this string value)
    {
        if (string.IsNullOrEmpty(value))
        {
            return value;
        }

        var builder = new System.Text.StringBuilder();
        for (var i = 0; i < value.Length; i++)
        {
            var ch = value[i];
            if (char.IsUpper(ch) && i > 0)
            {
                builder.Append('_');
            }

            builder.Append(char.ToLowerInvariant(ch));
        }

        return builder.ToString();
    }
}
