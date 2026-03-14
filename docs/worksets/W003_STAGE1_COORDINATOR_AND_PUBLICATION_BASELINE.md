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
2. Accepted candidate-result intake boundary and commit-bundle publication boundary.
3. Typed reject consequence mapping for fence, token, capability, and other coordinator-relevant no-publish cases.
4. Stable observer-visible publication contract for Stage 1 sequential execution.

### Out of scope
1. Stage 2 concurrency throughput realization.
2. Full seam closure beyond the already acknowledged shared direction.
3. Complete TLA+ artifact authoring.

## Deliverables
1. Coordinator transition packet covering accept, reject, publish, and stable-view transitions.
2. Publication packet covering `AcceptedCandidateResult` intake, atomic commit-bundle publication, and reject-is-no-publish consequences.
3. Reject-detail packet covering the coordinator-local mapping that W008 and W009 must later consume.

## Gate Model
### Entry gate
- W001 is integrated.
- W002 is sufficiently resolved for coordinator-facing structural and snapshot assumptions.
- W005 accepted seam direction is available for candidate-result versus publication terminology.

### Exit gate
- Stage 1 coordinator transitions are explicit enough to implement without re-opening seam ownership questions.
- Publication and reject boundaries are explicit enough to drive replay planning and TLA+ safety planning.
- Stable observer-visible state rules are explicit enough to bind to Stage 1 trace and pack planning.

## Pre-Closure Verification Checklist
| # | Check | Yes/No |
|---|-------|--------|
| 1 | Spec text and realization notes updated for all in-scope items? | no |
| 2 | Pack expectations updated for affected packs? | no |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | no |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes |
| 6 | All required tests pass? | no |
| 7 | No known semantic gaps remain in declared scope? | no |
| 8 | Completion language audit passed (no premature "done"/"complete" per AGENTS.md Section 3)? | yes |
| 9 | IN_PROGRESS_FEATURE_WORKLIST.md updated? | no |
| 10 | CURRENT_BLOCKERS.md updated (new/resolved)? | no |

## Status
- execution_state: planned
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - coordinator state-machine packet is not yet authored in implementation-facing detail
  - reject-detail mapping is not yet bound to replay and pack classes
  - no TLA+ or replay artifacts exist for the Stage 1 publication contract
  - no exercised coordinator implementation exists
- claim_confidence: provisional
- reviewed_inbound_observations: `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md` missing
