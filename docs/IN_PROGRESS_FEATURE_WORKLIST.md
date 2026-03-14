# IN_PROGRESS_FEATURE_WORKLIST.md — OxCalc

Canonical repo-level register of feature areas that are in-progress under workset completion doctrine.

Status: active.
Last updated: 2026-03-14.

## Status Vocabulary

- `in-progress`: partial implementation or canonical spec work exists, parity/completeness not yet achieved.
- `blocked`: in-progress with active blocker (see CURRENT_BLOCKERS.md).
- `planned`: explicitly accepted into scope, no shipped work yet.

## Active Feature Register

### IP-01: Core Rewrite and Canonicalization

- **Status**: in-progress
- **Current floor**: rewritten canonical core-engine spec set drafted; bootstrap set archived; repo integration still underway.
- **Remaining gaps**: supersession treatment finalization, workset alignment, OxFml handoff packet drafting, follow-on tightening passes.
- **Why still open**: rewrite has produced the canonical set but repo-level integration is not yet fully closed.
- **Canonical owner**: W001.

### IP-02: TreeCalc Structural State and Snapshot Kernel

- **Status**: planned
- **Current floor**: canonical architecture and state-kernel docs drafted; no exercised implementation.
- **Remaining gaps**: stable-ID policy closure, immutable structural kernel realization, pinned-reader semantics, projection/facade realization.
- **Why still open**: architecture is documented but not realized or verified yet.
- **Canonical owner**: W002.

### IP-03: Stage 1 Coordinator and Publication Baseline

- **Status**: planned
- **Current floor**: canonical coordinator/publication architecture drafted; no exercised implementation.
- **Remaining gaps**: single-publisher realization, accept/reject enforcement, atomic publication bundle, stable observer-visible state, replay-oriented reject detail.
- **Why still open**: coordinator remains spec-first and unexercised.
- **Canonical owner**: W003.

### IP-04: Incremental Recalc and Overlay Baseline

- **Status**: planned
- **Current floor**: canonical recalc and overlay architecture drafted; no exercised implementation.
- **Remaining gaps**: invalidation-state machine, verification-oriented incremental subset, dynamic-dependency overlay handling, fallback/economics instrumentation.
- **Why still open**: baseline direction is now explicit, but realization and evidence are not in place.
- **Canonical owner**: W004.

### IP-05: OxFml Seam Hardening and Handoff Closure

- **Status**: planned
- **Current floor**: OxCalc-local seam requirements are drafted; no handoff packet filed yet.
- **Remaining gaps**: shared-clause packet for accepted-result payloads, reject taxonomy/detail, fence consequences, runtime-derived reporting requirements.
- **Why still open**: canonical shared seam changes require OxFml-side acknowledgment.
- **Canonical owner**: W005.

### IP-06: Core Formalization and Gate Binding

- **Status**: planned
- **Current floor**: formalization/assurance and realization roadmap docs drafted; no realized Lean/TLA+/pack closure yet.
- **Remaining gaps**: initial Lean model objects, initial TLA+ coordinator model, staged gate binding to real worksets and pack artifacts.
- **Why still open**: near-formal direction is specified but not yet exercised.
- **Canonical owner**: W006.
