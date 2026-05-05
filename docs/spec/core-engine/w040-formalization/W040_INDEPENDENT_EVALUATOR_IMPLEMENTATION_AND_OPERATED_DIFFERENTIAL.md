# W040 Independent Evaluator And Operated Differential Evidence

Status: `calc-tv5.7_bounded_independent_evaluator_diversity_validated_no_promotion`
Workset: `W040`
Parent epic: `calc-tv5`
Bead: `calc-tv5.7`

## 1. Purpose

This packet records the W040 independent-evaluator and operated-differential tranche.

The target is not full independent-evaluator promotion. The target is to introduce real independent implementation evidence where feasible, bind it into diversity and differential authority artifacts, and retain exact blockers for the breadth that is still not independently implemented or operated.

W040 adds a bounded independent scalar arithmetic evaluator inside the diversity runner. It is intentionally narrow: integer literals, names, `+`, `*`, and parentheses. It does not reuse TraceCalc, optimized/core, OxFml, or OxFunc execution.

No fully independent evaluator, operated cross-engine differential service, broad OxFml seam, callable metadata projection, Stage 2 policy, pack-grade replay, C5, or release-grade verification is promoted by this bead.

## 2. Artifact Surface

Run id: `w040-independent-evaluator-operated-differential-001`

| Artifact | Result |
|---|---|
| `docs/test-runs/core-engine/diversity-seam/w040-independent-evaluator-operated-differential-001/run_summary.json` | W040 summary and no-promotion flags |
| `docs/test-runs/core-engine/diversity-seam/w040-independent-evaluator-operated-differential-001/source_evidence_index.json` | 10 source rows binding W039 diversity, W040 conformance, proof/model, Stage 2, service, and W073 inputs |
| `docs/test-runs/core-engine/diversity-seam/w040-independent-evaluator-operated-differential-001/w040_independent_scalar_evaluator_implementation.json` | 5 bounded independent scalar evaluator cases, all matched |
| `docs/test-runs/core-engine/diversity-seam/w040-independent-evaluator-operated-differential-001/w040_independent_evaluator_row_set.json` | 8 independent-evaluator authority/classification rows |
| `docs/test-runs/core-engine/diversity-seam/w040-independent-evaluator-operated-differential-001/w040_cross_engine_differential_register.json` | 8 cross-engine differential and service-dependency rows |
| `docs/test-runs/core-engine/diversity-seam/w040-independent-evaluator-operated-differential-001/w040_differential_authority_register.json` | 7 differential-authority rows |
| `docs/test-runs/core-engine/diversity-seam/w040-independent-evaluator-operated-differential-001/w040_exact_diversity_blocker_register.json` | 5 exact remaining diversity/service blockers |
| `docs/test-runs/core-engine/diversity-seam/w040-independent-evaluator-operated-differential-001/promotion_decision.json` | bounded implementation evidence accepted; promotion flags remain false |
| `docs/test-runs/core-engine/diversity-seam/w040-independent-evaluator-operated-differential-001/validation.json` | validation status and counts |

## 3. Evidence Result

The W040 diversity runner emits:

1. 10 source evidence rows.
2. 5 bounded independent scalar evaluator cases.
3. 5 bounded independent scalar evaluator matches.
4. 8 independent-evaluator rows.
5. 8 cross-engine differential rows.
6. 7 differential-authority rows.
7. 11 accepted boundary or bounded-implementation rows.
8. 4 service-blocked rows.
9. 5 exact diversity/service blockers.
10. 0 failed rows.

Independent scalar evaluator cases:

1. `scalar.literal`: `2` => `2`
2. `scalar.precedence`: `2+3*4` => `14`
3. `scalar.parentheses`: `(2+3)*4` => `20`
4. `scalar.named_reference_y`: with `X=3`, `X*20` => `60`
5. `scalar.incremental_chain_z`: with `X=3`, `Y=60`, `X+Y` => `63`

This narrows the W039 full-absence blocker to a bounded implementation slice. It does not discharge full TreeCalc/OxCalc evaluator breadth.

## 4. Exact Blockers

Exact remaining blockers:

1. `w040_diversity.full_independent_evaluator_breadth_absent`
2. `w040_diversity.operated_cross_engine_service_absent`
3. `w040_diversity.stage2_differential_service_dependency_absent`
4. `w040_diversity.mismatch_triage_and_quarantine_service_absent`
5. `w040_diversity.release_grade_promotion_authority_absent`

## 5. OxFml Formatting Intake

The current OxFml formatting update remains a carried guard:

1. W073 aggregate and visualization conditional-formatting metadata is `typed_rule`-only for the named families.
2. W072 bounded `thresholds` strings are not fallback input for those families.
3. This bead does not construct an OxFml conditional-formatting request payload.
4. No OxFml handoff is filed by this bead because no exercised OxCalc artifact exposes a formatting mismatch.

## 6. Promotion Consequence

The following remain unpromoted:

1. full independent-evaluator diversity,
2. operated cross-engine differential service,
3. Stage 2 diversity service dependency,
4. service-level mismatch triage/quarantine,
5. broad OxFml seam and callable metadata projection,
6. pack-grade replay,
7. `cap.C5.pack_valid`,
8. release-grade verification.

## 7. Semantic-Equivalence Statement

This bead adds a bounded independent scalar evaluator in the diversity runner, W040 diversity runner logic, emitted diversity artifacts, exact blocker rows, tests, and status text only.

It does not change the evaluator kernels used by OxCalc, coordinator scheduling, invalidation strategy, dependency graph construction, soft-reference update behavior, recalc behavior, publication semantics, reject policy, overlay lifecycle behavior, TraceCalc semantics, TreeCalc/CoreEngine runtime behavior, optimized/core runtime behavior, OxFml evaluator behavior, OxFunc kernels, Lean/TLA model semantics, pack/C5 capability policy, Stage 2 scheduler policy, service operation, alert-dispatch behavior, or retained-history behavior.

Observable runtime behavior is invariant under this bead. The new bounded evaluator is evidence for diversity classification, not a production evaluator used by the core engine.

## 8. Verification

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | passed |
| `cargo test -p oxcalc-tracecalc diversity_seam -- --nocapture` | passed; 3 tests |
| `cargo run -p oxcalc-tracecalc-cli -- diversity-seam w040-independent-evaluator-operated-differential-001` | passed; emitted 10 source rows, 8 diversity rows, 8 cross-engine rows, 5 exact blockers, 0 failed rows |
| JSON parse for `docs/test-runs/core-engine/diversity-seam/w040-independent-evaluator-operated-differential-001/*.json` | passed |
| `cargo test -p oxcalc-tracecalc` | passed |
| `cargo test -p oxcalc-tracecalc-cli` | passed |
| `scripts/check-worksets.ps1` | passed after bead closure |
| `br ready --json` | passed after bead closure; next ready bead is `calc-tv5.8` |
| `br dep cycles --json` | passed with `count: 0` |
| `git diff --check` | passed; CRLF normalization warnings only |

## 9. Pre-Closure Verification Checklist

| # | Check | Result |
|---|---|---|
| 1 | Spec text and realization notes updated for all in-scope items? | yes; this packet, W040 README/status surfaces, feature map, runner artifacts, and machine-readable evidence record the slice |
| 2 | Pack expectations updated for affected packs? | yes; pack/C5 remains unpromoted and independent-diversity/service blockers remain exact |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes; the W040 diversity run emits deterministic implementation, diversity, cross-engine, authority, blocker, validation, and decision artifacts |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes; Section 7 states no runtime or scheduler strategy changed |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes; W073 typed-only formatting is retained as a guard and no handoff trigger exists |
| 6 | All required tests pass? | yes; see Section 8 |
| 7 | No known semantic gaps remain in declared scope? | yes for the W040.7 bounded implementation/diversity target; broader independent evaluator, operated differential, Stage 2, OxFml, pack/C5, and release-grade lanes remain open |
| 8 | Completion language audit passed? | yes; bounded scalar evaluator evidence is not reported as full independent-evaluator promotion |
| 9 | `WORKSET_REGISTER.md` updated when ordered workset truth changed? | not applicable; W040 ordering did not change |
| 10 | `IN_PROGRESS_FEATURE_WORKLIST.md` updated when feature-map truth changed? | yes; IP-04 records the W040.7 evidence state |
| 11 | execution-state blocker surface updated? | yes; `.beads/` records `calc-tv5.7` closure and `calc-tv5.8` readiness |

## 10. Completion Claim Self-Audit

| Step | Audit result |
|---|---|
| Scope re-read | pass; `calc-tv5.7` asks for independent evaluator implementation surface and operated differential evidence, or exact blockers |
| Gate criteria re-read | pass; shared projections and file-backed rows are not counted as full independent implementation authority |
| Silent scope reduction check | pass; bounded independent scalar evidence is named as bounded, and full breadth plus operated-service blockers remain exact |
| "Looks done but is not" pattern check | pass; the bounded scalar evaluator is not represented as full TreeCalc/OxCalc evaluator diversity or as an operated differential service |
| Result | pass for the `calc-tv5.7` bounded implementation/diversity target |

## 11. Three-Axis Report

- execution_state: `calc-tv5.7_bounded_independent_evaluator_diversity_validated_no_promotion`
- scope_completeness: `scope_partial`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- open_lanes:
  - `calc-tv5.8` OxFml seam breadth and callable metadata implementation is next
  - full independent-evaluator breadth remains open
  - operated cross-engine differential service remains open
  - Stage 2 differential service dependency remains blocked
  - service-level mismatch triage/quarantine remains open
  - pack/C5 and release-grade decision remain open
