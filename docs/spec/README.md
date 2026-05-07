# OxCalc Spec Index

This directory is the OxCalc-owned mutable spec library.

## Canonical OxCalc Set
The rewritten canonical core-engine set is:
- `docs/spec/core-engine/CORE_ENGINE_ARCHITECTURE.md`
- `docs/spec/core-engine/CORE_ENGINE_STATE_AND_SNAPSHOTS.md`
- `docs/spec/core-engine/CORE_ENGINE_RECALC_AND_INCREMENTAL_MODEL.md`
- `docs/spec/core-engine/CORE_ENGINE_OVERLAY_AND_DERIVED_RUNTIME.md`
- `docs/spec/core-engine/CORE_ENGINE_COORDINATOR_AND_PUBLICATION.md`
- `docs/spec/core-engine/CORE_ENGINE_OPLOG_UNDO_REDO_AND_COLLAB_ARCHITECTURE_PLAN.md`
- `docs/spec/core-engine/CORE_ENGINE_OXCALCTREE_CONSUMER_INTERFACE_AND_HOST_CONTRACT_V1.md`
- `docs/spec/core-engine/CORE_ENGINE_OXFML_SEAM.md`
- `docs/spec/core-engine/CORE_ENGINE_DOWNSTREAM_HOST_SEAM_REFERENCE.md`
- `docs/spec/core-engine/CORE_ENGINE_FORMALIZATION_AND_ASSURANCE.md`
- `docs/spec/core-engine/CORE_ENGINE_REALIZATION_ROADMAP.md`
- `docs/spec/core-engine/CORE_ENGINE_TREECALC_SEMANTIC_COMPLETION_PLAN.md`

## TreeCalc Terminology Rule
1. `DNA TreeCalc` is the future separate repo/product and host: a large-scale incremental calculation test-bed and product, with explicit nodes/names as formula holders and no grid-structure ownership in OxCalc.
2. `OxCalcTree` is the OxCalc-owned tree-runtime consumer contract/API surface that such a host should consume.
3. Unqualified `TreeCalc` in OxCalc specs, worksets, fixtures, and local runtime code usually means the internal tree-substrate/runtime/reference preparation lane beneath `OxCalcTree`, not the `DNA TreeCalc` product itself.

## Actual Runtime Consumer Rule
If a host such as `DNA TreeCalc` intends to consume the OxCalc runtime directly:
1. start with `README.md`, `CHARTER.md`, `OPERATIONS.md`, `docs/WORKSET_REGISTER.md`, `docs/BEADS.md`, and `docs/IN_PROGRESS_FEATURE_WORKLIST.md`,
2. then read `docs/spec/core-engine/CORE_ENGINE_OXCALCTREE_CONSUMER_INTERFACE_AND_HOST_CONTRACT_V1.md` as the host-facing OxCalc tree-runtime contract,
3. then read `docs/spec/core-engine/CORE_ENGINE_OXFML_SEAM.md` as the canonical OxCalc-local seam companion,
4. then read `docs/spec/core-engine/CORE_ENGINE_TREECALC_SEMANTIC_COMPLETION_PLAN.md` as the TreeCalc-first execution and widening plan,
5. use `docs/spec/core-engine/CORE_ENGINE_OXFML_MINIMAL_UPSTREAM_HOST_INTERFACES.md` only as an implementation-backed packet companion, not as the primary OxCalc consumer contract.

## Downstream Host Seam-Reference Rule
If a downstream host such as `DNA OneCalc` needs OxCalc as seam-reference material only:
1. start with `README.md`, `CHARTER.md`, `OPERATIONS.md`, `docs/WORKSET_REGISTER.md`, `docs/BEADS.md`, and `docs/IN_PROGRESS_FEATURE_WORKLIST.md`,
2. then use `docs/spec/core-engine/CORE_ENGINE_DOWNSTREAM_HOST_SEAM_REFERENCE.md` as the local authority filter — this is the single entry point for downstream hosts,
3. read `docs/spec/core-engine/CORE_ENGINE_OXFML_SEAM.md` as the canonical OxCalc-local seam companion,
4. read `docs/spec/core-engine/CORE_ENGINE_OXFML_MINIMAL_UPSTREAM_HOST_INTERFACES.md` only as the first deterministic upstream-host packet companion — reference material for understanding the exercised packet shape, not a host API to adopt verbatim,
5. treat `docs/spec/core-engine/CORE_ENGINE_TREECALC_OXFML_SEAM_NEGOTIATION_MATRIX.md` as a temporary narrower-topic tracker rather than as seam authority,
6. treat `docs/spec/core-engine/CORE_ENGINE_TREECALC_SEMANTIC_COMPLETION_PLAN.md` as a planning companion for the consumed OxFml seam pipeline, not as seam authority.

### Document Classification For Downstream Hosts
Every OxCalc doc that a downstream host might encounter falls into one of these classes:

| Class | Meaning | Downstream host rule |
|---|---|---|
| **canonical-local-reference** | Authoritative OxCalc-owned spec text that defines local coordinator-facing seam requirements. | Read and use as seam-reference material. |
| **supporting-companion** | Implementation-backed or planning companion that supports the canonical set with narrower detail. | Read for context; do not treat as seam authority. |
| **temporary-planning** | Active negotiation, note-exchange, or planning material that will be superseded or retired. | Read only for narrower open topics and non-assumptions; never cite as stable seam authority. |
| **historical/non-authority** | Archives, mirrors, snapshots, handoff records, and note-exchange docs. | Do not read as current seam-reference truth. |

The classification of each individual document is stated in `CORE_ENGINE_DOWNSTREAM_HOST_SEAM_REFERENCE.md` Sections 4–6.

## Operation-Model And Collaboration Positioning
- `docs/spec/core-engine/CORE_ENGINE_OPLOG_UNDO_REDO_AND_COLLAB_ARCHITECTURE_PLAN.md`
  - canonical companion defining OxCalc's intended ownership and staged realization path for the operation model, undo/redo, collaboration substrate, and replay-export relation to `OxReplay`.

## Supporting Realization and Test Docs
- `docs/spec/core-engine/CORE_ENGINE_TEST_HARNESS_AND_FIXTURES.md`
  - supporting companion for self-contained fixture, scenario, and alternate calculation-space design.
- `docs/spec/core-engine/CORE_ENGINE_TEST_SCENARIO_SCHEMA_AND_TRACECALC.md`
  - supporting companion defining canonical JSON scenario schema and the first concrete `TraceCalc` surface.
- `docs/spec/core-engine/CORE_ENGINE_TEST_VALIDATOR_AND_RUNNER_CONTRACT.md`
  - supporting companion defining how the self-contained corpus is validated, executed, and emitted as run artifacts.
- `docs/spec/core-engine/CORE_ENGINE_TRACECALC_REFERENCE_MACHINE.md`
  - supporting companion defining the executable semantic oracle and later-engine conformance baseline.
- `docs/spec/core-engine/CORE_ENGINE_REPLAY_APPLIANCE_ADAPTER.md`
  - supporting companion defining how OxCalc-owned `TraceCalc`, runner, oracle, and diff artifacts project into the Foundation Replay appliance rollout.
- `docs/spec/core-engine/CORE_ENGINE_TREECALC_OXFML_SEAM_NEGOTIATION_MATRIX.md`
  - supporting planning companion defining the topic-by-topic TreeCalc seam negotiation shape for the next OxCalc↔OxFml note rounds and W026 intake work; not canonical seam authority.
- `docs/spec/core-engine/CORE_ENGINE_TREECALC_ASSURANCE_AUTHORITY_MAP.md`
  - supporting companion for W031, mapping older assurance/replay/pack planning surfaces to the W030 TreeCalc local baseline and identifying residual packetization candidates; not a host-facing API or seam authority.
- `docs/spec/core-engine/CORE_ENGINE_OXCALC_OXFML_FORMALIZATION_PASS_PLAN.md`
  - active planning companion for W033, defining the OxCalc-local formalization and spec-evolution pass over OxCalc plus read-only OxFml evaluator/FEC/F3E seam surfaces; treats current implementation behavior and current spec text as evidence surfaces rather than immutable final targets, includes TraceCalc-as-oracle refinement planning, historical no-loss review, and the narrow `LET`/`LAMBDA` boundary carrier fragment while excluding general OxFunc semantic kernels.
- `docs/spec/core-engine/w033-formalization/`
  - active W033 spec-evidence artifact root for source-freeze, review-ledger, authority-matrix, refinement, formal, replay, pack, handoff/watch, and closure-audit packets.
- `docs/spec/core-engine/w034-formalization/`
  - active W034 planning and evidence artifact root for post-W033 formalization deepening, TraceCalc oracle widening, optimized/core-engine conformance widening, Lean/TLA model-family deepening, pack/capability gate binding, and OxFml seam-watch intake.
- `docs/spec/core-engine/w035-formalization/`
  - active W035 planning and evidence artifact root for residual proof-obligation mapping, TraceCalc oracle matrix expansion, implementation conformance hardening, Lean assumption discharge, TLA non-routine exploration, continuous assurance, pack/Stage 2 reassessment, and current OxFml seam-watch intake such as W073 typed conditional-formatting metadata.
  - current W035 packet examples: `W035_RESIDUAL_PROOF_OBLIGATION_AND_SPEC_EVOLUTION_LEDGER.md`, `W035_TRACECALC_ORACLE_MATRIX_EXPANSION.md`, `W035_IMPLEMENTATION_CONFORMANCE_HARDENING.md`, `W035_LEAN_ASSUMPTION_DISCHARGE_AND_SEAM_PROOF_MAP.md`, `W035_TLA_NON_ROUTINE_EXPLORATION_AND_SCHEDULER_PRECONDITIONS.md`, `W035_CONTINUOUS_ASSURANCE_AND_CROSS_ENGINE_DIFFERENTIAL_GATE.md`, `W035_PACK_CAPABILITY_AND_STAGE2_READINESS_REASSESSMENT.md`, and `W035_CLOSURE_AUDIT_AND_SUCCESSOR_PACKET.md`.
- `docs/spec/core-engine/w036-formalization/`
  - active W036 evidence artifact root for TraceCalc coverage closure criteria, optimized/core-engine conformance closure, Lean theorem coverage expansion, concrete TLA Stage 2 partition modeling, stronger cross-engine differential evidence, continuous-assurance operation/history, and pack-grade reassessment.
  - current W036 packet examples: `W036_RESIDUAL_COVERAGE_AND_PROMOTION_BLOCKER_LEDGER.md`, `W036_TRACECALC_COVERAGE_CLOSURE_CRITERIA_AND_MATRIX_EXPANSION.md`, `W036_OPTIMIZED_CORE_ENGINE_CONFORMANCE_CLOSURE_PLAN_AND_FIRST_FIXES.md`, `W036_LEAN_THEOREM_COVERAGE_EXPANSION.md`, `W036_TLA_STAGE2_PARTITION_AND_SCHEDULER_EQUIVALENCE_MODEL.md`, `W036_INDEPENDENT_EVALUATOR_DIVERSITY_AND_CROSS_ENGINE_DIFFERENTIAL_HARNESS.md`, `W036_CONTINUOUS_ASSURANCE_OPERATION_AND_HISTORY_WINDOW.md`, `W036_PACK_GRADE_REPLAY_AND_CAPABILITY_PROMOTION_GATE_REASSESSMENT.md`, and `W036_CLOSURE_AUDIT_AND_SUCCESSOR_PACKET.md`.
- `docs/spec/core-engine/w037-formalization/`
  - W037 evidence artifact root for residual full-verification mapping, TraceCalc observable closure, optimized/core-engine conformance implementation closure, direct OxFml evaluator and narrow `LET`/`LAMBDA` seam evidence, Lean/TLA proof and model closure inventory, Stage 2 deterministic replay and partition promotion criteria, operated continuous assurance, cross-engine service piloting, pack-grade replay governance, C5 candidate decision, closure audit, and current OxFml formatting watch inputs such as W073 typed conditional-formatting metadata plus the distinct `format_delta`/`display_delta` boundary.
  - current W037 packet examples: `W037_RESIDUAL_FULL_VERIFICATION_AND_PROMOTION_GATE_LEDGER.md`, `W037_TRACECALC_OBSERVABLE_CLOSURE_AND_MULTI_READER_REPLAY.md`, `W037_OPTIMIZED_CORE_ENGINE_CONFORMANCE_IMPLEMENTATION_CLOSURE.md`, `W037_DIRECT_OXFML_EVALUATOR_AND_LET_LAMBDA_SEAM_EVIDENCE.md`, `W037_LEAN_TLA_PROOF_MODEL_CLOSURE_INVENTORY.md`, `W037_STAGE2_DETERMINISTIC_REPLAY_AND_PARTITION_PROMOTION_CRITERIA.md`, `W037_OPERATED_CONTINUOUS_ASSURANCE_AND_CROSS_ENGINE_SERVICE_PILOT.md`, `W037_PACK_GRADE_REPLAY_GOVERNANCE_AND_C5_CANDIDATE_DECISION.md`, and `W037_CLOSURE_AUDIT_AND_FULL_VERIFICATION_RELEASE_DECISION.md`.
- `archive/w038-formalization/` through `archive/w045-formalization/`
  - archived W038-W045 packet roots. These files are retained as predecessor evidence and historical context, not active W046 spec roots.
  - useful semantic fragments from these packets should be distilled into `docs/spec/core-engine/w046-formalization/` before they are used to steer new implementation or proof work.
- `docs/spec/core-engine/w046-formalization/`
  - active W046 artifact root for the post-W045 engine semantic proof spine: semantic state/transition catalog, dependency graph construction, reverse-edge consistency, SCC/cycle classification, invalidation closure, soft-reference/dynamic-reference rebind, recalc tracker transitions, evaluation order, working-value read discipline, TraceCalc refinement, OxFml seam behavior, and downstream evidence classification.
  - planned W046 packet examples: `W046_ENGINE_SEMANTIC_CATALOG_AND_EFFECT_SIGNATURE_PLAN.md`, `W046_DEPENDENCY_GRAPH_REVERSE_EDGE_AND_SCC_MODEL.md`, `W046_INVALIDATION_SOFT_REFERENCE_DYNAMIC_REFERENCE_AND_REBIND_MODEL.md`, `W046_RECALC_TRACKER_TRANSITION_PRE_POST_MODEL.md`, `W046_EVALUATION_ORDER_AND_WORKING_VALUE_READ_DISCIPLINE_MODEL.md`, `W046_TRACECALC_REFINEMENT_KERNEL_AND_REPLAY_BINDING.md`, `W046_OXFML_SEAM_LET_LAMBDA_FORMATTING_PUBLICATION_AND_CALLABLE_BOUNDARY_MODEL.md`, `W046_PROOF_SERVICE_AND_EVIDENCE_CLASSIFIER_COVERAGE_LEDGER.md`, `W046_SCALE_PERFORMANCE_SEMANTIC_REGRESSION_SIGNATURES.md`, `W046_STAGE2_PACK_C5_OPERATED_SERVICE_AND_RELEASE_CONSEQUENCE_REASSESSMENT.md`, and `W046_CLOSURE_AUDIT_SEMANTIC_SPINE_COVERAGE_AND_SUCCESSOR_ROUTING.md`.
- `docs/spec/core-engine/CORE_ENGINE_OXFML_MINIMAL_UPSTREAM_HOST_INTERFACES.md`
  - supporting companion defining the first OxCalc-owned minimal upstream host packet and adapter used to drive OxFml in deterministic automated scaffolding, now exercised against OxFml V1 `consumer::runtime` and `consumer::replay`; reference material for downstream hosts, but not a production API freeze.

## Seed Test Corpus
- `docs/test-corpus/core-engine/tracecalc/README.md`
  - first checked-in self-contained `TraceCalc` scenario corpus.

## Archived Rewrite-Control Material
The rewrite-control artifacts used to establish the canonical set are preserved for provenance under:
- `archive/rewrite-control-2026-03/`

These files are historical planning and promotion-control artifacts, not active canonical guidance.

## Bootstrap Archive and Reference-Only Material
The previous bootstrap set is preserved under:
- `archive/bootstrap-2026-03/`

Bootstrap redirect/reference-only files remain in `docs/spec/core-engine/` for provenance and pointer stability.
Foundation snapshot files in `docs/spec/core-engine/` are local reference support, not OxCalc-owned canonical architecture.

## Visibility and Related Policy Docs
- `docs/spec/visibility/*`
  - retained for visibility-priority and formatting-boundary policy work.

## Consumed Mirror Set
- `docs/spec/fec-f3e/*`
  - copied from OxFml-owned canonical seam specs for local implementation reference.

## Mirror Policy
1. OxCalc owns its canonical core-engine spec set in this repo.
2. OxFml owns the canonical shared FEC/F3E seam specification.
3. Foundation retains doctrine and conformance-policy ownership and keeps read-only mirrors/snapshots for cross-program assurance.
