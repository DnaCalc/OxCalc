# W040 Lean TLA Full-Verification Discharge Tranche

Status: `calc-tv5.4_lean_tla_discharge_classified_no_promotion`
Workset: `W040`
Parent epic: `calc-tv5`
Bead: `calc-tv5.4`

## 1. Purpose

This packet attacks the W040 Lean/TLA full-verification discharge target.

The result is deliberately non-promoting. It adds a checked Lean classification file, a W040 formal-assurance runner profile, generated proof/model registers, and a fresh W040-local TLC check of the five Stage 2 partition configs named by the model-bound register.

The target is not to claim full Lean verification, full TLA verification, Rust-engine totality, Stage 2 production policy, pack-grade replay, C5, release-grade verification, broad OxFml display/publication closure, callable metadata projection, or general OxFunc kernel ownership. The target is to replace a broad Lean/TLA gap with direct proof/model rows, bounded model evidence, accepted boundaries, and exact blockers before `calc-tv5.5` starts.

## 2. Authority Inputs Reviewed

| Input | Role |
|---|---|
| `docs/test-runs/core-engine/formal-inventory/w037-proof-model-closure-001/run_summary.json` | retained Lean/TLA inventory; records 12 Lean files checked, 11 routine TLC configs, and zero failed TLC configs |
| `docs/test-runs/core-engine/formal-inventory/w037-proof-model-closure-001/tla_inventory.json` | retained routine bounded TLA model inventory |
| `archive/test-runs-core-engine-w038-w045/release-grade-ledger/w040-direct-verification-obligation-map-001/direct_verification_obligation_map.json` | W040 obligations `W040-OBL-008`, `W040-OBL-009`, and `W040-OBL-020` for Lean/TLA discharge and LET/LAMBDA carrier scope |
| `archive/test-runs-core-engine-w038-w045/formal-assurance/w040-rust-totality-refinement-proof-tranche-001/run_summary.json` | preceding W040 Rust proof/refinement classification packet |
| `archive/test-runs-core-engine-w038-w045/formal-assurance/w040-rust-totality-refinement-proof-tranche-001/w040_rust_exact_blocker_register.json` | Rust totality/refinement exact blockers that remain proof dependencies |
| `formal/lean/OxCalc/CoreEngine/W039Stage2ProductionPolicy.lean` | checked Stage 2 no-promotion predicate carried into the W040 Lean/TLA tranche |
| `formal/tla/CoreEngineW036Stage2Partition.tla` and its five bounded configs | W040-local TLC model surface for Stage 2 partition bounded profiles |
| `../OxFml/docs/worksets/W073_conditional_formatting_typed_visualization_payload.md` and local W040 intake notes | W073 typed-only formatting guard retained; no OxCalc fallback from aggregate/visualization `thresholds` strings |

## 3. Artifact Surface

Run id: `w040-lean-tla-full-verification-discharge-001`

| Artifact | Result |
|---|---|
| `formal/lean/OxCalc/CoreEngine/W040LeanTlaFullVerificationDischarge.lean` | checked Lean classification surface for 11 W040 Lean/TLA rows |
| `archive/test-runs-core-engine-w038-w045/formal-assurance/w040-lean-tla-full-verification-discharge-001/run_summary.json` | records 11 proof/model rows, 6 local checked-proof rows, 3 bounded-model rows, 1 accepted external seam, 2 accepted boundaries, 5 totality boundaries, 5 exact blockers, and 0 failed rows |
| `archive/test-runs-core-engine-w038-w045/formal-assurance/w040-lean-tla-full-verification-discharge-001/w040_lean_tla_discharge_ledger.json` | machine-readable W040 Lean/TLA row ledger |
| `archive/test-runs-core-engine-w038-w045/formal-assurance/w040-lean-tla-full-verification-discharge-001/w040_lean_proof_register.json` | six local checked-proof classification rows |
| `archive/test-runs-core-engine-w038-w045/formal-assurance/w040-lean-tla-full-verification-discharge-001/w040_tla_model_bound_register.json` | three bounded-model rows, including the retained routine TLC boundary and Stage 2 partition bounded model evidence |
| `archive/test-runs-core-engine-w038-w045/formal-assurance/w040-lean-tla-full-verification-discharge-001/w040_lean_tla_exact_blocker_register.json` | five exact remaining Lean/TLA proof/model blockers |
| `archive/test-runs-core-engine-w038-w045/formal-assurance/w040-lean-tla-full-verification-discharge-001/source_evidence_index.json` | source evidence index binding W037 inventory, W039 formal assurance, W039 Stage 2 Lean policy, W040 direct obligations, W040 Lean/TLA file, and W040 Rust exact blockers |
| `archive/test-runs-core-engine-w038-w045/formal-assurance/w040-lean-tla-full-verification-discharge-001/validation.json` | validation status `formal_assurance_w040_lean_tla_discharge_valid` |
| `target/tla-w040-lean-tla/*` | W040-local TLC metadirs for the five Stage 2 partition configs; target output is not committed |

## 4. Row Disposition

| W040 row | Disposition | Evidence consequence |
|---|---|---|
| `w040_lean_inventory_checked_no_placeholder_evidence` | checked Lean inventory evidence | zero local Lean placeholders observed, but full Lean verification remains unpromoted |
| `w040_lean_rust_totality_classification_bridge` | checked Lean bridge evidence | binds the W040 Rust classification as an input, not a totality discharge |
| `w040_lean_stage2_policy_predicate_carried` | checked Lean policy predicate | carries the W039 Stage 2 no-promotion predicate into W040 |
| `w040_tla_routine_config_bounded_model_boundary` | bounded model with exact totality boundary | 11 retained routine TLC configs have zero failed configs, but unbounded TLA verification remains blocked |
| `w040_tla_stage2_partition_bounded_model_evidence` | bounded Stage 2 model evidence | five W036 Stage 2 partition configs rechecked locally for W040, but production policy remains unpromoted |
| `w040_tla_fairness_scheduler_assumption_boundary` | exact model assumption boundary | fairness and unbounded scheduler assumptions remain explicit blockers |
| `w040_full_lean_verification_exact_blocker` | exact Lean verification blocker | checked classification files do not prove every Rust, OxFml, and coordinator semantic path |
| `w040_full_tla_verification_exact_blocker` | exact TLA verification blocker | bounded TLC coverage does not discharge unbounded completeness, fairness, or production analyzer soundness |
| `w040_rust_totality_dependency_exact_blocker` | exact Rust dependency blocker | W040 Rust totality/refinement blockers remain proof dependencies |
| `w040_let_lambda_external_oxfunc_boundary` | accepted external seam boundary | LET/LAMBDA remains a narrow OxCalc/OxFml/OxFunc carrier seam; general OxFunc kernels remain external-owner scope |
| `w040_formal_model_spec_evolution_guard` | accepted spec-evolution guard | formal evidence may correct specs or implementation before any later promotion |

## 5. Lean And TLA Surface

`formal/lean/OxCalc/CoreEngine/W040LeanTlaFullVerificationDischarge.lean` defines 11 W040 proof/model rows and proves:

1. checked non-promoting classification for Lean inventory, Rust bridge, and Stage 2 policy predicate rows,
2. bounded non-promoting classification for the Stage 2 partition TLA row,
3. exact totality-boundary classification for routine bounded TLA, fairness/scheduler assumptions, full Lean verification, full TLA verification, and Rust totality dependency rows,
4. accepted external boundary classification for the LET/LAMBDA seam,
5. accepted spec-evolution guard classification,
6. summary values of 11 rows, 6 local-proof rows, 3 bounded-model rows, 1 accepted external seam, 2 accepted boundaries, 5 totality boundaries, 5 exact blockers, and no full Lean, full TLA, Rust totality, Stage 2, or general OxFunc promotion.

`src/oxcalc-tracecalc/src/formal_assurance.rs` now has a W040 Lean/TLA discharge profile.

The W040 profile:

1. reads W037 formal-inventory artifacts,
2. reads W039 formal-assurance and Stage 2 policy inputs,
3. reads W040 direct obligation-map artifacts,
4. reads W040 Rust totality/refinement summary, validation, and exact blockers,
5. checks that the W040 Lean/TLA classification file exists,
6. scans `formal/lean` for explicit `axiom`, `sorry`, and `admit` placeholders,
7. emits a Lean/TLA discharge ledger, Lean proof register, TLA model-bound register, exact-blocker register, source evidence index, validation, and run summary,
8. validates that W040 carries 11 proof/model rows, 6 local proof rows, 3 bounded model rows, 2 accepted boundaries, 5 totality boundaries, 5 exact blockers, and 0 failed rows.

The W040-local TLC rerun covered:

| Config | Result | Generated states | Distinct states |
|---|---:|---:|---:|
| `CoreEngineW036Stage2Partition.scheduler_blocked.cfg` | passed | 60,632 | 10,975 |
| `CoreEngineW036Stage2Partition.partition_cross_dep.cfg` | passed | 60,632 | 10,975 |
| `CoreEngineW036Stage2Partition.bounded_ready.cfg` | passed | 15,306 | 3,395 |
| `CoreEngineW036Stage2Partition.fence_reject.cfg` | passed | 54,690 | 6,490 |
| `CoreEngineW036Stage2Partition.multi_reader.cfg` | passed | 120,062 | 15,872 |

These are bounded TLC checks. They strengthen the model evidence for the named profiles but do not discharge unbounded fairness, scheduler coverage, production partition analyzer soundness, or full TLA verification.

## 6. OxFml And LET/LAMBDA Seam

This bead does not construct a new OxFml formatting payload and does not file an OxFml handoff.

It preserves two seam constraints for later W040 work:

1. W073 aggregate and visualization conditional-formatting metadata is `typed_rule`-only for `colorScale`, `dataBar`, `iconSet`, `top`, `bottom`, `aboveAverage`, and `belowAverage`; W072 bounded `thresholds` strings are not a fallback there.
2. `LET`/`LAMBDA` remains a narrow carrier seam that threads through OxCalc, OxFml, and a small OxFunc-owned fragment. General OxFunc kernels remain outside OxCalc formalization scope unless a specific carrier obligation requires a peek.

## 7. Semantic-Equivalence Statement

This bead adds a checked Lean classification file, W040 formal-assurance runner profile, emitted artifacts, W040-local TLC verification, and status/spec text.

It does not change coordinator scheduling, invalidation strategy, dependency graph construction, soft-reference update behavior, recalc behavior, publication semantics, reject policy, overlay lifecycle behavior, TreeCalc runtime behavior, TraceCalc reference semantics, optimized/core runtime semantics, OxFml evaluator behavior, OxFunc kernels, Stage 2 scheduler policy, pack/C5 capability policy, operated-service behavior, alert dispatch behavior, or retained-history behavior.

Observable runtime behavior is invariant under this bead. The W040 runner classifies Lean/TLA proof rows, bounded model rows, accepted seams, exact blockers, and no-promotion predicates only.

## 8. Verification

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | passed before document edits |
| `lean formal/lean/OxCalc/CoreEngine/W040LeanTlaFullVerificationDischarge.lean` | passed |
| `rg -n "^\\s*(axiom|sorry|admit)\\b" formal/lean` | passed; no placeholders found |
| `cargo test -p oxcalc-tracecalc formal_assurance -- --nocapture` | passed; 4 focused tests |
| `cargo run -p oxcalc-tracecalc-cli -- formal-assurance w040-lean-tla-full-verification-discharge-001` | passed; emitted W040 Lean/TLA artifacts |
| Five-command W040-local TLC rerun over `formal/tla/CoreEngineW036Stage2Partition.tla` configs | passed; no TLC errors found |
| `cargo test -p oxcalc-tracecalc` | passed; 40 tests |
| `cargo test -p oxcalc-tracecalc-cli` | passed; 0 tests |
| JSON parse for `archive/test-runs-core-engine-w038-w045/formal-assurance/w040-lean-tla-full-verification-discharge-001/*.json` | passed |
| `scripts/check-worksets.ps1` | passed after bead closure |
| `br ready --json` | passed after bead closure; next ready bead is `calc-tv5.5` |
| `br dep cycles --json` | passed with `count: 0` |
| `git diff --check` | passed; CRLF normalization warnings only |

## 9. Pre-Closure Verification Checklist

| # | Check | Result |
|---|---|---|
| 1 | Spec text and realization notes updated for all in-scope items? | yes; this packet, W040 workset/status surfaces, feature map, formal README, Lean file, runner profile, and machine-readable artifacts record the slice |
| 2 | Pack expectations updated for affected packs? | yes; pack/C5 remains unpromoted and `calc-tv5.9` owns reassessment |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes; this bead emits deterministic formal-assurance artifacts and W040-local TLC verification for the named bounded model profiles |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes; Section 7 states no runtime strategy changed |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes; W073 typed-only formatting is retained as a guard and no handoff trigger exists |
| 6 | All required tests pass? | yes for the current target; see Section 8 |
| 7 | No known semantic gaps remain in declared scope? | yes for this classification target; broader Lean/TLA and model-totality gaps remain exact blockers with owner lanes |
| 8 | Completion language audit passed? | yes; no full Lean/TLA verification, Rust totality, Stage 2 policy, pack-grade replay, C5, release-grade verification, broad OxFml, callable metadata, or general OxFunc promotion is claimed |
| 9 | `WORKSET_REGISTER.md` updated when ordered workset truth changed? | not applicable; ordered workset truth is unchanged |
| 10 | `IN_PROGRESS_FEATURE_WORKLIST.md` updated when feature-map truth changed? | yes; IP-04 records the W040 Lean/TLA state |
| 11 | execution-state blocker surface updated? | yes; `.beads/` records `calc-tv5.4` closure and `calc-tv5.5` readiness |

## 10. Completion Claim Self-Audit

| Step | Audit result |
|---|---|
| Scope re-read | pass; `calc-tv5.4` asks for W040 Lean/TLA full-verification discharge tranche |
| Gate criteria re-read | pass; checked Lean proof rows, bounded model rows, model assumptions, exact blockers, and promotion predicates have deterministic evidence |
| Silent scope reduction check | pass; full Lean verification, full TLA verification, Rust-engine totality, unbounded fairness/scheduler coverage, production Stage 2 policy, pack/C5, release-grade verification, broad OxFml, callable metadata projection, and general OxFunc kernels remain unpromoted |
| "Looks done but is not" pattern check | pass; checked Lean classification, zero-placeholder census, W037 routine inventory, and W040-local bounded TLC runs are not represented as full formal verification |
| Result | pass for the `calc-tv5.4` target after final bead closure validation |

## 11. Three-Axis Report

- execution_state: `calc-tv5.4_lean_tla_discharge_classified_no_promotion`
- scope_completeness: `scope_partial`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- open_lanes:
  - `calc-tv5.5` Stage 2 production policy and equivalence implementation is next
  - full Lean verification remains an exact blocker
  - full TLA verification remains an exact blocker
  - fairness and unbounded scheduler coverage remain exact model blockers
  - Rust totality/refinement exact blockers remain proof dependencies
  - production partition analyzer soundness and observable-result invariance for promoted Stage 2 profiles remain open
  - callable metadata projection remains an exact proof/seam blocker
  - operated services, retained history, independent evaluator diversity, OxFml seam breadth, pack/C5, and release-grade decision remain open
