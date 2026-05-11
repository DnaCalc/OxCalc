#!/usr/bin/env python3
"""Build the W048 cross-corpus conformance summary from checked artifacts."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "docs/test-runs/core-engine/w048-conformance-001"

EXCEL = ROOT / "docs/test-runs/excel-cycles/w048-excel-cycles-001/observation.json"
TRACE_ROOT = ROOT / "docs/test-runs/core-engine/tracecalc-reference-machine/w048-tracecalc-cycles-003"
TREE_ROOT = ROOT / "docs/test-runs/core-engine/treecalc-local/w048-treecalc-cycles-001"
GRAPH = TREE_ROOT / "w048_materialized_graph_check_summary.json"

TRACE_FIXTURES = [
    "tc_w048_structural_self_cycle_reject_001",
    "tc_w048_ctro_candidate_cycle_reject_001",
    "tc_w048_ctro_release_reentry_downstream_001",
]

TREE_CASES = {
    "tc_w048_structural_self_cycle_reject_001": {
        "initial_state": "rejected",
        "reject_kind": "SyntheticCycleReject",
    },
    "tc_w048_structural_two_node_cycle_reject_001": {
        "initial_state": "rejected",
        "reject_kind": "SyntheticCycleReject",
    },
    "tc_w048_ctro_dynamic_self_cycle_reject_001": {
        "initial_state": "rejected",
        "reject_kind": "SyntheticCycleReject",
    },
    "tc_w048_ctro_dynamic_release_reentry_downstream_001": {
        "initial_state": "rejected",
        "reject_kind": "SyntheticCycleReject",
        "post_edit_state": "published",
        "post_edit_values": {"2": "10", "4": "11"},
    },
}

REQUIRED_CASES = [
    ("structural direct self-cycle", "covered", ["tracecalc", "treecalc", "graph"]),
    ("structural two-node SCC", "covered", ["treecalc", "graph"]),
    ("structural three-node SCC with deterministic member ordering", "covered_by_existing_cycle_region_checker", ["graph checker SCC ordering over full TreeCalc run"]),
    ("guarded activation cycle with prior-value retention question", "observed_or_deferred", ["Excel observation ledger", "iterative-profile bead calc-zci1.4"]),
    ("CTRO dynamic self-cycle", "covered", ["treecalc", "graph"]),
    ("CTRO dynamic two-node SCC", "deferred", ["no dedicated checked fixture yet"]),
    ("CTRO candidate cycle rejected with no overlay commit", "covered", ["tracecalc", "treecalc no publication bundle"]),
    ("CTRO cycle release and re-entry", "covered", ["tracecalc", "treecalc post_edit"]),
    ("downstream dependent blocked by cycle and recomputed after release", "covered", ["tracecalc", "treecalc post_edit downstream value"]),
    ("iterative self-cycle after profile selection", "deferred", ["calc-zci1.4"]),
    ("order-sensitive iterative SCC after profile selection", "deferred", ["calc-zci1.4"]),
    ("graph materialization reverse-edge converse case", "covered", ["check-w048-materialized-graphs.py"]),
    ("candidate-effective graph cycle introduction case", "covered", ["TreeCalc W048 graph sidecars"]),
    ("innovation profile example when admitted", "deferred", ["calc-zci1.8"]),
]


def load_json(path: Path, *, encoding: str = "utf-8"):
    with path.open("r", encoding=encoding) as f:
        return json.load(f)


def main() -> int:
    errors: list[str] = []
    excel = load_json(EXCEL, encoding="utf-8-sig")
    if excel.get("observation_count", 0) < 12:
        errors.append(f"Excel observation_count too low: {excel.get('observation_count')}")

    trace_summary = load_json(TRACE_ROOT / "run_summary.json")
    if trace_summary.get("scenario_count") != 34:
        errors.append(f"TraceCalc scenario_count expected 34 observed {trace_summary.get('scenario_count')}")
    if trace_summary.get("result_counts", {}).get("passed") != trace_summary.get("scenario_count"):
        errors.append("TraceCalc run did not pass all scenarios")
    for fixture in TRACE_FIXTURES:
        if not (ROOT / f"docs/test-corpus/core-engine/tracecalc/hand-auditable/{fixture}.json").exists():
            errors.append(f"missing TraceCalc fixture {fixture}")

    tree_summary = load_json(TREE_ROOT / "run_summary.json")
    if tree_summary.get("case_count") != 33:
        errors.append(f"TreeCalc case_count expected 33 observed {tree_summary.get('case_count')}")
    if tree_summary.get("expectation_mismatch_count") != 0:
        errors.append("TreeCalc expectation mismatches are nonzero")

    graph_summary = load_json(GRAPH)
    if graph_summary.get("check_error_count") != 0:
        errors.append("W048 graph checker errors are nonzero")
    if graph_summary.get("case_count") != 33 or graph_summary.get("layer_count") != 99:
        errors.append("W048 graph checker did not cover the expected 33 cases / 99 layers")
    if graph_summary.get("cycle_region_count", 0) < 12:
        errors.append("W048 graph checker cycle_region_count below expected floor 12")

    case_index = {case["case_id"]: case for case in load_json(TREE_ROOT / "case_index.json")}
    observed_tree_cases = {}
    for case_id, expectation in TREE_CASES.items():
        indexed = case_index.get(case_id)
        if not indexed:
            errors.append(f"missing TreeCalc case index entry {case_id}")
            continue
        if indexed.get("result_state") != expectation["initial_state"]:
            errors.append(f"{case_id} initial state mismatch: {indexed.get('result_state')}")
        if indexed.get("expectation_mismatches"):
            errors.append(f"{case_id} has expectation mismatches")
        result = load_json(TREE_ROOT / "cases" / case_id / "result.json")
        if result.get("reject_detail") and result["reject_detail"].get("kind") != expectation["reject_kind"]:
            errors.append(f"{case_id} reject kind mismatch")
        case_record = {
            "initial_state": indexed.get("result_state"),
            "expectation_mismatches": indexed.get("expectation_mismatches"),
            "reject_kind": result.get("reject_detail", {}).get("kind"),
        }
        if "post_edit_state" in expectation:
            post = load_json(TREE_ROOT / "cases" / case_id / "post_edit/result.json")
            if post.get("result_state") != expectation["post_edit_state"]:
                errors.append(f"{case_id} post-edit state mismatch: {post.get('result_state')}")
            for node_id, expected_value in expectation["post_edit_values"].items():
                if post.get("published_values", {}).get(node_id) != expected_value:
                    errors.append(f"{case_id} post-edit value {node_id} mismatch")
            case_record["post_edit_state"] = post.get("result_state")
            case_record["post_edit_published_values"] = {
                k: post.get("published_values", {}).get(k) for k in expectation["post_edit_values"]
            }
        observed_tree_cases[case_id] = case_record

    summary = {
        "schema_version": "oxcalc.w048.conformance_summary.v1",
        "run_id": "w048-conformance-001",
        "status": "passed" if not errors else "failed",
        "errors": errors,
        "sources": {
            "excel_observation": str(EXCEL.relative_to(ROOT)).replace("\\", "/"),
            "tracecalc_run": str(TRACE_ROOT.relative_to(ROOT)).replace("\\", "/"),
            "treecalc_run": str(TREE_ROOT.relative_to(ROOT)).replace("\\", "/"),
            "graph_checker_summary": str(GRAPH.relative_to(ROOT)).replace("\\", "/"),
        },
        "source_summaries": {
            "excel_observation_count": excel.get("observation_count"),
            "tracecalc_scenario_count": trace_summary.get("scenario_count"),
            "tracecalc_result_counts": trace_summary.get("result_counts"),
            "treecalc_case_count": tree_summary.get("case_count"),
            "treecalc_expectation_mismatch_count": tree_summary.get("expectation_mismatch_count"),
            "treecalc_result_counts": tree_summary.get("result_counts"),
            "graph_case_count": graph_summary.get("case_count"),
            "graph_layer_count": graph_summary.get("layer_count"),
            "graph_cycle_region_count": graph_summary.get("cycle_region_count"),
            "graph_check_error_count": graph_summary.get("check_error_count"),
        },
        "tracecalc_fixture_ids": TRACE_FIXTURES,
        "treecalc_case_results": observed_tree_cases,
        "required_case_matrix": [
            {"requirement": req, "status": status, "evidence": evidence}
            for req, status, evidence in REQUIRED_CASES
        ],
        "explicit_deferred_lanes": [
            "CTRO dynamic two-node SCC dedicated fixture",
            "iterative self-cycle and order-sensitive SCC after calc-zci1.4 profile selection",
            "innovation profile example after calc-zci1.8 admission",
            "formal proof/model artifact expansion under calc-zci1.5",
        ],
    }

    OUT.mkdir(parents=True, exist_ok=True)
    (OUT / "w048_conformance_summary.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
    print(f"w048 conformance {summary['status']}: {OUT / 'w048_conformance_summary.json'}")
    if errors:
        for error in errors:
            print(f"ERROR: {error}")
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
