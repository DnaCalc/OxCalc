# W027: TreeCalc Dependency Graph and Invalidation Closure

## Purpose
Replace planner-only dependency derivation with real dependency graph build and invalidation closure over TreeCalc structure plus consumed OxFml bind facts.

## Position and Dependencies
- **Depends on**: W025, W026
- **Blocks**: W028, W029, W030, W031
- **Cross-repo**: none unless dependency consequence transport from OxFml proves narrower than currently consumed

## Scope
### In scope
1. static dependency edge derivation from real bind facts
2. reverse-edge and dependency identity realization
3. explicit cycle region representation for first TreeCalc-ready scope
4. invalidation state transitions tied to structure edits, upstream publication, and dependency consequences
5. deterministic dependency diagnostics or artifacts suitable for replay and witness binding
6. explicit distinction between rebind-required and recalc-only invalidation causes

### Out of scope
1. evaluator-backed candidate-result production
2. runtime-derived dynamic dependency overlay closure
3. final TreeCalc baseline runs
4. broader pack-grade replay promotion

## Deliverables
1. a real dependency graph build path from structure plus consumed bind products
2. replay-visible dependency identity and diagnostics for the first TreeCalc family subset
3. invalidation closure rules for structural edits and dependency changes
4. explicit cycle-region or blocked-state handling for the first phase

## Gate Model
### Entry gate
- W025 has provided the widened structural model
- W026 has locked the first consumed bind/reference package

### Exit gate
- structural dependency graph and reverse edges exist for the covered TreeCalc formula families
- dependency identity is deterministic and replay-visible
- invalidation closure is explicit for structure edits and dependency changes in phase scope

## Pre-Closure Verification Checklist
1. Spec text and realization notes updated for all in-scope items: no
2. Pack expectations updated for affected packs: no
3. At least one deterministic replay artifact exists per in-scope behavior: no
4. Semantic-equivalence statement provided for policy or strategy changes: no
5. FEC/F3E cross-repo impact assessed and handoff filed if needed: no
6. All required tests pass: no
7. No known semantic gaps remain in declared scope: no
8. Completion language audit passed: no
9. `IN_PROGRESS_FEATURE_WORKLIST.md` updated: no
10. `CURRENT_BLOCKERS.md` updated if needed: no

## Status
- execution_state: planned
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - real dependency graph build from bind facts is not realized yet
  - replay-visible dependency identity is not realized yet
  - invalidation closure over real TreeCalc dependency changes is not realized yet
- claim_confidence: draft
- reviewed_inbound_observations: latest OxFml seam baseline consumed; no new active trigger beyond declared dependency-projection watch points
