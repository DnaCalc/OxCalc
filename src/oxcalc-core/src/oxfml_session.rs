#![forbid(unsafe_code)]

//! OxCalc-side driver for the public OxFml runtime session facade.

use oxfml_core::consumer::runtime::{
    RuntimeEnvironment, RuntimeFormulaRequest, RuntimeFormulaResult, RuntimeManagedCommitResult,
    RuntimeManagedOpenResult, RuntimeManagedSessionDiagnostics, RuntimeManagedSessionError,
    RuntimeSessionFacade,
};
use oxfml_core::seam::{FenceSnapshot, RejectRecord};
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum OxfmlRecalcSessionError {
    #[error("OxFml runtime preparation failed during {phase}: {detail}")]
    Preparation { phase: &'static str, detail: String },
    #[error("OxFml runtime session rejected during {phase}: {reject:?}")]
    Reject {
        phase: &'static str,
        reject: Box<RejectRecord>,
    },
    #[error("OxFml runtime invocation failed: {detail}")]
    Invocation { detail: String },
}

pub struct OxfmlRecalcSessionDriver<'a> {
    session: RuntimeSessionFacade<'a>,
}

impl<'a> OxfmlRecalcSessionDriver<'a> {
    #[must_use]
    pub fn new(environment: RuntimeEnvironment<'a>) -> Self {
        Self {
            session: environment.open_session(),
        }
    }

    #[must_use]
    pub fn from_session(session: RuntimeSessionFacade<'a>) -> Self {
        Self { session }
    }

    pub fn ensure_prepared<'q>(
        &mut self,
        request: &RuntimeFormulaRequest<'q>,
    ) -> Result<RuntimeManagedOpenResult, OxfmlRecalcSessionError> {
        self.session
            .open_managed_session(request)
            .map_err(|error| map_managed_error("ensure_prepared", error))
    }

    pub fn invoke<'q>(
        &mut self,
        request: RuntimeFormulaRequest<'q>,
    ) -> Result<RuntimeFormulaResult, OxfmlRecalcSessionError> {
        self.session
            .execute(request)
            .map_err(|detail| OxfmlRecalcSessionError::Invocation { detail })
    }

    pub fn invoke_managed_commit<'q>(
        &mut self,
        request: RuntimeFormulaRequest<'q>,
        commit_attempt_id: impl Into<String>,
    ) -> Result<RuntimeManagedCommitResult, OxfmlRecalcSessionError> {
        self.session
            .execute_and_commit_managed(request, commit_attempt_id)
            .map_err(|error| map_managed_error("invoke_managed_commit", error))
    }

    pub fn commit_prepared(
        &mut self,
        commit_attempt_id: impl Into<String>,
        observed_fence: FenceSnapshot,
    ) -> Result<RuntimeManagedCommitResult, OxfmlRecalcSessionError> {
        self.session
            .commit_managed(commit_attempt_id, observed_fence)
            .map_err(|error| map_managed_error("commit_prepared", error))
    }

    #[must_use]
    pub fn managed_session_diagnostics(&self) -> Option<RuntimeManagedSessionDiagnostics> {
        self.session.managed_session_diagnostics()
    }
}

fn map_managed_error(
    phase: &'static str,
    error: RuntimeManagedSessionError,
) -> OxfmlRecalcSessionError {
    match error {
        RuntimeManagedSessionError::Preparation(detail) => {
            OxfmlRecalcSessionError::Preparation { phase, detail }
        }
        RuntimeManagedSessionError::Reject(reject) => OxfmlRecalcSessionError::Reject {
            phase,
            reject: Box::new(reject),
        },
    }
}

#[cfg(test)]
mod tests {
    use oxfml_core::EvaluationBackend;
    use oxfml_core::consumer::runtime::{
        RuntimeEnvironment, RuntimeFormulaRequest, RuntimeManagedSessionPhase,
    };
    use oxfml_core::interface::TypedContextQueryBundle;
    use oxfml_core::seam::{AcceptDecision, ValuePayload};
    use oxfml_core::source::FormulaSourceRecord;

    use super::*;

    fn request(formula_stable_id: &str, formula_text: &str) -> RuntimeFormulaRequest<'static> {
        RuntimeFormulaRequest::new(
            FormulaSourceRecord::new(formula_stable_id, 1, formula_text),
            TypedContextQueryBundle::default(),
        )
        .with_backend(EvaluationBackend::OxFuncBacked)
    }

    #[test]
    fn session_driver_maps_ensure_prepared_to_managed_open() {
        let mut driver = OxfmlRecalcSessionDriver::new(RuntimeEnvironment::new());
        let request = request("calc:session:prepared", "=SUM(1,2)");

        let open = driver
            .ensure_prepared(&request)
            .expect("managed preparation should succeed through OxFml V1");

        assert_eq!(
            open.semantic_plan.formula_stable_id,
            "calc:session:prepared"
        );
        assert!(open.session_id.starts_with("session:"));
        assert!(open.syntax_diagnostics.is_empty());
        assert!(open.bind_diagnostics.is_empty());
        let diagnostics = driver
            .managed_session_diagnostics()
            .expect("managed session diagnostics should be available after preparation");
        assert_eq!(diagnostics.phase, RuntimeManagedSessionPhase::Open);
        assert_eq!(diagnostics.formula_stable_id, "calc:session:prepared");
    }

    #[test]
    fn session_driver_invokes_full_runtime_result_through_session_facade() {
        let mut driver = OxfmlRecalcSessionDriver::new(RuntimeEnvironment::new());
        let request = request("calc:session:invoke", "=SUM(1,2)");

        let run = driver
            .invoke(request)
            .expect("session facade invocation should return a runtime result");

        assert_eq!(
            run.candidate_result.value_delta.published_payload,
            ValuePayload::Number("3".to_string())
        );
        assert!(matches!(run.commit_decision, AcceptDecision::Accepted(_)));
        assert_eq!(run.source.formula_stable_id.0, "calc:session:invoke");
    }

    #[test]
    fn session_driver_reuses_facade_host_artifacts_for_repeated_formula() {
        let mut driver = OxfmlRecalcSessionDriver::new(RuntimeEnvironment::new());
        let request = request("calc:session:repeat", "=SUM(1,2)");

        let first = driver
            .invoke(request.clone())
            .expect("first invocation should return a runtime result");
        let second = driver
            .invoke(request)
            .expect("second invocation should return a runtime result");

        assert!(!first.artifact_reuse.green_tree_reused);
        assert!(second.artifact_reuse.green_tree_reused);
        assert!(second.artifact_reuse.red_projection_reused);
        assert!(second.artifact_reuse.bound_formula_reused);
        assert!(second.artifact_reuse.semantic_plan_reused);
    }

    #[test]
    fn session_driver_can_drive_current_v1_managed_commit_surface() {
        let mut driver = OxfmlRecalcSessionDriver::new(RuntimeEnvironment::new());
        let request = request("calc:session:managed-commit", "=SUM(2,3)");

        let commit = driver
            .invoke_managed_commit(request, "commit:calc-session-managed")
            .expect("managed commit should return a structured OxFml decision");

        assert_eq!(commit.session.phase, RuntimeManagedSessionPhase::Committed);
        match commit.commit_decision {
            AcceptDecision::Accepted(bundle) => {
                assert_eq!(
                    bundle.value_delta.published_payload,
                    ValuePayload::Number("5".to_string())
                );
            }
            AcceptDecision::Rejected(reject) => {
                panic!("expected accepted commit, got reject: {reject:?}");
            }
        }
    }
}
