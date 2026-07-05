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
    RefError {
        workbook_id: String,
        sheet_id: String,
        source_text: String,
        reason: String,
    },
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
    Insert { before: u32, count: u32 },
    Delete { first: u32, count: u32 },
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
        }
    }

    #[must_use]
    pub fn with_formula_anchor_after(mut self, anchor: ExcelGridFormulaAnchor) -> Self {
        self.formula_anchor_after = Some(anchor);
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
