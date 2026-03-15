using OxCalc.Core.TraceCalc;

var runId = args.Length > 0 ? args[0] : $"tracecalc-run-{DateTime.UtcNow:yyyyMMddHHmmss}";
var repoRoot = ResolveRepoRoot(AppContext.BaseDirectory);
var runner = new TraceCalcRunner();
var summary = runner.ExecuteManifest(repoRoot, runId);
Console.WriteLine($"Run '{summary.RunId}' wrote {summary.ScenarioCount} scenario results to '{summary.ArtifactRoot}'.");

static string ResolveRepoRoot(string startPath)
{
    var directory = new DirectoryInfo(startPath);
    while (directory is not null)
    {
        if (File.Exists(Path.Combine(directory.FullName, "OxCalc.slnx")))
        {
            return directory.FullName;
        }

        directory = directory.Parent;
    }

    throw new InvalidOperationException("Could not resolve repo root from the current application base directory.");
}
