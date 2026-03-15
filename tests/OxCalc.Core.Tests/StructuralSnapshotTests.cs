using System.Collections.Immutable;
using OxCalc.Core.Structural;

namespace OxCalc.Core.Tests;

public class StructuralSnapshotTests
{
    [Fact]
    public void SnapshotBuildsProjectionAndLookupSurface()
    {
        var snapshot = StructuralSnapshotTestData.CreateSeedSnapshot(new StructuralSnapshotId(1));

        Assert.Equal("root/alpha/leaf", snapshot.GetProjectionPath(new TreeNodeId(3)));
        Assert.True(snapshot.TryResolveProjectionPath("root/alpha/leaf", out var resolved));
        Assert.Equal(new TreeNodeId(3), resolved);
    }

    [Fact]
    public void SuccessorSnapshotRetainsStableIdsForUnchangedNodes()
    {
        var initial = StructuralSnapshotTestData.CreateSeedSnapshot(new StructuralSnapshotId(1));
        var builder = new StructuralSnapshotBuilder(initial);
        builder.SetNode(new StructuralNode(
            new TreeNodeId(4),
            StructuralNodeKind.Constant,
            "beta",
            new TreeNodeId(1),
            ImmutableArray<TreeNodeId>.Empty,
            new FormulaArtifactId("const:beta:v2")));

        var successor = builder.Build(new StructuralSnapshotId(2));

        Assert.Equal(initial.RootNodeId, successor.RootNodeId);
        Assert.Equal(initial.Nodes[new TreeNodeId(2)].NodeId, successor.Nodes[new TreeNodeId(2)].NodeId);
        Assert.Equal(initial.Nodes[new TreeNodeId(3)].NodeId, successor.Nodes[new TreeNodeId(3)].NodeId);
        Assert.Equal(new FormulaArtifactId("const:beta:v2"), successor.Nodes[new TreeNodeId(4)].FormulaArtifactId);
    }

    [Fact]
    public void PinnedViewRemainsBoundToOriginalSnapshotAfterSuccessorBuild()
    {
        var initial = StructuralSnapshotTestData.CreateSeedSnapshot(new StructuralSnapshotId(1));
        var pinned = initial.Pin();

        var builder = new StructuralSnapshotBuilder(initial);
        builder.SetNode(new StructuralNode(
            new TreeNodeId(3),
            StructuralNodeKind.Calculation,
            "leaf-v2",
            new TreeNodeId(2),
            ImmutableArray<TreeNodeId>.Empty,
            new FormulaArtifactId("fx:leaf:v2")));
        var successor = builder.Build(new StructuralSnapshotId(2));

        Assert.Equal(new StructuralSnapshotId(1), pinned.SnapshotId);
        Assert.Equal("root/alpha/leaf", pinned.GetProjectionPath(new TreeNodeId(3)));
        Assert.Equal("root/alpha/leaf-v2", successor.GetProjectionPath(new TreeNodeId(3)));
    }
}
