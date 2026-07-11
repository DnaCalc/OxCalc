//! oxdoc-model workbook ingestion — the load half of the document surface
//! (W062 R6, D4 §§8-13).
//!
//! This module owns the [`OxCalcIngestSink`] implementation that drives an
//! `oxdoc-model` [`DocumentEvent`] stream into the OxCalc structural model. The
//! module boundary is deliberately strict (D4 §8): only `consumer.rs` types and
//! `oxdoc_model` types cross it, so a later crate split (if wasm size ever
//! argues for one) is mechanical.
//!
//! **Scope through R6.4.** The Tier-A prelude subset (workbook header settings,
//! sheet lifecycle, *literal* cells) plus **formula ingest**: `SpreadsheetMlA1`
//! and `R1C1RelativeTemplate` cells bind through the single key mint (the
//! derived-key doctrine), `SharedFormulaRegion` expands as a repeated-formula
//! region, `FormulaTopology` routes record kinds (shared / legacy-CSE array /
//! data-table / unknown), `FileCached` caches seed the pre-engine publication,
//! and OxFml-unacceptable text *degrades* (retained text + cache + a
//! [`BindDegradation`] ledger row) rather than failing the load (D4 §10). Names,
//! tables, merges (R6.3), and the **inert Tier-B store** (R6.4,
//! [`IngestedDocumentFacts`]) — every Tier-B variant retained verbatim, its
//! digest driving a `#workbook-ingest` meta-child's revision identity, with
//! inert overlay rects for the rect-claiming families — have landed. The public
//! one-call verb (R6.5) and output projection (R6.6) are later beads. Every
//! `DocumentEvent` variant is nonetheless *accounted* for here — consumed
//! (Tier A) or retained + ledgered (Tier B/X) — so nothing is ever silently
//! dropped (D4 §12).
//!
//! **The honesty enforcement (D4 §12).** [`OxCalcWorkbookIngestSink::feature`]
//! ends in an *exhaustive* match over [`OxCalcDocumentFeature`] with **no
//! wildcard arm**. A 30th upstream feature variant is therefore a compile error
//! in this module, not a silent drop — that is the C13 tripwire that keeps the
//! `#[non_exhaustive]` growth of `DocumentEvent` loud.

use std::collections::BTreeMap;

use oxdoc_model::{
    CellChunk, CellPayload, CellRangeSpec, DefinedNameMetadataSpec, DefinedNameSpec, DocumentEvent,
    FormulaRecord, FormulaRecordKind, FormulaTextKind, MergedCellRegions, OxCalcCachedValue,
    OxCalcCellChunk, OxCalcCellInput, OxCalcCellValue, OxCalcDocumentFeature, OxCalcFormulaInput,
    OxCalcIngestError, OxCalcIngestSink, OxCalcWorkbookPrelude, PackedCellAddr,
    SharedFormulaRegion, SharedStringEntry, SheetRef, TableSpec, WorkbookHeader,
    WorkbookModelAccess, WorkbookModelOutput, drive_oxcalc_ingest,
    drive_oxcalc_ingest_from_model_access,
};
use oxfml_core::source::FormulaChannelKind;
use oxfunc_core::value::{CalcValue, CoreValue, ExcelText, WorksheetErrorCode};

use crate::consumer::{
    OxCalcDocumentContext, OxCalcDocumentError, OxCalcTreeWorkspaceCreate, OxCalcTreeWorkspaceId,
};
use crate::grid::authored::GridAuthoredCell;
use crate::grid::coords::ExcelGridCellAddress;
use crate::workbook_settings::{CalcMode, DateSystem, IterationSettings, WorkbookCalcSettings};

/// Every `oxdoc-model` [`DocumentEvent`] variant, as a value-level tag (D4 §12).
///
/// The disposition table (D4 §12) is 29 rows; this enum has exactly one
/// discriminant per row so the load report can account for *every* variant
/// (consumed or ledgered) and the no-silent-loss invariant is testable as one
/// assertion. The three prelude events (`WorkbookHeader`, `StringTable`,
/// `StyleTable`) and the three sheet/cell method-borne events (`SheetBegin`,
/// `SheetEnd`, `CellChunk`) are surfaced through the sink's dedicated methods;
/// the remaining 23 arrive through [`OxCalcWorkbookIngestSink::feature`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DocumentVariantTag {
    WorkbookHeader,
    StringTable,
    StyleTable,
    DifferentialStyleTable,
    SheetBegin,
    SheetEnd,
    SheetDimension,
    ColumnProps,
    RowProps,
    MergedCellRegions,
    SheetViewState,
    Hyperlinks,
    DataValidations,
    AutoFilter,
    SortState,
    CommentNotice,
    ThreadedCommentPeople,
    SheetReviewComments,
    DrawingFormControls,
    CellFormatRuns,
    ConditionalFormatRegion,
    FormulaTopology,
    CellChunk,
    SharedFormulaRegion,
    TableOverlay,
    DefinedName,
    ExternalLink,
    CalcChainHint,
    OpaquePartNotice,
}

impl DocumentVariantTag {
    /// Every tag, in disposition-table order. The length is the source of the
    /// 29/29 accounting check (D4 §12 count check).
    pub const ALL: [DocumentVariantTag; 29] = [
        DocumentVariantTag::WorkbookHeader,
        DocumentVariantTag::StringTable,
        DocumentVariantTag::StyleTable,
        DocumentVariantTag::DifferentialStyleTable,
        DocumentVariantTag::SheetBegin,
        DocumentVariantTag::SheetEnd,
        DocumentVariantTag::SheetDimension,
        DocumentVariantTag::ColumnProps,
        DocumentVariantTag::RowProps,
        DocumentVariantTag::MergedCellRegions,
        DocumentVariantTag::SheetViewState,
        DocumentVariantTag::Hyperlinks,
        DocumentVariantTag::DataValidations,
        DocumentVariantTag::AutoFilter,
        DocumentVariantTag::SortState,
        DocumentVariantTag::CommentNotice,
        DocumentVariantTag::ThreadedCommentPeople,
        DocumentVariantTag::SheetReviewComments,
        DocumentVariantTag::DrawingFormControls,
        DocumentVariantTag::CellFormatRuns,
        DocumentVariantTag::ConditionalFormatRegion,
        DocumentVariantTag::FormulaTopology,
        DocumentVariantTag::CellChunk,
        DocumentVariantTag::SharedFormulaRegion,
        DocumentVariantTag::TableOverlay,
        DocumentVariantTag::DefinedName,
        DocumentVariantTag::ExternalLink,
        DocumentVariantTag::CalcChainHint,
        DocumentVariantTag::OpaquePartNotice,
    ];
}

/// The tier a variant lands in (D4 §12). Every tier is ledgered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IngestTier {
    /// Calculation-bearing: enters the calc model (authored truth, settings,
    /// lifecycle). Round-trips from the model.
    A,
    /// Inert document fact: retained verbatim in the inert store, never
    /// calc-visible, replayed verbatim at save.
    B,
    /// Typed exclusion: not retained; ledgered with the reason. Exactly one
    /// variant (`CalcChainHint`) earns this.
    X,
}

/// One row of the ingest fidelity ledger (D4 §§9, 12, 13).
///
/// Every retention and every exclusion produces a row; nothing is droppable
/// without one. In this bead the Tier-B rows are *stubs* (the real inert store
/// lands in R6.4): the row records the variant, its tier, a stable disposition
/// code, and how many instances were observed. A `handled` row for a Tier-A
/// variant records that the variant was *consumed* into the calc model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IngestLedgerRow {
    /// The `DocumentEvent` variant this row accounts for.
    pub variant: DocumentVariantTag,
    /// The tier this variant lands in (D4 §12).
    pub tier: IngestTier,
    /// A stable, machine-readable disposition code (e.g.
    /// `"consumed"`, `"retained-inert"`, `"excluded-engine-derives-order"`).
    /// Mirrors the D4 §12 disposition column at code granularity.
    pub disposition: &'static str,
    /// How many instances of this variant the stream carried. Zero rows are
    /// never emitted — a ledger row's presence means the variant was observed.
    pub observed: u32,
}

/// The outcome of a workbook load (D4 §9).
///
/// Carries the structural counts, the ingest fidelity ledger (one row per
/// observed variant — the no-silent-loss home), the bind-degradation list
/// (empty in this bead; populated in R6.2), and the recalc path that load ran.
///
/// **The no-silent-loss invariant (D4 §9).** Every `DocumentEvent` in the
/// stream is either *consumed* (Tier A, folded into the calc model) or has a
/// *ledger row* (Tier B/X). [`WorkbookLoadReport::accounts_for_all_variants`]
/// checks that every observed variant has a ledger disposition; a variant with
/// no row is a silent path and fails the invariant.
#[derive(Debug, Clone, PartialEq)]
pub struct WorkbookLoadReport {
    /// Number of sheets created (Tier A, D4 §12 row 5).
    pub sheets: u32,
    /// Number of literal cells folded into authored truth (Tier A, row 23).
    pub cells: u32,
    /// Number of formula cells that **bound** through the single key mint and
    /// entered the calc graph (D4 §10, row 23). The honest per-cell disposition
    /// the R6.1 carry-forward asks for: a formula-bearing load reports
    /// `formulas_bound > 0`, distinguishable from a literals-only load. Degraded
    /// formulas are counted in [`bind_degradations`](Self::bind_degradations),
    /// not here.
    pub formulas_bound: u32,
    /// Number of `RichStub(u32)` cells observed (D4 §12 row 23). The authored
    /// `GridCellInput::RichStub` round-trip is a later cell-input bead (row 23 is
    /// Tier A, distinct from the row-19 `DrawingFormControls`→`RichObject` overlay
    /// R6.4 owns); counted here as deferred, never consumed as a fake value.
    pub rich_stubs_deferred: u32,
    /// Number of formula cells retained as `NotCalcModeled` (DataTable / Unknown
    /// topology records, D4 §12 row 22): they publish their `FileCached` value
    /// and never enter the calc graph.
    pub not_calc_modeled: u32,
    /// Number of region-managed cells (`Formula { region: Some(id), .. }`) whose
    /// referenced `SharedFormulaRegion` never arrived — a malformed/partial
    /// stream (oxdoc-model does NOT enforce the cell↔region pairing). Their cache
    /// still publishes (so the cell renders), but no formula backs them: the
    /// no-silent-loss regime (C13) accounts for them here rather than dropping
    /// them, and the `SharedFormulaRegion` ledger row carries the disposition.
    pub region_cells_unbacked: u32,
    /// Number of defined names installed into the calc model (D4 §12 row 26).
    /// Counts both static (rect-denoting) and dynamic names, at either scope. A
    /// name that could NOT be installed — an unresolvable target sheet, an
    /// unmodelable sheet scope, or a rejected dynamic defining formula — is
    /// excluded from this count AND surfaced as a [`BindDegradation`] row in
    /// [`bind_degradations`](Self::bind_degradations) carrying its text + reason,
    /// so a dropped name is never silent (C13). It never fabricates a binding on
    /// the wrong sheet.
    pub names: u32,
    /// Number of structured tables installed (D4 §12 row 25). A table whose sheet
    /// or range could not be resolved is dropped from this count.
    pub tables: u32,
    /// Defined-name metadata (comment/hidden/function flags/raw attrs), keyed by
    /// name (D4 §12 row 26, Tier-B half). A read-back copy of the retention home
    /// [`IngestedDocumentFacts::name_metadata`] (§13) — the store is the home; the
    /// report echoes it for inspection. Only names carrying non-empty metadata
    /// appear.
    pub name_metadata: Vec<IngestedDefinedNameMetadata>,
    /// The ingest fidelity ledger: one row per *observed* variant, in
    /// disposition-table order.
    pub ledger: Vec<IngestLedgerRow>,
    /// Bind degradations (D4 §10): one row per authored fact retained but NOT
    /// installed into the calc model, so nothing is silently dropped (C13). Two
    /// sources feed this one channel:
    /// - a **formula cell** whose text OxFml rejected (address `R{row}C{col}`):
    ///   the text is retained, the cell publishes its `FileCached` cache (or a
    ///   `#NAME?`-class error), and ingest still SUCCEEDS;
    /// - a **defined name** that could not be installed (address `name:{name}`):
    ///   an unresolvable target sheet, an unmodelable sheet scope, or a rejected
    ///   dynamic defining formula. The name text + reason are retained here.
    ///
    /// Empty when every formula bound and every name installed.
    pub bind_degradations: Vec<BindDegradation>,
    /// Inert overlay rects claimed at load (D4 §12 rows 19/21/22, §13): the
    /// rect-claiming Tier-B families — legacy-CSE arrays (`Cse`), conditional
    /// formats (`ConditionalFormat`), and cell-anchored drawing/form controls
    /// (`RichObject`) — each claim an inert overlay rect carrying **no** engine
    /// calc semantics. A read-back copy of [`IngestedDocumentFacts::inert_overlays`]
    /// (the spatial index into the retention store): the store is the home, the
    /// overlay is the index. Materialize the engine `GridOverlayExtension` seats
    /// via [`IngestedDocumentFacts::overlay_seats_for_sheet`].
    pub inert_overlays: Vec<IngestedInertOverlay>,
    /// How many bound formula cells reference an **external** workbook
    /// (`[Book2]Sheet1!A1`, D4 §14): each binds normally (authored text retained)
    /// but publishes its `FileCached` value **pinned** — recalc never evaluates it
    /// and never clobbers the cache — and carries an `ExternalReferenceNotLinked`
    /// disposition in [`bind_degradations`](Self::bind_degradations). A subset of
    /// [`formulas_bound`](Self::formulas_bound); surfaced here so a caller sees
    /// how many cells hold pinned external caches (the D2 §5 upgrade worklist).
    /// The `ExternalLinkSpec` targets themselves live in the Tier-B store
    /// ([`IngestedDocumentFacts::external_links`]).
    pub external_ref_cells_pinned: u32,
    /// One row per external-referencing formula cell whose `FileCached` value was
    /// pinned at load (D4 §14). The no-silent-loss ledger for external references:
    /// every external-referencing cell is a typed row here — its address, retained
    /// text, and the `ExternalReferenceNotLinked` reason — never a bare skip. The
    /// count [`external_ref_cells_pinned`](Self::external_ref_cells_pinned) equals
    /// this list's length. Empty when the workbook has no external references.
    pub external_reference_pins: Vec<ExternalReferencePin>,
    /// How many engine recalc passes the load ran (D4 §6 load-recalc policy /
    /// perf counter). `CalcMode::Automatic` → one open-recalc pass per
    /// calc-bearing sheet (non-zero, published values `Calculated`);
    /// `CalcMode::Manual` → **zero** (the workbook renders from `FileCached`
    /// caches until F9). The Manual-zero-eval acceptance asserts this is `0`.
    pub engine_recalcs_at_load: u32,
    /// Which load-recalc path ran (D4 §6/§9): `Automatic` (one open-recalc,
    /// published values `Calculated`, differential clean) or `Manual` (no engine
    /// evaluation, published values `FileCached` until an explicit
    /// `recalculate_workbook`). Set from the loaded workbook's `CalcMode`, an
    /// ordinary revision-1 setting — not ingest-private state (D4 §6).
    pub recalc_path: LoadRecalcPath,
}

impl WorkbookLoadReport {
    /// The no-silent-loss invariant (D4 §9): every observed `DocumentEvent`
    /// variant has a ledger disposition. Returns `Err` naming the first
    /// variant that was observed-but-unaccounted (a silent path).
    ///
    /// `observed` is the set of variant tags the stream actually carried (the
    /// sink records it during the drive). A variant that was observed but has
    /// no ledger row is exactly a silent drop — the failure this invariant
    /// exists to catch.
    pub fn accounts_for_all_variants(
        &self,
        observed: &[DocumentVariantTag],
    ) -> Result<(), DocumentVariantTag> {
        for variant in observed {
            if !self.ledger.iter().any(|row| row.variant == *variant) {
                return Err(*variant);
            }
        }
        Ok(())
    }
}

/// A formula bind degradation (D4 §10): a formula cell whose text OxFml could
/// not accept as a formula (corrupt/unsupported grammar). The authored text is
/// retained verbatim (round-trip safe), the cell publishes its `FileCached`
/// value if the file carried one else a `#NAME?`-class typed error, and ingest
/// SUCCEEDS. **The formula is never discarded and never silently rewritten.**
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindDegradation {
    /// The degraded formula cell's address, in `R{row}C{col}` one-based notation
    /// (the sheet-local coordinate the ingest carries; the workbook/sheet token
    /// is implied by the load's single-workbook scope).
    pub address: String,
    /// The authored formula text that was retained rather than bound (with the
    /// leading `=` restored, exactly as it would round-trip to the file).
    pub text: String,
    /// Diagnostics OxFml produced when it rejected the text as a formula
    /// (`message [start..end]`-rendered). Non-empty — the rejection reason is
    /// always carried, never invented.
    pub diagnostics: Vec<String>,
}

/// An external-workbook-referencing formula cell pinned at load (D4 §14).
///
/// A formula whose bound references include an external-workbook token
/// (`[Book2]Sheet1!A1`) **binds normally** — its authored text is retained and
/// it enters the calc graph — but OxCalc cannot honestly evaluate it in R6 (D2
/// §5 cross-workspace routing is not built here). So its `FileCached` value is
/// **pinned** (Excel-without-the-source-open: the last-fetched external cache
/// renders), a `recalculate_workbook` neither evaluates the cell nor clobbers
/// the cache, and this row records the disposition — the no-silent-loss ledger
/// entry the D4 §14 contract requires (never a bare skip). The pin upgrades to a
/// real cross-workspace edge when D2 §5 lands; these rows are that worklist.
///
/// **This is a successful bind, not a degradation** (distinct from
/// [`BindDegradation`], which is OxFml-rejected text): the cell is in the graph,
/// it simply publishes a pinned pre-engine value with a named reason.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalReferencePin {
    /// The pinned cell's address, in `R{row}C{col}` one-based notation (the
    /// sheet-local coordinate; the workbook/sheet token is implied by the load's
    /// single-workbook scope, as [`BindDegradation::address`]).
    pub address: String,
    /// The authored formula text retained verbatim (leading `=` restored) — the
    /// external reference round-trips to the file unchanged.
    pub text: String,
    /// The named reason this cell holds a pinned cache: always
    /// `"ExternalReferenceNotLinked"` in R6 (the D2 §5 typed exclusion). Carried
    /// as a field rather than implied so the ledger row names its own disposition.
    pub reason: &'static str,
    /// Whether a `FileCached` value backed the pin. `true` when the file carried
    /// a cache for the cell (the pinned render value); `false` when it did not —
    /// in which case the cell publishes a `#REF!`-class typed error pinned (a
    /// newly authored external ref with no cache follows D2's `#REF!` rule; a
    /// loaded external ref with no cache is a malformed file, handled the same
    /// honest way rather than fabricating a value).
    pub had_file_cache: bool,
}

/// The named reason an external-referencing cell holds a pinned cache (D4 §14 /
/// D2 §5). The single trust point for the external-pin ledger disposition.
pub const EXTERNAL_REFERENCE_NOT_LINKED: &str = "ExternalReferenceNotLinked";

/// An inert overlay rect claimed at load (D4 §12 rows 19/21/22, §13).
///
/// The rect-claiming Tier-B families — legacy-CSE arrays (row 22), conditional
/// formats (row 21), and cell-anchored drawing/form controls (row 19) — claim an
/// inert overlay rect: a *spatial index* into the retained document facts,
/// carrying **no** engine calc semantics. The retention home is the typed
/// [`IngestedDocumentFacts`] store; this record is a rect + a [`payload`] store
/// key into it, so a renderer/save can find the retained spec. Rect-less Tier-B
/// families (styles, people, links, …) live in the store with **no** overlay.
///
/// [`payload`]: IngestedInertOverlay::payload
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct IngestedInertOverlay {
    /// Which overlay family claims the rect.
    pub kind: InertOverlayKind,
    /// The upstream sheet id the rect belongs to.
    pub sheet_id: u32,
    /// The claimed rectangle in one-based `(top_row, left_col, bottom_row,
    /// right_col)` coordinates.
    pub rect: (u32, u32, u32, u32),
    /// The store key this rect indexes into (`{family}:{sheet_id}#{ordinal}`):
    /// the retention home is the store, the overlay is only the spatial index
    /// (D4 §13). Projected as the [`GridOverlayExtension::payload`] when the
    /// overlay seat is materialized.
    ///
    /// [`GridOverlayExtension::payload`]: crate::grid::machine::GridOverlayExtension::payload
    pub payload: String,
}

/// The overlay family of an [`IngestedInertOverlay`] (D4 §12 rows 19/21/22).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum InertOverlayKind {
    /// A legacy-CSE array rect. The cells inside ingest as normal formulas; the
    /// overlay itself is inert (no array-formula eval semantics are built).
    Cse,
    /// A conditional-format region rect (row 21). The full spec is retained in
    /// the store; the overlay is a spatial index. CF rules are NOT bound in R6.
    ConditionalFormat,
    /// A cell-anchored drawing/form-control rect (row 19). The full
    /// `DrawingFormControlsSpec` is retained in the store; the overlay indexes
    /// the anchor rect so spills/axis edits can see it (inert today).
    RichObject,
}

impl InertOverlayKind {
    /// The store-key family prefix + engine [`OverlayKind`] this inert family
    /// projects to when the overlay seat is materialized.
    ///
    /// [`OverlayKind`]: crate::grid::machine::OverlayKind
    #[must_use]
    fn overlay_kind(self) -> crate::grid::machine::OverlayKind {
        match self {
            InertOverlayKind::Cse => crate::grid::machine::OverlayKind::Cse,
            InertOverlayKind::ConditionalFormat => {
                crate::grid::machine::OverlayKind::ConditionalFormat
            }
            InertOverlayKind::RichObject => crate::grid::machine::OverlayKind::RichObject,
        }
    }

    /// The store-key family prefix (stable, machine-readable).
    #[must_use]
    fn family_prefix(self) -> &'static str {
        match self {
            InertOverlayKind::Cse => "cse",
            InertOverlayKind::ConditionalFormat => "cf",
            InertOverlayKind::RichObject => "rich",
        }
    }
}

/// The retention home for a workbook's inert Tier-B document facts (D4 §13).
///
/// **This is the no-silent-loss contract's headline for R6.4.** Every Tier-B
/// variant the disposition table (D4 §12) names is retained here **verbatim**
/// (its owned upstream spec, byte-faithful), so the output projection (R6.6) can
/// replay it at save with no fidelity loss. A ledger row with no stored payload
/// would be a silent loss at save time; the store closes that gap. Rect-claiming
/// families (CF, cell-anchored drawing/form controls, legacy-CSE arrays)
/// additionally get an inert overlay rect (a spatial index into this store — see
/// [`inert_overlays`](Self::inert_overlays)); rect-less families (styles, dxfs,
/// people, links, …) live here with **no** overlay, proving the store — not the
/// overlay — is the retention home.
///
/// Held as `Arc<IngestedDocumentFacts>` on live workspace state and cloned by
/// pointer onto retained revisions (the `deleted_table_facts` retention shape).
/// **Immutable after load** in R6 — no edit verb touches it — so its identity is
/// a load-time digest ([`digest`](Self::digest)) written into a
/// `#workbook-ingest` meta-child, giving it a revision-identity contribution
/// with zero new snapshot plumbing (D1 §4/§5).
///
/// The field order and the derived `Serialize` are load-bearing: [`digest`] is a
/// stable hash of the serialized store, so two loads with identical Tier-B facts
/// digest identically and a single perturbed retained fact moves the digest (and
/// therefore the revision identity). All fields are accumulated in stream order,
/// so the serialization is deterministic.
///
/// [`digest`]: Self::digest
#[derive(Debug, Clone, Default, PartialEq, serde::Serialize)]
pub struct IngestedDocumentFacts {
    /// The workbook style table (row 3, prelude). `None` when the prelude carried
    /// none. Number formats included (display-only; see D4 T5).
    pub style_table: Option<oxdoc_model::StyleTableSpec>,
    /// Differential (dxf) styles (row 4), in stream order.
    pub differential_styles: Vec<oxdoc_model::DifferentialStyleSpec>,
    /// Per-sheet `SheetDimension` claims (row 7). The grid bounds are set by
    /// profile policy, not by this claim; retained for round-trip (save writes
    /// the recomputed extent, using this only if the authored extent shrank).
    pub sheet_dimensions: Vec<oxdoc_model::SheetDimensionSpec>,
    /// Column property runs (row 8), keyed by upstream sheet id. Hidden/width/
    /// outline runs retained inert (a named SUBTOTAL hidden-column gap, D4 §12).
    pub column_props: Vec<SheetAxisRuns>,
    /// Row property runs (row 9), keyed by upstream sheet id (same gap family).
    pub row_props: Vec<SheetAxisRuns>,
    /// Sheet view state (row 11): frozen panes, selection, zoom.
    pub sheet_views: Vec<oxdoc_model::SheetViewState>,
    /// Hyperlinks (row 12).
    pub hyperlinks: Vec<oxdoc_model::HyperlinksSpec>,
    /// Data validations (row 13). Validation formulas are NOT bound — UI-gate
    /// facts, not calc-graph members.
    pub data_validations: Vec<oxdoc_model::DataValidationsSpec>,
    /// Auto-filter state (row 14; named filter/SUBTOTAL gap).
    pub auto_filters: Vec<oxdoc_model::AutoFilterSpec>,
    /// Sort state (row 15).
    pub sort_states: Vec<oxdoc_model::SortStateSpec>,
    /// Legacy comment notices (row 16).
    pub comment_notices: Vec<oxdoc_model::CommentNoticeSpec>,
    /// Threaded-comment people (row 17). A rect-less family: retained here with
    /// no overlay, proving the store is the home.
    pub threaded_comment_people: Vec<oxdoc_model::ThreadedCommentPeopleSpec>,
    /// Sheet review comments (row 18).
    pub sheet_review_comments: Vec<oxdoc_model::SheetReviewCommentsSpec>,
    /// Drawing/form-control specs (row 19). Retained verbatim; controls whose
    /// host drawing object carries a cell anchor additionally claim an inert
    /// `RichObject` overlay rect (see [`inert_overlays`](Self::inert_overlays)).
    pub drawing_form_controls: Vec<oxdoc_model::DrawingFormControlsSpec>,
    /// Per-cell format runs (row 20), keyed by upstream sheet id.
    pub cell_format_runs: Vec<SheetCellFormatRuns>,
    /// Conditional-format regions (row 21). Full spec retained; each region
    /// claims an inert `ConditionalFormat` overlay rect. CF rule formulas are
    /// NOT bound in R6.
    pub conditional_formats: Vec<oxdoc_model::ConditionalFormatRegion>,
    /// Formula-topology attrs + unsupported fragments (row 22). The whole
    /// `FormulaTopology` value is retained for round-trip — the routed calc
    /// dispositions (bind / CSE overlay / not-calc-modeled) are Tier A and live
    /// in the calc model; the file-topology metadata (per-record `attrs`, the
    /// topology's + records' `unsupported_fragments`) is Tier B and lives here.
    pub formula_topologies: Vec<oxdoc_model::FormulaTopology>,
    /// Defined-name metadata (row 26, Tier-B half), keyed by name: the
    /// comment/hidden/function flags + raw attrs. Only names carrying non-empty
    /// metadata appear.
    pub name_metadata: Vec<IngestedDefinedNameMetadata>,
    /// External-link targets (row 27). Retained verbatim; the bind-degradation
    /// contract (§14) is R6.5's — this is the ingest-side retention.
    pub external_links: Vec<oxdoc_model::ExternalLinkSpec>,
    /// Opaque-part notices (row 29). `GeometryCoupling::{SheetAnchor,SourceRange}`
    /// notices carry a live staleness gap surfaced in the ledger.
    pub opaque_notices: Vec<oxdoc_model::OpaquePartNotice>,
    /// Unknown BIFF error bytes retained at their cell (R6.1): a cell whose error
    /// code has no classic BIFF mapping publishes `#VALUE!` but its **raw byte**
    /// is retained here so save writes the byte back verbatim, never laundering
    /// an unknown code into a known one (D4 §10). Keyed by `(sheet, row, col)`.
    pub unknown_error_bytes: Vec<UnknownErrorByteRetention>,
    /// The inert overlay rects the rect-claiming families claim (rows 19/21/22):
    /// a *spatial index* into the store. Each carries its `payload` store key.
    /// Rect-less families are absent here (they live in the typed fields above).
    pub inert_overlays: Vec<IngestedInertOverlay>,
    /// The upstream `SheetRef.sheet_id` of each loaded sheet, in stream/creation
    /// order (W062 R6.6, D4 §7a). This is a *projection-support index*, not a
    /// Tier-B fact: the save projection walks the workbook's sheets in registry
    /// order (which equals stream order — sheets are created in stream order at
    /// load, C3) and reads the sheet at position `i`'s upstream id from
    /// `sheet_stream_ids[i]`, so the re-emitted `SheetBegin(SheetRef{sheet_id})`
    /// carries the *same* upstream id every sheet-scoped Tier-B fact stores inside
    /// it (`SheetViewState.sheet_id`, `MergedCellRegions.sheet_id`, …). Without
    /// this, the projection could not reproduce the upstream sheet ids and the
    /// stream validator (`ensure_sheet_id`) would reject a Tier-B fact whose
    /// `sheet_id` did not match the open `SheetBegin`.
    ///
    /// Deliberately `#[serde(skip)]` and excluded from [`is_empty`](Self::is_empty):
    /// the store's digest is a *portable* Tier-B identity (two identical Tier-B
    /// loads digest identically), and while the sheet ids are themselves portable,
    /// they are structural bookkeeping, not retained document facts — so they must
    /// not perturb the identity digest nor force a `#workbook-ingest` meta-child on
    /// an otherwise Tier-B-empty load.
    #[serde(skip)]
    pub sheet_stream_ids: Vec<u32>,
}

impl IngestedDocumentFacts {
    /// A stable content digest of the retained store (D4 §13): the load-time
    /// identity written into the `#workbook-ingest` meta-child. Two stores with
    /// identical retained facts digest identically; perturbing ONE retained fact
    /// moves the digest (and therefore the revision identity). Computed from the
    /// canonical serialization (all fields are in stream order, so it is
    /// deterministic), hashed with the same `DefaultHasher` the workspace-
    /// identity strings use, and rendered as a fixed-width hex token.
    #[must_use]
    pub fn digest(&self) -> String {
        use std::hash::Hasher as _;
        // Serialization cannot fail: every field is a plain serde-derived spec
        // with no non-string map keys or float NaN traps in the identity path.
        // A serialization error would still be accounted (not silent): fall back
        // to a marker that differs from every real digest so identity still moves
        // rather than silently colliding.
        let serialized = serde_json::to_string(self)
            .unwrap_or_else(|error| format!("oxcalc-ingest-facts-unserializable:{error}"));
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        hasher.write(b"oxcalc-workbook-ingest:v1");
        hasher.write(serialized.as_bytes());
        format!("ingest-{:016x}", hasher.finish())
    }

    /// Whether the store retained nothing (every Tier-B field empty). A load with
    /// no Tier-B facts at all writes no `#workbook-ingest` meta-child (its digest
    /// would be the constant empty-store digest, contributing nothing an absent
    /// child would not — the settings-subtree "absent means default" discipline).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.style_table.is_none()
            && self.differential_styles.is_empty()
            && self.sheet_dimensions.is_empty()
            && self.column_props.is_empty()
            && self.row_props.is_empty()
            && self.sheet_views.is_empty()
            && self.hyperlinks.is_empty()
            && self.data_validations.is_empty()
            && self.auto_filters.is_empty()
            && self.sort_states.is_empty()
            && self.comment_notices.is_empty()
            && self.threaded_comment_people.is_empty()
            && self.sheet_review_comments.is_empty()
            && self.drawing_form_controls.is_empty()
            && self.cell_format_runs.is_empty()
            && self.conditional_formats.is_empty()
            && self.formula_topologies.is_empty()
            && self.name_metadata.is_empty()
            && self.external_links.is_empty()
            && self.opaque_notices.is_empty()
            && self.unknown_error_bytes.is_empty()
            && self.inert_overlays.is_empty()
    }

    /// Project the rect-claiming families' inert overlay seats as engine
    /// [`GridOverlayExtension`] values (D4 §13): the spatial-index readout. The
    /// store is the retention home; these are index-only, built with the inert
    /// blockage/admission the overlay seam constructs today
    /// (`SpillBlock::None` / `refuses_axis_edit: false`), `payload` = the store
    /// key. Rect-less families produce nothing here. `bounds` is the sheet grid
    /// bounds the rect is expressed against; the `workbook`/`sheet` tokens name
    /// the grid the rect lives on.
    ///
    /// This is a *readout* off the store, not a plumb into the engine's
    /// `GridOverlaySet` (which has no extension storage yet — that is CSE-1 /
    /// CF-1 / RICH-1, deliberately out of R6 scope). It lets a consumer inspect
    /// the inert rects with their store keys without the engine owning them.
    ///
    /// [`GridOverlayExtension`]: crate::grid::machine::GridOverlayExtension
    #[must_use]
    pub fn overlay_seats_for_sheet(
        &self,
        upstream_sheet_id: u32,
        workbook_token: &str,
        sheet_token: &str,
        bounds: crate::grid::coords::ExcelGridBounds,
    ) -> Vec<crate::grid::machine::GridOverlayExtension> {
        self.inert_overlays
            .iter()
            .filter(|overlay| overlay.sheet_id == upstream_sheet_id)
            .filter_map(|overlay| {
                let (top_row, left_col, bottom_row, right_col) = overlay.rect;
                let claimed_rect = crate::grid::geometry::GridRect::new(
                    workbook_token.to_string(),
                    sheet_token.to_string(),
                    top_row,
                    left_col,
                    bottom_row,
                    right_col,
                    bounds,
                )
                .ok()?;
                Some(crate::grid::machine::GridOverlayExtension {
                    kind_tag: overlay.kind.overlay_kind(),
                    claimed_rect,
                    // Inert as constructed today (D4 §13): the overlay is a
                    // spatial index, not an engine-active blocker/refuser.
                    block_mode: crate::grid::machine::SpillBlock::None,
                    refuses_axis_edit: false,
                    payload: overlay.payload.clone(),
                })
            })
            .collect()
    }
}

/// A per-sheet axis-run retention (D4 §12 rows 8/9): the upstream sheet id the
/// runs belong to (the feature event carries only `&[AxisRun]`, so the sheet is
/// the open sheet at the event) plus the owned runs.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct SheetAxisRuns {
    pub sheet_id: u32,
    pub runs: Vec<oxdoc_model::AxisRun>,
}

/// A per-sheet cell-format-run retention (D4 §12 row 20): the upstream sheet id
/// plus the owned runs (the feature event carries only `&[CellFormatRun]`).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct SheetCellFormatRuns {
    pub sheet_id: u32,
    pub runs: Vec<oxdoc_model::CellFormatRun>,
}

/// An unknown BIFF error byte retained at its cell (R6.1, D4 §10). The cell
/// publishes `#VALUE!`, but the raw byte is retained here so a save writes it
/// back verbatim rather than laundering the unknown code into `#VALUE!`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct UnknownErrorByteRetention {
    /// The upstream sheet id the cell lives on.
    pub sheet_id: u32,
    /// One-based row.
    pub row: u32,
    /// One-based column.
    pub col: u32,
    /// The raw error byte the file carried (outside the classic BIFF set).
    pub raw_byte: u8,
}

/// Which recalc path the load ran (D4 §6/§9).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LoadRecalcPath {
    /// No recalc issued. The pre-R6.5 default (formula binding without the
    /// mode-conditional open-recalc); retained as the `Default` so an
    /// unpopulated report reads as "no load-recalc policy applied".
    #[default]
    None,
    /// `CalcMode::Automatic` open-recalc (D4 §6): the load issued exactly one
    /// engine pass per calc-bearing sheet (Excel's open-recalc), so published
    /// values are engine `Calculated`.
    Automatic,
    /// `CalcMode::Manual` (D4 §6): the load ran **no** engine evaluation; the
    /// workbook renders from `FileCached` caches until an explicit F9
    /// (`recalculate_workbook`).
    Manual,
}

/// Errors the ingest sink or its commit can return.
#[derive(Debug, thiserror::Error)]
pub enum OxCalcWorkbookIngestError {
    /// A cell address, sheet id, or value shape the ingest cannot honor.
    #[error("ingest rejected the document: {0}")]
    Rejected(String),
    /// The single-transaction commit hit a structural/engine error from the
    /// consumer (e.g. a case-fold-duplicate sheet name — D1 `validate()`).
    #[error(transparent)]
    Commit(#[from] OxCalcDocumentError),
}

impl From<OxCalcWorkbookIngestError> for OxCalcDocumentError {
    /// Lower the ingest error into the document error the public
    /// `load_workbook_model` verb returns (W062 R6.5, D4 §9). A `Commit` error is
    /// already an [`OxCalcDocumentError`] (the single-transaction load's own
    /// structural/engine failure — the load-fail class); a `Rejected` stream/sink
    /// mismatch becomes [`OxCalcDocumentError::WorkbookIngestRejected`].
    fn from(error: OxCalcWorkbookIngestError) -> Self {
        match error {
            OxCalcWorkbookIngestError::Commit(inner) => inner,
            OxCalcWorkbookIngestError::Rejected(detail) => {
                OxCalcDocumentError::WorkbookIngestRejected { detail }
            }
        }
    }
}

/// A per-sheet accumulation: the sheet's display name, its engine-facing tokens,
/// its literal cells, its formula cells, its shared/CSE regions, and the
/// cached-value publications the file carried — gathered during the drive and
/// installed at commit.
#[derive(Debug, Clone, Default)]
struct SheetAccumulator {
    /// The upstream sheet id (`SheetRef.sheet_id`), the key of the ingest-local
    /// map (D4 §9).
    upstream_sheet_id: u32,
    /// The display name (`SheetRef.name`) → node symbol → sheet registry.
    display_name: String,
    /// Literal cells, in stream order (address, value). Last-write-wins into the
    /// address-keyed authored map at install, matching the engine's overwrite
    /// semantics.
    literals: Vec<(u32, u32, CalcValue)>,
    /// Formula cells (D4 §10): `(row, col, source_text-with-'=', channel,
    /// cached)`. The commit builder binds each through the single key mint; a
    /// cell OxFml rejects degrades (retained text + cache + ledger row) instead
    /// of failing the load.
    formulas: Vec<IngestFormula>,
    /// Shared-formula regions (`SharedFormulaRegion`, D4 §12 row 24) and legacy
    /// CSE array rects (`FormulaTopology` `Array`, row 22): one R1C1 template
    /// tiled over a rect, installed as a repeated-formula region at commit.
    repeated_regions: Vec<IngestRepeatedRegion>,
    /// FileCached publications for region-managed cells (D4 §12 row 24): a shared
    /// region's member/anchor cells arrive as `Formula { region: Some(_), .. }`
    /// carrying only a cache. The region installs their formula (single mint at
    /// the anchor), so they are never individually bound — only their cache is
    /// published for pre-F9 render.
    region_cell_caches: Vec<(u32, u32, CalcValue)>,
    /// Cached values for cells that do **not** enter the calc graph
    /// (DataTable / Unknown formula records, D4 §12 row 22): they publish their
    /// `FileCached` value and are ledgered `NotCalcModeled`, never bound.
    unmodeled_cached: Vec<(u32, u32, CalcValue)>,
}

/// One formula cell staged for bind-or-degrade at commit (D4 §10).
#[derive(Debug, Clone)]
pub struct IngestFormula {
    pub row: u32,
    pub col: u32,
    /// The formula text with its leading `=` restored (SpreadsheetML/R1C1 both
    /// arrive `=`-less); this is what OxFml binds and what round-trips.
    pub source_text: String,
    pub channel: FormulaChannelKind,
    /// The `FileCached` value the file carried for this cell (published pre-recalc
    /// and retained on a bind degradation), or `None`.
    pub cached: Option<CalcValue>,
}

/// One repeated-formula region staged at commit (D4 §12 rows 22/24): an R1C1
/// template tiled over a one-based rect.
#[derive(Debug, Clone)]
pub struct IngestRepeatedRegion {
    pub top_row: u32,
    pub left_col: u32,
    pub bottom_row: u32,
    pub right_col: u32,
    /// The R1C1 template text with its leading `=` restored.
    pub source_text: String,
    pub channel: FormulaChannelKind,
}

/// A merged-region rectangle staged for install on a sheet's grid at commit (D4
/// §12 row 10), in one-based coordinates. Installed into `GridInputState`'s
/// `merged_regions` so the build registers it via the engine's live
/// `add_merged_region` — spill blocking and merged-follower edit admission are
/// live engine semantics, not inert retention.
#[derive(Debug, Clone, Copy)]
pub struct IngestMergedRegionInstall {
    pub top_row: u32,
    pub left_col: u32,
    pub bottom_row: u32,
    pub right_col: u32,
}

/// A structured-table overlay staged for install on a sheet's grid at commit (D4
/// §12 row 25). Carries the table identity + parsed range + per-column bands
/// (derived from the header row). Installed into `GridInputState`'s
/// `table_overlays` so `set_table_overlay` registers the structured-reference
/// resolution as live engine semantics.
#[derive(Debug, Clone)]
pub struct IngestTableOverlayInstall {
    /// The table name (also the structured-reference prefix: `Name[Col]`).
    pub name: String,
    /// The whole table range in one-based coordinates.
    pub top_row: u32,
    pub left_col: u32,
    pub bottom_row: u32,
    pub right_col: u32,
    /// One entry per column, in left-to-right order: `(column_name, col)`. The
    /// name is read from the header cell (top row of the column) if present, else
    /// synthesized (`Column{n}`) so a structured reference still binds. The data
    /// rect is rows `top_row+1..=bottom_row` at that column (the header row is
    /// structural, D4 §5).
    pub columns: Vec<IngestTableColumn>,
    /// Whether the table has a header row (top row is a header band). Always
    /// `true` here: oxdoc-model's `TableSpec` carries no header flag, and an Excel
    /// table's first row is its header by construction. Recorded explicitly so
    /// R6.4 can revisit if a headerless-table spec ever arrives upstream.
    pub has_header: bool,
}

/// One column band of an ingested table (D4 §12 row 25): the structured-reference
/// column name and its one-based column index.
#[derive(Debug, Clone)]
pub struct IngestTableColumn {
    pub name: String,
    pub col: u32,
}

/// A defined name staged for install on a sheet's grid at commit (D4 §12 row 26).
///
/// Resolved from a `DefinedNameSpec` against the completed sheet map: the target
/// grid is the one that OWNS the name's target (the engine's `set_defined_name`
/// requires a static name's rect to sit on the authoring sheet — `check_rect`),
/// which is also the sheet whose formulas resolve the name natively. A
/// rect-denoting `formula_text` installs as a **static** name (a fixed rect); any
/// other text installs as a **dynamic** name whose defining formula binds through
/// the single key mint (§3).
#[derive(Debug, Clone)]
pub struct IngestDefinedNameInstall {
    /// The name text.
    pub name: String,
    /// Workbook scope (`None`) shadows nothing; sheet scope (`Some(())`) confines
    /// the name to the target grid's own sheet and shadows the workbook name of
    /// the same text there (V8 precedence). The scope's sheet id is the target
    /// grid's own `sheet_id`, applied by the builder.
    pub sheet_scoped: bool,
    /// The install target.
    pub target: IngestDefinedNameTarget,
}

/// The target of an ingested defined name (D4 §12 row 26).
#[derive(Debug, Clone)]
pub enum IngestDefinedNameTarget {
    /// A rect-denoting name: a fixed one-based rect on the target sheet.
    Static {
        top_row: u32,
        left_col: u32,
        bottom_row: u32,
        right_col: u32,
    },
    /// A dynamic name: a defining formula bound through the single mint (§3) at
    /// the target sheet's dynamic-name anchor. `source_text` carries the leading
    /// `=` restored and is the author's DISPLAY-qualified text verbatim — it is
    /// bound as-is (existence-blind, keeping display sheet names), NOT rewritten to
    /// an engine `sheet_id` token, so [`GridFormulaCell::source_text`] retains the
    /// authored form the save projection (W062 R6.66) re-emits.
    Dynamic { source_text: String },
}

/// One ingested defined name's metadata (D4 §12 row 26, Tier-B half): the
/// comment/hidden/function flags + raw attrs, keyed by `(name, scope)`. Retained
/// in the [`IngestedDocumentFacts`] store (§13); the load report echoes it. Only
/// names carrying non-empty metadata produce an entry.
///
/// The scope half is essential: a workbook-scoped and a sheet-scoped name of the
/// SAME text (a legal Excel shadow pair) each keep their own metadata, so the
/// save projection re-attaches by `(name, scope_sheet_id)`, not name text alone
/// (W062 R6.68 / calc-5kqg.68). `scope_sheet_id` is `None` for a workbook-scoped
/// name and the upstream sheet id for a sheet-scoped one — the exact
/// `DefinedNameSpec::scope_sheet_id` the name was ingested with.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct IngestedDefinedNameMetadata {
    /// The name the metadata is keyed by (with `scope_sheet_id`).
    pub name: String,
    /// The name's scope: `None` = workbook-scoped, `Some(upstream_sheet_id)` =
    /// sheet-scoped. Disambiguates a same-text shadow pair.
    pub scope_sheet_id: Option<u32>,
    /// The upstream metadata spec, retained verbatim for the round-trip.
    pub metadata: DefinedNameMetadataSpec,
}

/// The workbook-load sink (D4 §§8, 9, 12).
///
/// Accumulates the Tier-A prelude subset (settings, sheets, literal cells) and
/// a per-variant fidelity ledger during the `oxdoc-model` drive, then installs
/// everything in **one** transaction at [`Self::commit_into`]. The sink never
/// calls the public verbs per event (that would mint a revision per cell and
/// re-validate per edit, D4 §9); it gathers plain intermediate state and the
/// commit folds it into the calc model in a single revision mint.
pub struct OxCalcWorkbookIngestSink {
    /// Workbook settings from the header (D4 §12 row 1). `None` until
    /// `workbook()` is called (always first, per the stream validator).
    settings: Option<WorkbookCalcSettings>,
    /// Resolved shared-string table (D4 §11): owned text, resolve-at-ingest.
    string_table: Vec<String>,
    /// Per-sheet accumulators, in `SheetBegin` order (= sheet order, C3).
    sheets: Vec<SheetAccumulator>,
    /// The currently-open sheet's index into `sheets` (set by `sheet_begin`,
    /// cleared by `sheet_end`).
    open_sheet: Option<usize>,
    /// The fidelity ledger, keyed by variant so repeated instances accumulate
    /// into one row's `observed` count. Rendered to a Vec (table order) at
    /// commit.
    ledger: BTreeMap<DocumentVariantTag, IngestLedgerRow>,
    /// Every variant tag the stream actually carried, in first-seen order — the
    /// no-silent-loss invariant's `observed` set.
    observed: Vec<DocumentVariantTag>,
    /// Count of `RichStub` cells observed across all chunks (D4 §12 row 23):
    /// authored round-trip deferred to a later cell-input bead, surfaced in the
    /// report as deferred, never consumed as a fake value.
    rich_stubs_deferred: u32,
    /// Per-address routing overrides declared by `FormulaTopology` (D4 §12 row
    /// 22), keyed by `(upstream_sheet_id, row_one_based, col_one_based)`. The
    /// stream validator requires `FormulaTopology` to precede the sheet's cell
    /// chunk, so these are populated before the matching `cell_chunk` formula
    /// arrives; `cell_chunk` consults the map to route a DataTable/Unknown cell
    /// away from binding (publish cached + ledger `NotCalcModeled`).
    topology_overrides: BTreeMap<(u32, u32, u32), TopologyRoute>,
    /// Merged-region rects accumulated during the drive (D4 §12 row 10), keyed by
    /// upstream sheet id. `MergedCellRegions` arrives inside its sheet (the
    /// validator's `ensure_sheet_id`), but the install is still **deferred** to
    /// commit — a merge is a `GridInputState` fact registered when the sheet's
    /// grid is built, so all sheet/merge state lands in the single load
    /// transaction rather than per event. One entry per `(sheet_id, rect)` in
    /// stream order.
    merged_regions: Vec<(u32, IngestMergedRect)>,
    /// Structured-table overlays accumulated during the drive (D4 §12 row 25).
    /// `TableOverlay` carries its own `sheet_id`, resolved to a grid at commit;
    /// the range string is parsed there. Stream order preserved.
    table_overlays: Vec<IngestTableSpec>,
    /// Defined names accumulated during the drive (D4 §12 row 26). **Position-free
    /// in the stream** (`validate_event_stream` order-constrains neither this nor
    /// its target sheet's `SheetBegin`), so the install is deferred to commit —
    /// after every sheet exists — making forward references (`Sheet2!`-targeting
    /// names arriving before `Sheet2`'s `SheetBegin`) ordering-proof (D4 §9). We
    /// accumulate the owned spec here and resolve scope/target at commit against
    /// the completed sheet map, never relying on validator ordering.
    defined_names: Vec<DefinedNameSpec>,
    /// The inert Tier-B retention store accumulated during the drive (D4 §13).
    /// **The no-silent-loss home for R6.4:** every Tier-B variant retained here
    /// verbatim so R6.6 can replay it. Sealed into an `Arc` at commit and written
    /// as the workspace's `ingested_document_facts`, with its digest driving the
    /// `#workbook-ingest` meta-child's identity. Rect-less families accumulate
    /// straight into the typed fields; rect-claiming families additionally push
    /// an [`IngestedInertOverlay`] onto [`inert_overlays`].
    ///
    /// [`inert_overlays`]: IngestedDocumentFacts::inert_overlays
    document_facts: IngestedDocumentFacts,
}

/// One merged-region rectangle staged for install at commit (D4 §12 row 10), in
/// one-based `(top_row, left_col, bottom_row, right_col)` coordinates. The
/// oxdoc-model `CellRangeSpec` carries pre-parsed `start`/`end` addresses, so no
/// A1 parsing happens here — the coordinates come straight off the wire.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct IngestMergedRect {
    top_row: u32,
    left_col: u32,
    bottom_row: u32,
    right_col: u32,
}

/// One structured-table overlay staged for install at commit (D4 §12 row 25):
/// the upstream sheet id it belongs to, the table name (structured-reference
/// prefix), and the A1 range text (parsed at commit into a rect + column bands).
#[derive(Debug, Clone, PartialEq, Eq)]
struct IngestTableSpec {
    sheet_id: u32,
    name: String,
    range: String,
}

/// How a `FormulaTopology` record overrides the default (bind) handling for one
/// cell address (D4 §12 row 22). `Array` cells still bind (the anchor is a
/// normal formula; the inert `Cse` overlay claims the rect separately), so only
/// the non-calc-modeled kinds carry an override.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TopologyRoute {
    /// DataTable / Unknown: do not bind. Publish the cell's `FileCached` value
    /// and ledger `NotCalcModeled`.
    NotCalcModeled,
}

impl Default for OxCalcWorkbookIngestSink {
    fn default() -> Self {
        Self::new()
    }
}

impl OxCalcWorkbookIngestSink {
    #[must_use]
    pub fn new() -> Self {
        Self {
            settings: None,
            string_table: Vec::new(),
            sheets: Vec::new(),
            open_sheet: None,
            ledger: BTreeMap::new(),
            observed: Vec::new(),
            rich_stubs_deferred: 0,
            topology_overrides: BTreeMap::new(),
            merged_regions: Vec::new(),
            table_overlays: Vec::new(),
            defined_names: Vec::new(),
            document_facts: IngestedDocumentFacts::default(),
        }
    }

    /// The variant tags the stream carried, in first-seen order. Exposed for the
    /// no-silent-loss invariant check after a drive.
    #[must_use]
    pub fn observed(&self) -> &[DocumentVariantTag] {
        &self.observed
    }

    /// Record that `variant` was observed, folding it into its ledger row and
    /// the observed set. The single choke point for the no-silent-loss regime:
    /// **every** event the sink touches goes through here, so a variant with no
    /// `ledger`/`observe` call is a silent path the invariant test catches.
    fn ledger_and_observe(
        &mut self,
        variant: DocumentVariantTag,
        tier: IngestTier,
        disposition: &'static str,
    ) {
        if !self.observed.contains(&variant) {
            self.observed.push(variant);
        }
        self.ledger
            .entry(variant)
            .and_modify(|row| row.observed += 1)
            .or_insert(IngestLedgerRow {
                variant,
                tier,
                disposition,
                observed: 1,
            });
    }

    /// Install the accumulated Tier-A state into `context`'s workbook workspace
    /// in **one** transaction (D4 §9), returning the load report. The workspace
    /// must already exist with a `Workbook`-role root (the public one-call verb
    /// that creates it is R6.5); this bead's callers create it and drive the
    /// sink, then commit.
    pub fn commit_into(
        self,
        context: &mut OxCalcDocumentContext,
        workspace_id: &OxCalcTreeWorkspaceId,
    ) -> Result<WorkbookLoadReport, OxCalcWorkbookIngestError> {
        let settings = self.settings.unwrap_or_default();
        let sheet_count = self.sheets.len() as u32;
        let cell_count: u32 = self
            .sheets
            .iter()
            .map(|sheet| sheet.literals.len() as u32)
            .sum();
        let not_calc_modeled: u32 = self
            .sheets
            .iter()
            .map(|sheet| sheet.unmodeled_cached.len() as u32)
            .sum();
        let rich_stubs_deferred = self.rich_stubs_deferred;
        // Move the accumulated Tier-B store out of the sink so the deferred-name
        // resolution below can fold name metadata into it and the commit can seal
        // it into an `Arc`. The inert-overlay spatial index rides along inside it.
        let mut document_facts = self.document_facts;

        // DEFERRED INSTALL (D4 §9): now that every sheet has been observed,
        // resolve the position-free name/table/merge accumulations against the
        // completed sheet map and distribute them per sheet. Forward references
        // (a name whose target sheet appears later in the stream) resolve here
        // because this runs after the WHOLE drive — never relying on validator
        // ordering. `resolve_deferred_installs` returns one bucket per sheet
        // (index-aligned with `self.sheets`), the name metadata write-through
        // stub, and the installed-name/table counts for the report.
        let sheet_index_by_upstream: BTreeMap<u32, usize> = self
            .sheets
            .iter()
            .enumerate()
            .map(|(index, sheet)| (sheet.upstream_sheet_id, index))
            .collect();
        // Display-name → sheet index (last-write-wins is irrelevant: the D1 build
        // path rejects case-fold-duplicate sheet names, so names are unique).
        let sheet_index_by_display: BTreeMap<String, usize> = self
            .sheets
            .iter()
            .enumerate()
            .map(|(index, sheet)| (sheet.display_name.clone(), index))
            .collect();
        // Upstream id → display name, so a name's `scope_sheet_id` (upstream) can
        // be compared against its static rect's target sheet (a display name).
        let sheet_display_by_upstream: BTreeMap<u32, String> = self
            .sheets
            .iter()
            .map(|sheet| (sheet.upstream_sheet_id, sheet.display_name.clone()))
            .collect();
        let DeferredInstalls {
            merges_by_sheet,
            tables_by_sheet,
            names_by_sheet,
            name_metadata,
            dropped_installs,
        } = resolve_deferred_installs(
            self.sheets.len(),
            &sheet_index_by_upstream,
            &sheet_index_by_display,
            &sheet_display_by_upstream,
            &self.merged_regions,
            &self.table_overlays,
            &self.defined_names,
        );
        // Fold the resolved defined-name metadata into the Tier-B store (D4 §12
        // row 26 A/B split): the round-trip home is the store, not the report.
        // Retained for EVERY name carrying metadata, install-independent.
        document_facts.name_metadata = name_metadata;

        // Record the upstream sheet ids in stream/creation order (W062 R6.6, D4
        // §7a): the save projection's index for re-emitting `SheetBegin` with the
        // same upstream id every sheet-scoped Tier-B fact stores inside it. Sheets
        // are created in this order (C3), so registry position `i` at projection
        // time reads `sheet_stream_ids[i]`. `#[serde(skip)]`, so it never enters
        // the store's identity digest.
        document_facts.sheet_stream_ids = self
            .sheets
            .iter()
            .map(|sheet| sheet.upstream_sheet_id)
            .collect();

        // Seal the store: immutable after load (R6). Its digest drives the
        // `#workbook-ingest` meta-child identity, and the same `Arc` is written
        // as the workspace's live `ingested_document_facts` (retained by pointer
        // onto revisions in the builder). The report's `inert_overlays` /
        // `name_metadata` are read back off the sealed store so the report and
        // the retention home never diverge.
        let document_facts = std::sync::Arc::new(document_facts);
        let inert_overlays = document_facts.inert_overlays.clone();
        let report_name_metadata = document_facts.name_metadata.clone();

        // The single-transaction builder on the context (consumer.rs) mints ONE
        // revision for the whole load. It is the only place that touches
        // consumer-private state; the sink hands it a plain plan (Tier A + the
        // sealed Tier-B store) and gets back the bind outcome for the report.
        let plan = WorkbookTierALoadPlan {
            settings,
            sheets: self
                .sheets
                .into_iter()
                .zip(merges_by_sheet)
                .zip(tables_by_sheet)
                .zip(names_by_sheet)
                .map(|(((sheet, merges), tables), names)| SheetTierALoad {
                    display_name: sheet.display_name,
                    upstream_sheet_id: sheet.upstream_sheet_id,
                    literals: sheet.literals,
                    formulas: sheet.formulas,
                    repeated_regions: sheet.repeated_regions,
                    region_cell_caches: sheet.region_cell_caches,
                    unmodeled_cached: sheet.unmodeled_cached,
                    merged_regions: merges,
                    table_overlays: tables,
                    defined_names: names,
                })
                .collect(),
            document_facts: std::sync::Arc::clone(&document_facts),
        };
        let outcome = context.commit_workbook_tier_a_load(workspace_id, plan)?;

        // Fold the resolution-time install drops (unresolvable name target /
        // unmodelable scope / unparseable table range) into the same
        // `bind_degradations` list the builder's commit-time degradations (bad
        // formula cells, rejected dynamic names) use — one honest channel for
        // every retained-but-not-installed authored fact (C13). The resolution-time
        // drops come first (they precede any commit-time bind).
        let mut bind_degradations = dropped_installs;
        bind_degradations.extend(outcome.bind_degradations);

        let ledger: Vec<IngestLedgerRow> = DocumentVariantTag::ALL
            .iter()
            .filter_map(|tag| self.ledger.get(tag).cloned())
            .collect();

        Ok(WorkbookLoadReport {
            sheets: sheet_count,
            cells: cell_count,
            formulas_bound: outcome.formulas_bound,
            rich_stubs_deferred,
            not_calc_modeled,
            region_cells_unbacked: outcome.region_cells_unbacked,
            names: outcome.names_installed,
            tables: outcome.tables_installed,
            name_metadata: report_name_metadata,
            ledger,
            bind_degradations,
            inert_overlays,
            external_ref_cells_pinned: outcome.external_reference_pins.len() as u32,
            external_reference_pins: outcome.external_reference_pins,
            // The load-recalc policy (D4 §6) the builder ran, resolving the R6.2
            // carry-forward: `CalcMode::Automatic` issued the open-recalc (values
            // are engine `Calculated`, differential clean); `CalcMode::Manual` ran
            // NO engine evaluation (values are `FileCached` until F9). The perf
            // counter that proves the Manual-zero-eval claim is
            // `engine_recalcs_at_load` — `0` under Manual.
            engine_recalcs_at_load: outcome.engine_recalcs_at_load,
            recalc_path: outcome.recalc_path,
        })
    }

    /// Resolve a cell value to a typed [`CalcValue`] (D4 §§10, 11). `SharedText`
    /// indices resolve against the prelude string table; error codes map through
    /// [`map_biff_error_code`]; formulas and rich stubs are deferred (R6.2/R6.4)
    /// and reported to the caller so the cell can be ledgered rather than
    /// consumed.
    fn resolve_literal(&self, value: OxCalcCellValue<'_>) -> CalcValue {
        match value {
            OxCalcCellValue::Number(n) => CalcValue::number(n),
            OxCalcCellValue::Bool(b) => CalcValue::logical(b),
            OxCalcCellValue::Error(code) => {
                let (mapped, _known) = map_biff_error_code(code);
                CalcValue::error(mapped)
            }
            OxCalcCellValue::SharedText(index) => {
                let text = self
                    .string_table
                    .get(index as usize)
                    .cloned()
                    .unwrap_or_default();
                CalcValue::text(ExcelText::from_interop_assignment(&text))
            }
            OxCalcCellValue::InlineText(text) => {
                CalcValue::text(ExcelText::from_interop_assignment(text))
            }
        }
    }

    /// Resolve a `FileCached` cached value to a typed [`CalcValue`] (D4 §6/§10).
    /// The provenance discriminant is `FileCached` by construction upstream; the
    /// value resolves through the same [`Self::resolve_literal`] path so shared
    /// strings and error codes map identically to a literal of the same shape.
    fn resolve_cached(&self, cached: OxCalcCachedValue<'_>) -> CalcValue {
        self.resolve_literal(cached.value)
    }

    /// Stage a `cell_chunk` formula for bind-or-degrade at commit (D4 §10). The
    /// leading `=` is restored (both `SpreadsheetMlA1` and `R1C1RelativeTemplate`
    /// arrive `=`-less, matching how they store in the file), the bind channel is
    /// picked from the text kind, and the `FileCached` cache (if any) is resolved
    /// to a typed value now so it round-trips regardless of the bind outcome. A
    /// formula with no text is treated as empty text (`=`), which OxFml rejects —
    /// the honest degradation path rather than a fabricated cell.
    fn stage_formula(&self, row: u32, col: u32, input: OxCalcFormulaInput<'_>) -> IngestFormula {
        let source_text = restore_leading_eq(input.text.unwrap_or(""));
        let channel = match input.text_kind {
            FormulaTextKind::SpreadsheetMlA1 => FormulaChannelKind::WorksheetA1,
            FormulaTextKind::R1C1RelativeTemplate => FormulaChannelKind::WorksheetR1C1,
        };
        let cached = input.cached.map(|cached| self.resolve_cached(cached));
        IngestFormula {
            row,
            col,
            source_text,
            channel,
            cached,
        }
    }

    /// Install a `SharedFormulaRegion` as a repeated-formula region on the open
    /// sheet (D4 §12 row 24). The R1C1 template tiles over `anchor..anchor+extent`
    /// via the existing FillRange machinery (`put_repeated_formula_region`),
    /// differential-proven per-cell at commit. The template's leading `=` is
    /// restored and the R1C1 bind channel is used.
    fn install_shared_formula_region(
        &mut self,
        region: &SharedFormulaRegion,
    ) -> Result<(), OxCalcWorkbookIngestError> {
        let sheet_index = self.open_sheet.ok_or_else(|| {
            OxCalcWorkbookIngestError::Rejected(
                "shared formula region arrived with no open sheet".to_string(),
            )
        })?;
        let top_row = region.anchor.row_one_based();
        let left_col = region.anchor.col_one_based();
        // `extent` is a 1-based span count; a 1x1 extent covers just the anchor.
        let bottom_row = top_row + region.extent.rows.saturating_sub(1);
        let right_col = left_col + region.extent.cols.saturating_sub(1);
        self.sheets[sheet_index]
            .repeated_regions
            .push(IngestRepeatedRegion {
                top_row,
                left_col,
                bottom_row,
                right_col,
                source_text: restore_leading_eq(&region.r1c1_text),
                channel: FormulaChannelKind::WorksheetR1C1,
            });
        Ok(())
    }

    /// Accumulate a `MergedCellRegions` event's rects for install at commit (D4
    /// §12 row 10). The validator (`ensure_sheet_id`) guarantees an open sheet
    /// matching `regions.sheet_id`, but the install is still deferred to commit:
    /// a merge is a `GridInputState` fact the sheet's grid registers at build,
    /// so it rides the single load transaction. `CellRangeSpec` carries
    /// pre-parsed `start`/`end` addresses (no A1 parsing here); `raw_refs`
    /// (unparsed-fallback ref strings) are R6.4's Tier-B store — a rect with no
    /// parsed range is not conjured from a raw ref here.
    fn accumulate_merged_regions(&mut self, regions: &MergedCellRegions) {
        for range in &regions.ranges {
            self.merged_regions.push((
                regions.sheet_id,
                IngestMergedRect {
                    top_row: range.start.row_one_based().min(range.end.row_one_based()),
                    left_col: range.start.col_one_based().min(range.end.col_one_based()),
                    bottom_row: range.start.row_one_based().max(range.end.row_one_based()),
                    right_col: range.start.col_one_based().max(range.end.col_one_based()),
                },
            ));
        }
    }

    /// Route a `FormulaTopology`'s records (D4 §12 row 22). The stream validator
    /// guarantees this precedes the sheet's cell chunk, so the per-address
    /// overrides it records are consulted by the later `cell_chunk`:
    ///
    /// - `Normal` — the cell binds as an ordinary formula from its cell input;
    ///   no override needed.
    /// - `Shared` — the shared expansion is owned by the `SharedFormulaRegion`
    ///   event; the topology record only marks membership, so no override here.
    /// - `Array` (legacy CSE) — the cell(s) ingest as normal formulas; the array
    ///   rect additionally claims an inert `Cse` overlay (no legacy-CSE eval
    ///   semantics are built). The overlay is recorded as a load fact.
    /// - `DataTable` / `Unknown` — the cell is **not** calc-modeled: an override
    ///   routes its cell input to publish-cached + ledger `NotCalcModeled`.
    fn route_formula_topology(
        &mut self,
        topology: &oxdoc_model::FormulaTopology,
    ) -> Result<(), OxCalcWorkbookIngestError> {
        // The open-sheet guard is a stream-shape invariant (the validator places
        // a topology inside its sheet); records route by their own `sheet_id`.
        if self.open_sheet.is_none() {
            return Err(OxCalcWorkbookIngestError::Rejected(
                "formula topology arrived with no open sheet".to_string(),
            ));
        }
        for record in &topology.records {
            self.route_formula_record(topology.sheet_id, record);
        }
        // The whole `FormulaTopology` is retained verbatim in the Tier-B store
        // (row 22): the routed calc dispositions above are Tier A (they live in
        // the calc model), but the file-topology METADATA — each record's
        // `attrs`, the topology's + records' `unsupported_fragments` — is Tier B
        // round-trip fidelity that must survive to save (R6.6). Retaining the
        // whole value (not a lossy summary) is the no-silent-loss discipline.
        self.document_facts
            .formula_topologies
            .push(topology.clone());
        Ok(())
    }

    /// Push a rect-claiming family's inert overlay onto the store's spatial index
    /// (D4 §13), minting a stable store key `{family}:{sheet_id}#{ordinal}` where
    /// `ordinal` is the per-family-per-sheet count so far. The retention home is
    /// the store's typed field; this is only the index into it.
    fn push_inert_overlay(
        &mut self,
        kind: InertOverlayKind,
        sheet_id: u32,
        rect: (u32, u32, u32, u32),
    ) {
        let ordinal = self
            .document_facts
            .inert_overlays
            .iter()
            .filter(|overlay| overlay.kind == kind && overlay.sheet_id == sheet_id)
            .count();
        let payload = format!("{}:{sheet_id}#{ordinal}", kind.family_prefix());
        self.document_facts
            .inert_overlays
            .push(IngestedInertOverlay {
                kind,
                sheet_id,
                rect,
                payload,
            });
    }

    /// The upstream sheet id of the currently-open sheet, if any. The sheet-
    /// scoped axis-run / cell-format-run features carry only their runs (no
    /// sheet id on the wire), so they key off the open sheet the validator
    /// guarantees. `None` only for a malformed stream (no open sheet); the
    /// caller ledgers the variant regardless, so the fact is never silent.
    fn open_sheet_upstream_id(&self) -> Option<u32> {
        self.open_sheet
            .map(|index| self.sheets[index].upstream_sheet_id)
    }

    /// Retain a `DrawingFormControlsSpec` verbatim (D4 §12 row 19) and claim an
    /// inert `RichObject` overlay rect for each drawing OBJECT carrying a
    /// resolvable cell anchor. The form controls themselves carry no cell rect
    /// (their geometry lives on the host drawing object's anchor), so the rects
    /// come from `spec.objects[].anchor`. A `TwoCell` anchor spans `from..to`; a
    /// `OneCell`/`Absolute` anchor with only a `from` marker claims that single
    /// cell. An anchor with no positional marker (a pure EMU-absolute placement)
    /// yields no rect — the spec is still fully retained, so nothing is lost;
    /// only the spatial index is skipped for an off-grid object.
    fn retain_drawing_form_controls(&mut self, spec: &oxdoc_model::DrawingFormControlsSpec) {
        let sheet_id = spec.sheet_id;
        for object in &spec.objects {
            let Some(rect) = drawing_anchor_rect(&object.anchor) else {
                continue;
            };
            self.push_inert_overlay(InertOverlayKind::RichObject, sheet_id, rect);
        }
        self.document_facts.drawing_form_controls.push(spec.clone());
    }

    /// Retain a `ConditionalFormatRegion` verbatim (D4 §12 row 21) and claim an
    /// inert `ConditionalFormat` overlay rect for each of its `ranges`. The
    /// region's `sqref` is the authored range text; the parsed `ranges` carry
    /// pre-parsed `start`/`end` addresses (no A1 parsing here). A region with no
    /// parsed ranges (a malformed/empty sqref) is still fully retained in the
    /// store — only the spatial index is skipped, never the spec.
    fn retain_conditional_format(&mut self, region: &oxdoc_model::ConditionalFormatRegion) {
        let sheet_id = region.sheet_id;
        for range in &region.ranges {
            let rect = (
                range.start.row_one_based().min(range.end.row_one_based()),
                range.start.col_one_based().min(range.end.col_one_based()),
                range.start.row_one_based().max(range.end.row_one_based()),
                range.start.col_one_based().max(range.end.col_one_based()),
            );
            self.push_inert_overlay(InertOverlayKind::ConditionalFormat, sheet_id, rect);
        }
        self.document_facts.conditional_formats.push(region.clone());
    }

    /// Route one `FormulaRecord` by kind (D4 §12 row 22). See
    /// [`Self::route_formula_topology`] for the per-kind contract.
    fn route_formula_record(&mut self, sheet_id: u32, record: &FormulaRecord) {
        let row = record.address.row_one_based();
        let col = record.address.col_one_based();
        match &record.kind {
            FormulaRecordKind::Normal | FormulaRecordKind::Shared(_) => {
                // Normal binds from its cell input; Shared's expansion is the
                // SharedFormulaRegion event's job. No override, no extra install.
            }
            FormulaRecordKind::Array(spec) => {
                // Legacy CSE: the cell(s) bind as normal formulas from their cell
                // inputs. The array rect claims an inert `Cse` overlay — no
                // legacy-CSE eval semantics are built (the overlay is inert). The
                // overlay is a spatial index into the store, not the retention
                // home (D4 §13).
                let rect = spec
                    .range
                    .as_ref()
                    .map(|range| {
                        (
                            range.start.row_one_based(),
                            range.start.col_one_based(),
                            range.end.row_one_based(),
                            range.end.col_one_based(),
                        )
                    })
                    .unwrap_or((row, col, row, col));
                self.push_inert_overlay(InertOverlayKind::Cse, sheet_id, rect);
            }
            FormulaRecordKind::DataTable(_) | FormulaRecordKind::Unknown { .. } => {
                // Not calc-modeled (D4 §12 row 22): the cell input publishes its
                // cached value and ledgers `NotCalcModeled`. Record the override
                // the later cell_chunk consults; the real topology-fact retention
                // is R6.4's Tier-B store.
                self.topology_overrides
                    .insert((sheet_id, row, col), TopologyRoute::NotCalcModeled);
            }
        }
    }
}

/// The one-based cell rect a drawing anchor claims (D4 §12 row 19), or `None`
/// when the anchor carries no positional cell marker (a pure EMU-absolute
/// placement — off the cell grid). OOXML spreadsheet-drawing `from`/`to` markers
/// are **zero-based** cell indices, so they are shifted to the engine's one-based
/// coordinates here. A two-cell anchor spans `from..=to` (the `to` cell the
/// object extends into is included — a faithful, inclusive spatial claim); a
/// one-cell/absolute anchor with only a `from` marker claims that single cell.
fn drawing_anchor_rect(anchor: &oxdoc_model::DrawingAnchor) -> Option<(u32, u32, u32, u32)> {
    let from = anchor.from?;
    let top_row = from.row + 1;
    let left_col = from.col + 1;
    let (bottom_row, right_col) = match anchor.to {
        Some(to) => ((to.row + 1).max(top_row), (to.col + 1).max(left_col)),
        None => (top_row, left_col),
    };
    Some((top_row, left_col, bottom_row, right_col))
}

/// Restore the leading `=` a formula stores without (D4 §10). SpreadsheetML and
/// R1C1 both carry `=`-less text; OxFml's cell-entry grammar expects the `=`.
/// Idempotent: text that already leads with `=` is returned unchanged.
fn restore_leading_eq(text: &str) -> String {
    if text.starts_with('=') {
        text.to_string()
    } else {
        format!("={text}")
    }
}

impl OxCalcIngestSink for OxCalcWorkbookIngestSink {
    type Error = OxCalcWorkbookIngestError;

    fn workbook(&mut self, prelude: OxCalcWorkbookPrelude<'_>) -> Result<(), Self::Error> {
        // D4 §12 rows 1-3: WorkbookHeader (A, consumed → settings), StringTable
        // (A, consumed → resolved at ingest, not stored), StyleTable (B, retained
        // verbatim in the Tier-B store).
        self.settings = Some(WorkbookCalcSettings {
            date_system: map_date_system(prelude.header.date_system),
            calc_mode: map_calc_mode(prelude.header.calc_mode),
            iteration: map_iteration_settings(prelude.header.iterative_calc),
        });
        self.ledger_and_observe(
            DocumentVariantTag::WorkbookHeader,
            IngestTier::A,
            "consumed",
        );

        self.string_table = prelude
            .string_table
            .iter()
            .map(|entry| entry.text.clone())
            .collect();
        self.ledger_and_observe(DocumentVariantTag::StringTable, IngestTier::A, "consumed");

        // StyleTable (row 3): retained verbatim in the Tier-B store — a rect-less
        // family (no overlay), proving the store is the retention home. Number
        // formats included (display-only; D4 T5). R6.6 replays it at save.
        self.document_facts.style_table = Some(prelude.style_table.clone());
        self.ledger_and_observe(
            DocumentVariantTag::StyleTable,
            IngestTier::B,
            "retained-inert",
        );
        Ok(())
    }

    fn sheet_begin(&mut self, sheet: &SheetRef) -> Result<(), Self::Error> {
        // D4 §12 row 5: SheetBegin (A) — sheet node creation, stream order =
        // sheet order (C3).
        self.open_sheet = Some(self.sheets.len());
        self.sheets.push(SheetAccumulator {
            upstream_sheet_id: sheet.sheet_id,
            display_name: sheet.name.clone(),
            ..SheetAccumulator::default()
        });
        self.ledger_and_observe(DocumentVariantTag::SheetBegin, IngestTier::A, "consumed");
        Ok(())
    }

    fn cell_chunk(&mut self, chunk: OxCalcCellChunk<'_>) -> Result<(), Self::Error> {
        // D4 §12 row 23: CellChunk (A). Literals → typed CalcValue (unknown error
        // bytes additionally retained in the Tier-B store); Formula → staged for
        // bind-or-degrade at commit (D4 §10), carrying its FileCached cache;
        // Empty → no record; RichStub → authored round-trip deferred to a later
        // cell-input bead, counted honestly, never consumed as a fake value.
        let sheet_index = self.open_sheet.ok_or_else(|| {
            OxCalcWorkbookIngestError::Rejected("cell chunk arrived with no open sheet".to_string())
        })?;
        for cell in &chunk.cells {
            let row = cell.address.row_one_based();
            let col = cell.address.col_one_based();
            match cell.input {
                OxCalcCellInput::Empty => {
                    // No record (D4 §12 row 23). Empty carries no fidelity.
                }
                OxCalcCellInput::Literal(value) => {
                    // Unknown BIFF error byte (R6.1, D4 §10): the cell publishes
                    // #VALUE! (via `resolve_literal`), but its RAW byte is retained
                    // in the Tier-B store so save writes it back verbatim — never
                    // laundering an unknown code into #VALUE!. A no-silent-loss
                    // retention: the published value is honest AND the byte
                    // survives the round-trip.
                    if let OxCalcCellValue::Error(code) = value {
                        let (_mapped, known) = map_biff_error_code(code);
                        if !known {
                            let sheet_id = self.sheets[sheet_index].upstream_sheet_id;
                            self.document_facts.unknown_error_bytes.push(
                                UnknownErrorByteRetention {
                                    sheet_id,
                                    row,
                                    col,
                                    raw_byte: code,
                                },
                            );
                        }
                    }
                    let calc = self.resolve_literal(value);
                    self.sheets[sheet_index].literals.push((row, col, calc));
                }
                OxCalcCellInput::Formula(formula) => {
                    // REGION-MANAGED cells first (D4 §12 row 24). A shared-region
                    // member/anchor arrives as `Formula { region: Some(_), .. }`
                    // (a drag-filled column: the anchor carries the template text,
                    // the members carry `text: None`). The `SharedFormulaRegion`
                    // feature installs the whole region via one mint at the anchor,
                    // so these cells must NOT be individually bound or degraded —
                    // that would fabricate a false `BindDegradation{text:"="}` for
                    // a `text: None` member and double-install the anchor. Publish
                    // only their FileCached cache for pre-F9 render.
                    if formula.region.is_some() {
                        if let Some(cached) = formula.cached.map(|c| self.resolve_cached(c)) {
                            self.sheets[sheet_index]
                                .region_cell_caches
                                .push((row, col, cached));
                        }
                        continue;
                    }

                    // D4 §10: restore the leading `=` (both SpreadsheetML and R1C1
                    // arrive `=`-less), pick the bind channel from the text kind,
                    // and stage it for the commit builder to bind-or-degrade. The
                    // FileCached cache (if any) resolves to a typed CalcValue now
                    // so the cache round-trips regardless of the bind outcome.
                    let staged = self.stage_formula(row, col, formula);
                    let upstream_sheet_id = self.sheets[sheet_index].upstream_sheet_id;
                    match self.topology_overrides.get(&(upstream_sheet_id, row, col)) {
                        // DataTable / Unknown (D4 §12 row 22): the cell is not
                        // calc-modeled. Publish its FileCached value (if any) and
                        // ledger `NotCalcModeled` — never bind.
                        Some(TopologyRoute::NotCalcModeled) => {
                            if let Some(cached) = staged.cached {
                                self.sheets[sheet_index]
                                    .unmodeled_cached
                                    .push((row, col, cached));
                            }
                        }
                        // Normal / Shared / Array anchors bind as usual (the shared
                        // region / inert Cse overlay claim was recorded separately).
                        None => self.sheets[sheet_index].formulas.push(staged),
                    }
                }
                OxCalcCellInput::RichStub(_) => {
                    // Row 23: the authored `GridCellInput::RichStub` round-trip is
                    // a later cell-input bead (distinct from the row-19
                    // DrawingFormControls→RichObject overlay this bead builds).
                    // Counted so the report ledgers it as deferred, never consumed.
                    self.rich_stubs_deferred += 1;
                }
            }
        }
        self.ledger_and_observe(DocumentVariantTag::CellChunk, IngestTier::A, "consumed");
        Ok(())
    }

    fn sheet_end(&mut self, _sheet_id: u32) -> Result<(), Self::Error> {
        // D4 §12 row 6: SheetEnd (A) — closes per-sheet accumulation; a
        // structural no-op beyond ordering.
        self.open_sheet = None;
        self.ledger_and_observe(DocumentVariantTag::SheetEnd, IngestTier::A, "consumed");
        Ok(())
    }

    fn feature(&mut self, feature: OxCalcDocumentFeature<'_>) -> Result<(), Self::Error> {
        // D4 §12: the wildcard-free exhaustive match. A 30th upstream feature
        // variant is a COMPILE ERROR here (C13 tripwire), never a silent drop.
        // Every Tier-B arm RETAINS its owned spec verbatim in `document_facts`
        // (the no-silent-loss home, R6.4) and ledgers `retained-inert`; the
        // rect-claiming families (CF, cell-anchored drawing/form controls, and —
        // in `route_formula_topology` — legacy-CSE arrays) additionally push an
        // inert overlay spatial index. Tier X CalcChainHint is ledgered-and-
        // dropped. Do NOT add a `_ =>` arm.
        match feature {
            OxCalcDocumentFeature::SheetDimension(spec) => {
                // Row 7: retained verbatim (grid bounds set by profile policy;
                // this claim used only if the authored extent shrank — R6.6).
                self.document_facts.sheet_dimensions.push(spec.clone());
                self.ledger_and_observe(
                    DocumentVariantTag::SheetDimension,
                    IngestTier::B,
                    "retained-inert",
                );
            }
            OxCalcDocumentFeature::ColumnProps(runs) => {
                // Row 8: hidden/width/outline runs retained inert (named SUBTOTAL
                // hidden-column gap). Keyed by the open sheet's upstream id.
                if let Some(sheet_id) = self.open_sheet_upstream_id() {
                    self.document_facts.column_props.push(SheetAxisRuns {
                        sheet_id,
                        runs: runs.to_vec(),
                    });
                }
                self.ledger_and_observe(
                    DocumentVariantTag::ColumnProps,
                    IngestTier::B,
                    "retained-inert",
                );
            }
            OxCalcDocumentFeature::RowProps(runs) => {
                // Row 9: same as row 8, row axis.
                if let Some(sheet_id) = self.open_sheet_upstream_id() {
                    self.document_facts.row_props.push(SheetAxisRuns {
                        sheet_id,
                        runs: runs.to_vec(),
                    });
                }
                self.ledger_and_observe(
                    DocumentVariantTag::RowProps,
                    IngestTier::B,
                    "retained-inert",
                );
            }
            OxCalcDocumentFeature::MergedCellRegions(regions) => {
                self.accumulate_merged_regions(regions);
                self.ledger_and_observe(
                    DocumentVariantTag::MergedCellRegions,
                    IngestTier::A,
                    "installed-merged-regions",
                );
            }
            OxCalcDocumentFeature::SheetViewState(spec) => {
                // Row 11: frozen panes, selection, zoom — retained verbatim.
                self.document_facts.sheet_views.push(spec.clone());
                self.ledger_and_observe(
                    DocumentVariantTag::SheetViewState,
                    IngestTier::B,
                    "retained-inert",
                );
            }
            OxCalcDocumentFeature::Hyperlinks(spec) => {
                // Row 12.
                self.document_facts.hyperlinks.push(spec.clone());
                self.ledger_and_observe(
                    DocumentVariantTag::Hyperlinks,
                    IngestTier::B,
                    "retained-inert",
                );
            }
            OxCalcDocumentFeature::DataValidations(spec) => {
                // Row 13: validation formulas are NOT bound (UI-gate facts).
                self.document_facts.data_validations.push(spec.clone());
                self.ledger_and_observe(
                    DocumentVariantTag::DataValidations,
                    IngestTier::B,
                    "retained-inert",
                );
            }
            OxCalcDocumentFeature::AutoFilter(spec) => {
                // Row 14 (named filter/SUBTOTAL gap).
                self.document_facts.auto_filters.push(spec.clone());
                self.ledger_and_observe(
                    DocumentVariantTag::AutoFilter,
                    IngestTier::B,
                    "retained-inert",
                );
            }
            OxCalcDocumentFeature::SortState(spec) => {
                // Row 15.
                self.document_facts.sort_states.push(spec.clone());
                self.ledger_and_observe(
                    DocumentVariantTag::SortState,
                    IngestTier::B,
                    "retained-inert",
                );
            }
            OxCalcDocumentFeature::CommentNotice(spec) => {
                // Row 16.
                self.document_facts.comment_notices.push(spec.clone());
                self.ledger_and_observe(
                    DocumentVariantTag::CommentNotice,
                    IngestTier::B,
                    "retained-inert",
                );
            }
            OxCalcDocumentFeature::ThreadedCommentPeople(spec) => {
                // Row 17: a RECT-LESS family — retained in the store with NO
                // overlay, proving the store (not the overlay) is the home.
                self.document_facts
                    .threaded_comment_people
                    .push(spec.clone());
                self.ledger_and_observe(
                    DocumentVariantTag::ThreadedCommentPeople,
                    IngestTier::B,
                    "retained-inert",
                );
            }
            OxCalcDocumentFeature::SheetReviewComments(spec) => {
                // Row 18.
                self.document_facts.sheet_review_comments.push(spec.clone());
                self.ledger_and_observe(
                    DocumentVariantTag::SheetReviewComments,
                    IngestTier::B,
                    "retained-inert",
                );
            }
            OxCalcDocumentFeature::DrawingFormControls(spec) => {
                // Row 19 (overlay: RichObject). The whole spec is retained; each
                // drawing object carrying a resolvable cell anchor additionally
                // claims an inert `RichObject` overlay rect so spills/axis edits
                // can see it (inert `SpillBlock::None` today). The store is the
                // retention home; the overlay is only the spatial index.
                self.retain_drawing_form_controls(spec);
                self.ledger_and_observe(
                    DocumentVariantTag::DrawingFormControls,
                    IngestTier::B,
                    "retained-inert",
                );
            }
            OxCalcDocumentFeature::CellFormatRuns(runs) => {
                // Row 20: per-cell style presence, keyed by the open sheet.
                if let Some(sheet_id) = self.open_sheet_upstream_id() {
                    self.document_facts
                        .cell_format_runs
                        .push(SheetCellFormatRuns {
                            sheet_id,
                            runs: runs.to_vec(),
                        });
                }
                self.ledger_and_observe(
                    DocumentVariantTag::CellFormatRuns,
                    IngestTier::B,
                    "retained-inert",
                );
            }
            OxCalcDocumentFeature::DifferentialStyleTable(specs) => {
                // Row 4: dxf styles retained verbatim.
                self.document_facts
                    .differential_styles
                    .extend(specs.iter().cloned());
                self.ledger_and_observe(
                    DocumentVariantTag::DifferentialStyleTable,
                    IngestTier::B,
                    "retained-inert",
                );
            }
            OxCalcDocumentFeature::ConditionalFormatRegion(region) => {
                // Row 21 (overlay: ConditionalFormat). Full spec retained in the
                // store; each region claims an inert `ConditionalFormat` overlay
                // rect keyed to the store. CF rules are NOT bound in R6.
                self.retain_conditional_format(region);
                self.ledger_and_observe(
                    DocumentVariantTag::ConditionalFormatRegion,
                    IngestTier::B,
                    "retained-inert",
                );
            }
            OxCalcDocumentFeature::FormulaTopology(topology) => {
                self.route_formula_topology(topology)?;
                self.ledger_and_observe(
                    DocumentVariantTag::FormulaTopology,
                    IngestTier::A,
                    "routed-topology",
                );
            }
            OxCalcDocumentFeature::SharedFormulaRegion(region) => {
                self.install_shared_formula_region(region)?;
                self.ledger_and_observe(
                    DocumentVariantTag::SharedFormulaRegion,
                    IngestTier::A,
                    "expanded-repeated-region",
                );
            }
            OxCalcDocumentFeature::TableOverlay(table) => {
                self.table_overlays.push(IngestTableSpec {
                    sheet_id: table.sheet_id,
                    name: table.name.clone(),
                    range: table.range.clone(),
                });
                self.ledger_and_observe(
                    DocumentVariantTag::TableOverlay,
                    IngestTier::A,
                    "installed-table-overlay",
                );
            }
            OxCalcDocumentFeature::DefinedName(name) => {
                // Position-free (D4 §9): accumulate the owned spec and defer the
                // install to commit, after every sheet exists, so a name whose
                // target sheet appears LATER in the stream still resolves.
                self.defined_names.push(name.clone());
                self.ledger_and_observe(
                    DocumentVariantTag::DefinedName,
                    IngestTier::A,
                    "installed-defined-name",
                );
            }
            OxCalcDocumentFeature::ExternalLink(spec) => {
                // Row 27: link targets retained verbatim (a rect-less family, no
                // overlay). The formula bind-degradation contract (§14) is R6.5's.
                self.document_facts.external_links.push(spec.clone());
                self.ledger_and_observe(
                    DocumentVariantTag::ExternalLink,
                    IngestTier::B,
                    "retained-inert",
                );
            }
            OxCalcDocumentFeature::CalcChainHint(_) => self.ledger_and_observe(
                DocumentVariantTag::CalcChainHint,
                IngestTier::X,
                "excluded-engine-derives-order",
            ),
            OxCalcDocumentFeature::OpaquePartNotice(notice) => {
                // Row 29: notices retained verbatim (a rect-less family). The
                // geometry-coupled staleness gap is surfaced in the ledger.
                self.document_facts.opaque_notices.push(notice.clone());
                let disposition = match notice.geometry_coupling {
                    oxdoc_model::GeometryCoupling::None => "retained-inert",
                    oxdoc_model::GeometryCoupling::SheetAnchor
                    | oxdoc_model::GeometryCoupling::SourceRange => {
                        "retained-inert-geometry-coupled-stale-gap"
                    }
                };
                self.ledger_and_observe(
                    DocumentVariantTag::OpaquePartNotice,
                    IngestTier::B,
                    disposition,
                );
            }
        }
        Ok(())
    }
}

/// Drive an `oxdoc-model` event stream into `context`'s workbook workspace and
/// return the load report (D4 §9). The workspace must already exist with a
/// `Workbook`-role root; this is the R6.1 entry point (the public one-call verb
/// that also creates the workspace is R6.5).
///
/// The whole load is one transaction: the sink accumulates during the drive,
/// then [`OxCalcWorkbookIngestSink::commit_into`] mints exactly one revision.
pub fn load_workbook_events(
    context: &mut OxCalcDocumentContext,
    workspace_id: &OxCalcTreeWorkspaceId,
    events: &[DocumentEvent],
) -> Result<WorkbookLoadReport, OxCalcWorkbookIngestError> {
    let mut sink = OxCalcWorkbookIngestSink::new();
    drive_oxcalc_ingest(events, &mut sink).map_err(|err| match err {
        OxCalcIngestError::Sink(err) => err,
        OxCalcIngestError::Projection(err) => {
            OxCalcWorkbookIngestError::Rejected(format!("{err:?}"))
        }
    })?;
    sink.commit_into(context, workspace_id)
}

/// The workspace-create request the public [`load_workbook_model`] verb takes
/// (D4 §9). An alias for [`OxCalcTreeWorkspaceCreate`] under the design's name:
/// the verb forces the `Workbook` role on the created root regardless of the
/// request's `is_workbook` flag (a workbook load always creates a workbook root),
/// so a caller may pass a plain [`OxCalcTreeWorkspaceCreate::new`] and still get
/// a workbook. Only the `workspace_id` and `root_symbol` are honored as-is.
pub type OxCalcWorkbookCreate = OxCalcTreeWorkspaceCreate;

/// Create a fresh workbook workspace and load an `oxdoc-model` event stream into
/// it in ONE transaction (the public one-call verb, D4 §9). This is the R6.5
/// entry point wrapping R6.1–R6.4: it creates the `Workbook`-role workspace, then
/// drives the sink's accumulation + single-revision commit + calc-mode-conditional
/// load-recalc policy (D4 §6).
///
/// Because the workspace is freshly created, the R6.1 freshness carry-forward
/// cannot bite (no pre-existing `#workbook-settings`/`#workbook-ingest` group to
/// duplicate); [`OxCalcWorkbookIngestSink::commit_into`]'s guard enforces it
/// regardless. The returned [`WorkbookLoadReport::recalc_path`] records which
/// load-recalc path ran (`Automatic` open-recalc vs `Manual` render-from-cache),
/// and [`WorkbookLoadReport::engine_recalcs_at_load`] is the perf counter proving
/// Manual ran zero engine passes.
///
/// The chosen `workspace_id` (from `create`) is the handle the caller addresses
/// the loaded workbook by; it is returned alongside the report for convenience.
pub fn load_workbook_model(
    context: &mut OxCalcDocumentContext,
    create: OxCalcWorkbookCreate,
    events: &[DocumentEvent],
) -> Result<(OxCalcTreeWorkspaceId, WorkbookLoadReport), OxCalcDocumentError> {
    let workspace_id = context.create_workspace(create.as_workbook())?;
    let report = load_workbook_events(context, &workspace_id, events)?;
    Ok((workspace_id, report))
}

/// Create a fresh workbook workspace and load it from a neutral
/// [`WorkbookModelAccess`] (the model-access variant of [`load_workbook_model`],
/// D4 §9). Loads identical content to the events path: the access is driven
/// through [`drive_oxcalc_ingest_from_model_access`] (eager-events today, §15
/// gap 3 for a lazy path), then committed as one transaction with the same
/// calc-mode-conditional load-recalc policy.
pub fn load_workbook_model_from_access<A>(
    context: &mut OxCalcDocumentContext,
    create: OxCalcWorkbookCreate,
    access: &A,
) -> Result<(OxCalcTreeWorkspaceId, WorkbookLoadReport), OxCalcDocumentError>
where
    A: WorkbookModelAccess + ?Sized,
{
    let workspace_id = context.create_workspace(create.as_workbook())?;
    let mut sink = OxCalcWorkbookIngestSink::new();
    drive_oxcalc_ingest_from_model_access(access, &mut sink).map_err(|err| {
        OxCalcDocumentError::WorkbookIngestRejected {
            detail: format!("{err:?}"),
        }
    })?;
    let report = sink.commit_into(context, &workspace_id)?;
    Ok((workspace_id, report))
}

// -- The save projection: whole-model output stream (D4 §7a) ------------------

/// One authored cell to project (W062 R6.6, D4 §7a): a literal value or a formula
/// (source text + its freshly-published cached value). Read back from
/// [`crate::grid::authored::GridInputState`] (authored truth, C6) — never from a
/// derived key — plus, for a formula, the published readout at that address (the
/// **publication-time** cache refresh, C12: fresh-cache-by-construction).
#[derive(Debug, Clone, PartialEq)]
pub enum ProjectedCell {
    /// A literal authored value. Projected directly to a [`CellPayload`] scalar.
    Literal(CalcValue),
    /// A formula: its authored `source_text` (leading `=` retained as authored)
    /// and the value currently published at its address. The projection strips
    /// the leading `=` for the wire `text` and folds the published value into
    /// `cached` (C12 — read from publication, never stored staleness).
    Formula {
        source_text: String,
        published: Option<CalcValue>,
    },
}

/// One sheet's projection inputs (W062 R6.6, D4 §7a): its upstream id (the id
/// every sheet-scoped Tier-B fact keys against and the projection re-emits on
/// `SheetBegin`), display name, the authored cells in address order, and the
/// sheet-scoped authored Tier-A collections (W062 R6.66 / calc-5kqg.66).
///
/// The sheet-scoped collections — merged regions, table overlays, repeated
/// (shared) formula regions — are rendered to their wire events by the projection
/// verb (which resolves the engine sheet token to the upstream id + display name)
/// and carried here as ready-to-emit payloads. Defined names are WORKBOOK-scoped
/// events and ride [`WorkbookProjectionInputs::defined_names`] instead.
#[derive(Debug, Clone)]
pub struct ProjectedSheet {
    pub upstream_sheet_id: u32,
    pub display_name: String,
    /// `(row_one_based, col_one_based, cell)` in ascending address order.
    pub cells: Vec<(u32, u32, ProjectedCell)>,
    /// Merged-region ranges on this sheet, emitted as one `MergedCellRegions`
    /// event (empty ⇒ no event).
    pub merged_regions: Vec<CellRangeSpec>,
    /// Repeated (shared) formula regions on this sheet, one `SharedFormulaRegion`
    /// event each, in authored order.
    pub shared_formula_regions: Vec<SharedFormulaRegion>,
    /// Structured-table overlays on this sheet, one `TableOverlay` event each.
    pub tables: Vec<TableSpec>,
}

/// The gathered read-back a whole-model projection assembles from (W062 R6.6,
/// D4 §7a). The caller ([`OxCalcDocumentContext::project_workbook_model_output`])
/// reads these off private workspace state (settings, per-sheet authored cells +
/// published values, and the sealed Tier-B store); [`assemble_workbook_model_output`]
/// turns them into a validator-accepted `WholeModelProjection` event stream.
#[derive(Debug, Clone)]
pub struct WorkbookProjectionInputs<'a> {
    pub settings: WorkbookCalcSettings,
    /// Sheets in **registry order** (C3), which equals load stream order — so the
    /// i-th sheet's `upstream_sheet_id` is `facts.sheet_stream_ids[i]`.
    pub sheets: Vec<ProjectedSheet>,
    /// Workbook-scoped `DefinedName` events (W062 R6.66 / calc-5kqg.66), gathered
    /// across every sheet's authored names and emitted in the prelude (a
    /// `DefinedName` is a workbook-scoped event; its `scope_sheet_id`
    /// distinguishes a workbook name from a sheet-scoped one). Each carries its
    /// Tier-B `metadata` half re-attached from the store.
    pub defined_names: Vec<DefinedNameSpec>,
    /// The sealed inert Tier-B store (D4 §13): the verbatim source of every Tier-B
    /// event the projection replays.
    pub facts: &'a IngestedDocumentFacts,
}

/// Reverse of [`map_biff_error_code`] (D4 §7a/§10): a typed [`WorksheetErrorCode`]
/// back to the BIFF error byte a save writes. `Some(byte)` for the classic BIFF
/// set (the only codes with a stable byte); `None` for the newer Excel errors
/// (`#SPILL!`, `#CALC!`, …) which have no classic byte. A `None` here is a real
/// fidelity boundary the caller resolves: an ingested cell whose error had no
/// classic byte kept its **raw** byte in the Tier-B `unknown_error_bytes` store
/// (R6.1), and the projection writes THAT byte back verbatim — so a `None` from
/// this map is only reached for an error value the engine *computed* into a
/// non-classic code, which is written as `#VALUE!`'s byte (`0x0F`) with no
/// invented byte (mapped, never guessed silently — the ingest-side rule mirrored
/// on the save side).
#[must_use]
pub fn worksheet_error_to_biff_byte(code: WorksheetErrorCode) -> Option<u8> {
    match code {
        WorksheetErrorCode::Null => Some(0x00),
        WorksheetErrorCode::Div0 => Some(0x07),
        WorksheetErrorCode::Value => Some(0x0F),
        WorksheetErrorCode::Ref => Some(0x17),
        WorksheetErrorCode::Name => Some(0x1D),
        WorksheetErrorCode::Num => Some(0x24),
        WorksheetErrorCode::NA => Some(0x2A),
        // No classic BIFF byte (newer Excel errors). The projection never invents
        // one: an ingested non-classic byte round-trips via the raw-byte store; an
        // engine-computed non-classic error writes `#VALUE!`'s byte, honestly.
        WorksheetErrorCode::Busy
        | WorksheetErrorCode::GettingData
        | WorksheetErrorCode::Spill
        | WorksheetErrorCode::Calc
        | WorksheetErrorCode::Field
        | WorksheetErrorCode::Blocked
        | WorksheetErrorCode::Connect => None,
    }
}

/// A shared-string table derived **at write time** from the authored text the
/// projection walks (D4 §11: no persistent/identity-visible string table; dedup is
/// a pure serialization concern here). Text values become `SharedText(index)`
/// payloads pointing into this table; the table is emitted as the stream's single
/// `StringTable` prelude event.
#[derive(Debug, Default)]
struct WriteSharedStrings {
    entries: Vec<String>,
    index_of: BTreeMap<String, u32>,
}

impl WriteSharedStrings {
    /// Intern one text, returning its shared index. Dedups by exact string (D4
    /// §11 dedup-at-write): a repeated string reuses its first-assigned index.
    fn intern(&mut self, text: String) -> u32 {
        if let Some(&index) = self.index_of.get(&text) {
            return index;
        }
        let index = self.entries.len() as u32;
        self.index_of.insert(text.clone(), index);
        self.entries.push(text);
        index
    }

    fn into_string_table(self) -> Vec<SharedStringEntry> {
        self.entries
            .into_iter()
            .map(|text| SharedStringEntry { text })
            .collect()
    }
}

/// Project one [`CalcValue`] scalar to a wire [`CellPayload`] (D4 §7a). Text
/// values are interned into the write-time shared-string table (§11). This is the
/// reverse of the ingest's [`OxCalcWorkbookIngestSink::resolve_literal`]:
/// `Number`↔`Number`, `Logical`↔`Bool`, `Text`↔`SharedText`, `Error`↔`Error(byte)`.
///
/// `raw_error_byte`: when the caller knows this cell retained an unknown BIFF byte
/// (the R6.1 `unknown_error_bytes` store), it passes it so the byte round-trips
/// verbatim instead of being re-derived from the typed error — the no-launder rule
/// on the save side. `None` uses [`worksheet_error_to_biff_byte`].
fn project_scalar_payload(
    value: &CalcValue,
    strings: &mut WriteSharedStrings,
    raw_error_byte: Option<u8>,
) -> CellPayload {
    match value.core() {
        CoreValue::Number(n) => CellPayload::Number(*n),
        CoreValue::Logical(b) => CellPayload::Bool(*b),
        CoreValue::Text(text) => CellPayload::SharedText(strings.intern(text.to_string_lossy())),
        CoreValue::Error(code) => {
            // A retained raw byte (unknown-at-ingest code) writes back verbatim;
            // otherwise map the typed code to its classic BIFF byte, falling back to
            // `#VALUE!`'s byte for a code with no classic byte (never invented).
            let byte = raw_error_byte
                .or_else(|| worksheet_error_to_biff_byte(*code))
                .unwrap_or(0x0F);
            CellPayload::Error(byte)
        }
        // Empty / Missing carry no authored value (D4 §12 row 23: Empty → no
        // record). A literal cell should never hold Array/Reference (those are
        // computed shapes, not authored literals); project defensively to Empty so
        // the projection is total rather than panicking.
        CoreValue::Empty | CoreValue::Missing | CoreValue::Array(_) | CoreValue::Reference(_) => {
            CellPayload::Empty
        }
    }
}

/// Strip a single leading `=` from an authored formula source text (D4 §7a: the
/// wire `text` is `source_text`-without-leading-`=`). Ingest restored the `=` for
/// the authored record; the projection removes it for the neutral output, exactly
/// inverting that step. A source text with no leading `=` (defensive) is returned
/// unchanged.
pub(crate) fn strip_leading_equals(source_text: &str) -> String {
    source_text
        .strip_prefix('=')
        .unwrap_or(source_text)
        .to_string()
}

/// Assemble a whole-model projection event stream from the gathered read-back
/// (W062 R6.6, D4 §7a). Produces a validator-accepted stream:
///
/// 1. Prelude: `WorkbookHeader` (from calc settings), the write-time `StringTable`
///    (§11 dedup-at-write, derived from the authored text walked), and the Tier-B
///    `StyleTable` / `DifferentialStyleTable` if the store retained them.
/// 2. Per sheet in registry order: `SheetBegin(SheetRef{upstream id})`, the
///    sheet-scoped Tier-B facts that key to this sheet (verbatim from the store, in
///    the validator's required order — sheet-view/dimension/merges before the cell
///    band, review/drawing before it too), the authored cells as one `CellChunk`
///    (literals as scalars, formulas as `Formula{text, cached}`), then `SheetEnd`.
/// 3. Workbook-scoped Tier-B facts (external links, threaded-comment people) after
///    the last sheet.
///
/// The string table is built in a **first pass** over every sheet's authored text
/// so the single prelude `StringTable` covers all sheets (a stream carries exactly
/// one). `CalcChainHint` is deliberately omitted (D4 §7a point 3 / §12: a perf hint
/// with no fidelity content — OxDoc regenerates or drops it).
#[must_use]
pub fn assemble_workbook_model_output(
    inputs: &WorkbookProjectionInputs<'_>,
) -> WorkbookModelOutput {
    let facts = inputs.facts;

    // Pass 1: intern every authored text (literals + no formula text is a string,
    // but a literal Text cell and a formula's cached Text value both intern here)
    // so the single prelude StringTable covers the whole workbook (D4 §11). The
    // per-cell payloads are re-derived in pass 2 against this same table, so the
    // interning order is identical and the indices align.
    let mut strings = WriteSharedStrings::default();
    for sheet in &inputs.sheets {
        for (_row, _col, cell) in &sheet.cells {
            match cell {
                ProjectedCell::Literal(value) => {
                    intern_text_of(value, &mut strings);
                }
                ProjectedCell::Formula { published, .. } => {
                    if let Some(value) = published {
                        intern_text_of(value, &mut strings);
                    }
                }
            }
        }
    }

    let mut events: Vec<DocumentEvent> = Vec::new();

    // -- Prelude (D4 §7a point 1 header + §11 strings + §12 rows 3/4 styles) ----
    events.push(DocumentEvent::WorkbookHeader(WorkbookHeader::new(
        project_date_system(inputs.settings.date_system),
        project_calc_mode(inputs.settings.calc_mode),
    )));
    // The write-time shared-string table (§11): consumed here as the single
    // prelude StringTable. Rebuilt from authored text, never a persisted table.
    let string_table = std::mem::take(&mut strings).into_string_table();
    events.push(DocumentEvent::StringTable(string_table));
    // StyleTable (Tier B, row 3): replay verbatim from the store. The validator
    // requires a StyleTable before any CellChunk; the store carries it only if the
    // load prelude did (else the fixture's minimal table is not synthesized — an
    // absent style table stays absent, faithful to the source).
    if let Some(style_table) = &facts.style_table {
        events.push(DocumentEvent::StyleTable(style_table.clone()));
    }
    // DifferentialStyleTable (Tier B, row 4): after StyleTable, before content.
    if !facts.differential_styles.is_empty() {
        events.push(DocumentEvent::DifferentialStyleTable(
            facts.differential_styles.clone(),
        ));
    }
    // DefinedName (Tier A + Tier-B metadata half, D4 §12 row 26) is a
    // WORKBOOK-scoped event; the validator accepts it before the first sheet, so
    // it replays in the prelude (W062 R6.66 / calc-5kqg.66). Both workbook-scoped
    // (`scope_sheet_id: None`) and sheet-scoped names ride here — the
    // `scope_sheet_id` field carries the distinction. Rendered by the verb (rect →
    // absolute A1 for a static name, retained source text for a dynamic one) with
    // the Tier-B metadata re-attached from the store.
    for name in &inputs.defined_names {
        events.push(DocumentEvent::DefinedName(name.clone()));
    }
    // ThreadedCommentPeople (Tier B, row 17) is WORKBOOK-scoped and the validator
    // rejects it *after* any sheet content (`WorkbookScopedEventAfterSheetContent`),
    // so it replays in the prelude — before the first `SheetBegin` — not after the
    // sheets. Verbatim from the store.
    for people in &facts.threaded_comment_people {
        events.push(DocumentEvent::ThreadedCommentPeople(people.clone()));
    }

    // -- Per-sheet content (D4 §7a point 1 cells + point 2 Tier-B verbatim) -----
    for sheet in &inputs.sheets {
        let sheet_id = sheet.upstream_sheet_id;
        events.push(DocumentEvent::SheetBegin(SheetRef {
            sheet_id,
            name: sheet.display_name.clone(),
        }));

        // Sheet-scoped Tier-B facts that must precede the cell band (the validator
        // rejects SheetReviewComments/DrawingFormControls *after* a CellChunk, and
        // keeps every other sheet-scoped fact inside the open sheet). Replayed
        // verbatim from the store, filtered to THIS sheet's upstream id.
        replay_pre_cell_sheet_facts(facts, sheet_id, &mut events);

        // Sheet-scoped authored Tier-A collections (W062 R6.66 / calc-5kqg.66),
        // emitted inside the sheet before the cell band. The stream validator ties
        // each only to an open sheet (no ordering vs the CellChunk), so the pre-cell
        // position is a safe, canonical placement. `MergedCellRegions` collapses a
        // sheet's rects into one event; shared-formula regions and tables emit one
        // event each in authored order.
        if !sheet.merged_regions.is_empty() {
            events.push(DocumentEvent::MergedCellRegions(MergedCellRegions {
                sheet_id,
                ranges: sheet.merged_regions.clone(),
                raw_refs: Vec::new(),
            }));
        }
        for region in &sheet.shared_formula_regions {
            events.push(DocumentEvent::SharedFormulaRegion(region.clone()));
        }
        for table in &sheet.tables {
            events.push(DocumentEvent::TableOverlay(table.clone()));
        }

        // Authored cells → one CellChunk (D4 §7a point 1). Literals as scalars,
        // formulas as Formula{text-without-'=', cached-from-publication}. Address
        // order is the sheet's ascending (row, col) — the caller sorts.
        let mut chunk_cells: Vec<(PackedCellAddr, CellPayload)> = Vec::new();
        for (row, col, cell) in &sheet.cells {
            let addr = PackedCellAddr::from_one_based(*row, *col)
                .expect("authored cell address is one-based and in-range");
            let payload = project_cell_payload(cell, facts, sheet_id, *row, *col, &mut strings);
            chunk_cells.push((addr, payload));
        }
        if !chunk_cells.is_empty() {
            events.push(DocumentEvent::CellChunk(CellChunk {
                row_band: 0,
                cells: chunk_cells,
            }));
        }

        // Sheet-scoped Tier-B facts that may follow the cell band.
        replay_post_cell_sheet_facts(facts, sheet_id, &mut events);

        events.push(DocumentEvent::SheetEnd { sheet_id });
    }

    // -- Workbook-scoped Tier-B facts (after the last sheet) --------------------
    replay_workbook_scoped_facts(facts, &mut events);

    WorkbookModelOutput::whole_model_projection(events)
}

/// Intern the text of a `Text`-cored value into the write-time table (a no-op for
/// non-text values). Shared by pass 1 (table build) so pass 2's re-derivation
/// aligns.
fn intern_text_of(value: &CalcValue, strings: &mut WriteSharedStrings) {
    if let CoreValue::Text(text) = value.core() {
        strings.intern(text.to_string_lossy());
    }
}

/// Project one [`ProjectedCell`] to its wire [`CellPayload`] (D4 §7a). A literal
/// is a scalar; a formula is `Formula{ region: None, text: preserved,
/// cached: from-publication }`.
fn project_cell_payload(
    cell: &ProjectedCell,
    facts: &IngestedDocumentFacts,
    sheet_id: u32,
    row: u32,
    col: u32,
    strings: &mut WriteSharedStrings,
) -> CellPayload {
    // A cell that retained an unknown BIFF error byte at ingest (R6.1) writes THAT
    // byte back verbatim, never re-derived from the typed #VALUE! it published.
    let raw_error_byte = facts
        .unknown_error_bytes
        .iter()
        .find(|retention| {
            retention.sheet_id == sheet_id && retention.row == row && retention.col == col
        })
        .map(|retention| retention.raw_byte);
    match cell {
        ProjectedCell::Literal(value) => project_scalar_payload(value, strings, raw_error_byte),
        ProjectedCell::Formula {
            source_text,
            published,
        } => CellPayload::Formula {
            region: None,
            // The authored text with its leading `=` stripped (D4 §7a): preserved
            // authored truth, never re-serialized from a bound key.
            text: Some(strip_leading_equals(source_text)),
            // The cache from PUBLICATION at projection time (C12): an edited-and-
            // recalculated formula saves a fresh cache by construction. A formula
            // with no published value (should not occur for a bound cell) writes no
            // cache rather than an invented one.
            cached: published
                .as_ref()
                .map(|value| Box::new(project_scalar_payload(value, strings, raw_error_byte))),
        },
    }
}

/// Replay the sheet-scoped Tier-B facts that must precede the cell band, in the
/// validator's required order (D4 §7a point 2). Filtered to `sheet_id`; every fact
/// is cloned **verbatim** from the store — event-level equality against the source
/// is the acceptance (R6.6 step 3).
fn replay_pre_cell_sheet_facts(
    facts: &IngestedDocumentFacts,
    sheet_id: u32,
    events: &mut Vec<DocumentEvent>,
) {
    for dimension in facts
        .sheet_dimensions
        .iter()
        .filter(|d| d.sheet_id == sheet_id)
    {
        events.push(DocumentEvent::SheetDimension(dimension.clone()));
    }
    for runs in facts.column_props.iter().filter(|r| r.sheet_id == sheet_id) {
        events.push(DocumentEvent::ColumnProps(runs.runs.clone()));
    }
    for runs in facts.row_props.iter().filter(|r| r.sheet_id == sheet_id) {
        events.push(DocumentEvent::RowProps(runs.runs.clone()));
    }
    for view in facts.sheet_views.iter().filter(|v| v.sheet_id == sheet_id) {
        events.push(DocumentEvent::SheetViewState(view.clone()));
    }
    for hyperlinks in facts.hyperlinks.iter().filter(|h| h.sheet_id == sheet_id) {
        events.push(DocumentEvent::Hyperlinks(hyperlinks.clone()));
    }
    for validations in facts
        .data_validations
        .iter()
        .filter(|v| v.sheet_id == sheet_id)
    {
        events.push(DocumentEvent::DataValidations(validations.clone()));
    }
    for filter in facts.auto_filters.iter().filter(|f| f.sheet_id == sheet_id) {
        events.push(DocumentEvent::AutoFilter(filter.clone()));
    }
    for sort_state in facts.sort_states.iter().filter(|s| s.sheet_id == sheet_id) {
        events.push(DocumentEvent::SortState(sort_state.clone()));
    }
    for notice in facts
        .comment_notices
        .iter()
        .filter(|n| n.sheet_id == sheet_id)
    {
        events.push(DocumentEvent::CommentNotice(notice.clone()));
    }
    // SheetReviewComments and DrawingFormControls MUST precede any CellChunk
    // (validator: SheetScopedEventAfterCellChunk). Replayed here, before the band.
    for comments in facts
        .sheet_review_comments
        .iter()
        .filter(|c| c.sheet_id == sheet_id)
    {
        events.push(DocumentEvent::SheetReviewComments(comments.clone()));
    }
    for controls in facts
        .drawing_form_controls
        .iter()
        .filter(|c| c.sheet_id == sheet_id)
    {
        events.push(DocumentEvent::DrawingFormControls(controls.clone()));
    }
    for regions in facts
        .conditional_formats
        .iter()
        .filter(|c| c.sheet_id == sheet_id)
    {
        events.push(DocumentEvent::ConditionalFormatRegion(regions.clone()));
    }
    for topology in facts
        .formula_topologies
        .iter()
        .filter(|t| t.sheet_id == sheet_id)
    {
        events.push(DocumentEvent::FormulaTopology(topology.clone()));
    }
    for runs in facts
        .cell_format_runs
        .iter()
        .filter(|r| r.sheet_id == sheet_id)
    {
        events.push(DocumentEvent::CellFormatRuns(runs.runs.clone()));
    }
}

/// Replay the sheet-scoped Tier-B facts that may follow the cell band (D4 §7a
/// point 2): merged regions (validator permits them anywhere in-sheet; placed
/// after the band alongside the cells they annotate). Verbatim, filtered to
/// `sheet_id`.
fn replay_post_cell_sheet_facts(
    facts: &IngestedDocumentFacts,
    sheet_id: u32,
    events: &mut Vec<DocumentEvent>,
) {
    // Merged regions are stored as the calc-model authored merges (Tier A) AND the
    // store does not separately retain a `MergedCellRegions` fact — the merges the
    // grid owns are projected from authored state, not replayed here. This hook is
    // a seam for any post-cell sheet-scoped Tier-B family; none exist in R6.6's
    // store shape, so it currently emits nothing (kept for the ordering contract's
    // symmetry with the pre-cell replay).
    let _ = (facts, sheet_id, events);
}

/// Replay the workbook-scoped Tier-B facts that the validator leaves position-free
/// after sheet content (D4 §7a point 2): external links (row 27) and opaque-part
/// notices (row 29). `ThreadedCommentPeople` is NOT here — it is workbook-scoped
/// but must precede sheet content, so it replays in the prelude. Verbatim.
fn replay_workbook_scoped_facts(facts: &IngestedDocumentFacts, events: &mut Vec<DocumentEvent>) {
    for link in &facts.external_links {
        events.push(DocumentEvent::ExternalLink(link.clone()));
    }
    for notice in &facts.opaque_notices {
        events.push(DocumentEvent::OpaquePartNotice(notice.clone()));
    }
}

/// Map OxCalc's `DateSystem` to oxdoc-model's (the save-side inverse of
/// [`map_date_system`], D4 §12 row 1).
fn project_date_system(date_system: DateSystem) -> oxdoc_model::DateSystem {
    match date_system {
        DateSystem::Excel1900 => oxdoc_model::DateSystem::Date1900,
        DateSystem::Excel1904 => oxdoc_model::DateSystem::Date1904,
    }
}

/// Map OxCalc's `CalcMode` to oxdoc-model's (the save-side inverse of
/// [`map_calc_mode`], D4 §12 row 1).
fn project_calc_mode(calc_mode: CalcMode) -> oxdoc_model::CalcMode {
    match calc_mode {
        CalcMode::Automatic => oxdoc_model::CalcMode::Automatic,
        CalcMode::Manual => oxdoc_model::CalcMode::Manual,
    }
}

/// Map a BIFF error code (D4 §10) to a typed [`WorksheetErrorCode`].
///
/// The classic BIFF error byte set is the writer-side canon
/// (`oxdoc-xlsx`'s `error_code`): `0x00`→`#NULL!`, `0x07`→`#DIV/0!`,
/// `0x0F`→`#VALUE!`, `0x17`→`#REF!`, `0x1D`→`#NAME?`, `0x24`→`#NUM!`,
/// `0x2A`→`#N/A`. The newer Excel errors (`#SPILL!`, `#CALC!`, …) have no
/// classic BIFF byte, so a byte outside this set is *unknown*: it maps to
/// `#VALUE!` and the caller ledgers a row (D4 §10 — mapped, never guessed
/// silently). Returns `(mapped, known)` so the caller can tell a real mapping
/// from the unknown-fallback.
#[must_use]
pub fn map_biff_error_code(code: u8) -> (WorksheetErrorCode, bool) {
    match code {
        0x00 => (WorksheetErrorCode::Null, true),
        0x07 => (WorksheetErrorCode::Div0, true),
        0x0F => (WorksheetErrorCode::Value, true),
        0x17 => (WorksheetErrorCode::Ref, true),
        0x1D => (WorksheetErrorCode::Name, true),
        0x24 => (WorksheetErrorCode::Num, true),
        0x2A => (WorksheetErrorCode::NA, true),
        // Unknown byte: publish #VALUE!, caller ledgers the raw byte (D4 §10).
        _ => (WorksheetErrorCode::Value, false),
    }
}

/// Map oxdoc-model's `DateSystem` to OxCalc's (D4 §12 row 1).
fn map_date_system(date_system: oxdoc_model::DateSystem) -> DateSystem {
    match date_system {
        oxdoc_model::DateSystem::Date1900 => DateSystem::Excel1900,
        oxdoc_model::DateSystem::Date1904 => DateSystem::Excel1904,
    }
}

/// Map oxdoc-model's `CalcMode` to OxCalc's (D4 §12 row 1).
fn map_calc_mode(calc_mode: oxdoc_model::CalcMode) -> CalcMode {
    match calc_mode {
        oxdoc_model::CalcMode::Automatic => CalcMode::Automatic,
        oxdoc_model::CalcMode::Manual => CalcMode::Manual,
    }
}

/// Map oxdoc-model's `IterativeCalcSettings` to OxCalc's `IterationSettings`
/// (D4 §12 row 1; W062 gap #1 / W055 enablement). The two structs are distinct
/// types (OxCalc keeps its own copy) whose fields are one-to-one and identically
/// named, so the mapping is a total field-by-field copy — every field carried,
/// none defaulted. Sourced from the file's OOXML `<calcPr>` iterate/iterateCount/
/// iterateDelta attributes upstream.
fn map_iteration_settings(iterative_calc: oxdoc_model::IterativeCalcSettings) -> IterationSettings {
    IterationSettings {
        enabled: iterative_calc.enabled,
        max_iterations: iterative_calc.max_iterations,
        max_change: iterative_calc.max_change,
    }
}

// -- The single-transaction load plan (consumed by consumer.rs's builder) -----

/// The Tier-A load plan handed to the consumer's single-transaction builder
/// (D4 §9). Plain data: settings + ordered sheets, each with its literal cells,
/// formula cells, repeated regions, and non-calc-modeled cached publications,
/// plus the sealed Tier-B inert store (§13). The builder binds formulas (single
/// key mint), seeds `FileCached` publications, installs everything in one
/// revision, writes the store's digest into the `#workbook-ingest` meta-child,
/// and sets the workspace's live `ingested_document_facts` to the store `Arc`.
#[derive(Debug, Clone)]
pub struct WorkbookTierALoadPlan {
    pub settings: WorkbookCalcSettings,
    pub sheets: Vec<SheetTierALoad>,
    /// The sealed inert Tier-B retention store (D4 §13). Immutable after load;
    /// held by the workspace as `Arc<IngestedDocumentFacts>` and its digest
    /// written into `#workbook-ingest` for revision identity.
    pub document_facts: std::sync::Arc<IngestedDocumentFacts>,
}

/// The outcome of a single-transaction Tier-A load (D4 §9/§10): the bind
/// degradations the builder produced (formula text OxFml rejected — retained,
/// never dropped) and the count of formula cells that bound into the calc graph.
/// The sink folds these into the [`WorkbookLoadReport`].
#[derive(Debug, Clone, Default)]
pub struct WorkbookTierALoadOutcome {
    /// One row per formula cell whose text OxFml rejected as a formula. The text
    /// is retained here, the cell publishes its `FileCached` value (or `#NAME?`),
    /// and the load still succeeds (D4 §10).
    pub bind_degradations: Vec<BindDegradation>,
    /// How many formula cells bound through the single key mint into the calc
    /// graph (excludes degraded and non-calc-modeled cells).
    pub formulas_bound: u32,
    /// How many region-managed cells were not covered by any installed
    /// `SharedFormulaRegion` rect (a dangling `region: Some(_)` — malformed
    /// stream). Their cache still publishes; this counts them so nothing is
    /// silently dropped (C13).
    pub region_cells_unbacked: u32,
    /// How many defined names were actually authored into the calc model (D4 §12
    /// row 26). Static names always install; a dynamic name whose defining
    /// formula OxFml rejected degrades (retained text + a `BindDegradation` row)
    /// and is NOT counted here — the count is the honest "bound into the graph"
    /// number, mirroring `formulas_bound`.
    pub names_installed: u32,
    /// How many structured tables were registered on their sheet's grid (D4 §12
    /// row 25). Every table the sink resolved to a sheet + rect installs.
    pub tables_installed: u32,
    /// One row per bound formula cell that references an **external** workbook
    /// (`[Book2]Sheet1!A1`, D4 §14): it binds normally (authored text retained)
    /// but its `FileCached` value is **pinned** — recalc never evaluates it (it
    /// cannot, honestly) and never clobbers the cache — with the
    /// `ExternalReferenceNotLinked` disposition. A subset of `formulas_bound`, so
    /// the no-silent-loss regime accounts for every external-referencing cell
    /// (C13). The pin holds until D2 §5 cross-workspace routing lands. The sink
    /// folds this verbatim into [`WorkbookLoadReport::external_reference_pins`].
    pub external_reference_pins: Vec<ExternalReferencePin>,
    /// How many engine recalc passes the load ran (the perf counter proving the
    /// D4 §6 load-recalc policy). Under `CalcMode::Automatic` the load issues the
    /// open-recalc — one engine pass per sheet carrying calc work — so this is
    /// non-zero; under `CalcMode::Manual` the load binds + seeds `FileCached` and
    /// runs **zero** engine passes (the workbook renders from caches until an
    /// explicit F9), so this is exactly `0`. The Manual-zero-eval acceptance
    /// asserts on this counter.
    pub engine_recalcs_at_load: u32,
    /// Which load-recalc path the builder ran (D4 §6/§9): `Automatic` (open-recalc,
    /// published values `Calculated`) or `Manual` (no engine evaluation, published
    /// values `FileCached` until F9). Folded into the
    /// [`WorkbookLoadReport::recalc_path`].
    pub recalc_path: LoadRecalcPath,
}

/// One sheet's Tier-A load: its display name, upstream id, literal cells,
/// formula cells (bound or degraded at commit), repeated-formula regions
/// (shared/CSE), and non-calc-modeled cached publications (DataTable/Unknown).
#[derive(Debug, Clone)]
pub struct SheetTierALoad {
    pub display_name: String,
    pub upstream_sheet_id: u32,
    /// `(row_one_based, col_one_based, value)` literal cells in stream order.
    pub literals: Vec<(u32, u32, CalcValue)>,
    /// Formula cells staged for bind-or-degrade at commit (D4 §10). Excludes
    /// region-managed cells (`region: Some(_)`), which the region installs.
    pub formulas: Vec<IngestFormula>,
    /// Repeated-formula regions (shared formulas / legacy-CSE arrays, D4 §12
    /// rows 22/24): one R1C1 template tiled over a rect.
    pub repeated_regions: Vec<IngestRepeatedRegion>,
    /// `(row, col, cached_value)` FileCached publications for region-managed
    /// cells (D4 §12 row 24): a shared region's member/anchor cells arrive as
    /// `Formula { region: Some(_), .. }` carrying only a cache; the region
    /// installs their formula (single mint at the anchor), so they are never
    /// individually bound — only their cache is published (transient).
    pub region_cell_caches: Vec<(u32, u32, CalcValue)>,
    /// `(row, col, cached_value)` publications for cells that do **not** enter
    /// the calc graph (DataTable/Unknown, D4 §12 row 22): they render their
    /// `FileCached` value and are never bound.
    pub unmodeled_cached: Vec<(u32, u32, CalcValue)>,
    /// Merged-region rects to register on this sheet's grid (D4 §12 row 10). The
    /// builder folds these into `GridInputState::merged_regions` before build, so
    /// `add_merged_region`'s live spill-block / edit-admission semantics apply.
    pub merged_regions: Vec<IngestMergedRegionInstall>,
    /// Structured-table overlays to register on this sheet's grid (D4 §12 row
    /// 25). The builder folds these into `GridInputState::table_overlays` before
    /// build, so `set_table_overlay`'s live structured-reference resolution
    /// applies.
    pub table_overlays: Vec<IngestTableOverlayInstall>,
    /// Defined names to author on this sheet's grid (D4 §12 row 26). The builder
    /// folds these into `GridInputState::defined_names` before build, so the
    /// engine's name setters register them with the same scope precedence a live
    /// `define_name` verb would (the derived namespace is a pure function of
    /// `defined_names`, so a rebuild-from-input re-registers identically).
    pub defined_names: Vec<IngestDefinedNameInstall>,
}

impl SheetTierALoad {
    /// Build the literal authored-cell list for
    /// [`crate::grid::authored::GridInputState`] seeding, given the workbook and
    /// sheet tokens. Formulas and regions are installed separately by the builder
    /// (they bind through the single key mint against the live sheet).
    #[must_use]
    pub fn authored_cells(
        &self,
        workbook_token: &str,
        sheet_token: &str,
    ) -> Vec<(ExcelGridCellAddress, GridAuthoredCell)> {
        self.literals
            .iter()
            .map(|(row, col, value)| {
                (
                    ExcelGridCellAddress::new(workbook_token, sheet_token, *row, *col),
                    GridAuthoredCell::Literal(value.clone()),
                )
            })
            .collect()
    }
}

/// The per-sheet distribution of the deferred name/table/merge installs (D4 §9),
/// index-aligned with the sink's `sheets` vector, plus the report facts.
struct DeferredInstalls {
    /// One bucket per sheet: the merged-region rects to register there.
    merges_by_sheet: Vec<Vec<IngestMergedRegionInstall>>,
    /// One bucket per sheet: the table overlays to register there.
    tables_by_sheet: Vec<Vec<IngestTableOverlayInstall>>,
    /// One bucket per sheet: the defined names to author there.
    names_by_sheet: Vec<Vec<IngestDefinedNameInstall>>,
    /// The defined-name metadata (Tier B, row 26 B-half), keyed by name — folded
    /// into [`IngestedDocumentFacts::name_metadata`] (the retention home) by the
    /// caller. Retained for EVERY name carrying metadata, whether or not its calc
    /// binding resolved — the round-trip home is independent of the install (D4
    /// §12 row 26 A/B split). The installed-name/table counts come from the
    /// builder's outcome (a dynamic name can still degrade at bind), not here.
    name_metadata: Vec<IngestedDefinedNameMetadata>,
    /// One [`BindDegradation`] per deferred install that could NOT be applied to
    /// the calc model at resolution time — covering **names** and **tables**
    /// alike (the same no-silent-loss doctrine, C13):
    /// - a **name** (`name:{name}`) whose target sheet is absent (a
    ///   `#REF!`-orphaned name from a deleted-sheet XLSX), or a sheet-scoped name
    ///   whose scope sheet differs from its static rect's sheet (an engine
    ///   limitation: `set_sheet_defined_name` requires the rect on the authoring
    ///   sheet, so the scope + target cannot both be honored);
    /// - a **table** (`table:{name}`) whose `range` string is not parseable A1
    ///   (`TableSpec.range` is a raw, un-validated producer string).
    ///
    /// These ride the existing degradation channel into the report's
    /// `bind_degradations`, so a dropped install is NEVER silent (names and tables
    /// get the same honesty as cells). Metadata for a dropped name is still
    /// retained (the Tier-B round-trip home is install-independent).
    dropped_installs: Vec<BindDegradation>,
}

/// Resolve the position-free deferred installs (D4 §9) against the completed
/// sheet map and distribute them per sheet. Runs at commit — after the whole
/// drive — so a name/table/merge whose target sheet appeared anywhere in the
/// stream resolves, forward references included. Nothing here relies on the
/// stream validator's ordering: the completed sheet map is the only authority.
///
/// - **Merges** (row 10): each carries an upstream sheet id; an unknown id drops
///   the rect (a malformed stream never fabricates a merge on the wrong sheet).
/// - **Tables** (row 25): each carries its own sheet id + A1 range string; the
///   range parses into a rect and per-column bands (header row = the top row). An
///   unparseable range (a raw, un-validated producer string) drops the table but
///   is **surfaced** via a `table:{name}` degradation (see `dropped_installs`),
///   never silently. The sheet-id miss is unreachable in a valid stream (the
///   validator guarantees an open sheet), so it is a plain skip.
/// - **Names** (row 26): position-free. A rect-denoting `formula_text` installs
///   as a **static** name on the sheet its rect names (the engine requires a
///   static rect on the authoring sheet); any other text installs as a
///   **dynamic** name on the scope sheet (sheet-scoped) or its own referenced
///   sheet. A name that resolves to NO sheet — or a sheet-scoped name whose scope
///   sheet differs from its rect's sheet (unmodelable, see `dropped_installs`) — is
///   NOT installed but is **accounted honestly**: a `BindDegradation` row records
///   the name text + reason (no-silent-loss, C13), and its metadata is retained.
fn resolve_deferred_installs(
    sheet_count: usize,
    sheet_index_by_upstream: &BTreeMap<u32, usize>,
    sheet_index_by_display: &BTreeMap<String, usize>,
    sheet_display_by_upstream: &BTreeMap<u32, String>,
    merged_regions: &[(u32, IngestMergedRect)],
    table_overlays: &[IngestTableSpec],
    defined_names: &[DefinedNameSpec],
) -> DeferredInstalls {
    let mut merges_by_sheet: Vec<Vec<IngestMergedRegionInstall>> = vec![Vec::new(); sheet_count];
    let mut tables_by_sheet: Vec<Vec<IngestTableOverlayInstall>> = vec![Vec::new(); sheet_count];
    let mut names_by_sheet: Vec<Vec<IngestDefinedNameInstall>> = vec![Vec::new(); sheet_count];
    let mut name_metadata: Vec<IngestedDefinedNameMetadata> = Vec::new();
    let mut dropped_installs: Vec<BindDegradation> = Vec::new();

    // Merges: upstream sheet id → sheet bucket. The `else` (sheet id absent from
    // the map) is UNREACHABLE for a valid stream: `MergedCellRegions` is
    // sheet-scoped and `validate_event_stream` (OxDoc lib.rs ~2894,
    // `ensure_sheet_id`) requires the matching sheet be open when the event
    // arrives, so its id is always in the completed sheet map. Hence a plain skip,
    // not a degradation — there is no reachable silent-loss path here.
    for (sheet_id, rect) in merged_regions {
        if let Some(&index) = sheet_index_by_upstream.get(sheet_id) {
            merges_by_sheet[index].push(IngestMergedRegionInstall {
                top_row: rect.top_row,
                left_col: rect.left_col,
                bottom_row: rect.bottom_row,
                right_col: rect.right_col,
            });
        }
    }

    // Tables: own sheet id + A1 range → rect + column bands (header = top row).
    for table in table_overlays {
        // Sheet-id miss is UNREACHABLE for a valid stream: `TableOverlay` is
        // sheet-scoped and `validate_event_stream` (OxDoc lib.rs ~3011,
        // `ensure_sheet`) requires a sheet open when it arrives; the table carries
        // that same sheet's id, so it is in the map. Skip (no reachable loss).
        let Some(&index) = sheet_index_by_upstream.get(&table.sheet_id) else {
            continue;
        };
        // The range parse CAN fail: `TableSpec.range` is a raw producer string the
        // validator does NOT check for A1 syntax, so a malformed/alternative range
        // is a reachable drop. Surface it on the same channel names use — never a
        // silent skip (C13).
        let Some((top_row, left_col, bottom_row, right_col)) = parse_a1_rect(&table.range) else {
            dropped_installs.push(BindDegradation {
                address: format!("table:{}", table.name),
                text: table.range.clone(),
                diagnostics: vec![format!("unparseable table range '{}'", table.range)],
            });
            continue;
        };
        // One column band per column of the range. Column names come from the
        // header cells only if the file carried them; oxdoc-model's `TableSpec`
        // does not, so the band names are synthesized positionally
        // (`Column{ordinal}`) — enough for a `Name[Column1]` structured reference
        // to resolve to the right data rect. Faithful column names are R6.4's
        // (the Tier-B table-part store carries `tableColumn` names).
        let columns = (left_col..=right_col)
            .enumerate()
            .map(|(ordinal, col)| IngestTableColumn {
                name: format!("Column{}", ordinal + 1),
                col,
            })
            .collect();
        tables_by_sheet[index].push(IngestTableOverlayInstall {
            name: table.name.clone(),
            top_row,
            left_col,
            bottom_row,
            right_col,
            columns,
            has_header: true,
        });
    }

    // Names: resolve scope + target, choose the authoring sheet.
    for spec in defined_names {
        // Retain metadata regardless of whether the calc binding resolves — the
        // round-trip home (Tier B) is independent of the calc install (D4 §12
        // row 26 splits A/B). R6.4 swaps the real store for this stub.
        if !spec.metadata.is_empty() {
            name_metadata.push(IngestedDefinedNameMetadata {
                name: spec.name.clone(),
                scope_sheet_id: spec.scope_sheet_id,
                metadata: spec.metadata.clone(),
            });
        }

        // A `BindDegradation` recording this name as unresolvable — the honest
        // no-silent-loss account (C13). Named once so both branches surface a drop
        // through the same channel a formula cell uses.
        let degrade = |diagnostic: String| BindDegradation {
            address: format!("name:{}", spec.name),
            text: restore_leading_eq(&spec.formula_text),
            diagnostics: vec![diagnostic],
        };

        let sheet_scoped = spec.scope_sheet_id.is_some();
        match parse_rect_denoting_reference(&spec.formula_text) {
            Some(RectReference {
                sheet: reference_sheet,
                top_row,
                left_col,
                bottom_row,
                right_col,
            }) => {
                // Static: the engine's `set_defined_name` / `set_sheet_defined_name`
                // require the rect on the AUTHORING sheet (`check_rect` rejects a
                // cross-sheet rect — verified at the setters), so a static name is
                // authored on the sheet its rect names. A target sheet absent from
                // the workbook (a `#REF!`-orphaned name from a deleted sheet) can
                // not be authored: it is DROPPED and surfaced (never silent).
                let Some(&index) = sheet_index_by_display.get(&reference_sheet) else {
                    dropped_installs.push(degrade(format!(
                        "unresolvable defined-name target sheet '{reference_sheet}'"
                    )));
                    continue;
                };
                // A SHEET-scoped static name whose scope sheet differs from its
                // target rect's sheet cannot be modeled faithfully: the engine
                // keys the scope by the authoring grid's own sheet id, and the rect
                // must live on that same sheet (`check_rect`). Honoring the scope
                // would put the rect on the wrong sheet; honoring the rect would
                // silently re-scope the name to the target sheet. Neither is
                // faithful, so we do NOT install and surface the limitation rather
                // than silently reassigning scope (review finding #2).
                if let Some(scope_upstream) = spec.scope_sheet_id {
                    let scope_display = sheet_display_by_upstream.get(&scope_upstream);
                    if scope_display != Some(&reference_sheet) {
                        let scope_label = scope_display
                            .cloned()
                            .unwrap_or_else(|| format!("#{scope_upstream}"));
                        dropped_installs.push(degrade(format!(
                            "sheet-scoped name scoped to '{scope_label}' but its static rect \
                             is on '{reference_sheet}'; a scoped name's rect must live on its \
                             scope sheet (engine limitation) — not modeled"
                        )));
                        continue;
                    }
                }
                names_by_sheet[index].push(IngestDefinedNameInstall {
                    name: spec.name.clone(),
                    sheet_scoped,
                    target: IngestDefinedNameTarget::Static {
                        top_row,
                        left_col,
                        bottom_row,
                        right_col,
                    },
                });
            }
            None => {
                // Dynamic: bind the defining formula through the single mint (§3)
                // at the anchor sheet. A SHEET-scoped dynamic name's scope both
                // confines its visibility AND fixes its anchor, so its scope sheet
                // MUST resolve — an absent scope sheet is surfaced, never silently
                // re-anchored (the scope-honesty rule that finding #2 established
                // for static names, applied to the dynamic scope too). A
                // WORKBOOK-scoped dynamic name is visible everywhere and (for a
                // fully-qualified formula) anchor-independent, so its anchor is: the
                // first embedded sheet qualifier that resolves → the first sheet.
                // A workbook with no sheets at all has nowhere to anchor.
                let home = if let Some(scope_upstream) = spec.scope_sheet_id {
                    match sheet_index_by_upstream.get(&scope_upstream).copied() {
                        Some(index) => Some(index),
                        None => {
                            dropped_installs.push(degrade(format!(
                                "sheet-scoped dynamic name's scope sheet #{scope_upstream} \
                                 is absent from the workbook — not modeled"
                            )));
                            continue;
                        }
                    }
                } else {
                    first_embedded_sheet_qualifier(&spec.formula_text)
                        .and_then(|sheet| sheet_index_by_display.get(&sheet).copied())
                        .or(if sheet_count > 0 { Some(0) } else { None })
                };
                let Some(index) = home else {
                    dropped_installs.push(degrade(
                        "workbook-scoped dynamic name has no sheet to anchor on".to_string(),
                    ));
                    continue;
                };
                names_by_sheet[index].push(IngestDefinedNameInstall {
                    name: spec.name.clone(),
                    sheet_scoped,
                    target: IngestDefinedNameTarget::Dynamic {
                        source_text: restore_leading_eq(&spec.formula_text),
                    },
                });
            }
        }
    }

    DeferredInstalls {
        merges_by_sheet,
        tables_by_sheet,
        names_by_sheet,
        name_metadata,
        dropped_installs,
    }
}

/// A rect-denoting reference parsed from a defined name's `formula_text` (D4 §12
/// row 26): the sheet qualifier (display name) plus the one-based rect.
struct RectReference {
    sheet: String,
    top_row: u32,
    left_col: u32,
    bottom_row: u32,
    right_col: u32,
}

/// Parse a defined name's `formula_text` as a **rect-denoting** reference — a
/// single sheet-qualified A1 cell or range (`Sheet1!$A$1`, `Sheet1!$A$1:$B$2`,
/// `'My Sheet'!A1:C3`), the shape that installs as a static name (D4 §12 row 26).
///
/// Returns `None` for anything else (an unqualified ref, a multi-area reference,
/// a function call, an arithmetic expression) — those install as dynamic names
/// bound through §3. The parse is deliberately conservative: it requires a sheet
/// qualifier (a static name's target sheet must be known to author the rect) and
/// rejects any character that is not part of a `$`-decorated A1 cell/range, so an
/// expression like `Sheet1!A1+1` is NOT mistaken for a rect.
fn parse_rect_denoting_reference(formula_text: &str) -> Option<RectReference> {
    let text = formula_text
        .strip_prefix('=')
        .unwrap_or(formula_text)
        .trim();
    let (sheet_part, local) = split_sheet_qualifier(text)?;
    let sheet = normalize_sheet_qualifier(sheet_part)?;
    let (top_row, left_col, bottom_row, right_col) = parse_a1_rect(local)?;
    Some(RectReference {
        sheet,
        top_row,
        left_col,
        bottom_row,
        right_col,
    })
}

/// The first embedded sheet qualifier in a formula — the display name a dynamic
/// name's anchor sheet is *preferentially* derived from (a mere anchor preference;
/// a fully-qualified formula evaluates the same on any anchor). Scans for the
/// first `Name!` or `'Quoted Name'!` token and returns the un-quoted name; `None`
/// when the formula carries no sheet qualifier at all. Unlike a strict leading
/// parse, this reaches inside a function call (`SUM(Sheet1!A1:A2)` → `Sheet1`).
fn first_embedded_sheet_qualifier(formula_text: &str) -> Option<String> {
    let bytes = formula_text.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        match bytes[i] {
            b'\'' => {
                // Quoted sheet name: `'...'!`. Find the closing quote (Excel
                // doubles embedded quotes as `''`), then require a trailing `!`.
                let start = i + 1;
                let mut j = start;
                loop {
                    let close = formula_text[j..].find('\'').map(|k| j + k)?;
                    // A doubled quote `''` is an escape, not the close.
                    if formula_text.as_bytes().get(close + 1) == Some(&b'\'') {
                        j = close + 2;
                        continue;
                    }
                    if formula_text.as_bytes().get(close + 1) == Some(&b'!') {
                        return Some(formula_text[start..close].replace("''", "'"));
                    }
                    // A quoted token not followed by `!` is not a sheet qualifier;
                    // resume scanning past it.
                    i = close + 1;
                    break;
                }
            }
            b'!' => {
                // An unquoted `!`: the identifier immediately to its left is a
                // candidate sheet name. Walk back over sheet-name characters
                // (letters, digits, `_`, `.`) to the token start.
                let mut start = i;
                while start > 0 {
                    let ch = bytes[start - 1];
                    if ch.is_ascii_alphanumeric() || ch == b'_' || ch == b'.' {
                        start -= 1;
                    } else {
                        break;
                    }
                }
                if start < i {
                    return Some(formula_text[start..i].to_string());
                }
                i += 1;
            }
            _ => i += 1,
        }
    }
    None
}

/// Split a sheet-qualified reference `Sheet!local` at the qualifier boundary,
/// honoring a leading `'quoted sheet'!`. Returns `(sheet_qualifier, local_ref)`
/// where `sheet_qualifier` keeps its quotes (stripped by
/// [`normalize_sheet_qualifier`]). `None` when there is no `!` outside quotes.
///
/// A quoted name's closing quote is found by SKIPPING the doubled `''` escape
/// (Excel doubles an embedded apostrophe: `O'Brien` → `'O''Brien'`), so a sheet
/// name containing `'` round-trips through the save-side renderer
/// ([`render_absolute_name_formula_text`] / [`quote_sheet_name_if_needed`]) — the
/// same `''`-aware scan [`first_embedded_sheet_qualifier`] uses. Without this a
/// static name on an apostrophe-named sheet re-parses as a bare (non-rect) ref and
/// is silently reclassified static → dynamic on reload.
fn split_sheet_qualifier(text: &str) -> Option<(&str, &str)> {
    if text.starts_with('\'') {
        let bytes = text.as_bytes();
        let mut i = 1; // past the opening quote
        while i < bytes.len() {
            if bytes[i] == b'\'' {
                if bytes.get(i + 1) == Some(&b'\'') {
                    i += 2; // a doubled `''` escape, not the close — skip both
                    continue;
                }
                // The closing quote (index `i`): the qualifier is `text[..=i]`
                // (both quotes retained), the local ref follows the `!`.
                let local = text.get(i + 1..)?.strip_prefix('!')?;
                return Some((&text[..=i], local));
            }
            i += 1;
        }
        None
    } else {
        let bang = text.find('!')?;
        Some((&text[..bang], &text[bang + 1..]))
    }
}

/// Normalize a sheet qualifier to its display name: strip surrounding single
/// quotes and un-double any `''` escape. `None` for an empty qualifier.
fn normalize_sheet_qualifier(sheet_part: &str) -> Option<String> {
    let name = if let Some(inner) = sheet_part
        .strip_prefix('\'')
        .and_then(|s| s.strip_suffix('\''))
    {
        inner.replace("''", "'")
    } else {
        sheet_part.to_string()
    };
    (!name.is_empty()).then_some(name)
}

/// The Excel column label for a one-based column index (`1 → "A"`, `27 → "AA"`).
/// The inverse of the column-letter parse in [`parse_a1_cell`] (W062 R6.66 /
/// calc-5kqg.66 — the A1-range renderer the Tier-A collection projection needs).
#[must_use]
pub(crate) fn excel_column_label(mut col: u32) -> String {
    debug_assert!(col > 0, "column index is one-based");
    let mut chars = Vec::new();
    while col > 0 {
        let remainder = (col - 1) % 26;
        chars.push(char::from(b'A' + remainder as u8));
        col = (col - 1) / 26;
    }
    chars.reverse();
    chars.into_iter().collect()
}

/// A **relative** A1 range for a one-based rect (`"A1:B3"`; a one-cell rect
/// renders `"A1:A1"`, the shape [`parse_a1_rect`] round-trips and the merged/table
/// range strings use). W062 R6.66.
#[must_use]
pub(crate) fn render_a1_range(
    top_row: u32,
    left_col: u32,
    bottom_row: u32,
    right_col: u32,
) -> String {
    format!(
        "{}{}:{}{}",
        excel_column_label(left_col),
        top_row,
        excel_column_label(right_col),
        bottom_row,
    )
}

/// An **absolute**, sheet-qualified A1 reference for a static defined name's rect
/// (`"Sheet1!$A$1"` for a one-cell rect, `"Sheet1!$A$1:$B$3"` for a range) — the
/// exact shape [`parse_rect_denoting_reference`] round-trips (W062 R6.66). The
/// sheet display name is single-quoted (with embedded `'` doubled) when it is not
/// a bare identifier, matching [`normalize_sheet_qualifier`]'s un-quoting.
#[must_use]
pub(crate) fn render_absolute_name_formula_text(
    sheet_display: &str,
    top_row: u32,
    left_col: u32,
    bottom_row: u32,
    right_col: u32,
) -> String {
    let start = format!("${}${}", excel_column_label(left_col), top_row);
    let local = if top_row == bottom_row && left_col == right_col {
        start
    } else {
        format!("{start}:${}${}", excel_column_label(right_col), bottom_row)
    };
    format!("{}!{local}", quote_sheet_name_if_needed(sheet_display))
}

/// Quote a sheet display name for a formula qualifier when it is not a bare
/// identifier (letters/digits/`_`/`.`, not starting with a digit). Embedded single
/// quotes double (`'` → `''`), the inverse of [`normalize_sheet_qualifier`].
#[must_use]
pub(crate) fn quote_sheet_name_if_needed(name: &str) -> String {
    let bare = !name.is_empty()
        && !name.starts_with(|c: char| c.is_ascii_digit())
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.');
    if bare {
        name.to_string()
    } else {
        format!("'{}'", name.replace('\'', "''"))
    }
}

/// Parse a local (sheet-qualifier-free) A1 cell or range into a one-based
/// `(top_row, left_col, bottom_row, right_col)` rect, tolerating `$` anchors.
/// `None` when the text is not exactly a cell or `cell:cell` range (so an
/// expression, a multi-area union, or a whole-axis reference is rejected — the
/// caller then treats the name as dynamic).
fn parse_a1_rect(local: &str) -> Option<(u32, u32, u32, u32)> {
    let local = local.trim();
    let (start, end) = local.split_once(':').unwrap_or((local, local));
    let (start_row, start_col) = parse_a1_cell(start)?;
    let (end_row, end_col) = parse_a1_cell(end)?;
    Some((
        start_row.min(end_row),
        start_col.min(end_col),
        start_row.max(end_row),
        start_col.max(end_col),
    ))
}

/// Parse a single A1 cell (`$A$1`, `A1`, `$AA$10`) into one-based `(row, col)`,
/// tolerating `$` anchors. `None` for anything that is not exactly a column
/// run followed by a row number.
fn parse_a1_cell(cell: &str) -> Option<(u32, u32)> {
    let cell = cell.trim();
    let mut chars = cell.chars().peekable();
    // Optional column anchor.
    if chars.peek() == Some(&'$') {
        chars.next();
    }
    let mut col = 0u32;
    let mut saw_col = false;
    while let Some(&ch) = chars.peek() {
        if ch.is_ascii_alphabetic() {
            saw_col = true;
            col = col
                .checked_mul(26)?
                .checked_add(u32::from(ch.to_ascii_uppercase() as u8 - b'A') + 1)?;
            chars.next();
        } else {
            break;
        }
    }
    if !saw_col {
        return None;
    }
    // Optional row anchor.
    if chars.peek() == Some(&'$') {
        chars.next();
    }
    let mut row = 0u32;
    let mut saw_row = false;
    while let Some(&ch) = chars.peek() {
        if ch.is_ascii_digit() {
            saw_row = true;
            row = row
                .checked_mul(10)?
                .checked_add(u32::from(ch as u8 - b'0'))?;
            chars.next();
        } else {
            break;
        }
    }
    // Any trailing character means this is not a bare cell (e.g. `A1+1`).
    if !saw_row || chars.next().is_some() || row == 0 || col == 0 {
        return None;
    }
    Some((row, col))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consumer::{GridCellEntryOutcome, OxCalcDocumentContext, OxCalcTreeWorkspaceCreate};
    use crate::grid::authored::GridInputCell;
    use oxdoc_model::{
        AutoFilterSpec, CalcMode as DocCalcMode, CellChunk, CellFormatRun, CellPayload,
        CellRangeSpec, CommentNoticeKind, CommentNoticeSpec, ConditionalFormatRegion,
        DataValidationsSpec, DateSystem as DocDateSystem, DefinedNameMetadataSpec, DefinedNameSpec,
        DifferentialStyleSpec, DocumentEvent, DrawingFormControlsSpec, Extent, ExternalLinkSpec,
        FormulaTopology, GeometryCoupling, HyperlinksSpec, MergedCellRegions, OpaquePartKind,
        OpaquePartNotice, PackedCellAddr, SharedFormulaRegion, SharedStringEntry,
        SheetDimensionSpec, SheetRef, SheetReviewCommentsSpec, SheetViewState, SortStateSpec,
        StyleTableSpec, TableSpec, ThreadedCommentPeopleSpec, WorkbookHeader,
    };
    use oxfunc_core::value::{ExcelText, WorksheetErrorCode};

    fn a1() -> PackedCellAddr {
        PackedCellAddr::from_one_based(1, 1).unwrap()
    }

    /// A minimal valid literals-only stream: two sheets, various literal kinds.
    /// Sheet1 carries A1=number, B1=inline text, C1=bool; Sheet2 carries A1=an
    /// error literal (BIFF `#DIV/0!`) and B1=a shared-string literal.
    fn literals_only_stream() -> Vec<DocumentEvent> {
        vec![
            DocumentEvent::WorkbookHeader(WorkbookHeader::new(
                DocDateSystem::Date1904,
                DocCalcMode::Manual,
            )),
            DocumentEvent::StringTable(vec![SharedStringEntry {
                text: "hello-shared".to_string(),
            }]),
            DocumentEvent::StyleTable(StyleTableSpec::minimal()),
            DocumentEvent::SheetBegin(SheetRef {
                sheet_id: 1,
                name: "Alpha".to_string(),
            }),
            DocumentEvent::CellChunk(CellChunk {
                row_band: 0,
                cells: vec![
                    (
                        PackedCellAddr::from_one_based(1, 1).unwrap(),
                        CellPayload::Number(42.0),
                    ),
                    (
                        PackedCellAddr::from_one_based(1, 2).unwrap(),
                        CellPayload::InlineText("inline!".to_string()),
                    ),
                    (
                        PackedCellAddr::from_one_based(1, 3).unwrap(),
                        CellPayload::Bool(true),
                    ),
                ],
            }),
            DocumentEvent::SheetEnd { sheet_id: 1 },
            DocumentEvent::SheetBegin(SheetRef {
                sheet_id: 2,
                name: "Beta".to_string(),
            }),
            DocumentEvent::CellChunk(CellChunk {
                row_band: 0,
                cells: vec![
                    (
                        PackedCellAddr::from_one_based(1, 1).unwrap(),
                        // BIFF 0x07 → #DIV/0!.
                        CellPayload::Error(0x07),
                    ),
                    (
                        PackedCellAddr::from_one_based(1, 2).unwrap(),
                        CellPayload::SharedText(0),
                    ),
                ],
            }),
            DocumentEvent::SheetEnd { sheet_id: 2 },
        ]
    }

    /// A synthetic stream carrying exactly one instance of **every** one of the
    /// 29 `DocumentEvent` variants, in an order the stream validator accepts:
    /// prelude (header/strings/styles/diff-styles), then the workbook-scoped
    /// events, then one sheet whose sheet-scoped-before-cells events precede its
    /// cell chunk. This drives the whole tier taxonomy through the sink.
    fn all_variant_stream() -> Vec<DocumentEvent> {
        vec![
            // -- prelude (rows 1-4) --
            DocumentEvent::WorkbookHeader(WorkbookHeader::new(
                DocDateSystem::Date1900,
                DocCalcMode::Automatic,
            )),
            DocumentEvent::StringTable(vec![SharedStringEntry {
                text: "s".to_string(),
            }]),
            DocumentEvent::StyleTable(StyleTableSpec::minimal()),
            DocumentEvent::DifferentialStyleTable(vec![DifferentialStyleSpec {
                dxf_id: 0,
                ..DifferentialStyleSpec::default()
            }]),
            // -- workbook-scoped events (must precede any sheet content) --
            DocumentEvent::DefinedName(DefinedNameSpec {
                name: "N".to_string(),
                formula_text: "Sheet1!$A$1".to_string(),
                scope_sheet_id: None,
                metadata: DefinedNameMetadataSpec::default(),
            }),
            DocumentEvent::ThreadedCommentPeople(ThreadedCommentPeopleSpec {
                people: Vec::new(),
                notices: Vec::new(),
                raw_attrs: Vec::new(),
            }),
            DocumentEvent::ExternalLink(ExternalLinkSpec {
                target: "book2.xlsx".to_string(),
            }),
            DocumentEvent::CalcChainHint(vec![a1()]),
            DocumentEvent::OpaquePartNotice(OpaquePartNotice {
                part_name: "xl/vbaProject.bin".to_string(),
                kind: OpaquePartKind::VbaProject,
                geometry_coupling: GeometryCoupling::None,
            }),
            // -- one sheet --
            DocumentEvent::SheetBegin(SheetRef {
                sheet_id: 1,
                name: "Sheet1".to_string(),
            }),
            // sheet-scoped, before any cell chunk
            DocumentEvent::SheetDimension(SheetDimensionSpec {
                sheet_id: 1,
                ref_text: "A1".to_string(),
                range: None,
            }),
            DocumentEvent::ColumnProps(Vec::new()),
            DocumentEvent::RowProps(Vec::new()),
            DocumentEvent::MergedCellRegions(MergedCellRegions {
                sheet_id: 1,
                ranges: Vec::new(),
                raw_refs: Vec::new(),
            }),
            DocumentEvent::SheetViewState(SheetViewState {
                sheet_id: 1,
                workbook_view_id: None,
                view: None,
                show_grid_lines: None,
                show_row_col_headers: None,
                right_to_left: None,
                tab_selected: None,
                zoom_scale: None,
                top_left_cell: None,
                pane: None,
                selections: Vec::new(),
                raw_attrs: Vec::new(),
                raw_children: Vec::new(),
            }),
            DocumentEvent::Hyperlinks(HyperlinksSpec {
                sheet_id: 1,
                links: Vec::new(),
            }),
            DocumentEvent::DataValidations(DataValidationsSpec {
                sheet_id: 1,
                disable_prompts: None,
                x_window: None,
                y_window: None,
                regions: Vec::new(),
                raw_attrs: Vec::new(),
            }),
            DocumentEvent::AutoFilter(AutoFilterSpec {
                sheet_id: 1,
                ref_text: "A1:A1".to_string(),
                range: None,
                columns: Vec::new(),
                raw_attrs: Vec::new(),
                raw_children: Vec::new(),
            }),
            DocumentEvent::SortState(SortStateSpec {
                sheet_id: 1,
                ref_text: None,
                range: None,
                case_sensitive: None,
                column_sort: None,
                sort_method: None,
                conditions: Vec::new(),
                raw_attrs: Vec::new(),
                raw_children: Vec::new(),
            }),
            DocumentEvent::CommentNotice(CommentNoticeSpec {
                sheet_id: 1,
                reference: None,
                kind: CommentNoticeKind::Note,
                author: None,
                text: None,
                source_id: None,
                unsupported_fragments: Vec::new(),
                raw_attrs: Vec::new(),
            }),
            DocumentEvent::SheetReviewComments(SheetReviewCommentsSpec {
                sheet_id: 1,
                legacy_notes: Vec::new(),
                threaded_comments: Vec::new(),
                placeholders: Vec::new(),
                vml_links: Vec::new(),
                notices: Vec::new(),
                raw_attrs: Vec::new(),
            }),
            DocumentEvent::DrawingFormControls(DrawingFormControlsSpec {
                sheet_id: 1,
                drawing_layer_id: None,
                objects: Vec::new(),
                controls: Vec::new(),
                notices: Vec::new(),
            }),
            DocumentEvent::CellFormatRuns(vec![CellFormatRun {
                row: 1,
                start_col: 1,
                len: 1,
                style_id: 0,
            }]),
            DocumentEvent::ConditionalFormatRegion(ConditionalFormatRegion {
                sheet_id: 1,
                sqref: "A1".to_string(),
                ranges: Vec::new(),
                pivot: false,
                rules: Vec::new(),
            }),
            DocumentEvent::FormulaTopology(FormulaTopology {
                sheet_id: 1,
                records: Vec::new(),
                unsupported_fragments: Vec::new(),
            }),
            DocumentEvent::SharedFormulaRegion(SharedFormulaRegion {
                region_id: 0,
                anchor: a1(),
                extent: Extent { rows: 1, cols: 1 },
                r1c1_text: "A1+1".to_string(),
            }),
            DocumentEvent::TableOverlay(TableSpec {
                name: "T".to_string(),
                sheet_id: 1,
                range: "A1:A1".to_string(),
            }),
            DocumentEvent::CellChunk(CellChunk {
                row_band: 0,
                cells: vec![(a1(), CellPayload::Number(1.0))],
            }),
            DocumentEvent::SheetEnd { sheet_id: 1 },
        ]
    }

    fn workbook_context() -> (OxCalcDocumentContext, OxCalcTreeWorkspaceId) {
        let mut context = OxCalcDocumentContext::default();
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new("workbook:ingest").as_workbook())
            .unwrap();
        (context, workspace_id)
    }

    // ---- Acceptance 1: literals-only workbook, one revision, readout matches --

    #[test]
    fn literals_only_workbook_loads_in_one_revision_and_reads_back() {
        let (mut context, workspace_id) = workbook_context();

        // The graph starts with exactly the creation revision.
        let before = context.workspace_view(&workspace_id).unwrap();
        let graph_before = before.retained_workspace_revision_count;
        let creation_revision = before.workspace_revision_id.clone();

        let report =
            load_workbook_events(&mut context, &workspace_id, &literals_only_stream()).unwrap();

        assert_eq!(report.sheets, 2, "two sheets created");
        assert_eq!(report.cells, 5, "five literal cells folded in");
        // The stream is Manual-mode, so the load takes the Manual render-from-cache
        // path (D4 §6): no engine evaluation, zero recalc passes.
        assert_eq!(report.recalc_path, LoadRecalcPath::Manual);
        assert_eq!(
            report.engine_recalcs_at_load, 0,
            "a Manual-mode literals-only load runs zero engine passes"
        );

        // Exactly ONE load transaction: the graph grew by one entry, and the
        // current revision's parent is the creation revision (a single mint over
        // the empty workbook, D4 §9).
        let after = context.workspace_view(&workspace_id).unwrap();
        assert_eq!(
            after.retained_workspace_revision_count,
            graph_before + 1,
            "load must mint exactly one revision"
        );
        assert_eq!(
            after.workspace_revision_parent_id.as_ref(),
            Some(&creation_revision),
            "the load revision's parent is the creation revision"
        );
        assert_ne!(
            after.workspace_revision_id, creation_revision,
            "the load advanced the revision"
        );

        // The authored readout matches the inputs exactly, per sheet.
        let sheet_nodes = context.sheets(&workspace_id).unwrap();
        assert_eq!(sheet_nodes.len(), 2);
        let alpha_node = sheet_nodes[0].node_id;
        let beta_node = sheet_nodes[1].node_id;

        let alpha = context
            .grid_authored_view(&workspace_id, alpha_node, None)
            .unwrap()
            .unwrap();
        let alpha_at = |row: u32, col: u32| {
            alpha
                .iter()
                .find(|cell| cell.address.row == row && cell.address.col == col)
                .and_then(|cell| cell.literal.clone())
        };
        assert_eq!(alpha_at(1, 1), Some(CalcValue::number(42.0)));
        assert_eq!(
            alpha_at(1, 2),
            Some(CalcValue::text(ExcelText::from_interop_assignment(
                "inline!"
            )))
        );
        assert_eq!(alpha_at(1, 3), Some(CalcValue::logical(true)));

        let beta = context
            .grid_authored_view(&workspace_id, beta_node, None)
            .unwrap()
            .unwrap();
        let beta_at = |row: u32, col: u32| {
            beta.iter()
                .find(|cell| cell.address.row == row && cell.address.col == col)
                .and_then(|cell| cell.literal.clone())
        };
        assert_eq!(
            beta_at(1, 1),
            Some(CalcValue::error(WorksheetErrorCode::Div0))
        );
        assert_eq!(
            beta_at(1, 2),
            Some(CalcValue::text(ExcelText::from_interop_assignment(
                "hello-shared"
            )))
        );

        // Differential clean: `grid_view` runs the reference + optimized engines
        // and reports any disagreement. The ingested workbook must be clean on
        // both sheets (contract C15 — literals carry no engine disagreement).
        for node in [alpha_node, beta_node] {
            let view = context.grid_view(&workspace_id, node).unwrap().unwrap();
            assert!(
                view.differential_mismatches.is_empty(),
                "ingested sheet must be differential-clean, got {:?}",
                view.differential_mismatches
            );
        }

        // Settings from the header were consumed (1904 date system + Manual).
        let settings = context.workbook_calc_settings(&workspace_id).unwrap();
        assert_eq!(settings.date_system, DateSystem::Excel1904);
        assert_eq!(settings.calc_mode, CalcMode::Manual);
    }

    /// W062 R6.69 (gap #1 consumer): a workbook header carrying NON-default
    /// iterative-calc settings (`<calcPr iterate iterateCount iterateDelta>`)
    /// must reach the engine as `WorkbookCalcSettings.iteration` — not silently
    /// collapse to the Excel default. Fail-until-fixed: before the ingest wired
    /// `prelude.header.iterative_calc`, the `workbook` handler filled iteration
    /// from `WorkbookCalcSettings::default()`, so this asserted the engine
    /// default (off / 100 / 0.001) instead of the file's values.
    #[test]
    fn workbook_header_iteration_settings_reach_the_engine() {
        let (mut context, workspace_id) = workbook_context();

        // A minimal one-sheet stream whose header carries non-default iteration.
        let stream = vec![
            DocumentEvent::WorkbookHeader(
                WorkbookHeader::new(DocDateSystem::Date1900, DocCalcMode::Automatic)
                    .with_iterative_calc(oxdoc_model::IterativeCalcSettings {
                        enabled: true,
                        max_iterations: 250,
                        max_change: 1e-6,
                    }),
            ),
            DocumentEvent::StringTable(Vec::new()),
            DocumentEvent::StyleTable(StyleTableSpec::minimal()),
            DocumentEvent::SheetBegin(SheetRef {
                sheet_id: 1,
                name: "Sheet1".to_string(),
            }),
            DocumentEvent::CellChunk(CellChunk {
                row_band: 0,
                cells: vec![(
                    PackedCellAddr::from_one_based(1, 1).unwrap(),
                    CellPayload::Number(1.0),
                )],
            }),
            DocumentEvent::SheetEnd { sheet_id: 1 },
        ];

        load_workbook_events(&mut context, &workspace_id, &stream).unwrap();

        // The file's iteration settings, not the engine default (off/100/0.001).
        let settings = context.workbook_calc_settings(&workspace_id).unwrap();
        assert!(
            settings.iteration.enabled,
            "iterative calc enabled must survive ingest, got {:?}",
            settings.iteration
        );
        assert_eq!(
            settings.iteration.max_iterations, 250,
            "max_iterations must survive ingest"
        );
        assert!(
            (settings.iteration.max_change - 1e-6).abs() < f64::EPSILON,
            "max_change must survive ingest, got {}",
            settings.iteration.max_change
        );
    }

    // ---- Acceptance 2: 29/29 accounting + no-silent-loss invariant -----------

    #[test]
    fn all_variant_stream_accounts_for_every_variant() {
        let (mut context, workspace_id) = workbook_context();

        // Drive the sink directly so we can read `observed()` (the invariant's
        // input) and then commit for the report.
        let mut sink = OxCalcWorkbookIngestSink::new();
        oxdoc_model::drive_oxcalc_ingest(&all_variant_stream(), &mut sink).unwrap();

        // Every one of the 29 variants was observed exactly once (the stream
        // carries one of each).
        let observed = sink.observed().to_vec();
        assert_eq!(
            observed.len(),
            29,
            "all 29 DocumentEvent variants observed, got {}: {observed:?}",
            observed.len()
        );

        let report = sink.commit_into(&mut context, &workspace_id).unwrap();

        // The no-silent-loss invariant: every observed variant has a ledger
        // disposition. This is the whole point of the honesty regime.
        assert_eq!(
            report.accounts_for_all_variants(&observed),
            Ok(()),
            "no observed variant may be a silent path"
        );

        // The ledger has one row per observed variant: 29 rows, in table order.
        assert_eq!(report.ledger.len(), 29, "one ledger row per variant");

        // Exactly one variant is Tier X (CalcChainHint), and it is excluded.
        let x_rows: Vec<_> = report
            .ledger
            .iter()
            .filter(|row| row.tier == IngestTier::X)
            .collect();
        assert_eq!(x_rows.len(), 1, "exactly one Tier-X exclusion (D4 §12)");
        assert_eq!(x_rows[0].variant, DocumentVariantTag::CalcChainHint);
        assert_eq!(x_rows[0].disposition, "excluded-engine-derives-order");
    }

    /// Mutation sentinel for the ledger invariant. If any single variant's
    /// disposition were dropped from the report, `accounts_for_all_variants`
    /// against the full observed set must fail. This proves the invariant test
    /// above is not vacuous: a dropped variant IS caught.
    #[test]
    fn ledger_invariant_catches_a_dropped_variant() {
        let (mut context, workspace_id) = workbook_context();
        let mut sink = OxCalcWorkbookIngestSink::new();
        oxdoc_model::drive_oxcalc_ingest(&all_variant_stream(), &mut sink).unwrap();
        let observed = sink.observed().to_vec();
        let mut report = sink.commit_into(&mut context, &workspace_id).unwrap();

        // Simulate a silent drop: remove one variant's ledger row. The invariant
        // must now fail, naming that exact variant.
        let dropped = DocumentVariantTag::AutoFilter;
        report.ledger.retain(|row| row.variant != dropped);
        assert_eq!(
            report.accounts_for_all_variants(&observed),
            Err(dropped),
            "dropping a variant's ledger row must fail the invariant"
        );
    }

    // ---- Acceptance 3: BIFF error-code mapping -------------------------------

    #[test]
    fn biff_error_codes_map_to_typed_errors() {
        let cases = [
            (0x00u8, WorksheetErrorCode::Null),
            (0x07, WorksheetErrorCode::Div0),
            (0x0F, WorksheetErrorCode::Value),
            (0x17, WorksheetErrorCode::Ref),
            (0x1D, WorksheetErrorCode::Name),
            (0x24, WorksheetErrorCode::Num),
            (0x2A, WorksheetErrorCode::NA),
        ];
        for (byte, expected) in cases {
            let (mapped, known) = map_biff_error_code(byte);
            assert!(known, "0x{byte:02X} is a known BIFF code");
            assert_eq!(mapped, expected, "0x{byte:02X} maps to {expected:?}");
        }
    }

    #[test]
    fn unknown_biff_error_code_falls_back_to_value_and_is_flagged() {
        // 0xFF (255 — oxdoc-xlsx's unknown sentinel) has no classic BIFF byte.
        let (mapped, known) = map_biff_error_code(0xFF);
        assert_eq!(mapped, WorksheetErrorCode::Value, "unknown → #VALUE!");
        assert!(!known, "unknown byte is flagged, not silently guessed");
    }

    #[test]
    fn unknown_error_literal_loads_as_value_error() {
        // A cell carrying an unknown error byte ingests as #VALUE! (D4 §10 — the
        // published value). The raw byte is additionally retained in the Tier-B
        // store (R6.4) so save writes it back verbatim (asserted below).
        let (mut context, workspace_id) = workbook_context();
        let stream = vec![
            DocumentEvent::WorkbookHeader(WorkbookHeader::new(
                DocDateSystem::Date1900,
                DocCalcMode::Automatic,
            )),
            DocumentEvent::StringTable(Vec::new()),
            DocumentEvent::StyleTable(StyleTableSpec::minimal()),
            DocumentEvent::SheetBegin(SheetRef {
                sheet_id: 1,
                name: "S".to_string(),
            }),
            DocumentEvent::CellChunk(CellChunk {
                row_band: 0,
                cells: vec![(a1(), CellPayload::Error(0xFF))],
            }),
            DocumentEvent::SheetEnd { sheet_id: 1 },
        ];
        load_workbook_events(&mut context, &workspace_id, &stream).unwrap();
        let node = context.sheets(&workspace_id).unwrap()[0].node_id;
        let readout = context
            .grid_authored_view(&workspace_id, node, None)
            .unwrap()
            .unwrap();
        let value = readout
            .iter()
            .find(|cell| cell.address.row == 1 && cell.address.col == 1)
            .and_then(|cell| cell.literal.clone());
        assert_eq!(value, Some(CalcValue::error(WorksheetErrorCode::Value)));
        // The raw byte survives in the store for a verbatim round-trip.
        let facts = context.ingested_document_facts(&workspace_id).unwrap();
        assert_eq!(facts.unknown_error_bytes.len(), 1);
        assert_eq!(facts.unknown_error_bytes[0].raw_byte, 0xFF);
    }

    // ---- Acceptance 4: load-fail vs degrade boundary -------------------------

    #[test]
    fn case_fold_duplicate_sheet_name_fails_the_load() {
        // Two sheets whose names differ only by case fail the single-transaction
        // load with a typed Err (D1 validate() — Sheet-sibling case-fold
        // uniqueness). A partial load never lands.
        let (mut context, workspace_id) = workbook_context();
        let stream = vec![
            DocumentEvent::WorkbookHeader(WorkbookHeader::new(
                DocDateSystem::Date1900,
                DocCalcMode::Automatic,
            )),
            DocumentEvent::StringTable(Vec::new()),
            DocumentEvent::StyleTable(StyleTableSpec::minimal()),
            DocumentEvent::SheetBegin(SheetRef {
                sheet_id: 1,
                name: "Data".to_string(),
            }),
            DocumentEvent::SheetEnd { sheet_id: 1 },
            DocumentEvent::SheetBegin(SheetRef {
                sheet_id: 2,
                name: "DATA".to_string(),
            }),
            DocumentEvent::SheetEnd { sheet_id: 2 },
        ];

        let before = context.workspace_view(&workspace_id).unwrap();
        let err = load_workbook_events(&mut context, &workspace_id, &stream).unwrap_err();
        match err {
            OxCalcWorkbookIngestError::Commit(OxCalcDocumentError::Structural(
                crate::structural::StructuralError::DuplicateSheetName { .. },
            )) => {}
            other => panic!("expected a DuplicateSheetName structural error, got {other:?}"),
        }

        // No partial load landed: the revision graph is unchanged (the mint only
        // happens on success, after all inserts validate).
        let after = context.workspace_view(&workspace_id).unwrap();
        assert_eq!(
            after.retained_workspace_revision_count, before.retained_workspace_revision_count,
            "a failed load must not mint a revision"
        );
    }

    // ==== R6.2: formula ingest ================================================

    use crate::workbook_settings::PublishedValueProvenance;
    use oxdoc_model::{
        ArrayFormulaSpec, CachedValueProvenance, DataTableFormulaSpec, FormulaCachedValueState,
        FormulaRecord, FormulaRecordAttributes, FormulaRecordKind,
    };

    fn addr(row: u32, col: u32) -> PackedCellAddr {
        PackedCellAddr::from_one_based(row, col).unwrap()
    }

    /// The prelude every formula fixture opens with (Automatic mode, as W011).
    /// Under R6.5's load-recalc policy (D4 §6), an Automatic load issues the
    /// open-recalc: bound formulas publish engine `Calculated` values at load.
    fn formula_prelude() -> Vec<DocumentEvent> {
        vec![
            DocumentEvent::WorkbookHeader(WorkbookHeader::new(
                DocDateSystem::Date1900,
                DocCalcMode::Automatic,
            )),
            DocumentEvent::StringTable(Vec::new()),
            DocumentEvent::StyleTable(StyleTableSpec::minimal()),
        ]
    }

    /// The `Manual`-mode prelude (D4 §6): a Manual load runs NO engine evaluation
    /// — bound formulas render their `FileCached` caches until an explicit F9
    /// (`recalculate_workbook`). Fixtures exercising the cache-then-F9 lifecycle
    /// (and the Manual-zero-eval perf-counter proof) open with this.
    fn manual_formula_prelude() -> Vec<DocumentEvent> {
        vec![
            DocumentEvent::WorkbookHeader(WorkbookHeader::new(
                DocDateSystem::Date1900,
                DocCalcMode::Manual,
            )),
            DocumentEvent::StringTable(Vec::new()),
            DocumentEvent::StyleTable(StyleTableSpec::minimal()),
        ]
    }

    /// The value of the single grid node's published cell at `(row, col)`.
    fn published_value(
        context: &OxCalcDocumentContext,
        workspace_id: &OxCalcTreeWorkspaceId,
        row: u32,
        col: u32,
    ) -> Option<(CalcValue, PublishedValueProvenance)> {
        let node = context.sheets(workspace_id).unwrap()[0].node_id;
        let view = context.grid_view(workspace_id, node).unwrap().unwrap();
        view.cells
            .iter()
            .find(|cell| cell.address.row == row && cell.address.col == col)
            .map(|cell| (cell.value.clone(), cell.provenance))
    }

    /// An [`ExcelGridCellAddress`] on the single ingested sheet, with the
    /// workbook/sheet tokens the ingest builder derives (`book:{workspace}` /
    /// `sheet:{node_id}`) — so a host verb like `enter_grid_cell` targets the
    /// right grid.
    fn ingested_address(
        context: &OxCalcDocumentContext,
        workspace_id: &OxCalcTreeWorkspaceId,
        row: u32,
        col: u32,
    ) -> ExcelGridCellAddress {
        let node = context.sheets(workspace_id).unwrap()[0].node_id;
        ExcelGridCellAddress::new(
            format!("book:{}", workspace_id.as_str()),
            format!("sheet:{}", node.0),
            row,
            col,
        )
    }

    /// The authored source text of the single grid node's cell at `(row, col)`.
    fn authored_source_text(
        context: &OxCalcDocumentContext,
        workspace_id: &OxCalcTreeWorkspaceId,
        row: u32,
        col: u32,
    ) -> Option<String> {
        let node = context.sheets(workspace_id).unwrap()[0].node_id;
        let readout = context
            .grid_authored_view(workspace_id, node, None)
            .unwrap()
            .unwrap();
        readout
            .iter()
            .find(|cell| cell.address.row == row && cell.address.col == col)
            .and_then(|cell| cell.source_text.clone())
    }

    // ---- Acceptance: the W011 fixture ----------------------------------------

    /// The canonical W011 two-cell sheet: `Sheet1!A1 = 7` (literal),
    /// `B1 = =A1*3` (formula) with a FileCached cache of 21.
    fn w011_sheet_events() -> Vec<DocumentEvent> {
        vec![
            DocumentEvent::SheetBegin(SheetRef {
                sheet_id: 1,
                name: "Sheet1".to_string(),
            }),
            DocumentEvent::CellChunk(CellChunk {
                row_band: 0,
                cells: vec![
                    (addr(1, 1), CellPayload::Number(7.0)),
                    (
                        addr(1, 2),
                        CellPayload::Formula {
                            region: None,
                            text: Some("A1*3".to_string()),
                            cached: Some(Box::new(CellPayload::Number(21.0))),
                        },
                    ),
                ],
            }),
            DocumentEvent::SheetEnd { sheet_id: 1 },
        ]
    }

    /// W011 under **Manual** mode (D4 §6): the load runs NO engine evaluation, so
    /// B1 renders its FileCached 21 (pre-engine), and an explicit
    /// `recalculate_workbook` (F9) replaces the cache with the engine's own 21
    /// (`Calculated`). The Manual-zero-eval acceptance is proven by the perf
    /// counter (`engine_recalcs_at_load == 0`) and `recalc_epoch == 0` pre-F9.
    #[test]
    fn w011_manual_load_renders_filecached_zero_eval_then_recalcs_by_engine() {
        let (mut context, workspace_id) = workbook_context();
        let mut stream = manual_formula_prelude();
        stream.extend(w011_sheet_events());

        let report = load_workbook_events(&mut context, &workspace_id, &stream).unwrap();
        assert_eq!(report.cells, 1, "one literal (A1)");
        assert_eq!(report.formulas_bound, 1, "B1 bound through the single mint");
        assert!(report.bind_degradations.is_empty(), "B1 is a valid formula");
        assert_eq!(
            report.recalc_path,
            LoadRecalcPath::Manual,
            "a Manual-mode load takes the Manual render-from-cache path"
        );
        // THE PERF-COUNTER PROOF of the Manual-zero-eval acceptance: the load ran
        // ZERO engine passes. This is the counter evidence, not a side-channel.
        assert_eq!(
            report.engine_recalcs_at_load, 0,
            "a Manual-mode load runs ZERO engine recalc passes (the perf-counter proof)"
        );

        // Pre-F9: BOTH cells render their FileCached publication (the literal 7 and
        // the formula's cache 21), tagged FileCached — no engine value exists yet.
        assert_eq!(
            published_value(&context, &workspace_id, 1, 2),
            Some((
                CalcValue::number(21.0),
                PublishedValueProvenance::FileCached
            )),
            "B1 renders its FileCached cache pre-recalc (Manual, no engine pass)"
        );

        // The authored formula text round-trips (leading `=` restored).
        assert_eq!(
            authored_source_text(&context, &workspace_id, 1, 2).as_deref(),
            Some("=A1*3"),
        );

        // Explicit recalc (F9): the seeded formula cell drains and B1 is replaced
        // by the engine's own value — 21, now Calculated (the FileCached cache is
        // gone). This is the FIRST engine pass over the workbook.
        let outcome = context.recalculate_workbook(&workspace_id).unwrap();
        assert!(outcome.drained_any(), "F9 drains the seeded formula cell");
        assert!(
            outcome.total_cells_evaluated() > 0,
            "the F9 recalc evaluated cells (counter evidence a real recalc ran)"
        );
        let (value, provenance) = published_value(&context, &workspace_id, 1, 2).unwrap();
        assert_eq!(
            value,
            CalcValue::number(21.0),
            "B1 == A1*3 == 21 by the engine"
        );
        assert!(
            matches!(provenance, PublishedValueProvenance::Calculated { .. }),
            "post-recalc B1 is engine-Calculated, not FileCached"
        );

        // The post-F9 differential is clean (the drain ran both engine lanes).
        let node = context.sheets(&workspace_id).unwrap()[0].node_id;
        let view = context.grid_view(&workspace_id, node).unwrap().unwrap();
        assert!(
            view.differential_mismatches.is_empty(),
            "post-F9 formula sheet is differential-clean, got {:?}",
            view.differential_mismatches
        );
    }

    /// W011 under **Automatic** mode (D4 §6/§9): the load issues EXACTLY ONE
    /// open-recalc (Excel's open-recalc), so A1 and B1 publish engine
    /// `Calculated` values immediately — no FileCached-until-F9. The load is
    /// differential-clean, and a subsequent F9 is a no-op (the workbook is fully
    /// recalculated).
    #[test]
    fn w011_automatic_load_open_recalcs_to_calculated_one_pass() {
        let (mut context, workspace_id) = workbook_context();
        let mut stream = formula_prelude();
        stream.extend(w011_sheet_events());

        let report = load_workbook_events(&mut context, &workspace_id, &stream).unwrap();
        assert_eq!(report.cells, 1, "one literal (A1)");
        assert_eq!(report.formulas_bound, 1, "B1 bound through the single mint");
        assert!(report.bind_degradations.is_empty(), "B1 is a valid formula");
        assert_eq!(
            report.recalc_path,
            LoadRecalcPath::Automatic,
            "an Automatic-mode load takes the open-recalc path"
        );
        // Exactly ONE engine pass ran (the single-sheet open-recalc).
        assert_eq!(
            report.engine_recalcs_at_load, 1,
            "an Automatic single-sheet load runs exactly ONE open-recalc pass"
        );

        // Post-load: B1 is the engine's own value (21), tagged Calculated — the
        // open-recalc replaced the FileCached cache at load. A1 (the literal) is
        // likewise engine-Calculated.
        let (b1_value, b1_provenance) = published_value(&context, &workspace_id, 1, 2).unwrap();
        assert_eq!(
            b1_value,
            CalcValue::number(21.0),
            "B1 == A1*3 == 21 by the engine"
        );
        assert!(
            matches!(b1_provenance, PublishedValueProvenance::Calculated { .. }),
            "post-open-recalc B1 is engine-Calculated, not FileCached"
        );
        assert!(
            matches!(
                published_value(&context, &workspace_id, 1, 1).map(|(_, p)| p),
                Some(PublishedValueProvenance::Calculated { .. })
            ),
            "the A1 literal is engine-Calculated by the open-recalc"
        );

        // The load is differential-clean (the open-recalc ran both engine lanes).
        let node = context.sheets(&workspace_id).unwrap()[0].node_id;
        let view = context.grid_view(&workspace_id, node).unwrap().unwrap();
        assert!(
            view.differential_mismatches.is_empty(),
            "Automatic-load formula sheet is differential-clean, got {:?}",
            view.differential_mismatches
        );

        // The authored formula text still round-trips (open-recalc never rewrites
        // authored truth).
        assert_eq!(
            authored_source_text(&context, &workspace_id, 1, 2).as_deref(),
            Some("=A1*3"),
        );

        // A subsequent F9 is a genuine no-op: the workbook was already fully
        // recalculated at load, so nothing is undrained (counter == 0).
        let outcome = context.recalculate_workbook(&workspace_id).unwrap();
        assert!(
            !outcome.drained_any(),
            "post-Automatic-load F9 finds nothing undrained — a no-op"
        );
        assert_eq!(outcome.total_cells_evaluated(), 0, "no cells re-evaluated");
    }

    // ---- Acceptance: shared-formula region -----------------------------------

    /// A `SharedFormulaRegion` (R1C1 template `=RC[-1]*3` over B1:B3) expands
    /// per-cell. Column A carries 1/2/3; each B cell is its A-neighbour times 3.
    /// The expansion is differential-proven (the reference and optimized engines
    /// agree on every expanded cell) and each B value is the correct relative
    /// adjustment.
    #[test]
    fn shared_formula_region_expands_per_cell_differential_proven() {
        let (mut context, workspace_id) = workbook_context();
        let mut stream = formula_prelude();
        stream.extend([
            DocumentEvent::SheetBegin(SheetRef {
                sheet_id: 1,
                name: "Sheet1".to_string(),
            }),
            // Emit the shared region before the cell chunk. (The stream validator
            // imposes NO cell↔region ordering for `SharedFormulaRegion` — unlike
            // `FormulaTopology` — so the ingest is order-robust; this is just the
            // natural authoring order.)
            DocumentEvent::SharedFormulaRegion(SharedFormulaRegion {
                region_id: 0,
                anchor: addr(1, 2),
                extent: Extent { rows: 3, cols: 1 },
                r1c1_text: "RC[-1]*3".to_string(),
            }),
            DocumentEvent::CellChunk(CellChunk {
                row_band: 0,
                cells: vec![
                    (addr(1, 1), CellPayload::Number(1.0)),
                    (addr(2, 1), CellPayload::Number(2.0)),
                    (addr(3, 1), CellPayload::Number(3.0)),
                ],
            }),
            DocumentEvent::SheetEnd { sheet_id: 1 },
        ]);

        let report = load_workbook_events(&mut context, &workspace_id, &stream).unwrap();
        assert_eq!(report.cells, 3, "three column-A literals");
        assert!(report.bind_degradations.is_empty());

        // The region is recalc-ready; F9 drains and publishes the expansion.
        // (Column A literals recalc to themselves; the region cells evaluate.)
        context.recalculate_workbook(&workspace_id).unwrap();

        // Each B cell is its A-neighbour * 3 — the relative-adjusted expansion,
        // derived independently here: B1=1*3, B2=2*3, B3=3*3.
        for (row, expected) in [(1u32, 3.0), (2, 6.0), (3, 9.0)] {
            let (value, provenance) = published_value(&context, &workspace_id, row, 2)
                .unwrap_or_else(|| panic!("B{row} published"));
            assert_eq!(
                value,
                CalcValue::number(expected),
                "B{row} = A{row}*3 = {expected} (relative-adjusted)"
            );
            assert!(
                matches!(provenance, PublishedValueProvenance::Calculated { .. }),
                "B{row} is engine-calculated after recalc"
            );
        }

        // Differential-proven: the reference and optimized engines agree on the
        // whole expanded region.
        let node = context.sheets(&workspace_id).unwrap()[0].node_id;
        let view = context.grid_view(&workspace_id, node).unwrap().unwrap();
        assert!(
            view.differential_mismatches.is_empty(),
            "shared-region expansion is differential-clean, got {:?}",
            view.differential_mismatches
        );
    }

    // ---- Acceptance: Array (legacy CSE) topology routing ---------------------

    /// A `FormulaTopology` `Array` record: the cells ingest as normal formulas
    /// AND the array rect claims an inert `Cse` overlay (no legacy-CSE eval
    /// semantics). The overlay claim is a load fact.
    #[test]
    fn array_topology_binds_cells_and_claims_inert_cse_overlay() {
        let (mut context, workspace_id) = workbook_context();
        let array_range = CellRangeSpec {
            text: "A1:A2".to_string(),
            start: addr(1, 1),
            end: addr(2, 1),
        };
        let mut array_attrs = FormulaRecordAttributes::normal();
        array_attrs.formula_type = Some("array".to_string());
        let mut stream = formula_prelude();
        stream.extend([
            DocumentEvent::SheetBegin(SheetRef {
                sheet_id: 1,
                name: "Sheet1".to_string(),
            }),
            DocumentEvent::FormulaTopology(FormulaTopology {
                sheet_id: 1,
                records: vec![FormulaRecord {
                    sheet_id: 1,
                    address: addr(1, 1),
                    kind: FormulaRecordKind::Array(ArrayFormulaSpec {
                        range: Some(array_range),
                        always_calculate: None,
                    }),
                    text: Some("SUM(C1:C2)".to_string()),
                    text_kind: FormulaTextKind::SpreadsheetMlA1,
                    cached_value: FormulaCachedValueState::Missing,
                    attrs: array_attrs,
                    unsupported_fragments: Vec::new(),
                }],
                unsupported_fragments: Vec::new(),
            }),
            DocumentEvent::CellChunk(CellChunk {
                row_band: 0,
                cells: vec![
                    // The array anchor cell ingests as a normal formula.
                    (
                        addr(1, 1),
                        CellPayload::Formula {
                            region: None,
                            text: Some("SUM(C1:C2)".to_string()),
                            cached: Some(Box::new(CellPayload::Number(0.0))),
                        },
                    ),
                    (addr(1, 3), CellPayload::Number(4.0)),
                    (addr(2, 3), CellPayload::Number(6.0)),
                ],
            }),
            DocumentEvent::SheetEnd { sheet_id: 1 },
        ]);

        let report = load_workbook_events(&mut context, &workspace_id, &stream).unwrap();
        // The array cell bound as a normal formula (NOT diverted to NotCalcModeled).
        assert_eq!(
            report.formulas_bound, 1,
            "the array cell bound as a normal formula"
        );
        assert_eq!(report.not_calc_modeled, 0);
        // Exactly one inert Cse overlay rect claims the array range A1:A2.
        assert_eq!(
            report.inert_overlays.len(),
            1,
            "one inert Cse overlay claim"
        );
        assert_eq!(report.inert_overlays[0].kind, InertOverlayKind::Cse);
        assert_eq!(report.inert_overlays[0].rect, (1, 1, 2, 1), "claims A1:A2");

        // The bound array cell evaluates as a normal formula after recalc: 4+6=10.
        context.recalculate_workbook(&workspace_id).unwrap();
        assert_eq!(
            published_value(&context, &workspace_id, 1, 1).map(|(v, _)| v),
            Some(CalcValue::number(10.0)),
            "the array cell evaluates as a normal SUM"
        );
    }

    // ---- Acceptance: DataTable / Unknown topology routing ---------------------

    /// A `FormulaTopology` `DataTable` record: the cell is NOT calc-modeled — it
    /// publishes its FileCached value and is counted `NotCalcModeled`, never
    /// bound.
    #[test]
    fn data_table_topology_retains_cached_and_is_not_calc_modeled() {
        let (mut context, workspace_id) = workbook_context();
        let mut dt_attrs = FormulaRecordAttributes::normal();
        dt_attrs.formula_type = Some("dataTable".to_string());
        let mut stream = formula_prelude();
        stream.extend([
            DocumentEvent::SheetBegin(SheetRef {
                sheet_id: 1,
                name: "Sheet1".to_string(),
            }),
            DocumentEvent::FormulaTopology(FormulaTopology {
                sheet_id: 1,
                records: vec![FormulaRecord {
                    sheet_id: 1,
                    address: addr(1, 1),
                    kind: FormulaRecordKind::DataTable(DataTableFormulaSpec {
                        range: None,
                        two_dimensional: None,
                        row_table: None,
                        first_input: None,
                        second_input: None,
                        first_input_deleted: None,
                        second_input_deleted: None,
                    }),
                    text: Some("TABLE(B1,C1)".to_string()),
                    text_kind: FormulaTextKind::SpreadsheetMlA1,
                    cached_value: FormulaCachedValueState::Present {
                        provenance: CachedValueProvenance::FileCached,
                    },
                    attrs: dt_attrs,
                    unsupported_fragments: Vec::new(),
                }],
                unsupported_fragments: Vec::new(),
            }),
            DocumentEvent::CellChunk(CellChunk {
                row_band: 0,
                cells: vec![(
                    addr(1, 1),
                    CellPayload::Formula {
                        region: None,
                        text: Some("TABLE(B1,C1)".to_string()),
                        cached: Some(Box::new(CellPayload::Number(42.0))),
                    },
                )],
            }),
            DocumentEvent::SheetEnd { sheet_id: 1 },
        ]);

        let report = load_workbook_events(&mut context, &workspace_id, &stream).unwrap();
        assert_eq!(report.formulas_bound, 0, "a DataTable cell is not bound");
        assert_eq!(
            report.not_calc_modeled, 1,
            "the DataTable cell is NotCalcModeled"
        );
        assert!(report.inert_overlays.is_empty());

        // The cell publishes its FileCached value and never evaluates.
        assert_eq!(
            published_value(&context, &workspace_id, 1, 1),
            Some((
                CalcValue::number(42.0),
                PublishedValueProvenance::FileCached
            )),
            "the DataTable cell renders its FileCached cache"
        );
        // A recalc drains nothing (the cell was never seeded / bound).
        let outcome = context.recalculate_workbook(&workspace_id).unwrap();
        assert!(!outcome.drained_any(), "no bound cell → F9 is a no-op");
        assert_eq!(
            published_value(&context, &workspace_id, 1, 1).map(|(_, p)| p),
            Some(PublishedValueProvenance::FileCached),
            "the cache is pinned; recalc did not clobber it"
        );
    }

    /// A `FormulaTopology` `Unknown` record routes exactly like DataTable:
    /// publish cached, NotCalcModeled, never bound.
    #[test]
    fn unknown_topology_retains_cached_and_is_not_calc_modeled() {
        let (mut context, workspace_id) = workbook_context();
        let mut unknown_attrs = FormulaRecordAttributes::normal();
        unknown_attrs.formula_type = Some("someFutureType".to_string());
        let mut stream = formula_prelude();
        stream.extend([
            DocumentEvent::SheetBegin(SheetRef {
                sheet_id: 1,
                name: "Sheet1".to_string(),
            }),
            DocumentEvent::FormulaTopology(FormulaTopology {
                sheet_id: 1,
                records: vec![FormulaRecord {
                    sheet_id: 1,
                    address: addr(1, 1),
                    kind: FormulaRecordKind::Unknown {
                        formula_type: "someFutureType".to_string(),
                    },
                    text: Some("FUTURE()".to_string()),
                    text_kind: FormulaTextKind::SpreadsheetMlA1,
                    cached_value: FormulaCachedValueState::Present {
                        provenance: CachedValueProvenance::FileCached,
                    },
                    attrs: unknown_attrs,
                    unsupported_fragments: Vec::new(),
                }],
                unsupported_fragments: Vec::new(),
            }),
            DocumentEvent::CellChunk(CellChunk {
                row_band: 0,
                cells: vec![(
                    addr(1, 1),
                    CellPayload::Formula {
                        region: None,
                        text: Some("FUTURE()".to_string()),
                        cached: Some(Box::new(CellPayload::Bool(true))),
                    },
                )],
            }),
            DocumentEvent::SheetEnd { sheet_id: 1 },
        ]);

        let report = load_workbook_events(&mut context, &workspace_id, &stream).unwrap();
        assert_eq!(report.formulas_bound, 0);
        assert_eq!(report.not_calc_modeled, 1);
        assert_eq!(
            published_value(&context, &workspace_id, 1, 1),
            Some((
                CalcValue::logical(true),
                PublishedValueProvenance::FileCached
            )),
        );
    }

    // ---- Acceptance: degradation never fails the load ------------------------

    /// A corrupt formula (`=1+`, which OxFml rejects as a formula) retains its
    /// authored text + its FileCached cache + emits a `BindDegradation` row, and
    /// ingest SUCCEEDS (no Err). The cell is never bound and never discarded.
    #[test]
    fn corrupt_formula_degrades_with_cache_and_ledger_row_ingest_succeeds() {
        let (mut context, workspace_id) = workbook_context();
        let mut stream = formula_prelude();
        stream.extend([
            DocumentEvent::SheetBegin(SheetRef {
                sheet_id: 1,
                name: "Sheet1".to_string(),
            }),
            DocumentEvent::CellChunk(CellChunk {
                row_band: 0,
                cells: vec![(
                    addr(1, 1),
                    CellPayload::Formula {
                        region: None,
                        // `1+` is a truncated expression OxFml rejects.
                        text: Some("1+".to_string()),
                        cached: Some(Box::new(CellPayload::Number(99.0))),
                    },
                )],
            }),
            DocumentEvent::SheetEnd { sheet_id: 1 },
        ]);

        // Ingest SUCCEEDS — degradation is never a load failure (D4 §10).
        let report = load_workbook_events(&mut context, &workspace_id, &stream).unwrap();
        assert_eq!(report.formulas_bound, 0, "the corrupt formula did not bind");
        assert_eq!(report.bind_degradations.len(), 1, "one BindDegradation row");
        let degradation = &report.bind_degradations[0];
        assert_eq!(degradation.address, "R1C1");
        assert_eq!(
            degradation.text, "=1+",
            "the authored text is retained verbatim (=-restored), never discarded"
        );
        assert!(
            !degradation.diagnostics.is_empty(),
            "the rejection carries OxFml's diagnostics: {:?}",
            degradation.diagnostics
        );

        // The cell publishes its FileCached cache (not a fabricated error).
        assert_eq!(
            published_value(&context, &workspace_id, 1, 1),
            Some((
                CalcValue::number(99.0),
                PublishedValueProvenance::FileCached
            )),
            "a degraded cell with a cache publishes the cache"
        );
    }

    /// A corrupt formula with NO cache publishes a `#NAME?`-class typed error
    /// (never a load failure, never a fabricated value with the wrong shape).
    #[test]
    fn corrupt_formula_without_cache_publishes_name_error() {
        let (mut context, workspace_id) = workbook_context();
        let mut stream = formula_prelude();
        stream.extend([
            DocumentEvent::SheetBegin(SheetRef {
                sheet_id: 1,
                name: "Sheet1".to_string(),
            }),
            DocumentEvent::CellChunk(CellChunk {
                row_band: 0,
                cells: vec![(
                    addr(1, 1),
                    CellPayload::Formula {
                        region: None,
                        text: Some("1+".to_string()),
                        cached: None,
                    },
                )],
            }),
            DocumentEvent::SheetEnd { sheet_id: 1 },
        ]);

        let report = load_workbook_events(&mut context, &workspace_id, &stream).unwrap();
        assert_eq!(report.bind_degradations.len(), 1);
        // A cache-less degraded cell publishes #NAME? tagged `Degraded` — NOT
        // `FileCached` (nothing was read from a file, so a FileCached tag would
        // be a small lie). It is still differential-invisible (pre-engine).
        assert_eq!(
            published_value(&context, &workspace_id, 1, 1),
            Some((
                CalcValue::error(WorksheetErrorCode::Name),
                PublishedValueProvenance::Degraded
            )),
            "a cache-less degraded cell publishes a #NAME?-class error, tagged Degraded"
        );
    }

    // ---- Acceptance: honest per-cell ledger (R6.1 carry-forward a) ------------

    /// A formula-bearing load's report is distinguishable from a literals-only
    /// load (the R6.1 carry-forward): `formulas_bound` reflects bound formulas,
    /// and a RichStub is ledgered as deferred (never "consumed").
    #[test]
    fn formula_load_report_is_distinguishable_from_literals_only() {
        // Literals-only load: no formulas, no rich stubs.
        let (mut context_lit, ws_lit) = workbook_context();
        let literal_report =
            load_workbook_events(&mut context_lit, &ws_lit, &literals_only_stream()).unwrap();
        assert_eq!(literal_report.formulas_bound, 0);
        assert_eq!(literal_report.rich_stubs_deferred, 0);
        assert_eq!(literal_report.not_calc_modeled, 0);
        assert!(literal_report.bind_degradations.is_empty());

        // Formula + RichStub load: the report reflects both honestly.
        let (mut context, workspace_id) = workbook_context();
        let mut stream = formula_prelude();
        stream.extend([
            DocumentEvent::SheetBegin(SheetRef {
                sheet_id: 1,
                name: "Sheet1".to_string(),
            }),
            DocumentEvent::CellChunk(CellChunk {
                row_band: 0,
                cells: vec![
                    (addr(1, 1), CellPayload::Number(7.0)),
                    (
                        addr(1, 2),
                        CellPayload::Formula {
                            region: None,
                            text: Some("A1*3".to_string()),
                            cached: Some(Box::new(CellPayload::Number(21.0))),
                        },
                    ),
                    (addr(1, 3), CellPayload::RichStub(5)),
                ],
            }),
            DocumentEvent::SheetEnd { sheet_id: 1 },
        ]);
        let report = load_workbook_events(&mut context, &workspace_id, &stream).unwrap();
        assert_eq!(
            report.formulas_bound, 1,
            "the formula bound — a distinguishing signal"
        );
        assert_eq!(
            report.rich_stubs_deferred, 1,
            "the RichStub is ledgered as deferred, never consumed"
        );
        assert_ne!(
            report.formulas_bound, literal_report.formulas_bound,
            "a formula load reports differently from literals-only"
        );
    }

    // ---- Acceptance (canonical): shared-region MEMBER cells --------------------

    /// The CANONICAL shared-formula shape (mirrors OxDoc's own driver fixture):
    /// the member cells arrive in a `CellChunk` as `Formula { region: Some(id),
    /// text: None, cached: Some(..) }` and the anchor as `Formula { region:
    /// Some(id), text: Some(tmpl), cached: Some(..) }`. These are REGION-MANAGED:
    /// the `SharedFormulaRegion` installs the whole region (one mint at the
    /// anchor), so no member/anchor is individually bound.
    ///
    /// Without the region-membership routing this FAILS: `text: None` becomes
    /// `""` → `"="` → OxFml rejects → a false `BindDegradation{ text:"=" }` per
    /// member, AND the anchor double-installs, inflating `formulas_bound`.
    #[test]
    fn canonical_shared_region_member_cells_are_region_managed_not_degraded() {
        let (mut context, workspace_id) = workbook_context();
        // Manual mode so the load renders FileCached caches and the explicit F9
        // below genuinely drains the region (D4 §6); the region-membership routing
        // this test guards is calc-mode-independent.
        let mut stream = manual_formula_prelude();
        stream.extend([
            DocumentEvent::SheetBegin(SheetRef {
                sheet_id: 1,
                name: "Sheet1".to_string(),
            }),
            // The shared region (=RC[-1]*3 over B1:B3) precedes the cell chunk.
            DocumentEvent::SharedFormulaRegion(SharedFormulaRegion {
                region_id: 7,
                anchor: addr(1, 2),
                extent: Extent { rows: 3, cols: 1 },
                r1c1_text: "RC[-1]*3".to_string(),
            }),
            DocumentEvent::CellChunk(CellChunk {
                // Cells in row-major address order (the stream validator's rule).
                row_band: 0,
                cells: vec![
                    (addr(1, 1), CellPayload::Number(1.0)),
                    // The region ANCHOR: region: Some, text: Some(template), cache.
                    (
                        addr(1, 2),
                        CellPayload::Formula {
                            region: Some(7),
                            text: Some("A1*3".to_string()),
                            cached: Some(Box::new(CellPayload::Number(3.0))),
                        },
                    ),
                    (addr(2, 1), CellPayload::Number(2.0)),
                    // The region MEMBERS: region: Some, text: NONE, cache only.
                    (
                        addr(2, 2),
                        CellPayload::Formula {
                            region: Some(7),
                            text: None,
                            cached: Some(Box::new(CellPayload::Number(6.0))),
                        },
                    ),
                    (addr(3, 1), CellPayload::Number(3.0)),
                    (
                        addr(3, 2),
                        CellPayload::Formula {
                            region: Some(7),
                            text: None,
                            cached: Some(Box::new(CellPayload::Number(9.0))),
                        },
                    ),
                ],
            }),
            DocumentEvent::SheetEnd { sheet_id: 1 },
        ]);

        let report = load_workbook_events(&mut context, &workspace_id, &stream).unwrap();
        // ZERO false degradations: a `text: None` member is NEVER bound as `"="`.
        assert!(
            report.bind_degradations.is_empty(),
            "region-managed cells must not degrade, got {:?}",
            report.bind_degradations
        );
        // The region counts ONCE, not per member cell (no anchor double-install).
        assert_eq!(
            report.formulas_bound, 1,
            "the shared region counts once, not per member"
        );

        // Pre-F9: every extent cell renders its FileCached cache.
        for (row, expected) in [(1u32, 3.0), (2, 6.0), (3, 9.0)] {
            assert_eq!(
                published_value(&context, &workspace_id, row, 2),
                Some((
                    CalcValue::number(expected),
                    PublishedValueProvenance::FileCached
                )),
                "B{row} renders its FileCached cache pre-recalc"
            );
        }

        // F9: the region drains and every cell evaluates to the correct
        // relative-adjusted value (B{r} = A{r}*3), differential-proven.
        context.recalculate_workbook(&workspace_id).unwrap();
        for (row, expected) in [(1u32, 3.0), (2, 6.0), (3, 9.0)] {
            let (value, provenance) = published_value(&context, &workspace_id, row, 2).unwrap();
            assert_eq!(
                value,
                CalcValue::number(expected),
                "B{row} = A{row}*3 = {expected} by the engine (relative-adjusted)"
            );
            assert!(
                matches!(provenance, PublishedValueProvenance::Calculated { .. }),
                "B{row} is engine-Calculated after F9"
            );
        }
        let node = context.sheets(&workspace_id).unwrap()[0].node_id;
        let view = context.grid_view(&workspace_id, node).unwrap().unwrap();
        assert!(
            view.differential_mismatches.is_empty(),
            "shared-region expansion is differential-clean, got {:?}",
            view.differential_mismatches
        );
    }

    // ---- Acceptance (canonical): FileCached pins survive a real F9 ------------

    /// A sheet with BOTH a bound formula (so F9 genuinely drains, not a no-op)
    /// AND a non-calc-modeled DataTable pinned cell. After a real
    /// `recalculate_workbook`: the bound formula is engine-`Calculated`, AND the
    /// pinned cell STILL renders its FileCached value (it does not vanish).
    ///
    /// Without the persistent `file_cached_pins` this FAILS: `rebuild_from_input`
    /// wipes `published` and recalc rebuilds it wholesale from the engine
    /// readout, which never covers the pinned (never-evaluated) cell — so the
    /// DataTable cell's 42 disappears from the readout after F9.
    #[test]
    fn canonical_filecached_pin_survives_a_genuine_recalc_drain() {
        let (mut context, workspace_id) = workbook_context();
        let mut dt_attrs = FormulaRecordAttributes::normal();
        dt_attrs.formula_type = Some("dataTable".to_string());
        // Manual mode so the load seeds caches and the explicit F9 below genuinely
        // drains B1 (D4 §6); the pin-survival contract this test guards holds
        // regardless of which recalc pass (open-recalc or F9) runs.
        let mut stream = manual_formula_prelude();
        stream.extend([
            DocumentEvent::SheetBegin(SheetRef {
                sheet_id: 1,
                name: "Sheet1".to_string(),
            }),
            // C1 is a DataTable cell (not calc-modeled → pinned).
            DocumentEvent::FormulaTopology(FormulaTopology {
                sheet_id: 1,
                records: vec![FormulaRecord {
                    sheet_id: 1,
                    address: addr(1, 3),
                    kind: FormulaRecordKind::DataTable(DataTableFormulaSpec {
                        range: None,
                        two_dimensional: None,
                        row_table: None,
                        first_input: None,
                        second_input: None,
                        first_input_deleted: None,
                        second_input_deleted: None,
                    }),
                    text: Some("TABLE(A1,B1)".to_string()),
                    text_kind: FormulaTextKind::SpreadsheetMlA1,
                    cached_value: FormulaCachedValueState::Present {
                        provenance: CachedValueProvenance::FileCached,
                    },
                    attrs: dt_attrs,
                    unsupported_fragments: Vec::new(),
                }],
                unsupported_fragments: Vec::new(),
            }),
            DocumentEvent::CellChunk(CellChunk {
                row_band: 0,
                cells: vec![
                    (addr(1, 1), CellPayload::Number(7.0)),
                    // B1 = =A1*3 — a BOUND formula, so F9 genuinely drains.
                    (
                        addr(1, 2),
                        CellPayload::Formula {
                            region: None,
                            text: Some("A1*3".to_string()),
                            cached: Some(Box::new(CellPayload::Number(21.0))),
                        },
                    ),
                    // C1 = the DataTable pinned cell, cached 42.
                    (
                        addr(1, 3),
                        CellPayload::Formula {
                            region: None,
                            text: Some("TABLE(A1,B1)".to_string()),
                            cached: Some(Box::new(CellPayload::Number(42.0))),
                        },
                    ),
                ],
            }),
            DocumentEvent::SheetEnd { sheet_id: 1 },
        ]);

        let report = load_workbook_events(&mut context, &workspace_id, &stream).unwrap();
        assert_eq!(report.formulas_bound, 1, "only B1 bound");
        assert_eq!(report.not_calc_modeled, 1, "C1 is the DataTable pin");

        // Pre-F9: C1 renders its FileCached 42.
        assert_eq!(
            published_value(&context, &workspace_id, 1, 3),
            Some((
                CalcValue::number(42.0),
                PublishedValueProvenance::FileCached
            )),
            "C1 renders its FileCached cache pre-recalc"
        );

        // A GENUINE drain (B1 is seeded).
        let outcome = context.recalculate_workbook(&workspace_id).unwrap();
        assert!(
            outcome.drained_any(),
            "F9 genuinely drains (B1 is bound+seeded)"
        );

        // B1 is now engine-Calculated (7*3 = 21).
        let (b1, b1_prov) = published_value(&context, &workspace_id, 1, 2).unwrap();
        assert_eq!(
            b1,
            CalcValue::number(21.0),
            "B1 == A1*3 == 21 by the engine"
        );
        assert!(
            matches!(b1_prov, PublishedValueProvenance::Calculated { .. }),
            "B1 is engine-Calculated after the drain"
        );

        // THE PIN SURVIVES: C1 STILL renders its FileCached 42 — the drain that
        // rebuilt `published` from the engine readout did not erase it.
        assert_eq!(
            published_value(&context, &workspace_id, 1, 3),
            Some((
                CalcValue::number(42.0),
                PublishedValueProvenance::FileCached
            )),
            "the DataTable pin survives a genuine F9 drain"
        );
    }

    /// The corrupt-degraded variant of the pin-survival contract: a bound formula
    /// (F9 drains) PLUS a corrupt-degraded formula. After a real recalc the bound
    /// formula is Calculated and the degraded cell STILL renders its pinned value.
    #[test]
    fn canonical_degraded_pin_survives_a_genuine_recalc_drain() {
        let (mut context, workspace_id) = workbook_context();
        // Manual mode so the load seeds caches and the explicit F9 genuinely drains
        // B1 (D4 §6); the degraded-pin-survival contract is calc-mode-independent.
        let mut stream = manual_formula_prelude();
        stream.extend([
            DocumentEvent::SheetBegin(SheetRef {
                sheet_id: 1,
                name: "Sheet1".to_string(),
            }),
            DocumentEvent::CellChunk(CellChunk {
                row_band: 0,
                cells: vec![
                    (addr(1, 1), CellPayload::Number(7.0)),
                    // B1 = =A1*3 — bound, so F9 drains.
                    (
                        addr(1, 2),
                        CellPayload::Formula {
                            region: None,
                            text: Some("A1*3".to_string()),
                            cached: Some(Box::new(CellPayload::Number(21.0))),
                        },
                    ),
                    // C1 = corrupt formula with a cache 99 → degraded pin.
                    (
                        addr(1, 3),
                        CellPayload::Formula {
                            region: None,
                            text: Some("1+".to_string()),
                            cached: Some(Box::new(CellPayload::Number(99.0))),
                        },
                    ),
                ],
            }),
            DocumentEvent::SheetEnd { sheet_id: 1 },
        ]);

        let report = load_workbook_events(&mut context, &workspace_id, &stream).unwrap();
        assert_eq!(report.formulas_bound, 1);
        assert_eq!(report.bind_degradations.len(), 1);

        let outcome = context.recalculate_workbook(&workspace_id).unwrap();
        assert!(outcome.drained_any(), "F9 drains (B1 is bound)");

        // B1 Calculated; the degraded C1 pin (cache 99, FileCached) survives.
        assert!(
            matches!(
                published_value(&context, &workspace_id, 1, 2).map(|(_, p)| p),
                Some(PublishedValueProvenance::Calculated { .. })
            ),
            "B1 is engine-Calculated after the drain"
        );
        assert_eq!(
            published_value(&context, &workspace_id, 1, 3),
            Some((
                CalcValue::number(99.0),
                PublishedValueProvenance::FileCached
            )),
            "the degraded-formula pin survives a genuine F9 drain"
        );
    }

    // ---- Acceptance (canonical): pins are pruned by authored mutations --------

    /// Repairing a degraded formula (authoring a real formula at the pinned
    /// address) retires the pin: after recalc the cell shows the ENGINE value, and
    /// the pin never shadows it again on a second recalc.
    ///
    /// Without pin-pruning this FAILS: the recalc re-stamp overwrites the fresh
    /// `Calculated` value with the stale `#NAME?`/cache pin, forever.
    #[test]
    fn canonical_repairing_a_degraded_cell_prunes_the_pin() {
        let (mut context, workspace_id) = workbook_context();
        let mut stream = formula_prelude();
        stream.extend([
            DocumentEvent::SheetBegin(SheetRef {
                sheet_id: 1,
                name: "Sheet1".to_string(),
            }),
            DocumentEvent::CellChunk(CellChunk {
                row_band: 0,
                cells: vec![
                    // A1 = corrupt formula with cache 99 → degraded pin.
                    (
                        addr(1, 1),
                        CellPayload::Formula {
                            region: None,
                            text: Some("1+".to_string()),
                            cached: Some(Box::new(CellPayload::Number(99.0))),
                        },
                    ),
                ],
            }),
            DocumentEvent::SheetEnd { sheet_id: 1 },
        ]);
        let report = load_workbook_events(&mut context, &workspace_id, &stream).unwrap();
        assert_eq!(report.bind_degradations.len(), 1);
        // Pre-repair: A1 renders the pinned cache 99.
        assert_eq!(
            published_value(&context, &workspace_id, 1, 1),
            Some((
                CalcValue::number(99.0),
                PublishedValueProvenance::FileCached
            )),
        );

        // The user REPAIRS the cell: types a valid formula `=1+2`.
        let node = context.sheets(&workspace_id).unwrap()[0].node_id;
        let a1 = ingested_address(&context, &workspace_id, 1, 1);
        context
            .enter_grid_cell(&workspace_id, node, &a1, "=1+2")
            .unwrap()
            .unwrap();

        // The cell now shows the ENGINE value 3, Calculated — the pin is gone.
        let (value, provenance) = published_value(&context, &workspace_id, 1, 1).unwrap();
        assert_eq!(
            value,
            CalcValue::number(3.0),
            "the repaired formula evaluates to 3"
        );
        assert!(
            matches!(provenance, PublishedValueProvenance::Calculated { .. }),
            "the repaired cell is engine-Calculated, not the stale pin"
        );

        // A SECOND recalc must not resurrect the pin (edit A1 again to force a
        // genuine drain, then confirm the engine value still wins).
        context
            .enter_grid_cell(&workspace_id, node, &a1, "=2+2")
            .unwrap()
            .unwrap();
        let (value2, provenance2) = published_value(&context, &workspace_id, 1, 1).unwrap();
        assert_eq!(value2, CalcValue::number(4.0), "the pin never resurrects");
        assert!(matches!(
            provenance2,
            PublishedValueProvenance::Calculated { .. }
        ));
    }

    /// Clearing a pinned cell (a DataTable / degraded cell) actually empties it —
    /// the pin is retired and the cell no longer renders the pinned value.
    ///
    /// Without pin-pruning this FAILS: `clear_grid_cell` at a pinned address is an
    /// idempotent no-op (the address is not in `input.cells`), so the zombie pin
    /// is un-removable and the cell stays stuck on the pinned value.
    #[test]
    fn canonical_clearing_a_pinned_cell_actually_clears_it() {
        let (mut context, workspace_id) = workbook_context();
        let mut dt_attrs = FormulaRecordAttributes::normal();
        dt_attrs.formula_type = Some("dataTable".to_string());
        let mut stream = formula_prelude();
        stream.extend([
            DocumentEvent::SheetBegin(SheetRef {
                sheet_id: 1,
                name: "Sheet1".to_string(),
            }),
            DocumentEvent::FormulaTopology(FormulaTopology {
                sheet_id: 1,
                records: vec![FormulaRecord {
                    sheet_id: 1,
                    address: addr(1, 1),
                    kind: FormulaRecordKind::DataTable(DataTableFormulaSpec {
                        range: None,
                        two_dimensional: None,
                        row_table: None,
                        first_input: None,
                        second_input: None,
                        first_input_deleted: None,
                        second_input_deleted: None,
                    }),
                    text: Some("TABLE(B1,C1)".to_string()),
                    text_kind: FormulaTextKind::SpreadsheetMlA1,
                    cached_value: FormulaCachedValueState::Present {
                        provenance: CachedValueProvenance::FileCached,
                    },
                    attrs: dt_attrs,
                    unsupported_fragments: Vec::new(),
                }],
                unsupported_fragments: Vec::new(),
            }),
            DocumentEvent::CellChunk(CellChunk {
                row_band: 0,
                cells: vec![(
                    addr(1, 1),
                    CellPayload::Formula {
                        region: None,
                        text: Some("TABLE(B1,C1)".to_string()),
                        cached: Some(Box::new(CellPayload::Number(42.0))),
                    },
                )],
            }),
            DocumentEvent::SheetEnd { sheet_id: 1 },
        ]);
        let report = load_workbook_events(&mut context, &workspace_id, &stream).unwrap();
        assert_eq!(report.not_calc_modeled, 1);
        // Pre-clear: A1 renders the pinned 42.
        assert_eq!(
            published_value(&context, &workspace_id, 1, 1),
            Some((
                CalcValue::number(42.0),
                PublishedValueProvenance::FileCached
            )),
        );

        // The user CLEARS the pinned cell.
        let node = context.sheets(&workspace_id).unwrap()[0].node_id;
        let a1 = ingested_address(&context, &workspace_id, 1, 1);
        context.clear_grid_cell(&workspace_id, node, &a1).unwrap();

        // The cell is actually EMPTY now — not stuck on the pinned 42.
        assert_eq!(
            published_value(&context, &workspace_id, 1, 1),
            None,
            "the cleared pinned cell is empty, not stuck on the zombie pin"
        );
        // A recalc does not resurrect it either.
        context.recalculate_workbook(&workspace_id).unwrap();
        assert_eq!(
            published_value(&context, &workspace_id, 1, 1),
            None,
            "the pin does not resurrect after clear + recalc"
        );
    }

    // ---- Acceptance (MINOR): dangling region cell is accounted, not dropped ---

    /// A cell with `region: Some(id)` whose `SharedFormulaRegion` never arrives is
    /// UNBACKED (oxdoc-model does not enforce the pairing). Its cache still
    /// publishes, but it is ACCOUNTED (`region_cells_unbacked`), never silently
    /// dropped — the honesty regime (C13) forbids a silent loss.
    ///
    /// Without the unbacked accounting this FAILS: the cell's formula vanishes
    /// with no count, no degradation, no ledger — a silent drop.
    #[test]
    fn canonical_dangling_region_cell_is_accounted_not_dropped() {
        let (mut context, workspace_id) = workbook_context();
        let mut stream = formula_prelude();
        stream.extend([
            DocumentEvent::SheetBegin(SheetRef {
                sheet_id: 1,
                name: "Sheet1".to_string(),
            }),
            // NO SharedFormulaRegion event — the region cell below is dangling.
            DocumentEvent::CellChunk(CellChunk {
                row_band: 0,
                cells: vec![
                    (addr(1, 1), CellPayload::Number(5.0)),
                    // B1 references region 7, which never arrives.
                    (
                        addr(1, 2),
                        CellPayload::Formula {
                            region: Some(7),
                            text: None,
                            cached: Some(Box::new(CellPayload::Number(15.0))),
                        },
                    ),
                ],
            }),
            DocumentEvent::SheetEnd { sheet_id: 1 },
        ]);

        // Ingest SUCCEEDS (a malformed stream degrades, never fails).
        let report = load_workbook_events(&mut context, &workspace_id, &stream).unwrap();
        // The dangling region cell is ACCOUNTED, not silently dropped.
        assert_eq!(
            report.region_cells_unbacked, 1,
            "the dangling region cell is counted"
        );
        assert_eq!(
            report.formulas_bound, 0,
            "no region was installed to bind it"
        );
        // Its cache still publishes (so the cell renders), pinned (no formula
        // binds it, so a transient publication would be erased by recalc).
        assert_eq!(
            published_value(&context, &workspace_id, 1, 2),
            Some((
                CalcValue::number(15.0),
                PublishedValueProvenance::FileCached
            )),
            "the dangling cell renders its cache, accounted as unbacked"
        );
    }

    // ==== R6.3: names, tables, merges =========================================

    /// The published value + provenance of the grid node at sheet position
    /// `sheet_index` (0-based, in sheet order) and cell `(row, col)`. The
    /// multi-sheet analog of [`published_value`].
    fn published_value_on_sheet(
        context: &OxCalcDocumentContext,
        workspace_id: &OxCalcTreeWorkspaceId,
        sheet_index: usize,
        row: u32,
        col: u32,
    ) -> Option<(CalcValue, PublishedValueProvenance)> {
        let node = context.sheets(workspace_id).unwrap()[sheet_index].node_id;
        let view = context.grid_view(workspace_id, node).unwrap().unwrap();
        view.cells
            .iter()
            .find(|cell| cell.address.row == row && cell.address.col == col)
            .map(|cell| (cell.value.clone(), cell.provenance))
    }

    /// The grid node id at sheet position `sheet_index` (0-based, sheet order).
    fn sheet_node(
        context: &OxCalcDocumentContext,
        workspace_id: &OxCalcTreeWorkspaceId,
        sheet_index: usize,
    ) -> crate::structural::TreeNodeId {
        context.sheets(workspace_id).unwrap()[sheet_index].node_id
    }

    /// An [`ExcelGridCellAddress`] on the sheet at position `sheet_index`, with
    /// the workbook/sheet tokens the ingest builder derives.
    fn ingested_address_on_sheet(
        context: &OxCalcDocumentContext,
        workspace_id: &OxCalcTreeWorkspaceId,
        sheet_index: usize,
        row: u32,
        col: u32,
    ) -> ExcelGridCellAddress {
        let node = sheet_node(context, workspace_id, sheet_index);
        ExcelGridCellAddress::new(
            format!("book:{}", workspace_id.as_str()),
            format!("sheet:{}", node.0),
            row,
            col,
        )
    }

    /// Acceptance (scope precedence, V8): a sheet-scoped name and a
    /// workbook-scoped name of the SAME text resolve per precedence —
    /// sheet-before-workbook — from a formula on the sheet the shadow lives on.
    ///
    /// Canonical shapes: two `DefinedName` events with the same text `Total`, one
    /// workbook-scoped (`scope_sheet_id: None`) pointing at `Sheet1!$B$1`, one
    /// sheet-scoped (`scope_sheet_id: Some(1)`) pointing at `Sheet1!$C$1`. A
    /// formula `=Total` on Sheet1 must read the SHEET-scoped target (`C1`), not
    /// the workbook one (`B1`) — the shadow wins (D2 §4.3 / V8).
    #[test]
    fn scoped_name_precedence_sheet_shadows_workbook_from_ingest() {
        let (mut context, workspace_id) = workbook_context();
        let mut stream = formula_prelude();
        stream.extend([
            // Both names arrive workbook-position (before any sheet), canonical
            // per the driver fixture (`oxcalc_ingest_driver_visits_...`): the
            // DefinedName event precedes the target sheet's SheetBegin.
            DocumentEvent::DefinedName(DefinedNameSpec {
                name: "Total".to_string(),
                formula_text: "Sheet1!$B$1".to_string(),
                scope_sheet_id: None,
                metadata: DefinedNameMetadataSpec::default(),
            }),
            DocumentEvent::DefinedName(DefinedNameSpec {
                name: "Total".to_string(),
                formula_text: "Sheet1!$C$1".to_string(),
                scope_sheet_id: Some(1),
                metadata: DefinedNameMetadataSpec::default(),
            }),
            DocumentEvent::SheetBegin(SheetRef {
                sheet_id: 1,
                name: "Sheet1".to_string(),
            }),
            DocumentEvent::CellChunk(CellChunk {
                row_band: 0,
                // Cells in ascending A1 order (the chunk-order the validator
                // requires): A1 = =Total, B1 = 10 (workbook target), C1 = 99
                // (sheet target, which must WIN).
                cells: vec![
                    (
                        addr(1, 1),
                        CellPayload::Formula {
                            region: None,
                            text: Some("Total".to_string()),
                            cached: None,
                        },
                    ),
                    (addr(1, 2), CellPayload::Number(10.0)),
                    (addr(1, 3), CellPayload::Number(99.0)),
                ],
            }),
            DocumentEvent::SheetEnd { sheet_id: 1 },
        ]);

        let report = load_workbook_events(&mut context, &workspace_id, &stream).unwrap();
        assert_eq!(report.names, 2, "both static names installed");

        // Automatic mode opens with a recalc (formula_prelude), but load itself
        // does not open-recalc (R6.5); drive F9 explicitly so =Total resolves.
        context.recalculate_workbook(&workspace_id).unwrap();

        let (value, _) = published_value(&context, &workspace_id, 1, 1).unwrap();
        assert_eq!(
            value,
            CalcValue::number(99.0),
            "=Total resolves the SHEET-scoped shadow (C1=99), not the workbook name (B1=10)"
        );
    }

    /// Acceptance (forward reference): a `DefinedName` whose target sheet appears
    /// LATER in the stream (the name event precedes that sheet's `SheetBegin`)
    /// loads clean and resolves post-commit. This is the ordering-proofness the
    /// deferred install (D4 §9) exists for.
    ///
    /// Canonical shape: the name event is emitted at workbook position, but its
    /// target is `Sheet2!$A$1` — Sheet2 is the SECOND sheet, so at the moment the
    /// name event is driven, Sheet2 does not yet exist. The install is deferred to
    /// commit (after both sheets exist), so the name resolves on Sheet2.
    #[test]
    fn forward_referencing_name_resolves_after_deferred_install() {
        let (mut context, workspace_id) = workbook_context();
        let mut stream = formula_prelude();
        stream.extend([
            // The name targets Sheet2 — which appears AFTER this event. A
            // non-deferred install would fail to resolve the sheet here.
            DocumentEvent::DefinedName(DefinedNameSpec {
                name: "FarInput".to_string(),
                formula_text: "Sheet2!$A$1".to_string(),
                scope_sheet_id: None,
                metadata: DefinedNameMetadataSpec::default(),
            }),
            DocumentEvent::SheetBegin(SheetRef {
                sheet_id: 1,
                name: "Sheet1".to_string(),
            }),
            DocumentEvent::CellChunk(CellChunk {
                row_band: 0,
                // Sheet1!A1 = =FarInput. This reads the name whose target is on
                // Sheet2 — but the name is authored ON Sheet2 (the rect's home),
                // so a Sheet1 formula reading it resolves cross-sheet only after
                // R6.5's load recalc policy wires cross-sheet views. Here we prove
                // the LOAD is CLEAN and the name RESOLVES on its own sheet
                // (Sheet2!A1 references itself trivially via a self formula), which
                // is the deferred-install acceptance: the name binds against a
                // sheet that did not exist when its event arrived.
                cells: vec![(addr(1, 1), CellPayload::Number(0.0))],
            }),
            DocumentEvent::SheetEnd { sheet_id: 1 },
            DocumentEvent::SheetBegin(SheetRef {
                sheet_id: 2,
                name: "Sheet2".to_string(),
            }),
            DocumentEvent::CellChunk(CellChunk {
                row_band: 0,
                cells: vec![
                    // Sheet2!A1 = 42 (the name's target), B1 = =FarInput on Sheet2
                    // resolves the workbook name natively to 42.
                    (addr(1, 1), CellPayload::Number(42.0)),
                    (
                        addr(1, 2),
                        CellPayload::Formula {
                            region: None,
                            text: Some("FarInput".to_string()),
                            cached: None,
                        },
                    ),
                ],
            }),
            DocumentEvent::SheetEnd { sheet_id: 2 },
        ]);

        // The load succeeds cleanly despite the forward reference.
        let report = load_workbook_events(&mut context, &workspace_id, &stream).unwrap();
        assert_eq!(report.sheets, 2);
        assert_eq!(
            report.names, 1,
            "the forward-referencing name installed on its target sheet (Sheet2)"
        );
        assert!(
            report.bind_degradations.is_empty(),
            "no degradation: the deferred install resolved the later sheet, got {:?}",
            report.bind_degradations
        );

        // Post-commit, F9: Sheet2!B1 = =FarInput resolves the name to Sheet2!A1=42.
        context.recalculate_workbook(&workspace_id).unwrap();
        let (value, _) = published_value_on_sheet(&context, &workspace_id, 1, 1, 2).unwrap();
        assert_eq!(
            value,
            CalcValue::number(42.0),
            "=FarInput on Sheet2 resolves the workbook name to Sheet2!A1 = 42 post-commit"
        );
    }

    /// Acceptance (merge — spill block): an ingested merged region blocks a spill.
    /// A merged region A2:B3 sits under a spilling formula at A1; the spill is
    /// blocked (`#SPILL!`) because a merged follower occupies its extent.
    ///
    /// Canonical shape: `MergedCellRegions { sheet_id, ranges }` arrives inside
    /// its sheet, exactly as the driver fixture emits it.
    #[test]
    fn ingested_merged_region_blocks_a_spill() {
        let (mut context, workspace_id) = workbook_context();
        let mut stream = formula_prelude();
        stream.extend([
            DocumentEvent::SheetBegin(SheetRef {
                sheet_id: 1,
                name: "Sheet1".to_string(),
            }),
            // A merged region A2:B3 (canonical CellRangeSpec with parsed coords).
            DocumentEvent::MergedCellRegions(MergedCellRegions {
                sheet_id: 1,
                ranges: vec![CellRangeSpec {
                    text: "A2:B3".to_string(),
                    start: addr(2, 1),
                    end: addr(3, 2),
                }],
                raw_refs: Vec::new(),
            }),
            DocumentEvent::CellChunk(CellChunk {
                row_band: 0,
                cells: vec![
                    // A1 spills a 3-row array DOWN into A1:A3 — but A2 is a merged
                    // follower, so the spill is blocked (#SPILL!).
                    (
                        addr(1, 1),
                        CellPayload::Formula {
                            region: None,
                            text: Some("{1;2;3}".to_string()),
                            cached: None,
                        },
                    ),
                ],
            }),
            DocumentEvent::SheetEnd { sheet_id: 1 },
        ]);

        let report = load_workbook_events(&mut context, &workspace_id, &stream).unwrap();
        assert!(
            report.bind_degradations.is_empty(),
            "the array formula binds; the merge is a live spill-block, got {:?}",
            report.bind_degradations
        );
        context.recalculate_workbook(&workspace_id).unwrap();

        // A1 is #SPILL! — the merged region A2:B3 blocks the array spill (a LIVE
        // engine semantic, not inert retention).
        let (value, _) = published_value(&context, &workspace_id, 1, 1).unwrap();
        assert_eq!(
            value,
            CalcValue::error(WorksheetErrorCode::Spill),
            "the ingested merge blocks the spill (#SPILL!), got {value:?}"
        );

        // Differential-clean: the reference and optimized engines agree on the
        // whole sheet, merge blockage included.
        let node = sheet_node(&context, &workspace_id, 0);
        let view = context.grid_view(&workspace_id, node).unwrap().unwrap();
        assert!(
            view.differential_mismatches.is_empty(),
            "the merge-blocked spill is differential-clean, got {:?}",
            view.differential_mismatches
        );
    }

    /// Acceptance (merge — edit admission): an edit to a merged FOLLOWER is
    /// rejected with the typed `MergedFollower` reason; the anchor is writable.
    /// This proves the ingested merge drives LIVE edit-admission semantics.
    #[test]
    fn ingested_merged_region_rejects_a_follower_edit() {
        let (mut context, workspace_id) = workbook_context();
        let mut stream = formula_prelude();
        stream.extend([
            DocumentEvent::SheetBegin(SheetRef {
                sheet_id: 1,
                name: "Sheet1".to_string(),
            }),
            DocumentEvent::MergedCellRegions(MergedCellRegions {
                sheet_id: 1,
                ranges: vec![CellRangeSpec {
                    text: "A1:B2".to_string(),
                    start: addr(1, 1),
                    end: addr(2, 2),
                }],
                raw_refs: Vec::new(),
            }),
            DocumentEvent::CellChunk(CellChunk {
                row_band: 0,
                cells: vec![(addr(1, 1), CellPayload::Number(1.0))],
            }),
            DocumentEvent::SheetEnd { sheet_id: 1 },
        ]);
        load_workbook_events(&mut context, &workspace_id, &stream).unwrap();

        let node = sheet_node(&context, &workspace_id, 0);
        // B2 is a merged follower (anchor is A1): editing it is a typed rejection.
        let follower = ingested_address_on_sheet(&context, &workspace_id, 0, 2, 2);
        let err = context
            .enter_grid_cell(&workspace_id, node, &follower, "5")
            .unwrap_err();
        assert!(
            matches!(
                err,
                OxCalcDocumentError::GridCellNotEditable {
                    reason: crate::grid::authored::GridCellNotEditable::MergedFollower { .. },
                    ..
                }
            ),
            "editing a merged follower is a typed MergedFollower rejection, got {err:?}"
        );

        // The anchor A1 remains editable (the merge does not lock the whole rect).
        let anchor = ingested_address_on_sheet(&context, &workspace_id, 0, 1, 1);
        context
            .enter_grid_cell(&workspace_id, node, &anchor, "8")
            .unwrap()
            .expect("the merge anchor is writable");
    }

    /// Acceptance (table — structured reference): a structured reference
    /// `T[Column1]` resolves to the table's column data range. An ingested table
    /// over A1:B3 (header row 1) with a SUM over its first column reads the two
    /// data rows.
    ///
    /// Canonical shape: `TableSpec { name, sheet_id, range }` arrives inside its
    /// sheet, exactly as the driver fixture emits it. The column names come from
    /// the positional synthesis (`Column1`, `Column2`) this bead uses (oxdoc-model
    /// carries no column names; faithful names are R6.4's Tier-B store).
    #[test]
    fn ingested_table_resolves_a_structured_reference() {
        let (mut context, workspace_id) = workbook_context();
        let mut stream = formula_prelude();
        stream.extend([
            DocumentEvent::SheetBegin(SheetRef {
                sheet_id: 1,
                name: "Sheet1".to_string(),
            }),
            // A table T over A1:B3: header row 1, data rows 2-3.
            DocumentEvent::TableOverlay(TableSpec {
                name: "T".to_string(),
                sheet_id: 1,
                range: "A1:B3".to_string(),
            }),
            DocumentEvent::CellChunk(CellChunk {
                row_band: 0,
                // Ascending A1 (row-major) order: A1 (header), D1 (formula, still
                // row 1), then A2/A3 (data).
                cells: vec![
                    (addr(1, 1), CellPayload::InlineText("Col1".to_string())),
                    // D1 = SUM(T[Column1]) — the structured reference to the first
                    // column's data range (A2:A3), which must sum to 12.
                    (
                        addr(1, 4),
                        CellPayload::Formula {
                            region: None,
                            text: Some("SUM(T[Column1])".to_string()),
                            cached: None,
                        },
                    ),
                    (addr(2, 1), CellPayload::Number(5.0)),
                    (addr(3, 1), CellPayload::Number(7.0)),
                ],
            }),
            DocumentEvent::SheetEnd { sheet_id: 1 },
        ]);

        let report = load_workbook_events(&mut context, &workspace_id, &stream).unwrap();
        assert_eq!(report.tables, 1, "one table installed");
        assert!(
            report.bind_degradations.is_empty(),
            "SUM(T[Column1]) binds against the ingested table, got {:?}",
            report.bind_degradations
        );

        context.recalculate_workbook(&workspace_id).unwrap();
        let (value, _) = published_value(&context, &workspace_id, 1, 4).unwrap();
        assert_eq!(
            value,
            CalcValue::number(12.0),
            "SUM(T[Column1]) resolves the column data range A2:A3 = 5+7 = 12, got {value:?}"
        );

        // Differential-clean: the reference and optimized engines agree on the
        // structured-reference resolution.
        let node = sheet_node(&context, &workspace_id, 0);
        let view = context.grid_view(&workspace_id, node).unwrap().unwrap();
        assert!(
            view.differential_mismatches.is_empty(),
            "the structured-reference resolution is differential-clean, got {:?}",
            view.differential_mismatches
        );
    }

    /// Acceptance (deferred-install ordering-proofness, explicit): the SAME
    /// name/table/merge stream loads to the SAME result whether the position-free
    /// `DefinedName` event arrives BEFORE or AFTER its target sheet. This is the
    /// direct proof that the commit-time deferred install (D4 §9) does not rely on
    /// validator ordering — an out-of-order stream is byte-equivalent in outcome.
    #[test]
    fn deferred_install_is_ordering_proof_out_of_order_stream() {
        // Helper: build the stream with the DefinedName either before Sheet1
        // (position-free, forward-ish) or after the sheet's content.
        let build = |name_first: bool| {
            let name_event = DocumentEvent::DefinedName(DefinedNameSpec {
                name: "Anchor".to_string(),
                formula_text: "Sheet1!$B$1".to_string(),
                scope_sheet_id: None,
                metadata: DefinedNameMetadataSpec::default(),
            });
            let sheet_events = vec![
                DocumentEvent::SheetBegin(SheetRef {
                    sheet_id: 1,
                    name: "Sheet1".to_string(),
                }),
                DocumentEvent::CellChunk(CellChunk {
                    row_band: 0,
                    // Ascending A1 order: A1 (= =Anchor), then B1 (the target).
                    cells: vec![
                        (
                            addr(1, 1),
                            CellPayload::Formula {
                                region: None,
                                text: Some("Anchor".to_string()),
                                cached: None,
                            },
                        ),
                        (addr(1, 2), CellPayload::Number(55.0)),
                    ],
                }),
                DocumentEvent::SheetEnd { sheet_id: 1 },
            ];
            let mut stream = formula_prelude();
            if name_first {
                // Position-free: the name precedes its target sheet (canonical
                // workbook-position, as the driver fixture emits DefinedName).
                stream.push(name_event);
                stream.extend(sheet_events);
            } else {
                // Out of order: the name arrives AFTER the whole sheet. The stream
                // validator leaves DefinedName position-free, so this is a legal
                // stream — and the deferred install must resolve it identically.
                stream.extend(sheet_events);
                stream.push(name_event);
            }
            stream
        };

        // Both orderings load and resolve =Anchor to B1 = 55.
        let resolve = |stream: &[DocumentEvent]| {
            let (mut context, workspace_id) = workbook_context();
            let report = load_workbook_events(&mut context, &workspace_id, stream).unwrap();
            assert_eq!(report.names, 1, "the name installs regardless of position");
            context.recalculate_workbook(&workspace_id).unwrap();
            published_value(&context, &workspace_id, 1, 1).unwrap().0
        };

        let name_first = resolve(&build(true));
        let name_last = resolve(&build(false));
        assert_eq!(
            name_first,
            CalcValue::number(55.0),
            "name-before-sheet resolves =Anchor to B1 = 55"
        );
        assert_eq!(
            name_last, name_first,
            "name-after-sheet loads to the SAME result — the deferred install does not \
             depend on stream ordering (D4 §9)"
        );
    }

    /// Defined-name metadata (Tier-B, row 26 B-half): a name carrying
    /// comment/hidden/function flags retains that metadata in the
    /// `IngestedDocumentFacts` store (the round-trip home), keyed by name — even
    /// though the calc binding is the Tier-A install. The report echoes the store.
    #[test]
    fn defined_name_metadata_is_retained_on_the_report() {
        let (mut context, workspace_id) = workbook_context();
        let mut stream = formula_prelude();
        let metadata = DefinedNameMetadataSpec {
            comment: Some("a documented name".to_string()),
            hidden: Some(true),
            ..DefinedNameMetadataSpec::default()
        };
        stream.extend([
            DocumentEvent::DefinedName(DefinedNameSpec {
                name: "Documented".to_string(),
                formula_text: "Sheet1!$A$1".to_string(),
                scope_sheet_id: None,
                metadata: metadata.clone(),
            }),
            DocumentEvent::SheetBegin(SheetRef {
                sheet_id: 1,
                name: "Sheet1".to_string(),
            }),
            DocumentEvent::CellChunk(CellChunk {
                row_band: 0,
                cells: vec![(addr(1, 1), CellPayload::Number(1.0))],
            }),
            DocumentEvent::SheetEnd { sheet_id: 1 },
        ]);

        let report = load_workbook_events(&mut context, &workspace_id, &stream).unwrap();
        assert_eq!(report.names, 1, "the name installs (Tier A)");
        assert_eq!(
            report.name_metadata.len(),
            1,
            "the name's metadata is retained (Tier B)"
        );
        assert_eq!(report.name_metadata[0].name, "Documented");
        assert_eq!(report.name_metadata[0].metadata, metadata);
        // The retention home is the STORE (a rect-less family, no overlay); the
        // report merely echoes it.
        let facts = context.ingested_document_facts(&workspace_id).unwrap();
        assert_eq!(facts.name_metadata, report.name_metadata);
        assert_eq!(facts.name_metadata[0].metadata, metadata);
    }

    /// A dynamic (non-rect-denoting) defined name binds its defining formula
    /// through the single mint (§3) and installs. `=SUM(Sheet1!$A$1:$A$2)` is not
    /// a bare rect, so it takes the dynamic lane; it resolves to the sum of its
    /// realized extent.
    #[test]
    fn dynamic_defined_name_binds_through_single_mint_and_resolves() {
        let (mut context, workspace_id) = workbook_context();
        let mut stream = formula_prelude();
        stream.extend([
            DocumentEvent::DefinedName(DefinedNameSpec {
                name: "Dyn".to_string(),
                // A function call over a range — NOT a bare rect, so it takes the
                // dynamic lane. Sheet-relative (unqualified) so it binds against
                // the anchor sheet's own cells (the anchor is the first sheet, the
                // only sheet here); a workbook-scoped dynamic name is visible
                // everywhere it is read.
                formula_text: "SUM(A1:A2)".to_string(),
                scope_sheet_id: None,
                metadata: DefinedNameMetadataSpec::default(),
            }),
            DocumentEvent::SheetBegin(SheetRef {
                sheet_id: 1,
                name: "Sheet1".to_string(),
            }),
            DocumentEvent::CellChunk(CellChunk {
                row_band: 0,
                // Ascending A1 order: A1, C1 (formula, still row 1), then A2.
                cells: vec![
                    (addr(1, 1), CellPayload::Number(4.0)),
                    (
                        // =SUM(Dyn): consume the dynamic name's realized extent
                        // (A1:A2) — a dynamic name binds an EXTENT, so a reducer
                        // over it reads the whole range, summing to 10.
                        addr(1, 3),
                        CellPayload::Formula {
                            region: None,
                            text: Some("SUM(Dyn)".to_string()),
                            cached: None,
                        },
                    ),
                    (addr(2, 1), CellPayload::Number(6.0)),
                ],
            }),
            DocumentEvent::SheetEnd { sheet_id: 1 },
        ]);

        let report = load_workbook_events(&mut context, &workspace_id, &stream).unwrap();
        assert_eq!(report.names, 1, "the dynamic name bound and installed");
        assert!(
            report.bind_degradations.is_empty(),
            "the dynamic defining formula binds through the single mint, got {:?}",
            report.bind_degradations
        );

        context.recalculate_workbook(&workspace_id).unwrap();
        let (value, _) = published_value(&context, &workspace_id, 1, 3).unwrap();
        assert_eq!(
            value,
            CalcValue::number(10.0),
            "=SUM(Dyn) resolves the dynamic name's extent A1:A2 = 4+6 = 10, got {value:?}"
        );
    }

    /// The A1 rect parser accepts `$`-anchored cells and ranges and rejects
    /// non-rect text (an expression, a bare unqualified ref). This guards the
    /// static-vs-dynamic decision the name install pivots on.
    #[test]
    fn rect_denoting_reference_parse_is_conservative() {
        // Rect-denoting: sheet-qualified cell / range, with or without `$`.
        let cell = parse_rect_denoting_reference("Sheet1!$A$1").unwrap();
        assert_eq!(cell.sheet, "Sheet1");
        assert_eq!(
            (cell.top_row, cell.left_col, cell.bottom_row, cell.right_col),
            (1, 1, 1, 1)
        );
        let range = parse_rect_denoting_reference("Sheet1!$B$2:$C$4").unwrap();
        assert_eq!(
            (
                range.top_row,
                range.left_col,
                range.bottom_row,
                range.right_col
            ),
            (2, 2, 4, 3)
        );
        // A leading `=` is tolerated.
        assert!(parse_rect_denoting_reference("=Sheet1!A1").is_some());
        // A quoted sheet name resolves and un-quotes.
        let quoted = parse_rect_denoting_reference("'My Sheet'!A1").unwrap();
        assert_eq!(quoted.sheet, "My Sheet");

        // NOT rect-denoting → dynamic lane:
        assert!(
            parse_rect_denoting_reference("Sheet1!A1+1").is_none(),
            "an expression is not a bare rect"
        );
        assert!(
            parse_rect_denoting_reference("SUM(Sheet1!A1:A2)").is_none(),
            "a function call is not a bare rect"
        );
        assert!(
            parse_rect_denoting_reference("A1").is_none(),
            "an UNqualified ref has no target sheet → dynamic"
        );
        assert!(
            parse_rect_denoting_reference("Sheet1!A1:A2,Sheet1!B1").is_none(),
            "a multi-area union is not a single rect"
        );
    }

    /// Review finding #1 (no-silent-loss for names): a defined name whose target
    /// sheet does NOT exist in the workbook (a `#REF!`-orphaned name from a
    /// deleted-sheet XLSX) is not installed — but the drop is SURFACED as a
    /// `BindDegradation` (retained text + reason), never silent. Names get the
    /// same honesty as formula cells (C13).
    ///
    /// MUTATION-CONFIRMATION: the assertions below fail if the drop is silent —
    /// `report.names` would still exclude the name (as before the fix) but
    /// `bind_degradations` would be empty, so the "not silent" assertion catches
    /// exactly the regression the fix prevents. (Deleting the `dropped_installs`
    /// push in `resolve_deferred_installs` reproduces the silent-loss bug: the
    /// name vanishes with `report.names == 1` real name and an empty degradation
    /// list — this test then FAILS on the degradation assertions.)
    #[test]
    fn name_with_unresolvable_target_sheet_is_surfaced_not_silently_dropped() {
        let (mut context, workspace_id) = workbook_context();
        let mut stream = formula_prelude();
        stream.extend([
            // `BadRef` targets sheet "Missing", which never appears in the stream.
            DocumentEvent::DefinedName(DefinedNameSpec {
                name: "BadRef".to_string(),
                formula_text: "Missing!$A$1".to_string(),
                scope_sheet_id: None,
                metadata: DefinedNameMetadataSpec::default(),
            }),
            // A well-formed name on the SAME stream proves selective accounting:
            // `Good` installs; only `BadRef` is degraded.
            DocumentEvent::DefinedName(DefinedNameSpec {
                name: "Good".to_string(),
                formula_text: "Sheet1!$A$1".to_string(),
                scope_sheet_id: None,
                metadata: DefinedNameMetadataSpec::default(),
            }),
            DocumentEvent::SheetBegin(SheetRef {
                sheet_id: 1,
                name: "Sheet1".to_string(),
            }),
            DocumentEvent::CellChunk(CellChunk {
                row_band: 0,
                cells: vec![(addr(1, 1), CellPayload::Number(1.0))],
            }),
            DocumentEvent::SheetEnd { sheet_id: 1 },
        ]);

        // Ingest SUCCEEDS — an orphaned name never fails the load.
        let report = load_workbook_events(&mut context, &workspace_id, &stream).unwrap();

        // Only `Good` installed into the calc model; `BadRef` did not.
        assert_eq!(
            report.names, 1,
            "only the resolvable name installs (BadRef's target sheet is absent)"
        );

        // The drop is SURFACED, not silent: a `name:BadRef` degradation carries the
        // retained text + the honest reason.
        let bad = report
            .bind_degradations
            .iter()
            .find(|row| row.address == "name:BadRef")
            .expect("BadRef's drop is accounted as a BindDegradation, not silently lost");
        assert_eq!(
            bad.text, "=Missing!$A$1",
            "the name's text is retained verbatim"
        );
        assert!(
            bad.diagnostics.iter().any(|d| d.contains("Missing")),
            "the degradation names the unresolvable target sheet, got {:?}",
            bad.diagnostics
        );
        // `Good` is NOT degraded — the accounting is selective, not blanket.
        assert!(
            report
                .bind_degradations
                .iter()
                .all(|row| row.address != "name:Good"),
            "the resolvable name is not degraded"
        );
    }

    /// Review finding #2 (surfaced-limitation branch, chosen because the engine's
    /// `set_sheet_defined_name` / `set_defined_name` call `check_rect`, which
    /// rejects a rect whose sheet_id ≠ the authoring sheet — verified at
    /// optimized_sheet.rs / calc_ref_sheet.rs): a sheet-scoped name whose scope
    /// sheet differs from its static rect's sheet cannot be modeled faithfully, so
    /// it is NOT silently re-scoped to the target sheet — the limitation is
    /// SURFACED via the degradation channel. A scope change must never be silent.
    #[test]
    fn sheet_scoped_name_with_cross_sheet_rect_is_surfaced_not_silently_rescoped() {
        let (mut context, workspace_id) = workbook_context();
        let mut stream = formula_prelude();
        stream.extend([
            // `Widget` is SHEET-scoped to Sheet1 (scope_sheet_id: Some(1)) but its
            // static rect targets Sheet2. The engine cannot author a Sheet1-scoped
            // name whose rect lives on Sheet2, so this must be surfaced, not
            // silently re-scoped to Sheet2.
            DocumentEvent::DefinedName(DefinedNameSpec {
                name: "Widget".to_string(),
                formula_text: "Sheet2!$A$1".to_string(),
                scope_sheet_id: Some(1),
                metadata: DefinedNameMetadataSpec::default(),
            }),
            DocumentEvent::SheetBegin(SheetRef {
                sheet_id: 1,
                name: "Sheet1".to_string(),
            }),
            DocumentEvent::CellChunk(CellChunk {
                row_band: 0,
                cells: vec![(addr(1, 1), CellPayload::Number(1.0))],
            }),
            DocumentEvent::SheetEnd { sheet_id: 1 },
            DocumentEvent::SheetBegin(SheetRef {
                sheet_id: 2,
                name: "Sheet2".to_string(),
            }),
            DocumentEvent::CellChunk(CellChunk {
                row_band: 0,
                cells: vec![(addr(1, 1), CellPayload::Number(9.0))],
            }),
            DocumentEvent::SheetEnd { sheet_id: 2 },
        ]);

        let report = load_workbook_events(&mut context, &workspace_id, &stream).unwrap();

        // The name is NOT installed (neither on Sheet1 nor silently on Sheet2).
        assert_eq!(
            report.names, 0,
            "the cross-sheet-rect sheet-scoped name is not installed"
        );
        // The limitation is SURFACED: a `name:Widget` degradation names both the
        // scope sheet and the target sheet and the engine limitation.
        let widget = report
            .bind_degradations
            .iter()
            .find(|row| row.address == "name:Widget")
            .expect("the scope≠target limitation is surfaced, not silent");
        assert_eq!(widget.text, "=Sheet2!$A$1");
        assert!(
            widget.diagnostics.iter().any(|d| d.contains("Sheet1"))
                && widget.diagnostics.iter().any(|d| d.contains("Sheet2")),
            "the degradation names both the scope sheet and the target sheet, got {:?}",
            widget.diagnostics
        );

        // Mutation-confirmation that the re-scope is genuinely avoided: `Widget` is
        // NOT authored on Sheet2 (a silent re-scope would put it there). Read
        // Sheet2's authored names via the document readout.
        let sheet2 = sheet_node(&context, &workspace_id, 1);
        let names_on_sheet2 = context
            .document_defined_names(&workspace_id, sheet2)
            .unwrap();
        assert!(
            names_on_sheet2.iter().all(|n| n.name != "Widget"),
            "Widget must NOT leak onto Sheet2 (no silent re-scope), got {names_on_sheet2:?}"
        );
    }

    /// Table no-silent-loss (same doctrine as the name drops): a `TableOverlay`
    /// whose `range` string is not parseable A1 (a malformed / alternative-producer
    /// range — `TableSpec.range` is a raw String the validator does NOT check) is
    /// not installed, but the drop is SURFACED as a `table:{name}` degradation
    /// (retained range text + reason), never silent (C13).
    ///
    /// MUTATION-CONFIRMATION: the `expect` on the `table:BadTable` degradation
    /// fails if the parse-fail path reverts to a bare `continue` — the table then
    /// vanishes with `report.tables == 0` and an empty degradation list, so this
    /// test catches exactly the silent-drop regression the fix closes.
    #[test]
    fn table_with_unparseable_range_is_surfaced_not_silently_dropped() {
        let (mut context, workspace_id) = workbook_context();
        let mut stream = formula_prelude();
        stream.extend([
            DocumentEvent::SheetBegin(SheetRef {
                sheet_id: 1,
                name: "Sheet1".to_string(),
            }),
            // A well-formed table proves selective accounting: `GoodTable`
            // installs; only `BadTable` is degraded.
            DocumentEvent::TableOverlay(TableSpec {
                name: "GoodTable".to_string(),
                sheet_id: 1,
                range: "A1:A2".to_string(),
            }),
            // `BadTable` carries a range string that is not parseable A1.
            DocumentEvent::TableOverlay(TableSpec {
                name: "BadTable".to_string(),
                sheet_id: 1,
                range: "not-a-range".to_string(),
            }),
            DocumentEvent::CellChunk(CellChunk {
                row_band: 0,
                cells: vec![(addr(1, 1), CellPayload::Number(1.0))],
            }),
            DocumentEvent::SheetEnd { sheet_id: 1 },
        ]);

        // Ingest SUCCEEDS — a malformed table range never fails the load.
        let report = load_workbook_events(&mut context, &workspace_id, &stream).unwrap();

        // Only the well-formed table installed; `BadTable` did not.
        assert_eq!(
            report.tables, 1,
            "only the parseable table installs (BadTable's range is unparseable)"
        );

        // The drop is SURFACED, not silent: a `table:BadTable` degradation carries
        // the retained range text + the honest reason.
        let bad = report
            .bind_degradations
            .iter()
            .find(|row| row.address == "table:BadTable")
            .expect("BadTable's drop is accounted as a BindDegradation, not silently lost");
        assert_eq!(
            bad.text, "not-a-range",
            "the range string is retained verbatim"
        );
        assert!(
            bad.diagnostics.iter().any(|d| d.contains("not-a-range")),
            "the degradation names the unparseable range, got {:?}",
            bad.diagnostics
        );
        // `GoodTable` is NOT degraded — the accounting is selective, not blanket.
        assert!(
            report
                .bind_degradations
                .iter()
                .all(|row| row.address != "table:GoodTable"),
            "the well-formed table is not degraded"
        );
    }

    // ==== R6.4: Tier-B inert store + overlay seats + digest meta-child =========

    use crate::grid::coords::ExcelGridBounds;
    use crate::grid::machine::{OverlayKind, SpillBlock};
    use oxdoc_model::{
        ConditionalFormatRule, ConditionalFormatRuleType, DrawingAnchor, DrawingAnchorEditAs,
        DrawingAnchorKind, DrawingAnchorMarker, DrawingObjectKind, DrawingObjectProvenance,
        DrawingObjectSpec, EmuSize, FormControlProperties, FormControlSpec, FormControlType,
        HyperlinkSpec, ThreadedCommentPersonSpec,
    };

    /// A minimal prelude for a single-sheet Tier-B fixture: header + empty string
    /// table + a real (minimal) style table, then one `SheetBegin`. Cell content
    /// is up to the caller; `SheetEnd` too.
    fn tier_b_prelude() -> Vec<DocumentEvent> {
        vec![
            DocumentEvent::WorkbookHeader(WorkbookHeader::new(
                DocDateSystem::Date1900,
                DocCalcMode::Automatic,
            )),
            DocumentEvent::StringTable(Vec::new()),
            DocumentEvent::StyleTable(StyleTableSpec::minimal()),
        ]
    }

    /// The canonical two-cell drawing anchor from OxDoc's driver fixture
    /// (`from{row:0,col:1}` .. `to{row:4,col:3}`, both zero-based). Its one-based
    /// inclusive rect is `(1, 2, 5, 4)`.
    fn canonical_drawing_anchor() -> DrawingAnchor {
        DrawingAnchor {
            kind: DrawingAnchorKind::TwoCell,
            from: Some(DrawingAnchorMarker {
                row: 0,
                col: 1,
                row_offset_emu: 12_700,
                col_offset_emu: 25_400,
            }),
            to: Some(DrawingAnchorMarker {
                row: 4,
                col: 3,
                row_offset_emu: 0,
                col_offset_emu: 0,
            }),
            position: None,
            extents: Some(EmuSize {
                cx: 914_400,
                cy: 457_200,
            }),
            edit_as: Some(DrawingAnchorEditAs::TwoCell),
            raw_attrs: Vec::new(),
        }
    }

    /// A canonical cell-anchored form-control host (mirrors OxDoc's fixture shape:
    /// a `FormControlHost` drawing object with a two-cell anchor + one Button
    /// control). The anchor drives the inert `RichObject` overlay rect.
    fn canonical_drawing_form_controls(sheet_id: u32) -> DrawingFormControlsSpec {
        DrawingFormControlsSpec {
            sheet_id,
            drawing_layer_id: Some("drawing-layer-1".to_string()),
            objects: vec![DrawingObjectSpec {
                sheet_id,
                object_id: "shape-1".to_string(),
                source_object_id: Some("1025".to_string()),
                kind: DrawingObjectKind::FormControlHost,
                name: Some("Button 1".to_string()),
                description: None,
                alt_text: None,
                label_text: Some("Run".to_string()),
                anchor: canonical_drawing_anchor(),
                linked_control_id: Some("ctrl-1".to_string()),
                provenance: DrawingObjectProvenance::Modeled,
                notices: Vec::new(),
            }],
            controls: vec![FormControlSpec {
                sheet_id,
                control_id: "ctrl-1".to_string(),
                name: Some("Button 1".to_string()),
                code_name: Some("Button1".to_string()),
                control_type: FormControlType::Button,
                source_shape_id: Some("1025".to_string()),
                shape_object_id: Some("shape-1".to_string()),
                properties: FormControlProperties::default(),
                notices: Vec::new(),
            }],
            notices: Vec::new(),
        }
    }

    /// A canonical conditional-format region over `B2:C3` (a two-cell range) with
    /// one expression rule. The parsed `ranges` drive the inert
    /// `ConditionalFormat` overlay rect `(2, 2, 3, 3)`.
    fn canonical_conditional_format(sheet_id: u32) -> ConditionalFormatRegion {
        ConditionalFormatRegion {
            sheet_id,
            sqref: "B2:C3".to_string(),
            ranges: vec![CellRangeSpec {
                text: "B2:C3".to_string(),
                start: PackedCellAddr::from_one_based(2, 2).unwrap(),
                end: PackedCellAddr::from_one_based(3, 3).unwrap(),
            }],
            pivot: false,
            rules: vec![ConditionalFormatRule {
                rule_type: ConditionalFormatRuleType::Expression,
                priority: Some(1),
                dxf_id: Some(0),
                stop_if_true: false,
                operator: None,
                text: None,
                time_period: None,
                rank: None,
                std_dev: None,
                above_average: None,
                percent: None,
                bottom: None,
                equal_average: None,
                formulas: vec!["$B2>10".to_string()],
                color_scale: None,
                data_bar: None,
                icon_set: None,
                raw_attrs: Vec::new(),
                raw_children: Vec::new(),
            }],
        }
    }

    /// A one-sheet stream carrying a rich spread of Tier-B facts: a style table
    /// (rect-less), threaded-comment people (rect-less), a hyperlink (rect-less),
    /// a conditional format (rect-claiming), and a cell-anchored form control
    /// (rect-claiming). One literal cell keeps the sheet non-empty.
    fn tier_b_rich_stream() -> Vec<DocumentEvent> {
        let mut stream = tier_b_prelude();
        stream.extend([
            DocumentEvent::ThreadedCommentPeople(ThreadedCommentPeopleSpec {
                people: vec![ThreadedCommentPersonSpec {
                    person_id: "person-1".to_string(),
                    display_name: Some("Ada Lovelace".to_string()),
                    provider_id: Some("AD".to_string()),
                    user_id: Some("ada@example.com".to_string()),
                    raw_attrs: Vec::new(),
                }],
                notices: Vec::new(),
                raw_attrs: Vec::new(),
            }),
            DocumentEvent::SheetBegin(SheetRef {
                sheet_id: 1,
                name: "Sheet1".to_string(),
            }),
            DocumentEvent::Hyperlinks(HyperlinksSpec {
                sheet_id: 1,
                links: vec![HyperlinkSpec {
                    sheet_id: 1,
                    reference: CellRangeSpec {
                        text: "A1".to_string(),
                        start: PackedCellAddr::from_one_based(1, 1).unwrap(),
                        end: PackedCellAddr::from_one_based(1, 1).unwrap(),
                    },
                    relationship_id: Some("rId1".to_string()),
                    target: Some("https://example.com".to_string()),
                    target_mode: None,
                    location: None,
                    display: Some("Example".to_string()),
                    tooltip: None,
                    raw_attrs: Vec::new(),
                }],
            }),
            DocumentEvent::DrawingFormControls(canonical_drawing_form_controls(1)),
            DocumentEvent::ConditionalFormatRegion(canonical_conditional_format(1)),
            DocumentEvent::CellChunk(CellChunk {
                row_band: 0,
                cells: vec![(a1(), CellPayload::Number(1.0))],
            }),
            DocumentEvent::SheetEnd { sheet_id: 1 },
        ]);
        stream
    }

    // ---- Acceptance: digest drives identity (with mutation-check) -------------

    /// Two loads of IDENTICAL Tier-B facts digest identically, and the stored
    /// `#workbook-ingest/facts_digest` equals the store's own digest. Perturbing
    /// ONE retained fact (a single style-table field) moves the digest AND the
    /// stored digest text — the store's identity contribution tracks its content.
    #[test]
    fn store_digest_drives_identity_and_a_perturbed_fact_moves_it() {
        // Load A and load A' (a byte-identical rebuild of the same stream) into
        // two fresh workspaces: the digests must be equal.
        let (mut ctx_a, ws_a) = workbook_context();
        let report_a = load_workbook_events(&mut ctx_a, &ws_a, &tier_b_rich_stream()).unwrap();
        let facts_a = ctx_a.ingested_document_facts(&ws_a).unwrap();

        let mut ctx_b = OxCalcDocumentContext::default();
        let ws_b = ctx_b
            .create_workspace(OxCalcTreeWorkspaceCreate::new("workbook:ingest-b").as_workbook())
            .unwrap();
        let _report_b = load_workbook_events(&mut ctx_b, &ws_b, &tier_b_rich_stream()).unwrap();
        let facts_b = ctx_b.ingested_document_facts(&ws_b).unwrap();

        assert_eq!(
            facts_a.digest(),
            facts_b.digest(),
            "identical Tier-B facts digest identically"
        );
        // The stored `#workbook-ingest/facts_digest` equals the store's digest by
        // construction (the identity-bearing text), and is present (facts exist).
        let stored_a = ctx_a.workbook_ingest_facts_digest(&ws_a).unwrap();
        assert_eq!(
            stored_a.as_deref(),
            Some(facts_a.digest().as_str()),
            "the meta-child stores exactly the store's digest"
        );
        assert_eq!(
            ctx_b.workbook_ingest_facts_digest(&ws_b).unwrap(),
            stored_a,
            "identical facts ⇒ identical stored digest text"
        );
        // Sanity: the report saw the rich Tier-B facts (not an empty load).
        assert!(!facts_a.is_empty(), "the store retained the Tier-B facts");
        assert_eq!(report_a.inert_overlays.len(), 2, "CF + RichObject rects");

        // MUTATION-CHECK: perturb ONE retained fact — swap the style table for a
        // structurally different one (an extra number-format entry). Load into a
        // fresh workspace and assert the digest AND the stored text MOVE.
        let mut perturbed_stream = tier_b_rich_stream();
        let mut perturbed_styles = StyleTableSpec::minimal();
        perturbed_styles
            .number_formats
            .push(oxdoc_model::NumberFormatSpec {
                num_fmt_id: 164,
                format_code: "0.000".to_string(),
            });
        // The StyleTable is the 3rd prelude event (index 2).
        perturbed_stream[2] = DocumentEvent::StyleTable(perturbed_styles);

        let mut ctx_c = OxCalcDocumentContext::default();
        let ws_c = ctx_c
            .create_workspace(OxCalcTreeWorkspaceCreate::new("workbook:ingest-c").as_workbook())
            .unwrap();
        let _report_c = load_workbook_events(&mut ctx_c, &ws_c, &perturbed_stream).unwrap();
        let facts_c = ctx_c.ingested_document_facts(&ws_c).unwrap();

        assert_ne!(
            facts_a.digest(),
            facts_c.digest(),
            "perturbing one retained style field MUST move the store digest"
        );
        assert_ne!(
            ctx_c.workbook_ingest_facts_digest(&ws_c).unwrap(),
            stored_a,
            "the moved digest is reflected in the stored `#workbook-ingest` text"
        );
    }

    /// The load-time digest moves the workspace REVISION identity. Two loads into
    /// same-id fresh workspaces that differ in EXACTLY ONE retained Tier-B fact
    /// (an extra data-validation) — everything else (workspace id, string table,
    /// styles, sheet, cell) identical — produce different revision ids, and that
    /// divergence traces to the `#workbook-ingest` digest (the stored digests
    /// differ). The empty-store case (proving an all-empty store writes NO
    /// subtree) is covered by `create_workspace`'s default and asserted below.
    #[test]
    fn ingest_digest_contributes_to_workspace_revision_identity() {
        // The baseline: prelude (with its style table) + one sheet + one cell.
        let base_stream = {
            let mut stream = tier_b_prelude();
            stream.extend([
                DocumentEvent::SheetBegin(SheetRef {
                    sheet_id: 1,
                    name: "Sheet1".to_string(),
                }),
                DocumentEvent::CellChunk(CellChunk {
                    row_band: 0,
                    cells: vec![(a1(), CellPayload::Number(1.0))],
                }),
                DocumentEvent::SheetEnd { sheet_id: 1 },
            ]);
            stream
        };
        // The SAME stream plus exactly one extra rect-less Tier-B fact — the only
        // difference, so any revision-id divergence is the ingest digest's.
        let with_extra_fact = {
            let mut stream = tier_b_prelude();
            stream.extend([
                DocumentEvent::SheetBegin(SheetRef {
                    sheet_id: 1,
                    name: "Sheet1".to_string(),
                }),
                DocumentEvent::DataValidations(DataValidationsSpec {
                    sheet_id: 1,
                    disable_prompts: None,
                    x_window: None,
                    y_window: None,
                    regions: Vec::new(),
                    raw_attrs: vec![oxdoc_model::XmlAttrSpec {
                        name: "count".to_string(),
                        value: "1".to_string(),
                    }],
                }),
                DocumentEvent::CellChunk(CellChunk {
                    row_band: 0,
                    cells: vec![(a1(), CellPayload::Number(1.0))],
                }),
                DocumentEvent::SheetEnd { sheet_id: 1 },
            ]);
            stream
        };

        let (mut ctx_base, ws_base) = workbook_context();
        load_workbook_events(&mut ctx_base, &ws_base, &base_stream).unwrap();
        let base_revision = ctx_base
            .workspace_view(&ws_base)
            .unwrap()
            .workspace_revision_id;
        let base_digest = ctx_base.workbook_ingest_facts_digest(&ws_base).unwrap();

        let mut ctx_extra = OxCalcDocumentContext::default();
        let ws_extra = ctx_extra
            .create_workspace(
                // Same workspace id so the workspace-id identity component matches;
                // the ONLY difference is the ingest digest.
                OxCalcTreeWorkspaceCreate::new("workbook:ingest").as_workbook(),
            )
            .unwrap();
        load_workbook_events(&mut ctx_extra, &ws_extra, &with_extra_fact).unwrap();
        let extra_revision = ctx_extra
            .workspace_view(&ws_extra)
            .unwrap()
            .workspace_revision_id;
        let extra_digest = ctx_extra.workbook_ingest_facts_digest(&ws_extra).unwrap();

        // Both loads carry a style table, so both write an ingest subtree — but
        // the extra data-validation moves the digest, and hence the revision id.
        assert!(
            base_digest.is_some(),
            "the base load writes an ingest digest"
        );
        assert!(
            extra_digest.is_some(),
            "the extra load writes an ingest digest"
        );
        assert_ne!(
            base_digest, extra_digest,
            "one extra retained fact moves the stored digest"
        );
        assert_ne!(
            base_revision, extra_revision,
            "the moved ingest digest moves the workspace revision identity"
        );

        // A workspace that never loaded a document carries the empty store ⇒ NO
        // ingest subtree (the empty-store discipline: absent means default).
        let (empty_ctx, empty_ws) = workbook_context();
        assert_eq!(
            empty_ctx.workbook_ingest_facts_digest(&empty_ws).unwrap(),
            None,
            "an un-loaded workspace writes no ingest subtree"
        );
    }

    // ---- Acceptance: overlay readout (CF / RichObject / Cse rects) ------------

    /// The rect-claiming families project inert `GridOverlayExtension` seats at
    /// the correct rects, with store-key payloads and inert block/admission. The
    /// store — not the overlay — is the retention home, so this is a readout OFF
    /// the store (`overlay_seats_for_sheet`), keyed to the canonical rects.
    #[test]
    fn overlay_seats_project_cf_and_richobject_rects_with_store_keys() {
        let (mut context, workspace_id) = workbook_context();
        let report =
            load_workbook_events(&mut context, &workspace_id, &tier_b_rich_stream()).unwrap();
        let facts = context.ingested_document_facts(&workspace_id).unwrap();

        // Two rect-claiming families on sheet 1: one CF region (B2:C3) and one
        // cell-anchored form control (from{0,1}..to{4,3} ⇒ B1:D5 one-based).
        let seats = facts.overlay_seats_for_sheet(
            1,
            "book:test",
            "sheet:test",
            ExcelGridBounds::strict_excel(),
        );
        assert_eq!(
            seats.len(),
            2,
            "one CF + one RichObject seat, got {seats:?}"
        );

        let cf = seats
            .iter()
            .find(|seat| seat.kind_tag == OverlayKind::ConditionalFormat)
            .expect("a ConditionalFormat seat");
        assert_eq!(
            (
                cf.claimed_rect.top_row,
                cf.claimed_rect.left_col,
                cf.claimed_rect.bottom_row,
                cf.claimed_rect.right_col
            ),
            (2, 2, 3, 3),
            "the CF rect claims B2:C3"
        );
        assert_eq!(cf.payload, "cf:1#0", "the CF payload is its store key");
        // Block / admission are inert (the overlay is a spatial index, not an
        // engine-active blocker or axis-edit refuser).
        assert_eq!(
            cf.block_mode,
            SpillBlock::None,
            "CF seat does not block spills"
        );
        assert!(!cf.refuses_axis_edit, "CF seat admits axis edits (inert)");

        let rich = seats
            .iter()
            .find(|seat| seat.kind_tag == OverlayKind::RichObject)
            .expect("a RichObject seat");
        assert_eq!(
            (
                rich.claimed_rect.top_row,
                rich.claimed_rect.left_col,
                rich.claimed_rect.bottom_row,
                rich.claimed_rect.right_col
            ),
            (1, 2, 5, 4),
            "the RichObject rect claims the two-cell anchor B1:D5"
        );
        assert_eq!(
            rich.payload, "rich:1#0",
            "the RichObject payload is its store key"
        );
        assert_eq!(rich.block_mode, SpillBlock::None);
        assert!(!rich.refuses_axis_edit);

        // The report's inert_overlays echo the same two claims with the same keys.
        assert_eq!(report.inert_overlays.len(), 2);
        let payloads: std::collections::BTreeSet<&str> = report
            .inert_overlays
            .iter()
            .map(|overlay| overlay.payload.as_str())
            .collect();
        assert!(payloads.contains("cf:1#0") && payloads.contains("rich:1#0"));
    }

    /// A legacy-CSE array claims an inert `Cse` seat at its array rect (the array
    /// cells still bind as normal formulas). The seat carries the `cse:` store
    /// key and inert block/admission — the canonical rect-claiming CSE shape.
    #[test]
    fn overlay_seats_project_cse_rect_with_store_key() {
        let (mut context, workspace_id) = workbook_context();
        let array_range = CellRangeSpec {
            text: "A1:A2".to_string(),
            start: PackedCellAddr::from_one_based(1, 1).unwrap(),
            end: PackedCellAddr::from_one_based(2, 1).unwrap(),
        };
        let mut stream = tier_b_prelude();
        stream.extend([
            DocumentEvent::SheetBegin(SheetRef {
                sheet_id: 1,
                name: "Sheet1".to_string(),
            }),
            DocumentEvent::FormulaTopology(FormulaTopology {
                sheet_id: 1,
                records: vec![FormulaRecord {
                    sheet_id: 1,
                    address: addr(1, 1),
                    kind: FormulaRecordKind::Array(ArrayFormulaSpec {
                        range: Some(array_range),
                        always_calculate: None,
                    }),
                    text: Some("SUM(C1:C2)".to_string()),
                    text_kind: FormulaTextKind::SpreadsheetMlA1,
                    cached_value: FormulaCachedValueState::Missing,
                    attrs: FormulaRecordAttributes::normal(),
                    unsupported_fragments: Vec::new(),
                }],
                unsupported_fragments: Vec::new(),
            }),
            DocumentEvent::CellChunk(CellChunk {
                row_band: 0,
                cells: vec![
                    (
                        addr(1, 1),
                        CellPayload::Formula {
                            region: None,
                            text: Some("SUM(C1:C2)".to_string()),
                            cached: Some(Box::new(CellPayload::Number(0.0))),
                        },
                    ),
                    (addr(1, 3), CellPayload::Number(4.0)),
                    (addr(2, 3), CellPayload::Number(6.0)),
                ],
            }),
            DocumentEvent::SheetEnd { sheet_id: 1 },
        ]);
        let report = load_workbook_events(&mut context, &workspace_id, &stream).unwrap();
        let facts = context.ingested_document_facts(&workspace_id).unwrap();

        let seats = facts.overlay_seats_for_sheet(
            1,
            "book:test",
            "sheet:test",
            ExcelGridBounds::strict_excel(),
        );
        assert_eq!(seats.len(), 1, "one Cse seat");
        assert_eq!(seats[0].kind_tag, OverlayKind::Cse);
        assert_eq!(
            (
                seats[0].claimed_rect.top_row,
                seats[0].claimed_rect.left_col,
                seats[0].claimed_rect.bottom_row,
                seats[0].claimed_rect.right_col
            ),
            (1, 1, 2, 1),
            "the Cse rect claims A1:A2"
        );
        assert_eq!(seats[0].payload, "cse:1#0");
        assert_eq!(seats[0].block_mode, SpillBlock::None);
        assert!(!seats[0].refuses_axis_edit);
        // The whole FormulaTopology was also retained verbatim in the store
        // (attrs + unsupported fragments round-trip), not just the overlay rect.
        assert_eq!(
            facts.formula_topologies.len(),
            1,
            "the FormulaTopology is retained verbatim for round-trip"
        );
        assert_eq!(report.formulas_bound, 1, "the array anchor bound normally");
    }

    // ---- Acceptance: store survives undo/redo BY POINTER ----------------------

    /// The inert store survives revision navigation BY POINTER: after load, the
    /// live `Arc` is retained on the load revision; navigating to the pre-load
    /// (creation) revision swaps in the empty default store, and navigating back
    /// to the load revision restores the VERY SAME `Arc` (pointer identity), not
    /// a rebuilt store. This is the immutable-store retention contract (D4 §13).
    #[test]
    fn store_survives_undo_redo_by_pointer() {
        let (mut context, workspace_id) = workbook_context();
        let creation_revision = context
            .workspace_view(&workspace_id)
            .unwrap()
            .workspace_revision_id;

        load_workbook_events(&mut context, &workspace_id, &tier_b_rich_stream()).unwrap();
        let load_revision = context
            .workspace_view(&workspace_id)
            .unwrap()
            .workspace_revision_id;
        let facts_at_load = context.ingested_document_facts(&workspace_id).unwrap();
        assert!(!facts_at_load.is_empty(), "the load populated the store");

        // Navigate BACK to the creation revision: the store reverts to the empty
        // default (a DIFFERENT Arc — the load's store did not exist pre-load).
        context
            .navigate_workspace_revision(&workspace_id, &creation_revision)
            .unwrap();
        let facts_at_creation = context.ingested_document_facts(&workspace_id).unwrap();
        assert!(
            facts_at_creation.is_empty(),
            "the pre-load revision carries the empty store"
        );
        assert!(
            !std::sync::Arc::ptr_eq(&facts_at_load, &facts_at_creation),
            "creation and load hold distinct stores"
        );

        // Navigate FORWARD to the load revision: the SAME Arc is restored — the
        // store survives by pointer, never rebuilt.
        context
            .navigate_workspace_revision(&workspace_id, &load_revision)
            .unwrap();
        let facts_after_redo = context.ingested_document_facts(&workspace_id).unwrap();
        assert!(
            std::sync::Arc::ptr_eq(&facts_at_load, &facts_after_redo),
            "navigating back to the load revision restores the SAME store Arc (pointer identity)"
        );
        assert_eq!(
            facts_after_redo.digest(),
            facts_at_load.digest(),
            "and its digest is unchanged"
        );
    }

    // ---- Acceptance: rect-less family retention (store is the home) -----------

    /// A workbook whose ONLY Tier-B facts are rect-LESS (a style table + a
    /// threaded-comment person, no rect) retains them in the store with NO
    /// overlay — proving the store, not the overlay, is the retention home (D4
    /// §13). A ledger row alone would be a silent loss at save; the typed store
    /// closes that gap.
    #[test]
    fn rect_less_families_retained_in_store_with_no_overlay() {
        let (mut context, workspace_id) = workbook_context();
        let mut stream = tier_b_prelude();
        stream.extend([
            DocumentEvent::ThreadedCommentPeople(ThreadedCommentPeopleSpec {
                people: vec![ThreadedCommentPersonSpec {
                    person_id: "person-1".to_string(),
                    display_name: Some("Grace Hopper".to_string()),
                    provider_id: None,
                    user_id: Some("grace@example.com".to_string()),
                    raw_attrs: Vec::new(),
                }],
                notices: Vec::new(),
                raw_attrs: Vec::new(),
            }),
            DocumentEvent::SheetBegin(SheetRef {
                sheet_id: 1,
                name: "Sheet1".to_string(),
            }),
            DocumentEvent::CellChunk(CellChunk {
                row_band: 0,
                cells: vec![(a1(), CellPayload::Number(1.0))],
            }),
            DocumentEvent::SheetEnd { sheet_id: 1 },
        ]);
        let report = load_workbook_events(&mut context, &workspace_id, &stream).unwrap();
        let facts = context.ingested_document_facts(&workspace_id).unwrap();

        // The rect-less families ARE retained in the typed store.
        assert!(facts.style_table.is_some(), "the style table is retained");
        assert_eq!(
            facts.threaded_comment_people.len(),
            1,
            "the threaded-comment people are retained"
        );
        assert_eq!(
            facts.threaded_comment_people[0].people[0]
                .display_name
                .as_deref(),
            Some("Grace Hopper"),
            "the person's real shape is retained, not a lossy summary"
        );
        // NO overlay was claimed for these rect-less families.
        assert!(
            facts.inert_overlays.is_empty(),
            "rect-less families claim NO overlay — the store is the home"
        );
        assert!(report.inert_overlays.is_empty());
        // The store is non-empty (retention happened), so the digest subtree was
        // written (the rect-less retention still participates in identity).
        assert!(!facts.is_empty());
        assert!(
            context
                .workbook_ingest_facts_digest(&workspace_id)
                .unwrap()
                .is_some(),
            "rect-less-only retention still writes the ingest digest"
        );
    }

    /// The unknown BIFF error byte (R6.1) is retained in the store at its cell:
    /// the cell publishes `#VALUE!`, but the raw byte survives for a verbatim
    /// round-trip — a no-silent-loss retention the store now owns.
    #[test]
    fn unknown_error_byte_retained_in_store_at_its_cell() {
        let (mut context, workspace_id) = workbook_context();
        let mut stream = tier_b_prelude();
        stream.extend([
            DocumentEvent::SheetBegin(SheetRef {
                sheet_id: 1,
                name: "Sheet1".to_string(),
            }),
            DocumentEvent::CellChunk(CellChunk {
                row_band: 0,
                // 0xFF has no classic BIFF mapping ⇒ published #VALUE!, byte retained.
                cells: vec![(addr(2, 3), CellPayload::Error(0xFF))],
            }),
            DocumentEvent::SheetEnd { sheet_id: 1 },
        ]);
        load_workbook_events(&mut context, &workspace_id, &stream).unwrap();
        let facts = context.ingested_document_facts(&workspace_id).unwrap();

        assert_eq!(
            facts.unknown_error_bytes.len(),
            1,
            "the raw byte is retained"
        );
        let retained = facts.unknown_error_bytes[0];
        assert_eq!(retained.sheet_id, 1);
        assert_eq!((retained.row, retained.col), (2, 3), "at its cell C2");
        assert_eq!(retained.raw_byte, 0xFF, "the raw byte survives verbatim");
    }

    /// Every Tier-B variant in the 29-variant stream now carries REAL retention
    /// (a `retained-inert*` disposition), not the old `retained-inert-stub`. The
    /// no-silent-loss invariant still holds and the ledger is still 29 rows.
    #[test]
    fn all_tier_b_variants_carry_real_retention_not_a_stub() {
        let (mut context, workspace_id) = workbook_context();
        let mut sink = OxCalcWorkbookIngestSink::new();
        oxdoc_model::drive_oxcalc_ingest(&all_variant_stream(), &mut sink).unwrap();
        let observed = sink.observed().to_vec();
        let report = sink.commit_into(&mut context, &workspace_id).unwrap();

        // The invariant still holds: every observed variant is accounted.
        assert_eq!(report.accounts_for_all_variants(&observed), Ok(()));
        assert_eq!(report.ledger.len(), 29);

        // No Tier-B row is a stub anymore: every Tier-B disposition starts with
        // `retained-inert` (the real store), never `retained-inert-stub`.
        for row in &report.ledger {
            if row.tier == IngestTier::B {
                assert!(
                    row.disposition.starts_with("retained-inert")
                        && row.disposition != "retained-inert-stub",
                    "Tier-B variant {:?} must carry REAL retention, got {:?}",
                    row.variant,
                    row.disposition
                );
            }
        }
        // The store actually retained the variants the 29-stream carried (spot
        // check the rect-less style table + the rect-less external link + the
        // opaque notice + the differential styles).
        let facts = context.ingested_document_facts(&workspace_id).unwrap();
        assert!(facts.style_table.is_some(), "StyleTable retained");
        assert_eq!(facts.external_links.len(), 1, "ExternalLink retained");
        assert_eq!(facts.opaque_notices.len(), 1, "OpaquePartNotice retained");
        assert_eq!(facts.differential_styles.len(), 1, "dxf retained");
        assert_eq!(facts.sheet_views.len(), 1, "SheetViewState retained");
        assert_eq!(facts.data_validations.len(), 1, "DataValidations retained");
    }

    // ---- W062 R6.6: the W011 five-step round-trip contract (PIVOT B) ----------

    /// A `SheetViewState` Tier-B fact on `Sheet1` (upstream id 1): a non-trivial,
    /// sheet-scoped Tier-B event whose verbatim replay in step 3 is a real
    /// assertion (event-level equality against this exact value), not a tautology.
    /// Distinctive fields (zoom 85, grid lines off) make an accidental match
    /// vanishingly unlikely.
    fn w011_sheet_view_fact() -> SheetViewState {
        SheetViewState {
            sheet_id: 1,
            workbook_view_id: Some(0),
            view: Some("normal".to_string()),
            show_grid_lines: Some(false),
            show_row_col_headers: Some(true),
            right_to_left: None,
            tab_selected: Some(true),
            zoom_scale: Some(85),
            top_left_cell: None,
            pane: None,
            selections: Vec::new(),
            raw_attrs: Vec::new(),
            raw_children: Vec::new(),
        }
    }

    /// The W011 source stream: the Automatic-mode prelude, one sheet-scoped Tier-B
    /// fact (a `SheetViewState`), then the two-cell body (`A1 = 7`, `B1 = =A1*3`
    /// cached 21). The `SheetViewState` sits inside the sheet scope, before the
    /// cell chunk — the validator-legal position for a sheet-scoped fact.
    fn w011_round_trip_source_stream() -> Vec<DocumentEvent> {
        let mut stream = formula_prelude();
        stream.push(DocumentEvent::SheetBegin(SheetRef {
            sheet_id: 1,
            name: "Sheet1".to_string(),
        }));
        stream.push(DocumentEvent::SheetViewState(w011_sheet_view_fact()));
        stream.push(DocumentEvent::CellChunk(CellChunk {
            row_band: 0,
            cells: vec![
                (addr(1, 1), CellPayload::Number(7.0)),
                (
                    addr(1, 2),
                    CellPayload::Formula {
                        region: None,
                        text: Some("A1*3".to_string()),
                        cached: Some(Box::new(CellPayload::Number(21.0))),
                    },
                ),
            ],
        }));
        stream.push(DocumentEvent::SheetEnd { sheet_id: 1 });
        stream
    }

    /// The pull-out helper the round-trip test uses to read the sole
    /// `WholeModelProjection` event stream out of a `WorkbookModelOutput`.
    fn whole_model_events(output: &oxdoc_model::WorkbookModelOutput) -> Vec<DocumentEvent> {
        assert_eq!(output.entries.len(), 1, "one whole-model projection entry");
        match &output.entries[0] {
            oxdoc_model::WorkbookModelOutputEntry::WholeModelProjection { events } => {
                events.clone()
            }
            other => panic!("expected WholeModelProjection, got {other:?}"),
        }
    }

    /// **THE PIVOT-B ACCEPTANCE (W062 R6.6): the W011 five-step round-trip
    /// contract as one decisive test.** Closing this un-pauses DnaTreeCalc W011
    /// with zero hand-keyed constants. Each step's decisive assertion below:
    ///
    /// 1. Load `A1=7`, `B1==A1*3` (cached 21) via `load_workbook_model`; the
    ///    Automatic open-recalc replaces the FileCached (7,21) with engine (7,21).
    /// 2. `enter_grid_cell(A1,"10")` → literal → revision advances → auto-recalc →
    ///    B1 publishes 30.
    /// 3. `project_workbook_model_output` → A1 is `Number(10.0)`, B1 is
    ///    `Formula{ text: Some("A1*3") preserved, cached: Some(Number(30.0))
    ///    refreshed-from-publication }`, and the source's `SheetViewState` Tier-B
    ///    event replays VERBATIM (event-level equality).
    /// 4. `workbook_authored_delta(since = load revision)` reports EXACTLY ONE
    ///    cell-input edit (A1 literal); B1 appears NOWHERE.
    /// 5. Reload the projected stream into a FRESH context via `load_workbook_model`
    ///    (event-stream reload, NOT snapshot import): authored views equal,
    ///    published values equal after recalc — the full circle.
    #[test]
    fn w011_five_step_round_trip_contract() {
        // ===== STEP 1: load the two-cell workbook via load_workbook_model. =====
        let mut context = OxCalcDocumentContext::default();
        let source_stream = w011_round_trip_source_stream();
        let (workspace_id, report) = load_workbook_model(
            &mut context,
            OxCalcWorkbookCreate::new("workbook:w011-round-trip"),
            &source_stream,
        )
        .unwrap();
        assert_eq!(report.cells, 1, "one literal (A1)");
        assert_eq!(
            report.formulas_bound, 1,
            "B1 bound strict-excel through the single mint"
        );
        assert_eq!(
            report.recalc_path,
            LoadRecalcPath::Automatic,
            "the Automatic-mode load takes the open-recalc path",
        );
        // The load revision is the delta basis for step 4 — captured BEFORE the edit.
        let load_revision = context
            .workspace_view(&workspace_id)
            .unwrap()
            .workspace_revision_id;
        // Post-load: B1 is the engine's own 21 (Calculated), the FileCached (7,21)
        // replaced by the open-recalc — agreeing per the differential (7,21==7,21).
        let (b1_loaded, b1_prov_loaded) = published_value(&context, &workspace_id, 1, 2).unwrap();
        assert_eq!(
            b1_loaded,
            CalcValue::number(21.0),
            "B1 == A1*3 == 21 at load"
        );
        assert!(
            matches!(b1_prov_loaded, PublishedValueProvenance::Calculated { .. }),
            "the open-recalc made B1 engine-Calculated, not FileCached",
        );

        // ===== STEP 2: enter A1 = "10" → literal → recalc → B1 publishes 30. =====
        let node = context.sheets(&workspace_id).unwrap()[0].node_id;
        let a1 = ingested_address(&context, &workspace_id, 1, 1);
        let outcome = context
            .enter_grid_cell(&workspace_id, node, &a1, "10")
            .unwrap()
            .unwrap();
        assert!(
            matches!(outcome, GridCellEntryOutcome::Literal { .. }),
            "'10' takes the literal branch (C9), not a formula",
        );
        // The edit advanced the workspace revision (A1's authored input changed).
        let post_edit_revision = context
            .workspace_view(&workspace_id)
            .unwrap()
            .workspace_revision_id;
        assert_ne!(
            post_edit_revision, load_revision,
            "the literal edit minted a new workspace revision",
        );
        // Under Automatic mode the edit auto-recalcs: B1 = 10*3 = 30, Calculated.
        let (b1_after_edit, b1_prov_after_edit) =
            published_value(&context, &workspace_id, 1, 2).unwrap();
        assert_eq!(
            b1_after_edit,
            CalcValue::number(30.0),
            "B1 == A1*3 == 30 after A1:=10"
        );
        assert!(
            matches!(
                b1_prov_after_edit,
                PublishedValueProvenance::Calculated { .. }
            ),
            "B1's 30 is a fresh engine value",
        );

        // ===== STEP 3: project → the event stream, with the shapes and the =====
        //               Tier-B verbatim replay the contract fixes.
        let output = context
            .project_workbook_model_output(&workspace_id)
            .unwrap();
        let events = whole_model_events(&output);

        // -- 3a: A1 is Number(10.0). Find the single CellChunk and read its cells. --
        let chunk = events
            .iter()
            .find_map(|event| match event {
                DocumentEvent::CellChunk(chunk) => Some(chunk),
                _ => None,
            })
            .expect("the projection emits one CellChunk for the sheet");
        let a1_payload = chunk
            .cells
            .iter()
            .find(|(addr, _)| *addr == super::PackedCellAddr::from_one_based(1, 1).unwrap())
            .map(|(_, payload)| payload)
            .expect("A1 present in the projected chunk");
        assert_eq!(
            *a1_payload,
            CellPayload::Number(10.0),
            "A1 projects as the edited literal Number(10.0)",
        );

        // -- 3b: B1 is Formula{ text: Some("A1*3") preserved, cached: Number(30.0) --
        //        refreshed from PUBLICATION (not the stored 21). --
        let b1_payload = chunk
            .cells
            .iter()
            .find(|(addr, _)| *addr == super::PackedCellAddr::from_one_based(1, 2).unwrap())
            .map(|(_, payload)| payload)
            .expect("B1 present in the projected chunk");
        assert_eq!(
            *b1_payload,
            CellPayload::Formula {
                region: None,
                text: Some("A1*3".to_string()),
                cached: Some(Box::new(CellPayload::Number(30.0))),
            },
            "B1 projects as Formula with PRESERVED authored text and a cache \
             REFRESHED from publication (30, not the stored 21)",
        );

        // -- 3c: the source's SheetViewState Tier-B event replays VERBATIM. --
        //        Event-level equality against the exact source value — the
        //        no-silent-loss / verbatim-replay assertion the contract demands.
        let source_view = DocumentEvent::SheetViewState(w011_sheet_view_fact());
        let projected_view_count = events.iter().filter(|event| **event == source_view).count();
        assert_eq!(
            projected_view_count, 1,
            "the SheetViewState Tier-B fact replays exactly once, byte-for-byte \
             equal to the source event (verbatim, not re-synthesized)",
        );
        // And it sits inside the sheet scope, before the cell chunk (validator-legal
        // position) — locate SheetBegin < SheetViewState < CellChunk < SheetEnd.
        let idx_begin = events
            .iter()
            .position(|e| matches!(e, DocumentEvent::SheetBegin(_)))
            .unwrap();
        let idx_view = events.iter().position(|e| *e == source_view).unwrap();
        let idx_chunk = events
            .iter()
            .position(|e| matches!(e, DocumentEvent::CellChunk(_)))
            .unwrap();
        let idx_end = events
            .iter()
            .position(|e| matches!(e, DocumentEvent::SheetEnd { .. }))
            .unwrap();
        assert!(
            idx_begin < idx_view && idx_view < idx_chunk && idx_chunk < idx_end,
            "the Tier-B view replays in its sheet scope, before the cell band",
        );

        // The projected stream is itself validator-clean (a real, loadable stream).
        oxdoc_model::validate_event_stream(&events)
            .expect("the projected stream passes the oxdoc-model stream validator");

        // ===== STEP 4: authored delta since the LOAD revision = exactly one A1. =====
        let delta = context
            .workbook_authored_delta(&workspace_id, &load_revision)
            .unwrap();
        assert_eq!(
            delta.cells.len(),
            1,
            "exactly ONE cell-input edit since load (A1), got {:?}",
            delta.cells,
        );
        let edit = &delta.cells[0];
        assert_eq!(edit.address.row, 1, "the one edit is at row 1");
        assert_eq!(edit.address.col, 1, "the one edit is at column 1 (A1)");
        // It is a literal CHANGE (A1 was literal 7, now literal 10) — from grid-input
        // snapshot diffs only, never a derived value.
        match &edit.change {
            crate::authored_delta::CellInputChange::Changed { old, new } => {
                assert_eq!(
                    *old,
                    GridInputCell::Literal(CalcValue::number(7.0)),
                    "A1 was the literal 7 at load",
                );
                assert_eq!(
                    *new,
                    GridInputCell::Literal(CalcValue::number(10.0)),
                    "A1 is now the literal 10",
                );
            }
            other => panic!("A1 edit should be a literal Change, got {other:?}"),
        }
        // B1 appears NOWHERE in the delta: its authored truth (=A1*3) never changed,
        // even though its PUBLISHED value moved 21 → 30. The delta is authored-only.
        assert!(
            delta.cells.iter().all(|cell| cell.address.col != 2),
            "B1 (col 2) must NOT appear in the delta — its authored truth is unchanged",
        );
        assert!(delta.sheet_collections.is_empty(), "no collection edits");
        assert!(delta.sheets.is_empty(), "no sheet lifecycle edits");
        assert!(delta.settings.is_empty(), "no settings edits");

        // ===== STEP 5: reload the PROJECTED stream into a FRESH context. =====
        //               Event-stream reload (NOT snapshot import) — re-ingests
        //               Tier B from the events, so the facts-store snapshot
        //               boundary (R6.4 carry-forward) does not bite.
        let mut fresh_context = OxCalcDocumentContext::default();
        let (fresh_workspace, fresh_report) = load_workbook_model(
            &mut fresh_context,
            OxCalcWorkbookCreate::new("workbook:w011-reloaded"),
            &events,
        )
        .unwrap();
        assert_eq!(fresh_report.cells, 1, "reload: one literal (A1)");
        assert_eq!(fresh_report.formulas_bound, 1, "reload: B1 re-bound");

        // 5a: authored views equal — A1 is the literal 10, B1's source text is =A1*3.
        let fresh_node = fresh_context.sheets(&fresh_workspace).unwrap()[0].node_id;
        let fresh_a1_readout = fresh_context
            .grid_authored_view(&fresh_workspace, fresh_node, None)
            .unwrap()
            .unwrap()
            .into_iter()
            .find(|cell| cell.address.row == 1 && cell.address.col == 1)
            .unwrap();
        assert_eq!(
            fresh_a1_readout.literal,
            Some(CalcValue::number(10.0)),
            "reloaded A1's authored literal is 10 (the projected value round-tripped)",
        );
        assert_eq!(
            authored_source_text(&fresh_context, &fresh_workspace, 1, 2).as_deref(),
            Some("=A1*3"),
            "reloaded B1's authored formula text is =A1*3 (leading = restored on ingest)",
        );

        // 5b: published values equal after recalc — B1 = 30 by the engine on reload.
        fresh_context
            .recalculate_workbook(&fresh_workspace)
            .unwrap();
        let (fresh_b1, fresh_b1_prov) =
            published_value(&fresh_context, &fresh_workspace, 1, 2).unwrap();
        assert_eq!(
            fresh_b1,
            CalcValue::number(30.0),
            "reloaded B1 recomputes to 30 — the full circle closes",
        );
        assert!(
            matches!(fresh_b1_prov, PublishedValueProvenance::Calculated { .. }),
            "reloaded-and-recalculated B1 is engine-Calculated",
        );

        // 5c: the source SheetViewState survived the whole round trip — the fresh
        // context re-ingested it verbatim into its Tier-B store.
        let fresh_facts = fresh_context
            .ingested_document_facts(&fresh_workspace)
            .unwrap();
        assert_eq!(
            fresh_facts.sheet_views,
            vec![w011_sheet_view_fact()],
            "the SheetViewState round-tripped verbatim into the reloaded store",
        );
    }

    /// The projection's scalar-payload mapping and write-time shared-string dedup
    /// (D4 §7a / §11), which the all-numeric W011 fixture does not exercise: a
    /// number, a bool, an error, and two literal-text cells that share ONE string.
    /// Proves (a) each `CalcValue` core projects to the right `CellPayload`, and
    /// (b) the string table is de-duplicated at write — two equal texts collapse to
    /// one `SharedText` index and one `StringTable` entry (§11 dedup-at-write).
    #[test]
    fn projection_maps_literal_kinds_and_dedups_shared_strings() {
        let mut context = OxCalcDocumentContext::default();
        let mut stream = formula_prelude();
        stream.push(DocumentEvent::SheetBegin(SheetRef {
            sheet_id: 1,
            name: "Sheet1".to_string(),
        }));
        stream.push(DocumentEvent::CellChunk(CellChunk {
            row_band: 0,
            cells: vec![
                (addr(1, 1), CellPayload::Number(3.5)),
                (addr(1, 2), CellPayload::Bool(true)),
                (addr(1, 3), CellPayload::Error(0x07)), // #DIV/0!
                (addr(1, 4), CellPayload::InlineText("dup".to_string())),
                (addr(1, 5), CellPayload::InlineText("dup".to_string())),
            ],
        }));
        stream.push(DocumentEvent::SheetEnd { sheet_id: 1 });
        let (workspace_id, _report) = load_workbook_model(
            &mut context,
            OxCalcWorkbookCreate::new("workbook:projection-kinds"),
            &stream,
        )
        .unwrap();

        let output = context
            .project_workbook_model_output(&workspace_id)
            .unwrap();
        let events = whole_model_events(&output);

        // The write-time StringTable carries exactly ONE entry ("dup") despite two
        // text cells — dedup at write (§11).
        let string_table = events
            .iter()
            .find_map(|event| match event {
                DocumentEvent::StringTable(entries) => Some(entries.clone()),
                _ => None,
            })
            .expect("a StringTable prelude event");
        assert_eq!(
            string_table,
            vec![SharedStringEntry {
                text: "dup".to_string()
            }],
            "the two equal text literals collapse to ONE shared-string entry",
        );

        let chunk = events
            .iter()
            .find_map(|event| match event {
                DocumentEvent::CellChunk(chunk) => Some(chunk),
                _ => None,
            })
            .expect("a CellChunk");
        let payload_at = |row: u32, col: u32| -> CellPayload {
            chunk
                .cells
                .iter()
                .find(|(a, _)| *a == PackedCellAddr::from_one_based(row, col).unwrap())
                .map(|(_, p)| p.clone())
                .unwrap()
        };
        assert_eq!(
            payload_at(1, 1),
            CellPayload::Number(3.5),
            "number → Number"
        );
        assert_eq!(payload_at(1, 2), CellPayload::Bool(true), "bool → Bool");
        assert_eq!(
            payload_at(1, 3),
            CellPayload::Error(0x07),
            "the #DIV/0! error round-trips to its classic BIFF byte 0x07",
        );
        // Both text cells point at shared index 0 (the one deduped entry).
        assert_eq!(
            payload_at(1, 4),
            CellPayload::SharedText(0),
            "text → SharedText(0)"
        );
        assert_eq!(
            payload_at(1, 5),
            CellPayload::SharedText(0),
            "the duplicate text reuses the SAME shared index (dedup at write)",
        );

        // The whole projected stream validates.
        oxdoc_model::validate_event_stream(&events).expect("projected kinds stream validates");
    }

    /// W062 R6.66 (calc-5kqg.66): the save-side name renderer and the load-side
    /// rect parser are symmetric even for a sheet name containing an apostrophe
    /// (Excel doubles it: `O'Brien` → `'O''Brien'`). Without the `''`-aware close
    /// scan in `split_sheet_qualifier`, the rendered text re-parses as a bare
    /// (non-rect) reference and a STATIC name is silently reclassified dynamic.
    #[test]
    fn apostrophe_sheet_name_static_name_renders_and_reparses_as_rect() {
        // Multi-cell rect.
        let rendered = render_absolute_name_formula_text("O'Brien", 1, 1, 2, 3);
        assert_eq!(rendered, "'O''Brien'!$A$1:$C$2");
        let parsed = parse_rect_denoting_reference(&rendered)
            .expect("re-parses as a STATIC rect, not dynamic");
        assert_eq!(
            parsed.sheet, "O'Brien",
            "the sheet name un-doubles the '' escape"
        );
        assert_eq!(
            (
                parsed.top_row,
                parsed.left_col,
                parsed.bottom_row,
                parsed.right_col
            ),
            (1, 1, 2, 3),
        );

        // Single cell.
        let one = render_absolute_name_formula_text("A'B", 5, 4, 5, 4);
        assert_eq!(one, "'A''B'!$D$5");
        let parsed_one =
            parse_rect_denoting_reference(&one).expect("single-cell apostrophe name re-parses");
        assert_eq!(parsed_one.sheet, "A'B");
        assert_eq!((parsed_one.top_row, parsed_one.left_col), (5, 4));

        // A bare identifier is NOT quoted (unchanged behavior).
        assert_eq!(
            render_absolute_name_formula_text("Sheet1", 1, 1, 1, 1),
            "Sheet1!$A$1"
        );
        // A name with a space is quoted (no apostrophe to double).
        assert_eq!(
            render_absolute_name_formula_text("My Sheet", 1, 1, 1, 1),
            "'My Sheet'!$A$1",
        );
        assert_eq!(
            parse_rect_denoting_reference("'My Sheet'!$A$1")
                .unwrap()
                .sheet,
            "My Sheet",
        );
    }

    /// W062 R6.66 (calc-5kqg.66): a workbook carrying an authored merged region
    /// projects it back as a `MergedCellRegions` event that RELOADS intact (the
    /// projection is now lossless for Tier-A collections, not a typed refusal).
    #[test]
    fn projection_round_trips_a_merged_region() {
        let mut context = OxCalcDocumentContext::default();
        let mut stream = formula_prelude();
        stream.push(DocumentEvent::SheetBegin(SheetRef {
            sheet_id: 1,
            name: "Sheet1".to_string(),
        }));
        stream.push(DocumentEvent::MergedCellRegions(MergedCellRegions {
            sheet_id: 1,
            ranges: vec![CellRangeSpec {
                text: "A1:B2".to_string(),
                start: addr(1, 1),
                end: addr(2, 2),
            }],
            raw_refs: Vec::new(),
        }));
        stream.push(DocumentEvent::CellChunk(CellChunk {
            row_band: 0,
            cells: vec![(addr(3, 1), CellPayload::Number(1.0))],
        }));
        stream.push(DocumentEvent::SheetEnd { sheet_id: 1 });
        let (workspace_id, _report) = load_workbook_model(
            &mut context,
            OxCalcWorkbookCreate::new("workbook:merge-round-trip"),
            &stream,
        )
        .unwrap();

        // The projection emits the merged region (no longer a typed refusal).
        let events = whole_model_events(
            &context
                .project_workbook_model_output(&workspace_id)
                .unwrap(),
        );
        oxdoc_model::validate_event_stream(&events).expect("projected stream validates");
        let merged: Vec<&MergedCellRegions> = events
            .iter()
            .filter_map(|event| match event {
                DocumentEvent::MergedCellRegions(regions) => Some(regions),
                _ => None,
            })
            .collect();
        assert_eq!(merged.len(), 1, "one MergedCellRegions event projected");
        assert_eq!(merged[0].sheet_id, 1, "keyed to the upstream sheet id");
        assert_eq!(merged[0].ranges.len(), 1);
        assert_eq!(merged[0].ranges[0].start, addr(1, 1), "A1 start");
        assert_eq!(merged[0].ranges[0].end, addr(2, 2), "B2 end");
        assert_eq!(merged[0].ranges[0].text, "A1:B2", "rendered A1 range");

        // Reload into a FRESH context and re-project: the merged region survives the
        // full circle (idempotent — the reloaded workbook re-emits the same event).
        let mut fresh = OxCalcDocumentContext::default();
        let (fresh_ws, _) = load_workbook_model(
            &mut fresh,
            OxCalcWorkbookCreate::new("workbook:merge-reloaded"),
            &events,
        )
        .unwrap();
        let reprojected =
            whole_model_events(&fresh.project_workbook_model_output(&fresh_ws).unwrap());
        let reprojected_merged: Vec<&MergedCellRegions> = reprojected
            .iter()
            .filter_map(|event| match event {
                DocumentEvent::MergedCellRegions(regions) => Some(regions),
                _ => None,
            })
            .collect();
        assert_eq!(
            reprojected_merged, merged,
            "the merged region round-trips intact through load → project → reload → project",
        );
    }

    /// Collect the four Tier-A collection event kinds from a projected stream, in
    /// stream order, for round-trip comparison.
    fn collection_events(events: &[DocumentEvent]) -> Vec<DocumentEvent> {
        events
            .iter()
            .filter(|event| {
                matches!(
                    event,
                    DocumentEvent::MergedCellRegions(_)
                        | DocumentEvent::TableOverlay(_)
                        | DocumentEvent::DefinedName(_)
                        | DocumentEvent::SharedFormulaRegion(_)
                )
            })
            .cloned()
            .collect()
    }

    /// **THE calc-5kqg.66 ACCEPTANCE (W062 R6.66): all four authored Tier-A
    /// collections round-trip.** A workbook carrying a merged region, a table, a
    /// workbook-scoped + a sheet-scoped defined name (the latter with Tier-B
    /// metadata), and a repeated (shared-formula) region projects to a
    /// validator-clean stream whose collection events reload into a FRESH context
    /// and re-project IDENTICALLY — the collections survive the full circle.
    #[test]
    fn projection_round_trips_all_tier_a_collections() {
        let mut context = OxCalcDocumentContext::default();
        let stream = vec![
            DocumentEvent::WorkbookHeader(WorkbookHeader::new(
                DocDateSystem::Date1900,
                DocCalcMode::Automatic,
            )),
            DocumentEvent::StringTable(Vec::new()),
            DocumentEvent::StyleTable(StyleTableSpec::minimal()),
            // Workbook-scoped events (prelude): a workbook name (with Tier-B
            // metadata) and a sheet-scoped name targeting Sheet2.
            DocumentEvent::DefinedName(DefinedNameSpec {
                name: "WBN".to_string(),
                formula_text: "Sheet1!$A$1".to_string(),
                scope_sheet_id: None,
                metadata: DefinedNameMetadataSpec {
                    comment: Some("the workbook name".to_string()),
                    ..DefinedNameMetadataSpec::default()
                },
            }),
            DocumentEvent::DefinedName(DefinedNameSpec {
                name: "SN".to_string(),
                formula_text: "Sheet2!$A$1".to_string(),
                scope_sheet_id: Some(2),
                metadata: DefinedNameMetadataSpec::default(),
            }),
            // Sheet1: a merged region, a shared-formula region, a table.
            DocumentEvent::SheetBegin(SheetRef {
                sheet_id: 1,
                name: "Sheet1".to_string(),
            }),
            DocumentEvent::MergedCellRegions(MergedCellRegions {
                sheet_id: 1,
                ranges: vec![CellRangeSpec {
                    text: "D1:E2".to_string(),
                    start: PackedCellAddr::from_one_based(1, 4).unwrap(),
                    end: PackedCellAddr::from_one_based(2, 5).unwrap(),
                }],
                raw_refs: Vec::new(),
            }),
            DocumentEvent::SharedFormulaRegion(SharedFormulaRegion {
                region_id: 0,
                anchor: PackedCellAddr::from_one_based(1, 8).unwrap(),
                extent: Extent { rows: 2, cols: 1 },
                r1c1_text: "A1+1".to_string(),
            }),
            DocumentEvent::TableOverlay(TableSpec {
                name: "T".to_string(),
                sheet_id: 1,
                range: "A5:B6".to_string(),
            }),
            DocumentEvent::CellChunk(CellChunk {
                row_band: 0,
                cells: vec![(addr(1, 1), CellPayload::Number(1.0))],
            }),
            DocumentEvent::SheetEnd { sheet_id: 1 },
            // Sheet2: the sheet-scoped name's target cell.
            DocumentEvent::SheetBegin(SheetRef {
                sheet_id: 2,
                name: "Sheet2".to_string(),
            }),
            DocumentEvent::CellChunk(CellChunk {
                row_band: 0,
                cells: vec![(addr(1, 1), CellPayload::Number(2.0))],
            }),
            DocumentEvent::SheetEnd { sheet_id: 2 },
        ];

        let (workspace_id, _report) = load_workbook_model(
            &mut context,
            OxCalcWorkbookCreate::new("workbook:collections"),
            &stream,
        )
        .unwrap();

        // -- Project: a validator-clean stream carrying all four collection kinds. --
        let events = whole_model_events(
            &context
                .project_workbook_model_output(&workspace_id)
                .unwrap(),
        );
        oxdoc_model::validate_event_stream(&events)
            .expect("the projected collection-bearing stream validates");

        // Merged region → its upstream sheet id + rendered A1 range.
        let merged: Vec<_> = events
            .iter()
            .filter_map(|e| match e {
                DocumentEvent::MergedCellRegions(r) => Some(r),
                _ => None,
            })
            .collect();
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].sheet_id, 1);
        assert_eq!(merged[0].ranges[0].text, "D1:E2");

        // Table → name + upstream sheet id + rendered range.
        let tables: Vec<_> = events
            .iter()
            .filter_map(|e| match e {
                DocumentEvent::TableOverlay(t) => Some(t),
                _ => None,
            })
            .collect();
        assert_eq!(tables.len(), 1);
        assert_eq!(tables[0].name, "T");
        assert_eq!(tables[0].sheet_id, 1);
        assert_eq!(tables[0].range, "A5:B6");

        // Shared-formula region → anchor + extent + r1c1 text.
        let shared: Vec<_> = events
            .iter()
            .filter_map(|e| match e {
                DocumentEvent::SharedFormulaRegion(s) => Some(s),
                _ => None,
            })
            .collect();
        assert_eq!(shared.len(), 1);
        assert_eq!(
            shared[0].anchor,
            PackedCellAddr::from_one_based(1, 8).unwrap()
        );
        assert_eq!(shared[0].extent, Extent { rows: 2, cols: 1 });
        assert_eq!(shared[0].r1c1_text, "A1+1");

        // Defined names → both, in the prelude, with faithful scope + formula_text +
        // the Tier-B metadata half re-attached.
        let names: Vec<_> = events
            .iter()
            .filter_map(|e| match e {
                DocumentEvent::DefinedName(n) => Some(n),
                _ => None,
            })
            .collect();
        assert_eq!(names.len(), 2, "both defined names projected");
        let wbn = names.iter().find(|n| n.name == "WBN").expect("WBN present");
        assert_eq!(wbn.scope_sheet_id, None, "WBN is workbook-scoped");
        assert_eq!(
            wbn.formula_text, "Sheet1!$A$1",
            "WBN rect → absolute A1 ref"
        );
        assert_eq!(
            wbn.metadata.comment.as_deref(),
            Some("the workbook name"),
            "WBN's Tier-B metadata half re-attached",
        );
        let sn = names.iter().find(|n| n.name == "SN").expect("SN present");
        assert_eq!(
            sn.scope_sheet_id,
            Some(2),
            "SN is scoped to Sheet2 (upstream id 2)"
        );
        assert_eq!(sn.formula_text, "Sheet2!$A$1", "SN rect → absolute A1 ref");

        // Every DefinedName sits in the prelude, before the first SheetBegin.
        let first_sheet = events
            .iter()
            .position(|e| matches!(e, DocumentEvent::SheetBegin(_)))
            .unwrap();
        for (index, event) in events.iter().enumerate() {
            if matches!(event, DocumentEvent::DefinedName(_)) {
                assert!(
                    index < first_sheet,
                    "DefinedName is a prelude (workbook-scoped) event"
                );
            }
        }

        // -- Reload into a FRESH context + re-project: collections survive intact. --
        let mut fresh = OxCalcDocumentContext::default();
        let (fresh_ws, _) = load_workbook_model(
            &mut fresh,
            OxCalcWorkbookCreate::new("workbook:collections-reloaded"),
            &events,
        )
        .unwrap();
        let reprojected =
            whole_model_events(&fresh.project_workbook_model_output(&fresh_ws).unwrap());
        assert_eq!(
            collection_events(&reprojected),
            collection_events(&events),
            "all four Tier-A collections round-trip intact (load → project → reload → project)",
        );
    }

    /// A DYNAMIC defined name (a non-rect-denoting formula, installed as a dynamic
    /// name) round-trips: its authored source text re-emits as the projected
    /// `formula_text` and reloads to the same dynamic name. Guards the Err removal
    /// for the dynamic branch, which the static-name acceptance does not exercise.
    #[test]
    fn projection_round_trips_a_dynamic_defined_name() {
        let mut context = OxCalcDocumentContext::default();
        let stream = vec![
            DocumentEvent::WorkbookHeader(WorkbookHeader::new(
                DocDateSystem::Date1900,
                DocCalcMode::Automatic,
            )),
            DocumentEvent::StringTable(Vec::new()),
            DocumentEvent::StyleTable(StyleTableSpec::minimal()),
            // A non-rect formula → a dynamic name (an arithmetic expression).
            DocumentEvent::DefinedName(DefinedNameSpec {
                name: "DYN".to_string(),
                formula_text: "Sheet1!$A$1+1".to_string(),
                scope_sheet_id: None,
                metadata: DefinedNameMetadataSpec::default(),
            }),
            DocumentEvent::SheetBegin(SheetRef {
                sheet_id: 1,
                name: "Sheet1".to_string(),
            }),
            DocumentEvent::CellChunk(CellChunk {
                row_band: 0,
                cells: vec![(addr(1, 1), CellPayload::Number(5.0))],
            }),
            DocumentEvent::SheetEnd { sheet_id: 1 },
        ];
        let (workspace_id, _report) = load_workbook_model(
            &mut context,
            OxCalcWorkbookCreate::new("workbook:dynamic-name"),
            &stream,
        )
        .unwrap();

        let events = whole_model_events(
            &context
                .project_workbook_model_output(&workspace_id)
                .unwrap(),
        );
        oxdoc_model::validate_event_stream(&events).expect("projected stream validates");
        let dyn_name = events
            .iter()
            .find_map(|e| match e {
                DocumentEvent::DefinedName(n) if n.name == "DYN" => Some(n),
                _ => None,
            })
            .expect("the dynamic name projected");
        assert_eq!(
            dyn_name.formula_text, "Sheet1!$A$1+1",
            "the dynamic name re-emits its authored source text",
        );
        assert_eq!(dyn_name.scope_sheet_id, None);

        // Reloads to the same dynamic name (installed again, re-projects identically).
        let mut fresh = OxCalcDocumentContext::default();
        let (fresh_ws, _fresh_report) = load_workbook_model(
            &mut fresh,
            OxCalcWorkbookCreate::new("workbook:dynamic-name-reloaded"),
            &events,
        )
        .unwrap();
        let reprojected =
            whole_model_events(&fresh.project_workbook_model_output(&fresh_ws).unwrap());
        assert_eq!(
            collection_events(&reprojected),
            collection_events(&events),
            "the dynamic name round-trips intact",
        );
    }

    /// W062 R6.68 (calc-5kqg.68): a workbook-scoped and a sheet-scoped defined
    /// name of the SAME text (a legal Excel shadow pair), each carrying DISTINCT
    /// Tier-B metadata, both round-trip their OWN metadata half — the metadata
    /// store is keyed by (name, scope), not name text alone.
    #[test]
    fn projection_round_trips_shadow_pair_metadata_by_scope() {
        let mut context = OxCalcDocumentContext::default();
        let stream = vec![
            DocumentEvent::WorkbookHeader(WorkbookHeader::new(
                DocDateSystem::Date1900,
                DocCalcMode::Automatic,
            )),
            DocumentEvent::StringTable(Vec::new()),
            DocumentEvent::StyleTable(StyleTableSpec::minimal()),
            // Workbook-scoped `Foo` → Sheet1!$A$1, comment "workbook Foo".
            DocumentEvent::DefinedName(DefinedNameSpec {
                name: "Foo".to_string(),
                formula_text: "Sheet1!$A$1".to_string(),
                scope_sheet_id: None,
                metadata: DefinedNameMetadataSpec {
                    comment: Some("workbook Foo".to_string()),
                    ..DefinedNameMetadataSpec::default()
                },
            }),
            // Sheet-scoped `Foo` on Sheet1 → Sheet1!$B$1, comment "sheet Foo".
            DocumentEvent::DefinedName(DefinedNameSpec {
                name: "Foo".to_string(),
                formula_text: "Sheet1!$B$1".to_string(),
                scope_sheet_id: Some(1),
                metadata: DefinedNameMetadataSpec {
                    comment: Some("sheet Foo".to_string()),
                    ..DefinedNameMetadataSpec::default()
                },
            }),
            DocumentEvent::SheetBegin(SheetRef {
                sheet_id: 1,
                name: "Sheet1".to_string(),
            }),
            DocumentEvent::CellChunk(CellChunk {
                row_band: 0,
                cells: vec![
                    (addr(1, 1), CellPayload::Number(1.0)),
                    (addr(1, 2), CellPayload::Number(2.0)),
                ],
            }),
            DocumentEvent::SheetEnd { sheet_id: 1 },
        ];
        let (workspace_id, _report) = load_workbook_model(
            &mut context,
            OxCalcWorkbookCreate::new("book:shadow"),
            &stream,
        )
        .unwrap();

        let events = whole_model_events(
            &context
                .project_workbook_model_output(&workspace_id)
                .unwrap(),
        );
        let names: Vec<_> = events
            .iter()
            .filter_map(|e| match e {
                DocumentEvent::DefinedName(n) => Some(n),
                _ => None,
            })
            .collect();
        assert_eq!(names.len(), 2, "both shadow-pair names projected");

        // Each name carries its OWN metadata half, disambiguated by scope.
        let workbook_foo = names
            .iter()
            .find(|n| n.name == "Foo" && n.scope_sheet_id.is_none())
            .expect("workbook-scoped Foo present");
        assert_eq!(
            workbook_foo.metadata.comment.as_deref(),
            Some("workbook Foo"),
            "the workbook-scoped Foo keeps its own metadata",
        );
        let sheet_foo = names
            .iter()
            .find(|n| n.name == "Foo" && n.scope_sheet_id == Some(1))
            .expect("sheet-scoped Foo present");
        assert_eq!(
            sheet_foo.metadata.comment.as_deref(),
            Some("sheet Foo"),
            "the sheet-scoped Foo keeps ITS own metadata (not the workbook Foo's)",
        );
    }

    /// W062 R6.68 (calc-5kqg.68): the metadata re-attachment key holds when the
    /// scope sheet is NOT the first sheet and its upstream id is non-contiguous
    /// (Sheet2 with upstream id 5). The projection derives `scope_sheet_id` from
    /// `facts.sheet_stream_ids` (position → upstream id), which must equal the
    /// ingest-stored `spec.scope_sheet_id` (5), else the metadata would silently
    /// fail to re-attach. Also proves idempotence through a full reload.
    #[test]
    fn shadow_pair_metadata_reattaches_on_non_first_noncontiguous_scope_sheet() {
        let mut context = OxCalcDocumentContext::default();
        let stream = vec![
            DocumentEvent::WorkbookHeader(WorkbookHeader::new(
                DocDateSystem::Date1900,
                DocCalcMode::Automatic,
            )),
            DocumentEvent::StringTable(Vec::new()),
            DocumentEvent::StyleTable(StyleTableSpec::minimal()),
            // Workbook-scoped `Bar` → Sheet1!$A$1.
            DocumentEvent::DefinedName(DefinedNameSpec {
                name: "Bar".to_string(),
                formula_text: "Sheet1!$A$1".to_string(),
                scope_sheet_id: None,
                metadata: DefinedNameMetadataSpec {
                    comment: Some("workbook Bar".to_string()),
                    ..DefinedNameMetadataSpec::default()
                },
            }),
            // Sheet-scoped `Bar` on Sheet2 (upstream id 5) → Sheet2!$A$1.
            DocumentEvent::DefinedName(DefinedNameSpec {
                name: "Bar".to_string(),
                formula_text: "Sheet2!$A$1".to_string(),
                scope_sheet_id: Some(5),
                metadata: DefinedNameMetadataSpec {
                    comment: Some("sheet Bar".to_string()),
                    ..DefinedNameMetadataSpec::default()
                },
            }),
            DocumentEvent::SheetBegin(SheetRef {
                sheet_id: 1,
                name: "Sheet1".to_string(),
            }),
            DocumentEvent::CellChunk(CellChunk {
                row_band: 0,
                cells: vec![(addr(1, 1), CellPayload::Number(1.0))],
            }),
            DocumentEvent::SheetEnd { sheet_id: 1 },
            // Sheet2's upstream id is 5 (non-contiguous with its 0-based position 1).
            DocumentEvent::SheetBegin(SheetRef {
                sheet_id: 5,
                name: "Sheet2".to_string(),
            }),
            DocumentEvent::CellChunk(CellChunk {
                row_band: 0,
                cells: vec![(addr(1, 1), CellPayload::Number(2.0))],
            }),
            DocumentEvent::SheetEnd { sheet_id: 5 },
        ];
        let (workspace_id, _report) = load_workbook_model(
            &mut context,
            OxCalcWorkbookCreate::new("book:shadow2"),
            &stream,
        )
        .unwrap();

        let events = whole_model_events(
            &context
                .project_workbook_model_output(&workspace_id)
                .unwrap(),
        );
        let sheet_bar = events
            .iter()
            .find_map(|e| match e {
                DocumentEvent::DefinedName(n) if n.name == "Bar" && n.scope_sheet_id == Some(5) => {
                    Some(n)
                }
                _ => None,
            })
            .expect("sheet-scoped Bar projected with the non-contiguous upstream scope id 5");
        assert_eq!(
            sheet_bar.metadata.comment.as_deref(),
            Some("sheet Bar"),
            "the sheet-scoped Bar re-attaches its own metadata across the non-contiguous id",
        );

        // Full circle: reload the projected stream and re-project — both metadata
        // halves survive intact (idempotent).
        let mut fresh = OxCalcDocumentContext::default();
        let (fresh_ws, _) = load_workbook_model(
            &mut fresh,
            OxCalcWorkbookCreate::new("book:shadow2-reloaded"),
            &events,
        )
        .unwrap();
        let reprojected =
            whole_model_events(&fresh.project_workbook_model_output(&fresh_ws).unwrap());
        assert_eq!(
            collection_events(&reprojected),
            collection_events(&events),
            "the shadow-pair metadata round-trips intact through reload + re-project",
        );
    }
}

// ==== R6.5: load_workbook_model verb + load recalc policy + external pins =====
#[cfg(test)]
mod r6_5_tests {
    use super::{
        EXTERNAL_REFERENCE_NOT_LINKED, LoadRecalcPath, OxCalcWorkbookCreate, load_workbook_events,
        load_workbook_model, load_workbook_model_from_access,
    };
    use crate::consumer::{
        OxCalcDocumentContext, OxCalcDocumentError, OxCalcTreeWorkspaceCreate,
        OxCalcTreeWorkspaceId,
    };
    use crate::workbook_settings::PublishedValueProvenance;
    use oxdoc_model::{
        CalcMode as DocCalcMode, CellChunk, CellPayload, DateSystem as DocDateSystem,
        DocumentEvent, ExternalLinkSpec, LoadProfile, PackedCellAddr, SheetRef, SheetSummary,
        StyleTableSpec, SurfaceMaterialization, SurfaceRequest, WorkbookHeader,
        WorkbookModelAccess, WorkbookModelAccessError, WorkbookModelCapabilities,
        WorkbookModelContext, WorkbookSummary,
    };
    use oxfunc_core::value::{CalcValue, WorksheetErrorCode};

    fn addr(row: u32, col: u32) -> PackedCellAddr {
        PackedCellAddr::from_one_based(row, col).unwrap()
    }

    /// A two-sheet workbook: Sheet1 has `A1=7`, `B1==A1*3` (cached 21); Sheet2
    /// has `A1=10`, `B1==A1*2` (cached 20). Both formulas are SAME-SHEET: this
    /// fixture's acceptance is R6.5's one-revision load + per-sheet calc. The
    /// cross-sheet reference *evaluation* path (a `Sheet1!A1` from Sheet2) is
    /// wired and proven separately by the `r6_65_cross_sheet_load_tests` module
    /// (W062 R6.65 / calc-5kqg.65); keeping this fixture same-sheet isolates the
    /// per-sheet-calc counters from the cross-sheet closure.
    fn two_sheet_stream(calc_mode: DocCalcMode) -> Vec<DocumentEvent> {
        vec![
            DocumentEvent::WorkbookHeader(WorkbookHeader::new(DocDateSystem::Date1900, calc_mode)),
            DocumentEvent::StringTable(Vec::new()),
            DocumentEvent::StyleTable(StyleTableSpec::minimal()),
            DocumentEvent::SheetBegin(SheetRef {
                sheet_id: 1,
                name: "Sheet1".to_string(),
            }),
            DocumentEvent::CellChunk(CellChunk {
                row_band: 0,
                cells: vec![
                    (addr(1, 1), CellPayload::Number(7.0)),
                    (
                        addr(1, 2),
                        CellPayload::Formula {
                            region: None,
                            text: Some("A1*3".to_string()),
                            cached: Some(Box::new(CellPayload::Number(21.0))),
                        },
                    ),
                ],
            }),
            DocumentEvent::SheetEnd { sheet_id: 1 },
            DocumentEvent::SheetBegin(SheetRef {
                sheet_id: 2,
                name: "Sheet2".to_string(),
            }),
            DocumentEvent::CellChunk(CellChunk {
                row_band: 0,
                cells: vec![
                    (addr(1, 1), CellPayload::Number(10.0)),
                    // Same-sheet formula (cross-sheet reference *evaluation* is D2
                    // §4.1 / R4.x runtime routing, out of this bead's scope; the
                    // multi-sheet acceptance is one-revision load + per-sheet calc).
                    (
                        addr(1, 2),
                        CellPayload::Formula {
                            region: None,
                            text: Some("A1*2".to_string()),
                            cached: Some(Box::new(CellPayload::Number(20.0))),
                        },
                    ),
                ],
            }),
            DocumentEvent::SheetEnd { sheet_id: 2 },
        ]
    }

    /// The published cell value + provenance at `(row, col)` on sheet index `si`.
    fn published(
        context: &OxCalcDocumentContext,
        workspace_id: &OxCalcTreeWorkspaceId,
        si: usize,
        row: u32,
        col: u32,
    ) -> Option<(CalcValue, PublishedValueProvenance)> {
        let node = context.sheets(workspace_id).unwrap()[si].node_id;
        let view = context.grid_view(workspace_id, node).unwrap().unwrap();
        view.cells
            .iter()
            .find(|cell| cell.address.row == row && cell.address.col == col)
            .map(|cell| (cell.value.clone(), cell.provenance))
    }

    // ---- Acceptance: the public one-call verb, multi-sheet, one revision -----

    /// `load_workbook_model` (events path) creates the workbook workspace AND
    /// loads a MULTI-SHEET workbook in ONE revision, returning the report. Under
    /// Automatic the open-recalc ran, so both sheets' same-sheet formulas are
    /// engine `Calculated` (`Sheet1!B1 = A1*3 = 21`, `Sheet2!B1 = A1*2 = 20`).
    #[test]
    fn load_workbook_model_loads_multi_sheet_in_one_revision() {
        let mut context = OxCalcDocumentContext::default();
        let (workspace_id, report) = load_workbook_model(
            &mut context,
            OxCalcWorkbookCreate::new("book:multi"),
            &two_sheet_stream(DocCalcMode::Automatic),
        )
        .unwrap();

        assert_eq!(report.sheets, 2, "two sheets created in one load");
        assert_eq!(report.cells, 2, "two literals (A1 on each sheet)");
        assert_eq!(report.formulas_bound, 2, "both B1 formulas bound");
        assert_eq!(report.recalc_path, LoadRecalcPath::Automatic);

        // Exactly ONE load transaction over the fresh workspace: the current
        // revision's parent is the creation revision.
        let view = context.workspace_view(&workspace_id).unwrap();
        assert_eq!(
            view.workspace_revision_graph_entries.len(),
            2,
            "creation revision + one load transaction"
        );

        // The open-recalc computed both sheets, incl. the cross-sheet dependent.
        assert_eq!(
            published(&context, &workspace_id, 0, 1, 2).map(|(v, _)| v),
            Some(CalcValue::number(21.0)),
            "Sheet1!B1 = A1*3 = 21"
        );
        let (s2b1, s2b1_prov) = published(&context, &workspace_id, 1, 1, 2).unwrap();
        assert_eq!(s2b1, CalcValue::number(20.0), "Sheet2!B1 = A1*2 = 20");
        assert!(
            matches!(s2b1_prov, PublishedValueProvenance::Calculated { .. }),
            "the second sheet's formula is engine-Calculated by the open-recalc"
        );
    }

    /// The model-access variant loads IDENTICAL content to the events path. Both
    /// drive the same stream; the readouts (sheet count, cell values, provenance)
    /// are equal. This is the D4 §9 "model-access loads the same content" claim.
    #[test]
    fn load_workbook_model_from_access_loads_identical_content() {
        let stream = two_sheet_stream(DocCalcMode::Automatic);

        // Events path.
        let mut ctx_events = OxCalcDocumentContext::default();
        let (ws_events, report_events) = load_workbook_model(
            &mut ctx_events,
            OxCalcWorkbookCreate::new("book:events"),
            &stream,
        )
        .unwrap();

        // Model-access path (eager-event backed, mirroring oxdoc-d07.9's shape).
        struct EagerAccess {
            context: WorkbookModelContext,
            events: Vec<DocumentEvent>,
        }
        impl WorkbookModelAccess for EagerAccess {
            fn context(&self) -> &WorkbookModelContext {
                &self.context
            }
            fn eager_events(&self) -> Option<&[DocumentEvent]> {
                Some(&self.events)
            }
            fn materialize_surface(
                &self,
                request: SurfaceRequest,
            ) -> Result<SurfaceMaterialization, WorkbookModelAccessError> {
                Err(WorkbookModelAccessError::SurfaceUnavailable {
                    request,
                    reason: "eager-event backed test access".to_string(),
                })
            }
        }
        let access = EagerAccess {
            context: WorkbookModelContext {
                profile: LoadProfile::full(),
                summary: WorkbookSummary {
                    schema: "document-event.v1".to_string(),
                    date_system: Some(oxdoc_model::DateSystem::Date1900),
                    calc_mode: Some(oxdoc_model::CalcMode::Automatic),
                    sheet_count: 2,
                },
                sheets: vec![
                    SheetSummary {
                        sheet_id: 1,
                        name: "Sheet1".to_string(),
                        source_order: 0,
                        used_range: None,
                    },
                    SheetSummary {
                        sheet_id: 2,
                        name: "Sheet2".to_string(),
                        source_order: 1,
                        used_range: None,
                    },
                ],
                capabilities: WorkbookModelCapabilities {
                    eager_events_available: true,
                    deferred_materialization_available: false,
                    source_preservation_available: false,
                    macro_storage_available: false,
                },
                surfaces: Vec::new(),
            },
            events: stream.clone(),
        };
        let mut ctx_access = OxCalcDocumentContext::default();
        let (ws_access, report_access) = load_workbook_model_from_access(
            &mut ctx_access,
            OxCalcWorkbookCreate::new("book:access"),
            &access,
        )
        .unwrap();

        // Reports agree on the structural counts + recalc path.
        assert_eq!(report_events.sheets, report_access.sheets);
        assert_eq!(report_events.cells, report_access.cells);
        assert_eq!(report_events.formulas_bound, report_access.formulas_bound);
        assert_eq!(report_events.recalc_path, report_access.recalc_path);

        // The published readouts are identical cell-for-cell (values + provenance
        // class) across both paths.
        for (si, row, col) in [(0usize, 1u32, 1u32), (0, 1, 2), (1, 1, 1), (1, 1, 2)] {
            let a = published(&ctx_events, &ws_events, si, row, col);
            let b = published(&ctx_access, &ws_access, si, row, col);
            assert_eq!(
                a.as_ref().map(|(v, _)| v),
                b.as_ref().map(|(v, _)| v),
                "cell (s{si}, {row}, {col}) value differs between events and model-access loads"
            );
        }
    }

    // ---- Acceptance: Manual load renders FileCached with ZERO engine runs ----

    /// The Manual-zero-eval acceptance, PERF-COUNTER proven: a Manual-mode
    /// multi-sheet load runs ZERO engine recalc passes
    /// (`report.engine_recalcs_at_load == 0`), and both sheets render their
    /// FileCached caches (no engine value exists). The first explicit F9 then
    /// evaluates (counter > 0).
    #[test]
    fn manual_load_renders_filecached_with_zero_engine_runs() {
        let mut context = OxCalcDocumentContext::default();
        let (workspace_id, report) = load_workbook_model(
            &mut context,
            OxCalcWorkbookCreate::new("book:manual"),
            &two_sheet_stream(DocCalcMode::Manual),
        )
        .unwrap();

        assert_eq!(report.recalc_path, LoadRecalcPath::Manual);
        // THE PERF COUNTER: zero engine passes at load.
        assert_eq!(
            report.engine_recalcs_at_load, 0,
            "a Manual-mode load runs ZERO engine recalc passes (perf-counter proof)"
        );

        // Both formula cells render their FileCached caches (pre-engine).
        assert_eq!(
            published(&context, &workspace_id, 0, 1, 2),
            Some((
                CalcValue::number(21.0),
                PublishedValueProvenance::FileCached
            )),
            "Sheet1!B1 renders FileCached 21 (no engine pass)"
        );
        assert_eq!(
            published(&context, &workspace_id, 1, 1, 2),
            Some((
                CalcValue::number(20.0),
                PublishedValueProvenance::FileCached
            )),
            "Sheet2!B1 renders FileCached 20 (no engine pass)"
        );

        // The first F9 is the first real engine pass: it evaluates and both
        // formulas become engine-Calculated.
        let outcome = context.recalculate_workbook(&workspace_id).unwrap();
        assert!(
            outcome.drained_any(),
            "F9 drains the Manual-seeded formulas"
        );
        assert!(
            outcome.total_cells_evaluated() > 0,
            "the F9 evaluated cells (counter evidence a real recalc ran)"
        );
        let (s1b1, s1b1_prov) = published(&context, &workspace_id, 0, 1, 2).unwrap();
        assert_eq!(s1b1, CalcValue::number(21.0));
        assert!(matches!(
            s1b1_prov,
            PublishedValueProvenance::Calculated { .. }
        ));
    }

    // ---- Acceptance: Automatic load runs exactly one recalc, differential clean

    /// An Automatic-mode multi-sheet load runs the open-recalc: two sheets, two
    /// engine passes (one per sheet — the `recalculate_workbook`-shape drain),
    /// both differential-clean, published values `Calculated`. A subsequent F9 is
    /// a no-op (fully recalculated).
    #[test]
    fn automatic_load_open_recalcs_differential_clean_calculated() {
        let mut context = OxCalcDocumentContext::default();
        let (workspace_id, report) = load_workbook_model(
            &mut context,
            OxCalcWorkbookCreate::new("book:auto"),
            &two_sheet_stream(DocCalcMode::Automatic),
        )
        .unwrap();

        assert_eq!(report.recalc_path, LoadRecalcPath::Automatic);
        // One engine pass per calc-bearing sheet (the open-recalc).
        assert_eq!(
            report.engine_recalcs_at_load, 2,
            "an Automatic two-sheet load runs one open-recalc pass per sheet"
        );

        // Both formulas are engine-Calculated, and both sheets are differential-clean.
        for si in [0usize, 1] {
            let node = context.sheets(&workspace_id).unwrap()[si].node_id;
            let view = context.grid_view(&workspace_id, node).unwrap().unwrap();
            assert!(
                view.differential_mismatches.is_empty(),
                "sheet {si} is differential-clean after the open-recalc, got {:?}",
                view.differential_mismatches
            );
            let (_, prov) = published(&context, &workspace_id, si, 1, 2).unwrap();
            assert!(
                matches!(prov, PublishedValueProvenance::Calculated { .. }),
                "sheet {si} B1 is engine-Calculated after the open-recalc"
            );
        }

        // A subsequent F9 is a no-op (the workbook was already fully recalculated).
        let outcome = context.recalculate_workbook(&workspace_id).unwrap();
        assert!(!outcome.drained_any(), "post-open-recalc F9 is a no-op");
        assert_eq!(outcome.total_cells_evaluated(), 0);
    }

    // ---- Acceptance: external-reference pin survives a recalc ------------------

    /// The canonical external-reference fixture (D4 §14): `A1 = 7`, and
    /// `B1 = =SUM([Book2]Sheet1!A:A)` — a formula whose bound refs include an
    /// external-workbook token — with a FileCached cache of 99, plus an
    /// `ExternalLinkSpec` for `[Book2]`. The external cell:
    ///   - BINDS (authored text retained, counted in `formulas_bound`),
    ///   - is PINNED FileCached (its cache renders), NOT engine-evaluated,
    ///   - carries an `ExternalReferenceNotLinked` ledger row,
    ///   - keeps its FileCached value across a `recalculate_workbook` (the pin is
    ///     never clobbered — the DECISIVE mutation-checked assertion),
    ///   - and the `ExternalLinkSpec` is retained in the Tier-B store + surfaced.
    ///
    /// Uses Manual mode so a genuine F9 drain runs over the sheet (A1 is a bound
    /// literal edit target), proving the pin survives a REAL recalc, not a no-op.
    #[test]
    fn canonical_external_reference_pin_survives_a_recalc_and_is_ledgered() {
        let mut context = OxCalcDocumentContext::default();
        let stream = vec![
            DocumentEvent::WorkbookHeader(WorkbookHeader::new(
                DocDateSystem::Date1900,
                DocCalcMode::Manual,
            )),
            DocumentEvent::StringTable(Vec::new()),
            DocumentEvent::StyleTable(StyleTableSpec::minimal()),
            DocumentEvent::SheetBegin(SheetRef {
                sheet_id: 1,
                name: "Sheet1".to_string(),
            }),
            DocumentEvent::CellChunk(CellChunk {
                row_band: 0,
                cells: vec![
                    // A1 = 7 (a plain literal; also gives a same-sheet bound cell
                    // below so the F9 drain is genuine, not a no-op).
                    (addr(1, 1), CellPayload::Number(7.0)),
                    // B1 = =A1*3 — an ordinary bound formula (drains on F9).
                    (
                        addr(1, 2),
                        CellPayload::Formula {
                            region: None,
                            text: Some("A1*3".to_string()),
                            cached: Some(Box::new(CellPayload::Number(21.0))),
                        },
                    ),
                    // C1 = =SUM([Book2]Sheet1!A:A) — the EXTERNAL-referencing cell,
                    // cached 99. Binds, but is pinned (never evaluated).
                    (
                        addr(1, 3),
                        CellPayload::Formula {
                            region: None,
                            text: Some("SUM([Book2]Sheet1!A:A)".to_string()),
                            cached: Some(Box::new(CellPayload::Number(99.0))),
                        },
                    ),
                ],
            }),
            DocumentEvent::SheetEnd { sheet_id: 1 },
            DocumentEvent::ExternalLink(ExternalLinkSpec {
                target: "Book2.xlsx".to_string(),
            }),
        ];

        let (workspace_id, report) =
            load_workbook_model(&mut context, OxCalcWorkbookCreate::new("book:ext"), &stream)
                .unwrap();

        // The external cell BOUND (authored text retained + in the graph), and is
        // ACCOUNTED as a pin — never a bare skip (C13 / D4 §14).
        assert_eq!(
            report.formulas_bound, 2,
            "B1 and C1 both bound (C1 is external, still bound)"
        );
        assert_eq!(
            report.external_ref_cells_pinned, 1,
            "exactly one external-referencing cell pinned"
        );
        assert_eq!(
            report.external_reference_pins.len(),
            1,
            "one pin ledger row"
        );
        let pin = &report.external_reference_pins[0];
        assert_eq!(pin.address, "R1C3", "the pin is C1 (R1C3)");
        assert_eq!(
            pin.reason, EXTERNAL_REFERENCE_NOT_LINKED,
            "the named disposition"
        );
        assert_eq!(
            pin.text, "=SUM([Book2]Sheet1!A:A)",
            "authored text retained verbatim"
        );
        assert!(
            pin.had_file_cache,
            "the file carried a cache for the external cell"
        );
        // It is NOT a degradation (it bound honestly).
        assert!(
            report.bind_degradations.is_empty(),
            "an external-ref cell is a pin, not a degradation, got {:?}",
            report.bind_degradations
        );

        // The ExternalLinkSpec target is retained in the Tier-B store + surfaced.
        let facts = context.ingested_document_facts(&workspace_id).unwrap();
        assert_eq!(
            facts.external_links.len(),
            1,
            "ExternalLinkSpec retained in Tier-B"
        );
        assert_eq!(facts.external_links[0].target, "Book2.xlsx");

        // PRE-recalc: C1 renders its FileCached 99 (Manual, no engine pass).
        assert_eq!(
            published(&context, &workspace_id, 0, 1, 3),
            Some((
                CalcValue::number(99.0),
                PublishedValueProvenance::FileCached
            )),
            "C1 renders its pinned FileCached 99 pre-recalc"
        );

        // A GENUINE recalc (B1/A1 are seeded, so the drain is real, not a no-op).
        let outcome = context.recalculate_workbook(&workspace_id).unwrap();
        assert!(
            outcome.drained_any(),
            "F9 genuinely drains (B1 is bound+seeded)"
        );
        assert!(outcome.total_cells_evaluated() > 0, "cells were evaluated");

        // B1 is now engine-Calculated (7*3 = 21) — the recalc really ran.
        let (b1, b1_prov) = published(&context, &workspace_id, 0, 1, 2).unwrap();
        assert_eq!(
            b1,
            CalcValue::number(21.0),
            "B1 == A1*3 == 21 by the engine"
        );
        assert!(matches!(
            b1_prov,
            PublishedValueProvenance::Calculated { .. }
        ));

        // THE DECISIVE ASSERTION (mutation-checked): the external pin SURVIVES the
        // recalc UNCHANGED — still FileCached 99, never clobbered by an invented
        // error and never engine-evaluated (D4 §14). Were the cell not pinned, the
        // engine would have evaluated `SUM([Book2]Sheet1!A:A)` to a `#REF!`-class
        // value here and clobbered the cache — this assertion is exactly that guard.
        assert_eq!(
            published(&context, &workspace_id, 0, 1, 3),
            Some((
                CalcValue::number(99.0),
                PublishedValueProvenance::FileCached
            )),
            "the external-ref pin keeps its FileCached 99 across a genuine recalc (never clobbered)"
        );
    }

    /// An external-referencing cell with NO file cache pins a `#REF!` (D2's
    /// honesty rule — never a fabricated value), still ledgered, still surviving
    /// recalc. `had_file_cache` is `false`.
    #[test]
    fn external_reference_without_cache_pins_ref_error_ledgered() {
        let mut context = OxCalcDocumentContext::default();
        let stream = vec![
            DocumentEvent::WorkbookHeader(WorkbookHeader::new(
                DocDateSystem::Date1900,
                DocCalcMode::Manual,
            )),
            DocumentEvent::StringTable(Vec::new()),
            DocumentEvent::StyleTable(StyleTableSpec::minimal()),
            DocumentEvent::SheetBegin(SheetRef {
                sheet_id: 1,
                name: "Sheet1".to_string(),
            }),
            DocumentEvent::CellChunk(CellChunk {
                row_band: 0,
                cells: vec![(
                    addr(1, 1),
                    CellPayload::Formula {
                        region: None,
                        text: Some("[Book2]Sheet1!A:A".to_string()),
                        cached: None,
                    },
                )],
            }),
            DocumentEvent::SheetEnd { sheet_id: 1 },
        ];

        let (workspace_id, report) = load_workbook_model(
            &mut context,
            OxCalcWorkbookCreate::new("book:extnocache"),
            &stream,
        )
        .unwrap();

        assert_eq!(report.external_reference_pins.len(), 1);
        let pin = &report.external_reference_pins[0];
        assert!(
            !pin.had_file_cache,
            "no file cache backed the external cell"
        );
        assert_eq!(pin.reason, EXTERNAL_REFERENCE_NOT_LINKED);

        // The cell publishes a pinned #REF! (never fabricated, never dropped).
        assert_eq!(
            published(&context, &workspace_id, 0, 1, 1),
            Some((
                CalcValue::error(WorksheetErrorCode::Ref),
                PublishedValueProvenance::FileCached
            )),
            "a cache-less external ref pins #REF! (D2 honesty rule)"
        );
    }

    // ---- Acceptance: freshness guard (the R6.1 carry-forward) -----------------

    /// A SECOND load into a workspace already carrying a loaded workbook is
    /// rejected with a typed `WorkbookNotFreshForLoad` — NOT a silent duplicate
    /// `#workbook-settings`/`#workbook-ingest` group (the R6.1 carry-forward). The
    /// first load's content is untouched.
    #[test]
    fn second_load_into_non_fresh_workspace_is_a_typed_error_not_a_duplicate() {
        let mut context = OxCalcDocumentContext::default();
        // First load: a full workbook (settings differ from default → a
        // `#workbook-settings` group; Tier-B facts → a `#workbook-ingest` group).
        let workspace_id = context
            .create_workspace(OxCalcTreeWorkspaceCreate::new("book:refresh").as_workbook())
            .unwrap();
        let mut first = two_sheet_stream(DocCalcMode::Manual);
        // Add a Tier-B fact so a `#workbook-ingest` group lands too.
        first.push(DocumentEvent::ExternalLink(ExternalLinkSpec {
            target: "Book2.xlsx".to_string(),
        }));
        load_workbook_events(&mut context, &workspace_id, &first).unwrap();

        // The workspace now carries sheets + both meta groups. A SECOND load must
        // be a typed error, not a silent duplicate.
        let err = load_workbook_events(&mut context, &workspace_id, &first).unwrap_err();
        match err {
            super::OxCalcWorkbookIngestError::Commit(
                OxCalcDocumentError::WorkbookNotFreshForLoad {
                    workspace_id: ws, ..
                },
            ) => {
                assert_eq!(ws, "book:refresh");
            }
            other => panic!("expected WorkbookNotFreshForLoad, got {other:?}"),
        }

        // The first load's content is intact (still two sheets, no duplication).
        assert_eq!(
            context.sheets(&workspace_id).unwrap().len(),
            2,
            "the rejected second load left the first load's sheets untouched"
        );
    }

    /// The public `load_workbook_model` verb is naturally fresh-safe: two calls
    /// (with DIFFERENT workspace ids) each create their own fresh workspace, so
    /// neither hits the freshness guard — the guard fires only on a misuse
    /// (a second load into a LIVE workspace, tested above).
    #[test]
    fn public_verb_is_naturally_fresh_safe_across_two_loads() {
        let mut context = OxCalcDocumentContext::default();
        let stream = two_sheet_stream(DocCalcMode::Automatic);
        let (_ws_a, report_a) = load_workbook_model(
            &mut context,
            OxCalcWorkbookCreate::new("book:fresh-a"),
            &stream,
        )
        .unwrap();
        let (_ws_b, report_b) = load_workbook_model(
            &mut context,
            OxCalcWorkbookCreate::new("book:fresh-b"),
            &stream,
        )
        .unwrap();
        assert_eq!(report_a.sheets, 2);
        assert_eq!(report_b.sheets, 2);
    }
}

// ==== R6 follow-up (calc-5kqg.65): cross-sheet reference EVALUATION on load ====
#[cfg(test)]
mod r6_65_cross_sheet_load_tests {
    use super::{LoadRecalcPath, OxCalcWorkbookCreate, load_workbook_model};
    use crate::consumer::{OxCalcDocumentContext, OxCalcTreeWorkspaceId};
    use crate::grid::coords::ExcelGridCellAddress;
    use crate::workbook_settings::PublishedValueProvenance;
    use oxdoc_model::{
        CalcMode as DocCalcMode, CellChunk, CellPayload, DateSystem as DocDateSystem,
        DocumentEvent, PackedCellAddr, SheetRef, StyleTableSpec, WorkbookHeader,
    };
    use oxfunc_core::value::CalcValue;

    fn addr(row: u32, col: u32) -> PackedCellAddr {
        PackedCellAddr::from_one_based(row, col).unwrap()
    }

    /// A two-sheet workbook with a genuine CROSS-SHEET formula: Sheet1 has
    /// `A1=7` (a LITERAL); Sheet2 has `B1 = =Sheet1!A1+10` (cached 17). The
    /// reference is authored against the display name `Sheet1`.
    fn cross_sheet_stream(calc_mode: DocCalcMode) -> Vec<DocumentEvent> {
        vec![
            DocumentEvent::WorkbookHeader(WorkbookHeader::new(DocDateSystem::Date1900, calc_mode)),
            DocumentEvent::StringTable(Vec::new()),
            DocumentEvent::StyleTable(StyleTableSpec::minimal()),
            DocumentEvent::SheetBegin(SheetRef {
                sheet_id: 1,
                name: "Sheet1".to_string(),
            }),
            DocumentEvent::CellChunk(CellChunk {
                row_band: 0,
                cells: vec![(addr(1, 1), CellPayload::Number(7.0))],
            }),
            DocumentEvent::SheetEnd { sheet_id: 1 },
            DocumentEvent::SheetBegin(SheetRef {
                sheet_id: 2,
                name: "Sheet2".to_string(),
            }),
            DocumentEvent::CellChunk(CellChunk {
                row_band: 0,
                cells: vec![(
                    addr(1, 2),
                    CellPayload::Formula {
                        region: None,
                        text: Some("Sheet1!A1+10".to_string()),
                        cached: Some(Box::new(CellPayload::Number(17.0))),
                    },
                )],
            }),
            DocumentEvent::SheetEnd { sheet_id: 2 },
        ]
    }

    /// The published `(value, provenance)` at `(row, col)` on sheet index `si`.
    fn published(
        context: &OxCalcDocumentContext,
        workspace_id: &OxCalcTreeWorkspaceId,
        si: usize,
        row: u32,
        col: u32,
    ) -> Option<(CalcValue, PublishedValueProvenance)> {
        let node = context.sheets(workspace_id).unwrap()[si].node_id;
        let view = context.grid_view(workspace_id, node).unwrap().unwrap();
        view.cells
            .iter()
            .find(|cell| cell.address.row == row && cell.address.col == col)
            .map(|cell| (cell.value.clone(), cell.provenance))
    }

    /// An [`ExcelGridCellAddress`] on sheet index `si`, with the workbook/sheet
    /// tokens the ingest builder derives (`book:{workspace}` / `sheet:{node_id}`),
    /// so a host edit (`enter_grid_cell`) targets the right loaded grid.
    fn loaded_address(
        context: &OxCalcDocumentContext,
        workspace_id: &OxCalcTreeWorkspaceId,
        si: usize,
        row: u32,
        col: u32,
    ) -> ExcelGridCellAddress {
        let node = context.sheets(workspace_id).unwrap()[si].node_id;
        ExcelGridCellAddress::new(
            format!("book:{}", workspace_id.as_str()),
            format!("sheet:{}", node.0),
            row,
            col,
        )
    }

    /// Sheet index `si`'s grid is differential-clean (the open-recalc's dirty-vs-
    /// mark-all lanes agreed — the two-model discipline over the load path).
    fn assert_differential_clean(
        context: &OxCalcDocumentContext,
        workspace_id: &OxCalcTreeWorkspaceId,
        si: usize,
    ) {
        let node = context.sheets(workspace_id).unwrap()[si].node_id;
        let view = context.grid_view(workspace_id, node).unwrap().unwrap();
        assert!(
            view.differential_mismatches.is_empty(),
            "sheet {si} is differential-clean, got {:?}",
            view.differential_mismatches
        );
    }

    /// ACCEPTANCE 1 (calc-5kqg.65): a freshly-loaded cross-sheet formula
    /// evaluates to the engine value under an Automatic open-recalc, differential
    /// clean, and the literal target sheet renders its value (a literal-only
    /// sheet is never drained, so its authored value is published at staging).
    #[test]
    fn automatic_load_evaluates_cross_sheet_reference() {
        let mut context = OxCalcDocumentContext::default();
        let (workspace_id, report) = load_workbook_model(
            &mut context,
            OxCalcWorkbookCreate::new("book:xsheet"),
            &cross_sheet_stream(DocCalcMode::Automatic),
        )
        .unwrap();

        assert_eq!(report.recalc_path, LoadRecalcPath::Automatic);
        assert_eq!(report.sheets, 2);
        assert_eq!(
            report.formulas_bound, 1,
            "the one cross-sheet formula bound"
        );
        assert!(
            report.bind_degradations.is_empty(),
            "the cross-sheet formula binds (existence-blind), it does not degrade"
        );

        // The literal-only target sheet renders its authored value (FileCached —
        // published at staging, never engine-evaluated), so the cross-sheet gather
        // has a value to resolve against.
        assert_eq!(
            published(&context, &workspace_id, 0, 1, 1),
            Some((CalcValue::number(7.0), PublishedValueProvenance::FileCached)),
            "Sheet1!A1 literal renders 7 (FileCached, staged)"
        );

        // THE cross-sheet acceptance: Sheet2!B1 resolves Sheet1!A1 across sheets.
        let (s2b1, s2b1_prov) = published(&context, &workspace_id, 1, 1, 2).unwrap();
        assert_eq!(
            s2b1,
            CalcValue::number(17.0),
            "Sheet2!B1 = Sheet1!A1 + 10 = 7 + 10 = 17 (cross-sheet engine value)"
        );
        assert!(
            matches!(s2b1_prov, PublishedValueProvenance::Calculated { .. }),
            "the cross-sheet dependent is engine-Calculated by the open-recalc, got {s2b1_prov:?}"
        );
        assert_differential_clean(&context, &workspace_id, 1);
    }

    /// ACCEPTANCE 2 (calc-5kqg.65): after the load, a cross-sheet EDIT to the
    /// target (`Sheet1!A1`) propagates through the workbook closure to the
    /// dependent on the other sheet (`Sheet2!B1`) automatically (R4.6 over the
    /// loaded grids, which are keyed by node token — the edge re-key bridges the
    /// token space to the reference's display name).
    #[test]
    fn cross_sheet_edit_after_load_propagates() {
        let mut context = OxCalcDocumentContext::default();
        let (workspace_id, _report) = load_workbook_model(
            &mut context,
            OxCalcWorkbookCreate::new("book:xedit"),
            &cross_sheet_stream(DocCalcMode::Automatic),
        )
        .unwrap();

        // Baseline from the load: Sheet2!B1 = 7 + 10 = 17.
        assert_eq!(
            published(&context, &workspace_id, 1, 1, 2).map(|(v, _)| v),
            Some(CalcValue::number(17.0)),
        );

        // Edit Sheet1!A1 -> 99. Under Automatic the edit auto-recalcs and drives
        // the cross-sheet closure: Sheet2!B1 must re-resolve to 99 + 10 = 109.
        let s1_a1 = loaded_address(&context, &workspace_id, 0, 1, 1);
        let s1_node = context.sheets(&workspace_id).unwrap()[0].node_id;
        context
            .enter_grid_cell(&workspace_id, s1_node, &s1_a1, "99")
            .unwrap()
            .unwrap();

        assert_eq!(
            published(&context, &workspace_id, 0, 1, 1).map(|(v, _)| v),
            Some(CalcValue::number(99.0)),
            "Sheet1!A1 edited to 99",
        );
        let (s2b1, s2b1_prov) = published(&context, &workspace_id, 1, 1, 2).unwrap();
        assert_eq!(
            s2b1,
            CalcValue::number(109.0),
            "the Sheet1!A1 edit propagated cross-sheet: Sheet2!B1 = 99 + 10 = 109"
        );
        assert!(
            matches!(s2b1_prov, PublishedValueProvenance::Calculated { .. }),
            "the propagated dependent is engine-Calculated, got {s2b1_prov:?}"
        );
        assert_differential_clean(&context, &workspace_id, 1);
    }

    /// A cross-sheet RANGE reference resolves for a LOADED workbook (W062 R6.67 /
    /// calc-5kqg.67): Sheet1 has `A1=1, A2=2, A3=3` (literals); Sheet2 has
    /// `B1 = =SUM(Sheet1!A1:A3)`, which the Automatic open-recalc resolves to 6
    /// through the cross-sheet range view (the token-keyed loaded-grid analogue of
    /// the authored range test in `consumer.rs`).
    #[test]
    fn automatic_load_evaluates_cross_sheet_range() {
        let stream = vec![
            DocumentEvent::WorkbookHeader(WorkbookHeader::new(
                DocDateSystem::Date1900,
                DocCalcMode::Automatic,
            )),
            DocumentEvent::StringTable(Vec::new()),
            DocumentEvent::StyleTable(StyleTableSpec::minimal()),
            DocumentEvent::SheetBegin(SheetRef {
                sheet_id: 1,
                name: "Sheet1".to_string(),
            }),
            DocumentEvent::CellChunk(CellChunk {
                row_band: 0,
                cells: vec![
                    (addr(1, 1), CellPayload::Number(1.0)),
                    (addr(2, 1), CellPayload::Number(2.0)),
                    (addr(3, 1), CellPayload::Number(3.0)),
                ],
            }),
            DocumentEvent::SheetEnd { sheet_id: 1 },
            DocumentEvent::SheetBegin(SheetRef {
                sheet_id: 2,
                name: "Sheet2".to_string(),
            }),
            DocumentEvent::CellChunk(CellChunk {
                row_band: 0,
                cells: vec![(
                    addr(1, 2),
                    CellPayload::Formula {
                        region: None,
                        text: Some("SUM(Sheet1!A1:A3)".to_string()),
                        cached: Some(Box::new(CellPayload::Number(6.0))),
                    },
                )],
            }),
            DocumentEvent::SheetEnd { sheet_id: 2 },
        ];
        let mut context = OxCalcDocumentContext::default();
        let (workspace_id, _report) = load_workbook_model(
            &mut context,
            OxCalcWorkbookCreate::new("book:xrange"),
            &stream,
        )
        .unwrap();

        let (s2b1, s2b1_prov) = published(&context, &workspace_id, 1, 1, 2).unwrap();
        assert_eq!(
            s2b1,
            CalcValue::number(6.0),
            "Sheet2!B1 = SUM(Sheet1!A1:A3) = 6 (cross-sheet range engine value)"
        );
        assert!(
            matches!(s2b1_prov, PublishedValueProvenance::Calculated { .. }),
            "the cross-sheet range dependent is engine-Calculated, got {s2b1_prov:?}"
        );
        assert_differential_clean(&context, &workspace_id, 1);
    }

    /// A Manual-mode load runs ZERO engine passes: the cross-sheet dependent
    /// renders its FileCached cache (17), NOT an engine value. The first explicit
    /// F9 then resolves it across sheets to the engine value (still 17 here, but
    /// now `Calculated`), proving the same cross-sheet wiring drives the F9 drain.
    #[test]
    fn manual_load_renders_cache_then_f9_resolves_cross_sheet() {
        let mut context = OxCalcDocumentContext::default();
        let (workspace_id, report) = load_workbook_model(
            &mut context,
            OxCalcWorkbookCreate::new("book:xmanual"),
            &cross_sheet_stream(DocCalcMode::Manual),
        )
        .unwrap();

        assert_eq!(report.recalc_path, LoadRecalcPath::Manual);
        assert_eq!(
            report.engine_recalcs_at_load, 0,
            "a Manual load runs zero engine passes"
        );
        assert_eq!(
            published(&context, &workspace_id, 1, 1, 2),
            Some((
                CalcValue::number(17.0),
                PublishedValueProvenance::FileCached
            )),
            "Sheet2!B1 renders its FileCached cache 17 before any F9"
        );

        // First F9: the cross-sheet dependent resolves through the engine.
        context.recalculate_workbook(&workspace_id).unwrap();
        let (s2b1, s2b1_prov) = published(&context, &workspace_id, 1, 1, 2).unwrap();
        assert_eq!(
            s2b1,
            CalcValue::number(17.0),
            "F9 resolves Sheet2!B1 = Sheet1!A1 + 10 = 17 across sheets"
        );
        assert!(
            matches!(s2b1_prov, PublishedValueProvenance::Calculated { .. }),
            "after F9 the cross-sheet dependent is engine-Calculated, got {s2b1_prov:?}"
        );
        assert_differential_clean(&context, &workspace_id, 1);
    }
}
