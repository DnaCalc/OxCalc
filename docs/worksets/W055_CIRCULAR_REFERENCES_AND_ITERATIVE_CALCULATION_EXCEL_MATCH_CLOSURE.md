# W055 Circular References And Iterative Calculation Excel-Match Closure

Status: `in_progress`

Parent predecessor: `W048` single-host circular-reference scope

Parent epic: `calc-9ouy`

## 1. Purpose

W055 turns the W048 single-host fixture slice into a product-grade circular
reference and iterative calculation feature area.

The first goal is not to claim every hard Excel case at once. The first goal is
to replace fixture-keyed behavior with a general profile-driven cycle engine,
then expose the needed host-facing cycle config and diagnostics, then widen the
Excel evidence family by family.

Formal proof is not required for the product claim. Formal obligations must be
recorded separately for W049 or a successor proof lane.

## 2. Current Product Status

W048 supports declared circular-reference and iterative fixtures for one
observed Excel host/version.

W055 has not yet implemented broader product scope.

Current limitation:

1. cross-version Excel behavior is not claimed,
2. W048 iterative publication is fixture/probe driven,
3. dynamic-array spill cycles, data tables, external workbook link cycles, and
   broad thread variants are not product-supported yet.

## 3. Work Tranches

### Tranche A — General Cycle Engine

This is the first implementation tranche.

In scope:

1. direct self-references,
2. two-node and larger structural cycles,
3. CTRO/dynamic-reference cycles already represented by W048-style evidence,
4. iterative calculation with `MaxIterations` and `MaxChange`,
5. member order and initial-vector rules as profile data,
6. profile-specific terminal states and publish/reject rules,
7. atomic publication or no-publication rejection,
8. downstream invalidation and recomputation after release or publication,
9. replay-visible cycle-region and iteration summaries,
10. typed OxCalcTree host input for cycle profile and iterative bounds,
11. typed OxCalcTree result diagnostics for circular-reference outcomes.

Acceptance requires replacing fixture-keyed terminal values with a general
algorithm for the declared scope.

### Tranche B — Excel Observation Matrix

This tranche widens evidence.

Each family must get observations, an accepted blocker, or an explicit
exclusion before it can be included in a product claim:

1. edit-order and calculation-chain sensitivity,
2. blank, zero, numeric, text, logical, error, and prior-state initial vectors,
3. manual versus automatic recalculation,
4. full recalculation and workbook reopen behavior,
5. volatile and externally invalidated functions inside cycle regions,
6. least-significant-bit numeric parity for tested numeric surfaces.

### Tranche C — Hard Excel Families

These are not allowed to hide inside the general cycle claim.

Each family gets its own lane:

1. dynamic-array spill cycles,
2. data-table circular-reference behavior,
3. external workbook link cycles,
4. multi-threaded and cross-thread variants.

Each lane may close as implemented, blocked, or explicitly excluded from the
declared product scope.

### Downstream Host Contract

DnaTreeCalc requested the production contract for cycle profile selection,
iteration bounds, and returned circular-reference diagnostics.

W055 treats this as part of Tranche A, not as a later documentation cleanup.
The intake packet is:
`docs/spec/core-engine/w055-cycles/W055_DNATREECALC_CYCLE_CONFIG_HANDOVER_INTAKE.md`.

The contract packet is:
`docs/spec/core-engine/w055-cycles/W055_HOST_CONTRACT_TERMINAL_SEMANTICS_AND_PARITY_GATES.md`.

Current disposition:

1. `compatibility_basis` may describe fixture provenance, but it is not the
   final cycle config API,
2. the production input field is `OxCalcTreeContext recalculation configuration.cycle_config`,
3. the production result field is `OxCalcTreeCalculationOutcome.cycle_diagnostics`,
4. DnaTreeCalc iterative cycle corpus cases should stay pending until the typed
   implementation and acceptance evidence land.

### Parity Levels

W055 reports Excel parity only by declared scope.

Allowed claims:

1. `Excel-faithful (covered surfaces)` for the current W048/W055 single-host
   covered surfaces,
2. `Tranche A Excel parity` after the general engine replaces fixture-keyed
   behavior for the declared Tranche A surface and conformance passes,
3. hard-family parity only after each separate hard-family lane closes.

W055 does not claim broad Excel parity for all circular-reference behavior until
cross-version, thread, dynamic-array, data-table, and external-link dimensions
are implemented, blocked with accepted limitation, or explicitly excluded from
the named product scope.

## 4. Dependencies

Required now:

1. `W048` for cycle vocabulary and single-host evidence,
2. `W050` for the formula-authority and prepared-runtime seam.

Required only for some lanes:

1. `W051` for sparse/range-backed behavior, data-table-adjacent range surfaces,
   and any spill/range fixture that needs sparse readers,
2. OxFml for formula/evaluator behavior, dynamic arrays/spills, external
   references, and replay surfaces,
3. OxFunc for function semantics, volatility, numeric precision, external
   invalidation, and data-table-adjacent kernel behavior.

Foundation involvement is needed only if profiles or conformance-pack policy
change.

## 5. First Work

The first W055 beads are:

1. `calc-9ouy.1` rollout, scope, and evidence root,
2. `calc-9ouy.2` general cycle-engine design,
3. `calc-9ouy.3` fixture-keyed iterative implementation replacement,
4. `calc-9ouy.4` Tranche A conformance and replay evidence,
5. `calc-9ouy.5` Excel observation matrix,
6. `calc-9ouy.6` hard Excel family lanes,
7. `calc-9ouy.7` formalization handoff packet,
8. `calc-9ouy.8` DnaTreeCalc cycle config host contract,
9. `calc-9ouy.9` OxCalcTree typed cycle config and diagnostics implementation,
10. `calc-9ouy.10` DnaTreeCalc cycle bridge acceptance evidence.

Suggested artifact roots:

1. `docs/test-runs/excel-cycles/w055-*`
2. `docs/test-runs/core-engine/w055-cycles/`

The first rollout bead may refine these roots before evidence is emitted.

The rollout/scope packet is
`docs/spec/core-engine/w055-cycles/W055_TRANCHE_A_ROLLOUT_AND_SCOPE.md`.

The DnaTreeCalc handover intake packet is
`docs/spec/core-engine/w055-cycles/W055_DNATREECALC_CYCLE_CONFIG_HANDOVER_INTAKE.md`.

The host contract and parity-gate packet is
`docs/spec/core-engine/w055-cycles/W055_HOST_CONTRACT_TERMINAL_SEMANTICS_AND_PARITY_GATES.md`.

## 6. Closure Gate

W055 can claim product support only for the declared scope.

For that scope, closure requires:

1. supported cycle modes named by profile,
2. Excel observations or accepted blockers for every included scenario family,
3. TraceCalc and TreeCalc/core matching the accepted observation set,
4. a general implementation path, not fixture-keyed result tables,
5. numeric precision stated and checked where claimed,
6. dynamic arrays, data tables, external links, and thread variants either
   implemented, blocked, or explicitly excluded,
7. typed host-facing cycle config and result diagnostics specified and
   implemented,
8. DnaTreeCalc bridge or handoff evidence shows the typed contract satisfies the
   downstream ask,
9. profile selectors and capability manifests updated,
10. spec and replay contracts updated,
11. proof/model obligations packetized separately,
12. final report states product status, evidence, still-open work, and formal
    status separately.

## 7. Status

Product status: W048 single-host fixture scope is supported; W055 product scope
is not implemented yet.

Evidence: W048 Excel observations, TraceCalc fixtures, TreeCalc/core fixtures,
and conformance checks are the starting evidence.

Still open: general cycle engine, typed OxCalcTree cycle config/result contract,
typed Rust facade implementation, DnaTreeCalc acceptance evidence, Excel
observation matrix, hard Excel-family lanes, conformance comparison, spec/replay
updates, formalization handoff.

Formal status: no W055 proof claim.
