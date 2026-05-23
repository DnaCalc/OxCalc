# W056 TreeCalc Full Reference And Table Lowering

Status: `in_progress`

Parent predecessor: `W051` first reference-carrier pattern

Parent epic: `calc-4vs8`.

Initial successor beads:

1. `calc-4vs8.1` — TreeReference variant inventory and host-reference correlation.
2. `calc-4vs8.2` — structured table dependency lowering.
3. `calc-4vs8.3` — dependency invalidation and dynamic rebind widening.
4. `calc-4vs8.21` through `calc-4vs8.29` — first node-associated
   TreeCalc table completion spine: table-node snapshot projection,
   TreeCalc table-path structured-reference prebind, table reference readers,
   per-row column-formula runtime, update/invalidation scenarios, retained
   TreeCalc/OxReplay evidence, and Excel update-oracle intake for the declared
   table slice.
5. `calc-4vs8.34` through `calc-4vs8.38` — second-pass table product-promotion
   spine: full-scope closure map, lifecycle/callback contract freeze,
   structured-table `ReferenceLike` function breadth, matched TreeCalc/Excel
   replay/value-wire intake, and final table-topic promotion audit.
6. `calc-4vs8.39` through `calc-4vs8.43` — third-pass full-intended table
   support spine: empty data-body packet/reader support, executable function
   breadth evidence, DnaTreeCalc lifecycle bridge acceptance, table namespace/
   anchor/workspace collision semantics, and final table support audit without
   relying on the earlier typed projection exclusions.
7. `calc-4vs8.44` through `calc-4vs8.56` — fourth-pass comprehensive
   node-table design and rollout spine: whole-system ownership map, virtual
   Excel-anchor identity, generic OxFml packet contract, OxCalc resolver/
   namespace versioning, full `ReferenceLike` reader surface, row-context
   prepared identity, complete dependency/invalidation matrix, DnaTreeCalc
   activation, OxXlPlay oracle construction, OxReplay retained evidence,
   UDF/VBA/XLL impact, cross-repo handoff coordination, and dynamic table
   reference rebind/`INDIRECT` semantics.
8. `calc-4vs8.30` through `calc-4vs8.33`, plus `calc-8tox` — remaining
   non-table reference completion spine: cross-workspace provider/alias
   semantics, workspace-qualified carriers, reference literals/dynamic
   carriers, W074 callable/name intake, and retained non-table corpus evidence.

## 1. Purpose

W056 widens the W051 first `ChildrenV1` carrier into the full TreeCalc
reference and table-lowering scope.

W051 proves the first pattern: an OxCalc-owned reference collection can be
correlated to an OxFml host-reference handle, lowered into membership and
member-value dependency facts, invalidated on membership/order changes, and
transported as an opaque `ReferenceLike` without OxFunc learning TreeCalc
syntax.

W056 owns the broader closure: all admitted TreeCalc reference variants,
dynamic rebinding, host namespace versioning, caller-context identity, and
structured table row/column/header/totals dependency lowering.

## 2. Scope

In scope:

1. TreeCalc reference variants beyond `ChildrenV1`, including ancestor,
   sibling, preceding/following, explicit path, dynamic, unresolved,
   host-sensitive, capability-sensitive, cross-workspace, and future
   recursive/set-producing selectors.
2. Dependency descriptors and reverse edges for scalar references,
   set-membership references, member-value references, dynamic rebind
   outcomes, unresolved references, and host-sensitive references.
3. Invalidation facts for namespace mutation, caller-context mutation,
   dynamic rebind, base deletion, membership change, order change, member value
   publication, cross-workspace availability, and capability denial.
4. Host namespace versioning and caller-context identity as prepared-identity
   inputs.
5. Correlation of OxCalc reference carriers to OxFml host-reference handles
   and formal-reference/runtime-input records.
6. Structured table lowering using the converged first packet:
   `table_catalog`, `enclosing_table_ref`, and `caller_table_region`.
7. Table dependencies for row membership, column identity/range,
   header-region identity, totals-row identity, `#This Row` caller-region
   facts, and omitted-table-name enclosing-table facts.

Out of scope:

1. OxFml grammar or parser ownership.
2. OxFunc worksheet function semantics or TreeCalc-specific function branches.
3. DNA TreeCalc product UI, persistence, or orchestration.
4. Private OxFml/OxFunc shims or TreeCalc-only evaluator bridges.

## 3. Ownership

OxCalc owns TreeCalc model custody, reference resolution, dependency edges,
invalidation facts, dynamic rebind policy, host namespace versioning, caller
context identity, and table lowering into calculation dependencies.

OxFml owns formula grammar, generic host formula context consumption,
structured-reference grammar and bind normalization, name/call precedence, and
canonical evaluator/runtime artifacts.

OxFunc owns values, references as function inputs, worksheet function
semantics, and argument-admission metadata.

Integration rule: OxCalc talks to OxFml through generic `HostFormulaContext`
and public runtime/replay surfaces only. W056 must not add OxFml-private
bridges, OxFunc TreeCalc semantics, or parser shims in host repos.

## 4. Initial Lanes

1. full TreeReference variant inventory and compatibility matrix,
2. reference carrier to OxFml host-reference correlation contract,
3. dependency descriptor and reverse-edge widening,
4. invalidation fact widening,
5. dynamic rebind and host namespace versioning,
6. caller-context identity and relative-reference replay,
7. cross-workspace reference availability and degradation,
8. structured table packet intake,
9. row/column/header/totals dependency lowering,
10. W074/W036 OxFml evidence intake and handoff watch,
11. end-to-end TreeCalc reference/table scenarios.

## 4A. `calc-4vs8.1` Implementation-Input Surface

The first W056 tranche turns the full TreeCalc reference inventory into typed
OxCalc-owned implementation inputs in
`src/oxcalc-core/src/formula.rs`:

1. `TreeReferenceInventoryVariant` names the admitted and blocked reference
   families, including current concrete carriers, set-producing selectors,
   cross-workspace references, structured table references, and bare
   name/callable references.
2. `TreeReferenceImplementationInput` records, per variant, whether the
   family is a current carrier, an admitted implementation input, or a typed
   exclusion; the host-reference correlation need; namespace and caller-context
   identity needs; dependency descriptor facts; invalidation facts; and any
   successor bead.
3. Existing `TreeReference` values map back to this inventory through
   `TreeReference::inventory_variant()` and
   `TreeReference::implementation_input()`.

Current admitted implementation inputs are not a product-complete full
TreeCalc reference claim. They are the typed work inputs for the remaining
W056 beads. In particular:

1. structured table references are admitted implementation inputs linked to
   `calc-4vs8.2`, `calc-4vs8.4`, and `calc-4vs8.9`,
2. dependency/reverse-edge, dynamic rebind, namespace, and caller-context
   widening continues in `calc-4vs8.3`,
3. formula-call registry-view admission and capability-denied runtime
   classification have landed in OxFml W074 `fml-ds0.7`; final bare
   name/callable precedence remains blocked on the broader W074 Excel oracle
   matrix,
4. cross-workspace now has the versioned workspace availability/degradation
   packet, provider/alias resolver shape, and workspace-qualified carrier/
   dependency facts; DnaTreeCalc/OxReplay corpus activation remains open.
   Resolved ordered selector packets now have carriers via `calc-4vs8.12`,
   while remaining raw selector parser/resolver packets, traversal bounds, and
   corpus activation remain open.

## 4B. `calc-4vs8.2` Structured Table Lowering Surface

The second W056 tranche adds the first OxCalc-owned typed structured table
dependency-lowering surface in `src/oxcalc-core/src/structured_table.rs`.

Current implemented scope:

1. consumes only public OxFml table-context packet types:
   `table_catalog`, `enclosing_table_ref`, `caller_table_region`, stable row
   membership/order identities, and exact header/totals region refs,
2. accepts public OxFml `StructuredReferenceBindRecord` packets and maps them
   into normalized `StructuredTableReferenceIntake` values rather than parsing
   formula text or mirroring structured-reference grammar,
3. lowers available facts for table identity, stable row membership, stable row
   order, selected column identity, header text, exact header region, data
   region, exact totals region, caller row context for `#This Row`, and
   omitted-table-name enclosing-table dependency,
4. preserves table dependencies as context-only dependency descriptors so the
   dependency graph retains them without inventing TreeNodeId reverse edges,
5. records typed blockers only when optional packet facts are absent or when the
   packet states that the requested header/totals/caller row context does not
   exist.

Current non-claim:

This is an implemented OxCalc intake/lowering surface for the current generic
packet, including the stable table fact fields added by OxFml `fml-ds0.8` and
the exercised normalized structured-reference bind packets added by OxFml
`fml-ds0.9`. It is not full structured table behavior. Full behavior remains
blocked until DnaTreeCalc/OxReplay retained table evidence runs through the
real bridge and the remaining W056 cross-workspace/name/selector surfaces are
exercised.

## 4B.1. Node-Associated Table Completion Spine

The structured table packet work above is the substrate, not the product
closure. A DnaTreeCalc table is a node-associated data surface that OxCalc must
project into the generic Excel-shaped table packet OxFml already understands.
That projection must behave like an Excel table anchored at a cell for OxFml,
while preserving TreeCalc ownership of the node, row ids, column ids, table
identity, and structural invalidation facts.

The added W056 table spine is:

1. `calc-4vs8.21` — define the OxCalc table-node snapshot and virtual
   Excel-anchor projection contract. The projection must preserve stable
   `table_node_id` / `table_id`, row membership/order identities, column
   identities, header/totals refs, virtual workbook/sheet/range refs, and table
   context identity without introducing `EvalValue::Table`.
2. `calc-4vs8.22` — add public TreeCalc structured-reference prebind for
   table paths such as `path[Col]`, `path[@Col]`, `path[#Headers]`,
   `path[#Data]`, `path[#Totals]`, and composite section/column forms. The
   packet must preserve the original TreeCalc source spans/tokens while feeding
   OxFml only generic structured-reference/table facts.
3. `calc-4vs8.23` — extend the sparse/reference-reader pattern to table column,
   data, header, totals, whole-table, and current-row selections, preserving
   opaque `ReferenceLike` carriage through OxFunc and avoiding eager
   materialization as closure evidence.
4. `calc-4vs8.24` — evaluate table column formulas per row using the same
   OxFml formula text plus row-specific `caller_table_region`, with prepared
   identity including table/caller/registry/host/structure inputs.
5. `calc-4vs8.25` — make update and invalidation scenarios executable for body
   edits, row insert/delete/reorder, column insert/delete/reorder/rename, header
   edits, totals toggles/edits, table rename/move/delete, save/reopen, and
   structural rebind.
6. `calc-4vs8.26` — perform the retained evidence and seam audit using
   DnaTreeCalc active corpus, OxXlPlay Excel observation, and OxReplay retained
   comparison outputs.
7. `calc-4vs8.27` — consume the DnaTreeCalc residual full table corpus work:
   `#All`, bracket-escaped table names, bracket-escaped column names, composite
   escaped refs, current-row escaped refs, and typed table/column diagnostics
   through the real LiveOxCalc/OxCalc bridge.
8. `calc-4vs8.28` — consume retained DnaTreeCalc/OxReplay producer and
   comparison artifacts for `table_slice`, table-cell/per-node values,
   effective display where TreeCalc can state it or typed unavailability where
   it cannot, execution outcome, dependency evidence, invalidation evidence,
   and retained artifact refs.
9. `calc-4vs8.29` — consume OxXlPlay/OxReplay table update-oracle evidence for
   body, row, column, header, totals, rename, move, delete, save/reopen, and
   structured-reference recalculation scenarios.

Cross-repo counterpart beads now exist for the table spine:

1. DnaTreeCalc `dtc-z0i.5.1` through `dtc-z0i.5.4` own table-node persistence,
   bridge projection, active corpus activation, update scenarios, and retained
   producer artifacts.
2. OxFml `fml-ds0.12` through `fml-ds0.14` own generic structured-reference
   bind packet breadth, table-context prepared identity, and table/name oracle
   residuals without TreeCalc semantics.
3. OxFunc `oxf-ypq2.13` and `oxf-ypq2.14` own opaque structured-table
   `ReferenceLike` guardrails and range-taking function inventory.
4. OxXlPlay `oxxlplay-4nd.1` through `oxxlplay-4nd.3` own workbook/table
   observation fixtures and update/oracle observations.
5. OxReplay `oxreplay-p1w.1` through `oxreplay-p1w.3` own retained producer
   artifact intake, dependency/invalidation evidence intake, and final
   table-scope diff/explain closure.
6. Third-pass full-intended table support anchors are: OxFml `fml-ds0.15`
   for zero-row generic table packets; DnaTreeCalc `dtc-z0i.5.5` and
   `dtc-z0i.5.6` for lifecycle bridge acceptance and empty-body corpus;
   OxFunc `oxf-ypq2.15`/`oxf-ypq2.16` for broader structured-table
   `ReferenceLike` function evidence; OxXlPlay `oxxlplay-4nd.5` for residual
   Excel table observation; and OxReplay `oxreplay-qb9` for third-pass retained
   evidence intake.

Current audit intake from `calc-4vs8.26`: DnaTreeCalc commits `b59b2fb` and
`8eba3cb` activate the table structured-reference corpus through
LiveOxCalc/OxCalc paths, including `#All`, bracket-escaped table and column
names, composite escaped refs, escaped current-row refs, unknown-column
diagnostics, row formula values, totals formula values, dependency lowering,
update classification, and retained TreeCalc producer artifacts under
`../DnaTreeCalc/docs/test-runs/w056-table-structured-references-001/`.
OxReplay commits `b341f8b` and `e6de7a4` accept that retained producer and
dependency/invalidation evidence through validate-bundle/replay/diff/explain
without parsing TreeCalc structured-reference text. That closes
`calc-4vs8.27` and `calc-4vs8.28`.

OxXlPlay commit `8176223` first closed `oxxlplay-4nd.3` with retained
`xlplay_table_update_oracle_001` artifacts, and commit `c3a4c88` closes the
residual `oxxlplay-4nd.4` update-oracle breadth. OxReplay commits `c387bc9`,
`9e4c503`, and `16791fb` accept the refreshed artifact through
validate-bundle/replay, admit `table_update_oracle` as opaque exact JSON, and
retain self-diff/explain evidence. The admitted Excel-observed update slice now
covers body edit, row insert/delete/reorder, column insert/delete/reorder/rename,
header edit, totals-row toggle/edit, table rename, table resize, structured-
reference formula recalculation, accepted isolated `table_delete`, explicit
`table_move` unavailability, typed `save_reopen` capture rejection, and
`execution_outcome.class_id`. Excel dependency/invalidation internals remain
explicitly unavailable from Excel COM rather than inferred by OxCalc or
OxReplay.

That closes the `calc-4vs8.29` Excel update-oracle intake blocker. The final
`calc-4vs8.26` audit no longer depends on missing retained Excel update breadth;
OxReplay commit `7e21a6f` closes `oxreplay-p1w.3` with matched TreeCalc/Excel
table diff/explain evidence under
`../OxReplay/docs/test-corpus/bundles/host_rollout_matched_table_001/` and
`../OxReplay/docs/test-runs/w007-host-rollout-host_rollout_matched_table_001-baseline/`.
The residual shared `comparison_value` helper replacement remains
`BLK-REPLAY-003`, but it is a final shared-value implementation cleanup rather
than a TreeCalc table semantic or matched-scenario closure blocker.

The architecture rule for all of these beads is strict: OxCalc/DnaTreeCalc own
TreeCalc table meaning and structural identity; OxFml owns generic structured
reference parsing/binding; OxFunc owns function semantics over opaque carriers;
OxXlPlay observes Excel; OxReplay compares retained declared payloads. No repo
may close a table bead by parsing another repo's private strings or mirroring
another repo's semantics.

## 4B.2. Final Node-Associated Table Audit

Product status:

The W056 node-associated table slice is complete for the declared structured
table scope. OxCalc can project a TreeCalc node table as an Excel-shaped virtual
table context for OxFml, lower generic structured-reference packets into
OxCalc-owned row/column/header/totals/caller-row dependencies, provide sparse
reference readers without adding `EvalValue::Table`, evaluate per-row table
formulas with row-specific caller context, and record update/invalidation facts
for the declared table edits.

Supported structured-reference scope:

1. table paths and omitted-table/caller-row forms: `path[Col]`, `path[@Col]`,
   `[#Headers]`, `[#Data]`, `[#Totals]`, `#All`, composite structured refs,
   escaped table names, escaped column names, escaped composite refs, and
   escaped current-row refs,
2. table reference carriers/readers: whole table, data body, selected column,
   current row, headers, totals, and sparse blank/defined traversal,
3. table formula runtime: one formula text evaluated per table row through
   generic OxFml table context, stable dispatch skeleton reuse, row-specific
   caller context identity, totals formula execution, and typed rejection of
   current-row references outside data rows,
4. update and invalidation facts: body value/formula edits, row
   insert/delete/reorder, column insert/delete/reorder/rename, header edits,
   totals toggle/formula edit, table rename/move/delete, save/reopen identity
   preservation, structural rebind, and unaffected reader identity stability.

Evidence:

1. OxCalc engine/runtime table beads `calc-4vs8.21` through `calc-4vs8.25`
   closed with Rust coverage and `cargo test -p oxcalc-core`.
2. DnaTreeCalc retained table artifacts landed at
   `../DnaTreeCalc/docs/test-runs/w056-table-structured-references-001/`,
   covering the active LiveOxCalc/OxCalc table corpus, row formulas, totals
   formulas, diagnostics, dependency lowering, update classification, and
   retained replay projection.
3. OxReplay retained intake accepts the DnaTreeCalc artifact at
   `../OxReplay/docs/test-runs/dnatreecalc-w056-table-structured-references-001-baseline/`
   for `table_slice`, value/display/outcome, dependency evidence,
   invalidation evidence, and retained artifact refs without parsing TreeCalc
   structured-reference text.
4. OxXlPlay retained Excel oracle artifacts at
   `../OxXlPlay/states/excel/xlplay_table_update_oracle_001/` now cover the
   declared update family, including accepted isolated `table_delete`, typed
   `save_reopen` capture rejection, explicit `table_move` unavailability, and
   `execution_outcome.class_id`.
5. OxReplay accepts the refreshed OxXlPlay oracle at
   `../OxReplay/docs/test-runs/oxxlplay-seam-xlplay_table_update_oracle_001-baseline/`
   and admits `table_update_oracle` as opaque exact JSON through
   `table_update_oracle_json_exact`.
6. OxReplay closes matched TreeCalc/Excel table comparison mechanics at
   `../OxReplay/docs/test-corpus/bundles/host_rollout_matched_table_001/`
   with retained diff/explain baselines at
   `../OxReplay/docs/test-runs/w007-host-rollout-host_rollout_matched_table_001-baseline/`.
   The retained diff is equivalent across `comparison_value`,
   `effective_display_text`, `execution_outcome`, `table_slice`,
   `table_update_oracle`, `dependency_evidence`, `invalidation_evidence`, and
   `retained_artifact_ref`.

Known exclusions and non-table residuals:

1. Excel COM does not expose internal dependency graphs, dirty-set contents, or
   invalidation event order for table updates. Those are retained as explicit
   capture limits in the OxXlPlay artifact and do not move into OxCalc runtime
   semantics.
2. OxXlPlay retains `save_reopen` as a typed capture rejection because hidden
   Excel COM `SaveCopyAs`/open can block unattended observation on the exercised
   host. OxCalc still has engine-side save/reopen identity coverage for its
   model state.
3. OxXlPlay retains `table_move` as typed unavailability for this oracle slice.
   OxCalc owns TreeCalc table move/rebind semantics and tests them locally.
4. OxReplay `oxreplay-p1w.3` is closed for matched TreeCalc/Excel table
   comparison mechanics. The remaining `BLK-REPLAY-003` shared
   `comparison_value` helper replacement is a value-wire implementation cleanup,
   not TreeCalc table parsing, lowering, dependency, or invalidation behavior.

Formal status:

This audit is product/evidence closure for the W056 table slice only. It does
not close the broader W056 reference runtime, W074 name/call precedence,
cross-workspace references, raw ordered-selector syntax, or the remaining
DnaTreeCalc W004/W005 non-table reference corpus.

## 4B.3. Second-Pass Table Product Promotion Spine

The first node-associated table spine proves the architecture and the declared
table slice. It is not the whole long-term table topic. Full product promotion
needs a second pass that freezes the table lifecycle contract and compares the
retained TreeCalc/Excel table evidence without moving semantics into the wrong
repo.

Additional W056 beads:

1. `calc-4vs8.34` — table full-product closure map and ownership audit. This
   audits the closed table slice against the intended long-term product table
   support and records the authoritative remaining table plan across OxCalc,
   DnaTreeCalc, OxFml, OxFunc, OxXlPlay, and OxReplay.
2. `calc-4vs8.35` — table lifecycle callback contract and version policy. This
   freezes the OxCalc-owned input/callback surface for table create/delete,
   rename/move, row and column edits, header/totals/body/formula edits,
   save/reopen identity, structural rebind, version bumps, dependency facts,
   diagnostics, and prepared-identity contributions.
3. `calc-4vs8.36` — structured-table `ReferenceLike` function breadth intake.
   This widens beyond the first aggregate proof by coordinating OxFunc/OxFml
   evidence for range/table-consuming functions through opaque sparse
   references, with typed exclusions where a function needs richer host context.
4. `calc-4vs8.37` — matched TreeCalc/Excel table replay and value-wire intake.
   This consumes OxReplay matched-scenario evidence, local `comparison_value`
   comparator evidence with the shared-helper replacement retained as
   `BLK-REPLAY-003`, `execution_outcome` class identity, and table
   diff/explain baselines without parsing private TreeCalc or Excel strings.
5. `calc-4vs8.38` — final table-topic product promotion audit. This promotes
   table support only after the first slice plus the second-pass table beads
   have evidence, repo-local checks, fresh-eyes review, and explicit exclusions.

Systematic cross-repo sequence:

1. OxCalc executes `calc-4vs8.34` first, reconciling this workset, the workset
   register, active worklist, and open blocker text so the remaining table plan
   is visible and does not reopen closed scoped work.
2. OxCalc then executes `calc-4vs8.35`, keeping TreeCalc table lifecycle meaning
   in OxCalc/DnaTreeCalc and passing only generic table context to OxFml.
3. OxFunc/OxFml execute or reconcile the structured-table `ReferenceLike`
   breadth behind `calc-4vs8.36`; OxFunc owns function semantics and must not
   inspect TreeCalc table selectors.
4. DnaTreeCalc keeps table corpus/product evidence on the real
   LiveOxCalc/OxCalc bridge, including persistence, structural updates, and
   retained artifacts, without adding formula parsing.
5. OxXlPlay supplies Excel observation only; capture limits remain typed
   observation facts and do not become OxCalc runtime semantics.
6. OxReplay closes the matched table comparison/value-wire lane consumed by
   `calc-4vs8.37`, comparing declared retained payloads only.
7. OxCalc executes `calc-4vs8.38` as the final table-topic audit and keeps any
   remaining non-table W056 blockers under the non-table spine.

Known counterpart anchors at creation time:

1. DnaTreeCalc `dtc-z0i.5` remains the table structured-reference bridge
   activation parent after `dtc-z0i.5.1` through `dtc-z0i.5.4` supplied the
   first table corpus, update, and retained-artifact slices.
2. OxFml `fml-ds0.12` is the closed generic W056 structured-reference packet
   coverage slice; broader W036 table formula semantics and non-table W074
   name/call precedence remain separate.
3. OxFunc `oxf-ypq2.15` and `oxf-ypq2.16` are the current structured-table
   sparse-reader/function-classification lanes that feed `calc-4vs8.36`.

Third-pass full-intended table support beads:

The second-pass table spine promotes the declared node-associated table topic
with explicit typed exclusions. It is not the final long-term table product
finish line. The third pass exists so the remaining table exclusions and
counterpart implementation lanes have ordinary bead ownership instead of being
left as prose caveats.

1. `calc-4vs8.39` — empty-body table packet and reader support. This removes
   the typed projection exclusion for zero-row data bodies by coordinating a
   generic OxFml table packet shape, OxCalc projection/readers, OxFunc sparse
   reference behavior, DnaTreeCalc corpus activation, and OxXlPlay/OxReplay
   retained evidence for empty headers/data/totals and first-row/last-row
   updates.
2. `calc-4vs8.40` — structured-table `ReferenceLike` function implementation
   evidence. This converts the `calc-4vs8.36` inventory into executable
   OxFunc/OxFml/OxCalc evidence for admitted range/table consumers such as
   shape, indexed-reference, range-scan, lookup/match, and criteria-aggregate
   families, with typed host-context lanes for dynamic-array, subtotal/
   aggregate, metadata, volatile reference, and operator cases.
3. `calc-4vs8.41` — DnaTreeCalc table lifecycle bridge acceptance. This proves
   DnaTreeCalc sends real create/delete, rename/move, row/column/header/totals/
   body/formula, save/reopen, and structural-rebind events into the OxCalc
   lifecycle callback contract with persistence and retained evidence.
4. `calc-4vs8.42` — table namespace, anchor collision, and workspace semantics.
   This closes the multi-table, table-name collision, virtual anchor movement,
   workspace-qualified table reference, alias/`!`, canonicalization, and
   structured-reference source-preservation edge cases, deferring only those
   name/call decisions that genuinely depend on W074.
5. `calc-4vs8.43` — full intended table support final audit. This promotes the
   whole table topic only after the third-pass residual beads and counterpart
   repo evidence are closed or have explicit user-accepted typed exclusions.
   OxVba/future-UDF participation is an impact-scan or typed non-impact row
   unless a concrete table-reference UDF/XLL path is admitted.

Systematic third-pass cross-repo sequence:

1. OxFml widens only generic table/structured-reference packets for zero-row
   ranges or other missing generic shapes; it does not learn TreeCalc table
   paths, lifecycle, or selectors.
2. OxCalc consumes those generic packets, owns the TreeCalc table projection,
   dependency, invalidation, lifecycle, namespace, virtual-anchor, and prepared
   identity facts, and keeps `EvalValue` free of table-specific variants.
3. OxFunc implements admitted function families over opaque `ReferenceLike`
   and sparse-reader APIs; any function needing hidden/filter/metadata/spill
   policy gets typed generic host-context requirements instead of table
   branches.
4. DnaTreeCalc supplies product table lifecycle/corpus events through the real
   LiveOxCalc/OxCalc bridge and emits retained artifacts; it does not parse
   formulas or mirror OxCalc dependency classification.
5. OxXlPlay observes Excel workbook/table behavior where Excel can state it;
   capture limits stay typed observation facts.
6. OxReplay compares declared retained payloads and classifies non-comparable
   lanes without importing TreeCalc or Excel private semantics.
7. OxCalc closes `calc-4vs8.43` only when the table topic no longer depends on
   ordinary typed exclusions for intended support; parent W056 may still remain
   open for non-table references.

Fourth-pass comprehensive table completion spine:

The third pass made the residuals explicit, but the long-term feature needs a
more architectural breakdown so node-associated tables do not grow quick fixes
or parallel paths. The fourth pass is the controlling table completion spine for
the final W056 table audit.

1. `calc-4vs8.44` — whole-system architecture and ownership map. This bead
   states the desired end state: a DnaTreeCalc node table is exposed to OxFml
   as an Excel-shaped ListObject/table context anchored at a virtual cell,
   while TreeCalc custody, identity, dependencies, namespace versions, caller
   context, and invalidation remain OxCalc-owned.
2. `calc-4vs8.45` — virtual Excel anchor and identity contract. This covers
   stable table handle, table node id, virtual workbook/sheet/range, header/
   data/totals regions, row membership/order, column identity, table structure
   and value versions, caller row identity, and save/reopen semantics.
3. `calc-4vs8.46` — generic structured-reference packet contract for node
   tables. This keeps OxFml responsible for generic structured-reference
   parsing/binding and requires exact source spans/tokens, selected sections/
   token kinds, regions/columns, effective table identity, diagnostics,
   caller-context dependency, and replay identity without TreeCalc semantics.
4. `calc-4vs8.47` — table catalog resolver and namespace versioning. This
   covers table names, node paths, workspace aliases, root/workspace anchors,
   omitted-table row context, unavailable workspace/table states, table/name
   collisions, and W074-gated name/call boundaries.
5. `calc-4vs8.48` — full table `ReferenceLike` reader surface. This closes
   reference-preserving readers for whole table, data body, selected columns,
   multi-column ranges, headers, totals, `#All`, current row, omitted-table
   current row, empty tables, sparse blanks, errors, and stable reader identity.
6. `calc-4vs8.49` — table formula row-context and prepared identity. This
   covers per-row formulas, totals formulas, `#This Row`, omitted-table
   current-row context, LET/LAMBDA lexical locals remaining OxFml-internal,
   dispatch skeleton reuse, registry snapshot identity, and DnaOneCalc
   no-host-reference guardrails.
7. `calc-4vs8.50` — complete table dependency and invalidation matrix. This
   covers row/column/header/totals/data/caller-row/anchor/workspace/function
   dependencies and every intended table update: value/formula edits,
   row/column insert/delete/reorder, renames, totals toggles, table move/delete,
   node move/delete, save/reopen, workspace availability, and structural rebind.
8. `calc-4vs8.51` — DnaTreeCalc full node-table corpus and bridge activation.
   This is the product activation lane for persistence, UX-relevant table
   interactions, path-qualified references, omitted/current-row forms, escaped
   names, empty-body transitions, formulas, diagnostics, cross-workspace table
   references, and retained producer artifacts through the real bridge.
9. `calc-4vs8.52` — OxXlPlay Excel table oracle and workbook construction.
   This maps TreeCalc node-table scenarios onto Excel ListObject fixtures and
   captures observed structured-reference behavior plus typed COM limitations.
10. `calc-4vs8.53` — OxReplay retained table comparison and promotion evidence.
   This owns validate/replay/diff/explain baselines for DnaTreeCalc and
   OxXlPlay table artifacts, including table slices, values, display, outcomes,
   dependency/invalidation evidence, prepared identity, source preservation,
   function admission, and capability snapshots.
11. `calc-4vs8.54` — table UDF, VBA, XLL, and registry impact scan. This keeps
   future UDF support registry-backed through OxFunc/OxFml and verifies table
   references remain opaque `ReferenceLike` values rather than TreeCalc-specific
   function branches.
12. `calc-4vs8.55` — cross-repo table rollout and handoff coordination. This
   ensures counterpart beads or handoff packets exist in the affected repos and
   prevents downstream work from filling missing contracts with private adapters.
13. `calc-4vs8.56` — dynamic table reference rebind and `INDIRECT` semantics.
   This keeps table-valued dynamic references, renamed/moved/deleted dynamic
   table targets, cross-workspace dynamic table targets, volatile rebind, and
   typed denial for unsupported dynamic structured-reference forms under the
   table spine rather than smuggling them through the non-table dynamic lane.

`calc-4vs8.43` now depends on this fourth-pass spine as well as the third-pass
residual beads. It must not close the full intended table topic until these
beads are either closed with evidence or explicitly converted into
user-accepted typed exclusions.

## 4B.3. Whole-System Node-Table Architecture Map

This section is the controlling architecture map for `calc-4vs8.44`. Later
table beads may refine details, but they must preserve this ownership shape.

Desired end state:

1. DnaTreeCalc exposes a table as product state attached to a TreeCalc node.
   It owns user-facing creation, editing, persistence, shell/skin behavior,
   corpus activation, and retained producer artifacts.
2. OxCalc is the table semantic custodian for calculation. It owns table
   identity, virtual Excel-anchor projection, catalog lookup, row/column
   membership, dependency edges, invalidation, dynamic rebind, caller context,
   sparse readers, and prepared/cache identity inputs.
3. OxFml sees an ordinary Excel-shaped table environment: a generic
   `TableDescriptor` catalog, optional enclosing table, optional
   `caller_table_region`, generic structured-reference bind packets, source
   spans/tokens, diagnostics, and opaque identity tokens. It owns formula
   grammar, parse/bind, `LET`/`LAMBDA` lexical scope, structured-reference
   normalization, and W074 name/call evidence.
4. OxFunc sees scalars, arrays, or opaque `ReferenceLike`/reader-backed
   references. It owns function semantics, UDF registration, capability
   overlays, and registry snapshot identity. It never inspects TreeCalc table
   selectors, table paths, row ids, or column ids.
5. OxXlPlay observes the nearest Excel ListObject behavior by constructing
   ordinary workbooks with anchored tables. It records observed values,
   formulas, class ids, and typed COM capture limits; it does not define
   OxCalc invalidation semantics.
6. OxReplay validates and compares declared retained payloads. It may normalize
   and explain table evidence, but it does not parse TreeCalc formulas, infer
   Excel private dependency graphs, or replace the producing repo's authority.
7. DnaOneCalc remains the no-host-reference guardrail. Ordinary single-formula
   use still requires no table catalog or host resolver; `LET`/`LAMBDA`
   locals/callables remain OxFml-internal. Future VBA/XLL UDFs enter through
   OxFunc/OxFml registry surfaces, not DnaOneCalc-local function mirrors.
8. OxVba participates only by supplying future VBA/XLL discovery metadata that
   can feed OxFunc registration requests. It does not own table-reference
   semantics.

Architecture value:

1. the same generic OxFml/OxFunc path serves worksheet-like tables, TreeCalc
   node tables, DnaOneCalc no-host formulas, and future registered UDFs,
2. TreeCalc structural meaning stays where structural edits are visible, in
   DnaTreeCalc/OxCalc,
3. Excel compatibility can be observed through OxXlPlay without copying Excel
   private internals into OxCalc,
4. replay comparison receives declared facts from producers instead of
   reconstructing semantics from strings,
5. later table breadth adds facts to typed interfaces rather than creating
   table-only side channels.

Interface flow:

1. DnaTreeCalc sends OxCalc a node-table snapshot or lifecycle callback packet.
   The packet carries product identities and edits: table node, table id/name,
   row ids/order, column ids/order, formulas, totals/header state, persistence
   identity, and changed rows/columns.
2. OxCalc projects that snapshot into two records:
   - a TreeCalc-owned projection record retaining node paths, row/column ids,
     formula metadata, table namespace version, table invalidation identity,
     workspace availability, and lifecycle versions,
   - a generic OxFml table context containing table id/name, virtual
     workbook/sheet/range, header/data/totals region refs, column descriptors,
     opaque row membership/order tokens, enclosing table ref, and optional
     `caller_table_region`.
3. OxFml parses and binds formula text against only the generic context. It
   returns generic host-reference and structured-reference bind records with
   exact source spans/tokens, stable handles, selected sections/regions/
   columns, effective table identity, caller-context dependency, typed
   diagnostics, and replay identity.
4. OxCalc lowers those bind records into TreeCalc table selections. It creates
   dependency descriptors, reverse/context edges, invalidation facts, dynamic
   rebind facts, and sparse `ReferenceLike` readers keyed to stable handles and
   reader identities.
5. OxFunc evaluates function calls over the scalar/array/reference inputs it is
   given. If a function needs additional generic context, such as hidden-row,
   filter, spill, metadata, or volatile reference policy, that requirement is
   a typed OxFunc/OxFml/OxCalc interface fact, not a TreeCalc branch.
6. OxCalc publishes accepted results through its coordinator and emits retained
   evidence. OxReplay validates/diffs/explains declared payloads. OxXlPlay
   supplies Excel-observed comparison artifacts where Excel can state behavior.

Required typed interfaces:

1. `TreeCalcTableNodeSnapshot` and lifecycle callback packets describe product
   table state and edits from DnaTreeCalc to OxCalc.
2. `TreeCalcTableNodeProjection` and virtual-anchor identity describe how
   OxCalc maps node tables into Excel-shaped table context.
3. The table catalog resolver returns stable handles, opaque selectors,
   resolution layer, shape hint, effective table identity, virtual anchor
   identity, caller-context dependency/id, namespace versions, workspace
   availability version, and typed diagnostics.
4. OxFml structured-reference bind records preserve source spans/tokens,
   selected regions/sections/columns, effective table identity, diagnostics,
   caller-context dependency, and replay identity.
5. Table sparse readers expose `declared_extent`, `defined_cardinality`,
   `defined_iter`, `read_at(coord) -> Defined(EvalValue) | Blank`,
   `contains(coord)`, and stable `reader_identity`.
6. Prepared/cache identity includes host namespace version, table namespace
   version, structure context version, table context identity, caller context
   identity, registry snapshot identity, capability profile identity,
   resolution rule version, workspace availability version, and dynamic selector
   identity where relevant.
7. Retained evidence records table slices, values, display/outcome facts,
   dependency/invalidation facts, source preservation, prepared identity,
   function reference admission, Excel observations, and typed projection gaps.

Update and rebind lifecycle:

1. Body value edits invalidate value dependencies and affected readers but do
   not change table namespace or prepared identity unless a formula-visible
   context fact changes.
2. Formula edits invalidate affected prepared callables, registry-sensitive
   bindings where applicable, and the published values that depend on them.
3. Row insert/delete/reorder changes row membership/order versions, reader
   identity where traversal shape changes, and caller-row identity for affected
   per-row formulas.
4. Column insert/delete/reorder/rename changes column identity/order/header
   facts, structured-reference binding outcomes, and any prepared formulas that
   selected those columns by name or range.
5. Header edits change header text dependencies and may change structured
   reference binding if they alter canonical column identity.
6. Totals toggle/edit changes totals-region identity and totals formula/value
   dependencies; current-row references in totals context remain typed rejects.
7. Table rename/path change changes table namespace facts and invalidates
   prepared identities that relied on table lookup, without forcing OxFml to
   know TreeCalc paths.
8. Table move or virtual anchor movement changes virtual workbook/sheet/range
   identity and invalidates any prepared or replay artifact whose observable
   structured-reference meaning depends on the anchor.
9. Table delete, node delete, workspace close, or unavailable workspace
   transitions produce typed unavailable/deleted diagnostics and release old
   membership/value/context dependencies.
10. Dynamic table references and `INDIRECT`-style selectors rebind through
   OxCalc dynamic-reference facts. If the dynamic target would require
   unsupported structured-reference parsing or ambiguous name/call precedence,
   the outcome is a typed exclusion or W074-gated blocker, not local inference.
11. Save/reopen must preserve stable logical table identity where the product
   model says the table is the same object. If persistence changes a version or
   anchor identity, the change must be replay-visible and invalidate prepared
   identity deterministically.

Hard non-goals:

1. no `EvalValue::Table` variant as a shortcut for table objects,
2. no OxFml parser branch for TreeCalc table paths,
3. no OxFunc branch that inspects TreeCalc table selectors or row/column ids,
4. no DnaTreeCalc formula parser or dependency classifier,
5. no OxReplay comparison rule that derives semantics by parsing producer
   private strings,
6. no eager materialization as closure evidence for reference-preserving table
   behavior,
7. no DnaOneCalc host resolver requirement for ordinary single-formula use,
8. no private adapters between repos when a typed public packet or handoff is
   missing.

Promotion gates from this map:

1. `calc-4vs8.45` may specify virtual-anchor identity only within the
   projection and identity rules above.
2. `calc-4vs8.46` may widen generic OxFml packet facts, but cannot ask OxFml to
   parse TreeCalc table paths.
3. `calc-4vs8.47` owns resolver and namespace facts, with final bare name/call
   precedence still W074-gated.
4. `calc-4vs8.48` and `calc-4vs8.49` must prove reference-preserving readers
   and row-context prepared identity through generic OxFml/OxFunc inputs.
5. `calc-4vs8.50` owns invalidation facts; Excel observation may inform cases
   but does not replace OxCalc dependency semantics.
6. `calc-4vs8.51`, `calc-4vs8.52`, and `calc-4vs8.53` provide product corpus,
   Excel observation, and retained comparison evidence respectively.
7. `calc-4vs8.54` keeps future UDF/VBA/XLL work registry-backed and opaque.
8. `calc-4vs8.55` owns the cross-repo handoff/dependency graph so downstream
   repos do not bridge around missing contracts.
9. `calc-4vs8.56` owns dynamic table rebind, including `INDIRECT`-style cases,
   under this same generic host-reference and sparse-reader model.

## 4B.4. `calc-4vs8.45` Virtual Anchor And Identity Contract

The node-table virtual anchor is the contract that lets OxFml bind structured
references as though the table were an Excel ListObject anchored on a sheet,
while OxCalc keeps table ownership in the TreeCalc model.

The contract has three identity layers:

1. Stable logical table identity: `table_node_id`, `table_id`, table display
   path, canonical path, and table name/version. These identify the product
   table and its namespace binding.
2. Generic Excel-shaped projection identity: virtual workbook scope, virtual
   sheet scope, start row/column, table range, header/data/totals region refs,
   table context identity, and opaque row membership/order and column tokens
   carried in `TableDescriptor`.
3. OxCalc-only dependency identity: row membership/order facts, row values,
   column identity, body/totals formula metadata, table invalidation identity,
   and lifecycle version state. These never become OxFml semantics.

Identity and update rules:

1. An unchanged save/reopen preserves stable table id, table node id, virtual
   anchor identity, table context identity, and table invalidation identity.
2. A table rename, display path change, canonical path change, or namespace
   version change preserves the table id but changes namespace identity and the
   table context identity used for prepared/cache invalidation.
3. A workspace alias, workbook/sheet scope, or virtual anchor movement changes
   virtual anchor identity and table context identity. Row and column
   dependency identities remain stable when their membership/order/content has
   not changed.
4. Row reorder changes row-order identity and table context identity while
   preserving row-membership identity when the set is unchanged.
5. Row insert/delete changes row-membership identity, row-order identity, table
   range, and caller-row-sensitive prepared identity inputs.
6. Column insert/delete/reorder/rename changes column identity and any
   structured-reference binding that selected affected columns.
7. Header, totals, body formula, table delete, table move, and structural
   rebind effects are classified by the lifecycle/update contracts, but they
   consume the same identity layers rather than a separate table path.

Current executable surface:

1. `TreeCalcTableNodeSnapshot` supplies the stable logical table id, virtual
   anchor, row/column identity versions, and namespace version.
2. `project_treecalc_table_node_snapshot` produces `TreeCalcTableNodeProjection`
   with a generic `TableDescriptor`, `StructuredTableContextPacket`, table
   context identity, table invalidation identity, namespace token, virtual
   anchor token, row membership/order identities, column identity, and formula
   metadata tokens.
3. Focused tests now include
   `virtual_anchor_identity_contract_separates_table_namespace_anchor_and_membership_changes`,
   proving unchanged save/reopen stability, namespace/path changes, workspace
   alias changes, anchor movement, row reorder, and row insertion effects.

`calc-4vs8.35` implemented contract:

The table lifecycle boundary is now represented in
`src/oxcalc-core/src/structured_table.rs` by
`TreeCalcTableLifecycleCallbackPacket`,
`TreeCalcTableLifecycleVersionState`,
`TreeCalcTableLifecycleContextVersions`, and
`classify_treecalc_table_lifecycle_callback`. This is the OxCalc/DnaTreeCalc
callback contract for node-associated table state transitions; it is not an
OxFml table semantic surface.

The callback packet names the lifecycle event, optional before/after version
states, context versions, owner node ids, source host-reference handles, and
changed row/column ids. The event set covers table create/delete, body
value/formula edits, row insert/delete/reorder, column insert/delete/reorder,
column rename, header text edit, totals toggle/formula edit, table rename/move,
save/reopen, and structural rebind. Create is after-only, delete is
before-only, ordinary updates are before/after, and malformed packet shapes emit
typed diagnostics.

`TreeCalcTableLifecycleVersionState` carries the stable table/node/row/column
identity inputs DnaTreeCalc and OxCalc must preserve across callbacks:
`table_node_id`, `table_id`, display/canonical path, virtual workbook and sheet
scope refs, table range/header/totals/data-region refs, virtual anchor
identity/token, table context identity, table invalidation identity, table
namespace identity/version, row membership/order identities and versions, column
identity/version, stable row ids, and stable column ids.
`TreeCalcTableLifecycleContextVersions` contributes host namespace, structure
context, registry snapshot, and resolution-rule identity fragments.

The classifier turns a packet into the OxCalc-owned product facts used by the
dependency graph and prepared/cache invalidation:

1. changed dependency kinds, including row membership/order, column identity,
   header text, table regions, caller row context, and structural rebind facts,
2. invalidation reasons for value changes, row/column/header/totals changes,
   table rename/move/delete, save/reopen, and structural rebind,
3. prepared identity inputs for host namespace, structure context, table
   context, caller context, resolution rule, registry snapshot when relevant,
   and structural rebind,
4. invalidation seed identities that keep OxCalc table lifecycle facts
   correlated to source host-reference handles without exposing TreeCalc
   semantics to OxFml or OxFunc,
5. typed diagnostics for missing or unexpected before/after states, missing
   owner node, and stable table-node/table-id violations.

Executable scope:

1. `table_lifecycle_callback_matrix_covers_full_w056_update_set` exercises the
   callback packet path for body edits, formula edits, row insert/delete/reorder,
   column insert/delete/reorder/rename, header edit, totals toggle/formula edit,
   table rename/move/delete, save/reopen, and structural rebind,
2. focused structured-table tests cover row-order lifecycle invalidation,
   create/delete lifecycle packet shape, source host-reference handle
   preservation, changed row/column ids, prepared identity inputs, invalidation
   seed generation, callback identity, and stable-id diagnostics,
3. save/reopen is part of the event and scenario set; the engine-side
   classifier can leave readers stable when no table version/input changed,
   while still carrying host namespace, structure context, registry snapshot,
   and resolution-rule context-version identity for retained replay and
   prepared/cache invalidation,
4. DnaTreeCalc counterpart work remains anchored under `dtc-z0i.5` and its
   successor residuals: DnaTreeCalc must call this packet boundary from the real
   table product lifecycle rather than adding local formula parsing or private
   table invalidation semantics.

Non-claim:

This contract completes the OxCalc side of the table lifecycle callback/version
policy for W056. It does not by itself promote the full table topic: function
breadth remains under `calc-4vs8.36`, retained matched replay/value-wire intake
under `calc-4vs8.37`, and final product promotion under `calc-4vs8.38`.

## 4B.5. `calc-4vs8.46` Generic Structured-Reference Packet Contract

Product status:

OxCalc can now consume and emit the W056 node-table structured-reference packet
shape without formula-text parsing. The consumed packet is still the public
OxFml `StructuredReferenceBindRecord`; TreeCalc table-path ownership stays in
OxCalc/DnaTreeCalc, while OxFml owns generic structured-reference parsing,
section/column selection, source-token preservation, diagnostics, and runtime/
replay projection.

Required packet facts:

1. stable `bind_record_handle`,
2. exact `source_span_utf8` over the authored formula text,
3. exact `source_token_text`,
4. typed `source_token_kind` preserving structured-reference token
   classification,
5. explicit-table versus omitted-table facts,
6. effective table id/name when binding succeeds,
7. selected column ids, selected section qualifiers, and selected region
   descriptors,
8. `uses_this_row` and `caller_context_dependent`,
9. optional generic resolved-reference descriptor,
10. typed diagnostic links for recognized bind failures.

OxCalc-specific node-table prebind facts:

1. path span/token and structured-tail span/token are preserved separately from
   the generic source token,
2. table path tokens resolve only against OxCalc/DnaTreeCalc
   `TreeCalcTableNodeProjection` values,
3. `host_ref_handle`, `selector_payload`, caller-context dependency, and
   replay identity are OxCalc-owned and correlate back to the generic bind
   record,
4. `StructuredTableReferenceIntake` stores the generic `source_token_kind` so
   dependency lowering and sparse readers can preserve token classification
   without re-reading formula text.

Counterpart evidence:

1. OxFml `fml-ds0.16` adds the public typed
   `StructuredReferenceSourceTokenKind` field and asserts it for explicit,
   omitted, zero-row, diagnostic, and runtime/replay structured-reference
   packets.
2. OxCalc focused tests assert that TreeCalc table prebinds and
   `StructuredTableDependencyLoweringRequest::from_oxfml_bind_record` preserve
   `source_token_kind = StructuredReference`.

Non-goals:

1. OxFml does not parse TreeCalc table paths, table lifecycle, row identity,
   column identity, or invalidation semantics.
2. OxCalc does not reimplement OxFml's structured-reference grammar. It only
   recognizes TreeCalc table path prefixes and forwards the structured tail
   through the generic packet shape.
3. Bare host names, table-name collisions, and callable/lambda-valued host
   nodes remain in the W074 name/call precedence evidence lane.

## 4B.6. `calc-4vs8.47` Table Catalog Resolver And Namespace Versioning

Product status:

OxCalc now has a public table-catalog resolver for node-associated TreeCalc
tables. The resolver treats a TreeCalc node table like an Excel-shaped table
anchored at a virtual cell for OxFml purposes, while keeping table lookup,
workspace qualification, namespace versioning, caller-context dependency, and
diagnostics in OxCalc.

Implemented Rust surface:

1. `TreeCalcTableCatalogResolveRequest` describes the host lookup request:
   explicit table name/path, same-node table, omitted-table caller context, or
   stable table id.
2. `TreeCalcTableCatalogResolverContext` carries current workspace, table
   projections, workspace aliases, workspace availability, host namespace
   version, structure-context version, resolution-rule version, W074-gated
   namespace-adjacency facts, and deleted-table facts.
3. `resolve_treecalc_table_catalog_reference` returns a
   `TreeCalcTableCatalogResolution` with stable table-reference handle, opaque
   selector, resolution layer, shape hint, effective table id/node id, virtual
   anchor identity, caller-context dependency/id, host namespace version, table
   namespace version, structure-context version, resolution-rule version,
   workspace availability version, and typed diagnostics.
4. `TreeCalcTableNodeProjection` now carries the table namespace version
   directly, so resolver identity and prepared/cache invalidation do not need to
   reconstruct it from private snapshot state.

Covered lookup scenarios:

1. same-workspace table-name lookup,
2. canonical/display/bracket-escaped path lookup through the existing
   projection token set,
3. first-position `!` current-workspace-root lookup,
4. workspace alias and direct workspace-qualified lookup,
5. same-node table lookup,
6. omitted-table lookup through `caller_table_region`,
7. unavailable workspace lookup,
8. deleted table lookup,
9. ambiguous table selector diagnostics,
10. W074-gated adjacency diagnostics for host names, functions, defined names,
    and lambda-valued nodes.

Identity and invalidation facts:

1. the stable resolver handle includes selector identity, resolution layer,
   shape hint, effective table id/node id, virtual anchor identity, host
   namespace version, table namespace version, structure-context version,
   resolution-rule version, workspace availability version, caller-context id,
   and diagnostic class set,
2. alias target changes, table namespace changes, workspace availability
   changes, and caller-context changes therefore change prepared/cache identity
   deterministically,
3. explicit structured-reference routing remains unblocked because OxCalc
   supplies the generic table context and OxFml still sees only
   `TableDescriptor`, enclosing-table, caller-region, and structured-reference
   bind records.

Evidence:

Focused Rust tests cover current workspace table lookup, same-node lookup,
omitted caller-table lookup, current-root lookup, workspace alias lookup,
direct workspace-qualified lookup, exact stable-table-id lookup, unavailable
workspace diagnostics, deleted table diagnostics, table selector ambiguity,
W074-gated namespace adjacency, bracket-escaped `!` tokens, caller-row
invalidation under coarse caller ids, and handle changes after alias/namespace
mutation.

Still open:

1. `calc-4vs8.48` must close the full table `ReferenceLike` reader surface.
2. `calc-4vs8.49` must close row-context formula/prepared identity behavior.
3. `calc-4vs8.50` must close dependency and invalidation matrices for all
   table lifecycle updates.
4. Bare host-name/callable precedence remains OxFml W074-gated; this resolver
   records adjacency diagnostics but does not freeze precedence.

`calc-4vs8.36` implemented intake:

OxCalc now records the structured-table function breadth surface as typed Rust
inventory in `src/oxcalc-core/src/structured_table.rs`:
`TREECALC_STRUCTURED_TABLE_FUNCTION_ADMISSION_INVENTORY` and
`treecalc_structured_table_function_admission`. The inventory is an OxCalc
intake and coordination surface; OxFunc still owns the function implementation
and must consume generic sparse/reference resolver APIs without seeing TreeCalc
selectors.

Current admitted and blocked lanes:

1. `SUM`, `COUNT`, `COUNTA`, and `COUNTBLANK` are the current
   reference-preserving evidence lane. Runtime tests execute them through
   OxFml/OxFunc with sparse `ReferenceLike` bindings, not table-specific
   `EvalValue` variants and not dense eager arrays.
2. `ROWS` and `COLUMNS` are admitted pending OxFunc evidence as shape-only
   consumers of declared reference extent.
3. `INDEX` is admitted pending OxFunc evidence as a resolver-indexed reference
   consumer that needs coordinate reads and reference-returning forms.
4. `MATCH`, `XMATCH`, `XLOOKUP`, `VLOOKUP`, `HLOOKUP`, and `LOOKUP` are
   admitted pending OxFunc evidence as lookup/match functions requiring generic
   sparse cells, declared extent, and multi-range alignment.
5. ordinary range scans such as `AVERAGE`, `MIN`, `MAX`, `PRODUCT`,
   statistical functions, logical scans, `TEXTJOIN`, and `CONCAT` are admitted
   pending OxFunc evidence as sparse `ReferenceLike` consumers over generic
   reader APIs.
6. `SUMIF`, `SUMIFS`, `COUNTIF`, `COUNTIFS`, `AVERAGEIF`, `AVERAGEIFS`,
   `MAXIFS`, and `MINIFS` are admitted pending OxFunc evidence as
   criteria/value-range consumers requiring aligned sparse references.
7. `FILTER`, `SORT`, `SORTBY`, `UNIQUE`, `TAKE`, `DROP`, `CHOOSECOLS`,
   `CHOOSEROWS`, `TOCOL`, `TOROW`, `WRAPROWS`, and `WRAPCOLS` require typed
   dynamic-array/spill policy before product admission over table references.
8. `SUBTOTAL` and `AGGREGATE` require typed row-hidden/filter/subtotal context;
   table-specific branches are not acceptable closure evidence.
9. `AREAS`, `FORMULATEXT`, `CELL`, `ROW`, `COLUMN`, and `ADDRESS` require typed
   metadata disclosure and caller-context rules.
10. `OFFSET`, `INDIRECT`, `OP_IMPLICIT_INTERSECTION`, and `OP_SPILL_REF` require
   typed dynamic rebind, reference coordinate, caller, or spill context.
11. `CALL` is a typed exclusion for native invocation policy and must not be
    used as a table-reference closure shortcut.

The inventory links the OxFunc counterpart beads `oxf-ypq2.15` and
`oxf-ypq2.16`. Each lane explicitly records that OxFunc must not inspect
TreeCalc selectors and that eager materialization is not allowed as closure
evidence. Focused tests assert those invariants, ensure function names are
assigned to only one lane, and verify the first aggregate group continues to
travel through sparse `ReferenceLike` values with only defined cells carried.

Non-claim:

This closes OxCalc's W056 function-breadth intake and coordination contract. It
does not close OxFunc W093 implementation for the pending groups, dynamic-array
spill semantics, row-hidden/filter context, metadata disclosure policy, native
CALL policy, or final table promotion.

`calc-4vs8.37` implemented intake:

OxCalc now consumes OxReplay commit `7e21a6f`, which closes `oxreplay-p1w.3`
for the matched TreeCalc/Excel table comparison scope. The retained matched
bundle lives at
`../OxReplay/docs/test-corpus/bundles/host_rollout_matched_table_001/`, with
regenerated diff/explain baselines at
`../OxReplay/docs/test-runs/w007-host-rollout-host_rollout_matched_table_001-baseline/`.

Consumed replay families:

1. `comparison_value`: equivalent through OxReplay's local typed comparator
   seam for the retained scalar value (`6` versus `6.0`). The final replacement
   with an OxFunc-owned replay-wire helper remains `BLK-REPLAY-003` and is not
   treated as TreeCalc table semantics.
2. `effective_display_text`: equivalent retained display text (`$6.00`) without
   widening OxCalc display policy.
3. `execution_outcome`: equivalent typed accepted/evaluation outcome with
   `class_id = accepted:value`.
4. `table_slice`: exact retained table slice comparison over declared payloads;
   OxReplay does not parse TreeCalc structured-reference text or Excel formulas.
5. `table_update_oracle`: exact opaque retained oracle payload comparison
   through `table_update_oracle_json_exact`.
6. `dependency_evidence` and `invalidation_evidence`: explicit typed
   non-comparable/observation-limit lanes where Excel COM does not publish the
   internal graph or invalidation event order. OxCalc keeps ownership of
   TreeCalc dependency and invalidation facts.
7. `retained_artifact_ref`: exact retained artifact reference comparison tying
   the matched replay to the DnaTreeCalc producer, OxXlPlay oracle, and
   OxReplay intake baselines.

The retained `diff.json` reports `equivalent: true` with no mismatches; the
retained `explain.json` reports equivalent scenarios. This closes the OxCalc
side of the matched replay/value-wire intake for `calc-4vs8.37` without
creating a private adapter bridge or moving replay comparison policy into
OxCalc runtime semantics.

Non-claim:

This closes the matched table replay/evidence intake only. It does not close
the final table-topic promotion audit (`calc-4vs8.38`), the OxFunc-owned
shared value helper replacement, or the remaining non-table W056 reference
spine.

`calc-4vs8.38` final table-topic promotion audit:

Product status:

The node-associated TreeCalc table topic is promoted for the declared W056 table
scope. OxCalc treats a table attached to a TreeCalc node as an Excel-shaped
virtual table context for OxFml, while preserving TreeCalc table identity,
structure, dependencies, invalidation, and update semantics in OxCalc/
DnaTreeCalc. OxFml receives only generic structured-reference/table packets;
OxFunc receives opaque sparse/reference/value surfaces; OxXlPlay observes Excel
behavior; OxReplay compares retained declared payloads.

Promoted support:

1. table projection: stable virtual workbook/sheet/table/anchor identity for a
   node-associated table without requiring `EvalValue::Table`,
2. structured-reference binding intake: explicit table paths,
   omitted-table/caller-row forms, `#This Row`, `#Headers`, `#Data`, `#Totals`,
   `#All`, selected columns, composite references, and bracket-escaped table
   and column names through generic OxFml packets,
3. dependency and invalidation ownership: row membership/order, column identity,
   header text, data/totals regions, caller row context, structural rebind,
   lifecycle callbacks, and prepared-identity version inputs remain OxCalc facts,
4. sparse/reference runtime: table data, selected column, current row, headers,
   totals, and whole-table carriers use sparse `ReferenceLike`/reader surfaces
   and do not rely on dense eager materialization as closure evidence,
5. table formula runtime: one formula text can be evaluated per data row through
   row-specific caller context, with totals formula support and typed rejection
   of current-row references outside data rows,
6. function breadth intake: first aggregate group is implemented through opaque
   sparse references, and broader function families are inventoried as admitted
   pending OxFunc evidence, richer host-context lanes, or typed exclusions,
7. retained evidence: DnaTreeCalc, OxXlPlay, and OxReplay retained artifacts
   cover the table producer slice, Excel update oracle, matched table
   diff/explain comparison, dependency/invalidation observation limits, and
   retained artifact refs.

Promotion evidence:

1. OxCalc beads `calc-4vs8.21` through `calc-4vs8.29` closed the first
   node-associated table spine: projection, prebind, sparse readers, per-row
   formula runtime, update/invalidation scenarios, retained table evidence, and
   Excel update-oracle intake.
2. OxCalc beads `calc-4vs8.34` through `calc-4vs8.37` closed the second-pass
   table spine: full closure map, lifecycle callback/version contract,
   structured-table function-breadth intake, and matched TreeCalc/Excel
   replay/value-wire intake.
3. DnaTreeCalc retained table corpus evidence is at
   `../DnaTreeCalc/docs/test-runs/w056-table-structured-references-001/`.
4. OxXlPlay retained Excel table/update oracle evidence is at
   `../OxXlPlay/states/excel/xlplay_table_update_oracle_001/`.
5. OxReplay retained matched comparison evidence is at
   `../OxReplay/docs/test-corpus/bundles/host_rollout_matched_table_001/` and
   `../OxReplay/docs/test-runs/w007-host-rollout-host_rollout_matched_table_001-baseline/`.

Remaining exclusions and separate work:

1. W056 remains open for non-table reference families, broad W004/W005 corpus
   activation, callable host names/lambda-valued nodes, and W074 name/call
   precedence.
2. `calc-4vs8.39` through `calc-4vs8.56` now own the full-intended table
   residuals and architecture/rollout work that were acceptable exclusions for
   the declared table-topic promotion but are not acceptable final product gaps.
3. OxFunc remains responsible for implementing pending broader function-family
   admissions over opaque references; OxCalc has only inventoried and classified
   those lanes for table promotion, with executable follow-through now tracked
   by `calc-4vs8.40`.
4. `BLK-REPLAY-003` remains the future OxFunc/OxReplay shared
   `comparison_value` helper replacement. It is outside TreeCalc table
   semantics and does not block the table-topic promotion.
5. Empty data-body tables remain a typed projection exclusion for the earlier
   promotion slice only; `calc-4vs8.39` owns removing that exclusion for full
   intended table support.
6. Excel COM dependency/invalidation internals, `save_reopen` capture, and
   table-move observation limits remain typed retained observation facts rather
   than OxCalc runtime semantics.

Formal status:

The final table-topic audit promotes the declared node-associated TreeCalc
table scope only. It does not close parent W056, final non-table references,
W074 name/call precedence, W093 pending function implementation lanes, or the
future shared replay value-helper cleanup.

Promotion rule:

The second-pass table beads may promote the table topic only if the architecture
still has one owner for each concern: OxCalc/DnaTreeCalc for TreeCalc table
identity and lifecycle, OxFml for generic structured-reference binding, OxFunc
for function semantics over opaque carriers, OxXlPlay for Excel observation,
and OxReplay for retained comparison. Any private bridge, string parser, or
duplicated semantic mirror blocks promotion.

## 4C. Non-Table Reference Completion Spine

After `calc-4vs8.26`, the first table slice is no longer the active W056
blocker. The remaining work now splits into the second-pass table
product-promotion spine above and the non-table reference runtime/evidence spine
below.

New execution beads:

1. `calc-4vs8.30` — cross-workspace alias and availability semantics. This
   owns workspace-provider lookup, workspace aliases, first-position `!`
   alias/base-token behavior, availability-version prepared identity, and typed
   degradation diagnostics. OxCalc now has the provider/alias packet shape and
   local resolver tests.
2. `calc-8tox` — workspace-qualified cross-workspace carrier and dependency
   path. This owns preserving external workspace identity through formula
   carriers, dependency descriptors, workspace reverse-edge facts, and prepared
   identity inputs without collapsing external targets to local `TreeNodeId`.
3. `calc-4vs8.31` — reference literals, mixed reference arrays, and dynamic
   references. This owns typed carriers or exclusions for explicit reference
   literals, ordered/duplicate-preserving reference arrays, scalar/reference
   rejection behavior, dynamic `INDIRECT`/CTRO-style rebind, and prepared
   identity inputs.
4. `calc-4vs8.32` — callable host-name and W074 precedence intake. This remains
   blocked on OxFml W074 evidence for built-in/UDF/defined-name/
   defined-name-LAMBDA/table-name/lexical/host-reference precedence and
   namespace mutation invalidation. TreeCalc host names and lambda-valued nodes
   stay mapped to the closest Excel defined-name lane until W074 justifies an
   explicit extension.
5. `calc-4vs8.33` — full non-table reference corpus and retained evidence
   intake. This consumes DnaTreeCalc/OxReplay evidence for walk-up, dotted
   descent, anchors, aliases, escaping/canonicalization, meta accessors,
   sibling/preceding/following/ancestor/recursive selectors, reference
   literals/arrays, dynamic references, cross-workspace references, and
   node-as-function/lambda-valued nodes.

The broad blocker `calc-4vs8.5` now tracks the remaining non-table/upstream
W056 reference closure gap. It no longer treats the first node-associated table
slice, second-pass table promotion, DnaTreeCalc table corpus residuals,
OxXlPlay table update oracles, OxReplay table-update-oracle policy, or matched
TreeCalc/Excel table replay evidence as missing packet/lowering blockers.

Implementation note for `calc-4vs8.21`: OxCalc now has the first executable
projection surface in `src/oxcalc-core/src/structured_table.rs`.
`TreeCalcTableNodeSnapshot` preserves the TreeCalc-owned node identity,
display/canonical path, virtual anchor, row ids/order, column ids/order,
namespace and version facts, body formula metadata, and totals formula
metadata. `project_treecalc_table_node_snapshot` emits a generic
OxFml `TableDescriptor` with parseable virtual A1 table/column/header/totals
refs and opaque descriptor-visible row membership/order tokens. It also
computes a tokenized generic `table_context_identity` for OxFml prepared/cache
identity and a separate OxCalc-only `table_invalidation_identity` that retains
raw row ids, TreeCalc paths, body formula metadata, and totals formula
metadata. Focused Rust tests prove deterministic projection, range shape
(`B3:D7`, column body ranges, header/totals ranges), separator-framed OxCalc
identity components, and identity changes for table rename/namespace, row
membership add/replace, row order, column rename/add/reorder/version,
header/totals presence, and virtual anchor movement. Empty data-body tables are
currently a typed projection exclusion because the current generic OxFml
`TableDescriptor` requires parseable data-column A1 area refs. The final table
promotion carries this as an explicit typed exclusion; future OxFml packet
widening is required before empty data-body tables can enter the promoted table
scope.

Implementation note for `calc-4vs8.22`: OxCalc now has a first public
TreeCalc table-path structured-reference prebind surface in
`src/oxcalc-core/src/structured_table.rs`.
`prebind_treecalc_table_structured_references` scans authored TreeCalc formula
text only for table-path host-reference tokens and produces generic OxFml
`StructuredReferenceBindRecord` packets. The packet preserves the original
`source_span_utf8`, exact `source_token_text`, typed `source_token_kind`, path
span/token, structured-tail span/token, stable host reference handle, resolved
`table_node_id`/`table_id`, selector payload, caller-context dependency flag,
typed diagnostics, and replay identity. It covers `path[Col]`, `path[@Col]`,
section/column composites such
as `path[[#Headers],[Col]]`, omitted current-row forms such as `[@Col]`, and
diagnostics for unknown table paths or columns. This is a host-hook prebind
surface, not a TreeCalc formula parser and not an OxFml TreeCalc branch.

Implementation note for `calc-4vs8.23`: OxCalc now has the first executable
structured-table sparse/reference-reader carrier surface in
`src/oxcalc-core/src/structured_table.rs`.
`TreeCalcTableSparseReader` consumes a `TreeCalcTableNodeSnapshot`, its
generic virtual `TableDescriptor` projection, and either a public OxFml
`StructuredReferenceBindRecord` or normalized `StructuredTableReferenceIntake`.
It builds a sparse reader with `declared_extent`, `defined_cardinality`,
`defined_iter`, `read_at`, `contains`, stable `reader_identity`, and an opaque
`ReferenceLike` keyed to the OxFml resolved A1 or area target. Runtime bindings
flow through public OxFml sparse-reference value bindings and scalar cell
values for current-row forms, keeping `EvalValue` free of table-specific
variants and keeping OxFunc unaware of TreeCalc selectors.

The implemented `calc-4vs8.23` scope covers:

1. single table columns such as `path[Col]`,
2. contiguous multi-column ranges such as `path[[#Data],[Amount]:[Tax]]`,
3. `#Headers`, `#Data`, `#Totals`, and `#All` row-section projections,
4. omitted/current-row forms such as `[@Col]` when the caller supplies the
   generic enclosing table and caller data-row context,
5. aggregate runtime carriage for `SUM`, `COUNT`, `COUNTA`, and `COUNTBLANK`
   through OxFml/OxFunc sparse `ReferenceLike` handling,
6. typed reader errors for missing caller context, missing selected columns,
   absent header/totals regions, caller-table mismatch, invalid ranges, empty
   selections, range overflow, and currently unsupported non-contiguous column
   selections.

This is reference-preserving table-carrier behavior for the declared reader
slice. It is not full table product closure. Per-row table formula scheduling,
table update/invalidation scenarios, DnaTreeCalc active corpus coverage,
OxXlPlay Excel observation, OxReplay retained comparison, and final table seam
audit remain in `calc-4vs8.24` through `calc-4vs8.26` and their cross-repo
counterparts.

Implementation note for `calc-4vs8.24`: OxCalc now has the first executable
row-context runtime surface for TreeCalc table formulas in
`src/oxcalc-core/src/structured_table.rs`.
`evaluate_treecalc_table_column_formula_rows` evaluates one authored formula
text per data row by supplying OxFml with the generic virtual table catalog,
enclosing table ref, and row-specific `caller_table_region`. It uses the
`TreeCalcTableSparseReader` from `calc-4vs8.23` to build sparse reference
bindings and current-row scalar cell bindings, then invokes OxFml/OxFunc
without asking DnaTreeCalc to emulate `#This Row` semantics. The runtime report
records row id, row offset, primary virtual locus, caller-context identity,
prepared formula key, dispatch skeleton key, registry snapshot identity,
host formula context, and structured-reference bind handles for each row.

`evaluate_treecalc_table_totals_formula` covers the first totals-row formula
path by using the totals virtual locus and a generic totals caller region.
Explicit table column aggregates such as `SUM(path[Col])` flow through the same
sparse `ReferenceLike` carrier path. Current-row references in the totals
region are typed rejects rather than silently coerced to a data row.

This is row-context formula runtime for the declared table-formula slice, not
full table closure. It does not yet perform dependency/invalidation scenario
closure for row/column/header/totals edits, retained DnaTreeCalc corpus
activation, or OxReplay/OxXlPlay comparison; those remain in `calc-4vs8.25`
and `calc-4vs8.26`.

Implementation note for `calc-4vs8.25`: OxCalc now has the first executable
table update/dependency/invalidation scenario surface in
`src/oxcalc-core/src/structured_table.rs`.
`classify_treecalc_table_update` maps body value edits, body formula edits,
row insert/delete/reorder, column insert/delete/reorder/rename, header text
edits, totals row toggle/formula edits, table rename/move/delete, save/reopen,
and structural rebind into OxCalc-owned dependency kinds, invalidation reasons,
prepared-identity inputs, source-reference handle correlation, and invalidation
seeds. The surface keeps body cell value edits as value invalidations rather
than prepared-identity churn, while row/column/region/table-shape changes
enter table context and caller-context identity as appropriate.

`validate_treecalc_table_reference_after_update` supplies typed post-update
diagnostics for deleted tables, missing selected columns, missing caller row
context, and absent header/totals regions. Focused tests also prove save/reopen
identity preservation and that a body value edit does not change an unaffected
table reader identity. This closes the OxCalc-local update/invalidation
scenario model, but retained DnaTreeCalc/OxXlPlay/OxReplay product evidence
and final seam audit remain in `calc-4vs8.26`.

## 4C. `calc-4vs8.3` Dependency, Invalidation, And Rebind Surface

The third W056 tranche adds the first OxCalc-owned typed projection over the
dependency graph for the admitted TreeReference carrier families in
`src/oxcalc-core/src/tree_reference_rebind.rs`.

Current implemented scope:

1. projects existing `DependencyGraph` descriptors into typed W056 descriptor
   facts with source-reference handle, target, descriptor kind, namespace
   identity need, caller-context identity need, invalidation facts, and
   prepared-identity invalidation consequence,
2. exposes target reverse-edge facts for concrete `TreeNodeId` dependencies
   and context reverse-edge facts for retained context-only descriptors such
   as structured table and runtime-fact carriers,
3. exposes dynamic rebind facts that distinguish unresolved dynamic potential
   from resolved dynamic target edges and list the activation, release,
   reclassification, and upstream-publication invalidation facts,
4. groups host namespace, structure-context, resolution-rule,
   capability-profile, table-context, cross-workspace, and caller-context
   identity requirements as prepared-identity invalidation inputs,
5. preserves cross-workspace references as a typed blocker requiring a
   versioned cross-workspace availability/degradation model rather than
   inventing an executable fallback.

Current non-claim:

This is an implemented typed OxCalc surface over current descriptors and graph
facts. It is not full runtime behavior for every W056 carrier. End-to-end
runtime closure remains blocked where the cross-repo program has not yet
emitted exercised generic host-reference, final name/call precedence,
cross-workspace oracle packet surfaces, raw selector parser/resolver packet
surfaces plus traversal-bound evidence, or retained full-bridge evidence. The
bounded W074 registry/capability slice is
no longer missing: OxFml `fml-ds0.7` at commit `9da8456` proves runtime
registry-view formula-call admission and capability-denied classification, and
OxFml `fml-ds0.9` at commit `6895e6a` proves normalized structured-reference
bind packet projection, but neither freezes built-in/UDF/defined-name or
defined-name-`LAMBDA` precedence or TreeCalc name/call semantics.

## 4D. `calc-4vs8.6` Runtime Prepared-Identity Contribution

The fourth W056 tranche consumes the typed dependency/rebind identity needs
during local TreeCalc runtime preparation in `src/oxcalc-core/src/treecalc.rs`.

Current implemented scope:

1. derives W056 namespace and caller-context identity needs from the translated
   TreeCalc reference carriers and runtime-fact carriers before calling the
   public OxFml runtime prepare path,
2. projects host namespace, resolution-rule, capability-profile, structure
   context, caller-context, table-context, and cross-workspace availability
   identity inputs through OxFml's public `RuntimeHostFormulaContext` where the
   current public surface can carry them,
3. keeps the first `ChildrenV1` host-reference bind results and sparse
   reference-values path on the same public OxFml runtime surface,
4. records W056 prepared-identity requirement diagnostics so the runtime path
   exposes which namespace/caller-context classes contributed,
5. includes the OxFml `prepared_formula_key` in the local per-edge value-cache
   call-site key so host-context and prepared-package identity changes cannot
   reuse stale cached values under the same plan-template/hole-binding pair,
6. preserves cross-workspace as a typed projected identity input only; no
   executable cross-workspace fallback is invented.

Current non-claim:

This is runtime prepared-identity/cache invalidation for the carriers that
OxCalc can currently project through public OxFml runtime context fields. It is
not full W056 closure. The stable structured-table packet facts added by OxFml
`fml-ds0.8` are now consumed by the OxCalc lowering surface; broader upstream
packet/oracle surfaces, including final name/call precedence, exercised
normalized structured-reference bind packets, and public cross-workspace
availability/degradation semantics, remain blocked by `calc-4vs8.5`. The
registry-view/capability-denial portion of that upstream dependency is now
evidenced by OxFml `fml-ds0.7` and should not be treated as an open blocker.

## 4E. `calc-4vs8.7` Raw Formula-Text Children Prebind Surface

The fifth W056 tranche adds an OxCalc-owned public prebind surface in
`src/oxcalc-core/src/formula.rs` for the first DnaTreeCalc raw formula-text
pressure point.

Current implemented scope:

1. `prebind_treecalc_formula_text(owner_node_id, source_text)` accepts original
   TreeCalc formula text and returns a `TreeFormula` suitable for the existing
   OxFml runtime path,
2. recognizes free-standing `@CHILDREN` and `.*` as explicit host references
   whose base is the formula owner/caller context,
3. rewrites only the OxFml-submitted formula source to neutral formal tokens
   such as `TREE_REF_<owner>_<n>`,
4. preserves the exact source token text and UTF-8 span on
   `TreeCalcChildrenReferenceCollection`,
5. emits `TreeFormula::opaque_oxfml` with
   `TreeFormulaReferenceCarrier::named` carrying
   `TreeCalcReferenceCollection::ChildrenV1`,
6. preserves the existing TreeCalc host-context identities and the existing
   public OxFml sparse reference-values path,
7. returns typed diagnostics for unsupported raw TreeCalc reference families
   and for qualified `base.@CHILDREN` / `base.*` syntax instead of guessing a
   name/path precedence rule.
8. `prebind_treecalc_formula_text_with_resolved_bases(...)` admits qualified
   `base.@CHILDREN` / `base.*` when the caller supplies an exact
   UTF-8-span-keyed resolved-base packet with `base_node_id`, base span,
   selector span, resolution layer, and resolution identity.
9. `treecalc_formula_text_qualified_children_base_queries(...)` exposes the
   exact source/base/selector spans and token text for qualified children
   selectors so host repos can resolve only the base token and feed back a
   typed resolved-base packet without parsing formula text or constructing
   private span keys.

Current non-claim:

This is not a full raw TreeCalc formula parser. The qualified children path
does not resolve raw `base` text; it consumes a typed resolved base supplied by
the caller or a future OxCalc-owned resolver. Structured table formula text,
full explicit path resolution, cross-workspace syntax, and bare name/callable
precedence remain W056/W074 successor scope until they can be resolved through
typed caller-supplied path and namespace surfaces.

Qualified children blocker:

`calc-4vs8.8` closes the narrow blocker by adding the caller-supplied resolved
base contract. The default `prebind_treecalc_formula_text(...)` still receives
only owner node and formula text, so it continues to reject qualified syntax.
The new resolved-base entry point lets an upstream caller or future OxCalc
path resolver provide the exact source span and stable base `TreeNodeId`
without freezing broader TreeCalc name/path precedence or asking a host repo to
mirror OxCalc reference semantics. Full OxCalc-owned explicit path resolution
over a pinned structural snapshot remains successor W056 scope.
`calc-4vs8.10` closes the remaining typed-input gap for host callers by exposing
the query packet they need to build those resolved-base packets from
OxCalc-scanned source coordinates rather than host-side formula parsing.

## 4F. `calc-4vs8.13` Raw Ordered-Selector Prebind Surface

The next W056 tranche extends the same public prebind/query pattern to the
authored DnaTreeCalc ordered selector spellings:

1. `treecalc_formula_text_ordered_selector_queries(...)` scans original formula
   text outside string literals and returns source-preserving query packets for
   `@PRECEDING`, `@FOLLOWING`, `@ANCESTORS`, and recursive descent `**`,
   including qualified forms such as `base.@FOLLOWING`, `Q2.**`, and
   `Accounts.2005.**.Margin`,
2. each query carries selector family, exact source span, optional base span,
   selector span, optional recursive tail span, exact source token text, base
   token text, selector token text, and tail token text,
3. `TreeCalcOrderedSelectorResolution` is the caller-supplied resolved
   collection packet: family, source/base/selector/tail spans, exact source
   token, stable base `TreeNodeId`, ordered member node ids, resolution layer,
   and resolution identity,
4. `prebind_treecalc_formula_text_with_resolved_ordered_selectors(...)` rewrites
   the OxFml-submitted source to neutral `TREE_REF_*` tokens and emits
   `TreeCalcReferenceCollection::OrderedSelectorV1` carriers preserving the
   original source token/span and resolver-supplied member order,
5. unresolved ordered selectors produce typed
   `MissingOrderedSelectorResolution` diagnostics rather than guessing path or
   traversal semantics, and unsupported `@...` selectors remain typed rejects,
6. the runtime sparse reference-values path dispatches ordered selector
   collections to `TreeCalcOrderedSelectorSparseReader`; it no longer treats all
   collection carriers as `ChildrenV1`.

Current non-claim:

This is a source-preserving query/resolved-collection carrier surface. It does
not itself resolve raw base paths, compute traversal membership, impose
traversal bounds, activate the DnaTreeCalc corpus, or define final name/call
precedence. DnaTreeCalc or a future OxCalc resolver must supply the resolved
collection packet through the public query coordinates.

## 4G. `calc-4vs8.15` W093/W074 Registered-External Intake

The current W056 blocker is narrower after the OxFunc W093 and OxFml W074
registered-external reconciliation tranche:

1. OxFunc W093 source mapping and reconciliation beads have landed for OxFml
   invalidation handoff acknowledgement, UDF collision/name-precedence evidence
   intake, registered-external seam alignment, public UDF source mapping, and
   JavaScript custom-function metadata mapping.
2. Descriptor-only `REGISTER.ID`/`CALL` state remains adjacent
   registered-external state, not ordinary bind-visible UDF registration.
3. Ordinary bind-visible UDF registration still flows through the OxFunc
   registry snapshot/change-set lane, while descriptor-only mutation can
   preserve ordinary registry snapshot identity and use targeted reevaluation.
4. OxFunc's W093 reconciliation does not add any TreeCalc-specific function
   branch and does not inspect TreeCalc selectors.
5. OxFml W074 has acknowledged the same registered-external split, so W056 no
   longer treats the source-mapping or descriptor-only/friendly-UDF distinction
   as a missing upstream packet.

Current non-claim:

This intake does not freeze name/call precedence. OxFml W074 still owns the
formula-call registry lookup migration, broad cache-invalidation evidence,
host namespace mutation invalidation, and Excel-oracle-backed precedence rows
for built-in/UDF/defined-name/defined-name-`LAMBDA` collisions. TreeCalc bare
host names and lambda-valued host nodes therefore remain mapped to the closest
Excel defined-name lane until that evidence lands.

## 4H. `calc-4vs8.16` Ordered Selector Traversal Resolver

The next W056 tranche moves ordered-selector membership calculation into an
OxCalc-owned resolver over a pinned `StructuralSnapshot`:

1. `TreeCalcOrderedSelectorTraversalPolicy` carries the explicit traversal
   bound as replayable policy identity
   `treecalc-traversal-bound:v1:max_recursive_descendants=<n>`.
2. `resolve_treecalc_ordered_selector_traversal(...)` computes member order for
   resolved `SiblingSetV1`, `PrecedingV1`, `FollowingV1`, `AncestorsV1`, and
   `RecursiveDescendantsV1` selectors from the structural snapshot.
3. recursive descent uses stable structural preorder and fails with a typed
   `RecursiveTraversalLimitExceeded` error when the explicit bound is exceeded.
4. recursive tail paths such as `**.Margin` are resolved by applying the tail
   segments below the base and then each descendant in traversal order; a tail
   that matches no member returns empty membership with a typed diagnostic
   rather than silently becoming an unrelated reference.
5. `TreeCalcOrderedSelectorQuery::to_resolution_with_structural_traversal(...)`
   builds an `OxCalcStructuralTraversal` resolution packet that can feed the
   existing `OrderedSelectorV1` carrier and sparse reader path.

Current non-claim:

This is traversal membership and bound policy for a resolved base node. It does
not resolve raw base-token text, define cross-workspace availability, freeze
TreeCalc name/call precedence, activate DnaTreeCalc corpus families, or supply
retained OxReplay evidence.

## 4I. `calc-4vs8.17` Explicit Host-Path Base Resolution

The next W056 tranche adds an OxCalc-owned base resolver for explicit
host-reference selector bases:

1. `resolve_treecalc_explicit_host_path_base(...)` resolves only the base token
   that is already part of explicit host-reference syntax such as
   `Branch.@CHILDREN`, `Branch.@FOLLOWING`, or `Root.**.Leaf`.
2. the resolver maps exact projection paths, dotted projection paths, and
   root-descendant dotted paths to stable `TreeNodeId` values and records the
   canonical projection path in the resolution identity.
3. bracketed path segments such as `Branch.[Leaf]` are admitted for structural
   path resolution without asking a host repo to parse TreeCalc text.
4. `TreeCalcQualifiedChildrenBaseQuery` can now build a
   `TreeCalcQualifiedBaseResolutionLayer::OxCalcStructuralPath` packet directly
   from a structural snapshot.
5. `TreeCalcOrderedSelectorQuery` can combine structural base resolution with
   the `calc-4vs8.16` traversal resolver to produce an
   `OxCalcStructuralTraversal` ordered-selector packet.
6. cross-workspace tokens containing `!` remain typed rejects until W056 has a
   versioned workspace availability/degradation model.

Current non-claim:

This is not bare formula-name lookup, function-call lookup, or callable host
node precedence. It applies only to explicit host-reference selector bases and
does not freeze W074 name/call semantics, table syntax, workspace aliases,
DnaTreeCalc corpus activation, or retained replay evidence.

## 4J. `calc-4vs8.18` Cross-Workspace Availability Packet

The next W056 tranche adds the OxCalc-owned cross-workspace
availability/degradation packet:

1. `TreeCalcCrossWorkspaceAvailabilityPacket` carries a stable workspace
   handle, workspace selector token, `availability_version`, status
   (`Available`, `Unavailable`, `Degraded`), optional degradation layer, and
   typed diagnostics.
2. `prepared_identity_component()` projects the availability version, workspace
   handle, status, and degradation layer as a deterministic prepared/cache
   identity contribution.
3. at this tranche, the cross-workspace inventory row still pointed to
   `NeedsCrossWorkspaceProvider`: the packet/version model existed, but
   execution still required a workspace provider and alias model.
4. at this tranche, explicit host-path tokens containing `!` were still typed
   rejects in `resolve_treecalc_explicit_host_path_base(...)`; `calc-4vs8.30`
   supersedes that with the first-position sheet-separator rule.

Current non-claim:

This is not executable cross-workspace reference resolution. It supplies the
packet and identity shape needed to report availability/degradation without
inventing workspace alias semantics, external workspace lookup, or retained
cross-workspace corpus evidence.

## 4J.1. `calc-4vs8.30` Cross-Workspace Provider And Alias Packet

The W056 non-table tranche now admits the OxCalc-owned provider/alias surface
for explicit host-path base tokens:

1. `TreeCalcWorkspaceResolutionRegistry` is the testable host-provider shape:
   it registers the current workspace, loaded external workspace snapshots,
   workspace aliases, and each workspace's `availability_version`.
2. `resolve_treecalc_workspace_host_path_base(...)` returns
   `TreeCalcWorkspaceHostPathBaseResolution`, carrying the original base token,
   optional workspace selector token, local path token, stable workspace handle,
   stable `workspace#node` handle, canonical projection path, workspace
   resolution layer, local resolution layer, availability packet, and
   deterministic resolution identity.
3. `[alias]Path`, quoted direct selectors such as
   `['C:\Work\projections.dnatree']Path`, `[alias][Escaped Segment].Path`, and
   `[alias]` workspace-root forms are resolved through the registry without
   hosts parsing OxFml private strings. Unregistered `[Word]...` forms fall
   back to escaped current-workspace path binding unless the selector looks
   like a direct workspace path; this preserves the DnaTreeCalc alias-first
   bracket-position rule.
4. unloaded or unregistered workspaces produce typed
   `WorkspaceUnavailable` diagnostics plus an availability-version prepared
   identity contribution. No stale-value fallback is introduced.
5. `!` is no longer treated as the cross-workspace marker. It is admitted only
   as the sheet-position separator alias after the first path segment
   (`Sheet1!Foo.Bar` == `Sheet1.Foo.Bar`); leading or mid-path `!` remains a
   typed syntax error.
6. the cross-workspace inventory row points past the old provider-missing
   blocker; `calc-8tox` consumes the remaining workspace-qualified carrier
   gap.

Current non-claim:

This does not activate the DnaTreeCalc `references/cross-workspace` corpus,
does not define bare host-name/call precedence, and does not add dynamic
`INDIRECT` cross-workspace rebind. The workspace-qualified carrier/dependency
path is consumed by `calc-8tox`; dynamic/callable/evidence closure remains under
`calc-4vs8.31`, `calc-4vs8.32`, and `calc-4vs8.33`.

## 4J.2. `calc-8tox` Workspace-Qualified Cross-Workspace Carrier

The W056 non-table tranche now extends the provider/alias packet into a typed
runtime carrier and dependency surface:

1. `TreeReference::CrossWorkspaceResolved` preserves the stable workspace
   handle, local target node id, workspace-qualified target handle,
   availability version, carrier id, and resolution detail.
2. `TreeCalcWorkspaceHostPathBaseResolution::to_workspace_qualified_reference`
   creates that carrier directly from the public provider/alias packet, so
   host repos do not parse OxCalc private strings or reconstruct carrier
   identity.
3. `DependencyDescriptor.workspace_target` carries a typed
   `WorkspaceQualifiedTarget`; `target_node_id` stays `None`, preventing
   external targets from being mistaken for local graph nodes.
4. `DependencyGraph.workspace_reverse_edges` and
   `W056ReferenceDependencySurface.workspace_target_reverse_edges` expose the
   workspace-qualified reverse-edge fact separately from local reverse edges.
5. prepared identity for those descriptors uses
   `CrossWorkspaceAvailabilityVersion` with no caller-context dependency. The
   actual per-reference workspace handle, target handle, and availability
   version are projected into host namespace identity from the carrier rather
   than requiring callers to mirror a single environment-level availability
   string.
6. runtime formal input binding treats workspace-qualified targets as opaque
   `ReferenceLike` values keyed by the stable workspace-qualified target
   handle; it does not read `working_values` by the external numeric
   `TreeNodeId`.
7. local references, local collection carriers, and local reverse edges keep
   their existing `TreeNodeId` behavior.

Current non-claim:

This is still not DnaTreeCalc corpus activation, retained replay evidence,
dynamic `INDIRECT` cross-workspace rebind, or W074 name/call precedence. Those
remain under the open W056 successor beads.

## 4J.3. `calc-4vs8.31` Reference Literal Arrays And Dynamic Carrier Intake

The W056 non-table tranche now adds the first explicit reference-literal array
carrier while preserving existing dynamic-reference behavior:

1. `TreeCalcReferenceCollection::ReferenceLiteralArrayV1` is an OxCalc-owned
   collection carrier for authored reference literal arrays. It carries a
   stable carrier id, host-reference handle, owner node, exact source token,
   optional source span, opaque selector payload, member node ids, membership
   version, and order version.
   It is constructed from typed reference-literal array elements rather than
   from a raw member list; scalar elements return a typed
   `MixedScalarReferenceArray` error before lowering.
2. `TreeReferenceCollectionFamily::ReferenceLiteralArrayV1` records dependency
   membership and member-value facts without pretending the array was produced
   by tree traversal. Membership identity is set-based; order identity
   preserves authored order and duplicates.
3. `TreeCalcReferenceLiteralArraySparseReader` exposes the W051 sparse-reader
   surface for reference-literal arrays: declared extent, defined cardinality,
   defined iterator, `read_at`, `contains`, and stable reader identity.
4. Runtime binding passes the carrier through the generic OxFml host-reference
   path and sparse `ReferenceLike` values. OxFunc still sees only an opaque
   structured reference target; it does not inspect TreeCalc selectors.
5. Dynamic `DynamicPotential` and `DynamicResolved` carriers remain the
   admitted OxCalc dynamic-dependency path. Focused dynamic tests continue to
   prove potential versus resolved rebind facts, dynamic runtime effects, and
   resolved shape updates.
6. Mixed scalar/reference array literals are an executable typed exclusion
   under `TreeReferenceInventoryVariant::MixedReferenceArray` until there is an
   explicit generic contract for scalar/reference mixing. They are rejected at
   the typed element-packet boundary and must not be silently coerced into a
   reference-only carrier.

Current non-claim:

This is not DnaTreeCalc corpus activation for reference literals, dynamic
`INDIRECT` end-to-end product evidence, cross-workspace dynamic rebind, or W074
name/call precedence. Those remain under `calc-4vs8.32`, `calc-4vs8.33`, and
the narrowed `calc-4vs8.5` blocker.

## 4K. `calc-4vs8.19` OxFml Host Namespace Invalidation Intake

OxFml W074 `fml-ds0.6.1` at commit `4a050f9` narrows one upstream W056
prepared-identity blocker:

1. `host_namespace_version` now has OxFml runtime/replay evidence as a prepared
   identity input even when `host_reference_bind_results` is empty.
2. this supports the DnaOneCalc-style no-host-reference case: ordinary
   single-formula execution still does not require host references, while a
   host that supplies a namespace version can invalidate prepared identity
   conservatively.
3. this is a generic host-context invalidation fact, not a TreeCalc-specific
   semantic branch.

Current non-claim:

This does not freeze bare host-name resolution, callable TreeCalc host nodes,
workbook/sheet/UDF/defined-name precedence, or broader formula-call registry
lookup/cache invalidation. Those remain under OxFml W074 and the open
`calc-4vs8.5` blocker.

## 4L. `calc-4vs8.20` DnaTreeCalc Structural Traversal Activation Intake

DnaTreeCalc W004 `dtc-z0i.9` at commit `fe678cf` narrows the receiving-side
corpus blocker for the explicit structural path/traversal slice:

1. DnaTreeCalc expanded the active ordered raw corpus from four to seven cases.
2. the live bridge now uses OxCalc structural path/traversal resolvers when
   equivalent to host-visible member projection.
3. traversal-bound failures are pinned as typed OxCalc policy failures.
4. W005 remains open; no shell/skin/save-reopen/click-through closure was
   claimed.

Current non-claim:

This does not close W004/W005 non-table corpus activation, bare host-name/
dotted formula binding, dynamic/cross-workspace references,
node-as-function/lambda-valued nodes, or retained OxReplay evidence for
non-table references.

## 5. Closure Gate

W056 closes only for a declared full-reference/table-lowering scope when:

1. admitted TreeCalc reference variants have implemented carriers or explicit
   typed exclusions,
2. dependency and invalidation facts are replay-visible and tested for each
   admitted variant,
3. dynamic rebind and host namespace version changes produce deterministic
   prepared-identity invalidation,
4. caller-context-sensitive references are exercised with stable
   caller-context identity,
5. structured table row/column/header/totals dependencies are lowered from the
   generic OxFml table packet without OxCalc parsing formula language,
6. node-associated TreeCalc table support has passed through the
   `calc-4vs8.21` through `calc-4vs8.26` spine: table-node snapshot projection,
   TreeCalc table-path structured-reference prebind, table reference readers,
   per-row column-formula runtime, update/invalidation scenarios, and retained
   evidence closure,
7. OxFml/OxFunc integration remains through public generic host-context and
   reference/value surfaces,
8. known exclusions and any new cross-repo handoffs are explicit.

## 6. Status

Product status: table-topic promoted for the declared node-associated TreeCalc
table scope; broader W056 remains in progress for non-table references,
W004/W005 corpus activation, and W074-dependent name/call behavior. W051 is
closed for the first OxCalc `ChildrenV1` carrier pattern; W056 now has a typed Rust
implementation-input inventory for the broader reference family, a first
structured table-context dependency-lowering surface for the current generic
OxFml table packet, and a typed dependency/reverse-edge/invalidation/rebind
projection over current OxCalc graph facts. Runtime preparation now consumes
the typed W056 identity needs through public OxFml `RuntimeHostFormulaContext`
fields where available, and the local edge-value cache includes the resulting
prepared formula identity in its call-site key. OxCalc now also exposes a
public raw TreeCalc formula-text prebind for free-standing `@CHILDREN` and
`.*`, plus a caller-supplied resolved-base prebind contract for
`base.@CHILDREN` and `base.*` with a public qualified-base query packet,
producing a neutral OxFml source plus a source-preserving `ChildrenV1` carrier
for the existing OxFml/OxFunc path. DnaTreeCalc commit `6611684` has activated
the first receiving-side raw corpus slice for free-standing and qualified
children forms through that public query/resolved-base packet rather than
local TreeCalc formula parsing. OxFml commit `9269421` has recorded the
W074-CALC005-014 defined-name/table-name collision row: in the observed Excel
COM 16.0 collision state, bare `=Table1` resolves to the workbook defined name
while `Table1[Amount]` structured syntax is rejected at formula authoring.
`calc-4vs8.12` adds OxCalc-owned ordered selector collection carriers for
resolved sibling, preceding, following, ancestor, and recursive-descendant
selector packets, including membership/order dependency facts and sparse
reference-reader support. `calc-4vs8.13` exposes public raw formula-text query
packets for `@PRECEDING`, `@FOLLOWING`, `@ANCESTORS`, and recursive `**`
spelling, plus caller-supplied resolved-collection prebind into the existing
`OrderedSelectorV1` carrier and runtime sparse reader path. DnaTreeCalc commit
`66355f8` activates the first receiving-side ordered-selector corpus slice for
`@PRECEDING`, `@FOLLOWING`, `@ANCESTORS`, and `Base.**.Margin` through those
public query/resolved-collection packets. `calc-4vs8.14` keeps the generic
OxFml host-reference bind packet shape explicit by reporting ordered-selector
family hints separately from `ChildrenV1`. `calc-4vs8.15` records the
cross-repo W093/W074 registered-external reconciliation intake: OxFunc source
mapping, registered-external split, and JavaScript custom-function metadata
mapping are no longer W056 upstream packet blockers, while W074 name/call and
cache-invalidation evidence remains open. `calc-4vs8.16` adds an OxCalc-owned
ordered-selector traversal resolver and replayable traversal-bound policy for
resolved base nodes, so sibling/preceding/following/ancestor/recursive
membership no longer has to be supplied by a host repo once the base node is
known. `calc-4vs8.17` adds explicit structural path base resolution for
qualified selector tokens, so `base.@CHILDREN`, `base.@FOLLOWING`, and
`base.**.Tail`-style packets can be resolved by OxCalc without host-side path
semantics when the token is an admitted explicit host-reference selector base.
`calc-4vs8.18` adds the typed cross-workspace availability/degradation packet
and prepared-identity component, narrowing the previous missing-packet blocker
to the remaining workspace provider and alias semantics. `calc-4vs8.30` adds
that provider/alias surface, including first-position `!` sheet-separator
semantics and stable unavailable-workspace identity. `calc-8tox` extends it
into a workspace-qualified carrier/dependency path that preserves the external
workspace handle, target handle, target node id, and availability version
without collapsing external targets into local `TreeNodeId` reverse edges.
OxFml commit `4a050f9` adds W074 evidence that `host_namespace_version`
participates in prepared identity even without explicit host-reference bind
results, narrowing the host namespace mutation invalidation gap without
freezing name/call precedence.
DnaTreeCalc commit `fe678cf` activates the scoped explicit structural
path/traversal raw ordered corpus slice through the public OxCalc resolver path,
narrowing receiving-side corpus activation without closing W004/W005.
The table area now has a closed table-topic completion spine in the bead graph:
`calc-4vs8.21` through `calc-4vs8.29` and `calc-4vs8.34` through
`calc-4vs8.38`, with matching DnaTreeCalc, OxFml, OxFunc, OxXlPlay, and
OxReplay beads for persistence/corpus activation, generic structured-reference
packet breadth, opaque function admission, Excel observation, retained
comparison, lifecycle/version policy, and final promotion. This proves the
declared node-associated TreeCalc table topic without adding TreeCalc semantics
to OxFml/OxFunc. This is not a full-reference W056 closure claim.
A new comprehensive table completion spine, `calc-4vs8.44` through
`calc-4vs8.56`, is open to turn that scoped promotion into the intended
long-term table feature. It decomposes the remaining table work into
architecture/ownership, virtual anchor identity, generic OxFml packet contract,
OxCalc resolver and namespace versioning, full `ReferenceLike` readers,
row-context prepared identity, complete dependency/invalidation scenarios,
DnaTreeCalc activation, OxXlPlay oracle construction, OxReplay retained
evidence, UDF/VBA/XLL impact, cross-repo rollout coordination, and dynamic
table reference rebind/`INDIRECT` semantics.

Evidence: W051 focused tests cover the first carrier's local membership/member
value dependency descriptors, reference-preserving formal input binding,
OxFml sparse reference-values binding, OxFunc aggregate consumption, and
membership/order invalidation facts. `calc-4vs8.1` adds focused Rust tests for
the W056 inventory, concrete `TreeReference` mapping, and `ChildrenV1`
handle/dependency/invalidation correlation facts. `calc-4vs8.2` adds focused
Rust tests for available table fact lowering, typed blocker emission for missing
row/region packet facts, graph retention of context-only descriptors, and
omitted-table-name enclosing context failure. `calc-4vs8.3` adds focused Rust
tests for target reverse-edge facts, context reverse-edge facts,
namespace/caller-context prepared-identity inputs, dynamic potential versus
resolved dynamic rebind facts, and cross-workspace typed blocker preservation.
`calc-4vs8.6` adds focused Rust tests for host namespace and caller-context
prepared-key changes, capability-profile prepared-key changes, table-context
and cross-workspace public host-context projection, and prepared-formula-key
participation in the local edge-value cache key. `calc-4vs8.7` adds focused
Rust tests proving `=SUM(@CHILDREN)` and `=SUM(.*)` prebind to neutral
`TREE_REF_*` OxFml source, preserve source token text/spans, produce
`ChildrenV1` carriers, reject unsupported raw TreeCalc reference families, and
execute end-to-end through the existing OxCalc/OxFml/OxFunc reference path.
`calc-4vs8.8` adds focused Rust tests proving `=SUM(base.@CHILDREN)` and
`=SUM(base.*)` can prebind through exact source-span-keyed resolved-base
packets, preserve qualified token text/spans, bind to the supplied base
`TreeNodeId`, reject qualified syntax without a matching resolved base, and
execute end-to-end through the same reference-preserving path.
`calc-4vs8.10` adds focused Rust tests proving `base.@CHILDREN` and `base.*`
query packets expose source span, base span, selector span, source token, base
token, selector token, convert back into the existing resolved-base packet, and
ignore string literals.
DnaTreeCalc commit `6611684` adds receiving-side evidence that its live OxCalc
bridge can run the active raw children corpus slice for `@CHILDREN`, `.*`,
`base.@CHILDREN`, and `base.*` through OxCalc's public query packet without
local formula parsing, including ordered dependency projection.
`calc-4vs8.4` consumes OxFml `fml-ds0.8` stable table packet facts and adds
focused Rust coverage proving row membership, row order, exact header region,
and exact totals region lower as context dependency descriptors when supplied,
while typed blockers remain for legacy packets where those optional facts are
absent.
`calc-4vs8.9` consumes OxFml `fml-ds0.9` structured-reference bind packets and
adds focused Rust coverage proving explicit table, omitted table-name,
`#This Row`, selected section/region, selected column, handle-correlation, and
diagnostic-bearing unresolved records map into OxCalc table lowering without
formula-text parsing. Omitted table-name packets preserve OxFml's bound
effective table id as the lowering target and separately validate the enclosing
table context, surfacing a typed blocker if those packet facts disagree.
OxFml commit `9269421` narrows W074 table-name collision evidence but does not
freeze TreeCalc bare name/callable precedence.
`calc-4vs8.12` adds focused Rust tests proving ordered selector collections
lower to `TreeReferenceCollectionMembership` plus member-value descriptors,
preserve host-reference handles and ordered member ids, enter W056 inventory as
admitted implementation inputs, project through the local TreeCalc carrier
projection path, and expose a sparse reader that uses resolver-supplied member
order without parsing TreeCalc text.
`calc-4vs8.13` adds focused Rust tests proving ordered-selector query packets
preserve source/base/selector/tail spans and exact token text for unqualified,
qualified, and recursive-tail forms; unresolved ordered selectors emit typed
diagnostics; prebinding with caller-supplied resolved collections produces
neutral OxFml source and `OrderedSelectorV1` carriers; string literals are
ignored; and `=SUM(@PRECEDING)` executes through OxFml/OxFunc using the generic
sparse reference-values path rather than eager materialization.
`calc-4vs8.14` adds focused Rust coverage that `RuntimeHostReferenceBindResult`
shape hints distinguish `ordered_collection:children_v1` from
`ordered_collection:treecalc_ordered_selector_v1:<family>`.
`calc-4vs8.15` is a coordination intake: OxFunc W093 source mapping,
registered-external seam alignment, and JavaScript custom-function metadata
mapping have landed without transferring TreeCalc selector semantics into
OxFunc, and OxFml W074 has acknowledged the descriptor-only registered-external
split. This removes stale W093 source/reconciliation packet blockers from the
W056 status surface only.
`calc-4vs8.16` adds focused Rust coverage proving resolved structural
traversal for sibling-set, preceding, following, ancestor, and recursive
selectors; query-to-resolution projection with an `OxCalcStructuralTraversal`
resolution layer and traversal policy identity; recursive tail filtering; typed
recursive traversal-bound failure; and typed empty-tail-match diagnostics.
`calc-4vs8.17` adds focused Rust coverage proving exact projection path,
dotted projection path, root-descendant dotted/bracketed path, typed
cross-workspace rejection, qualified children structural-base resolution, and
ordered selector structural-base-plus-traversal resolution.
`calc-4vs8.18` adds focused Rust coverage proving unavailable/degraded/
available cross-workspace packet statuses, typed diagnostics, deterministic
prepared-identity projection, and the W056 inventory transition from missing
model to missing provider.
`calc-4vs8.19` records the OxFml evidence passed by `cargo fmt --all --
--check`, focused runtime/replay host-namespace identity tests, full
`cargo test -p oxfml_core`, and `git diff --check`.
`calc-4vs8.20` records DnaTreeCalc evidence: corpus validation, `cargo fmt
--check`, `cargo build --workspace`, `cargo test --workspace`, `cargo clippy
--workspace -- -D warnings`, and `git diff --check`.
`calc-4vs8.21` adds focused Rust coverage for node-associated table snapshot
projection into generic OxFml `TableDescriptor` packets, deterministic virtual
A1 range assignment, table context identity, OxCalc-only invalidation identity,
and typed projection exclusions for shapes the generic packet cannot yet carry.
`calc-4vs8.22` adds focused Rust coverage for TreeCalc table-path
structured-reference prebind packets that preserve source spans/tokens,
path/tail spans, handles, selector payloads, caller-context dependency,
diagnostics, replay identity, omitted current-row forms, bracket escaping, and
unknown table/column diagnostics without adding TreeCalc parsing to OxFml.
`calc-4vs8.23` adds focused Rust coverage for table sparse readers over data
columns, contiguous multi-column data ranges, header/totals/#All sections,
current-row scalar runtime binding, sparse traversal counters, and
reference-preserving aggregate execution for `SUM`, `COUNT`, `COUNTA`, and
`COUNTBLANK` through generic OxFml/OxFunc sparse `ReferenceLike` bindings.
`calc-4vs8.24` adds focused Rust coverage for row-context table formula
runtime: one formula text evaluated per data row through generic OxFml table
context, row-specific caller context, sparse/current-row bindings, prepared
identity variation by row, stable dispatch skeleton reuse, totals formula
execution, and typed rejection of current-row references outside data rows.
`calc-4vs8.25` adds focused Rust coverage for the full declared table update
scenario set: body value/formula edits, row insert/delete/reorder, column
insert/delete/reorder/rename, header edits, totals toggle/formula edit, table
rename/move/delete, save/reopen identity preservation, structural rebind,
typed post-update diagnostics, and unaffected reader identity stability.
`calc-4vs8.26` audit intake records cross-repo progress rather than closure:
DnaTreeCalc activated table structured-reference corpus slices through
LiveOxCalc/OxCalc (`e59c6f1`, `a5f7b65`, `b59b2fb`, `8eba3cb`), OxXlPlay
produced retained Excel table observation/update artifacts (`6c0f53e`,
`8176223`, `c3a4c88`), and OxReplay recorded retained-artifact intake,
comparison-policy, and refreshed update-oracle evidence (`a195815`, `b341f8b`,
`e6de7a4`, `c387bc9`, `9e4c503`, `16791fb`, `7e21a6f`). The audit filed and
then consumed `calc-4vs8.27`, `calc-4vs8.28`, `calc-4vs8.29`, and
`calc-4vs8.37`; final table closure now depends on the audit verdict itself.
The remaining shared `comparison_value` helper replacement is tracked as
`BLK-REPLAY-003` outside TreeCalc table semantics.
`calc-4vs8.31` adds focused Rust coverage for reference literal arrays and
dynamic carrier intake: `ReferenceLiteralArrayV1` lowers to membership and
member-value descriptors while preserving authored order and duplicates,
projects through a sparse reader and generic OxFml host-reference packet, and
executes `SUM` through opaque `ReferenceLike` sparse values. Existing dynamic
tests continue to prove dynamic potential/resolved rebind facts and runtime
effects. Mixed scalar/reference arrays are rejected at the typed element-packet
constructor and remain a typed exclusion rather than a silent reference-only
coercion.
OxFml `fml-ds0.12` at commit `466213b` closes the generic W056
structured-reference packet coverage that table nodes depend on: explicit and
omitted table refs, current-row refs, `#Headers`, `#Data`, `#Totals`, `#All`,
section+column and multi-column forms, escaped column names, typed diagnostics,
source span/token preservation, effective table identity, selected
columns/sections/regions, sparse `ReferenceLike` runtime use, and replay
projection. This removes W056 table packet coverage as an upstream blocker;
broader W036 table formula semantics and non-table W074 name/call precedence
remain separate.

Still open: W074 final non-table name/call precedence evidence, W074
formula-call registry lookup and cache-invalidation migration, bare host-name
and callable host-node precedence,
exercised OxFml host-reference packets beyond the admitted
children/table/ordered-selector/cross-workspace/reference-literal-array
slices, DnaTreeCalc activation for the remaining W004/W005 non-table reference
suite, broader end-to-end scenarios, and retained evidence intake. The
provider/alias/first-position `!` packet shape, workspace-qualified carrier
path, reference-literal array carrier path, and generic structured-reference
table packet coverage are no longer missing, but blocker `calc-4vs8.5` remains
open for the remaining full-W056 non-table closure scope.

Formal status: no proof claim.
