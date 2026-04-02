# CORE_ENGINE_TREECALC_SEMANTIC_COMPLETION_PLAN.md

## 1. Purpose and Status
This document defines the design and execution-planning target for the first semantically-complete TreeCalc engine phase.

Status:
1. active planning companion,
2. intended canonical bridge between the current `TraceCalc` proving substrate and the first TreeCalc-ready engine,
3. sequential and semantics-first in immediate realization scope,
4. explicitly pre-optimization, but written to preserve the intended high-performance architecture path.

This document exists because the current realized OxCalc floor proves important coordinator, recalc, replay, and retained-witness machinery, but it does not yet realize a true TreeCalc formula engine.

The purpose here is to make the next target explicit:
1. what the first TreeCalc-ready engine actually is,
2. what it must do end-to-end,
3. what the OxCalc/OxFml seam must provide,
4. what work sequence gives real line of sight to that target,
5. what must remain true so later optimization waves do not require semantic redesign.

Classification: **supporting-companion** per `CORE_ENGINE_DOWNSTREAM_HOST_SEAM_REFERENCE.md` Section 4.1.

For downstream hosts that use OxCalc as seam-reference material only, this document is a supporting consumer-model companion.
Read `CORE_ENGINE_DOWNSTREAM_HOST_SEAM_REFERENCE.md` first (the single entry point and authority filter for downstream hosts), then `CORE_ENGINE_OXFML_SEAM.md`, then use this document to understand how the consumed OxFml seam is expected to participate in the first TreeCalc-ready execution pipeline.
This document does not define seam authority — it describes how OxCalc intends to consume the seam that OxFml owns.

For actual runtime consumers such as `DNA TreeCalc`, read `CORE_ENGINE_OXCALCTREE_CONSUMER_INTERFACE_AND_HOST_CONTRACT_V1.md` first.
This document then serves as the TreeCalc-first execution, widening, and residual-planning companion for that consumer contract.

## 2. Target Outcome
The target defined by this document is:

**the first semantically-complete, sequential, TreeCalc-ready core engine**

For this repo, that means an engine that can:
1. hold a tree-structured calculation substrate of named nodes,
2. attach real formula artifacts to those nodes,
3. consume OxFml-owned parse and bind products rather than test-only scripted evaluation steps,
4. resolve direct and relative references in TreeCalc scope through the OxFml seam plus OxCalc structural truth,
5. build the structural dependency graph and runtime-derived dependency consequences required for recalculation,
6. execute deterministic evaluation and candidate-result intake,
7. apply coordinator acceptance, rejection, and publication rules,
8. preserve pinned-reader and stable-publication semantics,
9. emit deterministic replay, diff, explain, and witness artifacts for the covered TreeCalc scope.

The target does **not** yet require:
1. default parallel or async execution,
2. economics-tuned incremental strategy,
3. grid substrate semantics,
4. broader Excel surface beyond the declared TreeCalc node-and-reference scope,
5. pack-grade replay promotion beyond the already-declared replay capability floor.

## 3. Definition Of "Semantically-Complete" For This Phase
For this phase, "semantically-complete" does not mean "all future optimization and substrate work is finished."

It means the following semantic chain is real and exercised end-to-end:
1. TreeCalc structure exists as immutable structural truth.
2. Each calculation node can own a real OxFml formula artifact package.
3. Reference meaning is explicit for the covered TreeCalc forms.
4. Structural dependency and runtime-derived dependency facts are explicit and replay-visible.
5. Evaluator-produced candidate results are real seam objects, not scenario-script placeholders.
6. Coordinator publication is the only path to stable observer-visible state.
7. Reject-is-no-publish behavior is preserved across the covered formula and reference families.
8. Dynamic/runtime-derived facts that matter to semantics are not hidden in mutable implementation detail.
9. The covered TreeCalc corpus is executable through the actual engine pipeline, not only through `TraceCalc` fixture scripts.

This phase is therefore complete only when the first TreeCalc-ready engine exists as an **actual runtime pipeline**, not just as spec text or as a proving harness.

## 4. What Exists Now And What Does Not

### 4.1 What Exists Now
OxCalc already has meaningful executable foundations:
1. immutable structural snapshots and projection-path lookup,
2. sequential candidate/reject/publish coordinator logic,
3. invalidation and overlay lifecycle state,
4. planner-driven DAG/SCC handling in the `TraceCalc` proving lane,
5. deterministic replay, diff, explain, retained-witness, and replay-appliance artifact emission,
6. a first live OxFml V1 consumer-facade intake for the deterministic upstream-host runtime/replay slice through `consumer::runtime` and `consumer::replay`.

These are real assets and should be preserved.

### 4.2 What Does Not Exist Yet
The following are still absent from the live TreeCalc engine path:
1. real node-bound formula artifact ownership as the driver of evaluation,
2. real OxFml bind products as the driver of reference meaning,
3. automatic dependency-graph build from formulas and bind facts,
4. actual evaluator-produced candidate results as the active execution path for the broader TreeCalc dependency-driven scope beyond the current first local slice,
5. real tree-relative and direct-node reference support beyond the test-only `TraceCalc` calc-space,
6. end-to-end structure -> formula -> bind -> dependency -> evaluation -> publication execution over the real engine substrate.

The work sequence below exists to close exactly that gap.

## 5. TreeCalc Engine Scope For This Phase

### 5.1 Structural Substrate
The first TreeCalc-ready engine phase assumes:
1. a tree-structured node substrate,
2. named nodes with stable IDs,
3. optional containment hierarchy used for projection and relative-reference context,
4. no grid substrate,
5. no hidden grid assumptions in semantic rules.

### 5.2 Node Kinds
The minimum TreeCalc node taxonomy for this phase should cover:
1. root/container nodes,
2. calculation nodes with formulas,
3. constant/value nodes,
4. structural grouping nodes where needed for relative-reference context,
5. explicit reserve room for later host-only or synthetic nodes without letting them become semantic smuggling channels.

### 5.3 Reference Families In Scope
The first TreeCalc-ready engine should cover at least:
1. direct named-node references,
2. tree-relative references based on explicit relative-navigation semantics,
3. static multi-dependency formulas,
4. conditional dependency selection where the effective edge set depends on runtime facts,
5. explicit direct-binding-sensitive families where semantic truth depends on concrete identity,
6. typed unsupported or out-of-scope reference paths as explicit no-publish or unsupported outcomes.

### 5.4 Reference Families Out Of Scope
Not required for this first phase:
1. grid-address references,
2. full Excel workbook-sheet-range semantics,
3. broad host-query families beyond the declared TreeCalc/OxFml seam floor,
4. advanced spilling/grid occupancy semantics outside the already bounded shape-effect categories.

## 6. End-To-End Semantic Pipeline To Realize
The first TreeCalc-ready engine must make the following pipeline real:

1. **Structural intake**
   - create or update immutable TreeCalc structural snapshots
   - preserve stable node identity
   - preserve explicit projection context for relative-reference interpretation

2. **Formula artifact attachment**
   - each formula-bearing node points to an immutable OxFml-owned formula artifact package
   - formula artifact identity participates in snapshot/version/fence discipline

3. **Bind and reference meaning intake**
   - OxCalc consumes OxFml bind products and reference descriptors
   - OxCalc does not reinterpret formula grammar locally
   - OxCalc does own how bind products participate in dependency, invalidation, and publication

4. **Structural dependency derivation**
   - build the static dependency graph derivable from structure plus bind facts
   - isolate cycle regions explicitly
   - keep dependency additions/removals/reclassifications explicit where runtime discovery changes the effective graph

5. **Invalidation and work discovery**
   - derive stale or needed work from structural edits, upstream publication, or runtime-derived dependency changes
   - keep the invalidation state model explicit

6. **Evaluation and candidate-result production**
   - invoke OxFml-backed evaluator work
   - produce real `AcceptedCandidateResult` seam objects
   - preserve typed reject and no-publish outcomes

7. **Overlay and runtime-derived fact handling**
   - dynamic dependencies, capability-sensitive effects, format-sensitive effects, execution restrictions, and shape effects are handled as explicit runtime-derived state
   - no hidden mutable caches may carry semantic truth

8. **Coordinator accept/reject/publication**
   - apply snapshot/fence/token/profile/capability compatibility checks
   - accept or reject candidate work deterministically
   - publish accepted results atomically

9. **Observer-visible stabilization**
   - preserve stable published views
   - preserve pinned-reader views
   - preserve replay-visible distinction between candidate, reject, and commit

10. **Replay and assurance emission**
   - emit trace, diff, explain, witness, and retained-failure artifacts for the covered TreeCalc scope
   - keep the ordinary engine path and assurance surfaces aligned

## 7. Design Constraints For The First TreeCalc-Ready Engine

### 7.1 No Hidden Structural Mutation
Dependency truth derived from formulas and bind artifacts must never be maintained only as mutable runtime state.

The engine may:
1. cache,
2. overlay,
3. retain derived data,
4. preserve runtime-observed edges explicitly.

The engine may not:
1. smuggle structural dependency truth into mutable side tables,
2. treat replay-only artifacts as the true engine state,
3. let runtime convenience replace versioned structural truth.

### 7.2 OxFml Owns Parse/Bind/Evaluator Meaning
OxCalc must not drift into owning formula-language semantics.

OxCalc owns:
1. structural truth,
2. dependency integration,
3. invalidation policy,
4. coordinator policy,
5. publication semantics,
6. replay and assurance binding for the engine.

OxFml owns:
1. parse and bind semantics,
2. evaluator artifact meaning,
3. typed execution and reject contexts,
4. replay-safe identity and fence meaning where evaluator/runtime artifacts are canonical.

### 7.3 Sequential First, But Concurrency-Compatible
The first TreeCalc-ready engine is sequential.

But every design choice in this phase must preserve:
1. single-publisher coordinator authority,
2. immutable structural truth,
3. explicit runtime-derived state,
4. replay-visible candidate/reject/publication distinction,
5. future Stage 2 concurrency without semantic replacement.

### 7.4 No Test-Only Semantics In The Real Engine Path
`TraceCalc` remains valuable as:
1. self-contained corpus,
2. executable semantic oracle substrate,
3. retained-witness and replay proving surface.

But the first TreeCalc-ready engine phase is not satisfied by improving `TraceCalc` alone.
It must move actual engine execution onto real formula/bind/candidate-result flows.

## 8. Engine Surfaces To Realize In OxCalc

### 8.0 Consumer-Facing Runtime Surface
The intended host-facing OxCalc runtime surface for this phase is now packetized separately in `CORE_ENGINE_OXCALCTREE_CONSUMER_INTERFACE_AND_HOST_CONTRACT_V1.md`.

Current working rule:
1. hosts should consume OxCalc through that explicit environment/document/request/result/facade object set,
2. this document explains how the underlying TreeCalc-first engine must widen beneath that host-facing contract,
3. narrower seam and dependency-preparation details may remain below the consumer contract until later widening closes them honestly.

### 8.1 Structural Model
OxCalc needs a richer structural model than the current proving-floor root-with-children shape.

This phase should explicitly realize:
1. stable node identity,
2. node symbol and projection identity,
3. parent/child and sibling relation enough for relative-reference context,
4. formula-artifact attachment at node level,
5. structural edit operations that respin immutable snapshots.

### 8.2 Formula Attachment Model
Each formula-bearing node should carry, directly or via explicit structural payload:
1. formula artifact reference,
2. bind-product reference,
3. stable formula artifact identity,
4. compatibility and version handles required for coordinator and replay.

### 8.3 Dependency Model
OxCalc should explicitly realize:
1. static dependency edges derived from bind facts,
2. reverse dependencies,
3. explicit cycle region representation,
4. runtime-derived dependency change objects,
5. dependency identity stable enough for replay and witness reduction.

### 8.4 Invalidation Model
The first TreeCalc-ready phase must carry the already-declared invalidation vocabulary into the real engine path:
1. `clean`
2. `dirty_pending`
3. `needed`
4. `evaluating`
5. `verified_clean`
6. `publish_ready`
7. `rejected_pending_repair`
8. `cycle_blocked`

### 8.5 Candidate Intake Model
The live engine path must consume real seam-produced candidate results, not only local synthetic values.

### 8.6 Publication Model
The current coordinator model should remain authoritative, but it must now be driven by real TreeCalc evaluator outputs.

### 8.7 Overlay Model
The overlay layer must move from proving-floor support to actual TreeCalc runtime relevance:
1. dynamic dependency overlay,
2. invalidation/execution-state overlay,
3. capability/fence attachment overlay,
4. format-sensitive and execution-restriction-sensitive runtime attachments where semantically relevant.

## 9. OxFml Seam Requirements For This Phase
This phase depends on a narrower, more explicit consumed seam than the current proving floor.

The goal is not to reopen seam ownership broadly.
The goal is to define the exact consumed floor needed for the first real TreeCalc engine path.

### 9.1 Required Seam Inputs
OxCalc must be able to consume, per formula-bearing node or candidate-result family:
1. formula artifact identity,
2. bind-product identity,
3. snapshot and fence compatibility basis,
4. profile/version basis,
5. candidate-result and reject identities,
6. dependency consequence facts,
7. runtime-derived effect facts,
8. publication-consequence categories.

### 9.2 Identity And Fence Floor
The consumed identity/fence floor for this phase should include:
1. `formula_stable_id`
2. `formula_token`
3. `snapshot_epoch`
4. `bind_hash`
5. `profile_version`
6. `capability_view_key` as important consumed compatibility state
7. `candidate_result_id`
8. `commit_attempt_id` where present
9. `reject_record_id` where relevant
10. optional `fence_snapshot_ref` where present

### 9.3 Bind And Reference Meaning Floor
OxCalc needs OxFml to surface, for the covered TreeCalc scope:
1. static direct references,
2. relative-reference descriptors or already-bound relative targets,
3. typed unresolved or host-query-sensitive references,
4. dependency additions/removals/reclassifications as evaluator/runtime facts,
5. dynamic selection families where the effective runtime dependency set is not fully static.

### 9.4 Candidate Result Floor
The live TreeCalc engine path must consume candidate-result categories aligned with:
1. `value_delta`
2. `shape_delta`
3. `topology_delta`
4. optional `format_delta`
5. optional `display_delta`
6. spill or shape event families where present
7. surfaced evaluator/runtime facts that matter to coordinator behavior

### 9.5 Reject Context Floor
The first TreeCalc-ready engine should rely on typed reject contexts covering at least:
1. snapshot mismatch
2. artifact/token mismatch
3. profile mismatch
4. capability denial
5. publication-fence mismatch
6. execution restriction or invalid phase outcome
7. dynamic dependency failure
8. host-sensitive resolution failure where relevant

### 9.6 Runtime-Derived Effect Floor
The consumed runtime-derived effect floor should include at least:
1. dynamic reference activation/release
2. region or shape activation/release
3. capability observations
4. format observations
5. execution-restriction observations
6. host-query-sensitive facts where they affect candidate or publication meaning

### 9.7 Direct-Binding And Host-Sensitive Truth
Where TreeCalc semantics depend on concrete binding identity rather than name-only semantics:
1. OxFml remains authoritative for the meaning of those bindings,
2. OxCalc must preserve them in replay, reduced witnesses, and pack-candidate families,
3. the first TreeCalc-ready engine should not erase or normalize those identities away.

### 9.8 Semantic-Format Versus Display Boundary
This phase should explicitly consume:
1. semantic consequences needed for stabilized engine truth,
2. format-sensitive consequences that may affect runtime or later observer policy,
3. display-sensitive consequences only to the extent needed to keep the seam honest for future widening.

This first phase does **not** need to solve broad display semantics.
It does need to avoid collapsing semantic, format, and display categories into one generic side effect.

## 10. TreeCalc Reference Semantics To Lock Locally
This section defines the local coordinator-facing questions that must be explicit before implementation starts.

### 10.1 Absolute Direct Node References
Need explicit local semantics for:
1. stable target-node identity,
2. what happens if the target node moves structurally but remains semantically the same node,
3. what happens if the target disappears,
4. replay-visible dependency identity.

### 10.2 Tree-Relative References
Need explicit local semantics for:
1. what structural context anchors relative lookup,
2. whether relative meaning is bind-time fixed or runtime-context-sensitive,
3. which structural edits force rebind or re-evaluation,
4. how relative-reference changes surface as dependency reclassification versus hard rebinding.

### 10.3 Named Or Symbolic References
Need explicit local semantics for:
1. whether they resolve to stable node identity before evaluation,
2. whether the bind product already fixes them,
3. how ambiguity or rebinding failure is represented.

### 10.4 Dynamic Reference Families
Need explicit local semantics for:
1. what part of dependency meaning is static,
2. what part is runtime-observed,
3. what candidate-result payload must surface when the effective dependency set changes,
4. what triggers fallback.

## 11. Runtime And Coordinator Semantics To Lock

### 11.1 Structural Edit Consequences
The first TreeCalc-ready engine should make structural edit consequences explicit for:
1. node rename,
2. node move in the tree,
3. formula replacement,
4. node addition/removal,
5. changes that alter relative-reference meaning.

### 11.2 Rebind Versus Recalc
Need an explicit boundary between:
1. structure or formula changes that require rebind,
2. changes that leave bind valid but require recalc,
3. runtime-derived changes that require only dynamic dependency overlay updates.

### 11.3 Candidate/Publication Consequences
Need explicit rules for:
1. when topology consequences become publication-visible,
2. when runtime-derived facts remain internal only,
3. when format-sensitive facts must remain attached to stable publication bundles,
4. when reject paths preserve diagnostics but no publish-scoped effect.

### 11.4 Verified-Clean Semantics
Need an explicit rule for what "verified clean" means in the real TreeCalc path:
1. no observable semantic change for the declared equality surface,
2. no publication emitted,
3. explicit trace and replay semantics.

## 12. Required Evidence For This Phase
The first TreeCalc-ready engine phase should not be declared reached without all of the following:

### 12.1 Structural/Formula Evidence
1. checked-in TreeCalc corpus scenarios that use real node/formula/reference families,
2. real formula artifact and bind intake through the live engine path.

### 12.2 Coordinator Evidence
1. deterministic accept/reject/publication artifacts for the covered TreeCalc scope,
2. pinned-view invariants exercised over real formula-driven runs.

### 12.3 Dependency Evidence
1. dependency graph artifacts or deterministic diagnostics showing real formula-driven dependency derivation,
2. exercised dynamic-dependency or dependency-reclassification cases where in scope.

### 12.4 Replay Evidence
1. ordinary replay-appliance bundle roots for the TreeCalc corpus,
2. engine diff and explain artifacts over the real engine path,
3. retained-witness continuation where the real engine path creates new mismatch families.

### 12.5 Assurance Evidence
1. W008 and W009 bindings refreshed against the real TreeCalc engine path,
2. at least one formal or model artifact updated where object names or transition meaning changed materially.

## 13. Work Sequence To Reach The First TreeCalc-Ready Engine
This sequence is the line-of-sight plan.
It is intentionally phrased as a work sequence that can later be broken into discrete worksets.

### TS-1: TreeCalc Structural And Formula-Carrying Substrate
Objective:
1. widen the structural model so nodes can carry real formula/bind artifact references and relative-reference context.

Exit gate:
1. immutable snapshots can represent the first real TreeCalc node/formula substrate,
2. structural edit and identity rules are explicit enough to implement rebind/recalc consequences.

### TS-2: OxFml TreeCalc Bind And Reference Intake
Objective:
1. lock and consume the first real OxFml bind/reference package needed for TreeCalc.

Exit gate:
1. OxCalc can consume formula artifact identities, bind identities, and reference meaning for the covered TreeCalc families,
2. unresolved seam items are narrowed explicitly.

### TS-3: Dependency Graph Build From Real Formula/Bind Facts
Objective:
1. replace planner-only dependency derivation with real static dependency build from consumed bind facts.

Exit gate:
1. structural dependency graph and reverse edges exist for the covered TreeCalc formula families,
2. dependency identity is replay-visible.

### TS-4: Real Candidate-Result Intake Path
Objective:
1. move candidate intake from synthetic/scripted `TraceCalc` candidates to real OxFml-backed evaluator outputs.

Exit gate:
1. the coordinator consumes real seam-produced candidate results and typed rejects for the covered TreeCalc scope.

### TS-5: Runtime-Derived Dependency And Overlay Closure
Objective:
1. make dynamic dependency, capability, execution-restriction, and shape-sensitive runtime effects real in the engine path.

Exit gate:
1. runtime-derived facts that affect recalc or publication are explicit, replay-visible, and no longer test-only constructs.

### TS-6: Real TreeCalc Recalc/Publication End-To-End Runs
Objective:
1. execute a TreeCalc corpus through structure -> dependency -> evaluation -> candidate -> publish.

Exit gate:
1. the live engine path can run the first TreeCalc corpus without `TraceCalc` scripted semantics standing in for real formula execution.

### TS-7: Corpus And Oracle Widening For TreeCalc Scope
Objective:
1. widen the corpus and oracle surface so the real TreeCalc engine can be compared deterministically.

Exit gate:
1. ordinary TreeCalc runs have conformance and replay artifacts analogous to the current `TraceCalc` lane.

### TS-8: Sequential TreeCalc-Ready Baseline
Objective:
1. emit the first baseline run that honestly represents the first TreeCalc-ready sequential engine.

Exit gate:
1. one checked-in TreeCalc-ready baseline exists,
2. semantic-equivalence statement is explicit for any strategy substitutions used along the way.

### TS-9: Assurance And Pack Refresh For The New Engine Path
Objective:
1. refresh TLA+/replay/pack bindings around the live TreeCalc path so the proving surface matches the engine we actually have.

Exit gate:
1. no major semantic clause remains bound only to the older proving substrate.

## 14. Recommended Workset Breakdown Direction
This document does not assign final workset numbers.

But the recommended decomposition is:
1. one workset for structural/future-proof TreeCalc substrate widening,
2. one workset for OxFml seam intake focused on TreeCalc formula/bind/reference packages,
3. one workset for real dependency graph build and invalidation closure,
4. one workset for evaluator-backed candidate-result integration,
5. one workset for runtime-derived dependency and overlay closure,
6. one workset for TreeCalc corpus/oracle widening and first baseline evidence,
7. one workset for assurance refresh and residual packetization.

That split keeps the major semantic boundaries visible:
1. structural truth,
2. seam intake,
3. dependency truth,
4. candidate/publication truth,
5. runtime-derived truth,
6. evidence truth.

### 14.1 Packetized Workset Mapping
The current recommended packetization of this sequence is:
1. `W025_TREECALC_STRUCTURAL_AND_FORMULA_SUBSTRATE_WIDENING.md`
   - covers `TS-1`
2. `W026_TREECALC_OXFML_BIND_REFERENCE_AND_SEAM_INTAKE.md`
   - covers the consumed-seam floor for `TS-2`
3. `W027_TREECALC_DEPENDENCY_GRAPH_AND_INVALIDATION_CLOSURE.md`
   - covers `TS-3` plus the structural invalidation closure portions of `TS-5`
4. `W028_TREECALC_EVALUATOR_BACKED_CANDIDATE_RESULT_INTEGRATION.md`
   - covers `TS-4`
5. `W029_TREECALC_RUNTIME_DERIVED_EFFECTS_AND_OVERLAY_CLOSURE.md`
   - covers the runtime-derived effect and overlay portions of `TS-5`
6. `W030_TREECALC_CORPUS_ORACLE_AND_FIRST_SEQUENTIAL_BASELINE.md`
   - covers `TS-6`, `TS-7`, and `TS-8`
7. `W031_TREECALC_ASSURANCE_REFRESH_AND_RESIDUAL_PACKETIZATION.md`
   - covers `TS-9` and any residual packetization required after the first TreeCalc-ready baseline

This decomposition is the intended line-of-sight sequence after the current replay-pack residual lane.

Current host-contract interpretation:
1. `W025` through `W031` now widen the engine beneath the OxCalc-owned TreeCalc-first consumer contract in `CORE_ENGINE_OXCALCTREE_CONSUMER_INTERFACE_AND_HOST_CONTRACT_V1.md`,
2. actual runtime consumers should prefer that explicit `OxCalcTreeEnvironment` / `OxCalcTreeDocument` / `OxCalcTreeRecalcRequest` / `OxCalcTreeRecalcResult` / `OxCalcTreeRuntimeFacade` object set over local proving-floor engine types,
3. narrower seam-intake, dependency-preparation, replay, and assurance details remain packetized below that host-facing contract until later widening closes them honestly.

Current W026 packet floor under this plan:
1. Sequence 1 now has an executed first caller-context/reference floor:
   - first closed subset is `DirectNode`, admitted `RelativePath` (`ParentNode` and `Ancestor(n >= 1)` descendant lookup), `SiblingOffset`, `Unresolved`, `HostSensitive`, and `DynamicPotential`
   - explicit per-formula identity and caller-context carriage is now part of the consumed seam floor for that subset
   - rebind-versus-recalc and dependency-descriptor lowering are now explicit beneath the host-facing contract
2. Sequence 2 now has an executed first runtime-derived transport floor:
   - current explicit correlation floor is `candidate_result_id` plus `publication_id`
   - current emitted runtime-derived families are only `DynamicDependency` and `ExecutionRestriction`
   - `CapabilitySensitive` remains admitted but unexercised
   - direct host-facing and replay-facing reachability for that emitted subset is now locked
3. Sequence 3 now has an executed first publication/topology floor:
   - `value_delta` is the only currently published consequence family
   - `shape_delta`, `topology_delta`, optional `format_delta`, and optional `display_delta` remain explicit current absences
   - current execution-restriction observations remain runtime-effect-plus-typed-no-publish context rather than published consequence families
4. this plan should therefore treat broader caller-context, broader runtime-derived family realization, and broader publication/topology consequence breadth as later widening work rather than as ambiguity about the current first floor
5. W026 has now reached its declared gate for that first packet:
   - W027 may proceed on executed dependency-descriptor and invalidation-seed truth
   - W028 may proceed on executed candidate/reject/correlation and publication-consequence truth
   - W029 may proceed on the explicit W026-to-W029 boundary and current emitted runtime-derived family floor

## 15. Non-Negotiable Guardrails For Later Performance Work
The following must remain true so later ultraperformance work still lands on the right semantic base:
1. no scheduler or caching shortcut may redefine stabilized semantic truth,
2. no optimization may bypass single-publisher coordinator authority,
3. dynamic dependencies remain explicit runtime-derived state rather than hidden mutable graph edits,
4. formula parse/bind/evaluator meaning remains OxFml-owned,
5. relative-reference meaning remains explicit and replay-visible,
6. direct-binding-sensitive truth remains preserved where semantics depend on it,
7. structural truth remains immutable and versioned,
8. concurrency arrives only after the sequential TreeCalc engine path is semantically real.

## 16. Immediate Design Questions To Resolve Early
The following questions should be resolved early in the next work sequence rather than deferred:
1. exact first TreeCalc reference-family subset,
2. exact bind-product shape OxCalc will consume,
3. exact structural edit families the first TreeCalc engine must support,
4. exact dependency artifact or diagnostic shape emitted by the live engine path,
5. exact verified-clean semantics for real formula-driven runs,
6. exact format/display consequence floor to consume without overcommitting.

## 17. Relationship To Existing Work
This document does not replace:
1. the canonical architecture docs,
2. the current roadmap,
3. the `TraceCalc` proving lane,
4. the current replay and retained-witness lanes.

It does:
1. define the next real semantic target,
2. explain the gap between today’s proving substrate and that target,
3. define the execution line needed to close that gap.

## 18. Status
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - this document defines the broader target and sequence, but only the first narrow TreeCalc local slice plus the ordinary runtime/replay V1 facade intake are currently exercised
  - broader TreeCalc bind/reference intake, dependency build, and candidate-result consumption still remain open beyond the current local slice
  - W026 is now reached-gate for the first executed Sequence 1/2/3 packet floor, while broader caller-context, runtime-derived family, and publication/topology breadth widening now belongs to later packets rather than to unresolved first-floor seam ambiguity
- claim_confidence: provisional
- reviewed_inbound_observations: latest OxFml downstream note consumed as seam baseline; no new immediate handoff trigger exists yet
