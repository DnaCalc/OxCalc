# W026: TreeCalc OxFml Bind, Reference, and Seam Intake

## Purpose
Lock and consume the first real OxFml bind/reference package required for the TreeCalc-ready engine so OxCalc stops depending on proving-lane-only reference semantics.

## Position and Dependencies
- **Depends on**: W020, W025
- **Blocks**: W027, W028, W029, W030, W031
- **Cross-repo**: may justify a narrower `HANDOFF-CALC-002` only if exercised TreeCalc bind intake reveals a still-unresolved consumed seam obligation

## Scope
### In scope
1. consumed identity and fence floor for formula-bearing TreeCalc nodes
2. consumed bind/reference package for direct, relative, unresolved, and host-sensitive TreeCalc reference families in first-phase scope
3. candidate-result, reject-context, and runtime-derived fact categories that the TreeCalc path needs from OxFml
4. explicit local resolution of what OxCalc consumes versus what OxFml continues to own semantically
5. explicit narrowing of any residual seam uncertainty before engine implementation proceeds

### Out of scope
1. full evaluator-backed execution
2. dependency graph realization in Rust
3. runtime-derived overlay realization in Rust
4. broader display or grid semantics outside the declared TreeCalc phase scope

## Deliverables
1. a narrowed consumed-seam packet for TreeCalc formula, bind, reference, candidate, reject, and runtime-derived inputs
2. explicit mapping from OxFml bind/reference products into OxCalc structural and dependency integration points
3. a decision on whether a narrower `HANDOFF-CALC-002` is required before W027/W028
4. updates to the canonical seam docs so later implementation work is not operating on compressed local summaries
5. at least one TreeCalc-focused note-exchange round using the topic-matrix shape rather than broad prose-only seam commentary
6. one bounded intake pass over `OXFML_HOST_RUNTIME_AND_EXTERNAL_REQUIREMENTS.md` so the first coordinator-host slice is not planned on implicit host/runtime assumptions

## Gate Model
### Entry gate
- W025 has defined the widened TreeCalc structural substrate and formula attachment points
- W020 remains the carried downstream seam-intake baseline
- the TreeCalc seam negotiation topics and reply shape are recorded in the local seam docs and outbound note

### Exit gate
- OxCalc can consume formula artifact identities, bind identities, and reference meaning for the first TreeCalc families
- unresolved seam items are explicitly narrowed and either deferred or packetized
- no hidden formula-language reinterpretation obligation remains on the OxCalc side for in-scope families
- the active TreeCalc seam topics have at least one completed note-exchange pass with explicit `already canonical` / `canonical but narrower` / `still open` classification

## Pre-Closure Verification Checklist
1. Spec text and realization notes updated for all in-scope items: no
2. Pack expectations updated for affected packs: no
3. At least one deterministic replay artifact exists per in-scope behavior: no
4. Semantic-equivalence statement provided for policy or strategy changes: no
5. FEC/F3E cross-repo impact assessed and handoff filed if needed: no
6. All required tests pass: no
7. No known semantic gaps remain in declared scope: no
8. Completion language audit passed: no
9. `IN_PROGRESS_FEATURE_WORKLIST.md` updated: no
10. `CURRENT_BLOCKERS.md` updated if needed: no

## Status
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - a widened minimal upstream host packet and adapter now exist for automated scaffolding and the first seam-backed TreeCalc direct-host slice, including typed host-info outcome variants, RTD stand-ins, in-memory runtime catalog snapshots, first replay-capture packet projection, explicit crate-level scaffolding tests, and a first checked-in upstream-host fixture corpus that now also covers the agreed first table-context packet plus one bounded evaluator-facing structured-reference family, but the broader consumed bind/reference intake is still open
  - TreeCalc-specific relative-reference and unresolved-reference carrier rules remain canonical but narrower beyond the first consumed subset
  - runtime-derived effect and execution-restriction transport shape remain canonical but narrower beyond the current semantic minimum
  - host/runtime direct-host versus integrated-host split is now sufficient for first implementation planning, but caller-anchor/address-mode carriage for the first TreeCalc relative-reference subset remains narrower
  - the consume-now topics identified by the latest OxFml reply are only partially packetized into executed seam intake work
  - narrower handoff need remains deferred pending live W026 evidence
- claim_confidence: draft
- reviewed_inbound_observations: W020 remains the current downstream seam baseline; the latest OxFml topic-matrix and narrower W026-focused replies make formula/bind identity, candidate consequence, reject-context, and direct-binding preservation consume-now topics, confirm that W026 may proceed on a narrowed first subset for relative-reference, unresolved-reference, runtime-derived transport, and semantic-format/display handling without a new handoff, and the latest host/runtime reply agrees that `OXFML_HOST_RUNTIME_AND_EXTERNAL_REQUIREMENTS.md` is strong enough for the first coordinator-host implementation slice while keeping caller-anchor/address-mode, execution-restriction transport breadth, and publication/topology breadth as note-level residuals
