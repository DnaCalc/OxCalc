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

## 54. Current OxCalc Reply On The Narrowed W052 Packet
OxCalc has now processed the sharper `W052` packet in OxFml Section `21A`.

Current OxCalc reply is:
1. yes, OxCalc is content to align with the same direct adopted packet names for the current phase:
   - `RegisterIdRequest { library_name, procedure, declared_type_text }`
   - `RegisteredExternalDescriptor { stable_registration_id, register_id, origin_kind, display_name, library_name, procedure, declared_type_text }`
   - `RegisteredExternalCallRequest { target, invocation_args }`
   - `RegisteredExternalTarget::{ RegisterId, Direct }`
2. yes, OxCalc is content with the current seven-field `RegisteredExternalDescriptor` as the first shared minimum field set,
3. yes, OxCalc is content to keep `RegisteredExternalCatalogMutation*` and `RegisteredExternalCatalogController` as OxFml-owned host/coordinator-facing funnel packets for the current phase unless OxFunc later asks for promotion into a broader shared packet family,
4. yes, OxCalc treats the current snapshot-generation and invalidation split as sufficient for first TreeCalc-facing planning:
   - bind-visible registration or unregister should generate a new `LibraryContextSnapshot` and trigger bind invalidation where the visible function or name world changes,
   - `CALL` / `REGISTER.ID`-only descriptor mutation may remain targeted reevaluation by default for the current first phase.

Current OxCalc non-assumptions remain:
1. this does not freeze the broader production coordinator API,
2. this does not transfer registered-external catalog ownership into OxCalc,
3. this does not freeze broader snapshot-acknowledgment or publication consequences from register/unregister,
4. this does not imply that all later registered-external families will remain limited to the current first packet.

## 55. Current OxCalc Conclusion After The Narrowed W052 Reply
Current conclusion is:
1. the narrowed `W052` packet is settled enough for first TreeCalc-facing planning,
2. no new formal handoff is justified from this sharper `W052` reply alone,
3. the remaining active note-level pressure is still concentrated in the carried `W026` residuals rather than in the registered-external packet family itself.

## 56. Current OxCalc Acknowledgment Of The W054 Consumer-Facade Direction
OxCalc has now reviewed OxFml's new consumer-facing interface rearchitecture direction under `W054` and the associated `OXFML_CONSUMER_INTERFACE_REARCHITECTURE_PLAN.md`.

Current OxCalc acknowledgment is:
1. yes, OxCalc agrees this should be treated as a coordinated downstream-consumer migration rather than as a crate-local cleanup,
2. yes, OxCalc agrees the consumer-facing packaging reset should not reopen the frozen OxFml <-> OxFunc seam,
3. yes, OxCalc agrees the first facade OxFml should build is the runtime facade because it is the only new facade family OxCalc needs for first uptake,
4. yes, OxCalc agrees the current supported Rust-facing surface remains valid until the facade modules actually exist,
5. yes, once the runtime facade exists and is settled enough, OxCalc intends to migrate onto that new seam directly rather than building a second long-lived OxCalc-local wrapper vocabulary over the current flat export set.

Current OxCalc non-assumptions are:
1. this is not read as permission to redefine candidate, commit, reject, trace, or runtime-library-context meaning on the OxCalc side,
2. this is not read as a request for OxCalc to introduce a permanent local abstraction layer that competes with the planned OxFml consumer facade,
3. this is not read as a freeze of final Rust type names before the runtime facade is actually implemented.

## 57. Current OxCalc Read Of Direct Impact
The immediate impact on OxCalc is concentrated in the current direct `oxfml_core` execution entrypoints and packet-carrier usage.

Current OxCalc read is:
1. the active local TreeCalc and upstream-host scaffolding lane currently depends directly on:
   - `SingleFormulaHost`
   - `HostRecalcOutput`
   - `FirstHostReplayCapturePacket`
   - `BindContext`
   - `FormulaSourceRecord`
   - `StructureContextVersion`
   - `AcceptDecision`
   - `LibraryContextSnapshot`
2. the first migration target for OxCalc should therefore be runtime/session entry and result packaging rather than editor-facing or replay-facing facades,
3. OxCalc still needs the runtime-facing surface to preserve the already-consumed host/runtime packet floor:
   - formula source and structure context,
   - caller anchor and address-mode-sensitive context where required,
   - defined-name, direct-binding, and table-context inputs,
   - provider-plus-pin library-context selection,
   - candidate / commit / reject / trace result families,
   - typed returned-value surface,
   - surfaced coordinator-relevant runtime facts,
4. OxCalc does not need OxFml to absorb coordinator policy into that facade; OxCalc only needs a cleaner consumer entry surface over the already-admitted semantic packet floor.

## 58. Current OxCalc Requests For The Runtime-Facade Migration Shape
The most useful runtime-facade refinements for OxCalc are packaging and migration clarifications rather than new semantic families.

Current OxCalc requests are:
1. please publish a concrete current-surface to target-facade migration table for the first OxCalc uptake, showing how the current supported entrypoints and packet families map onto:
   - `RuntimeEnvironment`
   - `RuntimeFormulaRequest`
   - `RuntimeFormulaResult`
   - `RuntimeSessionFacade`
2. please keep the runtime facade centered on the already-consumed packet truth rather than on new facade-only vocabulary where existing shared names are already stable,
3. please make provider-plus-pin library-context selection first-class in the runtime facade so OxCalc is not pushed back toward ambient mutable catalog reads during migration,
4. please keep candidate / commit / reject / trace output families visible in the runtime result/session facade rather than requiring OxCalc to stitch those back together from lower-level internal pieces,
5. please keep `value_delta`, `shape_delta`, and `topology_delta` explicitly distinct in the runtime-facing result shape, with optional `format_delta` and `display_delta` still explicit when present,
6. please keep the stand-in and replay-capture projection rule consistent with the new facade direction so OxCalc does not need one integration path for runtime execution and another unrelated path for deterministic capture.

## 59. Current OxCalc Problems And Improvement Opportunities
The main risks OxCalc sees are migration clarity and preserving the still-narrower residuals without hiding them under cleaner packaging.

Current OxCalc problem and opportunity list is:
1. migration clarity:
   - the largest practical risk for OxCalc is not semantic disagreement but a two-step churn where we first adapt to one partial packaging reset and then immediately adapt again to the real runtime facade,
   - a concrete migration matrix for the current direct OxCalc entrypoints would reduce that churn materially.
2. keep the W026 residuals visible:
   - caller-anchor and address-mode carriage,
   - execution-restriction transport breadth,
   - publication and topology consequence breadth
   should remain explicitly documented as `canonical but narrower` even after runtime-facade packaging exists, so cleaner packaging does not accidentally get read as broader seam closure.
3. caller-context dependence opportunity:
   - OxCalc would benefit if the runtime-facing bind or execution result preserved an explicit signal for whether caller-anchor or closely related caller context was actually semantically used for the first admitted relative-reference subset,
   - that would help OxCalc avoid over-conservative rebind and invalidation decisions without inventing new semantics locally.
4. coordinator-facing fact visibility opportunity:
   - OxCalc would benefit if execution-restriction, capability-sensitive, topology-sensitive, and dependency-sensitive surfaced facts remain reachable from the runtime-facing result in a consumer-oriented way rather than only through low-level artifact drilling,
   - this is a packaging request, not a request to redefine those facts.
5. replay-capture migration opportunity:
   - OxCalc would benefit if replay-capture projection remains a direct projection from runtime result or session result under the new facade,
   - that would let OxCalc retire proving-host-oriented helper usage at the same time it migrates onto the runtime facade rather than carrying a split integration model longer than necessary.

## 60. Current OxCalc Questions Back To OxFml On The Runtime-Facade Round
The next most useful OxFml reply for OxCalc would answer:
1. whether OxFml is willing to publish a concrete current-to-target migration table for the current OxCalc entrypoints listed in Section 57,
2. whether `RuntimeSessionFacade` is intended to subsume the session-lifecycle surface OxCalc will need for candidate / commit / reject coordination entry, or whether first uptake should expect a mixed facade-plus-lower-level session phase,
3. whether replay capture for the current OxCalc-facing lane is intended to project directly from `RuntimeFormulaResult` / `RuntimeSessionFacade` so current `FirstHostReplayCapturePacket` usage can migrate in the same wave,
4. whether OxFml wants caller-anchor dependence for the first admitted relative-reference subset surfaced explicitly in the runtime-facing result or bind metadata, or whether OxCalc should continue to derive that only from lower-level bind/reference artifacts,
5. whether OxFml agrees that W054 should preserve the current consume-now packet truth while leaving the carried `W026` residuals explicitly open rather than implicitly narrowed by packaging alone.

## 61. Current OxCalc Intake Of The Proposed Consumer-Facade Final Packet
OxCalc has now processed the updated OxFml upstream note together with:
1. `../OxFml/docs/spec/OXFML_CONSUMER_INTERFACE_AND_FACADE_CONTRACT_V1.md`
2. `../OxFml/docs/spec/OXFML_CONSUMER_INTERFACE_REARCHITECTURE_PLAN.md`
3. `../OxFml/docs/worksets/W054_consumer_facing_interface_rearchitecture_and_facade_packaging.md`

Current OxCalc read is:
1. the new consumer contract is strong enough to act as the implementation-driving OxFml packet for `W054`,
2. the runtime-first migration direction is the right packaging order for OxCalc,
3. the contract correctly preserves the hard boundary that packaging must not reopen the frozen OxFml <-> OxFunc seam,
4. the contract correctly keeps coordinator policy and publication semantics above the OxFml runtime facade,
5. the contract is close to a usable final consumer-packet shape for OxCalc migration, but OxCalc does not yet read it as a fully final shared seam-freeze text for runtime intake without a few narrower runtime-result clarifications.

## 62. Current OxCalc Accepted Parts Of The Proposed Final Packet
The following parts are accepted by OxCalc as the right target direction:
1. `RuntimeEnvironment`, `RuntimeFormulaRequest`, `RuntimeFormulaResult`, and `RuntimeSessionFacade` are the right first runtime-facing object set,
2. provider-plus-pin library-context selection remains explicit and first-class,
3. runtime/session result families should remain the preferred source for replay projection during migration,
4. candidate versus commit separation, reject-is-no-publish, and explicit `value_delta` / `shape_delta` / `topology_delta` preservation remain mandatory in the runtime-facing result shape,
5. current flat crate-root and proving-host entrypoints should remain transition compatibility only rather than the long-term OxCalc integration shape,
6. cleaner consumer packaging must not be read as closing the carried `W026` residuals by implication.

## 63. Current OxCalc Narrow Gaps Before Treating This As A Final Runtime Seam Packet
OxCalc still sees three narrow gaps that should be tightened before treating the new runtime consumer contract as effectively final for OxCalc-facing seam intake.

### 63.1 Correlation and fence-bearing runtime result fields remain too implicit
Current OxCalc concern:
1. `RuntimeFormulaResult` and `RuntimeSessionFacade` say they preserve candidate / commit / reject truth and replay-correlation handles,
2. but they do not yet explicitly name the stable coordinator-relevant correlation subset OxCalc needs to see preserved:
   - `candidate_result_id`
   - `commit_attempt_id`
   - `reject_record_id`
   - optional fence snapshot references where present.

Current OxCalc request:
1. please make that correlation subset explicit in the runtime-facing contract rather than leaving it only under generic replay-correlation wording.

### 63.2 Execution-restriction and dependency-sensitive surfaced facts remain too generic
Current OxCalc concern:
1. the proposed runtime result preserves runtime-effect and capability-sensitive facts in consumer-oriented canonical form,
2. but it does not yet explicitly say that first-slice execution-restriction facts, topology/effect fact refs, and dependency-sensitive surfaced facts remain reachable enough for coordinator use,
3. and those are still one of the few likely future narrow handoff triggers if live TreeCalc pressure increases.

Current OxCalc request:
1. please state explicitly in the runtime-facing contract that cleaner packaging does not hide or collapse:
   - execution-restriction-sensitive surfaced facts,
   - capability-sensitive surfaced facts,
   - topology/effect fact refs where they currently carry coordinator-relevant truth,
   - dependency-sensitive surfaced facts where publication or invalidation meaning depends on them.

### 63.3 Caller-context carriage is still not explicit enough for the first TreeCalc relative subset
Current OxCalc concern:
1. `RuntimeFormulaRequest` names optional per-request caller-anchor, direct-cell, or probe-only context,
2. but the proposed final consumer packet still does not state the first TreeCalc-facing caller-context floor as explicitly as the current `W026` residual lane does:
   - `FormulaSourceRecord`
   - `caller_anchor`
   - formula-channel and address-mode context
   - structure-context identity
3. nor does it yet say whether caller-anchor dependence for the first admitted relative-reference subset will be surfaced explicitly in runtime-facing bind or execution results.

Current OxCalc request:
1. please keep the `W026` Sequence 1 carried floor explicit in the runtime-facing contract,
2. and if OxFml is willing, add an explicit caller-context-dependence signal so OxCalc does not have to infer all rebind pressure from lower-level bind artifacts alone.

## 64. Current OxCalc Conclusion On The Proposed Final Seam Packet
Current conclusion is:
1. OxCalc accepts `OXFML_CONSUMER_INTERFACE_AND_FACADE_CONTRACT_V1.md` as the right implementation-driving consumer packet for `W054`,
2. OxCalc accepts the runtime-first migration shape and the current-surface to target-surface mapping direction,
3. OxCalc does not object to treating the contract as the target consumer architecture packet,
4. OxCalc does not yet treat it as a fully final runtime seam-freeze packet until the three narrow runtime-result and caller-context clarifications in Section 63 are either incorporated explicitly or rejected explicitly with rationale,
5. none of those gaps reopens the broad seam; they are bounded finish-work on the runtime-facing contract.

## 65. Current OxCalc Final Acceptance Of The Runtime Consumer Seam Packet
OxCalc has now processed the latest OxFml updates to:
1. `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md`
2. `../OxFml/docs/spec/OXFML_CONSUMER_INTERFACE_AND_FACADE_CONTRACT_V1.md`

Current OxCalc read is:
1. the previously open finish-pass items are now answered explicitly enough in the OxFml-owned consumer packet,
2. the contract now names the stable correlation subset OxCalc needs,
3. the contract now keeps surfaced execution-restriction, capability-sensitive, topology/effect, and dependency-sensitive fact families reachable enough for coordinator-facing consumption,
4. the contract now keeps the first admitted caller-context floor explicit and adds a caller-context dependence signal where OxFml can surface it honestly.

Current OxCalc acceptance is:
1. yes, OxCalc can now accept `OXFML_CONSUMER_INTERFACE_AND_FACADE_CONTRACT_V1.md` as the current seam spec for the runtime-facing migration and consumer-facade direction,
2. yes, OxCalc accepts it as the target packet to implement against for `W054`,
3. yes, OxCalc accepts that the remaining `W026` residuals stay explicitly narrower and note-level rather than blocking this runtime consumer packet,
4. yes, OxCalc is content to wait for the corresponding OxFml runtime-facade implementation wave before refactoring its own usage onto the new surface.

Current OxCalc non-assumptions remain:
1. this does not claim the facade modules already exist,
2. this does not collapse the carried `W026` residuals into closed broad semantic guarantees,
3. this does not move coordinator policy or publication ownership into OxFml,
4. this does not close existing handoff records by itself.

Current working rule from the OxCalc side is:
1. treat the consumer-facade contract as accepted seam truth for the next implementation wave,
2. await the corresponding OxFml implementation surfaces and migration table,
3. then refactor OxCalc usage directly onto that implemented seam rather than introducing a competing long-lived local abstraction.

## 66. Current OxCalc Intake After The Landed OxFml V1 Surface
OxCalc has now completed the first live uptake of the landed OxFml V1 consumer surface in local code.

Current OxCalc implementation read is:
1. the minimal upstream-host deterministic runtime path now executes through:
   - `oxfml_core::consumer::runtime::RuntimeEnvironment`
   - `oxfml_core::consumer::runtime::RuntimeFormulaRequest`
   - `oxfml_core::consumer::runtime::RuntimeFormulaResult`
2. the paired replay-facing deterministic projection now executes through:
   - `oxfml_core::consumer::replay::ReplayProjectionRequest`
   - `oxfml_core::consumer::replay::ReplayProjectionService`
   - `oxfml_core::consumer::replay::ReplayProjectionResult`
3. OxCalc no longer depends on direct `oxfml_core::host` access for that ordinary runtime/replay intake path,
4. the migrated slice passes local `oxcalc-core` tests after the uptake.

Current consequence for the OxCalc <-> OxFml seam read is:
1. the runtime/replay public-entry migration is now implementation-backed on the OxCalc side rather than only an accepted target packet,
2. the remaining live OxCalc-facing pressure is no longer about whether the V1 consumer facade exists or is adoptable,
3. the remaining pressure is the narrower W026 TreeCalc lane:
   - direct bind/reference intake breadth,
   - caller-context breadth,
   - execution-restriction transport breadth,
   - publication/topology breadth.

Current non-claim remains:
1. this does not claim the broader TreeCalc bind/reference seam is now closed,
2. this does not claim OxCalc has migrated every OxFml-facing callsite to facade-only surfaces,
3. this does not collapse note-level W026 residuals into broader shared seam closure.
