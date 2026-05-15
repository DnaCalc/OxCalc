# Core Engine Future Ideas: Rich And Virtual Data

Status: `future_ideas_note`

This note parks non-workset ideas that came out of W050 rich/sparse
discussion but go beyond Excel-compatible worksheet calculation. It is not a
workset, not an execution target, and not a commitment to implement these
features.

## 1. Boundary

Keep in active worksets:

1. Excel-compatible sparse range readers for whole-column and large-area
   worksheet references (`W051`).
2. Ordinary dynamic-array result surfaces that Excel already supports.
3. Image/rich returned-value surfaces where Excel has visible behavior and
   OxFunc/OxFml already publish concrete carrier metadata.
4. Goal Seek / Solver / what-if style sensitivity lanes where they map to
   recognizable spreadsheet behavior (`W052` planning owns that decision).

Park here:

1. Generic virtualized arrays whose identity is not a worksheet range,
   spill range, or Excel-compatible array result.
2. External query objects, lazy query plans, and producer-specific pushdown.
3. Generic rich producer/consumer protocols beyond concrete Excel functions
   such as `IMAGE`.
4. Capability families such as arbitrary `Queryable`, host-defined rich
   object protocols, or custom object graphs.
5. Cross-workbook or service-backed rich data streams whose semantics are
   closer to dataframes/query engines than to Excel cell/range behavior.

## 2. Ideas

### 2.1 Virtualized Arrays

Represent a large logical array as a shaped, indexable carrier that can answer
element reads and materialise slices without constructing the whole dense
array. This is useful for engine experiments, but it should not enter the
workset queue unless there is a concrete Excel-compatible formula surface that
requires it.

### 2.2 Queryable Rich Objects

Represent external or host-provided objects with capability keys such as
`Queryable`, `Indexable`, `Enumerable`, `Shaped`, and `Materialisable`.
Producer-specific pushdown could reduce evaluation cost, but replay would
need to record the resolved iterator/read set rather than merely the query
plan.

### 2.3 Generic Rich Producer Protocol

Generalize beyond `IMAGE` / `_webimage` returned-carrier metadata to a typed
producer protocol where functions publish required and exercised capability
sets. This should stay outside active worksets until at least one Excel-visible
producer/consumer pair needs the generic form.

### 2.4 Dataframe-Like Carriers

Table-shaped or dataframe-like carriers could support query, grouping,
projection, and typed columns. This is intentionally outside current Excel
scope unless it is grounded in a concrete worksheet feature or compatibility
target.

## 3. Promotion Rule

An idea from this note may become a workset only when it has:

1. a concrete user-visible spreadsheet behavior or product decision,
2. an owner split across OxCalc/OxFml/OxFunc,
3. replay-visible identity and evidence requirements,
4. a bounded first slice that does not reopen W050's formal-input and
   prepared-package seam,
5. a clear statement of whether it is Excel-compatible behavior or an
   intentional extension.

Until then, these are design options, not pending work.
