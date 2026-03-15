using System.Collections.Immutable;
using OxCalc.Core.Coordinator;
using OxCalc.Core.Structural;

namespace OxCalc.Core.Tests;

public class TreeCalcCoordinatorTests
{
    [Fact]
    public void CandidateResultIsNotPublishedUntilAcceptAndPublish()
    {
        var snapshot = StructuralSnapshotTestData.CreateSeedSnapshot(new StructuralSnapshotId(1));
        var coordinator = new TreeCalcCoordinator(snapshot);
        var candidate = CreateCandidate(snapshot.SnapshotId, "cand-1", new TreeNodeId(3), "42");

        coordinator.AdmitCandidateWork(candidate);
        coordinator.RecordAcceptedCandidateResult(candidate.CandidateResultId);

        Assert.Null(coordinator.PublishedView.Publication);
        Assert.Empty(coordinator.PublishedView.Values);

        var bundle = coordinator.AcceptAndPublish("pub-1");

        Assert.Equal("pub-1", bundle.PublicationId);
        Assert.Equal("42", coordinator.PublishedView.Values[new TreeNodeId(3)]);
    }

    [Fact]
    public void RejectDoesNotAdvancePublishedView()
    {
        var snapshot = StructuralSnapshotTestData.CreateSeedSnapshot(new StructuralSnapshotId(1));
        var coordinator = new TreeCalcCoordinator(snapshot);
        var initial = CreateCandidate(snapshot.SnapshotId, "cand-1", new TreeNodeId(3), "42");
        coordinator.AdmitCandidateWork(initial);
        coordinator.RecordAcceptedCandidateResult(initial.CandidateResultId);
        coordinator.AcceptAndPublish("pub-1");

        var rejected = CreateCandidate(snapshot.SnapshotId, "cand-2", new TreeNodeId(3), "99");
        coordinator.AdmitCandidateWork(rejected);
        coordinator.RecordAcceptedCandidateResult(rejected.CandidateResultId);
        coordinator.RejectCandidateWork(rejected.CandidateResultId, RejectKind.PublicationFenceMismatch, "fence drift");

        Assert.Equal("pub-1", coordinator.PublishedView.Publication?.PublicationId);
        Assert.Equal("42", coordinator.PublishedView.Values[new TreeNodeId(3)]);
        Assert.Single(coordinator.RejectLog);
    }

    [Fact]
    public void PinnedReaderRetainsPriorPublicationAfterLaterPublish()
    {
        var snapshot = StructuralSnapshotTestData.CreateSeedSnapshot(new StructuralSnapshotId(1));
        var coordinator = new TreeCalcCoordinator(snapshot);
        var first = CreateCandidate(snapshot.SnapshotId, "cand-1", new TreeNodeId(3), "42");
        coordinator.AdmitCandidateWork(first);
        coordinator.RecordAcceptedCandidateResult(first.CandidateResultId);
        coordinator.AcceptAndPublish("pub-1");

        var pinned = coordinator.PinReader("reader-1");

        var second = CreateCandidate(snapshot.SnapshotId, "cand-2", new TreeNodeId(3), "99");
        coordinator.AdmitCandidateWork(second);
        coordinator.RecordAcceptedCandidateResult(second.CandidateResultId);
        coordinator.AcceptAndPublish("pub-2");

        Assert.Equal("pub-1", pinned.PublicationId);
        Assert.Equal("42", pinned.Values[new TreeNodeId(3)]);
        Assert.Equal("pub-2", coordinator.PublishedView.Publication?.PublicationId);
        Assert.Equal("99", coordinator.PublishedView.Values[new TreeNodeId(3)]);
    }

    private static AcceptedCandidateResult CreateCandidate(
        StructuralSnapshotId snapshotId,
        string candidateResultId,
        TreeNodeId targetNodeId,
        string publishedValue)
    {
        return new AcceptedCandidateResult(
            candidateResultId,
            snapshotId,
            "artifact:v1",
            "compat:v1",
            [targetNodeId],
            ImmutableDictionary<TreeNodeId, string>.Empty.Add(targetNodeId, publishedValue),
            ImmutableArray<DependencyShapeUpdate>.Empty,
            [new RuntimeEffect("format_observed", "none")],
            ["candidate_recorded"]);
    }
}
