# Core Engine Transaction Scope Spike

Status: first_slice_implemented
Owner: OxCalc
Consumer pressure: DNA TreeCalc stack-requirements W2 `transaction-scope`

## Product Status

Go for a Stage 1 OxCalcTree transaction-scope implementation, with cost `L`.

The first implementation should be an OxCalc-owned batch edit and single recalc/publication API over
the existing sequential OxCalcTree context. It should not be represented as host-only transaction ids
or Skin IR bookkeeping. Existing single-edit APIs may remain as compatibility conveniences, but the
transactional path must be the authoritative path for multi-target authoring verbs.

Implemented slices:

1. `OxCalcTreeContext::apply_edit_transaction`.
2. Engine-generated `OxCalcTreeTransactionId`.
3. `OxCalcTreeEdit` payloads for existing node input/formula, rename, move/reorder, and delete
   operations.
4. `TransactionRecalcPolicy::{RecalculateAndPublishOnce, ApplyOnly}`.
5. Rollback on edit failure and transactional recalc rejection.
6. Engine-owned node id reservation for transaction builders that must reference generated nodes in
   later edits, through `OxCalcTreeContext::reserve_node_id` and
   `OxCalcTreeNodeCreate::with_reserved_node_id`.

## Original Readiness Finding

The spike began from useful publication pieces, but no edit transaction boundary:

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
   - At spike start there was no API that applied several edits, rolled back on any edit/recalc
     failure, and published exactly once. The implemented slices above add that API for the declared
     Stage 1 scope.

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

Status: implemented for existing node-level operations plus reserved-id generated-node table
transactions.

Implemented:

1. Typed edit payloads for existing node-level operations:
   - set node input/content,
   - rename node,
   - move/reorder node,
   - delete node.
2. All edits apply against a rollback guard:
   - capture the pre-transaction workspace state,
   - capture context allocators (`next_node_id`, `next_snapshot_id`, `next_candidate_index`),
   - restore all captured state on edit failure or transactional recalc rejection.
3. Recalc is deferred until all edits have applied.
4. At most one `recalculate` runs for `RecalculateAndPublishOnce`.
5. A typed transaction outcome carries the engine transaction id and produced workspace
   revision / calculation result.
6. Existing single-edit APIs keep their current behavior; they are not silently given
   multi-edit rollback semantics.
7. Transaction builders can reserve engine-owned node ids before applying a transaction, then use
   those ids in `AddNode` requests and later transaction edits such as `SetNodeTable` snapshots.
   This supports generated table cell nodes without host-side id prediction.

## Required Tests

1. Two content edits in one transaction recalculate once and publish one candidate/publication result.
2. A later invalid edit in a transaction restores the prior workspace revision, publication payload,
   allocator state, pending invalidation seeds, and last result.
3. A transaction whose recalc rejects restores the prior edit and publication state.
4. Reorder/move plus content edit produces one post-transaction workspace revision and one recalc.
5. Existing single-edit APIs retain current behavior.
6. Reserved generated node ids can be referenced by a later table snapshot edit in the same
   transaction.

Implemented evidence:

1. `treecalc_context_edit_transaction_publishes_once_for_multiple_node_edits`
2. `treecalc_context_edit_transaction_rolls_back_on_edit_failure`
3. `treecalc_context_edit_transaction_rolls_back_on_recalc_rejection`
4. `treecalc_context_edit_transaction_moves_and_edits_with_one_publication`
5. `treecalc_context_edit_transaction_can_reference_reserved_added_node_ids`

## Non-Goals

1. No retained revision graph in this slice.
2. No undo/redo navigation in this slice.
3. No addressable candidate overlay handles in this slice.
4. No scenario or what-if substrate in this slice.
5. No host-side transaction id fabrication as a substitute for engine outcome ids.

## Downstream Consequence

DnaTreeCalc can mark current W2 `edit-transaction-id` coverage complete when host receipts are
threaded from these engine transaction outcomes. Skin IR may carry `AuthoringScope`; scoped existing
node edits can route through batch transactions, while later W3 authoring verbs should add their own
closed intents and consume this transaction API rather than fabricating host transaction ids.
