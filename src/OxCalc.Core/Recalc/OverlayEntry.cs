namespace OxCalc.Core.Recalc;

public sealed record OverlayEntry(
    OverlayKey Key,
    bool IsProtected,
    bool IsEvictionEligible,
    string Detail);
