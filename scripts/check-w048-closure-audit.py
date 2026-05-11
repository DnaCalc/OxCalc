#!/usr/bin/env python3
"""Verify W048 closure evidence across beads, artifacts, and checker summaries."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "docs/test-runs/core-engine/w048-closure-audit-001"
CHILDREN = [f"calc-zci1.{i}" for i in range(1, 9)]

REQUIRED_FILES = [
    "scripts/run-w048-excel-cycle-probes.ps1",
    "scripts/check-w048-materialized-graphs.py",
    "scripts/check-w048-conformance.py",
    "scripts/check-w048-iterative-profile-decision.py",
    "scripts/check-w048-formal-cycle-artifacts.py",
    "scripts/check-w048-innovation-ledger.py",
    "docs/spec/core-engine/w048-cycles/W048_EXCEL_OBSERVATION_LEDGER.md",
    "docs/spec/core-engine/w048-cycles/W048_MATERIALIZED_GRAPH_SIDECAR_EVIDENCE.md",
    "docs/spec/core-engine/w048-cycles/W048_TRACECALC_REFERENCE_CYCLE_BEHAVIOR.md",
    "docs/spec/core-engine/w048-cycles/W048_TREECALC_OPTIMIZED_CYCLE_BEHAVIOR.md",
    "docs/spec/core-engine/w048-cycles/W048_CORPUS_AND_CONFORMANCE_EVIDENCE.md",
    "docs/spec/core-engine/w048-cycles/W048_ITERATIVE_PROFILE_DECISION_AND_EXCEL_DISPOSITION.md",
    "docs/spec/core-engine/w048-cycles/W048_FORMAL_CYCLE_DEFINITIONS_AND_CHECKER_ARTIFACTS.md",
    "docs/spec/core-engine/w048-cycles/W048_INNOVATION_OPPORTUNITY_LEDGER.md",
    "docs/test-runs/excel-cycles/w048-excel-cycles-001/observation.json",
    "docs/test-runs/core-engine/tracecalc-reference-machine/w048-tracecalc-cycles-003/run_summary.json",
    "docs/test-runs/core-engine/treecalc-local/w048-treecalc-cycles-001/run_summary.json",
    "docs/test-runs/core-engine/treecalc-local/w048-treecalc-cycles-001/w048_materialized_graph_check_summary.json",
    "docs/test-runs/core-engine/w048-conformance-001/w048_conformance_summary.json",
    "docs/test-runs/core-engine/formal/w048-cycle-artifacts-001/w048_formal_cycle_checker_summary.json",
]


def load_json(path: Path, *, encoding: str = "utf-8"):
    return json.loads(path.read_text(encoding=encoding))


def bead_statuses() -> dict[str, str]:
    statuses = {}
    for line in (ROOT / ".beads/issues.jsonl").read_text(encoding="utf-8").splitlines():
        obj = json.loads(line)
        if obj.get("id", "").startswith("calc-zci1"):
            statuses[obj["id"]] = obj.get("status")
    return statuses


def main() -> int:
    errors: list[str] = []
    statuses = bead_statuses()
    for child in CHILDREN:
        if statuses.get(child) != "closed":
            errors.append(f"{child} expected closed observed {statuses.get(child)}")

    missing_files = [path for path in REQUIRED_FILES if not (ROOT / path).exists()]
    for path in missing_files:
        errors.append(f"missing required file {path}")

    if not missing_files:
        excel = load_json(ROOT / "docs/test-runs/excel-cycles/w048-excel-cycles-001/observation.json", encoding="utf-8-sig")
        if excel.get("observation_count") != 12:
            errors.append("Excel observation_count expected 12")
        trace = load_json(ROOT / "docs/test-runs/core-engine/tracecalc-reference-machine/w048-tracecalc-cycles-003/run_summary.json")
        if trace.get("scenario_count") != 34 or trace.get("result_counts", {}).get("passed") != 34:
            errors.append("TraceCalc W048 run expected 34 passed scenarios")
        tree = load_json(ROOT / "docs/test-runs/core-engine/treecalc-local/w048-treecalc-cycles-001/run_summary.json")
        if tree.get("case_count") != 33 or tree.get("expectation_mismatch_count") != 0:
            errors.append("TreeCalc W048 run expected 33 cases and 0 mismatches")
        graph = load_json(ROOT / "docs/test-runs/core-engine/treecalc-local/w048-treecalc-cycles-001/w048_materialized_graph_check_summary.json")
        if graph.get("case_count") != 33 or graph.get("layer_count") != 99 or graph.get("cycle_region_count") != 12 or graph.get("check_error_count") != 0:
            errors.append("W048 graph checker summary expected 33 cases / 99 layers / 12 cycle regions / 0 errors")
        conformance = load_json(ROOT / "docs/test-runs/core-engine/w048-conformance-001/w048_conformance_summary.json")
        if conformance.get("status") != "passed" or conformance.get("errors"):
            errors.append("W048 conformance summary must pass with no errors")
        formal = load_json(ROOT / "docs/test-runs/core-engine/formal/w048-cycle-artifacts-001/w048_formal_cycle_checker_summary.json")
        if formal.get("status") != "passed" or formal.get("errors"):
            errors.append("W048 formal cycle checker summary must pass with no errors")

    summary = {
        "schema_version": "oxcalc.w048.closure_audit.v1",
        "run_id": "w048-closure-audit-001",
        "status": "passed" if not errors else "failed",
        "errors": errors,
        "bead_statuses": {key: statuses.get(key) for key in ["calc-zci1", *CHILDREN]},
        "required_file_count": len(REQUIRED_FILES),
        "missing_files": missing_files,
        "scope_completeness": "scope_complete" if not errors else "scope_partial",
        "target_completeness": "target_complete" if not errors else "target_partial",
        "integration_completeness": "integrated" if not errors else "partial",
        "open_lanes": [] if not errors else errors,
    }
    OUT.mkdir(parents=True, exist_ok=True)
    (OUT / "w048_closure_audit_summary.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
    print(f"w048 closure audit {summary['status']}: {OUT / 'w048_closure_audit_summary.json'}")
    if errors:
        for error in errors:
            print(f"ERROR: {error}")
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
