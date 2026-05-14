# W051 Sparse Range Readers And Defined-Entry Semantics

Status: `pre_planning`

Parent predecessor: `W050` (Lane C hole-type taxonomy, Lane G capability vocabulary)

Parent epic: TBD (allocated when W051 is activated)

## 1. Purpose

W051 lands the first concrete rich-value class after W050 admits `SparseRangeHole` and the `RichValueHole` capability vocabulary as identity-only scaffolding. W050 reserves the hole-type variant and the kernel-side `SparseIteratorOk` argument-preparation profile but ships no implementation; W051 makes them real.

W051 implements:

1. `SparseRangeReader` — the concrete reader binding for reference ranges that are large but mostly blank, exposing the typed protocol `declared_extent`, `defined_cardinality`, `defined_iter`, `read_at(coord) -> Defined(EvalValue) | Blank`, `contains(coord)`.
2. Activation of the kernel-side `SparseIteratorOk` argument-preparation profile so aggregation kernels (`SUM`, `COUNT`, `AVERAGE`, `MIN`, `MAX`, criteria-family functions) consume whole-column references without dense materialisation.

W051 is in a deliberate `pre_planning` state. Scope, beads, exit gates, and evidence policy are decided after W050 lands the hole-type taxonomy and capability vocabulary. This document is pre-planning background only; do not infer a bead path or commit to artefacts from it.

## 2. Pre-Planning Background

### 2.1 Cell-value model is two-state

The reader's `read_at` return is `Defined(EvalValue) | Blank` — a two-state model, not three. Excel does not expose an observable cell-value-level distinction between never-assigned and assigned-then-cleared cells: `ISBLANK`, `COUNTBLANK`, `COUNTA`, arithmetic coercion (blank coerces to 0), and equality comparisons all treat them identically as `Blank`. The VBA surface agrees — `Range.Value` returns `Empty` for both, and `IsEmpty` is TRUE for both.

The distinction that *is* observable is empty-string `""` vs blank: `ISBLANK("")` is FALSE, `COUNTA` counts an empty-string cell. The reader carries this correctly — empty string is `Defined(EvalValue::Text(""))`, distinct from `Blank`.

Sheet-structural state that *does* persist across clear operations — used range, cell formatting, conditional-format ranges, data-validation rules, comments — is observable at the *sheet* level (for example via `Worksheet.UsedRange`, the OOXML `dimension` element), not the cell-value level. That state is owned by other Repository surfaces, not by the `SparseRangeReader`. W051 must not conflate the two layers.

### 2.2 Ownership split

W051 is primarily an OxFunc-owned workset. The `SparseIteratorOk` argument-preparation profile and the aggregation-kernel updates are OxFunc's. OxCalc supplies the `SparseRangeReader` implementation backed by its structural store; OxFml threads the sparse binding through semantic plan and evaluation context. A cross-repo handoff packet to OxFunc is expected.

### 2.3 Replay

Sparse readers are recorded by their defined-entry stream and declared extent. Replay reconstructs the reader from the recorded stream; iteration order is part of the contract; any query pushdowns are recorded as resolved iterators (what was read, not what was queryable). Determinism is preserved.

## 3. Relationship To W050

W051 depends on W050 Lane C (the hole-type taxonomy, which admits `SparseRangeHole`) and Lane G (the capability vocabulary, which admits `RichValueHole` with `Indexable + Enumerable + Shaped + Materialisable`). `SparseRangeReader` is the first concrete realisation of a rich value: it is the `Indexable + Enumerable + Shaped` capability set with sparse-iteration semantics, and its `Materialisable` capability is the dense view it abstracts.

W051 does not reopen W050's seam design. It implements against the hole-type and capability surfaces W050 commits.

## 4. Open Scoping Questions

Deferred until W050 lands and W051 is planned in detail:

- Which OxFunc aggregation kernels take the `SparseIteratorOk` profile first, and in what order?
- Does the reader support predicate pushdown (`intersect(predicate)`) in the first cut, or is that a successor?
- How are very large declared extents bounded against pathological `defined_cardinality` queries?
- What is the backing-store contract OxCalc's structural store must satisfy to produce a `SparseRangeReader` cheaply?

## 5. Status Surface

- execution_state: `pre_planning`
- scope_completeness: `scope_partial`
- target_completeness: `target_partial`
- integration_completeness: `partial`
- prerequisites: W050 Lane C (hole-type taxonomy) and Lane G (capability vocabulary) landed
- bead_path: not yet specified — W051 epic id and bead structure allocated when W051 is activated
- exit_gate: not yet specified
- evidence_policy: not yet specified
- upstream_dependencies: `OxFunc` (primary owner of `SparseIteratorOk` and aggregation-kernel updates), `OxFml` (threads sparse binding through semantic plan)
