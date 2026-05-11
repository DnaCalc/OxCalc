# W048 Excel Circular-Reference Observation Ledger

Status: `active_execution_evidence`

## 1. Purpose

This ledger records the first W048 black-box Excel circular-reference probe packet and maps it to OxCalc cycle-policy fields. It is evidence input for W048; it is not an Excel-compatibility closure claim.

## 2. Observation Packet

| Field | Value |
| --- | --- |
| run_id | `w048-excel-cycles-001` |
| artifact root | `docs/test-runs/excel-cycles/w048-excel-cycles-001/` |
| runner | `scripts/run-w048-excel-cycle-probes.ps1` |
| Excel version/build | `16.0` / `19929` |
| platform/locale | Windows / `en-ZA` |
| probe count | 12 |
| evidence files | `environment.json`, `probe_plan.json`, `raw_com_log.jsonl`, per-probe `workbook_before.xlsx`, `workbook_after.xlsx`, `cell_snapshots.json`, `calc_chain_snapshot.json`, `observation.json`, aggregate `observation.json`, `normalization_notes.md` |

Clean-room basis: COM automation against Excel, saved workbook artifacts, and public workbook ZIP metadata. No private implementation material or reverse engineering was used.

## 3. Packet-Level Caveats

1. The run records a first single-host observation packet only; it does not cover multi-threaded variants, UI warning capture, or cross-version Excel behavior.
2. `Application.CircularReference` returned no reported cell for the recorded probes in this automation run. W048 therefore treats reported-root evidence from this packet as inconclusive, while still retaining saved calculation-chain and cell-output evidence.
3. The initial application-level calculation-mode set failed before workbook creation with `0x800A03EC`; the runner retried workbook-scoped setup inside each probe. The packet is valid for observed cell/chain artifacts but should be paired with a second runner variant before any strong Excel-match statement.
4. Saved `xl/calcChain.xml` was present in the recorded workbooks and is treated as public workbook metadata, not as internal algorithm evidence.

## 4. Probe Summary

| Probe | Cycle family | Key observed surface | Calc-chain entries | W048 use |
| --- | --- | --- | --- | --- |
| `excel_struct_self_001` | structural self | `A1 = A1 + 1` yielded `A1 = 1`; no COM-reported circular cell | `A1` | direct self-cycle baseline; root inconclusive |
| `excel_struct_self_prior_001` | structural self/prior | prior seed `5` then `A1 = A1 + 1` yielded `A1 = 1` | `A1` | prior-retention requires second-pass/UI confirmation |
| `excel_struct_two_node_001` | structural SCC | `A1 = 1`, `B1 = 2` after A-then-B edit | `B1`, `A1` | edit/chain-order candidate evidence |
| `excel_struct_three_node_001` | structural SCC | `A1 = 2`, `B1 = 1`, `C1 = 3` | `C1`, `B1`, `A1` | SCC chain-order evidence |
| `excel_struct_guarded_activation_001` | guarded self | activation left `A1 = 0`, `B1 = 1` | `A1` | guarded activation/prior-value follow-up input |
| `excel_iter_self_increment_001` | iterative self | with iteration enabled and max iterations `5`, `A1 = 1` | `A1` | iterative initial-value/terminal probe input |
| `excel_iter_two_node_order_001` | iterative SCC/order | with max iterations `1`, `A1 = 11`, `B1 = 22` | `B1`, `A1` | distinguishes update/order hypotheses; sequential behavior candidate |
| `excel_chain_edit_order_ab_001` | edit-order SCC | A-then-B edit yielded `A1 = 1`, `B1 = 2` | `B1`, `A1` | edit-order sensitivity evidence |
| `excel_chain_edit_order_ba_001` | edit-order SCC | B-then-A edit yielded `A1 = 2`, `B1 = 1` | `A1`, `B1` | edit-order sensitivity evidence |
| `excel_ctro_indirect_self_001` | dynamic reference | selector `B1 = "A1"`, `A1 = INDIRECT(B1)+1` yielded `A1 = 1` | `A1` | CTRO analog self-cycle input |
| `excel_ctro_indirect_release_001` | dynamic release | selector changed from `A1` to `C1`; final `A1 = 11` | `A1` | release/re-entry input |
| `excel_ctro_downstream_001` | dynamic downstream | after release, `A1 = 11`, downstream `D1 = 12` | `A1`, `D1` | downstream recompute input |

## 5. Disposition For W048 Policy Work

1. **Non-iterative Stage 1**: this packet supports keeping OxCalc's Stage 1 policy conservative: detect SCCs in OxCalc-owned graph layers, reject cycle-region publication, and emit diagnostics rather than adopting Excel's automation-observed one-pass values as default semantics.
2. **Root/order**: saved calculation-chain entries changed with edit order in the two-node pair (`B1,A1` versus `A1,B1`). Excel-compatible root/order remains a candidate profile field and is not reduced to canonical node id by this packet.
3. **Initial value**: self/prior and iterative probes are insufficient for a strong initial-value claim because the COM-reported circular root was absent. W048 should preserve `published_prior_value`, `zero_or_blank_default`, and chain-ordered hypotheses until a second packet captures UI warning/root behavior or a more direct object-model surface.
4. **Update model**: the one-iteration two-node probe produced `A1 = 11`, `B1 = 22`, matching the sequential A-then-B hypothesis for that workbook/edit sequence. This is evidence for `excel_chain_ordered` as a candidate, not a final algorithm decision.
5. **CTRO analogs**: INDIRECT release/downstream probes provide concrete expected visible surfaces for W048 release/re-entry fixtures (`A1 = 11`, `D1 = 12` after selector moves from self to `C1`). They do not by themselves settle candidate-overlay commit policy inside OxCalc.

## 6. Required Follow-Up Beads

1. `calc-zci1.2`: consume this packet when designing graph sidecars so saved chain/order and OxCalc `cycle_root`/`member_order` can be compared without trusting traversal accidents.
2. `calc-zci1.3` and `calc-zci1.6`: use the CTRO release/downstream visible surfaces as fixture targets while keeping OxCalc's default non-iterative cycle publication policy conservative.
3. `calc-zci1.4`: run a second Excel packet or explicitly defer Excel-compatible iteration if root/warning and initial-vector behavior remain inconclusive.
4. `calc-zci1.7`: include `w048-excel-cycles-001` in the circular-reference corpus manifest and assert that packet caveats are visible to conformance summaries.
