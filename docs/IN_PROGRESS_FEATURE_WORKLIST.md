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

- **Status**: in-progress
- **Current floor**: OxCalc-local seam requirements are drafted; `HANDOFF-CALC-001` is filed, acknowledged by OxFml, and reflected in OxFml canonical seam updates.
- **Remaining gaps**: OxCalc-side alignment to `AcceptedCandidateResult` terminology and typed reject consequences, replay artifacts for candidate-result versus publication boundaries, exhaustive runtime-derived effect taxonomy, and any narrower follow-on handoff if required.
- **Why still open**: acknowledgment exists, but evidence and full downstream alignment are still partial.
- **Canonical owner**: W005.

### IP-06: Core Formalization and Gate Binding

- **Status**: planned
- **Current floor**: formalization/assurance and realization roadmap docs drafted; no realized Lean/TLA+/pack closure yet.
- **Remaining gaps**: initial Lean model objects, initial TLA+ coordinator model, staged gate binding to real worksets and pack artifacts.
- **Why still open**: near-formal direction is specified but not yet exercised.
- **Canonical owner**: W006.
