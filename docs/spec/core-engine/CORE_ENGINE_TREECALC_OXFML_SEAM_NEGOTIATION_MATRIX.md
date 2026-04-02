# CORE_ENGINE_TREECALC_OXFML_SEAM_NEGOTIATION_MATRIX.md

## 1. Purpose and Status

**This document is NOT canonical seam authority.**

Classification: **temporary-planning** per `CORE_ENGINE_DOWNSTREAM_HOST_SEAM_REFERENCE.md` Section 4.1.

This document turns the TreeCalc-facing OxCalc↔OxFml seam into a negotiation-ready matrix for the next `NOTES_FOR_*` passes.

Status:
1. active planning and intake companion,
2. intended bridge between the canonical local seam doc and W026,
3. note-exchange oriented with the first implementation-backed intake now underway,
4. explicitly pre-handoff unless a narrower trigger is reached,
5. will be superseded or retired when the topics it tracks are consumed into executed seam intake work.

This document exists so the next seam passes are structured around concrete consumed-carrier questions, not broad prose uncertainty.
It is not the canonical local seam-reference source of truth for downstream hosts.
Downstream hosts such as `DNA OneCalc` must read `CORE_ENGINE_DOWNSTREAM_HOST_SEAM_REFERENCE.md` first (the single entry point), then `CORE_ENGINE_OXFML_SEAM.md`, and only then use this matrix for narrower open topics and explicit non-assumptions.
Actual runtime consumers such as `DNA TreeCalc` should read `CORE_ENGINE_OXCALCTREE_CONSUMER_INTERFACE_AND_HOST_CONTRACT_V1.md` first and treat this matrix only as the narrower residual tracker beneath that host-facing contract.

## 2. Working Rule
For the first TreeCalc-ready engine phase:
1. OxFml remains authoritative for formula, bind, evaluator, reject, and replay-safe identity meaning,
2. OxCalc remains authoritative for coordinator consequences, invalidation integration, publication semantics, and replay meaning on the engine side,
3. note exchange is the default negotiation mechanism,
4. a narrower handoff is filed only when a coordinator-facing consumed clause cannot be stabilized through note exchange alone.

## 3. Required Reply Shape For Each Topic
Each active seam topic should be answered in the next `NOTES_FOR_*` rounds with:
1. current OxFml classification:
   - `already canonical`
   - `canonical but narrower`
   - `still open`
2. carrier surface OxCalc should consume now,
3. explicit non-assumptions OxCalc must preserve,
4. whether the topic is sufficient for W026 consumption now,
5. whether the topic is only note-level or now deserves a narrower handoff.

## 4. Topic Matrix

### 4.1 Formula and Bind Identity
OxCalc consumed need:
1. `formula_stable_id`
2. `formula_token`
3. `bind_hash`
4. `snapshot_epoch`
5. `profile_version`
6. important consumed compatibility state for `capability_view_key`

Why it matters:
1. formula-bearing nodes need immutable attachment and compatibility identity,
2. rebind versus recalc cannot be decided honestly without this split,
3. replay and witness surfaces need stable identity that is not only local node naming.

Expected current state:
1. mostly consumable now,
2. `capability_view_key` remains canonical but narrower in some clauses.

W026 output needed:
1. consumed-now node attachment fields,
2. replay-visible versus compatibility-only distinction,
3. handoff trigger only if live TreeCalc use reveals a missing coordinator-facing clause.

### 4.2 Direct and Relative Reference Descriptors
OxCalc consumed need:
1. direct named-node reference descriptors,
2. tree-relative reference descriptors or already-bound relative targets,
3. unresolved-reference and host-sensitive-reference carriers,
4. rule for what structural context anchors relative meaning.

Why it matters:
1. TreeCalc semantics are not only direct absolute references,
2. structural edits must map to rebind or recalc deterministically,
3. relative meaning cannot be left as hidden evaluator context if OxCalc must manage invalidation.

Expected current state:
1. likely narrower than the already-canonical identity floor,
2. likely a prime W026 note topic before implementation starts.

W026 output needed:
1. first in-scope relative-reference subset,
2. bind-time-fixed versus context-sensitive decision,
3. explicit list of edits that force rebind.

### 4.3 Dependency Fact Carriage
OxCalc consumed need:
1. static dependency facts,
2. runtime additions,
3. runtime removals,
4. runtime reclassifications,
5. stable dependency fact identity for replay and witness use.

Why it matters:
1. OxCalc cannot build a real graph from prose-only dependency meaning,
2. runtime dependency changes must not become hidden mutable state,
3. retained/reduced witnesses must preserve enough identity to stay diagnostic.

Expected current state:
1. semantic intent stable enough to consume now,
2. retained/reduced witness projection closure still narrower.

W026 output needed:
1. consumed-now dependency fact floor for live graph build,
2. explicit deferred closure for broader retained/reduced transport rules.

### 4.4 Candidate Consequence Carriage
OxCalc consumed need:
1. stable correlation ids
2. `value_delta`
3. `shape_delta`
4. `topology_delta`
5. optional `format_delta`
6. optional `display_delta`
7. spill or shape events
8. surfaced evaluator/runtime facts required for coordinator behavior

Why it matters:
1. candidate result is not publication,
2. coordinator-controlled publication requires explicit consequence shape,
3. verified-clean versus publish-ready requires a non-collapsed equality surface.

Expected current state:
1. canonical category split already stable enough to consume,
2. optionality breadth may still need narrower note confirmation for some families.

W026 output needed:
1. consumed-now first TreeCalc candidate package,
2. publish-critical versus replay-only carried fields,
3. exact verified-clean comparison surface for the first TreeCalc phase.

### 4.5 Reject Context Carriage
OxCalc consumed need:
1. typed reject context families for mismatch, capability, phase, execution restriction, dynamic dependency, and host-sensitive failure,
2. stable correlation ids where present,
3. enough detail to preserve no-publish reasoning without coordinator reinterpretation.

Why it matters:
1. reject must remain typed and replay-visible,
2. TreeCalc bind and host-sensitive families will widen failure shapes,
3. coordinator policy must not invent evaluator meaning after the fact.

Expected current state:
1. important canonical context families already stable,
2. local projection labels may remain local-only in some cases.

W026 output needed:
1. consumed canonical reject context subset,
2. list of purely local OxCalc projection labels,
3. explicit handoff trigger only if a required reject family is missing.

### 4.6 Runtime-Derived Effects and Execution Restrictions
OxCalc consumed need:
1. dynamic dependency activation and release,
2. capability observations,
3. execution-restriction observations,
4. shape or topology runtime effects,
5. format-sensitive runtime observations where semantically relevant.

Why it matters:
1. runtime-derived effects must become explicit engine state,
2. execution restriction is one of the few still-likely narrower handoff triggers,
3. overlay closure depends on this being real rather than hidden evaluator state.

Expected current state:
1. semantic consumption is stable enough now,
2. final transport-carrier closure is still narrower.

W026 output needed:
1. semantic minimum OxCalc consumes now,
2. transport-shape assumptions OxCalc must avoid,
3. explicit residual criteria for any later handoff.

### 4.7 Direct-Binding and Host-Sensitive Truth
OxCalc consumed need:
1. explicit distinction between direct-binding-sensitive and name-only families,
2. preserved concrete identity where semantic correctness depends on it,
3. replay and retained-witness preservation rules for those identities.

Why it matters:
1. TreeCalc witness and pack lanes must not erase real semantic identity,
2. host-sensitive truth is already known to be canonical on the OxFml side,
3. broader program-grade pack work will keep stressing this area.

Expected current state:
1. semantic ownership is already clear,
2. broader naming/indexing conventions remain open.

W026 output needed:
1. consumed direct-binding-sensitive floor for TreeCalc engine work,
2. explicit note-only residuals for later broader pack-family naming.

### 4.8 Semantic, Format, and Display Boundary
OxCalc consumed need:
1. a semantic consequence floor for the first TreeCalc-ready engine,
2. explicit format-sensitive carriage where runtime or later observer policy depends on it,
3. enough display-facing visibility that OxCalc does not accidentally collapse the categories.

Why it matters:
1. later Excel-compatible widening will care about this boundary,
2. current TreeCalc work should not overcommit display semantics too early,
3. replay honesty depends on not flattening the categories.

Expected current state:
1. canonical category split exists,
2. shared interpretation remains narrower.

W026 output needed:
1. consumed-now semantic and format floor,
2. explicit deferred display-facing questions,
3. note-only residual until live TreeCalc evidence says otherwise.

### 4.9 Host Runtime and External Requirements
OxCalc consumed need:
1. a clear direct-host versus OxCalc-integrated host split,
2. explicit required inputs for formula, structure, direct bindings, defined names, host-query/provider families, runtime library-context snapshots, and capability/fence inputs,
3. explicit required output families for candidate, commit, reject, trace, and `ReturnedValueSurface`,
4. stable coordinator-relevant ids and consequence categories preserved without host-side reinterpretation.

Why it matters:
1. TreeCalc intake should not proceed on an implicit host contract,
2. runtime library-context truth is now explicit OxFml/OxFunc-owned seam surface rather than a local convenience,
3. the first coordinator-host implementation slice needs a bounded contract that is narrower than full product-host closure but stronger than proving-host-only prose.

Expected current state:
1. sufficient now for the first host/coordinator implementation slice,
2. caller-anchor/address-mode handling for the first TreeCalc relative-reference subset remains narrower,
3. provider-failure and callable-publication remain watch lanes only.
4. OxFml has now explicitly agreed with OxCalc's `already canonical` read for this first slice.

W026 output needed:
1. consumed-now host/runtime baseline for the first integrated host slice,
2. explicit residual note-level topics that remain narrower,
3. no handoff trigger unless live host evidence reveals a missing coordinator-facing clause.

## 5. Negotiation Sequence
Recommended note sequence for W026 preparation:
1. Round A: identity, bind, direct and relative reference descriptors
2. Round B: dependency facts, candidate consequences, reject contexts
3. Round C: runtime-derived effects, execution restrictions, direct-binding preservation, semantic-format-display boundary
4. Round D: only if needed, a narrower handoff on the one remaining coordinator-facing insufficiency

### 5.1 Current Residual Packetization
After the broader topic-matrix rounds, the active residual packetization for W026 is now:
1. Sequence 1: caller-anchor and address-mode carriage for the first TreeCalc relative-reference subset
2. Sequence 2: execution-restriction transport breadth
3. Sequence 3: publication/topology consequence breadth

Working rule:
1. each residual sequence should narrow consumed-now assumptions and explicit non-assumptions separately,
2. each sequence should be able to stay note-level unless live implementation pressure shows a concrete insufficiency,
3. the residual packet should not reopen already-consumed identity, candidate, reject-context, direct-binding, stand-in, or table-context topics.

## 6. Current Topic-Matrix Intake From OxFml
The latest OxFml reply materially narrows the W026 starting state.

Current local intake is:
1. formula and bind artifact identity carriage: `already canonical`
2. direct and relative reference descriptor carriage: `canonical but narrower`
3. unresolved and host-sensitive reference carrier rules: `canonical but narrower`
4. dependency fact carriage: semantically `already canonical`, with narrower retained/reduced projection closure
5. candidate-result consequence optionality and correlation guarantees: `already canonical`
6. reject-context carrier and diagnostic guarantees: `already canonical` for the current typed families
7. runtime-derived effect and execution-restriction transport: `canonical but narrower`
8. direct-binding-sensitive witness preservation rules: `already canonical`
9. semantic-format versus display-facing consequence boundary: `canonical but narrower`

The practical consequence for W026 is:
1. identity, candidate consequence, reject-context, and direct-binding preservation should now be treated as consume-now topics,
2. relative-reference descriptor carriage, unresolved or host-sensitive reference carriers, runtime-derived effect transport shape, and semantic-format-display reading remain the main note-level refinement topics,
3. the OxFml host/runtime draft is also sufficient to consume now for the first integrated host slice,
4. no new narrower handoff is justified yet from this note round alone.

The latest narrower W026-focused reply also supplies the first practical carrier guidance for the remaining four topics:
1. relative-reference carriage is sufficient now for a narrowed first subset using current normalized reference-expression and bound-reference artifacts where contextual dependence is preserved honestly,
2. unresolved and host-sensitive carriers are sufficient now if OxCalc preserves the current accepted-unresolved versus reject distinction plus typed unresolved/bind diagnostics and host-query capability-view surfaces,
3. runtime-derived effect transport is sufficient now semantically through current surfaced evaluator facts and topology/effect refs, while final carrier closure remains open,
4. semantic-format versus display-facing consequence handling is sufficient now for a semantics-first first phase so long as `format_delta` and `display_delta` remain explicitly distinct and broader display closure is not over-claimed.

This means W026 is now blocked only by live consumption work, not by broad seam uncertainty.

### 6.1 Current Intake Of The Three-Sequence Residual Reply
OxFml has now also answered the later three-sequence residual round.

Current local intake is:
1. Sequence 1 caller-anchor and address-mode carriage: `canonical but narrower`
2. Sequence 2 execution-restriction transport breadth: `canonical but narrower`
3. Sequence 3 publication/topology consequence breadth: `canonical but narrower`

Current practical consequence:
1. all three residual sequences may remain note-level under W026,
2. all three now have explicit consumed-now carriers rather than only abstract residual labels,
3. no narrower handoff is justified unless live TreeCalc evidence later exposes a concrete insufficiency in one of those carried families.

### 6.2 Current Executed Intake Floor
The matrix is no longer purely prospective.

Current executed intake on the OxCalc side is:
1. the minimal upstream-host runtime/replay path is now migrated onto the landed OxFml V1 public consumer surface:
   - `consumer::runtime`
   - `consumer::replay`
2. this means the host/runtime baseline in Topic `4.9` is now consumed in live OxCalc code for the deterministic scaffolding packet,
3. this does not yet close the narrower TreeCalc bind/reference intake topics that still rely on direct consumed OxFml parse/bind products in the current local TreeCalc dependency-preparation slice,
4. this therefore narrows the live open lane to TreeCalc-specific bind/reference and transport-breadth questions rather than to the ordinary runtime/replay entry surface.

### 6.3 Current Executed W026 Packet Floor
W026 is now no longer just a residual topic ledger. The first executed packet floor is:
1. Sequence 1 caller-context and reference intake:
   - closed first subset: `DirectNode`, admitted `RelativePath` (`ParentNode` and `Ancestor(n >= 1)` descendant lookup), `SiblingOffset`, `Unresolved`, `HostSensitive`, `DynamicPotential`
   - explicit per-formula and caller-context packet:
     - `formula_stable_id`
     - `formula_token`
     - optional `bind_artifact_id`
     - `structure_context_version`
     - `caller_anchor`
     - `formula_channel_kind`
     - `address_mode`
   - rebind-versus-recalc and dependency-descriptor mapping are now executed for that subset
2. Sequence 2 execution-restriction and runtime-derived transport:
   - current explicit correlation floor:
     - `candidate_result_id`
     - `publication_id`
   - explicit current absences:
     - `commit_attempt_id`
     - `reject_record_id`
     - `fence_snapshot_ref`
   - current emitted runtime-derived families:
     - `DynamicDependency`
     - `ExecutionRestriction`
   - `CapabilitySensitive` remains admitted but unexercised
   - current family reachability is now explicit on `OxCalcTreeRecalcResult`, `result.json`, and `explain.json`
3. Sequence 3 publication/topology breadth:
   - `value_delta` is the only currently published consequence family
   - `shape_delta`, `topology_delta`, optional `format_delta`, and optional `display_delta` remain explicit current absences
   - current carriage split is explicit:
     - publish-critical: `value_delta`
     - replay-visible but not publish-critical: `published_runtime_effects`, `trace_markers`
     - local-floor-only evidence: `dependency_shape_updates`
   - current execution-restriction observations are runtime-effect-plus-typed-no-publish context, not publication-sensitive or topology-sensitive consequence families

Current non-overclaim remains:
1. Sequence 1, Sequence 2, and Sequence 3 all remain `canonical but narrower` beyond this executed first floor,
2. this planning companion should now treat broader residual breadth as later evidence-driven widening rather than as ambiguity about the current consumed-now packet.

## 7. Exit Condition For The Planning Phase
This planning companion has served its purpose when:
1. W026 has a consumed-now topic ledger for all in-scope TreeCalc seam topics,
2. any remaining uncertainty is explicitly classified as note-only residual or narrower handoff,
3. no major TreeCalc engine implementation decision still depends on compressed seam assumptions.

## 8. Status
- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - relative-reference descriptor carriage remains canonical but narrower beyond the first explicitly consumed subset
  - unresolved and host-sensitive reference carriers remain canonical but narrower beyond the first explicitly named families
  - runtime-derived effect transport remains canonical but narrower beyond the current `DynamicDependency` / `ExecutionRestriction` emitted floor
  - publication/topology breadth remains canonical but narrower beyond the current `value_delta`-only published floor and explicit current absences
  - W026 has now consumed the ordinary runtime/replay entry surface plus the first executed three-sequence residual floor, but broader TreeCalc-specific bind/reference and transport-breadth widening still remains
  - no narrower handoff has been justified yet
- claim_confidence: provisional
- reviewed_inbound_observations: latest OxFml downstream note and returned classifications consumed as the starting baseline
