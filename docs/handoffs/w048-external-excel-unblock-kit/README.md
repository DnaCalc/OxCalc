# W048 External Excel Unblock Kit

Status: `ready_for_external_input`

Parent workset: `W048 Circular Dependency Calculation Processing`

Related beads:

- `calc-zci1.16` / `BLK-W048-EXCEL-ROOT`
- `calc-zci1.19` / `BLK-W048-EXCEL-VERSION`

## 1. Purpose

This packet defines the exact external evidence needed to unblock the remaining W048 workset blockers. It is intended for a user or agent with access to either:

1. an interactive Excel UI session capable of observing circular-reference warnings/status-bar report cells; or
2. a second Excel host/version/channel different from the current local host.

Current local blocker state:

1. `BLK-W048-EXCEL-ROOT`: `Application.CircularReference` returned null across targeted COM variants in `w048-excel-root-report-001`; W048 still needs UI warning/report-cell evidence or another public object-model surface.
2. `BLK-W048-EXCEL-VERSION`: all local Excel packets use the same observed host family (`16.0` / `19929`); W048 still needs a second host/version packet or explicit user acceptance of a single-host scope.

## 2. Files To Copy To The External Excel Host

Minimum scripts:

```text
scripts/run-w048-excel-cycle-probes.ps1
scripts/run-w048-excel-root-report-probes.ps1
scripts/run-w048-excel-initial-vector-probes.ps1
scripts/run-w048-excel-nonnumeric-prior-probes.ps1
scripts/run-w048-excel-multithread-variant-probes.ps1
scripts/check-w048-excel-observation-packet.ps1
scripts/check-w048-excel-root-report-probes.ps1
scripts/check-w048-excel-initial-vector-probes.ps1
scripts/check-w048-excel-nonnumeric-prior-probes.ps1
scripts/check-w048-excel-multithread-variant-probes.ps1
```

Prepared handoff bundle in this repo:

```text
docs/handoffs/w048-external-excel-unblock-kit/
docs/handoffs/w048-external-excel-unblock-kit.zip
```

Recommended destination layout on the external host after extracting/copying the bundle:

```text
OxCalc-w048-excel-unblock/
  README.md
  scripts/
  docs/test-runs/excel-cycles/
```

Run PowerShell from the `OxCalc-w048-excel-unblock/` root.

## 3. Required Command Set For Second-Version Packet

Run these commands on a second Excel host/version/channel:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/run-w048-excel-cycle-probes.ps1 -RunId w048-excel-cycles-bitexact-SECONDHOST-001 -ProbeSet bitexact
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/run-w048-excel-initial-vector-probes.ps1 -RunId w048-excel-initial-vector-SECONDHOST-001
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/run-w048-excel-nonnumeric-prior-probes.ps1 -RunId w048-excel-nonnumeric-prior-SECONDHOST-001
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/run-w048-excel-multithread-variant-probes.ps1 -RunId w048-excel-multithread-variant-SECONDHOST-001
```

Then validate locally on that host:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/check-w048-excel-observation-packet.ps1 docs/test-runs/excel-cycles/w048-excel-cycles-bitexact-SECONDHOST-001 -MinimumProbeCount 19
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/check-w048-excel-initial-vector-probes.ps1 docs/test-runs/excel-cycles/w048-excel-initial-vector-SECONDHOST-001
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/check-w048-excel-nonnumeric-prior-probes.ps1 docs/test-runs/excel-cycles/w048-excel-nonnumeric-prior-SECONDHOST-001
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/check-w048-excel-multithread-variant-probes.ps1 docs/test-runs/excel-cycles/w048-excel-multithread-variant-SECONDHOST-001
```

Return the full `docs/test-runs/excel-cycles/*SECONDHOST*` directories.

## 4. Required Evidence For Root/Report-Cell Packet

The COM packet is already runnable:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/run-w048-excel-root-report-probes.ps1 -RunId w048-excel-root-report-SECONDHOST-001
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/check-w048-excel-root-report-probes.ps1 docs/test-runs/excel-cycles/w048-excel-root-report-SECONDHOST-001
```

However, the local COM result was null. If the external host also returns null, UI evidence is required.

A UI evidence packet should include:

1. Excel version/build/channel and OS details;
2. workbook/probe id;
3. formula edit sequence;
4. whether iterative calculation is enabled;
5. screenshot or transcription of the circular-reference warning/status bar/navigation surface;
6. reported cell address/root if visible;
7. final cell values after calculation/rebuild;
8. saved workbook artifact if possible.

Minimum UI probes:

| Probe id | Setup | Required visible evidence |
| --- | --- | --- |
| `ui_root_self_no_iteration_001` | `A1 = A1 + 1`, iteration off | warning/status-bar/report cell if shown |
| `ui_root_two_node_ab_001` | edit `A1 = B1 + 1`, then `B1 = A1 + 1`, iteration off | warning/status-bar/report cell if shown |
| `ui_root_two_node_ba_001` | edit `B1 = A1 + 1`, then `A1 = B1 + 1`, iteration off | warning/status-bar/report cell if shown |
| `ui_root_self_iteration_001` | `A1 = A1 + 1`, iteration on | warning/status-bar/report cell if shown |

If no UI warning/status-bar/report cell is available, the packet must state that explicitly and include screenshots showing the absence.

## 5. Acceptance Paths

W048 can proceed to parent disposition through either path:

### Path A: Evidence Unblocks Both Blockers

Required:

1. second-host/version normalized packets are returned and pass checkers;
2. root/report-cell UI or public object-model evidence is returned;
3. `W048_ITERATIVE_PROFILE_DECISION.json`, `W048_EXCEL_OBSERVATION_LEDGER.md`, and conformance evidence are updated;
4. blockers `calc-zci1.16` and `calc-zci1.19` close with fresh-eyes reviews;
5. whole-workset audit is rerun.

### Path B: User Accepts Narrow Scope

Required user acceptance statement:

```text
I accept W048 closure scoped to the current single observed Excel host/version and without exact Excel UI report-cell/root matching. Keep root/report-cell and cross-version behavior as documented limitations rather than closure blockers.
```

If this is accepted, W048 must update the profile and audit to say the closure is single-host/single-thread/default-root-limited rather than broad Excel compatibility.

## 6. Fresh-Eyes Review

Review date: 2026-05-11

Review questions:

1. Does this kit avoid requiring Python?
2. Does it name exact commands and return artifacts?
3. Does it distinguish evidence-based unblock from user scope acceptance?
4. Does it prevent accidental broad Excel-compatibility overclaiming?

Findings:

1. Yes. The kit uses PowerShell scripts only.
2. Yes. It names run commands, checker commands, expected artifact roots, and UI evidence fields.
3. Yes. Path A requires evidence; Path B requires explicit user acceptance of a narrower claim.
4. Yes. Any narrower path must be recorded as a limitation in profile/conformance/audit surfaces.
5. A copy-ready bundle and zip have been prepared under `docs/handoffs/` so no repo checkout is required on the external Excel host.

## 7. Three-Axis Status

- scope_completeness: `scope_partial`
- target_completeness: `target_partial`
- integration_completeness: `partial`
- open_lanes:
  - `BLK-W048-EXCEL-ROOT`;
  - `BLK-W048-EXCEL-VERSION`.
