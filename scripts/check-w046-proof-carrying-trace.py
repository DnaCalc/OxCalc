#!/usr/bin/env python3
"""Validate W046 proof-carrying trace projections over existing artifacts.

The checker is intentionally small and deterministic: it reads already-emitted
TreeCalc/TraceCalc JSON artifacts, derives local semantic facts, and reports
missing/mismatched proof facts without mutating the baseline roots.
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any


REPO_ROOT = Path(__file__).resolve().parents[1]


class CheckFailure(Exception):
    pass


def load_json(path: Path) -> Any:
    try:
        with path.open("r", encoding="utf-8") as fh:
            return json.load(fh)
    except FileNotFoundError as exc:
        raise CheckFailure(f"missing artifact: {path}") from exc
    except json.JSONDecodeError as exc:
        raise CheckFailure(f"invalid JSON artifact {path}: {exc}") from exc


def resolve_artifact(path_value: str, base: Path = REPO_ROOT) -> Path:
    path = Path(path_value)
    if path.is_absolute():
        return path
    return base / path


def as_node_set(values: list[Any]) -> set[int]:
    return {int(value) for value in values}


def sorted_int_strings(mapping: dict[str, Any]) -> list[int]:
    return sorted(int(key) for key in mapping.keys())


def validate_treecalc_result(result_path: Path) -> dict[str, Any]:
    result = load_json(result_path)
    case_id = result.get("case_id", result_path.parent.name)
    facts: list[str] = []
    failures: list[str] = []
    blockers: list[str] = []

    def fail(message: str) -> None:
        failures.append(message)

    graph_path_value = result.get("dependency_graph_path")
    closure_path_value = result.get("invalidation_closure_path")
    if not graph_path_value:
        fail("result is missing dependency_graph_path")
        graph = {"edges": [], "descriptors": [], "cycle_groups": [], "diagnostics": []}
    else:
        graph = load_json(resolve_artifact(graph_path_value))
        facts.append("graph_artifact_parseable")

    if not closure_path_value:
        fail("result is missing invalidation_closure_path")
        closure_records: list[dict[str, Any]] = []
    else:
        closure_records = load_json(resolve_artifact(closure_path_value))
        facts.append("invalidation_closure_artifact_parseable")

    edges = graph.get("edges", [])
    if not isinstance(edges, list):
        fail("dependency_graph.edges is not a list")
        edges = []
    descriptors = graph.get("descriptors", [])
    if not isinstance(descriptors, list):
        fail("dependency_graph.descriptors is not a list")
        descriptors = []

    edge_ids: set[str] = set()
    reverse_index: dict[int, set[int]] = {}
    for edge in edges:
        try:
            owner = int(edge["owner_node_id"])
            target = int(edge["target_node_id"])
            edge_id = str(edge["edge_id"])
        except (KeyError, TypeError, ValueError) as exc:
            fail(f"malformed dependency edge: {edge!r} ({exc})")
            continue
        if edge_id in edge_ids:
            fail(f"duplicate dependency edge id: {edge_id}")
        edge_ids.add(edge_id)
        reverse_index.setdefault(target, set()).add(owner)
    facts.append("forward_edges_well_formed")
    facts.append("derived_reverse_index_converse")

    descriptor_ids = {str(desc.get("descriptor_id")) for desc in descriptors}
    for edge in edges:
        descriptor_id = str(edge.get("descriptor_id"))
        if descriptor_id not in descriptor_ids:
            fail(f"edge {edge.get('edge_id')} references missing descriptor {descriptor_id}")
    facts.append("edge_descriptor_links_checked")

    evaluation_order = [int(node) for node in result.get("evaluation_order", [])]
    order_index = {node: index for index, node in enumerate(evaluation_order)}
    closure_nodes: set[int] = set()
    rebind_nodes: set[int] = set()
    for record in closure_records:
        try:
            node = int(record["node_id"])
        except (KeyError, TypeError, ValueError) as exc:
            fail(f"malformed invalidation closure record: {record!r} ({exc})")
            continue
        closure_nodes.add(node)
        if bool(record.get("requires_rebind", False)):
            rebind_nodes.add(node)
    if set(evaluation_order).issubset(closure_nodes) or result.get("result_state") == "rejected":
        facts.append("evaluation_order_covered_by_invalidation_closure_or_reject")
    else:
        fail(
            "evaluation_order contains nodes not present in invalidation closure: "
            f"{sorted(set(evaluation_order) - closure_nodes)}"
        )

    for edge in edges:
        try:
            owner = int(edge["owner_node_id"])
            target = int(edge["target_node_id"])
        except (KeyError, TypeError, ValueError):
            continue
        if owner in order_index and target in order_index and not order_index[target] < order_index[owner]:
            fail(f"dependency target {target} is not before owner {owner} in evaluation_order")
    facts.append("formula_edges_respect_evaluation_order")

    for edge in edges:
        try:
            owner = int(edge["owner_node_id"])
            target = int(edge["target_node_id"])
        except (KeyError, TypeError, ValueError):
            continue
        if owner in order_index and target not in order_index:
            facts.append("stable_input_read_observed")
        elif owner in order_index and target in order_index and order_index[target] < order_index[owner]:
            facts.append("prior_formula_read_observed")

    result_state = result.get("result_state")
    candidate = result.get("candidate_result")
    publication = result.get("publication_bundle")
    reject = result.get("reject_detail")
    if result_state == "published":
        if not candidate:
            fail("published result is missing candidate_result")
        if not publication:
            fail("published result is missing publication_bundle")
        if reject is not None:
            fail("published result carries reject_detail")
        if candidate and publication:
            candidate_targets = as_node_set(candidate.get("target_set", []))
            if candidate_targets != set(evaluation_order):
                fail(
                    "candidate target_set does not match evaluation_order: "
                    f"targets={sorted(candidate_targets)} order={evaluation_order}"
                )
            candidate_values = sorted_int_strings(candidate.get("value_updates", {}))
            published_values = sorted_int_strings(publication.get("published_view_delta", {}))
            if candidate_values != published_values:
                fail(
                    "candidate value_updates keys do not match publication published_view_delta keys: "
                    f"candidate={candidate_values} published={published_values}"
                )
            facts.append("candidate_publication_bundle_consistent")
    elif result_state == "rejected":
        if publication is not None:
            fail("rejected result carries publication_bundle")
        if reject is None:
            fail("rejected result is missing reject_detail")
        facts.append("reject_has_no_publication_bundle")
    else:
        blockers.append(f"unclassified_treecalc_result_state:{result_state}")

    if rebind_nodes and result_state != "rejected":
        fail(f"requires_rebind nodes published instead of rejecting: {sorted(rebind_nodes)}")
    elif rebind_nodes:
        facts.append("rebind_nodes_reject_no_publish")

    if any(desc.get("kind") == "DynamicPotential" for desc in descriptors):
        if result_state != "rejected":
            fail("dynamic-potential descriptor did not reject")
        else:
            facts.append("dynamic_potential_descriptor_rejects")

    return {
        "artifact_type": "treecalc_result",
        "case_id": case_id,
        "path": result_path.relative_to(REPO_ROOT).as_posix(),
        "result_state": result_state,
        "facts": sorted(set(facts)),
        "failures": failures,
        "blockers": blockers,
    }


def validate_tracecalc_result(result_path: Path) -> dict[str, Any]:
    result = load_json(result_path)
    scenario_id = result.get("scenario_id", result_path.parent.name)
    facts: list[str] = []
    failures: list[str] = []
    blockers: list[str] = []

    artifact_paths = result.get("artifact_paths", {})
    for required in ["trace", "rejects", "published_view", "counters"]:
        if required not in artifact_paths:
            failures.append(f"tracecalc result missing artifact_paths.{required}")
    if failures:
        trace = {"events": []}
    else:
        trace = load_json(resolve_artifact(artifact_paths["trace"]))
        for key in ["rejects", "published_view", "counters"]:
            load_json(resolve_artifact(artifact_paths[key]))
        facts.append("tracecalc_sidecars_parseable")

    events = trace.get("events", [])
    if not isinstance(events, list):
        failures.append("trace.events is not a list")
        events = []
    labels = [event.get("label") for event in events]
    normalized = [event.get("normalized_event_family") for event in events]
    if any(not label for label in labels):
        failures.append("trace event missing label")
    if any(not family for family in normalized):
        failures.append("trace event missing normalized_event_family")
    else:
        facts.append("normalized_event_families_present")

    replay_projection = result.get("replay_projection") or trace.get("replay_projection")
    if not replay_projection:
        failures.append("missing replay_projection")
    else:
        required_surfaces = set(replay_projection.get("required_equality_surfaces", []))
        missing_surfaces = required_surfaces - set(artifact_paths.keys())
        # Published view is named published_view in artifact_paths; reject_set is carried by rejects.
        missing_surfaces.discard("reject_set") if "rejects" in artifact_paths else None
        missing_surfaces.discard("trace_labels") if "trace" in artifact_paths else None
        if missing_surfaces:
            failures.append(f"missing required equality surface artifacts: {sorted(missing_surfaces)}")
        else:
            facts.append("required_equality_surfaces_have_artifacts")

    if "cycle_region_detected" in labels:
        if "candidate_rejected" not in labels:
            failures.append("cycle trace lacks candidate_rejected event")
        if "publication_committed" in labels or "candidate_published" in labels:
            failures.append("cycle trace contains publication event")
        facts.append("cycle_region_reject_no_publication")

    if result.get("assertion_failures") == [] and result.get("conformance_mismatches") == []:
        facts.append("tracecalc_assertions_and_conformance_clean")
    else:
        failures.append("tracecalc assertions or conformance mismatches are present")

    if result.get("validation_failures") == []:
        facts.append("tracecalc_validation_clean")
    else:
        failures.append("tracecalc validation failures are present")

    return {
        "artifact_type": "tracecalc_result",
        "scenario_id": scenario_id,
        "path": result_path.relative_to(REPO_ROOT).as_posix(),
        "result_state": result.get("result_state"),
        "facts": sorted(set(facts)),
        "failures": failures,
        "blockers": blockers,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--treecalc-result", action="append", default=[], help="TreeCalc result.json path")
    parser.add_argument("--tracecalc-result", action="append", default=[], help="TraceCalc result.json path")
    parser.add_argument("--output", required=True, help="Output JSON path")
    args = parser.parse_args()

    checked = []
    for path in args.treecalc_result:
        checked.append(validate_treecalc_result(resolve_artifact(path)))
    for path in args.tracecalc_result:
        checked.append(validate_tracecalc_result(resolve_artifact(path)))

    failures = [failure for item in checked for failure in item["failures"]]
    blockers = [blocker for item in checked for blocker in item["blockers"]]
    output = {
        "schema_version": "oxcalc.w046.proof_carrying_trace.validation.v1",
        "checker": "scripts/check-w046-proof-carrying-trace.py",
        "checked_artifacts": checked,
        "checked_artifact_count": len(checked),
        "failure_count": len(failures),
        "blocker_count": len(blockers),
        "failures": failures,
        "blockers": blockers,
        "result": "passed" if not failures else "failed",
    }
    output_path = resolve_artifact(args.output)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(json.dumps(output, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(json.dumps({"result": output["result"], "checked_artifact_count": len(checked), "failure_count": len(failures)}))
    return 0 if not failures else 1


if __name__ == "__main__":
    raise SystemExit(main())
