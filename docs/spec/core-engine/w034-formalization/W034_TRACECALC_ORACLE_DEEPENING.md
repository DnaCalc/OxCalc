# W034 TraceCalc Oracle Deepening

Status: `calc-e77.2_evidence_packet`
Workset: `W034`
Parent epic: `calc-e77`
Bead: `calc-e77.2`
Created: 2026-05-05

## 1. Purpose

This packet records the W034 TraceCalc oracle-deepening slice.

The target is a wider executable oracle surface for:

1. stale and compatibility fence rejection,
2. dynamic-reference negative behavior,
3. overlay retention and release pressure,
4. the narrow `LET`/`LAMBDA` carrier fragment where OxCalc must preserve OxFml/OxFunc-facing callable facts,
5. replay-equivalent observable histories.

This slice widens TraceCalc's covered reference behavior. It does not claim full formula semantics, full OxFml fixture counterpart coverage, pack-grade replay, Stage 2 promotion, or production/optimized-core conformance beyond the paired TraceCalc oracle/engine comparison emitted by the runner.

## 2. OxFml Formatting Intake

Current inbound OxFml formatting changes were reviewed before this run:

1. `format_delta` and `display_delta` remain distinct canonical seam categories.
2. Broader display-facing closure remains deferred unless a concrete publication or replay mismatch appears.
3. OxFml's W073 first slice now treats `VerificationConditionalFormattingRule.typed_rule` metadata as the active input contract for `colorScale`, `dataBar`, `iconSet`, `top`, `bottom`, `aboveAverage`, and `belowAverage` option families.
4. The W034 TraceCalc scenarios in this bead do not construct `VerificationConditionalFormattingRule` or conditional-formatting payloads.

Decision for this bead: `no_local_formatting_patch_required`.

The formatting update remains a W034 seam-watch input. If a later W034 artifact constructs the W073 conditional-formatting payload families, OxCalc must use typed metadata and must not rely on the earlier bounded threshold-string convention.

## 3. Corpus Widening

New hand-auditable TraceCalc scenarios:

| Scenario | Coverage |
|---|---|
| `tc_w034_snapshot_fence_reject_001` | snapshot/fence mismatch, typed reject, no-publish preservation |
| `tc_w034_capability_fence_reject_001` | capability-view mismatch, typed reject, no-publish preservation |
| `tc_w034_dynamic_dependency_negative_001` | dynamic-reference switch to unresolved runtime dependency, typed reject, affected-work volume |
| `tc_w034_overlay_eviction_after_unpin_001` | protected dynamic-dependency overlay retained under pinned reader, then eviction-eligible and released after unpin |
| `tc_w034_let_lambda_higher_order_replay_001` | higher-order callable carrier identity, returned callable identity, dependency/runtime-effect visibility, ordinary-value publication |
| `tc_w034_replay_equivalent_independent_order_001` | deterministic batch publication for independent input edits with replay-history projection metadata |

The manifest now selects 21 scenarios: the prior 15-scenario TraceCalc corpus plus these 6 W034 scenarios.

## 4. Run Evidence

Run:

```powershell
cargo run -p oxcalc-tracecalc-cli -- w034-tracecalc-oracle-deepening-001
```

Evidence:

1. `docs/test-runs/core-engine/tracecalc-reference-machine/w034-tracecalc-oracle-deepening-001/run_summary.json`
2. `docs/test-runs/core-engine/tracecalc-reference-machine/w034-tracecalc-oracle-deepening-001/conformance/engine_diff.json`
3. `docs/test-runs/core-engine/tracecalc-reference-machine/w034-tracecalc-oracle-deepening-001/replay-appliance/validation/bundle_validation.json`
4. `docs/test-runs/core-engine/tracecalc-reference-machine/w034-tracecalc-oracle-deepening-001/scenarios/tc_w034_snapshot_fence_reject_001/result.json`
5. `docs/test-runs/core-engine/tracecalc-reference-machine/w034-tracecalc-oracle-deepening-001/scenarios/tc_w034_capability_fence_reject_001/result.json`
6. `docs/test-runs/core-engine/tracecalc-reference-machine/w034-tracecalc-oracle-deepening-001/scenarios/tc_w034_dynamic_dependency_negative_001/result.json`
7. `docs/test-runs/core-engine/tracecalc-reference-machine/w034-tracecalc-oracle-deepening-001/scenarios/tc_w034_overlay_eviction_after_unpin_001/result.json`
8. `docs/test-runs/core-engine/tracecalc-reference-machine/w034-tracecalc-oracle-deepening-001/scenarios/tc_w034_let_lambda_higher_order_replay_001/result.json`
9. `docs/test-runs/core-engine/tracecalc-reference-machine/w034-tracecalc-oracle-deepening-001/scenarios/tc_w034_replay_equivalent_independent_order_001/result.json`

Summary:

| Measure | Value |
|---|---:|
| scenarios | 21 |
| passed scenarios | 21 |
| W034 new scenarios | 6 |
| validation failures in W034 new scenarios | 0 |
| conformance mismatches in W034 new scenarios | 0 |
| whole-run engine diffs | 21 rows; all `mismatches: []` |
| replay bundle validation | `bundle_valid`, `degraded_capture: false`, `missing_paths: []` |

## 5. Obligation Mapping

| W034 obligation | Evidence in this bead | Carry after this bead |
|---|---|---|
| `W034-OBL-001` stale-fence and reject oracle depth | snapshot mismatch and capability mismatch scenarios added; prior publication-fence and artifact-token rejects remain in the same 21-scenario run | broader matrix expansion remains possible in later conformance/model beads |
| `W034-OBL-002` dynamic dependency negative cases | unresolved dynamic-reference reject scenario added with `dynamic_dependency_failure` counters and no-publish assertion | over/under-invalidation classification beyond this negative case remains later work |
| `W034-OBL-003` overlay retention and eviction pressure | pinned-reader overlay retain/release scenario added with `overlay_retained`, `eviction_eligibility_opened`, and `overlay_released` assertions | TLA overlay/pinned-reader interleavings remain `calc-e77.5` |
| `W034-OBL-004` `LET`/`LAMBDA` carrier breadth | higher-order carrier scenario added with returned callable identity, runtime-effect visibility, and ordinary-value publication policy metadata | general callable publication policy and broad higher-order function semantics remain open |
| `W034-OBL-012` direct OxFml fixture depth | latest OxFml formatting intake checked; no W073 packet constructed by this TraceCalc slice | direct fixture counterpart widening remains carried to `calc-e77.3` and later OxFml bridge work |

## 6. Semantic-Equivalence Statement

This bead widens TraceCalc fixtures, deterministic run artifacts, and the runner's seed-corpus count assertion. It does not change coordinator scheduling, recalc invalidation strategy, publication semantics, reject policy, TraceCalc transition semantics, TreeCalc behavior, OxFml fixture content, Lean models, TLA models, pack decisions, or formatting/display seam meaning.

Observable behavior for existing TraceCalc scenarios remains invariant under this bead. The generated run includes the existing scenarios and reports `passed` result state plus empty oracle/engine mismatches for all of them.

## 7. Verification

Commands run:

| Command | Result |
|---|---|
| JSON parse check over new W034 scenarios and `MANIFEST.json` | passed |
| `cargo test -p oxcalc-tracecalc runner::tests::execute_manifest_produces_passing_conformance_artifacts_for_seed_corpus` | passed |
| `cargo run -p oxcalc-tracecalc-cli -- w034-tracecalc-oracle-deepening-001` | passed; 21 TraceCalc scenarios |
| `cargo test -p oxcalc-tracecalc` | passed; 10 tests |
| `cargo test -p oxcalc-tracecalc-cli` | passed; 0 tests |
| `cargo fmt --all -- --check` | passed |

## 8. Pre-Closure Verification Checklist

| # | Check | Result |
|---|---|---|
| 1 | Spec text and realization notes updated for in-scope items? | yes; this packet records scenario widening, OxFml formatting intake, run evidence, limits, and obligation mapping |
| 2 | Pack expectations updated for affected packs? | yes; new scenarios name pack bindings without promoting pack-grade replay |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes; W034 run artifacts exist for stale/fence rejects, dynamic negative behavior, overlay release, higher-order carrier metadata, and replay-equivalent history projection |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes; Section 6 records that no runtime strategy or policy changed |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes; formatting/W073 changes are a watch input, no W073 packet is constructed here, and no concrete OxFml mismatch appeared |
| 6 | All required tests pass? | yes; see Section 7 |
| 7 | No known semantic gaps remain in declared target? | yes for this TraceCalc oracle-widening target; broader conformance, formal, pack, and direct fixture lanes remain mapped to later W034 beads |
| 8 | Completion language audit passed? | yes; this packet does not claim full formalization, full production conformance, pack-grade replay, Stage 2 promotion, or full formatting/display closure |
| 9 | `WORKSET_REGISTER.md` updated when ordered workset truth changed? | not applicable; ordered workset truth did not change |
| 10 | `IN_PROGRESS_FEATURE_WORKLIST.md` updated when feature-map truth changed? | yes; W034 current-state text now records this TraceCalc oracle-deepening slice |
| 11 | execution-state blocker surface updated? | yes; `calc-e77.2` is represented in `.beads/` |

## 9. Completion Claim Self-Audit

| Step | Audit result |
|---|---|
| Scope re-read | pass; `calc-e77.2` asks for widened TraceCalc reference-machine scenarios and oracle self-checks for stale-fence matrices, dynamic dependency negative cases, overlay retention/eviction pressure, `LET`/`LAMBDA` carrier variants, and replay-equivalent histories |
| Gate criteria re-read | pass; deterministic run artifacts and comparison rows exist for the widened oracle surface |
| Silent scope reduction check | pass; direct OxFml fixture counterpart depth, optimized/core-engine conformance, Lean/TLA widening, pack-grade replay, Stage 2 promotion, and full formatting/display closure are explicitly carried rather than silently claimed |
| "Looks done but is not" pattern check | pass; the paired TraceCalc reference/engine comparison is reported as oracle self-check evidence, not as independent production conformance |
| Result | pass for the `calc-e77.2` declared TraceCalc oracle-deepening target |

## 10. Three-Axis Report

- execution_state: `calc-e77.2_tracecalc_oracle_deepening_authored`
- scope_completeness: `scope_partial`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- open_lanes:
  - `calc-e77.3` optimized/core-engine conformance widening
  - `calc-e77.4` Lean proof-family deepening
  - `calc-e77.5` TLA model-family and contention precondition slice
  - `calc-e77.6` pack capability and continuous scale gate binding
  - `calc-e77.7` W034 closure audit and successor packetization
