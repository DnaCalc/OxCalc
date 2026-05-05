# OxCalc Workset Register

Status: `active_register`
Date: 2026-04-03

## 1. Purpose
This is the live ordered workset register for current OxCalc execution.

It defines the current workset set, dependency order, and intended rollout shape
for the repo under the lighter bead-structured doctrine.

This file is not an execution-status board.
It owns workset truth, not bead state.

## 2. Planning-Surface Clarification
Planning and execution truth in OxCalc is now split as follows:
1. [CHARTER.md](../CHARTER.md) owns mission, scope, and completion doctrine.
2. [OPERATIONS.md](../OPERATIONS.md) owns the operating model and execution rules.
3. this register owns the ordered workset set and dependency shape.
4. `.beads/` owns epics, beads, readiness, blockers, in-progress state, and closure.
5. current spec, seam, replay, oracle, and evidence artifacts remain the supporting truth surfaces for supported claims.

Transition note:
1. `W032` is the doctrine-migration provenance packet for this shift.
2. `.beads/` is now bootstrapped and owns live execution-state truth.
3. this register is now authoritative for workset order, while closed or reached-gate worksets remain provenance packets rather than live trackers.

## 3. Use Rule
Use this document as:
1. the repo-local workset authority,
2. the source for future `workset -> epic -> bead` rollout,
3. the current ordered implementation map for active OxCalc work.

Do not use this document as:
1. a second blocker tracker,
2. a substitute for the bead graph,
3. a reason to keep one status narrative per workset forever,
4. a duplicate of current seam, runtime, or evidence truth surfaces.

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
   `execution_target`

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
   `execution_target`

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
   `execution_target`

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
   `execution_target`

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
   `execution_target`

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
   `execution_target`

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
   `execution_target`

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
   `execution_target`

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
   `execution_target`
