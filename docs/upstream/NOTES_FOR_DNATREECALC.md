# Notes for DnaTreeCalc

Status: `active`
Owner lane: `OxCalc`
Relationship: outbound observation and integration note for DnaTreeCalc
TreeCalc formula-text intake

## 1. Purpose

Record the corrected boundary for DnaTreeCalc consumption. Earlier notes that
described OxCalc raw formula-text recognition or a "host-reference bind" surface are
superseded. Those APIs are migration debt under `calc-4vs8.33.4`, not the
intended integration surface.

## 2. Core Message

DnaTreeCalc should use `OxCalcTreeContext` directly: create workspaces, add
nodes/tables, set formula text, call recalculation, and render OxCalc views.
DnaTreeCalc must not parse, query, rewrite, or lower TreeCalc formula syntax.

The corrected formula route is:

1. DnaTreeCalc stores/edits user formula text through OxCalc context APIs.
2. OxCalc supplies `HostFormulaContext` to OxFml with dialect/profile ids,
   namespace versions, caller/table context identities, and declarative
   host-reference syntax rules.
3. OxFml parses and binds the formula, including TreeCalc-enabled host syntax
   only through generic host hooks.
4. OxFml calls OxCalc resolver surfaces with source-preserving packets.
5. OxCalc resolves those packets against canonical workspace/node/table state
   and returns carriers/readers, dependency facts, invalidation facts, and
   typed diagnostics.

## 3. Current Evidence

Current corrected evidence:

1. OxCalc `OxCalcTreeContext` owns direct workspace/node/table custody.
2. OxCalc host contexts now carry declarative TreeCalc host-reference syntax
   rules for the first collection families: `@CHILDREN`, `.*`,
   `@PRECEDING`, `@FOLLOWING`, `@ANCESTORS`, and recursive `**`.
3. OxFml has the first generic runtime host-reference syntax-rule hook and
   emits source-preserving syntax matches without TreeCalc topology semantics.
4. The older OxCalc formula-text and table structured-reference recognizers are
   explicitly deprecated as boundary defects while the remaining tests and
   corpus activation are migrated.

## 4. Interface Implications

DnaTreeCalc integration should:

1. construct and mutate `OxCalcTreeContext` directly,
2. submit formula text only as formula text, never as parsed reference packets,
3. read values, diagnostics, dependency evidence, and table views from OxCalc
   context views/results,
4. assert typed pending/exclusion outcomes from OxCalc for families not yet
   admitted by the OxFml/OxCalc host-hook path,
5. avoid local parsing, string matching, or callback-specific adapters for
   `@CHILDREN`, `.*`, ordered selectors, recursive selectors, reference
   literals, dynamic references, or structured table references.

## 5. Minimum Invariants

1. DnaTreeCalc does not parse or lower TreeCalc formula references locally.
2. OxCalc remains the owner of TreeCalc model custody, reference resolution,
   dependency/invalidation facts, and source-preserving resolver outputs.
3. OxFml remains the owner of formula grammar, call parsing, name/call
   precedence, lexical scope, prepared identity, and evaluator/runtime behavior.
4. OxFunc remains unaware of TreeCalc syntax and consumes only ordinary
   values/arrays or opaque references.

## 6. Open Questions

1. DnaTreeCalc active W004/W005 corpus activation must be reissued through the
   corrected direct-context plus OxFml host-hook route.
2. OxFml must widen host syntax parsing from the first declarative literal
   hook to base/tail-aware host-reference and structured-reference packets.
3. OxCalc must delete the deprecated recognizers and expose only resolver
   surfaces that receive OxFml packets, not formula strings.

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
