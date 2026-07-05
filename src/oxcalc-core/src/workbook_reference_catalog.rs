#![forbid(unsafe_code)]

//! The workbook reference catalog: the `!`-qualifier routing surface (W062 D2
//! §4.1/§10).
//!
//! # What this module is (R3.2 — `calc-5kqg.21`)
//!
//! [`WorkbookReferenceCatalog`] is the single, snapshot-derived lookup surface
//! that routes a container qualifier (a sheet name) to the sheet's stable node
//! identity, its rename-immune [`SheetIdentityToken`], and its grid engine
//! handle. It is built by a **pure function over [`StructuralSnapshot`]**
//! ([`WorkbookReferenceCatalog::build`]) — it reads the D1 sheet registry
//! (`sheet_index`, C1), enumerates sheets in authored C3 order
//! (`sheet_nodes`), and reads each sheet node's grid backing for the engine
//! handle. It touches nothing outside the snapshot; the consumer's dependency
//! extraction and instantiation paths are **not** wired to it here.
//!
//! # What this module is NOT (scope split with R3.3)
//!
//! - **No resolution-behavior change.** This bead delivers the catalog *type*
//!   plus its lookup queries and tests. Nothing in bind, instantiation, or
//!   dependency extraction consults the catalog yet. That routing — and the
//!   name-keyed-dormant → token-keyed migration when a never-existed sheet is
//!   later created — is R3.3 (`calc-5kqg.22`).
//! - **No key-string adoption.** Existing normal-form/dependency key strings
//!   stay byte-for-byte stable (D2 §10). The [`SheetIdentityToken`] type is
//!   minted and exposed here, but no existing key string is rewritten to carry
//!   it; that adoption lands with R3.3's resolution wiring where D2 §10
//!   sequences it.
//! - **Existence-blind bind is preserved.** The catalog is a resolution-time
//!   fact: an unknown sheet name yields a name-keyed [`CatalogLookup::Dormant`]
//!   record (the `NameIdentity`-style dormant edge, D2 §4.1). Bind still never
//!   checks sheet existence.
//!
//! # Determinism
//!
//! [`WorkbookReferenceCatalog::build`] is a pure function of the snapshot: the
//! same snapshot (including one rebuilt from a serialized wire form) yields an
//! equal catalog, and enumeration order is the authored `child_ids`
//! (C3) order. This is what lets a rebind rebuild the routing table from a
//! snapshot with no hidden state.

use std::collections::BTreeMap;

use crate::reference_vocabulary::{NormalizedContainerName, SheetIdentityToken};
use crate::structural::{NormalizedSheetName, StructuralSnapshot, TreeNodeId};

/// The routed identity of a sheet the catalog resolved a qualifier to (D2
/// §4.1): the stable node id, its rename-immune [`SheetIdentityToken`], and the
/// grid engine handle the router hands to the coordination layer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SheetRouting {
    /// The resolved sheet node's stable identity.
    pub node_id: TreeNodeId,
    /// The rename-immune identity token minted from `node_id` (D2 §10). This is
    /// what a normal-form/dependency key's sheet component carries once R3.3
    /// adopts it; the display name is never the key component.
    pub token: SheetIdentityToken,
    /// The sheet's grid engine handle — the `grid_id` carried by the node's
    /// grid backing. `None` for a Sheet-role node that has no grid backing in
    /// this snapshot (e.g. a structural-only fixture). The value maps the
    /// resolved node to the grid engine that owns its cells (D2 §4.1).
    pub engine_handle: Option<String>,
}

/// The outcome of a catalog lookup for a container qualifier (D2 §4.1).
///
/// A known sheet routes to [`SheetRouting`]; an unknown name becomes a
/// **name-keyed dormant sheet-identity edge** — value semantics `#REF!` while
/// dormant, keyed on the normalized name so that if a sheet with that name is
/// later created, R3.3's heal-triggered rebind can migrate the record to the
/// token-keyed form. This bead only *reports* the dormant lookup; it performs
/// no migration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CatalogLookup {
    /// The qualifier named a sheet present in this workbook.
    Routed(SheetRouting),
    /// The qualifier named a sheet that does not currently exist; carries the
    /// normalized name for the dormant edge.
    Dormant { name: NormalizedContainerName },
}

impl CatalogLookup {
    /// The routing if this lookup resolved to a live sheet, else `None`.
    #[must_use]
    pub fn routed(&self) -> Option<&SheetRouting> {
        match self {
            Self::Routed(routing) => Some(routing),
            Self::Dormant { .. } => None,
        }
    }

    /// Whether the qualifier fell dormant (unknown sheet name).
    #[must_use]
    pub fn is_dormant(&self) -> bool {
        matches!(self, Self::Dormant { .. })
    }
}

/// A single catalog entry for a live sheet, carrying the routing plus the
/// normalized name it was indexed under (for rename re-indexing, D2 §4.1).
#[derive(Debug, Clone, PartialEq, Eq)]
struct SheetEntry {
    normalized_name: NormalizedSheetName,
    routing: SheetRouting,
}

/// The workbook reference catalog (D2 §4.1): the snapshot-derived router from a
/// sheet name (display or normalized) to the sheet's node id, identity token,
/// and grid engine handle.
///
/// Built purely from a [`StructuralSnapshot`] via [`Self::build`]. It carries no
/// engine or consumer references — it is a value that a resolver (R3.3)
/// consults, so it rebuilds deterministically from any snapshot, including one
/// rebuilt from its serialized wire form.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WorkbookReferenceCatalog {
    /// Normalized name → entry. Case-insensitive lookup lane (C1). Re-indexed
    /// on rename by rebuilding from the renamed snapshot; the node id and token
    /// inside the entry are rename-stable.
    by_normalized_name: BTreeMap<NormalizedSheetName, SheetEntry>,
    /// Identity token → node id. The token-keyed lane R3.3's resolution wiring
    /// routes through once keys adopt the token.
    node_by_token: BTreeMap<SheetIdentityToken, TreeNodeId>,
    /// Sheet identity tokens in authored (C3) order. Enumeration order matches
    /// [`StructuralSnapshot::sheet_nodes`], not the folded key order.
    tokens_in_order: Vec<SheetIdentityToken>,
}

impl WorkbookReferenceCatalog {
    /// Builds the catalog from a structural snapshot — a **pure function** over
    /// the D1 sheet registry (D2 §4.1).
    ///
    /// - Sheet set and per-sheet node id come from the registry (`sheet_index`,
    ///   C1); the case-insensitive lane keys on [`NormalizedSheetName`].
    /// - Enumeration order is authored `child_ids` (C3) order via
    ///   [`StructuralSnapshot::sheet_nodes`].
    /// - The identity token is minted from each sheet node's stable
    ///   [`TreeNodeId`] (D2 §10).
    /// - The engine handle is the `grid_id` of the node's grid backing, if any.
    ///
    /// No consumer or engine state is read; the result is fully determined by
    /// the snapshot, so a snapshot rebuilt from its wire form yields an equal
    /// catalog.
    #[must_use]
    pub fn build(snapshot: &StructuralSnapshot) -> Self {
        // Reverse the registry (name -> node) into a node -> name lane so that,
        // walking sheets in authored order, each node recovers the normalized
        // name it is registered under. There is exactly one name per node (the
        // registry rejects case-twins at construction), so this is a bijection.
        let name_by_node: BTreeMap<TreeNodeId, NormalizedSheetName> = snapshot
            .sheet_index()
            .iter()
            .map(|(name, node_id)| (*node_id, name.clone()))
            .collect();

        let mut by_normalized_name = BTreeMap::new();
        let mut node_by_token = BTreeMap::new();
        let mut tokens_in_order = Vec::new();

        for node_id in snapshot.sheet_nodes() {
            let Some(normalized_name) = name_by_node.get(&node_id).cloned() else {
                // A Sheet-role node not present in the registry would be a
                // D1 invariant violation; skip defensively rather than panic.
                continue;
            };
            let token = SheetIdentityToken::from_node_id(node_id);
            let engine_handle = snapshot
                .node_backings()
                .get(&node_id)
                .and_then(|backing| backing.as_grid())
                .map(|grid| grid.grid_id.clone());

            let routing = SheetRouting {
                node_id,
                token: token.clone(),
                engine_handle,
            };
            node_by_token.insert(token.clone(), node_id);
            tokens_in_order.push(token);
            by_normalized_name.insert(
                normalized_name.clone(),
                SheetEntry {
                    normalized_name,
                    routing,
                },
            );
        }

        Self {
            by_normalized_name,
            node_by_token,
            tokens_in_order,
        }
    }

    /// Routes a display sheet name to its routing (D2 §4.1). Case-insensitive:
    /// the name is folded to its [`NormalizedSheetName`] via the shared V3 fold
    /// before lookup. Unknown names fall to a name-keyed
    /// [`CatalogLookup::Dormant`].
    #[must_use]
    pub fn resolve_display_name(&self, display_name: &str) -> CatalogLookup {
        self.resolve_normalized(&NormalizedSheetName::from_symbol(display_name))
    }

    /// Routes an already-normalized sheet name to its routing (D2 §4.1).
    #[must_use]
    pub fn resolve_normalized(&self, normalized: &NormalizedSheetName) -> CatalogLookup {
        match self.by_normalized_name.get(normalized) {
            Some(entry) => CatalogLookup::Routed(entry.routing.clone()),
            None => CatalogLookup::Dormant {
                name: NormalizedContainerName::from_symbol(normalized.as_str()),
            },
        }
    }

    /// The identity token for a sheet node, if that node is a registered sheet
    /// in this catalog. Rename-immune: minted from the node id (D2 §10).
    #[must_use]
    pub fn token_for_node(&self, node_id: TreeNodeId) -> Option<SheetIdentityToken> {
        // The token is a pure function of the node id, but we only mint it for
        // nodes the catalog actually holds so callers cannot fabricate routes
        // to non-sheet nodes.
        self.tokens_in_order
            .iter()
            .find(|token| token.node_id() == Some(node_id))
            .cloned()
    }

    /// The node id a sheet identity token routes to, if the token is registered.
    #[must_use]
    pub fn node_for_token(&self, token: &SheetIdentityToken) -> Option<TreeNodeId> {
        self.node_by_token.get(token).copied()
    }

    /// The catalog's sheet identity tokens in authored (C3) order.
    #[must_use]
    pub fn tokens_in_order(&self) -> &[SheetIdentityToken] {
        &self.tokens_in_order
    }

    /// The number of registered sheets.
    #[must_use]
    pub fn len(&self) -> usize {
        self.tokens_in_order.len()
    }

    /// Whether the catalog holds no sheets (a plain tree snapshot).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.tokens_in_order.is_empty()
    }
}

/// The context-level workspace alias catalog (W062 D2 §5/§9): the **shared
/// seat** both external-workbook routing (`[Book2]Sheet1!A1`, §5, R3.7) and
/// cross-workspace `!` (`Workspace!Path`, §9, R3.6) resolve their container
/// alias against.
///
/// This bead (R3.2) delivers the seat only: a pure, deterministic map from a
/// normalized container alias to an opaque workspace identity. No resolution
/// path consults it yet — R3.6/R3.7 wire it. An unknown alias is a name-keyed
/// dormant fact, mirroring [`CatalogLookup::Dormant`], so unloaded/never-loaded
/// siblings follow the same dormant-edge discipline as unknown sheets.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WorkspaceAliasCatalog {
    /// Normalized alias → opaque workspace identity. Keyed on the shared V3
    /// fold so alias lookup is case-insensitive like every other container
    /// name lane.
    aliases: BTreeMap<NormalizedContainerName, String>,
}

/// The outcome of a workspace-alias lookup (D2 §5/§9).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceAliasLookup {
    /// The alias names a registered workspace; carries its opaque identity.
    Routed { workspace_id: String },
    /// The alias is not registered (workbook not loaded / workspace unknown);
    /// carries the normalized alias for the dormant edge.
    Dormant { alias: NormalizedContainerName },
}

impl WorkspaceAliasCatalog {
    /// An empty catalog (no sibling workspaces registered).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers (or re-points) an alias to a workspace identity. The alias is
    /// folded via the shared V3 fold before insertion.
    pub fn register_alias(&mut self, alias: &str, workspace_id: impl Into<String>) {
        self.aliases.insert(
            NormalizedContainerName::from_symbol(alias),
            workspace_id.into(),
        );
    }

    /// Removes an alias (workspace unloaded). Returns the identity it pointed
    /// to, if it was registered.
    pub fn unregister_alias(&mut self, alias: &str) -> Option<String> {
        self.aliases
            .remove(&NormalizedContainerName::from_symbol(alias))
    }

    /// Resolves a display alias, case-insensitively. Unknown aliases fall to a
    /// name-keyed [`WorkspaceAliasLookup::Dormant`].
    #[must_use]
    pub fn resolve_alias(&self, alias: &str) -> WorkspaceAliasLookup {
        let normalized = NormalizedContainerName::from_symbol(alias);
        match self.aliases.get(&normalized) {
            Some(workspace_id) => WorkspaceAliasLookup::Routed {
                workspace_id: workspace_id.clone(),
            },
            None => WorkspaceAliasLookup::Dormant { alias: normalized },
        }
    }

    /// The number of registered aliases.
    #[must_use]
    pub fn len(&self) -> usize {
        self.aliases.len()
    }

    /// Whether no aliases are registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.aliases.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::structural::{
        NodeBacking, StructuralGridShape, StructuralNode, StructuralNodeKind, StructuralSnapshot,
        StructuralSnapshotId, NodeRole,
    };

    fn sheet_node(node_id: u64, symbol: &str) -> StructuralNode {
        StructuralNode {
            node_id: TreeNodeId(node_id),
            kind: StructuralNodeKind::Container,
            symbol: symbol.to_string(),
            parent_id: Some(TreeNodeId(1)),
            child_ids: Vec::new(),
            role: Some(NodeRole::Sheet),
            is_meta: false,
        }
    }

    fn grid_backing(grid_id: &str, sheet_name: &str) -> NodeBacking {
        NodeBacking::Grid(StructuralGridShape {
            grid_id: grid_id.to_string(),
            sheet_name: sheet_name.to_string(),
            bounds_identity: "b".to_string(),
            cell_population_version: "c".to_string(),
            axis_state_version: "a".to_string(),
            overlay_set_version: "o".to_string(),
            merged_region_version: "m".to_string(),
        })
    }

    /// A 3-sheet workbook fixture with grid backings, in authored order
    /// Sheet1, Data, Summary → node ids 2, 3, 4.
    fn three_sheet_workbook() -> StructuralSnapshot {
        let root = StructuralNode {
            node_id: TreeNodeId(1),
            kind: StructuralNodeKind::Root,
            symbol: "Book".to_string(),
            parent_id: None,
            child_ids: vec![TreeNodeId(2), TreeNodeId(3), TreeNodeId(4)],
            role: Some(NodeRole::Workbook),
            is_meta: false,
        };
        let mut backings = BTreeMap::new();
        backings.insert(TreeNodeId(2), grid_backing("grid-2", "Sheet1"));
        backings.insert(TreeNodeId(3), grid_backing("grid-3", "Data"));
        backings.insert(TreeNodeId(4), grid_backing("grid-4", "Summary"));

        StructuralSnapshot::create_with_node_backings(
            StructuralSnapshotId(1),
            TreeNodeId(1),
            [
                root,
                sheet_node(2, "Sheet1"),
                sheet_node(3, "Data"),
                sheet_node(4, "Summary"),
            ],
            backings,
        )
        .unwrap()
    }

    // --- Acceptance: name -> node -> engine across a 3-sheet workbook -----

    #[test]
    fn catalog_routes_name_to_node_to_engine_across_three_sheets() {
        let snapshot = three_sheet_workbook();
        let catalog = WorkbookReferenceCatalog::build(&snapshot);

        assert_eq!(catalog.len(), 3);

        let sheet1 = catalog.resolve_display_name("Sheet1");
        let routing = sheet1.routed().expect("Sheet1 routes");
        assert_eq!(routing.node_id, TreeNodeId(2));
        assert_eq!(routing.token.as_str(), "sheet-node:2");
        assert_eq!(routing.engine_handle.as_deref(), Some("grid-2"));

        let data = catalog.resolve_display_name("Data");
        let routing = data.routed().expect("Data routes");
        assert_eq!(routing.node_id, TreeNodeId(3));
        assert_eq!(routing.engine_handle.as_deref(), Some("grid-3"));

        let summary = catalog.resolve_display_name("Summary");
        let routing = summary.routed().expect("Summary routes");
        assert_eq!(routing.node_id, TreeNodeId(4));
        assert_eq!(routing.engine_handle.as_deref(), Some("grid-4"));
    }

    #[test]
    fn catalog_lookup_is_case_insensitive_via_shared_fold() {
        let snapshot = three_sheet_workbook();
        let catalog = WorkbookReferenceCatalog::build(&snapshot);

        for name in ["Sheet1", "SHEET1", "sheet1"] {
            let routing = catalog
                .resolve_display_name(name)
                .routed()
                .cloned()
                .unwrap_or_else(|| panic!("{name} routes"));
            assert_eq!(routing.node_id, TreeNodeId(2));
        }
    }

    #[test]
    fn unknown_sheet_name_falls_to_name_keyed_dormant_edge() {
        let snapshot = three_sheet_workbook();
        let catalog = WorkbookReferenceCatalog::build(&snapshot);

        let lookup = catalog.resolve_display_name("Ghost");
        assert!(lookup.is_dormant());
        assert!(lookup.routed().is_none());
        match lookup {
            CatalogLookup::Dormant { name } => {
                // The dormant edge is keyed on the normalized name (folded).
                assert_eq!(name, NormalizedContainerName::from_symbol("ghost"));
            }
            CatalogLookup::Routed(_) => unreachable!(),
        }
    }

    #[test]
    fn enumeration_is_authored_c3_order_not_folded_key_order() {
        // Authored order Zebra, Alpha, Middle => tokens in that order, distinct
        // from the folded map order (alpha, middle, zebra).
        let root = StructuralNode {
            node_id: TreeNodeId(1),
            kind: StructuralNodeKind::Root,
            symbol: "Book".to_string(),
            parent_id: None,
            child_ids: vec![TreeNodeId(2), TreeNodeId(3), TreeNodeId(4)],
            role: Some(NodeRole::Workbook),
            is_meta: false,
        };
        let snapshot = StructuralSnapshot::create(
            StructuralSnapshotId(1),
            TreeNodeId(1),
            [
                root,
                sheet_node(2, "Zebra"),
                sheet_node(3, "Alpha"),
                sheet_node(4, "Middle"),
            ],
        )
        .unwrap();
        let catalog = WorkbookReferenceCatalog::build(&snapshot);

        let order: Vec<&str> = catalog
            .tokens_in_order()
            .iter()
            .map(SheetIdentityToken::as_str)
            .collect();
        assert_eq!(order, vec!["sheet-node:2", "sheet-node:3", "sheet-node:4"]);
    }

    // --- Token queries ----------------------------------------------------

    #[test]
    fn token_for_node_and_node_for_token_round_trip() {
        let snapshot = three_sheet_workbook();
        let catalog = WorkbookReferenceCatalog::build(&snapshot);

        let token = catalog.token_for_node(TreeNodeId(3)).expect("registered");
        assert_eq!(token.as_str(), "sheet-node:3");
        assert_eq!(catalog.node_for_token(&token), Some(TreeNodeId(3)));

        // A non-sheet node is not routable.
        assert_eq!(catalog.token_for_node(TreeNodeId(1)), None);
        // A fabricated token for a non-registered node does not resolve.
        let bogus = SheetIdentityToken::from_node_id(TreeNodeId(99));
        assert_eq!(catalog.node_for_token(&bogus), None);
    }

    // --- Acceptance: rename changes no keys and no edges (property) -------

    #[test]
    fn rename_keeps_token_node_and_edges_stable_only_render_name_changes() {
        let snapshot = three_sheet_workbook();
        let before = WorkbookReferenceCatalog::build(&snapshot);
        let token_before = before.token_for_node(TreeNodeId(3)).unwrap();
        let engine_before = before
            .resolve_display_name("Data")
            .routed()
            .unwrap()
            .engine_handle
            .clone();

        // Perform an ACTUAL rename of the middle sheet Data -> Renamed.
        let outcome = snapshot
            .apply_edit(
                StructuralSnapshotId(2),
                crate::structural::StructuralEdit::RenameNode {
                    node_id: TreeNodeId(3),
                    new_symbol: "Renamed".to_string(),
                },
            )
            .unwrap();
        let after = WorkbookReferenceCatalog::build(&outcome.snapshot);

        // The identity token is unchanged (rename-immune, minted from node id).
        let token_after = after.token_for_node(TreeNodeId(3)).unwrap();
        assert_eq!(token_before, token_after);

        // Token -> node routing (the "edge") is unchanged.
        assert_eq!(
            before.node_for_token(&token_before),
            after.node_for_token(&token_after)
        );
        assert_eq!(after.node_for_token(&token_after), Some(TreeNodeId(3)));

        // The engine handle behind the token is unchanged.
        let engine_after = after
            .resolve_display_name("Renamed")
            .routed()
            .unwrap()
            .engine_handle
            .clone();
        assert_eq!(engine_before, engine_after);

        // Only the render/display lane changed: the OLD display name no longer
        // resolves, the NEW one resolves to the SAME node id + token.
        assert!(after.resolve_display_name("Data").is_dormant());
        let renamed = after.resolve_display_name("Renamed");
        let routing = renamed.routed().unwrap();
        assert_eq!(routing.node_id, TreeNodeId(3));
        assert_eq!(routing.token, token_before);
    }

    // --- Determinism: catalog rebuilds from a snapshot deterministically --

    #[test]
    fn catalog_rebuilds_equally_from_serialized_snapshot() {
        let snapshot = three_sheet_workbook();
        let built = WorkbookReferenceCatalog::build(&snapshot);

        // Round-trip the snapshot through its wire form (derived indexes rebuilt
        // on load) and rebuild the catalog: it must be equal.
        let json = serde_json::to_string(&snapshot).unwrap();
        let round: StructuralSnapshot = serde_json::from_str(&json).unwrap();
        let rebuilt = WorkbookReferenceCatalog::build(&round);

        assert_eq!(built, rebuilt);
    }

    #[test]
    fn build_is_deterministic_for_equal_snapshots() {
        let a = WorkbookReferenceCatalog::build(&three_sheet_workbook());
        let b = WorkbookReferenceCatalog::build(&three_sheet_workbook());
        assert_eq!(a, b);
    }

    // --- Plain tree snapshot has an empty catalog -------------------------

    #[test]
    fn plain_tree_snapshot_yields_empty_catalog() {
        let root = StructuralNode {
            node_id: TreeNodeId(1),
            kind: StructuralNodeKind::Root,
            symbol: "Root".to_string(),
            parent_id: None,
            child_ids: vec![TreeNodeId(2)],
            role: None,
            is_meta: false,
        };
        let child = StructuralNode {
            node_id: TreeNodeId(2),
            kind: StructuralNodeKind::Container,
            symbol: "Branch".to_string(),
            parent_id: Some(TreeNodeId(1)),
            child_ids: Vec::new(),
            role: None,
            is_meta: false,
        };
        let snapshot =
            StructuralSnapshot::create(StructuralSnapshotId(1), TreeNodeId(1), [root, child])
                .unwrap();
        let catalog = WorkbookReferenceCatalog::build(&snapshot);
        assert!(catalog.is_empty());
        assert_eq!(catalog.len(), 0);
        assert!(catalog.resolve_display_name("Branch").is_dormant());
    }

    #[test]
    fn sheet_without_grid_backing_routes_with_no_engine_handle() {
        // A Sheet-role node with no grid backing still routes (node + token),
        // but carries no engine handle.
        let root = StructuralNode {
            node_id: TreeNodeId(1),
            kind: StructuralNodeKind::Root,
            symbol: "Book".to_string(),
            parent_id: None,
            child_ids: vec![TreeNodeId(2)],
            role: Some(NodeRole::Workbook),
            is_meta: false,
        };
        let snapshot = StructuralSnapshot::create(
            StructuralSnapshotId(1),
            TreeNodeId(1),
            [root, sheet_node(2, "Bare")],
        )
        .unwrap();
        let catalog = WorkbookReferenceCatalog::build(&snapshot);
        let routing = catalog.resolve_display_name("Bare").routed().cloned().unwrap();
        assert_eq!(routing.node_id, TreeNodeId(2));
        assert_eq!(routing.token.as_str(), "sheet-node:2");
        assert_eq!(routing.engine_handle, None);
    }

    // --- Workspace alias catalog: the shared §5/§9 seat --------------------

    #[test]
    fn workspace_alias_catalog_routes_case_insensitively() {
        let mut aliases = WorkspaceAliasCatalog::new();
        assert!(aliases.is_empty());
        aliases.register_alias("Book2", "workspace:book2");
        assert_eq!(aliases.len(), 1);

        for name in ["Book2", "BOOK2", "book2"] {
            assert_eq!(
                aliases.resolve_alias(name),
                WorkspaceAliasLookup::Routed {
                    workspace_id: "workspace:book2".to_string()
                },
                "alias {name} must route via the shared fold"
            );
        }
    }

    #[test]
    fn unknown_workspace_alias_falls_to_name_keyed_dormant() {
        let aliases = WorkspaceAliasCatalog::new();
        assert_eq!(
            aliases.resolve_alias("Missing"),
            WorkspaceAliasLookup::Dormant {
                alias: NormalizedContainerName::from_symbol("missing")
            }
        );
    }

    #[test]
    fn unregistering_workspace_alias_returns_identity_and_goes_dormant() {
        let mut aliases = WorkspaceAliasCatalog::new();
        aliases.register_alias("Book2", "workspace:book2");

        // Unload: the identity is returned, subsequent lookups go dormant
        // (heal-on-load is R3.7's wiring; the seat only reports the state).
        assert_eq!(
            aliases.unregister_alias("book2"),
            Some("workspace:book2".to_string())
        );
        assert!(aliases.is_empty());
        assert!(matches!(
            aliases.resolve_alias("Book2"),
            WorkspaceAliasLookup::Dormant { .. }
        ));
        // Unregistering an unknown alias is a no-op None.
        assert_eq!(aliases.unregister_alias("Book2"), None);
    }
}
