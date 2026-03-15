using OxCalc.Core.Structural;

namespace OxCalc.Core.Coordinator;

public sealed record PublishedView(
    StructuralSnapshot Snapshot,
    PublicationBundle? Publication,
    IReadOnlyDictionary<TreeNodeId, string> Values);
