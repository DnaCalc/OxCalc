//! Authored grid cell state shared by both grid engines: literal values and
//! formula cells carrying their source text, R1C1-relative normal-form key, and
//! source channel.
//!
//! This module also owns [`GridInputState`] — the revision-shaped authored
//! truth for a grid-backed node (W062 R2.6, D1 §7.1). It records the authored
//! cell inputs, repeated-formula regions, merged-region declarations, and
//! table-overlay declarations from which a live derived engine sheet is a pure
//! function. The derived engine state (`GridOptimizedSheet`, published cells,
//! overlays, epochs) is evictable and lives in the consumer; the split lets a
//! retained revision hold authored truth alone (retention itself lands in R2.7).

use std::collections::BTreeMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;

use oxfml_core::source::FormulaChannelKind;
use oxfunc_core::value::CalcValue;
use serde::{Deserialize, Serialize};

use crate::grid::coords::{ExcelGridBounds, ExcelGridCellAddress};
use crate::grid::geometry::GridRect;
use crate::grid::machine::GridTableOverlay;

#[derive(Debug, Clone, PartialEq)]
pub enum GridAuthoredCell {
    Literal(CalcValue),
    Formula(GridFormulaCell),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridFormulaCell {
    pub source_text: String,
    pub normal_form_key: String,
    pub source_channel: FormulaChannelKind,
}

impl GridFormulaCell {
    #[must_use]
    pub fn new(source_text: impl Into<String>, normal_form_key: impl Into<String>) -> Self {
        Self {
            source_text: source_text.into(),
            normal_form_key: normal_form_key.into(),
            source_channel: FormulaChannelKind::WorksheetA1,
        }
    }

    #[must_use]
    pub const fn with_source_channel(mut self, source_channel: FormulaChannelKind) -> Self {
        self.source_channel = source_channel;
        self
    }
}

/// The consumer-authored scope of a defined name (W062 R3.5, D2 §4.3).
///
/// A workbook-scoped name is visible from every sheet; a sheet-scoped name is
/// visible only from formulas evaluated on the named sheet and shadows the
/// workbook-scoped name of the same text (D2 §4.3 precedence: sheet scope
/// outranks workbook scope). The sheet is named by its `sheet_id` string (the
/// engine's per-sheet key material), matching how the strict-excel profile keys
/// scoped names.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GridDefinedNameScope {
    /// Workbook-global: visible from every sheet.
    Workbook,
    /// Scoped to the sheet with this `sheet_id`; shadows the workbook-scoped
    /// name of the same text on that sheet.
    Sheet(String),
}

/// The consumer-authored target of a defined name (W062 R3.5).
///
/// A static name binds a rectangular extent (`=Sheet1!$A$1:$B$2`-style); a
/// dynamic name binds a defining formula whose realized extent is recomputed at
/// recalc time (the engine's `dynamic_defined_names` lane).
#[derive(Debug, Clone, PartialEq)]
pub enum GridDefinedNameTarget {
    /// A fixed rectangular extent.
    Static(GridRect),
    /// A defining formula (dynamic defined name).
    Dynamic(GridFormulaCell),
}

/// One consumer-authored defined name on a grid node (W062 R3.5, D2 §4.3).
///
/// This is *authored truth* — it lives in [`GridInputState`] so a derived sheet
/// rebuilt from input (revision navigation) re-registers the same names. The
/// derived engine sheet's name namespace is a pure function of this list, in
/// authoring order (a later authored name of the same scope+text wins).
#[derive(Debug, Clone, PartialEq)]
pub struct GridInputDefinedName {
    pub scope: GridDefinedNameScope,
    pub name: String,
    pub target: GridDefinedNameTarget,
}

/// A content address over a grid node's authored truth (W062 C5 / D1 §7.1).
///
/// Two [`GridInputState`]s with equal ids have identical authored content at
/// content-address confidence (equal ids ⇒ equal content up to digest
/// collision); an id is valid as a recalc basis and a cache key. Unbounded
/// authored bases enter the id **only as fixed-width digests** — the
/// digest-fold discipline of the warm-recalc OOM lesson
/// (`workspace_revision.rs` `identity_basis_digest`), because a grid's authored
/// record set is unbounded in exactly the way that caused that OOM.
///
/// Hash choice: the digest fold uses two independently seeded 64-bit
/// `DefaultHasher` lanes for a 128-bit token, matching the established
/// `identity_basis_digest` precedent. `DefaultHasher` (SipHash-1-3) is *not*
/// guaranteed stable across std versions, so ids are process-local content
/// addresses, not a persisted wire contract. Pinning a stable algorithm was
/// deferred: no stable-hash crate is a trivially available dependency of
/// `oxcalc-core`, and adding one is out of this bead's scope; the precedent
/// this mirrors carries the same documented instability.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct GridInputSnapshotId(pub String);

/// An authored cell input record — the grid analog of a node input record.
///
/// A formula record carries **only** its `source_text` and `source_channel`;
/// it deliberately excludes the minted R1C1 normal-form key (D4's refinement of
/// C5). The normal-form key is a derived artifact of the engine and is minted
/// when the derived sheet is (re)built from this record, never frozen into
/// authored truth — so two revisions whose authored formula text is equal share
/// a grid-input identity regardless of any key-minting drift.
#[derive(Debug, Clone, PartialEq)]
pub enum GridInputCell {
    Literal(CalcValue),
    Formula {
        source_text: String,
        source_channel: FormulaChannelKind,
    },
}

impl GridInputCell {
    /// Reconstruct the engine-facing [`GridAuthoredCell`] from this authored
    /// record, minting the normal-form key from the source text — byte-identical
    /// to how [`GridFormulaCell::new`] derives the key at authoring time (the key
    /// tracks the source text today; when the engine mints a distinct key it
    /// will do so here from the same inputs).
    #[must_use]
    pub fn to_authored_cell(&self) -> GridAuthoredCell {
        match self {
            Self::Literal(value) => GridAuthoredCell::Literal(value.clone()),
            Self::Formula {
                source_text,
                source_channel,
            } => GridAuthoredCell::Formula(
                GridFormulaCell::new(source_text.clone(), source_text.clone())
                    .with_source_channel(*source_channel),
            ),
        }
    }

    /// Capture an authored record from an engine-facing [`GridAuthoredCell`],
    /// dropping the minted normal-form key per the no-key-in-input-state rule.
    #[must_use]
    pub fn from_authored_cell(cell: &GridAuthoredCell) -> Self {
        match cell {
            GridAuthoredCell::Literal(value) => Self::Literal(value.clone()),
            GridAuthoredCell::Formula(formula) => Self::Formula {
                source_text: formula.source_text.clone(),
                source_channel: formula.source_channel,
            },
        }
    }
}

/// A repeated-formula region (a `FillRange`): one R1C1-relative formula tiled
/// over a rectangle, retained as a region rather than expanded to N cells.
#[derive(Debug, Clone, PartialEq)]
pub struct GridInputRepeatedRegion {
    pub rect: GridRect,
    pub source_text: String,
    pub source_channel: FormulaChannelKind,
}

/// The revision-shaped authored truth for a grid-backed node (W062 R2.6, D1
/// §7.1): the authored cell inputs, repeated-formula regions, merged-region
/// declarations, and table-overlay declarations, plus the grid's coordinate
/// identity and bounds. The live derived engine state is a pure function of
/// this — see the consumer's rebuild path.
///
/// The two retention classes of grid-backed state (W062 D1 §7.4 / C7).
///
/// D1 owns this class contract; **W054 owns eviction and pinning** of the
/// classes it defines. The enum fixes *which half of a grid backing is pinned
/// by what, and how it may be evicted* — the policy that acts on those rules is
/// W054's. Naming and the `selector_key` accessor follow the
/// [`EdgeValueCacheRetentionClass`](crate::value_cache) precedent so a future
/// GC keys eviction buckets by a stable string.
///
/// The mapping onto the live model is one class per half of
/// `GridNodeState { input, derived }`:
/// [`GridNodeState::retention_class_of_input`] classifies the authored
/// `Arc<GridInputState>` as [`RevisionRetainedGridInput`]; the derived engine
/// state is [`EphemeralDerivedGridState`].
///
/// [`RevisionRetainedGridInput`]: GridRetentionClass::RevisionRetainedGridInput
/// [`EphemeralDerivedGridState`]: GridRetentionClass::EphemeralDerivedGridState
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GridRetentionClass {
    /// Authored grid content pinned by a retained workspace revision or a
    /// candidate overlay basis. Evictable **only** by evicting the owning
    /// revision through the workspace revision retention policy
    /// (`enforce_workspace_revision_retention_policy`): dropping a retained
    /// revision drops its `grid_inputs` map, and structural per-sheet sharing
    /// (D1 §7.3) means the bytes free only when the *last* retaining revision
    /// goes. W054 GC must treat these as revision-pinned — whole revisions
    /// only, oldest-unpinned-first — never age-based per-sheet eviction, and
    /// candidate pins imply grid-input pins.
    RevisionRetainedGridInput,
    /// Derived grid state (optimized engine sheets, published cells, overlay
    /// projections). A pure function of the owning [`GridInputState`] plus a
    /// recalc; evictable at any time under memory pressure at the cost of
    /// rebuild-by-recalc.
    EphemeralDerivedGridState,
}

impl GridRetentionClass {
    /// Stable string key for a GC's eviction buckets, mirroring
    /// [`EdgeValueCacheRetentionClass::selector_key`](crate::value_cache).
    #[must_use]
    pub fn selector_key(self) -> &'static str {
        match self {
            Self::RevisionRetainedGridInput => "RevisionRetainedGridInput",
            Self::EphemeralDerivedGridState => "EphemeralDerivedGridState",
        }
    }

    /// Whether this class is revision-pinned — evictable only transitively by
    /// evicting the owning revision, never directly by an age/pressure GC. True
    /// for [`RevisionRetainedGridInput`], false for
    /// [`EphemeralDerivedGridState`].
    ///
    /// [`RevisionRetainedGridInput`]: GridRetentionClass::RevisionRetainedGridInput
    /// [`EphemeralDerivedGridState`]: GridRetentionClass::EphemeralDerivedGridState
    #[must_use]
    pub fn is_revision_pinned(self) -> bool {
        matches!(self, Self::RevisionRetainedGridInput)
    }
}

/// Content-addressed via [`GridInputState::identity`]; equal ids ⇒ equal
/// authored content at content-address confidence.
#[derive(Debug, Clone, PartialEq)]
pub struct GridInputState {
    pub workbook_id: String,
    pub sheet_id: String,
    pub bounds: ExcelGridBounds,
    /// Single authored cells, keyed by address. Ordered so the identity fold is
    /// deterministic and sheet rebuild is order-stable.
    pub cells: BTreeMap<ExcelGridCellAddress, GridInputCell>,
    /// Repeated-formula regions, in authoring order.
    pub repeated_regions: Vec<GridInputRepeatedRegion>,
    /// Committed merged-region declarations, in authoring order.
    pub merged_regions: Vec<GridRect>,
    /// Committed structured-table overlay declarations, in authoring order.
    pub table_overlays: Vec<GridTableOverlay>,
    /// Consumer-authored defined names (W062 R3.5, D2 §4.3), in authoring
    /// order. The derived sheet's name namespace is a pure function of this
    /// list; a later authored name of the same scope+text redefines the
    /// earlier one, matching the engine setters' last-write-wins semantics.
    pub defined_names: Vec<GridInputDefinedName>,
}

impl GridInputState {
    #[must_use]
    pub fn new(
        workbook_id: impl Into<String>,
        sheet_id: impl Into<String>,
        bounds: ExcelGridBounds,
    ) -> Self {
        Self {
            workbook_id: workbook_id.into(),
            sheet_id: sheet_id.into(),
            bounds,
            cells: BTreeMap::new(),
            repeated_regions: Vec::new(),
            merged_regions: Vec::new(),
            table_overlays: Vec::new(),
            defined_names: Vec::new(),
        }
    }

    /// Mint this state's content-addressed [`GridInputSnapshotId`].
    ///
    /// The unbounded authored basis (every cell record, region, merge, and table
    /// overlay) is streamed into a `String` basis, then folded to a fixed-width
    /// 128-bit digest before it enters the id — never verbatim.
    #[must_use]
    pub fn identity(&self) -> GridInputSnapshotId {
        let mut basis = String::new();
        // Bounded coordinate/identity fields go in verbatim.
        basis.push_str("grid-input:v1\n");
        basis.push_str(&format!("wb={:?}\n", self.workbook_id));
        basis.push_str(&format!("sh={:?}\n", self.sheet_id));
        basis.push_str(&format!("bounds={:?}\n", self.bounds));
        // Unbounded authored bases: streamed deterministically. `cells` is a
        // BTreeMap (address-ordered); the Vecs preserve authoring order.
        basis.push_str("cells:\n");
        for (address, cell) in &self.cells {
            basis.push_str(&format!("  {address:?} => {cell:?}\n"));
        }
        basis.push_str("repeated:\n");
        for region in &self.repeated_regions {
            basis.push_str(&format!("  {region:?}\n"));
        }
        basis.push_str("merged:\n");
        for rect in &self.merged_regions {
            basis.push_str(&format!("  {rect:?}\n"));
        }
        basis.push_str("tables:\n");
        for overlay in &self.table_overlays {
            basis.push_str(&format!("  {overlay:?}\n"));
        }
        basis.push_str("names:\n");
        for name in &self.defined_names {
            basis.push_str(&format!("  {name:?}\n"));
        }
        GridInputSnapshotId(grid_input_basis_digest(&basis))
    }
}

/// Collapses an unbounded authored grid basis to a fixed-width token for use
/// inside a [`GridInputSnapshotId`]. Two independently seeded 64-bit lanes give
/// a 128-bit token, making accidental id aliasing across distinct bases
/// negligible. Mirrors `workspace_revision.rs::identity_basis_digest`.
fn grid_input_basis_digest(basis: &str) -> String {
    let lane = |seed: &[u8]| {
        let mut hasher = DefaultHasher::new();
        hasher.write(seed);
        hasher.write(basis.as_bytes());
        hasher.finish()
    };
    format!(
        "grid-input-digest:v1:{:016x}{:016x}",
        lane(b"grid-input-identity-digest:lane:1"),
        lane(b"grid-input-identity-digest:lane:2")
    )
}
