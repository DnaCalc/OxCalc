# W015: Replay Appliance Adapter and Bundle Validator Rollout

## Purpose
Incorporate the Foundation Replay appliance handoff into the OxCalc canonical spec set without weakening OxCalc ownership of coordinator semantics, `TraceCalc`, reference-machine behavior, or Stage 1 replay-class meaning.

This workset establishes the OxCalc-local adapter contract, capability manifest, normalized event-family mapping, and bundle-validator expectations for Replay appliance rollout.

## Position and Dependencies
- **Depends on**: W009, W011, W012, W014
- **Blocks**: W016
- **Cross-repo**: aligned to the Foundation replay handoff run `20260315-215019-replay-appliance-authoritative-pass-01`; any narrower shared seam pressure discovered here must route through W005

## Scope
### In scope
1. OxCalc-local Replay appliance adapter spec incorporation.
2. Machine-readable capability manifest for the OxCalc adapter.
3. Explicit event-family normalization for current OxCalc label drift.
4. Bundle-validator and normalized bundle emission expectations.
5. Registry-version pinning and lifecycle/quarantine rollout expectations.
6. Capability evidence ladder and honest capability-claim floor.

### Out of scope
1. Realizing the replay-appliance bundle emitter in code.
2. Realizing explain or witness-distillation execution.
3. Claiming `cap.C5.pack_valid`.
4. Replacing `TraceCalc` authoring with a new DSL.

## Deliverables
1. Canonical adapter companion at `docs/spec/core-engine/CORE_ENGINE_REPLAY_APPLIANCE_ADAPTER.md`.
2. Machine-readable capability manifest at `docs/spec/core-engine/CORE_ENGINE_REPLAY_ADAPTER_CAPABILITY_MANIFEST_V1.json`.
3. Updated `TraceCalc`, validator-runner, oracle, and assurance docs with replay-appliance incorporation points.
4. Explicit local conflict notes where Foundation wording had to be adapted to preserve OxCalc semantics.
5. Bundle-validator expectations and normalized bundle root declaration for future implementation.

## Gate Model
### Entry gate
- W009, W011, and W012 remain the active OxCalc-owned replay, harness, and oracle authorities.
- W014 exists as the next execution wave and can carry the resulting rollout forward.
- The Foundation replay handoff pack has been read and reconciled against OxCalc-local semantics.

### Exit gate
- The OxCalc adapter spec is explicit enough to implement without reopening authority or mapping questions.
- The capability manifest exists and does not overclaim capability.
- Event-family normalization and label-drift resolution are explicit.
- Bundle-validator and normalized-bundle expectations are explicit enough to guide implementation.
- The highest locally proven capability level and the unproven rollout path are both explicit.

## Core Rollout Rules
1. OxCalc semantics remain authoritative for `TraceCalc`, `engine_diff`, and reference-machine meaning.
2. The adapter is a projection layer over local artifacts, not a replacement semantics layer.
3. Label drift must be normalized explicitly, never silently erased.
4. Capability claims stop at the highest level proven by current local evidence.
5. Pack-grade rollout remains blocked until pack-grade evidence exists.

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
  - normalized replay-appliance bundle emission is still specification-only
  - capability claims above `cap.C1.replay_valid` remain unproven locally
  - bundle-validator conformance artifacts do not yet exist
- claim_confidence: draft
- reviewed_inbound_observations: `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md` missing
