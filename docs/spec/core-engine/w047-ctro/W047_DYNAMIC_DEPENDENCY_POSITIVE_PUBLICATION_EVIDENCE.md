# W047 Dynamic Dependency Positive Publication Evidence

Status: `calc-aylq.7_dynamic_positive_publication_validated`

Parent workset: `W047 Calc-Time Rebinding Overlay Design Sweep`

Parent bead: `calc-aylq.7`

Predecessor gates:

1. `W047_EFFECTIVE_GRAPH_OVERLAY_AND_FRONTIER_REPAIR_SEMANTICS.md`
2. `W047_CTRO_SCENARIO_MATRIX_AND_TRACE_FACTS.md`
3. `W047_IMPLEMENTATION_ROADMAP_AND_SUCCESSOR_GATES.md`

## 1. Purpose

This packet records the bounded W047 implementation and evidence for dynamic dependency positive publication under the Calc-Time Rebinding Overlay model.

The covered behavior is the TreeCalc/core-engine local floor:

1. previous published dynamic runtime effects are accepted as the local `O_published` input;
2. the current graph's dynamic facts are compared with `O_published`;
3. dependency activation, release, and reclassification are treated as CTRO changes rather than structural rebind requirements;
4. accepted candidates can carry value deltas plus dependency-shape updates;
5. publication commits the value view and published dynamic runtime effect together;
6. downstream dependents of the dynamic owner are invalidated and recalculated in the same local run.

This packet does not claim Excel-compatible `INDIRECT` behavior, grid/spill behavior, CTRO-created cycle behavior, iterative calculation, or W049 formal/checker/sidecar evidence.

## 2. Implementation Surface

| Surface | Change | Evidence |
| --- | --- | --- |
| invalidation reason taxonomy | Added CTRO-specific dynamic dependency reasons: `DynamicDependencyActivated`, `DynamicDependencyReleased`, and `DynamicDependencyReclassified`. These mark affected nodes dirty but do not force the structural rebind gate. | `src/oxcalc-core/src/dependency.rs`; unit test `invalidation_closure_keeps_dynamic_dependency_changes_repairable`. |
| TreeCalc published overlay input | `LocalTreeCalcInput` now accepts `seeded_published_runtime_effects`, allowing a post-edit run to compare the current dynamic dependency facts with the previously published dynamic facts. | `src/oxcalc-core/src/treecalc.rs`, `src/oxcalc-core/src/treecalc_fixture.rs`, `src/oxcalc-core/src/consumer.rs`. |
| CTRO dynamic diff | TreeCalc derives dynamic dependency facts from the current graph and seeded published runtime effects, then emits activation/release shape updates for the candidate. | `dynamic_dependency_shape_updates`, `dynamic_dependency_facts_from_graph`, and `dynamic_dependency_facts_from_runtime_effects` in `treecalc.rs`. |
| positive publication | A dynamic dependency activation can publish a value update and dynamic runtime effect instead of rejecting as host-injected rebind pressure. | Fixture `tc_local_dynamic_addition_auto_post_edit_001`; run `w047-ctro-dynamic-positive-publication-001`. |
| switch with downstream dependent | A dynamic owner can switch from one resolved target to another, publish release+activation shape updates, recalculate a downstream dependent, and publish both values atomically. | Fixture `tc_local_dynamic_target_switch_downstream_publish_001`; run `w047-ctro-dynamic-positive-publication-001`. |
| trace visibility | TreeCalc post-edit result and explain artifacts carry `candidate_result` and `publication_bundle`; trace artifacts emit `dependency_shape_update_observed` for activation and release updates. | `src/oxcalc-core/src/treecalc_runner.rs`; target-switch run artifact paths in Section 3. |

## 3. Deterministic Evidence

Command:

```powershell
cargo run -p oxcalc-tracecalc-cli -- treecalc w047-ctro-dynamic-positive-publication-001
```

Run summary:

| Artifact | Observed fact |
| --- | --- |
| `docs/test-runs/core-engine/treecalc-local/w047-ctro-dynamic-positive-publication-001/run_summary.json` | `case_count: 29`, `expectation_mismatch_count: 0`, result counts: `published: 18`, `rejected: 10`, `verified_clean: 1`. |
| `docs/test-runs/core-engine/treecalc-local/w047-ctro-dynamic-positive-publication-001/conformance/conformance_summary.json` | `conformance_pass_count: 29`, `mismatch_case_count: 0`, `expectation_mismatch_count: 0`. |
| `docs/test-runs/core-engine/treecalc-local/w047-ctro-dynamic-positive-publication-001/cases/tc_local_dynamic_target_switch_downstream_publish_001/post_edit/result.json` | `result_state: published`, target set `[3, 5]`, value updates `3 = 7` and `5 = 8`, dependency-shape updates `activate_dynamic_dep` on `[3, 4]` and `release_dynamic_dep` on `[2, 3]`, and a publication bundle carrying the same published values plus `published_runtime_effects`. |
| `docs/test-runs/core-engine/treecalc-local/w047-ctro-dynamic-positive-publication-001/cases/tc_local_dynamic_target_switch_downstream_publish_001/post_edit/trace.json` | Emits `dependency_shape_update_observed` events for both `activate_dynamic_dep` and `release_dynamic_dep`. |
| `docs/test-runs/core-engine/treecalc-local/w047-ctro-dynamic-positive-publication-001/cases/tc_local_dynamic_target_switch_downstream_publish_001/post_edit/invalidation_closure.json` | Dynamic dependency reasons on node `3` are repairable (`requires_rebind: false`); downstream node `5` is reached by invalidation and recalculated. |

Focused code validation:

```powershell
cargo test -p oxcalc-core treecalc -- --nocapture
```

Observed result: 33 passed, 0 failed, 24 filtered out.

Core-wide validation note:

```powershell
cargo test -p oxcalc-core
```

Observed result after the post-W047 review repair: 57 unit tests passed, 5 upstream-host integration tests passed, and doc-tests passed.

The broader review found that upstream-host fixtures with `trace_function_ids` expectations must request OxFml `PreparedCalls` trace mode explicitly; OxFml's default runtime path is value-only. The fixture harness now opts into `PreparedCalls` only for cases that assert trace function ids, preserving the value-only default for ordinary runtime calls.

## 4. Acceptance Mapping

| `calc-aylq.7` acceptance need | Evidence |
| --- | --- |
| calc-time dependency activation | `tc_local_dynamic_addition_auto_post_edit_001` publishes after a dynamic dependency resolves; target-switch fixture emits `activate_dynamic_dep`. |
| calc-time dependency release | `tc_local_dynamic_target_switch_downstream_publish_001` emits `release_dynamic_dep` against the previous published dynamic target. |
| downstream invalidation | target-switch fixture recalculates owner node `3` and downstream node `5`, publishing `3 = 7` and `5 = 8`. |
| positive publication | target-switch and dynamic-addition cases publish accepted candidates with value deltas and dynamic runtime effects. |
| proof-carrying trace visibility at current local granularity | result/explain artifacts expose candidate/publication bundles; trace artifact exposes dependency-shape update events. Native W049 sidecars remain deferred. |
| no structural rebind overclassification | CTRO dynamic invalidation reasons do not set `requires_rebind`; structural dependency changes still do. |

## 5. Residual Routes

| Residual | Route |
| --- | --- |
| CTRO self-cycle, CTRO multi-node cycle, cycle release, and iterative circular calculation | W048 circular dependency calculation processing. |
| Excel observation packets for structural and CTRO circular references | W048. |
| Grid/spill expansion, contraction, and spill-region cycles | Later grid/host substrate; cycle-sensitive probes start in W048. |
| Native `G_eff_next` sidecar, per-read events, formal overlay proofs, checker hardening, pack/C5/operated readiness | W049 after W048 stabilizes cycle behavior. |
| Upstream-host trace artifact expectations | Repaired in the post-W047 broader runtime review by explicit per-fixture trace-mode selection. |

## 6. Current Status

- execution_state: `calc-aylq.7_dynamic_positive_publication_validated`
- scope_completeness: `scope_complete`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- successor_lanes:
  - W048 structural and CTRO-created cycle calculation processing
  - W049 formal/checker/sidecar/readiness successor work
- known_non_w047_validation_gap: none observed in current review
