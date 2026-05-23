# W056 TreeCalc Full Reference And Table Lowering

Status: `in_progress`

Parent predecessor: `W051` first reference-carrier pattern

Parent epic: `calc-4vs8`.

Initial successor beads:

1. `calc-4vs8.1` — TreeReference variant inventory and host-reference correlation.
2. `calc-4vs8.2` — structured table dependency lowering.
3. `calc-4vs8.3` — dependency invalidation and dynamic rebind widening.
4. `calc-4vs8.21` through `calc-4vs8.26` — node-associated
   TreeCalc table completion spine: table-node snapshot projection,
   TreeCalc table-path structured-reference prebind, table reference readers,
   per-row column-formula runtime, update/invalidation scenarios, and retained
   evidence closure.

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
4. cross-workspace remains a typed exclusion until a versioned workspace
   availability/degradation model exists; resolved ordered selector packets
   now have carriers via `calc-4vs8.12`, while raw selector parser/resolver
   packets, traversal bounds, and corpus activation remain open.

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

Cross-repo counterpart beads now exist for the same spine:

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

The architecture rule for all of these beads is strict: OxCalc/DnaTreeCalc own
TreeCalc table meaning and structural identity; OxFml owns generic structured
reference parsing/binding; OxFunc owns function semantics over opaque carriers;
OxXlPlay observes Excel; OxReplay compares retained declared payloads. No repo
may close a table bead by parsing another repo's private strings or mirroring
another repo's semantics.

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
`TableDescriptor` requires parseable data-column A1 area refs; W056 table
reader and OxFml packet widening must settle that before full table closure.

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
3. the cross-workspace inventory row now points to
   `NeedsCrossWorkspaceProvider`: the packet/version model exists, but
   execution still requires a workspace provider and alias model.
4. explicit host-path tokens containing `!` remain typed rejects in
   `resolve_treecalc_explicit_host_path_base(...)` until the provider/alias
   model lands.

Current non-claim:

This is not executable cross-workspace reference resolution. It supplies the
packet and identity shape needed to report availability/degradation without
inventing workspace alias semantics, external workspace lookup, or retained
cross-workspace corpus evidence.

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

This does not close W004/W005, table activation, bare host-name/dotted formula
binding, dynamic/cross-workspace references, node-as-function/lambda-valued
nodes, or retained OxReplay evidence.

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

Product status: in progress through `calc-4vs8.20`. W051 is closed for the first
OxCalc `ChildrenV1` carrier pattern; W056 now has a typed Rust
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
to the remaining workspace provider and alias semantics.
OxFml commit `4a050f9` adds W074 evidence that `host_namespace_version`
participates in prepared identity even without explicit host-reference bind
results, narrowing the host namespace mutation invalidation gap without
freezing name/call precedence.
DnaTreeCalc commit `fe678cf` activates the scoped explicit structural
path/traversal raw ordered corpus slice through the public OxCalc resolver path,
narrowing receiving-side corpus activation without closing W004/W005.
The table area now has an explicit completion spine in the bead graph:
`calc-4vs8.21` through `calc-4vs8.26`, with matching DnaTreeCalc, OxFml,
OxFunc, OxXlPlay, and OxReplay beads for persistence/corpus activation, generic
structured-reference packet breadth, opaque function admission, Excel
observation, and retained comparison. Those beads define the remaining work
needed for node-associated TreeCalc tables to behave for OxFml like
Excel-anchored tables without adding TreeCalc semantics to OxFml/OxFunc.
This is not a full-reference/table-lowering product claim.

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

Still open: W074 final name/call precedence evidence beyond the observed
W074-CALC005-014 table-name row, W074 formula-call registry lookup and
cache-invalidation migration, bare host-name and callable host-node precedence,
exercised OxFml host-reference packets beyond the admitted
children/table/ordered-selector slices, cross-workspace provider and workspace
alias/first-position `!` semantics, table-specific path syntax, DnaTreeCalc
receiving-side corpus activation for table packets and the remaining W004/W005
reference suite, retained OxReplay/OxXlPlay evidence, and broader end-to-end
scenarios. Blocker `calc-4vs8.5` remains open for the remaining full-W056
closure scope.

Formal status: no proof claim.
