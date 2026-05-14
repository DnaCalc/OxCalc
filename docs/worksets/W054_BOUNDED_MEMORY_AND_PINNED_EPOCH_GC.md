# W054 Bounded-Memory And Pinned-Epoch GC

Status: `pre_planning`

Parent predecessors: `W050` (the artefact set must exist to measure) and `W051` (sparse-reader artefacts complete the artefact set)

Parent epic: TBD (allocated when W054 is activated)

## 1. Purpose

W054 specifies operational memory discipline for the artefact and overlay surfaces the W050 rework introduces — the compiled-artefact / plan-template cache, runtime overlays, the per-edge differential-evaluation value cache, Subscription Registry topic envelopes, and pinned reader views. Without an explicit eviction discipline, every one of these grows unboundedly across recalc waves.

W054 makes the bounded-memory contract part of the spec rather than an implementation detail. It implements:

1. A profile-declared retention class per cache surface — `Required`, `Best-Effort`, `Discardable`.
2. A pinned-epoch protection rule — active session, stabilisation window, observer-pinned — generalising Foundation's overlay-eviction discipline (`CORE_ENGINE_FORMAL_MODEL` §6.3) from overlays to every W050 cache surface.
3. Deterministic eviction order: given the same operation history and the same retention claims, two engines evict in the same order, and the eviction trace is part of replay conformance.

W054 is in a deliberate `pre_planning` state. Scope, beads, exit gates, and evidence policy are decided after W050 lands the artefact set and W051 completes it with sparse-reader artefacts. This document is pre-planning background only; do not infer a bead path or commit to artefacts from it.

## 2. Pre-Planning Background

### 2.1 Why W054 follows W050 and W051, not precedes them

The eviction *policy* cannot be specified honestly until the artefact set exists and retention costs can be measured. Specifying eviction thresholds before W050's plan-template cache, overlays, per-edge value cache, and topic envelopes exist — and before W051's sparse-reader artefacts complete the set — would be guessing. W054 requires post-W050 measurement infrastructure: artefact retention costs, overlay residency, pin-epoch distance histograms.

### 2.2 Why W054 precedes W053

W054 nails deterministic eviction in the simpler Stage-1 *sequential* setting. Doing this before W053 means the eviction discipline is settled in the setting with one evaluator and one publisher, before W053 adds the complication of partitioned and speculative evaluators. W053 then revisits the W054 retention model to add the speculative-candidate retention class — an extension over a settled base, not a co-design.

### 2.3 Replay-deterministic, not just replay-friendly

The distinguishing requirement is deterministic eviction *order*. A spec that does not pin eviction order produces replay artefacts that drift across implementations even when published results agree — a `replay-friendly` engine but not a `replay-deterministic` one. W054's eviction trace is a replay-conformance obligation: replay validates that two implementations, given the same operation history and retention claims, evicted in the same order.

### 2.4 Surfaces W054 governs

- the compiled-artefact / plan-template cache (W050 Lane C),
- runtime overlays and the overlay lifecycle (`OverlayKey` / `OverlayEntry`, W050 Lane B),
- the per-edge differential-evaluation value cache (W050 Lane F, Move B),
- Subscription Registry topic envelopes (W050 Lane D),
- pinned reader views (W050 Lane B),
- `SparseRangeReader` artefacts and any sparse backing residency (W051).

## 3. Relationship To W050, W051, W053

- W050: introduces the artefact and overlay surfaces W054 governs; Foundation `CORE_ENGINE_FORMAL_MODEL` §6.3 and §6.8 (overlay eviction is deterministic and epoch-safe; overlay lifecycle baseline) is the doctrinal seed W054 generalises.
- W051: completes the artefact set with `SparseRangeReader` artefacts, which W054's retention model must cover — hence W051 precedes W054.
- W053: extends the W054 retention model for partitioned and speculative evaluators; W054's deterministic-eviction discipline is the base W053 builds on.

## 4. Open Scoping Questions

Deferred until W050 and W051 land and W054 is planned in detail:

- What measurement infrastructure must land first — and is that a W054 lane or a W050 cross-cutting item?
- How are retention classes assigned per cache surface — fixed in the spec, or profile-declared and tunable?
- What is the eviction-order tie-break rule that makes two implementations agree?
- How is the eviction trace represented in the replay bundle, and what does conformance check?
- Does W054 cover memory-pressure backpressure (refuse new artefacts under pressure) or only eviction?

## 5. Status Surface

- execution_state: `pre_planning`
- scope_completeness: `scope_partial`
- target_completeness: `target_partial`
- integration_completeness: `partial`
- prerequisites: W050 (the artefact set must exist so retention costs are measurable), W051 (sparse-reader artefacts complete the artefact set)
- bead_path: not yet specified — W054 epic id and bead structure allocated when W054 is activated
- exit_gate: not yet specified — requires post-W050 measurement infrastructure before the eviction policy can be specified honestly
- evidence_policy: not yet specified
- upstream_dependencies: none planned at activation; to be re-evaluated when the W054 plan is finalised
