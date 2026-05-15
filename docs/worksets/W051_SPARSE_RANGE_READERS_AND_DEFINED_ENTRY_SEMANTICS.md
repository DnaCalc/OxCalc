# W051 Sparse Range Readers And Defined-Entry Semantics

Status: `activation_review_ready`

Parent predecessor: `W050` (closed production formula-authority scope; runtime prepared-package/formal-input seam and sparse identity reservation are now available)

Parent epic: TBD (allocated when W051 is activated)

## 1. Purpose

W051 lands Excel-scope sparse range readers after W050 removed the local formula-identity compatibility projections and moved OxCalc onto OxFml runtime prepared-package/formal-input surfaces. This is not a general rich-data workset. Its target is ordinary worksheet calculation over whole-column and large-area references without dense materialisation.

W051 implements:

1. `SparseRangeReader` — the concrete reader binding for reference ranges that are large but mostly blank, exposing the typed protocol `declared_extent`, `defined_cardinality`, `defined_iter`, `read_at(coord) -> Defined(EvalValue) | Blank`, `contains(coord)`.
2. Activation of the kernel-side sparse argument-preparation profile, currently reserved upstream as `SparseRangeAccepted(extent_class, cardinality_class)` / `SparseIteratorOk`-equivalent metadata, so selected aggregation kernels consume whole-column references without dense materialisation.
3. Replay-visible iteration order and defined-entry evidence for Excel-compatible range reads.

W051 is ready for activation review, not execution. The post-W050 seam is available, but the concrete sparse-reader runtime API is not yet present in OxFunc/OxFml. Activation should first lock the smallest cross-repo reader contract and then create beads against that contract.

## 2. Post-W050 Validity Review

Still valid:

1. The Excel-scope boundary remains correct. W051 should cover worksheet references and range reads, not generic virtualized arrays or custom host objects.
2. The two-state cell-value model remains correct for formula evaluation: `Defined(EvalValue)` versus `Blank`, with empty string represented as a defined text value.
3. W051 should not reopen W050's formula-authority seam. It should use OxFml runtime prepared packages, formal references, and formal input bindings as the intake surface.
4. Replay must record the resolved defined-entry stream and declared extent actually read, not an abstract queryable object.

Needs updating from the original pre-planning text:

1. W050 has landed for its production formula-authority scope. References to "after W050 lands" are stale.
2. OxFunc currently has sparse admission reserved as metadata, but no kernel consumes a sparse reader yet.
3. OxFml currently carries prepared-package identity, formal references, formal input bindings, and the `arg_admission_metadata_version` invalidation bridge. It does not yet carry a sparse reader runtime/replay surface.
4. OxCalc currently has TreeCalc structural snapshots, repository/published-value maps, and fixture range/value surfaces. It does not yet have a production sheet-coordinate sparse range store or reader adapter.

## 3. Pre-Planning Background

### 3.1 Cell-value model is two-state

The reader's `read_at` return is `Defined(EvalValue) | Blank` — a two-state model, not three. Excel does not expose an observable cell-value-level distinction between never-assigned and assigned-then-cleared cells: `ISBLANK`, `COUNTBLANK`, `COUNTA`, arithmetic coercion (blank coerces to 0), and equality comparisons all treat them identically as `Blank`. The VBA surface agrees — `Range.Value` returns `Empty` for both, and `IsEmpty` is TRUE for both.

The distinction that is observable is empty-string `""` vs blank: `ISBLANK("")` is FALSE, and `COUNTA` counts an empty-string cell. The reader carries this as `Defined(EvalValue::Text(""))`, distinct from `Blank`.

Sheet-structural state that persists across clear operations — used range, cell formatting, conditional-format ranges, data-validation rules, comments — is observable at the sheet level, for example via `Worksheet.UsedRange` or the OOXML `dimension` element. That state belongs in sparse sheet structural management and may back efficient range bounds, but it is not part of the formula value-level `SparseRangeReader.read_at` state. W051 must keep the structural surface and the value reader connected but distinct.

### 3.2 Explicit Scope Boundary

In W051 scope:

1. Excel-compatible cell/range semantics for whole-column and large-area reads.
2. Defined-entry versus blank behavior for worksheet formulas.
3. Aggregation and criteria-family functions whose Excel behavior can be preserved while avoiding dense materialisation.
4. Replay of the concrete defined-entry stream, declared extent, and deterministic iteration order used by a formula run.

Out of W051 scope:

1. Generic virtualized arrays not corresponding to worksheet range/reference behavior.
2. Queryable external rich objects and producer-specific pushdown protocols.
3. Generic rich producer/consumer protocols beyond the concrete Excel sparse reader.
4. New non-Excel capability families such as arbitrary `Queryable` or custom host objects.

Those out-of-scope ideas are parked in `docs/spec/core-engine/CORE_ENGINE_FUTURE_IDEAS_RICH_AND_VIRTUAL_DATA.md`.

### 3.3 Ownership split

W051 is cross-repo work:

1. OxCalc owns the `SparseRangeReader` implementation backed by a sheet/range structural store or adapter.
2. OxFml owns threading the sparse binding through semantic plan, runtime environment, prepared-package identity, and replay projection.
3. OxFunc owns the sparse argument-preparation metadata/profile and aggregation-kernel consumption.

The first activation step should be a narrow handoff to OxFunc and OxFml that asks for the concrete reader API and runtime/replay intake shape. It should not ask either repo to implement generic rich data.

### 3.4 Replay

Sparse readers are recorded by their defined-entry stream and declared extent. Replay reconstructs the reader from the recorded stream. Iteration order is part of the contract. Producer-side filtering and pushdown are deferred unless the first slice records the resolved iterator exactly as read.

## 4. Activation Candidate Slices

1. Contract slice: define the shared `SparseRangeReader` shape, coordinate model, extent representation, defined-entry ordering, and blank/empty-string semantics.
2. OxCalc slice: implement a reader adapter over the available structural/value store, with a fixture-backed path if the production sheet-coordinate store is not ready yet. The adapter must not require O(extent) traversal for `defined_cardinality`, `defined_iter`, `contains`, or `read_at`.
3. OxFunc slice: activate sparse admission for the first function family. Recommended first group is `SUM`, `COUNT`, `COUNTA`, and `COUNTBLANK`, because they exercise numeric coercion, defined-entry counting, blank counting, and empty-string distinction before widening to `AVERAGE`, `MIN`, `MAX`, and criteria-family functions.
4. OxFml slice: thread sparse formal input bindings through runtime prepared identity and replay without dense materialisation.
5. Integration slice: add TreeCalc/OxFml/OxFunc evidence for at least one whole-column or large-area sparse reference where replay shows declared extent, defined-entry stream, iteration order, and final formula result.

## 5. Activation Questions

These should be answered before W051 moves from activation review to execution:

1. What exact Rust/shared shape represents a sparse range reader at the OxFml/OxFunc boundary?
2. Does the first OxCalc reader bind to a production sheet-coordinate store, a temporary adapter over existing TreeCalc value maps, or both with clear fixture/production labels?
3. Which first function group is accepted by OxFunc, and is the recommended `SUM`/`COUNT`/`COUNTA`/`COUNTBLANK` slice acceptable?
4. What are the complexity guarantees for `declared_extent`, `defined_cardinality`, `defined_iter`, `contains`, and `read_at`?
5. What replay columns/artifacts prove that dense materialisation did not occur?

## 6. Status Surface

- execution_state: `activation_review_ready`
- scope_completeness: `scope_partial`
- target_completeness: `target_partial`
- integration_completeness: `partial`
- prerequisites: W050 production formula-authority seam landed; OxFunc sparse admission is reserved but not active; OxFml sparse runtime/replay threading is not yet active; OxCalc sparse range reader implementation is not yet present
- bead_path: not yet specified — W051 epic id and bead structure allocated when W051 is activated
- activation_gate: cross-repo reader/API contract agreed, first function group selected, and replay artifact shape agreed
- exit_gate: not yet specified
- evidence_policy: replay evidence must include declared extent, deterministic defined-entry stream, blank/empty-string behavior, and a no-dense-materialisation check for at least one large range
- upstream_dependencies: `OxFunc` (sparse argument-preparation profile and aggregation kernels), `OxFml` (sparse binding through semantic plan/runtime/replay)
- open_lanes: sparse reader API, OxCalc backing reader, OxFml sparse binding/replay, OxFunc sparse kernel admission, integration evidence
