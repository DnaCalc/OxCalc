namespace OxCalc.Core.Structural;

public readonly record struct FormulaArtifactId(string Value)
{
    public override string ToString() => Value;
}
