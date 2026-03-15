using System.Collections.Immutable;
using System.Text.Json.Nodes;

namespace OxCalc.Core.TraceCalc;

public sealed record TraceCalcWorksetPlan(
    ImmutableArray<ImmutableArray<string>> Groups,
    ImmutableArray<string> OrderedNodes,
    ImmutableArray<string> ImpactedNodes,
    ImmutableArray<ImmutableArray<string>> CycleGroups)
{
    public bool HasCycleGroups => CycleGroups.Length > 0;
}

public sealed class TraceCalcScenarioPlanner
{
    private readonly Dictionary<string, TraceCalcNode> _nodes;
    private readonly Dictionary<string, ImmutableArray<string>> _directDependencies;
    private readonly Dictionary<string, ImmutableArray<string>> _reverseDependencies;

    public TraceCalcScenarioPlanner(TraceCalcScenario scenario)
    {
        _nodes = scenario.InitialGraph.Nodes.ToDictionary(node => node.NodeId, StringComparer.Ordinal);
        _directDependencies = BuildDirectDependencies(_nodes);
        _reverseDependencies = BuildReverseDependencies(_directDependencies);
    }

    public TraceCalcWorksetPlan PlanWorkset(IReadOnlyCollection<string> explicitTargets, IReadOnlyCollection<string> dirtySeeds)
    {
        var impacted = new HashSet<string>(StringComparer.Ordinal);
        var seedQueue = new Queue<string>();

        foreach (var dirtySeed in dirtySeeds.Where(_nodes.ContainsKey))
        {
            if (impacted.Add(dirtySeed))
            {
                seedQueue.Enqueue(dirtySeed);
            }
        }

        while (seedQueue.Count > 0)
        {
            var current = seedQueue.Dequeue();
            foreach (var dependent in _reverseDependencies.GetValueOrDefault(current, ImmutableArray<string>.Empty))
            {
                if (impacted.Add(dependent))
                {
                    seedQueue.Enqueue(dependent);
                }
            }
        }

        foreach (var target in explicitTargets.Where(_nodes.ContainsKey))
        {
            impacted.Add(target);
        }

        if (impacted.Count == 0)
        {
            return new TraceCalcWorksetPlan([], [], [], []);
        }

        var components = ComputeComponents(impacted);
        var componentIndex = new Dictionary<string, int>(StringComparer.Ordinal);
        for (var index = 0; index < components.Count; index++)
        {
            foreach (var nodeId in components[index])
            {
                componentIndex[nodeId] = index;
            }
        }

        var outgoing = new Dictionary<int, SortedSet<int>>();
        var indegrees = new Dictionary<int, int>();
        for (var index = 0; index < components.Count; index++)
        {
            outgoing[index] = [];
            indegrees[index] = 0;
        }

        foreach (var nodeId in impacted)
        {
            var fromComponent = componentIndex[nodeId];
            foreach (var dependency in _directDependencies.GetValueOrDefault(nodeId, ImmutableArray<string>.Empty))
            {
                if (!impacted.Contains(dependency))
                {
                    continue;
                }

                var dependencyComponent = componentIndex[dependency];
                if (dependencyComponent == fromComponent)
                {
                    continue;
                }

                if (outgoing[dependencyComponent].Add(fromComponent))
                {
                    indegrees[fromComponent] = indegrees[fromComponent] + 1;
                }
            }
        }

        var ready = new SortedSet<int>(Comparer<int>.Create((left, right) =>
        {
            var leftKey = components[left][0];
            var rightKey = components[right][0];
            var compare = StringComparer.Ordinal.Compare(leftKey, rightKey);
            return compare != 0 ? compare : left.CompareTo(right);
        }));

        foreach (var pair in indegrees.Where(static pair => pair.Value == 0))
        {
            ready.Add(pair.Key);
        }

        var orderedGroups = new List<ImmutableArray<string>>();
        while (ready.Count > 0)
        {
            var currentComponent = ready.Min;
            ready.Remove(currentComponent);
            orderedGroups.Add(components[currentComponent]);

            foreach (var nextComponent in outgoing[currentComponent])
            {
                indegrees[nextComponent] = indegrees[nextComponent] - 1;
                if (indegrees[nextComponent] == 0)
                {
                    ready.Add(nextComponent);
                }
            }
        }

        var orderedNodes = orderedGroups.SelectMany(static group => group).ToImmutableArray();
        var cycleGroups = components.Where(IsCycleGroup).ToImmutableArray();
        return new TraceCalcWorksetPlan(
            orderedGroups.ToImmutableArray(),
            orderedNodes,
            impacted.OrderBy(static nodeId => nodeId, StringComparer.Ordinal).ToImmutableArray(),
            cycleGroups);
    }

    private static Dictionary<string, ImmutableArray<string>> BuildDirectDependencies(IReadOnlyDictionary<string, TraceCalcNode> nodes)
    {
        var result = new Dictionary<string, ImmutableArray<string>>(StringComparer.Ordinal);
        foreach (var pair in nodes)
        {
            result[pair.Key] = ParseDependencies(pair.Value.Expression)
                .Where(nodes.ContainsKey)
                .Distinct(StringComparer.Ordinal)
                .OrderBy(static nodeId => nodeId, StringComparer.Ordinal)
                .ToImmutableArray();
        }

        return result;
    }

    private static Dictionary<string, ImmutableArray<string>> BuildReverseDependencies(IReadOnlyDictionary<string, ImmutableArray<string>> directDependencies)
    {
        var reverse = new Dictionary<string, HashSet<string>>(StringComparer.Ordinal);
        foreach (var nodeId in directDependencies.Keys)
        {
            reverse[nodeId] = [];
        }

        foreach (var pair in directDependencies)
        {
            foreach (var dependency in pair.Value)
            {
                if (!reverse.TryGetValue(dependency, out var dependents))
                {
                    dependents = [];
                    reverse[dependency] = dependents;
                }

                dependents.Add(pair.Key);
            }
        }

        return reverse.ToDictionary(
            static pair => pair.Key,
            static pair => pair.Value.OrderBy(static nodeId => nodeId, StringComparer.Ordinal).ToImmutableArray(),
            StringComparer.Ordinal);
    }

    private static IEnumerable<string> ParseDependencies(JsonObject expression)
    {
        var op = expression["op"]?.GetValue<string>();
        return op switch
        {
            "sum" or "concat" => ReadStringArray(expression["deps"]),
            "choose" => ReadExplicitDependencies(expression, "control", "when_true", "when_false"),
            "dyn_select" => ReadDynamicSelectDependencies(expression),
            "cap_gate" or "delay" => ReadExplicitDependencies(expression, "dep"),
            _ => ReadStringArray(expression["deps"])
        };
    }

    private static IEnumerable<string> ReadDynamicSelectDependencies(JsonObject expression)
    {
        foreach (var value in ReadExplicitDependencies(expression, "selector"))
        {
            yield return value;
        }

        if (expression["candidates"] is JsonObject candidates)
        {
            foreach (var property in candidates)
            {
                if (property.Value is not null)
                {
                    yield return property.Value.GetValue<string>();
                }
            }
        }
    }

    private static IEnumerable<string> ReadExplicitDependencies(JsonObject expression, params string[] propertyNames)
    {
        foreach (var propertyName in propertyNames)
        {
            if (expression[propertyName] is JsonValue value)
            {
                yield return value.GetValue<string>();
            }
        }
    }

    private static IEnumerable<string> ReadStringArray(JsonNode? node)
    {
        if (node is not JsonArray array)
        {
            yield break;
        }

        foreach (var item in array)
        {
            if (item is JsonValue value)
            {
                yield return value.GetValue<string>();
            }
        }
    }

    private List<ImmutableArray<string>> ComputeComponents(HashSet<string> impacted)
    {
        var indexByNode = new Dictionary<string, int>(StringComparer.Ordinal);
        var lowLink = new Dictionary<string, int>(StringComparer.Ordinal);
        var active = new HashSet<string>(StringComparer.Ordinal);
        var stack = new Stack<string>();
        var components = new List<ImmutableArray<string>>();
        var index = 0;

        foreach (var nodeId in impacted.OrderBy(static node => node, StringComparer.Ordinal))
        {
            if (!indexByNode.ContainsKey(nodeId))
            {
                StrongConnect(nodeId);
            }
        }

        return components;

        void StrongConnect(string nodeId)
        {
            indexByNode[nodeId] = index;
            lowLink[nodeId] = index;
            index++;
            stack.Push(nodeId);
            active.Add(nodeId);

            foreach (var dependency in _directDependencies.GetValueOrDefault(nodeId, ImmutableArray<string>.Empty))
            {
                if (!impacted.Contains(dependency))
                {
                    continue;
                }

                if (!indexByNode.ContainsKey(dependency))
                {
                    StrongConnect(dependency);
                    lowLink[nodeId] = Math.Min(lowLink[nodeId], lowLink[dependency]);
                }
                else if (active.Contains(dependency))
                {
                    lowLink[nodeId] = Math.Min(lowLink[nodeId], indexByNode[dependency]);
                }
            }

            if (lowLink[nodeId] != indexByNode[nodeId])
            {
                return;
            }

            var component = new List<string>();
            while (stack.Count > 0)
            {
                var member = stack.Pop();
                active.Remove(member);
                component.Add(member);
                if (string.Equals(member, nodeId, StringComparison.Ordinal))
                {
                    break;
                }
            }

            component.Sort(StringComparer.Ordinal);
            components.Add(component.ToImmutableArray());
        }
    }

    private bool IsCycleGroup(ImmutableArray<string> component)
    {
        if (component.Length > 1)
        {
            return true;
        }

        var nodeId = component[0];
        return _directDependencies.GetValueOrDefault(nodeId, ImmutableArray<string>.Empty).Contains(nodeId, StringComparer.Ordinal);
    }
}
