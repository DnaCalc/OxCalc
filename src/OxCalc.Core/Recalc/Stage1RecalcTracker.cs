using OxCalc.Core.Structural;

namespace OxCalc.Core.Recalc;

public sealed class Stage1RecalcTracker
{
    private readonly Dictionary<TreeNodeId, NodeCalcState> _nodeStates = new();
    private readonly Dictionary<OverlayKey, OverlayEntry> _overlays = new();
    private readonly HashSet<TreeNodeId> _demandSet = [];

    public Stage1RecalcTracker(StructuralSnapshot snapshot)
    {
        Snapshot = snapshot;
        foreach (var nodeId in snapshot.Nodes.Keys)
        {
            _nodeStates[nodeId] = NodeCalcState.Clean;
        }
    }

    public StructuralSnapshot Snapshot { get; }

    public IReadOnlyDictionary<TreeNodeId, NodeCalcState> NodeStates => _nodeStates;

    public IReadOnlyDictionary<OverlayKey, OverlayEntry> Overlays => _overlays;

    public IReadOnlyCollection<TreeNodeId> DemandSet => _demandSet;

    public NodeCalcState GetState(TreeNodeId nodeId) => _nodeStates[nodeId];

    public void MarkDirty(TreeNodeId nodeId)
    {
        _nodeStates[nodeId] = NodeCalcState.DirtyPending;
        ProtectExecutionOverlay(nodeId, "dirty_pending");
    }

    public void MarkNeeded(TreeNodeId nodeId)
    {
        RequireState(nodeId, NodeCalcState.DirtyPending);
        _nodeStates[nodeId] = NodeCalcState.Needed;
        _demandSet.Add(nodeId);
        ProtectExecutionOverlay(nodeId, "needed");
    }

    public void BeginEvaluate(TreeNodeId nodeId, string compatibilityBasis)
    {
        RequireState(nodeId, NodeCalcState.Needed);
        _nodeStates[nodeId] = NodeCalcState.Evaluating;
        ProtectExecutionOverlay(nodeId, "evaluating");
        UpsertOverlay(new OverlayKey(nodeId, OverlayKind.CapabilityFenceAttachment, Snapshot.SnapshotId, compatibilityBasis), true, false, "evaluation_basis");
    }

    public void VerifyClean(TreeNodeId nodeId)
    {
        RequireState(nodeId, NodeCalcState.Evaluating);
        _nodeStates[nodeId] = NodeCalcState.VerifiedClean;
        _demandSet.Remove(nodeId);
        ProtectExecutionOverlay(nodeId, "verified_clean");
    }

    public void ProduceCandidateResult(TreeNodeId nodeId, string compatibilityBasis, string payloadIdentity = "none")
    {
        RequireState(nodeId, NodeCalcState.Evaluating);
        _nodeStates[nodeId] = NodeCalcState.PublishReady;
        ProtectExecutionOverlay(nodeId, "publish_ready");
        UpsertOverlay(new OverlayKey(nodeId, OverlayKind.CapabilityFenceAttachment, Snapshot.SnapshotId, compatibilityBasis, payloadIdentity), true, false, "candidate_ready");
    }

    public void ProduceDependencyShapeUpdate(TreeNodeId nodeId, string compatibilityBasis, string payloadIdentity)
    {
        RequireState(nodeId, NodeCalcState.Evaluating);
        _nodeStates[nodeId] = NodeCalcState.PublishReady;
        UpsertOverlay(new OverlayKey(nodeId, OverlayKind.DynamicDependency, Snapshot.SnapshotId, compatibilityBasis, payloadIdentity), true, false, "candidate_shape_update");
        ProtectExecutionOverlay(nodeId, "publish_ready");
    }

    public void RejectOrFallback(TreeNodeId nodeId, string reason)
    {
        if (_nodeStates[nodeId] is not (NodeCalcState.Evaluating or NodeCalcState.PublishReady))
        {
            throw new InvalidOperationException($"Node '{nodeId}' is not eligible for reject/fallback from state '{_nodeStates[nodeId]}'.");
        }

        _nodeStates[nodeId] = NodeCalcState.RejectedPendingRepair;
        _demandSet.Add(nodeId);

        foreach (var key in _overlays.Keys.Where(key => key.OwnerNodeId == nodeId && key.OverlayKind == OverlayKind.DynamicDependency).ToArray())
        {
            _overlays.Remove(key);
        }

        ProtectExecutionOverlay(nodeId, $"fallback:{reason}");
    }

    public void PublishAndClear(TreeNodeId nodeId)
    {
        RequireState(nodeId, NodeCalcState.PublishReady);
        _nodeStates[nodeId] = NodeCalcState.Clean;
        _demandSet.Remove(nodeId);
        MarkExecutionOverlayEligible(nodeId, "published");
    }

    public void ReleaseAndEvictEligible(TreeNodeId nodeId)
    {
        if (_nodeStates[nodeId] is not (NodeCalcState.Clean or NodeCalcState.VerifiedClean))
        {
            throw new InvalidOperationException($"Node '{nodeId}' is not eligible for release from state '{_nodeStates[nodeId]}'.");
        }

        _demandSet.Remove(nodeId);
        foreach (var key in _overlays.Keys.Where(key => key.OwnerNodeId == nodeId).ToArray())
        {
            var entry = _overlays[key];
            _overlays[key] = entry with { IsProtected = false, IsEvictionEligible = true };
        }
    }

    private void ProtectExecutionOverlay(TreeNodeId nodeId, string detail)
    {
        var key = new OverlayKey(nodeId, OverlayKind.InvalidationExecutionState, Snapshot.SnapshotId, "stage1");
        UpsertOverlay(key, true, false, detail);
    }

    private void MarkExecutionOverlayEligible(TreeNodeId nodeId, string detail)
    {
        var key = new OverlayKey(nodeId, OverlayKind.InvalidationExecutionState, Snapshot.SnapshotId, "stage1");
        UpsertOverlay(key, false, true, detail);
    }

    private void UpsertOverlay(OverlayKey key, bool isProtected, bool isEvictionEligible, string detail)
    {
        _overlays[key] = new OverlayEntry(key, isProtected, isEvictionEligible, detail);
    }

    private void RequireState(TreeNodeId nodeId, NodeCalcState expected)
    {
        if (_nodeStates[nodeId] != expected)
        {
            throw new InvalidOperationException($"Node '{nodeId}' must be in state '{expected}' but is '{_nodeStates[nodeId]}'.");
        }
    }
}
