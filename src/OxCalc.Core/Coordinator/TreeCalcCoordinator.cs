using System.Collections.Immutable;
using OxCalc.Core.Structural;

namespace OxCalc.Core.Coordinator;

public sealed record CoordinatorCounters(int PublicationCount, int RejectCount, int PinCount)
{
    public CoordinatorCounters IncrementPublications() => this with { PublicationCount = PublicationCount + 1 };

    public CoordinatorCounters IncrementRejects() => this with { RejectCount = RejectCount + 1 };

    public CoordinatorCounters IncrementPins() => this with { PinCount = PinCount + 1 };
}

public sealed class TreeCalcCoordinator
{
    private readonly Dictionary<string, PinnedPublicationView> _pins = new(StringComparer.Ordinal);
    private readonly List<RejectDetail> _rejectLog = [];

    public TreeCalcCoordinator(StructuralSnapshot snapshot)
    {
        Snapshot = snapshot;
        PublishedView = new PublishedView(snapshot, null, ImmutableDictionary<TreeNodeId, string>.Empty);
        Counters = new CoordinatorCounters(0, 0, 0);
    }

    public StructuralSnapshot Snapshot { get; }

    public AcceptedCandidateResult? InFlightCandidate { get; private set; }

    public AcceptedCandidateResult? AcceptedCandidate { get; private set; }

    public PublishedView PublishedView { get; private set; }

    public CoordinatorCounters Counters { get; private set; }

    public IReadOnlyList<RejectDetail> RejectLog => _rejectLog;

    public IReadOnlyCollection<PinnedPublicationView> PinnedReaders => _pins.Values;

    public void AdmitCandidateWork(AcceptedCandidateResult candidate)
    {
        EnsureSnapshotMatches(candidate.StructuralSnapshotId);
        InFlightCandidate = candidate;
    }

    public void RecordAcceptedCandidateResult(string candidateResultId)
    {
        if (InFlightCandidate is null || InFlightCandidate.CandidateResultId != candidateResultId)
        {
            throw new InvalidOperationException($"Candidate '{candidateResultId}' is not currently admitted.");
        }

        AcceptedCandidate = InFlightCandidate;
    }

    public PublicationBundle AcceptAndPublish(string publicationId)
    {
        if (AcceptedCandidate is null)
        {
            throw new InvalidOperationException("No accepted candidate result is available for publication.");
        }

        var publishedValues = PublishedView.Values
            .ToImmutableDictionary()
            .SetItems(AcceptedCandidate.ValueUpdates);

        var bundle = new PublicationBundle(
            publicationId,
            AcceptedCandidate.CandidateResultId,
            Snapshot.SnapshotId,
            AcceptedCandidate.ValueUpdates,
            AcceptedCandidate.RuntimeEffects,
            ["publication_committed"]);

        PublishedView = new PublishedView(Snapshot, bundle, publishedValues);
        InFlightCandidate = null;
        AcceptedCandidate = null;
        Counters = Counters.IncrementPublications();
        return bundle;
    }

    public RejectDetail RejectCandidateWork(string candidateResultId, RejectKind kind, string detail)
    {
        if ((InFlightCandidate?.CandidateResultId != candidateResultId) && (AcceptedCandidate?.CandidateResultId != candidateResultId))
        {
            throw new InvalidOperationException($"Candidate '{candidateResultId}' is not currently known to the coordinator.");
        }

        var reject = new RejectDetail(candidateResultId, kind, detail);
        _rejectLog.Add(reject);
        if (InFlightCandidate?.CandidateResultId == candidateResultId)
        {
            InFlightCandidate = null;
        }

        if (AcceptedCandidate?.CandidateResultId == candidateResultId)
        {
            AcceptedCandidate = null;
        }

        Counters = Counters.IncrementRejects();
        return reject;
    }

    public PinnedPublicationView PinReader(string readerId)
    {
        var view = new PinnedPublicationView(
            readerId,
            Snapshot.SnapshotId,
            PublishedView.Publication?.PublicationId,
            Snapshot.Pin(),
            PublishedView.Values.ToImmutableDictionary());
        _pins[readerId] = view;
        Counters = Counters.IncrementPins();
        return view;
    }

    public bool UnpinReader(string readerId) => _pins.Remove(readerId);

    private void EnsureSnapshotMatches(StructuralSnapshotId snapshotId)
    {
        if (Snapshot.SnapshotId != snapshotId)
        {
            throw new InvalidOperationException(
                $"Candidate snapshot '{snapshotId}' does not match coordinator snapshot '{Snapshot.SnapshotId}'.");
        }
    }
}
