# W048 Excel Nonnumeric Prior Packet

Status: `blank_text_error_prior_observed`

Parent bead: `calc-zci1.18`

Former blocker: `BLK-W048-EXCEL-NONNUMERIC`

## 1. Purpose

This packet records targeted black-box Excel observations for iterative self-cycle behavior after blank, text, and error prior states. It replaces the ambiguous predecessor text-prior observation where formula entry was accidentally preserved as text.

## 2. Evidence Packet

| Field | Value |
| --- | --- |
| run_id | `w048-excel-nonnumeric-prior-001` |
| artifact root | `docs/test-runs/excel-cycles/w048-excel-nonnumeric-prior-001/` |
| runner | `scripts/run-w048-excel-nonnumeric-prior-probes.ps1` |
| checker | `scripts/check-w048-excel-nonnumeric-prior-probes.ps1` |
| probe count | 8 |

Prior states:

1. blank;
2. text;
3. `#N/A` via `=NA()`;
4. division error via `=1/0`.

Formulas:

1. `=A1+1`;
2. `=A1/2`.

## 3. Observed Result

Validation command:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/check-w048-excel-nonnumeric-prior-probes.ps1
```

Observed result:

```text
w048 excel nonnumeric-prior probe packet ok: run=w048-excel-nonnumeric-prior-001 observations=8
```

Summary:

| Prior | Formula | After formula assignment | Final after calculate (`MaxIterations=5`) |
| --- | --- | ---: | ---: |
| blank | `=A1+1` | 1 | 6 |
| blank | `=A1/2` | 0 | 0 |
| text | `=A1+1` | 1 | 6 |
| text | `=A1/2` | 0 | 0 |
| `#N/A` | `=A1+1` | 1 | 6 |
| `#N/A` | `=A1/2` | 0 | 0 |
| `#DIV/0!` | `=A1+1` | 1 | 6 |
| `#DIV/0!` | `=A1/2` | 0 | 0 |

Interpretation for W048 declared coverage:

1. Blank, text, and error prior states did not survive self-cycle formula assignment in the declared probes.
2. Assignment values match a zero/blank-derived base (`=A1+1 => 1`, `=A1/2 => 0`).
3. Subsequent calculation advances `=A1+1` to `6` with `MaxIterations=5` and leaves `=A1/2` at `0`.
4. `BLK-W048-EXCEL-NONNUMERIC` is unblocked for declared self-cycle prior-state coverage.

## 4. Fresh-Eyes Review For `calc-zci1.18`

Review date: 2026-05-11

Review questions:

1. Did the new packet avoid the prior formula-as-text ambiguity?
2. Did it cover blank, text, and error prior states?
3. Did the checker assert both assignment and final values?
4. Does this overclaim all Excel error/coercion semantics?

Findings:

1. Yes. The runner resets `NumberFormat` to `General` before formula assignment and records both assignment and post-calculate snapshots.
2. Yes. It covers blank, text, `#N/A`, and division-error prior states.
3. Yes. The checker requires assignment values and final values for increment and decay formulas.
4. No broad coercion claim is made. The claim is scoped to declared self-cycle prior-state probes.

Fresh-eyes result: `calc-zci1.18` unblocks blank/text/error prior-state behavior for declared W048 self-cycle coverage.

## 5. Three-Axis Status

- scope_completeness: `scope_partial`
- target_completeness: `target_partial`
- integration_completeness: `partial`
- open_lanes:
  - root/report-cell behavior;
  - cross-version repeat;
  - multithread variants.
