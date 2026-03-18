# W019: Execution Sequence F - Replay Distill and Pack Promotion

## Purpose
Carry replay-appliance work forward after W018 without reopening W018.

This packet exists to:
1. promote replay-appliance evidence from `cap.C3.explain_valid` toward `cap.C4.distill_valid`,
2. widen reduced-witness bundle evidence beyond the first retained-failure baseline,
3. define the first honest path toward `cap.C5.pack_valid`,
4. keep pack-grade claims gated behind explicit bundle-valid and lifecycle-valid evidence.

## Position and Dependencies
- **Depends on**: W016, W018
- **Blocks**: later replay-pack promotion lanes
- **Cross-repo**: any shared replay-governance or registry pressure still routes through the existing Foundation replay handoff and OxCalc seam doctrine

## In Scope
1. dedicated reduced-witness bundle-valid promotion evidence for `cap.C4.distill_valid`
2. broader mismatch-family explain and retained-local widening
3. pack-candidate policy and first non-pack / pack-candidate separation rules
4. first pack-grade validator and evidence requirements for `cap.C5.pack_valid`

## Out Of Scope
1. silently widening current capability claims without fresh checked-in evidence
2. weakening lifecycle or quarantine semantics to make pack promotion easier
3. replacing OxCalc-owned `TraceCalc`, reference-machine, or engine-diff semantics

## Deliverables
1. an execution-sequenced successor packet after W018
2. explicit `cap.C4.distill_valid` evidence criteria
3. explicit `cap.C5.pack_valid` evidence criteria
4. widened retained-failure and reduced-witness baseline planning

## Gate Model
### Entry gate
- W018 has reached its declared gate.
- checked-in replay-appliance-aware ordinary and retained-failure baselines exist.
- current capability claim has been refreshed honestly through `cap.C3.explain_valid`.

### Exit gate
- successor execution slices for `cap.C4.distill_valid` and later `cap.C5.pack_valid` are explicit,
- reduced-witness promotion evidence is no longer implicit future work,
- any remaining pack-grade gaps are named as later bounded lanes rather than open-ended backlog.

## Status
- execution_state: planned
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - W019 is authored as the successor packet but no realization slices have started yet
  - `cap.C4.distill_valid` and `cap.C5.pack_valid` remain unproven
- claim_confidence: draft
- reviewed_inbound_observations: `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md` missing
