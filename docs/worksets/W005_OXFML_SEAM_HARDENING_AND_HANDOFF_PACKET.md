# W005: OxFml Seam Hardening and Handoff Packet

## Purpose
Convert the OxCalc-local seam requirements into explicit shared-clause handoff material for OxFml and align canonical seam hardening with Stage 1 and pre-Stage 2 needs.

## Position and Dependencies
- **Depends on**: W001, W003, W004
- **Blocks**: W006
- **Cross-repo**: OxFml acknowledgment required for shared seam closure

## Scope
### In scope
1. Accepted-result payload handoff.
2. Reject-detail taxonomy and payload handoff.
3. Fence-consequence handoff.
4. Runtime-derived reporting handoff where needed.

### Out of scope
1. Claiming shared seam closure without OxFml acknowledgment.
2. Rewriting OxFml canonical seam docs directly in this repo.

## Deliverables
1. First OxFml handoff packet(s) and register entry.

## Gate Model
### Entry gate
- OxCalc-local seam requirements are explicit.

### Exit gate
- Handoff packet(s) filed and registered with clear shared-clause requests and evidence references.

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
- open_lanes: handoff drafting, evidence binding, OxFml acknowledgment
- claim_confidence: draft
