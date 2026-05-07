# W044 Independent Evaluator Breadth Mismatch Quarantine And Differential Service Implementation

## Purpose

This packet records `calc-b1t.7`, the W044 diversity and mismatch-service tranche.

The tranche broadens the independent reference-model slice from W043 by adding deterministic soft-reference rebind and retained-witness attachment controls, then binds that slice against W044 optimized/core, Rust, Lean/TLA, Stage 2, operated-assurance, retained-history, retained-witness, W073 formatting, and pack-governance evidence.

It does not promote full independent evaluator breadth, operated cross-engine differential service, mismatch quarantine service, retained-witness attachment service, pack-grade replay, C5, Stage 2 production policy, broad OxFml closure, or release-grade verification.

## Evidence

| Artifact | Role |
| --- | --- |
| `archive/test-runs-core-engine-w038-w045/diversity-seam/w044-independent-evaluator-breadth-mismatch-quarantine-differential-service-001/run_summary.json` | W044 diversity-seam summary |
| `archive/test-runs-core-engine-w038-w045/diversity-seam/w044-independent-evaluator-breadth-mismatch-quarantine-differential-service-001/w044_independent_reference_model_implementation.json` | 8-case independent reference model |
| `archive/test-runs-core-engine-w038-w045/diversity-seam/w044-independent-evaluator-breadth-mismatch-quarantine-differential-service-001/w044_independent_evaluator_breadth_register.json` | independent-evaluator breadth classification |
| `archive/test-runs-core-engine-w038-w045/diversity-seam/w044-independent-evaluator-breadth-mismatch-quarantine-differential-service-001/w044_cross_engine_differential_service_register.json` | differential-service and service-blocker classification |
| `archive/test-runs-core-engine-w038-w045/diversity-seam/w044-independent-evaluator-breadth-mismatch-quarantine-differential-service-001/w044_mismatch_quarantine_authority_router.json` | mismatch-authority and retained-witness attachment classification |
| `archive/test-runs-core-engine-w038-w045/diversity-seam/w044-independent-evaluator-breadth-mismatch-quarantine-differential-service-001/w044_exact_diversity_blocker_register.json` | exact diversity/service blockers |
| `archive/test-runs-core-engine-w038-w045/diversity-seam/w044-independent-evaluator-breadth-mismatch-quarantine-differential-service-001/promotion_decision.json` | no-promotion decision |
| `archive/test-runs-core-engine-w038-w045/diversity-seam/w044-independent-evaluator-breadth-mismatch-quarantine-differential-service-001/validation.json` | validation status |

## Result Summary

The run `w044-independent-evaluator-breadth-mismatch-quarantine-differential-service-001` records:

1. 21 source-evidence rows.
2. 8 independent reference-model cases, all matching expected bindings.
3. 15 independent-evaluator breadth rows.
4. 16 cross-engine differential/service rows.
5. 15 mismatch-authority rows.
6. 25 accepted boundary rows.
7. 14 service-blocked rows.
8. 9 exact diversity/service blockers.
9. 0 failed rows.

The added reference-model controls are:

1. `reference_model.w044_soft_reference_rebind_control`: `Base=5`, `Target=Base*4`, `Resolved=20`, `Final=23`.
2. `reference_model.w044_retained_witness_attachment_control`: `Actual=21`, `Oracle=20`, `Mismatch=1`, `WitnessAttached=1`, `QuarantineRoute=1`.

These rows are deterministic control evidence for comparison and routing behavior. They are not broad dynamic-reference, scheduler, publication, service, or OxFml/OxFunc authority.

## W073 Formatting Intake

The packet carries the current OxFml W073 direct-replacement formatting rule:

1. `VerificationConditionalFormattingRule.typed_rule` is the only accepted metadata source for `colorScale`, `dataBar`, `iconSet`, `top`, `bottom`, `aboveAverage`, and `belowAverage`.
2. W072 bounded `thresholds` strings are intentionally ignored for those aggregate and visualization families.
3. `thresholds` remains meaningful for scalar/operator/expression rules where threshold text is the actual rule input.
4. Downstream typed-rule request construction remains required but unverified by OxCalc and is routed to `calc-b1t.8`.

No OxFml handoff is triggered by this packet.

## Exact Blockers

The packet retains these exact blockers:

1. `w044_diversity.full_independent_evaluator_breadth_absent`
2. `w044_diversity.operated_cross_engine_differential_service_absent`
3. `w044_diversity.mismatch_quarantine_service_absent`
4. `w044_diversity.stage2_operated_differential_dependency_absent`
5. `w044_diversity.retained_witness_attachment_service_absent`
6. `w044_diversity.oxfml_callable_public_migration_dependency_absent`
7. `w044_diversity.optimized_core_callable_metadata_dependency_absent`
8. `w044_diversity.formal_model_bound_dependency_absent`
9. `w044_diversity.pack_grade_replay_governance_dependency_absent`

## Semantic-Equivalence Statement

This W044 diversity runner extends the bounded independent named-reference model with soft-reference rebind and retained-witness attachment controls, binds W044 operated-assurance, Stage 2, proof/model, and optimized-core evidence, and classifies diversity, differential-service, mismatch-quarantine, retained-witness, and spec-evolution authority rows only.

It does not change evaluator kernels used by OxCalc, coordinator scheduling, recalc, publication, replay, pack, service, TraceCalc, TreeCalc, OxFml, OxFunc, Lean, or TLA semantics.

## Verification

Commands run:

1. `cargo fmt --all -- --check` - passed.
2. `cargo test -p oxcalc-tracecalc diversity_seam_runner_binds_w044_reference_model_without_service_promotion -- --nocapture` - passed.
3. `cargo run -p oxcalc-tracecalc-cli -- diversity-seam w044-independent-evaluator-breadth-mismatch-quarantine-differential-service-001` - passed.

Additional validation commands are recorded in the closure report for `calc-b1t.7`.

## Status

- execution_state: `calc-b1t.7_evidence_packet_recorded`
- scope_completeness: `scope_partial`
- target_completeness: `target_complete`
- integration_completeness: `integrated`
- open_lanes:
  - `calc-b1t.8` OxFml public migration, typed formatting request construction, callable, and registered-external uptake
  - `calc-b1t.9` release-scale replay/performance evidence under semantic guards
  - `calc-b1t.10` pack-grade replay governance service and C5 reassessment
  - `calc-b1t.11` closure audit and release-grade verification decision
  - operated cross-engine differential service, mismatch quarantine service, retained-witness attachment service, retained-history endpoint, retention SLO enforcement, full independent evaluator breadth, broad OxFml closure, pack-grade replay, C5, Stage 2 production policy, and release-grade verification remain unpromoted
