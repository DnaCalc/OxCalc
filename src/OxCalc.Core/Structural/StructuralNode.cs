using System.Collections.Immutable;

namespace OxCalc.Core.Structural;

public sealed record StructuralNode(
    TreeNodeId NodeId,
    StructuralNodeKind Kind,
    string Symbol,
    TreeNodeId? ParentId,
    ImmutableArray<TreeNodeId> ChildIds,
    FormulaArtifactId? FormulaArtifactId = null)
{
    public StructuralNode WithParent(TreeNodeId? parentId) => this with { ParentId = parentId };

    public StructuralNode WithChildren(ImmutableArray<TreeNodeId> childIds) => this with { ChildIds = childIds };
}
