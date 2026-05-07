# W046 Engine Semantic Catalog And Effect-Signature Plan

Status: `spec_drafted`

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
| `docs/spec/core-engine/w045-formalization/W045_CLOSURE_AUDIT_AND_RELEASE_GRADE_VERIFICATION_DECISION.md` | predecessor audit and residual lanes |

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

## 7. Current Status

- execution_state: `calc-gucd.1_redirect_and_semantic_catalog_in_progress`
- scope_completeness: `scope_partial`
- target_completeness: `target_partial`
- integration_completeness: `partial`
- open_lanes:
  - catalog promotion into durable `docs/spec` surfaces is pending
  - new-idea triage is drafted but not yet bound to Lean/TLA/TraceCalc artifacts
  - graph/reverse-edge/SCC bead has not started
  - invalidation/rebind bead has not started
  - recalc/evaluation/TraceCalc refinement beads have not started
  - release-readiness classification remains a supporting lane only
