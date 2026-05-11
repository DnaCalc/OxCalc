# W048 Corpus And Conformance Evidence

Status: `active_execution_evidence`

## 1. Scope

This packet binds the W048 circular-reference corpus and conformance run surface across Excel observations, TraceCalc reference fixtures, TreeCalc optimized/core fixtures, and graph/checker projections.

## 2. Evidence Roots

| Evidence family | Root / file | Observed summary |
| --- | --- | --- |
| Excel black-box observations | `docs/test-runs/excel-cycles/w048-excel-cycles-001/observation.json` | 12 observations |
| TraceCalc reference behavior | `docs/test-runs/core-engine/tracecalc-reference-machine/w048-tracecalc-cycles-003/` | 34 scenarios passed |
| TreeCalc optimized/core behavior | `docs/test-runs/core-engine/treecalc-local/w048-treecalc-cycles-001/` | 33 cases, 0 expectation mismatches |
| Materialized graph checker | `docs/test-runs/core-engine/treecalc-local/w048-treecalc-cycles-001/w048_materialized_graph_check_summary.json` | 33 cases, 99 graph layers, 12 cycle-region records, 0 checker errors |
| Cross-corpus conformance summary | `docs/test-runs/core-engine/w048-conformance-001/w048_conformance_summary.json` | `status=passed` |

## 3. Corpus Coverage Matrix

| Requirement from test plan | Status | Evidence |
| --- | --- | --- |
| structural direct self-cycle | covered | TraceCalc fixture; TreeCalc fixture; graph cycle-region sidecars |
| structural two-node SCC | covered | TreeCalc fixture; graph cycle-region sidecars |
| structural three-node SCC with deterministic member ordering | covered by checker floor | W048 materialized graph checker validates SCC/cycle-region ordering over full run |
| guarded activation cycle with prior-value retention question | observed/deferred | Excel observation ledger; iterative disposition continues under `calc-zci1.4` |
| CTRO dynamic self-cycle | covered | TreeCalc fixture; graph cycle-region sidecars |
| CTRO dynamic two-node SCC | deferred | no dedicated fixture yet; retained as explicit gap in conformance summary |
| CTRO candidate cycle rejected with no overlay commit | covered | TraceCalc no-overlay fixture and TreeCalc no-publication/no-commit result artifacts |
| CTRO cycle release and re-entry | covered | TraceCalc release fixture; TreeCalc post-edit release fixture |
| downstream dependent blocked by cycle and recomputed after release | covered | TraceCalc release/downstream fixture; TreeCalc post-edit downstream value `11` |
| iterative self-cycle after profile selection | deferred | `calc-zci1.4` owns profile selection before fixture promotion |
| order-sensitive iterative SCC after profile selection | deferred | `calc-zci1.4` owns profile selection before fixture promotion |
| graph materialization reverse-edge converse case | covered | `scripts/check-w048-materialized-graphs.py` over TreeCalc run |
| candidate-effective graph cycle introduction case | covered | W048 TreeCalc graph sidecars |
| innovation profile example when admitted | deferred | `calc-zci1.8` owns profile-gated experimental entries |

## 4. Checker Command

```powershell
python scripts/check-w048-conformance.py
```

Observed result:

```text
w048 conformance passed: docs/test-runs/core-engine/w048-conformance-001/w048_conformance_summary.json
```

The checker fails if any of these floors are not met:

1. Excel observation count is below 12.
2. TraceCalc W048 run is not 34/34 passed.
3. Required W048 TraceCalc fixtures are missing.
4. TreeCalc W048 run is not 33 cases with 0 expectation mismatches.
5. Required W048 TreeCalc cases are missing or have unexpected reject/publication states.
6. W048 graph checker is not 33 cases / 99 layers / at least 12 cycle-region records / 0 checker errors.

## 5. Review Notes

The conformance summary is intentionally not a final W048 closure claim. It records a passed corpus/conformance slice and preserves explicit deferred lanes for beads that are still open: iterative profile selection (`calc-zci1.4`), W048 formal proof/model artifacts (`calc-zci1.5`), and innovation profiles (`calc-zci1.8`).
