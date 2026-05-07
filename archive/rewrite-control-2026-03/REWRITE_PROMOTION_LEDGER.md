# OxCalc Core-Engine Rewrite Promotion Ledger

## 1. Purpose
This ledger records the topic-by-topic promotion decisions for the OxCalc core-engine
spec rewrite.

It is the control surface between the source corpus and the new canonical OxCalc-owned
spec set. It prevents source ideas from being merely cited without being explicitly
classified and placed.

## 2. Disposition Vocabulary
- `promote_baseline`: becomes part of the rewritten baseline architecture/spec set.
- `promote_staged_later`: becomes an explicit intended later-stage architecture or
  realization lane, not hidden in archive notes.
- `retain_deferred`: remains open or intentionally deferred with rationale.
- `archive_superseded`: retained only for provenance/history, not active guidance.

## 3. Topic Entries

### RW-001: Immutable Structural Truth And Stable Identity
- **Source lineage**: Foundation February formal-model notes; March 8 deep design/review pack; March 9 layout synthesis.
- **Current OxCalc state**: partial baseline in the consolidated model, but not extensive enough for implementation guidance.
- **Rewrite disposition**: `promote_baseline`
- **Target canonical document**: `CORE_ENGINE_STATE_AND_SNAPSHOTS.md`
- **Implementation implications**: structural truth is immutable and versioned; stable identity is decoupled from transient projection.
- **Formal/pack/replay implications**: Lean-facing state kernel; proof target for no hidden mutation of structural truth.
- **Handoff implications**: none.

### RW-002: TreeCalc-First Tree-Only Scope
- **Source lineage**: Foundation March 8 synthesis pass-02; March 9 layout synthesis; existing OxCalc charter/model.
- **Current OxCalc state**: present, but compressed to a few baseline paragraphs.
- **Rewrite disposition**: `promote_baseline`
- **Target canonical document**: `CORE_ENGINE_ARCHITECTURE.md`
- **Implementation implications**: first executable OxCalc target is tree-based, no-grid, no hidden grid substrate assumptions.
- **Formal/pack/replay implications**: explicit subset relation to later grid-capable architecture; semantic-gap registry obligations.
- **Handoff implications**: none.

### RW-003: Versioned Snapshots, Epochs, And Stable Observer Views
- **Source lineage**: Foundation architecture and March 8 deep-design findings on torn-state risk; March 8 synthesis pass-02.
- **Current OxCalc state**: baseline epoch semantics exist, but observer-visible stable-state rules are under-specified.
- **Rewrite disposition**: `promote_baseline`
- **Target canonical document**: `CORE_ENGINE_STATE_AND_SNAPSHOTS.md`
- **Implementation implications**: snapshot pinning and stable observable state are required even during ongoing recalculation.
- **Formal/pack/replay implications**: TLA+ safety targets for snapshot visibility and pinned-epoch safety.
- **Handoff implications**: seam implications for what evaluator sessions may observe.

### RW-004: Runtime Overlay Taxonomy And Lifecycle
- **Source lineage**: March 5 DAG synthesis; March 8 deep research pack; March 8 synthesis pass-02.
- **Current OxCalc state**: overlay keying and eviction are present, but taxonomy and lifecycle are too compressed.
- **Rewrite disposition**: `promote_baseline`
- **Target canonical document**: `CORE_ENGINE_OVERLAY_AND_DERIVED_RUNTIME.md`
- **Implementation implications**: overlays are explicit derived runtime structures, not incidental caches.
- **Formal/pack/replay implications**: `PACK.fec.overlay_lifecycle`, overlay-GC safety, replay-visible lifecycle transitions.
- **Handoff implications**: evaluator seam must surface overlay-relevant deltas where required.

### RW-005: Single-Publisher Coordinator And Atomic Publish Contract
- **Source lineage**: March 8 review/synthesis; March 9 locked ODRs.
- **Current OxCalc state**: present as baseline contract, but not yet expanded into a full coordinator spec.
- **Rewrite disposition**: `promote_baseline`
- **Target canonical document**: `CORE_ENGINE_COORDINATOR_AND_PUBLICATION.md`
- **Implementation implications**: accepted commit publishes one atomic derived bundle; reject is strict no-publish.
- **Formal/pack/replay implications**: atomicity proof targets, reject replay detail requirements, TLA+ publication-fence model.
- **Handoff implications**: coordinator-facing FEC/F3E clauses likely require OxFml handoff alignment.

### RW-006: OxFml Evaluator Seam Boundary And Ownership Split
- **Source lineage**: Foundation architecture split, March 8 synthesis pass-02, March 9 layout synthesis.
- **Current OxCalc state**: boundary is stated, but the coordinator-facing seam requirements are not yet documented in OxCalc-local detail.
- **Rewrite disposition**: `promote_baseline`
- **Target canonical document**: `CORE_ENGINE_OXFML_SEAM.md`
- **Implementation implications**: OxCalc must define the coordinator-facing contract it expects from OxFml without claiming canonical seam ownership.
- **Formal/pack/replay implications**: replay contracts and reject-detail structure need explicit seam mapping.
- **Handoff implications**: yes, for any shared seam clause that requires canonical OxFml updates.

### RW-007: Explicit Invalidation-State Model
- **Source lineage**: March 5 DAG deep research and transfer matrix; March 8 design pack.
- **Current OxCalc state**: names like `necessary`, `stale`, and `height` are retained, but the state machine is not yet a proper implementation framework.
- **Rewrite disposition**: `promote_baseline`
- **Target canonical document**: `CORE_ENGINE_RECALC_AND_INCREMENTAL_MODEL.md`
- **Implementation implications**: invalidation state becomes an explicit engine model rather than informal scheduler vocabulary.
- **Formal/pack/replay implications**: replay traces and proof targets for invalidation-state transitions.
- **Handoff implications**: none.

### RW-008: Deterministic Topo/SCC Recalc Baseline
- **Source lineage**: March 5 DAG deep research and transfer matrix; March 8/9 synthesis.
- **Current OxCalc state**: baseline semantics present, but implementation-path detail is thin.
- **Rewrite disposition**: `promote_baseline`
- **Target canonical document**: `CORE_ENGINE_RECALC_AND_INCREMENTAL_MODEL.md`
- **Implementation implications**: deterministic topo/SCC scheduling remains the first realization baseline.
- **Formal/pack/replay implications**: replay determinism, SCC-order determinism, cycle-mode pack coverage.
- **Handoff implications**: none.

### RW-009: Verification And Early-Cutoff Incremental Strategy
- **Source lineage**: March 5 deep research reports; March 8 deep design pack and dual reviews.
- **Current OxCalc state**: only lightly represented through retained vocabulary; the optimization strategy itself is not carried into the OxCalc spec.
- **Rewrite disposition**: `promote_baseline`
- **Target canonical document**: `CORE_ENGINE_RECALC_AND_INCREMENTAL_MODEL.md`
- **Implementation implications**: the architecture should explicitly target verification-oriented incremental recalc and early cutoff, while the roadmap may still stage realization details.
- **Formal/pack/replay implications**: proof targets for cutoff soundness; decisive experiment and instrumentation requirements.
- **Handoff implications**: none.

### RW-010: Dynamic Dependency Handling Through Explicit Runtime Overlay
- **Source lineage**: March 5 research; March 8 design pack; March 8 synthesis pass-02.
- **Current OxCalc state**: present at a high level, but not developed enough as an implementation framework.
- **Rewrite disposition**: `promote_baseline`
- **Target canonical document**: `CORE_ENGINE_OVERLAY_AND_DERIVED_RUNTIME.md`
- **Implementation implications**: dynamic references are modeled through explicit runtime-observed dependency state rather than hidden mutation.
- **Formal/pack/replay implications**: from-scratch consistency targets and `PACK.dag.dynamic_dependency_bind_semantics`.
- **Handoff implications**: seam deltas may require canonical OxFml coordination.

### RW-011: Dynamic Topological Maintenance
- **Source lineage**: March 5 research reports; March 8 design pack; March 9 layout synthesis.
- **Current OxCalc state**: correctly retained as advanced/provisional.
- **Rewrite disposition**: `promote_staged_later`
- **Target canonical document**: `CORE_ENGINE_RECALC_AND_INCREMENTAL_MODEL.md`
- **Implementation implications**: treat as an intended later optimization lane with explicit crossover experiments, not default baseline.
- **Formal/pack/replay implications**: `PACK.dag.dynamic_topo_vs_rebuild` and parity/economics evidence.
- **Handoff implications**: none.

### RW-012: Staged Concurrent And Async Evaluation
- **Source lineage**: Foundation architecture/operations; March 8 design pack; March 9 layout synthesis.
- **Current OxCalc state**: stages are named, but the architectural preparation is still too compressed.
- **Rewrite disposition**: `promote_baseline`
- **Target canonical document**: `CORE_ENGINE_COORDINATOR_AND_PUBLICATION.md`
- **Implementation implications**: concurrency and async behavior are designed in from the start, even while Stage 1 remains sequential.
- **Formal/pack/replay implications**: TLA+ concurrency model, deterministic contention replay, fence safety, async publication invariants.
- **Handoff implications**: seam and reject-detail clauses may need handoff alignment.

### RW-013: Performance Instrumentation And Decisive Experiments
- **Source lineage**: March 5 deep research experiment plans; March 8 design pack; March 9 improvement notes.
- **Current OxCalc state**: pack names exist, but the deciding experiments and counters are not made explicit in OxCalc-owned planning.
- **Rewrite disposition**: `promote_baseline`
- **Target canonical document**: `CORE_ENGINE_REALIZATION_ROADMAP.md`
- **Implementation implications**: performance measurement begins with the first implementation wave; crossover experiments are mandatory design inputs.
- **Formal/pack/replay implications**: pack-owner obligations, counter schemas, replay/equivalence evidence.
- **Handoff implications**: none.

### RW-014: Lean-Facing Semantic Model And Theorem Backlog
- **Source lineage**: Foundation February formal-model drafts; March 5-9 promotion cycle.
- **Current OxCalc state**: mentioned only lightly in deferred kickoff notes.
- **Rewrite disposition**: `promote_baseline`
- **Target canonical document**: `CORE_ENGINE_FORMALIZATION_AND_ASSURANCE.md`
- **Implementation implications**: Lean formalization is planned as part of the core-engine architecture, not a detached future note.
- **Formal/pack/replay implications**: theorem backlog for replay determinism, structural rewrite totality, no hidden mutation, commit atomicity.
- **Handoff implications**: none.

### RW-015: TLA+ Model Of Coordinator, Fences, And Overlay GC Safety
- **Source lineage**: project doctrine plus the March 8-9 concurrency and publication synthesis.
- **Current OxCalc state**: implied but not explicitly carried as a core-engine-local requirement.
- **Rewrite disposition**: `promote_baseline`
- **Target canonical document**: `CORE_ENGINE_FORMALIZATION_AND_ASSURANCE.md`
- **Implementation implications**: TLA+ modeling becomes part of the intended design and verification route for async/concurrent behavior.
- **Formal/pack/replay implications**: model-check obligations for commit fences, publish order, session conflicts, pinned-epoch GC safety.
- **Handoff implications**: none.

### RW-016: Roslyn-Inspired Persistence Principles And Structure Design Space
- **Source lineage**: Foundation February formal-model archive and persistence-plan notes.
- **Current OxCalc state**: only the high-level green/red idea is retained.
- **Rewrite disposition**: `promote_baseline`
- **Target canonical document**: `CORE_ENGINE_STATE_AND_SNAPSHOTS.md`
- **Implementation implications**: the spec should explicitly adopt immutable-core/facade principles and preserve the most relevant structure-design guidance.
- **Formal/pack/replay implications**: stable-identity and no-hidden-mutation contracts.
- **Handoff implications**: none.

### RW-017: Concrete Grid Persistence Strategy Alternatives
- **Source lineage**: Foundation February persistence-plan archive.
- **Current OxCalc state**: not represented in OxCalc.
- **Rewrite disposition**: `retain_deferred`
- **Target canonical document**: `CORE_ENGINE_STATE_AND_SNAPSHOTS.md` and archive notes.
- **Implementation implications**: retain as later grid-phase design material; do not let grid-storage choice block TreeCalc-first execution.
- **Formal/pack/replay implications**: later-phase only.
- **Handoff implications**: none.

### RW-018: Grid/Spill-Heavy Future Semantics In The TreeCalc Rewrite
- **Source lineage**: existing bootstrap model plus March 8 synthesis.
- **Current OxCalc state**: present in the current consolidated doc because the model still speaks in hybrid terms.
- **Rewrite disposition**: `promote_staged_later`
- **Target canonical document**: `CORE_ENGINE_REALIZATION_ROADMAP.md`
- **Implementation implications**: keep future grid/spill material explicit but move it out of the TreeCalc baseline architecture sections.
- **Formal/pack/replay implications**: later grid-phase obligations only.
- **Handoff implications**: possible future seam work.

### RW-019: Over-Normative Design Draft Details From March 8 Reviews
- **Source lineage**: March 8 prompt-pack drafts and review critiques.
- **Current OxCalc state**: some details were already intentionally compressed by synthesis.
- **Rewrite disposition**: `archive_superseded`
- **Target canonical document**: none; retain only in the archive/promotion rationale.
- **Implementation implications**: do not promote speculative specifics merely because they were once written down.
- **Formal/pack/replay implications**: none directly.
- **Handoff implications**: none.

## 4. Status
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - bootstrap archive execution,
  - first canonical architecture draft,
  - shared-seam handoff packet identification
