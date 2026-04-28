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
    Unresolved {
        token: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum FormulaBinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TreeFormula {
    Literal {
        value: String,
    },
    Reference(TreeReference),
    Binary {
        op: FormulaBinaryOp,
        left: Box<TreeFormula>,
        right: Box<TreeFormula>,
    },
    FunctionCall {
        function_name: String,
        arguments: Vec<TreeFormula>,
        may_introduce_dynamic_dependencies: bool,
    },
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
            let mut next_index = 0usize;
            collect_descriptors(
                snapshot,
                binding,
                &binding.expression,
                &mut next_index,
                &mut descriptors,
            );
        }

        descriptors.sort_by(|left, right| left.descriptor_id.cmp(&right.descriptor_id));
        descriptors
    }
}

fn collect_descriptors(
    snapshot: &StructuralSnapshot,
    binding: &TreeFormulaBinding,
    expression: &TreeFormula,
    next_index: &mut usize,
    descriptors: &mut Vec<DependencyDescriptor>,
) {
    match expression {
        TreeFormula::Literal { .. } => {}
        TreeFormula::Reference(reference) => {
            descriptors.push(lower_reference(snapshot, binding, reference, *next_index));
            *next_index += 1;
        }
        TreeFormula::Binary { left, right, .. } => {
            collect_descriptors(snapshot, binding, left, next_index, descriptors);
            collect_descriptors(snapshot, binding, right, next_index, descriptors);
        }
        TreeFormula::FunctionCall { arguments, .. } => {
            for argument in arguments {
                collect_descriptors(snapshot, binding, argument, next_index, descriptors);
            }
        }
    }
}

fn lower_reference(
    snapshot: &StructuralSnapshot,
    binding: &TreeFormulaBinding,
    reference: &TreeReference,
    index: usize,
) -> DependencyDescriptor {
    let descriptor_id = format!("bind:{}:ref:{index}", binding.formula_artifact_id.0);
    let kind = reference.descriptor_kind();
    let target_node_id = reference.resolve_target(snapshot, binding.owner_node_id);
    let carrier_detail = reference.carrier_detail();
    let requires_rebind_on_structural_change = reference.requires_rebind_on_structural_change();

    DependencyDescriptor {
        descriptor_id,
        owner_node_id: binding.owner_node_id,
        target_node_id,
        kind,
        carrier_detail,
        requires_rebind_on_structural_change,
    }
}

impl TreeReference {
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
            | TreeReference::DynamicPotential { .. }
            | TreeReference::Unresolved { .. } => None,
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
            TreeReference::DynamicPotential { .. } => DependencyDescriptorKind::DynamicPotential,
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
            TreeReference::Unresolved { token } => format!("unresolved:{token}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::dependency::{DependencyDescriptorKind, DependencyDiagnosticKind, DependencyGraph};
    use crate::structural::{StructuralNode, StructuralNodeKind, StructuralSnapshotId};

    use super::*;

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
            expression: TreeFormula::Binary {
                op: FormulaBinaryOp::Add,
                left: Box::new(TreeFormula::Reference(TreeReference::DirectNode {
                    target_node_id: TreeNodeId(3),
                })),
                right: Box::new(TreeFormula::Reference(TreeReference::SiblingOffset {
                    offset: 1,
                    tail_segments: vec![],
                })),
            },
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
            expression: TreeFormula::Binary {
                op: FormulaBinaryOp::Add,
                left: Box::new(TreeFormula::Reference(TreeReference::RelativePath {
                    base: RelativeReferenceBase::ParentNode,
                    path_segments: vec!["Neighbor".to_string()],
                })),
                right: Box::new(TreeFormula::Reference(TreeReference::RelativePath {
                    base: RelativeReferenceBase::Ancestor(2),
                    path_segments: vec!["Sibling".to_string()],
                })),
            },
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
            expression: TreeFormula::FunctionCall {
                function_name: "CHOOSE".to_string(),
                arguments: vec![
                    TreeFormula::Reference(TreeReference::HostSensitive {
                        carrier_id: "host.selection".to_string(),
                        detail: "active branch".to_string(),
                    }),
                    TreeFormula::Reference(TreeReference::Unresolved {
                        token: "../Missing".to_string(),
                    }),
                ],
                may_introduce_dynamic_dependencies: true,
            },
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
            TreeReference::Unresolved {
                token: "../Missing".to_string(),
            }
            .requires_rebind_on_structural_change()
        );
    }
}
