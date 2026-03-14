# W007: Lean-Facing State Objects and Transition Boundary Plan

## Purpose
Define the first Lean-facing object inventory and transition boundaries for the TreeCalc core engine so later proofs and executable models can share the same state vocabulary.

## Position and Dependencies
- **Depends on**: W006
- **Blocks**: W008, W009
- **Cross-repo**: must respect accepted OxFml seam terminology and ownership boundaries

## Scope
### In scope
1. Structural snapshot, runtime state, and publication-state object inventory.
2. Initial transition boundary list for invalidation, evaluation result handling, publication, and rejection.
3. Mapping from OxCalc canonical docs to initial formal object names.

### Out of scope
1. Full Lean implementation.
2. Full theorem backlog closure.
3. TLA+ concurrency state machine details.

## Deliverables
1. A Lean-facing object and transition planning packet suitable for first formal artifact authoring.

## Gate Model
### Entry gate
- W006 has established the assurance-planning sequence.

### Exit gate
- The object inventory and transition boundaries are explicit enough to start formal artifact authoring without re-opening core terminology.

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
  - object inventory not yet authored
  - theorem and invariant backlog not yet enumerated
  - replay-grounded validation inputs not yet attached
- claim_confidence: draft
- reviewed_inbound_observations: ../OxFml/docs/upstream/NOTES_FOR_OXCALC.md missing

