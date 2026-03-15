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
5. Stage 1 runtime-derived effect subset consumed from accepted candidate results.

### Out of scope
1. Default dynamic-topological maintenance.
2. Full SAC-style repair.
3. Stage 2 concurrency policy.

## Deliverables
1. Invalidation-state packet covering dirty, necessary, verified, and fallback-relevant transitions.
2. Overlay packet covering key shape, retention rules, eviction triggers, and runtime-derived effect handling assumptions.
3. Measurement packet covering fallback rates, overlay reuse or miss rates, and the Stage 1 experiment hooks needed by later work.
4. Stage 1 runtime-derived effect subset and dependency-shape handling packet for W009 replay and W010 measurement consumption.

## Gate Model
### Entry gate
- W001 is integrated.
- W002 and W003 are sufficiently resolved for snapshot, publication, and reader-view assumptions.
- W005 accepted seam direction is reviewed for runtime-derived effect boundaries.

### Exit gate
- Stage 1 recalc and overlay subset is explicit enough to implement without re-opening baseline architecture choices.
- Overlay lifecycle and fallback policy are explicit enough to bind into W009 replay planning and W010 measurement planning.
- Runtime-derived effect assumptions are explicit enough to decide whether a narrower follow-on seam handoff is needed.
- The minimum Stage 1 effect subset and dependency-shape update handling are explicit in OxCalc-local terms.

## Stage 1 Runtime-Derived Effect Subset
The Stage 1 OxCalc-local runtime-derived effect subset should include at least:
1. `dynamic_ref_activated`
2. `dynamic_ref_released`
3. `region_shape_activated`
4. `region_shape_released`
5. `capability_observed`
6. `format_observed`

This is the minimum local subset the overlay and fallback baseline should consume.
It is not a claim that the shared OxFml-side taxonomy is closed.

## Stage 1 Dependency-Shape Update Handling
The Stage 1 dependency-shape update kinds the recalc and overlay baseline should consume are:
1. `none`
2. `activate_dynamic_dep`
3. `release_dynamic_dep`
4. `change_region_membership`
5. `synthetic_spill_shape`

### Handling Direction
1. `activate_dynamic_dep` and `release_dynamic_dep` feed dynamic-overlay updates and fallback decisions.
2. `change_region_membership` and `synthetic_spill_shape` feed region- or spill-like overlay updates where a scenario or implementation path models them.
3. `none` keeps candidate-result publication explicit even when dependency shape is unchanged.

## Stage 1 Overlay Key and Fallback Direction
The minimum Stage 1 overlay key should be based on:
1. `owner_node_id`
2. `overlay_kind`
3. `struct_snapshot_id`
4. `compatibility_basis`
5. optional effect-specific payload identity

The minimum fallback triggers should include at least:
1. missing required dynamic-dependency effect detail,
2. incompatible overlay key basis,
3. unsupported region-shape or spill-shape consequence,
4. rejected candidate result,
5. explicit host or harness fallback injection.

## Stage 1 Invalidation Transition Packet
The first implementation-facing Stage 1 invalidation transition packet should include at least:

| Transition | Source -> target | Trigger | Minimum required consequence |
|---|---|---|---|
| `I1 MarkDirty` | `clean | verified_clean -> dirty_pending` | structural edit, upstream publication delta, or explicit external invalidation | record stale frontier without forcing immediate evaluation |
| `I2 MarkNeeded` | `dirty_pending -> needed` | demanded frontier or stabilization target requires the node | put node into deterministic work-discovery order |
| `I3 BeginEvaluate` | `needed -> evaluating` | deterministic scheduler selects the node on a compatible basis | begin evaluator work and protect competing implicit state changes |
| `I4 VerifyClean` | `evaluating -> verified_clean` | early-cutoff or conservative verification proves unchanged observable result | avoid publication while still resolving demanded work |
| `I5 ProduceDependencyShapeUpdate` | `evaluating -> publish_ready` | evaluation yields values and dependency-shape/runtime-effect outputs | make candidate-result publication possible and update overlay candidates |
| `I6 RejectOrFallback` | `evaluating | publish_ready -> rejected_pending_repair` | typed reject, incompatible overlay key, or insufficient effect detail | clear publish eligibility and force explicit conservative re-entry |
| `I7 PublishAndClear` | `publish_ready -> clean` | coordinator accepts and publishes the candidate result | commit stable effects and clear stale state |
| `I8 ReleaseAndEvictEligible` | `clean | verified_clean -> clean` | demanded frontier is released and no pin protects prior overlay state | keep node clean while making overlays eligible for deterministic eviction |

## Stage 1 Overlay Retention Matrix
The first implementation-facing Stage 1 overlay retention matrix should be:

| Overlay class | Retain while | Evict when |
|---|---|---|
| `invalidation_execution_state` | node is not yet back to a stable `clean` or `verified_clean` state, or a pinned reader still protects the prior view | node is stable, pin protection is gone, and replay policy does not still reference the instance |
| `dynamic_dependency` | key basis remains compatible and no reject or fallback path has invalidated the observed dependency shape | superseded by accepted publication, invalidated by reject or fallback, or beyond the safe pinned-epoch boundary |
| `capability_fence_attachment` | associated capability basis and candidate/publication decision are still live | capability or publication fence mismatch occurs, or decision is resolved and no pin still depends on the attachment |
| `observer_priority_metadata` | demanded frontier or pinned-reader policy still needs the metadata | frontier is released, newer publication supersedes it, or policy marks it dispensable |

This matrix is the first Stage 1 floor for deterministic overlay reuse and eviction behavior.

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
  - the Stage 1 invalidation states, overlay key floor, fallback transition, and retention or eviction eligibility are now scaffolded in executable code with passing tests, but they remain a single-node baseline rather than a scheduler-rich engine slice
  - topo or SCC scheduling, multi-node propagation, and richer observer-demand integration are still absent from the implementation floor
  - W009 and W010 still need to bind the implemented transitions and overlay states into replay artifacts and counter families
  - runtime-derived effect handling is still narrower in code than in the full Stage 1 spec packet and may yet force a follow-on seam handoff
- claim_confidence: provisional
- reviewed_inbound_observations: `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md` missing
