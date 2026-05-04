# Core Engine OxCalc + OxFml Formalization Pass Plan

Status: `planning_baseline`
Owner repo: `OxCalc`
Scope: `OxCalc + OxFml seam/formal surfaces; OxFunc semantic kernels excluded except narrow LET/LAMBDA boundary carrier assumptions`

## 1. Purpose

This document plans a comprehensive formalization pass over the multi-node core engine and its evaluator-facing seam.

The pass is executed in the OxCalc repo because OxCalc owns the coordinator, recalc, dependency, overlay, publication, replay-consumption, and TreeCalc runtime integration model. The pass includes OxFml because the core engine cannot be formalized honestly without modeling the evaluator session, candidate, commit, reject, fence, trace, and runtime-effect facts that OxCalc consumes.

This document does not transfer canonical ownership of OxFml formula or evaluator semantics into OxCalc. OxFml remains authoritative for formula grammar, bind, evaluator session lifecycle, FEC/F3E evaluator-side clauses, trace schema, reject meaning, and replay-safe identity. OxCalc formal artifacts may model those surfaces as imported or assumed contracts, and any required normative OxFml change must go through the normal handoff path.

## 1A. Highest-Level Goal

The formalization effort exists to make the core calculation engine trustworthy under change.

The goal is not to prove every spreadsheet function or to move OxFunc semantics into this repo. The goal is to make the engine/seam contract precise enough that later optimization, scaling, TreeCalc widening, replay promotion, and staged concurrency cannot accidentally change observable meaning.

There is one explicit boundary exception: `LET` and `LAMBDA` are special enough that the formalization must model their OxFml/OxFunc interaction at the carrier level. They thread formula binding, prepared-call shape, local names, lambda values, invocation identity, and dependency/runtime facts through the surfaces that OxCalc consumes. This does not make OxCalc authoritative for general OxFunc function semantics; it records the cross-lane fragment needed to reason about OxCalc-visible engine behavior.

At the highest level, the pass should let OxCalc answer these questions with checkable artifacts rather than prose intuition:
1. did OxFml produce a typed candidate, commit, reject, fence, trace, or runtime-effect fact that OxCalc consumed without reinterpretation,
2. did OxCalc preserve the required invariants while updating dependency, invalidation, overlay, coordinator, and publication state,
3. did rejected work avoid publication,
4. did accepted work publish atomically,
5. did dependency and runtime-effect facts avoid silent loss,
6. did overlay reuse and eviction remain epoch/fence safe,
7. did a strategy change preserve the same stabilized observable results.

The intended end state is a coupled assurance stack: specs, Lean/TLA+ models, replay witnesses, tests, pack mappings, and capability claims all describing the same behavior at their appropriate level of precision.

TraceCalc is a central part of that stack. The formalization path should check both the reference-machine surface and the production/core-engine surface, but with different authority roles: TraceCalc is the executable correctness oracle for the spec surfaces it realizes, while optimized or production implementations must demonstrate conformance to that oracle for covered behavior.

The pass is also a quality-tempering pass over the core-engine specs themselves. The formal work should not merely cite the initial specs; it should read through them, sharpen ambiguous state and transition language, correct drift against realized artifacts, and keep the prose documents synchronized with the proof, model-check, replay, and pack obligations that emerge.

The pass should also perform a no-loss review against the original bootstrap formal/theory materials. Those archived materials are not canonical, but they preserve early intent around layered formal models, replay-first semantics, graph/SCC baselines, dynamic dependency overlays, FEC/F3E transactionality, visibility as scheduling metadata, and advanced lanes such as self-adjusting computation. W033 should either reconnect those ideas to current specs and artifacts, defer them explicitly, or record why they are no longer carried forward.

The pass is not a fixed-spec compliance exercise. Current implementation behavior and current spec text are important evidence surfaces, but neither is treated as the final word when formalization exposes a better model. W033 should support deliberate spec evolution: clarify the model, revise OxCalc-owned scope where evidence justifies it, file handoffs where OxFml ownership is involved, and record deferred or rejected alternatives when the evidence is not yet strong enough.

## 1B. Formal Leverage Model

The pass should use several complementary forms of leverage:

1. Operational semantics and refinement:
   - treat TraceCalc as a small-step executable reference semantics for covered behavior,
   - state production/core-engine behavior as a refinement of TraceCalc over an explicit observable surface,
   - define the observable surface in terms of published values, rejects, publication epochs, dependency/runtime-effect facts, trace identity, and replay identity.
2. TLA+ temporal and state modeling:
   - use TLA+ for publication fences, accept/reject publication transitions, pinned readers, overlay retention, dependency invalidation interleavings, scheduling-policy variation, and later concurrency pressure.
3. Lean transition invariants:
   - use Lean for crisp state-transition properties such as reject-is-no-publish, accepted-candidate-is-not-publication, publication atomicity, invalidation-closure coverage, protected-overlay retention, and replay-equivalent sequential histories.
4. Graph theory and incremental-computation vocabulary:
   - make DAG, SCC, runtime dependency graph, invalidation closure, topological order, cycle region, fixed point, dirty set, and affected set vocabulary explicit in the spec and artifacts.
5. Abstract interpretation and dataflow conservatism:
   - model uncertain references, dynamic references, and runtime-derived dependencies as conservative approximations where over-invalidation is allowed but under-invalidation is a correctness fault.
6. Metamorphic and differential testing:
   - add transformations that should preserve observable results, including independent-node reorderings, from-scratch versus incremental recalc, `LET` inlining, `LAMBDA` call refactoring, and scheduling-policy changes.

## 2. Scope Boundary

### 2.1 In Scope

1. OxCalc core state and transition vocabulary:
   - structural snapshots,
   - runtime views,
   - dependency graphs,
   - invalidation closure,
   - coordinator state,
   - publication state,
   - pinned readers,
   - runtime overlays,
   - emitted replay and explain artifacts.
2. OxFml-consumed evaluator and seam vocabulary:
   - `prepare -> open_session -> capability_view -> execute -> commit`,
   - session identity and fences,
   - accepted candidate results,
   - commit bundles,
   - typed reject records,
   - runtime-derived evaluator facts,
   - trace and replay correlation keys,
   - public consumer facade packets used by OxCalc.
3. Narrow OxFml/OxFunc special-function boundary fragment:
   - `LET` local-name binding and value visibility as represented in OxFml evaluator facts,
   - `LAMBDA` value, closure/call identity, and invocation carrier shape where these affect OxCalc-visible dependency, runtime-effect, trace, or replay facts,
   - prepared-call and function-boundary packet assumptions needed for OxCalc to consume the resulting evaluator facts safely,
   - no ownership of general OxFunc function kernels, coercion semantics, or catalog truth.
4. Cross-lane invariants:
   - candidate result is not publication,
   - reject is no-publish,
   - accepted publication is atomic,
   - OxFml facts do not become OxCalc scheduler policy by implication,
   - coordinator publication does not reinterpret OxFml-owned reject or fence meaning,
   - dynamic dependency and runtime-effect facts are carried into OxCalc invalidation and replay without silent loss.
5. Replay and evidence coupling:
   - TraceCalc and TreeCalc local baselines,
   - OxFml replay fixtures and retained witness registers as read-only inputs,
   - cross-lane evidence matrices,
   - pack and capability-claim mapping.

### 2.2 Out Of Scope

1. OxFunc function semantic kernels, coercion semantics, and catalog truth, except for the narrow `LET`/`LAMBDA` carrier assumptions named above.
2. Direct edits to the OxFml repo from this OxCalc pass.
3. Full Excel grid, file adapter, UI, VBA, or host-product semantics.
4. Stage 2 concurrency promotion as a realized policy change.
5. Pack-grade claims that are not backed by current replay, proof, model-check, and capability evidence.

OxFunc may appear only as an opaque provider of prepared-call outcomes and function-catalog facts already surfaced through OxFml. The formal pass may state assumptions about OxFunc-facing packets, including the `LET`/`LAMBDA` carrier fragment, but it must not prove or redefine OxFunc semantics.

## 3. Source Authority Set

The W033 entry source freeze and artifact layout are recorded in
`docs/spec/core-engine/w033-formalization/W033_SOURCE_AUTHORITY_AND_ARTIFACT_LAYOUT.md`.
That packet pins the OxCalc, OxFml, and Foundation source bases used by the pass,
declares the W033 artifact root, and records local tool availability.

### 3.1 OxCalc Sources

1. `README.md`
2. `CHARTER.md`
3. `OPERATIONS.md`
4. `docs/WORKSET_REGISTER.md`
5. `docs/BEADS.md`
6. `docs/spec/core-engine/CORE_ENGINE_ARCHITECTURE.md`
7. `docs/spec/core-engine/CORE_ENGINE_STATE_AND_SNAPSHOTS.md`
8. `docs/spec/core-engine/CORE_ENGINE_RECALC_AND_INCREMENTAL_MODEL.md`
9. `docs/spec/core-engine/CORE_ENGINE_OVERLAY_AND_DERIVED_RUNTIME.md`
10. `docs/spec/core-engine/CORE_ENGINE_COORDINATOR_AND_PUBLICATION.md`
11. `docs/spec/core-engine/CORE_ENGINE_OXFML_SEAM.md`
12. `docs/spec/core-engine/CORE_ENGINE_FORMALIZATION_AND_ASSURANCE.md`
13. `docs/spec/core-engine/CORE_ENGINE_REALIZATION_ROADMAP.md`
14. `docs/spec/core-engine/CORE_ENGINE_FORMAL_MODEL.md`
15. `docs/spec/core-engine/CORE_ENGINE_THEORY_AND_ALTERNATIVE_PATHS.md`
16. `docs/spec/core-engine/CORE_ENGINE_TRACECALC_REFERENCE_MACHINE.md`
17. `docs/spec/core-engine/CORE_ENGINE_TEST_HARNESS_AND_FIXTURES.md`
18. `docs/spec/core-engine/CORE_ENGINE_TEST_SCENARIO_SCHEMA_AND_TRACECALC.md`
19. `docs/spec/core-engine/CORE_ENGINE_TEST_VALIDATOR_AND_RUNNER_CONTRACT.md`
20. `docs/spec/core-engine/CORE_ENGINE_REPLAY_APPLIANCE_ADAPTER.md`
21. `docs/spec/core-engine/CORE_ENGINE_TREECALC_SEMANTIC_COMPLETION_PLAN.md`
22. `docs/spec/core-engine/CORE_ENGINE_TREECALC_ASSURANCE_AUTHORITY_MAP.md`
23. `docs/spec/core-engine/CORE_ENGINE_TREECALC_OXFML_SEAM_NEGOTIATION_MATRIX.md`
24. `docs/spec/core-engine/CORE_ENGINE_OXFML_MINIMAL_UPSTREAM_HOST_INTERFACES.md`
25. `docs/spec/core-engine/CORE_ENGINE_DOWNSTREAM_HOST_SEAM_REFERENCE.md`
26. `docs/spec/core-engine/CORE_ENGINE_OXCALCTREE_CONSUMER_INTERFACE_AND_HOST_CONTRACT_V1.md`

W033 must treat the initial core-engine spec documents as an active review surface, not as frozen background. At minimum, the pass should read through and reconcile the architecture, state/snapshot, recalc/incremental, overlay/runtime, coordinator/publication, OxFml seam, formalization/assurance, roadmap, TraceCalc, TreeCalc, harness, replay, upstream/downstream host, and consumer-interface documents. Clauses that are clarified or corrected by formal work should be updated in place when OxCalc owns them, or routed into a handoff/deferred lane when another repo owns the surface.

### 3.1A Historical And Original-Idea Inputs

These inputs are historical no-loss review material, not canonical authority when they conflict with current doctrine or current specs:
1. `docs/spec/core-engine/archive/bootstrap-2026-03/CORE_ENGINE_FORMAL_MODEL.md`
2. `docs/spec/core-engine/archive/bootstrap-2026-03/CORE_ENGINE_THEORY_AND_ALTERNATIVE_PATHS.md`
3. `docs/spec/core-engine/archive/rewrite-control-2026-03/SPEC_REWRITE_DOCUMENT_MAP.md`
4. `docs/spec/core-engine/archive/rewrite-control-2026-03/REWRITE_PROMOTION_LEDGER.md`
5. `docs/spec/core-engine/archive/rewrite-control-2026-03/SPEC_REWRITE_PLAN.md`

The no-loss review should explicitly check whether original ideas were:
1. promoted into current canonical specs,
2. realized in current code, formal artifacts, or replay evidence,
3. deferred with a current owner or open lane,
4. intentionally not carried forward because the current architecture rejects them.

### 3.2 OxFml Read-Only Inputs

1. `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md`
2. `../OxFml/docs/spec/OXFML_CONSUMER_INTERFACE_AND_FACADE_CONTRACT_V1.md`
3. `../OxFml/docs/spec/OXFML_FORMALIZATION_AND_VERIFICATION.md`
4. `../OxFml/docs/spec/OXFML_FORMAL_ARTIFACT_REGISTER.md`
5. `../OxFml/docs/spec/OXFML_DELTA_EFFECT_TRACE_AND_REJECT_TAXONOMIES.md`
6. `../OxFml/docs/spec/OXFML_HOST_RUNTIME_AND_EXTERNAL_REQUIREMENTS.md`
7. `../OxFml/docs/spec/OXFML_FIXTURE_HOST_AND_COORDINATOR_STANDIN_PACKET.md`
8. `../OxFml/docs/spec/fec-f3e/FEC_F3E_DESIGN_SPEC.md`
9. `../OxFml/docs/spec/fec-f3e/FEC_F3E_FORMAL_AND_ASSURANCE_MAP.md`
10. `../OxFml/docs/spec/fec-f3e/FEC_F3E_TESTING_AND_REPLAY.md`

### 3.3 Existing Formal And Evidence Floors

1. OxCalc Lean floor:
   - `formal/lean/OxCalc/CoreEngine/Stage1State.lean`
2. OxCalc TLA+ floor:
   - `formal/tla/CoreEngineStage1.tla`
   - `formal/tla/CoreEngineStage1.cfg`
   - `formal/tla/CoreEngineStage1.smoke.cfg`
3. OxCalc replay/evidence floors:
   - `formal/replay/stage1-hand-authored/`
   - `docs/test-corpus/core-engine/tracecalc/`
   - `docs/test-runs/core-engine/tracecalc-reference-machine/`
   - `docs/test-runs/core-engine/treecalc-local/`
   - `docs/test-runs/core-engine/treecalc-scale/`
4. OxFml read-only formal floors:
   - `../OxFml/formal/lean/*.lean`
   - `../OxFml/formal/tla/*.tla`
   - `../OxFml/formal/run_formal.ps1`
5. OxFml read-only replay and witness floors:
   - `../OxFml/crates/oxfml_core/tests/fixtures/session_lifecycle_replay_cases.json`
   - `../OxFml/crates/oxfml_core/tests/fixtures/fec_commit_replay_cases.json`
   - `../OxFml/crates/oxfml_core/tests/fixtures/execution_contract_replay_cases.json`
   - `../OxFml/crates/oxfml_core/tests/fixtures/witness_distillation/retained_witness_set_index.json`

## 4. Work Product Families

### 4.1 Authority And Traceability Matrix

Create an OxCalc-local matrix that maps each formalized claim to:
1. owning source document,
2. source clause or section,
3. modeled object or transition,
4. proof target, TLA+ property, replay witness, or pack obligation,
5. current evidence state,
6. open handoff or non-assumption.

The matrix should distinguish:
1. OxCalc-owned claims,
2. OxFml-owned claims consumed by OxCalc,
3. shared FEC/F3E clauses where OxCalc contributes coordinator-facing requirements,
4. OxFunc-opaque assumptions, including the narrow `LET`/`LAMBDA` carrier fragment where OxCalc-visible behavior depends on it.

### 4.2 Lean-Oriented Model Surface

Expected OxCalc-local Lean module families:
1. `CoreIds`: stable identities, epoch ids, snapshot ids, candidate ids, commit ids, reject ids.
2. `CoreSnapshots`: structural truth, runtime view, and derived-output separation.
3. `OxfmlSeam`: imported OxFml candidate, commit, reject, fence, and trace contract shapes as abstract ADTs.
4. `OxfmlOxfuncBoundary`: opaque `LET`/`LAMBDA` carrier facts for binding, lambda identity, prepared calls, and dependency/runtime-effect visibility.
5. `Coordinator`: accept/reject/publication state and transition skeleton.
6. `Recalc`: dirty, needed, evaluating, verified-clean, publish-ready, rejected, and cycle-blocked states.
7. `Dependencies`: static dependency graph, runtime-effective dependency graph, invalidation closure.
8. `Overlays`: runtime overlays, pinning, retention, and deterministic eviction predicates.
9. `TreeCalc`: tree-substrate specialization with no grid ownership.
10. `Theorems`: first invariant statements and proof backlog.

First theorem families:
1. reject is no-publish,
2. accepted candidate is not publication,
3. accepted publication is atomic,
4. structural runtime work does not mutate structural truth,
5. replay-equivalent sequential histories preserve published outcomes,
6. invalidation closure contains every statically or runtime-dependency-affected target in scope,
7. overlay eviction does not remove protected state,
8. OxFml reject/fence meaning is preserved through OxCalc consumption.

### 4.3 TLA+-Oriented Model Surface

Expected OxCalc-local TLA+ model families:
1. strengthened `CoreEngineStage1` baseline,
2. `CoreOxfmlFecBridge` for evaluator session facts entering coordinator state,
3. `CorePublicationFence` for accept/reject/publication transitions,
4. `CorePinnedReaderOverlay` for pin and overlay retention safety,
5. `CoreDynamicDependency` for dependency update and recalc invalidation interleavings,
6. later `CoreStage2Contention` only after Stage 1 bridge obligations are stable.

First safety properties:
1. no torn publication,
2. reject is no-publish,
3. stale or incompatible fence cannot publish,
4. pinned readers keep a compatible view,
5. protected overlays are not evicted,
6. coordinator-visible evaluator facts are consumed without silently widening publication.

### 4.4 Replay, Oracle, And Witness Surface

Expected OxCalc-local evidence products:
1. a cross-lane replay inventory for OxCalc and OxFml fixture families,
2. a minimal cross-lane witness set tying OxFml candidate/commit/reject events to OxCalc coordinator outcomes,
3. TreeCalc replay projections that expose dependency graph, invalidation closure, runtime effects, overlays, candidate, publication, reject, and trace correlations,
4. retained-witness lifecycle status for any mismatch or gap used as evidence,
5. TraceCalc reference-machine checks that establish the oracle's covered behavior before using it as a comparison authority,
6. production/core-engine conformance checks against TraceCalc for covered observable semantics,
7. explicit capability levels for any Replay appliance claim.

### 4.5 Core Spec Review And Maintenance Surface

The pass should produce a spec-maintenance surface that identifies:
1. clauses confirmed by formal/replay evidence,
2. clauses corrected during the pass,
3. clauses that remain provisional because artifact evidence is missing,
4. vocabulary splits that could cause proof, replay, or implementation drift,
5. spec text that needs OxFml handoff rather than OxCalc-local edits,
6. spec clauses that should be linked to TraceCalc oracle coverage or production conformance coverage.

The maintenance goal is not cosmetic cleanup. It is to keep the core-engine spec set correct as the formal model becomes sharper.

### 4.5A Spec Evolution Decision Surface

The pass should produce a decision surface for discoveries that affect the scope or spec:
1. observed behavior in the current implementation that appears semantically important,
2. current spec clauses that are too vague, too narrow, too broad, stale, or contradicted by stronger evidence,
3. domain concepts discovered during modeling that need new vocabulary or state/transition treatment,
4. current-scope boundaries that should widen, narrow, split, or remain deferred,
5. evidence classification for each decision: proof/model-check, TraceCalc oracle, production conformance, replay witness, scale/measurement signal, historical no-loss input, OxFml upstream source, or unresolved.

Each decision must classify the outcome as:
1. OxCalc-owned spec patch,
2. OxFml handoff candidate,
3. implementation fault or mismatch,
4. TraceCalc oracle gap,
5. deferred open lane,
6. intentional non-carry-forward decision.

### 4.6 Historical No-Loss And Original-Idea Crosswalk

The pass should produce a historical crosswalk that maps original formal/theory ideas to current W033 handling:
1. layered formal model and executable reference machine,
2. fixed-point and stabilization theory,
3. DAG, SCC, dynamic topological maintenance, and cycle-region handling,
4. dynamic dependencies and calc-time overlays,
5. self-adjusting computation and trace-repair alternatives,
6. FEC/F3E transactional seam discipline,
7. MVCC, single-publisher, pinned-reader, and epoch/fence behavior,
8. visibility as scheduling metadata rather than semantic truth,
9. external streams, volatile inputs, provider failures, and callable publication watch lanes,
10. proof/pack/economics gating for advanced optimizations.

Each row should identify whether the idea is current scope, deferred, out of scope, or carried only as a guardrail.

### 4.7 Observable Surface And Refinement Relation

The pass should produce a first refinement packet that states:
1. the TraceCalc oracle state and transition surface for covered behavior,
2. the production/core-engine observed surface to compare against it,
3. the equality or compatibility relation for published values, rejects, publication epochs, dependency/runtime-effect facts, trace identity, replay identity, and relevant counters,
4. allowed internal differences such as scheduling, storage layout, batching, caching, and artifact ordering where ordering is declared non-semantic,
5. the conditions under which a mismatch is a spec gap, implementation fault, oracle gap, or intentional strategy difference requiring semantic-equivalence treatment.

### 4.8 Pack And Claim Mapping

The pass should produce a pack-oriented obligation map for:
1. `PACK.fec.commit_atomicity`,
2. `PACK.fec.reject_detail_replay`,
3. `PACK.fec.overlay_lifecycle`,
4. `PACK.concurrent.epochs`,
5. `PACK.dag.dynamic_dependency_bind_semantics`,
6. `PACK.overlay.fallback_economics`,
7. `PACK.visibility.policy_equivalence`,
8. `PACK.trace.forensic_plane`,
9. `PACK.replay.appliance`,
10. `PACK.diff.cross_engine.continuous`,
11. `PACK.reject.calculus`,
12. `PACK.scaling.signature`.

Each pack row must identify whether the claim is:
1. proof-backed,
2. model-check-backed,
3. replay-backed,
4. locally exercised only,
5. deferred with rationale.

## 5. Execution Sequence

### Phase A: Core Spec Review, Historical Crosswalk, And Authority Inventory

1. Freeze the exact OxCalc and OxFml source docs used by the pass.
2. Read through the initial OxCalc core-engine spec set and build a correction ledger.
3. Review the historical bootstrap formal/theory materials for no-loss intent coverage.
4. Build the first spec-evolution decision ledger from discovered ambiguities, mismatches, better domain models, and scope-pressure points.
5. Patch OxCalc-owned spec drift discovered by the pass, or record a deferred item with rationale when evidence is not yet strong enough.
6. Build the first claim-to-artifact matrix.
7. Mark OxFml clauses as imported, shared, or handoff-sensitive.
8. Mark OxFunc-facing items as opaque assumptions, separating ordinary function kernels from the `LET`/`LAMBDA` boundary carrier fragment.

### Phase B: Formal Leverage And Vocabulary Alignment

1. Normalize identity vocabulary across OxCalc and OxFml.
2. Align candidate, commit, reject, fence, trace, and runtime-effect names.
3. Align `LET`/`LAMBDA` local binding, lambda value, call identity, and prepared-call carrier vocabulary where OxFml and OxFunc meet.
4. Map each first-pass claim to at least one leverage family: refinement/operational semantics, TLA+, Lean, graph/incremental-computation theory, abstract interpretation/dataflow, or metamorphic/differential testing.
5. Identify any vocabulary split that could cause replay or proof drift.
6. File an OxFml handoff only if a normative seam change is required.

### Phase C: Lean Skeleton Widening

1. Replace the single Stage 1 Lean file with a module family or add adjacent modules.
2. Keep OxFml shapes abstract but typed.
3. Encode the first invariant statements before chasing broader proof depth.
4. Add a local runner path or documented check command for the Lean artifacts.

### Phase D: TLA+ Bridge Widening

1. Split or extend the Stage 1 TLA model into coordinator, FEC bridge, pinned-reader, overlay, and dynamic-dependency concerns.
2. Keep Stage 2 contention separate until Stage 1 bridge safety properties are stable.
3. Add smoke and non-smoke configs with bounded state spaces.
4. Archive model-check outputs as evidence only when the run configuration is declared.

### Phase E: Replay And Evidence Bridge

1. Map OxFml replay fixtures to OxCalc coordinator outcomes.
2. Add or identify cross-lane witness cases for candidate/publication, reject/no-publish, fence mismatch, dynamic dependency, overlay retention, and verified-clean behavior.
3. Expand TraceCalc as the executable correctness oracle for covered spec behavior, including oracle self-checks where the reference-machine surface itself is being widened.
4. Compare production/core-engine execution against TraceCalc for covered observable behavior before using performance or optimization evidence as semantic evidence.
5. Keep replay projection additive; do not replace native OxCalc or OxFml artifact meaning.
6. Preserve witness lifecycle and pack-eligibility state.

### Phase F: Pack And Capability Binding

1. Bind each formal claim to proof, TLA+, replay, and pack evidence.
2. State the highest current capability level honestly.
3. Keep `cap.C5.pack_valid` out of scope unless evidence exists.
4. Produce promotion notes only for claims backed by current artifacts.

### Phase G: Closure Audit And Successor Packetization

1. Run the repo-local validation commands relevant to docs, Rust, Lean, TLA+, and replay artifacts.
2. Fill the pre-closure checklist and self-audit for the declared workset scope.
3. Confirm that W033-touched spec clauses are mapped to current evidence or explicitly deferred.
4. Convert uncovered topics into beads, blockers, or handoff candidates.
5. Return control to the normal checkpoint-at-gates mode before any later proof or engine-realization lane.

## 6. Initial Non-Assumptions

1. OxFml's current public consumer facade is an input, not an OxCalc-owned API.
2. OxFml W026 residuals remain note-level unless live evidence exposes a concrete insufficiency.
3. Structured references, table context, immutable edit packets, fixture-host packets, and registered-external packets are admitted only where current OxFml notes say they are settled enough for first-wave planning.
4. Provider failure and callable publication remain watch lanes until coordinator-visible evidence exists.
5. Stage 2 concurrency promotion is not implied by Stage 1 model checking.
6. Current scale/performance instrumentation is evidence for measurement planning, not proof of semantic correctness.
7. OxFunc semantics are not modeled here beyond opaque packet assumptions already surfaced by OxFml, with the named `LET`/`LAMBDA` carrier fragment treated as a boundary obligation rather than a general function-semantic model.
8. TraceCalc is a correctness oracle only for the behavior it covers; uncovered behavior remains outside its authority until the corpus, reference-machine semantics, and comparison artifacts are widened.
9. A core-engine spec clause is not strengthened merely because the plan names a formal technique; the clause must be tied to proof, model-check, replay, pack, or deferred evidence.
10. Historical/bootstrap ideas are not resurrected as scope merely by appearing in archived documents; W033 must classify them through the no-loss crosswalk before they affect current scope.
11. Current implementation behavior is evidence, not normative authority; it must be classified before it changes the spec.
12. Current spec text is authoritative at entry, but not immutable; W033 may evolve OxCalc-owned specs through explicit evidence-backed or explicitly deferred decisions.

## 7. Immediate Open Lanes

1. Exact file layout for the widened Lean module family.
2. Whether Lean and TLA artifacts stay only in OxCalc or are later proposed for Green-owned promotion.
3. Cross-lane trace-schema split between OxCalc and OxFml.
4. Cross-lane fixture selection for the first minimal witness set.
5. Replay capability claim target for the first pass.
6. Pack rows that remain planning-only versus locally exercised.
7. Exact first artifact shape for the `LET`/`LAMBDA` OxFml/OxFunc boundary fragment.
8. Core-engine spec review ledger format and first document sweep order.
9. First refinement relation shape between TraceCalc and production/core-engine behavior.
10. First metamorphic test families that should graduate into TraceCalc or TreeCalc evidence.
11. Historical no-loss crosswalk format and first recovered/deferred idea set.
12. Spec-evolution decision ledger format and first classification taxonomy.

Resolved entry-layout decisions:
1. W033 spec-evidence packets live under `docs/spec/core-engine/w033-formalization/`.
2. Existing OxFml formal artifacts are referenced by path and upstream revision, not mirrored into OxCalc by default.
3. Existing OxCalc formal/replay roots remain the target for Lean, TLA+, replay, measurement, and run artifacts unless a later bead declares a more specific root first.

## 7A. Bead Rollout

The W033 execution graph now exists in `.beads/`.

Parent epic:
1. `calc-uri` - W033 OxCalc + OxFml core formalization and spec evolution.

Child bead sequence:
1. `calc-uri.1` - source authority and artifact layout.
2. `calc-uri.2` - core spec review ledger and sweep order.
3. `calc-uri.3` - spec-evolution decision taxonomy and first ledger.
4. `calc-uri.4` - historical no-loss crosswalk.
5. `calc-uri.5` - cross-lane authority and claim matrix.
6. `calc-uri.6` - object vocabulary and `LET`/`LAMBDA` carrier boundary.
7. `calc-uri.7` - observable surface and TraceCalc refinement relation.
8. `calc-uri.8` - TraceCalc oracle self-check first slice.
9. `calc-uri.9` - production conformance comparison first slice.
10. `calc-uri.10` - metamorphic and differential test families.
11. `calc-uri.11` - Lean module family first slice.
12. `calc-uri.12` - TLA bridge first slice.
13. `calc-uri.13` - replay and witness bridge.
14. `calc-uri.14` - pack and capability binding.
15. `calc-uri.15` - OxFml handoff and watch candidate packetization.
16. `calc-uri.16` - closure audit and successor packetization.

Current ready bead:
1. `calc-uri.1`.

## 8. Status

- execution_state: bead_rollout_created
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - parent epic `calc-uri` is open
  - source authority and artifact layout packet exists under `docs/spec/core-engine/w033-formalization/`
  - core-engine spec review ledger exists at `docs/spec/core-engine/w033-formalization/W033_CORE_SPEC_REVIEW_LEDGER.md`
  - spec-evolution decision ledger exists at `docs/spec/core-engine/w033-formalization/W033_SPEC_EVOLUTION_DECISION_LEDGER.md`
  - historical no-loss crosswalk exists at `docs/spec/core-engine/w033-formalization/W033_HISTORICAL_NO_LOSS_CROSSWALK.md`
  - authority and claim matrix exists at `docs/spec/core-engine/w033-formalization/W033_AUTHORITY_AND_CLAIM_MATRIX.md`
  - object vocabulary and `LET`/`LAMBDA` boundary packet exists at `docs/spec/core-engine/w033-formalization/W033_OBJECT_VOCABULARY_AND_LET_LAMBDA_BOUNDARY.md`
  - TraceCalc observable-surface/refinement packet exists at `docs/spec/core-engine/w033-formalization/W033_TRACECALC_REFINEMENT_PACKET.md`
  - TraceCalc oracle self-check first slice exists at `docs/spec/core-engine/w033-formalization/W033_TRACECALC_ORACLE_SELF_CHECK_FIRST_SLICE.md`
  - production/core-engine conformance first slice exists at `docs/spec/core-engine/w033-formalization/W033_PRODUCTION_CONFORMANCE_FIRST_SLICE.md`
  - metamorphic and differential test-family packet exists at `docs/spec/core-engine/w033-formalization/W033_METAMORPHIC_DIFFERENTIAL_TEST_FAMILIES.md`
  - Lean first-slice packet exists at `docs/spec/core-engine/w033-formalization/W033_LEAN_MODULE_FAMILY_FIRST_SLICE.md`
  - Lean artifact exists at `formal/lean/OxCalc/CoreEngine/W033FirstSlice.lean`
  - TLA bridge first-slice packet exists at `docs/spec/core-engine/w033-formalization/W033_TLA_BRIDGE_FIRST_SLICE.md`
  - no new replay bridge or pack artifacts have been authored by this plan
  - OxFml is included as a consumed formal/seam surface, but direct OxFml repo edits remain out of scope for this OxCalc pass
  - OxFunc semantic kernels are explicitly excluded, with only the narrow `LET`/`LAMBDA` carrier fragment admitted as an opaque boundary obligation
  - the refinement relation, graph/dataflow obligations, and metamorphic test families are planning-level only
- reviewed_inbound_observations: `../OxFml/docs/upstream/NOTES_FOR_OXCALC.md`
