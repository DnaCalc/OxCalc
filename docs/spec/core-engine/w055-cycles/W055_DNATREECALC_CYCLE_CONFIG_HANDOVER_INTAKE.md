# W055 DnaTreeCalc Cycle Config Handover Intake

Status: `active_w055_intake`

Source handover:
`C:/Work/DnaCalc/DnaTreeCalc/docs/handovers/HANDOVER_OXCALC_iterative_cycle_config.md`

Owner bead: `calc-9ouy.8`

Parent workset:
`docs/worksets/W055_CIRCULAR_REFERENCES_AND_ITERATIVE_CALCULATION_EXCEL_MATCH_CLOSURE.md`

## 1. Downstream Ask

DnaTreeCalc now has a live Rust bridge to the OxCalc tree consumer facade.
The acyclic smoke path is enough for first integration checks, but cycle
configuration is not yet a production host contract.

DnaTreeCalc needs W055 to answer four things:

1. how a host selects the cycle profile,
2. where `Maximum Iterations` and `Maximum Change` are submitted,
3. which typed cycle diagnostics come back on the result,
4. which honest coverage label should be used for `cycle.excel_match_iterative`.

## 2. Current OxCalc Surface

The current OxCalc tree facade does not yet have a typed production cycle
configuration field.

Current facts:

1. `OxCalcTreeContext recalculation configuration` carries `candidate_result_id`,
   `publication_id`, `compatibility_basis`, and `artifact_token_basis`.
2. `OxCalcTreeHostCapabilitySnapshot` carries `capability_profile_id` and
   runtime-effect booleans, but no cycle profile or iteration bounds.
3. `OxCalcTreeRuntimePolicy` controls diagnostics, overlays, derivation traces,
   and scheduling policy, but no cycle semantics.
4. `OxCalcTreeCalculationOutcome` exposes reject detail, dependency graph,
   invalidation closure, evaluation order, published values, node states, and
   string diagnostics, but no typed cycle diagnostic records.

W048 TreeCalc fixtures opt into `cycle.excel_match_iterative` through
`compatibility_basis` and fixture/probe identifiers. W055 must not treat that
fixture admission path as the final production host API.

## 3. W055 Contract Direction

The production host contract is typed.

The exact contract packet is:
`docs/spec/core-engine/w055-cycles/W055_HOST_CONTRACT_TERMINAL_SEMANTICS_AND_PARITY_GATES.md`

W055 carries the cycle profile and iterative bounds as structured per-recalc
input on:

`OxCalcTreeContext recalculation configuration.cycle_config`

Rationale:

1. cycle profile and bounds affect the result of a specific recalculation,
2. a host may change workbook or workspace options between recalculations,
3. `capability_profile_id` describes host capability, not selected cycle
   semantics,
4. `compatibility_basis` should remain correlation/provenance text, not the
   long-term semantic configuration channel.

The host-facing input contract must name:

1. `cycle_profile_id`,
2. `maximum_iterations`,
3. `maximum_change`.

Default behavior:

1. absent cycle config uses `cycle.non_iterative_stage1`,
2. iterative profiles default to `maximum_iterations = 100`,
3. iterative profiles default to `maximum_change = 0.001`,
4. hosts may override both bounds when they submit cycle config,
5. the W048 stop metric remains maximum absolute visible numeric delta across
   cycle members unless W055 evidence falsifies it.

Profiles to preserve in the contract:

1. `cycle.non_iterative_stage1`,
2. `cycle.excel_match_iterative`,
3. `cycle.iterative_deterministic_v0`.

## 4. Result And Diagnostic Surface

W055 must add a typed host-visible cycle result surface. Hosts should not have
to scan string diagnostics to understand circular-reference behavior.

The result surface must cover:

1. cycle-region membership,
2. region source such as structural graph or CTRO/dynamic reference,
3. root/report node equivalent to Excel `Worksheet.CircularReference`,
4. member order used for iteration or diagnostics,
5. cycle profile used for the region,
6. `cycle_iteration_trace` summary,
7. terminal classification.

The iteration trace summary must include:

1. submitted max iterations,
2. submitted max change,
3. iteration count,
4. initial vector,
5. terminal vector,
6. terminal state.

Terminal classification must at least distinguish:

1. `converged`,
2. `max_iteration`,
3. `oscillation`,
4. `divergent`.

W055 must also decide whether non-iterative rejection and nonnumeric/error
iteration rejection are represented as terminal states or as reject reasons. The
contract packet records the target answer: both paths must be typed and directly
reachable from `OxCalcTreeCalculationOutcome.cycle_diagnostics`.

## 5. Coverage Label

DnaTreeCalc may use this label for the current W048 evidence floor:

`Excel-faithful (covered surfaces)`

The label must stay tied to the W048 single-host covered-fixture scope.

Covered by W048:

1. one observed Excel host/version,
2. declared Excel iterative fixtures,
3. TraceCalc and TreeCalc/core matching fixtures,
4. worksheet-scoped root/report-cell observations for declared non-iterative
   probes,
5. numeric, blank, text, and error prior-state lanes for declared probes,
6. materialized graph sidecars for the declared TreeCalc cycle fixtures.

Not covered by that label:

1. broad cross-version Excel behavior,
2. broad multithread or cross-thread behavior,
3. dynamic-array spill cycles,
4. data-table cycles,
5. external workbook link cycles,
6. any W055 Tranche A behavior not yet implemented by the general cycle engine.

## 6. DnaTreeCalc W002 Disposition

DnaTreeCalc W002 should keep iterative cycle corpus cases pending until W055
lands `OxCalcTreeContext recalculation configuration.cycle_config` and
`OxCalcTreeCalculationOutcome.cycle_diagnostics`.

A narrow non-iterative cycle smoke may be used only as a transitional facade
observation if it does not claim the final production cycle contract. The
active W002 corpus should not require hosts to encode cycle profile or bounds in
`compatibility_basis`.

## 7. W055 Integration

This handover is now part of W055 through `calc-9ouy.8`.

`calc-9ouy.8` is a prerequisite to the general cycle-engine design because the
engine design must expose a host contract, not only internal cycle mechanics.

Closure for this lane requires:

1. the production input location and field names are specified,
2. defaults and override rules are specified,
3. typed result and diagnostic fields are specified,
4. the Rust consumer facade implementation lane is explicit,
5. DnaTreeCalc receives a clear answer on which cycle corpus cases may activate.
