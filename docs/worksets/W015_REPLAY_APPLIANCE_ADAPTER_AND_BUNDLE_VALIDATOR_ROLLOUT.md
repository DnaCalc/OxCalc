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
6. A pre-rollout local coherence refactor that aligns `TraceCalc`, engine oracles, event-family handling, and diff surfaces to the Replay appliance shape without weakening OxCalc-owned semantics.

## Gate Model
### Entry gate
- W009, W011, and W012 remain the active OxCalc-owned replay, harness, and oracle authorities.
- W014 has reached its final gate and provides the widened Stage 1 baseline that W015 will project.
- The Foundation replay handoff pack has been read and reconciled against OxCalc-local semantics.

### Exit gate
- The local replay-facing surfaces are coherent enough that W015 does not need to spend its main implementation work compensating for avoidable local drift.
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

## Execution Packet Additions

### Environment Preconditions
- required tools:
  - `cargo`
  - `powershell`
- optional tools:
  - `lean`
  - `tlc`
- fallback rules:
  - if `lean` or `tlc` are unavailable, W015 may proceed on replay-adapter scope, but no claim may imply fresh formal-tool evidence beyond the last exercised baseline

### Evidence Layout
- canonical artifact root: `docs/test-runs/core-engine/tracecalc-reference-machine/`
- checked-in or ephemeral: checked-in baseline runs remain normative; ad hoc validation runs are ephemeral unless intentionally promoted
- baseline run naming:
  - carried active baseline entering W015: `w014-stage1-widening-baseline`
  - W015 may later promote one replay-appliance-aware baseline run explicitly rather than silently replacing `w014-stage1-widening-baseline`

### Replay-Corpus Readiness
- required replay classes with scenario ids:
  - `R1` -> `tc_accept_publish_001`
  - `R2` -> `tc_reject_no_publish_001`
  - `R3` -> `tc_multinode_dag_publish_001`, `tc_publication_fence_reject_001`
  - `R4` -> `tc_pinned_view_stability_001`
  - `R5` -> `tc_overlay_retention_001`
  - `R6` -> `tc_artifact_token_reject_001`, `tc_publication_fence_reject_001`
  - `R7` -> `tc_verify_clean_no_publish_001`
  - `R8` -> `tc_fallback_reentry_001`
- reserve or later replay classes:
  - retained-failure and reduced-witness lanes remain for `W016`
  - pack-grade replay-appliance validation remains later than the first W015 adapter rollout

### Coupled Widening Rule
- engine surfaces widened in this packet:
  - none as intended semantic expansion; only local replay-coherence refactor where needed
- oracle/conformance surfaces widened in the same slice:
  - local event-family normalization
  - local mismatch and severity projection scaffolding
  - replay-facing scenario metadata tightening
- widened comparison artifact:
  - `docs/test-runs/core-engine/tracecalc-reference-machine/w014-stage1-widening-baseline/`

## Execution Sequence

### Sequence 0. Local Replay-Coherence Refactor
Purpose:
1. align OxCalc-local `TraceCalc`, engine, oracle, and conformance surfaces to the replay rollout shape before adapter projection begins
2. reduce avoidable W015 complexity caused by local naming drift or underspecified diff surfaces

In scope:
1. normalize local event-family handling for candidate and publication drift labels
2. tighten typed mismatch and severity scaffolding for local `engine_diff`
3. tighten replay-facing corpus metadata so required scenarios carry `replay_projection`
4. add `witness_anchors` to scenarios that are expected to participate in retained-failure or witness-reduction lanes
5. reduce avoidable engine-oracle duplication where replay-facing helpers can be shared without changing semantics

Out of scope:
1. normalized replay-appliance bundle emission
2. capability-level promotion beyond currently proven local evidence
3. witness distillation execution

Entry gate:
1. `w014-stage1-widening-baseline` exists and remains the active normative baseline entering W015
2. replay-facing scenarios and pack-evidence expectations from W014 are available as the projection source

Exit gate:
1. local event-family normalization is embodied in OxCalc code or artifact models rather than only prose
2. local diff or severity scaffolding is explicit enough that W015 can project it without reopening OxCalc semantics
3. replay-facing scenarios carry the metadata required for W015, and retained-failure candidates carry `witness_anchors`
4. no semantic drift is introduced into the W014 covered corpus

### Sequence 1. Adapter and Manifest Incorporation
Purpose:
1. incorporate the adapter contract, capability manifest, and authority split into the canonical OxCalc docs

Exit gate:
1. the adapter and manifest are explicit and aligned to the post-Sequence-0 local surfaces

### Sequence 2. Normalized Event-Family and Bundle Projection Rules
Purpose:
1. lock the projection from OxCalc-local artifacts into normalized replay-appliance bundle surfaces

Exit gate:
1. label-drift resolution, preserved view surfaces, and bundle layout are explicit enough to implement directly

### Sequence 3. Bundle-Validator and Capability-Floor Rollout
Purpose:
1. define the first honest bundle-validator and capability-claim floor over emitted OxCalc evidence

Exit gate:
1. validator expectations and capability-floor claims are explicit enough to support later realization without overclaiming

## Pre-Closure Verification Checklist
| # | Check | Yes/No |
|---|-------|--------|
| 1 | Spec text and realization notes updated for all in-scope items? | yes |
| 2 | Pack expectations updated for affected packs? | yes |
| 3 | At least one deterministic replay artifact exists per in-scope behavior? | yes |
| 4 | Semantic-equivalence statement provided for policy/strategy changes? | yes |
| 5 | FEC/F3E cross-repo impact assessed and handoff filed if needed? | yes |
| 6 | All required tests pass? | yes |
| 7 | No known semantic gaps remain in declared scope? | yes |
| 8 | Completion language audit passed (no premature "done"/"complete" per AGENTS.md Section 3)? | yes |
| 9 | IN_PROGRESS_FEATURE_WORKLIST.md updated? | yes |
| 10 | CURRENT_BLOCKERS.md updated (new/resolved)? | yes |

## Status
- execution_state: complete
- scope_completeness: scope_complete
- target_completeness: target_complete
- integration_completeness: integrated
- open_lanes: none
- claim_confidence: validated
- reviewed_inbound_observations: `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md` missing
