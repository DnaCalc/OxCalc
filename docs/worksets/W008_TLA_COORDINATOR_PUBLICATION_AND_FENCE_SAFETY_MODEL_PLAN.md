# W008: TLA+ Coordinator, Publication, and Fence-Safety Model Plan

## Purpose
Define the first TLA+-oriented model scope for coordinator authority, publication safety, reject behavior, and fence compatibility.

## Position and Dependencies
- **Depends on**: W006, W007
- **Blocks**: W009
- **Cross-repo**: relies on accepted OxFml seam direction for candidate-result versus publication boundaries

## Scope
### In scope
1. State-machine boundary for coordinator, in-flight work, accepted candidate result, and published state.
2. Safety-property backlog for reject-is-no-publish, no torn publication, and fence compatibility.
3. Initial liveness or progress question list for later staged concurrency work.

### Out of scope
1. Full TLA+ model implementation.
2. Stage 2 contention policy closure.
3. Complete fairness or starvation analysis.

## Deliverables
1. A TLA+-oriented planning packet for coordinator and fence-safety artifact authoring.

## Gate Model
### Entry gate
- W007 has stabilized the initial state vocabulary and transition boundaries.

### Exit gate
- The coordinator model boundary and safety backlog are explicit enough to author the first TLA+ artifact.
- The model scope is aligned to the accepted `AcceptedCandidateResult` versus publication split and to W009 replay planning.

## TLA+ Model Boundary
The first TLA+ model should stay abstract and sequentially faithful, while still being structured so later concurrent interleavings can be introduced.

### Primary State Variables
1. `structSnapshot`
   - the currently relevant structural snapshot identity.
2. `runtimeView`
   - abstract runtime-derived state attached to `structSnapshot`.
3. `coordState`
   - coordinator-owned admission, decision, and publication control state.
4. `inFlight`
   - set or map of candidate work items currently admitted but not yet stabilized.
5. `acceptedCandidate`
   - candidate-result objects that exist after evaluator success but before publication.
6. `publishedView`
   - the current stable published runtime view.
7. `pinnedReaders`
   - active pinned reader views and their protected snapshot or publication basis.
8. `overlayState`
   - abstract overlay entries together with retention or protection metadata.
9. `rejectLog`
   - typed reject outcomes retained for replay-visible diagnostics.

### Derived or Ghost Variables
The initial model may also use ghost structure for proof or checking convenience:
1. `compatBasis`
   - abstract compatibility tuple covering snapshot, token, profile, and capability basis.
2. `publishHistory`
   - abstract history of committed publication events.
3. `decisionHistory`
   - abstract history of accept or reject decisions.

## Object Mapping From W007
The W007 vocabulary should map into the TLA+ state shape as follows:
1. `CoordinatorState` -> `coordState`
2. `AcceptedCandidateResultRef` -> members of `acceptedCandidate`
3. `CommitBundleRef` -> elements recorded in `publishHistory` and reflected in `publishedView`
4. `PinnedView` -> elements of `pinnedReaders`
5. `RejectDetail` -> elements of `rejectLog`
6. transitions `T4` through `T9` -> abstract TLA+ actions `A4` through `A9`

## Initial Abstract Actions
The first TLA+ action set should align to the publication-critical transitions from W007.

### A0. Init
Establish initial state for:
1. `structSnapshot`
2. empty `runtimeView`
3. empty `inFlight`
4. empty `acceptedCandidate`
5. initial stable `publishedView`
6. empty `rejectLog`

### A4. AdmitCandidateWork
Abstract counterpart to W007 `T4`.

Effects:
1. add a work item to `inFlight`
2. record the compatibility basis required for later accept or reject
3. leave `publishedView` unchanged

### A5. RecordAcceptedCandidateResult
Abstract counterpart to W007 `T5`.

Effects:
1. move an in-flight work item into a coordinator-ready accepted-candidate state
2. preserve the distinction between evaluator success and publication
3. leave `publishedView` unchanged

### A6. RejectCandidateWork
Abstract counterpart to W007 `T6`.

Effects:
1. remove or mark the affected in-flight or accepted candidate as rejected
2. append typed `RejectDetail` to `rejectLog`
3. preserve `publishedView` unchanged

### A7. AcceptAndPublish
Abstract counterpart to W007 `T7`.

Effects:
1. consume a compatible accepted candidate
2. append a committed bundle to `publishHistory`
3. update `publishedView` atomically
4. update coordinator decision state accordingly

### A8. PinReader
Abstract counterpart to W007 `T8`.

Effects:
1. add a pinned reader entry tied to the current stable `publishedView` and `structSnapshot`
2. leave publication state unchanged

### A9. UnpinAndReleaseProtection
Abstract counterpart to W007 `T9`.

Effects:
1. remove or relax pinned-reader protection
2. update overlay or retention eligibility metadata
3. leave accepted publication state unchanged

## First Safety Property Backlog
The initial model should check at least the following safety properties.

### S1. No Torn Publication
No reachable state exposes partially updated publication results.

Intuition:
1. `publishedView` changes only through `A7`
2. one accepted publication step yields one coherent state transition

### S2. Reject-Is-No-Publish
A rejection never changes accepted published state.

Intuition:
1. `A6` may extend `rejectLog`
2. `A6` must not alter `publishedView` or `publishHistory`

### S3. Candidate-Result Is Not Publication
Accepted candidate results may exist without becoming published.

Intuition:
1. `acceptedCandidate` and `publishedView` are different state components
2. `A5` alone cannot change committed visibility

### S4. Fence-Safe Publication
Publication occurs only when the relevant compatibility basis holds.

Intuition:
1. `A7` is enabled only for compatible candidate work
2. incompatible work must route to `A6` rather than partial publication

### S5. Pinned-Reader Stability
Pinned readers continue to see a snapshot-compatible stable view while later work proceeds.

Intuition:
1. actions on `inFlight`, `acceptedCandidate`, or later publication may not retroactively mutate an existing pinned view
2. pinned protection constrains overlay or publication-side cleanup

### S6. Overlay Eviction Safety
No protected overlay state is discarded while still required by a pinned reader or otherwise protected stable view.

Intuition:
1. `A9` is the only path by which protection may be reduced
2. eviction eligibility must follow, not precede, reader release

## Initial Progress and Liveness Questions
The first TLA+ planning packet should name the progress questions without over-claiming them.

### L1. Reject-Or-Publish Resolution
For admitted work in the sequential baseline, can the model express that admitted work eventually reaches either rejection or accepted publication under the declared assumptions?

### L2. No Infinite Ready-But-Never-Decided Stutter
Can the model rule out a coordinator state where compatible accepted candidates remain permanently undecided in the baseline mode?

### L3. Reader Pinning Does Not Deadlock Publication
Can the model express that pinned readers constrain cleanup and not the existence of later publication itself?

These are planning questions first; strong fairness assumptions should remain explicit and minimal in the first model.

## Explicit Exclusions For The First TLA+ Packet
The first TLA+ artifact should not yet attempt to model:
1. full Stage 2 partition scheduling,
2. dynamic-topological maintenance,
3. detailed economics or optimization policy,
4. exhaustive runtime-derived effect taxonomy,
5. final trace schema shape.

## Hand-Off To W009
W009 should consume this packet by binding replay and pack expectations to:
1. `A5` versus `A7` candidate-result versus publication separation
2. `A6` typed no-publish reject outcomes
3. `A8` and `A9` pinned-reader and retention transitions
4. `S1` through `S6` as the first pack-backed safety classes

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
  - TLA+ model boundary exists, but no artifact has been authored yet
  - liveness questions are named, but fairness and progress assumptions are still open
  - replay linkage for candidate-result and fence rejects is still absent
  - W009 must still bind the safety classes to replay and pack artifacts
- claim_confidence: provisional
- reviewed_inbound_observations: `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md` missing
