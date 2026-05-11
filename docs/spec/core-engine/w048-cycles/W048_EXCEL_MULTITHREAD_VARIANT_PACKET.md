# W048 Excel Multithread Variant Packet

Status: `multithread_variants_observed`

Parent bead: `calc-zci1.20`

Former blocker: `BLK-W048-EXCEL-MT`

## 1. Purpose

This packet records targeted black-box Excel observations for multithreaded calculation variants of the declared W048 falsification fixtures.

## 2. Evidence Packet

| Field | Value |
| --- | --- |
| run_id | `w048-excel-multithread-variant-001` |
| artifact root | `docs/test-runs/excel-cycles/w048-excel-multithread-variant-001/` |
| runner | `scripts/run-w048-excel-multithread-variant-probes.ps1` |
| checker | `scripts/check-w048-excel-multithread-variant-probes.ps1` |
| probe count | 4 |
| thread count | recorded in `environment.json`; observed checker run used 16 |

## 3. Observed Result

Validation command:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/check-w048-excel-multithread-variant-probes.ps1
```

Observed result:

```text
w048 excel multithread variant packet ok: run=w048-excel-multithread-variant-001 observations=4 threads=16
```

Summary:

| Probe | Multithread observed values |
| --- | --- |
| `excel_iter_two_node_order_001` | `A1=3`, `B1=6` |
| `excel_iter_three_node_order_001` | `A1=5`, `B1=107`, `C1=6` |
| `excel_iter_fraction_precision_001` | `A1=0.49999989546242096` |
| `excel_ctro_indirect_iterative_self_001` | `A1=10`, `B1=A1` |

## 4. Interpretation

The multithread variant packet does not match the prior single-threaded falsification fixture surfaces. W048 therefore must keep thread-mode as an explicit Excel-match profile dimension:

1. `cycle.excel_match_iterative.single_thread_observed` remains the implemented declared fixture profile for TraceCalc/TreeCalc.
2. `cycle.excel_match_iterative.multithread_observed` is now an observed variant surface, not yet implemented in TraceCalc/TreeCalc.
3. Broad Excel compatibility must either implement/profile both thread modes or explicitly scope the claim to the single-threaded observed profile.

## 5. Fresh-Eyes Review For `calc-zci1.20`

Review date: 2026-05-11

Review questions:

1. Did the packet actually enable multithreaded calculation?
2. Were the declared falsification fixtures covered?
3. Did the result match the single-threaded packet?
4. Does this close all thread-mode work?

Findings:

1. Yes. `environment.json` records multithreaded calculation enabled and thread count.
2. Yes. The four declared falsification fixture probes were covered.
3. No. Observed values differ from the single-threaded packet, so thread mode is semantically relevant.
4. No. The blocker to run variants is resolved, but implementation/profile work for multithread compatibility remains an open lane before broad Excel closure.

Fresh-eyes result: `calc-zci1.20` satisfies the requirement to run multithread variants, while preserving thread-mode as a profile dimension.

## 6. Three-Axis Status

- scope_completeness: `scope_partial`
- target_completeness: `target_partial`
- integration_completeness: `partial`
- open_lanes:
  - root/report-cell behavior;
  - second Excel host/version repeat;
  - optional future implementation of `cycle.excel_match_iterative.multithread_observed` if W048 broad closure includes multithread mode.
