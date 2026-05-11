# W048 Innovation Opportunity Ledger

Status: `profile_gated_ledger_recorded`

Machine-readable ledger: `W048_INNOVATION_OPPORTUNITY_LEDGER.json`

Validation command: `scripts/check-w048-innovation-ledger.ps1`

## 1. Purpose

OxCalc's first priority is explicit Excel matching where that is the declared profile. W048 also owns a separate lane for opportunities where OxCalc can be better than ordinary spreadsheet circular-reference handling in model-oriented use cases.

This ledger keeps those opportunities visible without contaminating the default Excel-match semantics.

## 2. Profile Rule

Every innovation must be profile-gated.

Required fields:

1. profile name;
2. target user/modeling problem;
3. relationship to Excel behavior;
4. semantic contract;
5. termination or rejection policy;
6. graph/materialization requirements;
7. replay/formal obligations;
8. tests required before use.

## 3. Candidate Opportunities

### 3.1 Monotone Fixed-Point Profile

Problem:

Some model cycles are intentional monotone equations rather than accidental circular references.

Candidate OxCalc behavior:

1. admit only formulas that satisfy a declared monotonicity/domain contract;
2. calculate least fixed point or bounded fixed point under a declared lattice/domain;
3. emit fixed-point trace and convergence evidence.

Excel relation:

1. not default Excel compatibility;
2. opt-in model profile.

### 3.2 Explicit Recurrence Profile

Problem:

Users often encode recurrence/state using circular references because spreadsheet cells have no direct recurrence construct.

Candidate OxCalc behavior:

1. offer explicit recurrence nodes with time/index state;
2. separate recurrence from accidental SCCs;
3. expose previous-state dependency as a typed edge rather than an ordinary same-wave cycle.

Excel relation:

1. intentionally clearer than circular-reference iteration;
2. requires explicit user/model opt-in.

### 3.3 Cycle Diagnostic Graph View

Problem:

Circular references are difficult to debug when the graph is large or dynamic.

Candidate OxCalc behavior:

1. materialized cycle region with root/order policy;
2. entry and exit edge listing;
3. CTRO provenance for dynamic edges;
4. release/re-entry history;
5. stable graph hash for audit.

Excel relation:

1. can improve diagnostics while preserving Excel-compatible calculation behavior.

### 3.4 Safer Retained-Prior-Value Semantics

Problem:

Retained prior values can hide failed cycle recalculation if they are not distinguished from new publication.

Candidate OxCalc behavior:

1. keep previous published value visible only with explicit stale/cycle-retained state;
2. prevent downstream consumers from treating retained values as fresh unless profile allows it;
3. expose diagnostic state in graph and trace artifacts.

Excel relation:

1. can match visible Excel behavior while offering stronger machine-readable state.

### 3.5 Bounded Iteration With Oscillation Diagnostics

Problem:

Bounded iterative cycles can diverge or oscillate.

Candidate OxCalc behavior:

1. record per-iteration deltas;
2. detect simple oscillation patterns;
3. classify terminal state as convergence, max-iteration, divergence, or oscillation;
4. decide publication based on profile.

Excel relation:

1. may match Excel values while adding diagnostics;
2. any difference in terminal publication must be opt-in.

### 3.6 Local Frontier Repair After Release

Problem:

Full rebuild after a dynamic cycle release can be expensive.

Candidate OxCalc behavior:

1. use materialized reverse edges and overlay deltas to invalidate exactly the affected members and dependents;
2. repair acyclic frontier order locally;
3. prove or check equivalence to full rebuild for the profile.

Excel relation:

1. optimization only if observable results match the selected profile.

## 4. Non-Default Rule

An innovation cannot become default behavior unless:

1. Excel observations show compatibility for the declared Excel-match surface, or
2. the profile is explicitly not an Excel-match profile and callers opt in.

## 5. Machine-Readable Entries

The JSON ledger records six profile-gated entries:

1. `cycle.fixed_point.monotone_lfp.v0`
2. `cycle.recurrence.explicit_state.v0`
3. `cycle.diagnostics.materialized_region.v0`
4. `cycle.retained_prior.safe_state.v0`
5. `cycle.iterative.bounded_with_oscillation_diagnostics.v0`
6. `cycle.release.local_frontier_repair.v0`

Each entry names the target problem, Excel relationship, semantic contract, termination/rejection policy, graph requirements, required tests, formal obligations, and admission gate.

## 6. Status Surface

- execution_state: `ledger_recorded_and_checker_passed`
- scope_completeness: `scope_partial`
- target_completeness: `target_partial`
- integration_completeness: `partial`
- open_lanes:
  - candidate profiles are not admitted as default behavior.
  - formal/profile obligations remain prerequisites before any experimental profile can be enabled.
  - successor work must add fixtures before any profile changes publication semantics.
