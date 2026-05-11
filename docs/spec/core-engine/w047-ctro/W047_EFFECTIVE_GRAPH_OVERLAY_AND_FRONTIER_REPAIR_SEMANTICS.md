# W047 Effective Graph Overlay And Frontier Repair Semantics

Status: `calc-aylq.2_semantics_validated`

Parent workset: `W047 Calc-Time Rebinding Overlay Design Sweep`

Parent bead: `calc-aylq.2`

Predecessor gate: `calc-aylq.1` crosswalk at `W047_HISTORICAL_NO_LOSS_CTRO_CROSSWALK.md`

## 1. Purpose

This packet specifies the design-level semantics for Calc-Time Rebinding Overlay effective graphs and frontier repair. It is the W047 bridge between the historical crosswalk and later implementation/scenario work.

The scope is intentionally narrow:

1. define the state vocabulary for structural graphs, published overlays, candidate overlays, and effective graphs;
2. define pre/post conditions for effective graph composition, dependency release/activation, region/spill resize, frontier repair, fallback/reject, and publication;
3. define invariants that later Rust implementation, TraceCalc scenarios, and W049 formal/checker work must preserve;
4. record exact W049 targets and blockers without adding new Lean, TLA, checker, or sidecar artifacts under W047.

This packet does not claim runtime behavior is exercised. Runtime evidence belongs to later W047 implementation/scenario beads.

## 2. State Vocabulary

| Symbol | Meaning | Mutability |
| --- | --- | --- |
| `S` | Immutable structural snapshot: nodes, formulas, names, structural bindings, and host-provided static dependency descriptors. | Immutable during the calculation attempt. |
| `G_struct` | Dependency graph derived only from `S`: nodes, structural edges, reverse edges, diagnostics, SCC/cycle groups. | Rebuilt only through structural/model change. |
| `O_published` | Accepted runtime overlay from prior publications, keyed by structural snapshot and compatibility basis. | Stable input to a calculation attempt. |
| `G_eff` | Current effective graph, `compose(S, G_struct, O_published)`. | Derived, not separately mutable. |
| `Frontier` | Needed/evaluating/blocked/clean region derived from invalidation over `G_eff`. | Repaired only through explicit transition. |
| `EvalResult` | Formula-evaluation result: value, observed reads, runtime effects, diagnostics, and reject/fallback signals. | Candidate-local. |
| `O_candidate` | Runtime overlay staged from accepted candidate facts in the current calculation attempt. | Candidate-local until publication. |
| `G_eff_next` | Provisional effective graph, `compose(S, G_struct, O_candidate)`, used for cycle classification and frontier repair before publication. | Derived, candidate-local. |
| `RejectRecord` | Structured no-publish record with reason, affected nodes, overlay deltas, cycle groups, and fallback diagnostics. | Published as diagnostics only. |
| `PublicationBundle` | Atomic publication unit containing values plus accepted overlay consequences. | Commit-scoped. |

Dependency edges use the normal graph direction: formula owner depends on precedent. Reverse edges are derived for invalidation.

## 3. Overlay Delta Taxonomy

`O_candidate` is not an arbitrary replacement for structural truth. It contains typed deltas relative to `O_published`.

| Delta kind | Meaning | Required data |
| --- | --- | --- |
| `activate_dependency` | Runtime evaluation observed a dependency edge that was not present in `O_published` for the same owner/basis. | owner node, precedent node or region, effect source, compatibility key. |
| `release_dependency` | Runtime evaluation no longer needs a previously published runtime dependency for the same owner/basis. | owner node, released precedent or region, prior overlay key, release cause. |
| `reclassify_dependency` | A dependency remains but changes class, such as dynamic-to-static-equivalent or capability-sensitive-to-normal. | old class, new class, owner, precedent, basis. |
| `activate_region` | Runtime shape/spill evaluation exposes new region members as dependencies or produced outputs. | owner, region identity, member set/range, shape basis. |
| `release_region` | Runtime shape/spill evaluation removes previously published region members. | owner, prior region identity, released members, shape basis. |
| `unsupported_or_unresolved` | Runtime facts are insufficient to build safe deltas. | owner, missing fact, fallback or reject reason. |
| `cycle_introduction` | `G_eff_next` SCC classification finds a cycle involving candidate overlay consequences. | cycle group, participating structural and overlay edges. |

## 4. Transition Semantics

### 4.1 Build Effective Graph

Preconditions:

1. `S` is fixed for the calculation attempt.
2. `G_struct` was built from `S`.
3. Every retained entry in `O_published` has a compatibility key accepted for `S`.
4. Rejected or fallback-only diagnostic overlays are not included in `O_published`.

Postconditions:

1. `G_eff` contains every node and structural edge from `G_struct`.
2. `G_eff` adds active dependency and region/spill consequences from compatible `O_published`.
3. Released or incompatible overlay entries are excluded.
4. Reverse edges are derived from the composed forward edges.
5. SCC/cycle groups are classified over the composed graph.
6. `S` and `G_struct` are unchanged.

### 4.2 Derive Initial Invalidation

Preconditions:

1. `G_eff` exists with forward and reverse edges.
2. Invalidation seeds are structural edits, value edits, host-invalidated nodes, volatile nodes, or explicit rebind/recalc seeds.
3. Cycle groups from `G_eff` are available.

Postconditions:

1. The invalidation closure includes each seed and every effective-graph dependent reachable through reverse edges.
2. Nodes in cycle groups that intersect the needed region are marked through the profile-governed cycle policy.
3. In the current non-iterative Stage 1 profile, cycle-region nodes become blocked rather than eligible for ordinary acyclic evaluation.
4. No node is marked clean merely because an overlay release might remove a future dependency; release takes effect only after accepted publication or explicit repair.

### 4.3 Evaluate And Stage Candidate Overlay

Preconditions:

1. The node is in the acyclic needed frontier for `G_eff`.
2. All required precedents under the current order policy are either clean, published, or otherwise available under the active profile.
3. The evaluator reports observed dynamic dependencies, region/spill facts, runtime effects, or structured insufficiency where the formula can change dependency shape.

Postconditions:

1. Value facts are staged as candidate value deltas.
2. Runtime dependency and shape facts are staged in `O_candidate`, not committed to `O_published`.
3. Unsupported or unresolved dynamic facts produce structured `unsupported_or_unresolved` deltas and force reject or fallback before publication.
4. If the evaluator cannot provide enough facts to distinguish value-only recalc from dependency-shape change, the safe result is reject/fallback, not silent publication.

### 4.4 Classify Overlay Deltas

Preconditions:

1. `O_published` and `O_candidate` are available for the same `S`.
2. Overlay keys include owner, overlay kind, structural snapshot or compatibility basis, and payload identity sufficient for deterministic comparison.

Postconditions:

1. Every candidate overlay consequence is classified as activation, release, reclassification, region/spill resize, unchanged, unsupported/unresolved, or cycle-relevant after graph composition.
2. Release never deletes structural edges.
3. Activation never bypasses compatibility checking.
4. Delta classification is deterministic under stable node, edge, and overlay-key ordering.

### 4.5 Compose Candidate Effective Graph And Classify Cycles

Preconditions:

1. `O_candidate` contains only candidate-local overlay consequences for the same structural snapshot basis.
2. Unsupported/unresolved deltas have already been routed to reject/fallback.

Postconditions:

1. `G_eff_next = compose(S, G_struct, O_candidate)`.
2. The same SCC/cycle classifier used for structural graph paths runs over `G_eff_next`.
3. Candidate-created cycles and pre-existing cycles are reported as cycle groups with provenance on participating structural and overlay edges.
4. In the current non-iterative Stage 1 profile, any cycle group in the candidate-needed region blocks ordinary publication and yields `CycleBlocked` / `SyntheticCycleReject`.
5. `O_published` remains unchanged if cycle classification rejects the candidate.

### 4.6 Repair Frontier And Order

Preconditions:

1. `G_eff_next` has no unhandled cycle group in the candidate-needed region.
2. Overlay deltas have been classified.
3. The implementation can recompute reverse edges, impacted region, and deterministic order for the affected graph slice.

Postconditions:

1. Frontier repair invalidates every node whose value may differ because of activation, release, region/spill resize, or reclassification.
2. Activation of a new precedent makes the owner and its downstream dependents needed unless the owner was already evaluated against that exact precedent value in the candidate attempt.
3. Release of an old precedent does not erase already-needed downstream work unless the repaired frontier proves the released edge can no longer affect the current publication candidate.
4. Region/spill expansion adds newly exposed members and their affected dependents to the needed or produced-output region before publication.
5. Region/spill contraction records released outputs and stale consumers explicitly; it does not leave consumers silently clean.
6. If deterministic local repair cannot be proven for the affected slice, the transition routes to conservative fallback or reject.
7. Repaired order is stable under deterministic node-id and edge ordering.

### 4.7 Fallback Or Reject

Preconditions:

1. Some required CTRO fact is unsupported, unresolved, incompatible, cycle-blocked, or not safely repairable.
2. A structured reason and affected node set can be produced.

Postconditions:

1. No candidate value publishes.
2. No candidate overlay commits into `O_published`.
3. Previous `O_published` remains the effective runtime overlay for later calculations unless invalidated by an explicit accepted publication or structural change.
4. Diagnostics may retain `O_candidate` facts as reject evidence only.
5. Fallback is replay-visible and distinguishes unsupported repair from proved cycle rejection.

### 4.8 Atomic Publication

Preconditions:

1. Candidate values were evaluated against `G_eff_next`.
2. Cycle classification and frontier repair accepted the candidate-needed region.
3. Publication fence, structural snapshot basis, overlay compatibility, and evaluator/candidate basis still match.
4. Reject/fallback conditions are absent.

Postconditions:

1. Value deltas and overlay consequences publish as one `PublicationBundle`.
2. `O_published` advances to the accepted overlay state.
3. Released overlay entries become unavailable for future invalidation except where pinned readers or replay retention require diagnostics.
4. New reverse edges derived from accepted overlay consequences govern future invalidation.
5. No reader sees a value publication without the corresponding overlay consequences.

## 5. Invariants

| Invariant | Statement | Primary owner |
| --- | --- | --- |
| structural immutability | CTRO never mutates `S` or treats runtime facts as structural graph truth. | core engine |
| effective graph determinism | Given `S`, `G_struct`, compatible overlay set, and deterministic ordering, composed `G_eff` is deterministic. | core engine |
| overlay compatibility | An overlay entry is usable only under its declared structural snapshot and compatibility basis. | core engine/coordinator |
| no under-invalidation | A repaired frontier must include every node whose value or publication eligibility may differ under the candidate overlay. | core engine |
| no silent release | Releasing a runtime dependency changes future graph shape only through accepted publication or explicit repair; it cannot silently mark affected work clean. | core engine |
| shared cycle policy | Structural cycles and CTRO-created cycles are classified by the same SCC/cycle machinery and profile policy. | graph layer |
| no overlay commit on reject | Candidate overlay consequences are not visible as published runtime graph truth after reject/fallback. | coordinator |
| atomic value-overlay publication | Published values and accepted overlay consequences become visible together. | coordinator |
| diagnostic provenance | Reject/fallback records preserve whether a dependency change was structural, published-overlay-derived, or candidate-overlay-derived. | trace/replay |
| retention safety | Accepted overlays remain available while active readers, replay witnesses, or pinned epochs require them. | coordinator/runtime |

## 6. Semantic-Equivalence Statement For Strategy Changes

W047 may later choose local frontier repair, conservative rebuild, or full fallback for a CTRO delta. For all non-iterative Stage 1 profiles, these strategies are semantically equivalent only when they produce the same observable outcome:

1. the same accepted value map for every published node;
2. the same accepted overlay consequences, modulo deterministic ordering of equivalent deltas;
3. the same cycle-blocked or reject/no-publish decision for every affected cycle group;
4. the same future invalidation graph for accepted publications;
5. replay-visible diagnostics distinguishing optimized repair from conservative fallback.

If local repair cannot prove that equivalence, W047 must use conservative fallback or reject rather than publish through the optimized path.

## 7. W049 Formal And Checker Targets Or Blockers

W047 records these as W049 targets. They are not W047 deliverables.

| Surface | Target property | W047 blocker |
| --- | --- | --- |
| Lean | Effective graph composition preserves structural edges and applies only compatible overlay consequences. | Need landed Rust CTRO state/types and non-vacuous property definitions tied to implementation behavior. |
| Lean | No-under-invalidation after activation, release, and region/spill resize. | Need concrete frontier-repair implementation and scenario artifacts. |
| Lean | Reject/fallback implies no candidate overlay publication. | Need accepted Rust publication/reject state model. |
| TLA | Candidate overlay, cycle classification, frontier repair, reject, and atomic publication transitions. | Need non-smoke state space and post-W047 transition vocabulary. |
| TLA | Strategy equivalence between local repair and conservative rebuild for acyclic CTRO deltas. | Need bounded but meaningful model shape and fixture-derived cases. |
| Checker | Trace facts for `G_struct`, `O_published`, `O_candidate`, `G_eff`, `G_eff_next`, delta classification, cycle groups, repair decision, reject/fallback, and publication bundle. | Need native trace/schema changes, which W047 explicitly routes to W049. |
| TraceCalc | Reference scenarios for dynamic target switch, unresolved target, downstream dependent, spill/region resize, structural cycle, CTRO-created cycle, cycle release, and fallback. | Need `calc-aylq.3` scenario matrix and available substrate support. |

Exact blocker for W047: adding Lean/TLA/checker/sidecar artifacts now would repeat the W046 shallow-evidence failure mode. W047 should instead land the implementation and scenario core, then W049 can formalize against that single authority.

## 8. Acceptance Mapping

| `calc-aylq.2` acceptance item | Evidence in this packet |
| --- | --- |
| Effective graph composition | Sections 2, 4.1, 4.5, and invariant table. |
| Dependency release/activation | Sections 3, 4.4, 4.6, and invariants `no under-invalidation` / `no silent release`. |
| Region/spill resize | Sections 3, 4.6, and W049 target table. |
| Frontier repair | Sections 4.6 and 6. |
| Fallback/reject | Section 4.7 and invariant `no overlay commit on reject`. |
| Publication | Section 4.8 and invariant `atomic value-overlay publication`. |
| Pre/post conditions | Section 4 transition subsections. |
| Invariants | Section 5. |
| Lean/TLA/checker targets or blockers | Section 7. |

## 9. Current Status

- execution_state: `calc-aylq.2_semantics_validated`
- scope_completeness: `scope_partial`
- target_completeness: `target_partial`
- integration_completeness: `partial`
- open_lanes:
  - `calc-aylq.3` scenario matrix and evidence plan
  - `calc-aylq.4` implementation remodeling and CTRO phase landing
  - `calc-aylq.7` dynamic dependency positive publication implementation refinement
  - W049 formal/checker/sidecar/readiness successor work
