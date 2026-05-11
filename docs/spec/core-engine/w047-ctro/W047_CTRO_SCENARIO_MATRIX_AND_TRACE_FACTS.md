# W047 CTRO Scenario Matrix And Trace Facts

Status: `calc-aylq.7_dynamic_positive_publication_validated`

Parent workset: `W047 Calc-Time Rebinding Overlay Design Sweep`

Parent bead: `calc-aylq.3`

Predecessor gates:

1. `W047_HISTORICAL_NO_LOSS_CTRO_CROSSWALK.md`
2. `W047_EFFECTIVE_GRAPH_OVERLAY_AND_FRONTIER_REPAIR_SEMANTICS.md`

## 1. Purpose

This packet binds the W047 CTRO scenario matrix to current TraceCalc and TreeCalc surfaces. It states reference semantics, current representation, expected graph/invalidation/order/candidate/publication facts, W047 trace-fact notes, W048 cycle-processing handoff points, W049 proof-carrying trace blockers, and exact blockers for unsupported grid/spill syntax.

This is a design/evidence-plan packet. It does not claim new runtime behavior beyond the artifacts and tests cited here.

## 2. Current Evidence Surface

| Surface | Current evidence | W047 use |
| --- | --- | --- |
| TraceCalc dynamic dependency switch | `docs/test-corpus/core-engine/tracecalc/hand-auditable/tc_dynamic_dep_switch_001.json`; validated run under `docs/test-runs/core-engine/tracecalc-reference-machine/w046-indirect-explorer-tracecalc-001/scenarios/tc_dynamic_dep_switch_001/` has `result_state: passed`. | Reference semantics for release/activate dynamic dependency, candidate publication, and trace labels. |
| TraceCalc structural cycle reject | `tc_cycle_region_reject_001`; validated run has `result_state: passed`, `reject_kind: synthetic_cycle_reject`, and counters for cycle groups/reject class. | Reference semantics for current non-iterative structural cycle no-publication behavior. |
| TraceCalc fallback re-entry | `tc_fallback_reentry_001`; validated run has `result_state: passed` and required equality surfaces including published view, reject set, trace labels, counters, and candidate/publication boundary. | Reference semantics for visible fallback/re-entry after dynamic dependency failure. |
| TreeCalc static recalc | `tc_local_recalc_after_constant_edit_001` under `docs/test-runs/core-engine/treecalc-local/w025-treecalc-local-baseline/`. | Current local graph-stable comparator. |
| TreeCalc unresolved dynamic reject | `tc_local_dynamic_reject_001` and unit test `local_treecalc_engine_emits_runtime_effect_for_dynamic_reference`. | Current local unsupported dynamic target behavior: runtime effect overlay plus `DynamicDependencyFailure`, no publication. |
| TreeCalc resolved dynamic publication | Unit test `local_treecalc_engine_publishes_resolved_dynamic_reference_shape_update`; W046 indirect explorer case `tc_local_dynamic_resolved_publish_001`. | Current local positive dynamic dependency shape-update floor. |
| TreeCalc dynamic release/reclassification post-edit | W046 indirect explorer cases `tc_local_dynamic_release_reclassification_post_edit_001` and `tc_local_dynamic_release_reclassification_auto_post_edit_001`. | Current local evidence that dynamic release/reclassification invalidation is treated as rebind pressure rather than a clean value-only match. |
| TreeCalc dynamic target switch with downstream dependent | W047 fixture `tc_local_dynamic_target_switch_downstream_publish_001`; validated run under `docs/test-runs/core-engine/treecalc-local/w047-ctro-dynamic-positive-publication-001/`. | Current local CTRO evidence that a dynamic owner can release one target, activate another, recalculate a downstream dependent, and publish value plus overlay consequences atomically. |
| TreeCalc structural cycle reject | Unit test `local_treecalc_engine_rejects_cycles_in_formula_family`. | Current local no-publish cycle behavior: `SyntheticCycleReject`, no candidate result, no publication bundle. |
| TreeCalc rebind/fallback reject | Unit test `local_treecalc_engine_rejects_rerun_when_invalidation_requires_rebind`; `tc_local_rebind_after_rename_001/post_edit`. | Current local conservative re-entry/fallback boundary when rebind is required. |
| TreeCalc shape/topology reject overlay | `tc_local_shape_topology_reject_001` and `tc_local_post_edit_shape_topology_overlay_001`. | Current local exact blocker for spill/region semantics: shape/topology fact is runtime-visible but not evaluated. |

## 3. Scenario Matrix

| Scenario | Reference semantics | Current TraceCalc representation | Current TreeCalc representation | Expected W047 facts | Route/blocker |
| --- | --- | --- | --- | --- | --- |
| static value edit | A value-only edit invalidates dependents through existing graph; graph shape and overlay state do not change; publication may carry value deltas only. | `tc_accept_publish_001` / `tc_multinode_dag_publish_001` cover accept/publish families; no CTRO-specific overlay needed. | `tc_local_recalc_after_constant_edit_001` covers graph-stable recalc with zero dependency-shape updates. | `G_struct` unchanged; `O_published` unchanged; invalidation closure reaches downstream dependents; topological order remains acyclic; candidate has value deltas and no overlay deltas; publication bundle has no dependency-shape updates. | Covered as comparator. |
| dynamic target switch | Runtime dependency changes from old target to new target under same structural snapshot; old overlay edge releases, new overlay edge activates; candidate values and overlay consequences publish atomically. | `tc_dynamic_dep_switch_001` emits `release_dynamic_dep`, `activate_dynamic_dep`, runtime release/activation effects, and publishes `cand1` as `pub1`. | W047 fixture `tc_local_dynamic_target_switch_downstream_publish_001` seeds the previously published dynamic runtime effect, switches the dynamic target, emits `release_dynamic_dep` and `activate_dynamic_dep`, recalculates the owner and downstream dependent, and publishes both values. | `O_candidate` contains release+activation or resolved bind; `G_eff_next` replaces old runtime edge; invalidation includes owner and downstream dependents; order repaired or fallback; candidate carries value + dependency-shape updates; publication commits value and overlay together. | Covered for bounded TreeCalc dynamic-target switch and downstream publication. Broad Excel/INDIRECT observation and cycle-sensitive variants stay W048. |
| unresolved dynamic target | Dynamic dependency cannot be resolved safely; no value publication and no overlay commit. Runtime effect may survive as diagnostic evidence. | Negative dynamic scenarios exist, including `tc_w034_dynamic_dependency_negative_001`; fallback semantics also covered by `tc_fallback_reentry_001`. | `tc_local_dynamic_reject_001` and `local_treecalc_engine_emits_runtime_effect_for_dynamic_reference` reject with `DynamicDependencyFailure`, emit `runtime_effect.dynamic_reference`, and project a dynamic dependency overlay as diagnostic. | `O_candidate` is diagnostic only; reject kind is dynamic dependency failure; publication bundle absent; prior `O_published` remains effective; trace records missing/unresolved dynamic fact. | Covered for unsupported local dynamic target. |
| dynamic target with downstream dependent | When a dynamic target changes, dependents of the dynamic owner must be invalidated and recalculated or blocked under the repaired effective graph. | No direct scenario with a downstream dependent tied to dynamic-owner output was found in the current validated TraceCalc set. | W047 fixture `tc_local_dynamic_target_switch_downstream_publish_001` covers the bounded TreeCalc case: node `3` switches from dynamic target node `2` to node `4`, downstream node `5` depends on node `3`, and post-edit publication yields `3 = 7`, `5 = 8`. | Trace must show reverse closure through `G_eff_next`, downstream needed state, repaired order, downstream candidate value, and atomic publication. Current TreeCalc artifacts show invalidation of owner/downstream, candidate target set `[3, 5]`, release+activation dependency-shape updates, and atomic publication. | Covered for bounded TreeCalc local evidence. TraceCalc and broad Excel/INDIRECT comparison remain future evidence, with cycle-sensitive variants in W048. |
| spill expansion | Runtime shape grows; new produced cells or region members and their dependents enter graph/invalidation before publication. | No current TraceCalc grid/spill expansion fixture. | TreeCalc has shape/topology-sensitive runtime-effect rejection/overlay cases, not positive spill expansion. | `activate_region`, new region members, dependent invalidation, stale/new outputs, candidate shape/topology deltas, atomic value+shape publication. | Exact blocker: OxCalc local TreeCalc has no grid/spill substrate. Positive spill semantics require later grid/host work; W048 owns cycle-sensitive spill probes. |
| spill contraction | Runtime shape shrinks; released outputs and stale consumers are explicitly represented; no silent clean state for consumers. | No current TraceCalc grid/spill contraction fixture. | TreeCalc shape/topology cases reject and project diagnostic overlays only. | `release_region`, released output set, stale consumer set, repaired invalidation/order, no stale published cells surviving as clean values without policy. | Exact blocker: no local grid/spill substrate. |
| structural direct cycle | Structural graph contains an SCC; current non-iterative Stage 1 profile rejects/no-publishes rather than evaluating through the cycle. | `tc_cycle_region_reject_001` validates `synthetic_cycle_reject`, cycle-region trace labels, and no changed published view. | `local_treecalc_engine_rejects_cycles_in_formula_family` rejects with `SyntheticCycleReject`; no candidate result, publication bundle, runtime effects, or overlays. | `G_struct` and `G_eff` SCCs recorded; invalidation marks cycle region blocked; reject record has cycle provenance; no publication bundle; old published values remain. | Covered for structural cycle floor. W048 owns broader Excel comparison and iterative-profile questions. |
| CTRO self-cycle | Candidate overlay points a formula back to itself; it must hit the same no-publication cycle policy as structural self-cycle. | No current TraceCalc fixture distinguishes candidate-overlay self-cycle from structural cycle. | No current TreeCalc candidate-overlay self-cycle fixture. | `O_candidate` edge owner -> owner; `G_eff_next` SCC self-loop; `CycleBlocked` / `SyntheticCycleReject`; no value publication; no overlay commit; diagnostic provenance `candidate_overlay`. | W048 handoff: required cycle-processing fixture and Excel probe. |
| CTRO multi-node cycle | Candidate overlay creates a two-or-more-node SCC after one or more runtime dependency facts are observed. | No current TraceCalc candidate-overlay SCC fixture. | No current TreeCalc candidate-overlay SCC fixture. | `G_eff_next` SCC group with structural+overlay edge provenance; deterministic group ordering; no partial overlay commit; downstream state policy recorded. | W048 handoff. |
| CTRO cycle release | A previously blocked or rejected dynamic cycle becomes acyclic after a later dynamic target change; re-entry and downstream invalidation must be deterministic. | No current TraceCalc cycle-release fixture. | No current TreeCalc cycle-release fixture. | prior blocked region, new acyclic `O_candidate`, release of cycle edge, re-entry frontier, downstream invalidation, publication/no-publication policy. | W048 handoff. |
| conservative fallback/rebuild | If local repair cannot prove semantic equivalence, fallback/reject is visible and no accepted overlay commits through the unsafe path. | `tc_fallback_reentry_001` rejects first dynamic attempt, re-admits work, then publishes a later candidate. | Rebind-required and missing-target TreeCalc tests reject with host-injected failure; W046 indirect explorer records fallback counters for rebind cases. | fallback reason, affected work volume, no-publish for rejected attempt, retained old overlay, later re-entry or explicit block, publication only after safe candidate. | Covered for fallback/re-entry pattern, not for every CTRO delta family. |

## 4. W047 Trace-Fact Notes

W047 should record the following facts in scenario outputs where current tooling can expose them. Native sidecar/checker implementation remains W049.

| Fact family | Required fields | Current status |
| --- | --- | --- |
| structural graph | structural snapshot id, node ids, structural edges, reverse edges, cycle groups. | Partly present in TreeCalc `dependency_graph.json`; TraceCalc has scenario graph and trace labels. |
| published overlay | overlay kind, owner, payload identity, compatibility basis, published runtime effect. | TraceCalc has `published_runtime_effects`; TreeCalc has `runtime_effect_overlays` for diagnostic/runtime effects. |
| invalidation | invalidation seeds, reason kinds, impacted order, `requires_rebind`, cycle-blocked nodes. | TreeCalc has `invalidation_closure.json`; TraceCalc has trace labels and counters. |
| candidate overlay | `O_candidate` delta kind, owner, activated/released target or region, effect family, compatibility basis. | TraceCalc dynamic switch carries dependency-shape updates; TreeCalc W047 dynamic target-switch artifacts carry `activate_dynamic_dep` and `release_dynamic_dep` candidate updates plus `dependency_shape_update_observed` trace events. |
| candidate graph | `G_eff_next` forward/reverse edges, SCC groups, provenance of structural versus candidate-overlay edges. | Not native yet. W049 trace/checker target; W048 cycle scenarios must define expected fields. |
| frontier repair | affected nodes, repaired needed set, repaired order, fallback reason when repair is unsafe. | Partial: TraceCalc fallback/re-entry labels; TreeCalc rebind/fallback diagnostics. |
| candidate/publication boundary | candidate result id, publication id, value deltas, dependency-shape updates, runtime effects, reject id where relevant. | Present in current TraceCalc scenario schema and TreeCalc result/explain artifacts at local granularity. |
| no-publish/no-overlay-commit | reject kind, rejected candidate id, preserved old published view, discarded or diagnostic-only candidate overlay. | Present in current reject scenarios, but not with full candidate-overlay graph provenance. |

## 5. W048 Cycle-Processing Handoff Points

W048 owns the following scenario expansion before W049 formalizes cycle behavior:

1. structural self-cycle and multi-node SCC Excel probes;
2. CTRO self-cycle introduced by dynamic target switch;
3. CTRO multi-node cycle introduced by dynamic target switch;
4. CTRO cycle release and deterministic re-entry;
5. downstream dependent behavior when a CTRO cycle is introduced or released;
6. cycle provenance trace field: `structural | published_overlay | candidate_overlay`;
7. non-iterative last-successful-value policy versus future iterative profile initialization.

## 6. W049 Trace/Checker Blockers

W049 should not start by adding checker rows until these blockers are resolved:

1. no native `G_eff_next` trace surface exists for candidate overlay graph composition;
2. no native per-read trace event stream exists for TreeCalc formula evaluation;
3. no native sidecar distinguishes published overlay, candidate overlay, and diagnostic-only reject overlay;
4. no checker rule currently proves no-under-invalidation after CTRO frontier repair;
5. no checker rule currently proves no-overlay-commit after CTRO cycle reject;
6. no non-vacuous formal state model exists for candidate overlay graph repair;
7. no W048 cycle-release fixture exists to drive deterministic re-entry checking.

## 7. Exact Blockers

| Blocker | Scope affected | Required unblock |
| --- | --- | --- |
| no grid/spill substrate in local TreeCalc | spill expansion, spill contraction, spill-region cycle | Host/grid substrate or narrow explicit local region model. |
| no candidate-overlay cycle fixture | CTRO self-cycle, CTRO multi-node cycle | W048 fixture and expected no-publication/reject trace. |
| no cycle-release fixture | CTRO cycle release | W048 release/re-entry scenario with downstream invalidation policy. |
| no native `G_eff_next` sidecar | proof-carrying overlay validation | W049 sidecar/checker design after implementation core lands. |
| no iterative calculation profile | Excel iterative circular calculation | W048 policy decision and later implementation, if admitted. |

## 8. Acceptance Mapping

| `calc-aylq.3` acceptance item | Evidence in this packet |
| --- | --- |
| Reference semantics | Section 3 scenario matrix. |
| Current TraceCalc representation | Sections 2 and 3 TraceCalc columns. |
| Current TreeCalc representation | Sections 2 and 3 TreeCalc columns. |
| Expected graph/invalidation/order/candidate/publication facts | Section 3 expected W047 facts and Section 4 trace-fact notes. |
| W047 trace-fact notes | Section 4. |
| W048 cycle-processing handoff points | Section 5. |
| W049 proof-carrying trace schema blockers | Section 6. |
| Exact blockers for unsupported grid/spill syntax | Sections 3 and 7. |

## 9. Current Status

- execution_state: `calc-aylq.7_dynamic_positive_publication_validated`
- scope_completeness: `scope_complete`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- successor_lanes:
  - W048 circular dependency calculation processing
  - W049 formal/checker/sidecar/readiness successor work
- known_non_w047_validation_gap: none observed in current review
