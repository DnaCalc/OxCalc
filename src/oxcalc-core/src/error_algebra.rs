#![forbid(unsafe_code)]

//! Profile-governed worksheet error precedence algebra.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum WorksheetErrorKind {
    Null,
    Div0,
    Value,
    Ref,
    Name,
    Num,
    NotAvailable,
}

impl WorksheetErrorKind {
    #[must_use]
    pub fn display_text(self) -> &'static str {
        match self {
            Self::Null => "#NULL!",
            Self::Div0 => "#DIV/0!",
            Self::Value => "#VALUE!",
            Self::Ref => "#REF!",
            Self::Name => "#NAME?",
            Self::Num => "#NUM!",
            Self::NotAvailable => "#N/A",
        }
    }
}

pub const CANONICAL_EXCEL_LEGACY_ERROR_ORDER: [WorksheetErrorKind; 7] = [
    WorksheetErrorKind::Null,
    WorksheetErrorKind::Div0,
    WorksheetErrorKind::Value,
    WorksheetErrorKind::Ref,
    WorksheetErrorKind::Name,
    WorksheetErrorKind::Num,
    WorksheetErrorKind::NotAvailable,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ErrorAlgebra {
    CanonicalExcelLegacy,
}

impl ErrorAlgebra {
    #[must_use]
    pub fn selector_key(self) -> &'static str {
        match self {
            Self::CanonicalExcelLegacy => "CanonicalExcelLegacy",
        }
    }

    #[must_use]
    pub fn precedence_order(self) -> &'static [WorksheetErrorKind] {
        match self {
            Self::CanonicalExcelLegacy => &CANONICAL_EXCEL_LEGACY_ERROR_ORDER,
        }
    }

    #[must_use]
    pub fn rank(self, error: WorksheetErrorKind) -> usize {
        self.precedence_order()
            .iter()
            .position(|candidate| *candidate == error)
            .expect("canonical error order covers every admitted legacy error")
    }

    #[must_use]
    pub fn dominant_error(
        self,
        errors: impl IntoIterator<Item = WorksheetErrorKind>,
    ) -> Option<WorksheetErrorKind> {
        errors.into_iter().min_by_key(|error| self.rank(*error))
    }

    #[must_use]
    pub fn handoff_clause(self) -> &'static ErrorAlgebraClause {
        ERROR_ALGEBRA_HANDOFF_CLAUSES
            .iter()
            .find(|clause| clause.selector_value == Some(self))
            .expect("every error algebra selector has an exact handoff clause")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorAlgebraProfile {
    pub profile_version: String,
    pub error_algebra: ErrorAlgebra,
}

impl ErrorAlgebraProfile {
    #[must_use]
    pub fn new(profile_version: impl Into<String>, error_algebra: ErrorAlgebra) -> Self {
        Self {
            profile_version: profile_version.into(),
            error_algebra,
        }
    }

    #[must_use]
    pub fn replay_profile_key(&self) -> String {
        format!(
            "{}|error_algebra:{}",
            self.profile_version,
            self.error_algebra.selector_key()
        )
    }

    #[must_use]
    pub fn dominant_error(
        &self,
        errors: impl IntoIterator<Item = WorksheetErrorKind>,
    ) -> Option<WorksheetErrorKind> {
        self.error_algebra.dominant_error(errors)
    }

    #[must_use]
    pub fn handoff_clause(&self) -> &'static ErrorAlgebraClause {
        self.error_algebra.handoff_clause()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ErrorAlgebraClause {
    pub clause_id: &'static str,
    pub selector_value: Option<ErrorAlgebra>,
    pub exact_clause: &'static str,
}

pub const ERROR_ALGEBRA_HANDOFF_CLAUSES: [ErrorAlgebraClause; 2] = [
    ErrorAlgebraClause {
        clause_id: "CALC-003.ERR.CanonicalExcelLegacy",
        selector_value: Some(ErrorAlgebra::CanonicalExcelLegacy),
        exact_clause: "When a profile declares ErrorAlgebra=CanonicalExcelLegacy, OxFml/OxFunc kernels that must collapse multiple worksheet-error candidates into one observable result MUST select the earliest error in the precedence order #NULL!, #DIV/0!, #VALUE!, #REF!, #NAME?, #NUM!, #N/A; kernels MUST record the active error algebra in replay and MUST NOT substitute function-local or runtime-dependent precedence unless the active profile declares a different ErrorAlgebra selector.",
    },
    ErrorAlgebraClause {
        clause_id: "CALC-003.ERR.ExtensionRule",
        selector_value: None,
        exact_clause: "Any non-canonical ErrorAlgebra profile MUST use a new selector key and profile_version, MUST list a total precedence order over every admitted worksheet-error code plus explicit placement for newly admitted codes, and MUST be replay-invalid against traces recorded under CanonicalExcelLegacy unless an explicit migration proof is attached.",
    },
];

#[must_use]
pub fn error_algebra_handoff_clauses() -> &'static [ErrorAlgebraClause] {
    &ERROR_ALGEBRA_HANDOFF_CLAUSES
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use serde_json::json;

    use super::*;

    fn e2_artifact_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/test-runs/core-engine/w050-e2-error-algebra-selector-001")
    }

    fn selector_artifact_json() -> serde_json::Value {
        json!({
            "run_id": "w050-e2-error-algebra-selector-001",
            "selector_name": "ErrorAlgebra",
            "validation_status": "pass",
            "primary_validation_command": "cargo test -p oxcalc-core error_algebra -- --nocapture",
            "handoff_target": "HANDOFF_CALC_003_OXFML_NUMERICAL_REDUCTION_AND_ERROR_ALGEBRA.md",
            "profile_record": {
                "required_fields": [
                    "profile_version",
                    "error_algebra"
                ],
                "replay_key_format": "<profile_version>|error_algebra:<selector_key>"
            },
            "canonical_excel_legacy": {
                "selector_value": ErrorAlgebra::CanonicalExcelLegacy.selector_key(),
                "precedence_order": ErrorAlgebra::CanonicalExcelLegacy
                    .precedence_order()
                    .iter()
                    .enumerate()
                    .map(|(rank, error)| json!({
                        "rank": rank + 1,
                        "error": error.display_text(),
                    }))
                    .collect::<Vec<_>>()
            },
            "handoff_clauses": error_algebra_handoff_clauses()
                .iter()
                .map(|clause| json!({
                    "clause_id": clause.clause_id,
                    "selector_value": clause.selector_value.map(ErrorAlgebra::selector_key),
                    "exact_clause": clause.exact_clause,
                }))
                .collect::<Vec<_>>()
        })
    }

    #[test]
    fn error_algebra_profile_serializes_selector() {
        let profile = ErrorAlgebraProfile::new(
            "profile:correctness-floor:v1",
            ErrorAlgebra::CanonicalExcelLegacy,
        );

        assert_eq!(
            profile.replay_profile_key(),
            "profile:correctness-floor:v1|error_algebra:CanonicalExcelLegacy"
        );

        let json = serde_json::to_value(&profile).unwrap();
        assert_eq!(json["profile_version"], "profile:correctness-floor:v1");
        assert_eq!(json["error_algebra"], "CanonicalExcelLegacy");

        let round_trip: ErrorAlgebraProfile = serde_json::from_value(json).unwrap();
        assert_eq!(round_trip, profile);
    }

    #[test]
    fn canonical_error_algebra_orders_legacy_errors() {
        assert_eq!(
            ErrorAlgebra::CanonicalExcelLegacy
                .precedence_order()
                .iter()
                .map(|error| error.display_text())
                .collect::<Vec<_>>(),
            vec![
                "#NULL!", "#DIV/0!", "#VALUE!", "#REF!", "#NAME?", "#NUM!", "#N/A"
            ]
        );

        let profile = ErrorAlgebraProfile::new(
            "profile:correctness-floor:v1",
            ErrorAlgebra::CanonicalExcelLegacy,
        );
        assert_eq!(
            profile.dominant_error([
                WorksheetErrorKind::NotAvailable,
                WorksheetErrorKind::Num,
                WorksheetErrorKind::Div0,
            ]),
            Some(WorksheetErrorKind::Div0)
        );
        assert_eq!(
            profile.dominant_error([WorksheetErrorKind::Name, WorksheetErrorKind::Value]),
            Some(WorksheetErrorKind::Value)
        );
    }

    #[test]
    fn error_algebra_clauses_are_handoff_ready() {
        let clauses = error_algebra_handoff_clauses();
        assert_eq!(clauses.len(), 2);

        let canonical = ErrorAlgebra::CanonicalExcelLegacy.handoff_clause();
        assert_eq!(canonical.clause_id, "CALC-003.ERR.CanonicalExcelLegacy");
        assert_eq!(
            canonical.selector_value,
            Some(ErrorAlgebra::CanonicalExcelLegacy)
        );
        assert!(
            canonical
                .exact_clause
                .contains("ErrorAlgebra=CanonicalExcelLegacy")
        );
        assert!(
            canonical
                .exact_clause
                .contains("#NULL!, #DIV/0!, #VALUE!, #REF!, #NAME?, #NUM!, #N/A")
        );
        assert!(canonical.exact_clause.contains("MUST"));

        let extension = clauses
            .iter()
            .find(|clause| clause.clause_id == "CALC-003.ERR.ExtensionRule")
            .expect("extension rule clause should exist");
        assert_eq!(extension.selector_value, None);
        assert!(extension.exact_clause.contains("new selector key"));
        assert!(extension.exact_clause.contains("replay-invalid"));
    }

    #[test]
    fn checked_in_error_algebra_artifact_matches_runtime_clauses() {
        let artifact_path = e2_artifact_root().join("selector_artifact.json");
        let artifact = serde_json::from_str::<serde_json::Value>(
            &fs::read_to_string(artifact_path).expect("E2 selector artifact should be checked in"),
        )
        .expect("E2 selector artifact should be valid JSON");

        assert_eq!(artifact, selector_artifact_json());
    }
}
