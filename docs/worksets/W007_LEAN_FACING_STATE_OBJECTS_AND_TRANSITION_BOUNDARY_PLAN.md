# W007: Lean-Facing State Objects and Transition Boundary Plan

## Purpose
Define the first Lean-facing object inventory and transition boundaries for the TreeCalc core engine so later proofs and executable models can share the same state vocabulary.

## Position and Dependencies
- **Depends on**: W006
- **Blocks**: W008, W009
- **Cross-repo**: must respect accepted OxFml seam terminology and ownership boundaries

## Scope
### In scope
1. Structural snapshot, runtime state, and publication-state object inventory.
2. Initial transition boundary list for invalidation, evaluation result handling, publication, and rejection.
3. Mapping from OxCalc canonical docs to initial formal object names.

### Out of scope
1. Full Lean implementation.
2. Full theorem backlog closure.
3. TLA+ concurrency state machine details.

## Deliverables
1. A Lean-facing object and transition planning packet suitable for first formal artifact authoring.

## Gate Model
### Entry gate
- W006 has established the assurance-planning sequence.

### Exit gate
- The object inventory and transition boundaries are explicit enough to start formal artifact authoring without re-opening core terminology.
- The state vocabulary is compatible with W008 TLA+ planning and W009 replay planning.

## Lean-Facing Object Inventory
The first Lean-facing object set should separate immutable truth, runtime-derived state, and publication state explicitly.

### A. Structural Truth Objects
1. `TreeNodeId`
   - stable identity for a TreeCalc structural entity.
2. `StructuralSnapshotId`
   - immutable identity for one structural snapshot.
3. `FormulaArtifactRef`
   - immutable OxFml artifact handle referenced from structural truth.
4. `NodePayloadRef`
   - immutable node-local semantic payload handle.
5. `TreeNodeRecord`
   - structural node record keyed by `TreeNodeId`, with parent or child containment and payload references.
6. `StructuralSnapshot`
   - immutable root object containing node map, root identity, projection metadata, and snapshot identity.
7. `ProjectionDescriptor`
   - host-facing projection metadata derived from stable identity, not engine truth.

### B. Runtime-Derived Objects
1. `EpochId`
   - epoch identifier for runtime attachment and publication fencing.
2. `NodeCalcState`
   - invalidation or progress state for a node, such as clean, stale, necessary, evaluating, ready, or blocked.
3. `OverlayKey`
   - runtime-derived key used for overlay reuse and eviction safety.
4. `OverlayEntry`
   - runtime-derived overlay record for dynamic dependencies or related effects.
5. `RuntimeViewState`
   - runtime-derived state attached to a structural snapshot and epoch basis.
6. `CandidateWorkId`
   - identity for a unit of in-flight evaluation work.
7. `AcceptedCandidateResultRef`
   - Lean-facing reference to the evaluator-produced `AcceptedCandidateResult` boundary accepted by the shared seam vocabulary.
8. `RejectDetail`
   - typed reject detail for no-publish outcomes.

### C. Publication and Reader Objects
1. `PublishedViewId`
   - identity for a stable published runtime view.
2. `PublishedNodeResult`
   - published node-facing result content after coordinator acceptance.
3. `CommitBundleRef`
   - reference to the published bundle produced from an accepted candidate result.
4. `PinnedReaderId`
   - identity for a pinned reader or observer.
5. `PinnedView`
   - tuple of structural snapshot, published view, and status visible to one pinned reader.
6. `CoordinatorState`
   - abstract coordinator-owned publication state relevant to sequential proof boundaries.

## State Separation Rules
The Lean-facing model should encode these separation rules directly:
1. `StructuralSnapshot` is immutable truth and cannot be mutated by runtime transitions.
2. `RuntimeViewState` is derived and must carry a snapshot and epoch attachment basis.
3. `AcceptedCandidateResultRef` is not publication.
4. `CommitBundleRef` is published state and exists only after coordinator acceptance.
5. `PinnedView` must always reference a snapshot-compatible published view.

## Initial Transition Boundary Set
The first transition boundary set should stay sequential and architecture-level.

### T1. Structural Successor Transition
Input:
1. base `StructuralSnapshot`
2. admissible structural operation

Output:
1. successor `StructuralSnapshot`
2. invalidation seed for affected runtime state

Invariant focus:
1. no hidden structural mutation
2. deterministic successor relation

### T2. Runtime Attachment Initialization
Input:
1. `StructuralSnapshot`
2. `EpochId`

Output:
1. empty or initialized `RuntimeViewState`
2. initial coordinator-facing attachment basis

Invariant focus:
1. runtime state is attached, not fused into structural truth

### T3. Invalidation Marking Transition
Input:
1. `RuntimeViewState`
2. affected structural or upstream change signal

Output:
1. updated `NodeCalcState` assignments
2. replay-visible invalidation deltas at the abstract level

Invariant focus:
1. invalidation state remains explicit and deterministic

### T4. Candidate Work Admission Transition
Input:
1. `CoordinatorState`
2. candidate node or work frontier
3. compatibility basis drawn from snapshot, token, and profile context

Output:
1. admitted `CandidateWorkId`
2. in-flight work state

Invariant focus:
1. coordinator owns admission and compatibility checking

### T5. Accepted Candidate Result Intake Transition
Input:
1. `CandidateWorkId`
2. evaluator-produced `AcceptedCandidateResultRef`

Output:
1. coordinator-local ready-for-decision state

Invariant focus:
1. accepted candidate result remains distinct from publication

### T6. Reject Transition
Input:
1. in-flight candidate work
2. incompatibility or other reject basis

Output:
1. `RejectDetail`
2. no-publish stable outcome

Invariant focus:
1. reject-is-no-publish
2. typed no-publish detail preserved for replay

### T7. Accept-And-Publish Transition
Input:
1. coordinator-ready accepted candidate result
2. compatible snapshot and fence basis

Output:
1. `CommitBundleRef`
2. updated `PublishedViewId`
3. updated stable observer-visible state

Invariant focus:
1. accepted publication is atomic
2. publication occurs only through coordinator acceptance

### T8. Pin-Reader Transition
Input:
1. `PinnedReaderId`
2. current stable published view

Output:
1. `PinnedView`

Invariant focus:
1. a pinned reader sees one coherent snapshot and publication bundle

### T9. Unpin-And-Evict-Eligibility Transition
Input:
1. release of pinned reader state
2. overlay or runtime retention checks

Output:
1. updated protection set
2. newly evictable runtime-derived entries where allowed

Invariant focus:
1. pinned-reader safety precedes eviction

## First Lean Naming Map
The first naming map from OxCalc prose to Lean-facing objects should be:
1. structural snapshot -> `StructuralSnapshot`
2. snapshot version identity -> `StructuralSnapshotId`
3. stable ID -> `TreeNodeId`
4. runtime-derived state -> `RuntimeViewState`
5. invalidation state -> `NodeCalcState`
6. overlay entry -> `OverlayEntry`
7. accepted candidate result -> `AcceptedCandidateResultRef`
8. reject detail -> `RejectDetail`
9. published bundle -> `CommitBundleRef`
10. pinned reader view -> `PinnedView`
11. coordinator publication state -> `CoordinatorState`

## Immediate Theorem-Oriented Targets Anchored To This Packet
This packet should feed the earliest theorem backlog for:
1. no hidden structural mutation under `T1`
2. deterministic structural successor relation under `T1`
3. runtime-versus-structural separation under `T2`
4. reject-is-no-publish under `T6`
5. accepted publication atomicity under `T7`
6. pinned-reader stability under `T8` and `T9`

## Hand-Off To Later Worksets
This packet should hand forward to:
1. W008: use `CoordinatorState`, `AcceptedCandidateResultRef`, `CommitBundleRef`, `PinnedView`, and transitions `T4` through `T9`
2. W009: use `T3`, `T6`, `T7`, `T8`, and `T9` as replay and pack-binding anchors
3. later implementation work: use `TreeNodeId`, `StructuralSnapshot`, `RuntimeViewState`, and `OverlayEntry` as the first implementation-facing vocabulary set

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
  - object inventory exists, but no Lean artifact has been authored yet
  - theorem backlog is only the first anchored slice, not a closed assurance map
  - replay-grounded validation inputs are still absent
  - W008 and W009 must still consume this packet
- claim_confidence: provisional
- reviewed_inbound_observations: `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md` missing
