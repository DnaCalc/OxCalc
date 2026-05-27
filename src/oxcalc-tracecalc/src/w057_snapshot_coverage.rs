#![forbid(unsafe_code)]

//! W057 snapshot-layer TraceCalc coverage and differential mapping.

use serde_json::{Value, json};

const W057_COVERAGE_SCHEMA_V1: &str = "oxcalc.tracecalc.w057_snapshot_coverage.v1";

#[must_use]
pub fn w057_snapshot_coverage_packet(run_id: &str) -> Value {
    let rows = w057_snapshot_coverage_rows();
    let covered_or_blocked_count = rows
        .iter()
        .filter(|row| {
            matches!(
                row["coverage_state"].as_str(),
                Some("covered_by_tracecalc_and_optimized_fixture")
                    | Some("covered_by_combined_tracecalc_and_optimized_fixture")
                    | Some("blocked_exact_tracecalc_language_gap")
            )
        })
        .count();
    let exact_blocker_count = rows
        .iter()
        .filter(|row| {
            row.get("exact_blocker")
                .is_some_and(|blocker| !blocker.is_null())
        })
        .count();

    json!({
        "schema_version": W057_COVERAGE_SCHEMA_V1,
        "run_id": run_id,
        "workset": "W057",
        "purpose": "TraceCalc and optimized TreeCalc differential coverage for the W056 epoch/snapshot scenario set under the W057 snapshot-layer vocabulary.",
        "coverage_policy": "Each row must either name TraceCalc oracle evidence plus an optimized fixture, or carry an exact blocker that explains why TraceCalc cannot yet express the scenario language directly.",
        "row_count": rows.len(),
        "covered_or_blocked_count": covered_or_blocked_count,
        "exact_blocker_count": exact_blocker_count,
        "rows": rows,
    })
}

#[must_use]
pub fn w057_snapshot_coverage_rows() -> Vec<Value> {
    vec![
        covered(
            "w057_value_update_epoch_split",
            "input value update advances node input epoch without structural snapshot change",
            "tc_w035_dirty_seed_closure_no_under_invalidation_001",
            "treecalc_context_input_value_update_recalculates_dependents_without_full_reset",
            "TraceCalc covers dirty-seed downstream closure and publication; the optimized fixture proves the W057 structure/node-input epoch split.",
        ),
        blocked(
            "w057_formula_same_dependency_shape",
            "formula text update with unchanged dependency shape recalculates without dependency-shape publication",
            "treecalc_context_formula_edit_same_host_reference_target_ignores_source_span_handle",
            "TRC-W057-004",
            "TraceCalc can express verify-clean/no-publish, but it cannot yet express an authored formula text edit whose typed dependency shape remains unchanged.",
        ),
        covered(
            "w057_formula_changed_dependency_shape",
            "formula text update with changed static dependency shape publishes dependency-shape delta",
            "tc_w035_static_dependency_add_publish_001",
            "treecalc_context_formula_edit_changed_dependency_preserves_structure_and_recalculates",
            "TraceCalc covers static dependency-shape publication and the optimized fixture proves structure identity is preserved.",
        ),
        combined(
            "w057_literal_to_formula_activation",
            "literal-to-formula transition publishes activated dependency-shape facts",
            "tc_w035_static_dependency_add_publish_001",
            "treecalc_context_literal_to_formula_preserves_structure_and_publishes_activation",
            "TraceCalc covers the activation publication surface; the optimized fixture covers the input-kind transition and structure preservation.",
        ),
        blocked(
            "w057_formula_to_literal_release",
            "formula-to-literal transition releases formula dependency-shape facts",
            "treecalc_context_formula_to_literal_preserves_structure_and_publishes_release",
            "TRC-W057-001",
            "TraceCalc has dynamic dependency release rows but no authored static formula-to-literal edit step; add a TraceCalc node-input edit operation before claiming direct oracle coverage for this transition.",
        ),
        blocked(
            "w057_dynamic_target_value_update",
            "value update of a previously published dynamic target invalidates through the old effective graph",
            "treecalc_context_indirect_resolves_reference_text_and_records_ctro_edge",
            "TRC-W057-005",
            "TraceCalc has dynamic switch and dirty-closure rows, but no direct scenario step for editing the value of a previously published dynamic target while invalidating through the old CTRO effective graph.",
        ),
        covered(
            "w057_dynamic_target_switch",
            "dynamic target switch publishes runtime dependency-shape/effect delta",
            "tc_w035_dynamic_dependency_switch_publish_001",
            "treecalc_context_indirect_resolves_reference_text_and_records_ctro_edge",
            "TraceCalc covers dynamic switch publication and the optimized fixture covers the local INDIRECT CTRO edge.",
        ),
        blocked(
            "w057_unresolved_to_resolved_formula_edit",
            "formula edit transitions an unresolved reference to a resolved dependency",
            "treecalc_context_formula_edit_unresolved_to_resolved_preserves_structure",
            "TRC-W057-002",
            "TraceCalc can express unresolved dynamic reject and static dependency add separately, but cannot yet express a formula text edit from unresolved to resolved in one scenario.",
        ),
        combined(
            "w057_resolved_to_unresolved_formula_edit",
            "formula edit transitions a resolved dependency to unresolved and rejects without publication",
            "tc_w034_dynamic_dependency_negative_001",
            "treecalc_context_formula_edit_resolved_to_unresolved_rejects_without_structural_change",
            "TraceCalc covers unresolved dependency reject/no-publish; the optimized fixture covers the resolved-to-unresolved formula edit transition.",
        ),
        blocked(
            "w057_rename_delete_move_structural_edits",
            "rename/delete/move structural edits advance structure snapshot and preserve compatible layer facts",
            "treecalc_context_structural_edits_advance_revision_roots_and_preserve_inputs;treecalc_context_delete_prunes_inputs_publication_runtime_and_table_shape",
            "TRC-W057-003",
            "TraceCalc has no structural edit step language for rename, delete, move, reorder, or table-shape edits; W057 optimized fixtures are the current executable evidence.",
        ),
        covered(
            "w057_ctro_cycle_reject",
            "candidate CTRO dependency shape that creates a cycle rejects without publishing",
            "tc_w048_ctro_candidate_cycle_reject_001",
            "treecalc_context_formula_edit_cycle_reject_preserves_structure_and_prior_publication;treecalc_context_literal_to_formula_cycle_reject_preserves_prior_literal_value",
            "TraceCalc covers CTRO candidate cycle reject and optimized fixtures cover prior publication preservation across formula/literal transition rejects.",
        ),
    ]
}

fn covered(
    row_id: &str,
    surface: &str,
    tracecalc_scenario_id: &str,
    optimized_fixture_ref: &str,
    rationale: &str,
) -> Value {
    json!({
        "row_id": row_id,
        "surface": surface,
        "coverage_state": "covered_by_tracecalc_and_optimized_fixture",
        "tracecalc_scenario_id": tracecalc_scenario_id,
        "optimized_fixture_ref": optimized_fixture_ref,
        "rationale": rationale,
        "exact_blocker": Value::Null,
    })
}

fn combined(
    row_id: &str,
    surface: &str,
    tracecalc_scenario_id: &str,
    optimized_fixture_ref: &str,
    rationale: &str,
) -> Value {
    json!({
        "row_id": row_id,
        "surface": surface,
        "coverage_state": "covered_by_combined_tracecalc_and_optimized_fixture",
        "tracecalc_scenario_id": tracecalc_scenario_id,
        "optimized_fixture_ref": optimized_fixture_ref,
        "rationale": rationale,
        "exact_blocker": Value::Null,
    })
}

fn blocked(
    row_id: &str,
    surface: &str,
    optimized_fixture_ref: &str,
    blocker_id: &str,
    blocker_detail: &str,
) -> Value {
    json!({
        "row_id": row_id,
        "surface": surface,
        "coverage_state": "blocked_exact_tracecalc_language_gap",
        "tracecalc_scenario_id": Value::Null,
        "optimized_fixture_ref": optimized_fixture_ref,
        "rationale": "Optimized fixture exists, but TraceCalc needs a scenario-language extension before this can become direct oracle coverage.",
        "exact_blocker": {
            "blocker_id": blocker_id,
            "detail": blocker_detail,
        },
    })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::fs;
    use std::path::Path;

    use super::*;
    use crate::contracts::load_manifest;

    #[test]
    fn w057_snapshot_coverage_rows_are_all_covered_or_exactly_blocked() {
        let packet = w057_snapshot_coverage_packet("test");
        assert_eq!(packet["schema_version"], W057_COVERAGE_SCHEMA_V1);
        assert_eq!(packet["row_count"], 11);
        assert_eq!(packet["covered_or_blocked_count"], packet["row_count"]);

        for row in packet["rows"].as_array().unwrap() {
            assert!(
                row["optimized_fixture_ref"]
                    .as_str()
                    .is_some_and(|value| !value.is_empty())
            );
            let has_tracecalc = row
                .get("tracecalc_scenario_id")
                .and_then(Value::as_str)
                .is_some_and(|value| !value.is_empty());
            let has_blocker = row
                .get("exact_blocker")
                .and_then(|blocker| blocker.get("blocker_id"))
                .and_then(Value::as_str)
                .is_some_and(|value| !value.is_empty());
            assert!(
                has_tracecalc || has_blocker,
                "row {} needs TraceCalc coverage or an exact blocker",
                row["row_id"]
            );
        }
    }

    #[test]
    fn w057_snapshot_coverage_refs_existing_tracecalc_and_optimized_fixtures() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let manifest =
            load_manifest(&repo_root.join("docs/test-corpus/core-engine/tracecalc/MANIFEST.json"))
                .unwrap();
        let tracecalc_ids = manifest
            .scenarios
            .iter()
            .map(|scenario| scenario.scenario_id.as_str())
            .collect::<BTreeSet<_>>();
        let consumer_source =
            fs::read_to_string(repo_root.join("src/oxcalc-core/src/consumer.rs")).unwrap();

        for row in w057_snapshot_coverage_rows() {
            if let Some(scenario_id) = row["tracecalc_scenario_id"].as_str() {
                assert!(
                    tracecalc_ids.contains(scenario_id),
                    "unknown TraceCalc scenario {scenario_id}"
                );
            }
            for fixture_ref in row["optimized_fixture_ref"].as_str().unwrap().split(';') {
                assert!(
                    consumer_source.contains(&format!("fn {fixture_ref}(")),
                    "unknown optimized fixture {fixture_ref}"
                );
            }
        }
    }
}
