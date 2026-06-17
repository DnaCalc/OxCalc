# CORE_ENGINE_GRID_REFINEMENT_AND_EQUIVALENCE

Status: **Promoted planning spec** (2026-06-13). Companion to `CORE_ENGINE_GRID_MODEL.md` and `CORE_ENGINE_GRID_REFERENCE_MACHINE.md`.

## 1. Purpose

This document defines how optimized grid implementations prove they refine the semantic model in `CORE_ENGINE_GRID_MODEL.md`. It owns the abstraction function, observation surfaces, and Invariant Register used by the grid perf register.

The relation is observational refinement, not trace equality. Optimized block storage, template regions, interval indexes, spill ledgers, visibility indexes, tile publication, and future feature-rendered regions may differ internally from the reference machine as long as the observation surfaces below match after abstraction.

## 2. Abstraction function

`alpha(OptimizedGridState) -> GridSpecState` expands implementation structures into the semantic state:

- sparse blocks flatten into the finite-support authored grid;
- template regions expand to per-cell `FormulaCell` entries carrying the R1C1-relative normal form;
- computed-layer spill body cells plus `SpillLedger` expand to derived support and spill facts;
- axis run storage expands to per-row/per-column `AxisProps` with manual/filter/outline provenance;
- table overlays expand to claimed grid rects and structured-reference slices;
- feature-rendered regions, such as future pivot reports, expand only to their claimed rect, writer class, edit-admission policy, and refresh/staleness flags until their semantics are admitted.

Epoch numerics, cache residency, wall-clock timings, and diagnostic wording are outside `alpha` unless a later spec names them as observation surfaces.

## 3. Observation surfaces

A conforming optimized engine must match the simple reference oracles at these surfaces:

1. **Value readout:** coordinate probes through the grid reader return the same
   values/errors/blanks as GridCalc-Ref.
2. **Invalidation closure:** the set of recomputed or dirtied semantic targets matches
   GridInvalidation-Ref, a scalar expanded dependency oracle, modulo allowed over-invalidation
   only where the row explicitly permits it.
3. **Committed effects:** `#REF!`, `#SPILL!`, spill extents, blocked-by facts,
   axis-visibility effects, and feature-rendered-region edit refusals match GridCalc-Ref and
   the spec.
4. **Materialization choices:** splitting, merging, or materializing virtual/template cells changes no observable value.

## 4. Invariant Register

| id | invariant | abstraction clause | oracle / check | status |
|---|---|---|---|---|
| I-GRID-1 | finite support: unoccupied blanks cost no authored state and read as blank | sparse blocks -> total grid | flatten blocks vs BTreeMap ref on sampled blank probes | claimed |
| I-GRID-2 | R1C1-relative normal form is formula identity | template expansion preserves normal-form text | translate/fill materialization differential | claimed |
| I-GRID-3 | materialization invariance | virtual/template cells expand to equivalent per-cell formulas | split/merge/materialize metamorphic cases | claimed |
| I-GRID-4 | schedule invariance | final valuation independent of conforming order | full vs visible-first vs reference schedule | claimed |
| I-SP1 | spill bodies never appear in authored state | authored grid excludes spill body support | authored-layer scan after spill storms | claimed |
| I-SP2 | spill ledger equals computed-layer support | `SpillLedger` <-> derived support | ledger expansion vs computed cells | claimed |
| I-SP3 | spill arbitration deterministic under spec order | conflicting anchors order by COM-pinned rule | mutual blockage corpus | claimed; `[verify-COM]` order pending |
| I-SP4 | no-edit quiesced recalc changes no spill extent | spill facts stable after warm pass | spill-storm warm recalc | claimed |
| I-SP5 | scalarizer equivalence for spill | ref fixpoint equals optimized values/errors/extents | GridCalc-Ref spill differential | claimed |
| I-VIS-1 | hidden toggle dirties exactly intersecting visibility consumers under Exact mode | visibility edges expand to row spans | scalarized visibility-edge compare | claimed |
| I-VIS-2 | hide/filter storms change no insensitive value | non-sensitive formulas ignore AxisState | metamorphic random toggle storm | claimed |
| I-VIS-3 | manual/filter/outline provenance remains separated | AxisState provenance expands per row | SUBTOTAL/AGGREGATE COM-pinned cases | claimed; `[verify-COM]` rule gaps pending |
| I-FRR-1 | feature-rendered regions are not ordinary formulas | writer class is explicit and outside recalc relation | pivot-report source-change sets needs-refresh, not recompute | reserved |

## 5. Property families

- Recalc idempotence: immediate no-edit recalc changes no value.
- Translation invariance: translating occupied content within bounds translates the valuation.
- Materialization invariance: template splitting/coalescing/forcing changes no value.
- Insert-then-delete identity: row/column edit pairs restore values and references where Excel does.
- Viewport schedule invariance: viewport visibility changes ordering/timing only, never dirty truth.

## 6. Register discipline

Every grid optimization requires:

1. an invariant row here;
2. a paired perf row in `CORE_ENGINE_GRID_PERF_REGISTER.md` when performance is claimed;
3. at least one GridCalc-Ref differential, scalarizer, or closed-form counter check;
4. explicit `[verify-COM]` status where Excel behavior is empirical rather than specified by existing function contracts.
