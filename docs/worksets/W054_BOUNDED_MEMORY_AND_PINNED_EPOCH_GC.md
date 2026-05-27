# W054 Bounded-Memory And Pinned-Epoch GC

Status: `initial_slice_active`

Parent predecessors:
- `W050` formula-authority rework
- `W051` sparse range readers

Parent epic: allocate or promote from the first W054 bead when the next slice
needs multi-bead tracking.

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

## 3A. Initial Implementation Slice

The first W054 code slice is deliberately narrow and does not claim full
bounded-memory closure.

Implemented surface:

1. Per-edge value cache entries retain the existing
   `W054PendingEphemeralPerEdgeValueCache` class and bounded
   `max_entries` policy.
2. Per-edge value cache eviction now emits a replay-visible
   `EdgeValueCacheEvictionTraceRecord` with retention class, eviction reason,
   evicted cache key, and evicted insertion sequence.
3. The eviction tie-break is deterministic:
   `evicted_insertion_sequence_then_cache_key`.
4. The cache exposes an `eviction_count` counter and retained eviction trace.
5. TreeCalc diagnostics project the cache eviction trace through
   `edge_value_cache_eviction_trace:*`.
6. Coordinator reader lifecycle counters now distinguish `pin_count` and
   `unpin_count`; failed/unmatched unpin attempts do not increment
   `unpin_count`.

Evidence:

1. `per_edge_value_cache_eviction_is_bounded_oldest_first` proves the bounded
   oldest-first eviction order, counter increment, and in-memory trace record.
2. `checked_in_w054_per_edge_value_cache_eviction_trace_artifact_matches_runtime_validation`
   validates
   `docs/test-runs/core-engine/w054-bounded-memory/per-edge-value-cache-eviction-trace-001/run_artifact.json`.
3. `coordinator_counts_pin_and_unpin_reader_transitions` proves pin/unpin
   counters and idempotent unmatched unpin behavior.

Still open after this slice: prepared formula/plan-template retention classes,
runtime overlay retention, subscription/topic envelope retention,
sparse-reader artifact retention, pin protection across all retained surfaces,
and a full bounded-memory scenario across the W050/W051 artifact set.

## 4. Relationship To W053

W054 is Stage-1 memory discipline. W053 later extends it for partitioned and
speculative evaluators.

Do not mix Stage-2 speculative retention into W054 unless the W054 scope is
explicitly widened.

## 5. Relationship To W057

W057 owns the workspace revision and snapshot-layer representation rework.
W054 should not absorb that design work just because retention needs stable
identities.

W054 may close bounded-memory slices against the current implementation
surfaces when retention classes, pin rules, deterministic eviction, counters,
and replay traces are explicit. W057 later retargets those retention identities
onto `WorkspaceRevision`, `StructureSnapshot`, `NodeInputSnapshot`,
`NamespaceSnapshot`, `FormulaBindingSnapshot`, `DependencyShapeSnapshot`,
`PublicationSnapshot`, and `RuntimeOverlaySet` identities.

## 5A. W057 Retention Identity Map

W057.14 retargets W054 identity vocabulary without claiming full W054 closure.

The retention identity map is:

| Retained surface | Identity basis | First W054 class/status | Pin and eviction rule |
| --- | --- | --- | --- |
| workspace revision roots | `WorkspaceRevisionId`, `StructureSnapshotId`, `NodeInputSnapshotId`, `NamespaceSnapshotId` | required while current or pinned | retain while a session, publication, pinned reader, formula binding, dependency shape, cache, or overlay references the revision; evict only after all derived references are released |
| formula binding snapshot facts | `WorkspaceRevisionId`, `StructureSnapshotId`, `NamespaceSnapshotId`, `FormulaBindingSnapshotId` | open W054 class | retain across literal value edits when binding facts remain compatible; invalidate on formula text, namespace, registry, capability, or structure incompatibility |
| prepared formula artifacts and plan templates | `StructureSnapshotId`, `NamespaceSnapshotId`, formula catalog artifact-token basis, argument-preparation profile | open W054 class | retain across literal value edits; do not key directly by `WorkspaceRevisionId`, `NodeInputSnapshotId`, `PublicationSnapshotId`, or `RuntimeOverlaySetId` when those ids changed for reasons outside formula preparation |
| dependency graph and dependency-shape facts | `WorkspaceRevisionId`, `FormulaBindingSnapshotId`, `DependencyShapeSnapshotId` | open W054 class | retain while the graph shape is current or pinned; publish dependency-shape deltas atomically with value publication before evicting superseded shape facts |
| publication and pinned reader values | `WorkspaceRevisionId`, `PublicationSnapshotId`, and pinned structural view identity | required for active pinned readers | keep the publication snapshot and pinned structural/value view until the reader unpins; unmatched unpin attempts do not advance lifecycle counters |
| runtime overlays and CTRO effects | `PublicationSnapshotId`, `RuntimeOverlaySetId`, owner node, overlay kind, compatibility basis, payload identity | best-effort unless pinned or publication-required | retain only when the publication and compatibility basis remain compatible; release after unpin or superseding publication, then evict deterministically |
| per-edge value cache | `WorkspaceRevisionId`, `FormulaBindingSnapshotId`, `DependencyShapeSnapshotId`, `PublicationSnapshotId`, `RuntimeOverlaySetId`, call-site id, hole-binding fingerprint | `W054PendingEphemeralPerEdgeValueCache` | bounded `MaxEntriesOldestFirst`; bypass on upstream publication, external invalidation, dependency-shape delta, or explicit invalidation seed |
| sparse reader artifacts | `WorkspaceRevisionId`, `StructureSnapshotId`, `NodeInputSnapshotId`, `NamespaceSnapshotId`, relevant `PublicationSnapshotId` | open W051/W054 class | keep only for compatible sparse-reader basis and active reader windows; full eviction policy remains open |
| Stage-2 speculative candidates | Stage-2 candidate identity and fingerprint basis on top of the W054 identities | routed to W053 | not promoted in W054; W053 must define deterministic speculative retention before Stage 2 activation |

The first code retargeting is intentionally narrow:

1. optimized TreeCalc edge-value cache keys already embed an explicit
   `edge_value_cache_basis` carrying `WorkspaceRevisionId`,
   `FormulaBindingSnapshotId`, `DependencyShapeSnapshotId`,
   `PublicationSnapshotId`, and `RuntimeOverlaySetId`;
2. the local retention guardrail now emits a `retention_identity_basis` packet
   carrying all W057 root/layer ids and uses that basis for dynamic-overlay
   compatibility keys instead of the older `snapshot:9031` placeholder;
3. the same guardrail keeps its candidate artifact-token basis narrower than
   the runtime compatibility basis, using structure and namespace identity
   directly and excluding workspace revision, node-input, publication, and
   runtime-overlay ids from the artifact token;
4. W053 speculative retention remains marked `routed_forward_not_promoted`.

## 6. Closure Gate

W054 can close its first scope when:

1. every declared cache/overlay surface has a retention class,
2. pinned-epoch protection is implemented or explicitly blocked,
3. eviction order is deterministic,
4. replay records and validates eviction decisions,
5. memory counters show the bounded scenario behaved as expected,
6. W053-only speculative retention is routed forward.

## 7. Status

Product status: initial implementation slice active. Per-edge value-cache
eviction is bounded, deterministic, counted, and replay-visible; coordinator
reader pin/unpin lifecycle counters are explicit. W057.14 has retargeted the
first retention identity map and the local retention guardrail onto explicit
workspace revision and snapshot-layer ids. No full bounded-memory product claim
yet.

Evidence: W050 defines the main artifact set. The first checked W054 artifact is
`docs/test-runs/core-engine/w054-bounded-memory/per-edge-value-cache-eviction-trace-001/run_artifact.json`.
The optimized TreeCalc runner retention guardrail now validates that
`WorkspaceRevisionId`, `StructureSnapshotId`, `NodeInputSnapshotId`,
`NamespaceSnapshotId`, `FormulaBindingSnapshotId`, `DependencyShapeSnapshotId`,
`PublicationSnapshotId`, and `RuntimeOverlaySetId` are present in its retention
identity basis.

Still open: complete counters and retention classes for every W054 surface,
pin rules across those surfaces, overlay/cache eviction order beyond the
per-edge value cache and local guardrail, sparse-reader artifact retention, and
a full bounded-memory run.

Formal status: no proof claim.
