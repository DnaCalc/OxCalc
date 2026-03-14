# OxCalc Core-Engine Spec Rewrite Document Map

## 1. Purpose
This document defines the target canonical structure for the rewritten OxCalc core-engine
spec set and the promotion-ledger framework used to populate it.

The goal is to replace the current bootstrap-oriented set with a document family that is:

1. OxCalc-owned,
2. TreeCalc-first,
3. architecture-extensive,
4. near-formal in orientation,
5. implementation-guiding.

## 2. Canonical Document Set (Target)

### 2.1 Top-Level Architecture
`CORE_ENGINE_ARCHITECTURE.md`

Purpose:
1. state the authoritative TreeCalc-first OxCalc architecture,
2. define the architectural pillars,
3. define the core layering and ownership boundaries,
4. define Stage 1 baseline versus staged-later architecture.

Must cover:
1. immutable structural truth,
2. versioned runtime/MVCC layers,
3. staged coordinator and publication authority,
4. near-formal assurance framing,
5. OxCalc/OxFml/OxFunc boundaries.

### 2.2 Structural State And Snapshots
`CORE_ENGINE_STATE_AND_SNAPSHOTS.md`

Purpose:
1. define the immutable structural kernel,
2. define stable identity and versioning,
3. define snapshot shape and pinned-reader semantics,
4. define what runtime state is forbidden from mutating structural truth.

Must cover:
1. tree substrate baseline,
2. stable IDs and address/reference projection rules,
3. immutable structure plus facade/derived-view discipline,
4. epoch-bearing snapshots,
5. observer-visible stable state during in-flight recalculation.

### 2.3 Recalc And Incremental Model
`CORE_ENGINE_RECALC_AND_INCREMENTAL_MODEL.md`

Purpose:
1. define invalidation-state semantics,
2. define deterministic recalc flow,
3. define intended incremental model for TreeCalc,
4. define staged advanced optimization lanes.

Must cover:
1. dirty/stale/necessary/verified-or-equivalent state vocabulary,
2. topo/SCC baseline,
3. early-cutoff and verification policy,
4. dynamic dependency policy,
5. dynamic-topo and SAC-inspired promotion lanes,
6. required decisive experiments and crossover measurements.

### 2.4 Overlay And Derived Runtime Model
`CORE_ENGINE_OVERLAY_AND_DERIVED_RUNTIME.md`

Purpose:
1. define runtime overlays as explicit derived state,
2. define lifecycle, keying, retention, and GC,
3. define dynamic-dependency/runtime edge behavior.

Must cover:
1. overlay taxonomy,
2. overlay creation/retain/evict rules,
3. epoch-safe retention and GC,
4. dynamic-reference handling,
5. later-phase grid/spill-related overlay boundary notes where necessary.

### 2.5 Coordinator, Publication, And Concurrency
`CORE_ENGINE_COORDINATOR_AND_PUBLICATION.md`

Purpose:
1. define the coordinator as the single publication authority,
2. define commit/reject/publication semantics,
3. define staged concurrency and async behavior,
4. define TLA+-relevant safety and liveness contracts.

Must cover:
1. sequential Stage 1 baseline,
2. snapshot/token/capability fences,
3. accepted-commit atomicity,
4. reject-is-no-publish rule,
5. stable observer visibility during in-flight work,
6. Stage 2+ concurrent/async progression.

### 2.6 OxFml Seam Requirements
`CORE_ENGINE_OXFML_SEAM.md`

Purpose:
1. define OxCalc's coordinator-facing seam requirements,
2. clarify what OxCalc requires from OxFml and what remains OxFml-owned,
3. identify shared-clause handoff needs.

Must cover:
1. session/open/execute/commit expectations,
2. topology/value/shape/publication responsibilities,
3. rejection-detail requirements,
4. snapshot-fence and capability-fence implications,
5. handoff-required clauses.

### 2.7 Formal Models And Assurance Mapping
`CORE_ENGINE_FORMALIZATION_AND_ASSURANCE.md`

Purpose:
1. define the near-formal modeling plan,
2. map architecture to Lean/TLA+/pack/replay obligations,
3. define what is intended to be formally checked and how.

Must cover:
1. Lean-oriented semantic model boundaries,
2. theorem backlog,
3. TLA+ model boundaries for coordinator/concurrency/GC,
4. replay artifact requirements,
5. pack mapping and empirical gate mapping.

### 2.8 Roadmap And Promotion Gates
`CORE_ENGINE_REALIZATION_ROADMAP.md`

Purpose:
1. define the staged implementation plan,
2. define promotion gates,
3. define performance and experiment requirements from the start.

Must cover:
1. TreeCalc baseline,
2. later concurrency increments,
3. later grid introduction boundary,
4. decisive experiments,
5. performance instrumentation expectations,
6. gate criteria for staged promotion.

## 3. Archive Structure (Target)
The existing bootstrap set should move under an explicit archive path, for example:

`docs/spec/core-engine/archive/bootstrap-2026-03/`

Archive contents should preserve:
1. original filenames where practical,
2. explicit superseded-bootstrap banner,
3. provenance note explaining replacement by the new canonical set.

## 4. Promotion Ledger Framework
Before or during drafting, each major topic should be recorded in a promotion ledger.

Suggested canonical location:
`docs/spec/core-engine/REWRITE_PROMOTION_LEDGER.md`

Each entry should include:

1. topic id,
2. topic name,
3. source lineage,
4. current OxCalc state,
5. rewrite disposition:
   - `promote_baseline`
   - `promote_staged_later`
   - `retain_deferred`
   - `archive_superseded`
6. target canonical document,
7. implementation implications,
8. formal/pack/replay implications,
9. OxFml/Foundation handoff implications if any.

## 5. Topic Families For The Promotion Ledger
At minimum, the ledger should cover these topic families:

1. immutable structure and snapshot model,
2. stable identity and addressing/reference projection,
3. runtime overlay taxonomy and lifecycle,
4. invalidation-state model,
5. deterministic recalc baseline,
6. early cutoff / verification semantics,
7. dynamic dependency handling,
8. dynamic-topo maintenance policy,
9. coordinator publication and reject semantics,
10. observer-visible stable-state semantics during ongoing recalc,
11. Stage 2+ concurrency and async progression,
12. OxFml seam contracts,
13. formal model and theorem backlog,
14. TLA+ concurrency/GC model targets,
15. performance instrumentation and decisive experiments,
16. future grid-phase boundary notes.

## 6. Drafting Order
Recommended drafting order:

1. `REWRITE_PROMOTION_LEDGER.md`
2. `CORE_ENGINE_ARCHITECTURE.md`
3. `CORE_ENGINE_STATE_AND_SNAPSHOTS.md`
4. `CORE_ENGINE_RECALC_AND_INCREMENTAL_MODEL.md`
5. `CORE_ENGINE_OVERLAY_AND_DERIVED_RUNTIME.md`
6. `CORE_ENGINE_COORDINATOR_AND_PUBLICATION.md`
7. `CORE_ENGINE_OXFML_SEAM.md`
8. `CORE_ENGINE_FORMALIZATION_AND_ASSURANCE.md`
9. `CORE_ENGINE_REALIZATION_ROADMAP.md`
10. archive bootstrap set and mark old docs superseded if not already done

## 7. Notes On Existing Bootstrap Docs
The current docs should be treated as bootstrap inputs rather than the final target shape:

1. `CORE_ENGINE_FORMAL_MODEL.md`
   - contains useful consolidated baseline language,
   - too compressed for the new role,
   - should likely be superseded rather than endlessly expanded.
2. `CORE_ENGINE_THEORY_AND_ALTERNATIVE_PATHS.md`
   - useful as theory/context input,
   - likely partly absorbed into architecture and roadmap docs,
   - remainder may move to archive or a reduced supporting note.
3. `FOUNDATION_ARCHITECTURE_SNAPSHOT.md`
4. `FOUNDATION_OPERATIONS_SNAPSHOT.md`
   - remain snapshots/reference support,
   - should not compete with new OxCalc canonical docs.

## 8. Status
- execution_state: planned
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - promotion ledger drafting,
  - archive path creation,
  - canonical-doc drafting start,
  - handoff identification for shared seam clauses
