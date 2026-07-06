# OxCalc Workset Register

Status: `active_register`
Date: 2026-05-27

## 1. Purpose
This is the living workset register for OxCalc.

It defines large work areas, dependency order, default rollout shape, and coarse work history.

This file is not an execution-status board.
It owns workset planning truth, not bead state or feature-status claims.

## 2. Planning-Surface Clarification
Planning and execution truth in OxCalc is split as follows:
1. [CHARTER.md](../CHARTER.md) owns mission, scope, and completion doctrine.
2. [OPERATIONS.md](../OPERATIONS.md) owns the operating model.
3. [SPEC.md](SPEC.md) indexes the active spec/design truth.
4. this register owns the ordered workset set and dependency shape.
5. `.beads/` owns epics, beads, readiness, blockers, in-progress state, dependencies, and closure.
6. current spec, seam, replay, oracle, and evidence artifacts support product claims.

Transition note:
1. `W032` is the doctrine-migration provenance packet for this shift.
2. `.beads/` is now bootstrapped and owns live execution-state truth.
3. this register is now authoritative for workset order, while closed or reached-gate worksets remain provenance packets rather than live trackers.

## 3. Use Rule
Use this document as:
1. the repo-local workset authority,
2. the source for future `workset -> epic -> bead` rollout,
3. the current ordered implementation map for active OxCalc work,
4. a coarse history of workset-level decisions.

Do not use this document as:
1. a second blocker tracker,
2. a substitute for the bead graph,
3. a reason to keep one status narrative per workset forever,
4. a duplicate of current seam, runtime, or evidence truth surfaces,
5. the place to answer broad product-feature status questions.

## 4. Register Contract
Each workset in this register carries:
1. stable workset id,
2. title,
3. purpose,
4. depends_on,
5. parent doctrine/spec surfaces,
6. primary upstream repo dependencies,
7. closure condition,
8. initial epic lanes,
9. rollout mode:
   - `execution_target`: expected to roll into epics/beads,
   - `tracking_anchor`: current authority/provenance packet that normally stays narrow unless reopened.

## 5. Sequencing Rule
The sequence below is the default expansion order for the live repo.

It does mean:
1. doctrine reset and workset-register adoption come before new broad execution under the bead model,
2. reached-gate predecessor packets remain authoritative provenance anchors,
3. active TreeCalc widening should now flow through explicit successor worksets and beads rather than through prose-only continuation.

### 5.1 Current Go-Forward Sequence (2026-07-04, rewritten around W062)
W050, W051, and W057 are closed and are the settled substrate the rest of this sequence builds on. The active go-forward sequence is now organized around **W062 (Ideal Engine Model Rework)** as the central program:

`W062 (R0 bootstrap -> R1 designs D1-D4 -> R2..R6 implementation -> R7 downstream adaptation)`

with the other worksets absorbed into, folded into, or explicitly sequenced against W062's waves:

1. **Absorbed into W062 D4 (document surface):** W059 (OxFml authored input and literal value authority). W059's discussion-stage scope becomes an input to the D4 design bead (calc-5kqg.7); it does not proceed independently.
2. **Absorbed into W062 D3 (workbook calculation):** W060 (calc-time reference representation and host reference system). W060's discussion-stage scope becomes an input to the D3 design bead (calc-5kqg.6).
3. **Becomes W062's strict-excel execution arm (D2/R3):** W061. Continues on its own epic (calc-kaqc) but sequenced inside W062's D2/R3 wave, gated by the OxFml W077 upstream lane (item 7).
4. **Co-owned design, not absorbed:** W055's general cycle-engine design bead (calc-9ouy.2) is paused pending joint authorship with W062 D3 (workbook-wide cycle detection). W055's implementation/host-contract/conformance beads (.3/.4/.9/.10) continue independently as tree/single-scope work; .5/.6/.7 continue as background lanes.
5. **In-flight, continuing:** W056's calc-4vs8.5.1 (CTRO intake for FEC reference-text dynamic references) and calc-4vs8.33 (non-table reference corpus/retained evidence intake) continue — tree-side work W062 does not redesign directly. The stash-collision risk noted during reconciliation was resolved 2026-07-04: stash@{0} dropped under the W062 R0 triage (calc-5kqg.1).
6. **Retention dependency re-affirmed:** W054 keeps its initial_slice_active tree-side scope; W062 R2/D1's grid-revision-retention work must define the grid-side retention class(es) W054 then owns — grid backing state carries zero retention coverage today.
7. **Parallel upstream lane, executed by this program:** OxFml W077 (strict Excel grid BindProfile and R1C1 identity, epic fml-7t6) plus the 3D sheet-range grammar production run as a parallel upstream lane executed directly under W062 (tracked by calc-5kqg.3), with a named entry gate before W062 R3 consumes them.
8. **Folded into W062 Direction 4 concurrency-prep, not closed:** W053 contributes its executor-building scope only after W062 R4 lands the concurrency-prep constraints. W053 remains the eventual workset that builds the concurrent executor; W062 does not build it.
9. **Re-sequenced to wait for W062:** W049 now also depends on W062, since W062 resettles the engine model after W050/W057; formalizing before W062 lands would formalize a shape about to be replaced.
10. **Unaffected, continuing independently:** W051 (closed, substrate only), W052 (sensitivity/derivative seam — omitted from the original W062 R0 list; corrected), W058 (retained-replay registry — cross-repo deliverable).

This sequence supersedes the 2026-05-14 and 2026-05-28 readings of this subsection. See §5.2 for worksets already retired to provenance status (unchanged by this pass).

### 5.2 Retired-To-Provenance Worksets (2026-05-14)
The worksets below are retired to `tracking_anchor` provenance status. They are no longer pending or forward-execution work; they remain authoritative provenance packets for the work they recorded and may be reopened only on an explicit concrete mismatch.

Retired in this pass:
1. **W027–W031 — TreeCalc structural/runtime substrate.** Dependency graph + invalidation closure, evaluator-backed candidate integration, runtime-derived effects + overlay closure, corpus oracle + first sequential baseline, assurance refresh. The substrate they produced (`StructuralSnapshot`, `TreeNodeId`, `Stage1RecalcTracker`, `TreeCalcCoordinator`, overlay lifecycle, `DependencyGraph`, `InvalidationClosure`, replay/witness families) is explicitly absorbed and preserved by the W050 rework (W050 §10.10 "what survives") — nothing is dropped, it is recomposed in new plumbing.
2. **W033–W046 — core formalization chain.** The W033→W046 release-grade verification tranche is superseded by **W049's formalization restart**. W049's purpose is precisely to restart formalization against a single authoritative implementation after CTRO and cycles landed; it explicitly inherits the W046 failure-mode punch list. The chain's residual obligations are not carried as separate pending worksets — they are subsumed by W049's restart scope.
3. **W047 — Calc-Time Rebinding Overlay design sweep.** CTRO is landed in the implementation core. W047's own scope reset already routed its formal/checker/sidecar/readiness-gate residue to W049 and its circular-dependency processing to W048. There is no forward W047 work.
4. **W048 — Circular Dependency Calculation Processing.** Already closed (`closed_single_host_scope`); listed here for completeness. W048 was the trigger for the W050 design rework, which now subsumes the forward calculation-model work.

Carry-forward items (NOT lost — already wired into W049's register entry, repeated here so they are not forgotten):
- The W046 successor obligations currently mislabeled against W047 beads `calc-aylq.1`–`.4` (Rust Tarjan and topological queue line proof / native proof-carrying trace sidecar enrichment / dynamic dependency positive publication refinement / semantic pack and operated-service readiness gate) transfer into W049. W049 now records them in its inherited-obligation table. When W049 starts, each obligation must be taken on, deferred, or dropped with a recorded reason.
- The W046 failure-mode punch list (avoid record-projection Lean theorems, smoke TLA models, silent-degrade checkers, predecessor-only binding registers, unbound evidence roots, terminology drift) is inherited by W049 per W049's purpose.

After W050 closure, the forward-pending set is the §5.1 sequence (`W051 -> W055 -> W054 -> W049 -> W052 -> W053`) plus the registered W051 successor side lane `W056`. W057's first representation scope is closed and now serves as predecessor evidence for W054 and W049. The next execution move is to continue W055 circular-reference widening, W054 bounded-memory retention over the W057 identities, or W049 formalization when its predecessors are accepted. The pre-rework worksets W020, W024, W025, W026, W032 were already `tracking_anchor` and are unchanged.

## 6. Active Workset Sequence

### W032 OxCalc Beads Migration And Light Doctrine Reorientation
1. purpose:
   migrate OxCalc from the older workset-plus-feature-register-plus-blocker execution doctrine to `docs/WORKSET_REGISTER.md` plus `workset -> epic -> bead`, while keeping the active tree light and preserving current truth surfaces.
2. depends_on:
   `W026`
3. parent_doctrine_and_spec_surfaces:
   `CHARTER.md`, `OPERATIONS.md`, `docs/worksets/W032_OXCALC_BEADS_MIGRATION_AND_LIGHT_DOCTRINE_REORIENTATION.md`
4. upstream_dependencies:
   none
5. closure_condition:
   `docs/WORKSET_REGISTER.md`, `docs/BEADS.md`, and `.beads/` exist, active doctrine docs point to them, blocker truth no longer belongs to `CURRENT_BLOCKERS.md`, and the current TreeCalc line is represented in the new ordered workset surface.
6. initial_epic_lanes:
   doctrine rewrite, register bootstrap, bead bootstrap, feature-map reduction
7. rollout_mode:
   `tracking_anchor`

### W020 OxFml Downstream Integration Round 01
1. purpose:
   remain the provenance owner for the first post-W018 OxFml downstream integration round and the narrower topic-matrix seam intake it established.
2. depends_on:
   `W018`
3. parent_doctrine_and_spec_surfaces:
   `docs/worksets/W020_OXFML_DOWNSTREAM_INTEGRATION_ROUND_01.md`, `docs/spec/core-engine/CORE_ENGINE_OXFML_SEAM.md`
4. upstream_dependencies:
   `OxFml`
5. closure_condition:
   remain a narrow provenance and seam-intake anchor unless a future downstream mismatch reopens it explicitly.
6. initial_epic_lanes:
   none unless reopened
7. rollout_mode:
   `tracking_anchor`

### W024 Broader Program-Scope Pack Promotion
1. purpose:
   remain the provenance owner for the broader program-grade replay/pack residual after the earlier pack-grade execution sequence.
2. depends_on:
   `W023`
3. parent_doctrine_and_spec_surfaces:
   `docs/worksets/W024_EXECUTION_SEQUENCE_J_BROADER_PROGRAM_SCOPE_PACK_PROMOTION.md`
4. upstream_dependencies:
   none
5. closure_condition:
   remain narrow unless the broader pack-promotion feature area is explicitly reopened.
6. initial_epic_lanes:
   none unless reopened
7. rollout_mode:
   `tracking_anchor`

### W025 TreeCalc Structural And Formula Substrate Widening
1. purpose:
   remain the provenance owner for the first TreeCalc structural/formula substrate floor beneath the OxCalcTree host-facing contract.
2. depends_on:
   `W024`
3. parent_doctrine_and_spec_surfaces:
   `docs/worksets/W025_TREECALC_STRUCTURAL_AND_FORMULA_SUBSTRATE_WIDENING.md`, `docs/spec/core-engine/CORE_ENGINE_TREECALC_SEMANTIC_COMPLETION_PLAN.md`
4. upstream_dependencies:
   `OxFml`
5. closure_condition:
   remain a reached-gate first-substrate provenance packet unless a concrete mismatch reopens it.
6. initial_epic_lanes:
   none unless reopened
7. rollout_mode:
   `tracking_anchor`

### W026 TreeCalc OxFml Bind, Reference, And Seam Intake
1. purpose:
   remain the provenance owner for the first executed TreeCalc consumed-seam packet.
2. depends_on:
   `W025`, `W020`
3. parent_doctrine_and_spec_surfaces:
   `docs/worksets/W026_TREECALC_OXFML_BIND_REFERENCE_AND_SEAM_INTAKE.md`, `docs/spec/core-engine/CORE_ENGINE_OXFML_SEAM.md`, `docs/spec/core-engine/CORE_ENGINE_TREECALC_SEMANTIC_COMPLETION_PLAN.md`
4. upstream_dependencies:
   `OxFml`
5. closure_condition:
   remain a reached-gate provenance packet for the first consumed seam floor unless later live insufficiency reopens it explicitly.
6. initial_epic_lanes:
   none unless reopened
7. rollout_mode:
   `tracking_anchor`

### W027 TreeCalc Dependency Graph And Invalidation Closure
1. purpose:
   replace planner-only dependency derivation with real dependency graph build and invalidation closure over TreeCalc structure plus consumed bind facts.
2. depends_on:
   `W025`, `W026`
3. parent_doctrine_and_spec_surfaces:
   `docs/worksets/W027_TREECALC_DEPENDENCY_GRAPH_AND_INVALIDATION_CLOSURE.md`, `docs/spec/core-engine/CORE_ENGINE_TREECALC_SEMANTIC_COMPLETION_PLAN.md`
4. upstream_dependencies:
   `OxFml`
5. closure_condition:
   structural dependency graph and reverse edges exist for the covered TreeCalc formula families, dependency identity is deterministic and replay-visible, and invalidation closure is explicit for structure edits and dependency changes in phase scope.
6. initial_epic_lanes:
   dependency graph realization, invalidation closure, replay-visible diagnostics
7. rollout_mode:
   `tracking_anchor` (retired to provenance 2026-05-14 — see §5.2)

### W028 TreeCalc Evaluator-Backed Candidate Result Integration
1. purpose:
   widen the TreeCalc line from current local candidate adaptation toward the first evaluator-backed candidate-result and publication integration floor.
2. depends_on:
   `W027`
3. parent_doctrine_and_spec_surfaces:
   `docs/worksets/W028_TREECALC_EVALUATOR_BACKED_CANDIDATE_RESULT_INTEGRATION.md`, `docs/spec/core-engine/CORE_ENGINE_TREECALC_SEMANTIC_COMPLETION_PLAN.md`
4. upstream_dependencies:
   `OxFml`
5. closure_condition:
   evaluator-backed candidate-result integration is explicit, replay-visible, and coherent with the current TreeCalc publication path for declared first-phase scope.
6. initial_epic_lanes:
   candidate integration, publication/reject mapping, replay/evidence
7. rollout_mode:
   `tracking_anchor` (retired to provenance 2026-05-14 — see §5.2)

### W029 TreeCalc Runtime-Derived Effects And Overlay Closure
1. purpose:
   widen the current runtime-derived effect family floor into the first honest TreeCalc runtime-derived and overlay-closure packet.
2. depends_on:
   `W027`, `W028`
3. parent_doctrine_and_spec_surfaces:
   `docs/worksets/W029_TREECALC_RUNTIME_DERIVED_EFFECTS_AND_OVERLAY_CLOSURE.md`, `docs/spec/core-engine/CORE_ENGINE_TREECALC_SEMANTIC_COMPLETION_PLAN.md`
4. upstream_dependencies:
   `OxFml`
5. closure_condition:
   runtime-derived family and overlay closure are explicit for the declared TreeCalc-first phase without reopening W026 first-packet seam truth.
6. initial_epic_lanes:
   runtime-derived realization, overlay closure, replay/explain widening
7. rollout_mode:
   `tracking_anchor` (retired to provenance 2026-05-14 — see §5.2)

### W030 TreeCalc Corpus Oracle And First Sequential Baseline
1. purpose:
   widen the TreeCalc fixture/oracle lane into the first broader sequential baseline beyond the current local pre-oracle floor.
2. depends_on:
   `W027`, `W028`, `W029`
3. parent_doctrine_and_spec_surfaces:
   `docs/worksets/W030_TREECALC_CORPUS_ORACLE_AND_FIRST_SEQUENTIAL_BASELINE.md`, `docs/spec/core-engine/CORE_ENGINE_TREECALC_SEMANTIC_COMPLETION_PLAN.md`
4. upstream_dependencies:
   none
5. closure_condition:
   the first broader TreeCalc corpus/oracle/baseline packet exists with deterministic emitted evidence for the declared sequential scope.
6. initial_epic_lanes:
   corpus widening, oracle/baseline execution, emitted evidence
7. rollout_mode:
   `tracking_anchor` (retired to provenance 2026-05-14 — see §5.2)

### W031 TreeCalc Assurance Refresh And Residual Packetization
1. purpose:
   refresh the TreeCalc assurance and residual packetization lane after the first broader runtime and corpus widening packets land.
2. depends_on:
   `W027`, `W028`, `W029`, `W030`
3. parent_doctrine_and_spec_surfaces:
   `docs/worksets/W031_TREECALC_ASSURANCE_REFRESH_AND_RESIDUAL_PACKETIZATION.md`, `docs/spec/core-engine/CORE_ENGINE_TREECALC_SEMANTIC_COMPLETION_PLAN.md`
4. upstream_dependencies:
   none
5. closure_condition:
   assurance refresh and residual packetization are explicit and evidence-backed for the declared TreeCalc-first sequential floor.
6. initial_epic_lanes:
   assurance refresh, residual narrowing, closure evidence
7. rollout_mode:
   `tracking_anchor` (retired to provenance 2026-05-14 — see §5.2)

### W033 OxCalc + OxFml Core Formalization Pass
1. purpose:
   plan and execute a comprehensive OxCalc-owned formalization and spec-evolution pass over OxCalc core-engine semantics plus the OxFml evaluator/FEC/F3E surfaces consumed by OxCalc; treat current implementation behavior and current specs as evidence surfaces rather than immutable final targets; include the narrow `LET`/`LAMBDA` OxFml/OxFunc carrier fragment while keeping general OxFunc semantic kernels out of scope.
2. depends_on:
   `W031`, `W032`, `W020`, `W026`
3. parent_doctrine_and_spec_surfaces:
   `docs/worksets/W033_OXCALC_OXFML_CORE_FORMALIZATION_PASS.md`, `docs/spec/core-engine/CORE_ENGINE_OXCALC_OXFML_FORMALIZATION_PASS_PLAN.md`, `docs/spec/core-engine/CORE_ENGINE_FORMALIZATION_AND_ASSURANCE.md`, `docs/spec/core-engine/CORE_ENGINE_TRACECALC_REFERENCE_MACHINE.md`, `docs/spec/core-engine/CORE_ENGINE_REALIZATION_ROADMAP.md`, `docs/spec/core-engine/CORE_ENGINE_OXFML_SEAM.md`
4. upstream_dependencies:
   `OxFml`
5. closure_condition:
   the cross-lane formalization scope is decomposed into explicit epics and beads, current core-engine specs are reviewed and corrected or deferred, spec-evolution decisions are classified, original formal/theory ideas have no-loss coverage, OxCalc-owned/OxFml-owned/shared/OxFunc-opaque/`LET`-`LAMBDA` carrier clauses are mapped, TraceCalc oracle claims are separated from production/core-engine conformance claims, first-pass Lean/TLA+/replay/pack obligations are explicit, and any OxFml seam pressure is packetized as handoff or watch-lane work.
6. initial_epic_lanes:
   core spec review and correction ledger, spec-evolution decision ledger, historical no-loss crosswalk, authority inventory, vocabulary alignment, formal leverage mapping, observable-surface/refinement packet, Lean model widening, TLA+ model widening, replay and witness bridge, pack and capability binding, OxFml handoff/watch lane, closure audit
7. rollout_mode:
   `tracking_anchor` (retired to provenance 2026-05-14 — see §5.2)

### W034 Core Formalization Deepening And Implementation Verification
1. purpose:
   deepen the post-W033 formalization from first-pass and successor-slice evidence into a broader proof/model/replay/conformance tranche that can support later implementation improvement, pack-gate decisions, and Stage 2 precondition assessment without treating the current specs or current implementation as immutable targets.
2. depends_on:
   `W033`
3. parent_doctrine_and_spec_surfaces:
   `docs/worksets/W034_CORE_FORMALIZATION_DEEPENING_AND_IMPLEMENTATION_VERIFICATION.md`, `docs/spec/core-engine/w034-formalization/W034_FORMALIZATION_DEEPENING_PLAN.md`, `docs/spec/core-engine/CORE_ENGINE_OXCALC_OXFML_FORMALIZATION_PASS_PLAN.md`, `docs/spec/core-engine/CORE_ENGINE_FORMALIZATION_AND_ASSURANCE.md`, `docs/spec/core-engine/CORE_ENGINE_TRACECALC_REFERENCE_MACHINE.md`, `docs/spec/core-engine/CORE_ENGINE_OXFML_SEAM.md`
4. upstream_dependencies:
   `OxFml`
5. closure_condition:
   W034 residual obligations are explicitly mapped from W033 and current inbound OxFml notes, TraceCalc oracle coverage is widened for the declared tranche, optimized/core-engine conformance is checked against that oracle surface, Lean and TLA model families are deepened with checked artifacts, pack/capability and continuous scale gates state their actual evidence consequence, Stage 2 contention remains unpromoted unless its gates are satisfied, and any spec evolution, implementation fault, OxFml handoff pressure, or successor lane is recorded rather than left as prose memory.
6. initial_epic_lanes:
   residual obligation and authority ledger, TraceCalc oracle deepening, optimized/core-engine conformance widening, Lean proof-family deepening, TLA model-family and contention preconditions, pack/capability and continuous scale gate binding, closure audit and successor packetization
7. rollout_mode:
   `tracking_anchor` (retired to provenance 2026-05-14 — see §5.2)

### W035 Core Formalization Proof And Assurance Hardening
1. purpose:
   continue after W034 by converting bounded proof/model/replay/conformance evidence into stronger formal proof, continuous assurance, implementation hardening, and promotion-gate evidence without overclaiming full verification.
2. depends_on:
   `W034`
3. parent_doctrine_and_spec_surfaces:
   `docs/worksets/W035_CORE_FORMALIZATION_PROOF_AND_ASSURANCE_HARDENING.md`, `docs/spec/core-engine/w034-formalization/W034_CLOSURE_AUDIT_AND_SUCCESSOR_PACKET.md`, `docs/spec/core-engine/CORE_ENGINE_FORMALIZATION_AND_ASSURANCE.md`, `docs/spec/core-engine/CORE_ENGINE_TRACECALC_REFERENCE_MACHINE.md`, `docs/spec/core-engine/CORE_ENGINE_OXFML_SEAM.md`
4. upstream_dependencies:
   `OxFml`
5. closure_condition:
   W034 residuals are assigned to executable evidence or implementation work, TraceCalc oracle matrices are widened, implementation conformance gaps are hardened or reclassified, Lean assumptions are discharged or explicitly mapped, TLA exploration moves beyond routine smoke where practical, continuous assurance criteria are stronger than single-run timing, pack/Stage 2 readiness is reassessed with machine-readable decisions, and no promotion claim is made without direct gate evidence.
6. initial_epic_lanes:
   residual proof-obligation ledger, TraceCalc oracle matrix expansion, implementation conformance hardening, Lean assumption discharge, TLA non-routine exploration, continuous assurance and cross-engine differential gate, pack/Stage 2 readiness reassessment, closure audit
7. rollout_mode:
   `tracking_anchor` (retired to provenance 2026-05-14 — see §5.2)

### W036 Core Formalization Verification Closure Expansion
1. purpose:
   continue after W035 by converting no-promotion blockers and bounded evidence into a deeper verification tranche aimed at TraceCalc coverage-closure criteria, optimized/core-engine conformance closure, stronger Lean/TLA proof obligations, concrete Stage 2 equivalence evidence, continuous assurance operation, and pack-grade replay readiness without treating W035 evidence as final proof.
2. depends_on:
   `W035`
3. parent_doctrine_and_spec_surfaces:
   `docs/worksets/W036_CORE_FORMALIZATION_VERIFICATION_CLOSURE_EXPANSION.md`, `docs/spec/core-engine/w035-formalization/W035_CLOSURE_AUDIT_AND_SUCCESSOR_PACKET.md`, `docs/spec/core-engine/CORE_ENGINE_FORMALIZATION_AND_ASSURANCE.md`, `docs/spec/core-engine/CORE_ENGINE_TRACECALC_REFERENCE_MACHINE.md`, `docs/spec/core-engine/CORE_ENGINE_OXFML_SEAM.md`
4. upstream_dependencies:
   `OxFml`
5. closure_condition:
   W035 open lanes and no-promotion blockers are explicitly mapped to W036 obligations, TraceCalc coverage closure criteria are machine-readable, optimized/core-engine conformance gaps are resolved or carried as blockers, Lean/TLA proof/model artifacts are expanded without overclaiming total verification, independent-evaluator diversity and cross-engine differential evidence state actual limits, continuous-assurance operation/history is evidenced or blocked, pack/Stage 2 promotion decisions state exact evidence consequence, and the closure audit maps the active full-formalization objective to concrete artifacts before any completion claim.
6. initial_epic_lanes:
   residual coverage and promotion-blocker ledger, TraceCalc coverage closure criteria and matrix expansion, optimized/core-engine conformance closure plan and first fixes, Lean theorem coverage expansion, TLA Stage 2 partition and scheduler equivalence model, independent evaluator diversity and cross-engine differential harness, continuous assurance operation and history window, pack-grade replay and capability promotion gate reassessment, closure audit and successor/full-verification decision
7. rollout_mode:
   `tracking_anchor` (retired to provenance 2026-05-14 — see §5.2)

### W037 Core Formalization Full-Verification Promotion Gates
1. purpose:
   continue after W036 by converting remaining no-promotion blockers into direct full-verification, implementation-conformance, direct OxFml evaluator, operated-assurance, pack-grade replay, and Stage 2 promotion-gate evidence where possible, while allowing specs and scope to evolve from new evidence rather than treating earlier specs or implementations as immutable targets.
2. depends_on:
   `W036`
3. parent_doctrine_and_spec_surfaces:
   `docs/worksets/W037_CORE_FORMALIZATION_FULL_VERIFICATION_PROMOTION_GATES.md`, `docs/spec/core-engine/w036-formalization/W036_CLOSURE_AUDIT_AND_SUCCESSOR_PACKET.md`, `docs/spec/core-engine/CORE_ENGINE_FORMALIZATION_AND_ASSURANCE.md`, `docs/spec/core-engine/CORE_ENGINE_TRACECALC_REFERENCE_MACHINE.md`, `docs/spec/core-engine/CORE_ENGINE_OXFML_SEAM.md`
4. upstream_dependencies:
   `OxFml`
5. closure_condition:
   W036 residuals are assigned to W037 proof, implementation, replay, direct-OxFml, operated-service, pack, deferral, or successor obligations; TraceCalc observable closure and optimized/core-engine conformance are directly evidenced or blocked; direct OxFml evaluator and narrow `LET`/`LAMBDA` seam evidence is exercised or named as a blocker; Lean/TLA proof/model claims are checked and assumption-bounded; Stage 2 deterministic replay and partition promotion criteria include semantic-equivalence evidence; operated assurance/service claims use operated artifacts rather than simulation; pack/C5 decisions state exact evidence consequence; and the closure audit maps the full-formalization objective to artifacts before any release-grade claim.
6. initial_epic_lanes:
   residual full-verification and promotion-gate ledger, TraceCalc observable closure and multi-reader replay, optimized/core-engine conformance implementation closure, direct OxFml evaluator and `LET`/`LAMBDA` seam evidence, Lean/TLA proof and model closure inventory, Stage 2 deterministic replay and partition promotion criteria, operated continuous assurance and cross-engine service pilot, pack-grade replay governance and C5 candidate decision, closure audit and full-verification release decision
7. rollout_mode:
   `tracking_anchor` (retired to provenance 2026-05-14 — see §5.2)

### W038 Core Formalization Release-Grade Closure Hardening
1. purpose:
   continue after W037 by converting release-grade residual blockers into direct TraceCalc authority, optimized/core-engine conformance, proof/model assumption discharge, Stage 2 replay-equivalence, operated-service, independent-diversity, OxFml seam/watch, pack-grade replay, and C5 evidence where possible, while allowing specs and scope to evolve from new evidence rather than treating earlier specs or implementations as immutable targets.
2. depends_on:
   `W037`
3. parent_doctrine_and_spec_surfaces:
   `archive/worksets-w038-w045/W038_CORE_FORMALIZATION_RELEASE_GRADE_CLOSURE_HARDENING.md`, `docs/spec/core-engine/w037-formalization/W037_CLOSURE_AUDIT_AND_FULL_VERIFICATION_RELEASE_DECISION.md`, `docs/spec/core-engine/CORE_ENGINE_FORMALIZATION_AND_ASSURANCE.md`, `docs/spec/core-engine/CORE_ENGINE_TRACECALC_REFERENCE_MACHINE.md`, `docs/spec/core-engine/CORE_ENGINE_OXFML_SEAM.md`
4. upstream_dependencies:
   `OxFml`
5. closure_condition:
   W037 residuals are assigned to W038 proof, implementation, replay, service, diversity, pack, handoff/watch, deferral, or successor obligations; TraceCalc authority exclusions are discharged, accepted, or kept as exact blockers; optimized/core-engine conformance blockers are fixed, diff-promoted, spec-evolved, or carried without declared-gap match promotion; Lean/TLA assumptions and totality boundaries are checked or blocked; Stage 2 partition replay and semantic-equivalence evidence exists before policy promotion; operated assurance, cross-engine service, and alert/quarantine claims use operated artifacts; independent evaluator diversity is backed by independent implementation authority; OxFml formatting and `LET`/`LAMBDA` seam rows remain current; pack/C5 decisions state exact evidence consequence; and the closure audit maps release-grade objectives to artifacts before any promotion claim.
6. initial_epic_lanes:
   residual release-grade obligation ledger and objective map, TraceCalc oracle authority and authority-exclusion discharge, optimized core-engine conformance blocker closure and fixes, proof-model assumption discharge and totality boundary hardening, Stage 2 partition replay and semantic-equivalence execution, operated assurance alert-quarantine and cross-engine service, independent evaluator diversity and OxFml seam watch closure, pack-grade replay governance and C5 release decision, closure audit and release-grade verification decision
7. rollout_mode:
   `tracking_anchor` (retired to provenance 2026-05-14 — see §5.2)

### W039 Core Formalization Release-Grade Successor Closure
1. purpose:
   continue after W038 by attacking the remaining release-grade blockers directly: optimized/core exact blockers, Lean/TLA totality, Stage 2 production partition policy, operated assurance services, retained history, independent evaluator diversity, OxFml seam breadth including current W073 typed-only conditional-formatting metadata, callable metadata, pack-grade replay governance, C5 reassessment, and release-grade decision.
2. depends_on:
   `W038`
3. parent_doctrine_and_spec_surfaces:
   `archive/worksets-w038-w045/W039_CORE_FORMALIZATION_RELEASE_GRADE_SUCCESSOR_CLOSURE.md`, `archive/w038-formalization/W038_CLOSURE_AUDIT_AND_RELEASE_GRADE_VERIFICATION_DECISION.md`, `docs/spec/core-engine/CORE_ENGINE_FORMALIZATION_AND_ASSURANCE.md`, `docs/spec/core-engine/CORE_ENGINE_TRACECALC_REFERENCE_MACHINE.md`, `docs/spec/core-engine/CORE_ENGINE_OXFML_SEAM.md`
4. upstream_dependencies:
   `OxFml`
5. closure_condition:
   W038 residuals are mapped to W039 obligations; optimized/core exact blockers are fixed, directly evidenced, spec-evolved, or retained without declared-gap match promotion; Lean/TLA totality and proof/model rows distinguish discharged, bounded, assumed, and blocked cases; Stage 2 production partition and pack-grade replay evidence is present before policy promotion; operated assurance, retained-history, alert/quarantine, and cross-engine differential claims use service artifacts; independent evaluator diversity uses independent implementation authority; OxFml W073 typed-only formatting, broad seam breadth, public consumer surfaces, and callable metadata are evidenced, handed off, or blocked; pack/C5 and release-grade decisions state exact evidence consequence before any promotion claim.
6. initial_epic_lanes:
   residual successor obligation ledger and promotion-readiness map, optimized core exact blocker implementation closure, Lean/TLA totality and proof-model closure tranche, Stage 2 production partition policy and replay governance, operated assurance service and retained history substrate, independent evaluator row set and cross-engine diversity, OxFml seam breadth and callable metadata closure, pack-grade replay governance and C5 reassessment, closure audit and release-grade decision
7. rollout_mode:
   `tracking_anchor` (retired to provenance 2026-05-14 — see §5.2)

### W040 Core Formalization Release-Grade Direct Verification
1. purpose:
   continue after W039 by converting exact residual release-grade blockers into direct implementation, proof, model, service, diversity, OxFml/callable, pack/C5, and release-grade evidence where possible, while preserving exact blockers where direct evidence is still insufficient.
2. depends_on:
   `W039`
3. parent_doctrine_and_spec_surfaces:
   `archive/worksets-w038-w045/W040_CORE_FORMALIZATION_RELEASE_GRADE_DIRECT_VERIFICATION.md`, `archive/w039-formalization/W039_CLOSURE_AUDIT_AND_RELEASE_GRADE_DECISION.md`, `docs/spec/core-engine/CORE_ENGINE_FORMALIZATION_AND_ASSURANCE.md`, `docs/spec/core-engine/CORE_ENGINE_TRACECALC_REFERENCE_MACHINE.md`, `docs/spec/core-engine/CORE_ENGINE_OXFML_SEAM.md`
4. upstream_dependencies:
   `OxFml`
5. closure_condition:
   W039 residuals are mapped to W040 direct-verification obligations; optimized/core exact blockers are fixed, directly evidenced, spec-evolved, or retained without declared-gap match promotion; Rust totality/refinement and Lean/TLA verification claims are checked or blocked; Stage 2 production policy claims have production-relevant partition soundness and observable-result invariance evidence; operated service claims use runnable service artifacts; independent evaluator claims use independent implementation authority; OxFml W073 typed-only formatting, broad seam breadth, public consumer surfaces, and callable metadata are evidenced, handed off, or blocked; pack/C5 and release-grade decisions state exact evidence consequence before any promotion claim.
6. initial_epic_lanes:
   residual direct-verification obligation map, optimized core exact blocker fixes and differentials, Rust totality and refinement proof tranche, Lean/TLA full-verification discharge tranche, Stage 2 production policy and equivalence implementation, operated assurance and retained-history service implementation, independent evaluator implementation and operated differential, OxFml seam breadth and callable metadata implementation, pack-grade replay governance and C5 promotion decision, closure audit and release-grade verification decision
7. rollout_mode:
   `tracking_anchor` (retired to provenance 2026-05-14 — see §5.2)

### W041 Core Formalization Release-Grade Successor Verification
1. purpose:
   continue after W040 by converting the remaining release-grade blockers into direct implementation, proof, model, operated-service, diversity, OxFml/callable, pack/C5, and release-grade verification evidence where possible, while preserving exact blockers where direct evidence is still insufficient.
2. depends_on:
   `W040`
3. parent_doctrine_and_spec_surfaces:
   `archive/worksets-w038-w045/W041_CORE_FORMALIZATION_RELEASE_GRADE_SUCCESSOR_VERIFICATION.md`, `archive/w040-formalization/W040_CLOSURE_AUDIT_AND_RELEASE_GRADE_VERIFICATION_DECISION.md`, `docs/spec/core-engine/CORE_ENGINE_FORMALIZATION_AND_ASSURANCE.md`, `docs/spec/core-engine/CORE_ENGINE_TRACECALC_REFERENCE_MACHINE.md`, `docs/spec/core-engine/CORE_ENGINE_OXFML_SEAM.md`
4. upstream_dependencies:
   `OxFml`
5. closure_condition:
   W040 residual lanes are mapped to W041 obligations; optimized/core residual blockers are fixed, directly evidenced, spec-evolved, or retained without declared-gap match promotion; Rust totality/refinement and Lean/TLA verification claims are checked or blocked; Stage 2 production policy claims have production-relevant partition soundness and observable-result invariance evidence; operated service claims use operated service artifacts and retained lifecycle evidence; independent evaluator claims use independent implementation authority and breadth; OxFml W073 typed-only formatting, broad display/publication, public migration, callable metadata, and callable carrier sufficiency are evidenced, handed off, or blocked; pack/C5 and release-grade decisions state exact evidence consequence before any promotion claim.
6. initial_epic_lanes:
   residual release-grade successor obligation map, optimized core residual blocker implementation and differential closure, Rust totality refinement and panic-boundary discharge, Lean/TLA full-verification and fairness discharge, Stage 2 production analyzer and pack-equivalence proof tranche, operated assurance retained-history and alert-dispatch service tranche, independent evaluator breadth and operated differential service tranche, OxFml broad display/publication and callable-carrier closure, pack-grade replay governance and C5 reassessment, closure audit and release-grade verification decision
7. rollout_mode:
   `tracking_anchor` (retired to provenance 2026-05-14 — see §5.2)

### W042 Core Formalization Release-Grade Evidence Closure Expansion
1. purpose:
   continue after W041 by converting the remaining release-grade blockers into stronger direct conformance, proof, model, operated-service, diversity, OxFml/callable, pack/C5, and release-grade verification evidence where possible, while preserving exact blockers where direct evidence is still insufficient.
2. depends_on:
   `W041`
3. parent_doctrine_and_spec_surfaces:
   `archive/worksets-w038-w045/W042_CORE_FORMALIZATION_RELEASE_GRADE_EVIDENCE_CLOSURE_EXPANSION.md`, `archive/w041-formalization/W041_CLOSURE_AUDIT_AND_RELEASE_GRADE_VERIFICATION_DECISION.md`, `docs/spec/core-engine/CORE_ENGINE_FORMALIZATION_AND_ASSURANCE.md`, `docs/spec/core-engine/CORE_ENGINE_TRACECALC_REFERENCE_MACHINE.md`, `docs/spec/core-engine/CORE_ENGINE_OXFML_SEAM.md`
4. upstream_dependencies:
   `OxFml`
5. closure_condition:
   W041 residual lanes are mapped to W042 obligations; optimized/core counterpart conformance and callable metadata projection are fixed, directly evidenced, spec-evolved, or retained without declared-gap match promotion; Rust totality/refinement and Lean/TLA verification claims are checked or blocked; Stage 2 and pack-equivalence claims have production-relevant partition soundness, scheduler equivalence, and observable-result invariance evidence; operated service claims use operated service artifacts and retained lifecycle evidence; independent evaluator and mismatch quarantine claims use independent implementation authority and service behavior; OxFml W073 typed-only formatting, broad display/publication, public migration, callable carrier sufficiency, registered-external callable projection, and provider-failure/callable-publication semantics are evidenced, handed off, or blocked; pack/C5 and release-grade decisions state exact evidence consequence before any promotion claim.
6. initial_epic_lanes:
   residual release-grade closure obligation ledger, optimized core counterpart conformance and callable metadata projection, Rust totality refinement and core panic-boundary closure, Lean/TLA fairness and full-verification expansion, Stage 2 production analyzer and pack-grade equivalence closure, operated assurance retained-history retained-witness and alert service closure, independent evaluator breadth mismatch quarantine and operated differential service, OxFml public migration callable carrier and registered-external closure, pack-grade replay governance and C5 reassessment, closure audit and release-grade verification decision
7. rollout_mode:
   `tracking_anchor` (retired to provenance 2026-05-14 — see §5.2)

### W043 Core Formalization Release-Grade Proof And Operated-Service Integration
1. purpose:
   continue after W042 by converting remaining release-grade blockers into stronger direct proof, implementation, operated-service, independent-evaluator, OxFml/callable, pack/C5, and release-grade verification evidence where possible, while preserving exact blockers where direct evidence is still insufficient.
2. depends_on:
   `W042`
3. parent_doctrine_and_spec_surfaces:
   `archive/worksets-w038-w045/W043_CORE_FORMALIZATION_RELEASE_GRADE_PROOF_AND_OPERATED_SERVICE_INTEGRATION.md`, `archive/w042-formalization/W042_CLOSURE_AUDIT_AND_RELEASE_GRADE_VERIFICATION_DECISION.md`, `docs/spec/core-engine/CORE_ENGINE_FORMALIZATION_AND_ASSURANCE.md`, `docs/spec/core-engine/CORE_ENGINE_TRACECALC_REFERENCE_MACHINE.md`, `docs/spec/core-engine/CORE_ENGINE_OXFML_SEAM.md`
4. upstream_dependencies:
   `OxFml`
5. closure_condition:
   W042 residual lanes are mapped to W043 obligations; optimized/core broad conformance and callable metadata blockers are fixed, directly evidenced, spec-evolved, or retained without declared-gap match promotion; Rust totality/refinement and panic-free core claims are checked or blocked; Lean/TLA verification and unbounded fairness claims are checked or blocked; Stage 2 and pack-equivalence claims have production-relevant partition soundness, scheduler equivalence, and observable-result invariance evidence; operated service claims use operated artifacts and retained lifecycle evidence; independent evaluator and mismatch quarantine claims use implementation authority and service behavior; OxFml W073 typed-only formatting, broad display/publication, public migration, callable carrier sufficiency, registered-external callable projection, provider-failure/callable-publication semantics, and current inbound observation-ledger lanes are evidenced, handed off, watched, or blocked; pack/C5 and release-grade decisions state exact evidence consequence before any promotion claim.
6. initial_epic_lanes:
   residual release-grade proof-service obligation map, optimized core broad conformance and callable metadata closure, Rust totality refinement and panic-free core proof frontier, Lean/TLA full-verification and unbounded fairness discharge, Stage 2 production partition analyzer and scheduler equivalence, operated assurance retained-history witness SLO and alert service, independent evaluator breadth mismatch quarantine and differential service, OxFml public migration formatting callable and registered-external seam, pack-grade replay governance and C5 release reassessment, closure audit and release-grade verification decision
7. rollout_mode:
   `tracking_anchor` (retired to provenance 2026-05-14 — see §5.2)

### W044 Core Formalization Release-Grade Blocker Burn-Down And Service Proof Closure
1. purpose:
   continue after W043 by converting the remaining release-grade no-promotion blockers into direct implementation, proof/model, operated-service, independent-evaluator, OxFml migration, scaling, and pack-governance evidence where possible, while preserving exact blockers where direct evidence is still insufficient.
2. depends_on:
   `W043`
3. parent_doctrine_and_spec_surfaces:
   `archive/worksets-w038-w045/W044_CORE_FORMALIZATION_RELEASE_GRADE_BLOCKER_BURN_DOWN_AND_SERVICE_PROOF_CLOSURE.md`, `archive/w043-formalization/W043_CLOSURE_AUDIT_AND_RELEASE_GRADE_VERIFICATION_DECISION.md`, `docs/spec/core-engine/CORE_ENGINE_FORMALIZATION_AND_ASSURANCE.md`, `docs/spec/core-engine/CORE_ENGINE_TRACECALC_REFERENCE_MACHINE.md`, `docs/spec/core-engine/CORE_ENGINE_OXFML_SEAM.md`
4. upstream_dependencies:
   `OxFml`
5. closure_condition:
   W043 residual blockers are mapped to W044 obligations; optimized/core dynamic transition breadth, callable metadata projection, Rust totality/refinement, panic-free core boundaries, Lean/TLA verification, unbounded fairness, Stage 2 production partition-analyzer soundness, scheduler equivalence, pack-grade replay equivalence, operated services, retained-history and retained-witness lifecycle, retention SLO, independent evaluator breadth, mismatch quarantine, OxFml W073 typed-rule request construction, public migration, callable carrier sufficiency, registered-external callable projection, provider-failure/callable-publication semantics, release-scale replay/performance evidence, pack/C5, and release-grade decisions are evidenced, handed off, watched, blocked, or promoted only by direct evidence.
6. initial_epic_lanes:
   residual release-grade blocker reclassification and promotion-contract map, optimized core dynamic transition and callable metadata implementation tranche, Rust totality refinement and panic-surface proof expansion, Lean/TLA unbounded fairness and full-verification proof expansion, Stage 2 production partition analyzer and scheduler equivalence implementation, operated continuous assurance retained-history witness SLO and alert service, independent evaluator breadth mismatch quarantine and differential service implementation, OxFml public migration typed formatting callable and registered-external uptake, release-scale replay performance and scaling evidence under semantic guards, pack-grade replay governance service and C5 reassessment, closure audit and release-grade verification decision
7. rollout_mode:
   `tracking_anchor` (retired to provenance 2026-05-14 — see §5.2)

### W045 Core Formalization Release-Grade Service And Cross-Repo Uptake Verification
1. purpose:
   continue after W044 by converting the remaining release-grade no-promotion blockers into direct implementation, proof/model, operated-service, independent-evaluator, OxFml public-surface, W073 downstream-uptake, continuous-scale, pack-governance, and release-grade evidence where possible, while preserving exact blockers where direct evidence is still insufficient.
2. depends_on:
   `W044`
3. parent_doctrine_and_spec_surfaces:
   `archive/worksets-w038-w045/W045_CORE_FORMALIZATION_RELEASE_GRADE_SERVICE_AND_CROSS_REPO_UPTAKE_VERIFICATION.md`, `archive/w044-formalization/W044_CLOSURE_AUDIT_AND_RELEASE_GRADE_VERIFICATION_DECISION.md`, `docs/spec/core-engine/CORE_ENGINE_FORMALIZATION_AND_ASSURANCE.md`, `docs/spec/core-engine/CORE_ENGINE_TRACECALC_REFERENCE_MACHINE.md`, `docs/spec/core-engine/CORE_ENGINE_OXFML_SEAM.md`
4. upstream_dependencies:
   `OxFml`
5. closure_condition:
   W044 residual lanes are mapped to W045 obligations; current OxFml public-surface and W073 typed-only formatting updates are reviewed; optimized/core counterpart coverage and callable metadata projection are fixed, directly evidenced, spec-evolved, or retained without declared-gap match promotion; Rust totality/refinement and Lean/TLA verification claims are checked or blocked; Stage 2 and pack-equivalence claims have production-relevant partition soundness, scheduler equivalence, and observable-result invariance evidence; operated service claims use operated artifacts and retained lifecycle evidence; independent evaluator and mismatch quarantine claims use independent implementation authority and service behavior; W073 downstream typed-rule request construction, broad OxFml display/publication, public migration, callable carrier sufficiency, registered-external callable projection, provider-failure/callable-publication semantics, continuous scale assurance, pack/C5, and release-grade decisions state exact evidence consequence before any promotion claim.
6. initial_epic_lanes:
   residual release-grade successor obligation and current OxFml intake map, optimized core counterpart coverage and callable metadata projection closure, Rust totality refinement and panic-surface hardening, Lean/TLA verification fairness and totality discharge, Stage 2 production partition and pack-grade equivalence service evidence, operated assurance retained-history retained-witness SLO service implementation, independent evaluator breadth mismatch quarantine and operated differential service, OxFml public surface W073 downstream typed formatting callable and registered-external uptake, continuous release-scale assurance and semantic regression service, pack-grade replay governance service and C5 reassessment, closure audit and release-grade verification decision
7. rollout_mode:
   `tracking_anchor` (retired to provenance 2026-05-14 — see §5.2)

### W046 Core Formalization Engine Semantic Proof Spine
1. purpose:
   continue after W045 by redirecting the core formalization effort toward the calculation engine semantic proof spine: dependency graph construction, reverse-edge consistency, SCC/cycle classification, invalidation closure, rebind requirements, recalc state transitions, evaluation order, working-value reads, OxFml runtime adaptation, rejection, publication, and TraceCalc refinement. Proof-service, release-grade, C5, operated-service, pack-governance, independent-evaluator, scale, and promotion-readiness work remain supporting evidence layers over that semantic spine.
2. depends_on:
   `W045`
3. parent_doctrine_and_spec_surfaces:
   `docs/worksets/W046_CORE_FORMALIZATION_ENGINE_SEMANTIC_PROOF_SPINE.md`, `archive/w045-formalization/W045_CLOSURE_AUDIT_AND_RELEASE_GRADE_VERIFICATION_DECISION.md`, `docs/showcase/oxcalc_w033_w045_engine_formalization_review_catalog.md`, `docs/spec/core-engine/CORE_ENGINE_FORMALIZATION_AND_ASSURANCE.md`, `docs/spec/core-engine/CORE_ENGINE_TRACECALC_REFERENCE_MACHINE.md`, `docs/spec/core-engine/CORE_ENGINE_OXFML_SEAM.md`
4. upstream_dependencies:
   `OxFml`
5. closure_condition:
   the showcase engine catalog is promoted into OxCalc spec surfaces; graph/reverse-edge/SCC, invalidation/rebind, recalc-state, evaluation-order, working-value, and TraceCalc-refinement lanes emit Lean/TLA targets, checked artifacts, replay evidence, or exact blockers; the algebraic-effects seam idea is incorporated into the spec model as an effect-signature/handler-law layer or explicitly deferred; current OxFml public-surface and W073 typed-only formatting updates are reviewed; proof-service, release-grade, C5, pack-grade replay, Stage 2, operated-service, independent-evaluator, OxFml/callable, release-scale, and promotion-readiness consequences are classified only after direct semantic evidence or exact blockers are recorded.
6. initial_epic_lanes:
   redirect showcase finding uptake engine semantic catalog and effect-signature plan, dependency graph reverse-edge and SCC model, invalidation soft-reference dynamic-reference and rebind model, recalc tracker transition pre/post model, evaluation-order and working-value read-discipline model, TraceCalc refinement kernel and TreeCalc/CoreEngine replay binding, OxFml seam LET/LAMBDA formatting/publication and callable-boundary model, proof-service and evidence-classifier coverage ledger recast over the semantic spine, scale/performance semantic-regression signatures, Stage 2 pack-governance C5 operated-service independent-evaluator and release-readiness consequence reassessment, closure audit semantic-spine coverage decision and successor routing
7. rollout_mode:
   `tracking_anchor` (retired to provenance 2026-05-14 — see §5.2)

### W047 Calc-Time Rebinding Overlay Design Sweep
1. purpose:
   continue after W046 by restoring Calc-Time Rebinding Overlay as a central engine design concept rather than a bolt-on: runtime-derived dependency-shape changes discovered during evaluation, including dynamic references, region membership, dynamic array/spill resizing, and candidate-overlay cycle creation or release, must be modeled as effective-graph overlay changes with explicit SCC/cycle classification, frontier repair, fallback/reject, candidate, publication, trace, and proof obligations.
2. depends_on:
   `W046`
3. parent_doctrine_and_spec_surfaces:
   `docs/worksets/W047_CALC_TIME_REBINDING_OVERLAY_DESIGN_SWEEP.md`, `docs/spec/core-engine/w047-ctro/W047_HISTORICAL_NO_LOSS_CTRO_CROSSWALK.md`, `docs/spec/core-engine/w047-ctro/W047_EFFECTIVE_GRAPH_OVERLAY_AND_FRONTIER_REPAIR_SEMANTICS.md`, `docs/spec/core-engine/w047-ctro/W047_CTRO_SCENARIO_MATRIX_AND_TRACE_FACTS.md`, `docs/spec/core-engine/w047-ctro/W047_IMPLEMENTATION_ROADMAP_AND_SUCCESSOR_GATES.md`, `docs/spec/core-engine/w047-ctro/W047_DYNAMIC_DEPENDENCY_POSITIVE_PUBLICATION_EVIDENCE.md`, `docs/worksets/W003_STAGE1_COORDINATOR_AND_PUBLICATION_BASELINE.md`, `docs/worksets/W004_INCREMENTAL_RECALC_AND_OVERLAY_BASELINE.md`, `docs/worksets/W007_LEAN_FACING_STATE_OBJECTS_AND_TRANSITION_BOUNDARY_PLAN.md`, `docs/worksets/W008_TLA_COORDINATOR_PUBLICATION_AND_FENCE_SAFETY_MODEL_PLAN.md`, `docs/worksets/W009_REPLAY_AND_PACK_BINDING_FOR_STAGE1_SEAM_AND_COORDINATOR_BEHAVIOR.md`, `docs/worksets/W010_EXPERIMENT_REGISTER_AND_MEASUREMENT_SCHEMA_PLANNING.md`, `docs/worksets/W012_TRACECALC_REFERENCE_MACHINE_AND_CONFORMANCE_ORACLE.md`, `docs/spec/core-engine/CORE_ENGINE_RECALC_AND_INCREMENTAL_MODEL.md`, `docs/spec/core-engine/CORE_ENGINE_OXFML_SEAM.md`, `docs/spec/core-engine/w046-formalization/`
4. upstream_dependencies:
   `OxFml` for dynamic reference, region/spill, and runtime-derived effect seam implications if W047 discovers current shared contracts are too narrow.
5. closure_condition:
   historical no-loss crosswalk is complete; Calc-Time Rebinding Overlay is integrated into core recalc, graph, OxFml seam, TraceCalc, TreeCalc, and design surfaces (formal/checker/sidecar surfaces are explicitly **not** changed under W047 — see scope reset 2026-05-10 in W047 §1); static, dynamic switch, unresolved dynamic, downstream dependent, spill expansion/contraction, structural cycle, CTRO-created cycle, CTRO cycle release, and fallback scenarios are covered or exact-blocked; effective graph/frontier repair/cycle-policy semantics are specified at design level (bounded-model/checker validation deferred to W049); implementation lands the CTRO phase in the engine core; circular dependency calculation processing is routed to W048 before formalization; pack/C5/operated/release promotion gates pass through W049's evidence layer.
6. initial_epic_lanes:
   historical no-loss design sweep and CTRO doctrine, effective graph frontier repair and shared cycle-policy semantics (design-level), scenario matrix and TraceCalc/TreeCalc evidence plan, implementation/evidence roadmap and successor gates, dynamic dependency positive publication implementation refinement and CTRO phase landing.
7. rollout_mode:
   `tracking_anchor` (retired to provenance 2026-05-14 — see §5.2; CTRO landed in the implementation core; formal/checker/sidecar/readiness-gate residue already routed to W049, circular-dependency processing already routed to W048 — see W047 §1 scope reset and §10 deferred bead path).

### W048 Circular Dependency Calculation Processing
1. purpose:
   execute circular dependency calculation processing end to end before successor formalization/deepening: structural cycles, CTRO-created cycles, cycle release/re-entry, downstream invalidation, non-iterative no-publication policy, Excel bit-exact iterative calculation behavior, Excel comparison probes, materialized graph facts, TraceCalc reference behavior, TreeCalc optimized/core behavior, W048 formal definitions/models/checker targets, deterministic test corpus, and profile-gated innovation opportunities must be handled as calculation behavior rather than deferred proof-only work. Reopened scope correction: the predecessor W048 closure narrowed the target to conservative non-iterative Stage 1 and is superseded for full W048.
2. depends_on:
   `W047`
3. parent_doctrine_and_spec_surfaces:
   `docs/worksets/W048_CIRCULAR_DEPENDENCY_CALCULATION_PROCESSING.md`, `docs/spec/core-engine/w048-cycles/`, `docs/worksets/W047_CALC_TIME_REBINDING_OVERLAY_DESIGN_SWEEP.md`, `docs/spec/core-engine/w047-ctro/`, `docs/spec/core-engine/CORE_ENGINE_RECALC_AND_INCREMENTAL_MODEL.md`
4. upstream_dependencies:
   Excel behavior observation inputs for circular-reference comparison; OxFml handoff only if W048 discovers shared evaluator-facing contract changes are needed.
5. closure_condition:
   W048 can route successor work only after it records the selected non-iterative cycle policy, binds materialized graph requirements to TraceCalc/TreeCalc artifacts or exact blockers, exercises or exact-blocks structural and CTRO-created cycle fixtures, specifies cycle release/re-entry and downstream invalidation behavior, records Excel observation disposition for the core probe set, implements or exactly blocks Excel-compatible bit-exact iterative calculation with user acceptance, executes the declared circular-reference corpus or records exact blockers, introduces W048-owned formal definitions/models/checker targets grounded in artifacts, captures innovation profiles separately from default Excel-match behavior, and validates W048 tooling without Python.
6. initial_epic_lanes:
   predecessor beads: Excel circular-reference probes (`calc-zci1.1`), materialized dependency graph layers and sidecars (`calc-zci1.2`), TraceCalc reference cycle implementation (`calc-zci1.3`), iterative-profile algorithm decision and Excel disposition (`calc-zci1.4`), W048 formal definitions and proof/model artifacts (`calc-zci1.5`), TreeCalc optimized cycle implementation (`calc-zci1.6`), circular-reference test corpus and conformance runs (`calc-zci1.7`), innovation opportunity ledger and experimental profiles (`calc-zci1.8`). Reopened beads: scope repair (`calc-zci1.9`), non-Python tooling migration (`calc-zci1.10`), Excel bit-exact observation suite (`calc-zci1.11`), Excel-match iterative profile specification (`calc-zci1.12`), TraceCalc iterative implementation (`calc-zci1.13`), TreeCalc iterative implementation (`calc-zci1.14`), full conformance and closure audit (`calc-zci1.15`), root/report-cell evidence packet (`calc-zci1.16`), numeric-prior initial-vector packet (`calc-zci1.17`), blank/text/error prior packet (`calc-zci1.18`), cross-version blocker packet (`calc-zci1.19`), and multithread variant packet (`calc-zci1.20`).
7. rollout_mode:
   `closed_single_host_scope` (child beads `calc-zci1.1` through `calc-zci1.20` are closed; prior closure audit at `docs/test-runs/core-engine/w048-closure-audit-001/w048_closure_audit_summary.json` is superseded by `docs/spec/core-engine/w048-cycles/W048_REOPEN_SCOPE_AUDIT_AND_REPAIR_PLAN.md`; final accepted-scope audit is `docs/spec/core-engine/w048-cycles/W048_SINGLE_HOST_SCOPE_ACCEPTANCE_AND_FINAL_AUDIT.md`; `calc-zci1.16` is cleared by `w048-excel-root-report-002`; `calc-zci1.19` is closed by explicit user acceptance of single-host Excel scope, with cross-version behavior documented as a limitation).

### W050 Calculation Model Rework — Unified Recalc Session, Plan Templates, And Engine Improvement Moves
1. purpose:
   the umbrella workset for bringing the OxCalc + OxFml + LET/LAMBDA calculation model into alignment with current best thinking. It reached the production formula-authority gate by removing OxCalc-local prepared-callable compatibility projections, replacing synthetic A1 input transport with OxFml formal input bindings, consuming OxFml runtime prepared-package/template-hole/formal-reference identity, and routing non-W050 successor work to W049/W051/W054/W052. LET/LAMBDA are no longer a special side path for this boundary: ordinary cell formulas now use the same session/prepared-package shape at the OxCalc/OxFml seam.
2. depends_on:
   `W048`, `OxFml formula/evaluator seam`
3. parent_doctrine_and_spec_surfaces:
   `docs/worksets/W050_OXCALC_OXFML_FORMULA_AUTHORITY_REWORK.md`, `docs/spec/core-engine/CORE_ENGINE_OXFML_SEAM.md`, `docs/spec/core-engine/CORE_ENGINE_OXFML_MINIMAL_UPSTREAM_HOST_INTERFACES.md`, `docs/spec/core-engine/CORE_ENGINE_TREECALC_SEMANTIC_COMPLETION_PLAN.md`, `docs/spec/core-engine/CORE_ENGINE_OXCALC_OXFML_FORMALIZATION_PASS_PLAN.md`, `docs/spec/core-engine/CORE_ENGINE_RECALC_AND_INCREMENTAL_MODEL.md`, `../Foundation/CORE_ENGINE_FORMAL_MODEL.md`, `docs/worksets/W048_CIRCULAR_DEPENDENCY_CALCULATION_PROCESSING.md`, `docs/worksets/W047_CALC_TIME_REBINDING_OVERLAY_DESIGN_SWEEP.md`, `src/oxcalc-core/src/treecalc.rs`, `src/oxcalc-core/src/formula.rs`, `src/oxcalc-core/src/upstream_host.rs`, OxFml seam specs in `../OxFml/docs/spec/`
4. upstream_dependencies:
   `OxFml` — the session-shaped consumer surface expressed over `oxfml_core::consumer::runtime` without reopening the frozen `OxFml_V1` consumer-facade contract; plan-template identity surfacing (`shape_key`/`dispatch_skeleton_key`/`plan_template_key`); `NumericalReductionPolicy`/`ErrorAlgebra` threading through semantic plan and evaluation context (OxFunc cooperation required for kernel-side reduction discipline); capability-set hole admission. Three cross-repo handoff packets drive these: `HANDOFF_CALC_002` (recalc session + plan templates), `HANDOFF_CALC_003` (numerical reduction + error algebra), `HANDOFF_CALC_004` (capability-set hole admission). Returned value/effect/dependency surfaces remain opaque to OxCalc, especially dynamic arrays/spills and volatile functions.
5. closure_condition:
   reached-gate closure for the W050 production scope: runtime seam, prepared-package identity, formal input transport, bridge metadata, subscription/topic envelopes, correctness-floor selectors, derivation trace, value-cache/scheduling evidence, and successor routing are validated. Fixture-only upstream-host scaffolding and legacy fixture/scale quarantine surfaces remain test support, not production formula-authority debt.
6. initial_epic_lanes:
   parent epic `calc-cwpl`; seven lanes — Lane A removal (`calc-cwpl.A1`–`.A4`), Lane B new seam (`calc-cwpl.B1`–`.B6`), Lane C plan-template identity (`calc-cwpl.C1`–`.C4`), Lane D external invalidation (`calc-cwpl.D1`–`.D4`), Lane E correctness floor (`calc-cwpl.E1`–`.E3`), Lane F performance/observability (`calc-cwpl.F1`–`.F3`), Lane G forward scaffolding (`calc-cwpl.G1`–`.G3`); cross-repo handoff beads `calc-cwpl.H1`–`.H3`. Predecessor beads `calc-cwpl.1`–`.6` are mapped into the lane structure (W050 §6). Phasing: Wave 1 lands Lanes B + C concurrently with A following; Wave 2 lands Lanes D + E in parallel; Wave 3 lands Lane F; Lane G lands cheapest alongside Lane C.
7. rollout_mode:
   `closed` (parent epic `calc-cwpl` and child lane epics are closed for the declared W050 production formula-authority scope; successor work is routed to W051, W055, W054, W049, and W052).

### W049 Core Engine Formalization Restart After CTRO And Cycles
1. purpose:
   resume formal verification work on the calculation engine after the W047 CTRO phase has landed in the implementation core, W048 has grounded circular dependency behavior, and W057 has closed the workspace revision/snapshot-layer representation cutover. W049 inherits the W046 failure-mode punch list: avoid record-projection Lean theorems, smoke TLA models, silent-degrade checkers, predecessor-only binding registers, unbound evidence roots, and terminology drift. Formalize around a single authoritative implementation rather than producing a parallel decorative layer. Per the go-forward sequence in §5.1, W049 is sequenced after W050, W051, W054, and W057 so that it formalizes the *settled* post-rework engine — the unified recalc-session / prepared-formula package / plan-template / formal-input model with Excel-scope sparse-reader, bounded-memory discipline, and explicit workspace revision/snapshot layers — rather than the pre-rework per-formula-packet engine that W050 demolishes; formalizing before W050 or before W057 would be wasted work.
2. depends_on:
   `W047`, `W048`, `W050`, `W051`, `W054`, `W057`, `W062` (W062 resettles the engine model again; W049 must formalize the post-W062 architecture, not the pre-W062 shape — re-sequenced 2026-07-04 per W062 R0.)
3. parent_doctrine_and_spec_surfaces:
   `docs/worksets/W049_CORE_ENGINE_FORMALIZATION_RESTART_AFTER_CTRO_AND_CYCLES.md`, `docs/worksets/W050_OXCALC_OXFML_FORMULA_AUTHORITY_REWORK.md`, `docs/worksets/W048_CIRCULAR_DEPENDENCY_CALCULATION_PROCESSING.md`, `docs/worksets/W047_CALC_TIME_REBINDING_OVERLAY_DESIGN_SWEEP.md`, `docs/worksets/W046_CORE_FORMALIZATION_ENGINE_SEMANTIC_PROOF_SPINE.md`, `docs/spec/core-engine/w046-formalization/`, `docs/spec/core-engine/w047-ctro/`
4. upstream_dependencies:
   `OxFml` — W049 formalizes the post-W050 engine, which consumes the session-shaped OxFml seam (`ensure_prepared` / `invoke`, plan-template identity, typed invocation outcomes); the formalization models those consumed surfaces as imported/assumed contracts rather than re-deriving them. To be re-evaluated when the W049 plan is finalized.
5. closure_condition:
   W049 closes when a selected set of formal/checker targets constrain real implementation or replay evidence, inherited W046 obligations are dispositioned, stale/unbound evidence roots are bound or retired, and product/evidence/open/formal status is reported separately.
6. initial_epic_lanes:
   terminology and authority glossary, inherited W046 obligation disposition, evidence-root audit, selected proof/checker targets, implementation-bound artifacts, cycle/CTRO intake, closure review.
7. rollout_mode:
   `queued_successor`

### W051 Sparse Range Readers And Defined-Entry Semantics
1. purpose:
   Excel-compatible sparse range readers and defined-entry semantics on top of the closed W050 formula-authority seam, plus the first TreeCalc reference-collection compatibility lane for DNA TreeCalc `@CHILDREN` / `.*` style ordered reference arrays. Implements `SparseRangeReader` — `declared_extent`, `defined_cardinality`, `defined_iter`, `read_at(coord) -> Defined(EvalValue) | Blank`, `contains(coord)` — and activates the kernel-side sparse argument-preparation profile, currently reserved upstream as `SparseRangeAccepted(extent_class, cardinality_class)` / `SparseIteratorOk`-equivalent metadata. Recommended first function group is `SUM`, `COUNT`, `COUNTA`, and `COUNTBLANK`, before widening to `AVERAGE`, `MIN`, `MAX`, and criteria-family functions. The TreeCalc lane is not generic virtual data: it exists so `=SUM(@CHILDREN)` can be parsed and bound by OxFml through a generic OxCalc-supplied host formula context (`dialect_id = oxcalc.treecalc-v1`, `capability_profile_id = host-capabilities:treecalc-v1`, `resolution_rule_version = treecalc-host-resolution:v1`), resolved/lowered by OxCalc against the OxCalc-owned tree model, and invoked in OxFunc as ordinary values/arrays or an opaque `ReferenceLike` plus resolver, without OxFunc parsing TreeCalc text or tree structure. The first reference-collection carrier is `TreeCalcReferenceCollection::ChildrenV1`, correlated to OxFml host-reference handles while child membership, sibling order, member value edges, and invalidation remain OxCalc-owned semantics. The cell-value model is two-state: `Defined` covers all assigned values including empty-string `""`; `Blank` covers both never-assigned and assigned-then-cleared cells, which Excel treats identically at the cell-value level. Sheet-structural state that persists across clear operations — used range, cell formatting, conditional-format ranges, data-validation rules, comments — belongs in sparse sheet structural management and may back efficient range bounds, but it is not part of the formula value-level `SparseRangeReader.read_at` state. Rich data types beyond Excel-compatible sheet/reference semantics are parked in `docs/spec/core-engine/CORE_ENGINE_FUTURE_IDEAS_RICH_AND_VIRTUAL_DATA.md`, not in W051. Name/call precedence for built-in/UDF/defined-name/defined-name-`LAMBDA` collisions is routed to OxFml `W074-CALC005` evidence before product semantics are frozen.
2. depends_on:
   `W050` (runtime seam, formal input/reference transport, prepared-package identity, and sparse range identity reservation)
3. parent_doctrine_and_spec_surfaces:
   `docs/worksets/W051_SPARSE_RANGE_READERS_AND_DEFINED_ENTRY_SEMANTICS.md`, `docs/worksets/W050_OXCALC_OXFML_FORMULA_AUTHORITY_REWORK.md`, `docs/spec/core-engine/CORE_ENGINE_OXFML_SEAM.md`, `docs/spec/core-engine/CORE_ENGINE_OXCALCTREE_CONSUMER_INTERFACE_AND_HOST_CONTRACT_V1.md`, `docs/spec/core-engine/CORE_ENGINE_FUTURE_IDEAS_RICH_AND_VIRTUAL_DATA.md`, `docs/handoffs/HANDOFF_CALC_005_OXFML_HOST_CONTEXT_AND_NAMESPACE_RESOLUTION.md`, `../OxFml/docs/handoffs/HANDOFF_CALC_005_OXFML_RECEIPT.md`
4. upstream_dependencies:
   `OxFunc` owns sparse argument-preparation metadata/profile and aggregation-kernel consumption. `OxFml` owns sparse binding through semantic plan, runtime prepared identity, replay projection, and W074 evidence for name/call precedence. `OxCalc` owns the concrete backing reader/adapter, TreeCalc host-reference syntax, resolver output, namespace/caller context identity, reference-collection carrier, set-membership dependency, and invalidation facts.
5. closure_condition:
   W051 closes its first scope when the sparse reader API is specified and implemented in OxCalc, OxFml carries sparse/reference inputs through runtime/replay, OxFunc consumes the reader or resolver-backed reference path for the first function group, replay evidence shows declared extent and defined-entry order, the TreeCalc `SUM(@CHILDREN)` equivalent is exercised through `TreeCalcReferenceCollection::ChildrenV1` without OxFunc TreeCalc-specific logic, and dense materialization is ruled out for the declared large-range path. A labeled `treecalc_eager_values_fallback.v1` path may be used only as a stepping stone and does not close the reference-preserving TreeCalc scenario by itself.
6. initial_epic_lanes:
   shared reader contract, coordinate/extent model, replay shape, OxCalc reader adapter, generic OxCalc-to-OxFml host formula context and bind-output shape, exact `@CHILDREN` / `.*` source-span and source-token preservation, host namespace version and caller-context identity, W074-derived function/UDF/defined-name/defined-name-LAMBDA shadowing matrix, `TreeCalcReferenceCollection::ChildrenV1` resolver/reader carrier, current-member value dependency and set-membership dependency shape, first OxFunc function group, OxFml runtime/replay threading, integration evidence.
7. rollout_mode:
   `closed_first_scope` (epic `calc-yptj` and children are closed for the
   declared OxCalc W051 first product scope: sparse reader API, worksheet
   adapter, `ChildrenV1` sparse reader, generic TreeCalc host context,
   source-preserving host-reference bind output, OxFml sparse
   reference-values binding, and OxFunc first aggregate consumption. Product
   DnaTreeCalc formula-text bridge activation remained successor work and is
   now reduced by W056 `calc-4vs8.7` for free-standing `@CHILDREN` / `.*`,
   `calc-4vs8.8` / `calc-4vs8.10` for caller-supplied resolved-base
   `base.@CHILDREN` / `base.*`, and DnaTreeCalc commit `6611684` for the first
   receiving-side corpus slice. Full TreeCalc references, structured tables,
   and W074 name/call precedence remain successor work.)

Restated 2026-07-04 (W062 R0): W051's closed first scope is untouched by W062 — the sparse-reader API contract is a dependency W062 D2/D3 build on top of, not a surface W062 redesigns.

### W056 TreeCalc Full Reference And Table Lowering
1. purpose:
   widen W051's first `TreeCalcReferenceCollection::ChildrenV1` carrier pattern into the full TreeCalc reference and structured table-lowering scope. W056 owns the admitted TreeReference variants beyond children, dependency edges, invalidation facts, dynamic rebind, host namespace versioning, caller context identity, table row/column/header/totals dependencies, and correlation to OxFml host-reference handles while preserving OxFml integration through generic host context only. Boundary correction 2026-05-24: OxCalc-authored formula-text parse/rewrite surfaces are migration defects, not product architecture. OxFml must parse and bind formula text through declarative host syntax rules in `HostFormulaContext`, then emit source-preserving generic host-reference and structured-reference packets. OxCalc resolves those packets against `OxCalcTreeContext` into `TreeReference` carriers/readers, dependency descriptors, invalidation facts, and prepared-identity inputs. Focused children, ordered-selector, recursive-tail, sibling-offset, reference-literal, walk-up, ancestor-anchor, bracket-escaped path, meta-node invisibility, and table active slices have now been reissued through this OxFml-owned parse/bind or unresolved-host-name bind seam; broader W004 families still need activation/retained evidence before W056 closure. The table lane still consumes generic OxFml table-context packet and `StructuredReferenceBindRecord` projections; OxCalc owns node-table projection, sparse readers, lifecycle/dependency facts, and invalidation semantics. The table promotion lane is closed for the declared node-associated TreeCalc table scope through `calc-4vs8.34` through `calc-4vs8.38`, with later table hardening spines remaining active. New correction beads `calc-4vs8.33.4`, OxFml `fml-f64`, and DnaTreeCalc `dtc-z0i.16` own the deletion of OxCalc-local formula parsing and the host-hook replacement work.
2. depends_on:
   `W051`; OxFml `W074-CALC005` for name/call precedence where bare names or callables are involved, with the current W051/W056 TreeCalc host-name mapping consumed from OxFml commit `4a55709` and `HANDOFF_CALC_005_W074_NAME_CALL_FREEZE.md`; OxFml structured-reference/table packet work where table grammar/bind semantics are involved.
3. parent_doctrine_and_spec_surfaces:
   `docs/worksets/W056_TREECALC_FULL_REFERENCE_AND_TABLE_LOWERING.md`, `docs/worksets/W051_SPARSE_RANGE_READERS_AND_DEFINED_ENTRY_SEMANTICS.md`, `docs/spec/core-engine/CORE_ENGINE_OXCALCTREE_CONSUMER_INTERFACE_AND_HOST_CONTRACT_V1.md`, `docs/spec/core-engine/CORE_ENGINE_OXFML_SEAM.md`, `docs/handoffs/HANDOFF_CALC_005_OXFML_HOST_CONTEXT_AND_NAMESPACE_RESOLUTION.md`, `docs/handoffs/HANDOFF_CALC_006_W056_TABLE_ROLLOUT_COORDINATION.md`, `docs/upstream/NOTES_FOR_OXFML.md`, `docs/upstream/NOTES_FOR_OXXLPLAY.md`
4. upstream_dependencies:
   `OxFml` owns formula grammar, generic host-context consumption, structured-reference grammar/bind normalization, and W074 name/call evidence. `OxFunc` owns function semantics and reference/value argument admission. OxCalc owns TreeCalc model custody, reference resolution, dependency/invalidation lowering, host namespace versioning, caller context identity, and table dependency lowering.
5. closure_condition:
   W056 closes only for a declared full-reference/table-lowering scope when admitted TreeCalc reference variants have implemented carriers or typed exclusions, dependency and invalidation facts are replay-visible, dynamic rebind and namespace/caller-context versioning invalidate prepared identities deterministically, structured table row/column/header/totals dependencies lower from the generic OxFml packet without OxCalc parsing formula language, OxFml/OxFunc integration uses only public generic host-context/reference/value surfaces, and any cross-repo handoffs are explicit.
6. initial_epic_lanes:
   parent epic `calc-4vs8`; `calc-4vs8.1` full TreeReference inventory and host-reference correlation, `calc-4vs8.3` dependency/reverse-edge widening plus invalidation/dynamic rebind/namespace/caller-context work, `calc-4vs8.12` ordered selector collection carriers, `calc-4vs8.33.4` deletion of OxCalc formula-text parse/rewrite surfaces and migration to OxFml-owned host syntax parse/bind, `calc-4vs8.2`/`calc-4vs8.9` structured table packet and bind-record intake plus row/column/header/totals dependency lowering, `calc-4vs8.21` through `calc-4vs8.29` first node-associated table spine for table-node snapshot projection, table reference readers, per-row formula runtime, update/invalidation scenarios, retained table evidence, and Excel update-oracle intake; `calc-4vs8.34` through `calc-4vs8.38` second-pass table product-promotion spine for lifecycle/callback freeze, function breadth, matched replay/value-wire intake, and final scoped audit; `calc-4vs8.39` through `calc-4vs8.43` third-pass full intended table support spine for empty-body support, function implementation evidence, lifecycle bridge acceptance, namespace/anchor/workspace semantics, and final full-table audit; `calc-4vs8.44` through `calc-4vs8.56` fourth-pass comprehensive table completion spine for whole-system ownership, virtual Excel-anchor identity, generic OxFml packet contract, OxCalc resolver/namespace versioning, full table `ReferenceLike` readers, row-context prepared identity, dependency/invalidation matrix, DnaTreeCalc activation, OxXlPlay oracle construction, OxReplay retained evidence, UDF/VBA/XLL impact, cross-repo rollout coordination, and dynamic table reference rebind/`INDIRECT` semantics; `calc-4vs8.57` through `calc-4vs8.63` fifth-pass node-table hardening spine for current-state architecture revalidation, abstraction consolidation, lifecycle execution matrix, ReferenceLike/function/UDF integration closure, oracle/replay/value-wire convergence, cross-repo rollout reconciliation, and final completion audit; remaining W056 execution also focuses on non-table reference corpus/evidence under `calc-4vs8.33` and `calc-4vs8.5`, plus W074/W036 intake and handoff watch where future extensions require it.
7. rollout_mode:
   `in_progress_successor`

Disposition 2026-07-04 (W062 R0): in-flight beads calc-4vs8.5.1 (CTRO intake for FEC reference-text dynamic references) and calc-4vs8.33 (full non-table reference corpus/retained evidence intake) continue — both are tree-side CTRO/reference-resolution work that W062 D1-D3 do not redesign directly. Stash-collision risk resolved 2026-07-04: stash@{0} was dropped (W062 R0 triage, calc-5kqg.1, verdict drop-all — see docs/worksets/W062_R0_STASH_TRIAGE.md).

### W057 Workspace Revision And Snapshot-Layer Rework
1. purpose:
   rework OxCalc's internal workspace representation around explicit immutable snapshot layers and discardable contextual views. W057 adopts domain names rather than red/green implementation names: `WorkspaceRevision` is the durable tuple of `StructureSnapshot`, `NodeInputSnapshot`, and `NamespaceSnapshot`; `FormulaBindingSnapshot` and `DependencyShapeSnapshot` are derived typed-fact layers; `PublicationSnapshot` and `RuntimeOverlaySet` are publication/runtime layers; `WorkspaceRevisionView` and `EvaluationContextView` are disposable contextual views. The workset exists because W056 exposed that value edits, formula edits, literal/formula transitions, CTRO runtime effects, dependency-shape publication, and retention identities must not continue to blur through structural machinery or OxCalc-side formula interpretation.
2. depends_on:
   `W056` epoch/snapshot correction and formula-authority retraction lessons; `W050` formula-authority boundary; `W054` retention identity requirements as a downstream consumer rather than as the owner of the representation rewrite.
3. parent_doctrine_and_spec_surfaces:
   `docs/worksets/W057_WORKSPACE_REVISION_AND_SNAPSHOT_LAYER_REWORK.md`, `CHARTER.md`, `docs/spec/core-engine/CORE_ENGINE_ARCHITECTURE.md`, `docs/spec/core-engine/CORE_ENGINE_STATE_AND_SNAPSHOTS.md`, `docs/spec/core-engine/CORE_ENGINE_OXFML_SEAM.md`, `docs/worksets/W056_TREECALC_FULL_REFERENCE_AND_TABLE_LOWERING.md`, `docs/worksets/W054_BOUNDED_MEMORY_AND_PINNED_EPOCH_GC.md`
4. upstream_dependencies:
   `OxFml` owns formula grammar, parse, bind, prepared identity, dynamic/runtime-reference declarations, diagnostics, and any formula-language semantic facts. W057 may require additional typed OxFml/FEC facts, but it must not fill gaps by inspecting OxCalc-side formula text or function names. `OxFunc` remains owner of function semantics and value/kernel behavior.
5. closure_condition:
   W057 closes its first scope when `WorkspaceRevision`, `StructureSnapshot`, `NodeInputSnapshot`, and `NamespaceSnapshot` are specified and implemented for the direct `OxCalcTreeContext` path; literal value edits, formula text edits, literal-to-formula transitions, and formula-to-literal transitions preserve `StructureSnapshot` identity while advancing `NodeInputSnapshot` identity; structural edits advance `StructureSnapshot` identity and preserve compatible node inputs by stable identity; namespace/capability mutations advance `NamespaceSnapshot` identity and invalidate formula artifacts through explicit compatibility rules; authoritative input truth no longer lives in mutable side maps or content-like structural fields; formula/bind facts are consumed as typed OxFml outputs; publication/reject behavior is preserved by tests and TraceCalc/optimized differential evidence; and W054 has an explicit retention-identity map for the new layers.
6. initial_epic_lanes:
   live parent epic `calc-ujl4`; child beads `calc-ujl4.1` through `calc-ujl4.16` mirror workset labels `W057.1` through `W057.16` in `docs/worksets/W057_WORKSPACE_REVISION_AND_SNAPSHOT_LAYER_REWORK.md`: corpus guardrails and field authority audit, core snapshot types, structural input/artifact authority removal, workspace lifecycle and structural edits on `WorkspaceRevision`, node input path, formula text and literal/formula transitions, namespace snapshot, formula binding snapshot intake, dependency-shape snapshot publication, publication/runtime overlay separation, export/import/views, optimized runtime cutover, TraceCalc/differential migration, W054 retention identity retarget, legacy leftover deletion, and closure audit.
7. rollout_mode:
   `closed_first_scope` (parent epic `calc-ujl4` and children `calc-ujl4.1`
   through `calc-ujl4.16` are closed for the declared W057 representation
   scope: direct `OxCalcTreeContext` and local optimized TreeCalc now use
   explicit workspace revision roots, derived formula/dependency layers,
   publication/runtime layers, and a W054 retention identity map. Full W054
   bounded-memory closure, W049 formalization, broader W055/W056 product
   semantics, localized structural impact compatibility, dependency-component
   reuse, publication shards, and subtree hashing remain successor work.)

### W058 Retained-Replay Outstanding Registry
1. purpose:
   central registry of outstanding retained-replay (OxReplay) obligations for non-table TreeCalc reference families, and the home for reconciling the `ProductGreen` evidence bar. Opened 2026-05-30 after a W056 evidence pass (A1/A2 = `calc-4vs8.71`/`.72`) found the retained-replay half of "product-green" genuinely absent for non-table families while direct-context corpora are green. Decision: the DNA TreeCalc UX prototype proceeds on direct-context-green; true retained replay is deferred and tracked here rather than faked.
2. depends_on:
   `W056` (owns the reference resolution and the deferred prototype-evidence beads); broader `calc-4vs8.33` non-table evidence umbrella.
3. parent_doctrine_and_spec_surfaces:
   `docs/worksets/W058_RETAINED_REPLAY_OUTSTANDING_REGISTRY.md`, `docs/worksets/W056_TREECALC_FULL_REFERENCE_AND_TABLE_LOWERING.md`
4. upstream_dependencies:
   cross-repo deliverable: `DnaTreeCalc` emits normalized-replay navigation artifacts; `OxReplay` retains and validates/diffs them as opaque JSON without TreeCalc-private parsing. OxReplay retained validation currently covers only the table lane.
5. closure_condition:
   W058 closes when a non-table retained-replay lane exists in OxReplay for the prototype navigation families, DnaTreeCalc emits the artifacts, OxReplay validates/diffs them without parsing producer-private strings, the deferred A1/A2 families are flipped to `ProductGreen`, and the `ProductGreen` bar is reconciled (the `children` family was flipped without a retained artifact).
6. initial_epic_lanes:
   live parent epic `calc-gogj`; `calc-4vs8.74` (navigation-evidence blocker) and `calc-gogj.1` (children ProductGreen-bar reconciliation). Deferred consumers: `calc-4vs8.71`/`.72`.
7. rollout_mode:
   `active_registry`

Restated 2026-07-04 (W062 R0): W058 is a cross-repo (DnaTreeCalc + OxReplay) retained-evidence deliverable, independent of the W062 engine-model rework; it is not absorbed, gated, or otherwise changed by W062 and continues on its own beads (calc-gogj.1, calc-4vs8.74).

### W059 OxFml Authored Input And Literal Value Authority
1. purpose:
   discussion and design-prep workset for removing OxCalc-local string-to-value parsing from authored node input. W059 treats all authored text input as Excel-style cell-entry text, whether or not it starts with `=`, and routes interpretation through OxFml so OxCalc records typed `CalcValue` outputs and/or OxFml-owned formula artifacts rather than deriving evaluator semantics locally.
2. depends_on:
   `W050` formula-authority boundary, `W057` workspace revision/snapshot-layer split, and the W098 value-model migration across OxFunc/OxFml/OxCalc.
3. parent_doctrine_and_spec_surfaces:
   `docs/worksets/W059_OXFML_AUTHORED_INPUT_AND_LITERAL_VALUE_AUTHORITY.md`, `docs/spec/core-engine/CORE_ENGINE_OXFML_SEAM.md`, `docs/spec/core-engine/CORE_ENGINE_OXCALCTREE_CONSUMER_INTERFACE_AND_HOST_CONTRACT_V1.md`
4. upstream_dependencies:
   `OxFml` owns WorksheetA1 cell-entry classification, parse/bind/runtime artifacts, `RuntimeFormulaResult`, prepared formula identity/package surfaces, and `RuntimeFormulaResult::published_calc_value()`. `OxFunc` owns the `CalcValue` value universe and function/value semantics. DNA OneCalc is evidence for the existing OxFml-authored cell-entry route.
5. closure_condition:
   W059 exits design prep when OxCalc no longer owns local authored-text parsing, non-`=` literals and leading-`=` formulas share an OxFml-authored input interpretation path, the first OxFml call returns exactly `CalcValue`, `BoundFormula`, or diagnostics, OxCalc builds dependency-tree work from `BoundFormula`, typed API inputs have a direct `CalcValue` route, node callable values retain the required OxFml callable/prepared artifact shape, and tests cover admitted literal/formula/callable paths with typed diagnostics or follow-up beads for exclusions.
6. initial_epic_lanes:
   live parent epic `calc-rqoq`; discussion and evidence lane first: inspect DNA OneCalc/OxFml live literal handling, define the authored-input result enum (`CalcValue`, `BoundFormula`, or diagnostics), then replace `treecalc.rs` / `repository.rs` / coordinator helper string parsers with OxFml-owned interpretation.
7. rollout_mode:
   `discussion_active`

### W060 Calc-Time Reference Representation And Host Reference System
1. purpose:
   design and execution-planning workset for replacing the remaining calc-time `HOST_REF_*` formal-token bridge with a typed reference representation based on `CalcValue::Reference` and an explicit host reference-system interface. W060 does not reopen the `BoundFormula` reference representation used for dependency graph construction; it owns the runtime value/reference lane after dependency scheduling.
2. depends_on:
   `W059` for authored-input and `BoundFormula` boundary cleanup, `W051`/`W056` for the current TreeCalc sparse/reference-reader pressure points, and the W098 CalcValue value-model migration across OxFunc/OxFml/OxCalc.
3. parent_doctrine_and_spec_surfaces:
   `docs/worksets/W060_CALC_TIME_REFERENCE_REPRESENTATION_AND_HOST_REFERENCE_SYSTEM.md`, `docs/worksets/W059_OXFML_AUTHORED_INPUT_AND_LITERAL_VALUE_AUTHORITY.md`, `docs/spec/core-engine/CORE_ENGINE_OXFML_SEAM.md`, `docs/spec/core-engine/CORE_ENGINE_OXCALCTREE_CONSUMER_INTERFACE_AND_HOST_CONTRACT_V1.md`
4. upstream_dependencies:
   `OxFunc` owns `CalcValue`, `CoreValue::Reference`, reference-visible function semantics, and function execution context dereference/coercion policy. `OxFml` owns formula syntax, bind, and evaluator entry points. `OxCalc` owns host reference systems/profiles, including the current TreeCalc reference system and future Excel-compatible/grid profiles.
5. closure_condition:
   W060 closes its first scope when the reference-system API is specified and implemented for the TreeCalc runtime path; `CalcValue::Reference` carries opaque host-owned reference identity without `HOST_REF_*`; OxFunc dereference, sparse enumeration, fact query, text-resolution, and transform/composition requests route through FEC/reference-system traits; OxCalc implements the TreeCalc reference system for the exercised `@CHILDREN`, reference-literal array, and direct host-reference paths; dependency graph construction still consumes `BoundFormula`/`BoundExpr`; and focused OxFunc/OxFml/OxCalc tests cover the canonical structural and CTRO examples.
6. initial_epic_lanes:
   discussion/design lane first: specify the reference-system API, expand `ReferenceLike` beyond string identity, replace runtime formal-token sparse-reference bindings with typed host reference handles, preserve the `BoundFormula` dependency-graph path, and introduce profile-ready transform requests for `OFFSET`, reference-form `INDEX`, union/intersection, and structural selectors.
7. rollout_mode:
   `discussion_active`

### W062 Ideal Engine Model Rework
1. purpose:
   rework OxCalc to its ideal end state with no legacy/compatibility constraints (downstream adapts afterward): one general structural document model (workbook = workspace whose root plays the Workbook role, sheets as child nodes with grid backings, meta-children for properties, more general than Excel's object model); multi-profile reference architecture with profile-carried structural vocabulary and a strict-excel profile fully covering Excel referencing; two-model calculation (simple-correct oracle + extremely-optimizable engine, permanent differential) with a workbook-scoped dependency graph (cell granularity for grid edges, name granularity where the tree joins); document-level consumer surface (load/bind/seed-names/readout/clear/output); full-scope oxdoc-model ingestion. Program plan and verified starting map: `docs/worksets/W062_IDEAL_ENGINE_MODEL_REWORK.md`.
2. depends_on:
   `W057` (revision/snapshot layers as substrate), `W061` (folded in as the strict-excel execution arm); OxFml `W077` for the generic BindProfile ABI, caller-independent template identity, and the 3D sheet-range grammar production (parallel upstream lane with an R3 entry gate); OxDoc oxdoc-model ingest/output contracts (already shipped). Reconciliation dispositions for W049/W051/W052/W053/W054/W055/W056/W058/W059/W060 are an R0 deliverable (bead `calc-5kqg.2`).
3. parent_doctrine_and_spec_surfaces:
   `docs/worksets/W062_IDEAL_ENGINE_MODEL_REWORK.md`; R1 design docs `docs/design/W062_D1..D4_*.md` (to be authored); existing `docs/spec/core-engine/CORE_ENGINE_GRID_MODEL.md`, `CORE_ENGINE_REFERENCE_PROFILE_CONTRACT.md`, `CORE_ENGINE_GRID_REFERENCE_MACHINE.md` (amended by R1).
4. upstream_dependencies:
   `OxFml` owns grammar/bind lifecycle/profile dispatch/normal-form envelopes (W077); `OxFunc` owns function/value semantics; `OxDoc` owns file read/write and the neutral model + ingest seam (`OxCalcIngestSink`); `OxXlPlay`/`OxReplay` own Excel observation for later oracle evidence; DnaTreeCalc adapts downstream in R7 (W011 resumes on the new surface).
5. closure_condition:
   R1 designs (D1-D4) authored, Fable-reviewed, and landed; R2-R6 implemented with the workbook-level two-model differential clean and the incremental cost bar met (single-cell edit touches the dirty cone, evidenced by perf counters); R6 ingestion round-trips OxDoc fixtures (literal edit + cached-value refresh class) with no silent loss; downstream unblocked (DnaTreeCalc W011 resumable on the new document surface); register sequencing reconciled with every absorbed/paused workset disposition recorded.
6. initial_epic_lanes:
   R0 bootstrap (stash triage; register reconciliation; OxFml W077 activation with R3 gate); R1 architecture designs D1 structural model / D2 reference architecture / D3 workbook calculation / D4 document surface + ingestion; R2 structural roles/settings/retention; R3 reference vocabulary + strict-excel completion; R4 workbook graph + oracle + incremental consumer; R5 document surface verbs; R6 oxdoc ingestion; R7 downstream adaptation. Epic bead: `calc-5kqg`.
7. rollout_mode:
   `execution_target` (preliminary plan landed; R1 designs govern implementation waves; full 5.1 sequencing rewrite is R0 bead `calc-5kqg.2`)

Note (2026-07-06, W062 R6 wave close — bead `calc-5kqg.64`, R6.7): **R6 (oxdoc-model ingestion) is COMPLETE** — all 7 beads landed on `main`: R6.1 oxdoc dep edge + ingest skeleton + tier ledger + Tier A cells (`bebbd698`, `calc-5kqg.58`); R6.2 formula ingest — bind, shared regions, topology routing, degradation (`83ef1bbf`, `calc-5kqg.59`); R6.3 names, tables, merges, settings ingest — deferred installs (`71f52d21`, `calc-5kqg.60`); R6.4 Tier B inert store + overlay seats + digest meta-child (`f10e4593`, `calc-5kqg.61`); R6.5 `load_workbook_model` verb + calc-mode load recalc + ExternalLink pins (`be8ef7ee`, `calc-5kqg.62`); R6.6 output projection + W011 five-step round trip, Pivot B (`03bc5058`, `calc-5kqg.63`); R6.7 this bead — upstream gap handovers + register reconciliation (docs-only, no commit hash — closes on this bead's own commit).

R6.7 filed the four D4 §15 upstream gaps plus the C13 mapping-total ask as an OxDoc-repo handover doc (no prior handover-doc convention existed in OxDoc; this is the first): `docs/handovers/W062-INGEST-UPSTREAM-GAPS.md` in the OxDoc repo, indexed from OxDoc `docs/SPEC.md`. Gaps filed: (1) iteration-calc settings absent from `WorkbookHeader` (oxdoc-model `lib.rs:946-949`) — blocking-adjacent for W055 Excel-match, OxCalc defaults silently to `enabled=false/100/0.001` (C4) until the header grows the field; (2) no cell-level `WorkbookModelEditKind` variant (`lib.rs:525-532`) — the §7b `workbook_authored_delta` (R5.7) is the natural producer of a minimal edit set once a `CellEdit`-class variant exists, non-blocking (whole-model projection works today per R6.6); (3) `drive_oxcalc_ingest_from_model_access` requires eager events (`lib.rs:2723-2745`) — a chunked/deferred drive path matters for large-workbook wasm loads, non-blocking; (4) no sheet reorder/rename modeled-edit shape, folded into (2); plus the C13 ask that every new calc-relevant `DocumentEvent` variant upstream also get a matching `OxCalcDocumentFeature` variant (verified: 29 `DocumentEvent` variants vs. 23 `OxCalcDocumentFeature` variants today, with OxCalc's sink doing a wildcard-free exhaustive match over the latter at `oxdoc_ingest.rs:1678`) — otherwise a new event is absorbed inside the upstream driver and never reaches the sink, silently, with no compile-error tripwire.

**Two R7-prerequisite gaps discovered during R6 (both filed as beads, both now CLOSED — landed W062 R6.65/R6.66):**
- **`calc-5kqg.65`** (CLOSED, R6.65) — cross-sheet reference EVALUATION for freshly-loaded sheets. A loaded `Sheet2!B1 = Sheet1!A1+10` was publishing `#VALUE!` because loaded grids key `sheet_id` by a node token (`sheet:{id}`) while formulas reference the display name (`Sheet1`), and a literal-only target sheet never published its literal for the cross-sheet gather. Fixed by re-keying cross-sheet edge target cells into the target grid's id space (`WorkbookCrossSheetEdges::build`), locating gathered values by `(row,col)` within the target node (`gather_cross_sheet_cells`), and publishing authored literals at staging for non-drained sheets. A general cross-sheet RANGE gap (`=SUM(Sheet1!A1:A3)` → `#REF!`, reproduces in the authored path too) was spun out as **`calc-5kqg.67`**.
- **`calc-5kqg.66`** (CLOSED, R6.66) — Tier-A authored **collection** projection. `project_workbook_model_output` now re-emits merged regions (`MergedCellRegions`), table overlays (`TableOverlay`), defined names + their Tier-B metadata half (`DefinedName`, in the prelude), and repeated/shared-formula regions (`SharedFormulaRegion`) as their `DocumentEvent`s — resolving the engine sheet token to the upstream id + display name via `IngestedDocumentFacts.sheet_stream_ids`, with an A1-range renderer for the range strings. The typed `UnprojectableTierACollections` refusal is removed; a collection-bearing workbook now round-trips (load → project → reload) intact.

Together with gap (1) above (iteration settings), `.65` and `.66` were the real-multi-feature-xlsx prerequisites for R7 (DnaTreeCalc W011 resuming against actual multi-sheet, collection-bearing workbooks rather than the two-cell fixture) — both now landed. R7 is unblocked.

### W061 Strict Excel Grid Planning And Reference Floor
1. purpose:
   promote the strict Excel grid planning set into OxCalc-owned implementation surfaces and stand up the first GridCalc-Ref reference-machine floor. This workset prepares the implementation run for bounded grid coordinates, R1C1-relative formula identity, materialization invariance, spill extents, hidden-row AxisState, feature-rendered-region extension points, and counter-gated grid performance evidence.
2. depends_on:
   `W047`, `W050`, `W060`; OxFml W077 for the generic `BindProfile`/reference-profile ABI, source-preserving syntax/bind lifecycle, symbolic references, caller-independent normal-form identity, and edit-transform envelopes; OxDoc bootstrap for document-event ingest/export contracts; OxXlPlay scenario ops for `[verify-COM]` spill and hidden-row captures.
3. parent_doctrine_and_spec_surfaces:
   `docs/worksets/W061_STRICT_EXCEL_GRID_PLANNING_AND_REFERENCE_FLOOR.md`, `docs/spec/core-engine/CORE_ENGINE_REFERENCE_PROFILE_CONTRACT.md`, `docs/spec/core-engine/CORE_ENGINE_GRID_MODEL.md`, `docs/spec/core-engine/CORE_ENGINE_GRID_REFINEMENT_AND_EQUIVALENCE.md`, `docs/spec/core-engine/CORE_ENGINE_GRID_REFERENCE_MACHINE.md`, `docs/spec/core-engine/CORE_ENGINE_GRID_PERF_REGISTER.md`
4. upstream_dependencies:
   `OxFml` owns the generic reference-profile ABI, formula syntax/bind lifecycle, profile dispatch, source preservation, and normal-form key envelopes; `OxCalc` owns the `strict-excel-grid` profile semantics and runtime provider; `OxFunc` owns function/value semantics and host-info/provider traits; `OxDoc` owns file read/write, `PartStore`/opaque-byte preservation, and fidelity ledgers; `OxXlPlay`/`OxReplay` own COM observation and comparison verdicts.
5. closure_condition:
   W061 closes when the grid specs are indexed and reviewed, the OxFml reference-profile contract target is packetized for W077, GridCalc-Ref runs the first bounded grid corpus, GridInvalidation-Ref runs scalar dirty-closure checks, optimized-vs-reference differential mode exists for small scenarios, first touched perf rows produce deterministic counters, and spill/hidden-row Excel-observation blockers are explicit. Current ordering correction: the executable grid floor may advance inside OxCalc, but broad new grid semantics do not become shared seam claims until W077 has an acknowledged reference-profile ABI and the same lifecycle is proven by DnaTreeCalc plus a tiny fake profile.
6. initial_epic_lanes:
   reference-profile contract hardening; W077 ABI alignment for profile ids/versions, bind contexts, symbolic reference payloads, normal-form keys, dependency envelopes, argument preparation, transform envelopes, and render/source-preservation policy; TreeCalc/fake-profile canary proof for non-grid reference binding; grid reference machine; optimized authored storage candidate for sparse cells, dense value regions, repeated formula regions, and table overlays; grid corpus seed with value, differential, and invalidation artifact lanes; OxXlPlay COM capture prerequisites; perf counters/register assertions; defined-name and structured-reference provider floors; spill ledger/provider floor; hidden-row AxisState/provider floor; structural edit algebra/matrix and feature-rendered-region admission policy.
7. rollout_mode:
   `execution_target` (reclassified 2026-07-04 per W062 R0: W061 becomes this program's strict-excel execution arm under W062 Direction 2 / R3. calc-kaqc.1-.5 continue as W061's own epic with a related edge to calc-5kqg.5 [D2 reference architecture design], since W077 ABI alignment and the vocabulary layer are now designed jointly. Code-state note: the optimized-vs-reference differential mode closure item is already met on main — GridEngineMode::Both (grid/machine/differential.rs) is implemented and exercised in 40+ test sites as inherited W057-era infrastructure; remaining W061 closure items — GridCalc-Ref corpus, spill/hidden-row planning, perf counters — are still open.)

### W052 Sensitivity And Derivative Seam
1. purpose:
   layer the `Differentiable(parameter_set)` capability onto numeric rich values, enabling sensitivity/derivative queries (`partial(parameter) -> RichValue`) over the call-site graph. Goal Seek, Solver, and what-if analysis become capability queries against a graph of differentiable rich values rather than bolt-on iteration loops, composing with replay and the single-publisher coordinator. Requires OxFunc kernels to carry a derivative-metadata profile (`Analytical(kernel)` | `Finite(epsilon)` | `Discontinuous`); the W050 commitment that capability-vocabulary admission is additive means no retrofit of existing artefacts is required.
2. depends_on:
   `W050` (Lane G capability vocabulary admission)
3. parent_doctrine_and_spec_surfaces:
   `docs/worksets/W052_SENSITIVITY_AND_DERIVATIVE_SEAM.md`, `docs/worksets/W050_OXCALC_OXFML_FORMULA_AUTHORITY_REWORK.md` §10.12, §11.2
4. upstream_dependencies:
   `OxFunc` is the primary owner of per-kernel derivative metadata; `OxFml` threads the `Differentiable` capability through semantic plan.
5. closure_condition:
   W052 closes its first scope when at least one derivative-capable path is exercised, unsupported/discontinuous paths have typed outcomes, sensitivity results are replay-visible, and Goal Seek/Solver product work is either implemented or routed to a successor.
6. initial_epic_lanes:
   capability contract, OxFunc derivative metadata handoff, discontinuity outcome rule, OxFml runtime/replay threading, OxCalc graph-walk scenario, replay evidence.

Note (2026-07-04, W062 R0 reconciliation): W062's ideal-engine-rework vision (structural model, reference profiles, workbook-scoped calc graph, document surface, ingestion) does not touch OxFunc kernel-side derivative metadata or the Differentiable capability lane. W052 is unaffected by W062 and remains an independent queued_successor gated only on W050 Lane G capability admission (already closed). The W062 workset doc's R0 disposition list originally omitted W052; corrected 2026-07-04 (calc-5kqg.2).

7. rollout_mode:
   `queued_successor`

### W053 Staged Concurrency Stage 2
1. purpose:
   Stage 2 of the Foundation staged-realization contract: partitioned parallel evaluators behind the same single-publisher coordinator authority, with speculative evaluation (provisional reference bindings, fingerprint-checked at commit) as the conflict-resolution discipline. Targets wall-clock speedup on multi-core hardware without losing the Stage 1 single-publisher correctness invariant. The §10 design baseline in W050 is deliberately Stage-2-shaped — independent acyclic nodes carry no ordering constraint beyond the dependency graph — so W053 partitions the schedule while keeping the single Coordinator commit authority intact. W053 must demonstrate semantic equivalence against the formalized Stage-1 baseline produced by W049, and revisits the W054 bounded-memory retention model for partitioned and speculative evaluators (speculative candidates introduce a new retention class).
2. depends_on:
   `W050` (the Stage 1 sequential coordinator on the new session model must land first); `W049` (formalized Stage-1 baseline to prove semantic-equivalence-under-strategy-change against; W049 is itself now sequenced after W062 — see W049 depends_on); Foundation Wave B FEC/F3E concurrency-hardening gates; `W062` Direction 4 (concurrency-prep constraints — deterministic worklist, pure providers, no ambient interior mutability, Send-auditable state — land inside W062 R4; W053 builds the actual partitioned/speculative executor on top of that prep rather than re-deriving it)
3. parent_doctrine_and_spec_surfaces:
   `docs/worksets/W053_STAGED_CONCURRENCY_STAGE_2.md`, `../Foundation/CORE_ENGINE_FORMAL_MODEL.md` §6.8 staged-realization contract, `docs/worksets/W050_OXCALC_OXFML_FORMULA_AUTHORITY_REWORK.md` §11.3, `docs/spec/core-engine/CORE_ENGINE_COORDINATOR_AND_PUBLICATION.md`
4. upstream_dependencies:
   Foundation Wave B FEC/F3E concurrency-hardening gates, plus any OxFml runtime/fence surfaces needed for deterministic contention replay.
5. closure_condition:
   W053 closes its first Stage-2 scope when partitioned evaluation is implemented for a declared graph class, stale-fingerprint/contention paths are exercised, replay reproduces accepted/rejected outcomes, observable results match the Stage-1 baseline, and speculative-candidate retention is deterministic.
6. initial_epic_lanes:
   partition strategy, speculative candidate identity, fingerprint granularity, contention replay artifacts, W054 retention extension, Stage-1 versus Stage-2 differential scenarios.
7. rollout_mode:
   `queued_successor`

### W054 Bounded-Memory And Pinned-Epoch GC
1. purpose:
   operational memory discipline for the artefact and overlay surfaces W050 introduces — the compiled-artefact / plan-template cache, runtime overlays, the per-edge differential-evaluation value cache, Subscription Registry topic envelopes, and pinned reader views. Each cache carries a profile-declared retention class (Required / Best-Effort / Discardable) and a pinned-epoch protection rule (active session, stabilisation window, observer-pinned). Eviction is deterministic — given the same operation history and the same retention claims, two engines evict in the same order — and the eviction trace is part of replay conformance. This is the difference between a `replay-friendly` engine and a `replay-deterministic` one: a spec that does not pin eviction order produces replay artefacts that drift across implementations even when results agree. W054 makes the bounded-memory contract part of the spec rather than an implementation detail.
2. depends_on:
   `W050` (the new artefact set must exist so retention costs are measurable); `W051` (sparse-reader artefacts are part of the artefact set W054's retention model must cover). Per §5.1, W054 precedes W053; W053 then revisits this retention model for partitioned and speculative evaluators.
3. parent_doctrine_and_spec_surfaces:
   `docs/worksets/W054_BOUNDED_MEMORY_AND_PINNED_EPOCH_GC.md`, `../Foundation/CORE_ENGINE_FORMAL_MODEL.md` §6.3 (overlay eviction is deterministic and epoch-safe) and §6.8 (overlay lifecycle baseline), `docs/spec/core-engine/CORE_ENGINE_OVERLAY_AND_DERIVED_RUNTIME.md`, `docs/worksets/W050_OXCALC_OXFML_FORMULA_AUTHORITY_REWORK.md` §11.3
4. upstream_dependencies:
   none planned; W054 consumes W050/W051 engine artifacts and Foundation pinned-epoch doctrine.
5. closure_condition:
   W054 closes its first scope when every declared cache/overlay surface has a retention class, pinned-epoch protection is implemented or blocked explicitly, eviction order is deterministic, replay records eviction decisions, structural edits distinguish snapshot advancement from retention compatibility through impact-closure or explicit conservative fallback traces, and W053-only speculative retention is routed forward. Re-affirmed 2026-07-04 (W062 R0): grid backing state (GridBackingState, held live-only in consumer.rs) has zero retention-class coverage today — RetentionClass currently has exactly one variant (EdgeValueCacheRetentionClass::W054PendingEphemeralPerEdgeValueCache, tree-side only). W062 R2/D1 grid-revision-retention work must define the grid-side retention class(es) that W054 then owns for eviction/pinning; this is a forward dependency from W062 onto W054, not the reverse. Re-targeted 2026-07-05 (W062 R2.8, bead calc-5kqg.15 closed): the grid-side retention classes now EXIST — `GridRetentionClass::{RevisionRetainedGridInput, EphemeralDerivedGridState}` (grid/authored.rs, with `selector_key`/`is_revision_pinned` accessors following the `EdgeValueCacheRetentionClass` precedent), classified per grid half by `GridNodeState::retention_class_of_{input,derived}` (consumer.rs). W062 defines the class CONTRACT and its pinning rules (D1 §7.4 / C7): `RevisionRetainedGridInput` is revision-pinned and evicted only transitively through `enforce_workspace_revision_retention_policy` — dropping a retained revision drops its otherwise-unshared grid-input Arcs (proven by strong-count tests `evicting_a_revision_drops_its_unshared_grid_input_arc` and `candidate_pinned_revision_never_drops_its_grid_input_arc`), whole revisions only, oldest-unpinned-first, candidate pins imply grid-input pins; `EphemeralDerivedGridState` is freely evictable at recalc cost. **W054 now OWNS eviction/pinning of these classes**, including the named-but-unimplemented byte-budget seat `OxCalcTreeRevisionRetentionPolicy::retained_grid_input_byte_budget: Option<u64>` (documented, `None` in R2; W054 implements any budget enforcement, evicting whole revisions oldest-unpinned-first, never tearing sheets out of a retained revision). No new eviction heuristics landed in W062 — those remain W054's.
6. initial_epic_lanes:
   residency counters, cache/overlay surface list, retention classes, pin/unpin rules, eviction ordering, replay-visible eviction trace, structural-impact compatibility bases, conservative rebuild/fallback counters, bounded-memory scenario. First active slice: per-edge value-cache eviction has deterministic oldest-first trace/counter evidence and coordinator pin/unpin counters are explicit; remaining surfaces are still open.
7. rollout_mode:
   `initial_slice_active`

### W055 Circular References And Iterative Calculation Excel-Match Closure
1. purpose:
   turn the W048 single-host scoped circular-reference evidence slice into a product-grade Excel-match feature-area target. W055 explicitly includes the difficult Excel cases that must be teased out through black-box observation and then specified and matched in code: ordinary structural cycles, CTRO/dynamic-reference cycles, iterative calculation, dynamic-array spill cycles, data-table cycles, external workbook link cycles, volatile/external invalidation inside cycle regions, edit-order and calculation-chain sensitivity, workbook reopen behavior, and cross-thread/multithread variants. The intended end state is a clear product claim for the declared Excel-match scope, with formal proof status reported separately from implementation status.
2. depends_on:
   `W048`, `W050` for Tranche A; `W051` only for sparse/range-backed cycle lanes such as data-table-adjacent or spill/range fixtures.
3. parent_doctrine_and_spec_surfaces:
   `docs/worksets/W055_CIRCULAR_REFERENCES_AND_ITERATIVE_CALCULATION_EXCEL_MATCH_CLOSURE.md`, `docs/spec/core-engine/w055-cycles/W055_TRANCHE_A_ROLLOUT_AND_SCOPE.md`, `docs/spec/core-engine/w055-cycles/W055_DNATREECALC_CYCLE_CONFIG_HANDOVER_INTAKE.md`, `docs/spec/core-engine/w055-cycles/W055_HOST_CONTRACT_TERMINAL_SEMANTICS_AND_PARITY_GATES.md`, `docs/worksets/W048_CIRCULAR_DEPENDENCY_CALCULATION_PROCESSING.md`, `docs/spec/core-engine/w048-cycles/`, `docs/spec/core-engine/CORE_ENGINE_RECALC_AND_INCREMENTAL_MODEL.md`, `docs/spec/core-engine/CORE_ENGINE_PROFILE_SELECTORS.md`, `docs/spec/core-engine/CORE_ENGINE_TRACECALC_REFERENCE_MACHINE.md`, `docs/spec/core-engine/CORE_ENGINE_OXCALCTREE_CONSUMER_INTERFACE_AND_HOST_CONTRACT_V1.md`
4. upstream_dependencies:
   `OxFml` for formula/evaluator-facing behavior, dynamic arrays/spills, external references, and replay surfaces; `OxFunc` for function semantics, volatility, external invalidation, numeric precision, and data-table-adjacent kernel behavior; Foundation only if doctrine/profile/conformance-pack promotion changes are needed.
5. closure_condition:
   W055 closes only for a declared product scope. That scope requires profile-named cycle modes, typed host-facing cycle config and diagnostics implemented in the OxCalcTree facade, DnaTreeCalc acceptance evidence, Excel observations or accepted blockers for included scenario families, TraceCalc and TreeCalc/core conformance, a general implementation path rather than fixture-keyed result tables, explicit handling or exclusion of hard Excel families, updated spec/replay contracts, and separate formalization obligations.
6. initial_epic_lanes:
   parent epic `calc-9ouy`; rollout/scope (`calc-9ouy.1`), general cycle-engine design (`calc-9ouy.2`), fixture-keyed implementation replacement (`calc-9ouy.3`), Tranche A conformance/replay evidence (`calc-9ouy.4`), Excel observation matrix (`calc-9ouy.5`), hard Excel family lanes (`calc-9ouy.6`), formalization handoff (`calc-9ouy.7`), DnaTreeCalc cycle config host contract (`calc-9ouy.8`), OxCalcTree typed cycle config/diagnostics implementation (`calc-9ouy.9`), DnaTreeCalc bridge acceptance evidence (`calc-9ouy.10`).
7. rollout_mode:
   `in_progress`

Disposition 2026-07-04 (W062 R0): calc-9ouy.2 (general cycle engine design) is paused pending joint authorship with W062 D3 (calc-5kqg.6), which extends cycle detection workbook-wide — the two designs must agree on one cycle-engine shape rather than W055 freezing a tree-only design D3 then has to retrofit. calc-9ouy.3/.4/.9/.10 (Tranche A implementation, host contract, conformance) continue independently — tree/single-scope work not blocked by the workbook graph. calc-9ouy.5/.6/.7 continue as background lanes, unaffected by W062.
