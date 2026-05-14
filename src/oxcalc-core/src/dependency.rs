#![forbid(unsafe_code)]

//! Dependency graph and invalidation substrate for TreeCalc widening.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::recalc::NodeCalcState;
use crate::structural::{StructuralSnapshot, StructuralSnapshotId, TreeNodeId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DependencyDescriptorKind {
    StaticDirect,
    RelativeBound,
    DynamicPotential,
    HostSensitive,
    CapabilitySensitive,
    ShapeTopology,
    Unresolved,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyDescriptor {
    pub descriptor_id: String,
    pub source_reference_handle: Option<String>,
    pub owner_node_id: TreeNodeId,
    pub target_node_id: Option<TreeNodeId>,
    pub kind: DependencyDescriptorKind,
    pub carrier_detail: String,
    pub requires_rebind_on_structural_change: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyEdge {
    pub edge_id: String,
    pub descriptor_id: String,
    pub owner_node_id: TreeNodeId,
    pub target_node_id: TreeNodeId,
    pub kind: DependencyDescriptorKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DependencyDiagnosticKind {
    MissingOwner,
    MissingTarget,
    UnresolvedReference,
    HostSensitiveReference,
    DynamicPotentialReference,
    CapabilitySensitiveReference,
    ShapeTopologyReference,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyDiagnostic {
    pub descriptor_id: String,
    pub kind: DependencyDiagnosticKind,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyGraph {
    pub snapshot_id: StructuralSnapshotId,
    pub descriptors_by_owner: BTreeMap<TreeNodeId, Vec<DependencyDescriptor>>,
    pub edges_by_owner: BTreeMap<TreeNodeId, Vec<DependencyEdge>>,
    pub reverse_edges: BTreeMap<TreeNodeId, Vec<DependencyEdge>>,
    pub cycle_groups: Vec<Vec<TreeNodeId>>,
    pub diagnostics: Vec<DependencyDiagnostic>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InvalidationReasonKind {
    StructuralRebindRequired,
    StructuralRecalcOnly,
    UpstreamPublication,
    DependencyAdded,
    DependencyRemoved,
    DependencyReclassified,
    DynamicDependencyActivated,
    DynamicDependencyReleased,
    DynamicDependencyReclassified,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidationSeed {
    pub node_id: TreeNodeId,
    pub reason: InvalidationReasonKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeInvalidationRecord {
    pub node_id: TreeNodeId,
    pub calc_state: NodeCalcState,
    pub requires_rebind: bool,
    pub reasons: Vec<InvalidationReasonKind>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidationClosure {
    pub impacted_order: Vec<TreeNodeId>,
    pub records: BTreeMap<TreeNodeId, NodeInvalidationRecord>,
}

impl DependencyGraph {
    #[must_use]
    pub fn build(snapshot: &StructuralSnapshot, descriptors: &[DependencyDescriptor]) -> Self {
        let mut descriptors_by_owner = BTreeMap::<TreeNodeId, Vec<DependencyDescriptor>>::new();
        let mut edges_by_owner = BTreeMap::<TreeNodeId, Vec<DependencyEdge>>::new();
        let mut diagnostics = Vec::new();

        for descriptor in descriptors {
            descriptors_by_owner
                .entry(descriptor.owner_node_id)
                .or_default()
                .push(descriptor.clone());

            if snapshot.try_get_node(descriptor.owner_node_id).is_none() {
                diagnostics.push(DependencyDiagnostic {
                    descriptor_id: descriptor.descriptor_id.clone(),
                    kind: DependencyDiagnosticKind::MissingOwner,
                    detail: format!(
                        "owner {} is not present in snapshot",
                        descriptor.owner_node_id
                    ),
                });
                continue;
            }

            match descriptor.target_node_id {
                Some(target_node_id) => {
                    if snapshot.try_get_node(target_node_id).is_none() {
                        diagnostics.push(DependencyDiagnostic {
                            descriptor_id: descriptor.descriptor_id.clone(),
                            kind: DependencyDiagnosticKind::MissingTarget,
                            detail: format!("target {} is not present in snapshot", target_node_id),
                        });
                        continue;
                    }

                    let edge = DependencyEdge {
                        edge_id: format!(
                            "dep:{}:{}:{}",
                            descriptor.owner_node_id.0, target_node_id.0, descriptor.descriptor_id
                        ),
                        descriptor_id: descriptor.descriptor_id.clone(),
                        owner_node_id: descriptor.owner_node_id,
                        target_node_id,
                        kind: descriptor.kind,
                    };
                    edges_by_owner
                        .entry(descriptor.owner_node_id)
                        .or_default()
                        .push(edge);
                }
                None => {
                    let kind = match descriptor.kind {
                        DependencyDescriptorKind::HostSensitive => {
                            DependencyDiagnosticKind::HostSensitiveReference
                        }
                        DependencyDescriptorKind::DynamicPotential => {
                            DependencyDiagnosticKind::DynamicPotentialReference
                        }
                        DependencyDescriptorKind::CapabilitySensitive => {
                            DependencyDiagnosticKind::CapabilitySensitiveReference
                        }
                        DependencyDescriptorKind::ShapeTopology => {
                            DependencyDiagnosticKind::ShapeTopologyReference
                        }
                        _ => DependencyDiagnosticKind::UnresolvedReference,
                    };
                    diagnostics.push(DependencyDiagnostic {
                        descriptor_id: descriptor.descriptor_id.clone(),
                        kind,
                        detail: descriptor.carrier_detail.clone(),
                    });
                }
            }
        }

        for descriptors in descriptors_by_owner.values_mut() {
            descriptors.sort_by(|left, right| left.descriptor_id.cmp(&right.descriptor_id));
        }

        for edges in edges_by_owner.values_mut() {
            edges.sort_by(|left, right| left.edge_id.cmp(&right.edge_id));
        }

        let mut reverse_edges = BTreeMap::<TreeNodeId, Vec<DependencyEdge>>::new();
        for edges in edges_by_owner.values() {
            for edge in edges {
                reverse_edges
                    .entry(edge.target_node_id)
                    .or_default()
                    .push(edge.clone());
            }
        }
        for edges in reverse_edges.values_mut() {
            edges.sort_by(|left, right| left.edge_id.cmp(&right.edge_id));
        }

        let cycle_groups = find_cycle_groups(snapshot, &edges_by_owner);

        Self {
            snapshot_id: snapshot.snapshot_id(),
            descriptors_by_owner,
            edges_by_owner,
            reverse_edges,
            cycle_groups,
            diagnostics,
        }
    }

    #[must_use]
    pub fn derive_invalidation_closure(&self, seeds: &[InvalidationSeed]) -> InvalidationClosure {
        let cycle_members = self
            .cycle_groups
            .iter()
            .flatten()
            .copied()
            .collect::<BTreeSet<_>>();

        let mut impacted_order = Vec::new();
        let mut queued = BTreeSet::new();
        let mut queue = VecDeque::new();
        let mut records = BTreeMap::<TreeNodeId, NodeInvalidationRecord>::new();

        for seed in seeds {
            if queued.insert(seed.node_id) {
                queue.push_back(seed.node_id);
                impacted_order.push(seed.node_id);
            }

            let record = records
                .entry(seed.node_id)
                .or_insert_with(|| NodeInvalidationRecord {
                    node_id: seed.node_id,
                    calc_state: NodeCalcState::DirtyPending,
                    requires_rebind: false,
                    reasons: Vec::new(),
                });
            if !record.reasons.contains(&seed.reason) {
                record.reasons.push(seed.reason);
            }
            if reason_requires_rebind(seed.reason) {
                record.requires_rebind = true;
            }
            record.calc_state =
                derive_seed_state(seed.reason, cycle_members.contains(&seed.node_id));
        }

        while let Some(target_node_id) = queue.pop_front() {
            let Some(reverse_edges) = self.reverse_edges.get(&target_node_id) else {
                continue;
            };

            for edge in reverse_edges {
                let dependent_node_id = edge.owner_node_id;
                let record =
                    records
                        .entry(dependent_node_id)
                        .or_insert_with(|| NodeInvalidationRecord {
                            node_id: dependent_node_id,
                            calc_state: NodeCalcState::DirtyPending,
                            requires_rebind: false,
                            reasons: Vec::new(),
                        });

                if !record
                    .reasons
                    .contains(&InvalidationReasonKind::UpstreamPublication)
                {
                    record
                        .reasons
                        .push(InvalidationReasonKind::UpstreamPublication);
                }
                if cycle_members.contains(&dependent_node_id) {
                    record.calc_state = NodeCalcState::CycleBlocked;
                }

                if queued.insert(dependent_node_id) {
                    queue.push_back(dependent_node_id);
                    impacted_order.push(dependent_node_id);
                }
            }
        }

        for record in records.values_mut() {
            record.reasons.sort();
        }

        InvalidationClosure {
            impacted_order,
            records,
        }
    }
}

fn derive_seed_state(reason: InvalidationReasonKind, in_cycle: bool) -> NodeCalcState {
    if in_cycle {
        return NodeCalcState::CycleBlocked;
    }

    match reason {
        InvalidationReasonKind::UpstreamPublication => NodeCalcState::Needed,
        InvalidationReasonKind::StructuralRecalcOnly => NodeCalcState::DirtyPending,
        InvalidationReasonKind::StructuralRebindRequired
        | InvalidationReasonKind::DependencyAdded
        | InvalidationReasonKind::DependencyRemoved
        | InvalidationReasonKind::DependencyReclassified
        | InvalidationReasonKind::DynamicDependencyActivated
        | InvalidationReasonKind::DynamicDependencyReleased
        | InvalidationReasonKind::DynamicDependencyReclassified => NodeCalcState::DirtyPending,
    }
}

fn reason_requires_rebind(reason: InvalidationReasonKind) -> bool {
    matches!(
        reason,
        InvalidationReasonKind::StructuralRebindRequired
            | InvalidationReasonKind::DependencyAdded
            | InvalidationReasonKind::DependencyRemoved
            | InvalidationReasonKind::DependencyReclassified
    )
}

fn find_cycle_groups(
    snapshot: &StructuralSnapshot,
    edges_by_owner: &BTreeMap<TreeNodeId, Vec<DependencyEdge>>,
) -> Vec<Vec<TreeNodeId>> {
    struct TarjanState {
        index: usize,
        stack: Vec<TreeNodeId>,
        on_stack: BTreeSet<TreeNodeId>,
        indices: BTreeMap<TreeNodeId, usize>,
        lowlinks: BTreeMap<TreeNodeId, usize>,
        groups: Vec<Vec<TreeNodeId>>,
    }

    fn strong_connect(
        node_id: TreeNodeId,
        edges_by_owner: &BTreeMap<TreeNodeId, Vec<DependencyEdge>>,
        state: &mut TarjanState,
    ) {
        state.indices.insert(node_id, state.index);
        state.lowlinks.insert(node_id, state.index);
        state.index += 1;
        state.stack.push(node_id);
        state.on_stack.insert(node_id);

        if let Some(edges) = edges_by_owner.get(&node_id) {
            for edge in edges {
                let successor = edge.target_node_id;
                if !state.indices.contains_key(&successor) {
                    strong_connect(successor, edges_by_owner, state);
                    let successor_lowlink = state.lowlinks[&successor];
                    let current_lowlink = state.lowlinks[&node_id];
                    state
                        .lowlinks
                        .insert(node_id, current_lowlink.min(successor_lowlink));
                } else if state.on_stack.contains(&successor) {
                    let successor_index = state.indices[&successor];
                    let current_lowlink = state.lowlinks[&node_id];
                    state
                        .lowlinks
                        .insert(node_id, current_lowlink.min(successor_index));
                }
            }
        }

        if state.indices[&node_id] == state.lowlinks[&node_id] {
            let mut group = Vec::new();
            while let Some(stack_node_id) = state.stack.pop() {
                state.on_stack.remove(&stack_node_id);
                group.push(stack_node_id);
                if stack_node_id == node_id {
                    break;
                }
            }

            group.sort();
            let is_self_cycle = edges_by_owner
                .get(&node_id)
                .is_some_and(|edges| edges.iter().any(|edge| edge.target_node_id == node_id));
            if group.len() > 1 || is_self_cycle {
                state.groups.push(group);
            }
        }
    }

    let mut state = TarjanState {
        index: 0,
        stack: Vec::new(),
        on_stack: BTreeSet::new(),
        indices: BTreeMap::new(),
        lowlinks: BTreeMap::new(),
        groups: Vec::new(),
    };

    for node_id in snapshot.nodes().keys().copied() {
        if !state.indices.contains_key(&node_id) {
            strong_connect(node_id, edges_by_owner, &mut state);
        }
    }

    state.groups.sort();
    state.groups
}

#[cfg(test)]
mod tests {
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
                    formula_artifact_id: None,
                    bind_artifact_id: None,
                    constant_value: None,
                },
                StructuralNode {
                    node_id: TreeNodeId(2),
                    kind: StructuralNodeKind::Calculation,
                    symbol: "A".to_string(),
                    parent_id: Some(TreeNodeId(1)),
                    child_ids: vec![],
                    formula_artifact_id: None,
                    bind_artifact_id: None,
                    constant_value: None,
                },
                StructuralNode {
                    node_id: TreeNodeId(3),
                    kind: StructuralNodeKind::Calculation,
                    symbol: "B".to_string(),
                    parent_id: Some(TreeNodeId(1)),
                    child_ids: vec![],
                    formula_artifact_id: None,
                    bind_artifact_id: None,
                    constant_value: None,
                },
                StructuralNode {
                    node_id: TreeNodeId(4),
                    kind: StructuralNodeKind::Calculation,
                    symbol: "C".to_string(),
                    parent_id: Some(TreeNodeId(1)),
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
    fn dependency_graph_builds_edges_and_cycle_groups() {
        let graph = DependencyGraph::build(
            &snapshot(),
            &[
                DependencyDescriptor {
                    descriptor_id: "dep-a-b".to_string(),
                    source_reference_handle: None,
                    owner_node_id: TreeNodeId(2),
                    target_node_id: Some(TreeNodeId(3)),
                    kind: DependencyDescriptorKind::StaticDirect,
                    carrier_detail: "A->B".to_string(),
                    requires_rebind_on_structural_change: false,
                },
                DependencyDescriptor {
                    descriptor_id: "dep-b-a".to_string(),
                    source_reference_handle: None,
                    owner_node_id: TreeNodeId(3),
                    target_node_id: Some(TreeNodeId(2)),
                    kind: DependencyDescriptorKind::RelativeBound,
                    carrier_detail: "B->A".to_string(),
                    requires_rebind_on_structural_change: true,
                },
                DependencyDescriptor {
                    descriptor_id: "dep-c-unresolved".to_string(),
                    source_reference_handle: None,
                    owner_node_id: TreeNodeId(4),
                    target_node_id: None,
                    kind: DependencyDescriptorKind::Unresolved,
                    carrier_detail: "relative sibling lookup".to_string(),
                    requires_rebind_on_structural_change: true,
                },
            ],
        );

        assert_eq!(graph.edges_by_owner[&TreeNodeId(2)].len(), 1);
        assert_eq!(graph.descriptors_by_owner[&TreeNodeId(2)].len(), 1);
        assert_eq!(
            graph.descriptors_by_owner[&TreeNodeId(4)][0].descriptor_id,
            "dep-c-unresolved"
        );
        assert_eq!(graph.reverse_edges[&TreeNodeId(2)].len(), 1);
        assert_eq!(graph.cycle_groups, vec![vec![TreeNodeId(2), TreeNodeId(3)]]);
        assert_eq!(graph.diagnostics.len(), 1);
        assert_eq!(
            graph.diagnostics[0].kind,
            DependencyDiagnosticKind::UnresolvedReference
        );
    }

    #[test]
    fn invalidation_closure_distinguishes_rebind_and_cycle_blocked_records() {
        let graph = DependencyGraph::build(
            &snapshot(),
            &[
                DependencyDescriptor {
                    descriptor_id: "dep-a-b".to_string(),
                    source_reference_handle: None,
                    owner_node_id: TreeNodeId(2),
                    target_node_id: Some(TreeNodeId(3)),
                    kind: DependencyDescriptorKind::StaticDirect,
                    carrier_detail: "A->B".to_string(),
                    requires_rebind_on_structural_change: false,
                },
                DependencyDescriptor {
                    descriptor_id: "dep-b-a".to_string(),
                    source_reference_handle: None,
                    owner_node_id: TreeNodeId(3),
                    target_node_id: Some(TreeNodeId(2)),
                    kind: DependencyDescriptorKind::RelativeBound,
                    carrier_detail: "B->A".to_string(),
                    requires_rebind_on_structural_change: true,
                },
                DependencyDescriptor {
                    descriptor_id: "dep-c-a".to_string(),
                    source_reference_handle: None,
                    owner_node_id: TreeNodeId(4),
                    target_node_id: Some(TreeNodeId(2)),
                    kind: DependencyDescriptorKind::StaticDirect,
                    carrier_detail: "C->A".to_string(),
                    requires_rebind_on_structural_change: false,
                },
            ],
        );

        let closure = graph.derive_invalidation_closure(&[InvalidationSeed {
            node_id: TreeNodeId(2),
            reason: InvalidationReasonKind::StructuralRebindRequired,
        }]);

        assert_eq!(
            closure.records[&TreeNodeId(2)].calc_state,
            NodeCalcState::CycleBlocked
        );
        assert!(closure.records[&TreeNodeId(2)].requires_rebind);
        assert_eq!(
            closure.records[&TreeNodeId(4)].calc_state,
            NodeCalcState::DirtyPending
        );
        assert!(!closure.records[&TreeNodeId(4)].requires_rebind);
    }

    #[test]
    fn invalidation_closure_records_upstream_publication_for_seed_and_dependents() {
        let graph = DependencyGraph::build(
            &snapshot(),
            &[DependencyDescriptor {
                descriptor_id: "dep-c-a".to_string(),
                source_reference_handle: None,
                owner_node_id: TreeNodeId(4),
                target_node_id: Some(TreeNodeId(2)),
                kind: DependencyDescriptorKind::StaticDirect,
                carrier_detail: "C->A".to_string(),
                requires_rebind_on_structural_change: false,
            }],
        );

        let closure = graph.derive_invalidation_closure(&[InvalidationSeed {
            node_id: TreeNodeId(2),
            reason: InvalidationReasonKind::UpstreamPublication,
        }]);

        assert_eq!(
            closure.records[&TreeNodeId(2)].calc_state,
            NodeCalcState::Needed
        );
        assert_eq!(
            closure.records[&TreeNodeId(2)].reasons,
            vec![InvalidationReasonKind::UpstreamPublication]
        );
        assert_eq!(
            closure.records[&TreeNodeId(4)].reasons,
            vec![InvalidationReasonKind::UpstreamPublication]
        );
        assert_eq!(closure.impacted_order, vec![TreeNodeId(2), TreeNodeId(4)]);
    }

    #[test]
    fn invalidation_closure_marks_dependency_change_reasons_as_rebind_required() {
        let graph = DependencyGraph::build(&snapshot(), &[]);

        let closure = graph.derive_invalidation_closure(&[
            InvalidationSeed {
                node_id: TreeNodeId(4),
                reason: InvalidationReasonKind::DependencyRemoved,
            },
            InvalidationSeed {
                node_id: TreeNodeId(4),
                reason: InvalidationReasonKind::DependencyAdded,
            },
            InvalidationSeed {
                node_id: TreeNodeId(4),
                reason: InvalidationReasonKind::DependencyReclassified,
            },
        ]);

        assert_eq!(
            closure.records[&TreeNodeId(4)].calc_state,
            NodeCalcState::DirtyPending
        );
        assert!(closure.records[&TreeNodeId(4)].requires_rebind);
        assert_eq!(
            closure.records[&TreeNodeId(4)].reasons,
            vec![
                InvalidationReasonKind::DependencyAdded,
                InvalidationReasonKind::DependencyRemoved,
                InvalidationReasonKind::DependencyReclassified,
            ]
        );
    }

    #[test]
    fn invalidation_closure_keeps_dynamic_dependency_changes_repairable() {
        let graph = DependencyGraph::build(&snapshot(), &[]);

        let closure = graph.derive_invalidation_closure(&[
            InvalidationSeed {
                node_id: TreeNodeId(4),
                reason: InvalidationReasonKind::DynamicDependencyReleased,
            },
            InvalidationSeed {
                node_id: TreeNodeId(4),
                reason: InvalidationReasonKind::DynamicDependencyActivated,
            },
            InvalidationSeed {
                node_id: TreeNodeId(4),
                reason: InvalidationReasonKind::DynamicDependencyReclassified,
            },
        ]);

        assert_eq!(
            closure.records[&TreeNodeId(4)].calc_state,
            NodeCalcState::DirtyPending
        );
        assert!(!closure.records[&TreeNodeId(4)].requires_rebind);
        assert_eq!(
            closure.records[&TreeNodeId(4)].reasons,
            vec![
                InvalidationReasonKind::DynamicDependencyActivated,
                InvalidationReasonKind::DynamicDependencyReleased,
                InvalidationReasonKind::DynamicDependencyReclassified,
            ]
        );
    }
}
