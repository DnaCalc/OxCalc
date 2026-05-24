# CORE_ENGINE_OXCALCTREE_CONSUMER_INTERFACE_AND_HOST_CONTRACT_V1.md

## 1. Purpose and Status
This document defines the OxCalc-owned tree-runtime consumer contract for hosts that actually consume the OxCalc runtime, with `DNA TreeCalc` as the first target consumer.

Terminology rule:
1. `DNA TreeCalc` is the future separate repo/product and host after `DNA OneCalc`: a large-scale incremental calculation test-bed and product with explicit nodes/names as formula holders and no grid-structure ownership in OxCalc.
2. `OxCalcTree` and the `OxCalcTree*` object names in this document are OxCalc-owned tree-runtime consumer contract/API surfaces for that integration.
3. This document is not the `DNA TreeCalc` product specification; it defines what OxCalc exposes to such a product.
4. Unqualified `TreeCalc` in this document refers to the OxCalc tree-runtime/substrate lane unless explicitly prefixed with `DNA`.

Status:
1. active canonical local consumer-facing contract for the TreeCalc-first phase,
2. intended to do for OxCalc what the OxFml V1 consumer packet now does for OxFml:
   - define one explicit host-facing object set,
   - separate consumer packaging from deeper substrate details,
   - keep narrower seam residuals explicit rather than implicit,
3. implementation-backed at the first local sequential TreeCalc slice,
4. not yet a full product-host API freeze,
5. aligned to the landed OxFml V1 `consumer::runtime` and `consumer::replay` entry surface,
6. now explicitly commits the host-driven direct-context interaction direction as the implementation-backed public shape.

This document is for actual OxCalc runtime consumers.
Hosts that use OxCalc only as seam-reference material and do not consume the OxCalc runtime directly should still start with `CORE_ENGINE_DOWNSTREAM_HOST_SEAM_REFERENCE.md`.

## 2. Why This Exists
OxCalc already has:
1. canonical architecture,
2. canonical coordinator/publication rules,
3. a canonical OxFml seam companion,
4. TreeCalc execution-planning and workset packetization,
5. implementation-backed local runtime code.

What it did not have was one explicit OxCalc-owned tree-runtime contract that a host such as `DNA TreeCalc` could read as the intended OxCalc entry surface.

This keeps the boundary explicit: `DNA TreeCalc` owns product/host concerns,
persistence, handle lifetime, and edit orchestration, while OxCalc owns the
`OxCalcTree` engine contract, the canonical calculation tree model structure,
and the internal tree-substrate/runtime implementation that backs it.

The result was too much spread across:
1. architecture and seam docs,
2. TreeCalc planning docs,
3. narrower packet companions,
4. internal engine types.

This document closes that gap.

## 3. Hard Boundaries
This consumer contract must be read under these non-negotiable boundaries.

### 3.1 OxFml Owns Formula-Language Meaning
OxCalc consumer packaging does not reopen or replace:
1. parse semantics,
2. bind semantics,
3. evaluator artifact meaning,
4. canonical shared evaluator/runtime seam ownership.

Hosts that consume OxCalc must still treat OxFml as authoritative for those meanings.

### 3.2 OxCalc Owns Coordinator and Publication Meaning
This contract is where OxCalc is authoritative.

OxCalc owns:
1. candidate-versus-publication distinction on the engine side,
2. coordinator accept/reject/publication behavior,
3. dependency and invalidation integration,
4. runtime-overlay meaning on the engine side,
5. stable published-view semantics.

### 3.3 Passive Host-Driven Engine Boundary
OxCalc is a passive library at the host boundary.

The host drives every advance through explicit calls:
1. open or construct engine state,
2. apply typed edits or external updates,
3. request recalc or bounded progress,
4. resume pending work with completion data,
5. query stable views,
6. close or drop the handle.

OxCalc must not:
1. own an ambient event loop or background scheduler that progresses between host calls,
2. call back into the host,
3. hide mutable document or scheduler state behind process-global storage,
4. require a particular executor model for correctness,
5. publish derived state outside coordinator-controlled candidate and publication paths.

Future parallel, async, or GPU execution is compatible with this rule only when
the executor is host-supplied or scoped to the host call/step. Any such work
must be joined, suspended, or returned as explicit progress data before control
returns to the host. No engine-owned runtime may keep ticking the model after a
call returns.

### 3.4 Consumer Packaging Does Not Close Narrower TreeCalc Residuals
This contract does not imply closure of the still-open TreeCalc residual lanes:
1. caller-context breadth,
2. bind/reference intake breadth,
3. execution-restriction transport breadth,
4. publication/topology breadth beyond the current local floor.

Those remain explicit `W026` and successor work lanes until exercised evidence closes them.

### 3.5 W051 Reference-Collection Boundary
The W051 TreeCalc reference-collection lane is now an explicit consumer-contract
pressure point.

For a DNA TreeCalc host, a formula such as `=SUM(@CHILDREN)` must be able to
enter OxCalcTree as formula text that OxFml parses and binds through an
OxCalc-supplied host formula context. OxCalc then resolves the resulting
reference collection against the OxCalc-owned tree model rather than relying
on precomputed formula text magic or a host-specific OxFunc call.

Contract requirements:
1. the host-facing document/update path must carry formula text and tree-model
   creation/update inputs for the OxCalc-owned model, not a host-parsed
   expression tree,
2. OxFml must own parse/bind and consume an OxCalc-supplied host formula
   context for reference dialect, host namespace lookup, and caller context,
3. OxFml must produce a reference/formal-input structure that identifies the
   collection selector, caller/base context, source token, and resolution
   layer,
4. OxCalc must own resolution and lowering from that OxFml structure into
   dependency, invalidation, replay, and OxFml runtime input surfaces,
5. current member value changes, child-membership changes, and sibling-order
   changes must be dependency relevant for `@CHILDREN` / `.*`,
6. bare call names must resolve through a declared hierarchy whose
   function/UDF/defined-name/defined-name-`LAMBDA` shadowing behavior matches
   observed Excel before product semantics are frozen; TreeCalc host callable
   references then map onto the closest Excel-defined-name lane or an explicit
   TreeCalc extension, and explicit host-reference syntax must be available for
   collisions,
7. OxFml runtime must be able to preserve reference identity or dereference
   through the supplied OxCalc adapter,
8. OxFunc must remain TreeCalc-opaque and see ordinary values/arrays or an
   opaque `ReferenceLike` plus resolver.

The first W051 host-context ids are:
1. `dialect_id = oxcalc.treecalc-v1`,
2. `capability_profile_id = host-capabilities:treecalc-v1`,
3. `resolution_rule_version = treecalc-host-resolution:v1`.

The first explicit host-reference syntax slice is `@CHILDREN`, `.*`,
`base.@CHILDREN`, and `base.*`, where `base` is either the caller node or an
explicit single base path using ancestor anchors, workspace-root anchors,
workspace selectors, dotted paths, bracket-escaped path segments, or the
first-position `!` sheet separator alias. Source spans are preserved over the
full formula source, and the exact selector token text is retained for replay
and diagnostics.

OxCalc lowers the resulting handle to
`TreeCalcReferenceCollection::ChildrenV1`. The local carrier records the stable
host reference handle, base `TreeNodeId`, source token text/span, opaque
selector, membership version, order version, and `collection` shape hint; the
current member list is derived from the pinned structural snapshot during
lowering. Current-member value edges, child-membership descriptors, and
sibling-order invalidation are OxCalc-owned facts correlated to the OxFml
host-reference handle; they are not OxFml or OxFunc semantics.

Current implementation status: OxCalc has implemented the local `ChildrenV1`
carrier/dependency pattern and preserves the runtime formal input as an opaque
structured `ReferenceLike`. The executable raw formula-text prebind slice
admits free-standing `@CHILDREN` and `.*`, whose base is the formula
owner/caller context, and qualified `base.@CHILDREN` and `base.*` when the
caller supplies an exact UTF-8-span-keyed resolved-base packet. That packet
names the full qualified token span, base span, selector span, stable
`TreeNodeId`, resolution layer, and resolution identity; OxCalc does not infer
the raw `base` token in this entry point. Full OxCalc-owned typed path
resolution over a pinned structural snapshot and explicit path syntax remains
successor W056 scope. The consumer contract is still not product-complete for
DNA TreeCalc formula-text parsing/binding or reference-array formulas until
OxFml's generic host-context path and the OxFml/OxFunc resolver/admission path
are exercised end to end for the broader reference suite.

### 3.6 W056 Reference Inventory Boundary

W056 starts from the W051 `ChildrenV1` carrier pattern and widens the TreeCalc
reference inventory as typed OxCalc-owned implementation input. The active Rust
surface is `TreeReferenceImplementationInput` in
`src/oxcalc-core/src/formula.rs`.

That inventory records, per reference family:
1. whether the family is a current carrier, an admitted implementation input,
   or a typed exclusion,
2. the required correlation back to OxFml host-reference handles or formal
   reference/source-token identity,
3. host namespace, resolution-rule, structure-context, capability-profile,
   table-context, or cross-workspace identity needs,
4. caller-context identity needs such as caller node, ancestor walk, sibling
   position, host runtime context, or table caller region,
5. dependency descriptor facts and invalidation facts that OxCalc owns.

This inventory is implementation input, not a completed full-reference product
claim. Structured table references are explicitly routed to W056 bead
`calc-4vs8.2`; dependency/reverse-edge, dynamic rebind, namespace, and
caller-context widening are routed to `calc-4vs8.3`; bare name/callable
references consume the OxFml W074-CALC005 current mapping rule: TreeCalc host
value names follow the Excel defined-name value lane, TreeCalc lambda-valued
nodes follow the defined-name-`LAMBDA` lane, and built-ins keep the
call-callee frontier unless a future versioned extension is separately
evidenced.

`calc-4vs8.3` adds the first typed dependency/rebind projection in
`src/oxcalc-core/src/tree_reference_rebind.rs`. That surface preserves target
reverse edges, context-only reverse edges, descriptor-level invalidation facts,
dynamic potential versus resolved dynamic rebind state, and prepared-identity
inputs for host namespace, structure context, capability profile,
table-context, cross-workspace availability, and caller context. It is a typed
OxCalc runtime input surface, not a full end-to-end W056 product claim.

### 3.7 W056 Table-Node Projection Boundary

The first W056 table-node contract is now implemented in
`src/oxcalc-core/src/structured_table.rs` as
`TreeCalcTableNodeSnapshot` -> `TreeCalcTableNodeProjection`.

This projection is deliberately split:

1. TreeCalc/OxCalc-specific facts stay in the input/projection record:
   `table_node_id`, display path, canonical path, virtual anchor identity,
   table namespace version, row membership/order versions, column identity
   version, body formula metadata, and totals formula metadata.
2. OxFml receives only a generic Excel-shaped `TableDescriptor` catalog entry:
   stable `table_id`, `table_name`, virtual workbook/sheet refs,
   `table_range_ref`, column ids/names/ordinals/ranges, header/totals presence,
   exact header/totals range refs, and opaque stable row membership/order
   tokens.
3. OxCalc computes a generic `table_context_identity` for OxFml prepared/cache
   invalidation and a separate OxCalc-only `table_invalidation_identity` that
   keeps raw TreeCalc row ids and body/totals formula metadata out of OxFml's
   semantics.

The virtual-anchor contract makes a table node look to OxFml like an Excel
table anchored at an ordinary cell range. It does not introduce
`EvalValue::Table`, does not require OxFml to understand TreeCalc paths, and
does not make OxFunc inspect table selectors. Descriptor-visible row
membership/order identities are opaque tokens; raw row ids remain in the
OxCalc projection identity only. Current executable evidence covers non-empty
data-body tables. Empty data-body tables are explicitly typed as not
representable by the current generic `TableDescriptor` because OxFml's current
range parser expects parseable A1 area refs for column data ranges. That is a
W056 widening target rather than a silent fallback.

### 3.8 W056 Table Custody Boundary

Node-associated table lifecycle is context state owned by OxCalc. Product code
sets, clears, and reads table state through `OxCalcTreeContext`, which stores
the node-associated `TreeCalcTableNodeSnapshot`, derives the canonical/display
path from the current structural snapshot, derives the table namespace and
structure-context versions, and exposes typed `OxCalcTreeTableView` values.
DnaTreeCalc does not construct semantic rebind packets, call table classifiers,
or preserve private table catalog state outside the context.

The direct context table surface covers create/delete, rename/move, body
value/formula edit, row insert/delete/reorder, column insert/delete/reorder/
rename, header text edit, totals toggle/formula edit, table resize, node
rename/move/delete, workspace open/close, workspace alias mutation, function
registry snapshot mutation, save/reopen, and structural rebind as OxCalc-owned
state transitions and typed classification outcomes.

The packet carries the event kind, before/after
`TreeCalcTableLifecycleVersionState` values where the event shape requires
them, context versions, owner node ids, source host-reference handles, and
changed row/column ids. The version state names the stable table/node/row/column
handles plus the virtual workbook/sheet/anchor identities, table context
identity, table invalidation identity, table range/header/totals/data-region
refs, table namespace version, row membership/order versions, and column
identity version. It also carries workspace availability and workspace alias
versions so workspace open/close and alias mutation are replay-visible.
Context versions include the host namespace, structure context, registry
snapshot, resolution-rule, workspace availability, and workspace alias inputs
used for prepared/cache invalidation.

OxCalc exposes table projection, catalog lookup, generic structured-reference
dependency lowering, and dynamic table rebind classification through
`OxCalcTreeContext`. The report data is the only table lifecycle
interpretation product consumers should depend on: changed dependency kinds,
invalidation reasons, prepared-identity inputs, invalidation seed identities,
changed rows/columns, source handles, and typed diagnostics. Stable
`table_node_id` and `table_id` violations remain OxCalc diagnostics because
they indicate a table identity transition that must be represented as a
create/delete or structural rebind, not a host-side semantic workaround.

`StructuredTableDependencyFactKind` is the replay-facing fact inventory for
this contract. It covers table identity, row membership/order/value, column
identity/order, header text/region, data region, totals region/value/formula
metadata, caller row context, omitted-table enclosing context, virtual
anchor/range, workspace availability, and function registry snapshot
dependency. Generic structured-reference lowering emits the facts available
from OxFml's public table packet; OxCalc's table snapshot/projection inventory
supplies the full product facts for replay and invalidation evidence. The
function registry snapshot fact is conditional: it is present when the table
formula path has registered-function dependency evidence and absent for
constant-only or registry-independent table scenarios.

Dynamic references that resolve to node-associated tables use the same
ownership boundary. `TreeCalcDynamicTableRebindRequest` is an OxCalc-owned
classification input for `INDIRECT`-style selector churn, dynamic function
results, volatile re-evaluation, table lifecycle events, current-row targets,
and cross-workspace table targets. The resulting report names table dependency
facts, dynamic dependency activation/release/reclassification, table lifecycle
invalidation reasons, prepared-identity inputs, and typed exclusions. Dynamic
selector identity is an explicit prepared/cache identity input even when a
selector change rebinds to the same physical table. Cross-workspace dynamic
table targets add workspace availability sensitivity without dropping normal
table row, column, and data-region dependencies. OxFml must supply generic
structured-reference bind packets for dynamic table targets; OxCalc does not
ask OxFml to parse TreeCalc syntax at runtime. Unsupported runtime
structured-reference parsing and non-table dynamic targets remain typed
exclusions rather than fallback eager materialization.

OxFml receives none of this lifecycle meaning. After OxCalc classifies the
callback, OxFml only sees the resulting generic table descriptor catalog,
structured-reference packets, caller table region, sparse reference bindings,
and prepared/cache identity tokens. OxFunc receives only opaque references or
ordinary scalar/array values. No consumer should infer TreeCalc table lifecycle
semantics from private strings or from OxFml/OxFunc artifacts.

### 3.9 W056 Structured-Table Function Breadth Boundary

The function breadth boundary is recorded by OxCalc as
`TREECALC_STRUCTURED_TABLE_FUNCTION_ADMISSION_INVENTORY`, while OxFunc remains
the implementation owner for function semantics. The inventory names the
functions that can consume node-associated table references through generic
reference APIs and the functions that need typed host context before product
admission.

The current evidence lane is `SUM`, `COUNT`, `COUNTA`, and `COUNTBLANK` over
sparse `ReferenceLike` table references. The admitted-pending lanes are
shape-only functions (`ROWS`, `COLUMNS`), indexed reference functions
(`INDEX`), ordinary range-scan/statistical/logical/text functions,
lookup/match functions, and criteria aggregate functions. Typed host-context
lanes cover dynamic-array transforms, `SUBTOTAL`/`AGGREGATE`, metadata
functions, volatile reference constructors, and reference operators. `CALL` is
excluded pending native invocation policy.

Every lane carries the same ownership rule: OxCalc supplies generic sparse
bindings, declared extent, coordinate access, multi-range alignment, caller
context, dynamic-array/spill policy, row-hidden/filter state, or metadata
policy as typed generic inputs where needed. OxFunc must not inspect TreeCalc
selectors, and dense eager materialization cannot be used as table-reference
closure evidence.

### 3.10 W056 Whole-System Node-Table Architecture Rule

The controlling W056 node-table map is recorded in
`docs/worksets/W056_TREECALC_FULL_REFERENCE_AND_TABLE_LOWERING.md` Section
4B.3. This host contract consumes that map as normative for the OxCalcTree
boundary.

The architectural rule is:

1. DnaTreeCalc owns product table state, editing, persistence, and corpus
   activation.
2. OxCalc owns calculation custody for node tables: virtual Excel-anchor
   projection, table catalog resolution, dependency facts, invalidation,
   dynamic rebind, caller context, sparse readers, and prepared identity.
3. OxFml owns generic formula and structured-reference parsing/binding against
   `TableDescriptor`, enclosing table, and `caller_table_region` packets only.
4. OxFunc owns function semantics over scalar/array/reference inputs and
   registry mutation. Table references remain opaque `ReferenceLike` or
   reader-backed values.
5. OxXlPlay observes Excel ListObject behavior; OxReplay compares declared
   retained payloads; neither repo defines TreeCalc table semantics.
6. DnaOneCalc ordinary single-formula use remains no-host-reference; future
   VBA/XLL function admission flows through OxFunc/OxFml registry surfaces.

No consumer of this contract may close table behavior by adding a private
adapter, parsing another repo's formula strings, mirroring another repo's
precedence rules, materializing table references eagerly, or asking OxFml/
OxFunc to learn TreeCalc table selectors.

Current resolver realization:

`src/oxcalc-core/src/structured_table.rs` exposes
`resolve_treecalc_table_catalog_reference` as the first concrete node-table
catalog resolver. The resolver consumes OxCalc-owned `TreeCalcTableNodeProjection`
records plus workspace alias/availability and namespace-adjacency facts, then
emits a `TreeCalcTableCatalogResolution` with stable table-reference handle,
opaque selector, resolution layer, shape hint, effective table identity,
virtual anchor identity, caller-context dependency/id, host namespace version,
table namespace version, structure-context version, resolution-rule version,
workspace availability version, and typed diagnostics. Same-node,
omitted-caller-table, current-root, workspace-qualified, unavailable-workspace,
deleted-table, ambiguous-selector, and W074-mapped adjacency cases are
represented explicitly.

This resolver is not an OxFml grammar extension. OxFml still receives only the
generic table catalog/context and structured-reference bind packets; TreeCalc
path, workspace, namespace, and lifecycle facts remain OxCalc-owned.

Current reader realization:

`TreeCalcTableSparseReader` is the OxCalc-owned reader for node-associated
tables. It exposes whole data-body references, selected data columns,
contiguous multi-column ranges, all-column references, `#Headers`, `#Data`,
`#Totals`, `#All`, current-row references, omitted-table current-row
references, empty data-body zero-row references, and single-row tables through
the generic sparse-reader contract. It preserves sparse blanks, defined empty
strings, typed worksheet error cells, row/column order, and a stable
`reader_identity` split between source identity and snapshot identity.

The current fully exercised function lane is `SUM`, `COUNT`, `COUNTA`, and
`COUNTBLANK` over sparse `ReferenceLike` table bindings. Wider range-taking
functions are admitted only through generic reader/context lanes and OxFunc
counterpart beads; non-contiguous column unions and context-sensitive functions
remain typed successor lanes rather than eager-materialized shortcuts.

Current row-context formula realization:

OxCalc evaluates node-table column formulas by reusing one authored formula
text across rows and asking OxFml to parse/bind that text against row-specific
generic table context, not TreeCalc-specific grammar rules. OxCalc consumes the
public `StructuredReferenceBindRecord` packets produced by OxFml to attach
sparse `ReferenceLike` bindings and current-row scalar bindings. Each row
execution carries a `caller_table_region`, a virtual primary locus, and a
caller-context identity. Totals-row formulas use the totals virtual locus;
`#This Row` in a totals context is a typed reject.

For replay and cache invalidation, table formula results expose
`TreeCalcTableFormulaPreparedIdentityFacts`: dialect id, capability profile id,
resolution rule version, host namespace version, table namespace version,
structure context version, table context identity, caller context id, host and
function registry snapshot identities, capability overlay identity, prepared
formula key, dispatch skeleton key, and plan-template key. Row movement, row
insert/delete, table namespace rename, host namespace, structure context,
resolution rule, capability profile, actual OxFunc registry snapshot changes,
and capability overlays are prepared identity inputs; the reusable dispatch
skeleton is not treated as proof that caller/table identity can be ignored.

## 4. Consumer Layers
The intended OxCalc public shape for TreeCalc-style hosts now has two layers.

### 4.1 Canonical engine substrate
This remains the richer internal and assurance-oriented engine surface:
1. structural snapshots and edits,
2. formula catalogs and local translation support,
3. dependency/invalidation substrate,
4. coordinator and recalc state,
5. replay/evidence emission helpers,
6. narrower seam-consumption details that are not yet stabilized as host-facing contract.

### 4.2 Consumer-facing direct context
This is the only host-facing entry surface for the TreeCalc-first phase.
The contract direction is a host-held direct context whose internal state is
OxCalc-owned. The implemented object set is:
1. `OxCalcTreeContextOptions`
2. `OxCalcTreeContext`
3. `OxCalcTreeCalculationOutcome`

Current implementation note:
1. this object set now exists in `src/oxcalc-core/src/consumer.rs`,
2. it owns one or more workspaces, canonical nodes, formula text, edit state,
   publication state, and stable views,
3. `recalculate` drives the local sequential TreeCalc engine through
   OxCalc-owned private inputs,
4. no host-facing snapshot/request wrapper or prepared-carrier adapter is part
   of the public contract.

## 5. Primary Consumer Contract
The stable OxCalc tree-runtime consumer direction is an explicit engine handle
plus host-driven calls against that handle.

The handle model means:
1. the host owns the handle lifetime,
2. OxCalc owns the handle's internal structure, dependency graph, publication state, pins, and runtime overlays,
3. host edits enter as typed calls or edit batches rather than direct mutation of OxCalc internals,
4. every accepted structural edit creates a new pinned structural version or a typed rejection,
5. recalc, F9, external-value/RTD updates, and future async completions are all explicit synchronous calls into OxCalc,
6. stable reads observe a published version or an explicitly pinned view.

The V1 direct context is the first slice of that model:
1. `OxCalcTreeContext` is the host-held engine context,
2. workspace and node mutation methods are typed calls against that context,
3. `OxCalcTreeContext::recalculate` is the host-driven run operation,
4. `OxCalcTreeCalculationOutcome` is the stable result/read surface returned
   from that run.

Working rule:
1. hosts should prefer this object set over reaching directly into local proving-floor engine types,
2. OxCalc may evolve richer internals underneath it,
3. edit, step, pin, and close APIs must widen this object set rather than replace it with a callback or service boundary,
4. host-facing packaging should not require hosts to stitch coordinator, dependency, and local runtime internals together by hand.

## 6. OxCalcTree Runtime Contract

### 6.1 OxCalcTreeContextOptions
`OxCalcTreeContextOptions` is the stable host-facing configuration object for the current TreeCalc-first phase.

In the current phase it is no longer an empty placeholder.
It carries the first non-narrow consumer inputs needed by TreeCalc-style hosts:
1. selected OxCalc runtime lane,
2. optional host/session identity,
3. host capability snapshot for runtime-derived effect families,
4. runtime policy inputs for diagnostics and overlay projection.

These fields are consumer context, not formula-language semantics or coordinator publication state.
They are projected into deterministic diagnostics so hosts can verify which
context basis was used for a run. Runtime-derived effect production also
receives this context, allowing explicit policy such as runtime-effect overlay
projection without changing candidate acceptance, reject/no-publish, or
coordinator publication authority.

It must not:
1. hide OxFml-owned semantic inputs behind ambient mutable state,
2. collapse candidate/publication distinction,
3. smuggle scheduler or mutation policy in undocumented ways,
4. imply an engine-owned ambient executor or callback channel.

### 6.2 OxCalcTreeContext
`OxCalcTreeContext` is the canonical host-facing TreeCalc engine context.

It owns:
1. one or more workspaces,
2. canonical node ids, symbols, formula text, and structural relations,
3. OxCalc-owned formula binding/catalog construction,
4. seeded and last-published values,
5. the latest calculation outcome used by read views.

Working meaning:
1. hosts mutate structure and formula text through typed context calls,
2. hosts read stable node/workspace views from OxCalc,
3. hosts do not construct formula catalogs or reference carriers,
4. raw formula text is interpreted by OxCalc/OxFml through the TreeCalc host
   context.

### 6.3 Context-owned recalc configuration
Per-run identity and correlation are OxCalc-owned context state.

Working meaning:
1. candidate/publication identity is derived by OxCalc from workspace/context
   versions,
2. artifact-token and compatibility bases remain internal LocalTreeCalc engine
   facts,
3. hosts configure recalc behavior through typed context options or future
   context mutation APIs, not through a separate request DTO.

W055 cycle-config extension:

1. cycle configuration is context-owned typed run configuration for selecting
   circular-reference and iterative-calculation behavior.
2. absent cycle configuration means `cycle.non_iterative_stage1`.
3. the cycle profile id admits `cycle.non_iterative_stage1`,
   `cycle.excel_match_iterative`, and `cycle.iterative_deterministic_v0`.
4. maximum-iteration and maximum-change settings carry
   host overrides for iterative profiles; absent values use profile defaults.
5. internal compatibility basis strings must not be used as the semantic cycle
   configuration channel.

### 6.4 OxCalcTreeCalculationOutcome
`OxCalcTreeCalculationOutcome` is the canonical host-facing result object for the current TreeCalc-first phase.

It returns:
1. run state:
   - `Published`
   - `VerifiedClean`
   - `Rejected`
2. dependency graph
3. invalidation closure
4. evaluation order
5. runtime effects
6. runtime-effect overlays
7. optional accepted candidate result
8. optional publication bundle
9. optional reject detail
10. published values
11. node states
12. diagnostics
13. W055 target extension: cycle diagnostics

It must preserve:
1. candidate versus publication distinction,
2. reject-is-no-publish behavior,
3. replay-visible runtime-derived effects, including explicit runtime-effect family classification where the current engine can distinguish dynamic-dependency versus execution-restriction truth,
4. explicit diagnostics rather than opaque success or failure.

Anticipated async/completion extension:
1. a future run state such as `Pending` or `AwaitingCompletion` is accepted as the contract direction for async function, RTD, streaming, or externally completed work,
2. pending state must return completion descriptors as data, including opaque completion tokens, affected node or work ids, candidate/version basis, and resume requirements,
3. OxCalc must not block, spawn an ambient task, or call the host to complete that work,
4. the host resumes by making another explicit synchronous call against the same handle or pinned version,
5. pending state is not accepted publication and must not leak uncommitted candidate results into stable reads.

W055 cycle-diagnostics extension:

1. `cycle_diagnostics` is the production result field for circular-reference
   and iterative-calculation outcomes.
2. each record identifies the cycle region, selected profile, region source,
   members, root/report node when available, member order, terminal state,
   publication decision, reject kind, and iteration trace summary.
3. string diagnostics may mirror these facts, but hosts should not parse string
   diagnostics to understand cycle behavior.
4. non-iterative cycle rejection must expose a typed `Worksheet.CircularReference`
   equivalent when available.

Current direct reachability rule:
1. emitted runtime-derived families in the current TreeCalc-first lane must be directly reachable on `OxCalcTreeCalculationOutcome.runtime_effects`
2. the corresponding overlay projection must be directly reachable on `OxCalcTreeCalculationOutcome.runtime_effect_overlays`
3. hosts must not be forced to inspect narrower local engine internals just to discover whether the current run emitted `DynamicDependency` or `ExecutionRestriction`
4. admitted but currently unexercised families such as `CapabilitySensitive` or `ShapeTopology` do not need to appear on the host-facing result until the live TreeCalc-first lane emits them as distinct families

Current W026 reachability boundary:
1. the current W026 coordinator-facing consequence floor must remain directly reachable on `OxCalcTreeCalculationOutcome` through:
   - `run_state`
   - `runtime_effects`
   - `runtime_effect_overlays`
   - `candidate_result`
   - `publication_bundle`
   - `reject_detail`
   - `dependency_graph`
   - `invalidation_closure`
   - `evaluation_order`
   - `published_values`
   - `diagnostics`
2. this direct host-facing boundary is required because W026 now treats runtime-derived family reachability, candidate/publication/reject distinction, no-publish rejection, and the first publication-consequence split as consumed-now host-visible truth rather than as replay-only or implementation-local detail
3. narrower W026 seam facts may remain below the host-facing contract for now, including:
   - per-formula identity and compatibility carriage
   - caller-context carriage
   - structural invalidation seeds and rebind-versus-recalc lowering
   - dependency-descriptor mapping
   - residual-carrier lowering and other internal TreeCalc preparation details
4. hosts may consume emitted replay artifacts for evidence and diagnosis, but the contract in this document is still the primary host-facing OxCalc surface for the current TreeCalc-first phase

No second seam layer rule:
1. W026 is a consumed-seam packet that explains what this host-facing contract must preserve; it is not a second host API that hosts should bind to independently
2. hosts should not reach around `OxCalcTreeContext` and `OxCalcTreeCalculationOutcome` to depend on proving-floor engine types or packet-companion structs merely because W026 names narrower seam facts beneath this contract
3. future W026 or successor packet widening may require this contract to expose additional facts directly, but it does not authorize a parallel host-facing OxCalc seam layer beside this contract

### 6.5 Context Operations
`OxCalcTreeContext` exposes ordinary host-facing operations:
1. create a workspace,
2. add, rename, move, reorder, delete, and edit nodes,
3. set, clear, resolve, lower, classify, and read node-associated tables,
4. recalculate a workspace,
5. read workspace, node, and table views,
6. configure host capability/runtime policy through context options.

The committed widening direction is additive context operations:
1. edit-batch calls,
2. recalc/step calls,
3. explicit pin/read calls for stable views,
4. explicit close/drop semantics where needed by a host.

Current scope note:
1. the first implementation covers direct synchronous local sequential recalc,
2. version navigation, explicit cancellation, and steppable progress are
   successor context operations,
3. those APIs must preserve this host-driven/passive interaction shape rather
   than introduce a separate scheduler or callback mechanism.

### 6.6 Version, Cancellation, And Concurrent-Read Contract Direction
The existing architecture and implementation already contain the core substrate
for versioned structural truth and no-publish rejection:
1. `StructuralSnapshot` is immutable and `StructuralEdit::apply_edit` creates a successor snapshot with a new `StructuralSnapshotId`,
2. coordinator state separates candidate, accepted candidate, publication, reject detail, and pinned publication views,
3. a rejected candidate publishes no stable state,
4. pinned readers observe a stable structural/publication view while later work proceeds.

The contract direction built on that substrate is:
1. incremental edits produce named structural versions and invalidation consequences,
2. undo/redo is version navigation over retained engine versions, not a host-forged inverse edit,
3. cancellation abandons the in-flight candidate or progress state and preserves the last stable publication,
4. safe read-during-recalc is through immutable published or pinned views,
5. retention of older versions and derived artifacts is governed by bounded-memory and pinned-epoch rules rather than by host mutation.

Current implementation boundary:
1. structural snapshot edits and coordinator pin/reject primitives exist in Rust,
2. the host-facing persistent handle, edit-to-version map, undo/redo navigation surface, cancellation API, and concurrent read API are not implemented in the current `OxCalcTreeContext`,
3. W054 owns bounded-memory and pinned-epoch retention policy,
4. W053 owns Stage 2 partitioned/concurrent promotion,
5. W051 owns the TreeCalc reference-collection custody lane that depends on this handle model.

### 6.7 System Of Record And Host Sync Contract
System-of-record ownership is split as follows:
1. OxCalc owns the canonical calculation tree structure inside the engine handle,
2. OxCalc owns canonical node formula text, node-associated table state, dependency descriptors, invalidation, publication, pinned views, runtime overlays, and calc-state derived from that structure,
3. DNA TreeCalc owns UI/view state, skin state, edit buffers, file/workflow commands, command grouping, and edit orchestration,
4. OxFml owns formula parse/bind/evaluator meanings consumed through the OxCalc-supplied host formula context,
5. OxFunc owns worksheet value/function semantics.

The host-to-engine sync contract is:
1. the host sends document seeds, typed edits, edit batches, external value updates, recalc requests, completion tokens, and pin/read requests into OxCalc,
2. OxCalc returns version ids, edit impacts, invalidation consequences, run state, pending descriptors, publications, rejects, diagnostics, and stable read views as data,
3. a host-side mirror is permitted only as a projection or cache for rendering/persistence; it is reconciled by OxCalc version and publication ids,
4. if the host mirror disagrees with the engine-held calculation structure for engine-visible facts, the engine-held version is authoritative for calculation,
5. persistent serialization may include host product data plus OxCalc-exported structure/version state, but it must not require the host to mutate OxCalc internals directly.

## 7. Relationship To OxFml V1
The OxCalc consumer contract is intentionally shaped to align with the OxFml V1 approach.

Current alignment is:
1. explicit context options,
2. explicit host-held context,
3. explicit calculation outcome,
4. explicit statement that consumer packaging does not replace deeper semantic ownership.

Current non-equivalence is also intentional:
1. OxFml exposes formula-language runtime and replay entrypoints,
2. OxCalc exposes a host-facing engine/coordinator context,
3. OxCalc still carries narrower TreeCalc bind/reference residuals because its first serious host target is later in the pipeline than OxFml's current direct runtime target.

## 8. Current Implementation Reality
The current implementation-backed object set lives in:
1. `src/oxcalc-core/src/consumer.rs`

The current underlying local runtime remains:
1. `src/oxcalc-core/src/treecalc.rs`

The current OxFml-facing deterministic host packet that feeds the first local slice remains:
1. `src/oxcalc-core/src/upstream_host.rs`

Current interpretation rule:
1. ordinary TreeCalc-style hosts should reason about OxCalc consumption through the consumer contract in this document,
2. implementation-backed packet companions remain valid supporting detail,
3. narrower seam-intake planning docs remain supporting or temporary material rather than host-facing contract,
4. current Rust code implements the first persistent host-held `OxCalcTreeContext`
   with direct workspace/node edit and recalc calls; pending/completion-token
   state, explicit cancellation, and async/parallel executor injection remain
   successor context operations.

## 9. Scope Boundary For V1
This V1 contract includes:
1. direct local sequential TreeCalc runtime execution through `OxCalcTreeContext`,
2. explicit context/options/outcome packaging,
3. explicit coordinator-facing result families,
4. implementation-backed alignment to OxFml V1 runtime/replay intake,
5. the normative interaction direction that direct context execution is the first slice of a host-held, OxCalc-owned engine-handle model.

This V1 contract does not include:
1. complete host session lifecycle beyond the current in-memory context,
2. full structural-edit host API beyond the current node operations,
3. implemented version-navigation undo/redo API,
4. implemented pending/completion-token API,
5. implemented cancellation or steppable-progress API,
6. implemented executor injection, Stage 2 partitioning, GPU execution, or async execution,
7. full product-host integration policy,
8. closure of W026 residuals,
9. W051 reference-collection implementation and evidence.

## 10. Reading Order
For an actual OxCalc runtime consumer such as `DNA TreeCalc`, the intended reading order is:
1. `README.md`
2. `CHARTER.md`
3. `OPERATIONS.md`
4. `docs/WORKSET_REGISTER.md`
5. `docs/BEADS.md`
6. `docs/IN_PROGRESS_FEATURE_WORKLIST.md`
7. `docs/spec/README.md`
8. `CORE_ENGINE_ARCHITECTURE.md`
9. `CORE_ENGINE_COORDINATOR_AND_PUBLICATION.md`
10. this document
11. `CORE_ENGINE_OXFML_SEAM.md`
12. `CORE_ENGINE_TREECALC_SEMANTIC_COMPLETION_PLAN.md`

Use `CORE_ENGINE_OXFML_MINIMAL_UPSTREAM_HOST_INTERFACES.md` only as an implementation-backed packet companion after the consumer contract is understood.

## 11. Status
Product status: OxCalc commits the host-driven engine-handle interaction shape
as the contract direction for OxCalcTree. The current implemented user-visible
slice is synchronous local sequential execution through
`OxCalcTreeContext::recalculate(workspace_id)` after typed context edits.

Evidence: the Rust context exists in `src/oxcalc-core/src/consumer.rs`; the
underlying local runtime exists in `src/oxcalc-core/src/treecalc.rs`;
`StructuralSnapshot` successor edits and coordinator candidate/publication/
reject/pin primitives exist in `src/oxcalc-core/src/structural.rs` and
`src/oxcalc-core/src/coordinator.rs`; W051 records OxCalc custody of the
TreeCalc model for reference-collection resolution. `OxCalcTreeContext` also
owns the first direct node-table lifecycle surface: `set_node_table`,
`clear_node_table`, `table_view`, `workspace_table_views`,
`table_context_packet`, `resolve_table_reference`, `lower_table_reference`,
`lower_table_bind_record`, and `classify_dynamic_table_rebind`.

Still open: edit-to-version and undo/redo surface, pending/completion-token
API, explicit cancellation and steppable recalc, executor injection, Stage 2
concurrency/GPU/async execution, end-to-end W051 generic host-context/
reference-array execution evidence, W054 retention policy, W053 partitioned
concurrency, DnaTreeCalc migration off its bridge modules, full W056
reference/table-lowering product closure, and closure of the remaining W026
residual lanes.

Formal status: no new proof claim. Existing state/snapshot and coordinator
docs define the pinned-reader, no-torn-view, reject-is-no-publish, and
single-publisher obligations that later handle/concurrency work must model and
exercise.
