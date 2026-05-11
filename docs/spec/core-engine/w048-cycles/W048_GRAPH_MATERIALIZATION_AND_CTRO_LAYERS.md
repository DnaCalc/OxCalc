# W048 Graph Materialization And CTRO Layers

Status: `active_execution_workset`

## 1. Purpose

W048 requires explicit dependency graph artifacts, not only trace events. Graphs must be materialized in both directions and at every relevant layer:

1. structural graph derived from the model and bind artifacts;
2. published effective graph after accepted runtime-derived overlays;
3. candidate effective graph produced during a recalculation wave before publication.

The graph artifact is the shared substrate for cycle classification, invalidation, frontier repair, replay, Excel comparison, W048 formal artifacts, and later W049 successor consumption.

## 2. Current Rust Floor

Current known engine floor:

1. `src/oxcalc-core/src/dependency.rs` builds `DependencyGraph` with descriptors, `edges_by_owner`, `reverse_edges`, `cycle_groups`, and diagnostics.
2. Edges and descriptors are sorted deterministically.
3. `find_cycle_groups` classifies non-trivial SCCs and self-loops.
4. `derive_invalidation_closure` walks `reverse_edges` and can mark cycle members `CycleBlocked`.
5. `src/oxcalc-core/src/treecalc.rs` derives a deterministic formula order or returns `CycleDetected`.
6. `src/oxcalc-core/src/treecalc_runner.rs` emits a graph sidecar with descriptors, forward edges, diagnostics, and cycle groups.
7. `src/oxcalc-tracecalc/src/planner.rs` has direct dependencies, reverse dependencies, component ordering, and cycle groups for TraceCalc planning.

W048 first sidecar evidence:

1. `scripts/check-w048-materialized-graphs.py` now normalizes TreeCalc run artifacts into W048 graph-layer sidecars.
2. `docs/test-runs/core-engine/treecalc-local/w048-materialized-graph-001/` contains the first checked packet: 29 cases, 87 graph layers, 111 reverse-edge records, and zero checker errors.
3. Per-case `w048_materialized_graph_layers.json` files expose structural, published-effective, and candidate-effective layers with forward edges, materialized reverse edges, edge provenance, overlay delta slots, cycle-region slots, topological order/blocked reason, and stable graph hashes.
4. `W048_MATERIALIZED_GRAPH_SIDECAR_EVIDENCE.md` records the evidence summary, review commands, and current limits.

Remaining W048 gap:

1. The current checked TreeCalc corpus has no circular-reference fixture, so cycle-region records are schema-present but empty in this packet.
2. Non-empty cycle-region, release/re-entry, and no-overlay-commit evidence belongs to `calc-zci1.3`, `calc-zci1.6`, and `calc-zci1.7`.
3. Native Rust runner emission may later absorb the checker projection once the schema stabilizes; the current packet is a checked replay-sidecar projection over runner artifacts.

## 3. Graph Layers

### 3.1 `G_struct`

`G_struct` is the graph derivable from the structural snapshot and stable formula/bind artifacts before runtime-derived overlay effects.

Required properties:

1. stable under value-only recalculation;
2. rebuilt or revalidated under structural changes or bind changes;
3. contains static direct, relative-bound, unresolved, host-sensitive, dynamic-potential, capability-sensitive, and shape-topology descriptors;
4. records unresolved or potential edges as diagnostics/descriptors, not as hidden concrete edges.

### 3.2 `O_published`

`O_published` is the accepted overlay state from previous calculation waves.

Overlay entries may represent:

1. dynamic dependency activation;
2. dynamic dependency release;
3. dynamic dependency reclassification;
4. dynamic region/spill membership activation or release;
5. retained unresolved/potential facts that remain part of the effective graph basis.

### 3.3 `G_eff`

`G_eff = compose(G_struct, O_published)`.

This is the graph used as the published basis for invalidation, demanded work selection, and ordinary recalculation before the current wave produces new overlay facts.

### 3.4 `O_candidate`

`O_candidate` is the staged overlay effect set discovered during the current wave.

It is not committed until:

1. candidate graph classification accepts the graph under the active profile;
2. frontier/order repair accepts the work plan;
3. candidate result and overlay state can publish atomically.

### 3.5 `G_eff_candidate`

`G_eff_candidate = compose(G_struct, O_candidate)`, or equivalently the candidate successor to `G_eff` for the current wave.

This graph is mandatory for CTRO-created cycle detection. A cycle created only in `G_eff_candidate` is a real cycle region for policy purposes, with `cycle_source = candidate_overlay`.

## 4. Materialized Graph Artifact

Every graph layer artifact should have this shape:

```json
{
  "graph_id": "g-eff-candidate:<snapshot>:<wave>",
  "graph_layer": "candidate_effective",
  "basis": {
    "snapshot_id": 1,
    "published_overlay_epoch": 7,
    "candidate_wave_id": "wave-12",
    "profile_id": "stage1.non_iterative"
  },
  "nodes": [],
  "forward_edges": [],
  "reverse_edges": [],
  "edge_provenance": [],
  "cycle_regions": [],
  "topological_order": [],
  "blocked_reason": null,
  "graph_hash": "sha256:..."
}
```

The exact JSON field spelling can evolve during implementation, but these semantic objects must remain present. The first executable projection uses `schema_version = oxcalc.w048.materialized_graph_layers.v1` in per-case `w048_materialized_graph_layers.json` files.

## 5. Node Records

Required node fields:

1. `node_id`;
2. stable symbol/path when available;
3. node kind;
4. formula/bind artifact id;
5. current calc state;
6. published value hash or redacted value reference when value evidence is part of the packet;
7. demanded/necessary marker when the engine has an observer/target frontier.

Node records are not a replacement for the structural snapshot. They are a graph-facing projection of the snapshot plus runtime state needed for replay.

## 6. Edge Records

Required edge fields:

1. `edge_id`;
2. `owner_node_id`;
3. `target_node_id`;
4. `descriptor_id`;
5. dependency kind;
6. `edge_origin`;
7. overlay epoch/wave if overlay-derived;
8. dynamic carrier detail when relevant;
9. stability class: structural, published overlay, candidate overlay;
10. value-read eligibility under the active profile.

`edge_origin` values:

1. `structural_static`;
2. `structural_relative_bound`;
3. `published_dynamic_activation`;
4. `published_dynamic_release_shadow`;
5. `candidate_dynamic_activation`;
6. `candidate_dynamic_release`;
7. `candidate_dynamic_reclassification`;
8. `candidate_region_activation`;
9. `candidate_region_release`;
10. `host_declared`.

## 7. Reverse Edges

Reverse edges must be materialized, not only derived by a checker.

Required invariant:

1. Every forward edge `(owner -> target)` appears in `reverse_edges[target]`.
2. Every reverse edge entry has an identical forward edge counterpart.
3. Both directions use deterministic ordering.

This is already true in the in-memory `DependencyGraph`. W048 asks for the same fact to be a first-class replay-visible artifact for every graph layer.

## 8. Overlay Delta Records

Every CTRO graph transition should materialize delta records:

```json
{
  "overlay_delta_id": "delta:wave-12:C1",
  "owner_node_id": 30,
  "delta_kind": "activate_dependency",
  "previous_edge_id": null,
  "candidate_edge_id": "dep:30:12:dyn:C1",
  "carrier": {
    "kind": "dynamic_reference",
    "formula_token": "INDIRECT",
    "observed_target": "A1"
  },
  "provenance": "candidate_overlay",
  "wave_id": "wave-12"
}
```

Delta kinds:

1. `activate_dependency`;
2. `release_dependency`;
3. `reclassify_dependency`;
4. `activate_region`;
5. `release_region`;
6. `unsupported_or_unresolved`;
7. `cycle_introduction`;
8. `cycle_release`.

`cycle_introduction` and `cycle_release` are not replacements for SCC classification. They are summary delta classifications derived from the materialized graph transition.

## 9. Cycle Region Records

Required fields:

1. `cycle_region_id`;
2. `graph_id`;
3. `cycle_source`;
4. sorted `members`;
5. `member_order`;
6. `cycle_root`;
7. `root_policy`;
8. internal edges;
9. incoming boundary edges;
10. outgoing boundary edges;
11. `introduced_by_overlay_delta_ids`;
12. `released_from_cycle_region_id` when applicable;
13. `terminal_policy`;
14. `terminal_state`;
15. prior value basis for each member if relevant;
16. iteration summary if iterative profile is active.

For Stage 1 non-iterative profile, `terminal_state` is expected to route to `cycle_blocked` / `synthetic_cycle_reject` style behavior, with no accepted candidate overlay commit.

## 10. Topological And SCC Metadata

For acyclic graph layers:

1. emit `topological_order`;
2. emit tie-break policy;
3. emit ordering basis, such as stable node id order;
4. emit whether order was rebuilt, repaired locally, or reused.

For cyclic graph layers:

1. emit SCC groups;
2. emit condensation graph order where meaningful;
3. emit rejected/blocked cycle regions;
4. emit the policy that prevented ordinary acyclic evaluation.

## 11. Candidate Graph Publication Rule

The W048 Stage 1 candidate rule is:

1. evaluate against `G_eff` as the published basis;
2. stage `O_candidate` from runtime-derived observations;
3. materialize `G_eff_candidate`;
4. classify cycle regions over `G_eff_candidate`;
5. if active profile rejects a cycle, publish no candidate values and commit no candidate overlay;
6. retain `O_published` as the effective basis;
7. emit diagnostics and graph artifacts for the rejected candidate.

This rule keeps CTRO-created cycles from mutating the graph basis through a failed candidate.

The first sidecar packet projects candidate dependency-shape updates into `overlay_deltas` when current TreeCalc result artifacts expose them. It does not yet prove CTRO-created cycle rejection because the current corpus does not include a CTRO-created cycle fixture.

## 12. Release And Re-Entry Rule

A cycle release must be represented as a graph transition:

1. prior graph layer contains cycle region `C`;
2. new candidate graph layer does not contain `C` or an equivalent SCC;
3. released members and downstream dependents receive invalidation seeds;
4. the next accepted publication creates a new `O_published` basis;
5. diagnostics record the old cycle region and new acyclic order.

If the previous cycle was only present in a rejected candidate graph, release must compare against the last published graph plus the new candidate, not against an uncommitted overlay.

## 13. W048 Formal Artifact Targets

W048 should introduce formal definitions/models that can state and check:

1. reverse-edge converse for materialized graphs;
2. graph-layer composition determinism;
3. candidate overlay no-commit on cycle reject;
4. SCC/cycle-region classification over each layer;
5. no-under-invalidation after edge activation/release;
6. cycle release re-entry covers members and downstream dependents;
7. iterative profile determinism once W048 chooses an algorithm.

W049 may later deepen or reorganize these artifacts, but W048 owns the first formal cycle definitions aligned with the implementation and test corpus.

## 14. Status Surface

- execution_state: `in_progress`
- scope_completeness: `scope_partial`
- target_completeness: `target_partial`
- integration_completeness: `partial`
- open_lanes:
  - materialized graph sidecar realization: `calc-zci1.2`
  - TraceCalc/TreeCalc behavior fixtures: `calc-zci1.3`
  - W048 proof/model artifacts: `calc-zci1.5`
