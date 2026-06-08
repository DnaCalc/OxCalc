# Core Engine Transaction Scope Spike

Status: spike_decision
Owner: OxCalc
Consumer pressure: DNA TreeCalc stack-requirements W2 `transaction-scope`

## Product Status

Go for a Stage 1 OxCalcTree transaction-scope implementation, with cost `L`.

The first implementation should be an OxCalc-owned batch edit and single recalc/publication API over
the existing sequential OxCalcTree context. It should not be represented as host-only transaction ids
or Skin IR bookkeeping. Existing single-edit APIs may remain as compatibility conveniences, but the
transactional path must be the authoritative path for multi-target authoring verbs.

## Code Evidence

Current code has useful publication pieces, but no edit transaction boundary:

1. `src/oxcalc-core/src/recalc.rs`
   - `RecalcTracker::produce_candidate_result` marks one node `PublishReady`.
   - `RecalcTracker::publish_and_clear` publishes one `PublishReady` node back to `Clean`.
   - This is a per-node lifecycle helper, not a multi-edit transaction scope.
2. `src/oxcalc-core/src/coordinator.rs`
   - `TreeCalcCoordinator` admits one `AcceptedCandidateResult`, accepts it, and publishes one
     `PublicationBundle`.
   - `AcceptedCandidateResult.target_set` and `PublicationBundle.published_calc_value_delta` already
     model atomic publication of a candidate result, but this coordinator is not currently wired as a
     public edit transaction API for OxCalcTree.
3. `src/oxcalc-core/src/consumer.rs`
   - `OxCalcTreeContext::recalculate` builds one candidate outcome for the current workspace revision
     and writes publication state once when `run_state == Published`.
   - Structural and input edit methods (`add_node`, `set_node_formula_text`, `set_node_input_value`,
     `rename_node`, `move_node`, `delete_node`, `set_node_table`) mutate workspace state immediately,
     advance snapshot/input state, clear or seed publication/invalidation state, and leave recalc as a
     separate call.
   - There is no API that applies several edits, rolls back on any edit/recalc failure, and publishes
     exactly once.

## Decision

`transaction-scope` is larger than a projection exposure, but it is bounded enough to start before
revision-graph retention and candidate-overlay handles.

Implement the first slice as an OxCalcTree context feature:

```rust
pub struct OxCalcTreeEditTransaction {
    pub transaction_id: OxCalcTreeTransactionId,
    pub workspace_id: OxCalcTreeWorkspaceId,
    pub edits: Vec<OxCalcTreeEdit>,
    pub recalc_policy: TransactionRecalcPolicy,
}

pub enum TransactionRecalcPolicy {
    RecalculateAndPublishOnce,
    ApplyOnly,
}

pub struct OxCalcTreeTransactionOutcome {
    pub transaction_id: OxCalcTreeTransactionId,
    pub workspace_revision_id: WorkspaceRevisionId,
    pub calculation: Option<OxCalcTreeCalculationOutcome>,
    pub edit_count: usize,
}
```

The exact payload names can change during implementation. The invariant cannot: one conceptual
transaction is accepted as one workspace mutation and, when recalc is requested, one candidate /
publication outcome.

## First Implementation Slice

1. Add typed edit payloads for existing node-level operations only:
   - set node input/content,
   - rename node,
   - move/reorder node,
   - delete node.
2. Apply all edits against a rollback guard:
   - capture the pre-transaction workspace state,
   - capture context allocators (`next_node_id`, `next_snapshot_id`, `next_candidate_index`),
   - restore all captured state on edit failure or transactional recalc rejection.
3. Defer recalc until all edits have applied.
4. Run at most one `recalculate` for `RecalculateAndPublishOnce`.
5. Return a typed transaction outcome carrying the engine transaction id and produced workspace
   revision / calculation result.
6. Keep existing single-edit APIs as wrappers or compatibility paths; do not silently give them
   multi-edit rollback semantics.

## Required Tests

1. Two content edits in one transaction recalculate once and publish one candidate/publication result.
2. A later invalid edit in a transaction restores the prior workspace revision, publication payload,
   allocator state, pending invalidation seeds, and last result.
3. A transaction whose recalc rejects restores the prior edit and publication state.
4. Reorder/move plus content edit produces one post-transaction workspace revision and one recalc.
5. Existing single-edit APIs retain current behavior.

## Non-Goals

1. No retained revision graph in this slice.
2. No undo/redo navigation in this slice.
3. No addressable candidate overlay handles in this slice.
4. No scenario or what-if substrate in this slice.
5. No host-side transaction id fabrication as a substitute for engine outcome ids.

## Downstream Consequence

DnaTreeCalc should not mark `edit-transaction-id` complete until this engine transaction outcome is
available and threaded through `IntentReceipt`. Skin IR may carry `AuthoringScope` now, but
multi-target mutating verbs must stay blocked until the OxCalc transactional API exists.
