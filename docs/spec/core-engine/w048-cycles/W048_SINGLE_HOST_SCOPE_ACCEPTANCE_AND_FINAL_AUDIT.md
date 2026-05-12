# W048 Single-Host Scope Acceptance And Final Audit

Status: `accepted_single_host_scope_and_parent_disposition_ready`

Audit date: 2026-05-12

Parent bead: `calc-zci1`

User acceptance received:

```text
Let us record the single host scope and accept and close like that.
```

## 1. Scope Accepted

W048's Excel-match claim is scoped to the current single observed Excel host/version:

- Excel COM version: `16.0`
- Excel build: `19929`
- Excel product version: `16.0.19929.20136`
- local inventory packet: `docs/test-runs/excel-cycles/w048-excel-version-inventory-001/inventory.json`

Cross-version behavior remains a documented limitation rather than a closure blocker. W048 does not claim broad cross-version Excel compatibility.

## 2. Objective Restated As Concrete Deliverables

User objective: run the W048 workset and beads; after each bead perform a fresh-eyes review; after all beads are complete, perform a whole-workset fresh-eyes audit and fix/rework issues.

Concrete deliverables audited here:

1. W048 bead graph executed for all available/non-blocked beads;
2. every reopened bead has a fresh-eyes review packet or section;
3. blockers discovered during fresh-eyes review were either resolved with evidence or explicitly accepted as scope limitations;
4. whole-workset fresh-eyes audit was performed and reworked after findings;
5. local validation is PowerShell/Rust/C# and not Python;
6. status surfaces say single-host scoped Excel evidence, not broad cross-version compatibility;
7. parent W048 bead can be closed under the accepted single-host scope.

## 3. Prompt-To-Artifact Checklist

| Requirement | Evidence | Audit result |
| --- | --- | --- |
| Run W048 workset/beads | `.beads/issues.jsonl`; child beads `calc-zci1.1` through `calc-zci1.20` | satisfied after closing accepted-scope version bead |
| Fresh-eyes review after each reopened bead | `W048_REOPEN_SCOPE_AUDIT_AND_REPAIR_PLAN.md`; `W048_TOOLING_MIGRATION_OFF_PYTHON.md`; `W048_EXCEL_OBSERVATION_LEDGER.md`; `W048_ITERATIVE_PROFILE_DECISION_AND_EXCEL_DISPOSITION.md`; `W048_TRACECALC_REFERENCE_CYCLE_BEHAVIOR.md`; `W048_TREECALC_OPTIMIZED_CYCLE_BEHAVIOR.md`; `W048_CORPUS_AND_CONFORMANCE_EVIDENCE.md`; `W048_EXCEL_ROOT_REPORT_BLOCKER_PACKET.md`; `W048_EXCEL_VERSION_REPEAT_BLOCKER_PACKET.md`; `W048_EXCEL_MULTITHREAD_VARIANT_PACKET.md` | satisfied |
| Whole-workset fresh-eyes audit | `W048_WHOLE_WORKSET_FRESH_EYES_AUDIT.md`; this final accepted-scope audit | satisfied |
| Fix/rework audit findings | root/report repaired by `w048-excel-root-report-002`; graph sidecars regenerated for `w048-treecalc-cycles-002`; local version inventory added; stale blocker wording tightened | satisfied |
| No Python W048 checker path | `scripts/check-w048-*.ps1`; removed W048 Python checkers | satisfied |
| Excel bit-exact single-host evidence | `w048-excel-cycles-bitexact-001`; `w048-excel-root-report-002`; `w048-excel-initial-vector-001`; `w048-excel-nonnumeric-prior-001`; `w048-excel-multithread-variant-001` | satisfied for accepted single host |
| TraceCalc iterative evidence | `w048-tracecalc-cycles-004`; `scripts/check-w048-tracecalc-iterative-cycles.ps1` | satisfied for declared fixtures |
| TreeCalc iterative evidence | `w048-treecalc-cycles-002`; `scripts/check-w048-treecalc-iterative-cycles.ps1` | satisfied for declared fixtures |
| Materialized graph sidecars | `w048_materialized_graph_check_summary.json`; `scripts/check-w048-materialized-graphs.ps1` | satisfied |
| Conformance status truth | `w048-conformance-002/w048_conformance_summary.json`; `scripts/check-w048-conformance.ps1` | satisfied as single-host scoped, not broad cross-version |
| Version blocker disposition | `W048_EXCEL_VERSION_REPEAT_BLOCKER_PACKET.md`; user acceptance quoted above | accepted as scope limitation |

## 4. Fresh-Eyes Findings

1. The remaining version blocker cannot be resolved locally: `w048-excel-version-inventory-001` found one distinct local Excel product version.
2. User accepted single-host scoped closure, so `BLK-W048-EXCEL-VERSION` moves from active blocker to documented limitation.
3. The accepted scope must be visible in conformance/profile/status surfaces.
4. No broad cross-version Excel compatibility claim is made.
5. The parent W048 disposition is valid only for the declared fixtures and the single observed Excel host/version.

## 5. Completion Claim Self-Audit

1. Are all child beads closed or explicitly dispositioned? yes, after closing `calc-zci1.19` under accepted single-host scope.
2. Is any unresolved blocker hidden? no; cross-version behavior is documented as a limitation.
3. Are proxy green signals being treated as sufficient by themselves? no; the audit maps each objective requirement to artifacts and caveats.
4. Is the single-host scope explicit? yes.
5. Is there any remaining required local action? no.

## 6. Three-Axis Status

- scope_completeness: `scope_complete_single_host`
- target_completeness: `target_complete_single_host`
- integration_completeness: `integrated_single_host`
- open_lanes: []
- documented_limitations:
  - no broad cross-version Excel compatibility claim;
  - TreeCalc/TraceCalc cover declared W048 fixtures rather than arbitrary Excel workbook parsing;
  - multithread Excel behavior remains an observed profile dimension, not the implemented default fixture profile.

## 7. Audit Decision

W048 can close under the user-accepted single-host scope. Successor work must preserve the limitation that cross-version Excel behavior is not claimed by W048 evidence.
