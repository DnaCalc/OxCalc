# W048 Corpus And Conformance Evidence

Status: `active_execution_evidence` (TreeCalc iterative rows withdrawn; see the block below)

## Withdrawn 2026-09-14 — TreeCalc iterative conformance rows were self-certifying (calc-9ouy.11)

Every TreeCalc optimized/core row in this packet that rests on the four
Excel-match iterative fixtures (`tc_w048_excel_iter_two_node_order_001`,
`tc_w048_excel_iter_three_node_order_001`,
`tc_w048_excel_iter_fraction_precision_001`,
`tc_w048_excel_ctro_indirect_iterative_self_001`) is withdrawn as engine
evidence: the TreeCalc results were produced by a lookup keyed on the probe id
in the run's `runtime_policy_id` (see
`W048_TREECALC_OPTIMIZED_CYCLE_BEHAVIOR.md`, withdrawal block), so "37 cases,
0 expectation mismatches" in §2 and the `TraceCalc/TreeCalc iterative
fixtures` evidence in the §3 rows *structural three-node SCC with member
ordering*, *CTRO iterative dynamic self-cycle*, *order-sensitive iterative
SCC*, and *fractional precision iterative case* certify only that the table
matched itself. The `w048-conformance-002` summary's `treecalc_expectation_mismatch_count = 0`
and `treecalc_iterative_fixtures` entries inherit the same status. The Excel
observations in those rows remain valid targets; the TraceCalc rows are
hand-auditable trace replays and are not engine-calculation evidence either
way.

Current engine answer for these four cases: the typed rejection
`LocalTreeCalcError::IterativeCalcNotImplemented` (iteration requested, not
implemented), reported by the TreeCalc runner and fixture test as
`open_gap_against_expected` with open gap `calc-9ouy.3`. A fresh TreeCalc run
of the checked-in corpus therefore reports `open_gap_case_count = 4` and
`undeclared_mismatch_count = 0`, not `expectation_mismatch_count = 0`. The
retained run roots named in §2 are append-only and are not regenerated.

## 1. Scope

This packet binds the W048 circular-reference corpus and conformance run surface across Excel observations, TraceCalc reference fixtures, TreeCalc optimized/core fixtures, materialized graph/checker projections, and formal/checker artifacts.

This is a reopened W048 conformance packet. It records passed declared coverage under the user-accepted single-host Excel scope; it is not a broad cross-version Excel-compatibility closure claim.

## 2. Evidence Roots

| Evidence family | Root / file | Observed summary |
| --- | --- | --- |
| Excel core black-box observations | `docs/test-runs/excel-cycles/w048-excel-cycles-001/observation.json` | 12 observations |
| Excel bit-exact black-box observations | `docs/test-runs/excel-cycles/w048-excel-cycles-bitexact-001/observation.json` | 19 observations |
| TraceCalc reference behavior | `docs/test-runs/core-engine/tracecalc-reference-machine/w048-tracecalc-cycles-004/` | 38 scenarios passed |
| TreeCalc optimized/core behavior | `docs/test-runs/core-engine/treecalc-local/w048-treecalc-cycles-002/` | 37 cases, 0 expectation mismatches |
| Materialized graph checker | `docs/test-runs/core-engine/treecalc-local/w048-treecalc-cycles-002/w048_materialized_graph_check_summary.json` | reopened iterative sidecar floor: 37 cases, 111 graph layers, 24 cycle-region records, 0 checker errors |
| Excel root/report-cell observations | `docs/test-runs/excel-cycles/w048-excel-root-report-002/observation.json` | 5 worksheet-scoped root/report probes |
| Cross-corpus conformance summary | `docs/test-runs/core-engine/w048-conformance-002/w048_conformance_summary.json` | `status=passed_single_host_scoped` |

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
| cross-version and multithread variants | cross-version accepted as limitation; multithread observed variant | `W048_SINGLE_HOST_SCOPE_ACCEPTANCE_AND_FINAL_AUDIT.md`; `w048-excel-multithread-variant-001` |
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
w048 conformance ok: status=passed_single_host_scoped path=docs/test-runs/core-engine/w048-conformance-002/w048_conformance_summary.json
```

The checker fails if any of these floors are not met:

1. Excel core observation count is below 12.
2. Excel bit-exact observation count is below 19.
3. TraceCalc W048 run is not 38/38 passed.
4. TreeCalc W048 run is not 37 cases with 0 expectation mismatches.
5. Required TraceCalc and TreeCalc iterative fixtures are missing.
6. W048 graph checker reopened floor is not 37 cases / 111 layers / at least 24 cycle-region records / 0 checker errors.
7. Single-host accepted scope is missing when the summary status is `passed_single_host_scoped`.
8. Any status axis claims broad cross-version closure instead of single-host scoped closure.

## 5. Fresh-Eyes Review For `calc-zci1.15`

Review date: 2026-05-11

Review questions:

1. Does the conformance checker cover the reopened iterative evidence, not just predecessor Stage 1 evidence?
2. Does a green conformance checker accidentally imply full Excel closure?
3. Are TraceCalc and TreeCalc both bound to the same declared Excel fixtures?
4. Are graph sidecars and formal/innovation checks still visible?

Findings:

1. `scripts/check-w048-conformance.ps1` now targets `w048-conformance-002` and requires the bit-exact Excel packet, TraceCalc 38-scenario run, TreeCalc 37-case run, and iterative fixture IDs on both engines.
2. The conformance status is now `passed_single_host_scoped`, with cross-version behavior documented as a limitation rather than an active blocker.
3. Both engines cover the same four falsification fixtures from `W048_ITERATIVE_PROFILE_DECISION.json`.
4. Graph sidecars are regenerated for the reopened iterative TreeCalc run; formal and innovation checker commands remain part of the audit surface.

Fresh-eyes result: `calc-zci1.15` satisfies the reopened conformance audit gate for declared coverage. Later fresh-eyes repairs cleared `BLK-W048-EXCEL-ROOT` via `w048-excel-root-report-002` and closed the version lane by explicit user-accepted single-host scope.

## 6. Three-Axis Status

- scope_completeness: `scope_complete_single_host`
- target_completeness: `target_complete_single_host`
- integration_completeness: `integrated_single_host`
- open_lanes: []
- documented_limitations:
  - no broad cross-version Excel compatibility claim.
- cleared_lanes:
  - `BLK-W048-EXCEL-ROOT` by `w048-excel-root-report-002`;
  - `BLK-W048-EXCEL-INITIAL` by `w048-excel-initial-vector-001`;
  - `BLK-W048-EXCEL-NONNUMERIC` by `w048-excel-nonnumeric-prior-001`;
  - `BLK-W048-EXCEL-MT` as a run requirement by `w048-excel-multithread-variant-001`, with thread mode retained as a profile dimension.
