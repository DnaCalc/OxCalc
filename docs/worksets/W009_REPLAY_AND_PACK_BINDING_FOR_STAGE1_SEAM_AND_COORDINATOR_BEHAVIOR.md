# W009: Replay and Pack Binding for Stage 1 Seam and Coordinator Behavior

## Purpose
Bind the accepted seam direction and Stage 1 coordinator architecture to explicit replay artifact classes and first-pack expectations.

## Position and Dependencies
- **Depends on**: W006, W007, W008
- **Blocks**: none
- **Cross-repo**: must remain aligned with OxFml replay and seam vocabulary

## Scope
### In scope
1. Candidate-result versus publication replay cases.
2. Typed reject replay cases for fence and capability mismatch.
3. First-pack binding for Stage 1 seam and coordinator obligations.
4. Trace-schema pressure that may require narrower follow-on handoff.

### Out of scope
1. Implementing the replay harness.
2. Running the packs.
3. Full Stage 2 concurrency replay coverage.

## Deliverables
1. A replay and pack-binding planning packet for first assurance artifact creation.

## Gate Model
### Entry gate
- W007 and W008 have established the formal model boundary and coordinator safety scope.

### Exit gate
- Stage 1 replay artifact classes and first-pack bindings are explicit enough to drive artifact authoring.

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
  - replay corpus classes not yet authored
  - first-pack binding matrix not yet authored
  - narrower follow-on seam pressure not yet assessed from concrete replay cases
- claim_confidence: draft
- reviewed_inbound_observations: ../OxFml/docs/upstream/NOTES_FOR_OXCALC.md missing

