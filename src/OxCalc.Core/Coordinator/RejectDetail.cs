namespace OxCalc.Core.Coordinator;

public sealed record RejectDetail(
    string CandidateResultId,
    RejectKind Kind,
    string Detail);
