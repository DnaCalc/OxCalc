# CORE_ENGINE_OXFML_SEAM.md

## 1. Purpose and Status
This document defines the OxCalc view of the OxFml seam for the rewritten core-engine spec set.

Status:
1. active rewrite baseline,
2. intended canonical OxCalc-local seam companion,
3. coordinator-facing in emphasis,
4. partially aligned to OxFml canonical seam updates from `HANDOFF-CALC-001`.

**This document does not claim canonical ownership of the shared evaluator protocol.**
OxFml remains the canonical owner of shared FEC/F3E seam specification.
OxCalc docs must not be cited as permission to invent a private evaluator contract when OxFml has not frozen the shared meaning.

This document exists to make OxCalc's coordinator-facing requirements explicit.
Actual OxCalc runtime consumers such as `DNA TreeCalc` should read `CORE_ENGINE_OXCALCTREE_CONSUMER_INTERFACE_AND_HOST_CONTRACT_V1.md` before this seam companion.
For downstream hosts that use OxCalc as seam-reference material only:
1. read `CORE_ENGINE_DOWNSTREAM_HOST_SEAM_REFERENCE.md` first — it is the single entry point and authority filter for downstream hosts, including the document classification summary and host-packet interpretation model,
2. then read this document as the canonical OxCalc-local seam companion,
3. then read `CORE_ENGINE_OXFML_MINIMAL_UPSTREAM_HOST_INTERFACES.md` as the first deterministic upstream-host packet companion — reference material only, not a host API to adopt verbatim (see `CORE_ENGINE_DOWNSTREAM_HOST_SEAM_REFERENCE.md` Section 7.1),
4. read `CORE_ENGINE_TREECALC_OXFML_SEAM_NEGOTIATION_MATRIX.md` only for narrower residual topics and non-assumptions; it is temporary-planning material and not seam authority

## 2. Ownership Rule
The seam is shared, but ownership is split.

### 2.1 OxFml Owns
OxFml owns:
1. formula grammar,
2. parse and bind semantics,
3. evaluator-side session and execution semantics,
4. canonical shared seam specification text,
5. evaluator-facing trace and result contracts where those are canonical seam artifacts.

### 2.2 OxCalc Owns
OxCalc owns:
1. coordinator acceptance and rejection consequences,
2. publication-fence requirements,
3. snapshot compatibility requirements from the coordinator side,
4. scheduling and stabilization interaction,
5. what evaluator-produced `AcceptedCandidateResult` artifacts must provide for coordinator-controlled publication.

### 2.3 Shared-Clause Rule
Where a clause is shared but canonical in OxFml, OxCalc must express its requirement locally and then hand off canonical text changes rather than silently diverging.

### 2.4 Consumer-Contract Alignment Rule
For actual OxCalc runtime consumers:
1. `CORE_ENGINE_OXCALCTREE_CONSUMER_INTERFACE_AND_HOST_CONTRACT_V1.md` defines the OxCalc-facing tree-host object set,
2. this seam companion defines the consumed evaluator/coordinator boundary that object set must preserve,
3. consumer packaging must not be read as permission to collapse or reinterpret OxFml-owned artifact meaning.

## 3. Why This Seam Must Be Explicit
The seam must be explicit because:
1. the evaluator is not the coordinator,
2. evaluator-produced `AcceptedCandidateResult` is not identical to committed publication,
3. replay and reject behavior depend on shared structure,
4. later concurrency makes weak seam wording unsafe.

If the seam is left implicit, publication, runtime state, and evaluator behavior will drift into one another.
The rewrite rejects that outcome.

## 4. OxCalc Expectations Of Evaluator Artifacts
OxCalc treats evaluator artifacts as immutable, versioned inputs.

The seam therefore requires that OxCalc be able to reason about:
1. which immutable evaluator artifact a candidate work unit is based on,
2. what token or version discipline guards that artifact,
3. what profile/version context applies,
4. what compatibility assumptions are being asserted by candidate work.

OxCalc does not need to own the evaluator internals to require this compatibility structure.

## 5. Candidate Work Boundary
The seam must expose a clear boundary between:
1. structural/evaluator inputs,
2. candidate evaluation work,
3. evaluator-produced `AcceptedCandidateResult`,
4. accepted publication consequences.

This distinction matters because the coordinator must be able to:
1. reject candidate work safely,
2. publish accepted work atomically,
3. preserve stable observer-visible state,
4. replay and diagnose accept/reject behavior.

## 6. Snapshot and Fence Requirements
From the OxCalc side, the seam must support coordinator reasoning about compatibility and fences.

At minimum, the seam must make it possible for the coordinator to determine:
1. which snapshot or structural basis candidate work depends on,
2. which evaluator artifact/token basis candidate work depends on,
3. whether profile/version assumptions match,
4. whether candidate work is eligible for publication under current coordinator state.

The exact canonical field names belong in shared seam specs, but the architectural requirement is fixed here.

## 7. Accepted Candidate Result Requirements
For accepted work, the seam must provide an evaluator-produced `AcceptedCandidateResult` structure rich enough for coordinator-controlled publication.

This means OxCalc must be able to receive or derive, through the seam, the information required to:
1. publish accepted results atomically,
2. update stable observer-visible derived state coherently,
3. integrate relevant topology/dependency consequences,
4. preserve replay and diagnostic fidelity.

The coordinator does not accept opaque success without adequate publication-relevant structure.

## 8. Reject Detail Requirements
Rejected work is architecturally no-publish.

But reject outcomes must still provide structured detail sufficient for:
1. deterministic replay,
2. diagnostics,
3. seam-hardening work,
4. staged concurrency analysis.

From the OxCalc side, the seam must support reject detail that distinguishes at least:
1. compatibility or fence mismatch,
2. artifact/token mismatch,
3. capability or session mismatch where relevant,
4. other coordinator-relevant reject classes that affect replay and migration understanding.

The canonical taxonomy belongs in shared seam work, but the requirement for structured detail is locked here.

## 9. Publication Ownership Rule
The seam must not blur evaluator success with committed publication.

The evaluator may produce an `AcceptedCandidateResult`.
The coordinator alone decides whether that result becomes committed published consequences.

Therefore the seam must preserve the distinction between:
1. evaluator-produced `AcceptedCandidateResult`,
2. coordinator-accepted publication,
3. rejected no-publish outcome.

## 10. Dynamic Dependency and Runtime-Derived Consequences
Where evaluator execution reveals runtime-relevant facts that matter to OxCalc coordination,
the seam must support explicit transmission or derivation of those facts.

This is necessary for cases such as:
1. runtime-observed dependency effects,
2. runtime capability or fence implications,
3. other evaluator-discovered facts that influence recalc or publication.

These effects must not be left as hidden evaluator internals if OxCalc is expected to coordinate on them.

## 11. Stage-1 Versus Later-Stage Seam Pressure

### 11.1 Stage 1
Stage 1 may realize a conservative subset of the full seam-hardening story.

But even in Stage 1, the seam must already preserve:
1. candidate-versus-publication distinction,
2. explicit compatibility or fence basis,
3. reject detail adequate for replay and diagnostics,
4. coordinator ownership of accept or reject consequences.

### 11.2 Later Stages
Later concurrent and async stages increase seam pressure.

They require stronger handling for:
1. contention and retry visibility,
2. fence mismatches under concurrent work,
3. deterministic replay of staged concurrency outcomes,
4. publication safety under overlapping candidate work.

The seam should therefore be written now with later hardening in mind.

## 12. Handoff Rule
Whenever OxCalc local requirements imply changes to canonical shared seam text, OxCalc must:
1. document the local requirement here,
2. prepare an explicit handoff packet for OxFml,
3. register the handoff,
4. avoid claiming the shared clause is fully resolved until the canonical side acknowledges it.

## 13. Formalization and Evidence Direction
This seam is assurance-relevant, not only integration-relevant.

Expected obligations include:
1. replay-visible candidate-versus-publication distinctions,
2. structured reject-detail coverage,
3. fence-safety modeling tied into coordinator assurance,
4. pack obligations for commit atomicity and reject determinism,
5. evidence artifacts sufficient for staged concurrency hardening.

## 14. Current Handoff State
`HANDOFF-CALC-001` has been filed and acknowledged.
The current shared direction now includes:
1. explicit `AcceptedCandidateResult` terminology at the OxFml seam,
2. typed no-publish reject detail for fence and capability incompatibility,
3. coordinator-relevant runtime-derived effect surfacing as a general seam rule.

Follow-on handoff pressure remains only where OxCalc later needs narrower or stronger requirements than the current shared canonical wording.

`HANDOFF-FML-001` has now also been received from OxFml.
That inbound handoff and the current OxFml downstream note strengthen the currently consumed floor with:
1. minimum typed schema objects for accepted candidate, commit, reject-context, and trace-correlation payload families,
2. a stronger managed-session baseline for stale-fence rejection, capability denial, session termination, and execution-restriction-sensitive no-publish paths,
3. a stronger replay and retained-local floor through the current OxFml-local `cap.C3.explain_valid` posture,
4. an explicit DNA OneCalc downstream host boundary that must not be mistaken for OxCalc coordinator policy.

The latest note-exchange round with OxFml also narrows several earlier uncertainties:
1. identity and fence vocabulary consumption is now treated as already canonical on the OxFml side,
2. candidate-result and commit-bundle consequence categories are now treated as already canonical on the OxFml side,
3. host-query and direct-binding-sensitive truth is now treated as already canonical on the OxFml side,
4. dependency consequence taxonomy and semantic-display boundary remain canonical but narrower rather than fully open.

## 15. OxCalc-Local Stage 1 Minimum Seam Packet

### 15.1 AcceptedCandidateResult Minimum
For Stage 1, OxCalc requires the shared seam to preserve enough information to derive or surface a minimum local `AcceptedCandidateResult` containing:
1. `candidate_result_id`
2. consumed identity and fence basis:
   - `formula_stable_id`
   - `formula_token`
   - `snapshot_epoch`
   - `bind_hash`
   - `profile_version`
   - important-but-still-narrower `capability_view_key`
3. trace and publication correlation:
   - `commit_attempt_id` where present
   - `reject_record_id` where relevant
   - optional `fence_snapshot_ref`
4. candidate publication-consequence categories:
   - `value_delta`
   - `shape_delta`
   - `topology_delta`
   - optional `format_delta`
   - optional `display_delta`
   - optional spill-event set
5. surfaced evaluator facts needed for coordinator correctness where not already derivable from the deltas
6. diagnostic and trace correlation metadata

This is an OxCalc-local minimum requirement for coordinator-controlled publication.
It does not claim that the shared OxFml-side canonical field names or artifact layering are identical.
But it now explicitly consumes the already-canonical OxFml category split rather than compressing it into generic local buckets alone.

### 15.2 Runtime-Derived Effect Subset
For Stage 1, OxCalc expects at least the following local runtime-derived effect subset to be preservable through the seam:
1. `dynamic_ref_activated`
2. `dynamic_ref_released`
3. `region_shape_activated`
4. `region_shape_released`
5. `capability_observed`
6. `format_observed`
7. `execution_restriction_observed`

This subset is the local coordinator and overlay floor.
It is not a claim that the broader shared runtime-derived effect taxonomy is closed.
Current shared reading after the latest note round:
1. execution-restriction effects are stable enough to consume semantically now,
2. OxCalc should not yet assume one final frozen single-object carrier for those effects,
3. dependency additions, removals, and reclassifications remain intended evaluator/runtime facts, but their exact retained/reduced witness projection closure is still narrower than a fully frozen universal rule.

### 15.3 Reject Subset
For Stage 1, OxCalc expects the shared seam to support a local typed reject subset covering at least:
1. `snapshot_mismatch`
2. `artifact_token_mismatch`
3. `profile_version_mismatch`
4. `capability_mismatch`
5. `publication_fence_mismatch`
6. `dynamic_dependency_failure`
7. `synthetic_cycle_reject`
8. `host_injected_failure`

This is the minimum local reject floor needed for coordinator no-publish behavior, replay classification, and self-contained harness scenarios.
It does not claim that the shared OxFml-side canonical taxonomy or ownership split is fully closed.

The current stronger OxFml-managed baseline makes the following canonical context families especially important to preserve without coordinator reinterpretation:
1. `FenceMismatchContext`
2. `CapabilityDenialContext`
3. `SessionTerminationContext`
4. `DynamicReferenceFailureContext`

### 15.4 Host-Boundary Preservation Rule
OxCalc does not own DNA OneCalc host policy.
But where retained witnesses, pack-candidate artifacts, or replay-valid scenarios depend on concrete host-sensitive truth, OxCalc must preserve the OxFml-declared direct-binding boundary rather than collapsing those cases into name-only or prose-only artifacts.

This is a replay and evidence-preservation rule.
It is not a transfer of host-policy ownership into OxCalc.

Current shared reading after the latest note round:
1. typed host-query capability views are already canonical on the OxFml side,
2. direct-cell-binding-sensitive truth is already canonical on the OxFml side where semantic correctness depends on concrete resolution,
3. the broader naming and indexing convention for direct-binding-sensitive pack-candidate families remains open and belongs to later replay widening rather than immediate seam redefinition.

## 16. Open Detailed Questions
These remain seam-hardening questions rather than reasons to weaken the split:
1. exact accepted-result payload naming and artifact partition in shared canonical terms,
2. exact reject taxonomy ownership partition beyond the now-locked Stage 1 local subset,
3. exact broader runtime-derived effect taxonomy beyond the Stage 1 local subset, especially execution-restriction and capability-sensitive transport closure,
4. exact retained/reduced witness projection closure for dependency additions, removals, and reclassifications,
5. exact trace schema mapping for coordinator-facing replay and diagnostics, especially stable use of `candidate_result_id`, `commit_attempt_id`, `reject_record_id`, and optional fence snapshot references,
6. exact replay-facing preservation rule for direct-binding-sensitive witness and pack-candidate families once W019 broadens them,
7. exact shared reading of semantic-format versus display-facing publication consequences before broader retained and pack-candidate widening.

The latest OxFml note also makes one useful non-trigger explicit:
1. current OxFunc refinement and round-closure work does not yet introduce a new OxCalc-facing seam change,
2. availability/provider-failure handling and callable-publication restriction are the most likely future upstream semantic lanes to become coordinator-visible later,
3. OxCalc should treat those as watch lanes rather than as current seam-closure blockers.

## 17. TreeCalc Seam Negotiation Topics
The next TreeCalc-ready engine phase requires a narrower negotiation shape than the earlier Stage 1 seam passes.

The required note-exchange topics are:
1. formula and bind artifact identity carriage for formula-bearing TreeCalc nodes,
2. direct-reference versus relative-reference descriptor carriage,
3. unresolved or host-sensitive reference carrier rules,
4. dependency consequence carriage for additions, removals, and reclassifications,
5. candidate-result consequence optionality and correlation guarantees,
6. reject-context carrier and minimum diagnostic guarantee,
7. runtime-derived effect transport and projection rules,
8. direct-binding and host-sensitive witness-preservation rules,
9. semantic-format versus display-facing consequence boundary.

These topics are negotiation topics, not yet all formal handoff triggers.
The purpose is to force explicit consumption decisions before W026 and later TreeCalc execution work.

## 18. Required Consumed Topic Matrix For W026
For the first TreeCalc-ready engine phase, OxCalc should process the seam in the following topic matrix.

### 18.1 Topic A: Formula and Bind Artifact Identity
OxCalc needs:
1. stable formula artifact identity for formula-bearing nodes,
2. bind-product identity and version basis,
3. compatibility basis needed to determine whether a structure/formula edit implies rebind or only recalc.

Expected current answer shape from OxFml:
1. canonical now for `formula_stable_id`, `formula_token`, `bind_hash`, `snapshot_epoch`, and `profile_version`,
2. narrower but consumable for `capability_view_key` where compatibility-sensitive evaluation meaning depends on it.

W026 should explicitly record:
1. which of these are required on every formula-bearing node,
2. which may remain optional until candidate-result time,
3. which are replay-visible identifiers versus compatibility-only handles.

### 18.2 Topic B: Reference Descriptor Carriage
OxCalc needs:
1. direct-node reference descriptors,
2. relative-reference descriptors or already-bound relative targets,
3. explicit unresolved or host-sensitive reference forms,
4. a rule for whether relative meaning is fixed at bind time or remains contextual.

W026 should force explicit answers to:
1. what the first in-scope relative-reference subset is,
2. whether the bind product already resolves relative navigation fully,
3. which structural edits force rebind rather than recalc.

### 18.3 Topic C: Dependency Consequence Carriage
OxCalc needs:
1. static dependency facts suitable for graph build,
2. runtime-derived dependency additions, removals, and reclassifications,
3. explicit identity for dependency facts that later replay and reduced-witness lanes can preserve.

Current shared read:
1. semantic intent is stable enough to consume now,
2. exact retained/reduced witness closure remains narrower than a universal rule.

W026 should therefore separate:
1. consumed now for live dependency and recalc semantics,
2. still-open retained/reduced witness projection closure.

### 18.4 Topic D: Candidate Result and Commit Consequence Carriage
OxCalc needs:
1. `candidate_result_id`,
2. stable correlation with `commit_attempt_id` where present,
3. optional `fence_snapshot_ref` where present,
4. canonical consequence categories:
   - `value_delta`
   - `shape_delta`
   - `topology_delta`
   - optional `format_delta`
   - optional `display_delta`
   - spill or shape events
5. surfaced evaluator/runtime facts required for coordinator correctness.

W026 should make explicit:
1. which optional consequence families must still preserve explicit absence/presence semantics,
2. which families are publish-critical for the first TreeCalc phase,
3. which remain carried only for replay honesty rather than first-phase coordinator behavior.

### 18.5 Topic E: Reject Context Carriage
OxCalc needs typed reject carriers for at least:
1. snapshot mismatch,
2. token or artifact mismatch,
3. profile mismatch,
4. capability denial,
5. publication-fence mismatch,
6. execution restriction or invalid phase,
7. dynamic dependency failure,
8. host-sensitive resolution failure where relevant.

W026 should clarify:
1. which reject contexts are canonical OxFml object families already,
2. which local OxCalc labels remain merely local projections,
3. which reject families must preserve additional host-sensitive or bind-sensitive diagnostics.

### 18.6 Topic F: Runtime-Derived Effect Transport
OxCalc needs explicit carriage for:
1. dynamic dependency activation and release,
2. capability observations,
3. execution-restriction observations,
4. shape and topology-sensitive runtime effects,
5. format-sensitive runtime effects where semantically relevant.

Current shared read:
1. these are stable enough to consume semantically now,
2. the final single transport carrier is not yet frozen.

W026 should therefore force explicit recording of:
1. semantic minimum consumed now,
2. transport-shape assumptions OxCalc must not make yet,
3. what later evidence would justify a narrower handoff.

### 18.7 Topic G: Direct-Binding and Host-Sensitive Truth
OxCalc needs:
1. preserved concrete binding identity where semantic truth depends on it,
2. explicit distinction between direct-binding-sensitive families and name-only families,
3. replay-visible host-sensitive identity in retained and reduced witnesses where required.

W026 should keep explicit:
1. this is already canonical in OxFml semantic ownership,
2. OxCalc is only consuming and preserving it,
3. broader naming/indexing conventions for later pack families may still remain open.

### 18.8 Topic H: Semantic-Format Versus Display Boundary
OxCalc needs:
1. a first consumed semantic floor,
2. explicit format-sensitive consequences where they may affect runtime or later observer policy,
3. display-sensitive consequences kept visible enough not to be silently collapsed.

W026 should not force premature closure here.
It should instead record:
1. what is consumed now for the first TreeCalc phase,
2. what remains canonical but narrower,
3. what evidence in later TreeCalc runs would justify a narrower handoff.

## 19. Note-Exchange Rule For W026
W026 should treat `NOTES_FOR_OXFML.md` and `NOTES_FOR_OXCALC.md` as structured negotiation instruments rather than general commentary.

Each pass should record, for every active topic:
1. OxCalc consumed need,
2. current OxFml classification:
   - `already canonical`
   - `canonical but narrower`
   - `still open`
3. consumed-now carrier assumptions,
4. non-assumptions OxCalc must preserve,
5. explicit trigger for whether note-level clarification is enough or a narrower handoff is required.

The note passes should stop being generic once W026 starts.
They should function as a bounded seam issue ledger until the first TreeCalc-ready intake floor is locked.

The latest OxFml topic-matrix reply makes the current practical split clearer:
1. consume now:
   - formula and bind identity carriage
   - candidate consequence and correlation floor
   - reject-context typed families for the current floor
   - direct-binding-sensitive witness preservation
2. keep in note-level refinement:
   - direct and relative reference descriptor carriage
   - unresolved and host-sensitive reference carriers
   - runtime-derived effect transport shape
   - semantic-format versus display-facing boundary

This means the current seam state is clear enough to proceed into W026 planning and later implementation preparation without reopening the shared ownership split.
It does not mean every transport shape is frozen.

The latest narrower W026-focused OxFml reply further sharpens this:
1. W026 can proceed now on a narrowed first relative-reference subset,
2. W026 can proceed now on explicitly named unresolved and host-sensitive carrier families,
3. W026 can proceed now on the semantic floor for runtime-derived effects and execution-restriction transport,
4. W026 can proceed now on a semantics-first semantic-format/display split so long as broader display closure is not over-claimed.

So the seam interface is settled enough for the first TreeCalc intake phase.
What remains unsettled is not the ownership split or the consumed semantic floor; it is broader transport-shape closure beyond the first subset.

## 20. Handoff Trigger Rule For The TreeCalc Seam Phase
For the TreeCalc semantic-completion lane, a new narrower handoff should be filed only if one of the following occurs:
1. OxCalc cannot consume the first in-scope bind/reference package without OxFml changing or clarifying a coordinator-facing seam clause,
2. execution-restriction transport is too narrow for live TreeCalc coordinator semantics,
3. dependency consequence transport is too narrow for live TreeCalc graph build or publication semantics,
4. candidate-result consequence optionality is too weak for coordinator-controlled publication,
5. direct-binding-sensitive truth cannot be preserved honestly for the first TreeCalc witness families.

Otherwise the issue should remain in the note-exchange lane and be resolved there.

## 21. Host Runtime Draft Intake
OxCalc now also treats `../OxFml/docs/spec/OXFML_HOST_RUNTIME_AND_EXTERNAL_REQUIREMENTS.md` as the bounded OxFml-owned packet for the next host/coordinator seam round.

Current local read is:
1. it is sufficient for first implementation planning across the reduced direct-host lane and the first OxCalc-integrated host lane,
2. it preserves the authority split correctly:
   - OxFml owns artifact meaning, typed effect/reject semantics, and runtime library-context truth,
   - OxCalc owns scheduler, publication, invalidation integration, and broader graph coordination outside OxFml artifact meaning,
3. it does not yet need to be treated as shared seam-freeze text,
4. it narrows host/coordinator seam uncertainty to a small set of remaining carrier-breadth questions.

### 21.1 Consumed-now host/runtime floor
For the first host/coordinator implementation slice, OxCalc now treats the following as consume-now:
1. direct-host versus OxCalc-integrated host split,
2. formula and structure inputs,
3. direct-cell and defined-name bindings,
4. typed host-query/provider families in the currently covered floor:
   - `INFO`
   - `CELL`
   - `RTD`
5. runtime `LibraryContextProvider` plus immutable `LibraryContextSnapshot` as the normative runtime catalog seam,
6. candidate / commit / reject / trace output families,
7. `ReturnedValueSurface` split,
8. coordinator-relevant ids:
   - `candidate_result_id`
   - `commit_attempt_id` where present
   - `reject_record_id`
   - optional `fence_snapshot_ref`

### 21.2 Host/runtime residuals that remain narrower
The remaining narrower host/runtime questions are:
1. caller-anchor and address-mode carriage for the first TreeCalc relative-reference subset,
2. execution-restriction transport shape beyond the current semantic minimum,
3. publication/topology consequence breadth beyond the current exercised local floor,
4. provider-failure and callable-publication watch lanes if they later become coordinator-visible.

Current local read:
1. these remain note-level topics,
2. no new handoff is justified yet from the host/runtime draft alone,
3. they become handoff candidates only if live TreeCalc or host evidence exposes insufficiency.
4. the latest OxFml reply explicitly agrees the host/runtime draft is strong enough for first implementation planning, while preserving these caution points as non-frozen residuals.
5. the later `W051` and `W052` stand-in packet refinements sharpen deterministic scaffolding inputs without changing this residual set:
   - stand-in packet identity, structure-context identity, and formula-slot identity are now accepted refinements,
   - `RegisteredExternalProvider` remains optional,
   - any later host-initiated registration lane should be modeled as a typed mutation request into OxFunc-owned catalog truth rather than as coordinator-owned catalog mutation.
6. the latest narrowed `W052` reply further sharpens this registered-external lane without changing the broader seam split:
   - direct adoption of `RegisterIdRequest`, `RegisteredExternalDescriptor`, `RegisteredExternalCallRequest`, and `RegisteredExternalTarget::{ RegisterId, Direct }` is now the settled current direction for the first packet,
   - the current seven-field `RegisteredExternalDescriptor` is sufficient for first TreeCalc-facing planning,
   - `RegisteredExternalCatalogMutation*` and `RegisteredExternalCatalogController` remain OxFml-owned host/coordinator funnel packets for the current phase,
   - bind-visible register or unregister implies new `LibraryContextSnapshot` generation plus bind invalidation where the visible function or name world changes, while `CALL` / `REGISTER.ID`-only descriptor mutation may remain targeted reevaluation by default.

### 21.2C Current Executed W026 Residual Floor
The W026 residual lane is no longer only a note-level planning split.
The current executed OxCalc floor is now:
1. Sequence 1 caller-context and reference intake:
   - first closed subset is `DirectNode`, admitted `RelativePath` (`ParentNode` and `Ancestor(n >= 1)` descendant lookup), `SiblingOffset`, `Unresolved`, `HostSensitive`, and `DynamicPotential`
   - explicit carried identity and caller-context floor is now recorded and exercised:
     - `formula_stable_id`
     - `formula_token`
     - optional `bind_artifact_id`
     - `structure_context_version`
     - `caller_anchor`
     - `formula_channel_kind`
     - `address_mode`
   - rebind-versus-recalc and dependency-descriptor mapping are now explicit for that subset
2. Sequence 2 candidate/reject/runtime-derived transport:
   - current correlation floor is explicit:
     - `candidate_result_id`
     - `publication_id`
   - `commit_attempt_id`, `reject_record_id`, and `fence_snapshot_ref` remain explicit current absences in the first TreeCalc lane
   - current emitted runtime-derived families are only:
     - `DynamicDependency`
     - `ExecutionRestriction`
   - `CapabilitySensitive` remains admitted but currently unexercised
   - those emitted families are now directly reachable on `OxCalcTreeRecalcResult` and in emitted `result.json` / `explain.json`
3. Sequence 3 publication/topology breadth:
   - `value_delta` is the only currently published consequence family on the first local TreeCalc floor
   - `shape_delta`, `topology_delta`, optional `format_delta`, and optional `display_delta` remain explicit current absences rather than silent members of `value_delta`
   - current publication sidecars are classified explicitly:
     - publish-critical now: `value_delta`
     - replay-visible but not publish-critical yet: `published_runtime_effects`, `trace_markers`
     - local-floor-only evidence: `dependency_shape_updates`
   - current execution-restriction observations remain runtime-effect-plus-typed-no-publish context rather than publication-sensitive or topology-sensitive consequences

Current non-overclaim remains:
1. all three residual lanes remain `canonical but narrower` beyond the executed first floor above,
2. the executed first floor is sufficient for continued TreeCalc seam intake and host-contract sync,
3. broader relative-reference closure, broader runtime-derived family realization, and broader publication/topology consequence breadth still require later evidence rather than inference.

### 21.2A Current V1 Public-Entry Read
After the landed OxFml consumer-interface refactor, the current local runtime-facing read is:
1. ordinary OxCalc runtime consumption should target `oxfml_core::consumer::runtime`,
2. ordinary OxCalc replay projection should target `oxfml_core::consumer::replay`,
3. the current minimal upstream-host packet in OxCalc is now realized on that public surface through `RuntimeEnvironment`, `RuntimeFormulaRequest`, `RuntimeFormulaResult`, and `ReplayProjectionService`,
4. direct parse and bind intake in the current TreeCalc dependency-preparation lane remains explicit local seam-consumption work under `W026` and is not yet being described as facade-only.

### 21.3 Consumed-now local narrowing for the remaining residuals
OxCalc is now treating the remaining residuals as bounded consume-now topics rather than general seam uncertainty.

For caller-anchor and address-mode carriage:
1. `caller_anchor`, formula-channel, address-mode, and structure-context identity remain explicit host-supplied inputs where relative or host-sensitive meaning depends on them,
2. the first TreeCalc subset should only consume relative forms whose contextual dependence is preserved honestly in the current bound/reference artifact,
3. OxCalc must not assume full relative-reference closure or one final frozen caller-sensitive transport shape.

For execution-restriction transport breadth:
1. execution-restriction observations are already consumed semantically as surfaced evaluator/runtime facts,
2. OxCalc may consume them through current candidate-result, reject-context, topology/effect-ref, or runtime-effect families where that truth is explicitly carried,
3. OxCalc must not collapse them into scheduler policy or assume one final single-object carrier yet.

For publication/topology breadth:
1. `value_delta`, `shape_delta`, and `topology_delta` remain distinct publish-facing categories,
2. optional `format_delta` and `display_delta` remain distinct when present,
3. OxCalc must not treat the currently exercised local breadth as closure of the full publication/topology universe,
4. later evidence rather than prose-only agreement should determine whether currently optional consequence families become first-slice mandatory.

The latest OxFml residual reply further sharpens this local narrowing:
1. all three residuals remain `canonical but narrower`,
2. the current consumed-now carrier set above is sufficient for continued W026 intake planning,
3. no new narrower handoff is justified from this residual pass alone,
4. the remaining pressure is broader closure beyond the first carried subset rather than a missing first-slice seam clause.

## 22. W050 Session-Shaped First-Call Protocol

### 22.1 Scope And Status
W050 replaces the old per-formula upstream-host packet shape with a
wave/session protocol over OxFml's public consumer runtime facade.

This section is an OxCalc-local protocol requirement for W050. It does
not freeze OxFml canonical names, and it does not authorize an OxCalc
adapter around OxFml internals. Canonical shared text changes route
through `HANDOFF_CALC_002_OXFML_RECALC_SESSION_AND_PLAN_TEMPLATES.md`.

Reviewed inbound observations:
1. `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md` currently says ordinary
   downstream use targets `consumer::runtime`, `consumer::editor`, and
   `consumer::replay`.
2. Current OxFml V1 runtime contract exposes `RuntimeEnvironment`,
   `RuntimeFormulaRequest`, `RuntimeFormulaResult`, and
   `RuntimeSessionFacade`.
3. Public `substrate::...` access is no longer an ordinary downstream
   contract.

### 22.1A W050 Consolidated Local Contract Snapshot

This snapshot is the durable W050-local reading of the session model after
CALC-002/CALC-003/CALC-004 receiving-repo feedback has been recorded.

| Surface | Current W050 local position | Canonicalization path |
|---|---|---|
| Formula source authority | OxCalc stores opaque formula text and identity; OxFml parses, projects red view, binds, compiles, evaluates, and reports returned-value surfaces. OxCalc must not parse formula language or reconstruct source as semantic input. | CALC-002 receipt accepts the public runtime/session facade as the ordinary surface and promotes a narrower successor plan for canonical prepared formula fields. |
| Session model | A recalc wave is modeled as wave preparation, `ensure_prepared`, trace-visible compilation, dependency derivation, scheduled invoke, coordinator commit, and close/capture. | OxFml runtime facade remains the consumed surface; richer compile/cache trace fields belong in OxFml if exposed. |
| Prepared formula identity | OxCalc consumes OxFml `RuntimePreparedFormulaIdentity` from the public managed prepare/runtime facade. TreeCalc no longer derives local prepared-callable compatibility keys from bound-formula/semantic-plan artifacts. | CALC-002 canonical identity now lands through OxFml runtime fields; remaining gaps are richer public package naming and any additional template-hole detail OxFml chooses to expose. |
| Bind-output reference mapping | Current TreeCalc maps local residual/reference carriers to dependency descriptors and rebind seeds using OxFml `RuntimeFormalReference` identity plus OxCalc structural truth. Runtime invocation supplies values through OxFml formal-input bindings rather than synthetic A1 cells. | CALC-002 reference/input transport has landed for TreeCalc scalar formal inputs. Richer formal-reference package fields and non-scalar carrier kinds remain OxFml-owned extension surfaces if needed. |
| Typed outcomes | OxCalc consumes value publication, verified-clean suppression, typed reject/no-publish, runtime effects, derivation trace, topic envelopes, and correctness-floor selector replay records as distinct surfaces. | CALC-002 receipt covers structured result/reject/correlation fields; CALC-003 receipt accepts OxFml `CorrectnessFloorContext` carriage, OxFunc selector semantics, and `semantic_kernel_metadata_version`; CALC-004 receipt reserves capability/hole identity, accepts `arg_admission_metadata_version`, and defers producer activation. OxCalc now threads both bridge versions into TreeCalc prepared identity and OxFml runtime input. |
| External invalidation | Subscription registry, topic envelopes, stream-semantics selectors, dirty seeds, and subscription lifecycle diagnostics are OxCalc-local repository behavior. Evaluation remains pull; external signals only seed invalidation. | Future OxFml/OxFunc feedback may refine RTD/registered-external topic descriptors but does not move publication authority. |
| Optimization boundary | OxCalc may cache by public identity keys and recorded hole-binding fingerprints. It must not synthesize common-subexpression reuse by rewriting formulas, inferring algebraic equivalence, or inspecting OxFunc kernels. | Pure-kernel CSE, folded subplans, and internal expression ids are OxFml/OxFunc-owned trace/artifact surfaces if exposed later. |

Inbound observation and handoff references for this snapshot:

1. `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md` identifies
   `oxfml_core::consumer::{runtime, replay}` as the intended consumed runtime
   and replay surface.
2. `docs/handoffs/HANDOFF_CALC_002_OXFML_RECALC_SESSION_AND_PLAN_TEMPLATES.md`
   carries the prepared-callable/session and plan-template asks;
   `docs/handoffs/HANDOFF_CALC_002_OXFML_RECEIPT.md` records OxFml's
   narrower successor contract response.
3. `docs/handoffs/HANDOFF_CALC_003_OXFML_NUMERICAL_REDUCTION_AND_ERROR_ALGEBRA.md`
   and its OxFunc note carry correctness-floor selector threading and kernel
   obligations; `docs/handoffs/HANDOFF_CALC_003_OXFML_RECEIPT.md` records the
   OxFml/OxFunc ownership split.
4. `docs/handoffs/HANDOFF_CALC_004_OXFML_CAPABILITY_SET_HOLE_ADMISSION.md`
   and its OxFunc note carry sparse/rich hole and capability-set trace asks;
   `docs/handoffs/HANDOFF_CALC_004_OXFML_RECEIPT.md` records OxFml's
   identity-reservation response.

Replay and evidence obligations:

1. Every W050 behavior claim must point either to checked runtime tests, a
   checked evidence root under `docs/test-runs/core-engine`, or an explicit
   handoff/watch dependency.
2. Retired compatibility-only fields must not be reintroduced as active
   TreeCalc identity surfaces; successor detail belongs in OxFml-owned
   runtime/replay fields.
3. Spec text without replay, trace, or test evidence remains planning text; it
   does not close a W050 behavior lane.

### 22.2 First-Call Inputs
For a formula-bearing OxCalc node, the first call into OxFml is not a
synthetic workbook fixture. It is a request to prepare one opaque formula
slot in a pinned recalc session.

The minimum input families are:
1. Formula slot identity:
   - `formula_stable_id`
   - `formula_text_version`
   - formula source text as an opaque string owned semantically by OxFml
   - `formula_token` or the OxFml-derived equivalent when available
2. Caller and structure context:
   - `caller_anchor`
   - formula channel and address-mode context
   - `structure_context_version`
   - table/name/library context identities supplied by host/OxCalc-owned
     structural truth
3. Session-wide runtime context:
   - pinned `LibraryContextSnapshotRef` or inline snapshot through the
     OxFml runtime facade
   - typed query bundle/provider availability
   - profile selectors relevant to evaluation and replay, including
     `NumericalReductionPolicy` where numerical reductions are observable and
     `ErrorAlgebra` where multiple worksheet-error candidates can collapse to
     one observable result
4. Reference/value environment:
   - current published or working values for already-derived structural
     targets at invocation time, supplied through OxFml
     `RuntimeFormalInputBinding` records
   - OxFml `RuntimeFormalReference` handles/descriptors used when the
     prepared identity is available
   - no synthetic A1 address generation as the dependency identity layer
   - no OxCalc-authored formula AST lowered into Excel source

The OxCalc-owned Calculation Repository is the persistent home for these
inputs between waves. It owns structural snapshot identity, formula slot
identity, dependency graph state derived from OxFml bind outputs, per-node
calculation state, runtime overlays, pinned reader views, published value
view, and typed handles to OxFml artifacts. It does not own OxFml artifact
meaning or formula-language semantics.

### 22.3 Prepare And Bind Contract
W050 names the preparation operation `ensure_prepared`.

Conceptually:

```text
ensure_prepared(formula_stable_id, formula_text_version, source, caller_context)
  -> RuntimePreparedFormulaIdentity | BindResult
```

Required semantics:
1. OxFml parses, projects, binds, and compiles semantic plan artifacts.
2. OxFml owns diagnostics, unresolved-reference classification, formula
   grammar, operator/function meaning, callable meaning, and returned
   value surfaces.
3. OxFml returns or exposes a prepared callable identity with:
   - parse/red/bind/semantic-plan identity handles already present in the
     current V1 vocabulary where available,
   - W050 plan-template identities when CALC-002 support lands:
     `shape_key`, `dispatch_skeleton_key`, `plan_template_key`,
   - a normalized formal reference set,
   - capability requirements,
   - runtime-effect classification.
4. OxCalc maps returned normalized reference facts to structural targets
   for dependency graph construction. It does not pre-resolve references
   into synthetic defined names as the semantic truth.

Unresolved, host-sensitive, dynamic, capability-sensitive, and
shape/topology-sensitive facts are typed OxFml/evaluator facts consumed by
OxCalc. They are not failures of the protocol and they are not permission
for OxCalc to interpret formula text.

### 22.4 Invocation Contract
W050 names the evaluation operation `invoke`.

Conceptually:

```text
invoke(call_site, reference_bindings) -> InvocationOutcome
```

Required semantics:
1. A call site binds a prepared callable handle to the current caller
   context and current reference/value inputs.
2. OxFml evaluates through its runtime facade and returns a typed outcome:
   accepted candidate, rejected/no-publish detail, dependency/topology
   update, callable value, returned value surface, and trace/replay facts
   where available.
3. OxCalc remains the single publication authority. An accepted candidate
   is not a published value until the coordinator accepts the candidate
   against current fences and publishes one atomic derived bundle.
4. Reject is no-publish. Reject detail must remain typed and replay-visible.
5. Dynamic dependency updates and external/runtime effects enter the
   dependency and invalidation machinery as typed facts, not as inferred
   formula semantics.

### 22.5 Seven-Phase Wave Shape
The session-shaped first-call protocol is exercised inside a recalc wave:
1. wave preparation: open/pin the OxFml runtime session and compute dirty
   closure,
2. ensure prepared: prepare dirty formula slots and cache prepared callable
   identity,
3. compilation: record the trace-visible compile/prepare result identity and
   diagnostics that came back from the OxFml runtime session,
4. dependency derivation: map OxFml normalized references and runtime facts
   to OxCalc dependency graph state,
5. schedule and invoke: execute call sites in deterministic Stage 1 order,
6. coordinator commit: accept or reject candidates under OxCalc publication
   fences,
7. close and capture: emit replay/trace/evidence inputs without mutating
   tracked baselines during validation.

### 22.6 Current V1 OxCalc Driver Mapping
The current OxCalc B3 uptake introduces
`OxfmlRecalcSessionDriver` as an OxCalc-owned driver over public
`oxfml_core::consumer::runtime` types only.

Current mapping:
1. `ensure_prepared` maps to
   `RuntimeSessionFacade::open_managed_session`.
2. `invoke` maps to `RuntimeSessionFacade::execute`, because the current
   TreeCalc publication path still needs the full `RuntimeFormulaResult`
   surface: returned-value classification, candidate result, commit
   decision, trace events, replay capture, and artifact reuse report.
3. `invoke_managed_commit` maps to
   `RuntimeSessionFacade::execute_and_commit_managed` and is exercised as
   V1 compatibility evidence, but it is not yet sufficient as the sole
   TreeCalc invocation result because `RuntimeManagedCommitResult` does
   not carry every full runtime-result surface OxCalc currently consumes.
4. The deterministic upstream-host packet path now invokes through the
   session driver rather than directly through
   `RuntimeEnvironment::execute`.

The remaining gap is routed to `HANDOFF_CALC_002`: OxFml should either
confirm that the managed-session result is intended to grow the missing
full-result surfaces, or provide the canonical prepared-callable invocation
surface that preserves them.

### 22.7 Current V1 Wave Lifecycle Mapping
The current OxCalc B4 uptake introduces `OxfmlRecalcWave`, an
OxCalc-owned lifecycle runner over the B3 session driver.

Current mapping:
1. `WavePreparation` opens the OxCalc wave trace and pins the repository
   authority for the wave.
2. `EnsurePrepared` invokes `OxfmlRecalcSessionDriver::ensure_prepared`
   and records OxFml runtime-session authority.
3. `Compilation` records the OxFml runtime-session compile/prepare result
   as a distinct trace phase.
4. `DependencyDerivation` records the OxCalc repository/dependency-graph
   phase; canonical bind-output reference replacement remains Lane B5.
5. `ScheduleInvoke` invokes `OxfmlRecalcSessionDriver::invoke` after the
   dependency phase.
6. `CoordinatorCommit` records exactly one OxCalc coordinator decision:
   either a `PublicationBundle` or no-publish reject detail. The wave
   runner does not create publications.
7. `CloseCapture` emits the replay/capture trace and seals the wave.

The B4 trace enforces monotonic phase order and rejects skipped phases.
Semantic equivalence statement: this Stage 1 lifecycle surface is an
ordering and authority guard around the existing sequential strategy; it
does not change formula results, candidate values, coordinator fences, or
publication authority for any currently exercised profile.

### 22.8 Current V1 Bind-Output Reference Mapping
The current OxCalc B5 uptake adds a source-reference handle to every
`DependencyDescriptor` that can be tied to OxFml bind output or runtime
facts.

Current mapping:
1. Direct and rebind-sensitive structural references are derived from
   `BoundFormula.normalized_references` where current V1 exposes a
   `NormalizedReference::Name` corresponding to the current migration
   carrier.
2. Unresolved references are derived from
   `BoundFormula.unresolved_references`.
3. Host-sensitive, dynamic-potential, capability-sensitive, and
   shape/topology-sensitive carriers remain runtime facts and carry
   `runtime_fact:*` handles rather than pretending to be static
   dependency edges.
4. The dependency graph preserves the source-reference handle alongside
   the OxCalc target mapping, so mapping evidence no longer depends on an
   A1 address-string round trip.

Current B5 landing: current TreeCalc scalar formula inputs use OxFml formal
input bindings instead of synthetic `cell_values` and defined-name reference
fixtures. Dependency graph ownership still belongs to OxCalc structural
truth, with OxFml formal-reference handles preserved for identity and replay
correlation.

### 22.9 Current V1 TreeCalc Production Invocation Mapping
The current OxCalc B7 uptake removes the TreeCalc production evaluation
dependency on `MinimalUpstreamHostPacket`. `LocalTreeCalcEngine` now
enters OxFml through `OxfmlRecalcSessionDriver::invoke` after assembling
the public V1 runtime inputs directly from the prepared formula and the
current in-wave value map.

Current mapping:
1. `RuntimeFormulaRequest` uses the existing `PreparedOxfmlFormula`
   `FormulaSourceRecord` and `EvaluationBackend::OxFuncBacked`.
2. `RuntimeEnvironment` uses the prepared formula identity's
   `structure_context_version`, the TreeCalc owner node as caller
   position, and translated reference carriers as
   `RuntimeFormalInputBinding` entries. Prepare-time bindings provide the
   source-visible descriptor; invoke-time bindings include the matching
   `RuntimeFormalReference` handle when OxFml has exposed it.
3. Host-sensitive residuals are represented by a TreeCalc-local
   `HostInfoProvider` that preserves the current deterministic
   `treecalc.host_sensitive_reference` provider-failure outcome for
   `INFO` queries.
4. Dynamic-potential residuals are represented by a TreeCalc-local
   `RtdProvider` that returns `CapabilityDenied`, preserving the current
   no-publish dynamic-dependency failure path.
5. `MinimalUpstreamHostPacket` and its `Minimal*` facts remain only in
   deterministic upstream-host fixture/scaffolding surfaces
   (`upstream_host.rs`, `upstream_host_fixture.rs`, integration tests,
   and runner evidence). They are no longer used by the TreeCalc
   production recalc path.

Current B7 landing: TreeCalc production invocation no longer maps structural
targets through synthetic A1 `cell_values` or defined-name reference
bindings. Remaining formal-input extension work is limited to richer
OxFml-owned carrier kinds if a future formula surface needs more than scalar
value transport.

Semantic equivalence statement: B7 changes how TreeCalc constructs the
current V1 OxFml call, not the observable recalc strategy. The same
formula source, structure-context version, working values, reference
targets, host-sensitive provider-failure outcome, dynamic-potential RTD
denial, `RuntimeFormulaResult` adaptation, and OxCalc single-publisher
coordinator authority are preserved for the currently exercised Stage 1
profiles.

### 22.10 Current V1 Opaque Formula Source Mapping
The current OxCalc A3 uptake removes `TreeFormula` as a semantic AST.
TreeCalc production formula input is now an opaque OxFml source string
plus explicit reference/evaluator-fact carriers.

Current mapping:
1. `TreeFormula.source_text` is passed into `FormulaSourceRecord` without
   OxCalc semantic rewriting.
2. `TreeFormulaReferenceCarrier` records the source token, when one
   exists, and the OxCalc structural/evaluator fact carrier that maps the
   source token to dependency graph truth.
3. `project_opaque_formula` projects those carriers into the current V1
   `BindContext.names`, unresolved bindings, and runtime residual facts.
4. Legacy source construction is quarantined as `FixtureFormulaAst` for
   checked-in TreeCalc fixture JSON, unit tests, and the procedural scale
   runner. That quarantine renders fixture AST values into opaque
   `TreeFormula` values before they reach production recalc.
5. The previous `translate_formula` / `TranslationState` path and
   `formula_allows_lazy_residual_publication` special-case function are
   no longer TreeCalc source-code surfaces.

Current A3 limit: the fixture quarantine still renders formula source for
legacy corpora, and current V1 still uses synthetic source tokens plus A1
compatibility inputs. CALC-002 owns the canonical prepared-callable input
transport that can retire those compatibility carriers.

Semantic equivalence statement: A3 changes the local representation of
TreeCalc formula input, not the current Stage 1 recalc semantics. The
fixture quarantine renders the same formula source strings and carrier
sequence used before A3, and production recalc still feeds the same source
text, source tokens, dependency carriers, residual facts, session
invocation path, and OxCalc coordinator publication authority to the
currently exercised profiles.

### 22.11 Current V1 Plan-Template Identity Mapping
The current OxCalc C1 uptake derives W050 identity keys from public OxFml
bound and semantic-plan artifacts during TreeCalc preparation.

Current mapping:
1. `shape_key` has prefix `shape:v1:` and fingerprints the public
   `BoundFormula.root` shape with source leaves abstracted. Literal
   values, concrete reference coordinates/targets, and function surface
   names are holes; operator nesting, call arity, lazy-control posture,
   helper/lambda parameter slots, root grouping, reference class, range
   extent, address-mode posture, and caller-context dependence remain in
   the key input.
2. `dispatch_skeleton_key` has prefix `dispatch_skeleton:v1:` and
   fingerprints `shape_key`, the OxFunc catalog identity, the library
   context snapshot reference, public `FunctionPlanBinding` dispatch
   records, and function availability summaries.
3. `plan_template_key` has prefix `plan_template:v1:` and fingerprints
   `dispatch_skeleton_key`, semantic-plan locale/date/format profiles,
   evaluation requirements, execution profile, helper profile, capability
   requirements, and semantic diagnostic categories.
4. `PreparedOxfmlFormula` carries the derived keys. TreeCalc exposes them
   through `LocalTreeCalcRunArtifacts.prepared_formula_identities`, the
   runner `result.json` surface, diagnostics, and trace events labelled
   `prepared_formula_identity`.
5. The current V1 key inputs intentionally exclude `formula_stable_id`,
   formula token, source text, literal values, concrete reference
   coordinates/targets, `bind_hash`, and OxFml `semantic_plan_key`; those
   remain formula-instance or current-artifact identities rather than
   plan-template identities.

Current W050 landing: the OxCalc-side compatibility fingerprint layer has
been retired from TreeCalc. Current TreeCalc identity traces consume OxFml
`RuntimePreparedFormulaIdentity` fields (`prepared_formula_key`,
`plan_template`, `hole_binding`, `formal_references`, and the OxFunc bridge
metadata versions). OxFml remains owner of richer package naming and any
additional template-hole detail beyond the current runtime identity fields.

Semantic equivalence statement: C1 changes identity derivation and trace
projection only. It does not change formula source, bind context, runtime
environment, invocation path, candidate adaptation, rejection policy, or
OxCalc coordinator publication authority for any currently exercised
Stage 1 profile.

### 22.12 Current Runtime Prepared Formula Artifact Mapping
The current OxCalc C2 landing consumes OxFml runtime prepared identity instead
of deriving an OxCalc-local prepared-callable artifact.

Current mapping:
1. TreeCalc calls OxFml managed prepare and records
   `RuntimePreparedFormulaIdentity`.
2. `PreparedFormulaIdentityTrace.prepared_formula_key` is copied from
   OxFml `prepared_formula_key`.
3. `plan_template_key`, `dispatch_skeleton_key`, `shape_key` when present,
   `hole_binding_fingerprint`, and `template_hole_count` are copied from
   OxFml runtime identity fields rather than recomputed locally.
4. Formal-reference dependency descriptors are driven by OxFml
   `RuntimeFormalReference` handles/descriptors plus OxCalc structural truth
   for host mapping.
5. TreeCalc no longer exposes an OxCalc-local `PreparedCallable`,
   `prepared_callable_key`, or compatibility hole taxonomy as an active
   runtime artifact.

Current C2/C3 limit: OxFml's current public runtime identity does not yet
surface all former OxCalc compatibility template-hole detail, and its current
template key is not claimed by OxCalc as cross-call-site reuse evidence.
That detail is OxFml-owned successor surface, not an OxCalc projection.

Semantic equivalence statement: C2 changes artifact shape and trace
projection only. It does not change source parsing, binding, semantic-plan
compilation, runtime invocation, candidate adaptation, rejection policy, or
OxCalc coordinator publication authority for any currently exercised
Stage 1 profile.

### 22.13 Current V1 Default Hole-Type Taxonomy Mapping
The current OxCalc C3 uptake maps the W050 default hole taxonomy into the
prepared-call identity model and keeps the policy wide by default.

Current mapping:
1. `PlanTemplateHoleKind` has stable-keyed variants `ValueHole`,
   `RefOrValueHole`, `CallableHole`, `ShapeSensitiveHole`,
   `SparseRangeHole`, and `RichValueHole`.
2. `ValueHole(AnyValue)` is emitted for arguments whose OxFunc
   `ArgPreparationProfile` is `ValuesOnlyPreAdapter`.
3. `RefOrValueHole(ReferenceIdentityVisible)` is emitted for arguments
   whose OxFunc `ArgPreparationProfile` is `RefsVisibleInAdapter`.
4. Invocation callees are represented as `CallableHole(AnyCallable)` in
   the template skeleton; their concrete callee payload remains in
   `HoleBindings`.
5. Literal values, concrete references, omitted arguments, and helper names
   remain binding payloads. They do not narrow the template hole kind unless
   future evidence gates an explicit narrower producer.
6. `SparseRangeHole` and `RichValueHole` are representable and stable-keyed
   but are not emitted by the current V1 production path. No current kernel
   claim is made for sparse range readers or rich-value capability
   consumption.

Semantic equivalence statement: C3 changes template-hole classification and
fingerprint inputs only. It does not change source parsing, binding,
semantic-plan compilation, runtime invocation, candidate adaptation,
rejection policy, scheduling strategy, or OxCalc coordinator publication
authority for any currently exercised Stage 1 profile. Observable formula
results are invariant under this identity-taxonomy change for those
profiles.

### 22.14 Current V1 ArgPreparationProfile Invalidation Mapping
The current OxCalc C4 uptake treats OxFunc `ArgPreparationProfile` changes
as bind-visible name-world changes without claiming ownership of OxFunc
metadata.

Current mapping:
1. `LocalTreeCalcEnvironmentContext` carries an
   `arg_preparation_profile_version` value representing the pinned
   bind-visible OxFunc argument-preparation metadata snapshot.
2. TreeCalc formula preparation threads that version into the OxFml
   `StructureContextVersion`, together with the structural snapshot id.
   A changed profile version therefore produces a distinct bind-visible
   context.
3. OxCalc derives `StructuralRebindRequired` invalidation seeds for formula
   owners when the previous and next `arg_preparation_profile_version`
   differ.
4. Current V1 invalidation is conservative: without an OxFml/OxFunc public
   affected-callable surface, the profile-version transition marks every
   TreeCalc formula owner for rebind rather than attempting private
   function-use inspection.
5. Tests cover the structure-context-version change, the rebind-seed
   derivation, and the runtime rebind rejection path.

Current C4 limit: targeted invalidation for only formulas that call changed
functions remains an OxFml/OxFunc surface gap. OxCalc records the
version-change contract locally and avoids under-invalidation.

Current bridge uptake: after OxFunc/OxFml accepted
`semantic_kernel_metadata_version` and `arg_admission_metadata_version`, OxCalc
threads both values through TreeCalc structure-context material, compatibility
prepared-callable key material, OxFml `RuntimeEnvironment`, and prepared
identity diagnostics/artifacts. The bridge prevents local stale prepared
formula reuse across sibling metadata version changes. It does not claim
selector enforcement, rich-argument admission, producer capability emission, or
sparse reader behavior.

Current capability uptake: TreeCalc consumes OxFml returned-value
`producer_capability_set_keys` for the first `IMAGE` / `_webimage` rich carrier
and records those keys in diagnostics and derivation trace capability columns.
This is producer-capability evidence only. It does not claim rich-argument
admission, exercised capability-key emission, broad rich producer protocol, or
sparse reader behavior.

Semantic equivalence statement: C4 changes bind-visible invalidation and
structure-context identity only. It does not change source parsing,
binding semantics, semantic-plan compilation, runtime invocation, candidate
adaptation, rejection policy, scheduling strategy, or OxCalc coordinator
publication authority for any currently exercised Stage 1 profile. When the
profile version is unchanged, observable formula results are invariant.
When it changes, the conservative rebind gate prevents stale prepared
callables from publishing.

### 22.15 Current V1 Plan-Template Reuse Trace-Count Evidence
The current OxCalc C5 uptake adds deterministic trace-count evidence for
the plan-template identity split.

Current mapping:
1. TreeCalc diagnostics include
   `oxfml_plan_template_reuse_count:{plan_template_key}:call_sites=N;prepared_formulas=M;hole_bindings=K`
   only when the OxFml-owned runtime identity reports the same
   `plan_template_key` at more than one call site.
2. The counter groups current `PreparedFormulaIdentityTrace` records by
   `plan_template_key` and counts call sites, distinct
   `prepared_formula_key` values, and distinct `hole_binding_fingerprint`
   values.
3. After retirement of the OxCalc compatibility projection, the
   deterministic C5 test runs two `SUM` call sites, observes distinct OxFml
   runtime `plan_template_key` values, and verifies OxCalc does not
   synthesize a reuse count.
4. Cross-call-site template reuse is now an OxFml-owned evidence question.

Semantic equivalence statement: C5 adds reuse-count diagnostics and tests
only. It does not change source parsing, binding, semantic-plan
compilation, runtime invocation, candidate adaptation, rejection policy,
scheduling strategy, or OxCalc coordinator publication authority for any
currently exercised Stage 1 profile. Observable formula results are
invariant; the C5 evidence specifically checks distinct published values
while preventing an OxCalc-local reuse overclaim.

### 22.16 Current V1 Compile-Time Folding Identity Boundary
The current OxCalc C6 uptake records the compile-time constant-folding
boundary for plan-template identity.

Current mapping:
1. Current V1 `plan_template_key` derivation uses public OxFml parse, bind,
   and semantic-plan artifacts only.
2. No public OxFml consumer/runtime surface currently identifies a
   compile-time folded constant, canonical folded plan form, or folding
   trace suitable for OxCalc plan-template identity.
3. OxCalc does not constant-fold formula source text and does not move
   OxFunc function semantics into OxCalc to infer folded plans.
4. The deterministic C6 test records the current boundary: `=2+3*4` and
   `=14` remain distinct `shape_key` and `plan_template_key` values in the
   current V1 compatibility identity layer.

Current C6 gaps:
1. `HANDOFF-CALC-002` must ask OxFml for canonical plan-template fields
   that already reflect any OxFml-owned folded plan form, if folding is
   supported.
2. `HANDOFF-CALC-004` remains the relevant capability/hole-admission
   packet for future evidence-gated narrowing producers such as
   `ConstNumericHole`; such narrowings must enter `plan_template_key` only
   through an OxFml/OxFunc-owned producer contract.

Semantic equivalence statement: C6 adds boundary evidence, spec text, and
upstream notes only. It does not change source parsing, binding,
semantic-plan compilation, runtime invocation, candidate adaptation,
rejection policy, scheduling strategy, or OxCalc coordinator publication
authority for any currently exercised Stage 1 profile. Observable formula
results are invariant.

### 22.17 Current V1 Derivation Trace Invoke Outcome
The current OxCalc F3 uptake exposes derivation traces as an opt-in TreeCalc
invoke outcome over the public OxFml `PreparedCalls` trace mode.

Current mapping:
1. `LocalTreeCalcEnvironmentContext.derivation_trace_enabled` and
   `OxCalcTreeRuntimePolicy.derivation_trace_enabled` default to `false`.
2. When disabled, TreeCalc sends the default OxFml trace mode and emits no
   `DerivationTraceRecord`.
3. When enabled, TreeCalc sends `EvaluationTraceMode::PreparedCalls` through
   `RuntimeFormulaRequest::with_trace_mode` and records
   `DerivationTraceRecord` in `LocalTreeCalcRunArtifacts` and
   `OxCalcTreeRecalcResult`.
4. The record carries schema id
   `oxcalc.derivation_trace.invoke_outcome.v1`, owner/formula identity,
   template selection (`prepared_formula_key`, `shape_key`,
   `dispatch_skeleton_key`, `plan_template_key`, template holes when exposed
   by OxFml), hole-binding identity, a root prepared-formula invocation,
   child OxFml prepared-call entries, per-call `kernel_returned_value`,
   final returned value, and OxFml seam trace event summaries.
5. TreeCalc runner result/trace artifacts surface the same record when the
   opt-in flag is enabled.
6. Checked evidence is
   `docs/test-runs/core-engine/w050-f3-derivation-trace-invoke-outcome-001/run_artifact.json`.

Current F3 limit: the child invocation hierarchy reflects the current OxFml
prepared-call trace granularity. OxCalc records a root formula invocation with
ordered child prepared calls; deeper nesting remains an OxFml trace-schema
extension candidate rather than an OxCalc-private reconstruction of formula
semantics.

Semantic equivalence statement: F3 adds an opt-in observation surface and
does not change source parsing, binding, semantic-plan compilation,
evaluation, candidate adaptation, rejection policy, scheduling strategy, or
OxCalc coordinator publication authority. Observable formula results and
publication behavior are invariant between value-only and trace-mode runs
for the exercised Stage 1 profile.

### 22.18 Current V1 Compilation Trace and Pure-Kernel CSE Boundary
The current OxCalc F6 uptake records compilation as an observable recalc-wave
phase while preserving OxFml/OxFunc ownership of formula-language and
pure-kernel optimization semantics.

Current mapping:
1. `OxfmlRecalcWave::ensure_prepared` emits `EnsurePrepared`, delegates to the
   OxFml managed runtime session, then emits a trace-visible `Compilation`
   phase for the prepared formula.
2. The `Compilation` event is authoritative to `OxFmlRuntimeSession` and
   records formula stable id, session id, library-context snapshot reference,
   syntax/bind/semantic diagnostic counts, and semantic-plan function-binding
   count.
3. The recalc-wave phase order is now wave preparation -> ensure prepared ->
   compilation -> dependency derivation -> scheduled invoke -> coordinator
   commit -> close/capture. Multiple ensure/compilation pairs may occur before
   dependency derivation.
4. Pure-kernel common-subexpression reuse remains an OxFml/OxFunc-owned
   optimization boundary. OxCalc may group or cache only replay-visible
   artifacts it receives through public surfaces: prepared-callable identity,
   plan-template identity, hole-binding fingerprints, and per-edge value-cache
   keys. OxCalc must not rewrite formulas, infer algebraic equivalence, or
   inspect OxFunc kernel internals to synthesize CSE.

Successor / handoff note:
1. CALC-002 should expose canonical compilation trace fields if OxFml wants to
   publish richer compile/cache events than the current managed-session output.
2. Any pure-kernel CSE counters, folded subplans, or reusable internal
   expression ids must come from OxFml/OxFunc-owned trace or artifact surfaces;
   OxCalc's current role is to record the boundary and consume public identity
   keys.

Semantic equivalence statement: F6 adds trace observability and boundary text
only. It does not change source parsing, binding, semantic-plan compilation,
runtime invocation, candidate adaptation, rejection policy, scheduling
strategy, pure-kernel behavior, or OxCalc coordinator publication authority.
Observable formula results are invariant for the exercised Stage 1 profile.

### 22.19 CALC-002 Handoff Inputs And Receipt
`HANDOFF_CALC_002` must ask OxFml for canonical support or confirmation for:
1. prepared-callable identity surfaced through the public consumer runtime
   path,
2. plan-template identity fields:
   - `shape_key`
   - `dispatch_skeleton_key`
   - `plan_template_key`
3. a formal reference set suitable for OxCalc structural mapping,
4. capability requirement and runtime-effect classification surfaces,
5. stable invocation outcome categories for accepted candidate, reject,
   topology/dependency update, callable value, returned-value surface, and
   trace/replay correlation,
6. trace/replay columns for template identity and hole/capability identity,
7. an explicit migration table from current `RuntimeEnvironment` /
   `RuntimeFormulaRequest` / `RuntimeFormulaResult` /
   `RuntimeSessionFacade` fields to the W050 prepared-callable/session
   categories.
8. the B3-observed managed-session output gap: current
   `RuntimeManagedCommitResult` does not carry all full
   `RuntimeFormulaResult` surfaces consumed by OxCalc's coordinator path.
9. the B7 formal-input bridge: current TreeCalc production invocation maps
   structural scalar values through OxFml `RuntimeFormalInputBinding`
   records and validates them against `RuntimeFormalReference` handles when
   available. No synthetic A1 `cell_values` or defined-name reference
   fixtures remain in the TreeCalc invocation path.
10. the A3-observed source/carrier compatibility bridge: current TreeCalc
    opaque formula input still needs source-token carriers until OxFml
    exposes richer prepared formula input bindings and formal
    reference identities.
11. the retired C1/C2 identity compatibility bridge: OxCalc no longer
    derives local `shape_key`, `dispatch_skeleton_key`,
    `plan_template_key`, or prepared-callable keys from bound/semantic-plan
    artifacts. It consumes OxFml `RuntimePreparedFormulaIdentity`.
12. the C2 successor artifact-shape bridge: OxFml may expose richer
    canonical hole ids, hole kinds, hole-binding fingerprints, and package
    fields; OxCalc must not synthesize them as a shared seam.
13. the C3-observed hole-taxonomy bridge: OxFml should confirm the
    canonical naming, stable serialization, and emission rules for
    `ValueHole`, `RefOrValueHole`, `CallableHole`, `ShapeSensitiveHole`,
    `SparseRangeHole`, and `RichValueHole`, including the wide-by-default
    `ArgPreparationProfile` mapping and the absence of sparse/rich kernel
    claims until OxFunc exposes such producers.
14. the C4-observed metadata-invalidation bridge: OxFml/OxFunc should
    expose a canonical bind-visible `ArgPreparationProfile` metadata
    version and, if narrower invalidation is required, an affected-callable
    or affected-function surface that lets OxCalc avoid conservative
    all-formula rebind when that version changes.
15. the C5-observed reuse-count bridge: OxFml should eventually expose
    canonical plan-template reuse/cache counters or stable trace fields so
    OxCalc does not need compatibility-only grouping over locally derived
    `PreparedFormulaIdentityTrace` records.
16. the C6-observed compile-time folding bridge: if OxFml supports
    compile-time folding, canonical `PlanTemplate` and semantic-plan
    identity fields should expose the folded plan form or stable folding
    trace so OxCalc does not infer folding from source text.
17. the B6-observed opaque-result bridge: canonical returned-value,
    callable-value, spill/rich-value, dynamic-reference, and typed provider
    outcome categories should remain structured through the consumer
    runtime facade. Current V1 exposes enough surface for TreeCalc
    diagnostics and scalar/array publication, but returned callable values
    are still converted to worksheet fallback `Calc`/`Error(Calc)` before
    OxCalc receives a stable callable payload.
18. the B8-observed session-evidence bridge: current V1 lets OxCalc record
    candidate, commit, reject, returned-value-surface, and replay-facing
    correlation facts in deterministic run artifacts, but some facts still
    arrive as diagnostic strings rather than stable structured facade
    fields. OxFml should preserve canonical correlation columns for
    `candidate_result_id`, `commit_attempt_id`, reject trace correlation,
    returned-value surface classification, and replay diagnostics.
19. the B9-observed facade compatibility ledger: current OxCalc composes
    with the frozen public V1 consumer runtime facade through
    `RuntimeEnvironment`, `RuntimeFormulaRequest`, `RuntimeFormulaResult`,
    and `RuntimeSessionFacade`, so OxCalc should not add private adapters
    around OxFml internals. CALC-002 should still define which current
    compatibility derivations migrate into canonical public fields:
    prepared-callable and `PlanTemplate` identity, hole bindings, reference
    and input transport, full managed-result surfaces, stable
    replay/correlation columns, callable/rich/spill returned-value payloads,
    plan-reuse and folding trace fields, and targeted metadata-invalidation
    fields.
20. the F3-observed derivation-trace bridge: current OxFml
    `PreparedCalls` trace mode gives OxCalc ordered prepared-call entries and
    returned values, but not a deeper nested invocation tree. OxFml should
    confirm whether the canonical trace schema will expose parent/child
    invocation structure, or whether the root-plus-ordered-children V1 shape
    remains the intended public granularity.

That handoff is now acknowledged by
`docs/handoffs/HANDOFF_CALC_002_OXFML_RECEIPT.md`. OxCalc may continue to
prototype only against the current public V1 runtime facade until OxFml emits
canonical runtime/replay fields for `PreparedFormulaPackage`, `PlanTemplate`,
`HoleBindingSet`, and `FormalReference` transport. It must not add a
long-lived private seam or adapter that assumes OxFml internals will remain
accessible.

### 22.20 Current V1 Opaque Result Family Coverage
The current OxCalc B6 uptake records which result families TreeCalc can
consume through `OxfmlRecalcSessionDriver::invoke` without local formula
parsing or reconstruction.

Current mapping:
1. TreeCalc diagnostics now carry
   `oxfml_returned_value_surface_kind:*`,
   `oxfml_returned_value_surface_payload_summary:*`, and provider-outcome
   fields when OxFml supplies them.
2. Literal and function-call ordinary values are exercised by `=14` and
   `=SUM(2,3)` and publish through the OxCalc coordinator as scalar
   values.
3. LET/LAMBDA invocation is exercised by
   `=LET(base,2,LAMBDA(delta,base+delta)(5))` and publishes as an
   ordinary scalar value.
4. Dynamic-array/spill-like output is exercised by `=SEQUENCE(3)`; current
   V1 publishes the TreeCalc node value as the opaque summary
   `Array(3x1)` and records the returned surface payload summary as
   `Array(3x1)`.
5. Returned callable value is exercised by `=LAMBDA(x,x+1)`, but current
   V1 host publication converts it to worksheet fallback `Calc` and
   returned surface `Error(Calc)`. This is boundary evidence, not a
   callable payload transport claim.
6. Dynamic `INDIRECT` facts are exercised by
   `=INDIRECT(RTD("TREECALC","","carrier:indirect"))` with an explicit
   `DynamicPotential` carrier. TreeCalc rejects the run through the
   dynamic-dependency effect path while preserving the OxFml returned
   surface diagnostic `Error(Blocked)`.
7. Direct RTD/external provider outcome is exercised by
   `=RTD("TREECALC","","carrier:rtd")` with an explicit
   `DynamicPotential` carrier. TreeCalc rejects the run through the
   dynamic-dependency effect path while preserving the typed provider
   diagnostics `TypedHostProviderOutcome`, `CapabilityDenied`, and
   worksheet error `Blocked`.

Current B6 gaps:
1. Canonical callable value transport remains unavailable to TreeCalc
   through current V1 publication; CALC-002 owns the stable callable
   payload/result category.
2. Current dynamic-array publication is an opaque single-node summary
   (`Array(3x1)`), not a full spill-grid publication contract.
3. Direct RTD coverage uses the deterministic current TreeCalc provider
   shim (`CapabilityDenied`). Subscription registry and topic envelope
   semantics remain Lane D work.
4. Registered external providers beyond RTD are not exercised in B6 and
   remain later Lane D/G seam pressure.

Semantic equivalence statement: B6 adds diagnostics, tests, and boundary
notes only. It does not change source parsing, binding, semantic-plan
compilation, runtime invocation, candidate adaptation, residual rejection
policy, scheduling strategy, or OxCalc coordinator publication authority
for any currently exercised Stage 1 profile. Observable formula results
are invariant under this diagnostic addition.

### 22.21 Current V1 Session Corpus Evidence Packet
The current OxCalc B8 uptake makes the TreeCalc local runner emit a
deterministic session-path evidence packet for the full local corpus.

Current mapping:
1. `TreeCalcRunner` writes `session_path_evidence.json` at the TreeCalc
   local run root.
2. The packet schema is `oxcalc.treecalc.session_path_evidence.v1` and
   records:
   - declared artifact root,
   - checked-in-or-explicit-ephemeral evidence policy,
   - commands used to regenerate and validate the root,
   - one entry per initial and post-edit case execution.
3. Each entry records result/trace artifact paths, OxCalc candidate ids,
   OxFml candidate diagnostics, commit candidate ids, commit attempt ids,
   reject candidate ids, reject trace correlation ids, returned-value
   surface diagnostics, replay-facing diagnostics, and non-mutation checks.
4. The replay-appliance projection records
   `source_session_path_evidence_path`, preserves
   `session_correlation_keys` and `replay_facing_diagnostics`, and includes
   the packet in `bundle_validation.json` checked paths.
5. The checked-in B8 root is
   `docs/test-runs/core-engine/treecalc-local/w050-b8-treecalc-session-corpus-001`.
   It is generated with
   `cargo run -p oxcalc-tracecalc-cli -- treecalc w050-b8-treecalc-session-corpus-001`.

Current B8 limits:
1. `session_path_evidence.json` is an OxCalc deterministic evidence schema,
   not a canonical shared OxFml seam schema.
2. Some current V1 correlation facts are still recorded from diagnostics;
   B9/CALC-002 owns the compatibility ledger for which facts need stable
   structured facade fields.
3. The packet proves replay-path preservation and no-publish validation for
   the local TreeCalc corpus; it does not claim final replay-appliance pack
   breadth for later external-invalidation, rich-value, or
   correctness-floor lanes.

Semantic equivalence statement: B8 adds runner artifacts, replay-bundle
references, regression assertions, and documentation only. It does not
change source parsing, binding, semantic-plan compilation, runtime
invocation, candidate adaptation, reject policy, scheduling strategy, or
OxCalc coordinator publication authority for any currently exercised Stage
1 profile. Observable formula results are invariant under this artifact
emission addition.

### 22.22 Current V1 Compatibility And Gap Ledger
B9 records the current read-only compatibility position between W050 and
OxFml's frozen V1 consumer runtime facade.

Read-only OxFml V1 inventory:
1. `src/oxcalc-core/Cargo.toml` depends on
   `../../../OxFml/crates/oxfml_core`.
2. `../OxFml/docs/spec/OXFML_CONSUMER_INTERFACE_AND_FACADE_CONTRACT_V1.md`
   names `RuntimeEnvironment`, `RuntimeFormulaRequest`,
   `RuntimeFormulaResult`, and `RuntimeSessionFacade` as the runtime facade
   contract.
3. `../OxFml/docs/spec/OXFML_CONSUMER_INTERFACE_IMPLEMENTATION_PROGRAM_V1.md`
   maps lower-level `SessionService` prepare/open/execute/commit/terminate
   surfaces to `RuntimeSessionFacade::{open_managed_session,
   execute_managed, commit_managed, execute_and_commit_managed,
   abort_managed, expire_managed, managed_session_snapshot}`.
4. `../OxFml/crates/oxfml_core/src/consumer/runtime/mod.rs` exposes the
   current public structs used by OxCalc: `RuntimeEnvironment`,
   `RuntimeFormulaRequest`, `RuntimeFormulaResult`,
   `RuntimeManagedOpenResult`, `RuntimeManagedExecutionResult`,
   `RuntimeManagedCommitResult`, `RuntimeManagedSessionSnapshot`, and
   `RuntimeSessionFacade`.
5. `../OxFml/crates/oxfml_core/src/seam/mod.rs` exposes candidate/commit
   and reject structures carrying `candidate_result_id`,
   `commit_attempt_id`, `trace_correlation_id`, value/shape/topology
   deltas, returned-value surface, spill events, and execution profile
   summary.

Current OxCalc compatibility mapping:
1. `OxfmlRecalcSessionDriver::ensure_prepared` calls
   `RuntimeSessionFacade::open_managed_session`.
2. `OxfmlRecalcSessionDriver::invoke` calls
   `RuntimeSessionFacade::execute` because TreeCalc currently consumes the
   full `RuntimeFormulaResult` surface.
3. `OxfmlRecalcSessionDriver::invoke_managed_commit` and
   `commit_prepared` exercise `execute_and_commit_managed` and
   `commit_managed` as current V1 compatibility evidence.
4. `OxfmlRecalcWave` uses the same driver to preserve the six-phase wave
   shape while keeping OxCalc as the only coordinator publisher.
5. `LocalTreeCalcEngine` now invokes the driver directly and builds the
   current `RuntimeEnvironment` / `RuntimeFormulaRequest` from opaque
   OxFml source, explicit carriers, working values, and deterministic
   provider shims.

Compatibility result:
1. Current V1 is sufficient for the active W050 TreeCalc session path to
   stay on OxFml's public facade.
2. Current V1 is sufficient for deterministic B8 replay/session evidence
   over the local corpus.
3. Current V1 does not yet eliminate OxCalc's compatibility derivations for
   prepared-callable identity, plan-template identity, hole bindings,
   reference/input transport, or targeted metadata invalidation.
4. Current V1 managed commit is useful evidence but does not replace the
   full-result path because `RuntimeManagedCommitResult` does not carry all
   `RuntimeFormulaResult` surfaces currently consumed by TreeCalc evidence
   and coordinator checks.
5. No OxCalc-private adapter around OxFml internals is authorized by this
   ledger. Gaps stay routed to `HANDOFF_CALC_002`.

B9 validation evidence:
1. `cargo test -p oxcalc-core session_driver_ -- --nocapture` exercises the
   facade session driver mapping.
2. `cargo test -p oxcalc-core treecalc_runner_emits_local_run_artifacts -- --nocapture`
   exercises the B8 corpus evidence path on the current facade.
3. `cargo test -p oxcalc-core`, `cargo clippy --all-targets --all-features -- -D warnings`,
   `scripts/check-worksets.ps1`, and `br dep cycles` are the repository
   compatibility checks for this ledger.

Semantic equivalence statement: B9 adds a read-only inventory, gap ledger,
and validation checklist only. It does not change source parsing, binding,
semantic-plan compilation, runtime invocation, candidate adaptation,
reject policy, scheduling strategy, or OxCalc coordinator publication
authority for any currently exercised Stage 1 profile. Observable formula
results are invariant under this ledger addition.

## 23. Status
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - replay artifacts for broader candidate-result versus publication boundaries beyond the current B8 TreeCalc local corpus packet are not yet attached,
  - the Stage 1 local seam packet now consumes more of the already-canonical OxFml category split, but broader TreeCalc descriptor and transport questions remain open beyond the first consumed subset,
  - W026 now has a clear consume-now versus refine-in-notes split, but the topic-matrix pass is not yet converted into executed seam intake work,
  - a narrower follow-on handoff is not required yet, but remains an explicit later decision if W019 evidence creates stronger coordinator pressure
