# W036 Continuous Assurance Operation And History Window

Status: `calc-rqq.7_continuous_assurance_history_validated`
Workset: `W036`
Parent epic: `calc-rqq`
Bead: `calc-rqq.7`

## 1. Purpose

This packet evolves the W035 continuous-assurance gate into a W036 continuous-assurance operation evidence lane.

The target is machine-readable simulated multi-run history, regression thresholds, quarantine/alert policy, and semantic-first acceptance criteria. It is not an operated continuous-assurance service, continuous cross-engine differential service, timing-based correctness proof, pack C5 promotion, or Stage 2 policy promotion.

## 2. Authority Inputs Reviewed

| Input | Role in this bead |
|---|---|
| `docs/spec/core-engine/w035-formalization/W035_CONTINUOUS_ASSURANCE_AND_CROSS_ENGINE_DIFFERENTIAL_GATE.md` | predecessor gate packet |
| `docs/test-runs/core-engine/continuous-assurance/w035-continuous-assurance-gate-001/` | W035 schedule, differential gate, and no-promotion source |
| `docs/test-runs/core-engine/metamorphic-scale-semantic-binding/w034-continuous-scale-gate-binding-001/` | semantic scale-binding source |
| `docs/test-runs/core-engine/tracecalc-reference-machine/w036-tracecalc-coverage-closure-001/` | W036 TraceCalc coverage source |
| `docs/test-runs/core-engine/implementation-conformance/w036-implementation-conformance-closure-001/` | W036 implementation-conformance source |
| `docs/test-runs/core-engine/tla/w036-stage2-partition-001/` | W036 bounded TLA model source |
| `docs/test-runs/core-engine/independent-conformance/w036-independent-diversity-differential-001/` | W036 diversity source |
| `docs/test-runs/core-engine/cross-engine-differential/w036-independent-diversity-differential-001/` | W036 cross-engine differential source |
| `docs/spec/core-engine/w036-formalization/W036_RESIDUAL_COVERAGE_AND_PROMOTION_BLOCKER_LEDGER.md` | W036 obligation `W036-OBL-014` |
| `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md` | current inbound OxFml observation ledger |

## 3. Implementation Surface

| Surface | Change |
|---|---|
| `src/oxcalc-tracecalc/src/continuous_assurance.rs` | adds W036-mode evidence intake, simulated history, thresholds, quarantine policy, and validation metadata |
| `src/oxcalc-tracecalc-cli/src/main.rs` | reports W036 history, threshold, and quarantine counts for W036 continuous-assurance runs |
| `src/oxcalc-tracecalc/src/pack_capability.rs` | hardens test-only temp repo naming with an atomic counter after default parallel validation exposed a timestamp collision risk |
| `docs/test-runs/core-engine/continuous-assurance/w036-continuous-assurance-operation-001/` | checked-in W036 continuous-assurance operation/history evidence root |

No runtime evaluator, coordinator, dependency graph, soft-reference, recalc, publication, formal proof/model, pack-decision, or OxFml behavior changes in this bead.

## 4. Evidence Summary

Run id: `w036-continuous-assurance-operation-001`

| Metric | Value |
|---|---:|
| Source evidence rows | 11 |
| Scheduled lanes | 4 |
| Cross-engine gate rows | 6 |
| History-window rows | 6 |
| Regression threshold rules | 7 |
| Quarantine/alert rules | 7 |
| Simulated multi-run rows | 6 |
| Missing artifacts | 0 |
| Unexpected mismatches | 0 |
| No-promotion reasons | 11 |
| Operated continuous service promoted | no |
| Timing used as correctness evidence | no |

Primary artifacts:

1. `docs/test-runs/core-engine/continuous-assurance/w036-continuous-assurance-operation-001/run_summary.json`
2. `docs/test-runs/core-engine/continuous-assurance/w036-continuous-assurance-operation-001/evidence/source_evidence_index.json`
3. `docs/test-runs/core-engine/continuous-assurance/w036-continuous-assurance-operation-001/schedule/continuous_assurance_schedule.json`
4. `docs/test-runs/core-engine/continuous-assurance/w036-continuous-assurance-operation-001/history/assurance_history_window.json`
5. `docs/test-runs/core-engine/continuous-assurance/w036-continuous-assurance-operation-001/thresholds/regression_thresholds.json`
6. `docs/test-runs/core-engine/continuous-assurance/w036-continuous-assurance-operation-001/alerts/quarantine_policy.json`
7. `docs/test-runs/core-engine/continuous-assurance/w036-continuous-assurance-operation-001/operation/simulated_multi_run_evidence.json`
8. `docs/test-runs/core-engine/continuous-assurance/w036-continuous-assurance-operation-001/differentials/cross_engine_differential_gate.json`
9. `docs/test-runs/core-engine/continuous-assurance/w036-continuous-assurance-operation-001/decision/continuous_assurance_decision.json`
10. `docs/test-runs/core-engine/continuous-assurance/w036-continuous-assurance-operation-001/replay-appliance/validation/bundle_validation.json`

## 5. History Window

The W036 history window is simulated from checked-in evidence epochs:

| Window row | Source | State |
|---|---|---|
| 1 | W034 scale semantic binding | semantic scale binding present; no continuous-scale promotion |
| 2 | W035 continuous gate | gate present; no service promotion |
| 3 | W036 TraceCalc coverage closure | coverage criteria present; no full oracle claim |
| 4 | W036 implementation conformance closure | closure actions present; no declared gap promoted as a match |
| 5 | W036 TLA Stage 2 partition | bounded model evidence present; no Stage 2 promotion |
| 6 | W036 independent differential harness | deterministic differential harness present; no continuous service promotion |

The window records `continuous_service_present=false` and `timing_correctness_role=measurement_only_not_correctness_evidence`.

## 6. Threshold And Quarantine Policy

The regression threshold packet defines semantic-first gates:

1. missing artifacts must be zero,
2. unexpected mismatches must be zero,
3. failed oracle/conformance/TLC rows must be zero,
4. declared gaps must not be promoted as matches,
5. the W036 history window must contain at least six rows,
6. simulated history must not promote an operated service,
7. timing regression is report-only unless semantic thresholds fail.

The quarantine/alert policy quarantines missing evidence, mismatches, failed semantic rows, declared-gap-as-match errors, and unsupported promotion flags. It records W073 formatting payload mismatch as an OxFml handoff/watch alert and treats timing-only changes as performance alerts rather than correctness failures.

## 7. Schedule Lane

W036 keeps the three W035 lanes and adds `continuous.history.thresholds`.

The new lane requires:

1. `history/assurance_history_window.json`,
2. `thresholds/regression_thresholds.json`,
3. `alerts/quarantine_policy.json`,
4. `operation/simulated_multi_run_evidence.json`.

Its cadence is `simulated_from_checked_in_successor_evidence_until_runner_exists`, which prevents the packet from implying an operated runner.

## 8. Promotion Consequence

This bead satisfies the W036 target of defining or simulating multi-run history, thresholds, quarantine/alert policy, and semantic-first acceptance.

It carries these no-promotion consequences:

1. no operated regression runner,
2. simulated history is not a running service,
3. quarantine policy is not enforced by an alert service,
4. continuous cross-engine differential service is not operated,
5. performance/timing is not correctness proof,
6. full TraceCalc oracle coverage remains unpromoted,
7. full optimized/core-engine conformance remains unpromoted,
8. full independent evaluator diversity remains unpromoted,
9. pack C5 remains unpromoted,
10. Stage 2 scheduler policy remains unpromoted,
11. full Lean/TLA verification remains unpromoted.

## 9. OxFml Watch

No OxFml handoff is filed by this bead.

This bead constructs no conditional-formatting payloads. The quarantine policy records a watch rule that any exercised W073 aggregate or visualization conditional-formatting payload must use `typed_rule`, not W072-style `thresholds`.

Reviewed inbound observations: `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md`.

## 10. Semantic-Equivalence Statement

This bead adds W036 continuous-assurance artifact emission, CLI reporting, checked artifacts, and documentation only.

It does not change coordinator scheduling, invalidation strategy, dependency graph construction, soft-reference update behavior, recalc behavior, publication semantics, reject policy, overlay lifecycle behavior, TraceCalc execution semantics, TreeCalc/CoreEngine runtime behavior, Lean/TLA model semantics, pack-decision logic, or OxFml/OxFunc evaluator behavior.

Observable runtime behavior is invariant under this bead. The new W036 artifacts classify existing evidence and write history/threshold/quarantine packets; they do not change executable calculator behavior.

## 11. Verification

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | passed |
| `$env:CARGO_TARGET_DIR='target\w036-test-target'; cargo test -p oxcalc-tracecalc continuous_assurance` | passed; 2 tests |
| `$env:CARGO_TARGET_DIR='target\w036-test-target'; cargo run -p oxcalc-tracecalc-cli -- continuous-assurance w036-continuous-assurance-operation-001` | passed; emitted 11 source rows, 6 differential rows, 6 history rows, 7 threshold rules, 7 quarantine rules, and 11 no-promotion reasons |
| `$env:CARGO_TARGET_DIR='target\w036-test-target'; cargo test -p oxcalc-tracecalc` | passed; 20 tests |
| `$env:CARGO_TARGET_DIR='target\w036-test-target'; cargo test -p oxcalc-tracecalc-cli` | passed; 0 tests |
| `$env:CARGO_TARGET_DIR='target\w036-test-target'; cargo clippy -p oxcalc-tracecalc -p oxcalc-tracecalc-cli --all-targets -- -D warnings` | passed |
| `Get-ChildItem -Path docs/test-runs/core-engine/continuous-assurance/w036-continuous-assurance-operation-001 -Recurse -Filter *.json | ForEach-Object { Get-Content -Raw $_.FullName | ConvertFrom-Json | Out-Null }` | passed |
| `scripts/check-worksets.ps1` | passed |
| `br dep cycles --json` | passed; `count=0` |
| `git diff --check` | passed; CRLF normalization warnings only |

Validation note: the first broad default `cargo test -p oxcalc-tracecalc` attempt exposed a pack-capability test-only temp directory race. The pack tests passed in isolation and serially; this bead hardens the temp repo naming and the default broad run now passes.

## 12. Pre-Closure Verification Checklist

| # | Check | Result |
|---|---|---|
| 1 | Spec text and realization notes updated for all in-scope items? | yes; this packet, W036 status surfaces, W036 residual ledger, spec index, and feature-map surfaces record the continuous-assurance evidence |
| 2 | Pack expectations updated for affected packs? | yes; pack remains unpromoted and `calc-rqq.8` still owns reassessment |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes; this bead consumes deterministic checked-in predecessor/successor evidence and emits deterministic history, threshold, quarantine, decision, and validation artifacts |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes; Section 10 states no runtime strategy changed |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes; no OxFml-owned seam contradiction or handoff trigger appeared |
| 6 | All required tests pass? | yes; see Section 11 |
| 7 | No known semantic gaps remain in declared scope? | yes for this target; operated-service, alert-executor, timing, pack, Stage 2, and full-verification limits are explicit |
| 8 | Completion language audit passed? | yes; no operated service, full verification, pack, Stage 2, or timing-correctness promotion is claimed |
| 9 | `WORKSET_REGISTER.md` updated when ordered workset truth changed? | not applicable; ordered workset truth is unchanged |
| 10 | `IN_PROGRESS_FEATURE_WORKLIST.md` updated when feature-map truth changed? | yes; IP-04 points at this W036 continuous-assurance evidence |
| 11 | execution-state blocker surface updated? | yes; `.beads/` records `calc-rqq.7` execution state and later closure evidence |

## 13. Completion Claim Self-Audit

| Step | Audit result |
|---|---|
| Scope re-read | pass; `calc-rqq.7` asks for operated or simulated continuous-assurance evidence with schedule, history-window criteria, thresholds, quarantine/alert policy, and semantic-first treatment of timing |
| Gate criteria re-read | pass; machine-readable simulated multi-run evidence exists and timing remains subordinate to semantic correctness |
| Silent scope reduction check | pass; the packet explicitly records that evidence is simulated, not an operated service |
| "Looks done but is not" pattern check | pass; simulated history and policy artifacts are not represented as a running service, pack-grade replay, full verification, or correctness proof from timing |
| Result | pass for the `calc-rqq.7` continuous-assurance operation/history target |

## 14. Three-Axis Report

- execution_state: `calc-rqq.7_continuous_assurance_history_validated`
- scope_completeness: `scope_partial`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- open_lanes:
  - `calc-rqq.8` and `calc-rqq.9` remain open
  - operated continuous-assurance service remains partial because W036 emits simulated multi-run evidence, not a scheduled runner
  - continuous cross-engine differential service remains partial because W036 emits deterministic harness/history artifacts, not an operated service
  - alert/quarantine execution remains partial because W036 defines policy, not an enforcing alert dispatcher
  - full TraceCalc oracle coverage, full optimized/core-engine verification, full independent evaluator diversity, direct OxFml evaluator re-execution, pack-grade replay, full Lean/TLA verification, and Stage 2 policy remain unpromoted
