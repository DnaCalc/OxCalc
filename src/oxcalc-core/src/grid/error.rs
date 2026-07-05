//! The grid reference-system error type, shared across the grid engines and
//! their cross-cutting subsystems (storage, invalidation, structural edits,
//! recalc).

use thiserror::Error;

use crate::grid::coords::{ExcelGridBounds, ExcelGridCellAddress};
use crate::grid::machine::WorkbookCalcNodeId;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum GridRefError {
    #[error("grid address R{row}C{col} is outside bounds {max_rows}x{max_cols}")]
    AddressOutOfBounds {
        row: u32,
        col: u32,
        max_rows: u32,
        max_cols: u32,
    },
    #[error(
        "grid address belongs to {actual_workbook_id}/{actual_sheet_id}, expected {expected_workbook_id}/{expected_sheet_id}"
    )]
    AddressOnDifferentSheet {
        expected_workbook_id: String,
        expected_sheet_id: String,
        actual_workbook_id: String,
        actual_sheet_id: String,
    },
    #[error(
        "grid range R{top_row}C{left_col}:R{bottom_row}C{right_col} is outside bounds {max_rows}x{max_cols}"
    )]
    RangeOutOfBounds {
        top_row: u32,
        left_col: u32,
        bottom_row: u32,
        right_col: u32,
        max_rows: u32,
        max_cols: u32,
    },
    #[error(
        "grid range belongs to {actual_workbook_id}/{actual_sheet_id}, expected {expected_workbook_id}/{expected_sheet_id}"
    )]
    RangeOnDifferentSheet {
        expected_workbook_id: String,
        expected_sheet_id: String,
        actual_workbook_id: String,
        actual_sheet_id: String,
    },
    #[error(
        "foreign-sheet dependency edge routed into the {owning_workbook_id}/{owning_sheet_id} per-sheet index: address belongs to {actual_workbook_id}/{actual_sheet_id}"
    )]
    ForeignSheetDependency {
        owning_workbook_id: String,
        owning_sheet_id: String,
        actual_workbook_id: String,
        actual_sheet_id: String,
    },
    #[error("dense grid region has {cells} cells but {values} row-major values")]
    DenseRegionValueCountMismatch { cells: u64, values: usize },
    #[error("grid range has {cells} cells, above scalar invalidation limit {limit}")]
    RangeTooLargeForScalarInvalidation { cells: u64, limit: u64 },
    #[error("invalid grid structural edit: {detail}")]
    InvalidStructuralEdit { detail: String },
    #[error("invalid grid axis visibility dependency: {detail}")]
    InvalidAxisVisibilityDependency { detail: String },
    #[error("invalid grid axis value dependency: {detail}")]
    InvalidAxisValueDependency { detail: String },
    #[error("invalid grid defined name {name}")]
    InvalidDefinedName { name: String },
    #[error("grid defined name {name} was not found")]
    DefinedNameNotFound { name: String },
    #[error("grid defined name {name} already exists")]
    DefinedNameAlreadyExists { name: String },
    #[error("invalid grid table name {name}")]
    InvalidTableName { name: String },
    #[error("grid table overlay {name} was not found")]
    TableOverlayNotFound { name: String },
    #[error("grid table overlay {name} already exists")]
    TableOverlayAlreadyExists { name: String },
    #[error("feature-rendered region {feature_kind} refuses grid structural edit: {detail}")]
    FeatureRenderedRegionEditRefused {
        feature_kind: String,
        detail: String,
    },
    #[error("OxFml evaluation failed at {address:?}: {detail}")]
    OxfmlEvaluationFailed {
        address: ExcelGridCellAddress,
        detail: String,
    },
    #[error(
        "grid incremental recalc did not converge within {iteration_limit} formula evaluations"
    )]
    IncrementalRecalcDidNotConverge { iteration_limit: usize },
    #[error("grid effective dependency cycle detected: {cycle:?}")]
    EffectiveDependencyCycleDetected { cycle: Vec<ExcelGridCellAddress> },
    #[error("workbook effective dependency cycle detected: {cycle:?}")]
    WorkbookEffectiveDependencyCycleDetected { cycle: Vec<WorkbookCalcNodeId> },
    #[error("grid dynamic defined-name dependency cycle detected: {cycle:?}")]
    DynamicDefinedNameCycleDetected { cycle: Vec<String> },
    #[error("optimized grid warm no-op cache is stale for the current sheet state")]
    OptimizedWarmNoOpCacheStale,
    #[error(
        "optimized valuation grid identity mismatch: expected {expected_workbook_id}/{expected_sheet_id} {expected_bounds:?}, got {actual_workbook_id}/{actual_sheet_id} {actual_bounds:?}"
    )]
    ValuationGridIdentityMismatch {
        expected_workbook_id: String,
        expected_sheet_id: String,
        expected_bounds: ExcelGridBounds,
        actual_workbook_id: String,
        actual_sheet_id: String,
        actual_bounds: ExcelGridBounds,
    },
    #[error("grid formula structural transform failed at {address:?}: {detail}")]
    FormulaStructuralTransformFailed {
        address: ExcelGridCellAddress,
        detail: String,
    },
    #[error("grid formula reference enumeration failed at {address:?}: {detail}")]
    FormulaReferenceEnumerationFailed {
        address: ExcelGridCellAddress,
        detail: String,
    },
    #[error("grid reference provider failed: {detail}")]
    ReferenceProvider { detail: String },
    #[error(
        "optimized valuation has partial (visible-projection) coverage over {upstream_rect:?} and cannot seed a dirty recalc; escalate to mark-all instead"
    )]
    PartialValuationCoverage {
        upstream_rect: crate::grid::geometry::GridRect,
    },
}
