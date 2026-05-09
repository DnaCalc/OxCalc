# W046 Semantic Fragment Review Ledger

Status: `bead_closed`

Workset: `W046`

Parent epic: `calc-gucd`

Bead: `calc-gucd.1`

Date: 2026-05-07

## 1. Purpose

This ledger starts the first W046 semantic-fragment review after the cleanup prelude.

The review question is not whether W033-W045 produced enough closure or readiness artifacts. The question is: which concrete calculation-engine fragments now have enough implementation, spec, model, and replay shape to become the W046 semantic proof spine, and which fragments need new formal targets before successor beads proceed.

## 2. Review Stance

W033-W037 produced the useful semantic substrate. W038-W045 mostly produced downstream classification, closure, or readiness material, now archived unless a current W046 bead explicitly distills a semantic fragment from it.

The first-pass finding is uneven but useful:

1. publication, rejection, candidate separation, pinned readers, and overlay retention already have the strongest formal scaffolding;
2. dependency graph build, reverse-edge converse, SCC classification, invalidation closure, rebind, and working-value read discipline have real Rust and replay evidence but need first-class Lean/TLA or executable-rule counterparts;
3. TraceCalc is a meaningful correctness-oracle surface for covered scenarios, but the refinement relation is still mostly artifact/comparison-driven rather than a mechanized semantic relation;
4. the OxFml seam, especially `LET`/`LAMBDA`, has direct witness and boundary inventory artifacts, but needs an effect-signature view to explain which phase owns which authority.

## 3. Fragment Index

| Fragment | Semantic object | Implementation surface | Current spec and evidence | Current formal surface | W046 disposition |
| --- | --- | --- | --- | --- | --- |
| `GPH-001` | Prepared dependency descriptors lowered to graph edges and diagnostics | `src/oxcalc-core/src/dependency.rs` (`DependencyDescriptor`, `DependencyGraph::build`); `src/oxcalc-core/src/treecalc.rs` (`dependency_descriptor_lowering`, `dependency_graph_build_and_cycle_scan`) | `CORE_ENGINE_ARCHITECTURE.md`; `CORE_ENGINE_RECALC_AND_INCREMENTAL_MODEL.md`; TreeCalc emitted dependency graph artifacts | `calc-gucd.2` adds checked Lean/TLA graph-build semantic targets; full Rust line proof remains open | `calc-gucd.3+`: consume graph facts for invalidation, recalc, order, and refinement |
| `GPH-002` | Reverse dependency map as converse of forward edges | `DependencyGraph::build` builds `reverse_edges`; `TraceCalcScenarioPlanner` builds direct and reverse maps | TreeCalc graph JSON and TraceCalc planner artifacts | `formal/lean/OxCalc/CoreEngine/W046DependencyGraph.lean` proves list-level reverse constructor converse; TLA smoke checks bounded converse | `calc-gucd.3`: reverse-reachability closure |
| `SCC-001` | SCC and cycle-region classification | `dependency.rs` Tarjan `find_cycle_groups`; `TraceCalcScenarioPlanner::compute_components` and `is_cycle_group` | `tc_cycle_region_reject_001`; TreeCalc cycle/reject artifacts | `calc-gucd.2` checks cycle-group classification shape; arbitrary finite-graph SCC completeness and Rust Tarjan proof remain open | `calc-gucd.4`/`.5`: cycle-blocked recalc and acyclic order model |
| `INV-001` | Invalidation seeds and reverse-reachability closure | `DependencyGraph::derive_invalidation_closure`; `treecalc.rs` `invalidation_closure_derivation` | `tc_w035_dirty_seed_closure_no_under_invalidation_001`; TreeCalc post-edit artifacts; scale `relative-rebind-churn` | `calc-gucd.3` adds checked Lean/TLA no-under-invalidation model targets over graph facts; full Rust queue proof remains open | `calc-gucd.4+`: consume closure records in recalc/order/refinement lanes |
| `INV-002` | Soft/dynamic dependency transition and rebind gate | `DependencyDescriptorKind::DynamicPotential`; `requires_rebind_on_structural_change`; `treecalc.rs` `rebind_gate_scan`; dynamic shape updates | `tc_w034_dynamic_dependency_negative_001`; `tc_w035_dynamic_dependency_switch_publish_001`; TreeCalc rebind/dynamic post-edit artifacts | `calc-gucd.3` adds checked rebind reason, dynamic transition, and stale-binding no-publish model targets | `calc-gucd.4`/`.5`/`.7`: recalc-state, order/read, and OxFml `INDIRECT` seam follow-through |
| `REC-001` | Recalc tracker legal node-state transitions | `src/oxcalc-core/src/recalc.rs` (`Stage1RecalcTracker`, `NodeCalcState`) | TreeCalc result `node_states`; verified-clean and reject scenarios | `calc-gucd.4` adds checked Lean/TLA pre/post model over Rust tracker and coordinator transition names; full Rust line proof remains open | `calc-gucd.5` consumes tracker preconditions; `calc-gucd.6` consumes candidate/reject/publication events for refinement |
| `ORD-001` | Topological formula order or cycle rejection | `treecalc.rs` `topological_formula_order`; TraceCalc planner component topo order | TreeCalc `evaluation_order`; TraceCalc workset plan artifacts | `calc-gucd.5` adds checked Lean/TLA order/read model for dependency-before-dependent sample shape and cycle-reject terminal contract; full Rust topo algorithm proof remains open | `calc-gucd.6`: bind observable order events into TraceCalc refinement |
| `EVAL-001` | Working-value read discipline | `treecalc.rs` seeds `working_values`, evaluates in `evaluation_order`, inserts computed values after candidate production | simple formula, DAG, verified-clean, dynamic, and LET/LAMBDA evidence rows | `calc-gucd.5` adds checked Lean/TLA stable/prior read model, diagnostic short-circuit, verified-clean no-publication, and no-torn candidate targets | `calc-gucd.6`/`.7`: refinement relation and OxFml effect-boundary follow-through |
| `CAND-001` | Candidate production distinct from publication | `TreeCalcCoordinator::admit_candidate_work`, `record_accepted_candidate_result`, `AcceptedCandidateResult` | W033-W037 TraceCalc and TreeCalc candidate/publication artifacts | `calc-gucd.4` checks record/produce/admit candidate steps as no-publish and publish as accepted-candidate-only; refinement remains open | `calc-gucd.6`: reuse as observable event backbone |
| `REJ-001` | Typed rejection and no-publish semantics | `TreeCalcCoordinator::reject_candidate_work`; `Stage1RecalcTracker::reject_or_fallback`; `reject_run` | publication-fence, dynamic dependency, cycle, and callable reject scenarios | `calc-gucd.4` checks tracker and coordinator rejection no-publish pre/post conditions; failure-cause and seam widening remain open | `calc-gucd.6`, `calc-gucd.7`: connect rejection to evaluation failure and OxFml seam effects |
| `PUB-001` | Atomic publication bundle | `TreeCalcCoordinator::accept_and_publish`; `PublicationBundle`; TreeCalc artifact writer | published-value artifacts; publication bundle and trace markers | strong TLA/Lean publication fragments | supporting invariant for all later semantic beads |
| `TRC-001` | TraceCalc selected-kernel reference machine | `src/oxcalc-tracecalc/src/machine.rs`, `planner.rs`, `runner.rs`, `oracle_matrix.rs` | checked TraceCalc corpus and oracle matrix through W037 | refinement obligations in Lean are abstract; comparison artifacts exist | `calc-gucd.6`: define observable event vocabulary and refinement relation |
| `FML-001` | OxFml prepare/evaluate and formula-language boundary | `treecalc.rs` `prepare_oxfml_formula`, `oxfml_dependency_descriptors`, `evaluate_via_oxfml`; upstream-host fixtures | `CORE_ENGINE_OXFML_SEAM.md`; direct OxFml fixture bridge; upstream-host W037 artifacts | seam proof maps and callable-boundary inventory | `calc-gucd.7`: convert into effect signature and phase-authority laws |
| `FML-002` | Narrow `LET`/`LAMBDA` carrier fragment | TraceCalc and TreeCalc LET/LAMBDA cases; upstream-host direct runtime evidence | `W033_LET_LAMBDA_CARRIER_WITNESS_WIDENING.md`; W034/W035/W037 callable rows | `W034LetLambdaReplay.lean`; `W036CallableBoundaryInventory.lean` | `calc-gucd.7`: keep in scope as carrier/effect fragment, not general OxFunc semantics |
| `SCL-001` | Scale/performance phase timings bound to semantic signatures | `treecalc-scale` runner; `treecalc.rs` phase timer names | million-node grid, fanout, indirect, rebind artifacts; phase timings | no timing proof; semantic binding is evidence-level | `calc-gucd.9`: use only after graph/rebind/recalc semantics are explicit |

## 4. First Review Conclusions

1. `calc-gucd.2` should be the first heavy model bead because graph build and SCC classification are core engine mechanics with less formal weight than publication/reject.
2. `calc-gucd.3` should depend on exact graph facts, not just abstract affected-set vocabulary. Invalidation proof targets must reference the same forward/reverse edge relation as graph build.
3. `calc-gucd.4` aligned the model to the Rust `Stage1RecalcTracker` and `TreeCalcCoordinator` transition names, including the important observation that `mark_dirty` is locally permissive and relies on phase-level invalidation/scheduling guards.
4. `calc-gucd.5` now states the first checked evaluation-order and working-value read-discipline targets, but it is still a bounded semantic model rather than a Rust implementation proof for arbitrary formula DAGs.
5. `calc-gucd.6` should define observable equivalence before adding more comparison rows. The relation must compare values, diagnostics, dependency effects, invalidation records, evaluation order, rejection, publication, and trace events.
6. `calc-gucd.7` should use the algebraic-effects lens as a boundary architecture: OxFml may request reads, reference resolution, diagnostics, callable/carrier effects, and candidates; OxCalc handles graph, invalidation, rejection, and publication authority.

## 5. Initial Effect/Phase Authority Table

| Phase | Allowed effects | Forbidden authority | Primary formal target |
| --- | --- | --- | --- |
| formula preparation | `ResolveStatic`, descriptor emission, bind diagnostics | publish, mutate published values, clear invalidation | descriptor soundness and diagnostic preservation |
| dependency graph build | `EmitDependency`, `EmitDiagnostic` | evaluate values, publish candidates | forward/reverse converse and SCC classification |
| soft/dynamic update | `ResolveDynamic`, dependency-shape transition, rebind seeds | publish stale dynamic bindings | no-under-invalidation and rebind soundness |
| recalc scheduling | mark dirty/needed/evaluating, select order | evaluate out of dependency order | legal tracker transitions and dependency-before-dependent |
| formula evaluation | `ReadValue`, `CallFunction`, `BindLocal`, `EnterLambda`, candidate value/diagnostic production | direct publication | stable/prior working-value read discipline |
| candidate adaptation | produce candidate values, runtime effects, dependency-shape updates | expose as stable view | candidate-not-publication |
| rejection | emit typed reject, preserve diagnostics | mutate published view | reject-is-no-publish |
| publication | atomically commit accepted bundle | partial/torn visibility, evaluator-owned publication | atomic publication and single-publisher authority |

## 6. Next-Bead Inputs

The next W046 bead should start with these exact inputs:

1. `calc-gucd.2`: `dependency.rs`, `planner.rs`, `W046_ENGINE_STATE_TRANSITION_CATALOG.md`, TreeCalc dependency graph JSON, `tc_cycle_region_reject_001`, and a new graph/SCC model target.
2. `calc-gucd.3`: `derive_invalidation_closure`, dynamic/rebind TreeCalc post-edit artifacts, W034 dependency Lean fragments, and a new closure/rebind theorem target.
3. `calc-gucd.4`: closed with `W046_RECALC_TRACKER_TRANSITION_PRE_POST_MODEL.md`, `W046RecalcTrackerTransitions.lean`, `CoreEngineW046RecalcTracker.tla`, and TLC evidence.
4. `calc-gucd.5`: closed with `W046_EVALUATION_ORDER_AND_WORKING_VALUE_READ_DISCIPLINE_MODEL.md`, `W046EvaluationOrderReadDiscipline.lean`, `CoreEngineW046EvaluationOrder.tla`, and TLC evidence.
5. `calc-gucd.6`: TraceCalc machine/planner/runner, oracle matrix, TreeCalc local artifacts, evaluation-order/read decision vocabulary, and an observable-equivalence relation.
6. `calc-gucd.7`: OxFml seam docs, upstream-host fixtures, LET/LAMBDA witness docs, callable-boundary Lean, and the effect/handler-law table.

## 7. Current Status

- execution_state: `calc-gucd.1_closed`
- scope_completeness: `scope_complete`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
