# W051 Sparse Range Readers And Defined-Entry Semantics

Status: `open_next`

Parent predecessor: `W050` formula-authority rework

Parent epic: allocate when W051 starts.

## 1. Purpose

W051 adds Excel-compatible sparse range reading for large worksheet ranges.

The target is ordinary formulas over whole-column and large-area references
without dense materialization. This is not a generic rich-data or virtual-array
workset.

The reader surface is:

1. `declared_extent`,
2. `defined_cardinality`,
3. `defined_iter`,
4. `read_at(coord) -> Defined(EvalValue) | Blank`,
5. `contains(coord)`.

## 2. Product Scope

In scope:

1. worksheet range/reference reads,
2. whole-column and large-area formulas,
3. defined entry versus blank behavior,
4. empty string as `Defined(Text(""))`, not blank,
5. deterministic defined-entry iteration order,
6. replay of declared extent and the defined-entry stream,
7. first aggregation functions: `SUM`, `COUNT`, `COUNTA`, and `COUNTBLANK`.

Out of scope:

1. generic virtual arrays,
2. arbitrary queryable host objects,
3. non-Excel rich producer protocols,
4. custom pushdown/filter engines.

Those ideas stay in
`docs/spec/core-engine/CORE_ENGINE_FUTURE_IDEAS_RICH_AND_VIRTUAL_DATA.md`.

## 3. Ownership

OxCalc owns the concrete sparse reader and backing adapter.

OxFml owns sparse binding through semantic plan, runtime prepared identity, and
replay projection.

OxFunc owns sparse argument-preparation metadata and function-kernel
consumption.

## 4. First Work

The first W051 beads should:

1. write the shared reader contract,
2. decide the coordinate and extent types,
3. decide the replay columns/artifacts,
4. implement an OxCalc reader over the current value/structure store or a
   clearly labeled fixture adapter,
5. activate the first OxFunc function group,
6. thread sparse input through OxFml runtime/replay,
7. run an integration scenario over at least one large range.

## 5. Evidence

W051 evidence must show:

1. declared extent,
2. defined-entry stream,
3. deterministic iteration order,
4. blank versus empty-string behavior,
5. final formula result,
6. no dense traversal for the covered large-range path.

Suggested artifact root:

`docs/test-runs/core-engine/w051-sparse-ranges/`

The first rollout bead may refine that root before emitting checked evidence.

## 6. Closure Gate

W051 can close for its first product scope when:

1. the sparse reader API is specified and implemented in OxCalc,
2. OxFml can carry sparse inputs through runtime and replay for the declared
   scope,
3. OxFunc consumes the reader for the first function group,
4. TreeCalc/core and replay evidence match the declared scenarios,
5. dense materialization is ruled out by a check or counter,
6. exclusions are listed plainly.

## 7. Status

Product status: not implemented yet. The next workset in the sequence is ready
to roll out.

Evidence: W050 provides prepared runtime identity and formal input/reference
transport. OxFunc/OxFml sparse runtime consumption is still open.

Still open: reader contract, OxCalc adapter, OxFml threading, OxFunc kernel
admission, integration evidence.

Formal status: no proof claim.
