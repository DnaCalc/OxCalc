namespace OxCalc.Core.TraceCalc;

internal static class TraceCalcReplayMappings
{
    public static string NormalizeEventFamily(string sourceLabel) => sourceLabel switch
    {
        "candidate_admitted" => "candidate.admitted",
        "candidate_recorded" => "candidate.built",
        "candidate_emitted" => "candidate.built",
        "candidate_rejected" => "reject.issued",
        "publication_committed" => "publication.committed",
        "candidate_published" => "publication.committed",
        "reader_pinned" => "session.reader_pinned",
        "reader_unpinned" => "session.reader_unpinned",
        "overlay_retained" => "overlay.retained",
        "overlay_released" => "overlay.released",
        "node_verified_clean" => "candidate.verified_clean",
        "fallback_forced" => "candidate.fallback_forced",
        "eviction_eligibility_opened" => "overlay.eviction_eligible",
        _ => $"oxcalc.local.event.{sourceLabel}"
    };

    public static string ToRegistryMismatchKind(TraceCalcConformanceMismatchKind kind) => kind switch
    {
        TraceCalcConformanceMismatchKind.MissingScenarioResult => "mm.scenario.presence",
        TraceCalcConformanceMismatchKind.ResultStateMismatch => "mm.result.state",
        TraceCalcConformanceMismatchKind.PublishedViewMismatch => "mm.view.value",
        TraceCalcConformanceMismatchKind.PinnedViewMismatch => "mm.view.value",
        TraceCalcConformanceMismatchKind.RejectMismatch => "mm.reject.kind",
        TraceCalcConformanceMismatchKind.TraceCountMismatch => "mm.trace.event",
        TraceCalcConformanceMismatchKind.CounterMismatch => "mm.counter.value",
        TraceCalcConformanceMismatchKind.UnexpectedExtraArtifact => "mm.sidecar.payload",
        _ => "oxcalc.local.mismatch.unknown"
    };

    public static string ToSeverityClass(TraceCalcConformanceMismatchKind kind) => kind switch
    {
        TraceCalcConformanceMismatchKind.TraceCountMismatch => "sev.instrumentation",
        TraceCalcConformanceMismatchKind.CounterMismatch => "sev.instrumentation",
        TraceCalcConformanceMismatchKind.UnexpectedExtraArtifact => "sev.informational",
        _ => "sev.semantic"
    };

    public static string ToRequiredEqualitySurface(TraceCalcConformanceMismatchKind kind) => kind switch
    {
        TraceCalcConformanceMismatchKind.ResultStateMismatch => "assertion_result_set",
        TraceCalcConformanceMismatchKind.PublishedViewMismatch => "published_view",
        TraceCalcConformanceMismatchKind.PinnedViewMismatch => "pinned_view",
        TraceCalcConformanceMismatchKind.RejectMismatch => "reject_set",
        TraceCalcConformanceMismatchKind.TraceCountMismatch => "trace_labels",
        TraceCalcConformanceMismatchKind.CounterMismatch => "counters",
        TraceCalcConformanceMismatchKind.MissingScenarioResult => "assertion_result_set",
        TraceCalcConformanceMismatchKind.UnexpectedExtraArtifact => "oxcalc.local.optional_sidecar",
        _ => "oxcalc.local.unknown_surface"
    };
}
