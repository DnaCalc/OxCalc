#![forbid(unsafe_code)]

//! Profile-governed numerical reduction selector.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum NumericalReductionPolicy {
    SequentialLeftFold,
    PairwiseTree,
    KahanCompensated,
}

impl NumericalReductionPolicy {
    #[must_use]
    pub fn selector_key(self) -> &'static str {
        match self {
            Self::SequentialLeftFold => "SequentialLeftFold",
            Self::PairwiseTree => "PairwiseTree",
            Self::KahanCompensated => "KahanCompensated",
        }
    }

    #[must_use]
    pub fn behavior(self) -> NumericalReductionBehavior {
        match self {
            Self::SequentialLeftFold => NumericalReductionBehavior {
                order_basis: NumericalReductionOrderBasis::RecordedLogicalLeftFold,
                requires_declared_input_order: true,
                requires_replay_visible_tree_shape: false,
                requires_compensation_state: false,
            },
            Self::PairwiseTree => NumericalReductionBehavior {
                order_basis: NumericalReductionOrderBasis::DeterministicPairwiseTree,
                requires_declared_input_order: true,
                requires_replay_visible_tree_shape: true,
                requires_compensation_state: false,
            },
            Self::KahanCompensated => NumericalReductionBehavior {
                order_basis: NumericalReductionOrderBasis::RecordedLogicalKahan,
                requires_declared_input_order: true,
                requires_replay_visible_tree_shape: false,
                requires_compensation_state: true,
            },
        }
    }

    #[must_use]
    pub fn handoff_clause(self) -> &'static NumericalReductionPolicyClause {
        NUMERICAL_REDUCTION_HANDOFF_CLAUSES
            .iter()
            .find(|clause| clause.selector_value == self)
            .expect("every numerical reduction policy has an exact handoff clause")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum NumericalReductionOrderBasis {
    RecordedLogicalLeftFold,
    DeterministicPairwiseTree,
    RecordedLogicalKahan,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NumericalReductionBehavior {
    pub order_basis: NumericalReductionOrderBasis,
    pub requires_declared_input_order: bool,
    pub requires_replay_visible_tree_shape: bool,
    pub requires_compensation_state: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NumericalReductionProfile {
    pub profile_version: String,
    pub numerical_reduction_policy: NumericalReductionPolicy,
}

impl NumericalReductionProfile {
    #[must_use]
    pub fn new(
        profile_version: impl Into<String>,
        numerical_reduction_policy: NumericalReductionPolicy,
    ) -> Self {
        Self {
            profile_version: profile_version.into(),
            numerical_reduction_policy,
        }
    }

    #[must_use]
    pub fn replay_profile_key(&self) -> String {
        format!(
            "{}|numerical_reduction_policy:{}",
            self.profile_version,
            self.numerical_reduction_policy.selector_key()
        )
    }

    #[must_use]
    pub fn behavior(&self) -> NumericalReductionBehavior {
        self.numerical_reduction_policy.behavior()
    }

    #[must_use]
    pub fn handoff_clause(&self) -> &'static NumericalReductionPolicyClause {
        self.numerical_reduction_policy.handoff_clause()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NumericalReductionPolicyClause {
    pub clause_id: &'static str,
    pub selector_value: NumericalReductionPolicy,
    pub exact_clause: &'static str,
}

pub const NUMERICAL_REDUCTION_HANDOFF_CLAUSES: [NumericalReductionPolicyClause; 3] = [
    NumericalReductionPolicyClause {
        clause_id: "CALC-003.NRP.SequentialLeftFold",
        selector_value: NumericalReductionPolicy::SequentialLeftFold,
        exact_clause: "When a profile declares NumericalReductionPolicy=SequentialLeftFold, OxFml/OxFunc reduction kernels MUST reduce numeric sequences in the recorded logical input order, applying each operand to the accumulator exactly once from left to right; kernels MUST NOT rebalance, parallelize, or compensate the order unless the active profile changes.",
    },
    NumericalReductionPolicyClause {
        clause_id: "CALC-003.NRP.PairwiseTree",
        selector_value: NumericalReductionPolicy::PairwiseTree,
        exact_clause: "When a profile declares NumericalReductionPolicy=PairwiseTree, OxFml/OxFunc reduction kernels MUST reduce numeric sequences using a deterministic pairwise tree whose leaf order is the recorded logical input order and whose tree-shape identity is replay-visible; kernels MUST NOT choose runtime-dependent partitioning.",
    },
    NumericalReductionPolicyClause {
        clause_id: "CALC-003.NRP.KahanCompensated",
        selector_value: NumericalReductionPolicy::KahanCompensated,
        exact_clause: "When a profile declares NumericalReductionPolicy=KahanCompensated, OxFml/OxFunc reduction kernels MUST reduce numeric sequences in the recorded logical input order using Kahan-style compensation state that is part of the semantic algorithm; kernels MUST surface the selector in replay so a non-compensated result cannot satisfy this profile.",
    },
];

#[must_use]
pub fn numerical_reduction_handoff_clauses() -> &'static [NumericalReductionPolicyClause] {
    &NUMERICAL_REDUCTION_HANDOFF_CLAUSES
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::fs;
    use std::path::PathBuf;

    use serde_json::json;

    use super::*;

    fn e1_artifact_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../docs/test-runs/core-engine/w050-e1-numerical-reduction-policy-selector-001",
        )
    }

    fn selector_artifact_json() -> serde_json::Value {
        json!({
            "run_id": "w050-e1-numerical-reduction-policy-selector-001",
            "selector_name": "NumericalReductionPolicy",
            "validation_status": "pass",
            "primary_validation_command": "cargo test -p oxcalc-core numerical_reduction -- --nocapture",
            "handoff_target": "HANDOFF_CALC_003_OXFML_NUMERICAL_REDUCTION_AND_ERROR_ALGEBRA.md",
            "profile_record": {
                "required_fields": [
                    "profile_version",
                    "numerical_reduction_policy"
                ],
                "replay_key_format": "<profile_version>|numerical_reduction_policy:<selector_key>"
            },
            "initial_variants": numerical_reduction_handoff_clauses()
                .iter()
                .map(|clause| {
                    let behavior = clause.selector_value.behavior();
                    json!({
                        "selector_value": clause.selector_value.selector_key(),
                        "order_basis": behavior.order_basis,
                        "requires_declared_input_order": behavior.requires_declared_input_order,
                        "requires_replay_visible_tree_shape": behavior.requires_replay_visible_tree_shape,
                        "requires_compensation_state": behavior.requires_compensation_state,
                        "handoff_clause_id": clause.clause_id,
                        "exact_clause": clause.exact_clause,
                    })
                })
                .collect::<Vec<_>>()
        })
    }

    #[test]
    fn numerical_reduction_profile_serializes_selector() {
        let profile = NumericalReductionProfile::new(
            "profile:correctness-floor:v1",
            NumericalReductionPolicy::PairwiseTree,
        );

        assert_eq!(
            profile.replay_profile_key(),
            "profile:correctness-floor:v1|numerical_reduction_policy:PairwiseTree"
        );
        assert_eq!(
            profile.behavior().order_basis,
            NumericalReductionOrderBasis::DeterministicPairwiseTree
        );
        assert!(profile.behavior().requires_replay_visible_tree_shape);

        let json = serde_json::to_value(&profile).unwrap();
        assert_eq!(json["profile_version"], "profile:correctness-floor:v1");
        assert_eq!(json["numerical_reduction_policy"], "PairwiseTree");

        let round_trip: NumericalReductionProfile = serde_json::from_value(json).unwrap();
        assert_eq!(round_trip, profile);
    }

    #[test]
    fn numerical_reduction_policy_clauses_are_handoff_ready_and_distinct() {
        let clauses = numerical_reduction_handoff_clauses();
        assert_eq!(clauses.len(), 3);

        let clause_ids = clauses
            .iter()
            .map(|clause| clause.clause_id)
            .collect::<BTreeSet<_>>();
        assert_eq!(clause_ids.len(), 3);

        for policy in [
            NumericalReductionPolicy::SequentialLeftFold,
            NumericalReductionPolicy::PairwiseTree,
            NumericalReductionPolicy::KahanCompensated,
        ] {
            let clause = policy.handoff_clause();
            assert_eq!(clause.selector_value, policy);
            assert!(clause.clause_id.starts_with("CALC-003.NRP."));
            assert!(clause.exact_clause.contains(policy.selector_key()));
            assert!(clause.exact_clause.contains("MUST"));
            assert!(clause.exact_clause.contains("OxFml/OxFunc"));
        }
    }

    #[test]
    fn checked_in_numerical_reduction_policy_artifact_matches_runtime_clauses() {
        let artifact_path = e1_artifact_root().join("selector_artifact.json");
        let artifact = serde_json::from_str::<serde_json::Value>(
            &fs::read_to_string(artifact_path).expect("E1 selector artifact should be checked in"),
        )
        .expect("E1 selector artifact should be valid JSON");

        assert_eq!(artifact, selector_artifact_json());
    }
}
