# Notes for DnaTreeCalc

Status: `active`
Owner lane: `OxCalc`
Relationship: outbound observation and integration note for DnaTreeCalc
TreeCalc formula-text intake

## 1. Purpose

Record the OxCalc-owned surface that reduces DnaTreeCalc blocker
`BLK-DTC-001` without moving TreeCalc formula parsing into the host repo.

This note is an OxCalc-owned observation ledger. DnaTreeCalc has now consumed
the first free-standing and qualified children slice in commit `6611684`; the
broader reference/table suite remains successor work. OxCalc has also added the
public raw ordered-selector query/resolved-collection prebind surface that the
next DnaTreeCalc corpus activation slice can consume.

## 2. Core Message

OxCalc now exposes public raw formula-text prebind surfaces for the first
explicit TreeCalc host-reference collection syntax.

DnaTreeCalc should submit original formula text to OxCalc rather than parsing
or lowering `@CHILDREN` / `.*` locally. OxCalc rewrites only the source handed
to OxFml and returns a `TreeFormula` carrying the existing
`TreeCalcReferenceCollection::ChildrenV1` reference carrier.

For qualified children syntax, DnaTreeCalc must still not parse TreeCalc
semantics locally. The current receiving-side slice uses OxCalc's public
qualified-base query packets plus DnaTreeCalc's model-owned path lookup to pass
typed resolved-base packets into
`prebind_treecalc_formula_text_with_resolved_bases(...)`.

For ordered collection syntax, DnaTreeCalc should use OxCalc's public
`treecalc_formula_text_ordered_selector_queries(...)` packets for the authored
spellings `@PRECEDING`, `@FOLLOWING`, `@ANCESTORS`, and recursive `**`
forms, including qualified forms such as `Q2.**` and
`Accounts.2005.**.Margin`. DnaTreeCalc or a future OxCalc resolver supplies
the resolved base and ordered member node ids back through
`TreeCalcOrderedSelectorResolution`; OxCalc then emits the existing
`OrderedSelectorV1` carrier and sparse-reader path.

## 3. Current Evidence

Current OxCalc code shape:

1. `src/oxcalc-core/src/formula.rs` exposes
   `prebind_treecalc_formula_text(owner_node_id, source_text)`.
2. Free-standing `@CHILDREN` and `.*` bind to the formula owner/caller context.
3. `prebind_treecalc_formula_text_with_resolved_bases(...)` admits qualified
   `base.@CHILDREN` and `base.*` only when the caller supplies an exact
   source-span-keyed resolved-base packet with base `TreeNodeId`, base span,
   selector span, resolution layer, and resolution identity.
4. The returned `TreeFormula::opaque_oxfml` source uses neutral
   `TREE_REF_<owner>_<n>` tokens.
5. `TreeCalcChildrenReferenceCollection` preserves exact source token text and
   UTF-8 span from the original formula text.
6. The carrier is `TreeFormulaReferenceCarrier::named` over
   `TreeCalcReferenceCollection::ChildrenV1`.
7. Focused OxCalc tests cover `=SUM(@CHILDREN)`, `=SUM(.*)`,
   `=SUM(base.@CHILDREN)`, `=SUM(base.*)`, unsupported raw syntax diagnostics,
   and end-to-end execution through the existing OxCalc/OxFml/OxFunc path.
8. DnaTreeCalc commit `6611684` activates the matching corpus slice through
   the live OxCalc bridge and preserves ordered dependency projection.
9. `treecalc_formula_text_ordered_selector_queries(...)` exposes family,
   source/base/selector/tail spans, exact token text, optional base token, and
   optional recursive tail token for `@PRECEDING`, `@FOLLOWING`, `@ANCESTORS`,
   and `**`.
10. `prebind_treecalc_formula_text_with_resolved_ordered_selectors(...)`
    admits those ordered selector forms only when supplied with a
    source-span-keyed resolved collection containing base `TreeNodeId` and
    ordered member node ids. It preserves source text/spans on
    `TreeCalcReferenceCollection::OrderedSelectorV1`.
11. The TreeCalc runtime sparse reference-values path dispatches ordered
    selectors to `TreeCalcOrderedSelectorSparseReader`, preserving
    reference-like aggregate execution without eager materialization.

## 4. Interface Implications

DnaTreeCalc integration should:

1. pass the caller/owner `TreeNodeId` and original formula text to OxCalc,
2. for qualified children syntax, pass only an already-resolved stable
   `TreeNodeId` plus exact source/base/selector spans to OxCalc, or leave the
   case pending until OxCalc's explicit path resolver is available,
3. for ordered selectors, use OxCalc's ordered-selector query packets and feed
   back only resolved collection facts: base `TreeNodeId`, ordered member ids,
   resolution layer, and resolution identity,
4. store/use the returned `TreeFormula` in the existing OxCalcTree formula
   catalog path,
5. treat prebind diagnostics as typed host-facing diagnostics,
6. avoid local parsing or string matching for `@CHILDREN`, `.*`, or future
   TreeCalc reference syntax.

## 5. Minimum Invariants

1. DnaTreeCalc does not parse or lower TreeCalc formula references locally.
2. OxCalc remains the owner of TreeCalc model custody, reference resolution,
   dependency/invalidation facts, and source-preserving reference carriers.
3. OxFml remains the owner of formula grammar, call parsing, name/call
   precedence, prepared identity, and evaluator/runtime behavior.
4. OxFunc remains unaware of TreeCalc syntax and consumes only ordinary
   values/arrays or opaque references.

## 6. Open Questions

1. Full DnaTreeCalc W004/W005 corpus activation beyond the first active
   children slice.
2. OxCalc-owned explicit path-resolution for raw base text and
   traversal-membership computation for ordered selector packets. `calc-4vs8.13`
   supplies source-preserving query packets and resolved-collection prebind for
   `@PRECEDING`, `@FOLLOWING`, `@ANCESTORS`, and recursive `**`, but not the
   resolver that computes those member sets or DnaTreeCalc corpus activation.
3. How DnaTreeCalc wants to display typed prebind diagnostics for unsupported
   selectors and structured table references.

## 7. W056 Table Rollout Coordination

OxCalc has filed `HANDOFF-CALC-006` for the cross-repo W056 node-associated
table rollout.

Current OxCalc floor for DnaTreeCalc table activation:

1. table-node snapshots project to virtual Excel table descriptors,
2. table catalog resolution emits stable handles, namespace versions, effective
   table identity, virtual anchors, caller-context facts, and typed diagnostics,
3. structured-reference packets lower into sparse `ReferenceLike` readers for
   table, column, section, current-row, empty-body, and composite selections,
4. lifecycle/update packets classify row/column/header/totals/value/table/node/
   workspace/registry mutations into OxCalc-owned dependency and invalidation
   facts,
5. dynamic table rebind packets classify table, column, section, current-row,
   cross-workspace, renamed/moved/deleted, unavailable, unsupported runtime
   parse, and non-table dynamic selector cases.

DnaTreeCalc should add or extend local W004/W056 beads so the remaining product
activation covers:

1. dynamic table `INDIRECT` and selector-driven table references,
2. cross-workspace table references and unavailable workspace diagnostics,
3. renamed/moved/deleted table targets and same-table selector changes,
4. empty-body retained artifacts and first-row/last-row transitions,
5. lifecycle callback retained artifacts,
6. full namespace/anchor/workspace table pairing for OxReplay intake.

The integration rule remains unchanged: DnaTreeCalc supplies product table
state and host-owned resolution facts through public OxCalc packets. It should
not parse TreeCalc formula text, reconstruct private source-span keys, mirror
OxCalc dependency/invalidation classification, or materialize eager table
values as closure evidence.
