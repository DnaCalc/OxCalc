# HANDOFF-FML-001 OxCalc Receipt and Integration Note

## Purpose
Record the OxCalc-side receipt of OxFml's `HANDOFF-FML-001` and summarize the local integration consequences for the next downstream OxFml round.

This note does not close the broader seam lane.
It records reviewed intake, the current OxCalc response posture, and the immediate follow-on ownership path.
It is the current OxCalc-side response note for `HANDOFF-FML-001`.

## Receiving-Side Sources
1. `../OxFml/docs/handoffs/HANDOFF_FML_001_OXCALC_MINIMUM_SEAM_SCHEMAS.md`
2. `../OxFml/docs/handoffs/HANDOFF_REGISTER.csv`
3. `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md`
4. `../OxFml/docs/spec/fec-f3e/FEC_F3E_DESIGN_SPEC.md`
5. `../OxFml/docs/spec/OXFML_REPLAY_APPLIANCE_ADAPTER_V1.md`
6. `../OxFml/docs/spec/OXFML_DNA_ONECALC_HOST_POLICY_BASELINE.md`

## Verified OxFml Intake Summary
OxFml is now explicitly asking OxCalc to consume a stronger shared seam and replay floor.

The key promoted OxFml-side direction is:
1. minimum typed schema objects for accepted candidate, commit, reject-context, and trace-correlation payload families,
2. stronger managed-session reject and no-publish paths,
3. stronger replay and retained-witness governance through the current local `cap.C3.explain_valid` floor,
4. an explicit downstream DNA OneCalc host boundary with direct-cell-binding preservation where semantic truth depends on concrete resolution.

## OxCalc Review Position
Current OxCalc position is:
1. the minimum typed schema direction is sufficient for the current OxCalc Stage 1 coordinator, replay, and retained-witness floor,
2. OxCalc should treat `candidate_result_id`, `commit_attempt_id`, `reject_record_id`, and optional fence snapshot references as the critical correlation floor for deterministic coordinator replay,
3. OxCalc should consume typed `FenceMismatchContext`, `CapabilityDenialContext`, and `SessionTerminationContext` as authoritative evaluator/runtime meanings rather than locally inventing broader generic failure classes,
4. OxCalc does not currently require a new coordinator-owned field family beyond the minimum OxFml objects, but it does expect surfaced execution-restriction and capability-sensitive effects to remain explicit where publication or replay interpretation depends on them,
5. direct-cell-binding preservation matters to retained witnesses and DNA OneCalc-facing host/scenario packs, but it does not authorize collapsing those host-sensitive truths into OxCalc coordinator policy.

## Integration Consequences
This receipt implies the following local actions:
1. record the inbound handoff locally rather than leaving it only in OxFml,
2. create a successor OxCalc integration-round workset rather than reopening W005,
3. update W019 to consume the stronger OxFml replay, retained-local, and pack-candidate boundary,
4. answer OxFml in `docs/upstream/NOTES_FOR_OXFML.md`,
5. decide later whether a narrower `HANDOFF-CALC-002` is required for coordinator-facing execution-restriction or publication-consequence pressure.

## Current Decision On Follow-On Handoff
No immediate follow-on handoff is required today.

Reason:
1. the current OxFml handoff and note appear sufficient for OxCalc's present Stage 1, W018, and planned W019 floor,
2. the remaining pressure is narrower and should be tied to exercised W019 evidence rather than filed spec-first again.

## Remaining Open Lanes
1. explicit OxCalc-side intake and reply still need to propagate into live workset ownership,
2. W019 still needs to consume OxFml's stronger replay, retained-local, and pack-candidate floor,
3. a later narrower handoff may still be needed if OxCalc requires stronger surfaced execution-restriction effects or broader topology/publication consequence fields than OxFml now exposes.

## Status
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - W020 intake and alignment work still needs to be executed locally
  - W019 still needs to absorb OxFml replay and host-boundary observations
  - no decision has yet been exercised on whether a later narrow follow-on handoff is required
