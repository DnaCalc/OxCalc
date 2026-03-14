# CORE_ENGINE_ARCHITECTURE.md

## 1. Purpose and Status
This document defines the top-level architecture for the rewritten OxCalc core-engine spec set.

Status:
1. active rewrite baseline,
2. intended authoritative architecture for OxCalc core-engine work,
3. TreeCalc-first in immediate execution scope,
4. subject to supporting-detail expansion in companion documents rather than ad hoc drift.

This document is intentionally architecture-defining rather than implementation-minimal.
It exists to make the intended OxCalc engine shape explicit before realization work proceeds.

## 2. Architectural Mission
OxCalc defines the multi-node core calculation engine lane for DNA Calc.

For the current rewrite, the engine mission is:
1. build the core engine required for DNA TreeCalc first,
2. do so on a tree-based substrate with no grid as baseline scope,
3. preserve deterministic semantics under staged runtime strategy changes,
4. build from the start toward a fast, scalable, high-quality incremental engine,
5. make the OxFml evaluator boundary explicit and robust,
6. carry near-formal specification and verification into the architecture itself.

## 3. Immediate Target: DNA TreeCalc
The first serious OxCalc target is the engine needed for DNA TreeCalc.

This target deliberately constrains immediate scope:
1. tree-based structure,
2. no grid substrate,
3. no hidden grid assumptions in baseline architecture text,
4. no requirement that reference syntax exactly mirror Excel A1 surface forms,
5. explicit room for tree-host-oriented reference forms and projection rules,
6. later grid introduction as a major phase rather than a baseline premise.

Interpretation rule:
1. TreeCalc-first does not mean "toy" or "temporary."
2. TreeCalc-first means the initial executable engine proves the core architectural model on a simpler substrate before grid complexity is introduced.
3. All baseline semantics for TreeCalc must remain consistent with later expansion to a broader substrate.

## 4. Architectural Pillars

### 4.1 Immutable Structural Truth
Immutable structural truth is foundational.

The architecture must treat the following as immutable, versioned truth:
1. core tree structure,
2. stable node identity,
3. structural metadata that defines engine-observable shape,
4. immutable formula artifacts and bind products received from OxFml or derived through explicit versioned seams.

This rule exists for correctness first and reuse second:
1. runtime work must not silently mutate canonical structure,
2. stable snapshots must remain safely observable during later recalculation work,
3. replay and proof obligations become tractable only if truth is versioned and immutable,
4. future concurrency depends on readers being able to observe stable state without coordinator races.

Roslyn-style persistence lessons are adopted as architectural guidance:
1. immutable core structures should be context-free and compact,
2. context-bearing facades and cached projections should be derived and ephemeral,
3. edits should respin only the affected immutable spine and changed payloads,
4. unchanged substructures should be preserved by identity when semantics allow.

### 4.2 Versioned Runtime and MVCC-Derived State
Runtime state is not structural truth.

Runtime state includes:
1. invalidation state,
2. dependency overlays,
3. recalculation scheduling state,
4. pinned-reader and observer state,
5. publication bookkeeping,
6. reusable derived caches,
7. other epoch-scoped execution state.

The architecture requires this state to be:
1. explicitly versioned,
2. attached to immutable structural snapshots by epoch and fence rules,
3. safe under pinned-reader semantics,
4. evictable by deterministic epoch-safe rules,
5. observable through stable views even while later work is underway.

This is an MVCC-style discipline, but the engine should not rely on slogan-level MVCC language.
The spec set must define actual snapshot, fence, publish, reject, and retention semantics.

### 4.3 Single-Publisher Coordinator Authority
The baseline engine has one publication authority: the coordinator.

The coordinator is the single authority for:
1. snapshot acceptance and epoch advancement,
2. commit acceptance or rejection,
3. atomic publication of derived results,
4. runtime overlay lifecycle decisions that affect committed visibility,
5. safe observer-facing state transitions.

This is a baseline safety rule, not a performance concession.
Parallel evaluation may be introduced later, but parallel evaluators do not bypass the coordinator.

### 4.4 Staged Concurrent and Async Design
Concurrency and asynchronous recalculation are designed in from the start.

The architecture must be written so that:
1. Stage 1 sequential realization is a strict subset of the intended concurrent design,
2. later async and multithreaded work does not require replacing the structural model,
3. snapshot fences, publication rules, reject semantics, and observer visibility rules are already strong enough for staged concurrency,
4. deterministic replay remains mandatory under concurrent and async execution.

### 4.5 Near-Formal Assurance Architecture
Near-formal modeling is part of the architecture itself.

The rewritten OxCalc architecture must map claims to:
1. Lean-facing semantic models and theorem targets,
2. TLA+ concurrency and async models,
3. replay artifacts,
4. conformance packs,
5. deterministic empirical measurements where proof alone is insufficient.

Architectural text that cannot be expressed in one of these assurance forms should be treated carefully and explicitly labeled if still provisional.

## 5. Layered Engine Model
The engine is organized around explicit layers with strict truth/derived/runtime boundaries.

### 5.1 Structural Snapshot Layer
This layer contains immutable TreeCalc structural truth.

It includes:
1. node identity,
2. parent/child structural relationships,
3. structural metadata relevant to calculation,
4. references to immutable formula/bind artifacts as needed,
5. version identity for the snapshot itself.

This layer is the base object observed by higher layers.

### 5.2 Evaluator Artifact Layer
OxCalc depends on OxFml for formula-language and evaluator-facing artifacts.

For OxCalc architecture purposes, these artifacts are treated as:
1. immutable inputs to coordinator and dependency logic,
2. versioned by token/profile/bind context,
3. subject to explicit seam contracts rather than implicit mutation.

OxCalc does not own formula grammar or evaluator semantics, but it does own how these artifacts participate in coordinator, scheduling, publication, and replay behavior.

### 5.3 Structural Dependency Layer
This layer contains the dependency structure derivable from structural truth and stable evaluator artifacts.

Baseline properties:
1. deterministic derivation,
2. explicit forward and reverse dependency relations,
3. explicit cycle-region handling,
4. no hidden mutation from runtime discovery.

### 5.4 Runtime Overlay Layer
This layer contains epoch-scoped runtime-derived state that cannot be treated as immutable structural truth.

Examples include:
1. dynamic-dependency observations,
2. runtime invalidation state,
3. versioned dependency or capability overlays,
4. observer-facing scheduling metadata,
5. later-phase substrate-specific overlays.

The overlay layer is explicit because runtime-discovered behavior must not be represented as silent mutation of the structural dependency graph.

### 5.5 Publication and Observer Layer
This layer governs what stable state is visible to readers and subscribers.

It includes:
1. committed snapshot identity,
2. stabilized or published calculation view,
3. status signaling such as stale/pending/ready/error as defined by companion docs,
4. publication ordering and atomicity rules,
5. observer pinning and snapshot visibility rules.

## 6. Structural Identity and Projection Rules
Baseline identity is stable-ID based, not projection-text based.

The architecture therefore distinguishes:
1. identity,
2. projection,
3. reference syntax.

Identity:
1. engine truth is keyed by stable IDs appropriate to the tree substrate.

Projection:
1. user-facing or host-facing reference forms may be derived from structural state,
2. projection formats may differ by host or profile,
3. projection changes must not redefine engine identity.

Reference syntax:
1. TreeCalc may use reference forms that differ from Excel grid-address conventions,
2. later grid introduction may add projection families without invalidating the identity model.

## 7. Recalc and Incremental Architecture Direction
The architecture is conservative in scope but not timid in design.

### 7.1 Baseline Recalc Direction
The baseline recalc engine is deterministic and topo/SCC-based.

Required baseline traits:
1. deterministic scheduling order,
2. explicit cycle-region handling,
3. explicit invalidation-state model,
4. strong replay compatibility,
5. conservative fallback when later-stage optimization conditions are not met.

### 7.2 Incremental Ambition From The Start
Although Stage 1 realization may be conservative, the architecture is explicitly aimed at a strong incremental engine.

This means the spec must carry forward:
1. explicit invalidation-state semantics,
2. verification-oriented recalculation direction,
3. early-cutoff design intent,
4. runtime-observed dependency support through overlays,
5. instrumentation and decisive experiments to validate high-value optimization lanes.

The architecture must not pretend that dirty-closure-only thinking is sufficient for the intended long-term engine.

### 7.3 Dynamic Dependency Discipline
Dynamic references and other runtime-discovered dependency behaviors are first-class architectural concerns.

They must be handled through explicit runtime-derived state and explicit replay/proof obligations.
They must not be smuggled into the engine as ad hoc exceptions to a purely static dependency story.

### 7.4 Dynamic-Topo and SAC-Inspired Lanes
Advanced lanes such as dynamic topological maintenance and SAC-inspired repair remain intended design space, but not baseline realization commitments.

They are:
1. architecturally anticipated,
2. explicitly named in the roadmap,
3. subject to promotion by parity, replay, and economics evidence.

## 8. Snapshot, Epoch, and Publication Architecture
The engine must distinguish structural change, recalculation work, and observable publication.

At the top level, the architecture requires:
1. immutable structural snapshots,
2. epoch-bearing runtime state and publication fences,
3. stable observer-visible views,
4. atomic accepted-commit publication,
5. strict no-publish reject semantics.

Two rules are mandatory:
1. no observer may be forced to read a torn hybrid of incompatible structural/runtime state,
2. no accepted derived publication may be partially visible.

The detailed epoch model belongs in companion documents, but these invariants belong in the architecture itself.

## 9. Coordinator and OxFml Seam Architecture
OxCalc owns coordinator policy and publication semantics.
OxFml owns evaluator semantics and canonical shared seam specification.

The architecture must therefore make the coordinator-facing seam explicit.

That seam includes:
1. session and snapshot expectations,
2. token and capability fence implications,
3. commit acceptance and rejection consequences,
4. publication payload expectations for accepted work,
5. replay-oriented reject detail requirements,
6. ownership boundaries for what OxCalc may specify locally versus what must be handed off to OxFml.

The architecture should be written so that seam hardening is a normal follow-on activity, not a late clarification exercise.

## 10. Staged Realization Model

### 10.1 Stage 1
Stage 1 is the first realization baseline.

Its role is to prove:
1. immutable structural snapshot discipline,
2. deterministic topo/SCC coordinator behavior,
3. single-publisher commit and publication authority,
4. explicit epoch and replay rules,
5. stable observer view under ongoing work,
6. TreeCalc-first substrate semantics.

### 10.2 Stage 2
Stage 2 introduces partitioned or concurrent evaluator work behind the same coordinator publication authority.

Its role is to prove:
1. fence correctness under concurrency,
2. deterministic contention handling and replay,
3. safe publication under pinned-reader and observer constraints,
4. concurrency without semantic drift.

### 10.3 Stage 3 and Beyond
Stage 3 and beyond may introduce more ambitious runtime strategies, but only through explicit promotion gates.

No later-stage optimization is allowed to redefine baseline semantic truth.

## 11. TreeCalc-First, Grid-Later Boundary
This architecture intentionally separates:
1. core engine truth and coordinator design,
2. TreeCalc-first proving scope,
3. later grid introduction.

The later grid phase may add:
1. richer projection rules,
2. substrate-specific rewrite semantics,
3. more complex region and occupancy behavior,
4. later-phase overlay classes.

But it must fit into the same architectural pillars:
1. immutable structural truth,
2. versioned runtime and publication layers,
3. coordinator publication authority,
4. near-formal assurance discipline.

## 12. Formalization Direction
The architecture requires a supporting formalization program.

At minimum, the companion assurance document must define:
1. Lean-facing state and transition structures,
2. theorem targets for replay determinism and no hidden structural mutation,
3. TLA+ models for coordinator safety, async/concurrent publication, and pinned-epoch GC,
4. replay artifact requirements,
5. empirical pack obligations and decisive experiments.

Near-formal here means:
1. not every clause is proven immediately,
2. but every major architectural claim is expected to route toward proof, model-checking, or deterministic evidence.

## 13. Explicit Non-Baseline Items
The following are not baseline realization commitments in the immediate TreeCalc engine, even if they remain part of the broader architecture discussion:
1. grid-native substrate semantics,
2. grid-driven spill/occupancy baseline behavior,
3. default adoption of dynamic-topological maintenance,
4. default adoption of SAC-style repair as the first realization strategy,
5. speculative or lock-free publication paths.

These may remain staged-later lanes or deferred material, but they are not to be smuggled into the TreeCalc baseline through vague wording.

## 14. Relationship To Companion Documents
This top-level architecture document is complemented by:
1. `CORE_ENGINE_STATE_AND_SNAPSHOTS.md`
2. `CORE_ENGINE_RECALC_AND_INCREMENTAL_MODEL.md`
3. `CORE_ENGINE_OVERLAY_AND_DERIVED_RUNTIME.md`
4. `CORE_ENGINE_COORDINATOR_AND_PUBLICATION.md`
5. `CORE_ENGINE_OXFML_SEAM.md`
6. `CORE_ENGINE_FORMALIZATION_AND_ASSURANCE.md`
7. `CORE_ENGINE_REALIZATION_ROADMAP.md`

Those documents provide the detailed semantics, staged realization, and assurance mapping.
This document sets the architectural frame they must remain consistent with.

## 15. Status
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - supporting companion docs not yet drafted,
  - exact Stage 1 incremental wording still to be tightened in the recalc model,
  - OxFml handoff clauses not yet extracted into a handoff packet
