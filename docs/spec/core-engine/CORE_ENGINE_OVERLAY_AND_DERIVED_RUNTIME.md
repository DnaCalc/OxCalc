# CORE_ENGINE_OVERLAY_AND_DERIVED_RUNTIME.md

## 1. Purpose and Status
This document defines the OxCalc runtime overlay and derived-runtime model for the rewritten
core-engine spec set.

Status:
1. active rewrite baseline,
2. intended canonical companion for runtime-derived state,
3. TreeCalc-first in immediate scope,
4. later substrate-specific overlay families remain staged-later unless explicitly promoted.

This document defines:
1. what counts as runtime-derived state,
2. overlay taxonomy at the architectural level,
3. create/reuse/retain/evict rules,
4. epoch-safe attachment to structural snapshots,
5. what replay and assurance obligations overlays must satisfy.

## 2. Why Overlays Are Explicit
Overlays are explicit because some engine-relevant facts are runtime-derived and must not be
modeled as silent mutation of structural truth.

Without an explicit overlay model, the engine drifts into one of two failures:
1. runtime facts are hidden inside mutable caches with no semantic contract,
2. runtime-derived behavior is forced back into structural truth where it does not belong.

The rewritten OxCalc architecture rejects both failures.

## 3. Overlay Definition
An overlay is derived runtime state attached to structural truth under explicit version,
epoch, and fence rules.

An overlay is not:
1. canonical structural truth,
2. an implicit mutable side table with undefined lifecycle,
3. observer-visible committed state by default,
4. a substitute for the operation model.

An overlay may:
1. record runtime-observed dependency effects,
2. record execution-state or invalidation-state facts,
3. provide reusable derived data under strict fence compatibility,
4. support incremental execution and publication,
5. be retained or evicted under deterministic policy.

## 4. Overlay Principles

### 4.1 Derived, Not Foundational
Structural truth remains primary.
Overlays are secondary and derived.

### 4.2 Version-Fenced
Every overlay must be attributable to a compatible structural snapshot and version discipline.

### 4.3 Deterministic Lifecycle
Overlay creation, reuse, invalidation, retention, and eviction must be deterministic.

### 4.4 Replay-Relevant
Overlay-relevant behavior must be sufficiently visible to replay and diagnostics for the claims made about it.

### 4.5 Pinned-Reader Safe
Overlay reuse and eviction must respect pinned-reader and stable-view guarantees.

## 5. Overlay Taxonomy
The exact implementation taxonomy may evolve, but the architecture requires at least the
following classes of runtime-derived state to be modeled explicitly where they exist.

### 5.1 Invalidation and Execution-State Overlay
This overlay class carries runtime execution facts such as:
1. invalidation state,
2. demanded or necessary status,
3. verification progress,
4. in-flight execution state.

This class exists because these facts are runtime-dependent and should not be encoded as immutable structural truth.

### 5.2 Dynamic-Dependency Overlay
This overlay class carries runtime-observed dependency effects that are not fully fixed by static binding alone.

At the architecture level, it exists to support:
1. explicit runtime-observed dependencies,
2. replay-visible dependency-set evolution where required,
3. from-scratch-consistency obligations for dynamic dependency handling.

### 5.3 Capability and Fence-Scoped Runtime Attachments
Some runtime state is tied to evaluator sessions, capability decisions, or publication fences.

Where such state influences recalc or publication behavior, it must be modeled explicitly as runtime-derived and fence-scoped.

### 5.4 Observer and Priority Metadata Overlay
Observer-facing or scheduling-priority metadata may exist as runtime state.

This may include:
1. demanded frontier state,
2. visibility-priority metadata,
3. other runtime ordering hints.

These facts may shape work ordering or stabilization timing, but not semantic truth.

### 5.5 Staged-Later Substrate-Specific Overlays
Later phases may introduce richer substrate-specific overlays.

These remain architecturally anticipated but are not to be smuggled into the TreeCalc baseline without explicit promotion.

## 6. Overlay Attachment Model

### 6.1 Structural Attachment
Every overlay instance must be attributable to:
1. a structural snapshot identity,
2. the relevant version or token discipline,
3. the applicable profile/version context,
4. any additional required fence context for safe reuse.

### 6.2 Overlay Compatibility Rule
Overlay reuse is allowed only when the required compatibility conditions hold.

At a minimum, compatibility must consider:
1. structural snapshot compatibility,
2. profile/version compatibility,
3. token or artifact compatibility where relevant,
4. any runtime fence conditions required by the overlay class.

### 6.3 No Cross-Snapshot Smuggling
An overlay derived from one incompatible structural snapshot must not be treated as valid for another snapshot merely because it seems operationally convenient.

## 7. Overlay Lifecycle

### 7.1 Creation
Overlay state is created when runtime execution, observation, or publication logic requires derived state that does not belong in structural truth.

Creation must be:
1. explicit in implementation logic,
2. attributable to a compatible snapshot/fence context,
3. deterministic under equivalent replay conditions.

### 7.2 Reuse
Overlay reuse is permitted only when compatibility rules hold.

Reuse is valuable because:
1. it enables incremental execution efficiency,
2. it avoids unnecessary rebuild work,
3. it supports staged optimization lanes.

But reuse is never allowed to weaken semantic or replay guarantees.

### 7.3 Invalidation
Overlay state must be invalidated when its compatibility basis no longer holds.

Typical invalidation classes include:
1. structural-snapshot mismatch,
2. artifact/token mismatch,
3. profile/version mismatch,
4. explicit policy or fence mismatch,
5. runtime-discovered contradiction with current execution facts.

Detailed invalidation triggers are class-specific, but the architecture requires them to be explicit.

### 7.4 Retention
Overlay state may be retained across work when doing so is safe and justified.

Retention is never unconditional.
It is bounded by:
1. compatibility,
2. pinned-reader safety,
3. deterministic retention policy,
4. replay/evidence discipline.

### 7.5 Eviction
Overlay eviction must be deterministic and epoch-safe.

Eviction policy must ensure:
1. no pinned reader loses the state needed for its stable view,
2. no observer-visible committed state depends on already-evicted overlay material without replacement,
3. no future claim of replay determinism is undermined by ambiguous retention behavior.

### 7.6 Stage 1 Retention and Eviction Matrix
The first Stage 1 overlay retention matrix should be:

1. `invalidation_execution_state`
   - retain while the owning node remains `dirty_pending`, `needed`, `evaluating`, `publish_ready`, or protected by a pinned reader.
   - evict when the owning node has returned to `clean` or `verified_clean`, no pinned reader requires the prior state, and no replay capture policy still references the instance.
2. `dynamic_dependency`
   - retain while its `struct_snapshot_id`, `compatibility_basis`, and owning-node identity remain compatible, and while no reject or fallback path has invalidated the observed dependency shape.
   - evict when superseded by a newer accepted publication, invalidated by reject or fallback, or released beyond the safe pinned-epoch boundary.
3. `capability_fence_attachment`
   - retain while the associated capability basis and candidate/publication decision remain live.
   - evict immediately on capability mismatch, publication-fence mismatch, or after the accepted or rejected decision has been recorded and no pinned reader depends on the attachment.
4. `observer_priority_metadata`
   - retain only while the current demanded frontier or pinned-reader policy needs it.
   - evict when the demanded frontier is released, when a newer publication supersedes it, or when deterministic policy says it is no longer required.

The purpose of this first matrix is to remove ambiguity about when Stage 1 overlays survive across work and when they must be dropped.

### 7.7 Stage 1 Fallback Consequences
The first Stage 1 fallback consequences should be:
1. missing required runtime-derived effect detail invalidates reuse of the affected dynamic-dependency overlay,
2. incompatible overlay key basis forces discard rather than speculative remapping,
3. rejected candidate work may preserve diagnostics but must not preserve publish-scoped overlay consequences as if they were accepted,
4. unsupported region or spill-like shape effects force explicit conservative fallback and trace labeling,
5. host-injected fallback used by the harness must follow the same visibility and eviction rules as organic fallback.

## 8. Epoch-Safe Retention and Eviction

### 8.1 Epoch Safety Principle
Overlay retention and eviction must respect the minimum epoch or publication window required by active readers, active sessions, and stable observer-visible views.

### 8.2 Watermark-Like Rule
The architecture expects an explicit watermark or equivalent safe-eviction boundary.

The exact mechanism may vary, but the semantic rule is:
1. overlays older than the safe boundary may be evicted if not otherwise retained,
2. overlays at or beyond a pinned boundary must remain available while required.

### 8.3 Proof and Model Implications
Pinned-epoch overlay safety is not merely operational hygiene.
It is an assurance target and belongs in the formalization and pack regime.

## 9. Overlay Visibility Rules

### 9.1 Internal Versus Published Runtime State
Not all overlay state is observer-visible.

The architecture distinguishes:
1. internal runtime-derived state used only for execution,
2. runtime-derived state whose consequences become part of stable observer-visible publication.

### 9.2 Publish Boundary Rule
Overlay state affects stable observation only through explicit publication rules.

This prevents:
1. leakage of uncommitted runtime work,
2. accidental observer dependence on internal mutable state,
3. torn visibility of partially integrated overlay effects.

### 9.3 Reject Rule
Rejected work must not leave behind observer-visible overlay consequences.

Any overlay effects produced during rejected work may remain in diagnostics or replay artifacts where appropriate,
but they must not masquerade as accepted published state.

## 10. Dynamic Dependency Overlay Direction
Dynamic dependency handling is the most important baseline overlay class for the rewritten engine.

The architecture requires:
1. explicit runtime-derived representation of dynamic dependency effects,
2. compatibility with from-scratch-consistency reasoning,
3. conservative fallback when the optimized path is not yet justified,
4. instrumentation for economics and correctness.

The exact internal representation may evolve, but the spec now makes the architectural commitment explicit.

## 11. Observer and Scheduling Metadata Overlay Direction
Observer- or priority-related metadata may shape scheduling and stabilization behavior.

The architecture allows such metadata only under these rules:
1. it is runtime-derived, not semantic truth,
2. it is deterministic under the declared mode,
3. it cannot redefine final stabilized meaning,
4. fairness and starvation constraints remain explicit when such policies are enabled.

## 12. Overlay and Recalc Interaction
Overlays interact with recalc in three primary ways:
1. they may influence work discovery,
2. they may support work reuse and verification,
3. they may record runtime-observed facts needed for subsequent incremental work.

But overlays do not dissolve the boundary between:
1. immutable structure,
2. runtime execution state,
3. committed publication.

## 13. Overlay and Coordinator Interaction
The coordinator remains the authority for whether overlay consequences matter to committed publication.

The coordinator therefore governs:
1. which overlay-derived effects may become part of accepted published state,
2. how overlay state is treated on reject,
3. how overlay retention and eviction respect active readers and publication boundaries.

This prevents local runtime optimization logic from silently becoming publication law.

## 14. Formalization and Evidence Direction
Overlay behavior must be assurance-friendly.

Expected obligations include:
1. explicit compatibility and invalidation rules,
2. replay evidence for overlay-relevant behavior where required,
3. theorem or strong-evidence targets for dynamic dependency consistency,
4. TLA+ or equivalent model obligations for pinned-epoch-safe retention and eviction,
5. empirical counters for overlay reuse, miss, fallback, and GC behavior.

## 15. Open Detailed Questions
These remain implementation-detail questions within the now-locked architectural frame:
1. exact overlay key fields by overlay class beyond the now-declared Stage 1 floor,
2. exact retention thresholds and policy tuning,
3. exact representation of session-scoped versus publish-scoped runtime attachments,
4. exact observer-priority metadata encoding,
5. exact diagnostics schema for overlay-lifecycle replay.

## 16. Status
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - the first Stage 1 retention and eviction matrix is now explicit, but exact overlay-class schemas and thresholds are not yet fixed,
  - replay and pack binding for fallback and eviction behavior still need W009 realization,
  - seam consequences beyond the Stage 1 floor are not yet tied into narrower OxFml follow-on text


