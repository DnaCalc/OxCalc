# CORE_ENGINE_STATE_AND_SNAPSHOTS.md

## 1. Purpose and Status
This document defines the OxCalc state kernel for the rewritten core-engine spec set.

Status:
1. active rewrite baseline,
2. intended canonical state/snapshot companion to `CORE_ENGINE_ARCHITECTURE.md`,
3. TreeCalc-first in immediate substrate scope,
4. subject to supporting-detail refinement in later documents for recalc, overlays, and publication.

This document is about structural truth, stable identity, snapshots, projections, and
reader-visible stability. It is not the full recalc or coordinator document.

## 2. Scope
This document defines:
1. immutable structural truth for the TreeCalc-first engine,
2. stable identity rules,
3. projection and reference-syntax separation,
4. snapshot versioning and epoch attachment,
5. pinned-reader and observer stability rules,
6. what runtime state may and may not do relative to structural truth.

It does not define in full:
1. recalc state machines,
2. overlay taxonomies beyond state-boundary rules,
3. commit and reject semantics,
4. full OxFml seam details.

## 3. State-Kernel Principles
The OxCalc state kernel is governed by six rules.

### Rule 1: Structural Truth Is Immutable
Canonical structural state is immutable.

Any persistent structural change produces a new structural snapshot rather than mutating
an existing one in place.

### Rule 2: Identity Precedes Projection
Stable identity is the engine truth.
Projection and display forms are derived.

### Rule 3: Runtime State Is Derived and Versioned
Invalidation state, overlays, publications, and cached execution products are runtime state,
not structural truth. They are attached to structural snapshots through explicit epoch/fence rules.

### Rule 4: Pinned Readers See Stable State
Any pinned reader or observer must see a stable snapshot-consistent structural and runtime view.
No torn hybrid view is allowed.

### Rule 5: Structural Truth Must Not Be Smuggled Through Runtime Mutation
If a fact belongs to structural truth, it must not be maintained only as a mutable runtime cache.

### Rule 6: Future Concurrency Must Not Force State-Model Replacement
The state kernel must already be suitable for staged concurrent and async realization.

## 4. TreeCalc Structural Scope
The immediate structural substrate is tree-based.

Baseline assumptions:
1. the engine operates over tree-structured calculation entities,
2. no grid substrate is assumed,
3. no coordinate grid identity is required in baseline state truth,
4. reference forms and projections may be tree-host-specific,
5. later grid introduction is an extension of the state kernel, not its replacement.

TreeCalc-first therefore changes where the primary simplification happens:
1. not by weakening snapshot or epoch rigor,
2. but by avoiding grid-native substrate complexity in the first proving host.

## 5. Structural Truth Model

### 5.1 Structural Snapshot
The fundamental structural truth object is the structural snapshot.

It represents:
1. the immutable tree substrate,
2. the stable IDs for entities in that substrate,
3. the structural metadata needed for dependency and coordinator logic,
4. references to immutable evaluator artifacts where required,
5. the snapshot's version identity.

A structural snapshot is not:
1. a mutable working set,
2. a transient editor buffer,
3. a runtime overlay store,
4. a publication queue.

### 5.2 Structural Contents
At minimum, structural truth must support:
1. root and containment relationships,
2. typed node identity,
3. immutable payload references for node-local semantics,
4. host-relevant metadata needed for deterministic traversal and projection,
5. links to formula/bind artifacts through explicit versioned handles.

The precise node taxonomy may evolve, but it must remain compatible with the immutable,
versioned-snapshot discipline.

### 5.3 Structural Change Rule
A structural change creates a successor structural snapshot.

This requires:
1. a new snapshot version identity,
2. preservation of unchanged substructure identity when semantics allow,
3. explicit derivation invalidation for all runtime state attached to prior snapshots,
4. replay visibility through the operation model.

## 6. Stable Identity Model

### 6.1 Identity Is Stable-ID Based
Engine truth is keyed by stable IDs appropriate to the tree substrate.

These IDs must support:
1. stable reference across snapshots when the entity persists semantically,
2. deterministic replay,
3. precise dependency and observer tracking,
4. later projection into host-facing reference forms.

### 6.2 Identity Is Not Projection Text
Projection text is never the canonical identity of an entity.

This means:
1. a change in host-facing reference syntax does not redefine engine identity,
2. later grid coordinates may be projections rather than identity,
3. diagnostics may include projection text without making projection text the truth model.

### 6.3 Identity Reuse Policy
The engine must define stable-ID reuse carefully.

Baseline requirement:
1. identity reuse, if allowed at all, must be explicit, deterministic, and replay-safe.

This remains a detailed design point for supporting documents, but the architecture here
locks the principle that identity policy is a semantic concern, not a casual implementation detail.

## 7. Formula and Evaluator Artifact References
OxCalc consumes immutable evaluator-facing artifacts from OxFml.

For state-kernel purposes:
1. formula text is not interpreted by OxCalc as mutable structural truth,
2. OxCalc stores or references immutable evaluator artifacts through explicit handles,
3. those handles participate in snapshot/version/token discipline,
4. formula/bind artifact drift must be observable through versioned fences rather than hidden mutation.

This is how the state kernel stays compatible with an OxFml architecture that also treats
formula parse/bind structures as immutable, Roslyn-like artifacts.

## 8. Facades, Projections, and Derived Views

### 8.1 Facade Principle
The architecture permits facade or projection objects that attach context to immutable structure.

Examples may include:
1. traversal helpers,
2. host projection objects,
3. cached projection/index views,
4. observer-scoped browsing views.

These are allowed only if they are:
1. derived,
2. disposable,
3. rebuildable from structural truth plus explicit runtime state,
4. unable to silently redefine structural truth.

### 8.2 Projection Principle
Projection is any host-facing or user-facing representation derived from stable identity and structural state.

Projection may include:
1. tree-host reference forms,
2. display-oriented paths or symbolic labels,
3. later grid coordinates for later phases.

Projection rules must be:
1. deterministic,
2. version-aware where needed,
3. non-authoritative relative to stable identity.

### 8.3 Cached View Rule
Cached views may exist for performance, but they are not truth.

If a cached view disagrees with structural truth or explicit runtime derivation rules, the cached view is wrong.

## 9. Snapshot Versioning

### 9.1 Snapshot Identity
Every structural snapshot must carry a version identity.

That identity exists so that:
1. runtime state can fence against the structural truth it was derived from,
2. pinned readers know what they are observing,
3. replay and proof obligations have a stable anchor,
4. coordinator publication has an unambiguous base object.

### 9.2 Successor Relation
Snapshots form a deterministic successor relation through explicit operations.

The state kernel therefore assumes:
1. no hidden persistent mutation path,
2. all structural transitions are caused through the operation model,
3. any successor snapshot can be tied to an operation/replay history.

### 9.3 Multi-Snapshot Coexistence
Multiple snapshots may coexist while:
1. readers remain pinned to earlier ones,
2. runtime work proceeds against later ones,
3. publication catches up.

This coexistence is a design requirement, not a debug-only feature.

## 10. Epoch and Runtime Attachment Model
Structural snapshots and runtime execution state are related, but they are not the same thing.

At the top level:
1. structural snapshots are immutable truth,
2. runtime layers are attached through epoch/fence semantics,
3. publication determines what stable view becomes observer-visible,
4. runtime reuse and eviction must be defined against pinned-reader constraints.

Detailed epoch semantics belong to coordinator/publication documents, but the state-kernel
contract requires runtime state to record which structural snapshot and version discipline it belongs to.

## 11. Pinned Readers and Stable Views

### 11.1 Pinned Reader Rule
A pinned reader observes a stable state bundle.

That bundle includes:
1. a structural snapshot,
2. the runtime-derived view that is valid for that snapshot and publication state,
3. status information appropriate to that view.

### 11.2 No Torn View Rule
A reader must not observe a torn hybrid such as:
1. structure from one snapshot with publication state from an incompatible later snapshot,
2. partially published derived state,
3. overlay effects that are not valid for the observed snapshot fence.

### 11.3 Observer Stability During Ongoing Recalc
Ongoing recalculation work may proceed after a reader has pinned a stable view.

But that later work must not retroactively mutate what the pinned reader sees.

This rule is central for:
1. correctness under concurrency,
2. deterministic diagnostics,
3. replay fidelity,
4. future async publication.

## 12. What Runtime State May Not Do
Runtime state may not:
1. silently rewrite canonical structure,
2. act as the only source of a structural fact,
3. redefine stable identity,
4. force pinned readers onto incompatible state,
5. leak uncommitted or rejected state into stable publication.

These prohibitions matter as much as the positive model.
They protect the architecture from being eroded by performance shortcuts.

## 13. Later Grid Introduction Boundary
This document defines the state kernel so that later grid introduction can fit into it.

Later phases may add:
1. coordinate projections,
2. grid-oriented identity projections,
3. substrate-specific rewrite rules,
4. region and occupancy structures,
5. richer projection caches.

But later phases must still obey the rules defined here:
1. immutable structural truth,
2. identity preceding projection,
3. derived/runtime separation,
4. pinned-reader stability,
5. versioned snapshot coexistence.

## 14. Formalization Direction
The state kernel is intended to map directly into formal structures.

Minimum expected formalization consequences:
1. Lean-facing definitions for structural snapshots, stable IDs, and derived-view boundaries,
2. theorem targets for no hidden mutation of structural truth,
3. theorem or model obligations for pinned-reader safety,
4. TLA+ state variables that separate structural truth, runtime state, and published view.

The detailed proof and model-check plan belongs in the assurance companion document,
but this document defines the architectural objects that plan must operate on.

## 15. Open Detailed Questions
These remain detailed follow-on questions rather than reasons to weaken the model:
1. exact stable-ID scoping and reuse rules,
2. exact host-facing projection families for TreeCalc,
3. exact representation of immutable evaluator artifact handles,
4. exact facade/cache shape for performance-sensitive traversals.

## 16. Status
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - detailed ID policy not yet locked,
  - runtime overlay companion not yet drafted,
  - coordinator/publication companion not yet drafted
