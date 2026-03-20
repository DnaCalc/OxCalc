# W023: Execution Sequence I - Program-Grade Pack Promotion Beyond Semantic-Only TraceCalc Scope

## Purpose
Carry replay-pack work forward after W022 without reopening W022.

This packet exists to:
1. widen beyond the current semantic-only `TraceCalc` retained-failure pack scope,
2. prove or reject program-grade pack promotion against broader host and shared-lifecycle requirements,
3. decide whether broader direct-binding-sensitive host families create a real OxFml seam trigger,
4. leave any later `cap.C5.pack_valid` claim grounded in shared program evidence rather than local semantic-only rehearsal coverage.

## Position and Dependencies
- **Depends on**: W022
- **Blocks**: later program-grade replay-pack promotion lanes
- **Cross-repo**: may become the first packet that justifies a narrower `HANDOFF-CALC-002` if broader host-sensitive or shared-pack evidence creates stronger seam pressure

## Scope
### In scope
1. broader host-sensitive and shared-lifecycle retained witness families
2. widened direct-binding families beyond the current local semantic-only floor
3. explicit program-scope pack-grade validator widening
4. narrower handoff decision if broader pack evidence creates shared seam pressure

### Out of scope
1. weakening W022 semantic-only preservation discipline
2. claiming program-grade promotion on local-only semantic rehearsal evidence

## Deliverables
1. execution packet for the next pack-grade widening lane
2. explicit program-scope residual criteria beyond W022
3. narrower handoff criteria for any later `HANDOFF-CALC-002`

## Gate Model
### Entry gate
- W022 has reached its declared gate.

### Exit gate
- broader program-grade pack evidence either proves `cap.C5.pack_valid` or leaves a narrower, packetized residual with explicit blocker reasons and next steps.

## Execution Packet Additions
### Environment Preconditions
- required tools:
  - `cargo`
  - PowerShell for artifact checks
- optional tools:
  - none beyond the normal Rust toolchain

### Evidence Layout
- canonical artifact root:
  - `docs/test-runs/core-engine/tracecalc-retained-failures/w023-sequence2-host-sensitive-family/`
- checked-in or ephemeral:
  - checked-in
- baseline run naming:
  - Sequence 1 program-scope contract baseline: `w023-sequence1-program-scope-contract`
  - Sequence 2 host-sensitive family baseline: `w023-sequence2-host-sensitive-family`

### Replay-Corpus Readiness
- required retained-failure cases:
  - `rf_publication_fence_retained_local_001`
  - `rf_dynamic_dependency_retained_local_001`
  - `rf_publication_fence_retained_shared_001`
  - `rf_direct_binding_sensitive_retained_local_001`
  - `rf_host_sensitive_direct_binding_retained_shared_001`
- reserve or later replay families:
  - broader host-sensitive direct-binding families
  - provider-failure or callable-publication-sensitive witness families if activated upstream

### Coupled Widening Rule
- engine surfaces widened in this packet:
  - program-scope pack contract and validation sidecars over the retained-failure runner
  - broader host-sensitive direct-binding family coverage in retained-shared witness families
- oracle/conformance surfaces widened in the same slice:
  - none in Sequences 1 and 2; ordinary `TraceCalc` semantics remain unchanged
- widened comparison artifact:
  - `docs/test-runs/core-engine/tracecalc-retained-failures/w023-sequence2-host-sensitive-family/replay-appliance/validation/program_grade_validation.json`

## Sequences
### Sequence 1: Program-Scope Contract And Blocker Codification
1. Emit explicit program-scope pack contract and validation artifacts over the current semantic-only retained-failure family.
2. Make the broader host-sensitive and program-lifecycle blockers explicit in emitted evidence rather than only in prose.
3. Keep `cap.C5.pack_valid` unclaimed.

Exit condition:
1. a checked-in W023 baseline contains `program_grade_contract.json` and `program_grade_validation.json`
2. the emitted validator states that semantic-only family coverage is reached but program-grade promotion remains blocked
3. no narrower handoff is required yet

### Sequence 2: Broader Host-Sensitive Family Widening
1. Add broader host-sensitive direct-binding or host-query-dependent witness families beyond the current semantic-only local floor.
2. Reassess whether broader host identity and lifecycle evidence narrows the remaining blocker set.

Exit condition:
1. broader host-sensitive family coverage is either exercised or narrowed explicitly
2. `program_grade_validation.json` narrows the blocker set beyond the Sequence 1 local semantic-only floor

### Sequence 3: Capability And Handoff Reassessment
1. Decide whether the widened evidence justifies `cap.C5.pack_valid`.
2. Decide whether broader program-grade evidence now justifies a narrower `HANDOFF-CALC-002`.
3. If either remains unproven, packetize the residual lane explicitly.

Exit condition:
1. capability decision and handoff decision are explicit

## Pre-Closure Verification Checklist
1. spec text and realization notes updated for all in-scope items: yes
2. pack expectations updated for affected packs: yes
3. at least one deterministic replay artifact exists per in-scope behavior: yes
4. semantic-equivalence statement provided for policy or strategy changes: yes
5. FEC/F3E cross-repo impact assessed and handoff filed if needed: yes
6. all required tests pass: yes
7. no known semantic gaps remain in declared scope: yes
8. completion language audit passed: yes
9. `IN_PROGRESS_FEATURE_WORKLIST.md` updated: yes
10. `CURRENT_BLOCKERS.md` updated if needed: no new blocker entries were required

## Status
- execution_state: complete
- scope_completeness: scope_complete
- target_completeness: target_complete
- integration_completeness: integrated
- open_lanes:
  - none
- claim_confidence: provisional
- reviewed_inbound_observations: W020 remains the carried downstream seam intake baseline until broader program-grade pack evidence creates stronger pressure
