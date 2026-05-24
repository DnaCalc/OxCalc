# W055 Tranche A Rollout And Scope

Status: `active_rollout`

Parent workset: `docs/worksets/W055_CIRCULAR_REFERENCES_AND_ITERATIVE_CALCULATION_EXCEL_MATCH_CLOSURE.md`

Parent epic: `calc-9ouy`

Rollout bead: `calc-9ouy.1`

## 1. Product Scope

Tranche A starts W055 with the general cycle engine.

The first product scope is:

1. direct self-references,
2. two-node and larger structural cycles,
3. CTRO/dynamic-reference cycles already represented by W048 evidence,
4. iterative calculation with profile-declared `MaxIterations` and `MaxChange`,
5. member order and initial-vector rules represented as profile data,
6. profile-specific terminal states and publish/reject rules,
7. atomic cycle-region publication when the profile publishes terminal values,
8. no-publication rejection when the profile rejects the candidate,
9. downstream invalidation and recomputation after cycle release or publication,
10. replay-visible cycle-region and iteration summaries,
11. host-facing cycle config and typed diagnostics for the OxCalcTree facade.

The first scope does not claim dynamic-array spill cycles, data tables, external
workbook link cycles, broad cross-version Excel behavior, or broad thread
variants.

## 2. Starting Evidence

W048 is the starting evidence floor:

1. Excel observations under one observed Excel host/version,
2. TraceCalc W048 cycle fixtures,
3. TreeCalc/core W048 cycle fixtures,
4. materialized graph sidecars,
5. W048 conformance summary with `passed_single_host_scoped`.

W055 may reuse this evidence as predecessor evidence, but it must not present
the W048 fixture-keyed implementation as the final W055 implementation.

## 3. Artifact Roots

Canonical W055 roots:

1. Excel observations: `docs/test-runs/excel-cycles/w055-*`
2. Core/TraceCalc/TreeCalc evidence: `docs/test-runs/core-engine/w055-cycles/`
3. Spec/design packet: `docs/spec/core-engine/w055-cycles/`

Artifacts under those roots are checked in only when they become durable
evidence. Local scratch runs should stay outside the active evidence path.

## 4. Bead Map

| Bead | Purpose | Gate |
| --- | --- | --- |
| `calc-9ouy.1` | rollout, scope, evidence root | W055 docs/register point to `calc-9ouy`, Tranche A scope and roots are explicit |
| `calc-9ouy.2` | general cycle-engine design | order, initial vector, update model, stop metric, terminal state, publish/reject, downstream invalidation, and replay fields are specified |
| `calc-9ouy.3` | replace fixture-keyed implementation | W048 probe-id terminal table is no longer the implementation path for declared Tranche A scope |
| `calc-9ouy.4` | Tranche A conformance and replay evidence | TraceCalc and TreeCalc/core match the accepted evidence set and emit replay-visible summaries |
| `calc-9ouy.5` | Excel observation matrix | each included family has observation, accepted blocker, or explicit exclusion |
| `calc-9ouy.6` | hard Excel family lanes | dynamic arrays, data tables, external links, and thread variants are split into explicit lanes |
| `calc-9ouy.7` | formalization handoff | formal obligations are packetized separately from product status |
| `calc-9ouy.8` | DnaTreeCalc cycle config host contract | production input fields, defaults, result diagnostics, coverage label, and W002 activation guidance are explicit |
| `calc-9ouy.9` | OxCalcTree typed facade implementation | request carries structured cycle config and result carries typed cycle diagnostics |
| `calc-9ouy.10` | DnaTreeCalc bridge acceptance evidence | DnaTreeCalc can submit cycle config and assert typed diagnostics/results without using `compatibility_basis` as config |

## 5. Downstream Handover Intake

DnaTreeCalc requested a production host-facing contract for cycle profile
selection, iterative bounds, and circular-reference diagnostics.

The intake packet is:
`docs/spec/core-engine/w055-cycles/W055_DNATREECALC_CYCLE_CONFIG_HANDOVER_INTAKE.md`

The contract packet is:
`docs/spec/core-engine/w055-cycles/W055_HOST_CONTRACT_TERMINAL_SEMANTICS_AND_PARITY_GATES.md`

Current W055 disposition:

1. the current `compatibility_basis` fixture path is not the final production
   cycle config API,
2. the production field is `OxCalcTreeContext recalculation configuration.cycle_config`,
3. typed cycle diagnostics must be directly reachable from
   `OxCalcTreeCalculationOutcome.cycle_diagnostics`,
4. DnaTreeCalc iterative cycle corpus cases stay pending until that implementation
   lands.

## 6. Dependency Split

Required now:

1. `W048`,
2. `W050`.

Required only for later lanes:

1. `W051` for sparse/range-backed cycle behavior,
2. OxFml for dynamic arrays, spills, external references, and replay surfaces,
3. OxFunc for function semantics, volatility, external invalidation, numeric
   precision, and data-table-adjacent kernel behavior.

## 7. Current Status

Product status: W055 is activated, but no new product support is claimed yet.
W048 single-host fixture scope remains the only supported circular-reference
evidence floor.

Evidence: W048 predecessor evidence is available and checked by existing W048
scripts. DnaTreeCalc handover intake is recorded as W055 design input.

Still open: Tranche A implementation, typed host-facing cycle config/result
implementation, DnaTreeCalc acceptance evidence, and conformance evidence work
after rollout.

Formal status: no W055 proof claim.
