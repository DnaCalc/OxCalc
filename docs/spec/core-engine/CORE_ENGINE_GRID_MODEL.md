# CORE_ENGINE_GRID_MODEL

Status: **Promoted active planning spec** (2026-06-13). Canonical OxCalc grid semantic model. Historical DnaTreeCalc recon notes are archived under DnaTreeCalc/docs/archive/grid-recon-2026-06/. Sections marked **[verify-COM]** remain provisional until OxXlPlay captures pin live Excel behavior.

Companion documents (drafted/planned):
- `CORE_ENGINE_REFERENCE_PROFILE_CONTRACT.md` — OxCalc-local W077 contract target for the
  generic reference-profile seam.
- `CORE_ENGINE_GRID_REFINEMENT_AND_EQUIVALENCE.md` — abstraction function, observation
  surfaces, Invariant Register (planned).
- `CORE_ENGINE_GRID_PERF_REGISTER.md` — perf claims and counter gates (drafted).
- OxFml `docs/handoffs/HANDOFF-DNATREECALC-001_STRICT_EXCEL_GRID_R1C1_BIND_PROFILE.md` — cross-lane ask (drafted).
- OxDoc `docs/OXDOC_REQUIREMENTS.md` — file-I/O repo requirements (drafted;
  relocates to the OxDoc repo on creation).

## 1. Purpose

This spec defines the **semantic model** for Excel-grid support in OxCalc: what a sheet grid
*is*, what a formula on it *means*, and what any conforming implementation must observably do.
It mandates behavior, never representation (META_NODES convention): template sharing, sparse
blocks, virtual cells, interval indexes, and tile streaming are representation choices that
must be invisible at the observation surfaces defined in §11.

The default answer to any semantic question not settled here is: **what does Excel do?**
(CORE_MODEL_SPEC §5 convention), with disagreements resolved by OxXlPlay COM observation and
recorded in this document.

## 2. Profile position

The grid exists under the **`strict-excel-grid` capability profile**. The engine carries two
first-class profiles per workspace:

| Profile | Reference surface | Containers | Lineage |
|---|---|---|---|
| `treecalc-v1` (rich tree) | tree paths, collections, sibling offsets | named-node tree, loose tables | DNA TreeCalc, keeps evolving |
| `strict-excel-grid` | A1/R1C1 cell & range refs, defined names, structured refs | sheet grids, grid-overlay tables | strict-Excel baseline |

Profiles are **runtime data on persisted workspace state, never build features**. Both compile
into every binary; mixed-profile processes are expected.

Cross-profile invariant (pending owner ratification, to be mirrored into CORE_MODEL_SPEC §4):
**a `strict-excel-grid` formula never references a tree target.** Tree→grid references are
permitted in principle and deferred; hybrid surfaces are a future profile.

## 3. Layering and ownership

| Concern | Owner |
|---|---|
| Grid document state (cells, axis state, tables-as-overlays), dependency graph, invalidation, evaluation, publication, epochs, spill extents | OxCalc |
| Formula grammar, reference-expression slots, bind lifecycle, profile dispatch, normal-form key plumbing, source span/text preservation | OxFml |
| `strict-excel-grid` reference-profile semantics: A1/R1C1 coordinate interpretation, `$` fidelity payloads, bounds/`#REF!`, dependency emission, edit transforms, and source rendering policy | OxCalc, plugged into OxFml through the public reference-profile/`BindProfile` ABI |
| Function semantics incl. blank coercion, aggregate semantics, hidden-sensitivity declaration, reference ops through the function execution context | OxFunc |
| Calc-time reference system for the grid profile (dereference/enumerate/transform of `CalcValue::Reference`) | OxCalc, behind the W060 host reference system |
| Intents, orchestration, viewport declarations, rendering | Host (DnaTreeCalc shell / grid lens) |
| .xlsx read/write, round-trip preservation, fidelity ledger | OxDoc (separate repo) |
| Excel ground truth capture / comparison verdicts | OxXlPlay / OxReplay |

### 3.1 Public packet/type glossary

- `SheetGridIngest`: OxDoc→OxCalc ingest packet for workbook/sheet identity, bounds, authored
  cells, formula source text/channel, axis state, merged regions, table overlays, feature-rendered
  regions, and opaque-part handles. It is not a dependency graph, a spill ledger, or a computed
  valuation.
- `GridReferenceSystemProvider`: OxCalc's `excel.grid.v1` implementation of OxFunc's existing
  provider trait. It owns A1/area/spill-anchor dereference, sparse enumeration, text resolution
  routing, reference facts, and transform/compose requests.
- `GridHostInfoProvider`: OxCalc provider for hidden-sensitive aggregate context over
  `AxisState`; SUBTOTAL/AGGREGATE query it rather than having grid-specific evaluator code.
- `GridVisibilityRange`: row-visibility fact packet for manual/filter/outline provenance over a
  one-dimensional row span. Hidden columns remain calc-insensitive.
- `GridAxisEdit`: typed row/column insert/delete edit request. It is a piecewise transform with
  deletion holes and bounds outcomes, not a scalar coordinate delta.
- `FeatureRenderedRegion`: claimed rectangle rendered by a non-formula feature such as a pivot
  table. It carries edit-admission and `needs_refresh` facts but is not recalculated by the
  ordinary formula relation.

## 4. State space

### 4.1 Coordinates and bounds

`Row = [1, 1_048_576]`, `Col = [1, 16_384]`. An out-of-bounds pair is **not a coordinate**:
nothing can be addressed, stored, or referenced there. A reference whose resolution (entry,
copy/fill translation, insert/delete shift, or calc-time construction) would leave bounds is
`#REF!`. (Closes the currently-unbounded `column_to_index` path in OxFml — see handover.)

### 4.2 Sheet state

A sheet is a total function with finite support:

```
Grid : (Row × Col) → CellState        -- all but finitely many cells are Empty
CellState = Empty
          | Literal(CalcValue scalar/error/text)
          | FormulaCell(formula identity per §5)
```

`Grid` is the **authored** layer. The **computed** layer (§6) is a separate valuation produced
by recalculation; spill targets (§7) exist only there. A blank read of an `Empty` cell yields
the engine's blank value; blank-coercion semantics are OxFunc's and are not re-specified here.

### 4.3 Axis state (document state, calc-relevant)

Per row and per column:

```
AxisProps = { size, hidden_manual : bool, hidden_filter : bool,
              outline_level, collapsed : bool }
effective_hidden = hidden_manual ∨ hidden_filter ∨ outline_hidden
```

where `outline_hidden` is **derived** from outline levels and collapse state (a pure
function, never stored provenance). The two stored bits are independent — a row can be both
manually hidden and filtered out — and the distinction is load-bearing because SUBTOTAL
distinguishes manual hiding from filter hiding (§8.2). Whether setting row height to 0 counts
as manual hiding **[verify-COM]**.

Axis state is **document state**, not view state: it persists, travels through OxDoc, is
revisioned (hide/unhide is an undoable, replayable edit — it changes values per §8), and §8
makes calculation depend on it. Viewport/scroll/zoom state is *not* part of this spec's state
space (§6.3). Columns carry the same properties for symmetry, but column hiding is
**hard-exempt from calculation**: it never affects results and never invalidates
(well-sourced Excel behavior).

### 4.4 Merged regions

Merged regions are document state: a set of non-overlapping rects per sheet. The top-left cell
carries the content; other member coordinates read as `Empty` for calculation. Merged regions
block spill (§7.3). Full merge semantics (edit behavior, styling) are host/UX concerns.

## 5. Formula identity: source text and R1C1-relative normal form

Every `FormulaCell` preserves its authored source text/channel when available and also carries
a profile-owned canonical identity: its **R1C1-relative normal form** — the formula text with
every grid reference represented relative to the owning cell where relative parts are offsets
and absolute parts preserve `$` semantics as absolute coordinates.

- Two cells hold *the same formula* iff their normal forms are textually equal.
- A1 text (with `$` fidelity preserved) and R1C1 text are **presentation/source channels** over
  the normal form; entry in either channel binds to the same identity.
- Unchanged `.xlsx` formulas keep their original source text. A structurally changed formula is
  re-rendered by the grid profile, with A1 as the preferred `.xlsx` presentation unless the
  host explicitly requests another channel.
- Fill/copy/paste produce cells whose normal form equals the source's (references that shift
  out of bounds become `#REF!` per §4.1).
- Template regions (rectangles of identical-normal-form cells, however they arise — file
  import via shared formulas, host-declared fill/flash-fill, or engine coalescing of
  sequential writes) are **representation**. Conformance: *materialization invariance* —
  materializing or splitting any region, or forcing any virtual cell to a concrete one,
  changes no observable value.

Empirical basis: Enron corpus, 20.3M formulas, ~4.5% unique under exactly this equality.

## 6. Evaluation semantics

### 6.1 The recalc relation

A recalculation takes `(Grid, AxisState, previous valuation, dirty set)` to a new valuation
over occupied cells (plus spill targets), defined as the least fixpoint of per-cell evaluation
over the dependency relation. **Calc order is existentially quantified**: any
dependency-consistent order is conforming. Excel's self-optimizing chain, visible-first
scheduling, and future region-parallel execution are all in-spec by construction.

Conforming implementations must satisfy (tested as metamorphic properties):
1. **Recalc idempotence** — an immediate no-edit recalc changes no value.
2. **Schedule invariance** — any conforming schedule, once quiesced, equals full recalc.
3. **Translation invariance** — translating all occupied content by (Δr,Δc) within bounds
   translates the valuation.
4. **Materialization invariance** (§5).

### 6.2 Volatility and dynamic references

Volatile functions re-dirty their dependents each cycle (per OxFml `ExecutionProfileSummary`
facts). References constructed at calc time (`INDIRECT`, `OFFSET`, spill-range refs)
dereference through the grid implementation of the **W060 host reference system**
(`ReferenceSystemProvider`), and the edges discovered this way enter the effective graph
through the **W047 CTRO lane** (Calc-Time Rebinding Overlay — shipped: resolution facts
harvested per evaluation, lowered to dynamic descriptors, persisted by positive publication
for the next run). Shipped CTRO converges run-over-run with no within-run re-entry; §7.1
defines the bounded within-run repair this spec additionally requires for spill extents.
W047's frontier-repair semantics already name "region/spill resize" as a trigger — §7 makes
it real.

Current W061 defined-name floor: OxCalc owns a same-sheet defined-name namespace in the grid
machines and feeds it to `GridReferenceSystemProvider`. OxFml binds name tokens as symbolic
profile references, and the provider resolves `SUM(InputRange)` plus
`SUM(INDIRECT("InputRange"))` at runtime. GridInvalidation-Ref records finite
`Name(name, extent)` dependencies, scalarizes the current extent for ordinary value edits,
exposes name-key dirty closure for namespace changes, and transforms the finite extent under
row/column insert/delete. The grid machines transform same-sheet defined-name rects under the
same row/column edit lane. Defined-name lifecycle APIs now cover same-sheet rename and delete
in GridCalc-Ref and GridOptimizedSheet: direct formulas are rewritten on rename (`InputRange`
-> `DataRange`) and delete (`DataRange` -> `#NAME?`), while `INDIRECT("InputRange")` text is
preserved and resolves through runtime text routing. GridInvalidation-Ref retargets/drops
finite name dependencies for rename/delete lifecycle operations and returns the namespace-key
dirty closure. The seed corpus emits these checks in the `namespace_lifecycle` artifact lane.
Namespace versions and structured/table name interactions remain open.

Current W061 table-overlay floor: OxCalc owns same-sheet `GridTableOverlay` state and registers
provider-owned table metadata with `GridReferenceSystemProvider`. OxFml owns native
structured-reference syntax and dispatches through the generic `bind_structured_reference` hook;
the strict grid profile emits symbolic `excel.grid.v1` structured-reference payloads, while
OxCalc resolves explicit table refs, caller-local `[Column]` refs, and caller-local `[@Column]`
refs at runtime through provider-owned metadata. `SUM(Table1[Amount])`, repeated in-table
`=SUM([Amount])` and `=[@Amount]*2` formulas, `SUM(Table1[[#Data],[Amount]:[Tax]])`,
non-contiguous section-union formulas such as
`SUM(Table1[[#Headers],[#Totals],[Amount]:[Tax]])`, escaped-column formulas, and corresponding
`INDIRECT(...)` text forms evaluate in both reference and optimized grid engines.
GridInvalidation-Ref records finite `Table(table, extent)` dependencies and current-row scalar
cell dependencies, scalarizes the current extent for ordinary value edits, exposes table-key
dirty closure for namespace changes, and transforms the finite extent under row/column
insert/delete. First table-overlay lifecycle APIs now cover same-sheet resize, rename, and
delete in GridCalc-Ref and GridOptimizedSheet: stale table feature-rendered-region claims are
removed, explicit structured-reference formulas are rewritten on table rename
(`Table1[Column]` -> `Sales[Column]`) and table delete (`Sales[Column]` -> `#REF!`), renamed
tables resolve through the provider under the new name, and deleted overlays stop contributing
table metadata. GridInvalidation-Ref retargets/drops finite table dependencies for
rename/delete lifecycle operations, rebuilds scalar edges for table resize extents, and returns
the namespace-key dirty closure. The seed corpus emits these checks in the `namespace_lifecycle`
artifact lane. Table/name collision precedence for omitted references, `INDIRECT("Table1[...]")`
text rewrites, resize-driven formula source expansion/shrink semantics, full table namespace
versioning, and OxDoc table ingest/export fidelity remain open.

### 6.3 Visibility doctrine

**Viewport visibility never changes dirty-truth; it only changes evaluation order and
publication timing.** A cell's published value at quiescence is schedule-independent (§6.1
property 2). Off-screen volatile staleness under visibility-bounded scheduling is a documented
profile flag (owner decision; deviation from Excel must be declared, never silent).

Document hidden state (§4.3) is **not** viewport visibility: it is calc input (§8).

### 6.4 Cycles

v1: any true cycle (after the monotone-scan refinement below) is a cycle error per the
engine's cycle-group machinery. A region-level self-edge whose affine offsets are strictly
monotone in one axis (`=R[-1]C+1` fill-down) is **not** a cycle: it evaluates as a scan.
Excel-match iterative calculation enters later through the W048/W055 cycle-profile lane.

## 7. Dynamic-array spill — provisional

A `FormulaCell` whose result is an array (or dereferences to one) is a **spill anchor**.

### 7.1 Extent and ghost cells

The **spill extent** is the rect (anchor at top-left, result rows×cols) the result occupies in
the *computed* layer. Non-anchor cells of the extent are **ghost cells**: authored state
remains `Empty`; computed state carries the corresponding array element. Ghosts are values,
not blanks: `ISBLANK(ghost) = FALSE` **[verify-COM]**; `COUNTA` counts them **[verify-COM]**.

The extent is itself a first-class calc output (a *spill fact* with its own epoch).

Current W061 executable floor: committed spill facts can be installed as anchor-keyed ledger
entries, and array-valued formulas publish spill extents in GridCalc-Ref and GridOptimizedSheet.
Successful optimized array payloads publish as dense computed regions; retained evidence
`w061-grid-scale-sequence-spill-1m-002` proves a 1,000,000-row `SEQUENCE` output is one dense
numeric computed region with zero sparse computed cells and commits one spill fact/epoch anchor
back to sheet state, and
`w061-grid-scale-filter-spill-1m-006` proves dense-backed row-mask and column-mask `FILTER`
outputs publish as dense spills with zero sparse computed cells, commit one spill fact/epoch
anchor back to optimized sheet state, clear vacated cells after contraction, and survive a later
committed optimized recalc that shrinks a value-dependent FILTER spill from 500,000 to 499,999
rows while advancing its extent/value epoch to 2. `A1#` consumers dereference those
extents through both engines, ghost cells are visible in sampled readout, GridInvalidation-Ref
can dirty consumers from a `SpillFact(anchor)` shape dependency, and formula-owned spill-ledger
changes trigger bounded run-level repair passes so earlier `A1#` consumers can converge after a
later anchor publishes. Merged-region and table-overlay feature-region blockers are carried as
rect metadata and block spill extents without materializing grid storage. A first
mutual-blockage slice is executable: when a later dynamic-array anchor lies inside an earlier
blocked formula-owned spill extent, the later anchor also yields `#SPILL!`. This does **not**
yet claim broad mutual-spill arbitration, the broader FILTER value-dependent spill matrix,
full table/spill structural repair, full namespace versioning, or Excel-verified ordering.

**Shape-change obligation:** when an anchor's extent changes, the dirty region is
`old extent ∪ new extent` — contraction included, so vacated ghost coordinates publish as
empty and their dependents re-evaluate. Observable at the invalidation-closure surface (§11).

**Within-run convergence:** placement arbitration commits ledger and body values before
later-scheduled consumers read when row-major order already places anchors first. Growth into
coordinates no earlier consumer saw is repaired by **bounded run-level repair passes**: after
the main pass, if formula-owned spill facts differ from the baseline and the bound formula
surface includes spill-anchor references, both reference and optimized engines re-evaluate
formula cells until the spill ledger is stable or the formula-count cap is reached. Recalc
reports count primary evaluation separately from `spill_repair_*` passes/evaluations, so `P-00`
remains a primary-pass assertion rather than hiding repair revisits. Residual instability at
the cap still needs the circular-spill policy and Excel observation lane.

**Arbitration order:** when multiple anchors compete (e.g. adjacent template instances that
each spill), arbitration order is deterministic and specified — proposed row-major over
anchors **[verify-COM: pin Excel's actual order; COM evidence becomes the order of record]**.
Mutual blockage of neighboring spilling instances is correct Excel behavior and must be
reproducible, which is why the order is spec text and not an implementation accident.

### 7.2 Spill-range references

- `A1#` references anchor A1's current extent; its dependency is on the spill fact, so shape
  changes re-dirty consumers. `A1#` where A1 is not a spilling formula → `#REF!` **[verify-COM]**.
- A reference *into* the extent (e.g. `=B3` where B3 is a ghost) reads the ghost value and
  depends on the anchor's spill fact, not on an authored cell.
- Implicit intersection (`@`, `_XLFN.SINGLE`) follows existing OxFml machinery unchanged.

### 7.3 Blockage: `#SPILL!`

Spill fails — the anchor evaluates to `#SPILL!` and **no element is published, no partial
spill** — iff the extent (minus the anchor) intersects:
1. a non-`Empty` authored cell;
2. a merged region;
3. a grid-overlay table (§9) — additionally, a formula *inside* a table column never spills:
   it implicitly intersects, Excel-faithfully;
4. another anchor's extent;
5. the sheet boundary (extent would exceed §4.1 bounds) **[verify-COM: `#SPILL!` vs `#REF!`]**.

Current W061 executable floor blocks on authored cells, merged regions, table-overlay feature
regions, and sheet-boundary overflow in the target extent; both engines also treat
already-published spill extents as blockers for deterministic row-major publication plus
bounded repair. The same blocked-spill ledger now handles the narrow neighboring-anchor case
where the candidate anchor is inside an earlier blocked formula-owned spill extent; broader
mutual/body-overlap arbitration remains unclaimed. GridInvalidation-Ref has a separate
`SpillBlocker(extent)` watcher for finite blocked extents, so a merged/table/feature blocker
change can dirty the blocked anchor without pretending to be an ordinary value edit or an `A1#`
spill-shape change. Full spill arbitration, full table/spill structural repair, full namespace
versioning, and broad blocked-spill recovery remain open lanes.

Blockage is re-examined whenever the blocking state changes: an authored edit landing inside a
current or previously-blocked extent seeds invalidation of the anchor (clearing a blocker
un-`#SPILL!`s it; creating one `#SPILL!`s it). The executable floor currently proves this for a
merged-region blocker closure in the seed corpus; broader table/name/opaque geometry repair
remains future work. Typing into a ghost cell is an authored edit and therefore blocks the anchor.

Blocked-by diagnostics (which rect/cell blocked) are published as typed run effects (the
OxFml seam vocabulary — `SpillEvent`/`SpillFact`/`ShapeDelta` — finally gains a truthful
producer: the engine arbitrates and emits; OxFml stays declarative).

Spill places into hidden rows normally — hidden state has no spill interaction
**[verify-COM: cheap confirmation alongside the §8 capture work]**.

### 7.4 Profile split

Spill across cells exists only in `strict-excel-grid`: the `spill_reference` capability and
`#`/`@` admission gate on the bind profile. The tree profile keeps node-level arrays with no
inter-node spilling (CORE_MODEL_SPEC stands verbatim; the `#` operator never enters the tree
profile — ratification pending, §14). One array-value substrate, two surfacings. This section
also answers the parked `DynamicArraySpillPolicy` admission requirement on structured tables:
grid profile admits spill operators over tables, tree profile keeps the deny.

## 8. Hidden and summary rows — provisional

### 8.1 Three-layer visibility model

| Layer | Examples | Calc effect |
|---|---|---|
| Viewport | scroll, zoom, panes, camera | none — schedule only (§6.3) |
| Document hidden state | manual hide, AutoFilter hide, outline collapse | **calc input** for hidden-sensitive functions |
| Styling | font color, custom formats | none |

### 8.2 Hidden-sensitive functions (Excel-anchored)

The normative function rule tables live in the OxFunc function-lane contract
(`FUNCTION_SLICE_SUBTOTAL_AGGREGATE_CONTRACT_PRELIM.md`) — this spec references, never
restates. The anchors:

- `SUBTOTAL(1–11)` includes manually-hidden rows; `SUBTOTAL(101–111)` excludes them;
  **filter-hidden rows are always excluded by both** (well-sourced, matches OxFunc's rule
  tables).
- `AGGREGATE(…, option 0–7, …)` per Excel's documented option table. **[verify-COM — highest
  priority]**: Microsoft's docs and empirical reports conflict on whether option 4 ("ignore
  nothing") nevertheless ignores manually-hidden rows; OxFunc currently encodes the MS-doc
  reading and the rule table is a one-line flip if COM contradicts it.
- Both families always ignore nested SUBTOTAL/AGGREGATE results within their range (the
  nested fact is a template-level property in the grid).
- **Hidden columns never affect these functions** — sensitivity is row-only (§4.3 exemption).
- Outline collapse hides rows; collapsed rows count as hidden **[verify-COM: same as manual
  for the 1xx distinction? same recalc trigger?]**.

OxFunc already declares hidden-sensitivity (`HostInteractionClass::WorkbookState` +
refs-visible argument preparation) and reads it through the mandatory `HostInfoProvider`
aggregate-context query — the grid supplies the missing real implementation over axis state
(today every real host returns `#VALUE!` for reference-form SUBTOTAL/AGGREGATE because no
provider exists). A hidden-sensitive formula acquires a **visibility dependency** on the row
span it aggregates; for running probes (fill-down `SUBTOTAL(103, B$2:B2)` and the degenerate
single-cell visibility probe `SUBTOTAL(103, cell)`) the dependency span is affine in the
caller row, the visibility analogue of template-relative edges.

### 8.3 Invalidation policy

Hide/unhide/filter/outline-toggle are **typed document edits** producing invalidation seeds
that dirty exactly the visibility-dependents whose declared span intersects the affected axis
range. The hidden rows' *own* cells are not dirtied — their values do not change.

Excel's freshness mechanism is different in kind: event-driven row-flagging ("hiding or
unhiding rows, but not columns" is a documented recalc trigger; AutoFilter actions flag the
whole filter range), which over-dirties relative to true dependence and may under-dirty in
corner cases (constants-only hidden rows, cross-sheet aggregates) **[verify-COM: dirtying
scope matrix, incl. VBA-initiated hides and manual-calc observability]**.

Policy: **`visibility_staleness = Exact` is the default and the spec'd semantics** — precise
dependency-driven invalidation, with the conformance statement *"fresher than Excel is
conforming: any value a conforming full recalc would change is recomputed; Excel's
over-dirtying is not reproduced."* An `ExcelCompat` flag is **reserved but unbuilt** unless
OxReplay conformance diffs trip on observable staleness.

### 8.4 Scope for v1

Hidden state as calc input is v1. AutoFilter as an *engine feature* (criteria, reapply) is
deferred; the `Filter` provenance value is reserved so its arrival changes no schema. Outline
summary rows are derived hidden runs over the outline structure.

## 9. Tables as grid overlays

In `strict-excel-grid`, a table is a **claimed rect overlay** on the sheet: header/data/totals
slices at concrete coordinates, column formulas as template regions over column extents,
structured references resolving to rect slices. Overlay collisions (with another table, or via
spill §7.3) are errors, Excel-faithfully. Table semantics (structured refs, column formulas,
totals) are written once and shared with the tree profile's loose table facet through the
`TableBacking` seam; the interop shape for both is `TableSpec`.

## 10. Structural edits

Insert/delete rows/columns shift content, axis state, tables, merged regions, spill anchors,
and references by an **edit-transform algebra**. They are not scalar `row += delta` or
`col += delta` operations. Deletion creates holes; insertion may expand or shift ranges;
out-of-bounds and deleted references become first-class invalid references (`#REF!`) rather
than parse failures.

The first structural-edit matrix covers:

| Surface | Insert rows/cols | Delete rows/cols |
|---|---|---|
| Formula anchor | shifts if at/after insertion; formula normal form preserved under translation | deleted anchors remove authored cell; after-region anchors shift |
| Point refs | before unchanged; at/after shift; pushed past bounds → `#REF!` | inside deletion → `#REF!`; after deletion shifts |
| Finite ranges | before unchanged; after shift; insertion inside expands | before/after shift or unchanged; overlap shrinks; fully deleted → `#REF!` |
| Whole-row/column refs | same-axis insert expands or shifts per Excel; cross-axis unchanged | same-axis delete shrinks/deletes/shifts; cross-axis unchanged |
| Tables | overlay rect and table namespace version transform together | rows/cols shrink, delete, or require table-specific refusal per Excel capture |
| Merged regions | rect shifts/expands or refuses if merge rules require | rect shrinks/deletes; partial overlap is explicit |
| Spill anchors/extents | anchor and extent transform; deleted anchor drops spill fact | overlap shrinks extent; deleted anchor drops fact; consumers rebind through `A1#` |
| Feature-rendered regions | class policy may shift, flag `needs_refresh`, or refuse edit | class policy may shrink, flag `needs_refresh`, or refuse edit |
| Geometry-coupled opaque parts | OxDoc preserves bytes until an owning transform exists | edit is blocked or routed as an OxDoc follow-up, never silently rewritten |

This is the largest semantic surface and a known engine pathology (337s rebind churn baseline):
it is specified **first from OxXlPlay captures** and is generator bias #1 in the conformance
corpus. The matrix above is the active floor; later rows add sheet rename/delete, move range,
defined names, structured references, external links, and dynamic reference classification.

Current W061 executable floor: whole-row and whole-column references evaluate through the grid
provider and feed an `AxisValue(row|column, first..last)` invalidation dependency. This avoids
scalarizing large row/column references in the reference oracle while still allowing ordinary
cell edits to dirty the dependent formula chain and structural edits to shift/shrink the axis
dependency.

## 11. Conformance and refinement (pointer)

Correctness of any optimized implementation is **observational refinement** against
GridCalc-Ref (the TraceCalc-extension reference machine) under an abstraction function, at
three surfaces: (1) value readout via coordinate probes, (2) invalidation closure as a set,
(3) committed effects/errors (`#REF!`/`#SPILL!` placement, spill extents, blocked-by facts).
Diagnostics, timings, and epoch numerics are outside the relation. The machinery, invariant
register, and property generators are specified in
`CORE_ENGINE_GRID_REFINEMENT_AND_EQUIVALENCE.md`.

## 12. What this spec does not specify

Block/chunk storage, value/style interning, template-region representation and coalescing
heuristics, dependency-graph compression, interval indexes, scheduling internals, tile
streaming, rendering, persistence formats (OxDoc's), and undo/revision mechanics. All are
representation, constrained only by §5/§6 invariants and the §11 surfaces.

**Partially admitted, still mostly reserved:** *feature-rendered regions* (pivot table reports
being the archetype) are claimed rects whose cells are written by a non-formula producer inside
an explicit refresh **transaction** (a document edit, never part of the §6.1 recalc relation).
Current W061 executable floor keeps the open rect-class tag, admits pivot-like edit policy for
row/column structural edits, refuses edits inserted/deleted inside a pivot-like claimed rect,
and marks such a region `needs_refresh` when an edit before it shifts its geometry. Table-overlay
feature regions still transform normally and keep their refresh flag stable. The broader
computed-layer epoch class, source-rect invalidation, opaque-part ownership, Excel-pinned pivot
semantics, and full writer taxonomy remain reserved extension points.

## 13. Cross-lane prerequisites

1. **OxFml**: generic `BindProfile`/reference-profile ABI per
   `CORE_ENGINE_REFERENCE_PROFILE_CONTRACT.md`, default-preserving profile dispatch, formula
   grammar hooks for native Excel syntax, symbolic bound-reference packets,
   caller-independent normal-form/cache keys, source span/text preservation, and edit-transform
   lifecycle envelopes. OxFml does **not** own grid storage, dependency closure, spill
   arbitration, or grid-reference dereference semantics. W077 must cover caller-relative A1
   and symbolic R1C1 records, `GridBounds`/bounds-to-`#REF!` behavior as profile-owned facts,
   `$` fidelity, dependency envelopes, reference-visible argument preparation, transform
   algebra for fill/paste/insert/delete, and rendering separated from persisted source text.
   Native Excel syntax still belongs in OxFml grammar/bind; grid address meaning stays in the
   `strict-excel-grid` profile.
2. **OxFunc**: **no API change for v1** — the `HostInfoProvider` aggregate-context seam and
   the SUBTOTAL/AGGREGATE rule tables already exist and are tested; the grid supplies the
   first real provider. `ReferenceKind::SpillAnchor` dereference capability exists with zero
   implementors; the grid provider implements it. Registered deferral: a run-compressed
   aggregate-context seam variant for 1M-row spans.
3. **OxCalc**: strict grid profile adapter, grid `ReferenceSystemProvider` (W060) incl.
   `SpillAnchor`; spill placement arbitration + ledger + bounded grid-machine repair passes,
   with broader CTRO/frontier integration remaining a successor lane; `GridHostInfoProvider`;
   `GridVisibilityRange` 1-D interval edges + `GridAxisEdit` typed edits; `CalcTarget`
   generalization; W051 unified `ReferenceKind`.
4. **OxDoc**: boundary contract per OxDoc `docs/OXDOC_REQUIREMENTS.md`; xlsx hidden-row ingest sets
   provenance (`hidden="1"` inside an active AutoFilter range → Filter, else Manual; ledgered
   `Derived`). Follow-up lane: `PartStore`/opaque-byte preservation and hidden-provenance
   reconciliation for geometry-coupled opaque parts under structural edits.
5. **OxXlPlay**: **Wave-1 long pole.** New scenario ops (set_row_hidden, set_row_height incl.
   0, apply/clear AutoFilter, outline group/ShowLevels, VBA EntireRow.Hidden, dynamic-array
   entry) + a row-visibility observable view + recalc-witnessing via eval-counter UDFs —
   today none of the §7/§8 **[verify-COM]** items can be captured. Then the §10 shift-semantics
   families.

## 14. Open questions (owner)

1. Ratify the §2 cross-profile invariant as CORE_MODEL_SPEC §4 text.
2. Off-screen volatile staleness flag (§6.3): acceptable as documented profile deviation?
3. §7.1 within-run repair: confirm bounded repair passes (k≈4) over pure run-over-run
   convergence, and `#SPILL!`-with-reason-`circular` at the cap.
4. §7 arbitration order: sanction COM evidence as the order of record?
5. Spill body representation: materialized computed-layer cells (recommended: uniform
   readers, ~16 B/cell × extent) vs read-through overlay — representation choice, but with
   register-visible cost; confirm.
6. Ratify "`#` operator never enters the tree profile" as CORE_MODEL_SPEC text.
7. §8.3 `visibility_staleness = Exact` default with "fresher-than-Excel is conforming"
   language; `ExcelCompat` reserved, unbuilt — ratify.
8. Ratify AxisState (sizes + hidden bits + outline) as revisioned document state (hide/unhide
   is an undoable intent) — extends the axis-geometry ownership amendment.
9. Single `HostInfoProvider` seam for all grid host-state queries (recommended) vs a separate
   visibility reader.
10. Merged-region calc semantics (§4.4): confirm Empty-read for non-anchor members.
11. v1 cycle policy (§6.4): cycle-as-error acceptable until the cycle-profile lane lands?
