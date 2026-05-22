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
6. now explicitly commits the host-driven engine-handle interaction direction, with the current one-shot facade as the first implementation-backed slice rather than a separate path to be replaced later.

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
references remain gated on OxFml W074-CALC005 evidence.

`calc-4vs8.3` adds the first typed dependency/rebind projection in
`src/oxcalc-core/src/tree_reference_rebind.rs`. That surface preserves target
reverse edges, context-only reverse edges, descriptor-level invalidation facts,
dynamic potential versus resolved dynamic rebind state, and prepared-identity
inputs for host namespace, structure context, capability profile,
table-context, cross-workspace availability, and caller context. It is a typed
OxCalc runtime input surface, not a full end-to-end W056 product claim.

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

### 4.2 Consumer-facing runtime facade
This is the preferred host-facing entry surface for the TreeCalc-first phase.
The contract direction is a host-held engine handle whose internal state is
OxCalc-owned. The first implemented object set is the one-shot first slice of
that model:
1. `OxCalcTreeEnvironment`
2. `OxCalcTreeDocument`
3. `OxCalcTreeRecalcRequest`
4. `OxCalcTreeRecalcResult`
5. `OxCalcTreeRuntimeFacade`

Current implementation note:
1. this object set now exists in `src/oxcalc-core/src/consumer.rs`,
2. it is currently a thin consumer wrapper over the first local sequential TreeCalc engine,
3. that thinness is intentional for the current phase because it keeps consumer packaging explicit without inventing a second semantic layer,
4. the current `execute(document, request)` call is semantically equivalent to opening a transient handle, loading one document snapshot, running one host-driven recalc, returning the result, and dropping the transient handle.

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

The V1 one-shot facade is the first slice of that model:
1. `OxCalcTreeDocument` is the seed state for a transient handle,
2. `OxCalcTreeRecalcRequest` is the first host-driven run request,
3. `OxCalcTreeRecalcResult` is the stable result/read surface returned from that run,
4. `OxCalcTreeRuntimeFacade::execute` is the current transient-handle operation.

Working rule:
1. hosts should prefer this object set over reaching directly into local proving-floor engine types,
2. OxCalc may evolve richer internals underneath it,
3. persistent handle, edit, step, pin, and close APIs must widen this object set rather than replace it with a callback or service boundary,
4. host-facing packaging should not require hosts to stitch coordinator, dependency, and local runtime internals together by hand.

## 6. OxCalcTree Runtime Contract

### 6.1 OxCalcTreeEnvironment
`OxCalcTreeEnvironment` is the stable host-facing environment object for the current TreeCalc-first phase.

In the current phase it is no longer an empty placeholder.
It carries the first non-narrow consumer inputs needed by TreeCalc-style hosts:
1. selected OxCalc runtime lane,
2. optional host/session identity,
3. host capability snapshot for runtime-derived effect families,
4. runtime policy inputs for diagnostics and overlay projection.

These fields are consumer context, not formula-language semantics or coordinator publication state.
They are projected into deterministic diagnostics so hosts can verify which environment basis was used for a run.
Runtime-derived effect production also receives this environment context, allowing explicit policy such as runtime-effect overlay projection without changing candidate acceptance, reject/no-publish, or coordinator publication authority.

It must not:
1. hide OxFml-owned semantic inputs behind ambient mutable state,
2. collapse candidate/publication distinction,
3. smuggle scheduler or mutation policy in undocumented ways,
4. imply an engine-owned ambient executor or callback channel.

### 6.2 OxCalcTreeDocument
`OxCalcTreeDocument` is the snapshot-oriented input document for one TreeCalc runtime act.

It carries:
1. `StructuralSnapshot`
2. `TreeFormulaCatalog`
3. seeded published values

Working meaning:
1. structural truth is explicit,
2. formula attachment is explicit,
3. host-visible starting publication truth is explicit.

The document object is intentionally explicit because pinned structural truth is foundational in the OxCalc architecture.

### 6.3 OxCalcTreeRecalcRequest
`OxCalcTreeRecalcRequest` is the per-run execution request object.

It carries:
1. `candidate_result_id`
2. `publication_id`
3. `compatibility_basis`
4. `artifact_token_basis`
5. W055 target extension: `cycle_config`

Working meaning:
1. candidate/publication identity is explicit at the host-facing boundary,
2. compatibility and artifact-token basis remain visible to the consumer-facing runtime contract,
3. coordinator-facing correlation is not hidden in ambient runtime state.

W055 cycle-config extension:

1. `cycle_config` is the production field for selecting circular-reference and
   iterative-calculation behavior for a run.
2. absent `cycle_config` means `cycle.non_iterative_stage1`.
3. `cycle_config.cycle_profile_id` admits `cycle.non_iterative_stage1`,
   `cycle.excel_match_iterative`, and `cycle.iterative_deterministic_v0`.
4. `cycle_config.maximum_iterations` and `cycle_config.maximum_change` carry
   host overrides for iterative profiles; absent values use profile defaults.
5. `compatibility_basis` must not be used as the semantic cycle config channel.

### 6.4 OxCalcTreeRecalcResult
`OxCalcTreeRecalcResult` is the canonical host-facing result object for the current TreeCalc-first phase.

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
1. emitted runtime-derived families in the current TreeCalc-first lane must be directly reachable on `OxCalcTreeRecalcResult.runtime_effects`
2. the corresponding overlay projection must be directly reachable on `OxCalcTreeRecalcResult.runtime_effect_overlays`
3. hosts must not be forced to inspect narrower local engine internals just to discover whether the current run emitted `DynamicDependency` or `ExecutionRestriction`
4. admitted but currently unexercised families such as `CapabilitySensitive` or `ShapeTopology` do not need to appear on the host-facing result until the live TreeCalc-first lane emits them as distinct families

Current W026 reachability boundary:
1. the current W026 coordinator-facing consequence floor must remain directly reachable on `OxCalcTreeRecalcResult` through:
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
2. hosts should not reach around `OxCalcTreeRuntimeFacade` and `OxCalcTreeRecalcResult` to depend on proving-floor engine types or packet-companion structs merely because W026 names narrower seam facts beneath this contract
3. future W026 or successor packet widening may require this contract to expose additional facts directly, but it does not authorize a parallel host-facing OxCalc seam layer beside this contract

### 6.5 OxCalcTreeRuntimeFacade
`OxCalcTreeRuntimeFacade` is the ordinary host-facing runtime service.

It supports the first one-shot slice of the engine-handle model:
1. one-shot execution of an `OxCalcTreeDocument` plus `OxCalcTreeRecalcRequest`,
2. a stable environment-plus-request execution model,
3. explicit return of result, publication, reject, diagnostic, and derived-state families,
4. no engine-to-host callbacks,
5. no ambient executor or hidden service lifecycle.

The committed widening direction is:
1. `open(document) -> handle` or equivalent handle construction,
2. typed edit or edit-batch calls against the handle,
3. recalc/step calls against the handle,
4. explicit pin/read calls for stable views,
5. explicit close/drop semantics.

Current scope note:
1. the first implementation covers one-shot execution only,
2. persistent handles, incremental edit calls, version navigation, explicit cancellation, and steppable progress are not implemented in the current facade,
3. those APIs are successor work that must preserve this host-driven/passive interaction shape rather than introduce a separate scheduler or callback mechanism.

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
2. the host-facing persistent handle, edit-to-version map, undo/redo navigation surface, cancellation API, and concurrent read API are not implemented in the current `OxCalcTreeRuntimeFacade`,
3. W054 owns bounded-memory and pinned-epoch retention policy,
4. W053 owns Stage 2 partitioned/concurrent promotion,
5. W051 owns the TreeCalc reference-collection custody lane that depends on this handle model.

### 6.7 System Of Record And Host Sync Contract
System-of-record ownership is split as follows:
1. OxCalc owns the canonical calculation tree structure inside the engine handle,
2. OxCalc owns dependency descriptors, invalidation, publication, pinned views, runtime overlays, and calc-state derived from that structure,
3. DNA TreeCalc owns product workspace state, UI/view state, skin state, persistence policy, command grouping, and edit orchestration,
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
1. explicit environment object,
2. explicit request object,
3. explicit result object,
4. explicit ordinary runtime facade,
5. explicit statement that consumer packaging does not replace deeper semantic ownership.

Current non-equivalence is also intentional:
1. OxFml exposes formula-language runtime and replay facades,
2. OxCalc exposes a host-facing engine/coordinator runtime facade,
3. OxCalc still carries narrower TreeCalc bind/reference residuals because its first serious host target is later in the pipeline than OxFml's current direct runtime facade target.

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
4. current Rust code does not yet implement a persistent host-held `OxCalcTree` engine handle, typed incremental edit API, pending/completion-token state, explicit cancellation, or async/parallel executor injection.

## 9. Scope Boundary For V1
This V1 contract includes:
1. one-shot local sequential TreeCalc runtime execution,
2. explicit document/request/result packaging,
3. explicit coordinator-facing result families,
4. implementation-backed alignment to OxFml V1 runtime/replay intake,
5. the normative interaction direction that one-shot execution is the first slice of a host-held, OxCalc-owned engine-handle model.

This V1 contract does not include:
1. implemented persistent host session or handle lifecycle,
2. implemented full structural-edit host API,
3. implemented version-navigation undo/redo API,
4. implemented pending/completion-token API,
5. implemented cancellation or steppable-progress API,
6. implemented executor injection, Stage 2 partitioning, GPU execution, or async facade,
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
slice is synchronous one-shot local sequential execution through
`OxCalcTreeRuntimeFacade::execute(document, request)`.

Evidence: the Rust facade exists in `src/oxcalc-core/src/consumer.rs`; the
underlying local runtime exists in `src/oxcalc-core/src/treecalc.rs`;
`StructuralSnapshot` successor edits and coordinator candidate/publication/
reject/pin primitives exist in `src/oxcalc-core/src/structural.rs` and
`src/oxcalc-core/src/coordinator.rs`; W051 records OxCalc custody of the
TreeCalc model for reference-collection resolution.

Still open: persistent handle API, typed incremental edit API, edit-to-version
and undo/redo surface, pending/completion-token API, explicit cancellation and
steppable recalc, executor injection, Stage 2 concurrency/GPU/async facade,
end-to-end W051 generic host-context/reference-array execution evidence, W054
retention policy, W053 partitioned concurrency, W056 full TreeCalc
reference/table-lowering scope, and closure of the remaining W026 residual
lanes.

Formal status: no new proof claim. Existing state/snapshot and coordinator
docs define the pinned-reader, no-torn-view, reject-is-no-publish, and
single-publisher obligations that later handle/concurrency work must model and
exercise.
