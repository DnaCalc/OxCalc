#![forbid(unsafe_code)]

//! TreeCalc-local formula and reference substrate.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::dependency::{DependencyDescriptor, DependencyDescriptorKind};
use crate::structural::{BindArtifactId, FormulaArtifactId, StructuralSnapshot, TreeNodeId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum RelativeReferenceBase {
    SelfNode,
    ParentNode,
    Ancestor(usize),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TreeReference {
    DirectNode {
        target_node_id: TreeNodeId,
    },
    ProjectionPath {
        projection_path: String,
    },
    RelativePath {
        base: RelativeReferenceBase,
        path_segments: Vec<String>,
    },
    SiblingOffset {
        offset: isize,
        tail_segments: Vec<String>,
    },
    HostSensitive {
        carrier_id: String,
        detail: String,
    },
    CapabilitySensitive {
        carrier_id: String,
        detail: String,
    },
    ShapeTopology {
        carrier_id: String,
        detail: String,
    },
    DynamicPotential {
        carrier_id: String,
        detail: String,
    },
    DynamicResolved {
        target_node_id: TreeNodeId,
        carrier_id: String,
        detail: String,
    },
    Unresolved {
        token: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreeReferenceCarrierClass {
    FormulaReference,
    RuntimeFactProjection,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreeFormulaReferenceCarrier {
    pub source_token: Option<String>,
    pub reference: TreeReference,
}

impl TreeFormulaReferenceCarrier {
    #[must_use]
    pub fn named(source_token: impl Into<String>, reference: TreeReference) -> Self {
        Self {
            source_token: Some(source_token.into()),
            reference,
        }
    }

    #[must_use]
    pub fn fact(reference: TreeReference) -> Self {
        Self {
            source_token: None,
            reference,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreeFormula {
    pub source_text: String,
    #[serde(default)]
    pub reference_carriers: Vec<TreeFormulaReferenceCarrier>,
    #[serde(default)]
    pub lazy_residual_publication: bool,
}

impl TreeFormula {
    #[must_use]
    pub fn opaque_oxfml(
        source_text: impl Into<String>,
        reference_carriers: impl IntoIterator<Item = TreeFormulaReferenceCarrier>,
    ) -> Self {
        Self {
            source_text: normalize_formula_source(source_text.into()),
            reference_carriers: reference_carriers.into_iter().collect(),
            lazy_residual_publication: false,
        }
    }

    #[must_use]
    pub fn with_lazy_residual_publication(mut self, lazy_residual_publication: bool) -> Self {
        self.lazy_residual_publication = lazy_residual_publication;
        self
    }

    #[must_use]
    pub fn source_text(&self) -> &str {
        &self.source_text
    }

    #[must_use]
    pub fn reference_carriers(&self) -> &[TreeFormulaReferenceCarrier] {
        &self.reference_carriers
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum FixtureFormulaBinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FixtureFormulaPolicyClass {
    OpaqueOxfmlSource,
    LegacyStructuredQuarantine,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FixtureFormulaAst {
    Literal {
        value: String,
    },
    Reference(TreeReference),
    Binary {
        op: FixtureFormulaBinaryOp,
        left: Box<FixtureFormulaAst>,
        right: Box<FixtureFormulaAst>,
    },
    FunctionCall {
        function_name: String,
        arguments: Vec<FixtureFormulaAst>,
        may_introduce_dynamic_dependencies: bool,
    },
    /// Fixture-facing source carriage for OxFml helper-carrier witnesses whose
    /// syntax cannot be represented by the structured TreeFormula subset yet.
    RawOxfml {
        source_text: String,
        #[serde(default)]
        reference_carriers: Vec<TreeReference>,
    },
}

impl FixtureFormulaAst {
    #[must_use]
    pub fn policy_class(&self) -> FixtureFormulaPolicyClass {
        match self {
            FixtureFormulaAst::RawOxfml { .. } => FixtureFormulaPolicyClass::OpaqueOxfmlSource,
            FixtureFormulaAst::Literal { .. }
            | FixtureFormulaAst::Reference(_)
            | FixtureFormulaAst::Binary { .. }
            | FixtureFormulaAst::FunctionCall { .. } => {
                FixtureFormulaPolicyClass::LegacyStructuredQuarantine
            }
        }
    }

    #[must_use]
    pub fn to_tree_formula(&self, owner_node_id: TreeNodeId) -> TreeFormula {
        let mut state = FixtureFormulaRenderState {
            owner_node_id,
            next_reference_index: 0,
            reference_carriers: Vec::new(),
        };
        let source_text = state.render_formula(self);
        let lazy_residual_publication = matches!(
            self,
            FixtureFormulaAst::FunctionCall { function_name, .. }
                if function_name.eq_ignore_ascii_case("IF")
        );
        TreeFormula::opaque_oxfml(source_text, state.reference_carriers)
            .with_lazy_residual_publication(lazy_residual_publication)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreeFormulaBinding {
    pub owner_node_id: TreeNodeId,
    pub formula_artifact_id: FormulaArtifactId,
    pub bind_artifact_id: Option<BindArtifactId>,
    pub expression: TreeFormula,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TreeFormulaCatalog {
    bindings_by_owner: BTreeMap<TreeNodeId, TreeFormulaBinding>,
}

impl TreeFormulaCatalog {
    #[must_use]
    pub fn new(bindings: impl IntoIterator<Item = TreeFormulaBinding>) -> Self {
        let bindings_by_owner = bindings
            .into_iter()
            .map(|binding| (binding.owner_node_id, binding))
            .collect::<BTreeMap<_, _>>();
        Self { bindings_by_owner }
    }

    #[must_use]
    pub fn bindings_by_owner(&self) -> &BTreeMap<TreeNodeId, TreeFormulaBinding> {
        &self.bindings_by_owner
    }

    pub fn try_get_binding(&self, owner_node_id: TreeNodeId) -> Option<&TreeFormulaBinding> {
        self.bindings_by_owner.get(&owner_node_id)
    }

    #[must_use]
    pub fn owner_node_ids(&self) -> Vec<TreeNodeId> {
        self.bindings_by_owner.keys().copied().collect()
    }

    #[must_use]
    pub fn to_dependency_descriptors(
        &self,
        snapshot: &StructuralSnapshot,
    ) -> Vec<DependencyDescriptor> {
        let mut descriptors = Vec::new();

        for binding in self.bindings_by_owner.values() {
            descriptors.extend(
                binding
                    .expression
                    .reference_carriers()
                    .iter()
                    .enumerate()
                    .map(|(index, carrier)| lower_reference(snapshot, binding, carrier, index)),
            );
        }

        descriptors.sort_by(|left, right| left.descriptor_id.cmp(&right.descriptor_id));
        descriptors
    }
}

fn lower_reference(
    snapshot: &StructuralSnapshot,
    binding: &TreeFormulaBinding,
    carrier: &TreeFormulaReferenceCarrier,
    index: usize,
) -> DependencyDescriptor {
    let descriptor_id = format!("bind:{}:ref:{index}", binding.formula_artifact_id.0);
    let reference = &carrier.reference;
    let kind = reference.descriptor_kind();
    let target_node_id = reference.resolve_target(snapshot, binding.owner_node_id);
    let carrier_detail = reference.carrier_detail();
    let requires_rebind_on_structural_change = reference.requires_rebind_on_structural_change();

    DependencyDescriptor {
        descriptor_id,
        source_reference_handle: carrier
            .source_token
            .as_ref()
            .map(|token| format!("oxcalc_source_token:{token}")),
        owner_node_id: binding.owner_node_id,
        target_node_id,
        kind,
        carrier_detail,
        requires_rebind_on_structural_change,
    }
}

struct FixtureFormulaRenderState {
    owner_node_id: TreeNodeId,
    next_reference_index: usize,
    reference_carriers: Vec<TreeFormulaReferenceCarrier>,
}

impl FixtureFormulaRenderState {
    fn render_formula(&mut self, formula: &FixtureFormulaAst) -> String {
        match formula {
            FixtureFormulaAst::Literal { value } => render_literal(value),
            FixtureFormulaAst::Reference(reference) => self.render_reference(reference),
            FixtureFormulaAst::Binary { op, left, right } => {
                let left = self.render_formula(left);
                let right = self.render_formula(right);
                let operator = match op {
                    FixtureFormulaBinaryOp::Add => "+",
                    FixtureFormulaBinaryOp::Subtract => "-",
                    FixtureFormulaBinaryOp::Multiply => "*",
                    FixtureFormulaBinaryOp::Divide => "/",
                };
                format!("({left}{operator}{right})")
            }
            FixtureFormulaAst::FunctionCall {
                function_name,
                arguments,
                ..
            } => {
                let arguments = arguments
                    .iter()
                    .map(|argument| self.render_formula(argument))
                    .collect::<Vec<_>>();
                format!(
                    "{}({})",
                    function_name.to_ascii_uppercase(),
                    arguments.join(",")
                )
            }
            FixtureFormulaAst::RawOxfml {
                source_text,
                reference_carriers,
            } => {
                for reference in reference_carriers {
                    self.record_reference(reference);
                }
                source_text.trim_start_matches('=').to_string()
            }
        }
    }

    fn render_reference(&mut self, reference: &TreeReference) -> String {
        match reference {
            TreeReference::DirectNode { .. }
            | TreeReference::ProjectionPath { .. }
            | TreeReference::RelativePath { .. }
            | TreeReference::SiblingOffset { .. }
            | TreeReference::DynamicResolved { .. } => self.record_named_reference(reference),
            TreeReference::Unresolved { .. } => self.record_unresolved_reference(reference),
            TreeReference::HostSensitive { .. } => {
                self.record_fact(reference);
                "INFO(\"system\")".to_string()
            }
            TreeReference::CapabilitySensitive { .. } => {
                self.record_fact(reference);
                "INFO(\"osversion\")".to_string()
            }
            TreeReference::ShapeTopology { .. } => {
                self.record_fact(reference);
                "ROWS(A1:A1)".to_string()
            }
            TreeReference::DynamicPotential { carrier_id, .. } => {
                self.record_fact(reference);
                let topic = escape_excel_text(carrier_id);
                format!("RTD(\"TREECALC\",\"\",\"{topic}\")")
            }
        }
    }

    fn record_reference(&mut self, reference: &TreeReference) {
        match reference {
            TreeReference::DirectNode { .. }
            | TreeReference::ProjectionPath { .. }
            | TreeReference::RelativePath { .. }
            | TreeReference::SiblingOffset { .. }
            | TreeReference::DynamicResolved { .. } => {
                let _ = self.record_named_reference(reference);
            }
            TreeReference::Unresolved { .. } => {
                let _ = self.record_unresolved_reference(reference);
            }
            TreeReference::HostSensitive { .. }
            | TreeReference::CapabilitySensitive { .. }
            | TreeReference::ShapeTopology { .. }
            | TreeReference::DynamicPotential { .. } => self.record_fact(reference),
        }
    }

    fn record_named_reference(&mut self, reference: &TreeReference) -> String {
        let token = format!(
            "TREE_REF_{}_{}",
            self.owner_node_id.0, self.next_reference_index
        );
        self.next_reference_index += 1;
        self.reference_carriers
            .push(TreeFormulaReferenceCarrier::named(
                token.clone(),
                reference.clone(),
            ));
        token
    }

    fn record_unresolved_reference(&mut self, reference: &TreeReference) -> String {
        let token = format!(
            "TREE_UNRESOLVED_{}_{}",
            self.owner_node_id.0, self.next_reference_index
        );
        self.next_reference_index += 1;
        self.reference_carriers
            .push(TreeFormulaReferenceCarrier::named(
                token.clone(),
                reference.clone(),
            ));
        token
    }

    fn record_fact(&mut self, reference: &TreeReference) {
        self.reference_carriers
            .push(TreeFormulaReferenceCarrier::fact(reference.clone()));
    }
}

fn normalize_formula_source(source_text: String) -> String {
    if source_text.starts_with('=') {
        source_text
    } else {
        format!("={source_text}")
    }
}

fn render_literal(value: &str) -> String {
    if value.parse::<f64>().is_ok() {
        value.to_string()
    } else {
        format!("\"{}\"", escape_excel_text(value))
    }
}

fn escape_excel_text(value: &str) -> String {
    value.replace('"', "\"\"")
}

impl TreeReference {
    #[must_use]
    pub fn carrier_class(&self) -> TreeReferenceCarrierClass {
        match self {
            TreeReference::HostSensitive { .. }
            | TreeReference::CapabilitySensitive { .. }
            | TreeReference::ShapeTopology { .. }
            | TreeReference::DynamicPotential { .. } => {
                TreeReferenceCarrierClass::RuntimeFactProjection
            }
            TreeReference::DirectNode { .. }
            | TreeReference::ProjectionPath { .. }
            | TreeReference::RelativePath { .. }
            | TreeReference::SiblingOffset { .. }
            | TreeReference::DynamicResolved { .. }
            | TreeReference::Unresolved { .. } => TreeReferenceCarrierClass::FormulaReference,
        }
    }

    pub fn resolve_target(
        &self,
        snapshot: &StructuralSnapshot,
        owner_node_id: TreeNodeId,
    ) -> Option<TreeNodeId> {
        match self {
            TreeReference::DirectNode { target_node_id } => Some(*target_node_id),
            TreeReference::ProjectionPath { projection_path } => {
                snapshot.try_resolve_projection_path(projection_path)
            }
            TreeReference::RelativePath {
                base,
                path_segments,
            } => {
                let base_node_id = match base {
                    RelativeReferenceBase::SelfNode => Some(owner_node_id),
                    RelativeReferenceBase::ParentNode => snapshot.parent_id_of(owner_node_id),
                    RelativeReferenceBase::Ancestor(levels_up) => {
                        snapshot.nth_ancestor_of(owner_node_id, *levels_up)
                    }
                };
                base_node_id.and_then(|base_node_id| {
                    snapshot.try_resolve_descendant_path(base_node_id, path_segments)
                })
            }
            TreeReference::SiblingOffset {
                offset,
                tail_segments,
            } => {
                let sibling_node_id = snapshot.try_resolve_sibling_offset(owner_node_id, *offset);
                sibling_node_id.and_then(|sibling_node_id| {
                    if tail_segments.is_empty() {
                        Some(sibling_node_id)
                    } else {
                        snapshot.try_resolve_descendant_path(sibling_node_id, tail_segments)
                    }
                })
            }
            TreeReference::HostSensitive { .. }
            | TreeReference::CapabilitySensitive { .. }
            | TreeReference::ShapeTopology { .. }
            | TreeReference::Unresolved { .. } => None,
            TreeReference::DynamicPotential { .. } => None,
            TreeReference::DynamicResolved { target_node_id, .. } => Some(*target_node_id),
        }
    }

    #[must_use]
    pub fn descriptor_kind(&self) -> DependencyDescriptorKind {
        match self {
            TreeReference::DirectNode { .. } | TreeReference::ProjectionPath { .. } => {
                DependencyDescriptorKind::StaticDirect
            }
            TreeReference::RelativePath { .. } | TreeReference::SiblingOffset { .. } => {
                DependencyDescriptorKind::RelativeBound
            }
            TreeReference::HostSensitive { .. } => DependencyDescriptorKind::HostSensitive,
            TreeReference::CapabilitySensitive { .. } => {
                DependencyDescriptorKind::CapabilitySensitive
            }
            TreeReference::ShapeTopology { .. } => DependencyDescriptorKind::ShapeTopology,
            TreeReference::DynamicPotential { .. } | TreeReference::DynamicResolved { .. } => {
                DependencyDescriptorKind::DynamicPotential
            }
            TreeReference::Unresolved { .. } => DependencyDescriptorKind::Unresolved,
        }
    }

    #[must_use]
    pub fn requires_rebind_on_structural_change(&self) -> bool {
        matches!(
            self,
            TreeReference::RelativePath { .. }
                | TreeReference::SiblingOffset { .. }
                | TreeReference::HostSensitive { .. }
                | TreeReference::CapabilitySensitive { .. }
                | TreeReference::ShapeTopology { .. }
                | TreeReference::DynamicResolved { .. }
                | TreeReference::Unresolved { .. }
        )
    }

    #[must_use]
    pub fn carrier_detail(&self) -> String {
        match self {
            TreeReference::DirectNode { target_node_id } => {
                format!("direct_node:{target_node_id}")
            }
            TreeReference::ProjectionPath { projection_path } => {
                format!("projection_path:{projection_path}")
            }
            TreeReference::RelativePath {
                base,
                path_segments,
            } => {
                format!("relative_path:{base:?}:{}", path_segments.join("/"))
            }
            TreeReference::SiblingOffset {
                offset,
                tail_segments,
            } => format!("sibling_offset:{offset}:{}", tail_segments.join("/")),
            TreeReference::HostSensitive { carrier_id, detail } => {
                format!("host_sensitive:{carrier_id}:{detail}")
            }
            TreeReference::CapabilitySensitive { carrier_id, detail } => {
                format!("capability_sensitive:{carrier_id}:{detail}")
            }
            TreeReference::ShapeTopology { carrier_id, detail } => {
                format!("shape_topology:{carrier_id}:{detail}")
            }
            TreeReference::DynamicPotential { carrier_id, detail } => {
                format!("dynamic_potential:{carrier_id}:{detail}")
            }
            TreeReference::DynamicResolved {
                target_node_id,
                carrier_id,
                detail,
            } => {
                format!("dynamic_resolved:{target_node_id}:{carrier_id}:{detail}")
            }
            TreeReference::Unresolved { token } => format!("unresolved:{token}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::dependency::{DependencyDescriptorKind, DependencyDiagnosticKind, DependencyGraph};
    use crate::structural::{StructuralNode, StructuralNodeKind, StructuralSnapshotId};

    use super::*;

    fn fixture_formula(owner_node_id: TreeNodeId, ast: FixtureFormulaAst) -> TreeFormula {
        ast.to_tree_formula(owner_node_id)
    }

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
                    kind: StructuralNodeKind::Container,
                    symbol: "Branch".to_string(),
                    parent_id: Some(TreeNodeId(1)),
                    child_ids: vec![TreeNodeId(4), TreeNodeId(5)],
                    formula_artifact_id: None,
                    bind_artifact_id: None,
                    constant_value: None,
                },
                StructuralNode {
                    node_id: TreeNodeId(3),
                    kind: StructuralNodeKind::Calculation,
                    symbol: "Sibling".to_string(),
                    parent_id: Some(TreeNodeId(1)),
                    child_ids: vec![],
                    formula_artifact_id: None,
                    bind_artifact_id: None,
                    constant_value: None,
                },
                StructuralNode {
                    node_id: TreeNodeId(4),
                    kind: StructuralNodeKind::Calculation,
                    symbol: "Leaf".to_string(),
                    parent_id: Some(TreeNodeId(2)),
                    child_ids: vec![],
                    formula_artifact_id: None,
                    bind_artifact_id: None,
                    constant_value: None,
                },
                StructuralNode {
                    node_id: TreeNodeId(5),
                    kind: StructuralNodeKind::Calculation,
                    symbol: "Neighbor".to_string(),
                    parent_id: Some(TreeNodeId(2)),
                    child_ids: vec![],
                    formula_artifact_id: None,
                    bind_artifact_id: None,
                    constant_value: None,
                },
            ],
        )
        .unwrap()
    }

    #[test]
    fn formula_catalog_lowers_direct_and_relative_references() {
        let catalog = TreeFormulaCatalog::new([TreeFormulaBinding {
            owner_node_id: TreeNodeId(4),
            formula_artifact_id: FormulaArtifactId("formula:leaf".to_string()),
            bind_artifact_id: Some(BindArtifactId("bind:leaf".to_string())),
            expression: fixture_formula(
                TreeNodeId(4),
                FixtureFormulaAst::Binary {
                    op: FixtureFormulaBinaryOp::Add,
                    left: Box::new(FixtureFormulaAst::Reference(TreeReference::DirectNode {
                        target_node_id: TreeNodeId(3),
                    })),
                    right: Box::new(FixtureFormulaAst::Reference(TreeReference::SiblingOffset {
                        offset: 1,
                        tail_segments: vec![],
                    })),
                },
            ),
        }]);

        let descriptors = catalog.to_dependency_descriptors(&snapshot());

        assert_eq!(descriptors.len(), 2);
        assert_eq!(descriptors[0].kind, DependencyDescriptorKind::StaticDirect);
        assert_eq!(descriptors[0].target_node_id, Some(TreeNodeId(3)));
        assert_eq!(descriptors[1].kind, DependencyDescriptorKind::RelativeBound);
        assert_eq!(descriptors[1].target_node_id, Some(TreeNodeId(5)));
    }

    #[test]
    fn formula_catalog_lowers_parent_and_ancestor_relative_references() {
        let catalog = TreeFormulaCatalog::new([TreeFormulaBinding {
            owner_node_id: TreeNodeId(4),
            formula_artifact_id: FormulaArtifactId("formula:leaf".to_string()),
            bind_artifact_id: Some(BindArtifactId("bind:leaf".to_string())),
            expression: fixture_formula(
                TreeNodeId(4),
                FixtureFormulaAst::Binary {
                    op: FixtureFormulaBinaryOp::Add,
                    left: Box::new(FixtureFormulaAst::Reference(TreeReference::RelativePath {
                        base: RelativeReferenceBase::ParentNode,
                        path_segments: vec!["Neighbor".to_string()],
                    })),
                    right: Box::new(FixtureFormulaAst::Reference(TreeReference::RelativePath {
                        base: RelativeReferenceBase::Ancestor(2),
                        path_segments: vec!["Sibling".to_string()],
                    })),
                },
            ),
        }]);

        let descriptors = catalog.to_dependency_descriptors(&snapshot());

        assert_eq!(descriptors.len(), 2);
        assert_eq!(descriptors[0].kind, DependencyDescriptorKind::RelativeBound);
        assert_eq!(descriptors[0].target_node_id, Some(TreeNodeId(5)));
        assert_eq!(descriptors[1].kind, DependencyDescriptorKind::RelativeBound);
        assert_eq!(descriptors[1].target_node_id, Some(TreeNodeId(3)));
    }

    #[test]
    fn formula_catalog_surfaces_host_sensitive_and_unresolved_references() {
        let snapshot = snapshot();
        let catalog = TreeFormulaCatalog::new([TreeFormulaBinding {
            owner_node_id: TreeNodeId(4),
            formula_artifact_id: FormulaArtifactId("formula:leaf".to_string()),
            bind_artifact_id: None,
            expression: fixture_formula(
                TreeNodeId(4),
                FixtureFormulaAst::FunctionCall {
                    function_name: "CHOOSE".to_string(),
                    arguments: vec![
                        FixtureFormulaAst::Reference(TreeReference::HostSensitive {
                            carrier_id: "host.selection".to_string(),
                            detail: "active branch".to_string(),
                        }),
                        FixtureFormulaAst::Reference(TreeReference::Unresolved {
                            token: "../Missing".to_string(),
                        }),
                    ],
                    may_introduce_dynamic_dependencies: true,
                },
            ),
        }]);

        let graph =
            DependencyGraph::build(&snapshot, &catalog.to_dependency_descriptors(&snapshot));

        assert_eq!(graph.diagnostics.len(), 2);
        assert_eq!(
            graph.diagnostics[0].kind,
            DependencyDiagnosticKind::HostSensitiveReference
        );
        assert_eq!(
            graph.diagnostics[1].kind,
            DependencyDiagnosticKind::UnresolvedReference
        );
    }

    #[test]
    fn formula_catalog_lowers_resolved_dynamic_reference_as_dynamic_edge() {
        let snapshot = snapshot();
        let catalog = TreeFormulaCatalog::new([TreeFormulaBinding {
            owner_node_id: TreeNodeId(4),
            formula_artifact_id: FormulaArtifactId("formula:dynamic-resolved".to_string()),
            bind_artifact_id: Some(BindArtifactId("bind:dynamic-resolved".to_string())),
            expression: fixture_formula(
                TreeNodeId(4),
                FixtureFormulaAst::Reference(TreeReference::DynamicResolved {
                    target_node_id: TreeNodeId(3),
                    carrier_id: "carrier:dynamic".to_string(),
                    detail: "resolved_late_bound_projection".to_string(),
                }),
            ),
        }]);

        let descriptors = catalog.to_dependency_descriptors(&snapshot);

        assert_eq!(descriptors.len(), 1);
        assert_eq!(
            descriptors[0].kind,
            DependencyDescriptorKind::DynamicPotential
        );
        assert_eq!(descriptors[0].target_node_id, Some(TreeNodeId(3)));
        assert!(descriptors[0].requires_rebind_on_structural_change);
        assert_eq!(
            descriptors[0].carrier_detail,
            "dynamic_resolved:node:3:carrier:dynamic:resolved_late_bound_projection"
        );
    }

    #[test]
    fn runtime_fact_carriers_are_not_formula_reference_carriers() {
        let cases = [
            (
                TreeReference::HostSensitive {
                    carrier_id: "host.selection".to_string(),
                    detail: "active branch".to_string(),
                },
                DependencyDescriptorKind::HostSensitive,
                "host_sensitive:host.selection:active branch",
                true,
            ),
            (
                TreeReference::DynamicPotential {
                    carrier_id: "runtime.topic".to_string(),
                    detail: "late bound".to_string(),
                },
                DependencyDescriptorKind::DynamicPotential,
                "dynamic_potential:runtime.topic:late bound",
                false,
            ),
            (
                TreeReference::CapabilitySensitive {
                    carrier_id: "capability.profile".to_string(),
                    detail: "requires provider".to_string(),
                },
                DependencyDescriptorKind::CapabilitySensitive,
                "capability_sensitive:capability.profile:requires provider",
                true,
            ),
            (
                TreeReference::ShapeTopology {
                    carrier_id: "shape.range".to_string(),
                    detail: "range extent".to_string(),
                },
                DependencyDescriptorKind::ShapeTopology,
                "shape_topology:shape.range:range extent",
                true,
            ),
        ];

        for (reference, expected_kind, expected_detail, expected_rebind) in cases {
            assert_eq!(
                reference.carrier_class(),
                TreeReferenceCarrierClass::RuntimeFactProjection
            );
            assert_eq!(reference.resolve_target(&snapshot(), TreeNodeId(4)), None);
            assert_eq!(reference.descriptor_kind(), expected_kind);
            assert_eq!(reference.carrier_detail(), expected_detail);
            assert_eq!(
                reference.requires_rebind_on_structural_change(),
                expected_rebind
            );

            let carrier = TreeFormulaReferenceCarrier::fact(reference);
            assert_eq!(carrier.source_token, None);
        }
    }

    #[test]
    fn runtime_fact_carriers_surface_diagnostics_without_dependency_edges() {
        let snapshot = snapshot();
        let catalog = TreeFormulaCatalog::new([TreeFormulaBinding {
            owner_node_id: TreeNodeId(4),
            formula_artifact_id: FormulaArtifactId("formula:runtime-facts".to_string()),
            bind_artifact_id: Some(BindArtifactId("bind:runtime-facts".to_string())),
            expression: TreeFormula::opaque_oxfml(
                "=SUM(1,1)",
                [
                    TreeReference::HostSensitive {
                        carrier_id: "host.selection".to_string(),
                        detail: "active branch".to_string(),
                    },
                    TreeReference::DynamicPotential {
                        carrier_id: "runtime.topic".to_string(),
                        detail: "late bound".to_string(),
                    },
                    TreeReference::CapabilitySensitive {
                        carrier_id: "capability.profile".to_string(),
                        detail: "requires provider".to_string(),
                    },
                    TreeReference::ShapeTopology {
                        carrier_id: "shape.range".to_string(),
                        detail: "range extent".to_string(),
                    },
                ]
                .map(TreeFormulaReferenceCarrier::fact),
            ),
        }]);

        let descriptors = catalog.to_dependency_descriptors(&snapshot);
        assert_eq!(descriptors.len(), 4);
        assert!(
            descriptors
                .iter()
                .all(|descriptor| descriptor.target_node_id.is_none())
        );
        assert!(
            descriptors
                .iter()
                .all(|descriptor| descriptor.source_reference_handle.is_none())
        );

        let graph = DependencyGraph::build(&snapshot, &descriptors);
        assert!(graph.edges_by_owner.is_empty());
        assert_eq!(
            graph
                .diagnostics
                .iter()
                .map(|diagnostic| diagnostic.kind)
                .collect::<Vec<_>>(),
            vec![
                DependencyDiagnosticKind::HostSensitiveReference,
                DependencyDiagnosticKind::DynamicPotentialReference,
                DependencyDiagnosticKind::CapabilitySensitiveReference,
                DependencyDiagnosticKind::ShapeTopologyReference,
            ]
        );
    }

    #[test]
    fn first_closed_subset_rebind_flags_match_w026_floor() {
        assert!(
            !TreeReference::DirectNode {
                target_node_id: TreeNodeId(2)
            }
            .requires_rebind_on_structural_change()
        );
        assert!(
            TreeReference::RelativePath {
                base: RelativeReferenceBase::ParentNode,
                path_segments: vec!["A".to_string()],
            }
            .requires_rebind_on_structural_change()
        );
        assert!(
            TreeReference::RelativePath {
                base: RelativeReferenceBase::Ancestor(2),
                path_segments: vec!["A".to_string(), "Leaf".to_string()],
            }
            .requires_rebind_on_structural_change()
        );
        assert!(
            TreeReference::SiblingOffset {
                offset: 1,
                tail_segments: vec![],
            }
            .requires_rebind_on_structural_change()
        );
        assert!(
            TreeReference::HostSensitive {
                carrier_id: "host.selection".to_string(),
                detail: "active branch".to_string(),
            }
            .requires_rebind_on_structural_change()
        );
        assert!(
            !TreeReference::DynamicPotential {
                carrier_id: "runtime.topic".to_string(),
                detail: "late bound".to_string(),
            }
            .requires_rebind_on_structural_change()
        );
        assert!(
            TreeReference::DynamicResolved {
                target_node_id: TreeNodeId(2),
                carrier_id: "runtime.topic".to_string(),
                detail: "late bound".to_string(),
            }
            .requires_rebind_on_structural_change()
        );
        assert!(
            TreeReference::Unresolved {
                token: "../Missing".to_string(),
            }
            .requires_rebind_on_structural_change()
        );
    }

    #[test]
    fn raw_oxfml_formula_lowers_declared_reference_carriers() {
        let snapshot = snapshot();
        let catalog = TreeFormulaCatalog::new([TreeFormulaBinding {
            owner_node_id: TreeNodeId(4),
            formula_artifact_id: FormulaArtifactId("formula:raw-let-lambda".to_string()),
            bind_artifact_id: Some(BindArtifactId("bind:raw-let-lambda".to_string())),
            expression: fixture_formula(
                TreeNodeId(4),
                FixtureFormulaAst::RawOxfml {
                    source_text: "LET(base,TREE_REF_4_0,LAMBDA(delta,base+delta)(5))".to_string(),
                    reference_carriers: vec![
                        TreeReference::DirectNode {
                            target_node_id: TreeNodeId(3),
                        },
                        TreeReference::HostSensitive {
                            carrier_id: "carrier:lambda.host".to_string(),
                            detail: "call_argument_host_query".to_string(),
                        },
                    ],
                },
            ),
        }]);

        let descriptors = catalog.to_dependency_descriptors(&snapshot);

        assert_eq!(descriptors.len(), 2);
        assert_eq!(descriptors[0].kind, DependencyDescriptorKind::StaticDirect);
        assert_eq!(
            descriptors[0].carrier_detail,
            "direct_node:node:3".to_string()
        );
        assert_eq!(descriptors[1].kind, DependencyDescriptorKind::HostSensitive);
        assert_eq!(
            descriptors[1].carrier_detail,
            "host_sensitive:carrier:lambda.host:call_argument_host_query".to_string()
        );
    }
}
