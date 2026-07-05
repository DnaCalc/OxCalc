//! The workbook calculation node space and the sheet-identity routing
//! invariant (W062 D3 §2, bead `calc-5kqg.32` / R4.4).
//!
//! Three constituent node kinds — grid cells (sheet-qualified via
//! [`ExcelGridCellAddress`]), scoped names ([`ScopedNameKey`] at the workbook
//! layer), and tree nodes ([`TreeNodeId`]) — are unified under
//! [`WorkbookCalcNodeId`]. This is the total node space over which the
//! workbook dirty closure (R4.6) and the workbook oracle (R4.5) will operate;
//! this bead lands the space and its constructors only.
//!
//! The other half of R4.4 is the **routing invariant**: sheet identity on
//! [`ExcelGridCellAddress`]/[`GridRect`] — carried since the grid's inception
//! but never consulted in dependency routing — becomes *authoritative*. An
//! edge whose address names a different sheet than the per-sheet graph it is
//! being registered into is rejected at registration with a typed
//! [`GridRefError::ForeignSheetDependency`], closing the "identity carried but
//! never consulted" gap (D3 §2). The enforcement point is the per-sheet
//! [`GridDependencyIndex`](super::invalidation) registration path; see
//! [`OwningSheetIdentity`] below for the identity a per-sheet index is stamped
//! with.

use crate::grid::coords::ExcelGridCellAddress;
use crate::grid::geometry::GridRect;
use crate::structural::TreeNodeId;
use crate::workbook_settings::WorkbookSettingChanged;

/// The scope of a defined name at the workbook layer.
///
/// Excel resolves a bare name token against sheet scope first, then workbook
/// scope (sheet-scope-shadows-workbook-scope precedence; D3 §2.2). D3's
/// contract is that edge registration receives a *scope-resolved* name and
/// registers against the resolved key — this enum is that resolved scope.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NameScope {
    /// Workbook-global scope.
    Workbook,
    /// Scoped to a specific sheet (identified by its stable tree node id;
    /// D1 C1/C8: sheets are Sheet-role children and `TreeNodeId` is the
    /// stable identity).
    Sheet(TreeNodeId),
}

/// A scope-qualified defined-name key at the workbook layer.
///
/// Per-sheet indexes keep their bare `String` name keys (they are sheet-scoped
/// by construction); the workbook layer introduces this scope-qualified key
/// and owns the mapping (D3 §2.2). `normalized` is the resolved, normalized
/// name text.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ScopedNameKey {
    pub scope: NameScope,
    pub normalized: String,
}

impl ScopedNameKey {
    /// Construct a workbook-scoped name key.
    #[must_use]
    pub fn workbook(normalized: impl Into<String>) -> Self {
        Self {
            scope: NameScope::Workbook,
            normalized: normalized.into(),
        }
    }

    /// Construct a sheet-scoped name key.
    #[must_use]
    pub fn sheet(sheet: TreeNodeId, normalized: impl Into<String>) -> Self {
        Self {
            scope: NameScope::Sheet(sheet),
            normalized: normalized.into(),
        }
    }
}

/// The workbook calculation node space.
///
/// Grid cells join at cell granularity (carrying full workbook+sheet identity
/// already); scoped names are first-class nodes so name→name and name→cell
/// edges exist; the tree joins at name granularity via [`TreeNodeId`]
/// (D3 §2.1 / §8). This is a *total* space over the three constituent kinds —
/// every workbook-calc node is exactly one of these — which is what lets the
/// closure and oracle range over a single `BTreeSet<WorkbookCalcNodeId>`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WorkbookCalcNodeId {
    /// A grid cell, at cell granularity, carrying workbook+sheet identity.
    GridCell(ExcelGridCellAddress),
    /// A scoped defined name at the workbook layer.
    Name(ScopedNameKey),
    /// A tree node.
    TreeNode(TreeNodeId),
}

impl WorkbookCalcNodeId {
    /// Construct a grid-cell node.
    #[must_use]
    pub fn grid_cell(address: ExcelGridCellAddress) -> Self {
        Self::GridCell(address)
    }

    /// Construct a scoped-name node.
    #[must_use]
    pub fn name(key: ScopedNameKey) -> Self {
        Self::Name(key)
    }

    /// Construct a tree-node node.
    #[must_use]
    pub const fn tree_node(node: TreeNodeId) -> Self {
        Self::TreeNode(node)
    }
}

/// Workbook-level dirty seeds — the single workbook-level seed vocabulary
/// (D3 §2.1, X1).
///
/// Sheet-local seeds are *addressed* (a `Sheet { sheet, seed }` reuses the
/// per-sheet [`GridDirtySeed`](super::invalidation) vocabulary), never global.
/// D1 C4 `WorkbookSettingChanged` seeds enter as [`WorkbookDirtySeed::Setting`].
/// Document-surface verbs emit seeds; they never touch dirty state directly.
///
/// Cannot derive `Eq`/`Ord` because [`WorkbookSettingChanged`] carries setting
/// payloads that are only `PartialEq` — the closure keys its dirty *set* on
/// [`WorkbookCalcNodeId`] (which is fully ordered), not on the seed vocabulary.
#[derive(Debug, Clone, PartialEq)]
pub enum WorkbookDirtySeed {
    /// A sheet-local seed addressed to a specific sheet; reuses the per-sheet
    /// dirty-seed vocabulary unchanged.
    Sheet {
        sheet: TreeNodeId,
        seed: super::invalidation::GridDirtySeed,
    },
    /// A scoped name became dirty.
    Name(ScopedNameKey),
    /// A tree node became dirty.
    TreeNode(TreeNodeId),
    /// One workbook-wide volatile tick (D3 §7).
    Volatile,
    /// A workbook-wide external-input tick.
    External,
    /// A typed D1 C4 workbook-setting change.
    Setting(WorkbookSettingChanged),
}

/// The authoritative owning-sheet identity a per-sheet dependency index is
/// stamped with, so the routing invariant can be enforced at registration.
///
/// A per-sheet [`GridInvalidationRef`](super::invalidation) / index carries the
/// `(workbook_id, sheet_id)` of the sheet it belongs to. When set, every
/// dependent and every cell/range dependency address routed into that index
/// must resolve to this sheet; a foreign address is a routing bug and is
/// rejected with [`GridRefError::ForeignSheetDependency`] rather than silently
/// mis-filed (D3 §1 routing invariant).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OwningSheetIdentity {
    pub workbook_id: String,
    pub sheet_id: String,
}

impl OwningSheetIdentity {
    #[must_use]
    pub fn new(workbook_id: impl Into<String>, sheet_id: impl Into<String>) -> Self {
        Self {
            workbook_id: workbook_id.into(),
            sheet_id: sheet_id.into(),
        }
    }

    /// Does `address` belong to the sheet this identity names?
    #[must_use]
    pub fn owns_address(&self, address: &ExcelGridCellAddress) -> bool {
        address.workbook_id == self.workbook_id && address.sheet_id == self.sheet_id
    }

    /// Does `rect` belong to the sheet this identity names?
    #[must_use]
    pub fn owns_rect(&self, rect: &GridRect) -> bool {
        rect.workbook_id == self.workbook_id && rect.sheet_id == self.sheet_id
    }
}
