# W048 Whole-Workset Fresh-Eyes Audit

Status: `whole_workset_audited_with_named_blockers`

Audit date: 2026-05-11

Parent bead: `calc-zci1`

## 1. Objective Restated As Concrete Deliverables

User objective: run the W048 workset and beads; after each bead perform a fresh-eyes review; after all beads are complete, perform a whole-workset fresh-eyes audit and fix/rework issues.

Concrete deliverables checked here:

1. reopened W048 scope repaired and bead graph prepared;
2. each reopened child bead `calc-zci1.9` through `calc-zci1.15` executed and closed with a fresh-eyes review;
3. W048 Python checker dependency removed from the active local validation path;
4. Excel core and bit-exact observation packets created and normalized;
5. Excel-match iterative profile specified with falsification fixtures and exact blockers;
6. TraceCalc reference fixtures and replay-visible iteration traces added;
7. TreeCalc optimized/core fixtures and opt-in iterative publication path added;
8. conformance summary updated to cover reopened iterative evidence;
9. whole-workset audit performed without overclaiming parent W048 closure.

## 2. Prompt-To-Artifact Checklist

| Requirement | Evidence | Audit result |
| --- | --- | --- |
| Run W048 beads | `.beads/issues.jsonl`; `br ready --json`; child beads `calc-zci1.9` through `calc-zci1.15` closed | satisfied for reopened child beads |
| Fresh-eyes review after each bead | `W048_REOPEN_SCOPE_AUDIT_AND_REPAIR_PLAN.md`; `W048_TOOLING_MIGRATION_OFF_PYTHON.md`; `W048_EXCEL_OBSERVATION_LEDGER.md`; `W048_ITERATIVE_PROFILE_DECISION_AND_EXCEL_DISPOSITION.md`; `W048_TRACECALC_REFERENCE_CYCLE_BEHAVIOR.md`; `W048_TREECALC_OPTIMIZED_CYCLE_BEHAVIOR.md`; `W048_CORPUS_AND_CONFORMANCE_EVIDENCE.md` | satisfied |
| No Python local tooling | removed `scripts/check-w048-*.py`; added PowerShell checkers | satisfied for W048 checker surface |
| Excel observation suite | `w048-excel-cycles-001`; `w048-excel-cycles-bitexact-001`; `scripts/check-w048-excel-observation-packet.ps1` | satisfied for declared packets; blockers remain for root/prior/version/MT |
| Iterative profile specification | `W048_ITERATIVE_PROFILE_DECISION.json` v2; `scripts/check-w048-iterative-profile-decision.ps1` | satisfied with named blockers |
| TraceCalc iterative reference | `w048-tracecalc-cycles-004`; `scripts/check-w048-tracecalc-iterative-cycles.ps1`; four TraceCalc fixtures | satisfied for declared fixtures |
| TreeCalc iterative optimized/core slice | `w048-treecalc-cycles-002`; `scripts/check-w048-treecalc-iterative-cycles.ps1`; four TreeCalc fixtures | satisfied for declared fixtures |
| Reopened iterative graph sidecars | `w048-treecalc-cycles-002/w048_materialized_graph_check_summary.json`; `scripts/write-w048-materialized-graphs.ps1`; `scripts/check-w048-materialized-graphs.ps1` | satisfied: 37 cases / 111 layers / 24 cycle-region records |
| Conformance matrix | `w048-conformance-002`; `scripts/check-w048-conformance.ps1` | passed with named blockers |
| Whole-workset audit | this file | satisfied; parent W048 remains open/partial |

## 3. Fresh-Eyes Findings

1. **No proxy green overclaim**: `scripts/check-w048-conformance.ps1` now reports `passed_with_named_excel_blockers`, not final broad Excel closure.
2. **Status truth preserved**: parent `calc-zci1` remains open; W048 status axes remain partial while blockers exist.
3. **Iterative evidence exists on both engines**: TraceCalc and TreeCalc both cover the same four falsification fixtures.
4. **Observation caveats visible**: root/report-cell and cross-version repeat are explicit blockers; numeric/nonnumeric prior-state and multithread variant behavior have targeted evidence.
5. **Rework performed after audit finding**: the initial whole-workset audit found that materialized graph sidecar validation was still bound to the predecessor `w048-treecalc-cycles-001` floor. This was reworked by adding `scripts/write-w048-materialized-graphs.ps1`, regenerating sidecars for `w048-treecalc-cycles-002`, and validating 37 cases / 111 layers / 24 cycle-region records / 0 checker errors.
6. **Implementation scope caveat**: TreeCalc optimized support currently covers declared Excel falsification fixtures through an opt-in compatibility basis. It is not a universal parser/evaluator for arbitrary Excel circular-reference workbooks.

## 4. Exact Open Blockers

These active Excel blockers prevent marking parent W048 or the active user goal complete. The graph-sidecar item is retained below as a cleared audit-trail item:

1. `BLK-W048-EXCEL-ROOT`: capture report-cell/root behavior through UI warning or a public object-model surface that does not return null for circular references in current COM packets.
2. `BLK-W048-EXCEL-INITIAL`: cleared for numeric-prior self-cycle behavior by `w048-excel-initial-vector-001`; nonnumeric prior behavior remains under `BLK-W048-EXCEL-NONNUMERIC`.
3. `BLK-W048-EXCEL-NONNUMERIC`: cleared for declared self-cycle prior-state behavior by `w048-excel-nonnumeric-prior-001`.
4. `BLK-W048-EXCEL-VERSION`: repeat the falsification fixture set on a second Excel host/version before broad compatibility claims.
5. `BLK-W048-EXCEL-MT`: cleared as a run requirement by `w048-excel-multithread-variant-001`; thread mode remains a profile dimension because multithread values differ from single-thread fixtures.
6. `BLK-W048-GRAPH-ITER-SIDECARS`: cleared during post-audit rework by regenerating/checking `w048-treecalc-cycles-002` sidecars. Retained here as an audit trail item, not an active blocker.

## 5. Validation Commands From Whole-Workset Audit

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/check-w048-excel-observation-packet.ps1 docs/test-runs/excel-cycles/w048-excel-cycles-001
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/check-w048-excel-observation-packet.ps1 docs/test-runs/excel-cycles/w048-excel-cycles-bitexact-001 -MinimumProbeCount 19
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/check-w048-iterative-profile-decision.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/check-w048-tracecalc-iterative-cycles.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/check-w048-treecalc-iterative-cycles.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/write-w048-materialized-graphs.ps1 docs/test-runs/core-engine/treecalc-local/w048-treecalc-cycles-002
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/check-w048-materialized-graphs.ps1 docs/test-runs/core-engine/treecalc-local/w048-treecalc-cycles-002
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/check-w048-conformance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/check-w048-formal-cycle-artifacts.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/check-w048-innovation-ledger.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/check-w048-closure-audit.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/check-worksets.ps1
cargo test -p oxcalc-core
cargo test -p oxcalc-tracecalc
cargo fmt --all -- --check
git diff --check
br dep cycles --json
```

## 6. Three-Axis Status

- scope_completeness: `scope_partial`
- target_completeness: `target_partial`
- integration_completeness: `partial`
- open_lanes:
  - `BLK-W048-EXCEL-ROOT`;
  - `BLK-W048-EXCEL-VERSION`;
  - parent `calc-zci1` final closure after blocker disposition.

## 7. Audit Decision

Do not mark the active user goal complete. All reopened child beads have been processed and reviewed, but the W048 parent and broad Excel-match closure remain blocked by exact, recorded blockers.
