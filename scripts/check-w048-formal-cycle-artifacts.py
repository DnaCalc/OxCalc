#!/usr/bin/env python3
"""Check W048 formal cycle obligations against concrete TreeCalc artifacts."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
TREE_ROOT = ROOT / "docs/test-runs/core-engine/treecalc-local/w048-treecalc-cycles-001"
GRAPH_SUMMARY = TREE_ROOT / "w048_materialized_graph_check_summary.json"
OUT = ROOT / "docs/test-runs/core-engine/formal/w048-cycle-artifacts-001"

W048_CASES = [
    "tc_w048_structural_self_cycle_reject_001",
    "tc_w048_structural_two_node_cycle_reject_001",
    "tc_w048_ctro_dynamic_self_cycle_reject_001",
    "tc_w048_ctro_dynamic_release_reentry_downstream_001",
]


def load(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def edge_pair(edge: dict) -> tuple[int, int]:
    owner = edge.get("owner_node_id") or edge.get("from") or edge.get("source_node_id")
    target = edge.get("target_node_id") or edge.get("to")
    return int(owner), int(target)


def layer_edges(layer: dict) -> tuple[set[tuple[int, int]], set[tuple[int, int]]]:
    forward = set()
    reverse = set()
    for edge in layer.get("forward_edges", []):
        forward.add(edge_pair(edge))
    for edge in layer.get("reverse_edges", []):
        target = edge.get("target_node_id") or edge.get("to") or edge.get("source_node_id")
        owner = edge.get("owner_node_id") or edge.get("from")
        reverse.add((int(target), int(owner)))
    return forward, reverse


def main() -> int:
    errors: list[str] = []
    graph_summary = load(GRAPH_SUMMARY)
    if graph_summary.get("case_count") != 33:
        errors.append("graph summary case_count must be 33")
    if graph_summary.get("layer_count") != 99:
        errors.append("graph summary layer_count must be 99")
    if graph_summary.get("cycle_region_count") != 12:
        errors.append("graph summary cycle_region_count must be 12")
    if graph_summary.get("check_error_count") != 0:
        errors.append("graph summary check_error_count must be 0")

    layer_checks = []
    cycle_region_records = 0
    for case_dir in sorted((TREE_ROOT / "cases").iterdir()):
        layer_file = case_dir / "w048_materialized_graph_layers.json"
        if not layer_file.exists():
            continue
        data = load(layer_file)
        for layer in data.get("layers", []):
            forward, reverse = layer_edges(layer)
            missing_reverse = sorted([edge for edge in forward if (edge[1], edge[0]) not in reverse])
            missing_forward = sorted([edge for edge in reverse if (edge[1], edge[0]) not in forward])
            regions = layer.get("cycle_regions", [])
            cycle_region_records += len(regions)
            layer_name = layer.get("graph_layer") or layer.get("layer_name")
            for region in regions:
                members = region.get("members") or region.get("member_node_ids", [])
                if not members:
                    errors.append(f"{case_dir.name}:{layer_name} has empty cycle region")
            if missing_reverse or missing_forward:
                errors.append(
                    f"{case_dir.name}:{layer_name} converse mismatch "
                    f"missing_reverse={missing_reverse} missing_forward={missing_forward}"
                )
            layer_checks.append(
                {
                    "case_id": case_dir.name,
                    "layer_name": layer_name,
                    "forward_edge_count": len(forward),
                    "reverse_edge_count": len(reverse),
                    "cycle_region_count": len(regions),
                    "converse_ok": not missing_reverse and not missing_forward,
                }
            )

    if cycle_region_records != graph_summary.get("cycle_region_count"):
        errors.append(
            f"cycle region record count mismatch layers={cycle_region_records} summary={graph_summary.get('cycle_region_count')}"
        )

    case_results = {}
    for case_id in W048_CASES:
        result = load(TREE_ROOT / "cases" / case_id / "result.json")
        if result.get("result_state") != "rejected":
            errors.append(f"{case_id} must initially reject")
        if result.get("publication_bundle") is not None:
            errors.append(f"{case_id} rejected initial run must not publish")
        reject_kind = result.get("reject_detail", {}).get("kind")
        if reject_kind != "SyntheticCycleReject":
            errors.append(f"{case_id} reject kind must be SyntheticCycleReject")
        case_results[case_id] = {
            "initial_state": result.get("result_state"),
            "reject_kind": reject_kind,
            "publication_bundle_present": result.get("publication_bundle") is not None,
            "published_values": result.get("published_values"),
        }

    release_case = "tc_w048_ctro_dynamic_release_reentry_downstream_001"
    post = load(TREE_ROOT / "cases" / release_case / "post_edit/result.json")
    if post.get("result_state") != "published":
        errors.append("release/re-entry post-edit result must publish")
    if post.get("published_values", {}).get("2") != "10" or post.get("published_values", {}).get("4") != "11":
        errors.append("release/re-entry owner/downstream values must be 10/11")
    case_results[release_case]["post_edit_state"] = post.get("result_state")
    case_results[release_case]["post_edit_published_values"] = {
        "2": post.get("published_values", {}).get("2"),
        "4": post.get("published_values", {}).get("4"),
    }

    summary = {
        "schema_version": "oxcalc.w048.formal_cycle_checker_summary.v1",
        "run_id": "w048-cycle-artifacts-001",
        "status": "passed" if not errors else "failed",
        "errors": errors,
        "definition_packet": "docs/spec/core-engine/w048-cycles/W048_FORMAL_CYCLE_DEFINITIONS_AND_CHECKER_ARTIFACTS.md",
        "tla_model": "formal/tla/CoreEngineW048CycleRegions.tla",
        "source_treecalc_run": str(TREE_ROOT.relative_to(ROOT)).replace("\\", "/"),
        "graph_summary": {
            "case_count": graph_summary.get("case_count"),
            "layer_count": graph_summary.get("layer_count"),
            "cycle_region_count": graph_summary.get("cycle_region_count"),
            "check_error_count": graph_summary.get("check_error_count"),
        },
        "checked_layer_count": len(layer_checks),
        "checked_cycle_region_count": cycle_region_records,
        "case_results": case_results,
        "obligations": {
            "forward_reverse_converse": all(item["converse_ok"] for item in layer_checks),
            "cycle_regions_nonempty": not any("empty cycle region" in error for error in errors),
            "cycle_reject_no_publication": all(not result["publication_bundle_present"] for result in case_results.values()),
            "release_reentry_publishes_owner_and_downstream": case_results[release_case].get("post_edit_published_values") == {"2": "10", "4": "11"},
        },
    }
    OUT.mkdir(parents=True, exist_ok=True)
    (OUT / "w048_formal_cycle_checker_summary.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
    print(f"w048 formal cycle checker {summary['status']}: {OUT / 'w048_formal_cycle_checker_summary.json'}")
    if errors:
        for error in errors:
            print(f"ERROR: {error}")
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
