# W046 Engine State Transition Catalog

Status: `bead_closed`

Workset: `W046`

Parent epic: `calc-gucd`

Bead: `calc-gucd.1`

Date: 2026-05-07

## 1. Purpose

This catalog is the next step after `W046_SEMANTIC_FRAGMENT_REVIEW_LEDGER.md`.

It turns the first-pass fragment review into an engine transition surface that the W046 proof/model beads can consume. It does not prove the transitions. It names the state objects, transition boundaries, preconditions, postconditions, invariant targets, evidence inputs, and owner beads needed to start those proofs.

## 2. State Vocabulary

| State family | Objects | Active implementation surface | Proof/model pressure |
| --- | --- | --- | --- |
| `SNAP` | structural snapshot, node ids, formula catalog, seeded published values | `src/oxcalc-core/src/structural.rs`, `treecalc.rs` input setup | snapshot identity must remain stable across graph build, evaluation, candidate adaptation, and publication |
| `FML` | formula binding, prepared OxFml formula, bind diagnostics, formula owner | `treecalc.rs` `prepare_oxfml_formula` path | preparation may emit descriptors and diagnostics, but not publication |
| `DESC` | dependency descriptor, descriptor kind, owner, optional target, carrier detail, rebind flag | `src/oxcalc-core/src/dependency.rs` | descriptor validity and diagnostic preservation |
| `GRAPH` | forward edges, reverse edges, diagnostics, cycle groups | `DependencyGraph::build` | edge soundness, reverse-edge converse, SCC classification |
| `INV` | invalidation seed, invalidation closure, node invalidation record, rebind requirement | `DependencyGraph::derive_invalidation_closure` | reverse-reachability closure, no-under-invalidation, rebind soundness |
| `REC` | node calc state, demand set, execution overlays, dynamic dependency overlays | `Stage1RecalcTracker` | legal transition relation and no-publish states |
| `ORDER` | evaluation order, cycle rejection, impacted nodes | `topological_formula_order`, `TraceCalcScenarioPlanner` | dependency-before-dependent or rejected cycle |
| `WORK` | working values seeded from published values plus prior computed values | `treecalc.rs` `working_values` loop | stable/prior read discipline |
| `CAND` | local candidate, accepted candidate result, dependency shape updates, runtime effects, diagnostics | `LocalEvaluatorCandidate`, `AcceptedCandidateResult` | candidate is not publication |
| `REJ` | reject detail, reject kind, rejected-pending-repair state | `reject_run`, `reject_candidate_work`, `reject_or_fallback` | reject preserves published view |
| `PUB` | publication bundle, published view, published runtime effects, trace markers | `TreeCalcCoordinator::accept_and_publish` | atomic single-publisher commit |
| `TRC` | trace events, counters, scenario artifacts, oracle matrix rows | `src/oxcalc-tracecalc/src/*` | observable equivalence and replay/refinement relation |

## 3. Transition Catalog

| Transition | Reads | Writes | Preconditions | Postconditions | Invariant target | Owner bead |
| --- | --- | --- | --- | --- | --- | --- |
| `T01.PrepareFormula` | `SNAP`, formula binding | prepared formula, bind diagnostics | formula owner exists or diagnostic path records absence | prepared expression has owner; bind diagnostics are carried forward | preparation does not mutate graph, recalc, or publication state | `calc-gucd.7` |
| `T02.LowerDescriptors` | prepared formula | dependency descriptors | prepared formula exists | each descriptor has an owner, kind, carrier detail, and optional target | descriptor lowering is deterministic for a fixed snapshot and formula text | `calc-gucd.2`, `calc-gucd.7` |
| `T03.BuildGraph` | snapshot, descriptors | descriptors by owner, forward edges, diagnostics | descriptor owner/target are checked against snapshot | valid targets become edges; invalid/unresolved descriptors become diagnostics | no edge targets a missing node; diagnostic descriptors are preserved | `calc-gucd.2` |
| `T04.BuildReverseEdges` | forward edges | reverse edge map | graph build has produced forward edge set | each forward edge appears under its target in reverse map | forward/reverse edge converse | `calc-gucd.2` |
| `T05.ClassifySCC` | snapshot, forward edges | cycle groups | graph edge relation is fixed | non-trivial SCCs and self-loops are cycle groups | acyclic nodes are not cycle-blocked; cycle groups are closed under reachability within the SCC | `calc-gucd.2` |
| `T06.SeedInvalidation` | formula owners, explicit seeds, graph cycles | invalidation seed set | structural edit, upstream publication, dependency transition, or default formula-owner seed exists | every seed has reason and target node | rebind reasons are distinguished from recalc-only reasons | `calc-gucd.3` |
| `T07.CloseInvalidation` | reverse edges, seeds, cycle groups | impacted order, invalidation records | reverse edge map is the converse of forward graph | seed nodes and reverse-reachable dependents are recorded | no-under-invalidation: every reverse-reachable dependent from a seed is included | `calc-gucd.3` |
| `T08.MarkDirtyNeeded` | invalidation records, formula owners | `DirtyPending`, `Needed`, demand set, execution overlays | formula owner is in current snapshot | target nodes enter dirty/needed states before evaluation | no evaluation starts from `Clean` without a dirty/needed path | `calc-gucd.4` |
| `T09.SelectEvaluationOrder` | graph, formula owners, invalidation records | evaluation order or cycle rejection | graph is built and cycles classified | acyclic formula owners receive topo order; cycle groups reject or block | dependency-before-dependent for every evaluated edge | `calc-gucd.5` |
| `T10.RebindGate` | invalidation closure, evaluation order | reject or continue | closure records identify `requires_rebind` | any evaluated node requiring rebind rejects before value publication | stale binding cannot publish | `calc-gucd.3`, `calc-gucd.5` |
| `T11.BeginEvaluate` | `Needed` node state, compatibility basis | `Evaluating`, capability fence overlay | node is `Needed` | node enters `Evaluating`; compatibility basis is attached | evaluation is fence-bearing and tracked | `calc-gucd.4` |
| `T12.ReadWorkingValue` | working values, dependency references | evaluator input values | dependency-before-dependent order has been selected | read source is seeded published value or prior computed value | no read from future candidate state | `calc-gucd.5`, `calc-gucd.7` |
| `T13.EvaluateFormula` | prepared formula, working values, environment context | computed value or failure, diagnostics, runtime effects | node is `Evaluating` | success produces value; failure routes to reject path | OxFml evaluation may request effects but cannot publish | `calc-gucd.5`, `calc-gucd.7` |
| `T14.VerifyClean` | computed value, seeded published value | `VerifiedClean`, suppressed publication diagnostics | computed value equals published value | demand cleared; no candidate/publication emitted for node | verified clean is no-publication | `calc-gucd.4`, `calc-gucd.5` |
| `T15.ProduceCandidate` | computed value, runtime effects, dependency shape updates | `PublishReady`, local candidate, accepted candidate result | node is `Evaluating`; value differs or dynamic effects exist | candidate exists separately from publication | candidate-not-publication | `calc-gucd.4`, `calc-gucd.6` |
| `T16.RejectCandidate` | failure, stale fence, rebind, cycle, dynamic dependency error | reject detail, reject log, rejected-pending-repair | in-flight or accepted candidate exists, or recalc node is evaluating/publish-ready | candidate cleared; published view unchanged | reject-is-no-publish | `calc-gucd.4`, `calc-gucd.6`, `calc-gucd.7` |
| `T17.PublishCandidate` | accepted candidate, current published view | publication bundle, new published values, counters | accepted candidate exists and fence basis is compatible | value delta committed atomically; candidate cleared | no torn publication; single publisher | `calc-gucd.4`, `calc-gucd.6` |
| `T18.EmitTraceAndEvidence` | transition outcomes, artifacts, counters | trace events, result JSON, replay projection | run has reached publish, verified-clean, or reject terminal state | observable artifacts encode values, diagnostics, rejects, publications, dependency effects | replay/checker can validate observable relation without trusting all internals | `calc-gucd.6`, `calc-gucd.8` |

## 4. Cross-Transition Invariants

| Invariant | Statement | Current evidence | Formal target |
| --- | --- | --- | --- |
| `INV.GPH.CONVERSE` | every forward edge `(owner,target,descriptor)` appears in `reverse_edges[target]` and every reverse entry originates from a forward edge | TreeCalc dependency graph artifacts; `DependencyGraph::build` | checked Lean/TLA target in `W046_DEPENDENCY_GRAPH_REVERSE_EDGE_AND_SCC_MODEL.md` |
| `INV.GPH.DIAGNOSTIC_PRESERVATION` | unresolved or invalid descriptors are diagnostics, not silent missing edges | dependency diagnostics in TreeCalc artifacts | checked graph-build model target in `W046_DEPENDENCY_GRAPH_REVERSE_EDGE_AND_SCC_MODEL.md` |
| `INV.SCC.CYCLE_CLASSIFICATION` | non-trivial SCCs and self-loops are classified as cycle groups before order selection | `tc_cycle_region_reject_001` | checked SCC classification-shape model in `W046_DEPENDENCY_GRAPH_REVERSE_EDGE_AND_SCC_MODEL.md` |
| `INV.INV.NO_UNDER_INVALIDATION` | seed nodes and all reverse-reachable dependents are in invalidation records | W035 dirty-seed closure scenario; TreeCalc closure artifacts | checked Lean/TLA target in `W046_INVALIDATION_SOFT_REFERENCE_DYNAMIC_REFERENCE_AND_REBIND_MODEL.md` |
| `INV.INV.REBIND_NO_PUBLISH` | nodes with required rebind cannot publish through stale dependency bindings | rebind/dynamic post-edit artifacts | checked rebind gate model in `W046_INVALIDATION_SOFT_REFERENCE_DYNAMIC_REFERENCE_AND_REBIND_MODEL.md` |
| `INV.REC.LEGAL_STATES` | node states follow the declared recalc transition relation | TreeCalc node-state artifacts; TLA Stage 1 | checked Lean/TLA target in `W046_RECALC_TRACKER_TRANSITION_PRE_POST_MODEL.md` |
| `INV.ORDER.BEFORE_DEPENDENT` | if `owner` reads `target`, then `target` is evaluated earlier or read from seeded published state | TreeCalc evaluation-order artifacts | working-value model in `calc-gucd.5` |
| `INV.EVAL.STABLE_PRIOR_READS` | formula evaluation reads only stable published values or prior ordered computed values | implementation loop in `treecalc.rs` | read-discipline theorem in `calc-gucd.5` |
| `INV.CAND.NOT_PUBLICATION` | accepted candidate state is not stable published state | W033-W037 TLA/Lean and replay artifacts | checked recalc/coordinator pre/post model in `W046_RECALC_TRACKER_TRANSITION_PRE_POST_MODEL.md`; refinement widening remains `calc-gucd.6` |
| `INV.REJ.NO_PUBLISH` | rejected work does not advance published values | publication-fence, dynamic, cycle, callable reject artifacts | checked recalc/coordinator no-publish target in `W046_RECALC_TRACKER_TRANSITION_PRE_POST_MODEL.md`; failure-cause widening remains `.6`/`.7` |
| `INV.PUB.ATOMIC` | publication applies one coherent bundle and never exposes a partial value delta | publication bundle artifacts; TLA/Lean | accepted-candidate publication precondition checked in `W046_RECALC_TRACKER_TRANSITION_PRE_POST_MODEL.md`; atomic bundle refinement remains `calc-gucd.6` |
| `INV.TRC.OBS_EQUIV` | TraceCalc and TreeCalc/CoreEngine agree on observable values, diagnostics, dependency effects, invalidation, rejection, publication, and traces for covered fragments | independent conformance and oracle-matrix artifacts | refinement relation in `calc-gucd.6` |

## 5. Minimal Formal Work Products

The successor beads should not start by expanding documentation. They should produce at least one of these work products per lane:

1. graph lane: executable relation or Lean/TLA model for descriptors, forward edges, reverse edges, diagnostics, and SCCs;
2. invalidation lane: reachability model over the graph relation plus rebind/no-publish theorem targets;
3. recalc lane: transition table mapped to `Stage1RecalcTracker` methods and TLA action names;
4. evaluation lane: topological order and working-value read-discipline model;
5. TraceCalc lane: observable event vocabulary and refinement relation;
6. OxFml lane: effect signature, phase handlers, and handler laws for the narrow formula/evaluator boundary.

## 6. Immediate Routing

This catalog gives `calc-gucd.2` a concrete start condition:

1. import or mirror `DependencyDescriptorKind`, `DependencyDescriptor`, `DependencyEdge`, `DependencyDiagnostic`, and `DependencyGraph`;
2. model `T03.BuildGraph`, `T04.BuildReverseEdges`, and `T05.ClassifySCC`;
3. prove or model-check `INV.GPH.CONVERSE`, `INV.GPH.DIAGNOSTIC_PRESERVATION`, and `INV.SCC.CYCLE_CLASSIFICATION`;
4. bind at least one small replay/model fixture to `tc_cycle_region_reject_001` or a new smaller graph fixture.

### 6.1 `calc-gucd.2` Result

`calc-gucd.2` adds `W046_DEPENDENCY_GRAPH_REVERSE_EDGE_AND_SCC_MODEL.md`, `formal/lean/OxCalc/CoreEngine/W046DependencyGraph.lean`, `formal/tla/CoreEngineW046DependencyGraph.tla`, `formal/tla/CoreEngineW046DependencyGraph.smoke.cfg`, and the TLC evidence root `docs/test-runs/core-engine/tla/w046-dependency-graph-001/`.

The result checks the reverse-edge constructor theorem in Lean and model-checks a bounded TLA graph-build transition with valid forward edges, exact reverse converse, untargeted dynamic diagnostic preservation, and non-trivial SCC classification shape. It does not claim a line-by-line Rust Tarjan proof or arbitrary finite-graph SCC completeness.

### 6.2 `calc-gucd.3` Result

`calc-gucd.3` adds `W046_INVALIDATION_SOFT_REFERENCE_DYNAMIC_REFERENCE_AND_REBIND_MODEL.md`, `formal/lean/OxCalc/CoreEngine/W046InvalidationRebind.lean`, `formal/tla/CoreEngineW046InvalidationRebind.tla`, `formal/tla/CoreEngineW046InvalidationRebind.smoke.cfg`, and the TLC evidence root `docs/test-runs/core-engine/tla/w046-invalidation-rebind-001/`.

The result checks a Lean reachability/rebind model and model-checks a bounded TLA invalidation transition with reverse-reachability A->B->C, dependency-added/reclassified dynamic transition seeds, upstream dependent propagation, rebind flag soundness, and rejection before publication. It does not claim full Rust queue proof, full `INDIRECT`/OxFunc semantics, or unbounded TLA verification.

### 6.3 `calc-gucd.4` Result

`calc-gucd.4` adds `W046_RECALC_TRACKER_TRANSITION_PRE_POST_MODEL.md`, `formal/lean/OxCalc/CoreEngine/W046RecalcTrackerTransitions.lean`, `formal/tla/CoreEngineW046RecalcTracker.tla`, `formal/tla/CoreEngineW046RecalcTracker.smoke.cfg`, and the TLC evidence root `docs/test-runs/core-engine/tla/w046-recalc-tracker-001/`.

The result checks a Lean pre/post transition relation and model-checks a bounded TLA recalc tracker/coordinator model with dirty, needed, cycle-blocked closure records, evaluating, verified-clean, publish-ready, rejected-pending-repair, candidate intake, rejection, accepted-candidate publication, and tracker publication-clear paths. It models `mark_dirty` as the current permissive Rust method and keeps the stronger invalidation/scheduling phase guard outside that method. It also records that `CycleBlocked` is currently assigned by invalidation closure records rather than by a `Stage1RecalcTracker` mutator. It does not claim full Rust implementation proof, full cycle policy, evaluation-order/read-discipline proof, TraceCalc refinement, or unbounded TLA verification.

## 7. Current Status

- execution_state: `calc-gucd.1_closed`
- scope_completeness: `scope_complete`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
