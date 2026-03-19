# W022: Execution Sequence H - Shared Pack Family and Direct-Binding Widening

## Purpose
Carry replay-pack work forward after W021 without reopening W021.

This packet exists to:
1. widen beyond the current semantic-only retained-local `TraceCalc` pack scope,
2. add retained-shared or pack-promoted witness-family evidence where justified,
3. exercise direct-binding-sensitive pack families where semantic truth depends on concrete binding identity,
4. decide whether that widened evidence finally justifies `cap.C5.pack_valid` or a narrower formal handoff.

## Position and Dependencies
- **Depends on**: W021
- **Blocks**: later shared replay-pack promotion lanes
- **Cross-repo**: may become the first packet that justifies a narrower `HANDOFF-CALC-002` if widened shared-pack or direct-binding evidence creates stronger seam pressure

## Scope
### In scope
1. retained-shared or pack-promoted witness-family widening
2. direct-binding-sensitive pack evidence
3. pack-grade validator widening beyond the current semantic-only retained-local floor
4. narrower handoff decision if widened evidence creates stronger shared seam pressure

### Out of scope
1. reopening W021's semantic-only pack contract slice
2. weakening lifecycle or direct-binding preservation rules to make pack promotion easier

## Deliverables
1. explicit execution packet for the next pack-grade widening lane
2. bounded evidence plan for retained-shared or pack-promoted witness families
3. bounded evidence plan for direct-binding-sensitive pack families
4. explicit handoff-decision criteria for any later `HANDOFF-CALC-002`
5. first checked-in direct-binding-sensitive retained-local baseline for the current local `TraceCalc` scope

## Gate Model
### Entry gate
- W021 has reached its declared gate.

### Exit gate
- widened shared-pack or direct-binding evidence either proves `cap.C5.pack_valid` or leaves a narrower, packetized residual with explicit blocker reasons and next steps.

## Execution Packet Additions
### Environment Preconditions
- required tools:
  - `cargo`
  - PowerShell for artifact checks
- optional tools:
  - none beyond the normal Rust toolchain

### Evidence Layout
- canonical artifact root:
  - `docs/test-runs/core-engine/tracecalc-retained-failures/w022-sequence1-direct-binding-family/`
- checked-in or ephemeral:
  - checked-in
- baseline run naming:
  - Sequence 1 direct-binding family baseline: `w022-sequence1-direct-binding-family`

### Replay-Corpus Readiness
- required retained-failure cases:
  - `rf_publication_fence_retained_local_001`
  - `rf_dynamic_dependency_retained_local_001`
  - `rf_direct_binding_sensitive_retained_local_001`
- reserve or later replay families:
  - retained-shared and pack-promoted witness families remain later sequences

### Coupled Widening Rule
- engine surfaces widened in this packet:
  - retained-failure pack-family metadata and direct-binding-sensitive validation state
- oracle/conformance surfaces widened in the same slice:
  - none in Sequence 1; ordinary `TraceCalc` semantics remain unchanged
- widened comparison artifact:
  - `docs/test-runs/core-engine/tracecalc-retained-failures/w022-sequence1-direct-binding-family/replay-appliance/validation/pack_grade_validation.json`

## Sequences
### Sequence 1: Direct-Binding Family Declaration
1. Add explicit pack-family and binding-identity requirement fields to retained-failure cases and emitted pack-facing artifacts.
2. Add one direct-binding-sensitive retained-local case to the fixture manifest.
3. Emit a checked-in baseline proving the direct-binding family is now exercised in pack-facing validation.

Exit condition:
1. a checked-in W022 baseline contains at least one `binding.direct_identity_required` retained-local case
2. run-level pack validation no longer reports `pack.grade.direct_binding_family.unexercised`
3. `cap.C5.pack_valid` remains unclaimed

### Sequence 2: Shared-Lifecycle Family Widening
1. Add retained-shared or pack-promoted witness-family evidence where justified.
2. Update pack validation to distinguish lifecycle coverage from pack-valid promotion.

Exit condition:
1. the remaining shared-lifecycle blocker is either cleared or narrowed explicitly

### Sequence 3: Capability And Handoff Decision
1. Decide whether the widened evidence justifies `cap.C5.pack_valid`.
2. Decide whether widened shared-pack evidence now justifies a narrower `HANDOFF-CALC-002`.
3. If either remains unproven, packetize the residual lane explicitly.

Exit condition:
1. capability decision and handoff decision are explicit

## Pre-Closure Verification Checklist
1. spec text and realization notes updated for all in-scope items: no
2. pack expectations updated for affected packs: no
3. at least one deterministic replay artifact exists per in-scope behavior: no
4. semantic-equivalence statement provided for policy or strategy changes: no
5. FEC/F3E cross-repo impact assessed and handoff filed if needed: no
6. all required tests pass: no
7. no known semantic gaps remain in declared scope: no
8. completion language audit passed: no
9. `IN_PROGRESS_FEATURE_WORKLIST.md` updated: no
10. `CURRENT_BLOCKERS.md` updated if needed: no

## Status
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - Sequence 1 has reached its gate with a checked-in direct-binding-sensitive retained-local baseline at `w022-sequence1-direct-binding-family`
  - retained-shared or pack-promoted witness-family widening remains open
  - the remaining pack-grade blocker is now `pack.grade.shared_lifecycle.unexercised`
  - no narrower follow-on handoff is justified yet
- claim_confidence: provisional
- reviewed_inbound_observations: W020 remains the carried downstream seam intake baseline; provider-failure and callable-publication remain watch lanes only until widened pack evidence creates stronger pressure
