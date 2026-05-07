# W041 Independent Evaluator Breadth And Operated Differential Service Tranche

Status: `spec_drafted_with_checked_replay_evidence`

Bead: `calc-sui.7`

Run id: `w041-independent-evaluator-breadth-operated-differential-001`

## Purpose

This packet records the W041 independent-evaluator breadth and operated differential-service tranche after the W041 operated-assurance service-envelope evidence.

The narrow target is to replace the W040 bounded scalar-only diversity slice with stronger independent formula-fragment evidence where feasible:

1. a local evaluator fragment independent from TraceCalc, optimized/core, TreeCalc, OxFml, and OxFunc,
2. arithmetic with `+`, `-`, `*`, `/`, parentheses, unary minus, and named integer references,
3. comparison operators `>`, `>=`, `<`, `<=`, `=`, and `<>`,
4. `IF(condition, true_expr, false_expr)` branching,
5. retained source, independent-evaluator, cross-engine differential, mismatch-authority, blocker, promotion, and validation artifacts.

This packet does not promote full independent evaluator breadth, operated cross-engine differential service, mismatch quarantine service, broad OxFml display/publication, callable metadata projection, callable carrier sufficiency, Stage 2 policy, pack-grade replay, C5, or release-grade verification.

## OxFml Formatting Intake

The latest OxFml formatting update remains incorporated as an input-contract guard.

Current W041.7 consequence:

1. W073 is a direct-replacement input contract for aggregate and visualization conditional-formatting metadata.
2. `VerificationConditionalFormattingRule.typed_rule` remains the only accepted metadata source for `colorScale`, `dataBar`, `iconSet`, `top`, `bottom`, `aboveAverage`, and `belowAverage`.
3. W072 bounded `thresholds` strings are intentionally ignored for those families.
4. OxFml old-string non-interpretation evidence is retained through the W041.6 alert-dispatch source and W041.7 source rows.
5. This packet does not construct a conditional-formatting request payload, so no OxFml handoff is filed.
6. Broad OxFml display/publication and callable-carrier closure remains owned by `calc-sui.8`.

## Artifact Surface

| Artifact | Purpose |
| --- | --- |
| `archive/test-runs-core-engine-w038-w045/diversity-seam/w041-independent-evaluator-breadth-operated-differential-001/run_summary.json` | W041 diversity summary, row counts, no-promotion flags, and artifact paths |
| `archive/test-runs-core-engine-w038-w045/diversity-seam/w041-independent-evaluator-breadth-operated-differential-001/source_evidence_index.json` | 12 source rows binding W041 obligations, W040 diversity evidence, W041 conformance, TreeCalc, proof/model, Stage 2, operated-assurance, cross-engine service, alert-dispatch, and W073 formatting guard inputs |
| `archive/test-runs-core-engine-w038-w045/diversity-seam/w041-independent-evaluator-breadth-operated-differential-001/w041_independent_formula_evaluator_implementation.json` | 8 independent formula-fragment cases and expected/actual results |
| `archive/test-runs-core-engine-w038-w045/diversity-seam/w041-independent-evaluator-breadth-operated-differential-001/w041_independent_evaluator_breadth_register.json` | 10 independent-evaluator breadth classification rows |
| `archive/test-runs-core-engine-w038-w045/diversity-seam/w041-independent-evaluator-breadth-operated-differential-001/w041_cross_engine_differential_service_register.json` | 9 cross-engine differential/service rows |
| `archive/test-runs-core-engine-w038-w045/diversity-seam/w041-independent-evaluator-breadth-operated-differential-001/w041_mismatch_authority_router.json` | 9 mismatch authority and authority-routing rows |
| `archive/test-runs-core-engine-w038-w045/diversity-seam/w041-independent-evaluator-breadth-operated-differential-001/w041_exact_diversity_blocker_register.json` | 6 exact remaining diversity and service blockers |
| `archive/test-runs-core-engine-w038-w045/diversity-seam/w041-independent-evaluator-breadth-operated-differential-001/promotion_decision.json` | broadened formula-fragment evidence accepted; diversity, service, pack, C5, Stage 2, callable, and release-grade promotions remain false |
| `archive/test-runs-core-engine-w038-w045/diversity-seam/w041-independent-evaluator-breadth-operated-differential-001/validation.json` | validation status, counts, and empty failure list |

## Evidence Result

The W041.7 runner emits:

1. 12 source evidence rows.
2. 8 broadened independent formula-fragment cases.
3. 8 matched formula-fragment cases.
4. 10 independent-evaluator breadth rows.
5. 9 cross-engine differential/service rows.
6. 9 mismatch-authority rows.
7. 15 accepted boundary or service-contract rows.
8. 7 service-blocked rows.
9. 6 exact diversity/service blockers.
10. 0 failed rows.

The formula-fragment rows cover:

1. literal evaluation,
2. arithmetic precedence,
3. unary minus and subtraction,
4. division,
5. equality comparison,
6. `IF` branch selection,
7. the recalculated `X=3`, `Y=X*20`, `Z=X+Y` chain result `63`,
8. a not-equal guard.

Exact remaining blockers:

1. `w041_diversity.full_independent_evaluator_breadth_absent`
2. `w041_diversity.operated_cross_engine_differential_service_absent`
3. `w041_diversity.mismatch_triage_and_quarantine_service_absent`
4. `w041_diversity.stage2_operated_differential_dependency_absent`
5. `w041_diversity.oxfml_callable_breadth_dependency_absent`
6. `w041_diversity.release_grade_promotion_authority_absent`

## Promotion Consequence

This evidence narrows the W040 bounded scalar-only diversity gap by adding a broader independently executed formula fragment and authority-routing rows.

The following remain unpromoted:

1. full independent evaluator breadth,
2. operated cross-engine differential service,
3. mismatch triage/quarantine service,
4. broad OxFml display/publication,
5. callable metadata projection,
6. callable carrier sufficiency,
7. general OxFunc kernels,
8. Stage 2 production policy,
9. pack-grade replay,
10. `cap.C5.pack_valid`,
11. release-grade verification.

## Semantic Equivalence Statement

This packet adds W041 diversity runner logic, an independent formula-fragment evaluator used only for evidence generation, emitted diversity artifacts, exact blocker rows, tests, and status text.

It does not change coordinator scheduling, invalidation strategy, dependency graph construction, soft-reference update behavior, recalc behavior, publication semantics, reject policy, overlay lifecycle behavior, TraceCalc execution semantics, TreeCalc/CoreEngine runtime behavior, Lean/TLA model semantics, pack runtime behavior, OxFml/OxFunc evaluator behavior, or external service behavior.

Observable runtime behavior is invariant under this packet. The changed observable artifacts are W041.7 diversity evidence files and the diversity-seam runner test/CLI path.

## Validation

| Command | Result |
| --- | --- |
| `cargo fmt --all -- --check` | passed |
| `cargo test -p oxcalc-tracecalc diversity_seam -- --nocapture` | passed; 4 tests |
| `cargo run -p oxcalc-tracecalc-cli -- diversity-seam w041-independent-evaluator-breadth-operated-differential-001` | passed; emitted 12 source rows, 10 diversity rows, 9 OxFml seam-watch rows, 6 exact blockers, and 0 failed rows |
| `cargo test -p oxcalc-tracecalc` | passed; 50 tests |
| `cargo test -p oxcalc-tracecalc-cli` | passed |
| JSON parse for `archive/test-runs-core-engine-w038-w045/diversity-seam/w041-independent-evaluator-breadth-operated-differential-001/*.json` | passed |
| `scripts/check-worksets.ps1` | passed |
| `br ready --json` | passed after bead checkpoint; next ready bead is `calc-sui.8` |
| `br dep cycles --json` | passed; 0 cycles |
| `git diff --check` | passed; CRLF normalization warnings only |

## Status Report

- execution_state: `calc-sui.7_independent_evaluator_breadth_validated_no_promotion`
- scope_completeness: `scope_partial`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- open_lanes:
  - `calc-sui.8` OxFml broad display/publication and callable-carrier closure
  - full independent evaluator breadth remains open beyond the W041 formula fragment
  - operated recurring cross-engine differential service remains open
  - external mismatch triage/quarantine service remains open
  - broad OxFml display/publication, public migration, callable metadata, callable carrier sufficiency, pack/C5, Stage 2 policy, and release-grade verification remain unpromoted

## Pre-Closure Verification Checklist

| Check | Result |
| --- | --- |
| Workset and bead ids are explicit | yes: `W041`, `calc-sui.7` |
| Required artifacts exist | yes: W041.7 diversity-seam packet artifacts are present |
| Checked/replay evidence exists for changed classification | yes: runner test and generated diversity-seam packet |
| Independent evaluator boundary is explicit | yes: formula fragment only; no scheduling, publication, effect, callable-carrier, or full OxFml/OxFunc kernel authority |
| Service boundaries are explicit | yes: file-backed and service-contract evidence is separated from operated service promotion |
| No declared gap is match-promoted | yes: full independent breadth, operated cross-engine service, mismatch quarantine, Stage 2 service dependency, callable breadth, and release-grade blockers remain exact |
| Semantic-equivalence statement is present | yes |
| Cross-repo impact assessed | yes; W073 formatting update is a carried input-contract guard and no handoff is triggered |

## Completion Claim Self-Audit

| Audit Item | Result |
| --- | --- |
| Claim is limited to `calc-sui.7` target | yes |
| Scaffolding is not reported as implementation | yes |
| Spec text has checked/replay evidence | yes: runner test and generated diversity-seam packet |
| Cross-repo handoff is not treated as closure | yes; W073 remains an input-contract guard and broad OxFml closure remains under `calc-sui.8` |
| Uncertain lanes default to in-progress | yes; full independent breadth, operated service, external quarantine, broad OxFml, callable, pack/C5, and release-grade blockers are retained |
| Strategy-change equivalence statement is present | yes |
