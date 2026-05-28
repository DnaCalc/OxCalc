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
   TreeCalc table-path structured-reference host-reference bind, table reference readers,
   per-row column-formula runtime, update/invalidation scenarios, retained
   TreeCalc/OxReplay evidence, and Excel update-oracle intake for the declared
   table slice.
5. `calc-4vs8.34` through `calc-4vs8.38` — second-pass table product-promotion
   spine: full-scope closure map, lifecycle/callback contract freeze,
   structured-table `ReferenceLike` function breadth, matched TreeCalc/Excel
   replay/value-wire intake, and final table-topic promotion audit.
6. `calc-4vs8.39` through `calc-4vs8.43` — closed third-pass full-intended
   table support spine: empty data-body packet/reader support, executable
   function breadth evidence, DnaTreeCalc lifecycle bridge acceptance, table
   namespace/anchor/workspace collision semantics, and final table support
   audit without relying on the earlier typed projection exclusions.
7. `calc-4vs8.44` through `calc-4vs8.56` — fourth-pass comprehensive
   node-table design and rollout spine: whole-system ownership map, virtual
   Excel-anchor identity, generic OxFml packet contract, OxCalc resolver/
   namespace versioning, full `ReferenceLike` reader surface, row-context
   prepared identity, complete dependency/invalidation matrix, DnaTreeCalc
   activation, OxXlPlay oracle construction, OxReplay retained evidence,
   UDF/VBA/XLL impact, cross-repo handoff coordination, and dynamic table
   reference rebind/`INDIRECT` semantics.
8. `calc-4vs8.57` through `calc-4vs8.63` — completed fifth-pass node-table
   hardening
   spine: current-state architecture revalidation, abstraction consolidation,
   lifecycle execution matrix, ReferenceLike/function/UDF integration closure,
   oracle/replay/value-wire convergence, cross-repo rollout reconciliation,
   and final completion audit before the table topic is carried into parent
   W056 closure.
9. `calc-4vs8.30` through `calc-4vs8.33`, plus `calc-8tox` — remaining
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
3. formula-call registry-view admission, capability-denied runtime
   classification, host-name runtime/replay binding, and the current
   W051/W056 name/call freeze have landed in OxFml W074. OxCalc consumes
   `HANDOFF_CALC_005_W074_NAME_CALL_FREEZE.md` from OxFml commit `4a55709`:
   TreeCalc host value names map to the Excel defined-name value lane,
   lambda-valued host nodes map to the defined-name-`LAMBDA` lane, and
   built-ins keep the ordinary call-callee frontier unless a future
   versioned extension is separately evidenced,
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
2. `calc-4vs8.22` — add public TreeCalc structured-reference host-reference bind for
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
   through the direct OxCalcTreeContext path.
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
direct OxCalcTreeContext paths, including `#All`, bracket-escaped table and column
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
   covering the active direct OxCalcTreeContext table corpus, row formulas, totals
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
4. DnaTreeCalc keeps table corpus/product evidence on the direct
   OxCalcTreeContext path, including persistence, structural updates, and
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
   structured-reference source-preservation edge cases while consuming the
   current W074 host-name mapping without adding local precedence rules.
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
4. DnaTreeCalc supplies product table lifecycle/corpus events through the direct
   OxCalcTreeContext path and emits retained artifacts; it does not parse
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
   collisions, and W074-mapped name/call boundaries.
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
   row/column insert/delete/reorder, renames, totals toggles, table
   rename/move/delete/resize, node rename/move/delete, save/reopen, workspace
   open/close, alias mutation, registry snapshot mutation, and structural
   rebind.
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

`calc-4vs8.43` depends on this fourth-pass spine as well as the third-pass
residual beads. It may close the full intended table topic only after these
beads are closed with evidence or explicitly converted into user-accepted typed
exclusions.

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
   the outcome is a typed exclusion or a future versioned evidence lane, not
   local inference.
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
3. `calc-4vs8.47` owns resolver and namespace facts, consuming the current
   W074 host-name mapping while leaving any future built-in-call override as a
   separate versioned extension.
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

OxCalc-specific node-table host-reference bind facts:

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
2. OxCalc focused tests assert that TreeCalc table binds and
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
   version, structure-context version, resolution-rule version, W074-mapped
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
10. W074-mapped adjacency diagnostics for host names, functions, defined names,
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
W074-mapped namespace adjacency, bracket-escaped `!` tokens, caller-row
invalidation under coarse caller ids, and handle changes after alias/namespace
mutation.

Still open:

1. `calc-4vs8.48` is closed for the full table `ReferenceLike` reader
   surface.
2. `calc-4vs8.49` is closed for row-context formula/prepared identity behavior.
3. `calc-4vs8.50` is closed for dependency and invalidation matrices for all
   table lifecycle updates.
4. Current bare host-name/callable precedence is consumed from the OxFml W074
   handoff; this resolver records adjacency diagnostics and does not add a
   local precedence mirror.

## 4B.6A. `calc-4vs8.42` Table Namespace, Anchor, And Workspace Semantics

Product status:

OxCalc now closes the table namespace/anchor/workspace edge-case intake for
the current W056 table scope. A node-associated TreeCalc table is exposed to
OxFml as an Excel-shaped virtual table anchored in a workbook/sheet/range,
while OxCalc keeps the table catalog, path/canonicalization, namespace
versions, workspace availability, caller-table context, diagnostics, and
prepared-identity facts.

Implemented/evidenced scope:

1. table catalog resolution covers same-workspace table names, canonical and
   display path tokens, bracket-escaped table names, first-position `!`
   current-workspace root lookup, workspace aliases, direct workspace handles,
   same-node table lookup, omitted caller-table lookup, stable table ids,
   unavailable workspaces, deleted tables, and ambiguous selectors;
2. resolver outputs preserve stable table/reference handles, opaque selector
   facts, resolution layer, shape hint, effective table id/node id, virtual
   anchor identity, caller-context dependency/id, host namespace version, table
   namespace version, structure-context version, resolution-rule version,
   workspace availability version, diagnostics, and source span/token facts;
3. virtual-anchor identity separates stable table identity from workbook/sheet
   anchor movement, table path/namespace rename, row membership/order changes,
   column identity changes, workspace alias changes, save/reopen preservation,
   and explicit table movement;
4. the table path lane remains distinct from bare host-name and callable lanes:
   adjacency diagnostics now use W074-mapped codes/messages, explicit
   structured references stay on the table packet path, built-ins keep the
   call-callee frontier, and no OxCalc precedence mirror is introduced;
5. OxXlPlay retained observations provide black-box Excel ListObject evidence
   for table construction, updates, delete/save-reopen typed limits,
   empty-body and row-boundary residuals, and name/anchor collision
   availability; OxReplay and DnaTreeCalc retained evidence remain separate
   closure gates where still open.

Evidence:

1. `calc-4vs8.47` closed the resolver/namespace implementation with focused
   `table_catalog_resolver` tests.
2. `calc-4vs8.45` closed virtual Excel-anchor identity and save/reopen
   stability expectations.
3. `calc-4vs8.50` closed dependency/invalidation coverage for table rename,
   move, delete, node rename/move/delete, workspace open/close/alias mutation,
   and structural rebind.
4. `calc-4vs8.52` closed OxXlPlay retained Excel observation intake for table
   namespace/anchor residuals and typed COM limits.
5. `calc-4vs8.32` closed W074 intake for the current W051/W056 host-name
   mapping, so `.42` no longer treats name/call as pending W074 work.

Still open:

This closes the OxCalc namespace/anchor/workspace semantics intake, not the
final full-table audit by itself. The DnaTreeCalc retained
lifecycle/dynamic/cross-workspace artifacts are consumed through
`calc-4vs8.51`, retained comparison is consumed through `calc-4vs8.53`, and the
remaining final-audit risks are decided in `calc-4vs8.43`: cross-producer
namespace/anchor/workspace pairing, older OxXlPlay `execution_outcome.class_id`
gaps, direct Excel dependency/invalidation internals, and `BLK-REPLAY-003` for
the comparison-value wire helper.

## 4B.7. `calc-4vs8.48` Full Table `ReferenceLike` Reader Surface

Product status:

OxCalc now has a complete sparse/reference-reader surface for the declared
W056 node-associated table selections. `TreeCalcTableSparseReader` can expose a
TreeCalc node table as an opaque generic `ReferenceLike` plus sparse
`RuntimeSparseReferenceValuesBinding`, without adding table-specific
`EvalValue` variants and without giving OxFml or OxFunc TreeCalc selectors.

Implemented reader scope:

1. whole table data-body references,
2. selected data columns,
3. contiguous multi-column ranges,
4. full data-body/all-column references,
5. `#Headers`, `#Data`, `#Totals`, and `#All`,
6. current-row references with caller data-row context,
7. omitted-table current-row references through generic enclosing-table and
   caller-region packets,
8. empty data-body tables with zero-row references,
9. single-row data-body tables,
10. sparse blank cells,
11. defined empty strings,
12. typed worksheet error cells,
13. row and column order preservation,
14. stable reader identity split into reader id, source identity, and snapshot
    identity.

Reader contract:

The reader implements `declared_extent`, `defined_cardinality`,
`defined_iter`, `read_at(coord) -> Defined(EvalValue) | Blank`, `contains`,
and `reader_identity`. `contains` reports declared extent membership; blanks
inside the declared extent remain `read_at(...)=Blank` and are not emitted by
`defined_iter`.

Runtime/function carriage:

1. `SUM`, `COUNT`, `COUNTA`, and `COUNTBLANK` execute through OxFml/OxFunc
   sparse `ReferenceLike` bindings for the first aggregate evidence lane.
2. The wider range-taking groups remain admitted as generic reference-reader
   lanes in `TREECALC_STRUCTURED_TABLE_FUNCTION_ADMISSION_INVENTORY` with
   OxFunc counterpart beads. They are not allowed to inspect TreeCalc selectors
   or use eager materialization as closure evidence.
3. Context-sensitive groups remain typed host-context lanes until the required
   generic context exists: dynamic-array/spill policy, row visibility/filter
   state, metadata disclosure policy, volatile/dynamic rebind, implicit
   intersection caller context, or native invocation policy.

Typed exclusions and successor lanes:

1. unsupported non-contiguous column selections produce
   `TreeCalcTableSparseReaderError::NonContiguousColumnSelection`;
   contiguous multi-column ranges are supported,
2. missing selected columns, absent header/totals regions, caller-table
   mismatch, missing caller table region, non-data caller region, missing row
   offset, out-of-range caller row, invalid ranges, empty selections, and range
   overflow are typed reader errors,
3. broad OxFunc implementation for the admitted-pending function groups remains
   owned by the OxFunc counterpart beads; this bead closes the OxCalc reader
   and carrier side only.

Evidence:

Focused Rust tests cover data columns without dense blanks, whole data-body
references, all-column `#All`, headers, totals, selected columns,
contiguous multi-column ranges, current-row scalar formulas, omitted caller
context, empty data-body zero-row references, single-row references, sparse
blanks, defined empty strings, typed error cells, `contains`/`read_at`
behavior, row/column order preservation, row-order snapshot identity changes,
typed missing-column errors, typed non-contiguous-column exclusions, and the
first aggregate group through OxFml/OxFunc sparse bindings.

Still open:

1. `calc-4vs8.49` is closed for table formula row-context and prepared
   identity.
2. `calc-4vs8.50` is closed for the complete table dependency/invalidation
   matrix.
3. OxFunc counterpart beads must close the admitted-pending wider function
   groups before those functions can be claimed as fully implemented product
   behavior over table references.

## 4B.8. `calc-4vs8.49` Table Formula Row Context And Prepared Identity

Product status:

OxCalc now has an executable row-context formula runtime and replay-visible
prepared-identity fact surface for node-associated table formulas. One authored
formula text is evaluated per data row by first asking OxFml to parse and bind
the structured references against the generic virtual table catalog, enclosing
table ref, and row-specific `caller_table_region`; OxCalc then uses those
public `StructuredReferenceBindRecord` packets to supply sparse
`ReferenceLike` bindings and scalar current-row bindings. The row-context
runtime path uses OxFml structured-reference bind records directly and does not
teach OxFml TreeCalc table semantics.

Implemented runtime scope:

1. table column formulas reuse one authored formula text across all data rows,
2. each data-row execution carries a row-specific caller-context identity,
3. omitted/current-row references resolve through the generic enclosing table
   and caller data-row packet,
4. totals-row formulas execute at the totals virtual locus,
5. current-row references in totals context are typed rejects,
6. the dispatch skeleton remains stable for the same formula text while
   prepared formula keys vary by caller/table/context identity,
7. the runtime result exposes `TreeCalcTableFormulaPreparedIdentityFacts` for
   replay and cache-invalidation correlation.

Prepared/cache identity facts:

`TreeCalcTableFormulaPreparedIdentityFacts` records the OxCalc/OxFml prepared
identity inputs that matter for table formulas: dialect id, capability profile
id, resolution rule version, host namespace version, table namespace version,
structure context version, table context identity, caller context id,
host-supplied registry snapshot identity, runtime function-registry snapshot
identity, capability overlay identity, prepared formula key, dispatch skeleton
key, and plan-template key.

Update evidence covered by this bead:

Focused runtime tests prove that prepared identity changes for host namespace,
structure context, resolution rule, capability profile, actual OxFunc registry
snapshot mutation through UDF registration, capability overlay, table namespace
rename, row reorder, row insert, row delete, and caller row movement. The same
formula text continues to reuse the dispatch skeleton where only the
caller/table context changes. The existing totals-row tests prove totals-region
execution and reject `#This Row` outside the data region.

Explicit non-claim:

LET/LAMBDA lexical variables, callable locals, captures, and returned lambdas
remain OxFml-internal behavior. This OxCalc bead does not add host-side LET or
LAMBDA semantics and does not require a host namespace for DnaOneCalc ordinary
single-formula execution. The DnaOneCalc/OxFml no-host LET/LAMBDA guardrail
remains a cross-repo W074/DnaOneCalc evidence lane.

Consumed downstream:

1. `calc-4vs8.50` is closed for the complete table dependency/invalidation
   matrix using the prepared-identity fact surface.
2. `calc-4vs8.51` closed the DnaTreeCalc table corpus/bridge activation through
   the real bridge.
3. `calc-4vs8.52` and `calc-4vs8.53` closed retained Excel/replay table
   evidence intake for the admitted table lanes.
4. Bare host-name/callable precedence and lambda-valued TreeCalc nodes consume
   the current OxFml W074 handoff; they are not redefined by this table-formula
   row-context closure.

## 4B.9. `calc-4vs8.50` Complete Table Dependency And Invalidation Matrix

Product status:

OxCalc now has an explicit W056 table dependency/invalidation matrix for
node-associated tables. The matrix is OxCalc-owned and replay-visible: it
correlates source structured-reference handles and table lifecycle callbacks
to dependency fact kinds, invalidation reasons, prepared-identity inputs, and
typed lifecycle diagnostics without moving table semantics into OxFml.

Implemented dependency fact surface:

`StructuredTableDependencyFactKind` now covers table identity, row membership,
row order, row value, column identity, column order, header text, header
region, data region, totals region, totals value, totals formula metadata,
caller row context, omitted-table enclosing context, virtual anchor/range,
workspace availability, and function registry snapshot dependency. Generic
structured-reference lowering emits the facts available from OxFml's public
table packet; `inventory_treecalc_table_dependency_facts` supplies the full
OxCalc replay inventory from the TreeCalc table snapshot/projection plus
lifecycle context versions. The function-registry snapshot fact is included
only when the caller supplies registered-function dependency evidence for the
table formula path; constant-only and registry-independent table scenarios do
not get that fact.

Implemented update matrix:

`TreeCalcTableUpdateScenarioKind` and
`TreeCalcTableLifecycleEventKind` now explicitly cover body value edit, body
formula edit, row insert/delete/reorder, column insert/delete/reorder/rename,
header text edit, totals toggle/formula edit, table rename/move/delete/resize,
node rename/move/delete, save/reopen, workspace open/close, workspace alias
mutation, function registry snapshot mutation, and structural rebind. The
classifier distinguishes stable save/reopen from identity-changing
save/reopen, records workspace/alias versions in lifecycle identity, and keeps
registry snapshot mutation as a prepared-identity/capability-sensitive input
rather than a TreeCalc-specific function branch.

Evidence:

Focused Rust tests cover the full update matrix, lifecycle callback matrix,
stable save/reopen suppression, delete/open/close event shapes, workspace alias
identity changes, registry snapshot prepared-identity invalidation, the full
dependency inventory fact surface, conditional omission of registry facts,
host-sensitive workspace availability classification, and graph-safe
context-only table descriptors. The table formula prepared-identity tests from
`calc-4vs8.49` remain the runtime evidence that actual OxFunc registry and
capability-overlay mutations flow through generic prepared identity.

Explicit non-claim:

This closes the OxCalc dependency/invalidation contract. It does not activate
DnaTreeCalc corpus cases, does not provide Excel ListObject oracle evidence,
does not close OxReplay retained comparison artifacts, and does not admit a
TreeCalc override of Excel built-in call-callee resolution. The DnaTreeCalc,
Excel, and replay evidence lanes remain with `calc-4vs8.51`, `calc-4vs8.52`,
and `calc-4vs8.53`; the current W074 host-name mapping is consumed by
`calc-4vs8.32`.

## 4B.10. `calc-4vs8.56` Dynamic Table Rebind And `INDIRECT`

Product status:

OxCalc now has an explicit dynamic table rebind lane for W056. Dynamic
references that resolve to node-associated tables no longer disappear into the
generic non-table dynamic-reference bucket: `TreeCalcDynamicTableRebindRequest`
and `classify_treecalc_dynamic_table_rebind` classify table, column, section,
current-row, and cross-workspace table targets into OxCalc-owned dependency
facts, invalidation reasons, prepared-identity inputs, typed diagnostics, and
rebind status.

Implemented dynamic target scope:

1. whole table targets depend on table identity, row membership/order, column
   identity, and data region facts,
2. column targets additionally preserve column/data-region invalidation,
3. section targets preserve header/data/totals region invalidation,
4. current-row targets require caller-context identity and reject without it,
5. cross-workspace table targets are host-sensitive, carry workspace
   availability dependency, and still retain normal table content dependencies
   for row membership/order, column identity, and data region facts,
6. table rename/move/delete, node delete, workspace close, and stable
   save/reopen use the same table lifecycle scenario vocabulary as
   `calc-4vs8.50`,
7. `INDIRECT`/volatile selector churn is represented as dynamic dependency
   activation/release/reclassification plus table-context and dynamic-selector
   prepared identity.

Typed exclusions:

Runtime parsing of TreeCalc structured-reference text is not admitted as a
shortcut. When a dynamic selector would require runtime parsing of
TreeCalc-specific structured-reference syntax, OxCalc reports
`UnsupportedRuntimeStructuredReferenceParsing`; OxFml must instead provide a
generic structured-reference bind packet, or the dynamic table reference stays
a typed exclusion. Dynamic targets that are not tables are likewise typed
excluded from table `ReferenceLike` lowering.

Evidence:

Focused Rust tests cover table/column/current-row/cross-workspace dynamic
targets, stable save/reopen suppression, table rename/move/delete, node delete,
workspace close/unavailable target handling, missing current-row caller context,
unsupported runtime structured-reference parsing, dynamic non-table target
exclusion, cross-workspace table content dependency retention, and same-table
selector changes that still require selector-identity prepared/cache
invalidation. The report keeps generic OxFml bind-packet availability and
OxFunc opaque-reference admission explicit so the seam cannot be closed by
TreeCalc-specific OxFml/OxFunc branches or eager materialization.

Still open:

1. DnaTreeCalc dynamic table corpus activation remains with `calc-4vs8.51`.
2. Excel observation and retained comparison evidence remain with
   `calc-4vs8.52` and `calc-4vs8.53`.
3. Wider function implementation for `INDIRECT` remains an OxFunc/OxFml
   typed-host-context lane; this bead closes the OxCalc dynamic table rebind
   contract and typed exclusion surface.

## 4B.11. `calc-4vs8.55` Cross-Repo Table Rollout Coordination

Product status:

`HANDOFF-CALC-006` now records the W056 table rollout map for the affected
repos. It is a coordination packet, not behavior evidence. The packet names the
repo-local counterpart beads already observed, the exact residual beads or
extensions still required, and the promotion gates that prevent downstream
repos from inventing private adapters.

Counterpart map at final table audit:

| Repo | Existing anchor | Current closure reading |
|---|---|---|
| `DnaTreeCalc` | `dtc-z0i.5` plus `dtc-z0i.5.1` through `.5.8`; `dtc-z0i.7` / `.7.1` for dynamic/cross-workspace table activation | Table-specific empty-body, lifecycle, dynamic/cross-workspace, update, retained-artifact, and namespace/anchor/workspace activation is closed and consumed by `calc-4vs8.51` and final audit `calc-4vs8.43`. Broader W004/W005 non-table and product-skeleton work remains outside table-specific closure. |
| `OxXlPlay` | `oxxlplay-4nd` plus `.1` through `.5` observation packs | Excel black-box table observation is consumed by `calc-4vs8.52` and final audit `calc-4vs8.43`; dependency/invalidation internals remain typed unavailable, and successors are needed only for future structured-table dynamic extensions not admitted here. |
| `OxReplay` | `oxreplay-p1w` family and closed `oxreplay-qb9` | Retained comparison for structured, empty-body, lifecycle, dynamic/cross-workspace, and paired Excel table artifacts is consumed by `calc-4vs8.53` and final audit `calc-4vs8.43`. Residuals such as `BLK-REPLAY-003` remain shared implementation cleanup or typed projection gaps, not table semantic blockers. |
| `OxFml` | `fml-ds0.12`, `.13`, `.14`, `.15`; `fml-ds0.6.4`, `.5`, and parent `.6` | Preserve generic structured-reference and dynamic/host packets; do not parse TreeCalc dynamic selector strings; current W051/W056 host-name mapping is frozen through the W074 handoff, while future TreeCalc built-in-call override behavior remains a new evidence/versioning lane. |
| `OxFunc` | `oxf-fcdz`; `oxf-ypq2.13` through `.16`; open `oxf-ypq2.12` | Preserve opaque `ReferenceLike` behavior; broader W093 registry-backed formula-call lookup remains outside table-specific closure, and `calc-4vs8.54` found no new W056 table blocker because table formulas already carry registry snapshot and capability-overlay identity through generic surfaces. |
| `DnaOneCalc` | no W056 table-specific bead required | Preserve no-host single-formula execution and OxFml-internal LET/LAMBDA lexical references; future VBA/XLL UDFs go through registry surfaces. |
| `OxVba` | no W056 table-specific bead required | Feed VBA/XLL discovery metadata into OxFunc W093 descriptors without TreeCalc table-selector semantics. |
| `OxIde` / `DnaOxIde` / `DnaVisiCalc` / `Foundation` | impact-scan only | No W056 table-rollout bead is required unless a future UI, doctrine, replay-pack, or shared-interface dependency is discovered. |

Promotion gates:

1. no W056 table product closure until DnaTreeCalc has active corpus and
   retained artifacts for the admitted full table scope or typed exclusions,
2. no Excel-comparable closure until OxXlPlay has retained observations or
   typed capture limits for the comparable table lanes,
3. no retained-evidence closure until OxReplay validates/replays/diffs/explains
   declared payloads without parsing producer-private strings,
4. no future name/call extension beyond the W074 W051/W056 handoff without new
   oracle evidence and a versioned rule change,
5. no future UDF/name-call extension beyond the current table impact scan until
   W093 registry snapshot/change-set and formula-call lookup are stable,
6. no DnaOneCalc regression: ordinary single-formula use requires no host table
   namespace or host-reference resolver.

Historical non-claim:

This coordination bead did not close the dependent execution beads by itself.
Those lanes are now closed and consumed by the final audit: `calc-4vs8.51`,
`calc-4vs8.52`, `calc-4vs8.53`, `calc-4vs8.54`, and `calc-4vs8.43`.

Maintenance note, 2026-05-28:

DnaTreeCalc's retained table replay producers remain live tests, not static
documentation. `active_table_corpus.rs` compares checked-in normalized replay
bundles against fresh `OxCalcTreeContext` packet projections and updates each
bundle only behind an explicit env var:
`DNATREECALC_UPDATE_RETAINED_TABLE_REPLAY`,
`DNATREECALC_UPDATE_RETAINED_TABLE_LIFECYCLE`,
`DNATREECALC_UPDATE_RETAINED_EMPTY_BODY_TABLE_REPLAY`, or
`DNATREECALC_UPDATE_RETAINED_DYNAMIC_TABLE_REPLAY`. The
`w056-table-dynamic-cross-workspace-001` bundle was refreshed after live
dynamic table rebind identity digests changed; the observable case statuses,
dependency facts, invalidation reasons, prepared identity inputs, source
handles, retained artifact refs, and manifest remained unchanged.

## 4B.12. `calc-4vs8.52` OxXlPlay Excel Table Oracle Intake

Product status:

OxCalc has accepted the current OxXlPlay W009/W056 retained table observation
floor for `calc-4vs8.52`. This is Excel black-box evidence intake, not an
OxCalc semantic shortcut and not an OxXlPlay semantic claim.

Observed OxXlPlay anchors:

1. `oxxlplay-ze4` retained the first structured-reference workbook observation.
2. `oxxlplay-4nd.1` retained the WorkbookConstructionSpec table-node equivalent
   fixture.
3. `oxxlplay-4nd.2` retained standalone table-construction evidence.
4. `oxxlplay-4nd.3` retained table update oracle observations.
5. `oxxlplay-4nd.4` added table-delete and save/reopen residual coverage.
6. `oxxlplay-4nd.5` added third-pass residuals: empty data-body tables,
   first-row insert, last-row delete, empty-table column rename, current-row
   diagnostics, and multi-table/name/anchor collision availability.

Retained artifact roots:

1. `../OxXlPlay/states/excel/xlplay_structured_reference_workbook_001/`
2. `../OxXlPlay/states/excel/xlplay_workbook_construction_spec_001/`
3. `../OxXlPlay/states/excel/xlplay_table_construction_basic_001/`
4. `../OxXlPlay/states/excel/xlplay_table_update_oracle_001/`

Evidence scope:

The retained observations cover ordinary Excel ListObject construction, table
identity/ranges, headers, data bodies, totals rows, row-context formulas,
composite structured-reference formulas, body/totals formulas, table resize,
row/column insert/delete/reorder, header edit, table rename, accepted isolated
table delete, explicit table-move capture rejection, explicit save/reopen
capture rejection, empty-body observations, first/last row boundary updates,
current-row absence diagnostics, and name/anchor collision availability.
`execution_outcome.class_id` is present for the table update oracle family.

Typed limits:

Excel dependency graph, dirty-set, and invalidation event-order internals remain
typed unavailable. OxCalc may use before/after observations and derived table
slice deltas as oracle evidence, but it must not infer Excel internal dependency
or invalidation semantics from those observations.

Still open:

1. Dynamic structured-reference or `INDIRECT` table observations are not part of
   this close. If W056 later admits an Excel-comparable dynamic structured-table
   lane rather than an OxCalc typed exclusion, OxXlPlay should add a successor
   observation bead for that exact lane.
2. OxReplay retained comparison over the DnaTreeCalc/OxXlPlay payloads remains
   with `calc-4vs8.53`.
3. DnaTreeCalc product activation and retained producer artifacts remain with
   `calc-4vs8.51`.

## 4B.13. `calc-4vs8.54` Table UDF, VBA, XLL, And Registry Impact

Product status:

OxCalc's W056 table runtime is compatible with future registered UDFs without
adding an OxCalc-local function registry or a DnaTreeCalc/DnaOneCalc function
mirror. Table formulas carry OxFunc registry snapshot identity and capability
overlay identity as prepared/cache inputs; table reference arguments remain
ordinary opaque `ReferenceLike` values backed by OxCalc readers; VBA/XLL source
discovery remains metadata that feeds OxFunc registration descriptors.

Current OxCalc evidence:

1. `TreeCalcTableFormulaRuntimeContext` carries `function_registry`,
   optional `registry_snapshot_identity`, and optional `capability_overlay`.
2. `TreeCalcTableFormulaPreparedIdentityFacts` records both host-facing
   `host_registry_snapshot_identity` and actual
   `function_registry_snapshot_identity`, plus `capability_overlay_identity`.
3. `table_formula_prepared_identity_facts_track_context_and_mutations` proves
   table formula prepared identity changes when an XLL-shaped registered UDF
   mutates the function registry snapshot or when a capability overlay changes.
4. `treecalc_table_dependency_inventory_covers_full_w056_fact_surface` includes
   `StructuredTableDependencyFactKind::FunctionRegistrySnapshot` when a table
   formula can bind registered functions, and excludes it when the caller says
   the formula has no registry-sensitive path.
5. table lifecycle/update classification maps
   `FunctionRegistrySnapshotMutation` to `CapabilitySensitive` dependency
   change and `RegistrySnapshotIdentity` prepared identity input.

Consumed counterpart anchors:

| Repo | Anchor | W056 reading |
|---|---|---|
| `OxFunc` | `oxf-ypq2.13`, `oxf-ypq2.15`, `oxf-ypq2.16` | Structured-table `ReferenceLike` values are opaque to functions; first aggregates and widened range-taking lanes consume generic resolver/reader APIs only. |
| `OxFunc` | open `oxf-ypq2.12` | Broader formula-call registry lookup migration remains open outside table-specific closure; `calc-4vs8.54` found no new W056 table blocker because table formulas already carry registry snapshot and capability-overlay identity through the generic OxFunc/OxFml surfaces. |
| `OxFml` | `fml-ds0.11`, `fml-ds0.13` | Registered-external split and table-context prepared identity are acknowledged on the formula side. Descriptor-only `REGISTER.ID`/`CALL` is adjacent state; ordinary worksheet-visible UDFs use the registry path. |
| `OxFml` | `fml-ds0.6.4`, `.5`, and parent `.6` | Current W051/W056 host-name mapping is frozen for TreeCalc consumption; no built-in-call override is admitted without future evidence. |
| `DnaOneCalc` | ready `dno-7vt4.1`, `.4`, `.5`, `.7`, `.9`; epic `dno-srj` | Future VBA/XLL support is already a DnaOneCalc extension lane. Ordinary single-formula execution still needs no host table namespace or host-reference resolver. |
| `OxVba` | `WORKSET_2026-05-10_HOST_PROGRAM_DESIGN_AND_UDF_REWORK.md`; ready `bd-sg5h` | OxVba owns VBA project/procedure discovery and host-UDF descriptor facts; those facts must be transformed into OxFunc registration requests, not consumed as TreeCalc table semantics. |

Design decisions for table formulas and UDFs:

1. A table formula that contains `MYUDF(Table1[Amount])`,
   `MYUDF([@Amount])`, or `MYUDF(path[Amount])` binds the structured reference
   through the generic OxFml table/host packet and OxCalc table resolver. The
   UDF receives the resulting value/reference according to OxFunc's public
   argument-preparation rules.
2. OxFunc may admit a UDF parameter as reference-visible only through generic
   `ReferenceLike`/resolver metadata. It may reject the call with typed
   capability or argument-preparation diagnostics if the registered function
   does not accept references. It must not inspect TreeCalc table ids,
   selectors, node ids, virtual anchors, or source spans.
3. OxCalc dependency facts stay structural: table membership, order, row value,
   column identity, header/totals/data regions, caller row context, namespace,
   registry snapshot identity, and capability overlay identity. OxCalc does not
   own UDF execution semantics, parameter coercion, or native-call policy.
4. DnaOneCalc uses the same OxFml/OxFunc registry path for future VBA/XLL UDFs
   and keeps no-host single-formula formulas unchanged. LET/LAMBDA lexical
   locals and callable locals remain OxFml-internal and do not become host
   references.
5. OxVba supplies source/procedure/invocation descriptors and capability facts.
   It does not supply table selector semantics and does not bypass the OxFunc
   registry snapshot/change-set lane.

Update scenarios:

| Scenario | Owner | Expected W056 behavior |
|---|---|---|
| UDF registered or unregistered | OxFunc registry, consumed by OxFml/OxCalc | Registry snapshot identity changes; any prepared table formula that could bind the name invalidates. |
| UDF metadata changes but invocation target is unchanged | OxFunc/OxFml | Completion/help and bind metadata update through registry snapshot/change-set evidence; table dependency facts do not invent a local metadata cache. |
| Capability overlay denies a UDF or reference argument | OxFunc/OxFml, reflected by OxCalc identity | Prepared identity includes overlay identity; evaluation returns typed capability/argument diagnostics through generic function surfaces. |
| VBA project reload changes procedure catalog | OxVba produces descriptors, OxFunc owns registration | Registration requests mutate the OxFunc snapshot; OxCalc sees only registry identity and ordinary function diagnostics. |
| XLL registration id changes | OxFunc/OxVba/XLL host adapter | Invocation target metadata changes through the registry/change-set lane; no TreeCalc selector is exposed to the native host. |
| UDF receives `ReferenceLike` and later table rows reorder | OxCalc owns table dependency | Table membership/order invalidation refreshes the reference reader; UDF semantics are rerun only through normal formula evaluation. |
| DnaOneCalc evaluates a single formula with LET/LAMBDA only | OxFml | No host table namespace or host resolver is required. Registry-backed UDF support is optional extension state. |

Still open:

1. Broader W093/OxFml registry lookup work remains outside table-specific
   closure and future extension scope. W056 table closure consumes
   `calc-4vs8.54`'s impact scan and does not claim a general UDF invalidation
   freeze beyond the current registry snapshot/capability-overlay identity
   inputs used by table formulas.
2. W056 has consumed the current W074 W051/W056 name/call freeze for TreeCalc
   host value names and lambda-valued nodes. It does not admit a TreeCalc
   override of built-in call-callee resolution; that remains a future explicit
   extension lane if product evidence justifies it.
3. No new OxCalc table-specific blocker is needed for OxVba or DnaOneCalc at
   this time; their existing extension lanes are sufficient unless a future
   UDF API intentionally exposes reference metadata beyond generic
   `ReferenceLike` admission.

## 4B.14. `calc-4vs8.51` DnaTreeCalc Node-Table Corpus And Bridge Intake

Product status:

OxCalc has consumed the DnaTreeCalc table activation floor for W056, and
`calc-4vs8.51` is closed for the table corpus/bridge intake. DnaTreeCalc has
active node-table corpus slices and retained producer artifacts that run through
`OxCalcTreeContext` and OxCalc's public table-projection/resolution APIs
rather than local formula parsing, including the previously missing
empty-body, lifecycle, and dynamic/cross-workspace table routes.

Active DnaTreeCalc table corpus slices:

1. `tables/structured-references` has 13 active cases. The runner exercises
   path-qualified table references, current-row references, headers/data/totals
   sections, `#All`, escaped table and column names, table formula/totals
   formula values, dependency lowering, update classification, retained
   `SalesTable` replay producer views, and table context identity through the
   real bridge.
2. `tables/empty-body` has 8 active cases. The runner exercises headers-only
   and headers+totals zero-row tables, `#Data`, `[Col]`, `#All`, `#Headers`,
   `#Totals`, typed `[@Col]` caller-row diagnostics, first-row insert, and
   last-row delete endpoints through the real bridge.
3. The current full DnaTreeCalc corpus validator reports 43 files, 14
   workspaces, 175 cases, and 35 active / 140 pending. The table slices above
   are active; broader non-table W004 families remain intentionally pending.

Consumed DnaTreeCalc anchors:

| Anchor | OxCalc reading |
|---|---|
| `dtc-z0i.5` | Parent table structured-reference bridge activation has the table corpus/intake evidence needed by `calc-4vs8.51`; final product audit still decides whether any broader UX/persistence residuals matter. |
| `dtc-z0i.5.1` through `.5.5` | Table model persistence, bridge table catalog projection, full table structured-reference corpus, table update retained artifacts, and lifecycle callback bridge acceptance are present as prior table anchors. |
| `dtc-z0i.5.6` / `dtc-z0i.5.6.1` | Empty-body table corpus is active and retained empty-body artifacts/OxReplay intake closed at DnaTreeCalc commit `e7b22f3`. |
| `dtc-z0i.5.7` | Table lifecycle active corpus and retained lifecycle replay closed at DnaTreeCalc commit `cdde775`. |
| `dtc-z0i.5.8` | Retained structured-reference artifact refresh after OxCalc W056 enum widening closed at DnaTreeCalc commit `7ebaca4`. |
| `dtc-z0i.7` / `dtc-z0i.7.1` | Dynamic, cross-workspace, profile-gated, and unavailable/deleted table reference routes have a table-specific active/retained slice closed at DnaTreeCalc commit `86fd0ba`; broader non-table W004 successor work remains separate. |
| `dtc-osq.6`, `.7`, `.8` | W005 walk-up, save/reopen, and click-through skeleton evidence remain product-skeleton work and do not block OxCalc's table-intake bead unless a table UX/persistence path claims closure. |

Current local evidence observed from DnaTreeCalc:

1. `pwsh docs/test-corpus/tools/validate-corpus.ps1` passed with 43 files, 175
   cases, and 35 active / 140 pending.
2. `cargo test --workspace active_table_structured_reference_corpus_executes_through_oxcalc_table_path -- --nocapture`
   passed.
3. `cargo test --workspace active_empty_body_table_corpus_executes_through_oxcalc_table_path -- --nocapture`
   passed.
4. DnaTreeCalc commit `50374fe` updates retained-artifact id projection for
   the OxCalc W056 dynamic-selector prepared-identity input and the full table
   lifecycle/update scenario enum, keeping DnaTreeCalc compatible with the
   widened OxCalc table surface.
5. DnaTreeCalc closed the retained follow-up beads identified by fresh-eyes:
   `dtc-z0i.5.6.1` at `e7b22f3`, `dtc-z0i.5.7` at `cdde775`, and
   `dtc-z0i.7.1` at `86fd0ba`.
6. OxReplay consumed the resulting third-pass producer batch through
   `calc-4vs8.53` / `oxreplay-qb9`, with eight retained artifacts admitted and
   validated without TreeCalc-private parsing.

Still open:

1. The W005 shell/save-reopen/click-through beads remain DnaTreeCalc product
   work. OxCalc must not use their partial status to claim full TreeCalc UI or
   persistence closure.
2. `calc-4vs8.43` consumes this table corpus/bridge intake in the final
   full-table audit, including cross-producer namespace/anchor/workspace
   pairing, typed projection gaps, OxXlPlay capture limits, and any accepted
   exclusions.
3. Broader non-table W004/W005 reference activation remains outside this table
   corpus/bridge intake.

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
   node-associated table spine: projection, host-reference bind, sparse readers, per-row
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

## 4B.15. `calc-4vs8.43` Full Intended Table Support Final Audit

Product status:

`calc-4vs8.43` closed the prior full-intended table audit for the then-declared
W056 table scope. That evidence remains valid, and the active W056 table status
now includes the fifth-pass `calc-4vs8.57` through `calc-4vs8.63` hardening gate
before parent W056 carries the table claim forward. The intended architecture is
unchanged: a DnaTreeCalc node table is represented to OxFml as an Excel-shaped
virtual table/ListObject anchored at a deterministic virtual cell, while table
custody remains in OxCalc/DnaTreeCalc. OxCalc owns table catalog lookup, virtual
anchor identity, table namespace/versioning, caller row context, dependency and
invalidation facts, dynamic rebind, sparse readers, and prepared/cache identity
inputs. OxFml receives generic structured-reference and table-context packets
only. OxFunc receives opaque `ReferenceLike`/sparse-reader values only. OxXlPlay
supplies black-box Excel observation, and OxReplay compares retained declared
payloads.

Prior closed support:

1. virtual table identity and anchor stability for node-associated tables,
   including save/reopen identity, table rename/move/delete, and workspace
   availability inputs,
2. structured-reference packet intake for explicit table refs, omitted-table
   current-row refs, `#Headers`, `#Data`, `#Totals`, `#All`, selected columns,
   contiguous multi-column selections, escaped names, diagnostics, exact source
   spans/tokens, effective table identity, and replay identity,
3. sparse `ReferenceLike` readers for whole table/data-body selections,
   headers, totals, all sections, current row, omitted current row, empty data
   bodies, blanks, errors, stable reader identity, and aggregate/function
   consumption without dense eager materialization,
4. per-row and totals formula execution through generic OxFml caller table
   context, with row-specific prepared identity and LET/LAMBDA lexical handling
   left inside OxFml,
5. complete table dependency and invalidation ownership for row membership,
   row order, column identity/order/header text, data/totals regions, caller row
   context, table/node/workspace/alias changes, registry snapshot changes, and
   structural rebind,
6. dynamic table reference and `INDIRECT`-style rebind classification for table,
   column, section, current-row, cross-workspace, renamed/moved/deleted,
   unavailable, volatile, and unsupported runtime-parse targets,
7. DnaTreeCalc table corpus and retained producer activation through
   `OxCalcTreeContext`, covering structured references, empty-body tables,
   lifecycle events, dynamic/cross-workspace table refs, update artifacts, and
   retained replay bundles,
8. OxXlPlay retained Excel ListObject/workbook construction observations and
   OxReplay retained table comparison/evidence over declared payloads, including
   table slices, outcomes, dependency/invalidation observation limits, prepared
   identity, source preservation, function admission, and capability snapshots,
9. UDF/VBA/XLL impact scan: future UDF registration stays registry-backed
   through OxFunc/OxFml, and table references remain opaque values rather than
   TreeCalc selectors.

Promotion evidence:

1. Third-pass table beads `calc-4vs8.39` through `calc-4vs8.43` close the
   empty-body, function evidence, lifecycle bridge, namespace/anchor/workspace,
   and final audit lane.
2. Fourth-pass table beads `calc-4vs8.44` through `calc-4vs8.56` close the
   whole-system ownership, virtual anchor, generic packet, resolver/namespace,
   full reader, row-context/prepared-identity, dependency/invalidation,
   DnaTreeCalc activation, OxXlPlay oracle, OxReplay evidence, UDF impact,
   rollout coordination, and dynamic table rebind lanes.
3. Current OxCalc checks passed for the table spine while closing the dependent
   beads: focused `structured_table`, `table_catalog_resolver`,
   `virtual_anchor_identity_contract`, dynamic table rebind, prepared identity,
   dependency/invalidation, and full `cargo test -p oxcalc-core` runs.
4. Cross-repo retained evidence consumed here includes DnaTreeCalc structured,
   empty-body, lifecycle, and dynamic/cross-workspace table artifacts; OxXlPlay
   structured-reference, workbook construction, table-construction, and
   update-oracle artifacts; and OxReplay third-pass validation with eight
   admitted artifacts and no invalid cases.

Still open:

1. Parent W056 remains open for non-table reference families, including the
   remaining W004/W005 corpus activation and retained non-table replay evidence
   tracked by `calc-4vs8.33` and `calc-4vs8.5`.
2. DnaTreeCalc W005 shell/skin/save-reopen/click-through beads remain broader
   product-skeleton work. They are not hidden table-engine exclusions and do not
   transfer UX ownership to OxCalc.
3. Excel dependency graph and dirty-set event-order internals remain typed
   unavailable observation facts. OxCalc does not infer private Excel internals
   from OxXlPlay captures.
4. Older retained OxXlPlay artifacts without `execution_outcome.class_id` remain
   typed historical capture gaps; table closure relies on the later retained
   artifacts that do declare outcome classes where the comparison lane requires
   them.
5. `BLK-REPLAY-003` remains the future OxFunc/OxReplay shared
   `comparison_value` helper cleanup. It is not TreeCalc table semantics and is
   not table support closure evidence either way.

Formal status:

This closes the W056 table topic only. It does not close parent W056, the
remaining non-table reference suite, any future extension that lets TreeCalc
host names override Excel built-in calls, or a formal proof/model lane.

## 4B.16. `calc-4vs8.57` Through `calc-4vs8.63` Node-Table Hardening Spine

The earlier table spines closed the declared W056 table scope through executable
OxCalc tests and retained cross-repo evidence. The fifth-pass hardening spine is
opened because node-associated tables are architecturally central enough that
the project should revalidate the whole-system design before parent W056 carries
the table topic forward as stable foundation.

This spine does not authorize a parallel table path. Its purpose is to prove
that the intended path is the only product path:

1. DnaTreeCalc owns table-as-node product operations, corpus, persistence, UX
   evidence, and bridge activation.
2. OxCalc owns table custody, virtual Excel-anchor projection, catalog
   resolution, sparse/reference readers, dependency facts, invalidation facts,
   dynamic rebind, host namespace versioning, and caller context identity.
3. OxFml owns structured-reference grammar, generic table-context packets,
   bind records, prepared identity inputs, name/call lanes, and LET/LAMBDA
   lexical scope.
4. OxFunc owns function semantics, registry mutation, reference-visible
   argument admission, and typed rejection outcomes over opaque carriers.
5. DnaOneCalc owns no-host single-formula guardrails and future registry-backed
   UDF consumption.
6. OxXlPlay owns clean-room Excel ListObject construction and observations.
7. OxReplay owns retained validation, replay, comparison, and explanation over
   declared producer payloads.
8. OxVba owns VBA/XLL discovery metadata alignment for future registered UDFs.

The added beads are:

1. `calc-4vs8.57` — current-state architecture revalidation and gap ledger.
   This bead compares the desired design to actual repo surfaces and classifies
   each table item as implemented/evidenced, typed exclusion, upstream blocker,
   downstream blocker, or design mismatch. It must explicitly inspect for
   private bridges, host-side formula parsing, duplicated structured-reference
   grammar, TreeCalc-specific OxFml/OxFunc branches, eager materialization
   closure paths, and stale status wording.
2. `calc-4vs8.58` — abstraction consolidation and anti-shim implementation
   sweep. This bead verifies that table-node snapshots, virtual anchors,
   resolver output, structured-reference intake, `ReferenceLike` readers,
   row-context prepared identity, dependency descriptors, invalidation facts,
   and dynamic rebind all flow through the same typed abstractions. If current
   code has separate fixture, retained-artifact, dynamic, cross-workspace, or
   row-context paths that duplicate meaning, the bead must unify them or file
   exact blockers.
3. `calc-4vs8.59` — lifecycle update and invalidation execution matrix. This
   bead makes the full update matrix executable and auditable: create/delete,
   rename/move, save/reopen, virtual-anchor changes, workspace alias and
   availability changes, row/column insert/delete/reorder/rename, empty-body
   transitions, header/totals edits, formula edits, body value edits, node
   rename/move/delete, dynamic selector rebind, and caller-row movement.
4. `calc-4vs8.60` — `ReferenceLike` function and UDF integration closure. This
   bead proves admitted functions consume opaque table references through
   generic OxFml/OxFunc paths, preserves scalar/array regressions, inventories
   typed function exclusions, and verifies that future VBA/XLL UDF registration
   flows through registry identity rather than TreeCalc selector inspection.
5. `calc-4vs8.61` — oracle, replay, and value-wire convergence gate. This bead
   aligns DnaTreeCalc, OxCalc, OxXlPlay, and OxReplay retained evidence across
   table slices, per-node/table-cell values, display where declared, execution
   outcomes, function outcomes, source preservation, prepared identity,
   dependency facts, invalidation facts, dynamic rebind facts, lifecycle facts,
   namespace/anchor/workspace facts, and registry facts where present.
6. `calc-4vs8.62` — cross-repo rollout bead reconciliation. This bead links or
   files counterpart repo beads/handoffs for OxFml, OxFunc, DnaTreeCalc,
   DnaOneCalc, OxXlPlay, OxReplay, OxVba, and any impact-scan-only repo. It
   records dependency order so downstream repos cannot implement around missing
   upstream contracts.
7. `calc-4vs8.63` — full-feature completion audit after the hardening spine.
   This bead reconciles prior table closure claims with the hardening evidence,
   updates stale docs/comments, and either reaffirms full intended node-table
   support with evidence or leaves exact blockers instead of broad completion
   language.

Completion gate:

The hardening spine closes only when it can state the table topic in the normal
OxCalc product-status shape: supported scope, decisive evidence, still-open
typed exclusions, and formal status. It cannot close on dense materialization,
fixture-only readers, private formula parsing, mirrored structured-reference
grammar, TreeCalc-specific OxFml/OxFunc branches, Excel-internal inference, or
a DnaOneCalc no-host regression.

## 4B.17. `calc-4vs8.57` Current-State Node-Table Gap Ledger

This ledger is the `calc-4vs8.57` revalidation result. It maps the desired
node-associated table architecture to the actual current repo state before the
hardening implementation beads run.

Reviewed inbound observations:

1. OxFml `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md` sections 2E, 2F, 27, and
   28 remain aligned with the W056 table plan: table objects are host/OxCalc
   owned, OxFml owns generic structured-reference grammar/bind packets, and
   W051/W056 TreeCalc host names map through the defined-name /
   defined-name-`LAMBDA` lane.
2. The current OxFml table-context packet is still the right first semantic
   packet: `table_catalog`, `enclosing_table_ref`, and
   `caller_table_region`, with stable table identity, range, column catalog,
   header/totals presence, caller row, and prepared/replay identity facts.

Current-state classification:

| Area | Owner | Current state | Evidence | Gap or next bead |
| --- | --- | --- | --- | --- |
| Table-as-node product model and direct context activation | DnaTreeCalc | Intended product path is direct `OxCalcTreeContext` use. Legacy semantic adapter modules are now wrong-shape residue and must be deleted in the DnaTreeCalc boundary-cleanup beads. | Prior DnaTreeCalc table evidence remains migration evidence only until regenerated through the direct context path. | Delete DnaTreeCalc bridge modules/tests/docs and reissue active table corpus evidence through direct `OxCalcTreeContext`. |
| Table custody, virtual anchor, catalog resolver, sparse readers, prepared identity, dependency/invalidation, dynamic rebind, save/reopen identity | OxCalc | Implemented/evidenced in typed Rust surfaces and now reachable through context-native table APIs. `EvalValue` remains free of table-specific variants; table values travel through sparse/reference-reader surfaces. Workspace save/reopen uses an OxCalc-owned context snapshot rather than host reconstruction of private maps. | `src/oxcalc-core/src/structured_table.rs` defines the table mechanics; `src/oxcalc-core/src/consumer.rs` now stores node-associated table snapshots in `OxCalcTreeContext` and exposes `set_node_table`, `clear_node_table`, `table_view`, `workspace_table_views`, `table_context_packet`, `resolve_table_reference`, `lower_table_reference`, `lower_table_bind_record`, `classify_dynamic_table_rebind`, `export_workspace_snapshot`, and `import_workspace_snapshot`. Focused tests `treecalc_context_owns_node_table_lifecycle_and_views`, `treecalc_context_routes_table_catalog_lowering_and_dynamic_rebind`, and `treecalc_context_export_import_preserves_identity_and_recalc_state` are green. | DnaTreeCalc still needs direct-context migration and retained artifact refresh. Full W056 table closure still requires direct DnaTreeCalc/OxReplay/OxXlPlay evidence. |
| Generic structured-reference packets and table prepared identity | OxFml | Implemented/evidenced for W056 table needs without TreeCalc semantics. | OxFml `fml-ds0.12`, `.13`, `.15`, `.16`; W074 name-call handoff `fml-ds0.6.5`. | Broader W036/table-language work remains outside this W056 hardening unless live evidence exposes a missing generic packet fact. Keep under watch in `calc-4vs8.62`. |
| Opaque `ReferenceLike` function behavior | OxFunc | Implemented/evidenced for first aggregate group and widened/classified reference-visible structured-table function families through generic resolver APIs. | OxFunc `oxf-ypq2.13`, `.15`, `.16`. | Broader W093 formula-call registry migration `oxf-ypq2.12` remains open. It is not table-specific today, but `calc-4vs8.60` must verify table formulas still invalidate on registry/capability changes and do not require TreeCalc selector inspection. |
| DnaOneCalc no-host reference guardrail and future UDF path | DnaOneCalc | No table-specific host-reference dependency found. Ordinary single-formula use stays on OxFml internals; future VBA UDF work remains a registry-backed WS-15 lane. | `../DnaOneCalc/src/dnaonecalc-host/tests/scenarios/runtime_metadata.rs` includes ordinary no-host and LET/LAMBDA local/callable/capture guardrails; `../DnaOneCalc/src/dnaonecalc-host/src/adapters/oxfml/live_bridge.rs` documents default OxFunc registry use and future `with_function_registry` shape; WS-15 beads own desktop VBA UDF work. | `calc-4vs8.60` must keep DnaOneCalc table work non-impact unless UDF registry behavior changes. Do not introduce a host namespace requirement for ordinary formulas. |
| Excel ListObject observation | OxXlPlay | Implemented/evidenced as clean-room black-box observation, with explicit unavailable facts for Excel internals. | OxXlPlay `oxxlplay-4nd.1` through `.5`; retained `xlplay_workbook_construction_spec_001`, `xlplay_table_construction_basic_001`, and `xlplay_table_update_oracle_001`. | Excel dependency graph, dirty-set, event-order internals, modal table-move/save-reopen risks remain typed unavailable; `calc-4vs8.61` must preserve them as unavailable, not inferred. |
| Retained replay comparison | OxReplay | Implemented/evidenced for declared retained payloads; comparison is over typed views, not producer-private strings. | OxReplay `oxreplay-qb9`; retained batch `docs/test-runs/w007-w056-table-third-pass-intake-baseline/`; matched table mechanics under `host_rollout_matched_table_001`. | `BLK-REPLAY-003` remains for the OxFunc-owned `comparison_value` replay-wire helper; full namespace/anchor/workspace cross-producer pairing and older OxXlPlay outcome gaps remain final-audit risks for `calc-4vs8.61`, not table semantic ownership. |
| VBA/XLL discovery metadata and UDF registration pressure | OxVba / OxFunc / DnaOneCalc | No table-specific blocker found. VBA/XLL source discovery and UDF registration remain registry-seam work; table references should appear only as opaque references if later admitted to UDFs. | OxVba `WORKSET_2026-05-10_HOST_PROGRAM_DESIGN_AND_UDF_REWORK.md`; DnaOneCalc WS-15 open beads; OxFunc W093 registry contract surfaces. | Keep as impact-scan in `calc-4vs8.60` and `calc-4vs8.62`; do not add table-specific UDF branches. |

Ownership drift scan:

1. No current evidence requires OxFml to parse TreeCalc table paths or know
   TreeCalc table lifecycle semantics.
2. No current evidence requires OxFunc to inspect TreeCalc selectors or table
   packets.
3. No current evidence supports dense/eager materialization as table closure
   evidence; table values remain sparse/reference-reader backed.
4. No current evidence supports DnaTreeCalc-local formula parsing as a product
   route. The corpus may declare packet modes/handles as fixture input, but
   product formula meaning must continue through OxCalc/OxFml public packets.
5. No current evidence lets OxReplay infer TreeCalc or Excel table semantics
   from private strings. It compares declared producer payloads and records
   typed projection gaps.

Ledger consequence:

No new W056 table bead is required beyond the existing fifth-pass hardening
spine. The current gaps map to:

1. `calc-4vs8.58` for the OxCalc abstraction and anti-shim sweep,
2. `calc-4vs8.59` for lifecycle/update invalidation execution proof,
3. `calc-4vs8.60` for function/UDF/no-host guardrail revalidation,
4. `calc-4vs8.61` for replay/value-wire/oracle residuals,
5. `calc-4vs8.62` for cross-repo bead-state reconciliation, especially the
   open DnaTreeCalc table parent beads,
6. `calc-4vs8.63` for the final table status audit.

## 4B.18. `calc-4vs8.58` Node-Table Abstraction And Anti-Shim Sweep

Product status:

The OxCalc node-associated table path now has a focused anti-shim regression
covering the intended single route from a TreeCalc table node to formula
execution:

1. TreeCalc-owned table data enters OxCalc as `TreeCalcTableNodeSnapshot`.
2. `project_treecalc_table_node_snapshot` produces the generic OxFml-facing
   virtual Excel table packet (`TableDescriptor` inside
   `StructuredTableContextPacket`) plus OxCalc-only namespace, anchor,
   row-membership, row-order, column, body, totals, and invalidation
   identities.
3. OxFml `bind_formula` preserves source span, exact token text, token kind,
   bind-record handle, effective table identity, selected regions/columns,
   caller-context dependency, and typed diagnostics in a public
   `StructuredReferenceBindRecord`.
4. `StructuredTableDependencyLoweringRequest::from_oxfml_bind_record` and
   `lower_structured_table_dependencies` consume that same bind record and
   generic table context packet to produce OxCalc-owned dependency descriptors
   keyed by the source reference handle.
5. `TreeCalcTableSparseReader::from_oxfml_bind_record` consumes the same bind
   record and table projection to create the sparse/reference-reader surface
   used by runtime.
6. `RuntimeEnvironment` receives the generic table context and sparse
   reference values, and the aggregate executes through OxFml/OxFunc without
   exposing TreeCalc selectors to OxFunc.

Evidence:

1. New focused Rust test:
   `node_table_path_uses_shared_packet_reader_dependency_and_runtime_surfaces`
   in `src/oxcalc-core/src/structured_table.rs`.
2. The test proves the `SalesTable[Amount]` path preserves source-token facts,
   lowers row membership, row order, column identity, and data-region
   descriptors from the same source reference handle, creates a
   `ReferenceLike` sparse reader over the virtual Excel range, executes
   `SUM(SalesTable[Amount])` through the OxFml/OxFunc runtime, and confirms
   the `SUM` lane remains `SparseReferenceLike` with no TreeCalc selector
   visibility and no eager-materialization closure evidence.

Still open:

1. `calc-4vs8.59` must execute the lifecycle/update matrix over the same
   abstractions and record invalidation evidence for edits, row/column
   mutation, namespace movement, delete/unavailable cases, save/reopen, and
   registry/capability changes.
2. `calc-4vs8.60` must revalidate function/UDF integration and the DnaOneCalc
   no-host guardrail against the table formula path.
3. `calc-4vs8.61` through `.63` still own replay/value-wire convergence,
   cross-repo bead reconciliation, and the final scoped table completion audit.

Architecture result:

No new table-specific `EvalValue` variant, dense/eager materialization route,
private structured-reference parser in DnaTreeCalc, TreeCalc branch in OxFml,
or TreeCalc selector inspection in OxFunc was added for this bead. The current
OxCalc implementation keeps the table-associated node model, virtual Excel
anchor, generic OxFml packet, dependency lowering, sparse reader, runtime
binding, and OxFunc aggregate path on one typed abstraction chain.

## 4B.19. `calc-4vs8.59` Lifecycle Update And Invalidation Matrix

Product status:

OxCalc now has fifth-pass executable evidence for the node-table lifecycle
matrix over the same table snapshot, projection, lifecycle callback,
dependency-lowering, sparse-reader, prepared-identity, and dynamic-rebind
surfaces used by the rest of W056.

Executable evidence:

1. `TreeCalcTableUpdateScenarioKind::ALL` and
   `TreeCalcTableUpdateScenarioKind::stable_id()` provide a stable update
   scenario inventory for retained evidence and handoff correlation.
2. Existing matrix tests continue to cover body cell edit, body formula edit,
   row insert/delete/reorder, column insert/delete/reorder/rename, header text
   edit, totals toggle/edit, table rename/move/delete/resize, node
   rename/move/delete, save/reopen, workspace open/close, workspace alias
   mutation, function-registry snapshot mutation, and structural rebind.
3. New focused Rust test:
   `table_lifecycle_boundary_matrix_states_identity_stability` in
   `src/oxcalc-core/src/structured_table.rs`.
4. That test adds explicit boundary evidence for first-row insert, last-row
   delete, empty-body transition, virtual-anchor movement, cross-workspace
   availability-version movement, caller-row movement, and dynamic selector
   rebind.

Identity results:

1. First-row insert keeps the stable reader id but changes the source and
   snapshot identities because the declared table-reference extent changes; it
   invalidates row membership, row order, caller-context, and table-context
   prepared identity inputs.
2. Last-row delete changes row membership/order and reader snapshot identity,
   and shrinks the declared reader extent.
3. Empty-body transition stays reference-preserving through the zero-row
   `ReferenceLike` structured carrier, with table-context identity invalidated
   rather than materializing a dense empty value array.
4. Virtual-anchor movement changes the reference target/range and region
   dependency facts through `TableMove`.
5. Cross-workspace availability-version movement is host-sensitive and enters
   prepared identity through host namespace and resolution-rule inputs.
6. Caller-row movement changes `caller_context_id` and prepared formula key
   while preserving the dispatch skeleton for the same formula text.
7. Dynamic current-row table references rebind on row-order lifecycle changes
   through caller-row dependency facts, row-order invalidation, and
   caller-context prepared identity.

Cross-repo evidence classification:

1. DnaTreeCalc remains owner of actual product table operations and UI /
   persistence lifecycle evidence. Existing retained table artifacts cover the
   structured-reference, empty-body, lifecycle, and dynamic/cross-workspace
   table producer slices, but parent bead-state residue is still reconciled
   under `calc-4vs8.62`.
2. OxReplay remains owner of retained comparison. Current retained batches
   admit the declared DnaTreeCalc and OxXlPlay table artifacts, while older
   Excel internals such as dependency graph, dirty-set, event ordering, modal
   save/reopen, and clipboard-mediated table move remain typed unavailable or
   producer-capture limits, not inferred OxCalc semantics.
3. OxFml continues to own prepared identity inputs and generic structured
   bind records; OxCalc's lifecycle facts do not become OxFml table semantics.

Still open:

`calc-4vs8.60` must revalidate ReferenceLike/function/UDF and DnaOneCalc
guardrails, `calc-4vs8.61` must converge retained replay/value-wire evidence,
`calc-4vs8.62` must reconcile cross-repo bead state, and `calc-4vs8.63` must
perform the final table completion audit.

## 4B.20. `calc-4vs8.60` ReferenceLike Function And UDF Integration Closure

Product status:

OxCalc's node-associated table implementation is revalidated for the first
reference-preserving aggregate group and for generic registry-backed UDF
boundary routing. `SUM`, `COUNT`, `COUNTA`, and `COUNTBLANK` execute over
opaque structured-table sparse `ReferenceLike` bindings. A table current-row
reference also reaches the OxFml registry-backed host-function callback lane
through the generic function registry and typed host callback provider without
OxFunc or OxFml inspecting TreeCalc table selectors.

Executable evidence:

1. Existing `table_sparse_runtime_bindings_feed_first_aggregate_group`
   exercises `=SUM(SalesTable[Amount])`, `=COUNT(SalesTable[Amount])`,
   `=COUNTA(SalesTable[Amount])`, and
   `=COUNTBLANK(SalesTable[Amount])` through OxFml/OxFunc sparse
   `ReferenceLike` bindings with no dense table materialization.
2. Existing function-breadth inventory tests keep every admitted or blocked
   range-taking family on generic carrier modes:
   `SparseReferenceLike`, `ReferenceShapeOnly`,
   `ResolverIndexedReference`, `MultiReferenceResolver`, typed dynamic-array
   host context, or typed exclusion. All inventory rows assert no TreeCalc
   selector visibility and no eager-materialization closure.
3. New focused Rust test
   `table_current_row_reference_reaches_registry_backed_host_udf_boundary` in
   `src/oxcalc-core/src/structured_table.rs` builds a current-row
   `[@Amount]` table reader, feeds its runtime binding into OxFml with a
   registry-backed `TABLE_IDENTITY_PROBE` UDF entry, and confirms the call
   reaches the generic `HostFunctionProvider` boundary as a user-defined
   `vba_host_callback` availability lane.
4. Existing
   `table_formula_prepared_identity_facts_track_context_and_mutations` proves
   table formula prepared identity changes on host namespace version,
   structure context version, resolution-rule version, capability profile,
   OxFunc registry snapshot identity, and capability overlay changes.

Boundary result:

The UDF boundary test deliberately records today's remaining generic blocker:
the host callback receives the current-row structured-reference argument as a
generic `#VALUE!` argument, not as a TreeCalc selector and not as an eagerly
materialized table-specific value. This is the correct non-shim failure mode
for the current W093/W074 state. Table UDF value/reference admission therefore
remains blocked on the OxFml/OxFunc registry-backed host-callback
argument-preparation lane, not on OxCalc table lowering.

Cross-repo ownership:

1. OxFunc owns the opaque `ReferenceLike` aggregate behavior and W093 registry
   mutation/change-set semantics. Its table-specific evidence covers
   `oxf-ypq2.13`, `.15`, and `.16`; broader formula-call registry migration
   remains `oxf-ypq2.12`.
2. OxFml owns formula-call binding, host callback invocation, W074 name/call
   precedence evidence, and any future generic rule for whether a UDF receives
   a structured reference as a scalar value, an opaque `ReferenceLike`, or a
   typed rejection.
3. DnaOneCalc remains non-impact for table host references. Ordinary
   single-formula execution and LET/LAMBDA lexical variables/callables continue
   to require no host namespace or table resolver; future VBA/XLL UDF support
   must enter through the same OxFunc/OxFml registry surfaces.
4. OxVba supplies future VBA/XLL discovery metadata only. It does not receive
   TreeCalc table selectors, and it must not become a table-reference semantic
   owner.

Still open:

`calc-4vs8.60` does not freeze UDF table-reference argument semantics. Product
closure for UDFs consuming table references requires OxFml/OxFunc W093/W074
evidence that the generic host-callback/reference-visible path admits the
desired carrier or returns a typed rejection. The W056 table lane can proceed
because first aggregate behavior, registry/capability invalidation inputs, and
the non-shim UDF boundary are now explicit.

## 4B.21. `calc-4vs8.61` Oracle Replay And Value-Wire Convergence Gate

Product status:

OxCalc now has an executable W056 node-table replay/oracle convergence
inventory. It ties the table runtime facts in OxCalc to retained DnaTreeCalc
producer artifacts, OxXlPlay Excel ListObject observations, and OxReplay
validate/replay/diff/explain evidence without moving comparison policy into
OxCalc and without allowing private formula-string parsing or Excel-internal
dependency inference.

Executable evidence:

1. New Rust inventory
   `TREECALC_TABLE_REPLAY_EVIDENCE_LANES` in
   `src/oxcalc-core/src/structured_table.rs` records the retained evidence
   lanes, owner repo, artifacts/beads, replay view families, value-wire field,
   blockers, typed projection gaps, and non-inference rules.
2. New focused test
   `table_replay_evidence_inventory_converges_oracle_and_value_wire_lanes`
   asserts that all required lanes are present and that no lane allows
   producer-private structured-reference/formula parsing or Excel-internal
   dependency inference.
3. The required lane set covers DnaTreeCalc table producer views, OxCalc
   runtime packet and prepared-identity facts, OxXlPlay Excel ListObject
   oracle views, OxReplay third-pass intake, OxReplay matched TreeCalc/Excel
   comparison, the `comparison_value` helper replacement blocker,
   Excel dependency/dirty/event-order typed unavailability, legacy
   `execution_outcome.class_id` projection gaps, and namespace/anchor/workspace
   cross-producer pairing gaps.
4. The required view-family set covers `table_slice`, `per_node_value`,
   `comparison_value`, `effective_display_text`, `execution_outcome`,
   `table_update_oracle`, `dependency_evidence`, `invalidation_evidence`,
   `retained_artifact_ref`, `source_preservation`, `prepared_identity`,
   `dynamic_table_rebind`, and `registry_snapshot_identity`.

Consumed evidence:

1. DnaTreeCalc retained table producer artifacts:
   `../DnaTreeCalc/docs/test-runs/w056-table-structured-references-001/`,
   `../DnaTreeCalc/docs/test-runs/w056-table-empty-body-001/`,
   `../DnaTreeCalc/docs/test-runs/w056-table-lifecycle-001/`, and
   `../DnaTreeCalc/docs/test-runs/w056-table-dynamic-cross-workspace-001/`.
2. OxCalc executable table runtime evidence from `calc-4vs8.58`,
   `calc-4vs8.59`, and `calc-4vs8.60`: source preservation, prepared identity,
   dependency/invalidation, lifecycle update, dynamic rebind, sparse
   reference binding, and registry/capability identity.
3. OxXlPlay retained Excel observation artifacts:
   `../OxXlPlay/states/excel/xlplay_workbook_construction_spec_001/`,
   `../OxXlPlay/states/excel/xlplay_table_construction_basic_001/`, and
   `../OxXlPlay/states/excel/xlplay_table_update_oracle_001/`.
4. OxReplay retained comparison artifacts:
   `../OxReplay/docs/test-corpus/bundles/host_rollout_w056_table_third_pass_001/`,
   `../OxReplay/docs/test-runs/w007-w056-table-third-pass-intake-baseline/`,
   `../OxReplay/docs/test-corpus/bundles/host_rollout_matched_table_001/`,
   and
   `../OxReplay/docs/test-runs/w007-host-rollout-host_rollout_matched_table_001-baseline/`.

Convergence result:

The table evidence path is now explicit:

1. DnaTreeCalc publishes declared table-node producer views through the real
   bridge.
2. OxCalc publishes runtime packet, source, dependency, invalidation,
   lifecycle, dynamic rebind, prepared identity, and registry/capability facts.
3. OxXlPlay observes the nearest Excel ListObject behavior as black-box
   workbook/table evidence.
4. OxReplay validates, replays, diffs, and explains declared retained payloads.

The convergence inventory treats `comparison_value` as the shared value-wire
field for comparable outputs, but keeps `BLK-REPLAY-003` active for replacing
OxReplay's local comparator seam with an OxFunc-owned replay helper. That
blocker is not TreeCalc table semantics and is not an excuse to add an
OxCalc-local value adapter.

Typed gaps:

1. Excel dependency graph, dirty-set, and invalidation event-order internals
   remain typed unavailable. OxCalc may compare before/after observable outputs
   and declared table-update oracle payloads, but it must not infer Excel's
   internal dependency graph.
2. Older OxXlPlay structured/workbook/table-construction retained artifacts may
   lack `execution_outcome.class_id`; `xlplay_table_update_oracle_001` carries
   that class id. This is projection metadata, not a table semantic gap.
3. Dynamic/cross-workspace namespace, anchor, and workspace evidence exists,
   but full cross-producer namespace/anchor/workspace pairing remains a final
   audit projection gap for `calc-4vs8.63`.

Ownership result:

No replay/oracle lane introduced a private bridge, a producer-private formula
parser, a TreeCalc-specific OxFml/OxFunc branch, dense/eager table
materialization, or Excel-internal inference. OxCalc keeps only the typed
runtime and invalidation facts it owns; OxReplay keeps comparison governance;
OxXlPlay keeps Excel observation; DnaTreeCalc keeps product table producer
artifacts.

## 4B.22. `calc-4vs8.62` Cross-Repo Rollout Bead Reconciliation

Product status:

OxCalc now has an executable cross-repo rollout inventory for the W056
node-associated table hardening spine. The inventory links each affected repo
to its responsibility, counterpart beads or non-impact anchor, promotion order,
evidence obligation, residual action, and seam guardrails. This is coordination
evidence, not a substitute for another repo's local bead truth.

Executable evidence:

1. New Rust inventory
   `TREECALC_TABLE_CROSS_REPO_ROLLOUT_LANES` in
   `src/oxcalc-core/src/structured_table.rs` records the cross-repo rollout
   graph.
2. New focused test
   `table_cross_repo_rollout_inventory_records_counterparts_and_seam_rules`
   asserts that every affected repo has a lane, each lane has counterpart
   anchors or explicit non-impact rationale, dependency order is recorded, and
   no lane allows producer-private parsing or semantic mirrors.
3. The test pins the important residuals: DnaTreeCalc parent-bead
   reconciliation is open but not a table semantic blocker; OxFunc
   `oxf-ypq2.12` is adjacent unless future table UDF admission depends on it;
   OxReplay `BLK-REPLAY-003` remains value-wire cleanup and must not create an
   OxCalc value adapter.

Rollout lanes:

| Repo | Status | Counterpart anchors | W056 reading |
| --- | --- | --- | --- |
| `OxFml` | Closed evidence | `fml-ds0.12`, `.13`, `.15`, `.16`, `.6.5` | Generic structured-reference packets, table prepared identity, zero-row packets, token-kind preservation, and current W051/W056 name-call handoff are available without TreeCalc semantics. Future W036/W074 extensions need new versioned evidence. |
| `OxFunc` | Open adjacent non-blocking | `oxf-ypq2.13`, `.15`, `.16`, open `.12` | Opaque structured-table `ReferenceLike` behavior is evidenced for W056 table functions. Broader formula-call registry migration remains W093-adjacent, not a current table semantic blocker. |
| `OxCalc` | Closed evidence | `calc-4vs8.57` through `.61` | Table custody, virtual anchors, resolver, readers, lifecycle matrix, UDF boundary, and replay/value-wire convergence are now hardened for the declared table scope. |
| `DnaTreeCalc` | Open parent reconciliation | `dtc-z0i.5.1` through `.5.8`, `dtc-z0i.7.1`, open parents `dtc-z0i.5.6` and `dtc-z0i.5` | The child evidence needed by OxCalc exists for structured refs, empty bodies, lifecycle, retained artifacts, and dynamic/cross-workspace table cases. The open parents are bead-graph hygiene and should be closed or narrowed in DnaTreeCalc after its unrelated dirty bead state is reconciled. |
| `OxXlPlay` | Closed evidence | `oxxlplay-4nd.1` through `.5` | Excel workbook construction, standalone table construction, table update oracle, residual empty-body/boundary observations, and explicit unavailable Excel internals are retained. |
| `OxReplay` | Open adjacent non-blocking | `oxreplay-qb9`, `oxreplay-p1w.3`, `BLK-REPLAY-003` | Retained table comparison and matched TreeCalc/Excel table mechanics are evidenced over declared payloads. `BLK-REPLAY-003` remains shared `comparison_value` helper cleanup, not TreeCalc table semantics. |
| `DnaOneCalc` | Explicit non-impact | `dno-rl7u`, WS-15 beads `dno-7vt4.1`, `.4`, `.5`, `.7`, `.9` | Ordinary single-formula execution still requires no host table namespace; future VBA/XLL UDF work remains registry-backed through OxFunc/OxFml. |
| `OxVba` | Future extension tracked | `bd-sg5h`, `WORKSET_2026-05-10_HOST_PROGRAM_DESIGN_AND_UDF_REWORK.md` | VBA/XLL discovery supplies descriptor metadata for future registration requests only. It does not receive TreeCalc table selectors or own name/function precedence. |
| `OxIde` / `DnaOxIde` / `DnaVisiCalc` / `Foundation` | Explicit non-impact | impact scan | No direct W056 node-table seam was found. Open local beads only if a future UI, visual, doctrine, or shared-interface dependency appears. |

Dependency order:

1. OxFml and OxFunc generic packet/function contracts must stay ahead of OxCalc
   runtime closure.
2. OxCalc runtime closure must stay ahead of DnaTreeCalc table producer
   activation.
3. DnaTreeCalc producer artifacts and OxXlPlay observations must stay ahead of
   OxReplay retained comparison.
4. DnaOneCalc and OxVba remain registry/future-UDF consumers, not table
   semantic owners.

Residual actions:

1. DnaTreeCalc should close or narrow `dtc-z0i.5.6` and `dtc-z0i.5` in its own
   repo once its unrelated dirty bead-state change around `dtc-z0i.8` is
   reconciled. OxCalc does not edit that bead file from this bead.
2. OxFunc `oxf-ypq2.12` remains broader W093 formula-call registry migration.
   It becomes table-blocking only if a later product claim admits table
   references into UDF callbacks beyond the current typed boundary.
3. OxReplay `BLK-REPLAY-003` remains value-wire helper cleanup and must not
   create an OxCalc-local comparison adapter.

Coordinator seam review:

No rollout lane requires a private bridge, parsing another repo's private
strings, duplicated structured-reference grammar, duplicated name/call
precedence, eager table materialization, or a TreeCalc-specific OxFml/OxFunc
branch. Where a repo still has open adjacent work, the inventory classifies it
as parent reconciliation, future extension, or non-table adjacent work rather
than hiding it inside the W056 table semantic claim.

## 4B.23. `calc-4vs8.63` Final Node-Table Completion Audit

Product status:

The node-associated TreeCalc table topic is complete for the declared W056
table slice after the fifth-pass hardening spine. A table attached to a
DnaTreeCalc node can be projected by OxCalc as the host-owned equivalent of an
Excel table anchored at a stable virtual cell: OxFml supplies generic
structured-reference packets and source preservation, OxCalc owns the table
namespace/resolver/readers/dependencies/invalidation/caller context, OxFunc
sees only opaque `ReferenceLike` carriers or typed function rejections, and
DnaTreeCalc/OxXlPlay/OxReplay provide producer/oracle/replay evidence.

Supported table scope:

1. Structured-reference packets and projections for `path[Col]`,
   `path[@Col]`, `[#Headers]`, `[#Data]`, `[#Totals]`, `[#All]`, composite
   ranges, omitted table names, caller-row context, escaped names, and
   multi-column ranges.
2. Sparse `ReferenceLike` readers for data body, headers, totals,
   all-sections, current row, single-row tables, empty data bodies, and
   sparse traversal without `EvalValue::Table` or dense/eager closure
   evidence.
3. OxCalc-owned table dependency and invalidation facts for row membership,
   row order, column identity, header text, data region, totals region, caller
   row context, namespace version, structure context version, and registry
   snapshot identity.
4. Per-row column formulas, totals formulas, first aggregate and
   reference-visible function groups, typed exclusions for functions that need
   extra host context, and the current host-UDF boundary metadata.
5. Table create/delete/rename/move/reorder, stable virtual anchors,
   same-node table lookup, dynamic table references, `INDIRECT`-style table
   targets, workspace aliases, unavailable workspace degradation, and stable
   save/reopen identity.
6. Retained TreeCalc/OxCalc/OxXlPlay/OxReplay evidence for table slices,
   `comparison_value`, effective display text, execution outcome, dependency
   evidence, invalidation evidence, and table update observations.
7. DnaOneCalc ordinary single-formula use remains explicitly non-impact:
   no host table namespace is required, and LET/LAMBDA lexical references stay
   inside OxFml.

Executable evidence:

1. New Rust inventory `TREECALC_TABLE_FINAL_AUDIT_ITEMS` in
   `src/oxcalc-core/src/structured_table.rs` records the final audit status
   for table packets, readers, dependencies, functions/UDF boundary, dynamic
   rebind, replay/oracle value wire, DnaTreeCalc parent reconciliation,
   DnaOneCalc no-host guardrail, future VBA/XLL descriptors, and broader
   non-table W056 work.
2. New focused test
   `table_final_audit_marks_node_table_complete_without_parent_w056_overclaim`
   asserts that every table audit item has evidence anchors, no item allows
   dense/eager materialization, private formula parsing, or TreeCalc-specific
   OxFml/OxFunc branches, and only the explicit non-table W056 spine blocks
   parent W056 closure.
3. The final audit consumes the closed hardening beads:
   `calc-4vs8.57` architecture revalidation, `calc-4vs8.58` abstraction
   consolidation, `calc-4vs8.59` lifecycle execution matrix,
   `calc-4vs8.60` function/UDF boundary closure, `calc-4vs8.61`
   oracle/replay/value-wire convergence, and `calc-4vs8.62` cross-repo
   rollout reconciliation.

Still open:

1. Parent W056 remains open for non-table reference work: broad W004/W005
   activation, bare host-name/callable lanes, node-as-function, dynamic
   non-table references, cross-workspace non-table references, and retained
   non-table replay evidence.
2. DnaTreeCalc parent beads `dtc-z0i.5.6` and `dtc-z0i.5` need repo-local
   close-or-narrow graph hygiene after its unrelated bead-state dirt around
   `dtc-z0i.8` is reconciled. This does not block the OxCalc table semantic
   claim because the table child evidence is already linked.
3. OxFunc `oxf-ypq2.12` remains a broader formula-call registry migration
   item. It becomes table-blocking only if a later product claim admits table
   references into UDF callbacks beyond the current typed boundary.
4. OxReplay `BLK-REPLAY-003` remains shared `comparison_value` helper cleanup.
   It must not create an OxCalc-local value adapter and does not reopen table
   semantics.
5. Excel dependency graph, dirty-set, and event-order internals remain typed
   unavailable. OxCalc-owned invalidation facts are not inferred from Excel.
6. Full cross-producer namespace/anchor/workspace pairing remains a retained
   evidence projection gap. The admitted OxCalc dynamic table runtime behavior
   remains supported.

Formal/proof status:

This bead is an executable audit and coordination closure, not a new formal
model proof. It narrows table status for parent W056 by separating completed
node-table behavior from remaining non-table reference work. Any future table
extension that changes grammar, name/call precedence, UDF callback semantics,
or dynamic table host capabilities needs a new versioned packet/registry lane
and fresh evidence before it can amend this audit.

## 4C. Non-Table Reference Completion Spine

After `calc-4vs8.43`, the node-associated table topic has prior promotion
evidence, and `calc-4vs8.57` through `calc-4vs8.63` now serve as the explicit
hardening gate before parent W056 carries that table claim forward. In
parallel, the remaining non-table reference runtime/evidence spine is below.

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
4. `calc-4vs8.32` — callable host-name and W074 precedence intake. This
   consumes the closed OxFml W074 handoff for the current W051/W056 mapping:
   TreeCalc host value names map to the Excel defined-name value lane,
   lambda-valued host nodes map to the defined-name-`LAMBDA` lane, explicit
   host references bypass ordinary name/call ambiguity through the generic
   host hook, built-ins keep the call-callee frontier, and namespace/context/
   registry identities remain prepared-identity inputs. It does not add an
   OxCalc private precedence mirror or admit built-in call overrides.
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

### 4C.1. `calc-4vs8.64` Non-Table Reference Category Matrix And Red/Green Suite

Product status:

OxCalc now has an executable coordinator matrix for the full non-table
TreeCalc reference-resolution surface. The matrix is not a closure claim: it
records which categories are fully green, which have active direct-context
slices, which are deliberately typed pending/exclusion lanes, and which still
need DnaTreeCalc runner activation or retained non-table replay evidence. The
table topic remains out of scope here because it is closed for the declared
W056 table slice.

Executable surface:

1. `W056_NON_TABLE_REFERENCE_CATEGORIES` in
   `src/oxcalc-core/src/formula.rs` records the category id, examples,
   spec anchor, expected-outcome contract, corpus/test-suite anchors, runnable
   command, OxCalc status, DnaTreeCalc status, replay status, and current test
   result.
2. `w056_non_table_reference_category_matrix_is_complete_and_runnable` is the
   normal green inventory check. It fails if a category lacks enough
   descriptive detail to write expected-outcome cases, lacks a runnable suite
   command, or hides parent-W056 blockers.
3. `w056_non_table_reference_resolution_full_scope_red_green_gate` is an
   ignored red/green closure gate. It is intentionally runnable and currently
   red until every category stops blocking W056 non-table closure:
   `cargo test -p oxcalc-core w056_non_table_reference_resolution_full_scope_red_green_gate -- --ignored --nocapture`.

Reference matrix:

| Category | Examples | Spec/test suite | Current status and result | Remaining closure work |
| --- | --- | --- | --- | --- |
| Children collection | `@CHILDREN`, `.*`, `base.@CHILDREN`, `base.*` | `CORE_MODEL_SPEC.md` §3.5/§3.5b/§3.7; DnaTreeCalc `references/children-raw-active` and `references/set-membership-active`; OxCalc W051/W056 ChildrenV1 tests | Product green for the declared `ChildrenV1` path through OxFml host-reference syntax packets and OxCalc direct-context resolver outputs, including the broad raw set-membership mirror's `Q1.*` and `@CHILDREN` cases. | No declared-slice blocker; collection order/replay closure is tracked by the ordered/set-membership rows. |
| Bare walk-up and dotted descent | `Margin`, `Q1.Margin`, `A.B.C` | `CORE_MODEL_SPEC.md` §3.2/§3.7; DnaTreeCalc `references/walkup-raw-active` plus pending `references/walkup`; OxCalc host-name bind tests | Focused non-cell-like raw formula slice is green through OxFml unresolved-host-name bind candidates and OxCalc direct-context resolver outputs. | Full walk-up remains pending for name/cell precedence cases such as `Q1`; retained non-table replay. |
| Ancestor/root anchors | `^`, `^.Rate`, `^^.Year`, `^^^`, `[]Sheet1.Margin`, `[]` | `CORE_MODEL_SPEC.md` §3.1/§3.2/§3.7; DnaTreeCalc `references/anchors-raw-active` plus pending `references/anchors` | Focused ancestor-anchor forms are green through OxFml repeated-prefix host-reference packets and OxCalc `RelativePath` resolver outputs. | Workspace-root anchors, sheet aliases, broader anchor corpus, and retained replay evidence. |
| Workspace aliases and `!` syntax | `[projections]Branch1.MyNode`, `Sheet1!Foo`, `[ws][Branch X].MyNode` | `CORE_MODEL_SPEC.md` §3.1/§3.3; DnaTreeCalc `references/cross-workspace`; OxCalc `calc-4vs8.30`/`calc-8tox` | OxCalc provider/alias packet and workspace-qualified carrier are implemented. DnaTreeCalc direct-context raw cross-workspace syntax is currently typed pending/exclusion. | Implement product raw cross-workspace runtime or keep an explicit typed exclusion, then retain evidence through OxReplay. |
| Escaping, canonicalization, case | `[Sales Q1]`, `[Foo'[Bar]`, `Region.[Net Revenue]`, `sales.margin` | `CORE_MODEL_SPEC.md` §3.3/§3.4; DnaTreeCalc `references/escaping-raw-active` plus pending `references/escaping` | Focused bracket-escaped path forms are green through OxFml escaped-path packets and OxCalc decoded-segment resolver outputs. Case-insensitive canonical lookup is covered by the active walk-up slice. | Full syntax/canonical display/profile coverage and retained evidence. |
| Meta invisibility and accessors | hidden meta lookup, `@PREV` skipping meta sibling, `@NAME`, `@INDEX`, `@PARENT`, `@FORMULA` | `CORE_MODEL_SPEC.md` §2/§3.5/§6 item 9; DnaTreeCalc `references/meta-nodes` | Active meta-node slice is green through OxCalc context-owned `is_meta` state: host-name lookup hides meta-effective subtrees, active sibling navigation skips meta nodes, and metadata accessors execute through direct context. | Broader ordered selectors over meta-effective snapshots and retained evidence. |
| Single sibling navigation | `@PREV.Net`, `@NEXT.Margin`, `ref.@PREV` | `CORE_MODEL_SPEC.md` §3.5/§3.5b/§3.7; DnaTreeCalc `references/sibling-offsets` | Focused unqualified and qualified `@PREV`/`@NEXT` tail forms plus out-of-range descriptors are green through OxFml host-reference packets and OxCalc sibling carriers. Qualified forms such as `Q2.@PREV.Net` preserve relative-bound dependency details rather than becoming host-static direct references. | Retained replay for the non-table suite. |
| Ordered set selectors | `@PRECEDING`, `@FOLLOWING`, `@ANCESTORS` | `CORE_MODEL_SPEC.md` §3.5/§3.5b/§3.7; DnaTreeCalc `references/ordered-raw-active`, `references/set-membership-active`, and semantic oracle `references/set-membership` | Active ordered and broad raw set-membership slices are green through OxFml host-reference packets, including qualified structural-base selectors, non-empty/empty preceding sets, following sets, and ancestor sets. DnaTreeCalc now asserts semantic order through OxCalc collection descriptor projection instead of generic graph edge order. | Retained replay. |
| Recursive descent | `**.Margin`, `Base.**.Margin` | `CORE_MODEL_SPEC.md` §3.5b/§3.6/§3.7; DnaTreeCalc `references/ordered-raw-active` and `references/set-membership-active`; OxCalc traversal tests | Active recursive slices are green through OxFml host-reference packets carrying base and tail tokens into OxCalc traversal, including absolute recursive-tail and relative all-descendant forms. | Traversal-bound replay and retained replay. |
| Reference literals and arrays | `{A, C, A}`, `{A, 1}`, array-valued node reference | `CORE_MODEL_SPEC.md` §3.5b/§6 item 4; DnaTreeCalc `references/literals-active`, `references/literals`, `arrays/array-references` | Focused raw reference-only arrays are green through OxFml braced host-reference packets and OxCalc `ReferenceLiteralArrayV1`, preserving order and duplicates; mixed scalar/reference arrays remain typed exclusions. | Broad array-reference parsing and retained evidence. |
| Dynamic `INDIRECT` and CTRO | `INDIRECT("Sheet1!Foo")`, `INDIRECT(selector_node)`, dynamic target switch | `CORE_MODEL_SPEC.md` §4/§6 item 7/§10.3; DnaTreeCalc `dynamic-references/indirect`; OxCalc dynamic carrier tests | OxCalc `DynamicPotential`/`DynamicResolved` facts exist. DnaTreeCalc direct-context raw `INDIRECT` syntax is currently typed pending/exclusion. | Implement product raw dynamic runtime or keep an explicit typed exclusion, then retain dynamic evidence. |
| Cross-workspace runtime refs | `[accounts]Revenue`, `[Other.xlsx]Sheet1!Foo` | `CORE_MODEL_SPEC.md` §3.3/§10.4; DnaTreeCalc `references/cross-workspace`; OxCalc workspace-qualified carrier tests | OxCalc preserves external workspace handles and DnaTreeCalc direct-context raw runtime syntax is currently typed pending/exclusion. | Product runtime admission or explicit typed exclusion, retained replay, and cross-producer evidence. |
| Bare host names | `=Revenue`, `=Margin + 1`, `=My.Region.Sales` | `CORE_MODEL_SPEC.md` §3.2/§3.9; OxFml W074 handoff consumed by `calc-4vs8.32` | Current mapping is admitted: host values use the defined-name value lane. OxCalc inventory was updated from stale W074-blocked wording. | DnaTreeCalc raw formula host-name runner and retained evidence. |
| Node-as-function / lambda-valued nodes | `Doubler(5)`, `My.Node(1, 2)`, `^.Rate(x)` | `CORE_MODEL_SPEC.md` §3.8/§3.9; DnaTreeCalc `references/node-functions` | W074 mapping is closed for the current defined-name-LAMBDA lane. DnaTreeCalc direct-context callable host names are currently typed pending/exclusion. | Implement product callable host-node runtime or keep an explicit typed exclusion, then retain evidence. |
| Profile gating | `treecalc-v1` accepts `@ANCESTORS`; `strict-excel` rejects TreeCalc syntax | `CORE_MODEL_SPEC.md` §4; DnaTreeCalc `profiles/gating` | DnaTreeCalc has an active direct-context typed-pending runner. The future strict-Excel `INDIRECT("Sheet1!Foo")` case now rejects explicitly with `typed_exclusion:strict_excel_profile_not_supported`, rather than drifting into TreeCalc path semantics or failing as an accidental unresolved runtime reference. | Implement the future Excel-compatible strict profile, then add parser/binder profile admission, strict-Excel rejection evidence, and retained replay. |
| Structural-edit rebind | rename, move, delete, reorder siblings | `CORE_MODEL_SPEC.md` §8a; DnaTreeCalc `structural-edits/propagation`; OxCalc structural invalidation tests | DnaTreeCalc `dtc-z0i.17` is closed: the active direct-context runner covers delete, rename with explicit propagation, rename without propagation, move out of scope, and insert-shadow consequences through OxCalc edit APIs. | Broader propagation UX and retained invalidation/replay evidence. |
| Unresolved/invalid/self-reference diagnostics | `MissingName`, naked `[]`, self-reference through walk-up | `CORE_MODEL_SPEC.md` §3.2/§3.5b/§7; DnaTreeCalc `references/walkup-raw-active`, pending `references/walkup`, and `references/syntax` | Active walk-up subset covers unresolved diagnostics. Full self-reference/name-cell cases remain pending. | Full syntax/diagnostic runner and retained evidence. |

Specification clarity pass:

Every category above has enough spec and corpus detail to write expected-outcome
examples now: concrete syntax examples, scope/caller rules, success or rejection
outcomes, and the owning runner path are identified. The current gap is not
missing descriptive specification; it is activation and retained evidence for
the pending or partial categories.

Current check results observed in this pass:

1. OxCalc active broad-reference corpus:
   `cargo test -p oxcalc-core w056_active_reference_corpus_executes_broad_raw_formulas_through_oxfml_path -- --nocapture`;
   passed. This direct-context test exercises representative children,
   walk-up/dotted, ancestor, escaped path, metadata accessor, sibling,
   ordered-selector, recursive, reference-literal, dynamic `INDIRECT`, and bare
   host-name formulas through `OxCalcTreeContext`, with OxFml candidate
   diagnostics and source-correlated reference descriptors.
2. DnaTreeCalc active non-table reference runner group:
   `cargo test -p dnatreecalc-host --test active_children_corpus --test active_walkup_corpus --test active_anchor_corpus --test active_escaping_corpus --test active_sibling_offsets_corpus --test active_ordered_corpus --test active_set_membership_corpus --test active_reference_literals_corpus --test active_meta_nodes_corpus --test active_dynamic_cross_workspace_corpus --test active_profile_gating_corpus --test active_structural_edits_corpus -- --nocapture`;
   passed.
3. DnaTreeCalc profile-gating runner:
   `cargo test -p dnatreecalc-host --test active_profile_gating_corpus -- --nocapture`;
   passed. `prof-indirect-dispatch` now uses `INDIRECT("Sheet1!Foo")` and
   expects the current explicit strict-profile pending rejection:
   `typed_exclusion:strict_excel_profile_not_supported`.
4. DnaTreeCalc node-function typed-pending runner:
   `cargo test -p dnatreecalc-host --test active_node_functions_corpus -- --nocapture`;
   passed.
5. OxCalc matrix inventory:
   `cargo test -p oxcalc-core w056_non_table_reference_category_matrix_is_complete_and_runnable -- --nocapture`;
   passed.
6. OxCalc red/green closure gate:
   `cargo test -p oxcalc-core w056_non_table_reference_resolution_full_scope_red_green_gate -- --ignored --nocapture`;
   failed as intended, listing every still-red non-table category. The test is
   ignored by default so normal verification remains green while the full-scope
   red/green target stays executable.

### 4C.2 `calc-4vs8.5.1` Epoch/Snapshot Split For CTRO Value Updates

W056 dynamic-reference work now treats the main runtime axes as separate
versioned layers rather than as one structural edit stream:

1. `structural_snapshot_id` changes only for structural edits such as add,
   delete, rename, move, reorder, table-shape change, or formula attachment
   replacement. A literal input value update must not allocate a successor
   structural snapshot.
2. `value_epoch` is the input/value layer version. `OxCalcTreeContext` now
   records node input values separately from `StructuralSnapshot`, increments
   the workspace value epoch on `set_node_input_value`, and records the node's
   current `input_value_epoch` for reader/export visibility.
3. `formula_artifact_token` is the formula text/bind/prepared identity layer.
   Formula text changes still move through formula artifact identity; literal
   value edits do not increment formula artifact identity and do not force
   parse/bind.
4. `overlay_epoch` is the published runtime-effect layer for CTRO/dynamic
   dependency facts. In the current implementation this is represented by the
   published `RuntimeEffect` set plus candidate/publication identity; W054 owns
   the durable retention class, pinning, eviction trace, and explicit cache/GC
   counters for this layer.

Current implemented behavior:

1. `set_node_input_value` updates only the input-value layer, clears any stale
   formula-output seed for the edited literal node, and emits an
   `UpstreamPublication` invalidation seed. It no longer calls
   `StructuralSnapshot::apply_edit` and no longer advances the structural
   snapshot allocator.
2. Local recalc composes working values as structural seed constants,
   published values, then current input values. This preserves legacy initial
   literal-node fixtures while allowing the published value layer to mask
   stale structural constants during literal/formula transitions and ensuring
   later literal edits win through the value layer.
3. Invalidation still derives from the old published effective graph: static
   descriptors plus previously published CTRO dynamic dependency effects. A
   value edit to the old dynamic target therefore invalidates the dynamic owner
   before the owner has re-evaluated.
4. Candidate evaluation may publish new dynamic dependency effects. At
   publication, value deltas and dependency-effect deltas remain atomic; a
   reject path preserves the prior published value/effect state.

Focused evidence:

1. `treecalc_context_input_value_update_recalculates_dependents_without_full_reset`
   covers `A=3`, `B=A+1`, then `A=4`: the structural snapshot id is preserved,
   `value_epoch` increments, the `input_value_epoch` moves for `A`, `B`
   recalculates to `5`, and the exported snapshot keeps structural constant
   seed `3` separate from current input value `4`.
2. `treecalc_context_indirect_resolves_reference_text_and_records_ctro_edge`
   now also checks that a value edit to the old dynamic target
   `INDIRECT("B"&A)` preserves structural snapshot identity while invalidating
   the dynamic owner through the prior published CTRO overlay.

### 4C.3 `calc-4vs8.5.1` Formula-Token Split Follow-Up

The second epoch/snapshot slice separates formula-to-formula text edits from
structural snapshot allocation. For nodes that are already formula-backed and
remain formula-backed, `set_node_formula_text` now increments the formula text
version and formula artifact identity used by the catalog builder, records a
formula-edit classification diagnostic, and seeds recalc without replacing the
structural formula attachment or allocating a successor structural snapshot.

Current formula-edit classifications are replay-visible as diagnostics:

1. `same_dependencies` for formula text changes whose resolved dependency
   signature is unchanged.
2. `dependency_shape_changed` for resolved static dependency-shape changes.
3. `unresolved_to_resolved` and `resolved_to_unresolved` for host-name
   resolution transitions surfaced by the OxFml/OxCalc catalog build.
4. `cycle_candidate` when the successor formula catalog creates a dependency
   cycle before publication.

Static dependency-shape classifications are also first-class publication
deltas. `dependency_shape_changed`, `unresolved_to_resolved`, and
`resolved_to_unresolved` map to `DependencyShapeUpdate` records before the
candidate is admitted. Published candidates now carry these records in both
`AcceptedCandidateResult.dependency_shape_updates` and
`PublicationBundle.dependency_shape_updates`; the TreeCalc runner projects the
same records into the commit bundle JSON and classifies dependency-shape
updates as publish-critical rather than local-floor-only.

Focused evidence:

1. `treecalc_context_formula_edit_recalculates_dependents` now proves `A = 3`
   to `A = 4` preserves structural snapshot identity, leaves `value_epoch`
   unchanged, increments the formula text version, classifies
   `same_dependencies`, and recalculates dependent `B`.
2. `treecalc_context_formula_edit_changed_dependency_preserves_structure_and_recalculates`
   proves `B = A + 1` to `B = C + 1` preserves structural snapshot identity,
   leaves `value_epoch` unchanged, classifies `dependency_shape_changed`, and
   recalculates downstream `D`. The accepted candidate and publication bundle
   both carry a `static_dependency_shape_changed` delta over the edited owner
   and old/new static targets.
3. `treecalc_context_formula_edit_unresolved_to_resolved_preserves_structure`
   and
   `treecalc_context_formula_edit_resolved_to_unresolved_rejects_without_structural_change`
   cover both host-name resolution directions without structural snapshot
   mutation. The resolving direction publishes a `static_dependency_resolved`
   delta; the rejecting direction emits no publication bundle.
4. `treecalc_context_formula_edit_cycle_reject_preserves_structure_and_prior_publication`
   proves a successor self-cycle is rejected without structural snapshot
   mutation and without publishing over the prior accepted value.
5. `treecalc_runner_emits_local_run_artifacts` now treats
   `dependency_shape_updates` as a publish-critical commit-bundle category.

### 4C.4 `calc-4vs8.5.1` Literal/Formula Transition Cleanup

The third epoch/snapshot slice removes the remaining structural edit fallback
from literal-to-formula and formula-to-literal transitions. These transitions
now advance only the interim formula/input layers owned by `OxCalcTreeContext`:

1. Literal-to-formula increments the formula text version, removes the node's
   input value epoch, preserves the prior literal value as the published
   baseline for reject/no-publish behavior, and seeds formula recalc without
   replacing the structural formula attachment or structural constant.
2. Formula-to-literal increments the formula text version, writes the new input
   value and `input_value_epoch`, removes any stale formula-output publication
   seed for that node, and invalidates dependents through an
   `UpstreamPublication` seed.
3. Both directions classify the dependency-shape transition and publish it as a
   first-class `DependencyShapeUpdate`: `literal_to_formula` maps to
   `static_formula_dependency_activated`, and `formula_to_literal` maps to
   `static_formula_dependency_released`.
4. Candidate rejection still preserves the previous published value surface.
   A literal-to-formula edit that would create a cycle is rejected without a
   publication bundle, while the old literal value remains visible through the
   published baseline rather than falling back to stale structural seed data.

Focused evidence:

1. `treecalc_context_literal_to_formula_preserves_structure_and_publishes_activation`
   proves `A = 3` to `A = C + 1` preserves structural snapshot identity,
   leaves `value_epoch` unchanged, removes the input value epoch, recalculates
   `A` and dependent `B`, and publishes a
   `static_formula_dependency_activated` delta.
2. `treecalc_context_formula_to_literal_preserves_structure_and_publishes_release`
   proves `A = C + 1` to `A = 7` preserves structural snapshot identity,
   increments `value_epoch`, recalculates dependent `B`, and publishes a
   `static_formula_dependency_released` delta.
3. `treecalc_context_literal_to_formula_cycle_reject_preserves_prior_literal_value`
   proves `A = 4` to `A = B + 1` is rejected as a cycle without structural
   mutation or publication, while the previous literal value `4` and dependent
   publication `B = 5` remain visible.

### 4C.5 `calc-4vs8.5.1` Dynamic-Reference Classification Retraction

The attempted dynamic-reference formula-edit classification that recognized
runtime-reference function names inside OxCalc has been retracted. OxCalc must
not inspect formula text for evaluator/function semantics such as `INDIRECT` or
`OFFSET`.

Current corrected state:

1. `set_node_formula_text` compares OxCalc-owned dependency descriptors and
   unresolved diagnostics only. It does not probe formula text for dynamic
   function names.
2. `dynamic_dependency_changed` remains available only when already-typed
   dependency descriptor facts include `DynamicPotential` and their signatures
   actually change.
3. Runtime CTRO target deltas remain evaluation/publication facts derived from
   old published runtime effects versus candidate runtime effects.

Future requirement:

A formula-edit classification that distinguishes runtime-reference declaration
changes without relying on evaluated CTRO target deltas needs a typed OxFml/FEC
contract: prepared or bound formula facts must expose opaque runtime-reference
effect declarations, handles, resolver identity inputs, and typed diagnostics.
OxCalc can then compare those typed facts. Until that exists, OxCalc must not
infer dynamic-reference formula shape from function names or formula syntax.

Non-claims: dynamic-reference formula-edit classification beyond already-typed
dependency descriptors is not implemented in OxCalc. TraceCalc differential
lanes remain follow-up work for the broader W056 formula-edit coverage.

### 4C.6 Snapshot-Layer Rework Routed To W057

The W056 epoch/snapshot slices are corrective implementation steps, not the
final state-kernel design.

W056 establishes the product-facing invariants that value edits, formula text
edits, literal/formula transitions, static dependency-shape publication, CTRO
runtime effects, and no-publish rejection must not require structural snapshot
allocation. The current implementation still contains interim maps and
compatibility fields used to preserve behavior while those invariants are
exercised.

W057 owns the deeper representation rework:

1. introduce `WorkspaceRevision` as the immutable tuple of
   `StructureSnapshot`, `NodeInputSnapshot`, and `NamespaceSnapshot`,
2. move authoritative literal value and formula text input truth into
   `NodeInputSnapshot`,
3. consume OxFml parse/bind/prepared results as typed
   `FormulaBindingSnapshot` facts,
4. derive `DependencyShapeSnapshot` from workspace roots and typed formula
   facts,
5. keep `PublicationSnapshot` and `RuntimeOverlaySet` separate from authored
   workspace truth,
6. retarget W054 retention/pinning/eviction identities onto those layers.

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

Implementation note for `calc-4vs8.22`: the active table path now relies on
OxFml parsing and binding structured references against the generic virtual
table catalog supplied by OxCalc. OxCalc consumes the resulting
`StructuredReferenceBindRecord` packets to resolve table node/table identity,
selected regions/columns, row context, sparse readers, dependency facts, typed
diagnostics, and replay identity. It covers `path[Col]`, `path[@Col]`,
section/column composites such as `path[[#Headers],[Col]]`, omitted current-row
forms such as `[@Col]`, and diagnostics for unknown table paths or columns
through the OxFml bind record, not through an OxCalc formula parser.

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
surfaces plus traversal-bound evidence, or retained full direct-context evidence. The
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

## 4E. Boundary Correction For Former Formula-Text Parse/Rewrite Surfaces

Correction recorded 2026-05-24: OxCalc-owned formula-text parse/rewrite
surfaces are not the intended architecture. OxFml owns formula lexing, parsing,
binding, source spans, operator precedence, array grammar, structured-reference
grammar, `LET`/`LAMBDA`, and diagnostics around formula syntax. OxCalc owns the
TreeCalc model, table model, host namespace state, resolver callbacks,
reference carriers/readers, dependency facts, invalidation facts, and prepared
identity inputs.

The former `calc-4vs8.7`, `calc-4vs8.13`, and related parse/rewrite surfaces are
migration evidence only and are superseded by `calc-4vs8.33.4` plus OxFml
`fml-f64`. They must be deleted from the product API and replaced by:

1. OxCalc supplies `HostFormulaContext` with:
   - `dialect_id = oxcalc.treecalc-v1`,
   - `capability_profile_id = host-capabilities:treecalc-v1`,
   - `resolution_rule_version = treecalc-host-resolution:v1`,
   - host namespace version,
   - structure context version,
   - caller context identity,
   - table context identity,
   - registry snapshot/capability overlay identity where relevant,
   - declarative host syntax rules.
2. OxFml parses formula text once, applies the declared host syntax rules, and
   emits generic packets preserving source span/token and rule family.
3. OxCalc resolves those packets against `OxCalcTreeContext`, then lowers the
   result to `TreeReference`, sparse readers, `RuntimeHostReferenceBindResult`,
   dependency descriptors, and invalidation facts.
4. DnaTreeCalc consumes only `OxCalcTreeContext` edit/recalc/view APIs.

The initial declarative host syntax inventory is:

| Family | Pattern shape that OxFml must be able to recognize from host rules | OxFml packet facts | OxCalc resolver facts |
|---|---|---|---|
| Children | `@CHILDREN`, `.*`, `<host-path>.@CHILDREN`, `<host-path>.*` | source span/token, selector family, optional base token/span, shape `collection` | base node, `ChildrenV1`, membership/order versions, member value edges |
| Ordered selectors | `@PRECEDING`, `@FOLLOWING`, `@ANCESTORS`, qualified forms | source/base/selector spans, selector family, shape `collection` | ordered member ids, traversal policy identity, membership/order versions |
| Recursive descent | `**`, `**.<tail>`, `<host-path>.**`, `<host-path>.**.<tail>` | selector and tail spans/tokens | traversal result, tail resolution, traversal-bound diagnostics |
| Sibling offsets | `@PREV`, `@NEXT`, optional `.<tail>` | selector/tail spans, shape `single` | target sibling, unresolved/out-of-range diagnostic, rebind-on-order-change |
| Ancestor/root anchors | `^`, `^.<tail>`, repeated `^`, `[]`, `[].<tail>` | host path packet, not exponent/operator rewriting | caller-sensitive path target, root target, structural rebind facts |
| Workspace paths | `[workspace]<path>`, first-position `!<path>`, bracket-escaped segments | workspace/path token spans and opaque path payload | alias/provider resolution, availability version, canonical path |
| Reference literal arrays | `{<host-ref>(,<host-ref>)*}` only where host rules mark it reference-only | element spans and reference/scalar classification diagnostics | `ReferenceLiteralArrayV1` or typed mixed scalar/reference exclusion |
| Node table structured refs | `<host-path>[...]`, `[...]` with enclosing table context | generic structured-reference bind record plus optional host path payload | table node/table id, selected regions/columns, row context, sparse reader |

The old OxCalc-local formula rewriting and table structured-reference parsing
sections have been removed from this workset. Git history and bead history are
the record of that discarded shape. Current W056 closure evidence must use
OxFml parse/bind packets plus OxCalc resolver outputs only.

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

This intake did not itself freeze name/call precedence. OxFml W074 still owns
broader bind/editor cache migration and any future product extension that
changes the current rule. The current W051/W056 host-name rule is now consumed
by `calc-4vs8.32`: TreeCalc host values follow the Excel defined-name value
lane, lambda-valued host nodes follow the defined-name-`LAMBDA` lane, and
built-ins keep the call-callee frontier.

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
does not activate bare host-name/callable corpus evidence, and does not add
dynamic `INDIRECT` cross-workspace rebind. The workspace-qualified
carrier/dependency path is consumed by `calc-8tox`; the current W074 callable
mapping is consumed by `calc-4vs8.32`; dynamic and retained evidence closure
remains under `calc-4vs8.31` and `calc-4vs8.33`.

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

This is still not DnaTreeCalc corpus activation, retained replay evidence, or
dynamic `INDIRECT` cross-workspace rebind. Current W074 host-name mapping is
consumed by `calc-4vs8.32`; remaining corpus/replay/dynamic work stays under
the open W056 successor beads.

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
`INDIRECT` end-to-end product evidence, or cross-workspace dynamic rebind. The
current W074 host-name mapping is consumed by `calc-4vs8.32`; remaining
corpus/replay/dynamic work stays under `calc-4vs8.33` and the narrowed
`calc-4vs8.5` blocker.

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

This did not by itself freeze bare host-name resolution, callable TreeCalc host
nodes, workbook/sheet/UDF/defined-name precedence, or broader formula-call
registry lookup/cache invalidation. The current W051/W056 host-name mapping is
now consumed by `calc-4vs8.32`; broader bind/editor cache migration remains
non-blocking OxFml work unless a future TreeCalc extension needs a new rule.

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

This does not close W004/W005 non-table corpus activation for host-name/dotted
formula binding, dynamic/cross-workspace references, node-as-function/
lambda-valued nodes, or retained OxReplay evidence for non-table references.

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
6. node-associated TreeCalc table support has prior implementation and
   evidence through the `calc-4vs8.21` through `calc-4vs8.56` table spines and
   has passed the fifth-pass `calc-4vs8.57` through `calc-4vs8.63` hardening
   gate: architecture revalidation, abstraction consolidation, lifecycle
   execution, function/UDF integration, oracle/replay convergence, cross-repo
   rollout reconciliation, and final completion audit,
7. OxFml/OxFunc integration remains through public generic host-context and
   reference/value surfaces,
8. known exclusions and any new cross-repo handoffs are explicit.

## 6. Status

Product status: node-associated TreeCalc table support is complete for the
declared W056 table slice after the fifth-pass hardening gate
`calc-4vs8.57` through `calc-4vs8.63`. The table claim covers generic OxFml
structured-reference packets, OxCalc-owned table namespace/resolver/readers/
dependency/invalidation/caller-context facts, opaque OxFunc `ReferenceLike`
function consumption or typed exclusions, DnaTreeCalc producer evidence,
OxXlPlay Excel observation, OxReplay retained comparison, and DnaOneCalc
no-host non-impact. Broader W056 remains in progress for non-table references,
W004/W005 non-table corpus activation, dynamic/cross-workspace non-table
product evidence, and retained non-table replay evidence. Current W074
host-name mapping is consumed for W051/W056; future TreeCalc built-in-call
override behavior remains a new evidence/versioning lane. W051 is
closed for the first OxCalc `ChildrenV1` carrier pattern; W056 now has a typed Rust
implementation-input inventory for the broader reference family, a first
structured table-context dependency-lowering surface for the current generic
OxFml table packet, and a typed dependency/reverse-edge/invalidation/rebind
projection over current OxCalc graph facts. Runtime preparation now consumes
the typed W056 identity needs through public OxFml `RuntimeHostFormulaContext`
fields where available, and the local edge-value cache includes the resulting
prepared formula identity in its call-site key. OxCalc now also exposes a
public raw TreeCalc formula-text host-reference bind for free-standing `@CHILDREN` and
`.*`, plus a caller-supplied resolved-base host-reference bind contract for
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
spelling, plus caller-supplied resolved-collection host-reference bind into the existing
`OrderedSelectorV1` carrier and runtime sparse reader path. DnaTreeCalc commit
`66355f8` activates the first receiving-side ordered-selector corpus slice for
`@PRECEDING`, `@FOLLOWING`, `@ANCESTORS`, and `Base.**.Margin` through those
public query/resolved-collection packets. `calc-4vs8.14` keeps the generic
OxFml host-reference bind packet shape explicit by reporting ordered-selector
family hints separately from `ChildrenV1`. `calc-4vs8.15` records the
cross-repo W093/W074 registered-external reconciliation intake: OxFunc source
mapping, registered-external split, and JavaScript custom-function metadata
mapping are no longer W056 upstream packet blockers. Current W074 host-name
mapping is now consumed by `calc-4vs8.32`; broader bind/editor cache migration
remains non-blocking OxFml work. `calc-4vs8.16` adds an OxCalc-owned
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
The table area has prior closure evidence in the bead graph:
`calc-4vs8.21` through `calc-4vs8.29`, `calc-4vs8.34` through `calc-4vs8.38`,
`calc-4vs8.39` through `calc-4vs8.43`, and `calc-4vs8.44` through
`calc-4vs8.56`. The prior table scope includes persistence/corpus activation,
generic structured-reference packet breadth, opaque function admission, Excel
observation, retained comparison, lifecycle/version policy, empty-body support,
namespace/anchor/workspace semantics, virtual anchor identity, full
`ReferenceLike` readers, row-context prepared identity, complete dependency and
invalidation scenarios, DnaTreeCalc activation, OxXlPlay oracle construction,
OxReplay retained evidence, UDF/VBA/XLL impact, cross-repo rollout
coordination, and dynamic table reference rebind/`INDIRECT` semantics. This
evidence has now passed the `calc-4vs8.57` through `calc-4vs8.63` hardening
spine, which revalidates the intended node-associated TreeCalc table feature
without adding TreeCalc semantics to OxFml/OxFunc. It is still not a
full-reference W056 closure claim because the non-table reference suite remains
open.

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
Rust tests proving `=SUM(@CHILDREN)` and `=SUM(.*)` host-reference bind to neutral
`TREE_REF_*` OxFml source, preserve source token text/spans, produce
`ChildrenV1` carriers, reject unsupported raw TreeCalc reference families, and
execute end-to-end through the existing OxCalc/OxFml/OxFunc reference path.
`calc-4vs8.8` adds focused Rust tests proving `=SUM(base.@CHILDREN)` and
`=SUM(base.*)` can host-reference bind through exact source-span-keyed resolved-base
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
OxFml commit `9269421` narrowed W074 table-name collision evidence; the later
OxFml `4a55709` handoff now freezes the current W051/W056 TreeCalc host-name
mapping consumed by `calc-4vs8.32`.
`calc-4vs8.12` adds focused Rust tests proving ordered selector collections
lower to `TreeReferenceCollectionMembership` plus member-value descriptors,
preserve host-reference handles and ordered member ids, enter W056 inventory as
admitted implementation inputs, project through the local TreeCalc carrier
projection path, and expose a sparse reader that uses resolver-supplied member
order without parsing TreeCalc text.
`calc-4vs8.13` adds focused Rust tests proving ordered-selector query packets
preserve source/base/selector/tail spans and exact token text for unqualified,
qualified, and recursive-tail forms; unresolved ordered selectors emit typed
diagnostics; host-reference binding with caller-supplied resolved collections produces
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
structured-reference host-reference bind packets that preserve source spans/tokens,
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
direct OxCalcTreeContext paths (`e59c6f1`, `a5f7b65`, `b59b2fb`, `8eba3cb`), OxXlPlay
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
broader W036 table formula semantics remain separate, while the current
non-table W074 name/call mapping is consumed below.

OxFml `fml-ds0.6.4`, `fml-ds0.6.5`, and parent `fml-ds0.6` at commits
`f6811f4` and `4a55709` close the W074 name/call freeze needed by W056's
current TreeCalc host-name mapping. OxCalc consumes the handoff at
`../OxFml/docs/handoffs/HANDOFF_CALC_005_W074_NAME_CALL_FREEZE.md`: host value
names follow the Excel defined-name value lane, lambda-valued host nodes follow
the defined-name-`LAMBDA` lane, registered UDFs remain callable-only unless
shadowed by a visible host/defined-name lane, lexical LET/LAMBDA names remain
OxFml-internal, sheet/workspace/caller context and namespace versions are
identity inputs, and explicit TreeCalc references/structured table packets stay
on their generic host-hook lanes. This closes `calc-4vs8.32` as an upstream
W056 intake item, but it does not close full non-table corpus activation,
dynamic/cross-workspace retained evidence, or any future extension that would
let TreeCalc host names override Excel built-in calls.

Still open: exercised OxFml host-reference packets beyond the admitted
children/table/ordered-selector/cross-workspace/reference-literal-array
slices, DnaTreeCalc activation for the remaining W004/W005 non-table reference
suite, broader non-table end-to-end scenarios, and retained non-table evidence
intake. The
provider/alias/first-position `!` packet shape, workspace-qualified carrier
path, reference-literal array carrier path, generic structured-reference table
packet coverage, and current W074 W051/W056 host-name mapping are no longer
missing, but blocker `calc-4vs8.5` remains open for the remaining full-W056
non-table closure scope.

Formal status: no proof claim.
