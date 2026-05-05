# W037 TraceCalc Observable Closure And Multi-Reader Replay

Status: `calc-ubd.1_tracecalc_observable_closure_validated`
Workset: `W037`
Parent epic: `calc-ubd`
Bead: `calc-ubd.1`

## 1. Purpose

This packet records the W037 TraceCalc observable-closure slice.

W036 left the TraceCalc coverage matrix with one classified uncovered row: `w035_overlay_multi_reader_release_order`. W037 adds direct deterministic TraceCalc replay for that row and regenerates the oracle matrix under a W037 profile.

This is still not a full oracle or full verification claim. The W037 matrix has zero uncovered rows, but one row remains authority-excluded because the general OxFunc `LAMBDA` semantic kernel is outside the OxCalc TraceCalc profile. Full promotion also remains blocked by optimized/core-engine conformance, direct OxFml evaluator re-execution, Lean/TLA and Stage 2 partition work, independent evaluator diversity, operated assurance, and pack governance.

## 2. Authority Inputs Reviewed

| Input | Role |
|---|---|
| `docs/spec/core-engine/w036-formalization/W036_CLOSURE_AUDIT_AND_SUCCESSOR_PACKET.md` | W036 closure and W037 successor authority |
| `docs/spec/core-engine/w037-formalization/W037_RESIDUAL_FULL_VERIFICATION_AND_PROMOTION_GATE_LEDGER.md` | W037 obligation ledger |
| `docs/spec/core-engine/w036-formalization/W036_TRACECALC_COVERAGE_CLOSURE_CRITERIA_AND_MATRIX_EXPANSION.md` | W036 TraceCalc predecessor packet |
| `docs/worksets/W037_CORE_FORMALIZATION_FULL_VERIFICATION_PROMOTION_GATES.md` | W037 scope and gate model |
| `docs/test-runs/core-engine/tracecalc-reference-machine/w036-tracecalc-coverage-closure-001/` | W036 predecessor evidence |
| `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md` | current inbound OxFml seam and formatting watch ledger |
| `../OxFml/docs/worksets/W073_conditional_formatting_typed_visualization_payload.md` | current W073 formatting input-contract direction |
| `../OxFml/docs/handoffs/HANDOFF-DNAONECALC-012_W073_TYPED_CF_PAYLOAD_FIRST_SLICE.md` | W073 downstream handoff text |

Reviewed inbound observations: the latest OxFml W073 formatting update moves aggregate and visualization conditional-formatting metadata to a direct `typed_rule` input contract for `colorScale`, `dataBar`, `iconSet`, `top`, `bottom`, `aboveAverage`, and `belowAverage`. Bounded `thresholds` strings are intentionally ignored for those families and remain relevant only for scalar/operator/expression rule families where threshold text is the actual input.

## 3. Code And Corpus Surface

| Surface | Change |
|---|---|
| `src/oxcalc-tracecalc/src/machine.rs` | TraceCalc overlay release now defers while another reader remains pinned and emits `overlay_release_deferred_for_remaining_readers`. |
| `docs/test-corpus/core-engine/tracecalc/hand-auditable/tc_w037_overlay_multi_reader_release_order_001.json` | New hand-auditable scenario with two pinned readers and deterministic release ordering. |
| `docs/test-corpus/core-engine/tracecalc/MANIFEST.json` | Adds the W037 scenario to the checked corpus. |
| `src/oxcalc-tracecalc/src/oracle_matrix.rs` | Adds the W037 observable-closure profile, W037-specific no-loss and closure criteria, and test coverage for the multi-reader row. |
| `src/oxcalc-tracecalc/src/runner.rs` | Updates the expected corpus scenario count to include the W037 case. |
| `docs/test-runs/core-engine/tracecalc-reference-machine/w037-tracecalc-observable-closure-001/` | Checked-in W037 generated evidence root. |

## 4. Evidence Summary

W037 generated run:

`docs/test-runs/core-engine/tracecalc-reference-machine/w037-tracecalc-observable-closure-001/`

Oracle matrix summary:

| Metric | Value |
|---|---:|
| TraceCalc corpus scenarios | 31 |
| matrix rows | 32 |
| covered rows | 31 |
| classified uncovered rows | 0 |
| excluded rows | 1 |
| failed or missing rows | 0 |
| no-loss crosswalk gaps | 0 |
| full oracle claim | false |

The previous uncovered row now has direct replay:

| Row | W037 scenario | Evidence |
|---|---|---|
| `w035_overlay_multi_reader_release_order` | `tc_w037_overlay_multi_reader_release_order_001` | `reader_pinned`, `overlay_retained`, `overlay_release_deferred_for_remaining_readers`, and `overlay_released` are all observed. |

The excluded row remains:

| Row | Reason |
|---|---|
| `w035_callable_full_oxfunc_semantics` | General OxFunc `LAMBDA` semantic kernel is outside the OxCalc TraceCalc profile. The narrow OxCalc/OxFml `LET`/`LAMBDA` carrier fragment remains in later W037 scope. |

## 5. Multi-Reader Release Semantics

The W037 scenario exercises two readers pinned to the same view family while a dynamic-dependency overlay is retained.

Observed order:

1. reader A unpins,
2. TraceCalc emits `overlay_release_deferred_for_remaining_readers` with `remaining_reader_count = 1`,
3. reader B unpins,
4. TraceCalc emits `eviction_eligibility_opened`,
5. TraceCalc emits `overlay_released`.

This is a TraceCalc reference-machine replay refinement. It does not alter production TreeCalc/CoreEngine runtime behavior in this bead.

## 6. OxFml W073 Intake

The latest OxFml formatting update is incorporated as watch/input-contract evidence only.

Current OxCalc read:

1. any future OxCalc artifact that constructs W073 aggregate or visualization conditional-formatting payloads must emit `VerificationConditionalFormattingRule.typed_rule`,
2. `thresholds` must not be used for `colorScale`, `dataBar`, `iconSet`, `top`, `bottom`, `aboveAverage`, or `belowAverage`,
3. this W037 TraceCalc slice constructs no conditional-formatting payloads,
4. no OxFml handoff is triggered by this bead.

## 7. No Full Oracle Claim

The W037 TraceCalc observable profile has zero uncovered rows and one authority exclusion. That is enough for the `calc-ubd.1` target but not enough for a broader oracle or pack claim.

Remaining promotion blockers:

1. general OxFunc `LAMBDA` kernel remains outside the OxCalc TraceCalc profile,
2. optimized/core-engine conformance closure is not yet reached,
3. direct OxFml evaluator re-execution is not yet exercised,
4. Lean/TLA and Stage 2 partition work remain open,
5. independent evaluator diversity and operated assurance lanes remain open,
6. pack-grade replay, C5, and Stage 2 policy remain unpromoted.

## 8. Semantic-Equivalence Statement

W037 changes only the spec-purpose TraceCalc reference-machine replay profile for multi-reader overlay release ordering. It does not change production TreeCalc/CoreEngine runtime behavior, coordinator scheduling, invalidation, dependency graph construction, soft-reference resolution, recalc semantics, publication semantics, reject policy, pack decisions, continuous-assurance runners, or OxFml/OxFunc evaluator behavior.

Observable production behavior is invariant under this bead because no production coordinator, optimized/core-engine, OxFml evaluator, OxFunc kernel, pack, service, or Stage 2 scheduler path is changed.

## 9. Verification

| Command | Result |
|---|---|
| `cargo fmt --all` | passed |
| `$env:CARGO_TARGET_DIR='target\w037-test-target'; cargo test -p oxcalc-tracecalc oracle_matrix` | passed; 3 tests |
| `$env:CARGO_TARGET_DIR='target\w037-test-target'; cargo test -p oxcalc-tracecalc` | passed; 22 tests |
| `$env:CARGO_TARGET_DIR='target\w037-test-target'; cargo run -p oxcalc-tracecalc-cli -- tracecalc-oracle-matrix w037-tracecalc-observable-closure-001` | passed; emitted 32 matrix rows, 31 covered, 0 uncovered, 1 excluded, 0 failed/missing |
| `scripts/check-worksets.ps1` | passed after bead closure; `worksets=15`, `beads total=99`, `open=8`, `in_progress=0`, `ready=1`, `blocked=6`, `closed=91` |
| `br dep cycles --json` | passed with `count: 0` |
| `git diff --check` | passed; CRLF normalization warnings only |
| W073 intake check | passed; W037 docs and existing watch runners carry the `typed_rule`-only guardrail, with no OxCalc conditional-formatting request path in this bead |

## 10. Pre-Closure Verification Checklist

| # | Check | Result |
|---|---|---|
| 1 | Spec text and realization notes updated for all in-scope items? | yes; this packet, W037 status surfaces, corpus, oracle matrix, and run artifact root record the W037 TraceCalc slice |
| 2 | Pack expectations updated for affected packs? | yes; no pack promotion is made and pack blockers remain explicit |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes; `w037-tracecalc-observable-closure-001` covers the multi-reader row |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes; Section 8 states the TraceCalc-only replay refinement and production invariance |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes; W073 is watch/input-contract evidence only and no OxFml handoff is triggered |
| 6 | All required tests pass? | yes; see Section 9 |
| 7 | No known semantic gaps remain in declared scope? | yes for `calc-ubd.1`; one authority exclusion and broader promotion blockers remain outside this target |
| 8 | Completion language audit passed? | yes; broader formalization, pack, Stage 2, and full verification claims remain partial |
| 9 | `WORKSET_REGISTER.md` updated when ordered workset truth changed? | not applicable; ordered workset truth is unchanged |
| 10 | `IN_PROGRESS_FEATURE_WORKLIST.md` updated when feature-map truth changed? | yes; IP-04 now records the W037 TraceCalc observable-closure evidence |
| 11 | execution-state blocker surface updated? | yes; `.beads/` records this bead execution and the close command carries closure evidence |

## 11. Completion Claim Self-Audit

| Step | Audit result |
|---|---|
| Scope re-read | pass; `calc-ubd.1` asks for TraceCalc observable closure and direct multi-reader replay |
| Gate criteria re-read | pass; every in-scope row is covered or authority-excluded, and no full oracle claim is made |
| Silent scope reduction check | pass; the remaining authority exclusion and broader promotion blockers are explicit |
| "Looks done but is not" pattern check | pass; no full oracle, optimized/core-engine, OxFml evaluator, Lean/TLA, service, pack, C5, or Stage 2 promotion is claimed |
| Result | pass for the `calc-ubd.1` target |

## 12. Three-Axis Report

- execution_state: `calc-ubd.1_tracecalc_observable_closure_validated`
- scope_completeness: `scope_partial`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- open_lanes:
  - `calc-ubd.3` through `calc-ubd.9` remain in the sequential W037 path
  - full TraceCalc oracle promotion remains unclaimed because authority exclusions and non-TraceCalc gates remain
  - full optimized/core-engine verification and fully independent evaluator diversity remain open
  - direct OxFml evaluator re-execution and `LET`/`LAMBDA` seam evidence remain open
  - full Lean/TLA verification remains open
  - Stage 2 deterministic replay and partition promotion criteria remain open
  - pack-grade replay, C5, operated continuous-assurance service, operated continuous cross-engine differential service, and enforcing alert/quarantine service remain unpromoted
