# CORE_ENGINE_CANDIDATE_OVERLAY_HANDLE_SPIKE.md

Status: `parented_copy_layer_slice_landed`
Roadmap source: `../DnaTreeCalc/docs/ux/stack-requirements/ROADMAP.md` W4b
Execution bead: `calc-etez`

## Purpose

This spike answers the DnaTreeCalc W4b open question for
`candidate-overlay-handle`: can OxCalc expose the requested speculation
substrate by threading existing candidate or runtime-overlay state through the
host, or is new engine capability required?

The answer is: **new engine substrate is required, but it is schedulable.**
The first copy-based non-publishing candidate slice, first commit bridge,
first parented copy-layer slice, and candidate-private revision-history slice
are now implemented in `OxCalcTreeContext`, and DnaTreeCalc consumes them
through a content-only host/Skin IR projection.

OxCalc already has candidate/publication separation inside one synchronous
recalc attempt. It does not yet have handle-addressed, layerable,
non-publishing candidate contexts that a host can keep, compare, discard, or
commit later.

## Live Code Assessment

Current implementation facts:

| Surface | Current code state | W4b consequence |
|---|---|---|
| `OxCalcTreeContext::recalculate` | Builds one `candidate_result_id`, executes `LocalTreeCalcEngine`, then either publishes into workspace state or rejects. | This is a publish-at-end run path, not an addressable candidate context. |
| `TreeCalcCoordinator` | Holds at most one `in_flight_candidate` and one `accepted_candidate`; `accept_and_publish` clears both after publication. | The coordinator models candidate-vs-publication correctly, but not N retained candidates. |
| `RuntimeOverlaySet` | Stored on workspace state and keyed from the current `PublicationSnapshotId`. It is replaced from accepted runtime-effect overlays on published recalc. | This is published-basis overlay state, not a speculative overlay registry. |
| `Stage1RecalcTracker` | Tracks node execution states and transient overlay entries during a recalc. `publish_and_clear` returns a node to clean state. | This is execution bookkeeping for one run, not a host-addressable speculation object. |
| Retained revisions | W4a now retains in-memory workspace revisions and can navigate them. | This is the right immutable basis for candidates, but candidates are not stored yet. |
| Per-node published value epochs | Current code distinguishes published value epochs from input epochs. | This gives candidates a basis/provenance anchor, but does not by itself create speculation. |

The prior W047/W048 CTRO documents use "candidate overlay" to describe
runtime-derived dependency consequences before a publish/reject decision inside
a single run. That concept remains valid, but W4b asks for a broader product
substrate: retained candidate contexts that are addressable by handle and
structurally incapable of advancing the published workspace until an explicit
commit path runs.

## Go / No-Go

Go for implementation as a new OxCalc substrate.

No-go for any downstream host or Skin IR claim that treats the current
`RuntimeOverlaySet`, `AcceptedCandidateResult`, or diagnostic-only rejected
candidate facts as an addressable scenario/what-if substrate.

## Owning Boundary

| Layer | Owns | Must not own |
|---|---|---|
| OxCalc | Candidate handles, candidate registry, basis revision validation, candidate evaluation, non-publication isolation, discard, commit bridge into normal transaction/publication, candidate retention pressure, candidate provenance. | Formula parsing or formula rewrite. |
| OxFml | Formula parse/bind/evaluate artifacts and runtime candidate packet meaning for each evaluated formula. | Multi-node candidate lifecycle, candidate registry, publication state, or scenario persistence. |
| DnaTreeCalc host | Closed intents for preview/commit/discard, labels/pins, projection of OxCalc candidate facts, product command grouping. | Evaluating what-if values, mutating published values for preview, inventing dependency facts, or replaying inverse edits. |
| Skin IR / skins | Rendering published/speculative provenance and dispatching closed intents. | Any semantic interpretation of formulas, dependency graphs, candidate validity, or publication. |

## Required Engine Shape

The first target API should be additive to `OxCalcTreeContext`:

```rust
pub struct CandidateOverlayHandle(/* opaque */);

pub struct OxCalcTreeOpenCandidateRequest {
    pub workspace_id: OxCalcTreeWorkspaceId,
    pub basis_revision_id: WorkspaceRevisionId,
    pub parent_candidate: Option<CandidateOverlayHandle>,
}

pub struct OxCalcTreeCandidateEditTransaction {
    pub transaction_id: OxCalcTreeTransactionId,
    pub edits: Vec<OxCalcTreeEdit>,
}

pub struct OxCalcTreeCandidateView {
    pub handle: CandidateOverlayHandle,
    pub basis_revision_id: WorkspaceRevisionId,
    pub parent_candidate: Option<CandidateOverlayHandle>,
    pub run_state: OxCalcTreeRunState,
    pub value_epoch_basis: u64,
    pub published_revision_unchanged: WorkspaceRevisionId,
}
```

The implementation does not need to freeze these names before coding, but the
first slice must preserve these properties:

1. opening a candidate validates that its basis revision is retained,
2. evaluating a candidate never mutates `workspace_revision`,
   `publication_snapshot`, `publication_value_epoch`, or published values on
   the workspace,
3. discarding a candidate removes its handle and retained candidate artifacts,
4. committing a candidate is the only bridge to the live workspace publication
   path,
5. parent candidates are represented explicitly before layering semantics are
   claimed,
6. candidate handles are opaque at the host boundary.

## First Build Slice

The first implementation slice deliberately avoids scenario UI breadth. It
proves only the substrate invariant:

1. open a candidate on a retained revision,
2. apply a single node content edit into the candidate's private
   workspace-state copy,
3. evaluate the candidate into an addressable candidate view,
4. assert the published workspace revision, publication snapshot, published
   values, and published value epochs are unchanged,
5. discard the candidate and assert the handle is no longer readable.

Implementation:

1. `CandidateOverlayHandle`,
2. `OxCalcTreeOpenCandidateRequest`,
3. `OxCalcTreeCandidateView`,
4. `OxCalcTreeContext::open_candidate`,
5. `OxCalcTreeContext::apply_candidate_edit_transaction`,
6. `OxCalcTreeContext::evaluate_candidate`,
7. `OxCalcTreeContext::candidate_view`,
8. `OxCalcTreeContext::discard_candidate`.

## Commit Bridge Slice

The first commit bridge is implemented as `OxCalcTreeContext::commit_candidate`.

Scope:

1. commit is allowed only while the live workspace revision still equals the
   candidate basis revision,
2. commit replaces the live workspace with the candidate's private workspace
   state, including the candidate's already-evaluated publication and retained
   revision lineage,
3. commit removes the candidate handle,
4. stale-basis commit returns typed `CandidateBasisNotCurrent` and keeps the
   candidate retained for later discard or future rebase semantics.

This is deliberately not a rebase engine and not the optimized layered
candidate model. Those remain successor work under bead `calc-4ipg`.

The commit bridge is still conservative about transaction semantics: candidate
private edits already run through OxCalc edit-transaction machinery with
`ApplyOnly`; candidate evaluation then runs through OxCalc recalc/publication
inside the candidate context; commit promotes that candidate-owned state only
if the live basis is unchanged. A future widening may expose the candidate's
transaction summary more directly to hosts, but this slice does not ask the
host to replay edits or synthesize inverse operations.

## Parented Copy-Layer Slice

The first parent/layer semantics are implemented as copy-at-open layering:

1. opening a child candidate requires a retained parent candidate handle,
2. the parent candidate must belong to the requested workspace,
3. the parent candidate's basis revision must equal the requested child basis,
4. the child candidate's private workspace state starts as a copy of the
   parent's current private workspace state at child-open time,
5. later parent edits do not live-update already-open children in this slice,
6. parent discard or commit is rejected while a retained child depends on it,
7. a child commit publishes the child's stacked private state when the live
   workspace basis is still current.

This gives hosts a truthful parent handle and stacked what-if value surface
without claiming rebase, live parent subscriptions, optimized overlay deltas,
or scenario persistence.

## Cost And Risk

Rough cost: **large**.

Risk drivers:

1. OxCalc must split "run to publication" from "run to candidate result" in the
   `OxCalcTreeContext` facade without weakening coordinator authority.
2. Candidate state needs retention and deterministic eviction policy aligned
   with W054 rather than ad hoc host lifetime.
3. The first copy-based candidate implementation is acceptable as a correctness
   substrate, but later optimized layering must prove equivalence to the
   copy-based baseline.
4. Commit must not become inverse replay or host-applied mutation; it must
   publish through the existing transaction/recalc path.

## Fresh-Eyes Review Notes

Reviewed against:

1. `src/oxcalc-core/src/consumer.rs` `recalculate` and workspace state,
2. `src/oxcalc-core/src/coordinator.rs` candidate/publication fields,
3. `src/oxcalc-core/src/recalc.rs` `Stage1RecalcTracker`,
4. current W4b roadmap requirements in DnaTreeCalc,
5. inbound OxFml observations about runtime/evaluator ownership.

No current source contradicts the spike conclusion. The main naming risk is
overloading "candidate overlay": W047/W048 already use it for one-run CTRO
consequences, while W4b needs retained host-addressable speculation handles.
Specs and code should use `CandidateOverlayHandle` or `candidate context` for
the W4b substrate to avoid confusing it with transient publication candidates.

Fresh-eyes implementation review after the first slice found no publication
leak in the exercised path: candidate evaluation runs through a temporary
private context, the live workspace remains unchanged, and discard removes the
handle. The commit bridge adds a basis-current guard before promoting candidate
state. The copy-based path is intentionally conservative; optimization and
layering must refine against this behavior rather than replacing it with a
looser semantic claim.

## Status

Product status: W4b `candidate-overlay-handle` has its first OxCalc-owned
substrate, commit, parented copy-layer, and candidate-private revision-history
slices: a host can open an opaque candidate handle on a retained revision,
apply a private node edit, evaluate private candidate results, discard without
publishing, commit into the live workspace when the candidate basis is still
current, open a child candidate over a retained parent candidate's private
state, and read candidate-private revision graph entries with real transaction
ids.

Evidence: source inspection of `consumer.rs`, `coordinator.rs`, and `recalc.rs`
confirms one synchronous publish/reject candidate lane and one published-basis
runtime overlay set before this slice. The first implementation test
`treecalc_context_candidate_evaluation_does_not_publish_workspace_state`
exercises open/edit/evaluate/discard and asserts the live workspace revision,
publication snapshot, runtime overlay set, visible value, and published value
epoch are unchanged. `treecalc_context_candidate_commit_publishes_private_candidate_state`
exercises successful commit and proves the promoted private revision entry
carries the real apply-only transaction id without fabricating an invalidation
summary. `treecalc_context_candidate_commit_rejects_when_basis_is_no_longer_current`
exercises the stale-basis guard. `treecalc_context_child_candidate_starts_from_parent_private_state`
and `treecalc_context_child_candidate_commit_publishes_layered_state` exercise
copy-at-open parent layering and parent lifecycle guards.

Still open: live parent rebase/subscription semantics, optimized overlay-delta
layering, candidate structural edits, scenario/what-if Skin IR, richer
candidate transaction summaries for host consumption, and W054-aligned
candidate retention/GC.

Formal status: no new proof claim. The first implementation should become the
copy-based Stage 1 baseline that later optimized/layered candidates refine
against.
