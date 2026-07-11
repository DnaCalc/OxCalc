//! Profile-pure grid reference syntax tree and structural-edit description
//! types. These are pure data shared across the grid engines and the OxFml
//! binding seam — they carry no storage and no resolution behavior.

use oxfml_core::binding::ProfilePayload;
use serde::{Deserialize, Serialize};

use crate::grid::coords::{ExcelGridAxisRef, ExcelGridReferenceStyle};

pub const EXCEL_GRID_STRUCTURAL_EDIT_PAYLOAD_KIND: &str = "excel-grid-structural-edit.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ExcelGridReference {
    Cell {
        workbook_id: String,
        sheet_id: String,
        row: ExcelGridAxisRef,
        col: ExcelGridAxisRef,
        source_style: ExcelGridReferenceStyle,
        source_text: String,
        parsed_qualifier: Option<String>,
    },
    Area {
        workbook_id: String,
        sheet_id: String,
        start_row: ExcelGridAxisRef,
        start_col: ExcelGridAxisRef,
        end_row: ExcelGridAxisRef,
        end_col: ExcelGridAxisRef,
        source_style: ExcelGridReferenceStyle,
        source_text: String,
        parsed_qualifier: Option<String>,
    },
    WholeRow {
        workbook_id: String,
        sheet_id: String,
        start_row: ExcelGridAxisRef,
        end_row: ExcelGridAxisRef,
        source_style: ExcelGridReferenceStyle,
        source_text: String,
        parsed_qualifier: Option<String>,
    },
    WholeColumn {
        workbook_id: String,
        sheet_id: String,
        start_col: ExcelGridAxisRef,
        end_col: ExcelGridAxisRef,
        source_style: ExcelGridReferenceStyle,
        source_text: String,
        parsed_qualifier: Option<String>,
    },
    SpillAnchor {
        workbook_id: String,
        sheet_id: String,
        anchor_key: String,
        source_text: String,
        parsed_qualifier: Option<String>,
    },
    StructuredReference {
        workbook_id: String,
        sheet_id: String,
        source_text: String,
        parsed_qualifier: Option<String>,
    },
    Name {
        workbook_id: String,
        sheet_id: String,
        name: String,
        source_text: String,
        parsed_qualifier: Option<String>,
    },
    /// A 3D sheet-span reference `Sheet1:Sheet3!A1` (W062 D2 §4.2, W078). A
    /// contiguous run of sheets — identified by the rename-immune
    /// [`start_sheet`](Self::SheetSpan::start_sheet)/
    /// [`end_sheet`](Self::SheetSpan::end_sheet) identity tokens (§10) — that
    /// qualifies a single cell/area `target`. This is a **distinct** reference
    /// class: it is never lowered to an [`Area`](Self::Area) or a same-sheet
    /// multi-area union (mirrors OxFml's `NormalizedReference::SheetSpan3D`,
    /// NOTES_FOR_OXFUNC 7.2 items 4-5).
    ///
    /// **Rect ignore-rule (§4.2).** The span deliberately carries the authored
    /// `target` text rather than a sheet-embedding [`super::geometry::GridRect`]:
    /// the member sheets are a closure-time function of the *current* sheet
    /// order (D3/R4.12), so no single sheet identity belongs on the target. The
    /// normal-form key's `{rect}` component is this sheet-agnostic `target`
    /// text; the key never enumerates member sheets (§10) so it survives every
    /// sheet insert/move/delete inside the span.
    ///
    /// R3.9 binds, keys, renders, and serdes the span. Closure expansion against
    /// C3 sheet order and evaluation are **R4.12** — a bound span evaluates to a
    /// typed-pending `#REF!` until then (never a silently-wrong value).
    SheetSpan {
        workbook_id: String,
        start_sheet: String,
        end_sheet: String,
        target: String,
        source_text: String,
        parsed_qualifier: Option<String>,
    },
    RefError {
        workbook_id: String,
        sheet_id: String,
        source_text: String,
        reason: String,
    },
}

impl ExcelGridReference {
    /// The `{workbook}` component this reference carries. For an ordinary
    /// in-workbook reference this is the caller's own workbook id; for an
    /// **external** reference (`[Book2]Sheet1!A1`) it is the dormant-external
    /// identity token `extbook:{normalized_alias}` (W062 D2 §5/§10,
    /// [`crate::reference_vocabulary::ExternalBookToken`]). A router (and R6.5's
    /// ingest external-pin detector) tells an external reference from a local one
    /// by testing this component with
    /// [`ExternalBookToken::is_external_component`](crate::reference_vocabulary::ExternalBookToken::is_external_component).
    ///
    /// The `SheetSpan` variant is a 3D span, never external-workbook-qualified in
    /// R6 (its `workbook_id` is always the caller's own), so it returns its own
    /// component like any local reference.
    #[must_use]
    pub fn workbook_component(&self) -> &str {
        match self {
            ExcelGridReference::Cell { workbook_id, .. }
            | ExcelGridReference::Area { workbook_id, .. }
            | ExcelGridReference::WholeRow { workbook_id, .. }
            | ExcelGridReference::WholeColumn { workbook_id, .. }
            | ExcelGridReference::SpillAnchor { workbook_id, .. }
            | ExcelGridReference::StructuredReference { workbook_id, .. }
            | ExcelGridReference::Name { workbook_id, .. }
            | ExcelGridReference::SheetSpan { workbook_id, .. }
            | ExcelGridReference::RefError { workbook_id, .. } => workbook_id,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExcelGridStructuralEditAxis {
    Row,
    Column,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ExcelGridStructuralEditKind {
    Insert {
        before: u32,
        count: u32,
    },
    Delete {
        first: u32,
        count: u32,
    },
    /// The whole sheet named by [`ExcelGridStructuralEdit::sheet_id`] was
    /// deleted (W062 D2 §6 / contract V7). This is a container-level structural
    /// edit, not an axis edit: the strict-excel `HardRefError` policy makes
    /// every reference *targeting* that sheet a destructive `#REF!` transform
    /// (a `FullyInvalid` outcome carrying a `RefError` record), Excel-faithful,
    /// with no heal-on-recreate. The deleted sheet's rename-immune identity is
    /// the edit's `sheet_id` component (the same
    /// [`crate::reference_vocabulary::SheetIdentityToken`] a normal-form key's
    /// sheet component carries, §10); the [`ExcelGridStructuralEdit::axis`]
    /// field is meaningless for this kind and ignored. Enumerated axis indices
    /// (`before`/`first`) do not apply — a sheet deletion removes the container
    /// wholesale, not a row/column band.
    SheetDeleted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExcelGridStructuralEdit {
    pub workbook_id: String,
    pub sheet_id: String,
    pub axis: ExcelGridStructuralEditAxis,
    pub kind: ExcelGridStructuralEditKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExcelGridFormulaAnchor {
    pub workbook_id: String,
    pub sheet_id: String,
    pub row: u32,
    pub col: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExcelGridReferenceTransformPayload {
    pub edit: ExcelGridStructuralEdit,
    pub formula_anchor_before: Option<ExcelGridFormulaAnchor>,
    pub formula_anchor_after: Option<ExcelGridFormulaAnchor>,
    /// The workbook's sheet display names in C3 order **as they stand before the
    /// edit is applied** (W062 R4.12, D2 §6 / V7). Used only by the 3D sheet-span
    /// endpoint delete/shrink transform: to move a deleted endpoint to the
    /// nearest surviving sheet that was inside the old span, the transform needs
    /// the pre-deletion order (a single reference record has no registry). Empty
    /// (the serde default) for every non-span edit and for span edits where the
    /// caller supplies no order — in which case the span endpoint-shrink falls
    /// back to the terminal-collapse cases resolvable from identity alone.
    #[serde(default)]
    pub sheet_order_before_edit: Vec<String>,
}

impl ExcelGridFormulaAnchor {
    #[must_use]
    pub fn new(
        workbook_id: impl Into<String>,
        sheet_id: impl Into<String>,
        row: u32,
        col: u32,
    ) -> Self {
        Self {
            workbook_id: workbook_id.into(),
            sheet_id: sheet_id.into(),
            row,
            col,
        }
    }
}

impl ExcelGridReferenceTransformPayload {
    #[must_use]
    pub fn new(
        edit: ExcelGridStructuralEdit,
        formula_anchor_before: Option<ExcelGridFormulaAnchor>,
    ) -> Self {
        Self {
            edit,
            formula_anchor_before,
            formula_anchor_after: None,
            sheet_order_before_edit: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_formula_anchor_after(mut self, anchor: ExcelGridFormulaAnchor) -> Self {
        self.formula_anchor_after = Some(anchor);
        self
    }

    /// Attach the pre-edit sheet order for the 3D sheet-span endpoint
    /// delete/shrink transform (W062 R4.12, D2 §6 / V7). See
    /// [`Self::sheet_order_before_edit`].
    #[must_use]
    pub fn with_sheet_order_before_edit(
        mut self,
        sheet_order: impl IntoIterator<Item = String>,
    ) -> Self {
        self.sheet_order_before_edit = sheet_order.into_iter().collect();
        self
    }

    #[must_use]
    pub fn into_profile_payload(self) -> ProfilePayload {
        ProfilePayload {
            payload_kind: EXCEL_GRID_STRUCTURAL_EDIT_PAYLOAD_KIND.to_string(),
            encoding: "json".to_string(),
            data: serde_json::to_string(&self)
                .expect("excel grid structural edit payload serializes"),
        }
    }
}

impl ExcelGridStructuralEdit {
    #[must_use]
    pub fn insert_rows(
        workbook_id: impl Into<String>,
        sheet_id: impl Into<String>,
        before: u32,
        count: u32,
    ) -> Self {
        Self {
            workbook_id: workbook_id.into(),
            sheet_id: sheet_id.into(),
            axis: ExcelGridStructuralEditAxis::Row,
            kind: ExcelGridStructuralEditKind::Insert { before, count },
        }
    }

    #[must_use]
    pub fn delete_rows(
        workbook_id: impl Into<String>,
        sheet_id: impl Into<String>,
        first: u32,
        count: u32,
    ) -> Self {
        Self {
            workbook_id: workbook_id.into(),
            sheet_id: sheet_id.into(),
            axis: ExcelGridStructuralEditAxis::Row,
            kind: ExcelGridStructuralEditKind::Delete { first, count },
        }
    }

    #[must_use]
    pub fn insert_columns(
        workbook_id: impl Into<String>,
        sheet_id: impl Into<String>,
        before: u32,
        count: u32,
    ) -> Self {
        Self {
            workbook_id: workbook_id.into(),
            sheet_id: sheet_id.into(),
            axis: ExcelGridStructuralEditAxis::Column,
            kind: ExcelGridStructuralEditKind::Insert { before, count },
        }
    }

    #[must_use]
    pub fn delete_columns(
        workbook_id: impl Into<String>,
        sheet_id: impl Into<String>,
        first: u32,
        count: u32,
    ) -> Self {
        Self {
            workbook_id: workbook_id.into(),
            sheet_id: sheet_id.into(),
            axis: ExcelGridStructuralEditAxis::Column,
            kind: ExcelGridStructuralEditKind::Delete { first, count },
        }
    }

    /// A whole-sheet deletion structural edit (W062 D2 §6, contract V7). The
    /// `sheet_id` is the deleted sheet's rename-immune identity (its
    /// [`crate::reference_vocabulary::SheetIdentityToken`] string / normal-form
    /// sheet component, §10) — the strict profile's transform makes every
    /// reference whose target sheet equals it a hard `#REF!`. The `axis` field
    /// is a don't-care for this kind (there is no row/column band); `Row` is
    /// stored as a stable placeholder.
    #[must_use]
    pub fn delete_sheet(workbook_id: impl Into<String>, sheet_id: impl Into<String>) -> Self {
        Self {
            workbook_id: workbook_id.into(),
            sheet_id: sheet_id.into(),
            axis: ExcelGridStructuralEditAxis::Row,
            kind: ExcelGridStructuralEditKind::SheetDeleted,
        }
    }
}
