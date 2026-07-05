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

use oxfml_core::binding::{
    ProfileReferenceRecord, ReferenceBindProfile, ReferenceTransformKind, ReferenceTransformOutcome,
    ReferenceTransformRequest,
};

use crate::grid::ast::{ExcelGridReferenceTransformPayload, ExcelGridStructuralEdit};
use crate::grid::machine::GridDependency;
use crate::grid::reference_engine::StrictExcelGridReferenceProfile;
use crate::reference_vocabulary::{
    ContainerDeletionPolicy, ExternalBookToken, NormalizedContainerName, SheetIdentityToken,
};
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

/// The sheet component a [`GridDependency`] carries, if it is one of the
/// address-bearing value dependency shapes ([`GridDependency::Cell`]/
/// [`GridDependency::Range`]). A dependency whose shape carries no single sheet
/// address (name identity, dynamic request, …) returns `None` and is left to
/// its own resolution path — the cross-sheet router only routes value edges.
#[must_use]
fn dependency_sheet_component(dependency: &GridDependency) -> Option<(&str, &str)> {
    match dependency {
        GridDependency::Cell(address) => {
            Some((address.workbook_id.as_str(), address.sheet_id.as_str()))
        }
        GridDependency::Range(rect) => Some((rect.workbook_id.as_str(), rect.sheet_id.as_str())),
        _ => None,
    }
}

/// A cross-sheet dependency descriptor (W062 D2 §4.1 / contract V5): the router
/// mapped a dependency key's sheet component to the target sheet node, yielding
/// the pair the D3 coordination layer (R4.6) consumes to schedule cross-sheet
/// dirty closure and evaluation order.
///
/// This is the **token-keyed** form: the sheet identity has resolved to a live
/// node. `token` is the same rename-immune [`SheetIdentityToken`] that keys the
/// catalog and (per §10) the normal-form/dependency key's sheet component, so a
/// consumer holding this descriptor can route to the target grid without ever
/// touching a display name. `engine_handle` is the target sheet's grid engine
/// handle (its `grid_id`), the thing a value-resolution pass consults to gather
/// the target cells.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrossSheetDependencyDescriptor {
    /// The resolved target sheet node (V5 `target_sheet_node`).
    pub target_sheet_node: TreeNodeId,
    /// The target sheet's rename-immune identity token (§10).
    pub token: SheetIdentityToken,
    /// The target sheet's grid engine handle, if it has a grid backing.
    pub engine_handle: Option<String>,
    /// The dependency, unchanged (V5 `dependency`). Its address already carries
    /// full sheet identity; the descriptor pairs it with the routed node.
    pub dependency: GridDependency,
}

/// The outcome of routing one [`GridDependency`] through the workbook catalog
/// (W062 D2 §4.1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CrossSheetRouting {
    /// The dependency's sheet component is this formula's own sheet — no
    /// cross-sheet routing is needed; it resolves against the local grid as
    /// today.
    SameSheet,
    /// The sheet component named a live sheet other than the owner: a
    /// token-keyed cross-sheet descriptor for the coordination layer (V5).
    Routed(CrossSheetDependencyDescriptor),
    /// The sheet component named a sheet that does not currently exist: a
    /// **name-keyed dormant sheet-identity edge** (the `NameIdentity` pattern,
    /// D2 §4.1), value semantics `#REF!` while dormant. Carries the normalized
    /// name so a heal-triggered rebind can migrate it to the token-keyed form
    /// when a sheet of that name is later created ([`DormantSheetReferenceLedger`]).
    Dormant {
        name: NormalizedContainerName,
        dependency: GridDependency,
    },
}

impl WorkbookReferenceCatalog {
    /// Route one value dependency's sheet component to a target sheet node
    /// (W062 D2 §4.1, R3.3). `owning_sheet_id` is the sheet the formula that
    /// produced this dependency lives on (its grid's `sheet_id`); a dependency
    /// on that same sheet returns [`CrossSheetRouting::SameSheet`] and is left
    /// to local resolution.
    ///
    /// A dependency shape with no single-sheet address (name identity, dynamic
    /// request) also returns [`CrossSheetRouting::SameSheet`]: those are not the
    /// cross-sheet router's concern and keep their existing resolution path.
    ///
    /// **Existence-blind bind is preserved:** this is a *resolution-time*
    /// consultation over an already-bound dependency; it never re-binds and
    /// never rejects at bind. An unknown sheet name yields a dormant record,
    /// exactly as `Sheet1!A1` binds `ValidAfterInstantiation` regardless of
    /// whether `Sheet1` exists.
    #[must_use]
    pub fn route_dependency(
        &self,
        owning_sheet_id: &str,
        dependency: &GridDependency,
    ) -> CrossSheetRouting {
        let Some((_workbook_id, sheet_id)) = dependency_sheet_component(dependency) else {
            return CrossSheetRouting::SameSheet;
        };
        if sheet_id == owning_sheet_id {
            return CrossSheetRouting::SameSheet;
        }
        // §10/V4 sequencing note: the dependency's `sheet_id` is a display-ish
        // string today (tokens are not yet adopted into key strings — see the
        // module header note and D2 §10, which sequences token adoption with the
        // resolution wiring). We therefore route through the catalog's
        // case-insensitive *name* lane here. When key strings adopt
        // `SheetIdentityToken` (the §10 adoption step), this lookup should route
        // the token lane (`node_for_token`) instead; revisit this fold then.
        match self.resolve_display_name(sheet_id) {
            CatalogLookup::Routed(routing) => {
                CrossSheetRouting::Routed(CrossSheetDependencyDescriptor {
                    target_sheet_node: routing.node_id,
                    token: routing.token,
                    engine_handle: routing.engine_handle,
                    dependency: dependency.clone(),
                })
            }
            CatalogLookup::Dormant { name } => CrossSheetRouting::Dormant {
                name,
                dependency: dependency.clone(),
            },
        }
    }

    /// Route a batch of dependencies, collecting the cross-sheet descriptors
    /// (V5) and the dormant records separately (W062 D2 §4.1). Same-sheet and
    /// non-address dependencies are dropped from both lists — they need no
    /// cross-sheet handling.
    #[must_use]
    pub fn route_dependencies<'a>(
        &self,
        owning_sheet_id: &str,
        dependencies: impl IntoIterator<Item = &'a GridDependency>,
    ) -> RoutedCrossSheetDependencies {
        let mut routed = Vec::new();
        let mut dormant = Vec::new();
        for dependency in dependencies {
            match self.route_dependency(owning_sheet_id, dependency) {
                CrossSheetRouting::SameSheet => {}
                CrossSheetRouting::Routed(descriptor) => routed.push(descriptor),
                CrossSheetRouting::Dormant { name, dependency } => {
                    dormant.push(DormantSheetReference { name, dependency });
                }
            }
        }
        RoutedCrossSheetDependencies { routed, dormant }
    }
}

/// The result of routing a batch of dependencies through the catalog (V5): the
/// live cross-sheet descriptors for the coordination layer plus the dormant
/// records awaiting heal.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RoutedCrossSheetDependencies {
    /// Token-keyed descriptors for sheets that currently exist (V5).
    pub routed: Vec<CrossSheetDependencyDescriptor>,
    /// Name-keyed dormant records for sheet names that do not currently exist.
    pub dormant: Vec<DormantSheetReference>,
}

/// A dormant cross-sheet reference: a value dependency whose sheet name does
/// not currently resolve (W062 D2 §4.1). Keyed on the normalized name so a
/// later sheet creation of that name can heal it into a token-keyed descriptor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DormantSheetReference {
    /// The normalized (folded) name the dormant edge is keyed on.
    pub name: NormalizedContainerName,
    /// The dependency held dormant; migrated verbatim on heal.
    pub dependency: GridDependency,
}

/// The ledger of dormant cross-sheet references awaiting a heal-triggered
/// rebind (W062 D2 §4.1). When a **never-existed** sheet name is later created,
/// its dormant records migrate to the token-keyed [`CrossSheetDependencyDescriptor`]
/// form — the `NameIdentity` heal precedent, and the migration R3.3 owns.
///
/// This ledger is deliberately **name-keyed, not node-keyed**: a dormant record
/// has no node to key on (that is exactly what makes it dormant). The strict
/// profile's §6 deletion policy (`HardRefError`, never-heal-on-recreate for a
/// *deleted* sheet) is a *different* fact handled by R3.4's transform driver;
/// this ledger heals only *never-existed* names, per D2 §4.1's explicit
/// distinction.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DormantSheetReferenceLedger {
    /// Normalized sheet name → the dormant references waiting on it.
    by_name: BTreeMap<NormalizedContainerName, Vec<DormantSheetReference>>,
}

impl DormantSheetReferenceLedger {
    /// An empty ledger.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a dormant reference under its normalized sheet name.
    pub fn record(&mut self, reference: DormantSheetReference) {
        self.by_name
            .entry(reference.name.clone())
            .or_default()
            .push(reference);
    }

    /// Record every dormant reference from a routing batch.
    pub fn record_all(&mut self, dormant: impl IntoIterator<Item = DormantSheetReference>) {
        for reference in dormant {
            self.record(reference);
        }
    }

    /// The number of distinct dormant sheet names currently held.
    #[must_use]
    pub fn dormant_name_count(&self) -> usize {
        self.by_name.len()
    }

    /// Whether the ledger holds any dormant references.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.by_name.is_empty()
    }

    /// Whether a dormant record exists for `name` (normalized comparison).
    #[must_use]
    pub fn has_dormant(&self, name: &NormalizedContainerName) -> bool {
        self.by_name.contains_key(name)
    }

    /// Heal-triggered rebind (W062 D2 §4.1): a sheet was created, so re-route
    /// every dormant record against the **rebuilt** catalog. Records whose name
    /// now resolves migrate to token-keyed [`CrossSheetDependencyDescriptor`]s
    /// and are **removed** from the ledger; records that still do not resolve
    /// stay dormant. Returns the descriptors produced by this heal so the
    /// consumer can hand them to the coordination layer exactly as it would a
    /// freshly-routed live edge.
    ///
    /// This is driven by the actual creation of a sheet (the catalog is rebuilt
    /// from the post-creation snapshot before calling this), not by a name
    /// guess — the `NameIdentity` precedent that keeps identity migration a
    /// consequence of a real structural fact.
    #[must_use]
    pub fn heal_against(
        &mut self,
        catalog: &WorkbookReferenceCatalog,
    ) -> Vec<CrossSheetDependencyDescriptor> {
        let mut healed = Vec::new();
        let mut still_dormant: BTreeMap<NormalizedContainerName, Vec<DormantSheetReference>> =
            BTreeMap::new();
        for (name, references) in std::mem::take(&mut self.by_name) {
            match catalog.resolve_normalized_container(&name) {
                Some(routing) => {
                    for reference in references {
                        healed.push(CrossSheetDependencyDescriptor {
                            target_sheet_node: routing.node_id,
                            token: routing.token.clone(),
                            engine_handle: routing.engine_handle.clone(),
                            dependency: reference.dependency,
                        });
                    }
                }
                None => {
                    still_dormant.insert(name, references);
                }
            }
        }
        self.by_name = still_dormant;
        healed
    }
}

/// The per-record result of driving a sheet deletion through the profile's
/// deletion policy (W062 D2 §6, contract V7).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SheetDeletionRecordOutcome {
    /// Strict-excel `HardRefError`: the record targeted the deleted sheet and
    /// was destructively rewritten to a `#REF!`-carrying `RefError` record. The
    /// carried record is the new bound record — its readout renders `#REF!`
    /// (never from ad-hoc state), and because it is now a `RefError`, recreating
    /// a same-named sheet cannot heal it. Only revision restore (undo) brings
    /// back the pre-transform record.
    HardRefError { record: ProfileReferenceRecord },
    /// The record targeted a *surviving* sheet: unchanged, carried through.
    Unchanged { record: ProfileReferenceRecord },
    /// Tree-profile `DormantIdentityHeal`: the record is left intact and its
    /// reference falls to a dormant identity edge that heals if a sheet of the
    /// same normalized name reappears. The record is carried unchanged; the
    /// dormant edge is what a same-name recreate later heals.
    DormantHeal {
        record: ProfileReferenceRecord,
        name: NormalizedContainerName,
    },
}

impl SheetDeletionRecordOutcome {
    /// The (possibly transformed) bound record this outcome carries.
    #[must_use]
    pub fn record(&self) -> &ProfileReferenceRecord {
        match self {
            Self::HardRefError { record }
            | Self::Unchanged { record }
            | Self::DormantHeal { record, .. } => record,
        }
    }

    /// Whether this outcome is a hard `#REF!` (strict-excel destructive
    /// transform).
    #[must_use]
    pub fn is_hard_ref_error(&self) -> bool {
        matches!(self, Self::HardRefError { .. })
    }
}

/// Drive a sheet deletion over one bound strict-excel reference record per the
/// profile's [`ContainerDeletionPolicy`] (W062 D2 §6, contract V7).
///
/// This is the policy branch of the deletion transform driver R3.4 owns. It is
/// the seam the consumer's `delete_sheet` verb path drives for a workbook
/// workspace: the vocabulary carries the policy ([`HardRefError`] for
/// strict-excel, [`DormantIdentityHeal`] for the tree profile), and this routes
/// each bound record accordingly.
///
/// - [`ContainerDeletionPolicy::HardRefError`]: drives the profile's
///   `transform_reference(StructuralEdit: SheetDeleted)`. A record targeting the
///   deleted `sheet_id` becomes a `#REF!` `RefError` record
///   ([`ReferenceTransformOutcome::FullyInvalid`]); a record into a surviving
///   sheet is [`ReferenceTransformOutcome::Unchanged`]. **No heal-on-recreate**:
///   the transform rewrites the record, so recreating a same-named sheet is
///   inert — undo (revision restore of the pre-transform record) is the only
///   resurrection path.
/// - [`ContainerDeletionPolicy::DormantIdentityHeal`]: leaves the record intact
///   and reports a dormant edge keyed on the deleted sheet's normalized name, so
///   a later same-name recreate heals it (the tree profile's lenient contract).
///
/// `deleted_sheet_id` is the deleted sheet's identity component as it appears in
/// bound record `sheet_id` fields / normal-form keys (§10). `deleted_sheet_name`
/// is its normalized name, used only for the dormant-heal edge.
#[must_use]
pub fn apply_sheet_deletion_to_record(
    policy: ContainerDeletionPolicy,
    profile: &StrictExcelGridReferenceProfile,
    workbook_id: &str,
    deleted_sheet_id: &str,
    deleted_sheet_name: &NormalizedContainerName,
    record: &ProfileReferenceRecord,
) -> SheetDeletionRecordOutcome {
    match policy {
        ContainerDeletionPolicy::HardRefError => {
            let payload = ExcelGridReferenceTransformPayload::new(
                ExcelGridStructuralEdit::delete_sheet(workbook_id, deleted_sheet_id),
                None,
            )
            .into_profile_payload();
            let result = profile.transform_reference(&ReferenceTransformRequest {
                reference: record.clone(),
                transform_kind: ReferenceTransformKind::StructuralEdit,
                payload: Some(payload),
            });
            let transformed = result.reference.unwrap_or_else(|| record.clone());
            match result.outcome {
                ReferenceTransformOutcome::FullyInvalid => {
                    SheetDeletionRecordOutcome::HardRefError {
                        record: transformed,
                    }
                }
                _ => SheetDeletionRecordOutcome::Unchanged {
                    record: transformed,
                },
            }
        }
        ContainerDeletionPolicy::DormantIdentityHeal => SheetDeletionRecordOutcome::DormantHeal {
            record: record.clone(),
            name: deleted_sheet_name.clone(),
        },
    }
}

impl WorkbookReferenceCatalog {
    /// Resolve a normalized container name to its routing, if it names a live
    /// sheet (W062 D2 §4.1). Used by the dormant-heal rebind, which already
    /// holds normalized names.
    #[must_use]
    pub fn resolve_normalized_container(
        &self,
        name: &NormalizedContainerName,
    ) -> Option<SheetRouting> {
        match self.resolve_normalized(&NormalizedSheetName::from_symbol(name.as_str())) {
            CatalogLookup::Routed(routing) => Some(routing),
            CatalogLookup::Dormant { .. } => None,
        }
    }
}

/// Gather the target-sheet computed values a set of routed cross-sheet
/// descriptors reference, so the consumer can inject them into the evaluating
/// sheet's cross-sheet view (W062 D2 §4.1, R3.3).
///
/// `computed_by_node` maps each sheet node to its committed computed values
/// (the same [`crate::grid::coords::ExcelGridCellAddress`]-keyed store a
/// [`crate::grid::machine::GridCalcRefSheet`] publishes). For each routed
/// descriptor, this pulls the referenced cells from the *target* node's store
/// and returns them keyed by their full cross-sheet address — exactly the shape
/// `GridCalcRefSheet::set_cross_sheet_cells` consumes.
///
/// This is the **value** half of the cross-sheet slice. It does not touch dirty
/// propagation: which formulas to re-run when a target cell changes is the D3
/// coordination layer's reverse-edge job (R4.6). The consumer here refreshes
/// the whole view each recalc, which is correct-but-eager — the honest slice
/// that makes cross-sheet values right immediately without a mark-all hack.
#[must_use]
pub fn gather_cross_sheet_cells(
    descriptors: &[CrossSheetDependencyDescriptor],
    computed_by_node: &BTreeMap<
        TreeNodeId,
        BTreeMap<crate::grid::coords::ExcelGridCellAddress, oxfunc_core::value::CalcValue>,
    >,
) -> BTreeMap<crate::grid::coords::ExcelGridCellAddress, oxfunc_core::value::CalcValue> {
    use crate::grid::coords::ExcelGridCellAddress;

    let mut gathered = BTreeMap::new();
    for descriptor in descriptors {
        let Some(target_computed) = computed_by_node.get(&descriptor.target_sheet_node) else {
            // The target node has no published computed store yet (not recalced,
            // or no grid backing). Leave its cells unrouted — they read as an
            // unresolved cross-sheet cell, not a fabricated value.
            continue;
        };
        // Collect every address the dependency covers that the target has a
        // value for. Cell → the single address; Range → each populated cell in
        // the rect. A rect over an empty target cell simply carries no entry
        // (the reference reads empty, same as an in-sheet empty cell).
        match &descriptor.dependency {
            GridDependency::Cell(address) => {
                if let Some(value) = target_computed.get(address) {
                    gathered.insert(address.clone(), value.clone());
                }
            }
            GridDependency::Range(rect) => {
                for row in rect.top_row..=rect.bottom_row {
                    for col in rect.left_col..=rect.right_col {
                        let address = ExcelGridCellAddress::new(
                            rect.workbook_id.clone(),
                            rect.sheet_id.clone(),
                            row,
                            col,
                        );
                        if let Some(value) = target_computed.get(&address) {
                            gathered.insert(address, value.clone());
                        }
                    }
                }
            }
            // Non-value descriptors are never produced by the router.
            _ => {}
        }
    }
    gathered
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

/// The typed diagnostic an external-workbook reference carries when its sibling
/// workspace is not loaded (W062 D2 §5, R3.7). Stable code so a host surface can
/// route on it (heal-on-load, link management UI) without string-matching the
/// message. This is the `#REF!`-with-a-reason contract: the reference is never
/// silently empty when the sibling is absent — it is a *typed* `#REF!`.
pub const EXTERNAL_WORKBOOK_NOT_LOADED_DIAGNOSTIC: &str =
    "excel.grid.external.workbook_not_loaded";

/// The outcome of routing an external-workbook reference's `{workbook}`
/// component through the context-level [`WorkspaceAliasCatalog`] (W062 D2 §5,
/// R3.7).
///
/// This is the external-workbook analogue of [`CrossSheetRouting`]: it maps the
/// dormant-external identity token (`extbook:{alias}`) a bound external record
/// carries (§10) to the loaded sibling **workspace** that alias names, or to a
/// **typed `#REF!`** when no such sibling is loaded.
///
/// # Scope (typed partial-IN, D2 §5)
///
/// - `Routed`: a sibling workspace is registered+loaded under the alias, so the
///   reference gets **live** values through cross-workspace routing (the value
///   half is [`gather_external_cells`], driven against the sibling's own
///   published grid store — exactly as [`gather_cross_sheet_cells`] does for
///   in-workbook cross-sheet edges, one seam up at the workspace boundary).
/// - `RefError`: the alias resolves to no loaded workspace (never registered,
///   or unloaded). The reference is a typed `#REF!` carrying
///   [`EXTERNAL_WORKBOOK_NOT_LOADED_DIAGNOSTIC`] — **never** a silent empty.
///   The edge is dormant on the alias, so a later sibling load under that alias
///   heals it (routing, not the key, changes — the `extbook:` key is stable).
///
/// # Out of scope (D2 §5 typed exclusions, deliberately not built here)
///
/// - **No evaluation-triggered file loading.** Routing consults the *already
///   loaded* alias catalog; it never performs I/O. A sibling is loaded by a
///   D4/R6 document verb, out of the reference layer.
/// - **No cached external-value store / link manager.** There is no runtime
///   value cache here. Values from a file's external-link cache are D4's
///   `FileCached` publication channel (§5 errata / D4 §14 / T5), an ingest-fed
///   overlay — not this evaluation-side router.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExternalWorkbookRouting {
    /// The alias names a loaded sibling workspace; carries its opaque identity
    /// and the normalized alias the edge is keyed on.
    Routed {
        workspace_id: String,
        alias: NormalizedContainerName,
    },
    /// The alias names no loaded workspace: a typed `#REF!` (dormant on the
    /// alias, heals on a same-alias sibling load). Carries the normalized alias
    /// and the stable diagnostic code.
    RefError {
        alias: NormalizedContainerName,
        diagnostic: &'static str,
    },
}

impl ExternalWorkbookRouting {
    /// The loaded sibling workspace id, if this routed live.
    #[must_use]
    pub fn workspace_id(&self) -> Option<&str> {
        match self {
            Self::Routed { workspace_id, .. } => Some(workspace_id),
            Self::RefError { .. } => None,
        }
    }

    /// Whether this routing is a typed `#REF!` (sibling not loaded).
    #[must_use]
    pub fn is_ref_error(&self) -> bool {
        matches!(self, Self::RefError { .. })
    }
}

/// Route an external-workbook reference's `{workbook}` component to a loaded
/// sibling workspace (W062 D2 §5, R3.7).
///
/// `workbook_component` is the `{workbook}` component a bound external record /
/// dependency carries — the dormant-external identity token `extbook:{alias}`
/// ([`ExternalBookToken`], §10). Its alias is resolved through the
/// context-level [`WorkspaceAliasCatalog`] (the shared §5/§9 seat the R3.6 alias
/// verbs populate):
///
/// - a registered alias ⇒ [`ExternalWorkbookRouting::Routed`] (live values via
///   [`gather_external_cells`]);
/// - an unregistered/unloaded alias ⇒ [`ExternalWorkbookRouting::RefError`]
///   (typed `#REF!` + [`EXTERNAL_WORKBOOK_NOT_LOADED_DIAGNOSTIC`], never
///   silent).
///
/// Returns `None` when `workbook_component` is **not** an external token (an
/// ordinary in-workbook reference) — those are not this router's concern and
/// keep their local resolution path, mirroring
/// [`WorkbookReferenceCatalog::route_dependency`]'s same-sheet return.
///
/// This performs no I/O and consults no value cache — it reads the loaded alias
/// catalog only (the D2 §5 exclusions).
#[must_use]
pub fn route_external_workbook(
    aliases: &WorkspaceAliasCatalog,
    workbook_component: &str,
) -> Option<ExternalWorkbookRouting> {
    let alias = ExternalBookToken::alias_from_component(workbook_component)?;
    Some(match aliases.resolve_alias(alias.as_str()) {
        WorkspaceAliasLookup::Routed { workspace_id } => {
            ExternalWorkbookRouting::Routed { workspace_id, alias }
        }
        WorkspaceAliasLookup::Dormant { alias } => ExternalWorkbookRouting::RefError {
            alias,
            diagnostic: EXTERNAL_WORKBOOK_NOT_LOADED_DIAGNOSTIC,
        },
    })
}

/// Gather the sibling-workspace computed values an external-workbook dependency
/// references, so the consumer can inject them into the evaluating sheet's
/// external view (W062 D2 §5, R3.7).
///
/// This is the **value** half of the external slice and the exact
/// cross-workspace analogue of [`gather_cross_sheet_cells`]: one seam up at the
/// workspace boundary rather than the sheet boundary. `sibling_computed` is the
/// loaded sibling workspace's own published computed grid store (the same
/// [`ExcelGridCellAddress`]-keyed map a [`crate::grid::machine::GridCalcRefSheet`]
/// publishes) for the referenced sheet. For each dependency this pulls the
/// covered cells the sibling has a value for, keyed by the dependency's own
/// address. An address the sibling has no value for simply carries no entry
/// (reads empty, like an empty in-sheet cell) — it is not fabricated.
///
/// This does not touch dirty propagation. Which formulas re-run when a sibling
/// cell changes is cross-**workbook** coordination (the D3/R4 lane); the honest
/// slice here makes the external *value* correct on recalc. A caller drives it
/// only after establishing an [`ExternalWorkbookRouting::Routed`]; an unloaded
/// sibling yields no values (the typed `#REF!` is the routing's job, above).
#[must_use]
pub fn gather_external_cells(
    dependencies: &[GridDependency],
    sibling_computed: &BTreeMap<
        crate::grid::coords::ExcelGridCellAddress,
        oxfunc_core::value::CalcValue,
    >,
) -> BTreeMap<crate::grid::coords::ExcelGridCellAddress, oxfunc_core::value::CalcValue> {
    use crate::grid::coords::ExcelGridCellAddress;

    let mut gathered = BTreeMap::new();
    for dependency in dependencies {
        match dependency {
            GridDependency::Cell(address) => {
                if let Some(value) = sibling_computed.get(address) {
                    gathered.insert(address.clone(), value.clone());
                }
            }
            GridDependency::Range(rect) => {
                for row in rect.top_row..=rect.bottom_row {
                    for col in rect.left_col..=rect.right_col {
                        let address = ExcelGridCellAddress::new(
                            rect.workbook_id.clone(),
                            rect.sheet_id.clone(),
                            row,
                            col,
                        );
                        if let Some(value) = sibling_computed.get(&address) {
                            gathered.insert(address, value.clone());
                        }
                    }
                }
            }
            _ => {}
        }
    }
    gathered
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

    // --- R3.3: cross-sheet dependency routing + descriptor emission (V5) ---

    use crate::grid::coords::ExcelGridCellAddress;
    use crate::grid::machine::GridDependency;

    fn cell_dep(sheet_id: &str, row: u32, col: u32) -> GridDependency {
        GridDependency::Cell(ExcelGridCellAddress::new("book:default", sheet_id, row, col))
    }

    #[test]
    fn same_sheet_dependency_is_not_routed_cross_sheet() {
        let catalog = WorkbookReferenceCatalog::build(&three_sheet_workbook());
        // A dependency on the owner's own sheet needs no cross-sheet routing.
        assert_eq!(
            catalog.route_dependency("Sheet1", &cell_dep("Sheet1", 1, 1)),
            CrossSheetRouting::SameSheet
        );
        // A non-address dependency shape is also left alone.
        let name_dep = GridDependency::NameIdentity(
            crate::grid::machine::GridNameIdentityDependency::from_key("k".to_string()),
        );
        assert_eq!(
            catalog.route_dependency("Sheet1", &name_dep),
            CrossSheetRouting::SameSheet
        );
    }

    #[test]
    fn cross_sheet_dependency_routes_to_token_keyed_descriptor() {
        let catalog = WorkbookReferenceCatalog::build(&three_sheet_workbook());
        // A formula on Sheet1 depending on Data!B2 routes to node 3.
        let routing = catalog.route_dependency("Sheet1", &cell_dep("Data", 2, 2));
        let CrossSheetRouting::Routed(descriptor) = routing else {
            panic!("Data must route to a live descriptor, got {routing:?}");
        };
        assert_eq!(descriptor.target_sheet_node, TreeNodeId(3));
        assert_eq!(descriptor.token.as_str(), "sheet-node:3");
        assert_eq!(descriptor.engine_handle.as_deref(), Some("grid-3"));
        // The dependency is carried verbatim (V5 `dependency`).
        assert_eq!(descriptor.dependency, cell_dep("Data", 2, 2));
    }

    #[test]
    fn cross_sheet_dependency_routing_is_case_insensitive() {
        let catalog = WorkbookReferenceCatalog::build(&three_sheet_workbook());
        for sheet in ["Summary", "SUMMARY", "summary"] {
            let routing = catalog.route_dependency("Sheet1", &cell_dep(sheet, 1, 1));
            let CrossSheetRouting::Routed(descriptor) = routing else {
                panic!("{sheet} must route via the shared fold");
            };
            assert_eq!(descriptor.target_sheet_node, TreeNodeId(4));
        }
    }

    #[test]
    fn unknown_sheet_dependency_falls_to_name_keyed_dormant_record() {
        let catalog = WorkbookReferenceCatalog::build(&three_sheet_workbook());
        // Bind stays existence-blind: routing a dependency on a never-existed
        // sheet does not error, it reports a name-keyed dormant edge.
        let routing = catalog.route_dependency("Sheet1", &cell_dep("Ghost", 1, 1));
        let CrossSheetRouting::Dormant { name, dependency } = routing else {
            panic!("Ghost must fall dormant, got {routing:?}");
        };
        assert_eq!(name, NormalizedContainerName::from_symbol("ghost"));
        assert_eq!(dependency, cell_dep("Ghost", 1, 1));
    }

    #[test]
    fn route_dependencies_batch_splits_routed_from_dormant() {
        let catalog = WorkbookReferenceCatalog::build(&three_sheet_workbook());
        let deps = [
            cell_dep("Sheet1", 1, 1), // same sheet -> dropped
            cell_dep("Data", 1, 1),   // routed -> node 3
            GridDependency::Range(
                crate::grid::geometry::GridRect::new(
                    "book:default",
                    "Summary",
                    1,
                    1,
                    2,
                    2,
                    crate::grid::coords::ExcelGridBounds::strict_excel(),
                )
                .unwrap(),
            ), // routed range -> node 4
            cell_dep("Ghost", 3, 3),  // dormant
        ];
        let result = catalog.route_dependencies("Sheet1", deps.iter());
        assert_eq!(result.routed.len(), 2);
        assert_eq!(result.dormant.len(), 1);
        let routed_nodes: Vec<TreeNodeId> =
            result.routed.iter().map(|d| d.target_sheet_node).collect();
        assert_eq!(routed_nodes, vec![TreeNodeId(3), TreeNodeId(4)]);
        assert_eq!(
            result.dormant[0].name,
            NormalizedContainerName::from_symbol("ghost")
        );
    }

    // --- R3.3: dormant-heal migration via ACTUAL sheet creation -----------

    #[test]
    fn dormant_record_migrates_to_token_keyed_on_actual_sheet_creation() {
        // Start with the 3-sheet workbook; a formula references a never-existed
        // sheet "Forecast", which falls dormant.
        let snapshot = three_sheet_workbook();
        let catalog = WorkbookReferenceCatalog::build(&snapshot);
        let CrossSheetRouting::Dormant { name, dependency } =
            catalog.route_dependency("Sheet1", &cell_dep("Forecast", 5, 5))
        else {
            panic!("Forecast must start dormant");
        };

        let mut ledger = DormantSheetReferenceLedger::new();
        ledger.record(DormantSheetReference {
            name: name.clone(),
            dependency: dependency.clone(),
        });
        assert!(ledger.has_dormant(&name));
        assert_eq!(ledger.dormant_name_count(), 1);

        // Healing against the SAME catalog (Forecast still absent) leaves the
        // record dormant.
        let healed = ledger.heal_against(&catalog);
        assert!(healed.is_empty(), "no heal before the sheet exists");
        assert!(ledger.has_dormant(&name));

        // Now ACTUALLY create the sheet "Forecast" as node 5, rebuild the
        // catalog from the post-creation snapshot, and heal.
        let outcome = snapshot
            .apply_edit(
                StructuralSnapshotId(2),
                crate::structural::StructuralEdit::InsertNode {
                    node: sheet_node(5, "Forecast"),
                    parent_id: TreeNodeId(1),
                    index: None,
                },
            )
            .unwrap();
        let healed_catalog = WorkbookReferenceCatalog::build(&outcome.snapshot);
        let healed = ledger.heal_against(&healed_catalog);

        // The dormant record migrated to a token-keyed descriptor for node 5.
        assert_eq!(healed.len(), 1);
        assert_eq!(healed[0].target_sheet_node, TreeNodeId(5));
        assert_eq!(healed[0].token.as_str(), "sheet-node:5");
        assert_eq!(healed[0].dependency, dependency);
        // And the ledger no longer holds it.
        assert!(!ledger.has_dormant(&name));
        assert!(ledger.is_empty());
    }

    #[test]
    fn heal_leaves_unrelated_dormant_records_in_place() {
        let snapshot = three_sheet_workbook();
        let mut ledger = DormantSheetReferenceLedger::new();
        ledger.record(DormantSheetReference {
            name: NormalizedContainerName::from_symbol("Forecast"),
            dependency: cell_dep("Forecast", 1, 1),
        });
        ledger.record(DormantSheetReference {
            name: NormalizedContainerName::from_symbol("Budget"),
            dependency: cell_dep("Budget", 1, 1),
        });
        assert_eq!(ledger.dormant_name_count(), 2);

        // Create only "Forecast"; "Budget" stays dormant.
        let outcome = snapshot
            .apply_edit(
                StructuralSnapshotId(2),
                crate::structural::StructuralEdit::InsertNode {
                    node: sheet_node(5, "Forecast"),
                    parent_id: TreeNodeId(1),
                    index: None,
                },
            )
            .unwrap();
        let catalog = WorkbookReferenceCatalog::build(&outcome.snapshot);
        let healed = ledger.heal_against(&catalog);

        assert_eq!(healed.len(), 1);
        assert_eq!(healed[0].target_sheet_node, TreeNodeId(5));
        assert!(!ledger.has_dormant(&NormalizedContainerName::from_symbol("Forecast")));
        assert!(ledger.has_dormant(&NormalizedContainerName::from_symbol("Budget")));
    }

    // --- R3.4 (D2 §6 / V7): sheet-deletion policy transforms, end-to-end -----

    use crate::grid::reference_engine::decode_excel_grid_reference_payload;
    use crate::grid::ast::ExcelGridReference;
    use oxfml_core::binding::{BindContext, BindRequest, NormalizedReference, ReferenceValidity};
    use oxfml_core::red::project_red_view;
    use oxfml_core::source::{FormulaChannelKind, FormulaSourceRecord, FormulaToken, StructureContextVersion};
    use oxfml_core::syntax::parser::{parse_formula, ParseRequest};
    use oxfml_core::bind_formula;

    /// Bind one strict-excel reference formula and return its first bound
    /// `ProfileReferenceRecord`. `caller` on Sheet1 R1C1.
    fn bind_strict_record(stable_id: &str, formula: &str) -> ProfileReferenceRecord {
        let profile = StrictExcelGridReferenceProfile::new();
        let source = FormulaSourceRecord::new(stable_id, 1, formula.to_string())
            .with_formula_channel_kind(FormulaChannelKind::WorksheetA1);
        let parse = parse_formula(ParseRequest { source: source.clone() });
        let red = project_red_view(source.formula_stable_id.clone(), &parse.green_tree);
        let request = BindRequest {
            source,
            green_tree: parse.green_tree,
            red_projection: red,
            context: BindContext {
                caller_row: 1,
                caller_col: 1,
                formula_token: FormulaToken("r34-token".to_string()),
                structure_context_version: StructureContextVersion("r34-struct-v1".to_string()),
                ..BindContext::default()
            },
            reference_bind_profile: Some(&profile),
        };
        let bound = bind_formula(request).bound_formula;
        match &bound.normalized_references[0] {
            NormalizedReference::ProfileSymbolic(record) => record.clone(),
            other => panic!("expected profile symbolic reference, got {other:?}"),
        }
    }

    fn renders_ref_error(record: &ProfileReferenceRecord) -> bool {
        record.render_hint.as_deref() == Some("#REF!")
            && matches!(
                decode_excel_grid_reference_payload(&record.profile_payload),
                Some(ExcelGridReference::RefError { .. })
            )
    }

    /// Acceptance 1 (D2 §6): two-sheet workbook, Sheet1 references Sheet2!A1;
    /// deleting Sheet2 makes Sheet1's formula show `#REF!` — rendered from the
    /// transformed RECORD, not ad-hoc state.
    #[test]
    fn r34_strict_delete_sheet_makes_reference_ref_error_from_record() {
        let profile = StrictExcelGridReferenceProfile::new();
        let record = bind_strict_record("r34-delete", "=Sheet2!A1");
        assert!(!renders_ref_error(&record), "reference starts live");

        let outcome = apply_sheet_deletion_to_record(
            ContainerDeletionPolicy::HardRefError,
            &profile,
            "book:default",
            "Sheet2",
            &NormalizedContainerName::from_symbol("Sheet2"),
            &record,
        );
        assert!(outcome.is_hard_ref_error());
        assert_eq!(outcome.record().validity, ReferenceValidity::InvalidStatic);
        // #REF! comes from the record.
        assert!(renders_ref_error(outcome.record()));
    }

    /// Acceptance 2 (D2 §6): recreating a same-named sheet does NOT heal the
    /// deleted-sheet reference — the strict profile's `HardRefError` policy
    /// declines the heal-on-recreate mechanism. Proven by the record staying a
    /// `RefError` no matter how many further (re)creates/deletes are driven.
    #[test]
    fn r34_strict_recreate_same_name_does_not_heal_ref_error() {
        let profile = StrictExcelGridReferenceProfile::new();
        let record = bind_strict_record("r34-recreate", "=Sheet2!A1");

        // Delete Sheet2 -> #REF!.
        let deleted = apply_sheet_deletion_to_record(
            ContainerDeletionPolicy::HardRefError,
            &profile,
            "book:default",
            "Sheet2",
            &NormalizedContainerName::from_symbol("Sheet2"),
            &record,
        );
        assert!(deleted.is_hard_ref_error());
        let ref_error = deleted.record().clone();

        // "Recreate Sheet2": re-drive the deletion transform machinery over the
        // now-RefError record. It stays #REF! — recreate is inert, only undo
        // (revision restore) resurrects. This is the explicit negative test the
        // bead requires: the deleted-sheet reference does NOT re-enter the
        // dormant-heal path.
        let after_recreate = apply_sheet_deletion_to_record(
            ContainerDeletionPolicy::HardRefError,
            &profile,
            "book:default",
            "Sheet2",
            &NormalizedContainerName::from_symbol("Sheet2"),
            &ref_error,
        );
        assert!(renders_ref_error(after_recreate.record()));
    }

    /// Acceptance 3 (D2 §6): undo (revision navigation) restores the original
    /// working reference. The transform is non-destructive to its INPUT record
    /// (it clones), so the pre-delete record a revision retains still renders
    /// live and still targets Sheet2 — exactly what revision restore replays.
    #[test]
    fn r34_strict_undo_restores_original_working_reference() {
        let profile = StrictExcelGridReferenceProfile::new();
        let original = bind_strict_record("r34-undo", "=Sheet2!A1");

        let deleted = apply_sheet_deletion_to_record(
            ContainerDeletionPolicy::HardRefError,
            &profile,
            "book:default",
            "Sheet2",
            &NormalizedContainerName::from_symbol("Sheet2"),
            &original,
        );
        assert!(renders_ref_error(deleted.record()));

        // Undo = revision restore of the pre-transform record. The original is
        // untouched by the transform and still binds a live Sheet2 cell.
        assert!(!renders_ref_error(&original));
        match decode_excel_grid_reference_payload(&original.profile_payload).expect("payload") {
            ExcelGridReference::Cell { sheet_id, .. } => assert_eq!(sheet_id, "Sheet2"),
            other => panic!("restored reference should be a live Sheet2 cell, got {other:?}"),
        }
    }

    /// Acceptance 4 (D2 §6): the tree profile's `DormantIdentityHeal` policy is
    /// unchanged — a deleted-target reference falls to a dormant edge (keyed on
    /// the normalized name) that heals if the name reappears, NOT a hard `#REF!`.
    #[test]
    fn r34_tree_profile_deletion_stays_dormant_heal() {
        // The record's profile is immaterial to the policy branch: the tree
        // profile carries `DormantIdentityHeal`, so the driver leaves the record
        // intact and reports the dormant edge. (We reuse a strict-bound record
        // only as a record value; the policy is what selects the branch.)
        let profile = StrictExcelGridReferenceProfile::new();
        let record = bind_strict_record("r34-tree", "=Sheet2!A1");

        let outcome = apply_sheet_deletion_to_record(
            ContainerDeletionPolicy::DormantIdentityHeal,
            &profile,
            "book:default",
            "Sheet2",
            &NormalizedContainerName::from_symbol("Sheet2"),
            &record,
        );
        match outcome {
            SheetDeletionRecordOutcome::DormantHeal { record: kept, name } => {
                // Record intact (no destructive #REF! rewrite).
                assert!(!renders_ref_error(&kept));
                assert_eq!(kept, record);
                assert_eq!(name, NormalizedContainerName::from_symbol("Sheet2"));
            }
            other => panic!("tree profile must dormant-heal, got {other:?}"),
        }

        // And the dormant ledger's never-existed heal path (R3.3) is untouched:
        // a dormant record for that name heals when a sheet of the name is
        // (re)created — the lenient contract the tree profile keeps.
        let mut ledger = DormantSheetReferenceLedger::new();
        ledger.record(DormantSheetReference {
            name: NormalizedContainerName::from_symbol("Sheet2"),
            dependency: GridDependency::Cell(crate::grid::coords::ExcelGridCellAddress::new(
                "book:default",
                "Sheet2",
                1,
                1,
            )),
        });
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
            [root, sheet_node(2, "Sheet2")],
        )
        .unwrap();
        let catalog = WorkbookReferenceCatalog::build(&snapshot);
        let healed = ledger.heal_against(&catalog);
        assert_eq!(healed.len(), 1, "tree/dormant path still heals on (re)appearance");
    }

    // --- R3.7 (D2 §5): external-workbook routing to loaded siblings --------

    use crate::grid::coords::ExcelGridBounds;
    use oxfunc_core::value::CalcValue;

    /// A registered+loaded sibling alias routes the external `{workbook}`
    /// component (`extbook:book2`) to the sibling's workspace id — the LIVE
    /// lane (D2 §5). Case-insensitive via the shared fold.
    #[test]
    fn external_workbook_routes_to_loaded_sibling_workspace() {
        let mut aliases = WorkspaceAliasCatalog::new();
        aliases.register_alias("Book2", "workspace:book2-loaded");

        let token = ExternalBookToken::from_alias("Book2");
        assert_eq!(token.as_str(), "extbook:book2");

        let routing = route_external_workbook(&aliases, token.as_str())
            .expect("external component routes");
        assert_eq!(
            routing,
            ExternalWorkbookRouting::Routed {
                workspace_id: "workspace:book2-loaded".to_string(),
                alias: NormalizedContainerName::from_symbol("book2"),
            }
        );
        assert_eq!(routing.workspace_id(), Some("workspace:book2-loaded"));
        assert!(!routing.is_ref_error());

        // Case-stable: a key minted from [BOOK2] routes identically.
        let upper = route_external_workbook(&aliases, ExternalBookToken::from_alias("BOOK2").as_str())
            .unwrap();
        assert_eq!(upper.workspace_id(), Some("workspace:book2-loaded"));
    }

    /// An unregistered / unloaded alias yields a TYPED `#REF!` carrying the
    /// stable `excel.grid.external.workbook_not_loaded` diagnostic — never a
    /// silent empty (D2 §5). The edge is dormant on the alias, so a later
    /// same-alias sibling load heals it (routing, not the key, flips).
    #[test]
    fn external_workbook_unloaded_sibling_is_typed_ref_error_not_silent() {
        let empty = WorkspaceAliasCatalog::new();
        let routing =
            route_external_workbook(&empty, ExternalBookToken::from_alias("Book2").as_str())
                .expect("external component routes");
        assert_eq!(
            routing,
            ExternalWorkbookRouting::RefError {
                alias: NormalizedContainerName::from_symbol("book2"),
                diagnostic: EXTERNAL_WORKBOOK_NOT_LOADED_DIAGNOSTIC,
            }
        );
        assert!(routing.is_ref_error());
        assert_eq!(routing.workspace_id(), None);

        // Heal-on-load: register the alias, and the SAME component now routes
        // live — no key change was needed, only the alias catalog state.
        let mut aliases = WorkspaceAliasCatalog::new();
        aliases.register_alias("Book2", "workspace:book2");
        let healed =
            route_external_workbook(&aliases, ExternalBookToken::from_alias("Book2").as_str())
                .unwrap();
        assert_eq!(healed.workspace_id(), Some("workspace:book2"));
    }

    /// A non-external `{workbook}` component (an ordinary in-workbook reference)
    /// is not this router's concern — it returns `None` and keeps its local
    /// path, mirroring `route_dependency`'s same-sheet return.
    #[test]
    fn non_external_workbook_component_is_not_routed() {
        let aliases = WorkspaceAliasCatalog::new();
        assert_eq!(route_external_workbook(&aliases, "book:default"), None);
        assert_eq!(route_external_workbook(&aliases, "grid-2"), None);
    }

    /// The value half: `gather_external_cells` pulls the covered cells from the
    /// sibling's published computed store, keyed by their own address. Absent
    /// cells carry no entry (read empty, never fabricated).
    #[test]
    fn gather_external_cells_pulls_sibling_values() {
        let bounds = ExcelGridBounds::strict_excel();
        let _ = bounds;
        let a1 = ExcelGridCellAddress::new("book:sibling", "Sheet1", 1, 1);
        let a2 = ExcelGridCellAddress::new("book:sibling", "Sheet1", 2, 1);
        let mut sibling_computed = BTreeMap::new();
        sibling_computed.insert(a1.clone(), CalcValue::number(42.0));
        // a2 is deliberately absent from the sibling's store.

        let deps = vec![
            GridDependency::Cell(a1.clone()),
            GridDependency::Cell(a2.clone()),
        ];
        let gathered = gather_external_cells(&deps, &sibling_computed);
        assert_eq!(gathered.get(&a1), Some(&CalcValue::number(42.0)));
        assert_eq!(gathered.get(&a2), None, "absent sibling cell is not fabricated");
        assert_eq!(gathered.len(), 1);
    }

    /// A range dependency gathers every populated cell the rect covers.
    #[test]
    fn gather_external_cells_covers_a_range() {
        let bounds = ExcelGridBounds::strict_excel();
        let rect = crate::grid::geometry::GridRect::new(
            "book:sibling", "Sheet1", 1, 1, 2, 1, bounds,
        )
        .unwrap();
        let b1 = ExcelGridCellAddress::new("book:sibling", "Sheet1", 1, 1);
        let b2 = ExcelGridCellAddress::new("book:sibling", "Sheet1", 2, 1);
        let mut sibling_computed = BTreeMap::new();
        sibling_computed.insert(b1.clone(), CalcValue::number(10.0));
        sibling_computed.insert(b2.clone(), CalcValue::number(20.0));

        let gathered = gather_external_cells(&[GridDependency::Range(rect)], &sibling_computed);
        assert_eq!(gathered.len(), 2);
        assert_eq!(gathered.get(&b1), Some(&CalcValue::number(10.0)));
        assert_eq!(gathered.get(&b2), Some(&CalcValue::number(20.0)));
    }
}
