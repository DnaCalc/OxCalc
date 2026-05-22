#![forbid(unsafe_code)]

//! Sparse worksheet/reference reader substrate for W051.

use std::cell::Cell;
use std::collections::BTreeMap;

use oxfunc_core::value::EvalValue;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SparseCellCoord {
    pub row: u32,
    pub column: u32,
}

impl SparseCellCoord {
    #[must_use]
    pub const fn new(row: u32, column: u32) -> Self {
        Self { row, column }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SparseRangeExtent {
    pub start: SparseCellCoord,
    pub row_count: u32,
    pub column_count: u32,
}

impl SparseRangeExtent {
    #[must_use]
    pub const fn new(start: SparseCellCoord, row_count: u32, column_count: u32) -> Self {
        Self {
            start,
            row_count,
            column_count,
        }
    }

    #[must_use]
    pub fn contains(self, coord: SparseCellCoord) -> bool {
        let Some(row_end) = self.start.row.checked_add(self.row_count) else {
            return false;
        };
        let Some(column_end) = self.start.column.checked_add(self.column_count) else {
            return false;
        };

        coord.row >= self.start.row
            && coord.row < row_end
            && coord.column >= self.start.column
            && coord.column < column_end
    }

    #[must_use]
    pub fn declared_cell_count(self) -> u64 {
        u64::from(self.row_count) * u64::from(self.column_count)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SparseReaderIdentity {
    pub reader_id: String,
    pub source_identity: String,
    pub snapshot_identity: String,
}

impl SparseReaderIdentity {
    #[must_use]
    pub fn new(
        reader_id: impl Into<String>,
        source_identity: impl Into<String>,
        snapshot_identity: impl Into<String>,
    ) -> Self {
        Self {
            reader_id: reader_id.into(),
            source_identity: source_identity.into(),
            snapshot_identity: snapshot_identity.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SparseDefinedCell {
    pub coord: SparseCellCoord,
    pub value: EvalValue,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SparseCellRead {
    Defined(EvalValue),
    Blank,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SparseReaderAccessSummary {
    pub contains_calls: usize,
    pub read_at_calls: usize,
    pub defined_iter_calls: usize,
    pub defined_iter_yield_count: usize,
}

#[derive(Debug, Default)]
struct SparseReaderAccessTelemetry {
    contains_calls: Cell<usize>,
    read_at_calls: Cell<usize>,
    defined_iter_calls: Cell<usize>,
    defined_iter_yield_count: Cell<usize>,
}

impl SparseReaderAccessTelemetry {
    fn record_contains(&self) {
        self.contains_calls.set(self.contains_calls.get() + 1);
    }

    fn record_read_at(&self) {
        self.read_at_calls.set(self.read_at_calls.get() + 1);
    }

    fn record_defined_iter(&self) {
        self.defined_iter_calls
            .set(self.defined_iter_calls.get() + 1);
    }

    fn record_defined_yield(&self) {
        self.defined_iter_yield_count
            .set(self.defined_iter_yield_count.get() + 1);
    }

    fn summary(&self) -> SparseReaderAccessSummary {
        SparseReaderAccessSummary {
            contains_calls: self.contains_calls.get(),
            read_at_calls: self.read_at_calls.get(),
            defined_iter_calls: self.defined_iter_calls.get(),
            defined_iter_yield_count: self.defined_iter_yield_count.get(),
        }
    }
}

pub trait SparseRangeReader {
    fn reader_identity(&self) -> &SparseReaderIdentity;
    fn declared_extent(&self) -> SparseRangeExtent;
    fn defined_cardinality(&self) -> usize;
    fn defined_iter(&self) -> Box<dyn Iterator<Item = SparseDefinedCell> + '_>;
    fn read_at(&self, coord: SparseCellCoord) -> SparseCellRead;
    fn contains(&self, coord: SparseCellCoord) -> bool;
}

#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum SparseReaderError {
    #[error(
        "defined cell ({row},{column}) is outside declared sparse extent starting at ({start_row},{start_column}) with {row_count} rows and {column_count} columns"
    )]
    DefinedCellOutsideExtent {
        row: u32,
        column: u32,
        start_row: u32,
        start_column: u32,
        row_count: u32,
        column_count: u32,
    },
}

#[derive(Debug)]
pub struct WorksheetSparseRangeReader {
    identity: SparseReaderIdentity,
    extent: SparseRangeExtent,
    defined_cells: BTreeMap<SparseCellCoord, EvalValue>,
    telemetry: SparseReaderAccessTelemetry,
}

impl WorksheetSparseRangeReader {
    pub fn new(
        identity: SparseReaderIdentity,
        extent: SparseRangeExtent,
        defined_cells: impl IntoIterator<Item = (SparseCellCoord, EvalValue)>,
    ) -> Result<Self, SparseReaderError> {
        let mut indexed = BTreeMap::new();
        for (coord, value) in defined_cells {
            if !extent.contains(coord) {
                return Err(SparseReaderError::DefinedCellOutsideExtent {
                    row: coord.row,
                    column: coord.column,
                    start_row: extent.start.row,
                    start_column: extent.start.column,
                    row_count: extent.row_count,
                    column_count: extent.column_count,
                });
            }
            indexed.insert(coord, value);
        }

        Ok(Self {
            identity,
            extent,
            defined_cells: indexed,
            telemetry: SparseReaderAccessTelemetry::default(),
        })
    }

    #[must_use]
    pub fn access_summary(&self) -> SparseReaderAccessSummary {
        self.telemetry.summary()
    }
}

impl SparseRangeReader for WorksheetSparseRangeReader {
    fn reader_identity(&self) -> &SparseReaderIdentity {
        &self.identity
    }

    fn declared_extent(&self) -> SparseRangeExtent {
        self.extent
    }

    fn defined_cardinality(&self) -> usize {
        self.defined_cells.len()
    }

    fn defined_iter(&self) -> Box<dyn Iterator<Item = SparseDefinedCell> + '_> {
        self.telemetry.record_defined_iter();
        Box::new(self.defined_cells.iter().map(|(coord, value)| {
            self.telemetry.record_defined_yield();
            SparseDefinedCell {
                coord: *coord,
                value: value.clone(),
            }
        }))
    }

    fn read_at(&self, coord: SparseCellCoord) -> SparseCellRead {
        self.telemetry.record_read_at();
        if !self.extent.contains(coord) {
            return SparseCellRead::Blank;
        }
        self.defined_cells
            .get(&coord)
            .cloned()
            .map_or(SparseCellRead::Blank, SparseCellRead::Defined)
    }

    fn contains(&self, coord: SparseCellCoord) -> bool {
        self.telemetry.record_contains();
        self.extent.contains(coord)
    }
}

#[cfg(test)]
mod tests {
    use oxfunc_core::value::{EvalValue, ExcelText};

    use super::{
        SparseCellCoord, SparseCellRead, SparseDefinedCell, SparseRangeExtent, SparseRangeReader,
        SparseReaderError, SparseReaderIdentity, WorksheetSparseRangeReader,
    };

    fn identity() -> SparseReaderIdentity {
        SparseReaderIdentity::new(
            "reader:sheet1:a:c",
            "worksheet:Sheet1!A:C",
            "snapshot:worksheet:v1",
        )
    }

    #[test]
    fn worksheet_sparse_reader_preserves_defined_and_blank_cells() {
        let reader = WorksheetSparseRangeReader::new(
            identity(),
            SparseRangeExtent::new(SparseCellCoord::new(1, 1), 10, 3),
            [
                (SparseCellCoord::new(1, 1), EvalValue::Number(10.0)),
                (
                    SparseCellCoord::new(5, 2),
                    EvalValue::Text(ExcelText::from_interop_assignment("")),
                ),
                (SparseCellCoord::new(10, 3), EvalValue::Logical(false)),
            ],
        )
        .unwrap();

        assert_eq!(reader.declared_extent().declared_cell_count(), 30);
        assert_eq!(reader.defined_cardinality(), 3);
        assert!(reader.contains(SparseCellCoord::new(10, 3)));
        assert!(!reader.contains(SparseCellCoord::new(11, 3)));
        assert_eq!(
            reader.read_at(SparseCellCoord::new(1, 1)),
            SparseCellRead::Defined(EvalValue::Number(10.0))
        );
        assert_eq!(
            reader.read_at(SparseCellCoord::new(5, 2)),
            SparseCellRead::Defined(EvalValue::Text(ExcelText::from_interop_assignment("")))
        );
        assert_eq!(
            reader.read_at(SparseCellCoord::new(3, 2)),
            SparseCellRead::Blank
        );
        assert_eq!(
            reader.read_at(SparseCellCoord::new(11, 3)),
            SparseCellRead::Blank
        );
    }

    #[test]
    fn worksheet_sparse_reader_iterates_defined_cells_deterministically() {
        let reader = WorksheetSparseRangeReader::new(
            identity(),
            SparseRangeExtent::new(SparseCellCoord::new(1, 1), 20, 20),
            [
                (SparseCellCoord::new(10, 4), EvalValue::Number(10.0)),
                (SparseCellCoord::new(2, 4), EvalValue::Number(2.0)),
                (SparseCellCoord::new(2, 2), EvalValue::Number(1.0)),
            ],
        )
        .unwrap();

        let cells = reader.defined_iter().collect::<Vec<_>>();
        assert_eq!(
            cells,
            vec![
                SparseDefinedCell {
                    coord: SparseCellCoord::new(2, 2),
                    value: EvalValue::Number(1.0),
                },
                SparseDefinedCell {
                    coord: SparseCellCoord::new(2, 4),
                    value: EvalValue::Number(2.0),
                },
                SparseDefinedCell {
                    coord: SparseCellCoord::new(10, 4),
                    value: EvalValue::Number(10.0),
                },
            ]
        );
    }

    #[test]
    fn worksheet_sparse_reader_rejects_defined_cells_outside_extent() {
        let err = WorksheetSparseRangeReader::new(
            identity(),
            SparseRangeExtent::new(SparseCellCoord::new(1, 1), 1, 1),
            [(SparseCellCoord::new(2, 1), EvalValue::Number(1.0))],
        )
        .unwrap_err();

        assert_eq!(
            err,
            SparseReaderError::DefinedCellOutsideExtent {
                row: 2,
                column: 1,
                start_row: 1,
                start_column: 1,
                row_count: 1,
                column_count: 1,
            }
        );
    }

    #[test]
    fn worksheet_sparse_reader_exposes_non_dense_large_range_evidence() {
        let reader = WorksheetSparseRangeReader::new(
            SparseReaderIdentity::new(
                "reader:sheet1:a:a",
                "worksheet:Sheet1!A:A",
                "snapshot:large:v1",
            ),
            SparseRangeExtent::new(SparseCellCoord::new(1, 1), 1_048_576, 1),
            [
                (SparseCellCoord::new(1, 1), EvalValue::Number(1.0)),
                (SparseCellCoord::new(1_048_576, 1), EvalValue::Number(2.0)),
            ],
        )
        .unwrap();

        assert_eq!(reader.declared_extent().declared_cell_count(), 1_048_576);
        assert_eq!(reader.defined_cardinality(), 2);

        let sum = reader
            .defined_iter()
            .map(|cell| match cell.value {
                EvalValue::Number(value) => value,
                _ => 0.0,
            })
            .sum::<f64>();

        assert_eq!(sum, 3.0);
        assert_eq!(reader.access_summary().defined_iter_calls, 1);
        assert_eq!(reader.access_summary().defined_iter_yield_count, 2);
        assert_eq!(reader.access_summary().read_at_calls, 0);
        assert_eq!(reader.access_summary().contains_calls, 0);
    }
}
