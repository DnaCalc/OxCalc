using OxCalc.Core.TraceCalc;
using System.Text.Json.Nodes;

namespace OxCalc.Core.Tests;

public sealed class TraceCalcRunnerTests
{
    [Fact]
    public void ExecuteManifest_ProducesPassingConformanceArtifacts_ForSeedCorpus()
    {
        var repoRoot = ResolveRepoRoot();
        var runId = $"test-tracecalc-{Guid.NewGuid():N}";
        var artifactRoot = Path.Combine(repoRoot, "docs", "test-runs", "core-engine", "tracecalc-reference-machine", runId);
        var runner = new TraceCalcRunner();

        try
        {
            var summary = runner.ExecuteManifest(repoRoot, runId);

            Assert.Equal(runId, summary.RunId);
            Assert.Equal(12, summary.ScenarioCount);
            Assert.True(summary.ResultCounts.TryGetValue(nameof(TraceCalcScenarioResultState.Passed), out var passedCount));
            Assert.Equal(summary.ScenarioCount, passedCount);
            Assert.True(File.Exists(Path.Combine(artifactRoot, "run_summary.json")));
            Assert.True(File.Exists(Path.Combine(artifactRoot, "manifest_selection.json")));
            Assert.True(File.Exists(Path.Combine(artifactRoot, "conformance", "oracle_baseline.json")));
            Assert.True(File.Exists(Path.Combine(artifactRoot, "conformance", "engine_diff.json")));

            var diffDocument = JsonNode.Parse(File.ReadAllText(Path.Combine(artifactRoot, "conformance", "engine_diff.json")))!.AsArray();
            Assert.Contains(diffDocument, node => string.Equals(node?["scenario_id"]?.GetValue<string>(), "tc_verify_clean_no_publish_001", StringComparison.Ordinal));
            Assert.Contains(diffDocument, node => string.Equals(node?["scenario_id"]?.GetValue<string>(), "tc_fallback_reentry_001", StringComparison.Ordinal));
            Assert.DoesNotContain(diffDocument, node => node?["mismatches"]?.AsArray().Count > 0);

            var verifyResult = JsonNode.Parse(File.ReadAllText(Path.Combine(artifactRoot, "scenarios", "tc_verify_clean_no_publish_001", "result.json")))!.AsObject();
            Assert.Equal("passed", verifyResult["result_state"]?.GetValue<string>());

            var verifyTrace = JsonNode.Parse(File.ReadAllText(Path.Combine(artifactRoot, "scenarios", "tc_verify_clean_no_publish_001", "trace.json")))!.AsObject();
            var events = verifyTrace["events"]!.AsArray();
            Assert.Contains(events, node => string.Equals(node?["label"]?.GetValue<string>(), "node_verified_clean", StringComparison.Ordinal));
            Assert.Contains(events, node => string.Equals(node?["normalized_event_family"]?.GetValue<string>(), "candidate.verified_clean", StringComparison.Ordinal));

            var acceptTrace = JsonNode.Parse(File.ReadAllText(Path.Combine(artifactRoot, "scenarios", "tc_accept_publish_001", "trace.json")))!.AsObject();
            Assert.Equal("candidate.built", acceptTrace["events"]!.AsArray().Single(node => string.Equals(node?["label"]?.GetValue<string>(), "candidate_emitted", StringComparison.Ordinal))?["normalized_event_family"]?.GetValue<string>());

            var fallbackResult = JsonNode.Parse(File.ReadAllText(Path.Combine(artifactRoot, "scenarios", "tc_fallback_reentry_001", "result.json")))!.AsObject();
            Assert.Equal("passed", fallbackResult["result_state"]?.GetValue<string>());
            Assert.NotNull(fallbackResult["replay_projection"]);
        }
        finally
        {
            if (Directory.Exists(artifactRoot))
            {
                Directory.Delete(artifactRoot, recursive: true);
            }
        }
    }

    private static string ResolveRepoRoot()
    {
        var directory = new DirectoryInfo(AppContext.BaseDirectory);
        while (directory is not null)
        {
            if (File.Exists(Path.Combine(directory.FullName, "OxCalc.slnx")))
            {
                return directory.FullName;
            }

            directory = directory.Parent;
        }

        throw new InvalidOperationException("Could not resolve repo root for TraceCalc runner tests.");
    }
}
