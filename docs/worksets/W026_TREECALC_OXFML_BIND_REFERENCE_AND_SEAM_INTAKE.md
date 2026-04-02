# W026: TreeCalc OxFml Bind, Reference, and Seam Intake

## Purpose
Lock and consume the first real OxFml bind/reference package required for the TreeCalc-ready engine so OxCalc stops depending on proving-lane-only reference semantics.
This packet now operates beneath the landed `OxCalcTree` host-facing consumer contract and the landed OxFml V1 runtime/replay consumer surface; it must fold those cleaner entry surfaces into the TreeCalc seam lane without letting packaging imply broader seam closure than the evidence supports.

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
6. explicit placement of the consumed-now TreeCalc seam facts beneath the `OxCalcTreeEnvironment` / `OxCalcTreeDocument` / `OxCalcTreeRecalcRequest` / `OxCalcTreeRecalcResult` / `OxCalcTreeRuntimeFacade` contract so hosts do not need to reach into proving-floor engine types to understand W026 truth
7. explicit incorporation of the current runtime-derived effect family split where the local engine already distinguishes dynamic-dependency versus execution-restriction facts

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

## Execution Packet Additions
### Environment Preconditions
- repo-local docs and planning surfaces are readable and writable
- current OxFml downstream note is available at `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md`
- no code execution is required for note-only or packet-only narrowing slices
- if a later slice widens executable seam scaffolding, use existing `oxcalc-core` upstream-host tests and fixture corpus as the first validation floor

### Evidence Layout
- canonical artifact root:
  - `docs/upstream/NOTES_FOR_OXFML.md`
  - `docs/spec/core-engine/CORE_ENGINE_OXFML_SEAM.md`
  - `docs/spec/core-engine/CORE_ENGINE_TREECALC_OXFML_SEAM_NEGOTIATION_MATRIX.md`
  - `docs/worksets/W026_TREECALC_OXFML_BIND_REFERENCE_AND_SEAM_INTAKE.md`
- checked-in or ephemeral:
  - checked-in for note and seam-packet narrowing artifacts
  - later executable seam-intake evidence may be checked-in or ephemeral depending on the consuming implementation slice
- baseline run naming:
  - none required for note-only residual passes
  - if executable seam-intake evidence is added later, use `w026-*` naming under the relevant run root

### Replay-Corpus Readiness
- required replay classes with scenario ids:
  - none for the current note-only residual pass
- reserve or later replay classes:
  - TreeCalc bind/reference replay classes after executable seam intake
  - later candidate/publication and execution-restriction replay families if W026 widens beyond note-level packetization

### Coupled Widening Rule
- engine surfaces widened in this packet:
  - consumed-seam assumptions
  - the host-facing `OxCalcTree` contract truth that depends on those assumptions
  - bounded executable seam-intake evidence where the current TreeCalc local runtime or upstream-host scaffolding already exercises the consumed-now packet
- oracle/conformance surfaces widened in the same slice:
  - targeted fixture or emitted-artifact checks where W026 claims a carried family is now explicit rather than implicit
- widened comparison artifact:
  - use `w026-*` naming if new emitted evidence is added to prove the narrowed packet

## Sequences
### Sequence 1: Caller-Anchor And Address-Mode Packet
Primary work areas:
- first TreeCalc relative-reference subset
- caller-context and address-mode carriage
- rebind-versus-recalc consequences for caller-sensitive forms

Entry gate:
- current host/runtime packet is accepted as sufficient for first-slice planning
- current note lane already treats caller-anchor and address-mode as the main remaining relative-reference residual

Execution objective:
- record the consumed-now caller-context packet for the first TreeCalc relative-reference subset and narrow the remaining non-assumptions to the smallest practical set.

Exit condition:
1. `caller_anchor`, formula-channel, address-mode, and structure-context identity are named as the consumed-now first packet for the first TreeCalc subset
2. rebind-forcing versus recalc-only edit pressure is stated explicitly for caller-sensitive forms
3. the next OxFml reply needed on this topic is bounded and mismatch-driven rather than exploratory

### Sequence 2: Execution-Restriction Transport Packet
Primary work areas:
- execution-restriction observations
- candidate/reject transport shape
- capability-sensitive versus restriction-sensitive distinction

Entry gate:
- Sequence 1 packet exists locally

Execution objective:
- define the semantic minimum OxCalc consumes now for execution-restriction truth and the transport assumptions OxCalc must not make yet.

Exit condition:
1. the consumed-now carrier families for execution restriction are named explicitly
2. the non-assumptions around merged carriers, scheduler absorption, and publication-critical breadth are explicit
3. the handoff trigger for this topic is narrowed to live insufficiency rather than note-level uncertainty

### Sequence 3: Publication And Topology Breadth Packet
Primary work areas:
- candidate consequence breadth
- publish-critical versus replay-only carried families
- topology-sensitive consequence widening rules

Entry gate:
- Sequence 2 packet exists locally

Execution objective:
- define the first TreeCalc publication/topology breadth floor without over-claiming the whole broader consequence universe.

Exit condition:
1. `value_delta`, `shape_delta`, and `topology_delta` are explicitly preserved as distinct consumed categories for the first TreeCalc coordinator path
2. optional `format_delta` and `display_delta` handling is explicit and non-collapsed
3. the remaining publication/topology breadth residual is narrow enough to stay note-level unless live evidence later shows insufficiency

## Explicit W026 Closure Work List
This is the full W026 closure list. No scope growth is expected beyond these items; if every item below is closed with evidence, W026 should not need to revisit the same seam area again.

### Group A: Sequence 1 Caller-Context And Reference Intake
1. `Document update` Define the first closed TreeCalc reference subset. Name the exact direct-reference, relative-reference, unresolved-reference, host-sensitive, and dynamic-potential families that W026 closes now, and state explicitly which broader TreeCalc reference families remain outside this packet rather than being implicitly deferred.
2. `Document update + code review and update` Lock the per-formula identity and compatibility packet for that subset. Record the required floor for `formula_stable_id`, `formula_token`, bind identity or compatibility handle, structure-context identity, and artifact-token or compatibility basis, then verify the current TreeCalc and upstream-host intake paths carry those values honestly and patch them if they do not.
3. `Document update + code review and update` Lock the carried caller-context packet for the first relative-reference subset. Record `caller_anchor`, formula-channel context, address-mode context, and structure-context identity as the first-class carried inputs where meaning depends on them, then verify and update the local intake code so that those inputs are preserved explicitly rather than inferred or silently dropped.
4. `Document update + code review and update` Close the rebind-versus-recalc rule for caller-sensitive forms. State exactly which structural edit families force rebind and which remain recalc-only when caller-context matters, then verify the current structural-edit and TreeCalc local execution path matches that rule and patch any mismatch.
5. `Document update + code review and update` Close the dependency-descriptor mapping for the Sequence 1 reference families. Record the mapping from OxFml bind/reference products into OxCalc dependency descriptors for direct, unresolved, host-sensitive, and dynamic-potential carriers, then verify the current mapping in the TreeCalc local intake and patch it so W027 can rely on it without rereading seam notes.

### Group B: Sequence 2 Candidate, Reject, And Runtime-Derived Transport
6. `Document update` Lock the candidate/reject/correlation packet. Record the exact consumed-now correlation and consequence identifiers for W026 scope, including `candidate_result_id`, `commit_attempt_id` where present, `reject_record_id` where present, optional `fence_snapshot_ref`, and the first-phase typed reject families that must stay reachable rather than being collapsed into generic failure.
7. `Document update + code review and update` Close the distinction between canonical OxFml object families and local OxCalc projection labels. For each current reject and runtime-facing family, state whether OxCalc is consuming a canonical object family or projecting a local label, then review the current code and emitted artifacts so that naming and interpretation are consistent and do not blur ownership.
8. `Document update + code review and update` Lock the semantic minimum runtime-derived packet. Record the exact dynamic-dependency, execution-restriction, and capability-sensitive facts OxCalc consumes now, including the non-assumption that OxFml has not frozen one final merged carrier, then verify and patch the local TreeCalc runtime and result surfaces so those families are structurally distinguishable where W026 claims they are explicit.
9. `Document update + code review and update` Close the runtime-effect and overlay reachability rule. Record which runtime-derived families must remain reachable from `OxCalcTreeRecalcResult` and which may stay below the host-facing contract, then review and patch result serialization, explain output, and overlay projection so the declared families are actually observable without drilling into unrelated internals.
10. `Document update + code review and update` Close the W026-to-W029 boundary. State exactly where W026 ends for runtime-derived and execution-restriction transport truth, and where W029 begins for broader runtime-derived realization, then patch any docs or code comments that currently blur that line.

### Group C: Sequence 3 Publication And Topology Consequence Breadth
11. `Document update` Lock the first TreeCalc publication-consequence packet. Record `value_delta`, `shape_delta`, `topology_delta`, optional `format_delta`, and optional `display_delta`, and state the exact absence/presence semantics so the first TreeCalc coordinator path does not silently collapse optional consequence families.
12. `Document update + code review and update` Close the publish-critical versus replay-only distinction. State which consequence families are publish-critical now, which are replay-visible but not publish-critical yet, and which are only local-floor evidence today, then review the current coordinator/result/explain surfaces and patch any place where those categories are mixed or implied rather than explicit.
13. `Document update + code review and update` Close the execution-restriction versus publication/topology interaction rule. State whether any current execution-restriction observations must already be treated as publication-sensitive or topology-sensitive consequences rather than only runtime-effect sidecars, then verify the local path and patch it if the declared rule is not represented honestly.

### Group D: Host-Facing Contract And Canonical Packet Sync
14. `Document update` Close the `OxCalcTree` contract reachability wording for W026. Record exactly which W026 seam facts must remain directly reachable from `OxCalcTreeRecalcResult`, which may remain below the host-facing contract for now, and explicitly state that W026 does not authorize a second host-facing OxCalc seam layer.
15. `Document update` Synchronize the canonical packet text across the W026 authority set. Update this workset, `CORE_ENGINE_OXFML_SEAM.md`, the TreeCalc seam-negotiation companion, the TreeCalc semantic-completion companion, and the active upstream note record so the same Sequence 1/2/3 packet truth appears everywhere with no drift in consumed-now versus `canonical but narrower` wording.

### Group E: Evidence, Handoff Decision, And W026 Closure
16. `Code review and update` Attach deterministic exercised evidence for every family W026 claims is explicit. Review the current upstream-host scaffolding, TreeCalc fixtures, and emitted replay/explain artifacts, add or patch cases where needed, and ensure each consumed-now family in Sequences 1, 2, and 3 is actually exercised rather than only named in prose.
17. `Document update + code review and update` Make replay and explain visibility explicit for the carried families. Verify that caller-context-sensitive behavior, execution-restriction/runtime-derived families, and publication/topology consequence categories are visible in emitted artifacts where W026 claims they are explicit, and patch either the artifacts or the W026 claim until they match exactly.
18. `Document update` Make the narrower handoff decision final for W026. Record either that no narrower `HANDOFF-CALC-002` is required because the exercised packet is sufficient, or record the exact insufficiency and the exact missing family that forces the handoff; this item does not close by leaving an unspecified future decision.
19. `Document update` Perform the final W026 closure audit. Confirm that all three sequences have been converted from note-level residual summaries into one executed consumed-seam intake packet, that no hidden formula-language reinterpretation obligation remains on OxCalc for the declared first TreeCalc subset, and that W027, W028, and W029 can proceed without relying on seam assumptions that exist only in prose.

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
  - a widened minimal upstream host packet and adapter now exist for automated scaffolding and the first seam-backed TreeCalc direct-host slice, including typed host-info outcome variants, RTD stand-ins, in-memory runtime catalog snapshots, first replay-capture packet projection, explicit crate-level scaffolding tests, and a checked-in upstream-host fixture corpus that now also covers the agreed first table-context packet plus a bounded structured-reference evaluation subset, but the broader consumed bind/reference intake is still open
  - TreeCalc-specific relative-reference and unresolved-reference carrier rules remain canonical but narrower beyond the first consumed subset, but Sequence 1 now has an explicit consumed-now carrier floor and no current handoff trigger
  - runtime-derived effect and execution-restriction transport shape remain canonical but narrower beyond the current semantic minimum, but Sequence 2 now has an explicit consumed-now carrier floor and no current handoff trigger
  - host/runtime direct-host versus integrated-host split is now sufficient for first implementation planning, and caller-anchor/address-mode carriage for the first TreeCalc relative-reference subset remains a note-level residual under Sequence 1
  - the sharpened `W052` registered-external lane is now settled enough for first TreeCalc-facing planning on the current direct packet names, seven-field descriptor floor, current-phase OxFml-owned funnel-family ownership, and bind-visible-snapshot-versus-targeted-reevaluation split, but broader packet promotion and later coordinator acknowledgment consequences remain deferred
  - publication/topology consequence breadth beyond the current exercised local floor remains a bounded consumed-seam residual for the first TreeCalc coordinator path, but Sequence 3 now has an explicit consumed-now category floor and no current handoff trigger
  - the fixture-host stand-in packet is now converged enough for deterministic scaffolding, with explicit packet identity, structure-context identity, and formula-slot identity accepted as first refinements, while broader production coordinator-API freeze remains out of scope
  - the consume-now topics identified by the latest OxFml reply are only partially packetized into executed seam intake work, and W026 is now split into Sequence 1 caller-context narrowing, Sequence 2 execution-restriction transport narrowing, and Sequence 3 publication/topology breadth narrowing
  - narrower handoff need remains deferred pending live W026 evidence
- claim_confidence: draft
- reviewed_inbound_observations: W020 remains the current downstream seam baseline; the latest OxFml topic-matrix and narrower W026-focused replies make formula/bind identity, candidate consequence, reject-context, and direct-binding preservation consume-now topics, confirm that W026 may proceed on a narrowed first subset for relative-reference, unresolved-reference, runtime-derived transport, and semantic-format/display handling without a new handoff, the latest host/runtime reply agrees that `OXFML_HOST_RUNTIME_AND_EXTERNAL_REQUIREMENTS.md` is strong enough for the first coordinator-host implementation slice while keeping caller-anchor/address-mode, execution-restriction transport breadth, and publication/topology breadth as note-level residuals, the latest `W051`/`W052` stand-in refinement confirms deterministic fixture-host inputs plus optional registered-external presence without freezing the production coordinator API or shifting catalog ownership into OxCalc, the latest sharpened `W052` reply aligns on the direct adopted packet names, seven-field descriptor, current-phase OxFml-owned funnel-family ownership, and the bind-visible-snapshot-versus-targeted-reevaluation split as sufficient for first TreeCalc-facing planning, and the latest OxFml residual reply confirms that W026 Sequence 1, Sequence 2, and Sequence 3 all remain `canonical but narrower` with explicit consumed-now carriers and no current narrower handoff trigger
