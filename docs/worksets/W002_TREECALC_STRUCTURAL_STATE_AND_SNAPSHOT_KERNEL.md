# W002: TreeCalc Structural State and Snapshot Kernel

## Purpose
Realize the immutable structural state kernel for the TreeCalc-first engine, including stable identity, snapshot/version discipline, and pinned stable reader views.

## Position and Dependencies
- **Depends on**: W001
- **Blocks**: W003, W004, W006
- **Cross-repo**: none

## Scope
### In scope
1. Stable-ID policy and snapshot kernel realization.
2. Immutable structure plus derived/facade discipline.
3. Pinned-reader safety semantics in implementation-facing form.

### Out of scope
1. Full coordinator implementation.
2. Grid-native substrate work.

## Deliverables
1. Implementation-facing state-kernel work packet aligned to `CORE_ENGINE_STATE_AND_SNAPSHOTS.md`.

## Gate Model
### Entry gate
- W001 canonical rewrite integrated.

### Exit gate
- Stable-ID and snapshot-kernel subset is explicit enough to implement and verify.

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
- open_lanes: stable-ID closure, implementation realization, replay evidence, assurance closure
- claim_confidence: draft
