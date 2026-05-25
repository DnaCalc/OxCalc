# HANDOFF-CALC-006: W056 Table Rollout Coordination

Source repo: OxCalc
Source workset: W056 TreeCalc full reference and table lowering
Source bead: calc-4vs8.55
Filed: 2026-05-24
Status: filed

## Purpose

Coordinate the remaining node-associated table rollout across DnaTreeCalc,
OxFml, OxFunc, DnaOneCalc, OxXlPlay, OxReplay, and OxVba without introducing
private adapters or semantic mirrors.

The intended long-term shape remains:

1. DnaTreeCalc owns product table state, UX, persistence, corpus activation,
   and retained producer artifacts.
2. OxCalc owns TreeCalc table custody, virtual Excel anchors, table catalog
   resolution, sparse readers, dependency facts, invalidation, dynamic rebind,
   caller context, and prepared identity inputs.
3. OxFml owns generic structured-reference packets, host-reference/name binding
   machinery, lexical LET/LAMBDA scope, and W074 name/call evidence.
4. OxFunc owns function and UDF semantics over ordinary values and opaque
   ReferenceLike carriers.
5. OxXlPlay owns black-box Excel workbook/table observation only.
6. OxReplay owns retained comparison/diff/explain over declared payloads only.
7. OxVba supplies VBA/XLL metadata into the OxFunc/OxFml registry surfaces; it
   does not define TreeCalc table semantics.

## Current OxCalc Contract Floor

OxCalc has landed the W056 table contract floor through the following local
beads:

1. `calc-4vs8.44`: whole-system node-table architecture and ownership map.
2. `calc-4vs8.45`: virtual Excel-anchor identity for node-associated tables.
3. `calc-4vs8.46`: generic OxFml structured-reference packet intake contract.
4. `calc-4vs8.47`: table catalog resolver and namespace versioning.
5. `calc-4vs8.48`: sparse ReferenceLike reader surface for table selections.
6. `calc-4vs8.49`: table formula row-context and prepared identity.
7. `calc-4vs8.50`: complete table dependency and invalidation matrix.
8. `calc-4vs8.56`: dynamic table rebind and INDIRECT-style typed-exclusion
   surface.

This floor is OxCalc-owned. It does not close DnaTreeCalc product activation,
Excel oracle evidence, OxReplay retained comparison, W074 non-table name/call
freeze, or W093 registry formula-call migration.

## Required Counterpart Work

### DnaTreeCalc

Existing local anchors observed:

1. `dtc-z0i.5`: W004 table structured-reference bridge activation parent.
2. `dtc-z0i.5.1` through `dtc-z0i.5.5`: table model, bridge projection,
   structured-reference corpus, update retained artifacts, and lifecycle
   callback slices are present as table-spine anchors.
3. `dtc-z0i.5.6`: empty-body table activation is the current empty-body /
   transition residual.
4. `dtc-z0i.7`: dynamic/cross-workspace/profile bridge activation is the
   broader non-table dynamic lane.

Required DnaTreeCalc follow-up:

1. Add or extend local W004/W056 beads so dynamic table references are explicit:
   static and selector-driven `INDIRECT` to table, column, section, current-row,
   cross-workspace table, renamed/moved/deleted table, unavailable workspace,
   unsupported runtime structured-reference text, and non-table dynamic target.
2. Add or extend retained artifacts for empty-body transitions, lifecycle
   callbacks, full namespace/anchor/workspace table pairing, and dynamic table
   selector cases so OxReplay can consume them without TreeCalc-private shims.
3. Keep every case routed through direct OxCalcTreeContext public table projection,
   structured-reference packet, sparse-reader, lifecycle, and dynamic rebind
   APIs. DnaTreeCalc must not parse formula text or reconstruct private span
   keys.

OxCalc counterpart: `calc-4vs8.51`.

### OxXlPlay

Existing local anchors observed:

1. `oxxlplay-4nd`: W056 WorkbookConstructionSpec/table observation parent.
2. `oxxlplay-4nd.1` through `oxxlplay-4nd.5`: WorkbookConstructionSpec
   table-node equivalent, standalone table construction, update oracle,
   delete/save-reopen residuals, and third-pass residual observation packs.

Required OxXlPlay follow-up:

1. Add a successor observation bead if W056 needs Excel black-box evidence for
   structured-table `INDIRECT` forms, dynamic formula strings that resolve to
   structured references, table/current-row dynamic selectors, or comparable
   dynamic table rename/delete/unavailable observations.
2. Preserve the existing rule that WorkbookConstructionSpec is construction
   input and provenance only, not semantic authority.
3. Keep dependency graph, dirty-set, and invalidation event-order internals as
   typed unavailable evidence when Excel COM cannot expose them.

OxCalc counterpart: `calc-4vs8.52`.

### OxReplay

Existing local anchors observed:

1. `oxreplay-p1w`: upstream host-rollout retained artifact blocker.
2. `oxreplay-p1w.1` through `oxreplay-p1w.3`: first DnaTreeCalc artifact
   intake, dependency/invalidation intake, and matched table comparison slices.
3. `oxreplay-qb9`: third-pass full table evidence intake is the current W056
   residual lane.

Required OxReplay follow-up:

1. Keep `oxreplay-qb9` or successors open until retained evidence covers
   empty-body transitions, lifecycle callback artifacts, full
   namespace/anchor/workspace table pairing, dynamic table/INDIRECT cases, and
   any paired Excel observations that are admitted.
2. Compare only declared payloads and exact typed view families. Do not parse
   TreeCalc formula text, Excel formula strings, structured-reference strings,
   or table-selector payloads to infer semantics.
3. Preserve typed non-comparable lanes for Excel dependency/invalidation
   internals and for any producer artifact field whose source repo has not
   emitted a stable comparison view.

OxCalc counterpart: `calc-4vs8.53`.

### OxFml

Existing local anchors observed:

1. `fml-ds0.12`: generic structured-reference packet coverage for virtual
   tables.
2. `fml-ds0.13`: table-context prepared identity and mutation invalidation.
3. `fml-ds0.14`: table-name and structured-reference oracle residual.
4. `fml-ds0.15`: zero-row structured-table packet support.
5. `fml-ds0.6.4`: non-table Excel oracle expansion remains the next W074
   name/call freeze lane.

Required OxFml follow-up:

1. Keep W056 table packets generic: source spans/tokens, table catalog,
   selected columns/sections/regions, caller-context dependence, diagnostics,
   replay identity, and prepared/cache inputs.
2. Do not hardcode TreeCalc table syntax or parse TreeCalc dynamic selector
   strings at runtime. If OxCalc cannot receive a generic bind packet for a
   dynamic table selector, the case remains a typed exclusion.
3. Keep broad host-name/callable precedence blocked on W074 evidence; TreeCalc
   host names and lambda-valued nodes remain mapped to the closest Excel
   defined-name lane until W074 justifies an explicit extension.

OxCalc counterparts: `calc-4vs8.32`, `calc-4vs8.5`, and table consumers under
`calc-4vs8.46` through `calc-4vs8.56`.

### OxFunc

Existing local anchors observed:

1. `oxf-fcdz`: first W051/W056 aggregate opaque-reference admission seed.
2. `oxf-ypq2.13` through `oxf-ypq2.16`: structured-table ReferenceLike
   guardrails, range-taking inventory, sparse-reader widening, and
   reference-visible/context-sensitive classification.
3. `oxf-ypq2.12`: W093 OxFml migration for registry-backed formula-call lookup
   remains open.

Required OxFunc follow-up:

1. Keep structured-table carriers opaque. OxFunc must consume ReferenceLike and
   resolver/reader APIs only; it must not inspect TreeCalc table ids,
   selectors, dynamic selector payloads, or virtual anchor internals.
2. Finish W093 formula-call registry lookup migration with OxFml before UDF
   invalidation can be frozen across table formulas.
3. Route any function that needs table row visibility, filter/hidden-row state,
   source formula text, or reference metadata through generic host/query
   interfaces or typed exclusions, not TreeCalc branches.

OxCalc counterpart: `calc-4vs8.54`.

### DnaOneCalc

Required DnaOneCalc follow-up:

1. No ordinary single-formula execution should require a host namespace, table
   catalog, or host-reference resolver.
2. LET/LAMBDA lexical variables, callable locals, captures, and returned
   lambdas remain OxFml-internal.
3. Future VBA/XLL UDF support should flow through OxFunc/OxFml registry
   surfaces, not DnaOneCalc-local function mirrors.

No W056 table-specific DnaOneCalc bead is required unless table/reference
packets start leaking into no-host single-formula execution.

### OxVba

Required OxVba follow-up:

1. Align VBA/XLL discovery metadata with OxFunc W093 registration requests and
   invocation descriptors.
2. Do not expose TreeCalc selectors or table payloads as VBA/XLL semantic
   fields unless a future public ReferenceLike metadata API explicitly admits
   them.

No W056 table-specific OxVba bead is required for table closure; its role is
registry metadata feeding W093.

### Impact-Scan-Only Repos

`OxIde`, `DnaOxIde`, `DnaVisiCalc`, and `Foundation` do not need W056
table-rollout beads from this packet unless a future UI, doctrine, replay-pack,
or shared-interface dependency is discovered. The current W056 table closure
path should not route implementation through those repos.

## Promotion Gates

W056 table promotion is blocked until:

1. DnaTreeCalc emits active corpus and retained artifacts for the full admitted
   table scope, including dynamic table and cross-workspace table cases or
   explicit typed exclusions.
2. OxXlPlay has retained Excel observations for every Excel-comparable table
   lane needed by W056, with typed capture limits where Excel COM cannot expose
   internals.
3. OxReplay validates/replays/diffs/explains declared DnaTreeCalc and OxXlPlay
   table evidence without parsing private strings or importing semantic
   ownership.
4. OxFml keeps structured references and dynamic/host packets generic and W074
   non-table name/call freeze remains separately gated.
5. OxFunc W093 registry mutation/formula-call migration is stable enough for
   table formulas that can bind registered functions.
6. DnaOneCalc no-host formula execution remains unaffected.

No repo may close the whole table topic by eager materialization, private
bridges, formula-string parsing outside the owning layer, or duplicated
TreeCalc/Excel semantics.
