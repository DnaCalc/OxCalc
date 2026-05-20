# W054 Bounded-Memory And Pinned-Epoch GC

Status: `queued_successor`

Parent predecessors:
- `W050` formula-authority rework
- `W051` sparse range readers

Parent epic: allocate when W054 starts.

## 1. Purpose

W054 adds deterministic memory discipline for the artifacts introduced by W050
and W051.

Without this work, caches and overlays can grow without a replay-visible
retention rule.

W054 covers:

1. prepared formula and plan-template caches,
2. runtime overlays,
3. per-edge value caches,
4. subscription/topic envelopes,
5. pinned reader views,
6. sparse-reader artifacts from W051.

## 2. Product Scope

The product requirement is simple: the engine must know what it may retain,
what it may evict, and in what order.

W054 defines:

1. retention classes: `Required`, `Best-Effort`, `Discardable`,
2. pinned-epoch protection: active session, stabilization window,
   observer-pinned,
3. deterministic eviction order,
4. replay-visible eviction traces.

## 3. First Work

The first W054 beads should:

1. add or reuse measurement counters for artifact residency,
2. list every cache/overlay surface W054 owns,
3. assign first retention classes,
4. define pin and unpin rules,
5. define eviction ordering and tie-breaks,
6. emit a replay-visible eviction trace,
7. run a bounded-memory scenario that proves deterministic eviction.

Suggested artifact root:

`docs/test-runs/core-engine/w054-bounded-memory/`

The first rollout bead may refine that root before emitting checked evidence.

## 4. Relationship To W053

W054 is Stage-1 memory discipline. W053 later extends it for partitioned and
speculative evaluators.

Do not mix Stage-2 speculative retention into W054 unless the W054 scope is
explicitly widened.

## 5. Closure Gate

W054 can close its first scope when:

1. every declared cache/overlay surface has a retention class,
2. pinned-epoch protection is implemented or explicitly blocked,
3. eviction order is deterministic,
4. replay records and validates eviction decisions,
5. memory counters show the bounded scenario behaved as expected,
6. W053-only speculative retention is routed forward.

## 6. Status

Product status: queued successor work. No bounded-memory product claim yet.

Evidence: W050 defines the main artifact set. W051 must add sparse-reader
artifacts before W054 finalizes the surface list.

Still open: counters, retention classes, pin rules, eviction order, replay
trace, and bounded-memory run.

Formal status: no proof claim.
