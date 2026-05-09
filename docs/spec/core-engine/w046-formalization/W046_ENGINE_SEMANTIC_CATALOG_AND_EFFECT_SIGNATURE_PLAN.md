# W046 Engine Semantic Catalog And Effect-Signature Plan

Status: `bead_closed`

Workset: `W046`

Parent epic: `calc-gucd`

Bead: `calc-gucd.1`

## 1. Purpose

This packet redirects W046 from residual release-readiness classification toward the engine semantic proof spine.

The immediate target is not release-grade promotion. The target is to make the next W046 tranche exact before graph, invalidation, recalc, evaluation, and TraceCalc-refinement beads start.

## 2. Source Surfaces

| Surface | Role |
| --- | --- |
| `docs/showcase/oxcalc_w033_w045_engine_formalization_review_catalog.md` | engine catalog and showcase findings |
| `docs/worksets/W046_CORE_FORMALIZATION_ENGINE_SEMANTIC_PROOF_SPINE.md` | governing redirected workset |
| `docs/spec/core-engine/CORE_ENGINE_FORMALIZATION_AND_ASSURANCE.md` | existing assurance/formalization doctrine |
| `docs/spec/core-engine/CORE_ENGINE_TRACECALC_REFERENCE_MACHINE.md` | TraceCalc oracle/reference-machine scope |
| `docs/spec/core-engine/CORE_ENGINE_OXFML_SEAM.md` | OxCalc/OxFml seam scope |
| `archive/w045-formalization/W045_CLOSURE_AUDIT_AND_RELEASE_GRADE_VERIFICATION_DECISION.md` | predecessor audit and residual lanes |
| `W046_SEMANTIC_FRAGMENT_REVIEW_LEDGER.md` | first-pass semantic fragment review and successor-bead mapping |
| `W046_ENGINE_STATE_TRANSITION_CATALOG.md` | state vocabulary, transition catalog, invariant targets, and successor-bead routing |

## 3. Catalog Promotion Target

`calc-gucd.1` should promote the following semantic families into first-class spec objects before the rest of W046 proceeds:

1. `STATE` - snapshot, formula, descriptor, dependency graph, invalidation, recalc, candidate, publication, and evidence state.
2. `FML` - formula preparation, OxFml runtime adaptation, and narrow `LET`/`LAMBDA` carrier semantics.
3. `REF` - static, soft, dynamic, and unresolved reference descriptors.
4. `DEP` - dependency graph build, forward edges, reverse edges, diagnostics, and cycle groups.
5. `SCC` - SCC/cycle classification and self-cycle handling.
6. `INV` - invalidation seeds, closure, dynamic dependency transitions, soft-reference update, and rebind flags.
7. `ORDER` - topological evaluation order or cycle rejection.
8. `REC` - recalc tracker state transitions and node-state pre/post conditions.
9. `EVAL` - working-value reads, candidate production, diagnostics, and failure short-circuit.
10. `REJ` - typed rejection and no-publish semantics.
11. `PUB` - atomic publication and candidate/publication separation.
12. `ASSURE` - TraceCalc refinement, replay binding, independent evaluator evidence, and scale semantic signatures.

Each catalog row should name:

1. implementation code surface,
2. formal target,
3. known proof/model artifact,
4. replay or evidence root,
5. current blocker or assumption,
6. next W046 owner bead.

## 4. Algebraic-Effects Plan

W046 should evaluate algebraic effects as a specification architecture, not as a Rust runtime rewrite.

The proposed lens is:

1. OxFml formula evaluation is a mostly pure expression semantics that requests effects.
2. OxCalc phases are handlers for those effects under snapshots, graph state, invalidation state, and publication policy.
3. TraceCalc is the selected-kernel reference handler.
4. TreeCalc/CoreEngine are production/optimized handlers that should replay or refine against TraceCalc for selected fragments.

Candidate effect signature:

| Effect | Intended semantic use |
| --- | --- |
| `ReadValue(reference, context)` | read stable published value or prior ordered computed value |
| `ResolveStatic(reference, context)` | lower static formula reference into target descriptor |
| `ResolveDynamic(owner, text, context)` | resolve `INDIRECT` or soft/dynamic reference text |
| `EmitDependency(owner, target, descriptor_kind)` | record graph edge or dynamic dependency effect |
| `EmitDiagnostic(owner, diagnostic_kind)` | record unresolved, invalid, cycle, or evaluator diagnostic |
| `CallFunction(function_id, args)` | call OxFml/OxFunc-visible function boundary |
| `BindLocal(name, value)` | represent LET binding |
| `EnterLambda(params, body, closure_env)` | represent LAMBDA carrier environment |
| `ProduceCandidate(owner, value, diagnostics, effects)` | return candidate result before publication |
| `RejectCandidate(owner, reason)` | emit no-publish rejection |
| `Publish(bundle)` | atomically update published state |

Candidate handlers:

1. dependency-build handler,
2. soft-reference update handler,
3. recalc handler,
4. publication handler,
5. TraceCalc reference handler,
6. TreeCalc/CoreEngine optimized handlers.

Handler-law targets:

1. OxFml requests do not mutate the published model directly.
2. Dynamic dependency changes route through invalidation and rebind before publication.
3. Rejection preserves the published view.
4. Publication is atomic and comes only from accepted candidates.
5. LET/LAMBDA carrier effects are inside OxCalc formalization scope.
6. General OxFunc kernels remain outside this repo's formalization scope.

### 4.1 Effect-To-Transition Binding

This binding converts the algebraic-effects idea into W046 successor-bead work. The effect names are specification handles, not Rust API commitments.

| Effect | Primary transition binding | Handler authority | Owner bead | First invariant or law |
| --- | --- | --- | --- | --- |
| `ReadValue(reference, context)` | `T12.ReadWorkingValue`, `T13.EvaluateFormula` | recalc/evaluation handler may read seeded published values or prior ordered computed values | `calc-gucd.5`, `calc-gucd.7` | `INV.EVAL.STABLE_PRIOR_READS` |
| `ResolveStatic(reference, context)` | `T01.PrepareFormula`, `T02.LowerDescriptors` | preparation/dependency handler may lower references to descriptors | `calc-gucd.2`, `calc-gucd.7` | descriptor soundness and diagnostic preservation |
| `ResolveDynamic(owner, text, context)` | `T06.SeedInvalidation`, `T10.RebindGate`, `T13.EvaluateFormula` | dynamic/soft-reference handler may resolve or reject dynamic targets, but publication waits for rebind discipline | `calc-gucd.3`, `calc-gucd.7` | `INV.INV.REBIND_NO_PUBLISH` |
| `EmitDependency(owner, target, descriptor_kind)` | `T02.LowerDescriptors`, `T03.BuildGraph`, `T04.BuildReverseEdges` | graph handler records edge facts and reverse facts | `calc-gucd.2` | `INV.GPH.CONVERSE` |
| `EmitDiagnostic(owner, diagnostic_kind)` | `T01.PrepareFormula`, `T03.BuildGraph`, `T13.EvaluateFormula`, `T16.RejectCandidate` | diagnostics are preserved as evidence or rejection detail; they are not publication | `calc-gucd.2`, `calc-gucd.6`, `calc-gucd.7` | `INV.GPH.DIAGNOSTIC_PRESERVATION`; reject-is-no-publish |
| `CallFunction(function_id, args)` | `T13.EvaluateFormula` | formula handler can call through OxFml/OxFunc boundary under current seam assumptions | `calc-gucd.7` | general OxFunc kernels opaque; LET/LAMBDA carrier fragment visible |
| `BindLocal(name, value)` | `T13.EvaluateFormula` | formula handler records LET local binding semantics only where carrier facts are engine-visible | `calc-gucd.7` | LET carrier effects stay inside narrow seam |
| `EnterLambda(params, body, closure_env)` | `T13.EvaluateFormula` | formula handler records lambda carrier identity and closure visibility where consumed by OxCalc | `calc-gucd.7` | LAMBDA carrier fact visibility without broad OxFunc ownership |
| `ProduceCandidate(owner, value, diagnostics, effects)` | `T15.ProduceCandidate` | candidate handler creates candidate state separate from publication | `calc-gucd.4`, `calc-gucd.6` | `INV.CAND.NOT_PUBLICATION` |
| `RejectCandidate(owner, reason)` | `T10.RebindGate`, `T16.RejectCandidate` | rejection handler clears candidate/recalc work and preserves published view | `calc-gucd.3`, `calc-gucd.4`, `calc-gucd.6`, `calc-gucd.7` | `INV.REJ.NO_PUBLISH` |
| `Publish(bundle)` | `T17.PublishCandidate` | coordinator publication handler is the single publisher | `calc-gucd.4`, `calc-gucd.6` | `INV.PUB.ATOMIC` |

### 4.2 Handler Laws Carried Forward

| Handler law | Transition surface | Successor proof target |
| --- | --- | --- |
| evaluator purity boundary | `T01`, `T12`, `T13` | OxFml/OxFunc-visible evaluation requests effects; OxCalc handlers decide graph/recalc/publication consequences |
| graph handler authority | `T02`, `T03`, `T04`, `T05` | graph build owns edge, reverse-edge, diagnostic, and SCC facts |
| rebind-before-publish | `T06`, `T07`, `T10`, `T16` | dynamic dependency shape changes route through invalidation/rebind or reject |
| candidate/publication separation | `T15`, `T17` | candidate state is never stable publication |
| rejection preserves publication | `T10`, `T16` | reject decisions keep previous published view |
| single atomic publisher | `T17` | publication happens only through coordinator bundle commit |
| TraceCalc reference handler | `T18` plus selected prior transitions | TraceCalc and TreeCalc/CoreEngine compare on observable events, not internal scheduling shape |

## 5. New-Idea Intake

The governing W046 workset records these idea lenses for triage during `calc-gucd.1`:

1. algebraic effects for OxCalc/OxFml semantic requests and handlers,
2. dataflow and abstract interpretation for invalidation and closure,
3. type-and-effect rows for formula fragments and engine phase authority,
4. Datalog-style dependency semantics for executable graph rules,
5. small-step operational semantics for phase transitions,
6. refinement and observational equivalence for TraceCalc and TreeCalc/CoreEngine,
7. incremental computation theory for stability and recomputation bounds,
8. proof-carrying traces for independently checkable run evidence,
9. provenance semirings for algebraic explanation and change-impact structure,
10. event-sourced engine state for replay, undo/redo, collaboration, and auditability,
11. graph rewriting for dependency rebinding and future optimizer preservation,
12. spec-derived replay generation from the semantic catalog,
13. categorical/string-diagram semantics for compiled evaluators,
14. concurrent separation logic and rely-guarantee reasoning for future parallel execution,
15. e-graphs and equality-saturation proof traces for future optimizer rewrites.

The immediate W046 consequence is not to adopt every framework. The immediate consequence is to mine these lenses for clearer semantic objects, invariants, theorem targets, replay shapes, optimizer-preservation obligations, and concurrency boundaries.

## 6. Redirect Consequence

The previous W046 residual-promotion ledger shape is superseded by this semantic-spine plan.

Promotion-readiness gates are still relevant, but they are downstream consequences. They should not drive the first bead path until the graph, invalidation, recalc, evaluation, OxFml-seam, and TraceCalc-refinement semantics have specs, model targets, replay evidence, or exact blockers.

## 7. Successor-Bead Gate Table

`calc-gucd.1` hands off the following exact start conditions to successor beads:

| Bead | Lane | Exact inputs | First model artifact | First theorem/invariant target | First evidence seed | Stop condition |
| --- | --- | --- | --- | --- | --- | --- |
| `calc-gucd.2` | dependency graph, reverse-edge converse, SCC model | `dependency.rs`, `planner.rs`, `W046_ENGINE_STATE_TRANSITION_CATALOG.md`, TreeCalc graph JSON, `tc_cycle_region_reject_001` | graph relation over descriptors, edges, reverse edges, diagnostics, SCC groups | `INV.GPH.CONVERSE`, `INV.GPH.DIAGNOSTIC_PRESERVATION`, `INV.SCC.CYCLE_CLASSIFICATION` | existing cycle scenario or new small graph fixture | graph model/proof target exists or exact blocker recorded |
| `calc-gucd.3` | invalidation, soft/dynamic refs, rebind | `derive_invalidation_closure`, dynamic/rebind TreeCalc post-edit artifacts, W034 dependency Lean fragments | reachability/rebind transition model over graph facts | `INV.INV.NO_UNDER_INVALIDATION`, `INV.INV.REBIND_NO_PUBLISH` | W035 dirty-seed closure and rebind/dynamic post-edit artifacts | closure/rebind model target exists or exact blocker recorded |
| `calc-gucd.4` | recalc tracker transition pre/post | `Stage1RecalcTracker`, `TreeCalcCoordinator`, `CoreEngineStage1.tla`, `Stage1State.lean` | transition crosswalk from Rust methods to state-machine actions | `INV.REC.LEGAL_STATES`, `INV.CAND.NOT_PUBLICATION`, `INV.REJ.NO_PUBLISH`, `INV.PUB.ATOMIC` | TreeCalc node-state artifacts and publish/reject fixtures | transition table plus proof/model targets exist |
| `calc-gucd.5` | evaluation order and working-value reads | `topological_formula_order`, `working_values` loop, TreeCalc evaluation-order artifacts | checked topo/read-discipline model | `INV.ORDER.BEFORE_DEPENDENT`, `INV.EVAL.STABLE_PRIOR_READS` | multi-node DAG, verified-clean, dynamic, and simple formula artifacts plus W046 TLC evidence | checked read-discipline invariant target recorded |
| `calc-gucd.6` | TraceCalc refinement kernel and replay binding | TraceCalc machine/planner/runner, oracle matrix, TreeCalc local artifacts, `T18.EmitTraceAndEvidence` | observable event vocabulary and refinement relation | `INV.TRC.OBS_EQUIV` over values, diagnostics, dependencies, invalidation, rejection, publication, traces | W033-W037 TraceCalc/TreeCalc conformance artifacts plus W046 binding root | checked selected-kernel relation and replay binding register exist; exact blockers recorded |
| `calc-gucd.7` | OxFml seam and LET/LAMBDA carrier | OxFml seam docs, upstream-host fixtures, LET/LAMBDA witnesses, callable-boundary Lean, effect table above | effect signature and handler-law model | evaluator purity boundary, LET/LAMBDA carrier visibility, no direct publication from OxFml | W037 upstream-host direct evaluator artifacts and LET/LAMBDA cases | effect/handler-law spec exists or exact OxFml blocker recorded |
| `calc-gucd.8` | evidence layer over semantic spine | outputs from `.2` through `.7`; archived classifiers only as predecessor context | proof-service/evidence coverage ledger over semantic objects | no classifier row promotes beyond direct semantic evidence | current model/proof/replay outputs | classifier/evidence layer recast over semantic spine |
| `calc-gucd.9` | scale and phase timings as semantic regression evidence | treecalc-scale profiles, phase timing names, graph/rebind/recalc invariants | semantic regression signature table | timing never substitutes for correctness; phase timings bind to semantic phases | million-node grid/fanout/indirect/rebind artifacts | performance evidence mapped to semantic signatures |
| `calc-gucd.10` | downstream consequence reassessment | outputs from `.2` through `.9` | consequence matrix for Stage 2, pack, C5, operated service, release-readiness | promotion only from direct semantic evidence or exact blockers | current semantic-spine outputs | readiness consequences classified honestly |
| `calc-gucd.11` | W046 closure audit and successor routing | outputs from all W046 beads | semantic-spine coverage audit | coverage decision names exact open semantics | W046 evidence index | successor workset routing exists |

## 8. First Review Finding

The first semantic-fragment review is recorded in `W046_SEMANTIC_FRAGMENT_REVIEW_LEDGER.md`.

The review starts from actual engine fragments rather than closure taxonomy:

1. dependency graph build, reverse-edge converse, SCC classification, invalidation closure, rebind, topological order, and working-value reads are the highest-priority semantic gaps;
2. candidate/publication separation, reject-is-no-publish, atomic publication, pinned readers, and overlay retention already have stronger TLA/Lean scaffolding and can be reused as proof backbone;
3. TraceCalc should be formalized as a selected-kernel observable oracle before additional comparison rows are treated as semantic evidence;
4. the OxFml seam should be recast as an effect-signature and handler-law boundary, with `LET`/`LAMBDA` kept as a narrow carrier fragment.

The first transition catalog is recorded in `W046_ENGINE_STATE_TRANSITION_CATALOG.md`. It names the concrete phase transitions from formula preparation through graph build, invalidation, recalc scheduling, evaluation, rejection, publication, and trace emission. It also gives `calc-gucd.2` an exact start condition: model `BuildGraph`, `BuildReverseEdges`, and `ClassifySCC` against graph converse, diagnostic-preservation, and cycle-classification invariants.

## 9. Closure Audit For `calc-gucd.1`

`calc-gucd.1` is a planning and routing bead. It introduces no runtime, coordinator, evaluator, publication, FEC/F3E, OxFml, or OxFunc behavior change.

Semantic-equivalence statement:

Existing observable OxCalc behavior is invariant under this bead. The bead changes W046 planning/specification documents only. It does not alter dependency graph construction, invalidation, recalc tracker behavior, evaluation order, working-value reads, TraceCalc execution, TreeCalc/CoreEngine execution, OxFml evaluation, rejection, or publication.

Pre-Closure Verification Checklist:

| # | Check | Result |
|---|-------|--------|
| 1 | Spec text and realization notes updated for all in-scope items? | yes; semantic catalog, fragment ledger, transition catalog, effect binding, and successor gate table exist |
| 2 | Pack expectations updated for affected packs? | yes; no pack expectation changed by this planning bead |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes; no new behavior introduced, and successor gates name existing deterministic evidence seeds where behavior is referenced |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes; no policy/strategy change, invariant statement above |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes; no normative seam change or handoff needed |
| 6 | All required tests pass? | yes; workset/bead validation and dependency-cycle validation are required for this doc bead |
| 7 | No known semantic gaps remain in declared scope? | yes for the planning/routing scope; actual semantic proof gaps are routed to successor beads |
| 8 | Completion language audit passed? | yes; this packet does not claim successor models or proofs are finished |
| 9 | `WORKSET_REGISTER.md` updated when ordered workset truth changed? | yes; no ordered workset truth change |
| 10 | `IN_PROGRESS_FEATURE_WORKLIST.md` updated when feature-map truth changed? | yes; no feature-map truth change required beyond current W046 surfaces |
| 11 | execution-state blocker surface updated? | yes; `.beads/` owns `calc-gucd.1` state |

Completion Claim Self-Audit:

| Step | Result |
| --- | --- |
| Scope re-read | pass; the bead asked for W046 redirect, semantic object/transition identification, and algebraic-effects signature/handler-law plan |
| Gate criteria re-read | pass; successor beads now have exact owners, inputs, model targets, invariant targets, and evidence seeds |
| Silent scope reduction check | pass; graph/SCC, invalidation, recalc, evaluation, TraceCalc, OxFml, and evidence-layer work are explicitly successor-scoped, not hidden as finished here |
| "Looks done but is not" pattern check | pass; no scaffolding, compile-only code, or spec text is presented as a mechanized proof |
| Include result | pass; checklist, self-audit, semantic-equivalence statement, and three-axis report are included here |

## 10. Current Status

- execution_state: `calc-gucd.1_closed`
- scope_completeness: `scope_complete`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
