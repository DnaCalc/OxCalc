# W033 Lean Module Family First Slice

Status: `calc-uri.11_evidence_packet`
Workset: `W033`
Parent epic: `calc-uri`
Bead: `calc-uri.11`
Created: 2026-05-04

## 1. Purpose

This packet records the first W033 Lean widening slice.

The slice adds a checked Lean artifact that introduces abstract IDs, OxFml seam shapes, LET/LAMBDA carrier facts, coordinator state, overlays, conservative invalidation vocabulary, and first transition invariants.

## 2. Lean Artifact

| Artifact | Role | Check command | Result |
|---|---|---|---|
| `formal/lean/OxCalc/CoreEngine/W033FirstSlice.lean` | W033 first-slice formal vocabulary and small transition invariants | `lean formal\lean\OxCalc\CoreEngine\W033FirstSlice.lean` | passed |

The existing `formal/lean/OxCalc/CoreEngine/Stage1State.lean` remains the Stage 1 floor. `W033FirstSlice.lean` is additive and does not replace it.

## 3. Modeled Surface

| Lean surface | W033 claim linkage |
|---|---|
| abstract IDs for nodes, snapshots, epochs, candidates, commits, rejects, publications, traces, replay, and invocation contracts | authority matrix identity rows |
| `CandidateFact`, `CommitBundleFact`, `RejectFact`, `FenceFact` | OxFml/FEC/F3E seam-shape imports |
| `RuntimeEffectFact` | runtime-effect visibility and no-silent-loss rows |
| `LetLambdaCarrierFact` | narrow LET/LAMBDA carrier fragment |
| `OverlayFact` | protected overlay and eviction safety rows |
| `CoordinatorState` | candidate, reject, publication, commit-history, runtime-effect, and overlay state |
| `ConservativeAffectedSet` | over-invalidation allowed, under-invalidation not allowed |
| `ProtectedOverlaySafe` | protected overlays cannot be eviction-eligible |

## 4. Checked First Theorems

| Theorem | Meaning | Evidence status |
|---|---|---|
| `applyReject_noPublish` | The abstract reject transition preserves published view and commit history. | checked |
| `applyPublish_atomic` | The abstract publish transition updates public view and commit history together, with the published view sourced from the commit candidate. | checked |
| `conservativeAffectedSet_refl` | An exact affected set is conservative relative to itself. | checked |
| `emptyOverlays_safe` | Empty overlay state is trivially protected-overlay safe. | checked |

## 5. Proof Backlog

The checked slice is intentionally small. The first proof backlog is:

1. candidate result is not publication over richer coordinator histories,
2. stale or incompatible fence cannot publish,
3. published view atomicity over target-set updates rather than only commit metadata,
4. protected overlay retention under pin/unpin and eviction transitions,
5. invalidation closure contains every static and runtime dependency target,
6. dynamic dependency carrier facts are preserved into invalidation/replay surfaces,
7. LET/LAMBDA carrier facts preserve lexical capture and invocation-contract visibility at the abstract seam level,
8. replay-equivalent sequential histories preserve the observable surface,
9. production/core-engine observation refines TraceCalc observation for the first covered slice.

## 6. Rollout Direction

Future Lean module-family split, if W033 continues widening beyond this first slice:

1. `CoreIds`
2. `CoreSnapshots`
3. `OxfmlSeam`
4. `OxfmlOxfuncBoundary`
5. `Coordinator`
6. `Dependencies`
7. `Overlays`
8. `TraceCalcRefinement`
9. `Theorems`

The current checked artifact keeps these in one file to avoid introducing unproven import/build complexity before the first W033 slice typechecks.

## 6.1 Post-W033 Successor Slice

The successor packet `W033_FORMAL_MODEL_FAMILY_WIDENING.md` adds `formal/lean/OxCalc/CoreEngine/W033PostSlice.lean`.

That artifact widens the checked Lean surface for FEC bridge and publication fence compatibility, reject/no-publish, dependency closure, protected overlay retention, `LET`/`LAMBDA` carrier visibility, replay-equivalent histories, and explicit no-promotion of Stage 2 contention.

It remains additive to this first slice. It does not promote pack-grade replay, full OxFunc semantics, or Stage 2 concurrency policy.

## 7. Status

- execution_state: `lean_first_slice_checked`
- scope_completeness: `scope_partial`
- target_completeness: `target_partial`
- integration_completeness: `partial`
- open_lanes:
  - Lean module-family split remains future work
  - the post-W033 successor slice checks additional theorem families, but broader proof depth remains open
  - no OxFml formal artifact is modified by this packet
  - pack/capability binding has not yet consumed this Lean evidence
