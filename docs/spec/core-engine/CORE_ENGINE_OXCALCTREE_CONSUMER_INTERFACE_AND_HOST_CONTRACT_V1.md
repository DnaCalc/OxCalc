# CORE_ENGINE_OXCALCTREE_CONSUMER_INTERFACE_AND_HOST_CONTRACT_V1.md

## 1. Purpose and Status
This document defines the OxCalc-owned tree-runtime consumer contract for hosts that actually consume the OxCalc runtime, with `DNA TreeCalc` as the first target consumer.

Status:
1. active canonical local consumer-facing contract for the TreeCalc-first phase,
2. intended to do for OxCalc what the OxFml V1 consumer packet now does for OxFml:
   - define one explicit host-facing object set,
   - separate consumer packaging from deeper substrate details,
   - keep narrower seam residuals explicit rather than implicit,
3. implementation-backed at the first local sequential TreeCalc slice,
4. not yet a full product-host API freeze,
5. aligned to the landed OxFml V1 `consumer::runtime` and `consumer::replay` entry surface.

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

The result was too much spread across:
1. architecture and seam docs,
2. TreeCalc planning docs,
3. narrower packet companions,
4. internal engine types.

This document closes that gap.

## 3. Hard Boundaries
This consumer contract must be read under three non-negotiable boundaries.

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

### 3.3 Consumer Packaging Does Not Close Narrower TreeCalc Residuals
This contract does not imply closure of the still-open TreeCalc residual lanes:
1. caller-context breadth,
2. bind/reference intake breadth,
3. execution-restriction transport breadth,
4. publication/topology breadth beyond the current local floor.

Those remain explicit `W026` and successor work lanes until exercised evidence closes them.

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
This is the preferred host-facing entry surface for the TreeCalc-first phase:
1. `OxCalcTreeEnvironment`
2. `OxCalcTreeDocument`
3. `OxCalcTreeRecalcRequest`
4. `OxCalcTreeRecalcResult`
5. `OxCalcTreeRuntimeFacade`

Current implementation note:
1. this object set now exists in `src/oxcalc-core/src/consumer.rs`,
2. it is currently a thin consumer wrapper over the first local sequential TreeCalc engine,
3. that thinness is intentional for the current phase because it keeps consumer packaging explicit without inventing a second semantic layer.

## 5. Primary Consumer Contract
The first stable OxCalc tree-runtime consumer contract is:
1. an explicit environment object,
2. an explicit immutable or snapshot-oriented document object,
3. an explicit per-run request object,
4. an explicit canonical run result object,
5. one ordinary runtime facade that executes the run.

Working rule:
1. hosts should prefer this object set over reaching directly into local proving-floor engine types,
2. OxCalc may evolve richer internals underneath it,
3. but host-facing packaging should not require hosts to stitch coordinator, dependency, and local runtime internals together by hand.

## 6. OxCalcTree Runtime Contract

### 6.1 OxCalcTreeEnvironment
`OxCalcTreeEnvironment` is the stable host-facing environment object for the current TreeCalc-first phase.

In the current phase it is intentionally narrow.
It represents:
1. the selected OxCalc runtime lane,
2. the consumer-facing execution boundary,
3. the point where later policy or session widening can attach without rewriting host packaging.

It must not:
1. hide OxFml-owned semantic inputs behind ambient mutable state,
2. collapse candidate/publication distinction,
3. smuggle scheduler or mutation policy in undocumented ways.

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

Working meaning:
1. candidate/publication identity is explicit at the host-facing boundary,
2. compatibility and artifact-token basis remain visible to the consumer-facing runtime contract,
3. coordinator-facing correlation is not hidden in ambient runtime state.

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

It must preserve:
1. candidate versus publication distinction,
2. reject-is-no-publish behavior,
3. replay-visible runtime-derived effects, including explicit runtime-effect family classification where the current engine can distinguish dynamic-dependency versus execution-restriction truth,
4. explicit diagnostics rather than opaque success or failure.

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

It should support:
1. one-shot execution of an `OxCalcTreeDocument` plus `OxCalcTreeRecalcRequest`,
2. a stable environment-plus-request execution model,
3. later widening toward richer host/runtime lifecycle packaging without forcing host-side rewrites.

Current scope note:
1. the first implementation covers one-shot execution only,
2. later session, incremental, or driven-host packaging is a later bounded widening lane rather than implied current scope.

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
3. narrower seam-intake planning docs remain supporting or temporary material rather than host-facing contract.

## 9. Scope Boundary For V1
This V1 contract includes:
1. one-shot local sequential TreeCalc runtime execution,
2. explicit document/request/result packaging,
3. explicit coordinator-facing result families,
4. implementation-backed alignment to OxFml V1 runtime/replay intake.

This V1 contract does not include:
1. full host session lifecycle,
2. full structural-edit host API,
3. full product-host integration policy,
4. closure of W026 residuals,
5. later Stage 2 or concurrency-facing host packaging.

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
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - current consumer packaging is one-shot execution only
  - broader TreeCalc bind/reference intake and W026 residuals remain open
  - later host lifecycle and richer consumer-facing mutation/session surfaces are not yet defined here
  - current implementation is local-sequential TreeCalc-first scope, not a broader product-host freeze
