# OxCalc Workset Register

Status: `active_register`
Date: 2026-05-06

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
   `execution_target`

### W038 Core Formalization Release-Grade Closure Hardening
1. purpose:
   continue after W037 by converting release-grade residual blockers into direct TraceCalc authority, optimized/core-engine conformance, proof/model assumption discharge, Stage 2 replay-equivalence, operated-service, independent-diversity, OxFml seam/watch, pack-grade replay, and C5 evidence where possible, while allowing specs and scope to evolve from new evidence rather than treating earlier specs or implementations as immutable targets.
2. depends_on:
   `W037`
3. parent_doctrine_and_spec_surfaces:
   `docs/worksets/W038_CORE_FORMALIZATION_RELEASE_GRADE_CLOSURE_HARDENING.md`, `docs/spec/core-engine/w037-formalization/W037_CLOSURE_AUDIT_AND_FULL_VERIFICATION_RELEASE_DECISION.md`, `docs/spec/core-engine/CORE_ENGINE_FORMALIZATION_AND_ASSURANCE.md`, `docs/spec/core-engine/CORE_ENGINE_TRACECALC_REFERENCE_MACHINE.md`, `docs/spec/core-engine/CORE_ENGINE_OXFML_SEAM.md`
4. upstream_dependencies:
   `OxFml`
5. closure_condition:
   W037 residuals are assigned to W038 proof, implementation, replay, service, diversity, pack, handoff/watch, deferral, or successor obligations; TraceCalc authority exclusions are discharged, accepted, or kept as exact blockers; optimized/core-engine conformance blockers are fixed, diff-promoted, spec-evolved, or carried without declared-gap match promotion; Lean/TLA assumptions and totality boundaries are checked or blocked; Stage 2 partition replay and semantic-equivalence evidence exists before policy promotion; operated assurance, cross-engine service, and alert/quarantine claims use operated artifacts; independent evaluator diversity is backed by independent implementation authority; OxFml formatting and `LET`/`LAMBDA` seam rows remain current; pack/C5 decisions state exact evidence consequence; and the closure audit maps release-grade objectives to artifacts before any promotion claim.
6. initial_epic_lanes:
   residual release-grade obligation ledger and objective map, TraceCalc oracle authority and authority-exclusion discharge, optimized core-engine conformance blocker closure and fixes, proof-model assumption discharge and totality boundary hardening, Stage 2 partition replay and semantic-equivalence execution, operated assurance alert-quarantine and cross-engine service, independent evaluator diversity and OxFml seam watch closure, pack-grade replay governance and C5 release decision, closure audit and release-grade verification decision
7. rollout_mode:
   `execution_target`

### W039 Core Formalization Release-Grade Successor Closure
1. purpose:
   continue after W038 by attacking the remaining release-grade blockers directly: optimized/core exact blockers, Lean/TLA totality, Stage 2 production partition policy, operated assurance services, retained history, independent evaluator diversity, OxFml seam breadth including current W073 typed-only conditional-formatting metadata, callable metadata, pack-grade replay governance, C5 reassessment, and release-grade decision.
2. depends_on:
   `W038`
3. parent_doctrine_and_spec_surfaces:
   `docs/worksets/W039_CORE_FORMALIZATION_RELEASE_GRADE_SUCCESSOR_CLOSURE.md`, `docs/spec/core-engine/w038-formalization/W038_CLOSURE_AUDIT_AND_RELEASE_GRADE_VERIFICATION_DECISION.md`, `docs/spec/core-engine/CORE_ENGINE_FORMALIZATION_AND_ASSURANCE.md`, `docs/spec/core-engine/CORE_ENGINE_TRACECALC_REFERENCE_MACHINE.md`, `docs/spec/core-engine/CORE_ENGINE_OXFML_SEAM.md`
4. upstream_dependencies:
   `OxFml`
5. closure_condition:
   W038 residuals are mapped to W039 obligations; optimized/core exact blockers are fixed, directly evidenced, spec-evolved, or retained without declared-gap match promotion; Lean/TLA totality and proof/model rows distinguish discharged, bounded, assumed, and blocked cases; Stage 2 production partition and pack-grade replay evidence is present before policy promotion; operated assurance, retained-history, alert/quarantine, and cross-engine differential claims use service artifacts; independent evaluator diversity uses independent implementation authority; OxFml W073 typed-only formatting, broad seam breadth, public consumer surfaces, and callable metadata are evidenced, handed off, or blocked; pack/C5 and release-grade decisions state exact evidence consequence before any promotion claim.
6. initial_epic_lanes:
   residual successor obligation ledger and promotion-readiness map, optimized core exact blocker implementation closure, Lean/TLA totality and proof-model closure tranche, Stage 2 production partition policy and replay governance, operated assurance service and retained history substrate, independent evaluator row set and cross-engine diversity, OxFml seam breadth and callable metadata closure, pack-grade replay governance and C5 reassessment, closure audit and release-grade decision
7. rollout_mode:
   `execution_target`

### W040 Core Formalization Release-Grade Direct Verification
1. purpose:
   continue after W039 by converting exact residual release-grade blockers into direct implementation, proof, model, service, diversity, OxFml/callable, pack/C5, and release-grade evidence where possible, while preserving exact blockers where direct evidence is still insufficient.
2. depends_on:
   `W039`
3. parent_doctrine_and_spec_surfaces:
   `docs/worksets/W040_CORE_FORMALIZATION_RELEASE_GRADE_DIRECT_VERIFICATION.md`, `docs/spec/core-engine/w039-formalization/W039_CLOSURE_AUDIT_AND_RELEASE_GRADE_DECISION.md`, `docs/spec/core-engine/CORE_ENGINE_FORMALIZATION_AND_ASSURANCE.md`, `docs/spec/core-engine/CORE_ENGINE_TRACECALC_REFERENCE_MACHINE.md`, `docs/spec/core-engine/CORE_ENGINE_OXFML_SEAM.md`
4. upstream_dependencies:
   `OxFml`
5. closure_condition:
   W039 residuals are mapped to W040 direct-verification obligations; optimized/core exact blockers are fixed, directly evidenced, spec-evolved, or retained without declared-gap match promotion; Rust totality/refinement and Lean/TLA verification claims are checked or blocked; Stage 2 production policy claims have production-relevant partition soundness and observable-result invariance evidence; operated service claims use runnable service artifacts; independent evaluator claims use independent implementation authority; OxFml W073 typed-only formatting, broad seam breadth, public consumer surfaces, and callable metadata are evidenced, handed off, or blocked; pack/C5 and release-grade decisions state exact evidence consequence before any promotion claim.
6. initial_epic_lanes:
   residual direct-verification obligation map, optimized core exact blocker fixes and differentials, Rust totality and refinement proof tranche, Lean/TLA full-verification discharge tranche, Stage 2 production policy and equivalence implementation, operated assurance and retained-history service implementation, independent evaluator implementation and operated differential, OxFml seam breadth and callable metadata implementation, pack-grade replay governance and C5 promotion decision, closure audit and release-grade verification decision
7. rollout_mode:
   `execution_target`

### W041 Core Formalization Release-Grade Successor Verification
1. purpose:
   continue after W040 by converting the remaining release-grade blockers into direct implementation, proof, model, operated-service, diversity, OxFml/callable, pack/C5, and release-grade verification evidence where possible, while preserving exact blockers where direct evidence is still insufficient.
2. depends_on:
   `W040`
3. parent_doctrine_and_spec_surfaces:
   `docs/worksets/W041_CORE_FORMALIZATION_RELEASE_GRADE_SUCCESSOR_VERIFICATION.md`, `docs/spec/core-engine/w040-formalization/W040_CLOSURE_AUDIT_AND_RELEASE_GRADE_VERIFICATION_DECISION.md`, `docs/spec/core-engine/CORE_ENGINE_FORMALIZATION_AND_ASSURANCE.md`, `docs/spec/core-engine/CORE_ENGINE_TRACECALC_REFERENCE_MACHINE.md`, `docs/spec/core-engine/CORE_ENGINE_OXFML_SEAM.md`
4. upstream_dependencies:
   `OxFml`
5. closure_condition:
   W040 residual lanes are mapped to W041 obligations; optimized/core residual blockers are fixed, directly evidenced, spec-evolved, or retained without declared-gap match promotion; Rust totality/refinement and Lean/TLA verification claims are checked or blocked; Stage 2 production policy claims have production-relevant partition soundness and observable-result invariance evidence; operated service claims use operated service artifacts and retained lifecycle evidence; independent evaluator claims use independent implementation authority and breadth; OxFml W073 typed-only formatting, broad display/publication, public migration, callable metadata, and callable carrier sufficiency are evidenced, handed off, or blocked; pack/C5 and release-grade decisions state exact evidence consequence before any promotion claim.
6. initial_epic_lanes:
   residual release-grade successor obligation map, optimized core residual blocker implementation and differential closure, Rust totality refinement and panic-boundary discharge, Lean/TLA full-verification and fairness discharge, Stage 2 production analyzer and pack-equivalence proof tranche, operated assurance retained-history and alert-dispatch service tranche, independent evaluator breadth and operated differential service tranche, OxFml broad display/publication and callable-carrier closure, pack-grade replay governance and C5 reassessment, closure audit and release-grade verification decision
7. rollout_mode:
   `execution_target`

### W042 Core Formalization Release-Grade Evidence Closure Expansion
1. purpose:
   continue after W041 by converting the remaining release-grade blockers into stronger direct conformance, proof, model, operated-service, diversity, OxFml/callable, pack/C5, and release-grade verification evidence where possible, while preserving exact blockers where direct evidence is still insufficient.
2. depends_on:
   `W041`
3. parent_doctrine_and_spec_surfaces:
   `docs/worksets/W042_CORE_FORMALIZATION_RELEASE_GRADE_EVIDENCE_CLOSURE_EXPANSION.md`, `docs/spec/core-engine/w041-formalization/W041_CLOSURE_AUDIT_AND_RELEASE_GRADE_VERIFICATION_DECISION.md`, `docs/spec/core-engine/CORE_ENGINE_FORMALIZATION_AND_ASSURANCE.md`, `docs/spec/core-engine/CORE_ENGINE_TRACECALC_REFERENCE_MACHINE.md`, `docs/spec/core-engine/CORE_ENGINE_OXFML_SEAM.md`
4. upstream_dependencies:
   `OxFml`
5. closure_condition:
   W041 residual lanes are mapped to W042 obligations; optimized/core counterpart conformance and callable metadata projection are fixed, directly evidenced, spec-evolved, or retained without declared-gap match promotion; Rust totality/refinement and Lean/TLA verification claims are checked or blocked; Stage 2 and pack-equivalence claims have production-relevant partition soundness, scheduler equivalence, and observable-result invariance evidence; operated service claims use operated service artifacts and retained lifecycle evidence; independent evaluator and mismatch quarantine claims use independent implementation authority and service behavior; OxFml W073 typed-only formatting, broad display/publication, public migration, callable carrier sufficiency, registered-external callable projection, and provider-failure/callable-publication semantics are evidenced, handed off, or blocked; pack/C5 and release-grade decisions state exact evidence consequence before any promotion claim.
6. initial_epic_lanes:
   residual release-grade closure obligation ledger, optimized core counterpart conformance and callable metadata projection, Rust totality refinement and core panic-boundary closure, Lean/TLA fairness and full-verification expansion, Stage 2 production analyzer and pack-grade equivalence closure, operated assurance retained-history retained-witness and alert service closure, independent evaluator breadth mismatch quarantine and operated differential service, OxFml public migration callable carrier and registered-external closure, pack-grade replay governance and C5 reassessment, closure audit and release-grade verification decision
7. rollout_mode:
   `execution_target`

### W043 Core Formalization Release-Grade Proof And Operated-Service Integration
1. purpose:
   continue after W042 by converting remaining release-grade blockers into stronger direct proof, implementation, operated-service, independent-evaluator, OxFml/callable, pack/C5, and release-grade verification evidence where possible, while preserving exact blockers where direct evidence is still insufficient.
2. depends_on:
   `W042`
3. parent_doctrine_and_spec_surfaces:
   `docs/worksets/W043_CORE_FORMALIZATION_RELEASE_GRADE_PROOF_AND_OPERATED_SERVICE_INTEGRATION.md`, `docs/spec/core-engine/w042-formalization/W042_CLOSURE_AUDIT_AND_RELEASE_GRADE_VERIFICATION_DECISION.md`, `docs/spec/core-engine/CORE_ENGINE_FORMALIZATION_AND_ASSURANCE.md`, `docs/spec/core-engine/CORE_ENGINE_TRACECALC_REFERENCE_MACHINE.md`, `docs/spec/core-engine/CORE_ENGINE_OXFML_SEAM.md`
4. upstream_dependencies:
   `OxFml`
5. closure_condition:
   W042 residual lanes are mapped to W043 obligations; optimized/core broad conformance and callable metadata blockers are fixed, directly evidenced, spec-evolved, or retained without declared-gap match promotion; Rust totality/refinement and panic-free core claims are checked or blocked; Lean/TLA verification and unbounded fairness claims are checked or blocked; Stage 2 and pack-equivalence claims have production-relevant partition soundness, scheduler equivalence, and observable-result invariance evidence; operated service claims use operated artifacts and retained lifecycle evidence; independent evaluator and mismatch quarantine claims use implementation authority and service behavior; OxFml W073 typed-only formatting, broad display/publication, public migration, callable carrier sufficiency, registered-external callable projection, provider-failure/callable-publication semantics, and current inbound observation-ledger lanes are evidenced, handed off, watched, or blocked; pack/C5 and release-grade decisions state exact evidence consequence before any promotion claim.
6. initial_epic_lanes:
   residual release-grade proof-service obligation map, optimized core broad conformance and callable metadata closure, Rust totality refinement and panic-free core proof frontier, Lean/TLA full-verification and unbounded fairness discharge, Stage 2 production partition analyzer and scheduler equivalence, operated assurance retained-history witness SLO and alert service, independent evaluator breadth mismatch quarantine and differential service, OxFml public migration formatting callable and registered-external seam, pack-grade replay governance and C5 release reassessment, closure audit and release-grade verification decision
7. rollout_mode:
   `execution_target`

### W044 Core Formalization Release-Grade Blocker Burn-Down And Service Proof Closure
1. purpose:
   continue after W043 by converting the remaining release-grade no-promotion blockers into direct implementation, proof/model, operated-service, independent-evaluator, OxFml migration, scaling, and pack-governance evidence where possible, while preserving exact blockers where direct evidence is still insufficient.
2. depends_on:
   `W043`
3. parent_doctrine_and_spec_surfaces:
   `docs/worksets/W044_CORE_FORMALIZATION_RELEASE_GRADE_BLOCKER_BURN_DOWN_AND_SERVICE_PROOF_CLOSURE.md`, `docs/spec/core-engine/w043-formalization/W043_CLOSURE_AUDIT_AND_RELEASE_GRADE_VERIFICATION_DECISION.md`, `docs/spec/core-engine/CORE_ENGINE_FORMALIZATION_AND_ASSURANCE.md`, `docs/spec/core-engine/CORE_ENGINE_TRACECALC_REFERENCE_MACHINE.md`, `docs/spec/core-engine/CORE_ENGINE_OXFML_SEAM.md`
4. upstream_dependencies:
   `OxFml`
5. closure_condition:
   W043 residual blockers are mapped to W044 obligations; optimized/core dynamic transition breadth, callable metadata projection, Rust totality/refinement, panic-free core boundaries, Lean/TLA verification, unbounded fairness, Stage 2 production partition-analyzer soundness, scheduler equivalence, pack-grade replay equivalence, operated services, retained-history and retained-witness lifecycle, retention SLO, independent evaluator breadth, mismatch quarantine, OxFml W073 typed-rule request construction, public migration, callable carrier sufficiency, registered-external callable projection, provider-failure/callable-publication semantics, release-scale replay/performance evidence, pack/C5, and release-grade decisions are evidenced, handed off, watched, blocked, or promoted only by direct evidence.
6. initial_epic_lanes:
   residual release-grade blocker reclassification and promotion-contract map, optimized core dynamic transition and callable metadata implementation tranche, Rust totality refinement and panic-surface proof expansion, Lean/TLA unbounded fairness and full-verification proof expansion, Stage 2 production partition analyzer and scheduler equivalence implementation, operated continuous assurance retained-history witness SLO and alert service, independent evaluator breadth mismatch quarantine and differential service implementation, OxFml public migration typed formatting callable and registered-external uptake, release-scale replay performance and scaling evidence under semantic guards, pack-grade replay governance service and C5 reassessment, closure audit and release-grade verification decision
7. rollout_mode:
   `execution_target`

### W045 Core Formalization Release-Grade Service And Cross-Repo Uptake Verification
1. purpose:
   continue after W044 by converting the remaining release-grade no-promotion blockers into direct implementation, proof/model, operated-service, independent-evaluator, OxFml public-surface, W073 downstream-uptake, continuous-scale, pack-governance, and release-grade evidence where possible, while preserving exact blockers where direct evidence is still insufficient.
2. depends_on:
   `W044`
3. parent_doctrine_and_spec_surfaces:
   `docs/worksets/W045_CORE_FORMALIZATION_RELEASE_GRADE_SERVICE_AND_CROSS_REPO_UPTAKE_VERIFICATION.md`, `docs/spec/core-engine/w044-formalization/W044_CLOSURE_AUDIT_AND_RELEASE_GRADE_VERIFICATION_DECISION.md`, `docs/spec/core-engine/CORE_ENGINE_FORMALIZATION_AND_ASSURANCE.md`, `docs/spec/core-engine/CORE_ENGINE_TRACECALC_REFERENCE_MACHINE.md`, `docs/spec/core-engine/CORE_ENGINE_OXFML_SEAM.md`
4. upstream_dependencies:
   `OxFml`
5. closure_condition:
   W044 residual lanes are mapped to W045 obligations; current OxFml public-surface and W073 typed-only formatting updates are reviewed; optimized/core counterpart coverage and callable metadata projection are fixed, directly evidenced, spec-evolved, or retained without declared-gap match promotion; Rust totality/refinement and Lean/TLA verification claims are checked or blocked; Stage 2 and pack-equivalence claims have production-relevant partition soundness, scheduler equivalence, and observable-result invariance evidence; operated service claims use operated artifacts and retained lifecycle evidence; independent evaluator and mismatch quarantine claims use independent implementation authority and service behavior; W073 downstream typed-rule request construction, broad OxFml display/publication, public migration, callable carrier sufficiency, registered-external callable projection, provider-failure/callable-publication semantics, continuous scale assurance, pack/C5, and release-grade decisions state exact evidence consequence before any promotion claim.
6. initial_epic_lanes:
   residual release-grade successor obligation and current OxFml intake map, optimized core counterpart coverage and callable metadata projection closure, Rust totality refinement and panic-surface hardening, Lean/TLA verification fairness and totality discharge, Stage 2 production partition and pack-grade equivalence service evidence, operated assurance retained-history retained-witness SLO service implementation, independent evaluator breadth mismatch quarantine and operated differential service, OxFml public surface W073 downstream typed formatting callable and registered-external uptake, continuous release-scale assurance and semantic regression service, pack-grade replay governance service and C5 reassessment, closure audit and release-grade verification decision
7. rollout_mode:
   `execution_target`

### W046 Core Formalization Proof-Service Consolidation And Release-Grade Promotion Readiness
1. purpose:
   continue after W045 by converting remaining release-grade no-promotion blockers into proof-service consolidation, direct optimized/core implementation, formal proof/model discharge, operated service evidence, independent evaluator authority, OxFml downstream uptake, continuous scale assurance, pack-grade replay governance, C5, and release-grade promotion-readiness evidence where feasible, while preserving exact blockers where direct evidence is still insufficient.
2. depends_on:
   `W045`
3. parent_doctrine_and_spec_surfaces:
   `docs/worksets/W046_CORE_FORMALIZATION_PROOF_SERVICE_CONSOLIDATION_AND_RELEASE_GRADE_PROMOTION_READINESS.md`, `docs/spec/core-engine/w045-formalization/W045_CLOSURE_AUDIT_AND_RELEASE_GRADE_VERIFICATION_DECISION.md`, `docs/spec/core-engine/CORE_ENGINE_FORMALIZATION_AND_ASSURANCE.md`, `docs/spec/core-engine/CORE_ENGINE_TRACECALC_REFERENCE_MACHINE.md`, `docs/spec/core-engine/CORE_ENGINE_OXFML_SEAM.md`
4. upstream_dependencies:
   `OxFml`
5. closure_condition:
   W045 residual lanes are mapped to W046 promotion-readiness obligations; current OxFml public-surface and W073 typed-only formatting updates are reviewed; optimized/core broad dynamic-transition coverage, snapshot/capability counterparts, and callable metadata projection are fixed, directly evidenced, spec-evolved, or retained without declared-gap match promotion; Rust totality/refinement and Lean/TLA verification claims are checked or blocked; Stage 2 and pack-equivalence claims have production-relevant partition soundness, scheduler equivalence, and observable-result invariance evidence; operated service claims use operated artifacts and retained lifecycle evidence; independent evaluator, mismatch quarantine, and retained-witness attachment claims use independent implementation authority and service behavior; W073 downstream typed-rule request construction, broad OxFml display/publication, public migration, callable carrier sufficiency, registered-external callable projection, provider-failure/callable-publication semantics, continuous scale assurance, pack/C5, and release-grade decisions state exact evidence consequence before any promotion claim.
6. initial_epic_lanes:
   residual promotion-readiness ledger and current uptake map, optimized core broad transition snapshot capability and callable metadata implementation, Rust totality refinement and panic-free core proof-service consolidation, Lean TLA full-verification fairness and model-bound discharge consolidation, Stage 2 production policy pack-equivalence and scheduler service promotion readiness, operated assurance retained-history retained-witness retention-SLO alert service readiness, independent evaluator mismatch quarantine retained-witness attachment operated differential readiness, OxFml W073 downstream uptake public migration callable registered-external provider closure, continuous scale assurance operated semantic regression and release-scale correctness guard, pack-grade replay governance service and C5 promotion readiness decision, closure audit and release-grade verification decision
7. rollout_mode:
   `execution_target`
