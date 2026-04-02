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

### Sequence 1A: First Closed TreeCalc Reference Subset
The first closed TreeCalc reference subset for W026 is the subset already represented in the local TreeCalc reference model and exercised or intentionally surfaced in the current local intake floor.

Closed now for W026:
1. direct static node references through `TreeReference::DirectNode { target_node_id }`
2. relative descendant lookup through `TreeReference::RelativePath { base, path_segments }`, but only for:
   - `base = ParentNode`
   - `base = Ancestor(n)` where `n >= 1`
   - descendant lookup through explicit `path_segments`
3. sibling-relative lookup through `TreeReference::SiblingOffset { offset, tail_segments }`, including empty or explicit descendant `tail_segments`
4. explicit unresolved carrier intake through `TreeReference::Unresolved { token }`, treated as a first-class unresolved descriptor or bind-unresolved family rather than as silent fallback or local reinterpretation
5. explicit host-sensitive carrier intake through `TreeReference::HostSensitive { carrier_id, detail }`, treated as a first-class residual/runtime-derived family rather than as a locally reinterpreted ordinary reference
6. explicit dynamic-potential carrier intake through `TreeReference::DynamicPotential { carrier_id, detail }`, treated as a first-class residual/runtime-derived family rather than as a locally reinterpreted ordinary reference

Evidence for the current local floor:
1. `DirectNode` is exercised through the checked-in local TreeCalc publish, verified-clean, recalc-only, dependency-chain, move-target, and removal families
2. `RelativePath` is exercised through `tc_local_relative_sum_001` and `tc_local_rebind_after_rename_001`
3. `SiblingOffset` is exercised through `tc_local_sibling_offset_publish_001`
4. `HostSensitive` is exercised through `tc_local_host_sensitive_reject_001`, `tc_local_post_edit_host_sensitive_overlay_001`, and `tc_local_mixed_publish_then_post_edit_overlay_001`
5. `DynamicPotential` is exercised through `tc_local_dynamic_reject_001`
6. `Unresolved` is already explicit in the local lowering and dependency-diagnostic path, even though the checked-in TreeCalc fixture corpus does not yet carry a dedicated unresolved case

Explicitly outside this first closed subset:
1. `TreeReference::ProjectionPath { projection_path }` is representable in the local type system but is not part of the first W026 closed subset
2. `TreeReference::RelativePath` with `base = SelfNode` is representable in the local type system but is not part of the first W026 closed subset
3. any broader caller-sensitive, address-mode-sensitive, or projection-sensitive relative families beyond the exact forms listed above remain outside this first packet
4. any broader grid-, workbook-, or display-facing reference families remain outside W026 scope for the TreeCalc-first phase

Interpretation rule:
1. W026 now treats the exact forms above as the first closed TreeCalc reference subset
2. W026 does not treat the broader `TreeReference` enum as already closed by implication
3. later items in this workset may widen evidence or carrier reachability for this subset, but they do not reopen which reference families belong to the first closed subset unless live insufficiency forces an explicit narrower handoff

### Sequence 1B: Per-Formula Identity And Compatibility Packet
For the first closed TreeCalc reference subset, W026 now treats the following as the required per-formula identity and compatibility packet.

Required floor:
1. `formula_stable_id` as the immutable formula artifact identity
2. `formula_token` as the current bind/runtime correlation token for the specific formula text version being evaluated
3. optional `bind_artifact_id` as the current local bind-identity or compatibility handle where the TreeCalc local floor has one
4. `structure_context_version` as the carried structure-context identity for bind/runtime compatibility
5. `compatibility_basis` as the OxCalc-side run compatibility handle
6. `artifact_token_basis` as the OxCalc-side coordinator artifact-token basis

Current local carriage rule:
1. `formula_stable_id` and `formula_token` are explicit in the current TreeCalc `PreparedOxfmlFormula` intake and in the deterministic minimal upstream-host packet
2. `bind_artifact_id` is explicit in the TreeCalc formula binding and now explicit in the deterministic minimal upstream-host packet when present
3. `structure_context_version` is explicit in both bind-context construction and runtime-environment construction
4. `compatibility_basis` and `artifact_token_basis` remain explicit at the `OxCalcTreeRecalcRequest` and `LocalTreeCalcInput` boundary rather than being hidden ambient state

Interpretation rule:
1. this packet is the required identity and compatibility floor for the first W026 closed subset
2. W026 does not require a broader frozen shared naming claim such as final `bind_hash` transport across every current surface in order to close this first local intake item
3. later items may widen canonical cross-doc wording around bind identity, but they do not reopen whether the current local TreeCalc and upstream-host intake paths now carry the declared floor explicitly

### Sequence 1C: Caller-Context Packet For The First Closed Subset
For the first closed TreeCalc reference subset, W026 now treats the following as the explicit caller-context packet wherever relative or host-sensitive meaning depends on caller context.

Required caller-context floor:
1. `caller_anchor`
2. `formula_channel_kind`
3. `address_mode`
4. `structure_context_version`

Current local carriage rule:
1. the TreeCalc local seam-consumption lane carries `caller_anchor` explicitly through bind-context construction and through the deterministic upstream-host packet used for evaluation
2. `formula_channel_kind` is explicit in `FormulaSourceRecord` and in the deterministic upstream-host packet
3. `address_mode` is now explicit in the deterministic upstream-host packet as the first closed-subset value `A1`, rather than being inferred only from `formula_channel_kind`
4. `structure_context_version` is explicit in both bind-context construction and the deterministic upstream-host packet
5. for the current first subset, these fields are carried in the internal seam-consumption layer and deterministic host packet rather than in the broad `OxCalcTree` host-facing contract itself

Current first-subset admissibility rule:
1. `RelativePath` with `ParentNode` and `Ancestor(n >= 1)` bases is admitted only because the caller-context packet above remains explicit
2. `SiblingOffset` is admitted only because the caller-context packet above remains explicit
3. broader relative families remain outside the first packet rather than relying on implied caller-context behavior

Interpretation rule:
1. W026 now treats the caller-context floor above as explicit carried input, not evaluator-local ambient state
2. W026 does not treat this as broader closure of every caller-sensitive or address-mode-sensitive clause in the future TreeCalc universe
3. later items may widen replay visibility or contract reachability for this packet, but they do not reopen whether the first closed subset now has an explicit carried caller-context floor

### Sequence 1D: Rebind Versus Recalc Rule For Caller-Sensitive Forms
For the first closed TreeCalc reference subset, W026 now treats coarse structural edit pressure and per-formula invalidation reason as distinct layers.

Structural edit family rule:
1. `RenameNode`, `MoveNode`, `InsertNode`, and `ReplaceFormulaAttachment` produce structural rebind pressure in the current local floor through `StructuralEditImpact::RebindRequired`
2. `RemoveNode` produces structural removal pressure in the current local floor through `StructuralEditImpact::Removal` and is treated as rebind-forcing pressure for surviving formulas
3. `SetConstantValue` produces recalc-only pressure in the current local floor through `StructuralEditImpact::RecalcOnly`

Per-formula invalidation rule for the first closed subset:
1. `RelativePath` with admitted `ParentNode` and `Ancestor(n >= 1)` bases, `SiblingOffset`, `HostSensitive`, and `Unresolved` are the caller-sensitive or caller-context-dependent families for this item and carry `requires_rebind_on_structural_change = true` in the local reference and dependency-descriptor layer
2. those families become `StructuralRebindRequired` for a rerun when structural rebind pressure is present and at least one of the following is true:
   - the formula owner node is directly affected
   - the currently bound target node is directly affected
   - the caller context is affected through the owner parent or ancestor chain
3. `DirectNode` and `DynamicPotential` remain `StructuralRecalcOnly` for this item under structural rebind pressure unless the formula owner node itself is directly affected
4. when only recalc-only pressure is present, all formula families remain `StructuralRecalcOnly` for the rerun

Current local carriage rule:
1. the local reference model and dependency-descriptor lowering now carry the first closed-subset rebind classification explicitly through `TreeReference::requires_rebind_on_structural_change` and `DependencyDescriptor::requires_rebind_on_structural_change`
2. the local TreeCalc post-edit rerun path now derives explicit `InvalidationSeed` values from predecessor-plus-successor structural context together with `StructuralEditOutcome` facts and passes them into `LocalTreeCalcInput`, rather than defaulting every post-edit rerun to `StructuralRecalcOnly`
3. the current local sequential runtime still recomputes the full local formula set in topological order; this item closes invalidation-reason truth and carriage, not a narrower incremental scheduler claim

Evidence for the current local floor:
1. `formula.rs` now tests the first closed-subset rebind classification directly for `DirectNode`, admitted `RelativePath`, `SiblingOffset`, `HostSensitive`, `DynamicPotential`, and `Unresolved`
2. `treecalc.rs` now tests that a rename affecting an admitted relative reference yields `StructuralRebindRequired`, while a moved direct-reference target remains `StructuralRecalcOnly` at the per-formula invalidation layer
3. `tc_local_rebind_after_rename_001` remains the checked-in local fixture evidence that rename pressure forces a caller-sensitive rerun to rebind and then reject conservatively on the successor snapshot
4. `tc_local_move_direct_target_rebind_001` remains valid as edit-layer evidence that the structural edit family carried rebind pressure, while W026 now states explicitly that the per-formula invalidation reason for a surviving direct-only formula may still remain recalc-only

Interpretation rule:
1. W026 now treats structural edit family and per-formula invalidation reason as separate consumed-now facts and does not collapse them into one label
2. W026 now treats the local TreeCalc post-edit rerun artifacts as truthful for caller-sensitive rebind versus recalc distinction within the first closed subset
3. later items may widen publication or replay visibility for these facts, but they do not reopen the rule itself unless live insufficiency shows the declared caller-context impact test is too narrow

### Sequence 1E: Dependency-Descriptor Mapping For The First Closed Subset
For the first closed TreeCalc reference subset, W026 now treats the dependency-descriptor vocabulary as the stable OxCalc-owned consequence of OxFml bind/reference intake. The mapping below is the consumed-now floor that W027 may rely on.

Required mapping rule:
1. a translated direct or admitted relative reference that survives the current OxFml-backed TreeCalc intake remains an explicit translated reference binding and keeps its OxCalc reference-family identity when projected into a `DependencyDescriptor`
2. a translated reference that does not produce a bound target keeps its carrier family but carries `target_node_id = None`
3. host-sensitive and dynamic-potential residual carriers remain residual families when projected into dependency descriptors and are not locally reinterpreted as ordinary direct or relative references

Current first-subset mapping:
1. `DirectNode` maps to `DependencyDescriptorKind::StaticDirect` with `target_node_id = Some(target)` and `requires_rebind_on_structural_change = false`
2. admitted resolved `RelativePath` and `SiblingOffset` references map to `DependencyDescriptorKind::RelativeBound` with `target_node_id = Some(target)` and `requires_rebind_on_structural_change = true`
3. admitted relative or sibling references that miss on the current structural view still map to `DependencyDescriptorKind::RelativeBound`, but with `target_node_id = None`; they do not collapse into explicit `Unresolved`
4. explicit `TreeReference::Unresolved` maps to `DependencyDescriptorKind::Unresolved` with `target_node_id = None` and `requires_rebind_on_structural_change = true`
5. `TreeReference::HostSensitive` maps to `DependencyDescriptorKind::HostSensitive` with `target_node_id = None` and `requires_rebind_on_structural_change = true`
6. `TreeReference::DynamicPotential` maps to `DependencyDescriptorKind::DynamicPotential` with `target_node_id = None` and `requires_rebind_on_structural_change = false`

Current local carriage rule:
1. the local bare TreeCalc lowering path in `TreeFormulaCatalog::to_dependency_descriptors` and the OxFml-backed lowering path in `oxfml_dependency_descriptors` now preserve the same descriptor-kind and rebind semantics for the first closed subset
2. the OxFml-backed path now treats the translated reference binding list itself as the authoritative first-subset carrier for direct and admitted relative bindings, because the current OxFml bind may normalize or otherwise fail to preserve the synthetic TreeCalc reference names as a stable filtering surface in `normalized_references`
3. `DependencyGraph::build` then interprets `target_node_id = None` with `RelativeBound` or `Unresolved` as unresolved-reference diagnostics, while `HostSensitive` and `DynamicPotential` remain typed residual diagnostics instead of collapsing into generic unresolved state

Evidence for the current local floor:
1. `formula.rs` already exercises the local bare lowering for direct, relative, host-sensitive, and unresolved families
2. `treecalc.rs` now exercises a mixed-carrier OxFml-backed lowering case and asserts the exact mapping for direct, resolved relative, unresolved relative, explicit unresolved token, host-sensitive residual, and dynamic-potential residual carriers

Interpretation rule:
1. W026 now treats the descriptor mapping above as the stable consumed-now packet for Sequence 1 dependency truth
2. W026 does not require OxFml to publish a broader new shared descriptor taxonomy in order for OxCalc to close this item; the closure claim is only that OxCalc now consumes the current bind/reference products into its own descriptor vocabulary explicitly and consistently
3. later items may widen replay visibility or runtime consequences of these descriptors, but they do not reopen the mapping table itself unless live intake evidence shows a carrier family lands differently in code

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

### Sequence 2A: Candidate, Reject, And Correlation Packet
For the current TreeCalc-first runtime floor, W026 now treats the candidate/reject/correlation packet as a narrow local coordinator packet rather than as the full broader Stage 1 consequence universe.

Consumed-now correlation floor:
1. `candidate_result_id` is the primary stable correlation identifier and is explicit at the `OxCalcTreeRecalcRequest`, `AcceptedCandidateResult`, `PublicationBundle`, and `RejectDetail` surfaces
2. `publication_id` is explicit in the current TreeCalc request and publication consequence path, and remains the current publish-side consequence identifier for this lane
3. `structural_snapshot_id`, `compatibility_basis`, and `artifact_token_basis` remain explicit on `AcceptedCandidateResult` and are part of the current local candidate correlation packet

Consumed-now consequence reachability:
1. accepted work is reachable through `candidate_result: Option<AcceptedCandidateResult>` on `OxCalcTreeRecalcResult`
2. published work is reachable through `publication_bundle: Option<PublicationBundle>` on `OxCalcTreeRecalcResult`
3. rejected work is reachable through `reject_detail: Option<RejectDetail>` on `OxCalcTreeRecalcResult`
4. the current local replay/explain path also preserves `candidate_result_id`, `publication_id`, and typed `reject_detail.kind` in emitted TreeCalc artifacts

Current typed reject floor:
1. `RejectDetail.kind` is the required typed reject carrier for this lane and must not be collapsed into generic failure text
2. the current TreeCalc local execution path emits the following first-phase typed reject families:
   - `SyntheticCycleReject`
   - `DynamicDependencyFailure`
   - `HostInjectedFailure`
3. the broader coordinator-owned reject vocabulary already includes `SnapshotMismatch`, `ArtifactTokenMismatch`, `ProfileVersionMismatch`, `CapabilityMismatch`, and `PublicationFenceMismatch`, but W026 does not claim those families are already emitted by the current TreeCalc-first runtime lane

Explicit current absences:
1. `commit_attempt_id` is not a current TreeCalc-consumed or TreeCalc-emitted field
2. `reject_record_id` is not a current TreeCalc-consumed or TreeCalc-emitted field
3. `fence_snapshot_ref` is not a current TreeCalc-consumed or TreeCalc-emitted field
4. W026 therefore closes this item by recording those fields as explicit current absences rather than carrying them as implied present-soon packet members

Current local carriage rule:
1. `candidate_result_id` enters through `OxCalcTreeRecalcRequest` and flows into accepted, published, and rejected local coordinator consequences
2. `publication_id` enters through `OxCalcTreeRecalcRequest` and flows only into the publish-success consequence path
3. `reject_detail` currently carries `candidate_result_id`, `kind`, and freeform `detail`, with no separate stable reject record identity
4. the current local TreeCalc floor therefore supports candidate/publication/reject correlation, but not a broader commit-attempt or reject-record identity split

Evidence for the current local floor:
1. `consumer.rs`, `treecalc.rs`, and `coordinator.rs` expose the reachable `candidate_result`, `publication_bundle`, and `reject_detail` surfaces named above
2. `treecalc_runner.rs` emits `candidate_result_id`, `publication_id`, and typed `reject_detail.kind` into checked-in TreeCalc local artifacts
3. the checked-in TreeCalc fixtures already assert typed reject outcomes through `expected.reject_kind`

Interpretation rule:
1. W026 now treats this narrower packet as the stable consumed-now candidate/reject/correlation truth for the TreeCalc-first lane
2. W026 does not treat broader Stage 1 coordinator identifiers such as `commit_attempt_id`, `reject_record_id`, or `fence_snapshot_ref` as already consumed by this lane merely because they exist in broader OxCalc or OxFml planning
3. later items may widen runtime-derived and publication consequence breadth, but they do not reopen the current correlation floor unless the live TreeCalc lane gains new explicit identifiers

### Sequence 2B: Canonical OxFml Families Versus OxCalc Projection Labels
For the current TreeCalc-first runtime floor, W026 now treats canonical OxFml object families and OxCalc-local projection labels as explicitly distinct layers.

Canonical-family alignment now admitted:
1. OxFml-owned canonical artifact families in this area are `AcceptedCandidateResult`, `CommitBundle`, `RejectRecord`, and canonical OxFml `RejectCode`
2. OxCalc currently consumes those families only as alignment targets or ownership references for the TreeCalc-first lane; it does not yet expose field-complete canonical OxFml objects at the `OxCalcTree` boundary

Current OxCalc local projections:
1. `coordinator::AcceptedCandidateResult` is an OxCalc-local TreeCalc-first projection aligned to the canonical OxFml `AcceptedCandidateResult` family
2. `coordinator::PublicationBundle` is an OxCalc-local TreeCalc-first projection aligned to the canonical OxFml `CommitBundle` family
3. `coordinator::RejectDetail` is an OxCalc-local TreeCalc-first projection aligned to the canonical OxFml `RejectRecord` family
4. `coordinator::RejectKind` values such as `HostInjectedFailure`, `DynamicDependencyFailure`, and `SyntheticCycleReject` are OxCalc-local projection labels, not canonical OxFml `RejectCode` values
5. `RuntimeEffect`, `RuntimeEffectFamily`, and runtime-effect `kind` strings such as `runtime_effect.host_sensitive_reference` and `runtime_effect.dynamic_reference` are OxCalc-local projection labels over current local runtime facts, not canonical OxFml artifact families

Current local naming rule:
1. where TreeCalc artifacts surface a candidate or publication object, they now mark the aligned canonical family explicitly while preserving that the emitted object is still an OxCalc-local projection
2. where TreeCalc artifacts surface reject or runtime-effect `kind` labels, they now mark those labels as OxCalc-local projections rather than implied canonical OxFml codes or families
3. W026 therefore treats the local JSON labels and local Rust enum names as OxCalc-owned evidence labels unless a field is explicitly marked as aligned to a canonical OxFml family

Evidence for the current local floor:
1. `coordinator.rs`, `consumer.rs`, and `treecalc.rs` still carry the local TreeCalc-first projection structs and enums named above
2. `treecalc_runner.rs` now emits `aligned_canonical_family` and `projection_owner` markers for candidate/publication/reject artifacts plus `kind_owner` and `family_owner` markers for local runtime-effect and reject labels
3. `treecalc_runner.rs` now tests those emitted ownership markers against checked local run artifacts

Interpretation rule:
1. W026 now treats `AcceptedCandidateResult` and `CommitBundle` references in this lane as canonical-family alignment claims, not as proof that OxCalc is exposing the canonical OxFml field-complete objects unchanged
2. W026 now treats `RejectKind`, `RejectDetail.kind`, `RuntimeEffectFamily`, and runtime-effect `kind` strings as local OxCalc projection vocabulary unless and until a later seam packet promotes a canonical shared label set for those surfaces
3. later items may widen canonical reachability, but they do not reopen the ownership split itself unless the live code starts exposing additional OxFml-owned typed objects directly

### Sequence 2C: Semantic Minimum Runtime-Derived Packet
For the current TreeCalc-first runtime floor, W026 now treats the semantic minimum runtime-derived packet as a narrow set of explicit OxCalc-local runtime-effect families rather than as one frozen OxFml-wide merged carrier.

Consumed-now runtime-derived families:
1. `DynamicDependency` is explicit and currently represents local dynamic-reference-sensitive runtime facts in the TreeCalc-first lane
2. `ExecutionRestriction` is explicit and currently represents local host-sensitive or execution-restriction-sensitive runtime facts in the TreeCalc-first lane
3. `CapabilitySensitive` exists in the local `RuntimeEffectFamily` vocabulary as an admitted coordinator-facing family, but the current TreeCalc-first runtime lane does not yet emit a distinct capability-sensitive runtime effect

Current local fact mapping:
1. `runtime_effect.dynamic_reference` is the current OxCalc-local runtime-effect kind for the dynamic-dependency family and is emitted when a `DynamicPotential` carrier reaches the local runtime floor
2. `runtime_effect.host_sensitive_reference` is the current OxCalc-local runtime-effect kind for the execution-restriction family and is emitted when a `HostSensitive` carrier reaches the local runtime floor
3. no current TreeCalc local runtime path emits a distinct capability-sensitive runtime-effect kind; capability-sensitive meaning remains an explicit current absence rather than an implied merged member of another family

Non-assumption rule:
1. W026 does not assume OxFml has already frozen one final merged runtime-derived carrier for execution restriction, capability sensitivity, dependency sensitivity, and other surfaced evaluator facts
2. W026 therefore consumes the current TreeCalc-first local family split as an OxCalc-owned packet over the current lane, while leaving broader canonical OxFml carrier freezing to later seam work
3. W026 does not collapse execution-restriction and capability-sensitive meaning into the same claim merely because both families currently project to the same local overlay kind when present

Current local carriage rule:
1. `RuntimeEffect.family` is explicit on `OxCalcTreeRecalcResult`, `AcceptedCandidateResult`, `PublicationBundle.published_runtime_effects`, local trace events, and emitted TreeCalc result/explain/runtime-effect artifacts
2. `runtime_effect_overlay_kind` currently preserves `DynamicDependency` as `OverlayKind::DynamicDependency` and preserves `ExecutionRestriction` plus `CapabilitySensitive` as `OverlayKind::ExecutionRestriction`
3. W026 treats that overlay projection as sufficient for the current local floor because the source runtime-effect family remains explicit and replay-visible even where overlay kinds are narrower than runtime-effect families

Evidence for the current local floor:
1. `treecalc.rs` already proves that host-sensitive runtime effects carry `RuntimeEffectFamily::ExecutionRestriction` while dynamic-reference runtime effects carry `RuntimeEffectFamily::DynamicDependency`
2. `treecalc_runner.rs` now checks emitted runtime-effect artifacts for both the host-sensitive and dynamic-reference fixture cases and asserts the replay-visible family values plus explicit local ownership markers
3. there is no current checked-in TreeCalc fixture that emits a distinct `CapabilitySensitive` runtime effect, and W026 records that as an explicit current absence rather than a silent gap

Interpretation rule:
1. W026 now treats `DynamicDependency` and `ExecutionRestriction` as the only explicit emitted runtime-derived families in the current TreeCalc-first lane
2. W026 treats `CapabilitySensitive` as an admitted but currently unexercised local family, not as an emitted consumed-now fact
3. later items may widen runtime-derived breadth or align to a broader canonical OxFml carrier, but they do not reopen the current emitted family split unless the live TreeCalc lane starts surfacing a new distinct family

### Sequence 2D: Runtime-Effect And Overlay Reachability Rule
For the current TreeCalc-first runtime floor, W026 now locks the reachability rule for emitted runtime-derived families so hosts do not need to inspect unrelated local internals to understand the active runtime-effect packet.

Host-facing reachability floor:
1. `OxCalcTreeRecalcResult.runtime_effects` is a required direct carrier for all emitted runtime-derived families in the current TreeCalc-first lane
2. `OxCalcTreeRecalcResult.runtime_effect_overlays` is a required direct carrier for the overlay projection of those same emitted runtime-derived families
3. where a run reaches accepted-candidate or publication state, runtime-derived families also remain directly reachable through `AcceptedCandidateResult.runtime_effects` and `PublicationBundle.published_runtime_effects`, but hosts must not be forced to depend on those optional consequence objects just to observe the base runtime-derived packet

Replay-visible artifact floor:
1. `result.json` must expose dedicated `runtime_effects_path` and `runtime_effect_overlays_path` links so replay consumers can locate the canonical local sidecar files without searching through unrelated artifacts
2. `explain.json` must embed `runtime_effects` and `runtime_effect_overlays` directly so the current emitted runtime-derived families and their overlay projection remain visible in the first explanatory artifact without a second file hop
3. dedicated `runtime_effects.json` and `runtime_effect_overlays.json` remain the canonical local evidence files for this lane, and their presence in the artifact set does not reduce the host-facing or explain-facing direct reachability requirements above

Below-contract allowance rule:
1. residual-carrier translation details, local evaluator preparation state, and other intermediate seam-intake internals may remain below the host-facing consumer contract for this item
2. `CapabilitySensitive` and `ShapeTopology` may remain outside the current direct host-facing runtime-derived result floor until the live TreeCalc-first lane actually emits them as distinct runtime-derived families
3. local trace-event packaging remains supporting evidence for W026, not the required host-facing carrier for this item

Interpretation rule:
1. a TreeCalc-style host must be able to learn the emitted runtime-derived family split from `OxCalcTreeRecalcResult` directly, without drilling into `LocalTreeCalcRunArtifacts`, trace events, or other narrower local internals
2. overlay kinds remain narrower than runtime-derived families in the current lane, so overlay observation alone is insufficient; the source runtime-effect family must stay explicitly reachable beside the overlay projection
3. later seam widening may add more direct runtime-derived carriers, but it does not reopen this item unless the current direct reachability floor becomes false in live code

Evidence for the current local floor:
1. `consumer.rs` now proves the ordinary `OxCalcTreeRuntimeFacade` surfaces `ExecutionRestriction` and `DynamicDependency` directly on `OxCalcTreeRecalcResult.runtime_effects` together with the corresponding overlay kinds on `OxCalcTreeRecalcResult.runtime_effect_overlays`
2. `treecalc_runner.rs` now checks `result.json` for direct runtime-effect and overlay artifact reachability and checks `explain.json` for direct embedded runtime-effect family plus overlay visibility in both the host-sensitive and dynamic-reference cases
3. no current host-facing or replay-facing closure claim depends on unexercised `CapabilitySensitive` or `ShapeTopology` emission, and W026 now records that explicitly rather than leaving it implicit

### Sequence 2E: W026-To-W029 Boundary Rule
W026 now ends at the current consumed-now transport and reachability truth for runtime-derived and execution-restriction-sensitive facts. W029 begins where broader live runtime-derived realization starts.

W026 owns and closes:
1. the consumed-now semantic minimum family split for the current TreeCalc-first lane
2. the ownership split between canonical OxFml families and OxCalc-local projection labels for the current packet
3. the direct host-facing and replay-facing reachability rule for the emitted current-family subset
4. the statement of explicit current absences, including admitted but unexercised runtime-derived families that the current TreeCalc-first lane does not yet emit distinctly

W029 owns and widens:
1. new emitted runtime-derived families beyond the current `DynamicDependency` and `ExecutionRestriction` subset
2. capability-sensitive and shape/topology-sensitive runtime-derived realization when the live TreeCalc path actually exercises them
3. broader overlay-closure behavior across reject, fallback, accepted-candidate, and published-success paths
4. any later question about whether execution-restriction observations become richer runtime-derived or publication-sensitive realized behavior in the live engine path

Non-overclaim rule:
1. closing W026 item 10 does not claim W029 semantics are already realized
2. W026 does not own the full runtime-derived taxonomy, full overlay closure, or broader runtime-derived lifecycle semantics for the live TreeCalc engine
3. W029 does not reopen W026's consumed-now packet unless live implementation proves the current transport or reachability claims false

Current interpretation rule:
1. if a host or seam consumer only needs to know what the current TreeCalc-first lane transports and exposes directly today, W026 is the authority
2. if the work concerns realizing new emitted families, widening overlay behavior, or hardening runtime-derived behavior across more runtime states, W029 is the governing packet
3. the `OxCalcTree` host-facing contract remains above both packets and should not be widened merely because W029 grows the local runtime-derived realization beneath it

Evidence for the current boundary:
1. W026 now closes with direct consumer and replay reachability evidence only for the current emitted `DynamicDependency` and `ExecutionRestriction` subset
2. W029 still records capability-sensitive and shape/topology-sensitive runtime handling as unexecuted and keeps broader published-success-path runtime-derived overlay closure open
3. the live TreeCalc code still lowers only the current host-sensitive and dynamic-potential residual carriers into emitted runtime effects, which matches the W026 floor and leaves broader family realization to W029

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

### Sequence 3A: First TreeCalc Publication-Consequence Packet
For the current TreeCalc-first coordinator path, W026 now locks the first publication-consequence packet by distinguishing the publication-critical value delta that is carried today from the broader consequence families that remain explicit current absences rather than collapsed members of the published value update set.

Current publish-facing consequence categories:
1. `value_delta` is explicit and currently carried on `PublicationBundle.published_view_delta`
2. `shape_delta` is a distinct consequence category in the TreeCalc seam vocabulary, but the current local TreeCalc publication bundle does not yet emit a distinct published `shape_delta`
3. `topology_delta` is a distinct consequence category in the TreeCalc seam vocabulary, but the current local TreeCalc publication bundle does not yet emit a distinct published `topology_delta`
4. `format_delta` is an optional consequence category and is an explicit current absence in the first TreeCalc publication bundle
5. `display_delta` is an optional consequence category and is an explicit current absence in the first TreeCalc publication bundle

Presence and absence rule:
1. when `publication_bundle: Option<PublicationBundle>` is present, `published_view_delta` is the current TreeCalc-first published `value_delta`
2. the absence of a distinct `shape_delta` or `topology_delta` field on the current `PublicationBundle` does not mean those categories are semantically merged into `value_delta`; it means they are not yet published as distinct consequence families on this first local floor
3. the absence of `format_delta` or `display_delta` on the current `PublicationBundle` is explicit and must not be interpreted as `value_delta` standing in for either optional category

Current local interpretation rule:
1. `AcceptedCandidateResult.dependency_shape_updates` is the current pre-publication shape-sensitive witness surface, not a published `shape_delta` or `topology_delta`
2. the current local TreeCalc path emits `dependency_shape_updates` as empty in the exercised publication lane, so W026 does not treat shape- or topology-sensitive publication as already realized
3. the first publication-consequence packet is therefore honest only if it preserves `value_delta` as present and the other four consequence categories as explicit current absences

Evidence for the current local floor:
1. `coordinator.rs` publishes `PublicationBundle` with `published_view_delta`, `published_runtime_effects`, and trace markers, but no distinct published `shape_delta`, `topology_delta`, `format_delta`, or `display_delta`
2. `coordinator.rs` and `treecalc.rs` still surface `dependency_shape_updates` only on `AcceptedCandidateResult`, and the current local TreeCalc publication path emits that surface as empty
3. `treecalc_runner.rs` emits the current publication bundle into `result.json` and `explain.json` with `published_view_delta` and published runtime effects only, which matches the current first publication-consequence floor

### Sequence 3B: Publish-Critical Versus Replay-Only Carriage Rule
For the current TreeCalc-first publication lane, W026 now locks the carriage split between categories that are publish-critical now, categories that are replay-visible but not publish-critical yet, and categories that remain only local-floor evidence today.

Current carriage split:
1. `value_delta` is publish-critical now and is carried by `PublicationBundle.published_view_delta`
2. `published_runtime_effects` and `trace_markers` are replay-visible publication sidecars in the current local lane, but they are not yet treated as publish-critical consequence categories for the first TreeCalc coordinator floor
3. `dependency_shape_updates` remains only local-floor evidence on `AcceptedCandidateResult`; it is not yet a published consequence carrier
4. `shape_delta`, `topology_delta`, optional `format_delta`, and optional `display_delta` remain explicit current absences rather than replay-only members of the current publication bundle

Surface rule:
1. `coordinator.rs` and `consumer.rs` must keep `published_view_delta` distinct from `published_runtime_effects` so replay-visible runtime sidecars do not masquerade as publish-critical value consequences
2. `result.json` and `explain.json` must expose the current carriage split explicitly for published runs, rather than leaving `published_runtime_effects`, trace markers, and local-floor shape evidence to be inferred from partial fields
3. W026 does not treat the presence of replay-visible publication sidecars as proof that those sidecars already participate in first-phase publish-critical coordinator correctness

Current interpretation rule:
1. a first-phase TreeCalc host may rely on published `value_delta` as the current publish-critical consequence family
2. a first-phase TreeCalc host may inspect published runtime effects and trace markers for replay honesty, diagnostics, and later widening work, but W026 does not yet classify them as publish-critical
3. any future promotion of runtime sidecars, shape updates, topology consequences, or optional format/display consequences into publish-critical status belongs to later Sequence 3 widening and must not be inferred from the current local bundle shape

Evidence for the current local floor:
1. `coordinator.rs` still publishes only `published_view_delta`, `published_runtime_effects`, and `trace_markers` on `PublicationBundle`
2. `treecalc_runner.rs` now emits explicit publication carriage classification in both `result.json` and `explain.json` for published cases, keeping publish-critical versus replay-visible versus local-floor-only categories separate
3. `treecalc_runner.rs` now tests that emitted published artifacts preserve `value_delta` as publish-critical, `published_runtime_effects` plus `trace_markers` as replay-visible sidecars, and `dependency_shape_updates` as local-floor-only evidence

### Sequence 3C: Execution-Restriction Versus Publication/Topology Interaction Rule
For the current TreeCalc-first lane, W026 now locks the interaction rule between execution-restriction observations and publication/topology consequences.

Current interaction rule:
1. an execution-restriction observation in the current local TreeCalc lane is carried as a runtime-derived effect with family `ExecutionRestriction`
2. that observation is also reflected through typed no-publish rejection for the current host-sensitive failure floor
3. the current local TreeCalc lane does not yet promote execution-restriction observations into published `value_delta`, published `shape_delta`, published `topology_delta`, optional `format_delta`, or optional `display_delta` consequences

Current no-publication rule:
1. the current host-sensitive execution-restriction path rejects before `accept_and_publish`, so `publication_bundle` remains absent
2. the absence of a publication bundle in that path is not accidental local behavior; it is the current consumed-now rule for execution-restriction interaction with publication
3. W026 therefore treats current execution-restriction observations as runtime-effect sidecars plus typed no-publish context, not as publication-sensitive or topology-sensitive consequences

Current topology rule:
1. W026 does not treat the current execution-restriction floor as a topology-sensitive consequence carrier
2. any future promotion of execution-restriction facts into topology- or shape-sensitive consequence handling belongs to later Sequence 3 widening and must not be inferred from the current reject path
3. W029 may later widen runtime-derived realization, but it does not make current execution-restriction observations publication-sensitive or topology-sensitive by implication

Evidence for the current local floor:
1. `treecalc.rs` routes host-sensitive execution-restriction failure through `reject_run` before publication and returns `publication_bundle: None`
2. `consumer.rs` now tests the ordinary `OxCalcTreeRuntimeFacade` host-sensitive execution-restriction case and asserts that the result is rejected with no publication bundle
3. `treecalc_runner.rs` now emits explicit execution-restriction interaction classification in both `result.json` and `explain.json` and tests the checked host-sensitive reject fixture for `rejected_no_publication`, `publication_sensitive_consequence = false`, and `topology_sensitive_consequence = false`

## Explicit W026 Closure Work List
This is the full W026 closure list. No scope growth is expected beyond these items; if every item below is closed with evidence, W026 should not need to revisit the same seam area again.

### Group A: Sequence 1 Caller-Context And Reference Intake
1. `[closed] Document update` Define the first closed TreeCalc reference subset. Closed in `Sequence 1A: First Closed TreeCalc Reference Subset`; the exact in-packet direct, relative, unresolved, host-sensitive, and dynamic-potential families are now named explicitly, and the broader out-of-packet reference families are named explicitly rather than being implicitly deferred.
2. `[closed] Document update + code review and update` Lock the per-formula identity and compatibility packet for that subset. Closed in `Sequence 1B: Per-Formula Identity And Compatibility Packet`; the required floor is now explicit and the current TreeCalc and deterministic upstream-host intake paths now carry `formula_stable_id`, explicit `formula_token`, optional `bind_artifact_id`, `structure_context_version`, `compatibility_basis`, and `artifact_token_basis` without relying on implicit derivation inside the packet.
3. `[closed] Document update + code review and update` Lock the carried caller-context packet for the first relative-reference subset. Closed in `Sequence 1C: Caller-Context Packet For The First Closed Subset`; `caller_anchor`, `formula_channel_kind`, explicit `address_mode`, and `structure_context_version` are now recorded as the carried caller-context floor, and the deterministic upstream-host packet now preserves `address_mode` explicitly instead of inferring it only from channel kind.
4. `[closed] Document update + code review and update` Close the rebind-versus-recalc rule for caller-sensitive forms. Closed in `Sequence 1D: Rebind Versus Recalc Rule For Caller-Sensitive Forms`; W026 now distinguishes coarse structural edit pressure from per-formula invalidation reason, the caller-sensitive first-subset families are named explicitly, and the local TreeCalc post-edit rerun path now carries explicit structural invalidation seeds instead of flattening every rerun to `StructuralRecalcOnly`.
5. `[closed] Document update + code review and update` Close the dependency-descriptor mapping for the Sequence 1 reference families. Closed in `Sequence 1E: Dependency-Descriptor Mapping For The First Closed Subset`; W026 now names the exact mapping for direct, admitted relative, unresolved-relative, explicit unresolved, host-sensitive, and dynamic-potential carriers, and the OxFml-backed TreeCalc intake now has direct mixed-carrier tests proving the live lowering matches that packet.

### Group B: Sequence 2 Candidate, Reject, And Runtime-Derived Transport
6. `[closed] Document update` Lock the candidate/reject/correlation packet. Closed in `Sequence 2A: Candidate, Reject, And Correlation Packet`; W026 now names the exact consumed-now correlation and consequence identifiers for the TreeCalc-first lane, records `commit_attempt_id`, `reject_record_id`, and `fence_snapshot_ref` as explicit current absences rather than implied members, and locks the first-phase typed reject floor as a required typed carrier rather than generic failure text.
7. `[closed] Document update + code review and update` Close the distinction between canonical OxFml object families and local OxCalc projection labels. Closed in `Sequence 2B: Canonical OxFml Families Versus OxCalc Projection Labels`; W026 now names the current canonical-family alignments versus OxCalc-local labels, and TreeCalc-emitted artifacts now carry explicit ownership markers so local reject and runtime-effect labels do not masquerade as canonical OxFml names.
8. `[closed] Document update + code review and update` Lock the semantic minimum runtime-derived packet. Closed in `Sequence 2C: Semantic Minimum Runtime-Derived Packet`; W026 now names the exact emitted dynamic-dependency and execution-restriction families, records capability-sensitive runtime effects as an explicit current absence, keeps the non-assumption about a not-yet-frozen merged OxFml carrier explicit, and backs the emitted family split with checked TreeCalc artifact evidence.
9. `[closed] Document update + code review and update` Close the runtime-effect and overlay reachability rule. Closed in `Sequence 2D: Runtime-Effect And Overlay Reachability Rule`; W026 now names the exact host-facing and replay-facing reachability floor for emitted runtime-derived families, states which admitted but unexercised families may stay below the current host-facing contract, and backs that rule with direct `OxCalcTreeRecalcResult` plus `result.json` and `explain.json` evidence.
10. `[closed] Document update + code review and update` Close the W026-to-W029 boundary. Closed in `Sequence 2E: W026-To-W029 Boundary Rule`; W026 now explicitly stops at current runtime-derived transport and reachability truth for the emitted TreeCalc-first subset, W029 now explicitly owns broader runtime-derived family realization and overlay closure beneath that floor, and the local TreeCalc runtime comments now reflect the same split.

### Group C: Sequence 3 Publication And Topology Consequence Breadth
11. `[closed] Document update` Lock the first TreeCalc publication-consequence packet. Closed in `Sequence 3A: First TreeCalc Publication-Consequence Packet`; W026 now preserves `value_delta`, `shape_delta`, `topology_delta`, optional `format_delta`, and optional `display_delta` as distinct consequence categories, records `value_delta` as the only currently published category on the first local TreeCalc floor, and records the other four as explicit current absences rather than silent members of `published_view_delta`.
12. `[closed] Document update + code review and update` Close the publish-critical versus replay-only distinction. Closed in `Sequence 3B: Publish-Critical Versus Replay-Only Carriage Rule`; W026 now names the current publish-critical, replay-visible, and local-floor-only categories explicitly, and the emitted published TreeCalc artifacts now expose that split directly instead of leaving it implicit in partial publication fields.
13. `[closed] Document update + code review and update` Close the execution-restriction versus publication/topology interaction rule. Closed in `Sequence 3C: Execution-Restriction Versus Publication/Topology Interaction Rule`; W026 now states that the current execution-restriction floor is runtime-effect-plus-typed-no-publish context rather than a published consequence family, and the host-facing plus replay-facing evidence surfaces now represent that rule explicitly.

### Group D: Host-Facing Contract And Canonical Packet Sync
14. `[closed] Document update` Close the `OxCalcTree` contract reachability wording for W026. Closed in `CORE_ENGINE_OXCALCTREE_CONSUMER_INTERFACE_AND_HOST_CONTRACT_V1.md`; the host-facing contract now names the exact W026 facts that must remain directly reachable from `OxCalcTreeRecalcResult`, the narrower W026 seam facts that may remain below the host-facing contract for now, and the explicit rule that W026 does not authorize a second host-facing OxCalc seam layer.
15. `[closed] Document update` Synchronize the canonical packet text across the W026 authority set. Closed in `CORE_ENGINE_OXFML_SEAM.md`, `CORE_ENGINE_TREECALC_OXFML_SEAM_NEGOTIATION_MATRIX.md`, `CORE_ENGINE_TREECALC_SEMANTIC_COMPLETION_PLAN.md`, and `NOTES_FOR_OXFML.md`; those authority and companion docs now all state the same executed Sequence 1 / Sequence 2 / Sequence 3 packet floor, the same explicit current absences, and the same `canonical but narrower` rule for breadth beyond that first floor.

### Group E: Evidence, Handoff Decision, And W026 Closure
16. `[closed] Code review and update` Attach deterministic exercised evidence for every family W026 claims is explicit. Closed in `Sequence Evidence Matrix`; upstream-host scaffolding, TreeCalc reference-lowering tests, TreeCalc runtime tests, and emitted replay/explain artifact checks now collectively exercise the executed Sequence 1 / Sequence 2 / Sequence 3 floor rather than leaving any claimed explicit family only in prose.
17. `[closed] Document update + code review and update` Make replay and explain visibility explicit for the carried families. Closed in `Sequence Replay And Explain Visibility Rule`; caller-context-sensitive post-edit invalidation reasons, emitted runtime-derived families, and publication-consequence classifications are now visible on the emitted replay/explain surfaces where W026 claims they are explicit, with no remaining mismatch between the W026 claim set and the artifact floor.
18. `[closed] Document update` Make the narrower handoff decision final for W026. Closed in `Sequence Narrower Handoff Decision`; the exercised first packet is now explicitly treated as sufficient for W026 without filing `HANDOFF-CALC-002`, and any later handoff trigger is now reserved only for a live insufficiency beyond that exercised floor rather than for ambiguity about current consumed-now seam truth.
19. `[closed] Document update` Perform the final W026 closure audit. Closed in `Final W026 Closure Audit`; all three sequences are now locked as one executed consumed-seam intake packet, no hidden formula-language reinterpretation obligation remains on OxCalc for the declared first TreeCalc subset, and W027, W028, and W029 may now proceed on executed packet truth rather than on prose-only seam assumptions.

### Sequence Evidence Matrix
Current deterministic exercised evidence for the executed W026 floor is now:

1. Sequence 1 caller-context and reference intake:
   - upstream-host packet and fixture evidence:
     - `upstream_host.rs` proves carried `formula_stable_id`, `formula_token`, optional `bind_artifact_id`, `formula_channel_kind`, `address_mode`, `caller_anchor`, and `structure_context_version`
     - `upstream_host_scaffolding.rs` proves the checked-in deterministic upstream-host corpus preserves those same carried fields
   - TreeCalc reference-family evidence:
     - `formula.rs` proves direct, sibling-offset, parent-relative, and ancestor-relative lowering into dependency descriptors
     - `formula.rs` proves host-sensitive and unresolved carriers surface as typed dependency diagnostics
     - `treecalc.rs` proves the first mixed-carrier OxFml-backed mapping for direct, admitted relative, unresolved-relative, explicit unresolved, host-sensitive, and dynamic-potential carriers
   - structural invalidation evidence:
     - `treecalc.rs` proves relative-reference rename pressure becomes rebind-required while direct-only move pressure remains recalc-only
2. Sequence 2 candidate/reject/runtime-derived transport:
   - host-facing consumer evidence:
     - `consumer.rs` proves `ExecutionRestriction` and `DynamicDependency` are directly reachable on `OxCalcTreeRecalcResult`, with no publication bundle on the execution-restriction reject path
   - replay/explain artifact evidence:
     - `treecalc_runner.rs` proves `candidate_result_id`, `publication_id`, typed reject kind, and the current absence of later correlation ids on the local artifact floor
     - `treecalc_runner.rs` proves emitted `ExecutionRestriction` and `DynamicDependency` runtime-effect families on checked host-sensitive and dynamic-reference fixture cases
3. Sequence 3 publication/topology breadth:
   - published artifact evidence:
     - `treecalc_runner.rs` proves `value_delta` as the current published consequence family through `published_view_delta`
     - `treecalc_runner.rs` proves `published_runtime_effects` and `trace_markers` are replay-visible sidecars rather than publish-critical categories
     - `treecalc_runner.rs` proves `dependency_shape_updates` remains local-floor-only evidence and that `shape_delta`, `topology_delta`, optional `format_delta`, and optional `display_delta` remain explicit current absences
   - execution-restriction interaction evidence:
     - `treecalc_runner.rs` proves the checked host-sensitive reject fixture remains `rejected_no_publication` with `publication_sensitive_consequence = false` and `topology_sensitive_consequence = false`

Interpretation rule:
1. W026 now treats the matrix above as the required deterministic evidence floor for the executed first packet
2. if a claimed explicit family or explicit current absence is not represented by one of those exercised surfaces, it is not yet evidence-backed for W026
3. later items may widen replay or explain visibility further, but they do not reopen the fact that the first executed W026 floor is now evidence-backed across Sequence 1, Sequence 2, and Sequence 3

### Sequence Replay And Explain Visibility Rule
The emitted artifact floor for W026 is now explicitly locked as follows:

1. Sequence 1 caller-context-sensitive behavior:
   - `post_edit/invalidation_seeds.json` is the replay-visible sidecar for per-formula invalidation reasons
   - `post_edit/result.json` and `post_edit/explain.json` both embed the same `invalidation_seeds` list, so the rebind-versus-recalc distinction does not depend on trace-only or unit-test-only evidence
   - the checked rename-driven relative-reference case must surface `StructuralRebindRequired`, and the checked direct-target move case must surface `StructuralRecalcOnly`
2. Sequence 2 runtime-derived and execution-restriction transport:
   - `result.json`, `runtime_effects.json`, `runtime_effect_overlays.json`, and `explain.json` remain the explicit emitted surfaces for `DynamicDependency` and `ExecutionRestriction`
   - where W026 claims execution-restriction interaction is explicit, it must be visible through `execution_restriction_interaction` on both `result.json` and `explain.json`
3. Sequence 3 publication and consequence breadth:
   - where W026 claims the publish-critical versus replay-only split is explicit, it must be visible through `publication_bundle.carriage_classification` on both `result.json` and `explain.json`
   - where W026 claims a consequence category is an explicit current absence, it must remain named in that same `carriage_classification` object rather than being omitted silently

Closure rule:
1. item 17 only remains closed while the emitted artifacts named above continue to expose the exact Sequence 1, Sequence 2, and Sequence 3 families that W026 claims are explicit
2. if a family moves below emitted replay/explain visibility, W026 must either restore that visibility or narrow the claim before later items may treat the packet as closure-ready

### Sequence Narrower Handoff Decision
The narrower handoff decision for W026 is now final:

1. no narrower `HANDOFF-CALC-002` is required for the executed W026 first packet
2. the reason is not merely that OxFml note traffic stayed cooperative; it is that W026 now has an exercised local packet for:
   - the first closed Sequence 1 reference subset plus explicit identity and caller-context carriage
   - the first closed Sequence 2 correlation, reject, and runtime-derived transport floor
   - the first closed Sequence 3 publication-consequence and no-publication interaction floor
3. the explicit current absences in that first packet are now named and exercised as absences rather than as unresolved ambiguity
4. the remaining `canonical but narrower` topics are broader-breadth topics beyond the executed first floor, not missing consumed-now clauses required for W027, W028, or W029 to proceed honestly

Future-trigger rule:
1. a narrower handoff is only justified if a later TreeCalc or coordinator-facing lane exposes a concrete insufficiency beyond this first packet
2. that future trigger must name the exact missing family or missing carried fact; broader closure pressure alone is not enough
3. note-level residual language may continue to describe broader non-executed breadth, but it may not be used to imply that W026 still lacks a first executable consumed-seam packet

### Final W026 Closure Audit
W026 now reaches its declared gate for the first executed TreeCalc consumed-seam packet.

Scope audit:
1. the declared in-scope identity, bind/reference, candidate/reject/runtime-derived, ownership-split, and narrowing obligations have been converted from note-level summaries into explicit packet truth across Sequence 1, Sequence 2, and Sequence 3
2. the declared out-of-scope topics remain out of scope and are not being smuggled back into W026 through ambiguous wording
3. the landed `OxCalcTree` contract is now aligned to the executed W026 floor without introducing a second host-facing seam layer

Gate audit:
1. OxCalc can now consume formula artifact identities, bind identities, and reference meaning for the first declared TreeCalc families without depending on proving-lane-only implied semantics
2. unresolved seam items are no longer hidden; they are either explicit current absences inside the first packet or broader `canonical but narrower` residuals beyond it
3. no hidden formula-language reinterpretation obligation remains on OxCalc for the first closed TreeCalc subset:
   - OxFml remains the owner of formula parse/bind/evaluator semantics
   - OxCalc now owns only the declared consequence mapping, carriage, and coordinator-facing projection of the exercised first packet
4. the active TreeCalc seam topics have completed the required note-exchange pass, and the exercised local packet now fixes the first consumed-now floor without a narrower handoff

Successor-lane audit:
1. W027 may now proceed on explicit dependency-descriptor and invalidation-seed truth rather than on compressed seam prose
2. W028 may now proceed on explicit candidate/reject/correlation and publication-consequence packet truth rather than on inferred coordinator packet shape
3. W029 may now proceed on the explicit W026-to-W029 boundary rule and the executed current runtime-derived transport floor rather than on ambiguity about current emitted families
4. broader caller-context, broader runtime-derived-family realization, and broader publication/topology breadth remain later widening work, but they are no longer blockers to honest execution of W027, W028, or W029 for the first declared floor

Closure interpretation rule:
1. W026 is now reached-gate for its declared first-packet scope
2. later widening beyond that first packet belongs to W027 through W031 and must not be reported as unfinished W026 scope
3. if later work discovers a concrete insufficiency in the exercised packet itself, that is a new mismatch against the reached-gate packet, not evidence that W026 never established one

## Pre-Closure Verification Checklist
1. Spec text and realization notes updated for all in-scope items: yes
2. Pack expectations updated for affected packs: yes
3. At least one deterministic replay artifact exists per in-scope behavior: yes
4. Semantic-equivalence statement provided for policy or strategy changes: yes
5. FEC/F3E cross-repo impact assessed and handoff filed if needed: yes
6. All required tests pass: yes
7. No known semantic gaps remain in declared scope: yes
8. Completion language audit passed: yes
9. `IN_PROGRESS_FEATURE_WORKLIST.md` updated: yes
10. `CURRENT_BLOCKERS.md` updated if needed: yes

## Completion Claim Self-Audit
1. Scope re-read: pass
2. Gate criteria re-read: pass
3. Silent scope reduction check: pass
4. "Looks done but is not" pattern check: pass
5. Self-audit inclusion in report: pass

## Status
- execution_state: complete
- scope_completeness: scope_complete
- target_completeness: target_complete
- integration_completeness: partial
- open_lanes:
  - broader dependency-graph and invalidation-closure widening now belongs to W027
  - broader candidate-result and publication integration widening now belongs to W028
  - broader runtime-derived family and overlay-closure widening now belongs to W029
  - broader corpus, baseline, and assurance widening now belongs to W030 and W031
- claim_confidence: high
- reviewed_inbound_observations: W020 remains the current downstream seam baseline; the latest OxFml topic-matrix and narrower W026-focused replies make formula/bind identity, candidate consequence, reject-context, and direct-binding preservation consume-now topics, confirm that W026 may proceed on a narrowed first subset for relative-reference, unresolved-reference, runtime-derived transport, and semantic-format/display handling without a new handoff, the latest host/runtime reply agrees that `OXFML_HOST_RUNTIME_AND_EXTERNAL_REQUIREMENTS.md` is strong enough for the first coordinator-host implementation slice while keeping caller-anchor/address-mode, execution-restriction transport breadth, and publication/topology breadth as note-level residuals, the latest `W051`/`W052` stand-in refinement confirms deterministic fixture-host inputs plus optional registered-external presence without freezing the production coordinator API or shifting catalog ownership into OxCalc, the latest sharpened `W052` reply aligns on the direct adopted packet names, seven-field descriptor, current-phase OxFml-owned funnel-family ownership, and the bind-visible-snapshot-versus-targeted-reevaluation split as sufficient for first TreeCalc-facing planning, and the latest OxFml residual reply confirms that W026 Sequence 1, Sequence 2, and Sequence 3 all remain `canonical but narrower` with explicit consumed-now carriers and no current narrower handoff trigger
