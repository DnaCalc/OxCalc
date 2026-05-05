# W037 Direct OxFml Evaluator And LET/LAMBDA Seam Evidence

Status: `calc-ubd.4_direct_oxfml_evaluator_validated`
Workset: `W037`
Parent epic: `calc-ubd`
Bead: `calc-ubd.4`

## 1. Purpose

This packet records the W037 direct OxFml evaluator slice.

The target is to exercise the OxFml runtime facade directly from OxCalc-owned fixtures, bind the narrow `LET`/`LAMBDA` carrier seam to deterministic evidence, and exercise the current W073 formatting guardrail without promoting general OxFunc callable kernels, pack-grade replay, C5, Stage 2 policy, or full optimized/core-engine verification.

## 2. Authority Inputs Reviewed

| Input | Role |
|---|---|
| `docs/worksets/W037_CORE_FORMALIZATION_FULL_VERIFICATION_PROMOTION_GATES.md` | W037 scope and exit-gate authority |
| `docs/spec/core-engine/w037-formalization/W037_RESIDUAL_FULL_VERIFICATION_AND_PROMOTION_GATE_LEDGER.md` | W037 obligation map, including `W037-OBL-005`, `W037-OBL-006`, and `W037-OBL-015` |
| `docs/spec/core-engine/w037-formalization/W037_OPTIMIZED_CORE_ENGINE_CONFORMANCE_IMPLEMENTATION_CLOSURE.md` | prior W037 conformance packet and residual callable/direct-evaluator blocker |
| `docs/test-fixtures/core-engine/upstream-host/` | OxCalc-owned direct OxFml runtime-facade fixture corpus |
| `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md` | current OxFml consumer/runtime and formatting watch floor |
| `../OxFml/docs/worksets/W073_conditional_formatting_typed_visualization_payload.md` | W073 typed conditional-formatting input contract |
| `../OxFml/crates/oxfml_core/src/consumer/runtime/mod.rs` | public runtime facade exercised by the OxCalc upstream-host packet |
| `../OxFml/crates/oxfml_core/src/publication/mod.rs` | public formatting publication carrier and W073 `typed_rule` model |

## 3. Implementation Changes

This bead extends the existing OxCalc upstream-host scaffold rather than adding a production coordinator dependency.

1. `MinimalUpstreamHostPacket` now carries an optional `VerificationPublicationContext`.
2. `MinimalUpstreamHostPacket::recalc` passes that context through `RuntimeFormulaRequest::with_verification_publication_context` when present.
3. The checked upstream-host fixture corpus now has 12 cases:
   - 9 predecessor host/table/reference fixture rows,
   - 2 W037 direct `LET`/`LAMBDA` rows,
   - 1 W037/W073 typed conditional-formatting guard row.
4. `UpstreamHostRunner` emits deterministic direct-OxFml evidence artifacts under `docs/test-runs/core-engine/upstream-host/{run_id}`.
5. `oxcalc-tracecalc-cli upstream-host <run-id>` is the CLI entry point for this evidence lane.

## 4. Deterministic Evidence

| Artifact | Result |
|---|---|
| `docs/test-runs/core-engine/upstream-host/w037-direct-oxfml-evaluator-001/run_summary.json` | 12 cases, 0 expectation mismatches, 3 direct-OxFml rows, 2 `LET`/`LAMBDA` rows, 1 W073 formatting guard row |
| `docs/test-runs/core-engine/upstream-host/w037-direct-oxfml-evaluator-001/cases/uh_let_lambda_lexical_capture_eval_001/result.json` | direct OxFml runtime-facade result `Number(12)`, trace includes `SPECIAL.LAMBDA`, `SPECIAL.LAMBDA_INVOKE`, `FUNC.OP_ADD`, and nested `SPECIAL.LET` |
| `docs/test-runs/core-engine/upstream-host/w037-direct-oxfml-evaluator-001/cases/uh_returned_lambda_invocation_eval_001/result.json` | direct OxFml runtime-facade result `Number(15)`, trace includes returned-lambda construction and invocation |
| `docs/test-runs/core-engine/upstream-host/w037-direct-oxfml-evaluator-001/cases/uh_typed_cf_top_rank_guard_001/result.json` | W073 `typed_rule.rank` drives top-2 formatting; legacy `thresholds: ["legacy-count:1"]` is preserved but not interpreted for the aggregate family |

## 5. Decision Summary

| Obligation | W037 decision |
|---|---|
| `W037-OBL-005` direct OxFml evaluator re-execution | exercised for the upstream-host fixture slice through `oxfml_core::consumer::runtime::RuntimeEnvironment::execute`; direct-evaluator absence is no longer the blocker for this slice |
| `W037-OBL-006` narrow `LET`/`LAMBDA` carrier seam | exercised for lexical capture and returned-lambda invocation; general OxFunc callable kernel verification remains outside OxCalc scope and continues to feed `calc-ubd.5` proof/model inventory |
| `W037-OBL-015` OxFml formatting watch inputs | exercised for one W073 typed aggregate rule; `format_delta` and `display_delta` remain distinct and absent in the exercised row; no broad display-facing closure is claimed |
| OxFml handoff decision | no handoff filed; the exercised runtime facade and formatting payload were sufficient for this OxCalc artifact |
| Pack/C5 decision | not promoted; direct evaluator evidence removes one specific no-promotion reason but does not satisfy proof/model, operated-service, Stage 2, independent-evaluator, or pack-governance gates |

## 6. Semantic-Equivalence Statement

The runtime behavior of existing OxCalc coordinator, TreeCalc, TraceCalc, dependency, invalidation, publication, reject, overlay, pack, Lean, and TLA paths is invariant under this bead.

The implementation change is confined to the upstream-host evidence scaffold: an optional publication context is passed to OxFml only for fixtures that declare it. Existing upstream-host fixtures without publication context execute through the same `RuntimeEnvironment::execute` path as before. The new CLI runner serializes direct OxFml runtime-facade observations and does not change production scheduling, recalc strategy, dependency graph construction, soft-reference resolution, or publication policy.

For the new `LET`/`LAMBDA` rows, OxCalc observes OxFml/OxFunc callable behavior through the public OxFml runtime result. OxCalc does not reinterpret or reimplement the general callable kernel. For the W073 row, OxCalc observes OxFml's typed conditional-formatting result and records the distinction between typed rule metadata, retained legacy threshold text, `format_delta`, and `display_delta`.

## 7. Verification

| Command | Result |
|---|---|
| `cargo fmt --all` | passed |
| `cargo fmt --all -- --check` | passed |
| `cargo test -p oxcalc-core upstream_host -- --nocapture` | passed; 8 library tests and 1 integration test |
| `cargo run -p oxcalc-tracecalc-cli -- upstream-host w037-direct-oxfml-evaluator-001` | passed; emitted 12 direct upstream-host cases with 0 mismatches |
| `cargo test -p oxcalc-core` | passed; 52 library tests, 5 integration tests, 0 doc-tests |
| `cargo test -p oxcalc-tracecalc-cli` | passed; 0 tests |
| `cargo test -p oxcalc-tracecalc upstream_host` | passed; 0 executed, 23 filtered |
| `scripts/check-worksets.ps1` | passed after bead closure; worksets=15, beads total=99, open=6, in_progress=0, ready=1, blocked=4, closed=93 |
| `br ready --json` | passed; next ready bead is `calc-ubd.5` |
| `br dep cycles --json` | passed; `count: 0` |
| `git diff --check` | passed; line-ending warnings only |

## 8. Pre-Closure Verification Checklist

| # | Check | Result |
|---|---|---|
| 1 | Spec text and realization notes updated for all in-scope items? | yes; this packet, the W037 workset, the residual ledger, the upstream-host seam spec, fixture docs, and feature worklist record the slice |
| 2 | Pack expectations updated for affected packs? | yes; pack/C5 remains unpromoted and later `calc-ubd.8` owns reassessment |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes; direct OxFml runtime-facade, `LET`/`LAMBDA`, and W073 guard rows are emitted under `w037-direct-oxfml-evaluator-001` |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes; Section 6 states no production strategy or policy changed |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes; no concrete OxFml-owned insufficiency was exposed by the exercised runtime facade or formatting carrier |
| 6 | All required tests pass? | yes; commands in Section 7 passed |
| 7 | No known semantic gaps remain in declared scope? | yes for the `calc-ubd.4` target; broader proof/model, optimized/core, Stage 2, pack, C5, and operated-assurance lanes remain open |
| 8 | Completion language audit passed? | yes; the packet limits claims to the direct upstream-host fixture slice and keeps broader promotion lanes open |
| 9 | `WORKSET_REGISTER.md` updated when ordered workset truth changed? | not applicable; ordered workset truth is unchanged |
| 10 | `IN_PROGRESS_FEATURE_WORKLIST.md` updated when feature-map truth changed? | yes; IP-04 records the direct OxFml evidence slice and its non-promotion limits |
| 11 | execution-state blocker surface updated? | yes; the W037 workset and bead closure notes carry this target status |

## 9. Completion Claim Self-Audit

| Step | Audit result |
|---|---|
| Scope re-read | pass; the target is direct OxFml evaluator evidence, two narrow `LET`/`LAMBDA` rows, and one W073 formatting guard row |
| Gate criteria re-read | pass for `calc-ubd.4`; `W037-OBL-005`, the exercised slice of `W037-OBL-006`, and the exercised watch slice of `W037-OBL-015` have deterministic artifacts |
| Silent scope reduction check | pass; excluded pack-grade replay, C5, general OxFunc kernels, full optimized/core verification, full Lean/TLA verification, Stage 2 policy, and broad display-facing closure are explicitly listed as open lanes |
| "Looks done but is not" pattern check | pass; compile-only and scaffold-only evidence is not used as the basis for the target claim |
| Result | pass for the `calc-ubd.4` target only; W037 remains scope-partial |

## 10. Three-Axis Report

- execution_state: `calc-ubd.4_direct_oxfml_evaluator_validated`
- scope_completeness: `scope_partial`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- open_lanes:
  - `calc-ubd.5` Lean/TLA proof and model closure inventory is the next ready W037 target
  - full optimized/core-engine verification remains open with residual conformance blockers
  - full Lean/TLA verification remains open
  - Stage 2 deterministic replay and partition promotion criteria remain open
  - operated continuous assurance, operated cross-engine differential service, pack-grade replay governance, and C5 candidate decision remain unpromoted
