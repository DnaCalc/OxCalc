# W033 TraceCalc Oracle Self-Check First Slice

Status: `calc-uri.8_evidence_packet`
Workset: `W033`
Parent epic: `calc-uri`
Bead: `calc-uri.8`
Created: 2026-05-04

## 1. Purpose

This packet records the first W033 TraceCalc oracle self-check slice.

It establishes current covered TraceCalc behavior before later W033 beads use TraceCalc as a conformance oracle for production/core-engine comparisons. It does not claim coverage for behavior outside the listed scenario families.

## 2. Run Evidence

| Item | Value |
|---|---|
| W033 run id | `w033-tracecalc-oracle-self-check-001` |
| Artifact root | `docs/test-runs/core-engine/tracecalc-reference-machine/w033-tracecalc-oracle-self-check-001/` |
| Baseline compared | `w019-replay-distill-baseline` |
| Scenario count | `12` |
| Result count | `passed: 12` |
| Engine diff | all 12 entries have empty `mismatches` |
| Replay bundle validation | `status: bundle_valid`; `degraded_capture: false`; `missing_paths: []` |

Commands run:

```powershell
cargo run -p oxcalc-tracecalc-cli -- w033-tracecalc-oracle-self-check-001
.\scripts\compare-tracecalc-run.ps1 -CandidateRunId w033-tracecalc-oracle-self-check-001 -BaselineRunId w019-replay-distill-baseline -RepoRoot C:\Work\DnaCalc\OxCalc
```

Observed command outcomes:

1. The TraceCalc Rust run wrote 12 scenario artifacts to `docs/test-runs/core-engine/tracecalc-reference-machine/w033-tracecalc-oracle-self-check-001/`.
2. The parity check passed against `w019-replay-distill-baseline`.

Invocation note:

1. Calling `powershell -File scripts\compare-tracecalc-run.ps1` from this environment invoked Windows PowerShell and failed because that shell lacks the script's `ConvertFrom-Json -Depth` support.
2. The successful comparison used the current PowerShell 7 shell with `-RepoRoot` explicit.

## 3. Covered Scenario Slice

| Scenario | Focus | First W033 claim coverage |
|---|---|---|
| `tc_accept_publish_001` | candidate-publication boundary; accept-publish | candidate is not publication; accepted publication |
| `tc_reject_no_publish_001` | reject-is-no-publish; typed reject | reject is no-publish |
| `tc_pinned_view_stability_001` | pinned view; stable reader | pinned-reader view stability |
| `tc_dynamic_dep_switch_001` | dynamic dependency; runtime effects | runtime dependency visibility first floor |
| `tc_overlay_retention_001` | overlay retention; runtime effects | protected overlay retention first floor |
| `tc_scale_chain_seed_001` | scale seed; generator metadata | measurement/counter seed only; not semantic scale proof |
| `tc_verify_clean_no_publish_001` | verify-clean; no publication | verified-clean no-publication transition |
| `tc_multinode_dag_publish_001` | multi-node DAG; topo scheduling | deterministic topo/DAG publication floor |
| `tc_publication_fence_reject_001` | publication-fence reject; typed reject | stale/incompatible fence reject floor |
| `tc_artifact_token_reject_001` | artifact-token reject; typed reject | typed artifact-token reject and fallback floor |
| `tc_fallback_reentry_001` | fallback reentry; overlay reuse | fallback/reentry and overlay reuse first floor |
| `tc_cycle_region_reject_001` | cycle region; SCC handling | cycle-region typed reject floor |

## 4. Self-Check Assertions

The first slice establishes these assertions for the listed scenarios:

1. The TraceCalc manifest selection is stable against the baseline.
2. Every scenario result state is `passed`.
3. Oracle and engine result states match for every scenario.
4. Oracle and engine published views match for every scenario under the current comparator.
5. Oracle and engine pinned views match for every scenario under the current comparator.
6. Oracle and engine reject projections match for every scenario under the current comparator.
7. Oracle and engine trace label/family projections match for every scenario under the current comparator.
8. Oracle and engine semantic counters match for every scenario under the current comparator.
9. Replay appliance bundle validation reports no missing paths and no degraded capture.
10. The new run matches `w019-replay-distill-baseline` for the current comparison projection.

## 5. Current Oracle Gaps

This self-check does not yet cover:

1. production/core-engine comparison outside the TraceCalc crate's paired reference/engine machines,
2. direct OxFml fixture replay integration,
3. `LET`/`LAMBDA` carrier behavior,
4. broad formula-function semantics,
5. large-scale performance semantics,
6. full dynamic-dependency soft-reference families beyond current TraceCalc seed scenarios,
7. Stage 2 concurrency or async publication interleavings,
8. grid/spill/host/UI/file-adapter behavior.

These are not failures of the self-check. They are uncovered behavior and remain later W033 or successor lanes.

## 6. Downstream Conformance Inputs

`calc-uri.9` should use this self-check as the current TraceCalc authority floor for the first production/core-engine conformance slice.

Minimum inputs for that comparison:

1. run id `w033-tracecalc-oracle-self-check-001`,
2. scenario list from `manifest_selection.json`,
3. observable surface from `W033_TRACECALC_REFINEMENT_PACKET.md`,
4. conformance baseline `conformance/engine_diff.json`,
5. replay bundle validation status from `replay-appliance/validation/bundle_validation.json`,
6. parity check against `w019-replay-distill-baseline`.

## 7. Status

- execution_state: `tracecalc_oracle_self_check_first_slice_recorded`
- scope_completeness: `scope_partial`
- target_completeness: `target_partial`
- integration_completeness: `partial`
- open_lanes:
  - production/core-engine conformance first slice is not yet authored
  - LET/LAMBDA carrier behavior remains outside this self-check slice
  - OxFml replay/witness bridge has not yet consumed this run
  - pack/capability mapping has not yet consumed this run
