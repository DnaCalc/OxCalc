# W051 Sparse Range Readers And Defined-Entry Semantics

Status: `open_next`

Parent predecessor: `W050` formula-authority rework

Parent epic: allocate when W051 starts.

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

1. `dialect_id` / `capability_profile_id` such as `oxcalc.treecalc-v1`,
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
W051 must treat that as an open cross-repo dependency until OxFml acknowledges
the context, bind-output, and resolution-diagnostic shape.

For TreeCalc reference collections, OxFunc is intentionally reference-opaque:
it should continue to see the same value/reference universe it already uses for
worksheet ranges. A new OxFunc `ReferenceKind` is only acceptable if evidence
shows the existing `ReferenceLike` kind/target space cannot carry an opaque
host reference safely.

## 4. Current Code Inventory

The current investigated floor is:

1. DNA TreeCalc docs already specify `@CHILDREN` / `.*` as set-producing,
   ordered reference collections, explicitly say reference collections are
   references, not arrays, and assign formula parse/bind ownership to OxFml
   through an engine-supplied host context.
   Its live OxCalc bridge currently uses a temporary prepared-formula carrier
   for literal, binary, and direct-node smoke paths; that carrier is not the
   target formula-text parse/bind path.
2. OxCalc `TreeReference` currently has scalar/direct, relative, dynamic, and
   residual carrier variants, but no reference-collection variant. The current
   `SyntheticReferenceBinding` stores one `target_node_id`, and
   `formal_input_bindings_for_runtime` binds every translated reference as
   `DefinedNameBinding::Value(...)`, which scalarizes references before OxFml
   can preserve identity.
3. OxCalc dependency descriptors currently model targetful scalar edges or
   untargeted residual diagnostics. There is no set-membership descriptor for
   "this formula depends on the child collection of node X", no collection
   lowering into current member value edges, and no structural invalidation
   rule for add/remove/reorder of collection members.
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
   bind/formal-reference output shape for `@CHILDREN` / `.*`, with final
   call/name shadowing gated on an Excel oracle matrix for built-ins, UDFs,
   defined names, and defined-name `LAMBDA` invocation,
8. add the OxCalcTree reference-collection carrier/resolution shape that
   consumes that output,
9. add the corresponding dependency/invalidation shape for current member
   value changes, child-membership change, and sibling-order change,
10. choose the opaque `ReferenceLike` representation or record the exact
   blocker if a new kind is unavoidable,
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

Product status: not implemented yet. W051 now explicitly includes the
TreeCalc reference-collection compatibility lane required for
`=SUM(@CHILDREN)` / `=SUM(.*)`.

Evidence: W050 provides prepared runtime identity and formal input/reference
transport. Current code investigation shows OxFml and OxFunc already have
reference-binding/resolver surfaces, while OxCalc still lacks the TreeCalc
collection resolution carrier, set-membership dependency, and
reference-preserving runtime binding. The DNA TreeCalc bridge still uses a
temporary prepared-formula smoke carrier rather than the target
formula-text-to-OxFml path.

Still open: reader contract, OxCalc adapter, TreeCalc collection carrier,
set-membership dependency and invalidation, opaque `ReferenceLike` carrier
choice, OxFml resolver/materialization threading, OxFunc sparse admission
activation, and integration evidence.

Formal status: no proof claim.
