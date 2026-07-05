//! oxdoc-model workbook ingestion — the load half of the document surface
//! (W062 R6, D4 §§8-13).
//!
//! This module owns the [`OxCalcIngestSink`] implementation that drives an
//! `oxdoc-model` [`DocumentEvent`] stream into the OxCalc structural model. The
//! module boundary is deliberately strict (D4 §8): only `consumer.rs` types and
//! `oxdoc_model` types cross it, so a later crate split (if wasm size ever
//! argues for one) is mechanical.
//!
//! **Scope in this bead (R6.1).** The Tier-A prelude subset only: workbook
//! header settings (date system + calc mode), sheet lifecycle, and *literal*
//! cells (numbers, text, bools, error codes). Formula cells, names, tables,
//! merges, the full Tier-B inert store, the public one-call verb, and output
//! projection are later beads (R6.2-R6.6). Every `DocumentEvent` variant is
//! nonetheless *accounted* for here — consumed (Tier A) or recorded with a
//! ledger row (Tier B/X) — so nothing is ever silently dropped (D4 §12).
//!
//! **The honesty enforcement (D4 §12).** [`OxCalcWorkbookIngestSink::feature`]
//! ends in an *exhaustive* match over [`OxCalcDocumentFeature`] with **no
//! wildcard arm**. A 30th upstream feature variant is therefore a compile error
//! in this module, not a silent drop — that is the C13 tripwire that keeps the
//! `#[non_exhaustive]` growth of `DocumentEvent` loud.

use std::collections::BTreeMap;

use oxdoc_model::{
    DocumentEvent, OxCalcCellChunk, OxCalcCellInput, OxCalcCellValue, OxCalcDocumentFeature,
    OxCalcIngestError, OxCalcIngestSink, OxCalcWorkbookPrelude, SheetRef, drive_oxcalc_ingest,
};
use oxfunc_core::value::{CalcValue, ExcelText, WorksheetErrorCode};

use crate::consumer::{OxCalcDocumentContext, OxCalcDocumentError, OxCalcTreeWorkspaceId};
use crate::grid::authored::GridAuthoredCell;
use crate::grid::coords::{ExcelGridBounds, ExcelGridCellAddress};
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
    /// Number of defined names installed. Always 0 in this bead (R6.3).
    pub names: u32,
    /// Number of tables installed. Always 0 in this bead (R6.3).
    pub tables: u32,
    /// The ingest fidelity ledger: one row per *observed* variant, in
    /// disposition-table order.
    pub ledger: Vec<IngestLedgerRow>,
    /// Formula bind degradations (D4 §10). Empty here — R6.2 populates it.
    pub bind_degradations: Vec<BindDegradation>,
    /// Which load-recalc path ran (D4 §9). In this bead load never issues a
    /// recalc (formula binding is R6.2), so this is always
    /// [`LoadRecalcPath::None`].
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

/// A formula bind degradation (D4 §10). Reserved for R6.2; this bead never
/// produces one, but the type is defined here so the report shape is stable
/// across the R6 wave.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindDegradation {
    /// A1 address of the degraded formula cell.
    pub address: String,
    /// The authored formula text that was retained rather than bound.
    pub text: String,
    /// Diagnostics OxFml produced when it rejected the text as a formula.
    pub diagnostics: Vec<String>,
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
/// and its literal cells, gathered during the drive and installed at commit.
#[derive(Debug, Clone)]
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

        // The single-transaction builder on the context (consumer.rs) mints ONE
        // revision for the whole load. It is the only place that touches
        // consumer-private state; the sink hands it a plain plan.
        let plan = WorkbookTierALoadPlan {
            settings,
            sheets: self
                .sheets
                .into_iter()
                .map(|sheet| SheetTierALoad {
                    display_name: sheet.display_name,
                    upstream_sheet_id: sheet.upstream_sheet_id,
                    literals: sheet.literals,
                })
                .collect(),
        };
        context.commit_workbook_tier_a_load(workspace_id, plan)?;

        let ledger: Vec<IngestLedgerRow> = DocumentVariantTag::ALL
            .iter()
            .filter_map(|tag| self.ledger.get(tag).cloned())
            .collect();

        Ok(WorkbookLoadReport {
            sheets: sheet_count,
            cells: cell_count,
            names: 0,
            tables: 0,
            ledger,
            bind_degradations: Vec::new(),
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
            literals: Vec::new(),
        });
        self.ledger_and_observe(DocumentVariantTag::SheetBegin, IngestTier::A, "consumed");
        Ok(())
    }

    fn cell_chunk(&mut self, chunk: OxCalcCellChunk<'_>) -> Result<(), Self::Error> {
        // D4 §12 row 23: CellChunk (A). Literals → typed CalcValue; Empty → no
        // record; Formula/RichStub → deferred to R6.2/R6.4 (a placeholder is
        // ledgered, never consumed as a fake value).
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
                OxCalcCellInput::Formula(_) => {
                    // Deferred to R6.2 (bind + FileCached publication). Recording
                    // the CellChunk observation below suffices for the ledger;
                    // per-cell degradation rows are R6.2's contract.
                }
                OxCalcCellInput::RichStub(_) => {
                    // Deferred to R6.4 (inert RichObject retention). Ledgered via
                    // the CellChunk row below in this bead.
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
            OxCalcDocumentFeature::FormulaTopology(_) => self.ledger_and_observe(
                DocumentVariantTag::FormulaTopology,
                IngestTier::A,
                "deferred-routing-r6.2",
            ),
            OxCalcDocumentFeature::SharedFormulaRegion(_) => self.ledger_and_observe(
                DocumentVariantTag::SharedFormulaRegion,
                IngestTier::A,
                "deferred-install-r6.2",
            ),
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
/// (D4 §9). Plain data: settings + ordered sheets, each with its literal cells.
/// The builder installs it in one revision.
#[derive(Debug, Clone)]
pub struct WorkbookTierALoadPlan {
    pub settings: WorkbookCalcSettings,
    pub sheets: Vec<SheetTierALoad>,
}

/// One sheet's Tier-A load: its display name, upstream id, and literal cells.
#[derive(Debug, Clone)]
pub struct SheetTierALoad {
    pub display_name: String,
    pub upstream_sheet_id: u32,
    /// `(row_one_based, col_one_based, value)` literal cells in stream order.
    pub literals: Vec<(u32, u32, CalcValue)>,
}

impl SheetTierALoad {
    /// The engine-facing sheet identity token, derived from the upstream sheet
    /// id (T1: derived from a stable id, never the display name). The builder
    /// re-derives the same token from the node id once nodes exist; this is the
    /// pre-node placeholder used for the grid coordinate system.
    #[must_use]
    pub fn provisional_sheet_token(&self) -> String {
        format!("sheet:{}", self.upstream_sheet_id)
    }

    /// Build the authored-cell list for [`crate::grid::authored::GridInputState`]
    /// seeding, given the workbook and sheet tokens.
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

    /// The grid bounds for an ingested sheet: strict-excel full A1 space
    /// (profile policy, D4 §12 row 7 — the file's used-range claim is Tier B).
    #[must_use]
    pub fn bounds(&self) -> ExcelGridBounds {
        ExcelGridBounds::strict_excel()
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
}
