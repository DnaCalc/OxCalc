using System.Collections.Immutable;

namespace OxCalc.Core.Structural;

public sealed class StructuralSnapshot
{
    private readonly ImmutableDictionary<string, TreeNodeId> _pathIndex;

    private StructuralSnapshot(
        StructuralSnapshotId snapshotId,
        TreeNodeId rootNodeId,
        ImmutableDictionary<TreeNodeId, StructuralNode> nodes,
        ImmutableDictionary<string, TreeNodeId> pathIndex)
    {
        SnapshotId = snapshotId;
        RootNodeId = rootNodeId;
        Nodes = nodes;
        _pathIndex = pathIndex;
    }

    public StructuralSnapshotId SnapshotId { get; }

    public TreeNodeId RootNodeId { get; }

    public ImmutableDictionary<TreeNodeId, StructuralNode> Nodes { get; }

    public StructuralNode RootNode => Nodes[RootNodeId];

    public static StructuralSnapshot Create(
        StructuralSnapshotId snapshotId,
        TreeNodeId rootNodeId,
        IEnumerable<StructuralNode> nodes)
    {
        var nodeMap = nodes.ToImmutableDictionary(node => node.NodeId);
        Validate(snapshotId, rootNodeId, nodeMap);
        var pathIndex = BuildPathIndex(rootNodeId, nodeMap);
        return new StructuralSnapshot(snapshotId, rootNodeId, nodeMap, pathIndex);
    }

    public bool TryGetNode(TreeNodeId nodeId, out StructuralNode node)
    {
        if (Nodes.TryGetValue(nodeId, out var found) && found is not null)
        {
            node = found;
            return true;
        }

        node = null!;
        return false;
    }

    public string GetProjectionPath(TreeNodeId nodeId)
    {
        if (!TryGetNode(nodeId, out var node))
        {
            throw new KeyNotFoundException($"Unknown node '{nodeId}'.");
        }

        var segments = new Stack<string>();
        var cursor = node;

        while (true)
        {
            segments.Push(cursor.Symbol);
            if (cursor.ParentId is null)
            {
                break;
            }

            cursor = Nodes[cursor.ParentId.Value];
        }

        return string.Join('/', segments);
    }

    public bool TryResolveProjectionPath(string projectionPath, out TreeNodeId nodeId) =>
        _pathIndex.TryGetValue(projectionPath, out nodeId);

    public PinnedStructuralView Pin() => new(this);

    private static void Validate(
        StructuralSnapshotId snapshotId,
        TreeNodeId rootNodeId,
        ImmutableDictionary<TreeNodeId, StructuralNode> nodes)
    {
        if (!nodes.TryGetValue(rootNodeId, out var root))
        {
            throw new InvalidOperationException($"Snapshot {snapshotId} does not contain the declared root '{rootNodeId}'.");
        }

        if (root.ParentId is not null)
        {
            throw new InvalidOperationException($"Root '{rootNodeId}' may not declare a parent.");
        }

        var seen = new HashSet<TreeNodeId>();
        Visit(rootNodeId, nodes, seen);

        if (seen.Count != nodes.Count)
        {
            var detached = nodes.Keys.Except(seen).ToArray();
            throw new InvalidOperationException(
                $"Snapshot {snapshotId} contains detached or unreachable nodes: {string.Join(", ", detached)}.");
        }
    }

    private static void Visit(
        TreeNodeId nodeId,
        ImmutableDictionary<TreeNodeId, StructuralNode> nodes,
        HashSet<TreeNodeId> seen)
    {
        if (!seen.Add(nodeId))
        {
            throw new InvalidOperationException($"Cycle or duplicate structural reachability detected at '{nodeId}'.");
        }

        var node = nodes[nodeId];
        foreach (var childId in node.ChildIds)
        {
            if (!nodes.TryGetValue(childId, out var child))
            {
                throw new InvalidOperationException($"Node '{nodeId}' references missing child '{childId}'.");
            }

            if (child.ParentId != nodeId)
            {
                throw new InvalidOperationException(
                    $"Child '{childId}' does not point back to parent '{nodeId}'.");
            }

            Visit(childId, nodes, seen);
        }
    }

    private static ImmutableDictionary<string, TreeNodeId> BuildPathIndex(
        TreeNodeId rootNodeId,
        ImmutableDictionary<TreeNodeId, StructuralNode> nodes)
    {
        var builder = ImmutableDictionary.CreateBuilder<string, TreeNodeId>(StringComparer.Ordinal);
        var stack = new Stack<(TreeNodeId NodeId, string Path)>();
        var root = nodes[rootNodeId];
        stack.Push((rootNodeId, root.Symbol));

        while (stack.Count > 0)
        {
            var (nodeId, path) = stack.Pop();
            builder[path] = nodeId;

            var node = nodes[nodeId];
            for (var i = node.ChildIds.Length - 1; i >= 0; i--)
            {
                var childId = node.ChildIds[i];
                var child = nodes[childId];
                stack.Push((childId, $"{path}/{child.Symbol}"));
            }
        }

        return builder.ToImmutable();
    }
}
