# Excel Formatting Hierarchy and Visibility Model

## 1. Purpose
Define a concrete, implementation-facing model for:
1. formatting hierarchy and precedence,
2. defaults and locale interaction,
3. what formula evaluation can and cannot observe about formatting (including conditional formatting).

This document is a focused companion to:
1. `EXCEL_CELL_CONCRETE_MODEL.md` (`ECM-FMT-*` lanes),
2. `fec-f3e/FEC_F3E_REDESIGN_SPEC.md` (locale/profile and seam policy lanes),
3. `CONFORMANCE_REQUIREMENTS.csv` (`XLS-CF-FM-*` lanes).

## 2. Formatting Object Families (SpreadsheetML-facing)
Working model for worksheet-visible formatting stacks:
1. workbook style primitives (`numFmts`, `fonts`, `fills`, `borders`, etc.),
2. style XF families (`cellStyleXfs`, `cellXfs`) and style linkage (`xfId` lane),
3. per-cell style index usage (`s` / style index lane),
4. differential formats (`dxf`) used by conditional formatting and related overlays,
5. sheet defaults (`sheetFormatPr`, `baseColWidth`, `defaultRowHeight`) and row/column-default style lanes.

## 3. Style Resolution Pipeline (Draft Model)
Draft effective-format pipeline for a cell:
1. workbook/sheet baseline defaults,
2. row/column style defaults (where present),
3. cell style index mapping via `cellXfs` (+ referenced `cellStyleXfs` lane),
4. table/style-region overlays where applicable,
5. conditional-format differential overlay (`dxf`) where rule conditions match and precedence permits.

Boundary rule:
1. value semantics remain independent of formatting semantics.
2. formatting pipeline computes effective display/style state, not core value identity.

## 4. Precedence and Conflict Lanes
Current explicit lanes:
1. base style vs direct cell style index,
2. row/column default style interaction with cell style index,
3. table style region interaction with direct cell style,
4. conditional-format overlap and priority ordering,
5. spill-target and dynamic-array interaction with conditional formatting.

Status:
1. precedence is partially source-anchored and partially empirical/provisional.
2. conflict lanes must remain explicit until cross-build empirical closure.

## 5. Defaults and Origin of "Normal" Formatting
Defaults are modeled as a profile-sensitive composition, not a hardcoded constant:
1. workbook/template style tables,
2. sheet defaults (`sheetFormatPr` family),
3. host/build profile behavior (for example baseline default font family/size in newly created workbooks),
4. locale profile effects on number/date/time render behavior.

Implication:
1. default font/size claims are version/profile assertions that require explicit evidence capture.

## 6. Locale/Regional Interaction Model
Locale profile affects:
1. formatting parse/render behavior (number/date tokens and separators),
2. text-to-number/date interpretation lanes used by related functions,
3. display output under equivalent stored value/style state.

Locale profile does not alter:
1. core value-tag semantics,
2. style object identity (style ids/indices), except where locale-specific format code interpretation is defined.

## 7. Formula Visibility Boundary (Key)
This lane separates:
1. core formula value computation,
2. formatted-display introspection behaviors.

Working classification:
1. `TEXT(value, format_text)`:
   - explicit format-string conversion; does not require reading ambient cell formatting.
2. `CELL(...)` / `INFO(...)`:
   - host/context-introspection family; may expose environment and selected formatting-related metadata lanes.
3. legacy XLM `GET.*`/`GET.CELL`-style techniques:
   - treated as compatibility/probe lane requiring explicit empirical capture in this project.

Conditional-format visibility question:
1. unresolved: whether effective conditional-format result is directly observable through formula/evaluation functions in supported modern contexts.
2. policy: keep as provisional empirical lane until explicitly bounded.

## 8. Grid Formatting Semantics (Workbook-level View)
Grid-formatting behavior should be represented through:
1. persisted style objects and index references,
2. non-destructive overlays (table/CF differentials),
3. precedence/merge rules for effective display state,
4. explicit separation between persisted format state and transient effective-display state where applicable.

## 9. Formal Evidence Anchors (Focused Pass)
Key promoted anchors used here:
1. style index correction lane:
   - `CONF-discovered-ms-oe376-220816-823374c7-0409` (`p:5904`)
   - `CONF-discovered-ms-oi29500-250218-d35cbb01-0387` (`p:5446`)
2. style XF cardinality/relationship underspec lane:
   - `SPEC-discovered-ms-oe376-220816-823374c7-07670` (`p:6726`)
   - `SPEC-discovered-ms-oi29500-250218-d35cbb01-07068` (`p:6315`)
3. `xfId` overwrite linkage lane:
   - `SPEC-discovered-ms-oe376-171212-fc69605e-19192` (`page:324:block:74`)
4. `dxfId` optional lane:
   - `SPEC-discovered-ms-oe376-220816-823374c7-07427` (`p:6464`)
   - `SPEC-discovered-ms-oi29500-250218-d35cbb01-06824` (`p:6050`)
5. sheet defaults lane:
   - `SPEC-discovered-ms-oe376-220816-823374c7-07309` / `-07310` (`p:6365`/`p:6366`)
   - `SPEC-discovered-ms-oi29500-250218-d35cbb01-06711` / `-06712` (`p:5956`/`p:5957`)
6. `numFmtId` default/constraint tension lane:
   - no-default signals: `SPEC-discovered-ms-oi29500-250218-d35cbb01-06258`, `SPEC-discovered-ms-oe376-220816-823374c7-06881`
   - constrained case signal: `CONF-discovered-ms-xlsx-250916-d16a975a-0067`

## 10. Required Empirical Closure Tracks
1. precedence collisions across row/column/cell/table/CF layers,
2. workbook/template default style origin and drift by build/profile,
3. locale profile matrix for format parse/render effects,
4. formula-visible formatting and conditional-format observability (`TEXT`, `CELL`, `INFO`, legacy compatibility probes).

## 11. Status
1. This is a draft model intended to tighten conformance lanes and empirical plans.
2. Unresolved assertions are intentionally tracked as provisional in linked requirement/open-question artifacts.
