# CORE_ENGINE_OXFML_SEAM.md

## 1. Purpose and Status
This document defines the OxCalc view of the OxFml seam for the rewritten core-engine spec set.

Status:
1. active rewrite baseline,
2. intended canonical OxCalc-local seam companion,
3. coordinator-facing in emphasis,
4. partially aligned to OxFml canonical seam updates from `HANDOFF-CALC-001`.

This document does not claim canonical ownership of the shared evaluator protocol.
OxFml remains the canonical owner of shared FEC/F3E seam specification.

This document exists to make OxCalc's coordinator-facing requirements explicit.

## 2. Ownership Rule
The seam is shared, but ownership is split.

### 2.1 OxFml Owns
OxFml owns:
1. formula grammar,
2. parse and bind semantics,
3. evaluator-side session and execution semantics,
4. canonical shared seam specification text,
5. evaluator-facing trace and result contracts where those are canonical seam artifacts.

### 2.2 OxCalc Owns
OxCalc owns:
1. coordinator acceptance and rejection consequences,
2. publication-fence requirements,
3. snapshot compatibility requirements from the coordinator side,
4. scheduling and stabilization interaction,
5. what evaluator-produced `AcceptedCandidateResult` artifacts must provide for coordinator-controlled publication.

### 2.3 Shared-Clause Rule
Where a clause is shared but canonical in OxFml, OxCalc must express its requirement locally and then hand off canonical text changes rather than silently diverging.

## 3. Why This Seam Must Be Explicit
The seam must be explicit because:
1. the evaluator is not the coordinator,
2. evaluator-produced `AcceptedCandidateResult` is not identical to committed publication,
3. replay and reject behavior depend on shared structure,
4. later concurrency makes weak seam wording unsafe.

If the seam is left implicit, publication, runtime state, and evaluator behavior will drift into one another.
The rewrite rejects that outcome.

## 4. OxCalc Expectations Of Evaluator Artifacts
OxCalc treats evaluator artifacts as immutable, versioned inputs.

The seam therefore requires that OxCalc be able to reason about:
1. which immutable evaluator artifact a candidate work unit is based on,
2. what token or version discipline guards that artifact,
3. what profile/version context applies,
4. what compatibility assumptions are being asserted by candidate work.

OxCalc does not need to own the evaluator internals to require this compatibility structure.

## 5. Candidate Work Boundary
The seam must expose a clear boundary between:
1. structural/evaluator inputs,
2. candidate evaluation work,
3. evaluator-produced `AcceptedCandidateResult`,
4. accepted publication consequences.

This distinction matters because the coordinator must be able to:
1. reject candidate work safely,
2. publish accepted work atomically,
3. preserve stable observer-visible state,
4. replay and diagnose accept/reject behavior.

## 6. Snapshot and Fence Requirements
From the OxCalc side, the seam must support coordinator reasoning about compatibility and fences.

At minimum, the seam must make it possible for the coordinator to determine:
1. which snapshot or structural basis candidate work depends on,
2. which evaluator artifact/token basis candidate work depends on,
3. whether profile/version assumptions match,
4. whether candidate work is eligible for publication under current coordinator state.

The exact canonical field names belong in shared seam specs, but the architectural requirement is fixed here.

## 7. Accepted Candidate Result Requirements
For accepted work, the seam must provide an evaluator-produced `AcceptedCandidateResult` structure rich enough for coordinator-controlled publication.

This means OxCalc must be able to receive or derive, through the seam, the information required to:
1. publish accepted results atomically,
2. update stable observer-visible derived state coherently,
3. integrate relevant topology/dependency consequences,
4. preserve replay and diagnostic fidelity.

The coordinator does not accept opaque success without adequate publication-relevant structure.

## 8. Reject Detail Requirements
Rejected work is architecturally no-publish.

But reject outcomes must still provide structured detail sufficient for:
1. deterministic replay,
2. diagnostics,
3. seam-hardening work,
4. staged concurrency analysis.

From the OxCalc side, the seam must support reject detail that distinguishes at least:
1. compatibility or fence mismatch,
2. artifact/token mismatch,
3. capability or session mismatch where relevant,
4. other coordinator-relevant reject classes that affect replay and migration understanding.

The canonical taxonomy belongs in shared seam work, but the requirement for structured detail is locked here.

## 9. Publication Ownership Rule
The seam must not blur evaluator success with committed publication.

The evaluator may produce an `AcceptedCandidateResult`.
The coordinator alone decides whether that result becomes committed published consequences.

Therefore the seam must preserve the distinction between:
1. evaluator-produced `AcceptedCandidateResult`,
2. coordinator-accepted publication,
3. rejected no-publish outcome.

## 10. Dynamic Dependency and Runtime-Derived Consequences
Where evaluator execution reveals runtime-relevant facts that matter to OxCalc coordination,
the seam must support explicit transmission or derivation of those facts.

This is necessary for cases such as:
1. runtime-observed dependency effects,
2. runtime capability or fence implications,
3. other evaluator-discovered facts that influence recalc or publication.

These effects must not be left as hidden evaluator internals if OxCalc is expected to coordinate on them.

## 11. Stage-1 Versus Later-Stage Seam Pressure

### 11.1 Stage 1
Stage 1 may realize a conservative subset of the full seam-hardening story.

But even in Stage 1, the seam must already preserve:
1. candidate-versus-publication distinction,
2. explicit compatibility or fence basis,
3. reject detail adequate for replay and diagnostics,
4. coordinator ownership of accept or reject consequences.

### 11.2 Later Stages
Later concurrent and async stages increase seam pressure.

They require stronger handling for:
1. contention and retry visibility,
2. fence mismatches under concurrent work,
3. deterministic replay of staged concurrency outcomes,
4. publication safety under overlapping candidate work.

The seam should therefore be written now with later hardening in mind.

## 12. Handoff Rule
Whenever OxCalc local requirements imply changes to canonical shared seam text, OxCalc must:
1. document the local requirement here,
2. prepare an explicit handoff packet for OxFml,
3. register the handoff,
4. avoid claiming the shared clause is fully resolved until the canonical side acknowledges it.

## 13. Formalization and Evidence Direction
This seam is assurance-relevant, not only integration-relevant.

Expected obligations include:
1. replay-visible candidate-versus-publication distinctions,
2. structured reject-detail coverage,
3. fence-safety modeling tied into coordinator assurance,
4. pack obligations for commit atomicity and reject determinism,
5. evidence artifacts sufficient for staged concurrency hardening.

## 14. Current Handoff State
`HANDOFF-CALC-001` has been filed and acknowledged.
The current shared direction now includes:
1. explicit `AcceptedCandidateResult` terminology at the OxFml seam,
2. typed no-publish reject detail for fence and capability incompatibility,
3. coordinator-relevant runtime-derived effect surfacing as a general seam rule.

Follow-on handoff pressure remains only where OxCalc later needs narrower or stronger requirements than the current shared canonical wording.

`HANDOFF-FML-001` has now also been received from OxFml.
That inbound handoff and the current OxFml downstream note strengthen the currently consumed floor with:
1. minimum typed schema objects for accepted candidate, commit, reject-context, and trace-correlation payload families,
2. a stronger managed-session baseline for stale-fence rejection, capability denial, session termination, and execution-restriction-sensitive no-publish paths,
3. a stronger replay and retained-local floor through the current OxFml-local `cap.C3.explain_valid` posture,
4. an explicit DNA OneCalc downstream host boundary that must not be mistaken for OxCalc coordinator policy.

The latest note-exchange round with OxFml also narrows several earlier uncertainties:
1. identity and fence vocabulary consumption is now treated as already canonical on the OxFml side,
2. candidate-result and commit-bundle consequence categories are now treated as already canonical on the OxFml side,
3. host-query and direct-binding-sensitive truth is now treated as already canonical on the OxFml side,
4. dependency consequence taxonomy and semantic-display boundary remain canonical but narrower rather than fully open.

## 15. OxCalc-Local Stage 1 Minimum Seam Packet

### 15.1 AcceptedCandidateResult Minimum
For Stage 1, OxCalc requires the shared seam to preserve enough information to derive or surface a minimum local `AcceptedCandidateResult` containing:
1. `candidate_result_id`
2. consumed identity and fence basis:
   - `formula_stable_id`
   - `formula_token`
   - `snapshot_epoch`
   - `bind_hash`
   - `profile_version`
   - important-but-still-narrower `capability_view_key`
3. trace and publication correlation:
   - `commit_attempt_id` where present
   - `reject_record_id` where relevant
   - optional `fence_snapshot_ref`
4. candidate publication-consequence categories:
   - `value_delta`
   - `shape_delta`
   - `topology_delta`
   - optional `format_delta`
   - optional `display_delta`
   - optional spill-event set
5. surfaced evaluator facts needed for coordinator correctness where not already derivable from the deltas
6. diagnostic and trace correlation metadata

This is an OxCalc-local minimum requirement for coordinator-controlled publication.
It does not claim that the shared OxFml-side canonical field names or artifact layering are identical.
But it now explicitly consumes the already-canonical OxFml category split rather than compressing it into generic local buckets alone.

### 15.2 Runtime-Derived Effect Subset
For Stage 1, OxCalc expects at least the following local runtime-derived effect subset to be preservable through the seam:
1. `dynamic_ref_activated`
2. `dynamic_ref_released`
3. `region_shape_activated`
4. `region_shape_released`
5. `capability_observed`
6. `format_observed`
7. `execution_restriction_observed`

This subset is the local coordinator and overlay floor.
It is not a claim that the broader shared runtime-derived effect taxonomy is closed.
Current shared reading after the latest note round:
1. execution-restriction effects are stable enough to consume semantically now,
2. OxCalc should not yet assume one final frozen single-object carrier for those effects,
3. dependency additions, removals, and reclassifications remain intended evaluator/runtime facts, but their exact retained/reduced witness projection closure is still narrower than a fully frozen universal rule.

### 15.3 Reject Subset
For Stage 1, OxCalc expects the shared seam to support a local typed reject subset covering at least:
1. `snapshot_mismatch`
2. `artifact_token_mismatch`
3. `profile_version_mismatch`
4. `capability_mismatch`
5. `publication_fence_mismatch`
6. `dynamic_dependency_failure`
7. `synthetic_cycle_reject`
8. `host_injected_failure`

This is the minimum local reject floor needed for coordinator no-publish behavior, replay classification, and self-contained harness scenarios.
It does not claim that the shared OxFml-side canonical taxonomy or ownership split is fully closed.

The current stronger OxFml-managed baseline makes the following canonical context families especially important to preserve without coordinator reinterpretation:
1. `FenceMismatchContext`
2. `CapabilityDenialContext`
3. `SessionTerminationContext`
4. `DynamicReferenceFailureContext`

### 15.4 Host-Boundary Preservation Rule
OxCalc does not own DNA OneCalc host policy.
But where retained witnesses, pack-candidate artifacts, or replay-valid scenarios depend on concrete host-sensitive truth, OxCalc must preserve the OxFml-declared direct-binding boundary rather than collapsing those cases into name-only or prose-only artifacts.

This is a replay and evidence-preservation rule.
It is not a transfer of host-policy ownership into OxCalc.

Current shared reading after the latest note round:
1. typed host-query capability views are already canonical on the OxFml side,
2. direct-cell-binding-sensitive truth is already canonical on the OxFml side where semantic correctness depends on concrete resolution,
3. the broader naming and indexing convention for direct-binding-sensitive pack-candidate families remains open and belongs to later replay widening rather than immediate seam redefinition.

## 16. Open Detailed Questions
These remain seam-hardening questions rather than reasons to weaken the split:
1. exact accepted-result payload naming and artifact partition in shared canonical terms,
2. exact reject taxonomy ownership partition beyond the now-locked Stage 1 local subset,
3. exact broader runtime-derived effect taxonomy beyond the Stage 1 local subset, especially execution-restriction and capability-sensitive transport closure,
4. exact retained/reduced witness projection closure for dependency additions, removals, and reclassifications,
5. exact trace schema mapping for coordinator-facing replay and diagnostics, especially stable use of `candidate_result_id`, `commit_attempt_id`, `reject_record_id`, and optional fence snapshot references,
6. exact replay-facing preservation rule for direct-binding-sensitive witness and pack-candidate families once W019 broadens them,
7. exact shared reading of semantic-format versus display-facing publication consequences before broader retained and pack-candidate widening.

## 17. Status
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - replay artifacts not yet attached for candidate-result versus publication boundaries,
  - the Stage 1 local seam packet now consumes more of the already-canonical OxFml category split, but narrower projection-closure questions remain open,
  - W020 and W019 still need to consume the stronger OxFml inbound handoff and downstream note,
  - a narrower follow-on handoff is not required yet, but remains an explicit later decision if W019 evidence creates stronger coordinator pressure


