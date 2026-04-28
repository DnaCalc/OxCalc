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
Audit bead: `calc-8gw`.

1. Spec text and realization notes updated for all in-scope items: yes — this workset packet records the current executed floor, and the supporting TreeCalc semantic plan remains the governing phase-scope companion.
2. Pack expectations updated for affected packs: yes — no new pack family was introduced; W027 evidence remains bound to the existing TreeCalc local runner, dependency artifact, and validation surfaces.
3. At least one deterministic replay artifact exists per in-scope behavior: yes — checked-in TreeCalc local run artifacts include `dependency_graph.json`, `invalidation_closure.json`, and `post_edit/invalidation_seeds.json` for the covered phase scope.
4. Semantic-equivalence statement provided for policy or strategy changes: yes / not applicable — W027 does not promote a scheduler or coordinator strategy change; it realizes dependency graph and invalidation state beneath the existing sequential TreeCalc path, so observable formula results are unchanged except for intentional deterministic reject/diagnostic behavior when dependency state is invalid.
5. FEC/F3E cross-repo impact assessed and handoff filed if needed: yes — no new OxFml-owned FEC/F3E clause change was required; OxFml remains owner of parse/bind/evaluator meaning and W027 consumes the already admitted W026 bind/reference floor.
6. All required tests pass: yes — `cargo test --workspace`, scoped OxCalc `cargo fmt -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `scripts/check-worksets.ps1` pass for the audit run; clippy reports only a non-fatal warning in sibling `OxFunc`.
7. No known semantic gaps remain in declared scope: yes — remaining runtime-derived overlay, evaluator-backed candidate, and broader corpus/oracle lanes are explicitly outside W027 and belong to W028-W030.
8. Completion language audit passed: yes — closure language is limited to the declared W027 phase scope and does not claim W028-W031 semantics.
9. `WORKSET_REGISTER.md` updated when ordered workset truth changed: yes / not applicable — ordered workset truth did not change.
10. `IN_PROGRESS_FEATURE_WORKLIST.md` updated when feature-map truth changed: yes / not applicable — feature-map truth did not change for this W027 closure audit.
11. execution-state blocker surface updated (`.beads/` for ordinary blockers; prose blocker surface only for exceptional narrative blockers): yes — `calc-8gw` records the closure audit; no W027 follow-up blocker bead is required by this audit.

## Status
- execution_state: closure_recommended
- scope_completeness: scope_complete
- target_completeness: target_complete
- integration_completeness: integrated
- open_lanes: []
- closure_audit_result: pass for declared W027 phase scope
- next_ready_if_closed: `calc-g4q` / `W028 TreeCalc evaluator-backed candidate result integration`
- non_scope_successors:
  - runtime-derived dynamic dependency and overlay closure belongs to `W029`
  - broader sequential corpus/oracle baseline evidence belongs to `W030`
  - assurance refresh and residual packetization belongs to `W031`
- claim_confidence: high for W027 declared phase scope
- reviewed_inbound_observations: latest OxFml seam baseline consumed; no new active trigger beyond declared dependency-projection watch points
