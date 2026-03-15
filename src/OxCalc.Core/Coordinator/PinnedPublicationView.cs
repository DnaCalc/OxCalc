using OxCalc.Core.Structural;

namespace OxCalc.Core.Coordinator;

public sealed record PinnedPublicationView(
    string ReaderId,
    StructuralSnapshotId SnapshotId,
    string? PublicationId,
    PinnedStructuralView StructuralView,
    IReadOnlyDictionary<TreeNodeId, string> Values);
