# W019: Execution Sequence F - Replay Distill and Pack Promotion

## Purpose
Carry replay-appliance work forward after W018 without reopening W018.

This packet exists to:
1. promote replay-appliance evidence from `cap.C3.explain_valid` toward `cap.C4.distill_valid`,
2. widen reduced-witness bundle evidence beyond the first retained-failure baseline,
3. define the first honest path toward `cap.C5.pack_valid`,
4. keep pack-grade claims gated behind explicit bundle-valid and lifecycle-valid evidence,
5. consume the now-bounded OxFml seam inputs from W020 as explicit execution constraints rather than as generic background uncertainty.

## Position and Dependencies
- **Depends on**: W016, W018, W020
- **Blocks**: later replay-pack promotion lanes
- **Cross-repo**: consumes OxFml's stronger replay, retained-local, pack-candidate, and host-boundary floor through W020; any narrower shared seam or replay-governance pressure still routes through the existing Foundation replay handoff and OxCalc seam doctrine

## In Scope
1. dedicated reduced-witness bundle-valid promotion evidence for `cap.C4.distill_valid`
2. broader mismatch-family explain and retained-local widening
3. pack-candidate policy and first non-pack / pack-candidate separation rules
4. first pack-grade validator and evidence requirements for `cap.C5.pack_valid`
5. preservation of OxFml-authoritative replay-safe identity, fence meaning, and direct-binding-sensitive witness truth where those matter to retained or pack-candidate artifacts
6. explicit consumption of the already-canonical OxFml identity/fence subset and candidate/commit consequence categories in replay-facing reduced and retained artifacts
7. explicit narrowing of the still-open dependency-projection and semantic-display-boundary questions through exercised evidence rather than note-only discussion

## Out Of Scope
1. silently widening current capability claims without fresh checked-in evidence
2. weakening lifecycle or quarantine semantics to make pack promotion easier
3. replacing OxCalc-owned `TraceCalc`, reference-machine, or engine-diff semantics
4. treating note-level alignment as closure of retained/reduced witness projection questions without exercised artifacts

## Deliverables
1. an execution-sequenced successor packet after W018
2. explicit `cap.C4.distill_valid` evidence criteria
3. explicit `cap.C5.pack_valid` evidence criteria
4. widened retained-failure and reduced-witness baseline planning
5. explicit statement of which OxFml seam inputs are treated as already canonical versus canonical-but-narrower for this execution wave
6. explicit residual list for what would still justify a narrower `HANDOFF-CALC-002` after W019 evidence

## Gate Model
### Entry gate
- W018 has reached its declared gate.
- checked-in replay-appliance-aware ordinary and retained-failure baselines exist.
- current capability claim has been refreshed honestly through `cap.C3.explain_valid`.
- W020 has recorded the current OxFml downstream note and `HANDOFF-FML-001` intake.
- W020 has recorded OxFml's returned topic-by-topic classifications.

### Exit gate
- successor execution slices for `cap.C4.distill_valid` and later `cap.C5.pack_valid` are explicit,
- reduced-witness promotion evidence is no longer implicit future work,
- the W020-bounded seam topics are reflected in exercised artifact design, with:
  - identity/fence and candidate/commit consequence categories treated as consumed canonical inputs,
  - dependency retained/reduced projection and semantic-display boundary treated as narrowed evidence questions,
- any remaining pack-grade gaps are named as later bounded lanes rather than open-ended backlog.

## Status
- execution_state: planned
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - W019 is authored as the successor packet but no realization slices have started yet
  - direct-binding-sensitive retained and pack-candidate policy still needs exercised OxCalc evidence
  - dependency additions/removals/reclassifications still need retained/reduced witness projection closure in exercised artifacts
  - semantic-format versus display-facing boundary still needs exercised replay-facing narrowing before broader pack-candidate widening
  - `cap.C4.distill_valid` and `cap.C5.pack_valid` remain unproven
- claim_confidence: draft
- reviewed_inbound_observations: OxFml's current downstream note and `HANDOFF-FML-001` widen the consumed floor for typed reject contexts, execution-restriction effects, retained-local and pack-candidate governance, and DNA OneCalc host-boundary preservation; the returned classification pass now treats identity/fence vocabulary, candidate/commit consequence categories, and host-query/direct-binding-sensitive truth as already canonical, while leaving dependency projection and semantic-display boundary canonical but narrower
