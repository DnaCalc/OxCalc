# W008: TLA+ Coordinator, Publication, and Fence-Safety Model Plan

## Purpose
Define the first TLA+-oriented model scope for coordinator authority, publication safety, reject behavior, and fence compatibility.

## Position and Dependencies
- **Depends on**: W006, W007
- **Blocks**: W009
- **Cross-repo**: relies on accepted OxFml seam direction for candidate-result versus publication boundaries

## Scope
### In scope
1. State-machine boundary for coordinator, in-flight work, accepted candidate result, and published state.
2. Safety-property backlog for reject-is-no-publish, no torn publication, and fence compatibility.
3. Initial liveness or progress question list for later staged concurrency work.

### Out of scope
1. Full TLA+ model implementation.
2. Stage 2 contention policy closure.
3. Complete fairness or starvation analysis.

## Deliverables
1. A TLA+-oriented planning packet for coordinator and fence-safety artifact authoring.

## Gate Model
### Entry gate
- W007 has stabilized the initial state vocabulary and transition boundaries.

### Exit gate
- The coordinator model boundary and safety backlog are explicit enough to author the first TLA+ artifact.

## Pre-Closure Verification Checklist
| # | Check | Yes/No |
|---|-------|--------|
| 1 | Spec text and realization notes updated for all in-scope items? | no |
| 2 | Pack expectations updated for affected packs? | no |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | no |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | no |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | no |
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
  - state-machine scope not yet authored
  - safety and liveness backlog not yet enumerated in detail
  - replay linkage for candidate-result and fence rejects still absent
- claim_confidence: draft
- reviewed_inbound_observations: ../OxFml/docs/upstream/NOTES_FOR_OXCALC.md missing

