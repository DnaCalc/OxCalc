#!/usr/bin/env python3
"""Validate W048 innovation opportunity ledger profile-gating fields."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
LEDGER = ROOT / "docs/spec/core-engine/w048-cycles/W048_INNOVATION_OPPORTUNITY_LEDGER.json"
REQUIRED_FIELDS = [
    "profile_id",
    "status",
    "target_problem",
    "excel_relation",
    "semantic_contract",
    "termination_or_rejection_policy",
    "graph_requirements",
    "required_tests",
    "formal_obligations",
    "admission_gate",
]


def main() -> int:
    data = json.loads(LEDGER.read_text(encoding="utf-8"))
    errors: list[str] = []
    if data.get("schema_version") != "oxcalc.w048.innovation_opportunity_ledger.v1":
        errors.append("unexpected schema_version")
    if "No innovation profile is default Excel compatibility" not in data.get("default_rule", ""):
        errors.append("default rule must prohibit default Excel-compatibility promotion")
    entries = data.get("entries", [])
    if len(entries) < 6:
        errors.append("expected at least six innovation entries")
    ids = set()
    for entry in entries:
        profile_id = entry.get("profile_id", "<missing>")
        if profile_id in ids:
            errors.append(f"duplicate profile_id {profile_id}")
        ids.add(profile_id)
        for field in REQUIRED_FIELDS:
            if field not in entry:
                errors.append(f"{profile_id} missing {field}")
        for field in ["graph_requirements", "required_tests", "formal_obligations"]:
            if not isinstance(entry.get(field), list) or not entry.get(field):
                errors.append(f"{profile_id} {field} must be non-empty list")
        if entry.get("excel_relation") == "excel_match_default":
            errors.append(f"{profile_id} incorrectly marked as default Excel match")
        if entry.get("status") not in {
            "candidate_profile_not_admitted",
            "diagnostic_profile_partially_exercised",
            "optimization_profile_partially_exercised",
        }:
            errors.append(f"{profile_id} has unsupported status {entry.get('status')}")

    required_profiles = {
        "cycle.fixed_point.monotone_lfp.v0",
        "cycle.recurrence.explicit_state.v0",
        "cycle.diagnostics.materialized_region.v0",
        "cycle.retained_prior.safe_state.v0",
        "cycle.iterative.bounded_with_oscillation_diagnostics.v0",
        "cycle.release.local_frontier_repair.v0",
    }
    missing = sorted(required_profiles - ids)
    if missing:
        errors.append(f"missing profiles: {missing}")

    if errors:
        print("w048 innovation ledger FAILED")
        for error in errors:
            print(f"ERROR: {error}")
        return 1
    print("w048 innovation ledger ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
