# W048 Excel Initial-Vector Packet

Status: `numeric_prior_initial_vector_observed`

Parent bead: `calc-zci1.17`

Former blocker: `BLK-W048-EXCEL-INITIAL`

## 1. Purpose

This packet records targeted black-box Excel observations for iterative self-cycle initial-vector behavior with prior numeric values. It addresses whether prior numeric cell values survive formula assignment or whether the iterative cycle starts from a zero/blank-derived base.

## 2. Evidence Packet

| Field | Value |
| --- | --- |
| run_id | `w048-excel-initial-vector-001` |
| artifact root | `docs/test-runs/excel-cycles/w048-excel-initial-vector-001/` |
| runner | `scripts/run-w048-excel-initial-vector-probes.ps1` |
| checker | `scripts/check-w048-excel-initial-vector-probes.ps1` |
| probe count | 8 |

Probe variants seed `A1` with a numeric value, enable iteration, assign a self-referential formula, and compare the immediate formula-assignment result with subsequent calculate/full-rebuild/save-reopen results.

## 3. Observed Result

Validation command:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/check-w048-excel-initial-vector-probes.ps1
```

Observed result:

```text
w048 excel initial-vector probe packet ok: run=w048-excel-initial-vector-001 observations=8
```

Summary:

| Formula | Seed | Command | After formula assignment | Final observed value |
| --- | ---: | --- | ---: | ---: |
| `=A1+1` | 5 | none | 1 | 1 |
| `=A1/2` | 8 | none | 0 | 0 |
| `=A1+1` | 5 | calculate | 1 | 11 |
| `=A1/2` | 8 | calculate | 0 | 0 |
| `=A1+1` | 5 | full rebuild | 1 | 11 |
| `=A1/2` | 8 | full rebuild | 0 | 0 |
| `=A1+1` | 5 | save/reopen | 1 | 11 |
| `=A1/2` | 8 | save/reopen | 0 | 0 |

Interpretation for W048 declared coverage:

1. Prior numeric seeds did not survive formula assignment for these self-cycle probes.
2. Formula assignment produced values matching a zero/blank-derived base (`=A1+1 => 1`, `=A1/2 => 0`).
3. Subsequent calculation for `=A1+1` advanced from the assignment result to `11` with `MaxIterations=10`; this is consistent with iterative advancement from the observed post-assignment value rather than from the prior seed `5`.
4. The numeric-prior portion of `BLK-W048-EXCEL-INITIAL` is unblocked for declared self-cycle coverage.

## 4. Fresh-Eyes Review For `calc-zci1.17`

Review date: 2026-05-11

Review questions:

1. Did the packet distinguish prior numeric value from formula-assignment/reset behavior?
2. Were both increment and decay formulas tested?
3. Did the checker assert the surprising command-sensitive result rather than a stale expectation?
4. Does this settle blank/text/error prior behavior?

Findings:

1. Yes. Seeds `5` and `8` were recorded before formula assignment, and post-assignment values were independent of those seeds.
2. Yes. `=A1+1` and `=A1/2` were both tested.
3. Yes. The checker requires `=A1+1` to be `1` immediately after formula assignment and `11` after calculate/full-rebuild/save-reopen with `MaxIterations=10`.
4. No. Blank/text/error prior behavior remains under `BLK-W048-EXCEL-NONNUMERIC`.

Fresh-eyes result: `calc-zci1.17` unblocks numeric-prior initial-vector behavior for the declared W048 self-cycle coverage. It does not close nonnumeric prior behavior.

## 5. Three-Axis Status

- scope_completeness: `scope_partial`
- target_completeness: `target_partial`
- integration_completeness: `partial`
- open_lanes:
  - root/report-cell behavior;
  - blank/text/error prior values;
  - cross-version repeat;
  - multithread variants.
