#![forbid(unsafe_code)]

//! W056 TreeCalc reference dependency, invalidation, and rebind facts.
//!
//! This module is an OxCalc-owned typed projection over the existing dependency
//! graph. It does not parse formula text and does not consume OxFml private
//! strings.

use std::collections::{BTreeMap, BTreeSet};

use crate::dependency::{
    DependencyDescriptor, DependencyDescriptorKind, DependencyGraph, InvalidationReasonKind,
    WorkspaceQualifiedTarget,
};
use crate::formula::{
    CallerContextIdentityNeed, NamespaceIdentityNeed, TreeReferenceImplementationInput,
    tree_reference_implementation_inputs,
};
use crate::structural::TreeNodeId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ReferenceDescriptorFactRole {
    TargetReverseEdge,
    ContextReverseEdge,
    RuntimeFact,
    BlockedReference,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DynamicRebindState {
    Potential,
    Resolved,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceDependencyDescriptorFact {
    pub descriptor_id: String,
    pub owner_node_id: TreeNodeId,
    pub source_reference_handle: Option<String>,
    pub target_node_id: Option<TreeNodeId>,
    pub workspace_target: Option<WorkspaceQualifiedTarget>,
    pub kind: DependencyDescriptorKind,
    pub role: ReferenceDescriptorFactRole,
    pub namespace_identity_need: NamespaceIdentityNeed,
    pub caller_context_identity_need: CallerContextIdentityNeed,
    pub invalidation_facts: Vec<InvalidationReasonKind>,
    pub requires_rebind_on_structural_change: bool,
    pub prepared_identity_invalidates: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceTargetReverseEdgeFact {
    pub edge_id: String,
    pub descriptor_id: String,
    pub owner_node_id: TreeNodeId,
    pub target_node_id: TreeNodeId,
    pub kind: DependencyDescriptorKind,
    pub source_reference_handle: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceWorkspaceTargetReverseEdgeFact {
    pub edge_id: String,
    pub descriptor_id: String,
    pub owner_node_id: TreeNodeId,
    pub target: WorkspaceQualifiedTarget,
    pub kind: DependencyDescriptorKind,
    pub source_reference_handle: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceContextReverseEdgeFact {
    pub context_edge_id: String,
    pub descriptor_id: String,
    pub owner_node_id: TreeNodeId,
    pub kind: DependencyDescriptorKind,
    pub source_reference_handle: Option<String>,
    pub context_identity: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedIdentityInvalidationFact {
    pub owner_node_id: TreeNodeId,
    pub descriptor_id: String,
    pub namespace_identity_need: NamespaceIdentityNeed,
    pub caller_context_identity_need: CallerContextIdentityNeed,
    pub invalidation_facts: Vec<InvalidationReasonKind>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DynamicRebindFact {
    pub descriptor_id: String,
    pub owner_node_id: TreeNodeId,
    pub state: DynamicRebindState,
    pub target_node_id: Option<TreeNodeId>,
    pub invalidation_facts: Vec<InvalidationReasonKind>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct W056ReferenceDependencySurface {
    pub descriptor_facts: Vec<ReferenceDependencyDescriptorFact>,
    pub target_reverse_edges: Vec<ReferenceTargetReverseEdgeFact>,
    pub workspace_target_reverse_edges: Vec<ReferenceWorkspaceTargetReverseEdgeFact>,
    pub context_reverse_edges: Vec<ReferenceContextReverseEdgeFact>,
    pub prepared_identity_facts: Vec<PreparedIdentityInvalidationFact>,
    pub dynamic_rebind_facts: Vec<DynamicRebindFact>,
    pub admitted_inventory_inputs: Vec<TreeReferenceImplementationInput>,
    pub blocked_inventory_inputs: Vec<TreeReferenceImplementationInput>,
}

#[must_use]
pub fn w056_reference_dependency_surface(
    graph: &DependencyGraph,
) -> W056ReferenceDependencySurface {
    let descriptors = graph
        .descriptors_by_owner
        .values()
        .flatten()
        .cloned()
        .collect::<Vec<_>>();

    let mut descriptor_facts = descriptors
        .iter()
        .map(reference_dependency_descriptor_fact)
        .collect::<Vec<_>>();
    descriptor_facts.sort_by(|left, right| {
        left.owner_node_id
            .cmp(&right.owner_node_id)
            .then_with(|| left.descriptor_id.cmp(&right.descriptor_id))
    });

    let descriptor_by_id = descriptors
        .iter()
        .map(|descriptor| {
            (
                (descriptor.owner_node_id, descriptor.descriptor_id.as_str()),
                descriptor,
            )
        })
        .collect::<BTreeMap<_, _>>();
    let mut target_reverse_edges = graph
        .reverse_edges
        .values()
        .flatten()
        .filter_map(|edge| {
            let descriptor =
                descriptor_by_id.get(&(edge.owner_node_id, edge.descriptor_id.as_str()))?;
            Some(ReferenceTargetReverseEdgeFact {
                edge_id: edge.edge_id.clone(),
                descriptor_id: edge.descriptor_id.clone(),
                owner_node_id: edge.owner_node_id,
                target_node_id: edge.target_node_id,
                kind: edge.kind,
                source_reference_handle: descriptor.source_reference_handle.clone(),
            })
        })
        .collect::<Vec<_>>();
    target_reverse_edges.sort_by(|left, right| left.edge_id.cmp(&right.edge_id));

    let mut workspace_target_reverse_edges = graph
        .workspace_reverse_edges
        .values()
        .flatten()
        .filter_map(|edge| {
            let descriptor =
                descriptor_by_id.get(&(edge.owner_node_id, edge.descriptor_id.as_str()))?;
            Some(ReferenceWorkspaceTargetReverseEdgeFact {
                edge_id: edge.edge_id.clone(),
                descriptor_id: edge.descriptor_id.clone(),
                owner_node_id: edge.owner_node_id,
                target: edge.target.clone(),
                kind: edge.kind,
                source_reference_handle: descriptor.source_reference_handle.clone(),
            })
        })
        .collect::<Vec<_>>();
    workspace_target_reverse_edges.sort_by(|left, right| left.edge_id.cmp(&right.edge_id));

    let mut context_reverse_edges = descriptors
        .iter()
        .filter(|descriptor| {
            descriptor.target_node_id.is_none() && descriptor.workspace_target.is_none()
        })
        .map(|descriptor| ReferenceContextReverseEdgeFact {
            context_edge_id: format!(
                "context:{}:{}",
                descriptor.owner_node_id.0, descriptor.descriptor_id
            ),
            descriptor_id: descriptor.descriptor_id.clone(),
            owner_node_id: descriptor.owner_node_id,
            kind: descriptor.kind,
            source_reference_handle: descriptor.source_reference_handle.clone(),
            context_identity: descriptor.carrier_detail.clone(),
        })
        .collect::<Vec<_>>();
    context_reverse_edges.sort_by(|left, right| left.context_edge_id.cmp(&right.context_edge_id));

    let mut prepared_identity_facts = descriptor_facts
        .iter()
        .filter(|fact| fact.prepared_identity_invalidates)
        .map(|fact| PreparedIdentityInvalidationFact {
            owner_node_id: fact.owner_node_id,
            descriptor_id: fact.descriptor_id.clone(),
            namespace_identity_need: fact.namespace_identity_need,
            caller_context_identity_need: fact.caller_context_identity_need,
            invalidation_facts: fact.invalidation_facts.clone(),
        })
        .collect::<Vec<_>>();
    prepared_identity_facts.sort_by(|left, right| {
        left.owner_node_id
            .cmp(&right.owner_node_id)
            .then_with(|| left.descriptor_id.cmp(&right.descriptor_id))
    });

    let mut dynamic_rebind_facts = descriptors
        .iter()
        .filter(|descriptor| descriptor.kind == DependencyDescriptorKind::DynamicPotential)
        .map(|descriptor| DynamicRebindFact {
            descriptor_id: descriptor.descriptor_id.clone(),
            owner_node_id: descriptor.owner_node_id,
            state: if descriptor.target_node_id.is_some() {
                DynamicRebindState::Resolved
            } else {
                DynamicRebindState::Potential
            },
            target_node_id: descriptor.target_node_id,
            invalidation_facts: descriptor_invalidation_facts(descriptor),
        })
        .collect::<Vec<_>>();
    dynamic_rebind_facts.sort_by(|left, right| {
        left.owner_node_id
            .cmp(&right.owner_node_id)
            .then_with(|| left.descriptor_id.cmp(&right.descriptor_id))
    });

    let (admitted_inventory_inputs, blocked_inventory_inputs): (Vec<_>, Vec<_>) =
        tree_reference_implementation_inputs()
            .into_iter()
            .partition(TreeReferenceImplementationInput::is_admitted);

    W056ReferenceDependencySurface {
        descriptor_facts,
        target_reverse_edges,
        workspace_target_reverse_edges,
        context_reverse_edges,
        prepared_identity_facts,
        dynamic_rebind_facts,
        admitted_inventory_inputs,
        blocked_inventory_inputs,
    }
}

#[must_use]
pub fn reference_dependency_descriptor_fact(
    descriptor: &DependencyDescriptor,
) -> ReferenceDependencyDescriptorFact {
    let (namespace_identity_need, caller_context_identity_need) =
        if descriptor.workspace_target.is_some() {
            (
                NamespaceIdentityNeed::CrossWorkspaceAvailabilityVersion,
                CallerContextIdentityNeed::None,
            )
        } else {
            descriptor_identity_needs(descriptor.kind)
        };
    let invalidation_facts = descriptor_invalidation_facts(descriptor);
    let prepared_identity_invalidates = descriptor.requires_rebind_on_structural_change
        || namespace_identity_need != NamespaceIdentityNeed::None
        || caller_context_identity_need != CallerContextIdentityNeed::None
        || invalidation_facts
            .iter()
            .any(reason_requires_prepared_invalidation);

    ReferenceDependencyDescriptorFact {
        descriptor_id: descriptor.descriptor_id.clone(),
        owner_node_id: descriptor.owner_node_id,
        source_reference_handle: descriptor.source_reference_handle.clone(),
        target_node_id: descriptor.target_node_id,
        workspace_target: descriptor.workspace_target.clone(),
        kind: descriptor.kind,
        role: descriptor_role(descriptor),
        namespace_identity_need,
        caller_context_identity_need,
        invalidation_facts,
        requires_rebind_on_structural_change: descriptor.requires_rebind_on_structural_change,
        prepared_identity_invalidates,
    }
}

#[must_use]
pub fn descriptor_identity_needs(
    kind: DependencyDescriptorKind,
) -> (NamespaceIdentityNeed, CallerContextIdentityNeed) {
    match kind {
        DependencyDescriptorKind::StaticDirect => (
            NamespaceIdentityNeed::StructureContextVersion,
            CallerContextIdentityNeed::None,
        ),
        DependencyDescriptorKind::RelativeBound => (
            NamespaceIdentityNeed::HostNamespaceVersion,
            CallerContextIdentityNeed::CallerNode,
        ),
        DependencyDescriptorKind::TreeReferenceCollectionMembership
        | DependencyDescriptorKind::TreeReferenceCollectionMemberValue => (
            NamespaceIdentityNeed::HostNamespaceVersion,
            CallerContextIdentityNeed::CallerNode,
        ),
        DependencyDescriptorKind::StructuredTableIdentity
        | DependencyDescriptorKind::StructuredTableRowMembership
        | DependencyDescriptorKind::StructuredTableRowOrder
        | DependencyDescriptorKind::StructuredTableColumnIdentity
        | DependencyDescriptorKind::StructuredTableHeaderText
        | DependencyDescriptorKind::StructuredTableHeaderRegion
        | DependencyDescriptorKind::StructuredTableDataRegion
        | DependencyDescriptorKind::StructuredTableTotalsRegion
        | DependencyDescriptorKind::StructuredTableEnclosingTable => (
            NamespaceIdentityNeed::TableContextIdentity,
            CallerContextIdentityNeed::None,
        ),
        DependencyDescriptorKind::StructuredTableCallerContext => (
            NamespaceIdentityNeed::TableContextIdentity,
            CallerContextIdentityNeed::TableCallerRegion,
        ),
        DependencyDescriptorKind::DynamicPotential => (
            NamespaceIdentityNeed::StructureContextVersion,
            CallerContextIdentityNeed::HostRuntimeContext,
        ),
        DependencyDescriptorKind::HostSensitive => (
            NamespaceIdentityNeed::HostNamespaceVersion,
            CallerContextIdentityNeed::HostRuntimeContext,
        ),
        DependencyDescriptorKind::CapabilitySensitive => (
            NamespaceIdentityNeed::CapabilityProfileVersion,
            CallerContextIdentityNeed::HostRuntimeContext,
        ),
        DependencyDescriptorKind::ShapeTopology => (
            NamespaceIdentityNeed::StructureContextVersion,
            CallerContextIdentityNeed::HostRuntimeContext,
        ),
        DependencyDescriptorKind::Unresolved => (
            NamespaceIdentityNeed::HostNamespaceVersion,
            CallerContextIdentityNeed::CallerNode,
        ),
    }
}

#[must_use]
pub fn descriptor_invalidation_facts(
    descriptor: &DependencyDescriptor,
) -> Vec<InvalidationReasonKind> {
    let mut facts = match descriptor.kind {
        DependencyDescriptorKind::StaticDirect => vec![
            InvalidationReasonKind::StructuralRecalcOnly,
            InvalidationReasonKind::UpstreamPublication,
        ],
        DependencyDescriptorKind::RelativeBound => vec![
            InvalidationReasonKind::StructuralRebindRequired,
            InvalidationReasonKind::UpstreamPublication,
        ],
        DependencyDescriptorKind::TreeReferenceCollectionMembership => vec![
            InvalidationReasonKind::TreeReferenceMembershipChanged,
            InvalidationReasonKind::TreeReferenceOrderChanged,
        ],
        DependencyDescriptorKind::TreeReferenceCollectionMemberValue => {
            vec![InvalidationReasonKind::UpstreamPublication]
        }
        DependencyDescriptorKind::StructuredTableIdentity => {
            vec![InvalidationReasonKind::StructuredTableContextChanged]
        }
        DependencyDescriptorKind::StructuredTableRowMembership => {
            vec![InvalidationReasonKind::StructuredTableRowMembershipChanged]
        }
        DependencyDescriptorKind::StructuredTableRowOrder => {
            vec![InvalidationReasonKind::StructuredTableRowOrderChanged]
        }
        DependencyDescriptorKind::StructuredTableColumnIdentity
        | DependencyDescriptorKind::StructuredTableHeaderText => {
            vec![InvalidationReasonKind::StructuredTableColumnChanged]
        }
        DependencyDescriptorKind::StructuredTableHeaderRegion
        | DependencyDescriptorKind::StructuredTableDataRegion
        | DependencyDescriptorKind::StructuredTableTotalsRegion => {
            vec![InvalidationReasonKind::StructuredTableRegionChanged]
        }
        DependencyDescriptorKind::StructuredTableCallerContext
        | DependencyDescriptorKind::StructuredTableEnclosingTable => {
            vec![InvalidationReasonKind::StructuredTableCallerContextChanged]
        }
        DependencyDescriptorKind::DynamicPotential if descriptor.target_node_id.is_some() => vec![
            InvalidationReasonKind::DynamicDependencyActivated,
            InvalidationReasonKind::DynamicDependencyReleased,
            InvalidationReasonKind::DynamicDependencyReclassified,
            InvalidationReasonKind::UpstreamPublication,
        ],
        DependencyDescriptorKind::DynamicPotential => vec![
            InvalidationReasonKind::DynamicDependencyActivated,
            InvalidationReasonKind::DynamicDependencyReclassified,
        ],
        DependencyDescriptorKind::HostSensitive => vec![
            InvalidationReasonKind::StructuralRebindRequired,
            InvalidationReasonKind::ExternallyInvalidated,
        ],
        DependencyDescriptorKind::CapabilitySensitive => vec![
            InvalidationReasonKind::DependencyReclassified,
            InvalidationReasonKind::ExternallyInvalidated,
        ],
        DependencyDescriptorKind::ShapeTopology => vec![
            InvalidationReasonKind::DependencyReclassified,
            InvalidationReasonKind::StructuralRebindRequired,
        ],
        DependencyDescriptorKind::Unresolved => {
            vec![InvalidationReasonKind::StructuralRebindRequired]
        }
    };
    facts.sort();
    facts.dedup();
    facts
}

fn descriptor_role(descriptor: &DependencyDescriptor) -> ReferenceDescriptorFactRole {
    if descriptor.target_node_id.is_some() || descriptor.workspace_target.is_some() {
        return ReferenceDescriptorFactRole::TargetReverseEdge;
    }

    match descriptor.kind {
        DependencyDescriptorKind::DynamicPotential
        | DependencyDescriptorKind::HostSensitive
        | DependencyDescriptorKind::CapabilitySensitive
        | DependencyDescriptorKind::ShapeTopology => ReferenceDescriptorFactRole::RuntimeFact,
        DependencyDescriptorKind::Unresolved => ReferenceDescriptorFactRole::BlockedReference,
        _ => ReferenceDescriptorFactRole::ContextReverseEdge,
    }
}

fn reason_requires_prepared_invalidation(reason: &InvalidationReasonKind) -> bool {
    matches!(
        reason,
        InvalidationReasonKind::StructuralRebindRequired
            | InvalidationReasonKind::TreeReferenceMembershipChanged
            | InvalidationReasonKind::TreeReferenceOrderChanged
            | InvalidationReasonKind::StructuredTableContextChanged
            | InvalidationReasonKind::StructuredTableRowMembershipChanged
            | InvalidationReasonKind::StructuredTableRowOrderChanged
            | InvalidationReasonKind::StructuredTableColumnChanged
            | InvalidationReasonKind::StructuredTableRegionChanged
            | InvalidationReasonKind::StructuredTableCallerContextChanged
            | InvalidationReasonKind::DependencyAdded
            | InvalidationReasonKind::DependencyRemoved
            | InvalidationReasonKind::DependencyReclassified
            | InvalidationReasonKind::DynamicDependencyActivated
            | InvalidationReasonKind::DynamicDependencyReleased
            | InvalidationReasonKind::DynamicDependencyReclassified
    )
}

#[must_use]
pub fn prepared_identity_needs_by_owner(
    surface: &W056ReferenceDependencySurface,
) -> BTreeMap<TreeNodeId, BTreeSet<(NamespaceIdentityNeed, CallerContextIdentityNeed)>> {
    surface
        .prepared_identity_facts
        .iter()
        .fold(BTreeMap::new(), |mut by_owner, fact| {
            by_owner.entry(fact.owner_node_id).or_default().insert((
                fact.namespace_identity_need,
                fact.caller_context_identity_need,
            ));
            by_owner
        })
}

#[cfg(test)]
mod tests {
    use crate::dependency::{DependencyDescriptor, DependencyGraph};
    use crate::formula::TreeReferenceInventoryVariant;
    use crate::structural::{
        StructuralNode, StructuralNodeKind, StructuralSnapshot, StructuralSnapshotId,
    };

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
                    child_ids: vec![TreeNodeId(2), TreeNodeId(3), TreeNodeId(4)],
                },
                StructuralNode {
                    node_id: TreeNodeId(2),
                    kind: StructuralNodeKind::Calculation,
                    symbol: "A".to_string(),
                    parent_id: Some(TreeNodeId(1)),
                    child_ids: Vec::new(),
                },
                StructuralNode {
                    node_id: TreeNodeId(3),
                    kind: StructuralNodeKind::Calculation,
                    symbol: "B".to_string(),
                    parent_id: Some(TreeNodeId(1)),
                    child_ids: Vec::new(),
                },
                StructuralNode {
                    node_id: TreeNodeId(4),
                    kind: StructuralNodeKind::Calculation,
                    symbol: "C".to_string(),
                    parent_id: Some(TreeNodeId(1)),
                    child_ids: Vec::new(),
                },
            ],
        )
        .unwrap()
    }

    fn descriptor(
        descriptor_id: &str,
        owner_node_id: TreeNodeId,
        target_node_id: Option<TreeNodeId>,
        kind: DependencyDescriptorKind,
        carrier_detail: &str,
        source_reference_handle: Option<&str>,
        requires_rebind_on_structural_change: bool,
    ) -> DependencyDescriptor {
        DependencyDescriptor {
            descriptor_id: descriptor_id.to_string(),
            source_reference_handle: source_reference_handle.map(str::to_string),
            owner_node_id,
            target_node_id,
            workspace_target: None,
            kind,
            carrier_detail: carrier_detail.to_string(),
            tree_reference_collection: None,
            requires_rebind_on_structural_change,
        }
    }

    #[test]
    fn surface_records_target_and_context_reverse_edges() {
        let graph = DependencyGraph::build(
            &snapshot(),
            &[
                descriptor(
                    "dep:relative",
                    TreeNodeId(4),
                    Some(TreeNodeId(2)),
                    DependencyDescriptorKind::RelativeBound,
                    "relative_path:ParentNode:A",
                    Some("treecalc_reference_carrier:TREE_REF_4_0"),
                    true,
                ),
                descriptor(
                    "dep:table-context",
                    TreeNodeId(4),
                    None,
                    DependencyDescriptorKind::StructuredTableCallerContext,
                    "table_caller_context:v1:table=Sales;row_offset=2",
                    Some("oxfml-structured-ref:1"),
                    true,
                ),
            ],
        );

        let surface = w056_reference_dependency_surface(&graph);

        assert_eq!(surface.target_reverse_edges.len(), 1);
        assert_eq!(
            surface.target_reverse_edges[0].target_node_id,
            TreeNodeId(2)
        );
        assert_eq!(
            surface.target_reverse_edges[0]
                .source_reference_handle
                .as_deref(),
            Some("treecalc_reference_carrier:TREE_REF_4_0")
        );
        assert_eq!(surface.context_reverse_edges.len(), 1);
        assert_eq!(
            surface.context_reverse_edges[0].kind,
            DependencyDescriptorKind::StructuredTableCallerContext
        );
        assert_eq!(
            surface.context_reverse_edges[0].context_identity,
            "table_caller_context:v1:table=Sales;row_offset=2"
        );
    }

    #[test]
    fn surface_records_prepared_identity_inputs_for_namespace_and_caller_context() {
        let graph = DependencyGraph::build(
            &snapshot(),
            &[
                descriptor(
                    "dep:host-sensitive",
                    TreeNodeId(3),
                    None,
                    DependencyDescriptorKind::HostSensitive,
                    "host_sensitive:selection:active",
                    Some("runtime_fact:HostSensitive:selection"),
                    true,
                ),
                descriptor(
                    "dep:capability",
                    TreeNodeId(3),
                    None,
                    DependencyDescriptorKind::CapabilitySensitive,
                    "capability_sensitive:profile:provider",
                    Some("runtime_fact:CapabilitySensitive:profile"),
                    true,
                ),
            ],
        );

        let surface = w056_reference_dependency_surface(&graph);
        let by_owner = prepared_identity_needs_by_owner(&surface);

        assert!(by_owner[&TreeNodeId(3)].contains(&(
            NamespaceIdentityNeed::HostNamespaceVersion,
            CallerContextIdentityNeed::HostRuntimeContext
        )));
        assert!(by_owner[&TreeNodeId(3)].contains(&(
            NamespaceIdentityNeed::CapabilityProfileVersion,
            CallerContextIdentityNeed::HostRuntimeContext
        )));
        assert_eq!(surface.prepared_identity_facts.len(), 2);
    }

    #[test]
    fn surface_records_workspace_qualified_reverse_edges() {
        let target = WorkspaceQualifiedTarget {
            workspace_handle: "treecalc-workspace:projections".to_string(),
            target_node_id: TreeNodeId(102),
            target_node_handle: "treecalc-workspace:projections#node:102".to_string(),
            availability_version: "treecalc-cross-workspace-availability:v1:projections:loaded"
                .to_string(),
        };
        let descriptor = DependencyDescriptor {
            descriptor_id: "dep:xws".to_string(),
            source_reference_handle: Some("treecalc_reference_carrier:TREE_REF_4_0".to_string()),
            owner_node_id: TreeNodeId(4),
            target_node_id: None,
            workspace_target: Some(target.clone()),
            kind: DependencyDescriptorKind::HostSensitive,
            carrier_detail: "cross_workspace_resolved:carrier:xws".to_string(),
            tree_reference_collection: None,
            requires_rebind_on_structural_change: true,
        };
        let graph = DependencyGraph::build(&snapshot(), &[descriptor]);
        let surface = w056_reference_dependency_surface(&graph);

        assert!(surface.target_reverse_edges.is_empty());
        assert!(surface.context_reverse_edges.is_empty());
        assert_eq!(surface.workspace_target_reverse_edges.len(), 1);
        assert_eq!(surface.workspace_target_reverse_edges[0].target, target);
        assert_eq!(
            surface.descriptor_facts[0].workspace_target,
            Some(target.clone())
        );
        assert_eq!(
            surface.descriptor_facts[0].namespace_identity_need,
            NamespaceIdentityNeed::CrossWorkspaceAvailabilityVersion
        );
        assert!(
            prepared_identity_needs_by_owner(&surface)[&TreeNodeId(4)].contains(&(
                NamespaceIdentityNeed::CrossWorkspaceAvailabilityVersion,
                CallerContextIdentityNeed::None
            ))
        );
    }

    #[test]
    fn dynamic_rebind_facts_distinguish_potential_and_resolved_edges() {
        let graph = DependencyGraph::build(
            &snapshot(),
            &[
                descriptor(
                    "dep:dynamic:potential",
                    TreeNodeId(3),
                    None,
                    DependencyDescriptorKind::DynamicPotential,
                    "dynamic_potential:late",
                    Some("runtime_fact:DynamicPotential:late"),
                    false,
                ),
                descriptor(
                    "dep:dynamic:resolved",
                    TreeNodeId(4),
                    Some(TreeNodeId(2)),
                    DependencyDescriptorKind::DynamicPotential,
                    "dynamic_resolved:node:2:late",
                    Some("treecalc_reference_carrier:TREE_REF_4_0"),
                    true,
                ),
            ],
        );

        let surface = w056_reference_dependency_surface(&graph);

        assert_eq!(surface.dynamic_rebind_facts.len(), 2);
        assert_eq!(
            surface.dynamic_rebind_facts[0].state,
            DynamicRebindState::Potential
        );
        assert_eq!(
            surface.dynamic_rebind_facts[1].state,
            DynamicRebindState::Resolved
        );
        assert!(
            surface.dynamic_rebind_facts[1]
                .invalidation_facts
                .contains(&InvalidationReasonKind::DynamicDependencyReleased)
        );
        assert_eq!(
            surface.target_reverse_edges[0].descriptor_id,
            "dep:dynamic:resolved"
        );
    }

    #[test]
    fn inventory_surface_admits_cross_workspace_workspace_qualified_carrier() {
        let graph = DependencyGraph::build(&snapshot(), &[]);
        let surface = w056_reference_dependency_surface(&graph);

        let cross_workspace = surface
            .admitted_inventory_inputs
            .iter()
            .find(|input| input.variant == TreeReferenceInventoryVariant::CrossWorkspaceReference)
            .expect("cross-workspace inventory input should be admitted");

        assert_eq!(cross_workspace.blocker, None);
        assert_eq!(
            cross_workspace.namespace_identity_need,
            NamespaceIdentityNeed::CrossWorkspaceAvailabilityVersion
        );
        assert!(cross_workspace.evidence_note.contains("calc-8tox"));
    }
}
