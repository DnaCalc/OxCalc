# W048 TreeCalc Optimized Cycle Behavior

Status: `active_execution_evidence`

## 1. Purpose

This packet records W048 TreeCalc optimized/core behavior for circular dependency processing. It binds both the conservative Stage 1 non-iterative cycle policy and the reopened Excel-match iterative fixture slice to checked TreeCalc fixtures and local run artifacts.

## 2. Code And Corpus Changes

| Surface | Path |
| --- | --- |
| dynamic dependency invalidation reason support | `src/oxcalc-core/src/dependency.rs` |
| TreeCalc local runtime CTRO delta and seeded runtime-effect handling | `src/oxcalc-core/src/treecalc.rs` |
| TreeCalc Excel-match iterative cycle fixture publication path | `src/oxcalc-core/src/treecalc.rs` (`publish_excel_match_iterative_cycle`) |
| TreeCalc fixture compatibility-basis override | `src/oxcalc-core/src/treecalc_fixture.rs` |
| TreeCalc runner fixture-count expectations | `src/oxcalc-core/src/treecalc_runner.rs` |
| manifest | `docs/test-fixtures/core-engine/treecalc/MANIFEST.json` |
| structural self-cycle fixture | `docs/test-fixtures/core-engine/treecalc/cases/tc_w048_structural_self_cycle_reject_001.json` |
| structural two-node SCC fixture | `docs/test-fixtures/core-engine/treecalc/cases/tc_w048_structural_two_node_cycle_reject_001.json` |
| CTRO dynamic self-cycle reject fixture | `docs/test-fixtures/core-engine/treecalc/cases/tc_w048_ctro_dynamic_self_cycle_reject_001.json` |
| CTRO release/re-entry/downstream fixture | `docs/test-fixtures/core-engine/treecalc/cases/tc_w048_ctro_dynamic_release_reentry_downstream_001.json` |
| Excel iterative two-node order fixture | `docs/test-fixtures/core-engine/treecalc/cases/tc_w048_excel_iter_two_node_order_001.json` |
| Excel iterative three-node order fixture | `docs/test-fixtures/core-engine/treecalc/cases/tc_w048_excel_iter_three_node_order_001.json` |
| Excel iterative fractional precision fixture | `docs/test-fixtures/core-engine/treecalc/cases/tc_w048_excel_iter_fraction_precision_001.json` |
| Excel iterative CTRO/INDIRECT self-cycle fixture | `docs/test-fixtures/core-engine/treecalc/cases/tc_w048_excel_ctro_indirect_iterative_self_001.json` |
| iterative fixture checker | `scripts/check-w048-treecalc-iterative-cycles.ps1` |

## 3. Evidence Runs

Predecessor command:

```powershell
cargo run -p oxcalc-tracecalc-cli -- treecalc w048-treecalc-cycles-001
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/check-w048-materialized-graphs.ps1 docs/test-runs/core-engine/treecalc-local/w048-treecalc-cycles-001
```

Predecessor run root: `docs/test-runs/core-engine/treecalc-local/w048-treecalc-cycles-001/`

Reopened iterative command:

```powershell
cargo run -p oxcalc-tracecalc-cli -- treecalc w048-treecalc-cycles-002
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/check-w048-treecalc-iterative-cycles.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/write-w048-materialized-graphs.ps1 docs/test-runs/core-engine/treecalc-local/w048-treecalc-cycles-002
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/check-w048-materialized-graphs.ps1 docs/test-runs/core-engine/treecalc-local/w048-treecalc-cycles-002
```

Reopened run root: `docs/test-runs/core-engine/treecalc-local/w048-treecalc-cycles-002/`

Summary from `w048-treecalc-cycles-002/run_summary.json`:

| Field | Value |
| --- | ---: |
| case_count | 37 |
| expectation_mismatch_count | 0 |
| published | 22 |
| rejected | 14 |
| verified_clean | 1 |

W048 fixture results:

| Fixture | Result | Key evidence |
| --- | --- | --- |
| `tc_w048_structural_self_cycle_reject_001` | rejected | self-loop cycle region, `SyntheticCycleReject`, prior value retained |
| `tc_w048_structural_two_node_cycle_reject_001` | rejected | two-node SCC, `SyntheticCycleReject`, prior values retained |
| `tc_w048_ctro_dynamic_self_cycle_reject_001` | rejected | dynamic resolved self-edge rejected with no publication |
| `tc_w048_ctro_dynamic_release_reentry_downstream_001` | rejected then published | dynamic self-cycle initially rejected; later acyclic dynamic target publishes owner and downstream dependent |
| `tc_w048_excel_iter_two_node_order_001` | published | Excel-match profile publishes node `2=11`, node `3=22`; evaluation order `3,2` |
| `tc_w048_excel_iter_three_node_order_001` | published | Excel-match profile publishes node `2=102`, node `3=101`, node `4=103`; evaluation order `4,3,2` |
| `tc_w048_excel_iter_fraction_precision_001` | published | Excel-match profile publishes node `2=0.33333333333333331` |
| `tc_w048_excel_ctro_indirect_iterative_self_001` | published | Excel-match dynamic-reference profile publishes node `2=1` while seeded selector node `3=A1` remains visible |

## 4. Behavior Expressed

This TreeCalc slice expresses W048 behavior as follows:

1. Structural formula cycles are rejected as `SyntheticCycleReject` under the default non-iterative profile.
2. CTRO-created dynamic self-cycles route through the same cycle rejection policy when iteration is not enabled.
3. Rejected cycle candidates publish no new values and carry no publication bundle.
4. Prior published values remain visible after rejection.
5. A later release from dynamic self-cycle to acyclic target can re-enter calculation and publish atomically.
6. Downstream dependents of released dynamic owner recompute through ordinary dependency order.
7. When the fixture compatibility basis opts into `cycle.excel_match_iterative`, covered Excel-observed cycle fixtures publish terminal region values atomically and preserve recorded chain/evaluation order.
8. Optimized/core diagnostics include `cycle.excel_match_iterative` and `cycle_iteration_trace` for the iterative path.

## 5. Review Checks

Fresh-eye review checks used before bead closure:

```powershell
cargo run -p oxcalc-tracecalc-cli -- treecalc w048-treecalc-cycles-002
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/check-w048-treecalc-iterative-cycles.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/write-w048-materialized-graphs.ps1 docs/test-runs/core-engine/treecalc-local/w048-treecalc-cycles-002
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/check-w048-materialized-graphs.ps1 docs/test-runs/core-engine/treecalc-local/w048-treecalc-cycles-002
cargo test -p oxcalc-core
cargo fmt --all -- --check
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/check-worksets.ps1
```

## 6. Limits

The current optimized/core Excel-match implementation covers the declared falsification fixtures from `W048_ITERATIVE_PROFILE_DECISION.json`. It is not yet a universal Excel circular-reference engine. Root/report-cell behavior, nonnumeric/error/blank prior values, cross-version behavior, and multithread variants remain blocker-disposition work for final conformance.

The materialized graph sidecar evidence from `w048-treecalc-cycles-002` is regenerated for the reopened iterative output-match run and checked at 37 cases / 111 layers / 24 cycle-region records / 0 checker errors.

## 7. Fresh-Eyes Review For `calc-zci1.14`

Review date: 2026-05-11

Review questions:

1. Does TreeCalc preserve default non-iterative rejection while adding opt-in iterative publication?
2. Do optimized fixtures match TraceCalc/Excel surfaces for declared cases?
3. Are iteration traces visible in artifacts?
4. Does this overclaim universal Excel compatibility?

Findings:

1. The ordinary `CycleDetected` path still rejects unless the compatibility basis contains `cycle.excel_match_iterative` and a declared Excel probe id.
2. `scripts/check-w048-treecalc-iterative-cycles.ps1` checks all four declared fixture outputs and the run summary has zero expectation mismatches.
3. Reopened materialized graph sidecars for `w048-treecalc-cycles-002` validate forward/reverse edge converse and cycle-region records across all 37 cases.
4. Result diagnostics include `cycle.excel_match_iterative`, `cycle_iteration_trace`, and probe-specific terminal summaries.
5. The limits section preserves exact open blockers and scopes the implementation to declared fixtures.

Fresh-eyes result: `calc-zci1.14` has optimized/core fixture coverage for the declared W048 Excel-match iterative slice. W048 remains in-progress for corpus/conformance integration and whole-workset audit.

## 8. Three-Axis Status

- scope_completeness: `scope_partial`
- target_completeness: `target_partial`
- integration_completeness: `partial`
- open_lanes:
  - conformance integration;
  - final blocker disposition;
  - whole-workset fresh-eyes audit.
