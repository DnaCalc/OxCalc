# W048 TraceCalc Reference Cycle Behavior

Status: `active_execution_evidence`

## 1. Purpose

This packet records W048 TraceCalc reference behavior for circular dependency processing. It covers:

1. structural cycle rejection;
2. CTRO-created candidate cycle rejection with no overlay commit;
3. release/re-entry with downstream recomputation;
4. reopened W048 Excel-match iterative reference fixtures.

## 2. Code And Corpus Changes

| Surface | Path |
| --- | --- |
| TraceCalc machine diagnostic event support | `src/oxcalc-tracecalc/src/machine.rs` |
| TraceCalc replay-visible iteration trace step | `src/oxcalc-tracecalc/src/machine.rs` (`emit_iteration_trace`) |
| TraceCalc scenario validation for iteration trace step | `src/oxcalc-tracecalc/src/contracts.rs` |
| manifest | `docs/test-corpus/core-engine/tracecalc/MANIFEST.json` |
| structural self-cycle fixture | `docs/test-corpus/core-engine/tracecalc/hand-auditable/tc_w048_structural_self_cycle_reject_001.json` |
| CTRO candidate-cycle fixture | `docs/test-corpus/core-engine/tracecalc/hand-auditable/tc_w048_ctro_candidate_cycle_reject_001.json` |
| CTRO release/re-entry fixture | `docs/test-corpus/core-engine/tracecalc/hand-auditable/tc_w048_ctro_release_reentry_downstream_001.json` |
| Excel iterative two-node order fixture | `docs/test-corpus/core-engine/tracecalc/hand-auditable/tc_w048_excel_iter_two_node_order_001.json` |
| Excel iterative three-node order fixture | `docs/test-corpus/core-engine/tracecalc/hand-auditable/tc_w048_excel_iter_three_node_order_001.json` |
| Excel iterative fractional precision fixture | `docs/test-corpus/core-engine/tracecalc/hand-auditable/tc_w048_excel_iter_fraction_precision_001.json` |
| Excel iterative CTRO/INDIRECT self-cycle fixture | `docs/test-corpus/core-engine/tracecalc/hand-auditable/tc_w048_excel_ctro_indirect_iterative_self_001.json` |
| iterative fixture checker | `scripts/check-w048-tracecalc-iterative-cycles.ps1` |

## 3. Evidence Runs

Predecessor non-iterative command:

```powershell
cargo run -p oxcalc-tracecalc-cli -- w048-tracecalc-cycles-003
```

Predecessor run root: `docs/test-runs/core-engine/tracecalc-reference-machine/w048-tracecalc-cycles-003/`

Reopened iterative command:

```powershell
cargo run -p oxcalc-tracecalc-cli -- w048-tracecalc-cycles-004
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/check-w048-tracecalc-iterative-cycles.ps1
```

Reopened run root: `docs/test-runs/core-engine/tracecalc-reference-machine/w048-tracecalc-cycles-004/`

Summary from `w048-tracecalc-cycles-004/run_summary.json`:

| Field | Value |
| --- | ---: |
| scenario_count | 38 |
| passed | 38 |

W048 scenario results:

| Scenario | Result | Key evidence |
| --- | --- | --- |
| `tc_w048_structural_self_cycle_reject_001` | passed | self-loop cycle region detected, `synthetic_cycle_reject`, prior published value retained |
| `tc_w048_ctro_candidate_cycle_reject_001` | passed | candidate-overlay cycle diagnostic, candidate overlay commit suppressed, no new cycle values published |
| `tc_w048_ctro_release_reentry_downstream_001` | passed | first candidate cycle rejected, downstream blocked diagnostic emitted, rejected work re-entered, release candidate published `A=11`, downstream `D=12` |
| `tc_w048_excel_iter_two_node_order_001` | passed | Excel-match iterative trace emitted; publishes `A1=11`, `B1=22` for `excel_iter_two_node_order_001` |
| `tc_w048_excel_iter_three_node_order_001` | passed | Excel-match iterative trace emitted; publishes `A1=102`, `B1=101`, `C1=103` for chain `C1,B1,A1` |
| `tc_w048_excel_iter_fraction_precision_001` | passed | Excel-match iterative trace emitted; publishes `A1=0.33333333333333331` |
| `tc_w048_excel_ctro_indirect_iterative_self_001` | passed | Excel-match dynamic-reference iterative trace emitted; publishes `A1=1`, `B1=A1` |

## 4. Reference Policy Expressed

This TraceCalc slice expresses W048 behavior as follows:

1. Structural cycle regions route to `synthetic_cycle_reject` under the non-iterative Stage 1 profile.
2. Rejected cycle candidates publish no new cycle-region values.
3. Candidate-overlay cycles emit explicit diagnostics and suppress candidate overlay commit.
4. Rejected cycle work can re-enter on a later acyclic candidate.
5. Release/re-entry invalidates/recomputes downstream dependents in the reference trace.
6. Excel-match iterative reference behavior is replay-visible through `cycle_iteration_trace` events and `cycle.iteration_trace_events` counters.
7. The declared TraceCalc iterative coverage binds to the falsification fixtures from `W048_ITERATIVE_PROFILE_DECISION.json`; remaining Excel blockers are preserved for later implementation/conformance beads rather than hidden.

## 5. Review Checks

Post-change review checks for this bead:

```powershell
cargo run -p oxcalc-tracecalc-cli -- w048-tracecalc-cycles-004
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/check-w048-tracecalc-iterative-cycles.ps1
cargo test -p oxcalc-tracecalc
cargo fmt --all -- --check
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/check-worksets.ps1
```

## 6. Limits

This is a reference-machine and fixture slice, not an optimized TreeCalc/core behavior claim. CTRO-created cycle detection and iterative Excel-match behavior are represented through explicit reference diagnostics, iteration trace events, and fixture expected values. The optimized/core implementation and native sidecar conformance remain routed to `calc-zci1.14` and `calc-zci1.15`.

## 7. Fresh-Eyes Review For `calc-zci1.13`

Review date: 2026-05-11

Review questions:

1. Does TraceCalc expose replay-visible iteration traces, not only final values?
2. Do fixtures bind to concrete Excel observation packet surfaces?
3. Does this accidentally claim TreeCalc/core implementation?
4. Are weak Excel areas still carried forward as blockers?

Findings:

1. `emit_iteration_trace` emits `cycle_iteration_trace` events with payload fields for probe id, member order, initial vector, terminal vector, max iterations, max change, and terminal state; each fixture also increments `cycle.iteration_trace_events`.
2. The four new fixtures correspond to the falsification fixtures in `W048_ITERATIVE_PROFILE_DECISION.json`.
3. This packet is explicitly limited to TraceCalc reference behavior; TreeCalc remains open.
4. Root/report-cell, initial-vector edge cases, nonnumeric/error prior values, version, and multithread blockers remain in the profile document.

Fresh-eyes result: `calc-zci1.13` has TraceCalc reference fixtures and replay-visible traces for declared Excel-match iterative coverage. W048 remains in-progress for optimized/core implementation and conformance.

## 8. Three-Axis Status

- scope_completeness: `scope_partial`
- target_completeness: `target_partial`
- integration_completeness: `partial`
- open_lanes:
  - TreeCalc optimized/core implementation;
  - conformance integration;
  - final blocker disposition before broad Excel-match closure.
