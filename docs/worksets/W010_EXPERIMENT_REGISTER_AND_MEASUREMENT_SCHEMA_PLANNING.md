# W010: Experiment Register and Measurement-Schema Planning

## Purpose
Convert the roadmap's decisive experiments and early-counter requirements into explicit planning packets for later implementation and assurance work.

## Position and Dependencies
- **Depends on**: W006
- **Blocks**: none
- **Cross-repo**: none required initially

## Scope
### In scope
1. Early-cutoff experiment planning.
2. Dynamic-topo versus rebuild experiment planning.
3. Dynamic-dependency and overlay economics measurement planning.
4. Counter-schema planning for Stage 1 and Stage 2 promotion evidence.

### Out of scope
1. Running the experiments.
2. Final promotion-threshold decisions.
3. Production instrumentation implementation.

## Deliverables
1. An experiment-register and measurement-schema planning packet aligned to the realization roadmap.

## Gate Model
### Entry gate
- W006 has bound the roadmap to the assurance-planning sequence.

### Exit gate
- The decisive experiments and counter schemas are explicit enough to inform later implementation and assurance work.

## Pre-Closure Verification Checklist
| # | Check | Yes/No |
|---|-------|--------|
| 1 | Spec text and realization notes updated for all in-scope items? | no |
| 2 | Pack expectations updated for affected packs? | no |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | no |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | no |
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
  - experiment register not yet authored
  - counter schema not yet authored
  - promotion-threshold evidence rules not yet tightened
- claim_confidence: draft
- reviewed_inbound_observations: ../OxFml/docs/upstream/NOTES_FOR_OXCALC.md missing

