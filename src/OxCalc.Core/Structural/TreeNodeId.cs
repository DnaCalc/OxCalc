namespace OxCalc.Core.Structural;

public readonly record struct TreeNodeId(long Value)
{
    public override string ToString() => $"node:{Value}";
}
