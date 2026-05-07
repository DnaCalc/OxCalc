# OxCalc W033-W045 Engine Formalization Review Catalog

This catalog is the backbone for the densified showcase deck. It separates three surfaces for each engine area:

- **Engine design**: the Rust/spec mechanism that exists or is architecturally required.
- **Formal target**: the intended semantic object, invariant, precondition, postcondition, or theorem family.
- **Artifacts today**: checked Lean/TLA/replay/scale evidence currently present through W045.

The key finding is mixed: OxCalc has strong evidence for the shape of the engine and several real semantic proof/model fragments, but it does not yet have a full mechanized semantic proof of the Rust engine. The next phase should turn the catalog below into first-class Lean/TLA/TraceCalc objects, not merely add more closure classifiers.

## Current Formal Coverage Types

| Coverage label | Meaning | Examples present through W045 |
| --- | --- | --- |
| `proved_fragment` | A small semantic definition/theorem is present in Lean. | `applyReject_noPublish`, `applyPublish_atomic`, `conservativeAffectedSet_refl`, `emptyOverlays_safe` in `W033FirstSlice.lean`. |
| `checked_classifier` | Lean proves evidence-row classification, row counts, or no-promotion predicates. | W035-W045 assumption, Rust, Lean/TLA, Stage2, and promotion-gate files. |
| `bounded_model` | TLA+ model checking explores bounded transition systems and invariants. | `CoreEngineStage1.tla`, `CoreEngineW035NonRoutineInterleavings.tla`, `CoreEngineW036Stage2Partition.tla`. |
| `replay_evidence` | Deterministic TraceCalc/TreeCalc/OxFml/runner artifacts exercise behavior. | TreeCalc local, TraceCalc reference-machine, implementation-conformance, stage2-replay. |
| `scale_signature` | Large synthetic models are validated by closed-form expectations and phase timing. | `TreeCalcScaleProfile`, phase split, grid/fanout/indirect/rebind profiles. |
| `formal_target_only` | The theorem/invariant is clear but not yet mechanized against the implementation. | SCC correctness, full invalidation closure soundness/minimality, topological-order proof, soft-reference/INDIRECT totality. |

## Engine Formalization Catalog

| ID | Engine area | Engine design surface | Formal target | Artifacts today | W045 status |
| --- | --- | --- | --- | --- | --- |
| `STATE-01` | structural snapshots | `StructuralSnapshot`, immutable snapshot ids, stable node identity | snapshots are immutable inputs to dependency/recalc/coordinator phases; runtime work never mutates canonical structure | core architecture spec; TLA snapshot constants; TreeCalc artifacts carry snapshot ids | `replay_evidence`; deeper snapshot algebra planned |
| `FML-01` | formula catalog | `TreeFormulaCatalog`, `TreeFormulaBinding`, OxFml preparation | every formula binding either prepares to a bound formula with descriptors or rejects with diagnostics | `prepare_oxfml_formula`; direct OxFml and upstream-host cases | `replay_evidence`; totality proof planned |
| `FML-02` | LET/LAMBDA seam | OxFml/OxFunc-backed runtime candidate adaptation | callable value-carrier identity is tracked separately from metadata projection and full OxFunc kernel semantics | W033/W035/W044/W045 Lean carrier rows; W045 LET/LAMBDA cases | `checked_classifier` + `replay_evidence`; full kernel out of OxCalc scope |
| `REF-01` | reference lowering | `TreeReference` variants and `descriptor_kind` | reference kind classification is total and preserves semantic family | formula unit tests; descriptor taxonomy; W035 dependency closure row | partial `proved_fragment`/`checked_classifier`; full lowering proof planned |
| `DEP-01` | edge construction | `DependencyGraph::build` creates edges for targetful descriptors and diagnostics for untargeted descriptors | edge exists iff descriptor has a valid target; diagnostic kind matches untargeted descriptor family | TreeCalc dependency graphs; implementation tests; W045 optimized/core evidence | `replay_evidence`; Lean graph model planned |
| `DEP-02` | reverse edges | graph stores `edges_by_owner` and `reverse_edges` | reverse edge index is the exact converse of forward edges | code path and fixture artifacts | `formal_target_only` beyond tests |
| `SCC-01` | cycle groups | Tarjan-style `find_cycle_groups` | `cycle_groups` are exactly non-trivial SCCs plus self-cycles | cycle reject tests and artifacts | `replay_evidence`; mechanized SCC proof planned |
| `INV-01` | invalidation seeds | default, structural context, and dependency transition seed derivation | seeds cover structural edits, dependency add/remove/reclassify, upstream publications without under-invalidation | post-edit fixtures, W044/W045 dynamic transition evidence | partial `replay_evidence`; no-under-invalidation proof only classified |
| `INV-02` | invalidation closure | `derive_invalidation_closure` BFS over reverse edges | closure soundness: every reverse-reachable dependent from a seed is included | TreeCalc invalidation artifacts; scale runner | `formal_target_only` for full reachability proof |
| `INV-03` | rebind requirement | `requires_rebind` from seed reason and structural context | rebind flag iff some carried reason requires rebind; local evaluation rejects before stale-bind publication | rebind fixtures; relative-rebind scale profile | `replay_evidence`; soft-reference totality planned |
| `ORDER-01` | evaluation order | `topological_formula_order` or reject | every evaluated node appears after dependencies or graph rejects | cycle and publish/recalc fixtures | `replay_evidence`; topological proof planned |
| `REC-01` | recalc states | `Stage1RecalcTracker` over `Clean`, `DirtyPending`, `Needed`, `Evaluating`, `VerifiedClean`, `PublishReady`, `RejectedPendingRepair`, `CycleBlocked` | allowed transition relation and state preconditions are explicit | Rust state machine; TLA `NodeStates`; unit tests | `bounded_model` + tests; Lean transition proof partial |
| `REC-02` | verified-clean branch | if computed value matches published value, mark verified clean and suppress publication | verified clean preserves published view and demand set is cleared | TreeCalc verified-clean cases; TLA verify-clean no publish | `bounded_model` + `replay_evidence` |
| `EVAL-01` | working values | evaluation uses seeded published values plus prior computed updates | evaluation reads only stable/prior ordered values; failures short-circuit to reject | TreeCalc evaluation loop artifacts | `formal_target_only` beyond replay |
| `EVAL-02` | OxFml runtime candidate | `evaluate_via_oxfml`, residual gate, commit-bundle validation | accepted OxFml result validates into local success; rejected/residual outcome becomes no-publish failure | direct OxFml/upstream-host cases; OxFml seam docs | `replay_evidence`; seam proof remains partial |
| `DYN-01` | dynamic references | `DynamicPotential` and resolved dynamic descriptors emit shape updates/effects | dynamic dependency add/remove/reclassify is explicit, replay-visible, and never hidden mutation | W044 mixed dynamic transition, W045 carried evidence | partial `replay_evidence`; broad coverage exact blocker |
| `OVL-01` | overlays | protected execution/dynamic/capability/shape overlays | protected overlays are not evictable; released overlays become eligible | W033 `emptyOverlays_safe`; W035 TLA multi-reader model; TreeCalc overlay cases | `proved_fragment` + `bounded_model`; broader lifecycle planned |
| `REJ-01` | rejection | `reject_run`, `reject_candidate_work`, `RejectDetail` | reject preserves published view and emits typed no-publish evidence | W033 Lean `applyReject_noPublish`; TLA `RejectIsNoPublish`; many fixtures | strongest current semantic area |
| `PUB-01` | publication | `accept_and_publish`, `PublicationBundle`, coordinator counters | accepted publication is atomic and comes only from accepted candidate | W033 Lean `applyPublish_atomic`; TLA `NoTornPublication`; TreeCalc publish fixtures | strongest current semantic area |
| `PUB-02` | candidate vs publication | candidate/admission/result are distinct from publication | no candidate identity is treated as publication; rejected/candidate work cannot leak | W033 Lean/TLA and coordinator spec | `proved_fragment` + `bounded_model` |
| `FENCE-01` | snapshot/capability fences | compatibility basis, artifact token, snapshot id, capability profile | accepted candidate requires current compatible fences; stale/mismatch rejects without publish | TLA `AcceptedCandidateRequiresFences`; fixture and upstream-host evidence | partial; snapshot/capability breadth exact blocker |
| `S2-01` | Stage2 partition | bounded partition and scheduler models | observable results invariant under partition/permutation scheduling before promotion | W036/W045 Stage2 TLA/replay evidence and Lean predicates | bounded/predicate evidence; no Stage2 promotion |
| `SCALE-01` | scalable models | grid, fanout, dynamic indirect stripes, relative rebind churn | phase costs separated; closed-form expected values validate recalc | scale runner and W045 scale artifacts | `scale_signature`; continuous service unpromoted |
| `ASSURE-01` | TraceCalc oracle | reference machine, retained failures, differential runners | TreeCalc behavior refines TraceCalc for selected kernels and mismatch authority is explicit | TraceCalc W033-W045 roots, diversity-seam roots | strong replay evidence; full equivalence proof planned |

## Dense Transition Catalog For Deck Use

| Transition | Rust precondition | Rust postcondition | Formal artifact today | Missing formalization |
| --- | --- | --- | --- | --- |
| `mark_dirty(n)` | `n` may be any known node | `state[n] = DirtyPending`; protected execution overlay inserted | TLA `A1MarkDirty`; recalc unit test | Lean transition theorem over tracker map |
| `mark_needed(n)` | `state[n] = DirtyPending` | `state[n] = Needed`; `n` in demand set | TLA `A2MarkNeeded`; recalc unit test | proof that no other nodes change except overlays/demand |
| `begin_evaluate(n, basis)` | `state[n] = Needed` | `state[n] = Evaluating`; capability-fence overlay protected | TLA `A3BeginEvaluate` | compatibility-basis invariant tied to publication fence |
| `verify_clean(n)` | `state[n] = Evaluating` and computed equals published | `state[n] = VerifiedClean`; demand removed; no publication bundle | TLA verify-clean decision; TreeCalc verified-clean fixture | value-equality theorem and no-publish proof across TreeCalc |
| `produce_candidate_result(n)` | `state[n] = Evaluating` and computed differs | `state[n] = PublishReady`; candidate overlay protected | recalc unit test; TreeCalc publish fixture | candidate-production theorem over local candidate envelope |
| `reject_or_fallback(n)` | `state[n]` is `Evaluating` or `PublishReady` | `state[n] = RejectedPendingRepair`; demand retained; dynamic overlays removed | W033 Lean reject/no-publish; retained failures | full per-node reject transition proof |
| `publish_and_clear(n)` | `state[n] = PublishReady` after coordinator publish | `state[n] = Clean`; demand removed; execution overlay eligible | recalc unit test; W033 Lean publish atomic | bridge proof from coordinator publish to tracker state cleanup |
| `DependencyGraph::build` | prepared descriptors over snapshot | forward/reverse edge maps, diagnostics, cycle groups | TreeCalc graph artifacts | graph-construction semantics in Lean/TLA |
| `derive_invalidation_closure` | dependency graph and seeds | impacted order plus per-node invalidation records | TreeCalc/scale artifacts; W035 classified no-under-invalidation | reachability/minimality theorem |
| `accept_and_publish` | accepted candidate exists and snapshot matches | published view values extend previous values by candidate updates; counter increments | W033 Lean publish atomic; coordinator unit test | full candidate-to-publication refinement proof |
| `reject_candidate_work` | candidate known/admitted | reject log appends, candidate slots cleared, publication untouched | W033 Lean reject/no-publish; TLA | full coordinator implementation refinement proof |

## Formalization Direction Review

The current formalization direction should be adjusted in the showcase and in future worksets:

1. Keep the useful W038-W045 evidence classifiers, but do not let them dominate the story.
2. Promote the catalog above into a first-class core-engine proof backlog.
3. Build a Lean model for the dependency graph and invalidation closure, not just promotion predicates.
4. Add TLA or Lean transition models for `Stage1RecalcTracker` and `TreeCalcCoordinator` that mirror the Rust pre/postconditions.
5. Use TraceCalc as the executable semantic reference for selected kernels, then prove or replay refinement against TreeCalc.
6. Treat scale runs as semantic regression signatures only when their closed-form checks and phase timings are bound into replay/conformance artifacts.
