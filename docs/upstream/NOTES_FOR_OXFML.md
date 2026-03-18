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
