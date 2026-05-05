# W040 Formalization Artifact Root

This directory is reserved for W040 evidence packets.

W040 starts from `docs/spec/core-engine/w039-formalization/W039_CLOSURE_AUDIT_AND_RELEASE_GRADE_DECISION.md` and owns the next direct-verification tranche for remaining release-grade blockers: optimized/core exact blockers, Rust totality/refinement, Lean/TLA verification discharge, Stage 2 production partition policy, operated assurance services, retained history, independent evaluator implementation, operated cross-engine differential service, OxFml seam breadth, callable metadata, pack-grade replay governance, C5 reassessment, and release-grade decision.

The governing workset is `docs/worksets/W040_CORE_FORMALIZATION_RELEASE_GRADE_DIRECT_VERIFICATION.md`.

Current OxFml formatting intake:

1. W073 is now a direct-replacement input contract for aggregate/visualization metadata, not an additive fallback path.
2. W073 treats `VerificationConditionalFormattingRule.typed_rule` as the only accepted metadata source for `colorScale`, `dataBar`, `iconSet`, `top`, `bottom`, `aboveAverage`, and `belowAverage`.
3. W072 bounded `thresholds` strings are intentionally ignored for those aggregate and visualization families.
4. `thresholds` remains meaningful only for scalar/operator/expression rule families where threshold text is the actual rule input.
5. OxCalc W040 evidence must not assume a fallback from W073 typed metadata to W072 aggregate/visualization threshold strings.
6. The current OxFml focused evidence includes old-string non-interpretation rows for visualization and aggregate option strings.

Current packets:

1. `W040_RESIDUAL_DIRECT_VERIFICATION_OBLIGATION_MAP.md` - `calc-tv5.1` residual direct-verification obligation map.
2. `W040_OPTIMIZED_CORE_EXACT_BLOCKER_FIXES_AND_DIFFERENTIALS.md` - `calc-tv5.2` optimized/core exact blocker narrowing packet.
3. `W040_RUST_TOTALITY_AND_REFINEMENT_PROOF_TRANCHE.md` - `calc-tv5.3` Rust totality/refinement classification packet.
4. `W040_LEAN_TLA_FULL_VERIFICATION_DISCHARGE_TRANCHE.md` - `calc-tv5.4` Lean/TLA proof/model classification packet.
5. `W040_STAGE2_PRODUCTION_POLICY_AND_EQUIVALENCE_IMPLEMENTATION.md` - `calc-tv5.5` Stage 2 policy/equivalence packet.
6. `W040_OPERATED_ASSURANCE_AND_RETAINED_HISTORY_SERVICE_IMPLEMENTATION.md` - `calc-tv5.6` operated-assurance and retained-history service-artifact packet.
7. `W040_INDEPENDENT_EVALUATOR_IMPLEMENTATION_AND_OPERATED_DIFFERENTIAL.md` - `calc-tv5.7` bounded independent-evaluator diversity packet.
8. `W040_OXFML_SEAM_BREADTH_AND_CALLABLE_METADATA_IMPLEMENTATION.md` - current `calc-tv5.8` OxFml seam breadth and callable metadata packet.

Planned packets:

1. `W040_PACK_GRADE_REPLAY_GOVERNANCE_AND_C5_PROMOTION_DECISION.md`
2. `W040_CLOSURE_AUDIT_AND_RELEASE_GRADE_VERIFICATION_DECISION.md`
