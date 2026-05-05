# W041 Stage 2 Production Analyzer And Pack Equivalence Proof Tranche

Status: `spec_drafted_with_checked_replay_evidence`

Bead: `calc-sui.5`

Run id: `w041-stage2-production-analyzer-pack-equivalence-001`

## Purpose

This packet deepens the W041 Stage 2 lane after the optimized/core, Rust, and Lean/TLA packets.

The narrow new evidence is a W041 Stage 2 replay profile and checked Lean predicate that bind:

1. W040 bounded Stage 2 replay, permutation, observable-invariance, fence-counterpart, and bounded analyzer evidence,
2. W041 automatic dynamic transition evidence from `calc-sui.2`,
3. W041 Rust refinement and W041 Lean/TLA bounded-model evidence,
4. W041 snapshot and capability-view Stage 2 counterpart disposition,
5. W073 typed-only conditional-formatting intake,
6. declared-profile pack replay equivalence and no-proxy promotion guard.

It does not promote Stage 2 production policy, pack-grade replay, C5, full TLA verification, operated cross-engine differential service, or release-grade verification.

## Evidence Surfaces

| Artifact | Purpose |
| --- | --- |
| `formal/lean/OxCalc/CoreEngine/W041Stage2ProductionAnalyzerAndPackEquivalence.lean` | checked Lean predicate for Stage 2 analyzer and pack-equivalence promotion gates |
| `docs/test-runs/core-engine/stage2-replay/w041-stage2-production-analyzer-pack-equivalence-001/run_summary.json` | records row counts, evidence booleans, register paths, and no-promotion state |
| `docs/test-runs/core-engine/stage2-replay/w041-stage2-production-analyzer-pack-equivalence-001/w041_stage2_policy_gate_register.json` | records 14 W041 Stage 2 policy rows |
| `docs/test-runs/core-engine/stage2-replay/w041-stage2-production-analyzer-pack-equivalence-001/w041_production_analyzer_soundness_register.json` | records production-analyzer input rows and exact analyzer blockers |
| `docs/test-runs/core-engine/stage2-replay/w041-stage2-production-analyzer-pack-equivalence-001/w041_pack_equivalence_register.json` | records declared-profile pack-equivalence inputs and pack blockers |
| `docs/test-runs/core-engine/stage2-replay/w041-stage2-production-analyzer-pack-equivalence-001/w041_stage2_exact_blocker_register.json` | records 4 exact remaining Stage 2 blockers |
| `docs/test-runs/core-engine/stage2-replay/w041-stage2-production-analyzer-pack-equivalence-001/promotion_decision.json` | records no Stage 2 or pack-grade promotion and the exact remaining gates |
| `docs/test-runs/core-engine/stage2-replay/w041-stage2-production-analyzer-pack-equivalence-001/source_evidence_index.json` | indexes W041 and W040 source evidence |
| `docs/test-runs/core-engine/stage2-replay/w041-stage2-production-analyzer-pack-equivalence-001/validation.json` | records validation status and zero validation failures |

## Row Disposition

Observed counts:

1. 14 Stage 2 policy rows.
2. 10 satisfied declared-profile rows.
3. 5 partition replay rows.
4. 6 partition-order permutation rows.
5. 1 nontrivial permutation row.
6. 5 observable-invariance rows.
7. 1 W073 typed-formatting watch row.
8. 4 exact remaining Stage 2 blockers.
9. 0 failed rows.

The satisfied declared-profile rows are:

1. bounded baseline-versus-Stage-2 replay,
2. partition-order permutation replay,
3. observable-result invariance,
4. automatic dynamic transition analyzer input,
5. snapshot-fence Stage 2 counterpart evidence,
6. capability-view Stage 2 counterpart evidence,
7. bounded partition-analyzer predicate evidence,
8. W073 typed-formatting observable guard,
9. declared-profile pack replay equivalence,
10. no-proxy promotion guard.

## Remaining Exact Blockers

| Blocker | Consequence |
| --- | --- |
| `w041_stage2_full_production_analyzer_soundness_blocker` | Stage 2 production policy remains unpromoted |
| `w041_stage2_fairness_scheduler_unbounded_coverage_blocker` | full TLA verification and production scheduler coverage remain unpromoted |
| `w041_stage2_operated_cross_engine_service_dependency_blocker` | operated Stage 2 differential evidence remains required |
| `w041_stage2_pack_grade_replay_governance_blocker` | pack-grade replay, C5, and release-grade verification remain unpromoted |

## OxFml Formatting Intake

The current OxFml formatting update is incorporated as a guard row, not as a broad OxFml claim.

The W041.5 Stage 2 row preserves the W073 rule that `VerificationConditionalFormattingRule.typed_rule` is the accepted metadata source for `colorScale`, `dataBar`, `iconSet`, `top`, `bottom`, `aboveAverage`, and `belowAverage`. W072 bounded `thresholds` strings are not fallback metadata for those aggregate and visualization families.

Broad OxFml display/publication and callable-carrier closure remains under `calc-sui.8`.

## Semantic Equivalence Statement

This packet changes Stage 2 replay evidence generation, checked Lean classification, documentation, and generated evidence artifacts. It does not change optimized/core recalc behavior, TreeCalc invalidation behavior, publication policy, scheduling policy, OxFml runtime behavior, or OxFunc callable kernels.

Observable engine results are invariant under this packet. The changed observable artifacts are W041.5 Stage 2 replay evidence files and the checked Lean classification file.

The Stage 2 strategy-equivalence statement is limited to declared profiles: bounded replay, partition-order permutations, observable values, rejects, fence no-publication behavior, automatic dynamic transition evidence, W073 typed-formatting guard, and replay validation are equivalent for the recorded profiles. This does not prove production analyzer soundness, unbounded fairness, operated service equivalence, or pack-grade governance.

## Validation

| Command | Result |
| --- | --- |
| `lean formal/lean/OxCalc/CoreEngine/W041Stage2ProductionAnalyzerAndPackEquivalence.lean` | passed |
| `lean formal/lean/OxCalc/CoreEngine/W040Stage2ProductionPolicyAndEquivalence.lean` | passed |
| `cargo test -p oxcalc-tracecalc stage2_replay_runner_writes_w041_analyzer_pack_equivalence_without_promotion -- --nocapture` | passed |
| `cargo run -p oxcalc-tracecalc-cli -- stage2-replay w041-stage2-production-analyzer-pack-equivalence-001` | passed; emitted 5 partition rows, 6 permutation rows, 5 invariant rows, 4 exact blockers, and 0 failed rows |
| `cargo test -p oxcalc-tracecalc stage2_replay -- --nocapture` | passed; 4 tests |
| `cargo test -p oxcalc-tracecalc` | passed; 48 tests |
| `cargo test -p oxcalc-core` | passed; 58 tests |
| `scripts/check-worksets.ps1` | passed |
| `br dep cycles --json` | passed; 0 cycles |
| `git diff --check` | passed; CRLF warnings only |

## Status Report

- execution_state: `calc-sui.5_stage2_analyzer_pack_equivalence_validated_no_promotion`
- scope_completeness: `scope_partial`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- open_lanes:
  - `calc-sui.6` operated assurance, retained-history, and alert-dispatch service tranche
  - full production partition-analyzer soundness remains blocked
  - fairness and unbounded scheduler coverage remain blocked
  - operated cross-engine Stage 2 differential service remains blocked
  - pack-grade replay governance, C5, and release-grade verification remain unpromoted
  - full optimized/core verification, Rust totality/refinement, full Lean/TLA verification, broad OxFml display/publication, public migration, callable metadata, callable carrier sufficiency, and general OxFunc kernels remain unpromoted

## Pre-Closure Verification Checklist

| Check | Result |
| --- | --- |
| Workset and bead ids are explicit | yes: `W041`, `calc-sui.5` |
| Required artifacts exist | yes: Lean file and W041.5 Stage 2 replay packet artifacts are present |
| Checked proof/replay evidence exists for changed classification | yes: Lean check, runner test, and Stage 2 replay run |
| Model and profile bounds are explicit | yes: declared-profile evidence is separated from production analyzer and unbounded fairness blockers |
| No declared gap is match-promoted | yes: no-proxy guard is a satisfied row and promotion claims remain false |
| Residual blockers are explicit | yes: four exact remaining Stage 2 blockers |
| Semantic equivalence statement is present | yes |

## Completion Claim Self-Audit

| Audit Item | Result |
| --- | --- |
| Claim is limited to `calc-sui.5` target | yes |
| Scaffolding is not reported as implementation | yes |
| Spec text has checked/replay evidence | yes: Lean check, runner test, and generated replay packet |
| Cross-repo handoff is not treated as closure | yes; W073 remains a guard row and broad OxFml closure remains under `calc-sui.8` |
| Uncertain lanes default to in-progress | yes; production analyzer, fairness, operated service, and pack-governance blockers are retained |
| Strategy-change equivalence statement is present | yes |
