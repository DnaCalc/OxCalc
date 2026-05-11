# W048 Excel Circular-Reference Observation Ledger

Status: `active_execution_evidence`

## 1. Purpose

This ledger records W048 black-box Excel circular-reference probe packets and maps them to OxCalc cycle-policy fields. These packets are evidence input for W048; they are not an Excel-compatibility closure claim.

## 2. Observation Packets

| Field | Core packet | Bit-exact packet |
| --- | --- | --- |
| run_id | `w048-excel-cycles-001` | `w048-excel-cycles-bitexact-001` |
| artifact root | `docs/test-runs/excel-cycles/w048-excel-cycles-001/` | `docs/test-runs/excel-cycles/w048-excel-cycles-bitexact-001/` |
| runner | `scripts/run-w048-excel-cycle-probes.ps1` | `scripts/run-w048-excel-cycle-probes.ps1 -ProbeSet bitexact` |
| Excel version/build | `16.0` / `19929` | `16.0` / `19929` |
| platform/locale | Windows / `en-ZA` | Windows / `en-ZA` |
| probe count | 12 | 19 |
| evidence files | `environment.json`, `probe_plan.json`, `raw_com_log.jsonl`, per-probe `workbook_before.xlsx`, `workbook_after.xlsx`, `cell_snapshots.json`, `calc_chain_snapshot.json`, `observation.json`, aggregate `observation.json`, `normalization_notes.md` | same layout |

Additional blocker packet:

| Field | Value |
| --- | --- |
| run_id | `w048-excel-root-report-001` |
| artifact root | `docs/test-runs/excel-cycles/w048-excel-root-report-001/` |
| runner | `scripts/run-w048-excel-root-report-probes.ps1` |
| checker | `scripts/check-w048-excel-root-report-probes.ps1` |
| probe count | 5 |
| disposition | `Application.CircularReference` remained null across self/two-node/three-node and iterative/non-iterative variants; `BLK-W048-EXCEL-ROOT` remains open. |
| run_id | `w048-excel-initial-vector-001` |
| artifact root | `docs/test-runs/excel-cycles/w048-excel-initial-vector-001/` |
| runner | `scripts/run-w048-excel-initial-vector-probes.ps1` |
| checker | `scripts/check-w048-excel-initial-vector-probes.ps1` |
| probe count | 8 |
| disposition | numeric prior seeds did not survive self-cycle formula assignment in declared probes; numeric-prior portion of `BLK-W048-EXCEL-INITIAL` is unblocked. |
| run_id | `w048-excel-nonnumeric-prior-001` |
| artifact root | `docs/test-runs/excel-cycles/w048-excel-nonnumeric-prior-001/` |
| runner | `scripts/run-w048-excel-nonnumeric-prior-probes.ps1` |
| checker | `scripts/check-w048-excel-nonnumeric-prior-probes.ps1` |
| probe count | 8 |
| disposition | blank/text/error prior states did not survive self-cycle formula assignment in declared probes; `BLK-W048-EXCEL-NONNUMERIC` is unblocked for declared self-cycle coverage. |
| run_id | `w048-excel-multithread-variant-001` |
| artifact root | `docs/test-runs/excel-cycles/w048-excel-multithread-variant-001/` |
| runner | `scripts/run-w048-excel-multithread-variant-probes.ps1` |
| checker | `scripts/check-w048-excel-multithread-variant-probes.ps1` |
| probe count | 4 |
| disposition | multithread variants were run; values differ from single-threaded fixtures, so thread mode remains an explicit profile dimension. |

Clean-room basis: COM automation against Excel, saved workbook artifacts, and public workbook ZIP metadata. No private implementation material or reverse engineering was used.

## 3. Packet-Level Caveats

1. The runs record single-host observation packets only; they do not cover multi-threaded variants, UI warning capture, or cross-version Excel behavior.
2. `Application.CircularReference` returned no reported cell for the recorded probes in these automation runs, including targeted root/report packet `w048-excel-root-report-001`. W048 therefore treats reported-root evidence as inconclusive, while still retaining saved calculation-chain and cell-output evidence.
3. The initial application-level calculation-mode set failed before workbook creation with `0x800A03EC`; the runner retried workbook-scoped setup inside each probe. The packets are valid for observed cell/chain artifacts but should be paired with a second runner variant before any strong Excel-match statement.
4. Saved `xl/calcChain.xml` was present in most recorded workbooks and is treated as public workbook metadata, not as internal algorithm evidence.
5. The non-numeric prior probe preserved a formula-like text surface in the cell snapshot; it is retained as evidence that this path needs targeted follow-up before deriving coercion policy.

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
| `excel_iter_self_decay_001` | iterative convergence | seed `8`, then `A1 = A1 / 2` yielded `A1 = 0` | `A1` | convergence/max-change follow-up input |
| `excel_iter_three_node_order_001` | iterative SCC/order | max iterations `2` yielded `A1 = 102`, `B1 = 101`, `C1 = 103` | `C1`, `B1`, `A1` | three-node chain-order evidence |
| `excel_iter_oscillation_001` | iterative oscillation | seed `1`, then `A1 = -A1` yielded `A1 = 0` | `A1` | oscillation/initial-vector follow-up input |
| `excel_iter_non_numeric_prior_001` | iterative non-numeric prior | prior text then `A1 = A1 + 1` retained formula-like text surface | none | coercion/error follow-up input |
| `excel_iter_fraction_precision_001` | iterative fractional convergence | `A1 = (A1 + 1) / 3` yielded `0.33333333333333331` (`Text` `0.333333`) | `A1` | bit-exact numeric precision input |
| `excel_chain_full_rebuild_compare_001` | iterative full-rebuild compare | after full rebuild plus calculate, `A1 = 1`, `B1 = 2` | `B1`, `A1` | full-rebuild/order input |
| `excel_ctro_indirect_iterative_self_001` | iterative dynamic reference | `INDIRECT` self-cycle with iteration enabled yielded `A1 = 1` | `A1` | CTRO iterative input |

## 5. Disposition For W048 Policy Work

1. **Non-iterative Stage 1**: this packet supports keeping OxCalc's Stage 1 policy conservative: detect SCCs in OxCalc-owned graph layers, reject cycle-region publication, and emit diagnostics rather than adopting Excel's automation-observed one-pass values as default semantics.
2. **Root/order**: saved calculation-chain entries changed with edit order in the two-node pair (`B1,A1` versus `A1,B1`). Excel-compatible root/order remains a candidate profile field and is not reduced to canonical node id by this packet.
3. **Initial value**: self/prior and iterative probes are insufficient for a strong initial-value claim because the COM-reported circular root was absent. W048 should preserve `published_prior_value`, `zero_or_blank_default`, and chain-ordered hypotheses until a second packet captures UI warning/root behavior or a more direct object-model surface.
4. **Update model**: the one-iteration two-node probe produced `A1 = 11`, `B1 = 22`, matching the sequential A-then-B hypothesis for that workbook/edit sequence. The three-node iterative probe also tracks saved chain order (`C1,B1,A1`). This is evidence for `excel_chain_ordered` as a candidate, not a final algorithm decision.
5. **Convergence and terminal policy**: the bit-exact packet contains decay, fractional convergence, oscillation, and max-iteration probes. The targeted initial-vector and nonnumeric-prior packets add prior-state evidence: self-cycle formula assignment starts from a zero/blank-derived base for the tested formulas rather than retaining numeric, blank, text, or error prior states.
6. **CTRO analogs**: INDIRECT release/downstream probes provide concrete expected visible surfaces for W048 release/re-entry fixtures (`A1 = 11`, `D1 = 12` after selector moves from self to `C1`). They do not by themselves settle candidate-overlay commit policy inside OxCalc.

## 6. Required Follow-Up Beads

1. `calc-zci1.2`: consume this packet when designing graph sidecars so saved chain/order and OxCalc `cycle_root`/`member_order` can be compared without trusting traversal accidents.
2. `calc-zci1.3` and `calc-zci1.6`: use the CTRO release/downstream visible surfaces as fixture targets while keeping OxCalc's default non-iterative cycle publication policy conservative.
3. `calc-zci1.12`: consume observation packets when deriving the Excel iterative semantics profile. Root/warning behavior and non-numeric prior coercion remain weakly observed and must either receive targeted evidence or become explicit blockers.
4. `calc-zci1.16`: targeted COM root/report packet was run and did not unblock report-cell/root behavior; UI warning capture or another public object-model route is still required.
5. `calc-zci1.17`: targeted numeric-prior initial-vector packet was run and unblocked numeric-prior behavior for declared self-cycle coverage.
6. `calc-zci1.18`: targeted blank/text/error prior packet was run and unblocked nonnumeric prior behavior for declared self-cycle coverage.
4. `calc-zci1.13` through `calc-zci1.15`: include `w048-excel-cycles-001` and `w048-excel-cycles-bitexact-001` in the circular-reference corpus/conformance surface and assert that packet caveats are visible to conformance summaries.

## 7. Fresh-Eyes Review For `calc-zci1.11`

Review date: 2026-05-11

Review questions:

1. Does the suite cover every explicitly named axis in the reopened bead: root/order, initial vector, chain-state sensitivity, max-iteration/max-change, convergence, non-convergence/oscillation, blank/text/error prior values, cold-open/full-rebuild, and edit history?
2. Are the observations reproducible from repo-local non-Python tooling?
3. Are packet caveats stated rather than hidden behind a green checker?
4. Are the outputs normalized with workbook provenance and usable as expected surfaces for TraceCalc/TreeCalc work?

Findings:

1. Coverage is broad but not perfect: root/report-cell behavior, UI warning capture, cross-version behavior, and non-numeric prior coercion remain weakly observed. These are carried forward as explicit caveats for `calc-zci1.12`, not silently treated as settled.
2. Reproduction uses `scripts/run-w048-excel-cycle-probes.ps1` with `-ProbeSet core` or `-ProbeSet bitexact`; no Python tooling is required.
3. Validation uses `scripts/check-w048-excel-observation-packet.ps1`, which verifies packet layout, probe count, mandatory probes, per-probe artifacts, iterative/dynamic coverage, and saved calc-chain evidence. It does not claim semantic sufficiency by itself.
4. Both packets include environment metadata, probe plans, raw COM logs, saved workbooks, per-probe snapshots, calc-chain snapshots, and aggregate normalized observations.

Fresh-eyes result: `calc-zci1.11` has sufficient observation artifacts to feed the next W048 profile-derivation bead, with caveats preserved. It does not close Excel-match implementation.
