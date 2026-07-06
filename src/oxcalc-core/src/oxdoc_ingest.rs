//! oxdoc-model workbook ingestion — the load half of the document surface
//! (W062 R6, D4 §§8-13).
//!
//! This module owns the [`OxCalcIngestSink`] implementation that drives an
//! `oxdoc-model` [`DocumentEvent`] stream into the OxCalc structural model. The
//! module boundary is deliberately strict (D4 §8): only `consumer.rs` types and
//! `oxdoc_model` types cross it, so a later crate split (if wasm size ever
//! argues for one) is mechanical.
//!
//! **Scope through R6.2.** The Tier-A prelude subset (workbook header settings,
//! sheet lifecycle, *literal* cells) plus **formula ingest**: `SpreadsheetMlA1`
//! and `R1C1RelativeTemplate` cells bind through the single key mint (the
//! derived-key doctrine), `SharedFormulaRegion` expands as a repeated-formula
//! region, `FormulaTopology` routes record kinds (shared / legacy-CSE array /
//! data-table / unknown), `FileCached` caches seed the pre-engine publication,
//! and OxFml-unacceptable text *degrades* (retained text + cache + a
//! [`BindDegradation`] ledger row) rather than failing the load (D4 §10). Names,
//! tables, merges (R6.3), the full Tier-B inert store (R6.4), the public
//! one-call verb (R6.5), and output projection (R6.6) are later beads. Every
//! `DocumentEvent` variant is nonetheless *accounted* for here — consumed
//! (Tier A) or recorded with a ledger row (Tier B/X) — so nothing is ever
//! silently dropped (D4 §12).
//!
//! **The honesty enforcement (D4 §12).** [`OxCalcWorkbookIngestSink::feature`]
//! ends in an *exhaustive* match over [`OxCalcDocumentFeature`] with **no
//! wildcard arm**. A 30th upstream feature variant is therefore a compile error
//! in this module, not a silent drop — that is the C13 tripwire that keeps the
//! `#[non_exhaustive]` growth of `DocumentEvent` loud.

use std::collections::BTreeMap;

use oxdoc_model::{
    DocumentEvent, FormulaRecord, FormulaRecordKind, FormulaTextKind, OxCalcCachedValue,
    OxCalcCellChunk, OxCalcCellInput, OxCalcCellValue, OxCalcDocumentFeature, OxCalcFormulaInput,
    OxCalcIngestError, OxCalcIngestSink, OxCalcWorkbookPrelude, SharedFormulaRegion, SheetRef,
    drive_oxcalc_ingest,
};
use oxfml_core::source::FormulaChannelKind;
use oxfunc_core::value::{CalcValue, ExcelText, WorksheetErrorCode};

use crate::consumer::{OxCalcDocumentContext, OxCalcDocumentError, OxCalcTreeWorkspaceId};
use crate::grid::authored::GridAuthoredCell;
use crate::grid::coords::ExcelGridCellAddress;
use crate::workbook_settings::{CalcMode, DateSystem, WorkbookCalcSettings};

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
    /// `"consumed"`, `"retained-inert-stub"`, `"excluded"`). Mirrors the D4 §12
    /// disposition column at code granularity.
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
    /// Number of `RichStub` cells observed. Deferred to R6.4 (inert `RichObject`
    /// retention); ledgered here as deferred, never consumed as a fake value.
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
    /// Number of defined names installed. Always 0 in this bead (R6.3).
    pub names: u32,
    /// Number of tables installed. Always 0 in this bead (R6.3).
    pub tables: u32,
    /// The ingest fidelity ledger: one row per *observed* variant, in
    /// disposition-table order.
    pub ledger: Vec<IngestLedgerRow>,
    /// Formula bind degradations (D4 §10): one row per formula cell whose text
    /// OxFml rejected as a formula. The authored text is retained here (never
    /// discarded), the cell publishes its `FileCached` cache (or a `#NAME?`-class
    /// error), and ingest still SUCCEEDS. Empty when every formula bound.
    pub bind_degradations: Vec<BindDegradation>,
    /// Inert overlay rects claimed at load (D4 §12 rows 21/22, §13): legacy-CSE
    /// array rects claim an inert `Cse` overlay (the array cells ingest as normal
    /// formulas; the overlay carries **no** legacy-CSE eval semantics). The live
    /// `GridOverlayExtension` projection into the engine's overlay set is R6.4's
    /// (it owns the Tier-B store the overlay indexes into); this bead records the
    /// claim as a load fact so the disposition is inspectable and round-trip-safe.
    pub inert_overlays: Vec<IngestedInertOverlay>,
    /// Which load-recalc path ran (D4 §9). Load binds formulas and seeds
    /// `FileCached` publications but does **not** issue the open-recalc (that
    /// policy is R6.5); the workbook renders from caches until an explicit
    /// `recalculate_workbook`, so this is always [`LoadRecalcPath::None`].
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

/// An inert overlay rect claimed at load (D4 §12 rows 19/21/22, §13).
///
/// The rect-claiming Tier-B families (legacy-CSE arrays here; conditional
/// formats and rich objects in R6.4) claim an inert overlay rect: a spatial
/// index into the retained document facts, carrying **no** engine calc
/// semantics. This bead records only the legacy-CSE `Cse` claim (array cells
/// ingest as normal formulas alongside it); R6.4 projects these claims into live
/// `GridOverlayExtension` seats and adds the CF/RichObject families.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IngestedInertOverlay {
    /// Which overlay family claims the rect. Legacy-CSE arrays claim
    /// [`InertOverlayKind::Cse`].
    pub kind: InertOverlayKind,
    /// The upstream sheet id the rect belongs to (`FormulaTopology.sheet_id`).
    pub sheet_id: u32,
    /// The claimed rectangle in one-based `(top_row, left_col, bottom_row,
    /// right_col)` coordinates.
    pub rect: (u32, u32, u32, u32),
}

/// The overlay family of an [`IngestedInertOverlay`] (D4 §13). Only `Cse` is
/// produced in R6.2; `ConditionalFormat` and `RichObject` join in R6.4.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InertOverlayKind {
    /// A legacy-CSE array rect. The cells inside ingest as normal formulas; the
    /// overlay itself is inert (no array-formula eval semantics are built).
    Cse,
}

/// Which recalc path the load ran (D4 §9).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadRecalcPath {
    /// No recalc issued (this bead — formula binding is R6.2).
    None,
    /// `CalcMode::Automatic` open-recalc (R6.2+).
    Automatic,
    /// `CalcMode::Manual` — renders from caches until F9 (R6.2+).
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
    /// Inert `Cse` overlay claims (row 22): the array rect a legacy-CSE record
    /// covers. The array cells ingest as normal formulas / a repeated region;
    /// this records the inert spatial claim for the load report.
    cse_overlays: Vec<IngestedInertOverlay>,
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
    /// deferred to R6.4, surfaced in the report as deferred, never consumed.
    rich_stubs_deferred: u32,
    /// Per-address routing overrides declared by `FormulaTopology` (D4 §12 row
    /// 22), keyed by `(upstream_sheet_id, row_one_based, col_one_based)`. The
    /// stream validator requires `FormulaTopology` to precede the sheet's cell
    /// chunk, so these are populated before the matching `cell_chunk` formula
    /// arrives; `cell_chunk` consults the map to route a DataTable/Unknown cell
    /// away from binding (publish cached + ledger `NotCalcModeled`).
    topology_overrides: BTreeMap<(u32, u32, u32), TopologyRoute>,
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
        let inert_overlays: Vec<IngestedInertOverlay> = self
            .sheets
            .iter()
            .flat_map(|sheet| sheet.cse_overlays.iter().cloned())
            .collect();
        let rich_stubs_deferred = self.rich_stubs_deferred;

        // The single-transaction builder on the context (consumer.rs) mints ONE
        // revision for the whole load. It is the only place that touches
        // consumer-private state; the sink hands it a plain plan and gets back
        // the bind outcome (degradations + bound-formula count) for the report.
        let plan = WorkbookTierALoadPlan {
            settings,
            sheets: self
                .sheets
                .into_iter()
                .map(|sheet| SheetTierALoad {
                    display_name: sheet.display_name,
                    upstream_sheet_id: sheet.upstream_sheet_id,
                    literals: sheet.literals,
                    formulas: sheet.formulas,
                    repeated_regions: sheet.repeated_regions,
                    region_cell_caches: sheet.region_cell_caches,
                    unmodeled_cached: sheet.unmodeled_cached,
                })
                .collect(),
        };
        let outcome = context.commit_workbook_tier_a_load(workspace_id, plan)?;

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
            names: 0,
            tables: 0,
            ledger,
            bind_degradations: outcome.bind_degradations,
            inert_overlays,
            // Load binds + seeds FileCached but does not open-recalc (R6.5 owns
            // that policy). The workbook renders from caches until an explicit
            // `recalculate_workbook`.
            //
            // R6.5 CARRY-FORWARD (owner-annotated): this `None` label coexists
            // with the commit builder unconditionally running a two-lane
            // (reference + optimized) recalc PER SHEET at load to establish the
            // graph, retained valuation, and the load-time differential. That
            // internal recalc collides with R6.5's Manual-mode "zero engine runs"
            // acceptance. R6.5 resolves the tension (e.g. Manual load skips the
            // load-recalc and renders purely from caches); R6.2 leaves it here as
            // a named seam, not a silent contradiction.
            recalc_path: LoadRecalcPath::None,
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
        let sheet_index = self.open_sheet.ok_or_else(|| {
            OxCalcWorkbookIngestError::Rejected(
                "formula topology arrived with no open sheet".to_string(),
            )
        })?;
        for record in &topology.records {
            self.route_formula_record(sheet_index, topology.sheet_id, record);
        }
        // `unsupported_fragments` are retained verbatim in R6.4's Tier-B store;
        // this bead accounts for the topology via the routed dispositions above
        // and the `routed-topology` ledger row on the FormulaTopology variant.
        Ok(())
    }

    /// Route one `FormulaRecord` by kind (D4 §12 row 22). See
    /// [`Self::route_formula_topology`] for the per-kind contract.
    fn route_formula_record(
        &mut self,
        sheet_index: usize,
        sheet_id: u32,
        record: &FormulaRecord,
    ) {
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
                // legacy-CSE eval semantics are built (the overlay is inert).
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
                self.sheets[sheet_index].cse_overlays.push(IngestedInertOverlay {
                    kind: InertOverlayKind::Cse,
                    sheet_id,
                    rect,
                });
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
        // verbatim — stub in this bead, R6.4 owns the store).
        self.settings = Some(WorkbookCalcSettings {
            date_system: map_date_system(prelude.header.date_system),
            calc_mode: map_calc_mode(prelude.header.calc_mode),
            ..WorkbookCalcSettings::default()
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

        // StyleTable: retained verbatim in R6.4's inert store; a stub row here.
        self.ledger_and_observe(
            DocumentVariantTag::StyleTable,
            IngestTier::B,
            "retained-inert-stub",
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
        // D4 §12 row 23: CellChunk (A). Literals → typed CalcValue; Formula →
        // staged for bind-or-degrade at commit (D4 §10), carrying its FileCached
        // cache; Empty → no record; RichStub → deferred to R6.4 (inert
        // RichObject retention), counted honestly, never consumed as a fake
        // value.
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
                    match self
                        .topology_overrides
                        .get(&(upstream_sheet_id, row, col))
                    {
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
                    // Deferred to R6.4 (inert RichObject retention). Counted so the
                    // report ledgers it as deferred, never as consumed.
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
        // In this bead every non-Tier-A arm is a ledger-and-continue stub (the
        // real dispositions land in R6.2/R6.3/R6.4); Tier X CalcChainHint is
        // ledgered-and-dropped now. Do NOT add a `_ =>` arm.
        match feature {
            OxCalcDocumentFeature::SheetDimension(_) => self.ledger_and_observe(
                DocumentVariantTag::SheetDimension,
                IngestTier::B,
                "retained-inert-stub",
            ),
            OxCalcDocumentFeature::ColumnProps(_) => self.ledger_and_observe(
                DocumentVariantTag::ColumnProps,
                IngestTier::B,
                "retained-inert-stub",
            ),
            OxCalcDocumentFeature::RowProps(_) => self.ledger_and_observe(
                DocumentVariantTag::RowProps,
                IngestTier::B,
                "retained-inert-stub",
            ),
            OxCalcDocumentFeature::MergedCellRegions(_) => self.ledger_and_observe(
                DocumentVariantTag::MergedCellRegions,
                IngestTier::A,
                "deferred-install-r6.3",
            ),
            OxCalcDocumentFeature::SheetViewState(_) => self.ledger_and_observe(
                DocumentVariantTag::SheetViewState,
                IngestTier::B,
                "retained-inert-stub",
            ),
            OxCalcDocumentFeature::Hyperlinks(_) => self.ledger_and_observe(
                DocumentVariantTag::Hyperlinks,
                IngestTier::B,
                "retained-inert-stub",
            ),
            OxCalcDocumentFeature::DataValidations(_) => self.ledger_and_observe(
                DocumentVariantTag::DataValidations,
                IngestTier::B,
                "retained-inert-stub",
            ),
            OxCalcDocumentFeature::AutoFilter(_) => self.ledger_and_observe(
                DocumentVariantTag::AutoFilter,
                IngestTier::B,
                "retained-inert-stub",
            ),
            OxCalcDocumentFeature::SortState(_) => self.ledger_and_observe(
                DocumentVariantTag::SortState,
                IngestTier::B,
                "retained-inert-stub",
            ),
            OxCalcDocumentFeature::CommentNotice(_) => self.ledger_and_observe(
                DocumentVariantTag::CommentNotice,
                IngestTier::B,
                "retained-inert-stub",
            ),
            OxCalcDocumentFeature::ThreadedCommentPeople(_) => self.ledger_and_observe(
                DocumentVariantTag::ThreadedCommentPeople,
                IngestTier::B,
                "retained-inert-stub",
            ),
            OxCalcDocumentFeature::SheetReviewComments(_) => self.ledger_and_observe(
                DocumentVariantTag::SheetReviewComments,
                IngestTier::B,
                "retained-inert-stub",
            ),
            OxCalcDocumentFeature::DrawingFormControls(_) => self.ledger_and_observe(
                DocumentVariantTag::DrawingFormControls,
                IngestTier::B,
                "retained-inert-stub",
            ),
            OxCalcDocumentFeature::CellFormatRuns(_) => self.ledger_and_observe(
                DocumentVariantTag::CellFormatRuns,
                IngestTier::B,
                "retained-inert-stub",
            ),
            OxCalcDocumentFeature::DifferentialStyleTable(_) => self.ledger_and_observe(
                DocumentVariantTag::DifferentialStyleTable,
                IngestTier::B,
                "retained-inert-stub",
            ),
            OxCalcDocumentFeature::ConditionalFormatRegion(_) => self.ledger_and_observe(
                DocumentVariantTag::ConditionalFormatRegion,
                IngestTier::B,
                "retained-inert-stub",
            ),
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
            OxCalcDocumentFeature::TableOverlay(_) => self.ledger_and_observe(
                DocumentVariantTag::TableOverlay,
                IngestTier::A,
                "deferred-install-r6.3",
            ),
            OxCalcDocumentFeature::DefinedName(_) => self.ledger_and_observe(
                DocumentVariantTag::DefinedName,
                IngestTier::A,
                "deferred-install-r6.3",
            ),
            OxCalcDocumentFeature::ExternalLink(_) => self.ledger_and_observe(
                DocumentVariantTag::ExternalLink,
                IngestTier::B,
                "retained-inert-stub",
            ),
            OxCalcDocumentFeature::CalcChainHint(_) => self.ledger_and_observe(
                DocumentVariantTag::CalcChainHint,
                IngestTier::X,
                "excluded-engine-derives-order",
            ),
            OxCalcDocumentFeature::OpaquePartNotice(_) => self.ledger_and_observe(
                DocumentVariantTag::OpaquePartNotice,
                IngestTier::B,
                "retained-inert-stub",
            ),
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

// -- The single-transaction load plan (consumed by consumer.rs's builder) -----

/// The Tier-A load plan handed to the consumer's single-transaction builder
/// (D4 §9). Plain data: settings + ordered sheets, each with its literal cells,
/// formula cells, repeated regions, and non-calc-modeled cached publications.
/// The builder binds formulas (single key mint), seeds `FileCached`
/// publications, and installs everything in one revision.
#[derive(Debug, Clone)]
pub struct WorkbookTierALoadPlan {
    pub settings: WorkbookCalcSettings,
    pub sheets: Vec<SheetTierALoad>,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consumer::{OxCalcDocumentContext, OxCalcTreeWorkspaceCreate};
    use oxdoc_model::{
        AutoFilterSpec, CalcMode as DocCalcMode, CellChunk, CellFormatRun, CellPayload,
        CommentNoticeKind, CommentNoticeSpec, ConditionalFormatRegion, DataValidationsSpec,
        DateSystem as DocDateSystem, DefinedNameMetadataSpec, DefinedNameSpec,
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
        assert_eq!(report.recalc_path, LoadRecalcPath::None);

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
        // published value; the raw-byte retention + ledger row is R6.4's store).
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

    use oxdoc_model::{
        ArrayFormulaSpec, CachedValueProvenance, CellRangeSpec, DataTableFormulaSpec,
        FormulaCachedValueState, FormulaRecord, FormulaRecordAttributes, FormulaRecordKind,
    };
    use crate::workbook_settings::PublishedValueProvenance;

    fn addr(row: u32, col: u32) -> PackedCellAddr {
        PackedCellAddr::from_one_based(row, col).unwrap()
    }

    /// The prelude every formula fixture opens with (Automatic mode, as W011).
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

    /// W011: `Sheet1!A1 = 7` (literal), `B1 = =A1*3` (formula) with a FileCached
    /// cache of 21. PRE-recalc B1 renders the FileCached 21; the load is
    /// differential-clean; an explicit `recalculate_workbook` replaces the cache
    /// with the engine's own 21 (`Calculated`).
    #[test]
    fn w011_fixture_loads_filecached_then_recalcs_by_engine() {
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
                ],
            }),
            DocumentEvent::SheetEnd { sheet_id: 1 },
        ]);

        let report = load_workbook_events(&mut context, &workspace_id, &stream).unwrap();
        assert_eq!(report.cells, 1, "one literal (A1)");
        assert_eq!(report.formulas_bound, 1, "B1 bound through the single mint");
        assert!(report.bind_degradations.is_empty(), "B1 is a valid formula");
        assert_eq!(report.recalc_path, LoadRecalcPath::None, "load does not open-recalc");

        // PRE-recalc: B1 renders the FileCached 21, tagged FileCached (pre-engine).
        assert_eq!(
            published_value(&context, &workspace_id, 1, 2),
            Some((CalcValue::number(21.0), PublishedValueProvenance::FileCached)),
            "B1 renders its FileCached cache pre-recalc"
        );
        // A1 (a literal) is authored truth, engine-Calculated by the load recalc.
        assert!(
            matches!(
                published_value(&context, &workspace_id, 1, 1).map(|(_, p)| p),
                Some(PublishedValueProvenance::Calculated { .. })
            ),
            "the A1 literal is engine-calculated, not FileCached"
        );

        // The load is differential-clean (the load-recalc ran both engines; the
        // FileCached B1 is invisible to the differential by construction, C15).
        let node = context.sheets(&workspace_id).unwrap()[0].node_id;
        let view = context.grid_view(&workspace_id, node).unwrap().unwrap();
        assert!(
            view.differential_mismatches.is_empty(),
            "ingested formula sheet is differential-clean, got {:?}",
            view.differential_mismatches
        );

        // The authored formula text round-trips (leading `=` restored).
        assert_eq!(
            authored_source_text(&context, &workspace_id, 1, 2).as_deref(),
            Some("=A1*3"),
        );

        // Explicit recalc (F9): the seeded formula cell drains and B1 is replaced
        // by the engine's own value — 21, now Calculated (the FileCached cache is
        // gone).
        let outcome = context.recalculate_workbook(&workspace_id).unwrap();
        assert!(outcome.drained_any(), "F9 drains the seeded formula cell");
        let (value, provenance) = published_value(&context, &workspace_id, 1, 2).unwrap();
        assert_eq!(value, CalcValue::number(21.0), "B1 == A1*3 == 21 by the engine");
        assert!(
            matches!(provenance, PublishedValueProvenance::Calculated { .. }),
            "post-recalc B1 is engine-Calculated, not FileCached"
        );
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
        assert_eq!(report.formulas_bound, 1, "the array cell bound as a normal formula");
        assert_eq!(report.not_calc_modeled, 0);
        // Exactly one inert Cse overlay rect claims the array range A1:A2.
        assert_eq!(report.inert_overlays.len(), 1, "one inert Cse overlay claim");
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
        assert_eq!(report.not_calc_modeled, 1, "the DataTable cell is NotCalcModeled");
        assert!(report.inert_overlays.is_empty());

        // The cell publishes its FileCached value and never evaluates.
        assert_eq!(
            published_value(&context, &workspace_id, 1, 1),
            Some((CalcValue::number(42.0), PublishedValueProvenance::FileCached)),
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
            Some((CalcValue::logical(true), PublishedValueProvenance::FileCached)),
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
            Some((CalcValue::number(99.0), PublishedValueProvenance::FileCached)),
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
        assert_eq!(report.formulas_bound, 1, "the formula bound — a distinguishing signal");
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
        let mut stream = formula_prelude();
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
        let mut stream = formula_prelude();
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
            Some((CalcValue::number(42.0), PublishedValueProvenance::FileCached)),
            "C1 renders its FileCached cache pre-recalc"
        );

        // A GENUINE drain (B1 is seeded).
        let outcome = context.recalculate_workbook(&workspace_id).unwrap();
        assert!(outcome.drained_any(), "F9 genuinely drains (B1 is bound+seeded)");

        // B1 is now engine-Calculated (7*3 = 21).
        let (b1, b1_prov) = published_value(&context, &workspace_id, 1, 2).unwrap();
        assert_eq!(b1, CalcValue::number(21.0), "B1 == A1*3 == 21 by the engine");
        assert!(
            matches!(b1_prov, PublishedValueProvenance::Calculated { .. }),
            "B1 is engine-Calculated after the drain"
        );

        // THE PIN SURVIVES: C1 STILL renders its FileCached 42 — the drain that
        // rebuilt `published` from the engine readout did not erase it.
        assert_eq!(
            published_value(&context, &workspace_id, 1, 3),
            Some((CalcValue::number(42.0), PublishedValueProvenance::FileCached)),
            "the DataTable pin survives a genuine F9 drain"
        );
    }

    /// The corrupt-degraded variant of the pin-survival contract: a bound formula
    /// (F9 drains) PLUS a corrupt-degraded formula. After a real recalc the bound
    /// formula is Calculated and the degraded cell STILL renders its pinned value.
    #[test]
    fn canonical_degraded_pin_survives_a_genuine_recalc_drain() {
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
            Some((CalcValue::number(99.0), PublishedValueProvenance::FileCached)),
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
            Some((CalcValue::number(99.0), PublishedValueProvenance::FileCached)),
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
        assert_eq!(value, CalcValue::number(3.0), "the repaired formula evaluates to 3");
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
            Some((CalcValue::number(42.0), PublishedValueProvenance::FileCached)),
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
        assert_eq!(report.formulas_bound, 0, "no region was installed to bind it");
        // Its cache still publishes (so the cell renders), pinned (no formula
        // binds it, so a transient publication would be erased by recalc).
        assert_eq!(
            published_value(&context, &workspace_id, 1, 2),
            Some((CalcValue::number(15.0), PublishedValueProvenance::FileCached)),
            "the dangling cell renders its cache, accounted as unbacked"
        );
    }
}
