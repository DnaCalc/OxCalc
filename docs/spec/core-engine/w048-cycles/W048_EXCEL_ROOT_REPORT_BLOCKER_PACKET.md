# W048 Excel Root/Report-Cell Evidence Packet

Status: `cleared_for_documented_worksheet_circular_reference_surface`

Parent bead: `calc-zci1.16`

Former blocker: `BLK-W048-EXCEL-ROOT`

## 1. Purpose

This packet records the fresh-eyes repair of W048's Excel circular-reference report-cell/root probe.

The first packet (`w048-excel-root-report-001`) queried `Application.CircularReference` and returned null for all variants. A fresh-eyes pass found that the documented public Excel object-model surface is worksheet-scoped: `Worksheet.CircularReference`. The runner was repaired to probe both surfaces and generated `w048-excel-root-report-002`.

Result: `Worksheet.CircularReference` returns report-cell/root addresses for the declared non-iterative circular-reference probes. `Application.CircularReference` remains null in this environment and is retained only as historical negative evidence.

## 2. Evidence Packet

| Field | Value |
| --- | --- |
| run_id | `w048-excel-root-report-002` |
| artifact root | `docs/test-runs/excel-cycles/w048-excel-root-report-002/` |
| runner | `scripts/run-w048-excel-root-report-probes.ps1` |
| checker | `scripts/check-w048-excel-root-report-probes.ps1` |
| schema | `oxcalc.w048.excel_root_report_probe.v2` |
| Excel version/build | recorded in `environment.json` (`16.0` / `19929` in local run) |
| probe count | 5 |

Probe variants:

1. direct self-cycle, iteration disabled;
2. direct self-cycle, iteration enabled;
3. two-node cycle, A-then-B edit order;
4. two-node cycle, B-then-A edit order;
5. three-node cycle.

Each variant records both:

- `application`: `Application.CircularReference`;
- `worksheet`: `Worksheet.CircularReference`.

The selected report surface is `worksheet`, because it is the documented object-model route and it produced non-null report cells for non-iterative circular-reference detection.

## 3. Observed Result

Validation command:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/check-w048-excel-root-report-probes.ps1 docs/test-runs/excel-cycles/w048-excel-root-report-002
```

Observed result:

```text
w048 excel root/report probe packet ok: run=w048-excel-root-report-002 observations=5 status=observed_worksheet_circular_reference_reports
```

Summary:

| Probe | Worksheet reported addresses | Application reported addresses | Disposition |
| --- | --- | --- | --- |
| `root_self_no_iteration` | `Sheet1!A1` | none | report surface observed |
| `root_self_iteration` | none | none | iteration-enabled circular reference is admitted, so no report cell was surfaced |
| `root_two_node_ab` | `Sheet1!B1`, `Sheet1!A1` | none | report surface observed; address varies by moment/command |
| `root_two_node_ba` | `Sheet1!A1`, `Sheet1!B1` | none | report surface observed; address varies by moment/command |
| `root_three_node` | `Sheet1!C1`, `Sheet1!B1`, `Sheet1!A1` | none | report surface observed; address varies by moment/command |

## 4. Fresh-Eyes Review For `calc-zci1.16`

Review date: 2026-05-12

Review questions:

1. Did the prior blocker depend on a wrong or incomplete object-model surface?
2. Does the repaired probe use a public Excel object-model route?
3. Does the result unblock W048's report-cell/root requirement?
4. Are caveats explicit enough to prevent broad overclaiming?

Findings:

1. Yes. The original packet queried `Application.CircularReference`; the documented public surface is `Worksheet.CircularReference`.
2. Yes. `Worksheet.CircularReference` is public COM-visible object-model behavior and returned ranges in the local host.
3. Yes for declared non-iterative circular-reference report-cell/root evidence. `calc-zci1.16` can close because the exact unblock path requested either UI evidence or another public object-model surface that does not return null; the worksheet surface satisfies that path for declared probes.
4. Caveats are explicit: `Application.CircularReference` remains null; iteration-enabled self-cycle does not surface a report cell; multi-node report addresses can vary by moment/command and should be treated as observed report-surface behavior rather than a universal canonical root policy.

Fresh-eyes result: `calc-zci1.16` is cleared by `w048-excel-root-report-002`. Remaining broad Excel-compatibility closure is still blocked by `BLK-W048-EXCEL-VERSION`.

## 5. Three-Axis Status

- scope_completeness: `scope_partial`
- target_completeness: `target_partial`
- integration_completeness: `partial`
- open_lanes:
  - update W048 profile/conformance surfaces to reference worksheet-scoped root/report evidence;
  - second Excel host/version repeat under `BLK-W048-EXCEL-VERSION`.
