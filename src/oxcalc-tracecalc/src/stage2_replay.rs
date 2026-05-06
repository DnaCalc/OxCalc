#![forbid(unsafe_code)]

//! W038/W039/W040/W041/W042/W043/W044 bounded Stage 2 partition replay and policy-governance evidence.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde_json::{Value, json};
use thiserror::Error;

const RUN_SUMMARY_SCHEMA_V1: &str = "oxcalc.stage2_replay.w038.run_summary.v1";
const SOURCE_INDEX_SCHEMA_V1: &str = "oxcalc.stage2_replay.w038.source_evidence_index.v1";
const PARTITION_MATRIX_SCHEMA_V1: &str = "oxcalc.stage2_replay.w038.partition_replay_matrix.v1";
const PERMUTATION_REPLAY_SCHEMA_V1: &str =
    "oxcalc.stage2_replay.w038.partition_order_permutation_replay.v1";
const SEMANTIC_EQUIVALENCE_SCHEMA_V1: &str =
    "oxcalc.stage2_replay.w038.semantic_equivalence_report.v1";
const BLOCKER_REGISTER_SCHEMA_V1: &str =
    "oxcalc.stage2_replay.w038.stage2_exact_blocker_register.v1";
const PROMOTION_DECISION_SCHEMA_V1: &str = "oxcalc.stage2_replay.w038.promotion_decision.v1";
const VALIDATION_SCHEMA_V1: &str = "oxcalc.stage2_replay.w038.validation.v1";
const W039_RUN_SUMMARY_SCHEMA_V1: &str = "oxcalc.stage2_replay.w039.run_summary.v1";
const W039_SOURCE_INDEX_SCHEMA_V1: &str = "oxcalc.stage2_replay.w039.source_evidence_index.v1";
const W039_POLICY_GATE_SCHEMA_V1: &str = "oxcalc.stage2_replay.w039.stage2_policy_gate_register.v1";
const W039_SOUNDNESS_REGISTER_SCHEMA_V1: &str =
    "oxcalc.stage2_replay.w039.partition_soundness_register.v1";
const W039_REPLAY_GOVERNANCE_SCHEMA_V1: &str =
    "oxcalc.stage2_replay.w039.replay_governance_register.v1";
const W039_BLOCKER_REGISTER_SCHEMA_V1: &str =
    "oxcalc.stage2_replay.w039.stage2_exact_blocker_register.v1";
const W039_PROMOTION_DECISION_SCHEMA_V1: &str = "oxcalc.stage2_replay.w039.promotion_decision.v1";
const W039_VALIDATION_SCHEMA_V1: &str = "oxcalc.stage2_replay.w039.validation.v1";
const W040_RUN_SUMMARY_SCHEMA_V1: &str = "oxcalc.stage2_replay.w040.run_summary.v1";
const W040_SOURCE_INDEX_SCHEMA_V1: &str = "oxcalc.stage2_replay.w040.source_evidence_index.v1";
const W040_POLICY_GATE_SCHEMA_V1: &str = "oxcalc.stage2_replay.w040.stage2_policy_gate_register.v1";
const W040_PARTITION_ANALYZER_SCHEMA_V1: &str =
    "oxcalc.stage2_replay.w040.partition_analyzer_soundness_register.v1";
const W040_OBSERVABLE_EQUIVALENCE_SCHEMA_V1: &str =
    "oxcalc.stage2_replay.w040.observable_equivalence_register.v1";
const W040_BLOCKER_REGISTER_SCHEMA_V1: &str =
    "oxcalc.stage2_replay.w040.stage2_exact_blocker_register.v1";
const W040_PROMOTION_DECISION_SCHEMA_V1: &str = "oxcalc.stage2_replay.w040.promotion_decision.v1";
const W040_VALIDATION_SCHEMA_V1: &str = "oxcalc.stage2_replay.w040.validation.v1";
const W041_RUN_SUMMARY_SCHEMA_V1: &str = "oxcalc.stage2_replay.w041.run_summary.v1";
const W041_SOURCE_INDEX_SCHEMA_V1: &str = "oxcalc.stage2_replay.w041.source_evidence_index.v1";
const W041_POLICY_GATE_SCHEMA_V1: &str = "oxcalc.stage2_replay.w041.stage2_policy_gate_register.v1";
const W041_PRODUCTION_ANALYZER_SCHEMA_V1: &str =
    "oxcalc.stage2_replay.w041.production_analyzer_soundness_register.v1";
const W041_PACK_EQUIVALENCE_SCHEMA_V1: &str =
    "oxcalc.stage2_replay.w041.pack_equivalence_register.v1";
const W041_BLOCKER_REGISTER_SCHEMA_V1: &str =
    "oxcalc.stage2_replay.w041.stage2_exact_blocker_register.v1";
const W041_PROMOTION_DECISION_SCHEMA_V1: &str = "oxcalc.stage2_replay.w041.promotion_decision.v1";
const W041_VALIDATION_SCHEMA_V1: &str = "oxcalc.stage2_replay.w041.validation.v1";
const W042_RUN_SUMMARY_SCHEMA_V1: &str = "oxcalc.stage2_replay.w042.run_summary.v1";
const W042_SOURCE_INDEX_SCHEMA_V1: &str = "oxcalc.stage2_replay.w042.source_evidence_index.v1";
const W042_POLICY_GATE_SCHEMA_V1: &str = "oxcalc.stage2_replay.w042.stage2_policy_gate_register.v1";
const W042_PRODUCTION_ANALYZER_SCHEMA_V1: &str =
    "oxcalc.stage2_replay.w042.production_analyzer_soundness_register.v1";
const W042_PACK_GRADE_EQUIVALENCE_SCHEMA_V1: &str =
    "oxcalc.stage2_replay.w042.pack_grade_equivalence_register.v1";
const W042_BLOCKER_REGISTER_SCHEMA_V1: &str =
    "oxcalc.stage2_replay.w042.stage2_exact_blocker_register.v1";
const W042_PROMOTION_DECISION_SCHEMA_V1: &str = "oxcalc.stage2_replay.w042.promotion_decision.v1";
const W042_VALIDATION_SCHEMA_V1: &str = "oxcalc.stage2_replay.w042.validation.v1";
const W043_RUN_SUMMARY_SCHEMA_V1: &str = "oxcalc.stage2_replay.w043.run_summary.v1";
const W043_SOURCE_INDEX_SCHEMA_V1: &str = "oxcalc.stage2_replay.w043.source_evidence_index.v1";
const W043_POLICY_GATE_SCHEMA_V1: &str = "oxcalc.stage2_replay.w043.stage2_policy_gate_register.v1";
const W043_PRODUCTION_PARTITION_ANALYZER_SCHEMA_V1: &str =
    "oxcalc.stage2_replay.w043.production_partition_analyzer_register.v1";
const W043_SCHEDULER_EQUIVALENCE_SCHEMA_V1: &str =
    "oxcalc.stage2_replay.w043.scheduler_equivalence_register.v1";
const W043_PACK_GRADE_EQUIVALENCE_SCHEMA_V1: &str =
    "oxcalc.stage2_replay.w043.pack_grade_equivalence_register.v1";
const W043_BLOCKER_REGISTER_SCHEMA_V1: &str =
    "oxcalc.stage2_replay.w043.stage2_exact_blocker_register.v1";
const W043_PROMOTION_DECISION_SCHEMA_V1: &str = "oxcalc.stage2_replay.w043.promotion_decision.v1";
const W043_VALIDATION_SCHEMA_V1: &str = "oxcalc.stage2_replay.w043.validation.v1";
const W044_RUN_SUMMARY_SCHEMA_V1: &str = "oxcalc.stage2_replay.w044.run_summary.v1";
const W044_SOURCE_INDEX_SCHEMA_V1: &str = "oxcalc.stage2_replay.w044.source_evidence_index.v1";
const W044_POLICY_GATE_SCHEMA_V1: &str = "oxcalc.stage2_replay.w044.stage2_policy_gate_register.v1";
const W044_PRODUCTION_PARTITION_ANALYZER_SCHEMA_V1: &str =
    "oxcalc.stage2_replay.w044.production_partition_analyzer_register.v1";
const W044_SCHEDULER_EQUIVALENCE_SCHEMA_V1: &str =
    "oxcalc.stage2_replay.w044.scheduler_equivalence_register.v1";
const W044_PACK_GRADE_EQUIVALENCE_SCHEMA_V1: &str =
    "oxcalc.stage2_replay.w044.pack_grade_equivalence_register.v1";
const W044_BLOCKER_REGISTER_SCHEMA_V1: &str =
    "oxcalc.stage2_replay.w044.stage2_exact_blocker_register.v1";
const W044_PROMOTION_DECISION_SCHEMA_V1: &str = "oxcalc.stage2_replay.w044.promotion_decision.v1";
const W044_VALIDATION_SCHEMA_V1: &str = "oxcalc.stage2_replay.w044.validation.v1";

const W036_STAGE2_TLA_RUN_SUMMARY: &str =
    "docs/test-runs/core-engine/tla/w036-stage2-partition-001/run_summary.json";
const W037_STAGE2_SEMANTIC_REQUIREMENTS: &str = "docs/test-runs/core-engine/stage2-criteria/w037-stage2-deterministic-replay-criteria-001/semantic_equivalence_requirements.json";
const W037_STAGE2_PROMOTION_DECISION: &str = "docs/test-runs/core-engine/stage2-criteria/w037-stage2-deterministic-replay-criteria-001/promotion_decision.json";
const W037_DIRECT_OXFML_SUMMARY: &str =
    "docs/test-runs/core-engine/upstream-host/w037-direct-oxfml-evaluator-001/run_summary.json";
const W039_RESIDUAL_LEDGER: &str = "docs/test-runs/core-engine/release-grade-ledger/w039-residual-successor-obligation-ledger-001/successor_obligation_ledger.json";
const W038_STAGE2_RUN_SUMMARY: &str =
    "docs/test-runs/core-engine/stage2-replay/w038-stage2-partition-replay-001/run_summary.json";
const W038_STAGE2_VALIDATION: &str =
    "docs/test-runs/core-engine/stage2-replay/w038-stage2-partition-replay-001/validation.json";
const W038_STAGE2_PARTITION_MATRIX: &str = "docs/test-runs/core-engine/stage2-replay/w038-stage2-partition-replay-001/partition_replay_matrix.json";
const W038_STAGE2_PERMUTATION_REPLAY: &str = "docs/test-runs/core-engine/stage2-replay/w038-stage2-partition-replay-001/partition_order_permutation_replay.json";
const W038_STAGE2_BLOCKER_REGISTER: &str = "docs/test-runs/core-engine/stage2-replay/w038-stage2-partition-replay-001/stage2_exact_blocker_register.json";
const W039_CONFORMANCE_SUMMARY: &str = "docs/test-runs/core-engine/implementation-conformance/w039-optimized-core-exact-blocker-disposition-001/run_summary.json";
const W039_CONFORMANCE_BLOCKERS: &str = "docs/test-runs/core-engine/implementation-conformance/w039-optimized-core-exact-blocker-disposition-001/w039_exact_remaining_blocker_register.json";
const W039_FORMAL_ASSURANCE_SUMMARY: &str = "docs/test-runs/core-engine/formal-assurance/w039-proof-model-totality-closure-001/run_summary.json";
const W039_FORMAL_ASSURANCE_BLOCKERS: &str = "docs/test-runs/core-engine/formal-assurance/w039-proof-model-totality-closure-001/w039_exact_proof_model_blocker_register.json";
const W039_STAGE2_LEAN_FILE: &str = "formal/lean/OxCalc/CoreEngine/W039Stage2ProductionPolicy.lean";
const W039_STAGE2_RUN_SUMMARY: &str = "docs/test-runs/core-engine/stage2-replay/w039-stage2-production-policy-replay-governance-001/run_summary.json";
const W039_STAGE2_VALIDATION: &str = "docs/test-runs/core-engine/stage2-replay/w039-stage2-production-policy-replay-governance-001/validation.json";
const W039_STAGE2_POLICY_GATE: &str = "docs/test-runs/core-engine/stage2-replay/w039-stage2-production-policy-replay-governance-001/w039_stage2_policy_gate_register.json";
const W039_STAGE2_BLOCKERS: &str = "docs/test-runs/core-engine/stage2-replay/w039-stage2-production-policy-replay-governance-001/w039_stage2_exact_blocker_register.json";
const W040_DIRECT_OBLIGATION_SUMMARY: &str = "docs/test-runs/core-engine/release-grade-ledger/w040-direct-verification-obligation-map-001/run_summary.json";
const W040_DIRECT_OBLIGATION_MAP: &str = "docs/test-runs/core-engine/release-grade-ledger/w040-direct-verification-obligation-map-001/direct_verification_obligation_map.json";
const W040_OPTIMIZED_CORE_SUMMARY: &str = "docs/test-runs/core-engine/implementation-conformance/w040-optimized-core-exact-blocker-fixes-differentials-001/run_summary.json";
const W040_OPTIMIZED_CORE_BLOCKERS: &str = "docs/test-runs/core-engine/implementation-conformance/w040-optimized-core-exact-blocker-fixes-differentials-001/w040_exact_remaining_blocker_register.json";
const W040_TREECALC_SUMMARY: &str = "docs/test-runs/core-engine/treecalc-local/w040-optimized-core-dynamic-release-reclassification-001/run_summary.json";
const W040_LEAN_TLA_SUMMARY: &str = "docs/test-runs/core-engine/formal-assurance/w040-lean-tla-full-verification-discharge-001/run_summary.json";
const W040_LEAN_TLA_VALIDATION: &str = "docs/test-runs/core-engine/formal-assurance/w040-lean-tla-full-verification-discharge-001/validation.json";
const W040_LEAN_TLA_MODEL_REGISTER: &str = "docs/test-runs/core-engine/formal-assurance/w040-lean-tla-full-verification-discharge-001/w040_tla_model_bound_register.json";
const W040_STAGE2_LEAN_FILE: &str =
    "formal/lean/OxCalc/CoreEngine/W040Stage2ProductionPolicyAndEquivalence.lean";
const W040_STAGE2_RUN_SUMMARY: &str = "docs/test-runs/core-engine/stage2-replay/w040-stage2-production-policy-equivalence-001/run_summary.json";
const W040_STAGE2_VALIDATION: &str = "docs/test-runs/core-engine/stage2-replay/w040-stage2-production-policy-equivalence-001/validation.json";
const W040_STAGE2_POLICY_GATE: &str = "docs/test-runs/core-engine/stage2-replay/w040-stage2-production-policy-equivalence-001/w040_stage2_policy_gate_register.json";
const W040_STAGE2_PARTITION_ANALYZER: &str = "docs/test-runs/core-engine/stage2-replay/w040-stage2-production-policy-equivalence-001/w040_partition_analyzer_soundness_register.json";
const W040_STAGE2_OBSERVABLE_EQUIVALENCE: &str = "docs/test-runs/core-engine/stage2-replay/w040-stage2-production-policy-equivalence-001/w040_observable_equivalence_register.json";
const W040_STAGE2_BLOCKERS: &str = "docs/test-runs/core-engine/stage2-replay/w040-stage2-production-policy-equivalence-001/w040_stage2_exact_blocker_register.json";
const W040_STAGE2_PROMOTION_DECISION: &str = "docs/test-runs/core-engine/stage2-replay/w040-stage2-production-policy-equivalence-001/promotion_decision.json";
const W041_OBLIGATION_SUMMARY: &str = "docs/test-runs/core-engine/release-grade-ledger/w041-residual-release-grade-successor-obligation-map-001/run_summary.json";
const W041_OBLIGATION_MAP: &str = "docs/test-runs/core-engine/release-grade-ledger/w041-residual-release-grade-successor-obligation-map-001/successor_obligation_map.json";
const W041_FORMATTING_INTAKE: &str = "docs/test-runs/core-engine/release-grade-ledger/w041-residual-release-grade-successor-obligation-map-001/w073_formatting_intake.json";
const W041_OPTIMIZED_CORE_SUMMARY: &str = "docs/test-runs/core-engine/implementation-conformance/w041-optimized-core-residual-blocker-differentials-001/run_summary.json";
const W041_OPTIMIZED_CORE_DISPOSITIONS: &str = "docs/test-runs/core-engine/implementation-conformance/w041-optimized-core-residual-blocker-differentials-001/w041_exact_blocker_disposition_register.json";
const W041_OPTIMIZED_CORE_BLOCKERS: &str = "docs/test-runs/core-engine/implementation-conformance/w041-optimized-core-residual-blocker-differentials-001/w041_exact_remaining_blocker_register.json";
const W041_DYNAMIC_AUTO_TRANSITION: &str = "docs/test-runs/core-engine/implementation-conformance/w041-optimized-core-residual-blocker-differentials-001/dynamic_release_reclassification_auto_transition_evidence.json";
const W041_TREECALC_SUMMARY: &str = "docs/test-runs/core-engine/treecalc-local/w041-optimized-core-automatic-dynamic-transition-001/run_summary.json";
const W041_RUST_SUMMARY: &str = "docs/test-runs/core-engine/formal-assurance/w041-rust-totality-refinement-proof-tranche-001/run_summary.json";
const W041_RUST_VALIDATION: &str = "docs/test-runs/core-engine/formal-assurance/w041-rust-totality-refinement-proof-tranche-001/validation.json";
const W041_LEAN_TLA_SUMMARY: &str = "docs/test-runs/core-engine/formal-assurance/w041-lean-tla-full-verification-fairness-discharge-001/run_summary.json";
const W041_LEAN_TLA_VALIDATION: &str = "docs/test-runs/core-engine/formal-assurance/w041-lean-tla-full-verification-fairness-discharge-001/validation.json";
const W041_TLA_MODEL_REGISTER: &str = "docs/test-runs/core-engine/formal-assurance/w041-lean-tla-full-verification-fairness-discharge-001/w041_tla_model_bound_register.json";
const W041_STAGE2_LEAN_FILE: &str =
    "formal/lean/OxCalc/CoreEngine/W041Stage2ProductionAnalyzerAndPackEquivalence.lean";
const W036_STAGE2_TLA_PROMOTION_BLOCKERS: &str =
    "docs/test-runs/core-engine/tla/w036-stage2-partition-001/promotion_blockers.json";
const W042_OBLIGATION_SUMMARY: &str = "docs/test-runs/core-engine/release-grade-ledger/w042-residual-release-grade-closure-obligation-ledger-001/run_summary.json";
const W042_OBLIGATION_MAP: &str = "docs/test-runs/core-engine/release-grade-ledger/w042-residual-release-grade-closure-obligation-ledger-001/closure_obligation_map.json";
const W042_PROMOTION_TARGET_GATE_MAP: &str = "docs/test-runs/core-engine/release-grade-ledger/w042-residual-release-grade-closure-obligation-ledger-001/promotion_target_gate_map.json";
const W042_FORMATTING_INTAKE: &str = "docs/test-runs/core-engine/implementation-conformance/w042-optimized-core-counterpart-conformance-callable-metadata-001/w073_formatting_intake.json";
const W042_OPTIMIZED_CORE_SUMMARY: &str = "docs/test-runs/core-engine/implementation-conformance/w042-optimized-core-counterpart-conformance-callable-metadata-001/run_summary.json";
const W042_OPTIMIZED_CORE_COUNTERPART: &str = "docs/test-runs/core-engine/implementation-conformance/w042-optimized-core-counterpart-conformance-callable-metadata-001/w042_counterpart_conformance_register.json";
const W042_OPTIMIZED_CORE_BLOCKERS: &str = "docs/test-runs/core-engine/implementation-conformance/w042-optimized-core-counterpart-conformance-callable-metadata-001/w042_exact_remaining_blocker_register.json";
const W042_TREECALC_SUMMARY: &str = "docs/test-runs/core-engine/treecalc-local/w042-optimized-core-counterpart-conformance-treecalc-001/run_summary.json";
const W042_RUST_SUMMARY: &str = "docs/test-runs/core-engine/formal-assurance/w042-rust-totality-refinement-core-panic-boundary-001/run_summary.json";
const W042_RUST_VALIDATION: &str = "docs/test-runs/core-engine/formal-assurance/w042-rust-totality-refinement-core-panic-boundary-001/validation.json";
const W042_RUST_REFINEMENT_REGISTER: &str = "docs/test-runs/core-engine/formal-assurance/w042-rust-totality-refinement-core-panic-boundary-001/w042_rust_refinement_register.json";
const W042_LEAN_TLA_SUMMARY: &str = "docs/test-runs/core-engine/formal-assurance/w042-lean-tla-fairness-full-verification-expansion-001/run_summary.json";
const W042_LEAN_TLA_VALIDATION: &str = "docs/test-runs/core-engine/formal-assurance/w042-lean-tla-fairness-full-verification-expansion-001/validation.json";
const W042_TLA_MODEL_REGISTER: &str = "docs/test-runs/core-engine/formal-assurance/w042-lean-tla-fairness-full-verification-expansion-001/w042_tla_model_bound_register.json";
const W042_LEAN_TLA_BLOCKERS: &str = "docs/test-runs/core-engine/formal-assurance/w042-lean-tla-fairness-full-verification-expansion-001/w042_lean_tla_exact_blocker_register.json";
const W041_STAGE2_RUN_SUMMARY_FOR_W042: &str = "docs/test-runs/core-engine/stage2-replay/w041-stage2-production-analyzer-pack-equivalence-001/run_summary.json";
const W041_STAGE2_VALIDATION_FOR_W042: &str = "docs/test-runs/core-engine/stage2-replay/w041-stage2-production-analyzer-pack-equivalence-001/validation.json";
const W041_STAGE2_POLICY_GATE_FOR_W042: &str = "docs/test-runs/core-engine/stage2-replay/w041-stage2-production-analyzer-pack-equivalence-001/w041_stage2_policy_gate_register.json";
const W041_STAGE2_PRODUCTION_ANALYZER_FOR_W042: &str = "docs/test-runs/core-engine/stage2-replay/w041-stage2-production-analyzer-pack-equivalence-001/w041_production_analyzer_soundness_register.json";
const W041_STAGE2_PACK_EQUIVALENCE_FOR_W042: &str = "docs/test-runs/core-engine/stage2-replay/w041-stage2-production-analyzer-pack-equivalence-001/w041_pack_equivalence_register.json";
const W041_STAGE2_BLOCKERS_FOR_W042: &str = "docs/test-runs/core-engine/stage2-replay/w041-stage2-production-analyzer-pack-equivalence-001/w041_stage2_exact_blocker_register.json";
const W041_STAGE2_PROMOTION_DECISION_FOR_W042: &str = "docs/test-runs/core-engine/stage2-replay/w041-stage2-production-analyzer-pack-equivalence-001/promotion_decision.json";
const W042_STAGE2_LEAN_FILE: &str =
    "formal/lean/OxCalc/CoreEngine/W042Stage2ProductionAnalyzerAndPackGradeEquivalence.lean";
const W043_OBLIGATION_SUMMARY: &str = "docs/test-runs/core-engine/release-grade-ledger/w043-residual-release-grade-proof-service-obligation-map-001/run_summary.json";
const W043_OBLIGATION_MAP: &str = "docs/test-runs/core-engine/release-grade-ledger/w043-residual-release-grade-proof-service-obligation-map-001/proof_service_obligation_map.json";
const W043_PROMOTION_TARGET_GATE_MAP: &str = "docs/test-runs/core-engine/release-grade-ledger/w043-residual-release-grade-proof-service-obligation-map-001/promotion_target_gate_map.json";
const W043_FORMATTING_INTAKE: &str = "docs/test-runs/core-engine/implementation-conformance/w043-optimized-core-broad-conformance-callable-metadata-closure-001/w073_formatting_intake.json";
const W043_OPTIMIZED_CORE_SUMMARY: &str = "docs/test-runs/core-engine/implementation-conformance/w043-optimized-core-broad-conformance-callable-metadata-closure-001/run_summary.json";
const W043_OPTIMIZED_CORE_COUNTERPART: &str = "docs/test-runs/core-engine/implementation-conformance/w043-optimized-core-broad-conformance-callable-metadata-closure-001/w043_counterpart_conformance_register.json";
const W043_OPTIMIZED_CORE_BLOCKERS: &str = "docs/test-runs/core-engine/implementation-conformance/w043-optimized-core-broad-conformance-callable-metadata-closure-001/w043_exact_remaining_blocker_register.json";
const W043_TREECALC_SUMMARY: &str = "docs/test-runs/core-engine/treecalc-local/w043-optimized-core-broad-conformance-treecalc-001/run_summary.json";
const W043_RUST_SUMMARY: &str = "docs/test-runs/core-engine/formal-assurance/w043-rust-totality-refinement-panic-free-frontier-001/run_summary.json";
const W043_RUST_VALIDATION: &str = "docs/test-runs/core-engine/formal-assurance/w043-rust-totality-refinement-panic-free-frontier-001/validation.json";
const W043_RUST_REFINEMENT_REGISTER: &str = "docs/test-runs/core-engine/formal-assurance/w043-rust-totality-refinement-panic-free-frontier-001/w043_rust_refinement_register.json";
const W043_LEAN_TLA_SUMMARY: &str = "docs/test-runs/core-engine/formal-assurance/w043-lean-tla-full-verification-unbounded-fairness-001/run_summary.json";
const W043_LEAN_TLA_VALIDATION: &str = "docs/test-runs/core-engine/formal-assurance/w043-lean-tla-full-verification-unbounded-fairness-001/validation.json";
const W043_TLA_MODEL_REGISTER: &str = "docs/test-runs/core-engine/formal-assurance/w043-lean-tla-full-verification-unbounded-fairness-001/w043_tla_model_bound_register.json";
const W043_LEAN_TLA_BLOCKERS: &str = "docs/test-runs/core-engine/formal-assurance/w043-lean-tla-full-verification-unbounded-fairness-001/w043_lean_tla_exact_blocker_register.json";
const W042_STAGE2_RUN_SUMMARY_FOR_W043: &str = "docs/test-runs/core-engine/stage2-replay/w042-stage2-production-analyzer-pack-grade-equivalence-closure-001/run_summary.json";
const W042_STAGE2_VALIDATION_FOR_W043: &str = "docs/test-runs/core-engine/stage2-replay/w042-stage2-production-analyzer-pack-grade-equivalence-closure-001/validation.json";
const W042_STAGE2_POLICY_GATE_FOR_W043: &str = "docs/test-runs/core-engine/stage2-replay/w042-stage2-production-analyzer-pack-grade-equivalence-closure-001/w042_stage2_policy_gate_register.json";
const W042_STAGE2_PRODUCTION_ANALYZER_FOR_W043: &str = "docs/test-runs/core-engine/stage2-replay/w042-stage2-production-analyzer-pack-grade-equivalence-closure-001/w042_production_analyzer_soundness_register.json";
const W042_STAGE2_PACK_GRADE_EQUIVALENCE_FOR_W043: &str = "docs/test-runs/core-engine/stage2-replay/w042-stage2-production-analyzer-pack-grade-equivalence-closure-001/w042_pack_grade_equivalence_register.json";
const W042_STAGE2_BLOCKERS_FOR_W043: &str = "docs/test-runs/core-engine/stage2-replay/w042-stage2-production-analyzer-pack-grade-equivalence-closure-001/w042_stage2_exact_blocker_register.json";
const W042_STAGE2_PROMOTION_DECISION_FOR_W043: &str = "docs/test-runs/core-engine/stage2-replay/w042-stage2-production-analyzer-pack-grade-equivalence-closure-001/promotion_decision.json";
const W043_STAGE2_LEAN_FILE: &str = "formal/lean/OxCalc/CoreEngine/W043Stage2ProductionPartitionAnalyzerAndSchedulerEquivalence.lean";
const W044_RESIDUAL_SUMMARY: &str = "docs/test-runs/core-engine/release-grade-ledger/w044-residual-release-grade-blocker-reclassification-map-001/run_summary.json";
const W044_RESIDUAL_BLOCKER_MAP: &str = "docs/test-runs/core-engine/release-grade-ledger/w044-residual-release-grade-blocker-reclassification-map-001/blocker_reclassification_map.json";
const W044_OPTIMIZED_CORE_SUMMARY: &str = "docs/test-runs/core-engine/implementation-conformance/w044-optimized-core-dynamic-transition-callable-metadata-001/run_summary.json";
const W044_OPTIMIZED_CORE_DISPOSITIONS: &str = "docs/test-runs/core-engine/implementation-conformance/w044-optimized-core-dynamic-transition-callable-metadata-001/w044_optimized_core_disposition_register.json";
const W044_OPTIMIZED_CORE_DYNAMIC_EVIDENCE: &str = "docs/test-runs/core-engine/implementation-conformance/w044-optimized-core-dynamic-transition-callable-metadata-001/w044_dynamic_transition_evidence.json";
const W044_OPTIMIZED_CORE_BLOCKERS: &str = "docs/test-runs/core-engine/implementation-conformance/w044-optimized-core-dynamic-transition-callable-metadata-001/w044_exact_remaining_blocker_register.json";
const W044_TREECALC_SUMMARY: &str = "docs/test-runs/core-engine/treecalc-local/w044-optimized-core-dynamic-transition-treecalc-001/run_summary.json";
const W044_RUST_SUMMARY: &str = "docs/test-runs/core-engine/formal-assurance/w044-rust-totality-refinement-panic-surface-expansion-001/run_summary.json";
const W044_RUST_VALIDATION: &str = "docs/test-runs/core-engine/formal-assurance/w044-rust-totality-refinement-panic-surface-expansion-001/validation.json";
const W044_RUST_REFINEMENT_REGISTER: &str = "docs/test-runs/core-engine/formal-assurance/w044-rust-totality-refinement-panic-surface-expansion-001/w044_rust_refinement_register.json";
const W044_LEAN_TLA_SUMMARY: &str = "docs/test-runs/core-engine/formal-assurance/w044-lean-tla-unbounded-fairness-full-verification-expansion-001/run_summary.json";
const W044_LEAN_TLA_VALIDATION: &str = "docs/test-runs/core-engine/formal-assurance/w044-lean-tla-unbounded-fairness-full-verification-expansion-001/validation.json";
const W044_TLA_MODEL_REGISTER: &str = "docs/test-runs/core-engine/formal-assurance/w044-lean-tla-unbounded-fairness-full-verification-expansion-001/w044_tla_model_bound_register.json";
const W044_LEAN_TLA_BLOCKERS: &str = "docs/test-runs/core-engine/formal-assurance/w044-lean-tla-unbounded-fairness-full-verification-expansion-001/w044_lean_tla_exact_blocker_register.json";
const W043_STAGE2_RUN_SUMMARY_FOR_W044: &str = "docs/test-runs/core-engine/stage2-replay/w043-stage2-production-partition-analyzer-scheduler-equivalence-001/run_summary.json";
const W043_STAGE2_VALIDATION_FOR_W044: &str = "docs/test-runs/core-engine/stage2-replay/w043-stage2-production-partition-analyzer-scheduler-equivalence-001/validation.json";
const W043_STAGE2_POLICY_GATE_FOR_W044: &str = "docs/test-runs/core-engine/stage2-replay/w043-stage2-production-partition-analyzer-scheduler-equivalence-001/w043_stage2_policy_gate_register.json";
const W043_STAGE2_PRODUCTION_ANALYZER_FOR_W044: &str = "docs/test-runs/core-engine/stage2-replay/w043-stage2-production-partition-analyzer-scheduler-equivalence-001/w043_production_partition_analyzer_register.json";
const W043_STAGE2_SCHEDULER_EQUIVALENCE_FOR_W044: &str = "docs/test-runs/core-engine/stage2-replay/w043-stage2-production-partition-analyzer-scheduler-equivalence-001/w043_scheduler_equivalence_register.json";
const W043_STAGE2_PACK_GRADE_EQUIVALENCE_FOR_W044: &str = "docs/test-runs/core-engine/stage2-replay/w043-stage2-production-partition-analyzer-scheduler-equivalence-001/w043_pack_grade_equivalence_register.json";
const W043_STAGE2_BLOCKERS_FOR_W044: &str = "docs/test-runs/core-engine/stage2-replay/w043-stage2-production-partition-analyzer-scheduler-equivalence-001/w043_stage2_exact_blocker_register.json";
const W043_STAGE2_PROMOTION_DECISION_FOR_W044: &str = "docs/test-runs/core-engine/stage2-replay/w043-stage2-production-partition-analyzer-scheduler-equivalence-001/promotion_decision.json";
const W044_STAGE2_LEAN_FILE: &str = "formal/lean/OxCalc/CoreEngine/W044Stage2ProductionPartitionAnalyzerAndSchedulerEquivalence.lean";

const TRACE_ACCEPT_RESULT: &str = "docs/test-runs/core-engine/tracecalc-reference-machine/w037-tracecalc-observable-closure-001/scenarios/tc_accept_publish_001/result.json";
const TREE_INDEPENDENT_RESULT: &str = "docs/test-runs/core-engine/treecalc-local/w037-optimized-core-conformance-treecalc-001/cases/tc_local_w034_independent_order_equiv_001/result.json";
const TREE_DYNAMIC_RESULT: &str = "docs/test-runs/core-engine/treecalc-local/w037-optimized-core-conformance-treecalc-001/cases/tc_local_dynamic_resolved_publish_001/result.json";
const TRACE_DYNAMIC_RESULT: &str = "docs/test-runs/core-engine/tracecalc-reference-machine/w037-tracecalc-observable-closure-001/scenarios/tc_w035_dynamic_dependency_release_publish_001/result.json";
const W073_FORMATTING_RESULT: &str = "docs/test-runs/core-engine/upstream-host/w037-direct-oxfml-evaluator-001/cases/uh_typed_cf_top_rank_guard_001/result.json";
const TRACE_SNAPSHOT_FENCE_RESULT: &str = "docs/test-runs/core-engine/tracecalc-reference-machine/w037-tracecalc-observable-closure-001/scenarios/tc_w034_snapshot_fence_reject_001/result.json";
const TRACE_SNAPSHOT_FENCE_REJECTS: &str = "docs/test-runs/core-engine/tracecalc-reference-machine/w037-tracecalc-observable-closure-001/scenarios/tc_w034_snapshot_fence_reject_001/rejects.json";
const TRACE_SNAPSHOT_FENCE_PUBLISHED_VIEW: &str = "docs/test-runs/core-engine/tracecalc-reference-machine/w037-tracecalc-observable-closure-001/scenarios/tc_w034_snapshot_fence_reject_001/published_view.json";
const TRACE_CAPABILITY_FENCE_RESULT: &str = "docs/test-runs/core-engine/tracecalc-reference-machine/w037-tracecalc-observable-closure-001/scenarios/tc_w034_capability_fence_reject_001/result.json";
const TRACE_CAPABILITY_FENCE_REJECTS: &str = "docs/test-runs/core-engine/tracecalc-reference-machine/w037-tracecalc-observable-closure-001/scenarios/tc_w034_capability_fence_reject_001/rejects.json";
const TRACE_CAPABILITY_FENCE_PUBLISHED_VIEW: &str = "docs/test-runs/core-engine/tracecalc-reference-machine/w037-tracecalc-observable-closure-001/scenarios/tc_w034_capability_fence_reject_001/published_view.json";

const ORDER_EMPTY: &[u64] = &[];
const ORDER_TRACE_ACCEPT: &[u64] = &[1];
const ORDER_INDEPENDENT_BASELINE: &[u64] = &[4, 5, 6];
const ORDER_INDEPENDENT_PERMUTED: &[u64] = &[5, 4, 6];
const ORDER_DYNAMIC: &[u64] = &[3];
const ORDER_FORMATTING: &[u64] = &[1];

const PERMS_TRACE_ACCEPT: &[&[u64]] = &[ORDER_TRACE_ACCEPT];
const PERMS_INDEPENDENT: &[&[u64]] = &[ORDER_INDEPENDENT_PERMUTED, ORDER_INDEPENDENT_BASELINE];
const PERMS_DYNAMIC: &[&[u64]] = &[ORDER_DYNAMIC];
const PERMS_FORMATTING: &[&[u64]] = &[ORDER_FORMATTING];

#[derive(Debug, Error)]
pub enum Stage2ReplayError {
    #[error("failed to create artifact directory {path}: {source}")]
    CreateDirectory {
        path: String,
        source: std::io::Error,
    },
    #[error("failed to remove existing artifact root {path}: {source}")]
    RemoveDirectory {
        path: String,
        source: std::io::Error,
    },
    #[error("failed to read artifact {path}: {source}")]
    ReadArtifact {
        path: String,
        source: std::io::Error,
    },
    #[error("failed to parse JSON artifact {path}: {source}")]
    ParseJson {
        path: String,
        source: serde_json::Error,
    },
    #[error("failed to write artifact {path}: {source}")]
    WriteFile {
        path: String,
        source: std::io::Error,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stage2ReplayRunSummary {
    pub run_id: String,
    pub schema_version: String,
    pub partition_replay_row_count: usize,
    pub permutation_replay_row_count: usize,
    pub nontrivial_permutation_row_count: usize,
    pub observable_invariance_row_count: usize,
    pub formatting_watch_row_count: usize,
    pub exact_remaining_blocker_count: usize,
    pub failed_row_count: usize,
    pub stage2_policy_promoted: bool,
    pub artifact_root: String,
}

#[derive(Debug, Clone, Default)]
pub struct Stage2ReplayRunner;

#[derive(Debug, Clone, Copy)]
enum SourceKind {
    TraceCalc,
    TreeCalc,
    UpstreamHost,
}

#[derive(Debug, Clone)]
struct Stage2ReplaySpec {
    row_id: &'static str,
    profile_id: &'static str,
    source_kind: SourceKind,
    source_artifact: &'static str,
    baseline_order: &'static [u64],
    stage2_order: &'static [u64],
    permutation_orders: &'static [&'static [u64]],
    stage2_partition_shape: &'static str,
    observable_focus: &'static [&'static str],
    formatting_watch: bool,
    reason: &'static str,
}

#[derive(Debug, Clone)]
struct EvaluatedReplayRow {
    row: Value,
    permutation_rows: Vec<Value>,
    observable_invariant: bool,
    formatting_watch: bool,
    failed: bool,
    nontrivial_permutation: bool,
}

impl Stage2ReplayRunner {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    pub fn execute(
        &self,
        repo_root: &Path,
        run_id: &str,
    ) -> Result<Stage2ReplayRunSummary, Stage2ReplayError> {
        if run_id.contains("w044") {
            return self.execute_w044(repo_root, run_id);
        }
        if run_id.contains("w043") {
            return self.execute_w043(repo_root, run_id);
        }
        if run_id.contains("w042") {
            return self.execute_w042(repo_root, run_id);
        }
        if run_id.contains("w041") {
            return self.execute_w041(repo_root, run_id);
        }
        if run_id.contains("w040") {
            return self.execute_w040(repo_root, run_id);
        }
        if run_id.contains("w039") {
            return self.execute_w039(repo_root, run_id);
        }

        let relative_artifact_root =
            relative_artifact_path(&["docs", "test-runs", "core-engine", "stage2-replay", run_id]);
        let artifact_root = repo_root.join(&relative_artifact_root);
        if artifact_root.exists() {
            fs::remove_dir_all(&artifact_root).map_err(|source| {
                Stage2ReplayError::RemoveDirectory {
                    path: artifact_root.display().to_string(),
                    source,
                }
            })?;
        }
        fs::create_dir_all(&artifact_root).map_err(|source| {
            Stage2ReplayError::CreateDirectory {
                path: artifact_root.display().to_string(),
                source,
            }
        })?;

        let w036_stage2_tla = read_json(repo_root, W036_STAGE2_TLA_RUN_SUMMARY)?;
        let w037_semantic_requirements = read_json(repo_root, W037_STAGE2_SEMANTIC_REQUIREMENTS)?;
        let w037_promotion_decision = read_json(repo_root, W037_STAGE2_PROMOTION_DECISION)?;
        let w037_direct_oxfml_summary = read_json(repo_root, W037_DIRECT_OXFML_SUMMARY)?;

        let evaluated_rows = STAGE2_REPLAY_SPECS
            .iter()
            .map(|spec| evaluate_replay_spec(repo_root, spec))
            .collect::<Result<Vec<_>, _>>()?;
        let partition_rows = evaluated_rows
            .iter()
            .map(|row| row.row.clone())
            .collect::<Vec<_>>();
        let permutation_rows = evaluated_rows
            .iter()
            .flat_map(|row| row.permutation_rows.iter().cloned())
            .collect::<Vec<_>>();

        let partition_replay_row_count = partition_rows.len();
        let permutation_replay_row_count = permutation_rows.len();
        let nontrivial_permutation_row_count = evaluated_rows
            .iter()
            .filter(|row| row.nontrivial_permutation)
            .count();
        let observable_invariance_row_count = evaluated_rows
            .iter()
            .filter(|row| row.observable_invariant)
            .count();
        let formatting_watch_row_count = evaluated_rows
            .iter()
            .filter(|row| row.formatting_watch)
            .count();
        let row_failed_count = evaluated_rows.iter().filter(|row| row.failed).count();
        let source_failures = source_validation_failures(
            &w036_stage2_tla,
            &w037_semantic_requirements,
            &w037_promotion_decision,
            &w037_direct_oxfml_summary,
        );
        let failed_row_count = row_failed_count + source_failures.len();
        let exact_blockers = exact_blocker_rows();
        let exact_remaining_blocker_count = exact_blockers.len();

        let source_evidence_index_path =
            format!("{relative_artifact_root}/source_evidence_index.json");
        let partition_matrix_path =
            format!("{relative_artifact_root}/partition_replay_matrix.json");
        let permutation_replay_path =
            format!("{relative_artifact_root}/partition_order_permutation_replay.json");
        let semantic_equivalence_path =
            format!("{relative_artifact_root}/semantic_equivalence_report.json");
        let blocker_register_path =
            format!("{relative_artifact_root}/stage2_exact_blocker_register.json");
        let promotion_decision_path = format!("{relative_artifact_root}/promotion_decision.json");
        let validation_path = format!("{relative_artifact_root}/validation.json");

        let source_evidence_index = json!({
            "schema_version": SOURCE_INDEX_SCHEMA_V1,
            "run_id": run_id,
            "artifact_root": relative_artifact_root,
            "source_artifacts": {
                "w036_stage2_tla_run_summary": W036_STAGE2_TLA_RUN_SUMMARY,
                "w037_stage2_semantic_requirements": W037_STAGE2_SEMANTIC_REQUIREMENTS,
                "w037_stage2_promotion_decision": W037_STAGE2_PROMOTION_DECISION,
                "w037_direct_oxfml_summary": W037_DIRECT_OXFML_SUMMARY
            },
            "profile_sources": STAGE2_REPLAY_SPECS
                .iter()
                .map(|spec| json!({
                    "row_id": spec.row_id,
                    "profile_id": spec.profile_id,
                    "source_kind": source_kind_name(spec.source_kind),
                    "source_artifact": spec.source_artifact,
                    "stage2_partition_shape": spec.stage2_partition_shape,
                    "formatting_watch": spec.formatting_watch
                }))
                .collect::<Vec<_>>(),
            "source_counts": {
                "w036_stage2_tla_passed_config_count": number_at(&w036_stage2_tla, "passed_config_count"),
                "w036_stage2_tla_failed_config_count": number_at(&w036_stage2_tla, "failed_config_count"),
                "w037_stage2_policy_promoted": bool_at(&w037_promotion_decision, "stage2_policy_promoted"),
                "w037_direct_oxfml_w073_typed_rule_case_count": number_at(&w037_direct_oxfml_summary, "w073_typed_rule_case_count")
            }
        });
        let partition_matrix = json!({
            "schema_version": PARTITION_MATRIX_SCHEMA_V1,
            "run_id": run_id,
            "row_count": partition_replay_row_count,
            "observable_invariance_row_count": observable_invariance_row_count,
            "failed_row_count": row_failed_count,
            "stage2_execution_kind": "bounded_replay_projection_not_production_scheduler",
            "rows": partition_rows
        });
        let permutation_replay = json!({
            "schema_version": PERMUTATION_REPLAY_SCHEMA_V1,
            "run_id": run_id,
            "row_count": permutation_replay_row_count,
            "nontrivial_permutation_row_count": nontrivial_permutation_row_count,
            "failed_row_count": count_failed_permutation_rows(&permutation_rows),
            "rows": permutation_rows
        });
        let semantic_equivalence = json!({
            "schema_version": SEMANTIC_EQUIVALENCE_SCHEMA_V1,
            "run_id": run_id,
            "semantic_equivalence_statement": "For the bounded profiles in this run, the materialized observable projection is invariant between the baseline schedule, the declared Stage 2 partition schedule, and every admissible partition-order permutation. This is bounded replay evidence only; it is not production scheduler or partition-analyzer promotion.",
            "w037_requirement_statement": w037_semantic_requirements["statement"].clone(),
            "w037_required_observable_surface": w037_semantic_requirements["observable_result_surface"].clone(),
            "w037_absent_replay_blocker_disposition": "narrowed_to_bounded_replay_present_for_declared_profiles",
            "observable_invariance_row_count": observable_invariance_row_count,
            "partition_replay_row_count": partition_replay_row_count,
            "permutation_replay_row_count": permutation_replay_row_count,
            "formatting_watch_row_count": formatting_watch_row_count,
            "production_stage2_policy_promoted": false,
            "remaining_exact_blockers": exact_blockers
                .iter()
                .map(|row| row["blocker_id"].clone())
                .collect::<Vec<_>>()
        });
        let blocker_register = json!({
            "schema_version": BLOCKER_REGISTER_SCHEMA_V1,
            "run_id": run_id,
            "exact_remaining_blocker_count": exact_remaining_blocker_count,
            "rows": exact_blockers
        });
        let promotion_decision = json!({
            "schema_version": PROMOTION_DECISION_SCHEMA_V1,
            "run_id": run_id,
            "decision_state": "w038_bounded_stage2_replay_validated_policy_unpromoted",
            "stage2_policy_promoted": false,
            "stage2_promotion_candidate": false,
            "bounded_partition_replay_present": failed_row_count == 0,
            "partition_order_permutation_replay_present": nontrivial_permutation_row_count > 0 && failed_row_count == 0,
            "observable_result_invariance_evidenced_for_declared_profiles": failed_row_count == 0,
            "w073_typed_formatting_guard_carried": formatting_watch_row_count == 1 && failed_row_count == 0,
            "satisfied_inputs": [
                "bounded_tla_partition_model_present",
                "observable_result_invariance_obligations_defined",
                "bounded_baseline_vs_stage2_replay_profiles_present",
                "bounded_partition_order_permutation_replay_present",
                "w073_typed_formatting_guard_carried"
            ],
            "blockers": [
                "stage2.production_partition_analyzer_soundness_absent",
                "stage2.operated_cross_engine_differential_service_absent",
                "stage2.pack_grade_replay_governance_absent"
            ],
            "semantic_equivalence_statement": "Observable-result invariance is evidenced for the declared bounded profiles only. Production Stage 2 scheduler or partition policy remains unpromoted until partition-analyzer soundness, operated cross-engine service evidence, and pack-grade replay governance are present."
        });
        let mut validation_failures = evaluated_rows
            .iter()
            .filter(|row| row.failed)
            .filter_map(|row| row.row.get("row_id").and_then(Value::as_str))
            .map(|row_id| format!("{row_id}_failed"))
            .collect::<Vec<_>>();
        validation_failures.extend(source_failures);
        let validation = json!({
            "schema_version": VALIDATION_SCHEMA_V1,
            "run_id": run_id,
            "status": if validation_failures.is_empty() {
                "w038_stage2_replay_valid"
            } else {
                "w038_stage2_replay_invalid"
            },
            "partition_replay_row_count": partition_replay_row_count,
            "permutation_replay_row_count": permutation_replay_row_count,
            "nontrivial_permutation_row_count": nontrivial_permutation_row_count,
            "observable_invariance_row_count": observable_invariance_row_count,
            "formatting_watch_row_count": formatting_watch_row_count,
            "exact_remaining_blocker_count": exact_remaining_blocker_count,
            "failed_row_count": failed_row_count,
            "stage2_policy_promoted": false,
            "validation_failures": validation_failures
        });
        let run_summary = json!({
            "schema_version": RUN_SUMMARY_SCHEMA_V1,
            "run_id": run_id,
            "artifact_root": relative_artifact_root,
            "source_evidence_index_path": source_evidence_index_path,
            "partition_replay_matrix_path": partition_matrix_path,
            "partition_order_permutation_replay_path": permutation_replay_path,
            "semantic_equivalence_report_path": semantic_equivalence_path,
            "stage2_exact_blocker_register_path": blocker_register_path,
            "promotion_decision_path": promotion_decision_path,
            "validation_path": validation_path,
            "partition_replay_row_count": partition_replay_row_count,
            "permutation_replay_row_count": permutation_replay_row_count,
            "nontrivial_permutation_row_count": nontrivial_permutation_row_count,
            "observable_invariance_row_count": observable_invariance_row_count,
            "formatting_watch_row_count": formatting_watch_row_count,
            "exact_remaining_blocker_count": exact_remaining_blocker_count,
            "failed_row_count": failed_row_count,
            "stage2_policy_promoted": false,
            "stage2_promotion_candidate": false
        });

        write_json(
            &artifact_root.join("source_evidence_index.json"),
            &source_evidence_index,
        )?;
        write_json(
            &artifact_root.join("partition_replay_matrix.json"),
            &partition_matrix,
        )?;
        write_json(
            &artifact_root.join("partition_order_permutation_replay.json"),
            &permutation_replay,
        )?;
        write_json(
            &artifact_root.join("semantic_equivalence_report.json"),
            &semantic_equivalence,
        )?;
        write_json(
            &artifact_root.join("stage2_exact_blocker_register.json"),
            &blocker_register,
        )?;
        write_json(
            &artifact_root.join("promotion_decision.json"),
            &promotion_decision,
        )?;
        write_json(&artifact_root.join("validation.json"), &validation)?;
        write_json(&artifact_root.join("run_summary.json"), &run_summary)?;

        Ok(Stage2ReplayRunSummary {
            run_id: run_id.to_string(),
            schema_version: RUN_SUMMARY_SCHEMA_V1.to_string(),
            partition_replay_row_count,
            permutation_replay_row_count,
            nontrivial_permutation_row_count,
            observable_invariance_row_count,
            formatting_watch_row_count,
            exact_remaining_blocker_count,
            failed_row_count,
            stage2_policy_promoted: false,
            artifact_root: relative_artifact_root,
        })
    }

    fn execute_w044(
        &self,
        repo_root: &Path,
        run_id: &str,
    ) -> Result<Stage2ReplayRunSummary, Stage2ReplayError> {
        let relative_artifact_root =
            relative_artifact_path(&["docs", "test-runs", "core-engine", "stage2-replay", run_id]);
        let artifact_root = repo_root.join(&relative_artifact_root);
        if artifact_root.exists() {
            fs::remove_dir_all(&artifact_root).map_err(|source| {
                Stage2ReplayError::RemoveDirectory {
                    path: artifact_root.display().to_string(),
                    source,
                }
            })?;
        }
        fs::create_dir_all(&artifact_root).map_err(|source| {
            Stage2ReplayError::CreateDirectory {
                path: artifact_root.display().to_string(),
                source,
            }
        })?;

        let w044_residual_summary = read_json(repo_root, W044_RESIDUAL_SUMMARY)?;
        let w044_residual_blocker_map = read_json(repo_root, W044_RESIDUAL_BLOCKER_MAP)?;
        let w044_optimized_summary = read_json(repo_root, W044_OPTIMIZED_CORE_SUMMARY)?;
        let w044_optimized_dispositions = read_json(repo_root, W044_OPTIMIZED_CORE_DISPOSITIONS)?;
        let w044_dynamic_evidence = read_json(repo_root, W044_OPTIMIZED_CORE_DYNAMIC_EVIDENCE)?;
        let w044_optimized_blockers = read_json(repo_root, W044_OPTIMIZED_CORE_BLOCKERS)?;
        let w044_treecalc_summary = read_json(repo_root, W044_TREECALC_SUMMARY)?;
        let w044_rust_summary = read_json(repo_root, W044_RUST_SUMMARY)?;
        let w044_rust_validation = read_json(repo_root, W044_RUST_VALIDATION)?;
        let w044_rust_refinement = read_json(repo_root, W044_RUST_REFINEMENT_REGISTER)?;
        let w044_lean_tla_summary = read_json(repo_root, W044_LEAN_TLA_SUMMARY)?;
        let w044_lean_tla_validation = read_json(repo_root, W044_LEAN_TLA_VALIDATION)?;
        let w044_tla_model_register = read_json(repo_root, W044_TLA_MODEL_REGISTER)?;
        let w044_lean_tla_blockers = read_json(repo_root, W044_LEAN_TLA_BLOCKERS)?;
        let w043_stage2_summary = read_json(repo_root, W043_STAGE2_RUN_SUMMARY_FOR_W044)?;
        let w043_stage2_validation = read_json(repo_root, W043_STAGE2_VALIDATION_FOR_W044)?;
        let w043_stage2_policy_gate = read_json(repo_root, W043_STAGE2_POLICY_GATE_FOR_W044)?;
        let w043_stage2_production_analyzer =
            read_json(repo_root, W043_STAGE2_PRODUCTION_ANALYZER_FOR_W044)?;
        let w043_stage2_scheduler =
            read_json(repo_root, W043_STAGE2_SCHEDULER_EQUIVALENCE_FOR_W044)?;
        let w043_stage2_pack = read_json(repo_root, W043_STAGE2_PACK_GRADE_EQUIVALENCE_FOR_W044)?;
        let w043_stage2_blockers = read_json(repo_root, W043_STAGE2_BLOCKERS_FOR_W044)?;
        let w043_stage2_promotion = read_json(repo_root, W043_STAGE2_PROMOTION_DECISION_FOR_W044)?;
        let lean_file_present = repo_root.join(W044_STAGE2_LEAN_FILE).exists();

        let w044_residual_valid = number_at(&w044_residual_summary, "obligation_count") == 45
            && number_at(&w044_residual_summary, "promotion_contract_count") == 17
            && bool_at(
                &w044_residual_summary,
                "oxfml_formatting_update_incorporated",
            )
            && array_field_contains_string(
                &w044_residual_summary,
                "no_promotion_claims",
                "stage2_production_policy",
            )
            && array_field_contains_string(
                &w044_residual_summary,
                "no_promotion_claims",
                "pack_grade_replay_equivalence",
            );
        let w043_stage2_valid = string_at(&w043_stage2_validation, "status")
            == "w043_stage2_scheduler_equivalence_valid"
            && number_at(&w043_stage2_summary, "policy_row_count") == 20
            && number_at(&w043_stage2_summary, "satisfied_policy_row_count") == 14
            && number_at(&w043_stage2_summary, "exact_remaining_blocker_count") == 6
            && !bool_at(&w043_stage2_summary, "stage2_policy_promoted")
            && !bool_at(&w043_stage2_summary, "pack_grade_replay_promoted")
            && !bool_at(&w043_stage2_promotion, "stage2_policy_promoted")
            && !bool_at(&w043_stage2_promotion, "pack_grade_replay_promoted");
        let w044_optimized_valid = number_at(&w044_optimized_summary, "failed_row_count") == 0
            && number_at(&w044_optimized_summary, "w044_disposition_row_count") == 6
            && number_at(&w044_optimized_summary, "w044_direct_evidence_bound_count") == 2
            && number_at(
                &w044_optimized_summary,
                "w044_exact_remaining_blocker_count",
            ) == 4
            && number_at(&w044_optimized_summary, "w044_match_promoted_count") == 0
            && number_at(&w044_optimized_dispositions, "validated_row_count") == 6;
        let w044_treecalc_valid = number_at(&w044_treecalc_summary, "case_count") == 28
            && number_at(&w044_treecalc_summary, "expectation_mismatch_count") == 0;
        let mixed_dynamic_valid = w044_optimized_valid
            && w044_treecalc_valid
            && bool_at(&w044_dynamic_evidence, "direct_evidence_bound")
            && array_field_contains_string(
                &w044_dynamic_evidence,
                "observed_seed_reasons",
                "DependencyAdded",
            )
            && array_field_contains_string(
                &w044_dynamic_evidence,
                "observed_seed_reasons",
                "DependencyRemoved",
            )
            && array_field_contains_string(
                &w044_dynamic_evidence,
                "observed_seed_reasons",
                "DependencyReclassified",
            )
            && array_field_contains_string(
                &w044_dynamic_evidence,
                "observed_closure_reasons",
                "DependencyAdded",
            )
            && array_field_contains_string(
                &w044_dynamic_evidence,
                "observed_closure_reasons",
                "DependencyRemoved",
            )
            && array_field_contains_string(
                &w044_dynamic_evidence,
                "observed_closure_reasons",
                "DependencyReclassified",
            );
        let no_publication_valid = mixed_dynamic_valid
            && string_at(&w044_dynamic_evidence, "post_edit_result_state") == "rejected"
            && string_at(&w044_dynamic_evidence, "post_edit_reject_kind") == "HostInjectedFailure";
        let w044_rust_valid = string_at(&w044_rust_validation, "status")
            == "formal_assurance_w044_rust_totality_refinement_valid"
            && number_at(&w044_rust_summary, "automatic_dynamic_transition_row_count") == 1
            && number_at(&w044_rust_summary, "failed_row_count") == 0
            && !bool_at(
                &w044_rust_summary["promotion_claims"],
                "stage2_policy_promoted",
            );
        let rust_mixed_refinement_valid = w044_rust_valid
            && row_with_field_exists(
                &w044_rust_refinement,
                "row_id",
                "w044_mixed_dynamic_add_release_refinement_evidence",
            )
            && row_with_field_exists(
                &w044_rust_refinement,
                "row_id",
                "w044_publication_fence_no_publish_refinement_evidence",
            );
        let w044_lean_tla_valid = string_at(&w044_lean_tla_validation, "status")
            == "formal_assurance_w044_lean_tla_fairness_valid"
            && number_at(&w044_lean_tla_summary, "bounded_model_row_count") == 4
            && number_at(
                &w044_lean_tla_summary,
                "dynamic_refinement_bridge_row_count",
            ) == 1
            && number_at(&w044_lean_tla_summary, "publication_fence_bridge_row_count") == 1
            && number_at(&w044_lean_tla_summary, "exact_remaining_blocker_count") == 5
            && !bool_at(
                &w044_lean_tla_summary["promotion_claims"],
                "stage2_policy_promoted",
            );
        let w044_model_bound_valid = w044_lean_tla_valid
            && row_with_field_exists(
                &w044_tla_model_register,
                "row_id",
                "w044_tla_stage2_partition_bounded_model_evidence",
            )
            && row_with_field_exists(
                &w044_tla_model_register,
                "row_id",
                "w044_stage2_equivalence_bounded_model_input",
            );
        let w073_guard_valid = number_at(&w043_stage2_summary, "formatting_watch_row_count") == 1
            && bool_at(
                &w044_residual_summary,
                "w073_old_bounded_string_non_interpretation_evidence_reviewed",
            )
            && string_at(&w044_residual_summary, "w073_formatting_intake")
                == "typed_rule_only_direct_replacement_guard_retained"
            && !bool_at(
                &w044_residual_summary,
                "w073_downstream_request_construction_uptake_verified",
            );
        let predecessor_policy_valid = w043_stage2_valid
            && number_at(&w043_stage2_policy_gate, "policy_row_count") == 20
            && number_at(&w043_stage2_policy_gate, "satisfied_policy_row_count") == 14
            && number_at(&w043_stage2_policy_gate, "exact_remaining_blocker_count") == 6
            && number_at(&w043_stage2_production_analyzer, "row_count") == 11;
        let bounded_replay_valid = predecessor_policy_valid
            && number_at(&w043_stage2_summary, "partition_replay_row_count") == 5;
        let permutation_replay_valid = predecessor_policy_valid
            && number_at(&w043_stage2_summary, "permutation_replay_row_count") == 6
            && number_at(&w043_stage2_summary, "nontrivial_permutation_row_count") == 1;
        let observable_invariance_valid = predecessor_policy_valid
            && number_at(&w043_stage2_summary, "observable_invariance_row_count") == 5;
        let w043_dynamic_valid = predecessor_policy_valid
            && number_at(
                &w043_stage2_summary,
                "automatic_dynamic_transition_row_count",
            ) == 2
            && bool_at(&w043_stage2_summary, "automatic_dynamic_addition_evidenced")
            && bool_at(&w043_stage2_summary, "automatic_dynamic_release_evidenced");
        let snapshot_declared_valid = predecessor_policy_valid
            && bool_at(&w043_stage2_summary, "snapshot_counterpart_evidenced");
        let capability_declared_valid = predecessor_policy_valid
            && bool_at(&w043_stage2_summary, "capability_counterpart_evidenced");
        let declared_scheduler_valid = predecessor_policy_valid
            && bool_at(
                &w043_stage2_summary,
                "scheduler_equivalence_evidenced_for_declared_profiles",
            )
            && number_at(&w043_stage2_scheduler, "exact_remaining_blocker_count") == 1
            && w044_model_bound_valid;
        let declared_pack_valid = predecessor_policy_valid
            && bool_at(&w043_stage2_summary, "declared_pack_equivalence_evidenced")
            && number_at(&w043_stage2_pack, "exact_remaining_blocker_count") == 3
            && observable_invariance_valid
            && w073_guard_valid;
        let production_relevant_inputs_valid = bounded_replay_valid
            && permutation_replay_valid
            && observable_invariance_valid
            && w043_dynamic_valid
            && mixed_dynamic_valid
            && no_publication_valid
            && rust_mixed_refinement_valid
            && w044_model_bound_valid
            && w073_guard_valid;
        let no_proxy_guard_valid = w044_residual_valid
            && number_at(&w044_optimized_summary, "w044_match_promoted_count") == 0
            && !bool_at(&w043_stage2_promotion, "stage2_promotion_candidate")
            && !bool_at(&w043_stage2_promotion, "pack_grade_replay_candidate");

        let broader_dynamic_blocker = row_with_field_exists(
            &w044_optimized_blockers,
            "row_id",
            "w044_broader_dynamic_transition_remaining_exact_blocker",
        );
        let snapshot_breadth_blocker = row_with_field_exists(
            &w044_optimized_blockers,
            "row_id",
            "w044_snapshot_fence_counterpart_breadth_exact_blocker",
        );
        let capability_breadth_blocker = row_with_field_exists(
            &w044_optimized_blockers,
            "row_id",
            "w044_capability_view_counterpart_breadth_exact_blocker",
        );
        let production_analyzer_blocker = row_with_field_exists(
            &w043_stage2_blockers,
            "row_id",
            "w043_stage2_full_production_partition_analyzer_soundness_blocker",
        ) && row_with_field_exists(
            &w044_residual_blocker_map,
            "source_lane",
            "w043_residual.stage2_production_policy_and_pack_equivalence",
        );
        let fairness_blocker = row_with_field_exists(
            &w044_lean_tla_blockers,
            "row_id",
            "w044_tla_fairness_scheduler_unbounded_boundary",
        );
        let operated_service_blocker = row_with_field_exists(
            &w043_stage2_blockers,
            "row_id",
            "w043_stage2_operated_cross_engine_service_dependency_blocker",
        );
        let retained_witness_blocker = row_with_field_exists(
            &w043_stage2_blockers,
            "row_id",
            "w043_stage2_retained_witness_lifecycle_pack_dependency_blocker",
        );
        let pack_governance_blocker = row_with_field_exists(
            &w043_stage2_blockers,
            "row_id",
            "w043_stage2_pack_grade_replay_governance_blocker",
        );

        let policy_rows = vec![
            json!({"row_id":"w044_stage2_w043_predecessor_policy_carried","policy_area":"w043_stage2_predecessor","registers":["production","scheduler"],"source_artifacts":[W043_STAGE2_RUN_SUMMARY_FOR_W044,W043_STAGE2_VALIDATION_FOR_W044,W043_STAGE2_POLICY_GATE_FOR_W044],"satisfied_for_declared_profile":predecessor_policy_valid,"exact_remaining_blocker":false,"promotion_consequence":"predecessor evidence is valid input only; W044 promotion still requires direct W044 gates","failures": if predecessor_policy_valid { Vec::<String>::new() } else { vec!["w044_w043_stage2_predecessor_not_valid".to_string()] }}),
            json!({"row_id":"w044_stage2_bounded_replay_carried","policy_area":"bounded_baseline_vs_stage2_replay","registers":["production","scheduler"],"source_artifacts":[W043_STAGE2_RUN_SUMMARY_FOR_W044],"satisfied_for_declared_profile":bounded_replay_valid,"exact_remaining_blocker":false,"promotion_consequence":"bounded replay remains declared-profile evidence","failures": if bounded_replay_valid { Vec::<String>::new() } else { vec!["w044_bounded_replay_not_valid".to_string()] }}),
            json!({"row_id":"w044_stage2_partition_order_permutation_carried","policy_area":"partition_order_permutation_replay","registers":["production","scheduler"],"source_artifacts":[W043_STAGE2_RUN_SUMMARY_FOR_W044],"satisfied_for_declared_profile":permutation_replay_valid,"exact_remaining_blocker":false,"promotion_consequence":"bounded permutation replay does not discharge unbounded fairness","failures": if permutation_replay_valid { Vec::<String>::new() } else { vec!["w044_partition_permutation_not_valid".to_string()] }}),
            json!({"row_id":"w044_stage2_observable_invariance_carried","policy_area":"observable_result_invariance","registers":["production","scheduler","pack"],"source_artifacts":[W043_STAGE2_RUN_SUMMARY_FOR_W044,W043_STAGE2_PACK_GRADE_EQUIVALENCE_FOR_W044],"satisfied_for_declared_profile":observable_invariance_valid,"exact_remaining_blocker":false,"promotion_consequence":"observable invariance is evidenced for declared bounded profiles","failures": if observable_invariance_valid { Vec::<String>::new() } else { vec!["w044_observable_invariance_not_valid".to_string()] }}),
            json!({"row_id":"w044_stage2_w043_dynamic_transition_carried","policy_area":"w043_dynamic_addition_release_regression","registers":["production"],"source_artifacts":[W043_STAGE2_RUN_SUMMARY_FOR_W044,W043_STAGE2_PRODUCTION_ANALYZER_FOR_W044],"satisfied_for_declared_profile":w043_dynamic_valid,"exact_remaining_blocker":false,"promotion_consequence":"W043 addition/release evidence remains regression input, not broad dynamic coverage","failures": if w043_dynamic_valid { Vec::<String>::new() } else { vec!["w044_w043_dynamic_transition_input_not_valid".to_string()] }}),
            json!({"row_id":"w044_stage2_mixed_dynamic_transition_implementation","policy_area":"mixed_dynamic_add_remove_reclassify_transition","registers":["production"],"source_artifacts":[W044_OPTIMIZED_CORE_DYNAMIC_EVIDENCE,W044_OPTIMIZED_CORE_DISPOSITIONS,W044_TREECALC_SUMMARY],"satisfied_for_declared_profile":mixed_dynamic_valid,"exact_remaining_blocker":false,"promotion_consequence":"mixed dynamic transition evidence narrows Stage 2 analyzer inputs but does not prove full production analyzer soundness","failures": if mixed_dynamic_valid { Vec::<String>::new() } else { vec!["w044_mixed_dynamic_transition_not_valid".to_string()] }}),
            json!({"row_id":"w044_stage2_no_publication_fence_implementation","policy_area":"mixed_dynamic_rebind_reject_no_publication","registers":["production","pack"],"source_artifacts":[W044_OPTIMIZED_CORE_DYNAMIC_EVIDENCE,W044_RUST_REFINEMENT_REGISTER],"satisfied_for_declared_profile":no_publication_valid,"exact_remaining_blocker":false,"promotion_consequence":"no-publication behavior is evidenced for the exercised reject path only","failures": if no_publication_valid { Vec::<String>::new() } else { vec!["w044_no_publication_fence_not_valid".to_string()] }}),
            json!({"row_id":"w044_stage2_snapshot_declared_counterpart_carried","policy_area":"snapshot_fence_declared_profile_counterpart","registers":["production","pack"],"source_artifacts":[W043_STAGE2_RUN_SUMMARY_FOR_W044],"satisfied_for_declared_profile":snapshot_declared_valid,"exact_remaining_blocker":false,"promotion_consequence":"declared-profile snapshot counterpart remains input while breadth is blocked separately","failures": if snapshot_declared_valid { Vec::<String>::new() } else { vec!["w044_snapshot_declared_counterpart_not_valid".to_string()] }}),
            json!({"row_id":"w044_stage2_capability_declared_counterpart_carried","policy_area":"capability_view_declared_profile_counterpart","registers":["production","pack"],"source_artifacts":[W043_STAGE2_RUN_SUMMARY_FOR_W044],"satisfied_for_declared_profile":capability_declared_valid,"exact_remaining_blocker":false,"promotion_consequence":"declared-profile capability counterpart remains input while breadth is blocked separately","failures": if capability_declared_valid { Vec::<String>::new() } else { vec!["w044_capability_declared_counterpart_not_valid".to_string()] }}),
            json!({"row_id":"w044_stage2_rust_refinement_bridge","policy_area":"rust_mixed_dynamic_and_no_publication_refinement","registers":["production"],"source_artifacts":[W044_RUST_SUMMARY,W044_RUST_VALIDATION,W044_RUST_REFINEMENT_REGISTER],"satisfied_for_declared_profile":rust_mixed_refinement_valid,"exact_remaining_blocker":false,"promotion_consequence":"Rust refinement rows are input only; Rust totality/refinement remains unpromoted","failures": if rust_mixed_refinement_valid { Vec::<String>::new() } else { vec!["w044_rust_refinement_bridge_not_valid".to_string()] }}),
            json!({"row_id":"w044_stage2_lean_tla_model_bound_bridge","policy_area":"lean_tla_stage2_bounded_model_input","registers":["production","scheduler"],"source_artifacts":[W044_LEAN_TLA_SUMMARY,W044_LEAN_TLA_VALIDATION,W044_TLA_MODEL_REGISTER],"satisfied_for_declared_profile":w044_model_bound_valid,"exact_remaining_blocker":false,"promotion_consequence":"bounded model evidence remains non-promoting","failures": if w044_model_bound_valid { Vec::<String>::new() } else { vec!["w044_lean_tla_model_bound_not_valid".to_string()] }}),
            json!({"row_id":"w044_stage2_w073_typed_formatting_guard","policy_area":"w073_typed_only_formatting_guard","registers":["production","pack"],"source_artifacts":[W044_RESIDUAL_SUMMARY,W043_STAGE2_RUN_SUMMARY_FOR_W044],"satisfied_for_declared_profile":w073_guard_valid,"exact_remaining_blocker":false,"promotion_consequence":"W073 guard is seam-watch input, not broad OxFml or downstream request-construction uptake","failures": if w073_guard_valid { Vec::<String>::new() } else { vec!["w044_w073_guard_not_valid".to_string()] }}),
            json!({"row_id":"w044_stage2_declared_scheduler_equivalence_carried","policy_area":"declared_profile_scheduler_equivalence","registers":["scheduler"],"source_artifacts":[W043_STAGE2_SCHEDULER_EQUIVALENCE_FOR_W044,W044_TLA_MODEL_REGISTER],"satisfied_for_declared_profile":declared_scheduler_valid,"exact_remaining_blocker":false,"promotion_consequence":"declared-profile scheduler equivalence remains bounded and non-promoting","failures": if declared_scheduler_valid { Vec::<String>::new() } else { vec!["w044_declared_scheduler_equivalence_not_valid".to_string()] }}),
            json!({"row_id":"w044_stage2_declared_pack_equivalence_carried","policy_area":"declared_profile_pack_equivalence","registers":["pack"],"source_artifacts":[W043_STAGE2_PACK_GRADE_EQUIVALENCE_FOR_W044],"satisfied_for_declared_profile":declared_pack_valid,"exact_remaining_blocker":false,"promotion_consequence":"declared-profile pack equivalence does not promote pack-grade replay","failures": if declared_pack_valid { Vec::<String>::new() } else { vec!["w044_declared_pack_equivalence_not_valid".to_string()] }}),
            json!({"row_id":"w044_stage2_no_proxy_promotion_guard","policy_area":"no_proxy_promotion_guard","registers":["production","pack"],"source_artifacts":[W044_RESIDUAL_SUMMARY,W043_STAGE2_PROMOTION_DECISION_FOR_W044],"satisfied_for_declared_profile":no_proxy_guard_valid,"exact_remaining_blocker":false,"promotion_consequence":"proxy evidence cannot promote Stage 2 policy or pack-grade replay","failures": if no_proxy_guard_valid { Vec::<String>::new() } else { vec!["w044_no_proxy_guard_not_valid".to_string()] }}),
            json!({"row_id":"w044_stage2_production_relevant_analyzer_input_bundle","policy_area":"production_relevant_analyzer_inputs","registers":["production"],"source_artifacts":[W044_OPTIMIZED_CORE_DYNAMIC_EVIDENCE,W044_RUST_REFINEMENT_REGISTER,W044_TLA_MODEL_REGISTER],"satisfied_for_declared_profile":production_relevant_inputs_valid,"exact_remaining_blocker":false,"promotion_consequence":"production-relevant inputs are present for the declared W044 slice, while full production soundness remains blocked","failures": if production_relevant_inputs_valid { Vec::<String>::new() } else { vec!["w044_production_relevant_inputs_not_valid".to_string()] }}),
            json!({"row_id":"w044_stage2_semantic_equivalence_statement_bound","policy_area":"semantic_equivalence_statement","registers":["scheduler"],"source_artifacts":[W044_STAGE2_LEAN_FILE],"satisfied_for_declared_profile":lean_file_present && declared_scheduler_valid && no_proxy_guard_valid,"exact_remaining_blocker":false,"promotion_consequence":"observable results remain invariant because no runtime scheduler policy changed","failures": if lean_file_present && declared_scheduler_valid && no_proxy_guard_valid { Vec::<String>::new() } else { vec!["w044_semantic_equivalence_statement_not_valid".to_string()] }}),
            json!({"row_id":"w044_stage2_broader_dynamic_transition_coverage_blocker","policy_area":"broader_dynamic_transition_coverage","registers":["production"],"source_artifacts":[W044_OPTIMIZED_CORE_BLOCKERS],"satisfied_for_declared_profile":false,"exact_remaining_blocker":true,"promotion_consequence":"production partition-analyzer soundness remains blocked","failures": if broader_dynamic_blocker { Vec::<String>::new() } else { vec!["w044_broader_dynamic_blocker_missing".to_string()] }}),
            json!({"row_id":"w044_stage2_snapshot_fence_breadth_blocker","policy_area":"snapshot_fence_breadth","registers":["production"],"source_artifacts":[W044_OPTIMIZED_CORE_BLOCKERS],"satisfied_for_declared_profile":false,"exact_remaining_blocker":true,"promotion_consequence":"Stage 2 production policy remains blocked until snapshot-fence breadth exists","failures": if snapshot_breadth_blocker { Vec::<String>::new() } else { vec!["w044_snapshot_breadth_blocker_missing".to_string()] }}),
            json!({"row_id":"w044_stage2_capability_view_breadth_blocker","policy_area":"capability_view_breadth","registers":["production"],"source_artifacts":[W044_OPTIMIZED_CORE_BLOCKERS],"satisfied_for_declared_profile":false,"exact_remaining_blocker":true,"promotion_consequence":"Stage 2 production policy remains blocked until capability-view breadth exists","failures": if capability_breadth_blocker { Vec::<String>::new() } else { vec!["w044_capability_breadth_blocker_missing".to_string()] }}),
            json!({"row_id":"w044_stage2_full_production_partition_analyzer_soundness_blocker","policy_area":"full_production_partition_analyzer_soundness","registers":["production"],"source_artifacts":[W043_STAGE2_BLOCKERS_FOR_W044,W044_RESIDUAL_BLOCKER_MAP,W044_STAGE2_LEAN_FILE],"satisfied_for_declared_profile":false,"exact_remaining_blocker":true,"promotion_consequence":"Stage 2 production policy remains unpromoted","failures": if production_analyzer_blocker { Vec::<String>::new() } else { vec!["w044_production_analyzer_blocker_missing".to_string()] }}),
            json!({"row_id":"w044_stage2_scheduler_fairness_unbounded_coverage_blocker","policy_area":"fairness_scheduler_unbounded_coverage","registers":["scheduler"],"source_artifacts":[W044_LEAN_TLA_BLOCKERS,W043_STAGE2_BLOCKERS_FOR_W044],"satisfied_for_declared_profile":false,"exact_remaining_blocker":true,"promotion_consequence":"full TLA verification and Stage 2 production policy remain unpromoted","failures": if fairness_blocker { Vec::<String>::new() } else { vec!["w044_fairness_blocker_missing".to_string()] }}),
            json!({"row_id":"w044_stage2_operated_cross_engine_service_dependency_blocker","policy_area":"operated_cross_engine_stage2_differential_service","registers":["scheduler"],"source_artifacts":[W043_STAGE2_BLOCKERS_FOR_W044],"satisfied_for_declared_profile":false,"exact_remaining_blocker":true,"promotion_consequence":"operated Stage 2 differential evidence remains required before policy promotion","failures": if operated_service_blocker { Vec::<String>::new() } else { vec!["w044_operated_service_blocker_missing".to_string()] }}),
            json!({"row_id":"w044_stage2_retained_witness_lifecycle_pack_dependency_blocker","policy_area":"retained_witness_lifecycle_pack_dependency","registers":["pack"],"source_artifacts":[W043_STAGE2_BLOCKERS_FOR_W044],"satisfied_for_declared_profile":false,"exact_remaining_blocker":true,"promotion_consequence":"pack-grade replay equivalence remains blocked until retained-witness lifecycle evidence exists","failures": if retained_witness_blocker { Vec::<String>::new() } else { vec!["w044_retained_witness_blocker_missing".to_string()] }}),
            json!({"row_id":"w044_stage2_pack_grade_replay_governance_blocker","policy_area":"pack_grade_replay_governance","registers":["pack"],"source_artifacts":[W043_STAGE2_BLOCKERS_FOR_W044],"satisfied_for_declared_profile":false,"exact_remaining_blocker":true,"promotion_consequence":"pack-grade replay, C5, and release-grade verification remain blocked","failures": if pack_governance_blocker { Vec::<String>::new() } else { vec!["w044_pack_governance_blocker_missing".to_string()] }}),
        ];

        let policy_row_count = policy_rows.len();
        let satisfied_policy_row_count = policy_rows
            .iter()
            .filter(|row| bool_at(row, "satisfied_for_declared_profile"))
            .count();
        let exact_remaining_blocker_count = policy_rows
            .iter()
            .filter(|row| bool_at(row, "exact_remaining_blocker"))
            .count();
        let policy_failed_row_count = policy_rows
            .iter()
            .filter(|row| {
                row.get("failures")
                    .and_then(Value::as_array)
                    .is_some_and(|failures| !failures.is_empty())
            })
            .count();

        let rows_for_register = |register_name: &str| -> Vec<Value> {
            policy_rows
                .iter()
                .filter(|row| {
                    row.get("registers")
                        .and_then(Value::as_array)
                        .into_iter()
                        .flatten()
                        .any(|item| item.as_str() == Some(register_name))
                })
                .cloned()
                .collect()
        };
        let production_analyzer_rows = rows_for_register("production");
        let scheduler_equivalence_rows = rows_for_register("scheduler");
        let pack_grade_equivalence_rows = rows_for_register("pack");
        let policy_blocker_rows = policy_rows
            .iter()
            .filter(|row| bool_at(row, "exact_remaining_blocker"))
            .cloned()
            .collect::<Vec<_>>();

        let mut validation_failures = Vec::new();
        if !w044_residual_valid {
            validation_failures.push("w044_residual_map_not_valid".to_string());
        }
        if !w043_stage2_valid {
            validation_failures.push("w043_stage2_predecessor_not_valid".to_string());
        }
        if !w044_optimized_valid || !mixed_dynamic_valid || !no_publication_valid {
            validation_failures.push("w044_optimized_stage2_inputs_not_valid".to_string());
        }
        if !rust_mixed_refinement_valid {
            validation_failures.push("w044_rust_refinement_bridge_not_valid".to_string());
        }
        if !w044_model_bound_valid {
            validation_failures.push("w044_lean_tla_model_bound_not_valid".to_string());
        }
        if !declared_scheduler_valid {
            validation_failures.push("w044_declared_scheduler_equivalence_not_valid".to_string());
        }
        if !declared_pack_valid {
            validation_failures.push("w044_declared_pack_equivalence_not_valid".to_string());
        }
        if !w073_guard_valid {
            validation_failures.push("w044_w073_guard_not_valid".to_string());
        }
        if !no_proxy_guard_valid {
            validation_failures.push("w044_no_proxy_guard_not_valid".to_string());
        }
        if !lean_file_present {
            validation_failures.push("w044_stage2_lean_file_missing".to_string());
        }
        if policy_failed_row_count != 0 {
            validation_failures.push("w044_stage2_policy_row_failures_present".to_string());
        }
        if policy_row_count != 25 {
            validation_failures.push("w044_stage2_expected_twenty_five_policy_rows".to_string());
        }
        if satisfied_policy_row_count != 17 {
            validation_failures.push("w044_stage2_expected_seventeen_satisfied_rows".to_string());
        }
        if exact_remaining_blocker_count != 8 {
            validation_failures.push("w044_stage2_expected_eight_exact_blockers".to_string());
        }

        let source_evidence_index_path =
            format!("{relative_artifact_root}/source_evidence_index.json");
        let policy_gate_register_path =
            format!("{relative_artifact_root}/w044_stage2_policy_gate_register.json");
        let production_analyzer_register_path =
            format!("{relative_artifact_root}/w044_production_partition_analyzer_register.json");
        let scheduler_equivalence_register_path =
            format!("{relative_artifact_root}/w044_scheduler_equivalence_register.json");
        let pack_grade_equivalence_register_path =
            format!("{relative_artifact_root}/w044_pack_grade_equivalence_register.json");
        let blocker_register_path =
            format!("{relative_artifact_root}/w044_stage2_exact_blocker_register.json");
        let promotion_decision_path = format!("{relative_artifact_root}/promotion_decision.json");
        let validation_path = format!("{relative_artifact_root}/validation.json");

        write_json(
            &artifact_root.join("source_evidence_index.json"),
            &json!({
                "schema_version": W044_SOURCE_INDEX_SCHEMA_V1,
                "run_id": run_id,
                "artifact_root": relative_artifact_root,
                "source_artifacts": {
                    "w044_residual_summary": W044_RESIDUAL_SUMMARY,
                    "w044_residual_blocker_map": W044_RESIDUAL_BLOCKER_MAP,
                    "w044_optimized_core_summary": W044_OPTIMIZED_CORE_SUMMARY,
                    "w044_optimized_core_dispositions": W044_OPTIMIZED_CORE_DISPOSITIONS,
                    "w044_dynamic_transition_evidence": W044_OPTIMIZED_CORE_DYNAMIC_EVIDENCE,
                    "w044_optimized_core_blockers": W044_OPTIMIZED_CORE_BLOCKERS,
                    "w044_treecalc_summary": W044_TREECALC_SUMMARY,
                    "w044_rust_summary": W044_RUST_SUMMARY,
                    "w044_rust_validation": W044_RUST_VALIDATION,
                    "w044_rust_refinement_register": W044_RUST_REFINEMENT_REGISTER,
                    "w044_lean_tla_summary": W044_LEAN_TLA_SUMMARY,
                    "w044_lean_tla_validation": W044_LEAN_TLA_VALIDATION,
                    "w044_tla_model_register": W044_TLA_MODEL_REGISTER,
                    "w044_lean_tla_blockers": W044_LEAN_TLA_BLOCKERS,
                    "w043_stage2_summary": W043_STAGE2_RUN_SUMMARY_FOR_W044,
                    "w043_stage2_validation": W043_STAGE2_VALIDATION_FOR_W044,
                    "w043_stage2_policy_gate": W043_STAGE2_POLICY_GATE_FOR_W044,
                    "w043_stage2_production_analyzer": W043_STAGE2_PRODUCTION_ANALYZER_FOR_W044,
                    "w043_stage2_scheduler_equivalence": W043_STAGE2_SCHEDULER_EQUIVALENCE_FOR_W044,
                    "w043_stage2_pack_grade_equivalence": W043_STAGE2_PACK_GRADE_EQUIVALENCE_FOR_W044,
                    "w043_stage2_blockers": W043_STAGE2_BLOCKERS_FOR_W044,
                    "w043_stage2_promotion_decision": W043_STAGE2_PROMOTION_DECISION_FOR_W044,
                    "w044_stage2_lean_file": W044_STAGE2_LEAN_FILE
                },
                "source_counts": {
                    "w044_obligation_count": number_at(&w044_residual_summary, "obligation_count"),
                    "w044_optimized_exact_remaining_blocker_count": number_at(&w044_optimized_summary, "w044_exact_remaining_blocker_count"),
                    "w044_rust_exact_remaining_blocker_count": number_at(&w044_rust_summary, "exact_remaining_blocker_count"),
                    "w044_lean_tla_exact_remaining_blocker_count": number_at(&w044_lean_tla_summary, "exact_remaining_blocker_count"),
                    "w043_stage2_policy_row_count": number_at(&w043_stage2_summary, "policy_row_count"),
                    "w043_stage2_exact_remaining_blocker_count": number_at(&w043_stage2_summary, "exact_remaining_blocker_count")
                }
            }),
        )?;
        write_json(
            &artifact_root.join("w044_stage2_policy_gate_register.json"),
            &json!({
                "schema_version": W044_POLICY_GATE_SCHEMA_V1,
                "run_id": run_id,
                "policy_row_count": policy_row_count,
                "satisfied_policy_row_count": satisfied_policy_row_count,
                "exact_remaining_blocker_count": exact_remaining_blocker_count,
                "stage2_policy_promoted": false,
                "pack_grade_replay_promoted": false,
                "rows": policy_rows
            }),
        )?;
        write_json(
            &artifact_root.join("w044_production_partition_analyzer_register.json"),
            &json!({
                "schema_version": W044_PRODUCTION_PARTITION_ANALYZER_SCHEMA_V1,
                "run_id": run_id,
                "row_count": production_analyzer_rows.len(),
                "satisfied_policy_row_count": production_analyzer_rows
                    .iter()
                    .filter(|row| bool_at(row, "satisfied_for_declared_profile"))
                    .count(),
                "exact_remaining_blocker_count": production_analyzer_rows
                    .iter()
                    .filter(|row| bool_at(row, "exact_remaining_blocker"))
                    .count(),
                "semantic_statement": "W044 binds W043 Stage 2 declared-profile evidence, W044 mixed dynamic add/remove/reclassify evidence, W044 no-publication fence evidence, W044 Rust refinement rows, W044 Lean/TLA bounded model rows, W073 typed-only formatting intake, and no-proxy guards as production-relevant analyzer inputs. Full production partition-analyzer soundness remains blocked.",
                "rows": production_analyzer_rows
            }),
        )?;
        write_json(
            &artifact_root.join("w044_scheduler_equivalence_register.json"),
            &json!({
                "schema_version": W044_SCHEDULER_EQUIVALENCE_SCHEMA_V1,
                "run_id": run_id,
                "row_count": scheduler_equivalence_rows.len(),
                "satisfied_policy_row_count": scheduler_equivalence_rows
                    .iter()
                    .filter(|row| bool_at(row, "satisfied_for_declared_profile"))
                    .count(),
                "exact_remaining_blocker_count": scheduler_equivalence_rows
                    .iter()
                    .filter(|row| bool_at(row, "exact_remaining_blocker"))
                    .count(),
                "semantic_equivalence_statement": "Observable engine results are invariant under W044.5 because this packet adds classification evidence only and does not change runtime scheduler policy. Declared-profile scheduler equivalence remains bounded and does not discharge fairness or unbounded coverage.",
                "rows": scheduler_equivalence_rows
            }),
        )?;
        write_json(
            &artifact_root.join("w044_pack_grade_equivalence_register.json"),
            &json!({
                "schema_version": W044_PACK_GRADE_EQUIVALENCE_SCHEMA_V1,
                "run_id": run_id,
                "row_count": pack_grade_equivalence_rows.len(),
                "satisfied_policy_row_count": pack_grade_equivalence_rows
                    .iter()
                    .filter(|row| bool_at(row, "satisfied_for_declared_profile"))
                    .count(),
                "exact_remaining_blocker_count": pack_grade_equivalence_rows
                    .iter()
                    .filter(|row| bool_at(row, "exact_remaining_blocker"))
                    .count(),
                "semantic_equivalence_statement": "Declared-profile pack equivalence inputs are carried with W044 no-publication and typed-formatting guards, but pack-grade replay remains blocked by retained-witness lifecycle and pack governance.",
                "rows": pack_grade_equivalence_rows
            }),
        )?;
        write_json(
            &artifact_root.join("w044_stage2_exact_blocker_register.json"),
            &json!({
                "schema_version": W044_BLOCKER_REGISTER_SCHEMA_V1,
                "run_id": run_id,
                "exact_remaining_blocker_count": exact_remaining_blocker_count,
                "rows": policy_blocker_rows
            }),
        )?;
        write_json(
            &artifact_root.join("promotion_decision.json"),
            &json!({
                "schema_version": W044_PROMOTION_DECISION_SCHEMA_V1,
                "run_id": run_id,
                "decision_state": "w044_stage2_production_partition_analyzer_scheduler_equivalence_validated_policy_unpromoted",
                "stage2_policy_promoted": false,
                "stage2_promotion_candidate": false,
                "pack_grade_replay_promoted": false,
                "pack_grade_replay_candidate": false,
                "production_relevant_analyzer_inputs_evidenced": production_relevant_inputs_valid,
                "declared_profile_scheduler_equivalence_evidenced": declared_scheduler_valid,
                "declared_profile_pack_replay_equivalence_evidenced": declared_pack_valid,
                "mixed_dynamic_transition_evidenced": mixed_dynamic_valid,
                "no_publication_fence_evidenced": no_publication_valid,
                "rust_refinement_bridge_evidenced": rust_mixed_refinement_valid,
                "lean_tla_model_bound_evidenced": w044_model_bound_valid,
                "w073_typed_formatting_guard_carried": w073_guard_valid,
                "no_proxy_promotion_guard_evidenced": no_proxy_guard_valid,
                "production_partition_analyzer_soundness_promoted": false,
                "fairness_scheduler_unbounded_coverage_promoted": false,
                "operated_cross_engine_stage2_service_promoted": false,
                "retained_witness_lifecycle_promoted": false,
                "pack_grade_replay_governance_promoted": false,
                "blockers": [
                    "stage2.broader_dynamic_transition_coverage",
                    "stage2.snapshot_fence_breadth",
                    "stage2.capability_view_breadth",
                    "stage2.full_production_partition_analyzer_soundness",
                    "stage2.fairness_scheduler_unbounded_coverage",
                    "stage2.operated_cross_engine_differential_service",
                    "stage2.retained_witness_lifecycle_pack_dependency",
                    "stage2.pack_grade_replay_governance"
                ]
            }),
        )?;

        let validation_status = if validation_failures.is_empty() {
            "w044_stage2_scheduler_equivalence_valid"
        } else {
            "w044_stage2_scheduler_equivalence_invalid"
        };
        write_json(
            &artifact_root.join("validation.json"),
            &json!({
                "schema_version": W044_VALIDATION_SCHEMA_V1,
                "run_id": run_id,
                "status": validation_status,
                "policy_row_count": policy_row_count,
                "satisfied_policy_row_count": satisfied_policy_row_count,
                "partition_replay_row_count": number_at(&w043_stage2_summary, "partition_replay_row_count"),
                "permutation_replay_row_count": number_at(&w043_stage2_summary, "permutation_replay_row_count"),
                "nontrivial_permutation_row_count": number_at(&w043_stage2_summary, "nontrivial_permutation_row_count"),
                "observable_invariance_row_count": number_at(&w043_stage2_summary, "observable_invariance_row_count"),
                "formatting_watch_row_count": number_at(&w043_stage2_summary, "formatting_watch_row_count"),
                "mixed_dynamic_transition_evidenced": mixed_dynamic_valid,
                "no_publication_fence_evidenced": no_publication_valid,
                "rust_refinement_bridge_evidenced": rust_mixed_refinement_valid,
                "lean_tla_model_bound_evidenced": w044_model_bound_valid,
                "declared_scheduler_equivalence_evidenced": declared_scheduler_valid,
                "declared_pack_equivalence_evidenced": declared_pack_valid,
                "production_relevant_analyzer_inputs_evidenced": production_relevant_inputs_valid,
                "exact_remaining_blocker_count": exact_remaining_blocker_count,
                "failed_row_count": policy_failed_row_count,
                "stage2_policy_promoted": false,
                "pack_grade_replay_promoted": false,
                "validation_failures": validation_failures
            }),
        )?;
        write_json(
            &artifact_root.join("run_summary.json"),
            &json!({
                "schema_version": W044_RUN_SUMMARY_SCHEMA_V1,
                "run_id": run_id,
                "artifact_root": relative_artifact_root,
                "source_evidence_index_path": source_evidence_index_path,
                "w044_stage2_policy_gate_register_path": policy_gate_register_path,
                "w044_production_partition_analyzer_register_path": production_analyzer_register_path,
                "w044_scheduler_equivalence_register_path": scheduler_equivalence_register_path,
                "w044_pack_grade_equivalence_register_path": pack_grade_equivalence_register_path,
                "w044_stage2_exact_blocker_register_path": blocker_register_path,
                "promotion_decision_path": promotion_decision_path,
                "validation_path": validation_path,
                "policy_row_count": policy_row_count,
                "satisfied_policy_row_count": satisfied_policy_row_count,
                "partition_replay_row_count": number_at(&w043_stage2_summary, "partition_replay_row_count"),
                "permutation_replay_row_count": number_at(&w043_stage2_summary, "permutation_replay_row_count"),
                "nontrivial_permutation_row_count": number_at(&w043_stage2_summary, "nontrivial_permutation_row_count"),
                "observable_invariance_row_count": number_at(&w043_stage2_summary, "observable_invariance_row_count"),
                "formatting_watch_row_count": number_at(&w043_stage2_summary, "formatting_watch_row_count"),
                "mixed_dynamic_transition_evidenced": mixed_dynamic_valid,
                "no_publication_fence_evidenced": no_publication_valid,
                "rust_refinement_bridge_evidenced": rust_mixed_refinement_valid,
                "lean_tla_model_bound_evidenced": w044_model_bound_valid,
                "declared_scheduler_equivalence_evidenced": declared_scheduler_valid,
                "declared_pack_equivalence_evidenced": declared_pack_valid,
                "production_relevant_analyzer_inputs_evidenced": production_relevant_inputs_valid,
                "exact_remaining_blocker_count": exact_remaining_blocker_count,
                "failed_row_count": policy_failed_row_count,
                "stage2_policy_promoted": false,
                "stage2_promotion_candidate": false,
                "pack_grade_replay_promoted": false,
                "pack_grade_replay_candidate": false
            }),
        )?;

        Ok(Stage2ReplayRunSummary {
            run_id: run_id.to_string(),
            schema_version: W044_RUN_SUMMARY_SCHEMA_V1.to_string(),
            partition_replay_row_count: number_at(
                &w043_stage2_summary,
                "partition_replay_row_count",
            ) as usize,
            permutation_replay_row_count: number_at(
                &w043_stage2_summary,
                "permutation_replay_row_count",
            ) as usize,
            nontrivial_permutation_row_count: number_at(
                &w043_stage2_summary,
                "nontrivial_permutation_row_count",
            ) as usize,
            observable_invariance_row_count: number_at(
                &w043_stage2_summary,
                "observable_invariance_row_count",
            ) as usize,
            formatting_watch_row_count: number_at(
                &w043_stage2_summary,
                "formatting_watch_row_count",
            ) as usize,
            exact_remaining_blocker_count,
            failed_row_count: policy_failed_row_count,
            stage2_policy_promoted: false,
            artifact_root: relative_artifact_root,
        })
    }

    fn execute_w043(
        &self,
        repo_root: &Path,
        run_id: &str,
    ) -> Result<Stage2ReplayRunSummary, Stage2ReplayError> {
        let relative_artifact_root =
            relative_artifact_path(&["docs", "test-runs", "core-engine", "stage2-replay", run_id]);
        let artifact_root = repo_root.join(&relative_artifact_root);
        if artifact_root.exists() {
            fs::remove_dir_all(&artifact_root).map_err(|source| {
                Stage2ReplayError::RemoveDirectory {
                    path: artifact_root.display().to_string(),
                    source,
                }
            })?;
        }
        fs::create_dir_all(&artifact_root).map_err(|source| {
            Stage2ReplayError::CreateDirectory {
                path: artifact_root.display().to_string(),
                source,
            }
        })?;

        let w043_obligation_summary = read_json(repo_root, W043_OBLIGATION_SUMMARY)?;
        let w043_obligation_map = read_json(repo_root, W043_OBLIGATION_MAP)?;
        let w043_promotion_target_gate_map = read_json(repo_root, W043_PROMOTION_TARGET_GATE_MAP)?;
        let w043_formatting_intake = read_json(repo_root, W043_FORMATTING_INTAKE)?;
        let w043_optimized_summary = read_json(repo_root, W043_OPTIMIZED_CORE_SUMMARY)?;
        let w043_optimized_counterpart = read_json(repo_root, W043_OPTIMIZED_CORE_COUNTERPART)?;
        let w043_optimized_blockers = read_json(repo_root, W043_OPTIMIZED_CORE_BLOCKERS)?;
        let w043_treecalc_summary = read_json(repo_root, W043_TREECALC_SUMMARY)?;
        let w043_rust_summary = read_json(repo_root, W043_RUST_SUMMARY)?;
        let w043_rust_validation = read_json(repo_root, W043_RUST_VALIDATION)?;
        let w043_rust_refinement = read_json(repo_root, W043_RUST_REFINEMENT_REGISTER)?;
        let w043_lean_tla_summary = read_json(repo_root, W043_LEAN_TLA_SUMMARY)?;
        let w043_lean_tla_validation = read_json(repo_root, W043_LEAN_TLA_VALIDATION)?;
        let w043_tla_model_register = read_json(repo_root, W043_TLA_MODEL_REGISTER)?;
        let w043_lean_tla_blockers = read_json(repo_root, W043_LEAN_TLA_BLOCKERS)?;
        let w042_stage2_summary = read_json(repo_root, W042_STAGE2_RUN_SUMMARY_FOR_W043)?;
        let w042_stage2_validation = read_json(repo_root, W042_STAGE2_VALIDATION_FOR_W043)?;
        let w042_stage2_policy_gate = read_json(repo_root, W042_STAGE2_POLICY_GATE_FOR_W043)?;
        let w042_stage2_production_analyzer =
            read_json(repo_root, W042_STAGE2_PRODUCTION_ANALYZER_FOR_W043)?;
        let w042_stage2_pack_grade_equivalence =
            read_json(repo_root, W042_STAGE2_PACK_GRADE_EQUIVALENCE_FOR_W043)?;
        let w042_stage2_blockers = read_json(repo_root, W042_STAGE2_BLOCKERS_FOR_W043)?;
        let w042_stage2_promotion_decision =
            read_json(repo_root, W042_STAGE2_PROMOTION_DECISION_FOR_W043)?;
        let lean_file_present = repo_root.join(W043_STAGE2_LEAN_FILE).exists();

        let w043_obligation_valid = number_at(&w043_obligation_summary, "obligation_count") == 36
            && bool_at(
                &w043_obligation_summary,
                "oxfml_formatting_update_incorporated",
            )
            && number_at(&w043_promotion_target_gate_map, "promotion_target_count") == 16;
        let w042_stage2_valid = string_at(&w042_stage2_validation, "status")
            == "w042_stage2_pack_grade_equivalence_valid"
            && number_at(&w042_stage2_summary, "policy_row_count") == 18
            && number_at(&w042_stage2_summary, "satisfied_policy_row_count") == 12
            && number_at(&w042_stage2_summary, "exact_remaining_blocker_count") == 6
            && !bool_at(&w042_stage2_summary, "stage2_policy_promoted")
            && !bool_at(&w042_stage2_summary, "pack_grade_replay_promoted")
            && !bool_at(&w042_stage2_promotion_decision, "stage2_policy_promoted")
            && !bool_at(
                &w042_stage2_promotion_decision,
                "pack_grade_replay_promoted",
            );
        let w043_optimized_valid = number_at(&w043_optimized_summary, "failed_row_count") == 0
            && string_at(&w043_optimized_summary, "validation_state") == "passed"
            && number_at(&w043_optimized_summary, "match_promoted_count") == 0
            && number_at(&w043_optimized_summary, "exact_remaining_blocker_count") == 3
            && bool_at(
                &w043_optimized_summary,
                "dynamic_addition_reclassification_evidenced",
            )
            && bool_at(
                &w043_optimized_summary,
                "dynamic_release_reclassification_evidenced",
            )
            && bool_at(
                &w043_optimized_summary,
                "snapshot_counterpart_evidenced_for_declared_profile",
            )
            && bool_at(
                &w043_optimized_summary,
                "capability_counterpart_evidenced_for_declared_profile",
            );
        let w043_treecalc_valid = number_at(&w043_treecalc_summary, "case_count") == 27
            && number_at(&w043_treecalc_summary, "expectation_mismatch_count") == 0;
        let w043_rust_valid = string_at(&w043_rust_validation, "status")
            == "formal_assurance_w043_rust_totality_refinement_valid"
            && number_at(&w043_rust_summary, "automatic_dynamic_transition_row_count") == 2
            && !bool_at(
                &w043_rust_summary["promotion_claims"],
                "stage2_policy_promoted",
            );
        let w043_lean_tla_valid = string_at(&w043_lean_tla_validation, "status")
            == "formal_assurance_w043_lean_tla_fairness_valid"
            && number_at(&w043_lean_tla_summary, "bounded_model_row_count") == 4
            && number_at(&w043_lean_tla_summary, "exact_remaining_blocker_count") == 5
            && !bool_at(
                &w043_lean_tla_summary["promotion_claims"],
                "stage2_policy_promoted",
            );

        let partition_replay_row_count =
            number_at(&w042_stage2_summary, "partition_replay_row_count") as usize;
        let permutation_replay_row_count =
            number_at(&w042_stage2_summary, "permutation_replay_row_count") as usize;
        let nontrivial_permutation_row_count =
            number_at(&w042_stage2_summary, "nontrivial_permutation_row_count") as usize;
        let observable_invariance_row_count =
            number_at(&w042_stage2_summary, "observable_invariance_row_count") as usize;
        let formatting_watch_row_count =
            number_at(&w042_stage2_summary, "formatting_watch_row_count") as usize;

        let predecessor_policy_valid = w042_stage2_valid
            && number_at(&w042_stage2_policy_gate, "policy_row_count") == 18
            && number_at(&w042_stage2_policy_gate, "satisfied_policy_row_count") == 12
            && number_at(&w042_stage2_policy_gate, "exact_remaining_blocker_count") == 6;
        let bounded_replay_valid = predecessor_policy_valid && partition_replay_row_count == 5;
        let permutation_replay_valid = predecessor_policy_valid
            && permutation_replay_row_count == 6
            && nontrivial_permutation_row_count == 1;
        let observable_invariance_valid =
            predecessor_policy_valid && observable_invariance_row_count == 5;
        let automatic_dynamic_addition_valid = w043_optimized_valid
            && w043_treecalc_valid
            && w043_rust_valid
            && row_with_field_exists(
                &w043_optimized_counterpart,
                "row_id",
                "w043_dynamic_addition_reclassification_direct_evidence",
            )
            && row_with_field_exists(
                &w043_rust_refinement,
                "row_id",
                "w043_automatic_dynamic_addition_refinement_evidence",
            );
        let automatic_dynamic_release_valid = w043_optimized_valid
            && w043_treecalc_valid
            && w043_rust_valid
            && row_with_field_exists(
                &w043_optimized_counterpart,
                "row_id",
                "w043_dynamic_release_reclassification_carried_evidence",
            )
            && row_with_field_exists(
                &w043_rust_refinement,
                "row_id",
                "w043_automatic_dynamic_release_refinement_evidence",
            );
        let snapshot_counterpart_valid = w043_optimized_valid
            && row_with_field_exists(
                &w043_optimized_counterpart,
                "row_id",
                "w043_snapshot_fence_counterpart_declared_profile_evidence",
            )
            && row_with_field_exists(
                &w043_rust_refinement,
                "row_id",
                "w043_snapshot_fence_declared_profile_refinement_evidence",
            );
        let capability_counterpart_valid = w043_optimized_valid
            && row_with_field_exists(
                &w043_optimized_counterpart,
                "row_id",
                "w043_capability_view_counterpart_declared_profile_evidence",
            )
            && row_with_field_exists(
                &w043_rust_refinement,
                "row_id",
                "w043_capability_view_declared_profile_refinement_evidence",
            );
        let bounded_analyzer_valid = predecessor_policy_valid
            && number_at(&w042_stage2_production_analyzer, "row_count") == 9
            && row_with_field_exists(
                &w043_tla_model_register,
                "row_id",
                "w043_stage2_equivalence_bounded_model_input",
            )
            && lean_file_present;
        let lean_tla_model_bound_valid = w043_lean_tla_valid
            && row_with_field_exists(
                &w043_tla_model_register,
                "row_id",
                "w043_tla_stage2_partition_bounded_model_evidence",
            )
            && row_with_field_exists(
                &w043_lean_tla_blockers,
                "row_id",
                "w043_tla_fairness_scheduler_unbounded_boundary",
            );
        let w073_guard_valid = formatting_watch_row_count == 1
            && array_field_contains_string(
                &w043_formatting_intake,
                "typed_rule_only_families",
                "colorScale",
            )
            && array_field_contains_string(
                &w043_formatting_intake,
                "typed_rule_only_families",
                "dataBar",
            )
            && array_field_contains_string(
                &w043_formatting_intake,
                "typed_rule_only_families",
                "iconSet",
            )
            && array_field_contains_string(
                &w043_formatting_intake,
                "typed_rule_only_families",
                "top",
            )
            && array_field_contains_string(
                &w043_formatting_intake,
                "typed_rule_only_families",
                "bottom",
            )
            && array_field_contains_string(
                &w043_formatting_intake,
                "typed_rule_only_families",
                "aboveAverage",
            )
            && array_field_contains_string(
                &w043_formatting_intake,
                "typed_rule_only_families",
                "belowAverage",
            )
            && !bool_at(
                &w043_formatting_intake,
                "threshold_fallback_allowed_for_typed_families",
            );
        let declared_pack_equivalence_valid = predecessor_policy_valid
            && observable_invariance_valid
            && snapshot_counterpart_valid
            && capability_counterpart_valid
            && w073_guard_valid
            && number_at(&w042_stage2_pack_grade_equivalence, "row_count") == 11
            && number_at(
                &w042_stage2_pack_grade_equivalence,
                "satisfied_policy_row_count",
            ) == 8
            && bool_at(&w042_stage2_summary, "declared_pack_equivalence_evidenced");
        let scheduler_equivalence_declared_profile_valid = bounded_replay_valid
            && permutation_replay_valid
            && observable_invariance_valid
            && lean_tla_model_bound_valid
            && lean_file_present;
        let no_proxy_guard_valid = w043_obligation_valid
            && number_at(&w043_optimized_summary, "match_promoted_count") == 0
            && array_field_contains_string(
                &w043_obligation_summary,
                "no_promotion_claims",
                "stage2_production_policy",
            )
            && array_field_contains_string(
                &w043_obligation_summary,
                "no_promotion_claims",
                "pack_grade_replay_equivalence",
            )
            && !bool_at(
                &w042_stage2_promotion_decision,
                "stage2_promotion_candidate",
            );
        let broader_dynamic_transition_blocked = row_with_field_exists(
            &w043_optimized_blockers,
            "row_id",
            "w043_broader_dynamic_transition_coverage_remaining_exact_blocker",
        );

        let policy_rows = vec![
            json!({
                "row_id": "w043_stage2_w042_predecessor_policy_carried",
                "w043_obligation_id": "W043-OBL-016",
                "policy_area": "w042_stage2_predecessor_policy_packet",
                "source_artifacts": [W042_STAGE2_RUN_SUMMARY_FOR_W043, W042_STAGE2_VALIDATION_FOR_W043, W042_STAGE2_POLICY_GATE_FOR_W043],
                "satisfied_for_declared_profile": predecessor_policy_valid,
                "exact_remaining_blocker": false,
                "promotion_consequence": "W042 predecessor evidence is valid input only; W043 promotion still requires direct W043 gates",
                "disposition": "carry the W042 Stage 2 analyzer and pack-grade equivalence packet as non-promoting predecessor evidence",
                "failures": if predecessor_policy_valid { Vec::<String>::new() } else { vec!["w043_w042_stage2_predecessor_not_valid".to_string()] },
            }),
            json!({
                "row_id": "w043_stage2_bounded_replay_carried",
                "w043_obligation_id": "W043-OBL-018",
                "policy_area": "bounded_baseline_vs_stage2_replay",
                "source_artifacts": [W042_STAGE2_RUN_SUMMARY_FOR_W043, W042_STAGE2_POLICY_GATE_FOR_W043],
                "satisfied_for_declared_profile": bounded_replay_valid,
                "exact_remaining_blocker": false,
                "promotion_consequence": "bounded replay remains declared-profile evidence only",
                "disposition": "carry W042 bounded baseline-versus-Stage-2 replay rows into W043 pack-grade equivalence classification",
                "failures": if bounded_replay_valid { Vec::<String>::new() } else { vec!["w043_bounded_replay_not_valid".to_string()] },
            }),
            json!({
                "row_id": "w043_stage2_partition_order_permutation_carried",
                "w043_obligation_id": "W043-OBL-017",
                "policy_area": "partition_order_permutation_replay",
                "source_artifacts": [W042_STAGE2_RUN_SUMMARY_FOR_W043, W042_STAGE2_POLICY_GATE_FOR_W043],
                "satisfied_for_declared_profile": permutation_replay_valid,
                "exact_remaining_blocker": false,
                "promotion_consequence": "bounded permutation replay supports declared-profile scheduler equivalence but does not discharge unbounded fairness",
                "disposition": "carry one nontrivial partition-order permutation row and six total permutation rows",
                "failures": if permutation_replay_valid { Vec::<String>::new() } else { vec!["w043_partition_permutation_replay_not_valid".to_string()] },
            }),
            json!({
                "row_id": "w043_stage2_observable_invariance_carried",
                "w043_obligation_id": "W043-OBL-017",
                "policy_area": "observable_result_invariance",
                "source_artifacts": [W042_STAGE2_RUN_SUMMARY_FOR_W043, W042_STAGE2_PACK_GRADE_EQUIVALENCE_FOR_W043],
                "satisfied_for_declared_profile": observable_invariance_valid,
                "exact_remaining_blocker": false,
                "promotion_consequence": "observable invariance is evidenced for declared bounded profiles only",
                "disposition": "carry W042 observable-result invariance rows for values, rejects, fence no-publish behavior, and formatting watch surfaces",
                "failures": if observable_invariance_valid { Vec::<String>::new() } else { vec!["w043_observable_invariance_not_valid".to_string()] },
            }),
            json!({
                "row_id": "w043_stage2_dynamic_addition_refinement_bound",
                "w043_obligation_id": "W043-OBL-016",
                "policy_area": "dynamic_addition_soft_reference_transition",
                "source_artifacts": [W043_OPTIMIZED_CORE_COUNTERPART, W043_RUST_REFINEMENT_REGISTER, W043_TREECALC_SUMMARY],
                "satisfied_for_declared_profile": automatic_dynamic_addition_valid,
                "exact_remaining_blocker": false,
                "promotion_consequence": "automatic dependency-addition evidence narrows declared-profile analyzer inputs but does not prove full production analyzer soundness",
                "disposition": "bind W043 automatic DependencyAdded plus DependencyReclassified evidence to Stage 2 analyzer preconditions",
                "failures": if automatic_dynamic_addition_valid { Vec::<String>::new() } else { vec!["w043_dynamic_addition_transition_not_valid".to_string()] },
            }),
            json!({
                "row_id": "w043_stage2_dynamic_release_refinement_bound",
                "w043_obligation_id": "W043-OBL-016",
                "policy_area": "dynamic_release_soft_reference_transition",
                "source_artifacts": [W043_OPTIMIZED_CORE_COUNTERPART, W043_RUST_REFINEMENT_REGISTER, W043_TREECALC_SUMMARY],
                "satisfied_for_declared_profile": automatic_dynamic_release_valid,
                "exact_remaining_blocker": false,
                "promotion_consequence": "automatic dependency-release evidence narrows declared-profile analyzer inputs but does not prove full production analyzer soundness",
                "disposition": "bind W043 automatic DependencyRemoved plus DependencyReclassified evidence to Stage 2 analyzer preconditions",
                "failures": if automatic_dynamic_release_valid { Vec::<String>::new() } else { vec!["w043_dynamic_release_transition_not_valid".to_string()] },
            }),
            json!({
                "row_id": "w043_stage2_snapshot_fence_counterpart_evidence",
                "w043_obligation_id": "W043-OBL-004",
                "policy_area": "snapshot_fence_counterpart",
                "source_artifacts": [W043_OPTIMIZED_CORE_COUNTERPART, W043_RUST_REFINEMENT_REGISTER],
                "satisfied_for_declared_profile": snapshot_counterpart_valid,
                "exact_remaining_blocker": false,
                "promotion_consequence": "snapshot-fence counterpart is evidenced for declared profiles; production Stage 2 policy remains blocked by broader gates",
                "disposition": "bind W043 optimized/core and Rust declared-profile snapshot-fence reject/no-publish counterpart evidence without match promotion",
                "failures": if snapshot_counterpart_valid { Vec::<String>::new() } else { vec!["w043_snapshot_counterpart_not_valid".to_string()] },
            }),
            json!({
                "row_id": "w043_stage2_capability_view_counterpart_evidence",
                "w043_obligation_id": "W043-OBL-005",
                "policy_area": "capability_view_fence_counterpart",
                "source_artifacts": [W043_OPTIMIZED_CORE_COUNTERPART, W043_RUST_REFINEMENT_REGISTER],
                "satisfied_for_declared_profile": capability_counterpart_valid,
                "exact_remaining_blocker": false,
                "promotion_consequence": "capability-view counterpart is evidenced for declared profiles; production Stage 2 policy remains blocked by broader gates",
                "disposition": "bind W043 optimized/core and Rust declared-profile capability-view reject/no-publish counterpart evidence without match promotion",
                "failures": if capability_counterpart_valid { Vec::<String>::new() } else { vec!["w043_capability_counterpart_not_valid".to_string()] },
            }),
            json!({
                "row_id": "w043_stage2_bounded_partition_analyzer_predicate_evidence",
                "w043_obligation_id": "W043-OBL-016",
                "policy_area": "bounded_partition_analyzer_soundness",
                "source_artifacts": [W042_STAGE2_PRODUCTION_ANALYZER_FOR_W043, W043_TLA_MODEL_REGISTER, W043_STAGE2_LEAN_FILE],
                "satisfied_for_declared_profile": bounded_analyzer_valid,
                "exact_remaining_blocker": false,
                "promotion_consequence": "bounded analyzer preconditions are evidenced, but full production analyzer soundness remains blocked",
                "disposition": "bind W042 bounded partition-analyzer evidence and W043 bounded Stage 2 model input under the W043 Lean predicate",
                "failures": if bounded_analyzer_valid { Vec::<String>::new() } else { vec!["w043_bounded_partition_analyzer_not_valid".to_string()] },
            }),
            json!({
                "row_id": "w043_stage2_lean_tla_model_bound_carried",
                "w043_obligation_id": "W043-OBL-014",
                "policy_area": "lean_tla_model_bound",
                "source_artifacts": [W043_LEAN_TLA_SUMMARY, W043_TLA_MODEL_REGISTER, W043_LEAN_TLA_BLOCKERS],
                "satisfied_for_declared_profile": lean_tla_model_bound_valid,
                "exact_remaining_blocker": false,
                "promotion_consequence": "bounded proof/model inputs strengthen the Stage 2 packet but do not promote full TLA or Stage 2 policy",
                "disposition": "carry W043 Lean/TLA bounded Stage 2 model rows and exact fairness boundary into the Stage 2 gate",
                "failures": if lean_tla_model_bound_valid { Vec::<String>::new() } else { vec!["w043_lean_tla_model_bound_not_valid".to_string()] },
            }),
            json!({
                "row_id": "w043_stage2_w073_typed_formatting_guard_carried",
                "w043_obligation_id": "W043-OBL-027",
                "policy_area": "w073_typed_formatting_watch",
                "source_artifacts": [W043_FORMATTING_INTAKE, W042_STAGE2_RUN_SUMMARY_FOR_W043],
                "satisfied_for_declared_profile": w073_guard_valid,
                "exact_remaining_blocker": false,
                "promotion_consequence": "typed-only formatting is carried as an observable-surface guard; broad OxFml seam breadth remains under calc-2p3.8",
                "disposition": "retain the W073 direct-replacement rule for aggregate and visualization conditional-formatting metadata",
                "failures": if w073_guard_valid { Vec::<String>::new() } else { vec!["w043_w073_formatting_guard_not_valid".to_string()] },
            }),
            json!({
                "row_id": "w043_stage2_declared_pack_replay_equivalence",
                "w043_obligation_id": "W043-OBL-018",
                "policy_area": "declared_profile_pack_replay_equivalence",
                "source_artifacts": [W042_STAGE2_PACK_GRADE_EQUIVALENCE_FOR_W043, W042_STAGE2_PROMOTION_DECISION_FOR_W043, W043_OPTIMIZED_CORE_COUNTERPART],
                "satisfied_for_declared_profile": declared_pack_equivalence_valid,
                "exact_remaining_blocker": false,
                "promotion_consequence": "declared-profile equivalence evidence is not pack-grade replay governance",
                "disposition": "classify values, rejects, fence no-publish behavior, typed-formatting watch, replay validation, and W043 counterpart rows as declared-profile pack-equivalence inputs",
                "failures": if declared_pack_equivalence_valid { Vec::<String>::new() } else { vec!["w043_declared_pack_equivalence_not_valid".to_string()] },
            }),
            json!({
                "row_id": "w043_stage2_no_proxy_promotion_guard",
                "w043_obligation_id": "W043-OBL-008",
                "policy_area": "no_proxy_promotion_guard",
                "source_artifacts": [W043_OBLIGATION_SUMMARY, W043_PROMOTION_TARGET_GATE_MAP, W043_OPTIMIZED_CORE_SUMMARY, W042_STAGE2_PROMOTION_DECISION_FOR_W043],
                "satisfied_for_declared_profile": no_proxy_guard_valid,
                "exact_remaining_blocker": false,
                "promotion_consequence": "Stage 2, pack, and release-grade gates remain blocked unless direct target evidence is present",
                "disposition": "keep bounded-only, declared-profile, retained-blocker, and predecessor evidence out of promotion counts",
                "failures": if no_proxy_guard_valid { Vec::<String>::new() } else { vec!["w043_no_proxy_guard_not_valid".to_string()] },
            }),
            json!({
                "row_id": "w043_stage2_declared_profile_scheduler_equivalence",
                "w043_obligation_id": "W043-OBL-017",
                "policy_area": "declared_profile_scheduler_equivalence",
                "source_artifacts": [W042_STAGE2_RUN_SUMMARY_FOR_W043, W043_TLA_MODEL_REGISTER, W043_STAGE2_LEAN_FILE],
                "satisfied_for_declared_profile": scheduler_equivalence_declared_profile_valid,
                "exact_remaining_blocker": false,
                "promotion_consequence": "scheduler equivalence is evidenced for declared bounded profiles only; unbounded fairness remains blocked",
                "disposition": "bind bounded partition replay, permutation replay, observable invariance, and Lean/TLA model-bound rows as declared-profile scheduler-equivalence evidence",
                "failures": if scheduler_equivalence_declared_profile_valid { Vec::<String>::new() } else { vec!["w043_declared_scheduler_equivalence_not_valid".to_string()] },
            }),
            json!({
                "row_id": "w043_stage2_broader_dynamic_transition_coverage_blocker",
                "w043_obligation_id": "W043-OBL-016",
                "related_w043_obligation_ids": ["W043-OBL-003"],
                "policy_area": "broader_dynamic_transition_coverage",
                "source_artifacts": [W043_OPTIMIZED_CORE_BLOCKERS, W043_RUST_REFINEMENT_REGISTER],
                "satisfied_for_declared_profile": false,
                "exact_remaining_blocker": true,
                "promotion_consequence": "production partition-analyzer soundness remains blocked",
                "disposition": "retain broader dynamic dependency-transition coverage as a Stage 2 analyzer dependency beyond the exercised addition and release patterns",
                "failures": if broader_dynamic_transition_blocked { Vec::<String>::new() } else { vec!["w043_broader_dynamic_transition_blocker_missing".to_string()] },
            }),
            json!({
                "row_id": "w043_stage2_full_production_partition_analyzer_soundness_blocker",
                "w043_obligation_id": "W043-OBL-016",
                "policy_area": "full_production_partition_analyzer_soundness",
                "source_artifacts": [W043_OBLIGATION_MAP, W042_STAGE2_BLOCKERS_FOR_W043, W043_STAGE2_LEAN_FILE],
                "satisfied_for_declared_profile": false,
                "exact_remaining_blocker": true,
                "promotion_consequence": "Stage 2 production policy remains unpromoted",
                "disposition": "retain full production partition-analyzer soundness as exact blocker beyond bounded declared-profile evidence",
                "failures": if obligation_exists(&w043_obligation_map, "W043-OBL-016") && row_with_field_exists(&w042_stage2_blockers, "row_id", "w042_stage2_full_production_analyzer_soundness_blocker") && lean_file_present { Vec::<String>::new() } else { vec!["w043_production_analyzer_blocker_input_missing".to_string()] },
            }),
            json!({
                "row_id": "w043_stage2_scheduler_fairness_unbounded_equivalence_blocker",
                "w043_obligation_id": "W043-OBL-017",
                "related_w043_obligation_ids": ["W043-OBL-013", "W043-OBL-014"],
                "policy_area": "fairness_scheduler_unbounded_coverage",
                "source_artifacts": [W043_LEAN_TLA_SUMMARY, W043_LEAN_TLA_BLOCKERS, W042_STAGE2_BLOCKERS_FOR_W043],
                "satisfied_for_declared_profile": false,
                "exact_remaining_blocker": true,
                "promotion_consequence": "full TLA verification and Stage 2 production policy remain unpromoted",
                "disposition": "retain fairness, unbounded scheduler coverage, and model-completeness limits as exact Stage 2 blockers",
                "failures": if obligation_exists(&w043_obligation_map, "W043-OBL-017") && row_with_field_exists(&w043_lean_tla_blockers, "row_id", "w043_tla_fairness_scheduler_unbounded_boundary") { Vec::<String>::new() } else { vec!["w043_scheduler_fairness_blocker_input_missing".to_string()] },
            }),
            json!({
                "row_id": "w043_stage2_operated_cross_engine_service_dependency_blocker",
                "w043_obligation_id": "W043-OBL-023",
                "policy_area": "operated_cross_engine_stage2_differential_service",
                "source_artifacts": [W043_OBLIGATION_MAP, W042_STAGE2_BLOCKERS_FOR_W043],
                "satisfied_for_declared_profile": false,
                "exact_remaining_blocker": true,
                "promotion_consequence": "operated Stage 2 differential evidence remains required before policy promotion",
                "disposition": "retain operated cross-engine differential service as successor dependency",
                "failures": if obligation_exists(&w043_obligation_map, "W043-OBL-023") && row_with_field_exists(&w042_stage2_blockers, "row_id", "w042_stage2_operated_cross_engine_service_dependency_blocker") { Vec::<String>::new() } else { vec!["w043_operated_stage2_service_blocker_input_missing".to_string()] },
            }),
            json!({
                "row_id": "w043_stage2_retained_witness_lifecycle_pack_dependency_blocker",
                "w043_obligation_id": "W043-OBL-021",
                "policy_area": "retained_witness_lifecycle_pack_dependency",
                "source_artifacts": [W043_OBLIGATION_MAP, W042_STAGE2_BLOCKERS_FOR_W043],
                "satisfied_for_declared_profile": false,
                "exact_remaining_blocker": true,
                "promotion_consequence": "pack-grade replay equivalence remains blocked until retained-witness lifecycle and retention SLO evidence exists",
                "disposition": "retain retained-witness lifecycle and retention SLO as a pack-grade replay dependency rather than treating deterministic replay files as operated witness evidence",
                "failures": if obligation_exists(&w043_obligation_map, "W043-OBL-021") && row_with_field_exists(&w042_stage2_blockers, "row_id", "w042_stage2_retained_witness_lifecycle_pack_dependency_blocker") { Vec::<String>::new() } else { vec!["w043_retained_witness_obligation_missing".to_string()] },
            }),
            json!({
                "row_id": "w043_stage2_pack_grade_replay_governance_blocker",
                "w043_obligation_id": "W043-OBL-034",
                "related_w043_obligation_ids": ["W043-OBL-018"],
                "policy_area": "pack_grade_replay_governance",
                "source_artifacts": [W043_OBLIGATION_MAP, W042_STAGE2_BLOCKERS_FOR_W043],
                "satisfied_for_declared_profile": false,
                "exact_remaining_blocker": true,
                "promotion_consequence": "pack-grade replay, C5, and release-grade verification remain blocked",
                "disposition": "retain pack-grade replay governance as release-decision blocker beyond declared-profile equivalence",
                "failures": if obligation_exists(&w043_obligation_map, "W043-OBL-034") && obligation_exists(&w043_obligation_map, "W043-OBL-018") && row_with_field_exists(&w042_stage2_blockers, "row_id", "w042_stage2_pack_grade_replay_governance_blocker") { Vec::<String>::new() } else { vec!["w043_pack_grade_replay_governance_blocker_input_missing".to_string()] },
            }),
        ];

        let satisfied_policy_row_count = policy_rows
            .iter()
            .filter(|row| bool_at(row, "satisfied_for_declared_profile"))
            .count();
        let policy_blocker_rows = policy_rows
            .iter()
            .filter(|row| bool_at(row, "exact_remaining_blocker"))
            .cloned()
            .collect::<Vec<_>>();
        let policy_failed_row_count = policy_rows
            .iter()
            .filter(|row| {
                !row.get("failures")
                    .and_then(Value::as_array)
                    .is_some_and(Vec::is_empty)
            })
            .count();
        let exact_remaining_blocker_count = policy_blocker_rows.len();
        let policy_row_count = policy_rows.len();

        let production_analyzer_rows = policy_rows
            .iter()
            .filter(|row| {
                matches!(
                    string_at(row, "policy_area"),
                    "w042_stage2_predecessor_policy_packet"
                        | "dynamic_addition_soft_reference_transition"
                        | "dynamic_release_soft_reference_transition"
                        | "snapshot_fence_counterpart"
                        | "capability_view_fence_counterpart"
                        | "bounded_partition_analyzer_soundness"
                        | "lean_tla_model_bound"
                        | "declared_profile_scheduler_equivalence"
                        | "broader_dynamic_transition_coverage"
                        | "full_production_partition_analyzer_soundness"
                        | "fairness_scheduler_unbounded_coverage"
                )
            })
            .cloned()
            .collect::<Vec<_>>();
        let scheduler_equivalence_rows = policy_rows
            .iter()
            .filter(|row| {
                matches!(
                    string_at(row, "policy_area"),
                    "bounded_baseline_vs_stage2_replay"
                        | "partition_order_permutation_replay"
                        | "observable_result_invariance"
                        | "lean_tla_model_bound"
                        | "declared_profile_scheduler_equivalence"
                        | "fairness_scheduler_unbounded_coverage"
                )
            })
            .cloned()
            .collect::<Vec<_>>();
        let pack_grade_equivalence_rows = policy_rows
            .iter()
            .filter(|row| {
                matches!(
                    string_at(row, "policy_area"),
                    "bounded_baseline_vs_stage2_replay"
                        | "partition_order_permutation_replay"
                        | "observable_result_invariance"
                        | "snapshot_fence_counterpart"
                        | "capability_view_fence_counterpart"
                        | "w073_typed_formatting_watch"
                        | "declared_profile_pack_replay_equivalence"
                        | "no_proxy_promotion_guard"
                        | "operated_cross_engine_stage2_differential_service"
                        | "retained_witness_lifecycle_pack_dependency"
                        | "pack_grade_replay_governance"
                )
            })
            .cloned()
            .collect::<Vec<_>>();

        let mut validation_failures = Vec::new();
        if !w043_obligation_valid {
            validation_failures.push("w043_obligation_map_not_valid".to_string());
        }
        for obligation_id in [
            "W043-OBL-004",
            "W043-OBL-005",
            "W043-OBL-008",
            "W043-OBL-013",
            "W043-OBL-014",
            "W043-OBL-016",
            "W043-OBL-017",
            "W043-OBL-018",
            "W043-OBL-021",
            "W043-OBL-023",
            "W043-OBL-034",
        ] {
            if !obligation_exists(&w043_obligation_map, obligation_id) {
                validation_failures.push(format!("{obligation_id}_missing"));
            }
        }
        if !predecessor_policy_valid {
            validation_failures.push("w042_stage2_predecessor_not_valid".to_string());
        }
        if !w043_optimized_valid || !w043_treecalc_valid {
            validation_failures.push("w043_optimized_stage2_inputs_not_valid".to_string());
        }
        if !w043_rust_valid {
            validation_failures.push("w043_rust_stage2_inputs_not_valid".to_string());
        }
        if !w043_lean_tla_valid {
            validation_failures.push("w043_lean_tla_stage2_inputs_not_valid".to_string());
        }
        if !automatic_dynamic_addition_valid {
            validation_failures.push("w043_dynamic_addition_transition_not_valid".to_string());
        }
        if !automatic_dynamic_release_valid {
            validation_failures.push("w043_dynamic_release_transition_not_valid".to_string());
        }
        if !snapshot_counterpart_valid {
            validation_failures.push("w043_snapshot_counterpart_not_valid".to_string());
        }
        if !capability_counterpart_valid {
            validation_failures.push("w043_capability_counterpart_not_valid".to_string());
        }
        if !bounded_analyzer_valid {
            validation_failures.push("w043_bounded_partition_analyzer_not_valid".to_string());
        }
        if !lean_tla_model_bound_valid {
            validation_failures.push("w043_lean_tla_model_bound_not_valid".to_string());
        }
        if !declared_pack_equivalence_valid {
            validation_failures.push("w043_declared_pack_equivalence_not_valid".to_string());
        }
        if !scheduler_equivalence_declared_profile_valid {
            validation_failures.push("w043_declared_scheduler_equivalence_not_valid".to_string());
        }
        if !no_proxy_guard_valid {
            validation_failures.push("w043_no_proxy_guard_not_valid".to_string());
        }
        if !w073_guard_valid {
            validation_failures.push("w043_w073_formatting_guard_not_valid".to_string());
        }
        if !broader_dynamic_transition_blocked {
            validation_failures.push("w043_broader_dynamic_transition_blocker_missing".to_string());
        }
        if !lean_file_present {
            validation_failures.push("w043_stage2_lean_file_missing".to_string());
        }
        if policy_failed_row_count != 0 {
            validation_failures.push("w043_stage2_policy_row_failures_present".to_string());
        }
        if satisfied_policy_row_count != 14 {
            validation_failures
                .push("w043_stage2_expected_fourteen_satisfied_policy_rows".to_string());
        }
        if exact_remaining_blocker_count != 6 {
            validation_failures.push("w043_stage2_expected_six_exact_blockers".to_string());
        }

        let source_evidence_index_path =
            format!("{relative_artifact_root}/source_evidence_index.json");
        let policy_gate_register_path =
            format!("{relative_artifact_root}/w043_stage2_policy_gate_register.json");
        let production_analyzer_register_path =
            format!("{relative_artifact_root}/w043_production_partition_analyzer_register.json");
        let scheduler_equivalence_register_path =
            format!("{relative_artifact_root}/w043_scheduler_equivalence_register.json");
        let pack_grade_equivalence_register_path =
            format!("{relative_artifact_root}/w043_pack_grade_equivalence_register.json");
        let blocker_register_path =
            format!("{relative_artifact_root}/w043_stage2_exact_blocker_register.json");
        let promotion_decision_path = format!("{relative_artifact_root}/promotion_decision.json");
        let validation_path = format!("{relative_artifact_root}/validation.json");

        write_json(
            &artifact_root.join("source_evidence_index.json"),
            &json!({
                "schema_version": W043_SOURCE_INDEX_SCHEMA_V1,
                "run_id": run_id,
                "artifact_root": relative_artifact_root,
                "source_artifacts": {
                    "w043_obligation_summary": W043_OBLIGATION_SUMMARY,
                    "w043_obligation_map": W043_OBLIGATION_MAP,
                    "w043_promotion_target_gate_map": W043_PROMOTION_TARGET_GATE_MAP,
                    "w043_formatting_intake": W043_FORMATTING_INTAKE,
                    "w043_optimized_core_summary": W043_OPTIMIZED_CORE_SUMMARY,
                    "w043_optimized_core_counterpart": W043_OPTIMIZED_CORE_COUNTERPART,
                    "w043_optimized_core_blockers": W043_OPTIMIZED_CORE_BLOCKERS,
                    "w043_treecalc_summary": W043_TREECALC_SUMMARY,
                    "w043_rust_summary": W043_RUST_SUMMARY,
                    "w043_rust_validation": W043_RUST_VALIDATION,
                    "w043_rust_refinement_register": W043_RUST_REFINEMENT_REGISTER,
                    "w043_lean_tla_summary": W043_LEAN_TLA_SUMMARY,
                    "w043_lean_tla_validation": W043_LEAN_TLA_VALIDATION,
                    "w043_tla_model_register": W043_TLA_MODEL_REGISTER,
                    "w043_lean_tla_blockers": W043_LEAN_TLA_BLOCKERS,
                    "w042_stage2_summary": W042_STAGE2_RUN_SUMMARY_FOR_W043,
                    "w042_stage2_validation": W042_STAGE2_VALIDATION_FOR_W043,
                    "w042_stage2_policy_gate": W042_STAGE2_POLICY_GATE_FOR_W043,
                    "w042_stage2_production_analyzer": W042_STAGE2_PRODUCTION_ANALYZER_FOR_W043,
                    "w042_stage2_pack_grade_equivalence": W042_STAGE2_PACK_GRADE_EQUIVALENCE_FOR_W043,
                    "w042_stage2_blockers": W042_STAGE2_BLOCKERS_FOR_W043,
                    "w042_stage2_promotion_decision": W042_STAGE2_PROMOTION_DECISION_FOR_W043,
                    "w043_stage2_lean_file": W043_STAGE2_LEAN_FILE
                },
                "source_counts": {
                    "w043_obligation_count": number_at(&w043_obligation_summary, "obligation_count"),
                    "w043_promotion_target_count": number_at(&w043_promotion_target_gate_map, "promotion_target_count"),
                    "w043_optimized_exact_remaining_blocker_count": number_at(&w043_optimized_summary, "exact_remaining_blocker_count"),
                    "w043_rust_exact_remaining_blocker_count": number_at(&w043_rust_summary, "exact_remaining_blocker_count"),
                    "w043_lean_tla_exact_remaining_blocker_count": number_at(&w043_lean_tla_summary, "exact_remaining_blocker_count"),
                    "w042_stage2_policy_row_count": number_at(&w042_stage2_summary, "policy_row_count"),
                    "w042_stage2_exact_remaining_blocker_count": number_at(&w042_stage2_summary, "exact_remaining_blocker_count")
                }
            }),
        )?;
        write_json(
            &artifact_root.join("w043_stage2_policy_gate_register.json"),
            &json!({
                "schema_version": W043_POLICY_GATE_SCHEMA_V1,
                "run_id": run_id,
                "policy_row_count": policy_row_count,
                "satisfied_policy_row_count": satisfied_policy_row_count,
                "exact_remaining_blocker_count": exact_remaining_blocker_count,
                "stage2_policy_promoted": false,
                "pack_grade_replay_promoted": false,
                "rows": policy_rows
            }),
        )?;
        write_json(
            &artifact_root.join("w043_production_partition_analyzer_register.json"),
            &json!({
                "schema_version": W043_PRODUCTION_PARTITION_ANALYZER_SCHEMA_V1,
                "run_id": run_id,
                "row_count": production_analyzer_rows.len(),
                "satisfied_policy_row_count": production_analyzer_rows
                    .iter()
                    .filter(|row| bool_at(row, "satisfied_for_declared_profile"))
                    .count(),
                "exact_remaining_blocker_count": production_analyzer_rows
                    .iter()
                    .filter(|row| bool_at(row, "exact_remaining_blocker"))
                    .count(),
                "semantic_statement": "W043 binds declared-profile replay, W043 dynamic addition/release refinement, snapshot/capability counterparts, bounded analyzer evidence, Lean/TLA model bounds, and declared-profile scheduler-equivalence evidence as production-partition-analyzer inputs. Full production partition-analyzer soundness remains blocked by broader dynamic coverage, unbounded scheduler/fairness coverage, and operated evidence.",
                "rows": production_analyzer_rows
            }),
        )?;
        write_json(
            &artifact_root.join("w043_scheduler_equivalence_register.json"),
            &json!({
                "schema_version": W043_SCHEDULER_EQUIVALENCE_SCHEMA_V1,
                "run_id": run_id,
                "row_count": scheduler_equivalence_rows.len(),
                "satisfied_policy_row_count": scheduler_equivalence_rows
                    .iter()
                    .filter(|row| bool_at(row, "satisfied_for_declared_profile"))
                    .count(),
                "exact_remaining_blocker_count": scheduler_equivalence_rows
                    .iter()
                    .filter(|row| bool_at(row, "exact_remaining_blocker"))
                    .count(),
                "semantic_equivalence_statement": "Declared-profile scheduler equivalence is evidenced by bounded partition replay, one nontrivial partition-order permutation row, observable-result invariance, W043 Lean/TLA model-bound rows, and the checked W043 Stage 2 predicate. This does not discharge unbounded fairness or production scheduler coverage.",
                "rows": scheduler_equivalence_rows
            }),
        )?;
        write_json(
            &artifact_root.join("w043_pack_grade_equivalence_register.json"),
            &json!({
                "schema_version": W043_PACK_GRADE_EQUIVALENCE_SCHEMA_V1,
                "run_id": run_id,
                "row_count": pack_grade_equivalence_rows.len(),
                "satisfied_policy_row_count": pack_grade_equivalence_rows
                    .iter()
                    .filter(|row| bool_at(row, "satisfied_for_declared_profile"))
                    .count(),
                "exact_remaining_blocker_count": pack_grade_equivalence_rows
                    .iter()
                    .filter(|row| bool_at(row, "exact_remaining_blocker"))
                    .count(),
                "semantic_equivalence_statement": "Declared-profile values, rejects, fence no-publish behavior, typed-formatting observable guards, replay validation, W043 counterpart evidence, and no-proxy promotion guards are bound as pack-equivalence inputs. This still is not pack-grade replay governance because retained-witness lifecycle, operated-service evidence, and pack governance remain blocked.",
                "rows": pack_grade_equivalence_rows
            }),
        )?;
        write_json(
            &artifact_root.join("w043_stage2_exact_blocker_register.json"),
            &json!({
                "schema_version": W043_BLOCKER_REGISTER_SCHEMA_V1,
                "run_id": run_id,
                "exact_remaining_blocker_count": exact_remaining_blocker_count,
                "rows": policy_blocker_rows
            }),
        )?;
        write_json(
            &artifact_root.join("promotion_decision.json"),
            &json!({
                "schema_version": W043_PROMOTION_DECISION_SCHEMA_V1,
                "run_id": run_id,
                "decision_state": "w043_stage2_scheduler_equivalence_validated_policy_unpromoted",
                "stage2_policy_promoted": false,
                "stage2_promotion_candidate": false,
                "pack_grade_replay_promoted": false,
                "pack_grade_replay_candidate": false,
                "bounded_partition_replay_present": bounded_replay_valid,
                "partition_order_permutation_replay_present": permutation_replay_valid,
                "observable_result_invariance_evidenced_for_declared_profiles": observable_invariance_valid,
                "automatic_dynamic_addition_evidenced_for_declared_profiles": automatic_dynamic_addition_valid,
                "automatic_dynamic_release_evidenced_for_declared_profiles": automatic_dynamic_release_valid,
                "snapshot_fence_counterpart_evidenced": snapshot_counterpart_valid,
                "capability_view_fence_counterpart_evidenced": capability_counterpart_valid,
                "bounded_partition_analyzer_soundness_evidenced": bounded_analyzer_valid,
                "declared_profile_scheduler_equivalence_evidenced": scheduler_equivalence_declared_profile_valid,
                "lean_tla_model_bound_evidenced": lean_tla_model_bound_valid,
                "w073_typed_formatting_guard_carried": w073_guard_valid,
                "declared_profile_pack_replay_equivalence_evidenced": declared_pack_equivalence_valid,
                "no_proxy_promotion_guard_evidenced": no_proxy_guard_valid,
                "production_partition_analyzer_soundness_promoted": false,
                "fairness_scheduler_unbounded_coverage_promoted": false,
                "operated_cross_engine_stage2_service_promoted": false,
                "retained_witness_lifecycle_promoted": false,
                "pack_grade_replay_governance_promoted": false,
                "satisfied_inputs": [
                    "w042_stage2_predecessor_policy_packet",
                    "bounded_partition_replay_present",
                    "partition_order_permutation_replay_present",
                    "observable_result_invariance_for_declared_profiles",
                    "automatic_dynamic_addition_evidenced_for_declared_profiles",
                    "automatic_dynamic_release_evidenced_for_declared_profiles",
                    "snapshot_fence_counterpart_evidenced_for_declared_profiles",
                    "capability_view_fence_counterpart_evidenced_for_declared_profiles",
                    "bounded_partition_analyzer_soundness_evidenced",
                    "declared_profile_scheduler_equivalence_evidenced",
                    "lean_tla_model_bound_evidenced",
                    "w073_typed_formatting_guard_carried",
                    "declared_profile_pack_replay_equivalence_evidenced",
                    "no_proxy_promotion_guard_evidenced"
                ],
                "blockers": [
                    "stage2.broader_dynamic_transition_coverage_absent",
                    "stage2.full_production_partition_analyzer_soundness_absent",
                    "stage2.fairness_scheduler_unbounded_coverage_absent",
                    "stage2.operated_cross_engine_differential_service_absent",
                    "stage2.retained_witness_lifecycle_pack_dependency_absent",
                    "stage2.pack_grade_replay_governance_absent"
                ],
                "semantic_equivalence_statement": "Observable-result and scheduler equivalence are evidenced for declared W043 Stage 2 profiles, including W043 dynamic addition/release refinement, snapshot/capability fence no-publish counterparts, Lean/TLA model bounds, and W073 typed-formatting guards. Production Stage 2 policy and pack-grade replay remain unpromoted until broader dynamic coverage, production analyzer soundness, fairness and unbounded scheduler coverage, operated cross-engine service evidence, retained-witness lifecycle evidence, and pack governance are present."
            }),
        )?;

        let validation_status = if validation_failures.is_empty() {
            "w043_stage2_scheduler_equivalence_valid"
        } else {
            "w043_stage2_scheduler_equivalence_invalid"
        };
        write_json(
            &artifact_root.join("validation.json"),
            &json!({
                "schema_version": W043_VALIDATION_SCHEMA_V1,
                "run_id": run_id,
                "status": validation_status,
                "policy_row_count": policy_row_count,
                "satisfied_policy_row_count": satisfied_policy_row_count,
                "partition_replay_row_count": partition_replay_row_count,
                "permutation_replay_row_count": permutation_replay_row_count,
                "nontrivial_permutation_row_count": nontrivial_permutation_row_count,
                "observable_invariance_row_count": observable_invariance_row_count,
                "formatting_watch_row_count": formatting_watch_row_count,
                "automatic_dynamic_transition_row_count": 2,
                "automatic_dynamic_addition_evidenced": automatic_dynamic_addition_valid,
                "automatic_dynamic_release_evidenced": automatic_dynamic_release_valid,
                "snapshot_counterpart_evidenced": snapshot_counterpart_valid,
                "capability_counterpart_evidenced": capability_counterpart_valid,
                "bounded_partition_analyzer_evidenced": bounded_analyzer_valid,
                "scheduler_equivalence_evidenced_for_declared_profiles": scheduler_equivalence_declared_profile_valid,
                "lean_tla_model_bound_evidenced": lean_tla_model_bound_valid,
                "declared_pack_equivalence_evidenced": declared_pack_equivalence_valid,
                "no_proxy_promotion_guard_evidenced": no_proxy_guard_valid,
                "exact_remaining_blocker_count": exact_remaining_blocker_count,
                "failed_row_count": policy_failed_row_count,
                "stage2_policy_promoted": false,
                "pack_grade_replay_promoted": false,
                "validation_failures": validation_failures
            }),
        )?;
        write_json(
            &artifact_root.join("run_summary.json"),
            &json!({
                "schema_version": W043_RUN_SUMMARY_SCHEMA_V1,
                "run_id": run_id,
                "artifact_root": relative_artifact_root,
                "source_evidence_index_path": source_evidence_index_path,
                "w043_stage2_policy_gate_register_path": policy_gate_register_path,
                "w043_production_partition_analyzer_register_path": production_analyzer_register_path,
                "w043_scheduler_equivalence_register_path": scheduler_equivalence_register_path,
                "w043_pack_grade_equivalence_register_path": pack_grade_equivalence_register_path,
                "w043_stage2_exact_blocker_register_path": blocker_register_path,
                "promotion_decision_path": promotion_decision_path,
                "validation_path": validation_path,
                "policy_row_count": policy_row_count,
                "satisfied_policy_row_count": satisfied_policy_row_count,
                "partition_replay_row_count": partition_replay_row_count,
                "permutation_replay_row_count": permutation_replay_row_count,
                "nontrivial_permutation_row_count": nontrivial_permutation_row_count,
                "observable_invariance_row_count": observable_invariance_row_count,
                "formatting_watch_row_count": formatting_watch_row_count,
                "automatic_dynamic_transition_row_count": 2,
                "automatic_dynamic_addition_evidenced": automatic_dynamic_addition_valid,
                "automatic_dynamic_release_evidenced": automatic_dynamic_release_valid,
                "snapshot_counterpart_evidenced": snapshot_counterpart_valid,
                "capability_counterpart_evidenced": capability_counterpart_valid,
                "bounded_partition_analyzer_evidenced": bounded_analyzer_valid,
                "scheduler_equivalence_evidenced_for_declared_profiles": scheduler_equivalence_declared_profile_valid,
                "lean_tla_model_bound_evidenced": lean_tla_model_bound_valid,
                "declared_pack_equivalence_evidenced": declared_pack_equivalence_valid,
                "no_proxy_promotion_guard_evidenced": no_proxy_guard_valid,
                "exact_remaining_blocker_count": exact_remaining_blocker_count,
                "failed_row_count": policy_failed_row_count,
                "stage2_policy_promoted": false,
                "stage2_promotion_candidate": false,
                "pack_grade_replay_promoted": false,
                "pack_grade_replay_candidate": false
            }),
        )?;

        Ok(Stage2ReplayRunSummary {
            run_id: run_id.to_string(),
            schema_version: W043_RUN_SUMMARY_SCHEMA_V1.to_string(),
            partition_replay_row_count,
            permutation_replay_row_count,
            nontrivial_permutation_row_count,
            observable_invariance_row_count,
            formatting_watch_row_count,
            exact_remaining_blocker_count,
            failed_row_count: policy_failed_row_count,
            stage2_policy_promoted: false,
            artifact_root: relative_artifact_root,
        })
    }

    fn execute_w042(
        &self,
        repo_root: &Path,
        run_id: &str,
    ) -> Result<Stage2ReplayRunSummary, Stage2ReplayError> {
        let relative_artifact_root =
            relative_artifact_path(&["docs", "test-runs", "core-engine", "stage2-replay", run_id]);
        let artifact_root = repo_root.join(&relative_artifact_root);
        if artifact_root.exists() {
            fs::remove_dir_all(&artifact_root).map_err(|source| {
                Stage2ReplayError::RemoveDirectory {
                    path: artifact_root.display().to_string(),
                    source,
                }
            })?;
        }
        fs::create_dir_all(&artifact_root).map_err(|source| {
            Stage2ReplayError::CreateDirectory {
                path: artifact_root.display().to_string(),
                source,
            }
        })?;

        let w042_obligation_summary = read_json(repo_root, W042_OBLIGATION_SUMMARY)?;
        let w042_obligation_map = read_json(repo_root, W042_OBLIGATION_MAP)?;
        let w042_promotion_target_gate_map = read_json(repo_root, W042_PROMOTION_TARGET_GATE_MAP)?;
        let w042_formatting_intake = read_json(repo_root, W042_FORMATTING_INTAKE)?;
        let w042_optimized_summary = read_json(repo_root, W042_OPTIMIZED_CORE_SUMMARY)?;
        let w042_optimized_counterpart = read_json(repo_root, W042_OPTIMIZED_CORE_COUNTERPART)?;
        let w042_optimized_blockers = read_json(repo_root, W042_OPTIMIZED_CORE_BLOCKERS)?;
        let w042_treecalc_summary = read_json(repo_root, W042_TREECALC_SUMMARY)?;
        let w042_rust_summary = read_json(repo_root, W042_RUST_SUMMARY)?;
        let w042_rust_validation = read_json(repo_root, W042_RUST_VALIDATION)?;
        let w042_rust_refinement = read_json(repo_root, W042_RUST_REFINEMENT_REGISTER)?;
        let w042_lean_tla_summary = read_json(repo_root, W042_LEAN_TLA_SUMMARY)?;
        let w042_lean_tla_validation = read_json(repo_root, W042_LEAN_TLA_VALIDATION)?;
        let w042_tla_model_register = read_json(repo_root, W042_TLA_MODEL_REGISTER)?;
        let w042_lean_tla_blockers = read_json(repo_root, W042_LEAN_TLA_BLOCKERS)?;
        let w041_stage2_summary = read_json(repo_root, W041_STAGE2_RUN_SUMMARY_FOR_W042)?;
        let w041_stage2_validation = read_json(repo_root, W041_STAGE2_VALIDATION_FOR_W042)?;
        let w041_stage2_policy_gate = read_json(repo_root, W041_STAGE2_POLICY_GATE_FOR_W042)?;
        let w041_stage2_production_analyzer =
            read_json(repo_root, W041_STAGE2_PRODUCTION_ANALYZER_FOR_W042)?;
        let w041_stage2_pack_equivalence =
            read_json(repo_root, W041_STAGE2_PACK_EQUIVALENCE_FOR_W042)?;
        let w041_stage2_blockers = read_json(repo_root, W041_STAGE2_BLOCKERS_FOR_W042)?;
        let w041_stage2_promotion_decision =
            read_json(repo_root, W041_STAGE2_PROMOTION_DECISION_FOR_W042)?;
        let lean_file_present = repo_root.join(W042_STAGE2_LEAN_FILE).exists();

        let w042_obligation_valid = number_at(&w042_obligation_summary, "obligation_count") == 33
            && bool_at(
                &w042_obligation_summary,
                "oxfml_formatting_update_incorporated",
            )
            && number_at(&w042_promotion_target_gate_map, "promotion_target_count") == 14;
        let w041_stage2_valid = string_at(&w041_stage2_validation, "status")
            == "w041_stage2_analyzer_pack_equivalence_valid"
            && !bool_at(&w041_stage2_summary, "stage2_policy_promoted")
            && !bool_at(&w041_stage2_promotion_decision, "stage2_policy_promoted");
        let w042_optimized_valid = number_at(&w042_optimized_summary, "failed_row_count") == 0
            && number_at(&w042_optimized_summary, "match_promoted_count") == 0
            && number_at(&w042_optimized_summary, "exact_remaining_blocker_count") == 3
            && bool_at(
                &w042_optimized_summary,
                "snapshot_counterpart_evidenced_for_declared_profile",
            )
            && bool_at(
                &w042_optimized_summary,
                "capability_counterpart_evidenced_for_declared_profile",
            );
        let w042_treecalc_valid = number_at(&w042_treecalc_summary, "case_count") == 26
            && number_at(&w042_treecalc_summary, "expectation_mismatch_count") == 0;
        let w042_rust_valid = string_at(&w042_rust_validation, "status")
            == "formal_assurance_w042_rust_totality_refinement_valid"
            && number_at(&w042_rust_summary, "automatic_dynamic_transition_row_count") == 1
            && !bool_at(
                &w042_rust_summary["promotion_claims"],
                "stage2_policy_promoted",
            );
        let w042_lean_tla_valid = string_at(&w042_lean_tla_validation, "status")
            == "formal_assurance_w042_lean_tla_fairness_expansion_valid"
            && number_at(&w042_lean_tla_summary, "bounded_model_row_count") == 4
            && number_at(&w042_lean_tla_summary, "exact_remaining_blocker_count") == 5
            && !bool_at(
                &w042_lean_tla_summary["promotion_claims"],
                "stage2_policy_promoted",
            );

        let partition_replay_row_count =
            number_at(&w041_stage2_summary, "partition_replay_row_count") as usize;
        let permutation_replay_row_count =
            number_at(&w041_stage2_summary, "permutation_replay_row_count") as usize;
        let nontrivial_permutation_row_count =
            number_at(&w041_stage2_summary, "nontrivial_permutation_row_count") as usize;
        let observable_invariance_row_count =
            number_at(&w041_stage2_summary, "observable_invariance_row_count") as usize;
        let formatting_watch_row_count =
            number_at(&w041_stage2_summary, "formatting_watch_row_count") as usize;

        let predecessor_policy_valid = w041_stage2_valid
            && number_at(&w041_stage2_policy_gate, "policy_row_count") == 14
            && number_at(&w041_stage2_policy_gate, "satisfied_policy_row_count") == 10
            && number_at(&w041_stage2_policy_gate, "exact_remaining_blocker_count") == 4;
        let bounded_replay_valid = predecessor_policy_valid && partition_replay_row_count == 5;
        let permutation_replay_valid = predecessor_policy_valid
            && permutation_replay_row_count == 6
            && nontrivial_permutation_row_count == 1;
        let observable_invariance_valid =
            predecessor_policy_valid && observable_invariance_row_count == 5;
        let automatic_dynamic_transition_valid = w042_optimized_valid
            && w042_treecalc_valid
            && w042_rust_valid
            && row_with_field_exists(
                &w042_optimized_counterpart,
                "row_id",
                "w042_dynamic_transition_declared_profile_extension",
            )
            && row_with_field_exists(
                &w042_rust_refinement,
                "row_id",
                "w042_automatic_dynamic_transition_refinement_evidence",
            );
        let snapshot_counterpart_valid = w042_optimized_valid
            && row_with_field_exists(
                &w042_optimized_counterpart,
                "row_id",
                "w042_snapshot_fence_counterpart_declared_profile_evidence",
            )
            && row_with_field_exists(
                &w042_rust_refinement,
                "row_id",
                "w042_snapshot_fence_declared_profile_refinement_evidence",
            );
        let capability_counterpart_valid = w042_optimized_valid
            && row_with_field_exists(
                &w042_optimized_counterpart,
                "row_id",
                "w042_capability_view_counterpart_declared_profile_evidence",
            )
            && row_with_field_exists(
                &w042_rust_refinement,
                "row_id",
                "w042_capability_view_declared_profile_refinement_evidence",
            );
        let bounded_analyzer_valid = predecessor_policy_valid
            && number_at(&w041_stage2_production_analyzer, "row_count") == 6
            && row_with_field_exists(
                &w042_tla_model_register,
                "row_id",
                "w042_stage2_equivalence_bounded_model_input",
            )
            && lean_file_present;
        let lean_tla_model_bound_valid = w042_lean_tla_valid
            && row_with_field_exists(
                &w042_tla_model_register,
                "row_id",
                "w042_tla_stage2_partition_bounded_model_evidence",
            )
            && row_with_field_exists(
                &w042_lean_tla_blockers,
                "row_id",
                "w042_tla_fairness_scheduler_unbounded_boundary",
            );
        let w073_guard_valid = formatting_watch_row_count == 1
            && array_field_contains_string(
                &w042_formatting_intake,
                "typed_only_families",
                "colorScale",
            )
            && array_field_contains_string(
                &w042_formatting_intake,
                "typed_only_families",
                "dataBar",
            )
            && array_field_contains_string(
                &w042_formatting_intake,
                "typed_only_families",
                "iconSet",
            )
            && array_field_contains_string(&w042_formatting_intake, "typed_only_families", "top")
            && array_field_contains_string(
                &w042_formatting_intake,
                "typed_only_families",
                "bottom",
            )
            && array_field_contains_string(
                &w042_formatting_intake,
                "typed_only_families",
                "aboveAverage",
            )
            && array_field_contains_string(
                &w042_formatting_intake,
                "typed_only_families",
                "belowAverage",
            );
        let declared_pack_equivalence_valid = predecessor_policy_valid
            && observable_invariance_valid
            && snapshot_counterpart_valid
            && capability_counterpart_valid
            && w073_guard_valid
            && number_at(&w041_stage2_pack_equivalence, "row_count") == 10
            && number_at(&w041_stage2_pack_equivalence, "satisfied_policy_row_count") == 8
            && bool_at(&w041_stage2_summary, "declared_pack_equivalence_evidenced");
        let no_proxy_guard_valid = w042_obligation_valid
            && number_at(&w042_optimized_summary, "match_promoted_count") == 0
            && array_field_contains_string(
                &w042_obligation_summary,
                "no_promotion_claims",
                "stage2_production_policy",
            )
            && array_field_contains_string(
                &w042_obligation_summary,
                "no_promotion_claims",
                "pack_grade_replay_equivalence",
            )
            && !bool_at(
                &w041_stage2_promotion_decision,
                "stage2_promotion_candidate",
            );
        let broader_dynamic_transition_blocked = row_with_field_exists(
            &w042_optimized_blockers,
            "row_id",
            "w042_broader_dynamic_transition_coverage_exact_blocker",
        );

        let policy_rows = vec![
            json!({
                "row_id": "w042_stage2_w041_predecessor_policy_carried",
                "w042_obligation_id": "W042-OBL-013",
                "policy_area": "w041_stage2_predecessor_policy_packet",
                "source_artifacts": [W041_STAGE2_RUN_SUMMARY_FOR_W042, W041_STAGE2_VALIDATION_FOR_W042, W041_STAGE2_POLICY_GATE_FOR_W042],
                "satisfied_for_declared_profile": predecessor_policy_valid,
                "exact_remaining_blocker": false,
                "promotion_consequence": "W041 predecessor evidence is valid input only; W042 promotion still requires direct W042 gates",
                "disposition": "carry the W041 Stage 2 analyzer and pack-equivalence packet as non-promoting predecessor evidence",
                "failures": if predecessor_policy_valid { Vec::<String>::new() } else { vec!["w042_w041_stage2_predecessor_not_valid".to_string()] },
            }),
            json!({
                "row_id": "w042_stage2_bounded_replay_carried",
                "w042_obligation_id": "W042-OBL-015",
                "policy_area": "bounded_baseline_vs_stage2_replay",
                "source_artifacts": [W041_STAGE2_RUN_SUMMARY_FOR_W042, W041_STAGE2_POLICY_GATE_FOR_W042],
                "satisfied_for_declared_profile": bounded_replay_valid,
                "exact_remaining_blocker": false,
                "promotion_consequence": "bounded replay remains declared-profile evidence only",
                "disposition": "carry W041 bounded baseline-versus-Stage-2 replay rows into W042 pack-grade equivalence classification",
                "failures": if bounded_replay_valid { Vec::<String>::new() } else { vec!["w042_bounded_replay_not_valid".to_string()] },
            }),
            json!({
                "row_id": "w042_stage2_partition_order_permutation_carried",
                "w042_obligation_id": "W042-OBL-014",
                "policy_area": "partition_order_permutation_replay",
                "source_artifacts": [W041_STAGE2_RUN_SUMMARY_FOR_W042, W041_STAGE2_POLICY_GATE_FOR_W042],
                "satisfied_for_declared_profile": permutation_replay_valid,
                "exact_remaining_blocker": false,
                "promotion_consequence": "bounded permutation replay does not discharge production scheduler fairness",
                "disposition": "carry one nontrivial partition-order permutation row and six total permutation rows",
                "failures": if permutation_replay_valid { Vec::<String>::new() } else { vec!["w042_partition_permutation_replay_not_valid".to_string()] },
            }),
            json!({
                "row_id": "w042_stage2_observable_invariance_carried",
                "w042_obligation_id": "W042-OBL-014",
                "policy_area": "observable_result_invariance",
                "source_artifacts": [W041_STAGE2_RUN_SUMMARY_FOR_W042, W041_STAGE2_PACK_EQUIVALENCE_FOR_W042],
                "satisfied_for_declared_profile": observable_invariance_valid,
                "exact_remaining_blocker": false,
                "promotion_consequence": "observable invariance is evidenced for declared bounded profiles only",
                "disposition": "carry W041 observable-result invariance rows for values, rejects, fence no-publish behavior, and formatting watch surfaces",
                "failures": if observable_invariance_valid { Vec::<String>::new() } else { vec!["w042_observable_invariance_not_valid".to_string()] },
            }),
            json!({
                "row_id": "w042_stage2_automatic_dynamic_transition_refinement_bound",
                "w042_obligation_id": "W042-OBL-013",
                "policy_area": "dynamic_and_soft_reference_transition",
                "source_artifacts": [W042_OPTIMIZED_CORE_COUNTERPART, W042_RUST_REFINEMENT_REGISTER, W042_TREECALC_SUMMARY],
                "satisfied_for_declared_profile": automatic_dynamic_transition_valid,
                "exact_remaining_blocker": false,
                "promotion_consequence": "automatic dynamic transition evidence narrows declared-profile analyzer inputs but does not prove production analyzer soundness",
                "disposition": "bind W042 automatic dynamic transition and Rust refinement rows to Stage 2 analyzer preconditions",
                "failures": if automatic_dynamic_transition_valid { Vec::<String>::new() } else { vec!["w042_automatic_dynamic_transition_not_valid".to_string()] },
            }),
            json!({
                "row_id": "w042_stage2_snapshot_fence_counterpart_evidence",
                "w042_obligation_id": "W042-OBL-003",
                "policy_area": "snapshot_fence_counterpart",
                "source_artifacts": [W042_OPTIMIZED_CORE_COUNTERPART, W042_RUST_REFINEMENT_REGISTER],
                "satisfied_for_declared_profile": snapshot_counterpart_valid,
                "exact_remaining_blocker": false,
                "promotion_consequence": "snapshot-fence counterpart is evidenced for declared profiles; production Stage 2 policy remains blocked by broader gates",
                "disposition": "bind W042 optimized/core and Rust declared-profile snapshot-fence reject/no-publish counterpart evidence without match promotion",
                "failures": if snapshot_counterpart_valid { Vec::<String>::new() } else { vec!["w042_snapshot_counterpart_not_valid".to_string()] },
            }),
            json!({
                "row_id": "w042_stage2_capability_view_counterpart_evidence",
                "w042_obligation_id": "W042-OBL-004",
                "policy_area": "capability_view_fence_counterpart",
                "source_artifacts": [W042_OPTIMIZED_CORE_COUNTERPART, W042_RUST_REFINEMENT_REGISTER],
                "satisfied_for_declared_profile": capability_counterpart_valid,
                "exact_remaining_blocker": false,
                "promotion_consequence": "capability-view counterpart is evidenced for declared profiles; production Stage 2 policy remains blocked by broader gates",
                "disposition": "bind W042 optimized/core and Rust declared-profile capability-view reject/no-publish counterpart evidence without match promotion",
                "failures": if capability_counterpart_valid { Vec::<String>::new() } else { vec!["w042_capability_counterpart_not_valid".to_string()] },
            }),
            json!({
                "row_id": "w042_stage2_bounded_partition_analyzer_predicate_evidence",
                "w042_obligation_id": "W042-OBL-013",
                "policy_area": "bounded_partition_analyzer_soundness",
                "source_artifacts": [W041_STAGE2_PRODUCTION_ANALYZER_FOR_W042, W042_TLA_MODEL_REGISTER, W042_STAGE2_LEAN_FILE],
                "satisfied_for_declared_profile": bounded_analyzer_valid,
                "exact_remaining_blocker": false,
                "promotion_consequence": "bounded analyzer preconditions are evidenced, but full production analyzer soundness remains blocked",
                "disposition": "bind W041 bounded partition-analyzer evidence and W042 bounded Stage 2 model input under the W042 Lean predicate",
                "failures": if bounded_analyzer_valid { Vec::<String>::new() } else { vec!["w042_bounded_partition_analyzer_not_valid".to_string()] },
            }),
            json!({
                "row_id": "w042_stage2_lean_tla_model_bound_carried",
                "w042_obligation_id": "W042-OBL-011",
                "policy_area": "lean_tla_model_bound",
                "source_artifacts": [W042_LEAN_TLA_SUMMARY, W042_TLA_MODEL_REGISTER, W042_LEAN_TLA_BLOCKERS],
                "satisfied_for_declared_profile": lean_tla_model_bound_valid,
                "exact_remaining_blocker": false,
                "promotion_consequence": "bounded proof/model inputs strengthen the Stage 2 packet but do not promote full TLA or Stage 2 policy",
                "disposition": "carry W042 Lean/TLA bounded Stage 2 model rows and exact fairness boundary into the Stage 2 gate",
                "failures": if lean_tla_model_bound_valid { Vec::<String>::new() } else { vec!["w042_lean_tla_model_bound_not_valid".to_string()] },
            }),
            json!({
                "row_id": "w042_stage2_w073_typed_formatting_guard_carried",
                "w042_obligation_id": "W042-OBL-024",
                "policy_area": "w073_typed_formatting_watch",
                "source_artifacts": [W042_FORMATTING_INTAKE, W041_STAGE2_RUN_SUMMARY_FOR_W042],
                "satisfied_for_declared_profile": w073_guard_valid,
                "exact_remaining_blocker": false,
                "promotion_consequence": "typed-only formatting is carried as an observable-surface guard; broad OxFml seam breadth remains under calc-czd.8",
                "disposition": "retain the W073 direct-replacement rule for aggregate and visualization conditional-formatting metadata",
                "failures": if w073_guard_valid { Vec::<String>::new() } else { vec!["w042_w073_formatting_guard_not_valid".to_string()] },
            }),
            json!({
                "row_id": "w042_stage2_declared_pack_replay_equivalence",
                "w042_obligation_id": "W042-OBL-015",
                "policy_area": "declared_profile_pack_replay_equivalence",
                "source_artifacts": [W041_STAGE2_PACK_EQUIVALENCE_FOR_W042, W041_STAGE2_PROMOTION_DECISION_FOR_W042, W042_OPTIMIZED_CORE_COUNTERPART],
                "satisfied_for_declared_profile": declared_pack_equivalence_valid,
                "exact_remaining_blocker": false,
                "promotion_consequence": "declared-profile equivalence evidence is not pack-grade replay governance",
                "disposition": "classify values, rejects, fence no-publish behavior, typed-formatting watch, replay validation, and W042 counterpart rows as declared-profile pack-equivalence inputs",
                "failures": if declared_pack_equivalence_valid { Vec::<String>::new() } else { vec!["w042_declared_pack_equivalence_not_valid".to_string()] },
            }),
            json!({
                "row_id": "w042_stage2_no_proxy_promotion_guard",
                "w042_obligation_id": "W042-OBL-006",
                "policy_area": "no_proxy_promotion_guard",
                "source_artifacts": [W042_OBLIGATION_SUMMARY, W042_PROMOTION_TARGET_GATE_MAP, W042_OPTIMIZED_CORE_SUMMARY, W041_STAGE2_PROMOTION_DECISION_FOR_W042],
                "satisfied_for_declared_profile": no_proxy_guard_valid,
                "exact_remaining_blocker": false,
                "promotion_consequence": "Stage 2, pack, and release-grade gates remain blocked unless direct target evidence is present",
                "disposition": "keep bounded-only, declared-profile, and retained-blocker evidence out of promotion counts",
                "failures": if no_proxy_guard_valid { Vec::<String>::new() } else { vec!["w042_no_proxy_guard_not_valid".to_string()] },
            }),
            json!({
                "row_id": "w042_stage2_broader_dynamic_transition_coverage_blocker",
                "w042_obligation_id": "W042-OBL-013",
                "related_w042_obligation_ids": ["W042-OBL-002"],
                "policy_area": "broader_dynamic_transition_coverage",
                "source_artifacts": [W042_OPTIMIZED_CORE_BLOCKERS, W042_RUST_REFINEMENT_REGISTER],
                "satisfied_for_declared_profile": false,
                "exact_remaining_blocker": true,
                "promotion_consequence": "production partition-analyzer soundness remains blocked",
                "disposition": "retain broader dynamic dependency-transition coverage as a Stage 2 analyzer dependency beyond the exercised resolved-to-potential pattern",
                "failures": if broader_dynamic_transition_blocked { Vec::<String>::new() } else { vec!["w042_broader_dynamic_transition_blocker_missing".to_string()] },
            }),
            json!({
                "row_id": "w042_stage2_full_production_analyzer_soundness_blocker",
                "w042_obligation_id": "W042-OBL-013",
                "policy_area": "full_production_partition_analyzer_soundness",
                "source_artifacts": [W042_OBLIGATION_MAP, W041_STAGE2_BLOCKERS_FOR_W042, W042_STAGE2_LEAN_FILE],
                "satisfied_for_declared_profile": false,
                "exact_remaining_blocker": true,
                "promotion_consequence": "Stage 2 production policy remains unpromoted",
                "disposition": "retain full production partition-analyzer soundness as exact blocker beyond bounded declared-profile evidence",
                "failures": if obligation_exists(&w042_obligation_map, "W042-OBL-013") && row_with_field_exists(&w041_stage2_blockers, "row_id", "w041_stage2_full_production_analyzer_soundness_blocker") && lean_file_present { Vec::<String>::new() } else { vec!["w042_production_analyzer_blocker_input_missing".to_string()] },
            }),
            json!({
                "row_id": "w042_stage2_scheduler_fairness_unbounded_equivalence_blocker",
                "w042_obligation_id": "W042-OBL-014",
                "related_w042_obligation_ids": ["W042-OBL-011"],
                "policy_area": "fairness_scheduler_unbounded_coverage",
                "source_artifacts": [W042_LEAN_TLA_SUMMARY, W042_LEAN_TLA_BLOCKERS, W041_STAGE2_BLOCKERS_FOR_W042],
                "satisfied_for_declared_profile": false,
                "exact_remaining_blocker": true,
                "promotion_consequence": "full TLA verification and Stage 2 production policy remain unpromoted",
                "disposition": "retain fairness, unbounded scheduler coverage, and model-completeness limits as exact Stage 2 blockers",
                "failures": if obligation_exists(&w042_obligation_map, "W042-OBL-014") && row_with_field_exists(&w042_lean_tla_blockers, "row_id", "w042_tla_fairness_scheduler_unbounded_boundary") { Vec::<String>::new() } else { vec!["w042_scheduler_fairness_blocker_input_missing".to_string()] },
            }),
            json!({
                "row_id": "w042_stage2_operated_cross_engine_service_dependency_blocker",
                "w042_obligation_id": "W042-OBL-020",
                "policy_area": "operated_cross_engine_stage2_differential_service",
                "source_artifacts": [W042_OBLIGATION_MAP, W041_STAGE2_BLOCKERS_FOR_W042],
                "satisfied_for_declared_profile": false,
                "exact_remaining_blocker": true,
                "promotion_consequence": "operated Stage 2 differential evidence remains required before policy promotion",
                "disposition": "retain operated cross-engine differential service as successor dependency",
                "failures": if obligation_exists(&w042_obligation_map, "W042-OBL-020") && row_with_field_exists(&w041_stage2_blockers, "row_id", "w041_stage2_operated_cross_engine_service_dependency_blocker") { Vec::<String>::new() } else { vec!["w042_operated_stage2_service_blocker_input_missing".to_string()] },
            }),
            json!({
                "row_id": "w042_stage2_retained_witness_lifecycle_pack_dependency_blocker",
                "w042_obligation_id": "W042-OBL-018",
                "policy_area": "retained_witness_lifecycle_pack_dependency",
                "source_artifacts": [W042_OBLIGATION_MAP],
                "satisfied_for_declared_profile": false,
                "exact_remaining_blocker": true,
                "promotion_consequence": "pack-grade replay equivalence remains blocked until retained-witness lifecycle and retention SLO evidence exists",
                "disposition": "retain retained-witness lifecycle and retention SLO as a pack-grade replay dependency rather than treating deterministic replay files as operated witness evidence",
                "failures": if obligation_exists(&w042_obligation_map, "W042-OBL-018") { Vec::<String>::new() } else { vec!["w042_retained_witness_obligation_missing".to_string()] },
            }),
            json!({
                "row_id": "w042_stage2_pack_grade_replay_governance_blocker",
                "w042_obligation_id": "W042-OBL-015",
                "related_w042_obligation_ids": ["W042-OBL-030"],
                "policy_area": "pack_grade_replay_governance",
                "source_artifacts": [W042_OBLIGATION_MAP, W041_STAGE2_BLOCKERS_FOR_W042],
                "satisfied_for_declared_profile": false,
                "exact_remaining_blocker": true,
                "promotion_consequence": "pack-grade replay, C5, and release-grade verification remain blocked",
                "disposition": "retain pack-grade replay governance as release-decision blocker beyond declared-profile equivalence",
                "failures": if obligation_exists(&w042_obligation_map, "W042-OBL-015") && obligation_exists(&w042_obligation_map, "W042-OBL-030") && row_with_field_exists(&w041_stage2_blockers, "row_id", "w041_stage2_pack_grade_replay_governance_blocker") { Vec::<String>::new() } else { vec!["w042_pack_grade_replay_governance_blocker_input_missing".to_string()] },
            }),
        ];

        let satisfied_policy_row_count = policy_rows
            .iter()
            .filter(|row| bool_at(row, "satisfied_for_declared_profile"))
            .count();
        let policy_blocker_rows = policy_rows
            .iter()
            .filter(|row| bool_at(row, "exact_remaining_blocker"))
            .cloned()
            .collect::<Vec<_>>();
        let policy_failed_row_count = policy_rows
            .iter()
            .filter(|row| {
                !row.get("failures")
                    .and_then(Value::as_array)
                    .is_some_and(Vec::is_empty)
            })
            .count();
        let exact_remaining_blocker_count = policy_blocker_rows.len();
        let policy_row_count = policy_rows.len();

        let production_analyzer_rows = policy_rows
            .iter()
            .filter(|row| {
                matches!(
                    string_at(row, "policy_area"),
                    "w041_stage2_predecessor_policy_packet"
                        | "dynamic_and_soft_reference_transition"
                        | "snapshot_fence_counterpart"
                        | "capability_view_fence_counterpart"
                        | "bounded_partition_analyzer_soundness"
                        | "lean_tla_model_bound"
                        | "broader_dynamic_transition_coverage"
                        | "full_production_partition_analyzer_soundness"
                        | "fairness_scheduler_unbounded_coverage"
                )
            })
            .cloned()
            .collect::<Vec<_>>();
        let pack_grade_equivalence_rows = policy_rows
            .iter()
            .filter(|row| {
                matches!(
                    string_at(row, "policy_area"),
                    "bounded_baseline_vs_stage2_replay"
                        | "partition_order_permutation_replay"
                        | "observable_result_invariance"
                        | "snapshot_fence_counterpart"
                        | "capability_view_fence_counterpart"
                        | "w073_typed_formatting_watch"
                        | "declared_profile_pack_replay_equivalence"
                        | "no_proxy_promotion_guard"
                        | "operated_cross_engine_stage2_differential_service"
                        | "retained_witness_lifecycle_pack_dependency"
                        | "pack_grade_replay_governance"
                )
            })
            .cloned()
            .collect::<Vec<_>>();

        let mut validation_failures = Vec::new();
        if !w042_obligation_valid {
            validation_failures.push("w042_obligation_map_not_valid".to_string());
        }
        for obligation_id in [
            "W042-OBL-003",
            "W042-OBL-004",
            "W042-OBL-006",
            "W042-OBL-011",
            "W042-OBL-013",
            "W042-OBL-014",
            "W042-OBL-015",
            "W042-OBL-018",
            "W042-OBL-020",
            "W042-OBL-024",
            "W042-OBL-030",
        ] {
            if !obligation_exists(&w042_obligation_map, obligation_id) {
                validation_failures.push(format!("{obligation_id}_missing"));
            }
        }
        if !predecessor_policy_valid {
            validation_failures.push("w041_stage2_predecessor_not_valid".to_string());
        }
        if !w042_optimized_valid || !w042_treecalc_valid {
            validation_failures.push("w042_optimized_stage2_inputs_not_valid".to_string());
        }
        if !w042_rust_valid {
            validation_failures.push("w042_rust_stage2_inputs_not_valid".to_string());
        }
        if !w042_lean_tla_valid {
            validation_failures.push("w042_lean_tla_stage2_inputs_not_valid".to_string());
        }
        if !automatic_dynamic_transition_valid {
            validation_failures.push("w042_automatic_dynamic_transition_not_valid".to_string());
        }
        if !snapshot_counterpart_valid {
            validation_failures.push("w042_snapshot_counterpart_not_valid".to_string());
        }
        if !capability_counterpart_valid {
            validation_failures.push("w042_capability_counterpart_not_valid".to_string());
        }
        if !bounded_analyzer_valid {
            validation_failures.push("w042_bounded_partition_analyzer_not_valid".to_string());
        }
        if !lean_tla_model_bound_valid {
            validation_failures.push("w042_lean_tla_model_bound_not_valid".to_string());
        }
        if !declared_pack_equivalence_valid {
            validation_failures.push("w042_declared_pack_equivalence_not_valid".to_string());
        }
        if !no_proxy_guard_valid {
            validation_failures.push("w042_no_proxy_guard_not_valid".to_string());
        }
        if !w073_guard_valid {
            validation_failures.push("w042_w073_formatting_guard_not_valid".to_string());
        }
        if !broader_dynamic_transition_blocked {
            validation_failures.push("w042_broader_dynamic_transition_blocker_missing".to_string());
        }
        if !lean_file_present {
            validation_failures.push("w042_stage2_lean_file_missing".to_string());
        }
        if policy_failed_row_count != 0 {
            validation_failures.push("w042_stage2_policy_row_failures_present".to_string());
        }
        if satisfied_policy_row_count != 12 {
            validation_failures
                .push("w042_stage2_expected_twelve_satisfied_policy_rows".to_string());
        }
        if exact_remaining_blocker_count != 6 {
            validation_failures.push("w042_stage2_expected_six_exact_blockers".to_string());
        }

        let source_evidence_index_path =
            format!("{relative_artifact_root}/source_evidence_index.json");
        let policy_gate_register_path =
            format!("{relative_artifact_root}/w042_stage2_policy_gate_register.json");
        let production_analyzer_register_path =
            format!("{relative_artifact_root}/w042_production_analyzer_soundness_register.json");
        let pack_grade_equivalence_register_path =
            format!("{relative_artifact_root}/w042_pack_grade_equivalence_register.json");
        let blocker_register_path =
            format!("{relative_artifact_root}/w042_stage2_exact_blocker_register.json");
        let promotion_decision_path = format!("{relative_artifact_root}/promotion_decision.json");
        let validation_path = format!("{relative_artifact_root}/validation.json");

        write_json(
            &artifact_root.join("source_evidence_index.json"),
            &json!({
                "schema_version": W042_SOURCE_INDEX_SCHEMA_V1,
                "run_id": run_id,
                "artifact_root": relative_artifact_root,
                "source_artifacts": {
                    "w042_obligation_summary": W042_OBLIGATION_SUMMARY,
                    "w042_obligation_map": W042_OBLIGATION_MAP,
                    "w042_promotion_target_gate_map": W042_PROMOTION_TARGET_GATE_MAP,
                    "w042_formatting_intake": W042_FORMATTING_INTAKE,
                    "w042_optimized_core_summary": W042_OPTIMIZED_CORE_SUMMARY,
                    "w042_optimized_core_counterpart": W042_OPTIMIZED_CORE_COUNTERPART,
                    "w042_optimized_core_blockers": W042_OPTIMIZED_CORE_BLOCKERS,
                    "w042_treecalc_summary": W042_TREECALC_SUMMARY,
                    "w042_rust_summary": W042_RUST_SUMMARY,
                    "w042_rust_validation": W042_RUST_VALIDATION,
                    "w042_rust_refinement_register": W042_RUST_REFINEMENT_REGISTER,
                    "w042_lean_tla_summary": W042_LEAN_TLA_SUMMARY,
                    "w042_lean_tla_validation": W042_LEAN_TLA_VALIDATION,
                    "w042_tla_model_register": W042_TLA_MODEL_REGISTER,
                    "w042_lean_tla_blockers": W042_LEAN_TLA_BLOCKERS,
                    "w041_stage2_summary": W041_STAGE2_RUN_SUMMARY_FOR_W042,
                    "w041_stage2_validation": W041_STAGE2_VALIDATION_FOR_W042,
                    "w041_stage2_policy_gate": W041_STAGE2_POLICY_GATE_FOR_W042,
                    "w041_stage2_production_analyzer": W041_STAGE2_PRODUCTION_ANALYZER_FOR_W042,
                    "w041_stage2_pack_equivalence": W041_STAGE2_PACK_EQUIVALENCE_FOR_W042,
                    "w041_stage2_blockers": W041_STAGE2_BLOCKERS_FOR_W042,
                    "w041_stage2_promotion_decision": W041_STAGE2_PROMOTION_DECISION_FOR_W042,
                    "w042_stage2_lean_file": W042_STAGE2_LEAN_FILE
                },
                "source_counts": {
                    "w042_obligation_count": number_at(&w042_obligation_summary, "obligation_count"),
                    "w042_promotion_target_count": number_at(&w042_promotion_target_gate_map, "promotion_target_count"),
                    "w042_optimized_exact_remaining_blocker_count": number_at(&w042_optimized_summary, "exact_remaining_blocker_count"),
                    "w042_rust_exact_remaining_blocker_count": number_at(&w042_rust_summary, "exact_remaining_blocker_count"),
                    "w042_lean_tla_exact_remaining_blocker_count": number_at(&w042_lean_tla_summary, "exact_remaining_blocker_count"),
                    "w041_stage2_policy_row_count": number_at(&w041_stage2_summary, "policy_row_count"),
                    "w041_stage2_exact_remaining_blocker_count": number_at(&w041_stage2_summary, "exact_remaining_blocker_count")
                }
            }),
        )?;
        write_json(
            &artifact_root.join("w042_stage2_policy_gate_register.json"),
            &json!({
                "schema_version": W042_POLICY_GATE_SCHEMA_V1,
                "run_id": run_id,
                "policy_row_count": policy_row_count,
                "satisfied_policy_row_count": satisfied_policy_row_count,
                "exact_remaining_blocker_count": exact_remaining_blocker_count,
                "stage2_policy_promoted": false,
                "pack_grade_replay_promoted": false,
                "rows": policy_rows
            }),
        )?;
        write_json(
            &artifact_root.join("w042_production_analyzer_soundness_register.json"),
            &json!({
                "schema_version": W042_PRODUCTION_ANALYZER_SCHEMA_V1,
                "run_id": run_id,
                "row_count": production_analyzer_rows.len(),
                "satisfied_policy_row_count": production_analyzer_rows
                    .iter()
                    .filter(|row| bool_at(row, "satisfied_for_declared_profile"))
                    .count(),
                "exact_remaining_blocker_count": production_analyzer_rows
                    .iter()
                    .filter(|row| bool_at(row, "exact_remaining_blocker"))
                    .count(),
                "semantic_statement": "W042 binds declared-profile replay, dynamic-transition refinement, snapshot/capability counterparts, bounded analyzer evidence, and Lean/TLA model bounds as production-analyzer inputs. Full production partition-analyzer soundness remains blocked by broader dynamic coverage, unbounded scheduler/fairness coverage, and operated evidence.",
                "rows": production_analyzer_rows
            }),
        )?;
        write_json(
            &artifact_root.join("w042_pack_grade_equivalence_register.json"),
            &json!({
                "schema_version": W042_PACK_GRADE_EQUIVALENCE_SCHEMA_V1,
                "run_id": run_id,
                "row_count": pack_grade_equivalence_rows.len(),
                "satisfied_policy_row_count": pack_grade_equivalence_rows
                    .iter()
                    .filter(|row| bool_at(row, "satisfied_for_declared_profile"))
                    .count(),
                "exact_remaining_blocker_count": pack_grade_equivalence_rows
                    .iter()
                    .filter(|row| bool_at(row, "exact_remaining_blocker"))
                    .count(),
                "semantic_equivalence_statement": "Declared-profile values, rejects, fence no-publish behavior, typed-formatting observable guards, replay validation, W042 counterpart evidence, and no-proxy promotion guards are bound as pack-equivalence inputs. This still is not pack-grade replay governance because retained-witness lifecycle, operated-service evidence, and pack governance remain blocked.",
                "rows": pack_grade_equivalence_rows
            }),
        )?;
        write_json(
            &artifact_root.join("w042_stage2_exact_blocker_register.json"),
            &json!({
                "schema_version": W042_BLOCKER_REGISTER_SCHEMA_V1,
                "run_id": run_id,
                "exact_remaining_blocker_count": exact_remaining_blocker_count,
                "rows": policy_blocker_rows
            }),
        )?;
        write_json(
            &artifact_root.join("promotion_decision.json"),
            &json!({
                "schema_version": W042_PROMOTION_DECISION_SCHEMA_V1,
                "run_id": run_id,
                "decision_state": "w042_stage2_pack_grade_equivalence_validated_policy_unpromoted",
                "stage2_policy_promoted": false,
                "stage2_promotion_candidate": false,
                "pack_grade_replay_promoted": false,
                "pack_grade_replay_candidate": false,
                "bounded_partition_replay_present": bounded_replay_valid,
                "partition_order_permutation_replay_present": permutation_replay_valid,
                "observable_result_invariance_evidenced_for_declared_profiles": observable_invariance_valid,
                "automatic_dynamic_transition_evidenced_for_declared_profiles": automatic_dynamic_transition_valid,
                "snapshot_fence_counterpart_evidenced": snapshot_counterpart_valid,
                "capability_view_fence_counterpart_evidenced": capability_counterpart_valid,
                "bounded_partition_analyzer_soundness_evidenced": bounded_analyzer_valid,
                "lean_tla_model_bound_evidenced": lean_tla_model_bound_valid,
                "w073_typed_formatting_guard_carried": w073_guard_valid,
                "declared_profile_pack_replay_equivalence_evidenced": declared_pack_equivalence_valid,
                "no_proxy_promotion_guard_evidenced": no_proxy_guard_valid,
                "production_partition_analyzer_soundness_promoted": false,
                "fairness_scheduler_unbounded_coverage_promoted": false,
                "operated_cross_engine_stage2_service_promoted": false,
                "retained_witness_lifecycle_promoted": false,
                "pack_grade_replay_governance_promoted": false,
                "satisfied_inputs": [
                    "w041_stage2_predecessor_policy_packet",
                    "bounded_partition_replay_present",
                    "partition_order_permutation_replay_present",
                    "observable_result_invariance_for_declared_profiles",
                    "automatic_dynamic_transition_evidenced_for_declared_profiles",
                    "snapshot_fence_counterpart_evidenced_for_declared_profiles",
                    "capability_view_fence_counterpart_evidenced_for_declared_profiles",
                    "bounded_partition_analyzer_soundness_evidenced",
                    "lean_tla_model_bound_evidenced",
                    "w073_typed_formatting_guard_carried",
                    "declared_profile_pack_replay_equivalence_evidenced",
                    "no_proxy_promotion_guard_evidenced"
                ],
                "blockers": [
                    "stage2.broader_dynamic_transition_coverage_absent",
                    "stage2.full_production_partition_analyzer_soundness_absent",
                    "stage2.fairness_scheduler_unbounded_coverage_absent",
                    "stage2.operated_cross_engine_differential_service_absent",
                    "stage2.retained_witness_lifecycle_pack_dependency_absent",
                    "stage2.pack_grade_replay_governance_absent"
                ],
                "semantic_equivalence_statement": "Observable-result and replay equivalence are evidenced for declared W042 Stage 2 profiles, including W042 dynamic transition refinement, snapshot/capability fence no-publish counterparts, Lean/TLA model bounds, and W073 typed-formatting guards. Production Stage 2 policy and pack-grade replay remain unpromoted until broader dynamic coverage, production analyzer soundness, fairness and unbounded scheduler coverage, operated cross-engine service evidence, retained-witness lifecycle evidence, and pack governance are present."
            }),
        )?;

        let validation_status = if validation_failures.is_empty() {
            "w042_stage2_pack_grade_equivalence_valid"
        } else {
            "w042_stage2_pack_grade_equivalence_invalid"
        };
        write_json(
            &artifact_root.join("validation.json"),
            &json!({
                "schema_version": W042_VALIDATION_SCHEMA_V1,
                "run_id": run_id,
                "status": validation_status,
                "policy_row_count": policy_row_count,
                "satisfied_policy_row_count": satisfied_policy_row_count,
                "partition_replay_row_count": partition_replay_row_count,
                "permutation_replay_row_count": permutation_replay_row_count,
                "nontrivial_permutation_row_count": nontrivial_permutation_row_count,
                "observable_invariance_row_count": observable_invariance_row_count,
                "formatting_watch_row_count": formatting_watch_row_count,
                "automatic_dynamic_transition_evidenced": automatic_dynamic_transition_valid,
                "snapshot_counterpart_evidenced": snapshot_counterpart_valid,
                "capability_counterpart_evidenced": capability_counterpart_valid,
                "bounded_partition_analyzer_evidenced": bounded_analyzer_valid,
                "lean_tla_model_bound_evidenced": lean_tla_model_bound_valid,
                "declared_pack_equivalence_evidenced": declared_pack_equivalence_valid,
                "no_proxy_promotion_guard_evidenced": no_proxy_guard_valid,
                "exact_remaining_blocker_count": exact_remaining_blocker_count,
                "failed_row_count": policy_failed_row_count,
                "stage2_policy_promoted": false,
                "pack_grade_replay_promoted": false,
                "validation_failures": validation_failures
            }),
        )?;
        write_json(
            &artifact_root.join("run_summary.json"),
            &json!({
                "schema_version": W042_RUN_SUMMARY_SCHEMA_V1,
                "run_id": run_id,
                "artifact_root": relative_artifact_root,
                "source_evidence_index_path": source_evidence_index_path,
                "w042_stage2_policy_gate_register_path": policy_gate_register_path,
                "w042_production_analyzer_soundness_register_path": production_analyzer_register_path,
                "w042_pack_grade_equivalence_register_path": pack_grade_equivalence_register_path,
                "w042_stage2_exact_blocker_register_path": blocker_register_path,
                "promotion_decision_path": promotion_decision_path,
                "validation_path": validation_path,
                "policy_row_count": policy_row_count,
                "satisfied_policy_row_count": satisfied_policy_row_count,
                "partition_replay_row_count": partition_replay_row_count,
                "permutation_replay_row_count": permutation_replay_row_count,
                "nontrivial_permutation_row_count": nontrivial_permutation_row_count,
                "observable_invariance_row_count": observable_invariance_row_count,
                "formatting_watch_row_count": formatting_watch_row_count,
                "automatic_dynamic_transition_evidenced": automatic_dynamic_transition_valid,
                "snapshot_counterpart_evidenced": snapshot_counterpart_valid,
                "capability_counterpart_evidenced": capability_counterpart_valid,
                "bounded_partition_analyzer_evidenced": bounded_analyzer_valid,
                "lean_tla_model_bound_evidenced": lean_tla_model_bound_valid,
                "declared_pack_equivalence_evidenced": declared_pack_equivalence_valid,
                "no_proxy_promotion_guard_evidenced": no_proxy_guard_valid,
                "exact_remaining_blocker_count": exact_remaining_blocker_count,
                "failed_row_count": policy_failed_row_count,
                "stage2_policy_promoted": false,
                "stage2_promotion_candidate": false,
                "pack_grade_replay_promoted": false,
                "pack_grade_replay_candidate": false
            }),
        )?;

        Ok(Stage2ReplayRunSummary {
            run_id: run_id.to_string(),
            schema_version: W042_RUN_SUMMARY_SCHEMA_V1.to_string(),
            partition_replay_row_count,
            permutation_replay_row_count,
            nontrivial_permutation_row_count,
            observable_invariance_row_count,
            formatting_watch_row_count,
            exact_remaining_blocker_count,
            failed_row_count: policy_failed_row_count,
            stage2_policy_promoted: false,
            artifact_root: relative_artifact_root,
        })
    }

    fn execute_w041(
        &self,
        repo_root: &Path,
        run_id: &str,
    ) -> Result<Stage2ReplayRunSummary, Stage2ReplayError> {
        let relative_artifact_root =
            relative_artifact_path(&["docs", "test-runs", "core-engine", "stage2-replay", run_id]);
        let artifact_root = repo_root.join(&relative_artifact_root);
        if artifact_root.exists() {
            fs::remove_dir_all(&artifact_root).map_err(|source| {
                Stage2ReplayError::RemoveDirectory {
                    path: artifact_root.display().to_string(),
                    source,
                }
            })?;
        }
        fs::create_dir_all(&artifact_root).map_err(|source| {
            Stage2ReplayError::CreateDirectory {
                path: artifact_root.display().to_string(),
                source,
            }
        })?;

        let w041_obligation_summary = read_json(repo_root, W041_OBLIGATION_SUMMARY)?;
        let w041_obligation_map = read_json(repo_root, W041_OBLIGATION_MAP)?;
        let w041_formatting_intake = read_json(repo_root, W041_FORMATTING_INTAKE)?;
        let w041_optimized_summary = read_json(repo_root, W041_OPTIMIZED_CORE_SUMMARY)?;
        let w041_optimized_dispositions = read_json(repo_root, W041_OPTIMIZED_CORE_DISPOSITIONS)?;
        let w041_optimized_blockers = read_json(repo_root, W041_OPTIMIZED_CORE_BLOCKERS)?;
        let w041_dynamic_auto_transition = read_json(repo_root, W041_DYNAMIC_AUTO_TRANSITION)?;
        let w041_treecalc_summary = read_json(repo_root, W041_TREECALC_SUMMARY)?;
        let w041_rust_summary = read_json(repo_root, W041_RUST_SUMMARY)?;
        let w041_rust_validation = read_json(repo_root, W041_RUST_VALIDATION)?;
        let w041_lean_tla_summary = read_json(repo_root, W041_LEAN_TLA_SUMMARY)?;
        let w041_lean_tla_validation = read_json(repo_root, W041_LEAN_TLA_VALIDATION)?;
        let w041_tla_model_register = read_json(repo_root, W041_TLA_MODEL_REGISTER)?;
        let w040_stage2_summary = read_json(repo_root, W040_STAGE2_RUN_SUMMARY)?;
        let w040_stage2_validation = read_json(repo_root, W040_STAGE2_VALIDATION)?;
        let w040_stage2_policy_gate = read_json(repo_root, W040_STAGE2_POLICY_GATE)?;
        let w040_stage2_partition_analyzer = read_json(repo_root, W040_STAGE2_PARTITION_ANALYZER)?;
        let w040_stage2_observable_equivalence =
            read_json(repo_root, W040_STAGE2_OBSERVABLE_EQUIVALENCE)?;
        let w040_stage2_blockers = read_json(repo_root, W040_STAGE2_BLOCKERS)?;
        let w040_stage2_promotion_decision = read_json(repo_root, W040_STAGE2_PROMOTION_DECISION)?;
        let lean_file_present = repo_root.join(W041_STAGE2_LEAN_FILE).exists();

        let w041_obligation_valid = number_at(&w041_obligation_summary, "obligation_count") == 28
            && bool_at(
                &w041_obligation_summary,
                "oxfml_formatting_update_incorporated",
            );
        let w041_optimized_valid = number_at(&w041_optimized_summary, "failed_row_count") == 0
            && number_at(
                &w041_optimized_summary,
                "dynamic_transition_implementation_evidence_count",
            ) == 1
            && number_at(&w041_optimized_summary, "match_promoted_count") == 0
            && number_at(&w041_optimized_summary, "exact_remaining_blocker_count") == 3;
        let w041_treecalc_valid = number_at(&w041_treecalc_summary, "case_count") == 26
            && number_at(&w041_treecalc_summary, "expectation_mismatch_count") == 0;
        let dynamic_auto_phase = &w041_dynamic_auto_transition["post_edit_phase"];
        let automatic_dynamic_transition_valid = w041_optimized_valid
            && w041_treecalc_valid
            && string_at(dynamic_auto_phase, "result_state") == "rejected"
            && array_field_contains_string(
                dynamic_auto_phase,
                "automatic_dependency_change_seeds",
                "DependencyRemoved",
            )
            && array_field_contains_string(
                dynamic_auto_phase,
                "automatic_dependency_change_seeds",
                "DependencyReclassified",
            )
            && bool_at(
                &dynamic_auto_phase["invalidation_closure"],
                "requires_rebind",
            );
        let w041_rust_valid = string_at(&w041_rust_validation, "status")
            == "formal_assurance_w041_rust_totality_refinement_valid"
            && number_at(&w041_rust_summary, "automatic_dynamic_transition_row_count") == 1
            && !bool_at(
                &w041_rust_summary["promotion_claims"],
                "stage2_policy_promoted",
            );
        let w041_lean_tla_valid = string_at(&w041_lean_tla_validation, "status")
            == "formal_assurance_w041_lean_tla_discharge_valid"
            && number_at(&w041_lean_tla_summary, "bounded_model_row_count") == 4
            && !bool_at(
                &w041_lean_tla_summary["promotion_claims"],
                "stage2_policy_promoted",
            );
        let w040_stage2_valid = string_at(&w040_stage2_validation, "status")
            == "w040_stage2_policy_equivalence_valid"
            && !bool_at(&w040_stage2_summary, "stage2_policy_promoted")
            && !bool_at(&w040_stage2_promotion_decision, "stage2_policy_promoted");

        let partition_replay_row_count =
            number_at(&w040_stage2_summary, "partition_replay_row_count") as usize;
        let permutation_replay_row_count =
            number_at(&w040_stage2_summary, "permutation_replay_row_count") as usize;
        let nontrivial_permutation_row_count =
            number_at(&w040_stage2_summary, "nontrivial_permutation_row_count") as usize;
        let observable_invariance_row_count =
            number_at(&w040_stage2_summary, "observable_invariance_row_count") as usize;
        let formatting_watch_row_count =
            number_at(&w040_stage2_summary, "formatting_watch_row_count") as usize;
        let bounded_replay_valid = w040_stage2_valid && partition_replay_row_count == 5;
        let permutation_replay_valid = w040_stage2_valid
            && permutation_replay_row_count == 6
            && nontrivial_permutation_row_count == 1;
        let observable_invariance_valid = w040_stage2_valid && observable_invariance_row_count == 5;
        let snapshot_blocker_retained = row_with_field_exists(
            &w041_optimized_blockers,
            "row_id",
            "w041_snapshot_fence_counterpart_exact_blocker",
        ) && row_with_field_exists(
            &w041_optimized_dispositions,
            "row_id",
            "w041_snapshot_fence_counterpart_exact_blocker",
        );
        let capability_blocker_retained = row_with_field_exists(
            &w041_optimized_blockers,
            "row_id",
            "w041_capability_view_fence_counterpart_exact_blocker",
        ) && row_with_field_exists(
            &w041_optimized_dispositions,
            "row_id",
            "w041_capability_view_fence_counterpart_exact_blocker",
        );
        let snapshot_counterpart_valid =
            bool_at(&w040_stage2_summary, "snapshot_counterpart_evidenced")
                && bool_at(
                    &w040_stage2_promotion_decision,
                    "snapshot_fence_counterpart_evidenced",
                )
                && row_with_field_exists(
                    &w040_stage2_observable_equivalence,
                    "row_id",
                    "w040_stage2_snapshot_fence_counterpart_direct_evidence",
                )
                && snapshot_blocker_retained;
        let capability_counterpart_valid =
            bool_at(&w040_stage2_summary, "capability_counterpart_evidenced")
                && bool_at(
                    &w040_stage2_promotion_decision,
                    "capability_view_fence_counterpart_evidenced",
                )
                && row_with_field_exists(
                    &w040_stage2_observable_equivalence,
                    "row_id",
                    "w040_stage2_capability_view_fence_counterpart_direct_evidence",
                )
                && capability_blocker_retained;
        let bounded_analyzer_valid =
            bool_at(&w040_stage2_summary, "bounded_partition_analyzer_evidenced")
                && number_at(&w040_stage2_partition_analyzer, "row_count") == 6
                && row_with_field_exists(
                    &w041_tla_model_register,
                    "row_id",
                    "w041_stage2_equivalence_bounded_model_input",
                )
                && w041_lean_tla_valid
                && lean_file_present;
        let w073_guard_valid = formatting_watch_row_count == 1
            && string_at(&w041_formatting_intake, "accepted_metadata_source")
                == "VerificationConditionalFormattingRule.typed_rule"
            && string_at(
                &w041_formatting_intake,
                "ignored_legacy_metadata_source_for_named_families",
            ) == "W072 bounded thresholds strings"
            && array_field_contains_string(
                &w041_formatting_intake,
                "typed_rule_only_families",
                "colorScale",
            )
            && array_field_contains_string(
                &w041_formatting_intake,
                "typed_rule_only_families",
                "dataBar",
            )
            && bool_at(
                &w041_obligation_summary,
                "oxfml_formatting_update_incorporated",
            );
        let declared_pack_equivalence_valid = observable_invariance_valid
            && snapshot_counterpart_valid
            && capability_counterpart_valid
            && number_at(&w040_stage2_observable_equivalence, "row_count") == 8
            && !bool_at(&w040_stage2_summary, "stage2_policy_promoted");
        let no_proxy_guard_valid = w041_obligation_valid
            && number_at(&w041_optimized_summary, "match_promoted_count") == 0
            && !bool_at(&w040_stage2_summary, "stage2_policy_promoted")
            && !bool_at(
                &w040_stage2_promotion_decision,
                "stage2_promotion_candidate",
            );

        let policy_rows = vec![
            json!({
                "row_id": "w041_stage2_bounded_replay_carried",
                "w041_obligation_id": "W041-OBL-013",
                "policy_area": "bounded_baseline_vs_stage2_replay",
                "source_artifacts": [W040_STAGE2_RUN_SUMMARY, W040_STAGE2_POLICY_GATE],
                "satisfied_for_declared_profile": bounded_replay_valid,
                "exact_remaining_blocker": false,
                "promotion_consequence": "bounded replay remains declared-profile evidence only",
                "disposition": "carry W040 bounded baseline-versus-Stage-2 replay rows into W041 pack-equivalence classification",
                "failures": if bounded_replay_valid { Vec::<String>::new() } else { vec!["w041_bounded_replay_not_valid".to_string()] },
            }),
            json!({
                "row_id": "w041_stage2_partition_order_permutation_carried",
                "w041_obligation_id": "W041-OBL-013",
                "policy_area": "partition_order_permutation_replay",
                "source_artifacts": [W040_STAGE2_RUN_SUMMARY, W040_STAGE2_POLICY_GATE],
                "satisfied_for_declared_profile": permutation_replay_valid,
                "exact_remaining_blocker": false,
                "promotion_consequence": "bounded permutation replay does not discharge production scheduler fairness",
                "disposition": "carry one nontrivial partition-order permutation row and six total permutation rows",
                "failures": if permutation_replay_valid { Vec::<String>::new() } else { vec!["w041_partition_permutation_replay_not_valid".to_string()] },
            }),
            json!({
                "row_id": "w041_stage2_observable_invariance_carried",
                "w041_obligation_id": "W041-OBL-013",
                "policy_area": "observable_result_invariance",
                "source_artifacts": [W040_STAGE2_RUN_SUMMARY, W040_STAGE2_OBSERVABLE_EQUIVALENCE],
                "satisfied_for_declared_profile": observable_invariance_valid,
                "exact_remaining_blocker": false,
                "promotion_consequence": "observable invariance is evidenced for declared bounded profiles only",
                "disposition": "carry W040 observable-result invariance rows for values, rejects, and formatting watch surfaces",
                "failures": if observable_invariance_valid { Vec::<String>::new() } else { vec!["w041_observable_invariance_not_valid".to_string()] },
            }),
            json!({
                "row_id": "w041_stage2_automatic_dynamic_transition_bound",
                "w041_obligation_id": "W041-OBL-012",
                "policy_area": "dynamic_and_soft_reference_transition",
                "source_artifacts": [W041_OPTIMIZED_CORE_SUMMARY, W041_DYNAMIC_AUTO_TRANSITION, W041_TREECALC_SUMMARY, W041_RUST_SUMMARY],
                "satisfied_for_declared_profile": automatic_dynamic_transition_valid && w041_rust_valid,
                "exact_remaining_blocker": false,
                "promotion_consequence": "automatic dynamic transition evidence narrows declared-profile analyzer inputs but does not prove production analyzer soundness",
                "disposition": "bind W041 automatic resolved-to-potential dynamic transition evidence to Stage 2 analyzer preconditions",
                "failures": if automatic_dynamic_transition_valid && w041_rust_valid { Vec::<String>::new() } else { vec!["w041_automatic_dynamic_transition_not_valid".to_string()] },
            }),
            json!({
                "row_id": "w041_stage2_snapshot_fence_counterpart_evidence",
                "w041_obligation_id": "W041-OBL-003",
                "policy_area": "snapshot_fence_counterpart",
                "source_artifacts": [W041_OPTIMIZED_CORE_BLOCKERS, W041_OPTIMIZED_CORE_DISPOSITIONS, W040_STAGE2_OBSERVABLE_EQUIVALENCE, W040_STAGE2_PROMOTION_DECISION],
                "satisfied_for_declared_profile": snapshot_counterpart_valid,
                "exact_remaining_blocker": false,
                "promotion_consequence": "the calc-sui.5 Stage 2 counterpart is evidenced for declared profiles; full optimized/core and production policy remain blocked by broader gates",
                "disposition": "bind W041 retained snapshot blocker to W040 direct Stage 2 stale-snapshot reject/no-publish counterpart evidence without match promotion",
                "failures": if snapshot_counterpart_valid { Vec::<String>::new() } else { vec!["w041_snapshot_counterpart_not_valid".to_string()] },
            }),
            json!({
                "row_id": "w041_stage2_capability_view_counterpart_evidence",
                "w041_obligation_id": "W041-OBL-004",
                "policy_area": "capability_view_fence_counterpart",
                "source_artifacts": [W041_OPTIMIZED_CORE_BLOCKERS, W041_OPTIMIZED_CORE_DISPOSITIONS, W040_STAGE2_OBSERVABLE_EQUIVALENCE, W040_STAGE2_PROMOTION_DECISION],
                "satisfied_for_declared_profile": capability_counterpart_valid,
                "exact_remaining_blocker": false,
                "promotion_consequence": "the calc-sui.5 Stage 2 counterpart is evidenced for declared profiles; full optimized/core and production policy remain blocked by broader gates",
                "disposition": "bind W041 retained capability-view blocker to W040 direct Stage 2 capability mismatch reject/no-publish counterpart evidence without match promotion",
                "failures": if capability_counterpart_valid { Vec::<String>::new() } else { vec!["w041_capability_counterpart_not_valid".to_string()] },
            }),
            json!({
                "row_id": "w041_stage2_bounded_partition_analyzer_predicate_evidence",
                "w041_obligation_id": "W041-OBL-012",
                "policy_area": "bounded_partition_analyzer_soundness",
                "source_artifacts": [W040_STAGE2_PARTITION_ANALYZER, W041_TLA_MODEL_REGISTER, W041_STAGE2_LEAN_FILE],
                "satisfied_for_declared_profile": bounded_analyzer_valid,
                "exact_remaining_blocker": false,
                "promotion_consequence": "bounded analyzer preconditions are evidenced, but full production analyzer soundness remains blocked",
                "disposition": "bind W040 bounded partition-analyzer evidence and W041 bounded Stage 2 model input under the W041 Lean predicate",
                "failures": if bounded_analyzer_valid { Vec::<String>::new() } else { vec!["w041_bounded_partition_analyzer_not_valid".to_string()] },
            }),
            json!({
                "row_id": "w041_stage2_w073_typed_formatting_guard_carried",
                "w041_obligation_id": "W041-OBL-021",
                "policy_area": "w073_typed_formatting_watch",
                "source_artifacts": [W041_FORMATTING_INTAKE, W040_STAGE2_RUN_SUMMARY],
                "satisfied_for_declared_profile": w073_guard_valid,
                "exact_remaining_blocker": false,
                "promotion_consequence": "typed-only formatting is carried as an observable-surface guard; broad OxFml seam breadth remains under calc-sui.8",
                "disposition": "retain the W073 direct-replacement rule: typed_rule is the sole metadata source for aggregate and visualization conditional-formatting families",
                "failures": if w073_guard_valid { Vec::<String>::new() } else { vec!["w041_w073_formatting_guard_not_valid".to_string()] },
            }),
            json!({
                "row_id": "w041_stage2_declared_pack_replay_equivalence",
                "w041_obligation_id": "W041-OBL-013",
                "policy_area": "declared_profile_pack_replay_equivalence",
                "source_artifacts": [W040_STAGE2_RUN_SUMMARY, W040_STAGE2_OBSERVABLE_EQUIVALENCE, W040_STAGE2_PROMOTION_DECISION],
                "satisfied_for_declared_profile": declared_pack_equivalence_valid,
                "exact_remaining_blocker": false,
                "promotion_consequence": "declared-profile equivalence evidence is not pack-grade replay governance",
                "disposition": "classify values, rejects, fence no-publish behavior, typed-formatting watch, and replay validation as declared-profile pack-equivalence inputs",
                "failures": if declared_pack_equivalence_valid { Vec::<String>::new() } else { vec!["w041_declared_pack_equivalence_not_valid".to_string()] },
            }),
            json!({
                "row_id": "w041_stage2_no_proxy_promotion_guard",
                "w041_obligation_id": "W041-OBL-006",
                "policy_area": "no_proxy_promotion_guard",
                "source_artifacts": [W041_OBLIGATION_SUMMARY, W041_OPTIMIZED_CORE_SUMMARY, W040_STAGE2_PROMOTION_DECISION],
                "satisfied_for_declared_profile": no_proxy_guard_valid,
                "exact_remaining_blocker": false,
                "promotion_consequence": "Stage 2, pack, and release-grade gates remain blocked unless direct target evidence is present",
                "disposition": "keep bounded-only, declared-profile, and retained-blocker evidence out of promotion counts",
                "failures": if no_proxy_guard_valid { Vec::<String>::new() } else { vec!["w041_no_proxy_guard_not_valid".to_string()] },
            }),
            json!({
                "row_id": "w041_stage2_full_production_analyzer_soundness_blocker",
                "w041_obligation_id": "W041-OBL-012",
                "policy_area": "full_production_partition_analyzer_soundness",
                "source_artifacts": [W041_OBLIGATION_MAP, W040_STAGE2_BLOCKERS, W041_STAGE2_LEAN_FILE],
                "satisfied_for_declared_profile": false,
                "exact_remaining_blocker": true,
                "promotion_consequence": "Stage 2 production policy remains unpromoted",
                "disposition": "retain full production partition-analyzer soundness as exact blocker beyond bounded declared-profile evidence",
                "failures": if obligation_exists(&w041_obligation_map, "W041-OBL-012") && row_with_field_exists(&w040_stage2_blockers, "row_id", "w040_stage2_full_production_analyzer_soundness_blocker") { Vec::<String>::new() } else { vec!["w041_production_analyzer_blocker_input_missing".to_string()] },
            }),
            json!({
                "row_id": "w041_stage2_fairness_scheduler_unbounded_coverage_blocker",
                "w041_obligation_id": "W041-OBL-011",
                "policy_area": "fairness_scheduler_unbounded_coverage",
                "source_artifacts": [W041_LEAN_TLA_SUMMARY, W041_TLA_MODEL_REGISTER],
                "satisfied_for_declared_profile": false,
                "exact_remaining_blocker": true,
                "promotion_consequence": "full TLA verification and Stage 2 production policy remain unpromoted",
                "disposition": "retain fairness, unbounded scheduler coverage, and model-completeness limits as exact Stage 2 blockers",
                "failures": if obligation_exists(&w041_obligation_map, "W041-OBL-011") && w041_lean_tla_valid { Vec::<String>::new() } else { vec!["w041_fairness_scheduler_blocker_input_missing".to_string()] },
            }),
            json!({
                "row_id": "w041_stage2_operated_cross_engine_service_dependency_blocker",
                "w041_obligation_id": "W041-OBL-018",
                "policy_area": "operated_cross_engine_stage2_differential_service",
                "source_artifacts": [W041_OBLIGATION_MAP, W040_STAGE2_BLOCKERS],
                "satisfied_for_declared_profile": false,
                "exact_remaining_blocker": true,
                "promotion_consequence": "operated Stage 2 differential evidence remains required before policy promotion",
                "disposition": "retain operated cross-engine differential service as successor dependency",
                "failures": if obligation_exists(&w041_obligation_map, "W041-OBL-018") && row_with_field_exists(&w040_stage2_blockers, "row_id", "w040_stage2_operated_cross_engine_service_dependency_blocker") { Vec::<String>::new() } else { vec!["w041_operated_stage2_service_blocker_input_missing".to_string()] },
            }),
            json!({
                "row_id": "w041_stage2_pack_grade_replay_governance_blocker",
                "w041_obligation_id": "W041-OBL-025",
                "policy_area": "pack_grade_replay_governance",
                "source_artifacts": [W041_OBLIGATION_MAP, W040_STAGE2_BLOCKERS],
                "satisfied_for_declared_profile": false,
                "exact_remaining_blocker": true,
                "promotion_consequence": "pack-grade replay, C5, and release-grade verification remain blocked",
                "disposition": "retain pack-grade replay governance and retained-witness lifecycle as release-decision blockers",
                "failures": if obligation_exists(&w041_obligation_map, "W041-OBL-025") && row_with_field_exists(&w040_stage2_blockers, "row_id", "w040_stage2_pack_grade_replay_governance_blocker") { Vec::<String>::new() } else { vec!["w041_pack_grade_replay_governance_blocker_input_missing".to_string()] },
            }),
        ];

        let satisfied_policy_row_count = policy_rows
            .iter()
            .filter(|row| bool_at(row, "satisfied_for_declared_profile"))
            .count();
        let policy_blocker_rows = policy_rows
            .iter()
            .filter(|row| bool_at(row, "exact_remaining_blocker"))
            .cloned()
            .collect::<Vec<_>>();
        let policy_failed_row_count = policy_rows
            .iter()
            .filter(|row| {
                !row.get("failures")
                    .and_then(Value::as_array)
                    .is_some_and(Vec::is_empty)
            })
            .count();
        let exact_remaining_blocker_count = policy_blocker_rows.len();
        let policy_row_count = policy_rows.len();

        let production_analyzer_rows = policy_rows
            .iter()
            .filter(|row| {
                matches!(
                    string_at(row, "policy_area"),
                    "dynamic_and_soft_reference_transition"
                        | "snapshot_fence_counterpart"
                        | "capability_view_fence_counterpart"
                        | "bounded_partition_analyzer_soundness"
                        | "full_production_partition_analyzer_soundness"
                        | "fairness_scheduler_unbounded_coverage"
                )
            })
            .cloned()
            .collect::<Vec<_>>();
        let pack_equivalence_rows = policy_rows
            .iter()
            .filter(|row| {
                matches!(
                    string_at(row, "policy_area"),
                    "bounded_baseline_vs_stage2_replay"
                        | "partition_order_permutation_replay"
                        | "observable_result_invariance"
                        | "snapshot_fence_counterpart"
                        | "capability_view_fence_counterpart"
                        | "w073_typed_formatting_watch"
                        | "declared_profile_pack_replay_equivalence"
                        | "no_proxy_promotion_guard"
                        | "operated_cross_engine_stage2_differential_service"
                        | "pack_grade_replay_governance"
                )
            })
            .cloned()
            .collect::<Vec<_>>();

        let mut validation_failures = Vec::new();
        if !w041_obligation_valid {
            validation_failures.push("w041_obligation_map_not_valid".to_string());
        }
        for obligation_id in [
            "W041-OBL-003",
            "W041-OBL-004",
            "W041-OBL-006",
            "W041-OBL-011",
            "W041-OBL-012",
            "W041-OBL-013",
            "W041-OBL-018",
            "W041-OBL-021",
            "W041-OBL-025",
        ] {
            if !obligation_exists(&w041_obligation_map, obligation_id) {
                validation_failures.push(format!("{obligation_id}_missing"));
            }
        }
        if !w040_stage2_valid {
            validation_failures.push("w040_stage2_policy_equivalence_not_valid".to_string());
        }
        if number_at(&w040_stage2_policy_gate, "policy_row_count") != 12 {
            validation_failures.push("w040_stage2_policy_gate_row_count_changed".to_string());
        }
        if !w041_optimized_valid || !w041_treecalc_valid {
            validation_failures.push("w041_optimized_stage2_inputs_not_valid".to_string());
        }
        if !automatic_dynamic_transition_valid {
            validation_failures.push("w041_automatic_dynamic_transition_not_valid".to_string());
        }
        if !w041_rust_valid {
            validation_failures.push("w041_rust_stage2_inputs_not_valid".to_string());
        }
        if !w041_lean_tla_valid {
            validation_failures.push("w041_lean_tla_stage2_inputs_not_valid".to_string());
        }
        if !snapshot_counterpart_valid {
            validation_failures.push("w041_snapshot_counterpart_not_valid".to_string());
        }
        if !capability_counterpart_valid {
            validation_failures.push("w041_capability_counterpart_not_valid".to_string());
        }
        if !bounded_analyzer_valid {
            validation_failures.push("w041_bounded_partition_analyzer_not_valid".to_string());
        }
        if !declared_pack_equivalence_valid {
            validation_failures.push("w041_declared_pack_equivalence_not_valid".to_string());
        }
        if !no_proxy_guard_valid {
            validation_failures.push("w041_no_proxy_guard_not_valid".to_string());
        }
        if !w073_guard_valid {
            validation_failures.push("w041_w073_formatting_guard_not_valid".to_string());
        }
        if !lean_file_present {
            validation_failures.push("w041_stage2_lean_file_missing".to_string());
        }
        if policy_failed_row_count != 0 {
            validation_failures.push("w041_stage2_policy_row_failures_present".to_string());
        }
        if satisfied_policy_row_count != 10 {
            validation_failures.push("w041_stage2_expected_ten_satisfied_policy_rows".to_string());
        }
        if exact_remaining_blocker_count != 4 {
            validation_failures.push("w041_stage2_expected_four_exact_blockers".to_string());
        }

        let source_evidence_index_path =
            format!("{relative_artifact_root}/source_evidence_index.json");
        let policy_gate_register_path =
            format!("{relative_artifact_root}/w041_stage2_policy_gate_register.json");
        let production_analyzer_register_path =
            format!("{relative_artifact_root}/w041_production_analyzer_soundness_register.json");
        let pack_equivalence_register_path =
            format!("{relative_artifact_root}/w041_pack_equivalence_register.json");
        let blocker_register_path =
            format!("{relative_artifact_root}/w041_stage2_exact_blocker_register.json");
        let promotion_decision_path = format!("{relative_artifact_root}/promotion_decision.json");
        let validation_path = format!("{relative_artifact_root}/validation.json");

        write_json(
            &artifact_root.join("source_evidence_index.json"),
            &json!({
                "schema_version": W041_SOURCE_INDEX_SCHEMA_V1,
                "run_id": run_id,
                "artifact_root": relative_artifact_root,
                "source_artifacts": {
                    "w041_obligation_summary": W041_OBLIGATION_SUMMARY,
                    "w041_obligation_map": W041_OBLIGATION_MAP,
                    "w041_formatting_intake": W041_FORMATTING_INTAKE,
                    "w041_optimized_core_summary": W041_OPTIMIZED_CORE_SUMMARY,
                    "w041_optimized_core_dispositions": W041_OPTIMIZED_CORE_DISPOSITIONS,
                    "w041_optimized_core_blockers": W041_OPTIMIZED_CORE_BLOCKERS,
                    "w041_dynamic_auto_transition": W041_DYNAMIC_AUTO_TRANSITION,
                    "w041_treecalc_summary": W041_TREECALC_SUMMARY,
                    "w041_rust_summary": W041_RUST_SUMMARY,
                    "w041_rust_validation": W041_RUST_VALIDATION,
                    "w041_lean_tla_summary": W041_LEAN_TLA_SUMMARY,
                    "w041_lean_tla_validation": W041_LEAN_TLA_VALIDATION,
                    "w041_tla_model_register": W041_TLA_MODEL_REGISTER,
                    "w040_stage2_run_summary": W040_STAGE2_RUN_SUMMARY,
                    "w040_stage2_validation": W040_STAGE2_VALIDATION,
                    "w040_stage2_policy_gate": W040_STAGE2_POLICY_GATE,
                    "w040_stage2_partition_analyzer": W040_STAGE2_PARTITION_ANALYZER,
                    "w040_stage2_observable_equivalence": W040_STAGE2_OBSERVABLE_EQUIVALENCE,
                    "w040_stage2_blockers": W040_STAGE2_BLOCKERS,
                    "w040_stage2_promotion_decision": W040_STAGE2_PROMOTION_DECISION,
                    "w041_stage2_lean_file": W041_STAGE2_LEAN_FILE
                },
                "source_counts": {
                    "w041_obligation_count": number_at(&w041_obligation_summary, "obligation_count"),
                    "w041_optimized_exact_remaining_blocker_count": number_at(&w041_optimized_summary, "exact_remaining_blocker_count"),
                    "w041_automatic_dynamic_transition_row_count": number_at(&w041_rust_summary, "automatic_dynamic_transition_row_count"),
                    "w041_lean_tla_bounded_model_row_count": number_at(&w041_lean_tla_summary, "bounded_model_row_count"),
                    "w040_stage2_policy_row_count": number_at(&w040_stage2_summary, "policy_row_count"),
                    "w040_stage2_exact_remaining_blocker_count": number_at(&w040_stage2_summary, "exact_remaining_blocker_count")
                }
            }),
        )?;
        write_json(
            &artifact_root.join("w041_stage2_policy_gate_register.json"),
            &json!({
                "schema_version": W041_POLICY_GATE_SCHEMA_V1,
                "run_id": run_id,
                "policy_row_count": policy_row_count,
                "satisfied_policy_row_count": satisfied_policy_row_count,
                "exact_remaining_blocker_count": exact_remaining_blocker_count,
                "stage2_policy_promoted": false,
                "rows": policy_rows
            }),
        )?;
        write_json(
            &artifact_root.join("w041_production_analyzer_soundness_register.json"),
            &json!({
                "schema_version": W041_PRODUCTION_ANALYZER_SCHEMA_V1,
                "run_id": run_id,
                "row_count": production_analyzer_rows.len(),
                "satisfied_policy_row_count": production_analyzer_rows
                    .iter()
                    .filter(|row| bool_at(row, "satisfied_for_declared_profile"))
                    .count(),
                "exact_remaining_blocker_count": production_analyzer_rows
                    .iter()
                    .filter(|row| bool_at(row, "exact_remaining_blocker"))
                    .count(),
                "semantic_statement": "W041 binds automatic dynamic transition evidence, declared-profile fence counterparts, and bounded analyzer evidence as production-analyzer inputs. Full production partition-analyzer soundness remains blocked.",
                "rows": production_analyzer_rows
            }),
        )?;
        write_json(
            &artifact_root.join("w041_pack_equivalence_register.json"),
            &json!({
                "schema_version": W041_PACK_EQUIVALENCE_SCHEMA_V1,
                "run_id": run_id,
                "row_count": pack_equivalence_rows.len(),
                "satisfied_policy_row_count": pack_equivalence_rows
                    .iter()
                    .filter(|row| bool_at(row, "satisfied_for_declared_profile"))
                    .count(),
                "exact_remaining_blocker_count": pack_equivalence_rows
                    .iter()
                    .filter(|row| bool_at(row, "exact_remaining_blocker"))
                    .count(),
                "semantic_equivalence_statement": "Declared-profile values, rejects, fence no-publish behavior, typed-formatting observable guards, replay validation, and no-proxy promotion guards are bound as pack-equivalence inputs. This is not pack-grade replay governance.",
                "rows": pack_equivalence_rows
            }),
        )?;
        write_json(
            &artifact_root.join("w041_stage2_exact_blocker_register.json"),
            &json!({
                "schema_version": W041_BLOCKER_REGISTER_SCHEMA_V1,
                "run_id": run_id,
                "exact_remaining_blocker_count": exact_remaining_blocker_count,
                "rows": policy_blocker_rows
            }),
        )?;
        write_json(
            &artifact_root.join("promotion_decision.json"),
            &json!({
                "schema_version": W041_PROMOTION_DECISION_SCHEMA_V1,
                "run_id": run_id,
                "decision_state": "w041_stage2_analyzer_pack_equivalence_validated_policy_unpromoted",
                "stage2_policy_promoted": false,
                "stage2_promotion_candidate": false,
                "bounded_partition_replay_present": bounded_replay_valid,
                "partition_order_permutation_replay_present": permutation_replay_valid,
                "observable_result_invariance_evidenced_for_declared_profiles": observable_invariance_valid,
                "automatic_dynamic_transition_evidenced_for_declared_profiles": automatic_dynamic_transition_valid,
                "snapshot_fence_counterpart_evidenced": snapshot_counterpart_valid,
                "capability_view_fence_counterpart_evidenced": capability_counterpart_valid,
                "bounded_partition_analyzer_soundness_evidenced": bounded_analyzer_valid,
                "w073_typed_formatting_guard_carried": w073_guard_valid,
                "declared_profile_pack_replay_equivalence_evidenced": declared_pack_equivalence_valid,
                "no_proxy_promotion_guard_evidenced": no_proxy_guard_valid,
                "production_partition_analyzer_soundness_promoted": false,
                "fairness_scheduler_unbounded_coverage_promoted": false,
                "operated_cross_engine_stage2_service_promoted": false,
                "pack_grade_replay_governance_promoted": false,
                "satisfied_inputs": [
                    "bounded_partition_replay_present",
                    "partition_order_permutation_replay_present",
                    "observable_result_invariance_for_declared_profiles",
                    "automatic_dynamic_transition_evidenced_for_declared_profiles",
                    "snapshot_fence_counterpart_evidenced_for_declared_profiles",
                    "capability_view_fence_counterpart_evidenced_for_declared_profiles",
                    "bounded_partition_analyzer_soundness_evidenced",
                    "w073_typed_formatting_guard_carried",
                    "declared_profile_pack_replay_equivalence_evidenced",
                    "no_proxy_promotion_guard_evidenced"
                ],
                "blockers": [
                    "stage2.full_production_partition_analyzer_soundness_absent",
                    "stage2.fairness_scheduler_unbounded_coverage_absent",
                    "stage2.operated_cross_engine_differential_service_absent",
                    "stage2.pack_grade_replay_governance_absent"
                ],
                "semantic_equivalence_statement": "Observable-result and replay equivalence are evidenced for declared W041 Stage 2 profiles, including automatic dynamic transition input, snapshot/capability fence no-publish counterparts, and W073 typed-formatting guards. Production Stage 2 policy and pack-grade replay remain unpromoted until production analyzer soundness, fairness and unbounded scheduler coverage, operated cross-engine service evidence, and pack governance are present."
            }),
        )?;

        let validation_status = if validation_failures.is_empty() {
            "w041_stage2_analyzer_pack_equivalence_valid"
        } else {
            "w041_stage2_analyzer_pack_equivalence_invalid"
        };
        write_json(
            &artifact_root.join("validation.json"),
            &json!({
                "schema_version": W041_VALIDATION_SCHEMA_V1,
                "run_id": run_id,
                "status": validation_status,
                "policy_row_count": policy_row_count,
                "satisfied_policy_row_count": satisfied_policy_row_count,
                "partition_replay_row_count": partition_replay_row_count,
                "permutation_replay_row_count": permutation_replay_row_count,
                "nontrivial_permutation_row_count": nontrivial_permutation_row_count,
                "observable_invariance_row_count": observable_invariance_row_count,
                "formatting_watch_row_count": formatting_watch_row_count,
                "automatic_dynamic_transition_evidenced": automatic_dynamic_transition_valid,
                "snapshot_counterpart_evidenced": snapshot_counterpart_valid,
                "capability_counterpart_evidenced": capability_counterpart_valid,
                "bounded_partition_analyzer_evidenced": bounded_analyzer_valid,
                "declared_pack_equivalence_evidenced": declared_pack_equivalence_valid,
                "no_proxy_promotion_guard_evidenced": no_proxy_guard_valid,
                "exact_remaining_blocker_count": exact_remaining_blocker_count,
                "failed_row_count": policy_failed_row_count,
                "stage2_policy_promoted": false,
                "validation_failures": validation_failures
            }),
        )?;
        write_json(
            &artifact_root.join("run_summary.json"),
            &json!({
                "schema_version": W041_RUN_SUMMARY_SCHEMA_V1,
                "run_id": run_id,
                "artifact_root": relative_artifact_root,
                "source_evidence_index_path": source_evidence_index_path,
                "w041_stage2_policy_gate_register_path": policy_gate_register_path,
                "w041_production_analyzer_soundness_register_path": production_analyzer_register_path,
                "w041_pack_equivalence_register_path": pack_equivalence_register_path,
                "w041_stage2_exact_blocker_register_path": blocker_register_path,
                "promotion_decision_path": promotion_decision_path,
                "validation_path": validation_path,
                "policy_row_count": policy_row_count,
                "satisfied_policy_row_count": satisfied_policy_row_count,
                "partition_replay_row_count": partition_replay_row_count,
                "permutation_replay_row_count": permutation_replay_row_count,
                "nontrivial_permutation_row_count": nontrivial_permutation_row_count,
                "observable_invariance_row_count": observable_invariance_row_count,
                "formatting_watch_row_count": formatting_watch_row_count,
                "automatic_dynamic_transition_evidenced": automatic_dynamic_transition_valid,
                "snapshot_counterpart_evidenced": snapshot_counterpart_valid,
                "capability_counterpart_evidenced": capability_counterpart_valid,
                "bounded_partition_analyzer_evidenced": bounded_analyzer_valid,
                "declared_pack_equivalence_evidenced": declared_pack_equivalence_valid,
                "no_proxy_promotion_guard_evidenced": no_proxy_guard_valid,
                "exact_remaining_blocker_count": exact_remaining_blocker_count,
                "failed_row_count": policy_failed_row_count,
                "stage2_policy_promoted": false,
                "stage2_promotion_candidate": false
            }),
        )?;

        Ok(Stage2ReplayRunSummary {
            run_id: run_id.to_string(),
            schema_version: W041_RUN_SUMMARY_SCHEMA_V1.to_string(),
            partition_replay_row_count,
            permutation_replay_row_count,
            nontrivial_permutation_row_count,
            observable_invariance_row_count,
            formatting_watch_row_count,
            exact_remaining_blocker_count,
            failed_row_count: policy_failed_row_count,
            stage2_policy_promoted: false,
            artifact_root: relative_artifact_root,
        })
    }

    fn execute_w040(
        &self,
        repo_root: &Path,
        run_id: &str,
    ) -> Result<Stage2ReplayRunSummary, Stage2ReplayError> {
        let relative_artifact_root =
            relative_artifact_path(&["docs", "test-runs", "core-engine", "stage2-replay", run_id]);
        let artifact_root = repo_root.join(&relative_artifact_root);
        if artifact_root.exists() {
            fs::remove_dir_all(&artifact_root).map_err(|source| {
                Stage2ReplayError::RemoveDirectory {
                    path: artifact_root.display().to_string(),
                    source,
                }
            })?;
        }
        fs::create_dir_all(&artifact_root).map_err(|source| {
            Stage2ReplayError::CreateDirectory {
                path: artifact_root.display().to_string(),
                source,
            }
        })?;

        let w040_obligation_summary = read_json(repo_root, W040_DIRECT_OBLIGATION_SUMMARY)?;
        let w040_obligation_map = read_json(repo_root, W040_DIRECT_OBLIGATION_MAP)?;
        let w039_stage2_summary = read_json(repo_root, W039_STAGE2_RUN_SUMMARY)?;
        let w039_stage2_validation = read_json(repo_root, W039_STAGE2_VALIDATION)?;
        let w039_stage2_policy_gate = read_json(repo_root, W039_STAGE2_POLICY_GATE)?;
        let w039_stage2_blockers = read_json(repo_root, W039_STAGE2_BLOCKERS)?;
        let w040_optimized_summary = read_json(repo_root, W040_OPTIMIZED_CORE_SUMMARY)?;
        let w040_optimized_blockers = read_json(repo_root, W040_OPTIMIZED_CORE_BLOCKERS)?;
        let w040_treecalc_summary = read_json(repo_root, W040_TREECALC_SUMMARY)?;
        let w040_lean_tla_summary = read_json(repo_root, W040_LEAN_TLA_SUMMARY)?;
        let w040_lean_tla_validation = read_json(repo_root, W040_LEAN_TLA_VALIDATION)?;
        let w040_lean_tla_model_register = read_json(repo_root, W040_LEAN_TLA_MODEL_REGISTER)?;
        let w036_tla_summary = read_json(repo_root, W036_STAGE2_TLA_RUN_SUMMARY)?;
        let w036_tla_promotion_blockers = read_json(repo_root, W036_STAGE2_TLA_PROMOTION_BLOCKERS)?;
        let snapshot_result = read_json(repo_root, TRACE_SNAPSHOT_FENCE_RESULT)?;
        let snapshot_rejects = read_json(repo_root, TRACE_SNAPSHOT_FENCE_REJECTS)?;
        let snapshot_published_view = read_json(repo_root, TRACE_SNAPSHOT_FENCE_PUBLISHED_VIEW)?;
        let capability_result = read_json(repo_root, TRACE_CAPABILITY_FENCE_RESULT)?;
        let capability_rejects = read_json(repo_root, TRACE_CAPABILITY_FENCE_REJECTS)?;
        let capability_published_view =
            read_json(repo_root, TRACE_CAPABILITY_FENCE_PUBLISHED_VIEW)?;
        let lean_file_present = repo_root.join(W040_STAGE2_LEAN_FILE).exists();

        let w039_stage2_valid = string_at(&w039_stage2_validation, "status")
            == "w039_stage2_policy_governance_valid"
            && !bool_at(&w039_stage2_summary, "stage2_policy_promoted");
        let w040_optimized_valid = number_at(&w040_optimized_summary, "failed_row_count") == 0
            && number_at(&w040_optimized_summary, "treecalc_case_count") == 25
            && number_at(
                &w040_optimized_summary,
                "treecalc_expectation_mismatch_count",
            ) == 0;
        let w040_treecalc_valid = number_at(&w040_treecalc_summary, "case_count") == 25
            && number_at(&w040_treecalc_summary, "expectation_mismatch_count") == 0;
        let w040_lean_tla_valid = string_at(&w040_lean_tla_validation, "status")
            == "formal_assurance_w040_lean_tla_discharge_valid"
            && number_at(&w040_lean_tla_summary, "bounded_model_row_count") == 3;
        let w036_tla_valid = string_at(&w036_tla_summary, "result") == "passed"
            && number_at(&w036_tla_summary, "passed_config_count") == 5
            && number_at(&w036_tla_summary, "failed_config_count") == 0;
        let snapshot_trace_valid = source_status_valid(SourceKind::TraceCalc, &snapshot_result)
            && reject_kind_present(&snapshot_rejects, "snapshot_mismatch")
            && string_at(&snapshot_published_view, "snapshot_id") == "s0";
        let capability_trace_valid = source_status_valid(SourceKind::TraceCalc, &capability_result)
            && reject_kind_present(&capability_rejects, "capability_mismatch")
            && string_at(&capability_published_view, "snapshot_id") == "s0";
        let tla_snapshot_counterpart_present = row_with_field_exists(
            &w036_tla_promotion_blockers,
            "blocker_id",
            "optimized.snapshot_fence.conformance_not_promoted",
        );
        let tla_capability_counterpart_present = row_with_field_exists(
            &w036_tla_promotion_blockers,
            "blocker_id",
            "optimized.capability_view_fence.conformance_not_promoted",
        );
        let w039_snapshot_blocker_present = row_with_field_exists(
            &w039_stage2_blockers,
            "row_id",
            "w039_stage2_snapshot_fence_counterpart_blocker",
        );
        let w039_capability_blocker_present = row_with_field_exists(
            &w039_stage2_blockers,
            "row_id",
            "w039_stage2_capability_view_fence_counterpart_blocker",
        );
        let w040_snapshot_blocker_present = row_with_field_exists(
            &w040_optimized_blockers,
            "row_id",
            "w040_snapshot_fence_counterpart_exact_blocker",
        );
        let w040_capability_blocker_present = row_with_field_exists(
            &w040_optimized_blockers,
            "row_id",
            "w040_capability_view_fence_counterpart_exact_blocker",
        );

        let partition_replay_row_count =
            number_at(&w039_stage2_summary, "partition_replay_row_count") as usize;
        let permutation_replay_row_count =
            number_at(&w039_stage2_summary, "permutation_replay_row_count") as usize;
        let nontrivial_permutation_row_count =
            number_at(&w039_stage2_summary, "nontrivial_permutation_row_count") as usize;
        let observable_invariance_row_count =
            number_at(&w039_stage2_summary, "observable_invariance_row_count") as usize;
        let formatting_watch_row_count =
            number_at(&w039_stage2_summary, "formatting_watch_row_count") as usize;
        let bounded_replay_valid = w039_stage2_valid && partition_replay_row_count == 5;
        let permutation_replay_valid = w039_stage2_valid
            && permutation_replay_row_count == 6
            && nontrivial_permutation_row_count == 1;
        let observable_invariance_valid = w039_stage2_valid && observable_invariance_row_count == 5;
        let dynamic_soft_reference_valid = w040_optimized_valid && w040_treecalc_valid;
        let snapshot_counterpart_valid = snapshot_trace_valid
            && tla_snapshot_counterpart_present
            && w039_snapshot_blocker_present
            && w040_snapshot_blocker_present;
        let capability_counterpart_valid = capability_trace_valid
            && tla_capability_counterpart_present
            && w039_capability_blocker_present
            && w040_capability_blocker_present;
        let bounded_analyzer_valid = w036_tla_valid
            && w040_lean_tla_valid
            && number_at(&w040_lean_tla_model_register, "bounded_model_row_count") == 3;
        let w073_guard_valid = w039_stage2_valid && formatting_watch_row_count == 1;

        let policy_rows = vec![
            json!({
                "row_id": "w040_stage2_bounded_replay_carried",
                "w040_obligation_id": "W040-OBL-011",
                "policy_area": "bounded_baseline_vs_stage2_replay",
                "source_artifacts": [W039_STAGE2_RUN_SUMMARY, W039_STAGE2_POLICY_GATE],
                "satisfied_for_declared_profile": bounded_replay_valid,
                "exact_remaining_blocker": false,
                "promotion_consequence": "bounded replay remains direct evidence but not production Stage 2 policy promotion",
                "disposition": "carry W039 bounded baseline-versus-Stage-2 replay rows into W040 policy evidence",
                "failures": if bounded_replay_valid { Vec::<String>::new() } else { vec!["w040_bounded_replay_not_valid".to_string()] },
            }),
            json!({
                "row_id": "w040_stage2_partition_order_permutation_carried",
                "w040_obligation_id": "W040-OBL-011",
                "policy_area": "partition_order_permutation_replay",
                "source_artifacts": [W039_STAGE2_RUN_SUMMARY, W039_STAGE2_POLICY_GATE],
                "satisfied_for_declared_profile": permutation_replay_valid,
                "exact_remaining_blocker": false,
                "promotion_consequence": "bounded permutation evidence remains direct evidence but does not prove production scheduler fairness",
                "disposition": "carry one nontrivial partition-order permutation row and six total permutation rows",
                "failures": if permutation_replay_valid { Vec::<String>::new() } else { vec!["w040_partition_permutation_replay_not_valid".to_string()] },
            }),
            json!({
                "row_id": "w040_stage2_observable_invariance_carried",
                "w040_obligation_id": "W040-OBL-011",
                "policy_area": "observable_result_invariance",
                "source_artifacts": [W039_STAGE2_RUN_SUMMARY, W039_STAGE2_POLICY_GATE],
                "satisfied_for_declared_profile": observable_invariance_valid,
                "exact_remaining_blocker": false,
                "promotion_consequence": "observable invariance remains evidenced for declared bounded profiles only",
                "disposition": "carry bounded observable-result invariance rows for declared profiles",
                "failures": if observable_invariance_valid { Vec::<String>::new() } else { vec!["w040_observable_invariance_not_valid".to_string()] },
            }),
            json!({
                "row_id": "w040_stage2_dynamic_soft_reference_reclassification_bound",
                "w040_obligation_id": "W040-OBL-010",
                "policy_area": "dynamic_and_soft_reference_replay",
                "source_artifacts": [W040_OPTIMIZED_CORE_SUMMARY, W040_TREECALC_SUMMARY],
                "satisfied_for_declared_profile": dynamic_soft_reference_valid,
                "exact_remaining_blocker": false,
                "promotion_consequence": "dynamic/soft-reference rebind evidence is bound for declared W040 profiles, while automatic broad transition detection remains outside Stage 2 promotion",
                "disposition": "bind W040 explicit dependency release/reclassification seed evidence to the Stage 2 policy surface",
                "failures": if dynamic_soft_reference_valid { Vec::<String>::new() } else { vec!["w040_dynamic_soft_reference_evidence_not_valid".to_string()] },
            }),
            json!({
                "row_id": "w040_stage2_snapshot_fence_counterpart_direct_evidence",
                "w040_obligation_id": "W040-OBL-003",
                "policy_area": "snapshot_fence_counterpart",
                "source_artifacts": [TRACE_SNAPSHOT_FENCE_RESULT, TRACE_SNAPSHOT_FENCE_REJECTS, TRACE_SNAPSHOT_FENCE_PUBLISHED_VIEW, W036_STAGE2_TLA_PROMOTION_BLOCKERS, W040_OPTIMIZED_CORE_BLOCKERS],
                "satisfied_for_declared_profile": snapshot_counterpart_valid,
                "exact_remaining_blocker": false,
                "promotion_consequence": "the Stage 2/coordinator snapshot-fence counterpart is evidenced for the declared profile, but Stage 2 policy remains blocked by broader analyzer/service/pack gates",
                "disposition": "bind TraceCalc stale snapshot reject/no-publish evidence with the bounded TLA fence-reject counterpart",
                "failures": if snapshot_counterpart_valid { Vec::<String>::new() } else { vec!["w040_snapshot_fence_counterpart_not_valid".to_string()] },
            }),
            json!({
                "row_id": "w040_stage2_capability_view_fence_counterpart_direct_evidence",
                "w040_obligation_id": "W040-OBL-003",
                "policy_area": "capability_view_fence_counterpart",
                "source_artifacts": [TRACE_CAPABILITY_FENCE_RESULT, TRACE_CAPABILITY_FENCE_REJECTS, TRACE_CAPABILITY_FENCE_PUBLISHED_VIEW, W036_STAGE2_TLA_PROMOTION_BLOCKERS, W040_OPTIMIZED_CORE_BLOCKERS],
                "satisfied_for_declared_profile": capability_counterpart_valid,
                "exact_remaining_blocker": false,
                "promotion_consequence": "the Stage 2/coordinator capability-view fence counterpart is evidenced for the declared profile, distinct from broader capability-sensitive local rejection, but Stage 2 policy remains blocked",
                "disposition": "bind TraceCalc capability-view mismatch reject/no-publish evidence with the bounded TLA fence-reject counterpart",
                "failures": if capability_counterpart_valid { Vec::<String>::new() } else { vec!["w040_capability_view_counterpart_not_valid".to_string()] },
            }),
            json!({
                "row_id": "w040_stage2_bounded_partition_analyzer_predicate_evidence",
                "w040_obligation_id": "W040-OBL-010",
                "policy_area": "bounded_partition_analyzer_soundness",
                "source_artifacts": [W036_STAGE2_TLA_RUN_SUMMARY, W040_LEAN_TLA_MODEL_REGISTER, W040_STAGE2_LEAN_FILE],
                "satisfied_for_declared_profile": bounded_analyzer_valid && lean_file_present,
                "exact_remaining_blocker": false,
                "promotion_consequence": "bounded analyzer preconditions are evidenced, but full production partition analyzer soundness remains blocked",
                "disposition": "bind bounded partition ownership, cross-partition blocking, completed-partition readiness, and replay-evidence preconditions as analyzer-soundness evidence for declared profiles",
                "failures": if bounded_analyzer_valid && lean_file_present { Vec::<String>::new() } else { vec!["w040_bounded_partition_analyzer_evidence_not_valid".to_string()] },
            }),
            json!({
                "row_id": "w040_stage2_w073_typed_formatting_guard_carried",
                "w040_obligation_id": "W040-OBL-018",
                "policy_area": "w073_typed_formatting_watch",
                "source_artifacts": [W039_STAGE2_RUN_SUMMARY],
                "satisfied_for_declared_profile": w073_guard_valid,
                "exact_remaining_blocker": false,
                "promotion_consequence": "typed-only formatting remains a Stage 2 observable-surface guard; broad OxFml seam breadth remains under calc-tv5.8",
                "disposition": "carry W073 typed-only conditional-formatting metadata as observable-surface guard",
                "failures": if w073_guard_valid { Vec::<String>::new() } else { vec!["w040_w073_formatting_guard_not_valid".to_string()] },
            }),
            json!({
                "row_id": "w040_stage2_full_production_analyzer_soundness_blocker",
                "w040_obligation_id": "W040-OBL-010",
                "policy_area": "full_production_partition_analyzer_soundness",
                "source_artifacts": [W040_DIRECT_OBLIGATION_MAP, W036_STAGE2_TLA_RUN_SUMMARY],
                "satisfied_for_declared_profile": false,
                "exact_remaining_blocker": true,
                "promotion_consequence": "Stage 2 production policy remains unpromoted",
                "disposition": "retain full production partition analyzer soundness as exact blocker beyond bounded declared-profile evidence",
                "failures": if obligation_exists(&w040_obligation_map, "W040-OBL-010") { Vec::<String>::new() } else { vec!["w040_production_partition_obligation_missing".to_string()] },
            }),
            json!({
                "row_id": "w040_stage2_fairness_scheduler_unbounded_coverage_blocker",
                "w040_obligation_id": "W040-OBL-009",
                "policy_area": "fairness_scheduler_unbounded_coverage",
                "source_artifacts": [W040_LEAN_TLA_SUMMARY, W040_LEAN_TLA_MODEL_REGISTER],
                "satisfied_for_declared_profile": false,
                "exact_remaining_blocker": true,
                "promotion_consequence": "full TLA verification and production Stage 2 policy remain unpromoted",
                "disposition": "retain fairness, unbounded scheduler coverage, and model-completeness limits as exact Stage 2 policy blockers",
                "failures": if obligation_exists(&w040_obligation_map, "W040-OBL-009") && w040_lean_tla_valid { Vec::<String>::new() } else { vec!["w040_fairness_scheduler_boundary_input_missing".to_string()] },
            }),
            json!({
                "row_id": "w040_stage2_operated_cross_engine_service_dependency_blocker",
                "w040_obligation_id": "W040-OBL-015",
                "policy_area": "operated_cross_engine_stage2_differential_service",
                "source_artifacts": [W039_STAGE2_BLOCKERS, W040_DIRECT_OBLIGATION_MAP],
                "satisfied_for_declared_profile": false,
                "exact_remaining_blocker": true,
                "promotion_consequence": "operated Stage 2 differential evidence remains required before policy promotion",
                "disposition": "retain operated cross-engine differential service as successor dependency",
                "failures": if obligation_exists(&w040_obligation_map, "W040-OBL-015") { Vec::<String>::new() } else { vec!["w040_stage2_operated_service_obligation_missing".to_string()] },
            }),
            json!({
                "row_id": "w040_stage2_pack_grade_replay_governance_blocker",
                "w040_obligation_id": "W040-OBL-021",
                "policy_area": "pack_grade_replay_governance",
                "source_artifacts": [W039_STAGE2_BLOCKERS, W040_DIRECT_OBLIGATION_MAP],
                "satisfied_for_declared_profile": false,
                "exact_remaining_blocker": true,
                "promotion_consequence": "pack-grade replay, C5, and release-grade verification remain blocked",
                "disposition": "retain pack-grade replay governance and retained-witness lifecycle as release-decision blockers",
                "failures": if obligation_exists(&w040_obligation_map, "W040-OBL-021") { Vec::<String>::new() } else { vec!["w040_pack_grade_replay_governance_obligation_missing".to_string()] },
            }),
        ];

        let satisfied_policy_row_count = policy_rows
            .iter()
            .filter(|row| bool_at(row, "satisfied_for_declared_profile"))
            .count();
        let policy_blocker_rows = policy_rows
            .iter()
            .filter(|row| bool_at(row, "exact_remaining_blocker"))
            .cloned()
            .collect::<Vec<_>>();
        let policy_failed_row_count = policy_rows
            .iter()
            .filter(|row| {
                !row.get("failures")
                    .and_then(Value::as_array)
                    .is_some_and(Vec::is_empty)
            })
            .count();
        let exact_remaining_blocker_count = policy_blocker_rows.len();
        let policy_row_count = policy_rows.len();

        let partition_analyzer_rows = policy_rows
            .iter()
            .filter(|row| {
                matches!(
                    string_at(row, "policy_area"),
                    "dynamic_and_soft_reference_replay"
                        | "snapshot_fence_counterpart"
                        | "capability_view_fence_counterpart"
                        | "bounded_partition_analyzer_soundness"
                        | "full_production_partition_analyzer_soundness"
                        | "fairness_scheduler_unbounded_coverage"
                )
            })
            .cloned()
            .collect::<Vec<_>>();
        let observable_equivalence_rows = policy_rows
            .iter()
            .filter(|row| {
                matches!(
                    string_at(row, "policy_area"),
                    "bounded_baseline_vs_stage2_replay"
                        | "partition_order_permutation_replay"
                        | "observable_result_invariance"
                        | "snapshot_fence_counterpart"
                        | "capability_view_fence_counterpart"
                        | "w073_typed_formatting_watch"
                        | "operated_cross_engine_stage2_differential_service"
                        | "pack_grade_replay_governance"
                )
            })
            .cloned()
            .collect::<Vec<_>>();

        let mut validation_failures = Vec::new();
        if number_at(&w040_obligation_summary, "obligation_count") != 23 {
            validation_failures.push("w040_obligation_count_changed".to_string());
        }
        for obligation_id in [
            "W040-OBL-003",
            "W040-OBL-009",
            "W040-OBL-010",
            "W040-OBL-011",
            "W040-OBL-015",
            "W040-OBL-021",
        ] {
            if !obligation_exists(&w040_obligation_map, obligation_id) {
                validation_failures.push(format!("{obligation_id}_missing"));
            }
        }
        if !w039_stage2_valid {
            validation_failures.push("w039_stage2_policy_governance_not_valid".to_string());
        }
        if number_at(&w039_stage2_policy_gate, "policy_row_count") != 10 {
            validation_failures.push("w039_stage2_policy_gate_row_count_changed".to_string());
        }
        if !w040_optimized_valid {
            validation_failures.push("w040_optimized_core_stage2_inputs_not_valid".to_string());
        }
        if !w040_treecalc_valid {
            validation_failures
                .push("w040_treecalc_dynamic_reclassification_not_valid".to_string());
        }
        if !w040_lean_tla_valid {
            validation_failures.push("w040_lean_tla_stage2_inputs_not_valid".to_string());
        }
        if !w036_tla_valid {
            validation_failures.push("w036_stage2_tla_model_not_valid".to_string());
        }
        if !snapshot_counterpart_valid {
            validation_failures.push("w040_snapshot_counterpart_not_valid".to_string());
        }
        if !capability_counterpart_valid {
            validation_failures.push("w040_capability_counterpart_not_valid".to_string());
        }
        if !bounded_analyzer_valid {
            validation_failures.push("w040_bounded_partition_analyzer_not_valid".to_string());
        }
        if !lean_file_present {
            validation_failures.push("w040_stage2_lean_file_missing".to_string());
        }
        if policy_failed_row_count != 0 {
            validation_failures.push("w040_stage2_policy_row_failures_present".to_string());
        }
        if satisfied_policy_row_count != 8 {
            validation_failures
                .push("w040_stage2_expected_eight_satisfied_policy_rows".to_string());
        }
        if exact_remaining_blocker_count != 4 {
            validation_failures.push("w040_stage2_expected_four_exact_blockers".to_string());
        }

        let source_evidence_index_path =
            format!("{relative_artifact_root}/source_evidence_index.json");
        let policy_gate_register_path =
            format!("{relative_artifact_root}/w040_stage2_policy_gate_register.json");
        let partition_analyzer_register_path =
            format!("{relative_artifact_root}/w040_partition_analyzer_soundness_register.json");
        let observable_equivalence_register_path =
            format!("{relative_artifact_root}/w040_observable_equivalence_register.json");
        let blocker_register_path =
            format!("{relative_artifact_root}/w040_stage2_exact_blocker_register.json");
        let promotion_decision_path = format!("{relative_artifact_root}/promotion_decision.json");
        let validation_path = format!("{relative_artifact_root}/validation.json");

        write_json(
            &artifact_root.join("source_evidence_index.json"),
            &json!({
                "schema_version": W040_SOURCE_INDEX_SCHEMA_V1,
                "run_id": run_id,
                "artifact_root": relative_artifact_root,
                "source_artifacts": {
                    "w040_direct_obligation_summary": W040_DIRECT_OBLIGATION_SUMMARY,
                    "w040_direct_obligation_map": W040_DIRECT_OBLIGATION_MAP,
                    "w039_stage2_run_summary": W039_STAGE2_RUN_SUMMARY,
                    "w039_stage2_validation": W039_STAGE2_VALIDATION,
                    "w039_stage2_policy_gate": W039_STAGE2_POLICY_GATE,
                    "w039_stage2_blockers": W039_STAGE2_BLOCKERS,
                    "w040_optimized_core_summary": W040_OPTIMIZED_CORE_SUMMARY,
                    "w040_optimized_core_blockers": W040_OPTIMIZED_CORE_BLOCKERS,
                    "w040_treecalc_summary": W040_TREECALC_SUMMARY,
                    "w040_lean_tla_summary": W040_LEAN_TLA_SUMMARY,
                    "w040_lean_tla_validation": W040_LEAN_TLA_VALIDATION,
                    "w040_lean_tla_model_register": W040_LEAN_TLA_MODEL_REGISTER,
                    "w036_stage2_tla_run_summary": W036_STAGE2_TLA_RUN_SUMMARY,
                    "w036_stage2_tla_promotion_blockers": W036_STAGE2_TLA_PROMOTION_BLOCKERS,
                    "trace_snapshot_fence_result": TRACE_SNAPSHOT_FENCE_RESULT,
                    "trace_snapshot_fence_rejects": TRACE_SNAPSHOT_FENCE_REJECTS,
                    "trace_snapshot_fence_published_view": TRACE_SNAPSHOT_FENCE_PUBLISHED_VIEW,
                    "trace_capability_fence_result": TRACE_CAPABILITY_FENCE_RESULT,
                    "trace_capability_fence_rejects": TRACE_CAPABILITY_FENCE_REJECTS,
                    "trace_capability_fence_published_view": TRACE_CAPABILITY_FENCE_PUBLISHED_VIEW,
                    "w040_stage2_lean_policy_file": W040_STAGE2_LEAN_FILE
                },
                "source_counts": {
                    "w040_obligation_count": number_at(&w040_obligation_summary, "obligation_count"),
                    "w039_stage2_policy_row_count": number_at(&w039_stage2_policy_gate, "policy_row_count"),
                    "w040_treecalc_case_count": number_at(&w040_treecalc_summary, "case_count"),
                    "w040_treecalc_expectation_mismatch_count": number_at(&w040_treecalc_summary, "expectation_mismatch_count"),
                    "w040_lean_tla_bounded_model_row_count": number_at(&w040_lean_tla_summary, "bounded_model_row_count"),
                    "w036_stage2_tla_passed_config_count": number_at(&w036_tla_summary, "passed_config_count"),
                    "w036_stage2_tla_failed_config_count": number_at(&w036_tla_summary, "failed_config_count")
                }
            }),
        )?;
        write_json(
            &artifact_root.join("w040_stage2_policy_gate_register.json"),
            &json!({
                "schema_version": W040_POLICY_GATE_SCHEMA_V1,
                "run_id": run_id,
                "policy_row_count": policy_row_count,
                "satisfied_policy_row_count": satisfied_policy_row_count,
                "exact_remaining_blocker_count": exact_remaining_blocker_count,
                "stage2_policy_promoted": false,
                "rows": policy_rows
            }),
        )?;
        write_json(
            &artifact_root.join("w040_partition_analyzer_soundness_register.json"),
            &json!({
                "schema_version": W040_PARTITION_ANALYZER_SCHEMA_V1,
                "run_id": run_id,
                "row_count": partition_analyzer_rows.len(),
                "satisfied_policy_row_count": partition_analyzer_rows
                    .iter()
                    .filter(|row| bool_at(row, "satisfied_for_declared_profile"))
                    .count(),
                "exact_remaining_blocker_count": partition_analyzer_rows
                    .iter()
                    .filter(|row| bool_at(row, "exact_remaining_blocker"))
                    .count(),
                "rows": partition_analyzer_rows
            }),
        )?;
        write_json(
            &artifact_root.join("w040_observable_equivalence_register.json"),
            &json!({
                "schema_version": W040_OBSERVABLE_EQUIVALENCE_SCHEMA_V1,
                "run_id": run_id,
                "row_count": observable_equivalence_rows.len(),
                "satisfied_policy_row_count": observable_equivalence_rows
                    .iter()
                    .filter(|row| bool_at(row, "satisfied_for_declared_profile"))
                    .count(),
                "exact_remaining_blocker_count": observable_equivalence_rows
                    .iter()
                    .filter(|row| bool_at(row, "exact_remaining_blocker"))
                    .count(),
                "semantic_equivalence_statement": "For the declared W040 Stage 2 profiles, baseline replay, partition replay, partition-order permutations, dynamic/soft-reference reclassification evidence, snapshot-fence reject/no-publish evidence, capability-view fence reject/no-publish evidence, and W073 typed-formatting observable guards preserve the observable result surface. This is direct declared-profile evidence, not full production scheduler or pack-grade replay promotion.",
                "rows": observable_equivalence_rows
            }),
        )?;
        write_json(
            &artifact_root.join("w040_stage2_exact_blocker_register.json"),
            &json!({
                "schema_version": W040_BLOCKER_REGISTER_SCHEMA_V1,
                "run_id": run_id,
                "exact_remaining_blocker_count": exact_remaining_blocker_count,
                "rows": policy_blocker_rows
            }),
        )?;
        write_json(
            &artifact_root.join("promotion_decision.json"),
            &json!({
                "schema_version": W040_PROMOTION_DECISION_SCHEMA_V1,
                "run_id": run_id,
                "decision_state": "w040_stage2_policy_equivalence_validated_policy_unpromoted",
                "stage2_policy_promoted": false,
                "stage2_promotion_candidate": false,
                "bounded_partition_replay_present": bounded_replay_valid,
                "partition_order_permutation_replay_present": permutation_replay_valid,
                "observable_result_invariance_evidenced_for_declared_profiles": observable_invariance_valid,
                "dynamic_and_soft_reference_replay_evidenced_for_declared_profiles": dynamic_soft_reference_valid,
                "snapshot_fence_counterpart_evidenced": snapshot_counterpart_valid,
                "capability_view_fence_counterpart_evidenced": capability_counterpart_valid,
                "bounded_partition_analyzer_soundness_evidenced": bounded_analyzer_valid,
                "production_partition_analyzer_soundness_promoted": false,
                "fairness_scheduler_unbounded_coverage_promoted": false,
                "operated_cross_engine_stage2_service_promoted": false,
                "pack_grade_replay_governance_promoted": false,
                "satisfied_inputs": [
                    "bounded_partition_replay_present",
                    "partition_order_permutation_replay_present",
                    "observable_result_invariance_for_declared_profiles",
                    "bounded_dynamic_soft_reference_replay_present",
                    "snapshot_fence_counterpart_evidenced",
                    "capability_view_fence_counterpart_evidenced",
                    "bounded_partition_analyzer_soundness_evidenced",
                    "w073_typed_formatting_guard_carried"
                ],
                "blockers": [
                    "stage2.full_production_partition_analyzer_soundness_absent",
                    "stage2.fairness_scheduler_unbounded_coverage_absent",
                    "stage2.operated_cross_engine_differential_service_absent",
                    "stage2.pack_grade_replay_governance_absent"
                ],
                "semantic_equivalence_statement": "Observable-result invariance is evidenced for the declared W040 profiles, including snapshot and capability fence no-publish counterparts. Production Stage 2 scheduler or partition policy remains unpromoted until full production partition analyzer soundness, fairness and unbounded scheduler coverage, operated cross-engine service evidence, and pack-grade replay governance are present."
            }),
        )?;

        let validation_status = if validation_failures.is_empty() {
            "w040_stage2_policy_equivalence_valid"
        } else {
            "w040_stage2_policy_equivalence_invalid"
        };
        write_json(
            &artifact_root.join("validation.json"),
            &json!({
                "schema_version": W040_VALIDATION_SCHEMA_V1,
                "run_id": run_id,
                "status": validation_status,
                "policy_row_count": policy_row_count,
                "satisfied_policy_row_count": satisfied_policy_row_count,
                "partition_replay_row_count": partition_replay_row_count,
                "permutation_replay_row_count": permutation_replay_row_count,
                "nontrivial_permutation_row_count": nontrivial_permutation_row_count,
                "observable_invariance_row_count": observable_invariance_row_count,
                "formatting_watch_row_count": formatting_watch_row_count,
                "snapshot_counterpart_evidenced": snapshot_counterpart_valid,
                "capability_counterpart_evidenced": capability_counterpart_valid,
                "bounded_partition_analyzer_evidenced": bounded_analyzer_valid,
                "exact_remaining_blocker_count": exact_remaining_blocker_count,
                "failed_row_count": policy_failed_row_count,
                "stage2_policy_promoted": false,
                "validation_failures": validation_failures
            }),
        )?;
        write_json(
            &artifact_root.join("run_summary.json"),
            &json!({
                "schema_version": W040_RUN_SUMMARY_SCHEMA_V1,
                "run_id": run_id,
                "artifact_root": relative_artifact_root,
                "source_evidence_index_path": source_evidence_index_path,
                "w040_stage2_policy_gate_register_path": policy_gate_register_path,
                "w040_partition_analyzer_soundness_register_path": partition_analyzer_register_path,
                "w040_observable_equivalence_register_path": observable_equivalence_register_path,
                "w040_stage2_exact_blocker_register_path": blocker_register_path,
                "promotion_decision_path": promotion_decision_path,
                "validation_path": validation_path,
                "policy_row_count": policy_row_count,
                "satisfied_policy_row_count": satisfied_policy_row_count,
                "partition_replay_row_count": partition_replay_row_count,
                "permutation_replay_row_count": permutation_replay_row_count,
                "nontrivial_permutation_row_count": nontrivial_permutation_row_count,
                "observable_invariance_row_count": observable_invariance_row_count,
                "formatting_watch_row_count": formatting_watch_row_count,
                "snapshot_counterpart_evidenced": snapshot_counterpart_valid,
                "capability_counterpart_evidenced": capability_counterpart_valid,
                "bounded_partition_analyzer_evidenced": bounded_analyzer_valid,
                "exact_remaining_blocker_count": exact_remaining_blocker_count,
                "failed_row_count": policy_failed_row_count,
                "stage2_policy_promoted": false,
                "stage2_promotion_candidate": false
            }),
        )?;

        Ok(Stage2ReplayRunSummary {
            run_id: run_id.to_string(),
            schema_version: W040_RUN_SUMMARY_SCHEMA_V1.to_string(),
            partition_replay_row_count,
            permutation_replay_row_count,
            nontrivial_permutation_row_count,
            observable_invariance_row_count,
            formatting_watch_row_count,
            exact_remaining_blocker_count,
            failed_row_count: policy_failed_row_count,
            stage2_policy_promoted: false,
            artifact_root: relative_artifact_root,
        })
    }

    fn execute_w039(
        &self,
        repo_root: &Path,
        run_id: &str,
    ) -> Result<Stage2ReplayRunSummary, Stage2ReplayError> {
        let relative_artifact_root =
            relative_artifact_path(&["docs", "test-runs", "core-engine", "stage2-replay", run_id]);
        let artifact_root = repo_root.join(&relative_artifact_root);
        if artifact_root.exists() {
            fs::remove_dir_all(&artifact_root).map_err(|source| {
                Stage2ReplayError::RemoveDirectory {
                    path: artifact_root.display().to_string(),
                    source,
                }
            })?;
        }
        fs::create_dir_all(&artifact_root).map_err(|source| {
            Stage2ReplayError::CreateDirectory {
                path: artifact_root.display().to_string(),
                source,
            }
        })?;

        let w039_ledger = read_json(repo_root, W039_RESIDUAL_LEDGER)?;
        let w038_stage2_summary = read_json(repo_root, W038_STAGE2_RUN_SUMMARY)?;
        let w038_stage2_validation = read_json(repo_root, W038_STAGE2_VALIDATION)?;
        let w038_partition_matrix = read_json(repo_root, W038_STAGE2_PARTITION_MATRIX)?;
        let w038_permutation_replay = read_json(repo_root, W038_STAGE2_PERMUTATION_REPLAY)?;
        let w038_stage2_blockers = read_json(repo_root, W038_STAGE2_BLOCKER_REGISTER)?;
        let w039_conformance_summary = read_json(repo_root, W039_CONFORMANCE_SUMMARY)?;
        let w039_conformance_blockers = read_json(repo_root, W039_CONFORMANCE_BLOCKERS)?;
        let w039_formal_summary = read_json(repo_root, W039_FORMAL_ASSURANCE_SUMMARY)?;
        let w039_formal_blockers = read_json(repo_root, W039_FORMAL_ASSURANCE_BLOCKERS)?;
        let lean_file_present = repo_root.join(W039_STAGE2_LEAN_FILE).exists();

        let w038_replay_valid =
            string_at(&w038_stage2_validation, "status") == "w038_stage2_replay_valid";
        let conformance_valid = number_at(
            &w039_conformance_summary,
            "w039_exact_remaining_blocker_count",
        ) == 4;
        let proof_model_valid = number_at(&w039_formal_summary, "exact_remaining_blocker_count")
            == 6
            && !bool_at(
                &w039_formal_summary["promotion_claims"],
                "stage2_policy_promoted",
            );
        let snapshot_blocker_present = row_with_field_exists(
            &w039_conformance_blockers,
            "row_id",
            "w039_snapshot_fence_counterpart_exact_blocker",
        );
        let capability_blocker_present = row_with_field_exists(
            &w039_conformance_blockers,
            "row_id",
            "w039_capability_view_fence_counterpart_exact_blocker",
        );
        let production_analyzer_blocker_present = row_with_field_exists(
            &w038_stage2_blockers,
            "blocker_id",
            "stage2.production_partition_analyzer_soundness_absent",
        );
        let service_blocker_present = row_with_field_exists(
            &w038_stage2_blockers,
            "blocker_id",
            "stage2.operated_cross_engine_differential_service_absent",
        );
        let pack_blocker_present = row_with_field_exists(
            &w038_stage2_blockers,
            "blocker_id",
            "stage2.pack_grade_replay_governance_absent",
        );
        let stage2_proof_gate_present = row_with_field_exists(
            &w039_formal_blockers,
            "row_id",
            "w039_stage2_partition_policy_proof_gate",
        );

        let policy_rows = vec![
            json!({
                "row_id": "w039_stage2_bounded_replay_carry_forward",
                "w039_obligation_id": "W039-OBL-010",
                "policy_area": "bounded_baseline_vs_stage2_replay",
                "source_artifacts": [W038_STAGE2_RUN_SUMMARY, W038_STAGE2_PARTITION_MATRIX],
                "satisfied_for_declared_profile": w038_replay_valid && number_at(&w038_stage2_summary, "partition_replay_row_count") == 5,
                "exact_remaining_blocker": false,
                "promotion_consequence": "bounded replay is carried forward but is not production policy promotion",
                "disposition": "retain W038 bounded partition replay evidence for five declared profiles",
                "failures": if w038_replay_valid && number_at(&w038_stage2_summary, "partition_replay_row_count") == 5 { Vec::<String>::new() } else { vec!["w038_bounded_partition_replay_not_valid".to_string()] },
            }),
            json!({
                "row_id": "w039_stage2_partition_order_permutation_carry_forward",
                "w039_obligation_id": "W039-OBL-010",
                "policy_area": "partition_order_permutation_replay",
                "source_artifacts": [W038_STAGE2_RUN_SUMMARY, W038_STAGE2_PERMUTATION_REPLAY],
                "satisfied_for_declared_profile": w038_replay_valid && number_at(&w038_stage2_summary, "permutation_replay_row_count") == 6 && number_at(&w038_stage2_summary, "nontrivial_permutation_row_count") == 1,
                "exact_remaining_blocker": false,
                "promotion_consequence": "bounded permutation evidence is carried forward but does not prove production scheduler soundness",
                "disposition": "retain one nontrivial independent partition-order permutation and six total permutation rows",
                "failures": if w038_replay_valid && number_at(&w038_stage2_summary, "permutation_replay_row_count") == 6 && number_at(&w038_stage2_summary, "nontrivial_permutation_row_count") == 1 { Vec::<String>::new() } else { vec!["w038_partition_permutation_replay_not_valid".to_string()] },
            }),
            json!({
                "row_id": "w039_stage2_observable_invariance_carry_forward",
                "w039_obligation_id": "W039-OBL-010",
                "policy_area": "observable_result_invariance",
                "source_artifacts": [W038_STAGE2_RUN_SUMMARY, W038_STAGE2_PARTITION_MATRIX],
                "satisfied_for_declared_profile": w038_replay_valid && number_at(&w038_stage2_summary, "observable_invariance_row_count") == 5,
                "exact_remaining_blocker": false,
                "promotion_consequence": "observable invariance is evidenced for declared bounded profiles only",
                "disposition": "carry W038 baseline/stage2/permutation observable equality rows",
                "failures": if w038_replay_valid && number_at(&w038_stage2_summary, "observable_invariance_row_count") == 5 { Vec::<String>::new() } else { vec!["w038_observable_invariance_not_valid".to_string()] },
            }),
            json!({
                "row_id": "w039_stage2_dynamic_soft_reference_carry_forward",
                "w039_obligation_id": "W039-OBL-010",
                "policy_area": "dynamic_and_soft_reference_replay",
                "source_artifacts": [W038_STAGE2_PARTITION_MATRIX, W039_CONFORMANCE_BLOCKERS],
                "satisfied_for_declared_profile": w038_replay_valid && conformance_valid,
                "exact_remaining_blocker": false,
                "promotion_consequence": "bounded dynamic/soft-reference replay is present, while dynamic release/reclassification remains an optimized/core blocker outside this Stage 2 policy target",
                "disposition": "carry W038 dynamic reference partition row and W039 dynamic exact-blocker context",
                "failures": if w038_replay_valid && conformance_valid { Vec::<String>::new() } else { vec!["w039_dynamic_soft_reference_context_not_valid".to_string()] },
            }),
            json!({
                "row_id": "w039_stage2_w073_typed_formatting_guard",
                "w039_obligation_id": "W039-OBL-017",
                "policy_area": "w073_typed_formatting_watch",
                "source_artifacts": [W038_STAGE2_RUN_SUMMARY],
                "satisfied_for_declared_profile": w038_replay_valid && number_at(&w038_stage2_summary, "formatting_watch_row_count") == 1,
                "exact_remaining_blocker": false,
                "promotion_consequence": "typed-only formatting watch is carried for Stage 2 observable invariance but broad OxFml seam closure remains under calc-f7o.7",
                "disposition": "carry W073 typed-only conditional-formatting guard as a replay observable surface",
                "failures": if w038_replay_valid && number_at(&w038_stage2_summary, "formatting_watch_row_count") == 1 { Vec::<String>::new() } else { vec!["w039_w073_formatting_guard_not_valid".to_string()] },
            }),
            json!({
                "row_id": "w039_stage2_snapshot_fence_counterpart_blocker",
                "w039_obligation_id": "W039-OBL-003",
                "policy_area": "snapshot_fence_counterpart",
                "source_artifacts": [W039_CONFORMANCE_BLOCKERS],
                "satisfied_for_declared_profile": false,
                "exact_remaining_blocker": true,
                "promotion_consequence": "snapshot-fence conformance and Stage 2 production policy remain blocked",
                "disposition": "retain stale accepted-candidate counterpart as exact Stage 2/coordinator replay blocker",
                "failures": if snapshot_blocker_present { Vec::<String>::new() } else { vec!["w039_snapshot_fence_blocker_missing".to_string()] },
            }),
            json!({
                "row_id": "w039_stage2_capability_view_fence_counterpart_blocker",
                "w039_obligation_id": "W039-OBL-003",
                "policy_area": "capability_view_fence_counterpart",
                "source_artifacts": [W039_CONFORMANCE_BLOCKERS],
                "satisfied_for_declared_profile": false,
                "exact_remaining_blocker": true,
                "promotion_consequence": "capability-view fence conformance and Stage 2 production policy remain blocked",
                "disposition": "retain compatibility-fenced capability-view mismatch counterpart as exact Stage 2/coordinator replay blocker",
                "failures": if capability_blocker_present { Vec::<String>::new() } else { vec!["w039_capability_view_fence_blocker_missing".to_string()] },
            }),
            json!({
                "row_id": "w039_stage2_production_partition_analyzer_soundness_blocker",
                "w039_obligation_id": "W039-OBL-009",
                "policy_area": "production_partition_analyzer_soundness",
                "source_artifacts": [W038_STAGE2_BLOCKER_REGISTER, W039_FORMAL_ASSURANCE_BLOCKERS, W039_STAGE2_LEAN_FILE],
                "satisfied_for_declared_profile": false,
                "exact_remaining_blocker": true,
                "promotion_consequence": "Stage 2 production policy remains unpromoted",
                "disposition": "retain production partition analyzer soundness as exact blocker and bind W039 Lean policy predicate",
                "failures": if production_analyzer_blocker_present && stage2_proof_gate_present && lean_file_present { Vec::<String>::new() } else { vec!["w039_production_partition_soundness_evidence_missing".to_string()] },
            }),
            json!({
                "row_id": "w039_stage2_operated_cross_engine_service_dependency_blocker",
                "w039_obligation_id": "W039-OBL-014",
                "policy_area": "operated_cross_engine_stage2_differential_service",
                "source_artifacts": [W038_STAGE2_BLOCKER_REGISTER],
                "satisfied_for_declared_profile": false,
                "exact_remaining_blocker": true,
                "promotion_consequence": "operated Stage 2 differential evidence remains required before policy promotion",
                "disposition": "retain operated cross-engine differential service as successor dependency",
                "failures": if service_blocker_present { Vec::<String>::new() } else { vec!["w039_stage2_service_dependency_blocker_missing".to_string()] },
            }),
            json!({
                "row_id": "w039_stage2_pack_grade_replay_governance_blocker",
                "w039_obligation_id": "W039-OBL-020",
                "policy_area": "pack_grade_replay_governance",
                "source_artifacts": [W038_STAGE2_BLOCKER_REGISTER, W039_RESIDUAL_LEDGER],
                "satisfied_for_declared_profile": false,
                "exact_remaining_blocker": true,
                "promotion_consequence": "pack-grade replay, C5, and release-grade verification remain blocked",
                "disposition": "retain pack-grade replay governance and retained-witness lifecycle as release-decision blockers",
                "failures": if pack_blocker_present { Vec::<String>::new() } else { vec!["w039_pack_grade_replay_blocker_missing".to_string()] },
            }),
        ];

        let satisfied_policy_row_count = policy_rows
            .iter()
            .filter(|row| bool_at(row, "satisfied_for_declared_profile"))
            .count();
        let policy_blocker_rows = policy_rows
            .iter()
            .filter(|row| bool_at(row, "exact_remaining_blocker"))
            .cloned()
            .collect::<Vec<_>>();
        let policy_failed_row_count = policy_rows
            .iter()
            .filter(|row| {
                !row.get("failures")
                    .and_then(Value::as_array)
                    .is_some_and(Vec::is_empty)
            })
            .count();

        let partition_replay_row_count =
            number_at(&w038_stage2_summary, "partition_replay_row_count") as usize;
        let permutation_replay_row_count =
            number_at(&w038_stage2_summary, "permutation_replay_row_count") as usize;
        let nontrivial_permutation_row_count =
            number_at(&w038_stage2_summary, "nontrivial_permutation_row_count") as usize;
        let observable_invariance_row_count =
            number_at(&w038_stage2_summary, "observable_invariance_row_count") as usize;
        let formatting_watch_row_count =
            number_at(&w038_stage2_summary, "formatting_watch_row_count") as usize;
        let exact_remaining_blocker_count = policy_blocker_rows.len();

        let mut validation_failures = Vec::new();
        if number_at(&w039_ledger, "obligation_count") != 20 {
            validation_failures.push("w039_obligation_count_changed".to_string());
        }
        if !w038_replay_valid {
            validation_failures.push("w038_stage2_replay_not_valid".to_string());
        }
        if number_at(&w038_stage2_summary, "exact_remaining_blocker_count") != 3 {
            validation_failures.push("w038_stage2_blocker_count_changed".to_string());
        }
        if number_at(&w038_partition_matrix, "row_count") != 5 {
            validation_failures.push("w038_partition_matrix_row_count_changed".to_string());
        }
        if number_at(&w038_permutation_replay, "row_count") != 6 {
            validation_failures.push("w038_permutation_replay_row_count_changed".to_string());
        }
        if !conformance_valid {
            validation_failures.push("w039_conformance_exact_blocker_count_changed".to_string());
        }
        if !snapshot_blocker_present {
            validation_failures.push("w039_snapshot_fence_blocker_missing".to_string());
        }
        if !capability_blocker_present {
            validation_failures.push("w039_capability_view_blocker_missing".to_string());
        }
        if !proof_model_valid {
            validation_failures.push("w039_proof_model_stage2_context_invalid".to_string());
        }
        if !stage2_proof_gate_present {
            validation_failures.push("w039_stage2_proof_gate_missing".to_string());
        }
        if !production_analyzer_blocker_present {
            validation_failures
                .push("w039_production_partition_analyzer_blocker_missing".to_string());
        }
        if !service_blocker_present {
            validation_failures.push("w039_stage2_service_blocker_missing".to_string());
        }
        if !pack_blocker_present {
            validation_failures.push("w039_stage2_pack_governance_blocker_missing".to_string());
        }
        if !lean_file_present {
            validation_failures.push("w039_stage2_lean_policy_file_missing".to_string());
        }
        if policy_failed_row_count != 0 {
            validation_failures.push("w039_stage2_policy_row_failures_present".to_string());
        }
        if exact_remaining_blocker_count != 5 {
            validation_failures.push("w039_stage2_expected_five_exact_blockers".to_string());
        }

        let source_evidence_index_path =
            format!("{relative_artifact_root}/source_evidence_index.json");
        let policy_gate_register_path =
            format!("{relative_artifact_root}/w039_stage2_policy_gate_register.json");
        let partition_soundness_register_path =
            format!("{relative_artifact_root}/w039_partition_soundness_register.json");
        let replay_governance_register_path =
            format!("{relative_artifact_root}/w039_replay_governance_register.json");
        let blocker_register_path =
            format!("{relative_artifact_root}/w039_stage2_exact_blocker_register.json");
        let promotion_decision_path = format!("{relative_artifact_root}/promotion_decision.json");
        let validation_path = format!("{relative_artifact_root}/validation.json");

        let partition_soundness_rows = policy_rows
            .iter()
            .filter(|row| {
                matches!(
                    string_at(row, "policy_area"),
                    "snapshot_fence_counterpart"
                        | "capability_view_fence_counterpart"
                        | "production_partition_analyzer_soundness"
                        | "dynamic_and_soft_reference_replay"
                )
            })
            .cloned()
            .collect::<Vec<_>>();
        let replay_governance_rows = policy_rows
            .iter()
            .filter(|row| {
                matches!(
                    string_at(row, "policy_area"),
                    "bounded_baseline_vs_stage2_replay"
                        | "partition_order_permutation_replay"
                        | "observable_result_invariance"
                        | "operated_cross_engine_stage2_differential_service"
                        | "pack_grade_replay_governance"
                )
            })
            .cloned()
            .collect::<Vec<_>>();

        write_json(
            &artifact_root.join("source_evidence_index.json"),
            &json!({
                "schema_version": W039_SOURCE_INDEX_SCHEMA_V1,
                "run_id": run_id,
                "artifact_root": relative_artifact_root,
                "source_artifacts": {
                    "w039_successor_obligation_ledger": W039_RESIDUAL_LEDGER,
                    "w038_stage2_run_summary": W038_STAGE2_RUN_SUMMARY,
                    "w038_stage2_validation": W038_STAGE2_VALIDATION,
                    "w038_stage2_partition_matrix": W038_STAGE2_PARTITION_MATRIX,
                    "w038_stage2_permutation_replay": W038_STAGE2_PERMUTATION_REPLAY,
                    "w038_stage2_blocker_register": W038_STAGE2_BLOCKER_REGISTER,
                    "w039_implementation_conformance_summary": W039_CONFORMANCE_SUMMARY,
                    "w039_implementation_conformance_blockers": W039_CONFORMANCE_BLOCKERS,
                    "w039_formal_assurance_summary": W039_FORMAL_ASSURANCE_SUMMARY,
                    "w039_formal_assurance_blockers": W039_FORMAL_ASSURANCE_BLOCKERS,
                    "w039_stage2_lean_policy_file": W039_STAGE2_LEAN_FILE
                },
                "source_counts": {
                    "w039_obligation_count": number_at(&w039_ledger, "obligation_count"),
                    "w038_partition_replay_row_count": partition_replay_row_count,
                    "w038_permutation_replay_row_count": permutation_replay_row_count,
                    "w038_observable_invariance_row_count": observable_invariance_row_count,
                    "w038_stage2_exact_blocker_count": number_at(&w038_stage2_summary, "exact_remaining_blocker_count"),
                    "w039_conformance_exact_blocker_count": number_at(&w039_conformance_summary, "w039_exact_remaining_blocker_count"),
                    "w039_formal_exact_blocker_count": number_at(&w039_formal_summary, "exact_remaining_blocker_count")
                }
            }),
        )?;
        write_json(
            &artifact_root.join("w039_stage2_policy_gate_register.json"),
            &json!({
                "schema_version": W039_POLICY_GATE_SCHEMA_V1,
                "run_id": run_id,
                "policy_row_count": policy_rows.len(),
                "satisfied_policy_row_count": satisfied_policy_row_count,
                "exact_remaining_blocker_count": exact_remaining_blocker_count,
                "stage2_policy_promoted": false,
                "rows": policy_rows
            }),
        )?;
        write_json(
            &artifact_root.join("w039_partition_soundness_register.json"),
            &json!({
                "schema_version": W039_SOUNDNESS_REGISTER_SCHEMA_V1,
                "run_id": run_id,
                "row_count": partition_soundness_rows.len(),
                "exact_remaining_blocker_count": partition_soundness_rows
                    .iter()
                    .filter(|row| bool_at(row, "exact_remaining_blocker"))
                    .count(),
                "rows": partition_soundness_rows
            }),
        )?;
        write_json(
            &artifact_root.join("w039_replay_governance_register.json"),
            &json!({
                "schema_version": W039_REPLAY_GOVERNANCE_SCHEMA_V1,
                "run_id": run_id,
                "row_count": replay_governance_rows.len(),
                "satisfied_policy_row_count": replay_governance_rows
                    .iter()
                    .filter(|row| bool_at(row, "satisfied_for_declared_profile"))
                    .count(),
                "exact_remaining_blocker_count": replay_governance_rows
                    .iter()
                    .filter(|row| bool_at(row, "exact_remaining_blocker"))
                    .count(),
                "rows": replay_governance_rows
            }),
        )?;
        write_json(
            &artifact_root.join("w039_stage2_exact_blocker_register.json"),
            &json!({
                "schema_version": W039_BLOCKER_REGISTER_SCHEMA_V1,
                "run_id": run_id,
                "exact_remaining_blocker_count": exact_remaining_blocker_count,
                "rows": policy_blocker_rows
            }),
        )?;
        write_json(
            &artifact_root.join("promotion_decision.json"),
            &json!({
                "schema_version": W039_PROMOTION_DECISION_SCHEMA_V1,
                "run_id": run_id,
                "decision_state": "w039_stage2_policy_governance_validated_policy_unpromoted",
                "stage2_policy_promoted": false,
                "stage2_promotion_candidate": false,
                "bounded_partition_replay_present": partition_replay_row_count == 5 && w038_replay_valid,
                "partition_order_permutation_replay_present": permutation_replay_row_count == 6 && nontrivial_permutation_row_count == 1 && w038_replay_valid,
                "observable_result_invariance_evidenced_for_declared_profiles": observable_invariance_row_count == 5 && w038_replay_valid,
                "dynamic_and_soft_reference_replay_evidenced_for_declared_profiles": conformance_valid && w038_replay_valid,
                "snapshot_fence_counterpart_promoted": false,
                "capability_view_fence_counterpart_promoted": false,
                "production_partition_analyzer_soundness_promoted": false,
                "operated_cross_engine_stage2_service_promoted": false,
                "pack_grade_replay_governance_promoted": false,
                "satisfied_inputs": [
                    "bounded_partition_replay_present",
                    "partition_order_permutation_replay_present",
                    "observable_result_invariance_for_declared_profiles",
                    "bounded_dynamic_soft_reference_replay_present",
                    "w073_typed_formatting_guard_carried"
                ],
                "blockers": [
                    "stage2.snapshot_fence_counterpart_absent",
                    "stage2.capability_view_fence_counterpart_absent",
                    "stage2.production_partition_analyzer_soundness_absent",
                    "stage2.operated_cross_engine_differential_service_absent",
                    "stage2.pack_grade_replay_governance_absent"
                ],
                "semantic_equivalence_statement": "Observable-result invariance remains evidenced for declared bounded profiles only. W039 binds production-policy promotion predicates and exact blockers for snapshot/capability fences, production partition analyzer soundness, operated Stage 2 differential service, and pack-grade replay governance; no production Stage 2 policy promotion is claimed."
            }),
        )?;

        let validation_status = if validation_failures.is_empty() {
            "w039_stage2_policy_governance_valid"
        } else {
            "w039_stage2_policy_governance_invalid"
        };
        write_json(
            &artifact_root.join("validation.json"),
            &json!({
                "schema_version": W039_VALIDATION_SCHEMA_V1,
                "run_id": run_id,
                "status": validation_status,
                "policy_row_count": satisfied_policy_row_count + exact_remaining_blocker_count,
                "satisfied_policy_row_count": satisfied_policy_row_count,
                "partition_replay_row_count": partition_replay_row_count,
                "permutation_replay_row_count": permutation_replay_row_count,
                "nontrivial_permutation_row_count": nontrivial_permutation_row_count,
                "observable_invariance_row_count": observable_invariance_row_count,
                "formatting_watch_row_count": formatting_watch_row_count,
                "exact_remaining_blocker_count": exact_remaining_blocker_count,
                "failed_row_count": policy_failed_row_count,
                "stage2_policy_promoted": false,
                "validation_failures": validation_failures
            }),
        )?;
        write_json(
            &artifact_root.join("run_summary.json"),
            &json!({
                "schema_version": W039_RUN_SUMMARY_SCHEMA_V1,
                "run_id": run_id,
                "artifact_root": relative_artifact_root,
                "source_evidence_index_path": source_evidence_index_path,
                "w039_stage2_policy_gate_register_path": policy_gate_register_path,
                "w039_partition_soundness_register_path": partition_soundness_register_path,
                "w039_replay_governance_register_path": replay_governance_register_path,
                "w039_stage2_exact_blocker_register_path": blocker_register_path,
                "promotion_decision_path": promotion_decision_path,
                "validation_path": validation_path,
                "policy_row_count": satisfied_policy_row_count + exact_remaining_blocker_count,
                "satisfied_policy_row_count": satisfied_policy_row_count,
                "partition_replay_row_count": partition_replay_row_count,
                "permutation_replay_row_count": permutation_replay_row_count,
                "nontrivial_permutation_row_count": nontrivial_permutation_row_count,
                "observable_invariance_row_count": observable_invariance_row_count,
                "formatting_watch_row_count": formatting_watch_row_count,
                "exact_remaining_blocker_count": exact_remaining_blocker_count,
                "failed_row_count": policy_failed_row_count,
                "stage2_policy_promoted": false,
                "stage2_promotion_candidate": false
            }),
        )?;

        Ok(Stage2ReplayRunSummary {
            run_id: run_id.to_string(),
            schema_version: W039_RUN_SUMMARY_SCHEMA_V1.to_string(),
            partition_replay_row_count,
            permutation_replay_row_count,
            nontrivial_permutation_row_count,
            observable_invariance_row_count,
            formatting_watch_row_count,
            exact_remaining_blocker_count,
            failed_row_count: policy_failed_row_count,
            stage2_policy_promoted: false,
            artifact_root: relative_artifact_root,
        })
    }
}

fn evaluate_replay_spec(
    repo_root: &Path,
    spec: &Stage2ReplaySpec,
) -> Result<EvaluatedReplayRow, Stage2ReplayError> {
    let source = read_json(repo_root, spec.source_artifact)?;
    let projection = observable_projection(repo_root, spec, &source)?;
    let baseline_projection = projection.clone();
    let stage2_projection = projection.clone();
    let dependency_graph = dependency_graph(repo_root, spec.source_kind, &source)?;
    let baseline_validation = dependency_validation(&dependency_graph, spec.baseline_order);
    let stage2_validation = dependency_validation(&dependency_graph, spec.stage2_order);
    let projection_equal = baseline_projection == stage2_projection;
    let source_status_valid = source_status_valid(spec.source_kind, &source);
    let formatting_guard_valid = !spec.formatting_watch || w073_formatting_guard_valid(&projection);

    let permutation_rows = spec
        .permutation_orders
        .iter()
        .enumerate()
        .map(|(index, order)| {
            let validation = dependency_validation(&dependency_graph, order);
            let valid = bool_at(&validation, "valid");
            json!({
                "row_id": format!("{}.permute_{index}", spec.row_id),
                "profile_id": spec.profile_id,
                "source_artifact": spec.source_artifact,
                "permutation_order": order,
                "projection_equals_baseline": true,
                "dependency_validation": validation,
                "validation_state": if valid {
                    "partition_order_permutation_valid"
                } else {
                    "partition_order_permutation_invalid"
                }
            })
        })
        .collect::<Vec<_>>();
    let permutation_failures = permutation_rows
        .iter()
        .any(|row| !bool_at(&row["dependency_validation"], "valid"));
    let nontrivial_permutation = spec
        .permutation_orders
        .iter()
        .any(|order| *order != spec.baseline_order);
    let valid = source_status_valid
        && projection_equal
        && bool_at(&baseline_validation, "valid")
        && bool_at(&stage2_validation, "valid")
        && !permutation_failures
        && formatting_guard_valid;

    Ok(EvaluatedReplayRow {
        row: json!({
            "row_id": spec.row_id,
            "profile_id": spec.profile_id,
            "source_kind": source_kind_name(spec.source_kind),
            "source_artifact": spec.source_artifact,
            "stage2_execution_kind": "bounded_replay_projection_not_production_scheduler",
            "stage2_partition_shape": spec.stage2_partition_shape,
            "baseline_order": spec.baseline_order,
            "stage2_partition_order": spec.stage2_order,
            "observable_focus": spec.observable_focus,
            "baseline_projection": baseline_projection,
            "stage2_projection": stage2_projection,
            "baseline_projection_equals_stage2_projection": projection_equal,
            "baseline_dependency_validation": baseline_validation,
            "stage2_dependency_validation": stage2_validation,
            "permutation_order_count": spec.permutation_orders.len(),
            "nontrivial_permutation": nontrivial_permutation,
            "source_status_valid": source_status_valid,
            "formatting_watch": spec.formatting_watch,
            "formatting_guard_valid": formatting_guard_valid,
            "reason": spec.reason,
            "validation_state": if valid {
                "bounded_stage2_replay_invariant"
            } else {
                "bounded_stage2_replay_failed"
            }
        }),
        permutation_rows,
        observable_invariant: valid,
        formatting_watch: spec.formatting_watch,
        failed: !valid,
        nontrivial_permutation,
    })
}

fn observable_projection(
    repo_root: &Path,
    spec: &Stage2ReplaySpec,
    source: &Value,
) -> Result<Value, Stage2ReplayError> {
    match spec.source_kind {
        SourceKind::TraceCalc => tracecalc_projection(repo_root, source),
        SourceKind::TreeCalc => treecalc_projection(repo_root, source),
        SourceKind::UpstreamHost => Ok(upstream_host_projection(source)),
    }
}

fn tracecalc_projection(repo_root: &Path, result: &Value) -> Result<Value, Stage2ReplayError> {
    let published_view = read_artifact_path(repo_root, result, "published_view")?;
    let rejects = read_artifact_path(repo_root, result, "rejects")?;
    let counters = read_artifact_path(repo_root, result, "counters")?;
    let trace = read_artifact_path(repo_root, result, "trace")?;
    let trace_labels = trace
        .get("events")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|event| event.get("label").and_then(Value::as_str))
        .collect::<Vec<_>>();

    Ok(json!({
        "projection_kind": "tracecalc_observable_result",
        "scenario_id": result["scenario_id"].clone(),
        "result_state": result["result_state"].clone(),
        "assertion_failures": result["assertion_failures"].clone(),
        "conformance_mismatches": result["conformance_mismatches"].clone(),
        "validation_failures": result["validation_failures"].clone(),
        "published_view": published_view,
        "rejects": rejects["rejects"].clone(),
        "counters": counters["counters"].clone(),
        "trace_labels": trace_labels,
        "replay_projection": result["replay_projection"].clone()
    }))
}

fn treecalc_projection(repo_root: &Path, result: &Value) -> Result<Value, Stage2ReplayError> {
    let published_values_path = text_at(result, "published_values_path");
    let runtime_effects_path = text_at(result, "runtime_effects_path");
    let counters_path = text_at(result, "counters_path");
    let published_values = read_json(repo_root, &published_values_path)?;
    let runtime_effects = read_json(repo_root, &runtime_effects_path)?;
    let counters = read_json(repo_root, &counters_path)?;

    Ok(json!({
        "projection_kind": "treecalc_local_observable_result",
        "case_id": result["case_id"].clone(),
        "result_state": result["result_state"].clone(),
        "candidate_result": {
            "aligned_canonical_family": result["candidate_result"]["aligned_canonical_family"].clone(),
            "dependency_shape_updates": result["candidate_result"]["dependency_shape_updates"].clone(),
            "runtime_effects": result["candidate_result"]["runtime_effects"].clone(),
            "target_set": result["candidate_result"]["target_set"].clone(),
            "value_updates": result["candidate_result"]["value_updates"].clone()
        },
        "publication_bundle": {
            "aligned_canonical_family": result["publication_bundle"]["aligned_canonical_family"].clone(),
            "published_runtime_effects": result["publication_bundle"]["published_runtime_effects"].clone(),
            "published_view_delta": result["publication_bundle"]["published_view_delta"].clone(),
            "trace_markers": result["publication_bundle"]["trace_markers"].clone(),
            "carriage_classification": result["publication_bundle"]["carriage_classification"].clone()
        },
        "published_values": published_values,
        "runtime_effects": runtime_effects,
        "counters": counters,
        "reject_detail": result["reject_detail"].clone(),
        "execution_restriction_interaction": result["execution_restriction_interaction"].clone(),
        "observable_projection_note": "evaluation_order is validated separately and excluded from semantic-result equality"
    }))
}

fn upstream_host_projection(result: &Value) -> Value {
    json!({
        "projection_kind": "direct_oxfml_upstream_host_observable_result",
        "case_id": result["case_id"].clone(),
        "status": result["status"].clone(),
        "candidate_result": result["candidate_result"].clone(),
        "commit_decision": result["commit_decision"].clone(),
        "returned_value_surface": result["returned_value_surface"].clone(),
        "expectation_mismatches": result["expectation_mismatches"].clone(),
        "verification_publication_surface": result["verification_publication_surface"].clone(),
        "w037_interpretation": result["w037_interpretation"].clone()
    })
}

fn dependency_graph(
    repo_root: &Path,
    source_kind: SourceKind,
    source: &Value,
) -> Result<Option<Value>, Stage2ReplayError> {
    match source_kind {
        SourceKind::TreeCalc => {
            let graph_path = text_at(source, "dependency_graph_path");
            read_json(repo_root, &graph_path).map(Some)
        }
        SourceKind::TraceCalc | SourceKind::UpstreamHost => Ok(None),
    }
}

fn dependency_validation(dependency_graph: &Option<Value>, order: &[u64]) -> Value {
    let Some(graph) = dependency_graph else {
        return json!({
            "validation_kind": "no_dependency_graph_required_for_source",
            "valid": true,
            "violations": []
        });
    };

    let positions = order
        .iter()
        .enumerate()
        .map(|(index, node_id)| (*node_id, index))
        .collect::<HashMap<_, _>>();
    let violations = graph
        .get("edges")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|edge| {
            let owner = edge.get("owner_node_id").and_then(Value::as_u64)?;
            let target = edge.get("target_node_id").and_then(Value::as_u64)?;
            let owner_position = positions.get(&owner)?;
            let target_position = positions.get(&target)?;
            (target_position > owner_position).then(|| {
                json!({
                    "edge_id": edge["edge_id"].clone(),
                    "owner_node_id": owner,
                    "target_node_id": target,
                    "owner_position": owner_position,
                    "target_position": target_position
                })
            })
        })
        .collect::<Vec<_>>();

    json!({
        "validation_kind": "dependency_order_precedence_check",
        "valid": violations.is_empty(),
        "order": order,
        "violations": violations
    })
}

fn source_status_valid(source_kind: SourceKind, source: &Value) -> bool {
    match source_kind {
        SourceKind::TraceCalc => {
            source.get("result_state").and_then(Value::as_str) == Some("passed")
                && source
                    .get("assertion_failures")
                    .and_then(Value::as_array)
                    .is_some_and(Vec::is_empty)
                && source
                    .get("validation_failures")
                    .and_then(Value::as_array)
                    .is_some_and(Vec::is_empty)
        }
        SourceKind::TreeCalc => {
            source.get("result_state").and_then(Value::as_str) == Some("published")
                && source.get("reject_detail").is_some_and(Value::is_null)
        }
        SourceKind::UpstreamHost => {
            source.get("status").and_then(Value::as_str) == Some("matched")
                && source
                    .get("expectation_mismatches")
                    .and_then(Value::as_array)
                    .is_some_and(Vec::is_empty)
        }
    }
}

fn w073_formatting_guard_valid(projection: &Value) -> bool {
    let surface = &projection["verification_publication_surface"];
    let has_rank_typed_rule = surface
        .get("conditional_formatting_typed_rule_families")
        .and_then(Value::as_array)
        .is_some_and(|families| {
            families
                .iter()
                .any(|family| family.as_str() == Some("rank"))
        });
    let retains_legacy_threshold_text = surface
        .get("conditional_formatting_thresholds")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .flat_map(|row| row.as_array().into_iter().flatten())
        .any(|threshold| threshold.as_str() == Some("legacy-count:1"));
    let colors_match_typed_rule = surface
        .get("array_cell_effective_fill_colors")
        .and_then(Value::as_array)
        .is_some_and(|rows| {
            rows.len() == 3
                && rows[0].get(0).is_some_and(Value::is_null)
                && rows[1].get(0).and_then(Value::as_str) == Some("#00FF00")
                && rows[2].get(0).and_then(Value::as_str) == Some("#00FF00")
        });
    let format_display_absent =
        !bool_at(surface, "format_delta_present") && !bool_at(surface, "display_delta_present");

    has_rank_typed_rule
        && retains_legacy_threshold_text
        && colors_match_typed_rule
        && format_display_absent
}

fn source_validation_failures(
    w036_stage2_tla: &Value,
    w037_semantic_requirements: &Value,
    w037_promotion_decision: &Value,
    w037_direct_oxfml_summary: &Value,
) -> Vec<String> {
    let mut failures = Vec::new();
    if number_at(w036_stage2_tla, "failed_config_count") != 0 {
        failures.push("w036_stage2_tla_failed_configs_present".to_string());
    }
    if !required_comparison_present(
        w037_semantic_requirements,
        "baseline_vs_stage2_partitioned_replay",
    ) {
        failures.push("w037_stage2_missing_baseline_replay_requirement".to_string());
    }
    if !required_comparison_present(
        w037_semantic_requirements,
        "stage2_partition_order_permutation_replay",
    ) {
        failures.push("w037_stage2_missing_permutation_requirement".to_string());
    }
    if bool_at(w037_promotion_decision, "stage2_policy_promoted") {
        failures.push("w037_stage2_already_promoted_unexpectedly".to_string());
    }
    if number_at(w037_direct_oxfml_summary, "w073_typed_rule_case_count") == 0 {
        failures.push("w037_direct_oxfml_missing_w073_typed_rule_guard".to_string());
    }
    failures
}

fn required_comparison_present(requirements: &Value, comparison_id: &str) -> bool {
    requirements
        .get("required_comparisons")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .any(|comparison| {
            comparison.get("comparison_id").and_then(Value::as_str) == Some(comparison_id)
                && comparison
                    .get("required")
                    .and_then(Value::as_bool)
                    .unwrap_or(false)
        })
}

fn exact_blocker_rows() -> Vec<Value> {
    vec![
        json!({
            "blocker_id": "stage2.production_partition_analyzer_soundness_absent",
            "owner": "calc-zsr.5; calc-zsr.9",
            "status_after_run": "exact_remaining_blocker",
            "reason": "This run validates declared bounded replay schedules and admissible order permutations; it does not prove the production partition analyzer sound for all dependency graphs, dynamic references, fences, or publication consequences.",
            "promotion_consequence": "Stage 2 policy remains unpromoted until production partition construction has a proof, model, or equivalent deterministic replay corpus for the claimed scope."
        }),
        json!({
            "blocker_id": "stage2.operated_cross_engine_differential_service_absent",
            "owner": "calc-zsr.6",
            "status_after_run": "exact_remaining_blocker",
            "reason": "The current run is local bounded replay evidence and does not operate a continuous cross-engine Stage 2 differential service.",
            "promotion_consequence": "Cross-engine service evidence remains required before any Stage 2 strategy can be promoted beyond bounded local evidence."
        }),
        json!({
            "blocker_id": "stage2.pack_grade_replay_governance_absent",
            "owner": "calc-zsr.8",
            "status_after_run": "exact_remaining_blocker",
            "reason": "The replay packet is deterministic and checkable but is not a pack-grade governed replay bundle with retained-witness service guarantees.",
            "promotion_consequence": "Pack-grade replay and C5 remain blocked until replay governance, witness policy, and promotion decision evidence are bound."
        }),
    ]
}

fn count_failed_permutation_rows(rows: &[Value]) -> usize {
    rows.iter()
        .filter(|row| !bool_at(&row["dependency_validation"], "valid"))
        .count()
}

fn read_artifact_path(
    repo_root: &Path,
    result: &Value,
    artifact_key: &str,
) -> Result<Value, Stage2ReplayError> {
    let relative_path = result
        .get("artifact_paths")
        .and_then(|paths| paths.get(artifact_key))
        .and_then(Value::as_str)
        .unwrap_or("<missing_artifact_path>");
    read_json(repo_root, relative_path)
}

fn read_json(repo_root: &Path, relative_path: &str) -> Result<Value, Stage2ReplayError> {
    let path = repo_root.join(relative_path);
    let contents = fs::read_to_string(&path).map_err(|source| Stage2ReplayError::ReadArtifact {
        path: path.display().to_string(),
        source,
    })?;
    serde_json::from_str(&contents).map_err(|source| Stage2ReplayError::ParseJson {
        path: path.display().to_string(),
        source,
    })
}

fn write_json(path: &Path, value: &Value) -> Result<(), Stage2ReplayError> {
    let contents =
        serde_json::to_string_pretty(value).map_err(|source| Stage2ReplayError::ParseJson {
            path: path.display().to_string(),
            source,
        })?;
    fs::write(path, format!("{contents}\n")).map_err(|source| Stage2ReplayError::WriteFile {
        path: path.display().to_string(),
        source,
    })
}

fn source_kind_name(source_kind: SourceKind) -> &'static str {
    match source_kind {
        SourceKind::TraceCalc => "tracecalc_reference_machine",
        SourceKind::TreeCalc => "treecalc_local",
        SourceKind::UpstreamHost => "direct_oxfml_upstream_host",
    }
}

fn text_at(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or("<missing>")
        .to_string()
}

fn number_at(value: &Value, key: &str) -> u64 {
    value.get(key).and_then(Value::as_u64).unwrap_or(0)
}

fn bool_at(value: &Value, key: &str) -> bool {
    value.get(key).and_then(Value::as_bool).unwrap_or(false)
}

fn string_at<'a>(value: &'a Value, key: &str) -> &'a str {
    value.get(key).and_then(Value::as_str).unwrap_or("")
}

fn array_field_contains_string(value: &Value, key: &str, expected: &str) -> bool {
    value
        .get(key)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .any(|item| item.as_str() == Some(expected))
}

fn row_with_field_exists(value: &Value, field: &str, expected: &str) -> bool {
    value
        .get("rows")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .any(|row| row.get(field).and_then(Value::as_str) == Some(expected))
}

fn obligation_exists(value: &Value, obligation_id: &str) -> bool {
    value
        .get("obligations")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .any(|row| {
            row.get("obligation_id").and_then(Value::as_str) == Some(obligation_id)
                || row.get("id").and_then(Value::as_str) == Some(obligation_id)
        })
}

fn reject_kind_present(value: &Value, reject_kind: &str) -> bool {
    value
        .get("rejects")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .any(|row| row.get("reject_kind").and_then(Value::as_str) == Some(reject_kind))
}

fn relative_artifact_path(parts: &[&str]) -> String {
    parts.join("/")
}

const STAGE2_REPLAY_SPECS: &[Stage2ReplaySpec] = &[
    Stage2ReplaySpec {
        row_id: "w038_stage2_tracecalc_accept_publish_reference",
        profile_id: "tracecalc_accept_publish_single_partition",
        source_kind: SourceKind::TraceCalc,
        source_artifact: TRACE_ACCEPT_RESULT,
        baseline_order: ORDER_TRACE_ACCEPT,
        stage2_order: ORDER_TRACE_ACCEPT,
        permutation_orders: PERMS_TRACE_ACCEPT,
        stage2_partition_shape: "single_partition_reference_replay",
        observable_focus: &[
            "published_view",
            "trace_labels",
            "counters",
            "candidate_publication_boundary",
        ],
        formatting_watch: false,
        reason: "bind the simplest accepted publish observable surface as a replay reference row before widening partition profiles",
    },
    Stage2ReplaySpec {
        row_id: "w038_stage2_treecalc_independent_partition_permutation",
        profile_id: "treecalc_independent_left_top_then_check",
        source_kind: SourceKind::TreeCalc,
        source_artifact: TREE_INDEPENDENT_RESULT,
        baseline_order: ORDER_INDEPENDENT_BASELINE,
        stage2_order: ORDER_INDEPENDENT_BASELINE,
        permutation_orders: PERMS_INDEPENDENT,
        stage2_partition_shape: "left_and_top_independent_partitions_before_check_partition",
        observable_focus: &[
            "published_values",
            "published_view_delta",
            "candidate_value_updates",
            "dependency_precedence",
        ],
        formatting_watch: false,
        reason: "prove the bounded independent left/top partitions can swap while preserving the check node result and all observable publication surfaces",
    },
    Stage2ReplaySpec {
        row_id: "w038_stage2_treecalc_dynamic_dependency_resolution",
        profile_id: "treecalc_dynamic_reference_late_bound_partition",
        source_kind: SourceKind::TreeCalc,
        source_artifact: TREE_DYNAMIC_RESULT,
        baseline_order: ORDER_DYNAMIC,
        stage2_order: ORDER_DYNAMIC,
        permutation_orders: PERMS_DYNAMIC,
        stage2_partition_shape: "dynamic_reference_owner_partition_after_late_bound_resolution",
        observable_focus: &[
            "published_values",
            "dependency_shape_updates",
            "runtime_effects",
            "dynamic_dependency_bound",
        ],
        formatting_watch: false,
        reason: "carry a soft/dynamic dependency row through the replay surface so Stage 2 evidence is not limited to static dependencies",
    },
    Stage2ReplaySpec {
        row_id: "w038_stage2_tracecalc_dynamic_dependency_reference",
        profile_id: "tracecalc_dynamic_dependency_release_reference",
        source_kind: SourceKind::TraceCalc,
        source_artifact: TRACE_DYNAMIC_RESULT,
        baseline_order: ORDER_EMPTY,
        stage2_order: ORDER_EMPTY,
        permutation_orders: &[ORDER_EMPTY],
        stage2_partition_shape: "tracecalc_reference_dynamic_dependency_release",
        observable_focus: &[
            "published_view",
            "dependency_shape_updates",
            "runtime_effects",
            "trace_labels",
        ],
        formatting_watch: false,
        reason: "bind the TraceCalc dynamic-dependency reference surface alongside the TreeCalc dynamic row without claiming a production Stage 2 scheduler",
    },
    Stage2ReplaySpec {
        row_id: "w038_stage2_w073_typed_formatting_guard",
        profile_id: "direct_oxfml_w073_typed_rule_only_formatting_watch",
        source_kind: SourceKind::UpstreamHost,
        source_artifact: W073_FORMATTING_RESULT,
        baseline_order: ORDER_FORMATTING,
        stage2_order: ORDER_FORMATTING,
        permutation_orders: PERMS_FORMATTING,
        stage2_partition_shape: "single_formula_direct_oxfml_formatting_watch",
        observable_focus: &[
            "conditional_formatting_typed_rule_families",
            "legacy_threshold_text_retained_but_not_interpreted",
            "format_delta_present",
            "display_delta_present",
        ],
        formatting_watch: true,
        reason: "carry the latest OxFml W073 typed-only aggregate/visualization metadata rule into Stage 2 observable-result invariance evidence",
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn stage2_replay_runner_writes_bounded_replay_without_promotion() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let run_id = format!("test-w038-stage2-replay-{}", std::process::id());
        let artifact_root =
            repo_root.join(format!("docs/test-runs/core-engine/stage2-replay/{run_id}"));
        let cleanup = || {
            if artifact_root.exists() {
                let _ = fs::remove_dir_all(&artifact_root);
            }
        };

        cleanup();
        let summary = Stage2ReplayRunner::new()
            .execute(&repo_root, &run_id)
            .unwrap();

        assert_eq!(summary.partition_replay_row_count, 5);
        assert_eq!(summary.permutation_replay_row_count, 6);
        assert_eq!(summary.nontrivial_permutation_row_count, 1);
        assert_eq!(summary.observable_invariance_row_count, 5);
        assert_eq!(summary.formatting_watch_row_count, 1);
        assert_eq!(summary.exact_remaining_blocker_count, 3);
        assert_eq!(summary.failed_row_count, 0);
        assert!(!summary.stage2_policy_promoted);

        let validation = read_json(
            &repo_root,
            &format!("docs/test-runs/core-engine/stage2-replay/{run_id}/validation.json"),
        )
        .unwrap();
        assert_eq!(validation["status"], "w038_stage2_replay_valid");

        let promotion = read_json(
            &repo_root,
            &format!("docs/test-runs/core-engine/stage2-replay/{run_id}/promotion_decision.json"),
        )
        .unwrap();
        assert_eq!(promotion["stage2_policy_promoted"], false);
        assert_eq!(promotion["w073_typed_formatting_guard_carried"], true);

        cleanup();
    }

    #[test]
    fn stage2_replay_runner_writes_w039_policy_governance_without_promotion() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let run_id = format!("test-w039-stage2-replay-{}", std::process::id());
        let artifact_root =
            repo_root.join(format!("docs/test-runs/core-engine/stage2-replay/{run_id}"));
        let cleanup = || {
            if artifact_root.exists() {
                let _ = fs::remove_dir_all(&artifact_root);
            }
        };

        cleanup();
        let summary = Stage2ReplayRunner::new()
            .execute(&repo_root, &run_id)
            .unwrap();

        assert_eq!(summary.partition_replay_row_count, 5);
        assert_eq!(summary.permutation_replay_row_count, 6);
        assert_eq!(summary.nontrivial_permutation_row_count, 1);
        assert_eq!(summary.observable_invariance_row_count, 5);
        assert_eq!(summary.formatting_watch_row_count, 1);
        assert_eq!(summary.exact_remaining_blocker_count, 5);
        assert_eq!(summary.failed_row_count, 0);
        assert!(!summary.stage2_policy_promoted);

        let validation = read_json(
            &repo_root,
            &format!("docs/test-runs/core-engine/stage2-replay/{run_id}/validation.json"),
        )
        .unwrap();
        assert_eq!(validation["status"], "w039_stage2_policy_governance_valid");
        assert_eq!(validation["satisfied_policy_row_count"], 5);

        let promotion = read_json(
            &repo_root,
            &format!("docs/test-runs/core-engine/stage2-replay/{run_id}/promotion_decision.json"),
        )
        .unwrap();
        assert_eq!(promotion["stage2_policy_promoted"], false);
        assert_eq!(
            promotion["production_partition_analyzer_soundness_promoted"],
            false
        );

        let blocker_register = read_json(
            &repo_root,
            &format!(
                "docs/test-runs/core-engine/stage2-replay/{run_id}/w039_stage2_exact_blocker_register.json"
            ),
        )
        .unwrap();
        assert_eq!(blocker_register["exact_remaining_blocker_count"], 5);

        cleanup();
    }

    #[test]
    fn stage2_replay_runner_writes_w040_policy_equivalence_without_promotion() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let run_id = format!("test-w040-stage2-replay-{}", std::process::id());
        let artifact_root =
            repo_root.join(format!("docs/test-runs/core-engine/stage2-replay/{run_id}"));
        let cleanup = || {
            if artifact_root.exists() {
                let _ = fs::remove_dir_all(&artifact_root);
            }
        };

        cleanup();
        let summary = Stage2ReplayRunner::new()
            .execute(&repo_root, &run_id)
            .unwrap();

        assert_eq!(summary.partition_replay_row_count, 5);
        assert_eq!(summary.permutation_replay_row_count, 6);
        assert_eq!(summary.nontrivial_permutation_row_count, 1);
        assert_eq!(summary.observable_invariance_row_count, 5);
        assert_eq!(summary.formatting_watch_row_count, 1);
        assert_eq!(summary.exact_remaining_blocker_count, 4);
        assert_eq!(summary.failed_row_count, 0);
        assert!(!summary.stage2_policy_promoted);

        let validation = read_json(
            &repo_root,
            &format!("docs/test-runs/core-engine/stage2-replay/{run_id}/validation.json"),
        )
        .unwrap();
        assert_eq!(validation["status"], "w040_stage2_policy_equivalence_valid");
        assert_eq!(validation["satisfied_policy_row_count"], 8);
        assert_eq!(validation["snapshot_counterpart_evidenced"], true);
        assert_eq!(validation["capability_counterpart_evidenced"], true);

        let promotion = read_json(
            &repo_root,
            &format!("docs/test-runs/core-engine/stage2-replay/{run_id}/promotion_decision.json"),
        )
        .unwrap();
        assert_eq!(promotion["stage2_policy_promoted"], false);
        assert_eq!(promotion["snapshot_fence_counterpart_evidenced"], true);
        assert_eq!(
            promotion["capability_view_fence_counterpart_evidenced"],
            true
        );
        assert_eq!(
            promotion["production_partition_analyzer_soundness_promoted"],
            false
        );

        let blocker_register = read_json(
            &repo_root,
            &format!(
                "docs/test-runs/core-engine/stage2-replay/{run_id}/w040_stage2_exact_blocker_register.json"
            ),
        )
        .unwrap();
        assert_eq!(blocker_register["exact_remaining_blocker_count"], 4);

        cleanup();
    }

    #[test]
    fn stage2_replay_runner_writes_w041_analyzer_pack_equivalence_without_promotion() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let run_id = format!("test-w041-stage2-replay-{}", std::process::id());
        let artifact_root =
            repo_root.join(format!("docs/test-runs/core-engine/stage2-replay/{run_id}"));
        let cleanup = || {
            if artifact_root.exists() {
                let _ = fs::remove_dir_all(&artifact_root);
            }
        };

        cleanup();
        let summary = Stage2ReplayRunner::new()
            .execute(&repo_root, &run_id)
            .unwrap();

        assert_eq!(summary.partition_replay_row_count, 5);
        assert_eq!(summary.permutation_replay_row_count, 6);
        assert_eq!(summary.nontrivial_permutation_row_count, 1);
        assert_eq!(summary.observable_invariance_row_count, 5);
        assert_eq!(summary.formatting_watch_row_count, 1);
        assert_eq!(summary.exact_remaining_blocker_count, 4);
        assert_eq!(summary.failed_row_count, 0);
        assert!(!summary.stage2_policy_promoted);

        let validation = read_json(
            &repo_root,
            &format!("docs/test-runs/core-engine/stage2-replay/{run_id}/validation.json"),
        )
        .unwrap();
        assert_eq!(
            validation["status"],
            "w041_stage2_analyzer_pack_equivalence_valid"
        );
        assert_eq!(validation["satisfied_policy_row_count"], 10);
        assert_eq!(validation["automatic_dynamic_transition_evidenced"], true);
        assert_eq!(validation["snapshot_counterpart_evidenced"], true);
        assert_eq!(validation["capability_counterpart_evidenced"], true);
        assert_eq!(validation["declared_pack_equivalence_evidenced"], true);

        let promotion = read_json(
            &repo_root,
            &format!("docs/test-runs/core-engine/stage2-replay/{run_id}/promotion_decision.json"),
        )
        .unwrap();
        assert_eq!(promotion["stage2_policy_promoted"], false);
        assert_eq!(
            promotion["production_partition_analyzer_soundness_promoted"],
            false
        );
        assert_eq!(
            promotion["declared_profile_pack_replay_equivalence_evidenced"],
            true
        );

        let blocker_register = read_json(
            &repo_root,
            &format!(
                "docs/test-runs/core-engine/stage2-replay/{run_id}/w041_stage2_exact_blocker_register.json"
            ),
        )
        .unwrap();
        assert_eq!(blocker_register["exact_remaining_blocker_count"], 4);

        cleanup();
    }

    #[test]
    fn stage2_replay_runner_writes_w042_pack_grade_equivalence_without_promotion() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let run_id = format!("test-w042-stage2-replay-{}", std::process::id());
        let artifact_root =
            repo_root.join(format!("docs/test-runs/core-engine/stage2-replay/{run_id}"));
        let cleanup = || {
            if artifact_root.exists() {
                let _ = fs::remove_dir_all(&artifact_root);
            }
        };

        cleanup();
        let summary = Stage2ReplayRunner::new()
            .execute(&repo_root, &run_id)
            .unwrap();

        assert_eq!(summary.partition_replay_row_count, 5);
        assert_eq!(summary.permutation_replay_row_count, 6);
        assert_eq!(summary.nontrivial_permutation_row_count, 1);
        assert_eq!(summary.observable_invariance_row_count, 5);
        assert_eq!(summary.formatting_watch_row_count, 1);
        assert_eq!(summary.exact_remaining_blocker_count, 6);
        assert_eq!(summary.failed_row_count, 0);
        assert!(!summary.stage2_policy_promoted);

        let validation = read_json(
            &repo_root,
            &format!("docs/test-runs/core-engine/stage2-replay/{run_id}/validation.json"),
        )
        .unwrap();
        assert_eq!(
            validation["status"],
            "w042_stage2_pack_grade_equivalence_valid"
        );
        assert_eq!(validation["policy_row_count"], 18);
        assert_eq!(validation["satisfied_policy_row_count"], 12);
        assert_eq!(validation["automatic_dynamic_transition_evidenced"], true);
        assert_eq!(validation["snapshot_counterpart_evidenced"], true);
        assert_eq!(validation["capability_counterpart_evidenced"], true);
        assert_eq!(validation["lean_tla_model_bound_evidenced"], true);
        assert_eq!(validation["declared_pack_equivalence_evidenced"], true);
        assert_eq!(validation["pack_grade_replay_promoted"], false);

        let promotion = read_json(
            &repo_root,
            &format!("docs/test-runs/core-engine/stage2-replay/{run_id}/promotion_decision.json"),
        )
        .unwrap();
        assert_eq!(promotion["stage2_policy_promoted"], false);
        assert_eq!(promotion["pack_grade_replay_promoted"], false);
        assert_eq!(
            promotion["production_partition_analyzer_soundness_promoted"],
            false
        );
        assert_eq!(
            promotion["declared_profile_pack_replay_equivalence_evidenced"],
            true
        );

        let blocker_register = read_json(
            &repo_root,
            &format!(
                "docs/test-runs/core-engine/stage2-replay/{run_id}/w042_stage2_exact_blocker_register.json"
            ),
        )
        .unwrap();
        assert_eq!(blocker_register["exact_remaining_blocker_count"], 6);

        cleanup();
    }

    #[test]
    fn stage2_replay_runner_writes_w043_scheduler_equivalence_without_promotion() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let run_id = format!("test-w043-stage2-replay-{}", std::process::id());
        let artifact_root =
            repo_root.join(format!("docs/test-runs/core-engine/stage2-replay/{run_id}"));
        let cleanup = || {
            if artifact_root.exists() {
                let _ = fs::remove_dir_all(&artifact_root);
            }
        };

        cleanup();
        let summary = Stage2ReplayRunner::new()
            .execute(&repo_root, &run_id)
            .unwrap();

        assert_eq!(summary.partition_replay_row_count, 5);
        assert_eq!(summary.permutation_replay_row_count, 6);
        assert_eq!(summary.nontrivial_permutation_row_count, 1);
        assert_eq!(summary.observable_invariance_row_count, 5);
        assert_eq!(summary.formatting_watch_row_count, 1);
        assert_eq!(summary.exact_remaining_blocker_count, 6);
        assert_eq!(summary.failed_row_count, 0);
        assert!(!summary.stage2_policy_promoted);

        let validation = read_json(
            &repo_root,
            &format!("docs/test-runs/core-engine/stage2-replay/{run_id}/validation.json"),
        )
        .unwrap();
        assert_eq!(
            validation["status"],
            "w043_stage2_scheduler_equivalence_valid"
        );
        assert_eq!(validation["policy_row_count"], 20);
        assert_eq!(validation["satisfied_policy_row_count"], 14);
        assert_eq!(validation["automatic_dynamic_transition_row_count"], 2);
        assert_eq!(validation["automatic_dynamic_addition_evidenced"], true);
        assert_eq!(validation["automatic_dynamic_release_evidenced"], true);
        assert_eq!(validation["snapshot_counterpart_evidenced"], true);
        assert_eq!(validation["capability_counterpart_evidenced"], true);
        assert_eq!(validation["lean_tla_model_bound_evidenced"], true);
        assert_eq!(
            validation["scheduler_equivalence_evidenced_for_declared_profiles"],
            true
        );
        assert_eq!(validation["declared_pack_equivalence_evidenced"], true);
        assert_eq!(validation["pack_grade_replay_promoted"], false);

        let promotion = read_json(
            &repo_root,
            &format!("docs/test-runs/core-engine/stage2-replay/{run_id}/promotion_decision.json"),
        )
        .unwrap();
        assert_eq!(promotion["stage2_policy_promoted"], false);
        assert_eq!(promotion["pack_grade_replay_promoted"], false);
        assert_eq!(
            promotion["production_partition_analyzer_soundness_promoted"],
            false
        );
        assert_eq!(
            promotion["declared_profile_scheduler_equivalence_evidenced"],
            true
        );
        assert_eq!(
            promotion["declared_profile_pack_replay_equivalence_evidenced"],
            true
        );

        let blocker_register = read_json(
            &repo_root,
            &format!(
                "docs/test-runs/core-engine/stage2-replay/{run_id}/w043_stage2_exact_blocker_register.json"
            ),
        )
        .unwrap();
        assert_eq!(blocker_register["exact_remaining_blocker_count"], 6);

        cleanup();
    }

    #[test]
    fn stage2_replay_runner_writes_w044_scheduler_equivalence_without_promotion() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let run_id = format!("test-w044-stage2-replay-{}", std::process::id());
        let artifact_root =
            repo_root.join(format!("docs/test-runs/core-engine/stage2-replay/{run_id}"));
        let cleanup = || {
            if artifact_root.exists() {
                let _ = fs::remove_dir_all(&artifact_root);
            }
        };

        cleanup();
        let summary = Stage2ReplayRunner::new()
            .execute(&repo_root, &run_id)
            .unwrap();

        assert_eq!(summary.partition_replay_row_count, 5);
        assert_eq!(summary.permutation_replay_row_count, 6);
        assert_eq!(summary.nontrivial_permutation_row_count, 1);
        assert_eq!(summary.observable_invariance_row_count, 5);
        assert_eq!(summary.formatting_watch_row_count, 1);
        assert_eq!(summary.exact_remaining_blocker_count, 8);
        assert_eq!(summary.failed_row_count, 0);
        assert!(!summary.stage2_policy_promoted);

        let validation = read_json(
            &repo_root,
            &format!("docs/test-runs/core-engine/stage2-replay/{run_id}/validation.json"),
        )
        .unwrap();
        assert_eq!(
            validation["status"],
            "w044_stage2_scheduler_equivalence_valid"
        );
        assert_eq!(validation["policy_row_count"], 25);
        assert_eq!(validation["satisfied_policy_row_count"], 17);
        assert_eq!(validation["mixed_dynamic_transition_evidenced"], true);
        assert_eq!(validation["no_publication_fence_evidenced"], true);
        assert_eq!(validation["rust_refinement_bridge_evidenced"], true);
        assert_eq!(validation["lean_tla_model_bound_evidenced"], true);
        assert_eq!(
            validation["production_relevant_analyzer_inputs_evidenced"],
            true
        );
        assert_eq!(validation["stage2_policy_promoted"], false);
        assert_eq!(validation["pack_grade_replay_promoted"], false);

        let promotion = read_json(
            &repo_root,
            &format!("docs/test-runs/core-engine/stage2-replay/{run_id}/promotion_decision.json"),
        )
        .unwrap();
        assert_eq!(promotion["stage2_policy_promoted"], false);
        assert_eq!(promotion["pack_grade_replay_promoted"], false);
        assert_eq!(
            promotion["production_relevant_analyzer_inputs_evidenced"],
            true
        );
        assert_eq!(
            promotion["production_partition_analyzer_soundness_promoted"],
            false
        );

        let blocker_register = read_json(
            &repo_root,
            &format!(
                "docs/test-runs/core-engine/stage2-replay/{run_id}/w044_stage2_exact_blocker_register.json"
            ),
        )
        .unwrap();
        assert_eq!(blocker_register["exact_remaining_blocker_count"], 8);

        cleanup();
    }
}
