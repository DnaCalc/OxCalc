# W001: Core-Engine Spec Rewrite and Canonicalization

## Purpose
Replace the OxCalc bootstrap core-engine spec set with a rewritten canonical TreeCalc-first architecture library and integrate that rewrite into the repo-level spec surface.

## Position and Dependencies
- **Depends on**: none
- **Blocks**: W002, W003, W004, W005, W006
- **Cross-repo**: OxFml handoff packet(s) may be required before seam-related closure

## Scope
### In scope
1. Rewrite the canonical OxCalc core-engine architecture set.
2. Archive the bootstrap set.
3. Integrate the new canonical set into the repo spec index and repo-level registers.
4. Identify first shared-seam handoff pressure to OxFml.

### Out of scope
1. Realized engine implementation.
2. Claiming seam closure without OxFml acknowledgment.
3. Declaring formalization or pack closure complete.

## Deliverables
1. Canonical rewritten core-engine doc set exists in `docs/spec/core-engine/`.
2. Bootstrap set preserved in an archive location.
3. Repo index and feature/workset tracking point at the rewritten set.
4. First seam handoff candidates are explicitly identified.

## Gate Model
### Entry gate
- Rewrite direction and TreeCalc-first architecture are accepted.

### Exit gate
- New canonical docs exist and are internally aligned.
- Bootstrap set is archived and in-place bootstrap docs are superseded or redirecting.
- Repo-level tracking is updated to the rewritten roadmap.
- Remaining seam handoff work is explicitly identified.

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
| 9 | IN_PROGRESS_FEATURE_WORKLIST.md updated? | yes |
| 10 | CURRENT_BLOCKERS.md updated (new/resolved)? | yes |

## Status
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - seam handoff packet not yet filed
  - supersession treatment for remaining bootstrap/reference files may need tightening
  - no replay/pack/assurance closure yet
- claim_confidence: provisional
