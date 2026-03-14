# HANDOFF-CALC-001: Coordinator-Facing Seam Hardening for Accepted Results, Reject Detail, and Fence Consequences

## Purpose
This handoff packet requests canonical OxFml-side seam tightening for the coordinator-facing clauses that OxCalc now treats as baseline architecture requirements.

The goal is not to transfer ownership of the shared seam away from OxFml. The goal is to make the shared canonical seam text explicit enough for OxCalc's single-publisher coordinator, atomic publication, reject-is-no-publish, and staged concurrency hardening work.

## Source Scope
- Source workset: `W005_OXFML_SEAM_HARDENING_AND_HANDOFF_PACKET`
- OxCalc-local source docs:
  - `docs/spec/core-engine/CORE_ENGINE_OXFML_SEAM.md`
  - `docs/spec/core-engine/CORE_ENGINE_COORDINATOR_AND_PUBLICATION.md`
  - `docs/spec/core-engine/CORE_ENGINE_OVERLAY_AND_DERIVED_RUNTIME.md`
  - `docs/spec/core-engine/CORE_ENGINE_REALIZATION_ROADMAP.md`

## Core Message
OxCalc now treats the coordinator as the single publication authority and requires a sharper shared seam distinction between:
1. evaluator-produced candidate results,
2. coordinator accept/reject decisions,
3. stable published consequences.

To support that cleanly, the canonical shared seam needs stronger text for accepted-result payload structure, structured reject detail, and snapshot/token/capability fence consequences.

## Requested Canonical Shared-Seam Tightening

### 1. Accepted-Result Payload Structure
The canonical shared seam should state explicitly that accepted evaluator work must provide a candidate result structure rich enough for coordinator-controlled atomic publication.

Coordinator-facing requirement:
1. accepted candidate results must not collapse into an opaque "success" signal,
2. the coordinator must be able to publish one coherent derived bundle for accepted work,
3. the seam must preserve the distinction between evaluator output and committed publication.

Requested canonical clause direction:
1. define the accepted candidate-result payload boundary explicitly,
2. identify which result components are required for coordinator publication,
3. avoid wording that implies evaluator success is itself publication.

### 2. Structured Reject Detail
The canonical shared seam should require structured reject detail suitable for deterministic replay and coordinator diagnostics.

Coordinator-facing requirement:
1. rejected work is strict no-publish,
2. the coordinator must still be able to distinguish why work was rejected,
3. reject detail must be structured enough for staged concurrency hardening and replay analysis.

Requested canonical clause direction:
1. make reject detail mandatory rather than incidental,
2. ensure reject detail can express at least:
   - snapshot/fence incompatibility,
   - artifact or token mismatch,
   - capability/session mismatch where relevant,
   - other coordinator-relevant reject classes needed for deterministic replay.

### 3. Fence Consequences
The canonical shared seam should tighten how candidate work is related to snapshot, token, capability, and publication-fence consequences.

Coordinator-facing requirement:
1. candidate work must carry enough compatibility basis for the coordinator to decide accept versus reject,
2. stale or incompatible work must be rejectable without ambiguity,
3. later concurrent stages must not require seam reinterpretation.

Requested canonical clause direction:
1. make the compatibility basis of candidate work explicit,
2. state more clearly that incompatible candidate work is rejected rather than partially published,
3. preserve deterministic replay and diagnostic visibility of fence failures.

### 4. Runtime-Derived Effect Reporting Where Required
Where evaluator execution discovers facts that materially affect coordinator behavior, the shared seam should make clear how such effects are exposed or derivable.

Coordinator-facing requirement:
1. if OxCalc must coordinate on runtime-discovered effects, those effects cannot remain purely hidden evaluator internals,
2. the seam must support explicit transmission or derivation of such effects when they influence coordinator correctness.

Requested canonical clause direction:
1. clarify the boundary between evaluator-local detail and coordinator-relevant runtime-derived effects,
2. state what must be surfaced when those effects matter to accept/reject/publication behavior.

## Proposed Normative Direction
The following statements summarize the intended shared direction in OxCalc-local terms.
These are not canonical OxFml text yet; they are candidate target statements for synthesis on the OxFml side.

1. Evaluator-produced candidate results and committed publication are distinct layers.
2. Accepted candidate work must provide structured result content adequate for coordinator-controlled atomic publication.
3. Rejected candidate work publishes no accepted state and must emit structured reject detail suitable for deterministic replay and diagnostics.
4. Snapshot, token, capability, and related compatibility consequences must be explicit enough that the coordinator can deterministically accept or reject candidate work.
5. Coordinator-relevant runtime-derived effects must be surfaced or derivable where they affect correctness.

## Migration and Fallback Impact

### If Accepted
1. OxCalc can align its coordinator/publication implementation and assurance model to sharper shared seam wording.
2. Stage 1 and pre-Stage 2 hardening become less likely to drift from OxFml canonical seam text.
3. Reject replay, concurrency hardening, and publication modeling become easier to keep cross-repo consistent.

### If Deferred
1. OxCalc can still proceed with local coordinator architecture work.
2. However, the shared seam remains at higher risk of wording drift or implicit assumptions.
3. Pre-Stage 2 concurrency hardening becomes harder to coordinate cleanly across repos.

## Evidence and References
This packet is currently a spec-architecture handoff draft.

Current evidence basis:
1. rewritten OxCalc canonical architecture and seam set,
2. the Foundation March 5-9, 2026 research/synthesis line already promoted into OxCalc local architecture,
3. explicit OxCalc need for single-publisher coordinator semantics and reject-is-no-publish guarantees.

Deterministic replay artifacts for exercised behavior are not yet attached because the work remains specification-first at this stage.
This handoff should therefore be treated as opening a shared seam dependency, not closing one.

## Open Questions For OxFml
1. Which accepted-result payload components should remain canonical seam obligations versus OxCalc-local interpretation rules?
2. What is the preferred canonical reject taxonomy boundary between evaluator-side and coordinator-side classes?
3. What trace schema changes, if any, are needed so reject and fence outcomes remain replay-sufficient across repos?
4. Which runtime-derived effects should be first-class shared seam concepts versus local derivations?

## Requested Next Step
Please review this packet against the current canonical OxFml seam docs and determine:
1. which clauses should be promoted directly into canonical seam text,
2. which clauses need adaptation,
3. which remain deferred pending exercised evidence.

## Status
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - OxFml-side acknowledgment and canonical integration
  - replay/evidence attachment once exercised behavior exists
  - possible follow-on handoff for more detailed runtime-derived reporting requirements
