# W046 Formalization Artifact Root

This directory is reserved for W046 engine semantic proof-spine packets.

W046 starts from `archive/w045-formalization/W045_CLOSURE_AUDIT_AND_RELEASE_GRADE_VERIFICATION_DECISION.md` and the showcase review in `docs/showcase/`. It redirects the next formalization tranche toward first-class semantic models of the calculation engine before returning to downstream evidence classification.

The governing workset is `docs/worksets/W046_CORE_FORMALIZATION_ENGINE_SEMANTIC_PROOF_SPINE.md`.

## Governing Intent

W046 treats the calculation engine as the primary formal object:

1. dependency graph construction,
2. forward/reverse edge consistency,
3. SCC and cycle classification,
4. invalidation seeds and closure,
5. soft-reference and dynamic-reference rebind behavior,
6. recalc tracker state transitions,
7. evaluation order and working-value read discipline,
8. OxFml runtime candidate adaptation,
9. rejection and publication semantics,
10. TraceCalc refinement for selected kernels.

Proof-service, release-grade, C5, operated-service, pack-governance, independent-evaluator, scale, and promotion-readiness lanes are supporting evidence layers over that semantic spine.

## Current OxFml Intake

1. W073 remains a direct-replacement input contract for aggregate/visualization metadata, not an additive fallback path.
2. W073 treats `VerificationConditionalFormattingRule.typed_rule` as the only accepted metadata source for `colorScale`, `dataBar`, `iconSet`, `top`, `bottom`, `aboveAverage`, and `belowAverage`.
3. W072 bounded `thresholds` strings are intentionally ignored for those aggregate and visualization families.
4. `thresholds` remains meaningful only for scalar/operator/expression rule families where threshold text is the actual rule input.
5. DNA OneCalc W073 typed-rule request construction remains required downstream uptake unless W046 obtains direct current evidence.
6. Ordinary OxFml downstream use should target `consumer::runtime`, `consumer::editor`, and `consumer::replay`; public `substrate::...` access is not an ordinary integration contract.
7. `format_delta` and `display_delta` remain distinct seam categories.
8. `LET`/`LAMBDA` remains the narrow OxCalc/OxFml/OxFunc carrier seam inside the OxCalc formalization scope; general OxFunc kernels remain outside this repo's scope.
9. No direct OxFml edits are made from W046.

## Planned Packets

1. `W046_ENGINE_SEMANTIC_CATALOG_AND_EFFECT_SIGNATURE_PLAN.md` - `calc-gucd.1` redirect, showcase finding uptake, semantic catalog, and algebraic-effects handler-law plan.
2. `W046_SEMANTIC_FRAGMENT_REVIEW_LEDGER.md` - `calc-gucd.1` first-pass review ledger mapping engine fragments to implementation, specs, formal artifacts, evidence, and successor beads.
3. `W046_ENGINE_STATE_TRANSITION_CATALOG.md` - `calc-gucd.1` transition catalog naming state objects, transitions, invariants, evidence inputs, and owner beads.
4. `W046_DEPENDENCY_GRAPH_REVERSE_EDGE_AND_SCC_MODEL.md` - `calc-gucd.2` graph, reverse-edge, and cycle/SCC model.
5. `W046_INVALIDATION_SOFT_REFERENCE_DYNAMIC_REFERENCE_AND_REBIND_MODEL.md` - `calc-gucd.3` invalidation, soft-reference, dynamic-reference, and rebind model.
6. `W046_RECALC_TRACKER_TRANSITION_PRE_POST_MODEL.md` - `calc-gucd.4` recalc state transition model.
7. `W046_EVALUATION_ORDER_AND_WORKING_VALUE_READ_DISCIPLINE_MODEL.md` - `calc-gucd.5` evaluation order and read-discipline model.
8. `W046_TRACECALC_REFINEMENT_KERNEL_AND_REPLAY_BINDING.md` - `calc-gucd.6` TraceCalc reference kernel and TreeCalc/CoreEngine replay binding.
9. `W046_OXFML_SEAM_LET_LAMBDA_FORMATTING_PUBLICATION_AND_CALLABLE_BOUNDARY_MODEL.md` - `calc-gucd.7` OxFml seam and narrow carrier model.
10. `W046_PROOF_SERVICE_AND_EVIDENCE_CLASSIFIER_COVERAGE_LEDGER.md` - `calc-gucd.8` proof-service/classifier coverage recast over the semantic spine.
11. `W046_SCALE_PERFORMANCE_SEMANTIC_REGRESSION_SIGNATURES.md` - `calc-gucd.9` scale/performance semantic-regression signatures and phase timing.
12. `W046_STAGE2_PACK_C5_OPERATED_SERVICE_AND_RELEASE_CONSEQUENCE_REASSESSMENT.md` - `calc-gucd.10` downstream consequence reassessment after semantic-spine evidence.
13. `W046_CLOSURE_AUDIT_SEMANTIC_SPINE_COVERAGE_AND_SUCCESSOR_ROUTING.md` - `calc-gucd.11` closure audit and successor routing.

## Current Status

- execution_state: `calc-gucd.1_state_transition_catalog_added`
- scope_completeness: `scope_partial`
- target_completeness: `target_partial`
- integration_completeness: `partial`
- open_lanes:
  - `calc-gucd.1` semantic-spine redirect, catalog promotion, fragment review, transition catalog, and effect-signature plan validation is in progress
  - graph/reverse-edge/SCC formal model remains open
  - invalidation/soft-reference/rebind formal model remains open
  - recalc tracker and evaluation-order formal models remain open
  - TraceCalc refinement kernel and TreeCalc/CoreEngine replay binding remain open
  - OxFml seam, `LET`/`LAMBDA` carrier, and formatting/publication model remains open
  - release-grade verification, C5, pack-grade replay, Stage 2 production policy, operated services, independent evaluator breadth, broad OxFml/public migration, W073 downstream uptake, continuous scale assurance, and general OxFunc kernels remain unpromoted or external as classified
