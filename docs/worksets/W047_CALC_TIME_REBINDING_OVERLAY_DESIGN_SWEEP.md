# W047 Calc-Time Rebinding Overlay Design Sweep

Status: `activated_design_sweep`

Parent predecessor: `W046`

Parent epic: `calc-aylq`

## 1. Purpose

W047 starts from the W046 closure insight that the calculation engine still needs a first-class design for **Calc-Time Rebinding Overlay**.

A Calc-Time Rebinding Overlay is the runtime-derived effective-graph layer used when formula evaluation discovers that dependency shape changed without a structural model change. Examples include:

1. `INDIRECT("A" & B1)` changing its target from `A1` to `A2` because `B1` changed,
2. dynamic arrays or spill regions resizing during evaluation,
3. range or region membership changing because runtime formula results alter shape/topology,
4. dynamic dependency release/activation that should repair the current recalculation frontier rather than being hidden as a structural edit or collapsed into generic fallback.

The purpose of W047 is not to bolt this onto W046 after the fact. The purpose is to perform a careful design review sweep so the resulting engine design looks like the intended architecture would have looked if Calc-Time Rebinding Overlay had remained central from the beginning.

## 2. Core Thesis

The engine has three distinct change classes:

| Change class | Example | Graph consequence | Engine path |
| --- | --- | --- | --- |
| value-only recalc | `A1` changes; `C1 = A1 + B1` | graph shape unchanged | normal reverse invalidation and topological evaluation |
| structural/model change | node/formula/bind artifact changes | structural snapshot successor and dependency rebuild/rebind | structural invalidation/rebind path |
| calc-time rebinding overlay | `B1` changes dynamic address; spill range resizes | effective runtime graph changes under same structural snapshot | stage dependency-shape overlay, repair frontier/order, publish or reject atomically |

W047 owns the third class.

## 3. Why This Must Be A Sweep, Not A Patch

Calc-Time Rebinding Overlay cuts across nearly every W046 semantic object:

1. graph facts now have base structural edges plus runtime-derived overlay edges,
2. invalidation closure must handle overlay release/activation and downstream value propagation,
3. evaluation order may need repair after a formula emits a dependency-shape delta,
4. candidate results must carry value deltas and shape/topology deltas together,
5. publication must atomically commit both published values and accepted overlay consequences,
6. rejection/fallback must preserve the previous published view and previous effective graph,
7. proof-carrying traces must expose overlay deltas, not hide them in prose,
8. dynamic arrays/spill resizing need the same conceptual lane as `INDIRECT`, not a separate ad-hoc exception.

Therefore W047 must review and integrate the old W003/W004/W007/W008/W009/W010/W012/W027/W029 design material, W046 formal spine, current Rust implementation, TraceCalc scenarios, TreeCalc fixtures, and OxFml seam implications before implementation promotion.

## 4. Historical No-Loss Inputs

The sweep must consume these predecessor surfaces explicitly:

| Surface | Relevant idea to recover |
| --- | --- |
| `docs/worksets/W003_STAGE1_COORDINATOR_AND_PUBLICATION_BASELINE.md` | candidate result includes `dependency_shape_updates` and `runtime_effects` |
| `docs/worksets/W004_INCREMENTAL_RECALC_AND_OVERLAY_BASELINE.md` | dynamic dependency overlay, `activate_dynamic_dep`, `release_dynamic_dep`, `change_region_membership`, `synthetic_spill_shape`, fallback policy |
| `docs/worksets/W007_LEAN_FACING_STATE_OBJECTS_AND_TRANSITION_BOUNDARY_PLAN.md` | runtime-derived state and `OverlayEntry` separated from structural truth |
| `docs/worksets/W008_TLA_COORDINATOR_PUBLICATION_AND_FENCE_SAFETY_MODEL_PLAN.md` | `overlayState` participates in candidate, reject, publish, and eviction actions |
| `docs/worksets/W009_REPLAY_AND_PACK_BINDING_FOR_STAGE1_SEAM_AND_COORDINATOR_BEHAVIOR.md` | replay classes for fallback and overlay re-entry; dynamic dependency pack semantics |
| `docs/worksets/W010_EXPERIMENT_REGISTER_AND_MEASUREMENT_SCHEMA_PLANNING.md` | dynamic-topo versus rebuild and dynamic-dependency economics experiments |
| `docs/worksets/W012_TRACECALC_REFERENCE_MACHINE_AND_CONFORMANCE_ORACLE.md` | reference-machine state includes runtime overlay state and conformance surfaces |
| `docs/worksets/W027_TREECALC_DEPENDENCY_GRAPH_AND_INVALIDATION_CLOSURE.md` | current explicit graph, reverse edges, invalidation records, dependency-change reasons |
| `docs/worksets/W029_TREECALC_RUNTIME_DERIVED_EFFECTS_AND_OVERLAY_CLOSURE.md` | runtime-derived effects should be live, replay-visible, and not hidden mutable truth |
| `docs/spec/core-engine/CORE_ENGINE_RECALC_AND_INCREMENTAL_MODEL.md` | structural graph versus effective runtime graph distinction |
| `docs/spec/core-engine/CORE_ENGINE_OXFML_SEAM.md` | runtime-derived dependency, region, spill, and shape effects through the seam |
| W046 semantic-spine packets | graph/invalidation/order/reject/publication/refinement proof vocabulary to extend |

No W047 design packet may claim closure until it shows where each predecessor idea lands: retained, revised, rejected with reason, or routed to successor.

## 5. Intended Engine Pipeline

W047 should converge on an explicit pipeline for calc-time rebinding:

1. Start with immutable structural snapshot `S`.
2. Build base structural dependency graph `G_struct` from descriptors known before evaluation.
3. Attach accepted runtime overlay `O_published` from prior publications.
4. Define effective graph `G_eff = G_struct + O_published`.
5. Receive edit or upstream value change.
6. Derive initial invalidation closure over `G_eff`.
7. Evaluate needed formulas in deterministic order.
8. If evaluation emits dependency-shape effects, stage them in candidate overlay `O_candidate`.
9. Compare `O_candidate` against `O_published` for releases, activations, region/spill changes, and reclassifications.
10. Repair the current frontier and order under `G_eff' = G_struct + O_candidate`.
11. Continue evaluation or route to fallback/reject when repair is unsafe or unsupported.
12. Candidate carries value deltas, dependency-shape deltas, runtime effects, diagnostics, and proof-carrying trace facts.
13. Coordinator accepts and atomically publishes both values and overlay consequences, or rejects with no-publish/no-overlay-commit.
14. Retention/eviction policy protects old overlay state while readers or replay witnesses require it.

This is the pipeline W047 must prove, model, or explicitly scope down.

## 6. Design Rules To Prevent A Bolt-On

1. **Name the third state**: do not force calc-time rebinding into either structural mutation or value-only recalc.
2. **Effective graph is explicit**: every proof/model/checker row must say whether it uses `G_struct`, `O_published`, `O_candidate`, or `G_eff`.
3. **Candidate shape and value commit together**: publication cannot commit value changes while silently dropping shape changes.
4. **No hidden evaluator mutation**: OxFml/TreeCalc evaluation may discover shape facts, but only OxCalc-owned coordinator paths commit them.
5. **Frontier repair is a first-class transition**: if order/invalidation changes mid-calculation, the engine records repair, fallback, or rejection.
6. **Dynamic arrays are first-class stress cases**: spill/region resizing must be modeled beside `INDIRECT`, not postponed as unrelated future work.
7. **Fallback is honest but measured**: conservative rebuild/reject is allowed, but must be replay-visible and instrumented.
8. **Proof-carrying traces lead implementation**: design the facts the checker needs before optimizing the runtime path.
9. **No readiness promotion by proxy**: W047 may improve engine design, but pack/C5/operated/release claims remain consequence lanes.

## 7. Required Scenario Matrix

At minimum, W047 must include scenarios for:

| Scenario | Purpose |
| --- | --- |
| static value edit: `C1 = A1 + B1`, edit `A1` | baseline graph-stable recalc comparator |
| dynamic target switch: `INDIRECT("A" & B1)`, edit `B1: 1 -> 2` | dependency release/activation and frontier repair |
| unresolved dynamic target | reject/no-publish/no-overlay-commit boundary |
| dynamic target switch with downstream dependent `D1 = INDIRECT("C1")` | downstream value invalidation after overlay change |
| spill expansion | runtime region activation and new dependents |
| spill contraction | runtime region release and stale published cells/consumers |
| cycle introduced by calc-time overlay | cycle boundary discovered after runtime shape change |
| conservative fallback/rebuild | safe path when dynamic repair cannot be proven |

## 8. Formalization Targets

W047 should produce formal or executable targets for:

1. effective graph composition,
2. overlay delta classification: activate, release, reclassify, region/spill resize,
3. no-under-invalidation over effective graph,
4. frontier repair soundness after overlay delta,
5. order repair or deterministic fallback,
6. no-publish/no-overlay-commit on reject,
7. atomic publication of value plus overlay consequences,
8. retained overlay safety for pinned readers/replay,
9. proof-carrying trace schema for overlay facts.

## 9. Evidence Policy

W047 evidence should be staged:

1. **Design evidence**: historical no-loss crosswalk, scenario matrix, revised pipeline, invariants.
2. **Reference evidence**: TraceCalc oracle scenarios for dynamic target switch, spill/region changes, and fallback.
3. **Implementation evidence**: TreeCalc fixtures for current supported subset, exact blockers for unsupported syntax or grid/spill features.
4. **Formal evidence**: Lean/TLA or exact blockers for effective graph and frontier repair properties.
5. **Checker evidence**: proof-carrying trace rows that validate overlay deltas and publication/reject boundaries.
6. **Economics evidence**: counters for fallback rate, overlay repair rate, rebuild crossover, and retained overlay reuse.

## 10. Bead Path

The W047 bead path is intentionally front-loaded with design integration before implementation proof:

1. `calc-aylq.1` - historical no-loss design sweep and Calc-Time Rebinding Overlay doctrine.
2. `calc-aylq.2` - effective graph, overlay delta, frontier repair, and publication semantics.
3. `calc-aylq.3` - scenario matrix, TraceCalc/TreeCalc evidence plan, and proof-carrying trace schema.
4. `calc-aylq.4` - implementation remodeling plan, fallback/economics policy, and readiness gates.
5. `calc-aylq.5` - Rust Tarjan and topological queue line proof or revised obligation after overlay design.
6. `calc-aylq.6` - native proof-carrying trace sidecar enrichment for reverse edges, per-read events, and overlay deltas.
7. `calc-aylq.7` - dynamic dependency positive publication refinement under the CTRO model.
8. `calc-aylq.8` - semantic pack and operated-service readiness gate after CTRO evidence exists.

## 11. Exit Gate

W047 exits only when:

1. historical no-loss crosswalk is complete,
2. Calc-Time Rebinding Overlay is integrated into core recalc, graph, OxFml seam, TraceCalc, TreeCalc, proof-carrying trace, and formalization surfaces,
3. scenario matrix covers static, dynamic switch, unresolved dynamic, downstream dependent, spill expansion/contraction, cycle, and fallback paths or records exact blockers,
4. effective graph and frontier repair semantics are specified and at least bounded-model/checker validated or exact-blocked,
5. implementation plan distinguishes immediate TreeCalc work, reference-machine work, checker work, and deferred grid/spill substrate work,
6. fallback/economics policy is explicit and measurable,
7. successor readiness gates prevent pack/C5/operated/release promotion before CTRO evidence exists,
8. W046 residuals are reclassified under the CTRO model rather than simply carried forward unchanged.

## 12. Current Status

- execution_state: `calc-aylq_activated_design_sweep`
- scope_completeness: `scope_partial`
- target_completeness: `target_partial`
- integration_completeness: `partial`
- open_lanes:
  - historical no-loss crosswalk
  - effective graph and frontier repair semantics
  - scenario matrix and proof-carrying overlay trace schema
  - implementation remodeling and fallback/economics policy
  - Rust/formal/checker proof deepening after design sweep
