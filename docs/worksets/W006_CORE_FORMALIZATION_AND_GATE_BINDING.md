# W006: Core Formalization and Gate Binding

## Purpose
Bind the rewritten OxCalc core-engine architecture to initial Lean, TLA+, replay, and pack artifacts and convert the roadmap into executable gate-bearing work packets.

## Position and Dependencies
- **Depends on**: W001, W002, W003, W004, W005
- **Blocks**: W007, W008, W009, W010
- **Cross-repo**: depends on the accepted shared seam direction from OxFml and may require narrower follow-on handoffs later

## Scope
### In scope
1. Initial Lean-facing model object and transition boundary plan.
2. Initial TLA+ coordinator and publication model plan.
3. Replay and pack binding from roadmap to executable assurance worksets.
4. Workset decomposition for the next assurance-planning sequence.

### Out of scope
1. Claiming full formal closure.
2. Claiming all packs are green.
3. Claiming replay evidence exists before the first assurance artifacts are authored.

## Deliverables
1. Initial formalization and gate-binding package aligned to `CORE_ENGINE_FORMALIZATION_AND_ASSURANCE.md` and `CORE_ENGINE_REALIZATION_ROADMAP.md`.
2. Successor worksets `W007` through `W010` with explicit dependencies and gates.

## Gate Model
### Entry gate
- Core canonical docs and accepted seam direction are in place.

### Exit gate
- Initial formalization and gate-binding package is explicit enough to execute as the next assurance lane.
- Successor worksets exist for Lean, TLA+, replay and pack binding, and experiment planning.

## Pre-Closure Verification Checklist
| # | Check | Yes/No |
|---|-------|--------|
| 1 | Spec text and realization notes updated for all in-scope items? | yes |
| 2 | Pack expectations updated for affected packs? | no |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | no |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes |
| 6 | All required tests pass? | no |
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
  - formal artifact creation has not started yet
  - replay artifacts and exercised pack evidence are still absent
  - narrower follow-on seam pressure may emerge from W007 through W009
- claim_confidence: provisional
- reviewed_inbound_observations: `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md` missing

