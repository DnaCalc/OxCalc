#![forbid(unsafe_code)]

//! Consumer-facing OxCalc runtime contract for tree-substrate hosts.

use std::collections::BTreeMap;

use thiserror::Error;

use crate::coordinator::{AcceptedCandidateResult, PublicationBundle, RejectDetail, RuntimeEffect};
use crate::dependency::{DependencyGraph, InvalidationClosure};
use crate::formula::TreeFormulaCatalog;
use crate::recalc::{NodeCalcState, OverlayEntry};
use crate::structural::{StructuralSnapshot, TreeNodeId};
use crate::treecalc::{
    LocalTreeCalcEngine, LocalTreeCalcError, LocalTreeCalcInput, LocalTreeCalcRunArtifacts,
    LocalTreeCalcRunState,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OxCalcTreeDocument {
    pub structural_snapshot: StructuralSnapshot,
    pub formula_catalog: TreeFormulaCatalog,
    pub seeded_published_values: BTreeMap<TreeNodeId, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OxCalcTreeRecalcRequest {
    pub candidate_result_id: String,
    pub publication_id: String,
    pub compatibility_basis: String,
    pub artifact_token_basis: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OxCalcTreeRunState {
    Published,
    VerifiedClean,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OxCalcTreeRecalcResult {
    pub run_state: OxCalcTreeRunState,
    pub dependency_graph: DependencyGraph,
    pub invalidation_closure: InvalidationClosure,
    pub evaluation_order: Vec<TreeNodeId>,
    pub runtime_effects: Vec<RuntimeEffect>,
    pub runtime_effect_overlays: Vec<OverlayEntry>,
    pub candidate_result: Option<AcceptedCandidateResult>,
    pub publication_bundle: Option<PublicationBundle>,
    pub reject_detail: Option<RejectDetail>,
    pub published_values: BTreeMap<TreeNodeId, String>,
    pub node_states: BTreeMap<TreeNodeId, NodeCalcState>,
    pub diagnostics: Vec<String>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum OxCalcTreeRuntimeError {
    #[error(transparent)]
    Runtime(#[from] LocalTreeCalcError),
}

#[derive(Debug, Clone, Default)]
pub struct OxCalcTreeEnvironment;

impl OxCalcTreeEnvironment {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

#[derive(Debug, Clone, Default)]
pub struct OxCalcTreeRuntimeFacade {
    environment: OxCalcTreeEnvironment,
    engine: LocalTreeCalcEngine,
}

impl OxCalcTreeRuntimeFacade {
    #[must_use]
    pub fn new(environment: OxCalcTreeEnvironment) -> Self {
        Self {
            environment,
            engine: LocalTreeCalcEngine,
        }
    }

    #[must_use]
    pub fn environment(&self) -> &OxCalcTreeEnvironment {
        &self.environment
    }

    pub fn execute(
        &self,
        document: OxCalcTreeDocument,
        request: OxCalcTreeRecalcRequest,
    ) -> Result<OxCalcTreeRecalcResult, OxCalcTreeRuntimeError> {
        let artifacts = self.engine.execute(LocalTreeCalcInput {
            structural_snapshot: document.structural_snapshot,
            formula_catalog: document.formula_catalog,
            seeded_published_values: document.seeded_published_values,
            candidate_result_id: request.candidate_result_id,
            publication_id: request.publication_id,
            compatibility_basis: request.compatibility_basis,
            artifact_token_basis: request.artifact_token_basis,
        })?;
        Ok(OxCalcTreeRecalcResult::from(artifacts))
    }
}

impl From<LocalTreeCalcRunState> for OxCalcTreeRunState {
    fn from(value: LocalTreeCalcRunState) -> Self {
        match value {
            LocalTreeCalcRunState::Published => Self::Published,
            LocalTreeCalcRunState::VerifiedClean => Self::VerifiedClean,
            LocalTreeCalcRunState::Rejected => Self::Rejected,
        }
    }
}

impl From<LocalTreeCalcRunArtifacts> for OxCalcTreeRecalcResult {
    fn from(value: LocalTreeCalcRunArtifacts) -> Self {
        Self {
            run_state: value.result_state.into(),
            dependency_graph: value.dependency_graph,
            invalidation_closure: value.invalidation_closure,
            evaluation_order: value.evaluation_order,
            runtime_effects: value.runtime_effects,
            runtime_effect_overlays: value.runtime_effect_overlays,
            candidate_result: value.candidate_result,
            publication_bundle: value.publication_bundle,
            reject_detail: value.reject_detail,
            published_values: value.published_values,
            node_states: value.node_states,
            diagnostics: value.diagnostics,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formula::{FormulaBinaryOp, TreeFormula, TreeFormulaBinding};
    use crate::structural::{
        BindArtifactId, FormulaArtifactId, StructuralNode, StructuralNodeKind, StructuralSnapshot,
        StructuralSnapshotId,
    };

    fn snapshot() -> StructuralSnapshot {
        StructuralSnapshot::create(
            StructuralSnapshotId(1),
            TreeNodeId(1),
            [
                StructuralNode {
                    node_id: TreeNodeId(1),
                    kind: StructuralNodeKind::Root,
                    symbol: "Root".to_string(),
                    parent_id: None,
                    child_ids: vec![TreeNodeId(2), TreeNodeId(3)],
                    formula_artifact_id: None,
                    bind_artifact_id: None,
                    constant_value: None,
                },
                StructuralNode {
                    node_id: TreeNodeId(2),
                    kind: StructuralNodeKind::Constant,
                    symbol: "A".to_string(),
                    parent_id: Some(TreeNodeId(1)),
                    child_ids: vec![],
                    formula_artifact_id: None,
                    bind_artifact_id: None,
                    constant_value: Some("2".to_string()),
                },
                StructuralNode {
                    node_id: TreeNodeId(3),
                    kind: StructuralNodeKind::Calculation,
                    symbol: "B".to_string(),
                    parent_id: Some(TreeNodeId(1)),
                    child_ids: vec![],
                    formula_artifact_id: Some(FormulaArtifactId("formula:b".to_string())),
                    bind_artifact_id: Some(BindArtifactId("bind:b".to_string())),
                    constant_value: None,
                },
            ],
        )
        .unwrap()
    }

    #[test]
    fn treecalc_runtime_facade_executes_published_run() {
        let facade = OxCalcTreeRuntimeFacade::new(OxCalcTreeEnvironment::new());

        let result = facade
            .execute(
                OxCalcTreeDocument {
                    structural_snapshot: snapshot(),
                    formula_catalog: TreeFormulaCatalog::new([TreeFormulaBinding {
                        owner_node_id: TreeNodeId(3),
                        formula_artifact_id: FormulaArtifactId("formula:b".to_string()),
                        bind_artifact_id: Some(BindArtifactId("bind:b".to_string())),
                        expression: TreeFormula::Binary {
                            op: FormulaBinaryOp::Add,
                            left: Box::new(TreeFormula::Reference(
                                crate::formula::TreeReference::DirectNode {
                                    target_node_id: TreeNodeId(2),
                                },
                            )),
                            right: Box::new(TreeFormula::Literal {
                                value: "3".to_string(),
                            }),
                        },
                    }]),
                    seeded_published_values: BTreeMap::new(),
                },
                OxCalcTreeRecalcRequest {
                    candidate_result_id: "cand:consumer".to_string(),
                    publication_id: "pub:consumer".to_string(),
                    compatibility_basis: "snapshot:1".to_string(),
                    artifact_token_basis: "snapshot:1".to_string(),
                },
            )
            .unwrap();

        assert_eq!(result.run_state, OxCalcTreeRunState::Published);
        assert_eq!(result.published_values[&TreeNodeId(3)], "5");
        assert!(result.publication_bundle.is_some());
    }
}
