use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};

use crate::structural::{StructuralSnapshot, TreeNodeId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ContextHostNameResolution {
    Resolved(TreeNodeId),
    Ambiguous,
    Unsupported(&'static str),
    Unresolved,
}

/// Precomputed name-resolution index over one `(snapshot, meta_node_ids)`
/// pair. Hoists the work the per-call resolver otherwise repeats for every
/// formula in a run: meta-effectiveness ancestor walks, per-scope child
/// symbol scans, and the owner-independent visible-symbol sweep.
///
/// Resolution through the index is semantically identical to the free
/// functions below: both delegate to the same path/walkup logic and differ
/// only in how matching children are looked up.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TreeNameResolutionIndex {
    /// Nodes that are meta-effective (in `meta_node_ids`, or having an
    /// ancestor that is).
    meta_effective: BTreeSet<TreeNodeId>,
    /// Visible (non-meta-effective) children of each scope keyed by
    /// ASCII-uppercased symbol, preserving `child_ids` order. ASCII
    /// uppercase keying is equivalent to `eq_ignore_ascii_case`.
    children_by_symbol: BTreeMap<(TreeNodeId, String), Vec<TreeNodeId>>,
    /// Distinct symbols of all visible non-root nodes: the candidate set
    /// for context host-name bindings.
    visible_symbols: BTreeSet<String>,
}

impl TreeNameResolutionIndex {
    pub(crate) fn build(
        snapshot: &StructuralSnapshot,
        meta_node_ids: &BTreeSet<TreeNodeId>,
    ) -> Self {
        // Memoized equivalent of `is_meta_effective`: a node is
        // meta-effective iff it or any ancestor is in `meta_node_ids`.
        let mut memo: BTreeMap<TreeNodeId, bool> = BTreeMap::new();
        for &node_id in snapshot.nodes().keys() {
            let mut walked = Vec::new();
            let mut cursor = Some(node_id);
            let mut effective = false;
            while let Some(current) = cursor {
                if let Some(&known) = memo.get(&current) {
                    effective = known;
                    break;
                }
                if meta_node_ids.contains(&current) {
                    memo.insert(current, true);
                    effective = true;
                    break;
                }
                walked.push(current);
                cursor = snapshot.parent_id_of(current);
            }
            for chained in walked {
                memo.insert(chained, effective);
            }
        }
        let meta_effective = memo
            .into_iter()
            .filter_map(|(node_id, effective)| effective.then_some(node_id))
            .collect::<BTreeSet<_>>();

        let mut children_by_symbol: BTreeMap<(TreeNodeId, String), Vec<TreeNodeId>> =
            BTreeMap::new();
        for (scope_node_id, scope_node) in snapshot.nodes() {
            for child_id in scope_node.child_ids.iter().copied() {
                if meta_effective.contains(&child_id) {
                    continue;
                }
                let Some(child) = snapshot.try_get_node(child_id) else {
                    continue;
                };
                children_by_symbol
                    .entry((*scope_node_id, child.symbol.to_ascii_uppercase()))
                    .or_default()
                    .push(child_id);
            }
        }

        let visible_symbols = snapshot
            .nodes()
            .values()
            .filter(|node| {
                node.node_id != snapshot.root_node_id()
                    && !node.symbol.is_empty()
                    && !meta_effective.contains(&node.node_id)
            })
            .map(|node| node.symbol.clone())
            .collect::<BTreeSet<_>>();

        Self {
            meta_effective,
            children_by_symbol,
            visible_symbols,
        }
    }

    pub(crate) fn visible_symbols(&self) -> &BTreeSet<String> {
        &self.visible_symbols
    }

    pub(crate) fn resolve_context_host_name_token(
        &self,
        token: &str,
        owner_node_id: TreeNodeId,
        snapshot: &StructuralSnapshot,
    ) -> ContextHostNameResolution {
        resolve_token_with_backend(
            &ResolverBackend::Index(self),
            token,
            owner_node_id,
            snapshot,
        )
    }
}

pub(crate) fn resolve_context_host_name_token(
    token: &str,
    owner_node_id: TreeNodeId,
    snapshot: &StructuralSnapshot,
    meta_node_ids: &BTreeSet<TreeNodeId>,
) -> ContextHostNameResolution {
    resolve_token_with_backend(
        &ResolverBackend::Scan { meta_node_ids },
        token,
        owner_node_id,
        snapshot,
    )
}

/// Child-symbol lookup strategy; the resolution logic above it is shared.
enum ResolverBackend<'a> {
    Scan {
        meta_node_ids: &'a BTreeSet<TreeNodeId>,
    },
    Index(&'a TreeNameResolutionIndex),
}

impl ResolverBackend<'_> {
    /// Visible (non-meta-effective) children of `scope_node_id` whose
    /// symbol matches `symbol` ASCII-case-insensitively, in `child_ids`
    /// order.
    fn matching_children(
        &self,
        snapshot: &StructuralSnapshot,
        scope_node_id: TreeNodeId,
        symbol: &str,
    ) -> Cow<'_, [TreeNodeId]> {
        match self {
            Self::Scan { meta_node_ids } => {
                let Some(scope_node) = snapshot.try_get_node(scope_node_id) else {
                    return Cow::Owned(Vec::new());
                };
                Cow::Owned(
                    scope_node
                        .child_ids
                        .iter()
                        .copied()
                        .filter(|child_id| {
                            snapshot
                                .try_get_node(*child_id)
                                .is_some_and(|child| child.symbol.eq_ignore_ascii_case(symbol))
                                && !is_meta_effective(*child_id, snapshot, meta_node_ids)
                        })
                        .collect(),
                )
            }
            Self::Index(index) => index
                .children_by_symbol
                .get(&(scope_node_id, symbol.to_ascii_uppercase()))
                .map_or(Cow::Owned(Vec::new()), |children| {
                    Cow::Borrowed(children.as_slice())
                }),
        }
    }
}

fn resolve_token_with_backend(
    backend: &ResolverBackend<'_>,
    token: &str,
    owner_node_id: TreeNodeId,
    snapshot: &StructuralSnapshot,
) -> ContextHostNameResolution {
    if token.contains('!') {
        return ContextHostNameResolution::Unsupported("cross_workspace_host_path_pending");
    }
    let segments = token
        .split('.')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();
    resolve_context_host_path_segments(backend, &segments, owner_node_id, snapshot)
}

fn resolve_context_host_path_segments(
    backend: &ResolverBackend<'_>,
    segments: &[&str],
    owner_node_id: TreeNodeId,
    snapshot: &StructuralSnapshot,
) -> ContextHostNameResolution {
    if segments.is_empty() {
        return ContextHostNameResolution::Unresolved;
    }

    if snapshot
        .try_get_node(snapshot.root_node_id())
        .is_some_and(|root| root.symbol.eq_ignore_ascii_case(segments[0]))
    {
        return try_resolve_visible_descendant_path(
            backend,
            snapshot,
            snapshot.root_node_id(),
            &segments[1..],
        )
        .map_or(
            ContextHostNameResolution::Unresolved,
            ContextHostNameResolution::Resolved,
        );
    }

    let base = match resolve_context_walkup_symbol(backend, segments[0], owner_node_id, snapshot) {
        ContextHostNameResolution::Resolved(base_node_id) => base_node_id,
        other => return other,
    };
    if segments.len() == 1 {
        return ContextHostNameResolution::Resolved(base);
    }
    try_resolve_visible_descendant_path(backend, snapshot, base, &segments[1..]).map_or(
        ContextHostNameResolution::Unresolved,
        ContextHostNameResolution::Resolved,
    )
}

fn resolve_context_walkup_symbol(
    backend: &ResolverBackend<'_>,
    symbol: &str,
    owner_node_id: TreeNodeId,
    snapshot: &StructuralSnapshot,
) -> ContextHostNameResolution {
    let mut scope = Some(owner_node_id);
    while let Some(scope_node_id) = scope {
        match backend
            .matching_children(snapshot, scope_node_id, symbol)
            .as_ref()
        {
            [] => scope = snapshot.parent_id_of(scope_node_id),
            [node_id] => return ContextHostNameResolution::Resolved(*node_id),
            _ => return ContextHostNameResolution::Ambiguous,
        }
    }
    ContextHostNameResolution::Unresolved
}

fn try_resolve_visible_descendant_path(
    backend: &ResolverBackend<'_>,
    snapshot: &StructuralSnapshot,
    start_node_id: TreeNodeId,
    path_segments: &[&str],
) -> Option<TreeNodeId> {
    let mut cursor = Some(start_node_id);
    for segment in path_segments {
        cursor = cursor.and_then(|current| {
            backend
                .matching_children(snapshot, current, segment)
                .first()
                .copied()
        });
    }
    cursor
}

pub(crate) fn is_meta_effective(
    node_id: TreeNodeId,
    snapshot: &StructuralSnapshot,
    meta_node_ids: &BTreeSet<TreeNodeId>,
) -> bool {
    let mut cursor = Some(node_id);
    while let Some(current) = cursor {
        if meta_node_ids.contains(&current) {
            return true;
        }
        cursor = snapshot.parent_id_of(current);
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::structural::{StructuralNode, StructuralNodeKind, StructuralSnapshotId};

    fn node(
        node_id: u64,
        kind: StructuralNodeKind,
        symbol: &str,
        parent_id: Option<u64>,
        child_ids: Vec<u64>,
    ) -> StructuralNode {
        StructuralNode {
            node_id: TreeNodeId(node_id),
            kind,
            symbol: symbol.to_string(),
            parent_id: parent_id.map(TreeNodeId),
            child_ids: child_ids.into_iter().map(TreeNodeId).collect(),
            role: None,
            is_meta: false,
        }
    }

    /// Tree exercising case-insensitive matches, same-scope ambiguity,
    /// meta-effective exclusion (direct and inherited), and nested paths.
    fn resolution_snapshot() -> (StructuralSnapshot, BTreeSet<TreeNodeId>) {
        let snapshot = StructuralSnapshot::create(
            StructuralSnapshotId(7),
            TreeNodeId(1),
            [
                node(
                    1,
                    StructuralNodeKind::Root,
                    "Root",
                    None,
                    vec![2, 3, 4, 5, 6],
                ),
                node(
                    2,
                    StructuralNodeKind::Container,
                    "Alpha",
                    Some(1),
                    vec![7, 8],
                ),
                node(3, StructuralNodeKind::Constant, "beta", Some(1), vec![]),
                node(4, StructuralNodeKind::Constant, "BETA", Some(1), vec![]),
                node(5, StructuralNodeKind::Container, "Meta", Some(1), vec![9]),
                node(6, StructuralNodeKind::Calculation, "Gamma", Some(1), vec![]),
                node(7, StructuralNodeKind::Constant, "Child", Some(2), vec![]),
                node(8, StructuralNodeKind::Constant, "child", Some(2), vec![]),
                node(9, StructuralNodeKind::Constant, "Hidden", Some(5), vec![]),
            ],
        )
        .unwrap();
        let meta_node_ids = BTreeSet::from([TreeNodeId(5)]);
        (snapshot, meta_node_ids)
    }

    #[test]
    fn index_resolution_matches_scan_resolution_for_all_owners_and_tokens() {
        let (snapshot, meta_node_ids) = resolution_snapshot();
        let index = TreeNameResolutionIndex::build(&snapshot, &meta_node_ids);

        let tokens = [
            "Alpha",
            "ALPHA",
            "beta",
            "BETA",
            "Beta",
            "Meta",
            "Gamma",
            "Child",
            "CHILD",
            "Hidden",
            "Nope",
            "Alpha.Child",
            "alpha.child",
            "Root.Alpha.Child",
            "ROOT.alpha.CHILD",
            "Root.Meta.Hidden",
            "Alpha.Nope",
            "Other!Alpha",
            "Root",
            "",
            ".",
            "Alpha..Child",
        ];
        for owner_id in snapshot.nodes().keys() {
            for token in tokens {
                let scan =
                    resolve_context_host_name_token(token, *owner_id, &snapshot, &meta_node_ids);
                let indexed = index.resolve_context_host_name_token(token, *owner_id, &snapshot);
                assert_eq!(
                    scan, indexed,
                    "resolution diverged for token {token:?} from owner {owner_id:?}"
                );
            }
        }
    }

    #[test]
    fn index_resolution_classifies_expected_outcomes() {
        let (snapshot, meta_node_ids) = resolution_snapshot();
        let index = TreeNameResolutionIndex::build(&snapshot, &meta_node_ids);
        let owner = TreeNodeId(6);

        assert_eq!(
            index.resolve_context_host_name_token("Alpha", owner, &snapshot),
            ContextHostNameResolution::Resolved(TreeNodeId(2))
        );
        // Two case-insensitive matches in the same scope stay ambiguous.
        assert_eq!(
            index.resolve_context_host_name_token("beta", owner, &snapshot),
            ContextHostNameResolution::Ambiguous
        );
        // Descendant paths keep first-child-order resolution.
        assert_eq!(
            index.resolve_context_host_name_token("Alpha.CHILD", owner, &snapshot),
            ContextHostNameResolution::Resolved(TreeNodeId(7))
        );
        // Meta-effective nodes (direct and inherited) stay invisible.
        assert_eq!(
            index.resolve_context_host_name_token("Meta", owner, &snapshot),
            ContextHostNameResolution::Unresolved
        );
        assert_eq!(
            index.resolve_context_host_name_token("Hidden", owner, &snapshot),
            ContextHostNameResolution::Unresolved
        );
        assert_eq!(
            index.resolve_context_host_name_token("Cross!Name", owner, &snapshot),
            ContextHostNameResolution::Unsupported("cross_workspace_host_path_pending")
        );
    }

    #[test]
    fn index_visible_symbols_match_per_node_sweep() {
        let (snapshot, meta_node_ids) = resolution_snapshot();
        let index = TreeNameResolutionIndex::build(&snapshot, &meta_node_ids);

        let expected = snapshot
            .nodes()
            .values()
            .filter(|node| {
                node.node_id != snapshot.root_node_id()
                    && !node.symbol.is_empty()
                    && !is_meta_effective(node.node_id, &snapshot, &meta_node_ids)
            })
            .map(|node| node.symbol.clone())
            .collect::<BTreeSet<_>>();
        assert_eq!(*index.visible_symbols(), expected);
    }
}
