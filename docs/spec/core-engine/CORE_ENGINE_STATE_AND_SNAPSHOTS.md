# CORE_ENGINE_STATE_AND_SNAPSHOTS.md

## 1. Purpose and Status
This document defines the OxCalc state kernel for the rewritten core-engine spec set.

Status:
1. active rewrite baseline,
2. intended canonical state/snapshot companion to `CORE_ENGINE_ARCHITECTURE.md`,
3. TreeCalc-first in immediate substrate scope,
4. subject to supporting-detail refinement in later documents for recalc, overlays, and publication.

This document is about workspace truth, stable identity, snapshots, projections,
and reader-visible stability. It is not the full recalc or coordinator document.

## 2. Scope
This document defines:
1. immutable workspace truth for the TreeCalc-first engine,
2. stable identity rules,
3. projection and reference-syntax separation,
4. snapshot versioning and epoch attachment,
5. pinned-reader and observer stability rules,
6. what runtime state may and may not do relative to durable workspace truth.

It does not define in full:
1. recalc state machines,
2. overlay taxonomies beyond state-boundary rules,
3. commit and reject semantics,
4. full OxFml seam details.

## 3. State-Kernel Principles
The OxCalc state kernel is governed by seven rules.

### Rule 1: Structural Truth Is Immutable
Canonical structural state is immutable.

Any persistent structural change produces a new structural snapshot rather than mutating
an existing one in place.

### Rule 1A: Node Input Truth Is Immutable But Separate
Per-node calculation input is durable workspace truth, but it is not structural
topology truth.

Literal values, formula text, empty input state, and future host-owned input
variants belong in `NodeInputSnapshot`. An input edit produces a new
`NodeInputSnapshot`, not a new `StructureSnapshot`, unless the edit also changes
structure.

### Rule 2: Identity Precedes Projection
Stable identity is the engine truth.
Projection and display forms are derived.

### Rule 3: Runtime State Is Derived and Versioned
Invalidation state, overlays, publications, and cached execution products are runtime state,
not durable workspace truth. They are attached to workspace revision and layer
snapshot identities through explicit epoch/fence rules.

### Rule 4: Pinned Readers See Stable State
Any pinned reader or observer must see a stable snapshot-consistent workspace and runtime view.
No torn hybrid view is allowed.

### Rule 5: Durable Workspace Truth Must Not Be Smuggled Through Runtime Mutation
If a fact belongs to durable workspace truth, it must not be maintained only as a
mutable runtime cache or side map.

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

## 5. Workspace Revision Model

### 5.1 Workspace Revision
The fundamental edited truth object is `WorkspaceRevision`.

It is the immutable tuple of:
1. `StructureSnapshot`,
2. `NodeInputSnapshot`,
3. `NamespaceSnapshot`.

The revision tuple is the base object observed by higher layers. Derived and
runtime layers may be cached or retained against it, but they do not redefine
it.

### 5.2 Structure Snapshot
The fundamental structural truth object is the structural snapshot.

It represents:
1. the immutable tree substrate,
2. the stable IDs for entities in that substrate,
3. the structural metadata needed for dependency and coordinator logic,
4. the snapshot's version identity.

A structural snapshot is not:
1. a mutable working set,
2. a transient editor buffer,
3. a runtime overlay store,
4. a publication queue,
5. the authority for formula text or literal input values.

### 5.3 Structural Contents
At minimum, structural truth must support:
1. root and containment relationships,
2. typed node identity,
3. host-relevant metadata needed for deterministic traversal and projection,
4. table shape, anchors, row/column/header/totals structural metadata where
   admitted,
5. structural version identity.

The precise node taxonomy may evolve, but it must remain compatible with the immutable,
versioned-snapshot discipline.

### 5.4 Node Input Snapshot
`NodeInputSnapshot` is the durable input truth object.

It represents, per stable node identity:
1. input kind: empty, literal, formula, or future host-owned input variant,
2. literal input payloads,
3. formula text payloads,
4. input/formula text version identity,
5. stable node-input subtree identity or hash when implemented.

`NodeInputSnapshot` is not:
1. a published value store,
2. a formula binding result store,
3. a runtime dependency overlay,
4. a cache of structural constants.

### 5.5 Namespace Snapshot
`NamespaceSnapshot` is the durable host-context truth object.

It represents host facts that affect formula binding, prepared identity,
reference resolution, capability decisions, or cross-workspace availability:
1. host namespace and aliases,
2. function registry and capability profile identity,
3. workspace availability/degradation facts,
4. caller context and table context identities where durable,
5. namespace version identity.

### 5.6 Structural Change Rule
A structural change creates a successor structural snapshot.

This requires:
1. a new snapshot version identity,
2. preservation of unchanged substructure identity when semantics allow,
3. explicit derivation-impact classification for runtime state attached to prior snapshots,
4. replay visibility through the operation model.

A new `StructureSnapshotId` establishes a new structural truth root. It does
not by itself assert that every formula binding fact, dependency component,
publication value, runtime overlay, sparse reader, or cache entry is
semantically invalid.

Until a finer compatibility model is implemented, an engine may conservatively
rebuild or evict broadly. The intended model is stronger: every reusable
derived surface declares a compatibility basis, and structural edits invalidate
that surface only when the declared basis intersects the structural impact
closure of the edit or when the engine cannot prove non-intersection.

### 5.7 Input Change Rule
An input change creates a successor `NodeInputSnapshot`.

This includes:
1. literal value update,
2. formula text update,
3. literal-to-formula transition,
4. formula-to-literal transition,
5. empty-to-input and input-to-empty transitions.

These transitions preserve `StructureSnapshot` identity unless paired with a
structural operation.

### 5.8 Namespace Change Rule
A namespace, registry, capability, workspace-availability, alias, or durable
caller-context mutation creates a successor `NamespaceSnapshot`.

Prepared formula, formula binding, dependency-shape, and runtime artifacts must
declare compatibility with the namespace snapshot identity they consume.

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
1. formula text lives in `NodeInputSnapshot` and is opaque to OxCalc formula semantics,
2. OxFml parse/bind/prepared products live in or are referenced by
   `FormulaBindingSnapshot`,
3. OxCalc stores or references immutable evaluator artifacts through explicit handles,
4. those handles participate in snapshot/version/token discipline,
5. formula/bind artifact drift must be observable through versioned fences rather than hidden mutation,
6. OxCalc must not classify formula behavior by inspecting formula syntax or
   function names.

This is how the state kernel stays compatible with an OxFml architecture that also treats
formula parse/bind structures as immutable, Roslyn-like artifacts.

Derived formula and dependency state uses separate identities:
1. `FormulaBindingSnapshot` records typed OxFml parse/bind/prepared facts
   consumed by OxCalc for a compatible workspace revision and host context.
2. `DependencyShapeSnapshot` records static dependency facts derived from
   workspace roots and typed formula-binding facts.
3. Runtime dynamic dependency effects are publication/overlay facts, not
   hidden edits to either durable workspace truth or formula-binding truth.
4. `PublicationSnapshot` and `RuntimeOverlaySet` are attached to the revision
   and derived-layer identities they consume, but they are not authored
   workspace roots.

## 8. Facades, Projections, and Derived Views

### 8.1 Facade Principle
The architecture permits facade or projection objects that attach context to immutable workspace roots.

Examples may include:
1. traversal helpers,
2. host projection objects,
3. cached projection/index views,
4. observer-scoped browsing views.

These are allowed only if they are:
1. derived,
2. disposable,
3. rebuildable from durable workspace truth plus explicit runtime state,
4. unable to silently redefine durable workspace truth.

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

If a cached view disagrees with durable workspace truth or explicit runtime derivation rules, the cached view is wrong.

## 9. Snapshot Versioning

### 9.1 Snapshot Identity
Every durable workspace root snapshot must carry a version identity.

That identity exists so that:
1. runtime state can fence against the workspace truth it was derived from,
2. pinned readers know what they are observing,
3. replay and proof obligations have a stable anchor,
4. coordinator publication has an unambiguous base object.

The primary identities are:
1. `StructureSnapshotId`,
2. `NodeInputSnapshotId`,
3. `NamespaceSnapshotId`,
4. `WorkspaceRevisionId`.

### 9.2 Successor Relation
Snapshots form a deterministic successor relation through explicit operations.

The state kernel therefore assumes:
1. no hidden persistent mutation path,
2. all durable workspace transitions are caused through the operation model,
3. any successor snapshot can be tied to an operation/replay history.

### 9.3 Multi-Snapshot Coexistence
Multiple snapshots may coexist while:
1. readers remain pinned to earlier ones,
2. runtime work proceeds against later ones,
3. publication catches up.

This coexistence is a design requirement, not a debug-only feature.

## 10. Epoch and Runtime Attachment Model
Workspace revisions and runtime execution state are related, but they are not the same thing.

At the top level:
1. workspace revision roots are immutable truth,
2. runtime layers are attached through epoch/fence semantics,
3. publication determines what stable view becomes observer-visible,
4. runtime reuse and eviction must be defined against pinned-reader constraints.

Detailed epoch semantics belong to coordinator/publication documents, but the state-kernel
contract requires runtime state to record which workspace revision and layer snapshot
identities it belongs to.

## 11. Pinned Readers and Stable Views

### 11.1 Pinned Reader Rule
A pinned reader observes a stable state bundle.

That bundle includes:
1. a `WorkspaceRevision`,
2. the runtime-derived view that is valid for that revision and publication state,
3. status information appropriate to that view.

### 11.2 No Torn View Rule
A reader must not observe a torn hybrid such as:
1. structure, input, or namespace state from one revision with publication state
   from an incompatible later revision,
2. partially published derived state,
3. overlay effects that are not valid for the observed revision fence.

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
2. act as the only source of a durable workspace fact,
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
1. immutable durable workspace truth,
2. identity preceding projection,
3. derived/runtime separation,
4. pinned-reader stability,
5. versioned snapshot coexistence.

## 14. Formalization Direction
The state kernel is intended to map directly into formal structures.

Minimum expected formalization consequences:
1. Lean-facing definitions for workspace revisions, root snapshots, stable IDs, and derived-view boundaries,
2. theorem targets for no hidden mutation of durable workspace truth,
3. theorem or model obligations for pinned-reader safety,
4. TLA+ state variables that separate workspace truth, runtime state, and published view.

The detailed proof and model-check plan belongs in the assurance companion document,
but this document defines the architectural objects that plan must operate on.

## 15. Open Detailed Questions
These remain detailed follow-on questions rather than reasons to weaken the model:
1. exact stable-ID scoping and reuse rules,
2. exact host-facing projection families for TreeCalc,
3. exact representation of immutable evaluator artifact handles,
4. exact facade/cache shape for performance-sensitive traversals,
5. exact local structural compatibility basis taxonomy for future sheet,
   subtree, table-region, dependency-component, publication-shard, and subtree
   hash reuse.

## 16. Status
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - detailed ID policy not yet locked,
  - runtime overlay companion not yet drafted,
  - coordinator/publication companion not yet drafted
