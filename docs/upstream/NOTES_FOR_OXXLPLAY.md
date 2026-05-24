# Notes For OxXlPlay

Relationship: outbound observation and integration note for OxXlPlay.

This OxCalc-owned note records how OxCalc currently consumes OxXlPlay retained
Excel table evidence for W056. It does not ask OxXlPlay to own TreeCalc table
semantics.

## W056 Table Observation Intake

OxCalc has accepted the current OxXlPlay W009/W056 table observation floor for
`calc-4vs8.52`.

Observed OxXlPlay anchors:

1. `oxxlplay-ze4`: first structured-reference workbook observation tranche.
2. `oxxlplay-4nd.1`: WorkbookConstructionSpec table-node equivalent fixture.
3. `oxxlplay-4nd.2`: standalone table construction observation pack.
4. `oxxlplay-4nd.3`: table update observation and oracle scenarios.
5. `oxxlplay-4nd.4`: table delete and save/reopen residual coverage.
6. `oxxlplay-4nd.5`: third-pass table residual observation pack.

Retained artifact roots consumed by OxCalc planning:

1. `states/excel/xlplay_structured_reference_workbook_001/`
2. `states/excel/xlplay_workbook_construction_spec_001/`
3. `states/excel/xlplay_table_construction_basic_001/`
4. `states/excel/xlplay_table_update_oracle_001/`

The admitted observation scope covers ordinary Excel ListObject construction,
table identity/ranges, headers, data bodies, totals rows, row-context formulas,
composite structured-reference formulas, body/totals formulas, table resize,
row/column insert/delete/reorder, header edit, table rename, accepted isolated
table delete, explicit table-move capture rejection, explicit save/reopen
capture rejection, empty data-body observations, first-row insert, last-row
delete, empty-table column rename, current-row absence diagnostics, and
multi-table/name/anchor collision availability.

OxCalc treats these as black-box Excel observations only. WorkbookConstructionSpec
remains construction input and provenance; it is not a semantic shortcut.

## Still Open

1. Excel dependency graph, dirty-set, and invalidation event-order internals are
   still typed unavailable unless OxXlPlay can retain direct black-box evidence.
2. Dynamic structured-reference or `INDIRECT` table evidence is not required for
   the current `calc-4vs8.52` close. If W056 later admits an Excel-comparable
   dynamic structured-reference lane rather than an OxCalc typed exclusion,
   OxXlPlay should add a successor observation bead covering that exact lane.
3. Broader `xlplay_excel_oracle_fixture_pack_001` style work remains OxXlPlay
   support evolution, not a blocker for the current OxCalc W056 table oracle
   intake.

## Integration Rule

OxXlPlay should keep emitting retained observation payloads with source
metadata, provenance, capture-loss labels, typed COM limits, `table_slice`,
`comparison_value`, `effective_display_text`, `execution_outcome`, and
`table_update_oracle` views where applicable. OxReplay remains the comparison
owner; OxCalc remains the TreeCalc table dependency/invalidation owner.
