# CORE_ENGINE_FORMALIZATION_AND_ASSURANCE.md

## 1. Purpose and Status
This document defines the formalization and assurance framework for the rewritten OxCalc core-engine spec set.

Status:
1. active rewrite baseline,
2. intended canonical assurance companion,
3. near-formal in scope rather than proof-complete,
4. TreeCalc-first in immediate realization target.

This document defines:
1. what parts of the core engine are intended to be formally modeled,
2. how Lean, TLA+, replay artifacts, and packs relate to the architecture,
3. the initial theorem and model-check backlog,
4. evidence obligations for promoted claims.

## 2. Assurance Mission
OxCalc is not intended to be specified only informally.

The core-engine assurance mission is to maintain a near-formal model that couples:
1. architectural semantics,
2. explicit state and transition models,
3. proof-oriented targets where appropriate,
4. model-checked concurrency and async properties where appropriate,
5. deterministic replay and empirical evidence where proof alone is not enough.

This is not optional polish.
It is part of the core-engine design itself.

### 2.1 Highest-Level Goal

The formalization effort exists to make the core calculation engine trustworthy under change.

The practical purpose is to prevent semantic drift when the engine widens from simple sequential execution toward richer TreeCalc behavior, replay promotion, optimization, scaling, and later concurrency. Formalization should make the dangerous cases explicit and checkable: stale publication, torn publication, missed dependency invalidation, unsafe overlay reuse, reject paths that publish by accident, and scheduling or performance strategies that change stabilized observable results.

For the OxCalc + OxFml boundary, this means formalizing the engine/seam contract rather than proving every function. OxFml produces typed evaluator facts such as candidate results, commit bundles, rejects, fences, traces, and runtime-derived effects. OxCalc consumes those facts to maintain dependency, invalidation, overlay, coordinator, and publication state. OxFunc semantic kernels remain outside this OxCalc assurance surface except as opaque packet assumptions already surfaced through OxFml.

`LET` and `LAMBDA` are the named exception at the boundary-carrier level. Their local binding, lambda value, call identity, prepared-call shape, and dependency/runtime-effect visibility should be captured because the fragment threads through OxFml, OxFunc, and OxCalc-visible engine behavior. This does not turn the assurance surface into a general OxFunc semantics project.

The intended assurance result is a coupled stack where prose specs, Lean/TLA+ models, deterministic replay witnesses, tests, pack mappings, and capability claims all describe the same behavior at their appropriate precision.

Within that stack, TraceCalc has a specific authority role. Both the TraceCalc reference-machine surface and the production/core-engine surface may receive formal and replay checking, but TraceCalc stands as the executable correctness oracle for the behavior it covers. Production and optimized implementations must then show conformance to TraceCalc for covered observable semantics.

## 3. Assurance Surfaces
The rewritten core-engine spec set uses four assurance surfaces.

### 3.1 Lean-Oriented Semantic Surface
Used for:
1. state-kernel semantics,
2. structural and replay invariants,
3. transition properties that are well suited to proof-oriented treatment.

### 3.2 TLA+ Concurrency Surface
Used for:
1. coordinator publication safety,
2. fence behavior,
3. pinned-reader and publication interaction,
4. staged concurrent and async execution safety and progress questions.

### 3.3 Replay and Trace Surface
Used for:
1. deterministic operational evidence,
2. cross-engine differential reasoning where needed,
3. minimized failure and regression artifacts,
4. evidence for seam and publication behavior,
5. TraceCalc reference-machine oracle checks and production/core-engine conformance comparisons for covered behavior.

### 3.4 Pack and Empirical Surface
Used for:
1. executable conformance obligations,
2. staged promotion gates,
3. performance and economics measurements,
4. evidence where theorem proving is not the right first instrument.

## 4. Assurance Mapping Rule
Every significant architectural claim in OxCalc should route to at least one assurance form:
1. proof target,
2. model-check target,
3. replay artifact requirement,
4. pack requirement,
5. explicit deferred item with rationale.

Architecture text that cannot be mapped this way should be treated as provisional until clarified.

## 5. Lean-Oriented Model Boundaries
The initial Lean-facing model should cover the structural and sequentially meaningful core before it attempts full concurrency behavior.

### 5.1 Intended Lean Model Areas
1. structural snapshots,
2. stable identity and projection boundary,
3. structural successor relation under operations,
4. explicit truth/derived/runtime separation,
5. recalc-state transition skeleton,
6. accept/reject publication invariants at the abstract level.

### 5.2 Lean Model Exclusions In Early Passes
The earliest Lean passes do not need to encode every later-stage optimization detail or full concurrency scheduler behavior.

Those areas may instead first appear as:
1. abstract obligations,
2. replay-based obligations,
3. TLA+ obligations,
4. staged-later proof targets.

## 6. Initial Lean Theorem Backlog
The rewritten architecture expects an initial theorem backlog that includes at least:

### 6.1 No Hidden Structural Mutation
If two stable readers observe the same structural snapshot identity, runtime work alone cannot mutate the structural truth observed by either reader.

### 6.2 Deterministic Structural Successor Relation
Given the same admissible structural operation and the same base structural snapshot, the resulting successor structural snapshot is equivalent in the architecture-defined sense.

### 6.3 Replay Determinism for Admissible Sequential Histories
For admissible sequential histories under the declared profile and mode, replay yields equivalent structural and accepted publication outcomes.

### 6.4 Reject-Is-No-Publish Abstract Invariant
Rejected candidate work does not alter accepted published state.

### 6.5 Accepted Publication Atomicity Abstract Invariant
Accepted publication appears as one coherent state transition rather than partial observer-visible mutation.

### 6.6 Dynamic-Dependency Soundness Target
Where runtime-observed dependency behavior is modeled, the resulting accepted behavior must remain consistent with the architecture's from-scratch or equivalent correctness target.

This may initially be partially evidence-backed before stronger proof closure is achieved.

## 7. TLA+ Model Boundaries
The initial TLA+ surface should focus on coordinator and publication behavior.

### 7.1 Intended TLA+ Model Areas
1. coordinator accept/reject transitions,
2. snapshot and compatibility fences,
3. pinned-reader safety,
4. overlay retention/eviction safety relative to active readers and sessions,
5. staged concurrent and async publication interaction,
6. contention and retry behavior at the abstract protocol level.

### 7.2 Why TLA+ Is Central Here
These are the areas where:
1. interleavings matter,
2. local reasoning is often insufficient,
3. implementation shortcuts create subtle races,
4. the project explicitly wants stronger concurrency assurance.

## 8. Initial TLA+ Property Backlog
The rewritten architecture expects a first backlog including at least:

### 8.1 No Torn Publication Safety
Observers never see partially published accepted work.

### 8.2 Reject-Is-No-Publish Safety
Rejected work never advances stable accepted publication state.

### 8.3 Pinned-Reader Stability
A pinned reader continues to observe a stable view compatible with its pinned state while later work proceeds.

### 8.4 Overlay Eviction Safety
No overlay state required for a pinned reader or otherwise protected stable view is evicted prematurely.

### 8.5 Fence-Safe Accept/Reject Behavior
Candidate work is accepted only when the coordinator's compatibility rules hold.

### 8.6 Staged Concurrency Progress Questions
For later concurrent stages, the model should explore bounded progress or liveness questions appropriate to the declared policy, without weakening the safety priorities above.

## 9. Replay and Trace Obligations
Replay is a first-class assurance layer.

The rewritten architecture expects replay artifacts to support at least:
1. deterministic accept/reject outcomes,
2. stable observer-visible publication reasoning,
3. dynamic-dependency and overlay diagnostics where claimed,
4. cycle and iteration diagnostics where claimed,
5. staged concurrency contention evidence where applicable.

Replay is not only a debugging aid.
It is part of the conformance and promotion story.

## 9.1 Replay Adapter Capability Evidence Ladder
The Replay appliance rollout adds an explicit adapter-capability evidence ladder to the assurance story.

The OxCalc adapter must treat the Foundation capability levels as cumulative conformance targets:
1. `cap.C0.ingest_valid`
2. `cap.C1.replay_valid`
3. `cap.C2.diff_valid`
4. `cap.C3.explain_valid`
5. `cap.C4.distill_valid`
6. `cap.C5.pack_valid`

OxCalc may document the path to later levels, but it may only claim the highest level currently proven by local conformance artifacts.

Current local proof floor:
1. checked-in replay-appliance-aware baselines and validator artifacts now support `cap.C2.diff_valid`,
2. checked-in retained-failure explain artifacts now support `cap.C3.explain_valid`,
3. checked-in reduced-witness retained-failure artifacts now support `cap.C4.distill_valid`,
4. `cap.C5.pack_valid` remains a later lane.

## 9.2 OxCalc Conformance Targets `C0..C4`

### `C0`
OxCalc can ingest `TraceCalc` source scenarios and emitted run artifacts into a bundle-valid projection and surface projection gaps explicitly.

### `C1`
OxCalc can replay supported `TraceCalc` scenarios deterministically and preserve candidate/publication, reject, and pinned-view semantics.

### `C2`
OxCalc can emit typed mismatch classes and severity-class mappings over oracle-versus-engine comparison surfaces.

### `C3`
OxCalc can emit explain records over `why_changed`, `why_rejected`, and `why_diff` queries using bundle-valid refs alone.

### `C4`
OxCalc can distill reduced witnesses using declared reduction units, closure rules, and explicit preservation predicates, while preserving replay validity or marking outputs explanatory-only.

## 10. Pack Mapping Direction
The rewritten architecture should map directly to pack obligations.

At minimum, the architecture depends on or implies:
1. commit atomicity and reject replay packs,
2. overlay lifecycle and GC safety packs,
3. cycle semantics packs,
4. dynamic dependency semantics packs,
5. staged concurrency and epoch/fence packs,
6. visibility or scheduling-equivalence packs where such policies are enabled,
7. performance/economics packs for optimization-lane promotion.

## 10.1 Lifecycle and Quarantine Effects On Assurance Claims
Lifecycle and quarantine state now directly affect assurance claims.

Rules:
1. explanatory-only or quarantined witnesses are not pack-eligible,
2. a capability claim may not exceed the highest level backed by current conformance artifacts,
3. a retained failure may support local triage without supporting pack promotion,
4. pack-facing assurance claims must reference witness lifecycle and capability state as well as replay artifacts.

## 11. Empirical and Economics Obligations
Not every architectural decision can be fully settled by proof before implementation.

Where economics and crossover questions matter, the architecture requires explicit experiments.
Examples include:
1. early-cutoff benefit rates,
2. dynamic-topo versus rebuild crossover,
3. dynamic-dependency tracking cost versus conservative fallback,
4. overlay reuse and fallback economics,
5. staged concurrency replay and throughput signatures.

These experiments are assurance inputs, not mere performance vanity measurements.

## 12. Theory-To-Assurance Discipline
High-level theory from research is valuable only when it maps into the assurance stack.

The rewrite therefore requires that promoted theory be translated into one of:
1. a theorem target,
2. a TLA+ property,
3. a replay artifact requirement,
4. a pack obligation,
5. an explicit deferred item.

This prevents theory from remaining decorative.

## 13. Deferred But Intended Formal Areas
The following areas may remain deferred in exact closure while still being explicitly intended:
1. stronger proof treatment of dynamic-dependency incremental correctness,
2. stronger proof treatment of advanced optimization lanes,
3. richer liveness/fairness analysis for later scheduling policies,
4. later substrate-specific formalization after grid introduction.

Deferral is acceptable only when documented, not when hidden.

## 14. Relationship To Other Core Docs
This assurance document depends on and validates:
1. `CORE_ENGINE_ARCHITECTURE.md`
2. `CORE_ENGINE_STATE_AND_SNAPSHOTS.md`
3. `CORE_ENGINE_RECALC_AND_INCREMENTAL_MODEL.md`
4. `CORE_ENGINE_OVERLAY_AND_DERIVED_RUNTIME.md`
5. `CORE_ENGINE_COORDINATOR_AND_PUBLICATION.md`
6. `CORE_ENGINE_OXFML_SEAM.md`
7. `CORE_ENGINE_REPLAY_APPLIANCE_ADAPTER.md`

The roadmap document then binds these assurance obligations to staged realization and promotion gates.

## 15. Open Detailed Questions
These remain assurance-planning questions within the now-locked architecture:
1. exact Lean module split beyond the first Stage 1 state vocabulary file,
2. exact TLA+ state-variable factoring between coordinator, overlay, and observer state,
3. exact trace schema ownership split between OxCalc and OxFml,
4. exact boundary between theorem-backed claims and replay-backed claims for dynamic-dependency behavior.

## 16. Status
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - the first Lean, TLA+, replay-seed, and measurement-schema artifacts now exist under `formal/`; the Lean state file has been typechecked locally and the TLA+ smoke model has been checked with TLC
  - replay-appliance evidence now reaches ordinary bundle validation, explain, and distill validation, but pack-grade promotion remains a later lane
  - runtime instrumentation now emits per-scenario counter artifacts through the `TraceCalc` runner, but replay-linked summaries and promotion-threshold reporting remain later lanes
  - replay-appliance evidence now reaches `cap.C4.distill_valid`, while `cap.C5.pack_valid` remains a later bounded promotion lane
  - handoff packet text for shared trace/reject clauses is still only partially exercised
