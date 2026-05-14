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
        }
    }

    pub fn validate_replay_record(
        recorded: &CorrectnessFloorReplayRecord,
        active: &Self,
    ) -> Result<(), CorrectnessFloorReplayValidationError> {
        let expected = active.replay_record();
        if recorded == &expected {
            return Ok(());
        }

        Err(CorrectnessFloorReplayValidationError::SelectorMismatch {
            recorded: Box::new(recorded.clone()),
            active: Box::new(expected),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CorrectnessFloorReplayRecord {
    pub profile_version: String,
    pub numerical_reduction_policy: String,
    pub error_algebra: String,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CorrectnessFloorReplayValidationError {
    #[error("correctness-floor selector mismatch: recorded {recorded:?}; active {active:?}")]
    SelectorMismatch {
        recorded: Box<CorrectnessFloorReplayRecord>,
        active: Box<CorrectnessFloorReplayRecord>,
    },
}
