# W006: Core Formalization and Gate Binding

## Purpose
Bind the rewritten OxCalc core-engine architecture to initial Lean/TLA+/replay/pack artifacts and convert the roadmap into executable gate-bearing work packets.

## Position and Dependencies
- **Depends on**: W001, W002, W003, W004, W005
- **Blocks**: none
- **Cross-repo**: may depend on OxFml seam hardening results

## Scope
### In scope
1. Initial Lean-facing model object set.
2. Initial TLA+ coordinator/publication model.
3. Gate binding from roadmap to worksets/packs.

### Out of scope
1. Claiming full formal closure.
2. Claiming all packs are green.

## Deliverables
1. Initial formalization/gate work packet aligned to `CORE_ENGINE_FORMALIZATION_AND_ASSURANCE.md` and `CORE_ENGINE_REALIZATION_ROADMAP.md`.

## Gate Model
### Entry gate
- Core canonical docs and seam direction are in place.

### Exit gate
- Initial formalization and gate-binding package is explicit enough to execute as the next assurance lane.

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
- open_lanes: formal artifact creation, pack binding, replay artifacts, OxFml dependency closure where needed
- claim_confidence: draft
