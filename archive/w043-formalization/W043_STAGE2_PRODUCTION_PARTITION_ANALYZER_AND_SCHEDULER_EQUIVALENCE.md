# W043 Stage 2 Production Partition Analyzer And Scheduler Equivalence

Status: `calc-2p3.5_stage2_scheduler_equivalence_validated`

Bead: `calc-2p3.5`

Run id: `w043-stage2-production-partition-analyzer-scheduler-equivalence-001`

## Purpose

This packet deepens the W043 Stage 2 lane after the W043 Lean/TLA proof/model packet.

The narrow new evidence is a W043 Stage 2 policy classifier that binds:

1. the W042 Stage 2 analyzer and pack-grade equivalence predecessor packet,
2. bounded baseline-versus-Stage-2 replay and partition-order permutation evidence,
3. observable-result invariance rows for declared profiles,
4. W043 optimized/core snapshot and capability counterpart evidence,
5. W043 Rust dynamic-addition and dynamic-release refinement evidence,
6. W043 Lean/TLA bounded model rows and exact fairness boundary,
7. current W073 typed-only formatting intake,
8. declared-profile scheduler-equivalence evidence,
9. no-proxy promotion guards.

It does not promote Stage 2 production policy, pack-grade replay, C5, release-grade verification, production partition-analyzer soundness, scheduler fairness, operated cross-engine differential service, retained-witness lifecycle, or general OxFunc kernels.

## Evidence Surfaces

| Artifact | Purpose |
|---|---|
| `formal/lean/OxCalc/CoreEngine/W043Stage2ProductionPartitionAnalyzerAndSchedulerEquivalence.lean` | checked Lean predicate for W043 Stage 2 production-partition and scheduler-equivalence guards |
| `archive/test-runs-core-engine-w038-w045/stage2-replay/w043-stage2-production-partition-analyzer-scheduler-equivalence-001/run_summary.json` | records row counts, register paths, promotion guards, and no-failure summary |
| `archive/test-runs-core-engine-w038-w045/stage2-replay/w043-stage2-production-partition-analyzer-scheduler-equivalence-001/w043_stage2_policy_gate_register.json` | records the 20 W043 Stage 2 policy rows |
| `archive/test-runs-core-engine-w038-w045/stage2-replay/w043-stage2-production-partition-analyzer-scheduler-equivalence-001/w043_production_partition_analyzer_register.json` | records production-partition analyzer inputs and retained soundness blockers |
| `archive/test-runs-core-engine-w038-w045/stage2-replay/w043-stage2-production-partition-analyzer-scheduler-equivalence-001/w043_scheduler_equivalence_register.json` | records declared-profile scheduler-equivalence rows and retained unbounded fairness blocker |
| `archive/test-runs-core-engine-w038-w045/stage2-replay/w043-stage2-production-partition-analyzer-scheduler-equivalence-001/w043_pack_grade_equivalence_register.json` | records declared-profile pack-equivalence inputs and retained pack-grade blockers |
| `archive/test-runs-core-engine-w038-w045/stage2-replay/w043-stage2-production-partition-analyzer-scheduler-equivalence-001/w043_stage2_exact_blocker_register.json` | records 6 exact Stage 2 and pack-grade blockers |
| `archive/test-runs-core-engine-w038-w045/stage2-replay/w043-stage2-production-partition-analyzer-scheduler-equivalence-001/promotion_decision.json` | records no Stage 2 policy or pack-grade replay promotion |
| `archive/test-runs-core-engine-w038-w045/stage2-replay/w043-stage2-production-partition-analyzer-scheduler-equivalence-001/source_evidence_index.json` | indexes W042 Stage 2, W043 optimized/core, W043 Rust, W043 Lean/TLA, W073, and W043 obligation sources |
| `archive/test-runs-core-engine-w038-w045/stage2-replay/w043-stage2-production-partition-analyzer-scheduler-equivalence-001/validation.json` | records validation status and zero validation failures |

## Row Disposition

Observed counts:

1. 20 policy rows.
2. 14 satisfied declared-profile rows.
3. 5 partition replay rows carried.
4. 6 permutation replay rows carried.
5. 5 observable-invariance rows carried.
6. 1 formatting watch row carried.
7. 2 automatic dynamic-transition rows.
8. 6 exact remaining blockers.
9. 0 failed rows.

Satisfied declared-profile inputs:

1. W042 Stage 2 predecessor policy packet.
2. bounded baseline-versus-Stage-2 replay.
3. bounded partition-order permutation replay.
4. observable-result invariance for declared profiles.
5. W043 automatic dynamic addition refinement input.
6. W043 automatic dynamic release refinement input.
7. W043 snapshot-fence counterpart input.
8. W043 capability-view counterpart input.
9. bounded partition-analyzer predicate input.
10. W043 Lean/TLA bounded model input.
11. W073 typed-only formatting guard.
12. declared-profile pack replay equivalence.
13. no-proxy promotion guard.
14. declared-profile scheduler equivalence.

Exact blockers:

1. broader dynamic dependency-transition coverage,
2. full production partition-analyzer soundness,
3. scheduler fairness and unbounded model coverage,
4. operated cross-engine Stage 2 differential service,
5. retained-witness lifecycle and retention SLO,
6. pack-grade replay governance.

## Semantic-Equivalence Statement

This packet changes Stage 2 classification, checked Lean predicate text, evidence generation, and documentation only. It does not change optimized/core recalc behavior, TreeCalc invalidation behavior, TraceCalc behavior, publication policy, scheduling policy, OxFml runtime behavior, or OxFunc callable kernels.

Observable engine results are therefore invariant under this packet. The packet records that observable-result and scheduler equivalence are evidenced for declared W043 Stage 2 profiles only; production scheduling and pack-grade replay remain unpromoted until the retained blockers are discharged.

## Validation

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | passed |
| `lean formal/lean/OxCalc/CoreEngine/W043Stage2ProductionPartitionAnalyzerAndSchedulerEquivalence.lean` | passed |
| `lean formal/lean/OxCalc/CoreEngine/W043LeanTlaFullVerificationAndFairness.lean` | passed |
| `lean formal/lean/OxCalc/CoreEngine/W042Stage2ProductionAnalyzerAndPackGradeEquivalence.lean` | passed |
| `cargo test -p oxcalc-tracecalc stage2_replay_runner_writes_w043_scheduler_equivalence_without_promotion -- --nocapture` | passed |
| `cargo run -p oxcalc-tracecalc-cli -- stage2-replay w043-stage2-production-partition-analyzer-scheduler-equivalence-001` | passed; emitted 20 policy rows with 0 failed rows |
| JSON parse for W043.5 Stage 2 artifacts | passed |
| `cargo test -p oxcalc-tracecalc stage2_replay -- --nocapture` | passed |
| `cargo test -p oxcalc-tracecalc` | passed; 62 tests |
| `scripts/check-worksets.ps1` | passed after bead closure |
| `br ready --json` | passed after bead closure; next ready bead is `calc-2p3.6` |
| `br dep cycles --json` | passed with `count: 0` |
| `git diff --check` | passed after bead closure; CRLF normalization warnings only |

## Status Report

- execution_state: `calc-2p3.5_stage2_scheduler_equivalence_validated`
- scope_completeness: `scope_partial`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- open_lanes:
  - `calc-2p3.6` operated assurance, retained-history, witness, SLO, and alert service
  - broader dynamic dependency-transition coverage remains partial
  - full production partition-analyzer soundness remains blocked
  - scheduler fairness and unbounded model coverage remain exact blockers
  - operated cross-engine Stage 2 differential service remains blocked
  - retained-witness lifecycle and retention SLO remain blocked
  - pack-grade replay governance remains blocked
  - Stage 2 production policy, pack-grade replay, C5, release-grade verification, operated services, independent evaluator breadth, broad OxFml display/publication, public migration, callable metadata projection, callable carrier sufficiency, registered-external callable projection, provider-failure/callable-publication semantics, and general OxFunc kernels remain unpromoted

## Pre-Closure Verification Checklist

| # | Check | Result |
|---|---|---|
| 1 | Spec text and realization notes updated for all in-scope items? | yes; this packet, W043 README/status surfaces, feature map, Lean predicate, and Stage 2 artifacts record the slice |
| 2 | Pack expectations updated for affected packs? | yes; pack-grade replay and C5 remain unpromoted and `calc-2p3.9` owns reassessment |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes; W042 Stage 2, W043 optimized/core, W043 Rust, W043 Lean/TLA, and W043 Stage 2 artifacts are bound |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes; no runtime strategy changed and declared-profile observable-result/scheduler equivalence remains bounded |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes; W073 is carried as typed-only guard and no OxCalc handoff trigger exists |
| 6 | All required tests pass? | yes; see Validation |
| 7 | No known semantic gaps remain in declared scope? | yes for this W043.5 classification target; broader exact blockers are explicit |
| 8 | Completion language audit passed? | yes; no Stage 2, pack-grade replay, C5, release-grade, production analyzer, fairness, service, retained-witness, OxFml breadth, callable, or general OxFunc promotion is claimed |
| 9 | `WORKSET_REGISTER.md` updated when ordered workset truth changed? | not applicable; ordered workset truth is unchanged |
| 10 | `IN_PROGRESS_FEATURE_WORKLIST.md` updated when feature-map truth changed? | yes; IP-04 records the W043 Stage 2 update |
| 11 | execution-state blocker surface updated? | yes; `.beads/` records `calc-2p3.5` closure and `calc-2p3.6` readiness |

## Completion Claim Self-Audit

| Step | Audit result |
|---|---|
| Scope re-read | pass; `calc-2p3.5` asks for Stage 2 production partition-analyzer and scheduler-equivalence evidence without promotion from bounded evidence alone |
| Gate criteria re-read | pass; declared-profile rows, exact blockers, promotion guards, and pack-grade blockers are separated |
| Silent scope reduction check | pass; production analyzer soundness, scheduler fairness, unbounded coverage, operated service, retained-witness lifecycle, pack governance, Stage 2 policy, and C5 remain explicit open lanes |
| "Looks done but is not" pattern check | pass; bounded replay, declared-profile scheduler equivalence, and declared-profile pack equivalence are not reported as production Stage 2 policy or pack-grade replay |
| Result | pass for the `calc-2p3.5` target |
