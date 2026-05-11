# W047 Implementation Roadmap And Successor Gates

Status: `calc-aylq.7_dynamic_positive_publication_validated`

Parent workset: `W047 Calc-Time Rebinding Overlay Design Sweep`

Parent bead: `calc-aylq.4`

Predecessor gates:

1. `W047_HISTORICAL_NO_LOSS_CTRO_CROSSWALK.md`
2. `W047_EFFECTIVE_GRAPH_OVERLAY_AND_FRONTIER_REPAIR_SEMANTICS.md`
3. `W047_CTRO_SCENARIO_MATRIX_AND_TRACE_FACTS.md`

## 1. Purpose

This packet converts W047 CTRO design into an implementation and evidence roadmap. It distinguishes immediate W047 code work, W048 circular-dependency processing, W049 formal/checker/sidecar/readiness work, reference-machine work, OxFml handoff pressure, fallback/economics instrumentation, and no-promotion gates.

This packet does not claim implementation evidence. It is the scoping and routing surface for the remaining W047 implementation lane.

## 2. Current Implementation Baseline

Current local TreeCalc already has a partial CTRO-adjacent baseline:

| Area | Current surface | Interpretation |
| --- | --- | --- |
| graph build | `LocalTreeCalcEngine::execute` builds `DependencyGraph` and records cycle scan timing. | Structural graph path exists. |
| invalidation | `derive_invalidation_closure` is run before evaluation and carries `requires_rebind`. CTRO dynamic activation/release/reclassification reasons now dirty affected nodes without forcing structural rebind. | Bounded effective-graph invalidation exists for TreeCalc dynamic dependency facts; full native `G_eff_next` sidecar remains W049. |
| order | `topological_formula_order` derives formula order from current graph. | Order repair after candidate overlay is not implemented. |
| unresolved dynamic | `TreeReference::DynamicPotential` emits `runtime_effect.dynamic_reference`, projects a runtime-effect overlay, rejects with `DynamicDependencyFailure`. | Unsupported dynamic dependency is visible and no-publish. |
| resolved dynamic | `TreeReference::DynamicResolved` publishes and emits dynamic dependency activation shape updates plus runtime effect. | Positive dynamic dependency publication has a local floor. |
| dynamic switch with downstream dependent | Post-edit runs can seed previous published dynamic runtime effects, compare them with current dynamic facts, publish release+activation updates, and recalculate downstream dependents. | W047 fixture `tc_local_dynamic_target_switch_downstream_publish_001` is the bounded TreeCalc evidence floor. |
| rebind/fallback | `StructuralRebindRequired` rerun rejects with host-injected failure; fallback/re-entry is visible in TraceCalc. | Conservative safety path exists. |
| structural cycle | formula-family cycles reject with `SyntheticCycleReject`; no candidate result and no publication bundle. | Structural cycle floor exists. |
| shape/topology | shape/topology-sensitive references reject and project diagnostic overlays. | Spill/region semantics are exact-blocked. |

Current TraceCalc reference material already covers dynamic switch, structural cycle reject, and fallback re-entry. It does not yet cover candidate-overlay CTRO cycles or cycle release; those route to W048.

## 3. Immediate W047 Code Roadmap

W047 implementation should focus on the smallest code path that makes CTRO real without pulling in formal/checker/readiness work.

| Lane | Scope | Concrete target | Non-goal |
| --- | --- | --- | --- |
| CTRO state vocabulary | Add explicit runtime overlay state concepts to the core/TreeCalc path: published overlay input, candidate overlay output, and effective graph composition point. | Represent `O_published`, `O_candidate`, `G_eff`, and `G_eff_next` in or near the local TreeCalc execution path. | No Lean/TLA/checker sidecar. |
| dynamic dependency positive publication | Extend from `DynamicResolved` single-edge publication toward release/activation under the CTRO model. | Candidate carries dependency release/activation facts; publication commits value and overlay consequence together. | No broad Excel `INDIRECT` compatibility claim. |
| frontier repair floor | After candidate overlay classification, either repair impacted order/invalidation deterministically or route to fallback/reject. | Implement local deterministic repair only where equivalence to rebuild is evident; otherwise reject/fallback. | No speculative partial publication. |
| no-overlay-commit on reject | Preserve old published overlay when candidate fails. | Reject paths must retain candidate overlay only as diagnostics. | No hidden evaluator mutation. |
| traceable diagnostics | Emit enough existing diagnostics/artifacts to show graph, invalidation, candidate, publication, reject, and runtime effects. | Use current result/explain/trace artifacts where available; name W049 native sidecar gaps. | No new proof-carrying checker under W047. |

The first implementation target is now exercised as a bounded TreeCalc lane for dynamic dependency positive publication, release/activation, and downstream invalidation. CTRO-created cycles should not be implemented opportunistically inside W047 beyond routing them to the shared cycle policy; W048 owns the cycle-processing expansion.

## 4. Reference-Machine Work

| Reference surface | W047 action |
| --- | --- |
| TraceCalc dynamic dependency switch | Keep as reference semantics for release/activation and atomic publication. |
| TraceCalc fallback re-entry | Keep as reference semantics for visible fallback and later re-admission. |
| TraceCalc structural cycle reject | Keep as non-iterative cycle floor, but route CTRO cycle variants to W048. |
| TreeCalc local fixtures | W047 adds/adjusts implementation-facing cases for CTRO positive publication and downstream invalidation, including `tc_local_dynamic_target_switch_downstream_publish_001`. |
| Grid/spill reference cases | Record exact blockers unless a narrow local region model is introduced. |

## 5. W048 Circular Dependency Route

W048 owns behavior before formalization for:

1. structural self-cycle and multi-node SCC Excel probes;
2. CTRO self-cycle and CTRO multi-node cycle fixtures;
3. CTRO cycle release and deterministic re-entry;
4. downstream behavior when a CTRO-created cycle is introduced or released;
5. cycle provenance trace facts;
6. non-iterative last-successful-value policy and future iterative-profile questions.

W047 implementation should call or prepare for the same structural SCC classifier on `G_eff_next`, but W047 should not claim Excel-compatible cycle closure.

## 6. W049 Formal, Checker, Sidecar, And Readiness Route

W049 owns:

1. Lean/TLA model repair and non-vacuous proof targets;
2. native proof-carrying sidecars for reverse edges, per-read events, overlay deltas, frontier repair, cycle provenance, and publication bundles;
3. checker hardening against missing inputs and vacuous fact labels;
4. pack-grade replay, C5, operated-service, independent-evaluator, and continuous-scale readiness gates;
5. formalization around the landed Rust behavior after W047 and W048.

W047 must not add shallow Lean/TLA/checker artifacts to make readiness appear closer.

## 7. OxFml Handoff Pressure

Current OxFml seam text is sufficient for W047 planning at the local semantic floor:

1. candidate results distinguish value, shape, topology, runtime-derived facts, diagnostics, and trace correlation;
2. Stage 1 runtime-derived effects include dynamic dependency activation/release and region-shape activation/release;
3. Stage 1 reject subset includes `dynamic_dependency_failure`, `synthetic_cycle_reject`, and `host_injected_failure`;
4. open seam questions remain around exact retained/reduced witness projection, broader dependency reclassification transport, and exact trace schema mapping.

W047 should file an OxFml handoff only if implementation proves a shared-contract gap, such as:

1. a required dynamic dependency release/activation fact cannot be carried through current candidate/runtime-effect structures;
2. candidate-result consequence optionality is too weak for atomic value+overlay publication;
3. reject context lacks required diagnostics for dynamic dependency or CTRO cycle provenance;
4. trace correlation cannot preserve candidate, publication, and reject identifiers.

No handoff is required merely because W047 records W049 proof/checker obligations.

## 8. Fallback And Economics Instrumentation

W047 needs fallback to be honest and visible, not economically optimized.

Immediate counters/fields to preserve where the implementation already has a natural place:

1. fallback/reject reason;
2. affected work volume;
3. dynamic dependency failure count;
4. synthetic cycle reject count;
5. runtime overlay projection count;
6. dependency-shape update count;
7. publication versus no-publication outcome.

Economics questions routed to W049 or later operated evidence:

1. overlay repair versus rebuild crossover;
2. retained overlay reuse/miss/eviction rates;
3. dynamic-topology maintenance cost;
4. large-workbook or million-node economics;
5. readiness thresholds for pack/C5/operated promotion.

## 9. No-Promotion Gates

The following gates remain closed after W047 unless W049 later supplies direct evidence:

| Gate | W047 disposition |
| --- | --- |
| pack-grade replay | blocked; W049 readiness work. |
| C5 candidate | blocked; W049 readiness work. |
| operated service | blocked; W049 readiness work. |
| independent evaluator breadth | blocked; W049 readiness work. |
| release-grade verification | blocked; W049 readiness work. |
| formal proof of CTRO | blocked; W049 proof work. |
| Excel-compatible circular references | blocked; W048 behavior work first. |
| iterative calculation | blocked; W048 policy and later implementation. |

## 10. Successor Bead Scoping

| Bead | W047 disposition | Rationale |
| --- | --- | --- |
| `calc-aylq.5` | W049 placeholder, not W047 implementation work. | Rust/formal graph proof should wait for W047 implementation and W048 cycle semantics. |
| `calc-aylq.6` | W049 placeholder, not W047 implementation work. | Native proof-carrying sidecars require stable implementation and trace schema decisions. |
| `calc-aylq.7` | W047 implementation work. | Dynamic dependency positive publication is the immediate implementation refinement under CTRO. |
| `calc-aylq.8` | W049 placeholder, not W047 implementation work. | Pack/C5/operated readiness depends on W049 evidence. |

This keeps W047 under-scoping and over-scoping in check: the implementation work remains real, while formal/readiness surfaces do not masquerade as W047 results.

## 11. Acceptance Mapping

| `calc-aylq.4` acceptance item | Evidence in this packet |
| --- | --- |
| Roadmap distinguishes immediate code | Sections 2 and 3. |
| W048 cycle-processing work | Section 5. |
| W049 formal/checker/readiness work | Section 6. |
| Reference-machine work | Section 4. |
| OxFml handoff pressure | Section 7. |
| Fallback/economics instrumentation | Section 8. |
| No-promotion gates | Section 9. |
| Successor implementation beads ready and not over/under scoped | Section 10. |
| `calc-aylq.7` implementation evidence | `W047_DYNAMIC_DEPENDENCY_POSITIVE_PUBLICATION_EVIDENCE.md`. |

## 12. Current Status

- execution_state: `calc-aylq.7_dynamic_positive_publication_validated`
- scope_completeness: `scope_complete`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- successor_lanes:
  - W048 circular dependency calculation processing
  - W049 formal/checker/sidecar/readiness successor work
- known_non_w047_validation_gap: none observed in current review
