# W048 TraceCalc Reference Cycle Behavior

Status: `active_execution_evidence`

## 1. Purpose

This packet records the W048 TraceCalc reference behavior slice for circular dependency processing. It covers structural cycle rejection, CTRO-created candidate cycle rejection with no overlay commit, and release/re-entry with downstream recomputation.

## 2. Code And Corpus Changes

| Surface | Path |
| --- | --- |
| TraceCalc machine diagnostic event support | `src/oxcalc-tracecalc/src/machine.rs` |
| manifest | `docs/test-corpus/core-engine/tracecalc/MANIFEST.json` |
| structural self-cycle fixture | `docs/test-corpus/core-engine/tracecalc/hand-auditable/tc_w048_structural_self_cycle_reject_001.json` |
| CTRO candidate-cycle fixture | `docs/test-corpus/core-engine/tracecalc/hand-auditable/tc_w048_ctro_candidate_cycle_reject_001.json` |
| CTRO release/re-entry fixture | `docs/test-corpus/core-engine/tracecalc/hand-auditable/tc_w048_ctro_release_reentry_downstream_001.json` |

Machine changes are deliberately narrow:

1. `emit_plan_artifacts` now treats self-loop SCCs as cycle regions instead of only classifying multi-member SCCs as cyclic.
2. `emit_candidate_result` and `emit_reject` now emit explicit scenario-provided `diagnostic_events` into the trace. This gives W048 fixtures replay-visible labels for candidate-overlay cycle detection, no-overlay-commit suppression, and release/re-entry without hard-coding W048-specific behavior into the reference machine.

## 3. Evidence Run

Command:

```powershell
cargo run -p oxcalc-tracecalc-cli -- w048-tracecalc-cycles-003
```

Run root: `docs/test-runs/core-engine/tracecalc-reference-machine/w048-tracecalc-cycles-003/`

Summary from `run_summary.json`:

| Field | Value |
| --- | ---: |
| scenario_count | 34 |
| passed | 34 |

W048 scenario results:

| Scenario | Result | Key evidence |
| --- | --- | --- |
| `tc_w048_structural_self_cycle_reject_001` | passed | self-loop cycle region detected, `synthetic_cycle_reject`, prior published value retained |
| `tc_w048_ctro_candidate_cycle_reject_001` | passed | candidate-overlay cycle diagnostic, candidate overlay commit suppressed, no new cycle values published |
| `tc_w048_ctro_release_reentry_downstream_001` | passed | first candidate cycle rejected, downstream blocked diagnostic emitted, rejected work re-entered, release candidate published `A=11`, downstream `D=12` |

## 4. Reference Policy Expressed

This TraceCalc slice expresses W048 Stage 1 non-iterative behavior as follows:

1. Structural cycle regions route to `synthetic_cycle_reject`.
2. Rejected cycle candidates publish no new cycle-region values.
3. Candidate-overlay cycles emit an explicit `candidate_cycle_region_detected` diagnostic and suppress candidate overlay commit.
4. Rejected cycle work can re-enter on a later acyclic candidate.
5. Release/re-entry invalidates/recomputes downstream dependents in the reference trace.
6. Iterative behavior remains out of this slice and is routed to `calc-zci1.4`.

## 5. Review Checks

Post-change review checks used for this bead:

```powershell
cargo run -p oxcalc-tracecalc-cli -- w048-tracecalc-cycles-003
python - <<'PY'
import json, pathlib
root = pathlib.Path('docs/test-runs/core-engine/tracecalc-reference-machine/w048-tracecalc-cycles-003')
summary = json.load(open(root / 'run_summary.json', encoding='utf-8'))
assert summary['scenario_count'] == 34
assert summary['result_counts'] == {'passed': 34}
for sid in [
    'tc_w048_structural_self_cycle_reject_001',
    'tc_w048_ctro_candidate_cycle_reject_001',
    'tc_w048_ctro_release_reentry_downstream_001',
]:
    result = json.load(open(root / 'scenarios' / sid / 'result.json', encoding='utf-8'))
    assert result['result_state'] == 'passed', (sid, result)
print('w048 tracecalc cycle review ok')
PY
cargo test -p oxcalc-tracecalc
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/check-worksets.ps1
```

## 6. Limits

This is a reference-machine and fixture slice, not an optimized TreeCalc/core behavior claim. CTRO-created cycle detection is represented through explicit reference diagnostics and reject details in fixtures; the optimized/core implementation and native sidecar conformance remain routed to `calc-zci1.6` and `calc-zci1.7`.
