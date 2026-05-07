# W042 Lean/TLA Fairness And Full-Verification Expansion

Status: `spec_drafted_with_checked_evidence`

Bead: `calc-czd.4`

Run id: `w042-lean-tla-fairness-full-verification-expansion-001`

## Purpose

This packet deepens the W042 proof/model tranche after the W042 Rust totality/refinement packet.

The narrow new evidence is a W042 Lean/TLA classifier that binds:

1. the current checked Lean inventory and zero-placeholder audit,
2. the W041 Lean/TLA predecessor packet,
3. the W042 Rust automatic dynamic transition refinement row,
4. the W042 callable value-carrier row for the narrow `LET`/`LAMBDA` seam,
5. the W041 Stage 2 analyzer and pack-equivalence predicate,
6. the existing bounded TLC inventory.

It does not promote full Lean verification, full TLA verification, Rust totality/refinement, Stage 2 production policy, pack-grade replay, C5, callable carrier sufficiency, release-grade verification, or general OxFunc kernels.

## Evidence Surfaces

| Artifact | Purpose |
|---|---|
| `formal/lean/OxCalc/CoreEngine/W042LeanTlaFairnessFullVerificationExpansion.lean` | checked Lean row model for W042 proof/model classification |
| `archive/test-runs-core-engine-w038-w045/formal-assurance/w042-lean-tla-fairness-full-verification-expansion-001/run_summary.json` | records row counts, register paths, promotion guards, and no-failure summary |
| `archive/test-runs-core-engine-w038-w045/formal-assurance/w042-lean-tla-fairness-full-verification-expansion-001/w042_lean_tla_discharge_ledger.json` | records the 14 W042 proof/model rows |
| `archive/test-runs-core-engine-w038-w045/formal-assurance/w042-lean-tla-fairness-full-verification-expansion-001/w042_lean_proof_register.json` | records 8 local checked-proof classification rows |
| `archive/test-runs-core-engine-w038-w045/formal-assurance/w042-lean-tla-fairness-full-verification-expansion-001/w042_tla_model_bound_register.json` | records 4 bounded-model rows |
| `archive/test-runs-core-engine-w038-w045/formal-assurance/w042-lean-tla-fairness-full-verification-expansion-001/w042_lean_tla_exact_blocker_register.json` | records 5 exact proof/model blockers |
| `archive/test-runs-core-engine-w038-w045/formal-assurance/w042-lean-tla-fairness-full-verification-expansion-001/source_evidence_index.json` | indexes W041 Lean/TLA, W042 Rust, W041 Stage 2, TLA inventory, W042 obligations, and Lean file evidence |
| `archive/test-runs-core-engine-w038-w045/formal-assurance/w042-lean-tla-fairness-full-verification-expansion-001/validation.json` | records validation status and zero validation failures |
| `archive/test-runs-core-engine-w038-w045/implementation-conformance/w042-optimized-core-counterpart-conformance-callable-metadata-001/w073_formatting_intake.json` | carries the current OxFml W073 typed-only formatting intake into this packet as a watched source |

## Row Disposition

| Row | Disposition |
|---|---|
| Lean inventory and placeholder audit | checked Lean evidence |
| W041 Lean/TLA predecessor bridge | checked Lean bridge evidence |
| W042 Rust dynamic refinement bridge | checked Lean refinement bridge |
| W042 callable carrier bridge | checked Lean callable-carrier bridge |
| W041 Stage 2 analyzer/pack predicate | checked Lean policy predicate |
| routine TLC config set | bounded model with exact totality boundary |
| Stage 2 partition bounded configs | bounded Stage 2 model evidence |
| W041 Stage 2 equivalence packet | bounded Stage 2 equivalence input |
| scheduler fairness and unbounded interleaving | exact model-assumption boundary |
| full Lean verification | exact proof blocker |
| full TLA verification | exact model blocker |
| Rust totality/refinement dependency | exact proof/model blocker |
| `LET`/`LAMBDA` carrier seam | accepted external seam boundary |
| spec-evolution guard | accepted boundary |

Observed counts:

1. 14 proof/model rows.
2. 8 local checked-proof classification rows.
3. 4 bounded-model rows.
4. 1 accepted external seam.
5. 2 accepted boundaries.
6. 5 totality-boundary rows.
7. 5 exact remaining blockers.
8. 0 failed rows.

## Fairness And Model-Bound Position

The W042.4 packet keeps the TLA floor explicit:

1. The routine TLC inventory still has 11 passed configs and 0 failed configs.
2. The W041 Stage 2 packet has bounded partition replay, permutation replay, observable-invariance, bounded analyzer evidence, declared-pack-equivalence evidence, counterpart evidence, and 4 exact Stage 2 blockers.
3. Those rows are bounded or declared-profile evidence only.
4. Full TLA verification remains blocked by unbounded model coverage, scheduler fairness, production partition-analyzer soundness, and downstream operated/pack evidence.
5. Stage 2 production policy remains blocked for `calc-czd.5`.

This is a proof/model strengthening packet, not a promotion packet.

## Rust And Callable Bridges

W042.4 consumes the W042.3 Rust rows:

1. `w042_automatic_dynamic_transition_refinement_evidence`,
2. `w042_callable_value_carrier_totality_evidence`.

Those rows are checked proof inputs for the formal model. They do not discharge retained Rust or callable blockers:

1. runtime panic-surface totality boundary,
2. broader dynamic transition coverage,
3. callable metadata projection,
4. full optimized/core release-grade conformance,
5. callable carrier sufficiency beyond the current value-carrier row,
6. general OxFunc kernels.

## OxFml W073 Formatting Intake

The latest OxFml formatting update was reviewed against this packet.

The W073 contract remains typed-only for aggregate and visualization conditional-formatting metadata:

1. `VerificationConditionalFormattingRule.typed_rule` is required for `colorScale`, `dataBar`, `iconSet`, `top`, `bottom`, `aboveAverage`, and `belowAverage`.
2. W072 bounded `thresholds` strings are intentionally ignored for those families.
3. `thresholds` remains meaningful only for scalar/operator/expression rule families where threshold text is the real rule input.

W042.4 does not construct conditional-formatting requests and does not change OxFml evaluator behavior. The updated source index therefore carries the W042.2 W073 intake as a seam-watch source; the broader request-construction and public migration lane remains owned by `calc-czd.8`.

## Semantic-Equivalence Statement

This packet changes proof/model classification, formal-assurance runner output, documentation, source-evidence indexing, and a checked Lean classification file only. It does not change optimized/core recalc behavior, TreeCalc invalidation behavior, publication policy, scheduling policy, Stage 2 replay policy, OxFml runtime behavior, or OxFunc callable kernels.

Observable engine results are therefore invariant under this packet. The only changed observable artifacts are W042.4 formal-assurance evidence files and the checked Lean classification file.

## Validation

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | passed |
| `lean formal/lean/OxCalc/CoreEngine/W042LeanTlaFairnessFullVerificationExpansion.lean` | passed |
| `lean formal/lean/OxCalc/CoreEngine/W042RustTotalityAndRefinement.lean` | passed |
| `lean formal/lean/OxCalc/CoreEngine/W041Stage2ProductionAnalyzerAndPackEquivalence.lean` | passed |
| `cargo test -p oxcalc-tracecalc formal_assurance_runner_classifies_w042_lean_tla_fairness_expansion -- --nocapture` | passed |
| `cargo run -p oxcalc-tracecalc-cli -- formal-assurance w042-lean-tla-fairness-full-verification-expansion-001` | passed; emitted 14 rows with 0 failed rows |
| `cargo test -p oxcalc-tracecalc` | passed; 54 tests plus doctests |
| `scripts/check-worksets.ps1` | passed; worksets=20, beads total=152, open=7, in_progress=1, ready=0, blocked=6, closed=144 |
| `br dep cycles --json` | passed; cycles=0 |
| `git diff --check` | passed; CRLF normalization warnings only |

## Status Report

- execution_state: `calc-czd.4_lean_tla_fairness_full_verification_expansion_validated`
- scope_completeness: `scope_partial`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- open_lanes:
  - `calc-czd.5` Stage 2 production analyzer and pack-grade equivalence closure
  - full Lean verification remains blocked
  - full TLA verification remains blocked
  - scheduler fairness and unbounded model coverage remain exact blockers
  - runtime panic-surface proof, broader dynamic transition coverage, callable metadata projection, full optimized/core verification, release-grade verification, C5, pack-grade replay, Stage 2 policy, operated services, independent evaluator breadth, mismatch quarantine, broad OxFml display/publication, public migration, callable carrier sufficiency, registered-external callable projection, provider-failure/callable-publication semantics, and general OxFunc kernels remain unpromoted

## Pre-Closure Verification Checklist

| # | Check | Result |
|---|---|---|
| 1 | Spec text and realization notes updated for all in-scope items? | yes; this packet, W042 README/status surfaces, feature map, Lean row model, and formal-assurance artifacts record the slice |
| 2 | Pack expectations updated for affected packs? | yes; pack/C5 remains unpromoted and `calc-czd.9` owns reassessment |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes; W042.3, W041 Stage 2, W037 TLA inventory, and W042.4 formal-assurance artifacts are bound |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes; no runtime strategy changed |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes; no OxFml handoff trigger exists for this proof/model classifier |
| 6 | All required tests pass? | yes; see Validation |
| 7 | No known semantic gaps remain in declared scope? | yes for this W042.4 classification target; broader exact blockers are explicit |
| 8 | Completion language audit passed? | yes; no release-grade, full Lean/TLA, Rust totality/refinement, optimized/core, Stage 2, pack/C5, service, independent-diversity, broad OxFml, callable metadata, callable carrier, registered-external, provider-publication, or general OxFunc promotion is claimed |
| 9 | `WORKSET_REGISTER.md` updated when ordered workset truth changed? | yes; no ordered workset-truth change in this bead |
| 10 | `IN_PROGRESS_FEATURE_WORKLIST.md` updated when feature-map truth changed? | yes; IP-04 records the W042 Lean/TLA update |
| 11 | execution-state blocker surface updated? | yes; `.beads/` records `calc-czd.4` state |

## Completion Claim Self-Audit

| Step | Audit result |
|---|---|
| Scope re-read | pass; `calc-czd.4` asks for Lean/TLA verification deepening with fairness/model-bound separation |
| Gate criteria re-read | pass; discharged proof rows, bounded model rows, exact blockers, accepted boundaries, and no-promotion claims are separated |
| Silent scope reduction check | pass; full Lean/TLA, fairness, unbounded model coverage, Rust dependency, Stage 2 policy, pack/C5, service, OxFml breadth, callable metadata, callable carrier sufficiency, and general OxFunc lanes remain explicit open lanes |
| "Looks done but is not" pattern check | pass; checked classification and bounded model evidence are not reported as full verification |
| Result | pass for the `calc-czd.4` target |
