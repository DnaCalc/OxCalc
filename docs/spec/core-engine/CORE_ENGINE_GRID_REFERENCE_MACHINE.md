# CORE_ENGINE_GRID_REFERENCE_MACHINE

Status: **Promoted planning spec** (2026-06-13). Companion to `CORE_ENGINE_GRID_MODEL.md`.

## 1. Purpose

GridCalc-Ref is the simple-correct reference implementation for the strict Excel grid profile. It is product test code, not disposable scaffolding. Optimized grid code proves itself by differential execution or by closed-form checks against this reference where full differential scale is too large.

## 2. State model

GridCalc-Ref represents each sheet as ordinary maps:

- authored cells: `BTreeMap<(row, col), CellState>`;
- axis props: per-row/per-column `BTreeMap` or run-expanded maps;
- tables, merged regions, and feature-rendered regions as explicit rect records;
- computed valuation as a separate `BTreeMap<(row, col), CalcValue>`;
- spill records as a plain anchor-keyed map recomputed during recalc.

It deliberately does not use block storage, template coalescing, persistent graph compression, interval indexes, tile streaming, or optimized publication structures.

## 3. Recalc algorithm

The first reference floor is mark-all-dirty:

1. bind/evaluate each occupied formula independently through the ordinary OxFml/OxFunc stack;
2. recompute spill placement in deterministic spec order;
3. iterate bounded spill repair passes using the same cap as the semantic spec;
4. publish computed values and committed effects after the pass quiesces or reaches the spill cap.

The reference machine may share leaf formula/function evaluation with the optimized engine. It may not share optimized storage, graph, invalidation, or publication machinery.

## 4. Readout and differential contract

The differential harness runs one scenario against GridCalc-Ref and the optimized engine, then compares:

- all occupied authored cells;
- all committed spill extents and blocked-anchor facts;
- boundary probes around row 1, row 1,048,576, col 1, col 16,384, block edges, and sampled blanks;
- invalidation closure for small and medium cases;
- declared feature-rendered-region flags once admitted.

Full differential is capped at the reference budget. Above that cap, optimized runs use closed-form workload expectations and sampled readout cones.

## 5. Initial corpus families

- bounds and `#REF!` translation;
- R1C1 normal-form fill/copy equivalence;
- template materialization and punch-through overrides;
- insert/delete over formulas, ranges, tables, and merged regions;
- spill blockage, clearance, contraction, mutual blockage, `A1#`, hidden-row placement;
- hidden/manual/filter/outline visibility-sensitive aggregates;
- pivot/feature-rendered-region edit refusals and needs-refresh flags, reserved until admitted.

## 6. Implementation sequence

1. Build the BTreeMap sheet state and sampled readout.
2. Add formula evaluation through existing OxFml/OxFunc leaves.
3. Add bounds/reference adjustment and template materialization scenarios.
4. Add spill fixpoint and visibility AxisState floors.
5. Wire `--engine reference|optimized|both` into the existing scale/differential runner.