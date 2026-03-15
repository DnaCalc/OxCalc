namespace OxCalc.Core.Structural;

public sealed class PinnedStructuralView
{
    private readonly StructuralSnapshot _snapshot;

    internal PinnedStructuralView(StructuralSnapshot snapshot)
    {
        _snapshot = snapshot;
    }

    public StructuralSnapshotId SnapshotId => _snapshot.SnapshotId;

    public TreeNodeId RootNodeId => _snapshot.RootNodeId;

    public bool TryGetNode(TreeNodeId nodeId, out StructuralNode node) => _snapshot.TryGetNode(nodeId, out node);

    public string GetProjectionPath(TreeNodeId nodeId) => _snapshot.GetProjectionPath(nodeId);

    public bool TryResolveProjectionPath(string projectionPath, out TreeNodeId nodeId) =>
        _snapshot.TryResolveProjectionPath(projectionPath, out nodeId);
}
