# W046 Core Formalization Engine Semantic Proof Spine

Status: `planned_successor_scope_redirected`

Parent predecessor: `W045`

Parent epic: `calc-gucd`

## 1. Purpose

Continue after W045 by redirecting the core formalization effort from release-readiness classification toward the engine semantic proof spine.

W033 through W045 produced useful proof fragments, TLA models, replay roots, scale signatures, service envelopes, and promotion-readiness classifiers. The showcase review found that the next tranche must put more formal weight on the calculation engine's actual execution mechanics: dependency graph construction, reverse-edge consistency, connected component and cycle classification, invalidation closure, rebind requirements, recalc state transitions, evaluation order, working-value reads, OxFml runtime adaptation, dynamic references, rejection, publication, and TraceCalc refinement.

W046 is not a release-grade promotion by intent. Proof-service, release-grade, C5, operated-service, pack-governance, and promotion-readiness work remain in scope only as supporting evidence lanes after the semantic spine has first-class specs, model targets, replay bindings, or exact blockers.

W046 treats current implementation behavior, existing specs, TraceCalc, TreeCalc/CoreEngine evidence, Lean/TLA artifacts, and OxFml seam behavior as evidence surfaces that may correct or deepen the specs. It is not a test against a frozen initial spec.

## 2. Governing Shift

The governing shift is:

1. Start with engine semantics, not promotion classification.
2. Convert the showcase engine catalog into OxCalc-owned spec objects.
3. Build formal targets for the execution phases before claiming service or release readiness.
4. Use TraceCalc as the selected-kernel correctness oracle and TreeCalc/CoreEngine as production/optimized implementations to replay or refine against that oracle.
5. Keep closure artifacts as coverage accounting, not as a substitute for semantic models.

## 3. Primary Scope

In scope:

1. Promote the engine semantic state/transition catalog from the showcase review into `docs/spec`.
2. Specify the formal state model for formulas, prepared descriptors, dependency edges, reverse edges, cycle groups, invalidation seeds, rebind flags, recalc node state, candidate results, diagnostics, publication bundles, and retained trace evidence.
3. Add Lean/TLA model targets for `DependencyGraph::build`, forward/reverse edge converse, and SCC/cycle classification.
4. Add invalidation seed, closure, and rebind model targets with reachability and no-under-invalidation theorem statements.
5. Add recalc tracker transition preconditions and postconditions for dirty, needed, evaluating, verified-clean, publish-ready, rejected, and cycle-blocked states.
6. Add evaluation-order and working-value read-discipline targets: evaluated nodes read only stable published values or prior ordered computed values, and failures short-circuit to no-publish rejection.
7. Add a TraceCalc refinement kernel for selected formulas and model shapes, then bind TreeCalc/CoreEngine replay evidence against it.
8. Keep the OxCalc + OxFml seam in scope for evaluator-facing reference effects, dynamic references, formatting/publication boundaries consumed by OxCalc, and the narrow `LET`/`LAMBDA` carrier fragment.
9. Bind scale and performance evidence to semantic regression signatures, closed-form checks, and phase timings without treating timing as correctness proof.
10. Recast proof-service, release-grade, C5, operated-service, independent-evaluator, pack-governance, and promotion-readiness lanes as downstream evidence layers over the semantic spine.

Out of scope:

1. Editing OxFml, OxFunc, DNA OneCalc, or host repos directly from this repo.
2. General OxFunc kernel semantics outside the narrow `LET`/`LAMBDA` carrier seam.
3. Claiming broad OxFml, public migration, or downstream W073 uptake without direct current-surface evidence.
4. Treating W073 old aggregate/visualization threshold strings as fallback metadata.
5. Promoting release-grade, C5, pack-grade replay, Stage 2 production policy, operated service, independent evaluator, mismatch quarantine, scaling correctness, callable, registered-external, provider-publication, continuous scale assurance, or general OxFunc claims from proxy evidence.

## 4. Current OxFml Intake

W046 carries the current OxFml formatting and public-surface intake as constraints on the seam model:

1. W073 aggregate and visualization conditional-formatting metadata is `typed_rule`-only.
2. W072 bounded `thresholds` strings are intentionally ignored for `colorScale`, `dataBar`, `iconSet`, `top`, `bottom`, `aboveAverage`, and `belowAverage`.
3. DNA OneCalc typed-rule request construction remains required downstream uptake unless W046 obtains direct current evidence.
4. Ordinary OxFml downstream use should target `consumer::runtime`, `consumer::editor`, and `consumer::replay`; public `substrate::...` access is not an ordinary integration contract.
5. `format_delta` and `display_delta` remain distinct seam categories.
6. `LET`/`LAMBDA` remains the narrow OxCalc/OxFml/OxFunc carrier seam inside the OxCalc formalization scope; general OxFunc kernels remain outside this repo's scope.

## 5. Formalization Spine

W046 works through this proof spine before returning to promotion-readiness classification:

1. **State and transition catalog**
   - Define the semantic objects and transition names used by the engine.
   - Map each object to Rust code, TraceCalc behavior, Lean/TLA target, replay roots, and known blockers.
   - Required output: a spec catalog that names invariants, preconditions, postconditions, and observation boundaries.

2. **Dependency graph, reverse edges, and SCCs**
   - Model prepared descriptors, owner nodes, target nodes, forward edges, reverse edges, untargeted descriptors, diagnostics, and cycle groups.
   - Theorem targets: forward edge soundness, reverse edge converse, self-cycle detection, non-trivial SCC classification, and untargeted descriptor diagnostic preservation.

3. **Invalidation, soft references, and rebind**
   - Model invalidation seeds from structural edits, dependency transitions, dynamic references, upstream publications, and soft-reference updates.
   - Theorem targets: reverse-reachability closure soundness, no-under-invalidation, rebind flag soundness, and stale-binding no-publish rejection.

4. **Recalc tracker state machine**
   - Model dirty, needed, evaluating, verified-clean, publish-ready, rejected-pending-repair, and cycle-blocked states.
   - Theorem targets: allowed transition relation, demand clearing, no publish from rejected states, verified-clean no-publish, and publication only from accepted candidates.

5. **Evaluation order and working values**
   - Model topological formula order or cycle rejection.
   - Model read discipline for seeded published values plus prior computed updates.
   - Theorem targets: dependency-before-dependent evaluation, stable/prior read discipline, diagnostic short-circuit, and no torn candidate bundle.

6. **TraceCalc refinement kernel**
   - Select checkable formula/model fragments, including simple formulas, ranges, dynamic references, `INDIRECT`, and narrow `LET`/`LAMBDA` carrier cases where available.
   - Bind TraceCalc as the reference handler/oracle and TreeCalc/CoreEngine as optimized handlers.
   - Required output: replay artifacts that compare observable values, diagnostics, dependency effects, invalidation records, rejection, and publication decisions.

7. **Evidence and readiness layer**
   - Only after the semantic spine has specs, models, replay roots, or exact blockers, classify proof-service, operated-service, scale, Stage 2, pack/C5, independent-evaluator, OxFml-public, and release-readiness consequences.

## 6. New Ideas To Explore

1. **Algebraic effects for the OxCalc/OxFml semantic boundary**
   - Treat OxFml formula evaluation as a mostly pure computation that requests effects, and treat OxCalc phases as handlers for those effects.
   - Candidate effect signature:
     - `ReadValue(reference, context)`
     - `ResolveStatic(reference, context)`
     - `ResolveDynamic(owner, text, context)`
     - `EmitDependency(owner, target, descriptor_kind)`
     - `EmitDiagnostic(owner, diagnostic_kind)`
     - `CallFunction(function_id, args)`
     - `BindLocal(name, value)`
     - `EnterLambda(params, body, closure_env)`
     - `ProduceCandidate(owner, value, diagnostics, effects)`
     - `RejectCandidate(owner, reason)`
     - `Publish(bundle)`
   - Candidate handlers:
     - dependency-build handler records static edges, untargeted descriptors, dynamic potential, and diagnostics;
     - soft-reference update handler resolves dynamic references and records dependency-shape changes;
     - recalc handler reads stable/prior values and produces candidates or typed rejection;
     - publication handler atomically accepts validated bundles or preserves the published view on rejection;
     - TraceCalc handler acts as the selected-kernel reference oracle;
     - TreeCalc/CoreEngine handlers act as production/optimized implementations to replay/refine against the oracle.
   - Handler-law targets:
     - OxFml requests do not mutate the published model directly;
     - dynamic dependency changes must route through invalidation and rebind before publication;
     - reject handlers preserve published values;
     - publish handlers are atomic;
     - LET/LAMBDA carrier effects are inside OxCalc formalization scope while general OxFunc kernels remain outside this repo.

2. **Dataflow and abstract-interpretation lens**
   - Treat dependency/invalidation as monotone reachability and dataflow equations over graph state.
   - Use this to state closure, no-under-invalidation, and fixpoint/minimality targets more sharply.

3. **Type-and-effect rows**
   - Classify every formula/evaluator action by the effects it may request: static refs, dynamic refs, range reads, callable calls, diagnostics, formatting deltas, dependency effects, candidate production, rejection, and publication.
   - Use this to state phase legality: dependency build may collect references and diagnostics; soft-reference update may resolve dynamic targets; recalc may read values and produce candidates; OxFml may request effects but cannot publish.
   - Candidate target: a type/effect table for formula fragments and engine phases that prevents accidental cross-phase authority.

4. **Datalog-style dependency semantics**
   - Specify edges, reverse edges, invalidation closure, SCC membership, affected nodes, and rebind requirements as relations and rules.
   - Use this as a compact executable reference semantics for graph phases, separate from Rust implementation strategy.
   - Candidate target: Datalog-like rule tables that can be replayed against TreeCalc graph artifacts.

5. **Small-step operational semantics**
   - Define the engine as explicit transitions such as `BuildGraph`, `MarkDirty`, `ResolveSoftRefs`, `SelectOrder`, `EvaluateNode`, `AcceptCandidate`, `RejectCandidate`, and `Publish`.
   - Use this to attach preconditions, postconditions, invariants, and observable events to each calculation phase.
   - Candidate target: the transition system that Lean/TLA, TraceCalc, and replay artifacts all reference.

6. **Refinement and observational equivalence**
   - State each optimized path as a refinement of a smaller reference transition system.
   - Compare observable values, diagnostics, dependency effects, rejection, and publication rather than internal scheduling details.
   - Candidate target: TraceCalc and TreeCalc/CoreEngine as machines over the same observable event vocabulary.

7. **Incremental computation theory**
   - Treat edits as change sets and recalc as self-adjusting computation over a dependency graph.
   - Use this to reason about stability, affected-region soundness, recomputation bounds, and when an optimized evaluator may safely preserve cached results.
   - Candidate target: formal language for minimality and bounded recomputation claims.

8. **Proof-carrying traces**
   - Have important runs emit enough local evidence to check the result without trusting the whole execution: graph facts, invalidation facts, order facts, candidate facts, rejection facts, and publication facts.
   - Use this to separate expensive execution from cheaper independent validation.
   - Candidate target: a trace checker that validates emitted evidence against the semantic catalog.

9. **Provenance semirings and proof-relevant explanations**
   - Track why each value or diagnostic exists: source values, dependency paths, dynamic-resolution choices, function calls, invalidation reasons, and publication decisions.
   - This overlaps with proof-carrying traces, but the emphasis is different: proof-carrying traces check a run; provenance semirings give algebraic structure to explanations and change impact.
   - Candidate target: provenance annotations that can compose across formulas, ranges, dynamic references, and aggregate functions.

10. **Event-sourced engine state**
    - Treat core state as derived from an append-only event log: formula edits, value edits, reference resolutions, invalidations, recalc transitions, candidate decisions, publications, and rejections.
    - Use this to align formal replay, undo/redo, collaboration, retained evidence, and auditability.
    - Candidate target: a canonical event vocabulary that can reconstruct semantic state and compare replay runs.

11. **Graph rewriting**
    - Model dependency changes, dynamic-reference rebinding, SCC changes, and optimizer graph transformations as graph rewrite rules.
    - This is especially relevant once enormous models are compiled to faster evaluators: optimizer rewrites must preserve observable semantics while changing graph shape.
    - Candidate target: local rewrite rules with preservation obligations for reachability, SCC classification, invalidation, and evaluation order.

12. **Spec-derived replay generation**
   - Generate deterministic TraceCalc/TreeCalc/CoreEngine replay cases from the formal catalog, especially for graph transitions, `INDIRECT`, soft references, LET/LAMBDA carriers, and invalidation closure.

13. **Categorical/string-diagram semantics for compiled evaluators**
    - Treat formulas, dependency edges, cached values, dynamic-resolution points, and publication fences as compositional wiring diagrams.
    - Use traced monoidal/category-style language to reason about feedback, cycles, batching, fusion, and compilation from a high-level dependency graph into a fast evaluator.
    - Candidate target: optimizer laws that state when two compiled evaluator graphs are equivalent to the same semantic wiring diagram.

14. **Concurrent separation logic and rely-guarantee reasoning**
    - Treat concurrent evaluation as workers owning disjoint or protected fragments of candidate state while sharing published snapshots and publication fences.
    - Use rely-guarantee or separation-logic style reasoning to prove no torn publication, no stale read publication, safe parallel candidate production, and valid merge/commit behavior.
    - Candidate target: concurrency invariants for a future parallel evaluator and enormous-model execution profile.

15. **E-graphs and equality-saturation optimizer proofs**
    - Treat formula simplification, range fusion, common-subexpression sharing, and compiled evaluator optimization as equality-preserving rewrites recorded in an e-graph.
    - Use extracted rewrite certificates or replayable rewrite traces as evidence that an optimized expression graph preserves the formal semantics.
    - Candidate target: a future optimizer proof lane where performance rewrites emit semantic preservation evidence.

## 7. Exit Gate

W046 exits only when:

1. The engine semantic state/transition catalog is promoted into `docs/spec` and mapped to implementation code, formal artifacts, replay artifacts, and exact blockers.
2. Graph/reverse-edge/SCC, invalidation/rebind, recalc-state, evaluation-order, working-value, and TraceCalc-refinement lanes emit Lean/TLA targets, checked artifacts, replay evidence, or exact blockers.
3. The new-ideas section is triaged, with algebraic effects either incorporated into the spec model as an effect-signature/handler-law layer or explicitly deferred with a reason.
4. Current OxFml inbound observations are reviewed, including W073 typed-rule direct replacement and public consumer surface updates.
5. Proof-service, release-grade, C5, pack-grade replay, Stage 2, operated-service, independent-evaluator, OxFml/callable, release-scale, and promotion-readiness decisions are classified only as consequences of direct semantic evidence or exact blockers.
6. Closure audit includes prompt-to-artifact checklist, OPERATIONS checklist, completion-claim self-audit, semantic-equivalence statement, direct-evidence coverage audit, reviewed inbound observations line, and three-axis report.

## 8. Planned Bead Path

1. `calc-gucd.1` - W046 redirect, showcase finding uptake, engine semantic catalog, and effect-signature plan.
2. `calc-gucd.2` - dependency graph build, reverse-edge converse, and SCC/cycle formal model.
3. `calc-gucd.3` - invalidation seed, closure, soft-reference, dynamic reference, and rebind formal model.
4. `calc-gucd.4` - recalc tracker transition preconditions and postconditions.
5. `calc-gucd.5` - evaluation-order and working-value read-discipline model.
6. `calc-gucd.6` - TraceCalc refinement kernel and TreeCalc/CoreEngine replay binding.
7. `calc-gucd.7` - OxFml seam, `LET`/`LAMBDA` carrier, formatting/publication, and callable-boundary model.
8. `calc-gucd.8` - proof-service and evidence-classifier coverage ledger recast over the semantic spine.
9. `calc-gucd.9` - scale/performance semantic-regression signatures and phase-timing evidence binding.
10. `calc-gucd.10` - Stage 2, pack-governance, C5, operated-service, independent-evaluator, and release-readiness consequence reassessment.
11. `calc-gucd.11` - closure audit, semantic-spine coverage decision, and successor routing.

## 9. Evidence Policy

The W046 canonical artifact root is `docs/spec/core-engine/w046-formalization/` for packet docs and `docs/test-runs/core-engine/` for emitted evidence roots. Artifact roots remain checked in when they are normative baseline evidence for a bead.

Validation requirements scale by bead:

1. Spec/model beads require deterministic artifact indexes, proof/model commands where available, and explicit assumption/blocker registers.
2. Code-bearing beads require focused tests plus relevant workspace checks.
3. Runner/evidence beads require deterministic artifact emission, JSON validation, and non-mutation of prior baseline artifacts.
4. Closure-audit beads require focused retests of the highest-risk seam and pack lanes, workset/bead validation, cycle checks, JSON validation, and diff hygiene.

## 10. Current Status

- execution_state: `calc-gucd.2_dependency_graph_model_ready`
- scope_completeness: `scope_partial`
- target_completeness: `target_partial`
- integration_completeness: `partial`
- open_lanes:
  - `calc-gucd.2` graph/reverse-edge/SCC formal model is ready to start
  - invalidation/soft-reference/rebind formal model remains open
  - recalc tracker and evaluation-order formal models remain open
  - TraceCalc refinement kernel and TreeCalc/CoreEngine replay binding remain open
  - OxFml seam, `LET`/`LAMBDA` carrier, and formatting/publication model remains open
  - release-grade verification, pack-grade replay, C5, Stage 2 production policy, operated services, independent evaluator breadth, broad OxFml/public migration, W073 downstream uptake, continuous scale assurance, and general OxFunc kernels remain unpromoted or external as classified
