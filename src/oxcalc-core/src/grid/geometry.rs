//! Profile-pure grid geometry: resolved rectangles in absolute grid space and
//! the structured-table descriptions built from them. Shared by both grid
//! engines, carrying no storage and no resolution behavior.
//!
//! This module is the single home for grid rectangle algebra: the three
//! historically parallel rectangle types are now unified into [`GridRect`].

use crate::grid::coords::{ExcelGridBounds, ExcelGridCellAddress};
use crate::grid::error::GridRefError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExcelGridStructuredTableColumn {
    pub column_name: String,
    pub ordinal: u32,
    pub data_rect: GridRect,
}

impl ExcelGridStructuredTableColumn {
    #[must_use]
    pub fn new(column_name: impl Into<String>, ordinal: u32, data_rect: GridRect) -> Self {
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
    pub table_range: GridRect,
    pub header_rect: Option<GridRect>,
    pub totals_rect: Option<GridRect>,
    pub columns: Vec<ExcelGridStructuredTableColumn>,
}

impl ExcelGridStructuredTable {
    #[must_use]
    pub fn new(
        table_name: impl Into<String>,
        table_range: GridRect,
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
    pub fn with_header_rect(mut self, header_rect: GridRect) -> Self {
        self.header_rect = Some(header_rect);
        self
    }

    #[must_use]
    pub fn with_totals_rect(mut self, totals_rect: GridRect) -> Self {
        self.totals_rect = Some(totals_rect);
        self
    }
}

/// A validated grid rectangle in absolute coordinates: the single rectangle
/// type used at the grid reference boundary and inside the optimized engine.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct GridRect {
    pub workbook_id: String,
    pub sheet_id: String,
    pub top_row: u32,
    pub left_col: u32,
    pub bottom_row: u32,
    pub right_col: u32,
}

impl GridRect {
    pub fn new(
        workbook_id: impl Into<String>,
        sheet_id: impl Into<String>,
        row_a: u32,
        col_a: u32,
        row_b: u32,
        col_b: u32,
        bounds: ExcelGridBounds,
    ) -> Result<Self, GridRefError> {
        let top_row = row_a.min(row_b);
        let bottom_row = row_a.max(row_b);
        let left_col = col_a.min(col_b);
        let right_col = col_a.max(col_b);
        if !bounds.contains_row(top_row)
            || !bounds.contains_row(bottom_row)
            || !bounds.contains_col(left_col)
            || !bounds.contains_col(right_col)
        {
            return Err(GridRefError::RangeOutOfBounds {
                top_row,
                left_col,
                bottom_row,
                right_col,
                max_rows: bounds.max_rows,
                max_cols: bounds.max_cols,
            });
        }
        Ok(Self {
            workbook_id: workbook_id.into(),
            sheet_id: sheet_id.into(),
            top_row,
            left_col,
            bottom_row,
            right_col,
        })
    }

    /// The top-left cell of this rect as an [`ExcelGridCellAddress`] — the anchor
    /// a repeated-formula region binds its R1C1-relative key at.
    #[must_use]
    pub fn top_left(&self) -> ExcelGridCellAddress {
        ExcelGridCellAddress::new(
            self.workbook_id.clone(),
            self.sheet_id.clone(),
            self.top_row,
            self.left_col,
        )
    }

    #[must_use]
    pub const fn row_count(&self) -> u32 {
        self.bottom_row - self.top_row + 1
    }

    #[must_use]
    pub const fn col_count(&self) -> u32 {
        self.right_col - self.left_col + 1
    }

    #[must_use]
    pub fn cell_count(&self) -> u64 {
        u64::from(self.row_count()) * u64::from(self.col_count())
    }

    #[must_use]
    pub fn contains(&self, address: &ExcelGridCellAddress) -> bool {
        address.workbook_id == self.workbook_id
            && address.sheet_id == self.sheet_id
            && self.top_row <= address.row
            && address.row <= self.bottom_row
            && self.left_col <= address.col
            && address.col <= self.right_col
    }

    pub(crate) fn check_workbook_sheet(
        &self,
        expected_workbook_id: &str,
        expected_sheet_id: &str,
    ) -> Result<(), GridRefError> {
        if self.workbook_id != expected_workbook_id || self.sheet_id != expected_sheet_id {
            return Err(GridRefError::AddressOnDifferentSheet {
                expected_workbook_id: expected_workbook_id.to_string(),
                expected_sheet_id: expected_sheet_id.to_string(),
                actual_workbook_id: self.workbook_id.clone(),
                actual_sheet_id: self.sheet_id.clone(),
            });
        }
        Ok(())
    }

    pub(crate) fn scalar_cells(
        &self,
        limit: u64,
    ) -> Result<Vec<ExcelGridCellAddress>, GridRefError> {
        let cell_count = self.cell_count();
        if cell_count > limit {
            return Err(GridRefError::RangeTooLargeForScalarInvalidation {
                cells: cell_count,
                limit,
            });
        }
        let mut cells = Vec::with_capacity(usize::try_from(cell_count).unwrap_or(usize::MAX));
        for row in self.top_row..=self.bottom_row {
            for col in self.left_col..=self.right_col {
                cells.push(ExcelGridCellAddress::new(
                    self.workbook_id.clone(),
                    self.sheet_id.clone(),
                    row,
                    col,
                ));
            }
        }
        Ok(cells)
    }
}
