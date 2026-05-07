#![forbid(unsafe_code)]

//! Direct OxFml fixture intake and OxCalc-owned projection artifacts.

use std::fs;
use std::path::Path;

use serde_json::{Value, json};
use thiserror::Error;

const BRIDGE_RUN_SUMMARY_SCHEMA_V1: &str = "oxcalc.oxfml_fixture_bridge.run_summary.v1";
const BRIDGE_FIXTURE_INDEX_SCHEMA_V1: &str = "oxcalc.oxfml_fixture_bridge.fixture_index.v1";
const BRIDGE_FAMILY_PROJECTION_SCHEMA_V1: &str = "oxcalc.oxfml_fixture_bridge.family_projection.v1";
const BRIDGE_COMPARISON_SCHEMA_V1: &str = "oxcalc.oxfml_fixture_bridge.comparison_summary.v1";
const BRIDGE_HANDOFF_WATCH_SCHEMA_V1: &str = "oxcalc.oxfml_fixture_bridge.handoff_watch.v1";
const BRIDGE_REPLAY_BUNDLE_SCHEMA_V1: &str = "oxcalc.oxfml_fixture_bridge.bundle_manifest.v1";
const BRIDGE_REPLAY_VALIDATION_SCHEMA_V1: &str = "oxcalc.oxfml_fixture_bridge.bundle_validation.v1";

const TRACECALC_W033_RUN_ID: &str = "w033-tracecalc-oracle-self-check-001";
const TREECALC_W033_RUN_ID: &str = "w033-treecalc-witness-bridge-001";

#[derive(Debug, Error)]
pub enum OxFmlFixtureBridgeError {
    #[error("missing OxFml fixture root {path}")]
    MissingFixtureRoot { path: String },
    #[error("failed to create directory {path}: {source}")]
    CreateDirectory {
        path: String,
        source: std::io::Error,
    },
    #[error("failed to remove existing artifact root {path}: {source}")]
    RemoveDirectory {
        path: String,
        source: std::io::Error,
    },
    #[error("failed to read fixture {path}: {source}")]
    ReadFixture {
        path: String,
        source: std::io::Error,
    },
    #[error("failed to read evidence artifact {path}: {source}")]
    ReadEvidence {
        path: String,
        source: std::io::Error,
    },
    #[error("failed to parse JSON from {path}: {source}")]
    ParseJson {
        path: String,
        source: serde_json::Error,
    },
    #[error("fixture {path} must contain a JSON array")]
    FixtureNotArray { path: String },
    #[error("fixture case in {path} is missing string case_id")]
    MissingCaseId { path: String },
    #[error("failed to write artifact {path}: {source}")]
    WriteFile {
        path: String,
        source: std::io::Error,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OxFmlFixtureBridgeRunSummary {
    pub run_id: String,
    pub schema_version: String,
    pub family_count: usize,
    pub fixture_case_count: usize,
    pub comparison_row_count: usize,
    pub matched_comparison_count: usize,
    pub deferred_comparison_count: usize,
    pub missing_evidence_count: usize,
    pub mismatch_count: usize,
    pub handoff_triggered: bool,
    pub artifact_root: String,
}

#[derive(Debug, Clone, Copy)]
struct FamilySpec {
    family_id: &'static str,
    file_name: &'static str,
    semantic_surface: &'static str,
    comparison_scope: &'static str,
}

#[derive(Debug, Clone, Default)]
pub struct OxFmlFixtureBridgeRunner;

impl OxFmlFixtureBridgeRunner {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    pub fn execute(
        &self,
        repo_root: &Path,
        run_id: &str,
    ) -> Result<OxFmlFixtureBridgeRunSummary, OxFmlFixtureBridgeError> {
        let source_root = repo_root.join("../OxFml/crates/oxfml_core/tests/fixtures");
        if !source_root.exists() {
            return Err(OxFmlFixtureBridgeError::MissingFixtureRoot {
                path: source_root.display().to_string(),
            });
        }

        let artifact_root = repo_root.join(format!(
            "docs/test-runs/core-engine/oxfml-fixture-bridge/{run_id}"
        ));
        let relative_artifact_root = relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "oxfml-fixture-bridge",
            run_id,
        ]);

        if artifact_root.exists() {
            fs::remove_dir_all(&artifact_root).map_err(|source| {
                OxFmlFixtureBridgeError::RemoveDirectory {
                    path: artifact_root.display().to_string(),
                    source,
                }
            })?;
        }

        create_directory(&artifact_root)?;
        create_directory(&artifact_root.join("family-projections"))?;
        create_directory(&artifact_root.join("comparisons"))?;
        create_directory(&artifact_root.join("handoff"))?;
        create_directory(&artifact_root.join("replay-appliance"))?;
        create_directory(&artifact_root.join("replay-appliance/validation"))?;

        let mut family_index_entries = Vec::new();
        let mut all_case_projections = Vec::new();

        for spec in family_specs() {
            let source_path = source_root.join(spec.file_name);
            let source_relative_path = format!(
                "../OxFml/crates/oxfml_core/tests/fixtures/{}",
                spec.file_name
            );
            let cases = load_fixture_cases(&source_path)?;
            let case_projections = cases
                .iter()
                .map(|case| project_case(spec, &source_relative_path, case))
                .collect::<Result<Vec<_>, _>>()?;
            let projection_path = format!(
                "{relative_artifact_root}/family-projections/{}.json",
                spec.family_id
            );
            let projection = json!({
                "schema_version": BRIDGE_FAMILY_PROJECTION_SCHEMA_V1,
                "family_id": spec.family_id,
                "semantic_surface": spec.semantic_surface,
                "comparison_scope": spec.comparison_scope,
                "source_fixture_path": source_relative_path,
                "case_count": case_projections.len(),
                "cases": case_projections,
            });
            write_json(
                &artifact_root
                    .join("family-projections")
                    .join(format!("{}.json", spec.family_id)),
                &projection,
            )?;
            family_index_entries.push(json!({
                "family_id": spec.family_id,
                "semantic_surface": spec.semantic_surface,
                "comparison_scope": spec.comparison_scope,
                "source_fixture_path": source_relative_path,
                "projection_path": projection_path,
                "case_count": projection["case_count"],
            }));
            all_case_projections.extend(
                projection["cases"]
                    .as_array()
                    .expect("projection cases should be an array")
                    .iter()
                    .cloned(),
            );
        }

        let fixture_case_count = all_case_projections.len();
        let fixture_index = json!({
            "schema_version": BRIDGE_FIXTURE_INDEX_SCHEMA_V1,
            "run_id": run_id,
            "source_fixture_root": "../OxFml/crates/oxfml_core/tests/fixtures",
            "artifact_root": relative_artifact_root,
            "family_count": family_index_entries.len(),
            "fixture_case_count": fixture_case_count,
            "families": family_index_entries,
        });
        write_json(&artifact_root.join("fixture_index.json"), &fixture_index)?;

        let comparison_rows = build_comparison_rows(repo_root, &all_case_projections)?;
        let comparison_counts = comparison_counts(&comparison_rows);
        let comparison_summary = json!({
            "schema_version": BRIDGE_COMPARISON_SCHEMA_V1,
            "run_id": run_id,
            "source_fixture_index_path": format!("{relative_artifact_root}/fixture_index.json"),
            "tracecalc_reference_run_id": TRACECALC_W033_RUN_ID,
            "treecalc_reference_run_id": TREECALC_W033_RUN_ID,
            "comparison_row_count": comparison_rows.len(),
            "matched_comparison_count": comparison_counts.matched,
            "deferred_comparison_count": comparison_counts.deferred,
            "missing_evidence_count": comparison_counts.missing_evidence,
            "mismatch_count": comparison_counts.mismatched,
            "rows": comparison_rows,
        });
        write_json(
            &artifact_root.join("comparisons/comparison_summary.json"),
            &comparison_summary,
        )?;

        let handoff_watch = handoff_watch_json(
            run_id,
            &relative_artifact_root,
            comparison_counts.mismatched,
        );
        write_json(
            &artifact_root.join("handoff/handoff_watch.json"),
            &handoff_watch,
        )?;

        let required_artifacts = required_artifacts(run_id);
        let bundle_manifest = json!({
            "schema_version": BRIDGE_REPLAY_BUNDLE_SCHEMA_V1,
            "run_id": run_id,
            "artifact_root": relative_artifact_root,
            "source_fixture_root": "../OxFml/crates/oxfml_core/tests/fixtures",
            "required_root_artifacts": required_artifacts,
            "claimed_capability": "projection_and_current_evidence_comparison",
            "excluded_capabilities": [
                "direct_evaluator_reexecution",
                "pack_grade_replay",
                "general_oxfunc_kernel_validation"
            ],
        });
        write_json(
            &artifact_root.join("replay-appliance/bundle_manifest.json"),
            &bundle_manifest,
        )?;

        let summary = OxFmlFixtureBridgeRunSummary {
            run_id: run_id.to_string(),
            schema_version: BRIDGE_RUN_SUMMARY_SCHEMA_V1.to_string(),
            family_count: family_specs().len(),
            fixture_case_count,
            comparison_row_count: comparison_counts.total,
            matched_comparison_count: comparison_counts.matched,
            deferred_comparison_count: comparison_counts.deferred,
            missing_evidence_count: comparison_counts.missing_evidence,
            mismatch_count: comparison_counts.mismatched,
            handoff_triggered: comparison_counts.mismatched > 0,
            artifact_root: relative_artifact_root.clone(),
        };
        write_json(
            &artifact_root.join("run_summary.json"),
            &json!({
                "schema_version": summary.schema_version,
                "run_id": summary.run_id,
                "family_count": summary.family_count,
                "fixture_case_count": summary.fixture_case_count,
                "comparison_row_count": summary.comparison_row_count,
                "matched_comparison_count": summary.matched_comparison_count,
                "deferred_comparison_count": summary.deferred_comparison_count,
                "missing_evidence_count": summary.missing_evidence_count,
                "mismatch_count": summary.mismatch_count,
                "handoff_triggered": summary.handoff_triggered,
                "artifact_root": summary.artifact_root,
                "fixture_index_path": format!("{relative_artifact_root}/fixture_index.json"),
                "comparison_summary_path": format!("{relative_artifact_root}/comparisons/comparison_summary.json"),
                "handoff_watch_path": format!("{relative_artifact_root}/handoff/handoff_watch.json"),
                "bundle_validation_path": format!("{relative_artifact_root}/replay-appliance/validation/bundle_validation.json"),
            }),
        )?;

        let validation_path =
            artifact_root.join("replay-appliance/validation/bundle_validation.json");
        write_json(
            &validation_path,
            &json!({
                "schema_version": BRIDGE_REPLAY_VALIDATION_SCHEMA_V1,
                "run_id": run_id,
                "status": "pending_final_validation_write"
            }),
        )?;
        let missing_paths = required_artifacts
            .iter()
            .filter(|relative_path| !repo_root.join(relative_path.as_str()).exists())
            .cloned()
            .collect::<Vec<_>>();
        write_json(
            &validation_path,
            &json!({
                "schema_version": BRIDGE_REPLAY_VALIDATION_SCHEMA_V1,
                "run_id": run_id,
                "status": if missing_paths.is_empty() { "bundle_valid" } else { "missing_required_artifacts" },
                "missing_paths": missing_paths,
                "validated_required_artifact_count": required_artifacts.len(),
            }),
        )?;

        Ok(summary)
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct ComparisonCounts {
    total: usize,
    matched: usize,
    deferred: usize,
    missing_evidence: usize,
    mismatched: usize,
}

fn family_specs() -> &'static [FamilySpec] {
    &[
        FamilySpec {
            family_id: "fec_commit",
            file_name: "fec_commit_replay_cases.json",
            semantic_surface: "FEC candidate, commit, fence, and reject facts",
            comparison_scope: "direct current TraceCalc/TreeCalc outcome comparison where mapped",
        },
        FamilySpec {
            family_id: "session_lifecycle",
            file_name: "session_lifecycle_replay_cases.json",
            semantic_surface: "session phase, capability, contention, and commit lifecycle facts",
            comparison_scope: "current outcome comparison for shared accept/reject/fence classes",
        },
        FamilySpec {
            family_id: "prepared_call",
            file_name: "prepared_call_replay_cases.json",
            semantic_surface: "OxFml prepared-call argument and result-carrier facts",
            comparison_scope: "projection-only until OxCalc has direct prepared-call witnesses",
        },
        FamilySpec {
            family_id: "higher_order_callable",
            file_name: "higher_order_callable_cases.json",
            semantic_surface: "higher-order callable result and array payload facts",
            comparison_scope: "projection-only; successor LET/LAMBDA bead owns exercised carrier widening",
        },
    ]
}

fn load_fixture_cases(path: &Path) -> Result<Vec<Value>, OxFmlFixtureBridgeError> {
    let content =
        fs::read_to_string(path).map_err(|source| OxFmlFixtureBridgeError::ReadFixture {
            path: path.display().to_string(),
            source,
        })?;
    let value = serde_json::from_str::<Value>(&content).map_err(|source| {
        OxFmlFixtureBridgeError::ParseJson {
            path: path.display().to_string(),
            source,
        }
    })?;
    value
        .as_array()
        .cloned()
        .ok_or_else(|| OxFmlFixtureBridgeError::FixtureNotArray {
            path: path.display().to_string(),
        })
}

fn project_case(
    spec: &FamilySpec,
    source_relative_path: &str,
    case: &Value,
) -> Result<Value, OxFmlFixtureBridgeError> {
    let Some(case_id) = case.get("case_id").and_then(Value::as_str) else {
        return Err(OxFmlFixtureBridgeError::MissingCaseId {
            path: source_relative_path.to_string(),
        });
    };
    let expected = case.get("expected").unwrap_or(&Value::Null);
    let expected_decision = expected.get("decision").and_then(Value::as_str);
    let expected_payload = expected
        .get("published_payload")
        .or_else(|| expected.get("payload_summary"))
        .and_then(Value::as_str);
    let expected_reject_code = expected.get("reject_code").and_then(Value::as_str);
    let prepared_function_ids = expected
        .get("prepared_calls")
        .and_then(Value::as_array)
        .map(|calls| {
            calls
                .iter()
                .filter_map(|call| call.get("function_id").and_then(Value::as_str))
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    Ok(json!({
        "family_id": spec.family_id,
        "case_id": case_id,
        "source_fixture_path": source_relative_path,
        "formula": case.get("formula").and_then(Value::as_str),
        "action": case.get("action").and_then(Value::as_str),
        "expected": {
            "phase": expected.get("phase").and_then(Value::as_str),
            "decision": expected_decision,
            "reject_code": expected_reject_code,
            "payload_summary": expected_payload,
            "result_class": expected.get("result_class").and_then(Value::as_str),
            "result_structure_class": expected.get("result_structure_class").and_then(Value::as_str),
            "blankness_class": expected.get("blankness_class").and_then(Value::as_str),
            "capability_dependencies": expected.get("capability_dependencies").cloned().unwrap_or_else(|| json!([])),
            "trace_event_kinds": expected.get("trace_event_kinds").cloned().unwrap_or_else(|| json!([])),
            "prepared_function_ids": prepared_function_ids,
            "callable_carrier": expected.get("callable_carrier").cloned(),
            "callable_profile": expected.get("callable_profile").and_then(Value::as_str),
            "array_numbers": expected.get("array_numbers").cloned(),
            "array_logicals": expected.get("array_logicals").cloned(),
            "evaluation_error_contains": expected.get("evaluation_error_contains").and_then(Value::as_str),
            "dependency_consequence_evidence_classes": expected
                .get("dependency_consequence_evidence_classes")
                .cloned()
                .unwrap_or_else(|| json!([])),
            "overlay_families": expected.get("overlay_families").cloned().unwrap_or_else(|| json!([])),
            "dynamic_reference_failure_classes": expected
                .get("dynamic_reference_failure_classes")
                .cloned()
                .unwrap_or_else(|| json!([])),
        },
    }))
}

fn build_comparison_rows(
    repo_root: &Path,
    projections: &[Value],
) -> Result<Vec<Value>, OxFmlFixtureBridgeError> {
    projections
        .iter()
        .map(|projection| comparison_row(repo_root, projection))
        .collect()
}

fn comparison_row(repo_root: &Path, projection: &Value) -> Result<Value, OxFmlFixtureBridgeError> {
    let family_id = projection["family_id"]
        .as_str()
        .unwrap_or("<missing-family>");
    let case_id = projection["case_id"].as_str().unwrap_or("<missing-case>");
    let Some(mapping) = mapped_current_evidence(case_id) else {
        return Ok(json!({
            "family_id": family_id,
            "fixture_case_id": case_id,
            "comparison_state": "deferred_no_current_counterpart",
            "handoff_trigger": false,
            "rationale": if family_id == "prepared_call" || family_id == "higher_order_callable" {
                "direct OxCalc prepared-call/callable-carrier witnesses are successor-scoped"
            } else {
                "no current TraceCalc/TreeCalc scenario is a specific counterpart for this fixture case"
            },
            "current_outcomes": [],
        }));
    };

    let mut current_outcomes = Vec::new();
    for scenario_id in mapping.tracecalc_scenarios {
        current_outcomes.push(tracecalc_outcome(repo_root, scenario_id)?);
    }
    for (case_id, expected_state) in mapping.treecalc_cases {
        current_outcomes.push(treecalc_outcome(repo_root, case_id, expected_state)?);
    }

    let has_mismatch = current_outcomes.iter().any(|outcome| {
        outcome["comparison_state"]
            .as_str()
            .is_some_and(|state| state == "mismatch")
    });
    let has_missing = current_outcomes.iter().any(|outcome| {
        outcome["comparison_state"]
            .as_str()
            .is_some_and(|state| state == "missing_current_evidence")
    });
    let comparison_state = if has_mismatch {
        "mismatch"
    } else if has_missing {
        "missing_current_evidence"
    } else {
        "matched_current_evidence"
    };

    Ok(json!({
        "family_id": family_id,
        "fixture_case_id": case_id,
        "comparison_state": comparison_state,
        "handoff_trigger": has_mismatch,
        "observable_class": mapping.observable_class,
        "rationale": mapping.rationale,
        "current_outcomes": current_outcomes,
    }))
}

#[derive(Debug, Clone, Copy)]
struct CurrentEvidenceMapping {
    observable_class: &'static str,
    rationale: &'static str,
    tracecalc_scenarios: &'static [&'static str],
    treecalc_cases: &'static [(&'static str, &'static str)],
}

fn mapped_current_evidence(case_id: &str) -> Option<CurrentEvidenceMapping> {
    match case_id {
        "fec_001_accept" => Some(CurrentEvidenceMapping {
            observable_class: "accepted_publication",
            rationale: "FEC accepted candidate/commit aligns with current TraceCalc publish and TreeCalc local publication witnesses",
            tracecalc_scenarios: &["tc_accept_publish_001"],
            treecalc_cases: &[("tc_local_publish_001", "published")],
        }),
        "fec_002_formula_token_reject" => Some(CurrentEvidenceMapping {
            observable_class: "formula_token_reject_no_publish",
            rationale: "OxFml formula-token fence reject aligns with current TraceCalc artifact-token reject; TreeCalc has no token-fence counterpart yet",
            tracecalc_scenarios: &["tc_artifact_token_reject_001"],
            treecalc_cases: &[],
        }),
        "fec_003_capability_view_reject" => Some(CurrentEvidenceMapping {
            observable_class: "capability_view_reject_no_publish",
            rationale: "OxFml capability-view fence reject aligns with TraceCalc reject/no-publish and TreeCalc capability-sensitive reject witnesses",
            tracecalc_scenarios: &["tc_reject_no_publish_001"],
            treecalc_cases: &[("tc_local_capability_sensitive_reject_001", "rejected")],
        }),
        "session_001_commit" => Some(CurrentEvidenceMapping {
            observable_class: "session_commit_accepted_publication",
            rationale: "OxFml committed session path aligns with current accepted-publication witnesses",
            tracecalc_scenarios: &["tc_accept_publish_001"],
            treecalc_cases: &[("tc_local_publish_001", "published")],
        }),
        "session_002_capability_denied" => Some(CurrentEvidenceMapping {
            observable_class: "session_capability_denied_no_publish",
            rationale: "OxFml capability denial aligns with current reject/no-publish and capability-sensitive local rejection witnesses",
            tracecalc_scenarios: &["tc_reject_no_publish_001"],
            treecalc_cases: &[("tc_local_capability_sensitive_reject_001", "rejected")],
        }),
        "session_007_commit_stale_fence" => Some(CurrentEvidenceMapping {
            observable_class: "session_stale_fence_reject_no_publish",
            rationale: "OxFml stale-fence commit rejection aligns with the current TraceCalc publication-fence reject witness",
            tracecalc_scenarios: &["tc_publication_fence_reject_001"],
            treecalc_cases: &[],
        }),
        _ => None,
    }
}

fn tracecalc_outcome(
    repo_root: &Path,
    scenario_id: &str,
) -> Result<Value, OxFmlFixtureBridgeError> {
    let relative_path = relative_artifact_path([
        "docs",
        "test-runs",
        "core-engine",
        "tracecalc-reference-machine",
        TRACECALC_W033_RUN_ID,
        "scenarios",
        scenario_id,
        "result.json",
    ]);
    let path = repo_root.join(&relative_path);
    if !path.exists() {
        return Ok(json!({
            "engine": "tracecalc",
            "current_run_id": TRACECALC_W033_RUN_ID,
            "current_case_id": scenario_id,
            "artifact_path": relative_path,
            "expected_result_state": "passed",
            "observed_result_state": null,
            "comparison_state": "missing_current_evidence",
        }));
    }
    let result = read_json(&path)?;
    let observed = result.get("result_state").and_then(Value::as_str);
    let matched = observed == Some("passed")
        && empty_array(result.get("assertion_failures"))
        && empty_array(result.get("validation_failures"))
        && empty_array(result.get("conformance_mismatches"));
    Ok(json!({
        "engine": "tracecalc",
        "current_run_id": TRACECALC_W033_RUN_ID,
        "current_case_id": scenario_id,
        "artifact_path": relative_path,
        "expected_result_state": "passed",
        "observed_result_state": observed,
        "comparison_state": if matched { "matched" } else { "mismatch" },
    }))
}

fn treecalc_outcome(
    repo_root: &Path,
    case_id: &str,
    expected_state: &str,
) -> Result<Value, OxFmlFixtureBridgeError> {
    let relative_path = relative_artifact_path([
        "docs",
        "test-runs",
        "core-engine",
        "treecalc-local",
        TREECALC_W033_RUN_ID,
        "cases",
        case_id,
        "result.json",
    ]);
    let path = repo_root.join(&relative_path);
    if !path.exists() {
        return Ok(json!({
            "engine": "treecalc",
            "current_run_id": TREECALC_W033_RUN_ID,
            "current_case_id": case_id,
            "artifact_path": relative_path,
            "expected_result_state": expected_state,
            "observed_result_state": null,
            "comparison_state": "missing_current_evidence",
        }));
    }
    let result = read_json(&path)?;
    let observed = result.get("result_state").and_then(Value::as_str);
    Ok(json!({
        "engine": "treecalc",
        "current_run_id": TREECALC_W033_RUN_ID,
        "current_case_id": case_id,
        "artifact_path": relative_path,
        "expected_result_state": expected_state,
        "observed_result_state": observed,
        "comparison_state": if observed == Some(expected_state) { "matched" } else { "mismatch" },
    }))
}

fn comparison_counts(rows: &[Value]) -> ComparisonCounts {
    let mut counts = ComparisonCounts {
        total: rows.len(),
        ..ComparisonCounts::default()
    };
    for row in rows {
        match row["comparison_state"].as_str() {
            Some("matched_current_evidence") => counts.matched += 1,
            Some("deferred_no_current_counterpart") => counts.deferred += 1,
            Some("missing_current_evidence") => counts.missing_evidence += 1,
            Some("mismatch") => counts.mismatched += 1,
            _ => {}
        }
    }
    counts
}

fn handoff_watch_json(run_id: &str, relative_artifact_root: &str, mismatch_count: usize) -> Value {
    json!({
        "schema_version": BRIDGE_HANDOFF_WATCH_SCHEMA_V1,
        "run_id": run_id,
        "decision": if mismatch_count == 0 { "no_new_handoff_required" } else { "handoff_review_required" },
        "handoff_triggered": mismatch_count > 0,
        "concrete_upstream_mismatch_count": mismatch_count,
        "comparison_summary_path": format!("{relative_artifact_root}/comparisons/comparison_summary.json"),
        "handoff_register_update_required": mismatch_count > 0,
        "watch_rows": [
            {
                "watch_id": "W033-HW-001",
                "surface": "FEC candidate/commit/reject/fence facts",
                "classification_after_bridge": if mismatch_count == 0 { "no_new_handoff_required" } else { "handoff_review_required" }
            },
            {
                "watch_id": "W033-HW-009",
                "surface": "Direct OxFml fixture replay inside OxCalc",
                "classification_after_bridge": if mismatch_count == 0 { "projection_bridge_present_no_upstream_mismatch" } else { "mismatch_requires_triage" }
            },
            {
                "watch_id": "W033-HW-002",
                "surface": "LET/LAMBDA minimum callable carrier",
                "classification_after_bridge": "successor_scoped_to_calc-688"
            }
        ],
        "deferred_successor_beads": [
            "calc-688",
            "calc-y0r",
            "calc-lwh",
            "calc-rcr",
            "calc-8lg"
        ]
    })
}

fn required_artifacts(run_id: &str) -> Vec<String> {
    [
        "run_summary.json",
        "fixture_index.json",
        "family-projections/fec_commit.json",
        "family-projections/session_lifecycle.json",
        "family-projections/prepared_call.json",
        "family-projections/higher_order_callable.json",
        "comparisons/comparison_summary.json",
        "handoff/handoff_watch.json",
        "replay-appliance/bundle_manifest.json",
        "replay-appliance/validation/bundle_validation.json",
    ]
    .iter()
    .map(|artifact| {
        relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "oxfml-fixture-bridge",
            run_id,
            artifact,
        ])
    })
    .collect()
}

fn create_directory(path: &Path) -> Result<(), OxFmlFixtureBridgeError> {
    fs::create_dir_all(path).map_err(|source| OxFmlFixtureBridgeError::CreateDirectory {
        path: path.display().to_string(),
        source,
    })
}

fn write_json(path: &Path, value: &Value) -> Result<(), OxFmlFixtureBridgeError> {
    let content = serde_json::to_string_pretty(value).expect("JSON serialization should succeed");
    fs::write(path, format!("{content}\n")).map_err(|source| OxFmlFixtureBridgeError::WriteFile {
        path: path.display().to_string(),
        source,
    })
}

fn read_json(path: &Path) -> Result<Value, OxFmlFixtureBridgeError> {
    let content =
        fs::read_to_string(path).map_err(|source| OxFmlFixtureBridgeError::ReadEvidence {
            path: path.display().to_string(),
            source,
        })?;
    serde_json::from_str(&content).map_err(|source| OxFmlFixtureBridgeError::ParseJson {
        path: path.display().to_string(),
        source,
    })
}

fn empty_array(value: Option<&Value>) -> bool {
    value
        .and_then(Value::as_array)
        .is_none_or(std::vec::Vec::is_empty)
}

fn relative_artifact_path<'a>(segments: impl IntoIterator<Item = &'a str>) -> String {
    let parts = segments.into_iter().collect::<Vec<_>>();
    historical_w038_w045_artifact_path(&parts).unwrap_or_else(|| parts.join("/"))
}

fn historical_w038_w045_artifact_path(parts: &[&str]) -> Option<String> {
    if parts.len() >= 5
        && parts[0] == "docs"
        && parts[1] == "test-runs"
        && parts[2] == "core-engine"
        && matches!(
            parts[4].get(..4),
            Some("w038" | "w039" | "w040" | "w041" | "w042" | "w043" | "w044" | "w045")
        )
    {
        let mut archived = vec!["archive", "test-runs-core-engine-w038-w045"];
        archived.extend_from_slice(&parts[3..]);
        Some(archived.join("/"))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    #[test]
    fn bridge_projects_fixtures_and_compares_current_evidence() {
        let repo_root = unique_temp_repo();
        create_test_fixture_root(&repo_root);
        create_test_current_evidence(&repo_root);

        let summary = OxFmlFixtureBridgeRunner::new()
            .execute(&repo_root, "bridge-test")
            .expect("bridge run should write artifacts");

        assert_eq!(summary.family_count, 4);
        assert_eq!(summary.fixture_case_count, 4);
        assert_eq!(summary.matched_comparison_count, 2);
        assert_eq!(summary.deferred_comparison_count, 2);
        assert_eq!(summary.mismatch_count, 0);
        assert!(!summary.handoff_triggered);

        let run_summary =
            read_json(&repo_root.join(
                "docs/test-runs/core-engine/oxfml-fixture-bridge/bridge-test/run_summary.json",
            ))
            .unwrap();
        assert_eq!(run_summary["schema_version"], BRIDGE_RUN_SUMMARY_SCHEMA_V1);
        assert_eq!(run_summary["handoff_triggered"], false);

        let validation = read_json(
            &repo_root.join(
                "docs/test-runs/core-engine/oxfml-fixture-bridge/bridge-test/replay-appliance/validation/bundle_validation.json",
            ),
        )
        .unwrap();
        assert_eq!(validation["status"], "bundle_valid");

        fs::remove_dir_all(repo_root.parent().unwrap()).unwrap();
    }

    fn unique_temp_repo() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let base = std::env::temp_dir().join(format!(
            "oxcalc-oxfml-bridge-test-{}-{nanos}",
            std::process::id()
        ));
        let repo_root = base.join("OxCalc");
        fs::create_dir_all(&repo_root).unwrap();
        repo_root
    }

    fn create_test_fixture_root(repo_root: &Path) {
        let source_root = repo_root.join("../OxFml/crates/oxfml_core/tests/fixtures");
        fs::create_dir_all(&source_root).unwrap();
        fs::write(
            source_root.join("fec_commit_replay_cases.json"),
            r#"[
  {
    "case_id": "fec_001_accept",
    "expected": {
      "decision": "accepted",
      "published_payload": "Number(42)"
    }
  }
]
"#,
        )
        .unwrap();
        fs::write(
            source_root.join("session_lifecycle_replay_cases.json"),
            r#"[
  {
    "case_id": "session_001_commit",
    "formula": "=SUM(InputValue,2)",
    "action": "execute_commit",
    "expected": {
      "phase": "Committed",
      "decision": "accepted",
      "published_payload": "Number(7)"
    }
  }
]
"#,
        )
        .unwrap();
        fs::write(
            source_root.join("prepared_call_replay_cases.json"),
            r#"[
  {
    "case_id": "prepared_001_sum_name",
    "formula": "=SUM(InputValue,2)",
    "expected": {
      "prepared_calls": [
        {
          "function_id": "FUNC.SUM",
          "prepared_arguments": []
        }
      ],
      "payload_summary": "Number(7)"
    }
  }
]
"#,
        )
        .unwrap();
        fs::write(
            source_root.join("higher_order_callable_cases.json"),
            r#"[
  {
    "case_id": "higher_order_001_map_inline_lambda",
    "formula": "=MAP(SEQUENCE(3),LAMBDA(x,x+1))",
    "expected": {
      "payload_summary": "Array(3x1)",
      "array_numbers": [2.0, 3.0, 4.0]
    }
  }
]
"#,
        )
        .unwrap();
    }

    fn create_test_current_evidence(repo_root: &Path) {
        let trace_result_dir = repo_root.join(
            "docs/test-runs/core-engine/tracecalc-reference-machine/w033-tracecalc-oracle-self-check-001/scenarios/tc_accept_publish_001",
        );
        fs::create_dir_all(&trace_result_dir).unwrap();
        fs::write(
            trace_result_dir.join("result.json"),
            r#"{
  "result_state": "passed",
  "assertion_failures": [],
  "validation_failures": [],
  "conformance_mismatches": []
}
"#,
        )
        .unwrap();

        let tree_result_dir = repo_root.join(
            "docs/test-runs/core-engine/treecalc-local/w033-treecalc-witness-bridge-001/cases/tc_local_publish_001",
        );
        fs::create_dir_all(&tree_result_dir).unwrap();
        fs::write(
            tree_result_dir.join("result.json"),
            r#"{
  "case_id": "tc_local_publish_001",
  "result_state": "published"
}
"#,
        )
        .unwrap();
    }
}
