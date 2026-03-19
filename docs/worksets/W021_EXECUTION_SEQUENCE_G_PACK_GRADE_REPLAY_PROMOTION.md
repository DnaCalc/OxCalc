# W021: Execution Sequence G - Pack-Grade Replay Promotion

## Purpose
Carry replay work forward after W019 without reopening W019.

This packet exists to:
1. define the first real `cap.C5.pack_valid` promotion lane,
2. convert rehearsal-only pack-candidate evidence into pack-grade validator and governance evidence,
3. close the remaining pack-grade blockers left explicit by W019.

## Position and Dependencies
- **Depends on**: W019, W020
- **Blocks**: later shared replay-pack promotion lanes
- **Cross-repo**: may become the first packet that justifies a narrower `HANDOFF-CALC-002` if exercised pack-grade evidence creates stronger shared seam pressure

## Scope
### In scope
1. pack-grade validator and bundle-evidence requirements
2. pack-candidate to pack-valid promotion rules
3. pack-governance and retained/shared lifecycle widening where justified
4. any narrower seam or replay-governance pressure discovered by exercised pack-grade evidence

### Out of scope
1. reopening W019's `cap.C4.distill_valid` lane
2. claiming pack validity without checked-in pack-grade evidence

## Deliverables
1. explicit execution packet for pack-grade replay promotion
2. concrete `cap.C5.pack_valid` evidence criteria
3. bounded list of narrower seam triggers that would justify a fresh handoff
4. first checked-in semantic-only pack-grade contract artifact for the current `TraceCalc` scope

## Gate Model
### Entry gate
- W019 has reached its declared gate.

### Exit gate
- `cap.C5.pack_valid` is either proven by checked-in evidence or left as a bounded residual with explicit blocker reasons and next evidence steps.

## Execution Packet Additions
### Environment Preconditions
- required tools:
  - `cargo`
  - PowerShell for artifact checks
- optional tools:
  - none beyond the normal Rust toolchain

### Evidence Layout
- canonical artifact root:
  - `docs/test-runs/core-engine/tracecalc-retained-failures/w021-pack-grade-gate-baseline/`
- checked-in or ephemeral:
  - checked-in
- baseline run naming:
  - Sequence 1 semantic-only pack contract baseline: `w021-sequence1-pack-contract`
  - W021 gate baseline: `w021-pack-grade-gate-baseline`

### Replay-Corpus Readiness
- required retained-failure cases:
  - `rf_publication_fence_retained_local_001`
  - `rf_dynamic_dependency_retained_local_001`
  - `rf_fallback_explanatory_only_001`
  - `rf_verify_clean_quarantined_001`
- reserve or later replay families:
  - broader display-sensitive and non-TraceCalc pack families remain later lanes

### Coupled Widening Rule
- engine surfaces widened in this packet:
  - retained-failure pack-facing validation and governance sidecars
- oracle/conformance surfaces widened in the same slice:
  - none in Sequence 1; ordinary `TraceCalc` semantics remain unchanged
- widened comparison artifact:
  - `docs/test-runs/core-engine/tracecalc-retained-failures/w021-sequence1-pack-contract/replay-appliance/validation/pack_grade_contract.json`

## Sequences
### Sequence 1: Semantic-Only Pack Contract
1. Convert W021 into a real execution packet with explicit semantic-only pack-grade criteria for the current `TraceCalc` scope.
2. Emit a run-level `pack_grade_contract.json` declaring the current semantic-only boundary, eligible retained-local family set, and remaining validator gap.
3. Check in one retained-failure baseline carrying that contract artifact.

Exit condition:
1. `pack_grade_contract.json` exists in a checked-in W021 run root
2. the current semantic-only pack scope is explicit in emitted evidence rather than only in prose
3. `cap.C5.pack_valid` remains unclaimed

### Sequence 2: Pack Validator Realization
1. Realize a run-level `pack_grade_validation.json`.
2. Prove pack-grade validation for the current semantic-only `TraceCalc` family or leave bounded blockers in the emitted validation artifact.
3. Keep validator output grounded in retained-local replay-valid witness families only.

Exit condition:
1. `pack_grade_validation.json` exists in the checked-in W021 baseline
2. any failure to reach `cap.C5.pack_valid` is bounded in emitted evidence rather than only in docs

### Sequence 3: Capability Promotion Decision
1. If Sequence 2 proves the semantic-only pack-grade floor, refresh the canonical capability manifest and docs to claim `cap.C5.pack_valid`.
2. If Sequence 2 leaves bounded blockers, keep the manifest conservative and carry those blockers forward explicitly.

Exit condition:
1. manifest, run-local capability snapshot, and checked-in validation artifacts agree on the highest honest capability claim

### Sequence 4: Narrower Handoff And Successor Decision
1. Decide whether exercised W021 evidence justifies a narrower `HANDOFF-CALC-002`.
2. If W021 does not fully discharge pack-grade promotion, author the explicit successor lane instead of leaving open backlog.

Exit condition:
1. handoff-decision state is explicit
2. any residual lane is packetized explicitly

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

## Completion Claim Self-Audit
1. Scope Re-Read: pass
2. Gate Criteria Re-Read: pass
3. Silent Scope Reduction Check: pass
4. "Looks Done But Is Not" Pattern Check: pass
5. Include Result: pass

## Status
- execution_state: complete
- scope_completeness: scope_complete
- target_completeness: target_complete
- integration_completeness: integrated
- open_lanes: none
- claim_confidence: validated
- reviewed_inbound_observations: W020 remained the carried downstream seam intake throughout W021; the packet did not justify a narrower `HANDOFF-CALC-002`, and the remaining replay-pack pressure is now explicitly packetized in W022 around retained-shared or pack-promoted witness families plus direct-binding-sensitive pack evidence
