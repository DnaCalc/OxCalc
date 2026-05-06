# W045 Formalization Artifact Root

This directory is reserved for W045 evidence packets.

W045 starts from `docs/spec/core-engine/w044-formalization/W044_CLOSURE_AUDIT_AND_RELEASE_GRADE_VERIFICATION_DECISION.md` and owns the next service and cross-repo uptake verification tranche for remaining release-grade blockers: optimized/core counterpart coverage, callable metadata projection, Rust totality/refinement, panic-free core boundaries, full Lean/TLA verification, unbounded fairness, Stage 2 production partition-analyzer soundness, scheduler equivalence, pack-grade replay equivalence, operated assurance services, retained history, retained witness lifecycle, retention SLO, mismatch quarantine, independent evaluator breadth, operated cross-engine differential service, broad OxFml display/publication, public migration, W073 downstream typed-rule request-construction uptake, callable carrier sufficiency, registered-external callable projection, provider-failure/callable-publication publication semantics, continuous release-scale assurance, pack-grade replay governance, C5 reassessment, and release-grade decision.

The governing workset is `docs/worksets/W045_CORE_FORMALIZATION_RELEASE_GRADE_SERVICE_AND_CROSS_REPO_UPTAKE_VERIFICATION.md`.

## Current OxFml Intake

1. W073 remains a direct-replacement input contract for aggregate/visualization metadata, not an additive fallback path.
2. W073 treats `VerificationConditionalFormattingRule.typed_rule` as the only accepted metadata source for `colorScale`, `dataBar`, `iconSet`, `top`, `bottom`, `aboveAverage`, and `belowAverage`.
3. W072 bounded `thresholds` strings are intentionally ignored for those aggregate and visualization families.
4. `thresholds` remains meaningful only for scalar/operator/expression rule families where threshold text is the actual rule input.
5. DNA OneCalc W073 typed-rule request construction remains required downstream uptake and is not verified by OxCalc.
6. Ordinary OxFml downstream use should target `consumer::runtime`, `consumer::editor`, and `consumer::replay`; public `substrate::...` access is not an ordinary integration contract.
7. `format_delta` and `display_delta` remain distinct seam categories.
8. `LET`/`LAMBDA` remains the narrow OxCalc/OxFml/OxFunc carrier seam inside the OxCalc formalization scope; general OxFunc kernels remain outside this repo's scope.
9. No direct OxFml edits are made from W045.

## Planned Packets

1. `W045_RESIDUAL_RELEASE_GRADE_SUCCESSOR_OBLIGATION_AND_CURRENT_OXFML_INTAKE_MAP.md` - `calc-zkio.1` residual map, validated.
2. `W045_OPTIMIZED_CORE_COUNTERPART_COVERAGE_AND_CALLABLE_METADATA_PROJECTION_CLOSURE.md` - `calc-zkio.2` optimized/core and callable metadata tranche, validated.
3. `W045_RUST_TOTALITY_REFINEMENT_AND_PANIC_SURFACE_HARDENING.md` - `calc-zkio.3` Rust totality/refinement tranche, validated.
4. `W045_LEAN_TLA_VERIFICATION_FAIRNESS_AND_TOTALITY_DISCHARGE.md` - `calc-zkio.4` Lean/TLA proof/model tranche, validated.
5. `W045_STAGE2_PRODUCTION_PARTITION_AND_PACK_GRADE_EQUIVALENCE_SERVICE_EVIDENCE.md` - `calc-zkio.5` Stage 2 tranche.
6. `W045_OPERATED_ASSURANCE_RETAINED_HISTORY_RETAINED_WITNESS_SLO_SERVICE_IMPLEMENTATION.md` - `calc-zkio.6` operated assurance/service tranche.
7. `W045_INDEPENDENT_EVALUATOR_BREADTH_MISMATCH_QUARANTINE_AND_OPERATED_DIFFERENTIAL_SERVICE.md` - `calc-zkio.7` diversity/service tranche.
8. `W045_OXFML_PUBLIC_SURFACE_W073_DOWNSTREAM_TYPED_FORMATTING_CALLABLE_AND_REGISTERED_EXTERNAL_UPTAKE.md` - `calc-zkio.8` OxFml seam tranche.
9. `W045_CONTINUOUS_RELEASE_SCALE_ASSURANCE_AND_SEMANTIC_REGRESSION_SERVICE.md` - `calc-zkio.9` scale assurance tranche.
10. `W045_PACK_GRADE_REPLAY_GOVERNANCE_SERVICE_AND_C5_REASSESSMENT.md` - `calc-zkio.10` pack/C5 tranche.
11. `W045_CLOSURE_AUDIT_AND_RELEASE_GRADE_VERIFICATION_DECISION.md` - `calc-zkio.11` closure audit and release-grade decision.

## Current Status

- execution_state: `calc-zkio.5_ready_stage2_production_partition_and_pack_grade_equivalence_service_evidence`
- scope_completeness: `scope_partial`
- target_completeness: `target_partial`
- integration_completeness: `partial`
- open_lanes:
  - `calc-zkio.1` residual successor obligation and current OxFml intake map is validated
  - `calc-zkio.2` optimized/core and callable metadata tranche is validated
  - `calc-zkio.3` Rust totality/refinement and panic-surface tranche is validated
  - `calc-zkio.4` Lean/TLA verification, fairness, and totality discharge is validated
  - `calc-zkio.5` Stage 2 production partition and pack-grade equivalence service evidence is ready next
  - release-grade verification, full formalization, C5, pack-grade replay, Stage 2 production policy, operated services, independent evaluator breadth, broad OxFml/public migration, W073 downstream uptake, callable metadata, continuous scale assurance, and general OxFunc kernels remain unpromoted or external as classified
