# W004: Incremental Recalc and Overlay Baseline

## Purpose
Realize the TreeCalc baseline for invalidation-state, verification-oriented incremental recalc direction, and explicit runtime overlay handling.

## Position and Dependencies
- **Depends on**: W001, W002, W003
- **Blocks**: W006
- **Cross-repo**: possible OxFml seam consequences for runtime-derived reporting

## Scope
### In scope
1. Invalidation-state model.
2. Stage 1 conservative subset of the verification-oriented incremental architecture.
3. Dynamic-dependency overlay baseline and explicit fallback policy.
4. Overlay economics and fallback instrumentation requirements.

### Out of scope
1. Default dynamic-topological maintenance.
2. Full SAC-style repair.

## Deliverables
1. Implementation-facing work packet aligned to `CORE_ENGINE_RECALC_AND_INCREMENTAL_MODEL.md` and `CORE_ENGINE_OVERLAY_AND_DERIVED_RUNTIME.md`.

## Gate Model
### Entry gate
- W001 integrated; W002/W003 sufficient for baseline recalc realization.

### Exit gate
- Stage 1 recalc/overlay subset is explicit enough to implement and evaluate with decisive experiments.

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
- open_lanes: dynamic-dependency/economics evidence, replay artifacts, implementation realization, handoff if runtime-derived seam deltas are needed
- claim_confidence: draft
