# W041 Lean/TLA Full Verification And Fairness Discharge

Status: `spec_drafted_with_checked_evidence`

Bead: `calc-sui.4`

Run id: `w041-lean-tla-full-verification-fairness-discharge-001`

## Purpose

This packet deepens the W041 proof/model tranche after the W041 Rust totality/refinement packet.

The narrow new evidence is a W041 Lean/TLA classifier that binds:

1. the current checked Lean inventory and zero-placeholder audit,
2. the W040 Lean/TLA predecessor packet,
3. the W041 Rust automatic dynamic transition refinement row,
4. the W040 Stage 2 policy/equivalence predicate and bounded evidence,
5. the existing bounded TLC inventory.

It does not promote full Lean verification, full TLA verification, Rust totality/refinement, Stage 2 production policy, pack-grade replay, C5, or general OxFunc kernels.

## Evidence Surfaces

| Artifact | Purpose |
| --- | --- |
| `formal/lean/OxCalc/CoreEngine/W041LeanTlaFullVerificationAndFairnessDischarge.lean` | checked Lean row model for W041 proof/model classification |
| `archive/test-runs-core-engine-w038-w045/formal-assurance/w041-lean-tla-full-verification-fairness-discharge-001/run_summary.json` | records row counts, register paths, promotion guards, and no-failure summary |
| `archive/test-runs-core-engine-w038-w045/formal-assurance/w041-lean-tla-full-verification-fairness-discharge-001/w041_lean_tla_discharge_ledger.json` | records the 13 W041 proof/model rows |
| `archive/test-runs-core-engine-w038-w045/formal-assurance/w041-lean-tla-full-verification-fairness-discharge-001/w041_lean_proof_register.json` | records 7 local checked-proof classification rows |
| `archive/test-runs-core-engine-w038-w045/formal-assurance/w041-lean-tla-full-verification-fairness-discharge-001/w041_tla_model_bound_register.json` | records 4 bounded-model rows |
| `archive/test-runs-core-engine-w038-w045/formal-assurance/w041-lean-tla-full-verification-fairness-discharge-001/w041_lean_tla_exact_blocker_register.json` | records 5 exact proof/model blockers |
| `archive/test-runs-core-engine-w038-w045/formal-assurance/w041-lean-tla-full-verification-fairness-discharge-001/source_evidence_index.json` | indexes predecessor, W041 Rust, Stage 2, TLA inventory, and Lean file evidence |
| `archive/test-runs-core-engine-w038-w045/formal-assurance/w041-lean-tla-full-verification-fairness-discharge-001/validation.json` | records validation status and zero validation failures |

## Row Disposition

| Row | Disposition |
| --- | --- |
| Lean inventory and placeholder audit | checked Lean evidence |
| W040 Lean/TLA predecessor bridge | checked Lean bridge evidence |
| W041 Rust dynamic refinement bridge | checked Lean refinement bridge |
| W040 Stage 2 policy predicate | checked Lean policy predicate |
| routine TLC config set | bounded model with exact totality boundary |
| Stage 2 partition bounded configs | bounded Stage 2 model evidence |
| W040 Stage 2 equivalence packet | bounded Stage 2 equivalence input |
| scheduler fairness and unbounded interleaving | exact model-assumption boundary |
| full Lean verification | exact proof blocker |
| full TLA verification | exact model blocker |
| Rust totality/refinement dependency | exact proof/model blocker |
| LET/LAMBDA carrier seam | accepted external seam boundary |
| spec-evolution guard | accepted boundary |

Observed counts:

1. 13 proof/model rows.
2. 7 local checked-proof classification rows.
3. 4 bounded-model rows.
4. 1 accepted external seam.
5. 2 accepted boundaries.
6. 5 totality-boundary rows.
7. 5 exact remaining blockers.
8. 0 failed rows.

## Fairness And Model-Bound Position

The W041.4 packet keeps the TLA floor explicit:

1. The routine TLC inventory still has 11 passed configs and 0 failed configs.
2. The W040 Stage 2 packet still has bounded partition replay, permutation replay, observable-invariance, fence counterpart, and bounded partition-analyzer evidence.
3. Those rows are direct bounded evidence only.
4. Full TLA verification remains blocked by unbounded model coverage, scheduler fairness, and production partition-analyzer soundness.
5. Stage 2 production policy remains blocked for `calc-sui.5`.

This is a proof/model strengthening packet, not a promotion packet.

## Rust Refinement Bridge

W041.4 consumes the W041.3 Rust row `w041_automatic_dynamic_transition_refinement_evidence`.

That row is used as checked proof input for the formal model: the exercised resolved-to-potential automatic dynamic transition is now part of the refinement evidence surface. This does not discharge the retained Rust blockers:

1. runtime panic-surface totality boundary,
2. snapshot-fence refinement boundary,
3. capability-view fence refinement boundary,
4. callable metadata projection totality/refinement boundary.

## Semantic Equivalence Statement

This packet changes proof/model classification, formal-assurance runner output, documentation, and a checked Lean classification file only. It does not change optimized/core recalc behavior, TreeCalc invalidation behavior, publication policy, scheduling policy, Stage 2 replay policy, OxFml runtime behavior, or OxFunc callable kernels.

Observable engine results are therefore invariant under this packet. The only changed observable artifacts are W041.4 formal-assurance evidence files and the checked Lean classification file.

## Validation

| Command | Result |
| --- | --- |
| `lean formal/lean/OxCalc/CoreEngine/W041LeanTlaFullVerificationAndFairnessDischarge.lean` | passed |
| `lean formal/lean/OxCalc/CoreEngine/W041RustTotalityAndRefinement.lean` | passed |
| `cargo test -p oxcalc-tracecalc formal_assurance_runner_classifies_w041_lean_tla_discharge -- --nocapture` | passed |
| `cargo run -p oxcalc-tracecalc-cli -- formal-assurance w041-lean-tla-full-verification-fairness-discharge-001` | passed; emitted 13 rows with 0 failed rows |

## Status Report

- execution_state: `calc-sui.4_lean_tla_fairness_boundaries_classified_no_promotion`
- scope_completeness: `scope_partial`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- open_lanes:
  - `calc-sui.5` Stage 2 production analyzer and pack-equivalence proof tranche
  - full Lean verification remains blocked
  - full TLA verification remains blocked
  - scheduler fairness and unbounded model coverage remain exact blockers
  - runtime panic-surface proof, snapshot/capability refinement, callable metadata projection, full optimized/core verification, release-grade verification, C5, pack-grade replay, Stage 2 policy, operated services, independent evaluator breadth, public migration, broad OxFml display/publication, callable carrier sufficiency, and general OxFunc kernels remain unpromoted

## Pre-Closure Verification Checklist

| Check | Result |
| --- | --- |
| Workset and bead ids are explicit | yes: `W041`, `calc-sui.4` |
| Required artifacts exist | yes: Lean file and W041.4 formal-assurance packet artifacts are present |
| Checked proof/model evidence exists for changed classification | yes: Lean check and formal-assurance run |
| Model bounds are explicit | yes: bounded TLC and Stage 2 rows retain exact unbounded/fairness blockers |
| No declared gap is match-promoted | yes: promotion claims remain false and exact blockers are retained |
| Residual blockers are explicit | yes: five exact proof/model blockers |
| Semantic equivalence statement is present | yes |

## Completion Claim Self-Audit

| Audit Item | Result |
| --- | --- |
| Claim is limited to `calc-sui.4` target | yes |
| Scaffolding is not reported as implementation | yes |
| Spec text has checked evidence | yes: Lean check and formal-assurance run |
| Cross-repo handoff is not treated as closure | yes; LET/LAMBDA remains a narrow accepted external seam |
| Uncertain lanes default to in-progress | yes; exact blockers and open lanes are retained |
| Strategy-change equivalence statement is present | yes |
