# W048 TreeCalc Optimized Cycle Behavior

Status: `active_execution_evidence`

## 1. Purpose

This packet records the W048 TreeCalc optimized/core behavior slice for circular dependency processing. It binds the Stage 1 non-iterative cycle policy to checked TreeCalc fixtures, local run artifacts, materialized graph sidecars, and Rust tests.

## 2. Code And Corpus Changes

| Surface | Path |
| --- | --- |
| dynamic dependency invalidation reason support | `src/oxcalc-core/src/dependency.rs` |
| TreeCalc local runtime CTRO delta and seeded runtime-effect handling | `src/oxcalc-core/src/treecalc.rs` |
| TreeCalc fixture post-edit published runtime-effect seeding | `src/oxcalc-core/src/treecalc_fixture.rs` |
| TreeCalc runner fixture-count expectations | `src/oxcalc-core/src/treecalc_runner.rs` |
| manifest | `docs/test-fixtures/core-engine/treecalc/MANIFEST.json` |
| structural self-cycle fixture | `docs/test-fixtures/core-engine/treecalc/cases/tc_w048_structural_self_cycle_reject_001.json` |
| structural two-node SCC fixture | `docs/test-fixtures/core-engine/treecalc/cases/tc_w048_structural_two_node_cycle_reject_001.json` |
| CTRO dynamic self-cycle reject fixture | `docs/test-fixtures/core-engine/treecalc/cases/tc_w048_ctro_dynamic_self_cycle_reject_001.json` |
| CTRO release/re-entry/downstream fixture | `docs/test-fixtures/core-engine/treecalc/cases/tc_w048_ctro_dynamic_release_reentry_downstream_001.json` |

## 3. Evidence Run

Command:

```powershell
cargo run -p oxcalc-tracecalc-cli -- treecalc w048-treecalc-cycles-001
python scripts/check-w048-materialized-graphs.py docs/test-runs/core-engine/treecalc-local/w048-treecalc-cycles-001
```

Run root: `docs/test-runs/core-engine/treecalc-local/w048-treecalc-cycles-001/`

Summary from `run_summary.json`:

| Field | Value |
| --- | ---: |
| case_count | 33 |
| expectation_mismatch_count | 0 |
| published | 18 |
| rejected | 14 |
| verified_clean | 1 |

Summary from `w048_materialized_graph_check_summary.json`:

| Field | Value |
| --- | ---: |
| case_count | 33 |
| graph layers | 99 |
| cycle region records | 12 |
| check errors | 0 |

W048 fixture results:

| Fixture | Initial result | Post-edit result | Key evidence |
| --- | --- | --- | --- |
| `tc_w048_structural_self_cycle_reject_001` | rejected | n/a | self-loop cycle region, `SyntheticCycleReject`, prior value retained |
| `tc_w048_structural_two_node_cycle_reject_001` | rejected | n/a | two-node SCC, `SyntheticCycleReject`, prior values retained |
| `tc_w048_ctro_dynamic_self_cycle_reject_001` | rejected | n/a | dynamic resolved self-edge rejected with no publication |
| `tc_w048_ctro_dynamic_release_reentry_downstream_001` | rejected | published | dynamic self-cycle initially rejected; later acyclic dynamic target publishes owner and downstream dependent |

## 4. Behavior Expressed

This TreeCalc slice expresses W048 Stage 1 non-iterative behavior as follows:

1. Structural formula cycles are rejected as `SyntheticCycleReject`.
2. CTRO-created dynamic self-cycles route through the same cycle rejection policy.
3. Rejected cycle candidates publish no new values and carry no publication bundle.
4. Prior published values remain visible after rejection.
5. A later release from dynamic self-cycle to acyclic target can re-enter calculation and publish atomically.
6. Downstream dependents of released dynamic owner recompute through ordinary dependency order.
7. Materialized graph sidecars expose non-empty cycle-region records for W048 TreeCalc cycle cases.

## 5. Review Checks

Fresh-eye review checks used before bead closure:

```powershell
cargo run -p oxcalc-tracecalc-cli -- treecalc w048-treecalc-cycles-001
python scripts/check-w048-materialized-graphs.py docs/test-runs/core-engine/treecalc-local/w048-treecalc-cycles-001
python - <<'PY'
import json, pathlib
root = pathlib.Path('docs/test-runs/core-engine/treecalc-local/w048-treecalc-cycles-001')
summary = json.load(open(root / 'run_summary.json', encoding='utf-8'))
assert summary['case_count'] == 33
assert summary['expectation_mismatch_count'] == 0
mat = json.load(open(root / 'w048_materialized_graph_check_summary.json', encoding='utf-8'))
assert mat['case_count'] == 33
assert mat['layer_count'] == 99
assert mat['cycle_region_count'] == 12
assert mat['check_error_count'] == 0
print('w048 treecalc review ok')
PY
cargo test -p oxcalc-core
cargo fmt --all -- --check
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/check-worksets.ps1
```

## 6. Limits

This is not an iterative-calculation implementation. Iterative profile selection remains routed to `calc-zci1.4`. The current CTRO-created cycle fixture uses a resolved dynamic self-edge as the candidate-effective graph witness; broader dynamic-array/spill and data-table cycle behavior remain out of this bead's scope.
