# W048 Cycle Literature And Decision Map

Status: `active_execution_workset`

## 1. Working Question

The central W048 question is not merely whether a dependency graph contains an SCC. OxCalc must decide which deterministic choices inside a cyclic region can affect observable results and how those choices become explicit, replayable, and comparable with Excel.

The term `cycle_rooting` is used here for the family of choices that answer:

1. which cycle member is treated as the first member for reporting, ordering, or iteration;
2. which existing value is used as the starting value for each member;
3. whether cycle members update from a snapshot vector or from values updated earlier in the same pass;
4. how an SCC's members and boundary dependents re-enter calculation after the cycle is released.

For non-iterative cycle rejection, root/order is mostly diagnostic and replay-visible. For iterative calculation, root/order can affect values when the update model is sequential or when floating-point cutoff and prior values participate.

## 2. Public Excel Evidence

Public Microsoft documentation establishes a comparison baseline but does not fully specify the algorithm.

### 2.1 Recalculation Model

Microsoft documents Excel recalculation as a three-stage process:

1. construct a dependency tree;
2. construct a calculation chain;
3. recalculate cells.

The dependency tree records precedent/dependent relations. The calculation chain lists formula cells in the order Excel should calculate them. During recalculation Excel can revise the chain when it encounters a formula whose precedent has not yet been calculated. Excel also marks direct and indirect dependents dirty after changes.

W048 consequence:

1. Excel has a deterministic operational state model, not just a pure formula graph.
2. Calculation-chain order is a candidate explanation for cycle reporting and iteration order.
3. Probes must record whether `Calculate`, `CalculateFull`, and dependency-tree rebuild commands produce the same observed root/order.

Source: `https://learn.microsoft.com/en-us/office/client-developer/excel/excel-recalculation`

### 2.2 Calculation Chain Persistence

Microsoft documents calculation-chain metadata saved to a workbook. The chain tracks calculation order, can take multiple edits/calculations to settle into an optimized state, and may have multiple valid states for a given workbook.

W048 consequence:

1. Excel-compatible determinism may be deterministic over workbook state plus calculation-chain metadata, not over formula text alone.
2. Observation packets must include workbook provenance and saved calculation-chain state where available.
3. Cold-open and full-rebuild variants are required because they may change chain state.

Source: `https://support.microsoft.com/en-us/office/excel-calculation-chain-metadata-6e1b5819-6abd-4e94-bff5-838d4c576e01`

### 2.3 Circular Reference Surface

Microsoft documents direct and indirect circular references, circular-reference warnings, status-bar/navigation behavior, and the non-iterative display behavior where Excel may show zero or retain the last successful calculated value.

W048 consequence:

1. A non-iterative cycle does not necessarily erase the prior displayed value in Excel.
2. OxCalc must separate publication semantics from display/retention semantics.
3. Last-successful-value behavior must be probed before OxCalc claims Excel compatibility for non-iterative cycles.

Source: `https://support.microsoft.com/en-us/office/remove-or-allow-a-circular-reference-in-excel-8540bd0f-6e97-4483-bcf7-1b49cd50d123`

### 2.4 Iterative Calculation Surface

Microsoft documents iterative calculation as repeated recalculation until a numeric condition is met. Excel normally has iterative calculation off. When iterative calculation is enabled with default settings, Excel stops after 100 iterations or after all values in the circular reference change by less than 0.001 between iterations, whichever comes first. VBA exposes `Application.Iteration`, `Application.MaxIterations`, and `Application.MaxChange`.

W048 consequence:

1. OxCalc's future iterative profile needs an explicit bounded/threshold terminal policy.
2. The public docs do not specify member order, root selection, initial-value vector, or exact change metric details for all value types.
3. Probes must infer those details under controlled workbooks.

Sources:

1. `https://support.microsoft.com/en-gb/office/change-formula-recalculation-iteration-or-precision-in-excel-73fc7dac-91cf-4d36-86e8-67124f6bcce4`
2. `https://learn.microsoft.com/en-us/office/vba/api/excel.application.iteration`
3. `https://learn.microsoft.com/en-us/office/vba/api/excel.application.maxiterations`
4. `https://learn.microsoft.com/en-us/office/vba/api/excel.application.maxchange`

### 2.5 Data Table Exception

Microsoft documents data tables as a special recalculation structure where circular references may be tolerated differently and data tables do not use multi-threaded calculation.

W048 consequence:

1. Data-table cycle behavior is not a TreeCalc baseline.
2. A future host may admit a distinct data-table profile, but W048 should not mix it into general formula SCC semantics.

Source: `https://learn.microsoft.com/en-us/office/client-developer/excel/excel-recalculation`

## 3. Literature And Foundation Intake

### 3.1 Jane Street Incremental

Jane Street's public Incremental material frames spreadsheet-style recomputation as graph-structured self-adjusting computation. The public interface describes a DAG, observed/necessary nodes, topological stabilization, a recompute heap ordered by height, dynamic graph changes through `bind`, and configurable cutoff.

Useful W048 transfers:

1. `necessary` and `observed` distinguish full graph truth from demanded stabilization work.
2. `stale`, cutoff, and stabilization vocabulary help make incremental recalculation explicit.
3. Dynamic dependencies should be graph updates with explicit scope/lifecycle, not hidden formula side effects.
4. Graph export/analyzer ideas support W048's materialized graph sidecar requirement.

Important limit:

1. Incremental requires an acyclic graph for ordinary stabilization. It informs OxCalc graph discipline, but it does not provide Excel circular-reference semantics.

Sources:

1. `https://blog.janestreet.com/introducing-incremental/`
2. `https://github.com/janestreet/incremental`

### 3.2 Self-Adjusting Computation And Adapton

The self-adjusting computation literature emphasizes consistency under mutation and memoized change propagation. Foundation notes map this to OxCalc as dynamic-dependency soundness, from-scratch consistency, and replay obligations.

Useful W048 transfers:

1. CTRO dynamic dependencies must be observationally equivalent to a from-scratch effective graph for the declared profile.
2. Dynamic edge activation/release must not under-invalidate dependents.
3. Candidate graph facts must be replay-visible so a later checker can distinguish semantic changes from scheduler choices.

Sources:

1. `https://arxiv.org/abs/1106.0478`
2. `https://arxiv.org/abs/1609.05337`
3. `../Foundation/research/runs/20260305-201430-dag-computation-theory-deep-research-pass-01/outputs/10_conformance_and_proof_obligations.md`

### 3.3 Build Systems A La Carte

Build Systems a la Carte separates tasks, stores, schedulers, and rebuilders. It explicitly notes that many build-system correctness definitions assume acyclic tasks, while iterative computations need bounded execution or fixed-point discipline.

Useful W048 transfers:

1. Separate semantic target from scheduling mechanics.
2. Treat cycles as profile-governed behavior, not as accidental scheduler loops.
3. Use explicit stores and traces when comparing incremental results against a from-scratch model.

Source: `https://simon.peytonjones.org/assets/pdfs/build-systems-original.pdf`

### 3.4 SCC, Fixed-Point, And Dynamic Topology Work

Foundation research already promotes deterministic topo/SCC as the baseline skeleton and fixed-point theory as a possible future semantic model for monotone cycle profiles.

Useful W048 transfers:

1. Tarjan/Kahn-style graph algorithms are adequate for baseline full graph classification.
2. Dynamic topological/cycle maintenance is a performance lane, not the first semantic authority.
3. Tarski-style fixed points can justify some monotone iterative profiles, but Excel compatibility is operational and must be probed.

Foundation anchors:

1. `../Foundation/notes/THEORY_TO_PACK_REGISTER.md` `TH-001`
2. `../Foundation/research/runs/20260305-201430-dag-computation-theory-deep-research-pass-01/outputs/03_algorithm_family_map.md`
3. `../Foundation/research/runs/20260305-201430-dag-computation-theory-deep-research-pass-01/outputs/04_dnacalc_transfer_matrix.md`
4. `../Foundation/research/runs/20260305-201430-dag-computation-theory-deep-research-pass-01/outputs/05_deep_research_synthesis.md`

## 4. W048 Decision Axes

### 4.1 Detection Graph

Options:

1. `G_struct`: structural-derived graph before runtime effects.
2. `G_eff`: structural graph plus published overlay.
3. `G_eff_candidate`: structural graph plus candidate overlay for the current wave.

W048 direction:

1. Classify SCCs over each graph layer using the same deterministic relation.
2. Preserve layer/provenance in diagnostics.
3. Do not create a separate semantic class for CTRO-created cycles; use provenance plus the shared cycle policy.

### 4.2 Cycle Region Identity

Required fields:

1. stable `cycle_region_id`;
2. graph layer and graph hash;
3. sorted member list;
4. member internal edges;
5. incoming boundary edges;
6. outgoing boundary edges;
7. provenance of edges that make the SCC cyclic;
8. root/order policy used for diagnostics or iteration.

### 4.3 Rooting Policy

Candidate policies:

| Policy | Meaning | Excel-match risk |
| --- | --- | --- |
| `canonical_node_id_root` | smallest stable node id in the SCC | deterministic but may diverge from Excel display/report order |
| `worksheet_scan_root` | first member by workbook sheet/range scan | plausible for simple grid probes, weak for tree hosts |
| `calculation_chain_root` | first member by current chain state | likely important for Excel compatibility, but requires chain-state observation |
| `edit_admission_root` | member associated with most recent edit/dirty seed | may explain warning/report cell behavior |
| `first_back_edge_root` | first root emitted by classifier traversal | deterministic only if traversal is fixed; poor semantic surface |
| `no_iteration_root` | root is diagnostic only under non-iterative reject | acceptable for Stage 1 no-publication profile |

W048 direction:

1. Use canonical ordering for OxCalc diagnostics unless Excel probes justify an Excel-compatible root profile.
2. For iterative Excel-compatible profile, keep root/order as a declared field, not a hidden property of SCC traversal.

### 4.4 Initial Value Policy

Candidate policies:

1. `published_prior_value`: start from last published values for cycle members.
2. `excel_display_prior_value`: start from observed display values including retained last-successful values.
3. `zero_or_blank_default`: start from type default where no prior value exists.
4. `candidate_prepass_value`: use values produced before the SCC is recognized.
5. `profile_error`: no iteration allowed; prior values may remain display-only.

Open W048 probe:

1. Determine how Excel initializes direct self-cycles, indirect cycles, guarded cycles, cold-open cycles, and newly introduced dynamic cycles.

### 4.5 Update Model

Candidate policies:

1. `jacobi_snapshot`: each iteration reads the prior iteration vector.
2. `gauss_seidel_ordered`: each member reads values updated earlier in the same iteration order.
3. `excel_chain_ordered`: sequential update in current Excel chain order.
4. `region_function_iteration`: evaluate the SCC as a pure vector function with explicit snapshot boundaries.

W048 note:

1. `jacobi_snapshot` reduces root/order sensitivity.
2. `gauss_seidel_ordered` and `excel_chain_ordered` can make root/order observable.
3. Excel probes should include order-sensitive formulas that distinguish these models.

### 4.6 Terminal Policy

Required profile fields:

1. `max_iterations`;
2. `max_change`;
3. value-domain eligibility;
4. change metric;
5. divergence/oscillation disposition;
6. max-iteration terminal disposition;
7. trace event sequence.

Excel docs establish the existence of `max_iterations` and `max_change`, but not all edge details.

### 4.7 Publication Boundary

Candidate policies:

1. `whole_wave_reject`: reject the candidate bundle when any candidate graph SCC violates the profile.
2. `cycle_region_reject`: reject cycle-region values while publishing acyclic candidates outside the region.
3. `frontier_partition_publish`: publish candidates whose dependency cone is independent from the rejected cycle.
4. `display_retention_only`: leave previously published/displayed values visible but publish no new value for the rejected region.

W048 Stage 1 direction:

1. Keep `whole_wave_reject` for CTRO-created cycles until a narrower rule has deterministic replay evidence.
2. Preserve prior published values separately from candidate rejection.
3. Model Excel retained-display behavior as an observation target, not as assumed OxCalc publication behavior.

### 4.8 Engine Implementation Surface

W048 owns the implementation consequences of these decisions.

Required implementation surfaces:

1. TraceCalc reference behavior for each selected cycle policy;
2. TreeCalc optimized/core behavior that conforms to TraceCalc for declared cases;
3. materialized graph sidecars from both engines where applicable;
4. deterministic test corpus and run artifacts;
5. formal definitions/models that name the same graph, cycle-region, terminal, and publication states.

W048 is not merely preparing a target for a later formalization workset. W048 introduces the cycle definitions and first formal artifacts required to make its own implementation claims reviewable.

### 4.9 Release And Re-Entry

Release occurs when a later graph layer no longer contains the SCC.

Required decisions:

1. which nodes become dirty/needed after release;
2. whether release requires full graph rebuild or local frontier repair;
3. whether downstream dependents read prior values or wait for recomputation;
4. how previous cycle diagnostics are retired or retained;
5. how release is replayed when the prior candidate overlay was rejected.

W048 direction:

1. Treat release as a graph transition with explicit invalidation seeds.
2. Downstream dependents must be covered by reverse-reachability from changed or released edges.
3. A rejected candidate overlay must not become the published basis for release.

## 5. Provisional W048 Vocabulary

| Term | Meaning |
| --- | --- |
| `cycle_region` | Non-trivial SCC or self-loop SCC in a materialized dependency graph layer. |
| `cycle_source` | Discovery provenance: `structural`, `published_overlay`, or `candidate_overlay`. |
| `cycle_root` | Declared first member for diagnostic or iteration purposes under a root policy. |
| `cycle_member_order` | Stable ordered member list used by diagnostics or iteration. |
| `cycle_boundary` | Incoming and outgoing edges between the SCC and the rest of the graph. |
| `cycle_terminal_policy` | Rule that maps a cycle region to reject, prior-value retention, iteration, or another terminal state. |
| `cycle_release` | Transition where a previous cycle region is absent from the new effective graph basis. |
| `cycle_reentry` | Recalculation of previously blocked cycle members and downstream dependents after release. |
| `cycle_profile` | Named behavior contract that binds detection graph, root/order, initial values, terminal policy, publication policy, and trace/formal facts. |
| `excel_match_profile` | Cycle profile intended to reproduce observed Excel behavior for a declared probe set. |
| `oxcalc_innovation_profile` | Cycle profile intentionally offering behavior beyond Excel, never used as default Excel compatibility without explicit opt-in. |

## 6. Current Status Surface

- execution_state: `in_progress`
- scope_completeness: `scope_partial`
- target_completeness: `target_partial`
- integration_completeness: `partial`
- open_lanes:
  - Excel root/order and iteration observations
  - graph materialization sidecar contract
  - TraceCalc and TreeCalc fixtures
  - W048 proof/model definitions and checks
  - innovation profile ledger
