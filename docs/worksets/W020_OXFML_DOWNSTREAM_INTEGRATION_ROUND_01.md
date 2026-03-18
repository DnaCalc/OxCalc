# W020: OxFml Downstream Integration Round 01

## Purpose
Process the first stronger downstream OxFml observation round and inbound handoff after W018 without reopening W005.

This packet exists to:
1. intake OxFml's strengthened seam, replay, retained-witness, and host-boundary floor,
2. process `HANDOFF-FML-001` locally,
3. align OxCalc seam and replay planning docs to the stronger OxFml floor,
4. produce the first OxCalc `NOTES_FOR_OXFML.md` reply and decide whether any narrower follow-on handoff is required,
5. use that note as an explicit forward-and-back seam-alignment pass so remaining uncertainties are named rather than left implicit.

## Position and Dependencies
- **Depends on**: W018
- **Blocks**: W019 seam-sensitive replay widening, later OxFml integration rounds
- **Cross-repo**: consumes `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md` and `../OxFml/docs/handoffs/HANDOFF_FML_001_OXCALC_MINIMUM_SEAM_SCHEMAS.md`; may open a later narrower handoff only if exercised OxCalc evidence requires it

## Scope
### In scope
1. local receipt and register processing for `HANDOFF-FML-001`
2. local intake of the current OxFml downstream note
3. OxCalc-local seam and replay planning alignment where the stronger OxFml floor is already sufficient
4. first outbound OxCalc response note to OxFml
5. explicit decision on whether a new narrower handoff is required now

### Out of scope
1. reopening W005 history
2. pretending the OxCalc/OxFml seam is fully stabilized after one note-exchange round
3. changing OxFml canonical seam text directly from this repo
4. filing a new handoff without narrow exercised pressure

## Deliverables
1. local `HANDOFF-FML-001` receipt and register entry
2. OxCalc-owned `docs/upstream/NOTES_FOR_OXFML.md`
3. updated local ownership for the next OxFml integration round
4. explicit residual list for what still separates current intake from a future narrower handoff
5. explicit seam-alignment topics and open uncertainties recorded in the outbound note rather than only in local internal review
6. local recording of OxFml's returned topic-by-topic classifications so W019 can consume them as narrower bounded inputs rather than generic uncertainty

## Gate Model
### Entry gate
- `HANDOFF-FML-001` is filed on the OxFml side.
- `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md` exists.
- W018 has reached its declared gate.

### Exit gate
- `HANDOFF-FML-001` is recorded and acknowledged locally,
- `docs/upstream/NOTES_FOR_OXFML.md` exists and answers OxFml's current integration questions,
- local OxCalc seam and replay planning docs reflect the stronger OxFml floor where it already applies,
- the outbound note explicitly names the still-open seam uncertainties instead of treating them as implicit future cleanup,
- the returned OxFml note has been read back into local planning and the main topic classifications are reflected locally,
- the need for any narrower follow-on handoff is stated explicitly as `required now` or `not required yet`.

## Status
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - W019 still needs to consume the now-bounded OxFml replay and host-boundary floor in exercised evidence
  - the local seam docs still need to finish absorbing the returned OxFml classifications everywhere they matter
  - OxFml's latest OxFunc-refinement intake now explicitly says there is no new OxCalc-facing seam trigger yet; provider-failure and callable-publication remain watch lanes only
  - no final exercised decision exists yet on whether W019 evidence will require a narrower follow-on handoff
- claim_confidence: provisional
- reviewed_inbound_observations: `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md` and `HANDOFF-FML-001` are the direct inputs for this round; OxFml has now returned explicit topic classifications marking identity/fence, candidate/commit consequence shape, and host-query/direct-binding truth as already canonical, with dependency projection and semantic-display boundary narrower but still open; the latest OxFunc-refinement closure adds no new OxCalc-facing seam trigger yet and should currently be treated as watch-lane context rather than intake pressure
