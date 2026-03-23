#![forbid(unsafe_code)]

//! Structural snapshot kernel lane.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TreeNodeId(pub u64);

impl Display for TreeNodeId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "node:{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct StructuralSnapshotId(pub u64);

impl Display for StructuralSnapshotId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "snapshot:{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct FormulaArtifactId(pub String);

impl Display for FormulaArtifactId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct BindArtifactId(pub String);

impl Display for BindArtifactId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StructuralNodeKind {
    Root,
    Container,
    Calculation,
    Constant,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuralNode {
    pub node_id: TreeNodeId,
    pub kind: StructuralNodeKind,
    pub symbol: String,
    pub parent_id: Option<TreeNodeId>,
    pub child_ids: Vec<TreeNodeId>,
    pub formula_artifact_id: Option<FormulaArtifactId>,
    pub bind_artifact_id: Option<BindArtifactId>,
    pub constant_value: Option<String>,
}

impl StructuralNode {
    #[must_use]
    pub fn with_parent(mut self, parent_id: Option<TreeNodeId>) -> Self {
        self.parent_id = parent_id;
        self
    }

    #[must_use]
    pub fn with_children(mut self, child_ids: Vec<TreeNodeId>) -> Self {
        self.child_ids = child_ids;
        self
    }

    #[must_use]
    pub fn with_symbol(mut self, symbol: impl Into<String>) -> Self {
        self.symbol = symbol.into();
        self
    }

    #[must_use]
    pub fn with_formula_attachment(
        mut self,
        formula_artifact_id: Option<FormulaArtifactId>,
        bind_artifact_id: Option<BindArtifactId>,
    ) -> Self {
        self.formula_artifact_id = formula_artifact_id;
        self.bind_artifact_id = bind_artifact_id;
        self
    }

    #[must_use]
    pub fn with_constant_value(mut self, constant_value: Option<String>) -> Self {
        self.constant_value = constant_value;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelativeReferenceContext {
    pub node_id: TreeNodeId,
    pub parent_id: Option<TreeNodeId>,
    pub ancestor_ids: Vec<TreeNodeId>,
    pub sibling_index: Option<usize>,
    pub projection_path: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructuralEditImpact {
    NoRebind,
    RecalcOnly,
    RebindRequired,
    Removal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuralEditOutcome {
    pub snapshot: StructuralSnapshot,
    pub impact: StructuralEditImpact,
    pub affected_node_ids: Vec<TreeNodeId>,
    pub diagnostic_events: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StructuralEdit {
    RenameNode {
        node_id: TreeNodeId,
        new_symbol: String,
    },
    MoveNode {
        node_id: TreeNodeId,
        new_parent_id: TreeNodeId,
        new_index: Option<usize>,
    },
    ReplaceFormulaAttachment {
        node_id: TreeNodeId,
        formula_artifact_id: Option<FormulaArtifactId>,
        bind_artifact_id: Option<BindArtifactId>,
    },
    SetConstantValue {
        node_id: TreeNodeId,
        constant_value: Option<String>,
    },
    InsertNode {
        node: StructuralNode,
        parent_id: TreeNodeId,
        index: Option<usize>,
    },
    RemoveNode {
        node_id: TreeNodeId,
    },
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum StructuralError {
    #[error("snapshot {snapshot_id} does not contain the declared root {root_node_id}")]
    MissingRoot {
        snapshot_id: StructuralSnapshotId,
        root_node_id: TreeNodeId,
    },
    #[error("root {root_node_id} may not declare a parent")]
    RootHasParent { root_node_id: TreeNodeId },
    #[error("cycle or duplicate structural reachability detected at {node_id}")]
    CycleDetected { node_id: TreeNodeId },
    #[error("node {node_id} references missing child {child_id}")]
    MissingChild {
        node_id: TreeNodeId,
        child_id: TreeNodeId,
    },
    #[error("child {child_id} does not point back to parent {parent_id}")]
    ParentMismatch {
        child_id: TreeNodeId,
        parent_id: TreeNodeId,
    },
    #[error("snapshot {snapshot_id} contains detached or unreachable nodes: {detached}")]
    DetachedNodes {
        snapshot_id: StructuralSnapshotId,
        detached: String,
    },
    #[error("projection path '{projection_path}' is not unique within snapshot {snapshot_id}")]
    DuplicateProjectionPath {
        snapshot_id: StructuralSnapshotId,
        projection_path: String,
    },
    #[error("cannot remove the structural root {node_id}")]
    CannotRemoveRoot { node_id: TreeNodeId },
    #[error("cannot move the structural root {node_id}")]
    CannotMoveRoot { node_id: TreeNodeId },
    #[error("child insertion index {index} is out of range for parent {parent_id}")]
    InvalidChildInsertionIndex { parent_id: TreeNodeId, index: usize },
    #[error("unknown node {node_id}")]
    UnknownNode { node_id: TreeNodeId },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuralSnapshot {
    snapshot_id: StructuralSnapshotId,
    root_node_id: TreeNodeId,
    nodes: BTreeMap<TreeNodeId, StructuralNode>,
    path_index: BTreeMap<String, TreeNodeId>,
}

impl StructuralSnapshot {
    pub fn create(
        snapshot_id: StructuralSnapshotId,
        root_node_id: TreeNodeId,
        nodes: impl IntoIterator<Item = StructuralNode>,
    ) -> Result<Self, StructuralError> {
        let node_map = nodes
            .into_iter()
            .map(|node| (node.node_id, node))
            .collect::<BTreeMap<_, _>>();
        validate(snapshot_id, root_node_id, &node_map)?;
        let path_index = build_path_index(snapshot_id, root_node_id, &node_map)?;

        Ok(Self {
            snapshot_id,
            root_node_id,
            nodes: node_map,
            path_index,
        })
    }

    #[must_use]
    pub fn snapshot_id(&self) -> StructuralSnapshotId {
        self.snapshot_id
    }

    #[must_use]
    pub fn root_node_id(&self) -> TreeNodeId {
        self.root_node_id
    }

    #[must_use]
    pub fn nodes(&self) -> &BTreeMap<TreeNodeId, StructuralNode> {
        &self.nodes
    }

    pub fn try_get_node(&self, node_id: TreeNodeId) -> Option<&StructuralNode> {
        self.nodes.get(&node_id)
    }

    pub fn get_projection_path(&self, node_id: TreeNodeId) -> Result<String, StructuralError> {
        let mut segments = Vec::new();
        let mut cursor = self
            .nodes
            .get(&node_id)
            .ok_or(StructuralError::UnknownNode { node_id })?;

        loop {
            segments.push(cursor.symbol.clone());
            match cursor.parent_id {
                Some(parent_id) => {
                    cursor = self
                        .nodes
                        .get(&parent_id)
                        .ok_or(StructuralError::UnknownNode { node_id: parent_id })?;
                }
                None => break,
            }
        }

        segments.reverse();
        Ok(segments.join("/"))
    }

    pub fn try_resolve_projection_path(&self, projection_path: &str) -> Option<TreeNodeId> {
        self.path_index.get(projection_path).copied()
    }

    #[must_use]
    pub fn parent_id_of(&self, node_id: TreeNodeId) -> Option<TreeNodeId> {
        self.try_get_node(node_id).and_then(|node| node.parent_id)
    }

    pub fn nth_ancestor_of(&self, node_id: TreeNodeId, levels_up: usize) -> Option<TreeNodeId> {
        let mut cursor = Some(node_id);
        for _ in 0..levels_up {
            cursor = cursor.and_then(|current| self.parent_id_of(current));
        }
        cursor
    }

    pub fn try_resolve_child_by_symbol(
        &self,
        parent_id: TreeNodeId,
        symbol: &str,
    ) -> Option<TreeNodeId> {
        self.try_get_node(parent_id).and_then(|parent| {
            parent.child_ids.iter().copied().find(|child_id| {
                self.try_get_node(*child_id)
                    .is_some_and(|child| child.symbol == symbol)
            })
        })
    }

    pub fn try_resolve_descendant_path(
        &self,
        start_node_id: TreeNodeId,
        path_segments: &[String],
    ) -> Option<TreeNodeId> {
        let mut cursor = Some(start_node_id);
        for segment in path_segments {
            cursor = cursor.and_then(|current| self.try_resolve_child_by_symbol(current, segment));
        }
        cursor
    }

    pub fn try_resolve_sibling_offset(
        &self,
        node_id: TreeNodeId,
        offset: isize,
    ) -> Option<TreeNodeId> {
        let context = self.describe_relative_context(node_id).ok()?;
        let parent_id = context.parent_id?;
        let parent = self.try_get_node(parent_id)?;
        let base_index = isize::try_from(context.sibling_index?).ok()?;
        let target_index = base_index.checked_add(offset)?;
        let target_index = usize::try_from(target_index).ok()?;
        parent.child_ids.get(target_index).copied()
    }

    pub fn describe_relative_context(
        &self,
        node_id: TreeNodeId,
    ) -> Result<RelativeReferenceContext, StructuralError> {
        let node = self
            .nodes
            .get(&node_id)
            .ok_or(StructuralError::UnknownNode { node_id })?;
        let projection_path = self.get_projection_path(node_id)?;
        let sibling_index = match node.parent_id {
            Some(parent_id) => {
                let parent = self
                    .nodes
                    .get(&parent_id)
                    .ok_or(StructuralError::UnknownNode { node_id: parent_id })?;
                parent
                    .child_ids
                    .iter()
                    .position(|child_id| *child_id == node_id)
            }
            None => None,
        };
        let mut ancestor_ids = Vec::new();
        let mut cursor = node.parent_id;
        while let Some(parent_id) = cursor {
            ancestor_ids.push(parent_id);
            cursor = self
                .nodes
                .get(&parent_id)
                .ok_or(StructuralError::UnknownNode { node_id: parent_id })?
                .parent_id;
        }

        Ok(RelativeReferenceContext {
            node_id,
            parent_id: node.parent_id,
            ancestor_ids,
            sibling_index,
            projection_path,
        })
    }

    pub fn apply_edit(
        &self,
        successor_snapshot_id: StructuralSnapshotId,
        edit: StructuralEdit,
    ) -> Result<StructuralEditOutcome, StructuralError> {
        let mut builder = StructuralSnapshotBuilder::new(Some(self));

        let (impact, affected_node_ids, diagnostic_events) = match edit {
            StructuralEdit::RenameNode {
                node_id,
                new_symbol,
            } => {
                builder.rename_node(node_id, new_symbol.clone())?;
                (
                    StructuralEditImpact::RebindRequired,
                    vec![node_id],
                    vec![format!("node_renamed:{node_id}:{new_symbol}")],
                )
            }
            StructuralEdit::MoveNode {
                node_id,
                new_parent_id,
                new_index,
            } => {
                builder.move_node(node_id, new_parent_id, new_index)?;
                (
                    StructuralEditImpact::RebindRequired,
                    vec![node_id, new_parent_id],
                    vec![format!("node_moved:{node_id}:{new_parent_id}")],
                )
            }
            StructuralEdit::ReplaceFormulaAttachment {
                node_id,
                formula_artifact_id,
                bind_artifact_id,
            } => {
                builder.replace_formula_attachment(
                    node_id,
                    formula_artifact_id.clone(),
                    bind_artifact_id.clone(),
                )?;
                (
                    StructuralEditImpact::RebindRequired,
                    vec![node_id],
                    vec![format!(
                        "formula_attachment_replaced:{node_id}:{}:{}",
                        formula_artifact_id
                            .as_ref()
                            .map_or("none", |id| id.0.as_str()),
                        bind_artifact_id.as_ref().map_or("none", |id| id.0.as_str())
                    )],
                )
            }
            StructuralEdit::SetConstantValue {
                node_id,
                constant_value,
            } => {
                builder.set_constant_value(node_id, constant_value.clone())?;
                (
                    StructuralEditImpact::RecalcOnly,
                    vec![node_id],
                    vec![format!(
                        "constant_value_changed:{node_id}:{}",
                        constant_value.unwrap_or_default()
                    )],
                )
            }
            StructuralEdit::InsertNode {
                node,
                parent_id,
                index,
            } => {
                let inserted_id = node.node_id;
                builder.insert_node(node, parent_id, index)?;
                (
                    StructuralEditImpact::RebindRequired,
                    vec![inserted_id, parent_id],
                    vec![format!("node_inserted:{inserted_id}:{parent_id}")],
                )
            }
            StructuralEdit::RemoveNode { node_id } => {
                let removed = builder.remove_node(node_id)?;
                (
                    StructuralEditImpact::Removal,
                    removed.clone(),
                    vec![format!(
                        "node_removed:{}",
                        removed
                            .into_iter()
                            .map(|id| id.to_string())
                            .collect::<Vec<_>>()
                            .join(",")
                    )],
                )
            }
        };

        let snapshot = builder.build(successor_snapshot_id)?;
        Ok(StructuralEditOutcome {
            snapshot,
            impact,
            affected_node_ids,
            diagnostic_events,
        })
    }

    #[must_use]
    pub fn pin(&self) -> PinnedStructuralView {
        PinnedStructuralView {
            snapshot: self.clone(),
        }
    }
}

fn validate(
    snapshot_id: StructuralSnapshotId,
    root_node_id: TreeNodeId,
    nodes: &BTreeMap<TreeNodeId, StructuralNode>,
) -> Result<(), StructuralError> {
    let root = nodes
        .get(&root_node_id)
        .ok_or(StructuralError::MissingRoot {
            snapshot_id,
            root_node_id,
        })?;

    if root.parent_id.is_some() {
        return Err(StructuralError::RootHasParent { root_node_id });
    }

    let mut seen = BTreeSet::new();
    visit(root_node_id, nodes, &mut seen)?;

    if seen.len() != nodes.len() {
        let detached = nodes
            .keys()
            .filter(|node_id| !seen.contains(node_id))
            .map(std::string::ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ");
        return Err(StructuralError::DetachedNodes {
            snapshot_id,
            detached,
        });
    }

    Ok(())
}

fn visit(
    node_id: TreeNodeId,
    nodes: &BTreeMap<TreeNodeId, StructuralNode>,
    seen: &mut BTreeSet<TreeNodeId>,
) -> Result<(), StructuralError> {
    if !seen.insert(node_id) {
        return Err(StructuralError::CycleDetected { node_id });
    }

    let node = nodes
        .get(&node_id)
        .ok_or(StructuralError::UnknownNode { node_id })?;
    for child_id in &node.child_ids {
        let child = nodes.get(child_id).ok_or(StructuralError::MissingChild {
            node_id,
            child_id: *child_id,
        })?;
        if child.parent_id != Some(node_id) {
            return Err(StructuralError::ParentMismatch {
                child_id: *child_id,
                parent_id: node_id,
            });
        }
        visit(*child_id, nodes, seen)?;
    }

    Ok(())
}

fn build_path_index(
    snapshot_id: StructuralSnapshotId,
    root_node_id: TreeNodeId,
    nodes: &BTreeMap<TreeNodeId, StructuralNode>,
) -> Result<BTreeMap<String, TreeNodeId>, StructuralError> {
    let mut index = BTreeMap::new();
    let root = nodes
        .get(&root_node_id)
        .expect("validated snapshots contain the root");
    let mut stack = vec![(root_node_id, root.symbol.clone())];

    while let Some((node_id, path)) = stack.pop() {
        if index.insert(path.clone(), node_id).is_some() {
            return Err(StructuralError::DuplicateProjectionPath {
                snapshot_id,
                projection_path: path,
            });
        }

        let node = nodes
            .get(&node_id)
            .expect("validated snapshots contain child nodes");

        for child_id in node.child_ids.iter().rev() {
            let child = nodes
                .get(child_id)
                .expect("validated snapshots contain child nodes");
            stack.push((*child_id, format!("{path}/{}", child.symbol)));
        }
    }

    Ok(index)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PinnedStructuralView {
    snapshot: StructuralSnapshot,
}

impl PinnedStructuralView {
    #[must_use]
    pub fn snapshot_id(&self) -> StructuralSnapshotId {
        self.snapshot.snapshot_id()
    }

    #[must_use]
    pub fn root_node_id(&self) -> TreeNodeId {
        self.snapshot.root_node_id()
    }

    pub fn try_get_node(&self, node_id: TreeNodeId) -> Option<&StructuralNode> {
        self.snapshot.try_get_node(node_id)
    }

    pub fn get_projection_path(&self, node_id: TreeNodeId) -> Result<String, StructuralError> {
        self.snapshot.get_projection_path(node_id)
    }

    pub fn try_resolve_projection_path(&self, projection_path: &str) -> Option<TreeNodeId> {
        self.snapshot.try_resolve_projection_path(projection_path)
    }
}

#[derive(Debug, Clone)]
pub struct StructuralSnapshotBuilder {
    nodes: BTreeMap<TreeNodeId, StructuralNode>,
    root_node_id: Option<TreeNodeId>,
}

impl StructuralSnapshotBuilder {
    #[must_use]
    pub fn new(predecessor: Option<&StructuralSnapshot>) -> Self {
        match predecessor {
            Some(snapshot) => Self {
                nodes: snapshot.nodes.clone(),
                root_node_id: Some(snapshot.root_node_id),
            },
            None => Self {
                nodes: BTreeMap::new(),
                root_node_id: None,
            },
        }
    }

    #[must_use]
    pub fn root_node_id(&self) -> Option<TreeNodeId> {
        self.root_node_id
    }

    pub fn set_root(&mut self, root_node_id: TreeNodeId) {
        self.root_node_id = Some(root_node_id);
    }

    pub fn set_node(&mut self, node: StructuralNode) {
        self.nodes.insert(node.node_id, node);
    }

    pub fn rename_node(
        &mut self,
        node_id: TreeNodeId,
        new_symbol: String,
    ) -> Result<(), StructuralError> {
        let node = self
            .nodes
            .get(&node_id)
            .cloned()
            .ok_or(StructuralError::UnknownNode { node_id })?;
        self.nodes.insert(node_id, node.with_symbol(new_symbol));
        Ok(())
    }

    pub fn replace_formula_attachment(
        &mut self,
        node_id: TreeNodeId,
        formula_artifact_id: Option<FormulaArtifactId>,
        bind_artifact_id: Option<BindArtifactId>,
    ) -> Result<(), StructuralError> {
        let node = self
            .nodes
            .get(&node_id)
            .cloned()
            .ok_or(StructuralError::UnknownNode { node_id })?;
        self.nodes.insert(
            node_id,
            node.with_formula_attachment(formula_artifact_id, bind_artifact_id),
        );
        Ok(())
    }

    pub fn set_constant_value(
        &mut self,
        node_id: TreeNodeId,
        constant_value: Option<String>,
    ) -> Result<(), StructuralError> {
        let node = self
            .nodes
            .get(&node_id)
            .cloned()
            .ok_or(StructuralError::UnknownNode { node_id })?;
        self.nodes
            .insert(node_id, node.with_constant_value(constant_value));
        Ok(())
    }

    pub fn attach_child(
        &mut self,
        parent_id: TreeNodeId,
        child_id: TreeNodeId,
        index: Option<usize>,
    ) -> Result<(), StructuralError> {
        let parent = self
            .nodes
            .get(&parent_id)
            .cloned()
            .ok_or(StructuralError::UnknownNode { node_id: parent_id })?;
        let child = self
            .nodes
            .get(&child_id)
            .cloned()
            .ok_or(StructuralError::UnknownNode { node_id: child_id })?;

        let mut child_ids = parent
            .child_ids
            .iter()
            .copied()
            .filter(|existing| *existing != child_id)
            .collect::<Vec<_>>();
        let insertion_index = index.unwrap_or(child_ids.len());
        if insertion_index > child_ids.len() {
            return Err(StructuralError::InvalidChildInsertionIndex {
                parent_id,
                index: insertion_index,
            });
        }
        child_ids.insert(insertion_index, child_id);

        self.nodes
            .insert(parent_id, parent.with_children(child_ids));
        self.nodes
            .insert(child_id, child.with_parent(Some(parent_id)));
        Ok(())
    }

    pub fn replace_children(
        &mut self,
        parent_id: TreeNodeId,
        child_ids: Vec<TreeNodeId>,
    ) -> Result<(), StructuralError> {
        let parent = self
            .nodes
            .get(&parent_id)
            .cloned()
            .ok_or(StructuralError::UnknownNode { node_id: parent_id })?;
        self.nodes
            .insert(parent_id, parent.with_children(child_ids.clone()));

        for child_id in child_ids {
            let child = self
                .nodes
                .get(&child_id)
                .cloned()
                .ok_or(StructuralError::UnknownNode { node_id: child_id })?;
            self.nodes
                .insert(child_id, child.with_parent(Some(parent_id)));
        }

        Ok(())
    }

    pub fn move_node(
        &mut self,
        node_id: TreeNodeId,
        new_parent_id: TreeNodeId,
        new_index: Option<usize>,
    ) -> Result<(), StructuralError> {
        let node = self
            .nodes
            .get(&node_id)
            .cloned()
            .ok_or(StructuralError::UnknownNode { node_id })?;
        let old_parent_id = node
            .parent_id
            .ok_or(StructuralError::CannotMoveRoot { node_id })?;

        let old_parent =
            self.nodes
                .get(&old_parent_id)
                .cloned()
                .ok_or(StructuralError::UnknownNode {
                    node_id: old_parent_id,
                })?;
        let old_child_ids = old_parent
            .child_ids
            .iter()
            .copied()
            .filter(|child_id| *child_id != node_id)
            .collect::<Vec<_>>();
        self.nodes
            .insert(old_parent_id, old_parent.with_children(old_child_ids));

        let new_parent =
            self.nodes
                .get(&new_parent_id)
                .cloned()
                .ok_or(StructuralError::UnknownNode {
                    node_id: new_parent_id,
                })?;
        let mut new_child_ids = new_parent
            .child_ids
            .iter()
            .copied()
            .filter(|child_id| *child_id != node_id)
            .collect::<Vec<_>>();
        let insertion_index = new_index.unwrap_or(new_child_ids.len());
        if insertion_index > new_child_ids.len() {
            return Err(StructuralError::InvalidChildInsertionIndex {
                parent_id: new_parent_id,
                index: insertion_index,
            });
        }
        new_child_ids.insert(insertion_index, node_id);

        self.nodes
            .insert(new_parent_id, new_parent.with_children(new_child_ids));
        self.nodes
            .insert(node_id, node.with_parent(Some(new_parent_id)));
        Ok(())
    }

    pub fn insert_node(
        &mut self,
        node: StructuralNode,
        parent_id: TreeNodeId,
        index: Option<usize>,
    ) -> Result<(), StructuralError> {
        let node_id = node.node_id;
        self.nodes
            .insert(node_id, node.with_parent(Some(parent_id)));
        self.attach_child(parent_id, node_id, index)
    }

    pub fn remove_node(&mut self, node_id: TreeNodeId) -> Result<Vec<TreeNodeId>, StructuralError> {
        if Some(node_id) == self.root_node_id {
            return Err(StructuralError::CannotRemoveRoot { node_id });
        }

        let node = self
            .nodes
            .get(&node_id)
            .cloned()
            .ok_or(StructuralError::UnknownNode { node_id })?;
        let parent_id = node
            .parent_id
            .ok_or(StructuralError::CannotRemoveRoot { node_id })?;
        let parent = self
            .nodes
            .get(&parent_id)
            .cloned()
            .ok_or(StructuralError::UnknownNode { node_id: parent_id })?;
        let child_ids = parent
            .child_ids
            .iter()
            .copied()
            .filter(|child_id| *child_id != node_id)
            .collect::<Vec<_>>();
        self.nodes
            .insert(parent_id, parent.with_children(child_ids));

        let removed_ids = collect_subtree_ids(node_id, &self.nodes)?;
        for removed_id in &removed_ids {
            self.nodes.remove(removed_id);
        }

        Ok(removed_ids)
    }

    pub fn build(
        &self,
        snapshot_id: StructuralSnapshotId,
    ) -> Result<StructuralSnapshot, StructuralError> {
        let root_node_id = self.root_node_id.ok_or(StructuralError::MissingRoot {
            snapshot_id,
            root_node_id: TreeNodeId(0),
        })?;
        StructuralSnapshot::create(snapshot_id, root_node_id, self.nodes.values().cloned())
    }
}

fn collect_subtree_ids(
    root_node_id: TreeNodeId,
    nodes: &BTreeMap<TreeNodeId, StructuralNode>,
) -> Result<Vec<TreeNodeId>, StructuralError> {
    let mut stack = vec![root_node_id];
    let mut removed_ids = Vec::new();
    while let Some(node_id) = stack.pop() {
        let node = nodes
            .get(&node_id)
            .ok_or(StructuralError::UnknownNode { node_id })?;
        removed_ids.push(node_id);
        for child_id in node.child_ids.iter().rev() {
            stack.push(*child_id);
        }
    }
    Ok(removed_ids)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(
        node_id: u64,
        kind: StructuralNodeKind,
        symbol: &str,
        parent_id: Option<u64>,
        child_ids: &[u64],
    ) -> StructuralNode {
        StructuralNode {
            node_id: TreeNodeId(node_id),
            kind,
            symbol: symbol.to_string(),
            parent_id: parent_id.map(TreeNodeId),
            child_ids: child_ids.iter().copied().map(TreeNodeId).collect(),
            formula_artifact_id: None,
            bind_artifact_id: None,
            constant_value: None,
        }
    }

    #[test]
    fn structural_snapshot_builds_projection_paths_and_relative_context() {
        let root = node(1, StructuralNodeKind::Root, "Root", None, &[2, 3]);
        let child = node(2, StructuralNodeKind::Container, "Branch", Some(1), &[4]);
        let sibling = node(3, StructuralNodeKind::Calculation, "Sibling", Some(1), &[]);
        let grandchild = node(4, StructuralNodeKind::Calculation, "Leaf", Some(2), &[]);

        let snapshot = StructuralSnapshot::create(
            StructuralSnapshotId(1),
            TreeNodeId(1),
            [root, child, sibling, grandchild],
        )
        .unwrap();

        assert_eq!(
            snapshot.get_projection_path(TreeNodeId(4)).unwrap(),
            "Root/Branch/Leaf"
        );
        assert_eq!(
            snapshot.try_resolve_projection_path("Root/Branch/Leaf"),
            Some(TreeNodeId(4))
        );

        let context = snapshot.describe_relative_context(TreeNodeId(4)).unwrap();
        assert_eq!(context.parent_id, Some(TreeNodeId(2)));
        assert_eq!(context.ancestor_ids, vec![TreeNodeId(2), TreeNodeId(1)]);
        assert_eq!(context.sibling_index, Some(0));
    }

    #[test]
    fn structural_edit_move_and_rename_requires_rebind() {
        let root = node(1, StructuralNodeKind::Root, "Root", None, &[2, 3]);
        let branch = node(2, StructuralNodeKind::Container, "Branch", Some(1), &[4]);
        let sibling = node(3, StructuralNodeKind::Container, "Sibling", Some(1), &[]);
        let leaf = node(4, StructuralNodeKind::Calculation, "Leaf", Some(2), &[]);

        let snapshot = StructuralSnapshot::create(
            StructuralSnapshotId(1),
            TreeNodeId(1),
            [root, branch, sibling, leaf],
        )
        .unwrap();

        let moved = snapshot
            .apply_edit(
                StructuralSnapshotId(2),
                StructuralEdit::MoveNode {
                    node_id: TreeNodeId(4),
                    new_parent_id: TreeNodeId(3),
                    new_index: None,
                },
            )
            .unwrap();
        assert_eq!(moved.impact, StructuralEditImpact::RebindRequired);
        assert_eq!(
            moved.snapshot.get_projection_path(TreeNodeId(4)).unwrap(),
            "Root/Sibling/Leaf"
        );

        let renamed = moved
            .snapshot
            .apply_edit(
                StructuralSnapshotId(3),
                StructuralEdit::RenameNode {
                    node_id: TreeNodeId(4),
                    new_symbol: "LeafRenamed".to_string(),
                },
            )
            .unwrap();
        assert_eq!(renamed.impact, StructuralEditImpact::RebindRequired);
        assert_eq!(
            renamed.snapshot.get_projection_path(TreeNodeId(4)).unwrap(),
            "Root/Sibling/LeafRenamed"
        );
    }

    #[test]
    fn structural_edit_remove_subtree_reports_removal() {
        let root = node(1, StructuralNodeKind::Root, "Root", None, &[2]);
        let branch = node(2, StructuralNodeKind::Container, "Branch", Some(1), &[3]);
        let leaf = node(3, StructuralNodeKind::Constant, "Leaf", Some(2), &[])
            .with_constant_value(Some("5".to_string()));

        let snapshot = StructuralSnapshot::create(
            StructuralSnapshotId(1),
            TreeNodeId(1),
            [root, branch, leaf],
        )
        .unwrap();

        let outcome = snapshot
            .apply_edit(
                StructuralSnapshotId(2),
                StructuralEdit::RemoveNode {
                    node_id: TreeNodeId(2),
                },
            )
            .unwrap();

        assert_eq!(outcome.impact, StructuralEditImpact::Removal);
        assert_eq!(
            outcome.affected_node_ids,
            vec![TreeNodeId(2), TreeNodeId(3)]
        );
        assert!(outcome.snapshot.try_get_node(TreeNodeId(2)).is_none());
        assert!(outcome.snapshot.try_get_node(TreeNodeId(3)).is_none());
    }
}
