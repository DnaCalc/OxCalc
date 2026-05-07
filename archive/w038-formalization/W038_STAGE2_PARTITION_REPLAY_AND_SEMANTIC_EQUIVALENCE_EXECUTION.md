# W038 Stage 2 Partition Replay And Semantic-Equivalence Execution

Status: `calc-zsr.5_stage2_partition_replay_validated`
Workset: `W038`
Parent epic: `calc-zsr`
Bead: `calc-zsr.5`

## 1. Purpose

This packet executes the W038 bounded Stage 2 replay slice.

The target is not to promote a production Stage 2 scheduler or partition policy. The target is to replace the W037 "deterministic partition replay absent" blocker with bounded baseline-versus-Stage-2 replay evidence, partition-order permutation evidence, and an exact remaining blocker register for the production evidence still missing.

The latest OxFml formatting update is carried in this packet as a watch/input-contract row: W073 aggregate and visualization conditional-formatting metadata is `typed_rule`-only for the W073 families, and legacy bounded `thresholds` strings are not interpreted for those families.

## 2. Authority Inputs Reviewed

| Input | Role |
|---|---|
| `docs/worksets/W038_CORE_FORMALIZATION_RELEASE_GRADE_CLOSURE_HARDENING.md` | W038 gate model and `calc-zsr.5` target |
| `docs/spec/core-engine/w038-formalization/W038_RESIDUAL_RELEASE_GRADE_OBLIGATION_LEDGER_AND_OBJECTIVE_MAP.md` | W038 obligations `W038-OBL-009`, `W038-OBL-011`, `W038-OBL-012`, and `W038-OBL-013` |
| `docs/test-runs/core-engine/stage2-criteria/w037-stage2-deterministic-replay-criteria-001/` | W037 Stage 2 criteria and no-promotion decision |
| `docs/test-runs/core-engine/tla/w036-stage2-partition-001/run_summary.json` | bounded TLA Stage 2 partition model evidence |
| `docs/test-runs/core-engine/tracecalc-reference-machine/w037-tracecalc-observable-closure-001/` | TraceCalc observable reference rows |
| `docs/test-runs/core-engine/treecalc-local/w037-optimized-core-conformance-treecalc-001/` | optimized/core local TreeCalc rows with dependency graphs |
| `docs/test-runs/core-engine/upstream-host/w037-direct-oxfml-evaluator-001/` | direct OxFml evaluator row for W073 typed conditional-formatting guard |
| `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md` plus current OxFml W073 diffs | inbound formatting update: W073 aggregate/visualization metadata is typed-only |

## 3. Artifact Surface

Run id: `w038-stage2-partition-replay-001`

| Artifact | Result |
|---|---|
| `src/oxcalc-tracecalc/src/stage2_replay.rs` | W038 bounded Stage 2 replay runner and validator |
| `archive/test-runs-core-engine-w038-w045/stage2-replay/w038-stage2-partition-replay-001/run_summary.json` | 5 partition replay rows, 6 permutation rows, 5 invariant rows, 1 formatting watch row, 3 exact blockers, 0 failed rows |
| `archive/test-runs-core-engine-w038-w045/stage2-replay/w038-stage2-partition-replay-001/partition_replay_matrix.json` | baseline-versus-Stage-2 replay projection matrix |
| `archive/test-runs-core-engine-w038-w045/stage2-replay/w038-stage2-partition-replay-001/partition_order_permutation_replay.json` | admissible partition-order permutation rows |
| `archive/test-runs-core-engine-w038-w045/stage2-replay/w038-stage2-partition-replay-001/semantic_equivalence_report.json` | bounded semantic-equivalence statement and W037 replay-blocker disposition |
| `archive/test-runs-core-engine-w038-w045/stage2-replay/w038-stage2-partition-replay-001/stage2_exact_blocker_register.json` | 3 remaining exact Stage 2 blockers |
| `archive/test-runs-core-engine-w038-w045/stage2-replay/w038-stage2-partition-replay-001/promotion_decision.json` | Stage 2 policy remains unpromoted |
| `archive/test-runs-core-engine-w038-w045/stage2-replay/w038-stage2-partition-replay-001/source_evidence_index.json` | source artifact index including W073 formatting guard |
| `archive/test-runs-core-engine-w038-w045/stage2-replay/w038-stage2-partition-replay-001/validation.json` | validation status `w038_stage2_replay_valid` |

## 4. Replay Profiles

| Row | Source | Stage 2 value |
|---|---|---|
| `w038_stage2_tracecalc_accept_publish_reference` | TraceCalc accept/publish row | binds the simplest accepted publication surface as replay reference |
| `w038_stage2_treecalc_independent_partition_permutation` | TreeCalc independent left/top/check row | proves the independent left/top partitions can swap before the check partition while preserving observable results |
| `w038_stage2_treecalc_dynamic_dependency_resolution` | TreeCalc dynamic reference row | carries dynamic/soft dependency binding through the partition replay surface |
| `w038_stage2_tracecalc_dynamic_dependency_reference` | TraceCalc dynamic dependency release row | binds the TraceCalc dynamic-dependency observable reference surface |
| `w038_stage2_w073_typed_formatting_guard` | direct OxFml upstream-host row | carries W073 typed-only conditional-formatting semantics, retained legacy threshold text, and distinct absent `format_delta`/`display_delta` observations |

The runner materializes stable observable projections and validates dependency-order admissibility for the TreeCalc dependency-graph-backed rows. It compares baseline projection, declared Stage 2 projection, and every listed partition-order permutation for equality over the declared observable surfaces.

## 5. Disposition Summary

| Lane | W038 disposition | Evidence consequence |
|---|---|---|
| W037 deterministic partition replay absent | narrowed by bounded replay evidence | baseline-versus-Stage-2 replay exists for 5 declared profiles |
| partition-order permutation replay | bounded evidence present | 6 permutation rows are valid; 1 row is a nontrivial independent-order swap |
| observable-result invariance | bounded evidence present | all 5 declared profiles preserve observable projections |
| W073 formatting update | watch/input-contract row carried | typed `rank` metadata drives the formatting result; legacy `thresholds` text is retained as input text but not interpreted for that family |
| production partition analyzer soundness | exact remaining blocker | no production analyzer proof or corpus-wide scheduler proof is claimed |
| operated cross-engine Stage 2 differential service | exact remaining blocker | `calc-zsr.6` owns operated service evidence |
| pack-grade Stage 2 replay governance | exact remaining blocker | `calc-zsr.8` owns pack-grade replay/C5 reassessment |

## 6. Semantic-Equivalence Statement

For the bounded profiles in `w038-stage2-partition-replay-001`, observable results are invariant between the baseline schedule, the declared Stage 2 partition schedule, and every admissible partition-order permutation emitted by the runner.

This does not change coordinator scheduling, invalidation strategy, dependency graph construction, soft-reference update behavior, recalc behavior, publication semantics, reject policy, overlay lifecycle behavior, TraceCalc semantics, TreeCalc runtime behavior, OxFml evaluator behavior, OxFunc kernels, pack/C5 capability policy, operated service behavior, or alert/quarantine policy.

Stage 2 production scheduler and partition policy remain unpromoted. A future promotion needs production partition-analyzer soundness, operated cross-engine service evidence, and pack-grade replay governance for the claimed scope.

## 7. Verification

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | passed |
| `cargo test -p oxcalc-tracecalc stage2_replay -- --nocapture` | passed; 1 test |
| `cargo test -p oxcalc-tracecalc-cli` | passed; 0 tests |
| `cargo run -p oxcalc-tracecalc-cli -- stage2-replay w038-stage2-partition-replay-001` | passed; emitted W038 Stage 2 replay artifacts |
| `scripts\run-tlc.ps1` W036 Stage 2 partition config set with `-metadir target\tla-w038-stage2\...` | passed; 5 configs |
| JSON parse for `archive/test-runs-core-engine-w038-w045/stage2-replay/w038-stage2-partition-replay-001/*.json` | passed |
| `scripts/check-worksets.ps1` | passed after bead closure |
| `br ready --json` | passed after bead closure; next ready bead is `calc-zsr.6` |
| `br dep cycles --json` | passed with `count: 0` |
| `git diff --check` | passed; CRLF normalization warnings only |

## 8. Pre-Closure Verification Checklist

| # | Check | Result |
|---|---|---|
| 1 | Spec text and realization notes updated for all in-scope items? | yes; this packet, W038 README/status surfaces, machine-readable artifacts, and feature map record the slice |
| 2 | Pack expectations updated for affected packs? | yes; pack-grade Stage 2 replay governance remains unpromoted and `calc-zsr.8` owns reassessment |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes; `w038-stage2-partition-replay-001` emits deterministic partition replay, permutation replay, semantic-equivalence, blocker, and validation artifacts |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes; Section 6 states bounded observable invariance and no production Stage 2 policy promotion |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes; OxFml W073 formatting update is already aligned with OxCalc's typed-only watch row, and no OxFml-owned contract defect is exposed |
| 6 | All required tests pass? | yes; see Section 7 |
| 7 | No known semantic gaps remain in declared scope? | yes for the bounded replay target; production partition analyzer, operated service, and pack-grade replay remain exact blockers |
| 8 | Completion language audit passed? | yes; the packet limits claims to bounded replay evidence and keeps Stage 2 policy unpromoted |
| 9 | `WORKSET_REGISTER.md` updated when ordered workset truth changed? | not applicable; ordered workset truth is unchanged |
| 10 | `IN_PROGRESS_FEATURE_WORKLIST.md` updated when feature-map truth changed? | yes; IP-04 records the W038 Stage 2 replay state |
| 11 | execution-state blocker surface updated? | yes; `.beads/` records `calc-zsr.5` closure and `calc-zsr.6` readiness |

## 9. Completion Claim Self-Audit

| Step | Audit result |
|---|---|
| Scope re-read | pass; `calc-zsr.5` asks for baseline-versus-Stage-2 partition replay and semantic-equivalence evidence, or exact blockers for production partition soundness |
| Gate criteria re-read | pass; bounded replay and permutation evidence exists before any scheduler/policy promotion claim, and production blockers remain exact |
| Silent scope reduction check | pass; the packet explicitly narrows the claim to declared bounded profiles and does not promote production Stage 2 scheduling |
| "Looks done but is not" pattern check | pass; no simulated replay row is reported as production scheduler soundness, operated service evidence, pack-grade replay, C5, or release-grade verification |
| Result | pass for the `calc-zsr.5` bounded Stage 2 replay target |

## 10. Three-Axis Report

- execution_state: `calc-zsr.5_stage2_partition_replay_validated`
- scope_completeness: `scope_partial`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- open_lanes:
  - production Stage 2 partition analyzer soundness remains unpromoted
  - operated assurance, alert/quarantine, and cross-engine service remain open under `calc-zsr.6`
  - independent evaluator diversity and OxFml seam watch closure remain open under `calc-zsr.7`
  - pack-grade replay governance, C5, and W038 release decision remain open
  - full Lean/TLA verification and release-grade verification remain unpromoted
