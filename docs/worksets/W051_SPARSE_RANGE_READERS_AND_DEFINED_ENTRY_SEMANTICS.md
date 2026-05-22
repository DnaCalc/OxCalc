# W051 Sparse Range Readers And Defined-Entry Semantics

Status: `open_next`

Parent predecessor: `W050` formula-authority rework

Parent epic: `calc-hkj9`.

## 1. Purpose

W051 adds Excel-compatible sparse range reading for large worksheet ranges and
the shared reference-reader abstraction needed by TreeCalc reference
collections.

The primary target is ordinary formulas over whole-column and large-area
worksheet references without dense materialization. The TreeCalc compatibility
target is ordered reference collections such as DNA TreeCalc `.@CHILDREN` /
`.*`, where a host formula such as `=SUM(@CHILDREN)` is formula text parsed
and bound by OxFml using an OxCalc-supplied host formula context, then
resolved and lowered by OxCalc against the OxCalc-owned tree model before
OxFunc sees ordinary values or a reference-like carrier.

This is not a generic rich-data or virtual-array workset. TreeCalc reference
collections are in scope only because they are reference-like formula inputs:
they must behave like a non-grid range/reference carrier with deterministic
iteration, blank/defined semantics, dependency identity, and replay-visible
dereference behavior.

The reader surface is:

1. `declared_extent`,
2. `defined_cardinality`,
3. `defined_iter`,
4. `read_at(coord) -> Defined(EvalValue) | Blank`,
5. `contains(coord)`.

## 2. Product Scope

In scope:

1. worksheet range/reference reads,
2. whole-column and large-area formulas,
3. defined entry versus blank behavior,
4. empty string as `Defined(Text(""))`, not blank,
5. deterministic defined-entry iteration order,
6. replay of declared extent and the defined-entry stream,
7. first aggregation functions: `SUM`, `COUNT`, `COUNTA`, and `COUNTBLANK`,
8. ordered TreeCalc reference collections as reference-like inputs, first
   `@CHILDREN` / `.*` over regular child nodes in sibling order,
9. reference-preserving and value/iterator materialization paths for those
   inputs, selected by the downstream function and evaluator profile rather
   than by OxFunc understanding TreeCalc syntax.

Out of scope:

1. generic virtual arrays,
2. arbitrary queryable host objects,
3. non-Excel rich producer protocols,
4. custom pushdown/filter engines,
5. per-function TreeCalc special cases inside OxFunc,
6. broad TreeCalc set algebra beyond the first reference-collection carriers
   named in this workset.

Those ideas stay in
`docs/spec/core-engine/CORE_ENGINE_FUTURE_IDEAS_RICH_AND_VIRTUAL_DATA.md`.

### 2.1 TreeCalc Reference-Collection Scenario

The required product scenario is:

1. DNA TreeCalc owns the host UI, persistence/orchestration, and user editing
   surface. It loads, creates, and updates the OxCalc-owned tree model through
   the OxCalcTree contract. It does not own tree-model semantics, parse formula
   text, or lower `@CHILDREN` itself.
2. OxFml owns generic formula-text parsing and binding, including non-empty
   literal classification and formulas such as `=SUM(@CHILDREN)`. It must be
   extensible by an OxCalc-supplied host formula context that recognizes
   TreeCalc reference syntax such as `@CHILDREN` / `.*` and returns generic
   host-reference/formal-reference structures for OxCalc to consume.
3. OxCalc consumes the OxFml bind/reference structures and resolves the
   collection against stable structural truth in its TreeCalc support model:
   base node, selector kind, child-membership version, deterministic sibling
   order, and explicit exclusion of meta-effective children for the first
   `@CHILDREN` scope.
   The dependency shape must cover both the collection membership/order and
   the current member value reads.
4. OxCalc supplies the corresponding runtime formal input/reference binding
   back through the OxFml runtime seam. Any template hole, defined-name token,
   or formal-reference handle is an OxFml bind/runtime artifact, not a private
   OxCalc parser rewrite of the formula source.
5. OxFml carries the formal input as a reference binding when reference
   identity is needed, or dereferences/materializes it through the supplied
   OxCalc adapter when the active function/profile requests values.
6. OxFunc receives either ordinary values/arrays or `CallArgValue::Reference`
   / `EvalValue::Reference(ReferenceLike)` plus a resolver. It must not parse
   TreeCalc text, inspect tree structure, or gain TreeCalc-specific function
   branches for `SUM(@CHILDREN)`.
7. Dereferencing the carrier pulls values in the right order from the
   OxCalc-owned tree model and value store, populated by DNA TreeCalc host
   create/update operations, through OxFml's evaluation call and into OxFunc's
   existing reference-resolution machinery.

The first expected behavior is that `=SUM(@CHILDREN)` over numeric child values
matches `SUM(A1:B5)` in the ways that matter for aggregate reference inputs:
number cells contribute, reference-derived text/logicals are ignored for `SUM`,
errors propagate, blanks are blanks, empty strings are defined text, and
iteration order is deterministic and replay-visible.

For the canonical closure path, `SUM(@CHILDREN)` should exercise the same
reference/resolver class as worksheet ranges, not a TreeCalc-specific OxFunc
branch. A pre-materialized value-array path may be used as a scoped stepping
stone only if it is labeled as such and does not claim closure for
reference-preserving behavior.

## 3. Ownership

OxCalc owns the concrete sparse reader and backing adapter.

OxFml owns generic formula parsing/binding, sparse binding through semantic
plan, runtime prepared identity, and replay projection.

OxFunc owns sparse argument-preparation metadata and function-kernel
consumption.

DNA TreeCalc owns the host/product surface: UI, workspace persistence,
orchestration, and edit requests. OxFml owns generic formula parse/bind and the
generic extension points for host reference/name dialects. OxCalc owns the
TreeCalc host formula context, custody of the tree model structure, reference
resolution, dependency descriptors, invalidation, and dereference adapter that
make OxFml's reference structures executable.

### 3.1 Host Formula Context And Namespace Resolution

The preferred W051 direction is a generic OxCalc-to-OxFml host formula context,
not TreeCalc syntax hardcoded into OxFml.

The context supplied by OxCalc to OxFml should include:

1. `dialect_id = oxcalc.treecalc-v1` and
   `capability_profile_id = host-capabilities:treecalc-v1`,
2. a host reference parser hook for reference positions and callee-prefix
   positions, returning opaque host-reference syntax nodes with source spans,
3. a host namespace resolver for node names, defined names, relative paths,
   set-producing selectors, and host-sensitive references,
4. a function registry view that includes built-in OxFunc functions and
   registered UDF functions,
5. caller context sufficient to bind lexical walk-up and relative references
   without OxFml owning the tree model.

The bind hierarchy should be stable and replay-visible, but W051 does not
freeze the final name-shadowing order yet. Final precedence for built-in
functions, registered UDFs, workbook/sheet defined names, and defined-name
`LAMBDA` invocation must be matched to observed Excel behavior before OxFml
promotes product semantics. TreeCalc node names and lambda-valued nodes should
then be mapped onto the closest Excel-defined-name category or given an
explicit, documented extension point.

The current planning shape is:

1. OxFml reserved syntax and lexical scopes keep their existing meaning
   (`LET`, `LAMBDA`, lambda parameters, and other OxFml-owned special forms).
2. Bare call position is resolved by the Excel-matched function/UDF/
   defined-name-LAMBDA precedence that OxFml records from oracle cases.
3. If the Excel-matched precedence leaves room for host callable names, OxFml
   asks the OxCalc host namespace whether the callee token/path resolves to a
   callable reference, such as a node whose value is a `Lambda`.
4. Explicit host-reference syntax or explicit paths bypass the bare-function
   ambiguity and bind through the OxCalc host namespace.
5. In non-call value/reference position, bare names bind through the host
   namespace, unless an Excel-defined rule says a function/UDF symbol is
   meaningful in that non-call position.
6. Ambiguities must be reported as bind diagnostics with the chosen resolution
   path and available explicit-disambiguation syntax.

This preserves OxFml as the formula parser/binder while keeping function
semantics with OxFunc and tree-name/reference semantics with OxCalc.

The OxFml-side request is filed as
`docs/handoffs/HANDOFF_CALC_005_OXFML_HOST_CONTEXT_AND_NAMESPACE_RESOLUTION.md`.
OxFml receipt
`../OxFml/docs/handoffs/HANDOFF_CALC_005_OXFML_RECEIPT.md` accepts this as
`accept_as_w051_plan_with_w074_evidence_gate`. W051 therefore owns the
OxCalc-side context and resolver shape below; W074 remains the evidence gate
for final Excel name/call precedence.

For TreeCalc reference collections, OxFunc is intentionally reference-opaque:
it should continue to see the same value/reference universe it already uses for
worksheet ranges. A new OxFunc `ReferenceKind` is only acceptable if evidence
shows the existing `ReferenceLike` kind/target space cannot carry an opaque
host reference safely.

### 3.2 CALC-005 Receipt Intake: W051 Implementation Inputs

This section is the OxCalc-side plan needed before W051 implementation starts.
It answers the shape requests in the OxFml receipt without asking OxFml to
hardcode TreeCalc syntax.

Stable context ids:

1. `dialect_id`: `oxcalc.treecalc-v1`
2. `capability_profile_id`: `host-capabilities:treecalc-v1`
3. `resolution_rule_version`: `treecalc-host-resolution:v1`
4. strict-Excel rejection profile for TreeCalc-only syntax:
   `host-capabilities:strict-excel`

`dialect_id` versions the host reference syntax admitted by the hook.
`capability_profile_id` versions the enabled host capability bundle. They are
separate prepared-identity inputs because a future profile can disable or
enable features without changing the syntax grammar, and a future dialect can
change syntax while preserving a broader capability bundle.

First explicit host-reference syntax that enters the OxFml host hook:

1. free-standing self selector: `@CHILDREN`, canonicalized to
   `caller.@CHILDREN`,
2. free-standing children sugar: `.*`, canonicalized to
   `caller.@CHILDREN`,
3. postfix selector on a resolved single base: `base.@CHILDREN`,
4. postfix children sugar on a resolved single base: `base.*`,
5. explicit base paths for the first selector scope:
   - ancestor anchors: `^`, `^.Name`, `^^.Name`, and deeper `^` stacks,
   - workspace-root anchors: `[]Name`, `[][Escaped Name]`,
   - workspace selectors: `[workspace]Name` and `['quoted path']Name`,
   - dotted paths: `Name.Child`, `Name.[Escaped Child]`,
   - first-position sheet separator alias: `Sheet1!Name`.

A bare single identifier with no anchor, selector, or path separator is not an
explicit host-reference bypass. It stays in the name/call precedence lane that
W074 must settle. For the first W051 closure, only the children collection
selector is in scope; `@ANCESTORS`, `@PRECEDING`, `@FOLLOWING`, recursive
descent `**`, structured table references, and cross-workspace value loading
remain outside the first collection carrier unless a successor slice names
them explicitly.

Source preservation rule:

1. OxFml source spans remain over `FormulaSourceRecord.formula_text`, including
   the leading `=` when present.
2. The host hook must return `source_span_utf8` as `[start_byte, end_byte)`,
   `source_token_text` as the exact source substring, and
   `source_token_kind` as `TreeCalcPath`, `TreeCalcChildrenAccessor`, or
   `TreeCalcChildrenSugar`.
3. Bracket escaping, quoting, case, and separator spelling are preserved in
   `source_token_text`; canonical selector fields are separate.
4. Replay output must preserve the full host reference source substring plus
   per-token spans for the base path and the selector token when both exist.

Host namespace resolver output shape:

1. `host_ref_handle`: stable OxCalc-owned handle. For the first collection
   carrier, the stable form is equivalent to
   `treecalc-hostref:v1:children:<base_node_id>`; serialization may be an
   interned id, but the identity inputs are dialect, selector kind, base
   `TreeNodeId`, and resolution rule version.
2. `opaque_selector`: OxCalc-owned payload with schema
   `oxcalc.treecalc.host_selector.v1`. For the first carrier it contains
   `selector_kind = Children`, `base_node_id`, `include_meta = false`,
   `order = sibling_index`, and an internal selector fingerprint. OxFml stores
   or echoes the payload but does not inspect it.
3. `resolution_layer`: one of `explicit_host_ref`, `host_path`,
   `host_name_defined_name_like`, `function`, `defined_name`, `lexical`, or
   `unresolved`. W051 uses `explicit_host_ref` or `host_path` for the explicit
   syntax above; bare names and callees stay evidence-gated by W074.
4. `shape_hint`: `single`, `collection`, `dynamic`, or `unknown`.
   `@CHILDREN` and `.*` return `collection`.
5. `caller_context_dependency`: `none`, `caller_node`, `ancestor_walk`,
   `workspace_selector`, or `active_selection`. Free-standing `@CHILDREN` and
   `.*` are `caller_node`; explicit absolute workspace-root selectors are
   `none` after their base is resolved.
6. `caller_context_id`: required whenever `caller_context_dependency` is not
   `none`. The first identity is
   `treecalc-caller:v1:<structure_context_version>:<caller_node_id>:<formula_slot_id-or-none>`.
7. `diagnostics`: typed records, not prose-only strings. First W051 diagnostic
   codes are `UnresolvedHostName`, `AmbiguousHostName`, `CapabilityDenied`,
   `UnsupportedSelector`, `NonSingleBaseForSelector`,
   `SetReferenceUsedAsCallable`, `ExternalWorkspaceUnavailable`,
   `MetaNodeInvisible`, and `NameCollisionRequiresExplicitSyntax`.

Prepared identity and cache invalidation inputs:

1. `host_namespace_version`: OxCalc-issued version over bind-visible TreeCalc
   namespace truth. It changes on node create/delete, rename, move, sibling
   order changes that affect walk-up or path lookup, meta-visibility changes,
   workspace alias changes, and any host namespace rule change.
2. `structure_context_version`: existing structural snapshot/version identity
   consumed by the host/runtime packet.
3. `caller_context_id`: defined above; it changes when the caller node,
   formula slot, or structural context that makes a relative or self selector
   meaningful changes.
4. `registry_snapshot_identity`: OxFunc/OxFml function registry view identity,
   included where function/name classification can affect bind results.
5. `resolution_rule_version`: `treecalc-host-resolution:v1`, included so a
   future resolver rule change invalidates prepared identities deterministically.

First TreeCalc reference-collection carrier:

1. carrier name: `TreeCalcReferenceCollection::ChildrenV1`,
2. host-hook carrier fields: `host_ref_handle`, `base_node_id`,
   `source_span_utf8`, `source_token_text`, `opaque_selector`, and
   `shape_hint = collection`,
3. OxCalc lowering emits typed `TreeReferenceCollectionDependency` facts with
   `membership_version`, `order_version`, and `member_node_ids` derived from
   the structural member snapshot, not parsed back out of trace text,
4. membership: regular children only, excluding meta-effective children,
5. order: ascending persisted `sibling_index`,
6. empty result: valid empty reference collection, not `#REF!`,
7. base deleted or no longer resolvable: typed `#REF!`/unresolved diagnostic
   through the resolver path.

W051's target transport is reference-preserving: expose this carrier through
`ReferenceLike` plus an OxCalc resolver/reader. The first adapter may encode
the handle as an opaque target such as
`ReferenceLike { kind: Structured, target: "treecalc-hostref:v1:<host_ref_handle>" }`
until OxFunc and OxFml need a dedicated host-reference kind. That encoding is
only a transport label; OxFunc must not inspect the TreeCalc selector. If the
current OxFml/OxFunc argument-preparation metadata cannot pass that reference
for the first function group, W051 may use a labeled fallback named
`treecalc_eager_values_fallback.v1`, but that fallback is not closure evidence
for the reference-preserving scenario.

Set-membership dependency and invalidation facts owned by OxCalc:

1. `TreeCalcSetMembershipDependency { owner_node_id, host_ref_handle,
   selector_kind = Children, base_node_id, membership_version, order_version }`
   is emitted by OxCalc and correlated to the OxFml host-reference handle.
2. Current member value dependencies are separate value edges from the formula
   owner to each `member_node_id`.
3. Child add/remove, child meta-visibility changes, and base deletion
   invalidate membership.
4. Sibling reorder invalidates order even if membership is unchanged.
5. Member value publication invalidates the value edge without changing
   membership.
6. Base rename/move invalidates host namespace and may require rebind for
   path-resolved references; a handle already bound to stable `base_node_id`
   remains correlated for diagnostics until rebind decides otherwise.
7. OxFml semantics stop at the host-reference handle and opaque selector.
   Member lists, child filtering, ordering, membership versions, and structural
   invalidation are OxCalc semantics and replay facts.

W074-blocked decisions:

1. bare call precedence across built-ins, registered UDFs, workbook/sheet
   defined names, and defined-name `LAMBDA`,
2. non-call bare-name behavior when the same text can be a function, UDF,
   defined name, or host name,
3. TreeCalc lambda-valued node invocation beyond mapping it to the closest
   Excel defined-name `LAMBDA` lane,
4. cache invalidation after registry mutation, defined-name mutation,
   capability overlay denial, and host namespace mutation where those facts
   change name/call classification.

## 4. Current Code Inventory

The current investigated floor is:

1. DNA TreeCalc docs already specify `@CHILDREN` / `.*` as set-producing,
   ordered reference collections, explicitly say reference collections are
   references, not arrays, and assign formula parse/bind ownership to OxFml
   through an engine-supplied host context.
   Its live OxCalc bridge currently uses a temporary prepared-formula carrier
   for literal, binary, and direct-node smoke paths; that carrier is not the
   target formula-text parse/bind path.
2. OxCalc `TreeReference` now includes the first reference-collection carrier
   pattern:
   `TreeCalcReferenceCollection::ChildrenV1(TreeCalcChildrenReferenceCollection)`.
   The carrier records the stable host reference handle, base node, source
   token text/span, opaque selector, membership version, and order version.
   Runtime preparation now keeps this carrier as
   `DefinedNameBinding::Reference(ReferenceLike { kind: Structured, target:
   treecalc-hostref:v1:children:<base_node_id> })` instead of scalarizing it to
   a value.
3. OxCalc dependency descriptors now include collection-membership and
   collection-member-value facts for `ChildrenV1`. Membership descriptors are
   correlated to the OxFml host-reference handle; member value descriptors
   create ordinary reverse edges from each current member node to the formula
   owner. Local invalidation facts distinguish child membership changes from
   sibling-order-only changes.
4. OxFml already has `RuntimeFormalInputBinding` with
   `DefinedNameBinding::Reference(ReferenceLike)`, and its ordinary function
   path can preserve references when the function metadata requests
   `RefsVisibleInAdapter`. For values-only calls, reference materialization is
   currently handled through the local resolver path.
5. OxFunc already has `ReferenceLike`, `CallArgValue::Reference`,
   `EvalValue::Reference`, and `ReferenceResolver`. `SUM` already exercises a
   resolver-backed reference path for aggregate inputs, including multi-area
   and reference-derived array cases. This is the reason W051 should avoid
   OxFunc TreeCalc-specific changes unless the opaque carrier kind proves
   insufficient.
6. `SUM` currently advertises a values-only argument-preparation profile even
   though its aggregate adapter can consume references. W051 must decide the
   first-function-group admission shape that lets OxFml pass a
   `ReferenceLike`/reader through when needed, or explicitly mark any eager
   value-array materialization as a temporary fallback.

## 5. First Work

The first W051 beads should:

1. write the shared reader contract,
2. decide the coordinate and extent types,
3. decide the replay columns/artifacts,
4. implement an OxCalc reader over the current value/structure store or a
   clearly labeled fixture adapter,
5. activate the first OxFunc function group,
6. thread sparse input through OxFml runtime/replay,
7. lock the generic OxCalc-to-OxFml host formula context, including function
   registry lookup, host namespace lookup, reference parser hooks, and
   bind/formal-reference output shape for `@CHILDREN` / `.*`, using
   `dialect_id = oxcalc.treecalc-v1`,
   `capability_profile_id = host-capabilities:treecalc-v1`, and
   `resolution_rule_version = treecalc-host-resolution:v1`; final call/name
   shadowing remains gated on the W074 Excel oracle matrix for built-ins,
   UDFs, defined names, and defined-name `LAMBDA` invocation,
8. add the OxCalcTree reference-collection carrier/resolution shape that
   consumes that output,
9. add the corresponding dependency/invalidation shape for current member
   value changes, child-membership change, and sibling-order change,
10. attempt the opaque `ReferenceLike` plus OxCalc resolver/reader carrier for
    `TreeCalcReferenceCollection::ChildrenV1`; any eager value-array path must
    be labeled `treecalc_eager_values_fallback.v1` and treated as non-closing
    for reference-preserving behavior,
11. thread a reference-binding and resolver/iterator materialization path
    through OxFml without requiring OxFunc to understand TreeCalc reference
    text,
12. run integration scenarios over at least one large worksheet range and one
    TreeCalc `SUM(@CHILDREN)` equivalent.

## 6. Evidence

W051 evidence must show:

1. declared extent,
2. defined-entry stream,
3. deterministic iteration order,
4. blank versus empty-string behavior,
5. final formula result,
6. no dense traversal for the covered large-range path,
7. for TreeCalc reference collections: OxFml parse/bind identity, source
   selector, base node, ordered member list, dependency descriptors,
   dereference trace, and final function result,
8. a check that OxFunc received either ordinary values/arrays or an opaque
   `ReferenceLike`, not TreeCalc syntax or tree-structure objects.

Suggested artifact root:

`docs/test-runs/core-engine/w051-sparse-ranges/`

The first rollout bead may refine that root before emitting checked evidence.

## 7. Closure Gate

W051 can close for its first product scope when:

1. the sparse reader API is specified and implemented in OxCalc,
2. OxFml can carry sparse inputs through runtime and replay for the declared
   scope,
3. OxFunc consumes the reader for the first function group,
4. OxCalcTree can carry at least the `@CHILDREN` / `.*` reference collection
   scenario through OxCalc/OxFml/OxFunc without OxFunc TreeCalc-specific logic,
   and can demonstrate the reference-preserving/resolver-backed path or name a
   remaining exact blocker,
5. TreeCalc/core and replay evidence match the declared scenarios,
6. dense materialization is ruled out by a check or counter for the large-range
   path and any TreeCalc value materialization is either bounded by the
   scenario or replaced by a reference/iterator path,
7. exclusions are listed plainly.

## 8. Status

Product status: the OxCalc-local first reference-carrier pattern is
implemented for `TreeCalcReferenceCollection::ChildrenV1`. W051 now explicitly
includes the TreeCalc reference-collection compatibility lane required for
`=SUM(@CHILDREN)` / `=SUM(.*)`. The OxFml receipt for `HANDOFF-CALC-005`
has been processed into the plan: OxCalc supplies
`dialect_id = oxcalc.treecalc-v1`,
`capability_profile_id = host-capabilities:treecalc-v1`, explicit
`@CHILDREN` / `.*` host-reference source preservation, the
`TreeCalcReferenceCollection::ChildrenV1` carrier, and OxCalc-owned
membership/value invalidation facts.

Evidence: W050 provides prepared runtime identity and formal input/reference
transport. Current code now adds `ChildrenV1` to the OxCalc formula substrate,
lowers it into collection-membership plus current-member value descriptors,
preserves the runtime formal input as an opaque structured `ReferenceLike`, and
emits local invalidation reasons for membership versus order changes. Focused
checks:
`cargo test -p oxcalc-core children_collection -- --nocapture`. The
HANDOFF-CALC-005 receipt accepts the generic host-context direction and routes
final name/call precedence evidence to W074. The DNA TreeCalc bridge still
uses a temporary prepared-formula smoke carrier rather than the target
formula-text-to-OxFml path.

Still open: sparse worksheet reader contract, OxCalc sparse adapter
implementation, OxFml resolver/materialization threading for the public
runtime path, OxFunc sparse/reference admission activation for the first
function group, end-to-end `SUM(@CHILDREN)` evidence through generic
`HostFormulaContext`, and the W074-CALC005 Excel oracle matrix for
built-in/UDF/defined-name/defined-name-`LAMBDA` shadowing. The `ReferenceLike`
plus resolver/reader path is the W051 target; eager materialization is only a
labeled fallback if metadata blocks that target. Full TreeCalc reference
families and structured table lowering are registered in successor W056 rather
than being hidden inside this first carrier pattern.

Formal status: no proof claim.
