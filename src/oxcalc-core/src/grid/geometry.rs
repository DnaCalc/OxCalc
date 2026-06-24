//! Profile-pure grid geometry: resolved rectangles in absolute grid space and
//! the structured-table descriptions built from them. Shared by both grid
//! engines, carrying no storage and no resolution behavior.
//!
//! This module is the intended single home for grid rectangle algebra; the
//! historically parallel rectangle types are consolidated here over the course
//! of the grid module decomposition.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExcelGridResolvedRect {
    pub workbook_id: String,
    pub sheet_id: String,
    pub top_row: u32,
    pub left_col: u32,
    pub bottom_row: u32,
    pub right_col: u32,
}

impl ExcelGridResolvedRect {
    #[must_use]
    pub const fn row_count(&self) -> u32 {
        self.bottom_row - self.top_row + 1
    }

    #[must_use]
    pub const fn col_count(&self) -> u32 {
        self.right_col - self.left_col + 1
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExcelGridStructuredTableColumn {
    pub column_name: String,
    pub ordinal: u32,
    pub data_rect: ExcelGridResolvedRect,
}

impl ExcelGridStructuredTableColumn {
    #[must_use]
    pub fn new(
        column_name: impl Into<String>,
        ordinal: u32,
        data_rect: ExcelGridResolvedRect,
    ) -> Self {
        Self {
            column_name: column_name.into(),
            ordinal,
            data_rect,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExcelGridStructuredTable {
    pub table_name: String,
    pub table_range: ExcelGridResolvedRect,
    pub header_rect: Option<ExcelGridResolvedRect>,
    pub totals_rect: Option<ExcelGridResolvedRect>,
    pub columns: Vec<ExcelGridStructuredTableColumn>,
}

impl ExcelGridStructuredTable {
    #[must_use]
    pub fn new(
        table_name: impl Into<String>,
        table_range: ExcelGridResolvedRect,
        columns: Vec<ExcelGridStructuredTableColumn>,
    ) -> Self {
        Self {
            table_name: table_name.into(),
            table_range,
            header_rect: None,
            totals_rect: None,
            columns,
        }
    }

    #[must_use]
    pub fn with_header_rect(mut self, header_rect: ExcelGridResolvedRect) -> Self {
        self.header_rect = Some(header_rect);
        self
    }

    #[must_use]
    pub fn with_totals_rect(mut self, totals_rect: ExcelGridResolvedRect) -> Self {
        self.totals_rect = Some(totals_rect);
        self
    }
}
