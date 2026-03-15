using System.Collections.Immutable;

namespace OxCalc.Core.Structural;

public sealed class StructuralSnapshotBuilder
{
    private readonly Dictionary<TreeNodeId, StructuralNode> _nodes;

    public StructuralSnapshotBuilder(StructuralSnapshot? predecessor = null)
    {
        _nodes = predecessor is null
            ? new Dictionary<TreeNodeId, StructuralNode>()
            : predecessor.Nodes.ToDictionary(pair => pair.Key, pair => pair.Value);
        RootNodeId = predecessor?.RootNodeId;
    }

    public TreeNodeId? RootNodeId { get; private set; }

    public void SetRoot(TreeNodeId rootNodeId)
    {
        RootNodeId = rootNodeId;
    }

    public void SetNode(StructuralNode node)
    {
        _nodes[node.NodeId] = node;
    }

    public void AttachChild(TreeNodeId parentId, TreeNodeId childId, int? index = null)
    {
        var parent = RequireNode(parentId);
        var child = RequireNode(childId);

        var childIds = parent.ChildIds.Remove(childId);
        var insertionIndex = index ?? childIds.Length;
        childIds = childIds.Insert(insertionIndex, childId);

        _nodes[parentId] = parent.WithChildren(childIds);
        _nodes[childId] = child.WithParent(parentId);
    }

    public void ReplaceChildren(TreeNodeId parentId, IEnumerable<TreeNodeId> childIds)
    {
        var parent = RequireNode(parentId);
        var orderedChildren = childIds.ToImmutableArray();
        _nodes[parentId] = parent.WithChildren(orderedChildren);

        foreach (var childId in orderedChildren)
        {
            var child = RequireNode(childId);
            _nodes[childId] = child.WithParent(parentId);
        }
    }

    public StructuralSnapshot Build(StructuralSnapshotId snapshotId)
    {
        if (RootNodeId is null)
        {
            throw new InvalidOperationException("A structural snapshot must declare a root node.");
        }

        return StructuralSnapshot.Create(snapshotId, RootNodeId.Value, _nodes.Values);
    }

    private StructuralNode RequireNode(TreeNodeId nodeId)
    {
        if (!_nodes.TryGetValue(nodeId, out var node))
        {
            throw new KeyNotFoundException($"Unknown node '{nodeId}'.");
        }

        return node;
    }
}
