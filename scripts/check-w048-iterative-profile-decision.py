#!/usr/bin/env python3
"""Validate the W048 iterative-profile decision packet."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DECISION = ROOT / "docs/spec/core-engine/w048-cycles/W048_ITERATIVE_PROFILE_DECISION.json"


def main() -> int:
    data = json.loads(DECISION.read_text(encoding="utf-8"))
    errors: list[str] = []

    if data.get("schema_version") != "oxcalc.w048.iterative_profile_decision.v1":
        errors.append("unexpected schema_version")

    default = data.get("default_profile", {})
    if default.get("profile_id") != "cycle.non_iterative_stage1":
        errors.append("missing default non-iterative profile")
    if default.get("cycle_terminal_state") != "reject_candidate":
        errors.append("default terminal state must reject candidate")
    if "no_new_cycle_values" not in default.get("publication_rule", ""):
        errors.append("default publication rule must prohibit new cycle values")

    excel = data.get("excel_disposition", {})
    if excel.get("profile_id") != "cycle.excel_match_iterative":
        errors.append("missing Excel-match profile disposition")
    if excel.get("status") != "not_admitted_yet":
        errors.append("Excel-match iterative profile must remain not admitted")
    if not excel.get("source_run", "").endswith("w048-excel-cycles-001/observation.json"):
        errors.append("Excel disposition must bind the W048 observation run")

    future = data.get("future_profile", {})
    required = {
        "profile_id": "cycle.iterative_deterministic_v0",
        "admission": "explicit_opt_in_only",
        "root_policy": "canonical_node_id_root",
        "update_model": "jacobi_snapshot_region_function",
        "publication_rule": "publish_only_after_converged_complete_region_and_dependent_wave",
    }
    for key, expected in required.items():
        if future.get(key) != expected:
            errors.append(f"future_profile.{key} expected {expected!r} observed {future.get(key)!r}")

    initial = future.get("initial_value_policy", {})
    for key in ["cycle_members", "non_numeric_or_error_prior", "missing_boundary_value"]:
        if key not in initial:
            errors.append(f"missing initial_value_policy.{key}")

    stop = future.get("stop_metric", {})
    if stop.get("kind") != "max_absolute_member_delta":
        errors.append("stop metric must be max_absolute_member_delta")
    if stop.get("default_max_change") != "0.001":
        errors.append("default max_change must be 0.001")

    bound = future.get("iteration_bound", {})
    if bound.get("default_max_iterations") != 100:
        errors.append("default max_iterations must be 100")

    terminal = future.get("terminal_states", {})
    for key in [
        "converged",
        "max_iterations_without_convergence",
        "divergence_detected",
        "oscillation_detected",
        "non_numeric_result",
    ]:
        if key not in terminal:
            errors.append(f"missing terminal state {key}")
    if terminal.get("converged") != "accept_candidate_and_publish_whole_region_atomically":
        errors.append("converged terminal must publish whole region atomically")
    for key, value in terminal.items():
        if key != "converged" and value != "reject_candidate_no_publication":
            errors.append(f"{key} terminal must reject with no publication")

    diagnostics = set(future.get("diagnostics", []))
    for required_diag in ["cycle_region_id", "cycle_root", "cycle_member_order", "iteration_index", "max_member_delta", "terminal_state"]:
        if required_diag not in diagnostics:
            errors.append(f"missing diagnostic {required_diag}")

    if "semantic_equivalence_requirement" not in future:
        errors.append("missing semantic_equivalence_requirement")

    if len(data.get("open_observation_requirements_before_excel_match", [])) < 5:
        errors.append("Excel-match open observation requirements too small")

    if errors:
        print("w048 iterative profile decision FAILED")
        for error in errors:
            print(f"ERROR: {error}")
        return 1
    print("w048 iterative profile decision ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
