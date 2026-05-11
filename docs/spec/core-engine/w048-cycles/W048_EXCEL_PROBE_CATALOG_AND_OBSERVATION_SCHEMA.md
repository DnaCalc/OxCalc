# W048 Excel Probe Catalog And Observation Schema

Status: `active_execution_workset`

## 1. Purpose

Excel matching must be based on explicit observation packets. Public documentation establishes high-level behavior, but W048 needs probes for the details that can affect cycle outputs:

1. reported cycle cell/root;
2. calculation-chain sensitivity;
3. retained prior values;
4. initial vector for iteration;
5. update order;
6. max-change metric;
7. release/re-entry behavior;
8. downstream dependent state.

This document defines the probe catalog and observation schema. Normalized observation ledgers live in `W048_EXCEL_OBSERVATION_LEDGER.md` and currently point to:

1. `docs/test-runs/excel-cycles/w048-excel-cycles-001/` — first 12-probe core packet;
2. `docs/test-runs/excel-cycles/w048-excel-cycles-bitexact-001/` — expanded 19-probe bit-exact packet.

The W048 scope includes executing these probes where Excel is available, normalizing observations, and using them to drive OxCalc implementation and tests.

## 2. Clean-Room Rule

Allowed evidence:

1. public Microsoft documentation;
2. public workbook file metadata created by our probes;
3. black-box Excel observations through UI, COM/VBA object model, and saved workbook artifacts;
4. reproducible probe scripts and normalized observation JSON.

Disallowed evidence:

1. reverse-engineered Excel internals;
2. private implementation notes;
3. undocumented binary inspection that cannot be reproduced cleanly;
4. claims inferred from a single unrecorded manual observation.

## 3. Observation Packet Layout

Recommended artifact root:

`docs/test-runs/excel-cycles/<run_id>/`

Recommended files:

1. `environment.json`
2. `probe_plan.json`
3. `workbook_before.xlsx`
4. `workbook_after.xlsx`
5. `raw_com_log.jsonl`
6. `cell_snapshots.json`
7. `calc_chain_snapshot.json`
8. `observation.json`
9. `normalization_notes.md`

If Excel is unavailable in the current environment, the packet should record that blocker and keep the probe definitions executable for a Windows/Excel host.

## 4. Environment Schema

```json
{
  "excel_version": "16.0.x",
  "channel": "Current Channel",
  "platform": "Windows",
  "workbook_calculation_mode": "manual",
  "application_calculation_mode": "manual",
  "iteration_enabled": false,
  "max_iterations": 100,
  "max_change": 0.001,
  "multi_threaded_calculation_enabled": false,
  "thread_count": 1,
  "precision_as_displayed": false,
  "calculate_before_save": false,
  "locale": "en-US",
  "probe_runner": "powershell-com",
  "observation_time_utc": "2026-05-11T00:00:00Z"
}
```

Single-threaded mode is preferred for first observations so calculation-chain effects are easier to isolate. Multi-threaded variants remain necessary before an Excel-match claim.

## 5. Operation Step Schema

```json
{
  "step_index": 3,
  "operation": "set_formula",
  "target": "Sheet1!A1",
  "formula": "=B1+1",
  "before": {
    "value2": 0,
    "text": "0",
    "formula": ""
  },
  "after": {
    "value2": 0,
    "text": "0",
    "formula": "=B1+1"
  },
  "calculation_command": "Application.CalculateFullRebuild",
  "circular_reference": "Sheet1!A1",
  "calculation_state": "xlDone"
}
```

The runner should record:

1. formula edits in order;
2. value edits in order;
3. calculation command used;
4. workbook save/reopen actions;
5. `Application.CircularReference` where available;
6. relevant `Range.Value2`, `Range.Text`, and `Range.Formula`;
7. warning visibility if manually observed;
8. calculation-chain file metadata when saved workbook exposes it.

## 6. Normalized Observation Schema

```json
{
  "probe_id": "excel_struct_two_node_001",
  "run_id": "20260511-excel-cycles-001",
  "status": "observed",
  "cycle_kind": "structural",
  "iteration_profile": {
    "enabled": false,
    "max_iterations": 100,
    "max_change": 0.001
  },
  "reported_cycle_cells": ["Sheet1!A1"],
  "cell_results": [
    {
      "cell": "Sheet1!A1",
      "formula": "=B1+1",
      "value2": 0,
      "text": "0",
      "error": null
    }
  ],
  "chain_sensitive": "unknown",
  "root_hypothesis": "calculation_chain_root",
  "update_model_hypothesis": "unknown",
  "initial_value_hypothesis": "published_prior_value",
  "notes": []
}
```

Allowed `status` values:

1. `defined`;
2. `observed`;
3. `blocked`;
4. `superseded`.

Allowed hypothesis confidence:

1. `observed_directly`;
2. `inferred_from_probe_pair`;
3. `plausible`;
4. `unknown`.

## 7. Probe Families

### 7.1 Non-Iterative Structural Probes

| Probe id | Workbook sketch | Question |
| --- | --- | --- |
| `excel_struct_self_001` | `A1 = A1 + 1` | direct self-cycle warning/report cell and displayed value |
| `excel_struct_self_prior_001` | seed `A1 = 5`, calculate, then set `A1 = A1 + 1` | prior-value retention versus zero/default |
| `excel_struct_two_node_001` | `A1 = B1 + 1`; `B1 = A1 + 1` | reported root/order and no-iteration values |
| `excel_struct_three_node_001` | `A1 = B1 + 1`; `B1 = C1 + 1`; `C1 = A1 + 1` | SCC report order and chain order |
| `excel_struct_guarded_activation_001` | `A1 = IF(B1=0,0,A1+1)`; toggle `B1` | last-successful behavior and warning timing |

### 7.2 Iterative Structural Probes

| Probe id | Workbook sketch | Question |
| --- | --- | --- |
| `excel_iter_self_increment_001` | `A1 = A1 + 1` | max-iteration terminal and starting value |
| `excel_iter_self_decay_001` | `A1 = A1 / 2` with prior seed | convergence and max-change interpretation |
| `excel_iter_two_node_order_001` | order-sensitive two-node cycle | Jacobi versus sequential update |
| `excel_iter_three_node_order_001` | three-node asymmetric formulas | member order/root effect |
| `excel_iter_oscillation_001` | formula alternates between two values | terminal state after oscillation |

### 7.3 Calculation Chain Sensitivity Probes

| Probe id | Workbook sketch | Question |
| --- | --- | --- |
| `excel_chain_edit_order_ab_001` | create `A1/B1` cycle by editing A then B | edit order effect |
| `excel_chain_edit_order_ba_001` | same cycle by editing B then A | compare root/order with AB |
| `excel_chain_cold_open_001` | save/reopen existing cycle | saved chain effect |
| `excel_chain_full_rebuild_001` | run full rebuild before observation | dependency-tree/chain rebuild effect |
| `excel_chain_full_rebuild_compare_001` | iterative two-node cycle, full rebuild then ordinary calculate | full rebuild versus ordinary calculate in saved chain state |
| `excel_chain_range_rowmajor_001` | use row-major range calculation where available | row-major command effect |

### 7.4 CTRO/Dynamic Reference Probes

| Probe id | Workbook sketch | Question |
| --- | --- | --- |
| `excel_ctro_indirect_self_001` | selector changes dynamic reference target to owner | dynamic self-cycle behavior |
| `excel_ctro_indirect_two_node_001` | two dynamic references form SCC | dynamic SCC root/order |
| `excel_ctro_indirect_release_001` | dynamic target moves from self-cycle to acyclic target | release/re-entry timing |
| `excel_ctro_downstream_001` | dynamic cycle has dependent `D1 = C1 + 1` | downstream retained/stale/recomputed value |
| `excel_ctro_volatile_selector_001` | volatile selector affects dynamic target | volatility plus dynamic cycle |
| `excel_ctro_indirect_iterative_self_001` | `INDIRECT` self-cycle with iteration enabled | dynamic-reference iterative profile input |

### 7.5 Region And Spill Probes

| Probe id | Workbook sketch | Question |
| --- | --- | --- |
| `excel_spill_cycle_001` | dynamic array spill overlaps or feeds own input | circular warning versus spill-specific error |
| `excel_spill_release_001` | spill cycle later shrinks/releases | release behavior |
| `excel_data_table_cycle_001` | data table result depends on table value | documented data-table exception boundary |

Data-table probes should be classified separately from ordinary formula cycles.

## 8. Probes That Distinguish Update Models

A useful order-sensitive probe should produce different values under Jacobi and sequential update.

Example:

1. seed `A1 = 1`, `B1 = 10`;
2. set `A1 = B1 + 1`;
3. set `B1 = A1 * 2`;
4. enable iteration with `MaxIterations = 1`;
5. compare result:
   - Jacobi snapshot from prior vector: `A1 = 11`, `B1 = 2`;
   - sequential A then B: `A1 = 11`, `B1 = 22`;
   - sequential B then A: `B1 = 2`, `A1 = 3`.

The exact formulas may need adjustment to avoid Excel warning interruptions, but this class of probe is required.

## 9. Root/Order Probes

Root/order should be probed by varying only one factor:

1. formula edit order;
2. calculation command;
3. sheet order;
4. range order;
5. save/reopen;
6. full rebuild;
7. multi-threading setting.

If a pair differs only in edit order and reports different cycle cells or iterative results, W048 should treat chain/edit history as part of the Excel-match profile.

## 10. Execution Packets

Run `w048-excel-cycles-001` executed a 12-probe core packet through `scripts/run-w048-excel-cycle-probes.ps1`. It covers structural self/two/three-node cycles, prior-value and guarded activation probes, first iterative order probes, edit-order calculation-chain probes, and INDIRECT/dynamic-reference CTRO analog release/downstream probes.

Evidence root: `docs/test-runs/excel-cycles/w048-excel-cycles-001/`.

Run `w048-excel-cycles-bitexact-001` executed the expanded 19-probe `bitexact` packet through the same PowerShell runner. It includes the 12 core probes plus convergence, three-node order, oscillation, non-numeric prior, fractional precision, full-rebuild compare, and iterative INDIRECT self-cycle probes.

Evidence root: `docs/test-runs/excel-cycles/w048-excel-cycles-bitexact-001/`.

Both packets are validated by:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/check-w048-excel-observation-packet.ps1 docs/test-runs/excel-cycles/w048-excel-cycles-001
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/check-w048-excel-observation-packet.ps1 docs/test-runs/excel-cycles/w048-excel-cycles-bitexact-001 -MinimumProbeCount 19
```

Interpretation status:

1. observed cell values and saved calculation-chain entries are recorded as black-box workbook evidence;
2. `Application.CircularReference` did not surface a reported root in these automation packets, so root/report-cell behavior remains inconclusive;
3. edit-order probes produced different saved chain entries and different visible values, so W048 must keep chain/order as an explicit Excel-compatible profile axis;
4. CTRO analog release/downstream probes give first visible targets for W048 release/re-entry fixtures, but do not change the conservative OxCalc no-publication/no-overlay-commit Stage 1 policy;
5. iterative probes now cover max-iteration, update order, convergence, oscillation, non-numeric prior state, fractional precision, full-rebuild comparison, and dynamic-reference self-cycles, but they are still observation inputs rather than an implementation/closure claim.

See `W048_EXCEL_OBSERVATION_LEDGER.md` for the normalized summary and caveats.

## 11. OxCalc Comparison Fields

Each Excel observation should map to OxCalc comparison fields:

1. `cycle_source`;
2. `cycle_region_members`;
3. `cycle_root`;
4. `member_order`;
5. `initial_value_policy`;
6. `update_model`;
7. `terminal_policy`;
8. `publication_policy`;
9. `downstream_state`;
10. `release_reentry_policy`.

If Excel behavior cannot be matched without hidden chain state, the W048 output should say so explicitly and choose an OxCalc profile:

1. `excel_observed_compat` when probe evidence supports a matching policy;
2. `excel_visible_noniterative_compat` for warning/retained-value surface only;
3. `oxcalc_deterministic_profile` when OxCalc chooses clearer deterministic behavior over unobserved Excel internals;
4. `deferred_excel_gap` when evidence is insufficient.

## 11. Status Surface

- execution_state: `in_progress`
- scope_completeness: `scope_partial`
- target_completeness: `target_partial`
- integration_completeness: `partial`
- open_lanes:
  - Excel-match iterative profile derivation from recorded packets
  - comparison against OxCalc graph/cycle artifacts
  - TraceCalc/TreeCalc implementation and conformance fixtures
