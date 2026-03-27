# Notes for OxFml

Status: `active`
Owner lane: `OxCalc`
Relationship: outbound observation and response note for the next OxCalc/OxFml integration round

## 1. Purpose
Record the OxCalc-side intake of OxFml's stronger seam, replay, retained-witness, and host-boundary floor, and answer the specific downstream questions raised in `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md`.

This note is an OxCalc-owned observation ledger entry.
It is not a canonical OxFml seam edit and it does not close any handoff by itself.
It should be read together with OxCalc's local `HANDOFF-FML-001` receipt note.

## 2. Core Message
OxCalc can consume the stronger OxFml floor without changing its core coordinator semantics.

The most important points for the next OxCalc/OxFml round are:
1. OxCalc accepts the stronger candidate-versus-commit separation and the minimum typed schema direction as sufficient for the current Stage 1, W018, and planned W019 floor,
2. the highest-value surfaced evaluator/runtime facts for OxCalc are still typed fence mismatch, capability denial, session termination, execution-restriction effects, and correlation ids tying candidate, commit-attempt, reject, and optional fence snapshots together,
3. retained-witness and pack-candidate work should preserve direct cell bindings anywhere semantic truth depends on concrete cell resolution,
4. DNA OneCalc remains a downstream reduced-profile host boundary and must not be treated as a substitute for OxCalc coordinator semantics.

This note is also intended as a forward-and-back alignment pass.
Its goal is to make the remaining seam uncertainties explicit before they drift into implementation-only assumptions.

## 3. Current Evidence
Relevant current OxCalc evidence and planning surfaces are:
1. `docs/spec/core-engine/CORE_ENGINE_OXFML_SEAM.md`
2. `docs/spec/core-engine/CORE_ENGINE_REPLAY_APPLIANCE_ADAPTER.md`
3. `docs/test-runs/core-engine/tracecalc-reference-machine/w018-replay-appliance-bundle-baseline/`
4. `docs/test-runs/core-engine/tracecalc-retained-failures/w018-retained-replay-appliance-bundle-baseline/`
5. `docs/worksets/W019_EXECUTION_SEQUENCE_F_REPLAY_DISTILL_AND_PACK_PROMOTION.md`
6. `docs/worksets/W020_OXFML_DOWNSTREAM_INTEGRATION_ROUND_01.md`

The most relevant exercised local families today are:
1. publication-fence retained-local witnesses,
2. explanatory-only fallback witnesses,
3. quarantined capture-insufficient witnesses,
4. ordinary replay-appliance-aware candidate/publication/reject baselines.

## 4. Current OxCalc Assessment Of Seam Sufficiency
Current OxCalc reading is:
1. the seam is strong enough to continue current TreeCalc core-engine calculation work,
2. the seam is not yet explicit enough to say that all Excel-like hidden machinery relevant to coordinator behavior is fully surfaced,
3. the main remaining risk is not an incorrect current split, but that some evaluator-side identity, provenance, and publication-consequence categories are still compressed too much on the OxCalc side.

The most important still-compressed areas from the OxCalc perspective are:
1. identity and fence categories,
2. candidate-result versus commit-bundle consequence decomposition,
3. dependency additions/removals/reclassifications as typed evaluator facts,
4. host-query and direct-binding-sensitive truth where semantic meaning depends on concrete resolution,
5. semantic-format versus display-facing publication consequences.

## 5. Interface Implications
For the next integration round, OxCalc expects:
1. `candidate_result_id`, `commit_attempt_id`, `reject_record_id`, and optional fence snapshot references to remain stable correlation keys in replay-facing surfaces,
2. typed `FenceMismatchContext`, `CapabilityDenialContext`, and `SessionTerminationContext` to remain authoritative OxFml meanings rather than generic coordinator errors,
3. surfaced execution-restriction and capability-sensitive effects to remain explicit where coordinator replay, publication interpretation, or retained-witness classification depends on them,
4. dependency additions/removals/reclassifications to remain evaluator/runtime facts rather than inferred coordinator policy judgments,
5. direct-cell-binding-sensitive host and witness families to preserve concrete binding identity rather than collapsing into name-only or prose-only artifacts,
6. pack-candidate rehearsal artifacts to remain explicitly non-pack-eligible until a later stronger promotion act.

## 6. Proposed Alignment Topics For This Round
The next useful OxCalc/OxFml alignment topics are:
1. **Identity and fence vocabulary consumption**
   - OxCalc wants to consume a more explicit subset of OxFml's stable-id / version-key / fingerprint / runtime-handle distinctions rather than relying on generic local buckets such as `artifact_token_basis` and `compatibility_basis`.
2. **Candidate-result and commit-bundle consequence shape**
   - OxCalc wants to align its local seam wording more directly with OxFml's `value_delta`, `shape_delta`, `topology_delta`, optional `format_delta`, optional `display_delta`, spill events, and typed evaluator-fact refs.
3. **Dependency consequence taxonomy**
   - OxCalc wants explicit confidence that dependency additions, removals, and reclassifications remain surfaced evaluator facts in retained and reduced replay-facing families rather than being flattened into generic topology summaries.
4. **Host-query and direct-binding-sensitive truth**
   - OxCalc wants to preserve direct-cell-binding-sensitive and host-query-sensitive truth where semantic correctness depends on concrete cell, workbook, environment, locale, or selection-sensitive facts.
5. **Semantic-display boundary**
   - OxCalc wants clearer shared understanding of which format/display consequences are seam-significant and which remain purely downstream/UI-local.

## 7. Minimum Invariants
The following invariants remain mandatory from the OxCalc side:
1. candidate result and committed publication remain distinct artifact stages,
2. reject remains no-publish unless OxFml later promotes a different typed path explicitly,
3. fence meaning remains OxFml-owned and replay-preserved,
4. capability denial and execution restriction remain typed evaluator/runtime outcomes rather than generic coordinator failure classes,
5. replay-valid retained witnesses and pack-candidate artifacts must preserve direct cell bindings where semantic truth depends on concrete cell resolution,
6. DNA OneCalc host-policy reductions must not be mistaken for OxCalc coordinator semantics.

## 8. Explicit Uncertainties We Want To Keep Visible
These are not objections to the current seam.
They are the main places where OxCalc thinks implicit machinery could still slip through if we do not keep naming it:
1. whether OxCalc should consume a canonical identity/fence matrix directly from OxFml docs rather than continuing with local summary buckets,
2. whether `capability_view_key` should now be treated as effectively first-class in the consumed fence basis even while its final canonical fence status remains open,
3. whether dependency reclassification and execution-restriction facts are guaranteed to remain explicit in retained and reduced witness families,
4. whether direct-binding-sensitive host truth will stay explicit once pack-candidate rehearsal broadens,
5. whether locale/date-system/format-service-sensitive consequences need an explicit shared seam note beyond the current format- and display-delta wording.

## 9. Open Questions
The next useful OxFml-side clarifications for OxCalc would be:
1. whether execution-restriction effects already have a canonical minimum object family OxCalc should consume directly, or whether OxCalc should expect only a narrower effect-ref surface for now,
2. whether dependency additions/removals/reclassifications are expected to remain commit-bundle facts in all replay-facing families, including retained and reduced witnesses,
3. whether `commit_attempt_id` and optional fence snapshot references are expected to remain stable across retained-local and future reduced-witness bundle families,
4. how OxFml intends to distinguish direct-binding-sensitive pack-candidate families from name-only families once broader pack-candidate rehearsal widens,
5. whether OxCalc should now consume a more explicit identity-category subset rooted in `formula_stable_id`, `formula_token`, `snapshot_epoch`, `bind_hash`, `profile_version`, and `capability_view_key`,
6. whether OxFml wants a separate shared note on semantic-format versus display-facing seam consequences before W019 broadens retained and pack-candidate evidence.

## 10. Requested Reply Shape
The most useful OxFml reply would be:
1. for each topic in Section 6, mark it as `already canonical`, `canonical but narrower`, or `still open`,
2. for each question in Section 9, answer whether OxCalc should treat the current floor as stable enough to consume now,
3. call out any place where OxCalc is over-reading OxFml's current canonical intent,
4. identify any topic that should move from note-exchange into a narrower formal handoff.

## 11. Returned OxFml Classification Summary
OxFml has now answered the requested topic-by-topic pass.

Current OxCalc intake of that reply is:
1. identity and fence vocabulary consumption: `already canonical`
2. candidate-result and commit-bundle consequence shape: `already canonical`
3. dependency consequence taxonomy: `canonical but narrower`
4. host-query and direct-binding-sensitive truth: `already canonical`
5. semantic-display boundary: `canonical but narrower`

The most important immediate consequences for OxCalc are:
1. OxCalc should stop treating identity/fence categories as only local summary buckets and consume the explicit canonical subset now,
2. OxCalc should stop treating candidate-result and commit-bundle consequence categories as merely generic local payloads and consume the explicit canonical category split now,
3. OxCalc should continue to treat dependency retained/reduced witness projection and semantic-display boundary as narrower closure work rather than as settled universal transport rules.

## 12. Current OxCalc Position After The Returned Classification Pass
Current OxCalc position is:
1. the seam is strong enough to continue current TreeCalc and replay-widening work,
2. no immediate new formal handoff is required from this note round,
3. the remaining pressure is now narrower and better bounded than before.

The main still-open pressure points are:
1. execution-restriction effect transport closure,
2. dependency-addition/removal/reclassification projection closure in retained and reduced witness families,
3. semantic-format versus display-facing seam reading before broader retained and pack-candidate widening.

## 13. Current OxCalc Position On New Handoff Pressure
OxCalc does not currently think a new formal handoff is required immediately.

Current position:
1. the current OxFml note and `HANDOFF-FML-001` are sufficient for the next OxCalc intake and planning round,
2. any later handoff should be narrower and tied to exercised evidence from W019 rather than filed spec-first,
3. the most likely future trigger would be narrower coordinator pressure around execution-restriction effects or publication/topology consequence breadth.

## 14. Working Rule For This Note
This note should be used to negotiate and narrow seam uncertainties explicitly.

It should not be used to:
1. silently rewrite OxFml canonical meaning from the OxCalc side,
2. treat note-level agreement as handoff closure,
3. imply that the current seam is final or exhaustive.

## 15. Current OxCalc Position After W019 Evidence
OxCalc has now exercised the two bounded uncertainty areas that remained active after the first returned-classification pass.

Current local read is:
1. dependency additions/removals/reclassifications are now exercised locally in replay-valid retained-local and reduced-witness families, and nothing in that evidence currently forces a narrower formal handoff,
2. the semantic-format versus display-facing boundary is now narrowed locally to `semantic_only_tracecalc_scope` for the current Stage 1 `TraceCalc` surface, without over-claiming broader display-facing closure,
3. the latest OxFml/OxFunc note adds no new OxCalc-facing seam trigger,
4. provider-failure and callable-publication remain watch lanes only,
5. a narrower `HANDOFF-CALC-002` is not required yet.

## 16. Current Working Rule After This Round
After the current note round and W019 evidence:
1. treat the OxCalc/OxFml seam as strong enough for continued coordinator, replay, and retained-witness work,
2. keep using W019/W021 evidence rather than note-only alignment to decide whether a narrower follow-on handoff is needed,
3. escalate to a narrower formal handoff only if later exercised evidence shows stronger coordinator pressure around execution-restriction transport or publication/topology consequence breadth.

## 17. Next TreeCalc-Facing Note Pass Shape
OxCalc is now preparing W026 and the first TreeCalc-ready engine intake lane.

For that lane, OxCalc wants the next note exchange to move from broad seam-alignment prose to a topic-matrix pass.
The active TreeCalc-facing topics are:
1. formula and bind artifact identity carriage
2. direct and relative reference descriptor carriage
3. unresolved and host-sensitive reference carrier rules
4. dependency fact carriage for additions, removals, and reclassifications
5. candidate-result consequence optionality and correlation guarantees
6. reject-context carrier and diagnostic guarantees
7. runtime-derived effect and execution-restriction transport
8. direct-binding-sensitive witness preservation rules
9. semantic-format versus display-facing consequence boundary

## 18. Requested OxFml Reply Shape For The Next Pass
For each active topic in Section 17, the most useful OxFml reply would now be:
1. current classification:
   - `already canonical`
   - `canonical but narrower`
   - `still open`
2. carrier surface OxCalc should consume now
3. explicit non-assumptions OxCalc must preserve
4. whether the current floor is sufficient for W026 intake now
5. whether the topic remains note-only or now deserves a narrower handoff

## 19. Current OxCalc Working Hypotheses For W026
Unless OxFml says otherwise, OxCalc is currently planning W026 on the following basis:
1. formula identity and bind identity are stable enough to consume now for first TreeCalc formula-bearing nodes
2. direct absolute reference carriage is likely ready to consume before relative-reference closure is fully broad
3. relative-reference descriptors need a narrower pass on what is fixed at bind time versus what remains contextual
4. dependency additions/removals/reclassifications are consumable now for live graph and recalc semantics, even if broader retained/reduced witness closure remains narrower
5. candidate-result consequence categories are already stable enough to consume now through the current canonical split
6. runtime-derived effects are semantically consumable now, while final transport-carrier closure remains narrower
7. direct-binding-sensitive truth is already canonical to preserve where semantic correctness depends on concrete resolution
8. semantic-format versus display-facing interpretation remains a clarifying note topic rather than a new formal handoff trigger today

## 20. Specific Questions For The Next Pass
The next useful OxFml clarifications for W026 are:
1. for the first TreeCalc subset, which relative-reference forms are most stable to consume now without over-reading the bind package
2. whether OxFml expects relative-reference meaning to be fully bound before candidate evaluation for the first TreeCalc subset
3. which unresolved or host-sensitive reference carriers OxCalc should expect to preserve explicitly in candidate, reject, and replay families
4. whether dependency fact identity has a preferred replay-facing carrier OxCalc should preserve now rather than invent locally
5. whether any candidate consequence families that are optional in general become effectively mandatory for the first TreeCalc subset
6. which reject-context families should be treated as canonical object families from the start of W026 rather than merely local projections
7. whether execution-restriction observations have a preferred current effect-ref carrier OxCalc should consume now
8. whether OxFml wants the semantic-format versus display-facing boundary tracked as a separate note subtopic during W026 rather than waiting for later TreeCalc baseline evidence

## 21. Current Intake Of The Topic-Matrix Reply
OxCalc has now processed the latest OxFml topic-matrix reply.

Current local read is:
1. formula and bind identity carriage is clear enough to consume now,
2. candidate consequence carriage and reject-context carriage are clear enough to consume now for W026,
3. direct-binding-sensitive witness preservation is clear enough to consume now,
4. the main remaining refinement topics are:
   - relative-reference descriptor carriage
   - unresolved and host-sensitive reference carrier rules
   - runtime-derived effect and execution-restriction transport shape
   - semantic-format versus display-facing boundary
5. no new narrower handoff is justified yet from this reply alone.

## 22. Current OxCalc Conclusion After This Reply
Current conclusion is:
1. the interface and seam still merit refinement, but only in the bounded note-level areas listed in Section 21,
2. the overall seam state is now materially clear enough to proceed with W026 planning and later TreeCalc intake work,
3. the current uncertainty is no longer broad seam ambiguity; it is a narrower carrier and closure question set,
4. OxCalc should therefore proceed by consuming the clear-now topics and using later note passes only to narrow the remaining descriptor and transport questions.

## 23. W026 Narrow TreeCalc-Facing Pass
This is the next narrow W026-focused note pass.

OxCalc is now treating the following as consume-now topics for W026 unless OxFml says otherwise:
1. formula and bind identity carriage,
2. candidate consequence and correlation floor,
3. typed reject-context floor for the current families,
4. direct-binding-sensitive witness preservation.

This pass is therefore limited to the remaining narrower topics:
1. relative-reference descriptor carriage,
2. unresolved and host-sensitive reference carrier rules,
3. runtime-derived effect and execution-restriction transport shape,
4. semantic-format versus display-facing boundary.

No new handoff is being filed with this pass.

## 24. Topic A: Relative-Reference Descriptor Carriage
### OxCalc consumed need
For the first TreeCalc subset, OxCalc needs enough relative-reference meaning to:
1. determine the structural context that anchors lookup,
2. distinguish edits that force rebind from edits that require only recalc,
3. build deterministic dependency and invalidation behavior,
4. keep relative-reference meaning replay-visible rather than hidden in evaluator-local state.

### Current OxCalc working assumption
Current OxCalc working assumption is:
1. direct absolute reference carriage is ready sooner than full relative-reference closure,
2. the first TreeCalc subset will likely need a narrower relative-reference family than the eventual broader TreeCalc scope,
3. OxCalc should not assume that all relative-reference meaning is fully closed or fully frozen at the current seam floor.

### Clarifications requested from OxFml
The next useful clarifications are:
1. which relative-reference forms are most stable to consume now for the first TreeCalc subset,
2. whether OxFml expects those forms to be fully bound before candidate evaluation or whether some context-sensitive carrier remains live,
3. which structural edits should be read as rebind-forcing for those forms,
4. whether OxFml has a preferred carrier distinction between:
   - already-bound relative targets,
   - relative navigation descriptors,
   - unresolved relative forms.

### W026 sufficiency question
OxCalc’s current view is that W026 can proceed if the first relative-reference subset is narrowed explicitly, even if broader relative-reference closure remains deferred.

## 25. Topic B: Unresolved and Host-Sensitive Reference Carriers
### OxCalc consumed need
For the first TreeCalc-ready path, OxCalc needs unresolved and host-sensitive reference outcomes to remain explicit enough to:
1. preserve typed reject or no-publish behavior,
2. distinguish structurally unresolved from host-sensitive unresolved cases,
3. preserve replay and retained-witness fidelity,
4. avoid inventing local coordinator-side meaning for evaluator-side unresolved carriers.

### Current OxCalc working assumption
Current OxCalc working assumption is:
1. unresolved and host-sensitive carriers are canonical but narrower rather than absent,
2. the live TreeCalc path should preserve them explicitly in candidate, reject, and replay surfaces where relevant,
3. OxCalc should not normalize those cases into one generic “resolution failure” bucket.

### Clarifications requested from OxFml
The next useful clarifications are:
1. which unresolved-reference and host-sensitive-reference carriers OxCalc should expect to preserve explicitly in first-phase TreeCalc candidate and reject paths,
2. whether there is a preferred distinction between:
   - unresolved-at-bind,
   - unresolved-at-evaluate,
   - host-sensitive-but-resolvable-only-with-concrete-host-truth,
3. which of those carriers should already be treated as replay-facing first-class objects rather than as trace detail only.

### W026 sufficiency question
OxCalc’s current view is that W026 can proceed if the first in-scope unresolved and host-sensitive carrier families are named explicitly, even if broader host-lane closure remains deferred.

## 26. Topic C: Runtime-Derived Effects and Execution-Restriction Transport
### OxCalc consumed need
OxCalc needs the first TreeCalc-ready path to consume runtime-derived effects explicitly enough to:
1. surface dynamic dependency activation and release,
2. preserve capability-sensitive observations,
3. preserve execution-restriction observations,
4. keep runtime-derived effect truth out of hidden mutable implementation detail,
5. decide whether a candidate can publish, reject, or remain no-publish with deterministic replay.

### Current OxCalc working assumption
Current OxCalc working assumption is:
1. runtime-derived effects are semantically consumable now,
2. execution-restriction observations are important enough to be treated as explicit runtime facts,
3. OxCalc must not yet assume one final frozen single-object transport carrier,
4. W026 should consume the semantic floor now and leave transport-shape closure as a narrower carried question unless live evidence says otherwise.

### Clarifications requested from OxFml
The next useful clarifications are:
1. whether there is a preferred current effect-ref or object-family carrier OxCalc should consume now for execution-restriction observations,
2. which execution-restriction facts are most important for first TreeCalc coordinator behavior:
   - candidate eligibility,
   - typed reject/no-publish reasoning,
   - publication interpretation,
   - replay/explain fidelity,
3. whether OxFml expects capability-sensitive and execution-restriction-sensitive observations to travel in the same family for the first TreeCalc subset or remain distinct.

### W026 sufficiency question
OxCalc’s current view is that W026 can proceed if the semantic minimum consumed now is explicit, even if final transport-carrier closure remains deferred. This remains one of the few likely future narrower handoff triggers if the live TreeCalc path exposes insufficiency.

## 27. Topic D: Semantic-Format Versus Display-Facing Boundary
### OxCalc consumed need
For the first TreeCalc-ready engine phase, OxCalc needs:
1. a consumed semantic consequence floor,
2. explicit format-sensitive consequence visibility where runtime or later observer policy could depend on it,
3. enough visibility into display-facing categories to avoid silently collapsing them into semantic or format buckets.

### Current OxCalc working assumption
Current OxCalc working assumption is:
1. the current first TreeCalc-ready phase should remain semantics-first,
2. `format_delta` and `display_delta` should remain explicitly distinct categories,
3. broader display-facing closure should remain deferred unless first live TreeCalc evidence shows that it affects publication or replay truth materially.

### Clarifications requested from OxFml
The next useful clarifications are:
1. whether OxFml wants the first TreeCalc-ready phase to treat `format_delta` as replay-visible but not necessarily publication-critical in all in-scope cases,
2. whether there are any first-phase TreeCalc consequence families where `display_delta` should already be treated as more than carried honesty,
3. whether OxFml prefers this boundary to remain an explicit note subtopic during W026 rather than being treated as closed once the first TreeCalc subset is named.

### W026 sufficiency question
OxCalc’s current view is that W026 can proceed with this boundary still marked `canonical but narrower`, provided the consumed-now semantic and format floor is explicit and no broader display-facing promise is implied.

## 28. Requested OxFml Reply Shape For This Narrow Pass
For each of Sections 24 through 27, the most useful OxFml reply would be:
1. current classification:
   - `already canonical`
   - `canonical but narrower`
   - `still open`
2. the carrier or object family OxCalc should consume now,
3. explicit non-assumptions OxCalc must preserve,
4. whether the topic is sufficient for W026 intake now,
5. whether the topic should remain note-level or now deserves a narrower handoff.

## 29. Current OxCalc Working Rule After Writing This Pass
Until OxFml answers this narrower W026-focused pass:
1. OxCalc will treat the seam as clear enough to continue planning,
2. OxCalc will not treat the remaining four topics as broad architecture blockers,
3. OxCalc will not silently close those four topics in implementation without either:
   - note-level clarification,
   - or a narrower formal handoff if live evidence later demands it.

## 30. Current Intake Of OxFml's Narrower W026 Reply
OxCalc has now processed the latest narrower W026-focused reply.

Current local read is:
1. the seam interface is settled enough for the first TreeCalc intake phase,
2. no new formal handoff is justified now,
3. W026 no longer depends on broad seam clarification before it can start,
4. the remaining uncertainty is limited to broader closure beyond the first consumed subset.

More specifically:
1. relative-reference carriage is good enough now for a narrowed first subset,
2. unresolved and host-sensitive carriers are good enough now if the first explicitly named families remain distinct,
3. runtime-derived effects and execution-restriction transport are good enough now semantically, while final transport-carrier closure remains deferred,
4. semantic-format versus display-facing handling is good enough now for a semantics-first first phase, provided OxCalc keeps `format_delta` and `display_delta` distinct and does not over-claim broader display closure.

## 31. Current OxCalc Conclusion After The Narrower Reply
Current conclusion is:
1. the seam does not need more broad refinement before W026,
2. the current status is clear enough to proceed,
3. later note passes should now be triggered by live intake pressure rather than by general pre-implementation uncertainty,
4. any future narrower handoff should be reserved for real insufficiency in:
   - execution-restriction transport,
   - publication/topology consequence breadth,
   - or later broader TreeCalc reference-family closure.

## 32. Intake Of OxFml Host Runtime And External Requirements Draft
OxCalc has now reviewed `../OxFml/docs/spec/OXFML_HOST_RUNTIME_AND_EXTERNAL_REQUIREMENTS.md` as the bounded packet for the next host/coordinator seam round.

Current OxCalc read is:
1. this is the right canonical OxFml draft to anchor the next host/runtime note exchange,
2. it is strong enough to support first coordinator-host implementation planning,
3. it does not yet need to be treated as shared seam-freeze text,
4. the remaining uncertainty is narrower than before and is concentrated in carrier breadth rather than in authority split.

## 33. Topic-By-Topic OxCalc Review Of The Host Runtime Draft
### 33.1 Direct-host versus OxCalc-integrated split
OxCalc position:
1. sufficient for first implementation planning,
2. authority split is clear enough for the current reduced direct-host lane and the first coordinator-integrated lane,
3. OxCalc-owned concerns that still need explicit carry-through in later implementation packets are:
   - scheduler and publication policy staying outside OxFml artifact meaning,
   - stable published and pinned-view consequences on the coordinator side,
   - multi-node candidate grouping and graph-wide invalidation beyond the direct-host proving lane.

Current classification:
1. `already canonical` for the first implementation slice,
2. broader graph-host and scheduler closure remains deferred by design.

### 33.2 Required inputs
OxCalc position:
1. sufficient for the first coordinator-host implementation slice,
2. formula/structure inputs, direct-cell and defined-name bindings, typed host-query/provider families, runtime library-context snapshot/provider requirements, and capability/fence inputs are all in the right shape for the current floor,
3. the main narrower consumption point that still needs note-level care is caller-anchor/address-mode handling for the first TreeCalc relative-reference subset.

Current classification:
1. `already canonical` for the first direct and integrated host slice,
2. `canonical but narrower` only for the relative-reference contextual subset OxCalc will consume in W026.

### 33.3 Required outputs
OxCalc position:
1. sufficient for the first coordinator-host implementation slice,
2. the candidate / commit / reject / trace families and `ReturnedValueSurface` split are strong enough to preserve evaluator meaning without coordinator reinterpretation,
3. OxCalc expects the coordinator-relevant ids to remain stable where present:
   - `candidate_result_id`
   - `commit_attempt_id`
   - `reject_record_id`
   - optional `fence_snapshot_ref`
4. typed effect and topology-sensitive consequence surfaces are the right floor for the first slice.

Current classification:
1. `already canonical` for the first slice,
2. broader consequence breadth remains a later evidence topic rather than a current blocker.

### 33.4 Implementation sufficiency for the current local scope
OxCalc position:
1. sufficient now for the currently covered proving-host and single-formula direct-host slice,
2. sufficient now for OxCalc to plan around current candidate / commit / reject / effect carriers,
3. sufficient now for the currently admitted host-query/provider slice:
   - `INFO`
   - `CELL`
   - `RTD`
4. the currently covered higher-order callable floor remains upstream-semantic and does not create a new coordinator-host blocker today.

Current classification:
1. `already canonical` for the current admitted floor,
2. broader provider-failure and callable-publication consequences remain watch lanes only.

### 33.5 Explicit non-assumptions and deferrals
OxCalc position:
1. agreed,
2. deferred provider families, full scheduler/distributed policy, and full product-host specification should remain explicit non-assumptions,
3. OxCalc does not need those deferred lanes to proceed with the first host/coordinator implementation slice.

Current classification:
1. `already canonical` as a working boundary for the next implementation round.

## 34. Narrower Residuals After Reviewing The Host Runtime Draft
The host/runtime draft does not create a new broad seam problem.

The remaining narrower residuals are:
1. caller-anchor and address-mode carriage for the first TreeCalc relative-reference subset,
2. execution-restriction transport shape beyond the current semantic minimum,
3. publication/topology consequence breadth beyond the current local exercised floor,
4. provider-failure and callable-publication as watch lanes only if they later become coordinator-visible.

Current OxCalc read is:
1. these residuals remain note-level topics,
2. none justifies a new formal handoff today,
3. the most likely future handoff trigger still remains execution-restriction transport or publication/topology breadth if live evidence exposes insufficiency.

## 35. Requested OxFml Reply On This Host Runtime Review Pass
The most useful OxFml reply to this pass would be:
1. confirm whether OxFml agrees with OxCalc's `already canonical` read for the first host/coordinator implementation slice,
2. call out any place where OxCalc is still over-reading the host/runtime draft,
3. confirm whether caller-anchor/address-mode handling for the first TreeCalc relative-reference subset should stay in the W026 note lane rather than move to a handoff,
4. confirm whether provider-failure and callable-publication should remain watch lanes only until they become coordinator-visible in exercised evidence.

## 36. Current Intake Of OxFml's Host Runtime Reply
OxCalc has now processed OxFml's reply to the host/runtime review pass.

Current local read is:
1. OxFml agrees with OxCalc's `already canonical` assessment for the first host/coordinator implementation slice across:
   - direct-host versus OxCalc-integrated host split
   - required inputs
   - required outputs
   - implementation sufficiency for the currently covered local scope
   - explicit non-assumptions and deferrals
2. OxFml agrees the host/runtime draft is strong enough for first implementation planning,
3. OxFml still does not treat that draft as shared seam-freeze text,
4. no new formal handoff is warranted from the host/runtime review pass alone.

## 37. Current Caution Intake From OxFml
The caution points OxCalc is now carrying explicitly are:
1. do not over-read the host/runtime draft as full language or full built-in-function closure,
2. do not over-read caller-anchor and address-mode carriage for the first TreeCalc relative-reference subset as already frozen in the host packet,
3. do not over-read execution-restriction transport as one final single frozen carrier,
4. do not over-read publication and topology breadth beyond the current local exercised floor,
5. do not over-read provider-failure or callable-publication as active coordinator-facing seam clauses yet.

## 38. Current OxCalc Conclusion After The Host Runtime Reply
Current conclusion is:
1. the host/runtime packet is now settled enough for first implementation planning,
2. caller-anchor/address-mode handling for the first TreeCalc relative-reference subset should remain in the W026 note lane,
3. provider-failure and callable-publication remain watch lanes only until they become coordinator-visible in exercised evidence,
4. the remaining host/runtime uncertainty is bounded and note-level rather than a present handoff trigger.

## 39. Current Intake Of OxFml's Latest Follow-Up Note
OxCalc has now processed the latest OxFml follow-up note and related docs, especially:
1. `../OxFml/docs/spec/OXFML_HOST_RUNTIME_AND_EXTERNAL_REQUIREMENTS.md`
2. `../OxFml/docs/worksets/W036_structured_reference_and_table_formula_semantics.md`
3. `../OxFml/docs/worksets/W048_editor_language_service_and_immutable_formula_host_plan.md`
4. `../OxFml/docs/worksets/W051_oxcalc_fixture_host_and_stand_in_packet.md`

Current local read is:
1. the first host/runtime packet remains converged enough for the first TreeCalc implementation slice,
2. no new formal handoff is justified from this note alone,
3. OxFml's newest bounded asks are now:
   - structured-reference and table-context packet confirmation,
   - immutable edit and validated completion packet review,
   - fixture host and coordinator stand-in packet review.

## 40. Current OxCalc Implementation Confirmation
OxCalc has now landed the first seam-backed TreeCalc runtime slice locally.

Current local implementation read is:
1. the local TreeCalc runtime no longer uses its prior internal evaluator path for the active covered lane,
2. the active local lane now translates TreeCalc formulas into the agreed OxFml direct-host slice and executes through OxFml bind/evaluate,
3. the local dependency graph seed is now prepared from the translated OxFml bind slice plus explicit residual carriers rather than only from local lowering,
4. current local baseline `w025-treecalc-local-baseline` regenerates successfully with `13` cases and `0` expectation mismatches after the seam-backed conversion.

Important implementation-facing notes from this conversion:
1. the first direct-host slice is strong enough for local TreeCalc evaluation planning,
2. the conversion confirms that some previously local TreeCalc expectations should now follow seam-backed behavior rather than older local-only behavior,
3. the current local lane still uses the narrowed first direct-host packet, not the broader future W026/W029 carrier breadth.

## 41. Structured-Reference Packet Confirmation
OxCalc's current read is that OxFml's proposed first semantic table packet is the right starting point for both direct-host and TreeCalc-integrated use:
1. `table_catalog`
2. `enclosing_table_ref`
3. `caller_table_region`

Current OxCalc confirmation is:
1. yes, this is the right first semantic packet shape,
2. yes, totals/header/data region identity should remain explicit in the packet even if the first executed floor is smaller than the later full workbook-table model,
3. no narrower first anchor packet is required before first implementation use, provided `caller_table_region` is explicit enough to distinguish:
   - headers
   - data
   - totals
   - current-row-sensitive binding context where applicable.

Current non-assumptions from OxCalc are:
1. this does not imply OxCalc wants OxFml to own workbook table objects,
2. this does not imply the whole broader table model is frozen,
3. this does not imply broader structured-reference closure beyond the current first packet.

## 42. Immutable Edit And Validated Completion Packet Read
OxCalc's current read of the `W048` packet direction is:
1. the split into immutable edit request, immutable edit result, and validated completion application result is the right first packetization,
2. OxFml should return replacement-ready immutable formula artifacts and diagnostics rather than owning containing-spine replacement,
3. validated completion application should remain host-local by default rather than coordinator-visible by default,
4. coordinator visibility is only needed later if completion application itself starts producing replay-significant or publication-significant artifacts.

Current OxCalc view of the proposed fields is:
1. the proposed immutable edit request fields are sufficient for a first packet,
2. the proposed immutable edit result fields are sufficient for a first packet,
3. the proposed validated completion application result is sufficient for a first packet.

Current likely useful additions from the OxCalc side are modest:
1. an explicit host-owned formula-slot identity or slot-kind field may become useful once OxCalc moves beyond one narrow formula-bearing slot family,
2. an explicit packet/schema version field should remain easy to project,
3. no broader coordinator acknowledgment field is needed in the first packet.

Current OxCalc reuse read is:
1. yes, the same packet family should be reusable for ordinary cell formulas, host-managed defined-name formulas, and later other formula-bearing slots,
2. the host remains owner of containing-tree replacement and acceptance of the resulting immutable artifact.

## 43. Fixture Host And Coordinator Stand-In Packet Read
OxCalc's current read of the `W051` stand-in packet direction is:
1. yes, this is the right bounded next packet to support deterministic integration artifacts,
2. yes, those inputs should be modeled as host/coordinator-supplied truths rather than evaluator-owned meaning,
3. no, this should not be read as freezing the production OxCalc coordinator API.

Current OxCalc confirmation on the proposed packet shape is:
1. the stand-in packet families are directionally right:
   - formula-slot facts
   - binding-world facts
   - typed host/query facts
   - runtime catalog facts
2. `RegisteredExternalProvider` may stay present from the start as an optional stand-in packet field, even if the first executed floor still defers broader worksheet external-provider lanes,
3. candidate/commit/reject capture should remain a separate projection layer and should not be folded into the input stand-in packet.

Current likely useful additional identity fields are:
1. a stand-in packet identity or fixture-input identity,
2. explicit structure-context identity,
3. explicit formula-slot identity where the packet is reused across multiple slot families.

## 44. Current Intake Of OxFml's Latest Stand-In And Registered-External Refinement
OxCalc has now processed the latest OxFml follow-up covering the converged stand-in packet read and the first `W052` registered-external packet sharpening.

Current local read is:
1. the `W051` fixture-host and coordinator stand-in packet is now settled enough for deterministic automated scaffolding and first TreeCalc-facing integration artifacts,
2. the latest OxFml reply accepts OxCalc's suggested packet refinements:
   - stand-in packet or fixture-input identity,
   - explicit structure-context identity,
   - explicit formula-slot identity where reused across slot families,
3. `RegisteredExternalProvider` should remain optional in the stand-in packet from the start,
4. candidate / commit / reject capture should remain a separate projection layer rather than being folded into stand-in inputs,
5. the first `W052` registered-external family sharpens later host-facing mutation lanes without freezing the production OxCalc coordinator API.

Current OxCalc read of the `W052` sharpening is:
1. host- or coordinator-initiated registration should be modeled as typed mutation requests funneled into OxFunc-owned catalog truth,
2. OxCalc should preserve initiating channel and stable registration identity where later TreeCalc-facing integration uses that lane,
3. OxCalc should not infer ownership of the function catalog from this packet sharpening,
4. snapshot-generation and coordinator acknowledgment consequences from register and unregister remain narrower than the current first packet and are not yet being frozen.

## 45. Current OxCalc Watch Lanes After This Round
The active watch lanes remain:
1. caller-anchor and address-mode carriage for the first TreeCalc relative-reference subset,
2. execution-restriction transport shape,
3. publication/topology consequence breadth,
4. provider-failure and callable-publication only if they become coordinator-visible in exercised evidence.

Current OxCalc read is:
1. these remain note-level refinement lanes rather than current handoff triggers,
2. the latest OxFml note does not broaden them into a new immediate blocker set.

## 46. Current OxCalc Working Rule After This Exchange
After this note round, OxCalc's current working rule is:
1. continue first TreeCalc integration work on the converged host/runtime and structured-table packet floor,
2. treat the immutable edit packet, the stand-in packet, and the narrower registered-external mutation lane as bounded coordination topics rather than reopening broad seam design,
3. reserve a narrower formal handoff only for concrete insufficiency exposed by live implementation evidence,
4. keep the current direct-host TreeCalc conversion and upstream-host scaffolding corpus as implementation evidence for the first host/coordinator slice rather than over-claiming broader seam freeze.

## 47. Residual Narrowing Round
OxCalc is now taking the three remaining note-level host/coordinator residuals as one bounded refinement round:
1. caller-anchor and address-mode carriage for the first TreeCalc relative-reference subset,
2. execution-restriction transport breadth,
3. publication and topology consequence breadth.

Current OxCalc read is:
1. these do not justify a broad seam reopening,
2. these are still note-level topics rather than current handoff triggers,
3. the right next pass is to narrow consume-now assumptions and non-assumptions for each topic so implementation pressure can be judged concretely.
4. this residual round is now packetized locally as:
   - W026 Sequence 1: caller-anchor and address-mode carriage
   - W026 Sequence 2: execution-restriction transport breadth
   - W026 Sequence 3: publication and topology consequence breadth

## 48. W026 Sequence 1: Caller-Anchor And Address-Mode Carriage
### OxCalc consumed need
For the first TreeCalc relative-reference subset, OxCalc needs enough caller-context carriage to:
1. bind relative forms honestly for the first in-scope subset,
2. distinguish edits that force rebind from those that require only recalc,
3. preserve replay-visible context rather than hiding relative meaning in evaluator-local state,
4. avoid inventing local coordinator semantics for address-mode-sensitive interpretation.

### Current OxCalc consumed-now working assumption
OxCalc is currently planning on the following minimum floor:
1. `caller_anchor` remains explicit where relative or host-sensitive meaning depends on it,
2. formula-channel and address-mode remain explicit host-supplied context,
3. structure-context identity remains explicit and stable across bind and later candidate work,
4. the first TreeCalc subset only consumes relative forms whose contextual dependence is already preserved honestly in the bound/reference artifact.

### Explicit non-assumptions
OxCalc is not assuming:
1. full relative-reference closure,
2. one final frozen transport shape for every caller-sensitive relative form,
3. that every address-mode-sensitive clause is already closed for the broader TreeCalc universe.

### OxCalc question back to OxFml
For the first TreeCalc subset, OxCalc now wants the narrowest practical shared reading of:
1. which caller-anchor and address-mode facts must be preserved as first-class carried inputs at bind time,
2. which relative forms remain admissible only because that caller-context is still explicit,
3. which edit families should be read as rebind-forcing because caller-sensitive meaning may change.

## 49. W026 Sequence 2: Execution-Restriction Transport Breadth
### OxCalc consumed need
For the first TreeCalc-ready coordinator slice, OxCalc needs execution-restriction observations explicit enough to:
1. preserve typed no-publish or reject meaning,
2. keep capability-sensitive and restriction-sensitive outcomes out of generic scheduler failure buckets,
3. keep replay and explain fidelity when execution restriction changes candidate interpretation,
4. avoid silently flattening these facts into trace-only prose.

### Current OxCalc consumed-now working assumption
OxCalc is currently consuming:
1. execution-restriction observations semantically,
2. current candidate-result and commit-bundle surfaced evaluator facts,
3. current topology/effect fact refs where execution restriction is carried through that family,
4. distinct capability-sensitive and execution-restriction-sensitive readings unless OxFml later freezes a merged carrier explicitly.

### Explicit non-assumptions
OxCalc is not assuming:
1. one final single-object carrier,
2. that every execution-restriction observation is publication-critical,
3. that scheduler policy may absorb or reinterpret the typed evaluator/runtime meaning.

### OxCalc question back to OxFml
For the first TreeCalc slice, OxCalc now wants the narrowest practical shared reading of:
1. which execution-restriction facts must stay first-class in candidate and reject families,
2. which can remain fact-ref or sidecar-carried without losing coordinator truth,
3. whether any current first-slice execution-restriction observations should already be treated as topology-sensitive or publication-sensitive consequences rather than only runtime-effect sidecars.

## 50. W026 Sequence 3: Publication And Topology Consequence Breadth
### OxCalc consumed need
For the first TreeCalc-ready publication path, OxCalc needs enough consequence breadth to:
1. keep `value_delta`, `shape_delta`, and `topology_delta` distinct,
2. preserve publish-visible absence versus presence semantics across consequence categories,
3. keep topology and dependency consequences explicit where invalidation, publication, or replay meaning depends on them,
4. avoid compressing broader consequence truth into a value-only publication summary.

### Current OxCalc consumed-now working assumption
OxCalc is currently planning on the following first-phase rule:
1. `value_delta`, `shape_delta`, and `topology_delta` remain distinct canonical categories,
2. optional `format_delta` and optional `display_delta` remain carried explicitly when present,
3. first-phase TreeCalc stays semantics-first and does not over-claim broader display-facing or topology-breadth closure,
4. topology-sensitive consequence breadth beyond the currently exercised floor remains a residual to be widened by evidence rather than assumed now.

### Explicit non-assumptions
OxCalc is not assuming:
1. the whole broader publication consequence universe is already frozen,
2. every topology-sensitive consequence family is already mandatory for the first TreeCalc subset,
3. current local exercise breadth is equal to full coordinator publication breadth.

### OxCalc question back to OxFml
For the first TreeCalc slice, OxCalc now wants the narrowest practical shared reading of:
1. which topology-sensitive consequence families are first-slice publish-critical,
2. which may remain carried for replay honesty without first-slice publication dependence,
3. whether any currently optional consequence families become effectively mandatory for the first TreeCalc coordinator path once structural edits and dynamic dependency effects are combined.

## 51. Requested OxFml Reply Shape For This Residual Round
For each of Sections 48 through 50, the most useful OxFml reply would be:
1. current classification:
   - `already canonical`
   - `canonical but narrower`
   - `still open`
2. consumed-now carrier or object family OxCalc should rely on,
3. explicit non-assumptions OxCalc must preserve,
4. whether the topic remains note-level or now deserves a narrower handoff.

## 52. Current Intake Of OxFml's Residual W026 Reply
OxCalc has now processed OxFml's reply to the three-sequence residual round.

Current local read is:
1. OxFml agrees the three-sequence residual split is the right next note shape,
2. W026 Sequence 1, Sequence 2, and Sequence 3 all remain note-level rather than becoming current handoff triggers,
3. all three sequences remain `canonical but narrower`,
4. no new formal handoff is justified from this residual reply.

Current consumed-now carrier read from OxFml is:
1. for W026 Sequence 1 caller-anchor and address-mode carriage:
   - `FormulaSourceRecord`
   - `caller_anchor`
   - formula-channel and address-mode context
   - structure-context identity
   - current bound-reference and normalized-reference families only where the first subset already preserves contextual dependence honestly
2. for W026 Sequence 2 execution-restriction transport breadth:
   - current candidate-result and commit-bundle surfaced evaluator facts
   - current topology/effect fact refs
   - current typed capability-sensitive and execution-restriction-sensitive observations
3. for W026 Sequence 3 publication and topology consequence breadth:
   - `value_delta`
   - `shape_delta`
   - `topology_delta`
   - optional `format_delta`
   - optional `display_delta`
   - current spill and dependency-sensitive surfaced evaluator/runtime fact refs where present

Current explicit non-assumptions OxCalc is carrying forward are:
1. do not treat caller-anchor and address-mode carriage as full relative-reference closure,
2. do not treat execution-restriction transport as one final frozen single-object carrier,
3. do not treat currently optional consequence families as universally mandatory across the broader future TreeCalc scope,
4. do not treat the current local publication and topology breadth as equal to the full eventual coordinator publication universe.

## 53. Current OxCalc Conclusion After The Residual Reply
Current conclusion is:
1. W026 Sequence 1, Sequence 2, and Sequence 3 may continue on the current note-level floor,
2. the consumed-now carriers are now explicit enough for continued TreeCalc intake planning,
3. the remaining uncertainty is about broader closure, not about immediate first-slice insufficiency,
4. a narrower `HANDOFF-CALC-002` remains deferred unless live TreeCalc evidence later exposes a concrete gap in one of these carried families.
