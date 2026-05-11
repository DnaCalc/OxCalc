# W048 Excel Root/Report-Cell Blocker Packet

Status: `blocked_waiting_for_ui_or_alternate_public_surface`

Parent bead: `calc-zci1.16`

Blocker: `BLK-W048-EXCEL-ROOT`

## 1. Purpose

This packet records a targeted attempt to unblock Excel circular-reference report-cell/root behavior using a public Excel COM object-model surface: `Application.CircularReference`.

The attempt did not unblock the behavior. The blocker remains open because all observed variants returned null for `Application.CircularReference`.

## 2. Evidence Packet

| Field | Value |
| --- | --- |
| run_id | `w048-excel-root-report-001` |
| artifact root | `docs/test-runs/excel-cycles/w048-excel-root-report-001/` |
| runner | `scripts/run-w048-excel-root-report-probes.ps1` |
| checker | `scripts/check-w048-excel-root-report-probes.ps1` |
| Excel version/build | recorded in `environment.json` |
| probe count | 5 |

Probe variants:

1. direct self-cycle, iteration disabled;
2. direct self-cycle, iteration enabled;
3. two-node cycle, A-then-B edit order;
4. two-node cycle, B-then-A edit order;
5. three-node cycle.

Each variant recorded `Application.CircularReference` at initial state, after formula edits, and after calculation commands (`Worksheet.Calculate`, `Application.Calculate`, `CalculateFull`, `CalculateFullRebuild`).

## 3. Observed Result

Validation command:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/check-w048-excel-root-report-probes.ps1
```

Observed result:

```text
w048 excel root/report probe packet ok: run=w048-excel-root-report-001 observations=5 status=observed_object_model_null_for_all_variants
```

Summary:

| Probe | Reported addresses | Disposition |
| --- | --- | --- |
| `root_self_no_iteration` | none | null for all moments |
| `root_self_iteration` | none | null for all moments |
| `root_two_node_ab` | none | null for all moments |
| `root_two_node_ba` | none | null for all moments |
| `root_three_node` | none | null for all moments |

## 4. Fresh-Eyes Review For `calc-zci1.16`

Review date: 2026-05-11

Review questions:

1. Did the targeted packet actually probe report-cell/root behavior rather than reusing only calc-chain metadata?
2. Did it vary enough factors to avoid a trivial false negative?
3. Does the result unblock W048's report-cell/root requirement?
4. Is the remaining work exactly stated?

Findings:

1. The packet directly queried `Application.CircularReference` after edits and after four calculation commands.
2. The packet varied self/two-node/three-node cycles, edit order, and iteration enabled/disabled.
3. The result does not unblock W048: every `Application.CircularReference` observation was null.
4. The exact remaining unblock path is UI warning capture or another public object-model surface that returns a report cell/root in this environment.

Fresh-eyes result: `calc-zci1.16` remains blocked. This is useful evidence, but not an implementation or closure of report-cell/root behavior.

## 5. Three-Axis Status

- scope_completeness: `scope_partial`
- target_completeness: `target_partial`
- integration_completeness: `partial`
- open_lanes:
  - UI warning capture for circular-reference report-cell/root;
  - alternate public object-model route for report-cell/root;
  - profile update after root/report evidence is available.
