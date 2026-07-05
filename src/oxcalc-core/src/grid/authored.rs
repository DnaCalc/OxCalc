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

/// The outcome of the public `bind_grid_formula` verb (W062 R5.1, D4 §3).
///
/// A **successful** bind of authored formula text against the real workspace
/// context (names catalog, tables, bounds). Carries the ready-to-store
/// [`GridFormulaCell`] — source text + the engine-minted normal-form key
/// (from `BoundFormula.formula_template_identity.key`) + the source channel —
/// alongside the non-fatal bind record the D4 §3 contract names.
///
/// The verb is the **only** key mint for hosts (D4 C10): the minted key lives
/// on `formula.normal_form_key` and its *format* is engine-internal, not part
/// of this type's contract. A host never hand-keys a [`GridFormulaCell`]; it
/// binds text through the verb and stores the returned cell.
///
/// [`unresolved_names`](Self::unresolved_names) is a **first-class success
/// field, not a failure** (D4 §3): a formula referencing an as-yet-unseeded
/// defined name binds successfully, lists that name here, evaluates `#NAME?`
/// until the name is seeded, and self-heals thereafter. Binding fails (a typed
/// `Err`, never a `BoundGridFormula`) only when OxFml rejects the text as a
/// formula — parse/acceptance diagnostics — mirroring `enter_grid_cell`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundGridFormula {
    /// The authored formula cell: source text + minted normal-form key +
    /// channel, ready to feed the engine's `set_formula` seam.
    pub formula: GridFormulaCell,
    /// Names the bound formula references that are **not** currently defined on
    /// the sheet (workbook- or sheet-scoped). Each will evaluate `#NAME?` until
    /// seeded; a non-empty list is a successful bind, not a rejection.
    pub unresolved_names: Vec<String>,
    /// Non-fatal bind notes surfaced by OxFml for this formula (never a
    /// rejection; rejections are the typed `Err` path).
    pub diagnostics: Vec<GridBindDiagnostic>,
}

/// One non-fatal bind note carried by a [`BoundGridFormula`] (W062 R5.1).
///
/// A flattened, host-facing projection of OxFml's `BindDiagnostic`: the note
/// text and the byte span in the source formula it concerns.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridBindDiagnostic {
    pub message: String,
    pub span_start: usize,
    pub span_end: usize,
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
    /// Project this authored record to a value-shaped [`GridAuthoredCell`] for
    /// **readout** (source text, channel, value) — *not* an engine-seeding path.
    ///
    /// Per the derived-key doctrine (W062 R5.2, D4 §3), the normal-form key is
    /// derived state minted only by the engine bind (`bind_grid_formula`); this
    /// record holds no key and this method must not fabricate an authoritative
    /// one. For a formula it fills the key slot with the source text as an inert
    /// placeholder so callers can pattern-match the source text and channel;
    /// callers that seed a derived sheet must instead mint the key through the
    /// engine (see the consumer's `build_grid_sheet` / `mint_grid_formula`),
    /// never this placeholder.
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

/// The authored *kind* of a grid cell, read from [`GridInputState`] alone
/// (W062 R5.5, D4 §5): the input-only classification a per-cell authored
/// readout carries. Deliberately **not** derived — a `Formula` cell is
/// `Formula` here whether or not it has ever evaluated, and its readout shows
/// source text, never a computed value.
///
/// `RichStub` is reserved for the ingest path's inert rich-object placeholders
/// (D4 §12, Tier-B `RichObject` cells); the current entry verbs never author
/// one, so it exists in the enum for readout completeness and is produced only
/// when such a stub is present in input truth.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GridAuthoredKind {
    /// No authored record at this address.
    Empty,
    /// A typed literal value ([`GridInputCell::Literal`]).
    Literal,
    /// A formula cell ([`GridInputCell::Formula`]); the readout carries its
    /// source text and channel, never its computed value.
    Formula,
    /// An inert rich-object stub retained from ingest (D4 §12). Not authored by
    /// the entry verbs.
    RichStub,
}

/// The editability classification of a grid cell (W062 R5.5, D4 §5).
///
/// **Derived-and-advisory-plus-enforced**: a skin may gray a non-`Editable`
/// cell out, but the contract does not depend on skins being honest — the entry
/// verbs (R5.3) enforce the *same* classification with matching typed
/// rejections via [`GridCellEditability::rejection_reason`]. The readout and the
/// verbs consume one classifier ([`classify_grid_cell_editability`]), never two
/// parallel checks.
///
/// The structural classes (`RepeatedRegionMember`, `MergedFollower`,
/// `TableStructural`) are pure functions of [`GridInputState`]. `SpillDisplay`
/// is the one class that is inherently *derived* — a cell empty in input truth
/// onto which a neighbouring formula spills — so the classifier takes the active
/// spill extents as an explicit second input alongside the input state. This is
/// the deliberate asymmetry D4 §5 names: authored *facts* (kind / source text /
/// channel) are input-only, but *editability* is the derived-advisory overlay.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GridCellEditability {
    /// A plain cell: editing is admitted by every entry verb.
    Editable,
    /// A member (non-anchor) of a repeated-formula region (a `FillRange` /
    /// shared-formula tile). The `anchor` is the region's top-left cell, which
    /// owns the R1C1-relative source.
    RepeatedRegionMember { anchor: ExcelGridCellAddress },
    /// A follower (non-anchor) cell of a merged region. The `anchor` is the
    /// merge's top-left cell, the only writable cell of the merge.
    MergedFollower { anchor: ExcelGridCellAddress },
    /// A cell displaying a value spilled from a neighbouring array formula;
    /// nothing is authored here. The `anchor` is the spilling formula's cell.
    SpillDisplay { anchor: ExcelGridCellAddress },
    /// A structural (header / totals) cell of a structured table; its content is
    /// table machinery, not a free cell. Data-region cells of a table are
    /// ordinary `Editable`.
    TableStructural { table_id: String },
}

impl GridCellEditability {
    /// Whether an edit verb admits a write to a cell with this classification.
    /// Only [`Editable`](Self::Editable) is writable; every other class is a
    /// typed rejection (see [`rejection_reason`](Self::rejection_reason)).
    #[must_use]
    pub const fn is_editable(&self) -> bool {
        matches!(self, Self::Editable)
    }

    /// The typed rejection an entry verb must return for a non-editable cell,
    /// or `None` when the cell is [`Editable`](Self::Editable). One source of
    /// truth: the verbs surface exactly this reason so a readout's
    /// classification and a verb's rejection can never diverge.
    #[must_use]
    pub fn rejection_reason(&self) -> Option<GridCellNotEditable> {
        match self {
            Self::Editable => None,
            Self::RepeatedRegionMember { anchor } => {
                Some(GridCellNotEditable::RepeatedRegionMember {
                    anchor: anchor.clone(),
                })
            }
            Self::MergedFollower { anchor } => Some(GridCellNotEditable::MergedFollower {
                anchor: anchor.clone(),
            }),
            Self::SpillDisplay { anchor } => Some(GridCellNotEditable::SpillDisplaced {
                anchor: anchor.clone(),
            }),
            Self::TableStructural { table_id } => Some(GridCellNotEditable::TableStructural {
                table_id: table_id.clone(),
            }),
        }
    }
}

/// The typed reason an entry verb rejected a write to a non-editable cell
/// (W062 R5.5, D4 §5). One-to-one with the non-`Editable`
/// [`GridCellEditability`] classes, so a rejection is exactly the readout's
/// classification made a verb error — never a silent overwrite.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GridCellNotEditable {
    /// A member of a repeated-formula region; edit the region, not a tile.
    RepeatedRegionMember { anchor: ExcelGridCellAddress },
    /// A follower of a merged region; only the merge anchor is writable.
    MergedFollower { anchor: ExcelGridCellAddress },
    /// A spill-display cell; the value is owned by the spilling formula.
    SpillDisplaced { anchor: ExcelGridCellAddress },
    /// A structural (header / totals) table cell.
    TableStructural { table_id: String },
}

/// A per-cell authored readout row (W062 R5.5, D4 §5).
///
/// The authored *facts* — `kind`, `literal`, `source_text`, `channel` — are
/// read from [`GridInputState`] alone (C11 / C6: exact, never derived). A
/// formula row shows its `source_text` (e.g. `"=A1*3"`), never its computed
/// value. `editability` is the derived-advisory-plus-enforced overlay (see
/// [`GridCellEditability`]); it is the only field that may consult derived spill
/// facts.
///
/// This never merges with the computed [`grid_view`] readout: computed values +
/// epochs come from the published (derived) readout, authored facts from input
/// state — different staleness and identity rules (D4 §5's deliberate
/// asymmetry).
///
/// [`grid_view`]: crate::consumer::OxCalcTreeContext::grid_view
#[derive(Debug, Clone, PartialEq)]
pub struct GridAuthoredCellReadout {
    pub address: ExcelGridCellAddress,
    pub kind: GridAuthoredKind,
    /// The typed literal value for a `Literal` cell; `None` otherwise.
    pub literal: Option<CalcValue>,
    /// The formula display text for a `Formula` cell (e.g. `"=A1*3"`); `None`
    /// otherwise. Never a computed value.
    pub source_text: Option<String>,
    /// The formula source channel for a `Formula` cell; `None` otherwise.
    pub channel: Option<FormulaChannelKind>,
    pub editability: GridCellEditability,
}

/// An active spill extent as the editability classifier consumes it: the
/// spilling formula's `anchor` and the rectangle its values occupy. The anchor
/// itself is a normal formula cell (authored), so it is *not* a spill-display
/// cell; only the non-anchor cells of `extent` are.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridActiveSpill {
    pub anchor: ExcelGridCellAddress,
    pub extent: GridRect,
}

/// The **single** editability classifier shared by the authored readout
/// ([`grid_authored_view`]) and the entry verbs' enforcement (W062 R5.5, D4
/// §5). Property-tested for equivalence with the verbs, not duplicated.
///
/// Precedence (first match wins), chosen so the most specific structural claim
/// dominates:
/// 1. **Table structural** — an address inside a table's header or totals rect
///    is table machinery regardless of anything else authored there.
/// 2. **Merged follower** — a non-anchor cell of a merged region.
/// 3. **Repeated-region member** — a non-anchor cell of a repeated-formula
///    region.
/// 4. **Spill display** — a non-anchor cell of an active spill extent that has
///    no authored record of its own (a cell that *is* authored is classified by
///    its authored structure above, never as spill-display).
/// 5. **Editable** — everything else.
///
/// The `spills` argument is the derived overlay; passing an empty slice yields a
/// purely input-derived classification (every structural class still resolves).
///
/// [`grid_authored_view`]: crate::consumer::OxCalcTreeContext::grid_authored_view
#[must_use]
pub fn classify_grid_cell_editability(
    input: &GridInputState,
    spills: &[GridActiveSpill],
    address: &ExcelGridCellAddress,
) -> GridCellEditability {
    // 1. Table structural: header / totals rects are machinery.
    for overlay in &input.table_overlays {
        if let Some(header) = &overlay.header_rect {
            if header.contains(address) {
                return GridCellEditability::TableStructural {
                    table_id: overlay.table_id.clone(),
                };
            }
        }
        if let Some(totals) = &overlay.totals_rect {
            if totals.contains(address) {
                return GridCellEditability::TableStructural {
                    table_id: overlay.table_id.clone(),
                };
            }
        }
    }

    // 2. Merged follower: inside a merge rect but not its top-left anchor.
    for rect in &input.merged_regions {
        if rect.contains(address) {
            let anchor = rect.top_left();
            if anchor != *address {
                return GridCellEditability::MergedFollower { anchor };
            }
        }
    }

    // 3. Repeated-region member: inside a fill rect but not its anchor.
    for region in &input.repeated_regions {
        if region.rect.contains(address) {
            let anchor = region.rect.top_left();
            if anchor != *address {
                return GridCellEditability::RepeatedRegionMember { anchor };
            }
        }
    }

    // 4. Spill display: a non-anchor cell of an active spill extent with no
    //    authored record of its own. An authored cell inside a spill extent is
    //    a spill *blocker*, classified by its authored structure (Editable
    //    here) — never as spill-display.
    if !input.cells.contains_key(address) {
        for spill in spills {
            if spill.anchor != *address && spill.extent.contains(address) {
                return GridCellEditability::SpillDisplay {
                    anchor: spill.anchor.clone(),
                };
            }
        }
    }

    // 5. Editable: a plain cell (authored or empty) with no structural claim.
    GridCellEditability::Editable
}

/// The authored *kind* of a single address, read from input truth alone.
#[must_use]
pub fn authored_kind_of(input: &GridInputState, address: &ExcelGridCellAddress) -> GridAuthoredKind {
    match input.cells.get(address) {
        None => GridAuthoredKind::Empty,
        Some(GridInputCell::Literal(_)) => GridAuthoredKind::Literal,
        Some(GridInputCell::Formula { .. }) => GridAuthoredKind::Formula,
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
