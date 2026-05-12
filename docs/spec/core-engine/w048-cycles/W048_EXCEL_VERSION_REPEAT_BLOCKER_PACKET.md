# W048 Excel Version Repeat Blocker Packet

Status: `blocked_waiting_for_second_excel_host`

Parent bead: `calc-zci1.19`

Blocker: `BLK-W048-EXCEL-VERSION`

## 1. Purpose

This packet records the status of the cross-version Excel repeat requirement. W048 has multiple black-box Excel packets, but they were all captured on the same available Excel host/version in this environment.

## 2. Evidence Inventory

Observed local Excel version/build across W048 packets:

| Packet | Version/build source | Observed version/build |
| --- | --- | --- |
| `w048-excel-cycles-001` | `environment.json` | `16.0` / `19929` |
| `w048-excel-cycles-bitexact-001` | `environment.json` | `16.0` / `19929` |
| `w048-excel-root-report-001` | `environment.json` | same local host family |
| `w048-excel-initial-vector-001` | `environment.json` | same local host family |
| `w048-excel-nonnumeric-prior-001` | `environment.json` | same local host family |
| `w048-excel-root-report-002` | `environment.json` | `16.0` / `19929` |
| `w048-excel-version-inventory-001` | `inventory.json` | one distinct local product version: `16.0.19929.20136`; `second_host_available_in_local_inventory=false` |

## 3. Local Inventory Packet

Fresh-eyes follow-up on 2026-05-12 added a local Excel installation inventory packet:

| Field | Value |
| --- | --- |
| run_id | `w048-excel-version-inventory-001` |
| artifact root | `docs/test-runs/excel-cycles/w048-excel-version-inventory-001/` |
| runner | `scripts/run-w048-excel-version-inventory.ps1` |
| checker | `scripts/check-w048-excel-version-inventory.ps1` |
| result | one distinct local Excel product version; no second local host/version found |

The inventory intentionally records only installation/version/channel fields and excludes user identity, tenant, activation, and MRU data.

Validation command:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/check-w048-excel-version-inventory.ps1
```

Observed result:

```text
w048 excel version inventory ok: run=w048-excel-version-inventory-001 com=16.0/19929 product_versions=1 second_host_available=False
```

## 4. Fresh-Eyes Review For `calc-zci1.19`

Review date: 2026-05-12

Review questions:

1. Is a second Excel host/version available in the current repo or tool environment?
2. Can the requirement be satisfied by rerunning the same local host?
3. Is it safe to close the blocker as resolved?
4. What exact unblock input is needed?

Findings:

1. No second Excel host/version is available through the current local tooling surface; `w048-excel-version-inventory-001` found one distinct local Excel product version.
2. No. Rerunning the same host would add repeat evidence but would not satisfy the cross-version requirement.
3. No. Closing this as resolved would overclaim broad Excel compatibility.
4. Required unblock: run the falsification fixture set on another Excel version/channel/host and commit the normalized packet, or receive explicit user acceptance that W048's Excel-match claim is scoped to the single observed host/version.

Fresh-eyes result: `calc-zci1.19` remains blocked. The blocker is external/environmental, not an implementation failure.

## 5. Three-Axis Status

- scope_completeness: `scope_partial`
- target_completeness: `target_partial`
- integration_completeness: `partial`
- open_lanes:
  - second Excel host/version packet;
  - or explicit user acceptance of single-host scope.
