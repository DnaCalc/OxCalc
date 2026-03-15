using OxCalc.Core.TraceCalc;

namespace OxCalc.Core.Tests;

public sealed class TraceCalcScenarioPlannerTests
{
    [Fact]
    public void PlanWorkset_OrdersMultiNodeDagDeterministically()
    {
        var scenario = LoadScenario("tc_multinode_dag_publish_001");
        var planner = new TraceCalcScenarioPlanner(scenario);

        var plan = planner.PlanWorkset(["D"], ["A"]);

        Assert.Equal(["A", "B", "C", "D"], plan.OrderedNodes.ToArray());
        Assert.False(plan.HasCycleGroups);
    }

    [Fact]
    public void PlanWorkset_FindsCycleGroup_ForSyntheticCycleScenario()
    {
        var scenario = LoadScenario("tc_cycle_region_reject_001");
        var planner = new TraceCalcScenarioPlanner(scenario);

        var plan = planner.PlanWorkset(["A"], ["A"]);

        Assert.True(plan.HasCycleGroups);
        Assert.Contains(plan.CycleGroups, group => group.SequenceEqual(["A", "B"]));
        Assert.Contains(plan.Groups, group => group.SequenceEqual(["A", "B"]));
    }

    private static TraceCalcScenario LoadScenario(string scenarioId)
    {
        var repoRoot = ResolveRepoRoot();
        var path = Path.Combine(repoRoot, "docs", "test-corpus", "core-engine", "tracecalc", "hand-auditable", $"{scenarioId}.json");
        return TraceCalcJson.LoadScenario(path);
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

        throw new InvalidOperationException("Could not resolve repo root for TraceCalc planner tests.");
    }
}
