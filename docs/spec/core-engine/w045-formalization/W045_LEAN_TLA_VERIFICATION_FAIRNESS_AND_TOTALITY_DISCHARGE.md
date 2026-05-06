# W045 Lean/TLA Verification Fairness And Totality Discharge

Status: `calc-zkio.4_lean_tla_verification_fairness_totality_discharge_validated`
Workset: `W045`
Parent epic: `calc-zkio`
Bead: `calc-zkio.4`

## 1. Purpose

This packet deepens the W045 Lean/TLA proof/model frontier after `calc-zkio.3`.

The W045.4 runner consumes the W045 residual obligation map, W045 Rust totality/refinement packet, W044 Lean/TLA packet, W044 Stage 2 packet, W037 formal inventory, and current OxFml formatting intake. It emits a fresh W045 Lean/TLA row model and formal-assurance packet that carries checked predecessor evidence, bounded-model evidence, dynamic/refinement bridge rows, publication-fence bridge rows, exact model blockers, and accepted external seam boundaries.

This narrows the assurance surface. It does not promote full Lean verification, full TLA verification, scheduler fairness, unbounded model coverage, Rust-engine totality, Rust refinement, full optimized/core verification, Stage 2 production policy, pack-grade replay, C5, callable metadata projection, callable carrier sufficiency, release-grade verification, broad OxFml closure, W073 downstream uptake, registered-external/provider callable publication, or general OxFunc kernels.

## 2. Authority Inputs Reviewed

| Input | Role |
|---|---|
| `docs/worksets/W045_CORE_FORMALIZATION_RELEASE_GRADE_SERVICE_AND_CROSS_REPO_UPTAKE_VERIFICATION.md` | W045 scope and `calc-zkio.4` gate |
| `docs/spec/core-engine/w045-formalization/W045_RESIDUAL_RELEASE_GRADE_SUCCESSOR_OBLIGATION_AND_CURRENT_OXFML_INTAKE_MAP.md` | W045 obligations, promotion contracts, OxFml intake, and no-promotion claims |
| `docs/test-runs/core-engine/release-grade-ledger/w045-residual-release-grade-successor-obligation-current-oxfml-intake-map-001/` | machine-readable W045.1 successor map, contracts, and OxFml intake |
| `docs/spec/core-engine/w045-formalization/W045_RUST_TOTALITY_REFINEMENT_AND_PANIC_SURFACE_HARDENING.md` | W045.3 Rust totality/refinement tranche |
| `docs/test-runs/core-engine/formal-assurance/w045-rust-totality-refinement-panic-surface-hardening-001/` | W045.3 Rust totality/refinement, panic-surface, and exact-blocker artifacts |
| `docs/test-runs/core-engine/formal-assurance/w044-lean-tla-unbounded-fairness-full-verification-expansion-001/` | W044 Lean/TLA predecessor evidence |
| `docs/test-runs/core-engine/stage2-replay/w044-stage2-production-partition-analyzer-scheduler-equivalence-001/` | W044 Stage 2 bounded scheduler, partition, and pack-equivalence evidence |
| `docs/test-runs/core-engine/formal-inventory/w037-proof-model-closure-001/` | Lean inventory and bounded TLA config evidence |
| `formal/lean/OxCalc/CoreEngine/W044LeanTlaFullVerificationAndFairness.lean` | predecessor checked Lean row model |
| `formal/lean/OxCalc/CoreEngine/W045RustTotalityAndRefinement.lean` | W045 Rust checked row model |
| `../OxFml/docs/worksets/W073_conditional_formatting_typed_visualization_payload.md` | current W073 typed-only formatting guard |
| `../OxFml/docs/handoffs/HANDOFF-DNAONECALC-012_W073_TYPED_CF_PAYLOAD_FIRST_SLICE.md` | downstream typed-rule request-construction handoff |

## 3. Artifact Surface

Run id: `w045-lean-tla-verification-fairness-totality-discharge-001`

| Artifact | Result |
|---|---|
| `formal/lean/OxCalc/CoreEngine/W045LeanTlaVerificationFairnessAndTotality.lean` | checked Lean row model for W045 Lean/TLA classification |
| `docs/test-runs/core-engine/formal-assurance/w045-lean-tla-verification-fairness-totality-discharge-001/run_summary.json` | 18 proof/model rows, 11 local proof rows, 4 bounded-model rows, 1 accepted external seam, 4 accepted boundaries, 6 totality-boundary rows, 6 exact blockers, 0 failed rows |
| `w045_lean_tla_discharge_ledger.json` | machine-readable 18-row Lean/TLA discharge ledger |
| `w045_lean_proof_register.json` | 11 local proof/classification rows |
| `w045_tla_model_bound_register.json` | 4 bounded model rows |
| `w045_lean_tla_exact_blocker_register.json` | 6 exact remaining proof/model blockers |
| `source_evidence_index.json` | W045.1, W045.3, W044 Lean/TLA, W044 Stage 2, W037 formal inventory, and OxFml-intake source index |
| `validation.json` | validation status `formal_assurance_w045_lean_tla_fairness_totality_valid` |

## 4. Implementation Delta

Changed files:

1. `formal/lean/OxCalc/CoreEngine/W045LeanTlaVerificationFairnessAndTotality.lean`
2. `src/oxcalc-tracecalc/src/formal_assurance.rs`

The formal-assurance runner now has a W045 Lean/TLA profile that:

1. reads W045 residual successor obligations and OxFml intake,
2. reads W045.3 Rust formal-assurance ledger, validation, refinement, and exact-blocker artifacts,
3. reads W044 Lean/TLA predecessor artifacts,
4. reads W044 Stage 2 scheduler/partition/pack-equivalence artifacts,
5. reads W037 Lean inventory and bounded TLA inventory artifacts,
6. emits W045 Lean/TLA ledger, local proof register, bounded model register, exact-blocker register, source evidence index, validation, and run summary artifacts,
7. validates 18 rows, 1 dynamic-refinement bridge row, 1 publication-fence bridge row, 1 Rust dependency blocker, 1 Stage 2 dependency blocker, 6 exact blockers, and 0 failed rows.

No runtime coordinator, dependency, invalidation, recalc, publication, OxFml, OxFunc, Stage 2, pack, or service behavior is changed.

## 5. Row Disposition

| Row | Disposition |
|---|---|
| Lean inventory and zero-placeholder evidence | checked local proof/classification evidence |
| W044 Lean/TLA predecessor packet | checked non-promoting predecessor evidence |
| W045 Rust mixed dynamic refinement bridge | checked Lean/TLA proof input for the exercised mixed add/remove/reclassify pattern |
| W045 Rust publication no-publish fence | checked Lean/TLA proof input for the exercised reject/no-publication path |
| `LET`/`LAMBDA` ordinary value carrier | checked callable-carrier boundary bridge |
| W044 Stage 2 scheduler/pack predicates | checked non-promoting policy-predicate input |
| W037 routine TLC configs | bounded model evidence and exact totality boundary |
| W044 Stage 2 partition replay | bounded model input |
| W044 declared-profile scheduler/pack equivalence | bounded model input |
| scheduler fairness and unbounded model coverage | exact model-assumption blocker |
| full Lean verification | exact proof blocker |
| full TLA verification | exact model blocker |
| Rust totality/refinement dependency | exact proof/model blocker |
| Stage 2 production policy dependency | exact proof/model blocker |
| `LET`/`LAMBDA` external OxFunc boundary | accepted external seam boundary |
| formalization/spec-evolution guard | accepted boundary |
| W073 typed formatting guard | accepted formatting boundary |
| no-proxy proof/model promotion guard | accepted boundary |

Observed counts:

1. 18 proof/model rows.
2. 11 local checked-proof classification rows.
3. 4 bounded-model rows.
4. 1 accepted external seam.
5. 4 accepted boundaries.
6. 6 totality-boundary rows.
7. 6 exact remaining blockers.
8. 1 dynamic-refinement bridge row.
9. 1 publication-fence bridge row.
10. 1 Rust dependency blocker.
11. 1 Stage 2 dependency blocker.
12. 0 failed rows.

## 6. Remaining Exact Blockers

| Blocker | Owner |
|---|---|
| `w045_tla_routine_config_bounded_model_boundary` | `calc-zkio.4`; successor reassessment in `calc-zkio.11` |
| `w045_tla_fairness_scheduler_unbounded_boundary` | `calc-zkio.4`; `calc-zkio.5` |
| `w045_full_lean_verification_exact_blocker` | `calc-zkio.4`; `calc-zkio.11` |
| `w045_full_tla_verification_exact_blocker` | `calc-zkio.4`; `calc-zkio.11` |
| `w045_rust_totality_dependency_exact_blocker` | `calc-zkio.3`; `calc-zkio.4`; `calc-zkio.10` |
| `w045_stage2_production_policy_dependency_exact_blocker` | `calc-zkio.4`; `calc-zkio.5` |

The W037 routine TLC inventory remains bounded model evidence with 11 routine configs and 0 failed configs. It is not unbounded TLA verification.

## 7. OxFml Formatting Intake

The W045.4 packet carries the current OxFml W073 intake from W045.1 and the sibling OxFml formatting diff:

1. `VerificationConditionalFormattingRule.typed_rule` is the only accepted metadata source for `colorScale`, `dataBar`, `iconSet`, `top`, `bottom`, `aboveAverage`, and `belowAverage`.
2. W072 bounded `thresholds` strings are intentionally ignored for those aggregate and visualization families.
3. `thresholds` remains meaningful only for scalar/operator/expression rules.
4. DNA OneCalc downstream typed-rule request construction is required but remains unverified by OxCalc.
5. W045.4 does not construct conditional-formatting requests and does not require an OxCalc core-engine code change.
6. No OxFml handoff is required by this bead.

## 8. Semantic-Equivalence Statement

This packet adds a checked Lean classification file, a W045 formal-assurance runner profile, emitted formal-assurance artifacts, and documentation.

It does not change coordinator scheduling, invalidation strategy, dependency graph construction, soft-reference update behavior, recalc behavior, publication semantics, reject policy, overlay lifecycle behavior, TraceCalc semantics, TreeCalc/CoreEngine runtime behavior, OxFml runtime behavior, OxFunc callable kernels, Stage 2 scheduler policy, pack/C5 capability policy, operated-service behavior, alert dispatch behavior, retained-history behavior, retained-witness behavior, or release-scale behavior.

Observable engine results are invariant under this packet. The only changed observable artifacts are W045.4 formal-assurance evidence files and the checked Lean classification file.

## 9. Verification

| Command | Result |
|---|---|
| `lean formal/lean/OxCalc/CoreEngine/W045LeanTlaVerificationFairnessAndTotality.lean` | passed |
| `cargo test -p oxcalc-tracecalc formal_assurance_runner_classifies_w045_lean_tla_fairness_totality -- --nocapture` | passed |
| `cargo run -p oxcalc-tracecalc-cli -- formal-assurance w045-lean-tla-verification-fairness-totality-discharge-001` | passed; emitted 18 rows with 0 failed rows |
| JSON parse for W045.4 formal-assurance artifacts | passed |
| `cargo test -p oxcalc-tracecalc formal_assurance -- --nocapture` | passed; 14 tests |
| `cargo fmt --all -- --check` | passed |
| `scripts/check-worksets.ps1` | passed pre-close; `worksets=23; beads total=187; open=8; in_progress=1; ready=0; blocked=7; deferred=0; closed=178` |
| `br ready --json` | passed pre-close; no ready beads while `calc-zkio.4` is in progress |
| `br dep cycles --json` | passed pre-close; `count=0` |
| `git diff --check` | passed pre-close; CRLF normalization warnings only |
| JSON parse for W045.4 formal-assurance artifacts | passed post-close |
| `scripts/check-worksets.ps1` | passed post-close; `worksets=23; beads total=187; open=8; in_progress=0; ready=1; blocked=6; deferred=0; closed=179` |
| `br ready --json` | passed post-close; next ready bead is `calc-zkio.5` |
| `br dep cycles --json` | passed post-close; `count=0` |
| `git diff --check` | passed post-close; CRLF normalization warnings only |

## 10. Pre-Closure Verification Checklist

| # | Check | Result |
|---|---|---|
| 1 | Spec text and realization notes updated for all in-scope items? | yes; this packet, W045 README/status surfaces, feature map, Lean file, runner profile, and machine-readable artifacts record the slice |
| 2 | Pack expectations updated for affected packs? | yes; pack/C5 remains unpromoted and `calc-zkio.10` owns reassessment |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes; W045.3 Rust evidence, W044 Lean/TLA evidence, W044 Stage 2 evidence, W037 formal inventory, and W045.4 artifacts are bound |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes; no runtime strategy changed |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes; current W073 typed-only formatting intake is carried and no handoff trigger exists |
| 6 | All required tests pass? | yes for this target; see Section 9 |
| 7 | No known semantic gaps remain in declared scope? | yes for this W045.4 classification target; broader exact blockers are explicit |
| 8 | Completion language audit passed? | yes; no full Lean/TLA verification, unbounded fairness, Rust totality/refinement, optimized/core, Stage 2, pack/C5, service, independent-diversity, broad OxFml, W073 downstream uptake, callable metadata, callable carrier, registered-external, provider-publication, or general OxFunc promotion is claimed |
| 9 | `WORKSET_REGISTER.md` updated when ordered workset truth changed? | not applicable; ordered workset truth is unchanged |
| 10 | `IN_PROGRESS_FEATURE_WORKLIST.md` updated when feature-map truth changed? | yes |
| 11 | execution-state blocker surface updated? | yes; `.beads/` records `calc-zkio.4` closed and `calc-zkio.5` ready |

## 11. Completion Claim Self-Audit

| Step | Audit result |
|---|---|
| Scope re-read | pass; `calc-zkio.4` asks for Lean/TLA verification, fairness, and totality discharge |
| Gate criteria re-read | pass; each Lean/TLA proof/model claim has direct evidence, checked proof/model classification, bounded model evidence, accepted boundary, or exact blocker |
| Silent scope reduction check | pass; broader Lean/TLA verification, scheduler fairness, unbounded model coverage, Rust totality/refinement, Stage 2 production policy, pack-grade replay, C5, callable, broad OxFml, W073 downstream uptake, and release-grade lanes remain explicit open lanes |
| "Looks done but is not" pattern check | pass; checked Lean classification, bounded TLA inventory, predecessor bridge rows, and dynamic/publication bridge rows are not reported as whole-engine proof or runtime implementation |
| Result | pass for the `calc-zkio.4` target after final post-close validation |

## 12. Three-Axis Report

- execution_state: `calc-zkio.4_lean_tla_verification_fairness_totality_discharge_validated`
- scope_completeness: `scope_partial`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- open_lanes:
  - `calc-zkio.5` Stage 2 production partition and pack-grade equivalence service evidence is next
  - scheduler fairness and unbounded model coverage remain exact blockers
  - full Lean verification remains an exact blocker
  - full TLA verification remains an exact blocker
  - Rust totality/refinement dependency remains an exact blocker
  - Stage 2 production policy dependency remains an exact blocker
  - full optimized/core verification, release-grade verification, C5, pack-grade replay, Stage 2 policy, operated services, independent evaluator breadth, mismatch quarantine, broad OxFml display/publication, public migration, W073 downstream typed-rule uptake, callable metadata projection, callable carrier sufficiency, registered-external callable projection, provider-failure/callable-publication semantics, continuous scale assurance, and general OxFunc kernels remain unpromoted
