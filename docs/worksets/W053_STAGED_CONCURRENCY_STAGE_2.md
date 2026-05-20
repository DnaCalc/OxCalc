# W053 Staged Concurrency Stage 2

Status: `queued_successor`

Parent predecessors:
- `W050` Stage-1 sequential coordinator on the current session model
- `W049` formalized Stage-1 baseline
- `W054` bounded-memory and pinned-epoch GC

Parent epic: allocate when W053 starts.

## 1. Purpose

W053 adds Stage-2 partitioned evaluation while keeping one Coordinator as the
only publisher.

The goal is speedup on multi-core hardware without changing worksheet-visible
results.

## 2. Product Scope

In scope:

1. partitioned evaluators over the dependency graph,
2. SCC/cycle isolation,
3. topological frontier scheduling,
4. speculative candidates with input-binding fingerprints,
5. commit-time fingerprint checks,
6. deterministic replay of contention and rejection,
7. equivalence evidence against the Stage-1 baseline.

Out of scope unless explicitly added:

1. Stage-3 advanced dynamic-topology policies,
2. multiple publishers,
3. distributed multi-host arbitration,
4. nondeterministic replay.

## 3. Invariant

Stage 2 may add evaluators. It may not add publishers.

Accepted publication still flows through one Coordinator. Rejected candidates
publish no derived deltas and must carry structured rejection detail.

## 4. First Work

The first W053 beads should:

1. choose the partition strategy,
2. define speculative candidate identity and fingerprint granularity,
3. define contention replay artifacts,
4. extend W054 retention rules for speculative candidates,
5. run differential Stage-1 versus Stage-2 scenarios,
6. state the exact semantic-equivalence claim.

## 5. Closure Gate

W053 can close its first Stage-2 scope when:

1. partitioned evaluation is implemented for the declared graph class,
2. contention and stale-fingerprint paths are exercised,
3. replay reproduces the same accepted/rejected outcomes,
4. observable results match the Stage-1 baseline,
5. memory retention for speculative candidates is deterministic,
6. Stage-3 behavior remains unpromoted unless separately evidenced.

## 6. Status

Product status: queued successor work. Stage 2 is not implemented or promoted.

Evidence: W050 provides the Stage-1 session/coordinator shape. W049 and W054
must provide the baseline and memory discipline before W053 executes.

Still open: all W053 lanes.

Formal status: no Stage-2 equivalence proof yet.
