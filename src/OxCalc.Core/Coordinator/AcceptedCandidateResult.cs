using System.Collections.Immutable;
using OxCalc.Core.Structural;

namespace OxCalc.Core.Coordinator;

public sealed record AcceptedCandidateResult(
    string CandidateResultId,
    StructuralSnapshotId StructuralSnapshotId,
    string ArtifactTokenBasis,
    string CompatibilityBasis,
    ImmutableArray<TreeNodeId> TargetSet,
    ImmutableDictionary<TreeNodeId, string> ValueUpdates,
    ImmutableArray<DependencyShapeUpdate> DependencyShapeUpdates,
    ImmutableArray<RuntimeEffect> RuntimeEffects,
    ImmutableArray<string> DiagnosticEvents);
