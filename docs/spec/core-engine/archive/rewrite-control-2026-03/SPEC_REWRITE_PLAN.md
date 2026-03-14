# OxCalc Core-Engine Spec Rewrite Plan

## 1. Purpose
This document defines the rewrite program for the OxCalc core-engine spec set.

This is not a small edit pass. It is a deliberate recollation and replacement of the
bootstrap OxCalc spec set so that OxCalc owns a coherent, extensive, and execution-ready
design framework for the core engine.

The rewritten set must carry forward the strongest outcomes from the Foundation
March 5-9, 2026 research, review, synthesis, and improvement cycle, together with
still-relevant earlier formal-model and persistence-structure work.

## 2. Rewrite Intent
The rewrite has four primary goals:

1. Replace the current bootstrap-oriented OxCalc core-engine docs with an extensive
   OxCalc-owned architecture and realization framework.
2. Make the intended implementation path explicit for the first serious OxCalc engine:
   the tree-based, no-grid, DNA TreeCalc scope.
3. Bake in near-formal modeling from the start:
   - Lean-facing semantic structure and theorem targets,
   - TLA+ model-check targets for concurrent/asynchronous coordinator behavior,
   - replay/trace/pack obligations for all promoted claims.
4. Preserve the best high-value implementation ideas from the March 5-9 design cycle
   without reopening the full design space as if nothing had been decided.

## 3. Architectural Pillars To Lock Into The Rewrite

### 3.1 Immutable Structural Truth
The rewritten architecture must treat immutable structural truth as foundational:

1. Structural state is immutable and versioned.
2. Stable identity is not derived from transient coordinates or traversal position.
3. Formula structures are immutable and participate in the same broader discipline as
   Roslyn-style immutable trees and OxFml parse/bind artifacts.
4. All runtime state hangs from immutable structure rather than mutating structural truth.

### 3.2 Versioned Runtime and MVCC-Derived Layers
The rewritten architecture must treat runtime state as secondary, versioned, and derived:

1. Epochs, pinned snapshots, and publication fences are mandatory.
2. Runtime overlays, caches, invalidation state, and observer-facing views must be
   defined as epoch-aware state attached to immutable structural snapshots.
3. A stable structural and calculation view must remain observable even while later
   recalculation work is ongoing.

### 3.3 Staged Concurrent and Async Coordinator
The rewritten architecture must be concurrency-ready from the start:

1. Stage 1 remains a sequential single-publisher baseline.
2. Stage 2+ concurrency and asynchronous evaluation are designed into the coordinator,
   publication model, and replay obligations from the start.
3. Multithreaded and async recalculation are not treated as late bolt-ons.
4. Testing, deterministic replay, and performance instrumentation for concurrent behavior
   are planned from the beginning.

### 3.4 Near-Formal Core-Engine Modeling
The rewritten architecture must include near-formal modeling as a first-class concern:

1. Lean-facing semantic structure and theorem targets must be named explicitly.
2. TLA+ targets must be named explicitly for concurrent and async coordinator behavior.
3. Architectural claims must map to proof obligations, model checks, replay artifacts,
   and empirical packs.
4. The spec set must define what is intended to be formally modeled, what is deferred,
   and what empirical evidence is required for promotion.

## 4. Scope Of The Rewrite

### 4.1 In Scope
1. OxCalc-owned core-engine architecture and realization docs.
2. TreeCalc-first engine scope:
   - tree-based substrate,
   - no grid,
   - no hidden grid assumptions in the baseline architecture.
3. Coordinator/publication/epoch/overlay/invalidation model.
4. OxCalc's side of the OxFml evaluator seam.
5. Near-formal structure:
   - Lean model boundaries,
   - TLA+ model boundaries,
   - pack/replay/proof mapping.
6. Rewrite planning for later grid introduction as a major later phase.

### 4.2 Out Of Scope
1. Rewriting OxFml-owned canonical FEC/F3E specs in place.
2. Treating the Foundation mirror/snapshot files as editable source-of-truth.
3. Treating all March 5-9 options as still-open design space.
4. Prematurely locking later grid-phase semantics that depend on later work.

## 5. First Executable Target
The first serious OxCalc target is the engine needed for DNA TreeCalc.

That target is characterized by:

1. tree-based structure,
2. no grid substrate,
3. some tweaked reference syntax relative to Excel as needed by the tree host,
4. deterministic single-publisher coordinator baseline,
5. explicit OxFml seam discipline,
6. incremental-engine ambition from the start,
7. later grid introduction as a major phase rather than a hidden assumption.

## 6. Rewrite Source Corpus
The rewrite must be driven by a promotion ledger built from these source families.

### 6.1 Primary Promotion Corpus
1. `../Foundation/research/runs/20260305-201430-dag-computation-theory-deep-research-pass-01/*`
2. `../Foundation/synthesis/runs/20260307-223304-core-engine-dag-fec-synthesis-pass-01/*`
3. `../Foundation/prompts/runs/20260308-171858-core-engine-fec-f3e-deep-research-pack-01/*`
4. `../Foundation/prompts/runs/20260308-182605-core-engine-fec-f3e-dual-model-review-pass-01/*`
5. `../Foundation/prompts/runs/20260308-184205-core-engine-fec-f3e-dual-model-review-pass-02/*`
6. `../Foundation/synthesis/runs/20260308-213253-core-engine-fec-f3e-synthesis-pass-02/*`
7. `../Foundation/synthesis/runs/20260309-004109-improvement-notes-synthesis-pass-01/*`
8. `../Foundation/synthesis/runs/20260309-072109-core-engine-program-layout-synthesis-pass-01/*`

### 6.2 Earlier Inputs Retained Where Still Valuable
1. `../Foundation/notes/archive/formal-model/FORMAL_MODELS_IDEAS.md`
2. `../Foundation/notes/archive/formal-model/FORMAL_CORE_STATUS_AND_SUGGESTIONS_DRAFT.md`
3. `../Foundation/notes/archive/formal-model/FORMAL_MODEL_REMAINING_NOTES.md`

### 6.3 Existing OxCalc Bootstrap Inputs
1. `CORE_ENGINE_FORMAL_MODEL.md`
2. `CORE_ENGINE_THEORY_AND_ALTERNATIVE_PATHS.md`
3. `FOUNDATION_ARCHITECTURE_SNAPSHOT.md`
4. `FOUNDATION_OPERATIONS_SNAPSHOT.md`

## 7. Promotion Policy For This Rewrite
Before new source-of-truth drafting, every major idea or design clause must be classified.

Required classification states:

1. `promote_baseline`
2. `promote_staged_later`
3. `retain_deferred`
4. `archive_superseded`

For each promoted idea, the rewrite must also record:

1. source lineage,
2. why it is promoted now,
3. effect on implementation path,
4. formal/pack/replay consequences.

## 8. Decision Direction Already Agreed For The Rewrite
The rewrite should treat the following as locked direction unless later contradicted by
higher-precedence doctrine or a clearly documented design issue:

1. TreeCalc-first engine scope is the immediate OxCalc target.
2. Immutable structural truth plus versioned runtime/MVCC layers is foundational.
3. The architecture should be concurrency-ready and async-ready from the start.
4. Near-formal specification and verification is part of the core-engine architecture.
5. The March 5-9 cycle is the primary design source for the new framework.
6. We do not reopen the full option space as if no synthesis had happened.

## 9. Rewrite Work Sequence

### 9.1 Stage A - Freeze And Archive
1. Freeze the current bootstrap OxCalc-owned spec set.
2. Copy it into an archive location with explicit superseded-bootstrap labeling.
3. Keep mirror/snapshot files distinct from the new canonical OxCalc set.

### 9.2 Stage B - Promotion Ledger
1. Build a topic-by-topic promotion ledger from the source corpus.
2. Identify what is:
   - baseline TreeCalc architecture,
   - later-stage but already intended architecture,
   - genuinely deferred,
   - superseded.

### 9.3 Stage C - New Canonical Document Skeleton
1. Create the new document map.
2. Define responsibility boundaries for each document.
3. Eliminate overlap between top-level architecture, realization details, and archive notes.

### 9.4 Stage D - Rewrite Drafting
1. Draft top-level architecture first.
2. Draft state/snapshot/runtime model next.
3. Draft recalc/invalidation/incremental model next.
4. Draft coordinator/publication/concurrency model next.
5. Draft OxFml seam requirements next.
6. Draft formalization and promotion-gates docs next.

### 9.5 Stage E - Handoff And Follow-Ons
1. Identify shared-seam clauses that require OxFml handoff packets.
2. Register any required handoffs.
3. Update feature/workset planning to match the new spec set.

## 10. Required Rewrite Outputs
The rewrite program must produce:

1. archived bootstrap spec set,
2. promotion ledger,
3. new canonical document map,
4. rewritten top-level OxCalc architecture,
5. rewritten supporting core-engine docs,
6. explicit formal-model and pack-obligation mapping,
7. identified OxFml handoff requirements where needed.

## 11. Quality Bar For The New Spec Set
The rewritten set should be:

1. more explicit than the current OxCalc bootstrap set,
2. implementation-guiding, not only theory-summarizing,
3. architecture-rigorous, not vague about epochs/overlays/coordinator behavior,
4. TreeCalc-first in baseline scope,
5. near-formal in structure and obligations,
6. staged for later concurrency and grid work without hiding those concerns.

## 12. Open Drafting Questions To Resolve During The Rewrite
These are drafting-direction questions, not reasons to delay the rewrite:

1. exact wording of the Stage 1 incremental baseline:
   - dirty-closure only,
   - or verification-oriented hybrid baseline language with staged realization caveat.
2. how much spill/grid future material remains in core docs versus later-phase docs.
3. how much of the persistence-structure design space is retained in canonical docs versus archive notes.

## 13. Status
- execution_state: planned
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - promotion ledger construction,
  - bootstrap archive execution,
  - new canonical doc drafting,
  - OxFml seam handoff identification
