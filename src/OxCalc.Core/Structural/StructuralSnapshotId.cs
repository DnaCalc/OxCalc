namespace OxCalc.Core.Structural;

public readonly record struct StructuralSnapshotId(long Value)
{
    public override string ToString() => $"snapshot:{Value}";
}
