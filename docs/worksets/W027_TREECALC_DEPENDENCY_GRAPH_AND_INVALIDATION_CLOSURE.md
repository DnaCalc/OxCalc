# W027: TreeCalc Dependency Graph and Invalidation Closure

## Purpose
Replace planner-only dependency derivation with real dependency graph build and invalidation closure over TreeCalc structure plus consumed OxFml bind facts.
This packet widens the engine beneath the existing `OxCalcTree` host-facing consumer contract rather than introducing a second host-facing dependency surface.

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
- the `OxCalcTree` consumer contract exists as the current host-facing entry surface, with dependency realization still widening beneath it

### Exit gate
- structural dependency graph and reverse edges exist for the covered TreeCalc formula families
- dependency identity is deterministic and replay-visible
- invalidation closure is explicit for structure edits and dependency changes in phase scope

## Current Executed Floor
The first W027 floor is now exercised in live OxCalc code.

Current realized dependency substrate:
1. descriptor lowering into `DependencyDescriptor` and `DependencyGraph` is live for the current first TreeCalc subset:
   - `StaticDirect`
   - `RelativeBound`
   - `DynamicPotential`
   - `HostSensitive`
   - `Unresolved`
2. the graph now carries explicit:
   - `descriptors_by_owner`
   - `edges_by_owner`
   - `reverse_edges`
   - `cycle_groups`
   - `diagnostics`
3. replay-visible dependency identity is now stronger than local edge ids alone:
   - each lowered dependency descriptor now remains available as an explicit identity record
   - descriptor identity includes:
     - `descriptor_id`
     - `owner_node_id`
     - optional `target_node_id`
     - `kind`
     - `carrier_detail`
     - `requires_rebind_on_structural_change`
   - emitted `dependency_graph.json` now projects those descriptor records directly rather than forcing replay consumers to infer identity only from edge ids
4. invalidation closure is now derived from explicit seeds into:
   - `impacted_order`
   - per-node records with `calc_state`
   - explicit `requires_rebind`
   - sorted invalidation reasons
   - current exercised reasons now include:
     - `StructuralRebindRequired`
     - `StructuralRecalcOnly`
     - `UpstreamPublication`
     - `DependencyAdded`
     - `DependencyRemoved`
     - `DependencyReclassified`
5. cycle-sensitive state is no longer implicit:
   - cycle groups are computed during graph build
   - cycle members surface as `CycleBlocked` in invalidation closure where applicable
6. the host-facing TreeCalc contract already exposes this floor directly on `OxCalcTreeRecalcResult` through:
   - `dependency_graph`
   - `invalidation_closure`
7. deterministic artifact evidence for this floor already exists in the local TreeCalc runner and checked-in baseline through:
   - `dependency_graph.json`
   - `invalidation_closure.json`
   - `post_edit/invalidation_seeds.json`
8. non-structural invalidation evidence is now explicit rather than implied:
   - engine-model tests now exercise `UpstreamPublication`, `DependencyAdded`, `DependencyRemoved`, and `DependencyReclassified`
   - emitted-artifact tests now prove those same reasons survive `invalidation_closure.json` and `invalidation_seeds.json` projection

Current non-overclaim:
1. stronger replay-visible dependency identity beyond the current descriptor packet remains open only if later packets require cross-run or cross-host correlation stronger than the current descriptor fields
2. broader invalidation-cause widening beyond the current first non-structural reason set remains open
3. broader runtime-derived dependency closure still belongs to `W029`, not to this packet

## Pre-Closure Verification Checklist
1. Spec text and realization notes updated for all in-scope items: no
2. Pack expectations updated for affected packs: no
3. At least one deterministic replay artifact exists per in-scope behavior: no
4. Semantic-equivalence statement provided for policy or strategy changes: no
5. FEC/F3E cross-repo impact assessed and handoff filed if needed: no
6. All required tests pass: no
7. No known semantic gaps remain in declared scope: no
8. Completion language audit passed: no
9. `WORKSET_REGISTER.md` updated when ordered workset truth changed: no
10. `IN_PROGRESS_FEATURE_WORKLIST.md` updated when feature-map truth changed: no
11. execution-state blocker surface updated (`.beads/` for ordinary blockers; prose blocker surface only for exceptional narrative blockers): no

## Status
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - the first executed W027 floor now exists for graph build, reverse edges, cycle groups, diagnostics, structural invalidation seeds, and invalidation closure beneath `OxCalcTreeRecalcResult`
  - the first stronger replay-visible dependency identity floor now exists through explicit descriptor records beneath `DependencyGraph` and in emitted `dependency_graph.json`
  - any later identity widening would need to justify pressure beyond the current descriptor packet
  - broader invalidation-cause widening beyond the current first non-structural reason set remains open
  - runtime-derived dynamic dependency closure remains open and belongs to `W029`
  - broader sequential corpus/oracle baseline evidence remains open and belongs to `W030`
- claim_confidence: moderate
- reviewed_inbound_observations: latest OxFml seam baseline consumed; no new active trigger beyond declared dependency-projection watch points
