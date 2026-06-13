# W061: Strict Excel Grid Planning And Reference Floor

## Purpose

Promote the grid-extension planning set into OxCalc-owned implementation surfaces and create the first executable reference floor for `strict-excel-grid`.

## Depends on

- W047 CTRO design and positive-publication lessons.
- W050 plan-template/session direction.
- W060 host reference system and `ReferenceSystemProvider` shape.
- OxFml W077 for `BindProfile`, symbolic references, and R1C1 formula identity.
- OxDoc bootstrap for `.xlsx` event-stream contracts.

## Spec surfaces

- `docs/spec/core-engine/CORE_ENGINE_GRID_MODEL.md`
- `docs/spec/core-engine/CORE_ENGINE_GRID_REFINEMENT_AND_EQUIVALENCE.md`
- `docs/spec/core-engine/CORE_ENGINE_GRID_REFERENCE_MACHINE.md`
- `docs/spec/core-engine/CORE_ENGINE_GRID_PERF_REGISTER.md`

## Closure condition

W061 closes when the grid semantic docs are indexed, GridCalc-Ref has a first BTreeMap reference-machine implementation for bounds/R1C1/materialization cases, the differential harness can run `--engine reference|optimized|both` for a small grid corpus, the perf register emits counter assertions for the first touched rows, and all `[verify-COM]` spill/hidden-row claims have explicit OxXlPlay capture beads or blockers.

## Initial lanes

1. GridCalc-Ref BTreeMap state and sampled readout.
2. Grid corpus seed: bounds, normal-form translation, materialization, insert/delete.
3. OxXlPlay COM capture prerequisites for spill and hidden rows.
4. Perf counters and register assertions (`P-00`, `P-10`, `P-11`, `P-19`).
5. Spill reference floor: ledger, blockage watches, `A1#` provider path.
6. Hidden-row floor: AxisState provenance, `GridHostInfoProvider`, visibility invalidation.
7. Structural edits over formulas, tables, merged regions, and future feature-rendered regions.

## Rollout mode

`planning_promoted_reference_floor_next`