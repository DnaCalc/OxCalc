using System.Collections.Immutable;
using OxCalc.Core.Structural;

namespace OxCalc.Core.Coordinator;

public sealed record PublicationBundle(
    string PublicationId,
    string CandidateResultId,
    StructuralSnapshotId StructuralSnapshotId,
    ImmutableDictionary<TreeNodeId, string> PublishedViewDelta,
    ImmutableArray<RuntimeEffect> PublishedRuntimeEffects,
    ImmutableArray<string> TraceMarkers);
