# W003: Stage 1 Coordinator and Publication Baseline

## Purpose
Define the implementation-facing Stage 1 coordinator packet for accept or reject discipline, atomic publication, and stable observer-visible state.

## Position and Dependencies
- **Depends on**: W001, W002, W005
- **Blocks**: W004, W006
- **Cross-repo**: aligned to accepted OxFml seam direction from `HANDOFF-CALC-001`

## Scope
### In scope
1. Coordinator state-machine boundary for admission, compatibility check, acceptance, rejection, and publication.
2. Stage 1 `AcceptedCandidateResult` intake boundary and commit-bundle publication boundary.
3. Typed reject consequence mapping for fence, token, capability, and other coordinator-relevant no-publish cases.
4. Stable observer-visible publication contract for Stage 1 sequential execution.

### Out of scope
1. Stage 2 concurrency throughput realization.
2. Full seam closure beyond the already acknowledged shared direction.
3. Complete TLA+ artifact authoring.

## Deliverables
1. Coordinator transition packet covering admit, accept, reject, publish, and stable-view transitions.
2. Publication packet covering Stage 1 `AcceptedCandidateResult` intake, atomic commit-bundle publication, and reject-is-no-publish consequences.
3. Reject-detail packet covering the coordinator-local mapping that W008 and W009 must later consume.
4. Stage 1 local packet summary for candidate-result intake, publish bundle derivation, and reject classes.

## Gate Model
### Entry gate
- W001 is integrated.
- W002 is sufficiently resolved for coordinator-facing structural and snapshot assumptions.
- W005 accepted seam direction is available for candidate-result versus publication terminology.

### Exit gate
- Stage 1 coordinator transitions are explicit enough to implement without re-opening seam ownership questions.
- Publication and reject boundaries are explicit enough to drive replay planning and TLA+ safety planning.
- Stable observer-visible state rules are explicit enough to bind to Stage 1 trace and pack planning.
- The minimum Stage 1 candidate-result and reject packet shape is explicit in OxCalc-local terms.

## Stage 1 Local Candidate-Result Intake Packet
The minimum Stage 1 OxCalc-local `AcceptedCandidateResult` intake packet should contain:
1. `candidate_result_id`
2. `struct_snapshot_id`
3. `artifact_token_basis`
4. `compatibility_basis`
5. `target_set`
6. `value_updates`
7. `dependency_shape_updates`
8. `runtime_effects`
9. `diagnostic_events`

### Field Intent
1. `candidate_result_id`: stable identity for candidate-versus-publication separation and replay.
2. `struct_snapshot_id`: immutable structural basis the candidate result is computed against.
3. `artifact_token_basis`: evaluator artifact or token basis needed for fence and staleness checks.
4. `compatibility_basis`: coordinator-facing summary of the admission assumptions being asserted.
5. `target_set`: nodes or regions the result claims to cover.
6. `value_updates`: accepted value-state deltas for publication consideration.
7. `dependency_shape_updates`: explicit dependency or region-shape consequences.
8. `runtime_effects`: coordinator-relevant runtime-derived facts that may affect overlays or publication.
9. `diagnostic_events`: replay- and diagnostics-facing emitted details.

## Stage 1 Local Publish Bundle
The minimum Stage 1 OxCalc-local commit bundle derived from an accepted candidate result should contain:
1. `publication_id`
2. `candidate_result_id`
3. `published_view_delta`
4. `published_runtime_effects`
5. `counter_deltas`
6. `trace_markers`

This bundle is the atomic observer-visible publication surface for Stage 1.

## Stage 1 Local Reject Mapping
The minimum Stage 1 coordinator-local reject classes should be:
1. `snapshot_mismatch`
2. `artifact_token_mismatch`
3. `profile_version_mismatch`
4. `capability_mismatch`
5. `publication_fence_mismatch`
6. `dynamic_dependency_failure`
7. `synthetic_cycle_reject`
8. `host_injected_failure`

### Reject Mapping Intent
1. the first five classes protect publication fences and compatibility discipline,
2. `dynamic_dependency_failure` captures explicit runtime-dependency failure rather than generic failure,
3. `synthetic_cycle_reject` supports self-contained cycle-region and iteration-profile tests,
4. `host_injected_failure` keeps the harness able to force deterministic rejection scenarios.

## Stage 1 Coordinator Transition Packet
The first implementation-facing Stage 1 coordinator transition packet should contain at least:

| Transition | Input / precondition | Required state effect | Minimum trace pressure |
|---|---|---|---|
| `C1 AdmitCandidateWork` | local candidate intake packet received for current structural basis | create or replace in-flight coordinator record keyed by `candidate_result_id` and `struct_snapshot_id` | `candidate_admitted` |
| `C2 RecordAcceptedCandidateResult` | intake packet passes shape validation and is still pending decision | register `AcceptedCandidateResult` as non-published candidate work | `candidate_recorded` |
| `C3 RejectCandidateWork` | any typed reject class or fence mismatch is detected before publish | append reject detail, clear publication eligibility, preserve no stable publication side effects | `candidate_rejected` |
| `C4 AcceptAndPublish` | compatibility basis still holds and publish bundle derivation succeeds | emit one atomic publication bundle, advance published view, clear in-flight candidate | `publication_committed` |
| `C5 PinReader` | reader requests stable view protection | add reader pin against the current published epoch or publication boundary | `reader_pinned` |
| `C6 UnpinAndReleaseProtection` | reader releases protection | remove pin and re-open eviction eligibility where safe | `reader_unpinned` |

### Transition Notes
1. `C1` and `C2` preserve the candidate-result versus publication split required by the OxFml seam.
2. `C3` is architecturally no-publish and must never emit observer-visible publication deltas.
3. `C4` is the only Stage 1 transition allowed to create stable publication.
4. `C5` and `C6` keep pinned-reader safety in the same transition packet rather than leaving it implicit.
5. These names are intentionally aligned to W008 actions `A4` through `A9` so later TLA+ authoring does not have to reinterpret the packet.

## Stage 1 Coordinator State Variables
The minimum coordinator-local state variables to carry through W003 should be:
1. `published_view_ref`
2. `in_flight_candidate_ref`
3. `accepted_candidate_ref`
4. `pinned_reader_set`
5. `reject_log`
6. `publication_counter_state`

These are the minimum implementation-facing handles needed to make the Stage 1 transition packet testable and replay-visible.

## Pre-Closure Verification Checklist
| # | Check | Yes/No |
|---|-------|--------|
| 1 | Spec text and realization notes updated for all in-scope items? | yes |
| 2 | Pack expectations updated for affected packs? | no |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | no |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes |
| 6 | All required tests pass? | yes |
| 7 | No known semantic gaps remain in declared scope? | no |
| 8 | Completion language audit passed (no premature "done"/"complete" per AGENTS.md Section 3)? | yes |
| 9 | IN_PROGRESS_FEATURE_WORKLIST.md updated? | yes |
| 10 | CURRENT_BLOCKERS.md updated (new/resolved)? | no |

## Status
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - the Stage 1 coordinator now has executable admission, accept, reject, publish, pin, and unpin logic with passing tests, but no replay or conformance artifacts exist yet
  - publication bundle and reject-detail surfaces are still local implementation shapes rather than emitted artifact families
  - the single-publisher floor is realized only for the sequential Stage 1 path; no concurrency-facing guarantees are exercised yet
  - W008 and W009 still need to bind the implemented coordinator fields and transitions into authored TLA+ and replay artifacts
- claim_confidence: provisional
- reviewed_inbound_observations: `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md` missing
