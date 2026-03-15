# W016: Witness Distillation and Retained Failure Packs

## Purpose
Plan the OxCalc-local rollout of witness distillation, retained reduced witnesses, lifecycle/quarantine governance, and later pack-facing adoption over `TraceCalc`, `engine_diff`, and view-mismatch surfaces.

This workset turns the Foundation witness-distillation handoff into an OxCalc-owned rollout plan without weakening local scenario, oracle, or diff meaning.

## Position and Dependencies
- **Depends on**: W015
- **Blocks**: none
- **Cross-repo**: aligned to Foundation witness lifecycle and registry policy; any shared seam pressure discovered during reduced-witness rollout must route through W005

## Scope
### In scope
1. Reduction-unit planning for `TraceCalc` scenarios, phase blocks, event groups, reject records, and view slices.
2. Preservation-predicate planning for `engine_diff`, non-publication, reject-family, and view-mismatch cases.
3. Reduced-witness lifecycle, quarantine, retained-failure, and supersession policy as applied to OxCalc.
4. Artifact and bundle layout expectations for reduced witnesses and retained failures.
5. Pack-binding rules for reduced witnesses, including explicit non-pack-eligibility for explanatory-only or quarantined outputs.

### Out of scope
1. Claiming `cap.C4.distill_valid` before conformance artifacts exist.
2. Claiming `cap.C5.pack_valid`.
3. Generic source rewrites that are not lane-declared and replay-safe.
4. Replacing current local runner/oracle semantics with Replay appliance policy.

## Deliverables
1. OxCalc-local witness-distillation rollout plan tied to `TraceCalc` and `engine_diff`.
2. Explicit reduction-unit and closure-rule declarations for future implementation.
3. Lifecycle and quarantine rules for reduced witnesses and retained failures.
4. Retained-failure and witness-pack binding expectations for later pack adoption.

## Gate Model
### Entry gate
- W015 has established the adapter, manifest, normalized event-family mapping, and registry pinning rules.
- The OxCalc-local acceptance oracle remains the `TraceCalc Reference Machine` plus local diff surfaces.

### Exit gate
- The first OxCalc reduction-unit and closure-rule set is explicit enough to implement without reopening source-semantic authority.
- Lifecycle and quarantine rules are explicit enough to prevent overclaiming pack-grade status.
- Reduced-witness and retained-failure artifact expectations are explicit enough to guide implementation and later conformance evidence.
- Explanatory-only and quarantined outputs are explicitly non-pack-eligible.

## OxCalc-Specific Distillation Rules
1. Reduction starts from scenario, phase-block, and event-group structure before any finer pruning.
2. Retaining a publication mismatch retains candidate lineage and required view slices.
3. Retaining a reject retains reject context and the triggering candidate boundary.
4. Reduced witnesses must remain replay-valid or be marked explanatory-only.
5. Pack promotion is blocked while any witness remains explanatory-only or quarantined.

## Pre-Closure Verification Checklist
| # | Check | Yes/No |
|---|-------|--------|
| 1 | Spec text and realization notes updated for all in-scope items? | no |
| 2 | Pack expectations updated for affected packs? | no |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes |
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
  - witness distillation remains planning-only
  - no reduced-witness artifacts or lifecycle records exist yet
  - retained-failure pack bindings remain specification-only
- claim_confidence: draft
- reviewed_inbound_observations: `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md` missing
