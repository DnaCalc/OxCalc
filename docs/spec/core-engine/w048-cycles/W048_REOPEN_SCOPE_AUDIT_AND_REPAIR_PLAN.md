# W048 Reopen Scope Audit And Repair Plan

Status: `reopened_scope_repair`

Parent workset: `W048 Circular Dependency Calculation Processing`

Parent epic: `calc-zci1`

## 1. Reopen Trigger

W048 is reopened because the intended scope is the comprehensive circular-reference solution, including Excel-behavior matching and bit-exact iterative calculation behavior. The prior W048 closure narrowed the workset to a conservative non-iterative Stage 1 boundary and routed Excel-match iterative behavior to future work.

That closure is now superseded. W048 is `in_progress` until the full intended circular-reference solution is implemented, validated, and audited or until exact blockers are explicitly accepted by the user.

## 2. Why The Earlier Run Missed The Intended Scope

The prior run missed the intended W048 scope through a silent narrowing pattern:

1. The workset purpose said circular dependency calculation was owned end to end, but it also stated that Excel-compatible cycle closure and iterative calculation were not claimed until later evidence existed.
2. The closure gate allowed "select an initial iterative profile or explicitly defer it" as sufficient, which converted a core behavior requirement into a decision-record artifact.
3. Bead `calc-zci1.4` was scoped as an iterative-profile algorithm decision and Excel disposition, not as an implementation bead.
4. The closure audit treated `cycle.excel_match_iterative = not_admitted_yet` as acceptable closure evidence instead of a target gap.
5. Successor routing moved broader Excel-match probes, iterative behavior, and deeper formal proof out of W048, even though the intended W048 target was the full circular-reference solution.
6. Several W048 validation/checker surfaces used Python scripts, which is no longer allowed for local OxCalc tooling.

Root cause: scope text and closure text conflicted. The narrow closure condition won operationally, so the run produced a correct conservative Stage 1 packet but reported W048 as closed. Under the user's clarified intent, that was premature closure.

## 3. Corrected W048 Scope

W048 now owns the full circular-reference calculation target:

1. Excel-compatible non-iterative circular-reference behavior;
2. Excel-compatible iterative calculation behavior, including bit-exact values for covered probes;
3. circular-reference root/order, initial-value, calculation-chain, cold-open, edit-history, max-iteration, max-change, convergence, non-convergence, oscillation, blank/text/error-prior, and diagnostic behavior;
4. TraceCalc reference implementation for the Excel-match profile;
5. TreeCalc optimized/core implementation for the same profile;
6. materialized graph, cycle-region, invalidation, iteration-step, publication, and no-publication sidecars;
7. conformance against normalized Excel observations;
8. formal/model/checker artifacts grounded in implementation and evidence;
9. non-Python local tooling only: PowerShell, Rust, or C#.

## 4. Superseded Closure Artifacts

The following artifacts remain historical evidence for the conservative Stage 1 slice but no longer close W048:

1. `docs/spec/core-engine/w048-cycles/W048_CLOSURE_AUDIT_AND_SUCCESSOR_ROUTING.md`
2. `docs/test-runs/core-engine/w048-closure-audit-001/w048_closure_audit_summary.json`
3. `docs/spec/core-engine/w048-cycles/W048_ITERATIVE_PROFILE_DECISION_AND_EXCEL_DISPOSITION.md`
4. Python checker paths named in W048 docs or artifacts.

They may be consumed as predecessor evidence only after this reopen packet is considered first.

## 5. Reopened Bead Plan

New W048 beads:

| Bead | Purpose |
| --- | --- |
| `calc-zci1.9` | reopen audit and full Excel-match scope repair |
| `calc-zci1.10` | migrate local cycle tooling off Python |
| `calc-zci1.11` | Excel bit-exact circular-reference observation suite |
| `calc-zci1.12` | Excel-match iterative profile specification |
| `calc-zci1.13` | TraceCalc bit-exact iterative cycle reference implementation |
| `calc-zci1.14` | TreeCalc optimized iterative cycle implementation |
| `calc-zci1.15` | full circular-reference conformance and closure audit |

## 6. Updated Closure Rule

W048 cannot close while any of the following is true:

1. Excel-match iterative behavior is only deferred, not implemented or exactly blocked with user acceptance;
2. TraceCalc and TreeCalc do not both execute the declared Excel-match circular-reference profile;
3. conformance evidence does not compare Excel observations, TraceCalc, and TreeCalc for the declared coverage;
4. local W048 validation depends on Python scripts;
5. closure language depends on successor routing for core W048 semantics.

## 7. Fresh-Eyes Review For `calc-zci1.9`

Review date: 2026-05-11

Checks performed:

1. Re-read `docs/worksets/W048_CIRCULAR_DEPENDENCY_CALCULATION_PROCESSING.md` against the user-stated full Excel-match W048 intent.
2. Re-read `docs/spec/core-engine/w048-cycles/README.md` and the superseded closure/iterative-decision packets for stale closure claims.
3. Inspected bead graph readiness with `br ready --json` and dependency cycles with `br dep cycles --json`.
4. Ran `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/check-worksets.ps1`.
5. Ran `git diff --check`.

Findings and repairs:

1. Found one stale closure-gate clause that still allowed "iterative-profile selection or explicit deferral" before successor routing. Repaired it to require Excel-match iterative implementation/validation or exact user-accepted blockers.
2. Confirmed the prior closure audit is marked superseded and no longer reports current W048 status.
3. Confirmed new active beads `calc-zci1.9` through `calc-zci1.15` exist, with `calc-zci1.9` as the only ready bead and downstream beads blocked in order.
4. Confirmed W048 Python checker references were routed away from active closure evidence.
5. Follow-up `calc-zci1.10` added PowerShell replacements for W048 local checkers and removed the W048 Python checker scripts.
6. No engine behavior was changed by this bead.

Fresh-eyes result: `calc-zci1.9` scope repair is coherent after the stale successor-routing clause repair. W048 remains in progress and partial.

## 8. Three-Axis Status

- execution_state: `in_progress`
- scope_completeness: `scope_partial`
- target_completeness: `target_partial`
- integration_completeness: `partial`
- open_lanes:
  - keep W048 tooling on PowerShell/Rust/C# and do not reintroduce Python;
  - collect bit-exact Excel iterative observations;
  - specify the Excel-match iterative profile;
  - implement TraceCalc reference behavior;
  - implement TreeCalc optimized/core behavior;
  - run full conformance and closure audit.
