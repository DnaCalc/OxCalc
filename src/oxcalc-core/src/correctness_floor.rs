#![forbid(unsafe_code)]

//! Replay-visible correctness-floor selector profile.

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::error_algebra::ErrorAlgebra;
use crate::numerical_reduction::NumericalReductionPolicy;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CorrectnessFloorProfile {
    pub profile_version: String,
    pub numerical_reduction_policy: NumericalReductionPolicy,
    pub error_algebra: ErrorAlgebra,
}

impl Default for CorrectnessFloorProfile {
    fn default() -> Self {
        Self {
            profile_version: "profile:correctness-floor:v1".to_string(),
            numerical_reduction_policy: NumericalReductionPolicy::SequentialLeftFold,
            error_algebra: ErrorAlgebra::CanonicalExcelLegacy,
        }
    }
}

impl CorrectnessFloorProfile {
    #[must_use]
    pub fn new(
        profile_version: impl Into<String>,
        numerical_reduction_policy: NumericalReductionPolicy,
        error_algebra: ErrorAlgebra,
    ) -> Self {
        Self {
            profile_version: profile_version.into(),
            numerical_reduction_policy,
            error_algebra,
        }
    }

    #[must_use]
    pub fn replay_profile_key(&self) -> String {
        format!(
            "{}|numerical_reduction_policy:{}|error_algebra:{}",
            self.profile_version,
            self.numerical_reduction_policy.selector_key(),
            self.error_algebra.selector_key()
        )
    }

    #[must_use]
    pub fn replay_record(&self) -> CorrectnessFloorReplayRecord {
        CorrectnessFloorReplayRecord {
            profile_version: self.profile_version.clone(),
            numerical_reduction_policy: self.numerical_reduction_policy.selector_key().to_string(),
            error_algebra: self.error_algebra.selector_key().to_string(),
            semantic_kernel_metadata_version: None,
        }
    }

    pub fn validate_replay_record(
        recorded: &CorrectnessFloorReplayRecord,
        active: &Self,
    ) -> Result<(), CorrectnessFloorReplayValidationError> {
        let expected = active.replay_record();
        Self::validate_replay_record_against_active_record(recorded, &expected)
    }

    pub fn validate_replay_record_against_active_record(
        recorded: &CorrectnessFloorReplayRecord,
        active: &CorrectnessFloorReplayRecord,
    ) -> Result<(), CorrectnessFloorReplayValidationError> {
        if recorded == active {
            return Ok(());
        }

        Err(CorrectnessFloorReplayValidationError::SelectorMismatch {
            recorded: Box::new(recorded.clone()),
            active: Box::new(active.clone()),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CorrectnessFloorReplayRecord {
    pub profile_version: String,
    pub numerical_reduction_policy: String,
    pub error_algebra: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub semantic_kernel_metadata_version: Option<String>,
}

impl CorrectnessFloorReplayRecord {
    #[must_use]
    pub fn with_semantic_kernel_metadata_version(mut self, version: impl Into<String>) -> Self {
        self.semantic_kernel_metadata_version = Some(version.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CorrectnessFloorSelectorMismatchDiagnostic {
    pub diagnostic_code: String,
    pub mismatched_fields: Vec<String>,
    pub recorded: CorrectnessFloorReplayRecord,
    pub active: CorrectnessFloorReplayRecord,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CorrectnessFloorReplayValidationError {
    #[error("correctness-floor selector mismatch: recorded {recorded:?}; active {active:?}")]
    SelectorMismatch {
        recorded: Box<CorrectnessFloorReplayRecord>,
        active: Box<CorrectnessFloorReplayRecord>,
    },
}

impl CorrectnessFloorReplayValidationError {
    #[must_use]
    pub fn diagnostic(&self) -> CorrectnessFloorSelectorMismatchDiagnostic {
        match self {
            Self::SelectorMismatch { recorded, active } => {
                CorrectnessFloorSelectorMismatchDiagnostic {
                    diagnostic_code: "correctness_floor_selector_mismatch".to_string(),
                    mismatched_fields: correctness_floor_mismatched_fields(recorded, active),
                    recorded: (**recorded).clone(),
                    active: (**active).clone(),
                }
            }
        }
    }
}

fn correctness_floor_mismatched_fields(
    recorded: &CorrectnessFloorReplayRecord,
    active: &CorrectnessFloorReplayRecord,
) -> Vec<String> {
    let mut fields = Vec::new();
    if recorded.profile_version != active.profile_version {
        fields.push("profile_version".to_string());
    }
    if recorded.numerical_reduction_policy != active.numerical_reduction_policy {
        fields.push("numerical_reduction_policy".to_string());
    }
    if recorded.error_algebra != active.error_algebra {
        fields.push("error_algebra".to_string());
    }
    if recorded.semantic_kernel_metadata_version != active.semantic_kernel_metadata_version {
        fields.push("semantic_kernel_metadata_version".to_string());
    }
    fields
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use serde_json::json;

    use super::*;

    fn e4_artifact_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/test-runs/core-engine/w050-e4-profile-selector-tests-001")
    }

    fn default_profile() -> CorrectnessFloorProfile {
        CorrectnessFloorProfile::default()
    }

    fn explicit_profile() -> CorrectnessFloorProfile {
        CorrectnessFloorProfile::new(
            "profile:correctness-floor:pairwise-canonical",
            NumericalReductionPolicy::PairwiseTree,
            ErrorAlgebra::CanonicalExcelLegacy,
        )
    }

    fn numerical_policy_mismatch_diagnostic() -> CorrectnessFloorSelectorMismatchDiagnostic {
        let active = default_profile();
        let mut recorded = active.replay_record();
        recorded.numerical_reduction_policy = NumericalReductionPolicy::PairwiseTree
            .selector_key()
            .to_string();
        CorrectnessFloorProfile::validate_replay_record(&recorded, &active)
            .expect_err("numerical policy mismatch should reject replay")
            .diagnostic()
    }

    fn error_algebra_mismatch_diagnostic() -> CorrectnessFloorSelectorMismatchDiagnostic {
        let active = default_profile();
        let mut recorded = active.replay_record();
        recorded.error_algebra = "ProfileDeclaredTest".to_string();
        CorrectnessFloorProfile::validate_replay_record(&recorded, &active)
            .expect_err("error algebra mismatch should reject replay")
            .diagnostic()
    }

    fn selector_test_manifest_json() -> serde_json::Value {
        let default = default_profile();
        let explicit = explicit_profile();
        json!({
            "run_id": "w050-e4-profile-selector-tests-001",
            "validation_status": "pass",
            "primary_validation_command": "cargo test -p oxcalc-core correctness_floor_profile_selector -- --nocapture",
            "default_profile": {
                "profile": default,
                "replay_profile_key": default.replay_profile_key(),
                "replay_record": default.replay_record()
            },
            "explicit_profile": {
                "profile": explicit,
                "replay_profile_key": explicit.replay_profile_key(),
                "replay_record": explicit.replay_record()
            },
            "mismatch_diagnostics": [
                {
                    "case": "numerical_reduction_policy",
                    "diagnostic": numerical_policy_mismatch_diagnostic()
                },
                {
                    "case": "error_algebra",
                    "diagnostic": error_algebra_mismatch_diagnostic()
                }
            ],
            "spec_surfaces": [
                "docs/spec/core-engine/CORE_ENGINE_PROFILE_SELECTORS.md",
                "src/oxcalc-core/src/correctness_floor.rs"
            ]
        })
    }

    #[test]
    fn correctness_floor_profile_selector_defaults_are_replay_visible() {
        let profile = default_profile();

        assert_eq!(
            profile.replay_profile_key(),
            "profile:correctness-floor:v1|numerical_reduction_policy:SequentialLeftFold|error_algebra:CanonicalExcelLegacy"
        );
        assert_eq!(
            profile.replay_record(),
            CorrectnessFloorReplayRecord {
                profile_version: "profile:correctness-floor:v1".to_string(),
                numerical_reduction_policy: "SequentialLeftFold".to_string(),
                error_algebra: "CanonicalExcelLegacy".to_string(),
                semantic_kernel_metadata_version: None,
            }
        );

        let json = serde_json::to_value(&profile).unwrap();
        assert_eq!(json["numerical_reduction_policy"], "SequentialLeftFold");
        assert_eq!(json["error_algebra"], "CanonicalExcelLegacy");
    }

    #[test]
    fn correctness_floor_profile_selector_explicit_selection_round_trips() {
        let profile = explicit_profile();

        assert_eq!(
            profile.replay_profile_key(),
            "profile:correctness-floor:pairwise-canonical|numerical_reduction_policy:PairwiseTree|error_algebra:CanonicalExcelLegacy"
        );
        assert_eq!(
            profile.replay_record().numerical_reduction_policy,
            "PairwiseTree"
        );
        assert_eq!(
            profile.replay_record().error_algebra,
            "CanonicalExcelLegacy"
        );

        let round_trip: CorrectnessFloorProfile =
            serde_json::from_value(serde_json::to_value(&profile).unwrap()).unwrap();
        assert_eq!(round_trip, profile);
    }

    #[test]
    fn correctness_floor_profile_selector_mismatch_diagnostics_name_changed_selector() {
        let numerical_policy = numerical_policy_mismatch_diagnostic();
        assert_eq!(
            numerical_policy.diagnostic_code,
            "correctness_floor_selector_mismatch"
        );
        assert_eq!(
            numerical_policy.mismatched_fields,
            vec!["numerical_reduction_policy".to_string()]
        );
        assert_eq!(
            numerical_policy.recorded.numerical_reduction_policy,
            "PairwiseTree"
        );
        assert_eq!(
            numerical_policy.active.numerical_reduction_policy,
            "SequentialLeftFold"
        );

        let error_algebra = error_algebra_mismatch_diagnostic();
        assert_eq!(
            error_algebra.mismatched_fields,
            vec!["error_algebra".to_string()]
        );
        assert_eq!(error_algebra.recorded.error_algebra, "ProfileDeclaredTest");
        assert_eq!(error_algebra.active.error_algebra, "CanonicalExcelLegacy");
    }

    #[test]
    fn correctness_floor_replay_rejects_semantic_kernel_metadata_version_mismatch() {
        let active = default_profile()
            .replay_record()
            .with_semantic_kernel_metadata_version("semantic-kernel:v2");
        let recorded = default_profile()
            .replay_record()
            .with_semantic_kernel_metadata_version("semantic-kernel:v1");

        let diagnostic = CorrectnessFloorProfile::validate_replay_record_against_active_record(
            &recorded, &active,
        )
        .expect_err("metadata version mismatch should reject replay")
        .diagnostic();
        assert_eq!(
            diagnostic.mismatched_fields,
            vec!["semantic_kernel_metadata_version"]
        );

        let matching_record = CorrectnessFloorProfile::default()
            .replay_record()
            .with_semantic_kernel_metadata_version("semantic-kernel:v2");
        CorrectnessFloorProfile::validate_replay_record_against_active_record(
            &matching_record,
            &active,
        )
        .expect("matching metadata version should replay");
    }

    #[test]
    fn checked_in_correctness_floor_profile_selector_manifest_matches_runtime_diagnostics() {
        let artifact_path = e4_artifact_root().join("selector_test_manifest.json");
        let artifact = serde_json::from_str::<serde_json::Value>(
            &fs::read_to_string(artifact_path).expect("E4 selector manifest should be checked in"),
        )
        .expect("E4 selector manifest should be valid JSON");

        assert_eq!(artifact, selector_test_manifest_json());
    }
}
