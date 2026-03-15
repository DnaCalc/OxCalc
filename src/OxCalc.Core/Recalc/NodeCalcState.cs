namespace OxCalc.Core.Recalc;

public enum NodeCalcState
{
    Clean = 0,
    DirtyPending = 1,
    Needed = 2,
    Evaluating = 3,
    VerifiedClean = 4,
    PublishReady = 5,
    RejectedPendingRepair = 6,
    CycleBlocked = 7,
}
