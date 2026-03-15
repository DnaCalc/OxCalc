using System.Collections.Immutable;
using OxCalc.Core.Structural;

namespace OxCalc.Core.Tests;

internal static class StructuralSnapshotTestData
{
    public static StructuralSnapshot CreateSeedSnapshot(StructuralSnapshotId snapshotId)
    {
        var rootId = new TreeNodeId(1);
        var alphaId = new TreeNodeId(2);
        var leafId = new TreeNodeId(3);
        var betaId = new TreeNodeId(4);

        return StructuralSnapshot.Create(
            snapshotId,
            rootId,
            [
                new StructuralNode(rootId, StructuralNodeKind.Root, "root", null, [alphaId, betaId]),
                new StructuralNode(alphaId, StructuralNodeKind.Container, "alpha", rootId, [leafId]),
                new StructuralNode(leafId, StructuralNodeKind.Calculation, "leaf", alphaId, ImmutableArray<TreeNodeId>.Empty, new FormulaArtifactId("fx:leaf:v1")),
                new StructuralNode(betaId, StructuralNodeKind.Constant, "beta", rootId, ImmutableArray<TreeNodeId>.Empty, new FormulaArtifactId("const:beta:v1")),
            ]);
    }
}
