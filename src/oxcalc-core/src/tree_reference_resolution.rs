use std::collections::BTreeSet;

use crate::formula::split_treecalc_host_path_token;
use crate::structural::{StructuralSnapshot, TreeNodeId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ContextHostNameResolution {
    Resolved(TreeNodeId),
    Ambiguous,
    Unsupported(&'static str),
    Unresolved,
}

pub(crate) fn resolve_context_host_path_token(
    token: &str,
    owner_node_id: TreeNodeId,
    snapshot: &StructuralSnapshot,
    meta_node_ids: &BTreeSet<TreeNodeId>,
) -> ContextHostNameResolution {
    if token.contains('!') {
        return ContextHostNameResolution::Unsupported("cross_workspace_host_path_pending");
    }
    let segments = match split_treecalc_host_path_token(token) {
        Ok(segments) => segments,
        Err(_) => {
            return ContextHostNameResolution::Unsupported("invalid_bracket_escaped_host_path");
        }
    };
    resolve_context_host_path_segments(&segments, owner_node_id, snapshot, meta_node_ids)
}

pub(crate) fn resolve_context_host_name_token(
    token: &str,
    owner_node_id: TreeNodeId,
    snapshot: &StructuralSnapshot,
    meta_node_ids: &BTreeSet<TreeNodeId>,
) -> ContextHostNameResolution {
    if token.contains('[') || token.contains(']') {
        return ContextHostNameResolution::Unsupported("bracket_escaped_host_path_pending");
    }
    if token.contains('!') {
        return ContextHostNameResolution::Unsupported("cross_workspace_host_path_pending");
    }
    let segments = token
        .split('.')
        .filter(|segment| !segment.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();
    resolve_context_host_path_segments(&segments, owner_node_id, snapshot, meta_node_ids)
}

fn resolve_context_host_path_segments(
    segments: &[String],
    owner_node_id: TreeNodeId,
    snapshot: &StructuralSnapshot,
    meta_node_ids: &BTreeSet<TreeNodeId>,
) -> ContextHostNameResolution {
    if segments.is_empty() {
        return ContextHostNameResolution::Unresolved;
    }

    if snapshot
        .try_get_node(snapshot.root_node_id())
        .is_some_and(|root| root.symbol.eq_ignore_ascii_case(&segments[0]))
    {
        return try_resolve_visible_descendant_path(
            snapshot,
            meta_node_ids,
            snapshot.root_node_id(),
            &segments[1..],
        )
        .map_or(
            ContextHostNameResolution::Unresolved,
            ContextHostNameResolution::Resolved,
        );
    }

    let base =
        match resolve_context_walkup_symbol(&segments[0], owner_node_id, snapshot, meta_node_ids) {
            ContextHostNameResolution::Resolved(base_node_id) => base_node_id,
            other => return other,
        };
    if segments.len() == 1 {
        return ContextHostNameResolution::Resolved(base);
    }
    try_resolve_visible_descendant_path(snapshot, meta_node_ids, base, &segments[1..]).map_or(
        ContextHostNameResolution::Unresolved,
        ContextHostNameResolution::Resolved,
    )
}

fn resolve_context_walkup_symbol(
    symbol: &str,
    owner_node_id: TreeNodeId,
    snapshot: &StructuralSnapshot,
    meta_node_ids: &BTreeSet<TreeNodeId>,
) -> ContextHostNameResolution {
    let mut scope = Some(owner_node_id);
    while let Some(scope_node_id) = scope {
        match resolve_child_symbol_in_scope(symbol, scope_node_id, snapshot, meta_node_ids) {
            ContextHostNameResolution::Unresolved => {
                scope = snapshot.parent_id_of(scope_node_id);
            }
            other => return other,
        }
    }
    ContextHostNameResolution::Unresolved
}

fn resolve_child_symbol_in_scope(
    symbol: &str,
    scope_node_id: TreeNodeId,
    snapshot: &StructuralSnapshot,
    meta_node_ids: &BTreeSet<TreeNodeId>,
) -> ContextHostNameResolution {
    let Some(scope_node) = snapshot.try_get_node(scope_node_id) else {
        return ContextHostNameResolution::Unresolved;
    };
    let matches = scope_node
        .child_ids
        .iter()
        .copied()
        .filter(|child_id| {
            snapshot
                .try_get_node(*child_id)
                .is_some_and(|child| child.symbol.eq_ignore_ascii_case(symbol))
                && !is_meta_effective(*child_id, snapshot, meta_node_ids)
        })
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [] => ContextHostNameResolution::Unresolved,
        [node_id] => ContextHostNameResolution::Resolved(*node_id),
        _ => ContextHostNameResolution::Ambiguous,
    }
}

fn try_resolve_visible_descendant_path(
    snapshot: &StructuralSnapshot,
    meta_node_ids: &BTreeSet<TreeNodeId>,
    start_node_id: TreeNodeId,
    path_segments: &[String],
) -> Option<TreeNodeId> {
    let mut cursor = Some(start_node_id);
    for segment in path_segments {
        cursor = cursor.and_then(|current| {
            let parent = snapshot.try_get_node(current)?;
            parent.child_ids.iter().copied().find(|child_id| {
                snapshot
                    .try_get_node(*child_id)
                    .is_some_and(|child| child.symbol.eq_ignore_ascii_case(segment))
                    && !is_meta_effective(*child_id, snapshot, meta_node_ids)
            })
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
