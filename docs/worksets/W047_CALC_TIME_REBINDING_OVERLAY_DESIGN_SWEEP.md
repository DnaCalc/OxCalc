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

**Scope reset (2026-05-10).** A fresh-eyes review of W046 surfaced that its formal artifacts are largely tautological projections: Lean theorems that reduce to record-field selection, TLA models with two-step `MaxTransitions` whose invariants restate the actions' own assignments, and a Python checker that silently downgrades missing inputs to empty graphs and then unconditionally records "fact" labels. W047 must not extend that pattern. W047 is therefore re-scoped to **implementation-first**: land the Calc-Time Rebinding Overlay phase in the engine core and the surrounding design, reference, and scenario surfaces — and stop there. W047 does not produce new Lean modules, TLA models, sidecar emitters, or proof-carrying-trace checker rules; it does not attempt to fix or formally prove anything that was not already proven before W047. Formal-evidence deepening, sidecar enrichment, native trace emission, and readiness-gate work transfer to **W049** as a follow-on formalization workset that uses the W046 punch list and the W047 implementation core as its starting points. The intent is to build a single correct implementation core first, then systematically formalize around it — not to grow a parallel layer of decorative formal artifacts.

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

The active no-loss crosswalk packet is
`docs/spec/core-engine/w047-ctro/W047_HISTORICAL_NO_LOSS_CTRO_CROSSWALK.md`.
That packet is the input gate for subsequent W047 effective-graph,
scenario-matrix, implementation-remodeling, and positive-publication work.

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
10. Compose provisional effective graph `G_eff' = G_struct + O_candidate`.
11. Run the same SCC/cycle classification over `G_eff'` that the structural graph path uses over `G_struct` and `G_eff`.
12. If `G_eff'` introduces or preserves a cycle region, route that region through the profile-governed cycle policy before any frontier repair or overlay publication.
13. Repair the current frontier and order under `G_eff'` only for the acyclic remainder, or route to fallback/reject when repair is unsafe or unsupported.
14. Continue evaluation only after cycle classification and frontier repair have produced a deterministic next frontier.
15. Candidate carries value deltas, dependency-shape deltas, runtime effects, diagnostics, and proof-carrying trace obligations.
16. Coordinator accepts and atomically publishes both values and overlay consequences, or rejects with no-publish/no-overlay-commit.
17. Retention/eviction policy protects old overlay state while readers or replay witnesses require it.

This is the pipeline W047 must implement in the engine core and demonstrate via reference scenarios. Formal proof obligations against this pipeline are recorded as inputs to W049 and are explicitly out of scope for W047.

The active effective-graph semantics packet is
`docs/spec/core-engine/w047-ctro/W047_EFFECTIVE_GRAPH_OVERLAY_AND_FRONTIER_REPAIR_SEMANTICS.md`.

### 5.1 Cycle Policy Alignment

CTRO cycle handling must be semantically the same cycle policy as structural cycle handling, applied to a different graph layer.

Rules:
1. structural cycles are classified over `G_struct` plus any already-published overlay that participates in the current effective graph;
2. CTRO-created cycles are classified over provisional `G_eff'` after candidate overlay composition and before publication;
3. the cycle classifier returns SCC/cycle groups plus the affected nodes and edges, not an ad hoc CTRO-only failure code;
4. for the current Stage 1 / non-iterative profile floor, any cycle group detected in `G_eff'` becomes `CycleBlocked` / `SyntheticCycleReject` with no value publication and no overlay commit;
5. `O_published` remains the effective runtime overlay after a CTRO cycle reject; `O_candidate` is discarded or retained only as reject diagnostics;
6. downstream dependents of a rejected cycle region remain stale/needed/blocked according to the same invalidation closure policy used for structural cycle regions;
7. if a later profile enables iterative cycle semantics, both structural and CTRO-created cycle regions must enter the same deterministic SCC iteration lane with the same iteration bounds, convergence threshold, terminal diagnostics, and replay evidence;
8. a CTRO cycle may not be hidden behind generic fallback if the effective graph already proves the cycle; fallback is allowed only when the implementation cannot safely classify or repair the graph, and that fallback must be replay-visible.

This aligns with the current Rust floor: `DependencyGraph::build` records `cycle_groups`, invalidation can mark cycle members `CycleBlocked`, and local TreeCalc maps cycle detection to `SyntheticCycleReject` with no candidate publication. W047 should extend that policy to candidate overlay graph classification rather than inventing a parallel CTRO-specific cycle path.

## 6. Design Rules To Prevent A Bolt-On

1. **Name the third state**: do not force calc-time rebinding into either structural mutation or value-only recalc.
2. **Effective graph is explicit**: every proof/model/checker row must say whether it uses `G_struct`, `O_published`, `O_candidate`, or `G_eff`.
3. **Candidate shape and value commit together**: publication cannot commit value changes while silently dropping shape changes.
4. **No hidden evaluator mutation**: OxFml/TreeCalc evaluation may discover shape facts, but only OxCalc-owned coordinator paths commit them.
5. **Frontier repair is a first-class transition**: if order/invalidation changes mid-calculation, the engine records repair, fallback, or rejection.
6. **Dynamic arrays are first-class stress cases**: spill/region resizing must be modeled beside `INDIRECT`, not postponed as unrelated future work.
7. **Fallback is honest but measured**: conservative rebuild/reject is allowed, but must be replay-visible and instrumented.
8. **Cycle policy is shared**: structural and CTRO-created cycle regions differ in discovery timing, not in semantic policy; both flow through the same profile-governed cycle lane.
9. **Proof obligations are recorded, not discharged under W047**: when the design implies a checkable fact about overlay deltas, frontier repair, cycle classification, or atomic publication, name the obligation and route it to W049; do not extend the W046 fact-list checker, sidecar emitter, Lean record-projection pattern, or smoke-TLA pattern as part of W047.
10. **No readiness promotion by proxy**: W047 may improve engine design, but pack/C5/operated/release claims remain consequence lanes.

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
| structural direct cycle | baseline structural/SCC cycle path and current Stage 1 reject/no-publication semantics |
| CTRO self-cycle introduced by dynamic target switch | candidate overlay points a formula back to itself and must produce the same cycle policy outcome as a structural self-cycle |
| CTRO multi-node cycle introduced by dynamic target switch | candidate overlay creates an SCC across two or more formulas and must block/reject consistently with structural SCC handling |
| CTRO cycle release | previously-published overlay cycle candidate is replaced by an acyclic target and must define how the blocked region can re-enter calculation |
| cycle introduced by calc-time overlay | cycle boundary discovered after runtime shape change |
| conservative fallback/rebuild | safe path when dynamic repair cannot be proven |

### 7.1 Excel-Comparison Probing Examples

Microsoft's public Excel documentation establishes the comparison baseline:
1. Excel detects direct and indirect circular references and warns the user when a formula depends on itself, directly or indirectly.
2. Without intentional iterative calculation, Excel may display zero or retain the last successful calculated value after the circular-reference warning.
3. If iterative calculation is enabled, Excel recalculates until the configured maximum iterations or maximum change threshold is reached; the documented defaults are 100 iterations or 0.001 maximum change.
4. Excel's calculation chain is dynamic and may be reordered during recalculation when dependencies require it.

Probing examples for W047/W048 investigation:

| Probe id | Excel sheet sketch | Discovery class | Question |
| --- | --- | --- | --- |
| `excel_struct_self_001` | `A1 = A1 + 1` | structural self-cycle | Confirm warning, status-bar location, default displayed value, and iterative result under default iteration settings. |
| `excel_struct_two_node_001` | `A1 = B1 + 1`; `B1 = A1 + 1` | structural SCC | Confirm which cell Excel reports first, whether both cells retain last values, and how iterative mode diverges. |
| `excel_guarded_cycle_activation_001` | `A1 = IF(B1=0,0,A1+1)`; toggle `B1` from `0` to `1` | structural formula with runtime condition | Confirm the documented "last successful calculation" behavior when a condition activates self-reference. |
| `excel_ctro_indirect_self_001` | `B1 = "A1"`; `C1 = INDIRECT(B1)`; then set `B1 = "C1"` | CTRO dynamic target self-cycle | Determine whether Excel reports this like a normal circular reference and what value survives without iteration. |
| `excel_ctro_indirect_two_node_001` | `A1 = "D1"`; `B1 = "C1"`; `C1 = INDIRECT(A1)`; `D1 = INDIRECT(B1)` | CTRO dynamic target SCC | Determine warning/reporting order and whether calculation-chain reordering is observable. |
| `excel_ctro_cycle_release_001` | Start from `C1 = INDIRECT(B1)` with `B1 = "C1"`, then change `B1 = "A1"` and `A1 = 10` | CTRO cycle release | Determine whether Excel exits the circular-reference state cleanly and what recalculation event is required. |
| `excel_ctro_spill_cycle_001` | dynamic-array/spill formula whose output range becomes an input range to itself | CTRO region/spill cycle | Determine whether the public circular-reference behavior extends to spill-region membership and what exact error or warning is surfaced. |

These are proposed probes, not claimed observed behavior. W047 uses them to shape implementation policy; W048 owns direct Excel observation packets and deterministic replay/trace expectations.

The active W047 scenario matrix packet is
`docs/spec/core-engine/w047-ctro/W047_CTRO_SCENARIO_MATRIX_AND_TRACE_FACTS.md`.

The bounded dynamic dependency positive-publication evidence packet is
`docs/spec/core-engine/w047-ctro/W047_DYNAMIC_DEPENDENCY_POSITIVE_PUBLICATION_EVIDENCE.md`.

## 8. Formal Obligations Recorded For W049

W047 does **not** produce new formal artifacts. The CTRO design surfaces the following obligations, recorded as inputs to the W049 follow-on formalization workset:

1. effective graph composition law,
2. overlay delta classification (activate, release, reclassify, region/spill resize),
3. no-under-invalidation over effective graph,
4. frontier repair soundness after overlay delta,
5. order repair or deterministic fallback,
6. SCC/cycle classification equivalence between `G_struct`, `G_eff`, and candidate `G_eff'`,
7. no-publish/no-overlay-commit on reject,
8. atomic publication of value plus overlay consequences,
9. retained overlay safety for pinned readers/replay,
10. proof-carrying trace schema for overlay facts.

Each is a future W049 obligation. W047 implementation work must not introduce stub Lean predicates, smoke TLA models, or fact-list checker rows that pretend to discharge these obligations. If an obligation surfaces during implementation in a way that demands an in-line check, prefer a Rust assertion or test that fails loudly under a real run over a Lean/TLA artifact that passes vacuously.

## 9. Evidence Policy

W047 evidence is implementation-first and staged narrowly:

1. **Design evidence**: historical no-loss crosswalk, scenario matrix, revised pipeline, design invariants stated in prose.
2. **Reference evidence**: TraceCalc oracle scenarios for static comparator, dynamic target switch, spill/region changes, structural cycle, CTRO-created cycle, cycle release, and fallback paths — at the granularity the existing reference machinery already supports.
3. **Implementation evidence**: core engine and TreeCalc CTRO phase landing; current supported subset; exact blockers for unsupported syntax, cycles, or grid/spill features.
4. **Hand-off evidence**: explicit list of formal, checker, sidecar, and economics obligations registered as W049 inputs; nothing more.

W047 does **not** produce formal, checker, sidecar, or economics evidence. Broadening any of those layers under W046's pattern is an explicit anti-goal — adding a new shallow Lean module or smoke TLA model under W047 is worse than producing nothing on that axis under W047. W049 will design the evidence layer cleanly against the landed implementation core.

## 10. Bead Path

The W047 bead path is implementation-first; formal-evidence and readiness-gate beads route to W049 instead.

Active under W047:

1. `calc-aylq.1` - historical no-loss design sweep and Calc-Time Rebinding Overlay doctrine.
2. `calc-aylq.2` - effective graph, overlay delta, frontier repair, and publication semantics specified at design level (no Lean/TLA artifact under W047).
3. `calc-aylq.3` - scenario matrix and TraceCalc/TreeCalc evidence plan; proof-carrying-trace schema notes are recorded as W049 inputs, not implemented under W047.
4. `calc-aylq.4` - implementation/evidence roadmap, successor routing, fallback/economics counters, and no-promotion gates; no runtime behavior claim.
5. `calc-aylq.7` - core engine and TreeCalc CTRO phase landing for dynamic dependency positive publication under the CTRO model.

Deferred to W049 when W047 closes:

- `calc-aylq.5` - Rust/formal graph algorithm proof obligations after CTRO design — re-scope under W049 against the landed implementation.
- `calc-aylq.6` - native proof-carrying-trace sidecar enrichment — W049.
- `calc-aylq.8` - semantic pack and operated-service readiness gate — W049; gates pass through W049's evidence layer.

These three bead ids are retained as deferred placeholders. Their content will be re-issued under the W049 epic when the W049 plan is finalized; under W047 they are not worked.

## 11. Exit Gate

W047 exits only when:

1. historical no-loss crosswalk is complete,
2. Calc-Time Rebinding Overlay is integrated into core recalc, graph, OxFml seam, TraceCalc, TreeCalc, and design surfaces — formal/checker/sidecar surfaces are explicitly **not** changed under W047,
3. scenario matrix covers static, dynamic switch, unresolved dynamic, downstream dependent, spill expansion/contraction, structural cycle, CTRO-created cycle, cycle release, and fallback paths or records exact blockers,
4. effective graph, frontier repair, and shared cycle-policy semantics are specified at design level; bounded-model/checker validation is **deferred to W049** and recorded as an obligation, not produced under W047,
5. implementation plan distinguishes immediate TreeCalc/core-engine CTRO phase work, reference-machine work, and deferred grid/spill substrate work; checker, sidecar, and formal artifact work is explicitly handed to W049,
6. fallback policy is explicit at design level; economics counters are deferred to W049 unless the W047 implementation surface needs one to remain honest,
7. successor readiness gates (W049-administered) are recorded as preventing pack/C5/operated/release promotion before W049 evidence exists,
8. W046 residuals are reclassified under the CTRO model rather than simply carried forward unchanged,
9. no new formal/checker/sidecar artifacts are introduced under W047; obligations are recorded as W049 inputs.

## 12. Current Status

- execution_state: `calc-aylq.7_dynamic_positive_publication_validated`
- scope_completeness: `scope_complete`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- scope_mode: `implementation_first` (post 2026-05-10 reset; see §1 scope reset paragraph)
- successor_lanes:
  - W048 structural and CTRO-created cycle calculation processing
  - W049 formal/checker/sidecar/readiness successor work
- known_non_w047_validation_gap: none observed in current review
- deferred_to_w049:
  - Rust/formal graph algorithm proof obligations
  - native proof-carrying-trace sidecar enrichment
  - semantic pack / operated-service readiness gate evidence
