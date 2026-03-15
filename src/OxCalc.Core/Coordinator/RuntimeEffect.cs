namespace OxCalc.Core.Coordinator;

public sealed record RuntimeEffect(
    string Kind,
    string Detail);
