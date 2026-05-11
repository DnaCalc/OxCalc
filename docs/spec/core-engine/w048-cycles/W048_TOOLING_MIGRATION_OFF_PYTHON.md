# W048 Tooling Migration Off Python

Status: `calc-zci1.10_tooling_migrated`

Parent workset: `W048 Circular Dependency Calculation Processing`

Parent bead: `calc-zci1.10`

## 1. Purpose

Reopened W048 must not depend on Python for local validation or checker tooling. This packet records the first W048 tooling migration: predecessor Python checkers have been removed from `scripts/` and replaced with PowerShell entry points.

This is a tooling migration only. It does not change engine behavior, TraceCalc behavior, TreeCalc behavior, Excel observations, or W048 semantic scope.

## 2. Replacement Scripts

| Former W048 checker role | Replacement script |
| --- | --- |
| materialized graph sidecar checks | `scripts/check-w048-materialized-graphs.ps1` |
| cross-corpus conformance summary checks | `scripts/check-w048-conformance.ps1` |
| iterative profile decision predecessor check | `scripts/check-w048-iterative-profile-decision.ps1` |
| formal cycle artifact summary checks | `scripts/check-w048-formal-cycle-artifacts.ps1` |
| innovation opportunity ledger checks | `scripts/check-w048-innovation-ledger.ps1` |
| reopened closure-state audit | `scripts/check-w048-closure-audit.ps1` |

The old `scripts/check-w048-*.py` files are removed. W048 docs that previously named those paths now either point to PowerShell replacements or classify the old Python evidence as historical predecessor context.

## 3. Validation Commands

The following non-Python checks were run for this bead:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/check-w048-iterative-profile-decision.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/check-w048-innovation-ledger.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/check-w048-materialized-graphs.ps1 docs/test-runs/core-engine/treecalc-local/w048-treecalc-cycles-001
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/check-w048-materialized-graphs.ps1 docs/test-runs/core-engine/treecalc-local/w048-materialized-graph-001
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/check-w048-conformance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/check-w048-formal-cycle-artifacts.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/check-w048-closure-audit.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/check-worksets.ps1
git diff --check
br dep cycles --json
```

Observed result: all commands passed, with only normal line-ending warnings from Git where applicable.

## 4. Fresh-Eyes Review For `calc-zci1.10`

Review date: 2026-05-11

Review questions:

1. Did this bead actually remove W048 Python checker entry points rather than merely adding wrappers?
2. Do replacement scripts cover each predecessor checker role?
3. Are active W048 docs free of instructions to run Python for W048 validation?
4. Do replacement scripts validate real artifact contents rather than only file existence?
5. Does this bead accidentally change engine behavior or W048 semantic claims?

Findings:

1. `scripts/check-w048-*.py` files were removed and `scripts/check-w048-*.ps1` files were added.
2. Replacement coverage exists for materialized graph checks, conformance, iterative-decision predecessor validation, formal-cycle artifacts, innovation ledger, and reopened closure-state audit.
3. Remaining `Python` mentions in W048 docs are explanatory historical notes or no-Python doctrine, not active validation commands.
4. The PowerShell checks inspect JSON fields, run summaries, graph-layer sidecars, cycle-region records, conformance summaries, formal obligations, bead status, and reopened workset text.
5. No Rust, TraceCalc, TreeCalc, or Excel-observation behavior was changed.

Fresh-eyes result: `calc-zci1.10` satisfies the no-Python tooling migration gate for the current W048 local checker surface. Future W048 tooling must remain PowerShell, Rust, or C#.

## 5. Three-Axis Status

- execution_state: `in_progress`
- scope_completeness: `scope_partial`
- target_completeness: `target_partial`
- integration_completeness: `partial`
- open_lanes:
  - Excel bit-exact circular-reference observation suite;
  - Excel-match iterative profile specification;
  - TraceCalc bit-exact iterative implementation;
  - TreeCalc optimized iterative implementation;
  - full conformance and closure audit.
