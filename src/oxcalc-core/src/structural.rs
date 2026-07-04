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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum StructuralNodeKind {
    Root,
    Container,
    Calculation,
    Constant,
}

/// Authored document-vocabulary position of a node, orthogonal to both
/// [`StructuralNodeKind`] (the derived calc-DAG role) and [`NodeBacking`] (the
/// owned sub-model). A role is an authored fact, set at node creation or by an
/// explicit role edit and never derived from formula text. `None` means "plain
/// tree node" and is the default for everything TreeCalc builds today.
///
/// Marked `#[non_exhaustive]`: future document-artifact roles (e.g. `ChartSheet`)
/// extend this enum without disturbing existing match arms.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum NodeRole {
    /// The workbook document root. Legal only on the snapshot root.
    Workbook,
    /// A sheet within a workbook. Legal only as a direct child of a
    /// `Workbook`-role root.
    Sheet,
    // later: ChartSheet, and other document-artifact roles
}

/// A sheet name folded to its case-insensitive canonical form (D1 §1/§2).
///
/// Excel compares sheet names case-insensitively while preserving the authored
/// capitalization for display. The display capitalization lives on
/// [`StructuralNode::symbol`] untouched; all sheet lookups, uniqueness checks,
/// and identity keys use this folded form. The newtype makes the folded/display
/// distinction unforgeable in the type system: a `NormalizedSheetName` can only
/// be produced by [`NormalizedSheetName::from_symbol`], so no caller can key the
/// registry on a raw, unfolded string by accident.
///
/// The map key ordering of [`NormalizedSheetName`] (derived `Ord`) is the folded
/// byte order, distinct from the authored `child_ids` enumeration order —
/// `sheet_index` is keyed for lookup, `sheet_nodes()` walks `child_ids` for
/// order.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct NormalizedSheetName(String);

impl NormalizedSheetName {
    /// Folds a display symbol to its case-insensitive canonical form.
    ///
    /// **Fold rule (documented per bead):** Unicode simple case fold approximated
    /// by `char::to_lowercase`, the same mechanism the tree reference resolver
    /// already uses for case-insensitive character matching
    /// (`treecalc_runner.rs`). `to_lowercase` performs the full Unicode
    /// lowercase mapping, which coincides with simple case fold for the
    /// overwhelming majority of sheet-name characters across the common
    /// bicameral scripts (Latin, Greek, Cyrillic). Known divergences from a
    /// true case fold are accepted: German ß stays ß under `to_lowercase`
    /// (a full fold maps it to "ss", so `ß`/`SS` do *not* collide here),
    /// capital sigma always lowers to σ (never final-sigma ς, so `ς`/`σ`
    /// twins do not collide), and the Turkish dotless-i locale rule is not
    /// applied. A dedicated Unicode fold table is deliberately not pulled in:
    /// no case-fold crate is already
    /// a dependency of `oxcalc-core` (only `serde`/`thiserror`/the sibling OxFml
    /// and OxFunc cores), and adding one for the residual divergence is not
    /// warranted for sheet identity. If a fuller fold is ever required, this is
    /// the single choke point to upgrade.
    ///
    /// This is the shared fold entry point: general case-insensitive sibling
    /// uniqueness (`calc-uanv`) is expected to adopt this same function so the
    /// two lanes cannot drift. It is factored as a free-standing associated
    /// function for exactly that reason.
    #[must_use]
    pub fn from_symbol(symbol: &str) -> Self {
        Self(fold_name_case_insensitive(symbol))
    }

    /// The folded canonical form as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for NormalizedSheetName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Case-insensitive name fold shared across the structural layer.
///
/// This is the single fold function the sheet registry (this bead, `calc-5kqg.10`)
/// uses for [`NormalizedSheetName`]; general case-insensitive sibling uniqueness
/// (`calc-uanv`, currently unimplemented) should adopt it too, rather than
/// duplicating the fold rule. See [`NormalizedSheetName::from_symbol`] for the
/// exact rule chosen and why.
fn fold_name_case_insensitive(symbol: &str) -> String {
    let mut folded = String::with_capacity(symbol.len());
    for ch in symbol.chars() {
        for lowered in ch.to_lowercase() {
            folded.push(lowered);
        }
    }
    folded
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructuralNode {
    pub node_id: TreeNodeId,
    pub kind: StructuralNodeKind,
    pub symbol: String,
    pub parent_id: Option<TreeNodeId>,
    pub child_ids: Vec<TreeNodeId>,
    /// Authored document-vocabulary role; `None` for plain tree nodes.
    /// Serde-additive: pre-role snapshots load with `role: None`.
    #[serde(default)]
    pub role: Option<NodeRole>,
    /// Meta membership: `true` marks this node (and, transitively, its
    /// subtree) as a namespace-excluded property carrier. Meta-ness is a
    /// structural fact — it lives on the node it describes, feeds the
    /// structural snapshot id directly, and is never held in a side set.
    /// Serde-additive: pre-promotion snapshots load with `is_meta: false`.
    #[serde(default)]
    pub is_meta: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructuralTableShape {
    pub table_id: String,
    pub table_name: String,
    pub virtual_anchor_identity: String,
    pub row_membership_version: String,
    pub row_order_version: String,
    pub column_identity_version: String,
    pub body_shape_identity: String,
    pub totals_shape_identity: String,
    pub header_row_present: bool,
    pub totals_row_present: bool,
    pub row_count: usize,
    pub column_count: usize,
}

/// Identity/version facts for a grid backing attached to a sheet node.
///
/// Like [`StructuralTableShape`], this carries only identity and version facts,
/// never cells: the cells live in the node's grid backing store
/// (`GridOptimizedSheet`), addressed `(TreeNodeId, ExcelGridCellAddress)`. The
/// structural lane stays `O(nodes)`, with nodes far fewer than cells.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructuralGridShape {
    pub grid_id: String,
    pub sheet_name: String,
    pub bounds_identity: String,
    pub cell_population_version: String,
    pub axis_state_version: String,
    pub overlay_set_version: String,
    pub merged_region_version: String,
}

/// The content backing carried by a node, orthogonal to [`StructuralNodeKind`].
///
/// A node's *kind* describes its role in the calc DAG; its *backing* (if any)
/// describes a structured sub-model it owns. A table node carries a
/// [`NodeBacking::Table`]; a sheet node carries a [`NodeBacking::Grid`]. Future
/// backings (pivot, chart) extend this enum without disturbing node kind.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeBacking {
    Table(StructuralTableShape),
    Grid(StructuralGridShape),
}

impl NodeBacking {
    #[must_use]
    pub fn as_table(&self) -> Option<&StructuralTableShape> {
        match self {
            Self::Table(shape) => Some(shape),
            Self::Grid(_) => None,
        }
    }

    #[must_use]
    pub fn as_grid(&self) -> Option<&StructuralGridShape> {
        match self {
            Self::Grid(shape) => Some(shape),
            Self::Table(_) => None,
        }
    }
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
    pub fn with_role(mut self, role: Option<NodeRole>) -> Self {
        self.role = role;
        self
    }

    #[must_use]
    pub fn with_meta(mut self, is_meta: bool) -> Self {
        self.is_meta = is_meta;
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
    InsertNode {
        node: StructuralNode,
        parent_id: TreeNodeId,
        index: Option<usize>,
    },
    RemoveNode {
        node_id: TreeNodeId,
    },
    SetTableShape {
        node_id: TreeNodeId,
        table_shape: StructuralTableShape,
    },
    ClearTableShape {
        node_id: TreeNodeId,
    },
    SetGridShape {
        node_id: TreeNodeId,
        grid_shape: StructuralGridShape,
    },
    ClearGridShape {
        node_id: TreeNodeId,
    },
    /// Set (or clear) a node's meta membership. Meta-ness is a structural
    /// fact (`is_meta` on the node), so a membership change is an ordinary
    /// structural edit that mints a new snapshot id and is validated by the
    /// `#`-prefix, meta-child-inheritance, and meta/role-exclusion rules.
    SetNodeMeta {
        node_id: TreeNodeId,
        is_meta: bool,
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
    #[error("node backing references missing node {node_id}")]
    NodeBackingMissingNode { node_id: TreeNodeId },
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
    #[error("node id {node_id} already exists in snapshot {snapshot_id}")]
    DuplicateNodeId {
        snapshot_id: StructuralSnapshotId,
        node_id: TreeNodeId,
    },
    #[error("cannot remove the structural root {node_id}")]
    CannotRemoveRoot { node_id: TreeNodeId },
    #[error("cannot move the structural root {node_id}")]
    CannotMoveRoot { node_id: TreeNodeId },
    #[error("child insertion index {index} is out of range for parent {parent_id}")]
    InvalidChildInsertionIndex { parent_id: TreeNodeId, index: usize },
    #[error("unknown node {node_id}")]
    UnknownNode { node_id: TreeNodeId },
    #[error("node {node_id} carries the Workbook role but is not the snapshot root")]
    WorkbookRoleRequiresRoot { node_id: TreeNodeId },
    #[error(
        "node {node_id} carries the Sheet role but is not a direct child of a Workbook-role root"
    )]
    SheetRoleRequiresWorkbookParent { node_id: TreeNodeId },
    #[error("sheet name '{normalized}' is duplicated across sibling nodes {node_ids}")]
    DuplicateSheetName {
        normalized: String,
        node_ids: String,
    },
    #[error("meta node {node_id} may not carry a document role")]
    MetaNodeCannotCarryRole { node_id: TreeNodeId },
    #[error(
        "node {node_id} symbol '{symbol}' uses the reserved '#' meta prefix but is not a meta node"
    )]
    MetaPrefixReservedForMetaNodes { node_id: TreeNodeId, symbol: String },
    #[error(
        "node {node_id} is a child of meta node {parent_id} but is not itself meta: a child of a meta node must itself be meta"
    )]
    MetaChildRequiresMetaParent {
        node_id: TreeNodeId,
        parent_id: TreeNodeId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "StructuralSnapshotWire", into = "StructuralSnapshotWire")]
pub struct StructuralSnapshot {
    snapshot_id: StructuralSnapshotId,
    root_node_id: TreeNodeId,
    nodes: BTreeMap<TreeNodeId, StructuralNode>,
    node_backings: BTreeMap<TreeNodeId, NodeBacking>,
    path_index: BTreeMap<String, TreeNodeId>,
    /// Derived sheet-name → node registry (D1 §2). Built in the same validated
    /// pass as `path_index` from the Sheet-role children of the root, keyed on
    /// the normalized (case-folded) sheet name. **Never serialized** — it is
    /// rebuilt on deserialize by the `From<Wire>` constructor, so it cannot
    /// drift from the tree. Lookup order is the folded key order; enumeration
    /// order comes from [`Self::sheet_nodes`], which walks `child_ids`.
    sheet_index: BTreeMap<NormalizedSheetName, TreeNodeId>,
}

/// Serde wire form for [`StructuralSnapshot`].
///
/// Carries the authored structure only: `snapshot_id`, `root_node_id`,
/// `nodes`, and `node_backings`. Both derived indexes (`path_index` and
/// `sheet_index`) stay off the wire and are rebuilt on load through the
/// validated constructor (D1 §8 item 3), so a loaded snapshot cannot carry an
/// index that disagrees with its tree.
///
/// The pre-`NodeBacking` `table_shapes` legacy fold is gone (D1 §8 item 2):
/// snapshots serialized before the `NodeBacking` migration are no longer
/// readable; current-generation (`node_backings` form) snapshots remain
/// readable. This is the deliberate readability decision recorded in D1 §8.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct StructuralSnapshotWire {
    snapshot_id: StructuralSnapshotId,
    root_node_id: TreeNodeId,
    nodes: BTreeMap<TreeNodeId, StructuralNode>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    node_backings: BTreeMap<TreeNodeId, NodeBacking>,
}

impl From<StructuralSnapshot> for StructuralSnapshotWire {
    fn from(snapshot: StructuralSnapshot) -> Self {
        Self {
            snapshot_id: snapshot.snapshot_id,
            root_node_id: snapshot.root_node_id,
            nodes: snapshot.nodes,
            node_backings: snapshot.node_backings,
        }
    }
}

impl TryFrom<StructuralSnapshotWire> for StructuralSnapshot {
    type Error = StructuralError;

    /// Deserialization routes through the validated constructor
    /// ([`StructuralSnapshot::create_with_node_backings`]), so a wire payload
    /// that violates a structural, role, or meta invariant fails to load with a
    /// typed [`StructuralError`] rather than silently producing an invalid
    /// snapshot (D1 §8 item 3; closes the deserialize-bypasses-`validate()` gap
    /// noted in R2.1). Both derived indexes (`path_index`, `sheet_index`) are
    /// rebuilt by that same pass from the deserialized tree.
    fn try_from(wire: StructuralSnapshotWire) -> Result<Self, Self::Error> {
        Self::create_with_node_backings(
            wire.snapshot_id,
            wire.root_node_id,
            wire.nodes.into_values(),
            wire.node_backings,
        )
    }
}

impl StructuralSnapshot {
    pub fn create(
        snapshot_id: StructuralSnapshotId,
        root_node_id: TreeNodeId,
        nodes: impl IntoIterator<Item = StructuralNode>,
    ) -> Result<Self, StructuralError> {
        Self::create_with_node_backings(snapshot_id, root_node_id, nodes, BTreeMap::new())
    }

    pub fn create_with_node_backings(
        snapshot_id: StructuralSnapshotId,
        root_node_id: TreeNodeId,
        nodes: impl IntoIterator<Item = StructuralNode>,
        node_backings: BTreeMap<TreeNodeId, NodeBacking>,
    ) -> Result<Self, StructuralError> {
        let node_map = nodes
            .into_iter()
            .map(|node| (node.node_id, node))
            .collect::<BTreeMap<_, _>>();
        validate(snapshot_id, root_node_id, &node_map, &node_backings)?;
        let path_index = build_path_index(snapshot_id, root_node_id, &node_map)?;
        let sheet_index = build_sheet_index(root_node_id, &node_map)?;

        Ok(Self {
            snapshot_id,
            root_node_id,
            nodes: node_map,
            node_backings,
            path_index,
            sheet_index,
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

    #[must_use]
    pub fn node_backings(&self) -> &BTreeMap<TreeNodeId, NodeBacking> {
        &self.node_backings
    }

    /// The set of directly-meta nodes, derived from `is_meta` on the nodes.
    ///
    /// Meta membership is a structural fact carried on each node, not a stored
    /// side set. Name resolution consumes this derived set exactly as it
    /// consumed the retired `meta_node_ids` side set: it holds only the
    /// *directly* meta nodes, and `is_meta_effective` walks ancestors to cover
    /// inherited meta subtrees (an inherited fact validation already enforces).
    #[must_use]
    pub fn meta_node_ids(&self) -> BTreeSet<TreeNodeId> {
        self.nodes
            .iter()
            .filter_map(|(node_id, node)| node.is_meta.then_some(*node_id))
            .collect()
    }

    #[must_use]
    pub fn backing_for(&self, node_id: TreeNodeId) -> Option<&NodeBacking> {
        self.node_backings.get(&node_id)
    }

    #[must_use]
    pub fn table_shape_for(&self, node_id: TreeNodeId) -> Option<&StructuralTableShape> {
        self.node_backings
            .get(&node_id)
            .and_then(NodeBacking::as_table)
    }

    #[must_use]
    pub fn grid_shape_for(&self, node_id: TreeNodeId) -> Option<&StructuralGridShape> {
        self.node_backings
            .get(&node_id)
            .and_then(NodeBacking::as_grid)
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

    /// Resolves a sheet name to its node id via the derived sheet registry
    /// (D1 §1 contract: `sheet_index[NormalizedSheetName] -> TreeNodeId`).
    ///
    /// The lookup is case-insensitive: the argument is folded to its
    /// [`NormalizedSheetName`] before the registry is consulted, so `"Sheet1"`,
    /// `"SHEET1"`, and `"sheet1"` all resolve to the same node. Returns `None`
    /// if no Sheet-role sibling carries that (folded) name. The registry follows
    /// renames for free — a rename changes the key, not the node id — because
    /// [`TreeNodeId`] is the stable identity.
    #[must_use]
    pub fn try_resolve_sheet_name(&self, sheet_name: &str) -> Option<TreeNodeId> {
        self.sheet_index
            .get(&NormalizedSheetName::from_symbol(sheet_name))
            .copied()
    }

    /// The derived sheet registry, keyed on the normalized (case-folded) name.
    ///
    /// Key order is folded byte order (map ordering), which is *not* the
    /// authored sheet order — use [`Self::sheet_nodes`] for enumeration in
    /// `child_ids` order.
    #[must_use]
    pub fn sheet_index(&self) -> &BTreeMap<NormalizedSheetName, TreeNodeId> {
        &self.sheet_index
    }

    /// Enumerates the workbook's sheet nodes in authored order (D1 §2/§3).
    ///
    /// Sheet order is the root's `child_ids` filtered to `Sheet`-role children;
    /// there is no second order vector. Non-sheet root children interleave
    /// freely and are skipped here. When the root is not a workbook (no
    /// Sheet-role children exist), this yields an empty vector.
    #[must_use]
    pub fn sheet_nodes(&self) -> Vec<TreeNodeId> {
        let Some(root) = self.nodes.get(&self.root_node_id) else {
            return Vec::new();
        };
        root.child_ids
            .iter()
            .copied()
            .filter(|child_id| {
                self.nodes
                    .get(child_id)
                    .is_some_and(|child| child.role == Some(NodeRole::Sheet))
            })
            .collect()
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
            StructuralEdit::SetTableShape {
                node_id,
                table_shape,
            } => {
                builder.set_table_shape(node_id, table_shape.clone())?;
                (
                    StructuralEditImpact::RebindRequired,
                    vec![node_id],
                    vec![format!(
                        "table_shape_set:{node_id}:{}",
                        table_shape.table_id
                    )],
                )
            }
            StructuralEdit::ClearTableShape { node_id } => {
                builder.clear_table_shape(node_id)?;
                (
                    StructuralEditImpact::RebindRequired,
                    vec![node_id],
                    vec![format!("table_shape_cleared:{node_id}")],
                )
            }
            StructuralEdit::SetGridShape {
                node_id,
                grid_shape,
            } => {
                let grid_id = grid_shape.grid_id.clone();
                builder.set_grid_shape(node_id, grid_shape)?;
                (
                    StructuralEditImpact::RebindRequired,
                    vec![node_id],
                    vec![format!("grid_shape_set:{node_id}:{grid_id}")],
                )
            }
            StructuralEdit::ClearGridShape { node_id } => {
                builder.clear_grid_shape(node_id)?;
                (
                    StructuralEditImpact::RebindRequired,
                    vec![node_id],
                    vec![format!("grid_shape_cleared:{node_id}")],
                )
            }
            StructuralEdit::SetNodeMeta { node_id, is_meta } => {
                builder.set_node_meta(node_id, is_meta)?;
                (
                    StructuralEditImpact::RebindRequired,
                    vec![node_id],
                    vec![format!("node_meta_set:{node_id}:{is_meta}")],
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
    node_backings: &BTreeMap<TreeNodeId, NodeBacking>,
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

    for node_id in node_backings.keys() {
        if !nodes.contains_key(node_id) {
            return Err(StructuralError::NodeBackingMissingNode { node_id: *node_id });
        }
    }

    // Meta/role disjointness is checked before role placement so that a node
    // that is both meta and role-bearing reports the more fundamental
    // MetaNodeCannotCarryRole rather than a role-placement error.
    validate_meta(nodes)?;
    validate_roles(root_node_id, nodes)?;

    Ok(())
}

/// Enforces the role placement invariants of D1 §1. Roles are authored
/// document facts, so these rules hold for every constructor and every
/// `apply_edit` product that funnels through [`validate`]:
///
/// 1. `NodeRole::Workbook` is legal only on the snapshot root.
/// 2. `NodeRole::Sheet` is legal only on a direct child of a `Workbook`-role
///    root. A `MoveNode` that would carry a Sheet-role node under a
///    non-Workbook parent is rejected by this same rule, reached via the normal
///    build-then-validate path (no special-case move code).
///
/// Sheet-sibling name uniqueness ([`StructuralError::DuplicateSheetName`], D1 §1
/// rule 3) lands with the sheet registry in R2.3, and the meta/role exclusion
/// ([`StructuralError::MetaNodeCannotCarryRole`], §1) lands with the `is_meta`
/// promotion in R2.2; both variants are defined here as the role-invariant
/// vocabulary this bead introduces.
fn validate_roles(
    root_node_id: TreeNodeId,
    nodes: &BTreeMap<TreeNodeId, StructuralNode>,
) -> Result<(), StructuralError> {
    let root_is_workbook = nodes
        .get(&root_node_id)
        .is_some_and(|root| root.role == Some(NodeRole::Workbook));

    for (node_id, node) in nodes {
        match node.role {
            Some(NodeRole::Workbook) => {
                if *node_id != root_node_id || node.parent_id.is_some() {
                    return Err(StructuralError::WorkbookRoleRequiresRoot { node_id: *node_id });
                }
            }
            Some(NodeRole::Sheet) => {
                let parent_is_workbook_root = node
                    .parent_id
                    .is_some_and(|parent_id| parent_id == root_node_id && root_is_workbook);
                if !parent_is_workbook_root {
                    return Err(StructuralError::SheetRoleRequiresWorkbookParent {
                        node_id: *node_id,
                    });
                }
            }
            None => {}
        }
    }

    Ok(())
}

/// Enforces the meta-membership invariants of D1 §4. Meta-ness is now a
/// structural fact (`StructuralNode::is_meta`), so these rules hold for every
/// constructor and every `apply_edit` product that funnels through
/// [`validate`]:
///
/// 1. The `#` symbol prefix is reserved for meta nodes: a node whose symbol
///    begins with `#` must be meta ([`StructuralError::MetaPrefixReservedForMetaNodes`]).
///    Property paths can never collide with user symbols.
/// 2. Meta-child inheritance: a child of a meta node must itself be meta
///    ([`StructuralError::MetaChildRequiresMetaParent`]). Property subtrees are
///    opaque to name resolution as a unit.
/// 3. A meta node may not carry a document role
///    ([`StructuralError::MetaNodeCannotCarryRole`]): roles describe document
///    structure, meta subtrees are property carriers, and the two are disjoint.
fn validate_meta(nodes: &BTreeMap<TreeNodeId, StructuralNode>) -> Result<(), StructuralError> {
    for (node_id, node) in nodes {
        if node.symbol.starts_with('#') && !node.is_meta {
            return Err(StructuralError::MetaPrefixReservedForMetaNodes {
                node_id: *node_id,
                symbol: node.symbol.clone(),
            });
        }

        if node.is_meta && node.role.is_some() {
            return Err(StructuralError::MetaNodeCannotCarryRole { node_id: *node_id });
        }

        // Meta-child inheritance: a child of a meta node must itself be meta.
        // A meta node may still attach under a non-meta parent (that is how a
        // property group hangs off an ordinary node); it is *descendants* of a
        // meta node that inherit meta-ness, not its ancestors.
        if let Some(parent_id) = node.parent_id {
            let parent_is_meta = nodes.get(&parent_id).is_some_and(|parent| parent.is_meta);
            if parent_is_meta && !node.is_meta {
                return Err(StructuralError::MetaChildRequiresMetaParent {
                    node_id: *node_id,
                    parent_id,
                });
            }
        }
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

/// Builds the derived sheet registry (D1 §2), enforcing case-insensitive
/// Sheet-sibling name uniqueness.
///
/// Walks the root's `child_ids` (authored order) filtered to `Sheet`-role
/// children, folding each display symbol to a [`NormalizedSheetName`]. Two
/// Sheet-role siblings whose names fold to the same normalized form are a
/// [`StructuralError::DuplicateSheetName`] — this is the case-twin rejection
/// (`Dup`/`dup`) the Excel sheet-identity contract requires. Only sheets are
/// checked here: general case-insensitive sibling uniqueness across all node
/// kinds is `calc-uanv`'s scope and shares the same fold function.
///
/// Runs in the same validated pass as [`build_path_index`], so it sees only
/// structurally sound nodes (root present, `child_ids` resolve).
fn build_sheet_index(
    root_node_id: TreeNodeId,
    nodes: &BTreeMap<TreeNodeId, StructuralNode>,
) -> Result<BTreeMap<NormalizedSheetName, TreeNodeId>, StructuralError> {
    let mut index = BTreeMap::new();
    let root = nodes
        .get(&root_node_id)
        .expect("validated snapshots contain the root");

    for child_id in &root.child_ids {
        let child = nodes
            .get(child_id)
            .expect("validated snapshots contain child nodes");
        if child.role != Some(NodeRole::Sheet) {
            continue;
        }
        let normalized = NormalizedSheetName::from_symbol(&child.symbol);
        if let Some(existing) = index.insert(normalized.clone(), *child_id) {
            return Err(StructuralError::DuplicateSheetName {
                normalized: normalized.as_str().to_string(),
                node_ids: format!("{existing}, {child_id}"),
            });
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
    node_backings: BTreeMap<TreeNodeId, NodeBacking>,
    root_node_id: Option<TreeNodeId>,
}

impl StructuralSnapshotBuilder {
    #[must_use]
    pub fn new(predecessor: Option<&StructuralSnapshot>) -> Self {
        match predecessor {
            Some(snapshot) => Self {
                nodes: snapshot.nodes.clone(),
                node_backings: snapshot.node_backings.clone(),
                root_node_id: Some(snapshot.root_node_id),
            },
            None => Self {
                nodes: BTreeMap::new(),
                node_backings: BTreeMap::new(),
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

    pub fn set_node_meta(
        &mut self,
        node_id: TreeNodeId,
        is_meta: bool,
    ) -> Result<(), StructuralError> {
        let node = self
            .nodes
            .get(&node_id)
            .cloned()
            .ok_or(StructuralError::UnknownNode { node_id })?;
        self.nodes.insert(node_id, node.with_meta(is_meta));
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
            self.node_backings.remove(removed_id);
        }

        Ok(removed_ids)
    }

    pub fn set_backing(
        &mut self,
        node_id: TreeNodeId,
        backing: NodeBacking,
    ) -> Result<(), StructuralError> {
        if !self.nodes.contains_key(&node_id) {
            return Err(StructuralError::UnknownNode { node_id });
        }
        self.node_backings.insert(node_id, backing);
        Ok(())
    }

    pub fn clear_backing(&mut self, node_id: TreeNodeId) -> Result<(), StructuralError> {
        if !self.nodes.contains_key(&node_id) {
            return Err(StructuralError::UnknownNode { node_id });
        }
        self.node_backings.remove(&node_id);
        Ok(())
    }

    pub fn set_table_shape(
        &mut self,
        node_id: TreeNodeId,
        table_shape: StructuralTableShape,
    ) -> Result<(), StructuralError> {
        self.set_backing(node_id, NodeBacking::Table(table_shape))
    }

    pub fn clear_table_shape(&mut self, node_id: TreeNodeId) -> Result<(), StructuralError> {
        self.clear_backing(node_id)
    }

    pub fn set_grid_shape(
        &mut self,
        node_id: TreeNodeId,
        grid_shape: StructuralGridShape,
    ) -> Result<(), StructuralError> {
        self.set_backing(node_id, NodeBacking::Grid(grid_shape))
    }

    pub fn clear_grid_shape(&mut self, node_id: TreeNodeId) -> Result<(), StructuralError> {
        self.clear_backing(node_id)
    }

    pub fn build(
        &self,
        snapshot_id: StructuralSnapshotId,
    ) -> Result<StructuralSnapshot, StructuralError> {
        let root_node_id = self.root_node_id.ok_or(StructuralError::MissingRoot {
            snapshot_id,
            root_node_id: TreeNodeId(0),
        })?;
        StructuralSnapshot::create_with_node_backings(
            snapshot_id,
            root_node_id,
            self.nodes.values().cloned(),
            self.node_backings.clone(),
        )
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
            role: None,
            is_meta: false,
        }
    }

    fn table_shape(table_id: &str) -> StructuralTableShape {
        StructuralTableShape {
            table_id: table_id.to_string(),
            table_name: "Table1".to_string(),
            virtual_anchor_identity: "Book1:Sheet1:1:1".to_string(),
            row_membership_version: "rows:v1".to_string(),
            row_order_version: "row-order:v1".to_string(),
            column_identity_version: "columns:v1".to_string(),
            body_shape_identity: "body:v1".to_string(),
            totals_shape_identity: "totals:v1".to_string(),
            header_row_present: true,
            totals_row_present: false,
            row_count: 2,
            column_count: 1,
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
        let leaf = node(3, StructuralNodeKind::Constant, "Leaf", Some(2), &[]);

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

    #[test]
    fn structural_table_shape_edits_are_snapshot_facts() {
        let root = node(1, StructuralNodeKind::Root, "Root", None, &[2]);
        let table_node = node(2, StructuralNodeKind::Container, "Sales", Some(1), &[]);

        let snapshot =
            StructuralSnapshot::create(StructuralSnapshotId(1), TreeNodeId(1), [root, table_node])
                .unwrap();

        let with_table = snapshot
            .apply_edit(
                StructuralSnapshotId(2),
                StructuralEdit::SetTableShape {
                    node_id: TreeNodeId(2),
                    table_shape: table_shape("table:sales"),
                },
            )
            .unwrap();
        assert_eq!(with_table.impact, StructuralEditImpact::RebindRequired);
        assert_eq!(with_table.snapshot.snapshot_id(), StructuralSnapshotId(2));
        assert_eq!(
            with_table
                .snapshot
                .table_shape_for(TreeNodeId(2))
                .map(|shape| shape.table_id.as_str()),
            Some("table:sales")
        );

        let cleared = with_table
            .snapshot
            .apply_edit(
                StructuralSnapshotId(3),
                StructuralEdit::ClearTableShape {
                    node_id: TreeNodeId(2),
                },
            )
            .unwrap();
        assert_eq!(cleared.snapshot.snapshot_id(), StructuralSnapshotId(3));
        assert!(cleared.snapshot.table_shape_for(TreeNodeId(2)).is_none());
    }

    fn grid_shape(grid_id: &str) -> StructuralGridShape {
        StructuralGridShape {
            grid_id: grid_id.to_string(),
            sheet_name: "Sheet1".to_string(),
            bounds_identity: "1048576x16384".to_string(),
            cell_population_version: "cells:v1".to_string(),
            axis_state_version: "axes:v1".to_string(),
            overlay_set_version: "overlays:v1".to_string(),
            merged_region_version: "merges:v1".to_string(),
        }
    }

    #[test]
    fn structural_grid_shape_edits_are_snapshot_facts() {
        let root = node(1, StructuralNodeKind::Root, "Root", None, &[2]);
        let sheet_node = node(2, StructuralNodeKind::Container, "Sheet1", Some(1), &[]);

        let snapshot =
            StructuralSnapshot::create(StructuralSnapshotId(1), TreeNodeId(1), [root, sheet_node])
                .unwrap();

        let with_grid = snapshot
            .apply_edit(
                StructuralSnapshotId(2),
                StructuralEdit::SetGridShape {
                    node_id: TreeNodeId(2),
                    grid_shape: grid_shape("grid:sheet1"),
                },
            )
            .unwrap();
        assert_eq!(with_grid.impact, StructuralEditImpact::RebindRequired);
        assert_eq!(
            with_grid
                .snapshot
                .grid_shape_for(TreeNodeId(2))
                .map(|grid| grid.grid_id.as_str()),
            Some("grid:sheet1")
        );
        // A grid backing is not a table backing.
        assert!(
            with_grid
                .snapshot
                .node_backings()
                .values()
                .all(|backing| backing.as_table().is_none())
        );
        assert!(with_grid.snapshot.table_shape_for(TreeNodeId(2)).is_none());

        let cleared = with_grid
            .snapshot
            .apply_edit(
                StructuralSnapshotId(3),
                StructuralEdit::ClearGridShape {
                    node_id: TreeNodeId(2),
                },
            )
            .unwrap();
        assert!(cleared.snapshot.backing_for(TreeNodeId(2)).is_none());
    }

    #[test]
    fn node_backings_round_trip_through_serde() {
        let root = node(1, StructuralNodeKind::Root, "Root", None, &[2, 3]);
        let table_node = node(2, StructuralNodeKind::Container, "Sales", Some(1), &[]);
        let sheet_node = node(3, StructuralNodeKind::Container, "Sheet1", Some(1), &[]);

        let mut backings = BTreeMap::new();
        backings.insert(
            TreeNodeId(2),
            NodeBacking::Table(table_shape("table:sales")),
        );
        backings.insert(TreeNodeId(3), NodeBacking::Grid(grid_shape("grid:sheet1")));
        let snapshot = StructuralSnapshot::create_with_node_backings(
            StructuralSnapshotId(1),
            TreeNodeId(1),
            [root, table_node, sheet_node],
            backings,
        )
        .unwrap();

        let json = serde_json::to_string(&snapshot).unwrap();
        let round: StructuralSnapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(round, snapshot);
        assert!(matches!(
            round.backing_for(TreeNodeId(2)),
            Some(NodeBacking::Table(_))
        ));
        assert!(matches!(
            round.backing_for(TreeNodeId(3)),
            Some(NodeBacking::Grid(_))
        ));
    }

    #[test]
    fn pre_node_backing_table_shapes_payload_no_longer_folds_into_backings() {
        // D1 §8 item 2 (readability decision): the legacy `table_shapes` fold is
        // gone. A pre-`NodeBacking` snapshot — which carried its table shapes
        // under a `table_shapes` map and no `node_backings` — no longer yields
        // its table backings: the unknown `table_shapes`/`path_index` keys are
        // ignored on load, so the payload deserializes as a plain tree and the
        // authored table shape is LOST. That loss is the recorded readability
        // decision: pre-`NodeBacking` snapshots are unreadable as workbook
        // content. Current-generation (`node_backings` form) snapshots remain
        // readable (see `node_backings_round_trip_through_serde`).
        let legacy_json = r#"{
            "snapshot_id": 1,
            "root_node_id": 1,
            "nodes": {
                "1": {"node_id": 1, "kind": "Root", "symbol": "Root", "parent_id": null, "child_ids": [2]},
                "2": {"node_id": 2, "kind": "Container", "symbol": "Sales", "parent_id": 1, "child_ids": []}
            },
            "table_shapes": {"2": {"table_id": "table:sales"}},
            "path_index": {"Root": 1, "Root/Sales": 2}
        }"#;

        // The structurally-valid tree still loads (through the validated
        // constructor), but the legacy table shape does NOT fold in.
        let snapshot: StructuralSnapshot =
            serde_json::from_str(legacy_json).expect("plain tree remains structurally loadable");
        assert!(
            snapshot.table_shape_for(TreeNodeId(2)).is_none(),
            "legacy table_shapes fold must be gone: the table backing is not migrated"
        );
        assert!(
            snapshot.node_backings().is_empty(),
            "no backing of any kind materializes from a legacy table_shapes map"
        );
    }

    #[test]
    fn current_generation_wire_has_no_derived_indexes() {
        // D1 §8 item 3: neither derived index is serialized. `sheet_index` was
        // already off the wire (R2.3); `path_index` leaves it here.
        let root = node(1, StructuralNodeKind::Root, "Root", None, &[2]);
        let child = node(2, StructuralNodeKind::Container, "Sales", Some(1), &[]);
        let snapshot =
            StructuralSnapshot::create(StructuralSnapshotId(1), TreeNodeId(1), [root, child])
                .unwrap();

        let json = serde_json::to_string(&snapshot).unwrap();
        assert!(!json.contains("path_index"));
        assert!(!json.contains("sheet_index"));
        assert!(!json.contains("table_shapes"));

        // And the derived indexes rebuild on load: the round-trip is identical.
        let round: StructuralSnapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(round, snapshot);
    }

    #[test]
    fn deserialize_rejects_role_off_root_with_typed_error() {
        // D1 §8 item 3: deserialization routes through the validated
        // constructor, so a wire payload with the Sheet role on a node that is
        // not a Workbook-root child fails to load with the same typed error the
        // constructor raises — not a silently-loaded invalid snapshot.
        let root = node(1, StructuralNodeKind::Root, "Root", None, &[2]);
        let orphan_sheet = node_with_role(
            2,
            StructuralNodeKind::Container,
            "Sheet1",
            Some(1),
            &[],
            Some(NodeRole::Sheet),
        );
        let wire = StructuralSnapshotWire {
            snapshot_id: StructuralSnapshotId(1),
            root_node_id: TreeNodeId(1),
            nodes: [root, orphan_sheet]
                .into_iter()
                .map(|n| (n.node_id, n))
                .collect(),
            node_backings: BTreeMap::new(),
        };
        let err = StructuralSnapshot::try_from(wire).unwrap_err();
        assert_eq!(
            err,
            StructuralError::SheetRoleRequiresWorkbookParent {
                node_id: TreeNodeId(2)
            }
        );

        // The same rejection surfaces through the serde entry point.
        let json = r#"{
            "snapshot_id": 1,
            "root_node_id": 1,
            "nodes": {
                "1": {"node_id": 1, "kind": "Root", "symbol": "Root", "parent_id": null, "child_ids": [2]},
                "2": {"node_id": 2, "kind": "Container", "symbol": "Sheet1", "parent_id": 1, "child_ids": [], "role": "Sheet"}
            }
        }"#;
        let serde_err = serde_json::from_str::<StructuralSnapshot>(json).unwrap_err();
        assert!(
            serde_err.to_string().contains("Sheet role"),
            "serde error should carry the typed validation message: {serde_err}"
        );
    }

    #[test]
    fn deserialize_rejects_duplicate_case_twin_sheets_with_typed_error() {
        // D1 §8 item 3 + §2: a wire payload with two Sheet siblings whose names
        // fold to the same normalized form is a case-twin collision. The
        // deserialize path now enforces uniqueness with a typed error rather
        // than collapsing them.
        let root = node_with_role(
            1,
            StructuralNodeKind::Root,
            "Book",
            None,
            &[2, 3],
            Some(NodeRole::Workbook),
        );
        let sheet_a = node_with_role(
            2,
            StructuralNodeKind::Container,
            "Dup",
            Some(1),
            &[],
            Some(NodeRole::Sheet),
        );
        let sheet_b = node_with_role(
            3,
            StructuralNodeKind::Container,
            "dup",
            Some(1),
            &[],
            Some(NodeRole::Sheet),
        );
        let wire = StructuralSnapshotWire {
            snapshot_id: StructuralSnapshotId(1),
            root_node_id: TreeNodeId(1),
            nodes: [root, sheet_a, sheet_b]
                .into_iter()
                .map(|n| (n.node_id, n))
                .collect(),
            node_backings: BTreeMap::new(),
        };
        let err = StructuralSnapshot::try_from(wire).unwrap_err();
        assert!(
            matches!(err, StructuralError::DuplicateSheetName { .. }),
            "expected DuplicateSheetName, got {err:?}"
        );
    }

    #[test]
    fn deserialize_rejects_meta_prefix_on_non_meta_node_with_typed_error() {
        // D1 §8 item 3 + §5: a `#`-prefixed symbol on a non-meta node is a
        // reserved-prefix violation. The deserialize path rejects it typed.
        let root = node(1, StructuralNodeKind::Root, "Root", None, &[2]);
        let bad = node(2, StructuralNodeKind::Container, "#settings", Some(1), &[]);
        let wire = StructuralSnapshotWire {
            snapshot_id: StructuralSnapshotId(1),
            root_node_id: TreeNodeId(1),
            nodes: [root, bad].into_iter().map(|n| (n.node_id, n)).collect(),
            node_backings: BTreeMap::new(),
        };
        let err = StructuralSnapshot::try_from(wire).unwrap_err();
        assert_eq!(
            err,
            StructuralError::MetaPrefixReservedForMetaNodes {
                node_id: TreeNodeId(2),
                symbol: "#settings".to_string()
            }
        );
    }

    fn node_with_role(
        node_id: u64,
        kind: StructuralNodeKind,
        symbol: &str,
        parent_id: Option<u64>,
        child_ids: &[u64],
        role: Option<NodeRole>,
    ) -> StructuralNode {
        node(node_id, kind, symbol, parent_id, child_ids).with_role(role)
    }

    #[test]
    fn workbook_role_on_root_and_sheet_child_is_accepted() {
        let root = node_with_role(
            1,
            StructuralNodeKind::Root,
            "Book",
            None,
            &[2, 3],
            Some(NodeRole::Workbook),
        );
        let sheet = node_with_role(
            2,
            StructuralNodeKind::Container,
            "Sheet1",
            Some(1),
            &[],
            Some(NodeRole::Sheet),
        );
        // A non-sheet root child interleaves freely (D1 §3).
        let plain = node(3, StructuralNodeKind::Calculation, "Calc", Some(1), &[]);

        let snapshot = StructuralSnapshot::create(
            StructuralSnapshotId(1),
            TreeNodeId(1),
            [root, sheet, plain],
        )
        .unwrap();
        assert_eq!(
            snapshot.try_get_node(TreeNodeId(1)).unwrap().role,
            Some(NodeRole::Workbook)
        );
        assert_eq!(
            snapshot.try_get_node(TreeNodeId(2)).unwrap().role,
            Some(NodeRole::Sheet)
        );
    }

    #[test]
    fn workbook_role_off_root_is_rejected_by_constructor() {
        // Root has no Workbook role; a child claims Workbook.
        let root = node(1, StructuralNodeKind::Root, "Root", None, &[2]);
        let child = node_with_role(
            2,
            StructuralNodeKind::Container,
            "Inner",
            Some(1),
            &[],
            Some(NodeRole::Workbook),
        );

        let err = StructuralSnapshot::create(StructuralSnapshotId(1), TreeNodeId(1), [root, child])
            .unwrap_err();
        assert_eq!(
            err,
            StructuralError::WorkbookRoleRequiresRoot {
                node_id: TreeNodeId(2)
            }
        );
    }

    #[test]
    fn sheet_role_under_non_workbook_root_is_rejected_by_constructor() {
        // Root is a plain tree root (role None); a child claims Sheet.
        let root = node(1, StructuralNodeKind::Root, "Root", None, &[2]);
        let sheet = node_with_role(
            2,
            StructuralNodeKind::Container,
            "Sheet1",
            Some(1),
            &[],
            Some(NodeRole::Sheet),
        );

        let err = StructuralSnapshot::create(StructuralSnapshotId(1), TreeNodeId(1), [root, sheet])
            .unwrap_err();
        assert_eq!(
            err,
            StructuralError::SheetRoleRequiresWorkbookParent {
                node_id: TreeNodeId(2)
            }
        );
    }

    #[test]
    fn sheet_role_below_workbook_root_is_rejected_when_not_a_direct_child() {
        // Workbook root -> container -> sheet: sheet is a grandchild, illegal.
        let root = node_with_role(
            1,
            StructuralNodeKind::Root,
            "Book",
            None,
            &[2],
            Some(NodeRole::Workbook),
        );
        let container = node(2, StructuralNodeKind::Container, "Group", Some(1), &[3]);
        let sheet = node_with_role(
            3,
            StructuralNodeKind::Container,
            "Sheet1",
            Some(2),
            &[],
            Some(NodeRole::Sheet),
        );

        let err = StructuralSnapshot::create(
            StructuralSnapshotId(1),
            TreeNodeId(1),
            [root, container, sheet],
        )
        .unwrap_err();
        assert_eq!(
            err,
            StructuralError::SheetRoleRequiresWorkbookParent {
                node_id: TreeNodeId(3)
            }
        );
    }

    #[test]
    fn insert_edit_producing_workbook_off_root_is_rejected() {
        let root = node(1, StructuralNodeKind::Root, "Root", None, &[]);
        let snapshot =
            StructuralSnapshot::create(StructuralSnapshotId(1), TreeNodeId(1), [root]).unwrap();

        let err = snapshot
            .apply_edit(
                StructuralSnapshotId(2),
                StructuralEdit::InsertNode {
                    node: node_with_role(
                        2,
                        StructuralNodeKind::Container,
                        "Inner",
                        None,
                        &[],
                        Some(NodeRole::Workbook),
                    ),
                    parent_id: TreeNodeId(1),
                    index: None,
                },
            )
            .unwrap_err();
        assert_eq!(
            err,
            StructuralError::WorkbookRoleRequiresRoot {
                node_id: TreeNodeId(2)
            }
        );
    }

    #[test]
    fn move_sheet_under_non_workbook_parent_is_rejected_by_apply_edit() {
        // Workbook root with two direct children: a sheet and a plain container.
        // Moving the sheet under the plain container must fail (D1 §1 rule 4,
        // reached through the normal build-then-validate path).
        let root = node_with_role(
            1,
            StructuralNodeKind::Root,
            "Book",
            None,
            &[2, 3],
            Some(NodeRole::Workbook),
        );
        let sheet = node_with_role(
            2,
            StructuralNodeKind::Container,
            "Sheet1",
            Some(1),
            &[],
            Some(NodeRole::Sheet),
        );
        let container = node(3, StructuralNodeKind::Container, "Group", Some(1), &[]);

        let snapshot = StructuralSnapshot::create(
            StructuralSnapshotId(1),
            TreeNodeId(1),
            [root, sheet, container],
        )
        .unwrap();

        let err = snapshot
            .apply_edit(
                StructuralSnapshotId(2),
                StructuralEdit::MoveNode {
                    node_id: TreeNodeId(2),
                    new_parent_id: TreeNodeId(3),
                    new_index: None,
                },
            )
            .unwrap_err();
        assert_eq!(
            err,
            StructuralError::SheetRoleRequiresWorkbookParent {
                node_id: TreeNodeId(2)
            }
        );
    }

    #[test]
    fn move_sheet_between_workbook_children_positions_is_accepted() {
        // Reordering a sheet among the workbook root's direct children is legal.
        let root = node_with_role(
            1,
            StructuralNodeKind::Root,
            "Book",
            None,
            &[2, 3],
            Some(NodeRole::Workbook),
        );
        let sheet1 = node_with_role(
            2,
            StructuralNodeKind::Container,
            "Sheet1",
            Some(1),
            &[],
            Some(NodeRole::Sheet),
        );
        let sheet2 = node_with_role(
            3,
            StructuralNodeKind::Container,
            "Sheet2",
            Some(1),
            &[],
            Some(NodeRole::Sheet),
        );

        let snapshot = StructuralSnapshot::create(
            StructuralSnapshotId(1),
            TreeNodeId(1),
            [root, sheet1, sheet2],
        )
        .unwrap();

        let moved = snapshot
            .apply_edit(
                StructuralSnapshotId(2),
                StructuralEdit::MoveNode {
                    node_id: TreeNodeId(2),
                    new_parent_id: TreeNodeId(1),
                    new_index: Some(1),
                },
            )
            .unwrap();
        assert_eq!(
            moved
                .snapshot
                .try_get_node(TreeNodeId(1))
                .unwrap()
                .child_ids,
            vec![TreeNodeId(3), TreeNodeId(2)]
        );
    }

    #[test]
    fn role_round_trips_through_serde() {
        let root = node_with_role(
            1,
            StructuralNodeKind::Root,
            "Book",
            None,
            &[2],
            Some(NodeRole::Workbook),
        );
        let sheet = node_with_role(
            2,
            StructuralNodeKind::Container,
            "Sheet1",
            Some(1),
            &[],
            Some(NodeRole::Sheet),
        );
        let snapshot =
            StructuralSnapshot::create(StructuralSnapshotId(1), TreeNodeId(1), [root, sheet])
                .unwrap();

        let json = serde_json::to_string(&snapshot).unwrap();
        let round: StructuralSnapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(round, snapshot);
        assert_eq!(
            round.try_get_node(TreeNodeId(2)).unwrap().role,
            Some(NodeRole::Sheet)
        );
    }

    #[test]
    fn pre_role_node_payload_loads_with_role_none() {
        // A node object serialized before the `role` field existed omits it;
        // `#[serde(default)]` must load it as `None` (D1 §8 item 1).
        let legacy_node_json = r#"{
            "node_id": 7,
            "kind": "Container",
            "symbol": "Legacy",
            "parent_id": 1,
            "child_ids": []
        }"#;
        let loaded: StructuralNode = serde_json::from_str(legacy_node_json).unwrap();
        assert_eq!(loaded.role, None);
        assert_eq!(loaded.node_id, TreeNodeId(7));
    }

    #[test]
    fn pre_role_snapshot_payload_loads_with_all_roles_none() {
        // A whole snapshot serialized before `role` existed: node objects carry
        // no `role` key. It must load as a plain tree (all roles None).
        let root = node(1, StructuralNodeKind::Root, "Root", None, &[2]);
        let child = node(2, StructuralNodeKind::Container, "Child", Some(1), &[]);
        let base =
            StructuralSnapshot::create(StructuralSnapshotId(1), TreeNodeId(1), [root, child])
                .unwrap();

        // Serialize, then strip every `"role":null` token to emulate a payload
        // written before the field existed.
        let json = serde_json::to_string(&base).unwrap();
        let legacy = json
            .replace(",\"role\":null", "")
            .replace("\"role\":null,", "");
        assert!(!legacy.contains("\"role\""));

        let loaded: StructuralSnapshot = serde_json::from_str(&legacy).unwrap();
        assert_eq!(loaded, base);
        assert!(loaded.nodes().values().all(|node| node.role.is_none()));
    }

    #[test]
    fn meta_prefix_symbol_on_non_meta_node_is_rejected() {
        // D1 §4 rule 1: the `#` prefix is reserved for meta nodes.
        let root = node(1, StructuralNodeKind::Root, "Root", None, &[2]);
        let group = node(2, StructuralNodeKind::Container, "#settings", Some(1), &[]);
        let err = StructuralSnapshot::create(StructuralSnapshotId(1), TreeNodeId(1), [root, group])
            .unwrap_err();
        assert_eq!(
            err,
            StructuralError::MetaPrefixReservedForMetaNodes {
                node_id: TreeNodeId(2),
                symbol: "#settings".to_string(),
            }
        );
    }

    #[test]
    fn meta_prefix_symbol_on_meta_node_is_accepted() {
        // A `#`-prefixed symbol is legal precisely when the node is meta.
        let root = node(1, StructuralNodeKind::Root, "Root", None, &[2]);
        let group =
            node(2, StructuralNodeKind::Container, "#settings", Some(1), &[]).with_meta(true);
        StructuralSnapshot::create(StructuralSnapshotId(1), TreeNodeId(1), [root, group]).unwrap();
    }

    #[test]
    fn child_of_meta_node_that_is_not_meta_is_rejected() {
        // D1 §4 rule 2: meta-ness is inherited-checked. A child of a meta node
        // must itself be meta.
        let root = node(1, StructuralNodeKind::Root, "Root", None, &[2]);
        let group =
            node(2, StructuralNodeKind::Container, "#settings", Some(1), &[3]).with_meta(true);
        let leaf = node(3, StructuralNodeKind::Constant, "leaf", Some(2), &[]);
        let err =
            StructuralSnapshot::create(StructuralSnapshotId(1), TreeNodeId(1), [root, group, leaf])
                .unwrap_err();
        assert_eq!(
            err,
            StructuralError::MetaChildRequiresMetaParent {
                node_id: TreeNodeId(3),
                parent_id: TreeNodeId(2),
            }
        );
    }

    #[test]
    fn meta_node_may_attach_under_non_meta_parent() {
        // Inheritance flows to descendants, not ancestors: a property group
        // hangs off an ordinary (non-meta) node.
        let root = node(1, StructuralNodeKind::Root, "Root", None, &[2]);
        let group =
            node(2, StructuralNodeKind::Container, "#settings", Some(1), &[3]).with_meta(true);
        let leaf = node(3, StructuralNodeKind::Constant, "leaf", Some(2), &[]).with_meta(true);
        StructuralSnapshot::create(StructuralSnapshotId(1), TreeNodeId(1), [root, group, leaf])
            .unwrap();
    }

    #[test]
    fn meta_node_carrying_a_role_is_rejected() {
        // D1 §4 rule 3 / §1: meta subtrees and document roles are disjoint.
        let root = node(1, StructuralNodeKind::Root, "Root", None, &[2]);
        let sheetish = node(2, StructuralNodeKind::Container, "meta", Some(1), &[])
            .with_meta(true)
            .with_role(Some(NodeRole::Sheet));
        let err =
            StructuralSnapshot::create(StructuralSnapshotId(1), TreeNodeId(1), [root, sheetish])
                .unwrap_err();
        assert_eq!(
            err,
            StructuralError::MetaNodeCannotCarryRole {
                node_id: TreeNodeId(2),
            }
        );
    }

    #[test]
    fn set_node_meta_edit_toggles_membership_and_mints_new_snapshot_id() {
        let root = node(1, StructuralNodeKind::Root, "Root", None, &[2]);
        let child = node(2, StructuralNodeKind::Container, "props", Some(1), &[]);
        let base =
            StructuralSnapshot::create(StructuralSnapshotId(1), TreeNodeId(1), [root, child])
                .unwrap();
        assert!(base.meta_node_ids().is_empty());

        let outcome = base
            .apply_edit(
                StructuralSnapshotId(2),
                StructuralEdit::SetNodeMeta {
                    node_id: TreeNodeId(2),
                    is_meta: true,
                },
            )
            .unwrap();
        assert_eq!(outcome.snapshot.snapshot_id(), StructuralSnapshotId(2));
        assert!(
            outcome
                .snapshot
                .try_get_node(TreeNodeId(2))
                .unwrap()
                .is_meta
        );
        assert_eq!(
            outcome.snapshot.meta_node_ids(),
            BTreeSet::from([TreeNodeId(2)])
        );

        let cleared = outcome
            .snapshot
            .apply_edit(
                StructuralSnapshotId(3),
                StructuralEdit::SetNodeMeta {
                    node_id: TreeNodeId(2),
                    is_meta: false,
                },
            )
            .unwrap();
        assert!(cleared.snapshot.meta_node_ids().is_empty());
    }

    #[test]
    fn meta_node_ids_derivation_reflects_is_meta_flags() {
        let root = node(1, StructuralNodeKind::Root, "Root", None, &[2, 3]);
        let plain = node(2, StructuralNodeKind::Constant, "a", Some(1), &[]);
        let meta = node(3, StructuralNodeKind::Container, "#m", Some(1), &[]).with_meta(true);
        let snapshot =
            StructuralSnapshot::create(StructuralSnapshotId(1), TreeNodeId(1), [root, plain, meta])
                .unwrap();
        assert_eq!(snapshot.meta_node_ids(), BTreeSet::from([TreeNodeId(3)]));
    }

    #[test]
    fn is_meta_round_trips_through_serde() {
        let root = node(1, StructuralNodeKind::Root, "Root", None, &[2]);
        let meta = node(2, StructuralNodeKind::Container, "#m", Some(1), &[]).with_meta(true);
        let snapshot =
            StructuralSnapshot::create(StructuralSnapshotId(1), TreeNodeId(1), [root, meta])
                .unwrap();
        let json = serde_json::to_string(&snapshot).unwrap();
        let round: StructuralSnapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(round, snapshot);
        assert!(round.try_get_node(TreeNodeId(2)).unwrap().is_meta);
    }

    #[test]
    fn pre_promotion_node_payload_loads_with_is_meta_false() {
        // A node serialized before `is_meta` existed omits it; `#[serde(default)]`
        // must load it as `false` (D1 §8 item 1).
        let legacy_node_json = r#"{
            "node_id": 7,
            "kind": "Container",
            "symbol": "Legacy",
            "parent_id": 1,
            "child_ids": [],
            "role": null
        }"#;
        let loaded: StructuralNode = serde_json::from_str(legacy_node_json).unwrap();
        assert!(!loaded.is_meta);
        assert_eq!(loaded.node_id, TreeNodeId(7));
    }

    // --- R2.3: sheet registry (calc-5kqg.10) ---

    /// Builds a workbook root with the given Sheet-role children (by symbol),
    /// each a direct child in the order given. Returns the built snapshot.
    fn workbook_with_sheets(sheet_symbols: &[&str]) -> StructuralSnapshot {
        let child_ids: Vec<u64> = (0..sheet_symbols.len() as u64).map(|i| i + 2).collect();
        let root = node_with_role(
            1,
            StructuralNodeKind::Root,
            "Book",
            None,
            &child_ids,
            Some(NodeRole::Workbook),
        );
        let mut nodes = vec![root];
        for (offset, symbol) in sheet_symbols.iter().enumerate() {
            nodes.push(node_with_role(
                offset as u64 + 2,
                StructuralNodeKind::Container,
                symbol,
                Some(1),
                &[],
                Some(NodeRole::Sheet),
            ));
        }
        StructuralSnapshot::create(StructuralSnapshotId(1), TreeNodeId(1), nodes).unwrap()
    }

    #[test]
    fn sheet_registry_resolves_case_insensitively_and_preserves_display() {
        // D1 §1 contract: sheet_index[NormalizedSheetName] -> TreeNodeId;
        // display capitalization lives on the symbol.
        let snapshot = workbook_with_sheets(&["Sheet1", "Summary"]);

        // Lookup is case-insensitive.
        assert_eq!(
            snapshot.try_resolve_sheet_name("sheet1"),
            Some(TreeNodeId(2))
        );
        assert_eq!(
            snapshot.try_resolve_sheet_name("SHEET1"),
            Some(TreeNodeId(2))
        );
        assert_eq!(
            snapshot.try_resolve_sheet_name("Summary"),
            Some(TreeNodeId(3))
        );
        assert_eq!(snapshot.try_resolve_sheet_name("Nope"), None);

        // Display capitalization is untouched on the symbol.
        assert_eq!(snapshot.try_get_node(TreeNodeId(2)).unwrap().symbol, "Sheet1");
    }

    #[test]
    fn sheet_registry_ignores_non_sheet_root_children() {
        // A plain (role None) root child interleaved among sheets is not a sheet.
        let root = node_with_role(
            1,
            StructuralNodeKind::Root,
            "Book",
            None,
            &[2, 3, 4],
            Some(NodeRole::Workbook),
        );
        let sheet_a = node_with_role(
            2,
            StructuralNodeKind::Container,
            "Alpha",
            Some(1),
            &[],
            Some(NodeRole::Sheet),
        );
        let plain = node(3, StructuralNodeKind::Calculation, "Calc", Some(1), &[]);
        let sheet_b = node_with_role(
            4,
            StructuralNodeKind::Container,
            "Beta",
            Some(1),
            &[],
            Some(NodeRole::Sheet),
        );

        let snapshot = StructuralSnapshot::create(
            StructuralSnapshotId(1),
            TreeNodeId(1),
            [root, sheet_a, plain, sheet_b],
        )
        .unwrap();

        // The plain "Calc" node, interleaved between the sheets, is NOT in the
        // sheet registry: only Sheet-role children are registered.
        assert_eq!(snapshot.try_resolve_sheet_name("Calc"), None);
        assert_eq!(snapshot.try_resolve_sheet_name("Alpha"), Some(TreeNodeId(2)));
        assert_eq!(snapshot.try_resolve_sheet_name("Beta"), Some(TreeNodeId(4)));
        assert_eq!(snapshot.sheet_index().len(), 2);
        // Enumeration skips the interleaved non-sheet child.
        assert_eq!(snapshot.sheet_nodes(), vec![TreeNodeId(2), TreeNodeId(4)]);
    }

    #[test]
    fn case_twin_sheet_names_are_rejected_by_constructor() {
        // D1 §2 acceptance: case-twin sheet names (Dup/dup) are rejected.
        let root = node_with_role(
            1,
            StructuralNodeKind::Root,
            "Book",
            None,
            &[2, 3],
            Some(NodeRole::Workbook),
        );
        let sheet_a = node_with_role(
            2,
            StructuralNodeKind::Container,
            "Dup",
            Some(1),
            &[],
            Some(NodeRole::Sheet),
        );
        let sheet_b = node_with_role(
            3,
            StructuralNodeKind::Container,
            "dup",
            Some(1),
            &[],
            Some(NodeRole::Sheet),
        );

        let err = StructuralSnapshot::create(
            StructuralSnapshotId(1),
            TreeNodeId(1),
            [root, sheet_a, sheet_b],
        )
        .unwrap_err();
        assert_eq!(
            err,
            StructuralError::DuplicateSheetName {
                normalized: "dup".to_string(),
                node_ids: "node:2, node:3".to_string(),
            }
        );
    }

    #[test]
    fn sheet_enumeration_order_tracks_child_ids() {
        // D1 §2/§3 acceptance: enumeration order tracks child_ids, not the
        // folded key order of the registry map.
        let root = node_with_role(
            1,
            StructuralNodeKind::Root,
            "Book",
            None,
            // Authored order zebra..alpha is reverse of folded key order.
            &[2, 3, 4],
            Some(NodeRole::Workbook),
        );
        let zebra = node_with_role(
            2,
            StructuralNodeKind::Container,
            "Zebra",
            Some(1),
            &[],
            Some(NodeRole::Sheet),
        );
        let middle = node_with_role(
            3,
            StructuralNodeKind::Container,
            "Middle",
            Some(1),
            &[],
            Some(NodeRole::Sheet),
        );
        let alpha = node_with_role(
            4,
            StructuralNodeKind::Container,
            "Alpha",
            Some(1),
            &[],
            Some(NodeRole::Sheet),
        );

        let snapshot = StructuralSnapshot::create(
            StructuralSnapshotId(1),
            TreeNodeId(1),
            [root, zebra, middle, alpha],
        )
        .unwrap();

        // sheet_nodes() follows child_ids (authored) order.
        assert_eq!(
            snapshot.sheet_nodes(),
            vec![TreeNodeId(2), TreeNodeId(3), TreeNodeId(4)]
        );
        // The registry map key order is the folded order (alpha, middle, zebra):
        // distinct from enumeration order, proving they are different surfaces.
        let key_order: Vec<&str> = snapshot
            .sheet_index()
            .keys()
            .map(NormalizedSheetName::as_str)
            .collect();
        assert_eq!(key_order, vec!["alpha", "middle", "zebra"]);
    }

    #[test]
    fn sheet_rename_keeps_node_id_and_updates_index() {
        // D1 §2 acceptance: rename keeps the node id and updates the index.
        let snapshot = workbook_with_sheets(&["Sheet1", "Summary"]);
        assert_eq!(snapshot.try_resolve_sheet_name("Sheet1"), Some(TreeNodeId(2)));

        let outcome = snapshot
            .apply_edit(
                StructuralSnapshotId(2),
                StructuralEdit::RenameNode {
                    node_id: TreeNodeId(2),
                    new_symbol: "Renamed".to_string(),
                },
            )
            .unwrap();
        let renamed = outcome.snapshot;

        // Node id is stable across the rename.
        assert_eq!(renamed.try_get_node(TreeNodeId(2)).unwrap().symbol, "Renamed");
        // Old name no longer resolves; new name resolves to the SAME node id.
        assert_eq!(renamed.try_resolve_sheet_name("Sheet1"), None);
        assert_eq!(
            renamed.try_resolve_sheet_name("Renamed"),
            Some(TreeNodeId(2))
        );
        // The untouched sibling still resolves.
        assert_eq!(
            renamed.try_resolve_sheet_name("Summary"),
            Some(TreeNodeId(3))
        );
    }

    #[test]
    fn sheet_index_rebuilds_on_deserialize_and_is_not_serialized() {
        // D1 §2: the sheet registry is derived and never serialized; it must be
        // rebuilt on load so a round-tripped snapshot has the same registry.
        let snapshot = workbook_with_sheets(&["Sheet1", "Summary"]);
        let json = serde_json::to_string(&snapshot).unwrap();

        // The derived index does not appear in the wire form.
        assert!(!json.contains("sheet_index"));

        let round: StructuralSnapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(round.try_resolve_sheet_name("SHEET1"), Some(TreeNodeId(2)));
        assert_eq!(round.try_resolve_sheet_name("summary"), Some(TreeNodeId(3)));
        assert_eq!(round.sheet_nodes(), vec![TreeNodeId(2), TreeNodeId(3)]);
        assert_eq!(round.sheet_index(), snapshot.sheet_index());
    }

    #[test]
    fn plain_tree_snapshot_has_empty_sheet_registry() {
        // A snapshot with no workbook/sheet roles (everything TreeCalc builds
        // today) has an empty registry and empty enumeration.
        let root = node(1, StructuralNodeKind::Root, "Root", None, &[2]);
        let child = node(2, StructuralNodeKind::Container, "Branch", Some(1), &[]);
        let snapshot =
            StructuralSnapshot::create(StructuralSnapshotId(1), TreeNodeId(1), [root, child])
                .unwrap();
        assert!(snapshot.sheet_index().is_empty());
        assert!(snapshot.sheet_nodes().is_empty());
        assert_eq!(snapshot.try_resolve_sheet_name("Branch"), None);
    }

    #[test]
    fn normalized_sheet_name_folds_non_ascii_case() {
        // The fold rule uses char::to_lowercase (full Unicode lowercase),
        // so bicameral non-ASCII scripts fold too (Greek Sigma here).
        assert_eq!(
            NormalizedSheetName::from_symbol("ΣΙΓΜΑ").as_str(),
            NormalizedSheetName::from_symbol("σιγμα").as_str()
        );
        // Accented characters pass through the fold with only the cased
        // letters lowered; the newtype is a pure fold of the symbol.
        assert_eq!(NormalizedSheetName::from_symbol("Café").as_str(), "café");
    }
}
