# W003: Stage 1 Coordinator and Publication Baseline

## Purpose
Realize the sequential single-publisher coordinator and atomic publication baseline for TreeCalc Stage 1.

## Position and Dependencies
- **Depends on**: W001, W002
- **Blocks**: W004, W005, W006
- **Cross-repo**: OxFml seam hardening likely

## Scope
### In scope
1. Coordinator publication authority.
2. Accept/reject fence discipline.
3. Atomic accepted publication and reject-is-no-publish baseline.

### Out of scope
1. Stage 2 concurrency throughput realization.
2. Full seam closure without OxFml acknowledgment.

## Deliverables
1. Implementation-facing coordinator work packet aligned to `CORE_ENGINE_COORDINATOR_AND_PUBLICATION.md`.

## Gate Model
### Entry gate
- W001 integrated and W002 sufficiently resolved for coordinator realization.

### Exit gate
- Stage 1 coordinator baseline is explicit enough to implement, replay, and gate.

## Pre-Closure Verification Checklist
| # | Check | Yes/No |
|---|-------|--------|
| 1 | Spec text and realization notes updated for all in-scope items? | no |
| 2 | Pack expectations updated for affected packs? | no |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | no |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes |
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
- open_lanes: seam handoff, replay evidence, formal/tla closure, implementation realization
- claim_confidence: draft
