using System.Collections.Immutable;
using OxCalc.Core.Structural;

namespace OxCalc.Core.Coordinator;

public sealed record DependencyShapeUpdate(
    string Kind,
    ImmutableArray<TreeNodeId> AffectedNodeIds);
