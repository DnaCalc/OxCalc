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
    LocalTreeCalcEngine, LocalTreeCalcEnvironmentContext, LocalTreeCalcError, LocalTreeCalcInput,
    LocalTreeCalcRunArtifacts, LocalTreeCalcRunState,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OxCalcTreeRuntimeLane {
    LocalSequentialTreeCalc,
}

impl OxCalcTreeRuntimeLane {
    #[must_use]
    pub fn as_diagnostic_value(&self) -> &'static str {
        match self {
            Self::LocalSequentialTreeCalc => "local_sequential_treecalc",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OxCalcTreeHostCapabilitySnapshot {
    pub capability_profile_id: String,
    pub dynamic_dependency_effects: bool,
    pub execution_restriction_effects: bool,
    pub capability_sensitive_effects: bool,
    pub shape_topology_effects: bool,
}

impl Default for OxCalcTreeHostCapabilitySnapshot {
    fn default() -> Self {
        Self {
            capability_profile_id: "host-capabilities:default".to_string(),
            dynamic_dependency_effects: true,
            execution_restriction_effects: true,
            capability_sensitive_effects: false,
            shape_topology_effects: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OxCalcTreeRuntimePolicy {
    pub policy_id: String,
    pub emit_environment_diagnostics: bool,
    pub project_runtime_effect_overlays: bool,
}

impl Default for OxCalcTreeRuntimePolicy {
    fn default() -> Self {
        Self {
            policy_id: "runtime-policy:default".to_string(),
            emit_environment_diagnostics: true,
            project_runtime_effect_overlays: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OxCalcTreeEnvironment {
    pub runtime_lane: OxCalcTreeRuntimeLane,
    pub session_id: Option<String>,
    pub host_capabilities: OxCalcTreeHostCapabilitySnapshot,
    pub runtime_policy: OxCalcTreeRuntimePolicy,
}

impl Default for OxCalcTreeEnvironment {
    fn default() -> Self {
        Self {
            runtime_lane: OxCalcTreeRuntimeLane::LocalSequentialTreeCalc,
            session_id: None,
            host_capabilities: OxCalcTreeHostCapabilitySnapshot::default(),
            runtime_policy: OxCalcTreeRuntimePolicy::default(),
        }
    }
}

impl OxCalcTreeEnvironment {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn with_session_id(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }

    #[must_use]
    pub fn with_host_capabilities(
        mut self,
        host_capabilities: OxCalcTreeHostCapabilitySnapshot,
    ) -> Self {
        self.host_capabilities = host_capabilities;
        self
    }

    #[must_use]
    pub fn with_runtime_policy(mut self, runtime_policy: OxCalcTreeRuntimePolicy) -> Self {
        self.runtime_policy = runtime_policy;
        self
    }

    #[must_use]
    pub fn runtime_context(&self) -> LocalTreeCalcEnvironmentContext {
        LocalTreeCalcEnvironmentContext {
            runtime_lane: self.runtime_lane.as_diagnostic_value().to_string(),
            session_id: self.session_id.clone(),
            capability_profile_id: self.host_capabilities.capability_profile_id.clone(),
            dynamic_dependency_effects: self.host_capabilities.dynamic_dependency_effects,
            execution_restriction_effects: self.host_capabilities.execution_restriction_effects,
            capability_sensitive_effects: self.host_capabilities.capability_sensitive_effects,
            shape_topology_effects: self.host_capabilities.shape_topology_effects,
            runtime_policy_id: self.runtime_policy.policy_id.clone(),
            project_runtime_effect_overlays: self.runtime_policy.project_runtime_effect_overlays,
        }
    }

    #[must_use]
    pub fn diagnostics(&self) -> Vec<String> {
        if !self.runtime_policy.emit_environment_diagnostics {
            return Vec::new();
        }

        let session_id = self.session_id.as_deref().unwrap_or("none");
        vec![
            format!(
                "oxcalc_tree_environment_runtime_lane:{}",
                self.runtime_lane.as_diagnostic_value()
            ),
            format!("oxcalc_tree_environment_session_id:{session_id}"),
            format!(
                "oxcalc_tree_environment_capability_profile_id:{}",
                self.host_capabilities.capability_profile_id
            ),
            format!(
                "oxcalc_tree_environment_capability_dynamic_dependency:{}",
                self.host_capabilities.dynamic_dependency_effects
            ),
            format!(
                "oxcalc_tree_environment_capability_execution_restriction:{}",
                self.host_capabilities.execution_restriction_effects
            ),
            format!(
                "oxcalc_tree_environment_capability_sensitive:{}",
                self.host_capabilities.capability_sensitive_effects
            ),
            format!(
                "oxcalc_tree_environment_capability_shape_topology:{}",
                self.host_capabilities.shape_topology_effects
            ),
            format!(
                "oxcalc_tree_environment_runtime_policy_id:{}",
                self.runtime_policy.policy_id
            ),
            format!(
                "oxcalc_tree_environment_project_runtime_effect_overlays:{}",
                self.runtime_policy.project_runtime_effect_overlays
            ),
        ]
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
            invalidation_seeds: Vec::new(),
            candidate_result_id: request.candidate_result_id,
            publication_id: request.publication_id,
            compatibility_basis: request.compatibility_basis,
            artifact_token_basis: request.artifact_token_basis,
            environment_context: self.environment.runtime_context(),
        })?;
        let mut result = OxCalcTreeRecalcResult::from(artifacts);
        result.diagnostics.extend(self.environment.diagnostics());
        Ok(result)
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
    use crate::coordinator::{RejectKind, RuntimeEffectFamily};
    use crate::formula::{FormulaBinaryOp, TreeFormula, TreeFormulaBinding};
    use crate::recalc::OverlayKind;
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
    fn treecalc_environment_carries_non_narrow_consumer_inputs() {
        let environment = OxCalcTreeEnvironment::new()
            .with_session_id("session:tree-host")
            .with_host_capabilities(OxCalcTreeHostCapabilitySnapshot {
                capability_profile_id: "capability-profile:tree-host".to_string(),
                dynamic_dependency_effects: true,
                execution_restriction_effects: true,
                capability_sensitive_effects: true,
                shape_topology_effects: true,
            })
            .with_runtime_policy(OxCalcTreeRuntimePolicy {
                policy_id: "runtime-policy:tree-host".to_string(),
                emit_environment_diagnostics: true,
                project_runtime_effect_overlays: true,
            });
        let facade = OxCalcTreeRuntimeFacade::new(environment.clone());

        assert_eq!(facade.environment(), &environment);
        assert_eq!(
            facade.environment().runtime_lane,
            OxCalcTreeRuntimeLane::LocalSequentialTreeCalc
        );
        assert_eq!(
            facade.environment().session_id.as_deref(),
            Some("session:tree-host")
        );
        assert_eq!(
            facade.environment().host_capabilities.capability_profile_id,
            "capability-profile:tree-host"
        );
        assert!(
            facade
                .environment()
                .host_capabilities
                .capability_sensitive_effects
        );
        assert!(
            facade
                .environment()
                .host_capabilities
                .shape_topology_effects
        );
        assert_eq!(
            facade.environment().runtime_policy.policy_id,
            "runtime-policy:tree-host"
        );
    }

    #[test]
    fn treecalc_runtime_facade_projects_environment_diagnostics() {
        let facade = OxCalcTreeRuntimeFacade::new(
            OxCalcTreeEnvironment::new()
                .with_session_id("session:diagnostic")
                .with_host_capabilities(OxCalcTreeHostCapabilitySnapshot {
                    capability_profile_id: "capability-profile:diagnostic".to_string(),
                    dynamic_dependency_effects: true,
                    execution_restriction_effects: true,
                    capability_sensitive_effects: true,
                    shape_topology_effects: false,
                })
                .with_runtime_policy(OxCalcTreeRuntimePolicy {
                    policy_id: "runtime-policy:diagnostic".to_string(),
                    emit_environment_diagnostics: true,
                    project_runtime_effect_overlays: true,
                }),
        );

        let result = facade
            .execute(
                OxCalcTreeDocument {
                    structural_snapshot: snapshot(),
                    formula_catalog: TreeFormulaCatalog::new([TreeFormulaBinding {
                        owner_node_id: TreeNodeId(3),
                        formula_artifact_id: FormulaArtifactId("formula:b".to_string()),
                        bind_artifact_id: Some(BindArtifactId("bind:b".to_string())),
                        expression: TreeFormula::Literal {
                            value: "7".to_string(),
                        },
                    }]),
                    seeded_published_values: BTreeMap::new(),
                },
                OxCalcTreeRecalcRequest {
                    candidate_result_id: "cand:environment".to_string(),
                    publication_id: "pub:environment".to_string(),
                    compatibility_basis: "snapshot:1".to_string(),
                    artifact_token_basis: "snapshot:1".to_string(),
                },
            )
            .unwrap();

        assert!(result.diagnostics.iter().any(|diagnostic| {
            diagnostic == "oxcalc_tree_environment_runtime_lane:local_sequential_treecalc"
        }));
        assert!(result.diagnostics.iter().any(|diagnostic| {
            diagnostic == "oxcalc_tree_environment_session_id:session:diagnostic"
        }));
        assert!(result.diagnostics.iter().any(|diagnostic| {
            diagnostic
                == "oxcalc_tree_environment_capability_profile_id:capability-profile:diagnostic"
        }));
        assert!(result.diagnostics.iter().any(|diagnostic| {
            diagnostic == "oxcalc_tree_environment_capability_sensitive:true"
        }));
        assert!(result.diagnostics.iter().any(|diagnostic| {
            diagnostic == "oxcalc_tree_environment_capability_shape_topology:false"
        }));
        assert!(result.diagnostics.iter().any(|diagnostic| {
            diagnostic == "oxcalc_tree_environment_runtime_policy_id:runtime-policy:diagnostic"
        }));
    }

    #[test]
    fn treecalc_runtime_derived_effects_use_environment_context() {
        let facade = OxCalcTreeRuntimeFacade::new(
            OxCalcTreeEnvironment::new()
                .with_session_id("session:runtime-effects")
                .with_host_capabilities(OxCalcTreeHostCapabilitySnapshot {
                    capability_profile_id: "capability-profile:runtime-effects".to_string(),
                    dynamic_dependency_effects: true,
                    execution_restriction_effects: true,
                    capability_sensitive_effects: false,
                    shape_topology_effects: false,
                })
                .with_runtime_policy(OxCalcTreeRuntimePolicy {
                    policy_id: "runtime-policy:no-overlays".to_string(),
                    emit_environment_diagnostics: true,
                    project_runtime_effect_overlays: false,
                }),
        );

        let result = facade
            .execute(
                OxCalcTreeDocument {
                    structural_snapshot: snapshot(),
                    formula_catalog: TreeFormulaCatalog::new([TreeFormulaBinding {
                        owner_node_id: TreeNodeId(3),
                        formula_artifact_id: FormulaArtifactId("formula:b".to_string()),
                        bind_artifact_id: Some(BindArtifactId("bind:b".to_string())),
                        expression: TreeFormula::Reference(
                            crate::formula::TreeReference::DynamicPotential {
                                carrier_id: "carrier:dynamic".to_string(),
                                detail: "late_bound_projection".to_string(),
                            },
                        ),
                    }]),
                    seeded_published_values: BTreeMap::new(),
                },
                OxCalcTreeRecalcRequest {
                    candidate_result_id: "cand:environment-runtime".to_string(),
                    publication_id: "pub:environment-runtime".to_string(),
                    compatibility_basis: "snapshot:1".to_string(),
                    artifact_token_basis: "snapshot:1".to_string(),
                },
            )
            .unwrap();

        assert_eq!(result.run_state, OxCalcTreeRunState::Rejected);
        assert!(result.publication_bundle.is_none());
        assert!(result.runtime_effect_overlays.is_empty());
        assert_eq!(result.runtime_effects.len(), 1);
        assert_eq!(
            result.runtime_effects[0].family,
            RuntimeEffectFamily::DynamicDependency
        );
        assert!(
            result.runtime_effects[0]
                .detail
                .contains("session_id:session:runtime-effects")
        );
        assert!(
            result.runtime_effects[0]
                .detail
                .contains("capability_profile_id:capability-profile:runtime-effects")
        );
        assert!(
            result.runtime_effects[0]
                .detail
                .contains("runtime_policy_id:runtime-policy:no-overlays")
        );
        assert!(result.diagnostics.iter().any(|diagnostic| {
            diagnostic == "runtime_effect_environment_session_id:session:runtime-effects"
        }));
        assert!(result.diagnostics.iter().any(|diagnostic| {
            diagnostic
                == "runtime_effect_environment_capability_profile_id:capability-profile:runtime-effects"
        }));
        assert!(result.diagnostics.iter().any(|diagnostic| {
            diagnostic == "runtime_effect_environment_project_overlays:false"
        }));
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

    #[test]
    fn treecalc_runtime_facade_exposes_execution_restriction_family_directly() {
        let facade = OxCalcTreeRuntimeFacade::new(OxCalcTreeEnvironment::new());

        let result = facade
            .execute(
                OxCalcTreeDocument {
                    structural_snapshot: snapshot(),
                    formula_catalog: TreeFormulaCatalog::new([TreeFormulaBinding {
                        owner_node_id: TreeNodeId(3),
                        formula_artifact_id: FormulaArtifactId("formula:b".to_string()),
                        bind_artifact_id: Some(BindArtifactId("bind:b".to_string())),
                        expression: TreeFormula::Reference(
                            crate::formula::TreeReference::HostSensitive {
                                carrier_id: "carrier:host".to_string(),
                                detail: "active_selection".to_string(),
                            },
                        ),
                    }]),
                    seeded_published_values: BTreeMap::new(),
                },
                OxCalcTreeRecalcRequest {
                    candidate_result_id: "cand:host".to_string(),
                    publication_id: "pub:host".to_string(),
                    compatibility_basis: "snapshot:1".to_string(),
                    artifact_token_basis: "snapshot:1".to_string(),
                },
            )
            .unwrap();

        assert_eq!(result.run_state, OxCalcTreeRunState::Rejected);
        assert!(result.publication_bundle.is_none());
        assert_eq!(
            result.reject_detail.map(|detail| detail.kind),
            Some(RejectKind::HostInjectedFailure)
        );
        assert_eq!(result.runtime_effects.len(), 1);
        assert_eq!(
            result.runtime_effects[0].family,
            RuntimeEffectFamily::ExecutionRestriction
        );
        assert_eq!(result.runtime_effect_overlays.len(), 1);
        assert_eq!(
            result.runtime_effect_overlays[0].key.overlay_kind,
            OverlayKind::ExecutionRestriction
        );
    }

    #[test]
    fn treecalc_runtime_facade_exposes_dynamic_dependency_family_directly() {
        let facade = OxCalcTreeRuntimeFacade::new(OxCalcTreeEnvironment::new());

        let result = facade
            .execute(
                OxCalcTreeDocument {
                    structural_snapshot: snapshot(),
                    formula_catalog: TreeFormulaCatalog::new([TreeFormulaBinding {
                        owner_node_id: TreeNodeId(3),
                        formula_artifact_id: FormulaArtifactId("formula:b".to_string()),
                        bind_artifact_id: Some(BindArtifactId("bind:b".to_string())),
                        expression: TreeFormula::Reference(
                            crate::formula::TreeReference::DynamicPotential {
                                carrier_id: "carrier:dynamic".to_string(),
                                detail: "late_bound_projection".to_string(),
                            },
                        ),
                    }]),
                    seeded_published_values: BTreeMap::new(),
                },
                OxCalcTreeRecalcRequest {
                    candidate_result_id: "cand:dynamic".to_string(),
                    publication_id: "pub:dynamic".to_string(),
                    compatibility_basis: "snapshot:1".to_string(),
                    artifact_token_basis: "snapshot:1".to_string(),
                },
            )
            .unwrap();

        assert_eq!(result.run_state, OxCalcTreeRunState::Rejected);
        assert_eq!(
            result.reject_detail.map(|detail| detail.kind),
            Some(RejectKind::DynamicDependencyFailure)
        );
        assert_eq!(result.runtime_effects.len(), 1);
        assert_eq!(
            result.runtime_effects[0].family,
            RuntimeEffectFamily::DynamicDependency
        );
        assert_eq!(result.runtime_effect_overlays.len(), 1);
        assert_eq!(
            result.runtime_effect_overlays[0].key.overlay_kind,
            OverlayKind::DynamicDependency
        );
    }
}
