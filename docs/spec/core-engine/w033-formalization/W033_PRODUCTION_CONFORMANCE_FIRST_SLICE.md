# W033 Production Conformance First Slice

Status: `calc-uri.9_evidence_packet`
Workset: `W033`
Parent epic: `calc-uri`
Bead: `calc-uri.9`
Created: 2026-05-04

## 1. Purpose

This packet records the first W033 comparison between the TraceCalc oracle surface and a core-engine-backed execution surface.

This is a first slice, not broad optimized-production coverage. The current executable comparison is the TraceCalc engine machine backed by `oxcalc_core` coordinator, recalc, structural, and overlay components, compared against the TraceCalc reference-machine surface over the declared observable artifacts.

## 2. Evidence Inputs

| Input | Path or command | Result |
|---|---|---|
| TraceCalc oracle self-check run | `docs/test-runs/core-engine/tracecalc-reference-machine/w033-tracecalc-oracle-self-check-001/` | 12 scenarios passed |
| Deterministic comparison artifact | `docs/test-runs/core-engine/tracecalc-reference-machine/w033-tracecalc-oracle-self-check-001/conformance/engine_diff.json` | 12 rows; 0 rows with mismatches |
| Replay appliance diff projection | `docs/test-runs/core-engine/tracecalc-reference-machine/w033-tracecalc-oracle-self-check-001/replay-appliance/runs/w033-tracecalc-oracle-self-check-001/diff/engine_diff.json` | emitted by runner |
| Rust runner conformance test | `cargo test -p oxcalc-tracecalc execute_manifest_produces_passing_conformance_artifacts_for_seed_corpus` | 1 passed; 0 failed |
| Mismatch count check | PowerShell JSON parse of `conformance/engine_diff.json` | `total=12 mismatched=0` |

The W033 conformance input also depends on `W033_TRACECALC_REFINEMENT_PACKET.md` for the observable-surface definition and `W033_TRACECALC_ORACLE_SELF_CHECK_FIRST_SLICE.md` for oracle self-check evidence.

## 3. Comparison Scope

Included in this first slice:

1. accept and publish,
2. reject is no-publish,
3. pinned view stability,
4. dynamic dependency switch first floor,
5. overlay retention first floor,
6. scale-chain seed metadata,
7. verified-clean no-publication,
8. multi-node DAG publication,
9. publication fence reject,
10. artifact-token reject,
11. fallback reentry,
12. cycle-region reject.

Excluded from this first slice:

1. a separately optimized production engine,
2. TreeCalc local fixture-to-TraceCalc differential comparison,
3. OxFml fixture replay comparison,
4. `LET`/`LAMBDA` callable-carrier comparison,
5. large-scale performance conformance,
6. Stage 2 concurrency,
7. grid/spill/host/UI/file-adapter behavior.

## 4. Important Independence Caveat

The current TraceCalc reference machine and TraceCalc engine machine both execute through the current shared TraceCalc execution path, while that path uses real `oxcalc_core` components for coordinator, recalc, structural snapshot, and overlay behavior.

Therefore this first slice provides:

1. deterministic emitted comparison artifacts,
2. confirmation that the core-engine-backed TraceCalc surface matches the current oracle surface for the covered corpus,
3. validation that conformance mismatch classification and replay projection are currently clean,
4. a baseline for later independent production or TreeCalc comparisons.

It does not yet provide:

1. independent reference-vs-production implementation diversity,
2. broad optimized-engine semantic assurance,
3. a claim that performance or scale runs are semantically trustworthy without further conformance binding.

This independence caveat must be carried into `calc-uri.14` pack/capability binding and `calc-uri.16` closure audit.

## 5. Scenario Classification

| Scenario | Oracle result | Engine result | Mismatches | W033 classification |
|---|---|---|---:|---|
| `tc_accept_publish_001` | `passed` | `passed` | 0 | `conformance_pass_first_slice` |
| `tc_reject_no_publish_001` | `passed` | `passed` | 0 | `conformance_pass_first_slice` |
| `tc_pinned_view_stability_001` | `passed` | `passed` | 0 | `conformance_pass_first_slice` |
| `tc_dynamic_dep_switch_001` | `passed` | `passed` | 0 | `conformance_pass_first_slice` |
| `tc_overlay_retention_001` | `passed` | `passed` | 0 | `conformance_pass_first_slice` |
| `tc_scale_chain_seed_001` | `passed` | `passed` | 0 | `measurement_seed_conformance_pass_first_slice` |
| `tc_verify_clean_no_publish_001` | `passed` | `passed` | 0 | `conformance_pass_first_slice` |
| `tc_multinode_dag_publish_001` | `passed` | `passed` | 0 | `conformance_pass_first_slice` |
| `tc_publication_fence_reject_001` | `passed` | `passed` | 0 | `conformance_pass_first_slice` |
| `tc_artifact_token_reject_001` | `passed` | `passed` | 0 | `conformance_pass_first_slice` |
| `tc_fallback_reentry_001` | `passed` | `passed` | 0 | `conformance_pass_first_slice` |
| `tc_cycle_region_reject_001` | `passed` | `passed` | 0 | `conformance_pass_first_slice` |

## 6. Mismatch Classification Policy

Future non-empty `engine_diff.json` rows must be classified as follows before changing specs or implementation claims:

| Mismatch state | First classification | Next action |
|---|---|---|
| Oracle passed, engine failed, observable surface clear | `implementation_fault` | File implementation/successor bead with repro artifact. |
| Oracle and engine differ, but the expected surface is ambiguous | `spec_gap` | Record in spec-evolution ledger and patch/defer/handoff. |
| Oracle cannot represent the behavior needed for comparison | `tracecalc_oracle_gap` | Widen TraceCalc corpus or self-check before conformance promotion. |
| Difference is internal and observable surface is preserved | `intentional_strategy_difference` | Add semantic-equivalence statement and comparison evidence. |
| Difference depends on missing OxFml facts | `oxfml_handoff_gap` | Add handoff/watch row; do not patch OxFml directly. |
| Difference comes from fixture/projection/comparator error | `fixture_or_adapter_defect` | Correct artifact/tooling before semantic classification. |

## 7. Downstream Obligations

1. `calc-uri.10` should define metamorphic and differential families that can widen beyond this shared-executor comparison.
2. `calc-uri.13` should bridge TraceCalc evidence to OxFml and TreeCalc fixtures.
3. `calc-uri.14` must cap capability/pack claims at this first-slice evidence level.
4. `calc-uri.16` must packetize broader independent production/TreeCalc conformance as successor work unless it is added before closure.

## 8. Status

- execution_state: `production_conformance_first_slice_recorded`
- scope_completeness: `scope_partial`
- target_completeness: `target_partial`
- integration_completeness: `partial`
- open_lanes:
  - broad independent production conformance remains open
  - TreeCalc local-to-TraceCalc differential comparison remains open
  - OxFml fixture bridge remains open
  - LET/LAMBDA carrier comparison remains open
  - pack/capability binding has not yet consumed this evidence

## 9. Post-W033 Successor Note

The successor packet `W033_INDEPENDENT_CONFORMANCE_WIDENING.md` partially addresses the TreeCalc local-to-TraceCalc differential lane for `calc-y0r`.

That successor evidence adds TreeCalc counterpart fixtures, emits an independent conformance packet, and reports 3 exact value matches, 2 no-publication matches, 2 declared local capability gaps, 0 missing artifacts, and 0 unexpected mismatches.

The successor packet still does not promote pack-grade replay, a fully independent evaluator implementation, or continuous differential coverage.
