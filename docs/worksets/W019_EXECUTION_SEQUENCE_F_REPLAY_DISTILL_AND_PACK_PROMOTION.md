# W019: Execution Sequence F - Replay Distill and Pack Promotion

## Purpose
Carry replay-appliance work forward after W018 without reopening W018.

This packet exists to:
1. promote replay-appliance evidence from `cap.C3.explain_valid` to `cap.C4.distill_valid`,
2. widen reduced-witness bundle evidence beyond the first retained-failure baseline,
3. define the first honest path toward `cap.C5.pack_valid` without overclaiming it,
4. consume the now-bounded OxFml seam inputs from W020 as explicit execution constraints,
5. leave any remaining pack-grade work to an explicit successor packet rather than open-ended backlog.

## Position and Dependencies
- **Depends on**: W016, W018, W020
- **Blocks**: W021
- **Cross-repo**: consumes W020's OxFml downstream classifications; no narrower follow-on handoff is filed in this packet because the exercised W019 evidence did not yet require one

## Scope
### In scope
1. dedicated reduced-witness bundle-valid promotion evidence for `cap.C4.distill_valid`
2. retained-failure widening for dependency-projection-sensitive reduced witnesses
3. explicit semantic-format versus display-facing boundary narrowing for the current Stage 1 `TraceCalc` scope
4. pack-candidate rehearsal policy and first non-pack / pack-candidate separation artifacts
5. capability and assurance refresh after checked-in evidence exists
6. explicit successor-packet definition for later `cap.C5.pack_valid`

### Out of scope
1. claiming `cap.C5.pack_valid`
2. widening ordinary `TraceCalc` semantics beyond additive sidecars
3. weakening lifecycle or quarantine semantics to make pack promotion easier
4. filing `HANDOFF-CALC-002` without narrower exercised pressure

## Deliverables
1. checked-in retained-failure distillation baseline with replay-appliance bundle roots
2. replay-valid reduced-scenario artifacts for retained-local witness families
3. run-level `distill_validation.json` proving `cap.C4.distill_valid`
4. run-level `pack_candidate_validation.json` proving rehearsal-only pack-candidate separation without claiming pack validity
5. widened dependency-projection evidence through a dynamic-dependency retained-local case
6. updated capability manifest and replay docs reflecting the highest honest claim through `cap.C4.distill_valid`
7. explicit successor packet `W021` for later pack-grade promotion

## Gate Model
### Entry gate
- W018 has reached its declared gate.
- checked-in replay-appliance-aware ordinary and retained-failure baselines exist.
- current capability claim has been refreshed honestly through `cap.C3.explain_valid`.
- W020 has recorded OxFml's returned topic-by-topic classifications.

### Exit gate
- `cap.C4.distill_valid` is backed by checked-in reduced-witness bundle evidence.
- dependency retained/reduced projection is exercised in at least one replay-valid retained-local reduced witness.
- semantic-format versus display-facing boundary is narrowed explicitly in emitted evidence for the current Stage 1 scope.
- pack-candidate rehearsal is explicit and non-pack, with `cap.C5.pack_valid` still unclaimed.
- the remaining pack-grade gap is moved into an explicit successor packet rather than open-ended backlog.

## Execution Packet Additions
### Environment Preconditions
- required tools:
  - `cargo`
  - PowerShell for parity checks
- optional tools:
  - none for this packet beyond the normal Rust toolchain

### Evidence Layout
- canonical artifact roots:
  - `docs/test-runs/core-engine/tracecalc-reference-machine/w019-replay-distill-baseline/`
  - `docs/test-runs/core-engine/tracecalc-retained-failures/w019-distill-and-pack-candidate-baseline/`
- checked-in or ephemeral:
  - checked-in
- baseline run naming:
  - ordinary additive replay baseline: `w019-replay-distill-baseline`
  - retained-failure distill baseline: `w019-distill-and-pack-candidate-baseline`

### Replay-Corpus Readiness
- required replay classes with scenario ids:
  - `R3`, `R6`: `tc_publication_fence_reject_001`
  - `R8`: `tc_dynamic_dep_switch_001`
  - `R7`: `tc_verify_clean_no_publish_001`
- reserve or later replay classes:
  - pack-grade replay families remain later and are carried by `W021`

### Coupled Widening Rule
- engine surfaces widened in this packet:
  - retained-failure distillation and pack-candidate rehearsal output
- oracle/conformance surfaces widened in the same slice:
  - reduced-scenario replay validation against the reference machine and engine adapter
- widened comparison artifact:
  - `docs/test-runs/core-engine/tracecalc-retained-failures/w019-distill-and-pack-candidate-baseline/replay-appliance/validation/distill_validation.json`

## Sequences
### Sequence 1: Distill-Valid Artifact Family
1. Add reduced-scenario emission for retained-failure witness bundles.
2. Emit per-case `distillation_manifest.json` and `distill_validation.json`.
3. Prove replay-valid reduced witnesses for retained-local cases.

Exit condition:
1. at least one retained-local case reaches `distill_valid`
2. run-level `distill_validation.json` reaches `cap.C4.distill_valid`

### Sequence 2: Dependency Projection Narrowing
1. Add a retained-local dynamic-dependency case.
2. Preserve dependency additions/removals/effects explicitly in reduced-witness output.

Exit condition:
1. dependency projection is no longer note-only for current `TraceCalc` scope

### Sequence 3: Pack-Candidate Rehearsal
1. Emit per-case `pack_candidate_assessment.json`.
2. Emit run-level `pack_candidate_validation.json`.
3. Keep all outputs non-pack and explicitly blocked from `cap.C5.pack_valid`.

Exit condition:
1. rehearsal-only pack-candidate state exists
2. `cap.C5.pack_valid` remains explicitly unclaimed

### Sequence 4: Capability and Successor Refresh
1. Refresh ordinary and retained-failure baseline evidence.
2. Raise the canonical capability claim through `cap.C4.distill_valid`.
3. Author successor packet `W021`.

Exit condition:
1. docs, manifest, and baselines all align to the new capability floor

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
10. `CURRENT_BLOCKERS.md` updated (new or resolved): no new blocker entries were required

## Status
- execution_state: complete
- scope_completeness: scope_complete
- target_completeness: target_complete
- integration_completeness: integrated
- open_lanes: none
- claim_confidence: validated
- reviewed_inbound_observations: W020's OxFml intake remains the governing downstream input; W019 consumed the already-canonical identity/fence and candidate/commit consequence categories directly, while narrowing dependency projection and semantic-display boundary through exercised retained-failure evidence rather than note-only discussion
