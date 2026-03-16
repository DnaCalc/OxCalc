#![forbid(unsafe_code)]

//! Structural snapshot kernel lane.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{Display, Formatter};

use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TreeNodeId(pub u64);

impl Display for TreeNodeId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "node:{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StructuralSnapshotId(pub u64);

impl Display for StructuralSnapshotId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "snapshot:{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FormulaArtifactId(pub String);

impl Display for FormulaArtifactId {
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
        let path_index = build_path_index(root_node_id, &node_map);

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
    root_node_id: TreeNodeId,
    nodes: &BTreeMap<TreeNodeId, StructuralNode>,
) -> BTreeMap<String, TreeNodeId> {
    let mut index = BTreeMap::new();
    let root = nodes
        .get(&root_node_id)
        .expect("validated snapshots contain the root");
    let mut stack = vec![(root_node_id, root.symbol.clone())];

    while let Some((node_id, path)) = stack.pop() {
        index.insert(path.clone(), node_id);
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

    index
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn structural_snapshot_builds_projection_paths() {
        let root = StructuralNode {
            node_id: TreeNodeId(1),
            kind: StructuralNodeKind::Root,
            symbol: "Root".to_string(),
            parent_id: None,
            child_ids: vec![TreeNodeId(2)],
            formula_artifact_id: None,
        };
        let child = StructuralNode {
            node_id: TreeNodeId(2),
            kind: StructuralNodeKind::Calculation,
            symbol: "Child".to_string(),
            parent_id: Some(TreeNodeId(1)),
            child_ids: vec![],
            formula_artifact_id: None,
        };

        let snapshot =
            StructuralSnapshot::create(StructuralSnapshotId(1), TreeNodeId(1), [root, child])
                .unwrap();

        assert_eq!(
            snapshot.get_projection_path(TreeNodeId(2)).unwrap(),
            "Root/Child"
        );
        assert_eq!(
            snapshot.try_resolve_projection_path("Root/Child"),
            Some(TreeNodeId(2))
        );
    }
}
