#!/usr/bin/env python3
"""Build and check W048 materialized graph sidecars from TreeCalc run artifacts.

The checker is intentionally independent of the Rust runner output shape.  It
consumes per-case dependency_graph.json and result.json artifacts, materializes
structural / published-effective / candidate-effective graph layers, emits
reverse-edge facts, cycle-region records, stable hashes, and a run summary, then
fails if required graph invariants are violated.
"""

from __future__ import annotations

import argparse
import hashlib
import json
from collections import defaultdict, deque
from pathlib import Path
from typing import Any

SCHEMA = "oxcalc.w048.materialized_graph_layers.v1"
SUMMARY_SCHEMA = "oxcalc.w048.materialized_graph_check_summary.v1"


def load_json(path: Path) -> Any:
    with path.open("r", encoding="utf-8-sig") as handle:
        return json.load(handle)


def write_json(path: Path, value: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8") as handle:
        json.dump(value, handle, indent=2, sort_keys=True)
        handle.write("\n")


def stable_hash(value: Any) -> str:
    payload = json.dumps(value, sort_keys=True, separators=(",", ":"))
    return "sha256:" + hashlib.sha256(payload.encode("utf-8")).hexdigest()


def edge_origin(kind: str, layer: str) -> str:
    if kind == "DynamicPotential":
        return "candidate_dynamic_activation" if layer == "candidate_effective" else "published_dynamic_activation"
    if kind == "RelativeBound":
        return "structural_relative_bound"
    if kind == "HostSensitive":
        return "host_declared"
    if kind == "CapabilitySensitive":
        return "host_declared"
    if kind == "ShapeTopology":
        return "host_declared"
    return "structural_static"


def stability_class(layer: str, kind: str) -> str:
    if layer == "candidate_effective" and kind == "DynamicPotential":
        return "candidate_overlay"
    if layer == "published_effective" and kind == "DynamicPotential":
        return "published_overlay"
    return "structural"


def normalize_edges(edges: list[dict[str, Any]], layer: str) -> list[dict[str, Any]]:
    out = []
    for edge in edges:
        kind = str(edge.get("kind", "unknown"))
        owner = edge.get("owner_node_id")
        target = edge.get("target_node_id")
        out.append(
            {
                "edge_id": str(edge.get("edge_id")),
                "owner_node_id": owner,
                "target_node_id": target,
                "descriptor_id": str(edge.get("descriptor_id")),
                "kind": kind,
                "edge_origin": edge_origin(kind, layer),
                "overlay_epoch": None,
                "wave_id": "treecalc-local-run" if layer == "candidate_effective" else None,
                "dynamic_carrier_detail": None,
                "stability_class": stability_class(layer, kind),
                "value_read_eligible": True,
            }
        )
    return sorted(out, key=lambda item: item["edge_id"])


def reverse_edges(forward_edges: list[dict[str, Any]]) -> list[dict[str, Any]]:
    out = []
    for edge in forward_edges:
        out.append(
            {
                "target_node_id": edge["target_node_id"],
                "owner_node_id": edge["owner_node_id"],
                "edge_id": edge["edge_id"],
                "descriptor_id": edge["descriptor_id"],
                "kind": edge["kind"],
            }
        )
    return sorted(out, key=lambda item: (item["target_node_id"], item["owner_node_id"], item["edge_id"]))


def validate_reverse_converse(forward_edges: list[dict[str, Any]], reverse: list[dict[str, Any]]) -> list[str]:
    forward_set = {
        (edge["target_node_id"], edge["owner_node_id"], edge["edge_id"], edge["descriptor_id"])
        for edge in forward_edges
    }
    reverse_set = {
        (edge["target_node_id"], edge["owner_node_id"], edge["edge_id"], edge["descriptor_id"])
        for edge in reverse
    }
    errors = []
    for missing in sorted(forward_set - reverse_set):
        errors.append(f"missing_reverse_edge:{missing}")
    for extra in sorted(reverse_set - forward_set):
        errors.append(f"extra_reverse_edge:{extra}")
    return errors


def topo_from_result(result: dict[str, Any], cycle_groups: list[list[int]]) -> list[int]:
    if cycle_groups:
        return []
    order = result.get("evaluation_order", [])
    if isinstance(order, list):
        return [int(item) for item in order]
    return []


def boundary_edges(members: set[int], forward_edges: list[dict[str, Any]]) -> tuple[list[str], list[str], list[str]]:
    internal = []
    incoming = []
    outgoing = []
    for edge in forward_edges:
        owner = int(edge["owner_node_id"])
        target = int(edge["target_node_id"])
        if owner in members and target in members:
            internal.append(edge["edge_id"])
        elif owner not in members and target in members:
            incoming.append(edge["edge_id"])
        elif owner in members and target not in members:
            outgoing.append(edge["edge_id"])
    return sorted(internal), sorted(incoming), sorted(outgoing)


def cycle_regions(case_id: str, graph_id: str, cycle_groups: list[list[int]], forward_edges: list[dict[str, Any]], layer: str) -> list[dict[str, Any]]:
    regions = []
    source = "candidate_overlay" if layer == "candidate_effective" else "structural"
    for index, group in enumerate(cycle_groups, start=1):
        members = sorted(int(item) for item in group)
        internal, incoming, outgoing = boundary_edges(set(members), forward_edges)
        regions.append(
            {
                "cycle_region_id": f"cycle:{case_id}:{layer}:{index}",
                "graph_id": graph_id,
                "cycle_source": source,
                "members": members,
                "member_order": members,
                "cycle_root": members[0] if members else None,
                "root_policy": "canonical_node_id_root",
                "internal_edges": internal,
                "incoming_boundary_edges": incoming,
                "outgoing_boundary_edges": outgoing,
                "introduced_by_overlay_delta_ids": [],
                "released_from_cycle_region_id": None,
                "terminal_policy": "stage1_non_iterative_reject",
                "terminal_state": "cycle_blocked" if members else None,
                "prior_value_basis": [],
                "iteration_summary": None,
            }
        )
    return regions


def nodes_from_graph(graph: dict[str, Any], result: dict[str, Any]) -> list[dict[str, Any]]:
    node_ids: set[int] = set()
    formula_ids: dict[int, str] = {}
    for descriptor in graph.get("descriptors", []):
        owner = descriptor.get("owner_node_id")
        target = descriptor.get("target_node_id")
        if owner is not None:
            node_ids.add(int(owner))
            formula_ids.setdefault(int(owner), str(descriptor.get("descriptor_id")))
        if target is not None:
            node_ids.add(int(target))
    for edge in graph.get("edges", []):
        node_ids.add(int(edge["owner_node_id"]))
        node_ids.add(int(edge["target_node_id"]))
    published = result.get("publication_bundle", {}) or {}
    published_values = (published.get("published_view_delta", {}) or {}) if isinstance(published, dict) else {}
    return [
        {
            "node_id": node_id,
            "stable_symbol": f"node:{node_id}",
            "node_kind": "formula_or_value",
            "formula_bind_artifact_id": formula_ids.get(node_id),
            "calc_state": result.get("result_state"),
            "published_value_hash": stable_hash(published_values.get(str(node_id))) if str(node_id) in published_values else None,
            "demanded": True,
        }
        for node_id in sorted(node_ids)
    ]


def overlay_deltas(result: dict[str, Any], layer: str) -> list[dict[str, Any]]:
    if layer != "candidate_effective":
        return []
    candidate = result.get("candidate_result") or {}
    updates = candidate.get("dependency_shape_updates") or []
    deltas = []
    for index, update in enumerate(updates, start=1):
        owner = update.get("owner_node_id") or update.get("node_id")
        deltas.append(
            {
                "overlay_delta_id": f"delta:treecalc-local:{index}",
                "owner_node_id": owner,
                "delta_kind": str(update.get("kind", "dependency_shape_update")),
                "previous_edge_id": None,
                "candidate_edge_id": None,
                "carrier": update,
                "provenance": "candidate_overlay",
                "wave_id": "treecalc-local-run",
            }
        )
    return deltas


def materialize_layer(case_id: str, graph: dict[str, Any], result: dict[str, Any], layer: str) -> dict[str, Any]:
    graph_id = f"{layer}:{case_id}"
    forward = normalize_edges(graph.get("edges", []), layer)
    reverse = reverse_edges(forward)
    cycle_groups = [[int(item) for item in group] for group in graph.get("cycle_groups", [])]
    layer_doc = {
        "schema_version": SCHEMA,
        "case_id": case_id,
        "graph_id": graph_id,
        "graph_layer": layer,
        "basis": {
            "snapshot_id": graph.get("snapshot_id"),
            "published_overlay_epoch": "treecalc-local-seeded" if layer in {"published_effective", "candidate_effective"} else None,
            "candidate_wave_id": "treecalc-local-run" if layer == "candidate_effective" else None,
            "profile_id": "stage1.non_iterative",
        },
        "nodes": nodes_from_graph(graph, result),
        "forward_edges": forward,
        "reverse_edges": reverse,
        "edge_provenance": [
            {
                "edge_id": edge["edge_id"],
                "edge_origin": edge["edge_origin"],
                "stability_class": edge["stability_class"],
            }
            for edge in forward
        ],
        "overlay_deltas": overlay_deltas(result, layer),
        "cycle_regions": cycle_regions(case_id, graph_id, cycle_groups, forward, layer),
        "topological_order": topo_from_result(result, cycle_groups),
        "blocked_reason": "cycle_detected" if cycle_groups else None,
    }
    layer_doc["graph_hash"] = stable_hash({key: value for key, value in layer_doc.items() if key != "graph_hash"})
    return layer_doc


def check_layer(layer: dict[str, Any]) -> list[str]:
    errors = validate_reverse_converse(layer["forward_edges"], layer["reverse_edges"])
    if not layer["graph_hash"].startswith("sha256:"):
        errors.append("graph_hash_missing_sha256")
    if layer["cycle_regions"] and layer["topological_order"]:
        errors.append("cyclic_layer_has_topological_order")
    if not layer["cycle_regions"] and layer["blocked_reason"] is not None:
        errors.append("acyclic_layer_has_blocked_reason")
    return errors


def process_case(case_dir: Path) -> dict[str, Any]:
    graph_path = case_dir / "dependency_graph.json"
    result_path = case_dir / "result.json"
    graph = load_json(graph_path)
    result = load_json(result_path)
    case_id = case_dir.name
    layers = [
        materialize_layer(case_id, graph, result, "structural"),
        materialize_layer(case_id, graph, result, "published_effective"),
        materialize_layer(case_id, graph, result, "candidate_effective"),
    ]
    errors = []
    for layer in layers:
        errors.extend(f"{layer['graph_layer']}:{error}" for error in check_layer(layer))
    out = {
        "schema_version": SCHEMA,
        "case_id": case_id,
        "source_artifacts": {
            "dependency_graph": str(graph_path.as_posix()),
            "result": str(result_path.as_posix()),
        },
        "layers": layers,
        "check_errors": errors,
    }
    write_json(case_dir / "w048_materialized_graph_layers.json", out)
    return {
        "case_id": case_id,
        "materialized_graph_layers_path": str((case_dir / "w048_materialized_graph_layers.json").as_posix()),
        "layer_count": len(layers),
        "cycle_region_count": sum(len(layer["cycle_regions"]) for layer in layers),
        "reverse_edge_count": sum(len(layer["reverse_edges"]) for layer in layers),
        "check_errors": errors,
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("run_root", type=Path)
    parser.add_argument("--summary", type=Path)
    args = parser.parse_args()

    run_root = args.run_root
    cases_root = run_root / "cases"
    if not cases_root.is_dir():
        raise SystemExit(f"missing cases directory: {cases_root}")

    case_summaries = []
    for case_dir in sorted(path for path in cases_root.iterdir() if path.is_dir()):
        if (case_dir / "dependency_graph.json").exists() and (case_dir / "result.json").exists():
            case_summaries.append(process_case(case_dir))

    errors = [error for case in case_summaries for error in case["check_errors"]]
    summary = {
        "schema_version": SUMMARY_SCHEMA,
        "run_root": str(run_root.as_posix()),
        "case_count": len(case_summaries),
        "layer_count": sum(case["layer_count"] for case in case_summaries),
        "cycle_region_count": sum(case["cycle_region_count"] for case in case_summaries),
        "reverse_edge_count": sum(case["reverse_edge_count"] for case in case_summaries),
        "case_summaries": case_summaries,
        "check_error_count": len(errors),
        "check_errors": errors,
    }
    summary_path = args.summary or (run_root / "w048_materialized_graph_check_summary.json")
    write_json(summary_path, summary)
    if errors:
        print(f"w048 materialized graph check failed: {len(errors)} errors; summary={summary_path}")
        return 1
    print(f"w048 materialized graph check ok: cases={len(case_summaries)} layers={summary['layer_count']} summary={summary_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
