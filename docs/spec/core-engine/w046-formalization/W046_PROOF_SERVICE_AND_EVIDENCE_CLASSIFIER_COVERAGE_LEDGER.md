# W046 Proof-Service And Evidence-Classifier Coverage Ledger

Status: `calc-gucd.8_coverage_ledger_validated`

Workset: `W046`

Parent epic: `calc-gucd`

Bead: `calc-gucd.8`

## 1. Purpose

This packet recasts proof-service and evidence-classifier material as coverage accounting over the W046 semantic proof spine.

It does not promote W046 to release-grade verification, C5, Stage 2, operated service, pack-grade replay, or broad OxFml/OxFunc readiness. Classifier rows identify what each semantic object has: spec, checked model, replay/checker artifact, Rust bridge, exact blocker, or no additional coverage.

## 2. Coverage Ledger

Canonical ledger: `docs/test-runs/core-engine/refinement/w046-proof-service-coverage-001/semantic_spine_coverage_ledger.json`

Run summary: `docs/test-runs/core-engine/refinement/w046-proof-service-coverage-001/run_summary.json`

Coverage summary:

| Metric | Value |
| --- | --- |
| rows | `11` |
| rows with formal targets | `11` |
| rows with replay/checker artifacts | `11` |
| rows with Rust bridge | `9` |
| rows with exact blockers | `8` |
| promotion rows | `0` |

## 3. Coverage Rows

| Semantic object | Transition | Coverage class | Exact blockers |
| --- | --- | --- | --- |
| dependency descriptors | descriptor lowering | spec + model + checker + Rust bridge | none |
| forward/reverse edges | graph build and reverse index | spec + model + checker + Rust bridge projection | native reverse-edge JSON sidecar absent; Rust Tarjan line proof not discharged |
| SCC/cycle groups | cycle classify or reject | spec + model + TraceCalc checker + bridge | arbitrary finite SCC completeness; Rust Tarjan line proof |
| invalidation closure | seed to reverse-reachable closure | spec + model + checker + bridge | normalized invalidation comparator full cross-run gap |
| rebind gate | structural rebind reject/no-publish | spec + model + bridge residual | positive dynamic dependency publication refinement not discharged |
| recalc tracker states | dirty/needed/evaluating/verified-clean/publish/reject | spec + model only for tracker | tracker native trace bridge not independently checked in `.18` |
| evaluation order and reads | topological order and working-value reads | spec + model + checker + Rust bridge projection | Rust topological queue line proof; native per-read trace events absent |
| candidate/reject/publication | candidate then publish or reject/no-publish | spec + model + checker + Rust bridge | none |
| OxFml effect boundary | formula candidate/reject/format/display/publication authority | spec + model + replay roots + selected bridge | broad OxFml proof, positive format/display projection, registered-external publication consequence |
| TraceCalc refinement | observable TraceCalc-to-TreeCalc kernel match | spec + model + refinement root + checker + bridge | full TraceCalc/TreeCalc/CoreEngine refinement not claimed |
| integrated kernel | graph/invalidation/order/candidate/publication/reject/trace | integrated spec + model + checker + bridge | unbounded TLA and full Rust refinement not claimed |

## 4. Classifier Policy

The W046 classifier role is `coverage_accounting_only`.

Rules:

1. A row with formal targets means a spec/model exists and has been checked at its declared scope.
2. A row with replay/checker artifacts means deterministic artifacts exist for selected cases.
3. A row with a Rust bridge means `calc-gucd.18` maps selected real implementation/artifact facts into the semantic vocabulary.
4. A row with exact blockers is still useful accounting, not promotion.
5. Classifier rows cannot promote release-grade, C5, Stage 2, operated-service, or pack-grade claims.
6. Rows cannot erase residuals recorded by `.15-.18`.

## 5. Validation

| Command | Result |
| --- | --- |
| JSON parse/reference check for `w046-proof-service-coverage-001` ledger and summary | passed |
| classifier promotion row check | passed; `promotion_rows = 0` |
| referenced artifact existence check | passed |

## 6. Semantic-Equivalence Statement

This bead adds coverage ledger artifacts and documentation only.

Observable OxCalc behavior is invariant under this bead. It does not change dependency graph construction, invalidation closure, evaluation order, formula evaluation, candidate construction, rejection, publication, TraceCalc execution, TreeCalc execution, OxFml/OxFunc behavior, proof-service behavior, pack policy, performance behavior, or service readiness.

## 7. Pre-Closure Verification Checklist

| # | Check | Result |
|---|-------|--------|
| 1 | Spec text and realization notes updated for all in-scope items? | yes; this packet and coverage ledger cover semantic objects and transitions |
| 2 | Pack expectations updated for affected packs? | yes; no pack expectation changed |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes; rows point to existing deterministic model/checker/refinement roots or blockers |
| 4 | Semantic-equivalence statement provided? | yes; Section 6 |
| 5 | FEC/F3E impact assessed? | yes; no seam change and no handoff needed |
| 6 | Required validations pass? | yes; Section 5 |
| 7 | No known semantic gaps hidden? | yes; blockers remain explicit per row |
| 8 | Completion language audit passed? | yes; no promotion or full-proof claim |
| 9 | `WORKSET_REGISTER.md` updated when ordered truth changed? | no ordered register change required |
| 10 | `IN_PROGRESS_FEATURE_WORKLIST.md` updated? | yes; current semantic bead moves to `calc-gucd.9` after this bead |
| 11 | `.beads/` state updated? | yes; `.beads/` owns `calc-gucd.8` state |

## 8. Completion Claim Self-Audit

| Step | Result |
| --- | --- |
| Scope re-read | pass; bead asks for coverage ledger keyed by semantic object/transition with specs, formal targets, replay/checker artifacts, Rust bridge, and exact blockers |
| Gate criteria re-read | pass; ledger has 11 rows, 0 promotion rows, and reference validation |
| Silent scope reduction check | pass; tracker/Rust/proof/OxFml residuals are explicit |
| "Looks done but is not" pattern check | pass; coverage accounting is not promoted to readiness |
| Include result | pass; checklist, audit, semantic equivalence, and three-axis report are included |

## 9. Current Status

- execution_state: `calc-gucd.8_closed`
- scope_completeness: `scope_complete`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
