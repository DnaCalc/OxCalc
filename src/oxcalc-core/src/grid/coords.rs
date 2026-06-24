//! Profile-pure grid coordinate primitives shared by both grid engines: grid
//! bounds, reference style, axis references, and cell addresses. These carry no
//! storage and no engine-specific behavior — they are the spec coordinate
//! vocabulary of the `strict-excel-grid` profile.

use serde::{Deserialize, Serialize};

pub const STRICT_EXCEL_MAX_ROWS: u32 = 1_048_576;
pub const STRICT_EXCEL_MAX_COLS: u32 = 16_384;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ExcelGridBounds {
    pub max_rows: u32,
    pub max_cols: u32,
}

impl ExcelGridBounds {
    #[must_use]
    pub const fn strict_excel() -> Self {
        Self {
            max_rows: STRICT_EXCEL_MAX_ROWS,
            max_cols: STRICT_EXCEL_MAX_COLS,
        }
    }

    #[must_use]
    pub const fn contains_row(self, row: u32) -> bool {
        1 <= row && row <= self.max_rows
    }

    #[must_use]
    pub const fn contains_col(self, col: u32) -> bool {
        1 <= col && col <= self.max_cols
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExcelGridReferenceStyle {
    A1,
    R1C1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExcelGridAxisRef {
    Absolute(u32),
    Relative(i32),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExcelGridCellAddress {
    pub workbook_id: String,
    pub sheet_id: String,
    pub row: u32,
    pub col: u32,
}

impl ExcelGridCellAddress {
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
