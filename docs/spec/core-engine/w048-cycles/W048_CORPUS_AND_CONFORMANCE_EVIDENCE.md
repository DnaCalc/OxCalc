# W048 Corpus And Conformance Evidence

Status: `active_execution_evidence`

## 1. Scope

This packet binds the W048 circular-reference corpus and conformance run surface across Excel observations, TraceCalc reference fixtures, TreeCalc optimized/core fixtures, materialized graph/checker projections, and formal/checker artifacts.

This is a reopened W048 conformance packet. It records passed declared coverage plus named blockers; it is not a broad final Excel-compatibility closure claim.

## 2. Evidence Roots

| Evidence family | Root / file | Observed summary |
| --- | --- | --- |
| Excel core black-box observations | `docs/test-runs/excel-cycles/w048-excel-cycles-001/observation.json` | 12 observations |
| Excel bit-exact black-box observations | `docs/test-runs/excel-cycles/w048-excel-cycles-bitexact-001/observation.json` | 19 observations |
| TraceCalc reference behavior | `docs/test-runs/core-engine/tracecalc-reference-machine/w048-tracecalc-cycles-004/` | 38 scenarios passed |
| TreeCalc optimized/core behavior | `docs/test-runs/core-engine/treecalc-local/w048-treecalc-cycles-002/` | 37 cases, 0 expectation mismatches |
| Materialized graph checker | `docs/test-runs/core-engine/treecalc-local/w048-treecalc-cycles-002/w048_materialized_graph_check_summary.json` | reopened iterative sidecar floor: 37 cases, 111 graph layers, 24 cycle-region records, 0 checker errors |
| Excel root/report-cell observations | `docs/test-runs/excel-cycles/w048-excel-root-report-002/observation.json` | 5 worksheet-scoped root/report probes |
| Cross-corpus conformance summary | `docs/test-runs/core-engine/w048-conformance-002/w048_conformance_summary.json` | `status=passed_with_named_excel_version_blocker` |

## 3. Corpus Coverage Matrix

| Requirement from reopened W048 | Status | Evidence |
| --- | --- | --- |
| structural direct self-cycle | covered | TraceCalc fixture; TreeCalc fixture; graph cycle-region sidecars |
| structural two-node SCC | covered | TreeCalc fixture; graph cycle-region sidecars |
| structural three-node SCC with member ordering | covered by Excel observation and iterative fixtures | Excel `excel_struct_three_node_001`; TraceCalc/TreeCalc three-node iterative fixture |
| guarded activation cycle with prior-value retention question | observed, not fully generalized | Excel observation ledger; blocker `BLK-W048-EXCEL-INITIAL` |
| CTRO dynamic self-cycle | covered | TreeCalc fixture; graph cycle-region sidecars; Excel dynamic self observation |
| CTRO iterative dynamic self-cycle | covered for declared fixture | Excel `excel_ctro_indirect_iterative_self_001`; TraceCalc/TreeCalc fixtures |
| CTRO candidate cycle rejected with no overlay commit | covered | TraceCalc no-overlay fixture and TreeCalc no-publication/no-commit result artifacts |
| CTRO cycle release and re-entry | covered | TraceCalc release fixture; TreeCalc post-edit release fixture |
| downstream dependent blocked by cycle and recomputed after release | covered | TraceCalc release/downstream fixture; TreeCalc post-edit downstream value `11` |
| order-sensitive iterative SCC | covered for declared two-node and three-node fixtures | Excel bit-exact packet; TraceCalc/TreeCalc iterative fixtures |
| fractional precision iterative case | covered for declared fixture | Excel `0.33333333333333331`; TraceCalc/TreeCalc matching fixtures |
| root/report-cell behavior | covered for declared local non-iterative probes | `w048-excel-root-report-002`; `BLK-W048-EXCEL-ROOT` cleared |
| blank/text/error prior values | covered for declared self-cycle probes | `w048-excel-nonnumeric-prior-001`; `BLK-W048-EXCEL-NONNUMERIC` cleared |
| cross-version and multithread variants | version blocked; multithread observed variant | `BLK-W048-EXCEL-VERSION`; `w048-excel-multithread-variant-001` |
| graph materialization reverse-edge converse case | covered by reopened iterative sidecar floor | `scripts/check-w048-materialized-graphs.ps1` over `w048-treecalc-cycles-002` |
| formal definitions/checker artifacts | covered for current formal floor | `scripts/check-w048-formal-cycle-artifacts.ps1` |
| innovation profile examples | covered as profile-gated ledger | `scripts/check-w048-innovation-ledger.ps1` |

## 4. Checker Commands

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/check-w048-excel-observation-packet.ps1 docs/test-runs/excel-cycles/w048-excel-cycles-001
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/check-w048-excel-observation-packet.ps1 docs/test-runs/excel-cycles/w048-excel-cycles-bitexact-001 -MinimumProbeCount 19
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/check-w048-excel-root-report-probes.ps1 docs/test-runs/excel-cycles/w048-excel-root-report-002
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/check-w048-iterative-profile-decision.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/check-w048-tracecalc-iterative-cycles.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/check-w048-treecalc-iterative-cycles.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/write-w048-materialized-graphs.ps1 docs/test-runs/core-engine/treecalc-local/w048-treecalc-cycles-002
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/check-w048-materialized-graphs.ps1 docs/test-runs/core-engine/treecalc-local/w048-treecalc-cycles-002
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/check-w048-conformance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/check-w048-formal-cycle-artifacts.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/check-w048-innovation-ledger.ps1
```

Observed conformance result:

```text
w048 conformance passed with named blocker disposition: docs/test-runs/core-engine/w048-conformance-002/w048_conformance_summary.json
```

The checker fails if any of these floors are not met:

1. Excel core observation count is below 12.
2. Excel bit-exact observation count is below 19.
3. TraceCalc W048 run is not 38/38 passed.
4. TreeCalc W048 run is not 37 cases with 0 expectation mismatches.
5. Required TraceCalc and TreeCalc iterative fixtures are missing.
6. W048 graph checker reopened floor is not 37 cases / 111 layers / at least 24 cycle-region records / 0 checker errors.
7. The remaining named Excel version blocker is missing or a cleared root blocker is still listed as open.
8. Any status axis incorrectly claims final full closure while the version blocker remains.

## 5. Fresh-Eyes Review For `calc-zci1.15`

Review date: 2026-05-11

Review questions:

1. Does the conformance checker cover the reopened iterative evidence, not just predecessor Stage 1 evidence?
2. Does a green conformance checker accidentally imply full Excel closure?
3. Are TraceCalc and TreeCalc both bound to the same declared Excel fixtures?
4. Are graph sidecars and formal/innovation checks still visible?

Findings:

1. `scripts/check-w048-conformance.ps1` now targets `w048-conformance-002` and requires the bit-exact Excel packet, TraceCalc 38-scenario run, TreeCalc 37-case run, and iterative fixture IDs on both engines.
2. The conformance status is `passed_with_named_excel_version_blocker`, with all three status axes partial while the version blocker remains.
3. Both engines cover the same four falsification fixtures from `W048_ITERATIVE_PROFILE_DECISION.json`.
4. Graph sidecars are regenerated for the reopened iterative TreeCalc run; formal and innovation checker commands remain part of the audit surface.

Fresh-eyes result: `calc-zci1.15` satisfies the reopened conformance audit gate for declared coverage. A later fresh-eyes repair cleared `BLK-W048-EXCEL-ROOT` via `w048-excel-root-report-002`; W048 parent closure remains blocked by the second-host/version lane and whole-workset final disposition.

## 6. Three-Axis Status

- scope_completeness: `scope_partial`
- target_completeness: `target_partial`
- integration_completeness: `partial`
- open_lanes:
  - `BLK-W048-EXCEL-VERSION`;
  - whole-workset fresh-eyes audit and user disposition for the version blocker before parent closure.
- cleared_lanes:
  - `BLK-W048-EXCEL-ROOT` by `w048-excel-root-report-002`;
  - `BLK-W048-EXCEL-INITIAL` by `w048-excel-initial-vector-001`;
  - `BLK-W048-EXCEL-NONNUMERIC` by `w048-excel-nonnumeric-prior-001`;
  - `BLK-W048-EXCEL-MT` as a run requirement by `w048-excel-multithread-variant-001`, with thread mode retained as a profile dimension.
