# W048 Circular Dependency Calculation Processing

Status: `closure_audit_passed`

Parent predecessor: `W047` (Calc-Time Rebinding Overlay implementation core)

Successor dependency: `W049` (successor formalization/deepening after W048 cycle artifacts exist)

Parent epic: `calc-zci1`

Spec packet root: `docs/spec/core-engine/w048-cycles/`

## 1. Purpose

W048 owns circular dependency calculation processing end to end. It covers cycles discovered in the structural-derived graph and cycles introduced, preserved, released, or reclassified by Calc-Time Rebinding Overlay candidate graphs.

The workset includes design, planning, Excel exploration, reference and optimized/core implementations, formal definitions and first proof/model/checker artifacts, a deterministic testing corpus, and a profile-gated innovation ledger. It does not claim Excel-compatible cycle closure or iterative calculation support until those lanes have evidence.

## 2. Why W048 Comes Before W049

W047 makes CTRO a first-class effective-graph update: evaluation can discover that dependency shape changed under the same structural snapshot. That means circular dependencies can appear in at least three graph layers:

1. `G_struct`: the structural-derived graph before runtime-derived overlay effects;
2. `G_eff`: the published effective graph after accepted overlay state;
3. `G_eff_candidate`: the candidate effective graph after current-wave runtime observations.

Those discovery points must converge on one cycle policy surface inside W048. W049 may later deepen or reorganize formalization after W048 has introduced concrete behavior, tests, graph artifacts, and first formal definitions/models.

## 3. Current OxCalc Floor

Current known floor:

1. `DependencyGraph::build` records structural `cycle_groups`;
2. the in-memory graph stores forward edges and reverse edges;
3. invalidation closure can mark cycle members `CycleBlocked`;
4. local TreeCalc rejects formula-family cycles as `SyntheticCycleReject`;
5. TreeCalc graph sidecars expose descriptors, forward edges, diagnostics, and cycle groups;
6. TraceCalc planning has direct dependencies, reverse dependencies, SCC grouping, and cycle groups;
7. non-iterative Stage 1 therefore has an explicit no-publication cycle boundary;
8. broad Excel-compatible iterative cycle semantics are not yet selected.

W048 must widen the artifacts and policy around this floor, implement the selected behavior in TraceCalc and TreeCalc, and bind the behavior to tests and formal artifacts rather than hide cycle behavior behind fallback.

## 4. Public Excel Evidence Baseline

Official Microsoft documentation establishes the comparison target at a high level:

1. Excel constructs a dependency tree and calculation chain, marks direct and indirect dependents dirty, and can revise the calculation chain during recalculation.
2. Excel saves calculation-chain metadata to workbooks; the chain can take multiple edits/calculations to settle and a workbook can have multiple valid calculation chains.
3. A circular reference exists when a formula refers to its own cell directly or indirectly.
4. Excel warns on circular references and exposes navigation/status-bar help.
5. With iterative calculation off, Excel can display zero or retain the last successful calculated value.
6. With iterative calculation on, Excel stops by maximum-iteration or maximum-change settings; documented defaults are 100 iterations or 0.001 maximum change.
7. Data tables have special documented recalculation behavior and are not part of ordinary TreeCalc cycle semantics.

Reference URLs:

1. `https://learn.microsoft.com/en-us/office/client-developer/excel/excel-recalculation`
2. `https://support.microsoft.com/en-us/office/excel-calculation-chain-metadata-6e1b5819-6abd-4e94-bff5-838d4c576e01`
3. `https://support.microsoft.com/en-us/office/remove-or-allow-a-circular-reference-in-excel-8540bd0f-6e97-4483-bcf7-1b49cd50d123`
4. `https://support.microsoft.com/en-gb/office/change-formula-recalculation-iteration-or-precision-in-excel-73fc7dac-91cf-4d36-86e8-67124f6bcce4`
5. `https://learn.microsoft.com/en-us/office/vba/api/excel.application.iteration`
6. `https://learn.microsoft.com/en-us/office/vba/api/excel.application.maxiterations`
7. `https://learn.microsoft.com/en-us/office/vba/api/excel.application.maxchange`

## 5. Literature And Foundation Intake

W048 consumes the related literature as follows:

1. Jane Street Incremental: useful vocabulary for necessary/observed/stale nodes, height-ordered stabilization, dynamic graph changes through bind, cutoff, and analyzable graph exports; not a source of circular-reference semantics because its ordinary graph is a DAG.
2. Self-adjusting computation and Adapton: useful for dynamic-dependency soundness, mutation/change-propagation consistency, and from-scratch equivalence obligations.
3. Build Systems a la Carte: useful separation of task/store/scheduler/rebuilder and the warning that many correctness definitions assume acyclic tasks; iterative tasks need bounded or fixed-point discipline.
4. Foundation DAG research: already promotes deterministic topo/SCC, fixed-point semantics as a possible profile, dynamic dependency soundness, and replay-visible graph facts.

Detailed intake lives in `docs/spec/core-engine/w048-cycles/W048_CYCLE_LITERATURE_AND_DECISION_MAP.md`.

## 6. W048 Design Decisions To Make Explicit

W048 must make these choices explicit and executable:

1. detection graph: `G_struct`, `G_eff`, or `G_eff_candidate`;
2. cycle-region identity and stable member ordering;
3. cycle root/reporting policy;
4. initial-value policy for iteration or prior-value display;
5. update model: snapshot/Jacobi, ordered/sequential, Excel-chain ordered, or another declared model;
6. terminal policy: error/reject, max iterations, max change, divergence, oscillation;
7. publication boundary: whole-wave reject, cycle-region reject, frontier partition publication, or display-retention-only;
8. CTRO no-commit rule for rejected candidate overlays;
9. release/re-entry invalidation and downstream dependent state;
10. trace/sidecar fields required for replay and formalization.
11. formal definitions/model vocabulary for the chosen graph and cycle profiles.
12. test corpus obligations for Excel observations, TraceCalc, TreeCalc, and checker/formal projections.
13. innovation profiles that OxCalc may offer beyond Excel behavior.

## 7. Provisional Stage 1 Policy

W048 starts from this conservative Stage 1 target:

1. classify SCCs with the same deterministic classifier across structural, published-effective, and candidate-effective graph layers;
2. preserve cycle provenance as diagnostic data, not as a separate semantic class;
3. treat CTRO-created SCCs as ordinary cycle regions with `cycle_source = candidate_overlay`;
4. under non-iterative Stage 1, reject structural and CTRO-created formula-family cycles through the shared cycle policy;
5. publish no new cycle-region values on reject;
6. commit no candidate overlay that created a rejected cycle;
7. retain the last published effective graph as the basis after reject;
8. emit materialized graph, cycle-region, and invalidation facts;
9. route future iterative behavior through an explicit profile after Excel probes and algorithm decisions.

## 8. Required Materialized Graph Surface

W048 requires graph artifacts that expose:

1. graph layer and basis metadata;
2. nodes;
3. forward edges;
4. reverse edges;
5. edge provenance and overlay deltas;
6. SCC/cycle-region records;
7. topological order or blocked/rejected reason;
8. stable graph hash;
9. invalidation/re-entry relation to reverse edges.

The graph materialization contract lives in `docs/spec/core-engine/w048-cycles/W048_GRAPH_MATERIALIZATION_AND_CTRO_LAYERS.md`.

## 9. Excel Probe Surface

W048 probe families:

1. direct self-cycle and prior-value retention;
2. two-node and three-node structural SCCs;
3. guarded activation cycles;
4. iterative self and multi-node cycles;
5. order-sensitive probes that distinguish snapshot versus sequential update;
6. edit-order and calculation-chain sensitivity probes;
7. cold-open and full-rebuild variants;
8. `INDIRECT`/dynamic-reference CTRO analogs;
9. CTRO release and downstream dependent probes;
10. spill/region and data-table boundary probes.

The probe schema and catalog live in `docs/spec/core-engine/w048-cycles/W048_EXCEL_PROBE_CATALOG_AND_OBSERVATION_SCHEMA.md`.

## 10. Implementation And Formalization Route

Work proceeds through the W048 bead epic:

| Bead | Purpose |
| --- | --- |
| `calc-zci1.1` | Excel circular-reference probe harness and observation ledger |
| `calc-zci1.2` | materialized dependency graph layers and sidecars |
| `calc-zci1.3` | TraceCalc reference cycle implementation |
| `calc-zci1.4` | iterative-profile algorithm decision and Excel disposition |
| `calc-zci1.5` | W048 formal definitions and proof/model artifacts |
| `calc-zci1.6` | TreeCalc optimized cycle implementation |
| `calc-zci1.7` | circular-reference test corpus and conformance runs |
| `calc-zci1.8` | innovation opportunity ledger and experimental profiles |

The roadmap lives in `docs/spec/core-engine/w048-cycles/W048_ENGINE_AND_FORMALIZATION_ROADMAP.md`.

## 11. Test Corpus And Innovation Surfaces

W048 owns:

1. Excel observation packets;
2. TraceCalc reference fixtures;
3. TreeCalc optimized/core fixtures;
4. graph materialization checks;
5. cross-engine conformance runs;
6. formal/checker projections;
7. innovation opportunity profiles.

Supporting documents:

1. `docs/spec/core-engine/w048-cycles/W048_TEST_CORPUS_AND_CONFORMANCE_PLAN.md`
2. `docs/spec/core-engine/w048-cycles/W048_INNOVATION_OPPORTUNITY_LEDGER.md`

## 12. Successor Routing Gate

Successor work should not deepen or reorganize cycle behavior until W048 provides:

1. selected non-iterative cycle policy;
2. materialized graph contract and artifacts or exact blockers;
3. structural cycle fixture evidence;
4. CTRO-created cycle fixture evidence;
5. cycle release/re-entry fixture evidence or exact blocker;
6. Excel observation disposition for core probes;
7. iterative-profile selection or explicit deferral;
8. W048 formal definitions/models/checker targets grounded in W048 artifacts;
9. test corpus run evidence for the declared scope;
10. innovation ledger entries separated from default Excel-match behavior.

## 13. Non-Claims

This packet does not claim:

1. Excel-compatible circular-reference closure;
2. iterative calculation support;
3. dynamic-array/spill cycle support;
4. data-table cycle compatibility;
5. pack/C5/operated-service readiness;
6. formal proof of SCC/cycle equivalence.

It records the next engineering and evidence surface needed to make cycle behavior deterministic and comparable before W049 formalization.
It also records implementation, test, and formal-artifact scope owned by W048 itself.

## 14. Status Surface

- execution_state: `closure_audit_passed`
- scope_completeness: `scope_complete`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- prerequisites:
  - W047 CTRO design and bounded implementation-core progress sufficient to run structural and CTRO-created cycle probes
  - Excel host availability for black-box observation packets
- bead_path: `calc-zci1`
- exit_gate: satisfied by `docs/spec/core-engine/w048-cycles/W048_CLOSURE_AUDIT_AND_SUCCESSOR_ROUTING.md`
- evidence_policy: public docs plus reproducible black-box Excel probes plus TraceCalc/TreeCalc artifacts plus W048 checkers
- closure_audit: `docs/test-runs/core-engine/w048-closure-audit-001/w048_closure_audit_summary.json`
- open_lanes: []
