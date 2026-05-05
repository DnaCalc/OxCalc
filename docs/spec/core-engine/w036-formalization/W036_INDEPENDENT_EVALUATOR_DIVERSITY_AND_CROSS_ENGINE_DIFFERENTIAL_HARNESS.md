# W036 Independent Evaluator Diversity And Cross-Engine Differential Harness

Status: `calc-rqq.6_independent_diversity_differential_harness_validated`
Workset: `W036`
Parent epic: `calc-rqq`
Bead: `calc-rqq.6`

## 1. Purpose

This packet defines and exercises the W036 independent-evaluator diversity and cross-engine differential harness.

The target is a stronger machine-readable evidence packet that distinguishes TraceCalc oracle evidence, TreeCalc/CoreEngine projection evidence, OxFml direct-evaluator absence, formal model/proof evidence, declared gaps, and no-promotion consequences. It is not full independent evaluator diversity, full optimized/core-engine verification, continuous cross-engine service operation, pack-grade replay, or Stage 2 policy promotion.

## 2. Authority Inputs Reviewed

| Input | Role in this bead |
|---|---|
| `docs/spec/core-engine/w035-formalization/W035_CONTINUOUS_ASSURANCE_AND_CROSS_ENGINE_DIFFERENTIAL_GATE.md` | predecessor continuous-assurance and cross-engine gate |
| `docs/test-runs/core-engine/independent-conformance/w034-independent-conformance-001/` | base TraceCalc-to-TreeCalc observable differential source |
| `docs/test-runs/core-engine/treecalc-local/w034-independent-conformance-treecalc-001/` | TreeCalc/CoreEngine projection source |
| `docs/test-runs/core-engine/tracecalc-reference-machine/w036-tracecalc-coverage-closure-001/` | W036 TraceCalc coverage-criteria source |
| `docs/test-runs/core-engine/implementation-conformance/w036-implementation-conformance-closure-001/` | W036 implementation-conformance closure-action source |
| `docs/test-runs/core-engine/tla/w036-stage2-partition-001/` | W036 TLA bounded model source |
| `docs/spec/core-engine/w036-formalization/W036_RESIDUAL_COVERAGE_AND_PROMOTION_BLOCKER_LEDGER.md` | W036 obligations `W036-OBL-012`, `W036-OBL-013`, and `W036-OBL-017` |
| `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md` | current inbound OxFml observation ledger |

## 3. Implementation Surface

| Surface | Change |
|---|---|
| `src/oxcalc-tracecalc/src/independent_conformance.rs` | adds W036 artifact emission for evaluator diversity, cross-engine differential harness, promotion guard, and cross-engine root validation |
| `src/oxcalc-tracecalc-cli/src/main.rs` | reports W036 diversity/differential/blocker counts for W036 independent-conformance runs |
| `docs/test-runs/core-engine/independent-conformance/w036-independent-diversity-differential-001/` | checked-in W036 independent-conformance and diversity evidence root |
| `docs/test-runs/core-engine/cross-engine-differential/w036-independent-diversity-differential-001/` | checked-in W036 cross-engine differential evidence root |

No runtime evaluator, coordinator, dependency graph, recalc, publication, formal model, pack, or OxFml behavior changes in this bead.

## 4. Evidence Summary

Run id: `w036-independent-diversity-differential-001`

| Metric | Value |
|---|---:|
| Base comparison rows | 15 |
| Exact value matches | 5 |
| No-publication matches | 3 |
| Lifecycle surface matches | 1 |
| Declared base gaps | 6 |
| Missing artifacts | 0 |
| Unexpected mismatches | 0 |
| W036 evaluator diversity rows | 5 |
| Fully independent evaluator rows | 0 |
| W036 cross-engine differential rows | 6 |
| W036 promotion blockers | 6 |
| Full independent evaluator promoted | no |
| Continuous cross-engine service promoted | no |

Primary artifacts:

1. `docs/test-runs/core-engine/independent-conformance/w036-independent-diversity-differential-001/run_summary.json`
2. `docs/test-runs/core-engine/independent-conformance/w036-independent-diversity-differential-001/diversity/evaluator_diversity_register.json`
3. `docs/test-runs/core-engine/independent-conformance/w036-independent-diversity-differential-001/differentials/cross_engine_differential_harness.json`
4. `docs/test-runs/core-engine/independent-conformance/w036-independent-diversity-differential-001/decision/promotion_guard.json`
5. `docs/test-runs/core-engine/independent-conformance/w036-independent-diversity-differential-001/replay-appliance/validation/bundle_validation.json`
6. `docs/test-runs/core-engine/cross-engine-differential/w036-independent-diversity-differential-001/run_summary.json`
7. `docs/test-runs/core-engine/cross-engine-differential/w036-independent-diversity-differential-001/differentials/cross_engine_differential_harness.json`
8. `docs/test-runs/core-engine/cross-engine-differential/w036-independent-diversity-differential-001/decision/promotion_guard.json`
9. `docs/test-runs/core-engine/cross-engine-differential/w036-independent-diversity-differential-001/validation.json`

## 5. Evaluator Diversity Register

| Engine row | Classification | Promotion consequence |
|---|---|---|
| `tracecalc_reference_machine` | covered reference oracle, separate runner and schema from TreeCalc | correctness oracle for covered rows only; not a second production evaluator |
| `treecalc_core_projection` | optimized/core projection lane | useful conformance evidence, not independent evaluator diversity |
| `oxfml_direct_evaluator` | external evaluator candidate | direct re-execution remains absent and pack-owned |
| `tla_model_family` | formal model checker | model evidence, not runtime evaluator evidence |
| `lean_proof_inventory` | formal proof inventory | proof evidence, not runtime evaluator evidence |

The register records `fully_independent_evaluator_count=0` and `full_independent_evaluator_promoted=false`.

## 6. Cross-Engine Differential Rows

| Row | State | Promotion consequence |
|---|---|---|
| `w036_diff_tracecalc_treecalc_observable_surface` | 15 comparison rows matched or declared; 0 unexpected mismatches; 6 declared gaps | declared gaps remain non-matches |
| `w036_diff_tracecalc_coverage_closure` | 32 matrix rows; 30 covered; 1 uncovered; 1 excluded; 0 missing/failed | no full TraceCalc oracle claim |
| `w036_diff_implementation_conformance_closure` | 6 W036 action rows; 2 first-fix rows; 4 blocker-routed rows; 0 match-promoted rows | no full optimized/core-engine conformance claim |
| `w036_diff_tla_stage2_partition_model` | 5 TLA configs passed; 0 failed configs | bounded model evidence only |
| `w036_diff_direct_oxfml_evaluator_reexecution` | declared direct-evaluator gap | direct OxFml evaluator re-execution remains later pack-grade evidence work |
| `w036_diff_independent_evaluator_diversity` | declared full-diversity gap | no full independent evaluator implementation diversity claim |

The cross-engine differential root records `unexpected_mismatch_count=0`, `missing_artifact_count=0`, and `status=cross_engine_differential_harness_valid`.

## 7. Promotion Guard

The W036 guard keeps these lanes unpromoted:

1. full independent evaluator diversity,
2. running continuous cross-engine differential service,
3. full optimized/core-engine verification while declared gaps remain,
4. direct OxFml evaluator re-execution,
5. full Lean/TLA verification,
6. pack C5 and Stage 2 policy.

The guard is intentionally conservative: it validates that the differential harness is coherent, while refusing to convert deterministic packet evidence into a service, pack, or full verification claim.

## 8. Obligation Disposition

| Obligation | Disposition |
|---|---|
| `W036-OBL-012` | diversity criteria are explicit and current engines are classified; full independent evaluator diversity remains unpromoted |
| `W036-OBL-013` | machine-readable cross-engine differential artifacts now record mismatches, declared gaps, diversity limits, and promotion consequences |
| `W036-OBL-017` | direct OxFml evaluator re-execution remains absent and is carried as a pack-grade evidence question |

## 9. OxFml Watch

No OxFml handoff is filed by this bead.

The W036 diversity register treats OxFml as an external evaluator candidate and records direct evaluator re-execution as absent. W073 typed conditional-formatting metadata remains watch/input-contract evidence only; this bead constructs no conditional-formatting payloads.

## 10. Semantic-Equivalence Statement

This bead adds W036 independent-conformance artifact emission, cross-engine differential artifact emission, CLI reporting, checked artifacts, and documentation only.

It does not change coordinator scheduling, invalidation strategy, dependency graph construction, soft-reference update behavior, recalc behavior, publication semantics, reject policy, overlay lifecycle behavior, TraceCalc execution semantics, TreeCalc/CoreEngine runtime behavior, Lean/TLA model semantics, pack-decision logic, continuous-assurance runner semantics, or OxFml/OxFunc evaluator behavior.

Observable runtime behavior is invariant under this bead. The new W036 artifacts classify existing evidence and write differential/guard packets; they do not change executable calculator behavior.

## 11. Verification

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | passed |
| `$env:CARGO_TARGET_DIR='target\w036-test-target'; cargo test -p oxcalc-tracecalc independent_conformance` | passed; 2 tests |
| `$env:CARGO_TARGET_DIR='target\w036-test-target'; cargo run -p oxcalc-tracecalc-cli -- independent-conformance w036-independent-diversity-differential-001` | passed; emitted 15 comparison rows, 5 W036 diversity rows, 6 W036 differential rows, and 6 promotion blockers |
| `$env:CARGO_TARGET_DIR='target\w036-test-target'; cargo test -p oxcalc-tracecalc` | passed |
| `$env:CARGO_TARGET_DIR='target\w036-test-target'; cargo test -p oxcalc-tracecalc-cli` | passed; 0 tests |
| `scripts/check-worksets.ps1` | passed |
| `br dep cycles --json` | passed; `count=0` |
| `git diff --check` | passed; CRLF normalization warnings only |

## 12. Pre-Closure Verification Checklist

| # | Check | Result |
|---|---|---|
| 1 | Spec text and realization notes updated for all in-scope items? | yes; this packet, W036 status surfaces, W036 residual ledger, spec index, and feature-map surfaces record the diversity/differential evidence |
| 2 | Pack expectations updated for affected packs? | yes; pack remains unpromoted and `calc-rqq.8` still owns reassessment |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes; this bead consumes deterministic TraceCalc, TreeCalc/CoreEngine, implementation-conformance, and TLA artifacts, and emits deterministic diversity/differential packets |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes; Section 10 states no runtime policy or strategy changed |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes; no OxFml-owned seam contradiction or handoff trigger appeared |
| 6 | All required tests pass? | yes; see Section 11 |
| 7 | No known semantic gaps remain in declared scope? | yes for this target; diversity limits, direct OxFml evaluator absence, declared gaps, continuous-service absence, and promotion consequences are explicit |
| 8 | Completion language audit passed? | yes; no full independent evaluator diversity, cross-engine service, optimized/core-engine verification, pack, or Stage 2 promotion is claimed |
| 9 | `WORKSET_REGISTER.md` updated when ordered workset truth changed? | not applicable; ordered workset truth is unchanged |
| 10 | `IN_PROGRESS_FEATURE_WORKLIST.md` updated when feature-map truth changed? | yes; IP-04 points at this W036 diversity/differential evidence |
| 11 | execution-state blocker surface updated? | yes; `.beads/` records `calc-rqq.6` execution state and later closure evidence |

## 13. Completion Claim Self-Audit

| Step | Audit result |
|---|---|
| Scope re-read | pass; `calc-rqq.6` asks for stronger cross-engine differential harness and independent-evaluator diversity classification |
| Gate criteria re-read | pass; machine-readable artifacts state mismatches, declared gaps, diversity limits, and promotion consequences |
| Silent scope reduction check | pass; fully independent evaluator implementation, continuous service operation, direct OxFml re-execution, and full optimized/core-engine verification are explicitly carried as unpromoted lanes |
| "Looks done but is not" pattern check | pass; deterministic differential packet evidence is not represented as a running service, pack-grade replay, or full verification |
| Result | pass for the `calc-rqq.6` independent diversity and cross-engine differential harness target |

## 14. Three-Axis Report

- execution_state: `calc-rqq.6_independent_diversity_differential_harness_validated`
- scope_completeness: `scope_partial`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- open_lanes:
  - `calc-rqq.7` through `calc-rqq.9` remain open
  - full independent evaluator implementation diversity remains partial because current evidence has 0 fully independent evaluator rows
  - continuous cross-engine differential service remains partial because this bead emits deterministic harness artifacts, not operated service/history
  - optimized/core-engine conformance remains partial while declared gaps remain non-matches
  - direct OxFml evaluator re-execution, pack-grade replay, full Lean/TLA verification, continuous assurance operation, and Stage 2 policy remain unpromoted
