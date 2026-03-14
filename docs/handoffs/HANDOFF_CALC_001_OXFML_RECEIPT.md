# HANDOFF-CALC-001 OxCalc Receipt and Integration Note

## Purpose
Record the OxCalc-side receipt of OxFml's acknowledgment for `HANDOFF-CALC-001` and summarize the integration consequences for OxCalc-local spec and planning work.

This note does not close the originating workset.
It records receiving-repo awareness and the immediate follow-on implications.

## Receiving-Side Sources
1. `../OxFml/docs/handoffs/HANDOFF_CALC_001_OXFML_RESPONSE.md`
2. `../OxFml/docs/handoffs/HANDOFF_REGISTER.csv`
3. OxFml canonical seam updates under `../OxFml/docs/spec/`

## Verified OxFml Response Summary
OxFml acknowledged the handoff and adapted all four requested clause areas.

Clause decisions:
1. accepted-result payload structure: `adapt`
2. structured reject detail: `adapt`
3. fence consequences: `adapt`
4. runtime-derived effect reporting: `adapt` at the general-rule level; exhaustive taxonomy remains deferred pending exercised evidence

OxFml introduced or tightened:
1. an explicit `AcceptedCandidateResult` layer distinct from published `CommitBundle`,
2. typed no-publish consequences for fence and capability incompatibility,
3. clearer coordinator-relevant runtime-derived effect surfacing rules,
4. trace and replay expectations for candidate-versus-publication boundaries.

## OxCalc Integration Consequences
OxCalc should now align its local seam and coordinator wording to the OxFml canonical distinction between:
1. evaluator-produced `AcceptedCandidateResult`,
2. coordinator accept or reject,
3. committed published bundle.

OxCalc should also treat the following as now shared canonical direction:
1. reject-is-no-publish with typed fence and capability mismatch detail,
2. candidate-result versus publication separation in traces and replay,
3. runtime-derived effect surfacing as a general seam obligation.

OxCalc should not yet assume the runtime-derived effect taxonomy is exhaustive.
If tighter coordinator-facing obligations are needed later, they should be sent as a narrower follow-on handoff tied to replay-sensitive scenarios.

## Remaining Open Lanes
1. no replay artifacts yet exist in OxCalc for the new candidate-result versus publication boundary,
2. exhaustive runtime-derived effect taxonomy remains open,
3. W005 remains in progress until evidence and downstream alignment work advance further.

## Status
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - replay/evidence artifacts for candidate-result versus publication behavior
  - exhaustive runtime-derived effect taxonomy remains open
  - possible follow-on narrow handoff if OxCalc needs stronger surfaced effect requirements
