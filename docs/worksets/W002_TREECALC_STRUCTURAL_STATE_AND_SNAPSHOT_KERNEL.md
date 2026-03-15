# W002: TreeCalc Structural State and Snapshot Kernel

## Purpose
Define the implementation-facing TreeCalc structural kernel packet: stable identity, immutable snapshot shape, projection boundary, and pinned-reader semantics.

## Position and Dependencies
- **Depends on**: W001
- **Blocks**: W003, W004, W006
- **Cross-repo**: none

## Scope
### In scope
1. Stable-ID policy for TreeCalc structural nodes and attached formula artifacts.
2. Structural snapshot record shape and immutable-successor boundary.
3. Projection and facade boundary for reader-facing traversal and address-like lookup.
4. Pinned-reader semantics in implementation-facing terms.

### Out of scope
1. Full coordinator implementation.
2. Grid-native substrate work.
3. Full formal proof artifacts.

## Deliverables
1. Structural identity packet covering node-id classes, parent or child attachment rules, and formula-artifact attachment boundaries.
2. Snapshot-kernel packet covering root shape, successor construction boundary, and immutable-truth versus derived-runtime split.
3. Reader-view packet covering pin, unpin, and stable-view obligations that later replay and TLA+ work can consume.

## Gate Model
### Entry gate
- W001 canonical rewrite integrated.

### Exit gate
- Stable-ID policy is explicit enough to implement without identity churn under TreeCalc edits.
- Snapshot-kernel shape is explicit enough to code immutable successor construction without re-opening the architecture docs.
- Reader pinning obligations are explicit enough to bind into W007, W008, and later replay artifacts.

## Pre-Closure Verification Checklist
| # | Check | Yes/No |
|---|-------|--------|
| 1 | Spec text and realization notes updated for all in-scope items? | yes |
| 2 | Pack expectations updated for affected packs? | no |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | no |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes |
| 6 | All required tests pass? | yes |
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
  - the structural kernel is now scaffolded into executable code with passing snapshot and pinning tests, but replay and assurance bindings do not exist yet
  - stable IDs, projection paths, and immutable successor construction are realized only for the current TreeCalc seed shape and builder floor
  - formula-artifact attachment is still a minimal identifier carrier rather than a richer OxFml-integrated structural contract
  - W003 and W004 now depend on concrete kernel code, but W007 and W008 have not yet consumed the implemented object names
- claim_confidence: provisional
- reviewed_inbound_observations: `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md` missing
