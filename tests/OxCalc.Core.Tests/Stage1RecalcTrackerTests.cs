using OxCalc.Core.Recalc;
using OxCalc.Core.Structural;

namespace OxCalc.Core.Tests;

public class Stage1RecalcTrackerTests
{
    [Fact]
    public void PublishPathMovesNodeBackToClean()
    {
        var snapshot = StructuralSnapshotTestData.CreateSeedSnapshot(new StructuralSnapshotId(1));
        var tracker = new Stage1RecalcTracker(snapshot);
        var nodeId = new TreeNodeId(3);

        tracker.MarkDirty(nodeId);
        tracker.MarkNeeded(nodeId);
        tracker.BeginEvaluate(nodeId, "compat:v1");
        tracker.ProduceDependencyShapeUpdate(nodeId, "compat:v1", "dyn:v1");
        tracker.PublishAndClear(nodeId);

        Assert.Equal(NodeCalcState.Clean, tracker.GetState(nodeId));
        Assert.DoesNotContain(nodeId, tracker.DemandSet);
    }

    [Fact]
    public void VerifyCleanPathDoesNotCreatePublishReadyState()
    {
        var snapshot = StructuralSnapshotTestData.CreateSeedSnapshot(new StructuralSnapshotId(1));
        var tracker = new Stage1RecalcTracker(snapshot);
        var nodeId = new TreeNodeId(3);

        tracker.MarkDirty(nodeId);
        tracker.MarkNeeded(nodeId);
        tracker.BeginEvaluate(nodeId, "compat:v1");
        tracker.VerifyClean(nodeId);

        Assert.Equal(NodeCalcState.VerifiedClean, tracker.GetState(nodeId));
        Assert.DoesNotContain(nodeId, tracker.DemandSet);
    }

    [Fact]
    public void RejectOrFallbackDropsDynamicDependencyOverlayAndKeepsNodeDemanded()
    {
        var snapshot = StructuralSnapshotTestData.CreateSeedSnapshot(new StructuralSnapshotId(1));
        var tracker = new Stage1RecalcTracker(snapshot);
        var nodeId = new TreeNodeId(3);

        tracker.MarkDirty(nodeId);
        tracker.MarkNeeded(nodeId);
        tracker.BeginEvaluate(nodeId, "compat:v1");
        tracker.ProduceDependencyShapeUpdate(nodeId, "compat:v1", "dyn:v1");
        tracker.RejectOrFallback(nodeId, "missing_effect_detail");

        Assert.Equal(NodeCalcState.RejectedPendingRepair, tracker.GetState(nodeId));
        Assert.Contains(nodeId, tracker.DemandSet);
        Assert.DoesNotContain(tracker.Overlays.Keys, key => key.OwnerNodeId == nodeId && key.OverlayKind == OverlayKind.DynamicDependency);
    }

    [Fact]
    public void ReleaseAndEvictEligibleMarksOverlaysEligibleAfterStableState()
    {
        var snapshot = StructuralSnapshotTestData.CreateSeedSnapshot(new StructuralSnapshotId(1));
        var tracker = new Stage1RecalcTracker(snapshot);
        var nodeId = new TreeNodeId(3);

        tracker.MarkDirty(nodeId);
        tracker.MarkNeeded(nodeId);
        tracker.BeginEvaluate(nodeId, "compat:v1");
        tracker.VerifyClean(nodeId);
        tracker.ReleaseAndEvictEligible(nodeId);

        Assert.All(
            tracker.Overlays.Where(pair => pair.Key.OwnerNodeId == nodeId),
            pair =>
            {
                Assert.False(pair.Value.IsProtected);
                Assert.True(pair.Value.IsEvictionEligible);
            });
    }
}
