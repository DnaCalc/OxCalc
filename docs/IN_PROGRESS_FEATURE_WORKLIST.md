# IN_PROGRESS_FEATURE_WORKLIST.md — OxCalc

Canonical repo-level register of feature areas that are in-progress under workset completion doctrine.

Status: active.
Last updated: 2026-03-15.

## Status Vocabulary

- `in-progress`: partial implementation or canonical spec work exists, parity/completeness not yet achieved.
- `blocked`: in-progress with active blocker (see CURRENT_BLOCKERS.md).
- `planned`: explicitly accepted into scope, no shipped work yet.

## Active Feature Register

### IP-01: Core Rewrite and Canonicalization

- **Status**: in-progress
- **Current floor**: rewritten canonical core-engine spec set drafted; bootstrap set archived; repo integration now includes first OxFml seam handoff and receiving-side acknowledgment tracking.
- **Remaining gaps**: final integration tightening, follow-on seam alignment wording, workset closure discipline, and later replay-backed evidence.
- **Why still open**: the canonical set is established, but realization and assurance closure are still outstanding.
- **Canonical owner**: W001.

### IP-02: TreeCalc Structural State and Snapshot Kernel

- **Status**: planned
- **Current floor**: canonical architecture and state-kernel docs drafted; no exercised implementation.
- **Remaining gaps**: stable-ID policy closure, immutable structural kernel realization, pinned-reader semantics, projection or facade realization.
- **Why still open**: architecture is documented but not realized or verified yet.
- **Canonical owner**: W002.

### IP-03: Stage 1 Coordinator and Publication Baseline

- **Status**: planned
- **Current floor**: canonical coordinator and publication architecture drafted; no exercised implementation.
- **Remaining gaps**: single-publisher realization, accept/reject enforcement, atomic publication bundle, stable observer-visible state, replay-oriented reject detail.
- **Why still open**: coordinator remains spec-first and unexercised.
- **Canonical owner**: W003.

### IP-04: Incremental Recalc and Overlay Baseline

- **Status**: planned
- **Current floor**: canonical recalc and overlay architecture drafted; no exercised implementation.
- **Remaining gaps**: invalidation-state machine, verification-oriented incremental subset, dynamic-dependency overlay handling, fallback or economics instrumentation.
- **Why still open**: baseline direction is now explicit, but realization and evidence are not in place.
- **Canonical owner**: W004.

### IP-05: OxFml Seam Hardening and Handoff Closure

- **Status**: in-progress
- **Current floor**: OxCalc-local seam requirements are drafted; `HANDOFF-CALC-001` is filed, acknowledged by OxFml, and reflected in OxFml canonical seam updates.
- **Remaining gaps**: OxCalc-side alignment to `AcceptedCandidateResult` terminology and typed reject consequences, replay artifacts for candidate-result versus publication boundaries, exhaustive runtime-derived effect taxonomy, and any narrower follow-on handoff if required.
- **Why still open**: acknowledgment exists, but evidence and full downstream alignment are still partial.
- **Canonical owner**: W005.

### IP-06: Core Formalization and Gate Binding

- **Status**: in-progress
- **Current floor**: formalization and assurance direction is drafted, W006 is active, W007 contains the first Lean-facing object inventory and transition-boundary packet, W008 contains the first TLA+-oriented coordinator-state and safety-boundary packet, W009 contains the first replay-class and pack-binding packet, and W010 now contains the first experiment-register and measurement-schema packet.
- **Remaining gaps**: actual Lean or TLA+ artifact authoring, replay and pack artifact creation, initial counter-schema drafting, and execution of the remaining assurance-planning sequence.
- **Why still open**: the assurance lane now has initial state, concurrency-model, replay-binding, and measurement-planning packets, but not yet exercised formal, replay, or instrumentation artifacts.
- **Canonical owner**: W006.

### IP-07: Self-Contained Test Harness Planning

- **Status**: in-progress
- **Current floor**: `W011` now defines the first planning packet for a self-contained harness, minimal OxFml test-double surface, scriptable host, and alternate non-Excel calculation space for engine-only testing.
- **Remaining gaps**: concrete test-double payload shape, scenario schema, scriptable host contract, and later fixture implementation.
- **Why still open**: the harness boundary is now planned, but no fixture or host artifacts exist yet.
- **Canonical owner**: W011.
