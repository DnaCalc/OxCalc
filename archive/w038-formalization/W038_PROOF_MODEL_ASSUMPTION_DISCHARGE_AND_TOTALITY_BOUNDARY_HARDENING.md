# W038 Proof Model Assumption Discharge And Totality Boundary Hardening

Status: `calc-zsr.4_proof_model_assumption_discharge_validated`
Workset: `W038`
Parent epic: `calc-zsr`
Bead: `calc-zsr.4`

## 1. Purpose

This packet deepens the W037 proof/model inventory into a W038 assumption-discharge and totality-boundary ledger.

The target is not to promote full Lean verification, full TLA verification, Stage 2 policy, pack-grade replay, C5, or general OxFunc callable-kernel semantics. The target is to bind checked artifacts and classify each remaining proof/model lane as local proof evidence, bounded-model evidence, accepted external seam, explicit totality boundary, accepted spec-evolution guard, or exact remaining blocker.

## 2. Authority Inputs Reviewed

| Input | Role |
|---|---|
| `docs/worksets/W038_CORE_FORMALIZATION_RELEASE_GRADE_CLOSURE_HARDENING.md` | W038 gate model and `calc-zsr.4` target |
| `docs/spec/core-engine/w038-formalization/W038_RESIDUAL_RELEASE_GRADE_OBLIGATION_LEDGER_AND_OBJECTIVE_MAP.md` | W038 obligations `W038-OBL-007` through `W038-OBL-010`, `W038-OBL-019`, and `W038-OBL-020` |
| `docs/test-runs/core-engine/formal-inventory/w037-proof-model-closure-001/` | W037 proof/model inventory source |
| `archive/test-runs-core-engine-w038-w045/implementation-conformance/w038-optimized-core-conformance-disposition-001/` | callable metadata projection blocker input from `calc-zsr.3` |
| `docs/test-runs/core-engine/stage2-criteria/w037-stage2-deterministic-replay-criteria-001/` | Stage 2 no-promotion and replay-equivalence blocker input |
| `archive/test-runs-core-engine-w038-w045/tracecalc-authority/w038-tracecalc-authority-discharge-001/run_summary.json` | accepted external OxFunc semantic-kernel authority input |
| `formal/lean/OxCalc/CoreEngine/` | checked Lean proof and boundary files from Stage 1 through W038 |
| `formal/tla/` | routine bounded TLC model/config surface through W036 Stage 2 partition |

## 3. Artifact Surface

Run id: `w038-proof-model-assumption-discharge-001`

| Artifact | Result |
|---|---|
| `formal/lean/OxCalc/CoreEngine/W038AssumptionDischargeAndTotality.lean` | checked W038 proof/model assumption rows, totality boundaries, exact blockers, and non-promotion theorems |
| `archive/test-runs-core-engine-w038-w045/formal-assurance/w038-proof-model-assumption-discharge-001/run_summary.json` | 8 assumption rows, 3 local-proof rows, 2 bounded-model rows, 1 external-seam row, 2 accepted boundaries, 3 totality boundaries, 6 exact blockers, 0 failed rows |
| `archive/test-runs-core-engine-w038-w045/formal-assurance/w038-proof-model-assumption-discharge-001/w038_assumption_discharge_ledger.json` | machine-readable W038 assumption-discharge rows and evidence checks |
| `archive/test-runs-core-engine-w038-w045/formal-assurance/w038-proof-model-assumption-discharge-001/w038_totality_boundary_register.json` | 3 explicit totality boundaries |
| `archive/test-runs-core-engine-w038-w045/formal-assurance/w038-proof-model-assumption-discharge-001/w038_model_bound_register.json` | 2 bounded-model rows |
| `archive/test-runs-core-engine-w038-w045/formal-assurance/w038-proof-model-assumption-discharge-001/w038_exact_proof_model_blocker_register.json` | 6 exact proof/model blockers and owner lanes |
| `archive/test-runs-core-engine-w038-w045/formal-assurance/w038-proof-model-assumption-discharge-001/source_evidence_index.json` | source evidence index |
| `archive/test-runs-core-engine-w038-w045/formal-assurance/w038-proof-model-assumption-discharge-001/validation.json` | validation status `formal_assurance_w038_assumption_discharge_valid` |

## 4. Disposition Summary

| Source lane | W038 disposition | Evidence consequence |
|---|---|---|
| full Lean verification | explicit totality boundary plus exact blocker | W038 Lean file and W037 axiom-free inventory are bound; Rust-engine totality remains unpromoted |
| full TLA verification | bounded model evidence plus exact boundary | 11 routine TLC configs are bound as bounded evidence; unbounded model coverage remains unpromoted |
| general OxFunc callable kernels | accepted external seam boundary | OxCalc keeps only the narrow `LET`/`LAMBDA` carrier surface; general kernels remain external |
| Stage 2 replay/equivalence | exact blocker | deterministic partition replay and observable-result invariance remain owned by `calc-zsr.5` |
| pack-grade replay | exact blocker | proof/model inventory is not pack-grade replay governance |
| C5 | exact blocker | C5 waits for later direct W038 evidence and release decision |
| spec evolution | accepted spec-evolution guard | the formalization path keeps specs evolvable rather than freezing the initial model universe |
| callable metadata projection | exact proof/seam blocker | value-only carrier evidence is bound; callable metadata projection remains under `calc-zsr.7` |

## 5. Formal Surface

The checked Lean surface now contains 14 core-engine files, including `W038AssumptionDischargeAndTotality.lean`.

The W038 file proves that the eight assumption rows are non-promoting, that full Lean and callable metadata rows are exact totality boundaries, that the TLA row is bounded model evidence with an exact boundary, that the general OxFunc kernel row is an accepted external boundary, and that pack/C5/Stage 2 rows remain exact blockers.

The routine TLC surface remains the 11 checked configs from W037. W038 reran those configs with isolated `target/tla-w038/...` metadirs. These are bounded model checks only; they do not promote full TLA verification.

## 6. Semantic-Equivalence Statement

This bead adds a checked Lean classification file, a formal-assurance runner path, emitted artifacts, and status/spec text for proof/model assumption discharge.

It does not change coordinator scheduling, invalidation strategy, dependency graph construction, soft-reference update behavior, recalc behavior, publication semantics, reject policy, overlay lifecycle behavior, TreeCalc runtime behavior, TraceCalc reference semantics, OxFml evaluator behavior, OxFunc kernels, TLA model semantics, Stage 2 scheduler policy, pack/C5 capability policy, service behavior, or alert/quarantine policy.

Observable runtime behavior is invariant under this bead. The packet classifies proof/model evidence and exact blockers only.

## 7. Verification

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | passed |
| `cargo test -p oxcalc-tracecalc formal_assurance -- --nocapture` | passed; 1 test |
| `cargo test -p oxcalc-tracecalc` | passed; 27 tests plus doc-tests |
| `cargo test -p oxcalc-tracecalc-cli` | passed; 0 tests |
| `cargo run -p oxcalc-tracecalc-cli -- formal-assurance w038-proof-model-assumption-discharge-001` | passed; emitted W038 formal-assurance artifacts |
| `lean` over `formal/lean/OxCalc/CoreEngine/*.lean` | passed; 14 files |
| `rg -n "^\s*(axiom\|sorry\|admit)\b" formal\lean` guarded for no matches | passed; no matches |
| `scripts\run-tlc.ps1` routine config set with `-metadir target\tla-w038\...` | passed; 11 configs |
| JSON parse for `archive/test-runs-core-engine-w038-w045/formal-assurance/w038-proof-model-assumption-discharge-001/*.json` | passed |
| `scripts/check-worksets.ps1` | passed after bead closure |
| `br ready --json` | passed after bead closure; next ready bead is `calc-zsr.5` |
| `br dep cycles --json` | passed with `count: 0` |
| `git diff --check` | passed; CRLF normalization warnings only |

## 8. Pre-Closure Verification Checklist

| # | Check | Result |
|---|---|---|
| 1 | Spec text and realization notes updated for all in-scope items? | yes; this packet, W038 README/status surfaces, formal layout, machine-readable artifacts, and feature map record the slice |
| 2 | Pack expectations updated for affected packs? | yes; pack/C5 remains unpromoted and `calc-zsr.8` owns reassessment |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes for carried runtime behavior; this bead adds checked Lean and bounded TLC evidence rather than runtime replay |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes; Section 6 states no runtime strategy changed |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes; no OxFml/FEC/F3E change or handoff trigger exists for this bead |
| 6 | All required tests pass? | yes; see Section 7 |
| 7 | No known semantic gaps remain in declared scope? | yes for the `calc-zsr.4` classification target; broader full Lean/TLA, Stage 2, pack, C5, operated-service, and release-grade gaps remain exact blockers or later W038 lanes |
| 8 | Completion language audit passed? | yes; the packet limits claims to W038 assumption-discharge and totality-boundary classification |
| 9 | `WORKSET_REGISTER.md` updated when ordered workset truth changed? | not applicable; ordered workset truth is unchanged |
| 10 | `IN_PROGRESS_FEATURE_WORKLIST.md` updated when feature-map truth changed? | yes; IP-04 records the W038 proof/model state |
| 11 | execution-state blocker surface updated? | yes; `.beads/` records `calc-zsr.4` closure and `calc-zsr.5` readiness |

## 9. Completion Claim Self-Audit

| Step | Audit result |
|---|---|
| Scope re-read | pass; `calc-zsr.4` asks for checked artifacts and an assumption ledger distinguishing local proof, bounded model, external seam, and blocked rows |
| Gate criteria re-read | pass; W038 artifacts include assumption, totality-boundary, bounded-model, and exact-blocker registers |
| Silent scope reduction check | pass; full Lean/TLA verification, general OxFunc kernels, Stage 2 replay/policy, pack-grade replay, C5, operated services, and release-grade verification remain explicit open lanes |
| "Looks done but is not" pattern check | pass; checked Lean/TLC evidence is not represented as total proof, full TLA verification, pack-grade replay, C5, or Stage 2 policy promotion |
| Result | pass for the `calc-zsr.4` target |

## 10. Three-Axis Report

- execution_state: `calc-zsr.4_proof_model_assumption_discharge_validated`
- scope_completeness: `scope_partial`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- open_lanes:
  - `calc-zsr.5` Stage 2 partition replay and semantic-equivalence execution is next
  - full Lean verification remains unpromoted
  - full TLA verification remains unpromoted
  - Stage 2 deterministic replay/equivalence remains open
  - operated assurance, alert/quarantine, and cross-engine service remain open
  - independent evaluator diversity and OxFml seam watch closure remain open
  - pack-grade replay governance, C5, and W038 release decision remain open
