# W046 Finite Graph Dataflow And Evaluation-Order Proof Strengthening

Status: `calc-gucd.16_finite_graph_dataflow_order_validated`

Workset: `W046`

Parent epic: `calc-gucd`

Bead: `calc-gucd.16`

## 1. Purpose

This packet strengthens the W046 graph/dataflow/order proof targets beyond single smoke examples.

The scoped goal is to introduce reusable finite graph vocabulary and several checked graph shapes that connect:

1. reverse-edge converse,
2. reverse-reachability invalidation closure,
3. topological dependency-before-dependent order,
4. stable/prior working-value reads,
5. SCC/self-cycle classification,
6. cycle and rebind no-publish consequences.

This packet narrows the gap between phase-local smoke models and a deeper proof. It does not claim a line-by-line Rust proof of Tarjan, a line-by-line Rust proof of `topological_formula_order`, arbitrary finite-graph SCC completeness, or normalized dynamic-dependency projection closure.

## 2. Source Surfaces Reviewed

| Surface | Intake |
| --- | --- |
| `formal/lean/OxCalc/CoreEngine/W046DependencyGraph.lean` | reverse-edge converse and SCC/cycle vocabulary |
| `formal/lean/OxCalc/CoreEngine/W046InvalidationRebind.lean` | reverse reachability and no-under-invalidation vocabulary |
| `formal/lean/OxCalc/CoreEngine/W046EvaluationOrderReadDiscipline.lean` | topological order and stable/prior read vocabulary |
| `formal/tla/CoreEngineW046DependencyGraph.tla` | bounded graph/reverse/SCC smoke model |
| `formal/tla/CoreEngineW046InvalidationRebind.tla` | bounded invalidation/rebind smoke model |
| `formal/tla/CoreEngineW046EvaluationOrder.tla` | bounded evaluation-order/read model |
| `formal/lean/OxCalc/CoreEngine/W046IntegratedSemanticKernel.lean` | integrated publication prerequisites and successor blocker routing |
| `src/oxcalc-core/src/dependency.rs` | Tarjan cycle scan, reverse edges, invalidation closure |
| `src/oxcalc-core/src/treecalc.rs` | `topological_formula_order` and working-value loop |
| `src/oxcalc-tracecalc/src/planner.rs` | TraceCalc finite workset planner, SCC grouping, reverse dependencies |
| TreeCalc and TraceCalc replay roots in Section 8 | concrete chain, DAG, rebind, cycle, and scale evidence |

## 3. Strengthened Finite Vocabulary

The new Lean artifact introduces a reusable finite model surface:

| Concept | Meaning |
| --- | --- |
| `ReverseEdgeFacts` | every forward edge has a reverse record and every reverse record corresponds to a forward edge |
| `ReverseReachable` | seed-to-dependent reachability through reverse dependency edges |
| `ClosureCoversReverseReachable` | no-under-invalidation over a finite edge set, seed set, and closure set |
| `BeforeInOrder` | target appears before owner in an evaluation order |
| `TopologicalForEdges` | every formula-to-formula edge respects dependency-before-dependent order |
| `StableOrPriorReads` | every formula read is stable input or prior ordered formula result |
| `CycleGroupSupported` | cycle groups are non-trivial SCCs or self-loop singletons |
| `FiniteGraphDataflowOrderModel` | combined certificate for graph converse, closure, order, reads, and cycle support |

## 4. Checked Shape Set

| Shape | Covered fact families | Current role |
| --- | --- | --- |
| chain | reverse converse, A->B->C reverse closure, topological order, stable/prior reads | direct recalc-chain proof seed |
| diamond | fan-in order over B/C -> D with stable A input | multi-predecessor order/read proof seed |
| fanout rebind | A/B seed closure over fanout dependents plus rebind no-publish | dynamic/rebind fanout proof seed |
| self-cycle | single-node self-loop cycle support and reject path | self-cycle classification seed |
| two-node SCC | non-trivial SCC support and reject path | multi-node cycle classification seed |

## 5. Checked Lean Artifact

Artifact: `formal/lean/OxCalc/CoreEngine/W046FiniteGraphDataflowOrder.lean`

Checked command:

```powershell
lean formal\lean\OxCalc\CoreEngine\W046FiniteGraphDataflowOrder.lean
```

Result: passed.

Checked theorem targets:

| Theorem | Contract |
| --- | --- |
| `model_reverse_converse` | model exposes reusable forward/reverse converse fact |
| `model_no_under_invalidation` | model exposes seed-to-dependent closure coverage |
| `model_order_respects_formula_edges` | model exposes topological order obligation |
| `model_reads_are_stable_or_prior` | model exposes stable/prior read obligation |
| `model_cycle_groups_supported` | model exposes cycle group support obligation |
| `chain_reverse_converse` | chain reverse records match chain forward edges |
| `chain_reachable_A_to_C` | reverse reachability reaches transitive dependent C from seed A |
| `chain_closure_covers` | chain closure covers seed and reverse-reachable dependents |
| `chain_topological` | chain formula order respects formula-to-formula edge B before C |
| `chain_stable_or_prior` | chain reads use stable A or prior B |
| `diamond_topological` | diamond fan-in edges order B and C before D |
| `diamond_stable_or_prior` | diamond reads use stable A or prior B/C |
| `self_loop_group_supported` | self-loop singleton is supported as a cycle group |
| `two_node_scc_group_supported` | two-node non-trivial SCC is supported as a cycle group |
| `chain_model_reaches_all_dependents` | model-level closure witness reaches chain transitive dependent |
| `diamond_edges_ordered` | diamond shape exposes reusable order target |

## 6. Checked TLA Artifact

Artifacts:

1. `formal/tla/CoreEngineW046FiniteGraphDataflowOrder.tla`
2. `formal/tla/CoreEngineW046FiniteGraphDataflowOrder.smoke.cfg`
3. `docs/test-runs/core-engine/tla/w046-finite-graph-dataflow-order-001/`

Checked command:

```powershell
powershell -ExecutionPolicy Bypass -File scripts\run-tlc.ps1 formal\tla\CoreEngineW046FiniteGraphDataflowOrder.tla formal\tla\CoreEngineW046FiniteGraphDataflowOrder.smoke.cfg
```

Result: passed.

TLC summary:

| Field | Value |
| --- | --- |
| TLC version | `2.19 of 08 August 2024` |
| states generated | `11` |
| distinct states | `6` |
| queue left | `0` |
| complete-state depth | `2` |
| result | no error found |

Checked invariants:

1. `TypeInvariant`
2. `ReverseConverse`
3. `ClosureCoversExpectedDependents`
4. `OrderRespectsKnownFiniteShapes`
5. `StablePriorReadShapes`
6. `CyclesRejectRatherThanOrder`
7. `RebindRejectsRatherThanPublishes`
8. `PublishedOnlyAcyclicNoRebind`

Checked TLA shapes:

1. `chain`,
2. `diamond`,
3. `fanout_rebind`,
4. `self_cycle`,
5. `two_node_scc`.

## 7. Binding Root

Canonical binding root: `docs/test-runs/core-engine/refinement/w046-finite-graph-dataflow-order-001/`

Checked-in artifacts:

1. `run_summary.json`
2. `finite_graph_binding_register.json`

Binding summary:

| Metric | Value |
| --- | --- |
| checked shapes | `5` |
| exact blockers | `4` |
| unexpected mismatches | `0` |
| missing referenced artifacts | `0` |

## 8. Replay And Evidence Roots

| Root | Use |
| --- | --- |
| `docs/test-runs/core-engine/treecalc-local/w037-optimized-core-conformance-treecalc-001/cases/tc_local_recalc_chain_after_constant_edit_001/result.json` | chain order and value evidence |
| `docs/test-runs/core-engine/treecalc-local/w037-optimized-core-conformance-treecalc-001/cases/tc_local_recalc_chain_after_constant_edit_001/post_edit/result.json` | post-edit chain recomputation and publication evidence |
| `docs/test-runs/core-engine/tracecalc-reference-machine/w037-tracecalc-observable-closure-001/scenarios/tc_multinode_dag_publish_001/result.json` | multi-node DAG/fan-in proof seed |
| `docs/test-runs/core-engine/refinement/w046-tracecalc-refinement-kernel-001/kernel_binding_register.json` | selected-kernel DAG binding row |
| `docs/test-runs/core-engine/treecalc-local/w037-optimized-core-conformance-treecalc-001/cases/tc_local_rebind_after_rename_001/post_edit/result.json` | rebind no-publish evidence |
| `docs/test-runs/core-engine/treecalc-scale/million_relative_rebind_f8_r1/run_summary.json` | large fanout rebind seed count evidence |
| `docs/test-runs/core-engine/tracecalc-reference-machine/w037-tracecalc-observable-closure-001/scenarios/tc_cycle_region_reject_001/result.json` | cycle reject evidence |
| `docs/test-corpus/core-engine/tracecalc/hand-auditable/tc_cycle_region_reject_001.json` | hand-auditable cycle fixture |
| `docs/test-runs/core-engine/tla/w046-finite-graph-dataflow-order-001/run_summary.json` | checked finite shape model |

## 9. Exact Blockers And Limits

| Blocker | Meaning | Successor route |
| --- | --- | --- |
| `rust_tarjan_line_proof_not_discharged` | SCC/cycle shape obligations are stronger, but the Rust Tarjan implementation is not line-proven. | `calc-gucd.18` or later proof tranche |
| `rust_topological_queue_line_proof_not_discharged` | topological-order/read-discipline shapes are stronger, but the Rust queue implementation is not line-proven. | `calc-gucd.18` or later proof tranche |
| `arbitrary_finite_graph_scc_completeness_not_discharged` | The artifacts define reusable finite vocabulary and several graph shapes, not arbitrary finite graph SCC completeness. | later formal proof strengthening if required |
| `normalized_dynamic_dependency_projection_gap_inherited` | Dynamic dependency projection and normalized invalidation comparison blockers from `.6` remain. | `calc-gucd.17`/`.18` |

## 10. Handoff Assessment

No new OxFml handoff is filed by this bead.

Reason:

1. The bead is OxCalc-local graph/dataflow/order proof strengthening.
2. It does not change shared FEC/F3E or OxFml evaluator-facing text.
3. OxFml seam watch/blocker lanes from `.7` are not promoted here.

## 11. Successor Obligations

| Successor bead | Consumes from this packet |
| --- | --- |
| `calc-gucd.17` | finite graph/dataflow/order facts as proof-carrying trace validation predicates |
| `calc-gucd.18` | strengthened shape vocabulary and exact Rust line-proof blockers for refinement bridge |
| `calc-gucd.8` | semantic-object coverage rows keyed to graph/dataflow/order facts |
| `calc-gucd.9` | chain/diamond/fanout/cycle semantic signatures for scale/performance rows |
| `calc-gucd.10` | consequence reassessment using strengthened graph/order evidence and blockers |
| `calc-gucd.11` | closure audit over finite proof strengthening residuals |

## 12. Validation

| Command | Result |
| --- | --- |
| `lean formal\lean\OxCalc\CoreEngine\W046FiniteGraphDataflowOrder.lean` | passed |
| `powershell -ExecutionPolicy Bypass -File scripts\run-tlc.ps1 formal\tla\CoreEngineW046FiniteGraphDataflowOrder.tla formal\tla\CoreEngineW046FiniteGraphDataflowOrder.smoke.cfg` | passed |
| JSON parse/reference check for `w046-finite-graph-dataflow-order-001` roots | passed |
| `rg -n "\b(sorry|admit|axiom)\b" formal\lean\OxCalc\CoreEngine\W046FiniteGraphDataflowOrder.lean` | passed; no matches |

## 13. Semantic-Equivalence Statement

This bead changes formal artifacts, evidence metadata, spec/status documents, and bead state only.

Observable OxCalc behavior is invariant under this bead. It does not change dependency graph construction, SCC/cycle classification, invalidation closure, topological ordering, working-value reads, recalc tracker behavior, TraceCalc execution, TreeCalc/CoreEngine execution, OxFml/OxFunc behavior, rejection, publication, pack policy, proof-service policy, performance behavior, or service readiness.

## 14. Pre-Closure Verification Checklist

| # | Check | Result |
|---|-------|--------|
| 1 | Spec text and realization notes updated for all in-scope items? | yes; this packet, W046 README/status surfaces, Lean/TLA artifacts, and binding register record finite proof strengthening |
| 2 | Pack expectations updated for affected packs? | yes; no pack expectation changed by this proof-strengthening bead |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes; Section 8 lists TreeCalc/TraceCalc roots and the new TLA root |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes; no policy/strategy change, invariant statement in Section 13 |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes; no normative FEC/F3E change and no OxFml handoff needed |
| 6 | All required tests pass? | yes; see Section 12 |
| 7 | No known semantic gaps remain in declared scope? | yes for the declared `calc-gucd.16` target; Rust Tarjan/topo line proof, arbitrary finite SCC completeness, and dynamic projection gaps remain explicit blockers |
| 8 | Completion language audit passed? | yes; no full Rust proof, arbitrary finite graph proof, proof-carrying trace checker, Rust refinement bridge, full TLA, or release-grade proof is claimed |
| 9 | `WORKSET_REGISTER.md` updated when ordered workset truth changed? | yes; no ordered workset register change required |
| 10 | `IN_PROGRESS_FEATURE_WORKLIST.md` updated when feature-map truth changed? | yes; current semantic bead moved to `calc-gucd.17` |
| 11 | execution-state blocker surface updated? | yes; `.beads/` owns `calc-gucd.16` state |

## 15. Completion Claim Self-Audit

| Step | Result |
| --- | --- |
| Scope re-read | pass; bead asks for reusable finite graph/order/invalidation vocabulary, several graph shapes or blockers, checked commands, replay roots, and explicit Rust Tarjan/topo residuals |
| Gate criteria re-read | pass; Lean/TLA artifacts, five checked shapes, evidence roots, exact blockers, and successor routes are recorded |
| Silent scope reduction check | pass; Rust line proof, arbitrary SCC completeness, and dynamic projection closure are explicitly not hidden |
| "Looks done but is not" pattern check | pass; the packet says the model is strengthened and scoped, not a full mechanized proof of all graph behavior |
| Include result | pass; checklist, self-audit, semantic-equivalence statement, and three-axis report are included |

## 16. Current Status

- execution_state: `calc-gucd.16_closed`
- scope_completeness: `scope_complete`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
