#![forbid(unsafe_code)]

//! Replay-facing mapping helpers.

use crate::contracts::TraceCalcConformanceMismatchKind;

pub fn normalize_event_family(source_label: &str) -> String {
    match source_label {
        "candidate_admitted" => "candidate.admitted".to_string(),
        "candidate_recorded" | "candidate_emitted" => "candidate.built".to_string(),
        "candidate_rejected" => "reject.issued".to_string(),
        "publication_committed" | "candidate_published" => "publication.committed".to_string(),
        "reader_pinned" => "session.reader_pinned".to_string(),
        "reader_unpinned" => "session.reader_unpinned".to_string(),
        "overlay_retained" => "overlay.retained".to_string(),
        "overlay_released" => "overlay.released".to_string(),
        "node_verified_clean" => "candidate.verified_clean".to_string(),
        "fallback_forced" => "candidate.fallback_forced".to_string(),
        "eviction_eligibility_opened" => "overlay.eviction_eligible".to_string(),
        _ => format!("oxcalc.local.event.{source_label}"),
    }
}

pub fn registry_mismatch_kind(kind: TraceCalcConformanceMismatchKind) -> &'static str {
    match kind {
        TraceCalcConformanceMismatchKind::MissingScenarioResult => "mm.scenario.presence",
        TraceCalcConformanceMismatchKind::ResultStateMismatch => "mm.result.state",
        TraceCalcConformanceMismatchKind::PublishedViewMismatch => "mm.view.value",
        TraceCalcConformanceMismatchKind::PinnedViewMismatch => "mm.view.value",
        TraceCalcConformanceMismatchKind::RejectMismatch => "mm.reject.kind",
        TraceCalcConformanceMismatchKind::TraceCountMismatch => "mm.trace.event",
        TraceCalcConformanceMismatchKind::CounterMismatch => "mm.counter.value",
        TraceCalcConformanceMismatchKind::UnexpectedExtraArtifact => "mm.sidecar.payload",
    }
}

pub fn severity_class(kind: TraceCalcConformanceMismatchKind) -> &'static str {
    match kind {
        TraceCalcConformanceMismatchKind::TraceCountMismatch
        | TraceCalcConformanceMismatchKind::CounterMismatch => "sev.instrumentation",
        TraceCalcConformanceMismatchKind::UnexpectedExtraArtifact => "sev.informational",
        _ => "sev.semantic",
    }
}

pub fn required_equality_surface(kind: TraceCalcConformanceMismatchKind) -> &'static str {
    match kind {
        TraceCalcConformanceMismatchKind::ResultStateMismatch => "assertion_result_set",
        TraceCalcConformanceMismatchKind::PublishedViewMismatch => "published_view",
        TraceCalcConformanceMismatchKind::PinnedViewMismatch => "pinned_view",
        TraceCalcConformanceMismatchKind::RejectMismatch => "reject_set",
        TraceCalcConformanceMismatchKind::TraceCountMismatch => "trace_labels",
        TraceCalcConformanceMismatchKind::CounterMismatch => "counters",
        TraceCalcConformanceMismatchKind::MissingScenarioResult => "assertion_result_set",
        TraceCalcConformanceMismatchKind::UnexpectedExtraArtifact => {
            "oxcalc.local.optional_sidecar"
        }
    }
}
