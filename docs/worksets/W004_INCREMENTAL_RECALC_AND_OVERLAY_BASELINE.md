# W004: Incremental Recalc and Overlay Baseline

## Purpose
Define the implementation-facing Stage 1 packet for invalidation state, conservative verification-oriented incremental recalc, explicit overlay lifecycle, and fallback or economics instrumentation.

## Position and Dependencies
- **Depends on**: W001, W002, W003, W005
- **Blocks**: W006
- **Cross-repo**: may require a narrower OxFml follow-on handoff if runtime-derived effect requirements outgrow the current shared taxonomy

## Scope
### In scope
1. Invalidation-state machine for TreeCalc Stage 1.
2. Conservative subset of the verification-oriented incremental architecture.
3. Dynamic-dependency overlay baseline and explicit fallback policy.
4. Overlay retention, reuse, eviction, and measurement requirements.

### Out of scope
1. Default dynamic-topological maintenance.
2. Full SAC-style repair.
3. Stage 2 concurrency policy.

## Deliverables
1. Invalidation-state packet covering dirty, necessary, verified, and fallback-relevant transitions.
2. Overlay packet covering key shape, retention rules, eviction triggers, and runtime-derived effect handling assumptions.
3. Measurement packet covering fallback rates, overlay reuse or miss rates, and the Stage 1 experiment hooks needed by later work.

## Gate Model
### Entry gate
- W001 is integrated.
- W002 and W003 are sufficiently resolved for snapshot, publication, and reader-view assumptions.
- W005 accepted seam direction is reviewed for runtime-derived effect boundaries.

### Exit gate
- Stage 1 recalc and overlay subset is explicit enough to implement without re-opening baseline architecture choices.
- Overlay lifecycle and fallback policy are explicit enough to bind into W009 replay planning and W010 measurement planning.
- Runtime-derived effect assumptions are explicit enough to decide whether a narrower follow-on seam handoff is needed.

## Pre-Closure Verification Checklist
| # | Check | Yes/No |
|---|-------|--------|
| 1 | Spec text and realization notes updated for all in-scope items? | no |
| 2 | Pack expectations updated for affected packs? | no |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | no |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes |
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
  - invalidation-state packet is not yet authored in implementation-facing detail
  - overlay key, retention, and fallback matrix is not yet drafted
  - replay and pack classes for dynamic-dependency behavior are still absent
  - no exercised recalc or overlay implementation exists
- claim_confidence: provisional
- reviewed_inbound_observations: `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md` missing
