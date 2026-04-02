# W031: TreeCalc Assurance Refresh and Residual Packetization

## Purpose
Refresh the assurance, replay, and residual planning surfaces around the first live TreeCalc engine path so the proving surfaces no longer bind only to the older proving substrate.
This packet refreshes assurance and residual planning around the engine that sits beneath the existing `OxCalcTree` host-facing consumer contract.

## Position and Dependencies
- **Depends on**: W030
- **Blocks**: later concurrency and optimization promotion lanes that rely on the first TreeCalc-ready engine as semantic authority
- **Cross-repo**: narrower handoff only if the TreeCalc-ready baseline exposes a genuine consumed-seam insufficiency not already packetized

## Scope
### In scope
1. W008/W009/W010/W012 assurance refresh against the live TreeCalc engine path
2. replay and pack-binding refresh where object names or transition meaning changed materially
3. residual packetization for any semantic gaps that remain after the first TreeCalc-ready baseline
4. explicit statement of what later performance work may assume semantically

### Out of scope
1. concurrency realization itself
2. later economics-tuned optimization waves
3. broader grid substrate expansion

## Deliverables
1. refreshed assurance and replay-binding notes around the live TreeCalc path
2. explicit residual packets for any uncovered TreeCalc semantic gaps
3. an updated roadmap/workset state that treats the first TreeCalc-ready baseline as the new semantic authority
4. explicit carry-forward guardrails for later optimization and concurrency work

## Gate Model
### Entry gate
- W030 has produced the first checked-in sequential TreeCalc-ready baseline
- any material object-model or transition changes from the proving substrate are known
- the `OxCalcTree` consumer contract remains the host-facing contract to preserve while assurance and residual packetization are refreshed underneath it

### Exit gate
- no major semantic clause remains bound only to the older proving substrate
- later optimization and concurrency packets have a refreshed semantic authority to depend on
- remaining gaps are packetized explicitly rather than left as prose ambiguity

## Pre-Closure Verification Checklist
1. Spec text and realization notes updated for all in-scope items: no
2. Pack expectations updated for affected packs: no
3. At least one deterministic replay artifact exists per in-scope behavior: no
4. Semantic-equivalence statement provided for policy or strategy changes: no
5. FEC/F3E cross-repo impact assessed and handoff filed if needed: no
6. All required tests pass: no
7. No known semantic gaps remain in declared scope: no
8. Completion language audit passed: no
9. `IN_PROGRESS_FEATURE_WORKLIST.md` updated: no
10. `CURRENT_BLOCKERS.md` updated if needed: no

## Status
- execution_state: planned
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - this packet has not yet refreshed assurance and replay bindings around the landed `OxCalcTree` host-facing contract and the engine beneath it
  - assurance and replay bindings are still centered on the proving substrate
  - no TreeCalc-ready assurance refresh has been executed yet
  - residual TreeCalc semantic gaps have not been packetized yet
- claim_confidence: draft
- reviewed_inbound_observations: current OxFml downstream baseline remains sufficient unless the first TreeCalc-ready baseline reveals narrower seam pressure
