#![forbid(unsafe_code)]

//! Assertion and conformance helpers.

use std::collections::BTreeMap;

use crate::contracts::{
    TraceCalcConformanceMismatch, TraceCalcConformanceMismatchKind, TraceCalcExecutionArtifacts,
    TraceCalcRejectRecord, TraceCalcScenario, TraceCalcTraceEvent,
};

pub fn evaluate_assertions(
    scenario: &TraceCalcScenario,
    published_values: &[(String, String)],
    pinned_views: &[(String, Vec<(String, String)>)],
    trace_events: &[TraceCalcTraceEvent],
    counters: &[(String, i64)],
    rejects: &[TraceCalcRejectRecord],
) -> Vec<String> {
    let mut failures = Vec::new();
    let published_map = published_values.iter().cloned().collect::<BTreeMap<_, _>>();

    for expected_value in &scenario.expected.published_view.node_values {
        let observed = published_map.get(&expected_value.node_id);
        if observed != Some(&expected_value.value) {
            failures.push(format!(
                "Published value mismatch for node '{}': expected '{}' but observed '{}'.",
                expected_value.node_id,
                expected_value.value,
                observed.cloned().unwrap_or_else(|| "<missing>".to_string())
            ));
        }
    }

    let pinned_view_map = pinned_views.iter().cloned().collect::<BTreeMap<_, _>>();
    for expected_view in &scenario.expected.pinned_views {
        let Some(observed_view) = pinned_view_map.get(&expected_view.view_id) else {
            failures.push(format!("Missing pinned view '{}'.", expected_view.view_id));
            continue;
        };
        let observed_map = observed_view.iter().cloned().collect::<BTreeMap<_, _>>();
        for expected_value in &expected_view.node_values {
            let observed = observed_map.get(&expected_value.node_id);
            if observed != Some(&expected_value.value) {
                failures.push(format!(
                    "Pinned view mismatch for '{}' node '{}': expected '{}' but observed '{}'.",
                    expected_view.view_id,
                    expected_value.node_id,
                    expected_value.value,
                    observed.cloned().unwrap_or_else(|| "<missing>".to_string())
                ));
            }
        }
    }

    let mut trace_counts = BTreeMap::new();
    for event in trace_events {
        *trace_counts.entry(event.label.clone()).or_insert(0_i64) += 1;
    }
    for expectation in &scenario.expected.trace_labels {
        let observed = trace_counts
            .get(&expectation.label)
            .copied()
            .unwrap_or_default();
        if observed != expectation.count {
            failures.push(format!(
                "Trace label count mismatch for '{}': expected {} but observed {}.",
                expectation.label, expectation.count, observed
            ));
        }
    }

    let counter_map = counters.iter().cloned().collect::<BTreeMap<_, _>>();
    for expectation in &scenario.expected.counter_expectations {
        let observed = counter_map
            .get(&expectation.counter)
            .copied()
            .unwrap_or_default();
        if !compare_counter(observed, &expectation.comparison, expectation.value) {
            failures.push(format!(
                "Counter mismatch for '{}': expected {} {} but observed {}.",
                expectation.counter, expectation.comparison, expectation.value, observed
            ));
        }
    }

    let reject_map = rejects
        .iter()
        .cloned()
        .map(|reject| (reject.reject_id.clone(), reject))
        .collect::<BTreeMap<_, _>>();
    for expectation in &scenario.expected.rejects {
        let Some(observed_reject) = reject_map.get(&expectation.reject_id) else {
            failures.push(format!("Missing reject '{}'.", expectation.reject_id));
            continue;
        };
        if observed_reject.reject_kind != expectation.reject_kind {
            failures.push(format!(
                "Reject kind mismatch for '{}': expected '{}' but observed '{}'.",
                expectation.reject_id, expectation.reject_kind, observed_reject.reject_kind
            ));
        }
        if let Some(detail_contains) = &expectation.detail_contains
            && !observed_reject
                .reject_detail
                .to_ascii_lowercase()
                .contains(&detail_contains.to_ascii_lowercase())
        {
            failures.push(format!(
                "Reject detail mismatch for '{}': expected detail containing '{}'.",
                expectation.reject_id, detail_contains
            ));
        }
    }

    failures
}

pub fn compare_artifacts(
    oracle: &TraceCalcExecutionArtifacts,
    engine: &TraceCalcExecutionArtifacts,
) -> Vec<TraceCalcConformanceMismatch> {
    let mut mismatches = Vec::new();
    if oracle.result_state != engine.result_state {
        mismatches.push(TraceCalcConformanceMismatch {
            kind: TraceCalcConformanceMismatchKind::ResultStateMismatch,
            message: format!(
                "Result-state mismatch: oracle '{:?}', engine '{:?}'.",
                oracle.result_state, engine.result_state
            ),
        });
    }

    let oracle_published = oracle
        .published_values
        .iter()
        .cloned()
        .collect::<BTreeMap<_, _>>();
    let engine_published = engine
        .published_values
        .iter()
        .cloned()
        .collect::<BTreeMap<_, _>>();
    for (node_id, expected_value) in &oracle_published {
        if engine_published.get(node_id) != Some(expected_value) {
            mismatches.push(TraceCalcConformanceMismatch {
                kind: TraceCalcConformanceMismatchKind::PublishedViewMismatch,
                message: format!("Published view mismatch for node '{}'.", node_id),
            });
        }
    }

    let oracle_pinned = oracle
        .pinned_views
        .iter()
        .map(|view| (view.view_id.clone(), view.node_values.clone()))
        .collect::<BTreeMap<_, _>>();
    let engine_pinned = engine
        .pinned_views
        .iter()
        .map(|view| (view.view_id.clone(), view.node_values.clone()))
        .collect::<BTreeMap<_, _>>();
    for (view_id, oracle_values) in oracle_pinned {
        let Some(engine_values) = engine_pinned.get(&view_id) else {
            mismatches.push(TraceCalcConformanceMismatch {
                kind: TraceCalcConformanceMismatchKind::PinnedViewMismatch,
                message: format!("Missing pinned view '{}' in engine output.", view_id),
            });
            continue;
        };
        let engine_map = engine_values.iter().cloned().collect::<BTreeMap<_, _>>();
        for (node_id, expected_value) in oracle_values {
            if engine_map.get(&node_id) != Some(&expected_value) {
                mismatches.push(TraceCalcConformanceMismatch {
                    kind: TraceCalcConformanceMismatchKind::PinnedViewMismatch,
                    message: format!("Pinned-view mismatch for '{}' node '{}'.", view_id, node_id),
                });
            }
        }
    }

    let oracle_rejects = oracle
        .rejects
        .iter()
        .map(|reject| {
            format!(
                "{}:{}:{}",
                reject.reject_id, reject.reject_kind, reject.reject_detail
            )
        })
        .collect::<Vec<_>>();
    let engine_rejects = engine
        .rejects
        .iter()
        .map(|reject| {
            format!(
                "{}:{}:{}",
                reject.reject_id, reject.reject_kind, reject.reject_detail
            )
        })
        .collect::<Vec<_>>();
    if oracle_rejects != engine_rejects {
        mismatches.push(TraceCalcConformanceMismatch {
            kind: TraceCalcConformanceMismatchKind::RejectMismatch,
            message: "Reject outputs differ between oracle and engine.".to_string(),
        });
    }

    let oracle_trace_counts = count_trace_labels(&oracle.trace_events);
    let engine_trace_counts = count_trace_labels(&engine.trace_events);
    for (label, expected_count) in oracle_trace_counts {
        if engine_trace_counts.get(&label).copied().unwrap_or_default() != expected_count {
            mismatches.push(TraceCalcConformanceMismatch {
                kind: TraceCalcConformanceMismatchKind::TraceCountMismatch,
                message: format!("Trace count mismatch for label '{}'.", label),
            });
        }
    }

    let oracle_counters = oracle.counters.iter().cloned().collect::<BTreeMap<_, _>>();
    let engine_counters = engine.counters.iter().cloned().collect::<BTreeMap<_, _>>();
    for (counter, expected_value) in oracle_counters {
        if engine_counters.get(&counter).copied().unwrap_or_default() != expected_value {
            mismatches.push(TraceCalcConformanceMismatch {
                kind: TraceCalcConformanceMismatchKind::CounterMismatch,
                message: format!("Counter mismatch for '{}'.", counter),
            });
        }
    }

    mismatches
}

pub fn to_snake_case(value: &str) -> String {
    if value.is_empty() {
        return String::new();
    }

    let mut output = String::with_capacity(value.len() + 8);
    for (index, character) in value.chars().enumerate() {
        if character.is_ascii_uppercase() && index > 0 {
            output.push('_');
        }
        output.push(character.to_ascii_lowercase());
    }
    output
}

fn compare_counter(observed: i64, comparison: &str, expected: i64) -> bool {
    match comparison {
        "eq" => observed == expected,
        "ge" => observed >= expected,
        "gt" => observed > expected,
        "le" => observed <= expected,
        "lt" => observed < expected,
        _ => false,
    }
}

fn count_trace_labels(trace_events: &[TraceCalcTraceEvent]) -> BTreeMap<String, i64> {
    let mut counts = BTreeMap::new();
    for event in trace_events {
        *counts.entry(event.label.clone()).or_insert(0_i64) += 1;
    }
    counts
}
