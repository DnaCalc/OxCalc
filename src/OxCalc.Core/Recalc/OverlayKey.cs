using OxCalc.Core.Structural;

namespace OxCalc.Core.Recalc;

public sealed record OverlayKey(
    TreeNodeId OwnerNodeId,
    OverlayKind OverlayKind,
    StructuralSnapshotId StructuralSnapshotId,
    string CompatibilityBasis,
    string? PayloadIdentity = null);
