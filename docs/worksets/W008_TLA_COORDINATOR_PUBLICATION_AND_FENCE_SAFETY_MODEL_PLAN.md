# W008: TLA+ Coordinator, Publication, and Fence-Safety Model Plan

## Purpose
Define the first TLA+-oriented model scope for coordinator authority, publication safety, reject behavior, fence compatibility, and the Stage 1 transition packets now named in W003 and W004.

## Position and Dependencies
- **Depends on**: W006, W007
- **Blocks**: W009
- **Cross-repo**: relies on accepted OxFml seam direction for candidate-result versus publication boundaries

## Scope
### In scope
1. State-machine boundary for coordinator, in-flight work, accepted candidate result, published state, and Stage 1 invalidation state.
2. Safety-property backlog for reject-is-no-publish, no torn publication, fence compatibility, and pinned-reader or overlay protection.
3. Initial liveness or progress question list for later staged concurrency work.
4. Explicit binding from the Stage 1 coordinator and recalc transition packets into the first TLA+ action surface.

### Out of scope
1. Full TLA+ model implementation.
2. Stage 2 contention policy closure.
3. Complete fairness or starvation analysis.

## Deliverables
1. A TLA+-oriented planning packet for coordinator and fence-safety artifact authoring.
2. A first transition-binding matrix from W003/W004 Stage 1 transitions into TLA+ variables, guards, and actions.

## Gate Model
### Entry gate
- W007 has stabilized the initial state vocabulary and transition boundaries.
- W003 and W004 have named the Stage 1 coordinator and invalidation transition packets.

### Exit gate
- The coordinator model boundary and safety backlog are explicit enough to author the first TLA+ artifact.
- The model scope is aligned to the accepted `AcceptedCandidateResult` versus publication split and to W009 replay planning.
- The W003 and W004 Stage 1 transition packets are explicitly mapped into the first TLA+ action surface.

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
10. `nodeCalcState`
   - abstract per-node Stage 1 invalidation-state map using the W004 local vocabulary.
11. `demandSet`
   - abstract set of nodes currently required for the stabilization frontier.
12. `evictionEligibility`
   - abstract retention/eviction eligibility markers for overlay cleanup after release or publication.

### Derived or Ghost Variables
The initial model may also use ghost structure for proof or checking convenience:
1. `compatBasis`
   - abstract compatibility tuple covering snapshot, token, profile, and capability basis.
2. `publishHistory`
   - abstract history of committed publication events.
3. `decisionHistory`
   - abstract history of accept or reject decisions.
4. `transitionHistory`
   - abstract record of the Stage 1 transition labels exercised by the run.

## Object Mapping From W007 and Stage 1 Worksets
The W007 vocabulary and the newer Stage 1 workset packets should map into the TLA+ state shape as follows:
1. `CoordinatorState` -> `coordState`
2. `AcceptedCandidateResultRef` -> members of `acceptedCandidate`
3. `CommitBundleRef` -> elements recorded in `publishHistory` and reflected in `publishedView`
4. `PinnedView` -> elements of `pinnedReaders`
5. `RejectDetail` -> elements of `rejectLog`
6. transitions `T4` through `T9` -> abstract TLA+ actions `A4` through `A9`
7. W004 invalidation states `clean`, `dirty_pending`, `needed`, `evaluating`, `verified_clean`, `publish_ready`, `rejected_pending_repair`, `cycle_blocked` -> `nodeCalcState`
8. W003 coordinator packet `C1` through `C6` and W004 invalidation packet `I1` through `I8` -> the first Stage 1 binding matrix below

## Stage 1 Transition Binding Matrix
The first TLA+ packet should explicitly bind the Stage 1 workset transitions into abstract actions and state guards.

| Workset transition | Primary TLA+ action or action family | Minimum state variables touched | Minimum guard pressure |
|---|---|---|---|
| `I1 MarkDirty` | `A1 MarkDirty` | `nodeCalcState`, `runtimeView` | upstream change, structural edit, or external invalidation exists |
| `I2 MarkNeeded` | `A2 MarkNeeded` | `nodeCalcState`, `demandSet` | node is `dirty_pending` and demanded by current stabilization objective |
| `I3 BeginEvaluate` | `A3 BeginEvaluate` | `nodeCalcState`, `inFlight`, `compatBasis` | node is `needed`, basis is compatible, and no conflicting in-flight owner exists |
| `I4 VerifyClean` | `A3b VerifyClean` | `nodeCalcState`, `decisionHistory` | node is `evaluating` and verification proves unchanged observable result |
| `I5 ProduceDependencyShapeUpdate` | `A5 RecordAcceptedCandidateResult` | `nodeCalcState`, `acceptedCandidate`, `overlayState` | node is `evaluating` and evaluator output is well-formed |
| `I6 RejectOrFallback` | `A6 RejectCandidateWork` | `nodeCalcState`, `rejectLog`, `overlayState` | typed reject, incompatible overlay basis, or insufficient effect detail |
| `I7 PublishAndClear` | `A7 AcceptAndPublish` | `nodeCalcState`, `publishedView`, `publishHistory`, `overlayState` | candidate is compatible and accepted for atomic publish |
| `I8 ReleaseAndEvictEligible` | `A9 UnpinAndReleaseProtection` or `A10 MarkEvictionEligible` | `nodeCalcState`, `demandSet`, `evictionEligibility` | demanded frontier released and no pin still protects prior state |
| `C1 AdmitCandidateWork` | `A4 AdmitCandidateWork` | `coordState`, `inFlight`, `compatBasis` | candidate intake packet received |
| `C2 RecordAcceptedCandidateResult` | `A5 RecordAcceptedCandidateResult` | `acceptedCandidate`, `coordState` | candidate passes shape validation and remains undecided |
| `C3 RejectCandidateWork` | `A6 RejectCandidateWork` | `coordState`, `rejectLog`, `acceptedCandidate` | typed reject or fence mismatch detected |
| `C4 AcceptAndPublish` | `A7 AcceptAndPublish` | `publishedView`, `publishHistory`, `coordState`, `acceptedCandidate` | compatibility basis still holds |
| `C5 PinReader` | `A8 PinReader` | `pinnedReaders`, `publishedView` | reader requests stable-view protection |
| `C6 UnpinAndReleaseProtection` | `A9 UnpinAndReleaseProtection` | `pinnedReaders`, `overlayState`, `evictionEligibility` | protected reader releases pin |

This matrix is intentionally conservative. It does not force the first TLA+ artifact to model every optimization, but it does force every named Stage 1 transition to have a formal hook.

## Initial Abstract Actions
The first TLA+ action set should align to the publication-critical transitions from W007 and the Stage 1 realization packets.

### A0. Init
Establish initial state for:
1. `structSnapshot`
2. empty `runtimeView`
3. empty `inFlight`
4. empty `acceptedCandidate`
5. initial stable `publishedView`
6. empty `rejectLog`
7. initial `nodeCalcState`

### A1. MarkDirty
Abstract counterpart to W004 `I1`.

Effects:
1. change `nodeCalcState` from `clean` or `verified_clean` to `dirty_pending`
2. record that the stale frontier has expanded
3. leave `publishedView` unchanged

### A2. MarkNeeded
Abstract counterpart to W004 `I2`.

Effects:
1. move a `dirty_pending` node into the demanded frontier
2. update `demandSet`
3. leave `publishedView` unchanged

### A3. BeginEvaluate
Abstract counterpart to W004 `I3`.

Effects:
1. move a `needed` node into `evaluating`
2. create or update the associated in-flight execution record
3. freeze the compatibility basis used for later publish or reject decisions

### A3b. VerifyClean
Abstract counterpart to W004 `I4`.

Effects:
1. move an `evaluating` node to `verified_clean`
2. record a no-publication resolution for the demanded work item
3. leave `publishedView` unchanged

### A4. AdmitCandidateWork
Abstract counterpart to W003 `C1`.

Effects:
1. add a work item to `inFlight`
2. record the compatibility basis required for later accept or reject
3. leave `publishedView` unchanged

### A5. RecordAcceptedCandidateResult
Abstract counterpart to W003 `C2` and W004 `I5`.

Effects:
1. move an in-flight work item into a coordinator-ready accepted-candidate state
2. preserve the distinction between evaluator success and publication
3. register any dependency-shape or runtime-effect consequences in abstract `overlayState`
4. leave `publishedView` unchanged

### A6. RejectCandidateWork
Abstract counterpart to W003 `C3` and W004 `I6`.

Effects:
1. remove or mark the affected in-flight or accepted candidate as rejected
2. append typed `RejectDetail` to `rejectLog`
3. transition the affected node into `rejected_pending_repair` where appropriate
4. preserve `publishedView` unchanged

### A7. AcceptAndPublish
Abstract counterpart to W003 `C4` and W004 `I7`.

Effects:
1. consume a compatible accepted candidate
2. append a committed bundle to `publishHistory`
3. update `publishedView` atomically
4. clear the affected node back to `clean`
5. update coordinator decision state accordingly

### A8. PinReader
Abstract counterpart to W003 `C5`.

Effects:
1. add a pinned reader entry tied to the current stable `publishedView` and `structSnapshot`
2. leave publication state unchanged

### A9. UnpinAndReleaseProtection
Abstract counterpart to W003 `C6` and one path of W004 `I8`.

Effects:
1. remove or relax pinned-reader protection
2. update overlay or retention eligibility metadata
3. leave accepted publication state unchanged

### A10. MarkEvictionEligible
Abstract counterpart to W004 `I8` when eligibility changes are modeled separately from unpin events.

Effects:
1. mark overlay entries as eviction-eligible after demand or pin protection has been released
2. leave `publishedView` unchanged
3. preserve the rule that eligibility is not eviction itself

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
1. `A9` and `A10` are the only paths by which protection or eviction eligibility may be reduced
2. eviction eligibility must follow, not precede, reader release or frontier release

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
2. `A1` through `A3b` plus `A5` through `A7` for the Stage 1 invalidation and publication chain
3. `A6` typed no-publish reject outcomes
4. `A8`, `A9`, and `A10` for pinned-reader and retention transitions
5. `S1` through `S6` as the first pack-backed safety classes

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
  - the first TLA+ Stage 1 skeleton now exists under `formal/tla/`, but no TLC run or syntax check has been executed in this repo yet
  - liveness questions are named, but fairness and progress assumptions are still open
  - replay linkage for the implemented invalidation and publication transitions is still absent
  - W009 must still bind the safety classes and transition families to replay and pack artifacts
- claim_confidence: provisional
- reviewed_inbound_observations: `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md` missing
