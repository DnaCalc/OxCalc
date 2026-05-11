# W046 Scale Performance Semantic-Regression Signatures

Status: `calc-gucd.9_scale_semantic_signatures_validated`

Workset: `W046`

Parent epic: `calc-gucd`

Bead: `calc-gucd.9`

## 1. Purpose

This packet binds existing scale/performance runs to semantic-regression signatures backed by W046 semantic proof-spine artifacts.

Timing is useful only after semantic checks pass. No correctness claim in this packet derives from timing alone.

## 2. Signature Root

Signature ledger: `docs/test-runs/core-engine/semantic-regression/w046-scale-semantic-signatures-001/semantic_regression_signatures.json`

Run summary: `docs/test-runs/core-engine/semantic-regression/w046-scale-semantic-signatures-001/run_summary.json`

Summary:

| Metric | Value |
| --- | --- |
| profiles | `4` |
| profiles with passed source validation | `4` |
| profiles with closed-form expected values | `4` |
| profiles with phase timings | `4` |
| profiles with dynamic residuals | `1` |
| profiles with soft rebind churn | `1` |
| timing-only correctness claims | `0` |

## 3. Profile Signatures

| Profile | Model shape | Semantic facts checked | Phase timings collected | Correctness limits |
| --- | --- | --- | --- | --- |
| `million_grid_r2` | 1000x1000 `left_row + top_col` grid | 2,000,000 descriptors/edges, 0 diagnostics, 2,001 invalidation records, 0 cycles, closed-form sum/delta | descriptor lowering, graph/cycle scan, invalidation closure, model build, synthetic recalc, validation | synthetic closed-form recalc; no dynamic/rebind stress |
| `million_fanout_f8_r1` | 999,991 formulas each SUM 8 anchors | 7,999,928 descriptors/edges, 999,992 invalidation records, 0 cycles, closed-form sum/delta | descriptor lowering, graph/cycle scan, invalidation closure, model build, synthetic recalc, validation | static direct fanout only; not full publication trace |
| `million_relative_rebind_f8_r1` | relative parent-path formulas with root rename rebind churn | 7,999,928 relative descriptors/edges, 999,991 rebind seeds, 999,992 invalidation records, closed-form sum/delta | descriptor lowering, graph/cycle scan, rebind seed derivation, invalidation closure, model build, synthetic recalc, validation | closed-form owner-scan validates rebind seed count; not per-node replay sidecars |
| `million_indirect_r1` | grid plus dynamic `INDIRECT` residual carrier per formula | 3,000,000 descriptors, 2,000,000 static edges, 1,000,000 dynamic diagnostics, 2,001 invalidation records, closed-form sum/delta | descriptor lowering, graph/cycle scan, invalidation closure, model build, synthetic recalc, validation | dynamic `INDIRECT` remains residual diagnostic lane; no positive dynamic publication |

## 4. Semantic Sources

| Source | Role |
| --- | --- |
| `W046_INTEGRATED_SEMANTIC_KERNEL_AND_CROSS_PHASE_STATE_MACHINE.md` | integrated semantic vocabulary for graph/invalidation/order/candidate/publication/reject/trace |
| `docs/test-runs/core-engine/refinement/w046-proof-carrying-trace-001/checker_output.json` | proof-carrying trace fact names and validation baseline |
| `docs/test-runs/core-engine/refinement/w046-rust-refinement-bridge-001/implementation_semantic_mapping.json` | Rust/artifact authority mapping |
| `docs/test-runs/core-engine/refinement/w046-proof-service-coverage-001/semantic_spine_coverage_ledger.json` | semantic object coverage classes and exact blockers |

## 5. Optimization And Correctness Policy

1. Phase timing is telemetry, not correctness evidence.
2. A scale profile is semantically useful only when its source `validation.passed` is true.
3. Closed-form `expected_after_sum` and `expected_delta_sum` must equal observed values.
4. Dependency descriptor, edge, diagnostic, cycle, invalidation, dynamic, and rebind counts must match expected rows.
5. Optimization claims over these profiles would still need semantic preservation obligations for graph build, reverse edges, invalidation closure, order/read discipline, publication/reject behavior, and dynamic/rebind semantics.
6. This packet does not claim continuous scale assurance.

## 6. Validation

| Command | Result |
| --- | --- |
| JSON parse/reference check for `w046-scale-semantic-signatures-001` | passed |
| source scale run `validation.passed` check | passed for 4 profiles |
| closed-form expected/observed comparison | passed for 4 profiles |
| phase timing presence check | passed for 4 profiles |
| timing-only correctness claim check | passed; zero timing-only claims |

## 7. Semantic-Equivalence Statement

This bead adds semantic-regression signature metadata and documentation only.

Observable OxCalc behavior is invariant under this bead. It does not change graph construction, invalidation closure, evaluation order, formula evaluation, candidate/reject/publication behavior, TraceCalc execution, TreeCalc execution, OxFml/OxFunc behavior, performance behavior, proof-service behavior, pack policy, or service readiness.

## 8. Pre-Closure Verification Checklist

| # | Check | Result |
|---|-------|--------|
| 1 | Spec text and realization notes updated? | yes; this packet and signature ledger cover the declared profiles |
| 2 | Pack expectations updated? | no pack expectation changed |
| 3 | Deterministic replay artifact per in-scope behavior? | yes; profile source summaries and signature ledger are deterministic checked artifacts |
| 4 | Semantic-equivalence statement provided? | yes; Section 7 |
| 5 | FEC/F3E impact assessed? | yes; no seam change or handoff needed |
| 6 | Required validations pass? | yes; Section 6 |
| 7 | No semantic gaps hidden? | yes; profile correctness limits are explicit |
| 8 | Completion language audit passed? | yes; no optimization/readiness/continuous-assurance claim |
| 9 | `WORKSET_REGISTER.md` update needed? | no ordered workset register change required |
| 10 | `IN_PROGRESS_FEATURE_WORKLIST.md` updated? | yes; current semantic bead moves to `calc-gucd.10` after this bead |
| 11 | `.beads/` state updated? | yes; `.beads/` owns `calc-gucd.9` state |

## 9. Completion Claim Self-Audit

| Step | Result |
| --- | --- |
| Scope re-read | pass; bead asks for semantic-regression signatures, scale evidence root, closed-form values, semantic facts, phase timings, checker/validation result, and limits |
| Gate criteria re-read | pass; four profiles are bound to semantic facts and source validations |
| Silent scope reduction check | pass; no timing-only correctness or optimization claim is hidden |
| "Looks done but is not" pattern check | pass; timings are classified as telemetry after correctness checks |
| Include result | pass; checklist, audit, semantic equivalence, and three-axis report are included |

## 10. Current Status

- execution_state: `calc-gucd.9_closed`
- scope_completeness: `scope_complete`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
